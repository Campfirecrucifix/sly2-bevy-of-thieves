//! WIRE-W1H12: 1:1 queue ports (against `QMem`) of a batch of live-frontier natives, ported from the EE asm;
//! callees go through `m.call` / `vslot`. QMem convention: integer / pointer args in `a`, quadword and float
//! args in `v`.
#![allow(unused_variables, unused_mut, unused_assignments, clippy::all)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

const GS: u32 = 0x003d_d490; // game state / world

/// FUN_001a6ef0: empty.
pub fn ee_sub_1a6ef0<M: QMem + ?Sized>(m: &mut M) {}

/// FUN_00135e30: empty.
pub fn ee_sub_135e30<M: QMem + ?Sized>(m: &mut M) {}

/// FUN_00258f30: returns 0x2bad2c.
pub fn ee_sub_258f30<M: QMem + ?Sized>(m: &mut M) -> u32 {
    0x002b_ad2c
}

/// FUN_001fd070(o, p): (p - o[0xfc]) >> 2 (arithmetic).
pub fn ee_sub_1fd070<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> i32 {
    let v1 = m.r32(a0 + 0xfc);
    (a1.wrapping_sub(v1) as i32) >> 2
}

/// FUN_00256ce8(dst, f12): sq of mfc1(f12) (sign-extended 64-bit; upper 64 bits of v1 are not defined by
/// this function, written as 0).
pub fn ee_sub_256ce8<M: QMem + ?Sized>(m: &mut M, a0: u32, f12: f32) {
    let b = f12.to_bits();
    m.w32(a0, b);
    m.w32(a0 + 4, ((b as i32) >> 31) as u32);
    m.w32(a0 + 8, 0);
    m.w32(a0 + 12, 0);
}

/// FUN_00118458(a, b): FUN_00117968(a, 0x40, b).
pub fn ee_sub_118458<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.call(0x0011_7968, &[a0, 0x40, a1], &[]);
}

/// FUN_00208700(o, x): push x onto the 3-slot list at +0x398 (count +0x394).
pub fn ee_sub_208700<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let n = m.r32(a0 + 0x394);
    if n < 3 {
        m.w32(a0 + 0x394, n.wrapping_add(1));
        m.w32((n << 2).wrapping_add(a0) + 0x398, a1);
    }
}

/// FUN_00232320(a, b, c): FUN_00232838(0x5c, 0x30, a, b, c).
pub fn ee_sub_232320<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    m.call(0x0023_2838, &[0x5c, 0x30, a0, a1, a2], &[]);
}

/// FUN_00119390(dst, n, p): plane = n.xyzw with w = n.x*p.x + n.y*p.y + n.z*p.z.
pub fn ee_sub_119390<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: V4, a2: V4) {
    let (x, y, z) = (a1[0] * a2[0], a1[1] * a2[1], a1[2] * a2[2]);
    let acc = x + y;
    let d = acc + 1.0 * z;
    m.wv(a0, a1);
    m.wf(a0 + 0xc, d);
}

/// FUN_001e0290(a, x): FUN_001e01f8(a, 0, 1)->+0x10 = x.
pub fn ee_sub_1e0290<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let p = m.call(0x001e_01f8, &[a0, 0, 1], &[]).0;
    m.w32(p + 0x10, a1);
}

/// FUN_001ade38(o): o[0x24] -> vslot 48 (-1).
pub fn ee_sub_1ade38<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let p = m.r32(a0 + 0x24);
    if p != 0 {
        vslot(m, p, 48, &[0xffff_ffff], &[]);
    }
}

/// FUN_00172760(o): quadword +0x20 of o[0x44] when its state == 4 and +0x44 == o[0x10], else +0x30.
pub fn ee_sub_172760<M: QMem + ?Sized>(m: &mut M, a0: u32) -> V4 {
    let a1 = m.r32(a0 + 0x44);
    if m.r32(a1 + 0x54) == 4 && m.r32(a1 + 0x44) == m.r32(a0 + 0x10) {
        return m.rv(a1 + 0x20);
    }
    let v1 = m.r32(a0 + 0x44);
    m.rv(v1 + 0x30)
}

/// FUN_001f3590(o): reset the +0xe8 block.
pub fn ee_sub_1f3590<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let v = m.r32(a0 + 0xe8);
    m.w32(v + 0x80, 0);
    m.w32(v + 0x88, 0xc4da_c000);
    m.w32(v + 0x84, 0);
    let v = m.r32(a0 + 0xe8);
    m.w32(v + 0xc0, 3);
    let v = m.r32(a0 + 0xe8);
    m.w32(v + 0xa0, 3);
}

/// FUN_001f8240(id): find the node with +0xc == id in the list at 0x3e145c (next +0x20).
pub fn ee_sub_1f8240<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let mut p = m.r32(0x003e_145c);
    while p != 0 {
        if m.r32(p + 0xc) == a0 {
            return p;
        }
        p = m.r32(p + 0x20);
    }
    0
}

/// FUN_001bc530(o, i): i % o[0xe8], wrapped (negative -> +n-1; n-1 -> 0).
pub fn ee_sub_1bc530<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: i32) -> i32 {
    let n = m.r32(a0 + 0xe8) as i32;
    if n == 0 {
        panic!("FUN_001bc530: break 7 (division by zero)");
    }
    let r = a1.wrapping_rem(n);
    if r < 0 {
        r.wrapping_sub(1).wrapping_add(n)
    } else if r == n.wrapping_sub(1) {
        0
    } else {
        r
    }
}

/// FUN_0019c828(o): o[0x1c] == 0x11 ? FUN_00195a00(o, 3, 0, 0) : FUN_00195fb8(o, 4, 0).
pub fn ee_sub_19c828<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    if m.r32(a0 + 0x1c) == 0x11 {
        m.call(0x0019_5a00, &[a0, 3, 0, 0], &[]);
    } else {
        m.call(0x0019_5fb8, &[a0, 4, 0], &[]);
    }
}

/// FUN_001af490(o): when o[0x1e8] == 7: FUN_001aeaa8(o, 3); o[0x1ec] = o[0x24] - 1.
pub fn ee_sub_1af490<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    if m.r32(a0 + 0x1e8) == 7 {
        m.call(0x001a_eaa8, &[a0, 3], &[]);
        let v = m.r32(a0 + 0x24).wrapping_sub(1);
        m.w32(a0 + 0x1ec, v);
    }
}

/// FUN_0018f908(a, b, out): FUN_00137a30(a, 0xc, 3, b, 0, 0, 0, &tmp); *out = tmp.
pub fn ee_sub_18f908<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    let s = stack_push(m, 0x10);
    m.call(0x0013_7a30, &[a0, 0xc, 3, a1, 0, 0, 0, s], &[]);
    let v = m.r32(s);
    if a2 != 0 {
        m.w32(a2, v);
    }
    stack_pop(m, 0x10);
}

/// FUN_001637e8(o): o[0x50] == 0 && o[0xf4] not 2/3 && FUN_0020e040(GS) == 0.
pub fn ee_sub_1637e8<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    if m.r32(a0 + 0x50) != 0 {
        return 0;
    }
    let s = m.r32(a0 + 0xf4);
    if s == 2 || s == 3 {
        return 0;
    }
    (m.call(0x0020_e040, &[GS], &[]).0 == 0) as u32
}

/// FUN_001d14c0(o).
pub fn ee_sub_1d14c0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let v = m.r32(a0 + 0x55c);
    m.w32(a0 + 0x560, v);
    m.call(0x001d_4c88, &[a0], &[]);
    let p = m.r32(a0 + 0x570);
    if p != 0 {
        let v1 = m.r32(p + 0x24);
        let v2 = m.r32(v1 + 0x54);
        let a1 = m.r32(v2);
        m.call(0x001f_0948, &[p, a1], &[]);
    }
    m.call(0x001d_4940, &[a0, 0], &[]);
}

/// FUN_002550a0(p): *p = 0; FUN_00254f00(p, 0x2dd280); p[1] bit31 = DAT_002dd598 bit0.
pub fn ee_sub_2550a0<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    m.w32(a0, 0);
    m.call(0x0025_4f00, &[a0, 0x002d_d280], &[]);
    let a = m.r32(a0 + 4);
    let b = m.r32(0x002d_d598);
    m.w32(a0 + 4, (a & 0x7fff_ffff) | (b << 31));
    a0
}

/// FUN_001ea3b0(tbl, t): first range [lo, hi] at +0x10 (count +0xc) containing t.
pub fn ee_sub_1ea3b0<M: QMem + ?Sized>(m: &mut M, a0: u32, f12: f32) -> u32 {
    let n = m.r32(a0 + 0xc) as i32;
    let mut p = a0 + 0x10;
    let mut i: i32 = 0;
    if n <= 0 {
        return 0;
    }
    loop {
        let lo = m.rf(p);
        i = i.wrapping_add(1);
        if lo <= f12 {
            let hi = m.rf(p + 4);
            if f12 <= hi {
                return p;
            }
        }
        if !(i < n) {
            return 0;
        }
        p = p.wrapping_add(8);
    }
}

/// FUN_0012b6d8(curve, t): key index (1..n-1) whose time (+0x3c, stride 0x30) bounds t.
pub fn ee_sub_12b6d8<M: QMem + ?Sized>(m: &mut M, a0: u32, f12: f32) -> i32 {
    let v1 = (m.r32(a0 + 4) as i32).wrapping_sub(1);
    let mut a1: i32 = 1;
    if !(a1 < v1) {
        return a1;
    }
    let mut p = m.r32(a0 + 8);
    let f0 = m.rf(p + 0x3c);
    p = p.wrapping_add(0x3c);
    if f12 < f0 {
        return a1;
    }
    a1 += 1;
    loop {
        if !(a1 < v1) {
            return a1;
        }
        p = p.wrapping_add(0x30);
        let f0 = m.rf(p);
        if f12 < f0 {
            return a1;
        }
        a1 = a1.wrapping_add(1);
    }
}

/// FUN_002036e8(o, x).
pub fn ee_sub_2036e8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let h = m.r32(a0 + 0xec);
    if h == 0 {
        m.w32(a0 + 0xf0, a1);
        return;
    }
    m.call(0x0016_4640, &[h, 2], &[]);
    let s0 = m.call(0x001c_4840, &[GS], &[]).0;
    if s0 == 0 {
        return;
    }
    if m.call(0x0013_2b10, &[s0, 0x34], &[]).0 != 0 {
        m.call(0x0019_d948, &[s0.wrapping_add(0xf24)], &[]);
    }
}

/// FUN_001c41b8(o): query vslot 2 (msg 0xd) into {0x17, 1}; result word +4 if the tag stayed 0x17, else 1.
pub fn ee_sub_1c41b8<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let s = stack_push(m, 0x10);
    m.w32(s + 4, 1);
    m.w32(s, 0x17);
    vslot(m, a0, 2, &[0xd, 0, 0, 0, s], &[]);
    let v0 = m.r32(s);
    let v1 = m.r32(s + 4);
    let r = if v0 ^ 0x17 == 0 { v1 } else { 1 };
    m.call(0x0012_9ec8, &[s], &[]);
    stack_pop(m, 0x10);
    r
}

/// FUN_001b5038(o).
pub fn ee_sub_1b5038<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    if m.call(0x0014_43d8, &[a0], &[]).0 == 0 {
        return;
    }
    let s1 = a0.wrapping_add(0xf00);
    if m.r32(a0 + 0xf04) == 1 && m.r32(a0 + 0xf20) == 0 {
        let p = m.r32(a0 + 0x150);
        if m.call(0x001c_46c0, &[p, 0x4f0], &[f4(f32::from_bits(0x3e80_0000))]).0 != 0 {
            m.call(0x0019_cce8, &[s1], &[]);
        }
    }
    m.call(0x0019_d330, &[s1], &[]);
}

/// FUN_0018b2c0(o).
pub fn ee_sub_18b2c0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    if m.call(0x0014_43d8, &[a0], &[]).0 == 0 {
        return;
    }
    let s1 = a0.wrapping_add(0xf00);
    if m.r32(a0 + 0xf04) == 1 && m.r32(a0 + 0xf20) == 0 {
        let a1 = if m.r32(a0 + 0x1d40) ^ 7 != 0 { 0x4f0 } else { 0x410 };
        let p = m.r32(a0 + 0x150);
        if m.call(0x001c_46c0, &[p, a1], &[f4(f32::from_bits(0x3e80_0000))]).0 != 0 {
            m.call(0x0019_cce8, &[s1], &[]);
        }
    }
    m.call(0x0019_d330, &[s1], &[]);
}

