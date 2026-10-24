//! WIRE-W1H7: 1:1 queue ports (against `QMem`) of a batch of live-frontier natives, ported from the EE asm;
//! callees go through `m.call` / `vslot`. QMem convention: integer / pointer args in `a`, quadword and float
//! args in `v`.
#![allow(unused_variables, unused_mut, unused_assignments, clippy::all)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

const GS: u32 = 0x003d_d490; // game state / world

/// FUN_00171458(o): o+0x38 == 0.
pub fn ee_sub_171458<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    (m.r32(a0 + 0x38) == 0) as u32
}

/// FUN_0020f1b8(o): 0 < (s32)o+0x3d84.
pub fn ee_sub_20f1b8<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    (0 < m.r32(a0 + 0x3d84) as i32) as u32
}

/// FUN_001398e0(o): tail to FUN_001a5e28(o).
pub fn ee_sub_1398e0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5e28, &[a0], &[]);
}

/// FUN_00140f50(o): FUN_001a19a0(o + 0xdf0).
pub fn ee_sub_140f50<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_19a0, &[a0.wrapping_add(0xdf0)], &[]);
}

/// FUN_001e5778(o): FUN_0011a380(*(o+0x14) + 0x5c, o) == 0.
pub fn ee_sub_1e5778<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let p = m.r32(a0 + 0x14).wrapping_add(0x5c);
    (m.call(0x0011_a380, &[p, a0], &[]).0 == 0) as u32
}

/// FUN_001fd0c0(_, i): table 0x2c3c40[i] for i < 6 (unsigned), else -1.
pub fn ee_sub_1fd0c0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    if a1 < 6 {
        m.r32((a1 << 2).wrapping_add(0x002c_3c40))
    } else {
        0xffff_ffff
    }
}

/// FUN_001843c0(o): FUN_00231100(o); FUN_00182c70(o + 0x130).
pub fn ee_sub_1843c0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0023_1100, &[a0], &[]);
    m.call(0x0018_2c70, &[a0 + 0x130], &[]);
}

/// FUN_001ff3b8(o): FUN_00231100(o); FUN_00182c70(o + 0xe0).
pub fn ee_sub_1ff3b8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0023_1100, &[a0], &[]);
    m.call(0x0018_2c70, &[a0 + 0xe0], &[]);
}

/// FUN_0011f288(n, vecs): xyz-min of n quadwords starting from the constant at 0x2a2df0 (w kept).
pub fn ee_sub_11f288<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> V4 {
    let mut r = m.rv(0x002a_2df0);
    let (mut n, mut p) = (a0, a1);
    if n as i32 > 0 {
        loop {
            let v = m.rv(p);
            n = n.wrapping_sub(1);
            for k in 0..3 {
                r[k] = if v[k] < r[k] { v[k] } else { r[k] };
            }
            p = p.wrapping_add(0x10);
            if n == 0 {
                break;
            }
        }
    }
    r
}

/// FUN_00185bf8(o): FUN_00185130(o); o+0x108 = *0x2a5690.
pub fn ee_sub_185bf8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0018_5130, &[a0], &[]);
    let f = m.rf(0x002a_5690);
    m.wf(a0 + 0x108, f);
}

/// FUN_001520c8(o): FUN_001f2d48(o); FUN_001524a0(o, 0, 0).
pub fn ee_sub_1520c8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001f_2d48, &[a0], &[]);
    m.call(0x0015_24a0, &[a0, 0, 0], &[]);
}

/// FUN_001fb060(o): --(u16)o+0x4e; at 0 -> FUN_001fb658(o, 6, 4).
pub fn ee_sub_1fb060<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let v = (m.r16(a0 + 0x4e) as u32).wrapping_sub(1);
    m.w16(a0 + 0x4e, v as u16);
    if v & 0xffff == 0 {
        m.call(0x001f_b658, &[a0, 6, 4], &[]);
    }
}

/// FUN_001a5bf0(o): FUN_00132b10(o, 0x30) ? o : *(o+0x10).
pub fn ee_sub_1a5bf0<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    if m.call(0x0013_2b10, &[a0, 0x30], &[]).0 != 0 {
        a0
    } else {
        m.r32(a0 + 0x10)
    }
}

/// FUN_001bc160(o): FUN_001a5800(o); FUN_0011a220(0x3e1a4c, o).
pub fn ee_sub_1bc160<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5800, &[a0], &[]);
    m.call(0x0011_a220, &[0x003e_1a4c, a0], &[]);
}

/// FUN_00118420(o): FUN_00117968(o, FUN_00117c20(o), 0).
pub fn ee_sub_118420<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let v = m.call(0x0011_7c20, &[a0], &[]).0;
    m.call(0x0011_7968, &[a0, v, 0], &[]);
}

/// FUN_001c20b0(o): FUN_001a5848(o); ++GS+0x3d74, copied to *(GS+0x45cc)+0x28.
pub fn ee_sub_1c20b0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5848, &[a0], &[]);
    let v = m.r32(GS + 0x3d74).wrapping_add(1);
    let p = m.r32(GS + 0x45cc);
    m.w32(GS + 0x3d74, v);
    m.w32(p + 0x28, v);
}

/// FUN_001362f8(o, k): on change: o+0x16bc = k, o+0x16c0 = *0x2dded0; k == 0 -> FUN_00136f48(o+0x1710, -1).
pub fn ee_sub_1362f8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if a1 != m.r32(a0 + 0x16bc) {
        m.w32(a0 + 0x16bc, a1);
        let f = m.rf(0x002d_ded0);
        m.wf(a0 + 0x16c0, f);
        if a1 == 0 {
            m.call(0x0013_6f48, &[a0 + 0x1710, 0xffff_ffff], &[]);
        }
    }
}

/// FUN_0013ceb8(o): FUN_00183f08(o); +0x184 = 700, +0x18c = 15.0, +0x188 = -1.
pub fn ee_sub_13ceb8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0018_3f08, &[a0], &[]);
    m.w32(a0 + 0x184, 0x2bc);
    m.wf(a0 + 0x18c, 15.0);
    m.w32(a0 + 0x188, 0xffff_ffff);
}

/// FUN_001b0c68(o): FUN_001f2d48(o); FUN_001305b8(0, 0, 0, o+0x1c4, 0.0, 0.0).
pub fn ee_sub_1b0c68<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001f_2d48, &[a0], &[]);
    m.call(0x0013_05b8, &[0, 0, 0, a0 + 0x1c4], &[f4(0.0), f4(0.0)]);
}

/// FUN_00161958(o): virtual slot 49 of o+0x3bc4 (if any); o+0x476c = 0.
pub fn ee_sub_161958<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let p = m.r32(a0 + 0x3bc4);
    if p != 0 {
        vslot(m, p, 49, &[], &[]);
    }
    m.w32(a0 + 0x476c, 0);
}

/// FUN_001c0378(o): FUN_00231100(o); +0x110 = -1; FUN_0011a1f8(o+0x118, 0x60); +0x124 = -1.
pub fn ee_sub_1c0378<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0023_1100, &[a0], &[]);
    m.w32(a0 + 0x110, 0xffff_ffff);
    m.call(0x0011_a1f8, &[a0 + 0x118, 0x60], &[]);
    m.w32(a0 + 0x124, 0xffff_ffff);
}

/// FUN_00171f58(o): FUN_001eef68(o); FUN_00209890(*(o+0x10), 0x1f); +0x30 = -1; +0x40 = +0x38 = *0x2a5370.
pub fn ee_sub_171f58<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001e_ef68, &[a0], &[]);
    let p = m.r32(a0 + 0x10);
    m.call(0x0020_9890, &[p, 0x1f], &[]);
    let f = m.rf(0x002a_5370);
    m.w32(a0 + 0x30, 0xffff_ffff);
    m.wf(a0 + 0x40, f);
    m.wf(a0 + 0x38, f);
}

