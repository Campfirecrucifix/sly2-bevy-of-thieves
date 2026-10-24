//! WP-P5 states: JT main states other than 0/1/2 (forces dispatched by 0x188960, SetJtJts 0x18c250
//! enter/exit actions, per-jts updates of 0x18bce0) and the JMP body states 5..0x10 / 0x12..0x16 of 0x197500,
//! plus the stuck test / safe-point warp (the only "death / respawn" mechanism inside JT code).
//!
//! Integration (lead): keep one `P5` per JT next to `Jt`, fill a `P5World` per frame (engine-side data the
//! port does not own: swing/ride target, orbit target, anim times, contact normals, ...), then
//! - SetJtJts: route every SetJtJts through `FUN_0018c250_p5` (exit / enter actions + `Jt::FUN_0018c250`
//!   bookkeeping + jtbs switch). JMP body changes stay `Jt::FUN_0018c250(2, jtbs)` (= 0x195fb8).
//! - input step (0x18b7e8): jts 3 -> `FUN_00190520`; jts 2 -> in the 0x197500 loop: `FUN_00197500_pre` first,
//!   then for jtbs not handled by mod.rs call `FUN_00197500_p5` (returns false when it does not own the jtbs);
//!   jtbs 3/4 airborne without ledge -> `FUN_00197500_tail5`.
//! - update (0x18bce0 switch): `FUN_0018bce0_p5` (jts 3, 5, 10, 0xc, 0x15, 0x16, 0x18).
//! - forces (0x188960): `FUN_00188960_p5` returns `Some(Acc)` for jts 3, 4, 6, 8, 0xe, 0x18. Base terms are the
//!   lead's: gravity (0x1f9b20) is added by the physics driver for every jts; 0x204108 steering must be added
//!   for jts 4, 6, 0xe, 0x18 (not 3 / 8 / 0x15 / 0x16). Apply `Acc.lin`/`Acc.ang` as impulses *dt and
//!   `Acc.set_w` as an angular velocity set (vt+0x124).
//!
//! 1:1: 0x1903b8 (swing forces), 0x18fc60 (swing pendulum integrator, jt+0x1900 == 0 path), 0x190520 (swing
//! input / jtbs machine / blend weights / anim scrub), 0x190e18 (swing orientation dof target + 0x1960 decay),
//! 0x1a1148 (jts 4), 0x1f5e10 (pos/vel spring), 0x137248 (jts 6 orbit), 0x19df70 (jts 8, except 0x117308),
//! 0x188960 cases 0xe and 0x18, 0x1a0450 (8-way blend), 0x18df60 (jts 0x16 ride), 0x1f59a0 angular servo part,
//! 0x196928 (jump-time solver), 0x19a098 gating + residual test, 0x19f590 decision logic, 0x195fb8 entry 0x15
//! (ballistic launch to point), 0x197500 cases 5, 6, 8, 9..0x10, 0x12, 0x13/0x14, 0x15, 0x16, 0x19c688 cases
//! 0xe / 0xf / 0x10, SetJtJts field side effects listed in spec_states §2 for jts 3,4,5,6,8,10,0xc,0xe,0x15,
//! 0x16,0x18 and jtbs 3..0xd, 0x19eab0 state bookkeeping. Pure math: 0x11e290, 0x11f088, 0x11ba08, 0x11bc00,
//! 0x11c950 (0x11d018 / 0x11d1c0 / 0x11b500 / 0x11b570 / 0x11bcf8 / 0x11bff0 reused from physics/camera).
//! REPLACED: 0x11ad30 (Dantzig LCP of 0x19a098) by projected Gauss-Seidel on the same LCP; 0x19faf8 / 0x19f7d0
//! safe-point scoring (world ray queries) by nearest safe point; 0x19f3f8 predicted point by the static point;
//! 0x117308 rotation spline (jts 8 states 0/4) by a constant-rate slerp over the remaining anim time;
//! 0x11e390 (perpendicular fallback of 0x11ba08) by cross with x / y; 0x185e40 probe frame (0x185310 x3 +
//! 0x11b830) by the `Surface` normal with the facing kept; 0x2046b0 / 0x204660 facing = `Jt::yaw`.
//! UNPORTED (recorded in `Jt::calls` as (address, arg)): anim (0x12f9b8 / 0x1f03a8 / 0x1305b8 / 0x1384c0 via
//! JtAnim::play_aseg or calls), messages to targets (vt+0x1c msg 5 / 6 -> 0x1c), 0x1333c0 / 0x133448 /
//! 0x140c88 / 0x204f78 / 0x1f5720 / 0x18e540 / 0x1ccac0, constraints 0x230018 / 0x230100 + driven dof
//! 0x12d108 (value kept in `P5::dof_1a00`), rope-on-body 0x18f958 / 0x1f5078, jump off swing 0x19cc50 (148 B,
//! JMP launcher wrapper not specced), path frame 0x1c3500 (input `P5World::path8`), surface boundary
//! 0x1856b8 / 0x1856c0 / 0x185a80, foot IK 0x189c40 / 0x189ec8 (cosmetic, spec §4.5), sounds, camera cut
//! 0x14d8c0, keyed-motion object 0x12de58 / 0x12e0f0, carry machine 0x2206c8, 0x19b110 air attack.
//! DAMAGE / ZAP / DEATH / LIVES / CHECKPOINT: spec_states §0 found none of them in JT code (no SetJtJts call
//! site or generic state is a damage path; the Sly-1 names of jts 3/5/8 are wrong: 3 = swing, 5 = combat
//! stance, 8 = class-0xb attach). They live in a parent-class / player-manager path not identified yet, so
//! nothing is invented here. The JT-side respawn is the stuck test 0x19a098 -> safe-point warp 0x19f590
//! (teleport + camera cut when the jump time to the point > 1.25 s, else a ballistic "jump to point" jtbs
//! 0x15 that ends in a fall jtbs 4 at its deadline).
#![allow(non_snake_case, dead_code)]

use super::{Jt, JtAnim, FUN_0011e1a0, FUN_0019a750, JTS_JUMP};
use crate::engine::camera::math::{
    FUN_0011b480, FUN_0011bcf8 as bcf8_m4, FUN_0011bff0 as bff0_m4, FUN_0011cf60, FUN_0011cfe0, FUN_0011d1c0,
    FUN_0011e9f8, M4, PI,
};
use crate::engine::physics::driver::{FUN_0011b3d0, FUN_0011b500, FUN_0011b570, IDENT, M3};
use crate::engine::physics::dynamics::{FUN_0011d018, FUN_0011d508};
use crate::engine::physics::geom::{dot, V3};

// ------------------------------------------------------------------------------------------------ constants
/// DAT_002a2dc0 / DAT_002a2da0 / DAT_002a2db0 / DAT_002a2dd0.
pub const UP: V3 = [0.0, 0.0, 1.0];
pub const AX_X: V3 = [1.0, 0.0, 0.0];
pub const AX_Y: V3 = [0.0, 1.0, 0.0];
pub const ZERO: V3 = [0.0; 3];
/// 0x38d1b717
const EPS: f32 = 1e-4;

/// DAT_002bf288 / DAT_002bf28c (0x188960 cases 0xe / 0x18 accel limits), DAT_002bf290 / DAT_002bf294.
pub const DAT_002BF288: f32 = 4000.0;
pub const DAT_002BF28C: f32 = 4500.0;
pub const DAT_002BF290: f32 = 100.0;
pub const DAT_002BF294: f32 = 1.0;
/// 0x2bf548 = {15, 0, 0.1} (0x1903b8 servo params).
pub const DAT_002BF548: [f32; 3] = [15.0, 0.0, 0.1];
/// 0x18fc60 constants DAT_002bf4f0.. (kpar poly, clamps, kperp, pump).
pub const DAT_002BF4F0: [f32; 3] = [1.5, -0.35, 0.0];
pub const DAT_002BF500: f32 = -5.0;
pub const DAT_002BF504: f32 = -0.25;
pub const DAT_002BF508: f32 = -2.5;
pub const DAT_002BF50C: f32 = -1.0;
pub const DAT_002BF510: f32 = 0.25;
pub const DAT_002BF514: f32 = 5.0;
pub const DAT_002BF520: [f32; 3] = [0.5, -0.01, 0.0];
pub const DAT_002BF530: [f32; 4] = [1000.0, 0.0, 0.25, 5000.0];
pub const DAT_002BF540: f32 = 75.0;
pub const DAT_002BF544: f32 = 230.0;
/// 0x190520 constants.
pub const DAT_002BF554: f32 = 1.4;
pub const DAT_002BF558: f32 = 1.4;
pub const DAT_002BF55C: f32 = 0.25;
pub const DAT_002BF560: f32 = 0.1;
pub const DAT_002BF564: f32 = 0.4;
pub const DAT_002BF568: f32 = 80.0;
pub const DAT_002BF56C: f32 = 120.0;
pub const DAT_002BF570: [f32; 3] = [1.0, 0.0, 0.1];
pub const DAT_002BF580: [f32; 3] = [4.0, 0.0, 0.1];
pub const DAT_002BF58C: f32 = 1.7;
pub const DAT_002BF590: f32 = 3.0;
/// 0x190e18: DAT_002bf594 (1.5 rad about x), DAT_002bf598 {3000, 1000, 0.15}.
pub const DAT_002BF594: f32 = 1.5;
pub const DAT_002BF598: [f32; 3] = [3000.0, 1000.0, 0.15];
/// 0x1a1148 P = 0x2bfa90 {0, 10, 0.01}.
pub const DAT_002BFA90: [f32; 3] = [0.0, 10.0, 0.01];
/// 0x137248: DAT_002bc998 radius 250, DAT_002bc99c / DAT_002bc9a0 1500, 0x2bc9a8 {500, 0, 0.5}.
pub const DAT_002BC998: f32 = 250.0;
pub const DAT_002BC99C: f32 = 1500.0;
pub const DAT_002BC9A0: f32 = 1500.0;
pub const DAT_002BC9A8: [f32; 3] = [500.0, 0.0, 0.5];
/// 0x18df60 / 0x185e40 servo params 0x2bf3f8 / 0x2be920 {25, 0, 0.25}, position 0x2bf408 {1000, 0, 0.25}.
pub const DAT_002BF3F8: [f32; 3] = [25.0, 0.0, 0.25];
pub const DAT_002BE920: [f32; 3] = [25.0, 0.0, 0.25];
pub const DAT_002BF408: [f32; 3] = [1000.0, 0.0, 0.25];
/// 0x18bce0: DAT_002bf3d4 (jts 1 * 1.3), DAT_002bf3d8 (jts 5 * 0.85), DAT_002bf3dc (foot IK 20.0).
pub const DAT_002BF3D4: f32 = 1.3;
pub const DAT_002BF3D8: f32 = 0.85;
pub const DAT_002BF3DC: f32 = 20.0;
/// SetJtJts: DAT_002bf3e8 (jts 0x18 launch speed), DAT_002bf358 / DAT_002bf35c (jtbs 0xc / 0xd jt+0x200).
pub const DAT_002BF3E8: f32 = 1000.0;
pub const DAT_002BF358: f32 = 3.0;
pub const DAT_002BF35C: f32 = 0.1;
/// 0x197500: DAT_002bf82c / DAT_002bf830 (jtbs 0x16 timers), 0x15 lookahead 0.25, 0x19f590 DAT_002bf940.
pub const DAT_002BF82C: f32 = 0.5;
pub const DAT_002BF830: f32 = 3.0;
pub const DAT_002BF940: f32 = 1.25;
/// 0x195fb8 entry 0x15: DAT_002bf7a0 minimum flight time.
pub const DAT_002BF7A0: f32 = 0.7;
/// 0x19a098 thresholds (1.0 s in state, 1.0 s rate limit, residual z > -0.1).
pub const STUCK_T: f32 = 1.0;
pub const STUCK_RZ: f32 = -0.1;
/// joy bit tested by 0x190520 for the jump off the swing (FUN_001c4660(0.15, joy, 0x400)).
pub const JOY_SWING_JUMP: u32 = 0x400;
/// classes accepted by 0x19f590 / 0x197500 case 0x15 for the follow-up attack (0x13,0xb,0xd,0x27,0xc,0x22,0x26).
pub const ATTACK_CLASSES: [i32; 7] = [0x13, 0xb, 0xd, 0x27, 0xc, 0x22, 0x26];

