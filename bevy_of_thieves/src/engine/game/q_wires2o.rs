//! WIRE-S2: 1:1 QMem ports of live-frontier natives (shard 3/4).
#![allow(unused)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

const T: u32 = 0x002d_ded0; // clock: +0 t, +4 dt, +0x10
const CM: u32 = 0x002d_df40; // camera manager
const EPS: u32 = 0x38d1_b717; // 1e-4
const TWO_PI: u32 = 0x40c9_0fdb;
const HALF_PI: u32 = 0x3fc9_0fdb;
const PI: u32 = 0x4049_0fdb;

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}
fn fb(b: u32) -> f32 {
    f32::from_bits(b)
}
/// float-returning call (f0)
fn cf<M: QMem + ?Sized>(m: &mut M, addr: u32, a: &[u32], f: &[f32]) -> f32 {
    let v: Vec<V4> = f.iter().map(|&x| f4(x)).collect();
    m.call(addr, a, &v).1[0]
}
fn ang<M: QMem + ?Sized>(m: &mut M, x: f32) -> f32 {
    cf(m, 0x0011_cf60, &[], &[x])
}
fn vsub(a: V4, b: V4) -> V4 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2], a[3] - b[3]]
}
fn vadd(a: V4, b: V4) -> V4 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2], a[3] + b[3]]
}
fn vscl(a: V4, s: f32) -> V4 {
    [a[0] * s, a[1] * s, a[2] * s, a[3] * s]
}
/// vmul.xyz + vadday.x + vmaddz.x (with vf02.x = 1)
fn dot3(a: V4, b: V4) -> f32 {
    (a[0] * b[0] + a[1] * b[1]) + 1.0 * (a[2] * b[2])
}
/// VU0 normalize idiom: len = sqrt(dot3(v,v)); len < 1e-4 -> quad at `c`, else v * (1/len) (xyzw)
fn nrm<M: QMem + ?Sized>(m: &mut M, v: V4, c: u32) -> (V4, f32) {
    let l = dot3(v, v).sqrt();
    if l < fb(EPS) {
        (m.rv(c), l)
    } else {
        (vscl(v, 1.0 / l), l)
    }
}
/// vmul.xy + vaddy.x -> sqrt
fn len2(v: V4) -> f32 {
    (v[0] * v[0] + v[1] * v[1]).sqrt()
}
/// mfc1 + qmtc2: x = f, y = sign-extension word of f
fn qf(f: f32, hi: [f32; 2]) -> V4 {
    let sx = if (f.to_bits() as i32) < 0 { 0xffff_ffffu32 } else { 0 };
    [f, fb(sx), hi[0], hi[1]]
}

