//! WIRE-W1H9: 1:1 queue ports (against `QMem`) of a batch of EE functions, ported from the asm;
//! callees go through `m.call` / `vslot`. QMem convention: integer / pointer args in `a`, quadword and float
//! args in `v`.
#![allow(unused_variables, unused_mut, unused_assignments, clippy::all)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

const GS: u32 = 0x003d_d490; // game state / world

/// FUN_0014fd40: empty.
pub fn ee_sub_14fd40<M: QMem + ?Sized>(m: &mut M) {}

/// FUN_001fc9a8: empty.
pub fn ee_sub_1fc9a8<M: QMem + ?Sized>(m: &mut M) {}

/// FUN_0015c310(o, out): *out = o+0xe4; return o+0xe0.
pub fn ee_sub_15c310<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let v = m.r32(a0 + 0xe4);
    m.w32(a1, v);
    m.r32(a0 + 0xe0)
}

/// FUN_002252e0(p): FUN_00223f70(p).
pub fn ee_sub_2252e0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0022_3f70, &[a0], &[]);
}

/// FUN_00228bf8(p): FUN_001f36a8(p).
pub fn ee_sub_228bf8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001f_36a8, &[a0], &[]);
}

/// FUN_001fe5b8(o, mask, set): set/clear bits in +0x68.
pub fn ee_sub_1fe5b8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    let v = m.r32(a0 + 0x68);
    if a2 != 0 {
        m.w32(a0 + 0x68, v | a1);
    } else {
        m.w32(a0 + 0x68, v & !a1);
    }
}

/// FUN_0017f170(o, x): FUN_001cb518(o, x); +0x184 = +0x180.
pub fn ee_sub_17f170<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.call(0x001c_b518, &[a0, a1], &[]);
    let v = m.r32(a0 + 0x180);
    m.w32(a0 + 0x184, v);
}

/// FUN_001cbeb0(o): FUN_00205eb8(o); +0x4a4 = 1.
pub fn ee_sub_1cbeb0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0020_5eb8, &[a0], &[]);
    m.w32(a0 + 0x4a4, 1);
}

/// FUN_00170a28(o): FUN_001a5c48(o); FUN_00170a58(o).
pub fn ee_sub_170a28<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5c48, &[a0], &[]);
    m.call(0x0017_0a58, &[a0], &[]);
}

/// FUN_00180598(o): FUN_0017f110(o); FUN_00181268(o, -1).
pub fn ee_sub_180598<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0017_f110, &[a0], &[]);
    m.call(0x0018_1268, &[a0, 0xffff_ffff], &[]);
}

/// FUN_0013bc60(o, x): if +0x200 != x { +0x200 = x; FUN_00134320(GS, o) }.
pub fn ee_sub_13bc60<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if a1 != m.r32(a0 + 0x200) {
        m.w32(a0 + 0x200, a1);
        m.call(0x0013_4320, &[GS, a0], &[]);
    }
}

/// FUN_001b21f0(o): FUN_001b1db8(o); +0x274 = 1500.0.
pub fn ee_sub_1b21f0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001b_1db8, &[a0], &[]);
    m.wf(a0 + 0x274, f32::from_bits(0x44bb_8000));
}

/// FUN_001844a0(o): FUN_001a6178(o); FUN_0011a220(0x3e1438, o).
pub fn ee_sub_1844a0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_6178, &[a0], &[]);
    m.call(0x0011_a220, &[0x003e_1438, a0], &[]);
}

/// FUN_00185c28(o): FUN_001a5800(o); FUN_0011a220(0x3e142c, o).
pub fn ee_sub_185c28<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5800, &[a0], &[]);
    m.call(0x0011_a220, &[0x003e_142c, a0], &[]);
}

/// FUN_0017a6f8(o): FUN_001eef68(o); FUN_00209890(o+0x10, 0x16); +0x30 = -1.
pub fn ee_sub_17a6f8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001e_ef68, &[a0], &[]);
    let p = m.r32(a0 + 0x10);
    m.call(0x0020_9890, &[p, 0x16], &[]);
    m.w32(a0 + 0x30, 0xffff_ffff);
}

/// FUN_001c2078(o): FUN_001a5800(o); ++GS+0x3d74; (GS+0x45cc)+0x28 = it.
pub fn ee_sub_1c2078<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5800, &[a0], &[]);
    let v = m.r32(GS + 0x3d74).wrapping_add(1);
    let p = m.r32(GS + 0x45cc);
    m.w32(GS + 0x3d74, v);
    m.w32(p + 0x28, v);
}

/// FUN_001f78d8(o, mask): OR of the bytes o+0x110+i (i < 26) whose bit i is set in mask.
pub fn ee_sub_1f78d8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let base = a0.wrapping_add(0x110);
    let mut t0 = 0u32;
    let mut a2 = 0u32;
    let mut a3 = 1u32;
    loop {
        if a3 & a1 != 0 {
            t0 |= m.r8(base.wrapping_add(a2)) as u32;
        }
        a2 = a2.wrapping_add(1);
        a3 = a3 << 1;
        if (a2 as i32) >= 0x1a {
            break;
        }
    }
    t0
}

/// FUN_0013cef8(o): FUN_00183f50(o); +0x180 = GS+0x463c++.
pub fn ee_sub_13cef8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0018_3f50, &[a0], &[]);
    let v = m.r32(GS + 0x463c);
    m.w32(a0 + 0x180, v);
    m.w32(GS + 0x463c, v.wrapping_add(1));
}

/// FUN_0012e600(o): FUN_00231100(o); +0x100 = q[0x2a2da0]; +0x110 = q[0x2a2dc0].
pub fn ee_sub_12e600<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0023_1100, &[a0], &[]);
    let v = m.rv(0x002a_2da0);
    m.wv(a0 + 0x100, v);
    let v = m.rv(0x002a_2dc0);
    m.wv(a0 + 0x110, v);
}

/// FUN_001b1db8(o): FUN_0013a9e8(o); +0x264 = 1; +0x26c = 1.0; +0x260 = 800.0.
pub fn ee_sub_1b1db8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0013_a9e8, &[a0], &[]);
    m.w32(a0 + 0x264, 1);
    m.wf(a0 + 0x26c, 1.0);
    m.wf(a0 + 0x260, f32::from_bits(0x4448_0000));
}

/// FUN_00194640(p): p+8 == 8 -> 1 (ret) ; p+8 == 0xc -> (p+0xc - 9) < 2 ; else 0.
pub fn ee_sub_194640<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let v1 = m.r32(a0 + 8);
    if v1 == 8 {
        return 1;
    }
    if v1 == 0xc {
        let v = m.r32(a0 + 0xc).wrapping_sub(9);
        if v < 2 {
            return 1;
        }
        return 0;
    }
    0
}

/// FUN_0012e6e8(o): FUN_001a5848(o); if +0x124: FUN_0014e4c8(0x2ddf40, 0xf, 0xa, 0x2de3e8, 0, o).
pub fn ee_sub_12e6e8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5848, &[a0], &[]);
    if m.r32(a0 + 0x124) != 0 {
        m.call(0x0014_e4c8, &[0x002d_df40, 0xf, 0xa, 0x002d_e3e8, 0, a0], &[]);
    }
}

/// FUN_001bab50(o): FUN_001f2108(o); FUN_001f7918(o, 0x3ffffff, 8); FUN_001f7918(o, 0x80001, 0).
pub fn ee_sub_1bab50<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001f_2108, &[a0], &[]);
    m.call(0x001f_7918, &[a0, 0x03ff_ffff, 8], &[]);
    m.call(0x001f_7918, &[a0, 0x0008_0001, 0], &[]);
}

/// FUN_001c3390(o): FUN_001e5d80(o); FUN_00182c70(o+0x190); +0x1ac=1; +0x1c0=q[0x2a2da0]; +0x1d4=9; +0x1d0=8.
pub fn ee_sub_1c3390<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001e_5d80, &[a0], &[]);
    m.call(0x0018_2c70, &[a0 + 0x190], &[]);
    let q = m.rv(0x002a_2da0);
    m.w32(a0 + 0x1ac, 1);
    m.wv(a0 + 0x1c0, q);
    m.w32(a0 + 0x1d4, 9);
    m.w32(a0 + 0x1d0, 8);
}

/// FUN_0014f5a0(o, f): changed = |f - o+0x20| > 1e-4; +0x24 = +0x20 = f; +0x28 = 0; if changed FUN_0014db70(o).
pub fn ee_sub_14f5a0<M: QMem + ?Sized>(m: &mut M, a0: u32, f12: f32) {
    let eps = f32::from_bits(0x38d1_b717);
    let d = (f12 - m.rf(a0 + 0x20)).abs();
    let changed = eps < d;
    m.wf(a0 + 0x24, f12);
    m.wf(a0 + 0x20, f12);
    m.w32(a0 + 0x28, 0);
    if changed {
        m.call(0x0014_db70, &[a0], &[f4(f12)]);
    }
}

/// FUN_0021d2c8(o): FUN_001a5c48(o); if +0x38 == 1e10 { +0x38 = +0xec + [0x2c40c0] }.
pub fn ee_sub_21d2c8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5c48, &[a0], &[]);
    if m.rf(a0 + 0x38) == f32::from_bits(0x5015_02f9) {
        let f = m.rf(a0 + 0xec) + m.rf(0x002c_40c0);
        m.wf(a0 + 0x38, f);
    }
}

