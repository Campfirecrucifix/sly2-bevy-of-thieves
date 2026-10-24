//! WIRE-W1H11: 1:1 queue ports (against `QMem`) of a batch of live-frontier natives, ported from the EE asm;
//! callees go through `m.call` / `vslot`. QMem convention: integer / pointer args in `a`, quadword and float
//! args in `v`.
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, clippy::all)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

/// FUN_001a6118: empty.
pub fn ee_sub_1a6118<M: QMem + ?Sized>(m: &mut M, a0: u32) {}

/// FUN_0015a150(o): o+0x148 = DAT_002a514c.
pub fn ee_sub_15a150<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let f = m.r32(0x002a_514c);
    m.w32(a0 + 0x148, f);
}

/// FUN_00185130: tail FUN_00231100(a0).
pub fn ee_sub_185130<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0023_1100, &[a0], &[]);
}

/// FUN_0012e1f0: FUN_0011ccc0(0x3e123c) (tail call: returns the pool block; ANIM-ALL: the result was dropped,
/// so every ACTSEG record of FUN_0012f9b8 was null and no bone got an act).
pub fn ee_sub_12e1f0<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    m.call(0x0011_ccc0, &[0x003e_123c], &[]).0
}

/// FUN_00230380(k): 1 <= k < 4.
pub fn ee_sub_230380<M: QMem + ?Sized>(m: &mut M, a0: i32) -> u32 {
    if a0 >= 4 {
        return 0;
    }
    if a0 > 0 {
        1
    } else {
        0
    }
}

/// FUN_0014fac8(o): FUN_00205eb8(o); o+0x4a4 = 1.
pub fn ee_sub_14fac8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0020_5eb8, &[a0], &[]);
    m.w32(a0 + 0x4a4, 1);
}

/// FUN_001a3580(o): FUN_00231300(o); FUN_001a3968(o).
pub fn ee_sub_1a3580<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0023_1300, &[a0], &[]);
    m.call(0x001a_3968, &[a0], &[]);
}

/// FUN_00211e40(o): FUN_001a5270(o); o+0xb0 = 1.
pub fn ee_sub_211e40<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5270, &[a0], &[]);
    m.w32(a0 + 0xb0, 1);
}

/// FUN_0012eb48(o): FUN_00231100(o); FUN_001a6ca8(o, 0).
pub fn ee_sub_12eb48<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0023_1100, &[a0], &[]);
    m.call(0x001a_6ca8, &[a0, 0], &[]);
}

/// FUN_001c2048(o): FUN_001a5270(o); +0x150 = +0x120 = -1.
pub fn ee_sub_1c2048<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5270, &[a0], &[]);
    m.w32(a0 + 0x150, 0xffff_ffff);
    m.w32(a0 + 0x120, 0xffff_ffff);
}

/// FUN_001efa90(o): FUN_001a5270(o); +0x40 |= 0x40.
pub fn ee_sub_1efa90<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5270, &[a0], &[]);
    let v = m.r32(a0 + 0x40) | 0x40;
    m.w32(a0 + 0x40, v);
}

/// FUN_00185c60(o): FUN_001a5848(o); FUN_0011a2e8(0x3e142c, o).
pub fn ee_sub_185c60<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5848, &[a0], &[]);
    m.call(0x0011_a2e8, &[0x003e_142c, a0], &[]);
}

/// FUN_001836f8(o): FUN_001a6178(o); FUN_0011a220(0x3e1408, o).
pub fn ee_sub_1836f8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_6178, &[a0], &[]);
    m.call(0x0011_a220, &[0x003e_1408, a0], &[]);
}

/// FUN_001c33e8(o): FUN_001a6178(o); FUN_0011a220(0x3e13c0, o).
pub fn ee_sub_1c33e8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_6178, &[a0], &[]);
    m.call(0x0011_a220, &[0x003e_13c0, a0], &[]);
}

/// FUN_001505d8(o): FUN_00258040(o+0x38, 0); FUN_00258040(o+0x34, 0).
pub fn ee_sub_1505d8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0025_8040, &[a0 + 0x38, 0], &[]);
    m.call(0x0025_8040, &[a0 + 0x34, 0], &[]);
}

/// FUN_001dfad8(o): FUN_001a5270(o); +0x58 = -1; FUN_0011a1f8(o+0x5c, 0xe4).
pub fn ee_sub_1dfad8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5270, &[a0], &[]);
    m.w32(a0 + 0x58, 0xffff_ffff);
    m.call(0x0011_a1f8, &[a0 + 0x5c, 0xe4], &[]);
}

/// FUN_001c7e68(o): FUN_001cb518(o); vslot 75 (o, 0).
pub fn ee_sub_1c7e68<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001c_b518, &[a0], &[]);
    vslot(m, a0, 75, &[0], &[]);
}

/// FUN_0015cd68(o): FUN_001a5270(o); +0x120/+0x110/+0x114/+0x11c = -1; DAT_003e1c04 = o.
pub fn ee_sub_15cd68<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5270, &[a0], &[]);
    m.w32(a0 + 0x120, 0xffff_ffff);
    m.w32(a0 + 0x110, 0xffff_ffff);
    m.w32(a0 + 0x114, 0xffff_ffff);
    m.w32(a0 + 0x11c, 0xffff_ffff);
    m.w32(0x003e_1c04, a0);
}

/// FUN_001f8278(x): if !FUN_001f8240(x): FUN_0020a5f8(FUN_001ae4f8(0xac), x).
pub fn ee_sub_1f8278<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    if m.call(0x001f_8240, &[a0], &[]).0 != 0 {
        return;
    }
    let p = m.call(0x001a_e4f8, &[0xac], &[]).0;
    m.call(0x0020_a5f8, &[p, a0], &[]);
}

/// FUN_00184268(o): FUN_00183f08(o); +0xfc = 4; +0xf0/+0x184/+0x18c/+0x194/+0x19c = -1.
pub fn ee_sub_184268<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0018_3f08, &[a0], &[]);
    m.w32(a0 + 0xfc, 4);
    for off in [0xf0u32, 0x184, 0x18c, 0x194, 0x19c] {
        m.w32(a0 + off, 0xffff_ffff);
    }
}

/// FUN_001e0180(o, id, skip): walk list o+0x5c (next +0xe4) for node != skip with *(node+0x28)+0x10 == id.
pub fn ee_sub_1e0180<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) -> u32 {
    let mut p = m.r32(a0 + 0x5c);
    while p != 0 {
        if p != a2 {
            let q = m.r32(p + 0x28);
            if m.r32(q + 0x10) == a1 {
                return p;
            }
        }
        p = m.r32(p + 0xe4);
    }
    0
}

/// FUN_001fd028(o, i): 0 <= i < o+0x18 and idx = (o+0x100)[i] >= 0 -> o+0xfc + idx*4 (else 0).
pub fn ee_sub_1fd028<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: i32) -> u32 {
    if a1 < 0 || a1 >= m.r32(a0 + 0x18) as i32 {
        return 0;
    }
    let t = m.r32(a0 + 0x100);
    let v = m.r32((a1 as u32).wrapping_shl(2).wrapping_add(t)) as i32;
    if v < 0 {
        return 0;
    }
    m.r32(a0 + 0xfc).wrapping_add((v as u32).wrapping_shl(2))
}

/// FUN_0012e6a0(o): FUN_001a5800(o); if o+0x124: FUN_0014e328(0x2ddf40, 10, 0x2de3e8, 0, o).
pub fn ee_sub_12e6a0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5800, &[a0], &[]);
    if m.r32(a0 + 0x124) != 0 {
        m.call(0x0014_e328, &[0x002d_df40, 10, 0x002d_e3e8, 0, a0], &[]);
    }
}

/// FUN_001bbea8(o): FUN_001cb518(o); FUN_001baf18(o, 0x609); if o+0xe8: FUN_001f5720(o,1); FUN_001f57c0(o,1).
pub fn ee_sub_1bbea8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001c_b518, &[a0], &[]);
    m.call(0x001b_af18, &[a0, 0x609], &[]);
    if m.r32(a0 + 0xe8) != 0 {
        m.call(0x001f_5720, &[a0, 1], &[]);
        m.call(0x001f_57c0, &[a0, 1], &[]);
    }
}

/// FUN_00117d00(s): read one signed byte from stream s (buffered +4/+8, else FUN_00117968(s, 1, &b)).
pub fn ee_sub_117d00<M: QMem + ?Sized>(m: &mut M, a0: u32) -> i32 {
    let n = m.r32(a0 + 8) as i32;
    if n > 0 {
        let p = m.r32(a0 + 4);
        let b = m.r8(p) as i8 as i32;
        m.w32(a0 + 8, (n - 1) as u32);
        m.w32(a0 + 4, p.wrapping_add(1));
        b
    } else {
        let buf = stack_push(m, 0x10);
        m.call(0x0011_7968, &[a0, 1, buf], &[]);
        let b = m.r8(buf) as i8 as i32;
        stack_pop(m, 0x10);
        b
    }
}

/// FUN_001b0d08(o): FUN_001f36a8(o); FUN_001b1bf0(o); FUN_001b1a38(o,+0x18c,o+0x1cc); FUN_001b1a38(o,+0x1b0,o+0x1d0);
/// FUN_001b0d90(o).
pub fn ee_sub_1b0d08<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001f_36a8, &[a0], &[]);
    m.call(0x001b_1bf0, &[a0], &[]);
    let x = m.r32(a0 + 0x18c);
    m.call(0x001b_1a38, &[a0, x, a0 + 0x1cc], &[]);
    let x = m.r32(a0 + 0x1b0);
    m.call(0x001b_1a38, &[a0, x, a0 + 0x1d0], &[]);
    m.call(0x001b_0d90, &[a0], &[]);
}

/// FUN_001618f8(o, t): if o+0x3bc4 = p: p+0x8c = 0; vslot 48 (p, -1). o+0x476c = DAT_002dded0 + t.
pub fn ee_sub_1618f8<M: QMem + ?Sized>(m: &mut M, a0: u32, f12: f32) {
    let p = m.r32(a0 + 0x3bc4);
    if p != 0 {
        m.w32(p + 0x8c, 0);
        vslot(m, p, 48, &[0xffff_ffff], &[]);
    }
    let c = m.rf(0x002d_ded0);
    m.wf(a0 + 0x476c, c + f12);
}

/// FUN_001c82f0(o): FUN_001f36a8(o); FUN_001a64e8(o, 0x18, 0x2c0ec8); +0x204 = FUN_001f4c58(o, 0x645, 1);
/// FUN_001c8730(o, FUN_001c8110() ? 5 : 0).
pub fn ee_sub_1c82f0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001f_36a8, &[a0], &[]);
    m.call(0x001a_64e8, &[a0, 0x18, 0x002c_0ec8], &[]);
    let v = m.call(0x001f_4c58, &[a0, 0x645, 1], &[]).0;
    m.w32(a0 + 0x204, v);
    let r = m.call(0x001c_8110, &[], &[]).0;
    m.call(0x001c_8730, &[a0, if r == 0 { 0 } else { 5 }], &[]);
}

/// FUN_002281e8(o): FUN_00231100(o); +0x150 = 0x189; +0x154 = -1; FUN_001a6ca8(o, 0); set bit 2 in +0x40
/// (inline when tagged odd, else in *(+0x40)).
pub fn ee_sub_2281e8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0023_1100, &[a0], &[]);
    m.w32(a0 + 0x150, 0x189);
    m.w32(a0 + 0x154, 0xffff_ffff);
    m.call(0x001a_6ca8, &[a0, 0], &[]);
    let v = m.r32(a0 + 0x40);
    if (v ^ 1) & 1 != 0 {
        let w = m.r32(v) | 2;
        m.w32(v, w);
    } else {
        m.w32(a0 + 0x40, v | 2);
    }
}

/// FUN_001c7778(o): FUN_001c5e48(o); if !o+0x150: o+0x238 -= DAT_002dded4; if <= 0: vslot 75 (o, 5).
pub fn ee_sub_1c7778<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001c_5e48, &[a0], &[]);
    if m.r32(a0 + 0x150) != 0 {
        return;
    }
    let f = m.rf(a0 + 0x238) - m.rf(0x002d_ded4);
    m.wf(a0 + 0x238, f);
    if f <= 0.0 {
        vslot(m, a0, 75, &[5], &[]);
    }
}

/// FUN_001805c8(o): FUN_0017f138(o); +0x1d8 = FUN_001e94b0(0x2c9, 0x10); FUN_002303a0(o, +0x58/+0x5c, 2) when set;
/// FUN_0017f408(o, 5, o+0x220).
pub fn ee_sub_1805c8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0017_f138, &[a0], &[]);
    let v = m.call(0x001e_94b0, &[0x2c9, 0x10], &[]).0;
    m.w32(a0 + 0x1d8, v);
    let x = m.r32(a0 + 0x58);
    if x != 0 {
        m.call(0x0023_03a0, &[a0, x, 2], &[]);
    }
    let x = m.r32(a0 + 0x5c);
    if x != 0 {
        m.call(0x0023_03a0, &[a0, x, 2], &[]);
    }
    m.call(0x0017_f408, &[a0, 5, a0 + 0x220], &[]);
}

/// FUN_00182f28(o): FUN_001a5c48(o); if +0x190: FUN_00221268(+0x190, 0); if +0x38 == 1e10:
/// +0x38 = FUN_001e7d90(o) + DAT_002be8b4.
pub fn ee_sub_182f28<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5c48, &[a0], &[]);
    let p = m.r32(a0 + 0x190);
    if p != 0 {
        m.call(0x0022_1268, &[p, 0], &[]);
    }
    if m.rf(a0 + 0x38) == f32::from_bits(0x5015_02f9) {
        let f = m.call(0x001e_7d90, &[a0], &[]).1[0];
        let c = m.rf(0x002b_e8b4);
        m.wf(a0 + 0x38, f + c);
    }
}

