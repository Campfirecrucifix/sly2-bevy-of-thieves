//! WP-Q8: C2d2558 and C2d5898 slot overrides.
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem};
use crate::engine::classes::q::q8util::*;

// ---------------------------------------------------------------- C2d2558
/// FUN_00183340 (slot 6): parent init FUN_00231100; FUN_00182c70(this + 0xe0)
pub fn ee_sub_183340<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0023_1100, &[o], &[]);
    m.call(0x0018_2c70, &[o + 0xe0], &[]);
}
/// FUN_00183370: parent FUN_001a6178; FUN_0011a220(0x3e13fc, this)
pub fn ee_sub_183370<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001a_6178, &[o], &[]);
    m.call(0x0011_a220, &[0x003e_13fc, o], &[]);
}
/// FUN_001833a8: parent FUN_001a6150; FUN_0011a2e8(0x3e13fc, this)
pub fn ee_sub_1833a8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001a_6150, &[o], &[]);
    m.call(0x0011_a2e8, &[0x003e_13fc, o], &[]);
}
/// FUN_001833e0 (post-load): parent FUN_001a5c48; FUN_00221268(+0xe0, 0) when set; +0x38 == 1e10 -> +0x10c + DAT_002be8c0
pub fn ee_sub_1833e0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001a_5c48, &[o], &[]);
    let h = m.r32(o + 0xe0);
    if h != 0 {
        m.call(0x0022_1268, &[h, 0], &[]);
    }
    if m.rf(o + 0x38) == 1e10_f32 {
        let v = m.rf(o + 0x10c) + rof(0x002b_e8c0);
        m.wf(o + 0x38, v);
    }
}
/// FUN_00183450: parent FUN_002311b0 preserving the u64 at +0x104
pub fn ee_sub_183450<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let v = r64(m, o + 0x104);
    m.call(0x0023_11b0, &[o], &[]);
    w64(m, o + 0x104, v);
}

// ---------------------------------------------------------------- C2d5898
/// FUN_001c3dc8 (load): parent FUN_001f31e0(this, stream); FUN_00118420(stream)
pub fn ee_sub_1c3dc8<M: QMem + ?Sized>(m: &mut M, o: u32, stream: u32) {
    m.call(0x001f_31e0, &[o, stream], &[]);
    m.call(0x0011_8420, &[stream], &[]);
}
/// FUN_001c3e90: 0 when (DAT_002e1e00 & 1) == 0 and FUN_0020f0b0(world) == 0, else 1
pub fn ee_sub_1c3e90<M: QMem + ?Sized>(m: &mut M) -> u32 {
    if m.r32(0x002e_1e00) & 1 == 0 && m.call(0x0020_f0b0, &[WORLD], &[]).0 == 0 {
        0
    } else {
        1
    }
}
/// FUN_001c4228: post vslot 0x1c(0xc, 0, 2, &{&p2, &p3}, 0)
pub fn ee_sub_1c4228<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32, p3: u32) {
    let b = stack_push(m, 0x20);
    m.w32(b + 0x10, p2);
    m.w32(b + 0x14, p3);
    m.w32(b, b + 0x10);
    m.w32(b + 4, b + 0x14);
    vcall(m, o, 0xc, 0x1c, &[0xc, 0, 2, b, 0], &[]);
    stack_pop(m, 0x20);
}
