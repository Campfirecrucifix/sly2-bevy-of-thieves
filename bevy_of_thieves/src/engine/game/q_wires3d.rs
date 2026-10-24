//! WIRE-S3: 1:1 QMem ports of live-frontier natives (shard 5/6/7).
#![allow(unused)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

fn fb(b: u32) -> f32 {
    f32::from_bits(b)
}

fn dot3(a: V4, b: V4) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn sub4(a: V4, b: V4) -> V4 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2], a[3] - b[3]]
}

fn scale4(a: V4, s: f32) -> V4 {
    [a[0] * s, a[1] * s, a[2] * s, a[3] * s]
}

const T: u32 = 0x002d_ded0; // g_clock t
const DT: u32 = 0x002d_ded4; // g_clock dt
const GS: u32 = 0x003d_d490;
const PI: u32 = 0x4049_0fdb;
const HALF_PI: u32 = 0x3fc9_0fdb;
const EPS: u32 = 0x38d1_b717; // 1e-4

/// FUN_00190520: per-frame aim/turn state machine (+0x18d0) and blend weights.
pub fn ee_sub_190520<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let sp = stack_push(m, 0x120);
    let mut s1 = false;
    let mut s3 = false;
    m.call(0x0019_0be0, &[a0, sp + 0x90], &[]);
    if m.r32(a0 + 0x18fc) != 0 {
        let f0 = m.rf(sp + 0x90);
        let r = m.call(0x0011_cf60, &[], &[f4(f0 + fb(PI))]).1[0];
        m.wf(sp + 0x90, r);
    }
    let f12 = m.rf(sp + 0x90);
    m.call(0x0020_4660, &[a0], &[f4(f12)]);
    let p = m.r32(a0 + 0x150);
    let x = m.rf(p + 0x18) - m.rf(a0 + 0x1a8);
    let f2 = m.call(0x0011_cf60, &[], &[f4(x)]).1[0];
    let p = m.r32(a0 + 0x150);
    if fb(0x3e99_999a) < m.rf(p + 0x1c) {
        let f1 = f2.abs();
        let hit = if f1 < fb(0x3f86_0a92) {
            s1 = true;
            true
        } else if fb(0x4006_0a92) < f1 {
            s3 = true;
            true
        } else {
            false
        };
        if hit {
            let t = m.rf(T);
            m.wf(a0 + 0x18f4, t);
        }
    } else {
        m.w32(a0 + 0x18f8, u32::MAX);
    }
    let k = m.r32(a0 + 0x18c4);
    let q1 = m.call(0x001a_62e8, &[k], &[]).1;
    m.wv(sp + 0xa0, q1);
    let q2 = m.call(0x001a_62e8, &[a0], &[]).1;
    m.wv(sp, q2);
    let d = sub4(q1, q2);
    let len = dot3(d, d).sqrt();
    let v = if len < fb(EPS) { m.rv(0x002a_2dc0) } else { scale4(d, 1.0 / len) };
    m.wv(sp + 0x10, v);
    let up = m.rv(0x002a_2dc0);
    let f12 = m.rf(a0 + 0x1a8);
    m.call(0x0011_b500, &[sp + 0x60], &[up, f4(f12)]);
    let r0 = m.rv(sp + 0x60);
    let r1 = m.rv(sp + 0x70);
    let r2 = m.rv(sp + 0x80);
    let v = m.rv(sp + 0x10);
    let o = [dot3(r0, v), dot3(r1, v), dot3(r2, v), v[3]];
    m.wv(sp + 0x30, r0);
    m.wv(sp + 0x20, o);
    m.wv(sp + 0x40, r1);
    m.wv(sp + 0x50, r2);
    let a = m.call(0x0024_58f0, &[], &[f4(o[2]), f4(o[0])]).1[0];
    let f21 = m.call(0x0011_cf60, &[], &[f4(a - fb(HALF_PI))]).1[0];
    let mut s0 = m.r32(a0 + 0x18d0);
    match s0 as i32 {
        0 => {
            if s3 {
                s0 = 2;
            } else if s1 {
                if m.rf(0x002b_f554) < m.rf(T) - m.rf(a0 + 0x18d4) {
                    m.w32(a0 + 0x18f8, 0);
                    s0 = 1;
                }
            } else if m.rf(0x002b_f55c) < m.rf(T) - m.rf(a0 + 0x18f4) {
                s0 = 1;
            }
        }
        2 => {
            if s1 {
                s0 = 0;
            } else if s3 {
                if m.rf(0x002b_f558) < m.rf(T) - m.rf(a0 + 0x18d4) {
                    m.w32(a0 + 0x18f8, s0);
                    s0 = 1;
                }
            } else if m.rf(0x002b_f55c) < m.rf(T) - m.rf(a0 + 0x18f4) {
                s0 = 1;
            }
        }
        1 => {
            if s1 && m.r32(a0 + 0x18f8) != 0 {
                s0 = 0;
                m.w32(a0 + 0x18f8, u32::MAX);
            } else if s3 && m.r32(a0 + 0x18f8) != 2 {
                s0 = 2;
                m.w32(a0 + 0x18f8, u32::MAX);
            } else if f21.abs() < m.rf(0x002b_f560) {
                let q = vslot(m, a0, 32, &[], &[]).1;
                let l = dot3(q, q).sqrt();
                if l < m.rf(0x002b_f568) {
                    s0 = 3;
                }
            }
        }
        3 => {
            if s1 && m.r32(a0 + 0x18f8) != 0 {
                s0 = 0;
                m.w32(a0 + 0x18f8, u32::MAX);
            } else if s3 && m.r32(a0 + 0x18f8) != 2 {
                s0 = 2;
                m.w32(a0 + 0x18f8, u32::MAX);
            } else if m.rf(0x002b_f564) < f21.abs() {
                let q = vslot(m, a0, 32, &[], &[]).1;
                let l = dot3(q, q).sqrt();
                if m.rf(0x002b_f56c) < l {
                    s0 = 1;
                }
            }
        }
        _ => {}
    }
    m.call(0x0018_c250, &[a0, 3, s0], &[]);
    m.call(0x0018_fc60, &[a0], &[]);
    let v = m.r32(a0 + 0x1900);
    if v != 0 {
        let k = m.r32(v + 4);
        m.call(0x001f_5078, &[k], &[]);
    }
    let k = m.r32(a0 + 0x150);
    if m.call(0x001c_4660, &[k, 0x400], &[f4(fb(0x3e19_999a))]).0 != 0
        && vslot(m, a0, 98, &[], &[]).0 != 0
    {
        let k = m.r32(a0 + 0x150);
        m.call(0x001c_4738, &[k, 0x400], &[]);
        m.call(0x0019_cc50, &[a0 + 0xf00, 0, 0, 0], &[]);
    }
    let st = m.r32(a0 + 0x18d0);
    if st == 3 {
        stack_pop(m, 0x120);
        return;
    }
    // ponytail: f20/f22 are the caller's callee-saved values for other states; 0.0 here
    let (f20, f22) = match st {
        0 => (1.0f32, 1.0f32),
        1 => (0.0, if 0.5 < m.rf(a0 + 0x1998) { 1.0 } else { 0.0 }),
        2 => (1.0, 0.0),
        _ => (0.0, 0.0),
    };
    let f12 = m.rf(a0 + 0x199c);
    let dt = m.rf(DT);
    let w0 = m.call(0x0011_d018, &[0x002b_f570, 0], &[f4(f12), f4(f20), f4(dt)]).1[0];
    m.wf(a0 + 0x199c, w0);
    let f12 = m.rf(a0 + 0x1998);
    let dt = m.rf(DT);
    let w1 = m.call(0x0011_d018, &[0x002b_f580, 0], &[f4(f12), f4(f22), f4(dt)]).1[0];
    let f1 = m.rf(a0 + 0x199c);
    m.wf(a0 + 0x1998, w1);
    m.wf(a0 + 0x1970, f1 * w1);
    m.wf(a0 + 0x1988, 1.0 - f1);
    m.wf(a0 + 0x197c, f1 * (1.0 - w1));
    let k = m.r32(a0 + 0x1994);
    m.call(0x0013_84c0, &[k], &[]);
    let x = f21 / m.rf(0x002b_f58c);
    let c = if x < -1.0 {
        -1.0
    } else if 1.0 < x {
        1.0
    } else {
        x
    };
    let p = m.r32(a0 + 0x1994);
    let q = m.r32(a0 + 0x1030);
    let f12 = (m.rf(p + 0x74) * (c + 1.0) * 0.5 - m.rf(q + 0x1c)) / m.rf(DT);
    let f13 = m.rf(0x002b_f590);
    let r = m.call(0x0011_cfe0, &[], &[f4(f12), f4(f13)]).1[0];
    let q = m.r32(a0 + 0x1030);
    m.wf(q + 0x40, r);
    stack_pop(m, 0x120);
}

