//! WP-E8: the PO ground contact motor and the single-body contact solve that turns it into motion.
//!
//! Original chain (per frame, inside 0x1f8dd8): slot16 0x1f3980 (forces 0x1f9b20 / 0x204108 + integrate
//! 0x22de88) -> narrowphase: the PO's custom-contact slot FUN_00204ac8 adds ONE kind-0xe record ("ground
//! motor contact") from a segment probe FUN_0014c6d8 -> record build 0x1a7c18 (side 0 = the PO's SO jt+0x258,
//! point P0 = *(jt+0x218), side 1 = the probed SO, point P0 - gap n) -> notify slot +0x1bc = FUN_00204250 (kind
//! 0xe: +0x94 friction = DAT_002c3c90 1.5 and impulse bounds +0xa0 = m * -4500, +0xa4 = m * 9000 on walkable
//! ground n.z > 0.7, else only +0xa4) -> 0x1a7fc0 side velocities + slot +0x1cc = FUN_002044b0 (the motor:
//! rewrites the PO side velocity so that "relative velocity 0" means "normal speed f3 + f2, tangential speed
//! -R(+0x1b0).(+0x190) * +0x180") -> island 0x1f9410 mode 1: one row per contact (0x158668; kind 0xe row
//! bounded [+0xa0 dt, +0xa4 dt], b = n.v_rel), solve 0x158988, then the TANGENTIAL rows of 0x158b20 (the
//! contact with the largest post-solve tangential slip > 0.1 gets a bilateral row along -slip, bound
//! mu * max(|lambda_n|, -g.z m dt) for kind 0xe (mode 1), mu |lambda_n| otherwise, min(m 1500, mu |lambda_n|) in
//! mode 2), re-solve, repeat) -> 0x158e20 clamp 2 sqrt(max |v_rel|^2) -> slot 0x20c apply (0x1f9e70).
//!
//! 1:1: FUN_002044b0, FUN_00204ac8 (except 0x2048c8, see below), FUN_00204250 kind-0xe part, the 0x1f9410
//! mode-1 row setup for kinds 0xb / 0xe / other, 0x158b20 tangential row selection and bounds, 0x158e20 clamp.
//! REPLACED: 0x158988's pivoting LCP (0x158728 / 0x158958 / 0x11a770) by projected Gauss-Seidel on the same
//! rows; the PO is one body with linear response only (response tables (+0x54) not ported: contact torques
//! are not applied, the PO's rotation is the 0x1f59a0 servo); FUN_0014c6d8 = first entry of
//! `CollisionWorld::clip_segment` (0x118668) along the caller's probe segment; FUN_002048c8 (walk the steep
//! face's adjacent faces for a walkable one within 1.45 (k210 - k248)) is approximated by "a walkable face
//! hit by the same probe", see `ee_sub_2048c8`.
#![allow(non_snake_case)]
use super::driver::{AloPhys, FUN_001f9b20, FUN_001f9e70, FUN_001fa328, FUN_001fac30, FUN_0022de88, M3};
use super::dynamics::FUN_0011d018;
use super::geom::{dot, sub, V3};
use super::sphere::{FUN_0011e9f8, FUN_001a93c8, Sbi};
use super::CollisionWorld;

/// DAT_002c3c90: kind-0xe friction (+0x94) on walkable ground.
pub const DAT_002C3C90: f32 = 1.5;
/// 0x2c3c98: 0x11d018 profile of the "towards jt+0x210" speed A.
pub const DAT_002C3C98: [f32; 3] = [3000.0, 0.0, 0.35];
/// 0x2c3ca8: 0x11d018 profile of the "towards gap 0" speed B.
pub const DAT_002C3CA8: [f32; 3] = [3000.0, 0.0, 0.5];
/// 0x2044b0 / 0x204250 acceleration bounds (x dt / x mass).
pub const MOTOR_UP: f32 = 9000.0;
pub const MOTOR_DOWN: f32 = -4500.0;
/// DAT_002bdde8: mode-2 tangential bound per unit mass.
pub const DAT_002BDDE8: f32 = 1500.0;
/// 0x204ac8 / 0x204250: walkable when n.z > 0.7.
pub const WALKABLE_Z: f32 = 0.7;
/// DAT_002a2dc0
pub const UP: V3 = [0.0, 0.0, 1.0];
/// 0x158b20: minimum squared tangential slip for a new row (fVar11 = 0.01).
pub const SLIP2_MIN: f32 = 0.01;
/// Gauss-Seidel sweeps per (re)solve (REPLACED pivoting).
pub const PGS_ITERS: usize = 24;

fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn scale(a: V3, s: f32) -> V3 {
    [a[0] * s, a[1] * s, a[2] * s]
}
/// rows * v (vmula/vmadda with rows +0x1b0 / +0x1c0 / +0x1d0): r0 v.x + r1 v.y + r2 v.z.
pub fn rows_mul(r: &M3, v: V3) -> V3 {
    add(add(scale(r[0], v[0]), scale(r[1], v[1])), scale(r[2], v[2]))
}

