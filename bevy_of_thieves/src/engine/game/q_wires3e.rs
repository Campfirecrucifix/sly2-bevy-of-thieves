//! WIRE-S3: 1:1 QMem ports of live-frontier natives (shard 5/6/7).
#![allow(unused)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

const T: u32 = 0x002d_ded0; // g_clock t (+4 = dt)
const CM: u32 = 0x002d_df40; // camera manager
const GS: u32 = 0x003d_d490; // game state / world

fn dist3(a: V4, b: V4) -> f32 {
    let (x, y, z) = (a[0] - b[0], a[1] - b[1], a[2] - b[2]);
    (x * x + y * y + z * z).sqrt()
}

fn sd<M: QMem + ?Sized>(m: &mut M, a: u32, v: u32) {
    m.w32(a, v);
    m.w32(a + 4, ((v as i32) >> 31) as u32);
}

/// FUN_00150160: per-frame camera/look update (+0x20 blend, +0x18/+0x1c rates, +0x10/+0x14 angles), CM push.
pub fn ee_sub_150160<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    let o = a0;
    let s1 = a2;
    m.call(0x0014_fd40, &[a0, a1, a2], &[]);
    let dt = m.rf(T + 4);
    let mut f5 = m.rf(o + 0x20);
    let f3 = m.rf(s1 + 0x1074);
    let c0 = m.rf(0x002b_d980);
    let c1 = m.rf(0x002b_d984);
    let c2 = m.rf(0x002b_d988);
    let f0 = (c0 + f5 * (c1 + f5 * c2)) * f3 * f3.abs();
    m.wf(o + 0x24, f0);
    f5 += f0 * dt;
    if f5 < 0.0 {
        f5 = 0.0;
    } else if 1.0 < f5 {
        f5 = 1.0;
    }
    let f12 = m.rf(o + 0x24);
    m.wf(o + 0x20, f5);
    if f12 != 0.0 && 0.0 < f5 && f5 < 1.0 {
        let s0 = m.r32(o + 0x34);
        let r = m.call(0x0011_d948, &[0x002b_bb68], &[f4(f12.abs())]).1[0];
        m.call(0x001f_b2c0, &[s0], &[f4(r)]);
        let x = m.rf(o + 0x24).abs() * 0.5 + 0.5;
        let s0 = m.r32(o + 0x34);
        let r = m.call(0x0011_d948, &[0x002b_bb60], &[f4(x)]).1[0];
        m.call(0x001f_b1f8, &[s0], &[f4(r)]);
        let p = m.r32(o + 0x34);
        if (m.r16(p + 0x4e) as i16) > 0 {
            m.call(0x001f_b060, &[p], &[]);
        }
    } else {
        let p = m.r32(o + 0x34);
        if (m.r16(p + 0x4e) as i16) <= 0 {
            m.call(0x001f_b018, &[p], &[]);
        }
    }
    let f0 = 1.0 - m.rf(o + 0x20);
    let x = (1.0 - f0) * m.rf(o + 0x2c) + f0 * m.rf(o + 0x30);
    m.call(0x0014_f5a0, &[CM], &[f4(x)]);
    let f3 = m.rf(s1 + 0x1040);
    let f2 = m.rf(s1 + 0x103c);
    let f5c = m.rf(0x002b_d990);
    let f4c = m.rf(0x002b_d994);
    let mut f20 = -f3.abs() * f3 * f4c;
    let mut f22 = -f2.abs() * f2 * f5c;
    let sp = stack_push(m, 0x20);
    let k = m.r32(GS + 0x3b4c);
    m.call(0x0025_81c0, &[sp, k, 0x003d_4a60], &[]);
    let s0 = m.r32(sp + 4);
    m.call(0x0012_9ec8, &[sp], &[]);
    if s0 != 0 {
        f20 = -f20;
    }
    let dt = m.rf(T + 4);
    let k8 = m.rf(0x002b_d998);
    let t = f22 - m.rf(o + 0x18);
    let r = m.call(0x0011_cfe0, &[], &[f4(t), f4(k8 * dt)]).1[0];
    let k9 = m.rf(0x002b_d99c);
    let a = f20 - m.rf(o + 0x1c);
    let dt = m.rf(T + 4);
    f22 = m.rf(o + 0x18) + r;
    let r = m.call(0x0011_cfe0, &[], &[f4(a), f4(k9 * dt)]).1[0];
    let f21 = m.rf(T + 4) * 0.5;
    f20 = m.rf(o + 0x1c) + r;
    let tb = 0x002b_d9a0;
    let (x, y) = (m.rf(o + 0x10), m.rf(o + 0x3c));
    m.call(0x0011_d018, &[tb, sp + 0x10], &[f4(x), f4(y), f4(f21)]);
    let (x, y) = (m.rf(o + 0x10), m.rf(o + 0x40));
    m.call(0x0011_d018, &[tb, sp + 0x14], &[f4(x), f4(y), f4(f21)]);
    let (lo, hi) = (m.rf(sp + 0x10), m.rf(sp + 0x14));
    let v = if f22 < lo { lo } else if hi < f22 { hi } else { f22 };
    m.wf(o + 0x18, v);
    let (x, y) = (m.rf(o + 0x14), m.rf(o + 0x44));
    m.call(0x0011_d018, &[tb, sp + 0x10], &[f4(x), f4(y), f4(f21)]);
    let (x, y) = (m.rf(o + 0x14), m.rf(o + 0x48));
    m.call(0x0011_d018, &[tb, sp + 0x14], &[f4(x), f4(y), f4(f21)]);
    let (lo, hi) = (m.rf(sp + 0x10), m.rf(sp + 0x14));
    let f12 = if f20 < lo { lo } else if hi < f20 { hi } else { f20 };
    m.wf(o + 0x1c, f12);
    stack_pop(m, 0x20);
    let c = m.rf(CM + 0x20);
    let k = (1.0 - c) * m.rf(0x002b_d9b0) + c * m.rf(0x002b_d9b4);
    let dt = m.rf(T + 4);
    let span = m.rf(o + 0x40) - m.rf(o + 0x3c);
    let n10 = m.rf(o + 0x10) + k * m.rf(o + 0x18) * dt;
    m.wf(o + 0x10, n10);
    let dt = m.rf(T + 4);
    let n14 = m.rf(o + 0x14) + k * f12 * dt;
    m.wf(o + 0x14, n14);
    if f32::from_bits(0x40c9_0fdb) <= span {
        let r = m.call(0x0011_cf60, &[], &[f4(n10)]).1[0];
        m.wf(o + 0x10, r);
    }
    let q0 = m.call(0x0014_fec8, &[o], &[]).1;
    let q1 = m.call(0x0014_fe58, &[o], &[]).1;
    m.call(0x0014_e1a8, &[CM], &[q0, q1, f4(0.0), f4(0.0)]);
    let t = m.rf(T);
    m.wf(GS + 0x4750, t);
}

