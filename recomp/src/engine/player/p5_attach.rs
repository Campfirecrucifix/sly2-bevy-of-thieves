//! WP-P5 attach (jts 7): rope / pole / point / rail riding. Specs: C:/Temp/p5/spec_192bd8.md,
//! spec_191a10.md, spec_19b110.md §2 (path layout).
//!
//! 1:1: FUN_00192bd8 (attach substate setter: exit of the old substate, attach entry when old == -1,
//! entry of the new substate incl. the ASEG / secondary / visibility choice, detach for new == -1),
//! FUN_001928f8 wired to it (`attach_to`; the p4.rs version only records calls), FUN_00191a10 kinds 0 / 0xd
//! (member geometry FUN_00182cd8 + the class member vfns 0x183040/0x1830d8 rail, 0x1834d0/0x183540 pole,
//! 0x182cc8/0x183818 point, 0x184208 / 0x1ff4f0), kind 3 (rope), kinds 1 / 2 / 4 / 8 (substate choice) / 0xb,
//! the tail; FUN_001911a8 (attach input, restart loop, every kind); FUN_001940b0 (camera sub-mode),
//! FUN_001944c8 (jts-7 base anim pick), FUN_0011f088 (approach velocity), FUN_0020bb58 / FUN_0020bad8
//! (2-anim blend -> jt+0x190), FUN_00204660 (facing yaw + rows); polyline path API FUN_001e6610 / 6620 /
//! 6630 / 6648 / 6660 / 66c8 / 7538 / 7668 / 63c0 / 64c8 / 67b8 / 6f10 / 6f60 / 6a30, FUN_001e6ed8 dispatch.
//! Edge forces FUN_00194170 (kind 3, target +0x1bc != 0) are in p4.rs (`Jt::FUN_00194170`); feed it
//! `rope_dir()` below.
//!
//! REPLACED: object pointers -> `AttachTarget` plain data (class, WorldPos, WorldRot, +0x10c, +0x1bc, path,
//! guide path +0x1c8, anchor +0x15c) supplied by the caller; JT fields the port's `Jt` lacks live in
//! `AttachExt` (jt+0x190 velocity goal, jt+0x1b0 facing rows, jt+0x240 grab offset, jt+0x103c hang radius,
//! jt+0x1040 side, (jt+0x1030)+0x40 anim rate, 0x204740 body yaw, jt+0x230, jt+0x1024 camera override as an
//! enum); animation data (blend entry velocities a+0xc0 / a+0x100, a+0x240 2-anim blend, kind-3 profile
//! a+0x260 / a+0x2a0) are plain fields the lead fills from the ASEGs (default 0 = not moving: no invented
//! values); ASEG pointers -> `AttachEv::Anim { off }` (offset of the ASEG field, lead maps it); the
//! FUN_001e7538 / 7668 cached-segment search (+0x178) -> stateless search (same result); FUN_0011d898 fmod
//! -> rem_euclid; sqrt.s of a negative (PS2 FPU returns sqrt|x|) -> sqrt(|x|); 0x1e67b8 kind-3 entry reads
//! the stale stack qword sp+0 (source bug) -> uses the tangent (only the initial +0x24, recomputed by
//! 0x191a10 from +0x28); 0x184208 / 0x1ff4f0 leave the yaw out-param stale -> current jt yaw is kept.
//! Engine calls that are side effects (ownership elsewhere) are returned as `AttachEv` (SetJtJts via
//! vt+0x2c4, FUN_00195a00 jumps, velocity / translate / jt+0x230 sets, target notify, rider list, collision
//! ignore, camera) or `AttachEv::Call(addr, arg)`.
//! UNPORTED (recorded as Call): spline paths (type 1: 0x1e6db8 golden search 0x11d9e0, Hermite 0x117120,
//! arc table 0x1e5fc8; type-1 paths are walked as polylines over their knots), xf argument of the path
//! queries (FUN_001fa970; every caller here passes 0), 0x1a0450 8-way blend (rewrites jt+0x190 in substate 1
//! and for kind 8), 0x20bc08 footstep fx, rope IK 0x189c40 / 0x193c98, 0x205910 look-around, kinds 5 / 9
//! sub-object 0x1ccac0 / 0x1cc2e8 / 0x1cd098, kind 10 root motion (0x12ff10 + anim vt+0x1c -> jt+0x190 = 0
//! here), kind 2 gadget 0x1441e8 / 0x1441a0 / 0x141ea8, kind 8 stand-up BSP test (input `stand_clear`),
//! 0x19cc50 attack, vt+0x318 (input `attack_ok`), 0x131968 anim-done (input `anim_done`), 0x183b80,
//! 0x192a28 guards forget, 0x14e328 / 0x14e4c8 fx, 0x194118 / 0x140c88 camera objects (the global
//! DAT_002bf670 write of class 0x26), 0x18c120 part visibility (`AttachEv::Vis`), 0x2046f0 / 0x204ca8.
//!
//! Per frame in jts 7 (original order): input 0x18a8c8 -> `FUN_001911a8`; update 0x18b7e8 -> `FUN_00191a10`;
//! then the contact motor 0x2044b0 turns `AttachExt::v190` into motion (`AttachExt::goal_world`) and slot 69
//! adds `Jt::FUN_00194170` for kind 3. On leaving jts 7 (SetJtJts with jts != 7) call `FUN_00192bd8(.., -1)`.
#![allow(non_snake_case, dead_code)]

use super::p4::FUN_00132b10;
use super::{FUN_0011e1a0, Jt};
use crate::engine::physics::driver::{FUN_0011b500, M3};
use crate::engine::physics::dynamics::FUN_0011d018;
use crate::engine::physics::geom::{dot, V3};
use std::f32::consts::PI;

/// pad bits in the joy +0x30 / +0x88 layout (H): 0x40 X jump, 0x20 O held (hang), 0x400 attack.
pub const JOY_X: u32 = 0x40;
pub const JOY_O: u32 = 0x20;
pub const JOY_ATTACK: u32 = 0x400;
/// DAT_002bf63c: kind-3 entry "facing backwards" dot threshold.
pub const DAT_002BF63C: f32 = -0.2;
/// DAT_002bf640: kind-10 anim choice stick magnitude.
pub const DAT_002BF640: f32 = 0.5;
/// DAT_002bf5bc: kinds 0/0xd lateral stick gain.
pub const DAT_002BF5BC: f32 = 1.5;
/// DAT_002bf5c0 / DAT_002bf5c4: kind 2 gadget (0x1441e8 arg) / blend scale.
pub const DAT_002BF5C0: f32 = 3.0;
pub const DAT_002BF5C4: f32 = 0.75;
/// DAT_002bf5c8: kind-3 speed profile acceleration.
pub const DAT_002BF5C8: f32 = 0.2;
/// DAT_002bf5cc: kind-8 turn divisor.
pub const DAT_002BF5CC: f32 = 55.0;
/// DAT_002bf5b0 / DAT_002bf5b4 / DAT_002bf5b8: kind-8 stand-up height, kind-9 reach, kind-10 min time.
pub const DAT_002BF5B0: f32 = 200.0;
pub const DAT_002BF5B4: f32 = 25.0;
pub const DAT_002BF5B8: f32 = 0.35;
/// literal 0x3e99999a: kinds 0/0xd approach acceleration.
pub const APPROACH_ACC: f32 = 0.3;
const UP: V3 = [0.0, 0.0, 1.0]; // DAT_002a2dc0
const X1: V3 = [1.0, 0.0, 0.0]; // DAT_002a2da0

fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn scale(a: V3, s: f32) -> V3 {
    [a[0] * s, a[1] * s, a[2] * s]
}
fn len(a: V3) -> f32 {
    dot(a, a).sqrt()
}
fn cross(a: V3, b: V3) -> V3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
/// vsqrt / vdiv normalize with the (1,0,0) fallback below 1e-4 (0x38d1b717).
fn norm_or_x(a: V3) -> V3 {
    let l = len(a);
    if l < 0.0001 {
        X1
    } else {
        scale(a, 1.0 / l)
    }
}
fn lerp(a: V3, b: V3, f: f32) -> V3 {
    add(scale(a, 1.0 - f), scale(b, f))
}
/// 0x0011cf60: wrap an angle into [-pi, pi] (fmod 0x11d898 -> rem_euclid).
pub fn FUN_0011cf60(a: f32) -> f32 {
    if !(-PI..=PI).contains(&a) {
        (a + PI).rem_euclid(2.0 * PI) - PI
    } else {
        a
    }
}