/// FUN_0019f540(o, q): zeroed 0x20-byte local (FUN_002333b4), quadword q at +0x10, FUN_0019f590(o, local).
pub fn ee_sub_19f540<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: V4) {
    let s = stack_push(m, 0x20);
    m.call(0x0023_33b4, &[s, 0, 0x20], &[]);
    m.wv(s + 0x10, a1);
    m.call(0x0019_f590, &[a0, s], &[]);
    stack_pop(m, 0x20);
}

/// FUN_0021d278(o): FUN_00231100(o); FUN_001a6ca8(o, 3); +0xe8 = 0x17e, +0xf4 = -1, +0xec = 25.0.
pub fn ee_sub_21d278<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0023_1100, &[a0], &[]);
    m.call(0x001a_6ca8, &[a0, 3], &[]);
    m.w32(a0 + 0xe8, 0x17e);
    m.w32(a0 + 0xf4, 0xffff_ffff);
    m.wf(a0 + 0xec, 25.0);
}

/// FUN_0021d3d8(o): FUN_00231100(o); +0xf4 = 1, +0xe8 = 2000.0, +0xec = -800.0, +0xf0 = 800.0.
pub fn ee_sub_21d3d8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0023_1100, &[a0], &[]);
    m.w32(a0 + 0xf4, 1);
    m.wf(a0 + 0xe8, 2000.0);
    m.wf(a0 + 0xec, -800.0);
    m.wf(a0 + 0xf0, 800.0);
}

/// FUN_001635a0(o): FUN_001a5270(o); field resets; ++GS+0x3afc.
pub fn ee_sub_1635a0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5270, &[a0], &[]);
    m.w32(a0 + 0xf4, 0);
    for off in [0xc0u32, 0x54, 0xf0, 0x5c, 0xb8, 0xbc] {
        m.w32(a0 + off, 0xffff_ffff);
    }
    let v = m.r32(GS + 0x3afc).wrapping_add(1);
    m.w32(GS + 0x3afc, v);
}

/// FUN_001f00c8(o): FUN_001a5c48(o); with p = o+0x74: FUN_001f0380(o, FUN_0015e938(GS, 4, p[0], o), p[1], p[2]).
pub fn ee_sub_1f00c8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5c48, &[a0], &[]);
    let p = m.r32(a0 + 0x74);
    if p != 0 {
        let x = m.r32(p);
        let v = m.call(0x0015_e938, &[GS, 4, x, a0], &[]).0;
        let p = m.r32(a0 + 0x74);
        let (b, c) = (m.r32(p + 4), m.r32(p + 8));
        m.call(0x001f_0380, &[a0, v, b, c], &[]);
    }
}

/// FUN_001ed278(p): when *0x3e12c8: copy 4 bytes of p to local b; FUN_0011dcd8(b, a); FUN_001ed2d8(a);
/// FUN_0011dd30(a, p[3], p).
pub fn ee_sub_1ed278<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    if m.r32(0x003e_12c8) != 0 {
        let s = stack_push(m, 0x20);
        for i in 0..4 {
            let b = m.r8(a0 + i);
            m.w8(s + 0x10 + i, b);
        }
        m.call(0x0011_dcd8, &[s + 0x10, s], &[]);
        m.call(0x001e_d2d8, &[s], &[]);
        let b = m.r8(a0 + 3) as u32;
        m.call(0x0011_dd30, &[s, b, a0], &[]);
        stack_pop(m, 0x20);
    }
}

/// FUN_00150ef0(o): +0x20 = FUN_00150780(o), +0x30 = FUN_001507f8(o); +0xac = +0x18 when 0 <= it;
/// +0x50/+0x60 = +0x80/+0x90, +0x70 = +0xa0.
pub fn ee_sub_150ef0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let r = m.call(0x0015_0780, &[a0], &[]).1;
    m.wv(a0 + 0x20, r);
    let r = m.call(0x0015_07f8, &[a0], &[]).1;
    let f1 = m.rf(a0 + 0x18);
    m.wv(a0 + 0x30, r);
    if 0.0 <= f1 {
        m.wf(a0 + 0xac, f1);
    }
    let (x, y) = (m.rv(a0 + 0x80), m.rv(a0 + 0x90));
    let f = m.rf(a0 + 0xa0);
    m.wv(a0 + 0x50, x);
    m.wf(a0 + 0x70, f);
    m.wv(a0 + 0x60, y);
}

/// o+0x40 flag word update: bit0 clear -> it is a pointer, update *ptr; else update the inline word.
fn flag40<M: QMem + ?Sized>(m: &mut M, o: u32, f: impl Fn(u32) -> u32) {
    let a = m.r32(o + 0x40);
    if (a ^ 1) & 1 != 0 {
        let v = m.r32(a);
        m.w32(a, f(v));
    } else {
        m.w32(o + 0x40, f(a));
    }
}

/// FUN_001c6880(o): FUN_001c5e48(o); when +0x150 == 0: +0x238 -= *0x2dded4, at <= 0 -> virtual slot 75 (5).
pub fn ee_sub_1c6880<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001c_5e48, &[a0], &[]);
    if m.r32(a0 + 0x150) == 0 {
        let f = m.rf(a0 + 0x238) - m.rf(0x002d_ded4);
        m.wf(a0 + 0x238, f);
        if f <= 0.0 {
            vslot(m, a0, 75, &[5], &[]);
        }
    }
}

/// FUN_001c8280(o): FUN_001c3d70(o); field resets; flag |= 2.
pub fn ee_sub_1c8280<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001c_3d70, &[a0], &[]);
    m.w32(a0 + 0x194, 0xffff_ffff);
    m.w32(a0 + 0x1f0, 0x7b);
    for off in [0x1fcu32, 0x1f8, 0x1f4, 0x200, 0x180] {
        m.w32(a0 + off, 0xffff_ffff);
    }
    flag40(m, a0, |v| v | 2);
}

/// FUN_00223f70(o): FUN_001f2108(o); +0x178 = -10, +0x170 = 3000.0, +0x150 = +0x158 = -1; flag |= 6.
pub fn ee_sub_223f70<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001f_2108, &[a0], &[]);
    m.w32(a0 + 0x178, (-10i32) as u32);
    m.wf(a0 + 0x170, f32::from_bits(0x453b_8000));
    m.w32(a0 + 0x150, 0xffff_ffff);
    m.w32(a0 + 0x158, 0xffff_ffff);
    flag40(m, a0, |v| v | 6);
}

/// FUN_001f6b48(o, k): FUN_001f2ad0(o, e) for every 0xb0-byte entry e of +0xe4[+0xe0] with e+0xc == k.
pub fn ee_sub_1f6b48<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let mut i: i32 = 0;
    let mut off: u32 = 0;
    if (m.r32(a0 + 0xe0) as i32) > 0 {
        loop {
            let e = m.r32(a0 + 0xe4).wrapping_add(off);
            if m.r32(e + 0xc) == a1 {
                m.call(0x001f_2ad0, &[a0, e], &[]);
            }
            i += 1;
            off = off.wrapping_add(0xb0);
            if !(i < m.r32(a0 + 0xe0) as i32) {
                break;
            }
        }
    }
}

