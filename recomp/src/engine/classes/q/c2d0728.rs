//! WP-Q7: class C2d0728 (parent C2d06a0) queue package: slot 5, slot 9 (update: path-follow state
//! machine on +0x4c), and the path-intercept helpers it uses (FUN_001bd718 / FUN_001bd310 / FUN_001bcce0
//! / FUN_001bcf18 / FUN_001bcac0, FUN_001e7b38). Written from the disassembly (the decompile drops the
//! float / quadword arguments). Layout: +0x10 target ALO, +0x30 / +0x34 / +0x38 path query args, +0x40
//! hold, +0x4c state, +0x54 path, +0x58 query result, +0x5c / +0x60 path params, +0x68 / +0x6c radii,
//! +0x80 home position. State changes go through slot 14 (+0x7c).
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};
use crate::engine::classes::q::q7util::*;

/// &DAT_003dd020 (track query context)
pub const TRACK_CTX: u32 = 0x003d_d020;

/// FUN_00114470 (slot 5)
pub fn c2d0728_v5() -> u32 {
    0x002b_b7e0
}

/// FUN_001e7b38: signed path-parameter difference b - a, wrapped into (-L/2, L/2) on a closed path
/// (+0xe4) of length L = FUN_001e6648(path).
pub fn ee_sub_1e7b38<M: QMem + ?Sized>(m: &mut M, a: f32, b: f32, path: u32) -> f32 {
    let mut d = b - a;
    if m.r32(path + 0xe4) != 0 {
        let l = fr(m.call(0x001e_6648, &[path], &[]));
        if l * 0.5 < d {
            d -= l;
        } else if d < l * -0.5 {
            d += l;
        }
    }
    d
}

/// 2D (xy) distance, VU `vmul.xy` / `vaddy.x` / `vsqrt`
fn dist_xy(a: V4, b: V4) -> f32 {
    let d = sub(a, b);
    (d[0] * d[0] + d[1] * d[1]).sqrt()
}

/// FUN_001789f8 (slot 9)
pub fn c2d0728_v9<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef98, &[o], &[]);
    m.call(0x0017_7f78, &[o], &[]);
    let t = m.r32(o + 0x10);
    let pos = m.call(0x001a_62e8, &[t], &[]).1;
    let mut s1 = m.r32(o + 0x4c);
    loop {
        match s1 {
            0 | 2 | 3 => {
                let mut v = m.r32(o + 0x58);
                if m.r32(o + 0x40) == 0 {
                    let tg = m.r32(o + 0x10);
                    let e8 = m.r32(tg + 0xe8);
                    let v1 = if e8 != 0 { m.r32(e8 + 0xdc) } else { 0 };
                    let a1 = if v1 != 0 { m.r32(v1 + 0x10) } else { 0 };
                    // quirk: reads e8 + 0xdc even when e8 == 0
                    let a2 = m.r32(e8 + 0xdc).wrapping_add(0x20);
                    let a3 = m.r32(o + 0x30);
                    let dir = (m.r32(o + 0x38) == 0) as u32;
                    let t0 = m.rf(o + 0x34);
                    let r = ee_sub_1bd718(m, t0, pos, a1, a2, a3, dir, o + 0x54, o + 0x60);
                    m.w32(o + 0x58, r);
                    match r {
                        1 | 2 => {
                            let sp = stack_push(m, 0x10);
                            let p = m.r32(o + 0x54);
                            m.call(0x001e_6f60, &[p, TRACK_CTX, 0, 0, 0, sp], &[pos]);
                            let u = m.rf(sp);
                            let p60 = m.rf(o + 0x60);
                            let d = ee_sub_1e7b38(m, u, p60, p);
                            if 80.0 < d.abs() {
                                m.wf(o + 0x5c, u);
                                vslot(m, o, 14, &[1], &[]);
                            } else {
                                m.w32(o + 0x58, 0);
                            }
                            stack_pop(m, 0x10);
                        }
                        3 | 4 => {
                            vslot(m, o, 14, &[3], &[]);
                        }
                        _ => {}
                    }
                    v = m.r32(o + 0x58);
                }
                if v == 0 {
                    let r = if m.r32(o + 0x4c) != 0 { m.rf(o + 0x6c) } else { m.rf(o + 0x68) };
                    let home = m.rv(o + 0x80);
                    let k = if r < dist_xy(pos, home) { 0 } else { 2 };
                    vslot(m, o, 14, &[k], &[]);
                }
            }
            1 | 4 => {
                let p = m.r32(o + 0x54);
                let k = if (m.r64(p + 0x28) >> 60) & 1 == 0 || m.r32(p + 0x1d8) != 0 {
                    0
                } else {
                    let (a, b) = (m.rf(o + 0x5c), m.rf(o + 0x60));
                    if ee_sub_1e7b38(m, a, b, p).abs() < 80.0 {
                        0
                    } else {
                        let r = if m.r32(o + 0x4c) == 1 { m.rf(o + 0x68) } else { m.rf(o + 0x6c) };
                        let home = m.rv(o + 0x80);
                        if r < dist_xy(pos, home) {
                            1
                        } else {
                            4
                        }
                    }
                };
                vslot(m, o, 14, &[k], &[]);
            }
            _ => {}
        }
        let s = m.r32(o + 0x4c);
        if s == s1 {
            return;
        }
        s1 = s;
    }
}