/// FUN_001a23f8(o, out): walk the fallback chain (table 0x2bfb28) from index o+4 to the first non-null
/// o+0xd4[idx]; *out = 0; return it (0 if none).
pub fn ee_sub_1a23f8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let mut a2 = m.r32(a0 + 4);
    let mut v1 = 0u32;
    if a2 != 0xffff_ffff {
        let base = a0.wrapping_add(0xd4);
        let mut off = a2 << 2;
        v1 = m.r32(base.wrapping_add(off));
        if v1 == 0 {
            loop {
                a2 = m.r32(off.wrapping_add(0x002b_fb28));
                if a2 == 0xffff_ffff {
                    break;
                }
                off = a2 << 2;
                v1 = m.r32(base.wrapping_add(off));
                if v1 != 0 {
                    break;
                }
            }
        }
    }
    m.w32(a1, 0);
    v1
}

/// FUN_00183768(o): FUN_001a5c48(o); FUN_00221268(+0xe0, 0) if set; +0x38 == 1e10 -> [0x2be8c8].
pub fn ee_sub_183768<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5c48, &[a0], &[]);
    let p = m.r32(a0 + 0xe0);
    if p != 0 {
        m.call(0x0022_1268, &[p, 0], &[]);
    }
    if m.rf(a0 + 0x38) == f32::from_bits(0x5015_02f9) {
        let f = m.rf(0x002b_e8c8);
        m.wf(a0 + 0x38, f);
    }
}

/// FUN_0015dcd8(o): FUN_001a5e28(o); if +0x5c + +0x60 + +0x64 < [0x2dded0] - +0x68: vslot 8.
pub fn ee_sub_15dcd8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5e28, &[a0], &[]);
    let f2 = m.rf(a0 + 0x60);
    let f0 = m.rf(a0 + 0x5c);
    let f1 = m.rf(0x002d_ded0);
    let f0 = f0 + f2;
    let f3 = m.rf(a0 + 0x68);
    let f2 = m.rf(a0 + 0x64);
    let f1 = f1 - f3;
    let f0 = f0 + f2;
    if f0 < f1 {
        vslot(m, a0, 8, &[], &[]);
    }
}

/// FUN_0011d2d0(a0, a1, f12, f13, f14): FUN_0011cf60(f12 + FUN_0011d018(0, FUN_0011cf60(f13 - f12), f14, a0, a1)).
pub fn ee_sub_11d2d0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, f12: f32, f13: f32, f14: f32) -> f32 {
    let f0 = m.call(0x0011_cf60, &[], &[f4(f13 - f12)]).1[0];
    let f0 = m.call(0x0011_d018, &[a0, a1], &[f4(0.0), f4(f0), f4(f14)]).1[0];
    m.call(0x0011_cf60, &[], &[f4(f12 + f0)]).1[0]
}

/// FUN_0013bb80(o, x): FUN_001cb518(o, x); reset fields; FUN_0013bc60(o, o); FUN_001f5720/57c0/6860(o, 0);
/// FUN_001a58d0(o, 0, 0xf).
pub fn ee_sub_13bb80<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.call(0x001c_b518, &[a0, a1], &[]);
    m.w32(a0 + 0x174, 0);
    let f = m.rf(0x002a_4b20);
    m.wf(a0 + 0x154, f);
    m.call(0x0013_bc60, &[a0, a0], &[]);
    m.call(0x001f_5720, &[a0, 0], &[]);
    m.call(0x001f_57c0, &[a0, 0], &[]);
    m.call(0x001f_6860, &[a0, 0], &[]);
    m.w32(a0 + 0x1e8, 0);
    m.call(0x001a_58d0, &[a0, 0, 0xf], &[]);
}

/// FUN_001fc018(o, x): x == 0 -> vslot 6; else FUN_001fb658(o, vslot7(o) == 2 ? 0x1ff : 0x3ff, 8).
pub fn ee_sub_1fc018<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if a1 == 0 {
        vslot(m, a0, 6, &[], &[]);
        return;
    }
    let v = vslot(m, a0, 7, &[], &[]).0;
    let a1 = if v ^ 2 == 0 { 0x1ff } else { 0x3ff };
    m.call(0x001f_b658, &[a0, a1, 8], &[]);
}

/// FUN_001c6508(o): FUN_001c4e50(o); tuning constants.
pub fn ee_sub_1c6508<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001c_4e50, &[a0], &[]);
    let f5 = f32::from_bits(0x3e80_0000);
    let f6 = f32::from_bits(0x40a0_0000);
    m.wf(a0 + 0x234, f6);
    m.wf(a0 + 0x1d8, f32::from_bits(0x4234_0000));
    m.wf(a0 + 0x1dc, f32::from_bits(0x4282_0000));
    m.wf(a0 + 0x1d0, f32::from_bits(0x437a_0000));
    m.wf(a0 + 0x1d4, f32::from_bits(0x4316_0000));
    m.wf(a0 + 0x1e0, f32::from_bits(0x3f00_0000));
    m.wf(a0 + 0x1a0, f5);
    m.wf(a0 + 0x1e4, f5);
    m.wf(a0 + 0x230, f6);
}

/// FUN_001c7648(o): FUN_001c4e50(o); tuning constants.
pub fn ee_sub_1c7648<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001c_4e50, &[a0], &[]);
    m.wf(a0 + 0x1d8, f32::from_bits(0x420c_0000));
    m.wf(a0 + 0x234, f32::from_bits(0x4120_0000));
    m.wf(a0 + 0x1dc, f32::from_bits(0x4220_0000));
    m.wf(a0 + 0x1d0, f32::from_bits(0x437a_0000));
    m.wf(a0 + 0x1d4, f32::from_bits(0x4248_0000));
    m.wf(a0 + 0x1e4, f32::from_bits(0x3f19_999a));
    m.wf(a0 + 0x1e0, f32::from_bits(0x3f40_0000));
    m.wf(a0 + 0x230, f32::from_bits(0x4100_0000));
}

/// FUN_002259d8(o): FUN_001c3d70(o); +0x1ac..+0x1b4 = 8 bytes @0x2c4400; tuning constants.
pub fn ee_sub_2259d8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001c_3d70, &[a0], &[]);
    let lo = m.r32(0x002c_4400);
    let hi = m.r32(0x002c_4404);
    m.w32(a0 + 0x1ac, lo);
    m.w32(a0 + 0x1b0, hi);
    let f0 = m.rf(a0 + 0x1b0);
    m.wf(a0 + 0x1bc, f0);
    m.wf(a0 + 0x1a4, f32::from_bits(0xbfc0_0000));
    m.wf(a0 + 0x1a8, f32::from_bits(0x3fc0_0000));
    m.wf(a0 + 0x19c, f32::from_bits(0xbf33_3333));
    m.wf(a0 + 0x1a0, f32::from_bits(0x3f33_3333));
    m.w32(a0 + 0x1d4, 1);
    m.w32(a0 + 0x1d0, a0);
    m.w32(a0 + 0x1cc, 1);
}

/// FUN_00229ef0(o): FUN_001f2108(o); defaults; +0x40 flag 0xa (pointer or tagged); clear bit 35 of +0x148.
pub fn ee_sub_229ef0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001f_2108, &[a0], &[]);
    let a1 = m.r32(a0 + 0x40);
    m.w32(a0 + 0x154, 0x32);
    m.wf(a0 + 0x158, f32::from_bits(0x4396_0000));
    m.wf(a0 + 0x15c, f32::from_bits(0x43fa_0000));
    m.w32(a0 + 0x184, 0xffff_ffff);
    m.w32(a0 + 0x178, 0xffff_ffff);
    if (a1 ^ 1) & 1 != 0 {
        let v = m.r32(a1) | 0xa;
        m.w32(a1, v);
    } else {
        m.w32(a0 + 0x40, a1 | 0xa);
    }
    let v = m.r64(a0 + 0x148) & 0xffff_fff7_ffff_ffff;
    m.w32(a0 + 0x148, v as u32);
    m.w32(a0 + 0x14c, (v >> 32) as u32);
}

/// FUN_001c67d8(o): FUN_001c4e50(o); tuning constants.
pub fn ee_sub_1c67d8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001c_4e50, &[a0], &[]);
    m.wf(a0 + 0x1d8, f32::from_bits(0x4234_0000));
    m.wf(a0 + 0x234, f32::from_bits(0x40a0_0000));
    m.wf(a0 + 0x1dc, f32::from_bits(0x4296_0000));
    m.wf(a0 + 0x1d0, f32::from_bits(0x437a_0000));
    m.wf(a0 + 0x1d4, f32::from_bits(0x4248_0000));
    m.wf(a0 + 0x1e4, f32::from_bits(0x3e4c_cccd));
    m.wf(a0 + 0x1e0, f32::from_bits(0x3f26_6666));
    m.wf(a0 + 0x1a0, f32::from_bits(0x3e80_0000));
    m.w32(a0 + 0x23c, 0xa);
    m.w32(a0 + 0x240, 0x14);
    m.wf(a0 + 0x230, f32::from_bits(0x4080_0000));
}