/// FUN_00232838: queue a SIF/RPC packet (cmd a0, len a1, data a2, cb a3, arg t0) into the double buffer.
pub fn ee_sub_232838<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, t0: u32) -> u32 {
    let (s7, s0, s5) = (a0, a1, a2);
    let mut flag = 0u32;
    let mut s1 = s0.wrapping_add(4);
    if m.r32(0x002c_4684) == 0 && m.r32(0x002c_4640) == 0 && s0 == 0 && a3 == 0 {
        m.call(0x0023_1f38, &[0x0044_3840, 1], &[]);
        while m.call(0x0023_7488, &[0x0044_3800], &[]).0 != 0 {
            if m.r32(0x002c_46b8) == 0 {
                m.call(0x0021_ec18, &[0x002a_a1c8], &[]);
            }
            m.call(0x0023_1c48, &[], &[]);
            m.call(0x0023_4ca0, &[0], &[]);
        }
        return m.call(0x0023_7288, &[0x0044_3800, s7, 1, 0, 0, 0x0044_3840, 0xc, 0, 0], &[]).0;
    }
    if s1 & 3 != 0 {
        s1 = s0.wrapping_add(8).wrapping_sub(((s1 as i32) % 4) as u32);
    }
    if (s1 as i32) >= 0x201 {
        m.call(0x0021_ec18, &[0x002a_a5e8], &[]);
        loop {}
    }
    let mut s2 = 0u32;
    loop {
        let idx = m.r32(0x002c_4680);
        let buf = m.r32(0x002c_4660 + idx.wrapping_mul(4));
        if m.r32(buf) != 0x100 && (m.r32(0x002c_4668 + idx.wrapping_mul(4)) as i32) >= (s1 as i32) {
            break;
        }
        if m.r32(0x002c_4684) != 0 {
            m.w32(0x002c_4684, 0);
            flag = 1;
        }
        m.call(0x0023_1c48, &[], &[]);
        if s2 == 1 && m.r32(0x002c_46b8) == 0 {
            let i = m.r32(0x002c_4680);
            let b = m.r32(0x002c_4660 + i.wrapping_mul(4));
            let n = m.r32(b);
            m.call(0x0021_ec18, &[0x002a_a610, i, n], &[]);
        }
        s2 = s2.wrapping_add(1);
    }
    if s2 != 0 && m.r32(0x002c_46b8) == 0 {
        m.call(0x0021_ec18, &[0x002a_a670, s2], &[]);
    }
    if flag != 0 {
        m.w32(0x002c_4684, 1);
    }
    let idx = m.r32(0x002c_4680);
    let rem = m.r32(0x002c_4668 + idx.wrapping_mul(4)).wrapping_sub(0x1000);
    let mut p = m.r32(0x002c_4660 + idx.wrapping_mul(4)).wrapping_sub(rem);
    m.w16(p, s7 as u16);
    m.w16(p + 2, s0 as u16);
    p += 4;
    if s7 == 0x68 {
        for i in 0..12 {
            let b = m.r8(s5 + i);
            m.w8(p + i, b);
        }
        let (n, src) = (m.r32(s5 + 8), m.r32(s5 + 0xc));
        m.call(0x0023_320c, &[p + 12, src, n], &[]);
    } else {
        let mut i = 0u32;
        while (i as i32) < (s0 as i32) {
            let b = m.r8(s5 + i);
            m.w8(p + i, b);
            i += 1;
        }
    }
    let idx = m.r32(0x002c_4680);
    let ra = 0x002c_4668 + idx.wrapping_mul(4);
    let rb = 0x002c_4660 + idx.wrapping_mul(4);
    let rc = 0x002c_4670 + idx.wrapping_mul(4);
    let v = m.r32(ra).wrapping_sub(s1);
    m.w32(ra, v);
    let b = m.r32(rb);
    let n = m.r32(b);
    let e = (n << 4).wrapping_add(m.r32(rc));
    m.w32(e, a3);
    let b = m.r32(rb);
    let n = m.r32(b);
    let e = (n << 4).wrapping_add(m.r32(rc));
    sd(m, e + 8, t0);
    m.call(0x0023_2bb8, &[], &[]).0
}