/// FUN_001cb658(o): with p = o+0x3c (and p+0x24): table lookup via FUN_001c9950 / FUN_001c9958; else 0.
pub fn ee_sub_1cb658<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let p = m.r32(a0 + 0x3c);
    if p == 0 || m.r32(p + 0x24) == 0 {
        return 0;
    }
    let x = m.r32(p);
    let a = m.call(0x001c_9950, &[x], &[]).0;
    let p = m.r32(a0 + 0x3c);
    let x = m.r32(p);
    let b = m.call(0x001c_9958, &[x], &[]).0;
    let t = m.r32(0x003e_1398).wrapping_add(a.wrapping_mul(0x6c));
    let base = m.r32(t + 0xc);
    let arr = m.r32(t + 0x68);
    m.r32(arr.wrapping_add(b.wrapping_sub(base).wrapping_mul(0x50)))
}

/// FUN_001746b8(o): FUN_001a5848(o); FUN_00173928(o, e, 0, 0) for each 0x30-byte entry e of +0x60[+0x5c] with *e.
pub fn ee_sub_1746b8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5848, &[a0], &[]);
    if m.r32(a0 + 0x60) == 0 {
        return;
    }
    if (m.r32(a0 + 0x5c) as i32) > 0 {
        let mut i: i32 = 0;
        let mut off: u32 = 0;
        loop {
            let e = m.r32(a0 + 0x60).wrapping_add(off);
            if m.r32(e) != 0 {
                m.call(0x0017_3928, &[a0, e, 0, 0], &[]);
            }
            i += 1;
            off = off.wrapping_add(0x30);
            if !(i < m.r32(a0 + 0x5c) as i32) {
                break;
            }
        }
    }
}

/// FUN_001591b0(o): FUN_00231100(o); field init; FUN_0011a1f8(o+0x11c, 0x128); FUN_001a6ca8(o, 0); flag |= 2.
pub fn ee_sub_1591b0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0023_1100, &[a0], &[]);
    m.w32(a0 + 0x118, 0xffff_ffff);
    m.w32(a0 + 0xf4, 0xffff_ffff);
    m.w32(a0 + 0xf0, 0xffff_ffff);
    m.wf(a0 + 0xf8, 10.0);
    m.wf(a0 + 0x148, -1.0);
    m.wf(a0 + 0xfc, 10.0);
    m.call(0x0011_a1f8, &[a0 + 0x11c, 0x128], &[]);
    m.call(0x001a_6ca8, &[a0, 0], &[]);
    flag40(m, a0, |v| v | 2);
}

/// FUN_001a3360(o): FUN_00231100(o); field init; FUN_001a6ca8(o, 1); flag &= ~0x20.
pub fn ee_sub_1a3360<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0023_1100, &[a0], &[]);
    m.w32(a0 + 0xe0, 0);
    m.w32(a0 + 0x118, 0xffff_ffff);
    m.w32(a0 + 0xe4, 0xffff_ffff);
    m.wf(a0 + 0x164, 200.0);
    m.wf(a0 + 0x168, 2000.0);
    m.wf(a0 + 0x154, 240.0);
    m.w32(a0 + 0x158, 1);
    m.call(0x001a_6ca8, &[a0, 1], &[]);
    flag40(m, a0, |v| v & !0x20);
}

/// FUN_0020ad38(o): FUN_001a5270(o); field init; FUN_002090d0(o+0xb0, +0xac).
pub fn ee_sub_20ad38<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5270, &[a0], &[]);
    let f6 = m.rf(0x002a_88c8);
    let a1 = m.r32(a0 + 0xac);
    m.w32(a0 + 0x50, 0);
    m.wf(a0 + 0x54, 300.0);
    m.wf(a0 + 0x58, f32::from_bits(0xc59c_4000));
    m.wf(a0 + 0x5c, f32::from_bits(0x459c_4000));
    m.wf(a0 + 0x60, f32::from_bits(0x3e4c_cccd));
    m.wf(a0 + 0x64, 0.5);
    m.wf(a0 + 0x68, f32::from_bits(0x3e32_b8c4));
    m.w32(a0 + 0xa8, 1);
    m.w32(a0 + 0x84, 0xffff_ffff);
    m.wf(a0 + 0x6c, f6);
    m.w32(a0 + 0x7c, 0xffff_ffff);
    m.w32(a0 + 0x80, 0xffff_ffff);
    m.call(0x0020_90d0, &[a0 + 0xb0, a1], &[]);
}

/// FUN_00220ed0(o): FUN_00231100(o); field init (+0x1a0 = *0x2c41c0, +0x1b0 = 8 bytes of 0x2c41d0);
/// FUN_001a6ca8(o, 1); flag |= 2.
pub fn ee_sub_220ed0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0023_1100, &[a0], &[]);
    m.wf(a0 + 0x170, 2.0);
    m.wf(a0 + 0x174, 1.0);
    m.wf(a0 + 0x178, 2.0);
    m.wf(a0 + 0x17c, 1.0);
    m.w32(a0 + 0x180, 0xffff_ffff);
    let q = m.rv(0x002c_41c0);
    m.wv(a0 + 0x1a0, q);
    for i in 0..8 {
        let b = m.r8(0x002c_41d0 + i);
        m.w8(a0 + 0x1b0 + i, b);
    }
    m.w32(a0 + 0x13c, 0xffff_ffff);
    m.w32(a0 + 0x164, 7);
    m.call(0x001a_6ca8, &[a0, 1], &[]);
    flag40(m, a0, |v| v | 2);
}

/// FUN_0012cb98(k): allocate (FUN_001ae2a0 with {size, 0x10}) and set the vtable for kind 0/1/2; else 0.
pub fn ee_sub_12cb98<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let (size, vt) = match a0 as i32 {
        0 => (8u32, 0x002c_a8a0u32),
        1 => (0xc, 0x002c_a8c0),
        2 => (0x14, 0x002c_b588),
        _ => return 0,
    };
    let s = stack_push(m, 0x10);
    m.w32(s + 4, 0x10);
    m.w32(s, size);
    let v = m.call(0x001a_e2a0, &[s], &[]).0;
    m.w32(v, vt);
    stack_pop(m, 0x10);
    v
}

/// FUN_0012ed20(o): FUN_001a6178(o); for each 0x14-byte entry e of +0xe8[+0xe4]: s = FUN_001fbb90(e[0]),
/// configure via FUN_001fb2f8/4f0/1f8/2c0, FUN_00258040(+0xec + 4i, s).
pub fn ee_sub_12ed20<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_6178, &[a0], &[]);
    let mut i: i32 = 0;
    let mut off: u32 = 0;
    if (m.r32(a0 + 0xe4) as i32) <= 0 {
        return;
    }
    loop {
        let e = m.r32(a0 + 0xe8).wrapping_add(off);
        let x = m.r32(e);
        off = off.wrapping_add(0x14);
        let s = m.call(0x001f_bb90, &[x], &[]).0;
        m.call(0x001f_b2f8, &[s, a0], &[]);
        let (f12, f13) = (m.rf(e + 4), m.rf(e + 8));
        m.call(0x001f_b4f0, &[s], &[f4(f12), f4(f13)]);
        let f = m.rf(e + 0xc);
        m.call(0x001f_b1f8, &[s], &[f4(f)]);
        let f = m.rf(e + 0x10);
        m.call(0x001f_b2c0, &[s], &[f4(f)]);
        let p = ((i as u32) << 2).wrapping_add(m.r32(a0 + 0xec));
        i += 1;
        m.call(0x0025_8040, &[p, s], &[]);
        if !(i < m.r32(a0 + 0xe4) as i32) {
            break;
        }
    }
}