/// FUN_001347a8(o): FUN_001400b8(o) then sub-object init.
pub fn ee_sub_1347a8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0014_00b8, &[a0], &[]);
    let v = m.call(0x001f_4c58, &[a0, 0xb, 1], &[]).0;
    m.w32(a0 + 0x10b8, v);
    m.call(0x001a_0300, &[a0, a0 + 0x13e0, a0 + 0x14e0], &[]);
    m.call(0x001a_0300, &[a0, a0 + 0x14f0, a0 + 0x15f0], &[]);
    m.call(0x0020_b868, &[a0 + 0x1320], &[]);
    m.call(0x0020_b868, &[a0 + 0x1600], &[]);
    let q = m.rv(0x002b_c920);
    m.call(0x0013_6d28, &[a0 + 0x1710, a0], &[f4(-50.0), f4(50.0), q]);
    m.call(0x0014_26e8, &[a0, 0x16], &[]);
    m.call(0x0013_5248, &[a0, 0], &[]);
}

/// FUN_001f14f8(a0..t0, slot): release *slot (FUN_001f0598 + vslot 1); if a0: *slot = FUN_001f03a8(a0..t0); vslot 0.
pub fn ee_sub_1f14f8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, t0: u32, t1: u32) {
    let p = m.r32(t1);
    if p != 0 {
        m.call(0x001f_0598, &[p], &[]);
        let p = m.r32(t1);
        vslot(m, p, 1, &[], &[]);
    }
    m.w32(t1, 0);
    if a0 != 0 {
        let v = m.call(0x001f_03a8, &[a0, a1, a2, a3, t0], &[]).0;
        m.w32(t1, v);
        vslot(m, v, 0, &[], &[]);
    }
}

/// FUN_00207468(o, msg): message filter; returns 1 if handled (via FUN_001eed78(o+0x26c, 0xc, msg)).
pub fn ee_sub_207468<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let mut handle = vslot(m, a0, 0x55, &[], &[]).0 != 0;
    if !handle {
        let x = m.r32(a1 + 0xc);
        if vslot(m, a0, 0x4b, &[x], &[]).0 != 0 {
            return 1;
        }
        let p = m.r32(a1);
        if p != 0 && m.call(0x0013_2b10, &[p, 0x4f], &[]).0 != 0 {
            let p = m.r32(a1);
            let q = m.r32(p + 0x200);
            if m.r32(q + 0x14) == m.r32(a0 + 0x14) {
                return 1;
            }
        }
        if m.call(0x001f_7588, &[a0, a1], &[]).0 == 0 {
            return 0;
        }
    }
    m.call(0x001e_ed78, &[a0 + 0x26c, 0xc, a1], &[]);
    1
}

/// FUN_00211ec0(o): FUN_002123c8(o); resolve the id lists at +0x194/+0x19c and +0x110/+0x118 via FUN_0015e938.
pub fn ee_sub_211ec0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0021_23c8, &[a0], &[]);
    for (cnt, base) in [(0x194u32, 0x19cu32), (0x110, 0x118)] {
        let mut i = 0i32;
        if (m.r32(a0 + cnt) as i32) > 0 {
            let mut s0 = a0 + base;
            loop {
                let id = m.r32(s0.wrapping_sub(4));
                let v = m.call(0x0015_e938, &[GS, 5, id, 0], &[]).0;
                if v != 0 {
                    m.w32(s0, v);
                }
                i += 1;
                s0 = s0.wrapping_add(8);
                if i >= m.r32(a0 + cnt) as i32 {
                    break;
                }
            }
        }
    }
}

/// FUN_00127db8(node, key, deep): find the entry with +8 == key in list +0x1c (returns entry+0xc); if `deep`,
/// recurse into children +0xc[+8] not currently visited (+0x20 is the visit flag).
pub fn ee_sub_127db8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) -> u32 {
    let (s2, s3, s4) = (a0, a1, a2);
    m.w32(s2 + 0x20, 1);
    let mut v1 = m.r32(s2 + 0x1c);
    let mut r = 0u32;
    while v1 != 0 {
        if m.r32(v1 + 8) == s3 {
            r = v1.wrapping_add(0xc);
            m.w32(s2 + 0x20, 0);
            return r;
        }
        v1 = m.r32(v1 + 0x14);
    }
    if s4 == 0 {
        m.w32(s2 + 0x20, 0);
        return r;
    }
    if (m.r32(s2 + 8) as i32) > 0 {
        let mut s1 = 0i32;
        let mut s0 = s2 + 0xc;
        loop {
            let c = m.r32(s0);
            if m.r32(c.wrapping_add(0x20)) == 0 {
                r = m.call(0x0012_7db8, &[c, s3, s4], &[]).0;
                if r != 0 {
                    m.w32(s2 + 0x20, 0);
                    return r;
                }
            }
            s1 += 1;
            s0 = s0.wrapping_add(4);
            if s1 >= m.r32(s2 + 8) as i32 {
                break;
            }
        }
    }
    m.w32(s2 + 0x20, 0);
    r
}

/// FUN_00194558(p, kind): predicate over p+8 (jump tables 0x2a6250 / 0x2a6290).
pub fn ee_sub_194558<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    if a1 == 3 {
        let v1 = m.r32(a0 + 8);
        match v1 {
            0 | 1 | 3 | 4 | 13 => {
                let p = m.r32(a0);
                if m.r32(p + 0x12f0) == 0 {
                    return 0;
                }
                if m.r32(0x002d_fbc0) & 2 == 0 {
                    return 0;
                }
                let v = m.call(0x0014_fd50, &[0x002d_e2e0], &[]).0;
                (v == 0) as u32
            }
            _ => 0,
        }
    } else {
        let i = m.r32(a0 + 8).wrapping_sub(1);
        match i {
            0 | 7 | 10 | 11 => 0,
            2 | 3 => {
                if a1.wrapping_sub(1) < 2 {
                    return 1;
                }
                (a1 == 0x1f) as u32
            }
            _ => 1,
        }
    }
}

/// FUN_0012a0b0(o, kind): *o = kind; o+4 = pooled counter object for kinds 0x1a..0x1e (jump table 0x2a44d0).
pub fn ee_sub_12a0b0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.w32(a0, a1);
    let (arg, off) = match a1.wrapping_sub(0x1a) {
        0 => (0x0045_2188u32, 0x10u32),
        1 => (0x0045_21a0, 0x40),
        2 => (0x0045_21b8, 0x10),
        3 => (0x0045_21d0, 8),
        4 => (0x0045_21e8, 0xc),
        _ => {
            m.w32(a0 + 4, 0);
            return;
        }
    };
    let v = m.call(0x0011_ccc0, &[arg], &[]).0;
    m.w32(v + off, 0);
    m.w32(a0 + 4, v);
    let c = m.r32(v + off).wrapping_add(1);
    m.w32(v + off, c);
}

/// FUN_00163078(o): FUN_001a5e28(o); state machine on +0xc4.
pub fn ee_sub_163078<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5e28, &[a0], &[]);
    let v1 = m.r32(a0 + 0xc4) as i32;
    if v1 == 2 {
        let (x, y) = (m.r32(a0 + 0xb8), m.r32(a0 + 0xb4));
        if m.call(0x0021_21e8, &[x, y], &[]).0 != 0 {
            return;
        }
        m.call(0x0016_2fd0, &[a0], &[]);
        m.call(0x0016_3170, &[a0, 0], &[]);
    } else if v1 < 3 {
        if v1 == 1 {
            let (x, y) = (m.r32(a0 + 0xb8), m.r32(a0 + 0xb0));
            if m.call(0x0021_21e8, &[x, y], &[]).0 != 0 {
                return;
            }
            m.call(0x0016_2f60, &[a0], &[]);
            if m.r32(a0 + 0xc0) != 0 || m.r32(a0 + 0xcc) != 0 {
                m.call(0x0016_3170, &[a0, 3], &[]);
            } else {
                m.call(0x0016_3170, &[a0, 2], &[]);
            }
        }
    } else if v1 == 3 {
        let x = m.r32(a0 + 0xbc);
        if m.call(0x0021_21e8, &[x, 0x30f], &[]).0 != 0 {
            return;
        }
        m.call(0x0016_2fd0, &[a0], &[]);
    }
}

/// FUN_001e63c0(o): sample curve; k = FUN_001e7538(o, &num, &den); linear blend of the +0xf0 table, or (when
/// +0xe0) FUN_001e5fc8(o, k) and a 10-step interpolation over +0x14c plus +0xf0[k].
pub fn ee_sub_1e63c0<M: QMem + ?Sized>(m: &mut M, a0: u32) -> f32 {
    let sp = stack_push(m, 0x10);
    let s0 = m.call(0x001e_7538, &[a0, sp, sp | 4], &[]).0;
    let r;
    if m.r32(a0 + 0xe0) == 0 {
        let f2 = m.rf(sp + 4);
        let f1 = m.rf(sp) / f2;
        let p = (s0 << 2).wrapping_add(m.r32(a0 + 0xf0));
        let b = m.rf(p + 4);
        let a = m.rf(p);
        r = (1.0 - f1) * a + f1 * b;
    } else {
        m.call(0x001e_5fc8, &[a0, s0], &[]);
        let f0 = m.rf(sp) * 10.0;
        let f3 = m.rf(sp + 4);
        let p = (s0 << 2).wrapping_add(m.r32(a0 + 0xf0));
        let f0 = f0 / f3;
        let f4 = m.rf(p);
        let i = f0 as i32 as u32;
        let fr = f0 - (i as i32) as f32;
        let base = a0 + 0x14c;
        let lo = m.rf(base.wrapping_add(i << 2));
        let hi = m.rf(base.wrapping_add(i.wrapping_add(1) << 2));
        r = ((1.0 - fr) * lo + fr * hi) + f4;
    }
    stack_pop(m, 0x10);
    r
}