/// FUN_001725e0(o, x): if x != o+0x44: old -> vslot 1; o+0x44 = x; x -> vslot 0; o+0x48 = 1.
pub fn ee_sub_1725e0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let old = m.r32(a0 + 0x44);
    if a1 == old {
        return;
    }
    if old != 0 {
        vslot(m, old, 1, &[], &[]);
    }
    m.w32(a0 + 0x44, a1);
    if a1 != 0 {
        vslot(m, a1, 0, &[], &[]);
    }
    m.w32(a0 + 0x48, 1);
}

/// FUN_001391c8(o): FUN_00231100(o); FUN_0011a1f8(o+0x448, 0x1090); +0x18 = -1; +0x38 = 1e10;
/// FUN_001a6ca8(o, 1); flags +0x40 = (flags & 1) | 0xe (inline when tagged odd, else in *(+0x40)).
pub fn ee_sub_1391c8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0023_1100, &[a0], &[]);
    m.call(0x0011_a1f8, &[a0 + 0x448, 0x1090], &[]);
    m.w32(a0 + 0x18, 0xffff_ffff);
    m.w32(a0 + 0x38, 0x5015_02f9);
    m.call(0x001a_6ca8, &[a0, 1], &[]);
    let v = m.r32(a0 + 0x40);
    if (v ^ 1) & 1 != 0 {
        let w = (m.r32(v) & 1) | 0xe;
        m.w32(v, w);
    } else {
        m.w32(a0 + 0x40, (v & 1) | 0xe);
    }
}

/// FUN_00224258(o, msg): state +0x15c in {0x1a, 0x29} -> vslot 75 (o, 3); else FUN_001a58d0(o, msg+0x18==1 ? 0 : msg, 3);
/// vslot 75 (o, 2).
pub fn ee_sub_224258<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let st = m.r32(a0 + 0x15c);
    if st == 0x1a || st == 0x29 {
        vslot(m, a0, 75, &[3], &[]);
    } else {
        let x = if m.r32(a1 + 0x18) ^ 1 == 0 { 0 } else { a1 };
        m.call(0x001a_58d0, &[a0, x, 3], &[]);
        vslot(m, a0, 75, &[2], &[]);
    }
}

/// FUN_001fbf20(o, _, vol, _, pitch): flags +0x50 & 0x70 -> FUN_00232500(+0x40, mask, vol, +0x14, pitch, 0)
/// (0x10: vol = +0x44*1024; 0x20: |4; 0x40: |8, pitch = +0x18*12).
pub fn ee_sub_1fbf20<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, t0: u32) {
    let fl = m.r16(a0 + 0x50) as u32;
    if fl & 0x70 == 0 {
        return;
    }
    let mut a2 = a2;
    let mut t0 = t0;
    let mut mask = 0u32;
    if fl & 0x10 != 0 {
        mask = 1;
        a2 = (m.rf(a0 + 0x44) * 1024.0) as i32 as u32;
    }
    if fl & 0x20 != 0 {
        mask |= 4;
    }
    if fl & 0x40 != 0 {
        mask |= 8;
        t0 = (m.rf(a0 + 0x18) * 12.0) as i32 as u32;
    }
    let a3 = m.r32(a0 + 0x14);
    let a = m.r32(a0 + 0x40);
    m.call(0x0023_2500, &[a, mask, a2, a3, t0, 0], &[]);
}

/// FUN_001b9f80(o): FUN_00205eb8(o); tuning constants at +0x1070..; +0x364 = +0xe64 = 0; *(+0xf64)+0xf8 = 3;
/// +0x4a4 = 1.
pub fn ee_sub_1b9f80<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0020_5eb8, &[a0], &[]);
    m.w32(a0 + 0x1070, 0x3fac_cccd);
    m.w32(a0 + 0x1074, 0xb99d_4952);
    m.w32(a0 + 0x1078, 0);
    let p = m.r32(a0 + 0xf64);
    m.w32(a0 + 0x1080, 0x3f66_6666);
    m.w32(a0 + 0x1084, 0x3f99_999a);
    m.w32(a0 + 0x1088, 0x40e0_0000);
    m.w32(a0 + 0x364, 0);
    m.w32(a0 + 0xe64, 0);
    m.w32(p + 0xf8, 3);
    m.w32(a0 + 0x4a4, 1);
}

/// `vsqrt` of x*x + y*y + z*z (the `vmul.xyz / vaddw / vadday / vmaddz / vsqrt` idiom)
fn len3(a: V4) -> f32 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

/// FUN_001c4e50(o): FUN_001f2108(o); FUN_001f7918(o, 0x17e, 8); field init; flags +0x40 |= 2
/// (inline when tagged odd, else in *(+0x40)).
pub fn ee_sub_1c4e50<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001f_2108, &[a0], &[]);
    m.call(0x001f_7918, &[a0, 0x17e, 8], &[]);
    let fl = m.r32(a0 + 0x40);
    let c = m.r32(0x002a_7710);
    m.w32(a0 + 0x1c8, 1);
    m.w32(a0 + 0x21c, 0x41f0_0000);
    m.w32(a0 + 0x1cc, c);
    m.w32(a0 + 0x224, 0xffff_ffff);
    m.w32(a0 + 0x150, 0xffff_ffff);
    m.w32(a0 + 0x158, 0);
    for off in [0x15cu32, 0x1a4, 0x1ac, 0x1b4, 0x204, 0x20c, 0x214] {
        m.w32(a0 + off, 0xffff_ffff);
    }
    if (fl ^ 1) & 1 != 0 {
        let w = m.r32(fl) | 2;
        m.w32(fl, w);
    } else {
        m.w32(a0 + 0x40, fl | 2);
    }
}

/// FUN_001b0f88(o, msg): r = FUN_001f7588(o, msg); r && msg+0x44 & 8 && o+0x220 != 5 -> FUN_001b1328(o, 5);
/// p = o+0x1c8 -> vslot 50 (p, &{6, msg+0x30 qword, msg+0x20 qword}). Returns r.
pub fn ee_sub_1b0f88<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let r = m.call(0x001f_7588, &[a0, a1], &[]).0;
    if r == 0 {
        return r;
    }
    if m.r32(a1 + 0x44) & 8 == 0 {
        return r;
    }
    if m.r32(a0 + 0x220) == 5 {
        return r;
    }
    m.call(0x001b_1328, &[a0, 5], &[]);
    let p = m.r32(a0 + 0x1c8);
    if p != 0 {
        let sp = stack_push(m, 0x30);
        let v1 = m.rv(a1 + 0x30);
        let v0 = m.rv(a1 + 0x20);
        m.w32(sp, 6);
        m.wv(sp + 0x20, v0);
        m.wv(sp + 0x10, v1);
        vslot(m, p, 50, &[sp], &[]);
        stack_pop(m, 0x30);
    }
    r
}

/// FUN_0013ab58(o): FUN_001f36e0(o); if +0x154 > T_NOW and |vslot 32 (o)| >= +0x158: +0x1f4 -> FUN_0013ac18(o);
/// else FUN_0013b860(o).
pub fn ee_sub_13ab58<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001f_36e0, &[a0], &[]);
    let a = m.rf(a0 + 0x154);
    let t = m.rf(0x002d_ded0);
    let mut near = true;
    if !(a <= t) {
        let v = vslot(m, a0, 32, &[], &[]).1;
        let lim = m.rf(a0 + 0x158);
        near = len3(v) < lim;
        if !near {
            if m.r32(a0 + 0x1f4) != 0 {
                m.call(0x0013_ac18, &[a0], &[]);
            }
            return;
        }
    }
    m.call(0x0013_b860, &[a0], &[]);
}

/// FUN_001b7ae8(o): FUN_001b7d20(o); t = *(o+0x13f0)+0x180; o+0x13f0 = 0; if t+0x28 bit 60:
/// FUN_00132b10(t, 0x38) -> FUN_00209fb8(t); vslot 34 (t, FUN_001a6fc0(o, 0, qword 0x2c09d0)); vslot 35 (t, zero qword);
/// FUN_001f7a68(t, 3).
pub fn ee_sub_1b7ae8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001b_7d20, &[a0], &[]);
    let q = m.r32(a0 + 0x13f0);
    let t = m.r32(q + 0x180);
    m.w32(a0 + 0x13f0, 0);
    if (m.r64(t + 0x28) >> 60) & 1 == 0 {
        return;
    }
    if m.call(0x0013_2b10, &[t, 0x38], &[]).0 != 0 {
        m.call(0x0020_9fb8, &[t], &[]);
    }
    let c = m.rv(0x002c_09d0);
    let r = m.call(0x001a_6fc0, &[a0, 0], &[c]).1;
    vslot(m, t, 34, &[], &[r]);
    let z = m.rv(0x002a_2dd0);
    vslot(m, t, 35, &[], &[z]);
    m.call(0x001f_7a68, &[t, 3], &[]);
}

/// FUN_00129df8(msg): counter bump on msg kind (jump table 0x2a4470, kinds 0x1a..0x22) in stats msg[1];
/// 0x21 -> vslot 0 (msg[1]).
pub fn ee_sub_129df8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let k = m.r32(a0).wrapping_sub(0x1a);
    if k >= 9 {
        return;
    }
    let p = m.r32(a0 + 4);
    let off = match k {
        0 | 2 => 0x10,
        1 => 0x40,
        3 => 8,
        4 => 0xc,
        5 | 6 | 8 => 0,
        _ => {
            if p != 0 {
                vslot(m, p, 0, &[], &[]);
            }
            return;
        }
    };
    let v = m.r32(p + off).wrapping_add(1);
    m.w32(p + off, v);
}

/// FUN_001a15d0(o): take the pending pmf at o+0x1c (cleared via FUN_002333b4(o+0x1c, 0, 8)); if set call it
/// (o + adj); else kind o+4 in 8..12 -> vslot 92 (*o); else +0x30 with +0x148 bit 33 -> FUN_001a16a8(o);
/// else FUN_00195a00(*o+0x2c0, 0xb, 0, 0).
pub fn ee_sub_1a15d0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let adj = m.r16(a0 + 0x1c) as i16 as i32;
    let hi = m.r16(a0 + 0x1e);
    let f = m.r32(a0 + 0x20);
    m.call(0x0023_33b4, &[a0 + 0x1c, 0, 8], &[]);
    if hi != 0 {
        m.call(f, &[a0.wrapping_add(adj as u32)], &[]);
        return;
    }
    if m.r32(a0 + 4).wrapping_sub(8) < 5 {
        let o = m.r32(a0);
        vslot(m, o, 92, &[], &[]);
        return;
    }
    let p = m.r32(a0 + 0x30);
    if p != 0 && m.r64(p + 0x148) & (1u64 << 33) != 0 {
        m.call(0x001a_16a8, &[a0], &[]);
        return;
    }
    let o = m.r32(a0);
    m.call(0x0019_5a00, &[o.wrapping_add(0x2c0), 0xb, 0, 0], &[]);
}

/// FUN_00148f20(o, msg): r = FUN_001f7588(o, msg); r && !(msg+0x44 & 4) && FUN_00149100(o, *msg) ->
/// mode +0x2a8: 1 & !+0x1e0 -> FUN_001490a0(o, 1); 2: +0x1e0 == 0 & T_NOW - +0x1e4 >= +0x1e8 -> FUN_001490a0(o, 1),
/// +0x1e0 == 1 -> FUN_001490a0(o, 0). Returns r.
pub fn ee_sub_148f20<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let r = m.call(0x001f_7588, &[a0, a1], &[]).0;
    if r == 0 || m.r32(a1 + 0x44) & 4 != 0 {
        return r;
    }
    let k = m.r32(a1);
    if m.call(0x0014_9100, &[a0, k], &[]).0 == 0 {
        return r;
    }
    let mode = m.r32(a0 + 0x2a8);
    if mode == 1 {
        if m.r32(a0 + 0x1e0) != 0 {
            return r;
        }
        m.call(0x0014_90a0, &[a0, 1], &[]);
    } else if mode == 2 {
        let s = m.r32(a0 + 0x1e0);
        if s == 0 {
            let t = m.rf(0x002d_ded0) - m.rf(a0 + 0x1e4);
            if m.rf(a0 + 0x1e8) <= t {
                m.call(0x0014_90a0, &[a0, 1], &[]);
            }
        } else if s == 1 {
            m.call(0x0014_90a0, &[a0, 0], &[]);
        }
    }
    r
}

/// FUN_0017a8c8(o, st): state change -> FUN_001ef0d8(o+0x18); +0x30 = st; +0x34 = T_NOW; st == -1:
/// FUN_001ef038(o), *(+0x10)+0x32c = 0; -1 <= st < 5: p = FUN_00205438(+0x10, 0x9d); p+0x3c = table 0x2a55a0[st];
/// vslot 7 (p); *(+0x10)+0x32c = (u32)st < 2.
pub fn ee_sub_17a8c8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: i32) {
    if m.r32(a0 + 0x30) as i32 == a1 {
        return;
    }
    m.call(0x001e_f0d8, &[a0 + 0x18], &[]);
    m.w32(a0 + 0x30, a1 as u32);
    let t = m.r32(0x002d_ded0);
    m.w32(a0 + 0x34, t);
    if a1 == -1 {
        m.call(0x001e_f038, &[a0], &[]);
        let q = m.r32(a0 + 0x10);
        m.w32(q + 0x32c, 0);
        return;
    }
    if a1 < -1 || a1 >= 5 {
        return;
    }
    let q = m.r32(a0 + 0x10);
    let p = m.call(0x0020_5438, &[q, 0x9d], &[]).0;
    let k = m.r32(a0 + 0x30);
    let sp = stack_push(m, 0x20);
    for i in 0..5u32 {
        let w = m.r32(0x002a_55a0 + 4 * i);
        m.w32(sp + 4 * i, w);
    }
    let v = m.r32(sp.wrapping_add(k.wrapping_shl(2)));
    stack_pop(m, 0x20);
    m.w32(p + 0x3c, v);
    vslot(m, p, 7, &[], &[]);
    let k = m.r32(a0 + 0x30);
    let q = m.r32(a0 + 0x10);
    m.w32(q + 0x32c, (k < 2) as u32);
}