/// FUN_00138370(n, tbl, out, t, period): first entry (stride 8, {start, len}) with len != 0 and
/// wrapped (t - start) < len + 1e-4; writes the offset to *out.
pub fn ee_sub_138370<M: QMem + ?Sized>(m: &mut M, a0: i32, a1: u32, a2: u32, f12: f32, f13: f32) -> i32 {
    let f4c = f32::from_bits(0x38d1_b717);
    let mut v1: i32 = 0;
    let mut p = a1;
    loop {
        if !(v1 < a0) {
            return v1;
        }
        let f2 = m.rf(p + 4);
        if f2 == 0.0 {
            p = p.wrapping_add(8);
            v1 = v1.wrapping_add(1);
            continue;
        }
        let f0 = m.rf(p);
        let mut f1 = f12 - f0;
        if f1 < 0.0 {
            f1 = f1 + f13;
        }
        let f0 = f2 + f4c;
        if !(f1 < f0) {
            p = p.wrapping_add(8);
            v1 = v1.wrapping_add(1);
            continue;
        }
        if a2 != 0 {
            m.wf(a2, f1);
        }
        return v1;
    }
}

/// FUN_00228358(o, s): set state +0x13c (4 -> FUN_001a6ca8(o, 1); FUN_0020efc0(GS, 3); GS+0x3c84 = o),
/// +0x140 = DAT_002dded0.
pub fn ee_sub_228358<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if a1 == m.r32(a0 + 0x13c) {
        return;
    }
    if a1 == 4 {
        m.call(0x001a_6ca8, &[a0, 1], &[]);
        m.call(0x0020_efc0, &[GS, 3], &[]);
        m.w32(GS + 0x3c84, a0);
    }
    m.w32(a0 + 0x13c, a1);
    let f = m.rf(0x002d_ded0);
    m.wf(a0 + 0x140, f);
}

/// FUN_001b9b70(o, out): FUN_00143220(o, out); switch on o[0x13d0] (jump table 0x2a75f0).
pub fn ee_sub_1b9b70<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.call(0x0014_3220, &[a0, a1], &[]);
    let s = m.r32(a0 + 0x13d0);
    match s {
        0 | 1 | 9 | 10 => m.w32(a1, 1),
        11 => {
            m.w32(a1, 1);
            let v = m.rv(a0 + 0x1830);
            m.wv(a1 + 0x10, v);
            let v = m.rv(0x002a_2dd0);
            m.wv(a1 + 0x20, v);
        }
        _ => {}
    }
    m.w32(a1 + 4, 0x4348_0000);
}

/// FUN_0023d468(src double*, out): unpack an IEEE double into {class, sign, exp, mantissa(+0x10)}.
pub fn ee_sub_23d468<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let v = m.r64(a0);
    let mut mant = v & 0x000f_ffff_ffff_ffff;
    m.w32(a1 + 4, (v >> 63) as u32);
    let e = ((v >> 52) & 0x7ff) as u32;
    if e == 0 {
        m.w32(a1, 2);
        return;
    }
    if e != 0x7ff {
        mant = (mant << 8) | (0x8000u64 << 45);
        m.w32(a1 + 0x10, mant as u32);
        m.w32(a1 + 0x14, (mant >> 32) as u32);
        m.w32(a1 + 8, e.wrapping_sub(0x3ff));
        m.w32(a1, 3);
        return;
    }
    if mant == 0 {
        m.w32(a1, 4);
        return;
    }
    if mant & (0x8000u64 << 36) != 0 {
        m.w32(a1, 1);
    } else {
        m.w32(a1, 0);
    }
    m.w32(a1 + 0x10, mant as u32);
    m.w32(a1 + 0x14, (mant >> 32) as u32);
}

/// FUN_001d69f0(p0, p1, g, t): ballistic launch velocity from p0 to p1 in time t under gravity g.z along the
/// axis DAT_002a2dc0.
pub fn ee_sub_1d69f0<M: QMem + ?Sized>(m: &mut M, a0: V4, a1: V4, a2: V4, f12: f32) -> V4 {
    let f3 = a2[2];
    let mut f0 = f12 * 0.5;
    let d = [a1[0] - a0[0], a1[1] - a0[1], a1[2] - a0[2], a1[3] - a0[3]];
    let f1 = 1.0 / f12;
    f0 = f12 * f0;
    let mut f2 = d[2];
    f0 = f0 * f3;
    let ax = m.rv(0x002a_2dc0);
    let dot = (d[0] * ax[0] + d[1] * ax[1]) + 1.0 * (d[2] * ax[2]);
    f2 = f2 - f0;
    let h = [d[0] - ax[0] * dot, d[1] - ax[1] * dot, d[2] - ax[2] * dot, d[3] - ax[3] * dot];
    f2 = f2 / f12;
    [h[0] * f1 + ax[0] * f2, h[1] * f1 + ax[1] * f2, h[2] * f1 + ax[2] * f2, h[3] * f1 + ax[3] * f2]
}

/// FUN_00208948(o, x): remove every x from the list at +0x3ac (count +0x3a8) via FUN_002332b8 (memmove).
pub fn ee_sub_208948<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let mut s1: i32 = 0;
    let mut a2 = m.r32(a0 + 0x3a8) as i32;
    if a2 <= 0 {
        return;
    }
    let mut s3 = a0 + 0x3b0;
    let mut s0 = a0 + 0x3ac;
    loop {
        if m.r32(s0) == a1 {
            let n = ((a2.wrapping_sub(s1) as u32) << 2).wrapping_sub(4);
            m.call(0x0023_32b8, &[s0, s3, n], &[]);
            let v = m.r32(a0 + 0x3a8).wrapping_sub(1);
            m.w32(a0 + 0x3a8, v);
        }
        a2 = m.r32(a0 + 0x3a8) as i32;
        s1 = s1.wrapping_add(1);
        s3 = s3.wrapping_add(4);
        s0 = s0.wrapping_add(4);
        if !(s1 < a2) {
            return;
        }
    }
}

/// FUN_0022cad0(r, v): rotate v by Euler angles r.y / r.z via FUN_0011d608 (sincos); result quadword
/// {f2, f1, f0, sext(f0)}.
pub fn ee_sub_22cad0<M: QMem + ?Sized>(m: &mut M, a0: V4, a1: V4) -> V4 {
    let s = stack_push(m, 0x20);
    m.call(0x0011_d608, &[s, s + 4], &[f4(a0[1])]);
    m.call(0x0011_d608, &[s + 8, s + 0xc], &[f4(a0[2])]);
    let f1 = m.rf(s + 4);
    let f4_ = m.rf(s + 0xc);
    let f5 = a1[0];
    let f3 = m.rf(s + 8);
    let f0 = m.rf(s);
    let f6 = a1[1];
    let f7 = a1[2];
    stack_pop(m, 0x20);
    let mut f2 = f1 * f4_;
    let mut f1 = f1 * f3;
    let mut f0 = -f0;
    f2 = f2 * f5;
    let f4_ = f4_ * f6;
    f0 = f0 * f5;
    f1 = f1 * f5;
    let f3 = f3 * f6;
    f1 = f1 + f4_;
    f0 = f0 + f7;
    f2 = f2 - f3;
    let hi = ((f0.to_bits() as i32) >> 31) as u32;
    [f2, f1, f0, f32::from_bits(hi)]
}

/// FUN_001c7c30(o, flag): frame-guarded FUN_00258200 broadcast of {0x17, flag != 0}; DAT_003e1bf4 =
/// FUN_001c7d08().
pub fn ee_sub_1c7c30<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let sp = stack_push(m, 0x40);
    m.call(0x001a_e478, &[sp], &[]);
    let s2 = m.call(0x001c_7b18, &[a0], &[]).0;
    let v1 = (a1 != 0) as u32;
    if s2 == 0 {
        m.call(0x001a_e4a0, &[sp, 2], &[]);
        stack_pop(m, 0x40);
        return;
    }
    let s1 = sp + 0x10;
    m.w32(sp + 0x14, v1);
    m.w32(sp + 0x10, 0x17);
    m.call(0x0012_91a8, &[sp + 0x20, s1], &[]);
    m.call(0x0025_8200, &[sp + 0x30, s2, sp + 0x20, 0x003d_4a60], &[]);
    m.call(0x0012_9ec8, &[sp + 0x30], &[]);
    m.call(0x0012_9ec8, &[sp + 0x20], &[]);
    m.call(0x0012_9ec8, &[s1], &[]);
    let v = m.call(0x001c_7d08, &[], &[]).0;
    m.w32(0x003e_1bf4, v);
    m.call(0x001a_e4a0, &[sp, 2], &[]);
    stack_pop(m, 0x40);
}

/// FUN_001c5d18(o).
pub fn ee_sub_1c5d18<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    if m.r32(a0 + 0x1c8) == 0 {
        return;
    }
    let f1 = m.rf(a0 + 0x1cc);
    let f0 = m.rf(0x002d_ded0);
    if f1 < f0 {
        vslot(m, a0, 75, &[1], &[]);
        return;
    }
    let sp = stack_push(m, 0x40);
    let s3 = m.call(0x001c_4bd8, &[GS, 0x10, sp], &[]).0 as i32;
    let mut s1: i32 = 0;
    let mut s0 = sp;
    loop {
        if !(s1 < s3) {
            break;
        }
        if s1 > 0 && m.call(0x0013_2b10, &[a0, 0x5b], &[]).0 == 0 {
            break;
        }
        let a1 = m.r32(s0);
        let v = m.call(0x001c_5b20, &[a0, a1], &[]).0;
        s0 = s0.wrapping_add(4);
        if v != 0 {
            break;
        }
        s1 = s1.wrapping_add(1);
    }
    stack_pop(m, 0x40);
}

/// FUN_001f77c0(o): safe-iterate the DL at +0x4130 (iterator frame linked at DAT_002dd2fc); drop entries
/// whose +0x4c + 0.25 <= now (FUN_0011a2e8 unlink, FUN_0011cd98 free to the pool +0x4118).
pub fn ee_sub_1f77c0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let sp = stack_push(m, 0x10);
    let dl = a0.wrapping_add(0x4130);
    m.w32(sp + 4, dl);
    m.w32(sp, dl);
    let a1 = m.r32(0x002d_d2fc);
    let off = m.r32(dl + 8);
    m.w32(0x002d_d2fc, sp);
    m.w32(sp + 0xc, a1);
    let first = m.r32(a0 + 0x4130);
    m.w32(sp + 8, off);
    let mut s0 = first;
    m.w32(sp + 4, first.wrapping_add(off));
    let f20 = f32::from_bits(0x3e80_0000);
    while s0 != 0 {
        let f0 = m.rf(s0 + 0x4c) + f20;
        let f1 = m.rf(0x002d_ded0);
        if !(f1 < f0) {
            m.call(0x0011_a2e8, &[dl, s0], &[]);
            m.call(0x0011_cd98, &[a0.wrapping_add(0x4118), s0], &[]);
        }
        let v0 = m.r32(sp + 4);
        let o = m.r32(sp + 8);
        let n = m.r32(v0);
        s0 = n;
        m.w32(sp + 4, n.wrapping_add(o));
    }
    let v = m.r32(sp + 0xc);
    m.w32(0x002d_d2fc, v);
    stack_pop(m, 0x10);
}

/// FUN_001d2150(o).
pub fn ee_sub_1d2150<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let v1 = m.r32(a0 + 0x150);
    let f0 = m.r32(v1 + 0x18);
    m.w32(a0 + 0x544, f0);
    if m.r32(v1 + 0x30) & 0x40 == 0 {
        m.w32(a0 + 0x54c, f0);
        m.w32(a0 + 0x550, 0);
    } else {
        let a = m.r32(a0 + 0x51c);
        let b = m.r32(a0 + 0x540);
        m.w32(a0 + 0x550, a);
        m.w32(a0 + 0x54c, b);
    }
    let mut n = m.r32(a0 + 0x4f8) as i32;
    if n <= 0 {
        return;
    }
    let mut s3: i32 = 0;
    let mut s2 = a0 + 0x4fc;
    loop {
        let s0 = m.r32(s2);
        if m.call(0x001d_4e08, &[s0], &[]).0 != 0 {
            let p = m.r32(a0 + 0x150);
            let v1 = m.r32(s0 + 0x24);
            if m.r32(p + 0x30) & v1 != 0 {
                let x = m.r32(s0 + 0x20);
                m.call(0x001f_0660, &[x, 0x67], &[]);
            } else {
                let a1 = m.r32(s0 + 0x28);
                if m.call(0x001c_46c0, &[p, a1], &[f4(0.0)]).0 != 0 {
                    let x = m.r32(s0 + 0x20);
                    m.call(0x001f_0660, &[x, 0x68], &[]);
                }
            }
        }
        n = m.r32(a0 + 0x4f8) as i32;
        s3 = s3.wrapping_add(1);
        s2 = s2.wrapping_add(4);
        if !(s3 < n) {
            return;
        }
    }
}