/// FUN_00221dd8: CplcyFollow::update — follow-camera policy frame (track modes 1/2, manual mode 4, cut placement).
pub fn ee_sub_221dd8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    let sp = stack_push(m, 0x130);
    let (s2, s4, s5) = (a0, a1, a2);
    let fp = m.r32(s2);
    let s3 = m.r32(s2 + 0x10);
    let r = m.call(0x0013_2b10, &[fp, 0x34], &[]).0;
    let f21 = m.rf(T + 0x10);
    let s7 = if r != 0 { fp } else { 0 };
    m.call(0x0022_36e8, &[s2], &[]);
    let s6;
    if s3 == 0 {
        m.w32(sp + 0x128, 0);
        s6 = 4u32;
        m.w32(sp + 0x124, 0);
    } else {
        let v = m.r32(s3 + 0x168);
        s6 = m.r32(s3 + 0x164);
        m.w32(sp + 0x124, v);
        let v = m.r32(s3 + 0x16c);
        m.w32(sp + 0x128, v);
    }
    m.call(0x0022_1ce8, &[s2, s4], &[]);
    let o14 = m.r32(s2 + 0x14);
    m.call(0x0022_0dd8, &[o14, s4, sp, s3], &[]);
    if s7 != 0 && m.r32(s7 + 0x4f4) == 2 {
        let q = vsub(m.rv(s7 + 0x500), m.rv(sp));
        m.call(0x0014_e6d0, &[CM, CM + 0x2e0, CM + 0x2f0], &[q, f4(8.0), f4(-6.0)]);
    }
    let v = vadd(m.rv(sp), m.rv(0x002d_e220));
    m.wv(sp + 0x10, v);
    if s3 != 0 && m.r32(s3 + 0x190) != 0 {
        let w = m.r32(s4) & 0xffff_fffe;
        m.w32(s4, w);
    }
    let f13 = if m.r32(s4) & 1 == 0 {
        let o = m.r32(s2 + 0x14);
        m.rf(o + 0x64)
    } else {
        0.0
    };
    let (f12, f14) = (m.rf(s2 + 0x64), m.rf(T + 4));
    let f0 = cf(m, 0x0011_d018, &[0x002c_4230, 0], &[f12, f13, f14]);
    m.wf(s2 + 0x64, f0);
    let x = m.rf(s4 + 0x38).max(m.rf(s2 + 0x60));
    m.wf(s2 + 0x60, x);
    for o in [0x100, 0xf0, 0xf4, 0xfc] {
        m.wf(sp + o, 0.0);
    }
    if s3 != 0 && s6 & 3 != 0 {
        let q = m.rv(sp);
        m.call(0x0022_1408, &[s3, sp + 0x30, sp + 0x20], &[q]);
        if m.r32(s3 + 0x120) != 0 {
            let q = m.rv(sp + 0x30);
            m.call(0x0011_ef08, &[sp + 0x30, sp + 0x34, sp + 0x38], &[q]);
            let mut f0 = m.rf(sp + 0x30);
            if m.r32(s2 + 0x38) == 0 {
                let x = m.rf(s2 + 0x4c) - f0;
                let a = ang(m, x);
                let k = m.rf(s3 + 0x130);
                let d = cf(m, 0x0011_cfe0, &[], &[a, k]);
                let v = m.rf(sp + 0x30) + d;
                m.wf(sp + 0x30, v);
                f0 = m.rf(sp + 0x30);
            }
            m.wf(s2 + 0x4c, f0);
        }
        for (bit, out) in [(1u32, 0xf0u32), (2, 0xfc)] {
            if s6 & bit != 0 {
                let a2 = m.r32(s2 + 0x14);
                let f12 = m.rf(s3 + 0x140);
                let f13 = m.rf(a2 + 4);
                m.call(0x0022_15c0, &[s3, sp + 0x30, a2 + 0x10, a2 + 0x40, bit, sp + 0x40], &[f4(f12), f4(f13)]);
                let q = vsub(m.rv(sp + 0x40), m.rv(sp + 0x10));
                m.call(0x0011_efb0, &[sp + out, sp + out + 4, sp + out + 8], &[q]);
            }
        }
    }
    if m.r32(s2 + 0x38) != 0 {
        m.w32(s2 + 0x34, 4);
    }
    let g = m.r32(CM + 0x300);
    if g != 0 {
        if g == 2 {
            m.w32(s2 + 0x34, 4);
            let v1 = m.r32(s2 + 0x14);
            let x = m.rf(CM + 0x304);
            m.wf(s2 + 0x18, x);
            let f12 = m.rf(v1 + 0x5c) * m.rf(CM + 0x44);
            let f0 = cf(m, 0x0024_4b68, &[], &[f12]);
            let f12 = m.rf(CM + 0x308) - f0;
            let f0 = cf(m, 0x0011_d948, &[0x002c_41b0], &[f12]);
            m.wf(s2 + 0x1c, f0);
        } else if s6 & 3 == 0 {
            m.w32(s2 + 0x34, 4);
            let v1 = m.r32(s2 + 0x14);
            let x = m.rf(s4 + 0x30);
            m.wf(s2 + 0x18, x);
            let x = m.rf(v1 + 0x88);
            m.wf(s2 + 0x1c, x);
        } else if s6 & 2 == 0 {
            m.w32(s2 + 0x34, 1);
        } else if s6 & 1 == 0 {
            m.w32(s2 + 0x34, 2);
        } else {
            let x = m.rf(sp + 0xf0) - m.rf(s4 + 0x30);
            let f20 = ang(m, x).abs();
            let x = m.rf(sp + 0xfc) - m.rf(s4 + 0x30);
            let f0 = ang(m, x).abs();
            m.w32(s2 + 0x34, if f20 < f0 { 1 } else { 2 });
        }
        let t = m.rf(T);
        m.wf(s2 + 0x60, t);
        let x = m.rf(T) - m.rf(0x002c_41dc);
        m.wf(s2 + 0x48, x);
    }
    let v1 = m.r32(s2 + 0x34);
    if v1 & s6 == 0 && (v1 as i32) > 0 && (v1 as i32) < 3 {
        m.w32(s2 + 0x34, 4);
    }
    if s3 == 0 || m.r32(s3 + 0x190) == 0 {
        let q = m.rv(s4 + 0x20);
        m.wv(sp + 0x30, q);
    } else {
        let a = m.r32(s3 + 0x190);
        let p = m.call(0x001a_62e8, &[a], &[]).1;
        let d = vsub(p, m.rv(s4 + 0x10));
        m.wv(sp + 0x40, d);
        m.w32(sp + 0x48, 0);
        let v = m.rv(sp + 0x40);
        let (n, l) = nrm(m, v, 0x002a_2dd0);
        m.wv(sp + 0x40, n);
        let f2 = l;
        let mut f3 = m.rf(s3 + 0x1a0) + f2 * (m.rf(s3 + 0x1a4) + f2 * m.rf(s3 + 0x1a8));
        let f4v = m.rf(s3 + 0x1b0);
        let f5 = m.rf(s3 + 0x1b4);
        if f3 < f4v {
            f3 = f4v;
        } else if f5 < f3 {
            f3 = f5;
        }
        let v = vscl(m.rv(sp + 0x40), f3);
        m.wv(sp + 0x50, qf(f3, [0.0, 0.0]));
        m.wv(sp + 0x30, v);
    }
    let mut f26 = m.rf(s2 + 0x18);
    let mut f25 = m.rf(s2 + 0x1c);
    let mut manual = false;
    if s6 & 4 != 0 {
        manual = m.r32(s2 + 0x28) != 0
            || 0.0 < m.rf(s5 + 0x1078)
            || m.rf(s2 + 0x40) != 0.0
            || m.rf(s2 + 0x44) != 0.0;
    }
    if manual {
        manual_input(m, s2, s5, s7, f21, &mut f25, &mut f26);
    } else if m.r32(s2 + 0x34) == 4 {
        mode4(m, s2, s3, s4, s5, s6, s7, fp, sp, f21, &mut f25, &mut f26);
    }
    tail(m, s2, s3, s6, sp, f25, f26);
    stack_pop(m, 0x130);
}