/// FUN_00152148(o): FUN_00203fc0(o); (!+0x408 || FUN_00131968(+0x408)) -> FUN_002303a0(o, +0x58/+0x5c, 1),
/// +0x40c = 0, FUN_001524a0(o, +0x2c0, 8); mode = DAT_002de570 ? *(..+0x50) : 0: 0 & !+0x40c ->
/// FUN_00152390(o, -1, -1, 0); 2 -> FUN_001533b0(DAT_002de570, o); p = +0x408 -> o+0x190 = -FUN_00204e38(o, p, p+0x1c).
pub fn ee_sub_152148<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0020_3fc0, &[a0], &[]);
    let p = m.r32(a0 + 0x408);
    let go = p == 0 || m.call(0x0013_1968, &[p], &[]).0 != 0;
    if go {
        let x = m.r32(a0 + 0x58);
        m.call(0x0023_03a0, &[a0, x, 1], &[]);
        let x = m.r32(a0 + 0x5c);
        m.call(0x0023_03a0, &[a0, x, 1], &[]);
        m.w32(a0 + 0x40c, 0);
        let x = m.r32(a0 + 0x2c0);
        m.call(0x0015_24a0, &[a0, x, 8], &[]);
    }
    let g = m.r32(0x002d_e570);
    let mode = if g != 0 { m.r32(g + 0x50) } else { 0 };
    if mode == 0 {
        if m.r32(a0 + 0x40c) == 0 {
            m.call(0x0015_2390, &[a0, 0xffff_ffff, 0xffff_ffff, 0], &[]);
        }
    } else if mode == 2 {
        m.call(0x0015_33b0, &[g, a0], &[]);
    }
    let p = m.r32(a0 + 0x408);
    if p != 0 {
        let f = m.rf(p + 0x1c);
        let r = m.call(0x0020_4e38, &[a0, p], &[f4(f)]).1;
        m.wv(a0 + 0x190, [r[0] * -1.0, r[1] * -1.0, r[2] * -1.0, r[3] * -1.0]);
    }
}

/// FUN_00149b58(o, mask, id, t0 = key3, a3 = 64-bit key2 as [lo, hi] quadword): unlink every node of the
/// o+0x3ca8 list (next +0x10) matching the masked keys (1: +0 == id; 2: s16 +6 == key2.h16 and (s16 +8 ==
/// key2.h32 low half unless h16 == -1, else +8 == key2.h32); 4: +0xc == key3), FUN_0011cd98(o+0x3c90, node)
/// each; empty list -> +0x3cac = 0.
pub fn ee_sub_149b58<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, t0: u32, a3: V4) {
    let lo = a3[0].to_bits();
    let hi = a3[1].to_bits();
    let h16 = (lo >> 16) as u16 as i16 as i32;
    let l16 = hi as u16 as i16 as i32;
    let mut s0 = a0 + 0x3ca8;
    let mut n = m.r32(a0 + 0x3ca8);
    while n != 0 {
        let mut ok = true;
        if a1 & 1 != 0 && a2 != m.r32(n) {
            ok = false;
        }
        if ok && a1 & 2 != 0 {
            if h16 != m.r16(n + 6) as i16 as i32 {
                ok = false;
            } else if !(h16 != -1 && l16 == m.r16(n + 8) as i16 as i32) {
                if hi != m.r32(n + 8) {
                    ok = false;
                }
            }
        }
        if ok && a1 & 4 != 0 && t0 != m.r32(n + 0xc) {
            ok = false;
        }
        if ok {
            let nx = m.r32(n + 0x10);
            m.w32(s0, nx);
            m.call(0x0011_cd98, &[a0 + 0x3c90, n], &[]);
        } else {
            s0 = n + 0x10;
        }
        n = m.r32(s0);
    }
    if m.r32(a0 + 0x3ca8) == 0 {
        m.w32(a0 + 0x3cac, 0);
    }
}

/// FUN_001828a8(o, msg): msg+0x44 & 8 -> n = FUN_00182810(o, msg+0x20 qword); n: n+0x4c += msg+0x10; <= 0 ->
/// FUN_0015c198(+0x190, 4, 0, 0, 0, zero, n qword) when +0x190, FUN_00181eb0(o, n, 2); else n+0x58 =
/// dot(msg+0x30, K(0x2a2dc0) x n+0x10) * DAT_002be8a8, n+0x5c = FUN_0011d448(-15, 15), FUN_00181eb0(o, n, 1).
pub fn ee_sub_1828a8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    if m.r32(a1 + 0x44) & 8 == 0 {
        return 0;
    }
    let q = m.rv(a1 + 0x20);
    let n = m.call(0x0018_2810, &[a0], &[q]).0;
    if n == 0 {
        return 1;
    }
    let v = m.r32(n + 0x4c).wrapping_add(m.r32(a1 + 0x10));
    m.w32(n + 0x4c, v);
    if v as i32 > 0 {
        let b = m.rv(n + 0x10);
        let d = m.rv(a1 + 0x30);
        let a = m.rv(0x002a_2dc0);
        let c = [a[1] * b[2] - b[1] * a[2], a[2] * b[0] - b[2] * a[0], a[0] * b[1] - b[0] * a[1]];
        let dot = d[0] * c[0] + d[1] * c[1] + d[2] * c[2];
        let k = m.rf(0x002b_e8a8);
        m.wf(n + 0x58, dot * k);
        let r = m.call(0x0011_d448, &[], &[f4(-15.0), f4(15.0)]).1[0];
        m.wf(n + 0x5c, r);
        m.call(0x0018_1eb0, &[a0, n, 1], &[]);
        return 1;
    }
    let p = m.r32(a0 + 0x190);
    if p != 0 {
        let z = m.rv(0x002a_2dd0);
        let nq = m.rv(n);
        m.call(0x0015_c198, &[p, 4, 0], &[f4(0.0), f4(0.0), z, nq]);
    }
    m.call(0x0018_1eb0, &[a0, n, 2], &[]);
    1
}

/// FUN_001c5e48(o): FUN_001f36e0(o); FUN_001c6308(o); state +0x150 (jump table 0x2a7720):
/// 0: |+0x180|^2 > 10000 -> FUN_001c55e0(o); then (and 5) FUN_001c5d18(o); 3: +0x188 < 0 -> vslot 75 (o, 4);
/// 4: +0x1c4 && +0x188 < 25 -> +0x1c4 = 0, vslot 75 (o, 0); else T_NOW - +0x154 > DAT_002c0d50 -> vslot 75 (o, 2),
/// else FUN_001c55e0(o, 2), FUN_001c5d18(o).
pub fn ee_sub_1c5e48<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001f_36e0, &[a0], &[]);
    m.call(0x001c_6308, &[a0], &[]);
    match m.r32(a0 + 0x150) {
        0 => {
            let v = m.rv(a0 + 0x180);
            let d = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
            if 10000.0 < d {
                m.call(0x001c_55e0, &[a0], &[]);
            }
            m.call(0x001c_5d18, &[a0], &[]);
        }
        5 => {
            m.call(0x001c_5d18, &[a0], &[]);
        }
        3 => {
            if m.rf(a0 + 0x188) < 0.0 {
                vslot(m, a0, 75, &[4], &[]);
            }
        }
        4 => {
            if m.r32(a0 + 0x1c4) != 0 && m.rf(a0 + 0x188) < 25.0 {
                m.w32(a0 + 0x1c4, 0);
                vslot(m, a0, 75, &[0], &[]);
                return;
            }
            let t = m.rf(0x002d_ded0) - m.rf(a0 + 0x154);
            if m.rf(0x002c_0d50) < t {
                vslot(m, a0, 75, &[2], &[]);
            } else {
                m.call(0x001c_55e0, &[a0, 2], &[]);
                m.call(0x001c_5d18, &[a0], &[]);
            }
        }
        _ => {}
    }
}

/// FUN_0015cda8(o): FUN_002123c8(o); +0x118 = 8; unless DAT_002be26c: walk +0x118 down (>= 2) while the save
/// lookups FUN_002581c0(&buf, 0x3e0fe8[k], 0x3d4a60) give 0.0 and 0x3e100c[k-1]'s value < 100 (FUN_00129ec8(&buf)
/// after each); DAT_002be270 == -1 -> +0x110 = +0x118, FUN_0015d648(o, 0, -1); else +0x128 = it, +0x110 = 0,
/// DAT_002be270 = -1, FUN_0015d648(o, 8, 0). FUN_00187e78(0x2dfc00).
pub fn ee_sub_15cda8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0021_23c8, &[a0], &[]);
    m.w32(a0 + 0x118, 8);
    if m.r32(0x002b_e26c) == 0 {
        let buf = stack_push(m, 0x10);
        loop {
            let k = m.r32(a0 + 0x118);
            let key = m.r32(k.wrapping_shl(2).wrapping_add(0x003e_0fe8));
            m.call(0x0025_81c0, &[buf, key, 0x003d_4a60], &[]);
            let f20 = m.rf(buf + 4);
            m.call(0x0012_9ec8, &[buf], &[]);
            if f20 != 0.0 {
                break;
            }
            let k = m.r32(a0 + 0x118).wrapping_sub(1);
            let key = m.r32(k.wrapping_shl(2).wrapping_add(0x003e_100c));
            let mut s0 = 0i32;
            if key != 0 {
                m.call(0x0025_81c0, &[buf, key, 0x003d_4a60], &[]);
                s0 = m.r32(buf + 4) as i32;
                m.call(0x0012_9ec8, &[buf], &[]);
            }
            if s0 >= 100 {
                break;
            }
            let k = m.r32(a0 + 0x118).wrapping_sub(1);
            m.w32(a0 + 0x118, k);
            if (k as i32) < 2 {
                break;
            }
            if m.r32(0x002b_e26c) != 0 {
                break;
            }
        }
        stack_pop(m, 0x10);
    }
    let v = m.r32(0x002b_e270);
    if v == 0xffff_ffff {
        let k = m.r32(a0 + 0x118);
        m.w32(a0 + 0x110, k);
        m.call(0x0015_d648, &[a0, 0, 0xffff_ffff], &[]);
    } else {
        m.w32(a0 + 0x128, v);
        m.w32(a0 + 0x110, 0);
        m.w32(0x002b_e270, 0xffff_ffff);
        m.call(0x0015_d648, &[a0, 8, 0], &[]);
    }
    m.call(0x0018_7e78, &[0x002d_fc00], &[]);
}

/// FUN_00181840(o): for each 0x30-byte entry e of o+0x1cc (count +0x1c8): e+0x24 = FUN_0015e938(GS, 4, *e, o);
/// null -> drop (count--, revisit slot); else FUN_0011a1f8(e+0x10, 0x60) and e+4 times allocate a node
/// FUN_001ae2a0(&{0x70, 0x10}) (+0x20 = e, +0x24 = -1) appended by FUN_0011a220(e+0x10, node).
/// FUN_0017f138(o); push a 0xb0 record into +0xe4[+0xe0++] = {+0x60, o, .., -1, flags |= 3 << 48};
/// +0x28 bit 60 -> FUN_001f2b20(o).
pub fn ee_sub_181840<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let mut i: i32 = 0;
    if m.r32(a0 + 0x1c8) as i32 > 0 {
        let mut s0 = a0 + 0x1cc;
        let sp = stack_push(m, 0x10);
        loop {
            let s3 = s0;
            let k = m.r32(s0);
            let v = m.call(0x0015_e938, &[0x003d_d490, 4, k, a0], &[]).0;
            m.w32(s0 + 0x24, v);
            if v == 0 {
                s0 = s0.wrapping_sub(0x30);
                i -= 1;
                let c = m.r32(a0 + 0x1c8).wrapping_sub(1);
                m.w32(a0 + 0x1c8, c);
            } else {
                let s5 = s0 + 0x10;
                m.call(0x0011_a1f8, &[s5, 0x60], &[]);
                let mut s1: i32 = 0;
                if m.r32(s0 + 4) as i32 > 0 {
                    loop {
                        m.w32(sp, 0x70);
                        m.w32(sp + 4, 0x10);
                        s1 += 1;
                        let r = m.call(0x001a_e2a0, &[sp], &[]).0;
                        m.w32(r + 0x20, s3);
                        m.w32(r + 0x24, 0xffff_ffff);
                        m.call(0x0011_a220, &[s5, r], &[]);
                        if s1 >= m.r32(s3 + 4) as i32 {
                            break;
                        }
                    }
                }
            }
            i += 1;
            if i >= m.r32(a0 + 0x1c8) as i32 {
                break;
            }
            s0 = s0.wrapping_add(0x30);
        }
        stack_pop(m, 0x10);
    }
    m.call(0x0017_f138, &[a0], &[]);
    let k = m.r32(a0 + 0xe0);
    let base = m.r32(a0 + 0xe4);
    let id = m.r32(a0 + 0x60);
    m.w32(a0 + 0xe0, k.wrapping_add(1));
    let p = base.wrapping_add(k.wrapping_mul(0xb0));
    m.w32(p, id);
    m.w32(p + 0xc, 0xffff_ffff);
    m.w32(p + 4, a0);
    let h = m.r32(p + 0x14) | 0x3_0000;
    m.w32(p + 0x14, h);
    if (m.r64(a0 + 0x28) >> 60) & 1 != 0 {
        m.call(0x001f_2b20, &[a0], &[]);
    }
}