/// PO fields read by the motor (class 54 / 55 / JT PO layout).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PoMotor {
    /// +0x1b0 / +0x1c0 / +0x1d0 rows
    pub rows_1b0: M3,
    /// +0x190 local velocity goal (P22: -rate * blended root velocity)
    pub v190: V3,
    /// +0x180 speed scale
    pub k180: f32,
    /// +0x210 (hull min local z) / +0x248
    pub k210: f32,
    pub k248: f32,
    /// e8+0xd0 mass
    pub mass: f32,
    /// vt +0x2ac flags: bit 0 = add the motor contact, bit 3 = keep the probed normal
    pub flags_2ac: u32,
}

impl Default for PoMotor {
    fn default() -> Self {
        Self {
            rows_1b0: super::driver::IDENT,
            v190: [0.0; 3],
            k180: 1.0,
            k210: 0.0,
            k248: 0.0,
            mass: 1.0,
            flags_2ac: 1,
        }
    }
}

/// Contact record (XP) fields used here; side 0 / 1 records are 0x40 B each (+0x10 point, +0x30 velocity).
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Xp {
    pub point: [V3; 2],
    pub vel: [V3; 2],
    /// which side is the PO body (the other side is static level)
    pub po_side: usize,
    /// +0x80 normal (side 1 -> side 0)
    pub n: V3,
    /// +0x90
    pub gap: f32,
    /// +0x94 friction / +0x98 restitution
    pub f94: f32,
    pub f98: f32,
    /// +0x9c
    pub kind: i32,
    /// +0xa0 / +0xa4 (kind 0xe: impulse bounds per dt; else features)
    pub fa0: f32,
    pub fa4: f32,
    /// +0xb0 v0 - v1
    pub relvel: V3,
}

/// What FUN_0014c6d8 reports to 0x204ac8: probed point (auStack_c0), normal (uStack_a0 / fStack_98), the
/// t (fStack_80 > 0 required) and the surface feature (iStack_78, 0 = face).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GroundHit {
    pub point: V3,
    pub normal: V3,
    pub t: f32,
    pub feat: i32,
    /// surface index of the entry plane
    pub surf: Option<usize>,
}

/// 0x0014c6d8 (param_2 != 0 branch: 0x118668 segment clip). REPLACED source: the first entry along
/// p_from -> p_to over every collider (`CollisionWorld::clip_segment`); a start inside solid (no entry
/// plane) is not a hit.
pub fn FUN_0014c6d8(cw: &CollisionWorld, p_from: V3, p_to: V3) -> Option<GroundHit> {
    let mut best: Option<GroundHit> = None;
    for l in cw.clip_segment(p_from, p_to) {
        let pl = l.plane[0];
        let nrm = [pl[0], pl[1], pl[2]];
        if dot(nrm, nrm) < 0.5 || l.t[0] <= 0.0 {
            continue;
        }
        if best.map_or(true, |b| l.t[0] < b.t) {
            best = Some(GroundHit { point: l.pos[0], normal: nrm, t: l.t[0], feat: 0, surf: l.surf[0] });
        }
    }
    best
}

/// 0x002048c8 (approximated): the original walks the steep face's adjacent faces (edge lists +0x10 / +0x14)
/// for a walkable one (n.z > 0.7) within 1.45 (k210 - k248) of the probe point. Here: a second probe straight
/// down from the hit point by that reach finds a walkable face.
pub fn ee_sub_2048c8(cw: &CollisionWorld, m: &PoMotor, p0: V3, hit: &GroundHit) -> bool {
    let reach = 1.45 * (m.k210 - m.k248).abs();
    if reach <= 0.0 {
        return false;
    }
    let from = add(hit.point, scale(UP, 1.0));
    match FUN_0014c6d8(cw, from, add(from, scale(UP, -reach))) {
        Some(h) => h.normal[2] > WALKABLE_Z && dot(sub(p0, h.point), UP) <= reach,
        None => false,
    }
}

/// 0x00204ac8(po, other): the kind-0xe motor record, or None (flag bit 0 clear, no hit / t <= 0).
/// `walkable_near` = FUN_002048c8's answer (only asked for steep faces, feature 0, flag bit 3 clear).
pub fn FUN_00204ac8(m: &PoMotor, p0: V3, hit: Option<&GroundHit>, walkable_near: impl FnOnce(&GroundHit) -> bool) -> Option<Xp> {
    if m.flags_2ac & 1 == 0 {
        return None;
    }
    let h = hit?;
    if h.t <= 0.0 {
        return None;
    }
    let mut n = h.normal;
    if m.flags_2ac & 8 == 0 {
        // n.z <= 0.7: probed normal unless feature 0 and 0x2048c8 finds walkable ground; steep with a feature
        // keeps the probed normal; walkable -> up
        if h.normal[2] <= WALKABLE_Z {
            if h.feat == 0 && walkable_near(h) {
                n = UP;
            }
        } else {
            n = UP;
        }
    }
    let gap = dot(sub(p0, h.point), n);
    // 0x1a7c18(gap, .., side 0 = PO, side 1 = other, P0, n, 0xe, 0, 0)
    Some(Xp { point: [p0, sub(p0, scale(n, gap))], po_side: 0, n, gap, kind: 0xe, ..Default::default() })
}