/// FUN_001fdc08: pick dominant axis +0x64, ease its weight to 1, renormalize the 3 weights at +0x108.
pub fn ee_sub_1fdc08<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.w32(a0 + 0x64, 1);
    if m.r32(a0 + 0x5c) != 0 {
        m.w32(a0 + 0x64, 2);
    } else if m.r32(a0 + 0x60) != 0 {
        m.w32(a0 + 0x64, 0);
    } else if m.call(0x001c_4a48, &[GS], &[]).0 != 0 {
        let p = m.call(0x001c_4a48, &[GS], &[]).0;
        if m.call(0x0013_2b10, &[p, 0x34], &[]).0 != 0 {
            let p = m.call(0x001c_4a48, &[GS], &[]).0;
            let v = m.r32(p + 0x10ec);
            if v == 3 {
                m.w32(a0 + 0x64, 0);
            } else if v == 4 {
                m.w32(a0 + 0x64, 2);
            } else if m.r32(p + 0x10c8) != u32::MAX {
                m.w32(a0 + 0x64, 0);
            } else {
                let v = m.r32(p + 0x4c8) as i32;
                if v >= 0 && (v < 3 || v == 13) {
                    m.w32(a0 + 0x64, 0);
                }
            }
        }
    }
    let n = m.r32(a0 + 0x204);
    let s2 = if (n as i32) > 0 { m.r32(a0.wrapping_add(n << 2) + 0x204) } else { 0 };
    let base = a0 + 0x108;
    let fixed = s2 == 0 || m.r16(s2 + 0x52) != 0 || m.r16(s2 + 0x50) & 1 != 0;
    if fixed {
        let i = m.r32(a0 + 0x64);
        m.wf(base.wrapping_add(i << 2), 1.0);
    } else {
        let i = m.r32(a0 + 0x64);
        let f12 = m.rf(base.wrapping_add(i << 2));
        let f14 = m.rf(0x002d_df10);
        let p = a0.wrapping_add(i.wrapping_mul(12)).wrapping_add(0x114);
        let r = m.call(0x0011_d018, &[p, 0], &[f4(f12), f4(1.0), f4(f14)]).1[0];
        let i = m.r32(a0 + 0x64);
        m.wf(base.wrapping_add(i << 2), r);
    }
    let i = m.r32(a0 + 0x64);
    let w = m.rf(base.wrapping_add(i << 2));
    let f2 = w * w;
    let mut f1 = 0.0f32;
    for j in 0..3u32 {
        if j != i {
            let x = m.rf(base + j * 4);
            f1 += x * x;
        }
    }
    if f1 < fb(EPS) {
        for j in 0..3u32 {
            let v = if j == m.r32(a0 + 0x64) { 1.0 } else { 0.0 };
            m.wf(base + j * 4, v);
        }
    } else {
        let s = ((1.0 - f2) / f1).sqrt();
        for j in 0..3u32 {
            if j != m.r32(a0 + 0x64) {
                let x = m.rf(base + j * 4);
                m.wf(base + j * 4, x * s);
            }
        }
    }
    if s2 != 0 {
        for j in 0..3u32 {
            let f = m.rf(base + j * 4);
            m.call(0x001f_c2e0, &[s2, j], &[f4(f)]);
        }
    }
}