/// FUN_0017f1d0(o, msg): msg+0x44 & 8 -> +0x184 += msg+0x10; <= 0 -> vslot 81 (o, 0); else with target +0xe8
/// (state +0xc0 != 3): dir = unit(horizontal part (w.r.t. axis 0x2a2dc0) of FUN_001a62e8(o) - msg+0x20) (or
/// qword 0x2a2da0 when < 1e-4); push = clamp(+0x1a0 + n*(+0x1a4 + n*+0x1a8), +0x1b0, +0x1b4), n = -msg+0x10;
/// vslot 34 (o, FUN_001fa810(+0xe8+0xb0, +0x58, FUN_0011e9f8(+0x1b4, vslot 32 (o) + dir*push))). Returns handled.
pub fn ee_sub_17f1d0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    if m.r32(a1 + 0x44) & 8 == 0 {
        return 0;
    }
    let v = m.r32(a0 + 0x184).wrapping_add(m.r32(a1 + 0x10));
    m.w32(a0 + 0x184, v);
    if v as i32 <= 0 {
        vslot(m, a0, 81, &[0], &[]);
        return 1;
    }
    let t = m.r32(a0 + 0xe8);
    if t == 0 || m.r32(t + 0xc0) == 3 {
        return 1;
    }
    let a = m.call(0x001a_62e8, &[a0], &[]).1;
    let b = m.rv(a1 + 0x20);
    let mut d = [a[0] - b[0], a[1] - b[1], a[2] - b[2], a[3] - b[3]];
    let k = m.rv(0x002a_2dc0);
    let dot = d[0] * k[0] + d[1] * k[1] + d[2] * k[2];
    for i in 0..4 {
        d[i] -= k[i] * dot;
    }
    let len = len3(d);
    let dir = if len < f32::from_bits(0x38d1_b717) {
        m.rv(0x002a_2da0)
    } else {
        let q = 1.0 / len;
        [d[0] * q, d[1] * q, d[2] * q, d[3] * q]
    };
    let n = (m.r32(a1 + 0x10) as i32).wrapping_neg() as f32;
    let c2 = m.rf(a0 + 0x1a8);
    let c1 = m.rf(a0 + 0x1a4);
    let c0 = m.rf(a0 + 0x1a0);
    let lo = m.rf(a0 + 0x1b0);
    let f3 = c0 + n * (c1 + n * c2);
    let hi = m.rf(a0 + 0x1b4);
    let push = if f3 < lo {
        lo
    } else if hi < f3 {
        hi
    } else {
        f3
    };
    let r = vslot(m, a0, 32, &[], &[]).1;
    let p = [
        r[0] + dir[0] * push,
        r[1] + dir[1] * push,
        r[2] + dir[2] * push,
        r[3] + 1.0 + dir[3] * push,
    ];
    let lim = m.rf(a0 + 0x1b4);
    let q = m.call(0x0011_e9f8, &[], &[f4(lim), p]).1;
    let t = m.r32(a0 + 0xe8);
    let x = m.r32(a0 + 0x58);
    let q = m.call(0x001f_a810, &[t.wrapping_add(0xb0), x], &[q]).1;
    vslot(m, a0, 34, &[], &[q]);
    1
}

/// FUN_001428d8(o, o2, focus, out_d, out_h): d = FUN_001a62e8(o) - FUN_001a62e8(o2); horizontal distance
/// |d.xy| and height d.z, plus focus (o == *focus) quadratic biases (0x2bd280 / 0x2bd290) and, with +0x150 and
/// dist > 1e-4, a facing term clamp(quad(dot(FUN_0011e1a0(+0x150+0x18, +0x1c, 0), d)/dist), 0x2bbb60..64) *
/// quad(dist) (0x2bd2a0 / 0x2bd2b0); writes the results to the non-null outs.
pub fn ee_sub_1428d8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, t0: u32) {
    let a = m.call(0x001a_62e8, &[a0], &[]).1;
    let mut f23 = 0.0f32;
    let b = m.call(0x001a_62e8, &[a1], &[]).1;
    let mut f22 = f23;
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2], a[3] - b[3]];
    let pv = m.r32(a2);
    let mut f20 = (d[0] * d[0] + d[1] * d[1]).sqrt();
    let mut f21 = d[2];
    if a0 == pv {
        let (c0, c1, c2) = (m.rf(0x002b_d280), m.rf(0x002b_d284), m.rf(0x002b_d288));
        let (e0, e1, e2) = (m.rf(0x002b_d290), m.rf(0x002b_d294), m.rf(0x002b_d298));
        f22 = c0 + f20 * (c1 + f20 * c2);
        f23 = e0 + f21 * (e1 + f21 * e2);
    }
    let p = m.r32(a0 + 0x150);
    if p != 0 && f32::from_bits(0x38d1_b717) < f20 {
        let x = m.rf(p + 0x18);
        let y = m.rf(p + 0x1c);
        let q = m.call(0x0011_e1a0, &[], &[f4(x), f4(y), f4(0.0)]).1;
        let dot = q[0] * d[0] + q[1] * d[1] + q[2] * d[2];
        let f0 = dot / f20;
        let (h0, h1, h2) = (m.rf(0x002b_d2a0), m.rf(0x002b_d2a4), m.rf(0x002b_d2a8));
        let lo = m.rf(0x002b_bb60);
        let hi = m.rf(0x002b_bb64);
        let f2 = h0 + f0 * (h1 + f0 * h2);
        let f3 = if f2 < lo {
            lo
        } else if hi < f2 {
            hi
        } else {
            f2
        };
        let (g0, g1, g2) = (m.rf(0x002b_d2b0), m.rf(0x002b_d2b4), m.rf(0x002b_d2b8));
        f22 = f22 + f3 * (g0 + f20 * (g1 + f20 * g2));
    }
    f20 = f20 + f22;
    f21 = f21 + f23;
    if a3 != 0 {
        m.wf(a3, f20);
    }
    if t0 != 0 {
        m.wf(t0, f21);
    }
}

/// FUN_00132b10(obj, id): class of `obj` derives from `id`: rec = slot 5 (vt+0x34, class descriptor getter);
/// id <= rec+4 < DAT_002baaf0[id*0x14] (ANIM-ALL: the generated adapter passed `obj` as the class value).
pub fn ee_sub_132b10<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let rec = vslot(m, a0, 5, &[], &[]).0;
    let c = m.r32(rec.wrapping_add(4)) as i32;
    let id = a1 as i32;
    (id <= c && c < m.r32((0x002b_aaf0u32).wrapping_add((id as u32).wrapping_mul(0x14))) as i32) as u32
}

/// The FUN_00226550 DList pass: iterator frame `fr` = {list, &next, next offset, saved chain} linked at
/// DAT_002dd2fc (unlinked again after), `f(entry)` for every entry (the next pointer is re-read after each call,
/// so `f` may remove the current entry).
pub fn dlist_walk<M: QMem + ?Sized>(m: &mut M, fr: u32, list: u32, f: u32) {
    let prev = m.r32(0x002d_d2fc);
    let off = m.r32(list + 8);
    m.w32(fr + 0xc, prev);
    m.w32(0x002d_d2fc, fr);
    let head = m.r32(list);
    m.w32(fr, list);
    m.w32(fr + 4, head.wrapping_add(off));
    m.w32(fr + 8, off);
    let mut cur = head;
    while cur != 0 {
        m.call(f, &[cur], &[]);
        let nx = m.r32(fr + 4);
        let off = m.r32(fr + 8);
        cur = m.r32(nx);
        m.w32(fr + 4, cur.wrapping_add(off));
    }
}

/// ANIM-ALL: the ASEGA part of FUN_00226550 for hosts that run the object updates themselves (LiveQ):
/// FUN_00130918 over world+0x3d88 (every live ASEGA of the level, all actors), chain restored after.
pub fn world_asega_pass<M: QMem + ?Sized>(m: &mut M, world: u32) {
    let sp = stack_push(m, 0x10);
    let saved = m.r32(0x002d_d2fc);
    dlist_walk(m, sp, world + 0x3d88, 0x0013_0918);
    m.w32(0x002d_d2fc, saved);
    stack_pop(m, 0x10);
}

/// FUN_00226550(world): per-frame world update. FUN_00149c80; FUN_00147f40; dt <= 0 -> FUN_002267c0; else
/// FUN_001c4468 over the +0x3eac list (next +0x84), FUN_001f77c0, vslot 15 over the flagged (&2) entries of
/// +0x3f20[+0x3f14] (index local published in +0x3f1c), FUN_0019f120, FUN_00130918 / FUN_00156728 over the
/// +0x3d88 / +0x3ffc DLists with the iterator frame chained into DAT_002dd2fc, FUN_00149c80, FUN_001f8dd8,
/// FUN_001be1e8, FUN_001ec968, FUN_00149c80. Then FUN_001ab1a0(0x2e65e0), FUN_001fd830(0x3dd040),
/// DAT_002ddee0 > 0 -> FUN_0014d478(0x2ddf40), FUN_0020f7e0(world), FUN_00128148(0x2dd598).
pub fn ee_sub_226550<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0014_9c80, &[a0], &[]);
    m.call(0x0014_7f40, &[a0], &[]);
    if m.rf(0x002d_ded4) <= 0.0 {
        m.call(0x0022_67c0, &[a0], &[]);
    } else {
        let mut s0 = m.r32(a0 + 0x3eac);
        while s0 != 0 {
            m.call(0x001c_4468, &[s0], &[]);
            s0 = m.r32(s0 + 0x84);
        }
        m.call(0x001f_77c0, &[a0], &[]);
        let sp = stack_push(m, 0x30);
        let idx = sp + 0x20;
        m.w32(a0 + 0x3f1c, idx);
        m.w32(idx, 0);
        if m.r32(a0 + 0x3f14) as i32 > 0 {
            let mut i = 0u32;
            loop {
                let e = m.r32(a0 + 0x3f20).wrapping_add(i.wrapping_shl(3));
                if m.r32(e) & 2 != 0 {
                    let o = m.r32(e + 4);
                    vslot(m, o, 15, &[], &[]);
                }
                let v = m.r32(idx).wrapping_add(1);
                i = v;
                m.w32(idx, v);
                if (i as i32) >= m.r32(a0 + 0x3f14) as i32 {
                    break;
                }
            }
        }
        m.w32(a0 + 0x3f1c, 0);
        m.call(0x0019_f120, &[a0], &[]);
        // DList iterator frames {list, &next, next offset, saved chain}
        for (fr, list, f) in [(sp, a0 + 0x3d88, 0x0013_0918u32), (sp + 0x10, a0 + 0x3ffc, 0x0015_6728)] {
            dlist_walk(m, fr, list, f);
        }
        m.call(0x0014_9c80, &[a0], &[]);
        m.call(0x001f_8dd8, &[a0], &[]);
        m.call(0x001b_e1e8, &[], &[]);
        m.call(0x001e_c968, &[], &[]);
        m.call(0x0014_9c80, &[a0], &[]);
        let saved = m.r32(sp + 0xc);
        m.w32(0x002d_d2fc, saved);
        stack_pop(m, 0x30);
    }
    m.call(0x001a_b1a0, &[0x002e_65e0], &[]);
    m.call(0x001f_d830, &[0x003d_d040], &[]);
    if 0.0 < m.rf(0x002d_dee0) {
        m.call(0x0014_d478, &[0x002d_df40], &[]);
    }
    m.call(0x0020_f7e0, &[a0], &[]);
    m.call(0x0012_8148, &[0x002d_d598], &[]);
}

/// save-variable read used by FUN_001d8ff8: FUN_002581c0(&buf, key, a2); word = buf+4; FUN_00129ec8(&buf).
fn h11_savevar<M: QMem + ?Sized>(m: &mut M, buf: u32, key: u32, a2: u32) -> u32 {
    m.call(0x0025_81c0, &[buf, key, a2], &[]);
    let w = m.r32(buf + 4);
    m.call(0x0012_9ec8, &[buf], &[]);
    w
}

