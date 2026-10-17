//! WP-Q9: trivial slot overrides of many small classes (parent call + a few field writes), 1:1 from ghidra_functions.body.
//! Calls to other functions go through `m.call` by address (emulator or an existing override).
#![allow(unused_variables, unused_mut, dead_code)]
use crate::engine::classes::q::q2mem::QMem;
use crate::engine::classes::q::q9util::*;

/// slot 6 (c2cdd40, FUN_001559b0)
pub fn ee_sub_1559b0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    crate::engine::classes::q::c2cdd40::ee_sub_1559b0(m, o);
}

/// slot 8 (c2cdd40, FUN_001559f8)
pub fn ee_sub_1559f8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    crate::engine::classes::q::c2cdd40::ee_sub_1559f8(m, o);
}

/// slot 7 (c2cdd40, FUN_00155a38)
pub fn ee_sub_155a38<M: QMem + ?Sized>(m: &mut M, o: u32) {
    crate::engine::classes::q::c2cdd40::ee_sub_155a38(m, o);
}

/// slot 6 (c2cfc20, FUN_0017e888)
pub fn ee_sub_17e888<M: QMem + ?Sized>(m: &mut M, o: u32) {
    crate::engine::classes::q::c2cfc20::ee_sub_17e888(m, o);
}

/// slot 8 (c2cfc20, FUN_0017e8e8)
pub fn ee_sub_17e8e8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    crate::engine::classes::q::c2cfc20::ee_sub_17e8e8(m, o);
}

/// slot 6 (c2cfd20, FUN_0017dd70)
pub fn ee_sub_17dd70<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef68, &[o], &[]);
    m.w32(o + 0x30, u32::MAX);
    let ow = m.r32(o + 0x10);
    m.call(0x0020_9890, &[ow, 0x2b], &[]);
}

/// slot 7 (c2cfd20, FUN_0017dda8)
pub fn ee_sub_17dda8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef70, &[o], &[]);
    m.call(0x0017_e278, &[o, 0], &[]);
    let ow = m.r32(o + 0x10);
    let a = m.r32(o + 0x38);
    m.call(0x0020_8540, &[ow, a], &[]);
}

/// slot 6 (c2d0020, FUN_0017cac8)
pub fn ee_sub_17cac8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef68, &[o], &[]);
    m.w32(o + 0x30, u32::MAX);
    m.w32(o + 0x38, 0x3f80_0000);
    let ow = m.r32(o + 0x10);
    m.call(0x0020_9890, &[ow, 0x1b], &[]);
}

/// slot 7 (c2d0020, FUN_0017cb10)
pub fn ee_sub_17cb10<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef70, &[o], &[]);
    m.call(0x0017_cd00, &[o, 0], &[]);
}

/// slot 8 (c2d0020, FUN_0017cb40)
pub fn ee_sub_17cb40<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef78, &[o], &[]);
    m.call(0x0017_cd00, &[o, u32::MAX], &[]);
    let ow = m.r32(o + 0x10);
    m.call(0x0020_9970, &[ow, 0x1b], &[]);
}

/// slot 6 (c2d00a0, FUN_0017c638)
pub fn ee_sub_17c638<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef68, &[o], &[]);
    m.w32(o + 0x50, u32::MAX);
    let ow = m.r32(o + 0x10);
    m.call(0x0020_9890, &[ow, 0x26], &[]);
}

/// slot 8 (c2d00a0, FUN_0017c670)
pub fn ee_sub_17c670<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef78, &[o], &[]);
    let ow = m.r32(o + 0x10);
    m.call(0x0020_9970, &[ow, 0x26], &[]);
}

/// slot 7 (c2d00a0, FUN_0017c6a0)
pub fn ee_sub_17c6a0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef70, &[o], &[]);
    m.call(0x0017_c928, &[o, 0], &[]);
}

/// slot 6 (c2d0220, FUN_0017b590)
pub fn ee_sub_17b590<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef68, &[o], &[]);
    m.w32(o + 0x30, u32::MAX);
}

/// slot 7 (c2d0220, FUN_0017b5c0)
pub fn ee_sub_17b5c0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef70, &[o], &[]);
    m.call(0x0017_b750, &[o, 0], &[]);
}

