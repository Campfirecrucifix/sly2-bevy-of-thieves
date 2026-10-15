//! WP-Q8: C2ce118 (holds up to 8 child refs at +0xe4, count +0xe0) slot overrides.
use crate::engine::classes::q::q2mem::QMem;
use crate::engine::classes::q::q8util::*;

/// FUN_00156cc8 (slot 6): parent init FUN_00231100, FUN_001a6ca8(this, 0), +0x40 flags |= 2
pub fn ee_sub_156cc8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0023_1100, &[o], &[]);
    m.call(0x001a_6ca8, &[o, 0], &[]);
    set_tagged(m, o + 0x40, 2);
}
/// FUN_00156d20: parent FUN_001a5800(this), FUN_0011a220(0x3e1468, this)
pub fn ee_sub_156d20<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001a_5800, &[o], &[]);
    m.call(0x0011_a220, &[0x003e_1468, o], &[]);
}
/// FUN_00156d58: parent FUN_001a5848(this), FUN_00156de0(this, 1), FUN_0011a2e8(0x3e1468, this)
pub fn ee_sub_156d58<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001a_5848, &[o], &[]);
    ee_sub_156de0(m, o, 1);
    m.call(0x0011_a2e8, &[0x003e_1468, o], &[]);
}
/// FUN_00156d98: parent FUN_002311b0(this) preserving the u64 at +0x104
pub fn ee_sub_156d98<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let v = r64(m, o + 0x104);
    m.call(0x0023_11b0, &[o], &[]);
    w64(m, o + 0x104, v);
}
/// FUN_00156de0: release the child refs (all if force, else those FUN_00156870 rejects) and compact
pub fn ee_sub_156de0<M: QMem + ?Sized>(m: &mut M, o: u32, force: u64) {
    let mut i: i32 = 0;
    if 0 < m.r32(o + 0xe0) as i32 {
        let mut p5 = o + 0xe8;
        let mut p3 = o + 0xe4;
        loop {
            let c = m.r32(p3);
            if force == 0 && m.call(0x0015_6870, &[c], &[]).0 == 0 {
                p5 += 4;
                p3 += 4;
                i += 1;
            } else {
                vcall(m, c, 0xc, 0x14, &[], &[]);
                let n = m.r32(o + 0xe0) as i32;
                m.call(0x0023_32b8, &[p3, p5, ((n - i) * 4 - 4) as u32], &[]);
                m.w32(o + 0xe0, (n - 1) as u32);
            }
            if i >= m.r32(o + 0xe0) as i32 {
                break;
            }
        }
    }
}
/// FUN_00156eb8: walk the list at DAT_003e148c (next +0x58); add up to 8 unseen entries accepted by FUN_00156fe8
pub fn ee_sub_156eb8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001a_5e28, &[o], &[]);
    ee_sub_156de0(m, o, 0);
    let mut c = m.r32(0x003e_148c);
    if c == 0 {
        return;
    }
    loop {
        let cnt = m.r32(o + 0xe0) as i32;
        let mut known = false;
        if cnt >= 1 {
            let mut i = 0;
            while i < cnt {
                if c == m.r32(o + 0xe4 + 4 * i as u32) {
                    known = true;
                    break;
                }
                i += 1;
            }
        }
        if !known {
            if m.call(0x0015_6fe8, &[o, c], &[]).0 != 0 {
                if 7 < m.r32(o + 0xe0) {
                    return;
                }
                vcall(m, c, 0xc, 0xc, &[], &[]);
                let n = m.r32(o + 0xe0);
                m.w32(o + 0xe4 + n * 4, c);
                m.w32(o + 0xe0, n + 1);
                vcall(m, o, 0xc, 0x1c, &[5, 0, 0, 0, 0], &[]);
                return;
            }
        }
        c = m.r32(c + 0x58);
        if c == 0 {
            return;
        }
    }
}
