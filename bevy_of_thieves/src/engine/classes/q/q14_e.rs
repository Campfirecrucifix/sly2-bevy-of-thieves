//! WP-Q14 part E: file packages run_001168f8 and run_00147068 (+ the byte-identical pool static-init pairs of
//! the later runs). 1:1 from ghidra_functions(_v2).body on `QMem`.
#![allow(unused_variables, unused_mut, dead_code)]
use crate::engine::classes::q::q10_e::pool_static_init;
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem};
use crate::engine::classes::q::q6::vcall;
use crate::engine::game::q_00166008::{ee_sub_166bd8, ee_sub_166c70, ee_sub_166ce0, ee_sub_166da0, ee_sub_166e90};

macro_rules! pair {
    ($($init:ident, $wrap:ident);* $(;)?) => {$(
        pub fn $init<M: QMem + ?Sized>(m: &mut M, p1: u32, p2: u32) { pool_static_init(m, p1, p2) }
        pub fn $wrap<M: QMem + ?Sized>(m: &mut M) { pool_static_init(m, 1, 0xffff) }
    )*};
}
// FUN_00116ed8/117010, 00117650/117788, 00147068/001471a0, 00147cc8/00147e00, 0016f178/0016f128 ...
pair! {
    fun_00116ed8, fun_00117010; fun_00117650, fun_00117788; fun_00147068, fun_001471a0;
    fun_00147cc8, fun_00147e00; fun_0016f178, fun_0016f2b0; fun_00170850, fun_00170988;
}

/// FUN_001168f8
pub fn ee_sub_1168f8<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x0011_4ae0, &[1, 0xffff], &[]);
}
/// FUN_00116918(p, n): clear the object returned by virtual +0xc of the sub-object at p + short(*p+8)
pub fn ee_sub_116918<M: QMem + ?Sized>(m: &mut M, p: u32, n: u32) -> u32 {
    let r = vcall(m, p, 0, 0xc, &[], &[]).0;
    m.call(0x0023_33b4, &[r, 0, n], &[]);
    r
}
/// FUN_00116970(p, a, b, c)
pub fn ee_sub_116970<M: QMem + ?Sized>(m: &mut M, p: u32, a: u32, b: u32, c: u32) -> u32 {
    let r = vcall(m, p, 0, 0xc, &[a, c], &[]).0;
    m.call(0x001a_e5a0, &[r, b, a], &[]);
    r
}
/// FUN_001169d8(p, s, c): strlen(s) + 1 bytes
pub fn ee_sub_1169d8<M: QMem + ?Sized>(m: &mut M, p: u32, s: u32, c: u32) {
    let n = m.call(0x0021_ed28, &[s], &[]).0;
    ee_sub_116970(m, p, n.wrapping_add(1), s, c);
}
/// C2ca550_ctor (FUN_00116a98)
pub fn c2ca550_ctor<M: QMem + ?Sized>(m: &mut M, p: u32, a: u32, b: u32) -> u32 {
    m.w32(p + 4, a);
    m.w32(p + 8, b);
    m.w32(p, 0x002c_a550);
    m.w32(p + 0xc, 0);
    p
}
/// C2ca578_ctor_116b30 (FUN_00116b30)
pub fn c2ca578_ctor_116b30<M: QMem + ?Sized>(m: &mut M, p1: u32, p2: u32) {
    if p2 == 0xffff && p1 != 0 {
        m.w32(0x002d_d590, 0x002c_a578);
        m.w32(0x002d_d594, 0xffff_ffff);
        pool_static_init(m, p1, p2);
    }
}
/// thunk_C2ca578_ctor_116b30_116c80
pub fn thunk_c2ca578_ctor_116c80<M: QMem + ?Sized>(m: &mut M) {
    c2ca578_ctor_116b30(m, 1, 0xffff);
}
/// FUN_00116ca0 / 116cc0 / 116ce0 / 116d00: FUN_00233178(a, b, 4 / 8 / 0xc / 0x10)
pub fn ee_sub_116ca0<M: QMem + ?Sized>(m: &mut M, a: u32, b: u32) {
    m.call(0x0023_3178, &[a, b, 4], &[]);
}
pub fn ee_sub_116cc0<M: QMem + ?Sized>(m: &mut M, a: u32, b: u32) {
    m.call(0x0023_3178, &[a, b, 8], &[]);
}
pub fn ee_sub_116ce0<M: QMem + ?Sized>(m: &mut M, a: u32, b: u32) {
    m.call(0x0023_3178, &[a, b, 0xc], &[]);
}
pub fn ee_sub_116d00<M: QMem + ?Sized>(m: &mut M, a: u32, b: u32) {
    m.call(0x0023_3178, &[a, b, 0x10], &[]);
}
/// FUN_001177f0
pub fn ee_sub_1177f0<M: QMem + ?Sized>(m: &mut M, p: u32) {
    for o in [0x1c, 0x10, 4, 0x14, 8, 0x18] {
        m.w32(p + o, 0);
    }
}
/// FUN_001179b8
pub fn ee_sub_1179b8<M: QMem + ?Sized>(m: &mut M, a: u32, b: u32) {
    m.call(0x0011_7968, &[a, b, 0], &[]);
}