// ---------------------------------------------------------------------------------------- vector helpers
fn vadd(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn vsub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn vscale(a: V3, s: f32) -> V3 {
    [a[0] * s, a[1] * s, a[2] * s]
}
fn vlen(a: V3) -> f32 {
    dot(a, a).sqrt()
}
/// VU cross (vopmula a,b / vopmsub b,a) = a x b.
fn vcross(a: V3, b: V3) -> V3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
/// |v| < 1e-4 -> fallback else v / |v|.
fn norm_or(v: V3, fb: V3) -> V3 {
    let l = vlen(v);
    if l < EPS { fb } else { vscale(v, 1.0 / l) }
}
/// row vector * matrix (vmulax/vmadday/vmaddz): sum v_i * m.row_i.
fn vmul(v: V3, m: &M3) -> V3 {
    vadd(vadd(vscale(m[0], v[0]), vscale(m[1], v[1])), vscale(m[2], v[2]))
}
/// world -> frame-local (FUN_0022fcc8(0, frame, v) / FUN_001a6fc0(0, obj, v)): (v.r0, v.r1, v.r2).
fn to_local(v: V3, m: &M3) -> V3 {
    [dot(v, m[0]), dot(v, m[1]), dot(v, m[2])]
}
fn mmul(a: &M3, b: &M3) -> M3 {
    [vmul(a[0], b), vmul(a[1], b), vmul(a[2], b)]
}
fn m4(m: &M3) -> M4 {
    [[m[0][0], m[0][1], m[0][2], 0.0], [m[1][0], m[1][1], m[1][2], 0.0], [m[2][0], m[2][1], m[2][2], 0.0], [0.0, 0.0, 0.0, 1.0]]
}
fn m3(m: &M4) -> M3 {
    [[m[0][0], m[0][1], m[0][2]], [m[1][0], m[1][1], m[1][2]], [m[2][0], m[2][1], m[2][2]]]
}
fn rec(jt: &mut Jt, addr: u32, arg: i32) {
    jt.calls.push((addr, arg));
}
/// vt+0x10c linear velocity incl. the pending SetVelocity impulse (same as mod.rs `velocity`).
fn vel(jt: &Jt) -> V3 {
    vadd(jt.phys.v, jt.phys.imp_lin)
}
/// vt+0x11c SetVelocity (same mechanism as mod.rs `set_velocity`).
/// FEEL R91: delegates to mod.rs `set_velocity` (direct +v write since R61); the impulse path moved the body only
/// J dt/2 on the launch frame (0x1f9e70), so every P5 launch (jtbs 0x15 hop, ...) flew v dt/2 low (27.8 at 30 fps)
/// (live table-2 hop to 4449: arrived 43 under the top and fell). SLY2_FEEL_R91SETV=0 = old (impulse).
fn set_vel(jt: &mut Jt, v: V3) {
    if std::env::var("SLY2_FEEL_R91SETV").map_or(true, |x| x != "0") {
        jt.set_velocity(v);
        return;
    }
    jt.phys.imp_lin = vsub(v, jt.phys.v);
}
/// vt+0x124 set angular velocity (through the pending angular impulse, consumed by 0x1f9e70).
fn set_w(jt: &mut Jt, w: V3) {
    jt.phys.imp_ang = vsub(w, jt.phys.w);
}
fn grounded(jt: &Jt) -> bool {
    jt.ground_nz.is_some_and(|z| 0.7 < z)
}

// ------------------------------------------------------------------------------------------- pure math
/// 0x0011e290(v, a, &par, &perp) 1:1: k = v.a; par = a k; perp = v - par. Returns (k, par, perp).
pub fn FUN_0011e290(v: V3, a: V3) -> (f32, V3, V3) {
    let k = dot(v, a);
    let par = vscale(a, k);
    (k, par, vsub(v, par))
}

/// 0x0011f088(from, to, dt, p, &vout) 1:1: straight-line approach by 0x11d018 on the distance.
/// Returns (new point, velocity).
pub fn FUN_0011f088(from: V3, to: V3, dt: f32, p: [f32; 3]) -> (V3, V3) {
    let d = vsub(to, from);
    let l = vlen(d);
    if l > EPS {
        let mut v = 0.0;
        let s = FUN_0011d018(0.0, l, dt, p, Some(&mut v));
        (vadd(from, vscale(d, s / l)), vscale(d, v / l))
    } else {
        (to, ZERO)
    }
}

/// 0x0011ba08(out, a, b) 1:1: shortest-arc rotation with a * M parallel to b (row-vector convention).
/// REPLACED: the parallel fallback 0x11e390 (a perpendicular of a) = a x (1,0,0), or a x (0,1,0) when degenerate.
pub fn FUN_0011ba08(a: V3, b: V3) -> M3 {
    let (la, lb) = (vlen(a), vlen(b));
    if la < EPS || lb < EPS {
        return IDENT;
    }
    let c = vcross(a, b);
    let lc = vlen(c);
    let axis = if lc <= 1e-8 {
        let p = vcross(a, AX_X);
        let p = if vlen(p) < EPS { vcross(a, AX_Y) } else { p };
        vscale(p, 1.0 / vlen(p))
    } else {
        vscale(c, 1.0 / lc)
    };
    let cs = FUN_0011cfe0(dot(a, b) / (la * lb), 1.0);
    let s = (1.0 - cs * cs).max(0.0).sqrt();
    m3(&FUN_0011b480(s, cs, axis))
}

/// 0x0011bc00(out, A, B) 1:1: out = A^T B (out.row_k = sum_i A[i][k] B.row_i).
pub fn FUN_0011bc00(a: &M3, b: &M3) -> M3 {
    let mut o = [[0.0; 3]; 3];
    for k in 0..3 {
        for i in 0..3 {
            o[k] = vadd(o[k], vscale(b[i], a[i][k]));
        }
    }
    o
}

/// 0x0011bcf8 on a row matrix -> (angle, unit axis).
pub fn FUN_0011bcf8(m: &M3) -> (f32, V3) {
    bcf8_m4(&m4(m))
}

/// 0x0011c950(dt, Rfrom, Rto, p, &Rout, &wout) 1:1: rotation approach. Returns (Rout, wout).
pub fn FUN_0011c950(dt: f32, from: &M3, to: &M3, p: [f32; 3]) -> (M3, V3) {
    let d = FUN_0011bc00(from, to);
    let (ang, axis) = FUN_0011bcf8(&d);
    let mut v = 0.0;
    let s = FUN_0011d018(0.0, ang, dt, p, Some(&mut v));
    let r = if s == ang { *to } else { mmul(from, &FUN_0011b500(axis, s)) };
    (r, vscale(axis, v))
}

/// 0x001f59a0 angular part 1:1 (servo of the angular xf jt+0x5c toward target rows `target`, params
/// {max rate s, p1, p2}): over-speed damping, braking-distance rate profile, accel limited to (s-p1)/p2.
/// `alpha` = current angular accel accumulator (xf+0xf0). Returns the accel to add to it.
/// (The parent-frame conversion 0x22fd38 is the identity for the unparented JT.)
pub fn FUN_001f59a0(rot: &M3, w: V3, alpha: V3, target: &M3, p: [f32; 3], dt: f32) -> V3 {
    let s = p[0];
    let a = (s - p[1]) / p[2];
    let wl = vlen(w);
    let brake = (s * s) / (a + a);
    let mut add = ZERO;
    if s < wl {
        add = vscale(w, ((s - wl) * 2.5) / wl);
    }
    let acc = vadd(alpha, add);
    let mut t = *target;
    FUN_0011b3d0(&mut t);
    let d = FUN_0011bc00(rot, &t);
    let (mut ang, axis) = FUN_0011bcf8(&d);
    let mut sp = FUN_0011cfe0((ang / brake) * s, s);
    ang -= (dot(w, axis) + sp) * dt * 0.5;
    if ang < brake {
        sp = FUN_0011cfe0((ang / brake) * s, s);
    }
    let e = vsub(vscale(axis, sp), vadd(w, vscale(acc, dt)));
    let el = vlen(e);
    let k = if el < a * dt { 1.0 / dt } else { a / el };
    vadd(add, vscale(e, k))
}

/// 0x001f5e10(obj, tpos, pparams|0, tvel, vparams|0) 1:1 (no parent frame): spring of position and/or
/// velocity. Returns the accel added to xf+0xf0.
pub fn FUN_001f5e10(pos: V3, v: V3, tpos: V3, pp: Option<[f32; 3]>, tvel: V3, vp: Option<[f32; 3]>, dt: f32) -> V3 {
    let mut a = ZERO;
    if let Some(p) = pp {
        let d = vsub(tpos, pos);
        let r = vlen(d);
        if r > EPS {
            a = vadd(a, vscale(d, p[2] * r + p[1] + p[0] / r));
        }
    }
    if let Some(p) = vp {
        let dv = vsub(tvel, v);
        let r = vlen(dv);
        if r > EPS {
            let k = (p[2] * r + p[1] + p[0] / r).min(1.0 / dt);
            a = vadd(a, vscale(dv, k));
        }
    }
    a
}

/// 0x00196928(p1, p2, p3, p4, d, g) 1:1 (disasm; Ghidra lost the qword args): flight time to reach the
/// offset d: largest root of g.z/2 t^2 + p1 t = d.z (rising at most p1) and of -g.z/2 t^2 + p2 t + d.z ...,
/// and sqrt of the horizontal-slope limits (p3 |d.xy| - d.z) / -(g.z/2), (p4 |d.xy| - d.z) / (g.z/2).
pub fn FUN_00196928(p1: f32, p2: f32, p3: f32, p4: f32, d: V3, g: V3) -> f32 {
    let hg = g[2] * 0.5; // f22
    let dz = d[2]; // f21
    let mut f20 = 0.0f32;
    let mut r = [0.0f32; 2];
    let n = FUN_0011d508(-hg, -p1, dz, &mut r);
    if n > 0 {
        f20 = r[n - 1].max(f20); // max.s f20, f0, f20
    }
    let n = FUN_0011d508(hg, -p2, dz, &mut r);
    if n > 0 {
        f20 = r[n - 1].max(f20);
    }
    let h = (d[0] * d[0] + d[1] * d[1]).sqrt();
    let f0 = (p3 * h - dz) / -hg;
    if 0.0 < f0 {
        f20 = f0.sqrt().max(f20);
    }
    let f0 = (p4 * h - dz) / hg;
    if 0.0 < f0 {
        f20 = f0.sqrt().max(f20);
    }
    f20
}

// ------------------------------------------------------------------------------------------- data types
/// Swing / ride target jt+0x18c4 (class 0x13) as seen by the port.
#[derive(Clone, Copy, Debug)]
pub struct SwingTarget {
    /// FUN_001a62e8(T) pivot position
    pub pos: V3,
    /// FUN_001a6318(T) / vt+0xfc rows 0..2
    pub rot: M3,
    /// FUN_0022d348: velocity of T's body at the pivot (0 for a static pivot)
    pub v_point: V3,
    /// T+0x124 attach mode (3 = ride -> jts 0x16; 0 / 1 / 2 facing choice)
    pub mode_124: i32,
    /// T+0x134 rope length reduction (target length = 230 - this)
    pub f_134: f32,
}
impl Default for SwingTarget {
    fn default() -> Self {
        Self { pos: ZERO, rot: IDENT, v_point: ZERO, mode_124: 1, f_134: 0.0 }
    }
}

/// A moving object (position, velocity), e.g. the jts 6 orbit target jt+0x10dc.
#[derive(Clone, Copy, Debug, Default)]
pub struct Kin {
    pub pos: V3,
    pub v: V3,
}

/// Current anim player view (jt+0x1030 ...).
#[derive(Clone, Copy, Debug, Default)]
pub struct AnimView {
    /// (jt+0x1030)+0x1c current time
    pub t: f32,
    /// *((jt+0x1030)+0x14)+0x74 duration
    pub dur: f32,
    /// FUN_00204e38(jt, anim, t): root-motion velocity (jts 0xe)
    pub root_vel: V3,
    /// FUN_0012f990(*(jt+0x1a84), 0x16d) (jts 0xe gravity-cancel event time)
    pub ev_16d: f32,
    /// *(jt+0x1994)+0x74 swing blend duration (0x190520 scrub)
    pub dur_1994: f32,
    /// FUN_00131968(jt+0x1030) != 0 (anim finished) / jt+0x1030 == 0
    pub done: bool,
}

/// Safe point (jt+0x9e0 list entry / 0x19f590 dest, 32 B {obj, member, ?, ?, pos}).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SafePoint {
    pub pos: V3,
    /// class id of the object the point belongs to (is-a tests), None = no object
    pub obj_class: Option<i32>,
    /// member != 0
    pub member: bool,
    /// FEEL R92: the dest object (+0 obj) when it is a pick target (`Jt::p5_targets` id), None = no object
    pub obj: Option<usize>,
}

/// 8-entry blend (jt+0x13d0 / 0x1ac0 / 0x1e10; 0x20 B per entry: +0 weight, +4 enable, +0x10 velocity).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Blend8 {
    pub w: [f32; 8],
    pub en: [bool; 8],
    pub vel: [V3; 8],
}

/// jts-8 sub-object jt+0xf50.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Jts8 {
    /// s+4 sub-state (-1 detached)
    pub state: i32,
    /// s+8 time entered
    pub t_8: f32,
    /// s+0xc object present (jt+0xf5c)
    pub obj: bool,
    /// s+0x10 path param u (jt+0xf60)
    pub u: f32,
    /// s+0x18 direction flag
    pub dir_18: bool,
    /// s+0x1c
    pub f_1c: i32,
}
impl Default for Jts8 {
    fn default() -> Self {
        Self { state: -1, t_8: 0.0, obj: false, u: 0.0, dir_18: false, f_1c: 0 }
    }
}

