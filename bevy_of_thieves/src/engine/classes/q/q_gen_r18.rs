//! PART r18: mechanical 1:1 QMem ports of small branch-free functions, generated from the asm by
//! C:/Temp/PART/gen_r18.py (one per address; loads bound in program order, callee int args per the EE ABI).
#![allow(unused_variables, unused_mut, dead_code, unused_parens, clippy::all)]
use crate::engine::classes::q::q2mem::QMem;

/// FUN_00110ad8 (PART r18 asm-mechanical port)
pub fn ee_sub_110ad8<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x50], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d3540);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00110b48 (PART r18 asm-mechanical port)
pub fn ee_sub_110b48<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x170], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d33a0);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00110b80 (PART r18 asm-mechanical port)
pub fn ee_sub_110b80<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x120], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2dc070);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00110bb8 (PART r18 asm-mechanical port)
pub fn ee_sub_110bb8<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x100], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d6e08);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00110bf0 (PART r18 asm-mechanical port)
pub fn ee_sub_110bf0<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x110], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d5dc8);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00110c28 (PART r18 asm-mechanical port)
pub fn ee_sub_110c28<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x100], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d2cb8);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00110c60 (PART r18 asm-mechanical port)
pub fn ee_sub_110c60<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x120], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d2b18);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00110cd0 (PART r18 asm-mechanical port)
pub fn ee_sub_110cd0<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x1e0], &[]).0;
    m.w32(t1.wrapping_add(0x1b0), 0x2d56b8);
    m.w32(t1.wrapping_add(0xc), 0x2d56f8);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00110d18 (PART r18 asm-mechanical port)
pub fn ee_sub_110d18<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x1c0], &[]).0;
    m.w32(t1.wrapping_add(0x1b0), 0x2d26f8);
    m.w32(t1.wrapping_add(0xc), 0x2d2738);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00110d60 (PART r18 asm-mechanical port)
pub fn ee_sub_110d60<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x1d0], &[]).0;
    m.w32(t1.wrapping_add(0x1b0), 0x2d6fa8);
    m.w32(t1.wrapping_add(0xc), 0x2d6fe8);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00110da8 (PART r18 asm-mechanical port)
pub fn ee_sub_110da8<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x1f0], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d4de0);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00110de0 (PART r18 asm-mechanical port)
pub fn ee_sub_110de0<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x1c0], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2db6b8);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00110e18 (PART r18 asm-mechanical port)
pub fn ee_sub_110e18<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x180], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2cdba0);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00110e50 (PART r18 asm-mechanical port)
pub fn ee_sub_110e50<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x130], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d52b0);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00110e88 (PART r18 asm-mechanical port)
pub fn ee_sub_110e88<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x100], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2db518);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00110ec0 (PART r18 asm-mechanical port)
pub fn ee_sub_110ec0<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x140], &[]).0;
    m.w32(t1.wrapping_add(0x120), 0x2d2938);
    m.w32(t1.wrapping_add(0xc), 0x2d2978);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00110f08 (PART r18 asm-mechanical port)
pub fn ee_sub_110f08<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0xf0], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2c9a80);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00110f40 (PART r18 asm-mechanical port)
pub fn ee_sub_110f40<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x100], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2db378);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00110f78 (PART r18 asm-mechanical port)
pub fn ee_sub_110f78<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x130], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2dc3b0);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00110fb0 (PART r18 asm-mechanical port)
pub fn ee_sub_110fb0<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x140], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2dc210);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00110fe8 (PART r18 asm-mechanical port)
pub fn ee_sub_110fe8<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x2d0], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2cc7b0);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00111020 (PART r18 asm-mechanical port)
pub fn ee_sub_111020<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x2d0], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2c9c20);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00111058 (PART r18 asm-mechanical port)
pub fn ee_sub_111058<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x190], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2dc6f0);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00111090 (PART r18 asm-mechanical port)
pub fn ee_sub_111090<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x160], &[]).0;
    m.w32(t1.wrapping_add(0x150), 0x2d1b20);
    m.w32(t1.wrapping_add(0xc), 0x2d1b60);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00111138 (PART r18 asm-mechanical port)
pub fn ee_sub_111138<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x160], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2ceb78);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00111170 (PART r18 asm-mechanical port)
pub fn ee_sub_111170<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0xf0], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2ce828);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_001111a8 (PART r18 asm-mechanical port)
pub fn ee_sub_1111a8<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0xf0], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2ca098);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_001111e0 (PART r18 asm-mechanical port)
pub fn ee_sub_1111e0<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x110], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2ce4d0);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00111218 (PART r18 asm-mechanical port)
pub fn ee_sub_111218<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x110], &[]).0;
    m.w32(t1.wrapping_add(0x100), 0x2d2518);
    m.w32(t1.wrapping_add(0xc), 0x2d2558);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00111260 (PART r18 asm-mechanical port)
pub fn ee_sub_111260<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x180], &[]).0;
    m.w32(t1.wrapping_add(0x100), 0x2d1ea8);
    m.w32(t1.wrapping_add(0xc), 0x2d1ee8);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_001112a8 (PART r18 asm-mechanical port)
pub fn ee_sub_1112a8<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x1a0], &[]).0;
    m.w32(t1.wrapping_add(0x100), 0x2d1ea8);
    m.w32(t1.wrapping_add(0xc), 0x2d1d00);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_001112f0 (PART r18 asm-mechanical port)
pub fn ee_sub_1112f0<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x1a0], &[]).0;
    m.w32(t1.wrapping_add(0x100), 0x2d1ea8);
    m.w32(t1.wrapping_add(0xc), 0x2cb9f8);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00111338 (PART r18 asm-mechanical port)
pub fn ee_sub_111338<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x110], &[]).0;
    m.w32(t1.wrapping_add(0x100), 0x2d2338);
    m.w32(t1.wrapping_add(0xc), 0x2d2378);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00111380 (PART r18 asm-mechanical port)
pub fn ee_sub_111380<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x110], &[]).0;
    m.w32(t1.wrapping_add(0x100), 0x2d9d90);
    m.w32(t1.wrapping_add(0xc), 0x2d9dd0);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_001113c8 (PART r18 asm-mechanical port)
pub fn ee_sub_1113c8<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0xf0], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d3200);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00111400 (PART r18 asm-mechanical port)
pub fn ee_sub_111400<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0xe0], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2dc890);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00111438 (PART r18 asm-mechanical port)
pub fn ee_sub_111438<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x180], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2dc550);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00111470 (PART r18 asm-mechanical port)
pub fn ee_sub_111470<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x100], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2cabd8);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_001114a8 (PART r18 asm-mechanical port)
pub fn ee_sub_1114a8<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x140], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2ca9f0);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_001114e0 (PART r18 asm-mechanical port)
pub fn ee_sub_1114e0<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x130], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d4f80);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00111518 (PART r18 asm-mechanical port)
pub fn ee_sub_111518<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x120], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9f70);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00111550 (PART r18 asm-mechanical port)
pub fn ee_sub_111550<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x110], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2ce118);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00111588 (PART r18 asm-mechanical port)
pub fn ee_sub_111588<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x150], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9978);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    let t2 = m.call(0x00167310, &[t1.wrapping_add(0x80)], &[]).0;
    t1
}

/// FUN_001115d0 (PART r18 asm-mechanical port)
pub fn ee_sub_1115d0<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x150], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9978);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    let t2 = m.call(0x00167310, &[t1.wrapping_add(0x80)], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d4260);
    t1
}

/// FUN_00111628 (PART r18 asm-mechanical port)
pub fn ee_sub_111628<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x180], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9978);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    let t2 = m.call(0x00167310, &[t1.wrapping_add(0x80)], &[]).0;
    m.w32(t1.wrapping_add(0x174), 0);
    m.w32(t1.wrapping_add(0xc), 0x2d5898);
    t1
}

/// FUN_00111680 (PART r18 asm-mechanical port)
pub fn ee_sub_111680<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x2c0], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9978);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    let t2 = m.call(0x00167310, &[t1.wrapping_add(0x80)], &[]).0;
    m.w32(t1.wrapping_add(0x174), 0);
    m.w32(t1.wrapping_add(0xc), 0x2da110);
    t1
}

/// FUN_00111728 (PART r18 asm-mechanical port)
pub fn ee_sub_111728<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x1880], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9978);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    let t2 = m.call(0x00167310, &[t1.wrapping_add(0x80)], &[]).0;
    m.w32(t1.wrapping_add(0x174), 0);
    m.w32(t1.wrapping_add(0xc), 0x2cb230);
    t1
}

/// FUN_00111a68 (PART r18 asm-mechanical port)
pub fn ee_sub_111a68<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x410], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9978);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    let t2 = m.call(0x00167310, &[t1.wrapping_add(0x80)], &[]).0;
    m.w32(t1.wrapping_add(0x174), 0);
    m.w32(t1.wrapping_add(0xc), 0x2cd748);
    t1
}

/// FUN_00111ac0 (PART r18 asm-mechanical port)
pub fn ee_sub_111ac0<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x1e0], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9978);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    let t2 = m.call(0x00167310, &[t1.wrapping_add(0x80)], &[]).0;
    m.w32(t1.wrapping_add(0x174), 0);
    m.w32(t1.wrapping_add(0xc), 0x2dbd80);
    t1
}

/// FUN_00111b18 (PART r18 asm-mechanical port)
pub fn ee_sub_111b18<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x240], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9978);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    let t2 = m.call(0x00167310, &[t1.wrapping_add(0x80)], &[]).0;
    m.w32(t1.wrapping_add(0x174), 0);
    m.w32(t1.wrapping_add(0xc), 0x2d80d0);
    t1
}

