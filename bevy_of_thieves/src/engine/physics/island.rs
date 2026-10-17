//! WP-E6: multi-body physics step — broadphase 0x134040 (SAP 0x133e80 / pair toggle 0x133a38), islands 0x1f8f80
//! (flood fill 0x1f8e90, gather 0x1f90d0, mode filter 0x1f9240), island solver 0x1f9410 (rows 0x1585d8/0x158668,
//! LCP 0x158b20/0x158988, impulse accumulate + clamp 0x158e20) and the driver order of 0x1f8dd8.
//! 1:1 (structure + constants): SAP insertion sort with per-axis overlap bits; island flood skipping fully static
//! bodies (e8+0xc0 == 3 && e8+0xa0 == 3 -> `fixed`); mode-1 skip when every xf is static (asleep);
//! 0x1f9240 mode-4 gate (some gap < -2) and mode-2 gate (some n.v_rel < -5); mode 4 b = gap + 1 and bound 2*max|gap|,
//! angular bound 2pi (0x40c90fdb); mode 1/2 linear bound 2*sqrt(max|v_rel|^2), angular bound 50 (0x42480000);
//! impulse = sum x_i * row_i into +0xf0/+0x100 then 0x11e9f8 clamp; island-wide sleep (all bodies under the
//! 0x1f9410 thresholds -> 0x1f9c48 on every body).
//! REPLACED: narrowphase 0x1aa1d8 body/body is sphere/sphere (bodies are spheres here; the original walks the two
//! SOs' collision geoms through 0x1a8b50/0x1a9d38); the LCP pivoting (0x158988/0x11a770) is 0x11ad30 (PGS, same
//! complementary solution); the per-row responses come from mass `m` and the solid-sphere inertia 2/5 m r^2
//! (the original reads the SO's mass/inertia tensor; not yet located); mode-2 restitution (+0x98 * rand(0.9,1.1))
//! uses e = 0 (deterministic). Mode-4 position correction is applied as pos += x*row (slot 0x21c).
#![allow(non_snake_case)]
use super::driver::{near_zero, sleep_test, AloPhys, FUN_001f3980, FUN_001f9c48, FUN_001f9e70, FUN_0011b570};
use super::geom::{dot, BspC, CGeom, V3};
use super::bsp::FUN_00118668;
use super::driven::{vmul, transpose, GameRng};
use super::sphere::{FUN_0011ad30, FUN_0011e9f8, FUN_001a93c8, Sbi};
use super::CollisionWorld;
use std::collections::HashMap;

fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn scale(a: V3, s: f32) -> V3 {
    [a[0] * s, a[1] * s, a[2] * s]
}
fn cross(a: V3, b: V3) -> V3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

/// One simulated SO: ALO physics state plus its (sphere) collision shape.
#[derive(Clone, Debug)]
pub struct RigidBody {
    pub alo: AloPhys,
    pub radius: f32,
    pub mass: f32,
    /// e8+0xc0 == 3 && e8+0xa0 == 3: never joins an island as a member (0x1f8e90 / 0x1f8f80)
    pub fixed: bool,
    /// WP-E7: real collision mesh (body-local CGeom + BSP) for 0x1aa1d8 mesh/mesh; None = sphere
    pub mesh: Option<std::sync::Arc<(CGeom, BspC)>>,
    /// WP-E7: contact +0x98 restitution e (mode 2 b = n.v_rel (1 + e rand(0.9, 1.1)))
    pub restitution: f32,
}

impl RigidBody {
    pub fn sphere(pos: V3, radius: f32, mass: f32) -> Self {
        Self { alo: AloPhys { pos, ..Default::default() }, radius, mass, fixed: false, mesh: None, restitution: 0.0 }
    }
    fn inv_mass(&self) -> f32 {
        if self.fixed { 0.0 } else { 1.0 / self.mass }
    }
    fn inv_inertia(&self) -> f32 {
        if self.fixed { 0.0 } else { 2.5 / (self.mass * self.radius * self.radius) }
    }
    fn vel_at(&self, p: V3) -> V3 {
        add(self.alo.v, cross(self.alo.w, sub(p, self.alo.pos)))
    }
}

/// 0x133e80 / 0x133a38: sweep-and-prune over three axes. Endpoints are (value, body, is_max); swapping a min
/// left past another body's max starts overlap on that axis, a max left past a min ends it. Pairs live while all
/// three bits are set (the SO pair list +0xec+100).
#[derive(Default, Debug)]
pub struct Broadphase {
    axes: [Vec<(f32, usize, bool)>; 3],
    bits: HashMap<(usize, usize), u8>,
}

