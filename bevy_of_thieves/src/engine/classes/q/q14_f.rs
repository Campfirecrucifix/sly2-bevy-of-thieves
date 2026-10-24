//! WP-Q14 part F: file packages run_0016d698 (GS clear / frame-buffer-copy packet builders) and the first
//! part of run_0016ef00. 1:1 from ghidra_functions(_v2).body on `QMem`.
#![allow(unused_variables, unused_mut, dead_code)]
use crate::engine::classes::q::q10_e::pool_static_init;
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem, V4};

// GS packet helpers are separate functions in the original (jal), so they go through the call table;
// the u64 ones use `QMem::call64` so the live host keeps all 64 bits.
fn jal_16d698<M: QMem + ?Sized>(m: &mut M, tag: u16, pk: u32, out: u32) {
    m.call(0x0016_d698, &[tag as i16 as i32 as u32, pk, out], &[]);
}
fn jal_16d6c8<M: QMem + ?Sized>(m: &mut M, pk: u32, f: u32, out: u32) {
    m.call(0x0016_d6c8, &[pk, f, out], &[]);
}
fn ee_sub_166920<M: QMem + ?Sized>(m: &mut M, p: u32) -> u32 {
    m.call(0x0016_6920, &[p], &[]).0
}
fn ee_sub_1669e0<M: QMem + ?Sized>(m: &mut M, p: u32, eop: u32) {
    m.call(0x0016_69e0, &[p, eop], &[]);
}
fn ee_sub_166bd8<M: QMem + ?Sized>(m: &mut M, p: u32, prim: u64, nreg: u64, regs: u64) {
    m.call64(0x0016_6bd8, &[p as u64, prim, nreg, regs]);
}
fn ee_sub_166e90<M: QMem + ?Sized>(m: &mut M, p: u32, reg: u64, data: u64) {
    m.call64(0x0016_6e90, &[p as u64, reg, data]);
}
fn ee_sub_166ef0<M: QMem + ?Sized>(m: &mut M, p: u32, n: u64, regs: u64) {
    m.call64(0x0016_6ef0, &[p as u64, n, regs]);
}
fn ee_sub_166fe8<M: QMem + ?Sized>(m: &mut M, p: u32, x: u64, y: i32) {
    m.call64(0x0016_6fe8, &[p as u64, x, y as i64 as u64]);
}
fn ee_sub_167048<M: QMem + ?Sized>(m: &mut M, p: u32, x: u64, y: i32, z: u64) {
    m.call64(0x0016_7048, &[p as u64, x, y as i64 as u64, z]);
}

