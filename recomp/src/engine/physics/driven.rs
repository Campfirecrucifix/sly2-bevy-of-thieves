//! WP-E7: driven / animated xfs of the per-xf integrator 0x22de88 (kinds 4..0xb), parent frames 0x22fcc8,
//! attachment walk 0x229798 and the game RNG (newlib rand 0x2342a0 -> 0x11d3c0 -> 0x11d3f8 / 0x11d448).
//! Layout of one xf (0x100 B, obj+0x54, count obj+0x50): +0x00..+0x2f rows, +0x30 pos, +0x80 hermite segment,
//! +0x94 dof (1/4/5/9/10 translation, 6/7 rotation (euler), 0xb shear, 0xc scale), +0x98 kind, +0x9a source
//! sub-kind, +0xa0 parent, +0xa4 generation, +0xa8 deadline (kind 9), +0xac u16 speed, +0xb0 velocity,
//! +0xc0 target value (sub 5: target matrix +0xc0..+0xef), +0xd0 target velocity.
//! 1:1: kind 4 (mirror: pos = -src.pos, generation of src), kinds 5/6 (value from the source sub-kind through
//! 0x22c8c0; sub 5 copies the matrix; 0x20006 -> 0x2303a0), kinds 7/8/9/0xb linear (dof < 6) and angular
//! (dof 6/7) target following: distance/angle moved by 0x11d018 with profile {speed*c0 + |vt|, speed*c1 + |vt|, c2}
//! (c = 0x2c4600 {1.5, 0, 0.5} translation, 0x2c4610 {0.005, 0, 0.5} dof 0xb/0xc, 0x2c45f0 {0.015, 0, 0.5}
//! rotation, sub 3 rotation without |wt|), kind 9 deadline (s >= L dt / (deadline - now + dt)), reached ->
//! v = target velocity and 0x2303a0 (kind != 8, sub 1/2; kind 0xb only sub 2 for rotation), xf flag +0x40 & 0x10
//! = frozen (no motion). 0x22fcc8 (v * R_from then * R_to^T, 0x11b340 transpose). 0x229798 (walk +0x138 list
//! calling slot 70 +0x23c; default 0x2297f0 = no-op).
//! PART-3 (1:1 from the decompile): kind 0xb acceleration clamps (linear 0x2c461c 6.0 / 0x2c4620 0.02 per
//! speed unit for dof 0xb/0xc; angular 0x2c45fc 0.06 with expmap 0x11b570 of (v + v') dt/2), not-reached
//! velocity v = Tv + n (vel - vt), angular target velocity via 0x22cad0 (euler rate -> w; sub 5 = 0),
//! sub 1 hermite 0x117120(+0x84 - +0x80, *(+0x88) - +0x80, *(+0x8c), +0xc0, +0xd0, +0xe0, +0xf0), the global
//! generation counter DAT_003e11e8 and the 0x2303a0 argument (6 / 10 / 0xc) in `reach_k`.
//! REPLACED: slot-70 attach 0x2297f8 (1384 B) by a rigid attachment `Attachment`.
#![allow(non_snake_case)]
use super::driver::{FUN_0011b500, FUN_0011b570, M3, IDENT};
use super::dynamics::FUN_0011d018;
use super::geom::{dot, V3};
use crate::engine::anim::motion::ee_sub_22cad0;
use crate::engine::player::p5_states::FUN_0011bc00;
use crate::engine::player::p5_target::FUN_00117120;
use std::sync::atomic::{AtomicU32, Ordering};
use crate::engine::camera::math::{FUN_0011b628, FUN_0011bcf8, FUN_0011bff0, M4};

fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn scale(a: V3, s: f32) -> V3 {
    [a[0] * s, a[1] * s, a[2] * s]
}
/// row vector * rows: v.x r0 + v.y r1 + v.z r2
pub fn vmul(v: V3, m: &M3) -> V3 {
    add(add(scale(m[0], v[0]), scale(m[1], v[1])), scale(m[2], v[2]))
}
pub fn mmul(a: &M3, b: &M3) -> M3 {
    [vmul(a[0], b), vmul(a[1], b), vmul(a[2], b)]
}
pub fn transpose(m: &M3) -> M3 {
    [[m[0][0], m[1][0], m[2][0]], [m[0][1], m[1][1], m[2][1]], [m[0][2], m[1][2], m[2][2]]]
}
fn m4(m: &M3) -> M4 {
    [[m[0][0], m[0][1], m[0][2], 0.0], [m[1][0], m[1][1], m[1][2], 0.0], [m[2][0], m[2][1], m[2][2], 0.0], [0.0, 0.0, 0.0, 1.0]]
}
fn m3(m: &M4) -> M3 {
    [[m[0][0], m[0][1], m[0][2]], [m[1][0], m[1][1], m[1][2]], [m[2][0], m[2][1], m[2][2]]]
}

// ---------------------------------------------------------------- RNG
/// newlib `rand` (0x2342a0): state = state * 0x41c64e6d + 0x3039, returns state & 0x7fffffff (seed 1).
#[derive(Clone, Debug, PartialEq)]
pub struct GameRng(pub u32);
impl Default for GameRng {
    fn default() -> Self {
        Self(1)
    }
}
impl GameRng {
    pub fn FUN_002342a0(&mut self) -> i32 {
        self.0 = self.0.wrapping_mul(0x41c64e6d).wrapping_add(0x3039);
        (self.0 & 0x7fffffff) as i32
    }
    /// 0x11d3c0: rand() % 0x35f15e13
    pub fn FUN_0011d3c0(&mut self) -> i32 {
        self.FUN_002342a0() % 0x35f15e13
    }
    /// 0x11d3f8(lo, hi): lo + r % (hi - lo + 1)
    pub fn FUN_0011d3f8(&mut self, lo: i32, hi: i32) -> i32 {
        let r = self.FUN_0011d3c0();
        lo + r % (hi - (lo - 1))
    }
    /// 0x11d448(lo, hi): lo + (hi - lo) * (r % 0x16a1b) * 1.0787603e-05 (lo when equal, no draw)
    pub fn FUN_0011d448(&mut self, lo: f32, hi: f32) -> f32 {
        if lo == hi {
            return lo;
        }
        let r = self.FUN_0011d3c0();
        lo + (hi - lo) * (r % 0x16a1b) as f32 * 1.0787603e-05
    }
}

// ---------------------------------------------------------------- parent frames
/// 0x0022fcc8(from, to, v): v in `from`'s frame (rows +0x40..) to world, then into `to` (0x11b340 transpose).
pub fn FUN_0022fcc8(from: Option<&M3>, to: Option<&M3>, v: V3) -> V3 {
    let mut v = v;
    if let Some(f) = from {
        v = vmul(v, f);
    }
    if let Some(t) = to {
        v = vmul(v, &transpose(t));
    }
    v
}

/// Local (rot, pos) under a parent world (rot, pos): world = local * parent (row vectors).
pub fn compose(parent: (&M3, V3), local: (&M3, V3)) -> (M3, V3) {
    (mmul(local.0, parent.0), add(vmul(local.1, parent.0), parent.1))
}

// ---------------------------------------------------------------- attachments
/// One +0x138 attachment list entry {obj, ?, next}; slot 70 replaced by a rigid attachment.
#[derive(Clone, Debug, PartialEq)]
pub struct Attachment {
    pub child: usize,
    pub rot: M3,
    pub pos: V3,
}

/// 0x00229798: call slot 70 (+0x23c) of every attached object with the list node. Rigid replacement for the
/// slot: the child's world placement = its local placement under the parent's.
pub fn FUN_00229798(parent: (&M3, V3), list: &[Attachment], mut place: impl FnMut(usize, M3, V3)) {
    for a in list {
        let (r, p) = compose(parent, (&a.rot, a.pos));
        place(a.child, r, p);
    }
}