/// FUN_0011e390(a): unit vector perpendicular to a.xyz (a x p, p chosen from the smallest component), VU0.
pub fn ee_sub_11e390<M: QMem + ?Sized>(m: &mut M, a: V4) -> V4 {
    let (x, y, z) = (a[0], a[1], a[2]);
    // w of p = upper half of the sign-extended mfc1 word that landed in lane 3
    let sext = |f: f32| if (f.to_bits() as i32) < 0 { 0xffff_ffffu32 } else { 0 };
    let p: [f32; 3];
    let pw: u32;
    if x.abs() <= y.abs() {
        if x.abs() <= z.abs() {
            p = [0.0, z, -y];
            pw = sext(-y);
        } else {
            p = [y, -x, 0.0];
            pw = 0;
        }
    } else if y.abs() <= z.abs() {
        p = [z, 0.0, -x];
        pw = sext(-x);
    } else {
        p = [y, -x, 0.0];
        pw = 0;
    }
    let c = [a[1] * p[2] - p[1] * a[2], a[2] * p[0] - p[2] * a[0], a[0] * p[1] - p[0] * a[1]];
    let s = (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).abs().sqrt();
    let q = 1.0 / s;
    // VU has no NaN: 0xffffffff is -(2 - 2^-23) * 2^128; the product clamps to -FLT_MAX
    let w = if pw == 0 {
        0.0 * q
    } else {
        let v = -(2.0f64 - 2f64.powi(-23)) * 2f64.powi(128) * q as f64;
        if v.abs() > f32::MAX as f64 { -f32::MAX } else { v as f32 }
    };
    [c[0] * q, c[1] * q, c[2] * q, w]
}

/// FUN_00164270(o): decide the open state from the +0xe0 / +0xe8 links and the +0x98 entry list; FUN_001657d0(o, s).
pub fn ee_sub_164270<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let sp = stack_push(m, 0x10);
    let mut s2 = 0u32;
    let mut s3 = 0u32;
    if m.r32(a0 + 0xe0) != 0 {
        m.call(0x0025_81c0, &[sp, a0 + 0xe0, 0x003d_4a60], &[]);
        let s0 = m.r32(sp + 4);
        m.call(0x0012_9ec8, &[sp], &[]);
        s2 = s0;
    }
    if m.r32(a0 + 0xe8) != 0 {
        m.call(0x0025_81c0, &[sp, a0 + 0xe8, 0x003d_4a60], &[]);
        let s0 = m.r32(sp + 4);
        m.call(0x0012_9ec8, &[sp], &[]);
        s3 = s0;
    }
    if s3 != 0 {
        let v1 = m.r32(a0 + 0x7c);
        if v1 != 0 && (s2 as i32) < 3 && (s2 as i32) > 0 {
            m.w32(a0 + 0x64, 0);
            if m.r32(v1 + 0x64) == a0 {
                m.w32(v1 + 0x7c, 0);
            }
            s2 = 1;
            let n = m.r32(a0 + 0x94) as i32;
            if n > 0 {
                let arr = m.r32(a0 + 0x98);
                let mut i = 0i32;
                loop {
                    let e = m.r32(arr.wrapping_add((i as u32) << 2));
                    if (m.r32(e + 0x54) as i32) < 2 {
                        s2 = 0;
                        break;
                    }
                    i += 1;
                    if i >= n {
                        break;
                    }
                }
            }
        }
    }
    if s2 == 0 {
        s2 = (m.r32(a0 + 0x94) == 0) as u32;
    }
    m.call(0x0016_57d0, &[a0, s2], &[]);
    stack_pop(m, 0x10);
}

/// FUN_00248de0(x, y): single-precision kernel cos (__kernel_cosf).
pub fn ee_sub_248de0<M: QMem + ?Sized>(m: &mut M, f12: f32, f13: f32) -> f32 {
    let ix = (f12.to_bits() & 0x7fff_ffff) as i32;
    if !(0x31ff_ffff < ix) && (f12 as i32) == 0 {
        return 1.0;
    }
    let z = f12 * f12;
    let mut f0 = z * f32::from_bits(0xad47_d74e);
    f0 = f0 + f32::from_bits(0x310f_74f6);
    f0 = z * f0;
    f0 = f0 + f32::from_bits(0xb493_f27c);
    f0 = z * f0;
    f0 = f0 + f32::from_bits(0x37d0_0d01);
    f0 = z * f0;
    f0 = f0 + f32::from_bits(0xbab6_0b61);
    f0 = z * f0;
    f0 = f0 + f32::from_bits(0x3d2a_aaab);
    let f1 = z * f0;
    if !(0x3e99_9999 < ix) {
        let f1 = z * f1;
        let f2 = f12 * f13;
        let f0 = z * 0.5;
        let f1 = f1 - f2;
        let f0 = f0 - f1;
        return 1.0 - f0;
    }
    let qx = if 0x3f48_0000 < ix { 0x3e90_0000u32 } else { (ix as u32).wrapping_add(0xff00_0000) };
    let f4 = f32::from_bits(qx);
    let f2 = z * f1;
    let f3 = f12 * f13;
    let f0 = z * 0.5;
    let f2 = f2 - f3;
    let f0 = f0 - f4;
    let f1 = 1.0 - f4;
    let f0 = f0 - f2;
    f1 - f0
}

/// FUN_001317d8(o, pos, vel): clamp / wrap / bounce *pos against [0, (o+0x14)+0x74] per flags +0x3c; returns 1
/// when the velocity pointed out of range.
pub fn ee_sub_1317d8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) -> u32 {
    let t = m.r32(a0 + 0x14);
    let f20 = m.rf(t + 0x74);
    let mut f12 = m.rf(a1);
    let mut s3 = 0u32;
    let f21;
    if f20 <= f12 {
        if 0.0 < m.rf(a2) {
            s3 = 1;
        }
        let v1 = m.r32(a0 + 0x3c);
        if v1 & 8 != 0 {
            let f0 = m.call(0x0024_5a18, &[], &[f4(f12), f4(f20)]).1[0];
            m.wf(a1, f0);
            return s3;
        }
        if v1 & 0x10 == 0 {
            m.wf(a1, f20);
            let f0 = m.rf(a2).min(0.0);
            m.wf(a2, f0);
            return s3;
        }
        f21 = f20 + f20;
    } else {
        if !(f12 < 0.0) {
            return s3;
        }
        if m.rf(a2) < 0.0 {
            s3 = 1;
        }
        let v1 = m.r32(a0 + 0x3c);
        if v1 & 8 != 0 {
            let f0 = m.call(0x0011_d898, &[], &[f4(f12), f4(f20)]).1[0];
            m.wf(a1, f0);
            return s3;
        }
        if v1 & 0x10 == 0 {
            m.wf(a1, 0.0);
            let f0 = m.rf(a2).max(0.0);
            m.wf(a2, f0);
            return s3;
        }
        f21 = f20 + f20;
        f12 = -f12;
    }
    let f0 = m.call(0x0024_5a18, &[], &[f4(f12), f4(f21)]).1[0];
    m.wf(a1, f0);
    let v = m.rf(a2).abs();
    m.wf(a2, v);
    let v = m.rf(a0 + 0x44).abs();
    m.wf(a0 + 0x44, v);
    let f0 = m.rf(a1);
    if f20 <= f0 {
        m.wf(a1, f21 - f0);
        let v = -m.rf(a2);
        m.wf(a2, v);
        let v = -m.rf(a0 + 0x44);
        m.wf(a0 + 0x44, v);
    }
    s3
}

/// FUN_001b8d30(o, st): switch state +0x1840 (jump table 0x2a74e0); then FUN_001f15b8 + FUN_001305b8 with the
/// state's anim.
pub fn ee_sub_1b8d30<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let v1 = m.r32(a0 + 0x1840);
    if a1 == v1 {
        return;
    }
    if v1 == 2 {
        let p = m.r32(a0 + 0x1900);
        m.call(0x0020_c080, &[p], &[]);
        let p = m.r32(a0 + 0x1900);
        vslot(m, p, 1, &[], &[]);
        m.w32(a0 + 0x1900, 0);
    }
    m.w32(a0 + 0x1840, a1);
    let mut s3 = 0u32;
    let mut s2 = 0u32;
    let f0 = m.rf(0x002d_ded0);
    m.wf(a0 + 0x1844, f0);
    match a1.wrapping_add(1) {
        0 => {
            m.w32(a0 + 0x1908, 0);
            return;
        }
        1 => {
            s3 = m.r32(a0 + 0x1848);
            m.w32(a0 + 0x194, 0);
            s2 = 1;
            m.w32(a0 + 0x198, 0);
            m.w32(a0 + 0x190, 0);
        }
        2 => {
            s3 = m.r32(a0 + 0x184c);
            s2 = 9;
        }
        3 => {
            m.w32(a0 + 0x1908, 0);
            m.w32(a0 + 0x1900, a0 + 0x1890);
            m.call(0x0020_bf80, &[a0 + 0x1850, a0, 1, a0 + 0x1890], &[]);
            let s0 = m.r32(a0 + 0x1900);
            let r = m.call(0x001a_62e8, &[a0], &[]).1;
            let f = -m.rf(a0 + 0x186c);
            let p = m.call(0x001a_6318, &[a0], &[]).0;
            let q = m.rv(p);
            let o = [r[0] + q[0] * f, r[1] + q[1] * f, r[2] + q[2] * f, r[3] + q[3] * f];
            m.wv(s0 + 0x40, o);
            let v = m.rv(0x002a_2dd0);
            m.wv(a0 + 0x190, v);
        }
        4 => {
            s3 = m.r32(a0 + 0x1904);
            s2 = 1;
        }
        _ => {}
    }
    m.call(0x001f_15b8, &[a0 + 0x1034], &[]);
    m.call(0x0013_05b8, &[s3, a0, s2, a0 + 0x1030], &[f4(0.0), f4(1.0)]);
}