/// Engine-side inputs the port does not own (filled by the lead per frame).
#[derive(Clone, Debug, Default)]
pub struct P5World {
    /// jt+0x18c4 swing / ride target (also the JMP target of jtbs 0xe)
    pub swing: Option<SwingTarget>,
    /// jt+0x10dc (jts 6 orbit target)
    pub orbit: Option<Kin>,
    /// FUN_001c3500(obj, u, dir): jts 8 path frame
    pub path8: Option<M3>,
    /// path tangent z at u (0x19eab0 old -1: s+0x18 = t.z < -0.5)
    pub path8_tangent_z: f32,
    pub anim: AnimView,
    /// raw stick joy+0x10 / joy+0x14
    pub stick: [f32; 2],
    /// camera rows DAT_002ddfa0 / b0 / c0
    pub cam_rows: Option<M3>,
    /// contact normals of jt+0xec's list (c+0x80, negated when c+8 != jt) for 0x19a098
    pub contact_normals: Vec<V3>,
    /// jmp+0xc target: Some(active bit 60)
    pub jmp_target_active: Option<bool>,
    /// *(jmp+0xc)+0x17c (jtbs 6: 3 = abort)
    pub jmp_target_17c: i32,
    /// jtbs 0x12: FUN_0012f990(*(P2+0x1974), 0x1c7), *(P2+0x1914) (partner DAT_002f7900)
    pub ev_1c7: f32,
    pub partner_1914: i32,
    /// jtbs 0x13/0x14: FUN_001974c8(jt+0xec) != 0, FUN_00141ea8(jt, 0x1b)
    pub glide_stop: bool,
    pub mask_1b: u32,
    /// jtbs 0x16: FUN_001441a0(jt), FUN_00141ea8(jt, 0xc)
    pub f_1441a0: f32,
    pub mask_c: u32,
    /// jts 3 enter: pos(jt+0x19a4) - FUN_00190c58(jt) (initial grip offset jt+0x1960)
    pub grip_off: V3,
    /// jts 3: constraint anchor (xf jt+0x19cc)+0x70 and constraint frame (jt+0x19c0)+0xa0
    pub anchor_19cc: V3,
    pub frame_19c0: Option<M3>,
    /// (jt+0x1f10)+0x78 anim speed (jts 0x15 walk)
    pub speed_1f10: f32,
}

/// Port-owned JT fields of the P5 states.
#[derive(Clone, Debug)]
pub struct P5 {
    // ---- jts 3 swing
    /// jt+0x18f4 time of the last stick-in-direction
    pub t_18f4: f32,
    /// jt+0x18f8 pump latch (-1, 0, 2)
    pub f_18f8: i32,
    /// jt+0x18fc facing reversed
    pub f_18fc: bool,
    /// jt+0x1900 != 0 (rope hangs on a physics body; 0x18f958 unported)
    pub rope_1900: bool,
    /// jt+0x1910 pendulum accel
    pub a_1910: V3,
    /// jt+0x1960 grip offset (decays to 0)
    pub off_1960: V3,
    /// jt+0x1998 / 0x199c blend params, jt+0x1970 / 0x197c / 0x1988 weights
    pub w_1998: f32,
    pub w_199c: f32,
    pub w_1970: f32,
    pub w_197c: f32,
    pub w_1988: f32,
    /// value written to (jt+0x1030)+0x40 (anim rate) by 0x190520
    pub anim_rate: Option<f32>,
    /// FUN_0012d108(jt+0x1a00, q) driven-dof target (Euler of 0x11bff0)
    pub dof_1a00: V3,
    /// jt+0x1b0 target rotation (FUN_00204660 / 0x185e40 / 0x18df60)
    pub rot_1b0: M3,
    // ---- jts 4
    /// (jt+0xdf0)+4 kind
    pub kind_df4: i32,
    // ---- jts 8
    pub s8: Jts8,
    // ---- blends (jts 5 / 10 / 0x15)
    /// jt+0x1038 side (-1 undecided)
    pub side_1038: i32,
    pub blend_13d0: Blend8,
    pub blend_1ac0: Blend8,
    pub blend_1e10: Blend8,
    /// jt+0x180
    pub f_180: f32,
    /// jt+0x1a0 anim phase saved on leaving 1/10/0xc
    pub phase_1a0: f32,
    // ---- jts 0x15 / 0x16
    /// jt+0x1f18 surface object present
    pub surface_1f18: bool,
    /// jt+0x1fa0 ride offset (T local)
    pub off_1fa0: V3,
    /// jt+0x1f94 lean, 0x1f98 mag, weights 0x1f60 / 0x1f6c / 0x1f78 / 0x1f84
    pub f_1f94: f32,
    pub f_1f98: f32,
    pub w_1f60: f32,
    pub w_1f6c: f32,
    pub w_1f78: f32,
    pub w_1f84: f32,
    // ---- jts 0x18
    pub f_202c: f32,
    pub f_2014: f32,
    pub f_2020: f32,
    // ---- jtbs 6..9 bookkeeping jt+0x18e4 / 0x18e8
    pub f_18e4: f32,
    pub f_18e8: f32,
    // ---- JMP stuck / warp
    /// jt+0x131c rate-limit time of 0x19a098
    pub t_131c: f32,
    /// jt+0x9e0 safe points
    pub safe_points: Vec<SafePoint>,
    /// jt+0x240 HOFS
    pub hofs_240: V3,
    /// jt+0x490 = jmp+0x1d0 jump-to-point destination
    pub dest_490: Option<SafePoint>,
    /// (jmp+0x1f0)+0x1c deadline of the jump to point
    pub deadline_1f0: f32,
    /// jmp+0x1d8
    pub f_1d8: bool,
    /// FEEL R92: FUN_0019b110(jmp, obj) forced pick requested by the jtbs 0x15 body (run by the caller with the
    /// collision world)
    pub pick_15: Option<usize>,
}
impl Default for P5 {
    fn default() -> Self {
        Self {
            t_18f4: 0.0,
            f_18f8: -1,
            f_18fc: false,
            rope_1900: false,
            a_1910: ZERO,
            off_1960: ZERO,
            w_1998: 1.0,
            w_199c: 1.0,
            w_1970: 0.0,
            w_197c: 0.0,
            w_1988: 0.0,
            anim_rate: None,
            dof_1a00: ZERO,
            rot_1b0: IDENT,
            kind_df4: -1,
            s8: Jts8::default(),
            side_1038: -1,
            blend_13d0: Blend8::default(),
            blend_1ac0: Blend8::default(),
            blend_1e10: Blend8::default(),
            f_180: 0.0,
            phase_1a0: 0.0,
            surface_1f18: false,
            off_1fa0: ZERO,
            f_1f94: 0.0,
            f_1f98: 0.0,
            w_1f60: 0.0,
            w_1f6c: 0.0,
            w_1f78: 0.0,
            w_1f84: 0.0,
            f_202c: 0.0,
            f_2014: 0.0,
            f_2020: 0.0,
            f_18e4: 0.0,
            f_18e8: 0.0,
            t_131c: f32::MIN,
            safe_points: Vec::new(),
            // WP-P21: jt+0x240 = (0, 0, -75) after JT load 0x1885b8 is what 0x19b110 / 0x193a68 / 0x19faf8 read, but
            // applied here it lifts Sly 150 above tightropes / edges (0x193a68 and the 0x19b110 tail both subtract
            // it; shots/P21/edge_hofs75 vs edge_hofs0) while the JT model origin is at the feet: kept ZERO (REPLACED,
            // see ENGINE_PORT.md P21)
            hofs_240: ZERO,
            dest_490: None,
            deadline_1f0: 0.0,
            f_1d8: false,
            pick_15: None,
        }
    }
}

/// Force output of one JT for the frame (added to the driver accumulators).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Acc {
    /// FUN_001fac30 linear accel (xf jt+0x58 +0xf0)
    pub lin: V3,
    /// angular accel (xf jt+0x5c +0xf0)
    pub ang: V3,
    /// vt+0x124 set angular velocity
    pub set_w: Option<V3>,
}

// ============================================================================================ SetJtJts
/// 0x0018c250 SetJtJts with the P5 enter / exit / jtbs actions: exit(old) -> enter(new) -> mod.rs
/// bookkeeping (`Jt::FUN_0018c250`) -> jtbs switch (only outside jts 2: JMP body states go through 0x195fb8).
pub fn FUN_0018c250_p5(jt: &mut Jt, st: &mut P5, w: &P5World, jts: i32, jtbs: i32, dt: f32, anim: &mut dyn JtAnim) {
    if jt.jts == jts && jt.jtbs == jtbs {
        return;
    }
    let old = jt.jts;
    jt.last_target_18c250(jts);
    if jts != old {
        FUN_0018c250_exit(jt, st, w, old, jts, jtbs, &mut *anim);
        FUN_0018c250_enter(jt, st, w, old, jts, dt, anim);
    }
    let jtbs_changed = jt.jtbs != jtbs;
    jt.FUN_0018c250(jts, jtbs, anim);
    if jtbs_changed && jts != JTS_JUMP {
        FUN_0018c250_jtbs(jt, st, jtbs, anim);
    }
}

/// 0x0018c250 EXIT actions of the old jts (spec_states §2) for jts 3,4,6,8,10,0xc,0x15,0x16,0x18.
/// (jts 5 / 0xe have none.) Engine calls are recorded. WP-P16: + 0xd / 0x11 (p16), jts 3 cane reset is the
/// real FUN_0018e540(-1) (leaving jtcs 1 plays the target's swing sound).
pub fn FUN_0018c250_exit(jt: &mut Jt, st: &mut P5, w: &P5World, old: i32, new: i32, jtbs: i32, anim: &mut dyn JtAnim) {
    match old {
        // WP-P18: jts 0 cane exit chain; jts 9 clears +0x18c0
        0 => jt.FUN_0018c250_exit_0(jtbs, anim),
        9 => jt.p18.aim_18c0 = None,
        3 => {
            rec(jt, 0x1c, 6); // vt(target)+0x1c(target, 6) detach message
            rec(jt, 0x00133448, 0);
            jt.FUN_0018e540(-1, anim);
            rec(jt, 0x00230100, 0); // remove constraint jt+0x19c0 / jt+0x1a00
            if st.rope_1900 {
                rec(jt, 0x00229738, 0);
                rec(jt, 0x002296c0, 0);
                st.rope_1900 = false;
            }
            rec(jt, 0x00140c88, 0);
        }
        4 => {
            rec(jt, 0x001a0b40, -1);
            st.kind_df4 = -1;
        }
        6 => jt.FUN_001375d0(-1, anim), // WP-P25
        8 => {
            FUN_0019eab0(jt, st, w, -1);
            // FUN_0018e540(jt, new == 2 && jt+0x14d8 ? 0xb : -1)
            rec(jt, 0x0018e540, if new == JTS_JUMP { 0xb } else { -1 });
        }
        1 | 10 | 0xc => {
            // jt+0x1a0 = anim t / anim duration
            st.phase_1a0 = if w.anim.dur != 0.0 { w.anim.t / w.anim.dur } else { 0.0 };
        }
        0x15 => {
            rec(jt, 0x001f5720, 0);
            rec(jt, 0x1c, 6);
            rec(jt, 0x00140c88, 0);
            st.surface_1f18 = false;
        }
        0x16 => {
            rec(jt, 0x001f5720, 0);
            rec(jt, 0x1c, 6);
            rec(jt, 0x00133448, 0);
            rec(jt, 0x00140c88, 0);
        }
        // WP-P16: 0xd (+0x18c0 = 0) / 0x11 (+0x1d30 = 0, job event 0x38)
        0xd | 0x11 => jt.FUN_0018c250_exit_p16(old),
        0x18 => {
            jt.mv[3] = 1.0; // jt+0x200
            jt.joy.angle = jt.yaw; // joy+0x18 = jt+0x1a8
        }
        _ => {}
    }
}

