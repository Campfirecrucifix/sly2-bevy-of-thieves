//! WP-P5 target: O-button air target pick FUN_0019b110 (+ geometry callbacks), the JMP body-state entry for the
//! flight states FUN_00195fb8 (jtbs 9..0x10, 0x12), the kinematic flight (xf kind 0x11, FUN_0022de88 case 0x11)
//! with its abort test, and the target list helpers FUN_00195368 (sparkles) / FUN_00194f70 (persistent list).
//! Specs: C:/Temp/p5/spec_19b110.md, spec_195fb8.md (verified against the disassembly).
//!
//! Inputs are plain data: `PickIn` (JT values read by 0x19b110) + `TargetObj` per category-list object with a
//! `TargetGeom` (one variant per category list, carrying the fields the callbacks read). Call per O press:
//! `FUN_0019b110(&inp, None, &world)` -> `Option<PickOut>`; then `apply_pick(&mut jmp, &out, ..)` (writes the xf
//! fields like the 0x19b110 tail and runs FUN_00195fb8). Each physics step while jtbs is 9..0x10/0x12:
//! `FUN_0022de88_case11(&mut jmp.xf, t, dt)` -> (pos, vel) to write into the JT body; `FUN_00197f98_abort` ->
//! true means FUN_00195fb8(jmp, 4, None); `arrived(&jmp.xf, t)` -> call Jt::FUN_0019c688.
//!
//! 1:1: FUN_0019b110 (all 9 categories A..I incl. forced target, repeat penalty, biases, ties keep earlier),
//! FUN_0019b088, FUN_0019a4a0 (+FUN_0019a400), FUN_0019c300 (except the prediction, see REPLACED),
//! FUN_0019ae00 (sampling / thresholds; collision query through `ClearWorld`), FUN_0011d9e0, FUN_00117120,
//! path functions FUN_001e7410 / 001e64c8 / 001e63c0 / 001e6648 / 001e6610 / 001e6630 / 001e77e8 / 001e6660 /
//! 001e66c8 / 001e7538 / 001e7668 / 001e5fc8 / 001e67b8 / 001e6f10 / 001e6f60 (+0x1e6db8 / 0x1e69a0 spline
//! refinement), FUN_00193a68, FUN_00185310 (+FUN_0011ee08), FUN_00185d18, callbacks 0x19aa58 / 0x19abb8 /
//! 0x19abf0 / 0x19aab8 / 0x19ab38 / 0x19ac88; FUN_00195fb8 exit/common/entry for jtbs -1, 9..0x10, 0x12;
//! xf kind 0x11 step (0x22f13c); abort test of 0x197f98 (0x19878c); FUN_00194f70; FUN_00195368.
//! REPLACED: body motion prediction FUN_002309e8 (target xf at now+T) and point velocity FUN_0022d348 ->
//! `TargetObj::vel` linear velocity (rotation static, no angular velocity); world->local / local->world of
//! 0x19abf0 (FUN_001a6f00 then M*lp) collapse to `wp + vel*T`. TARGET member callbacks (member vtable +0x1c,
//! object-class code, not traced) -> `MemberGeom` (Fixed point / Pole / point on the owner path at arc length
//! u), see `member_pt`. FUN_0019ae00's SO gathering 0x20faa0 + segment test 0x14cc58 (+ filters 0x20f948 /
//! 0x19b0a8) -> `ClearWorld::seg_hit`. jt is-a 0x35 / 0x37, obj is-a 0x13 and partner (DAT_002f7900) state
//! come in as flags. Scratch stack slots of 0x19b110 (aim/aimVel when 0x19a400 finds no root) start at ZERO
//! (uninitialised on the PS2). Path segment cache +0x178 / arc table cache +0x148 are recomputed (same results).
//! Unported callees (recorded as `Fx::Call`): FUN_001305b8/FUN_001f15b8 ASEG start (-> `Fx::Anim`),
//! FUN_0018e540, FUN_001ccac0, FUN_00134320, FUN_00204da8, FUN_00140c88, vt +0x2d4; 0x197f98 retarget block
//! (3a, moving targets); FUN_0015a1c0 re-arm beyond the +0x130 timer; FUN_0015b428 particle emit (particles
//! returned); FUN_001956d0 / FUN_001f7bf8 indicator spawn / fade (callback / `IndFx`).
#![allow(non_snake_case, dead_code)]

use super::FUN_0019a750;
use crate::engine::physics::driven::GameRng;
use crate::engine::physics::dynamics::{FUN_0011d018, FUN_0011d508};
use crate::engine::physics::geom::V3;
use crate::engine::physics::CollisionWorld;

// ---------------------------------------------------------------- constants
/// DAT_002bf8e0 cost reject
pub const DAT_002BF8E0: f32 = 2500.0;
/// DAT_002bf900 repeat-target penalty (jmp+0xbc)
pub const DAT_002BF900: f32 = 1000.0;
/// category biases: A 0x2bf8e4, B 0x2bf8e8, C 0x2bf8ec, D 0x2bf908, E 0x2bf904, F 0x2bf8f0, G 0x2bf8f8,
/// H 0x2bf8f4, I 0x2bf8fc
pub const DAT_002BF8E4: f32 = -250.0;
pub const DAT_002BF8E8: f32 = 0.0;
pub const DAT_002BF8EC: f32 = 0.0;
pub const DAT_002BF908: f32 = 0.0;
pub const DAT_002BF904: f32 = -500.0;
pub const DAT_002BF8F0: f32 = 1250.0;
pub const DAT_002BF8F8: f32 = 500.0;
pub const DAT_002BF8F4: f32 = 500.0;
pub const DAT_002BF8FC: f32 = -250.0;
/// DAT_002bf90c cost scale (G, H)
pub const DAT_002BF90C: f32 = 1.25;
/// DAT_002bf910 (doubled in 0x19c300)
pub const DAT_002BF910: f32 = 2.0;
/// DAT_002bf914 / DAT_002bf918 search steps (G, H)
pub const DAT_002BF914: f32 = 100.0;
pub const DAT_002BF918: f32 = 100.0;
/// DAT_002be918 surface push-in speed (0x185d18)
pub const DAT_002BE918: f32 = -300.0;
/// DAT_002bf668 rope approach accel (0x193a68)
pub const DAT_002BF668: f32 = 0.2;
/// DAT_002bf844 flight abort threshold (0x197f98)
pub const DAT_002BF844: f32 = 10000.0;
/// 0x49742400 / 0x49769500 / 0x38d1b717 / 0x3e19999a
pub const C_1E6: f32 = 1.0e6;
pub const C_101E6: f32 = 1.01e6;
pub const C_1EM4: f32 = 1.0e-4;
pub const C_015: f32 = 0.15;
/// DAT_002bf710 / DAT_002bf714 indicator spring, DAT_002bf718 {1500, 0, 2} approach profile (0x194f70)
pub const DAT_002BF710: f32 = 15.0;
pub const DAT_002BF714: f32 = 5.0;
pub const DAT_002BF718: [f32; 3] = [1500.0, 0.0, 2.0];
const ZERO: V3 = [0.0; 3];
/// DAT_002a2dc0 up
const UP: V3 = [0.0, 0.0, 1.0];

// ---------------------------------------------------------------- vector helpers
fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn mul(a: V3, s: f32) -> V3 {
    [a[0] * s, a[1] * s, a[2] * s]
}
fn dot(a: V3, b: V3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross(a: V3, b: V3) -> V3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
fn len(a: V3) -> f32 {
    dot(a, a).sqrt()
}
/// vsqrt + vdiv (no zero guard, like the VU code)
fn normalize(a: V3) -> V3 {
    mul(a, 1.0 / len(a))
}

/// World matrix of an object (vtable +0xf8/+0xfc): rows 0..2 rotation, row 3 translation; p' = r0 x + r1 y + r2 z + r3.
pub type M4 = [V3; 4];
pub const M4_IDENT: M4 = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [0.0, 0.0, 0.0]];
fn xf_vec(m: &M4, v: V3) -> V3 {
    add(add(mul(m[0], v[0]), mul(m[1], v[1])), mul(m[2], v[2]))
}
fn xf_pt(m: &M4, v: V3) -> V3 {
    add(xf_vec(m, v), m[3])
}

// ---------------------------------------------------------------- pure math
/// 0x00117120(T, u, k, p0, v0, p1, v1) -> (pos, vel, acc): cubic Hermite over duration T at time u (1:1).
pub fn FUN_00117120(T: f32, u: f32, k: f32, p0: V3, v0: V3, p1: V3, v1: V3) -> (V3, V3, V3) {
    let s = u / T;
    let w = 1.0 - s;
    let pos = add(
        add(mul(p0, w * w * w + 3.0 * s * w * w), mul(v0, s * w * w * T)),
        add(mul(v1, -(s * s * w) * T), mul(p1, s * s * s + 3.0 * s * s * w)),
    );
    let d = sub(p1, p0);
    let vel = mul(
        add(add(mul(d, s * (6.0 - 6.0 * s) / T), mul(v0, 1.0 - s * (4.0 - 3.0 * s))), mul(v1, s * (3.0 * s - 2.0))),
        k,
    );
    let acc = mul(
        add(add(mul(d, (6.0 - 12.0 * s) / T), mul(v0, 6.0 * s - 4.0)), mul(v1, 6.0 * s - 2.0)),
        k * k / T,
    );
    (pos, vel, acc)
}

/// 0x0011d9e0(x0, step, lo, hi, cb, ctx, &xBest, &fBest): bracket the minimum by stepping downhill, then 10
/// bisection rounds (1:1). Returns (xBest, fBest).
pub fn FUN_0011d9e0<F: FnMut(f32) -> f32>(x0: f32, step: f32, lo: f32, hi: f32, mut cb: F) -> (f32, f32) {
    let mut b = if lo <= x0 { if x0 <= hi { x0 } else { hi } } else { lo };
    let mut a = (b - step).max(lo);
    let mut fa = cb(a);
    let mut fb = cb(b);
    let mut c = (b + step).min(hi);
    let mut fc;
    loop {
        fc = cb(c);
        while fa < fb {
            c = b;
            fc = fb;
            b = a;
            fb = fa;
            a = (b + b - c).max(lo);
            fa = cb(a);
        }
        if fb <= fc {
            break;
        }
        a = b;
        fa = fb;
        b = c;
        fb = fc;
        c = (b + b - a).min(hi);
    }
    let _ = (fa, fc);
    for _ in 0..10 {
        let m1 = (a + b) * 0.5;
        let f1 = cb(m1);
        let m2 = (b + c) * 0.5;
        let f2 = cb(m2);
        if f1 < fb {
            if f1 < f2 {
                c = b;
                b = m1;
                fb = f1;
            } else {
                a = b;
                b = m2;
                fb = f2;
            }
        } else if f2 < fb {
            a = b;
            b = m2;
            fb = f2;
        } else {
            a = m1;
            c = m2;
        }
    }
    (b, fb)
}

/// 0x0011ee08(q, a, b, &pt, &f, &dist): closest point on segment a..b to q -> (pt, f) (1:1).
pub fn FUN_0011ee08(q: V3, a: V3, b: V3) -> (V3, f32) {
    let d = sub(b, a);
    let f = (dot(sub(q, a), d) / dot(d, d)).clamp(0.0, 1.0);
    (add(a, mul(d, f)), f)
}