/// FUN_001d8ff8(ctx): apply the level's tunable save variables (keys GS+0x3b2c..0x3b44) to the audio / render
/// globals: FUN_001a2bd0 (when FUN_001a2c98() >= 2), FUN_001fe9c8(0x3dd040, v), FUN_001fea60(1 - v, 0x3dd040, ch, 1)
/// (ch 1; 0 and 3), DAT_003dd050 = v, and the 0x2debc0 / 0x2deb98 display words' low bit fields from
/// FUN_0023e1c8(v + 636) (bits 0..11) and FUN_0023e1c8(v + 50) & 0x7fe (bits 12..22).
pub fn ee_sub_1d8ff8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    const G: u32 = 0x003d_d490;
    let buf = stack_push(m, 0x10);
    if m.r32(G + 0x3b2c) != 0 && (m.call(0x001a_2c98, &[], &[]).0 as i32) >= 2 {
        let k = m.r32(G + 0x3b2c);
        let w = h11_savevar(m, buf, k, a0);
        m.call(0x001a_2bd0, &[w], &[]);
    }
    let k = m.r32(G + 0x3b30);
    if k != 0 {
        let w = h11_savevar(m, buf, k, a0);
        m.call(0x001f_e9c8, &[0x003d_d040, w], &[]);
    }
    let k = m.r32(G + 0x3b34);
    if k != 0 {
        let f = f32::from_bits(h11_savevar(m, buf, k, a0));
        m.call(0x001f_ea60, &[0x003d_d040, 1, 1], &[f4(1.0 - f)]);
    }
    let k = m.r32(G + 0x3b38);
    if k != 0 {
        let f = f32::from_bits(h11_savevar(m, buf, k, a0));
        m.call(0x001f_ea60, &[0x003d_d040, 0, 1], &[f4(1.0 - f)]);
        let k = m.r32(G + 0x3b38);
        let f = f32::from_bits(h11_savevar(m, buf, k, a0));
        m.call(0x001f_ea60, &[0x003d_d040, 3, 1], &[f4(1.0 - f)]);
    }
    let k = m.r32(G + 0x3b3c);
    if k != 0 {
        let w = h11_savevar(m, buf, k, a0);
        m.w32(0x003d_d050, w);
    }
    let k = m.r32(G + 0x3b40);
    if k != 0 {
        let f = f32::from_bits(h11_savevar(m, buf, k, a0));
        let v = m.call(0x0023_e1c8, &[], &[f4(f + 636.0)]).0 & 0xfff;
        for a in [0x002d_ebc0u32, 0x002d_eb98] {
            let w = (m.r32(a) & 0xffff_f000) | v;
            m.w32(a, w);
        }
    }
    let k = m.r32(G + 0x3b44);
    if k != 0 {
        let f = f32::from_bits(h11_savevar(m, buf, k, a0));
        let v = (m.call(0x0023_e1c8, &[], &[f4(f + 50.0)]).0 & 0x7fe) << 12;
        for a in [0x002d_ebc0u32, 0x002d_eb98] {
            let w = (m.r32(a) & 0xff80_0fff) | v;
            m.w32(a, w);
        }
    }
    stack_pop(m, 0x10);
}

fn h11_unit_or<M: QMem + ?Sized>(m: &mut M, v: V4) -> V4 {
    let len = len3(v);
    if len < f32::from_bits(0x38d1_b717) {
        m.rv(0x002a_2da0)
    } else {
        let q = 1.0 / len;
        [v[0] * q, v[1] * q, v[2] * q, v[3] * q]
    }
}

/// FUN_001999d0(o, mesh, kind, x, out, q (128-bit a2)): pick a contact normal into *out then return
/// FUN_001997d8(o, q, *out). kind 1: sum the FUN_00199408(o, mesh, cell, &n) normals over the cells FUN_001665f0(
/// mesh+0x90, mesh+0x68, (x - mesh+0x94) >> 4, 0x40, list) returns; kind 2 (x+8 >= 0.7): sum the FUN_00118668 hit
/// normals of the edges in the x+0x10 (next +8) / x+0x14 (next +0xc) lists that pass FUN_00199408; kind 3:
/// FUN_00199408(o, mesh, x, out). No hit -> 0; the sums are normalised (qword 0x2a2da0 when < 1e-4).
pub fn ee_sub_1999d0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a3: i32, t0: u32, t1: u32, a2: V4) -> u32 {
    let sp = stack_push(m, 0x120);
    let ok = match a3 {
        1 => {
            let mut acc = m.rv(0x002a_2dd0);
            let base = m.r32(a1 + 0x94);
            let idx = ((t0.wrapping_sub(base)) as i32 >> 4) as u32;
            let n = m.call(0x0016_65f0, &[a1 + 0x90, a1 + 0x68, idx, 0x40, sp], &[]).0 as i32;
            let mut hit = false;
            if n > 0 {
                let mut s1 = n as u32;
                let mut s0 = sp;
                loop {
                    let c = m.r32(s0);
                    if m.call(0x0019_9408, &[a0, a1, c, sp + 0x100], &[]).0 != 0 {
                        let v = m.rv(sp + 0x100);
                        hit = true;
                        for i in 0..4 {
                            acc[i] += v[i];
                        }
                    }
                    s1 = s1.wrapping_sub(1);
                    s0 += 4;
                    if s1 == 0 {
                        break;
                    }
                }
            }
            if hit {
                let u = h11_unit_or(m, acc);
                m.wv(t1, u);
            }
            hit
        }
        2 => {
            if m.rf(t0 + 8) < f32::from_bits(0x3f33_3333) {
                false
            } else {
                let z = m.rv(0x002a_2dd0);
                m.wv(sp + 0x110, z);
                let mut hit = false;
                for (head, next) in [(0x10u32, 8u32), (0x14, 0xc)] {
                    let mut s0 = m.r32(t0 + head);
                    while s0 != 0 {
                        if m.call(0x0019_9408, &[a0, a1, s0, sp], &[]).0 != 0 {
                            let w = m.r32(a0);
                            let i0 = m.r16(s0) as u32;
                            let i1 = m.r16(s0 + 2) as u32;
                            let vb = m.r32(a1 + 0x94);
                            let g = m.r32(w + 0x258);
                            let x = m.r32(g + 0x74);
                            let qa = m.rv(i0.wrapping_shl(4).wrapping_add(vb));
                            let qb = m.rv(i1.wrapping_shl(4).wrapping_add(vb));
                            let r = m.call(0x0011_8668, &[x, 0x7fff_ffff, 2, sp + 0x10], &[qa, qb]).0;
                            if r as i32 > 0 {
                                hit = true;
                                let mut acc = m.rv(sp + 0x110);
                                let v = m.rv(sp);
                                for i in 0..4 {
                                    acc[i] += v[i];
                                }
                                m.wv(sp + 0x110, acc);
                            }
                        }
                        s0 = m.r32(s0 + next);
                    }
                }
                if hit {
                    let acc = m.rv(sp + 0x110);
                    let u = h11_unit_or(m, acc);
                    m.wv(t1, u);
                }
                hit
            }
        }
        3 => m.call(0x0019_9408, &[a0, a1, t0, t1], &[]).0 != 0,
        _ => true,
    };
    stack_pop(m, 0x120);
    if !ok {
        return 0;
    }
    let q = m.rv(t1);
    m.call(0x0019_97d8, &[a0], &[a2, q]).0
}

fn h11_cross(a: V4, b: V4, w: f32) -> V4 {
    [a[1] * b[2] - b[1] * a[2], a[2] * b[0] - b[2] * a[0], a[0] * b[1] - b[0] * a[1], w]
}

fn h11_rnd<M: QMem + ?Sized>(m: &mut M, lo: f32, hi: f32) -> f32 {
    m.call(0x0011_d448, &[], &[f4(lo), f4(hi)]).1[0]
}

fn h11_rnd_at<M: QMem + ?Sized>(m: &mut M, a: u32) -> f32 {
    let lo = m.rf(a);
    let hi = m.rf(a + 4);
    h11_rnd(m, lo, hi)
}

/// FUN_001d4568(o, e): if FUN_001f7588(o, e): o+0x560 = max(o+0x560 + e+0x10, 0); FUN_001d4c88(o); (0 ->
/// FUN_001d4940(o, 3)); impulse by e+0xc kind: 0x100 random direction (FUN_0011e1a0 of three FUN_0011d448
/// rolls) + up x dir scaled; 0x1000 horizontal away from FUN_001a62e8(*e) blended with up (0x2a2dc0);
/// then slot 34(slot 32() + lin), slot 35(slot 33() + ang); return 1.
pub fn ee_sub_1d4568<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    if m.call(0x001f_7588, &[a0, a1], &[]).0 == 0 {
        return 0;
    }
    let v = (m.r32(a0 + 0x560).wrapping_add(m.r32(a1 + 0x10))) as i32;
    m.w32(a0 + 0x560, if v > -1 { v as u32 } else { 0 });
    m.call(0x001d_4c88, &[a0], &[]);
    if m.r32(a0 + 0x560) == 0 {
        m.call(0x001d_4940, &[a0, 3], &[]);
    }
    let mut lin: V4 = [0.0; 4];
    let mut ang: V4 = [0.0; 4];
    let pi = f32::from_bits(0x4049_0fdb);
    match m.r32(a1 + 0xc) {
        0x100 => {
            let r0 = h11_rnd(m, -pi, pi);
            let r1 = h11_rnd_at(m, 0x002c_21d0);
            let r2 = h11_rnd_at(m, 0x002c_21d8);
            lin = m.call(0x0011_e1a0, &[], &[f4(r0), f4(r1), f4(r2)]).1;
            let s = h11_rnd_at(m, 0x002c_21e0);
            let up = m.rv(0x002a_2dc0);
            let c = h11_cross(up, lin, 0.0);
            let u = h11_unit_or(m, c);
            ang = [u[0] * s, u[1] * s, u[2] * s, u[3] * s];
        }
        0x1000 => {
            let p = m.call(0x001a_62e8, &[a0], &[]).1;
            let x = m.r32(a1);
            let q = m.call(0x001a_62e8, &[x], &[]).1;
            let mut d = [p[0] - q[0], p[1] - q[1], p[2] - q[2], p[3] - q[3]];
            let up = m.rv(0x002a_2dc0);
            let dot = d[0] * up[0] + d[1] * up[1] + d[2] * up[2];
            for i in 0..4 {
                d[i] -= up[i] * dot;
            }
            let h = h11_unit_or(m, d);
            let e = h11_rnd_at(m, 0x002c_21e8);
            let f = h11_rnd_at(m, 0x002c_21f0);
            for i in 0..4 {
                lin[i] = h[i] * e + up[i] * f;
            }
            let g = h11_rnd_at(m, 0x002c_21f8);
            let c = h11_cross(up, h, h[3]);
            let k = h11_rnd_at(m, 0x002c_2200);
            for i in 0..4 {
                ang[i] = c[i] * g + up[i] * k;
            }
        }
        _ => {}
    }
    let cur = vslot(m, a0, 32, &[], &[]).1;
    let nv = [cur[0] + lin[0], cur[1] + lin[1], cur[2] + lin[2], cur[3] + lin[3]];
    vslot(m, a0, 34, &[], &[nv]);
    let cur = vslot(m, a0, 33, &[], &[]).1;
    let nv = [cur[0] + ang[0], cur[1] + ang[1], cur[2] + ang[2], cur[3] + ang[3]];
    vslot(m, a0, 35, &[], &[nv]);
    1
}

/// 0x1d8a38 — varargs formatter: builds string from fmt a1 + va_list (a2..t3, f12..f18) into string obj a0.
pub fn ee_sub_1d8a38<M: QMem + ?Sized>(
    m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, t0: u32, t1: u32, t2: u32, t3: u32,
    f12: f32, f14: f32, f16: f32, f18: f32,
) -> u32 {
    let sp = stack_push(m, 0x100);
    m.w32(sp + 0xd0, a2);
    m.w32(sp + 0xd4, 0);
    m.w32(sp + 0xd8, a3);
    m.w32(sp + 0xdc, 0);
    m.w32(sp + 0xe0, t0);
    m.w32(sp + 0xe4, 0);
    m.w32(sp + 0xe8, t1);
    m.w32(sp + 0xec, 0);
    m.w32(sp + 0xf0, t2);
    m.w32(sp + 0xf4, 0);
    m.w32(sp + 0xf8, t3);
    m.w32(sp + 0xfc, 0);
    m.wf(sp + 0xc0, f12);
    m.wf(sp + 0xc4, f14);
    m.wf(sp + 0xc8, f16);
    m.wf(sp + 0xcc, f18);
    m.w32(sp, 0xffff_ffff);
    m.call(0x1ae478, &[sp + 0x10], &[]);
    m.call(0x1d87c0, &[a1, sp + 0xd0, sp + 0x20], &[]);
    m.call(0x1d7ed8, &[sp + 0x30, 0x3e0ea8, sp + 0x20, 0x3d4a60], &[]);
    m.call(0x129658, &[sp, sp + 0x30], &[]);
    m.call(0x129ec8, &[sp + 0x30], &[]);
    m.call(0x1291a8, &[a0, sp], &[]);
    m.call(0x1ae4a0, &[sp + 0x10, 2], &[]);
    m.call(0x129ec8, &[sp], &[]);
    stack_pop(m, 0x100);
    a0
}