/// 0x00204250(po, xp, side): PO notify. +0x94 = +0x98 = 0; kind 0xe: walkable (n.z > 0.7) -> +0x94 =
/// DAT_002c3c90, +0xa0 = m * -4500, +0xa4 = m * 9000; else +0xa4 = m * 9000. (The kind 3 / side-1 kinds
/// 1,4,5,6 secondary kind-10 record is player::p5_hull::FUN_00204250.)
pub fn FUN_00204250(m: &PoMotor, xp: &mut Xp) {
    xp.f94 = 0.0;
    xp.f98 = 0.0;
    if xp.kind == 0xe {
        if xp.n[2] > WALKABLE_Z {
            xp.f94 = DAT_002C3C90;
            xp.fa0 = m.mass * MOTOR_DOWN;
        }
        xp.fa4 = m.mass * MOTOR_UP;
    }
}

/// 0x002044b0(po, xp, side): contact motor (slot +0x1cc, kind 0xe only; 0x1f4fc0 base is empty).
pub fn FUN_002044b0(m: &PoMotor, xp: &mut Xp, side: usize, dt: f32) {
    if xp.kind != 0xe {
        return;
    }
    let n = xp.n;
    let mut f3 = dot(sub(xp.vel[0], xp.vel[1]), n);
    let mut a = 0.0f32;
    let mut b = 0.0f32;
    FUN_0011d018(xp.gap, m.k210, dt, DAT_002C3C98, Some(&mut a));
    FUN_0011d018(xp.gap, 0.0, dt, DAT_002C3CA8, Some(&mut b));
    if f3 < a {
        f3 = a;
    }
    let f2 = if f3 < b {
        (dt * MOTOR_UP).min(b - f3)
    } else {
        (dt * MOTOR_DOWN).max(b - f3)
    };
    let v = rows_mul(&m.rows_1b0, m.v190);
    let vt = sub(v, scale(n, dot(v, n)));
    let s = &mut xp.vel[side];
    *s = add(add(*s, scale(n, -(f3 + f2))), scale(vt, m.k180));
    xp.gap += m.k210 - m.k248;
}

/// 0x001a7fc0 for a PO record: side velocities (PO side = body point velocity, linear part only as after
/// 0x188e78; level side static = 0), slot +0x1cc (0x2044b0) on the PO side, +0xb0 = v0 - v1.
pub fn FUN_001a7fc0(m: &PoMotor, xp: &mut Xp, v_body: V3, dt: f32) {
    xp.vel = [[0.0; 3]; 2];
    xp.vel[xp.po_side] = v_body;
    FUN_002044b0(m, xp, xp.po_side, dt);
    xp.relvel = sub(xp.vel[0], xp.vel[1]);
}

/// One solver row (0x28 B: +0 type, +4 lo, +8 hi, +0x10 b, +0x1c friction link; direction at +0x40 of the
/// 0x60 B response block).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Row {
    /// 1 = bilateral (bounded), 0 = one-sided
    pub ty: i32,
    pub lo: f32,
    pub hi: f32,
    /// the record this row belongs to
    pub xp: usize,
    /// unit direction in the record's v_rel frame (normal, or -slip for a tangential row)
    pub dir: V3,
    /// accumulated impulse (+0xc)
    pub lambda: f32,
    /// +0x1c: tangential row of this normal row / normal row of this tangential row
    pub link: Option<usize>,
    pub tangential: bool,
}

/// Body response sign: v_rel = v0 - v1, the PO is side `po_side`.
fn sgn(xp: &Xp) -> f32 {
    if xp.po_side == 0 { 1.0 } else { -1.0 }
}

fn row_vrel(xps: &[Xp], r: &Row, dv: V3) -> f32 {
    let x = &xps[r.xp];
    dot(r.dir, add(x.relvel, scale(dv, sgn(x))))
}

/// 0x158988 REPLACED: projected Gauss-Seidel on the rows (target: row v_rel = 0), body linear only.
fn pgs(xps: &[Xp], rows: &mut [Row], mass: f32, dv: &mut V3) {
    let inv_m = 1.0 / mass;
    for _ in 0..PGS_ITERS {
        for r in rows.iter_mut() {
            let k = dot(r.dir, r.dir) * inv_m;
            if k <= 0.0 {
                continue;
            }
            let err = row_vrel(xps, r, *dv);
            let lo = if r.ty == 0 { r.lo.max(0.0) } else { r.lo };
            let new = (r.lambda - err / k).clamp(lo, r.hi.max(lo));
            let d = new - r.lambda;
            r.lambda = new;
            *dv = add(*dv, scale(r.dir, d * inv_m * sgn(&xps[r.xp])));
        }
    }
}