/// 0x222364..0x2225d0: stick-driven orbit (mode 4 latch, +0x28 auto-return, +0x40/+0x44 rates)
fn manual_input<M: QMem + ?Sized>(m: &mut M, s2: u32, s5: u32, s7: u32, f21: f32, f25: &mut f32, f26: &mut f32) {
    if m.r32(s2 + 0x34) != 4 {
        let v = m.call(0x001f_bb90, &[3], &[]).0;
        m.call(0x001f_b2f8, &[v, 0], &[]);
    }
    if 0.0 < m.rf(s5 + 0x1078) {
        m.w32(s2 + 0x28, 0);
    }
    if m.r32(s2 + 0x28) != 0 {
        let s0 = if s7 != 0 && m.r32(s7 + 0x10e4) == 4 { 0x002c_4250 } else { 0x002c_4240 };
        let (f13, f14) = (m.rf(s2 + 0x40), m.rf(s2 + 0x2c));
        *f26 = cf(m, 0x0011_d340, &[s0, s2 + 0x40], &[*f26, f13, f14, f21]);
        let (f13, f14) = (m.rf(s2 + 0x44), m.rf(s2 + 0x30));
        *f25 = cf(m, 0x0011_d340, &[s0, s2 + 0x44], &[*f25, f13, f14, f21]);
        if *f26 == m.rf(s2 + 0x2c) && *f25 == m.rf(s2 + 0x30) {
            m.w32(s2 + 0x28, 0);
        }
    } else {
        let mut f1 = m.rf(s5 + 0x1070);
        let f3 = f1.abs();
        let mut f0 = m.rf(s5 + 0x1074);
        let f7 = m.rf(0x002c_41ec) * f21;
        let f5 = m.rf(0x002c_41e4);
        let f2 = m.rf(s2 + 0x40);
        let f4v = f0.abs();
        f1 = f1 * f3;
        let f6 = f2 - f7;
        let f3 = m.rf(0x002c_41e8);
        f0 = f0 * f4v;
        f1 = f1 * f5;
        let f8 = f2 + f7;
        let f2 = f0 * f3;
        if f1 < f6 {
            f1 = f6;
        } else if f8 < f1 {
            f1 = f8;
        }
        let f0 = m.rf(s2 + 0x44);
        m.wf(s2 + 0x40, f1);
        let hi = f0 + f7;
        let lo = f0 - f7;
        let r = if f2 < lo { lo } else if hi < f2 { hi } else { f2 };
        m.wf(s2 + 0x44, r);
        let f13 = m.rf(s5 + 0x1070).abs();
        let f12 = m.rf(s5 + 0x1074).abs();
        let f1 = cf(m, 0x0024_58f0, &[], &[f12, f13]);
        if f1 < fb(0x3e86_0a92) {
            m.w32(s2 + 0x44, 0);
        } else if fb(0x3fa7_8d36) < f1 {
            m.w32(s2 + 0x40, 0);
        }
        let x = *f26 + m.rf(s2 + 0x40) * f21;
        *f26 = ang(m, x);
        let x = *f25 + m.rf(s2 + 0x44) * f21;
        let f0 = ang(m, x);
        *f25 = cf(m, 0x0011_d948, &[0x002c_41b0], &[f0]);
        let a0 = 0x002d_eb30;
        let f2 = m.rf(T + 0x14);
        let f1 = m.rf(a0 + 4);
        if 5.0 < f2 - f1 {
            m.wf(a0 + 4, f2);
            m.call(0x0016_19a0, &[a0, 6, 0], &[]);
        }
    }
    m.w32(s2 + 0x34, 4);
    m.w32(s2 + 0x3c, 1);
    let t = m.rf(T);
    m.wf(s2 + 0x48, t);
}