/// 0x0018c250 ENTER actions of the new jts for 3,4,6,8,10,0xc,0xe,0x15,0x16,0x18.
pub fn FUN_0018c250_enter(jt: &mut Jt, st: &mut P5, w: &P5World, _old: i32, new: i32, dt: f32, anim: &mut dyn JtAnim) {
    match new {
        3 => {
            st.f_18f8 = -1;
            st.w_1998 = 1.0;
            st.w_199c = 1.0;
            st.t_18f4 = jt.t;
            st.dof_1a00 = ZERO; // (jt+0x1a00)+0x20..0x28
            anim.play_aseg(0x1994, 3, -1); // startf 0.5, blend 0
            rec(jt, 0x00230018, 0); // add constraint
            st.off_1960 = w.grip_off;
            FUN_00190e18(jt, st, w, dt);
            rec(jt, 0x1c, 5); // attach message
            rec(jt, 0x001333c0, 0);
            let tg = w.swing.unwrap_or_default();
            let a = FUN_00190be0(&tg);
            match tg.mode_124 {
                0 => {
                    if FUN_0011cf60(a - jt.joy.angle).abs() < PI * 0.5 {
                        face(jt, st, a);
                        st.f_18fc = false;
                    } else {
                        face(jt, st, FUN_0011cf60(a + PI));
                        st.f_18fc = true;
                    }
                }
                2 => {
                    face(jt, st, a + PI);
                    st.f_18fc = true;
                }
                _ => {
                    face(jt, st, a);
                    st.f_18fc = false;
                }
            }
            jt.FUN_0018e540(1, anim); // WP-P16: real call (jtcs 1 = swing grip)
            jt.v_target = ZERO;
            FUN_0018fc60(jt, st, w, dt);
            rec(jt, 0x00140c88, 1);
        }
        4 => {
            rec(jt, 0x0019cd38, -1);
            rec(jt, 0x0018df50, 0);
        }
        6 => {
            // WP-P25: FUN_001375d0(jt+0x1ff0, FUN_001374f0(jt+0x1ff0))
            let side = jt.FUN_001374f0();
            jt.FUN_001375d0(side, anim);
        }
        8 => FUN_0019eab0(jt, st, w, 0),
        // WP-EP1f: 9 enter = FUN_0018e540(jt, -1) + FUN_0018f688(jt) (aim pick within 0.25 / 1.0, mask 8, face it)
        9 => {
            jt.FUN_0018e540(-1, anim);
            jt.FUN_0018f688();
        }
        10 => {
            anim.play_aseg(0x1bc0, 10, -1); // mode 9, start = jt+0x1a0 * dur
            st.side_1038 = -1;
        }
        0xc => anim.play_aseg(0x1c34, 0xc, -1),
        // WP-P16: 0xd / 0xe / 0x11 enter (real FUN_0018e540(-1), +0x190..+0x198 = 0, state ASEG)
        0xd | 0xe | 0x11 => jt.FUN_0018c250_enter_p16(new, anim),
        0x15 => {
            rec(jt, 0x00185310, 0);
            rec(jt, 0x001f5720, 1);
            set_vel(jt, ZERO);
            set_w(jt, ZERO);
            jt.v_target = ZERO;
            rec(jt, 0x1c, 5);
            st.surface_1f18 = true;
        }
        0x16 => {
            anim.play_aseg(0x1f90, 0x16, -1);
            rec(jt, 0x0018e540, -1);
            rec(jt, 0x001f5720, 1);
            set_vel(jt, ZERO);
            set_w(jt, ZERO);
            rec(jt, 0x1c, 5);
            rec(jt, 0x001333c0, 0);
            rec(jt, 0x00140c88, 1);
        }
        0x18 => {
            st.f_202c = 0.0;
            st.f_2014 = 0.0;
            st.f_2020 = 1.0;
            rec(jt, 0x001384c0, 0x2038);
            anim.play_aseg(0x2038, 0x18, -1);
            let a = jt.joy.angle;
            face(jt, st, a);
            let v = FUN_0011e1a0(jt.yaw, DAT_002BF3E8, 0.0);
            set_vel(jt, v);
            jt.v_target = ZERO;
        }
        _ => {}
    }
}

/// 0x0018c250 jtbs switch (jtbs 3..0xd; 0..2 are the mod.rs anim). Only for jts != 2.
pub fn FUN_0018c250_jtbs(jt: &mut Jt, st: &mut P5, jtbs: i32, anim: &mut dyn JtAnim) {
    let jts = jt.jts;
    match jtbs {
        3 => anim.play_aseg(0x1950, jts, jtbs),
        4 => {
            anim.play_aseg(0x1bc4, jts, jtbs);
            jt.v_target = ZERO;
        }
        5 => {
            st.side_1038 = -1;
            jt.v_target = ZERO;
            anim.play_aseg(0x14d0, jts, jtbs);
        }
        6 | 7 => {
            // 0x18d220 cases 6 and 7 both FUN_0018e540(jt, -1) (WP-P13: real call, case 7 was missing)
            jt.FUN_0018e540(-1, anim);
            anim.play_aseg(if jtbs == 6 { 0x1d08 } else { 0x1d0c }, jts, jtbs);
            jt.v_target = ZERO;
            st.f_18e8 = 0.0;
            st.f_18e4 = 0.0;
            // WP-P13: jt+0x18e8 / +0x18e4 are the cane flags (p11_cane::Cane)
            jt.cane.f_18e8 = false;
            jt.cane.f_18e4 = false;
        }
        8 | 9 => {
            anim.play_aseg(if jtbs == 8 { 0x1d10 } else { 0x1fe8 }, jts, jtbs);
            st.f_18e4 = 0.0;
            jt.cane.f_18e4 = false; // WP-P13
            jt.v_target = ZERO;
        }
        10 => anim.play_aseg(0x1f14, jts, jtbs),
        0xb => anim.play_aseg(0x1f10, jts, jtbs),
        0xc => jt.mv[3] = DAT_002BF358,
        0xd => jt.mv[3] = DAT_002BF35C,
        _ => {}
    }
}

/// 0x00204660(jt, a) (REPLACED: facing = Jt::yaw, target rot jt+0x1b0 = yaw rotation about z).
fn face(jt: &mut Jt, st: &mut P5, a: f32) {
    jt.yaw = a;
    st.rot_1b0 = FUN_0011b500(UP, a);
}

/// 0x00190be0 1:1: atan2 of the target matrix row 0 (pivot's x axis yaw).
pub fn FUN_00190be0(t: &SwingTarget) -> f32 {
    t.rot[0][1].atan2(t.rot[0][0])
}

/// Swing entry used by 0x18bce0 sweep and 0x19c688 case 0xe: jt+0x18c4 = T;
/// SetJtJts(T+0x124 == 3 ? (0x16, -1) : (3, 1)).
pub fn enter_swing(jt: &mut Jt, st: &mut P5, w: &P5World, dt: f32, anim: &mut dyn JtAnim) {
    let ride = w.swing.is_some_and(|t| t.mode_124 == 3);
    if ride {
        FUN_0018c250_p5(jt, st, w, 0x16, -1, dt, anim);
    } else {
        FUN_0018c250_p5(jt, st, w, 3, 1, dt, anim);
    }
}

// =============================================================================================== jts 3
/// 0x0018fc60 1:1 (jt+0x1900 == 0 path): swing pendulum integrator, writes jt+0x1910.
pub fn FUN_0018fc60(jt: &mut Jt, st: &mut P5, w: &P5World, dt: f32) {
    let Some(tg) = w.swing else { return };
    let (p, rt1, vt) = (tg.pos, tg.rot[1], tg.v_point);
    let d = vsub(jt.phys.pos, p);
    let l = vlen(d);
    let n = if l < EPS { UP } else { vscale(d, 1.0 / l) };
    let v = vel(jt);
    let vr = vsub(v, vt);
    let (rd, _, vperp) = FUN_0011e290(vr, n);
    let ax = norm_or(vcross(n, vperp), AX_X);
    let vpl = vlen(vperp);
    let om = vpl / l;
    let wv = vscale(ax, om);
    let g = jt.phys.gravity;
    let gt = vsub(g, vscale(n, dot(g, n)));
    let ax2 = norm_or(vcross(n, gt), AX_Y);
    let al = vscale(ax2, vlen(gt) / l);
    let k = DAT_002BF4F0[0] + om * (DAT_002BF4F0[1] + om * DAT_002BF4F0[2]);
    let kpar = if k < DAT_002BF500 { DAT_002BF500 } else if DAT_002BF504 < k { DAT_002BF504 } else { k };
    let jb = jt.jtbs;
    let pump = jb == 0 || jb == 2;
    let kperp = if pump { DAT_002BF508 } else { DAT_002BF50C };
    let w1 = vadd(wv, vscale(al, dt));
    let (_, wpar, wperp) = FUN_0011e290(w1, rt1);
    let mut w2 = vadd(vscale(wpar, kpar * dt + 1.0), vscale(wperp, kperp * dt + 1.0));
    if pump && (jt.t - jt.t_jtbs) < DAT_002BF510 {
        let mut s = dt * DAT_002BF514;
        if jb == 0 {
            s = -s;
        }
        if st.f_18fc {
            s = -s;
        }
        let gz = -g[2];
        let h = (gz * d[2] + vpl * vpl * 0.5) / gz;
        let e = (DAT_002BF520[0] + h * (DAT_002BF520[1] + h * DAT_002BF520[2])).clamp(0.0, 1.0);
        w2 = vadd(w2, vscale(rt1, s * e));
    }
    let (lnew, rdnew) = FUN_0011d1c0(l, rd, DAT_002BF544 - tg.f_134, dt, &DAT_002BF530);
    let w3 = vscale(w2, DAT_002BF544.min(l) / DAT_002BF540.max(lnew));
    let rh = FUN_0011b570(vscale(vadd(wv, w3), dt * 0.5));
    let n2 = vmul(n, &rh);
    let vdes = vadd(vadd(vt, vcross(w3, vscale(n2, lnew))), vscale(n2, rdnew));
    let a = vsub(vscale(vsub(vdes, v), 1.0 / dt), g);
    if st.rope_1900 {
        // FUN_0018f958 coupled 2-body solve: unported, single-body result kept
        rec(jt, 0x0018f958, 0);
    }
    st.a_1910 = a;
}

/// 0x001903b8 1:1: jts 3 forces: jt+0x1910 + orientation servo (body z at the pivot, yawed by facing).
pub fn FUN_001903b8(jt: &Jt, st: &P5, w: &P5World, dt: f32) -> Acc {
    let mut out = Acc { lin: st.a_1910, ..Default::default() };
    let Some(tg) = w.swing else { return out };
    let a = FUN_0011ba08(UP, vsub(tg.pos, jt.phys.pos));
    let b = FUN_0011b500(UP, jt.yaw);
    let c = mmul(&b, &a);
    out.ang = FUN_001f59a0(&jt.phys.rot, jt.phys.w, jt.phys.alpha, &c, DAT_002BF548, dt);
    out
}

/// 0x00190520 1:1: jts 3 per-frame input (from 0x18b7e8): facing, stick latch, jtbs machine
/// 0 / 2 pump, 1 coast, 3 still; SetJtJts(3, new); 0x18fc60; jump off; blend weights; anim scrub.
pub fn FUN_00190520(jt: &mut Jt, st: &mut P5, w: &P5World, dt: f32, anim: &mut dyn JtAnim) {
    let tg = w.swing.unwrap_or_default();
    let mut a = FUN_00190be0(&tg);
    if st.f_18fc {
        a = FUN_0011cf60(a + PI);
    }
    face(jt, st, a);
    let t = jt.t;
    let da = FUN_0011cf60(jt.joy.angle - jt.yaw);
    let (mut fwd, mut back) = (false, false);
    if jt.joy.mag > 0.3 {
        if da.abs() < 1.0471976 {
            fwd = true;
            st.t_18f4 = t;
        } else if da.abs() > 2.0943952 {
            back = true;
            st.t_18f4 = t;
        }
    } else {
        st.f_18f8 = -1;
    }
    let u = norm_or(vsub(tg.pos, jt.phys.pos), UP);
    let b = FUN_0011b500(UP, jt.yaw);
    let l = to_local(u, &b);
    let ang = FUN_0011cf60(l[2].atan2(l[0]) - PI * 0.5);
    let sp = vlen(vel(jt));
    let t_in = t - jt.t_jtbs; // jt+0x18d4
    let jb = jt.jtbs;
    let mut new = jb;
    match jb {
        0 => {
            if back {
                new = 2;
            } else if fwd {
                if t_in > DAT_002BF554 {
                    st.f_18f8 = 0;
                    new = 1;
                }
            } else if t - st.t_18f4 > DAT_002BF55C {
                new = 1;
            }
        }
        2 => {
            if fwd {
                new = 0;
            } else if back {
                if t_in > DAT_002BF558 {
                    st.f_18f8 = 2;
                    new = 1;
                }
            } else if t - st.t_18f4 > DAT_002BF55C {
                new = 1;
            }
        }
        1 | 3 => {
            if fwd && st.f_18f8 != 0 {
                st.f_18f8 = -1;
                new = 0;
            } else if back && st.f_18f8 != 2 {
                st.f_18f8 = -1;
                new = 2;
            } else if jb == 1 && ang.abs() < DAT_002BF560 && sp < DAT_002BF568 {
                new = 3;
            } else if jb == 3 && ang.abs() > DAT_002BF564 && sp > DAT_002BF56C {
                new = 1;
            }
        }
        _ => {}
    }
    FUN_0018c250_p5(jt, st, w, 3, new, dt, anim);
    FUN_0018fc60(jt, st, w, dt);
    if st.rope_1900 {
        rec(jt, 0x001f5078, 0);
    }
    if jt.joy.FUN_001c4660(t, 0.15, JOY_SWING_JUMP) != 0 && !jt.binoc.active() {
        // jt.vt+0x31c(jt, 3) (false while 0x1443d8); WP-P15: FUN_0019cc50(jt+0xf00, 0, 0, 0)
        jt.joy.FUN_001c4738(JOY_SWING_JUMP);
        jt.FUN_0019cc50(0, 0, 0, anim);
    }
    let (ta, tb) = match jt.jtbs {
        3 => return,
        0 => (1.0, 1.0),
        1 => (0.0, if st.w_1998 > 0.5 { 1.0 } else { 0.0 }),
        2 => (1.0, 0.0),
        _ => return, // uninitialised registers on the PS2 (unreachable)
    };
    st.w_199c = FUN_0011d018(st.w_199c, ta, dt, DAT_002BF570, None);
    st.w_1998 = FUN_0011d018(st.w_1998, tb, dt, DAT_002BF580, None);
    let (aa, bw) = (st.w_199c, st.w_1998);
    st.w_1970 = aa * bw;
    st.w_1988 = 1.0 - aa;
    st.w_197c = aa * (1.0 - bw);
    rec(jt, 0x001384c0, 0x1994);
    let r = FUN_0011cfe0(ang / DAT_002BF58C, 1.0);
    st.anim_rate = Some(FUN_0011cfe0((w.anim.dur_1994 * (r + 1.0) * 0.5 - w.anim.t) / dt, DAT_002BF590));
}