// ------------------------------------------------------------------------------------------------
// Path object (class 10 subtree): spec_19b110 §2
// ------------------------------------------------------------------------------------------------

/// Path object fields (+0xe0 .. +0xf4) and its world matrix (vt+0xfc).
#[derive(Clone, Debug, PartialEq)]
pub struct Path {
    /// +0xe0: 0 polyline, 1 spline (UNPORTED: walked as a polyline)
    pub kind: i32,
    /// +0xe4
    pub looped: bool,
    /// +0xec knot params s[n]
    pub s: Vec<f32>,
    /// +0xf0 cumulative arc length L[n]
    pub l: Vec<f32>,
    /// +0xf4 local knots
    pub pts: Vec<V3>,
    /// vt+0xfc world matrix: rows 0..2 rotation, translation
    pub rot: M3,
    pub pos: V3,
}

impl Path {
    /// Test / level-loader helper: polyline with s = L = cumulative length, identity matrix.
    pub fn polyline(pts: Vec<V3>, looped: bool) -> Self {
        let mut l = vec![0.0f32];
        for i in 1..pts.len() {
            let d = len(sub(pts[i], pts[i - 1]));
            l.push(l[i - 1] + d);
        }
        Self { kind: 0, looped, s: l.clone(), l, pts, rot: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]], pos: [0.0; 3] }
    }
    fn n(&self) -> usize {
        self.pts.len()
    }
    /// W * p (vmulax/vmadday/vmaddaz/vmaddw)
    fn wp(&self, p: V3) -> V3 {
        add(self.wv(p), self.pos)
    }
    /// W.rot * v
    fn wv(&self, v: V3) -> V3 {
        let r = &self.rot;
        add(add(scale(r[0], v[0]), scale(r[1], v[1])), scale(r[2], v[2]))
    }
}

/// 0x001e6610: s[0].
pub fn FUN_001e6610(p: &Path) -> f32 {
    p.s[0]
}
/// 0x001e6620: L[0].
pub fn FUN_001e6620(p: &Path) -> f32 {
    p.l[0]
}
/// 0x001e6630: s[n-1].
pub fn FUN_001e6630(p: &Path) -> f32 {
    p.s[p.n() - 1]
}
/// 0x001e6648: L[n-1].
pub fn FUN_001e6648(p: &Path) -> f32 {
    p.l[p.n() - 1]
}
fn wrap_range(looped: bool, v: f32, a: f32, b: f32) -> f32 {
    if looped && (v < a || b < v) {
        a + (v - a).rem_euclid(b - a)
    } else {
        v
    }
}
/// 0x001e6660: wrap s into [s0, s_last] when looped (fmod 0x11d898 -> rem_euclid).
pub fn FUN_001e6660(p: &Path, s: f32) -> f32 {
    wrap_range(p.looped, s, p.s[0], FUN_001e6630(p))
}
/// 0x001e66c8: same on [0, L[n-1]].
pub fn FUN_001e66c8(p: &Path, l: f32) -> f32 {
    wrap_range(p.looped, l, 0.0, FUN_001e6648(p))
}
fn seg_of(tab: &[f32], v: f32) -> (usize, f32, f32) {
    let n = tab.len();
    let mut i = 0;
    while i + 2 < n && tab[i + 1] <= v {
        i += 1;
    }
    (i, v - tab[i], tab[i + 1] - tab[i])
}
/// 0x001e7538(p, s, &ds, &dseg): segment i (0..n-2) with s[i] <= s < s[i+1] (REPLACED: the +0x178 cache
/// start -> plain search, same result).
pub fn FUN_001e7538(p: &Path, s: f32) -> (usize, f32, f32) {
    seg_of(&p.s, FUN_001e6660(p, s))
}
/// 0x001e7668: same on L[].
pub fn FUN_001e7668(p: &Path, l: f32) -> (usize, f32, f32) {
    seg_of(&p.l, FUN_001e66c8(p, l))
}
/// 0x001e63c0(p, s) -> arc length (type 0; spline arc table 0x1e5fc8 UNPORTED).
pub fn FUN_001e63c0(p: &Path, s: f32) -> f32 {
    let (i, ds, dseg) = FUN_001e7538(p, s);
    p.l[i] + (ds / dseg) * (p.l[i + 1] - p.l[i])
}
/// 0x001e64c8(p, L) -> s (type 0).
pub fn FUN_001e64c8(p: &Path, l: f32) -> f32 {
    let (i, dl, dseg) = FUN_001e7668(p, l);
    let f = dl / dseg;
    (1.0 - f) * p.s[i] + f * p.s[i + 1]
}
/// 0x001e67b8(p, s, &pos, &dir): world position and unit tangent at s (type 0 lerp).
pub fn FUN_001e67b8(p: &Path, s: f32) -> (V3, V3) {
    let (i, ds, dseg) = FUN_001e7538(p, s);
    let f = ds / dseg;
    let pos = p.wp(lerp(p.pts[i], p.pts[i + 1], f));
    (pos, norm_or_x(p.wv(sub(p.pts[i + 1], p.pts[i]))))
}
/// 0x001e6f10(p, L, &pos, &dir) = 0x1e67b8(p, 0x1e64c8(p, L)).
pub fn FUN_001e6f10(p: &Path, l: f32) -> (V3, V3) {
    FUN_001e67b8(p, FUN_001e64c8(p, l))
}

/// Closest-point result (cp, dir, s, L) of 0x1e6f60 / 0x1e6ed8.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PathHit {
    pub cp: V3,
    pub dir: V3,
    pub s: f32,
    pub l: f32,
}

/// 0x001e6f60(p, q, xf = 0, &cp, &dir, &s, &L): closest point over all segments (type 0; strict <, init
/// FLT_MAX DAT_002a8138).
pub fn FUN_001e6f60(p: &Path, q: V3) -> PathHit {
    let mut best = f32::MAX;
    let mut hit = PathHit { cp: q, dir: X1, s: p.s[0], l: 0.0 };
    for i in 1..p.n() {
        let a = p.wp(p.pts[i - 1]);
        let b = p.wp(p.pts[i]);
        let d = sub(b, a);
        let dd = dot(d, d);
        let f = if dd > 0.0 { (dot(sub(q, a), d) / dd).clamp(0.0, 1.0) } else { 0.0 };
        let c = add(a, scale(d, f));
        let e = len(sub(q, c));
        if e < best {
            best = e;
            hit = PathHit { cp: c, dir: norm_or_x(d), s: (1.0 - f) * p.s[i - 1] + f * p.s[i], l: 0.0 };
        }
    }
    hit.l = FUN_001e63c0(p, hit.s);
    hit
}

/// 0x001e6a30(p, u0, q, xf = 0, &cp, &tan, &u, &L): incremental polyline walk from the segment of u0.
/// Reversal stops take the far end of the new segment (source quirk, kept).
pub fn FUN_001e6a30(p: &Path, u0: f32, q: V3) -> PathHit {
    let n = p.n();
    let (mut i, _, _) = FUN_001e7538(p, u0);
    let (mut back, mut fwd) = (false, false); // bVar1 / bVar2
    let f;
    loop {
        let a = p.wp(p.pts[i]);
        let d = p.wv(sub(p.pts[i + 1], p.pts[i]));
        let l = len(d);
        let h = dot(scale(d, 1.0 / l), sub(q, a));
        if h < 0.0 {
            if fwd {
                f = 1.0;
                break;
            }
            if i < 1 {
                if !p.looped {
                    f = 0.0;
                    break;
                }
                i = n - 2;
            } else {
                i -= 1;
            }
            back = true;
        } else if l < h {
            if back {
                f = 0.0;
                break;
            }
            if i < n - 2 {
                i += 1;
            } else if p.looped {
                i = 0;
            } else {
                f = 1.0;
                break;
            }
            fwd = true;
        } else {
            f = h / l;
            break;
        }
    }
    let cp = p.wp(lerp(p.pts[i], p.pts[i + 1], f));
    let dir = norm_or_x(p.wv(sub(p.pts[i + 1], p.pts[i])));
    let s = (1.0 - f) * p.s[i] + f * p.s[i + 1];
    PathHit { cp, dir, s, l: FUN_001e63c0(p, s) }
}