/// FUN_00136040(o, k): state change of +0x16b0; picks an anim (FUN_001305b8) and calls virtual slot 89.
pub fn ee_sub_136040<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if a1 == m.r32(a0 + 0x16b0) {
        return;
    }
    m.w32(a0 + 0x16b0, a1);
    let mut s1: u32 = 1;
    let mut s2: u32 = 0;
    let k = a1 as i32;
    if k == 1 {
        s2 = m.r32(a0 + 0x15f0);
        s1 = 9;
    } else if k < 2 {
        if k == 0 {
            s2 = m.r32(a0 + 0x14e4);
            m.w32(a0 + 0x194, 0);
            m.w32(a0 + 0x198, 0);
            m.w32(a0 + 0x190, 0);
        }
    } else if k == 2 {
        s2 = m.r32(a0 + 0x14e8);
        if s2 == 0 {
            s2 = m.r32(a0 + 0x15d4);
        }
        s1 = 9;
    }
    m.call(0x001f_15b8, &[a0 + 0x1034], &[]);
    m.call(0x0013_05b8, &[s2, a0, s1, a0 + 0x1030], &[f4(0.0), f4(1.0)]);
    vslot(m, a0, 89, &[], &[]);
}

/// FUN_001badf0(o, e): event filter; FUN_001baed8(o, e[0], e[0xc]) when accepted.
pub fn ee_sub_1badf0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    if m.call(0x001b_be48, &[a0, a1], &[]).0 != 0 {
        return 0;
    }
    if m.r32(a1 + 0x44) == 0 {
        return 0;
    }
    if m.r64(a1 + 0x40) & (0xc000u64 << 20) != (0x8000u64 << 20) {
        return 1;
    }
    let v1 = m.r32(a0 + 0x150);
    let take = if v1 == 0 || m.r32(a1) == a0 {
        true
    } else {
        let mut v1 = v1 as i32;
        let mut s0: i32 = 0;
        loop {
            if !(s0 < v1) {
                break false;
            }
            let t = m.r32(a0 + 0x154).wrapping_add((s0 as u32) << 3);
            let x = m.r32(a1);
            let y = m.r32(t);
            if m.call(0x0013_2b10, &[x, y], &[]).0 == 0 {
                v1 = m.r32(a0 + 0x150) as i32;
                s0 += 1;
                continue;
            }
            break s0 < m.r32(a0 + 0x150) as i32;
        }
    };
    if !take {
        return 0;
    }
    let a2 = m.r32(a1 + 0xc);
    let x = m.r32(a1);
    m.call(0x001b_aed8, &[a0, x, a2], &[]);
    1
}

/// FUN_001b0b78(o): FUN_001f2108(o); field init incl. 6 {-1, _, 0x2c0230[i]} triples at +0x17c; flag |= 2.
pub fn ee_sub_1b0b78<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001f_2108, &[a0], &[]);
    m.wf(a0 + 0x150, 400.0);
    m.wf(a0 + 0x154, 800.0);
    m.wf(a0 + 0x158, 2.0);
    m.wf(a0 + 0x160, 6.0);
    m.wf(a0 + 0x164, 10.0);
    m.wf(a0 + 0x168, 1.0);
    m.wf(a0 + 0x15c, 6.0);
    let mut n: i32 = 5;
    let mut src: u32 = 0x002c_0230;
    let mut dst = a0 + 0x17c;
    loop {
        m.w32(dst, 0xffff_ffff);
        n -= 1;
        let v = m.r32(src);
        src += 4;
        m.w32(dst + 8, v);
        dst += 0xc;
        if n < 0 {
            break;
        }
    }
    let a = m.r32(a0 + 0x40);
    m.wf(a0 + 0x1cc, 300.0);
    m.wf(a0 + 0x1d0, 1000.0);
    m.w32(a0 + 0x230, 0xffff_ffff);
    m.w32(a0 + 0x220, 0xffff_ffff);
    if (a ^ 1) & 1 != 0 {
        let v = m.r32(a) | 2;
        m.w32(a, v);
    } else {
        m.w32(a0 + 0x40, a | 2);
    }
}

/// FUN_00154c90(o): override *(*0x3e0f9c + 0xa8), else mean of the three FUN_002581c0 values of o+8/0xc/0x10.
pub fn ee_sub_154c90<M: QMem + ?Sized>(m: &mut M, a0: u32) -> f32 {
    let g = m.r32(0x003e_0f9c);
    if g != 0 {
        let p = m.r32(g + 0xa8);
        return m.rf(p);
    }
    if m.r32(a0 + 8) == 0 || m.r32(a0 + 0xc) == 0 || m.r32(a0 + 0x10) == 0 {
        return 0.0;
    }
    let s = stack_push(m, 0x10);
    let mut v = [0.0f32; 3];
    for (k, off) in [8u32, 0xc, 0x10].into_iter().enumerate() {
        let a1 = m.r32(a0 + off);
        m.call(0x0025_81c0, &[s, a1, 0x003d_4a60], &[]);
        v[k] = m.rf(s + 4);
        m.call(0x0012_9ec8, &[s], &[]);
    }
    stack_pop(m, 0x10);
    (v[0] + v[1] + v[2]) * f32::from_bits(0x3eaa_aaab)
}

/// FUN_00249888(k, x, y): kernel sin (fdlibm __kernel_sin, single precision).
pub fn ee_sub_249888<M: QMem + ?Sized>(m: &mut M, a0: u32, x: f32, y: f32) -> f32 {
    let ix = x.to_bits() & 0x7fff_ffff;
    if !(ix as i32 > 0x31ff_ffff) && (x as i32) == 0 {
        return x;
    }
    let z = x * x;
    let mut r = z * f32::from_bits(0x2f2e_c9d3) + f32::from_bits(0xb2d7_2f34);
    r = z * r + f32::from_bits(0x3638_ef1b);
    r = z * r + f32::from_bits(0xb950_0d01);
    r = z * r;
    let v = z * x;
    let r = r + f32::from_bits(0x3c08_8889);
    let s1 = f32::from_bits(0xbe2a_aaab);
    if a0 == 0 {
        x + v * (z * r + s1)
    } else {
        x - (((z * (y * 0.5 - v * r)) - y) - v * s1)
    }
}

/// FUN_00143ee0(o, id, a2, a3): id == -1 -> FUN_00143ff8; else resolve id (FUN_0015e938) and start it by
/// class 0x6d (FUN_001305b8) or 0x6f (FUN_001f1448), after virtual slot 87 (a3).
pub fn ee_sub_143ee0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32) {
    if a1 == 0xffff_ffff {
        m.call(0x0014_3ff8, &[a0, a1, a2, a3], &[]);
        return;
    }
    let s1 = m.call(0x0015_e938, &[GS, 1, a1, a0], &[]).0;
    if s1 == 0 {
        return;
    }
    if m.call(0x0013_2b10, &[s1, 0x6d], &[]).0 != 0 {
        vslot(m, a0, 87, &[a3], &[]);
        m.call(0x001f_15b8, &[a0 + 0x1034], &[]);
        m.call(0x0013_05b8, &[s1, a0, a2, a0 + 0x1030], &[f4(0.0), f4(1.0)]);
    } else if m.call(0x0013_2b10, &[s1, 0x6f], &[]).0 != 0 {
        vslot(m, a0, 87, &[a3], &[]);
        m.call(0x0013_0658, &[a0 + 0x1030], &[]);
        m.call(0x001f_1448, &[s1, a0, 0xffff_ffff, a2, a0 + 0x1034], &[]);
    }
}