/// 0x0011d898(x, m): fmodf(x, m), + m when negative.
fn FUN_0011d898(x: f32, m: f32) -> f32 {
    let r = x % m;
    if r < 0.0 {
        r + m
    } else {
        r
    }
}

// ---------------------------------------------------------------- path object (rope / rail)
/// Path object (+0xe0..+0x178). `s`, `l`, `pts` and the spline tangents are per knot.
#[derive(Clone, Debug, PartialEq)]
pub struct Path {
    /// +0xe0: false polyline (type 0), true spline (type 1)
    pub spline: bool,
    /// +0xe4
    pub looped: bool,
    /// +0xec knot parameters
    pub s: Vec<f32>,
    /// +0xf0 cumulative arc length per knot
    pub l: Vec<f32>,
    /// +0xf4 knots (local)
    pub pts: Vec<V3>,
    /// +0x140 per-knot incoming tangent / +0x144 outgoing tangent (spline, local)
    pub tan_in: Vec<V3>,
    pub tan_out: Vec<V3>,
    /// vtable +0xfc world matrix (0x1e5fc8 reads the same rows at +0x100)
    pub xf: M4,
}

/// Closest point result of FUN_001e6f60.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PathClosest {
    pub cp: V3,
    pub dir: V3,
    pub s: f32,
    pub l: f32,
}

impl Path {
    /// Helper (not original): polyline with s = l = cumulative world-independent length of `pts`.
    pub fn polyline(pts: Vec<V3>, xf: M4) -> Path {
        let mut l = vec![0.0f32];
        for i in 1..pts.len() {
            let v = l[i - 1] + len(sub(pts[i], pts[i - 1]));
            l.push(v);
        }
        Path { spline: false, looped: false, s: l.clone(), l, pts, tan_in: vec![], tan_out: vec![], xf }
    }
    fn n(&self) -> usize {
        self.s.len()
    }
    /// 0x001e6610 first knot param
    pub fn FUN_001e6610(&self) -> f32 {
        self.s[0]
    }
    /// 0x001e6630 last knot param
    pub fn FUN_001e6630(&self) -> f32 {
        self.s[self.n() - 1]
    }
    /// 0x001e6648 total arc length L[n-1]
    pub fn FUN_001e6648(&self) -> f32 {
        self.l[self.n() - 1]
    }
    /// 0x001e77e8 coarse search step (s_last - s0) / (n * 10)
    pub fn FUN_001e77e8(&self) -> f32 {
        (self.FUN_001e6630() - self.FUN_001e6610()) / (self.n() as f32 * 10.0)
    }
    /// 0x001e6660 wrap param into [s0, s_last] when looped
    pub fn FUN_001e6660(&self, x: f32) -> f32 {
        let (a, b) = (self.s[0], self.s[self.n() - 1]);
        if self.looped && (x < a || b < x) {
            a + FUN_0011d898(x - a, b - a)
        } else {
            x
        }
    }
    /// 0x001e66c8 same on arc length
    pub fn FUN_001e66c8(&self, x: f32) -> f32 {
        let (a, b) = (self.l[0], self.l[self.n() - 1]);
        if self.looped && (x < a || b < x) {
            a + FUN_0011d898(x - a, b - a)
        } else {
            x
        }
    }
    /// largest i in [0, n-2] with tab[i] <= x (0 when none): result of the cached scans of 0x1e7538/0x1e7668.
    fn seg(tab: &[f32], x: f32) -> usize {
        let mut i = 0;
        for j in 0..tab.len().saturating_sub(1) {
            if tab[j] <= x {
                i = j;
            }
        }
        i
    }
    /// 0x001e7538(s) -> (i, ds, dseg)
    pub fn FUN_001e7538(&self, x: f32) -> (usize, f32, f32) {
        let x = self.FUN_001e6660(x);
        let i = Self::seg(&self.s, x);
        (i, x - self.s[i], self.s[i + 1] - self.s[i])
    }
    /// 0x001e7668(L) -> (i, dL, dLseg)
    pub fn FUN_001e7668(&self, x: f32) -> (usize, f32, f32) {
        let x = self.FUN_001e66c8(x);
        let i = Self::seg(&self.l, x);
        (i, x - self.l[i], self.l[i + 1] - self.l[i])
    }
    /// world Hermite segment i (p0, v0 = +0x144[i], p1, v1 = +0x140[i+1])
    fn wseg(&self, i: usize) -> (V3, V3, V3, V3) {
        (
            xf_pt(&self.xf, self.pts[i]),
            xf_vec(&self.xf, self.tan_out[i]),
            xf_pt(&self.xf, self.pts[i + 1]),
            xf_vec(&self.xf, self.tan_in[i + 1]),
        )
    }
    /// 0x001e5fc8(i): 11-entry arc-length table (+0x14c) of spline segment i, 0.1 T steps.
    pub fn FUN_001e5fc8(&self, i: usize) -> [f32; 11] {
        let (p0, v0, p1, v1) = self.wseg(i);
        let T = self.s[i + 1] - self.s[i];
        let mut tab = [0.0f32; 11];
        let mut prev = ZERO;
        for k in 0..11 {
            let (p, _, _) = FUN_00117120(T, k as f32 * 0.1 * T, 1.0, p0, v0, p1, v1);
            if k > 0 {
                tab[k] = tab[k - 1] + len(sub(p, prev));
            }
            prev = p;
        }
        tab
    }
    /// 0x001e63c0(s) -> arc length
    pub fn FUN_001e63c0(&self, x: f32) -> f32 {
        let (i, ds, dseg) = self.FUN_001e7538(x);
        if !self.spline {
            let f = ds / dseg;
            (1.0 - f) * self.l[i] + f * self.l[i + 1]
        } else {
            let tab = self.FUN_001e5fc8(i);
            let x = ds * 10.0 / dseg;
            let k = x as i32; // trunc
            let f = x - k as f32;
            // k == 10 reads +0x178 (f = 0 there); clamp for safety
            let k0 = k.clamp(0, 10) as usize;
            let k1 = (k + 1).clamp(0, 10) as usize;
            (1.0 - f) * tab[k0] + f * tab[k1] + self.l[i]
        }
    }
    /// 0x001e64c8(L) -> param s
    pub fn FUN_001e64c8(&self, lq: f32) -> f32 {
        let (i, dl, dlseg) = self.FUN_001e7668(lq);
        let f = if !self.spline {
            dl / dlseg
        } else {
            let tab = self.FUN_001e5fc8(i);
            let mut k = 1;
            if tab[1] <= dl {
                k = 2;
                while k < 10 && tab[k] <= dl {
                    k += 1;
                }
            }
            ((k - 1) as f32 + (dl - tab[k - 1]) / (tab[k] - tab[k - 1])) * 0.1
        };
        (1.0 - f) * self.s[i] + f * self.s[i + 1]
    }
    /// 0x001e67b8(s) -> (world pos, unit world tangent)
    pub fn FUN_001e67b8(&self, x: f32) -> (V3, V3) {
        let (i, ds, dseg) = self.FUN_001e7538(x);
        if !self.spline {
            let f = ds / dseg;
            let p = add(mul(self.pts[i], 1.0 - f), mul(self.pts[i + 1], f));
            (xf_pt(&self.xf, p), normalize(xf_vec(&self.xf, sub(self.pts[i + 1], self.pts[i]))))
        } else {
            let (p, v, _) =
                FUN_00117120(dseg, ds, 1.0, self.pts[i], self.tan_out[i], self.pts[i + 1], self.tan_in[i + 1]);
            (xf_pt(&self.xf, p), normalize(xf_vec(&self.xf, v)))
        }
    }
    /// 0x001e6f10(L) = 0x1e67b8(0x1e64c8(L))
    pub fn FUN_001e6f10(&self, lq: f32) -> (V3, V3) {
        self.FUN_001e67b8(self.FUN_001e64c8(lq))
    }
    /// 0x001e6f60(q, xf = 0) closest point on the path (type 0 inline, type 1 via 0x1e6ed8 -> 0x1e6db8).
    pub fn FUN_001e6f60(&self, q: V3) -> PathClosest {
        let n = self.n();
        if !self.spline {
            let mut a = xf_pt(&self.xf, self.pts[0]);
            let mut best = f32::MAX; // DAT_002a8138
            let mut r = PathClosest { cp: ZERO, dir: ZERO, s: -1.0, l: 0.0 };
            for i in 1..n {
                let b = xf_pt(&self.xf, self.pts[i]);
                let d = sub(b, a);
                let f = (dot(sub(q, a), d) / dot(d, d)).clamp(0.0, 1.0);
                let p = add(mul(a, 1.0 - f), mul(b, f));
                let dist = len(sub(q, p));
                if dist < best {
                    best = dist;
                    r.s = (1.0 - f) * self.s[i - 1] + f * self.s[i];
                    r.cp = p;
                    r.dir = normalize(d);
                }
                a = b;
            }
            r.l = self.FUN_001e63c0(r.s);
            r
        } else {
            let mut k = 0usize;
            let mut best = f32::MAX;
            for i in 0..n {
                let d = len(sub(q, xf_pt(&self.xf, self.pts[i])));
                if d < best {
                    best = d;
                    k = i;
                }
            }
            // 0x1e6db8
            let (lo, hi) = if self.looped { (-f32::MAX, f32::MAX) } else { (self.s[0], self.s[n - 1]) };
            let (s, _) = FUN_0011d9e0(self.s[k], (self.s[1] - self.s[0]) * 0.1, lo, hi, |s| {
                // 0x1e69a0: squared distance
                let (p, _) = self.FUN_001e67b8(s);
                let d = sub(q, p);
                dot(d, d)
            });
            let s = self.FUN_001e6660(s);
            let (cp, dir) = self.FUN_001e67b8(s);
            PathClosest { cp, dir, s, l: self.FUN_001e63c0(s) }
        }
    }
    /// 0x001e7410(cb): knot param with the smallest cb(s[i]) (strict <, init FLT_MAX, default 0).
    pub fn FUN_001e7410<F: FnMut(f32) -> f32>(&self, mut cb: F) -> f32 {
        let mut best = f32::MAX; // DAT_002a813c
        let mut sb = 0.0;
        for &s in &self.s {
            let c = cb(s);
            if c < best {
                best = c;
                sb = s;
            }
        }
        sb
    }
}

// ---------------------------------------------------------------- triangle mesh (surface)
/// Surface object (+0xe4 verts, +0xe8 count, +0xec tris {i0,i1,i2}).
#[derive(Clone, Debug, PartialEq)]
pub struct Mesh {
    pub verts: Vec<V3>,
    pub tris: Vec<[usize; 3]>,
    /// vtable +0xfc world matrix
    pub xf: M4,
}

/// FUN_00185310 result: tri {index, u, v}, closest point, unit normal ((a-c) x (b-c)).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeshClosest {
    pub tri: (usize, f32, f32),
    pub cp: V3,
    pub n: V3,
}