impl Broadphase {
    fn key(a: usize, b: usize) -> (usize, usize) {
        if a < b { (a, b) } else { (b, a) }
    }
    /// 0x134040's 0x133e80 part: refresh endpoints, insertion sort, toggle pairs (0x133a38).
    pub fn update(&mut self, bodies: &[RigidBody]) {
        let n = bodies.len();
        if self.axes[0].len() != 2 * n {
            // first frame / body set changed: rebuild from scratch
            self.bits.clear();
            for ax in 0..3 {
                let mut l: Vec<(f32, usize, bool)> = (0..n).flat_map(|i| [(0.0, i, false), (0.0, i, true)]).collect();
                for e in l.iter_mut() {
                    let b = &bodies[e.1];
                    e.0 = b.alo.pos[ax] + if e.2 { b.radius + CONTACT_SLOP } else { -b.radius - CONTACT_SLOP };
                }
                l.sort_by(|a, b| a.0.total_cmp(&b.0));
                self.axes[ax] = l;
            }
            for i in 0..n {
                for j in i + 1..n {
                    let mut m = 0u8;
                    for ax in 0..3 {
                        let (a, b) = (&bodies[i], &bodies[j]);
                        if (a.alo.pos[ax] - b.alo.pos[ax]).abs() <= a.radius + b.radius + 2.0 * CONTACT_SLOP {
                            m |= 1 << ax;
                        }
                    }
                    self.bits.insert((i, j), m);
                }
            }
        }
        for ax in 0..3 {
            let list = &mut self.axes[ax];
            for e in list.iter_mut() {
                let b = &bodies[e.1];
                e.0 = b.alo.pos[ax] + if e.2 { b.radius + CONTACT_SLOP } else { -b.radius - CONTACT_SLOP };
            }
            for k in 1..list.len() {
                let mut i = k;
                while i > 0 && list[i].0 < list[i - 1].0 {
                    let (cur, prev) = (list[i], list[i - 1]);
                    if cur.1 != prev.1 {
                        let bit = self.bits.entry(Self::key(cur.1, prev.1)).or_insert(0);
                        if !cur.2 && prev.2 {
                            *bit |= 1 << ax;
                        } else if cur.2 && !prev.2 {
                            *bit &= !(1 << ax);
                        }
                    }
                    list.swap(i, i - 1);
                    i -= 1;
                }
            }
        }
    }
    /// Pairs overlapping on all three axes.
    pub fn pairs(&self) -> Vec<(usize, usize)> {
        let mut v: Vec<_> = self.bits.iter().filter(|e| *e.1 == 7).map(|e| *e.0).collect();
        v.sort();
        v
    }
}

/// A contact (0x60-byte DZ row source): body `a` is pushed along `n`, `b` (None = level) against it.
#[derive(Clone, Debug)]
pub struct Contact {
    pub a: usize,
    pub b: Option<usize>,
    pub n: V3,
    pub p: V3,
    /// +0x90 signed gap (negative = penetration)
    pub gap: f32,
}