/// 0x00190e18 1:1: swing orientation: driven-dof target from the pivot direction in the constraint frame
/// (R(1.5 about x) * shortest arc x -> dir, 0x11bff0) and decay of jt+0x1960 (0x11f088 {3000,1000,0.15}).
pub fn FUN_00190e18(jt: &mut Jt, st: &mut P5, w: &P5World, dt: f32) {
    if let Some(tg) = w.swing {
        let d = vsub(tg.pos, w.anchor_19cc);
        let dl = match &w.frame_19c0 {
            Some(f) => to_local(d, f),
            None => d,
        };
        let b = FUN_0011ba08(AX_X, dl);
        let r = FUN_0011b500(AX_X, DAT_002BF594);
        let m = mmul(&r, &b);
        st.dof_1a00 = bff0_m4(&m4(&m));
        rec(jt, 0x0012d108, 0);
    }
    st.off_1960 = FUN_0011f088(st.off_1960, ZERO, dt, DAT_002BF598).0;
}

// ======================================================================================== jts 4 / 6 / 8
/// 0x001a1148 1:1: jts 4 forces by (jt+0xdf0)+4 kind. kind 1/2: float (gravity cancelled, velocity damped
/// to 0); 5/0xb: same horizontally, vertical speed driven to -jt+0x198. Others: nothing.
pub fn FUN_001a1148(jt: &Jt, kind: i32, dt: f32) -> V3 {
    let g = jt.phys.gravity;
    let v = vel(jt);
    match kind {
        1 | 2 => vadd(vscale(g, -1.0), FUN_001f5e10(jt.phys.pos, v, ZERO, None, ZERO, Some(DAT_002BFA90), dt)),
        5 | 0xb => {
            let f198 = jt.v_target[2];
            let vz = if f198 != 0.0 { v[2] } else { 0.0 };
            let mut a = vadd(vscale(g, -1.0), FUN_001f5e10(jt.phys.pos, v, ZERO, None, [0.0, 0.0, vz], Some(DAT_002BFA90), dt));
            if f198 != 0.0 {
                a[2] += (-f198 - vz) / dt;
            }
            a
        }
        _ => ZERO,
    }
}

/// 0x00137248 1:1: jts 6 orbit of the target at radius 250 (accel of the relative velocity, limit 10500).
pub fn FUN_00137248(jt: &Jt, tg: Option<Kin>, dt: f32) -> V3 {
    let Some(tg) = tg else { return ZERO };
    let amax = DAT_002BC99C * DAT_002BC99C / DAT_002BC998 + DAT_002BC9A0;
    let vr = vsub(vel(jt), tg.v);
    let mut d = vsub(jt.phys.pos, tg.pos);
    d = vsub(d, vscale(UP, dot(d, UP)));
    let r = vlen(d);
    let n = if r < EPS { AX_X } else { vscale(d, 1.0 / r) };
    let wz = vcross(n, vr)[2];
    let rd = dot(vr, n);
    let mut vrad = 0.0;
    FUN_0011d018(r, DAT_002BC998, dt, DAT_002BC9A8, Some(&mut vrad));
    let dv = FUN_0011cfe0(vrad - rd, DAT_002BC9A0 * dt);
    let vn = vadd(vr, vscale(n, dv));
    let m = FUN_0011b500(UP, (wz / r) * dt);
    let a = vscale(vsub(vmul(vn, &m), vr), 1.0 / dt);
    FUN_0011e9f8(amax, a)
}

/// 0x0019df70: jts 8 orientation driver (sets the angular velocity, no linear force). 1:1 except
/// REPLACED 0x117308 (states 0 / 4: rotation spline from (R, w) to (M, 0) over the remaining anim time)
/// by a constant-rate step of angle * dt / rem toward M.
pub fn FUN_0019df70(jt: &Jt, st: &P5, w: &P5World, dt: f32) -> Option<V3> {
    let m = w.path8?; // FUN_001c3500(obj, u, s+0x18 == 0, &M)
    let r = jt.phys.rot;
    let rn = if st.s8.state == 0 || st.s8.state == 4 {
        let rem = dt.max(w.anim.dur - w.anim.t); // MAX.S
        let (ang, axis) = FUN_0011bcf8(&FUN_0011bc00(&r, &m));
        mmul(&r, &FUN_0011b500(axis, ang * (dt / rem).min(1.0)))
    } else {
        m
    };
    let (angle, axis) = FUN_0011bcf8(&FUN_0011bc00(&r, &rn));
    Some(vscale(axis, angle / dt))
}

/// 0x0019eab0 (brief, spec_forces §7.1): jts 8 sub-state setter. Bookkeeping 1:1; anim / messages / path
/// sampling recorded.
pub fn FUN_0019eab0(jt: &mut Jt, st: &mut P5, w: &P5World, new: i32) {
    let old = st.s8.state;
    if new == old {
        return;
    }
    match old {
        -1 => {
            rec(jt, 0x1c, 5);
            rec(jt, 0x001333c0, 0);
            rec(jt, 0x00204f78, 1);
            st.s8.dir_18 = w.path8_tangent_z < -0.5;
            rec(jt, 0x0019e110, 0); // u / jt+0x1010 / jt+0x1070 from the path
            rec(jt, 0x00140c88, 1);
            st.s8.obj = true;
        }
        0 => {
            if jt.is_player {
                rec(jt, 0x0018e540, 2); // jt+0x14d4 ? 10 : 2
            }
        }
        _ => {}
    }
    st.s8.state = new;
    st.s8.t_8 = jt.t;
    match new {
        0 => {
            st.s8.f_1c = 0;
            rec(jt, 0x001305b8, 0x74);
        }
        1 => rec(jt, 0x001305b8, 0x78),
        2 | 3 => rec(jt, 0x001305b8, 0x70),
        4 => {
            st.s8.dir_18 = !st.s8.dir_18;
            rec(jt, 0x001305b8, 0x7c);
        }
        -1 => {
            rec(jt, 0x1c, 6);
            rec(jt, 0x00133448, 0);
            rec(jt, 0x00204f78, 0);
            rec(jt, 0x00140c88, 0);
            st.s8.obj = false;
            return;
        }
        _ => {}
    }
    rec(jt, 0x001f15b8, 0);
    jt.v_target = ZERO;
}

// ============================================================================ 0x188960 cases 0xe / 0x18
/// 0x00188960 case 0xe 1:1: track the anim root-motion velocity (vertical too before event 0x16d, with
/// gravity cancelled), accel limit 4000.
pub fn FUN_00188960_e(jt: &Jt, w: &P5World, dt: f32) -> V3 {
    let v = vel(jt);
    let mut vdes = w.anim.root_vel;
    let mut lin = ZERO;
    if w.anim.t < w.anim.ev_16d {
        lin = vscale(jt.phys.gravity, -1.0);
    } else {
        vdes[2] = v[2];
    }
    let a = FUN_0011e9f8(DAT_002BF288, vscale(vsub(vdes, v), 1.0 / dt));
    vadd(lin, a)
}

/// 0x00188960 case 0x18 1:1 (only jtbs 0xd): forward speed -100/s^2, side damping 1/s, velocity turned by
/// the yaw rate, horizontal only, limit 4500.
pub fn FUN_00188960_18(jt: &Jt, dt: f32) -> V3 {
    if jt.jtbs != 0xd {
        return ZERO;
    }
    let v = vel(jt);
    let wz = jt.phys.w[2];
    let r0 = jt.phys.rot[0];
    let (s, _, perp) = FUN_0011e290(v, r0);
    let par = vscale(r0, s - DAT_002BF290 * dt);
    let perp = vscale(perp, 1.0 - DAT_002BF294 * dt);
    let m = FUN_0011b500(UP, wz * dt);
    let u = vmul(vadd(par, perp), &m);
    let mut a = vscale(vsub(u, v), 1.0 / dt);
    a = vsub(a, vscale(UP, dot(a, UP)));
    FUN_0011e9f8(DAT_002BF28C, a)
}

/// 0x00188960 per-jts bodies for jts 3, 4, 6, 8, 0xe, 0x18 (None = not a P5 jts). Base terms: see module doc.
pub fn FUN_00188960_p5(jt: &Jt, st: &P5, w: &P5World, dt: f32) -> Option<Acc> {
    Some(match jt.jts {
        3 => FUN_001903b8(jt, st, w, dt),
        4 => Acc { lin: FUN_001a1148(jt, st.kind_df4, dt), ..Default::default() },
        6 => Acc { lin: FUN_00137248(jt, w.orbit, dt), ..Default::default() },
        8 => Acc { set_w: FUN_0019df70(jt, st, w, dt), ..Default::default() },
        0xe => Acc { lin: FUN_00188960_e(jt, w, dt), ..Default::default() },
        0x18 => Acc { lin: FUN_00188960_18(jt, dt), ..Default::default() },
        _ => return None,
    })
}

// ========================================================================================= per-jts update
/// 0x001a0450 1:1: 8-way locomotion blend. Weights of the side's contributing entries from the velocity
/// goal (max(0, v.vi / (rate vi.vi))), idle entry gets 1 - sum (or normalise), then the velocity goal is
/// re-synthesised from the blend. Returns the new jt+0x190 (FUN_001384c0 push recorded by the caller).
pub fn FUN_001a0450(b: &mut Blend8, side: &mut i32, v190: V3, rate: f32) -> V3 {
    if !b.en[6] || !b.en[7] {
        *side = -1;
    } else if *side == -1 {
        *side = if dot(v190, b.vel[0]) < 0.0 { 1 } else { 0 };
    }
    b.w = [0.0; 8];
    let (idle, list): (usize, &[usize]) = match *side {
        0 => (6, &[2, 4, 0]),
        1 => (7, &[3, 5, 1]),
        _ => {
            if !b.en[7] {
                (6, &[2, 4, 0, 1])
            } else {
                (7, &[3, 5, 0, 1])
            }
        }
    };
    let mut sum = 0.0;
    if rate > 0.0 {
        for &i in list {
            let vi = b.vel[i];
            let wi = (dot(v190, vi) / (rate * dot(vi, vi))).max(0.0);
            b.w[i] = wi;
            sum += wi;
        }
    }
    if sum < 1.0 {
        b.w[idle] = 1.0 - sum;
    } else {
        b.w[idle] = 0.0;
        for &i in list {
            b.w[i] /= sum;
        }
    }
    let mut v = ZERO;
    for k in 0..8 {
        v = vadd(v, vscale(b.vel[k], b.w[k]));
    }
    vscale(v, rate)
}

/// Stick in world: (-joy+0x10, joy+0x14, 0) * camera rows DAT_002ddfa0.
fn stick_world(w: &P5World) -> V3 {
    let s = [-w.stick[0], w.stick[1], 0.0];
    match &w.cam_rows {
        Some(c) => vmul(s, c),
        None => s,
    }
}

/// 0x0018df60 1:1: jts 0x16 ride: orientation = target's, position = target point M * (-jt+0x1fa0),
/// lean weights from the stick in target space.
pub fn FUN_0018df60(jt: &mut Jt, st: &mut P5, w: &P5World, dt: f32) {
    let Some(tg) = w.swing else { return };
    st.rot_1b0 = tg.rot;
    let (_, wv) = FUN_0011c950(dt * 0.5, &jt.phys.rot, &tg.rot, DAT_002BF3F8);
    set_w(jt, wv);
    let target = vadd(vmul(vscale(st.off_1fa0, -1.0), &tg.rot), tg.pos);
    let (_, v) = FUN_0011f088(jt.phys.pos, target, dt * 0.5, DAT_002BF408);
    set_vel(jt, v);
    let sl = to_local(stick_world(w), &tg.rot);
    let f = norm_or(sl, UP)[1];
    st.f_1f94 = f;
    let m = jt.joy.mag;
    st.f_1f98 = m;
    st.w_1f60 = 1.0 - m;
    st.w_1f6c = (-f).max(0.0) * m;
    st.w_1f84 = f.max(0.0) * m;
    st.w_1f78 = (1.0 - f.max(0.0) - (-f).max(0.0)) * m;
    rec(jt, 0x001384c0, 0x1f90);
}

