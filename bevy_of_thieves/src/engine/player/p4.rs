//! WP-P4: action gate callee 0x173610, real ledge search 0x199d48 (+0x1999d0 / 0x199408), attack target list
//! (0x194688 insert, 0x194a40 build, 0x1938c0 ground cane pick, 0x1928f8 attach), edge forces 0x194170,
//! class is-a test 0x132b10, 0x188960 force dispatch for every jts.
//!
//! 1:1: FUN_00132b10 (table DAT_002baaf0 read from the ELF), FUN_00173610 (segment pos+up*d0 -> pos+up*d1,
//! mode 1 = records starting inside skipped, accept if t_in != 0 and (steep allowed or plane z >= 0.7),
//! writes the entry point), FUN_00199408 (ledge edge: one face z > 0.7, other < 0.55; direction = horizontal
//! edge perpendicular signed along the steep face normal), FUN_001999d0 face (kind 2) / edge (kind 3)
//! branches, FUN_00199d48 (kind / jtbs gates, contact walk, separating-contact skip DAT_002c3c60, probe
//! point = contact - 60 dir + 70 up (DAT_002bf8a4 / DAT_002bf8a8) outside the BSP, highest contact wins),
//! FUN_00194688 (4-slot candidate list ordered by priority desc / distance asc, group de-dup, 1000 =
//! DAT_002bf664 range), FUN_00194a40 list walk by category bits, FUN_001938c0, FUN_001928f8 field writes and
//! kind -> substate table, FUN_00194170.
//! REPLACED: SO contact list jt+0xec+100 -> FUN_001a93c8 sphere contacts of every collider at the JT centre
//! with radius JT_RADIUS + LEDGE_FWD (Sly's collision hull is not resolved; kinds 6/5/4 -> 2/3/1, the JT is
//! always contact side 0, so vertex contacts (kind 1, side 1 only) are never tested), contact +0xb8 =
//! v . n; the grab volume jt+600 (+0x74 BSP) clip of 0x1999d0 -> segment within LEDGE_GRAB_R of the centre;
//! FUN_001997d8 / FUN_001995d8 / FUN_0011e2d8: 1:1 since WP-P10 (11-segment clearance); FUN_0020faa0 / FUN_0014cc58 ->
//! CollisionWorld::clip_segment. Target categories / per-class target callbacks (+0x20 vtable +0x14/+0x1c/
//! +0x24) are supplied as `TargetCand` values by the caller (game side owns the object classes).
//! UNPORTED (recorded in Jt::calls): FUN_00192bd8 attach substate bodies (3.3 KB), FUN_0019b110 air target
//! pick bodies (rope/pole/point class geometry callbacks 0x1e7410/0x1e64c8/0x19c300/0x185d18),
//! FUN_00195368 / FUN_00194f70 (target sparkle + copy) and the music cue tail of 0x194a40, per-jts forces
//! 0x1903b8 / 0x1a1148 / 0x137248 / 0x19df70 / jts 0xe / 0x18.
#![allow(non_snake_case)]

use super::{add, scale, Jt, JtAnim, CALL_ATTACH, JT_RADIUS, LEDGE_FWD};
use crate::engine::physics::geom::{dot, CGeom, V3};
use crate::engine::physics::point::FUN_00118fb0;
use crate::engine::physics::sphere::FUN_001a93c8;
use crate::engine::physics::bsp::FUN_00118668;
use crate::engine::physics::driver::FUN_0011b500;
use crate::engine::physics::CollisionWorld;

/// JMP +0x108 / +0x10c / +0x110 / +0x114 / +0x118 / +0x11c (FUN_00195950 JMP init; no other writer): the
/// clearance segment coordinates of FUN_001995d8 (x along the ledge direction, z up).
pub const JMP_108: [f32; 6] = [-60.0, -50.0, 75.0, 50.0, -150.0, 75.0];
/// 0x3e860a92: 15 degrees, fan step of FUN_001995d8.
const DEG15: f32 = f32::from_bits(0x3e860a92);

fn cross(a: V3, b: V3) -> V3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