/// FUN_0019ae00: sweep a path p0->p1 (ctrl q0/q1) in 0.1 steps up to f12 checking clearance; 1 = blocked/far.
pub fn ee_sub_19ae00<M: QMem + ?Sized>(m: &mut M, a1: u32, a2: u32, t3: u32, p0: V4, q0: V4, p1: V4, q1: V4, f12: f32) -> u32 {
    let sp = stack_push(m, 0x80);
    let fr = sp + 0x70;
    m.call(0x001a_e478, &[fr], &[]);
    let (lo, hi) = if t3 != 0 {
        let s = m.rv(t3);
        let mut lo = s;
        let mut hi = [0.0; 4];
        for i in 0..3 {
            hi[i] = p0[i].max(p1[i].max(s[i]));
            lo[i] = p0[i].min(p1[i].min(s[i]));
        }
        (lo, hi)
    } else {
        (p0, p1)
    };
    m.call(0x0020_faa0, &[GS, a1, a2, sp + 0x74, sp + 0x78], &[lo, hi]);
    let mut f21 = 0.0f32;
    let mut prev = [0.0f32; 4];
    let res;
    loop {
        m.call(0x0011_7120, &[sp, 0, 0], &[p0, q0, p1, q1, f4(f12), f4(f21), f4(1.0)]);
        let cur = m.rv(sp);
        if f21 != 0.0 {
            let (w0, w1) = (m.r32(sp + 0x74), m.r32(sp + 0x78));
            if m.call(0x0014_cc58, &[0, w0, w1, sp + 0x10], &[prev, cur]).0 != 0 {
                let h = m.rv(sp + 0x10);
                if 100.0 < dist3(h, p1) {
                    res = 1;
                    break;
                }
            }
        }
        prev = cur;
        if f21 == f12 {
            if t3 == 0 {
                res = 0;
                break;
            }
            let s = m.rv(t3);
            let (w0, w1) = (m.r32(sp + 0x74), m.r32(sp + 0x78));
            if m.call(0x0014_cc58, &[0, w0, w1, sp + 0x10], &[p1, s]).0 == 0 {
                res = 0;
                break;
            }
            if m.rf(sp + 0x54) == 1.0 {
                res = 0;
                break;
            }
            let h = m.rv(sp + 0x10);
            res = if 50.0 < dist3(h, s) { 1 } else { 0 };
            break;
        }
        f21 = f12.min(f21 + f32::from_bits(0x3dcc_cccd));
    }
    m.call(0x001a_e4a0, &[fr, 2], &[]);
    stack_pop(m, 0x80);
    res
}