/// 0x1aa1d8 (REPLACED: sphere/sphere). Normal points from b to a.
pub fn FUN_001aa1d8(bodies: &[RigidBody], a: usize, b: usize) -> Option<Contact> {
    let (ba, bb) = (&bodies[a], &bodies[b]);
    if ba.mesh.is_some() || bb.mesh.is_some() {
        return mesh_contact(bodies, a, b);
    }
    let d = sub(ba.alo.pos, bb.alo.pos);
    let l = dot(d, d).sqrt();
    let gap = l - ba.radius - bb.radius;
    if gap > CONTACT_SLOP || l < 1e-6 {
        return None;
    }
    let n = scale(d, 1.0 / l);
    Some(Contact { a, b: Some(b), n, p: sub(ba.alo.pos, scale(n, ba.radius + gap * 0.5)), gap })
}
/// WP-E7 mesh narrowphase (REPLACED 0x1aa1d8's 0x1a8b50/0x1a9d38 walk): every vertex of one body's mesh is
/// taken into the other's local frame and the segment (own centre -> vertex) is clipped against the other's BSP
/// (0x118668, max 0x10); an entry before the vertex is a penetration, gap = signed distance of the vertex to the
/// entry plane. The deepest of both directions is the pair contact (normal from b to a). Sphere bodies use their
/// centre/radius as a one-point mesh.
fn mesh_contact(bodies: &[RigidBody], a: usize, b: usize) -> Option<Contact> {
    let mut best: Option<Contact> = None;
    for (p, q, sgn) in [(a, b, 1.0f32), (b, a, -1.0)] {
        let (bp, bq) = (&bodies[p], &bodies[q]);
        let Some(mq) = bq.mesh.as_ref() else { continue };
        let (g, bsp) = (&mq.0, &mq.1);
        let to_q = |w: V3| vmul(sub(w, bq.alo.pos), &transpose(&bq.alo.rot));
        let c = to_q(bp.alo.pos);
        let verts: Vec<V3> = match bp.mesh.as_ref() {
            Some(mp) => mp.0.verts.iter().map(|&v| add(vmul(v, &bp.alo.rot), bp.alo.pos)).collect(),
            None => {
                // sphere: deepest point towards q's centre
                let d = sub(bq.alo.pos, bp.alo.pos);
                let l = dot(d, d).sqrt().max(1e-6);
                vec![add(bp.alo.pos, scale(d, bp.radius / l))]
            }
        };
        for w in verts {
            let v = to_q(w);
            let mut lsg = Vec::new();
            if FUN_00118668(g, bsp, 0, bsp.nodes.len(), c, v, 0x10, &mut lsg) == 0 {
                continue;
            }
            let e = &lsg[0];
            if e.surf[0].is_none() || e.t[0] >= 1.0 {
                continue;
            }
            let nl = [e.plane[0][0], e.plane[0][1], e.plane[0][2]];
            let nw = vmul(nl, &bq.alo.rot); // q-local -> world
            // normal from b to a: q's outward normal points towards p
            let n = scale(nw, sgn);
            let gap = dot(nl, v) - e.plane[0][3];
            if best.as_ref().map_or(true, |bc| gap < bc.gap) {
                best = Some(Contact { a, b: Some(b), n, p: w, gap });
            }
        }
    }
    best
}

/// Contacts are kept while within this gap (resting contact needs touching bodies to stay in the island).
pub const CONTACT_SLOP: f32 = 1.0;

/// Level contacts of one body (the +0xec+0x68 list, narrowphase 0x1a93c8 per collider, max 0x40).
fn level_contacts(cw: &CollisionWorld, bodies: &[RigidBody], a: usize, out: &mut Vec<Contact>) {
    let b = &bodies[a];
    let mut sbi: Vec<Sbi> = Vec::new();
    for col in &cw.colliders {
        let left = 0x40usize.saturating_sub(sbi.len());
        if left == 0 {
            break;
        }
        FUN_001a93c8(b.radius, b.alo.pos, &col.geom, &col.bsp, left, &mut sbi);
    }
    for s in sbi {
        let n = [s.plane[0], s.plane[1], s.plane[2]];
        let gap = dot(n, b.alo.pos) - s.plane[3] - b.radius;
        out.push(Contact { a, b: None, n, p: sub(b.alo.pos, scale(n, b.radius)), gap });
    }
}

/// Response rows of a contact (0x60 B: per body linear +0x00 and angular +0x10 change per unit impulse).
fn rows(bodies: &[RigidBody], c: &Contact) -> [(Option<usize>, V3, V3); 2] {
    let row = |k: usize, s: f32| {
        let bk = &bodies[k];
        let j = scale(c.n, s);
        (Some(k), scale(j, bk.inv_mass()), scale(cross(sub(c.p, bk.alo.pos), j), bk.inv_inertia()))
    };
    [row(c.a, 1.0), c.b.map(|b| row(b, -1.0)).unwrap_or((None, [0.0; 3], [0.0; 3]))]
}

/// 0x1f9410 for one island and one mode bit (1 resting, 2 collision, 4 penetration).
pub fn FUN_001f9410(bodies: &mut [RigidBody], island: &[usize], cs: &[Contact], mode: u32, dt: f32) {
    FUN_001f9410_rng(bodies, island, cs, mode, dt, &mut GameRng::default())
}