/// 0x0011e2d8(dir, up, alt, out[2]): out[0] = n(up x dir) (alt x dir when |up x dir|^2 < 1e-4),
/// out[1] = n(dir x out[0]).
pub fn FUN_0011e2d8(dir: V3, up: V3, alt: V3) -> (V3, V3) {
    let mut s = cross(up, dir);
    if dot(s, s) < 0.0001 {
        s = cross(alt, dir);
    }
    let u = cross(dir, s);
    (scale(s, 1.0 / dot(s, s).sqrt()), scale(u, 1.0 / dot(u, u).sqrt()))
}

/// WP-P17: JMP +0xf0 hang offset (FUN_00195950 init from DAT_002bf740).
pub const JMP_F0: V3 = [37.0, 0.0, 25.0];

/// WP-P17: hang point of FUN_001991a0 / FUN_00195fb8 case 0x11: contact (world, FUN_0022fc48 of JMP +0xd0) -
/// JMP_F0 * M, M rows = {-dir, FUN_0011e2d8(-dir, (0,0,1), (0,1,0))} (DAT_002a2dc0 / DAT_002a2db0).
pub fn ledge_hang_point(contact: V3, dir: V3) -> V3 {
    let a = scale(dir, -1.0);
    let (side, up) = FUN_0011e2d8(a, [0.0, 0.0, 1.0], [0.0, 1.0, 0.0]);
    let o: V3 = std::array::from_fn(|k| JMP_F0[0] * a[k] + JMP_F0[1] * side[k] + JMP_F0[2] * up[k]);
    sub3(contact, o)
}

/// 0x001995d8(jmp, M, 22, out): 11 segments (point pairs) in the frame M = rows {x, y, z, origin}:
/// 0: (+0x114, 0, +0x11c) -> (+0x114, 0, +0x118); 1: (+0x108, 0, +0x110) -> (+0x114, 0, +0x11c);
/// 2..10: R_z(k * 15 deg, k = 4..-4) applied (row vector) to (+0x108, 0, +0x110) -> (+0x108, 0, +0x10c).
pub fn FUN_001995d8(m: &[V3; 4]) -> [V3; 22] {
    let [j108, j10c, j110, j114, j118, j11c] = JMP_108;
    let mut loc = [[0.0f32; 3]; 22];
    loc[0] = [j114, 0.0, j11c];
    loc[1] = [j114, 0.0, j118];
    loc[2] = [j108, 0.0, j110];
    loc[3] = [j114, 0.0, j11c];
    let (a, b) = ([j108, 0.0, j110], [j108, 0.0, j10c]);
    for (i, k) in (-4..=4).rev().enumerate() {
        let r = FUN_0011b500([0.0, 0.0, 1.0], k as f32 * DEG15);
        let rv = |p: V3| [0, 1, 2].map(|c| p[0] * r[0][c] + p[1] * r[1][c] + p[2] * r[2][c]);
        loc[4 + 2 * i] = rv(a);
        loc[5 + 2 * i] = rv(b);
    }
    loc.map(|p| [0, 1, 2].map(|c| p[0] * m[0][c] + p[1] * m[1][c] + p[2] * m[2][c] + m[3][c]))
}

/// DAT_002baaf0 + id*0x14: one past the last class id of each class subtree (FUN_00132b10).
pub static DAT_002BAAF0: std::sync::LazyLock<Vec<u16>> = std::sync::LazyLock::new(|| (0..199).map(|i| crate::engine::disc::exe::exe().w(0x002b_aaf0 + i * 0x14) as u16).collect());

/// 0x00132b10(obj, id): class `cls` derives from `id` (id <= cls < end of id's subtree).
pub fn FUN_00132b10(cls: i32, id: i32) -> bool {
    if !(0..199).contains(&id) {
        return false;
    }
    id <= cls && cls < DAT_002BAAF0[id as usize] as i32
}

/// DAT_002bf8a4 / DAT_002bf8a8: ledge probe back / up offsets (0x199d48).
pub const DAT_002BF8A4: f32 = 60.0;
pub const DAT_002BF8A8: f32 = 70.0;
/// DAT_002bf660: ground attack range; DAT_002bf664: target list range / empty-slot distance.
pub const DAT_002BF660: f32 = 250.0;
pub const DAT_002BF664: f32 = 1000.0;
/// DAT_002bf700: edge force speed profile {vmax, v0, accel} (0x194170).
pub const DAT_002BF700: [f32; 3] = [2500.0, 0.0, 1.0];
/// REPLACED grab volume (jt+600 BSP) / contact gathering radius.
pub const LEDGE_GRAB_R: f32 = JT_RADIUS + LEDGE_FWD;