/// 0x001e6ed8(u0, p, q, xf, ...): dispatch on +0xe0 (0 -> 0x1e6a30; 1 spline 0x1e6db8 UNPORTED -> 0x1e6a30).
pub fn FUN_001e6ed8(u0: f32, p: &Path, q: V3) -> PathHit {
    FUN_001e6a30(p, u0, q)
}

// ------------------------------------------------------------------------------------------------
// State
// ------------------------------------------------------------------------------------------------

/// Attach target (a->+0x14) as plain data.
#[derive(Clone, Debug, PartialEq)]
pub struct AttachTarget {
    pub class: i32,
    /// FUN_001a62e8 / FUN_001a6318
    pub pos: V3,
    pub rot: M3,
    /// +0x10c as int (class 0x26 attach kind) and as float (class 0x22 pole radius, class 0x27 / 0x22 param)
    pub f_10c: i32,
    pub r_10c: f32,
    /// +0x1bc: kind 3 edge-rope flag (forces only) / class 0xc direction bits / alt idle gate
    pub f_1bc: u32,
    /// +0x1c0 / +0x1c4: edge force params (p4 FUN_00194170)
    pub f_1c0: f32,
    pub f_1c4: f32,
    /// path data when IsA(class, 10) (rope / rail): +0xe0.. (+0xe4 looped)
    pub path: Option<Path>,
    /// +0x1c8 guide path
    pub guide: Option<Path>,
    /// +0x15c hiding-spot anchor (WorldPos, WorldRot) (kind 8)
    pub anchor: Option<(V3, M3)>,
    /// +0x190 (class 0xc) / +0xe0 (0x22, 0x26) own camera present
    pub own_cam: bool,
    /// +0x18c (kind 1)
    pub f_18c: bool,
}

impl AttachTarget {
    pub fn new(class: i32, pos: V3) -> Self {
        Self {
            class,
            pos,
            rot: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            f_10c: 0,
            r_10c: 0.0,
            f_1bc: 0,
            f_1c0: 0.0,
            f_1c4: 0.0,
            path: None,
            guide: None,
            anchor: None,
            own_cam: false,
            f_18c: false,
        }
    }
}

/// jt+0x1024 camera override relative to the attach.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CamOv {
    #[default]
    None,
    /// == a+0x68 (attach camera from 0x194118)
    Attach,
    /// == the target's own camera (*(a+0x18) member / +0x190 / +0xe0)
    Target,
    /// someone else's override
    Other,
}

/// 2-anim blend (0x20bb58 format; a+0x180 kind 1, a+0x240 kind 3, jt+0x1c30 kind 2).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Blend2 {
    /// +0x54 entry 1 enabled
    pub on: bool,
    /// +0x10 / +0x50 weights (written by 0x20bb58)
    pub w: [f32; 2],
    /// +0x34 / +0x74 over the ASEG duration (0x20bad8 factor)
    pub k: [f32; 2],
    /// +0x20 / +0x60 entry velocities (facing-local)
    pub vel: [V3; 2],
    /// ASEG present (a+0x184 for kind 1)
    pub aseg: bool,
}

/// ATTACH fields beyond p4::Attach plus the JT fields this code needs (see module doc).
#[derive(Clone, Debug, PartialEq)]
pub struct AttachExt {
    /// a+0x10 time the substate was entered
    pub t_sub: f32,
    /// a+0x1c u, a+0x20 second param (rail L / pole s / rope L)
    pub u: f32,
    pub s20: f32,
    /// a+0x24 facing backwards (kind 3), a+0x28 direction latch
    pub f_24: bool,
    pub f_28: i32,
    /// a+0x2c / +0x30 / +0x34 camera sub-mode (0x1940b0), a+0x50 weight latch
    pub cam_mode: i32,
    pub t_30: f32,
    pub f_34: i32,
    pub f_50: f32,
    /// a+0x70 / a+0x90 8-way blend weights w0 / w1 (written by 0x1a0450, UNPORTED)
    pub b8_w: [f32; 2],
    /// a+0xc0 (side 1) / a+0x100 (side 0) 8-way blend entry velocities
    pub b8_vel_c0: V3,
    pub b8_vel_100: V3,
    /// a+0x240 kind-3 blend, a+0x180 kind-1 blend, jt+0x1c30 kind-2 blend
    pub rope_blend: Blend2,
    pub hide_blend: Blend2,
    pub gadget_blend: Blend2,
    /// a+0x260 v0 / a+0x2a0 vmax kind-3 speed profile
    pub rope_v0: f32,
    pub rope_vmax: f32,
    /// a+0x314 kind-3 alt idle ASEG present; a+0x318 / a+0x31c
    pub alt_idle: bool,
    pub f_318: f32,
    pub f_31c: f32,
    /// a+0x4fc kind-0xb param
    pub f_4fc: i32,
    // ---- JT fields ----
    /// jt+0x190 velocity goal (negated, facing-local)
    pub v190: V3,
    /// jt+0x1b0 / 0x1c0 / 0x1d0 facing rows (0x204660)
    pub rows: M3,
    /// jt+0x240 grab offset, jt+0x103c hang radius, jt+0x1040 side flag
    pub grab: V3,
    pub hang_r: f32,
    pub side: i32,
    /// (jt+0x1030)+0x40 anim playback rate
    pub anim_rate: f32,
    /// 0x204740 actual body yaw (from the physics xf; lead keeps it current)
    pub body_yaw: f32,
    /// jt+0x230 (kind 3 / 4 entry)
    pub p230: V3,
    /// jt+0x1024
    pub cam: CamOv,
}

impl Default for AttachExt {
    fn default() -> Self {
        Self {
            t_sub: 0.0,
            u: 0.0,
            s20: 0.0,
            f_24: false,
            f_28: -1,
            cam_mode: -1,
            t_30: 0.0,
            f_34: 0,
            f_50: 0.0,
            b8_w: [0.0; 2],
            b8_vel_c0: [0.0; 3],
            b8_vel_100: [0.0; 3],
            rope_blend: Blend2::default(),
            hide_blend: Blend2::default(),
            gadget_blend: Blend2::default(),
            rope_v0: 0.0,
            rope_vmax: 0.0,
            alt_idle: false,
            f_318: 0.0,
            f_31c: 0.0,
            f_4fc: -1,
            v190: [0.0; 3],
            rows: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            grab: [0.0; 3],
            hang_r: 0.0,
            side: 0,
            anim_rate: 1.0,
            body_yaw: 0.0,
            p230: [0.0; 3],
            cam: CamOv::None,
        }
    }
}

impl AttachExt {
    /// World velocity the contact motor 0x2044b0 drives towards: Vw = rows * v190, motion = -Vw.
    pub fn goal_world(&self) -> V3 {
        let r = &self.rows;
        let v = self.v190;
        scale(add(add(scale(r[0], v[0]), scale(r[1], v[1])), scale(r[2], v[2])), -1.0)
    }
}

/// Requests for the lead (engine side effects of the attach code).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AttachEv {
    /// vt+0x2c4(jt, m) = 0x18d568: SetJtJts(jt, table[m], -1); m 2 = jts 7 (attach), 0 = jts 0 (drop)
    SetJtJts { m: i32 },
    /// FUN_00195a00(jt+0x2c0, kind, idx) (kind 1 jump off, 0 fall off the rope end, 7 out of a hide spot)
    Jump { kind: i32, idx: i32 },
    /// vt+0x11c (0x1f4488) set linear / vt+0x124 (0x1f46d8) angular velocity
    SetVelocity(V3),
    SetAngVelocity(V3),
    /// vt+0xec (0x143cf8)(jt, v): translate the JT by v (0x2041c0)
    Translate(V3),
    /// FUN_001305b8(f0, f1, ASEG at attach offset `off` (jt offset if `jt_rel`), jt, flags, jt+0x1030)
    Anim { off: u32, jt_rel: bool, flags: u32, f0: f32, f1: f32 },
    /// FUN_001f1448(secondary at attach offset `off`, jt, -1, flags, jt+0x1034)
    Snd { off: u32, flags: u32 },
    /// FUN_0018c120(jt, mask) part visibility (class 0x35 JTs)
    Vis(u32),
    /// target vt+0x1c(target, msg, 0, 0, 0, 0): 5 grab, 6 release
    Notify(i32),
    /// FUN_001333c0 / FUN_00133448 rider list add / remove
    Rider(bool),
    /// FUN_00204f78(jt, target, on): collision-pair ignore
    IgnoreCollision(bool),
    /// unported callee reached (address, arg)
    Call(u32, i32),
}