/// FUN_00111b70 (PART r18 asm-mechanical port)
pub fn ee_sub_111b70<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x1c0], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9978);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    let t2 = m.call(0x00167310, &[t1.wrapping_add(0x80)], &[]).0;
    m.w32(t1.wrapping_add(0x174), 0);
    m.w32(t1.wrapping_add(0xc), 0x2d1878);
    t1
}

/// FUN_00111bc8 (PART r18 asm-mechanical port)
pub fn ee_sub_111bc8<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x2b0], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9978);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    let t2 = m.call(0x00167310, &[t1.wrapping_add(0x80)], &[]).0;
    m.w32(t1.wrapping_add(0x174), 0);
    m.w32(t1.wrapping_add(0xc), 0x2d15d0);
    m.w32(t1.wrapping_add(0x2a4), 0);
    t1
}

/// FUN_00111c28 (PART r18 asm-mechanical port)
pub fn ee_sub_111c28<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x390], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9978);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    let t2 = m.call(0x00167310, &[t1.wrapping_add(0x80)], &[]).0;
    m.w32(t1.wrapping_add(0x174), 0);
    m.w32(t1.wrapping_add(0xc), 0x2d1080);
    t1
}

/// FUN_00111c80 (PART r18 asm-mechanical port)
pub fn ee_sub_111c80<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x1e0], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9978);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    let t2 = m.call(0x00167310, &[t1.wrapping_add(0x80)], &[]).0;
    m.w32(t1.wrapping_add(0x174), 0);
    m.w32(t1.wrapping_add(0xc), 0x2d1328);
    t1
}

/// FUN_00111cd8 (PART r18 asm-mechanical port)
pub fn ee_sub_111cd8<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x250], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9978);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    let t2 = m.call(0x00167310, &[t1.wrapping_add(0x80)], &[]).0;
    m.w32(t1.wrapping_add(0x174), 0);
    m.w32(t1.wrapping_add(0xc), 0x2d0dd8);
    t1
}

/// FUN_00111d30 (PART r18 asm-mechanical port)
pub fn ee_sub_111d30<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x2a0], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9978);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    let t2 = m.call(0x00167310, &[t1.wrapping_add(0x80)], &[]).0;
    m.w32(t1.wrapping_add(0x174), 0);
    m.w32(t1.wrapping_add(0xc), 0x2d0b30);
    t1
}

/// FUN_00111d88 (PART r18 asm-mechanical port)
pub fn ee_sub_111d88<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x5d0], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9978);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    let t2 = m.call(0x00167310, &[t1.wrapping_add(0x80)], &[]).0;
    m.w32(t1.wrapping_add(0x174), 0);
    m.w32(t1.wrapping_add(0xc), 0x2d7ca8);
    m.w32(t1.wrapping_add(0x598), 0);
    t1
}

/// FUN_00111de8 (PART r18 asm-mechanical port)
pub fn ee_sub_111de8<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x230], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9978);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    let t2 = m.call(0x00167310, &[t1.wrapping_add(0x80)], &[]).0;
    m.w32(t1.wrapping_add(0x174), 0);
    m.w32(t1.wrapping_add(0xc), 0x2d5b30);
    t1
}

/// FUN_00111e40 (PART r18 asm-mechanical port)
pub fn ee_sub_111e40<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x210], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9978);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    let t2 = m.call(0x00167310, &[t1.wrapping_add(0x80)], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2cb780);
    t1
}

/// FUN_00111e98 (PART r18 asm-mechanical port)
pub fn ee_sub_111e98<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x270], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9978);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    let t2 = m.call(0x00167310, &[t1.wrapping_add(0x80)], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d3fa0);
    t1
}

/// FUN_00111ef0 (PART r18 asm-mechanical port)
pub fn ee_sub_111ef0<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x280], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9978);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    let t2 = m.call(0x00167310, &[t1.wrapping_add(0x80)], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d3d20);
    t1
}

/// FUN_00111f48 (PART r18 asm-mechanical port)
pub fn ee_sub_111f48<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x290], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9978);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    let t2 = m.call(0x00167310, &[t1.wrapping_add(0x80)], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d3aa0);
    t1
}

/// FUN_00111fa0 (PART r18 asm-mechanical port)
pub fn ee_sub_111fa0<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x300], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9978);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    let t2 = m.call(0x00167310, &[t1.wrapping_add(0x80)], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2cc950);
    t1
}

/// FUN_00111ff8 (PART r18 asm-mechanical port)
pub fn ee_sub_111ff8<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x1d0], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9978);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    let t2 = m.call(0x00167310, &[t1.wrapping_add(0x80)], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d8738);
    t1
}

/// FUN_00112050 (PART r18 asm-mechanical port)
pub fn ee_sub_112050<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x180], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9978);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    let t2 = m.call(0x00167310, &[t1.wrapping_add(0x80)], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2caf88);
    t1
}

/// FUN_001120a8 (PART r18 asm-mechanical port)
pub fn ee_sub_1120a8<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x1b0], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9978);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    let t2 = m.call(0x00167310, &[t1.wrapping_add(0x80)], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d77d8);
    t1
}

/// FUN_00112100 (PART r18 asm-mechanical port)
pub fn ee_sub_112100<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x1a0], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9978);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    let t2 = m.call(0x00167310, &[t1.wrapping_add(0x80)], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d20d0);
    m.w32(t1.wrapping_add(0x170), 0x2d2090);
    t1
}

/// FUN_00112160 (PART r18 asm-mechanical port)
pub fn ee_sub_112160<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x190], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9978);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    let t2 = m.call(0x00167310, &[t1.wrapping_add(0x80)], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2dca30);
    t1
}

/// FUN_001121b8 (PART r18 asm-mechanical port)
pub fn ee_sub_1121b8<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x240], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9978);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    let t2 = m.call(0x00167310, &[t1.wrapping_add(0x80)], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d6928);
    t1
}

/// FUN_00112210 (PART r18 asm-mechanical port)
pub fn ee_sub_112210<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x240], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9978);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    let t2 = m.call(0x00167310, &[t1.wrapping_add(0x80)], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d6448);
    t1
}

/// FUN_00112268 (PART r18 asm-mechanical port)
pub fn ee_sub_112268<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x250], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9978);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    let t2 = m.call(0x00167310, &[t1.wrapping_add(0x80)], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d66b8);
    t1
}

/// FUN_001122c0 (PART r18 asm-mechanical port)
pub fn ee_sub_1122c0<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x250], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9978);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    let t2 = m.call(0x00167310, &[t1.wrapping_add(0x80)], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d61d8);
    t1
}

/// FUN_00112318 (PART r18 asm-mechanical port)
pub fn ee_sub_112318<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x230], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9978);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    let t2 = m.call(0x00167310, &[t1.wrapping_add(0x80)], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d5f68);
    t1
}

/// FUN_00112370 (PART r18 asm-mechanical port)
pub fn ee_sub_112370<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x250], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9978);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    let t2 = m.call(0x00167310, &[t1.wrapping_add(0x80)], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d3838);
    t1
}

/// FUN_001123c8 (PART r18 asm-mechanical port)
pub fn ee_sub_1123c8<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x160], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9978);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    let t2 = m.call(0x00167310, &[t1.wrapping_add(0x80)], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9500);
    t1
}

/// FUN_00112420 (PART r18 asm-mechanical port)
pub fn ee_sub_112420<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x180], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9978);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    let t2 = m.call(0x00167310, &[t1.wrapping_add(0x80)], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d4b78);
    t1
}

/// FUN_00112478 (PART r18 asm-mechanical port)
pub fn ee_sub_112478<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x170], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9978);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    let t2 = m.call(0x00167310, &[t1.wrapping_add(0x80)], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d5450);
    t1
}

/// FUN_001124d0 (PART r18 asm-mechanical port)
pub fn ee_sub_1124d0<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x1b0], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9978);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    let t2 = m.call(0x00167310, &[t1.wrapping_add(0x80)], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2dbb10);
    t1
}

/// FUN_00112528 (PART r18 asm-mechanical port)
pub fn ee_sub_112528<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x1b0], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9978);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    let t2 = m.call(0x00167310, &[t1.wrapping_add(0x80)], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2c9810);
    t1
}

/// FUN_00112580 (PART r18 asm-mechanical port)
pub fn ee_sub_112580<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x1d0], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9978);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    let t2 = m.call(0x00167310, &[t1.wrapping_add(0x80)], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2db8a0);
    t1
}

/// FUN_001125d8 (PART r18 asm-mechanical port)
pub fn ee_sub_1125d8<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x190], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9978);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    let t2 = m.call(0x00167310, &[t1.wrapping_add(0x80)], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2cbba0);
    t1
}

/// FUN_00112630 (PART r18 asm-mechanical port)
pub fn ee_sub_112630<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x180], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9978);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    let t2 = m.call(0x00167310, &[t1.wrapping_add(0x80)], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2cc548);
    t1
}

/// FUN_00112688 (PART r18 asm-mechanical port)
pub fn ee_sub_112688<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x1a0], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9978);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    let t2 = m.call(0x00167310, &[t1.wrapping_add(0x80)], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2db0d0);
    t1
}

/// FUN_001126e0 (PART r18 asm-mechanical port)
pub fn ee_sub_1126e0<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x160], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9978);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    let t2 = m.call(0x00167310, &[t1.wrapping_add(0x80)], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d8dd8);
    t1
}

/// FUN_00112738 (PART r18 asm-mechanical port)
pub fn ee_sub_112738<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x190], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9978);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    let t2 = m.call(0x00167310, &[t1.wrapping_add(0x80)], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2dcc98);
    t1
}

/// FUN_00112790 (PART r18 asm-mechanical port)
pub fn ee_sub_112790<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x170], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9978);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    let t2 = m.call(0x00167310, &[t1.wrapping_add(0x80)], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d7a40);
    t1
}