/// FUN_00127838: build a value from token a1 (type 0x18 / 0x1f dispatch on subtype via jump table) into a0.
pub fn ee_sub_127838<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let s0 = a0;
    match m.r32(a1 + 8) {
        0x18 => {
            m.call(0x0012_6438, &[s0, a1], &[]);
        }
        0x1f => {
            let a3 = m.r32(a1 + 0xc);
            let f = match m.r32(a3 + 8) {
                1 => 0x0012_6490,
                2 => 0x0012_6540,
                5 => 0x0012_65f0,
                6 => 0x0012_66c8,
                7 => 0x0012_6758,
                8 => 0x0012_6840,
                9 => 0x0012_6928,
                10 => 0x0012_6a18,
                12 => 0x0012_6ba0,
                13 => 0x0012_6db8,
                14 => 0x0012_6f78,
                15 => 0x0012_7068,
                16 => 0x0012_7228,
                17 => {
                    m.call(0x0012_a0b0, &[s0, u32::MAX], &[]);
                    return s0;
                }
                18 => {
                    let p = m.r32(a3 + 0x10);
                    m.call(0x0012_91a8, &[s0, p.wrapping_add(8)], &[]);
                    return s0;
                }
                _ => 0x0012_72e0,
            };
            m.call(f, &[s0, a1], &[]);
        }
        _ => {
            m.call(0x0012_91a8, &[s0, a1.wrapping_add(8)], &[]);
        }
    }
    s0
}

/// FUN_00171e00: if a1 == 0xffff && a0: bump six pool refcounts, first ref -> FUN_0011cca8 pool init.
pub fn ee_sub_171e00<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if a1 != 0xffff || a0 == 0 {
        return;
    }
    const P: [(u32, u32, u32); 6] = [
        (0x0045_20a8, 0x14, 0x2000),
        (0x0045_20d0, 0x18, 0x800),
        (0x0045_20f8, 0x28, 0x200),
        (0x0045_2120, 0x1c, 0x800),
        (0x0045_2148, 0xc, 0xc8),
        (0x0045_2170, 0x14, 0x100),
    ];
    for (i, (b, sz, n)) in P.into_iter().enumerate() {
        let c = 0x002d_d2e0 + 4 * i as u32;
        let v = m.r32(c).wrapping_add(1);
        m.w32(c, v);
        if v == 1 {
            m.call(0x0011_cca8, &[b, sz, n, 0x10], &[]);
        }
    }
}