/// WP-E7: 0x1f9410 with the game RNG for the mode-2 restitution factor (one 0x11d448(0.9, 1.1) draw per contact
/// with e != 0; e = max of the two bodies' `restitution` (contact +0x98; per-contact source not located)).
pub fn FUN_001f9410_rng(bodies: &mut [RigidBody], island: &[usize], cs: &[Contact], mode: u32, dt: f32, rng: &mut GameRng) {
    let n = cs.len();
    let r: Vec<_> = cs.iter().map(|c| rows(bodies, c)).collect();
    // relative normal velocity change at contact i per unit impulse at j
    let resp = |bodies: &[RigidBody], i: usize, j: usize| -> f32 {
        let c = &cs[i];
        let mut dv = 0.0;
        for (k, lin, ang) in r[j].iter() {
            let Some(k) = *k else { continue };
            for (side, s) in [(Some(c.a), 1.0), (c.b, -1.0)] {
                if side == Some(k) {
                    dv += s * dot(c.n, add(*lin, cross(*ang, sub(c.p, bodies[k].alo.pos))));
                }
            }
        }
        dv
    };
    let mut a = vec![0.0f32; n * n];
    for i in 0..n {
        for j in 0..n {
            a[i * n + j] = resp(bodies, i, j);
        }
    }
    let vrel = |bodies: &[RigidBody], c: &Contact| {
        let vb = c.b.map(|b| bodies[b].vel_at(c.p)).unwrap_or([0.0; 3]);
        dot(c.n, sub(bodies[c.a].vel_at(c.p), vb))
    };
    let (b, bound, abound): (Vec<f32>, f32, f32) = if mode == 4 {
        let b: Vec<f32> = cs.iter().map(|c| c.gap + 1.0).collect();
        let m = cs.iter().fold(0.0f32, |m, c| m.max(c.gap.abs()));
        (b, m + m, std::f32::consts::TAU)
    } else {
        let b: Vec<f32> = cs
            .iter()
            .map(|c| {
                let v = vrel(bodies, c);
                let e = bodies[c.a].restitution.max(c.b.map_or(0.0, |k| bodies[k].restitution));
                if mode == 2 && e != 0.0 { v * (1.0 + e * rng.FUN_0011d448(0.9, 1.1)) } else { v }
            })
            .collect();
        let m2 = island.iter().fold(0.0f32, |m, &k| m.max(dot(bodies[k].alo.v, bodies[k].alo.v)));
        let m = m2.sqrt().max(b.iter().fold(0.0f32, |m, x| m.max(x.abs())));
        (b, m + m, 50.0)
    };
    // x is an impulse (mass * velocity); the original's DZ rows are normalised so its bound is a velocity: scale by the
    // island mass so the bound acts on the velocity change (it is re-applied by the 0x11e9f8 clamps below).
    let mass: f32 = island.iter().map(|&k| bodies[k].mass).sum();
    let x = if n > 0 { FUN_0011ad30(bound * mass.max(1.0), &a, &b) } else { return };
    // 0x158e20: accumulate into +0xf0/+0x100 and clamp
    let mut jl: HashMap<usize, (V3, V3)> = HashMap::new();
    for (i, xi) in x.iter().enumerate() {
        for (k, lin, ang) in r[i].iter() {
            if let Some(k) = *k {
                let e = jl.entry(k).or_insert(([0.0; 3], [0.0; 3]));
                e.0 = add(e.0, scale(*lin, *xi));
                e.1 = add(e.1, scale(*ang, *xi));
            }
        }
    }
    for (k, (l, w)) in jl {
        let b = &mut bodies[k];
        if b.fixed {
            continue;
        }
        let (l, w) = (FUN_0011e9f8(bound, l), FUN_0011e9f8(abound, w));
        if mode == 4 {
            // slot 0x21c: positional correction
            b.alo.pos = add(b.alo.pos, l);
            if !near_zero(w) {
                let m = FUN_0011b570(w);
                let rot = b.alo.rot;
                b.alo.rot = [0, 1, 2].map(|i| add(add(scale(m[0], rot[i][0]), scale(m[1], rot[i][1])), scale(m[2], rot[i][2])));
            }
        } else {
            b.alo.imp_lin = add(b.alo.imp_lin, l);
            b.alo.imp_ang = add(b.alo.imp_ang, w);
            FUN_001f9e70(&mut b.alo, dt);
        }
    }
    if mode == 1 {
        // island-wide sleep: every body under the thresholds (and not already static kind 2)
        if island.iter().all(|&k| bodies[k].fixed || sleep_test(&bodies[k].alo)) {
            for &k in island {
                if !bodies[k].fixed {
                    FUN_001f9c48(&mut bodies[k].alo, dt);
                }
            }
        }
    }
}

/// Multi-body world (0x1f8dd8 state): bodies + persistent broadphase.
#[derive(Default, Debug)]
pub struct PhysWorld {
    pub bodies: Vec<RigidBody>,
    pub bp: Broadphase,
    /// WP-E7: game RNG state (newlib rand, seed 1)
    pub rng: GameRng,
}