/// One SO contact of the JT (REPLACED source, see module doc).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct JtContact {
    pub collider: usize,
    /// +0x9c: 1 vertex, 2 face, 3 edge
    pub kind: i32,
    /// +0xa0 feature (vertex / surface / edge index)
    pub feature: usize,
    /// side +0x10 contact point (+0x18 = z)
    pub pos: V3,
    /// contact normal (towards the JT)
    pub n: V3,
    /// +0xb8 relative normal velocity
    pub vn: f32,
}

/// Ledge found by FUN_00199d48: param_2 side, param_3 direction, return contact.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LedgeHit {
    pub contact: JtContact,
    pub side: u32,
    pub dir: V3,
}

/// One 0x30 B target slot (0x194688 / list DAT_003e1b08 at DAT_003e1b10).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct JtTarget {
    /// +0 object (index), class id (FUN_00132b10 tests), +0x10c sub-kind (class 0x26)
    pub obj: usize,
    pub class: i32,
    pub f_10c: i32,
    /// +4 target member (TARGET sub-object), its +0x18 group
    pub member: usize,
    pub group: i32,
    /// +8 distance, +0xc priority (TARGET +0x10)
    pub dist: f32,
    pub prio: i32,
    /// +0x14 time, +0x18 sub (vtable +0x14 out)
    pub t: f32,
    pub sub: i32,
    /// +0x20 position
    pub pos: V3,
}

/// Input of 0x194688: an object of a category list with its TARGET member already evaluated through the
/// member's vtable (+0x14 sub, +0x1c position) by the caller.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TargetCand {
    pub obj: usize,
    pub class: i32,
    pub f_10c: i32,
    /// obj +0x10 != 0 -> radius check with obj +0x38
    pub radius: Option<(V3, f32)>,
    pub member: usize,
    /// TARGET +0x10 priority, +0x18 group, +0x1c flag mask
    pub prio: i32,
    pub group: i32,
    pub mask: u32,
    pub sub: i32,
    pub pos: V3,
    /// list category bit (0x194a40 param_3 bit: 8, 0x10, 0x80, 4, 0x400, 0x200, 2, 1, 0x20)
    pub category: u32,
    /// category 1 only: obj +0xe8 & 1
    pub f_e8: bool,
}

/// jt+0x4c0 attach object (FUN_001928f8 / FUN_00192bd8).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Attach {
    /// +4 time, +8 kind, +0xc substate (FUN_00192bd8), +0x14 target, +0x18 member
    pub t: f32,
    pub kind: i32,
    pub sub: i32,
    pub target: Option<usize>,
    pub member: usize,
    /// +0x4c4 (piVar2[0x131]) / +0x500 (piVar2[0x140])
    pub f_131: Option<usize>,
    pub f_140: i32,
}