// ---------------------------------------------------------------- driven xfs
/// DAT_003e11e8: global xf generation counter (pre-incremented, stored to xf +0xa4).
pub static DAT_003E11E8: AtomicU32 = AtomicU32::new(0);
fn next_gen() -> u32 {
    DAT_003E11E8.fetch_add(1, Ordering::Relaxed).wrapping_add(1)
}

// 0x2c45f0 / 0x2c4600 / 0x2c4610 trapezoid profiles and clamp constants (exact ELF bits)
const C_ROT: [u32; 3] = [0x3c75c290, 0, 0x3f000000]; // 0.015, 0, 0.5
const C_ROT_ACC: u32 = 0x3d75c290; // 0x2c45fc 0.06
const C_LIN: [u32; 3] = [0x3fc00001, 0, 0x3f000000]; // 1.5000001, 0, 0.5
const C_SHEAR: [u32; 3] = [0x3ba3d70b, 0, 0x3f000000]; // 0.005, 0, 0.5
const C_LIN_ACC: u32 = 0x40c00001; // 0x2c461c 6.0000005
const C_SHEAR_ACC: u32 = 0x3ca3d70b; // 0x2c4620 0.02
fn fb(b: u32) -> f32 {
    f32::from_bits(b)
}

#[derive(Clone, Debug, PartialEq)]
pub struct DrivenXf {
    pub rot: M3,
    pub pos: V3,
    /// +0x94
    pub dof: i32,
    /// +0x98
    pub kind: u16,
    /// +0x9a
    pub sub: u16,
    /// +0xb0 (linear or angular velocity by dof)
    pub v: V3,
    /// +0xc0 / +0xd0 (sub 1: hermite p0 / v0)
    pub target: V3,
    pub target_v: V3,
    /// sub 5: +0xc0..+0xef
    pub target_m: M3,
    /// sub 1 hermite: +0xe0 p1, +0xf0 v1, +0x80 t0, +0x84 t1, *(+0x88) now, *(+0x8c) rate
    pub herm_p1: V3,
    pub herm_v1: V3,
    pub herm_t0: f32,
    pub herm_t1: f32,
    pub herm_now: f32,
    pub herm_k: f32,
    /// +0xa8 (kind 9)
    pub deadline: f32,
    /// +0xac
    pub speed: u16,
    /// object +0x40 & 0x10 (uStack_16c): frozen
    pub frozen: bool,
    /// +0xa4
    pub xf_gen: u32,
    /// set when 0x2303a0 was called this frame (kind re-evaluation; caller decides the next kind)
    pub reached: bool,
    /// the 0x2303a0 third argument of this frame (6 / 10 / 0xc; 0 = not called)
    pub reach_k: u32,
}

impl Default for DrivenXf {
    fn default() -> Self {
        Self {
            rot: IDENT, pos: [0.0; 3], dof: 1, kind: 7, sub: 3, v: [0.0; 3], target: [0.0; 3], target_v: [0.0; 3],
            target_m: IDENT, herm_p1: [0.0; 3], herm_v1: [0.0; 3], herm_t0: 0.0, herm_t1: 1.0, herm_now: 0.0,
            herm_k: 1.0, deadline: 0.0, speed: 0, frozen: false, xf_gen: 0, reached: false, reach_k: 0,
        }
    }
}

fn is_rot(dof: i32) -> bool {
    matches!(dof, 6 | 7)
}

