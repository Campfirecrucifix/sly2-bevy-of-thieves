//! WP-Q10: class packages C2d07b0, C2d08b0, C2d0930, C2d0a30, C2d0ab0 (family sharing the base slots
//! FUN_00174920 / FUN_00174950 / FUN_00174b28 / FUN_00174bd0) and FUN_00209bc0, FUN_0020c760.
//! Delay-slot effects were checked in the disassembly (e.g. FUN_001751f0 stores +0xf3c before the call).
use crate::engine::classes::q::q2mem::QMem;

const NEG1: u32 = u32::MAX;
const WORLD: u32 = 0x003d_d490;

/// FUN_00209bc0: FUN_00209ad0(o, *(u64 *)(o + 0x2c0)) (low word; the high word is not part of the u32 call ABI)
pub fn ee_sub_209bc0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let a = m.r32(o + 0x2c0);
    m.call(0x0020_9ad0, &[o, a], &[]);
}

/// FUN_00174950 (C2d0ab0_v7)
pub fn c2d0ab0_v7<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef70, &[o], &[]);
    let p = m.r32(o + 0x10);
    let a = m.r32(o + 0x3c);
    m.call(0x0020_9890, &[p, a], &[]);
    if m.r32(o + 0x40) == 0 {
        let v = m.call(0x0020_9bc0, &[p], &[]).0;
        m.w32(o + 0x40, v);
        if v == 0 {
            m.call(0x0017_49c0, &[o, 1], &[]);
            return;
        }
    }
    m.call(0x0017_49c0, &[o, 0], &[]);
}
/// FUN_00174b28 (C2d0ab0_v8)
pub fn c2d0ab0_v8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef78, &[o], &[]);
    m.call(0x0017_49c0, &[o, 1], &[]);
    let p = m.r32(o + 0x10);
    let a = m.r32(o + 0x3c);
    m.call(0x0020_9970, &[p, a], &[]);
}

// ---- C2d07b0 --------------------------------------------------------------------------------------

/// FUN_001751c0 (v6)
pub fn c2d07b0_v6<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0017_4920, &[o], &[]);
    m.w32(o + 0x3c, 3);
}
/// FUN_001751f0 (v7)
pub fn c2d07b0_v7<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let p = m.r32(o + 0x10);
    m.w32(p + 0xf3c, 1);
    m.call(0x0017_4950, &[o], &[]);
    let t = m.r32(o + 0x40);
    if m.r32(t) == 2 {
        let q = m.r32(t + 8);
        m.w32(q + 0x60, 1);
        m.w32(q + 0x64, 1);
        m.w32(o + 0x50, 1);
    }
}
/// FUN_001752c0 (v8)
pub fn c2d07b0_v8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0017_4b28, &[o], &[]);
    let p = m.r32(o + 0x10);
    m.w32(p + 0xf3c, 0);
}
/// FUN_00175258 (v9)
pub fn c2d07b0_v9<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0017_4b68, &[o], &[]);
    let t = m.r32(o + 0x40);
    if t == 0 || m.r32(t + 8) == 0 {
        m.w32(o + 0x50, 0);
    }
    if m.r32(o + 0x50) != 0 {
        let a = m.r32(o + 0x38);
        let pos = m.call(0x001a_62e8, &[a], &[]).1;
        let t = m.r32(o + 0x40);
        let q = m.r32(t + 8);
        m.call(0x0020_c6f8, &[q], &[pos]);
    }
}
/// FUN_001752f0 (v12)
pub fn c2d07b0_v12<M: QMem + ?Sized>(m: &mut M, o: u32, msg: u32, a2: u32) -> u32 {
    if msg == 9 {
        let t = m.r32(o + 0x40);
        if t != 0 && m.call(0x0020_9518, &[t], &[]).0 == m.r32(a2) {
            let p = m.r32(o + 0x10);
            match m.r32(a2 + 8) {
                0x2e => {
                    m.w32(p + 0xf3c, 0);
                    return 1;
                }
                0x2d => {
                    m.w32(p + 0xf3c, 1);
                    return 1;
                }
                0x30 => {
                    m.w32(o + 0x50, 0);
                    let t = m.r32(o + 0x40);
                    let q = m.r32(t + 8);
                    ee_sub_20c760(m, q);
                    return 1;
                }
                _ => {}
            }
        }
    }
    m.call(0x0017_4bd0, &[o, msg, a2], &[]).0
}
/// FUN_0020c760: when +0x60 is set clear it and, when +0x68 is set, FUN_0012e460(+0x68),
/// FUN_002303a0(+0x34, (+0x34)->+0x58, 2), clear bit 26 of the u64 at (+0x34)+0x148, FUN_00134320(world, +0x34)
pub fn ee_sub_20c760<M: QMem + ?Sized>(m: &mut M, p: u32) {
    if m.r32(p + 0x60) == 0 {
        return;
    }
    m.w32(p + 0x60, 0);
    if m.r32(p + 0x68) == 0 {
        return;
    }
    m.call(0x0012_e460, &[p + 0x68], &[]);
    let t = m.r32(p + 0x34);
    let a = m.r32(t + 0x58);
    m.call(0x0023_03a0, &[t, a, 2], &[]);
    let lo = m.r32(t + 0x148) & 0xfbff_ffff;
    m.w32(t + 0x148, lo);
    m.call(0x0013_4320, &[WORLD, t], &[]);
}