/// FUN_00221178(o, s): set state +0x134 (0 / 1 start sounds + vslot 2 messages), +0x138 = now.
pub fn ee_sub_221178<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if a1 == m.r32(a0 + 0x134) {
        return;
    }
    if a1 == 0 {
        m.call(0x0014_e4c8, &[0x002d_df40, 8, 0xffff_ffff, 0, 0, a0], &[]);
        vslot(m, a0, 2, &[6, 1, 0, 0, 0], &[]);
    } else if a1 == 1 {
        let x = m.r32(a0 + 0x128).wrapping_add(2);
        let y = m.r32(a0 + 0xe0);
        m.call(0x0014_e328, &[0x002d_df40, x, 0x002d_e2f0, y, a0], &[]);
        vslot(m, a0, 2, &[5, 1, 0, 0, 0], &[]);
    }
    m.w32(a0 + 0x134, a1);
    let f = m.r32(0x002d_ded0);
    m.w32(a0 + 0x138, f);
}

/// FUN_0015ad48(o, pos, vel): transform pos / vel into the frame of **(o+8) (vslot 31 matrix,
/// FUN_0011c168 compose, FUN_001a72d0 / FUN_001a7348 apply), then add FUN_0022d348's velocity by mode o[0x18].
pub fn ee_sub_15ad48<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    let s4 = a0.wrapping_add(0x10);
    let v = m.r32(a0 + 8);
    let obj = m.r32(v);
    if obj == 0 {
        return;
    }
    let sp = stack_push(m, 0x80);
    let s1 = vslot(m, obj, 31, &[sp], &[]).0;
    let s0 = sp + 0x40;
    m.call(0x0011_c168, &[s0, s1.wrapping_add(0x40), sp], &[]);
    let q = m.rv(a1);
    let r = m.call(0x001a_72d0, &[s0, 0], &[q]).1;
    m.wv(a1, r);
    if m.r32(s4 + 8) != 3 {
        let q = m.rv(a2);
        let r = m.call(0x001a_7348, &[s0, 0], &[q]).1;
        m.wv(a2, r);
        let k = m.r32(s4 + 8);
        if k != 2 {
            let q = if k != 1 { m.rv(s1.wrapping_add(0x70)) } else { m.rv(a1) };
            let r = m.call(0x0022_d348, &[s1, 0], &[q]).1;
            let p = m.rv(a2);
            m.wv(a2, [p[0] + r[0], p[1] + r[1], p[2] + r[2], p[3] + r[3]]);
        }
    }
    stack_pop(m, 0x80);
}

/// FUN_001fe738(o): flush pending +0x200 events, stop the sound, release the 6 handles at +0x22c.
pub fn ee_sub_1fe738<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let v = m.r32(a0 + 0x200);
    if v == 0 {
        return;
    }
    let s2 = a0.wrapping_add(0x200);
    let s1 = m.r32(v + 0x60);
    let s3 = a0.wrapping_add(0x22c);
    let mut v1 = m.r32(a0 + 0x244);
    loop {
        let n = m.r32(s1 + 0x54);
        if !((v1 as i32) < (n as i32)) {
            break;
        }
        let b = m.r32(s1 + 0x58);
        m.call(0x001f_e0e8, &[a0, b.wrapping_add(v1 << 3)], &[]);
        v1 = m.r32(a0 + 0x244).wrapping_add(1);
        m.w32(a0 + 0x244, v1);
    }
    let h = m.r32(a0 + 0x240);
    if h != 0 {
        m.call(0x001f_0660, &[h, 0x611], &[]);
    }
    if s2 != 0 {
        let p = m.r32(a0 + 0x200);
        if p != 0 {
            m.call(0x001f_b098, &[p], &[f4(0.0)]);
            m.call(0x0025_9b48, &[s2, 0], &[]);
        }
    }
    m.w32(a0 + 0x24c, 0);
    m.w32(a0 + 0x244, 0);
    m.w32(a0 + 0x248, 0);
    let mut k: i32 = 5;
    let mut p = s3;
    loop {
        let o = m.r32(p);
        if o != 0 {
            vslot(m, o, 1, &[], &[]);
        }
        k -= 1;
        p = p.wrapping_add(4);
        if k < 0 {
            break;
        }
    }
    m.call(0x0023_33b4, &[s3, 0, 0x18], &[]);
}

/// FUN_0021bc08(o): fill the whole history ring (+0x15c, n = +0x158, stride 0x50, cursor +0x160) with the
/// current vslot-30 matrix, timestamps stepping 1/60 up to now; +0x164 = 0.
pub fn ee_sub_21bc08<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let mut f20 = m.rf(0x002d_ded0);
    let p = vslot(m, a0, 30, &[], &[]).0;
    let q0 = m.rv(p);
    let n = m.r32(a0 + 0x158) as i32;
    let f1 = f32::from_bits(0x3c88_8889);
    let q1 = m.rv(p + 0x10);
    let mut t0 = n.wrapping_sub(1);
    let f0 = (n as f32) * f1;
    let q2 = m.rv(p + 0x20);
    f20 = f20 - f0;
    let q3 = m.rv(p + 0x30);
    if n == 0 {
        panic!("FUN_0021bc08: break 7 (division by zero)");
    }
    if t0 < 0 {
        m.w32(a0 + 0x164, 0);
        return;
    }
    loop {
        let a1 = m.r32(a0 + 0x160) as i32;
        t0 = t0.wrapping_sub(1);
        let v1 = m.r32(a0 + 0x158) as i32;
        let v0 = a1.wrapping_add(1);
        let base = m.r32(a0 + 0x15c);
        let off = a1.wrapping_mul(0x50) as u32;
        if v1 == 0 {
            panic!("FUN_0021bc08: break 7 (division by zero)");
        }
        let e = base.wrapping_add(off);
        m.w32(a0 + 0x160, v0.wrapping_rem(v1) as u32);
        m.wf(e, f20);
        m.w32(e + 4, 0);
        f20 = f20 + f1;
        m.wv(e + 0x10, q0);
        m.wv(e + 0x20, q1);
        m.wv(e + 0x30, q2);
        m.wv(e + 0x40, q3);
        if t0 < 0 {
            break;
        }
    }
    m.w32(a0 + 0x164, 0);
}

/// FUN_0018f560(o, out, x): FUN_0018f2b0 probe; out[3] = clamp((h + 50) / 110), out[0] = 1 - out[3];
/// FUN_001384c0(x); o[0x18c0] = hit; on hit and !o[0x18c8]: FUN_002046b0(o, pos), FUN_00204ca8(o).
pub fn ee_sub_18f560<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    let f14 = if m.r32(a0 + 0x18c8) != 0 { 0x3fcc_cccd } else { 0x4049_999a };
    let sp = stack_push(m, 0x20);
    let f12 = m.rf(a0 + 0x1a8);
    m.call(
        0x0018_f2b0,
        &[a0, 2, 0x002b_f4c0, sp + 0x10, sp],
        &[f4(f12), f4(f32::from_bits(0x3e19_999a)), f4(f32::from_bits(f14))],
    );
    let hit = m.r32(sp + 0x10);
    let mut f1 = 1.0f32;
    if hit != 0 {
        let mut f0 = m.rf(sp + 8);
        f0 = f0 - f32::from_bits(0xc248_0000);
        f0 = f0 * f32::from_bits(0x3c14_f209);
        if f0 < 0.0 {
            f1 = 0.0;
        } else if !(1.0 < f0) {
            f1 = f0;
        }
    }
    m.wf(a1 + 0xc, f1);
    m.wf(a1, 1.0 - f1);
    m.call(0x0013_84c0, &[a2], &[]);
    let hit = m.r32(sp + 0x10);
    m.w32(a0 + 0x18c0, hit);
    if hit != 0 && m.r32(a0 + 0x18c8) == 0 {
        let q = m.rv(sp);
        m.call(0x0020_46b0, &[a0], &[q]);
        m.call(0x0020_4ca8, &[a0], &[]);
    }
    stack_pop(m, 0x20);
}

/// shared body of FUN_001957f8 / FUN_001a3108 / FUN_001cdb78 / FUN_00223e18 (identical code): when
/// (a0 != 0 && a1 == 0xffff), refcount the six static pools at 0x2dd2e0.. and FUN_0011cca8-init each on 0->1.
fn pools_addref<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if a1 != 0xffff || a0 == 0 {
        return;
    }
    const P: [(u32, u32, u32, u32); 6] = [
        (0x002d_d2e0, 0x0045_20a8, 0x14, 0x2000),
        (0x002d_d2e4, 0x0045_20d0, 0x18, 0x800),
        (0x002d_d2e8, 0x0045_20f8, 0x28, 0x200),
        (0x002d_d2ec, 0x0045_2120, 0x1c, 0x800),
        (0x002d_d2f0, 0x0045_2148, 0xc, 0xc8),
        (0x002d_d2f4, 0x0045_2170, 0x14, 0x100),
    ];
    for (c, pool, sz, n) in P {
        let v = m.r32(c).wrapping_add(1);
        m.w32(c, v);
        if v == 1 {
            m.call(0x0011_cca8, &[pool, sz, n, 0x10], &[]);
        }
    }
}

/// FUN_001957f8(a, b): see `pools_addref`.
pub fn ee_sub_1957f8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    pools_addref(m, a0, a1)
}

/// FUN_001a3108(a, b): see `pools_addref`.
pub fn ee_sub_1a3108<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    pools_addref(m, a0, a1)
}

/// FUN_001cdb78(a, b): see `pools_addref`.
pub fn ee_sub_1cdb78<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    pools_addref(m, a0, a1)
}

/// FUN_00223e18(a, b): see `pools_addref`.
pub fn ee_sub_223e18<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    pools_addref(m, a0, a1)
}

/// FUN_001b9a08(o, s): DAT_002deb3c = s; FUN_001619a0(0x2deb30, 0x3c, o); switch s (jump table 0x2a75c0).
pub fn ee_sub_1b9a08<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.w32(0x002d_eb3c, a1);
    m.call(0x0016_19a0, &[0x002d_eb30, 0x3c, a0], &[]);
    match a1 {
        0xf => {
            let v = m.r32(a0 + 0x1ac4);
            m.w32(a0 + 0x9bc, a1);
            m.w32(a0 + 0x130c, v);
            m.call(0x0019_28f8, &[a0.wrapping_add(0x4c0), 0xb, 0, 0], &[]);
        }
        0x10 => {
            m.call(0x001b_6078, &[a0, 0x15], &[]);
        }
        0x13 => {
            m.call(0x0014_1cd0, &[a0, a1], &[]);
            m.w32(a0 + 0x143c, 1);
            m.call(0x001b_6078, &[a0, 8], &[]);
        }
        0x14 => {
            m.call(0x0014_1cd0, &[a0, a1], &[]);
            m.call(0x001b_6078, &[a0, 0xc], &[]);
            m.call(0x001b_8d30, &[a0, 0], &[]);
            m.call(0x001b_8d30, &[a0, 1], &[]);
        }
        0x15 => {
            let a2 = m.r32(a0 + 0x10dc);
            m.w32(a0 + 0x9bc, 0x15);
            m.call(0x0019_28f8, &[a0.wrapping_add(0x4c0), 0xb, a2, 0], &[]);
        }
        0x16 => {
            m.call(0x0014_1cd0, &[a0, a1], &[]);
            m.w32(a0 + 0x13e8, 1);
            m.call(0x001b_6078, &[a0, 3], &[]);
        }
        _ => {
            m.call(0x0014_1d38, &[a0, a1], &[]);
        }
    }
}