/// 0x00185310(mesh, q): closest point on the mesh (1:1; None when the mesh has no triangles, where the PS2
/// returns uninitialised stack).
pub fn FUN_00185310(m: &Mesh, q: V3) -> Option<MeshClosest> {
    let mut best = f32::MAX; // DAT_002a5684
    let mut out: Option<(usize, f32, f32, V3, V3)> = None;
    for (i, t) in m.tris.iter().enumerate() {
        let a = xf_pt(&m.xf, m.verts[t[0]]);
        let b = xf_pt(&m.xf, m.verts[t[1]]);
        let c = xf_pt(&m.xf, m.verts[t[2]]);
        let e1 = sub(a, c);
        let e2 = sub(b, c);
        let n = cross(e1, e2);
        let qc = sub(q, c);
        let ne2 = cross(n, e2);
        let inv = 1.0 / dot(e1, ne2);
        let mut u = inv * dot(qc, ne2);
        let mut v = inv * dot(n, cross(qc, e1));
        let p;
        if u < 0.0 {
            let (pp, f) = FUN_0011ee08(q, b, c);
            p = pp;
            u = 0.0;
            v = 1.0 - f;
        } else if v < 0.0 {
            let (pp, f) = FUN_0011ee08(q, a, c);
            p = pp;
            v = 0.0;
            u = 1.0 - f;
        } else if 1.0 < u + v {
            let (pp, f) = FUN_0011ee08(q, a, b);
            p = pp;
            u = 1.0 - f;
            v = f;
        } else {
            p = add(add(mul(a, u), mul(b, v)), mul(c, (1.0 - u) - v));
        }
        let d = sub(p, q);
        let d2 = dot(d, d);
        if d2 < best {
            best = d2;
            out = Some((i, u, v, p, n));
        }
    }
    out.map(|(i, u, v, p, n)| MeshClosest { tri: (i, u, v), cp: p, n: normalize(n) })
}

/// 0x00185d18(mesh, P, v, g, K, T) -> (tri, pos, vel): ballistic point at T projected onto the mesh, pushed out by
/// K along the normal, arrival velocity v + gT - 300 n (1:1).
pub fn FUN_00185d18(m: &Mesh, p: V3, v: V3, g: V3, k: f32, T: f32) -> ((usize, f32, f32), V3, V3) {
    let b = add(add(p, mul(v, T)), mul(g, T * T * 0.5));
    let r = FUN_00185310(m, b).unwrap_or(MeshClosest { tri: (0, 0.0, 0.0), cp: b, n: ZERO });
    (r.tri, add(r.cp, mul(r.n, k)), add(add(v, mul(g, T)), mul(r.n, DAT_002BE918)))
}

// ---------------------------------------------------------------- targets
/// TARGET member point callback (member vtable +0x1c; REPLACED, see module doc).
#[derive(Clone, Debug, PartialEq)]
pub enum MemberGeom {
    /// returns a fixed world point for every u
    Fixed(V3),
    /// pole axis a..b: point on the axis nearest the JT (clamped), plus `radius` towards angle u / scale
    Pole { a: V3, b: V3, radius: f32 },
    /// point on the owner's path at arc length u (0x1e6f10)
    PathArc,
}

/// One object of a 0x19b110 category list. `geom` selects the list.
#[derive(Clone, Debug, PartialEq)]
pub struct TargetObj {
    /// object handle (forced target / jmp+0xbc compare / result)
    pub id: usize,
    /// category mask field (+0xfc / +0x1ac / +0x11c), tested against jt+0x1000 (not for Surface)
    pub mask: u32,
    /// max drop P.z - pt.z (+0xe8 / +0x198 / +0x108)
    pub range: f32,
    /// REPLACED body velocity (FUN_0022d348 point velocity / FUN_002309e8 prediction)
    pub vel: V3,
    pub geom: TargetGeom,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TargetGeom {
    /// A: DAT_003e13d8 (kind 9), member +0xe0 evaluated at u = 0
    Spot9 { pt: V3 },
    /// B: DAT_003e13cc rope path (kind 0xa)
    Rope { path: Path },
    /// C: DAT_003e13b4 point/hook (kind 0xe): pos(X), +0xe8 bit 0, is-a 0x13
    Point { pos: V3, f_e8: bool, is_13: bool },
    /// D: DAT_003e142c surface mesh (kind 0x10)
    Surface { mesh: Mesh },
    /// F: DAT_003e13c0 path on a moving body (kind 0xf)
    PathBody { path: Path },
    /// G: DAT_003e13fc angle-parametrised target (kind 0xc): pos(X), +0x10c angle scale, member +0xe0
    Pole { pos: V3, scale: f32, member: MemberGeom },
    /// H: DAT_003e13f0 path with member +0x190 (kind 0xb)
    Rail { path: Path, member: MemberGeom },
    /// I: DAT_003e1408 (kind 0xd), member +0xe0 at u = 0
    Spot0xd { pt: V3 },
}

/// Category E (is-a 0x37 JT, partner DAT_002f7900).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Partner {
    /// o2 = *(0x2f7900)
    pub o2: usize,
    /// *(o2+0x13d0) == 0xf && *(o2+0x1914) == 0
    pub ready: bool,
    /// X = *(o2+0x191c), pos(X), its velocity
    pub x: usize,
    pub pos: V3,
    pub vel: V3,
}

/// Rope inputs of 0x193a68 (attach jt+0x4c0 and joystick jt+0x150).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RopeIn {
    /// attach +0x2a0 (speed scale / vmax) and +0x260 (v0)
    pub a2a0: f32,
    pub a260: f32,
    /// joy +0x18 angle, +0x1c magnitude
    pub joy_angle: f32,
    pub joy_mag: f32,
}

/// JT values read by 0x19b110.
#[derive(Clone, Debug)]
pub struct PickIn<'a> {
    /// DAT_002dded0 / DAT_002dded4
    pub t: f32,
    pub dt: f32,
    /// FUN_001a62e8(jt), jt vt+0x10c velocity, **(jt+0x218) sphere centre, jt+0x240 hand offset
    pub pos: V3,
    pub vel: V3,
    pub centre: V3,
    pub hofs: V3,
    /// FEEL R98: FUN_001a62e8 xform translation - `pos` (port pos is the JT point, Sly SO = pos + 45 up)
    pub so_ofs: V3,
    /// *(jt+0xe8)+0x80 gravity
    pub g: V3,
    /// jt+0x1020 reach radius, jt+0x1f30 surface standoff
    pub reach: f32,
    pub standoff: f32,
    /// jt+0x1000
    pub mask: u32,
    /// is_a(jt, 0x35) (categories A..D), is_a(jt, 0x37) (E)
    pub is_35: bool,
    pub is_37: bool,
    /// jmp+0xbc last target
    pub last: Option<usize>,
    pub rope: RopeIn,
    pub partner: Option<Partner>,
    pub targets: &'a [TargetObj],
}

/// Result of 0x19b110 (the values its tail writes).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PickOut {
    /// JMP body state for FUN_00195fb8 and target object
    pub kind: i32,
    pub obj: usize,
    /// xf+0xc0 = aimPt - HOFS, xf+0xd0 = aimVel, xf+0xa8 = t + tArrive
    pub aim: V3,
    pub aim_vel: V3,
    pub deadline: f32,
    pub t_arrive: f32,
    /// jmp+0x10 (rope param s) / jmp+0x14 (arc length / param)
    pub f_10: f32,
    pub f_14: f32,
    pub cost: f32,
}

/// Collision filter of FUN_0019ae00: 0x20f948 (layer, self excluded) or 0x19b0a8 (jt and partner excluded).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ClearFilter {
    Layer,
    Partner(usize),
}

/// FUN_0020faa0 gather + FUN_0014cc58 segment test (REPLACED interface): first hit on a->b as (point, hit +0x44 frac).
pub trait ClearWorld {
    fn seg_hit(&self, a: V3, b: V3, filter: ClearFilter) -> Option<(V3, f32)>;
}
/// No obstacles.
pub struct OpenWorld;
impl ClearWorld for OpenWorld {
    fn seg_hit(&self, _: V3, _: V3, _: ClearFilter) -> Option<(V3, f32)> {
        None
    }
}
/// REPLACED: static collision only (no SO ownership -> filter ignored); first clip record by entry t.
/// WP-P14: FUN_0020faa0 gathers only SOs whose bounding box overlaps the segment box (REPLACED: AABB of the
/// collider's vertices), so geometry far away (e.g. row 14 obj 3577, an open "solid outside" sky mesh at
/// z 25000 whose BSP classifies every point as inside) is never tested.
impl ClearWorld for CollisionWorld {
    fn seg_hit(&self, a: V3, b: V3, _: ClearFilter) -> Option<(V3, f32)> {
        use crate::engine::physics::bsp::FUN_00118668;
        let lo = [a[0].min(b[0]), a[1].min(b[1]), a[2].min(b[2])];
        let hi = [a[0].max(b[0]), a[1].max(b[1]), a[2].max(b[2])];
        let mut best: Option<(V3, f32)> = None;
        for c in &self.colliders {
            if c.bsp.nodes.is_empty() || c.geom.verts.is_empty() {
                continue;
            }
            let (mut bl, mut bh) = (c.geom.verts[0], c.geom.verts[0]);
            for v in &c.geom.verts {
                for k in 0..3 {
                    bl[k] = bl[k].min(v[k]);
                    bh[k] = bh[k].max(v[k]);
                }
            }
            if (0..3).any(|k| bh[k] < lo[k] || bl[k] > hi[k]) {
                continue;
            }
            let mut tmp = Vec::new();
            let n = FUN_00118668(&c.geom, &c.bsp, 0, c.bsp.nodes.len(), a, b, 0x40, &mut tmp);
            for l in &tmp[..n] {
                if best.map_or(true, |(_, t)| l.t[0] < t) {
                    best = Some((l.pos[0], l.t[0]));
                }
            }
        }
        best
    }
}

/// Trajectory result of 0x19a4a0 (aim / aim_vel None when 0x19a400 found no positive root: not written).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Traj {
    pub t: f32,
    pub cost: f32,
    pub aim: Option<V3>,
    pub aim_vel: Option<V3>,
}

/// 0x0019a400 + 0x0019a4a0(jmp, vin, X, tp, vx): ballistic arrival time at the height of tp, aim point / velocity
/// and Hermite acceleration cost (xy). `vtp` = velocity of the target point (REPLACED 0x22d348).
pub fn FUN_0019a4a0(p: V3, gz: f32, vin: V3, vtp: V3, tp: V3, vx: V3) -> Traj {
    let dvz = vin[2] - vtp[2];
    let mut roots = [0.0f32; 2];
    let n = FUN_0011d508(gz * 0.5, dvz, p[2] - tp[2], &mut roots);
    if n == 0 || roots[n - 1] <= 0.0 {
        let t = (-dvz / gz).max(0.0);
        let h = (p[2] - tp[2]) + (dvz * t + gz * (t * 0.5) * t);
        return Traj { t, cost: C_101E6 - h, aim: None, aim_vel: None };
    }
    let t = roots[n - 1];
    let vt = add(vtp, vx);
    let q = add(tp, mul(vtp, C_015.min(t)));
    let (_, _, a0) = FUN_00117120(t, 0.0, 1.0, p, vin, q, vt);
    let (_, _, a1) = FUN_00117120(t, t, 1.0, p, vin, q, vt);
    let l0 = (a0[0] * a0[0] + a0[1] * a0[1]).sqrt();
    let l1 = (a1[0] * a1[0] + a1[1] * a1[1]).sqrt();
    let cost = ((l0 + l1) * (t * 0.5)).min(C_1E6);
    Traj { t, cost, aim: Some(q), aim_vel: Some(vt) }
}