// ---- C2d08b0 --------------------------------------------------------------------------------------

/// FUN_00179868 (v6)
pub fn c2d08b0_v6<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0017_4920, &[o], &[]);
    m.w32(o + 0x3c, 7);
}
/// FUN_00179898 (v7)
pub fn c2d08b0_v7<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let a = m.r32(o + 0x5c);
    let idx = m.call(0x001f_78a0, &[a], &[]).0;
    let p = m.r32(o + 0x10);
    let ent = p.wrapping_add(idx.wrapping_shl(4)).wrapping_add(0xc10);
    m.w32(o + 0x40, ent);
    if m.r32(ent) == NEG1 {
        m.w32(o + 0x40, 0);
    }
    m.call(0x0017_9328, &[o], &[]);
    let p = m.r32(o + 0x10);
    if m.r32(p + 0x330) != 0 {
        return;
    }
    m.w32(p + 0x330, 1);
    let w = m.call(0x001c_4840, &[WORLD], &[]).0;
    let cls = if m.r32(o + 0x50) == w { 0x18 } else { 0x17 };
    m.call(0x0016_19a0, &[0x002d_eb30, cls, p], &[]);
    let n = m.r32(p + 0x3d8) as i32;
    let count = if n >= 0 {
        n
    } else {
        let (s, e) = (m.r32(p + 0x488) as i32, m.r32(p + 0x484) as i32);
        let mut c = 0i32;
        let mut i = s;
        while i < e {
            let ent = p.wrapping_add((i as u32).wrapping_mul(0x10)).wrapping_add(0x404);
            if m.r32(ent) == 0x2b3 {
                c = c.wrapping_add(m.r32(ent + 12) as i32);
            }
            i += 1;
        }
        (c + 1) / 2
    };
    let lo = m.r32(p + 0x3e0);
    let pos = m.call(0x001a_6f00, &[p, 0, lo], &[]).1;
    let a3 = m.r32(p + 0x3f0);
    m.call(0x0015_4e10, &[0x002d_e578, count as u32, a3, p], &[pos]);
    m.call(0x0013_4320, &[WORLD, p], &[]);
    let h = m.r32(p + 0xf64);
    m.call(0x0015_3ff8, &[h], &[]);
}
/// FUN_00179a20 (v8)
pub fn c2d08b0_v8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0017_9598, &[o], &[]);
    let p = m.r32(o + 0x10);
    m.call(0x0013_4320, &[WORLD, p], &[]);
}

// ---- C2d0930 --------------------------------------------------------------------------------------