impl DrivenXf {
    /// 0x0022ca18: the xf's current channel value by dof.
    pub fn FUN_0022ca18(&self) -> V3 {
        match self.dof {
            6 | 7 => FUN_0011bff0(&m4(&self.rot)),
            0xb => [self.rot[1][0], self.rot[2][0], self.rot[2][1]],
            0xc => [self.rot[0][0], self.rot[1][1], self.rot[2][2]],
            _ => self.pos,
        }
    }
    /// 0x0022c8c0: set the xf from a channel value by dof (1/4/5/9/10 translation with identity rows; 2/3
    /// identity; 6/7 euler 0x11b628; 0xb shear rows[1][0], rows[2][0], rows[2][1]; 0xc scale diagonal).
    pub fn FUN_0022c8c0(&mut self, v: V3) {
        match self.dof {
            1 | 4 | 5 | 9 | 10 => {
                self.rot = IDENT;
                self.pos = v;
            }
            6 | 7 => self.rot = m3(&FUN_0011b628(v)),
            0xb => {
                self.rot = IDENT;
                self.rot[1][0] = v[0];
                self.rot[2][0] = v[1];
                self.rot[2][1] = v[2];
            }
            0xc => {
                self.rot = IDENT;
                self.rot[0][0] = v[0];
                self.rot[1][1] = v[1];
                self.rot[2][2] = v[2];
            }
            _ => self.rot = IDENT,
        }
    }
    /// sub 1: 0x117120(+0x84 - t0, *(+0x88) - t0, *(+0x8c), +0xc0, +0xd0, +0xe0, +0xf0) -> (value, velocity)
    fn hermite(&self) -> (V3, V3) {
        let (p, v, _) = FUN_00117120(
            self.herm_t1 - self.herm_t0,
            self.herm_now - self.herm_t0,
            self.herm_k,
            self.target,
            self.target_v,
            self.herm_p1,
            self.herm_v1,
        );
        (p, v)
    }
    /// target value / velocity by sub-kind (0 current, 1 hermite, 3/4 +0xc0/+0xd0, 2 +0xc0 with no velocity,
    /// 5 euler of the +0xc0 matrix (0x11bff0), else 0).
    fn source(&self) -> (V3, V3) {
        match self.sub {
            0 => (self.FUN_0022ca18(), [0.0; 3]),
            1 => self.hermite(),
            3 | 4 => (self.target, self.target_v),
            2 => (self.target, [0.0; 3]),
            5 => (FUN_0011bff0(&m4(&self.target_m)), [0.0; 3]),
            _ => ([0.0; 3], [0.0; 3]),
        }
    }
}

/// 0x0022de88 for one xf of kind 4..0xb (kinds 1/2 are `driver::FUN_0022de88`). `now` = DAT_002dded0,
/// `dt` = DAT_002dded4, `mirror` = the kind-4 source xf (+0xa0 + 8) position and generation.
pub fn FUN_0022de88_driven(x: &mut DrivenXf, now: f32, dt: f32, mirror: Option<(V3, u32)>) {
    x.reached = false;
    x.reach_k = 0;
    match x.kind {
        4 => {
            if let Some((p, g)) = mirror {
                x.pos = scale(p, -1.0);
                x.xf_gen = g;
            }
        }
        5 | 6 => {
            if x.sub == 5 {
                x.rot = x.target_m;
            } else {
                let (t, _) = x.source();
                x.FUN_0022c8c0(t);
            }
            x.xf_gen = next_gen();
            // +0x98 read as int == 0x20006 (kind 6, sub 2) -> 0x2303a0(.., 10)
            if x.kind == 6 && x.sub == 2 {
                reach(x, 10);
            }
        }
        7 | 8 | 9 | 0xb => {
            if is_rot(x.dof) { angular(x, now, dt) } else { linear(x, now, dt) }
            x.xf_gen = next_gen();
        }
        _ => {}
    }
}

fn reach(x: &mut DrivenXf, k: u32) {
    x.reached = true;
    x.reach_k = k;
}

fn deadline_step(x: &DrivenXf, now: f32, dt: f32, l: f32, s: &mut f32, vel: &mut f32) {
    if x.kind == 9 {
        let mut smin = l;
        if now < x.deadline {
            smin = l * dt / ((x.deadline - now) + dt);
        }
        if *s < smin {
            *vel = smin / dt;
            *s = smin;
        }
    }
}