/// FUN_001b5d30(o, s): set state +0x17f4 (+0x17f8 = now) and restart the +0x1030 / +0x1034 sound pair.
pub fn ee_sub_1b5d30<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: i32) {
    let v = m.r32(a0 + 0x17f4);
    if a1 as u32 == v {
        return;
    }
    if v == 0 {
        let x = m.r32(a0 + 0x58);
        m.call(0x0023_03a0, &[a0, x, 2], &[]);
        let x = m.r32(a0 + 0x5c);
        m.call(0x0023_03a0, &[a0, x, 2], &[]);
    }
    m.w32(a0 + 0x17f4, a1 as u32);
    let mut s4 = 0u32;
    let mut s5 = 0u32;
    let f = m.r32(0x002d_ded0);
    let mut s3 = 1u32;
    m.w32(a0 + 0x17f8, f);
    if a1 == 1 {
        s4 = m.r32(a0 + 0x1634);
        m.call(0x0020_bd68, &[a0.wrapping_add(0x1630)], &[]);
        s3 = 9;
    } else if a1 < 2 {
        if a1 == 0 {
            let x = m.r32(a0 + 0x58);
            m.call(0x0023_03a0, &[a0, x, 1], &[]);
            let x = m.r32(a0 + 0x5c);
            m.call(0x0023_03a0, &[a0, x, 1], &[]);
            s5 = m.r32(a0 + 0x16e0);
            let q = m.rv(0x002a_2dd0);
            m.wv(a0 + 0x190, q);
        }
    } else if a1 == 2 {
        s4 = m.r32(a0 + 0x17f0);
        s3 = 9;
    }
    let s0 = a0.wrapping_add(0x1030);
    let s1 = a0.wrapping_add(0x1034);
    m.call(0x0013_0658, &[s0], &[]);
    m.call(0x001f_15b8, &[s1], &[]);
    m.call(0x0013_05b8, &[s4, a0, s3, s0], &[f4(0.0), f4(1.0)]);
    m.call(0x001f_1448, &[s5, a0, 0xffff_ffff, s3, s1], &[]);
}

/// FUN_001a3d00(w, o): allocate o a slot id (+0x118, lowest free bit of the 8-word mask +0x4590, table
/// +0x4154) if it has none, then mark it dirty (+0x4570 bitset) or forward to FUN_001a3b80.
pub fn ee_sub_1a3d00<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let (s1, s0) = (a0, a1);
    if (m.r32(s0 + 0x118) as i32) < 0 {
        let mut a3: u32 = 1;
        let mut t2 = s1.wrapping_add(0x4590);
        let mut i: u32 = 0;
        loop {
            if !(i < 8) {
                return;
            }
            let t1 = m.r32(t2);
            if t1 != 0 {
                let t1 = t1.wrapping_neg() & t1;
                let mut v0 = i << 5;
                let a5 = m.r32(t2);
                if t1 & 0xaaaa_aaaa != 0 {
                    v0 = a3;
                }
                if t1 & 0xcccc_cccc != 0 {
                    v0 = v0.wrapping_add(2);
                }
                if t1 & 0xf0f0_f0f0 != 0 {
                    v0 = v0.wrapping_add(4);
                }
                m.w32(t2, a5 & !t1);
                if t1 & 0xff00_ff00 != 0 {
                    v0 = v0.wrapping_add(8);
                }
                let v1 = if t1 & 0xffff_0000 != 0 { v0.wrapping_add(0x10) } else { v0 };
                m.w32(s1.wrapping_add(v1 << 2) + 0x4154, s0);
                m.w32(s0 + 0x118, v1);
                break;
            }
            a3 = a3.wrapping_add(0x20);
            t2 = t2.wrapping_add(4);
            i += 1;
        }
    }
    if m.r32(s1 + 0x4554) != 0 || m.r32(s0 + 0x15c) != 0 {
        return;
    }
    if m.r32(s0 + 0xe0) != 0 && m.r32(s0 + 0x160) == 0 && m.r32(s1 + 0x455c) != 0 {
        let q = m.call(0x001a_62e8, &[s0], &[]).1;
        let a3 = m.r32(s0 + 0x118);
        let a1 = m.r32(s1 + 0x455c);
        let f12 = m.rf(s0 + 0x168);
        m.call(0x001a_3b80, &[s1, a1, a3], &[q, f4(f12)]);
        return;
    }
    let a0v = m.r32(s0 + 0x118) as i32;
    let v0 = if a0v > -1 { a0v } else { a0v.wrapping_add(0x1f) };
    let bit = 1u32 << ((a0v as u32) & 0x1f);
    let p = s1.wrapping_add(0x4570).wrapping_add(((v0 >> 5) << 2) as u32);
    let v = m.r32(p) | bit;
    m.w32(p, v);
}

/// FUN_0019c688(msg): dispatch on msg[0x1c] (jump table 0x2a65c0).
pub fn ee_sub_19c688<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let k = m.r32(a0 + 0x1c);
    let (a1, a3, base): (u32, u32, u32) = match k {
        6 => {
            let a3 = m.r32(a0 + 0xc);
            (1, a3, a3.wrapping_add(0x150))
        }
        9 => {
            let a3 = m.r32(a0 + 0xc);
            (4, a3, a3.wrapping_add(0xe0))
        }
        10 => {
            let a3 = m.r32(a0 + 0xc);
            (3, a3, a3.wrapping_add(0x190))
        }
        11 => {
            let a3 = m.r32(a0 + 0xc);
            (0, a3, a3.wrapping_add(0x190))
        }
        12 => {
            let a3 = m.r32(a0 + 0xc);
            (0, a3, a3.wrapping_add(0xe0))
        }
        13 => {
            let v = m.r32(a0 + 0xc);
            let a1 = m.r32(v + 0x10c);
            (a1, v, v.wrapping_add(0xe0))
        }
        14 => {
            let v = m.r32(a0 + 0xc);
            let o = m.r32(a0);
            m.w32(o + 0x18c4, v);
            if m.r32(v + 0x124) == 3 {
                m.call(0x0018_c250, &[o, 0x16, 0xffff_ffff], &[]);
            } else {
                m.call(0x0018_c250, &[o, 3, 1], &[]);
            }
            return;
        }
        15 => {
            let o = m.r32(a0);
            let v = m.r32(a0 + 0xc);
            m.w32(o + 0xf5c, v);
            let o = m.r32(a0);
            let f = m.r32(a0 + 0x14);
            m.w32(o + 0xf60, f);
            let o = m.r32(a0);
            vslot(m, o, 87, &[8], &[]);
            return;
        }
        16 => {
            let v = m.r32(a0 + 0xc);
            let o = m.r32(a0);
            m.w32(o + 0x1f18, v);
            m.call(0x0018_c250, &[o, 0x15, 0xa], &[]);
            return;
        }
        18 => {
            let p = m.r32(0x002d_d5bc);
            m.call(0x0013_5248, &[p, 8], &[]);
            return;
        }
        _ => {
            m.call(0x0019_5fb8, &[a0, 4, 0], &[]);
            return;
        }
    };
    let o = m.r32(a0);
    let a3x = if a3 == 0 { 0 } else { base };
    m.call(0x0019_28f8, &[o.wrapping_add(0x4c0), a1, a3, a3x], &[]);
}

/// FUN_001b5658(o, s): set state +0x1458 (+0x145c = now; saves the playback fraction to +0x1a0 when leaving
/// states 0..2) and restart the +0x1030 / +0x1034 sound pair at the scaled offset.
pub fn ee_sub_1b5658<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: i32) {
    let v = m.r32(a0 + 0x1458) as i32;
    if a1 == v {
        return;
    }
    if v == 0 {
        let x = m.r32(a0 + 0x58);
        m.call(0x0023_03a0, &[a0, x, 2], &[]);
        let x = m.r32(a0 + 0x5c);
        m.call(0x0023_03a0, &[a0, x, 2], &[]);
    } else if v >= 0 && v < 3 {
        let p = m.r32(a0 + 0x1030);
        let q = m.r32(p + 0x14);
        let f0 = m.rf(p + 0x1c);
        let f1 = m.rf(q + 0x74);
        m.wf(a0 + 0x1a0, f0 / f1);
    }
    m.w32(a0 + 0x1458, a1 as u32);
    let mut s3 = 0u32;
    let mut s5 = 0u32;
    let f = m.r32(0x002d_ded0);
    let mut s4 = 1u32;
    let mut f20 = 0.0f32;
    m.w32(a0 + 0x145c, f);
    if a1 == 1 {
        s3 = m.r32(a0 + 0x1464);
        let f1 = m.rf(a0 + 0x1a0);
        s4 = 9;
        let f0 = m.rf(s3 + 0x74);
        f20 = f1 * f0;
        m.call(0x0020_bd68, &[a0.wrapping_add(0x1320)], &[]);
    } else if a1 < 2 {
        if a1 == 0 {
            let x = m.r32(a0 + 0x58);
            m.call(0x0023_03a0, &[a0, x, 1], &[]);
            let x = m.r32(a0 + 0x5c);
            m.call(0x0023_03a0, &[a0, x, 1], &[]);
            s5 = m.r32(a0 + 0x1510);
            let q = m.rv(0x002a_2dd0);
            m.wv(a0 + 0x190, q);
        }
    } else if a1 == 2 {
        s3 = m.r32(a0 + 0x151c);
        s4 = 9;
        let f1 = m.rf(a0 + 0x1a0);
        let f0 = m.rf(s3 + 0x74);
        f20 = f1 * f0;
    }
    let s0 = a0.wrapping_add(0x1030);
    let s1 = a0.wrapping_add(0x1034);
    m.call(0x0013_0658, &[s0], &[]);
    m.call(0x001f_15b8, &[s1], &[]);
    m.call(0x0013_05b8, &[s3, a0, s4, s0], &[f4(f20), f4(1.0)]);
    m.call(0x001f_1448, &[s5, a0, 0xffff_ffff, s4, s1], &[]);
}

/// FUN_001d4940(o, s): state change +0x180 (+0x184 = now) with the enter / leave effects of states 0 / 1 / 3
/// and the +0x570 anim cue FUN_001f0730(+0x570, DAT_002c2208[s]).
pub fn ee_sub_1d4940<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: i32) {
    let (s1, s2) = (a0, a1);
    let v1 = m.r32(s1 + 0x180);
    if s2 as u32 == v1 {
        return;
    }
    if v1 == 0 {
        let a2 = m.r32(s1 + 0x568);
        if a2 != 0 {
            vslot(m, a2, 48, &[0xffff_ffff], &[]);
            let x = m.r32(s1 + 0x568);
            m.call(0x0021_3b00, &[x, 0x34f], &[]);
            m.call(0x001d_4c88, &[s1], &[]);
        }
        if m.r32(s1 + 0x59c) != 0xffff_ffff {
            let s0 = m.call(0x001f_bb90, &[], &[]).0;
            m.call(0x001f_b2f8, &[s0, s1], &[]);
            m.call(0x0025_8040, &[s1.wrapping_add(0x598), s0], &[]);
            let x = m.r32(s1 + 0x598);
            m.call(0x001f_b4f0, &[x], &[f4(f32::from_bits(0x43fa_0000)), f4(f32::from_bits(0x44bb_8000))]);
            let x = m.r32(s1 + 0x598);
            m.call(0x001f_b018, &[x], &[]);
        }
    } else if v1 == 1 {
        m.call(0x001d_4b28, &[s1, 0xffff_ffff], &[]);
    }
    m.w32(s1 + 0x180, s2 as u32);
    let f = m.r32(0x002d_ded0);
    m.w32(s1 + 0x184, f);
    if s2 == 1 {
        m.call(0x001d_4b28, &[s1, 0], &[]);
    } else if s2 < 2 {
        if s2 == 0 {
            let a1v = m.r32(s1 + 0x568);
            if a1v != 0 {
                vslot(m, a1v, 49, &[], &[]);
            }
            let p = m.r32(s1 + 0x598);
            let s0 = s1.wrapping_add(0x598);
            if p != 0 {
                m.call(0x001f_b098, &[p], &[f4(0.0)]);
                m.call(0x0025_8040, &[s0, 0], &[]);
            }
        }
    } else if s2 == 3 {
        vslot(m, s1, 2, &[5, 0, 0, 0, 0], &[]);
    }
    let a0v = m.r32(s1 + 0x570);
    if a0v != 0 {
        let v = m.r32(s1 + 0x180);
        if v != 0xffff_ffff {
            let a1v = m.r32((v << 2).wrapping_add(0x002c_2208));
            m.call(0x001f_0730, &[a0v, a1v], &[]);
        }
    }
}