/// FUN_001127e8 (PART r18 asm-mechanical port)
pub fn ee_sub_1127e8<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0xc0], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2cad78);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00112820 (PART r18 asm-mechanical port)
pub fn ee_sub_112820<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0xe0], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2ca300);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00112858 (PART r18 asm-mechanical port)
pub fn ee_sub_112858<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x80], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d97a8);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00112890 (PART r18 asm-mechanical port)
pub fn ee_sub_112890<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0xc0], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d7f40);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00112900 (PART r18 asm-mechanical port)
pub fn ee_sub_112900<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0xb0], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d8c48);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00112938 (PART r18 asm-mechanical port)
pub fn ee_sub_112938<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x900], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2dcf00);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00112970 (PART r18 asm-mechanical port)
pub fn ee_sub_112970<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x70], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2cf058);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_001129a8 (PART r18 asm-mechanical port)
pub fn ee_sub_1129a8<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x70], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2ce2f8);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_001129e0 (PART r18 asm-mechanical port)
pub fn ee_sub_1129e0<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x60], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2c94e0);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00112a18 (PART r18 asm-mechanical port)
pub fn ee_sub_112a18<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0xb0], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2cfa90);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00112a50 (PART r18 asm-mechanical port)
pub fn ee_sub_112a50<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0xd0], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2da840);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00112a88 (PART r18 asm-mechanical port)
pub fn ee_sub_112a88<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x10b0], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2cd5b8);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00112ac0 (PART r18 asm-mechanical port)
pub fn ee_sub_112ac0<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x60], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2cda10);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00112af8 (PART r18 asm-mechanical port)
pub fn ee_sub_112af8<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x60], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2ccf10);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00112bb0 (PART r18 asm-mechanical port)
pub fn ee_sub_112bb0<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x60], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2da3d8);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00112be8 (PART r18 asm-mechanical port)
pub fn ee_sub_112be8<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x110], &[]).0;
    let t2 = m.call(0x00212158, &[t1], &[]).0;
    t2
}

/// FUN_00112c10 (PART r18 asm-mechanical port)
pub fn ee_sub_112c10<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x120], &[]).0;
    let t2 = m.call(0x00212158, &[t1], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2dabf0);
    t1
}

/// FUN_00112c50 (PART r18 asm-mechanical port)
pub fn ee_sub_112c50<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x110], &[]).0;
    let t2 = m.call(0x00212158, &[t1], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2daa50);
    t1
}

/// FUN_00112c90 (PART r18 asm-mechanical port)
pub fn ee_sub_112c90<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x220], &[]).0;
    let t2 = m.call(0x00212158, &[t1], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2dad90);
    t1
}

/// FUN_00112cd0 (PART r18 asm-mechanical port)
pub fn ee_sub_112cd0<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x130], &[]).0;
    let t2 = m.call(0x00211c58, &[t1], &[]).0;
    t2
}

/// FUN_00112cf8 (PART r18 asm-mechanical port)
pub fn ee_sub_112cf8<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x140], &[]).0;
    let t2 = m.call(0x00212158, &[t1], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2ceeb8);
    t1
}

/// FUN_00112d38 (PART r18 asm-mechanical port)
pub fn ee_sub_112d38<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x100], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2cf378);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00112d70 (PART r18 asm-mechanical port)
pub fn ee_sub_112d70<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x110], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2cf1e8);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00112dd0 (PART r18 asm-mechanical port)
pub fn ee_sub_112dd0<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x50], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9768);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00112e08 (PART r18 asm-mechanical port)
pub fn ee_sub_112e08<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x70], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d93c0);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00112e40 (PART r18 asm-mechanical port)
pub fn ee_sub_112e40<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x70], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9360);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00112e78 (PART r18 asm-mechanical port)
pub fn ee_sub_112e78<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x60], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9300);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00112eb0 (PART r18 asm-mechanical port)
pub fn ee_sub_112eb0<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x80], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d92a0);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00112ee8 (PART r18 asm-mechanical port)
pub fn ee_sub_112ee8<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x70], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9240);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00112f20 (PART r18 asm-mechanical port)
pub fn ee_sub_112f20<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x60], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d91e0);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00112f58 (PART r18 asm-mechanical port)
pub fn ee_sub_112f58<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x70], &[]).0;
    let t2 = m.call(0x0020c058, &[t1], &[]).0;
    t2
}

/// FUN_00112f80 (PART r18 asm-mechanical port)
pub fn ee_sub_112f80<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x40], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2da9d0);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00112fb8 (PART r18 asm-mechanical port)
pub fn ee_sub_112fb8<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0xb0], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2c94a0);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00112ff0 (PART r18 asm-mechanical port)
pub fn ee_sub_112ff0<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x30], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2ca490);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00113028 (PART r18 asm-mechanical port)
pub fn ee_sub_113028<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x30], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d9480);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_001130b0 (PART r18 asm-mechanical port)
pub fn ee_sub_1130b0<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x50], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d0ab0);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_001130e8 (PART r18 asm-mechanical port)
pub fn ee_sub_1130e8<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x50], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d0a30);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00113120 (PART r18 asm-mechanical port)
pub fn ee_sub_113120<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x60], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d07b0);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00113158 (PART r18 asm-mechanical port)
pub fn ee_sub_113158<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0xc0], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d09b0);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00113190 (PART r18 asm-mechanical port)
pub fn ee_sub_113190<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0xc0], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d0930);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_001131c8 (PART r18 asm-mechanical port)
pub fn ee_sub_1131c8<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0xc0], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d0830);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00113200 (PART r18 asm-mechanical port)
pub fn ee_sub_113200<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0xc0], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d08b0);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00113238 (PART r18 asm-mechanical port)
pub fn ee_sub_113238<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x50], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d0620);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00113270 (PART r18 asm-mechanical port)
pub fn ee_sub_113270<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x60], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d05a0);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_001132a8 (PART r18 asm-mechanical port)
pub fn ee_sub_1132a8<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0xc0], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d0728);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_001132e0 (PART r18 asm-mechanical port)
pub fn ee_sub_1132e0<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0xf0], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d06a0);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00113318 (PART r18 asm-mechanical port)
pub fn ee_sub_113318<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0xe0], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2cc4c0);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00113350 (PART r18 asm-mechanical port)
pub fn ee_sub_113350<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x60], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2cf7f8);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00113388 (PART r18 asm-mechanical port)
pub fn ee_sub_113388<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x50], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d0520);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_001133c0 (PART r18 asm-mechanical port)
pub fn ee_sub_1133c0<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0xe0], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2cfa10);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_001133f8 (PART r18 asm-mechanical port)
pub fn ee_sub_1133f8<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0xa0], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d0320);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00113430 (PART r18 asm-mechanical port)
pub fn ee_sub_113430<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x40], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d0220);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00113468 (PART r18 asm-mechanical port)
pub fn ee_sub_113468<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x40], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d4af8);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_001134a0 (PART r18 asm-mechanical port)
pub fn ee_sub_1134a0<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x60], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d02a0);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_001134d8 (PART r18 asm-mechanical port)
pub fn ee_sub_1134d8<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x40], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d0420);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00113510 (PART r18 asm-mechanical port)
pub fn ee_sub_113510<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x70], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d03a0);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00113548 (PART r18 asm-mechanical port)
pub fn ee_sub_113548<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x50], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d01a0);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00113580 (PART r18 asm-mechanical port)
pub fn ee_sub_113580<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x40], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d0120);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_001135b8 (PART r18 asm-mechanical port)
pub fn ee_sub_1135b8<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x40], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2cddc0);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_001135f0 (PART r18 asm-mechanical port)
pub fn ee_sub_1135f0<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x60], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2cdd40);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00113628 (PART r18 asm-mechanical port)
pub fn ee_sub_113628<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x50], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2cce90);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00113660 (PART r18 asm-mechanical port)
pub fn ee_sub_113660<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x70], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d00a0);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00113698 (PART r18 asm-mechanical port)
pub fn ee_sub_113698<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x40], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2d0020);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_001136d0 (PART r18 asm-mechanical port)
pub fn ee_sub_1136d0<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x40], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2cffa0);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00113760 (PART r18 asm-mechanical port)
pub fn ee_sub_113760<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x30], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2cff20);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00113798 (PART r18 asm-mechanical port)
pub fn ee_sub_113798<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x30], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2cfea0);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_001137d0 (PART r18 asm-mechanical port)
pub fn ee_sub_1137d0<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x50], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2cfd20);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00113808 (PART r18 asm-mechanical port)
pub fn ee_sub_113808<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x50], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2cfca0);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00113888 (PART r18 asm-mechanical port)
pub fn ee_sub_113888<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x60], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2cfda0);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_001138c0 (PART r18 asm-mechanical port)
pub fn ee_sub_1138c0<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x50], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2cc440);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_001138f8 (PART r18 asm-mechanical port)
pub fn ee_sub_1138f8<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0xa0], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2cc3c0);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00113930 (PART r18 asm-mechanical port)
pub fn ee_sub_113930<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x40], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2cc340);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_00113968 (PART r18 asm-mechanical port)
pub fn ee_sub_113968<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x40], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2cc2c0);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_001139a0 (PART r18 asm-mechanical port)
pub fn ee_sub_1139a0<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x40], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2cc240);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_001139d8 (PART r18 asm-mechanical port)
pub fn ee_sub_1139d8<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001ae4f8, &[0x40], &[]).0;
    m.w32(t1.wrapping_add(0xc), 0x2cc1c0);
    m.w32(t1, 0);
    m.w32(t1.wrapping_add(4), 0);
    t1
}

/// FUN_001202d0 (PART r18 asm-mechanical port)
pub fn ee_sub_1202d0<M: QMem + ?Sized>(m: &mut M, p0: u32) -> u32 {
    let t1 = m.call(0x0012a0b0, &[p0, 0x24], &[]).0;
    p0
}