// ---------------------------------------------------------------- run_00147068
/// FUN_00147410(o, new): state change of the screen-fade object (+0x150 state, +0x154 time)
pub fn ee_sub_147410<M: QMem + ?Sized>(m: &mut M, o: u32, new: i32) {
    let cur = m.r32(o + 0x150) as i32;
    if new == cur {
        return;
    }
    if cur == -1 {
        m.w32(0x002d_d644, o);
        m.call(0x001f_e5b8, &[0x003d_d040, 2, 0], &[]);
        m.call(0x0014_d1d0, &[0x002d_ded0, 8], &[super::q5util::f4(0.0)]);
        m.call(0x0020_efc0, &[0x003d_d490, 0], &[]);
        vcall(m, o, 0xc, 0x44, &[], &[]);
        if m.call(0x001a_61a0, &[o], &[]).0 == 0 {
            let c = m.r32(o + 0x14);
            m.call(0x0011_a220, &[0x003d_d4bc, c], &[]);
            let q = m.r32(c);
            vcall(m, q, 0xc, 0xbc, &[], &[]);
        }
    } else if cur == 3 && m.r32(o + 0x160) != 0 {
        { let c = m.r32(o + 0x160); m.call(0x0013_1a20, &[c], &[super::q5util::f4(1.0)]); }
    }
    m.w32(o + 0x150, new as u32);
    let t = m.r32(0x002d_df0c);
    m.w32(o + 0x154, t);
    if new == -1 {
        m.w32(0x002d_d644, 0);
        m.call(0x0014_d1d0, &[0x002d_ded0, 8], &[super::q5util::f4(1.0)]);
        m.call(0x001f_e5b8, &[0x003d_d040, 2, 1], &[]);
        m.call(0x0020_f030, &[0x003d_d490, 0], &[]);
        vcall(m, o, 0xc, 0x4c, &[], &[]);
    } else if new == 3 && m.r32(o + 0x160) != 0 {
        { let c = m.r32(o + 0x160); m.call(0x0013_1a20, &[c], &[super::q5util::f4(0.0)]); }
    }
}
/// FUN_001475c0: two GS sprite setups on the 0x2dea00 packet (frame-buffer copy rectangles)
pub fn ee_sub_1475c0<M: QMem + ?Sized>(m: &mut M) {
    let p = 0x002d_ea00;
    ee_sub_166bd8(m, p, 0, 1, 0xe);
    let v = (m.r32(0x002d_ece0) & 0x1ff) as u64 | (m.r32(0x002d_ecd0) & 0x3f0000) as u64;
    ee_sub_166e90(m, p, 0x4d, v);
    ee_sub_166e90(m, p, 0x48, 0x31001);
    m.call(0x0015_76f0, &[p], &[]);
    for (tag, x0, x1) in [(0x002d_fb00u32, 0x7000u32, 0x8000u32), (0x002d_fb20, 0x8000, 0x9000)] {
        if tag == 0x002d_fb20 {
            // the second block starts without the 0x4d / 0x48 register writes
        }
        m.call(0x0015_7500, &[p, 2, tag], &[]);
        m.call(0x0015_74b8, &[p], &[]);
        ee_sub_166bd8(m, p, 0x316, 7, 0x0535_31ee);
        ee_sub_166e90(m, p, 0x15, 0x61);
        ee_sub_166e90(m, p, 0x3f, 0);
        ee_sub_166c70(m, p, 0x8080_8080);
        ee_sub_166ce0(m, p, 8, 8);
        ee_sub_166da0(m, p, x0, 0x7900, 0);
        ee_sub_166ce0(m, p, 0x2008, 0xe08);
        ee_sub_166da0(m, p, x1, 0x8700, 0);
        if tag == 0x002d_fb00 {
            m.call(0x0015_76f0, &[p], &[]);
        }
    }
}
/// FUN_001477d8(o): per-frame draw of the fade quads. PARTIAL: the 2 x 4 corner draw loop (FUN_0011c1a8 matrix
/// transforms + FUN_0011d9d8 clamp + 0x2bd3f0 offset table, VU0 operands lost in the decompile) is not ported;
/// everything up to and including the matrix copy and the per-quad packet tag is.
pub fn ee_sub_1477d8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let st = m.r32(o + 0x150) as i32;
    if st == 1 {
        ee_sub_1475c0(m);
        ee_sub_147410(m, o, 2);
    } else if st < 1 || 3 < st {
        return;
    }
    let p = 0x002d_ea00;
    let c = m.r32(0x002d_ea4c);
    let f = m.r32(c + 0x30) & 0xffff_fffe;
    m.w32(c + 0x30, f);
    if m.r32(c + 0x24) == 0 {
        let t = m.r32(0x002d_ea34);
        m.w32(c + 0x24, t);
    }
    m.call(0x0015_76f0, &[p], &[]);
    m.call(0x0015_7500, &[p, 0x25, 0x002d_f4b0], &[]);
    m.call(0x0015_74b8, &[p], &[]);
    ee_sub_166bd8(m, p, 0, 1, 0xe);
    ee_sub_166e90(m, p, 0x47, 0x31001);
    let v = ((m.r32(0x002d_ece0) & 0x1ff) as u64) << 5 | (((m.r32(0x002d_ecd0) >> 16) & 0x3f) as u64) << 0xe | 0x6_a800_0000;
    ee_sub_166e90(m, p, 6, v);
    ee_sub_166e90(m, p, 0x14, 0x61);
    ee_sub_166e90(m, p, 0x3f, 0);
    ee_sub_166e90(m, p, 8, 0x0380_0080_000a);
    let sp = stack_push(m, 0x100);
    for i in 0..2u32 {
        let src = m.r32(o + 0x158 + 4 * i);
        let w = m.r32(o + 0x60);
        m.call(0x0022_ff40, &[sp + 0x80, 0, w, src + 0x40], &[]);
        for k in 0..16u32 {
            let v = m.r32(sp + 0x80 + 4 * k);
            m.w32(sp + 0x40 * i + 4 * k, v);
        }
    }
    let mut i = 0u32;
    loop {
        let (draw, next) = match i {
            1 => (m.r32(o + 0x168) != 0, 2),
            0 => (m.r32(o + 0x164) != 0 && m.r32(o + 0x16c) != 0, 1),
            2 => (m.r32(o + 0x164) != 0 && m.r32(o + 0x16c) == 0, 3),
            _ => (false, i + 1),
        };
        if draw {
            ee_sub_166bd8(m, p, 0x14, 3, 0x512);
            // corner loop not ported (see doc)
        }
        i = next;
        if 2 < i as i32 {
            break;
        }
    }
    stack_pop(m, 0x100);
}
/// FUN_00147c20(o, a, t)
pub fn ee_sub_147c20<M: QMem + ?Sized>(m: &mut M, o: u32, a: u32, t: u32) {
    m.w32(o + 0x170, t);
    m.w32(o + 0x164, 1);
    m.w32(o + 0x16c, 1);
    m.w32(o + 0x168, 1);
    let l = m.call(0x0015_e938, &[0x003d_d490, 1, a, o], &[]).0;
    if l != 0 {
        let f = m.r32(l + 0x40);
        if (f ^ 1) & 1 == 0 {
            m.w32(l + 0x40, f | 0x100);
        } else {
            let v = m.r32(f) | 0x100;
            m.w32(f, v);
        }
        m.call(0x0013_05b8, &[l, o, 4, o + 0x160], &[super::q5util::f4(0.0), super::q5util::f4(1.0)]);
        ee_sub_147410(m, o, 0);
    }
}
/// FUN_00147e20(a, b): unlink b from the list at a + 0x20 (head cache at +0x39a4). BOSS R21: FUN_0011a380 gets
/// b too (a1 is passed through unchanged; the decompile drops it): without it FUN_00147e88 re-inserted a
/// listed node and the proximity list looped on itself (barrel FUN_00183b80 -> FUN_00147f40)
pub fn ee_sub_147e20<M: QMem + ?Sized>(m: &mut M, a: u32, b: u32) {
    if m.call(0x0011_a380, &[a + 0x20, b], &[]).0 != 0 {
        if m.r32(a + 0x39a4) == b {
            let n = m.r32(b + 4);
            m.w32(a + 0x39a4, n);
        }
        m.call(0x0011_a2e8, &[a + 0x20, b], &[]);
    }
}
/// FUN_00147e88
pub fn ee_sub_147e88<M: QMem + ?Sized>(m: &mut M, a: u32, b: u32) {
    ee_sub_147e20(m, a, b);
    let h = m.r32(a + 0x39a4);
    m.call(0x0011_a280, &[a + 0x20, h, b], &[]);
}