/// FUN_001533b0(o, who): handle a pickup / checkpoint touch by `who` against the +0x1c4 entries (stride 0x24,
/// count +0x1c0 cached at entry).
pub fn ee_sub_1533b0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let (s2, s0) = (a0, a1);
    let s3 = m.r32(s0 + 0x150);
    let v = m.call(0x001c_4660, &[s3, 0xf0], &[f4(0.0)]).0;
    m.w32(s2 + 0x230, v);
    if v == 0 {
        return;
    }
    let n = m.r32(s2 + 0x1c0) as i32;
    if n <= 0 {
        return;
    }
    let mut v1: i32 = 0;
    let mut s1 = s2.wrapping_add(0x1c4);
    loop {
        let w = m.r32(s1);
        v1 = v1.wrapping_add(1);
        if w == s0 && m.r32(s1 + 0x20) == 0 {
            break;
        }
        if !(v1 < n) {
            return;
        }
        s1 = s1.wrapping_add(0x24);
    }
    m.call(0x0016_19a0, &[0x002d_eb30, 0x35, s2], &[]);
    let f0 = m.call(0x001f_e3f8, &[0x003d_d040], &[]).1[0];
    let f1 = m.rf(s1 + 0xc) - m.rf(s2 + 0x58);
    let fail = |m: &mut M, off: u32| {
        m.call(0x0015_2390, &[s0, 0xffff_ffff, 0xffff_ffff, 1], &[]);
        let f = m.rf(s2 + off);
        m.call(0x0015_3640, &[s2], &[f4(f)]);
    };
    if f0 < f1 {
        fail(m, 0x298);
        return;
    }
    if m.r32(s1 + 0x1c) != 0 {
        fail(m, 0x2a0);
        return;
    }
    m.w32(s1 + 0x1c, 1);
    let a1v = m.r32(s1 + 4);
    if m.call(0x001c_4700, &[s3, a1v], &[]).0 == 0 {
        fail(m, 0x29c);
        return;
    }
    let a2v = m.r32(s1 + 8);
    let a1v = m.r32(s1 + 4);
    m.call(0x0015_2390, &[s0, a1v, a2v, 0], &[]);
    let mut p = s2.wrapping_add(0x70);
    let f = m.call(0x0015_35a0, &[s2, s1], &[]).1[0];
    let f = m.call(0x0015_3638, &[s2], &[f4(f)]).1[0];
    m.call(0x0015_3640, &[s2], &[f4(f)]);
    let mut j: i32 = 0;
    loop {
        let n = m.r32(s2 + 0x60) as i32;
        if !(j < n) {
            return;
        }
        let f12 = m.rf(p);
        let f13 = m.rf(s1 + 0xc);
        let r = m.call(0x0011_d4d0, &[], &[f4(f12), f4(f13), f4(f32::from_bits(0x3dcc_cccd))]).0;
        if r != 0 {
            m.w32(p + 0x20, 1);
            return;
        }
        p = p.wrapping_add(0x30);
        j += 1;
    }
}

/// FUN_0022b220(o, desc, a, b): (re)start: flag desc bit 1 if it matches the current one at 0x3e10cc, spawn
/// via FUN_0015e938(GS, 0x405, a, 0) into o[0x54]; copy desc (0x4c bytes) to o+8, o[0x58] = a,
/// FUN_0022b090(o, b).
pub fn ee_sub_22b220<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32) {
    let (s1, s0, s3, s4) = (a0, a1, a2, a3);
    let v1 = m.r32(s1) as i32;
    if v1 == 1 {
        return;
    }
    if !(v1 < 2) {
        if v1 == 2 {
            m.call(0x0022_b090, &[s1, 0], &[]);
        } else if v1 == 3 {
            return;
        }
    }
    let a2p = s0.wrapping_add(4);
    let v0 = 0x003e_10d0u32;
    let eq = if a2p == v0 {
        true
    } else if a2p == 0 {
        false
    } else {
        let x = m.r32(v0 + 8) ^ m.r32(v0 + 0x38);
        let y = m.r32(a2p + 8) ^ m.r32(a2p + 0x38);
        x ^ y == 0
    };
    let w = m.r32(s0);
    m.w32(s0, if eq { w | 2 } else { w & 0xffff_fffd });
    let s2 = GS;
    let v = m.call(0x0015_e938, &[s2, 0x405, s3, 0], &[]).0;
    m.w32(s1 + 0x54, v);
    if v == 0 {
        m.call(0x0020_df80, &[s2, s0], &[]);
        return;
    }
    m.w32(s1 + 4, 1);
    let d = s1.wrapping_add(8);
    for i in 0..0x4c {
        let b = m.r8(s0.wrapping_add(i));
        m.w8(d.wrapping_add(i), b);
    }
    m.w32(s1 + 0x58, s3);
    m.call(0x0022_b090, &[s1, s4], &[]);
}

/// FUN_00247de8(x, y): fmodf (newlib e_fmodf, integer long-division); Zero[] table at 0x2abc08.
pub fn ee_sub_247de8<M: QMem + ?Sized>(m: &mut M, f12: f32, f13: f32) -> f32 {
    let x = f12.to_bits();
    let y = f13.to_bits();
    let sx = x & 0x8000_0000;
    let mut hx = (x ^ sx) as i32;
    let mut hy = (y & 0x7fff_ffff) as i32;
    if hy == 0 || 0x7f7f_ffff < hx || 0x7f80_0000 < hy {
        let f0 = f12 * f13;
        return f0 / f0;
    }
    if hx < hy {
        return f12;
    }
    let zero = |m: &mut M| m.rf(((sx >> 31) << 2).wrapping_add(0x002a_bc08));
    if hx == hy {
        return zero(m);
    }
    let mut ix: i32;
    if 0x007f_ffff < hx {
        ix = (hx >> 23) - 0x7f;
    } else {
        ix = -0x7e;
        let mut i = hx << 8;
        while i > 0 {
            ix -= 1;
            i <<= 1;
        }
    }
    let mut iy: i32;
    if 0x007f_ffff < hy {
        iy = (hy >> 23) - 0x7f;
    } else {
        iy = -0x7e;
        let mut i = hy << 8;
        if i >= 0 {
            loop {
                iy -= 1;
                i <<= 1;
                if i < 0 {
                    break;
                }
            }
        }
    }
    if ix < -0x7e {
        hx = ((hx as u32) << ((((-0x7e) - ix) as u32) & 0x1f)) as i32;
    } else {
        hx = (hx & 0x007f_ffff) | 0x0080_0000;
    }
    let mut t2 = iy < -0x7e;
    if t2 {
        hy = ((hy as u32) << ((((-0x7e) - iy) as u32) & 0x1f)) as i32;
    } else {
        hy = (hy & 0x007f_ffff) | 0x0080_0000;
    }
    let mut n = ix.wrapping_sub(iy);
    loop {
        let v0 = n;
        n = n.wrapping_sub(1);
        if v0 == 0 {
            break;
        }
        let hz = hx.wrapping_sub(hy);
        if hz < 0 {
            hx = hx.wrapping_shl(1);
        } else {
            if hz == 0 {
                return zero(m);
            }
            hx = hz.wrapping_shl(1);
        }
    }
    let hz = hx.wrapping_sub(hy);
    if !(hz < 0) {
        hx = hz;
    }
    if hx == 0 {
        return zero(m);
    }
    if !(0x007f_ffff < hx) {
        loop {
            hx = hx.wrapping_shl(1);
            iy -= 1;
            if 0x007f_ffff < hx {
                break;
            }
        }
        t2 = iy < -0x7e;
    }
    if !t2 {
        hx = (hx.wrapping_add(0xff80_0000u32 as i32)) | (iy.wrapping_add(0x7f) << 23);
    } else {
        hx = hx >> ((((-0x7e) - iy) as u32) & 0x1f);
    }
    f32::from_bits((hx as u32) | sx)
}

/// FUN_00118a88(node, limit, a3 = &budget, t0 = &out, a2 = point, f12 = eps): BSP plane classify / split.
/// Nodes below `limit` are interior; on-plane nodes recurse into both children and emit a 0xc-byte record.
pub fn ee_sub_118a88<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a3: u32, t0: u32, a2: V4, f12: f32) -> u32 {
    let mut s1 = a0;
    let mut f1: f32 = 0.0;
    let sp = stack_push(m, 0x30);
    while s1 != 0 {
        let pp = m.r32(s1 + 8);
        let pl = m.rv(pp);
        m.call(0x0025_64b8, &[sp], &[]);
        m.call(0x0025_64b8, &[sp + 0x10], &[]);
        let acc = pl[0] * a2[0] + pl[1] * a2[1] + 1.0 * (pl[2] * a2[2]);
        f1 = acc - 1.0 * pl[3];
        m.wv(sp + 0x10, [1.0, 0.0, 0.0, 1.0]);
        m.wv(sp, [f1, pl[1] * a2[1], pl[2] * a2[2], pl[3]]);
        let v1;
        if f12 < f1 {
            v1 = m.r32(s1 + 4);
        } else if f1 < -f12 {
            v1 = m.r32(s1);
        } else {
            let v1 = m.r32(s1);
            let mut s2 = if v1 < a1 { v1 } else { 0 };
            let s0;
            if s2 != 0 {
                s0 = m.call(0x0011_8a88, &[s2, a1, a3, t0], &[a2, f4(f12)]).0;
                s2 = m.r32(t0).wrapping_sub(0xc);
            } else {
                s0 = 0;
            }
            let v1 = m.r32(s1 + 4);
            let mut c = if v1 < a1 { v1 } else { 0 };
            let r;
            if c != 0 {
                r = m.call(0x0011_8a88, &[c, a1, a3, t0], &[a2, f4(f12)]).0;
                c = m.r32(t0).wrapping_sub(0xc);
            } else {
                r = 1;
            }
            if s0 == r && r != 2 {
                stack_pop(m, 0x30);
                return r;
            }
            if (m.r32(a3) as i32) <= 0 {
                stack_pop(m, 0x30);
                return 1;
            }
            let rec = m.r32(t0);
            m.w32(t0, rec.wrapping_add(0xc));
            let n = m.r32(a3).wrapping_sub(1);
            m.w32(a3, n);
            let p = m.r32(s1 + 8);
            m.w32(rec + 8, p);
            match s0 {
                1 => m.w32(rec, 0x002b_bb10),
                0 => m.w32(rec, 0),
                2 => m.w32(rec, s2),
                _ => {}
            }
            match r {
                1 => m.w32(rec + 4, 0),
                0 => m.w32(rec + 4, 0x002b_bb20),
                2 => m.w32(rec + 4, c),
                _ => {}
            }
            stack_pop(m, 0x30);
            return 2;
        }
        s1 = if v1 < a1 { v1 } else { 0 };
    }
    stack_pop(m, 0x30);
    (0.0 < f1) as u32
}

/// FUN_001d9b60(o, state): state change on +0x30 driving the +0x38 animator (vslot 48) per jump table 0x2a7910.
pub fn ee_sub_1d9b60<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let s1 = a0;
    let v1 = m.r32(s1 + 0x30);
    if v1 == a1 {
        return;
    }
    let a2 = m.r32(s1 + 0x38);
    if a2 == 0 {
        return;
    }
    if v1 == 2 || v1 == 6 {
        let x = m.r32(s1 + 0x3c);
        vslot(m, a2, 48, &[x], &[]);
    }
    m.w32(s1 + 0x30, a1);
    if a1 >= 8 {
        return;
    }
    let field = match a1 {
        0 | 5 => 0x44,
        2 => 0x40,
        6 => 0x48,
        7 => 0x4c,
        3 | 4 => return,
        _ => {
            // case 1
            let mut s5: i32 = 0;
            let mut s0: u32 = 0;
            let a2 = m.r32(s1 + 0x38);
            let x = m.r32(s1 + 0x3c);
            vslot(m, a2, 48, &[x], &[]);
            let p = m.r32(s1 + 0x38);
            m.call(0x0021_3b00, &[p, 0x34f], &[]);
            let f0 = m.rf(s1 + 0x20);
            let p = m.r32(s1 + 0x38);
            m.call(0x0021_2e40, &[p, 0x34f], &[f4(1.0 - f0)]);
            let p = m.r32(s1 + 0x34);
            if m.call(0x0013_2b10, &[p, 0x35], &[]).0 != 0 {
                s5 = 0x12;
                s0 = 0x002c_2de8;
            } else {
                let p = m.r32(s1 + 0x34);
                if m.call(0x0013_2b10, &[p, 0x37], &[]).0 != 0 {
                    s5 = 9;
                    s0 = 0x002c_3028;
                } else {
                    let p = m.r32(s1 + 0x34);
                    if m.call(0x0013_2b10, &[p, 0x36], &[]).0 != 0 {
                        s5 = 8;
                        s0 = 0x002c_3148;
                    }
                }
            }
            let mut s2 = s0;
            let s6: u32 = 0x002b_cd00;
            let mut s4 = 0;
            let mut s3: i32 = 0;
            let sp = stack_push(m, 0x20);
            while s3 < s5 {
                let id = m.r32(s2);
                if m.call(0x001d_aea8, &[id], &[]).0 == 0 {
                    s2 = s2.wrapping_add(0x20);
                    s3 += 1;
                    continue;
                }
                let e = (id << 5).wrapping_add(s6);
                for k in (0..0x20).step_by(4) {
                    let w = m.r32(e + k);
                    m.w32(sp + k, w);
                }
                let c = m.r32(sp + 0xc) as i32;
                s2 = s2.wrapping_add(0x20);
                if c <= 0 {
                    s3 += 1;
                    continue;
                }
                s4 = 1;
                break;
            }
            stack_pop(m, 0x20);
            let p = m.r32(s1 + 0x38);
            if s4 != 0 {
                m.call(0x0021_3b00, &[p, 0x350], &[]);
                let f0 = m.rf(s1 + 0x28);
                let p = m.r32(s1 + 0x38);
                m.call(0x0021_2e40, &[p, 0x350], &[f4(1.0 - f0)]);
            } else {
                m.call(0x0021_3928, &[p, 0x350], &[]);
            }
            return;
        }
    };
    let a2 = m.r32(s1 + 0x38);
    let x = m.r32(s1 + field);
    vslot(m, a2, 48, &[x], &[]);
}