/// Result of `FUN_001f9410_po`.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct PoSolve {
    /// body velocity change written to +0xf0 (after the 0x158e20 clamp)
    pub dv: V3,
    pub rows: Vec<Row>,
}

/// 0x001f9410 for one PO against static level records: rows (0x158668), solve, tangential rows 0x158b20,
/// clamp 0x158e20 (bound 2 sqrt(max |v_rel|^2)). `mode` bit 0 = velocity pass (b = n.v_rel), else the
/// restitution pass (b scaled by 1 + e rand: not used for the motor; e = 0 here). `gz` = e8+0x88.
pub fn FUN_001f9410_po(xps: &[Xp], mass: f32, gz: f32, mode: u32, dt: f32) -> PoSolve {
    let mut rows: Vec<Row> = Vec::new();
    let mut vmax2 = 0.0f32;
    for (i, x) in xps.iter().enumerate() {
        let (ty, lo, hi) = match x.kind {
            0xb => (1, f32::MIN, f32::MAX),
            0xe if mode & 1 != 0 => ((x.fa0 < 0.0) as i32, x.fa0 * dt, x.fa4 * dt),
            0xe => (0, 0.0, f32::MAX),
            _ => (0, 0.0, f32::MAX),
        };
        rows.push(Row { ty, lo, hi, xp: i, dir: x.n, lambda: 0.0, link: None, tangential: false });
        vmax2 = vmax2.max(dot(x.relvel, x.relvel));
    }
    let mut dv = [0.0; 3];
    loop {
        pgs(xps, &mut rows, mass, &mut dv);
        // 0x158b20: the normal row without a tangential row, |lambda| >= 1e-4 and mu >= 1e-4, with the largest
        // post-solve tangential slip (> 0.1)
        let mut best: Option<(usize, V3, f32)> = None;
        let mut best2 = SLIP2_MIN;
        for (ri, r) in rows.iter().enumerate() {
            if r.tangential || r.link.is_some() || r.lambda < 0.0001 || xps[r.xp].f94 < 0.0001 {
                continue;
            }
            let x = &xps[r.xp];
            let v = add(x.relvel, scale(dv, sgn(x)));
            let vt = sub(v, scale(x.n, dot(v, x.n)));
            let l2 = dot(vt, vt);
            if best2 < l2 {
                best2 = l2;
                best = Some((ri, vt, l2));
            }
        }
        let Some((ri, vt, l2)) = best else { break };
        let len = l2.sqrt();
        let x = &xps[rows[ri].xp];
        let ln = rows[ri].lambda.abs();
        let bound = if x.kind == 0xe {
            if mode & 1 != 0 {
                x.f94 * ln.max(-gz * mass * dt)
            } else {
                (mass * DAT_002BDDE8).min(x.f94 * ln)
            }
        } else {
            ln * x.f94
        };
        let ti = rows.len();
        rows[ri].link = Some(ti);
        rows.push(Row { ty: 1, lo: -bound, hi: bound, xp: rows[ri].xp, dir: scale(vt, -1.0 / len), lambda: 0.0, link: Some(ri), tangential: true });
    }
    let bound = 2.0 * vmax2.sqrt();
    PoSolve { dv: FUN_0011e9f8(bound, dv), rows }
}

/// Level contacts of a PO body (REPLACED source, as player::p5_hull: 0x1a93c8 sphere against every collider):
/// side 0 = level, side 1 = PO, n from the PO towards the level side reversed (side 1 -> side 0 = -n_s),
/// gap = n_s.c - w - r. Kinds: vertex 1, edge 3, face 0xd.
pub fn level_xps(cw: &CollisionWorld, c: V3, r: f32, out: &mut Vec<Xp>) {
    for col in cw.colliders.iter().filter(|col| col.near(c, r)) {
        let mut sbi: Vec<Sbi> = Vec::new();
        FUN_001a93c8(r, c, &col.geom, &col.bsp, 0x40, &mut sbi);
        for s in &sbi {
            let ns = [s.plane[0], s.plane[1], s.plane[2]];
            let d = dot(ns, c) - s.plane[3];
            let p = sub(c, scale(ns, d));
            let kind = match s.kind {
                4 => 1,
                5 => 3,
                _ => 0xd,
            };
            let n = scale(ns, -1.0);
            out.push(Xp { point: [p, add(p, scale(ns, d - r))], po_side: 1, n, gap: d - r, kind, ..Default::default() });
        }
    }
}