/// FUN_0012b3b0(k): allocate (FUN_001ae2a0 with {size, 0x10}) and set the vtable for kind 0..5; else 0.
pub fn ee_sub_12b3b0<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let (size, vt) = match a0 {
        0 => (0x10u32, 0x002c_a9a0u32),
        1 => (0xc, 0x002c_a978),
        2 => (0xc, 0x002c_a950),
        3 => (0x30, 0x002c_a928),
        4 => (0x30, 0x002c_a900),
        5 => (0x20, 0x002c_b5a8),
        _ => return 0,
    };
    let s = stack_push(m, 0x10);
    m.w32(s + 4, 0x10);
    m.w32(s, size);
    let v = m.call(0x001a_e2a0, &[s], &[]).0;
    m.w32(v, vt);
    stack_pop(m, 0x10);
    v
}

/// FUN_001e64c8(o): lerp between +0xec[i] and +0xec[i+1] (i from FUN_001e7668), by the ratio or (when
/// +0xe0) by the 10-step breakpoint table at +0x14c.
pub fn ee_sub_1e64c8<M: QMem + ?Sized>(m: &mut M, a0: u32) -> f32 {
    let s = stack_push(m, 0x10);
    let i = m.call(0x001e_7668, &[a0, s, s | 4], &[]).0;
    let t;
    if m.r32(a0 + 0xe0) == 0 {
        t = m.rf(s) / m.rf(s + 4);
    } else {
        m.call(0x001e_5fc8, &[a0, i], &[]);
        let f1 = m.rf(s);
        let a2 = a0 + 0x14c;
        let mut k: i32 = 1;
        if !(f1 < m.rf(a0 + 0x150)) {
            let mut v1 = a0 + 0x150;
            k = 2;
            loop {
                if !(k < 10) {
                    break;
                }
                v1 = v1.wrapping_add(4);
                if f1 < m.rf(v1) {
                    break;
                }
                k += 1;
            }
        }
        k -= 1;
        let x = m.rf(s);
        let lo = m.rf(a2.wrapping_add((k as u32) << 2));
        let hi = m.rf(a2.wrapping_add(((k + 1) as u32) << 2));
        let f3 = k as f32 + (x - lo) / (hi - lo);
        t = f3 * f32::from_bits(0x3dcc_cccd);
    }
    let e = (i << 2).wrapping_add(m.r32(a0 + 0xec));
    let (e0, e1) = (m.rf(e), m.rf(e + 4));
    stack_pop(m, 0x10);
    (1.0 - t) * e0 + t * e1
}

/// FUN_0017ecf0(o, force): step the value +0x38 by ±step (clamped / snapped through the +0x30 object),
/// then (unless unchanged and !force) apply via FUN_001e6f10 and FUN_0017dd38.
pub fn ee_sub_17ecf0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let x = m.r32(a0 + 0x10);
    let mut f12 = if m.call(0x0013_2b10, &[x, 0x3a], &[]).0 != 0 {
        m.rf(0x002b_e7bc)
    } else {
        m.rf(0x002b_e7c0)
    };
    if m.r32(a0 + 0x34) == 0 {
        f12 = -f12;
    }
    let p = m.r32(a0 + 0x30);
    let mut f20 = m.rf(a0 + 0x38) + f12;
    if m.r32(p + 0xe4) == 0 {
        let lo = m.call(0x001e_6620, &[p], &[]).1[0];
        let p2 = m.r32(a0 + 0x30);
        let hi = m.call(0x001e_6648, &[p2], &[]).1[0];
        if f20 < lo {
            f20 = lo;
        } else if hi < f20 {
            f20 = hi;
        }
    } else {
        f20 = m.call(0x001e_66c8, &[p], &[f4(f20)]).1[0];
    }
    if m.r32(a0 + 0x58) == 0 {
        let q = m.r32(a0 + 0x30);
        let q = m.r32(q + 0x180);
        m.call(0x001e_a000, &[q], &[]);
        let q = m.r32(a0 + 0x30);
        let q = m.r32(q + 0x180);
        let v = m.call(0x001e_a3b0, &[q], &[f4(f20)]).0;
        if v != 0 {
            let f = if m.r32(a0 + 0x34) == 0 { m.rf(v) } else { m.rf(v + 4) };
            let p = m.r32(a0 + 0x30);
            f20 = m.call(0x001e_66c8, &[p], &[f4(f)]).1[0];
        }
    }
    if a1 == 0 {
        let d = (m.rf(a0 + 0x38) - f20).abs();
        if d < m.rf(0x002b_e7c4) {
            return;
        }
    }
    let p = m.r32(a0 + 0x30);
    m.wf(a0 + 0x38, f20);
    m.call(0x001e_6f10, &[p, a0 + 0x40, 0], &[f4(f20)]);
    let s0 = m.r32(a0 + 0x18);
    let s = stack_push(m, 0x20);
    m.call(0x0025_62a8, &[s], &[]);
    let q = m.rv(a0 + 0x40);
    m.wv(s + 0x10, q);
    m.call(0x0017_dd38, &[s0, s], &[]);
    stack_pop(m, 0x20);
}

/// FUN_00166270(src, xf, dst): clone a point / normal set (FUN_001ae238 allocs), transforming points and
/// normals by the matrix xf (if any); each normal entry goes to FUN_00119390(e, n, point[e+0x1c]).
pub fn ee_sub_166270<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    let s = stack_push(m, 0x50);
    let n = m.r32(a0);
    m.w32(a2, n);
    m.w32(s + 4, 0x10);
    let n = m.r32(a0);
    let b = m.r32(a0 + 4);
    m.w32(s, n << 4);
    let v = m.call(0x001a_e238, &[s, b], &[]).0;
    m.w32(a2 + 4, v);
    m.w32(s + 4, 0x10);
    let n = m.r32(a0 + 8);
    m.w32(a2 + 8, n);
    let n = m.r32(a0 + 8);
    let b = m.r32(a0 + 0xc);
    m.w32(s, n << 5);
    let v = m.call(0x001a_e238, &[s, b], &[]).0;
    m.w32(a2 + 0xc, v);
    let x = m.r32(a0 + 0x10);
    m.w32(a2 + 0x10, x);
    let x = m.r32(a0 + 0x14);
    m.w32(a2 + 0x14, x);
    if a1 != 0 {
        let mx = [m.rv(a1), m.rv(a1 + 0x10), m.rv(a1 + 0x20), m.rv(a1 + 0x30)];
        for k in 0..4 {
            m.wv(s + 0x10 + 0x10 * k as u32, mx[k]);
        }
        let mut i: i32 = 0;
        if (m.r32(a2) as i32) > 0 {
            loop {
                let p = m.r32(a2 + 4).wrapping_add((i as u32) << 4);
                i += 1;
                let (m0, m1, m2, m3) = (m.rv(s + 0x10), m.rv(s + 0x20), m.rv(s + 0x30), m.rv(s + 0x40));
                let v = m.rv(p);
                let mut r = m0;
                for k in 0..3 {
                    r[k] = m0[k] * v[0] + m1[k] * v[1] + m2[k] * v[2] + m3[k] * 1.0;
                }
                m.wv(p, r);
                if !(i < m.r32(a2) as i32) {
                    break;
                }
            }
        }
        if (m.r32(a2 + 8) as i32) > 0 {
            let mut j: i32 = 0;
            loop {
                let e = m.r32(a2 + 0xc).wrapping_add((j as u32) << 5);
                j += 1;
                let (m0, m1, m2) = (m.rv(s + 0x10), m.rv(s + 0x20), m.rv(s + 0x30));
                let k1c = m.r32(e + 0x1c);
                let v = m.rv(e);
                let mut nrm = v;
                for k in 0..3 {
                    nrm[k] = m0[k] * v[0] + m1[k] * v[1] + m2[k] * v[2];
                }
                let len = (nrm[0] * nrm[0] + nrm[1] * nrm[1] + nrm[2] * nrm[2]).sqrt();
                let q = 1.0 / len;
                let n3 = [nrm[0] * q, nrm[1] * q, nrm[2] * q, nrm[3] * q];
                let pp = (k1c << 4).wrapping_add(m.r32(a2 + 4));
                let pt = m.rv(pp);
                m.call(0x0011_9390, &[e], &[n3, pt]);
                if !(j < m.r32(a2 + 8) as i32) {
                    break;
                }
            }
        }
    }
    stack_pop(m, 0x50);
}

