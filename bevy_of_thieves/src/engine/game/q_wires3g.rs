//! WIRE-S3: 1:1 QMem ports of live-frontier natives (shard 5/6/7).
#![allow(unused)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

/// FUN_00237af8: lazy init DAT_002c4a84 = FUN_00234a20(&{_, 1, 1, .., name 0x2aaa98}) when it is -1.
pub fn ee_sub_237af8<M: QMem + ?Sized>(m: &mut M) {
    if m.r32(0x002c_4a84) == 0xffff_ffff {
        let sp = stack_push(m, 0x40);
        m.w32(sp + 4, 1);
        m.w32(sp + 0x14, 0x002a_aa98);
        m.w32(sp + 8, 1);
        let v = m.call(0x0023_4a20, &[sp], &[]).0;
        m.w32(0x002c_4a84, v);
        stack_pop(m, 0x40);
    }
}

/// FUN_00241f28: a0 != 0 -> FUN_00237488(0x2c82c0); else (DAT_002c6750 > 0 -> FUN_00236960(0x2ab520))
/// and poll FUN_00237488 sleeping FUN_00235518(4000) until 0; returns 0.
pub fn ee_sub_241f28<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    if a0 != 0 {
        return m.call(0x0023_7488, &[0x002c_82c0], &[]).0;
    }
    if (m.r32(0x002c_6750) as i32) > 0 {
        m.call(0x0023_6960, &[0x002a_b520], &[]);
    }
    while m.call(0x0023_7488, &[0x002c_82c0], &[]).0 != 0 {
        m.call(0x0023_5518, &[0xfa0], &[]);
    }
    0
}

/// FUN_002374e0: lazy init DAT_002c4a88 / DAT_002c4a8c via FUN_00234a20 (names 0x2aaa10 / 0x2aaa20).
pub fn ee_sub_2374e0<M: QMem + ?Sized>(m: &mut M) {
    if m.r32(0x002c_4a88) == 0xffff_ffff {
        let sp = stack_push(m, 0x40);
        m.w32(sp + 4, 1);
        m.w32(sp + 0x14, 0x002a_aa10);
        m.w32(sp + 8, 1);
        let v = m.call(0x0023_4a20, &[sp], &[]).0;
        m.w32(0x002c_4a88, v);
        m.w32(sp + 0x14, 0x002a_aa20);
        let v = m.call(0x0023_4a20, &[sp], &[]).0;
        m.w32(0x002c_4a8c, v);
        stack_pop(m, 0x40);
    }
}

/// FUN_00153128(o, s): state +0x50 change; leaving 1 -> FUN_0020f030(GS, 3); +0x54 = t, +0x1c0 = 0;
/// entering 1 -> FUN_0020efc0(GS, 3).
pub fn ee_sub_153128<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let v = m.r32(a0 + 0x50);
    if v == a1 {
        return;
    }
    if v == 1 {
        m.call(0x0020_f030, &[0x003d_d490, 3], &[]);
    }
    m.w32(a0 + 0x50, a1);
    let t = m.rf(0x002d_ded0);
    m.w32(a0 + 0x1c0, 0);
    m.wf(a0 + 0x54, t);
    if a1 == 1 {
        m.call(0x0020_efc0, &[0x003d_d490, 3], &[]);
    }
}

/// FUN_00236960(fmt, ...): printf-style: s = FUN_00239450(); FUN_00235b08(0x236878, &buf, -1, fmt, &va);
/// s != 0 -> FUN_002394a0().
pub fn ee_sub_236960<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, t0: u32, t1: u32, t2: u32, t3: u32) {
    let sp = stack_push(m, 0xc0);
    for (k, x) in [a1, a2, a3, t0, t1, t2, t3].iter().enumerate() {
        let p = sp + 0x88 + 8 * k as u32;
        m.w32(p, *x);
        m.w32(p + 4, if (*x as i32) < 0 { 0xffff_ffff } else { 0 });
    }
    m.w32(sp, 0);
    let s1 = m.call(0x0023_9450, &[], &[]).0;
    m.call(0x0023_5b08, &[0x0023_6878, sp, 0xffff_ffff, a0, sp + 0x88], &[]);
    if s1 != 0 {
        m.call(0x0023_94a0, &[], &[]);
    }
    stack_pop(m, 0xc0);
}