impl PhysWorld {
    /// 0x134040: broadphase + narrowphase on overlapping pairs; returns body/body contacts.
    pub fn FUN_00134040(&mut self) -> Vec<Contact> {
        self.bp.update(&self.bodies);
        self.bp.pairs().into_iter().filter_map(|(i, j)| FUN_001aa1d8(&self.bodies, i, j)).collect()
    }

    /// 0x1f8f80(mode): flood islands over contact pairs (0x1f8e90), gather contacts (0x1f90d0), gate (0x1f9240),
    /// solve (0x1f9410). Returns the islands that were solved.
    pub fn FUN_001f8f80(&mut self, cw: &CollisionWorld, pair_cs: &[Contact], mode: u32, dt: f32) -> Vec<Vec<usize>> {
        let n = self.bodies.len();
        let mut adj = vec![Vec::new(); n];
        for c in pair_cs {
            let b = c.b.unwrap();
            adj[c.a].push(b);
            adj[b].push(c.a);
        }
        let mut seen = vec![false; n];
        let mut solved = Vec::new();
        for s in 0..n {
            if seen[s] || self.bodies[s].fixed {
                continue;
            }
            // mode 1: skip when the start body's xfs are static (asleep, 0x230348)
            if mode & 1 != 0 && self.bodies[s].alo.kind == 3 {
                continue;
            }
            // 0x1f8e90 flood fill (recursive in the original; explicit stack here)
            let mut island = Vec::new();
            let mut stack = vec![s];
            seen[s] = true;
            while let Some(k) = stack.pop() {
                island.push(k);
                for &o in &adj[k] {
                    if !seen[o] && !self.bodies[o].fixed {
                        seen[o] = true;
                        stack.push(o);
                    }
                }
            }
            // 0x1f90d0 gather: pair contacts touching the island + each member's level contacts, max 0x40
            let mut cs: Vec<Contact> = pair_cs.iter().filter(|c| island.contains(&c.a) || c.b.map_or(false, |b| island.contains(&b))).cloned().collect();
            for &k in &island {
                level_contacts(cw, &self.bodies, k, &mut cs);
            }
            cs.truncate(0x40);
            // 0x1f9240 mode gate
            let mut m = mode;
            if m & 1 == 0 {
                if m & 4 != 0 && !cs.iter().any(|c| c.gap < -2.0) {
                    m &= !4;
                }
                if m & 2 != 0 {
                    let vb = |c: &Contact| c.b.map(|b| self.bodies[b].vel_at(c.p)).unwrap_or([0.0; 3]);
                    if !cs.iter().any(|c| dot(c.n, sub(self.bodies[c.a].vel_at(c.p), vb(c))) < -5.0) {
                        m &= !2;
                    }
                }
            } else if cs.is_empty() {
                // mode 1 without contacts: nothing to solve, but 0x1f9410's sleep test still needs contacts -> skip
                m = 0;
            }
            if m == 0 {
                continue;
            }
            for bit in [4, 2, 1] {
                if m & bit != 0 {
                    if bit == 1 {
                        FUN_001f9410_rng(&mut self.bodies, &island, &cs, 1, dt, &mut self.rng);
                    } else {
                        // later passes see the updated state: re-evaluate gaps for mode 4
                        FUN_001f9410_rng(&mut self.bodies, &island, &cs, bit, dt, &mut self.rng);
                    }
                }
            }
            solved.push(island);
        }
        solved
    }

    /// 0x1f8dd8: slot16 per awake body, 0x134040, islands mode 1, 0x134040, islands mode 6.
    pub fn FUN_001f8dd8(&mut self, cw: &CollisionWorld, dt: f32) {
        for b in self.bodies.iter_mut() {
            if !b.fixed && !(b.alo.kind == 3 && near_zero(b.alo.imp_lin) && near_zero(b.alo.imp_ang)) {
                FUN_001f3980(&mut b.alo, dt);
            }
        }
        let cs = self.FUN_00134040();
        self.FUN_001f8f80(cw, &cs, 1, dt);
        let cs = self.FUN_00134040();
        self.FUN_001f8f80(cw, &cs, 6, dt);
    }

    pub fn momentum(&self) -> V3 {
        self.bodies.iter().filter(|b| !b.fixed).fold([0.0; 3], |m, b| add(m, scale(b.alo.v, b.mass)))
    }
}