fn xform3<M: QMem + ?Sized>(m: &mut M, r0: u32, r1: u32, r2: u32, v: V4) -> V4 {
    let (a, b, c) = (m.rv(r0), m.rv(r1), m.rv(r2));
    [
        a[0] * v[0] + b[0] * v[1] + c[0] * v[2],
        a[1] * v[0] + b[1] * v[1] + c[1] * v[2],
        a[2] * v[0] + b[2] * v[1] + c[2] * v[2],
        v[3],
    ]
}

fn add4(a: V4, b: V4) -> V4 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2], a[3] + b[3]]
}

/// FUN_0015b7e8(n, o, out): allocate the 4 per-particle arrays (FUN_001ae318), init each particle
/// (FUN_0015a808 / FUN_0015b0c8), transform pos / vel by the +0x1c frame, optional FUN_0015ad48.
pub fn ee_sub_15b7e8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    let (s7, s6, s4) = (a0, a1, a2);
    let sp = stack_push(m, 0x20);
    let s0 = sp; // {size, align} at +0x10 in the frame
    let s2 = s7 << 4;
    let s3 = s7 << 2;
    for (k, sz) in [s2, s2, s3, s3].into_iter().enumerate() {
        m.w32(s0, sz);
        m.w32(s0 + 4, 0x10);
        let v = m.call(0x001a_e318, &[s0], &[]).0;
        m.w32(s4 + 4 * k as u32, v);
    }
    let s1 = sp + 0x10;
    let p = m.r32(s6 + 0x84);
    m.call(0x0015_ae38, &[p, s6 + 0x90, s7, s1], &[]);
    if (s7 as i32) > 0 {
        let mut s5 = 0i32;
        loop {
            let o16 = (s5 as u32) << 4;
            let o4 = (s5 as u32) << 2;
            let a3 = m.r32(s4).wrapping_add(o16);
            m.call(0x0015_a808, &[s6, s5 as u32, s7, a3, s0], &[]);
            let st = m.r32(s4 + 0xc).wrapping_add(o4);
            let t0 = m.r32(s4).wrapping_add(o16);
            let t2 = m.r32(s4 + 4).wrapping_add(o16);
            let t3 = m.r32(s4 + 8).wrapping_add(o4);
            m.call(0x0015_b0c8, &[s6 + 0x50, s1, s6 + 0x10, s5 as u32, t0, s0, t2, t3, st], &[]);
            let f = m.r32(s6 + 0x1c);
            if f != 0 {
                let v0 = m.r32(s4).wrapping_add(o16);
                let v = m.rv(v0);
                let t = xform3(m, f + 0x10, f + 0x20, f + 0x30, v);
                let org = m.rv(f);
                m.wv(v0, add4(t, org));
                let v0 = m.r32(s4).wrapping_add(o16);
                let v1 = m.r32(s4 + 4).wrapping_add(o16);
                let org = m.rv(f);
                let p = m.rv(v0);
                let d = [p[0] - org[0], p[1] - org[1], p[2] - org[2], p[3] - org[3]];
                let vel = m.rv(v1);
                let t = xform3(m, f + 0x10, f + 0x20, f + 0x30, vel);
                let lin = m.rv(f + 0x40);
                let w = m.rv(f + 0x50);
                let t = add4(t, lin);
                let c = [w[1] * d[2] - d[1] * w[2], w[2] * d[0] - d[2] * w[0], w[0] * d[1] - d[0] * w[1], w[3]];
                m.wv(v1, add4(t, c));
            }
            if m.r32(s4 + 0x10) != 0 {
                let x = m.r32(s4).wrapping_add(o16);
                let y = m.r32(s4 + 4).wrapping_add(o16);
                m.call(0x0015_ad48, &[s6, x, y], &[]);
            }
            s5 += 1;
            if s5 >= s7 as i32 {
                break;
            }
        }
    }
    stack_pop(m, 0x20);
}

/// FUN_00164640(o, st, ctx): set state with temporarily swapped +0xf4/+0xf8 (and the +0x7c partner's
/// +0xf0/+0xf4) context; dispatches FUN_00164df0 / FUN_00163e88 / FUN_00163c28 / FUN_00163ce0.
pub fn ee_sub_164640<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    let s0 = a0;
    let mut s1 = a1;
    let v1 = m.r32(s0 + 0xf4);
    let sp4 = s0;
    let (mut sp8, mut spc) = (0u32, 0u32);
    if s0 != 0 {
        let v0 = m.r32(s0 + 0xf8);
        m.w32(s0 + 0xf4, v1);
        spc = v0;
        sp8 = v1;
        m.w32(s0 + 0xf8, a2);
    }
    let sp10 = m.r32(s0 + 0x7c);
    let a0f4 = m.r32(s0 + 0xf4);
    let (mut sp18, mut sp1c) = (0u32, 0u32);
    if sp10 != 0 {
        sp18 = m.r32(sp10 + 0xf0);
        m.w32(sp10 + 0xf0, a0f4);
        sp1c = m.r32(sp10 + 0xf4);
        m.w32(sp10 + 0xf4, a2);
    }
    if !s1 == 0 {
        s1 = 4;
    }
    let a0v = m.r32(s0 + 0x54);
    'body: {
        if a0v == 0xffff_ffff {
            break 'body;
        }
        let si = s1 as i32;
        if si == 4 {
            if a0v == 0 {
                if m.call(0x0016_45d0, &[s0], &[]).0 == 0 {
                    m.call(0x0016_3e88, &[s0, 4], &[]);
                    break 'body;
                }
                m.call(0x0016_4df0, &[s0, 1], &[]);
            } else if a0v == 3 {
                break 'body;
            }
            m.call(0x0016_3c28, &[s0], &[]);
            break 'body;
        }
        if si >= 5 {
            if si == 5 {
                m.call(0x0016_3ce0, &[s0], &[]);
            }
            break 'body;
        }
        if si < 0 {
            break 'body;
        }
        let mut v1 = (s1.wrapping_sub(a0v) == 1) as u32;
        let ai = a0v as i32;
        if ai == 1 {
            if s1 ^ 3 == 0 {
                v1 = a0v;
            }
        } else if ai < 2 {
            if ai == 0 && s1 == 1 {
                v1 = m.call(0x0016_45d0, &[s0], &[]).0;
            }
        } else if ai == 3 {
            break 'body;
        }
        if v1 == 0 {
            m.call(0x0016_3e88, &[s0, s1], &[]);
        } else {
            m.call(0x0016_4df0, &[s0, s1], &[]);
        }
    }
    if sp10 != 0 {
        m.w32(sp10 + 0xf0, sp18);
        m.w32(sp10 + 0xf4, sp1c);
    }
    if sp4 != 0 {
        m.w32(sp4 + 0xf4, sp8);
        m.w32(sp4 + 0xf8, spc);
    }
}

/// FUN_001638f8(o): post-load: resolve the +0x90 / +0x9c id lists (compacting into +0x98 / +0xa4), initial
/// state, link lookups +0xc8 / +0xd0 -> +0x64 / +0x68, flags.
pub fn ee_sub_1638f8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let s1 = a0;
    m.call(0x001a_5c48, &[s1], &[]);
    let p = m.r32(s1 + 0x7c);
    if p != 0 {
        m.call(0x0011_a070, &[p + 0x80, s1], &[]);
    }
    for (cnt, src, dst) in [(0x94u32, 0x90u32, 0x98u32), (0xa0, 0x9c, 0xa4)] {
        let mut s2 = 0i32;
        let mut s0 = 0u32;
        if (m.r32(s1 + cnt) as i32) > 0 {
            loop {
                let arr = m.r32(s1 + src);
                let id = m.r32(((s2 as u32) << 2).wrapping_add(arr));
                let v = m.call(0x0015_e938, &[GS, 0x105, id, 0], &[]).0;
                if v != 0 {
                    let d = m.r32(s1 + dst);
                    m.w32((s0 << 2).wrapping_add(d), v);
                    s0 = s0.wrapping_add(1);
                }
                s2 += 1;
                if s2 >= m.r32(s1 + cnt) as i32 {
                    break;
                }
            }
        }
        m.w32(s1 + cnt, s0);
    }
    if m.call(0x0016_4018, &[s1], &[]).0 == 0 {
        if m.call(0x0016_37e8, &[s1], &[]).0 != 0 {
            let (x, y) = (m.r32(s1 + 0xa8), m.r32(s1 + 0xac));
            m.call(0x0016_20b0, &[s1, 5, x, y], &[]);
            let (x, y) = (m.r32(s1 + 0xb0), m.r32(s1 + 0xb4));
            m.call(0x0016_23d0, &[s1, 5, x, y, 1], &[]);
        }
    }
    let sp = stack_push(m, 0x10);
    for (src, dst) in [(0xc8u32, 0x64u32), (0xd0, 0x68)] {
        if m.r32(s1 + src) != 0 {
            m.call(0x0025_81c0, &[sp, s1 + src, 0x003d_4a60], &[]);
            let s0 = m.r32(sp + 4);
            m.call(0x0012_9ec8, &[sp], &[]);
            m.w32(s1 + dst, s0);
        }
    }
    stack_pop(m, 0x10);
    if m.rf(s1 + 0x78) == 0.0 && m.r32(s1 + 0x80) == 0 {
        return;
    }
    let x = m.r64(s1 + 0x28);
    if ((x << 8) as i64 >> 60) == 2 {
        m.call(0x001a_6ca8, &[s1, 0], &[]);
    }
    let v1 = m.r32(s1 + 0x40);
    if (v1 ^ 1) & 1 != 0 {
        let v = m.r32(v1) | 2;
        m.w32(v1, v);
    } else {
        m.w32(s1 + 0x40, v1 | 2);
    }
}