/// FEEL R51: level hull contacts from the 1:1 mesh/mesh chain (so_image::mesh_pair = FUN_001aa1d8 ->
/// FUN_001a8b50 ...) of the PO's world-space hull SO (`geom` / `bsp`, body sphere `c` / `r`) against every
/// near collider; replaces `level_xps`' sphere (REPLACED source) when the caller supplies the hull. Level
/// body sphere = the collider's SO bounds box (centre, half diagonal), else its vertex bounds.
pub fn mesh_xps(cw: &CollisionWorld, geom: &super::geom::CGeom, bsp: &super::geom::BspC, c: V3, r: f32, out: &mut Vec<Xp>) {
    // FEEL R55: a collider with no vertices and no surfaces can only yield the kind-0xd fallback record, which
    // the island gather drops below, so it is skipped without running the chain (same output, much faster)
    for col in cw.colliders.iter().filter(|col| col.near(c, r) && !(col.geom.verts.is_empty() && col.geom.surfs.is_empty())) {
        let (lo, hi) = col.bbox.unwrap_or_else(|| {
            col.geom.verts.iter().fold(([f32::MAX; 3], [f32::MIN; 3]), |(l, h), v| (std::array::from_fn(|k| l[k].min(v[k])), std::array::from_fn(|k| h[k].max(v[k]))))
        });
        let lc = [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5, (lo[2] + hi[2]) * 0.5];
        let d = sub(hi, lo);
        let lr = dot(d, d).sqrt() * 0.5;
        let pairs = super::so_image::mesh_pair(geom, bsp, c, r, &col.geom, &col.bsp, lc, lr);
        if std::env::var_os("SLY2_FEEL_MESHDBG2").is_some() && pairs.is_empty() {
            eprintln!("MESHNEAR obj {} src {:?} nv {} ns {} ne {} bsp {} bbox {} sph {:?} lr {:.0}", col.object, col.source, col.geom.verts.len(), col.geom.surfs.len(), col.geom.edges.len(), col.bsp.nodes.len(), col.bbox.is_some(), col.sphere.map(|s| s.1 as i32), lr);
        }
        if std::env::var_os("SLY2_FEEL_MESHDBG").is_some() && !pairs.is_empty() {
            eprintln!("MESHCOL obj {} src {:?} nv {} ns {} bsp {} box {:?}..{:?} n {} kinds {:?} gaps {:?}", col.object, col.source, col.geom.verts.len(), col.geom.surfs.len(), col.bsp.nodes.len(), lo.map(|v| v as i32), hi.map(|v| v as i32), pairs.len(), pairs.iter().map(|p| p.4).collect::<Vec<_>>(), pairs.iter().map(|p| p.1 as i32).collect::<Vec<_>>());
        }
        if std::env::var_os("SLY2_FEEL_MESHDBG3").is_some() {
            for (h, g, p, n, k) in &pairs { eprintln!("MC3 obj {} h {} gap {:.1} k {} p {:?} n {:?}", col.object, h, g, k, p.map(|v| v as i32), n.map(|v| (v * 100.0) as i32)); }
        }
        for (hull_a, gap, p, n, kind) in pairs {
            // FEEL R52: FUN_001f90d0 (island gather) skips kind-0xd records (the 0x1a8b50 / 0x1aa1d8 "no contact"
            // fallback at the body centre, n DAT_002a2dc0): they never become solver rows
            if kind == 0xd {
                continue;
            }
            // FEEL R85 GUARD (not 1:1, source of the bad record unknown): a DEEP (gap < -40) hull-vertex / level-face
            // record (kind 1) whose vertex lies OUTSIDE the level BSP solid (FUN_00118fb0 None) is a phantom: EP1 hub
            // (-2150,-3690) got surf 8975's plane (a pole face 360 away) at gap -78 with no geometry at the vertex ->
            // mode-4 popped Sly back ~72 every 14 frames (USER 'random teleport'). SLY2_FEEL_R85VTX=0 = old.
            if kind == 1 && hull_a && gap < -40.0 && r85_vtx_guard() && !col.bsp.nodes.is_empty() && super::point::FUN_00118fb0(&col.geom, &col.bsp, 0, col.bsp.nodes.len(), p).is_none() {
                continue;
            }
            out.push(Xp { point: [p, add(p, scale(n, -gap))], po_side: if hull_a { 0 } else { 1 }, n, gap, kind, ..Default::default() });
        }
    }
}

/// FEEL R89: OFF by default: the phantom records came from the unwritten FUN_00118a88 sentinels (so_image::write_rodata,
/// SLY2_FEEL_R89SENT); SLY2_FEEL_R85VTX=1 re-enables the guard.
fn r85_vtx_guard() -> bool {
    std::env::var("SLY2_FEEL_R85VTX").is_ok_and(|v| v == "1")
}

/// Per-frame result of `po_step`.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct PoStep {
    /// the kind-0xe motor record (None = airborne / motor off)
    pub motor: Option<Xp>,
    /// every record solved this frame (motor first)
    pub xps: Vec<Xp>,
    pub solve: PoSolve,
}