/// FUN_001bd718: find the track segment `seg` crossing from `p` to the path point at parameter t +- step
/// (DAT_002c0ac4, by `dir`), *out_seg = seg; then FUN_001bd560 or, when that gives -1, the arc test
/// FUN_001bcce0 between the two projected parameters (*out_t = the target's parameter).
#[allow(clippy::too_many_arguments)]
pub fn ee_sub_1bd718<M: QMem + ?Sized>(m: &mut M, t: f32, p: V4, a1: u32, a2: u32, path: u32, dir: u32, out_seg: u32, out_t: u32) -> u32 {
    let step = m.rf(0x002c_0ac4);
    let tt = if dir != 0 { t + step } else { t - step };
    let u = fr(m.call(0x001e_66c8, &[path], &[f(tt)]));
    let sp = stack_push(m, 0x30);
    m.call(0x001e_6f10, &[path, sp, 0], &[f(u)]);
    let q = m.rv(sp);
    let seg = ee_sub_1bd310(m, a1, a2, p, q);
    if out_seg != 0 {
        m.w32(out_seg, seg);
    }
    let r = if seg == 0 {
        0
    } else {
        let r = m.call(0x001b_d560, &[seg, path, dir, sp + 0x10, 0], &[f(u)]).0;
        if r != u32::MAX {
            r
        } else {
            m.call(0x001e_6f60, &[seg, TRACK_CTX, 0, 0, 0, sp + 0x20], &[p]);
            let q2 = m.rv(sp + 0x10);
            m.call(0x001e_6f60, &[seg, TRACK_CTX, 0, 0, 0, sp + 0x24], &[q2]);
            let pt = if out_t != 0 {
                let v = m.r32(sp + 0x24);
                m.w32(out_t, v);
                out_t
            } else {
                sp + 0x24
            };
            let t0 = m.rf(sp + 0x20);
            ee_sub_1bcce0(m, t0, seg, a1, pt, u32::MAX)
        }
    };
    stack_pop(m, 0x30);
    r
}

/// FUN_001bd310: of the (<= 0x80) segments FUN_001c1700 finds between p and q, the first that
/// FUN_001bc688(.., 2) rejects or hits at a non-zero entry fraction; else the one with the smallest exit
/// fraction (< 1), else 0.
pub fn ee_sub_1bd310<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, p: V4, q: V4) -> u32 {
    let sp = stack_push(m, 0x300);
    let n = m.call(0x001c_1700, &[a0, a1, 0x80, sp], &[p, q]).0 as i32;
    let (mut best, mut fmin) = (0u32, 1.0f32);
    for i in 0..n.max(0) as u32 {
        let s0 = m.r32(sp + 4 * i);
        if m.call(0x001b_c688, &[s0, 2, sp + 0x200], &[p, q]).0 == 0 || m.rf(sp + 0x240) != 0.0 {
            best = s0;
            break;
        }
        let f1 = m.rf(sp + 0x244);
        if !(fmin <= f1) {
            fmin = f1;
            best = s0;
        }
    }
    stack_pop(m, 0x300);
    best
}

/// FUN_001bcce0: project the segment ends onto `path` (FUN_001bcf18, forward then backward search over
/// half the length), *pt = second parameter; compare both arc directions with FUN_001bcac0.
/// Returns 1 / 2 (direction), 4 (no projection / both blocked).
pub fn ee_sub_1bcce0<M: QMem + ?Sized>(m: &mut M, t: f32, path: u32, obj: u32, pt: u32, dir: u32) -> u32 {
    let sp = stack_push(m, 0x10);
    let r = 'r: {
        let mut ok = false;
        for k in 0..2 {
            let l = fr(m.call(0x001e_6648, &[path], &[]));
            if ee_sub_1bcf18(m, t, l * 0.5, path, obj, sp, k) != 0 {
                ok = true;
                break;
            }
        }
        if !ok {
            break 'r 4;
        }
        let f23 = m.rf(sp);
        let mut ok = false;
        for k in 0..2 {
            let l = fr(m.call(0x001e_6648, &[path], &[]));
            let t1 = m.rf(pt);
            if ee_sub_1bcf18(m, t1, l * 0.5, path, obj, sp + 4, k) != 0 {
                ok = true;
                break;
            }
        }
        if !ok {
            break 'r 4;
        }
        let f22 = m.rf(sp + 4);
        let mut d = f22 - f23;
        m.wf(pt, f22);
        if d < 0.0 {
            d += fr(m.call(0x001e_6648, &[path], &[]));
        }
        let k = m.rf(0x002c_0ab8);
        let w = match dir {
            1 => 1.0 - k,
            2 => k + 1.0,
            _ => 1.0,
        };
        let l = fr(m.call(0x001e_6648, &[path], &[]));
        let short = d < l * w * 0.5;
        let s0 = ee_sub_1bcac0(m, path, obj, 1, f23, f22);
        let v1 = ee_sub_1bcac0(m, path, obj, 0, f23, f22);
        if v1 < s0 {
            1
        } else if s0 < v1 {
            2
        } else if s0 == 2 {
            4
        } else if short {
            2
        } else {
            1
        }
    };
    stack_pop(m, 0x10);
    r
}