/// 0x11a770 — LCP-style pivoting solver step (lists at +0xc/+0x18/+0x24/+0x30, up to 10 pivots).
pub fn ee_sub_11a770<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let s0 = a0;
    let s2 = a1;
    if s2 != 0 {
        m.call(0x11a2e8, &[s0 + 0x30, s2], &[]);
        m.call(0x11a220, &[s0 + 0xc, s2], &[]);
        let n = m.r32(s0 + 0x3c).wrapping_add(1);
        m.w32(s0 + 0x3c, n);
    }
    let mut s3: i32 = 0;
    loop {
        // gather column values
        let mut a3 = m.r32(s0 + 0x40);
        let mut n0 = m.r32(s0 + 0xc);
        while n0 != 0 {
            let mut n1 = m.r32(s0 + 0xc);
            while n1 != 0 {
                let d = n1.wrapping_sub(m.r32(s0 + 8)) as i32;
                let idx = (((d as i64) * (0xcccc_cccdu32 as i32 as i64)) >> 32) as i32 >> 3;
                let base = m.r32(n0 + 0x18);
                let f = m.rf(((idx << 2) as u32).wrapping_add(base));
                m.wf(a3, f);
                a3 = a3.wrapping_add(4);
                n1 = m.r32(n1 + 0x20);
            }
            n0 = m.r32(n0 + 0x20);
        }
        let (c, l40, l44) = (m.r32(s0 + 0x3c), m.r32(s0 + 0x40), m.r32(s0 + 0x44));
        m.call(0x1195e0, &[c, l40, l44], &[]);
        let mut n0 = m.r32(s0 + 0xc);
        let mut v0 = m.r32(s0 + 0x48);
        while n0 != 0 {
            let f = -m.rf(n0 + 0x10);
            m.wf(v0, f);
            n0 = m.r32(n0 + 0x20);
            v0 = v0.wrapping_add(4);
        }
        let (c, l44, l48, l4c) = (m.r32(s0 + 0x3c), m.r32(s0 + 0x44), m.r32(s0 + 0x48), m.r32(s0 + 0x4c));
        m.call(0x1198f8, &[c, l44, l48, l4c], &[]);
        if (m.r32(s0 + 4) as i32) > 0 {
            let mut i: i32 = 0;
            loop {
                let mut n0 = m.r32(s0 + 0xc);
                let mut f2 = 0.0f32;
                let mut v1 = m.r32(s0 + 0x4c);
                let a3 = m.r32(s0 + 8);
                while n0 != 0 {
                    let b = m.r32(n0 + 0x18);
                    let x = m.rf(((i as u32) << 2).wrapping_add(b));
                    let y = m.rf(v1);
                    n0 = m.r32(n0 + 0x20);
                    v1 = v1.wrapping_add(4);
                    f2 += y * x;
                }
                m.wf((i.wrapping_mul(0x28) as u32).wrapping_add(a3) + 0x14, f2);
                i += 1;
                if i >= m.r32(s0 + 4) as i32 {
                    break;
                }
            }
        }
        // ratio test
        let mut s1 = 0u32;
        let mut f3 = 1.0f32;
        let mut a2 = 0u32;
        let mut a1p = m.r32(s0 + 0x4c);
        let t1 = m.r32(s0 + 4) as i32;
        let l18 = m.r32(s0 + 0x18);
        let l24 = m.r32(s0 + 0x24);
        let mut n0 = m.r32(s0 + 0xc);
        if n0 != 0 {
            let f7 = m.rf(0x2a4120);
            let f6 = m.rf(0x2a4124);
            let f5 = f32::from_bits(0x38d1b717);
            let f4 = f32::from_bits(0xb8d1b717);
            while n0 != 0 {
                let f2 = m.rf(a1p);
                a1p = a1p.wrapping_add(4);
                if f5 < f2 {
                    let f1 = m.rf(n0 + 8);
                    if f1 != f7 {
                        let f0 = (f1 - m.rf(n0 + 0xc)) / f2;
                        if f0 < f3 {
                            f3 = f0;
                            s1 = n0;
                            a2 = 4;
                        }
                    }
                } else if f2 < f4 {
                    let f1 = m.rf(n0 + 4);
                    if f1 != f6 {
                        let f0 = (f1 - m.rf(n0 + 0xc)) / f2;
                        if f0 < f3 {
                            f3 = f0;
                            s1 = n0;
                            a2 = 2;
                        }
                    }
                }
                n0 = m.r32(n0 + 0x20);
            }
        }
        let mut v0 = l18;
        let neg = f32::from_bits(0xb8d1b717);
        while v0 != 0 {
            let f1 = m.rf(v0 + 0x14);
            if !(neg < f1) {
                let f0 = -m.rf(v0 + 0x10) / f1;
                if f0 < f3 {
                    f3 = f0;
                    s1 = v0;
                    a2 = 1;
                }
            }
            v0 = m.r32(v0 + 0x20);
        }
        let mut v0 = l24;
        let pos = f32::from_bits(0x38d1b717);
        while v0 != 0 {
            let f1 = m.rf(v0 + 0x14);
            if !(f1 < pos) {
                let f0 = -m.rf(v0 + 0x10) / f1;
                if f0 < f3 {
                    f3 = f0;
                    s1 = v0;
                    a2 = 3;
                }
            }
            v0 = m.r32(v0 + 0x20);
        }
        if t1 > 0 {
            let mut i: i32 = 0;
            let mut off = 0u32;
            loop {
                let p = m.r32(s0 + 8).wrapping_add(off);
                i += 1;
                let (pa, pb) = (m.rf(p + 0x10), m.rf(p + 0x14));
                let v = pa + f3 * pb;
                m.wf(p + 0x10, v);
                off = off.wrapping_add(0x28);
                if i >= m.r32(s0 + 4) as i32 {
                    break;
                }
            }
        }
        let mut n0 = m.r32(s0 + 0xc);
        let mut a1p = m.r32(s0 + 0x4c);
        while n0 != 0 {
            let (pa, pb) = (m.rf(n0 + 0xc), m.rf(a1p));
            let v = pa + f3 * pb;
            m.wf(n0 + 0xc, v);
            n0 = m.r32(n0 + 0x20);
            a1p = a1p.wrapping_add(4);
        }
        // jump table @0x2a4130
        match a2 {
            0 => return,
            1 | 3 => {
                let n = m.r32(s0 + 0x3c).wrapping_add(1);
                m.w32(s0 + 0x3c, n);
                let l = if a2 == 1 { 0x18 } else { 0x24 };
                m.call(0x11a2e8, &[s0 + l, s1], &[]);
                m.call(0x11a220, &[s0 + 0xc, s1], &[]);
                s3 += 1;
            }
            _ => {
                // 2 | 4
                let n = m.r32(s0 + 0x3c).wrapping_sub(1);
                m.w32(s0 + 0x3c, n);
                m.call(0x11a2e8, &[s0 + 0xc, s1], &[]);
                let l = if a2 == 2 { 0x18 } else { 0x24 };
                m.call(0x11a220, &[s0 + l, s1], &[]);
                s3 += 1;
                if s1 == s2 {
                    return;
                }
            }
        }
        if s3 >= 10 {
            return;
        }
    }
}

fn h11_f(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

fn h11_neg(q: V4) -> V4 {
    [q[0] * -1.0, q[1] * -1.0, q[2] * -1.0, q[3] * -1.0]
}

/// `(*(*(o)+0x1c))(o + *(short*)(*o+0x18), 0, sp, f12, f13)` then store -result into s0+0x190.
fn h11_vcall_neg<M: QMem + ?Sized>(m: &mut M, s0: u32, o: u32, f12: f32, f13: f32) {
    let sp = stack_push(m, 0x10);
    let vt = m.r32(o);
    let adj = m.r16(vt + 0x18) as i16 as i32;
    let f = m.r32(vt + 0x1c);
    m.call(f, &[o.wrapping_add(adj as u32), 0, sp], &[h11_f(f12), h11_f(f13)]);
    let q = m.rv(sp);
    stack_pop(m, 0x10);
    m.wv(s0 + 0x190, h11_neg(q));
}

/// 0x18b7e8 — per-frame state update dispatch on *(a0+0x18c8) (jump table @0x2a5a10).
pub fn ee_sub_18b7e8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let s0 = a0;
    let f0 = m.call(0x204740, &[s0], &[]).1[0];
    let p = m.r32(s0 + 0x150);
    let f12 = m.rf(p + 0x18) - f0;
    m.call(0x11cf60, &[], &[h11_f(f12)]);
    m.call(0x194a40, &[0x3dd490, s0, 0xffff], &[]);
    m.call(0x18b2c0, &[s0], &[]);
    m.call(0x18a8c8, &[s0], &[]);
    m.call(0x18f0a8, &[s0], &[]);
    match m.r32(s0 + 0x18c8) {
        4 | 9 => {
            let s1 = m.r32(s0 + 0x1030);
            let q = m.rv(0x2a2dd0);
            m.wv(s0 + 0x190, q);
            if s1 != 0 {
                let a = m.r32(s1 + 0x14);
                let v = m.call(0x12ff10, &[a, s0, 1, 1], &[]).0;
                if v != 0 {
                    let o = m.r32(v + 4);
                    let (f12, f13) = (m.rf(s1 + 0x1c), m.rf(s1 + 0x40));
                    h11_vcall_neg(m, s0, o, f12, f13);
                }
            }
        }
        3 => {
            m.call(0x190520, &[s0], &[]);
        }
        2 => {
            m.call(0x197f98, &[s0 + 0x2c0], &[]);
        }
        7 => {
            m.call(0x191a10, &[s0 + 0x4c0], &[]);
        }
        6 => {
            m.call(0x137218, &[s0 + 0x1ff0], &[]);
        }
        10 => {
            let v = m.r32(s0 + 0x150);
            let a2 = m.r32(s0 + 0x1bc0);
            let (x, y) = (m.rf(v + 0x18), m.rf(v + 0x1c));
            m.call(0x1a0720, &[s0, s0 + 0x1ac0, a2], &[h11_f(x), h11_f(y)]);
        }
        11 => {
            m.call(0x18b460, &[s0], &[]);
        }
        12 => {
            let v = m.r32(s0 + 0x150);
            let y = m.rf(v + 0x1c);
            m.call(0x20bb58, &[s0 + 0x1c30], &[h11_f(y)]);
            let v = m.r32(s0 + 0x150);
            let x = m.rf(v + 0x18);
            m.call(0x204660, &[s0], &[h11_f(x)]);
        }
        5 => {
            let v1 = m.r32(s0 + 0x18d0) as i32;
            if v1 == 5 {
                let v = m.r32(s0 + 0x150);
                let a2 = m.r32(s0 + 0x14d0);
                let (x, y) = (m.rf(v + 0x18), m.rf(v + 0x1c));
                m.call(0x1a0720, &[s0, s0 + 0x13d0, a2], &[h11_f(x), h11_f(y)]);
            } else if v1 < 6 {
                if v1 == 4 && m.r32(s0 + 0x10dc) != 0 {
                    m.call(0x2046f0, &[s0], &[]);
                }
            } else if v1 < 10 && v1 >= 8 {
                let v = m.r32(s0 + 0x150);
                let f13 = m.rf(v + 0x1c) * m.rf(0x2bf3c0);
                let f12 = m.rf(v + 0x18) + f32::from_bits(0x40490fdb);
                let q = m.call(0x11e1a0, &[], &[h11_f(f12), h11_f(f13), h11_f(0.0)]).1;
                let r = m.call(0x1a6fc0, &[0, s0], &[q]).1;
                m.wv(s0 + 0x190, r);
            }
        }
        16 => {
            if m.call(0x1daea8, &[0x1c], &[]).0 != 0 {
                m.call(0x18e1e0, &[s0], &[]);
            } else if m.r32(s0 + 0x1d2c) != 0 {
                let a = m.r32(s0 + 0x1d1c);
                let v = m.call(0x12ff10, &[a, s0, 1, 1], &[]).0;
                if v != 0 {
                    let o = m.r32(v + 4);
                    let a3 = m.r32(s0 + 0x1030);
                    let (f12, f13) = (m.rf(a3 + 0x1c), m.rf(a3 + 0x40));
                    h11_vcall_neg(m, s0, o, f12, f13);
                }
            }
        }
        19 | 20 | 17 => {
            let a1 = m.r32(s0 + 0x1030);
            let y = m.rf(a1 + 0x1c);
            let q = m.call(0x204e38, &[s0], &[h11_f(y)]).1;
            m.wv(s0 + 0x190, h11_neg(q));
            let st = m.r32(s0 + 0x18c8);
            if st == 20 {
                let a0 = m.r32(s0 + 0x1030);
                let v = m.r32(s0 + 0x150);
                let y = m.rf(v + 0x1c);
                m.call(0x131a00, &[a0], &[h11_f(y)]);
                let v = m.r32(s0 + 0x150);
                let x = m.rf(v + 0x18);
                m.call(0x204660, &[s0], &[h11_f(x)]);
            } else if st == 17 {
                let v = m.r32(s0 + 0x150);
                if 0.0 < m.rf(v + 0x1c) {
                    let x = m.rf(v + 0x18);
                    m.call(0x204660, &[s0], &[h11_f(x)]);
                }
            }
        }
        24 => {
            let k = m.rf(0x2bf3c4);
            m.call(0x1441e8, &[s0], &[h11_f(k)]);
            if m.r32(s0 + 0x18d0) == 0xc {
                let f0 = m.call(0x204740, &[s0], &[]).1[0];
                let d = f0 - m.rf(s0 + 0x1a8);
                let r = m.call(0x11cf60, &[], &[h11_f(d)]).1[0];
                if r.abs() < m.rf(0x2bf3c8) {
                    m.call(0x18c250, &[s0, 0x18, 0xd], &[]);
                }
            } else {
                let v = m.r32(s0 + 0x150);
                let x = m.rf(v + 0x18);
                m.call(0x204660, &[s0], &[h11_f(x)]);
            }
            let mut f20 = 1.0f32;
            let z = vslot(m, s0, 33, &[], &[]).1[2];
            let f12 = m.rf(s0 + 0x1f4) * m.rf(0x2bf35c);
            let f12 = z / f12;
            let f0 = m.call(0x11cfe0, &[], &[h11_f(f12), h11_f(f20)]).1[0];
            let f1 = m.rf(0x2bf3cc) * f0;
            let f3 = f1.abs();
            m.wf(s0 + 0x2014, 0.0f32.max(f1));
            f20 -= f3;
            m.wf(s0 + 0x202c, 0.0f32.max(-f1));
            m.wf(s0 + 0x2020, f20);
            let a0 = m.r32(s0 + 0x2038);
            m.call(0x1384c0, &[a0], &[]);
        }
        _ => {}
    }
    m.call(0x1a1358, &[s0 + 0xdf0], &[]);
    m.call(0x19fc30, &[s0], &[]);
    let a0 = m.r32(s0 + 0x1310);
    if a0 != 0 && m.call(0x131968, &[a0], &[]).0 != 0 {
        m.call(0x1306a8, &[s0 + 0x1310], &[]);
        m.w32(s0 + 0x1314, 0xffff_ffff);
    }
}