/// FUN_001527c8(o): FUN_001a5c48(o); FUN_00149a50(GS, o, <8-byte handler @0x2a5010>, 0). The 64-bit handler
/// word is passed as its low word (project convention, cf. q_wireq2::ee_sub_149a50).
pub fn ee_sub_1527c8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5c48, &[a0], &[]);
    let k = m.r32(0x002a_5010);
    m.call(0x0014_9a50, &[GS, a0, k, 0], &[]);
}

/// FUN_00148c98(o): FUN_001f36e0(o); per-state (+0x1e0, jump table 0x2a4e90) choose the next state for
/// FUN_001490a0.
pub fn ee_sub_148c98<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let s3 = a0;
    m.call(0x001f_36e0, &[s3], &[]);
    let st = m.r32(s3 + 0x1e0);
    match st {
        0 | 1 => {
            let a0v = m.r32(s3 + 0x2a8) as i32;
            let mut s4 = m.r32(s3 + 0x1e0);
            if a0v < 3 {
                if a0v == 0 {
                    s4 = 0;
                    let e = m.r32(s3 + 0xec);
                    let mut s2 = m.r32(e + 0x64);
                    'outer: while s2 != 0 {
                        let mut s0 = m.r32(s2);
                        while s0 != 0 {
                            if m.r32(s0 + 0x9c) != 0xa {
                                let e = m.r32(s3 + 0xec);
                                let k = m.r32(s0 + 8);
                                let d = (m.r32(e + 0x60) != k) as u32;
                                let a = m.r32(s0.wrapping_add(d << 6) + 0xc);
                                let s1 = m.r32(s0.wrapping_add(1u32.wrapping_sub(d) << 6) + 8);
                                if a == s3 && m.call(0x0014_9100, &[s3, s1], &[]).0 != 0 {
                                    m.w32(s3 + 0x2f8, s1);
                                    let k = m.r32(0x002a_4e80);
                                    m.call(0x0014_9ab8, &[GS, s3, k, 0], &[]);
                                    s4 = m.r32(s3 + 0x1e0);
                                    break 'outer;
                                }
                            }
                            s0 = m.r32(s0 + 0xac);
                        }
                        s2 = m.r32(s2 + 4);
                    }
                }
            } else if a0v == 3 {
                s4 = 0;
                let v1 = m.r32(0x002e_1e40);
                if v1 != 0 && m.r32(v1 + 0x18c8) == 3 {
                    s4 = (m.r32(v1 + 0x18c4) == m.r32(s3 + 0x2f4)) as u32;
                }
            }
            m.call(0x0014_90a0, &[s3, s4], &[]);
        }
        2 => {
            let a = m.r32(0x002e_1e40);
            if a != 0 && m.r32(a + 0x18c8) == 3 && m.r32(a + 0x18c4) == m.r32(s3 + 0x2f4) {
                let f = m.rf(0x002d_ded0);
                m.wf(s3 + 0x1e4, f);
            }
            let f2 = m.rf(s3 + 0x1e8);
            if !(0.0 <= f2) {
                return;
            }
            let f0 = m.rf(0x002d_ded0) - m.rf(s3 + 0x1e4);
            if !(f2 < f0) {
                return;
            }
            m.call(0x0014_90a0, &[s3, 4], &[]);
        }
        4 => {
            let v1 = m.r32(s3 + 0x2a8) as i32;
            let a1: i32 = if v1 == 1 {
                0
            } else if v1 == 0 || v1 == 2 || v1 == 3 {
                1
            } else {
                -1
            };
            if a1 != -1 {
                m.call(0x0014_90a0, &[s3, a1 as u32], &[]);
            }
        }
        _ => {}
    }
}

fn sub4(a: V4, b: V4) -> V4 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2], a[3] - b[3]]
}

/// VU0 vopmula/vopmsub pair: a x b on xyz, w kept from `w`.
fn cross_w(a: V4, b: V4, w: f32) -> V4 {
    [a[1] * b[2] - b[1] * a[2], a[2] * b[0] - b[2] * a[0], a[0] * b[1] - b[0] * a[1], w]
}

fn dot3(a: V4, b: V4) -> f32 {
    // vmul.xyz + vadday.x + vmaddz.x (1.0 * z)
    let p = [a[0] * b[0], a[1] * b[1], a[2] * b[2]];
    (p[0] + p[1]) + 1.0 * p[2]
}

/// FUN_00185310(o, _, bary_out, pt_out, nrm_out, p): closest point to p over the triangle mesh (+0xe4 verts,
/// +0xe8 count, +0xec index triples) in the frame of vslot 0x1e; outputs {tri, u, v}, the point and the unit
/// normal. VU registers whose w lanes are never written in this function (vf10..vf12) are taken as 0.
pub fn ee_sub_185310<M: QMem + ?Sized>(m: &mut M, a0: u32, a2: u32, a3: u32, t0: u32, t1: u32, p: V4) {
    let s1 = a0;
    let (s3, s4, s5) = (a3, t0, t1);
    let mp = vslot(m, s1, 0x1e, &[], &[]).0;
    let r0 = m.rv(mp);
    let rr1 = m.rv(mp + 0x10);
    let rr2 = m.rv(mp + 0x20);
    let rr3 = m.rv(mp + 0x30);
    let mut f22 = m.rf(0x002a_5684);
    let b = stack_push(m, 0x30); // sp+0x60 (point) / sp+0x80 (t)
    let mut s2: V4 = [0.0; 4];
    let mut best: [u32; 3] = [0; 3];
    let mut sp144: V4 = [0.0; 4];
    let xf = |v: V4| -> V4 {
        [
            r0[0] * v[0] + rr1[0] * v[1] + rr2[0] * v[2] + rr3[0],
            r0[1] * v[0] + rr1[1] * v[1] + rr2[1] * v[2] + rr3[1],
            r0[2] * v[0] + rr1[2] * v[1] + rr2[2] * v[2] + rr3[2],
            0.0,
        ]
    };
    let mut s0 = 0i32;
    if (m.r32(s1 + 0xe8) as i32) > 0 {
        loop {
            let tri = m.r32(s1 + 0xec).wrapping_add((s0 as u32).wrapping_mul(0xc));
            let vb = m.r32(s1 + 0xe4);
            let i0 = m.r32(tri);
            let i2 = m.r32(tri + 8);
            let i1 = m.r32(tri + 4);
            let v0 = m.rv((i0 << 4).wrapping_add(vb));
            let v1 = m.rv((i1 << 4).wrapping_add(vb));
            let v2 = m.rv((i2 << 4).wrapping_add(vb));
            let va = xf(v0);
            let vbb = xf(v1);
            let vc = xf(v2);
            let e3 = sub4(va, vc);
            let e1 = sub4(vbb, vc);
            let n = cross_w(e3, e1, v1[3]);
            let d = sub4(p, vc);
            let ne1 = cross_w(n, e1, e1[3]);
            let de3 = cross_w(d, e3, n[3]);
            let inv = 1.0 / dot3(e3, ne1);
            let f2 = inv * dot3(d, ne1);
            let f1 = inv * dot3(n, de3);
            let (mut u, mut v) = (0.0f32, 0.0f32);
            let pt: V4;
            if f2 < 0.0 {
                m.call(0x0011_ee08, &[b, b + 0x20, 0], &[p, vbb, vc]);
                let t = m.rf(b + 0x20);
                u = 0.0;
                v = 1.0 - t;
                pt = m.rv(b);
            } else if f1 < 0.0 {
                m.call(0x0011_ee08, &[b, b + 0x20, 0], &[p, va, vc]);
                let t = m.rf(b + 0x20);
                v = 0.0;
                u = 1.0 - t;
                pt = m.rv(b);
            } else if 1.0 < f2 + f1 {
                m.call(0x0011_ee08, &[b, b + 0x20, 0], &[p, va, vbb]);
                let t = m.rf(b + 0x20);
                v = t;
                u = 1.0 - t;
                pt = m.rv(b);
            } else {
                let w = (1.0 - f2) - f1;
                u = f2;
                v = f1;
                pt = [
                    va[0] * f2 + vbb[0] * f1 + vc[0] * w,
                    va[1] * f2 + vbb[1] * f1 + vc[1] * w,
                    va[2] * f2 + vbb[2] * f1 + vc[2] * w,
                    va[3] * f2 + vbb[3] * f1 + vc[3] * w,
                ];
                m.wv(b, pt);
            }
            let dd = sub4(pt, p);
            let dist = dot3(dd, dd);
            if dist < f22 {
                f22 = dist;
                best = [s0 as u32, u.to_bits(), v.to_bits()];
                s2 = pt;
                sp144 = n;
            }
            s0 += 1;
            if s0 >= m.r32(s1 + 0xe8) as i32 {
                break;
            }
        }
    }
    stack_pop(m, 0x30);
    if s3 != 0 {
        m.w32(s3, best[0]);
        m.w32(s3 + 4, best[1]);
        m.w32(s3 + 8, best[2]);
    }
    if s4 != 0 {
        m.wv(s4, s2);
    }
    if s5 != 0 {
        let l = dot3(sp144, sp144).abs().sqrt();
        let q = 1.0 / l;
        m.wv(s5, [sp144[0] * q, sp144[1] * q, sp144[2] * q, sp144[3] * q]);
    }
}