/// 0x0019c300(jmp, X, T, off) -> (cost, aim, aimVel): ballistic flight of duration T to the reach sphere
/// (radius R) around the target point `xp` (= predicted M*off; REPLACED: caller passes base + vel*T) (1:1).
pub fn FUN_0019c300(p: V3, v: V3, g: V3, r: f32, T: f32, xp: V3) -> (f32, V3, V3) {
    let a = add(v, mul(g, T));
    let h = T * 0.5;
    let b = add(add(p, mul(v, h)), mul(a, h));
    let mut d = sub(xp, b);
    if dot(d, d) < r * r {
        return (len(d) - r, b, a);
    }
    let adz = d[2].abs();
    if r < adz || T == 0.0 {
        return (adz + C_101E6, p, a);
    }
    let dz = d[2];
    d[2] = 0.0;
    let hl = len(d);
    let dir = if hl < C_1EM4 { ZERO } else { mul(d, 1.0 / hl) };
    let e = hl - (r * r - dz * dz).sqrt();
    let k = (e + e) / (T * T);
    let a2 = add(add(v, mul(g, T)), mul(mul(dir, k), T));
    let cost = ((DAT_002BF910 + DAT_002BF910) * e / T).min(C_1E6);
    let aim = add(add(p, mul(v, h)), mul(a2, h));
    (cost, aim, a2)
}

/// 0x00193a68(attach, rope, s) -> (pos - HOFS, vel): grab point on the rope at s and its slide velocity from the
/// stick along the horizontal tangent, limited by the approach speeds to the rope ends (1:1).
pub fn FUN_00193a68(path: &Path, s: f32, ri: &RopeIn, hofs: V3, dt: f32) -> (V3, V3) {
    let (p, tan) = path.FUN_001e67b8(s);
    let h = sub(tan, mul(UP, dot(tan, UP)));
    let l = len(h);
    let h = if l < C_1EM4 { ZERO } else { mul(h, 1.0 / l) };
    let stick = super::FUN_0011e1a0(ri.joy_angle, ri.a2a0 * ri.joy_mag, 0.0);
    let w = dot(stick, h);
    let lq = path.FUN_001e63c0(s);
    let prm = [ri.a2a0, ri.a260, DAT_002BF668];
    let mut vlo = 0.0;
    let mut vhi = 0.0;
    FUN_0011d018(lq, 50.0, dt, prm, Some(&mut vlo));
    FUN_0011d018(lq, path.FUN_001e6648() - 50.0, dt, prm, Some(&mut vhi));
    let r = if w < vlo {
        vlo
    } else if vhi < w {
        vhi
    } else {
        w
    };
    (sub(p, hofs), mul(h, r))
}

/// 0x0019ae00(jmp, filt, ctx, t, P, vin, aim, aimV, tgt) -> blocked: samples the Hermite flight every 0.1 s and
/// tests each chord (hit farther than 100 from aim blocks), then aim -> tgt (hit with frac != 1 farther than 50).
pub fn FUN_0019ae00(w: &dyn ClearWorld, filt: ClearFilter, t: f32, p: V3, vin: V3, aim: V3, aim_vel: V3, tgt: Option<V3>) -> bool {
    let mut u = 0.0f32;
    let mut prev = ZERO;
    loop {
        let (pos, _, _) = FUN_00117120(t, u, 1.0, p, vin, aim, aim_vel);
        if u != 0.0 {
            if let Some((hp, _)) = w.seg_hit(prev, pos, filt) {
                if 100.0 < len(sub(hp, aim)) {
                    return true;
                }
            }
        }
        prev = pos;
        if u == t {
            break;
        }
        u = (u + 0.1).min(t);
    }
    if let Some(q) = tgt {
        if let Some((hp, f)) = w.seg_hit(aim, q, filt) {
            if f != 1.0 && 50.0 < len(sub(hp, q)) {
                return true;
            }
        }
    }
    false
}

/// 0x0019b088(jmp, X, c, bias): c + bias, + 1000 when X is the last target (jmp+0xbc).
pub fn FUN_0019b088(last: Option<usize>, x: usize, c: f32, bias: f32) -> f32 {
    let c = c + bias;
    if last == Some(x) {
        c + DAT_002BF900
    } else {
        c
    }
}

/// member +0x1c callback (REPLACED, see `MemberGeom`); `scale` = owner +0x10c (G), `path` = owner path (H).
pub fn member_pt(m: &MemberGeom, u: f32, jt_pos: V3, scale: f32, path: Option<&Path>) -> V3 {
    match m {
        MemberGeom::Fixed(p) => *p,
        MemberGeom::Pole { a, b, radius } => {
            let (c, _) = FUN_0011ee08(jt_pos, *a, *b);
            let ang = if scale != 0.0 { u / scale } else { u };
            add(c, [ang.cos() * radius, ang.sin() * radius, 0.0])
        }
        MemberGeom::PathArc => path.map(|p| p.FUN_001e6f10(u).0).unwrap_or(ZERO),
    }
}