/// FUN_00247760(x): asinf (fdlibm e_asinf).
pub fn ee_sub_247760<M: QMem + ?Sized>(m: &mut M, f12: f32) -> f32 {
    let h = |b: u32| f32::from_bits(b);
    let s1 = f12.to_bits() as i32;
    let s0 = (s1 as u32 & 0x7fff_ffff) as i32;
    let (ps0, ps1, ps2, ps3, ps4, ps5) =
        (h(0x3e2a_aaab), h(0xbea6_b090), h(0x3e4e_0aa8), h(0xbd24_1146), h(0x3a4f_7f04), h(0x3811_ef08));
    let (qs1, qs2, qs3, qs4) = (h(0xc019_d139), h(0x4001_572d), h(0xbf30_3361), h(0x3d9d_c62e));
    let pio2_hi = h(0x3fc9_0fda);
    let pio2_lo = h(0x33a2_2168);
    let pio4_hi = h(0x3f49_0fdb);
    if s0 == 0x3f80_0000 {
        return f12 * pio2_hi + f12 * pio2_lo;
    }
    if 0x3f80_0000 < s0 {
        let f0 = f12 - f12;
        return crate::engine::math::ps2f::ps2_div(f0, f0);
    }
    if !(0x3eff_ffff < s0) {
        if !(0x31ff_ffff < s0) {
            let f0 = f12 + h(0x7149_f2ca);
            if 1.0 < f0 {
                return f12;
            }
        }
        let t = f12 * f12;
        let p = t * (ps0 + t * (ps1 + t * (ps2 + t * (ps3 + t * (ps4 + t * ps5)))));
        let q = 1.0 + t * (qs1 + t * (qs2 + t * (qs3 + t * qs4)));
        let w = p / q;
        return f12 + f12 * w;
    }
    let w = m.call(0x0024_4ef8, &[], &[f4(f12)]).1[0];
    let t = (1.0 - w) * 0.5;
    let p = t * (ps0 + t * (ps1 + t * (ps2 + t * (ps3 + t * (ps4 + t * ps5)))));
    let q = 1.0 + t * (qs1 + t * (qs2 + t * (qs3 + t * qs4)));
    let s = m.call(0x0024_8ca8, &[], &[f4(t)]).1[0];
    let r;
    if 0x3f79_9999 < s0 {
        let w = p / q;
        r = pio2_hi - ((s + s * w) + (s + s * w) - pio2_lo);
    } else {
        let f = f32::from_bits(s.to_bits() & 0xffff_f000);
        let c = (t - f * f) / (s + f);
        let rr = p / q;
        let pp = (s + s) * rr - (pio2_lo - (c + c));
        let qq = pio4_hi - (f + f);
        r = pio4_hi - (pp - qq);
    }
    if s1 > 0 { r } else { -r }
}

/// FUN_001baf18(o, target): step the +0x15c cursor through the event table +0x164 (stride 0x10, {id, _, n,
/// actions}) to the target index, firing (forward, table 0x2a7640) or undoing (backward, 0x2a7660) actions.
pub fn ee_sub_1baf18<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let s1 = a0;
    let mut s4 = m.r32(s1 + 0x15c) as i32;
    let a2: i32;
    if a1 == 0x609 {
        s4 = 0;
        a2 = m.r32(s1 + 0x15c) as i32;
    } else if a1 == 0x60a {
        s4 = m.r32(s1 + 0x160) as i32;
        a2 = m.r32(s1 + 0x15c) as i32;
    } else if a1 == 0xffff_ffff {
        a2 = s4;
    } else {
        a2 = s4;
        let n = m.r32(s1 + 0x160) as i32;
        if n > 0 {
            let t = m.r32(s1 + 0x164);
            if m.r32(t) == a1 {
                s4 = 0;
            } else {
                let mut i: i32 = 1;
                while i < n {
                    if m.r32(((i as u32) << 4).wrapping_add(t)) == a1 {
                        s4 = i;
                        break;
                    }
                    i += 1;
                }
            }
        }
    }
    if a2 == s4 {
        return;
    }
    // per-action: returns false when the actor lookup failed (skip)
    let act = |m: &mut M, s0: u32, fwd: bool| {
        let ty = m.r32(s0);
        let mut a2 = 0u32;
        if (0xc3u32 >> (ty & 31)) & 1 != 0 {
            let id = m.r32(s0 + 4);
            a2 = m.call(0x0015_e938, &[GS, 1, id, s1], &[]).0;
            if a2 == 0 {
                return;
            }
        }
        let ty = m.r32(s0);
        if ty >= 6 {
            return;
        }
        let x = m.r32(s0 + 4);
        match (ty, fwd) {
            (0, true) | (1, false) => {
                vslot(m, a2, 7, &[], &[]);
            }
            (1, true) | (0, false) => {
                vslot(m, a2, 8, &[], &[]);
            }
            (2, true) | (3, false) => {
                m.call(0x001f_6978, &[s1, x], &[]);
            }
            (3, true) | (2, false) => {
                m.call(0x001f_69f8, &[s1, x], &[]);
            }
            (4, true) | (5, false) => {
                m.call(0x001f_6ac8, &[s1, x], &[]);
            }
            _ => {
                m.call(0x001f_6b48, &[s1, x], &[]);
            }
        }
    };
    if a2 < s4 {
        let mut a1 = a2;
        loop {
            m.w32(s1 + 0x15c, (a1 + 1) as u32);
            let s3 = m.r32(s1 + 0x164).wrapping_add((a1 as u32) << 4);
            let mut s2: i32 = 0;
            if (m.r32(s3 + 8) as i32) > 0 {
                loop {
                    let s0 = m.r32(s3 + 0xc).wrapping_add((s2 as u32) << 3);
                    act(m, s0, true);
                    s2 += 1;
                    if !(s2 < m.r32(s3 + 8) as i32) {
                        break;
                    }
                }
            }
            a1 = m.r32(s1 + 0x15c) as i32;
            if !(a1 < s4) {
                break;
            }
        }
    } else {
        let mut a2 = a2;
        loop {
            let v = a2.wrapping_sub(1);
            m.w32(s1 + 0x15c, v as u32);
            let s3 = m.r32(s1 + 0x164).wrapping_add((v as u32) << 4);
            let mut s2 = m.r32(s3 + 8) as i32;
            loop {
                let v0 = s2;
                s2 = s2.wrapping_sub(1);
                if !(v0 > 0) {
                    break;
                }
                let s0 = m.r32(s3 + 0xc).wrapping_add((s2 as u32) << 3);
                act(m, s0, false);
            }
            a2 = m.r32(s1 + 0x15c) as i32;
            if !(s4 < a2) {
                break;
            }
        }
    }
    m.w32(s1 + 0x158, 0);
}

/// FUN_00195368(_, o, n, emitters): spawn up to 10 particles from the n emitters (stride 0x30) of o[0x12f8]
/// and hand them to FUN_0015b428.
pub fn ee_sub_195368<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: i32, a3: u32) {
    let s7 = m.r32(a1 + 0x12f8);
    if s7 == 0 {
        return;
    }
    if m.r32(s7 + 0xe0) != 0 {
        m.call(0x0015_a1c0, &[s7], &[]);
    }
    let s1 = m.r32(s7 + 0xe4).wrapping_add(0x50);
    let sp = stack_push(m, 0x220);
    let mut s4: u32 = 0;
    let mut i: i32 = 0;
    'outer: while i < a2 {
        let s6 = a3.wrapping_add((i as u32).wrapping_mul(0x30));
        i += 1;
        if m.r32(s6 + 0xc) == 0xffff_ffff {
            continue;
        }
        let mut f20 = m.rf(s7 + 0xfc) * m.rf(0x002d_ded4);
        let f0 = m.call(0x0011_d448, &[], &[f4(0.0), f4(1.0)]).1[0];
        let mut s0 = f20 as i32;
        f20 = f20 - s0 as f32;
        if f0 < f20 {
            s0 = s0.wrapping_add(1);
        }
        s0 = s0.wrapping_sub(1);
        while s0 >= 0 {
            let s2 = sp + 0x10 + s4.wrapping_mul(16);
            let s5 = sp + 0x180 + s4.wrapping_mul(4);
            let s3 = sp + 0x150 + s4.wrapping_mul(4);
            let a3 = m.r32(s6 + 4);
            let vt = m.r32(a3 + 0x20);
            let adj = m.r16(vt + 0x28) as i16 as i32 as u32;
            let f = m.r32(vt + 0x2c);
            let x = m.rf(s6 + 0x18);
            m.call(f, &[a3.wrapping_add(adj), sp + 0x1b0, sp + 0x1c0], &[f4(x)]);
            let (x, y) = (m.rf(s1), m.rf(s1 + 4));
            let k = m.call(0x0011_d448, &[], &[f4(x), f4(y)]).1[0];
            let d = m.rv(sp + 0x1c0);
            let v1 = [d[0] * k, d[1] * k, d[2] * k, d[3] * k];
            m.wv(sp + 0x1e0, [k, 0.0, 0.0, 0.0]);
            m.wv(sp + 0x1d0, v1);
            let mut f20 = m.rf(0x002d_ded0);
            let (x, y) = (m.rf(s1 + 0x50), m.rf(s1 + 0x54));
            let f0 = m.call(0x0011_d448, &[], &[f4(x), f4(y)]).1[0];
            f20 = f20 - f0;
            m.wf(s3, f20);
            f20 = f20 + m.rf(s1 + 0x3c);
            m.wf(s5, f20);
            let (x, y) = (m.rf(s1 + 0x48), m.rf(s1 + 0x4c));
            let v0 = if x == y {
                m.rv(sp + 0x1b0)
            } else {
                let p = m.rv(sp + 0x1b0);
                let k = m.call(0x0011_d448, &[], &[f4(x), f4(y)]).1[0];
                let v1 = m.rv(sp + 0x1d0);
                let r = [p[0] + v1[0] * k, p[1] + v1[1] * k, p[2] + v1[2] * k, p[3] + v1[3] * k];
                m.wv(sp + 0x1e0, [k, 0.0, 0.0, 0.0]);
                m.wv(sp + 0x1b0, r);
                r
            };
            s4 += 1;
            m.wv(s2, v0);
            let v = m.rv(sp + 0x1d0);
            m.wv(s2 + 0xa0, v);
            if !(s4 < 10) {
                break 'outer;
            }
            s0 = s0.wrapping_sub(1);
        }
    }
    if (s4 as i32) > 0 {
        let s0 = sp + 0x1b0;
        m.call(0x0023_33b4, &[s0, 0, 0xc], &[]);
        m.w32(sp + 0x1b8, s7);
        m.w32(sp + 0x1b0, s7.wrapping_add(0x150));
        m.w32(sp, 0);
        let p = m.r32(s7 + 0xe4);
        m.call(0x0015_b428, &[p, s0, s4, sp + 0x10, sp + 0xb0, sp + 0x150, sp + 0x180, 0, 0], &[]);
    }
    stack_pop(m, 0x220);
}