/// One 0x1f8dd8 frame for a PO body driven by the contact motor: slot16 0x1f3980 (gravity / accel +
/// integrate; `accel` = the PO's own slot-69 forces, added through 0x1fac30 after gravity), probe 0x14c6d8 straight down from P0 + up * `probe.0`
/// to P0 - up * `probe.1` (P0 = pos + `p0_ofs` = jt+0x218, the foot point the motor keeps on the ground),
/// 0x204ac8 / 0x204250 / 0x1a7fc0 (+ motor), level hull contacts (`hull` = (centre offset from pos, radius),
/// REPLACED source as player::p5_hull), 0x1f9410 mode 1, slot 0x20c 0x1f9e70. No sleep: the PO xf is kind 2
/// (0x1f9410 never sleeps an island holding a kind-2 xf).
pub fn po_step(cw: &CollisionWorld, b: &mut AloPhys, m: &PoMotor, p0_ofs: V3, probe: (f32, f32), hull: Option<(V3, f32)>, accel: V3, dt: f32) -> PoStep {
    po_step_mesh(cw, b, m, p0_ofs, probe, hull, None, accel, dt, false)
}

/// FEEL R102: the bit-2 (collision) velocity pass of 0x1f8dd8's second island run. FEEL R113: DEFAULT ON (with the
/// R113GAP refresh it reproduces cap_feel_fall_step3_jumps' landing vz: no snap to 0, peak ~360, same decay; coordinator
/// decision R113: capture is ground truth). SLY2_FEEL_NO_R102BIT2=1 (or SLY2_FEEL_R102BIT2=0) = off (pre-R113 default).
pub fn r102_bit2() -> bool {
    static F: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *F.get_or_init(|| std::env::var_os("SLY2_FEEL_NO_R102BIT2").is_none() && std::env::var("SLY2_FEEL_R102BIT2").map_or(true, |v| v != "0"))
}

/// FEEL R67: the mode-4 (gap < -2) position pass of 0x1f8dd8's second island run; SLY2_FEEL_MODE4=0 turns it off.
pub fn po_mode4_on() -> bool {
    std::env::var("SLY2_FEEL_MODE4").map_or(true, |v| v != "0")
}

/// FEEL R73: the mode-4 pass for Bentley / Murray too: 0x1fa528 is vt+0x21c of all three players (0x2d2e98 /
/// 0x2d44c8 / 0x2cb230), so the original pushes POs out as well. Was Sly-only while the PO hulls sank 30-60 into
/// the floor (k210/k248 0/0, fixed R72); open-floor punch / jump unchanged with it. SLY2_FEEL_MODE4PO=0 = Sly only.
pub fn po_mode4_po_on() -> bool {
    std::env::var("SLY2_FEEL_MODE4PO").map_or(true, |v| v != "0")
}