/// FUN_00205eb8(o): FUN_00203948(o); FUN_00205e70(); spawn +0xf64 (FUN_0013e3c8) with flag &= ~2; big field
/// init; FUN_002081c8(o); FUN_00209768(o+0xfd0, o).
pub fn ee_sub_205eb8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0020_3948, &[a0], &[]);
    m.call(0x0020_5e70, &[], &[]);
    let v = m.call(0x0013_e3c8, &[0x10, 0, 0x288, 0xffff_ffff], &[]).0;
    m.w32(a0 + 0xf64, v);
    m.call(0x001a_6ca8, &[v, 2], &[]);
    let p = m.r32(a0 + 0xf64);
    flag40(m, p, |x| x & !2);
    let b = |x: u32| f32::from_bits(x);
    let f5 = b(0x3f49_0fdc);
    let f6 = b(0x3fa7_8d37);
    let f18 = m.rf(0x002a_8600);
    let (f0, f17, f16) = (300.0f32, 1200.0f32, 800.0f32);
    m.wf(a0 + 0xf50, -f5);
    m.wf(a0 + 0xf54, f5);
    m.wf(a0 + 0xf58, -f6);
    m.wf(a0 + 0xf5c, f6);
    m.wf(a0 + 0x318, f17);
    m.wf(a0 + 0x320, 450.0);
    m.wf(a0 + 0x324, f0);
    m.wf(a0 + 0x328, 600.0);
    m.wf(a0 + 0x2d0, f18);
    m.wf(a0 + 0x370, -100.0);
    m.wf(a0 + 0x374, 350.0);
    m.wf(a0 + 0x378, 1000.0);
    m.wf(a0 + 0x380, -800.0);
    m.wf(a0 + 0x384, f16);
    m.wf(a0 + 0x388, 3000.0);
    m.wf(a0 + 0xe7c, 16.0);
    m.wf(a0 + 0xe80, 20.0);
    m.w32(a0 + 0xe84, 0x32);
    m.wf(a0 + 0xe6c, 5.5);
    m.wf(a0 + 0xe70, 6.5);
    m.w32(a0 + 0x354, 0x0029_ff7e);
    m.wf(a0 + 0xf48, f0);
    m.wf(a0 + 0xf4c, f17);
    m.wf(a0 + 0x314, f16);
    m.wf(a0 + 0x31c, f0);
    m.wf(a0 + 0x2cc, f18);
    for off in [0x364u32, 0x37c, 0xe64, 0xf40] {
        m.w32(a0 + off, 1);
    }
    m.w32(a0 + 0xe68, 0xffff_ffff);
    m.w32(a0 + 0xe78, 0);
    m.w32(a0 + 0x350, 0x0020_f800);
    // 64-bit +0x148: & 0xffff_ffff_fff0_ffff | 0x20000 (only the low word changes)
    let lo = m.r32(a0 + 0x148);
    let a1 = m.r32(0x002c_3cf0);
    let f2 = m.rf(0x002a_8604);
    m.w32(a0 + 0x358, 0xf800);
    m.wf(a0 + 0x360, -10.0);
    m.w32(a0 + 0x4a0, 1);
    m.wf(a0 + 0x260, 1.5);
    m.w32(a0 + 0x148, (lo & 0xfff0_ffff) | 0x2_0000);
    m.w32(a0 + 0x3f0, (-200i32) as u32);
    m.w32(a0 + 0xfbc, 0xffff_ffff);
    m.wf(a0 + 0xff4, f2);
    m.w32(a0 + 0x1000, a1);
    m.w32(a0 + 0x3d8, 0xffff_ffff);
    m.w32(a0 + 0xf80, 0xffff_ffff);
    m.w32(a0 + 0x3fc, 0xffff_ffff);
    m.call(0x0020_81c8, &[a0], &[]);
    m.call(0x0020_9768, &[a0 + 0xfd0, a0], &[]);
}

/// FUN_0014c9f8(na, a, nb, b, cap, out, mode): merge two sorted 0x60-byte span lists by their (+0x40 / +0x44)
/// keys into a scratch buffer (FUN_001ae318), emitting spans covered by >= 1 (mode 0) / 2 (mode 1) lists;
/// copies the result to `out` (FUN_001ae5a0) and returns the span count.
/// Note: for mode not 0/1 the asm leaves the threshold in the caller's s7; modeled as 0.
pub fn ee_sub_14c9f8<M: QMem + ?Sized>(
    m: &mut M,
    a0: u32,
    a1: u32,
    a2: u32,
    a3: u32,
    t0: u32,
    t1: u32,
    t2: u32,
) -> u32 {
    let sp = stack_push(m, 0x30);
    m.call(0x001a_e478, &[sp], &[]);
    m.w32(sp + 0x10, t0.wrapping_mul(0x60));
    m.w32(sp + 0x14, 0x10);
    let buf = m.call(0x001a_e318, &[sp + 0x10], &[]).0;
    let (mut ka, mut kb): (u32, u32) = (0, 0);
    let mut inside: u32 = 0;
    m.w32(sp + 0x20, a1);
    m.w32(sp + 0x24, a3);
    let end_b = a2.wrapping_mul(0x60).wrapping_add(a3);
    let end_a = a0.wrapping_mul(0x60).wrapping_add(a1);
    let mut out = buf;
    let s7: i32 = match t2 {
        0 => 1,
        1 => 2,
        _ => 0,
    };
    let cnt = |o: u32| (o.wrapping_sub(buf).wrapping_mul(0xaaaa_aaab) as i32) >> 5;
    loop {
        let pa = m.r32(sp + 0x20);
        let pb = m.r32(sp + 0x24);
        let f1 = if pa < end_a {
            m.rf(pa.wrapping_add(ka << 2) + 0x40)
        } else if pb < end_b {
            m.rf(0x002a_4ff0)
        } else {
            break;
        };
        let f0 = if pb < end_b { m.rf(pb.wrapping_add(kb << 2) + 0x40) } else { m.rf(0x002a_4ff0) };
        let (t0v, slot) = if f1 <= f0 {
            let t = ka;
            ka = 1u32.wrapping_sub(t);
            (t, sp + 0x20)
        } else {
            let t = kb;
            kb = 1u32.wrapping_sub(t);
            (t, sp + 0x24)
        };
        let a3v = !((ka.wrapping_add(kb) as i32) < s7) as u32;
        let src = m.r32(slot);
        if a3v != inside {
            if a3v != 0 {
                let f = m.rf(src + 0x40);
                m.wf(out + 0x40, f);
                let q = m.rv(src);
                m.wv(out, q);
                let q = m.rv(src + 0x20);
                m.wv(out + 0x20, q);
                let w = m.r32(src + 0x4c);
                m.w32(out + 0x4c, w);
                inside = a3v;
            } else {
                let f = m.rf(src + 0x44);
                m.wf(out + 0x44, f);
                let q = m.rv(src + 0x10);
                m.wv(out + 0x10, q);
                let q = m.rv(src + 0x30);
                m.wv(out + 0x30, q);
                let w = m.r32(src + 0x50);
                m.w32(out + 0x50, w);
                out = out.wrapping_add(0x60);
                if (t0 as i32) < cnt(out) {
                    out = out.wrapping_sub(0x60);
                    break;
                }
                inside = a3v;
            }
        }
        if t0v == 1 {
            m.w32(slot, src + 0x60);
        }
    }
    let n = cnt(out);
    m.call(0x001a_e5a0, &[t1, buf, (n as u32).wrapping_mul(0x60)], &[]);
    m.call(0x001a_e4a0, &[sp, 2], &[]);
    stack_pop(m, 0x30);
    n as u32
}