/// FUN_00179598 (v8): FUN_00174b28(o); FUN_00156c80(world, -1, {DAT_002a2dd0}, 0, owner + 0x35c)
pub fn c2d0930_v8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0017_4b28, &[o], &[]);
    let p = m.r32(o + 0x10);
    let q = m.rv(0x002a_2dd0);
    m.call(0x0015_6c80, &[WORLD, NEG1, 0, p + 0x35c], &[q]);
}
/// FUN_001796a0 (v6)
pub fn c2d0930_v6<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0017_4920, &[o], &[]);
    m.w32(o + 0x3c, 5);
}
/// FUN_001796d0 (v7)
pub fn c2d0930_v7<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let a = m.r32(o + 0x5c);
    let idx = m.call(0x001f_78a0, &[a], &[]).0;
    let p = m.r32(o + 0x10);
    let ent = p.wrapping_add(idx.wrapping_shl(4)).wrapping_add(0xa70);
    m.w32(o + 0x40, ent);
    if m.r32(ent) == NEG1 {
        m.w32(o + 0x40, 0);
    }
    m.call(0x0017_9328, &[o], &[]);
    let p = m.r32(o + 0x10);
    m.call(0x0016_19a0, &[0x002d_eb30, 0x15, p], &[]);
    let h = m.r32(p + 0xf64);
    let a = m.r32(o + 0x50);
    m.call(0x0015_4038, &[h, a, 2], &[]);
}

// ---- C2d0a30 --------------------------------------------------------------------------------------

/// FUN_00174fc8 (v6)
pub fn c2d0a30_v6<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0017_4920, &[o], &[]);
    m.w32(o + 0x3c, 2);
}
/// FUN_00174ff8 (v7)
pub fn c2d0a30_v7<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0017_4950, &[o], &[]);
    let p = m.r32(o + 0x10);
    if m.call(0x0020_9a68, &[p, 1], &[]).0 != 0 {
        let q = m.rv(0x002a_2dd0);
        m.call(0x0020_ab68, &[p, 0, 0], &[q]);
        let h = m.r32(p + 0xf64);
        m.call(0x0015_3ff8, &[h], &[]);
        let p10 = m.r32(o + 0x10);
        let v = m.r32(p10 + 0xfcc);
        if v != 0 {
            m.w32(o + 0x38, v);
        }
    }
    let p = m.r32(o + 0x10);
    let a = m.r32(p + 0x58);
    m.call(0x0023_03a0, &[p, a, 1], &[]);
    if m.r32(o + 0x38) == 0 {
        let p = m.r32(o + 0x10);
        let a = m.r32(p + 0x5c);
        m.call(0x0023_03a0, &[p, a, 1], &[]);
    }
}
/// FUN_00175090 (v8)
pub fn c2d0a30_v8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0017_4b28, &[o], &[]);
    let p = m.r32(o + 0x10);
    let a = m.r32(p + 0x58);
    m.call(0x0023_03a0, &[p, a, 2], &[]);
    if m.r32(o + 0x38) == 0 {
        let a = m.r32(p + 0x5c);
        m.call(0x0023_03a0, &[p, a, 2], &[]);
    }
}
/// FUN_001750e0 (v12)
pub fn c2d0a30_v12<M: QMem + ?Sized>(m: &mut M, o: u32, msg: u32, a2: u32) -> u32 {
    if msg == 9 {
        let t = m.r32(o + 0x40);
        if t != 0 && m.call(0x0020_9518, &[t], &[]).0 == m.r32(a2) {
            let p = m.r32(o + 0x10);
            match m.r32(a2 + 8) {
                0x29 => {
                    m.w32(p + 0x1a4, 1);
                    let a = m.r32(p + 0x5c);
                    m.call(0x0023_03a0, &[p, a, 2], &[]);
                    return 1;
                }
                0x2a => {
                    m.w32(p + 0x1a4, 0);
                    if m.r32(o + 0x38) != 0 {
                        return 1;
                    }
                    let a = m.r32(p + 0x5c);
                    m.call(0x0023_03a0, &[p, a, 1], &[]);
                    return 1;
                }
                _ => {}
            }
        }
    }
    m.call(0x0017_4bd0, &[o, msg, a2], &[]).0
}