/// `po_step` with an optional world-space mesh hull (geom, bsp, body centre, radius): when given, the level
/// hull contacts come from `mesh_xps` (1:1 FUN_001aa1d8 chain) instead of `level_xps`' sphere.
#[allow(clippy::too_many_arguments)]
pub fn po_step_mesh(cw: &CollisionWorld, b: &mut AloPhys, m: &PoMotor, p0_ofs: V3, probe: (f32, f32), hull: Option<(V3, f32)>, mesh: Option<(&super::geom::CGeom, &super::geom::BspC, V3, f32)>, accel: V3, dt: f32, mode4: bool) -> PoStep {
    // 0x1f3980: previous velocities, accel reset, slot 69 (0x1f9b20 gravity + the PO's own forces), 0x22de88
    b.prev_v = b.v;
    b.prev_w = b.w;
    if matches!(b.kind, 1 | 2) {
        b.a = [0.0; 3];
        b.alpha = [0.0; 3];
        FUN_001f9b20(b);
        FUN_001fac30(b, accel);
    }
    let dbg_in = (b.pos[2], b.v[2]);
    FUN_0022de88(b, dt);
    let p0 = add(b.pos, p0_ofs);
    let hit = FUN_0014c6d8(cw, add(p0, scale(UP, probe.0)), add(p0, scale(UP, -probe.1)));
    let mut xps = Vec::new();
    let motor = FUN_00204ac8(m, p0, hit.as_ref(), |h| ee_sub_2048c8(cw, m, p0, h));
    if let Some(mut x) = motor {
        FUN_00204250(m, &mut x);
        FUN_001a7fc0(m, &mut x, b.v, dt);
        xps.push(x);
    }
    if hull.is_some() || mesh.is_some() {
        let mut lv = Vec::new();
        match (mesh, hull) {
            (Some((g, bsp, c, r)), _) => mesh_xps(cw, g, bsp, c, r, &mut lv),
            (None, Some((ofs, r))) => level_xps(cw, add(b.pos, ofs), r, &mut lv),
            (None, None) => {}
        }
        for mut x in lv {
            FUN_00204250(m, &mut x);
            FUN_001a7fc0(m, &mut x, b.v, dt);
            xps.push(x);
        }
    }
    let solve = if xps.is_empty() { PoSolve::default() } else { FUN_001f9410_po(&xps, m.mass, b.gravity[2], 1, dt) };
    b.imp_lin = add(b.imp_lin, solve.dv);
    let dbg_int = (b.pos[2], b.v[2]);
    let p_m1 = b.pos;
    FUN_001f9e70(b, dt);
    if std::env::var_os("SLY2_FEEL_BIT2DBG").is_some() && !xps.is_empty() {
        eprintln!("M1 in z {:.2} vz {:.1} | int z {:.2} vz {:.1} | m1 z {:.2} vz {:.1} dv {:.1}", dbg_in.0, dbg_in.1, dbg_int.0, dbg_int.1, b.pos[2], b.v[2], solve.dv[2]);
    }
    // FEEL R102 (default since R113, opt-out SLY2_FEEL_NO_R102BIT2=1): 0x1f9410 mode 1 ends with 0x1a7e60 / 0x1a7fc0 (the motor 0x2044b0
    // rewrites +0xb0 with the NEW velocity); 0x1f8dd8's 0x1f8f80(6) gate 0x1f9240 keeps bit 2 when a non-0xb record
    // has n.(+0xb0) < -5, and 0x1f9410 mode 2 solves the 0xe row UNBOUNDED (lo 0, hi max; b = n.v_rel, e +0x98 = 0).
    // FEEL R103 said the apply slot 0x214 of 0x2d2e98 is 0x1f9e70 (class_slots slot 64); FEEL R115: wrong word, +0x20c is
    // 0x1f9e70 (mode 1), +0x214 is 0x1fa328 (this pass, velocity only), 0x21c = 0x1fa528.
    // Mode 2 itself does not refresh +0xb0 (no 0x1a7e60 / 0x1a7fc0 when bit 0 is clear): the records are the fresh
    // ones of the 0x134040 re-detection between the passes, whose 0x1a7c18 computes +0xb0 by 0x1a7fc0 at creation =
    // the post-mode-1 velocity, as refreshed here. The 50 of 0x158e20 is the angular bound (unused for the PO).
    if r102_bit2() && !xps.is_empty() {
        for x in xps.iter_mut() {
            // FEEL R113: the end-of-mode-1 refresh is 0x1a7e60 THEN 0x1a7fc0 (decomp 0x1f9410 +327): 0x1a7e60 recomputes
            // the side points and +0x90 = n.(p0 - p1) from the bodies (positions unchanged by the mode-1 velocity pass),
            // so the motor's k210 - k248 shift does not accumulate: undo the creation shift before 0x2044b0 runs again
            // (R102 refreshed the shifted gap: raw + 70). SLY2_FEEL_NO_R113GAP=1 = R102 behaviour.
            if x.kind == 0xe && std::env::var_os("SLY2_FEEL_NO_R113GAP").is_none() {
                x.gap -= m.k210 - m.k248;
            }
            FUN_001a7fc0(m, x, b.v, dt);
        }
        if std::env::var_os("SLY2_FEEL_BIT2DBG").is_some() {
            eprintln!("BIT2 pos {:?} v {:?} recs {:?}", b.pos.map(|v| (v * 10.0).round() / 10.0), b.v.map(|v| (v * 10.0).round() / 10.0), xps.iter().map(|x| (x.kind, (x.gap * 10.0).round() / 10.0, (dot(x.n, x.relvel) * 10.0).round() / 10.0, x.n.map(|v| (v * 100.0) as i32))).collect::<Vec<_>>());
        }
        if xps.iter().any(|x| x.kind != 0xb && dot(x.n, x.relvel) < -5.0) {
            let s2 = FUN_001f9410_po(&xps, m.mass, b.gravity[2], 2, dt);
            b.imp_lin = add(b.imp_lin, s2.dv);
            // FEEL R113 (capture-backed): the mode-2 velocity change does not move the body this frame.
            // cap_feel_fall_step3_jumps run landings: sampled v after the landing step 832 (= the mode-2 tangential
            // row's drive, reproduced here) but the step displacement * 60 = 628 = (515 + 740) / 2, i.e. no jl dt / 2
            // half-step for the mode-2 impulse. FEEL R115: 1:1 mechanism found: 0x1f9410's bit-0-clear branch calls vt
            // +0x214 = FUN_001fa328 (velocity-only apply; JT vt 0x2d2e98 words +0x20c 0x1f9e70 / +0x214 0x1fa328 /
            // +0x21c 0x1fa528 in SCUS_973.16), mode 1 calls +0x20c = 0x1f9e70 (with the half-step). Capture mid-frame
            // samples agree (clk 286.445: vz -359 -> -59, z +1.25 = only the mode-1 +150 half-stepped).
            // SLY2_FEEL_NO_R113HALF=1 = 0x1f9e70 half-step (pre-R113); SLY2_FEEL_NO_R115HALF=1 = R113's pos restore.
            if std::env::var_os("SLY2_FEEL_NO_R113HALF").is_some() {
                FUN_001f9e70(b, dt);
            } else if std::env::var_os("SLY2_FEEL_NO_R115HALF").is_some() {
                let p_keep = b.pos;
                FUN_001f9e70(b, dt);
                b.pos = p_keep;
            } else {
                FUN_001fa328(b);
            }
            if std::env::var_os("SLY2_FEEL_BIT2DBG").is_some() {
                eprintln!("M2 z {:.2} vz {:.1} dv {:.1}", b.pos[2], b.v[2], s2.dv[2]);
            }
        }
    }
    // FEEL R67: 0x1f8dd8 runs the islands again in mode 6 after the mode-1 velocity pass; the 0x1f9240 gate keeps
    // bit 4 when some contact gap < -2: 0x1f9410 mode 4 solves b = gap + 1 (rows n, lambda >= 0), clamps by
    // 0x158e20(2 max|gap|) and slot 0x21c (JT vt 0x2d2e98+0x21c = 0x1fa528) moves the body by it (velocity untouched).
    // The mode-6 gather sees the same positions (mode 1 only changes velocity), so the mode-1 records are reused.
    // Bit 2 (restitution, n.v_rel < -5) is a no-op here: e = 0 for the level records (+0x98 = 0, FUN_00204250).
    // The motor's own kind-0xe record is left out: its gap is shifted to the hover target by 0x2044b0
    // (+= k210 - k248), and with it the pass lifts Murray at take-off (946 vs the captured 990; see FEEL R67).
    // `mode4`: Sly, and Bentley / Murray unless SLY2_FEEL_MODE4PO=0 (po_mode4_po_on). SLY2_FEEL_MODE4E=1 (probe) keeps
    // the 0xe record in the pass.
    // FEEL R115GAP4: FUN_001f9410 (CSo island solve) mode 1 applies through vt +0x20c = FUN_001f9e70, which half-steps the
    // body (pos += dv dt / 2), then ends with FUN_001a7e60 (records' +0x90 = n.(p0 - p1) recomputed from the moved bodies);
    // the mode-6 call (bit 4 push-out at the top of FUN_001f9410, rows b = +0x90 for type 1 / +0x90 + 1 else, then the
    // bit-2 velocity pass) sees those refreshed gaps. The port reused the pre-apply gaps -> pushed the mode-1 half-step
    // out a second time. Level points are static, so the refresh is gap += sgn n.dp. SLY2_FEEL_NO_R115GAP4=1 = stale gaps.
    let gap4 = std::env::var_os("SLY2_FEEL_NO_R115GAP4").is_none();
    let dp1 = sub(b.pos, p_m1);
    let m4e = std::env::var_os("SLY2_FEEL_MODE4E").is_some();
    let g4 = |x: &Xp| if gap4 && x.kind != 0xe { x.gap + sgn(x) * dot(x.n, dp1) } else { x.gap };
    if mode4 && po_mode4_on() && xps.iter().any(|x| (m4e || x.kind != 0xe) && g4(x) < -2.0) {
        let mut px: Vec<Xp> = xps.iter().filter(|x| m4e || x.kind != 0xe).copied().collect();
        let mut gmax = 0.0f32;
        for x in px.iter_mut() {
            x.gap = g4(x);
            x.relvel = scale(x.n, x.gap + 1.0);
            gmax = gmax.max(x.gap.abs());
        }
        let mut rows: Vec<Row> = (0..px.len()).map(|i| Row { ty: 0, lo: 0.0, hi: f32::MAX, xp: i, dir: px[i].n, lambda: 0.0, link: None, tangential: false }).collect();
        let mut dp = [0.0; 3];
        pgs(&px, &mut rows, m.mass, &mut dp);
        let dp = FUN_0011e9f8(2.0 * gmax, dp);
        if std::env::var_os("SLY2_FEEL_MESHDBG3").is_some() {
            eprintln!("MODE4 pos {:?} dp {:?} recs {:?}", b.pos.map(|v| v as i32), dp.map(|v| (v * 10.0).round() / 10.0), xps.iter().map(|x| (x.kind, (x.gap * 10.0).round() / 10.0, x.n.map(|v| (v * 100.0) as i32))).collect::<Vec<_>>());
        }
        b.pos = add(b.pos, dp);
    }
    if std::env::var_os("SLY2_FEEL_BIT2DBG").is_some() {
        eprintln!("PO in z {:.2} vz {:.1} out z {:.2} vz {:.1} nx {}", dbg_in.0, dbg_in.1, b.pos[2], b.v[2], xps.len());
    }
    PoStep { motor: xps.first().filter(|x| x.kind == 0xe).copied(), xps, solve }
}