/// FUN_00218790: 4-lane byte blend: t0[i] = clamp(sa*sc)*127+128, t1[i] = clamp(sa*(d-127)+(b-127))+127.
pub fn ee_sub_218790<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, t0: u32, t1: u32) {
    let k = f32::from_bits(0x3c01_0204);
    for i in 0..4u32 {
        let f1 = (m.r8(a0 + i) as i32 - 0x7f) as f32 * k;
        let f0 = (m.r8(a2 + i) as i32 - 0x7f) as f32 * k;
        let f3 = m.r8(a3 + i) as i32 as f32 - 127.0;
        let f2 = m.r8(a1 + i) as i32 as f32 - 127.0;
        let f1 = f1.min(1.0).max(-1.0);
        let f0 = f0.min(1.0).max(-1.0);
        let g0 = (f1 * f0).min(1.0).max(-1.0) * 127.0 + 128.0;
        let g1 = (f1 * f3 + f2).min(128.0).max(-127.0) + 127.0;
        m.w8(t0 + i, g0 as i32 as u8);
        m.w8(t1 + i, g1 as i32 as u8);
    }
}

/// FUN_00208d70: behaviour bits {0xb,0x1d,7,5,6,0x10,1,0x16,0x2a} & +0x2c0 == 0 -> +0xe64, else 0.
pub fn ee_sub_208d70<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let mut s = 0u64;
    for b in [0xb, 0x1d, 7, 5, 6, 0x10, 1, 0x16, 0x2a] {
        s |= m.call(0x0020_7a20, &[b], &[]).0 as u64;
    }
    if s & m.r64(a0 + 0x2c0) != 0 {
        0
    } else {
        m.r32(a0 + 0xe64)
    }
}

/// FUN_001d8a38: varargs format (FUN_001d87c0 / FUN_001d7ed8) into a string appended to a0; returns a0.
pub fn ee_sub_1d8a38<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, t0: u32, t1: u32, t2: u32, t3: u32, f12: f32, f14: f32, f16: f32, f18: f32) -> u32 {
    let sp = stack_push(m, 0x100);
    m.w32(sp, u32::MAX);
    for (i, v) in [a2, a3, t0, t1, t2, t3].into_iter().enumerate() {
        sd(m, sp + 0xd0 + 8 * i as u32, v);
    }
    for (i, f) in [f12, f14, f16, f18].into_iter().enumerate() {
        m.wf(sp + 0xc0 + 4 * i as u32, f);
    }
    m.call(0x001a_e478, &[sp + 0x10], &[]);
    m.call(0x001d_87c0, &[a1, sp + 0xd0, sp + 0x20], &[]);
    m.call(0x001d_7ed8, &[sp + 0x30, 0x003e_0ea8, sp + 0x20, 0x003d_4a60], &[]);
    m.call(0x0012_9658, &[sp, sp + 0x30], &[]);
    m.call(0x0012_9ec8, &[sp + 0x30], &[]);
    m.call(0x0012_91a8, &[a0, sp], &[]);
    m.call(0x001a_e4a0, &[sp + 0x10, 2], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x100);
    a0
}

/// FUN_00151018: +0x30.xyz approx-equal (1e-4) to FUN_001507f8(a0).xyz.
pub fn ee_sub_151018<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let s1 = m.rv(a0 + 0x30);
    let s0 = m.call(0x0015_07f8, &[a0], &[]).1;
    let eps = f4(f32::from_bits(0x38d1_b717));
    for i in 0..3 {
        if m.call(0x0011_d4d0, &[], &[f4(s1[i]), f4(s0[i]), eps]).0 == 0 {
            return 0;
        }
    }
    1
}