/// Resolve a member-function-pointer descriptor at `d` (+idx_off: s16 vtable index, +vt_off: s16 vtable-field
/// offset / raw fn, +adj_off: s16 this-adjust) against `obj` -> (fn, this).
fn h11_mfp<M: QMem + ?Sized>(m: &mut M, d: u32, obj: u32, adj_off: u32, idx_off: u32, fn_off: u32) -> (u32, u32) {
    let idx = m.r16(d + idx_off) as i16 as i32;
    if idx < 0 {
        let f = m.r32(d + fn_off);
        let adj = m.r16(d + adj_off) as i16 as i32;
        (f, obj.wrapping_add(adj as u32))
    } else {
        let voff = m.r16(d + fn_off) as i16 as i32;
        let vt = m.r32(obj.wrapping_add(voff as u32));
        let e = vt.wrapping_add((idx << 3) as u32).wrapping_sub(8);
        let f = m.r32(e + 4);
        let adj = m.r16(d + adj_off) as i16 as i32;
        let lo = m.r16(e) as i16 as i32;
        (f, obj.wrapping_add(lo.wrapping_add(adj) as u32))
    }
}

/// 48-byte copy into the variant payload then clear the w lanes (+0xc/+0x2c/+0x1c).
fn h11_copy_mat<M: QMem + ?Sized>(m: &mut M, dst: u32, src: u32) {
    let q0 = m.rv(src);
    m.wv(dst, q0);
    let q1 = m.rv(src + 0x10);
    m.wv(dst + 0x10, q1);
    let q2 = m.rv(src + 0x20);
    m.w32(dst + 0xc, 0);
    m.wv(dst + 0x20, q2);
    m.w32(dst + 0x2c, 0);
    m.w32(dst + 0x1c, 0);
}

/// 0x1ffd98 — reflection property getter: read property described by a2 from object a1 into variant, assign to a0.
pub fn ee_sub_1ffd98<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) -> u32 {
    let sp = stack_push(m, 0x30);
    let s0d = a2;
    let mut s2 = a1;
    let fl = m.r32(s0d + 4);
    m.w32(sp, 0xffff_ffff);
    if fl & 1 != 0 {
        let v = m.call(0x259c18, &[], &[]).0;
        m.call(0x259cf8, &[sp + 0x10, v], &[]);
        let p = m.r32(sp + 0x10);
        m.w32(p + 8, s2);
        let x = m.r32(s0d + 0x10);
        let p = m.r32(sp + 0x10);
        m.w32(p + 0xc, x);
        let x = m.r32(s0d + 0x18);
        let p = m.r32(sp + 0x10);
        m.w32(p + 0x10, x);
        let p = m.r32(sp + 0x10);
        m.call(0x129a70, &[sp, p], &[]);
        let s = m.r32(sp + 0x10);
        if s != 0 {
            let c = m.r32(s).wrapping_sub(1);
            m.w32(s, c);
            if c == 0 {
                m.call(0x256120, &[s], &[]);
                m.call(0x11cd98, &[0x452170, s], &[]);
            }
        }
    } else {
        let a3 = if fl & 0x2000 != 0 {
            let (f, this) = h11_mfp(m, s0d, s2, 0x20, 0x22, 0x24);
            s2 = m.call(f, &[this, 1], &[]).0;
            m.r32(s0d)
        } else {
            m.r32(s0d)
        };
        if fl & 2 != 0 {
            let p = s2.wrapping_add(m.r32(s0d + 8));
            let sel = if a3 < 0x18 { a3 } else { 3 };
            match sel {
                0 => {
                    let v = (m.r32(p) != 0) as u32;
                    m.call(0x129598, &[sp, v], &[]);
                }
                1 => {
                    let f = m.rf(p);
                    m.call(0x129558, &[sp], &[[f, 0.0, 0.0, 0.0]]);
                }
                2 => {
                    m.call(0x1296a8, &[sp, p], &[]);
                }
                4 => {
                    m.call(0x129ec8, &[sp], &[]);
                    m.call(0x12a0b0, &[sp, 0x1b], &[]);
                    let d = m.r32(sp + 4);
                    h11_copy_mat(m, d, p);
                }
                5..=8 | 14..=17 | 19 | 20 | 23 => {
                    let v = m.r32(p);
                    m.call(0x129518, &[sp, v], &[]);
                }
                9 => {
                    m.call(0x129808, &[sp, p], &[]);
                }
                10 => {
                    m.call(0x129878, &[sp, p], &[]);
                }
                11 => {
                    m.call(0x1298f0, &[sp, p], &[]);
                }
                13 => {}
                _ => {
                    if a3 & 0x1000 != 0 {
                        let v = m.r32(p);
                        m.call(0x129518, &[sp, v], &[]);
                    } else if a3 & 0x2000 != 0 {
                        let v = m.r32(p);
                        m.call(0x129a10, &[sp, v], &[]);
                    }
                }
            }
        } else {
            let sel = if a3 < 0x18 { a3 } else { 3 };
            match sel {
                0 => {
                    let (f, this) = h11_mfp(m, s0d, s2, 8, 0xa, 0xc);
                    let r = m.call(f, &[this], &[]).0;
                    m.call(0x129598, &[sp, (r != 0) as u32], &[]);
                }
                1 => {
                    let (f, this) = h11_mfp(m, s0d, s2, 8, 0xa, 0xc);
                    let r = m.call(f, &[this], &[]).1[0];
                    m.call(0x129558, &[sp], &[[r, 0.0, 0.0, 0.0]]);
                }
                2 => {
                    let (f, this) = h11_mfp(m, s0d, s2, 8, 0xa, 0xc);
                    let r = m.call(f, &[this], &[]).0;
                    m.call(0x129718, &[sp, r], &[]);
                }
                4 => {
                    m.call(0x129ec8, &[sp], &[]);
                    m.call(0x12a0b0, &[sp, 0x1b], &[]);
                    let s1 = m.r32(sp + 4);
                    let (f, this) = h11_mfp(m, s0d, s2, 8, 0xa, 0xc);
                    let r = m.call(f, &[this], &[]).0;
                    h11_copy_mat(m, s1, r);
                }
                5..=8 | 13..=17 | 19 | 20 | 23 => {
                    let (f, this) = h11_mfp(m, s0d, s2, 8, 0xa, 0xc);
                    let r = m.call(f, &[this], &[]).0;
                    m.call(0x129518, &[sp, r], &[]);
                }
                9 | 10 | 11 => {
                    let ty = match sel {
                        9 | 10 => 0x1c,
                        _ => 0x1e,
                    };
                    m.call(0x129ec8, &[sp], &[]);
                    m.call(0x12a0b0, &[sp, ty], &[]);
                    let (f, this) = h11_mfp(m, s0d, s2, 8, 0xa, 0xc);
                    let a1 = m.r32(sp + 4);
                    m.call(f, &[this, a1], &[]);
                }
                _ => {
                    if a3 & 0x8000 != 0 {
                        let t = sp + 0x20;
                        let (f, this) = h11_mfp(m, s0d, s2, 8, 0xa, 0xc);
                        m.call(f, &[t, this], &[]);
                        m.call(0x129658, &[sp, t], &[]);
                        m.call(0x129ec8, &[t], &[]);
                    } else if a3 & 0x1000 != 0 {
                        let (f, this) = h11_mfp(m, s0d, s2, 8, 0xa, 0xc);
                        let r = m.call(f, &[this], &[]).0;
                        m.call(0x129518, &[sp, r], &[]);
                    } else if a3 & 0x2000 != 0 {
                        let (f, this) = h11_mfp(m, s0d, s2, 8, 0xa, 0xc);
                        let r = m.call(f, &[this], &[]).0;
                        m.call(0x129a10, &[sp, r], &[]);
                    }
                }
            }
        }
    }
    m.call(0x1291a8, &[a0, sp], &[]);
    m.call(0x129ec8, &[sp], &[]);
    stack_pop(m, 0x30);
    a0
}

/// ANIM-ALL: FUN_00117898(bis, n, dst) -> bytes read. Bulk stream read (FUN_00117968 forwards here): loop
/// { FUN_00117810 refill; avail 0 -> +8 = -1, return; chunk = min(n, +8); FUN_001ae5a0(dst, +4, chunk) if dst;
/// +4 += chunk, +8 -= chunk }.
pub fn ee_sub_117898<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) -> u32 {
    let n = a1 as i32;
    let mut left = n;
    let mut dst = a2;
    if (m.r32(a0 + 8) as i32) < 0 {
        return 0;
    }
    while left > 0 {
        m.call(0x0011_7810, &[a0], &[]);
        let avail = m.r32(a0 + 8) as i32;
        if avail == 0 {
            m.w32(a0 + 8, u32::MAX);
            return (n - left) as u32;
        }
        let c = left.min(avail);
        if dst != 0 {
            let src = m.r32(a0 + 4);
            m.call(0x001a_e5a0, &[dst, src, c as u32], &[]);
            dst = dst.wrapping_add(c as u32);
        }
        let p = m.r32(a0 + 4).wrapping_add(c as u32);
        m.w32(a0 + 4, p);
        m.w32(a0 + 8, (avail - c) as u32);
        left -= c;
    }
    n as u32
}

/// ANIM-ALL: FUN_00117810(bis): refill when +8 == 0 (flag 2 -> FUN_001179f0, else the source virtual +0xc on
/// bis+0x30 then +8 = +0x14, +4 = +0x10, +0x10 += +0x14, +0x14 = 0); +0xc += +8.
pub fn ee_sub_117810<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    if m.r32(a0 + 8) != 0 {
        return;
    }
    if m.r32(a0) & 2 == 0 {
        let vt = m.r32(a0 + 0x30);
        let adj = m.r16(vt + 8) as i16 as i32;
        let f = m.r32(vt + 0xc);
        m.call(f, &[(a0 as i32 + adj) as u32], &[]);
        let (s14, s10) = (m.r32(a0 + 0x14), m.r32(a0 + 0x10));
        m.w32(a0 + 8, s14);
        m.w32(a0 + 4, s10);
        m.w32(a0 + 0x10, s10.wrapping_add(s14));
        m.w32(a0 + 0x14, 0);
    } else {
        m.call(0x0011_79f0, &[a0], &[]);
    }
    let v = m.r32(a0 + 0xc).wrapping_add(m.r32(a0 + 8));
    m.w32(a0 + 0xc, v);
}

/// ANIM-ALL: FUN_00117c20(bis) -> u16 (LE; < 2 bytes left -> FUN_00117968(bis, 2, tmp)). The table entry routed
/// to q9util::read_u16, which itself calls 0x117c20 (unbounded recursion -> budget abort, nothing read).
pub fn ee_sub_117c20<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let n = m.r32(a0 + 8) as i32;
    if n < 2 {
        let sp = stack_push(m, 0x10);
        m.call(0x0011_7968, &[a0, 2, sp], &[]);
        let v = m.r16(sp) as u32;
        stack_pop(m, 0x10);
        return v;
    }
    let p = m.r32(a0 + 4);
    let v = m.r16(p) as u32;
    m.w32(a0 + 8, (n - 2) as u32);
    m.w32(a0 + 4, p.wrapping_add(2));
    v
}

/// ANIM-ALL: FUN_001603b8(alo, bis): u8 n records of 0x10 bytes (FUN_001ae2a0): +0 = bone of s16 oid (vt+0x19c
/// = slot 50; 0 -> &DAT_002c44f0), +4 = bone of s16 oid, +8 / +0xc = u8; the +4 bone's +0xa8 = record;
/// FUN_002303a0(alo, bone, 0xf). Unported before: the ALO load (FUN_0022d7a0) lost a byte per record.
pub fn ee_sub_1603b8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let mut n = m.call(0x0011_7d00, &[a1], &[]).0 as i32;
    if n <= 0 {
        return;
    }
    while n != 0 {
        let sp = stack_push(m, 0x10);
        m.w32(sp, 0x10);
        m.w32(sp + 4, 0x10);
        let r = m.call(0x001a_e2a0, &[sp], &[]).0;
        stack_pop(m, 0x10);
        let x = m.call(0x0011_7d50, &[a1], &[]).0;
        let b = vslot(m, a0, 50, &[x], &[]).0;
        m.w32(r, if b == 0 { 0x002c_44f0 } else { b });
        n -= 1;
        let x = m.call(0x0011_7d50, &[a1], &[]).0;
        let b1 = vslot(m, a0, 50, &[x], &[]).0;
        m.w32(r + 4, b1);
        let v = m.call(0x0011_7d00, &[a1], &[]).0;
        m.w32(r + 8, v);
        let v = m.call(0x0011_7d00, &[a1], &[]).0;
        m.w32(r + 0xc, v);
        m.w32(b1.wrapping_add(0xa8), r);
        m.call(0x0023_03a0, &[a0, b1, 0xf], &[]);
    }
}