/// FUN_0016d698(tag, pk, out): *out++ = { u16 tag, u16 ((pk+0xc) - (pk+4)) / 16 + 0x23 }
pub fn ee_sub_16d698<M: QMem + ?Sized>(m: &mut M, tag: u16, pk: u32, out: u32) {
    let q = m.r32(out);
    m.w32(out, q.wrapping_add(4));
    m.w16(q, tag);
    let d = (m.r32(pk + 0xc) as i32).wrapping_sub(m.r32(pk + 4) as i32) >> 4;
    m.w16(q + 2, (d as i16).wrapping_add(0x23) as u16);
}
/// FUN_0016d6c8(pk, flags, out): clear-rectangle GS setup (flags: 4 / 1 test mode, 0x40 / 0x10 column origin,
/// 0x100 / 0x200 mask presets, 0x10000..0x800000 channel masks)
pub fn ee_sub_16d6c8<M: QMem + ?Sized>(m: &mut M, pk: u32, f: u32, out: u32) {
    ee_sub_166bd8(m, pk, 0x200, 1, 0xe);
    jal_16d698(m, if f & 4 == 0 { 5 } else { 4 }, pk, out);
    if f & 0x200 == 0 {
        if f & 0x100 == 0 {
            let mut v: u64 = u64::MAX;
            if f & 0x80_0000 != 0 {
                v = 0x7fff_ffff;
            }
            if f & 0x40_0000 != 0 {
                v &= 0xffff_ffff_ff7f_ffff;
            }
            if f & 0x20_0000 != 0 {
                v &= 0xffff_ffff_ffbf_ffff;
            }
            if f & 0x10_0000 != 0 {
                v &= 0xffff_ffff_ffdf_ffff;
            }
            if f & 0x8_0000 != 0 {
                v &= 0xffff_ffff_ffef_ffff;
            }
            if f & 0x4_0000 != 0 {
                v &= 0xffff_ffff_fff0_ffff;
            }
            if f & 0x2_0000 != 0 {
                v &= 0xffff_ffff_ffff_7fff;
            }
            if f & 0x1_0000 != 0 {
                v &= 0xffff_ffff_ffff_bfff;
            }
            ee_sub_166e90(m, pk, 0x4d, v << 32 | 0x200_0000);
        } else {
            ee_sub_166e90(m, pk, 0x4d, 0xffff_c000_0200_0000);
        }
    } else {
        ee_sub_166e90(m, pk, 0x4d, 0x3fff_0200_0000);
    }
    jal_16d698(m, if f & 1 == 0 { 7 } else { 6 }, pk, out);
    ee_sub_166e90(m, pk, 7, 0xe_a820_0000);
    ee_sub_166e90(m, pk, 0x15, 1);
    ee_sub_166e90(m, pk, 0x3f, 0);
    ee_sub_166e90(m, pk, 0x45, 0);
    ee_sub_166e90(m, pk, 0x19, 0);
    ee_sub_166e90(m, pk, 0x41, 0x1c0_0000_0200_0000);
    ee_sub_166e90(m, pk, 0x48, 0x31001);
    ee_sub_166e90(m, pk, 0x43, 0x2a);
    ee_sub_166e90(m, pk, 0x3b, 0x80_0000_0000);
    ee_sub_166e90(m, pk, 1, 0x1_8080_8080);
    ee_sub_166e90(m, pk, 0, 0x356);
    ee_sub_166ef0(m, pk, 4, 0x5353);
    let mut i1: i32 = 8;
    let mut u4: u32 = 8;
    if f & 0x40 != 0 {
        u4 = 0;
    }
    if f & 0x10 != 0 {
        i1 = 0;
    }
    if u4 < 0x200 {
        let mut i6 = (u4 * 0x10 + 0x80) as i32;
        let mut i5 = i1 * 0x10 + 0x88;
        let mut i1 = i1 * 0x10 + 8;
        loop {
            ee_sub_166fe8(m, pk, i1 as u64, 8);
            ee_sub_167048(m, pk, (u4 << 4) as u64, 0, 0);
            u4 += 0x10;
            ee_sub_166fe8(m, pk, i5 as u64, 0x1c08);
            ee_sub_167048(m, pk, i6 as u64, 0x1c00, 0);
            i6 += 0x100;
            i5 += 0x100;
            i1 += 0x100;
            if (u4 as i32) >= 0x200 {
                break;
            }
        }
    }
    ee_sub_166bd8(m, pk, 0x200, 1, 0xe);
    ee_sub_166e90(m, pk, 0x41, 0xe_0000_0200_0000);
}
/// FUN_0016dd58(p, _, mode): full-screen clear / copy packet (mode 8 = only the closing part)
pub fn ee_sub_16dd58<M: QMem + ?Sized>(m: &mut M, p: u32, _b: u32, mode: u32) {
    m.call(0x0015_74b8, &[p], &[]);
    let sp = stack_push(m, 0x2050);
    let (pk, buf, a0, out) = (sp, sp + 0x30, sp + 0x2030, sp + 0x2034);
    m.call(0x0022_6e88, &[p, 5, 8, 0x1b, 0, a0], &[]);
    let t = m.r32(a0);
    m.w32(out, t);
    ee_sub_166920(m, pk);
    m.call(0x0015_7330, &[pk, 0x200, buf], &[]);
    if mode != 8 {
        if m.r32(0x002d_eb74) != 0 {
            ee_sub_166bd8(m, pk, 0, 1, 0xe);
            let (mut i4, mut i3) = (0x7200u32, 0x7000u32);
            jal_16d698(m, 0, pk, out);
            let mut i2 = 0x1e0i32;
            ee_sub_166e90(m, pk, 0x4c, 0xffff_feff_0000_0000);
            ee_sub_166e90(m, pk, 0x47, 0x31001);
            ee_sub_166e90(m, pk, 0x42, 0x2_0000_00a9);
            ee_sub_166e90(m, pk, 1, 0x1_0000_0000);
            ee_sub_166e90(m, pk, 0, 0x46);
            ee_sub_166ef0(m, pk, 2, 0x55);
            loop {
                ee_sub_167048(m, pk, i3 as u64, 0x7900, 0);
                i3 += 0x200;
                ee_sub_167048(m, pk, i4 as u64, 0x8700, 0);
                i2 -= 0x20;
                i4 += 0x200;
                if i2 < 0 {
                    break;
                }
            }
        }
        jal_16d6c8(m, pk, 0x81_0095, out);
        jal_16d6c8(m, pk, 0x266, out);
    }
    ee_sub_166bd8(m, pk, 0, 1, 0xe);
    jal_16d698(m, 0, pk, out);
    ee_sub_166e90(m, pk, 0x4c, 0);
    ee_sub_166e90(m, pk, 0x46, 1);
    if mode != 8 {
        jal_16d698(m, 2, pk, out);
        ee_sub_166e90(m, pk, 6, 0x6_a800_0000);
        ee_sub_166e90(m, pk, 0x14, 1);
        let g = m.r32(0x003e_1330);
        ee_sub_166e90(m, pk, 0x16, g as u64);
        ee_sub_166e90(m, pk, 0x3f, 0);
        ee_sub_166e90(m, pk, 0x47, 0x31001);
        ee_sub_166e90(m, pk, 1, 0x1_8080_8080);
        ee_sub_166e90(m, pk, 0x42, 0x44);
        ee_sub_166e90(m, pk, 0, 0x156);
        ee_sub_166ef0(m, pk, 4, 0x5353);
        let (mut u1, mut i2) = (0u32, 0i32);
        loop {
            ee_sub_166fe8(m, pk, (u1 | 8) as u64, 8);
            let i3 = i2 + 0x20;
            ee_sub_167048(m, pk, ((i2 + 0x700) * 0x10) as u64, 0x7900, 0);
            ee_sub_166fe8(m, pk, ((i3 * 0x10) | 8) as u64, 0xe08);
            ee_sub_167048(m, pk, ((i2 + 0x720) * 0x10) as u64, 0x8700, 0);
            u1 = (i3 * 0x10) as u32;
            i2 = i3;
            if i3 >= 0x200 {
                break;
            }
        }
    }
    ee_sub_1669e0(m, pk, 1);
    jal_16d698(m, 0xffff, pk, out);
    let (s, e) = (m.r32(pk + 4), m.r32(pk + 0xc));
    if e.wrapping_sub(s) >> 4 < 0x100 {
        m.call(0x0022_6f28, &[p, 0xc, e.wrapping_sub(s) >> 4, s, 0x23], &[]);
    } else {
        m.call(0x0022_6f28, &[p, 0xc, 0xff, s, 0x23], &[]);
        m.call(0x0022_6f28, &[p, 0xc, (e.wrapping_sub(s) >> 4) - 0xff, s + 0xff0, 0x122], &[]);
    }
    m.call(0x0022_6d28, &[p], &[]);
    m.call(0x0022_6ce0, &[p, 0x1c48], &[]);
    m.call(0x0022_6d28, &[p], &[]);
    if m.r32(0x003e_12c8) != 0 {
        let a = if m.r32(0x002d_ea4c) == 0x002d_ea50 { 0x002d_f940 } else { 0x002d_fa80 };
        m.call(0x0015_7598, &[p, a], &[]);
        m.call(0x0022_6ca8, &[p, 0x32, 0x1e7], &[]);
    }
    m.call(0x0015_76f0, &[p], &[]);
    if mode != 8 {
        m.w32(0x002d_eb70, 0);
        m.w32(0x002d_eb74, 0);
    }
    stack_pop(m, 0x2050);
}
/// FUN_0016e1a8
pub fn ee_sub_16e1a8<M: QMem + ?Sized>(m: &mut M, p: u32) {
    m.call(0x0016_0918, &[p, 0x0025_e680, 0x0026_2320], &[]);
    m.call(0x0015_74b8, &[p], &[]);
    let sp = stack_push(m, 0x30);
    m.call(0x0022_6e88, &[p, 0xd, 0xb, 10, 0, sp], &[]);
    let t = m.r32(sp);
    m.call(0x0023_33b4, &[t, 0, 0x58], &[]);
    m.call(0x0022_6ce0, &[p, 0], &[]);
    m.call(0x0015_76f0, &[p], &[]);
    stack_pop(m, 0x30);
}
/// FUN_0016e228
pub fn ee_sub_16e228<M: QMem + ?Sized>(m: &mut M, p: u32) {
    m.call(0x0015_74b8, &[p], &[]);
    let sp = stack_push(m, 0x30);
    m.call(0x0022_6e88, &[p, 0xd, 0xb, 10, 0, sp], &[]);
    let t = m.r32(sp);
    m.call(0x0023_33b4, &[t, 0, 0x58], &[]);
    m.call(0x0022_6ce0, &[p, 0x3ab0], &[]);
    m.call(0x0022_6d28, &[p], &[]);
    m.call(0x0015_76f0, &[p], &[]);
    stack_pop(m, 0x30);
}
/// FUN_0016e298
pub fn ee_sub_16e298<M: QMem + ?Sized>(m: &mut M, p: u32) {
    ee_sub_16e1a8(m, p);
    let n = m.r32(0x003d_cfa0) as i32;
    let mut i = 0;
    while i < n {
        let v = m.r32(0x003d_cfa8 + 4 * i as u32);
        m.call(0x0015_7598, &[p, v], &[]);
        i += 1;
        if m.r32(0x003d_cfa0) as i32 <= i {
            break;
        }
    }
}
/// FUN_0016e318
pub fn ee_sub_16e318<M: QMem + ?Sized>(m: &mut M, p: u32, a: u32, b: u32) {
    m.call(0x0015_74b8, &[p], &[]);
    m.call(0x0022_6d10, &[p], &[]);
    m.call(0x0022_70c8, &[p, a, b, 0], &[]);
    m.call(0x0022_6d10, &[p], &[]);
    m.call(0x0015_76f0, &[p], &[]);
}
/// FUN_0016e380
pub fn ee_sub_16e380<M: QMem + ?Sized>(m: &mut M, p: u32) {
    ee_sub_16e228(m, p);
    if 0 < m.r32(0x003d_cfa0) as i32 {
        ee_sub_16e318(m, p, 0x25, 0x002d_f010);
    }
}
/// FUN_0016e3c8
pub fn ee_sub_16e3c8<M: QMem + ?Sized>(m: &mut M, p: u32) {
    ee_sub_16e1a8(m, p);
    ee_sub_16e318(m, p, 2, 0x002d_f700);
}
/// FUN_0016e400
pub fn ee_sub_16e400<M: QMem + ?Sized>(m: &mut M, p: u32) {
    ee_sub_16e228(m, p);
    ee_sub_16e318(m, p, 2, 0x002d_f720);
}

