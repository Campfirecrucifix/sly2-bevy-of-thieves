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
use super::driver::{AloPhys, FUN_001f9b20, FUN_001f9e70, FUN_001fac30, FUN_0022de88, M3};
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
    for col in &cw.colliders {
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
    // 0x1f3980: previous velocities, accel reset, slot 69 (0x1f9b20 gravity + the PO's own forces), 0x22de88
    b.prev_v = b.v;
    b.prev_w = b.w;
    if matches!(b.kind, 1 | 2) {
        b.a = [0.0; 3];
        b.alpha = [0.0; 3];
        FUN_001f9b20(b);
        FUN_001fac30(b, accel);
    }
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
    if let Some((ofs, r)) = hull {
        let mut lv = Vec::new();
        level_xps(cw, add(b.pos, ofs), r, &mut lv);
        for mut x in lv {
            FUN_00204250(m, &mut x);
            FUN_001a7fc0(m, &mut x, b.v, dt);
            xps.push(x);
        }
    }
    let solve = if xps.is_empty() { PoSolve::default() } else { FUN_001f9410_po(&xps, m.mass, b.gravity[2], 1, dt) };
    b.imp_lin = add(b.imp_lin, solve.dv);
    FUN_001f9e70(b, dt);
    PoStep { motor: xps.first().filter(|x| x.kind == 0xe).copied(), xps, solve }
}