/// float approx-equal of the x, y, z lanes through FUN_0011d4d0(a, b, 1e-4) (short-circuit x, y, z).
fn eq3<M: QMem + ?Sized>(m: &mut M, a: V4, b: V4) -> bool {
    let eps = f32::from_bits(0x38d1_b717);
    for k in 0..3 {
        if m.call(0x0011_d4d0, &[], &[f4(a[k]), f4(b[k]), f4(eps)]).0 == 0 {
            return false;
        }
    }
    true
}

/// FUN_0012b738(o, x): sample keyframe i = FUN_0012b6d8(o, *x+0x88) of the 0x30-byte table o+8 into x
/// (+0x80/+0x84, +0xc0/+0xd0, +0xe0/+0xf0); x+0x9a = 2 when both ends equal and both tangents are the zero
/// vector 0x2a2dd0, else 1.
pub fn ee_sub_12b738<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let x = a1;
    let p = m.r32(x + 0x88);
    let f12 = m.rf(p);
    let i = m.call(0x0012_b6d8, &[a0], &[f4(f12)]).0;
    let d = i.wrapping_mul(0x30);
    let e = d.wrapping_add(m.r32(a0 + 8));
    let f = m.rf(e.wrapping_sub(0x24));
    m.wf(x + 0x80, f);
    let e = d.wrapping_add(m.r32(a0 + 8));
    let (w0, w1, w2) = (m.r32(e.wrapping_sub(0x30)), m.r32(e.wrapping_sub(0x2c)), m.r32(e.wrapping_sub(0x28)));
    m.w32(x + 0xc0, w0);
    m.w32(x + 0xc4, w1);
    m.w32(x + 0xc8, w2);
    let e = d.wrapping_add(m.r32(a0 + 8));
    let q = m.rv(e.wrapping_sub(0x10));
    m.wv(x + 0xd0, q);
    let e = d.wrapping_add(m.r32(a0 + 8));
    let f = m.rf(e + 0xc);
    m.wf(x + 0x84, f);
    let e = d.wrapping_add(m.r32(a0 + 8));
    let (w0, w1, w2) = (m.r32(e), m.r32(e + 4), m.r32(e + 8));
    m.w32(x + 0xe0, w0);
    m.w32(x + 0xe4, w1);
    m.w32(x + 0xe8, w2);
    let e = d.wrapping_add(m.r32(a0 + 8));
    let s1 = m.rv(x + 0xc0);
    let s2 = m.rv(x + 0xe0);
    let q = m.rv(e + 0x10);
    m.wv(x + 0xf0, q);
    let mut s5 = false;
    if eq3(m, s1, s2) {
        let s1 = m.rv(x + 0xd0);
        let s0 = m.rv(0x002a_2dd0);
        if eq3(m, s0, s1) {
            let s1 = m.rv(x + 0xf0);
            let s0 = m.rv(0x002a_2dd0);
            s5 = eq3(m, s0, s1);
        }
    }
    m.w16(x + 0x9a, if s5 { 2 } else { 1 });
}

/// virtual call through a vtable at *obj (fn at +0x14, this-adjust s16 at +0x10).
fn vcall0<M: QMem + ?Sized>(m: &mut M, obj: u32, a: &[u32]) -> (u32, V4) {
    let vt = m.r32(obj);
    let adj = m.r16(vt + 0x10) as i16 as i32;
    let f = m.r32(vt + 0x14);
    let mut args = vec![obj.wrapping_add(adj as u32)];
    args.extend_from_slice(a);
    m.call(f, &args, &[])
}

/// FUN_00138660(o, x): evaluate the blend node o into the pose x: a direct child (o+0xc[idx]) delegates;
/// otherwise weighted sum of the child samples (FUN_00117120) into x+0xc0 / x+0xd0 (x+0x9a = 4).
pub fn ee_sub_138660<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let s2 = a1;
    let a = m.r32(a0 + 4);
    let v = m.r32(a + 0xd4);
    if (v as i32) >= 0 {
        let t = m.r32(a0 + 0xc);
        let c = m.r32((v << 2).wrapping_add(t));
        if c != 0 {
            vcall0(m, c, &[a1]);
            return;
        }
        let q = m.rv(a0 + 0x10);
        m.w16(s2 + 0x9a, 2);
        m.wv(s2 + 0xc0, q);
        m.wv(s2 + 0xe0, q);
        m.w32(s2 + 0xd4, 0);
        m.w32(s2 + 0xd8, 0);
        m.w32(s2 + 0xd0, 0);
        m.w32(s2 + 0xf4, 0);
        m.w32(s2 + 0xf8, 0);
        m.w32(s2 + 0x8c, s2 + 0x84);
        m.wf(s2 + 0x84, 1.0);
        m.w32(s2 + 0x88, s2 + 0x80);
        m.w32(s2 + 0xf0, 0);
        m.w32(s2 + 0x80, 0);
        return;
    }
    let sp = stack_push(m, 0x160);
    let p = m.r32(s2 + 0x88);
    let f12 = m.rf(p);
    let v1 = m.r32(a0 + 8);
    let f21;
    let mut s6: u32 = 0;
    if v1 == 0 {
        f21 = f12 / m.rf(a + 0x74);
    } else {
        let f13 = m.rf(a + 0x74);
        let (x, y) = (m.r32(v1 + 4), m.r32(v1 + 8));
        s6 = m.call(0x0013_8370, &[x, y, sp + 0x130], &[f4(f12), f4(f13)]).0;
        let a8 = m.r32(a0 + 8);
        let f1 = m.rf(sp + 0x130);
        let t = m.r32(a8 + 8);
        f21 = f1 / m.rf((s6 << 3).wrapping_add(t) + 4);
    }
    let mut acc0: V4 = [0.0; 4];
    let mut acc1: V4 = [0.0; 4];
    let mut v1 = m.r32(a0 + 4);
    if (m.r32(v1 + 0xc4) as i32) > 0 {
        let mut s4: u32 = 0;
        loop {
            let st = m.r32(v1 + 0xc0);
            let s3 = m.r32(v1 + 0xc8).wrapping_add(s4.wrapping_mul(st));
            if m.rf(s3) != 0.0 {
                let t = m.r32(a0 + 0xc);
                let s0 = m.r32((s4 << 2).wrapping_add(t));
                if s0 == 0 {
                    let q = m.rv(a0 + 0x10);
                    m.wf(sp + 0x10, 0.0);
                    m.wv(sp, q);
                    m.wf(sp + 0x18, 0.0);
                    m.wf(sp + 0x14, 0.0);
                } else {
                    let a8 = m.r32(a0 + 8);
                    let f20;
                    if a8 == 0 {
                        let c = m.r32(s3 + 4);
                        let f0 = m.rf(c + 0x74);
                        let o4 = m.r32(a0 + 4);
                        let f1 = m.rf(o4 + 0x74);
                        m.wf(sp + 0x134, f21 * f0);
                        f20 = f0 / f1;
                    } else {
                        let off = m.r32(a8);
                        let pp = s3.wrapping_add(off).wrapping_add(s6 << 3);
                        let c = m.r32(s3 + 4);
                        let f2 = m.rf(c + 0x74);
                        let f14 = m.rf(pp + 4);
                        let f1 = m.rf(pp);
                        let f0 = f1 + f21 * f14;
                        m.wf(sp + 0x134, f0);
                        if f2 < f0 {
                            m.wf(sp + 0x134, f0 - f2);
                        }
                        let t = m.r32(a8 + 8);
                        f20 = f14 / m.rf((s6 << 3).wrapping_add(t) + 4);
                    }
                    m.call(0x0023_33b4, &[sp + 0x20, 0, 0x100], &[]);
                    m.w32(sp + 0xa8, sp + 0x134);
                    vcall0(m, s0, &[sp + 0x20]);
                    let f0 = m.rf(sp + 0xa0);
                    let f12 = m.rf(sp + 0xa4) - f0;
                    let f13 = m.rf(sp + 0x134) - f0;
                    let q = [m.rv(sp + 0xe0), m.rv(sp + 0xf0), m.rv(sp + 0x100), m.rv(sp + 0x110)];
                    m.call(
                        0x0011_7120,
                        &[sp, sp + 0x10, 0],
                        &[q[0], q[1], q[2], q[3], f4(f12), f4(f13), f4(f20)],
                    );
                }
                let w = m.rf(s3);
                let (p0, p1) = (m.rv(sp), m.rv(sp + 0x10));
                for k in 0..4 {
                    acc0[k] = acc0[k] + p0[k] * w;
                    acc1[k] = acc1[k] + p1[k] * w;
                }
            }
            v1 = m.r32(a0 + 4);
            s4 = s4.wrapping_add(1);
            if !((s4 as i32) < m.r32(v1 + 0xc4) as i32) {
                break;
            }
        }
    }
    m.w16(s2 + 0x9a, 4);
    m.wv(s2 + 0xc0, acc0);
    m.wv(s2 + 0xd0, acc1);
    stack_pop(m, 0x160);
}