/// FUN_00120300 (PART r18 asm-mechanical port)
pub fn ee_sub_120300<M: QMem + ?Sized>(m: &mut M, p0: u32) -> u32 {
    let t1 = m.call(0x0012a0b0, &[p0, 0x24], &[]).0;
    p0
}

/// FUN_00120488 (PART r18 asm-mechanical port)
pub fn ee_sub_120488<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32, p3: u32, p4: u32) -> u32 {
    let t1 = m.call(0x00120398, &[p0, p2, 0, p3, p4], &[]).0;
    p0
}

/// FUN_001204b8 (PART r18 asm-mechanical port)
pub fn ee_sub_1204b8<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32, p3: u32, p4: u32) -> u32 {
    let t1 = m.call(0x00120398, &[p0, p2, 1, p3, p4], &[]).0;
    p0
}

/// FUN_001204e8 (PART r18 asm-mechanical port)
pub fn ee_sub_1204e8<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32, p3: u32, p4: u32) -> u32 {
    let t1 = m.call(0x00120398, &[p0, p2, 2, p3, p4], &[]).0;
    p0
}

/// FUN_00120518 (PART r18 asm-mechanical port)
pub fn ee_sub_120518<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32, p3: u32, p4: u32) -> u32 {
    let t1 = m.call(0x00120398, &[p0, p2, 3, p3, p4], &[]).0;
    p0
}

/// FUN_001207f0 (PART r18 asm-mechanical port)
pub fn ee_sub_1207f0<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32) -> u32 {
    let t1 = m.call(0x001205b8, &[p0, p2, p2.wrapping_add(8)], &[]).0;
    p0
}

/// FUN_00120bd0 (PART r18 asm-mechanical port)
pub fn ee_sub_120bd0<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32) -> u32 {
    let t1 = m.r32(p2.wrapping_add(4));
    let t2 = m.call(0x001291a8, &[p0, t1.wrapping_add(8)], &[]).0;
    p0
}

/// FUN_00120f80 (PART r18 asm-mechanical port)
pub fn ee_sub_120f80<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32) -> u32 {
    let t1 = m.call(0x001291a8, &[p0, p2], &[]).0;
    p0
}

/// FUN_00122560 (PART r18 asm-mechanical port)
pub fn ee_sub_122560<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32) -> u32 {
    let t1 = m.call(0x001224a8, &[p0, p2, 1], &[]).0;
    p0
}

/// FUN_00122590 (PART r18 asm-mechanical port)
pub fn ee_sub_122590<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32) -> u32 {
    let t1 = m.call(0x001224a8, &[p0, p2, 2], &[]).0;
    p0
}

/// FUN_001225c0 (PART r18 asm-mechanical port)
pub fn ee_sub_1225c0<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32) -> u32 {
    let t1 = m.call(0x001224a8, &[p0, p2, 3], &[]).0;
    p0
}

/// FUN_001225f0 (PART r18 asm-mechanical port)
pub fn ee_sub_1225f0<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32) -> u32 {
    let t1 = m.call(0x001224a8, &[p0, p2, 4], &[]).0;
    p0
}

/// FUN_00122620 (PART r18 asm-mechanical port)
pub fn ee_sub_122620<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32) -> u32 {
    let t1 = m.call(0x001224a8, &[p0, p2, 5], &[]).0;
    p0
}

/// FUN_00122650 (PART r18 asm-mechanical port)
pub fn ee_sub_122650<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32) -> u32 {
    let t1 = m.call(0x001224a8, &[p0, p2, 6], &[]).0;
    p0
}

/// FUN_00122680 (PART r18 asm-mechanical port)
pub fn ee_sub_122680<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32) -> u32 {
    let t1 = m.call(0x001224a8, &[p0, p2, 7], &[]).0;
    p0
}

/// FUN_00123a28 (PART r18 asm-mechanical port)
pub fn ee_sub_123a28<M: QMem + ?Sized>(m: &mut M, p0: u32) -> u32 {
    let t1 = m.call(0x00210a88, &[], &[]).0;
    m.w32(p0.wrapping_add(4), t1);
    m.w32(p0, 0x15);
    p0
}

/// FUN_00123ba0 (PART r18 asm-mechanical port)
pub fn ee_sub_123ba0<M: QMem + ?Sized>(m: &mut M, p0: u32) -> u32 {
    let t1 = m.call(0x00210b68, &[], &[]).0;
    m.w32(p0.wrapping_add(4), t1);
    m.w32(p0, 0x15);
    p0
}

/// FUN_001251e8 (PART r18 asm-mechanical port)
pub fn ee_sub_1251e8<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32) -> u32 {
    let t1 = m.r32(p2.wrapping_add(0xc));
    let t2 = m.r32(p2.wrapping_add(4));
    let t3 = m.call(0x001fbb90, &[t2], &[]).0;
    let t4 = m.call(0x001fb2f8, &[t3, t1], &[]).0;
    let t5 = m.call(0x001294c0, &[p0, t3], &[]).0;
    p0
}

/// FUN_001255c0 (PART r18 asm-mechanical port)
pub fn ee_sub_1255c0<M: QMem + ?Sized>(m: &mut M, p0: u32) -> u32 {
    let t1 = m.call(0x0012a0b0, &[p0, 0x24], &[]).0;
    p0
}

/// FUN_00128b40 (PART r18 asm-mechanical port)
pub fn ee_sub_128b40<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x00128a00, &[1, 0xffff], &[]).0;
    t1
}

/// FUN_00128c98 (PART r18 asm-mechanical port)
pub fn ee_sub_128c98<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x00128b60, &[1, 0xffff], &[]).0;
    t1
}

/// FUN_00128f50 (PART r18 asm-mechanical port)
pub fn ee_sub_128f50<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x00128e18, &[1, 0xffff], &[]).0;
    t1
}

/// FUN_00129188 (PART r18 asm-mechanical port)
pub fn ee_sub_129188<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x00129050, &[1, 0xffff], &[]).0;
    t1
}

/// FUN_001294a0 (PART r18 asm-mechanical port)
pub fn ee_sub_1294a0<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32) -> u32 {
    m.w32(p0.wrapping_add(4), p1);
    m.w32(p0, 0x20);
    let t1 = m.r32(p1);
    m.w32(p1, t1.wrapping_add(1));
    p0
}

/// FUN_0012e738 (PART r18 asm-mechanical port)
pub fn ee_sub_12e738<M: QMem + ?Sized>(m: &mut M, p0: u32) -> u32 {

    p0.wrapping_add(0x128)
}

/// FUN_00131f50 (PART r18 asm-mechanical port)
pub fn ee_sub_131f50<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x00131e18, &[1, 0xffff], &[]).0;
    t1
}

/// FUN_001320a8 (PART r18 asm-mechanical port)
pub fn ee_sub_1320a8<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x00131f70, &[1, 0xffff], &[]).0;
    t1
}

/// FUN_00132550 (PART r18 asm-mechanical port)
pub fn ee_sub_132550<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x00132418, &[1, 0xffff], &[]).0;
    t1
}

/// FUN_00132ae0 (PART r18 asm-mechanical port)
pub fn ee_sub_132ae0<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001329a8, &[1, 0xffff], &[]).0;
    t1
}

/// FUN_001336d0 (PART r18 asm-mechanical port)
pub fn ee_sub_1336d0<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x00133598, &[1, 0xffff], &[]).0;
    t1
}

/// FUN_001371b8 (PART r18 asm-mechanical port)
pub fn ee_sub_1371b8<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32) -> u32 {
    m.w32(p0, p1);
    m.w32(p0.wrapping_add(4), 0xffffffff);
    0xffffffff
}

/// FUN_00138fc0 (PART r18 asm-mechanical port)
pub fn ee_sub_138fc0<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x00138f08, &[1, 0x2dd5c0], &[]).0;
    let t2 = m.call(0x00138f08, &[2, 0x2dd600], &[]).0;
    t2
}

/// FUN_00139098 (PART r18 asm-mechanical port)
pub fn ee_sub_139098<M: QMem + ?Sized>(m: &mut M, p0: u32) -> u32 {
    let t1 = m.r32(p0.wrapping_add(0x108c));
    let t2 = m.call(0x0011a2e8, &[t1.wrapping_add(0x448), p0], &[]).0;
    let t3 = m.call(0x0011a220, &[0x2deb00, p0], &[]).0;
    t3
}

/// FUN_0013c440 (PART r18 asm-mechanical port)
pub fn ee_sub_13c440<M: QMem + ?Sized>(m: &mut M, p0: u32) {
    m.w32(p0.wrapping_add(0x158), 0);
}

/// FUN_00144268 (PART r18 asm-mechanical port)
pub fn ee_sub_144268<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32) -> u32 {
    let t1 = m.call(0x001c4228, &[p0, p1, p2], &[]).0;
    let t2 = m.r32(p0.wrapping_add(0xe00));
    let t3 = m.r32(t2);
    let t4 = m.call(0x00144188, &[p0, t3.wrapping_add(p1)], &[]).0;
    let t5 = m.call(0x001441a0, &[p0], &[]).0;
    let t6 = m.call(0x001441c0, &[p0, t5.wrapping_add(p2)], &[]).0;
    t6
}

/// FUN_00148af8 (PART r18 asm-mechanical port)
pub fn ee_sub_148af8<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32) -> u32 {
    let t1 = m.call(0x00148620, &[p0.wrapping_add(4), p1, p2], &[]).0;
    t1
}

/// FUN_00148b18 (PART r18 asm-mechanical port)
pub fn ee_sub_148b18<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32) -> u32 {
    let t1 = m.call(0x00148620, &[p0.wrapping_add(0x48), p1, p2], &[]).0;
    t1
}