fn linear(x: &mut DrivenXf, now: f32, dt: f32) {
    let (t, tv) = x.source();
    let cur = x.FUN_0022ca18();
    let d = sub(t, cur);
    let l = dot(d, d).sqrt();
    let n = if 0.0001 < l { scale(d, 1.0 / l) } else { [0.0; 3] };
    let vt = dot(tv, n);
    let c = if (0xb..0xd).contains(&x.dof) { C_SHEAR } else { C_LIN };
    let sp = x.speed as f32;
    let p = [sp * fb(c[0]) + vt.abs(), sp * fb(c[1]) + vt.abs(), fb(c[2])];
    let mut s = l;
    let mut vel = 0.0;
    if !x.frozen {
        s = FUN_0011d018(0.0, l, dt, p, Some(&mut vel));
        deadline_step(x, now, dt, l, &mut s, &mut vel);
        if x.kind == 0xb {
            let lim = sp * fb(if x.dof > 10 && x.dof < 0xd { C_SHEAR_ACC } else { C_LIN_ACC });
            let vn = add(tv, scale(n, vel - vt));
            let dv = sub(vn, x.v);
            let a = dot(dv, dv).sqrt() / dt;
            if lim < a {
                let v1 = add(x.v, scale(dv, lim / a));
                let half = scale(add(x.v, v1), dt * 0.5);
                x.v = v1;
                x.FUN_0022c8c0(add(cur, half));
                return;
            }
        }
    }
    x.FUN_0022c8c0(add(cur, scale(n, s)));
    if l <= s {
        if x.kind != 8 {
            match x.sub {
                1 => reach(x, 6),
                2 => reach(x, 10),
                _ => {}
            }
        }
        x.v = tv;
    } else {
        x.v = add(tv, scale(n, vel - vt));
    }
}

fn angular(x: &mut DrivenXf, now: f32, dt: f32) {
    let old = x.rot;
    // rows = 0x22c8c0(target euler) (sub 5: the +0xc0 matrix, w = 0); target w = 0x22cad0(euler, rate)
    let (tgt, wt3) = if x.sub == 5 {
        (x.target_m, [0.0; 3])
    } else {
        let (t, tr) = x.source();
        (m3(&FUN_0011b628(t)), ee_sub_22cad0(t, tr))
    };
    // 0x11bc00: relative rotation old -> target; 0x11bcf8 -> (angle, axis)
    let rel = FUN_0011bc00(&old, &tgt);
    let (ang, axis) = FUN_0011bcf8(&m4(&rel));
    let wt = dot(wt3, axis);
    let sp = x.speed as f32;
    let k = if x.sub == 3 { 0.0 } else { wt.abs() };
    let p = [sp * fb(C_ROT[0]) + k, sp * fb(C_ROT[1]) + k, fb(C_ROT[2])];
    let mut s = ang;
    let mut vel = 0.0;
    if !x.frozen {
        s = FUN_0011d018(0.0, ang, dt, p, Some(&mut vel));
        deadline_step(x, now, dt, ang, &mut s, &mut vel);
        if x.kind == 0xb {
            let v1 = add(wt3, scale(axis, vel - wt));
            let dv = sub(v1, x.v);
            let a = dot(dv, dv).sqrt() / dt;
            let lim = sp * fb(C_ROT_ACC);
            if lim < a {
                let v1 = add(x.v, scale(dv, lim / a));
                let m = FUN_0011b570(scale(add(x.v, v1), dt * 0.5));
                x.rot = mmul(&old, &m);
                super::driver::FUN_0011b3d0(&mut x.rot);
                x.v = v1;
                return;
            }
        }
    }
    if ang <= s {
        if x.kind != 8 {
            if x.kind == 0xb {
                if x.sub == 2 {
                    reach(x, 0xc);
                }
            } else if x.sub == 1 {
                reach(x, 6);
            } else if x.sub == 2 {
                reach(x, 10);
            }
        }
        x.rot = tgt;
        x.v = wt3;
    } else {
        x.rot = mmul(&old, &FUN_0011b500(axis, s));
        super::driver::FUN_0011b3d0(&mut x.rot);
        x.v = add(wt3, scale(axis, vel - wt));
    }
}