/// FUN_00193a68: project aim dir onto plane, out a2 = pos - owner[+0x240], a3 = dir * clamp(speed).
pub fn ee_sub_193a68<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, f12: f32) {
    let sp = stack_push(m, 0xf0);
    let o = m.r32(a0);
    let s0 = m.r32(o + 0x150);
    m.call(0x001e_67b8, &[a1, sp, sp + 0x10], &[]);
    let v3 = m.rv(sp + 0x10);
    let n = m.rv(0x002a_2dc0);
    let d = dot3(v3, n);
    let v2 = sub4(v3, scale4(n, d));
    let len = dot3(v2, v2).sqrt();
    let dir = if len < fb(EPS) { m.rv(0x002a_2dd0) } else { scale4(v2, 1.0 / len) };
    m.wv(sp + 0x10, dir);
    let f21 = 50.0f32;
    let o = m.r32(a0);
    let f13 = m.rf(a0 + 0x2a0) * m.rf(s0 + 0x1c);
    let p = m.r32(o + 0x150);
    let f12b = m.rf(p + 0x18);
    let q = m.call(0x0011_e1a0, &[], &[f4(f12b), f4(f13), f4(0.0)]).1;
    let dir = m.rv(sp + 0x10);
    let f22 = dot3(q, dir);
    m.wv(sp + 0x20, q);
    let f20 = m.call(0x001e_63c0, &[a1], &[f4(f12)]).1[0];
    let x = m.rf(a0 + 0x2a0);
    let y = m.rf(a0 + 0x260);
    let z = m.rf(0x002b_f668);
    m.wf(sp + 0x34, y);
    m.wf(sp + 0x30, x);
    m.wf(sp + 0x38, z);
    let dt = m.rf(DT);
    m.call(0x0011_d018, &[sp + 0x30, sp + 0x50], &[f4(f20), f4(f21), f4(dt)]);
    let f0 = m.call(0x001e_6648, &[a1], &[]).1[0];
    let dt = m.rf(DT);
    m.call(0x0011_d018, &[sp + 0x30, sp + 0x54], &[f4(f20), f4(f0 - f21), f4(dt)]);
    let lo = m.rf(sp + 0x50);
    let o = m.r32(a0);
    let v = sub4(m.rv(sp), m.rv(o + 0x240));
    m.wv(a2, v);
    let hi = m.rf(sp + 0x54);
    let s = if f22 < lo {
        lo
    } else if hi < f22 {
        hi
    } else {
        f22
    };
    m.wv(sp + 0x40, f4(s));
    let dir = m.rv(sp + 0x10);
    m.wv(a3, scale4(dir, s));
    stack_pop(m, 0xf0);
}