/// FUN_00148b38 (PART r18 asm-mechanical port)
pub fn ee_sub_148b38<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32, p3: u32, p4: u32) -> u32 {
    let t1 = m.call(0x0020fec0, &[0x3dd490, p0.wrapping_add(0xb4), 8, p0.wrapping_add(0xb0), p1, p2, p3, p4], &[]).0;
    t1
}

/// FUN_00148b78 (PART r18 asm-mechanical port)
pub fn ee_sub_148b78<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32) -> u32 {
    let t1 = m.call(0x00161f10, &[0x2deb30, p0.wrapping_add(0x138), 4, p0.wrapping_add(0x134), p1, p2], &[]).0;
    t1
}

/// FUN_00148c70 (PART r18 asm-mechanical port)
pub fn ee_sub_148c70<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32) -> u32 {
    let t1 = m.call(0x001f6068, &[p0, p1], &[]).0;
    m.w32(p0.wrapping_add(0x150), p0);
    t1
}

/// FUN_001491e0 (PART r18 asm-mechanical port)
pub fn ee_sub_1491e0<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32) -> u32 {
    let t1 = m.call(0x00148af8, &[p0.wrapping_add(0x150), p1, p2], &[]).0;
    t1
}

/// FUN_00149200 (PART r18 asm-mechanical port)
pub fn ee_sub_149200<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32) -> u32 {
    let t1 = m.call(0x00148b18, &[p0.wrapping_add(0x150), p1, p2], &[]).0;
    t1
}

/// FUN_00149270 (PART r18 asm-mechanical port)
pub fn ee_sub_149270<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32, p3: u32, p4: u32) -> u32 {
    let t1 = m.call(0x00148b38, &[p0.wrapping_add(0x150), p1, p2, p3, p4], &[]).0;
    t1
}

/// FUN_00149290 (PART r18 asm-mechanical port)
pub fn ee_sub_149290<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32) -> u32 {
    let t1 = m.call(0x00148b78, &[p0.wrapping_add(0x150), p1, p2], &[]).0;
    t1
}

/// FUN_0015ef00 (PART r18 asm-mechanical port)
pub fn ee_sub_15ef00<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x0015ef30, &[], &[]).0;
    let t2 = m.r32(0x2de9f0);
    m.w32(0x2de760, t2);
    0x2e0000
}

/// FUN_0015f888 (PART r18 asm-mechanical port)
pub fn ee_sub_15f888<M: QMem + ?Sized>(m: &mut M, p0: u32) -> u32 {
    let t1 = m.r32(p0);
    m.w32((t1 << 2).wrapping_add(0x2de9f0), p0);
    (t1 << 2).wrapping_add(0x2de9f0)
}

/// FUN_0015f928 (PART r18 asm-mechanical port)
pub fn ee_sub_15f928<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32) -> u32 {
    m.w32(p0, p1);
    m.w32(p0.wrapping_add(0xc), p2);
    m.w32(p0.wrapping_add(0x154), 0xff808080);
    m.w32(p0.wrapping_add(0x160), 0xff808080);
    m.w32(p0.wrapping_add(0x15c), 0xff808080);
    m.w32(p0.wrapping_add(0x158), 0xff808080);
    let t1 = m.call(0x0015f970, &[p0], &[]).0;
    p0
}

/// FUN_00162660 (PART r18 asm-mechanical port)
pub fn ee_sub_162660<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32, p3: u32) {
    m.w32(p0.wrapping_add(0xb4), p3);
    m.w32(p0.wrapping_add(0xac), p1);
    m.w32(p0.wrapping_add(0xb0), p2);
}

/// FUN_00162dd0 (PART r18 asm-mechanical port)
pub fn ee_sub_162dd0<M: QMem + ?Sized>(m: &mut M, p0: u32) -> u32 {
    let t1 = m.call(0x001619a0, &[0x2deb30, 0x1d, p0], &[]).0;
    t1
}

/// FUN_001657a8 (PART r18 asm-mechanical port)
pub fn ee_sub_1657a8<M: QMem + ?Sized>(m: &mut M, p0: u32) -> u32 {
    let t1 = m.r32(p0.wrapping_add(0x7c));
    let t2 = m.call(0x001619a0, &[0x2deb30, 0x1e, t1], &[]).0;
    t2
}

/// FUN_0016ea20 (PART r18 asm-mechanical port)
pub fn ee_sub_16ea20<M: QMem + ?Sized>(m: &mut M, p0: u32) -> u32 {
    let t1 = m.call(0x00160918, &[p0, 0x25b880, 0x25c1b0], &[]).0;
    t1
}

/// FUN_001840b8 (PART r18 asm-mechanical port)
pub fn ee_sub_1840b8<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32, p3: u32) -> u32 {
    let t1 = m.call(0x0020fec0, &[0x3dd490, p0.wrapping_add(0x11c), 4, p0.wrapping_add(0x118), 1, p1, p2, p3], &[]).0;
    t1
}

/// FUN_001840f8 (PART r18 asm-mechanical port)
pub fn ee_sub_1840f8<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32) -> u32 {
    let t1 = m.call(0x00161f10, &[0x2deb30, p0.wrapping_add(0x160), 4, p0.wrapping_add(0x15c), p1, p2], &[]).0;
    t1
}

/// FUN_001aaed8 (PART r18 asm-mechanical port)
pub fn ee_sub_1aaed8<M: QMem + ?Sized>(m: &mut M, p0: u32) -> u32 {
    let t1 = m.call(0x0021efd0, &[1, 1], &[]).0;
    m.w32(p0.wrapping_add(0x10), 0x20);
    m.w32(p0, t1);
    m.w32(p0.wrapping_add(4), 0xffffffff);
    t1
}

/// FUN_001b0270 (PART r18 asm-mechanical port)
pub fn ee_sub_1b0270<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001af660, &[0x3d9ed0], &[]).0;
    t1
}

/// FUN_001b0668 (PART r18 asm-mechanical port)
pub fn ee_sub_1b0668<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001af490, &[0x3d9ed0], &[]).0;
    t1
}

/// FUN_001b0720 (PART r18 asm-mechanical port)
pub fn ee_sub_1b0720<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001af6a0, &[0x3d9ed0], &[]).0;
    t1
}

/// FUN_001b0970 (PART r18 asm-mechanical port)
pub fn ee_sub_1b0970<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x001619a0, &[0x2deb30, 0x2a, 0], &[]).0;
    let t2 = m.r32(0x3e0f9c);
    let t3 = m.call(0x00162e38, &[t2], &[]).0;
    t3
}

/// FUN_001b1140 (PART r18 asm-mechanical port)
pub fn ee_sub_1b1140<M: QMem + ?Sized>(m: &mut M, p0: u32) -> u32 {
    let t1 = m.call(0x001f3c70, &[p0], &[]).0;
    t1
}

/// FUN_001b8938 (PART r18 asm-mechanical port)
pub fn ee_sub_1b8938<M: QMem + ?Sized>(m: &mut M, p0: u32) -> u32 {
    let t1 = m.r32(p0.wrapping_add(0x1030));
    let t2 = m.call(0x00131b60, &[t1, p0, 5, 0x2c09f8], &[]).0;
    let t3 = m.r32(p0.wrapping_add(0x1030));
    m.w32(p0.wrapping_add(0x1030), 0);
    m.w32(p0.wrapping_add(0x190c), t3);
    let t4 = m.call(0x001b6078, &[p0, 0], &[]).0;
    t4
}

/// FUN_001bba60 (PART r18 asm-mechanical port)
pub fn ee_sub_1bba60<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32) -> u32 {
    let t1 = m.call(0x001bbdb0, &[p0, 6], &[]).0;
    m.w32(t1.wrapping_add(4), p1);
    t1
}

/// FUN_001bba90 (PART r18 asm-mechanical port)
pub fn ee_sub_1bba90<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32) -> u32 {
    let t1 = m.call(0x001bbdb0, &[p0, 7], &[]).0;
    m.w32(t1.wrapping_add(4), p1);
    t1
}

/// FUN_001bbac0 (PART r18 asm-mechanical port)
pub fn ee_sub_1bbac0<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32) -> u32 {
    let t1 = m.call(0x001bbdb0, &[p0, 8], &[]).0;
    m.w32(t1.wrapping_add(4), p1);
    t1
}

/// FUN_001bbaf0 (PART r18 asm-mechanical port)
pub fn ee_sub_1bbaf0<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32) -> u32 {
    let t1 = m.call(0x001bbdb0, &[p0, 9], &[]).0;
    m.w32(t1.wrapping_add(4), p1);
    t1
}

/// FUN_001bbb20 (PART r18 asm-mechanical port)
pub fn ee_sub_1bbb20<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32) -> u32 {
    let t1 = m.call(0x001bbdb0, &[p0, 0xa], &[]).0;
    m.w32(t1.wrapping_add(4), p1);
    t1
}

/// FUN_001bbb50 (PART r18 asm-mechanical port)
pub fn ee_sub_1bbb50<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32) -> u32 {
    let t1 = m.call(0x001bbdb0, &[p0, 0xb], &[]).0;
    m.w32(t1.wrapping_add(4), p1);
    t1
}

/// FUN_001bbb80 (PART r18 asm-mechanical port)
pub fn ee_sub_1bbb80<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32) -> u32 {
    let t1 = m.call(0x001bbdb0, &[p0, 0], &[]).0;
    m.w32(t1.wrapping_add(4), p1);
    t1
}

/// FUN_001becf8 (PART r18 asm-mechanical port)
pub fn ee_sub_1becf8<M: QMem + ?Sized>(m: &mut M, p0: u32) -> u32 {
    let t1 = m.call(0x001beb28, &[p0], &[]).0;
    t1
}

/// FUN_001c3df8 (PART r18 asm-mechanical port)
pub fn ee_sub_1c3df8<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32) -> u32 {
    let t1 = m.call(0x001f6068, &[p0, p1], &[]).0;
    m.w32(p0.wrapping_add(0x168), p0);
    t1
}