/// slot 8 (c2d0220, FUN_0017b5f0)
pub fn ee_sub_17b5f0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef78, &[o], &[]);
    m.call(0x0017_b750, &[o, u32::MAX], &[]);
}

/// slot 6 (c2d02a0, FUN_0017a4b0)
pub fn ee_sub_17a4b0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0017_4920, &[o], &[]);
    m.w32(o + 0x50, u32::MAX);
    m.w32(o + 0x3c, 0x10);
}

/// slot 6 (c2d03a0, FUN_0017a9d0)
pub fn ee_sub_17a9d0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0017_4920, &[o], &[]);
    m.w32(o + 0x50, u32::MAX);
    m.w32(o + 0x3c, 0x17);
}

/// slot 8 (c2d03a0, FUN_0017b178)
pub fn ee_sub_17b178<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0017_b1a8, &[o, u32::MAX], &[]);
    m.call(0x0017_4b28, &[o], &[]);
}

/// state map (c2d03a0, FUN_0017b578)
pub fn ee_sub_17b578<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    if m.r32(o + 0x50) == 3 { 7 } else { 1 }
}

/// slot 6 (c2d04a0, FUN_001766d8)
pub fn ee_sub_1766d8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef68, &[o], &[]);
    m.w32(o + 0x30, u32::MAX);
}

/// slot 6 (c2d0830, FUN_00179750)
pub fn ee_sub_179750<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0017_4920, &[o], &[]);
    m.w32(o + 0x3c, 6);
}

/// slot 14 (c2d0830, FUN_00179820)
pub fn ee_sub_179820<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0017_9598, &[o], &[]);
    let ow = m.r32(o + 0x10);
    let a = m.r32(ow + 0xfec);
    if a != 0 {
        vcall(m, a, 0xc, 0x4c, &[], &[]);
    }
}

/// slot 6 (c2d4af8, FUN_001ba560)
pub fn ee_sub_1ba560<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef68, &[o], &[]);
    m.w32(o + 0x30, u32::MAX);
    let ow = m.r32(o + 0x10);
    m.call(0x0020_9890, &[ow, 0x25], &[]);
}

/// slot 7 (c2d4af8, FUN_001ba598)
pub fn ee_sub_1ba598<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef70, &[o], &[]);
    m.call(0x001b_a828, &[o, 0], &[]);
}

/// slot 8 (c2d4af8, FUN_001ba5c8)
pub fn ee_sub_1ba5c8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001b_a828, &[o, u32::MAX], &[]);
    m.call(0x001e_ef78, &[o], &[]);
    let ow = m.r32(o + 0x10);
    m.call(0x0020_9970, &[ow, 0x25], &[]);
}

/// create (c2d36d0, FUN_001da770)
pub fn ee_sub_1da770<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let k = m.r32(o + 0x20);
    let r = m.call(0x0015_e938, &[WORLD, 5, k, 0], &[]).0;
    m.w32(o + 0x24, r);
}

/// destroy (c2d36d0, FUN_001da7b0)
pub fn ee_sub_1da7b0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.w32(o + 0x24, 0);
    m.call(0x001d_a7d0, &[o], &[]);
}

/// slot (c2d37d0, FUN_001ae9a8)
pub fn ee_sub_1ae9a8<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32) {
    m.call(0x001d_98a8, &[o], &[]);
    if p2 != 0 {
        let a = m.r32(o + 0x10);
        m.call(0x0018_7e78, &[a], &[]);
    }
}

/// init (c2d7558, FUN_001d0ca8) - legacy 1-arg form kept until WIRE-A regenerates live_dispatch
/// (REQUESTS.md); passes zero for the pass-through registers. Use ee_sub_1d0ca8_pt.
pub fn ee_sub_1d0ca8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    ee_sub_1d0ca8_pt(m, o, [0.0; 4], 0, 0.0)
}
/// init (c2d7558, FUN_001d0ca8), 1:1
/// VFIX: a1 (128-bit), a2 and f12 pass through untouched to FUN_001ce920 (asm: jal with only a0 set).
pub fn ee_sub_1d0ca8_pt<M: QMem + ?Sized>(m: &mut M, o: u32, a1: crate::engine::classes::q::q2mem::V4, a2: u32, f12: f32) {
    m.call(0x001c_e920, &[o, a2], &[a1, [f12, 0.0, 0.0, 0.0]]);
    m.w32(o + 0x18, 0x40a0_0000);
    for i in 0..4 {
        let w = ro32(0x002c_1f40 + 4 * i);
        m.w32(o + 0xa0 + 4 * i, w);
    }
}