/// FUN_001bcf18: search outward from parameter t (both sides, step DAT_002c0abc, up to `range`) for a path
/// point FUN_001c0920(obj, point, dir) accepts; *out = its parameter. Returns 1 found / 0.
pub fn ee_sub_1bcf18<M: QMem + ?Sized>(m: &mut M, t: f32, range: f32, path: u32, obj: u32, out: u32, dir: u32) -> u32 {
    let mut d = 0.0f32;
    if !(d < range) {
        return 0;
    }
    let sp = stack_push(m, 0x20);
    let mut tt = t - d;
    let r = loop {
        let mut hit = None;
        for (x, b) in [(tt, sp), (t + d, sp + 0x10)] {
            let u = fr(m.call(0x001e_66c8, &[path], &[f(x)]));
            m.call(0x001e_6f10, &[path, b, 0], &[f(u)]);
            let pv = m.rv(b);
            if m.call(0x001c_0920, &[obj, dir], &[pv]).0 != 0 {
                hit = Some(u);
                break;
            }
        }
        if let Some(u) = hit {
            if out != 0 {
                m.wf(out, u);
            }
            break 1;
        }
        d += m.rf(0x002c_0abc);
        if !(d < range) {
            break 0;
        }
        tt = t - d;
    };
    stack_pop(m, 0x20);
    r
}

/// FUN_001bcac0: walk the path vertices (+0xf4, world space via slot 30's matrix) between parameters t0
/// and t1 (index FUN_001e7668, step FUN_001bc530 forward when `fwd` else back) testing every edge with
/// FUN_001c0980(obj, a, b, pass); pass 0 then pass 1. Returns the first clear pass, else 2.
pub fn ee_sub_1bcac0<M: QMem + ?Sized>(m: &mut M, path: u32, obj: u32, fwd: u32, t0: f32, t1: f32) -> u32 {
    let i0 = m.call(0x001e_7668, &[path, 0, 0], &[f(t0)]).0;
    let i1 = m.call(0x001e_7668, &[path, 0, 0], &[f(t1)]).0;
    let sp = stack_push(m, 0x60);
    let mut r = 2;
    for pass in 0..2u32 {
        let (s0, s3) = if fwd != 0 {
            let a = m.call(0x001b_c530, &[path, i0.wrapping_add(1)], &[]).0;
            let b = m.call(0x001b_c530, &[path, i1.wrapping_add(1)], &[]).0;
            (a, b)
        } else {
            (i0, i1)
        };
        let mp = vslot(m, path, 30, &[], &[]).0;
        copy(m, sp, mp, 0x40);
        m.call(0x001e_6f10, &[path, sp + 0x40, 0], &[f(t0)]);
        let mut prev = m.rv(sp + 0x40);
        let mut s1 = s0;
        let mut blocked = false;
        while s1 != s3 {
            let vb = m.r32(path + 0xf4);
            let p = m.rv(vb + s1 * 0x10);
            let mx = m.rm(sp);
            let r3 = [mx[0], mx[1], mx[2]];
            let w = add(mul3(&r3, p, 0.0), [mx[3][0], mx[3][1], mx[3][2], 0.0]);
            if m.call(0x001c_0980, &[obj, pass], &[prev, w]).0 != 0 {
                blocked = true;
                break;
            }
            let nx = if fwd != 0 { s1.wrapping_add(1) } else { s1.wrapping_sub(1) };
            s1 = m.call(0x001b_c530, &[path, nx], &[]).0;
            prev = w;
        }
        if !blocked {
            m.call(0x001e_6f10, &[path, sp + 0x50, 0], &[f(t1)]);
            let e = m.rv(sp + 0x50);
            if m.call(0x001c_0980, &[obj, pass], &[prev, e]).0 == 0 {
                r = pass;
                break;
            }
        }
    }
    stack_pop(m, 0x60);
    r
}