/// 0x2225e4..0x2230f0: mode 4 (free follow): pitch/yaw goals, track blend gains, track-point latch
fn mode4<M: QMem + ?Sized>(m: &mut M, s2: u32, s3: u32, s4: u32, s5: u32, s6: u32, s7: u32, fp: u32, sp: u32, f21_in: f32, f25: &mut f32, f26: &mut f32) {
    const K: u32 = 0x002a_2da0;
    let x = m.rf(s5 + 0x1040);
    m.wf(sp + 0x44, x);
    let x = m.rf(s5 + 0x103c);
    m.wf(sp + 0x40, x);
    m.w32(sp + 0x48, 0);
    let v = m.rv(sp + 0x40);
    let (n, _) = nrm(m, v, K);
    let d = vsub(m.rv(sp + 0x10), m.rv(CM + 0x90));
    m.wv(sp + 0x40, n);
    m.wv(sp + 0x50, d);
    let z = m.rf(sp + 0x58) * fb(0x3e99_999a);
    m.wf(sp + 0x58, z);
    let v = m.rv(sp + 0x50);
    let (n, len_b) = nrm(m, v, K);
    m.wv(sp + 0x50, n);
    let mut sel = 0xffff_ffffu32;
    if s3 != 0 {
        sel = m.r32(s3 + 0x180);
    }
    if sel == 0xffff_ffff {
        sel = m.r32(s4 + 0x3c);
    }
    if sel == 0 {
        let a = m.rv(0x002a_2dc0);
        let b = m.rv(sp + 0x50);
        let c = [a[1] * b[2] - b[1] * a[2], a[2] * b[0] - b[2] * a[0], a[0] * b[1] - b[0] * a[1], a[3]];
        m.wv(sp + 0x60, c);
    } else if sel == 1 {
        let q = m.rv(sp + 0x50);
        m.wv(sp + 0x60, q);
    }
    let v = m.rv(sp + 0x60);
    let (n60, _) = nrm(m, v, K);
    let f12 = dot3(m.rv(sp + 0x30), n60) / len_b;
    let f13 = m.rf(0x002c_41e0);
    m.wv(sp + 0x60, n60);
    let f0 = cf(m, 0x0011_cfe0, &[], &[f12, f13]);
    let f3 = m.rf(sp + 0x44);
    let mut f5 = m.rf(0x002c_41f0) + f3 * (m.rf(0x002c_41f4) + f3 * m.rf(0x002c_41f8));
    let (f6, f7) = (m.rf(0x002c_4200), m.rf(0x002c_4204));
    let mut f4v = f0 * m.rf(s2 + 0x64);
    if f5 < f6 {
        f5 = f6;
    } else if f7 < f5 {
        f5 = f7;
    }
    let f1 = m.rf(T) - m.rf(s2 + 0x48);
    let mut f3 = m.rf(0x002c_4220) + f1 * (m.rf(0x002c_4224) + f1 * m.rf(0x002c_4228));
    let (f6, f7) = (m.rf(0x002b_bb60), m.rf(0x002b_bb64));
    f4v = f4v * f5;
    if f3 < f6 {
        f3 = f6;
    } else if f7 < f3 {
        f3 = f7;
    }
    f4v = f4v * f3;
    let x = *f26 + f4v * m.rf(T + 4);
    let f29 = ang(m, x);
    let l = len2(m.rv(sp + 0x30)) * fb(0x3ada_740e);
    let f3 = 1.0f32.min(l);
    let f28;
    if 0.1 < f3 {
        let o = m.r32(s2 + 0x14);
        let x = m.rf(0x002c_4260) * f3;
        let y = m.rf(0x002c_4264) * f3;
        let z = m.rf(0x002c_4268);
        m.wf(sp + 0x78, z);
        m.wf(sp + 0x70, x);
        m.wf(sp + 0x74, y);
        let f0 = cf(m, 0x0011_d948, &[o + 0x88], &[*f25]);
        let f14 = m.rf(T + 4);
        f28 = cf(m, 0x0011_d2d0, &[sp + 0x70, 0], &[*f25, f0, f14]);
    } else {
        f28 = *f25;
    }
    if s6 & 3 == 0 {
        *f26 = f29;
        *f25 = f28;
        return;
    }
    let (f23, f24, mut f27);
    // 1 = L29e8 (gain 1, track-1 goals), 3 = attached-to-us (gain 3), 2 = track-2 goals, 0 = L2a40 facing blend
    let mut path = 0;
    if m.r32(s3 + 0x10c) == 0 && m.r32(s3 + 0x12c) == 0 {
        path = 1;
    } else if s7 != 0 && m.r32(s7 + 0x528) == s3 {
        path = 3;
    } else {
        let v1 = m.r32(s2 + 0x34);
        if v1 == 1 {
            path = 1;
        } else if v1 == 2 {
            path = 2;
        }
    }
    match path {
        1 => {
            f27 = 1.0;
            f23 = m.rf(sp + 0xf0);
            f24 = m.rf(sp + 0xf4);
        }
        3 => {
            f27 = 3.0;
            f23 = m.rf(sp + 0xf0);
            f24 = m.rf(sp + 0xf4);
        }
        2 => {
            f23 = m.rf(sp + 0xfc);
            f27 = 1.0;
            f24 = m.rf(sp + 0x100);
        }
        _ => {
            let p = m.call(0x001a_6318, &[fp], &[]).0;
            let q = m.rv(p);
            m.wv(sp + 0x80, q);
            m.w32(sp + 0x88, 0);
            let v = m.rv(sp + 0x80);
            let (n, _) = nrm(m, v, K);
            let a2 = m.rv(sp + 0x20);
            m.wv(sp + 0x80, n);
            let r = m.call(0x001a_6fc0, &[s3, 0], &[a2]).1;
            let f3 = dot3(m.rv(sp + 0x80), r);
            m.wv(sp + 0x20, r);
            let f0 = f3.abs();
            let f2 = f3 * f3;
            let f0 = f0 + f0;
            let f1 = 3.0 - f0;
            f27 = f2 * f1;
            if s6 & 2 == 0 || 0.0 < f3 {
                f23 = m.rf(sp + 0xf0);
                f24 = m.rf(sp + 0xf4);
            } else {
                f23 = m.rf(sp + 0xfc);
                f24 = m.rf(sp + 0x100);
            }
        }
    }
    let mut f21;
    if s7 == 0 {
        f21 = 0.0;
    } else {
        let v1 = m.r32(s7 + 0x4cc);
        let l = len2(m.rv(sp + 0x30));
        if v1 != 0xffff_ffff && m.r32(s7 + 0x4c8) == 0 {
            f21 = 1.0f32.min(l * fb(0x3ba3_d70a));
        } else {
            f21 = 1.0f32.min(l * fb(0x3ada_740e));
        }
    }
    let o = m.r32(s2 + 0x14);
    f21 = f21 * m.rf(o + 0x68);
    if m.r32(sp + 0x128) != 0 {
        let f0 = m.rf(s2 + 0x60) + m.rf(0x002c_41d8);
        let f1 = m.rf(s2 + 0x48) + m.rf(0x002c_41dc);
        let t = m.rf(T);
        let f12 = t - f0;
        let f22 = t - f1;
        let f4v = m.rf(s3 + 0x178);
        let mut f20 = 0.0f32;
        if 0.0 < f4v {
            f20 = cf(m, 0x0011_d948, &[0x002b_bb60], &[f12 / f4v]);
            let d = f22 / m.rf(s3 + 0x178);
            let f0 = cf(m, 0x0011_d948, &[0x002b_bb60], &[d]);
            f20 = f20 * f0;
        } else if 0.0 < f12 && 0.0 < f22 {
            f20 = 1.0;
        }
        let f0 = m.rf(s3 + 0x17c);
        let f1 = f21 * f27;
        f20 = f20 * f0;
        if f1 < f20 {
            f27 = 1.0;
            f21 = f20;
        }
    }
    if m.r32(sp + 0x124) != 0 && m.r32(s2 + 0x3c) == 0 {
        let f1 = m.rf(s3 + 0x170);
        let f2 = if 0.0 < f1 {
            let x = (m.rf(T) - m.rf(s3 + 0x138)) / f1;
            cf(m, 0x0011_d948, &[0x002b_bb60], &[x])
        } else {
            1.0
        };
        let f0 = f2 * m.rf(s3 + 0x174);
        let f1 = f21 * f27;
        if f1 < f0 {
            f27 = 1.0;
            f21 = f0;
        }
    }
    // L2d40
    let x = m.rf(0x002c_4208) * f21;
    let y = m.rf(0x002c_420c) * f21;
    let z = m.rf(0x002c_4210);
    m.wf(sp + 0x88, z);
    m.wf(sp + 0x80, x);
    m.wf(sp + 0x84, y);
    let f0 = ang(m, f23 - *f26);
    let f0 = ang(m, *f26 + f0);
    let dt = m.rf(T + 4);
    let mut f21 = cf(m, 0x0011_d2d0, &[sp + 0x80, 0], &[*f26, f0, dt]);
    let f0 = ang(m, f24 - *f25);
    let f0 = ang(m, *f25 + f0);
    let dt = m.rf(T + 4);
    let mut f22 = cf(m, 0x0011_d2d0, &[sp + 0x80, 0], &[*f25, f0, dt]);
    let f20 = ang(m, f23 - f21).abs();
    let f0 = ang(m, f23 - f29).abs();
    if f0 < f20 {
        f21 = f29;
    }
    let f20 = ang(m, f24 - f22).abs();
    let f0 = ang(m, f24 - f28).abs();
    if f0 < f20 {
        f22 = f28;
    }
    let f20 = ang(m, f29 - *f26);
    let f0 = ang(m, f21 - *f26);
    let w = 1.0 - f27;
    let f20 = *f26 + w * f20;
    *f26 = ang(m, f20 + f27 * f0);
    let f20 = ang(m, f28 - *f25);
    let f0 = ang(m, f22 - *f25);
    let f21 = *f25 + w * f20;
    *f25 = ang(m, f21 + f27 * f0);
    if m.r32(s2 + 0x38) != 0 {
        return;
    }
    let o = m.r32(s2 + 0x10);
    let v1 = m.r32(o + 0x10c);
    if v1 != 0 && m.r32(v1 + 0xe0) == 0 && (m.r32(v1 + 0xe8) as i32) >= 3 {
        return;
    }
    if s6 & 1 != 0 && latch(m, s2, sp, 0x18, 0x50, 0xf0, *f26) && latch(m, s2, sp, 0x1c, 0x54, 0xf4, *f25) {
        m.w32(s2 + 0x3c, 1);
        m.w32(s2 + 0x34, 1);
        return;
    }
    if s6 & 2 != 0 && latch(m, s2, sp, 0x18, 0x58, 0xfc, *f26) && latch(m, s2, sp, 0x1c, 0x5c, 0x100, *f25) {
        m.w32(s2 + 0x34, 2);
        m.w32(s2 + 0x3c, 1);
    }
}