/// FUN_002488c8(y, x): single-precision __ieee754_rem_pio2f; y[0..2] = remainder, returns n.
pub fn ee_sub_2488c8<M: QMem + ?Sized>(m: &mut M, a0: u32, f12: f32) -> i32 {
    let s1 = a0;
    let hx = f12.to_bits() as i32;
    let ix = hx & 0x7fff_ffff;
    let fb = f32::from_bits;
    if !(0x3f49_0fd8 < ix) {
        m.wf(s1, f12);
        m.w32(s1 + 4, 0);
        return 0;
    }
    if !(0x4016_cbe3 < ix) {
        let pio2_1 = fb(0x3fc9_0f80);
        if hx > 0 {
            let mut z = f12 - pio2_1;
            let (y0, f2);
            if (ix as u32 & 0xffff_fff0) != 0x3fc9_0fd0 {
                f2 = fb(0x3735_4443);
                y0 = z - f2;
            } else {
                z = z - fb(0x3735_4400);
                f2 = fb(0x2e85_a308);
                y0 = z - f2;
            }
            let y1 = (z - y0) - f2;
            m.wf(s1, y0);
            m.wf(s1 + 4, y1);
            return 1;
        } else {
            let mut z = f12 + pio2_1;
            let (y0, f2);
            if (ix as u32 & 0xffff_fff0) != 0x3fc9_0fd0 {
                f2 = fb(0x3735_4443);
                y0 = z + f2;
            } else {
                z = z + fb(0x3735_4400);
                f2 = fb(0x2e85_a308);
                y0 = z + f2;
            }
            let y1 = (z - y0) + f2;
            m.wf(s1, y0);
            m.wf(s1 + 4, y1);
            return -1;
        }
    }
    if !(0x4349_0f80 < ix) {
        let t = m.call(0x0024_4ef8, &[], &[f4(f12)]).1[0];
        let mut f5 = t;
        let n = ((f5 * fb(0x3f22_f984)) + 0.5) as i32;
        let f6 = n as f32;
        let f0 = f6 * fb(0x3fc9_0f80);
        let mut f3 = f6 * fb(0x3735_4443);
        let mut f4r = f5 - f0;
        let mut y0;
        let mut direct = false;
        if n < 0x20 {
            let tab = m.r32(((n.wrapping_sub(1) as u32) << 2).wrapping_add(0x002a_ffb0));
            if (ix as u32 & 0xffff_ff00) != tab {
                y0 = f4r - f3;
                m.wf(s1, y0);
                direct = true;
            }
        }
        if !direct {
            y0 = f4r - f3;
            let j = (ix >> 23) as i32;
            m.wf(s1, y0);
            let i = j - ((y0.to_bits() >> 23) & 0xff) as i32;
            if !(i < 9) {
                f5 = f4r;
                let w = f6 * fb(0x3735_4400);
                let f1 = f6 * fb(0x2e85_a308);
                f4r = f4r - w;
                let f0 = (f5 - f4r) - w;
                f3 = f1 - f0;
                y0 = f4r - f3;
                m.wf(s1, y0);
                let i = j - ((y0.to_bits() >> 23) & 0xff) as i32;
                if !(i < 0x1a) {
                    f5 = f4r;
                    let w = f6 * fb(0x2e85_a300);
                    let f2 = f6 * fb(0x248d_3132);
                    f4r = f4r - w;
                    let f0 = (f5 - f4r) - w;
                    f3 = f2 - f0;
                    y0 = f4r - f3;
                    m.wf(s1, y0);
                }
            }
        }
        let f1 = m.rf(s1);
        let f0 = (f4r - f1) - f3;
        m.wf(s1 + 4, f0);
        if hx >= 0 {
            return n;
        }
        m.wf(s1, -f1);
        m.wf(s1 + 4, -f0);
        return n.wrapping_neg();
    }
    if 0x7f7f_ffff < ix {
        let f0 = f12 - f12;
        m.wf(s1, f0);
        m.wf(s1 + 4, f0);
        return 0;
    }
    let sp = stack_push(m, 0x10);
    let e0 = (ix >> 23) - 0x86;
    let mut z = fb((ix as u32).wrapping_sub((e0 as u32) << 23));
    for i in 0..2u32 {
        let t = (z as i32) as f32;
        let f1 = z - t;
        m.wf(sp + 4 * i, t);
        z = f1 * 256.0;
    }
    m.wf(sp + 8, z);
    let mut a3: u32 = 3;
    if z == 0.0 {
        a3 = 2;
        loop {
            let a0i = a3.wrapping_sub(1);
            if m.rf(sp.wrapping_add(a0i << 2)) == 0.0 {
                a3 = a0i;
            } else {
                break;
            }
        }
    }
    let n = m.call(0x0024_8f38, &[sp, s1, e0 as u32, a3, 2, 0x002a_bc98], &[]).0 as i32;
    stack_pop(m, 0x10);
    if hx >= 0 {
        return n;
    }
    let f1 = -m.rf(s1);
    let f0 = -m.rf(s1 + 4);
    m.wf(s1, f1);
    m.wf(s1 + 4, f0);
    n.wrapping_neg()
}

fn len3(a: V4) -> f32 {
    dot3(a, a).abs().sqrt()
}

fn scale4(a: V4, q: f32) -> V4 {
    [a[0] * q, a[1] * q, a[2] * q, a[3] * q]
}

/// FUN_001c3500(o, flip, out, t): search outward from t (+/-10, alternating) along the path for a segment whose
/// direction is within ~154 deg of the path tangent; build the frame (3 rows) into `out`.
/// Frame locals live at a stack block mirroring the original sp offsets.
pub fn ee_sub_1c3500<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, f12: f32) -> () {
    let (s0, s7, s4) = (a0, a1, a2);
    let f24 = f12;
    let sp = stack_push(m, 0xe0);
    let eps = f32::from_bits(0x38d1_b717);
    let f22 = m.call(0x001e_6620, &[s0], &[]).1[0];
    let f23 = m.call(0x001e_6648, &[s0], &[]).1[0];
    let f0 = m.call(0x001e_6610, &[s0], &[]).1[0];
    m.call(0x001e_67b8, &[s0, 0, sp + 0x30], &[f4(f0)]);
    let v3 = m.rv(sp + 0x30);
    let l = len3(v3);
    let d = if l < eps { m.rv(0x002a_2da0) } else { scale4(v3, 1.0 / l) };
    m.wv(sp + 0x30, d);
    let mut s1: u32 = 0;
    loop {
        let f0 = if s1 & 1 != 0 {
            10.0 * (s1.wrapping_add(1) as i32 as f32)
        } else {
            (s1 as i32 as f32) * -10.0
        };
        let mut f21 = f24 + f0;
        if m.r32(s0 + 0xe4) == 0 {
            if f21 < f22 {
                s1 = s1.wrapping_add(1);
                continue;
            }
            if f23 < f24 {
                s1 = s1.wrapping_add(1);
                continue;
            }
        } else {
            let r = m.call(0x0011_d898, &[], &[f4(f21 - f22), f4(f23 - f22)]).1[0];
            f21 = f22 + r;
        }
        let f20 = 50.0f32;
        let t = m.call(0x001e_6728, &[s0], &[f4(f21 + f20)]).1[0];
        m.call(0x001e_6f10, &[s0, sp + 0x60, 0], &[f4(t)]);
        let t = m.call(0x001e_6728, &[s0], &[f4(f21 - f20)]).1[0];
        m.call(0x001e_6f10, &[s0, sp + 0x70, 0], &[f4(t)]);
        let v3 = sub4(m.rv(sp + 0x60), m.rv(sp + 0x70));
        let l = len3(v3);
        let v1 = if l < eps { m.rv(0x002a_2dc0) } else { scale4(v3, 1.0 / l) };
        m.wv(sp + 0x50, v1);
        if s1 == 0 {
            m.wv(sp + 0x40, v1);
        }
        let a = m.rv(sp + 0x30);
        let b = m.rv(sp + 0x50);
        if !(f32::from_bits(0xbf66_6666) < dot3(a, b)) {
            s1 = s1.wrapping_add(1);
            continue;
        }
        m.call(0x0011_ba08, &[sp + 0xb0], &[a, b]);
        for k in 0..3u32 {
            let v = m.rv(sp + 0xb0 + 0x10 * k);
            m.wv(sp + 0x80 + 0x10 * k, v);
        }
        let q = m.rv(s0 + 0x1c0);
        let r = m.call(0x001a_6fc0, &[s0, 0], &[q]).1;
        let v5 = m.rv(sp + 0x40);
        let l5 = len3(v5);
        let mut v2 = m.rv(sp + 0x60);
        let (r0, r1, r2) = (m.rv(sp + 0x80), m.rv(sp + 0x90), m.rv(sp + 0xa0));
        for i in 0..3 {
            v2[i] = r0[i] * r[0] + r1[i] * r[1] + r2[i] * r[2];
        }
        m.wv(sp + 0x60, v2);
        let v4 = if l5 < eps { m.rv(0x002a_2dc0) } else { scale4(v5, 1.0 / l5) };
        let v1 = cross_w(v4, m.rv(sp + 0x60), v2[3]);
        m.wv(sp + 0x10, v1);
        let l1 = len3(v1);
        m.wv(sp + 0x20, v4);
        let v2n = scale4(v1, 1.0 / l1);
        m.wv(sp + 0x10, v2n);
        m.wv(sp + 0x70, v1);
        if l1 < eps {
            let v = m.call(0x0011_e390, &[], &[v4]).1;
            m.wv(sp + 0x10, v);
        }
        let v4 = m.rv(sp + 0x10);
        let v2 = m.rv(sp + 0x20);
        let v0 = cross_w(v4, v2, v1[3]);
        m.wv(sp, v0);
        if s7 == 0 {
            m.wv(sp + 0x20, scale4(v2, -1.0));
            m.wv(sp + 0x10, scale4(v4, -1.0));
        }
        if s4 != 0 {
            let (a, b, c) = (m.rv(sp), m.rv(sp + 0x10), m.rv(sp + 0x20));
            m.wv(s4, a);
            m.wv(s4 + 0x20, c);
            m.wv(s4 + 0x10, b);
        }
        break;
    }
    stack_pop(m, 0xe0);
}