fn sub3(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// 0x00199408(jmp, geom, edge, &dir): ledge edge test.
pub fn FUN_00199408(g: &CGeom, edge: usize) -> Option<V3> {
    let e = g.edges.get(edge)?;
    let s2 = g.surfs.get(e.owner as usize)?.plane;
    let s3 = g.surfs.get(e.other as usize)?.plane;
    let mut steep = s3;
    if s2[2] <= 0.7 || 0.55 <= s3[2] {
        if 0.55 <= s2[2] {
            return None;
        }
        steep = s2;
        if s3[2] <= 0.7 {
            return None;
        }
    }
    let (a, b) = (g.verts[e.a as usize], g.verts[e.b as usize]);
    let d = sub3(b, a);
    let p = [-d[1], d[0], 0.0];
    let l = dot(p, p).sqrt();
    let mut r = if l < 0.0001 { [1.0, 0.0, 0.0] } else { scale(p, 1.0 / l) };
    if dot(r, [steep[0], steep[1], steep[2]]) < 0.0 {
        r = scale(r, -1.0);
    }
    Some(r)
}

/// REPLACED grab volume: segment a-b passes within LEDGE_GRAB_R of c.
fn seg_in_grab(c: V3, a: V3, b: V3) -> bool {
    let d = sub3(b, a);
    let dd = dot(d, d);
    let t = if dd > 0.0 { (dot(sub3(c, a), d) / dd).clamp(0.0, 1.0) } else { 0.0 };
    let q = add(a, scale(d, t));
    let w = sub3(c, q);
    dot(w, w) < LEDGE_GRAB_R * LEDGE_GRAB_R
}

impl Jt {
    /// Contacts of the JT this frame (REPLACED source of jt+0xec+100).
    pub fn contacts(&self, cw: &CollisionWorld) -> Vec<JtContact> {
        let c = add(self.phys.pos, self.hang_so_ofs()); // FEEL R78: SO position
        let v = self.velocity();
        let mut out = Vec::new();
        for (k, col) in cw.colliders.iter().enumerate() {
            // CRAWL R61: SO parts: the FUN_0020faa0 bounds node gather
            if col.bbox.is_some() && !col.near(c, LEDGE_GRAB_R) {
                continue;
            }
            if let Some((sc, sr)) = col.sphere {
                let d = sub3(sc, c);
                if dot(d, d) > (sr + LEDGE_GRAB_R) * (sr + LEDGE_GRAB_R) {
                    continue;
                }
            }
            let mut sbi = Vec::new();
            FUN_001a93c8(LEDGE_GRAB_R, c, &col.geom, &col.bsp, 0x40, &mut sbi);
            for s in sbi {
                let n = [s.plane[0], s.plane[1], s.plane[2]];
                let kind = match s.kind {
                    6 => 2,
                    5 => 3,
                    4 => 1,
                    _ => continue,
                };
                // contact point on the feature (Sbi.pos is the sphere point c - r n)
                let g = &col.geom;
                let pos = match kind {
                    2 => sub3(c, scale(n, dot(n, c) - s.plane[3])),
                    3 => {
                        let e = g.edges[s.feature];
                        let (a, b) = (g.verts[e.a as usize], g.verts[e.b as usize]);
                        let d = sub3(b, a);
                        let dd = dot(d, d);
                        let t = if dd > 0.0 { (dot(sub3(c, a), d) / dd).clamp(0.0, 1.0) } else { 0.0 };
                        add(a, scale(d, t))
                    }
                    _ => g.verts[s.feature],
                };
                out.push(JtContact { collider: k, kind, feature: s.feature, pos, n, vn: dot(v, n) });
            }
        }
        out
    }

    /// WP-P5 contact list of FUN_00199d48: Sly's GEOM-199 hull pairs (p5_hull, with the JT side uVar5) when
    /// `hull` is set, else the P4 sphere contacts (JT always side 0; fallback when the model has no hull 199).
    pub fn contacts_sided(&self, cw: &CollisionWorld) -> Vec<(JtContact, u32)> {
        match &self.hull {
            Some(h) => {
                let ctx = super::p5_hull::JtCtx {
                    rot: self.phys.rot,
                    // FEEL R78: hull frame = SO position (pos + 45 for Sly, as the motor's world_so)
                    pos: add(self.phys.pos, self.hang_so_ofs()),
                    v: self.velocity(),
                    w: self.phys.w,
                    jts: self.jts,
                    jtbs: self.jtbs,
                    // FEEL R77: also on the ledge climb-up jump (JMP kind 3, FUN_00195a00(3) from the hang): the
                    // replacement sphere (radius 105 around P0) reached the climbed edge ~115 under Sly at the apex and
                    // FUN_00199d48 re-grabbed it; the hull itself is far above it (capture cap_feel_wall_step3_ledge:
                    // lands on top at +86.8 over the hang). SLY2_FEEL_CLIMBTOUCH=0 = old (full sphere).
                    // FEEL R79: every jump (the 0x1aa1d8 hull pairs cannot reach outside the hull + 8 on any jump;
                    // R56's all-jump try predates the 1:1 hang height, R77). Opt-in SLY2_FEEL_R79TOUCH=1: the no-stick climb
                    // test then picks a new ledge (edge z 381.7) whose climb re-grabs (jtbs 17); open.
                    // FEEL R90: DEFAULT (SLY2_FEEL_R79TOUCH=0 = old full sphere): the sphere grabbed an object's top
                    // edge ~150 under a Sly falling onto that top and the hang pulled him down through it (USER
                    // PLAYTEST 'falls through the floor when jumping on top of objects', feel_r90_probe_object_land:
                    // 6 of 106 drops -> 0); the edge-381.7 re-grab is a top Sly slides back off (test skips it).
                    rope_jump: (self.rope_jump || std::env::var("SLY2_FEEL_R79TOUCH").map_or(true, |v| v != "0") || (self.kind == 3 && std::env::var("SLY2_FEEL_CLIMBTOUCH").map_or(true, |v| v != "0"))) && self.jts == super::JTS_JUMP,
                    t_18d4: self.t_jtbs,
                    t: self.t,
                };
                super::p5_hull::jt_hull_contacts(h, cw, None, &ctx, &mut Vec::new())
            }
            None => self.contacts(cw).into_iter().map(|c| (c, 0)).collect(),
        }
    }

    /// 0x001999d0(jmp, geom obj, pos, kind, feature, &dir): ledge direction from the contact feature, then
    /// FUN_001997d8 (REPLACED, see module doc).
    pub fn FUN_001999d0(&self, cw: &CollisionWorld, ct: &JtContact) -> Option<V3> {
        let g = &cw.colliders.get(ct.collider)?.geom; // original holds a geom pointer; a stale port index (list shrank) = no contact
        let dir = match ct.kind {
            2 => {
                let s = g.surfs.get(ct.feature)?;
                if s.plane[2] < 0.7 {
                    return None;
                }
                let mut acc = [0.0f32; 3];
                let mut found = false;
                for &e in s.own_edges.iter().chain(s.other_edges.iter()) {
                    if let Some(d) = FUN_00199408(g, e) {
                        let ed = g.edges[e];
                        // FUN_00118668(jt+600 BSP, a, b, 2) > 0: WP-P9 the hull part's BSP when Sly has his hull
                        // (1:1), else the P4 stand-in (REPLACED grab volume, sphere mode)
                        let (va, vb) = (g.verts[ed.a as usize], g.verts[ed.b as usize]);
                        let grab = match &self.hull {
                            // FEEL R78: the hull frame is the SO position (as world_so in the contact gen), not the
                            // port pos (45 below); SLY2_FEEL_HANGSO=0 = old
                            Some(h) => h.clip_edge(&self.phys.rot, add(self.phys.pos, self.hang_so_ofs()), va, vb),
                            None => seg_in_grab(add(self.phys.pos, self.hang_so_ofs()), va, vb), // FEEL R78: SO frame
                        };
                        if grab {
                            found = true;
                            acc = add(acc, d);
                        }
                    }
                }
                if !found {
                    return None;
                }
                let l = dot(acc, acc).sqrt();
                if l < 0.0001 {
                    [1.0, 0.0, 0.0] // DAT_002a2da0
                } else {
                    scale(acc, 1.0 / l)
                }
            }
            3 => FUN_00199408(g, ct.feature)?,
            // kind 1 (FUN_001665f0 vertex edges) is only reached for side 1 (never in the port)
            _ => return None,
        };
        self.FUN_001997d8(cw, ct.pos, dir).then_some(dir)
    }

    /// 0x001997d8(jmp, pos, dir): WP-P10 1:1 ledge clearance. Frame M = {dir, side, up', pos} (FUN_0011e2d8),
    /// the 11 segments of FUN_001995d8 in M, clipped (FUN_0014c6a0 -> FUN_00118668 max 1) against every part
    /// of every object in the JT contact list: a hit on segment 0 (the drop in front) or 1 (the reach over the
    /// edge) or any hit starting inside (t_in == 0) fails; else >= 2 hits whose entry plane z > 0.7 (floor
    /// behind the edge) pass. REPLACED: contact list (*jmp)+0xec->+0x64 = the objects of `contacts_sided`
    /// (object flag +0x70 & 4 and part flag +0x10 bit 48 are not carried by CollisionWorld: never skipped).
    pub fn FUN_001997d8(&self, cw: &CollisionWorld, pos: V3, dir: V3) -> bool {
        let (side, up) = FUN_0011e2d8(dir, [0.0, 0.0, 1.0], [0.0, 1.0, 0.0]);
        let segs = FUN_001995d8(&[dir, side, up, pos]);
        let mut objs: Vec<usize> = self.contacts_sided(cw).iter().map(|(c, _)| cw.colliders[c.collider].object).collect();
        objs.sort_unstable();
        objs.dedup();
        let mut n = 0;
        for &o in &objs {
            for i in (0..22).step_by(2) {
                for col in cw.colliders.iter().filter(|c| c.object == o && !c.bsp.nodes.is_empty()) {
                    let mut out = Vec::new();
                    let k = FUN_00118668(&col.geom, &col.bsp, 0, col.bsp.nodes.len(), segs[i], segs[i + 1], 1, &mut out);
                    if k == 0 || out.is_empty() {
                        continue;
                    }
                    if i < 4 || out[0].t[0] == 0.0 {
                        return false;
                    }
                    if 0.7 < out[0].plane[0][2] {
                        n += 1;
                    }
                }
            }
        }
        2 <= n
    }

    /// 0x00144068 1:1: JT busy = +0x270 > 0 || +0x1084 != 0 (writers: 0x143de8 / list push, other rows).
    pub fn FUN_00144068(&self) -> bool {
        0 < self.n_270 || self.held_1084.is_some()
    }

    /// 0x00199d48(jmp, &side, &dir): best ledge among the JT contacts.
    pub fn FUN_00199d48(&self, cw: &CollisionWorld) -> Option<LedgeHit> {
        if self.FUN_00144068() {
            return None;
        }
        let k = if self.jtbs == 1 {
            self.kind
        } else {
            if !(3..=5).contains(&self.jtbs) {
                return None;
            }
            self.kind
        };
        if k < 0x11 && 0xe < k {
            return None;
        }
        let mut best: Option<LedgeHit> = None;
        let mut best_z = 0.0f32;
        for (ct, side) in self.contacts_sided(cw) {
            // +0x9c dispatch: 2 needs side 0, 1 needs side 1, 3 always
            let ok = match ct.kind {
                2 => side != 1,
                3 => true,
                1 => side != 0,
                _ => false,
            };
            if !ok {
                continue;
            }
            let Some(dir) = self.FUN_001999d0(cw, &ct) else { continue };
            // (+0x70 & 2) -> FUN_001a7fc0(world, 1, &contact): event only
            // DAT_002c3c60[side] * +0xb8 > 0: separating
            if 0.0 < [1.0f32, -1.0][side as usize] * ct.vn {
                continue;
            }
            let p = add(add(ct.pos, scale(dir, -DAT_002BF8A4)), [0.0, 0.0, DAT_002BF8A8]);
            let col = &cw.colliders[ct.collider];
            if FUN_00118fb0(&col.geom, &col.bsp, 0, col.bsp.nodes.len(), p).is_some() {
                continue;
            }
            if best.is_some() && ct.pos[2] < best_z {
                continue;
            }
            best_z = ct.pos[2];
            best = Some(LedgeHit { contact: ct, side, dir });
        }
        best
    }

    /// 0x00173610(d0, d1, &pos, steep_ok, filter): first surface on pos+up*d0 -> pos+up*d1 (mode 1);
    /// writes the hit point into `pos`.
    pub fn FUN_00173610(cw: &CollisionWorld, d0: f32, d1: f32, pos: &mut V3, steep_ok: bool) -> bool {
        let a = add(*pos, [0.0, 0.0, d0]);
        let b = add(*pos, [0.0, 0.0, d1]);
        // FUN_0014cc58 mode 1: a record whose t_in is 0 (start inside) is skipped
        let Some(l) = cw.clip_segment(a, b).into_iter().find(|l| l.t[0] != 0.0) else { return false };
        if !steep_ok && l.plane[0][2] < 0.7 {
            return false;
        }
        *pos = l.pos[0];
        true
    }

    /// 0x00194688: insert a candidate into the 4-slot list (`list` = 0xc0 B at sp+0x180, 4 x 0x30).
    pub fn FUN_00194688(&self, c: &TargetCand, list: &mut [JtTarget; 4], from: V3) {
        if let Some((p, r)) = c.radius {
            let d = sub3(p, from);
            if r * r < dot(d, d) {
                return;
            }
        }
        // +0x4d4 / +0x2cc / +0x37c / +0xf5c / +0x18c4 / +0x1f18 current targets and the +0x4d8 group:
        // the JT's own attachments; only the attach target is tracked by the port
        if Some(c.obj) == self.attach.target {
            return;
        }
        if c.mask & self.mask_1000 == 0 {
            return;
        }
        let d = sub3(from, c.pos);
        let dist = dot(d, d).sqrt();
        if DAT_002BF664 < dist {
            return;
        }
        let n = list.len();
        if c.group != 0 {
            if let Some(i) = (0..n).take_while(|&i| list[i].member != usize::MAX).find(|&i| list[i].group == c.group) {
                if list[i].dist <= dist {
                    return;
                }
                list.copy_within(i + 1..n, i);
                list[n - 1] = JtTarget::EMPTY;
            }
        }
        // from the end: stop at a slot of higher priority or same priority and closer
        let mut k = n;
        while k > 0 {
            let s = &list[k - 1];
            if c.prio < s.prio || (s.prio == c.prio && s.dist < dist) {
                break;
            }
            k -= 1;
        }
        if k != n {
            list.copy_within(k..n - 1, k + 1);
            list[k] = JtTarget {
                obj: c.obj,
                class: c.class,
                f_10c: c.f_10c,
                member: c.member,
                group: c.group,
                dist,
                prio: c.prio,
                t: self.t,
                sub: c.sub,
                pos: c.pos,
            };
        }
    }

    /// 0x00194a40(world, jt, mask): rebuild the target list from the category lists (`cands` in list
    /// order: bit 8, 0x10, 0x80, 4, 0x400, 0x200, 2, 1 (class 0x13 with +0xe8 & 1), 0x20). The result is the
    /// global list DAT_003e1b08 (count) / DAT_003e1b10 (FUN_00194f70 copy, REPLACED: kept on the JT).
    pub fn FUN_00194a40(&mut self, cands: &[TargetCand], mask: u32) {
        let mut list = [JtTarget::EMPTY; 4];
        let from = self.phys.pos;
        for bit in [8u32, 0x10, 0x80, 4, 0x400, 0x200, 2, 1, 0x20] {
            if mask & bit == 0 {
                continue;
            }
            for c in cands.iter().filter(|c| c.category == bit) {
                if bit == 1 && !(c.f_e8 && FUN_00132b10(c.class, 0x13)) {
                    continue;
                }
                self.FUN_00194688(c, &mut list, from);
            }
        }
        // count = 4, or the index past the last used slot
        let mut n = 4;
        if list[3].member == usize::MAX {
            n = 3;
            while n > 0 && list[n - 1].member == usize::MAX {
                n -= 1;
            }
        }
        self.calls.push((0x00195368, n as i32));
        self.targets = list[..n].to_vec();
    }

    /// 0x001938c0(attach): ground cane attack target pick over the target list (within 250).
    pub fn FUN_001938c0(&mut self, anim: &mut dyn JtAnim) -> bool {
        let n = self.targets.len();
        if n == 0 {
            return false;
        }
        if DAT_002BF660 < self.targets[0].dist {
            return false;
        }
        let mut i = 0;
        loop {
            let tg = self.targets[i];
            let c = tg.class;
            let kind = if FUN_00132b10(c, 0x13) || FUN_00132b10(c, 0xb) {
                if self.is_player { 6 } else { 5 }
            } else if FUN_00132b10(c, 0x1b) {
                8
            } else if FUN_00132b10(c, 0xc) || FUN_00132b10(c, 0x22) {
                0
            } else if FUN_00132b10(c, 0x26) {
                tg.f_10c
            } else if FUN_00132b10(c, 0x23) {
                7
            } else {
                -1
            };
            if kind != -1 {
                self.FUN_001928f8(kind, Some(tg.obj), tg.member, anim);
                return true;
            }
            i += 1;
            if i >= n || DAT_002BF660 < self.targets[i].dist {
                return false;
            }
        }
    }

    /// 0x001928f8(attach, kind, target, member): start an attach / attack action.
    /// WP-P5: a target with attach data in `p5_targets` runs the real p5_attach::attach_to (+ jts 7); else the
    /// field writes below are kept and the substate bodies are recorded.
    pub fn FUN_001928f8(&mut self, kind: i32, target: Option<usize>, member: usize, anim: &mut dyn JtAnim) {
        if let Some(tg) = self.target_attach(target) {
            self.calls.push((CALL_ATTACH, kind));
            self.attach_real(kind, target, member, tg, anim);
            return;
        }
        // BOSS R22: kind 1 (TNT barrel, jtbs 6 arrival) needs no target data: the real FUN_00192bd8
        if kind == 1 && target.is_some() {
            self.calls.push((CALL_ATTACH, kind));
            let mut x = std::mem::take(&mut self.att);
            let mut ev = Vec::new();
            super::p5_attach::attach_to(self, &mut x, None, kind, target, member, &mut ev);
            self.att = x;
            self.att_tg = None;
            self.apply_attach_ev(ev, anim);
            return;
        }
        self.calls.push((0x00192bd8, -1));
        self.attach.sub = -1;
        self.attach.kind = kind;
        self.attach.member = member;
        self.attach.t = self.t;
        self.attach.target = target;
        // vtable +0x2c4(jt, 2): enter the attach state (jts set by the state body, unported)
        self.calls.push((0x2c4, 2));
        let sub = match kind {
            0 | 0xd => 5,
            1 => {
                self.calls.push((0x00192a28, 1));
                // WP-PJ4: PJA1 adapter dispatches the ee_sub_192a28 queue port (empty DAT_003e1444 chain
                // in the shadow image => safe no-op on Jt; still exercises the registered QMem port).
                super::pj_adapt::call_q_jtptr(self, 0x0019_2a28);
                0
            }
            5 | 9 => {
                self.attach.f_131 = target;
                self.calls.push((0x001ccac0, 0));
                0
            }
            6 => {
                // jt+0x1854 = target; FUN_0018e540(jt, 0); FUN_0018ed70(jt)
                // WP-P19: FUN_0018e540(0) really runs (was recorded only); FUN_0018ed70 stays recorded
                self.FUN_0018e540(0, anim);
                self.calls.push((0x0018ed70, 0));
                0
            }
            8 => {
                self.calls.push((0x00192a28, 8));
                super::pj_adapt::call_q_jtptr(self, 0x0019_2a28);
                7
            }
            0xb | 0xc => {
                self.attach.f_140 = 1;
                0
            }
            _ => 0,
        };
        self.calls.push((0x00192bd8, sub));
        self.attach.sub = sub;
        self.calls.push((CALL_ATTACH, kind));
        // WP-K33: kind 7 = FUN_001938c0 on an is-a 0x23 target (the DAT_003e1414 use points, classes 35-37; class 37
        // C2cb9f8 = the guard loot holder). Its jts-7 frame 0x1911a8 kind 7 is: drop (SetJtJts(jt, 0) -> attach -1)
        // + target vt+0x19c (slot 50)(target, jt). REPLACED: no attach state body runs for targets without attach
        // data, so that frame runs at the next JT input step (`k7_pending`, player/mod.rs `input`).
        self.k7_pending = kind == 7;
    }

    /// 0x00194170(attach) jts 7 edge forces: attach kind 3 with target +0x1bc set. `dir` = FUN_001e67b8
    /// (target edge direction), `t1c0` / `t1c4` = target +0x1c0 / +0x1c4. Returns the acceleration added.
    pub fn FUN_00194170(&self, dir: V3, t1c0: f32, t1c4: f32, dt: f32) -> V3 {
        if self.attach.kind != 3 {
            return [0.0; 3];
        }
        let s = dot(dir, self.velocity());
        let g = dot(dir, self.phys.gravity);
        let mut hi = 0.0f32;
        let mut lo = 0.0f32;
        crate::engine::physics::dynamics::FUN_0011d018(s, t1c0, dt, DAT_002BF700, Some(&mut hi));
        crate::engine::physics::dynamics::FUN_0011d018(s, -t1c0, dt, DAT_002BF700, Some(&mut lo));
        let f5 = t1c4 * g;
        if lo <= f5 {
            lo = f5;
            if hi < f5 {
                lo = hi;
            }
        }
        scale(dir, lo - g)
    }
}

impl JtTarget {
    /// empty slot: zeroed, +8 = DAT_002bf664, +0xc = -1 (0x194a40 init loop)
    pub const EMPTY: JtTarget = JtTarget {
        obj: usize::MAX,
        class: -1,
        f_10c: 0,
        member: usize::MAX,
        group: 0,
        dist: DAT_002BF664,
        prio: -1,
        t: 0.0,
        sub: 0,
        pos: [0.0; 3],
    };
}