/// goal-crossing test: (cur-prev)*(f-goal) <= 0 or |f-goal| < 1e-4, and |f-goal| < pi/2
fn latch<M: QMem + ?Sized>(m: &mut M, s2: u32, sp: u32, cur: u32, prev: u32, goal: u32, f: f32) -> bool {
    let x = m.rf(s2 + cur) - m.rf(s2 + prev);
    let f20 = ang(m, x);
    let x = f - m.rf(sp + goal);
    let f0 = ang(m, x);
    let f20 = f20 * f0;
    if !(f20 <= 0.0) {
        let x = f - m.rf(sp + goal);
        let f0 = ang(m, x).abs();
        if !(f0 < fb(EPS)) {
            return false;
        }
    }
    let x = f - m.rf(sp + goal);
    let f0 = ang(m, x).abs();
    f0 < fb(HALF_PI)
}

/// 0x2230f4..0x223684: pose from mode, pitch spring, offset, cut placement, write-back
fn tail<M: QMem + ?Sized>(m: &mut M, s2: u32, s3: u32, s6: u32, sp: u32, f25: f32, f26: f32) {
    // f30/f31 stay the caller's callee-saved values when the mode is none of 1/2/4 (unmodelled: 0)
    let (mut f30, mut f31) = (0.0f32, 0.0f32);
    match m.r32(s2 + 0x34) {
        1 => {
            let (a, b, c) = (m.rf(sp + 0xf0), m.rf(sp + 0xf4), m.rf(sp + 0xf8));
            let v0 = m.r32(s2 + 0x14);
            m.wf(sp + 0x108, a);
            m.wf(sp + 0x10c, b);
            m.wf(sp + 0x110, c);
            f30 = m.rf(v0 + 0x5c);
            f31 = m.rf(v0 + 0x58);
        }
        2 => {
            let v0 = m.r32(s2 + 0x14);
            let (a, b, c) = (m.rf(sp + 0xfc), m.rf(sp + 0x100), m.rf(sp + 0x104));
            let f0 = m.rf(v0 + 0x58);
            m.wf(sp + 0x108, a);
            m.wf(sp + 0x10c, b);
            f31 = -f0;
            m.wf(sp + 0x110, c);
            f30 = m.rf(v0 + 0x5c);
        }
        4 => {
            m.wf(sp + 0x108, f26);
            m.wf(sp + 0x10c, f25);
            let f0;
            if s6 & 3 == 0 {
                let v0 = m.r32(s2 + 0x14);
                f0 = m.rf(v0 + 4).abs();
                f30 = m.rf(v0 + 0x5c);
                f31 = m.rf(v0 + 0x58);
            } else {
                let f5 = if s6 & 2 == 0 {
                    1.0
                } else if s6 & 1 == 0 {
                    0.0
                } else {
                    wrap_goals(m, s2, sp);
                    let f3 = m.rf(sp + 0xfc);
                    let f1 = m.rf(s2 + 0x18) - f3;
                    let f0 = m.rf(sp + 0xf0) - f3;
                    let f5 = f1 / f0;
                    let f3 = f5 + f5;
                    let f0 = f5 * f5;
                    let f2 = 3.0 - f3;
                    f0 * f2
                };
                let v0 = m.r32(s2 + 0x14);
                let f1 = 1.0 - f5;
                let f3 = m.rf(v0 + 0x58);
                let a = m.rf(sp + 0xf8);
                let b = m.rf(sp + 0x104);
                let f4v = -f3;
                f30 = m.rf(v0 + 0x5c);
                f0 = f5 * a + f1 * b;
                f31 = f5 * f3 + f1 * f4v;
            }
            m.wf(sp + 0x110, f0);
        }
        _ => {}
    }
    if m.r32(CM + 0x300) != 0 {
        m.wf(s2 + 0x68, 0.0);
        m.wf(s2 + 0x6c, 0.0);
    } else {
        let v1 = m.r32(s2 + 0x14);
        let dt = m.rf(T + 4);
        let f0 = m.rf(v1 + 0x84) * m.rf(0x002c_426c) * dt;
        let f4v = m.rf(v1 + 0x80) * m.rf(0x002c_4270) * dt;
        let f6 = m.rf(s2 + 0x20);
        let f3 = m.rf(sp + 0x110) - f6;
        let f7 = m.rf(s2 + 0x24);
        let f0 = f0 + 1.0;
        let f1 = f30 - f7;
        let f2 = f0 * m.rf(s2 + 0x68) + f3 * f4v;
        let f0 = f0 * m.rf(s2 + 0x6c) + f1 * f4v;
        m.wf(s2 + 0x68, f2);
        m.wf(s2 + 0x6c, f0);
        let f2 = f2 * m.rf(T + 4);
        let f0 = f0 * m.rf(T + 4);
        let f6 = f6 + f2;
        f30 = f7 + f0;
        m.wf(sp + 0x110, f6);
    }
    let (a, b, c) = (m.rf(sp + 0x108), m.rf(sp + 0x10c), m.rf(sp + 0x110));
    let vf01 = m.call(0x0011_e208, &[], &[f4(a), f4(b), f4(c)]).1;
    let vf03 = m.rv(sp + 0x10);
    let vf02 = vadd(vf03, vf01);
    m.wv(sp + 0x40, vf01);
    m.wv(sp + 0x50, vf02);
    if !(s3 != 0 && m.r32(s3 + 0x18c) != 0) {
        if m.r32(CM + 0x300) != 0 {
            m.call(0x0022_3a70, &[s2, sp + 0x50], &[vf03, vf02]);
        } else if m.r32(0x002c_4118) == 0 {
            let a1 = m.rv(s2 + 0x70);
            let r = m.call(0x0014_f110, &[CM], &[a1, vf02]).1;
            m.wv(sp + 0x50, r);
        }
    }
    let vf02 = m.rv(sp + 0x10);
    let vf01 = vsub(m.rv(sp + 0x50), vf02);
    m.call(0x0011_efb0, &[sp + 0x108, sp + 0x10c, sp + 0x110], &[vf01]);
    m.wv(sp + 0x40, vf01);
    let v50 = m.rv(sp + 0x50);
    m.wv(s2 + 0x70, v50);
    let f1 = m.rf(sp + 0x110);
    let f0 = (f1 + m.rf(CM + 0x4c)) / f1;
    let f12 = f31 * m.rf(CM + 0x40);
    let vf03 = vscl(m.rv(sp + 0x40), f0);
    m.wv(sp + 0x60, qf(f0, [v50[2], v50[3]]));
    let vf02 = vadd(m.rv(sp + 0x10), vf03);
    m.wv(sp + 0x50, vf02);
    let f23 = cf(m, 0x0024_4b68, &[], &[f12]);
    let f12 = f30 * m.rf(CM + 0x44);
    let f22 = -cf(m, 0x0024_4b68, &[], &[f12]);
    if m.r32(s2 + 0x38) != 0 {
        let f0;
        if m.r32(CM + 0x300) != 0 {
            f0 = 0.0;
            m.wf(s2 + 0x84, f0);
        } else {
            let q = m.rv(sp + 0x40);
            m.call(0x0011_ef08, &[0, sp + 0x114, sp + 0x118], &[q]);
            m.call(0x0011_bc88, &[CM + 0x60, sp + 0x11c, sp + 0x120], &[]);
            let x = (m.rf(sp + 0x108) + fb(PI)) + f23;
            let f21 = ang(m, x);
            let (a, b) = (-m.rf(sp + 0x118), m.rf(sp + 0x114));
            let f0a = cf(m, 0x0024_58f0, &[], &[a, b]);
            let f20 = ang(m, f0a + f22);
            let x = m.rf(sp + 0x11c) - f21;
            let r = ang(m, x);
            m.wf(s2 + 0x84, r);
            let x = m.rf(sp + 0x120) - f20;
            f0 = ang(m, x);
        }
        m.wf(s2 + 0x88, f0);
        m.wf(s2 + 0x8c, 0.0);
        m.wf(s2 + 0x90, 0.0);
    } else {
        let f7 = m.rf(s2 + 0x84);
        let f3 = m.rf(s2 + 0x8c);
        let f8 = m.rf(T + 4);
        let f6 = m.rf(s2 + 0x88);
        let f2 = m.rf(s2 + 0x90);
        let f0 = f7 * m.rf(0x002c_4274) + f3 * m.rf(0x002c_4278);
        let f1 = f6 * m.rf(0x002c_427c) + f2 * m.rf(0x002c_4280);
        let f3 = f3 + f0 * f8;
        m.wf(s2 + 0x8c, f3);
        let f2 = f2 + f1 * m.rf(T + 4);
        m.wf(s2 + 0x90, f2);
        let f7 = f7 + f3 * m.rf(T + 4);
        m.wf(s2 + 0x84, f7);
        let f6 = f6 + f2 * m.rf(T + 4);
        m.wf(s2 + 0x88, f6);
    }
    let f12 = f23 + m.rf(s2 + 0x84);
    let f13 = f22 + m.rf(s2 + 0x88);
    let (a1, a2) = (m.rv(sp + 0x50), m.rv(sp + 0x10));
    m.call(0x0014_e1a8, &[CM], &[a1, a2, f4(f12), f4(f13)]);
    let mut v = [0.0f32; 7];
    for (k, o) in [0x108u32, 0x10c, 0x110, 0xf0, 0xf4, 0xfc, 0x100].iter().enumerate() {
        v[k] = m.rf(sp + o);
    }
    m.wf(s2 + 0x24, f30);
    for (k, o) in [0x18u32, 0x1c, 0x20, 0x50, 0x54, 0x58, 0x5c].iter().enumerate() {
        m.wf(s2 + o, v[k]);
    }
    m.w32(s2 + 0x38, 0);
}

/// 0x2231dc..0x2232a8: unwrap the two track yaw goals by 2pi around the current yaw
fn wrap_goals<M: QMem + ?Sized>(m: &mut M, s2: u32, sp: u32) {
    let f2 = m.rf(sp + 0xf0);
    let f0 = m.rf(s2 + 0x18);
    let f1 = m.rf(sp + 0xfc);
    if f0 < f2 {
        if f0 < f1 {
            if f1 < f2 {
                m.wf(sp + 0xf0, f2 - fb(TWO_PI));
            } else {
                m.wf(sp + 0xfc, f1 - fb(TWO_PI));
            }
        }
    } else if f2 < f0 && f1 < f0 {
        if f2 < f1 {
            m.wf(sp + 0xf0, f2 + fb(TWO_PI));
        } else {
            m.wf(sp + 0xfc, f1 + fb(TWO_PI));
        }
    }
}