/// FUN_001d4de0 (PART r18 asm-mechanical port)
pub fn ee_sub_1d4de0<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32) -> u32 {
    let t1 = m.call(0x001ae5a0, &[p0.wrapping_add(0x14), p1.wrapping_add(0x14), 0x9c], &[]).0;
    t1
}

/// FUN_001dd8e0 (PART r18 asm-mechanical port)
pub fn ee_sub_1dd8e0<M: QMem + ?Sized>(m: &mut M, p0: u32) -> u32 {
    let t1 = m.r32(p0.wrapping_add(8));
    let t2 = m.call(0x001a2bd0, &[t1], &[]).0;
    let t3 = m.call(0x0022b490, &[0x3dd490, 0, 0x189, 0xffffffff, 1, 0xffffffff], &[]).0;
    t3
}

/// FUN_001dfe80 (PART r18 asm-mechanical port)
pub fn ee_sub_1dfe80<M: QMem + ?Sized>(m: &mut M, p0: u32) -> u32 {
    let t1 = m.call(0x001c4840, &[0x3dd490], &[]).0;
    let t2 = m.call(0x001dfd58, &[p0, t1], &[]).0;
    t2
}

/// FUN_001dffe0 (PART r18 asm-mechanical port)
pub fn ee_sub_1dffe0<M: QMem + ?Sized>(m: &mut M, p0: u32) -> u32 {
    let t1 = m.call(0x001dfeb8, &[p0], &[]).0;
    m.w32(p0.wrapping_add(0x50), 0);
    let t2 = m.call(0x001e0290, &[p0, 0xffffffff], &[]).0;
    m.w32(p0.wrapping_add(0x7c), 1);
    1
}

/// FUN_001e02e0 (PART r18 asm-mechanical port)
pub fn ee_sub_1e02e0<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 2, 1], &[]).0;
    m.w32(t1.wrapping_add(0x10), p1);
    t1
}

/// FUN_001e03a0 (PART r18 asm-mechanical port)
pub fn ee_sub_1e03a0<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 4, 0], &[]).0;
    m.w32(t1.wrapping_add(0x14), p2);
    m.w32(t1.wrapping_add(0x10), p1);
    t1
}

/// FUN_001e03e0 (PART r18 asm-mechanical port)
pub fn ee_sub_1e03e0<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 5, 1], &[]).0;
    m.w32(t1.wrapping_add(0x14), p2);
    m.w32(t1.wrapping_add(0x10), p1);
    t1
}

/// FUN_001e0420 (PART r18 asm-mechanical port)
pub fn ee_sub_1e0420<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 6, 1], &[]).0;
    m.w32(t1.wrapping_add(0x10), p1);
    t1
}

/// FUN_001e0450 (PART r18 asm-mechanical port)
pub fn ee_sub_1e0450<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 7, 1], &[]).0;
    m.w32(t1.wrapping_add(0x10), p1);
    t1
}

/// FUN_001e0480 (PART r18 asm-mechanical port)
pub fn ee_sub_1e0480<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 8, 1], &[]).0;
    m.w32(t1.wrapping_add(0x10), p1);
    t1
}

/// FUN_001e04b0 (PART r18 asm-mechanical port)
pub fn ee_sub_1e04b0<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 9, 1], &[]).0;
    m.w32(t1.wrapping_add(0x10), p1);
    t1
}

/// FUN_001e04e0 (PART r18 asm-mechanical port)
pub fn ee_sub_1e04e0<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32, p3: u32, p4: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0xa, p3], &[]).0;
    m.w32(t1.wrapping_add(0x18), p4);
    m.w32(t1.wrapping_add(0x10), p1);
    m.w32(t1.wrapping_add(0x14), p2);
    t1
}

/// FUN_001e0530 (PART r18 asm-mechanical port)
pub fn ee_sub_1e0530<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32, p3: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0xb, p3], &[]).0;
    m.w32(t1.wrapping_add(0x14), p2);
    m.w32(t1.wrapping_add(0x10), p1);
    t1
}

/// FUN_001e0570 (PART r18 asm-mechanical port)
pub fn ee_sub_1e0570<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0xc, 1], &[]).0;
    m.w32(t1.wrapping_add(0x10), p1);
    t1
}

/// FUN_001e0610 (PART r18 asm-mechanical port)
pub fn ee_sub_1e0610<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32, p3: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0xe, p3], &[]).0;
    m.w32(t1.wrapping_add(0x14), p2);
    m.w32(t1.wrapping_add(0x10), p1);
    t1
}

/// FUN_001e0650 (PART r18 asm-mechanical port)
pub fn ee_sub_1e0650<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0xf, 1], &[]).0;
    m.w32(t1.wrapping_add(0x10), p1);
    t1
}

/// FUN_001e06b0 (PART r18 asm-mechanical port)
pub fn ee_sub_1e06b0<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x11, 1], &[]).0;
    m.w32(t1.wrapping_add(0x14), p2);
    m.w32(t1.wrapping_add(0x10), p1);
    t1
}

/// FUN_001e06f0 (PART r18 asm-mechanical port)
pub fn ee_sub_1e06f0<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x12, 1], &[]).0;
    m.w32(t1.wrapping_add(0x14), p2);
    m.w32(t1.wrapping_add(0x10), p1);
    t1
}

/// FUN_001e0730 (PART r18 asm-mechanical port)
pub fn ee_sub_1e0730<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x13, 1], &[]).0;
    m.w32(t1.wrapping_add(0x10), p1);
    t1
}

/// FUN_001e0760 (PART r18 asm-mechanical port)
pub fn ee_sub_1e0760<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x14, 1], &[]).0;
    m.w32(t1.wrapping_add(0x10), p1);
    t1
}

/// FUN_001e0790 (PART r18 asm-mechanical port)
pub fn ee_sub_1e0790<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x15, 1], &[]).0;
    m.w32(t1.wrapping_add(0x14), p2);
    m.w32(t1.wrapping_add(0x10), p1);
    t1
}

/// FUN_001e07d0 (PART r18 asm-mechanical port)
pub fn ee_sub_1e07d0<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32, p3: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x16, 1], &[]).0;
    m.w32(t1.wrapping_add(0x18), p3);
    m.w32(t1.wrapping_add(0x10), p1);
    m.w32(t1.wrapping_add(0x14), p2);
    t1
}

/// FUN_001e0820 (PART r18 asm-mechanical port)
pub fn ee_sub_1e0820<M: QMem + ?Sized>(m: &mut M, p0: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x17, 1], &[]).0;
    t1
}

/// FUN_001e0840 (PART r18 asm-mechanical port)
pub fn ee_sub_1e0840<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x18, 1], &[]).0;
    m.w32(t1.wrapping_add(0x14), p2);
    m.w32(t1.wrapping_add(0x10), p1);
    t1
}

/// FUN_001e0880 (PART r18 asm-mechanical port)
pub fn ee_sub_1e0880<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x19, 1], &[]).0;
    m.w32(t1.wrapping_add(0x14), p2);
    m.w32(t1.wrapping_add(0x10), p1);
    t1
}

/// FUN_001e0910 (PART r18 asm-mechanical port)
pub fn ee_sub_1e0910<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x1b, 0], &[]).0;
    m.w32(t1.wrapping_add(0x10), p1);
    t1
}

/// FUN_001e0940 (PART r18 asm-mechanical port)
pub fn ee_sub_1e0940<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32, p3: u32, p4: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x1c, p3], &[]).0;
    m.w32(t1.wrapping_add(0x18), p4);
    m.w32(t1.wrapping_add(0x10), p1);
    m.w32(t1.wrapping_add(0x14), p2);
    t1
}

/// FUN_001e0990 (PART r18 asm-mechanical port)
pub fn ee_sub_1e0990<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x1d, 0], &[]).0;
    m.w32(t1.wrapping_add(0x14), p2);
    m.w32(t1.wrapping_add(0x10), p1);
    t1
}

/// FUN_001e09d0 (PART r18 asm-mechanical port)
pub fn ee_sub_1e09d0<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32, p3: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x1e, 0], &[]).0;
    m.w32(t1.wrapping_add(0x18), p3);
    m.w32(t1.wrapping_add(0x10), p1);
    m.w32(t1.wrapping_add(0x14), p2);
    t1
}

/// FUN_001e0a20 (PART r18 asm-mechanical port)
pub fn ee_sub_1e0a20<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x1f, 0], &[]).0;
    m.w32(t1.wrapping_add(0x14), p2);
    m.w32(t1.wrapping_add(0x10), p1);
    t1
}

/// FUN_001e0a60 (PART r18 asm-mechanical port)
pub fn ee_sub_1e0a60<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x20, 0], &[]).0;
    m.w32(t1.wrapping_add(0x14), p2);
    m.w32(t1.wrapping_add(0x10), p1);
    t1
}

/// FUN_001e0aa0 (PART r18 asm-mechanical port)
pub fn ee_sub_1e0aa0<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x21, 0], &[]).0;
    m.w32(t1.wrapping_add(0x14), p2);
    m.w32(t1.wrapping_add(0x10), p1);
    t1
}

/// FUN_001e0ae0 (PART r18 asm-mechanical port)
pub fn ee_sub_1e0ae0<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32, p3: u32, p4: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x22, 0], &[]).0;
    m.w32(t1.wrapping_add(0x1c), p4);
    m.w32(t1.wrapping_add(0x10), p1);
    m.w32(t1.wrapping_add(0x14), p2);
    m.w32(t1.wrapping_add(0x18), p3);
    t1
}

/// FUN_001e0c78 (PART r18 asm-mechanical port)
pub fn ee_sub_1e0c78<M: QMem + ?Sized>(m: &mut M, p0: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x25, 1], &[]).0;
    t1
}