/// 0x0019b110(jmp, forced): O-button air target pick over categories A..I (see spec_19b110.md). Returns the
/// winning target (None = kind -1, nothing written).
pub fn FUN_0019b110(inp: &PickIn, forced: Option<usize>, world: &dyn ClearWorld) -> Option<PickOut> {
    let p = inp.pos;
    let v = inp.vel;
    let g = inp.g;
    let gz = g[2];
    // 0x19a750(jmp, 4): launch velocity kind 4
    let v0 = [v[0], v[1], FUN_0019a750(4, v[2], gz, 0.0)];
    let mask = inp.mask;
    let last = inp.last;
    let hofs = inp.hofs;
    let mut kind = -1i32;
    let mut obj = 0usize;
    let mut best = if forced.is_some() { -f32::MAX } else { f32::MAX }; // DAT_002a65b8 / DAT_002a65bc
    let mut t_arr = 0.0f32; // f24
    let mut f27 = 0.0f32;
    let mut f26 = 0.0f32;
    let mut aim_pt = ZERO; // sp+0x40
    let mut aim_vel = ZERO; // sp+0x60
    // scratch outputs of TRAJ (stale when 0x19a400 has no root)
    let mut aim_s = ZERO;
    let mut av_s = ZERO;
    let is_f = |id: usize| forced == Some(id);
    let clear = |t: f32, vin: V3, aim: V3, av: V3, tgt: Option<V3>, f: ClearFilter| {
        FUN_0019ae00(world, f, t, p, vin, aim, av, tgt)
    };
    // TRAJ + aimVel.z rewrite
    let traj = |vin: V3, vtp: V3, tp: V3, vx: V3, aim_s: &mut V3, av_s: &mut V3| -> Traj {
        let r = FUN_0019a4a0(p, gz, vin, vtp, tp, vx);
        if let (Some(a), Some(b)) = (r.aim, r.aim_vel) {
            *aim_s = a;
            *av_s = b;
        }
        av_s[2] = vin[2] + gz * r.t;
        r
    };

    if inp.is_35 {
        // A) DAT_003e13d8 -> kind 9
        for x in inp.targets {
            let TargetGeom::Spot9 { pt } = &x.geom else { continue };
            if x.mask & mask == 0 {
                continue;
            }
            let f = is_f(x.id);
            if !f && p[2] - pt[2] > x.range {
                continue;
            }
            let r = traj(v0, x.vel, *pt, ZERO, &mut aim_s, &mut av_s);
            let mut cost = r.cost;
            if !f {
                if cost >= DAT_002BF8E0 {
                    continue;
                }
                cost = FUN_0019b088(last, x.id, cost, DAT_002BF8E4);
                if best <= cost {
                    continue;
                }
                if clear(r.t, v0, aim_s, av_s, None, ClearFilter::Layer) {
                    continue;
                }
            }
            kind = 9;
            obj = x.id;
            aim_pt = aim_s;
            aim_vel = av_s;
            best = best.min(cost);
            t_arr = r.t;
        }
        // B) DAT_003e13cc rope -> kind 0xa
        for x in inp.targets {
            let TargetGeom::Rope { path } = &x.geom else { continue };
            if x.mask & mask == 0 {
                continue;
            }
            let f = is_f(x.id);
            if !f {
                let cp = path.FUN_001e6f60(inp.centre).cp;
                if p[2] - cp[2] > x.range {
                    continue;
                }
            }
            // cb 0x19aa58
            let cb = |s: f32| {
                let (pp, pv) = FUN_00193a68(path, s, &inp.rope, hofs, inp.dt);
                FUN_0019a4a0(p, gz, v0, x.vel, pp, pv).cost
            };
            let s0 = path.FUN_001e7410(cb);
            let step = path.FUN_001e77e8();
            let lo = path.FUN_001e64c8(50.0);
            let hi = path.FUN_001e64c8(path.FUN_001e6648() - 50.0);
            let (s, mut cost) = FUN_0011d9e0(s0, step, lo, hi, cb);
            if !f {
                if cost >= DAT_002BF8E0 {
                    continue;
                }
                cost = FUN_0019b088(last, x.id, cost, DAT_002BF8E8);
                if best <= cost {
                    continue;
                }
            }
            let (pt, pv) = FUN_00193a68(path, s, &inp.rope, hofs, inp.dt);
            let r = traj(v0, x.vel, pt, pv, &mut aim_s, &mut av_s);
            if !f && clear(r.t, v0, aim_s, av_s, None, ClearFilter::Layer) {
                continue;
            }
            kind = 0xa;
            obj = x.id;
            best = best.min(cost);
            f27 = s;
            t_arr = r.t;
            aim_pt = aim_s;
            aim_vel = av_s;
            f26 = path.FUN_001e63c0(s);
        }
        // C) DAT_003e13b4 point -> kind 0xe
        for x in inp.targets {
            let TargetGeom::Point { pos, f_e8, is_13 } = &x.geom else { continue };
            let f = is_f(x.id);
            if !f && (forced.is_some() || !(*f_e8 && *is_13)) {
                continue;
            }
            if x.mask & mask == 0 {
                continue;
            }
            let pt = *pos;
            if !f {
                if p[2] - pt[2] > x.range {
                    continue;
                }
                let d = sub(pt, p);
                if dot(d, d) > C_1E6 {
                    continue;
                }
            }
            // cb 0x19abb8 with off = ZERO
            let cb = |T: f32| FUN_0019c300(p, v, g, inp.reach, T, add(pt, mul(x.vel, T))).0;
            let (T, mut cost) = FUN_0011d9e0(0.5, 0.25, 0.25, 3.0, cb);
            let (_, aim, av) = FUN_0019c300(p, v, g, inp.reach, T, add(pt, mul(x.vel, T)));
            if !f {
                if cost >= DAT_002BF8E0 {
                    continue;
                }
                cost = FUN_0019b088(last, x.id, cost, DAT_002BF8EC);
                if best <= cost {
                    continue;
                }
                if clear(T, v, aim, av, Some(pt), ClearFilter::Layer) {
                    continue;
                }
            }
            kind = 0xe;
            obj = x.id;
            aim_pt = add(aim, hofs);
            aim_vel = av;
            best = best.min(cost);
            t_arr = T;
        }
        // D) DAT_003e142c surface -> kind 0x10
        for x in inp.targets {
            let TargetGeom::Surface { mesh } = &x.geom else { continue };
            let f = is_f(x.id);
            let Some(mc) = FUN_00185310(mesh, p) else { continue };
            if !f {
                if p[2] - mc.cp[2] > x.range {
                    continue;
                }
                if len(sub(mc.cp, p)) > 1000.0 {
                    continue;
                }
            }
            // cb 0x19ac88
            let cb = |T: f32| {
                let (_, p1, v1) = FUN_00185d18(mesh, p, v0, g, inp.standoff, T);
                let (_, _, a0) = FUN_00117120(T, 0.0, 1.0, p, v0, p1, v1);
                let (_, _, a1) = FUN_00117120(T, T, 1.0, p, v0, p1, v1);
                ((len(a0) + len(a1)) * T * 0.5).min(C_1E6)
            };
            let (T, mut cost) = FUN_0011d9e0(1.0, 0.25, 0.25, 2.0, cb);
            if !f {
                if cost >= DAT_002BF8E0 {
                    continue;
                }
                cost = FUN_0019b088(last, x.id, cost, DAT_002BF908);
                if best <= cost {
                    continue;
                }
            }
            let (_tri, aim, av) = FUN_00185d18(mesh, p, v0, g, inp.standoff, T);
            if !f && clear(T, v, aim, av, None, ClearFilter::Layer) {
                continue;
            }
            kind = 0x10;
            obj = x.id;
            aim_pt = add(aim, hofs);
            aim_vel = av;
            best = best.min(cost);
            t_arr = T;
        }
    }
    // E) partner -> kind 0x12
    if inp.is_37 {
        if let Some(pa) = inp.partner.filter(|pa| pa.ready) {
            let f = is_f(pa.x);
            let r = traj(v0, pa.vel, pa.pos, ZERO, &mut aim_s, &mut av_s);
            let mut cost = r.cost;
            let ok = if f {
                true
            } else if cost >= DAT_002BF8E0 {
                false
            } else {
                // QUIRK: repeat test against o2, not X
                cost = FUN_0019b088(last, pa.o2, cost, DAT_002BF904);
                !(best <= cost) && !clear(r.t, v0, aim_s, av_s, None, ClearFilter::Partner(pa.o2))
            };
            if ok {
                kind = 0x12;
                obj = pa.x;
                aim_pt = aim_s;
                aim_vel = av_s;
                t_arr = r.t;
                best = best.min(cost);
            }
        }
    }
    // F) DAT_003e13c0 path on a moving body -> kind 0xf
    for x in inp.targets {
        let TargetGeom::PathBody { path } = &x.geom else { continue };
        if x.mask & mask == 0 {
            continue;
        }
        let f = is_f(x.id);
        // FEEL R98: 0x19abf0 / 0x19c300 run on p = FUN_001a62e8 (the xform translation = port pos + so_ofs); the aim
        // is that xform's target, so it goes back to the port pos frame (- so_ofs). Capture cap_feel_r97_pipe_hop_fixed:
        // pick lq 50, T 0.325, xf aim (-1749.2,-3897.1,568.9) = ballistic(T) from the xform. Opt-out SLY2_FEEL_NO_R98PICKSO.
        let so = if std::env::var_os("SLY2_FEEL_NO_R98PICKSO").is_some() { ZERO } else { inp.so_ofs };
        let p = add(p, so);
        let pc = path.FUN_001e6f60(p);
        if !f {
            if p[2] - pc.cp[2] > x.range {
                continue;
            }
            if len(sub(pc.cp, p)) > 1000.0 {
                continue;
            }
        }
        let inner = |wp: V3| {
            FUN_0011d9e0(0.5, 0.25, 0.25, 3.0, |T| FUN_0019c300(p, v, g, inp.reach, T, add(wp, mul(x.vel, T))).0)
        };
        // cb 0x19abf0
        let (lq, mut cost) =
            FUN_0011d9e0(pc.l, 100.0, 50.0, path.FUN_001e6648() - 50.0, |l| inner(path.FUN_001e6f10(l).0).1);
        if !f {
            if cost >= DAT_002BF8E0 {
                continue;
            }
            cost = FUN_0019b088(last, x.id, cost, DAT_002BF8F0);
            if best <= cost {
                continue;
            }
        }
        let wp = path.FUN_001e6f10(lq).0;
        let (T, _) = inner(wp);
        let (_, aim, av) = FUN_0019c300(p, v, g, inp.reach, T, add(wp, mul(x.vel, T)));
        let aim = sub(aim, so);
        if !f && clear(T, v, aim, av, Some(wp), ClearFilter::Layer) {
            continue;
        }
        if std::env::var_os("SLY2_K22DBG").is_some() { eprintln!("R93DBG 0xf pc.l {} lq {lq} wp {wp:?} T {T} aim {aim:?} hofs {hofs:?} p {p:?} v {v:?} reach {}", pc.l, inp.reach); }
        kind = 0xf;
        obj = x.id;
        aim_pt = add(aim, hofs);
        aim_vel = av;
        best = best.min(cost);
        t_arr = T;
        f26 = lq;
    }
    // G) DAT_003e13fc -> kind 0xc
    for x in inp.targets {
        let TargetGeom::Pole { pos, scale, member } = &x.geom else { continue };
        if x.mask & mask == 0 {
            continue;
        }
        let f = is_f(x.id);
        let pt = *pos;
        if p[2] - pt[2] > x.range {
            continue; // no forced bypass
        }
        let d = sub(p, pt);
        let u0 = d[1].atan2(d[0]) * scale;
        // cb 0x19aab8
        let cb = |u: f32| FUN_0019a4a0(p, gz, v, x.vel, member_pt(member, u, p, *scale, None), ZERO).cost;
        let (u, cost0) = FUN_0011d9e0(u0, DAT_002BF914, -f32::MAX, f32::MAX, cb);
        let mut cost = cost0 * DAT_002BF90C;
        if !f {
            if cost >= DAT_002BF8E0 {
                continue;
            }
            cost = FUN_0019b088(last, x.id, cost, DAT_002BF8F8);
            if best <= cost {
                continue;
            }
        }
        let pt2 = member_pt(member, u, p, *scale, None);
        let r = traj(v, x.vel, pt2, ZERO, &mut aim_s, &mut av_s);
        if !f && clear(r.t, v, aim_s, av_s, None, ClearFilter::Layer) {
            continue;
        }
        kind = 0xc;
        obj = x.id;
        best = best.min(cost);
        t_arr = r.t;
        aim_pt = aim_s;
        aim_vel = av_s;
        f26 = u;
    }
    // H) DAT_003e13f0 -> kind 0xb
    for x in inp.targets {
        let TargetGeom::Rail { path, member } = &x.geom else { continue };
        if x.mask & mask == 0 {
            continue;
        }
        let f = is_f(x.id);
        let pc = path.FUN_001e6f60(p);
        if p[2] - pc.cp[2] > x.range {
            continue; // no forced bypass
        }
        // cb 0x19ab38
        let cb = |u: f32| FUN_0019a4a0(p, gz, v, x.vel, member_pt(member, u, p, 1.0, Some(path)), ZERO).cost;
        let (u, cost0) = FUN_0011d9e0(pc.l, DAT_002BF918, 30.0, path.FUN_001e6648() - 30.0, cb);
        let mut cost = cost0 * DAT_002BF90C;
        if !f {
            if cost >= DAT_002BF8E0 {
                continue;
            }
            cost = FUN_0019b088(last, x.id, cost, DAT_002BF8F4);
            if best <= cost {
                continue;
            }
        }
        let pt = member_pt(member, u, p, 1.0, Some(path));
        let r = traj(v, x.vel, pt, ZERO, &mut aim_s, &mut av_s);
        if !f && clear(r.t, v, aim_s, av_s, None, ClearFilter::Layer) {
            continue;
        }
        kind = 0xb;
        obj = x.id;
        best = best.min(cost);
        t_arr = r.t;
        aim_pt = aim_s;
        aim_vel = av_s;
        f26 = u;
    }
    // I) DAT_003e1408 -> kind 0xd
    for x in inp.targets {
        let TargetGeom::Spot0xd { pt } = &x.geom else { continue };
        if x.mask & mask == 0 {
            continue;
        }
        let f = is_f(x.id);
        if !f && p[2] - pt[2] > x.range {
            continue;
        }
        let r = traj(v, x.vel, *pt, ZERO, &mut aim_s, &mut av_s);
        let mut cost = r.cost;
        if !f {
            if cost >= DAT_002BF8E0 {
                continue;
            }
            cost = FUN_0019b088(last, x.id, cost, DAT_002BF8FC);
            if best <= cost {
                continue;
            }
            if clear(r.t, v, aim_s, av_s, None, ClearFilter::Layer) {
                continue;
            }
        }
        kind = 0xd;
        obj = x.id;
        best = best.min(cost);
        t_arr = r.t;
        aim_pt = aim_s;
        aim_vel = av_s;
    }
    if kind == -1 {
        return None;
    }
    Some(PickOut {
        kind,
        obj,
        aim: sub(aim_pt, hofs),
        aim_vel,
        deadline: inp.t + t_arr,
        t_arrive: t_arr,
        f_10: f27,
        f_14: f26,
        cost: best,
    })
}

// ---------------------------------------------------------------- JMP body state / flight
/// JT body xf fields used by the flight (jt+0x58).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FlightXf {
    /// +0x98 motion kind (2 dynamic, 0x11 Hermite drive)
    pub kind: u16,
    /// +0x30 position, +0xb0 velocity (parent frame = world here)
    pub pos: V3,
    pub vel: V3,
    /// +0xa8 deadline, +0xc0 target point, +0xd0 target velocity
    pub deadline: f32,
    pub aim: V3,
    pub aim_vel: V3,
}

/// JMP (jt+0x2c0) fields touched by the flight.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct JmpFlight {
    /// +0x1c jtbs, +0xc target, +0x20 time entered, +0x4 kind, +0x8 idx, +0x28 double-jump flag, +0xbc last target
    pub jtbs: i32,
    pub target: Option<usize>,
    pub t_jtbs: f32,
    pub kind: i32,
    pub idx: i32,
    pub f_28: bool,
    pub last: Option<usize>,
    /// +0x10 / +0x14
    pub f_10: f32,
    pub f_14: f32,
    pub xf: FlightXf,
}
impl Default for JmpFlight {
    fn default() -> Self {
        JmpFlight { jtbs: -1, target: None, t_jtbs: 0.0, kind: -1, idx: 0, f_28: false, last: None, f_10: 0.0, f_14: 0.0, xf: FlightXf { kind: 2, ..Default::default() } }
    }
}

/// Side effects of FUN_00195fb8 for the lead to apply.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Fx {
    /// FUN_002303a0(jt, xf, kind) (also stored in `JmpFlight::xf.kind`)
    XfKind(u16),
    /// jt+0x190.xyz = 0
    Clear190,
    /// jt vt+0x11c SetVelocity (also written to `xf.vel`)
    SetVelocity(V3),
    /// ASEG start (tail 0x196840): `field` = JMP offset (jmp_field) or JT offset; `force` = s1 cleared;
    /// start/speed = f22/f23; `half_len` = f22 = *(aseg+0x74) * 0.5 (non-player 0xe/0xf)
    Anim { field: u32, jmp_field: bool, force: bool, start: f32, speed: f32, half_len: bool },
    /// unported call (address, arg); target ids via `CallObj`
    Call(u32, i32),
    CallObj(u32, usize),
    /// exit / entry of a state not handled here (old / new jtbs)
    ExitUnported(i32),
    EntryUnported(i32),
}