fn isa(tg: Option<&AttachTarget>, id: i32) -> bool {
    tg.is_some_and(|t| FUN_00132b10(t.class, id))
}

/// 0x00204660(yaw, jt): jt+0x1a8 = yaw; jt+0x1b0.. = FUN_0011b500(yaw, up).
pub fn FUN_00204660(jt: &mut Jt, x: &mut AttachExt, yaw: f32) {
    jt.yaw = yaw;
    x.rows = FUN_0011b500(UP, yaw);
}

/// 0x0011f088(dt, x, target, prm, &out): approach velocity out (1:1); returns the new position.
pub fn FUN_0011f088(dt: f32, x: V3, target: V3, prm: [f32; 3], out: &mut V3) -> V3 {
    let e = sub(target, x);
    let l = len(e);
    if l <= 0.0001 {
        *out = [0.0; 3];
        return x;
    }
    let mut vel = 0.0;
    let step = FUN_0011d018(0.0, l, dt, prm, Some(&mut vel));
    *out = scale(e, vel / l);
    add(x, scale(e, step / l))
}

/// 0x0020bb58(w, B) + 0x20bad8: set the blend weights and write jt+0x190 = -(sum w k vel) * anim rate.
pub fn FUN_0020bb58(x: &mut AttachExt, which: u8, w: f32) {
    let rate = x.anim_rate;
    let b = match which {
        1 => &mut x.hide_blend,
        2 => &mut x.gadget_blend,
        _ => &mut x.rope_blend,
    };
    b.w[1] = if b.on { w } else { 0.0 };
    b.w[0] = if b.on { 1.0 - w } else { 1.0 };
    let v = add(scale(b.vel[0], b.w[0] * b.k[0]), scale(b.vel[1], b.w[1] * b.k[1]));
    x.v190 = scale(v, -rate);
}

/// 0x001940b0(a, m): camera sub-mode bookkeeping (1:1, no motion effect).
pub fn FUN_001940b0(jt: &Jt, x: &mut AttachExt, m: i32) {
    if m == x.cam_mode {
        return;
    }
    x.cam_mode = m;
    x.f_50 = 0.0;
    x.t_30 = jt.t;
    if m == 1 || m == 2 {
        x.f_34 = if x.cam == CamOv::Attach { 2 } else { 0 };
    }
    if m == 3 {
        x.f_34 = 1;
    }
}

/// 0x001944c8(a, &flag): jts-7 base ASEG (attach offset) and flag (1:1).
pub fn FUN_001944c8(jt: &Jt, _x: &AttachExt) -> (Option<u32>, i32) {
    let a = &jt.attach;
    match a.kind {
        0 => (Some(if a.sub == 5 { 0x60 } else { 0x54 }), 0),
        1 => (Some(0x58), 0),
        8 if (9..=10).contains(&a.sub) => (Some(0x64), 1),
        _ => (None, 0),
    }
}

/// 0x00182cd8(member, jt, &s, &pt, &yaw): member vt+0x14 (param) then vt+0x1c (desired JT origin, yaw) by
/// target class (1:1 of 0x183040/0x1830d8, 0x1834d0/0x183540, 0x182cc8/0x183818, 0x184208, 0x1ff4f0).
/// `None` yaw = not written (stale in the original).
pub fn FUN_00182cd8(jt: &Jt, x: &AttachExt, tg: &AttachTarget) -> (f32, V3, Option<f32>) {
    let q = jt.phys.pos; // *(jt+0x218)
    let flip = |y: f32| if x.side == 1 { y + PI } else { y };
    let c = tg.class;
    if FUN_00132b10(c, 0x26) {
        // 0x182cc8 / 0x183818
        let r = if tg.f_10c == 0 { x.hang_r } else { 0.0 };
        let row0 = tg.rot[0];
        let pt = add(sub(tg.pos, x.grab), scale(row0, r));
        (0.0, pt, Some(flip(row0[1].atan2(row0[0]))))
    } else if FUN_00132b10(c, 0x22) {
        // 0x1834d0 / 0x183540
        let r = tg.r_10c;
        let d = sub(q, tg.pos);
        let s = d[1].atan2(d[0]) * r;
        let th = s / r;
        let pt = sub(add(tg.pos, FUN_0011e1a0(th, r + x.hang_r, 0.0)), x.grab);
        (s, pt, Some(flip(th)))
    } else if FUN_00132b10(c, 0xc) {
        // 0x183040 / 0x1830d8
        let p = tg.path.as_ref().expect("class 0xc rail without path");
        let mut s = FUN_001e6f60(p, q).l;
        if !p.looped {
            s = s.clamp(30.0, FUN_001e6648(p) - 30.0);
        }
        let (pp, tan) = FUN_001e6f10(p, s);
        let mut n = norm_or_x(cross(tan, UP));
        let pt = sub(add(pp, scale(n, x.hang_r)), x.grab);
        if x.side == 1 {
            n = scale(n, -1.0);
        }
        (s, pt, Some(n[1].atan2(n[0])))
    } else if (0x23..=0x25).any(|k| FUN_00132b10(c, k)) {
        (0.0, tg.pos, None) // 0x184208: no -jt+0x240
    } else {
        (0.0, sub(tg.pos, x.grab), None) // 0x1ff4f0 (class 0x27)
    }
}

/// Edge direction for p4 `Jt::FUN_00194170` (FUN_001e67b8 tangent at a->u).
pub fn rope_dir(x: &AttachExt, tg: &AttachTarget) -> V3 {
    tg.path.as_ref().map_or(X1, |p| FUN_001e67b8(p, x.u).1)
}

// ------------------------------------------------------------------------------------------------
// 0x1928f8 / 0x192bd8
// ------------------------------------------------------------------------------------------------

/// 0x001928f8(a, kind, target, member) with the real 0x192bd8 (p4's version only records calls).
pub fn attach_to(
    jt: &mut Jt,
    x: &mut AttachExt,
    tg: Option<&AttachTarget>,
    kind: i32,
    target: Option<usize>,
    member: usize,
    out: &mut Vec<AttachEv>,
) {
    FUN_00192bd8(jt, x, tg, -1, out);
    jt.attach.kind = kind;
    jt.attach.member = member;
    jt.attach.t = jt.t;
    jt.attach.target = target;
    out.push(AttachEv::SetJtJts { m: 2 });
    let s = match kind {
        0 | 0xd => 5,
        1 => {
            out.push(AttachEv::Call(0x00192a28, 1));
            0
        }
        8 => {
            out.push(AttachEv::Call(0x00192a28, 8));
            7
        }
        5 | 9 => {
            jt.attach.f_131 = target;
            out.push(AttachEv::Call(0x001ccac0, 0));
            0
        }
        6 => {
            out.push(AttachEv::Call(0x0018e540, 0));
            out.push(AttachEv::Call(0x0018ed70, 0));
            0
        }
        0xb | 0xc => {
            jt.attach.f_140 = 1;
            0
        }
        _ => 0,
    };
    FUN_00192bd8(jt, x, tg, s, out);
}