/// init2 (c2d7558, FUN_001d0ce8)
pub fn ee_sub_1d0ce8<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32) {
    m.call(0x001c_e9b8, &[o, p2], &[]);
    let t = m.r32(p2 + 8);
    let w = m.r32(t + 0xc);
    m.w32(o + 0xf4, w);
}

/// init2 (c2d7738, FUN_001cf5d0)
pub fn ee_sub_1cf5d0<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32) {
    m.call(0x001c_e9b8, &[o, p2], &[]);
    let t = m.r32(p2 + 8);
    let w = m.r32(t + 0xc);
    m.w32(o + 0xf0, w);
    let w = m.r32(p2 + 0xf0);
    m.w32(o + 0xf4, w);
}

/// update (c2d7648, FUN_001d00d0)
pub fn ee_sub_1d00d0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.rf(o + 0x14) <= m.rf(CLOCK) {
        m.call(0x001c_f518, &[o], &[]);
    }
}

/// pred (c2d77d8, FUN_001ce6c0)
pub fn ee_sub_1ce6c0<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    (m.r32(o + 0x150) != 1) as u32
}

/// slot (c2d7a40, FUN_001d4ec8)
pub fn ee_sub_1d4ec8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001f_36a8, &[o], &[]);
    let a = m.r32(o + 0x160);
    m.call(0x001f_1448, &[a, o, u32::MAX, 0, o + 0x164], &[]);
    m.call(0x001d_4fa0, &[o, 0], &[]);
}

/// slot (c2d7a40, FUN_001d4f10)
pub fn ee_sub_1d4f10<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001c_b518, &[o], &[]);
    m.call(0x001d_4fa0, &[o, u32::MAX], &[]);
    m.call(0x001d_4fa0, &[o, 0], &[]);
}

/// init (c2d8468, FUN_001dcca0)
pub fn ee_sub_1dcca0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001d_a770, &[o], &[]);
    let t = m.r32(o + 0x24);
    if t != 0 {
        m.w32(t + 0xb0, 1);
    }
}

/// destroy (c2d8468, FUN_001dccd8)
pub fn ee_sub_1dccd8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001d_a7b0, &[o], &[]);
    m.call(0x001d_cd08, &[o], &[]);
}

/// inc (c2d8c08, FUN_001e1b58)
pub fn ee_sub_1e1b58<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let v = m.r32(o + 0x10).wrapping_add(1);
    m.w32(o + 0x10, v);
}

/// pred (c2d91e0, FUN_001edb80)
pub fn ee_sub_1edb80<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32) -> u32 {
    (m.r32(p2) == m.r32(o + 0x14)) as u32
}

/// init2 (c2d9300, FUN_001edfa0)
pub fn ee_sub_1edfa0<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32) {
    m.call(0x001e_d8e0, &[o, p2], &[]);
    let w = m.r32(p2 + 0xc);
    let v = m.r32(p2 + 0x10);
    m.w32(o + 0x54, v);
    m.w32(o + 0x50, w);
}

/// inc16 (c2d9c70, FUN_001fc090)
pub fn ee_sub_1fc090<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let v = m.r16(o + 0x4c).wrapping_add(1);
    m.w16(o + 0x4c, v);
}

/// slot (c2d9c70, FUN_001fc110)
pub fn ee_sub_1fc110<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0011_cd98, &[0x003d_d05c, o], &[]);
}

/// slot (c2ca760, FUN_0012d858)
pub fn ee_sub_12d858<M: QMem + ?Sized>(m: &mut M, o: u32) {
    crate::engine::classes::q::c2ca760::ee_sub_12d858(m, o);
}

/// slot (c2ca760, FUN_0012d890)
pub fn ee_sub_12d890<M: QMem + ?Sized>(m: &mut M, o: u32) {
    crate::engine::classes::q::c2ca760::ee_sub_12d890(m, o);
}