fn ratio_e00<M: QMem + ?Sized>(m: &mut M, p: u32) -> f32 {
    let c = m.r32(p + 0xe00);
    let den = m.r32(p + 0xdfc) as i32 as f32;
    (m.r32(c) as i32 as f32) / den
}

/// FUN_001d9df8(o): track the target (+0x34, via FUN_001c4840) and drive the meter state machine (+0x30,
/// jump table 0x2a7930) through FUN_001d9b60.
pub fn ee_sub_1d9df8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let s0 = a0;
    if m.r32(s0 + 0x38) == 0 {
        return;
    }
    let s2 = m.r32(s0 + 0x34);
    m.w32(s0 + 0x34, 0);
    let s1 = m.call(0x001c_4840, &[GS], &[]).0;
    if s1 != 0 && m.call(0x0013_2b10, &[s1, 0x34], &[]).0 != 0 {
        m.w32(s0 + 0x34, s1);
    }
    let v0 = m.r32(s0 + 0x34);
    let eps = f32::from_bits(0x38d1_b717);
    if v0 == 0 {
        m.wf(s0 + 0x28, 0.0);
        m.wf(s0 + 0x24, 0.0);
        m.wf(s0 + 0x20, 0.0);
        m.wf(s0 + 0x2c, 0.0);
        if s2 == 0 {
            return;
        }
        m.call(0x001d_9b60, &[s0, 0], &[]);
        return;
    }
    if s2 != v0 {
        m.call(0x001d_9b60, &[s0, if s2 == 0 { 3 } else { 5 }], &[]);
        let ids: Option<[u32; 5]> = {
            let p = m.r32(s0 + 0x34);
            if m.call(0x0013_2b10, &[p, 0x35], &[]).0 != 0 {
                Some([0x33d, 0x33e, 0x33f, 0x340, 0x341])
            } else {
                let p = m.r32(s0 + 0x34);
                if m.call(0x0013_2b10, &[p, 0x37], &[]).0 != 0 {
                    Some([0x347, 0x348, 0x349, 0x34a, 0x34b])
                } else {
                    let p = m.r32(s0 + 0x34);
                    if m.call(0x0013_2b10, &[p, 0x36], &[]).0 != 0 {
                        Some([0x342, 0x343, 0x344, 0x345, 0x346])
                    } else {
                        None
                    }
                }
            }
        };
        if let Some(ids) = ids {
            for (k, id) in ids.iter().enumerate() {
                m.w32(s0 + 0x3c + 4 * k as u32, *id);
            }
        }
        let p = m.r32(s0 + 0x34);
        let f0 = ratio_e00(m, p);
        m.wf(s0 + 0x20, f0);
        m.wf(s0 + 0x24, f0);
        let v = m.call(0x0014_41a0, &[p], &[]).0;
        let q = m.r32(s0 + 0x34);
        let f = (v as i32 as f32) / (m.r32(q + 0x1300) as i32 as f32);
        m.wf(s0 + 0x28, f);
        m.wf(s0 + 0x2c, f);
    }
    let st = m.r32(s0 + 0x30);
    match st {
        1 | 6 | 7 => {
            let p = m.r32(s0 + 0x34);
            let f0 = ratio_e00(m, p);
            m.wf(s0 + 0x24, f0);
            let v = m.call(0x0014_41a0, &[p], &[]).0;
            let p = m.r32(s0 + 0x34);
            let f13 = (v as i32 as f32) / (m.r32(p + 0x1300) as i32 as f32);
            m.wf(s0 + 0x2c, f13);
            let c = m.r32(p + 0xe00);
            if !((m.r32(c) as i32) > 0) {
                m.call(0x001d_9b60, &[s0, 0], &[]);
                return;
            }
            let f2 = m.rf(s0 + 0x20);
            let f0 = m.rf(s0 + 0x24);
            if f0 < f2 {
                m.call(0x001d_9b60, &[s0, 2], &[]);
                return;
            }
            if f2 < f0 {
                m.call(0x001d_9b60, &[s0, 3], &[]);
                return;
            }
            let f12 = m.rf(s0 + 0x28);
            if m.call(0x0011_d4d0, &[], &[f4(f12), f4(f13), f4(eps)]).0 == 0 {
                m.call(0x001d_9b60, &[s0, 4], &[]);
                return;
            }
            let p = m.r32(s0 + 0x34);
            if m.r32(p + 0x10ec) == 4 {
                m.call(0x001d_9b60, &[s0, 7], &[]);
                return;
            }
            if m.r32(p + 0x10c8) == 0xffff_ffff {
                m.call(0x001d_9b60, &[s0, 1], &[]);
            } else {
                m.call(0x001d_9b60, &[s0, 6], &[]);
            }
        }
        2 => {
            let (x, y) = (m.r32(s0 + 0x38), m.r32(s0 + 0x40));
            if m.call(0x0021_21e8, &[x, y], &[]).0 != 0 {
                return;
            }
            m.call(0x001d_9b60, &[s0, 3], &[]);
        }
        5 => {
            let (x, y) = (m.r32(s0 + 0x38), m.r32(s0 + 0x44));
            if m.call(0x0021_21e8, &[x, y], &[]).0 != 0 {
                return;
            }
            m.call(0x001d_9b60, &[s0, 1], &[]);
        }
        0 => {
            let p = m.r32(s0 + 0x34);
            let f0 = ratio_e00(m, p);
            m.wf(s0 + 0x24, f0);
            let v = m.call(0x0014_41a0, &[p], &[]).0;
            let f3 = m.rf(s0 + 0x24);
            let p = m.r32(s0 + 0x34);
            let f1 = m.rf(s0 + 0x20);
            let f0 = (v as i32 as f32) / (m.r32(p + 0x1300) as i32 as f32);
            m.wf(s0 + 0x2c, f0);
            if !(f1 < f3) {
                return;
            }
            let x = m.r32(s0 + 0x38);
            m.wf(s0 + 0x20, f3);
            m.call(0x0021_2e40, &[x, 0x34f], &[f4(1.0 - f3)]);
            let f0 = m.rf(s0 + 0x2c);
            let x = m.r32(s0 + 0x38);
            m.wf(s0 + 0x28, f0);
            m.call(0x0021_2e40, &[x, 0x350], &[f4(1.0 - f0)]);
            m.call(0x001d_9b60, &[s0, 1], &[]);
        }
        3 | 4 => {
            let (cur, tgt, id) = if st == 3 { (0x20u32, 0x24u32, 0x34fu32) } else { (0x28, 0x2c, 0x350) };
            let f12 = m.rf(s0 + cur);
            let f13 = if st == 3 {
                let p = m.r32(s0 + 0x34);
                ratio_e00(m, p)
            } else {
                let p = m.r32(s0 + 0x34);
                let v = m.call(0x0014_41a0, &[p], &[]).0;
                let p = m.r32(s0 + 0x34);
                (v as i32 as f32) / (m.r32(p + 0x1300) as i32 as f32)
            };
            m.wf(s0 + tgt, f13);
            if f12 < f13 {
                m.wf(s0 + cur, f13);
            } else {
                let f14 = m.rf(0x002d_ded4);
                let r = m.call(0x0011_d018, &[0x002c_3248, 0], &[f4(f12), f4(f13), f4(f14)]).1[0];
                m.wf(s0 + cur, r);
            }
            let f0 = m.rf(s0 + cur);
            let x = m.r32(s0 + 0x38);
            m.call(0x0021_2e40, &[x, id], &[f4(1.0 - f0)]);
            let (a, b) = (m.rf(s0 + cur), m.rf(s0 + tgt));
            if m.call(0x0011_d4d0, &[], &[f4(a), f4(b), f4(eps)]).0 == 0 {
                return;
            }
            m.call(0x001d_9b60, &[s0, 1], &[]);
        }
        _ => {}
    }
}