/// 0x00192bd8(a, new): attach substate setter (1:1 control flow; engine effects as events).
pub fn FUN_00192bd8(jt: &mut Jt, x: &mut AttachExt, tg: Option<&AttachTarget>, new: i32, out: &mut Vec<AttachEv>) {
    let old = jt.attach.sub;
    if new == old {
        return;
    }
    let kind = jt.attach.kind;
    // ---- A. exit old ----
    match old {
        1 => {} // jt+0x1038 = -1
        7 => {
            if new == 9 || new == 10 {
                if let Some((p, m)) = tg.and_then(|t| t.anchor) {
                    out.push(AttachEv::Translate(sub(p, x.grab)));
                    // jt+0x1060 = p + up * jt+0x1048 (not modelled)
                    FUN_00204660(jt, x, m[0][1].atan2(m[0][0])); // 0x2046b0 + vt+0xf4
                    out.push(AttachEv::SetVelocity([0.0; 3]));
                    out.push(AttachEv::SetAngVelocity([0.0; 3]));
                }
                out.push(AttachEv::Call(0x0014e328, 10));
            }
        }
        9 | 10 => {
            if !(9..=10).contains(&new) {
                out.push(AttachEv::Call(0x0014e4c8, 10));
            }
        }
        -1 => {
            // ---- B. attach entry ----
            if let Some(p) = tg.filter(|t| FUN_00132b10(t.class, 10)).and_then(|t| t.path.as_ref()) {
                let h = FUN_001e6f60(p, jt.phys.pos);
                x.u = h.s;
                x.s20 = h.l;
            }
            match kind {
                1 => {
                    out.push(AttachEv::Call(0x00183b80, 2));
                }
                2 => out.push(AttachEv::Call(0x001fbb90, 0x5c)),
                3 => {
                    out.push(AttachEv::Notify(5));
                    out.push(AttachEv::Rider(true));
                    out.push(AttachEv::IgnoreCollision(true));
                    x.f_28 = -1;
                    if let Some(p) = tg.and_then(|t| t.path.as_ref()) {
                        let (pos, tan) = FUN_001e67b8(p, x.u);
                        x.p230 = pos;
                        // REPLACED: the original normalizes the stale sp+0 qword, not `tan`
                        let dn = norm_or_x([tan[0], tan[1], 0.0]);
                        x.f_24 = dot(dn, jt.phys.rot[0]) < DAT_002BF63C;
                    }
                }
                4 => {
                    out.push(AttachEv::Notify(5));
                    out.push(AttachEv::Rider(true));
                    out.push(AttachEv::IgnoreCollision(true));
                    if let Some(t) = tg {
                        x.p230 = t.pos;
                    }
                }
                0xb => {
                    if tg.is_some() {
                        out.push(AttachEv::Rider(true));
                    }
                }
                0xc => {
                    if tg.is_some() {
                        out.push(AttachEv::Call(0x002046f0, 0));
                    }
                    out.push(AttachEv::Call(0x00204ca8, 0));
                }
                _ => {}
            }
            x.cam_mode = -1;
            jt.t_ground = jt.t; // jt+0x20c
        }
        _ => {}
    }
    // ---- C. enter new ----
    let mut anim: Option<(u32, bool)> = None;
    let mut snd: Option<u32> = None;
    let mut flags = 1u32;
    let (mut f0, mut f1) = (0.0f32, 1.0f32);
    let mut vis = 0u32;
    let mut zero = false;
    match new {
        0 | 5 => {
            zero = true;
            match kind {
                0 => (anim, flags) = (Some((0x174, false)), 9),
                1 => snd = Some(0x230),
                2 => (anim, flags) = (Some((0x514, false)), 9),
                3 => {
                    if tg.map_or(0, |t| t.f_1bc) == 0 || !x.alt_idle {
                        anim = Some((0x238, false));
                    } else {
                        anim = Some((0x314, false));
                        x.f_318 = 0.0;
                        x.f_31c = 0.0;
                    }
                    flags = 9;
                    vis = 0x1f;
                }
                4 => (anim, flags, vis) = (Some((0x234, false)), 9, 7),
                5 | 9 => {
                    f1 = 0.0;
                    jt.attach.f_131 = jt.attach.target;
                    out.push(AttachEv::Call(0x001ccac0, 0));
                    anim = Some((0x470, false));
                    f0 = f32::NAN; // anim->+0x74 * 0.5 (ASEG length, lead resolves)
                }
                6 | 7 => (anim, flags) = (Some((0x320, false)), 9),
                8 => {}
                10 => anim = Some((if DAT_002BF640 <= jt.joy.mag { 0x4f0 } else { 0x4f4 }, false)),
                0xb => {
                    anim = Some((0x4f8, false));
                    if x.f_4fc == 0x15 {
                        out.push(AttachEv::Call(0x002f7900, 0x1af0)); // *(DAT_002f7900+0x1af0) or 0x4f8
                    }
                }
                0xc => anim = Some((0x50c, false)),
                0xd => (anim, flags) = (Some((if new == 0 { 0x510 } else { 0x174 }, false)), 9),
                _ => {}
            }
        }
        1 => {
            flags = 9;
            if kind == 0xd || kind == 0 {
                anim = Some((0x170, false));
            }
            if kind == 0 && x.cam == CamOv::None {
                if let Some(t) = tg.filter(|t| [0xc, 0x22, 0x26].iter().any(|&k| FUN_00132b10(t.class, k))) {
                    if t.own_cam {
                        x.cam = CamOv::Target;
                        out.push(AttachEv::Call(0x00140c88, 1));
                    } else {
                        x.cam = CamOv::Attach;
                        out.push(AttachEv::Call(0x00194118, t.class));
                        out.push(AttachEv::Call(0x00140c88, 2));
                    }
                }
            }
        }
        2 => (zero, anim) = (true, Some((0x178, false))),
        3 => (zero, anim) = (true, Some((0x17c, false))),
        4 => {
            flags = 9;
            match kind {
                1 => {
                    anim = Some((0x184, false));
                    out.push(AttachEv::Call(0x0020bd68, 0x180));
                }
                2 => {
                    anim = Some((0x1c34, true));
                    out.push(AttachEv::Call(0x0020bd68, 0x1c30));
                }
                3 => {
                    anim = Some((0x244, false));
                    out.push(AttachEv::Call(0x0020bd68, 0x240));
                }
                _ => {}
            }
        }
        7 => {
            anim = Some((0x324, false));
            if let Some((_, m)) = tg.and_then(|t| t.anchor) {
                FUN_00204660(jt, x, m[0][1].atan2(m[0][0]));
            }
            x.v190 = [0.0; 3];
            out.push(AttachEv::Notify(5));
        }
        8 => {
            anim = Some((0x328, false));
            x.v190 = [0.0; 3];
            out.push(AttachEv::Notify(6));
        }
        9 => {
            snd = Some(0x32c);
            zero = true;
            if old == 7 {
                out.push(AttachEv::Call(0x00192bd8, 0x10)); // SetFlag10(jt, cam objects)
            }
        }
        10 => (anim, flags, zero) = (Some((0x430, false)), 9, true),
        -1 => {
            // ---- D. detach ----
            jt.attach.f_140 = 0;
            match kind {
                1 => out.push(AttachEv::Call(0x00183b80, 0)),
                2 => out.push(AttachEv::Call(0x001fb098, 0)),
                3 | 4 => {
                    out.push(AttachEv::Notify(6));
                    out.push(AttachEv::Rider(false));
                    out.push(AttachEv::IgnoreCollision(false));
                }
                5 | 9 => out.push(AttachEv::Call(0x001ccac0, -1)),
                6 => out.push(AttachEv::Call(0x0018e540, -1)),
                0xb => {
                    if tg.is_some() {
                        out.push(AttachEv::Rider(false));
                    }
                }
                0xc => out.push(AttachEv::Call(0x001cd2c0, 0)),
                _ => {}
            }
            if matches!(x.cam, CamOv::Attach | CamOv::Target) {
                x.cam = CamOv::None;
                out.push(AttachEv::Call(0x00140c88, 0));
            }
            x.u = 0.0;
            x.f_4fc = -1;
            jt.attach.kind = -1;
            jt.attach.target = None;
            jt.attach.member = 0;
            x.s20 = 0.0;
            x.f_34 = 0;
        }
        _ => {} // 6
    }
    if zero {
        x.v190 = [0.0; 3];
    }
    // ---- tail ----
    jt.attach.sub = new;
    x.t_sub = jt.t;
    if new != -1 {
        if let Some((off, jt_rel)) = anim {
            out.push(AttachEv::Anim { off, jt_rel, flags, f0, f1 });
        }
        if let Some(off) = snd {
            out.push(AttachEv::Snd { off, flags });
        }
        // vt+0x2d4 = 0x18c060 -> FUN_001944c8 (lead calls it), then 0x18c120 for class 0x35
        out.push(AttachEv::Vis(vis));
    }
}

// ------------------------------------------------------------------------------------------------
// 0x191a10 per-frame update
// ------------------------------------------------------------------------------------------------