// ---------------------------------------------------------------- run_0016ef00
/// FUN_0016ef00(pp, idx): sets the +0x148 bit 35 off on *pp, stores the constant quad 0x2a2dd0 (w = 1e6 as
/// 0x49742400) at pp+0x10 and the normalised-by-FUN_002599d8/e0/e8 components at pp[0x11] + idx * 0x18
pub fn ee_sub_16ef00<M: QMem + ?Sized>(m: &mut M, pp: u32, idx: u32) {
    let o = m.r32(pp);
    m.call(0x001d_6310, &[o], &[]);
    let v = m.r64(o + 0x148) & 0xffff_fff7_ffff_ffff;
    m.w32(o + 0x148, v as u32);
    m.w32(o + 0x14c, (v >> 32) as u32);
    let q = m.rv(0x002a_2dd0);
    m.w32(pp + 0x20, 0x4974_2400);
    m.wv(pp + 0x10, q);
    let p = m.r32(pp + 0x44).wrapping_add(idx.wrapping_mul(0x18));
    let sp = stack_push(m, 0x10);
    m.wv(sp, q);
    let a = m.call(0x0025_99d8, &[sp], &[]).0;
    let b = m.call(0x0025_99e0, &[sp], &[]).0;
    let c = m.call(0x0025_99e8, &[sp], &[]).0;
    m.w32(p + 0xc, 0x4974_2400);
    m.w32(p, a);
    m.w32(p + 4, b);
    m.w32(p + 8, c);
    stack_pop(m, 0x10);
}
/// FUN_0016efe0: same pool init after DAT_002deb50 = 0x80101010
pub fn ee_sub_16efe0<M: QMem + ?Sized>(m: &mut M, p1: u32, p2: u32) {
    if p2 == 0xffff && p1 != 0 {
        m.w32(0x002d_eb50, 0x8010_1010);
        pool_static_init(m, p1, p2);
    }
}
pub fn ee_sub_16f128<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_16efe0(m, 1, 0xffff);
}