/// FUN_0011c950: interpolated rotation of matrix a0 toward a1 by angle step f12; a3 = result, t0 = axis * scale.
pub fn ee_sub_11c950<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, t0: u32, f12: f32) {
    let sp = stack_push(m, 0x130);
    m.call(0x0011_bc00, &[sp + 0x10, a0, a1], &[]);
    m.call(0x0011_bcf8, &[sp + 0x10, sp + 0xa0, sp], &[]);
    let ang = m.rf(sp + 0xa0);
    let f0 = m.call(0x0011_d018, &[a2, sp + 0xa4], &[f4(0.0), f4(ang), f4(f12)]).1[0];
    if a3 != 0 {
        if f0 == m.rf(sp + 0xa0) {
            for i in 0..12 {
                let w = m.r32(a1 + i * 4);
                m.w32(a3 + i * 4, w);
            }
        } else {
            let axis = m.rv(sp);
            m.call(0x0011_b500, &[sp + 0x40], &[axis, f4(f0)]);
            let r2 = m.rv(sp + 0x40);
            let r3 = m.rv(sp + 0x50);
            let r1 = m.rv(sp + 0x60);
            let mul = |v: V4| -> V4 {
                [
                    r2[0] * v[0] + r3[0] * v[1] + r1[0] * v[2],
                    r2[1] * v[0] + r3[1] * v[1] + r1[1] * v[2],
                    r2[2] * v[0] + r3[2] * v[1] + r1[2] * v[2],
                    v[3],
                ]
            };
            let v5 = mul(m.rv(a0 + 0x20));
            let v4 = mul(m.rv(a0));
            let v6 = mul(m.rv(a0 + 0x10));
            m.wv(a3 + 0x20, v5);
            m.wv(a3, v4);
            m.wv(a3 + 0x10, v6);
            m.wv(sp + 0x70, v4);
            m.wv(sp + 0x10, v4);
            m.wv(sp + 0x80, v6);
            m.wv(sp + 0x20, v6);
            m.wv(sp + 0x90, v5);
            m.wv(sp + 0x30, v5);
        }
    }
    if t0 != 0 {
        let k = m.rf(sp + 0xa4);
        let v = m.rv(sp);
        m.wv(t0, scale4(v, k));
    }
    stack_pop(m, 0x130);
}

