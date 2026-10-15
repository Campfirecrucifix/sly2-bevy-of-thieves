//! WP-Q9: C2ca760 (path follower: current sub-object +0x1c, wanted +0x10) slot overrides.
use crate::engine::classes::q::q2mem::QMem;
use crate::engine::classes::q::q9util::*;

/// FUN_0012d858 (v0): +0x1c = +0x10; if set, virtual (sub +0x18 vtable, slot fn +0xc)
pub fn ee_sub_12d858<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let i = m.r32(o + 0x10);
    m.w32(o + 0x1c, i);
    if i != 0 {
        vcall(m, i, 0x18, 0xc, &[], &[]);
    }
}
/// FUN_0012d890 (v1): if +0x1c set: virtual (+0x14) and clear it
pub fn ee_sub_12d890<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let i = m.r32(o + 0x1c);
    if i != 0 {
        vcall(m, i, 0x18, 0x14, &[], &[]);
        m.w32(o + 0x1c, 0);
    }
}
/// FUN_0012d8d8 (v2): if +0x1c != +0x10: FUN_0022c840(+4, +8); leave the old one (+0x14) and enter the new (+0xc)
pub fn ee_sub_12d8d8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.r32(o + 0x1c) != m.r32(o + 0x10) {
        let (a, b) = (m.r32(o + 4), m.r32(o + 8));
        m.call(0x0022_c840, &[a, b], &[]);
        let mut i = m.r32(o + 0x1c);
        if i == 0 {
            i = m.r32(o + 0x10);
        } else {
            vcall(m, i, 0x18, 0x14, &[], &[]);
            i = m.r32(o + 0x10);
        }
        m.w32(o + 0x1c, i);
        if i != 0 {
            vcall(m, i, 0x18, 0xc, &[], &[]);
        }
    }
}
/// FUN_0012d958 (v3): no sub-object -> C2ca6a0_v3 (FUN_0012cf28); else virtual (+0x24), then data(+8)+0xc0 and +0xe0 += quad +0x20
pub fn ee_sub_12d958<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let i = m.r32(o + 0x1c);
    if i == 0 {
        m.call(0x0012_cf28, &[o], &[]);
    } else {
        vcall(m, i, 0x18, 0x24, &[], &[]);
        let d = m.r32(o + 8);
        vadd_mem(m, d + 0xc0, d + 0xc0, o + 0x20);
        let d = m.r32(o + 8);
        vadd_mem(m, d + 0xe0, d + 0xe0, o + 0x20);
    }
}
/// FUN_0012d9d0 (v4)(p2): virtual (+0x2c) of the sub-object (+0x1c); p2 != 0 -> *(v4*)p2 += quad +0x20
pub fn ee_sub_12d9d0<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32) {
    let i = m.r32(o + 0x1c);
    vcall(m, i, 0x18, 0x2c, &[], &[]);
    if p2 != 0 {
        vadd_mem(m, p2, p2, o + 0x20);
    }
}
