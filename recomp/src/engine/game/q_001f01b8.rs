//! WIRE-A: the SM / SMA link on `QMem` (the struct runtime is anim/sm.rs + classes k17): FUN_001f1448
//! (link an SMA to an object), FUN_001f0380 / FUN_001f01b8 (create the 0x50-byte SMA), FUN_001f0400 /
//! FUN_001f04f8 (label <-> state index of an SM: +0x50 count, +0x54 {label, next} pairs). 1:1 from the
//! decomp; the SMA transition runner (FUN_001f0d88, FUN_001f13a8) and the vtable calls go through
//! `m.call` / `vslot` and degrade where unported.
use crate::engine::classes::q::q2mem::{vslot, QMem};

/// FUN_001f0400(sm, label): state index of `label` (-1 none)
pub fn ee_sub_1f0400<M: QMem + ?Sized>(m: &mut M, sm: u32, label: u32) -> u32 {
    let n = m.r32(sm + 0x50) as i32;
    let p = m.r32(sm + 0x54);
    for i in 0..n.max(0) as u32 {
        if m.r32(p + i * 8) == label {
            return i;
        }
    }
    u32::MAX
}

/// FUN_001f04f8(sm, i): label of state `i` (-1 out of range)
pub fn ee_sub_1f04f8<M: QMem + ?Sized>(m: &mut M, sm: u32, i: u32) -> u32 {
    let i = i as i32;
    if -1 < i && i < m.r32(sm + 0x50) as i32 {
        let p = m.r32(sm + 0x54);
        return m.r32(p + i as u32 * 8);
    }
    u32::MAX
}

/// FUN_001f01b8(sm, obj, label, p4, p5) -> SMA (the original returns it through param_6)
pub fn ee_sub_1f01b8<M: QMem + ?Sized>(m: &mut M, sm: u32, obj: u32, label: u32, p4: u32, p5: u32) -> u32 {
    let s = m.call(0x0011_ccc0, &[0x003e_1278], &[]).0;
    for k in 0..0x14 {
        m.w32(s + k * 4, 0);
    }
    m.w32(s + 0x28, obj);
    m.w32(s + 0x2c, p4);
    m.w32(s + 0x48, p5);
    m.w32(s + 0x24, sm);
    m.w32(s + 0x38, u32::MAX);
    m.w32(s + 0x44, 0x3f80_0000);
    m.w32(s + 0xc, 0x002d_9768);
    m.call(0x0011_a220, &[0x003e_126c, s], &[]);
    m.call(0x0011_a220, &[sm + 0x68, s], &[]);
    vslot(m, s, 0, &[], &[]);
    vslot(m, sm, 0, &[], &[]);
    let o = m.r32(sm);
    m.call(0x0025_6ee0, &[s, o], &[]);
    vslot(m, s, 7, &[2, 0, 0, 0, 0], &[]);
    if m.call(0x001f_0638, &[s], &[]).0 == 0 {
        if label != u32::MAX {
            let i = ee_sub_1f0400(m, sm, label);
            m.w32(s + 0x34, i);
            m.w32(s + 0x3c, i);
            m.call(0x001f_13a8, &[s, u32::MAX, i], &[]);
        }
        m.call(0x001f_0d88, &[s, 0], &[]);
    }
    vslot(m, sm, 1, &[], &[]);
    s
}

/// FUN_001f0380(sm, obj, label, p4): FUN_001f01b8 with p5 = 0 (the decompiler drops the register args)
pub fn ee_sub_1f0380<M: QMem + ?Sized>(m: &mut M, sm: u32, obj: u32, label: u32, p4: u32) -> u32 {
    ee_sub_1f01b8(m, sm, obj, label, p4, 0)
}

/// FUN_001f1448(sm, obj, label, p4, slot): release the SMA at *slot (FUN_001f0598, vt +0x14), then link a new one
pub fn ee_sub_1f1448<M: QMem + ?Sized>(m: &mut M, sm: u32, obj: u32, label: u32, p4: u32, slot: u32) {
    let old = m.r32(slot);
    if old != 0 {
        m.call(0x001f_0598, &[old], &[]);
        vslot(m, old, 1, &[], &[]);
        m.w32(slot, 0);
    }
    if sm != 0 {
        let s = ee_sub_1f0380(m, sm, obj, label, p4);
        m.w32(slot, s);
        vslot(m, s, 0, &[], &[]);
    }
}