/// Class-T surface of jts 0x15 (FUN_00185310 closest-surface query).
pub trait Surface {
    /// (closest point S, surface normal N pointing to the walker, surface velocity at S)
    fn closest(&self, p: V3) -> (V3, V3, V3);
}

/// 0x00185e40: jts 0x15 walk on surface T. 1:1: stick -> velocity goal from blend axes jt+0x1e40 / 0x1e60
/// times speed, moving test (|v|^2 > 1 or stick), SetJtJts(0x15, 0xb | 10), 0x1a0450, velocity = surface
/// velocity + rows * (-goal), angular servo 0x11c950 {25,0,0.25} at dt/2.
/// REPLACED: target frame (three 0x185310 probes + 0x11b830) by z = surface normal with the facing kept.
/// UNPORTED (recorded): boundary keep 0x1856b8 / 0x1856c0 / 0x185a80.
pub fn FUN_00185e40(jt: &mut Jt, st: &mut P5, w: &P5World, surf: &dyn Surface, dt: f32, anim: &mut dyn JtAnim) {
    let (_s, n, vs) = surf.closest(jt.phys.pos);
    let z = norm_or(n, UP);
    let f = jt.phys.rot[0];
    let x = norm_or(vsub(f, vscale(z, dot(f, z))), AX_X);
    let f2 = [x, vcross(z, x), z];
    st.rot_1b0 = f2;
    let (_, wv) = FUN_0011c950(dt * 0.5, &jt.phys.rot, &f2, DAT_002BE920);
    set_w(jt, wv);
    rec(jt, 0x001856c0, 0);
    let mag = jt.joy.mag;
    let mut v190 = ZERO;
    if mag > 0.0 {
        let s = to_local(stick_world(w), &st.rot_1b0);
        let s = norm_or([s[0], s[1], 0.0], UP);
        let s = vscale(s, mag);
        v190 = vscale(vadd(vscale(st.blend_1e10.vel[1], s[1]), vscale(st.blend_1e10.vel[2], s[0])), w.speed_1f10);
    }
    jt.v_target = v190;
    let moving = dot(v190, v190) > 1.0 || mag != 0.0;
    if moving {
        FUN_0018c250_p5(jt, st, w, 0x15, 0xb, dt, anim);
        let mut side = st.side_1038;
        jt.v_target = FUN_001a0450(&mut st.blend_1e10, &mut side, jt.v_target, w.speed_1f10);
        st.side_1038 = side;
        let v = vadd(vs, vmul(vscale(jt.v_target, -1.0), &st.rot_1b0));
        set_vel(jt, v);
    } else {
        FUN_0018c250_p5(jt, st, w, 0x15, 10, dt, anim);
        set_vel(jt, vs);
    }
}

/// 0x0018bce0 per-jts update switch for the P5 jts (3: 0x190520 input (from 0x18b7e8) + 0x190e18;
/// 5 jtbs 5 / 10: 0x1a0450 blends; 0xc: 0x20bc08 (anim, recorded); 0x15: 0x185e40 (needs `surf`);
/// 0x16: 0x18df60; 0x18: foot IK 0x189c40 / 0x189ec8 (cosmetic, recorded)).
pub fn FUN_0018bce0_p5(jt: &mut Jt, st: &mut P5, w: &P5World, surf: Option<&dyn Surface>, dt: f32, anim: &mut dyn JtAnim) {
    match jt.jts {
        3 => {
            FUN_00190520(jt, st, w, dt, anim);
            if jt.jts == 3 {
                FUN_00190e18(jt, st, w, dt);
            }
        }
        5 if jt.jtbs == 5 => {
            let mut side = st.side_1038;
            jt.v_target = FUN_001a0450(&mut st.blend_13d0, &mut side, jt.v_target, w.anim_rate_or(1.0));
            st.side_1038 = side;
            rec(jt, 0x001384c0, 0x14d0);
            jt.v_target = vscale(jt.v_target, DAT_002BF3D8);
            st.f_180 = 1.0;
        }
        10 => {
            let mut side = st.side_1038;
            jt.v_target = FUN_001a0450(&mut st.blend_1ac0, &mut side, jt.v_target, w.anim_rate_or(1.0));
            st.side_1038 = side;
            rec(jt, 0x001384c0, 0x1bc0);
        }
        0xc => rec(jt, 0x0020bc08, 0x1c30),
        0x15 => {
            if let Some(s) = surf {
                FUN_00185e40(jt, st, w, s, dt, anim);
            }
        }
        0x16 => FUN_0018df60(jt, st, w, dt),
        0x18 => {
            rec(jt, 0x00189c40, 0x1520);
            rec(jt, 0x00189c40, 0x1620);
            rec(jt, 0x00189ec8, 0x1520);
            rec(jt, 0x00189ec8, 0x1620);
        }
        _ => {}
    }
}

impl P5World {
    /// (jt+0x1030)+0x40 playback rate. REPLACED source: not tracked by the port; `speed_1f10` when set.
    fn anim_rate_or(&self, d: f32) -> f32 {
        if self.speed_1f10 > 0.0 { self.speed_1f10 } else { d }
    }
}

// ================================================================================= JMP bodies (0x197500)
fn jmp_set(jt: &mut Jt, jtbs: i32, anim: &mut dyn JtAnim) {
    jt.FUN_0018c250(JTS_JUMP, jtbs, anim); // = 0x195fb8 (mod.rs set_jtbs)
}

/// 0x00197500 loop step (a): jmp+0xc target inactive -> 0x19c828. Returns true when it fired.
pub fn FUN_00197500_pre(jt: &mut Jt, w: &P5World, anim: &mut dyn JtAnim) -> bool {
    if w.jmp_target_active == Some(false) {
        jt.FUN_0019c828(anim);
        return true;
    }
    false
}

/// 0x00197500 switch bodies for jtbs 5, 6, 8, 9..0x10, 0x12, 0x13, 0x14, 0x15, 0x16 (1:1). Returns false
/// when the jtbs is not one of these (mod.rs owns 0..4, 7, 0x11). The caller keeps the restart loop.
pub fn FUN_00197500_p5(jt: &mut Jt, st: &mut P5, w: &P5World, dt: f32, anim: &mut dyn JtAnim) -> bool {
    let t = jt.t;
    let ground = grounded(jt);
    match jt.jtbs {
        5 => {
            if 0.25 < t - jt.t_jtbs && ground {
                FUN_0018c250_p5(jt, st, w, 0, -1, dt, anim); // vt+0x2c4(jt, 0)
            } else {
                FUN_00197500_tail5(jt, st, w, anim);
            }
        }
        6 => {
            if w.jmp_target_17c == 3 {
                jmp_set(jt, 4, anim);
            } else if jt.t_aseg_end < t {
                FUN_0019c688_p5(jt, st, w, dt, anim);
            }
        }
        8 => {
            if jt.t_aseg_end < t {
                jmp_set(jt, 4, anim);
            }
        }
        9..=0x10 => {
            if jt.t_aseg_end < t {
                FUN_0019c688_p5(jt, st, w, dt, anim);
            }
        }
        0x12 => {
            if jt.t_aseg_end - w.ev_1c7 <= t && w.partner_1914 != 3 {
                rec(jt, 0x002206c8, 3); // partner carry machine -> 3
            } else if jt.t_aseg_end < t {
                FUN_0019c688_p5(jt, st, w, dt, anim);
            }
        }
        0x13 | 0x14 => {
            let m = w.mask_1b | 8;
            if w.glide_stop || jt.joy.held & m == 0 {
                jmp_set(jt, 4, anim);
            } else if jt.jtbs == 0x13 && w.anim.done {
                jmp_set(jt, 0x14, anim);
            }
        }
        0x15 => FUN_00197500_15(jt, st, w, anim),
        0x16 => {
            if w.f_1441a0 <= 0.0 {
                jmp_set(jt, 4, anim);
            }
            let d = t - jt.t_jtbs;
            if DAT_002BF82C < d && jt.jts == JTS_JUMP {
                if ground {
                    FUN_0018c250_p5(jt, st, w, 0, -1, dt, anim);
                } else if jt.joy.held & w.mask_c == 0 {
                    jmp_set(jt, 4, anim);
                }
            }
            // the PS2 writes the JMP body state even after leaving jts 2; the port keeps jts 2 only
            if DAT_002BF830 < d && jt.jts == JTS_JUMP {
                jmp_set(jt, 4, anim);
            }
        }
        _ => return false,
    }
    true
}

/// 0x00197500 case-5 tail (also reached from jtbs 3/4 airborne without a ledge): stuck -> safe-point warp.
pub fn FUN_00197500_tail5(jt: &mut Jt, st: &mut P5, w: &P5World, anim: &mut dyn JtAnim) {
    let mut r = st.t_131c;
    let stuck = FUN_0019a098(jt, jt.t_jtbs, Some(&mut r), &w.contact_normals);
    st.t_131c = r;
    if stuck {
        FUN_0019f590(jt, st, w, None, anim);
    }
}

/// 0x00197500 case 0x15 (jump to point).
fn FUN_00197500_15(jt: &mut Jt, st: &mut P5, _w: &P5World, anim: &mut dyn JtAnim) {
    let t = jt.t;
    let d = st.dest_490.unwrap_or_default();
    if st.deadline_1f0 < t {
        // (jt+0x18f0 save / restore for class 0x35 has no port effect)
        let mut v = vel(jt);
        v[0] = 0.0;
        v[1] = 0.0;
        set_vel(jt, v);
        jt.FUN_00195a00(0xb, 0, anim);
        return;
    }
    let attack = d.obj_class.is_some_and(|c| ATTACK_CLASSES.contains(&c));
    if d.member && st.deadline_1f0 < t + 0.25 && attack {
        jt.FUN_00195a00(0xb, 0, anim);
        rec(jt, super::CALL_AIR_ATTACK, 1); // FUN_0019b110(jmp, obj)
        st.pick_15 = d.obj; // FEEL R92: run by Jt::FUN_00197500 (needs the collision world)
        return;
    }
    let v = vel(jt);
    if jt.f_28 && v[2] < 0.0 {
        rec(jt, 0x001305b8, 0x134);
        jt.f_28 = false;
        let h_rel = jt.phys.pos[2] - jt.z_contact;
        let vz = FUN_0019a750(6, v[2], jt.phys.gravity[2], h_rel);
        set_vel(jt, [v[0], v[1], vz]);
        return;
    }
    if !d.member && !st.f_1d8 {
        return;
    }
    // FUN_0019f3f8 predicted point -> FUN_0012e0f0 (keyed motion retarget): unported
    rec(jt, 0x0012e0f0, 0);
}

/// 0x0019c688 cases 0xe / 0xf / 0x10 with the P5 states; others delegate to mod.rs.
pub fn FUN_0019c688_p5(jt: &mut Jt, st: &mut P5, w: &P5World, dt: f32, anim: &mut dyn JtAnim) {
    match jt.jtbs {
        0xe => enter_swing(jt, st, w, dt, anim),
        0xf => {
            st.s8.obj = true; // jt+0xf5c = T, jt+0xf60 = jmp+0x14
            FUN_0018c250_p5(jt, st, w, 8, -1, dt, anim); // vt+0x2c4(jt, 8) -> generic 8 -> jts 8
        }
        0x10 => {
            st.surface_1f18 = true;
            FUN_0018c250_p5(jt, st, w, 0x15, 10, dt, anim);
        }
        _ => jt.FUN_0019c688(anim),
    }
}

/// 0x0019a098: STUCK test. 1:1 gating (>= 1 s in the state, 1 s rate limit through `rate`, no ground
/// contact jt+0x208, >= 1 contact); REPLACED 0x11ad30 (bounded LCP) by projected Gauss-Seidel on
/// A_ij = n_i.n_j, b_i = n_i.(-up), x >= 0. Stuck when r = -up + sum x_i n_i has r.z > -0.1.
/// NOTE: jt+0x208 is the best *ground* contact; mod.rs `ground_nz` is the best of all contacts.
pub fn FUN_0019a098(jt: &Jt, t_entered: f32, rate: Option<&mut f32>, normals: &[V3]) -> bool {
    let t = jt.t;
    if t - t_entered < STUCK_T {
        return false;
    }
    if let Some(r) = rate {
        if t - *r < STUCK_T {
            return false;
        }
        *r = t;
    }
    // FEEL R82: +0x208 is the kind-0xe motor record (0x204768), not any contact: the port gated on `ground_nz` (best
    // of ALL contacts), so a JT wedged mid-air on steep faces (no motor record, contacts != 0) never ran the test and
    // hung in the fall pose (USER PLAYTEST 6, canal). SLY2_FEEL_R82STUCK=0 = old gate.
    let gate = if std::env::var("SLY2_FEEL_R82STUCK").is_ok_and(|v| v == "0") { jt.ground_nz.is_some() } else { jt.rec_208 };
    if gate {
        return false;
    }
    let n = normals.len();
    if n == 0 {
        return false;
    }
    let g = vscale(UP, -1.0);
    let b: Vec<f32> = normals.iter().map(|ni| dot(*ni, g)).collect();
    let mut x = vec![0.0f32; n];
    for _ in 0..200 {
        for i in 0..n {
            let aii = dot(normals[i], normals[i]);
            if aii <= 0.0 {
                continue;
            }
            let mut wi = b[i];
            for j in 0..n {
                wi += dot(normals[i], normals[j]) * x[j];
            }
            x[i] = (x[i] - wi / aii).max(0.0);
        }
    }
    let mut r = g;
    for i in 0..n {
        r = vadd(r, vscale(normals[i], x[i]));
    }
    r[2] > STUCK_RZ
}

