//! WIRE-S1: 1:1 QMem queue ports of unported callers (round 2).

use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem, V4};

/// FUN_0021c5f8(tr, v): for each node in the +0xdc list, write the vector (x,y,z of `v` via
/// FUN_00259d98/da8/dc0) into the 0x18-stride entry at +0xc4 indexed by (node.+0 - +0xbc)/0x14.
pub fn ee_sub_21c5f8<M: QMem + ?Sized>(m: &mut M, tr: u32, v: V4) {
    let mut s1 = m.r32(tr + 0xdc);
    if s1 == 0 {
        return;
    }
    let sp = stack_push(m, 16);
    while s1 != 0 {
        let base = m.r32(tr + 0xbc);
        let n = m.r32(s1);
        let tab = m.r32(tr + 0xc4);
        for k in 0..4 {
            m.wf(sp + k * 4, v[k as usize]);
        }
        let i = (n.wrapping_sub(base).wrapping_mul(0xcccc_cccd) as i32) >> 2;
        let s0 = (i as u32).wrapping_mul(0x18).wrapping_add(tab);
        let x = m.call(0x0025_9d98, &[sp], &[]).1[0];
        let y = m.call(0x0025_9da8, &[sp], &[]).1[0];
        let z = m.call(0x0025_9dc0, &[sp], &[]).1[0];
        m.wf(s0 + 8, z);
        m.wf(s0, x);
        m.wf(s0 + 4, y);
        s1 = m.r32(s1 + 0xc);
    }
    stack_pop(m, 16);
}

/// FUN_001c8110(): under lock FUN_001ae478, look up FUN_001c8090 entry, build record via
/// FUN_002581c0(.., 0x3d4a60) and return its +4 field (0 if none).
pub fn ee_sub_1c8110<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let sp = stack_push(m, 0x20);
    m.call(0x001a_e478, &[sp], &[]);
    let e = m.call(0x001c_8090, &[], &[]).0;
    let r = if e != 0 {
        m.call(0x0025_81c0, &[sp + 0x10, e, 0x003d_4a60], &[]);
        let s0 = m.r32(sp + 0x14);
        m.call(0x0012_9ec8, &[sp + 0x10], &[]);
        m.call(0x001a_e4a0, &[sp, 2], &[]);
        s0
    } else {
        m.call(0x001a_e4a0, &[sp, 2], &[]);
        0
    };
    stack_pop(m, 0x20);
    r
}

/// FUN_001c8188(on): under lock, set the FUN_001c8090 entry's flag: if `on` and no existing
/// record, FUN_001abd98(0x2e65e0, 0); then store {0x17, on!=0} via FUN_001291a8/FUN_00258200.
pub fn ee_sub_1c8188<M: QMem + ?Sized>(m: &mut M, on: u32) {
    let sp = stack_push(m, 0x50);
    m.call(0x001a_e478, &[sp], &[]);
    let s1 = m.call(0x001c_8090, &[], &[]).0;
    if s1 == 0 {
        m.call(0x001a_e4a0, &[sp, 2], &[]);
        stack_pop(m, 0x50);
        return;
    }
    if on != 0 {
        m.call(0x0025_81c0, &[sp + 0x10, s1, 0x003d_4a60], &[]);
        let s0 = m.r32(sp + 0x14);
        m.call(0x0012_9ec8, &[sp + 0x10], &[]);
        if s0 == 0 {
            m.call(0x001a_bd98, &[0x002e_65e0, 0], &[]);
        }
    }
    let s0 = sp + 0x20;
    m.w32(sp + 0x24, (on != 0) as u32);
    m.w32(sp + 0x20, 0x17);
    m.call(0x0012_91a8, &[sp + 0x30, s0], &[]);
    m.call(0x0025_8200, &[sp + 0x40, s1, sp + 0x30, 0x003d_4a60], &[]);
    m.call(0x0012_9ec8, &[sp + 0x40], &[]);
    m.call(0x0012_9ec8, &[sp + 0x30], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x001a_e4a0, &[sp, 2], &[]);
    stack_pop(m, 0x50);
}