/// 0x00191a10(a): attach update, every frame in jts 7.
pub fn FUN_00191a10(jt: &mut Jt, x: &mut AttachExt, tg: Option<&AttachTarget>, dt: f32, out: &mut Vec<AttachEv>) {
    let p = jt.phys.pos;
    let (ang, mag) = (jt.joy.angle, jt.joy.mag);
    let kind = jt.attach.kind;
    'sw: {
        match kind {
            0 | 0xd => {
                let Some(t) = tg else { break 'sw };
                let (s, mut g, yaw) = FUN_00182cd8(jt, x, t);
                x.s20 = s;
                let yaw = yaw.unwrap_or(jt.yaw);
                let ev = if x.side == 1 { x.b8_vel_c0 } else { x.b8_vel_100 };
                let spd = len(ev) * x.anim_rate;
                g[2] = p[2];
                let prm = [spd, 0.0, APPROACH_ACC];
                let mut v = [0.0; 3];
                FUN_0011f088(dt, p, g, prm, &mut v);
                let (l, lat) = if FUN_00132b10(t.class, 0x26) {
                    ([0.0; 3], 0.0)
                } else {
                    let tt = FUN_0011e1a0(yaw + if x.side == 1 { -PI / 2.0 } else { PI / 2.0 }, 1.0, 0.0);
                    let sv = FUN_0011e1a0(ang, 1.0, 0.0);
                    let mut k = (DAT_002BF5BC * dot(tt, sv)).clamp(-1.0, 1.0) * mag;
                    if k.abs() < 0.2 {
                        k = 0.0;
                    }
                    // sqrt.s of a negative: PS2 FPU gives sqrt(|x|)
                    let lat = (spd * spd - dot(v, v)).abs().sqrt().min(k * spd);
                    let mut cap = lat;
                    if let Some(pa) = t.path.as_ref().filter(|pa| FUN_00132b10(t.class, 0xc) && !pa.looped) {
                        let (mut vlo, mut vhi) = (0.0, 0.0);
                        FUN_0011d018(x.s20, FUN_001e6620(pa) + 30.0, dt, prm, Some(&mut vlo));
                        FUN_0011d018(x.s20, FUN_001e6648(pa) - 30.0, dt, prm, Some(&mut vhi));
                        cap = if cap < vlo { vlo } else if vhi < cap { vhi } else { cap };
                    }
                    (scale(tt, cap), lat)
                };
                let w = add(v, l);
                let r = &x.rows;
                x.v190 = [-dot(r[0], w), -dot(r[1], w), -dot(r[2], w)];
                let mut sub_ = if len(x.v190) > 25.0 { 1 } else { 0 };
                if sub_ == 1 {
                    if kind == 0 {
                        if len(v) > 45.0 {
                            FUN_001940b0(jt, x, 0);
                        } else if x.v190[1] > 20.0 {
                            FUN_001940b0(jt, x, if x.side == 1 { 1 } else { 2 });
                        } else if x.v190[1] < -20.0 {
                            FUN_001940b0(jt, x, if x.side == 1 { 2 } else { 1 });
                        }
                    }
                } else {
                    if FUN_00132b10(t.class, 0xc) {
                        if lat < -25.0 && t.f_1bc & 1 != 0 {
                            sub_ = if kind == 0 && x.side == 0 { 3 } else { 2 };
                        } else if lat > 25.0 && t.f_1bc & 2 != 0 {
                            sub_ = if kind == 0 && x.side == 0 { 2 } else { 3 };
                        }
                    }
                    if kind == 0 {
                        FUN_001940b0(jt, x, 3);
                    }
                }
                FUN_00204660(jt, x, yaw);
                if jt.attach.sub == 5 && FUN_0011cf60(x.body_yaw - yaw).abs() > 0.25 {
                    sub_ = 5;
                }
                FUN_00192bd8(jt, x, tg, sub_, out);
            }
            3 => {
                let Some(t) = tg else { break 'sw };
                let Some(pa) = t.path.as_ref() else { break 'sw };
                let h = FUN_001e6ed8(x.u, pa, p);
                x.u = h.s;
                x.s20 = h.l;
                if t.f_1bc != 0
                    && !pa.looped
                    && (x.u < FUN_001e6610(pa) + 0.01 || FUN_001e6630(pa) - 0.01 < x.u)
                {
                    out.push(AttachEv::Jump { kind: 0, idx: 0 });
                    break 'sw;
                }
                let d = norm_or_x([h.dir[0], h.dir[1], 0.0]);
                let mut yaw = d[1].atan2(d[0]);
                let sv = FUN_0011e1a0(ang, 1.0, 0.0);
                let c = match t.guide.as_ref() {
                    Some(gp) => FUN_001e6f60(gp, p).dir,
                    None => d,
                };
                let c = norm_or_x([c[0], c[1], 0.0]);
                let cd = dot(sv, c);
                x.f_28 = match x.f_28 {
                    0 if mag < 0.25 => -1,
                    0 if cd < -0.7 => 1,
                    1 if mag < 0.25 => -1,
                    1 if cd > 0.7 => 0,
                    -1 if mag > 0.35 => {
                        if cd >= 0.0 {
                            0
                        } else {
                            1
                        }
                    }
                    l => l,
                };
                if x.f_28 == 0 {
                    x.f_24 = false;
                }
                if x.f_28 == 1 {
                    x.f_24 = true;
                }
                let w = if t.f_1bc == 0 {
                    let prm = [x.rope_vmax, x.rope_v0, DAT_002BF5C8];
                    let mut r = 0.0;
                    if pa.looped {
                        r = x.rope_vmax;
                    } else if x.f_24 {
                        FUN_0011d018(x.s20, 50.0, dt, prm, Some(&mut r));
                        r = -r;
                    } else {
                        FUN_0011d018(x.s20, FUN_001e6648(pa) - 50.0, dt, prm, Some(&mut r));
                    }
                    let f = (r - x.rope_v0) / (x.rope_vmax - x.rope_v0);
                    if mag < 0.0 {
                        0.0
                    } else {
                        f.min(mag)
                    }
                } else {
                    0.0
                };
                if x.f_24 {
                    yaw = FUN_0011cf60(yaw + PI);
                }
                if w > 0.0 {
                    FUN_00192bd8(jt, x, tg, 4, out);
                    FUN_0020bb58(x, 3, w);
                    FUN_00204660(jt, x, yaw);
                    out.push(AttachEv::Call(0x0020bc08, 0x240));
                } else {
                    FUN_00192bd8(jt, x, tg, 0, out);
                    FUN_00204660(jt, x, yaw);
                }
            }
            1 => {
                if tg.is_some_and(|t| t.f_18c) && mag > 0.0 && x.hide_blend.aseg {
                    FUN_00192bd8(jt, x, tg, 4, out);
                    FUN_0020bb58(x, 1, mag);
                    FUN_00204660(jt, x, ang);
                    out.push(AttachEv::Call(0x0020bc08, 0x180));
                } else {
                    FUN_00192bd8(jt, x, tg, 0, out);
                }
            }
            2 => {
                out.push(AttachEv::Call(0x001441e8, 3));
                let go = if jt.attach.sub == 0 { mag > 0.1 } else { mag > 0.0 };
                if go {
                    FUN_00192bd8(jt, x, tg, 4, out);
                    FUN_0020bb58(x, 2, mag * DAT_002BF5C4);
                    FUN_00204660(jt, x, ang);
                } else {
                    FUN_00192bd8(jt, x, tg, 0, out);
                }
            }
            4 => FUN_00204660(jt, x, ang),
            5 | 9 => out.push(AttachEv::Call(0x001cc2e8, 0)),
            8 => {
                if !(7..=8).contains(&jt.attach.sub) {
                    if mag > 0.0 {
                        FUN_00192bd8(jt, x, tg, 10, out);
                        // jt+0x190 from the a+0x330 8-way blend (0x1a0450 UNPORTED) and the yaw turn
                        out.push(AttachEv::Call(0x001a0450, 0x330));
                    } else {
                        FUN_00192bd8(jt, x, tg, 9, out);
                    }
                }
                out.push(AttachEv::Call(0x00205910, 0));
            }
            10 => {
                // anim root motion (0x12ff10 + vt+0x1c) UNPORTED -> zero
                out.push(AttachEv::Call(0x0012ff10, 0));
                x.v190 = [0.0; 3];
            }
            0xb => {
                if tg.is_some() {
                    out.push(AttachEv::Call(0x002046f0, 0));
                    out.push(AttachEv::Call(0x00204ca8, 0));
                }
            }
            _ => {}
        }
    }
    // ---- tail ----
    match jt.attach.sub {
        0 if jt.attach.kind == 3 => {
            out.push(AttachEv::Call(0x00189c40, 0));
            out.push(AttachEv::Call(0x00193c98, 0));
        }
        1 => {
            out.push(AttachEv::Call(0x001a0450, 0x70));
            if x.cam == CamOv::Attach && (1..=2).contains(&x.cam_mode) {
                let w = if x.cam_mode == 1 { x.b8_w[1] } else { x.b8_w[0] };
                x.f_50 = x.f_50.max(w);
            }
        }
        _ => {}
    }
}

// ------------------------------------------------------------------------------------------------
// 0x1911a8 input
// ------------------------------------------------------------------------------------------------

/// Inputs of 0x1911a8 the port does not own.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AttachInput {
    /// DAT_002dfbc4 & 1 (no need to hold O)
    pub auto_hold: bool,
    /// vt+0x318(jt, 3) != 0
    pub attack_ok: bool,
    /// FUN_00131968(jt+0x1030) anim done (or no anim in the slot)
    pub anim_done: bool,
    /// kind 8 stand-up clearance test passed (0x20faa0 / 0x1a8b50 ... UNPORTED)
    pub stand_clear: bool,
}

/// 0x001911a8(a): attach input (1:1 control flow; the restart loop runs while the substate changes).
pub fn FUN_001911a8(jt: &mut Jt, x: &mut AttachExt, tg: Option<&AttachTarget>, inp: &AttachInput, out: &mut Vec<AttachEv>) {
    let drop = |out: &mut Vec<AttachEv>| out.push(AttachEv::SetJtJts { m: 0 });
    let atk = |jt: &mut Jt, out: &mut Vec<AttachEv>| {
        if jt.joy.FUN_001c4660(jt.t, 0.15, JOY_ATTACK) != 0 && inp.attack_ok {
            jt.joy.FUN_001c4738(JOY_ATTACK);
            out.push(AttachEv::Call(0x0019cc50, 0));
        }
    };
    loop {
        let old = jt.attach.sub;
        let kind = jt.attach.kind;
        let t = jt.t;
        match kind {
            3 | 4 => {
                if jt.joy.FUN_001c46c0(t, 0.1, JOY_X) != 0 {
                    out.push(AttachEv::Jump { kind: 1, idx: 0 });
                } else {
                    atk(jt, out);
                }
            }
            1 => {
                if 0.15 < t - x.t_sub && jt.joy.FUN_001c46c0(t, 0.25, JOY_X) != 0 {
                    out.push(AttachEv::Jump { kind: 7, idx: 0 }); // t1 = a->+0x14 target
                } else {
                    atk(jt, out);
                }
            }
            2 => {
                out.push(AttachEv::Call(0x001441a0, 0)); // gadget charge / 0x141ea8 hold mask UNPORTED
            }
            5 => out.push(AttachEv::Call(0x001423a0, 0)),
            7 => {
                drop(out);
                out.push(AttachEv::Call(0x19c, 0)); // target vt+0x19c(target, jt)
            }
            8 => match jt.attach.sub {
                7 => {
                    if inp.anim_done {
                        FUN_00192bd8(jt, x, tg, 9, out);
                    }
                }
                8 => {
                    if inp.anim_done {
                        drop(out);
                    }
                }
                9 | 10 => {
                    if inp.stand_clear {
                        FUN_00192bd8(jt, x, tg, 8, out);
                    }
                }
                _ => {}
            },
            9 => out.push(AttachEv::Call(0x001cd098, 0)), // reach test DAT_002bf5b4 needs a+0x474
            10 => {
                if inp.anim_done {
                    drop(out);
                } else if DAT_002BF5B8 < t - jt.attach.t && (0.0 < jt.joy.mag || jt.joy.held & 0xd0 != 0) {
                    drop(out);
                    return;
                }
            }
            0xb | 0xc => {
                if inp.anim_done {
                    drop(out);
                }
            }
            _ => {
                // 0, 0xd, 6 and out of range (0x1918a0)
                if jt.joy.FUN_001c46c0(t, 0.1, JOY_X) != 0 {
                    out.push(AttachEv::Jump { kind: 1, idx: 0 });
                } else if jt.joy.FUN_001c4660(t, 0.15, JOY_ATTACK) != 0 && inp.attack_ok && (kind == 0 || kind == 0xd) {
                    jt.joy.FUN_001c4738(JOY_ATTACK);
                    out.push(AttachEv::Call(0x0019cc50, 0));
                } else if jt.attach.f_140 == 0 && !inp.auto_hold && jt.joy.held & JOY_O == 0 {
                    drop(out);
                }
            }
        }
        let s = jt.attach.sub;
        if s == old || s == -1 {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn jt_at(p: V3) -> Jt {
        let mut jt = Jt::new(p);
        jt.attach.kind = -1;
        jt.attach.sub = -1;
        jt
    }

    fn close(a: f32, b: f32, e: f32) -> bool {
        (a - b).abs() <= e
    }

    #[test]
    fn path_walk_and_closest() {
        let p = Path::polyline(vec![[0.0, 0.0, 0.0], [100.0, 0.0, 0.0], [100.0, 100.0, 0.0]], false);
        assert_eq!(p.l, vec![0.0, 100.0, 200.0]);
        let h = FUN_001e6f60(&p, [50.0, 10.0, 0.0]);
        assert!(close(h.s, 50.0, 1e-4) && h.cp == [50.0, 0.0, 0.0]);
        // walk forward from u = 50 to a point beside the second segment
        let h = FUN_001e6a30(&p, 50.0, [110.0, 30.0, 0.0]);
        assert!(close(h.s, 130.0, 1e-3), "{h:?}");
        assert_eq!(h.dir, [0.0, 1.0, 0.0]);
        // convex corner gap: forward then h < 0 on the new segment -> f = 1 at the new segment (quirk)
        let h = FUN_001e6a30(&p, 50.0, [120.0, -10.0, 0.0]);
        assert!(close(h.s, 200.0, 1e-3), "{h:?}");
        // past the end, not looped -> f = 1
        let h = FUN_001e6a30(&p, 150.0, [100.0, 300.0, 0.0]);
        assert!(close(h.s, 200.0, 1e-3));
        let (pos, dir) = FUN_001e6f10(&p, 150.0);
        assert_eq!((pos, dir), ([100.0, 50.0, 0.0], [0.0, 1.0, 0.0]));
    }

    #[test]
    fn grab_point_hang_jump_off() {
        let mut jt = jt_at([0.0, 0.0, 100.0]);
        let mut x = AttachExt { hang_r: 20.0, grab: [0.0, 0.0, 50.0], b8_vel_100: [300.0, 0.0, 0.0], ..Default::default() };
        let mut tg = AttachTarget::new(0x26, [-20.0, 0.0, 150.0]);
        tg.f_10c = 0; // kind 0 -> R = hang radius
        let mut ev = Vec::new();
        attach_to(&mut jt, &mut x, Some(&tg), 0, Some(7), 0, &mut ev);
        assert_eq!(jt.attach.sub, 5);
        assert!(ev.contains(&AttachEv::SetJtJts { m: 2 }));
        assert!(ev.contains(&AttachEv::Anim { off: 0x174, jt_rel: false, flags: 9, f0: 0.0, f1: 1.0 }));
        // pt = T - grab + row0 * 20 = (0, 0, 100): already there -> V = 0, lat = 0 (class 0x26)
        x.body_yaw = 2.0; // body not yet turned to yaw 0
        let mut ev = Vec::new();
        FUN_00191a10(&mut jt, &mut x, Some(&tg), 1.0 / 60.0, &mut ev);
        assert_eq!(jt.attach.sub, 5);
        assert_eq!(jt.yaw, 0.0);
        x.body_yaw = 0.1;
        FUN_00191a10(&mut jt, &mut x, Some(&tg), 1.0 / 60.0, &mut ev);
        assert_eq!(jt.attach.sub, 0);
        assert_eq!(x.cam_mode, 3);
        assert_eq!(x.v190, [0.0; 3]);
        // offset 10 in y: approach velocity points toward the hang point, capped by spd 300
        jt.phys.pos = [0.0, 10.0, 100.0];
        FUN_00191a10(&mut jt, &mut x, Some(&tg), 1.0 / 60.0, &mut ev);
        let g = x.goal_world();
        // 0x11d018(0, 10, dt, {300, 0, 0.3}): t = sqrt(10 / 500) = 0.1414, speed 300 (t - dt) / 0.3 = 124.7
        assert!(close(g[1], -124.7, 0.1) && g[0].abs() < 1e-4, "{g:?}");
        assert_eq!(jt.attach.sub, 1);
        // hold O: nothing; release O: drop
        jt.t = 1.0;
        jt.joy.held = JOY_O;
        let inp = AttachInput::default();
        let mut ev = Vec::new();
        FUN_001911a8(&mut jt, &mut x, Some(&tg), &inp, &mut ev);
        assert!(ev.is_empty());
        jt.joy.held = 0;
        FUN_001911a8(&mut jt, &mut x, Some(&tg), &inp, &mut ev);
        assert_eq!(ev, vec![AttachEv::SetJtJts { m: 0 }]);
        // X -> FUN_00195a00(jmp, 1, 0); press consumed
        jt.joy.feed(1.0, JOY_X | JOY_O, JOY_X);
        let mut ev = Vec::new();
        FUN_001911a8(&mut jt, &mut x, Some(&tg), &inp, &mut ev);
        assert_eq!(ev, vec![AttachEv::Jump { kind: 1, idx: 0 }]);
        assert_eq!(jt.joy.FUN_001c4660(1.0, 0.1, JOY_X), 0);
        // lead: jump leaves jts 7 -> detach
        let mut ev = Vec::new();
        FUN_00192bd8(&mut jt, &mut x, Some(&tg), -1, &mut ev);
        assert_eq!((jt.attach.kind, jt.attach.sub, jt.attach.target), (-1, -1, None));
        // the 10-unit approach ran at 124.7 > 25 (substate 1 -> attach camera), so the camera is dropped
        assert_eq!(ev, vec![AttachEv::Call(0x00140c88, 0)]);
    }

    #[test]
    fn ride_rope() {
        let path = Path::polyline(vec![[0.0, 0.0, 0.0], [100.0, 0.0, 0.0], [200.0, 0.0, 0.0]], false);
        let mut tg = AttachTarget::new(10, [0.0; 3]);
        tg.path = Some(path);
        let mut jt = jt_at([50.0, 0.0, 0.0]);
        let mut x = AttachExt { rope_vmax: 300.0, rope_v0: 0.0, ..Default::default() };
        x.rope_blend = Blend2 { on: true, k: [1.0, 1.0], vel: [[0.0; 3], [200.0, 0.0, 0.0]], aseg: true, ..Default::default() };
        let mut ev = Vec::new();
        attach_to(&mut jt, &mut x, Some(&tg), 3, Some(3), 0, &mut ev);
        assert_eq!((jt.attach.sub, x.u, x.s20, x.f_24, x.f_28), (0, 50.0, 50.0, false, -1));
        assert!(ev.contains(&AttachEv::Notify(5)) && ev.contains(&AttachEv::IgnoreCollision(true)));
        assert!(ev.contains(&AttachEv::Vis(0x1f)));
        jt.joy.angle = 0.0;
        jt.joy.mag = 1.0;
        let dt = 1.0 / 60.0;
        for _ in 0..20 {
            let mut ev = Vec::new();
            FUN_00191a10(&mut jt, &mut x, Some(&tg), dt, &mut ev);
            // r = 300 (far from the end, beyond the ramp 30) -> f = 1, w = min(1, 1)
            assert_eq!(jt.attach.sub, 4);
            assert_eq!(x.v190, [-200.0, 0.0, 0.0]);
            let g = x.goal_world();
            jt.phys.pos = add(jt.phys.pos, scale(g, dt));
        }
        // 19 integrated steps seen by the update (u lags one frame), position on segment 1
        assert!(close(x.u, 50.0 + 19.0 * 200.0 / 60.0, 1e-2), "{}", x.u);
        assert!(x.u > 100.0);
        let h = FUN_001e6a30(tg.path.as_ref().unwrap(), x.u, jt.phys.pos);
        assert!(close(h.cp[0], jt.phys.pos[0], 1e-3) && h.cp[1] == 0.0);
        // stick released -> latch -1, idle (substate 0 zeroes the goal), rope IK recorded
        jt.joy.mag = 0.0;
        let mut ev = Vec::new();
        FUN_00191a10(&mut jt, &mut x, Some(&tg), dt, &mut ev);
        assert_eq!((jt.attach.sub, x.f_28, x.v190), (0, -1, [0.0; 3]));
        assert!(ev.contains(&AttachEv::Call(0x00189c40, 0)));
        // stick backwards -> latch 1, facing backwards, yaw pi
        jt.joy.angle = PI;
        jt.joy.mag = 1.0;
        FUN_00191a10(&mut jt, &mut x, Some(&tg), dt, &mut ev);
        assert!(x.f_24 && x.f_28 == 1 && close(jt.yaw.abs(), PI, 1e-5));
        assert!(x.goal_world()[0] < -199.0);
        // X jumps off
        jt.t = 2.0;
        jt.joy.feed(2.0, JOY_X, JOY_X);
        let mut ev = Vec::new();
        FUN_001911a8(&mut jt, &mut x, Some(&tg), &AttachInput::default(), &mut ev);
        assert_eq!(ev, vec![AttachEv::Jump { kind: 1, idx: 0 }]);
        // edge rope: no stick drive, falling off the end -> FUN_00195a00(0)
        tg.f_1bc = 1;
        jt.phys.pos = [250.0, 0.0, 0.0];
        let mut ev = Vec::new();
        FUN_00191a10(&mut jt, &mut x, Some(&tg), dt, &mut ev);
        assert_eq!(ev, vec![AttachEv::Jump { kind: 0, idx: 0 }]);
    }

    #[test]
    fn climb_pole() {
        // class 0x22 round pole radius 20 at the origin, hang radius 30: JT origin at distance 50
        let mut tg = AttachTarget::new(0x22, [0.0; 3]);
        tg.r_10c = 20.0;
        let mut jt = jt_at([50.0, 0.0, 0.0]);
        let mut x = AttachExt { hang_r: 30.0, b8_vel_100: [100.0, 0.0, 0.0], body_yaw: 0.0, ..Default::default() };
        let mut ev = Vec::new();
        attach_to(&mut jt, &mut x, Some(&tg), 0, Some(1), 0, &mut ev);
        let dt = 1.0 / 60.0;
        // no stick: V = 0, lat = 0 -> substate 0
        FUN_00191a10(&mut jt, &mut x, Some(&tg), dt, &mut ev);
        assert_eq!((jt.attach.sub, x.s20), (0, 0.0));
        // stick +y (= T for side 0, yaw 0): k = 1, lat = min(sqrt(100^2 - 0), 100) = 100
        jt.joy.angle = PI / 2.0;
        jt.joy.mag = 1.0;
        let mut ev = Vec::new();
        FUN_00191a10(&mut jt, &mut x, Some(&tg), dt, &mut ev);
        assert!(close(x.v190[0], 0.0, 1e-4) && close(x.v190[1], -100.0, 1e-4) && x.v190[2] == 0.0);
        assert_eq!(jt.attach.sub, 1);
        assert_eq!(x.cam_mode, 1); // v190.y < -20, side 0 -> 1
        assert_eq!(x.cam, CamOv::Attach);
        assert!(ev.contains(&AttachEv::Anim { off: 0x170, jt_rel: false, flags: 9, f0: 0.0, f1: 1.0 }));
        let mut s_prev = x.s20;
        for _ in 0..30 {
            let g = x.goal_world();
            jt.phys.pos = add(jt.phys.pos, scale(g, dt));
            x.body_yaw = jt.yaw;
            FUN_00191a10(&mut jt, &mut x, Some(&tg), dt, &mut Vec::new());
            assert!(x.s20 > s_prev, "s must advance around the pole");
            s_prev = x.s20;
        }
        // stays near the hang circle (radius 50) thanks to the approach term
        let r = (jt.phys.pos[0].powi(2) + jt.phys.pos[1].powi(2)).sqrt();
        assert!((r - 50.0).abs() < 5.0, "{r}");
        // yaw follows theta (side 0)
        let th = jt.phys.pos[1].atan2(jt.phys.pos[0]);
        assert!(close(jt.yaw, th, 0.05));
        // detach drops the attach camera
        let mut ev = Vec::new();
        FUN_00192bd8(&mut jt, &mut x, Some(&tg), -1, &mut ev);
        assert_eq!(ev, vec![AttachEv::Call(0x00140c88, 0)]);
        assert_eq!(x.cam, CamOv::None);
    }
}