/// 0x00195fb8(jmp, jtbs, target) for the flight states: exits of 8, 9..0x10, 0x12; common part; entries of -1,
/// 9..0x10, 0x12 (others reported as `EntryUnported` after the common writes). `vel` = jt vt+0x10c current
/// velocity (for the kind-4 launch of 9/0xa/0x10/0x12), `gz` = gravity z, `is_player` = jt == DAT_002e1e40.
pub fn FUN_00195fb8(st: &mut JmpFlight, new: i32, target: Option<usize>, vel: V3, gz: f32, t: f32, is_player: bool) -> Vec<Fx> {
    let mut fx = Vec::new();
    if new == st.jtbs {
        return fx;
    }
    // 1a exit
    let old = st.jtbs;
    match old {
        8..=0x10 | 0x12 => {
            st.xf.kind = 2;
            fx.push(Fx::XfKind(2));
            fx.push(Fx::Clear190);
            if old == 0xe || old == 0xf {
                fx.push(if is_player { Fx::Call(0x0018e540, -1) } else { Fx::Call(0x001ccac0, -1) });
            }
        }
        // BOSS R22: exit 6 (barrel flight): jmp+0xc = 0; unless worn (jt+0x4c8 == 1, checked by the lead)
        // FUN_00134320 + (barrel +0x17c == 1) FUN_00183b80(barrel, 0); xf kind 2, jt+0x190 = 0
        6 => {
            fx.push(Fx::Call(0x0018_3b80, 0));
            st.xf.kind = 2;
            fx.push(Fx::XfKind(2));
            fx.push(Fx::Clear190);
        }
        -1..=5 => {}
        _ => fx.push(Fx::ExitUnported(old)),
    }
    // 1b common
    st.jtbs = new;
    st.target = target;
    st.t_jtbs = t;
    let mut force = false;
    let start = 0.0f32; // f22
    let mut speed = 1.0f32;
    let mut half_len = false;
    let mut field: Option<(u32, bool)> = None;
    match new {
        -1 => {
            // *(jmp+0x174) ledge-hang cleanup (FUN_00204da8 / FUN_00140c88) belongs to jtbs 0x11 (mod.rs)
            st.last = None;
            st.kind = -1;
        }
        // BOSS R22: entry 6 (barrel flight): xf kind 0x11, FUN_00134320, FUN_00183b80(barrel, 1), jt+0x190 = 0
        // (ASEG *(jt+0x1030)+0x14 not modelled)
        6 => {
            st.xf.kind = 0x11;
            fx.push(Fx::XfKind(0x11));
            fx.push(Fx::Call(0x0018_3b80, 1));
            fx.push(Fx::Clear190);
        }
        9 | 0xa | 0x10 | 0x12 => {
            st.xf.kind = 0x11;
            fx.push(Fx::XfKind(0x11));
            fx.push(Fx::Clear190);
            st.f_28 = false;
            let v = [vel[0], vel[1], FUN_0019a750(4, vel[2], gz, 0.0)];
            st.xf.vel = v;
            fx.push(Fx::SetVelocity(v));
            st.idx = 0;
            field = Some((if new == 0x10 { 0x14c } else { 0x148 }, true));
            force = true;
        }
        0xb..=0xf => {
            st.xf.kind = 0x11;
            fx.push(Fx::XfKind(0x11));
            fx.push(Fx::Clear190);
            st.f_28 = false;
            st.idx = 0;
            field = Some((0x120, true));
            if new == 0xe || new == 0xf {
                if let Some(o) = target {
                    if is_player {
                        fx.push(Fx::CallObj(0x1854, o)); // jt+0x1854 = target
                        fx.push(Fx::Call(0x0018e540, 0));
                    } else {
                        fx.push(Fx::CallObj(0x984, o)); // jt+0x984 = target
                        fx.push(Fx::Call(0x001ccac0, 0));
                        field = Some((0x930, false));
                        half_len = true;
                        speed = 0.0;
                    }
                }
            }
        }
        _ => fx.push(Fx::EntryUnported(new)),
    }
    // 1d tail
    if new == 0x12 {
        fx.push(Fx::Call(0x00134320, 0)); // FUN_00134320(0x3dd490, jt)
    }
    if new == -1 {
        return fx;
    }
    if let Some((f, j)) = field {
        fx.push(Fx::Anim { field: f, jmp_field: j, force, start, speed, half_len });
    }
    fx.push(Fx::Call(0x2d4, 0)); // jt vt+0x2d4
    fx
}

/// 0x0019b110 tail: write xf+0xa8/+0xc0/+0xd0, jmp+0x10/+0x14, then FUN_00195fb8(jmp, kind, obj).
pub fn apply_pick(st: &mut JmpFlight, out: &PickOut, vel: V3, gz: f32, t: f32, is_player: bool) -> Vec<Fx> {
    st.xf.deadline = out.deadline;
    st.xf.aim = out.aim;
    st.xf.aim_vel = out.aim_vel;
    st.f_10 = out.f_10;
    st.f_14 = out.f_14;
    FUN_00195fb8(st, out.kind, Some(out.obj), vel, gz, t, is_player)
}

/// FUN_0022de88 case 0x11 (0x22f13c): one physics step of the Hermite drive. `t` = clock after this step,
/// `dt` = step. Re-plans from the current (pos, vel) each step; snaps to aim with zero velocity once
/// deadline <= t. Returns (pos, vel) (also stored in `xf`).
pub fn FUN_0022de88_case11(xf: &mut FlightXf, t: f32, dt: f32) -> (V3, V3) {
    if xf.deadline <= t {
        xf.pos = xf.aim;
        xf.vel = ZERO;
    } else {
        let (p, v, _) = FUN_00117120((xf.deadline - t) + dt, dt, 1.0, xf.pos, xf.vel, xf.aim, xf.aim_vel);
        xf.pos = p;
        xf.vel = v;
    }
    (xf.pos, xf.vel)
}

/// 0x00197f98 abort test (0x19878c) for jtbs 6, 8, 9..0x10, 0x12: true -> FUN_00195fb8(jmp, 4, None).
pub fn FUN_00197f98_abort(xf: &FlightXf, t: f32, dt: f32) -> bool {
    let T = (xf.deadline - t) + dt;
    let (_, _, a0) = FUN_00117120(T, 0.0, 1.0, xf.pos, xf.vel, xf.aim, xf.aim_vel);
    let (_, _, a1) = FUN_00117120(T, T, 1.0, xf.pos, xf.vel, xf.aim, xf.aim_vel);
    (len(a0) + len(a1)) * T * 0.5 > DAT_002BF844
}

/// 0x00197500 jtbs 9..0x10: arrival when xf+0xa8 < t (then Jt::FUN_0019c688).
pub fn arrived(xf: &FlightXf, t: f32) -> bool {
    xf.deadline < t
}

// ---------------------------------------------------------------- target list helpers
/// 0x30 B target list entry as used by 0x194f70 / 0x195368.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TgtSlot {
    /// +0 obj, +4 member, +8 dist, +0xc flag/prio, +0x10 indicator object, +0x18 u, +0x1c du, +0x20 pos
    pub obj: usize,
    pub member: usize,
    pub dist: f32,
    pub flag: i32,
    pub ind: Option<usize>,
    pub u: f32,
    pub du: f32,
    pub pos: V3,
    /// *(member+0x14)
    pub member_14: i32,
    /// member vt+0xc: Some((lo, hi)) when the parameter wraps
    pub wrap: Option<(f32, f32)>,
}

/// Indicator side effects of 0x194f70.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IndFx {
    /// FUN_001f7bf8(0.5, ind) fade out
    FadeOut(usize, f32),
    /// ind vt+0xec(pos)
    Place(usize, V3),
    /// FUN_001619a0(0x2deb30, 0x3d, obj) newly targeted
    Event3d(usize),
}

/// 0x00194f70(W, jt, n, list): merge the fresh list into the persistent list `w` (W+0x4678/+0x4680), smoothing
/// indicator parameters. `templ` = jt+0x9c4, `spawn` = FUN_001956d0, `member_pt` = member vt+0x1c(u),
/// `smooth` = DAT_002e1e50. Note: no capacity check on W's 4 slots in the original (Vec here).
pub fn FUN_00194f70(
    w: &mut Vec<TgtSlot>,
    list: &mut Vec<TgtSlot>,
    dt: f32,
    smooth: bool,
    templ: Option<usize>,
    spawn: &mut dyn FnMut(&TgtSlot) -> Option<usize>,
    member_pt: &dyn Fn(&TgtSlot, f32) -> V3,
) -> Vec<IndFx> {
    let mut fx = Vec::new();
    // qsort cmp 0x194f48: a.dist < b.dist ? -1 : 1 (stable sort here)
    let by_dist = |a: &TgtSlot, b: &TgtSlot| if a.dist < b.dist { std::cmp::Ordering::Less } else if b.dist < a.dist { std::cmp::Ordering::Greater } else { std::cmp::Ordering::Equal };
    if !list.is_empty() {
        list.sort_by(by_dist);
        if list[0].member_14 != -1 {
            list[0].flag = 1;
        }
    }
    let mut i = w.len();
    while i > 0 {
        i -= 1;
        let Some(j) = list.iter().rposition(|k| k.obj == w[i].obj) else {
            if let Some(ind) = w[i].ind {
                fx.push(IndFx::FadeOut(ind, 0.5));
            }
            w.remove(i);
            continue;
        };
        let mut nj = list[j];
        let e = &mut w[i];
        e.dist = nj.dist;
        match e.ind {
            None => {
                e.u = nj.u;
                e.du = 0.0;
            }
            Some(ind) => {
                let mut wrap = nj.wrap;
                if let Some((lo, hi)) = wrap {
                    let per = hi - lo;
                    let d = nj.u - e.u;
                    if d > 0.5 * per {
                        nj.u = e.u + (d - per);
                    } else if d < -0.5 * per {
                        nj.u = e.u + (d + per);
                    } else {
                        wrap = None;
                    }
                }
                if smooth {
                    let dv = e.du + dt * ((nj.u - e.u) * DAT_002BF710 + (nj.du - e.du) * DAT_002BF714);
                    e.u += dt * (e.du + dv) * 0.5;
                    e.du = dv;
                } else {
                    let mut du = e.du;
                    e.u = FUN_0011d018(e.u, nj.u, dt, DAT_002BF718, Some(&mut du));
                    e.du = du;
                }
                if let Some((lo, hi)) = wrap {
                    let per = hi - lo;
                    if hi < e.u {
                        e.u -= per;
                    } else if e.u < lo {
                        e.u += per;
                    }
                }
                let p = member_pt(e, e.u);
                fx.push(IndFx::Place(ind, p));
            }
        }
        list.remove(j);
    }
    for k in list.iter_mut() {
        if templ.is_some() && k.flag == 1 {
            match spawn(k) {
                Some(ind) => k.ind = Some(ind),
                None => break,
            }
        }
        w.push(*k);
        fx.push(IndFx::Event3d(k.obj));
    }
    w.sort_by(by_dist);
    fx
}