/// WP-EP1f: 0x0019feb0(list jt+0x9e0, obj, p) 1:1 over `safe_points` (index 0 = list +0x10, at most 0x20
/// entries): an older entry (index >= 1) of the same object within 50 of `p` moves to index 1 (index 1 itself
/// stays), else every entry shifts down one; index 0 becomes `p`. REPLACED: the entry's object (+8, +0 its
/// +4 word, SO +0x10 bit 51) and time (+0xc = DAT_002dded0) are not kept (`obj_class` = the object stand-in).
pub fn FUN_0019feb0(list: &mut Vec<SafePoint>, obj: Option<i32>, p: V3) {
    let new = SafePoint { pos: p, obj_class: obj, member: false, obj: None };
    let hit = (1..list.len()).find(|&k| list[k].obj_class == obj && vlen(vsub(p, list[k].pos)) < 50.0);
    match hit {
        Some(k) => {
            if k > 1 {
                let e = list.remove(k);
                list.insert(1, e);
            }
            list[0] = new;
        }
        None => {
            list.insert(0, new);
            list.truncate(0x20);
        }
    }
}

/// WP-EP1f: HOFS jt+0x240 = `jt.att.grab` ((0, 0, -75) from P6Level::apply; `P5::hofs_240`, which the
/// 0x19b110 pick reads, stays zero: setting it moves the p14 / p21 edge attaches - open).
/// 0x0019faf8 REPLACED: best safe point = nearest entry of jt+0x9e0 (0x19f7d0 scores with world queries);
/// none -> pos + HOFS with no object.
pub fn FUN_0019faf8(jt: &Jt, st: &P5) -> SafePoint {
    let p = jt.phys.pos;
    st.safe_points
        .iter()
        .copied()
        .min_by(|a, b| vlen(vsub(a.pos, p)).total_cmp(&vlen(vsub(b.pos, p))))
        .unwrap_or(SafePoint { pos: vadd(p, jt.att.grab), obj_class: None, member: false, obj: None })
}

/// What 0x19f590 did.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Warp {
    /// teleported to the point (camera cut), fall jtbs 4
    Teleport(f32),
    /// ballistic jump to the point (JMP jtbs 0x15)
    JumpTo(f32),
}

/// 0x0019f590 (1:1 decision logic; REPLACED pickers see 0x19faf8 / 0x19f3f8): warp to a safe point.
/// dest None + player: flight time tm = 0x196928(1000, -1000, 1, -1, p - pos, g); tm > 1.25 -> teleport
/// (pos = p - HOFS, facing rot, camera cut, FUN_00195a00(0xb)); otherwise jump to the point (kind 10 ->
/// jtbs 0x15 with the 0x195fb8 entry launch).
pub fn FUN_0019f590(jt: &mut Jt, st: &mut P5, _w: &P5World, dest: Option<SafePoint>, anim: &mut dyn JtAnim) -> Warp {
    let given = dest.is_some();
    let d = dest.unwrap_or_else(|| FUN_0019faf8(jt, st));
    if !given && jt.is_player {
        let p = d.pos; // FUN_0019f3f8(0, ..) REPLACED: static point
        let dz = vsub(p, jt.phys.pos);
        let tm = FUN_00196928(1000.0, -1000.0, 1.0, -1.0, dz, jt.phys.gravity);
        if tm > DAT_002BF940 {
            jt.phys.pos = vsub(d.pos, jt.att.grab); // vt+0xec set position
            rec(jt, 0x002046b0, 0);
            jt.phys.rot = FUN_0011b500(UP, jt.yaw); // vt+0xf4(jt, jt+0x1b0)
            rec(jt, 0x0014d8c0, 0); // camera cut
            jt.FUN_00195a00(0xb, 0, anim);
            if d.member && d.obj_class.is_some_and(|c| ATTACK_CLASSES.contains(&c)) {
                rec(jt, 0x00148300, 0);
                rec(jt, super::CALL_AIR_ATTACK, 1);
            }
            rec(jt, 0x0015e1d8, 0);
            return Warp::Teleport(tm);
        }
    }
    st.dest_490 = Some(d);
    jt.FUN_00195a00(10, 0, anim);
    let tf = FUN_00195fb8_enter_15(jt, st);
    Warp::JumpTo(tf)
}

/// 0x0019f540(jt, pos) 1:1: dest = {0, 0, pos}; 0x19f590.
/// FEEL R91: the target-less point goes through 0x19f3f8 (param_3[1] == 0: point - jt+0x240) and 0x195fb8 case 0x15
/// aims d = that - SO position (jt+0x58 +0x70); in the port frame (pos = SO - 45, +0x240 = (0,0,-75)) Sly aims 30 above
/// the point (rec_ep1b intro hop apex/landing +47/+29 over the port). Bentley / Murray (PO frame) unchanged.
/// SLY2_FEEL_R91HOP=0 = old (aims at the point).
pub fn FUN_0019f540(jt: &mut Jt, st: &mut P5, w: &P5World, pos: V3, anim: &mut dyn JtAnim) -> Warp {
    let pos = if jt.po_move.is_none() && std::env::var("SLY2_FEEL_R91HOP").map_or(true, |v| v != "0") {
        vsub(vsub(pos, [0.0, 0.0, super::JT_K248]), jt.hang_so_ofs())
    } else {
        pos
    };
    FUN_0019f590(jt, st, w, Some(SafePoint { pos, obj_class: None, member: false, obj: None }), anim)
}

/// FEEL R92: FUN_001e1e30 case 0x2a on a class-0xb pipe target (text_001e0000.c:2321): dest = {t, ext = t+0x190,
/// pos[0] = ext vt+0x14} with vt+0x14 = FUN_001c3958 (vtable 0x2d56b8: s = FUN_001e6f60(path, FUN_001a62e8(jt)) length,
/// clamped to [100, len - 35] on an open path); the 0x15 aim 0x19f3f8 (member) = vt+0x1c FUN_001c3a08 = path(s) +
/// DAT_002c0d04 (-50) * FUN_001c3500 frame row 0, minus the SO position (port: - hang_so_ofs). At deadline - 0.25
/// the 0x15 body runs FUN_0019b110(jmp, t) (the pipe grab). None = `obj` is no PathBody pick target.
pub fn FUN_0019f590_pipe(jt: &mut Jt, st: &mut P5, w: &P5World, obj: usize, anim: &mut dyn JtAnim) -> Option<Warp> {
    use super::p5_target::TargetGeom;
    let tgt = jt.p5_targets.iter().find(|t| t.obj.id == obj);
    let axis = tgt.and_then(|t| t.attach.as_ref()).map_or([1.0, 0.0, 0.0], |a| a.axis_1c0);
    let path = tgt.and_then(|t| match &t.obj.geom {
        TargetGeom::PathBody { path } => Some(path.clone()),
        _ => None,
    })?;
    let so = vadd(jt.phys.pos, jt.hang_so_ofs());
    let mut s = path.FUN_001e6f60(so).l;
    if !path.looped {
        let hi = path.FUN_001e6648() - 35.0;
        s = if s < 100.0 { 100.0 } else if s <= hi { s } else { hi };
    }
    let (pp, _) = path.FUN_001e6f10(s);
    // FEEL R95: the 1:1 FUN_001c3500 frame (pipe +0x1c0 axis); SLY2_FEEL_R95FRAME=0 = old (row0 = horizontal toward)
    let fr = super::p14::r95_frame_on()
        .then(|| super::p14::frame_1c3500(&path, s, true, axis))
        .flatten()
        .unwrap_or_else(|| super::p14::path_frame(&path, s, false, vsub(pp, so), [jt.yaw.cos(), jt.yaw.sin(), 0.0]));
    let pt = vadd(pp, vscale(fr[0], -50.0));
    if std::env::var_os("SLY2_K22DBG").is_some() { eprintln!("R93DBG pipe so {so:?} s {s} len {} pp {pp:?} row0 {:?} pt {pt:?} spline {} l {:?} pts0 {:?} {:?}", path.FUN_001e6648(), fr[0], path.spline, path.l, path.FUN_001e6f10(0.0).0, path.l.iter().map(|&l| path.FUN_001e6f10(l).0).collect::<Vec<_>>()); }
    let pos = vsub(pt, jt.hang_so_ofs());
    Some(FUN_0019f590(jt, st, w, Some(SafePoint { pos, obj_class: Some(0xb), member: true, obj: Some(obj) }), anim))
}

/// 0x00195fb8 entry of jtbs 0x15 (1:1 except 0x19f3f8 = static point): flight time
/// T = max(0.7, 0x196928(1000, c, 1, e, d, g)) (c, e = 0, 500 for class 0x13 / 0xb points else -1000, -1),
/// v = d / T - g T / 2 (SetVelocity), deadline t + T, face d. Returns T.
pub fn FUN_00195fb8_enter_15(jt: &mut Jt, st: &mut P5) -> f32 {
    let d0 = st.dest_490.unwrap_or_default();
    let (c, e) = if d0.obj_class.is_some_and(|k| k == 0x13 || k == 0xb) { (0.0, 500.0) } else { (-1000.0, -1.0) };
    let g = jt.phys.gravity;
    let d = vsub(d0.pos, jt.phys.pos);
    let tf = FUN_00196928(1000.0, c, 1.0, e, d, g).max(DAT_002BF7A0);
    let v = vsub(vscale(d, 1.0 / tf), vscale(g, 0.5 * tf));
    set_vel(jt, v);
    st.deadline_1f0 = jt.t + tf;
    if std::env::var_os("SLY2_K22DBG").is_some() { eprintln!("R93DBG enter15 t {} pos {:?} dest {:?} cls {:?} tf {tf} g {:?} v {v:?}", jt.t, jt.phys.pos, d0.pos, d0.obj_class, g); }
    rec(jt, 0x0012de58, 5);
    rec(jt, 0x001f6860, 1);
    jt.v_target = ZERO;
    if d[0] != 0.0 || d[1] != 0.0 {
        jt.yaw = d[1].atan2(d[0]); // FUN_002046b0(jt, d)
    }
    tf
}