/// FUN_001afb88(o, msg): message handler (jump table 0x2a7030, msg 0x3f..0x68) that pushes option values
/// through FUN_00258200 into 0x3d4a60, then FUN_001d8ff8(0x3d4a60).
pub fn ee_sub_1afb88<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    const T: u32 = 0x003d_4a60;
    let sp = stack_push(m, 0x30);
    let emit = |m: &mut M, tag: u32, val: u32, x: u32| {
        m.w32(sp, tag);
        m.w32(sp + 4, val);
        m.call(0x0012_91a8, &[sp + 0x10, sp], &[]);
        m.call(0x0025_8200, &[sp + 0x20, x, sp + 0x10, T], &[]);
        m.call(0x0012_9ec8, &[sp + 0x20], &[]);
        m.call(0x0012_9ec8, &[sp + 0x10], &[]);
        m.call(0x0012_9ec8, &[sp], &[]);
    };
    let msg = m.r32(a1);
    match msg {
        0x67 | 0x68 => {
            let (fa, xa) = if msg == 0x67 { (0x003d_d190, 0x003e_0fc4) } else { (0x003d_d18c, 0x003e_0fc8) };
            let f = 1.0 - m.rf(fa);
            let x = m.r32(xa);
            emit(m, 0x16, f.to_bits(), x);
            m.call(0x001a_eaa8, &[a0, 9], &[]);
        }
        0x3f => {
            let f = m.rf(0x002f_6468) - f32::from_bits(0x441f_0000);
            let x = m.r32(GS + 0x3b40);
            emit(m, 0x16, f.to_bits(), x);
            let f = m.rf(0x002f_646c) - 50.0;
            let x = m.r32(GS + 0x3b44);
            emit(m, 0x16, f.to_bits(), x);
            m.call(0x001a_eaa8, &[a0, 9], &[]);
        }
        0x4c | 0x4d | 0x4e => {
            let v = match msg {
                0x4c => 1,
                0x4d => 2,
                _ => 0,
            };
            let x = m.r32(0x003e_0fc0);
            emit(m, 0x15, v, x);
        }
        0x48 | 0x49 => {
            let b = (msg == 0x49) as u32;
            m.w32(0x002c_2d8c, if b != 0 { 3 } else { 2 });
            let x = m.r32(0x003e_0fd8);
            emit(m, 0x17, b, x);
        }
        0x4f | 0x50 => {
            let x = m.r32(0x003e_0fcc);
            emit(m, 0x17, (msg == 0x4f) as u32, x);
        }
        0x51 | 0x52 => {
            let x = m.r32(0x003e_0fdc);
            emit(m, 0x17, (msg == 0x52) as u32, x);
        }
        0x53 | 0x54 => {
            let x = m.r32(0x003e_0fe0);
            emit(m, 0x17, (msg == 0x54) as u32, x);
        }
        _ => {}
    }
    stack_pop(m, 0x30);
    m.call(0x001d_8ff8, &[T], &[]);
}

/// FUN_001b93e8(o, state): set the +0x1450 state (0 idle, 1 / 2 grab the +0x13f0 target), drive the
/// +0x1088 flag.
pub fn ee_sub_1b93e8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let s2 = a0;
    let s0 = a1;
    let old = m.r32(s2 + 0x1450);
    if old == s0 {
        return;
    }
    if old == 1 {
        m.call(0x0012_e460, &[s2 + 0x13fc], &[]);
        m.call(0x0012_e460, &[s2 + 0x1400], &[]);
    }
    m.w32(s2 + 0x1450, s0);
    let f = m.rf(0x002d_ded0);
    m.wf(s2 + 0x1454, f);
    let sp = stack_push(m, 0x90);
    // target actor with the +0x2c bit 28 set, else 0
    let tgt = |m: &mut M| -> u32 {
        let a0 = m.r32(s2 + 0x13f0);
        if a0 == 0 {
            return 0;
        }
        let s3 = m.r32(a0 + 0x180);
        if (m.r64(s3 + 0x28) >> 60) & 1 == 0 {
            return 0;
        }
        s3
    };
    let mut drop = false;
    match s0 {
        0 => m.w32(s2 + 0x1448, 0),
        1 => {
            let s3 = tgt(m);
            if s3 == 0 {
                drop = true;
            } else {
                m.call(0x001f_5720, &[s3, 0], &[]);
                let p = m.r32(s3 + 0x58);
                let v = m.r16(p + 0x98) as i16 as i32 as u32;
                m.w32(s2 + 0x1404, v);
                let p = m.r32(s3 + 0x5c);
                let v = m.r16(p + 0x98) as i16 as i32 as u32;
                m.w32(s2 + 0x1408, v);
                m.call(0x001a_58d0, &[s3, 0, 0xf], &[]);
                m.call(0x001f_6860, &[s3, 1], &[]);
                let p = m.r32(s2 + 0x1030);
                let p = m.r32(p + 0x14);
                let s1 = m.call(0x0012_f938, &[p, 0x1c7], &[]).0;
                let a1 = m.r32(s2 + 0x60);
                let q = m.r32(s2 + 0x1030);
                let f0 = (m.rf(s1) - m.rf(q + 0x1c)) / m.rf(q + 0x40);
                let p = m.r32(s2 + 0x13ec);
                let a0 = m.r32(p + 0x60);
                m.wf(sp + 0x80, f0);
                m.call(0x0023_09e8, &[a0, a1, sp, 0], &[f4(f0)]);
                let s0 = sp + 0x40;
                m.call(0x001a_71b0, &[s0, s2, 0, sp], &[]);
                for k in (0..0x40).step_by(4) {
                    let w = m.r32(sp + 0x40 + k);
                    m.w32(sp + k, w);
                }
                m.call(0x001b_72b0, &[s2, 0, sp, s0, sp + 0x50], &[]);
                if m.call(0x0013_2b10, &[s3, 0x38], &[]).0 != 0 {
                    m.call(0x0020_9e28, &[s3, s2], &[]);
                }
                let q50 = m.rv(sp + 0x50);
                m.call(0x001b_7700, &[s2, s1, sp + 0x80, sp + 0x60, sp + 0x70], &[q50]);
                let f12 = m.rf(0x002d_ded0) + m.rf(sp + 0x80);
                let q50 = m.rv(sp + 0x50);
                let q70 = m.rv(sp + 0x70);
                let a1 = m.r32(s3 + 0x58);
                let r = m.call(0x0012_de58, &[s3, a1, 5], &[q50, q70, f4(f12)]).0;
                m.w32(s2 + 0x13fc, r);
                let q60 = m.rv(sp + 0x60);
                let p = m.r32(s3 + 0x58);
                let p = m.r32(p + 0xa0);
                let r = m.call(0x0022_fcc8, &[p, 0], &[q60]).0;
                vslot(m, s3, 34, &[r], &[]);
                let q40 = m.rv(sp + 0x40);
                let a1 = m.r32(s3 + 0x5c);
                let r = m.call(0x0012_dc18, &[s3, a1, 5, 7, 2], &[q40]).0;
                m.w32(s2 + 0x1400, r);
            }
        }
        2 => {
            let s3 = tgt(m);
            if s3 == 0 {
                drop = true;
            } else {
                let s0 = m.r32(s2 + 0x13ec);
                m.call(0x001a_58d0, &[s3, s0, 0xf], &[]);
                m.call(0x001f_6860, &[s3, 1], &[]);
                let a1 = m.r32(s0 + 0x60);
                m.call(0x001b_72b0, &[s2, a1, 0x002a_2d30, sp + 0x10, sp], &[]);
                let q = m.rv(sp + 0x10);
                let a1 = m.r32(s3 + 0x5c);
                let r = m.call(0x0012_dc18, &[s3, a1, 5, 7, 2], &[q]).0;
                m.w32(s2 + 0x13f8, r);
                let q = m.rv(sp);
                let a1 = m.r32(s3 + 0x58);
                let r = m.call(0x0012_dc18, &[s3, a1, 5, 7, 2], &[q]).0;
                m.w32(s2 + 0x13f4, r);
                for &(o, so, fa, r1, r2, r3a, r3) in &[
                    (0x190u32, 0x1610u32, 0x002c_0a3cu32, 0x1618u32, 0x161cu32, 0x15f4u32, 0x1604u32),
                    (0x198, 0x1614, 0x002c_0a40, 0x1620, 0x1624, 0x15fc, 0x160c),
                ] {
                    let p = m.r32(s2 + 0x13f0);
                    let a2 = m.r32(p + o);
                    if a2 == 0 {
                        continue;
                    }
                    let s1 = sp + 0x20;
                    let s0 = m.r32(s2 + so);
                    let r = vslot(m, a2, 31, &[s1], &[]).0;
                    m.w32(s0, r);
                    let f = m.rf(fa);
                    let q = m.rv(sp + 0x50);
                    m.wf(s0 + 0xcc, f);
                    m.wv(s0 + 0xa0, q);
                    let a1 = m.r32(s0 + 4);
                    let r = m.call(0x0012_dd28, &[s2, a1, 5, s0], &[]).0;
                    m.w32(s2 + r1, r);
                    let a1 = m.r32(s0 + 8);
                    let r = m.call(0x0012_dd28, &[s2, a1, 5, s0], &[]).0;
                    m.w32(s2 + r2, r);
                    let a1 = m.r32(s2 + r3a);
                    let t0 = m.r32(s0);
                    let r = m.call(0x0012_ddb0, &[s2, a1, 5, 5, t0, s1], &[]).0;
                    m.w32(s2 + r3, r);
                }
                if m.call(0x0013_2b10, &[s3, 0x38], &[]).0 != 0 {
                    m.call(0x0020_9e88, &[s3], &[]);
                }
                m.w32(s2 + 0x194, 0);
                m.w32(s2 + 0x198, 0);
                m.w32(s2 + 0x190, 0);
            }
        }
        _ => {}
    }
    stack_pop(m, 0x90);
    if drop {
        m.call(0x001b_7d20, &[s2], &[]);
        m.w32(s2 + 0x13f0, 0);
        m.call(0x001b_6078, &[s2, 0], &[]);
    }
    let v1 = m.r32(s2 + 0x1450) as i32;
    let p = m.r32(s2 + 0x1088);
    if v1 < 2 && v1 >= 0 {
        m.w32(p + 0x1c, 1);
    } else {
        m.w32(p + 0x1c, 0);
    }
}