/// Sparkle emitter jt+0x12f8 (+0xe0 armed, +0xf0 / +0xf8 / +0xfc, +0x130 timer) and its params *(E+0xe4).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Emitter {
    pub armed_e0: bool,
    pub f_f0: i32,
    pub f_f8: f32,
    /// +0xfc rate (particles / s)
    pub rate_fc: f32,
    pub t_130: f32,
    /// prm +0x50/+0x54 speed, +0xa0/+0xa4 age, +0x8c life, +0x98/+0x9c advance
    pub spd: (f32, f32),
    pub age: (f32, f32),
    pub life: f32,
    pub adv: (f32, f32),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Particle {
    pub pos: V3,
    pub vel: V3,
    pub birth: f32,
    pub death: f32,
}

/// 0x00195368(W, jt, n, list): target sparkles; at most 10 particles (then emits and stops). `member_pd` =
/// member vt+0x2c(u) -> (pos, dir). Returns the particles passed to FUN_0015b428.
pub fn FUN_00195368(
    em: Option<&mut Emitter>,
    list: &[TgtSlot],
    t: f32,
    dt: f32,
    rng: &mut GameRng,
    member_pd: &dyn Fn(&TgtSlot, f32) -> (V3, V3),
) -> Vec<Particle> {
    let mut out = Vec::new();
    let Some(e) = em else { return out };
    if e.armed_e0 {
        // FUN_0015a1c0 re-arm (partial: +0xe0 = 0, +0x130 timer)
        e.armed_e0 = false;
        let r = rng.FUN_0011d448(e.f_f8, e.rate_fc);
        e.t_130 = if e.f_f0 == 3 { r * 60.0 } else { r };
    }
    'emit: for s in list {
        if s.flag == -1 {
            continue;
        }
        let f = e.rate_fc * dt;
        let mut k = f as i32;
        if rng.FUN_0011d448(0.0, 1.0) < f - k as f32 {
            k += 1;
        }
        for _ in 0..k {
            let (mut pos, dir) = member_pd(s, s.u);
            let vel = mul(dir, rng.FUN_0011d448(e.spd.0, e.spd.1));
            let birth = t - rng.FUN_0011d448(e.age.0, e.age.1);
            let death = birth + e.life;
            if e.adv.0 != e.adv.1 {
                pos = add(pos, mul(vel, rng.FUN_0011d448(e.adv.0, e.adv.1)));
            }
            out.push(Particle { pos, vel, birth, death });
            if out.len() >= 10 {
                break 'emit;
            }
        }
    }
    out
}

// ---------------------------------------------------------------- tests
#[cfg(test)]
mod tests {
    use super::*;

    const G: V3 = [0.0, 0.0, -2000.0];