/// FUN_001d5340: if +8 && *a1 > 0 && FUN_001d55b8: start (+0x14 = f12, +0xc = a1, +0x10 = 0, +8 = -1,
/// +0x18 = a1[+8]) then FUN_001d53c0(a0, 2).
pub fn ee_sub_1d5340<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, f12: f32) {
    if m.r32(a0 + 8) == 0 || (m.r32(a1) as i32) <= 0 {
        return;
    }
    if m.call(0x001d_55b8, &[a0, a1], &[f4(f12)]).0 == 0 {
        return;
    }
    m.wf(a0 + 0x14, f12);
    m.w32(a0 + 0xc, a1);
    m.w32(a0 + 0x10, 0);
    let f = m.r32(a1 + 8);
    m.w32(a0 + 8, u32::MAX);
    m.w32(a0 + 0x18, f);
    m.call(0x001d_53c0, &[a0, 2], &[]);
}

/// FUN_001fe0e8: slot (s16 a1[+2]) != -1 with obj[+0x22c + 4slot]: slot 3 -> FUN_001f80d0(+0x238, a1, a0,
/// -1, f a1[+4]); else a1[+4] != -1 -> FUN_001f0660(that, a1[+4]).
pub fn ee_sub_1fe0e8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let v1 = m.r16(a1 + 2) as i16 as i32;
    if v1 == -1 {
        return;
    }
    let p = m.r32(a0.wrapping_add((v1 as u32) << 2) + 0x22c);
    if p == 0 {
        return;
    }
    if v1 == 3 {
        let q = m.r32(a0 + 0x238);
        let f = m.rf(a1 + 4);
        m.call(0x001f_80d0, &[q, a1, a0, u32::MAX], &[f4(f)]);
    } else {
        let k = m.r32(a1 + 4);
        if k != u32::MAX {
            m.call(0x001f_0660, &[p, k], &[]);
        }
    }
}

/// FUN_0014fde0: FUN_0014fca8(a0); vtable 0x2cd4e0; +0x28 = 7.0; +0x34 = +0x38 = 0; returns a0.
pub fn ee_sub_14fde0<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    m.call(0x0014_fca8, &[a0], &[]);
    m.w32(a0 + 0xc, 0x002c_d4e0);
    m.wf(a0 + 0x28, 7.0);
    m.w32(a0 + 0x34, 0);
    m.w32(a0 + 0x38, 0);
    a0
}

/// FUN_00150e20: +0x10 = a1; +0x40 != 0 -> FUN_00150ef0(a0); +0x40 = 0.
pub fn ee_sub_150e20<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.w32(a0 + 0x10, a1);
    if m.r32(a0 + 0x40) != 0 {
        m.call(0x0015_0ef0, &[a0], &[]);
    }
    m.w32(a0 + 0x40, 0);
}

/// FUN_001fe5e0: +0x68 == 3.
pub fn ee_sub_1fe5e0<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    (m.r32(a0 + 0x68) == 3) as u32
}

/// FUN_00113b00: returns 0x2bac14.
pub fn ee_sub_113b00<M: QMem + ?Sized>(m: &mut M) -> u32 {
    0x002b_ac14
}

/// FUN_00113d10: returns 0x2baea8.
pub fn ee_sub_113d10<M: QMem + ?Sized>(m: &mut M) -> u32 {
    0x002b_aea8
}

/// FUN_00114060: returns 0x2bb2cc.
pub fn ee_sub_114060<M: QMem + ?Sized>(m: &mut M) -> u32 {
    0x002b_b2cc
}

/// FUN_00114150: returns 0x2bb3f8.
pub fn ee_sub_114150<M: QMem + ?Sized>(m: &mut M) -> u32 {
    0x002b_b3f8
}

/// FUN_001144d0: returns 0x2bb858.
pub fn ee_sub_1144d0<M: QMem + ?Sized>(m: &mut M) -> u32 {
    0x002b_b858
}

/// FUN_001fdec0: no-op.
pub fn ee_sub_1fdec0<M: QMem + ?Sized>(m: &mut M) {}