/// FUN_0015cf30(o): menu / selector input update (mode +0x11c: 2 list, 8 sub-list, 0xa confirm, else
/// default), pad 0x2dfc00.
pub fn ee_sub_15cf30<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    const P: u32 = 0x002d_fc00;
    let s1 = a0;
    m.call(0x0021_2c50, &[s1], &[]);
    for &p in &[0x003e_9ed0u32, 0x002e_6550, 0x003e_a4f0] {
        if m.call(0x001d_9880, &[p], &[]).0 != 0 {
            return;
        }
    }
    m.call(0x0015_d5c8, &[s1], &[]);
    let l3a4 = |m: &mut M, a0: u32| {
        let h = m.call(0x001f_bb90, &[a0], &[]).0;
        m.call(0x001f_b2f8, &[h, 0], &[]);
    };
    let l544 = |m: &mut M, a1: u32| {
        m.call(0x0015_d648, &[s1, a1, 0xffff_ffff], &[]);
    };
    let mode = m.r32(s1 + 0x11c) as i32;
    match mode {
        2 => {
            let f1 = m.call(0x0018_80d8, &[P], &[]).0 as i32 as f32;
            if f1 < 0.0 && (m.r32(s1 + 0x110) as i32) <= 0 {
                return l3a4(m, 0x36);
            }
            if 0.0 < f1 && !((m.r32(s1 + 0x110) as i32) < (m.r32(s1 + 0x118) as i32)) {
                return l3a4(m, 0x36);
            }
            if f1 != 0.0 {
                let v = m.r32(s1 + 0x110);
                let v = if f1 < 0.0 { v.wrapping_sub(1) } else { v.wrapping_add(1) };
                m.w32(s1 + 0x114, v);
                return l544(m, if 0.0 < f1 { 4 } else { 6 });
            }
            if m.call(0x0018_7d80, &[P, 0x40], &[]).0 != 0 {
                return l544(m, 7);
            }
            if m.r32(s1 + 0x110) != 0 && m.call(0x0018_7d80, &[P, 0x10], &[]).0 != 0 {
                m.w32(s1 + 0x114, 0);
                return l544(m, 6);
            }
            match m.r32(s1 + 0x12c) {
                0 => {
                    let idx = m.r32(s1 + 0x110) << 2;
                    let s2 = m.r32(0x002b_e248u32.wrapping_add(idx));
                    let a1 = m.r32((GS + 0x3b7c).wrapping_add(idx));
                    let mut s3 = 0;
                    let mut big = false;
                    if a1 != 0 {
                        let sp = stack_push(m, 0x10);
                        m.call(0x0025_81c0, &[sp, a1, 0x003d_4a60], &[]);
                        let s0 = m.r32(sp + 4) as i32;
                        m.call(0x0012_9ec8, &[sp], &[]);
                        stack_pop(m, 0x10);
                        if !(s0 < 0x64) {
                            s3 = 1;
                            big = true;
                        }
                    }
                    if !big && m.r32(0x002b_e26c) != 0 {
                        s3 = 1;
                    }
                    if s2 == 0 || s3 == 0 {
                        return;
                    }
                    if 10.0 < m.rf(0x002d_df0c) - m.rf(s1 + 0x130) {
                        m.call(0x0015_d880, &[s1, 1], &[]);
                    }
                }
                1 => {
                    if m.call(0x0018_7d80, &[P, 0x80], &[]).0 != 0 {
                        let idx = m.r32(s1 + 0x110) << 2;
                        let a1 = m.r32(0x002b_e248u32.wrapping_add(idx));
                        m.call(0x001b_3a48, &[0x002f_66b0, a1], &[]);
                        m.call(0x0022_b490, &[GS, 0, 0x189, 0xffff_ffff, 0, 0xffff_ffff], &[]);
                        return l3a4(m, 0x34);
                    }
                    if 2.0 < m.rf(0x002d_df0c) - m.rf(s1 + 0x130) {
                        m.call(0x0015_d880, &[s1, 0], &[]);
                    }
                }
                _ => {}
            }
        }
        8 => {
            let v1 = m.r32(s1 + 0x120) as i32;
            match v1 {
                0 => {
                    if m.call(0x0018_7d80, &[P, 0x40], &[]).0 != 0 {
                        let a2 = m.r32(s1 + 0x128);
                        let a1 = m.r32(0x002b_e200u32.wrapping_add(a2 << 2));
                        m.w32(0x002b_e270, a2);
                        m.call(0x001b_3a48, &[0x002f_66b0, a1], &[]);
                        m.call(0x0022_b490, &[GS, 0, 0x189, 0xffff_ffff, 0, 0xffff_ffff], &[]);
                        return l3a4(m, 0x34);
                    }
                    if m.call(0x0018_7d80, &[P, 0x10], &[]).0 != 0 {
                        return l544(m, 9);
                    }
                    let f20 = m.call(0x0018_7e88, &[P], &[]).0 as i32 as f32;
                    if f20 < 0.0 && (m.r32(s1 + 0x128) as i32) <= 0 {
                        return l3a4(m, 0x36);
                    }
                    if 0.0 < f20 {
                        let v0 = m.call(0x0015_d798, &[s1], &[]).0 as i32;
                        if !((m.r32(s1 + 0x128) as i32) < v0) {
                            return l3a4(m, 0x36);
                        }
                    }
                    if f20 == 0.0 {
                        return;
                    }
                    let v = m.r32(s1 + 0x128);
                    let v = if f20 < 0.0 { v.wrapping_sub(1) } else { v.wrapping_add(1) };
                    m.w32(s1 + 0x128, v);
                    m.call(0x0015_d648, &[s1, 8, if 0.0 < f20 { 2 } else { 1 }], &[]);
                }
                -1 => {
                    if m.call(0x0018_7d80, &[P, 0x40], &[]).0 != 0 {
                        return l544(m, 0xa);
                    }
                    if m.call(0x0018_7d80, &[P, 0x10], &[]).0 != 0 {
                        return l544(m, 9);
                    }
                }
                1 | 2 => {
                    if m.call(0x0021_21e8, &[s1], &[]).0 == 0 {
                        m.call(0x0015_d648, &[s1, 8, 0], &[]);
                    }
                }
                _ => {}
            }
        }
        0xa => {
            if m.call(0x0021_21e8, &[s1], &[]).0 != 0 {
                return;
            }
            let idx = m.r32(s1 + 0x110) << 2;
            let a1 = m.r32((GS + 0x3b58).wrapping_add(idx));
            let mut s2 = 0;
            if a1 != 0 {
                s2 = 1;
                let sp = stack_push(m, 0x10);
                m.call(0x0025_81c0, &[sp, a1, 0x003d_4a60], &[]);
                let f20 = m.rf(sp + 4);
                m.call(0x0012_9ec8, &[sp], &[]);
                stack_pop(m, 0x10);
                if f20 != 0.0 {
                    s2 = 0;
                }
            }
            if s2 != 0 {
                let idx = m.r32(s1 + 0x110) << 2;
                let a2 = m.r32(0x002b_e2a8u32.wrapping_add(idx));
                let s0 = m.call(0x0015_e938, &[GS, 5, a2, 0], &[]).0;
                if s0 != 0 {
                    m.call(0x0022_8358, &[s0, 3], &[]);
                    m.call(0x0022_83d0, &[s0], &[]);
                } else {
                    s2 = 0;
                }
            }
            if s2 == 0 {
                let a0 = m.r32(s1 + 0x110);
                let v = m.call(0x0021_0b98, &[a0], &[]).0;
                m.call(0x0022_b490, &[GS, v, 0x189, 0xffff_ffff, 0, 0x30a], &[]);
            }
            m.call(0x0015_d648, &[s1, 0xffff_ffff, 0xffff_ffff], &[]);
        }
        -1 => {}
        _ => {
            if m.call(0x0021_21e8, &[s1], &[]).0 != 0 {
                return;
            }
            let k = m.r32(s1 + 0x11c) << 2;
            let a1 = m.r32(0x002b_e278u32.wrapping_add(k));
            let mut a2 = 0xffff_ffff;
            if a1 == 8 && m.r32(s1 + 0x110) == 0 {
                m.w32(s1 + 0x128, 0);
                a2 = 0;
            }
            m.call(0x0015_d648, &[s1, a1, a2], &[]);
        }
    }
}

/// FUN_001ec968: walk the list at DAT_003e159c (next +0x40) while node+0x30 == 0; FUN_001eda20(node) != 0
/// -> vslot 7.
pub fn ee_sub_1ec968<M: QMem + ?Sized>(m: &mut M) {
    let mut s0 = m.r32(0x003e_159c);
    while s0 != 0 && m.r32(s0 + 0x30) == 0 {
        if m.call(0x001e_da20, &[s0], &[]).0 != 0 {
            vslot(m, s0, 7, &[], &[]);
        }
        s0 = m.r32(s0 + 0x40);
    }
}

/// FUN_001d7c80(hdr, save): byte-sum of hdr[0]*10 bytes at hdr[1]; clear save (0x5400) when the
/// sum or the trailing u16 tag differs from save+0x5404 / +0x5400.
pub fn ee_sub_1d7c80<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let mut s1: u32 = 0;
    let mut v1 = m.r32(a0).wrapping_mul(10);
    let mut p = m.r32(a0 + 4);
    while v1 != 0 {
        let b = m.r8(p) as u32;
        v1 = v1.wrapping_sub(1);
        p = p.wrapping_add(1);
        s1 = s1.wrapping_add(b);
    }
    let v0 = m.r32(a0).wrapping_mul(10).wrapping_add(m.r32(a0 + 4));
    let s2 = m.r16(v0.wrapping_sub(6)) as u32;
    if s2 == m.r32(a1 + 0x5400) && s1 == m.r32(a1 + 0x5404) {
        return;
    }
    m.call(0x0023_33b4, &[a1, 0, 0x5400], &[]);
    m.w32(a1 + 0x5404, s1);
    m.w32(a1 + 0x5400, s2);
}

/// FUN_001ab1a0(o): card/clue hint update.
pub fn ee_sub_1ab1a0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    if m.call(0x0020_e040, &[GS], &[]).0 != 0 {
        return;
    }
    if m.r32(0x002d_eb4c) != 0 {
        return;
    }
    if m.r32(0x002e_6550 + 0x44) == 3 {
        return;
    }
    let s1 = a0.wrapping_add(0xfd5c);
    if m.call(0x001a_b088, &[s1], &[]).0 != 0 {
        return;
    }
    if m.r32(a0 + 8) == m.r32(a0 + 0xc) {
        let f1 = m.rf(a0 + 0x14);
        let f0 = m.rf(0x002d_df0c);
        if f1 <= f0 {
            // ldl/ldr: 64-bit a1 = *(u64*)0x2a6ba0; QMem int args are 32-bit, passed as (lo).
            let lo = m.r32(0x002a_6ba0);
            m.call(0x001a_af90, &[s1, lo], &[]);
        } else {
            m.call(0x001a_b630, &[a0], &[]);
        }
    } else {
        m.call(0x001a_b418, &[a0], &[]);
    }
    if m.call(0x001a_b088, &[a0.wrapping_add(0xfd5c)], &[]).0 != 0 {
        return;
    }
    if m.call(0x001b_42c0, &[0x002f_66b0], &[]).0 != 0 {
        return;
    }
    let s1 = 0x002e_6550u32;
    let s2 = m.r32(s1 + 0x44);
    if s2 != 1 {
        return;
    }
    let f0 = m.rf(0x002d_df0c) - m.rf(a0 + 0x10);
    if !(f32::from_bits(0x3e99_999a) <= f0) {
        return;
    }
    if m.call(0x001b_01e8, &[0x003d_9ed0], &[]).0 != 0 {
        if m.r32(a0 + 8) != 6 {
            m.call(0x001a_cb58, &[s1, 0x2e], &[]);
        }
        return;
    }
    let a = m.r32(a0 + 8) as i32;
    let id = match a {
        2 => 0x12,
        1 => 0x11,
        3 => 0x15,
        4 => 0x18,
        _ => return,
    };
    m.call(0x001a_cb58, &[s1, id], &[]);
}

/// FUN_001c4468(o): refresh pad/stick state (+0x80 mode: 0 -> FUN_001c42e8, 1 -> sample pad block) then
/// FUN_0011b720 -> o+0x40..0x70, FUN_001c4330 x2, FUN_001c45a8.
pub fn ee_sub_1c4468<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let sp = stack_push(m, 0x40);
    m.wf(sp + 0x30, 0.0);
    m.wf(sp + 0x34, 0.0);
    let v1 = m.r32(a0 + 0x80);
    if v1 == 0 {
        m.call(0x001c_42e8, &[a0], &[]);
    } else if v1 == 1 {
        let v = if m.r32(0x002d_fbc0) & 2 != 0 { 0x002d_fc00 } else { 0x002e_0d00 };
        let t0 = m.r32(a0 + 0x7c);
        let f = m.rf(v + 0x103c);
        m.wf(a0 + 0x10, f);
        let f = m.rf(v + 0x1040);
        m.wf(a0 + 0x14, f);
        let f = m.rf(v + 0x1070);
        m.wf(a0 + 0x20, f);
        let f = m.rf(v + 0x1074);
        m.wf(a0 + 0x24, f);
        let x = m.r32(v + 0x10bc);
        let y = m.r32(v + 0x10b4);
        let a3 = m.r32(t0 + 0x154);
        m.w32(a0 + 0x30, (y & !x) | a3);
        m.call(0x0011_bc88, &[0x002d_dfa0, sp + 0x30, sp + 0x34, a3], &[]);
    }
    let f12 = m.rf(sp + 0x30);
    let f13 = m.rf(sp + 0x34);
    m.call(0x0011_b720, &[sp], &[f4(f12), f4(f13)]);
    let q0 = m.rv(sp);
    let q1 = m.rv(sp + 0x10);
    let q2 = m.rv(sp + 0x20);
    let f12 = m.rf(sp + 0x30);
    m.wv(a0 + 0x40, q0);
    m.wv(a0 + 0x50, q1);
    m.wv(a0 + 0x60, q2);
    m.call(0x001c_4330, &[a0, a0 + 0x10, a0 + 0x14, a0 + 0x18, a0 + 0x1c, a0 + 0x70], &[f4(f12)]);
    let f12 = m.rf(sp + 0x30);
    m.call(0x001c_4330, &[a0, a0 + 0x20, a0 + 0x24, a0 + 0x28, a0 + 0x2c, a0 + 0x74], &[f4(f12)]);
    m.call(0x001c_45a8, &[a0], &[]);
    stack_pop(m, 0x40);
}
