//! WIRE-S1: 1:1 QMem queue ports of live-frontier natives (shard1/2).
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

/// FUN_001a5ed8: empty stub (nop).
pub fn ee_sub_1a5ed8<M: QMem + ?Sized>(_m: &mut M) {}

/// FUN_00113b40: returns the constant table/descriptor address 0x2bac64.
pub fn ee_sub_113b40<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_ac64
}

/// FUN_00113c20: returns the constant table/descriptor address 0x2bad7c.
pub fn ee_sub_113c20<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_ad7c
}

/// FUN_00113c70: returns the constant table/descriptor address 0x2bade0.
pub fn ee_sub_113c70<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_ade0
}

/// FUN_00113d30: returns the constant table/descriptor address 0x2baed0.
pub fn ee_sub_113d30<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_aed0
}

/// FUN_00114080: returns the constant table/descriptor address 0x2bb2f4.
pub fn ee_sub_114080<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_b2f4
}

/// FUN_001141c0: returns the constant table/descriptor address 0x2bb484.
pub fn ee_sub_1141c0<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_b484
}

/// FUN_001142d0: returns the constant table/descriptor address 0x2bb5d8.
pub fn ee_sub_1142d0<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_b5d8
}

/// FUN_00114370: returns the constant table/descriptor address 0x2bb6a0.
pub fn ee_sub_114370<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_b6a0
}

/// FUN_00114410: returns the constant table/descriptor address 0x2bb768.
pub fn ee_sub_114410<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_b768
}

/// FUN_00114490: returns the constant table/descriptor address 0x2bb808.
pub fn ee_sub_114490<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_b808
}

/// FUN_001145a0: returns the constant table/descriptor address 0x2bb95c.
pub fn ee_sub_1145a0<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_b95c
}

/// FUN_00114620: returns the constant table/descriptor address 0x2bb9fc.
pub fn ee_sub_114620<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_b9fc
}

/// FUN_001eda20(p): sync p+0x10 to the global frame time (0x2dded0); returns 1 if it changed, else 0.
pub fn ee_sub_1eda20<M: QMem + ?Sized>(m: &mut M, p: u32) -> u32 {
    let f0 = m.rf(p + 0x10);
    let f1 = m.rf(0x002d_ded0);
    if f0 == f1 {
        return 0;
    }
    m.wf(p + 0x10, f1);
    1
}

/// FUN_002321e8(a, b): send snd command 9 (bus volume) with an 8-byte {a, b} payload via FUN_00232838.
pub fn ee_sub_2321e8<M: QMem + ?Sized>(m: &mut M, a: u32, b: u32) {
    let sp = stack_push(m, 0x20);
    m.w32(sp, a);
    m.w32(sp + 4, b);
    m.call(0x0023_2838, &[9, 8, sp, 0, 0], &[]);
    stack_pop(m, 0x20);
}

/// FUN_001bc208(a, b): lexicographic compare of two packed float pairs (64-bit regs a/b; PROT3W
/// fetches the high word): -1 if a<b, 1 if a>b, else 0. Args are the (lo, hi) words of each pair.
pub fn ee_sub_1bc208<M: QMem + ?Sized>(_m: &mut M, a: V4, b: V4) -> i32 {
    // PART-R6: a/b are qwords in a0/a1 (mtc1 = lane x, PROT3W -> lane y); was 4 lane-0 scalars
    let (a0, a1, b0, b1) = (a[0], a[1], b[0], b[1]);
    if a0 < b0 {
        return -1;
    }
    if b0 < a0 {
        return 1;
    }
    if a1 < b1 {
        return -1;
    }
    if b1 < a1 {
        return 1;
    }
    0
}

/// FUN_001bed18(p): release the +0x128 binding: clear it, then for each of +0xe0 children (+0xe4..)
/// call slot 8 (flag bit0 of +0xf4..) and slot 20(1) (flag bit1); then own slot 20(0) and slot 2(3,0,0,0,0).
pub fn ee_sub_1bed18<M: QMem + ?Sized>(m: &mut M, p: u32) {
    let b = m.r32(p + 0x128);
    if b == 0 {
        return;
    }
    m.w32(b + 0x20, 0);
    m.w32(p + 0x128, 0);
    let mut i: i32 = 0;
    if 0 < m.r32(p + 0xe0) as i32 {
        let mut f = p + 0xf4;
        loop {
            let c = m.r32(f - 0x10);
            if m.r32(f) & 1 != 0 {
                vslot(m, c, 8, &[], &[]);
            }
            if m.r32(f) & 2 != 0 {
                vslot(m, c, 20, &[1], &[]);
            }
            i += 1;
            f += 4;
            if i >= m.r32(p + 0xe0) as i32 {
                break;
            }
        }
    }
    vslot(m, p, 20, &[0], &[]);
    vslot(m, p, 2, &[3, 0, 0, 0, 0], &[]);
}

/// FUN_00215730(e, o, a2, pa3, pt0): movie event bind. kind 4: resolve entry +0x10 of table +8->+0x84
/// (FUN_00217ef8) into o+0x58 and start it (FUN_00217ca8, 0x314); kind 6: look up +0x10 and store +0x14.
pub fn ee_sub_215730<M: QMem + ?Sized>(m: &mut M, e: u32, o: u32, a2: u32, a3: u32, t0: u32) {
    m.w32(o + 0x30, e);
    match m.r32(e + 4) {
        4 => {
            let i = m.r32(e + 0x10);
            let t = m.r32(e + 8);
            let t = m.r32(t + 0x84);
            let v = m.call(0x0021_7ef8, &[t.wrapping_add(i.wrapping_mul(0x30))], &[]).0;
            m.w32(o + 0x58, v);
            let sp = stack_push(m, 0x90);
            let w = m.r32(a3);
            m.w32(sp, w);
            let i = m.r32(e + 0x10);
            let t = m.r32(e + 8);
            let t = m.r32(t + 0x84);
            let w = m.r32(t0);
            m.w32(sp + 0x10, w);
            m.call(0x0021_7ca8, &[t.wrapping_add(i.wrapping_mul(0x30)), 0x314, v, a2, sp, sp + 0x10], &[]);
            stack_pop(m, 0x90);
        }
        6 => {
            let i = m.r32(e + 0x10);
            if i != 0xffff_ffff {
                let v = m.call(0x001b_eed8, &[i], &[]).0;
                if v != 0 {
                    let r = m.call(0x001b_eb28, &[v], &[]).0;
                    m.w32(e + 0x14, r);
                }
            }
        }
        _ => {}
    }
}