// ================================================================================================ tests
#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 60.0;

    fn jt_at(p: V3) -> Jt {
        let mut j = Jt::new(p);
        j.phys.gravity = [0.0, 0.0, -1750.0];
        j
    }
    fn close(a: f32, b: f32, e: f32) -> bool {
        (a - b).abs() <= e
    }

    #[test]
    fn swing_at_rest_cancels_gravity() {
        // pivot 230 straight above, at rest, coasting (jtbs 1): a = -g exactly (net zero)
        let mut jt = jt_at([0.0, 0.0, 0.0]);
        jt.jts = 3;
        jt.jtbs = 1;
        let w = P5World { swing: Some(SwingTarget { pos: [0.0, 0.0, 230.0], ..Default::default() }), ..Default::default() };
        let mut st = P5::default();
        FUN_0018fc60(&mut jt, &mut st, &w, DT);
        assert!(close(st.a_1910[0], 0.0, 1e-3) && close(st.a_1910[1], 0.0, 1e-3));
        assert!(close(st.a_1910[2], 1750.0, 1e-2), "{:?}", st.a_1910);
        let acc = FUN_00188960_p5(&jt, &st, &w, DT).unwrap();
        assert_eq!(acc.lin, st.a_1910);
    }

    #[test]
    fn swing_displaced_restoring_force() {
        // 0.3 rad out along +x, at rest: a + g = gravity's tangential part gt * (1 + kpar dt), kpar = -0.25,
        // gt = -1750 sin th (cos th, 0, sin th)
        let th: f32 = 0.3;
        let l = 230.0;
        let mut jt = jt_at([l * th.sin(), 0.0, 230.0 - l * th.cos()]);
        jt.jts = 3;
        jt.jtbs = 1;
        let w = P5World { swing: Some(SwingTarget { pos: [0.0, 0.0, 230.0], ..Default::default() }), ..Default::default() };
        let mut st = P5::default();
        FUN_0018fc60(&mut jt, &mut st, &w, DT);
        let net = vadd(st.a_1910, jt.phys.gravity);
        let k = -1750.0 * th.sin() * (1.0 - 0.25 * DT);
        let gt = [k * th.cos(), 0.0, k * th.sin()];
        for i in 0..3 {
            assert!(close(net[i], gt[i], 0.02 * 1750.0 * th.sin()), "{:?} vs {:?}", net, gt);
        }
        // pivot overhead check of 0x1903b8 orientation target: servo turns about y (towards the pivot)
        let acc = FUN_001903b8(&jt, &st, &w, DT);
        assert!(acc.ang[1].abs() > 1.0 && acc.ang[0].abs() < 1e-3, "{:?}", acc.ang);
        assert!(acc.ang[1] < 0.0); // body z must tilt toward -x (the pivot side)
    }

    #[test]
    fn swing_input_pump_and_latch() {
        let mut jt = jt_at([0.0, 0.0, 0.0]);
        let mut st = P5::default();
        let w = P5World { swing: Some(SwingTarget { pos: [0.0, 0.0, 230.0], ..Default::default() }), ..Default::default() };
        jt.jts = 3;
        jt.jtbs = 1;
        jt.t = 1.0;
        // stick backwards (facing 0 = +x, stick angle pi): jtbs 1 -> 2
        jt.joy.mag = 1.0;
        jt.joy.angle = PI;
        FUN_00190520(&mut jt, &mut st, &w, DT, &mut ());
        assert_eq!((jt.jts, jt.jtbs), (3, 2));
        // keep pushing back for > 1.4 s -> latch 2, coast
        jt.t = 2.5;
        FUN_00190520(&mut jt, &mut st, &w, DT, &mut ());
        assert_eq!((jt.jtbs, st.f_18f8), (1, 2));
        // back again is latched (no 1 -> 2); at rest under the pivot (ang 0, |v| < 80) -> 3 still
        jt.t = 2.6;
        FUN_00190520(&mut jt, &mut st, &w, DT, &mut ());
        assert_eq!((jt.jtbs, st.f_18f8), (3, 2));
        // stick released clears the latch; forward then pumps again (3 -> 0)
        jt.joy.mag = 0.0;
        jt.t = 2.7;
        FUN_00190520(&mut jt, &mut st, &w, DT, &mut ());
        assert_eq!((jt.jtbs, st.f_18f8), (3, -1));
        jt.joy.mag = 1.0;
        jt.joy.angle = 0.0;
        jt.t = 2.8;
        FUN_00190520(&mut jt, &mut st, &w, DT, &mut ());
        assert_eq!(jt.jtbs, 0);
    }

    #[test]
    fn jts4_float_and_orbit() {
        let mut jt = jt_at([0.0; 3]);
        jt.phys.v = [100.0, 0.0, 0.0];
        // kind 1: -g + dv * min(10 + 0.01*100, 60) = (-1100, 0, 1750)
        let a = FUN_001a1148(&jt, 1, DT);
        assert!(close(a[0], -1100.0, 1e-2) && close(a[2], 1750.0, 1e-3), "{:?}", a);
        // orbit: at radius 250, tangential speed 0: no radial correction, no rotation -> a = 0
        jt.phys.v = ZERO;
        jt.phys.pos = [250.0, 0.0, 0.0];
        let a = FUN_00137248(&jt, Some(Kin::default()), DT);
        assert!(vlen(a) < 1e-2, "{:?}", a);
    }

    #[test]
    fn jts18_brake() {
        let mut jt = jt_at([0.0; 3]);
        jt.jts = 0x18;
        jt.jtbs = 0xd;
        jt.phys.v = [500.0, 0.0, 0.0];
        // forward speed loses 100/s^2: a = (-100, 0, 0)
        let a = FUN_00188960_18(&jt, DT);
        assert!(close(a[0], -100.0, 0.1) && close(a[1], 0.0, 1e-3), "{:?}", a);
    }

    #[test]
    fn jump_time_solver() {
        // flat 200 ahead: roots of 875 t^2 - 1000 t = 0 -> 1.142857
        let tm = FUN_00196928(1000.0, -1000.0, 1.0, -1.0, [200.0, 0.0, 0.0], [0.0, 0.0, -1750.0]);
        assert!(close(tm, 8.0 / 7.0, 1e-4), "{tm}");
        // 2000 up: -875 t^2 + 1000 t + 2000 = 0 -> (1000 + sqrt(8e6)) / 1750 = 2.1876
        let tm = FUN_00196928(1000.0, -1000.0, 1.0, -1.0, [0.0, 0.0, 2000.0], [0.0, 0.0, -1750.0]);
        assert!(close(tm, (1000.0 + 8.0e6f32.sqrt()) / 1750.0, 1e-3), "{tm}");
    }

    /// Wedged in a V (two steep walls, no ground) in jtbs 5: stuck test fires exactly 1.0 s after the
    /// state was entered, then the safe-point warp.
    fn wedged() -> (Jt, P5, P5World) {
        let mut jt = jt_at([0.0, 0.0, 100.0]);
        jt.t = 10.0;
        jt.FUN_00195a00(8, 0, &mut ()); // kind 8 -> jtbs 5
        assert_eq!((jt.jts, jt.jtbs), (2, 5));
        jt.ground_nz = None;
        let w = P5World { contact_normals: vec![[0.8, 0.0, 0.6], [-0.8, 0.0, 0.6]], ..Default::default() };
        (jt, P5::default(), w)
    }

    #[test]
    fn stuck_then_teleport_respawn() {
        let (mut jt, mut st, w) = wedged();
        st.safe_points = vec![SafePoint { pos: [0.0, 0.0, 2100.0], ..Default::default() }];
        jt.t = 10.99;
        assert!(FUN_00197500_p5(&mut jt, &mut st, &w, DT, &mut ()));
        assert_eq!(jt.jtbs, 5);
        jt.t = 11.0;
        FUN_00197500_p5(&mut jt, &mut st, &w, DT, &mut ());
        // dz = 2000 -> tm 2.19 > 1.25 -> teleport, fall (kind 0xb -> jtbs 4)
        assert_eq!(jt.phys.pos, [0.0, 0.0, 2100.0]);
        assert_eq!((jt.jts, jt.jtbs, jt.kind), (2, 4, 0xb));
        assert!(jt.calls.contains(&(0x0014d8c0, 0)));
        assert_eq!(st.t_131c, 11.0);
    }

    #[test]
    fn stuck_then_jump_to_point_respawn() {
        let (mut jt, mut st, w) = wedged();
        st.safe_points = vec![SafePoint { pos: [200.0, 0.0, 100.0], ..Default::default() }];
        jt.t = 11.0;
        FUN_00197500_p5(&mut jt, &mut st, &w, DT, &mut ());
        // tm = 8/7 <= 1.25 -> jump to point: jtbs 0x15, flight 8/7 s, v = d/T - g T/2
        assert_eq!((jt.jts, jt.jtbs), (2, 0x15));
        let tf = 8.0f32 / 7.0;
        assert!(close(st.deadline_1f0, 11.0 + tf, 1e-4));
        let v = vel(&jt);
        assert!(close(v[0], 200.0 / tf, 1e-2) && close(v[2], 875.0 * tf, 1e-2), "{:?}", v);
        // before the deadline nothing changes, after it: v.xy = 0 and fall (jtbs 4)
        jt.f_28 = false;
        jt.t = 11.0 + tf - 0.01;
        FUN_00197500_p5(&mut jt, &mut st, &w, DT, &mut ());
        assert_eq!(jt.jtbs, 0x15);
        jt.t = 11.0 + tf + 0.01;
        FUN_00197500_p5(&mut jt, &mut st, &w, DT, &mut ());
        assert_eq!((jt.jtbs, jt.kind), (4, 0xb));
        let v = vel(&jt);
        assert_eq!((v[0], v[1]), (0.0, 0.0));
    }

    #[test]
    fn not_stuck_free_fall_or_grounded() {
        let (mut jt, mut st, mut w) = wedged();
        jt.t = 12.0;
        w.contact_normals = vec![[1.0, 0.0, 0.0]]; // one vertical wall: r = -up -> falls freely
        FUN_00197500_p5(&mut jt, &mut st, &w, DT, &mut ());
        assert_eq!(jt.jtbs, 5);
        // rate limit: a second test within 1 s of the last one is skipped
        let mut r = 11.5;
        assert!(!FUN_0019a098(&jt, 0.0, Some(&mut r), &[[0.8, 0.0, 0.6], [-0.8, 0.0, 0.6]]));
        // landing (grounded, > 0.25 s) leaves the jump through generic state 0
        jt.ground_nz = Some(1.0);
        FUN_00197500_p5(&mut jt, &mut st, &w, DT, &mut ());
        assert_eq!((jt.jts, jt.jtbs), (0, -1));
    }

    #[test]
    fn jtbs_16_timers_and_glide() {
        let mut jt = jt_at([0.0; 3]);
        let mut st = P5::default();
        let mut w = P5World { f_1441a0: 1.0, mask_c: 0x10, ..Default::default() };
        jt.t = 5.0;
        jt.FUN_00195a00(0x11, 0, &mut ()); // kind 0x11 -> jtbs 0x16
        assert_eq!(jt.jtbs, 0x16);
        jt.joy.held = 0x10;
        jt.t = 7.9;
        FUN_00197500_p5(&mut jt, &mut st, &w, DT, &mut ());
        assert_eq!(jt.jtbs, 0x16);
        jt.t = 8.01; // > 3.0 s
        FUN_00197500_p5(&mut jt, &mut st, &w, DT, &mut ());
        assert_eq!(jt.jtbs, 4);
        // paraglide 0x13: held R1 (8) keeps it, anim done -> 0x14, release -> 4
        jmp_set(&mut jt, 0x13, &mut ());
        jt.joy.held = 8;
        w.anim.done = true;
        FUN_00197500_p5(&mut jt, &mut st, &w, DT, &mut ());
        assert_eq!(jt.jtbs, 0x14);
        jt.joy.held = 0;
        FUN_00197500_p5(&mut jt, &mut st, &w, DT, &mut ());
        assert_eq!(jt.jtbs, 4);
    }

    #[test]
    fn arrival_0e_enters_swing_or_ride() {
        let mut jt = jt_at([0.0; 3]);
        let mut st = P5::default();
        let mut w = P5World { swing: Some(SwingTarget { pos: [0.0, 0.0, 230.0], ..Default::default() }), ..Default::default() };
        jmp_set(&mut jt, 0xe, &mut ());
        jt.t_aseg_end = -1.0;
        jt.t = 1.0;
        FUN_00197500_p5(&mut jt, &mut st, &w, DT, &mut ());
        assert_eq!((jt.jts, jt.jtbs), (3, 1));
        assert!(jt.calls.contains(&(0x00230018, 0)));
        assert_eq!(st.f_18f8, -1);
        // leave via generic 0 -> exit actions remove the constraint
        FUN_0018c250_p5(&mut jt, &mut st, &w, 0, -1, DT, &mut ());
        assert!(jt.calls.contains(&(0x00230100, 0)));
        w.swing.as_mut().unwrap().mode_124 = 3;
        jmp_set(&mut jt, 0xe, &mut ());
        FUN_00197500_p5(&mut jt, &mut st, &w, DT, &mut ());
        assert_eq!((jt.jts, jt.jtbs), (0x16, -1));
    }

    #[test]
    fn blend8_weights() {
        let mut b = Blend8::default();
        b.en = [true; 8];
        b.vel[0] = [100.0, 0.0, 0.0];
        b.vel[2] = [0.0, 100.0, 0.0];
        b.vel[4] = [0.0, -100.0, 0.0];
        let mut side = -1;
        // half speed forward: w0 = 50*100 / (1 * 10000) = 0.5, idle 6 = 0.5, v = 50
        let v = FUN_001a0450(&mut b, &mut side, [50.0, 0.0, 0.0], 1.0);
        assert_eq!(side, 0);
        assert!(close(b.w[0], 0.5, 1e-6) && close(b.w[6], 0.5, 1e-6));
        assert!(close(v[0], 50.0, 1e-4));
        // over speed is normalised
        let v = FUN_001a0450(&mut b, &mut side, [100.0, 100.0, 0.0], 1.0);
        assert!(close(b.w[0], 0.5, 1e-6) && close(b.w[2], 0.5, 1e-6) && b.w[6] == 0.0);
        assert!(close(v[0], 50.0, 1e-4) && close(v[1], 50.0, 1e-4));
    }

    #[test]
    fn jts18_enter_exit() {
        let mut jt = jt_at([0.0; 3]);
        let mut st = P5::default();
        let w = P5World::default();
        jt.joy.angle = 0.5;
        FUN_0018c250_p5(&mut jt, &mut st, &w, 0x18, 0xd, DT, &mut ());
        assert_eq!(jt.mv[3], DAT_002BF35C);
        let v = vel(&jt);
        assert!(close(v[0], 1000.0 * 0.5f32.cos(), 1e-2) && close(v[1], 1000.0 * 0.5f32.sin(), 1e-2));
        jt.yaw = 1.0;
        FUN_0018c250_p5(&mut jt, &mut st, &w, 0, -1, DT, &mut ());
        assert_eq!((jt.mv[3], jt.joy.angle), (1.0, 1.0));
    }
}