/// FUN_001c4330: 2D stick (a1,a2) -> radius t0 / heading a3, decaying t1 by 0.2.
pub fn ee_sub_1c4330<M: QMem + ?Sized>(m: &mut M, a1: u32, a2: u32, a3: u32, t0: u32, t1: u32, f12: f32) {
    let sp = stack_push(m, 0x80);
    let x = m.rf(a1);
    let y = m.rf(a2);
    m.wf(t0, (x * x + y * y).sqrt());
    let f0 = (m.rf(t1) - fb(0x3e4c_cccd)).max(0.0);
    m.wf(t1, f0);
    let f1 = m.rf(t0);
    if f1 < f0 || f1 == 0.0 {
        let a = m.rf(a3) - f12 + fb(HALF_PI);
        m.call(0x0011_d608, &[sp, sp + 4], &[f4(a)]);
        let r = m.rf(t1);
        let c = m.rf(sp + 4);
        m.wf(t0, r);
        m.wf(a1, r * c);
        let s = m.rf(sp);
        let r = m.rf(t0);
        m.wf(a2, r * s);
    } else {
        let yy = m.rf(a2);
        let xx = m.rf(a1);
        let a = m.call(0x0024_58f0, &[], &[f4(yy), f4(xx)]).1[0];
        let r = m.call(0x0011_cf60, &[], &[f4(a + f12 - fb(HALF_PI))]).1[0];
        m.wf(a3, r);
        let v = m.rf(t0);
        m.wf(t1, v);
    }
    stack_pop(m, 0x80);
}

/// FUN_0023dc18: compare two unpacked soft-floats (class +0, sign +4, exp +8, frac +0x10 u64).
pub fn ee_sub_23dc18<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let sgn = |s: u32| -> i32 { if s == 0 { 1 } else { -1 } };
    let c0 = m.r32(a0);
    if c0 < 2 {
        return 1;
    }
    let c1 = m.r32(a1);
    if c1 < 2 {
        return 1;
    }
    let r: i32 = if c0 == 4 {
        if c1 == 4 {
            m.r32(a1 + 4).wrapping_sub(m.r32(a0 + 4)) as i32
        } else {
            sgn(m.r32(a0 + 4))
        }
    } else if c1 == 4 {
        -sgn(m.r32(a1 + 4))
    } else if c0 == 2 {
        if c1 == 2 {
            0
        } else {
            -sgn(m.r32(a1 + 4))
        }
    } else if c1 == 2 {
        sgn(m.r32(a0 + 4))
    } else {
        let s = m.r32(a0 + 4);
        if s != m.r32(a1 + 4) {
            sgn(s)
        } else {
            let e0 = m.r32(a0 + 8) as i32;
            let e1 = m.r32(a1 + 8) as i32;
            if e1 < e0 {
                sgn(s)
            } else if e0 < e1 {
                -sgn(s)
            } else {
                let m0 = m.r64(a0 + 0x10);
                let m1 = m.r64(a1 + 0x10);
                if m1 < m0 {
                    sgn(s)
                } else if m0 < m1 {
                    -sgn(s)
                } else {
                    0
                }
            }
        }
    };
    r as u32
}

// SKIP 0x23c078: soft-float long->double; passes 64-bit doubles in GPRs (v0/a0 to 0x23dd80/0x23d808/0x23d748/0x23dfa0), not representable by m.call's u32.

/// FUN_001d8c10: varargs wrapper: format (a1.., f12..) via FUN_001d87c0 into buf, lookup FUN_001d7e80(0x3e0ea8, buf, 0x3d4a60).
pub fn ee_sub_1d8c10<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, t0: u32, t1: u32, t2: u32, t3: u32, f12: f32, f14: f32, f16: f32, f18: f32) -> u32 {
    let sp = stack_push(m, 0xf0);
    for (i, x) in [a1, a2, a3, t0, t1, t2, t3].iter().enumerate() {
        let p = sp + 0xb8 + 8 * i as u32;
        m.w32(p, *x);
        m.w32(p + 4, ((*x as i32) >> 31) as u32);
    }
    m.wf(sp + 0xa8, f12);
    m.wf(sp + 0xac, f14);
    m.wf(sp + 0xb0, f16);
    m.wf(sp + 0xb4, f18);
    m.w32(sp, u32::MAX);
    m.call(0x001a_e478, &[sp + 0x10], &[]);
    m.call(0x001d_87c0, &[a0, sp + 0xb8, sp + 0x20], &[]);
    let r = m.call(0x001d_7e80, &[0x003e_0ea8, sp + 0x20, 0x003d_4a60], &[]).0;
    m.call(0x001a_e4a0, &[sp + 0x10, 2], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0xf0);
    r
}