/// FUN_0021bd18(o): frame-count / duration sync (+0x150 / +0x158), alloc +0x15c; project every mesh point
/// on +0x170 to find [min, max], then build per-strip 0x20-byte vertex arrays (offset by (+0x168 - 1) * d,
/// normalized d) via FUN_0016ef00 / FUN_001ae2a0; FUN_0021bc08(o); flag |= 2 unless +0x154.
pub fn ee_sub_21bd18<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let o = a0;
    m.call(0x001f_36a8, &[o], &[]);
    if m.r32(o + 0x158) == 0 && m.rf(o + 0x150) == 0.0 {
        m.wf(o + 0x150, 0.5);
    }
    let f2 = m.rf(o + 0x150);
    if f2 != 0.0 {
        let n = (f2 * 60.0 + 0.5) as i32;
        m.w32(o + 0x158, (n as u32).wrapping_add(2));
    } else {
        let n = m.r32(o + 0x158).wrapping_sub(2) as i32;
        m.wf(o + 0x150, n as f32 * f32::from_bits(0x3c88_8889));
    }
    let sp = stack_push(m, 0x810);
    let n = m.r32(o + 0x158);
    m.w32(sp + 4, 0x10);
    m.w32(sp, n.wrapping_mul(0x50));
    let v = m.call(0x001a_e2a0, &[sp], &[]).0;
    m.w32(o + 0x15c, v);
    let mut f20 = m.rf(0x002a_9240);
    let mut f21 = m.rf(0x002a_9244);
    let dot = |a: V4, b: V4| (a[0] * b[0] + a[1] * b[1]) + a[2] * b[2];
    let mut s2 = m.r32(o + 0xdc);
    while s2 != 0 {
        if (m.r32(s2 + 4) as i32) > 0 {
            let mut i: i32 = 0;
            loop {
                let e = m.r32(s2 + 8).wrapping_add((i as u32) << 5);
                let (cnt, src) = (m.r32(e), m.r32(e + 0xc));
                m.call(0x001a_e5a0, &[sp + 0x10, src, cnt << 4], &[]);
                let mut k = m.r32(e) as i32;
                i += 1;
                if k > 0 {
                    let dir = m.rv(o + 0x170);
                    let mut p = sp + 0x10;
                    loop {
                        let d = dot(m.rv(p), dir);
                        if f21 < d {
                            f21 = d;
                        }
                        if d < f20 {
                            f20 = d;
                        }
                        k -= 1;
                        p += 0x10;
                        if k == 0 {
                            break;
                        }
                    }
                }
                if !(i < m.r32(s2 + 4) as i32) {
                    break;
                }
            }
        }
        s2 = m.r32(s2 + 0xc);
    }
    let mut s2 = m.r32(o + 0xdc);
    while s2 != 0 {
        let base = m.r32(o + 0xbc);
        let a1 = (m.r32(s2).wrapping_sub(base).wrapping_mul(0xcccc_cccd) as i32 >> 2) as u32;
        let a2 = m.r32(o + 0xc0).wrapping_add(a1.wrapping_mul(0x28));
        m.w32(a2 + 4, 0x002c_44f0);
        m.call(0x0016_ef00, &[o + 0x80, a1, a2], &[]);
        if (m.r32(s2 + 4) as i32) > 0 {
            let mut i: i32 = 0;
            loop {
                let e = m.r32(s2 + 8).wrapping_add((i as u32) << 5);
                let (cnt, src) = (m.r32(e), m.r32(e + 0xc));
                m.call(0x001a_e5a0, &[sp, src, cnt << 4], &[]);
                let c = m.r32(e);
                m.w32(sp + 0x800, c << 5);
                m.w32(sp + 0x804, 0x10);
                let mut out = m.call(0x001a_e2a0, &[sp + 0x800], &[]).0;
                i += 1;
                let n = m.r32(e) as i32;
                m.w32(e + 0x1c, out);
                if n > 0 {
                    let f3 = f21 - f20;
                    let mut j: i32 = 0;
                    let mut p = sp;
                    loop {
                        let v = m.rv(p);
                        let dir = m.rv(o + 0x170);
                        let d = dot(v, dir);
                        let f0 = (m.rf(o + 0x168) - 1.0) * d;
                        let t = (d - f20) / f3;
                        let mut r = v;
                        for k in 0..4 {
                            r[k] = v[k] + dir[k] * f0;
                        }
                        m.wv(out, r);
                        m.wf(out + 0x10, t);
                        let q = m.r32(e + 0x18);
                        if q != 0 {
                            let w = m.r32(((j as u32) << 2).wrapping_add(q));
                            m.w32(out + 0x14, w);
                        }
                        let q = m.r32(e + 0x14);
                        if q != 0 {
                            let w = m.r32(((j as u32) << 2).wrapping_add(q));
                            m.w32(out + 0x18, w);
                        }
                        j += 1;
                        p += 0x10;
                        out = out.wrapping_add(0x20);
                        if !(j < m.r32(e) as i32) {
                            break;
                        }
                    }
                }
                if !(i < m.r32(s2 + 4) as i32) {
                    break;
                }
            }
        }
        s2 = m.r32(s2 + 0xc);
    }
    stack_pop(m, 0x810);
    m.call(0x0021_bc08, &[o], &[]);
    if m.r32(o + 0x154) == 0 {
        flag40(m, o, |v| v | 2);
    }
}