    fn base<'a>(targets: &'a [TargetObj], pos: V3, vel: V3) -> PickIn<'a> {
        PickIn {
            t: 10.0,
            dt: 1.0 / 60.0,
            pos,
            vel,
            centre: pos,
            hofs: [0.0, 0.0, 0.0],
            so_ofs: [0.0, 0.0, 0.0],
            g: G,
            reach: 30.0,
            standoff: 20.0,
            mask: 0xffff_ffff,
            is_35: true,
            is_37: false,
            last: None,
            rope: RopeIn { a2a0: 300.0, a260: 0.0, joy_angle: 0.0, joy_mag: 0.0 },
            partner: None,
            targets,
        }
    }

    fn close(a: V3, b: V3, eps: f32) -> bool {
        (0..3).all(|i| (a[i] - b[i]).abs() <= eps)
    }

    #[test]
    fn hermite_endpoints() {
        let p0 = [0.0, 0.0, 0.0];
        let p1 = [100.0, 50.0, -20.0];
        let v0 = [10.0, 0.0, 300.0];
        let v1 = [0.0, -5.0, -100.0];
        let (p, v, _) = FUN_00117120(2.0, 0.0, 1.0, p0, v0, p1, v1);
        assert!(close(p, p0, 1e-4) && close(v, v0, 1e-3));
        let (p, v, _) = FUN_00117120(2.0, 2.0, 1.0, p0, v0, p1, v1);
        assert!(close(p, p1, 1e-3) && close(v, v1, 1e-3));
        // constant-acceleration flight: p1 = p0 + v0 T + g T^2/2, v1 = v0 + g T -> acc = g everywhere
        let T = 0.5;
        let p1 = add(add(p0, mul(v0, T)), mul(G, T * T * 0.5));
        let v1 = add(v0, mul(G, T));
        let (_, _, a) = FUN_00117120(T, 0.2, 1.0, p0, v0, p1, v1);
        assert!(close(a, G, 0.5));
    }

    #[test]
    fn golden_search_parabola() {
        let (x, f) = FUN_0011d9e0(0.5, 0.25, 0.25, 3.0, |x| (x - 1.3) * (x - 1.3) + 2.0);
        assert!((x - 1.3).abs() < 0.01, "{x}");
        assert!((f - 2.0).abs() < 1e-3);
        // clamped at hi
        let (x, _) = FUN_0011d9e0(0.5, 0.25, 0.25, 3.0, |x| -x);
        assert!((x - 3.0).abs() < 1e-4);
    }

    #[test]
    fn traj_ballistic() {
        // falling onto a point 100 below, v.z = 0: 0.5*2000 t^2 = 100 -> t = sqrt(0.1)
        let r = FUN_0019a4a0([0.0; 3], -2000.0, [0.0; 3], ZERO, [0.0, 0.0, -100.0], ZERO);
        assert!((r.t - 0.1f32.sqrt()).abs() < 1e-4);
        assert!(r.cost.abs() < 1e-2, "{}", r.cost); // straight drop: no horizontal accel
        // unreachable (above apex): no root -> 1.01e6 - ...
        let r = FUN_0019a4a0([0.0; 3], -2000.0, [0.0; 3], ZERO, [0.0, 0.0, 100.0], ZERO);
        assert!(r.aim.is_none() && r.cost > 1.0e6 && r.t == 0.0);
    }

    #[test]
    fn pick_point_in_reach() {
        // point 150 ahead, 50 below; JT moving towards it
        let tg = [TargetObj { id: 7, mask: 1, range: 500.0, vel: ZERO, geom: TargetGeom::Point { pos: [150.0, 0.0, -50.0], f_e8: true, is_13: true } }];
        let inp = base(&tg, [0.0; 3], [300.0, 0.0, 200.0]);
        let out = FUN_0019b110(&inp, None, &OpenWorld).expect("pick");
        assert_eq!((out.kind, out.obj), (0xe, 7));
        assert!(out.t_arrive >= 0.25 && out.t_arrive <= 3.0);
        assert!(out.cost < DAT_002BF8E0);
        // aim lands within reach of the point (0x19c300 contract)
        assert!(len(sub(out.aim, [150.0, 0.0, -50.0])) <= 30.0 + 1e-2, "{:?}", out.aim);
        assert!((out.deadline - (10.0 + out.t_arrive)).abs() < 1e-5);
        // repeat penalty does not change an uncontested pick; mask mismatch rejects
        let mut inp2 = base(&tg, [0.0; 3], [300.0, 0.0, 200.0]);
        inp2.mask = 2;
        assert!(FUN_0019b110(&inp2, None, &OpenWorld).is_none());
        // too far (> 1000) rejected unless forced
        let far = [TargetObj { id: 8, mask: 1, range: 500.0, vel: ZERO, geom: TargetGeom::Point { pos: [1500.0, 0.0, 0.0], f_e8: true, is_13: true } }];
        let inp3 = base(&far, [0.0; 3], [0.0; 3]);
        assert!(FUN_0019b110(&inp3, None, &OpenWorld).is_none());
        assert_eq!(FUN_0019b110(&inp3, Some(8), &OpenWorld).map(|o| o.kind), Some(0xe));
    }

    #[test]
    fn pick_point_in_reach_exact() {
        // hand-checked 0x19c300: P=0, V=0, g=(0,0,-2000), T=0.5 -> A=(0,0,-1000), B=(0,0,-250);
        // target (0,0,-250) is inside R=30 -> cost = -30, aim = B
        let (c, aim, av) = FUN_0019c300([0.0; 3], [0.0; 3], G, 30.0, 0.5, [0.0, 0.0, -250.0]);
        assert_eq!((c, aim, av), (-30.0, [0.0, 0.0, -250.0], [0.0, 0.0, -1000.0]));
        // target 130 to the side at the same height: e = 130 - 30 = 100, k = 2e/T^2 = 800, A' = (400,0,-1000),
        // cost = 4*100/0.5 = 800, aim = (100, 0, -250)
        let (c, aim, av) = FUN_0019c300([0.0; 3], [0.0; 3], G, 30.0, 0.5, [130.0, 0.0, -250.0]);
        assert!((c - 800.0).abs() < 1e-3);
        assert!(close(av, [400.0, 0.0, -1000.0], 1e-3) && close(aim, [100.0, 0.0, -250.0], 1e-3));
    }

    #[test]
    fn pick_pole() {
        let pole = MemberGeom::Pole { a: [120.0, 0.0, -300.0], b: [120.0, 0.0, 300.0], radius: 20.0 };
        let tg = [TargetObj { id: 3, mask: 0x10, range: 400.0, vel: ZERO, geom: TargetGeom::Pole { pos: [120.0, 0.0, 0.0], scale: 20.0, member: pole } }];
        let inp = base(&tg, [0.0, 0.0, 0.0], [400.0, 0.0, 300.0]);
        let out = FUN_0019b110(&inp, None, &OpenWorld).expect("pick");
        assert_eq!((out.kind, out.obj), (0xc, 3));
        // the grab point faces the JT: angle ~ pi -> x ~ 100
        let ang = out.f_14 / 20.0;
        assert!((ang.cos() + 1.0).abs() < 0.05, "u {}", out.f_14);
        assert!((out.aim[0] - 100.0).abs() < 1.0, "{:?}", out.aim);
        // aimVel.z = V.z + g t
        assert!((out.aim_vel[2] - (300.0 - 2000.0 * out.t_arrive)).abs() < 1e-2);
        // the pole is lost when it is out of the drop range (range test even when forced)
        let mut tg2 = tg.clone();
        tg2[0].range = 10.0;
        let inp2 = base(&tg2, [0.0, 0.0, 200.0], [400.0, 0.0, 300.0]);
        assert!(FUN_0019b110(&inp2, Some(3), &OpenWorld).is_none());
    }

    #[test]
    fn pick_rope_segment() {
        // horizontal rope along y at x = 200, z = -40 (hanging point); JT jumps along +x
        let path = Path::polyline(vec![[200.0, -300.0, -40.0], [200.0, 0.0, -40.0], [200.0, 300.0, -40.0]], M4_IDENT);
        assert_eq!(path.FUN_001e6648(), 600.0);
        assert!((path.FUN_001e64c8(50.0) - 50.0).abs() < 1e-4);
        assert!((path.FUN_001e63c0(350.0) - 350.0).abs() < 1e-4);
        let (p, d) = path.FUN_001e67b8(450.0);
        assert!(close(p, [200.0, 150.0, -40.0], 1e-3) && close(d, [0.0, 1.0, 0.0], 1e-5));
        let tg = [TargetObj { id: 5, mask: 0x200, range: 500.0, vel: ZERO, geom: TargetGeom::Rope { path } }];
        let inp = base(&tg, [0.0, 0.0, 0.0], [400.0, 0.0, 300.0]);
        let out = FUN_0019b110(&inp, None, &OpenWorld).expect("pick");
        assert_eq!((out.kind, out.obj), (0xa, 5));
        // nearest rope point straight ahead: s ~ 300 (y = 0); f_14 = arc length (polyline: == s)
        assert!((out.f_10 - 300.0).abs() < 5.0, "s {}", out.f_10);
        assert!((out.f_14 - out.f_10).abs() < 1e-3);
        assert!(close(out.aim, [200.0, out.aim[1], -40.0], 1e-2));
    }

    #[test]
    fn rope_point_velocity_limits() {
        let path = Path::polyline(vec![[0.0, 0.0, 0.0], [600.0, 0.0, 0.0]], M4_IDENT);
        let ri = RopeIn { a2a0: 300.0, a260: 0.0, joy_angle: 0.0, joy_mag: 1.0 };
        // stick along the rope: w = 300; far from the ends the approach speeds are -vmax / +vmax -> r = 300
        let (p, v) = FUN_00193a68(&path, 300.0, &ri, [0.0, 0.0, 10.0], 1.0 / 60.0);
        assert!(close(p, [300.0, 0.0, -10.0], 1e-4));
        assert!(close(v, [300.0, 0.0, 0.0], 1e-2), "{v:?}");
    }

    #[test]
    fn pick_surface() {
        // wall plane x = 200 (normal towards -x)
        let mesh = Mesh {
            verts: vec![[200.0, -500.0, -500.0], [200.0, 500.0, -500.0], [200.0, 0.0, 500.0]],
            tris: vec![[0, 2, 1]],
            xf: M4_IDENT,
        };
        let mc = FUN_00185310(&mesh, [0.0, 10.0, 0.0]).unwrap();
        assert!(close(mc.cp, [200.0, 10.0, 0.0], 1e-3));
        assert!(close(mc.n, [-1.0, 0.0, 0.0], 1e-5) || close(mc.n, [1.0, 0.0, 0.0], 1e-5));
        let tg = [TargetObj { id: 9, mask: 0, range: 500.0, vel: ZERO, geom: TargetGeom::Surface { mesh } }];
        let inp = base(&tg, [0.0, 0.0, 0.0], [400.0, 0.0, 300.0]);
        let out = FUN_0019b110(&inp, None, &OpenWorld).expect("pick");
        assert_eq!(out.kind, 0x10);
        assert!((out.aim[0] - (200.0 + 20.0 * mc.n[0])).abs() < 1e-2);
    }

    #[test]
    fn rejects_blocked_and_prefers_cheaper() {
        struct Wall;
        impl ClearWorld for Wall {
            fn seg_hit(&self, a: V3, b: V3, _: ClearFilter) -> Option<(V3, f32)> {
                // plane x = 50
                if (a[0] - 50.0) * (b[0] - 50.0) < 0.0 {
                    let f = (50.0 - a[0]) / (b[0] - a[0]);
                    Some((add(a, mul(sub(b, a), f)), f))
                } else {
                    None
                }
            }
        }
        let tg = [TargetObj { id: 7, mask: 1, range: 500.0, vel: ZERO, geom: TargetGeom::Point { pos: [150.0, 0.0, -50.0], f_e8: true, is_13: true } }];
        let inp = base(&tg, [0.0; 3], [300.0, 0.0, 200.0]);
        assert!(FUN_0019b110(&inp, None, &Wall).is_none());
        // two spots: A (bias -250) beats I (bias -250, same cost) by order; repeat penalty flips it
        let tg2 = [
            TargetObj { id: 1, mask: 1, range: 500.0, vel: ZERO, geom: TargetGeom::Spot9 { pt: [100.0, 0.0, -100.0] } },
            TargetObj { id: 2, mask: 1, range: 500.0, vel: ZERO, geom: TargetGeom::Spot9 { pt: [100.0, 0.0, -100.0] } },
        ];
        let mut inp2 = base(&tg2, [0.0; 3], [200.0, 0.0, 0.0]);
        assert_eq!(FUN_0019b110(&inp2, None, &OpenWorld).unwrap().obj, 1);
        inp2.last = Some(1);
        assert_eq!(FUN_0019b110(&inp2, None, &OpenWorld).unwrap().obj, 2);
    }

    #[test]
    fn flight_reaches_aim_at_deadline() {
        let tg = [TargetObj { id: 7, mask: 1, range: 500.0, vel: ZERO, geom: TargetGeom::Point { pos: [150.0, 0.0, -50.0], f_e8: true, is_13: true } }];
        let p0 = [0.0, 0.0, 0.0];
        let v_cur = [300.0, 0.0, 200.0];
        let inp = base(&tg, p0, v_cur);
        let out = FUN_0019b110(&inp, None, &OpenWorld).unwrap();
        let mut st = JmpFlight::default();
        st.jtbs = 4;
        st.xf.pos = p0;
        st.xf.vel = v_cur;
        let fx = apply_pick(&mut st, &out, v_cur, G[2], inp.t, true);
        assert_eq!(st.jtbs, 0xe);
        assert_eq!(st.xf.kind, 0x11);
        assert!(fx.contains(&Fx::XfKind(0x11)));
        assert!(fx.iter().any(|f| matches!(f, Fx::Anim { field: 0x120, jmp_field: true, .. })));
        // reference: single cubic from the entry state
        let (T0, s0, v0) = (out.deadline - inp.t, st.xf.pos, st.xf.vel);
        let dt = 1.0 / 60.0;
        let mut t = inp.t;
        let mut n = 0;
        while !arrived(&st.xf, t) {
            t += dt;
            n += 1;
            let (p, _) = FUN_0022de88_case11(&mut st.xf, t, dt);
            assert!(!FUN_00197f98_abort(&st.xf, t, dt) || t >= out.deadline);
            if t < out.deadline {
                let (r, _, _) = FUN_00117120(T0, t - inp.t, 1.0, s0, v0, out.aim, out.aim_vel);
                assert!(close(p, r, 0.05), "step {n}: {p:?} vs {r:?}");
            } else {
                assert_eq!(p, out.aim);
            }
            assert!(n < 1000);
        }
        assert!(t >= out.deadline && t - dt < out.deadline + 1e-4);
        assert_eq!(st.xf.pos, out.aim);
        // leaving the flight puts the body back to dynamic
        let fx = FUN_00195fb8(&mut st, 4, None, ZERO, G[2], t, true);
        assert_eq!(st.xf.kind, 2);
        assert!(fx.contains(&Fx::Call(0x0018e540, -1)));
    }

    #[test]
    fn launch_entry_sets_velocity() {
        let mut st = JmpFlight::default();
        st.jtbs = 4;
        let fx = FUN_00195fb8(&mut st, 0xa, Some(5), [100.0, 0.0, -200.0], G[2], 1.0, true);
        // kind 4: z = max(min(700 + 0.5 * -200, 700), -200) = 600
        assert_eq!(st.xf.vel, [100.0, 0.0, 600.0]);
        assert!(fx.contains(&Fx::SetVelocity([100.0, 0.0, 600.0])));
        assert!(fx.iter().any(|f| matches!(f, Fx::Anim { field: 0x148, force: true, .. })));
        assert_eq!((st.target, st.t_jtbs), (Some(5), 1.0));
        assert!(FUN_00195fb8(&mut st, 0xa, None, ZERO, G[2], 2.0, true).is_empty());
    }

    #[test]
    fn spline_path_arc_table() {
        // straight spline with tangents = segment vector: arc length equals chord
        let pts = vec![[0.0, 0.0, 0.0], [100.0, 0.0, 0.0], [200.0, 0.0, 0.0]];
        let tan = vec![[100.0, 0.0, 0.0]; 3];
        let path = Path { spline: true, looped: false, s: vec![0.0, 1.0, 2.0], l: vec![0.0, 100.0, 200.0], pts, tan_in: tan.clone(), tan_out: tan, xf: M4_IDENT };
        let tab = path.FUN_001e5fc8(0);
        assert!((tab[10] - 100.0).abs() < 1e-3 && (tab[5] - 50.0).abs() < 1e-3);
        assert!((path.FUN_001e63c0(1.25) - 125.0).abs() < 1e-2);
        assert!((path.FUN_001e64c8(125.0) - 1.25).abs() < 1e-3);
        let c = path.FUN_001e6f60([130.0, 40.0, 0.0]);
        assert!(close(c.cp, [130.0, 0.0, 0.0], 0.5), "{:?}", c.cp);
        assert!((c.s - 1.3).abs() < 0.01);
    }

    #[test]
    fn target_list_merge() {
        let slot = |obj: usize, dist: f32, u: f32| TgtSlot { obj, member: obj, dist, flag: 0, ind: None, u, du: 0.0, pos: [dist, 0.0, 0.0], member_14: 0, wrap: None };
        let mut w = vec![];
        let mut list = vec![slot(2, 300.0, 0.0), slot(1, 100.0, 0.0)];
        let mut next = 50;
        let fx = FUN_00194f70(&mut w, &mut list, 1.0 / 60.0, false, Some(1), &mut |_| { next += 1; Some(next) }, &|s, u| [s.pos[0], u, 0.0]);
        assert_eq!(w.iter().map(|s| s.obj).collect::<Vec<_>>(), vec![1, 2]);
        assert_eq!(w[0].ind, Some(51)); // closest gets the indicator
        assert_eq!(fx, vec![IndFx::Event3d(1), IndFx::Event3d(2)]);
        // next frame: obj 2 gone (no indicator -> no fade), obj 1 moves u towards 10 under {1500,0,2}
        let mut list = vec![slot(1, 90.0, 10.0)];
        let fx = FUN_00194f70(&mut w, &mut list, 1.0 / 60.0, false, Some(1), &mut |_| None, &|s, u| [s.pos[0], u, 0.0]);
        assert_eq!(w.len(), 1);
        assert_eq!(w[0].dist, 90.0);
        assert!(w[0].u > 0.0 && w[0].u < 10.0);
        assert!(matches!(fx[0], IndFx::Place(51, _)));
    }

    #[test]
    fn sparkles_cap_at_ten() {
        let mut em = Emitter { rate_fc: 6000.0, spd: (10.0, 10.0), age: (0.0, 0.0), life: 1.0, adv: (0.0, 0.0), ..Default::default() };
        let s = TgtSlot { obj: 1, member: 1, dist: 0.0, flag: 0, ind: None, u: 0.0, du: 0.0, pos: ZERO, member_14: 0, wrap: None };
        let mut rng = GameRng::default();
        let ps = FUN_00195368(Some(&mut em), &[s, s], 5.0, 1.0 / 60.0, &mut rng, &|_, _| ([1.0, 2.0, 3.0], [0.0, 0.0, 1.0]));
        assert_eq!(ps.len(), 10);
        assert_eq!(ps[0], Particle { pos: [1.0, 2.0, 3.0], vel: [0.0, 0.0, 10.0], birth: 5.0, death: 6.0 });
    }
}