/// FUN_001e0c98 (PART r18 asm-mechanical port)
pub fn ee_sub_1e0c98<M: QMem + ?Sized>(m: &mut M, p0: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x26, 1], &[]).0;
    t1
}

/// FUN_001e0e18 (PART r18 asm-mechanical port)
pub fn ee_sub_1e0e18<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32, p3: u32, p4: u32, p5: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x2a, p4], &[]).0;
    m.w32(t1.wrapping_add(0x1c), p5);
    m.w32(t1.wrapping_add(0x10), p1);
    m.w32(t1.wrapping_add(0x14), p2);
    m.w32(t1.wrapping_add(0x18), p3);
    t1
}

/// FUN_001e0f58 (PART r18 asm-mechanical port)
pub fn ee_sub_1e0f58<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x2e, 1], &[]).0;
    m.w32(t1.wrapping_add(0x14), p2);
    m.w32(t1.wrapping_add(0x10), p1);
    t1
}

/// FUN_001e0f98 (PART r18 asm-mechanical port)
pub fn ee_sub_1e0f98<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x2f, 1], &[]).0;
    m.w32(t1.wrapping_add(0x14), p2);
    m.w32(t1.wrapping_add(0x10), p1);
    t1
}

/// FUN_001e0fd8 (PART r18 asm-mechanical port)
pub fn ee_sub_1e0fd8<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x30, 1], &[]).0;
    m.w32(t1.wrapping_add(0x14), p2);
    m.w32(t1.wrapping_add(0x10), p1);
    t1
}

/// FUN_001e1018 (PART r18 asm-mechanical port)
pub fn ee_sub_1e1018<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x31, 1], &[]).0;
    m.w32(t1.wrapping_add(0x14), p2);
    m.w32(t1.wrapping_add(0x10), p1);
    t1
}

/// FUN_001e1058 (PART r18 asm-mechanical port)
pub fn ee_sub_1e1058<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x32, p2], &[]).0;
    m.w32(t1.wrapping_add(0x10), p1);
    t1
}

/// FUN_001e1208 (PART r18 asm-mechanical port)
pub fn ee_sub_1e1208<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x37, 1], &[]).0;
    m.w32(t1.wrapping_add(0x10), p1);
    t1
}

/// FUN_001e1388 (PART r18 asm-mechanical port)
pub fn ee_sub_1e1388<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x3a, 1], &[]).0;
    m.w32(t1.wrapping_add(0x10), p1);
    t1
}

/// FUN_001e13b8 (PART r18 asm-mechanical port)
pub fn ee_sub_1e13b8<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32, p3: u32, p4: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x3b, p4], &[]).0;
    m.w32(t1.wrapping_add(0x18), p3);
    m.w32(t1.wrapping_add(0x10), p1);
    m.w32(t1.wrapping_add(0x14), p2);
    t1
}

/// FUN_001e1408 (PART r18 asm-mechanical port)
pub fn ee_sub_1e1408<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x3c, p1], &[]).0;
    t1
}

/// FUN_001e1428 (PART r18 asm-mechanical port)
pub fn ee_sub_1e1428<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x3d, p2], &[]).0;
    m.w32(t1.wrapping_add(0x10), p1);
    t1
}

/// FUN_001e1458 (PART r18 asm-mechanical port)
pub fn ee_sub_1e1458<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x3e, p2], &[]).0;
    m.w32(t1.wrapping_add(0x10), p1);
    t1
}

/// FUN_001e1540 (PART r18 asm-mechanical port)
pub fn ee_sub_1e1540<M: QMem + ?Sized>(m: &mut M, p0: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x41, 1], &[]).0;
    t1
}

/// FUN_001e15b0 (PART r18 asm-mechanical port)
pub fn ee_sub_1e15b0<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x43, 0], &[]).0;
    m.w32(t1.wrapping_add(0x14), p2);
    m.w32(t1.wrapping_add(0x10), p1);
    t1
}

/// FUN_001e15f0 (PART r18 asm-mechanical port)
pub fn ee_sub_1e15f0<M: QMem + ?Sized>(m: &mut M, p0: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x44, 0], &[]).0;
    t1
}

/// FUN_001e1610 (PART r18 asm-mechanical port)
pub fn ee_sub_1e1610<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32, p3: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x45, p3], &[]).0;
    m.w32(t1.wrapping_add(0x14), p2);
    m.w32(t1.wrapping_add(0x10), p1);
    t1
}

/// FUN_001e1650 (PART r18 asm-mechanical port)
pub fn ee_sub_1e1650<M: QMem + ?Sized>(m: &mut M, p0: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x46, 1], &[]).0;
    t1
}

/// FUN_001e1670 (PART r18 asm-mechanical port)
pub fn ee_sub_1e1670<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32, p3: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x47, 1], &[]).0;
    m.w32(t1.wrapping_add(0x18), p3);
    m.w32(t1.wrapping_add(0x10), p1);
    m.w32(t1.wrapping_add(0x14), p2);
    t1
}

/// FUN_001e16c0 (PART r18 asm-mechanical port)
pub fn ee_sub_1e16c0<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32, p3: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x48, 1], &[]).0;
    m.w32(t1.wrapping_add(0x18), p3);
    m.w32(t1.wrapping_add(0x10), p1);
    m.w32(t1.wrapping_add(0x14), p2);
    t1
}

/// FUN_001e1710 (PART r18 asm-mechanical port)
pub fn ee_sub_1e1710<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x49, 1], &[]).0;
    m.w32(t1.wrapping_add(0x10), p1);
    t1
}

/// FUN_001e17a0 (PART r18 asm-mechanical port)
pub fn ee_sub_1e17a0<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x4b, 1], &[]).0;
    m.w32(t1.wrapping_add(0x14), p2);
    m.w32(t1.wrapping_add(0x10), p1);
    t1
}

/// FUN_001e17e0 (PART r18 asm-mechanical port)
pub fn ee_sub_1e17e0<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x4c, 1], &[]).0;
    m.w32(t1.wrapping_add(0x10), p1);
    t1
}

/// FUN_001e1810 (PART r18 asm-mechanical port)
pub fn ee_sub_1e1810<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x4d, 1], &[]).0;
    m.w32(t1.wrapping_add(0x10), p1);
    t1
}

/// FUN_001e1870 (PART r18 asm-mechanical port)
pub fn ee_sub_1e1870<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32, p3: u32, p4: u32, p5: u32, p6: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x4f, p6], &[]).0;
    m.w32(t1.wrapping_add(0x10), p5);
    m.w32(t1.wrapping_add(0x14), p1);
    m.w32(t1.wrapping_add(0x1c), p2);
    m.w32(t1.wrapping_add(0x18), p3);
    m.w32(t1.wrapping_add(0x20), p4);
    t1
}

/// FUN_001e18e0 (PART r18 asm-mechanical port)
pub fn ee_sub_1e18e0<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x50, 0], &[]).0;
    m.w32(t1.wrapping_add(0x10), p1);
    t1
}

/// FUN_001e1910 (PART r18 asm-mechanical port)
pub fn ee_sub_1e1910<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x51, 0], &[]).0;
    m.w32(t1.wrapping_add(0x10), p1);
    t1
}

/// FUN_001e19b0 (PART r18 asm-mechanical port)
pub fn ee_sub_1e19b0<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x54, 1], &[]).0;
    m.w32(t1.wrapping_add(0x10), p1);
    t1
}

/// FUN_001e19e0 (PART r18 asm-mechanical port)
pub fn ee_sub_1e19e0<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32, p3: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x55, p3], &[]).0;
    m.w32(t1.wrapping_add(0x14), p2);
    m.w32(t1.wrapping_add(0x10), p1);
    t1
}

/// FUN_001e1a20 (PART r18 asm-mechanical port)
pub fn ee_sub_1e1a20<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x56, 1], &[]).0;
    m.w32(t1.wrapping_add(0x14), p2);
    m.w32(t1.wrapping_add(0x10), p1);
    t1
}

/// FUN_001e1a60 (PART r18 asm-mechanical port)
pub fn ee_sub_1e1a60<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x57, 1], &[]).0;
    m.w32(t1.wrapping_add(0x10), p1);
    t1
}

/// FUN_001e1a90 (PART r18 asm-mechanical port)
pub fn ee_sub_1e1a90<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32, p3: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x58, p3], &[]).0;
    m.w32(t1.wrapping_add(0x14), p2);
    m.w32(t1.wrapping_add(0x10), p1);
    t1
}

/// FUN_001e1ad0 (PART r18 asm-mechanical port)
pub fn ee_sub_1e1ad0<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32, p3: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x59, p3], &[]).0;
    m.w32(t1.wrapping_add(0x14), p2);
    m.w32(t1.wrapping_add(0x10), p1);
    t1
}

/// FUN_001e1b10 (PART r18 asm-mechanical port)
pub fn ee_sub_1e1b10<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32, p3: u32) -> u32 {
    let t1 = m.call(0x001e01f8, &[p0, 0x5a, p3], &[]).0;
    m.w32(t1.wrapping_add(0x10), p1);
    m.w32(t1.wrapping_add(0x14), p2);
    m.w32(t1.wrapping_add(0x18), 0);
    t1
}

/// FUN_00203a08 (PART r18 asm-mechanical port)
pub fn ee_sub_203a08<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32) -> u32 {
    let t1 = m.call(0x001c3df8, &[p0, p1], &[]).0;
    let t2 = m.call(0x001eec68, &[p0.wrapping_add(0x26c), p0], &[]).0;
    t2
}

