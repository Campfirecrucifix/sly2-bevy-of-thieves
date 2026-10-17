//! WP-Q9: C2cfca0 (brain-driven moves along a table: owner +0x10, anim +0x18, table +0x30, index +0x34, state +0x38,
//! since +0x3c, end time +0x40; records of 0x20 bytes at table + 0x60) slot overrides.
//! FUN_0017e6b0 (state switch with the VU0 distance clamp) stays on the emulator path (partial).
use crate::engine::classes::q::q2mem::QMem;
use crate::engine::classes::q::q9util::*;

/// slot 6: parent FUN_001eef68; +0x38 = -1; FUN_00209890(owner, 0x2c)
pub fn ee_sub_17e3a0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef68, &[o], &[]);
    m.w32(o + 0x38, u32::MAX);
    let ow = m.r32(o + 0x10);
    m.call(0x0020_9890, &[ow, 0x2c], &[]);
}
/// slot 7: parent FUN_001eef70; FUN_001eefc0(this); FUN_0017e580(this)
pub fn ee_sub_17e3d8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef70, &[o], &[]);
    m.call(0x001e_efc0, &[o], &[]);
    ee_sub_17e580(m, o);
}
/// slot 8: parent FUN_001eef78; FUN_0017e6b0(this, -1); FUN_001ef038(this); FUN_00209970(owner, 0x2c)
pub fn ee_sub_17e410<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef78, &[o], &[]);
    m.call(0x0017_e6b0, &[o, u32::MAX], &[]);
    m.call(0x001e_f038, &[o], &[]);
    let ow = m.r32(o + 0x10);
    m.call(0x0020_9970, &[ow, 0x2c], &[]);
}
/// slot 9 (update): see the state tests below (+0x38 state 0 / 1, table mode +0x120 of the table)
pub fn ee_sub_17e458<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef98, &[o], &[]);
    if m.call(0x0017_e570, &[o], &[]).0 != 0 {
        return;
    }
    let st = m.r32(o + 0x38);
    if st == 0 {
        let tb = m.r32(o + 0x30);
        let mode = m.r32(tb + 0x120);
        if mode == 0 {
            if m.rf(CLOCK) <= m.rf(o + 0x40) {
                let ow = m.r32(o + 0x10);
                let a = m.r32(ow + 0xe8);
                let b = if a != 0 { m.r32(a + 0xdc) } else { 0 };
                let c = if b != 0 { m.r32(b + 0x10) } else { 0 };
                if c != m.r32(tb + 0x128) {
                    return;
                }
                let an = m.r32(o + 0x18);
                if m.call(0x0017_8d00, &[an], &[]).0 == 0 {
                    return;
                }
                ee_sub_17e580(m, o);
                return;
            }
        } else {
            if mode != 1 {
                return;
            }
            let an = m.r32(o + 0x18);
            if m.call(0x0017_4fb8, &[an], &[]).0 == 0 {
                return;
            }
            if ee_sub_17e5f0(m, o) != 0 {
                ee_sub_17e580(m, o);
                return;
            }
        }
        m.call(0x0017_e6b0, &[o, 1], &[]);
    } else {
        if st != 1 {
            return;
        }
        let an = m.r32(o + 0x18);
        if m.call(0x0017_eae8, &[an], &[]).0 == 0 {
            return;
        }
        ee_sub_17e580(m, o);
    }
}
/// FUN_0017e580: ++index; index < table[0x50] -> state -1 then 0, else state -1 and FUN_001ef038
pub fn ee_sub_17e580<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let i = m.r32(o + 0x34).wrapping_add(1);
    m.w32(o + 0x34, i);
    let tb = m.r32(o + 0x30);
    if (i as i32) < m.r32(tb + 0x50) as i32 {
        m.call(0x0017_e6b0, &[o, u32::MAX], &[]);
        m.call(0x0017_e6b0, &[o, 0], &[]);
    } else {
        m.call(0x0017_e6b0, &[o, u32::MAX], &[]);
        m.call(0x001e_f038, &[o], &[]);
    }
}
/// FUN_0017e5f0: FUN_001c1fa0(25.0, table + index * 0x20 + 0x60, owner) (result used by slot 9)
pub fn ee_sub_17e5f0<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let rec = m.r32(o + 0x30).wrapping_add(m.r32(o + 0x34).wrapping_mul(0x20)).wrapping_add(0x60);
    let ow = m.r32(o + 0x10);
    m.call(0x001c_1fa0, &[rec, ow], &[s(25.0)]).0
}
/// FUN_0017e628: brain 0xac of the owner: +0x68 = 1, +0x50 = FUN_001c1ec0(record, pos(owner)), run (0x44)
pub fn ee_sub_17e628<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let ow = m.r32(o + 0x10);
    let b = m.call(0x0020_5438, &[ow, 0xac], &[]).0;
    m.w32(b + 0x68, 1);
    let rec = m.r32(o + 0x30).wrapping_add(m.r32(o + 0x34).wrapping_mul(0x20)).wrapping_add(0x60);
    let p = m.call(0x001a_62e8, &[ow], &[]).1;
    let e = m.call(0x001c_1ec0, &[rec], &[p]).1;
    m.wv(b + 0x50, e);
    vcall(m, b, 0xc, 0x44, &[], &[]);
}