/// ANIM-ALL: FUN_0022de88(xf) for the anim-driven kinds 4..0xb (xf +0x98) on the guest bone record: the fields
/// (local rot +0/+0x10/+0x20, pos +0x30, dof +0x94, kind +0x98, sub +0x9a, +0xa4 gen, +0xa8 deadline, +0xac speed,
/// +0xb0 v, +0xc0/+0xd0 targets or +0xc0..+0xef matrix, hermite +0xe0/+0xf0/+0x80/+0x84/*(+0x88)/*(+0x8c)) go
/// through the 1:1 state-struct port `physics::driven::FUN_0022de88_driven`; its FUN_002303a0 re-kind is the guest
/// call. Kind 4 (mirror of +0xa0 + 8) and kinds 1/2 (physics bodies) are left to their owners.
pub fn ee_sub_22de88_driven<M: QMem + ?Sized>(m: &mut M, alo: u32, b: u32) {
    use crate::engine::physics::driven::{DrivenXf, FUN_0022de88_driven};
    let kind = m.r16(b + 0x98);
    // ANIM-ALL R78: FUN_0022de88 case 0x10 -> FUN_0021cb18(alo, xf +0xa8) (mask-tail spring chain)
    if kind == 0x10 && std::env::var_os("SLY2_R78TAILOFF").is_none() {
        let row = m.r32(b + 0xa8);
        if row != 0 {
            ee_sub_21cb18(m, alo, row);
        }
        return;
    }
    if !(5..=0xb).contains(&kind) {
        return;
    }
    let v3 = |m: &mut M, a: u32| [m.rf(a), m.rf(a + 4), m.rf(a + 8)];
    let p88 = m.r32(b + 0x88);
    let p8c = m.r32(b + 0x8c);
    let mut x = DrivenXf {
        rot: [v3(m, b), v3(m, b + 0x10), v3(m, b + 0x20)],
        pos: v3(m, b + 0x30),
        dof: m.r32(b + 0x94) as i32,
        kind,
        sub: m.r16(b + 0x9a),
        v: v3(m, b + 0xb0),
        target: v3(m, b + 0xc0),
        target_v: v3(m, b + 0xd0),
        target_m: [v3(m, b + 0xc0), v3(m, b + 0xd0), v3(m, b + 0xe0)],
        herm_p1: v3(m, b + 0xe0),
        herm_v1: v3(m, b + 0xf0),
        herm_t0: m.rf(b + 0x80),
        herm_t1: m.rf(b + 0x84),
        herm_now: if p88 != 0 { m.rf(p88) } else { 0.0 },
        herm_k: if p8c != 0 { m.rf(p8c) } else { 1.0 },
        deadline: m.rf(b + 0xa8),
        speed: m.r16(b + 0xac),
        frozen: false,
        xf_gen: m.r32(b + 0xa4),
        reached: false,
        reach_k: 0,
    };
    let (now, dt) = (m.rf(0x002d_ded0), m.rf(0x002d_ded4));
    FUN_0022de88_driven(&mut x, now, dt, None);
    for (k, r) in x.rot.iter().enumerate() {
        for (j, &f) in r.iter().enumerate() {
            m.wf(b + 0x10 * k as u32 + 4 * j as u32, f);
        }
    }
    for (j, &f) in x.pos.iter().enumerate() {
        m.wf(b + 0x30 + 4 * j as u32, f);
    }
    for (j, &f) in x.v.iter().enumerate() {
        m.wf(b + 0xb0 + 4 * j as u32, f);
    }
    m.w16(b + 0x9a, x.sub);
    m.w32(b + 0xa4, x.xf_gen);
    if x.reached {
        m.call(0x0023_03a0, &[alo, b, x.reach_k], &[]);
    }
}

/// ANIM-ALL: the guest pose of one ALO after the ASEGA pass: per bone, the top act refetches its segment
/// (FUN_0022dd00, bone act list +0x90) and the driven xf advances (`ee_sub_22de88_driven`); then the bone world
/// matrices (FUN_0022faa0). The FUN_001f3980 ALO update order.
pub fn alo_pose_pass<M: QMem + ?Sized>(m: &mut M, alo: u32) {
    let n = m.r32(alo + 0x50) as i32;
    let base = m.r32(alo + 0x54);
    if n <= 0 || base == 0 {
        return;
    }
    for i in 0..n as u32 {
        let b = base + (i << 8);
        if m.r32(b + 0x90) != 0 {
            m.call(0x0022_dd00, &[alo, b], &[]);
        }
        ee_sub_22de88_driven(m, alo, b);
        // ANIM-ALL R78: FUN_0022de88's common tail (0x22f8b8): the xf world (+0x40) = micro 0x60(parent world,
        // local) at once (local as world without a parent), +0xa4 = max(own, parent): later xfs of the same
        // pass (the FUN_0021cb18 spring rows, their anchor) read this frame's worlds, not last frame's
        if std::env::var_os("SLY2_R78TAILOFF").is_none() {
            let p = m.r32(b + 0xa0);
            let loc = m.rm(b);
            let w = if p != 0 {
                let par = m.rm(p + 0x40);
                let mut o = [[0.0f32; 4]; 4];
                for (i, oi) in o.iter_mut().enumerate() {
                    for k in 0..4 {
                        for c in 0..4 {
                            oi[c] += par[k][c] * loc[i][k];
                        }
                    }
                }
                let (pa, ow) = (m.r32(p + 0xa4) as i32, m.r32(b + 0xa4) as i32);
                if ow < pa {
                    m.w32(b + 0xa4, pa as u32);
                }
                o
            } else {
                loc
            };
            m.wm(b + 0x40, w);
        }
    }
    m.call(0x0022_faa0, &[alo], &[]);
}

/// ANIM-ALL R78: FUN_0021cb18(alo, row) 1:1 — the spring chain of a bone of kind +0x98 == 0x10 (Sly's mask
/// tail, bones 44..52), called from the FUN_0022de88 pose step with row = bone +0xa8. A row is 0x70 bytes:
/// local rot +0/+0x10/+0x20, rest offset +0x30, sim pos +0x40, sim vel +0x50, chain +0x60, xf +0x64. The
/// chain header: +0 rows, +4 row base, +8 substeps, +0xc stiffness, +0x10 damping, +0x14 drag, +0x18 free
/// angle, +0x1c last time, +0x20 gravity, +0x40 last anchor. Once per frame (first row) the substepped chain
/// is integrated from the anchor (xf parent +0x70); every row then aims its xf's local rotation at its sim
/// point (FUN_0011ba08) and bumps the xf generation.
pub fn ee_sub_21cb18<M: QMem + ?Sized>(m: &mut M, alo: u32, row: u32) {
    type V = V4;
    let sub = |a: V, b: V| -> V { [a[0] - b[0], a[1] - b[1], a[2] - b[2], a[3] - b[3]] };
    let madd = |a: V, b: V, s: f32| -> V { [a[0] + b[0] * s, a[1] + b[1] * s, a[2] + b[2] * s, a[3] + b[3] * s] };
    let scale = |a: V, s: f32| -> V { [a[0] * s, a[1] * s, a[2] * s, a[3] * s] };
    let dot3 = |a: V, b: V| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    // vmulax/vmadday/vmaddz.xyz: rows r * v, dest w kept from `w`
    let mul3 = |r: [V; 3], v: V, w: f32| -> V {
        let mut o = [0.0, 0.0, 0.0, w];
        for i in 0..3 {
            o[i] = r[0][i] * v[0] + r[1][i] * v[1] + r[2][i] * v[2];
        }
        o
    };
    let xf = m.r32(row + 0x64);
    let ch = m.r32(row + 0x60);
    let par = m.r32(xf + 0xa0);
    if ch == 0 || xf == 0 || par == 0 {
        return;
    }
    let sp = stack_push(m, 0x80);
    let (now, dt) = (m.rf(0x002d_ded0), m.rf(0x002d_ded4));
    if now != m.rf(ch + 0x1c) {
        let rows = m.r32(ch + 4);
        let (k, vanc, anc, rest);
        if row == rows {
            anc = m.rv(xf + 0x70);
            let fl = m.r32(alo + 0x40);
            vanc = if m.r32(fl) & 0x10 != 0 {
                m.call(0x0022_d348, &[par, 0], &[anc]).1
            } else {
                scale(sub(anc, m.rv(ch + 0x40)), 1.0 / dt)
            };
            m.wv(ch + 0x40, anc);
            let p = [m.rv(par + 0x40), m.rv(par + 0x50), m.rv(par + 0x60)];
            let r = [m.rv(row), m.rv(row + 0x10), m.rv(row + 0x20)];
            let f = [mul3(p, r[0], r[0][3]), mul3(p, r[1], r[1][3]), mul3(p, r[2], p[0][3])];
            rest = mul3(f, m.rv(row + 0x30), p[1][3]);
            k = 0i32;
        } else {
            k = (row.wrapping_sub(rows) as i32) / 0x70;
            for j in 0..k.max(0) as u32 {
                let q = rows + j * 0x70;
                let qx = m.r32(q + 0x64);
                let p30 = m.rv(q + 0x30);
                let w = m.call(0x0022_fc48, &[qx, 0], &[p30]).1;
                m.wv(q + 0x40, w);
                let v = m.call(0x0022_d348, &[qx, 0], &[w]).1;
                m.wv(q + 0x50, v);
            }
            vanc = m.rv(row - 0x20);
            anc = m.rv(row - 0x30);
            rest = if k < 2 { sub(anc, m.rv(par + 0x70)) } else { sub(anc, m.rv(row - 0xa0)) };
        }
        // REPLACED: the original's FUN_0022de88 composes each xf's world inline (0x22f8b8), so the anchor
        // (xf +0x70) is always posed; the port's world pass (FUN_0022faa0) runs after the pose step and an
        // ALO's first frame can still hold an unposed (NaN) world: no step then (the chain stays finite)
        if ![anc, rest, vanc].iter().all(|v| v[..3].iter().all(|x| x.is_finite())) {
            stack_pop(m, 0x80);
            return;
        }
        let ns = m.r32(ch + 8) as i32;
        let h = dt / ns as f32;
        let n = m.r32(ch) as i32;
        let def = m.rv(0x002a_2da0);
        for s in 0..ns.max(0) {
            let t = (s - ns + 1) as f32 * h;
            let mut v11 = vanc;
            let mut v10 = madd([anc[0], anc[1], anc[2], anc[3] + 1.0], vanc, t);
            let mut prev = rest;
            for i in k..n {
                let r = rows + i as u32 * 0x70;
                let mut v08 = m.rv(r + 0x40);
                let vel = m.rv(r + 0x50);
                let mut d = sub(v08, v10);
                let rv = sub(vel, v11);
                let ang = crate::engine::game::q_wirew1c::ee_sub_11e4b0(m, d, prev, sp + 0x10);
                let free = m.rf(ch + 0x18);
                if free < ang {
                    let axis = m.rv(sp + 0x10);
                    m.call(0x0011_b500, &[sp + 0x20], &[axis, [ang - free, 0.0, 0.0, 0.0]]);
                    let rm = [m.rv(sp + 0x20), m.rv(sp + 0x30), m.rv(sp + 0x40)];
                    d = mul3(rm, d, d[3]);
                    v08 = [v10[0] + d[0], v10[1] + d[1], v10[2] + d[2], v10[3] + d[3]];
                }
                let f0 = (-m.rf(ch + 0x10) * h).max(-1.0) + 1.0;
                let f2 = m.rf(ch + 0xc) * ang * h;
                let f1 = (-m.rf(ch + 0x14) * h).max(-1.0);
                let ax = m.rv(sp + 0x10);
                let cr = [ax[1] * d[2] - d[1] * ax[2], ax[2] * d[0] - d[2] * ax[0], ax[0] * d[1] - d[0] * ax[1], ax[3]];
                let g = m.rv(ch + 0x20);
                let mut v07 = madd(madd(madd(scale(vel, f0), rv, f1), cr, f2), g, h);
                v08 = madd([v08[0], v08[1], v08[2], v08[3] + 1.0], v07, h);
                let v03 = sub(v08, v10);
                let len = dot3(v03, v03).sqrt();
                let nrm = if len < f32::from_bits(0x38d1_b717) { def } else { scale(v03, 1.0 / len) };
                m.wv(sp + 0x10, nrm);
                let (dv, dw) = (dot3(v07, nrm), dot3(v11, nrm));
                v07 = madd(sub(v07, scale(nrm, dv)), nrm, dw);
                let p30 = m.rv(r + 0x30);
                let l = dot3(p30, p30).sqrt();
                m.wv(r + 0x50, v07);
                v08 = madd([v10[0], v10[1], v10[2], v10[3] + 1.0], nrm, l);
                v11 = v07;
                m.wv(r + 0x40, v08);
                v10 = v08;
                prev = d;
            }
        }
        m.wf(ch + 0x1c, now);
    }
    let p40 = m.rv(row + 0x40);
    if !p40[..3].iter().all(|v| v.is_finite()) {
        stack_pop(m, 0x80);
        return;
    }
    let loc = m.call(0x0022_fc48, &[0, par], &[p40]).1;
    let to = sub(loc, m.rv(xf + 0x30));
    let r = [m.rv(row), m.rv(row + 0x10), m.rv(row + 0x20)];
    let from = mul3(r, m.rv(row + 0x30), 0.0);
    m.call(0x0011_ba08, &[sp + 0x40], &[from, to]);
    let mm = [m.rv(sp + 0x40), m.rv(sp + 0x50), m.rv(sp + 0x60)];
    let (o0, o1, o2) = (mul3(mm, r[0], 0.0), mul3(mm, r[1], 0.0), mul3(mm, r[2], 0.0));
    m.wv(xf, o0);
    m.wv(xf + 0x10, o1);
    m.wv(xf + 0x20, o2);
    let g = m.r32(0x003e_11e8).wrapping_add(1);
    m.w32(0x003e_11e8, g);
    m.w32(xf + 0xa4, g);
    stack_pop(m, 0x80);
}