/// FUN_002088e0 (PART r18 asm-mechanical port)
pub fn ee_sub_2088e0<M: QMem + ?Sized>(m: &mut M, p0: u32) -> u32 {
    let t1 = m.r32(p0.wrapping_add(0xf64));
    m.w32(t1.wrapping_add(0xfc), 0);
    m.w32(p0.wrapping_add(0x394), 0);
    m.w32(p0.wrapping_add(0x3a4), 0);
    let t2 = m.call(0x002089e8, &[p0], &[]).0;
    let t3 = m.call(0x00208b30, &[p0], &[]).0;
    t3
}

/// FUN_002089e8 (PART r18 asm-mechanical port)
pub fn ee_sub_2089e8<M: QMem + ?Sized>(m: &mut M, p0: u32) {
    m.w32(p0.wrapping_add(0x3a8), 0);
}

/// FUN_00208b30 (PART r18 asm-mechanical port)
pub fn ee_sub_208b30<M: QMem + ?Sized>(m: &mut M, p0: u32) {
    m.w32(p0.wrapping_add(0x3c0), 0);
}

/// FUN_0020aa78 (PART r18 asm-mechanical port)
pub fn ee_sub_20aa78<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32) -> u32 {
    let t1 = m.call(0x0020aa40, &[p0, p1], &[]).0;
    let t2 = m.call(0x0020aaa8, &[p0], &[]).0;
    t2
}

/// FUN_0020ab58 (PART r18 asm-mechanical port)
pub fn ee_sub_20ab58<M: QMem + ?Sized>(m: &mut M, p0: u32) {
    m.w32(p0.wrapping_add(0xefc), 0);
    m.w32(p0.wrapping_add(0xed8), 0);
}

/// FUN_0020ade8 (PART r18 asm-mechanical port)
pub fn ee_sub_20ade8<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32) -> u32 {
    let t1 = m.call(0x001a6730, &[p0, p1], &[]).0;
    let t2 = m.call(0x00209590, &[p0.wrapping_add(0xb0)], &[]).0;
    t2
}

/// FUN_0020af28 (PART r18 asm-mechanical port)
pub fn ee_sub_20af28<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32, p3: u32) {
    m.w32(p0.wrapping_add(0x84), p3);
    m.w32(p0.wrapping_add(0x7c), p1);
    m.w32(p0.wrapping_add(0x80), p2);
}

/// FUN_0020af88 (PART r18 asm-mechanical port)
pub fn ee_sub_20af88<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32) -> u32 {
    let t1 = m.r32(p0.wrapping_add(0xac));
    let t2 = m.call(0x002090d0, &[p0.wrapping_add(0xb0)], &[]).0;
    m.w32(p0.wrapping_add(0xb4), p1);
    m.w32(p0.wrapping_add(0xbc), p2);
    let t3 = m.r32(p0.wrapping_add(0xac));
    let t4 = m.call(0x00209130, &[p0.wrapping_add(0xb0), t3], &[]).0;
    t4
}

/// FUN_00210b70 (PART r18 asm-mechanical port)
pub fn ee_sub_210b70<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let t1 = m.call(0x00210ae0, &[], &[]).0;
    let t2 = m.call(0x00210b98, &[t1], &[]).0;
    t2
}

/// FUN_002113e0 (PART r18 asm-mechanical port)
pub fn ee_sub_2113e0<M: QMem + ?Sized>(m: &mut M, p0: u32) -> u32 {
    let t1 = m.r32(0x3da0d8);
    let t2 = m.call(0x0013e368, &[0x3dd490, t1], &[]).0;
    let t3 = m.call(0x0021ed58, &[p0, t2], &[]).0;
    t3
}

/// FUN_00221140 (PART r18 asm-mechanical port)
pub fn ee_sub_221140<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32) -> u32 {
    let t1 = m.call(0x002311b0, &[p0, p1], &[]).0;
    let t2 = m.r32(p0.wrapping_add(0x10c));
    let t3 = m.r32(p0.wrapping_add(0x10));
    let t4 = m.call(0x0013e510, &[t2, t3], &[]).0;
    m.w32(p0.wrapping_add(0x10c), t4);
    t4
}

/// FUN_00228920 (PART r18 asm-mechanical port)
pub fn ee_sub_228920<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32) -> u32 {
    let t1 = m.call(0x00161f10, &[0x2deb30, p0.wrapping_add(0x128), 4, p0.wrapping_add(0x124), p1, p2], &[]).0;
    t1
}

/// FUN_0023fdd8 (PART r18 asm-mechanical port)
pub fn ee_sub_23fdd8<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32) -> u32 {
    let t1 = m.r32(0x450278);
    m.w32(p1.wrapping_add(0xc), p0);
    m.w32(p1.wrapping_add(0x14), t1);
    m.w32(0x450278, p1);
    m.w32(p1.wrapping_add(8), p0);
    m.w32(p1.wrapping_add(4), 0);
    m.w32(p1, 0);
    m.w32(p1.wrapping_add(0x10), 0);
    t1
}

/// FUN_002405f0 (PART r18 asm-mechanical port)
pub fn ee_sub_2405f0<M: QMem + ?Sized>(m: &mut M) -> u32 {

    0x2c66f0
}

/// FUN_0024b530 (PART r18 asm-mechanical port)
pub fn ee_sub_24b530<M: QMem + ?Sized>(m: &mut M) -> u32 {

    1
}

/// FUN_0024b610 (PART r18 asm-mechanical port)
pub fn ee_sub_24b610<M: QMem + ?Sized>(m: &mut M, p0: u32) -> u32 {
    let t1 = m.r32(p0.wrapping_add(0x40));
    let t2 = m.r32(t1);
    t2
}

/// FUN_0024b8f8 (PART r18 asm-mechanical port)
pub fn ee_sub_24b8f8<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32, p2: u32) {
    m.w32(p0.wrapping_add(0xc), p1);
    m.w32(p0.wrapping_add(4), p2);
    m.w32(p0, p1);
    m.w32(p0.wrapping_add(8), p1);
}

/// FUN_0024b910 (PART r18 asm-mechanical port)
pub fn ee_sub_24b910<M: QMem + ?Sized>(m: &mut M, p0: u32) -> u32 {
    let t1 = m.r32(p0.wrapping_add(8));
    m.w32(p0.wrapping_add(0xc), t1);
    t1
}

/// FUN_0024b920 (PART r18 asm-mechanical port)
pub fn ee_sub_24b920<M: QMem + ?Sized>(m: &mut M, p0: u32) -> u32 {
    let t1 = m.r32(p0.wrapping_add(0xc));
    m.w32(p0.wrapping_add(8), t1);
    t1
}

/// FUN_00256360 (PART r18 asm-mechanical port)
pub fn ee_sub_256360<M: QMem + ?Sized>(m: &mut M, p0: u32) -> u32 {
    m.w32(p0, 0);
    m.w32(p0.wrapping_add(4), 0);
    m.w32(p0.wrapping_add(0xc), 0x2d9978);
    let t1 = m.call(0x00167310, &[p0.wrapping_add(0x80)], &[]).0;
    m.w32(p0.wrapping_add(0x174), 0);
    m.w32(p0.wrapping_add(0xc), 0x2da568);
    let t2 = m.call(0x0020c058, &[p0.wrapping_add(0xdb0)], &[]).0;
    m.w32(p0.wrapping_add(0xe20).wrapping_add(0xc), 0x2cb1f0);
    m.w32(p0.wrapping_add(0xe20), 0);
    m.w32(p0.wrapping_add(0xe24), 0);
    m.w32(p0.wrapping_add(0xe20).wrapping_add(0xc), 0x2da9d0);
    p0
}

/// FUN_002563e8 (PART r18 asm-mechanical port)
pub fn ee_sub_2563e8<M: QMem + ?Sized>(m: &mut M, p0: u32) -> u32 {
    m.w32(p0, 0);
    m.w32(p0.wrapping_add(4), 0);
    m.w32(p0.wrapping_add(0xc), 0x2d9978);
    let t1 = m.call(0x00167310, &[p0.wrapping_add(0x80)], &[]).0;
    m.w32(p0.wrapping_add(0x174), 0);
    m.w32(p0.wrapping_add(0xc), 0x2d2e98);
    m.w32(p0.wrapping_add(0x2044), 0);
    m.w32(p0.wrapping_add(0x2048), 0);
    p0
}

/// FUN_00256440 (PART r18 asm-mechanical port)
pub fn ee_sub_256440<M: QMem + ?Sized>(m: &mut M, p0: u32) -> u32 {
    m.w32(p0, 0);
    m.w32(p0.wrapping_add(4), 0);
    m.w32(p0.wrapping_add(0xc), 0x2d9978);
    let t1 = m.call(0x00167310, &[p0.wrapping_add(0x80)], &[]).0;
    m.w32(p0.wrapping_add(0x174), 0);
    m.w32(p0.wrapping_add(0xc), 0x2d44c8);
    let t2 = m.call(0x0020c058, &[p0.wrapping_add(0x1890)], &[]).0;
    p0
}

/// FUN_00258620 (PART r18 asm-mechanical port)
pub fn ee_sub_258620<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32) -> u32 {
    let t1 = m.call(0x002311b0, &[p0, p1], &[]).0;
    let t2 = m.r32(p0.wrapping_add(0xe4));
    let t3 = m.r32(t2);
    m.w32(t2, t3.wrapping_add(1));
    t3.wrapping_add(1)
}

/// FUN_002590b8 (PART r18 asm-mechanical port). Verifies 1:1 but is NOT wired into live_dispatch: wiring it makes wired_guards::crawl_cellar_guard_hits_sly_live fail (r18 bisect)
pub fn ee_sub_2590b8<M: QMem + ?Sized>(m: &mut M, p0: u32, p1: u32) -> u32 {
    let t1 = m.call(0x002311b0, &[p0, p1], &[]).0;
    let t2 = m.r32(p0.wrapping_add(0xe4));
    let t3 = m.r32(t2);
    m.w32(t2, t3.wrapping_add(1));
    t3.wrapping_add(1)
}