/// FUN_00119308: plane through points p0,p1,p2: a0.xyz = unit normal, a0[+0xc] = n.p0.
pub fn ee_sub_119308<M: QMem + ?Sized>(m: &mut M, a0: u32, p0: V4, p1: V4, p2: V4) {
    let d1 = sub4(p1, p0);
    let d2 = sub4(p2, p0);
    let c = [
        d1[1] * d2[2] - d2[1] * d1[2],
        d1[2] * d2[0] - d2[2] * d1[0],
        d1[0] * d2[1] - d2[0] * d1[1],
        d1[3],
    ];
    let l = dot3(c, c).sqrt();
    let n = scale4(c, 1.0 / l);
    m.wf(a0 + 0xc, dot3(n, p0));
    m.wf(a0, n[0]);
    m.wf(a0 + 4, n[1]);
    m.wf(a0 + 8, n[2]);
}

/// FUN_00136c50: FUN_00141e70(o); (FUN_001c40c0(o) && FUN_00141ea8(o, 0xc)) ? FUN_001f6978(o, 0x24d) : FUN_001f69f8(o, 0x24d).
pub fn ee_sub_136c50<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0014_1e70, &[a0], &[]);
    if m.call(0x001c_40c0, &[a0], &[]).0 != 0 && m.call(0x0014_1ea8, &[a0, 0xc], &[]).0 != 0 {
        m.call(0x001f_6978, &[a0, 0x24d], &[]);
    } else {
        m.call(0x001f_69f8, &[a0, 0x24d], &[]);
    }
}

/// FUN_0015e1d8: unless flag bit 60 of +0x28: +0x68 = t; vslot 7.
pub fn ee_sub_15e1d8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    if (m.r32(a0 + 0x2c) >> 28) & 1 == 0 {
        let t = m.rf(T);
        m.wf(a0 + 0x68, t);
        vslot(m, a0, 7, &[], &[]);
    }
}

/// FUN_00150e60: +0x14 = a1; +0x40 != 0 -> FUN_00150ef0(o); +0x40 = 0.
pub fn ee_sub_150e60<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.w32(a0 + 0x14, a1);
    if m.r32(a0 + 0x40) != 0 {
        m.call(0x0015_0ef0, &[a0], &[]);
    }
    m.w32(a0 + 0x40, 0);
}

/// FUN_00113a40: return 0x2bab24.
pub fn ee_sub_113a40<M: QMem + ?Sized>(m: &mut M) -> u32 {
    0x002b_ab24
}

/// FUN_00113ac0: return 0x2babc4.
pub fn ee_sub_113ac0<M: QMem + ?Sized>(m: &mut M) -> u32 {
    0x002b_abc4
}

/// FUN_00113cc0: return 0x2bae44.
pub fn ee_sub_113cc0<M: QMem + ?Sized>(m: &mut M) -> u32 {
    0x002b_ae44
}

/// FUN_00113ed0: return 0x2bb0d8.
pub fn ee_sub_113ed0<M: QMem + ?Sized>(m: &mut M) -> u32 {
    0x002b_b0d8
}

/// FUN_00114140: return 0x2bb3e4.
pub fn ee_sub_114140<M: QMem + ?Sized>(m: &mut M) -> u32 {
    0x002b_b3e4
}

/// FUN_00114460: return 0x2bb7cc.
pub fn ee_sub_114460<M: QMem + ?Sized>(m: &mut M) -> u32 {
    0x002b_b7cc
}

/// FUN_00150ee8: +0x18 = f12.
pub fn ee_sub_150ee8<M: QMem + ?Sized>(m: &mut M, a0: u32, f12: f32) {
    m.wf(a0 + 0x18, f12);
}