/// FUN_00241a30: lazy init DAT_002c6768/676c/6770 (any == -1) + DAT_002c6760 via FUN_00234a20; DAT_002c6774 = 0.
pub fn ee_sub_241a30<M: QMem + ?Sized>(m: &mut M) {
    let n = 0xffff_ffff;
    if m.r32(0x002c_6768) != n && m.r32(0x002c_676c) != n && m.r32(0x002c_6770) != n {
        return;
    }
    let sp = stack_push(m, 0x60);
    m.w32(sp + 4, 1);
    m.w32(sp + 0x14, 0x002a_b418);
    m.w32(sp + 8, 1);
    let v = m.call(0x0023_4a20, &[sp], &[]).0;
    m.w32(0x002c_6768, v);
    m.w32(sp + 0x14, 0x002a_b428);
    let v = m.call(0x0023_4a20, &[sp], &[]).0;
    m.w32(0x002c_676c, v);
    m.w32(sp + 0x14, 0x002a_b438);
    let v = m.call(0x0023_4a20, &[sp], &[]).0;
    m.w32(0x002c_6770, v);
    m.w32(sp + 0x14, 0x002a_b448);
    m.w32(sp + 8, 0);
    let v = m.call(0x0023_4a20, &[sp], &[]).0;
    m.w32(0x002c_6760, v);
    m.w32(0x002c_6774, 0);
    stack_pop(m, 0x60);
}

/// FUN_002375e0(a0..t3, a8, a9 (stack args)): retry FUN_00237288 up to 101 times while it fails
/// (negative) and a9 == 0, sleeping FUN_00235518(ms) with exponential backoff; returns failure bit.
pub fn ee_sub_2375e0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, t0: u32, t1: u32, t2: u32, t3: u32, a8: u32, a9: u32) -> u32 {
    let mut n = 0i32;
    let mut d = 1i32;
    loop {
        let v = m.call(0x0023_7288, &[a0, a1, a2, a3, t0, t1, t2, t3, a8], &[]).0;
        let fail = v >> 31;
        if fail == 0 || a9 != 0 {
            return fail;
        }
        m.call(0x0023_5518, &[d.wrapping_mul(1000) as u32], &[]);
        if d < 0x7f {
            d <<= 1;
        }
        n += 1;
        if n >= 0x65 {
            return fail;
        }
    }
}

/// FUN_0024a0e8: scalbnf(x = f12, n = a0); copysignf via FUN_0024a780.
pub fn ee_sub_24a0e8<M: QMem + ?Sized>(m: &mut M, a0: u32, f12: f32) -> f32 {
    let n = a0 as i32;
    let huge = f32::from_bits(0x7149_f2ca);
    let tiny = f32::from_bits(0x0da2_4260);
    let mut x = f12;
    let mut ix = x.to_bits();
    let mut k = ((ix & 0x7f80_0000) >> 23) as i32;
    if k == 0 {
        if ix & 0x7fff_ffff == 0 {
            return x;
        }
        x *= f32::from_bits(0x4c00_0000);
        ix = x.to_bits();
        k = ((ix & 0x7f80_0000) >> 23) as i32 - 25;
        if n < -50000 {
            return x * tiny;
        }
    }
    if k == 0xff {
        return x + x;
    }
    k = k.wrapping_add(n);
    let big = if k >= 0xff {
        huge
    } else if k > 0 {
        return f32::from_bits((ix & 0x807f_ffff) | ((k as u32) << 23));
    } else if k < -24 {
        if n > 50000 { huge } else { tiny }
    } else {
        k += 25;
        return f32::from_bits((ix & 0x807f_ffff) | ((k as u32) << 23)) * f32::from_bits(0x3300_0000);
    };
    let c = m.call(0x0024_a780, &[], &[f4(big), f4(x)]).1[0];
    c * big
}
