//! WP-Q11 part D: class packages C2d8c08, C2d91e0, C2d9300, C2d93c0, C2d9420, C2d9500, C2d9c70, C2daa10, C2db338.
//! 1:1 from ghidra_functions.body on `QMem` (conventions as q11_a).
#![allow(unused_variables, unused_mut, dead_code)]
use crate::engine::classes::q::q10_a::ref_dec;
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem};
use crate::engine::classes::q::q6::set_tagged;
use crate::engine::classes::q::q7util::{f, fr, CLOCK};

/// Q11 `jr ra` only overrides
pub const NOPS: &[u32] = &[0x1eda50];

/// FUN_001e1b68 (C2d8c08_v1): release, pool 0x3e1560
pub fn c2d8c08_v1<M: QMem + ?Sized>(m: &mut M, o: u32) {
    ref_dec(m, o, 0x3e_1560);
}
/// FUN_001e1bc8 (C2d8c08_v4)
pub fn c2d8c08_v4<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32, p3: u32) {
    m.call(0x0013_3590, &[o, p2, p3], &[]);
    if p2 == 10 && m.r32(p3) == m.r32(o + 0x58) {
        let c = m.r32(o + 0x30);
        if m.r32(p3 + 8) == m.r32(c + 0x18) {
            m.w32(o + 0x5c, 1);
        }
    }
}

// ---- C2d91e0 ----------------------------------------------------------------------------------------

/// FUN_001ee688 (v6)
pub fn c2d91e0_v6<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32) {
    m.call(0x001e_d8e0, &[o, p2], &[]);
    for i in 0..4 {
        let v = m.r32(p2 + 0xc + 4 * i);
        m.w32(o + 0x50 + 4 * i, v);
    }
}
/// FUN_001ee6d8 (v7)
pub fn c2d91e0_v7<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.call(0x0011_a1e8, &[o + 0x34], &[]).0 == 0 {
        let t = m.rf(CLOCK) * m.rf(o + 0x50);
        let u = fr(m.call(0x0011_cf60, &[], &[f(t)]));
        let sp = stack_push(m, 0x10);
        m.call(0x0011_d608, &[sp, sp | 4], &[f(u)]);
        let x = m.rf(sp) * m.rf(o + 0x54) + m.rf(o + 0x58);
        let y = m.rf(sp + 4) * m.rf(o + 0x54) + m.rf(o + 0x5c);
        stack_pop(m, 0x10);
        m.call(0x001e_dad8, &[o], &[f(x), f(y)]);
    }
}
/// FUN_001ee768 (v8): fmod(clock * +0x50, 2*pi) / (2*pi)
pub fn c2d91e0_v8<M: QMem + ?Sized>(m: &mut M, o: u32) -> f32 {
    let t = m.rf(CLOCK) * m.rf(o + 0x50);
    let r = fr(m.call(0x0011_d898, &[], &[f(t), f(f32::from_bits(0x40c9_0fdb))]));
    r * 0.15915494
}

// ---- C2d9300 ----------------------------------------------------------------------------------------

/// FUN_001edfe0 (v7)
pub fn c2d9300_v7<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.call(0x0011_a1e8, &[o + 0x34], &[]).0 == 0 && 1 < m.r32(o + 0x1c) as i32 {
        let f4;
        if 0.0 < m.rf(o + 0x58) {
            f4 = m.rf(o + 0x58) - m.rf(CLOCK + 4);
        } else {
            let n = m.r32(o + 0x1c) as i32;
            let i1 = m.call(0x0011_d3f8, &[1, (n - 1) as u32], &[]).0 as i32;
            let idx = (m.r16(o + 0x28) as i32).wrapping_add(i1) % n;
            m.call(0x001e_da68, &[o, idx as u32], &[]);
            let (a, b) = (m.rf(o + 0x50), m.rf(o + 0x54));
            f4 = fr(m.call(0x0011_d448, &[], &[f(a), f(b)]));
        }
        m.wf(o + 0x58, f4);
    }
}

// ---- C2d93c0 ----------------------------------------------------------------------------------------

/// FUN_001edb98 (v6)
pub fn c2d93c0_v6<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32) {
    m.call(0x001e_d8e0, &[o, p2], &[]);
    for i in 0..4 {
        let v = m.r32(p2 + 0xc + 4 * i);
        m.w32(o + 0x50 + 4 * i, v);
    }
    let h = m.r16(p2 + 0x1c) as f32;
    m.wf(o + 100, h);
    let (a, b) = (m.rf(o + 0x50), m.rf(o + 0x54));
    let f2 = fr(m.call(0x0011_d448, &[], &[f(a), f(b)]));
    let n = m.r32(o + 0x1c) as i32 as f32;
    m.wf(o + 0x60, n / f2);
    let (a, b) = (m.rf(o + 0x58), m.rf(o + 0x5c));
    let u = fr(m.call(0x0011_d448, &[], &[f(a), f(b)]));
    m.wf(o + 0x6c, u);
    m.wf(o + 0x68, u);
}
/// FUN_001edc40 (v7)
pub fn c2d93c0_v7<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.call(0x0011_a1e8, &[o + 0x34], &[]).0 == 0 && 1 < m.r32(o + 0x1c) as i32 {
        if 0.0 < m.rf(o + 0x6c) {
            let v = m.rf(o + 0x6c) - m.rf(CLOCK + 4);
            m.wf(o + 0x6c, v);
        } else {
            let mut f3 = m.rf(o + 100) + m.rf(o + 0x60) * m.rf(CLOCK + 4);
            m.wf(o + 100, f3);
            let n = m.r32(o + 0x1c) as i32 as f32;
            if n <= f3 {
                let (a, b) = (m.rf(o + 0x50), m.rf(o + 0x54));
                let f3 = fr(m.call(0x0011_d448, &[], &[f(a), f(b)]));
                m.wf(o + 0x60, n / f3);
                let (a, b) = (m.rf(o + 0x58), m.rf(o + 0x5c));
                let u = fr(m.call(0x0011_d448, &[], &[f(a), f(b)]));
                m.wf(o + 0x6c, u);
                m.wf(o + 0x68, u);
            }
            let a = m.rf(o + 100);
            let n = m.r32(o + 0x1c) as i32 as f32;
            f3 = fr(m.call(0x0011_d898, &[], &[f(a), f(n)]));
            m.wf(o + 100, f3);
            m.call(0x001e_da68, &[o, f3 as i32 as u32], &[]);
        }
    }
}
/// FUN_001edd30 (v8)
pub fn c2d93c0_v8<M: QMem + ?Sized>(m: &mut M, o: u32) -> f32 {
    let n = m.r32(o + 0x1c) as i32 as f32;
    use crate::engine::math::ps2f::{ps2_add, ps2_div};
    // VFIX: PS2 DIV.S semantics (x/0 = +-max) - differs from IEEE when +0x60 == 0
    ps2_div(ps2_div(m.rf(o + 100), m.rf(o + 0x60)), ps2_add(ps2_div(n, m.rf(o + 0x60)), m.rf(o + 0x68)))
}

// ---- C2d9420 / C2d9500 / C2d9c70 / C2daa10 / C2db338 -------------------------------------------------

/// FUN_001ed8e0 (C2d9420_v6)
pub fn c2d9420_v6<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32) {
    let (a, b) = (m.r32(p2), m.r32(p2 + 4));
    m.w32(o + 0x14, a);
    m.w32(o + 0x18, b);
    let h = m.r16(p2 + 8) as u32;
    m.w32(o + 0x20, h);
    let h = m.r16(p2 + 10) as u32;
    m.w32(o + 0x1c, h);
    m.call(0x0011_a050, &[o + 0x34, 0x10], &[]);
    if m.r32(o + 0x30) == 0 {
        m.call(0x0011_a098, &[0x003e_159c, o], &[]);
    } else {
        m.call(0x0011_a070, &[0x003e_159c, o], &[]);
    }
}
/// FUN_001ef850 (C2d9500_v14)
pub fn c2d9500_v14<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001f_36a8, &[o], &[]);
    m.call(0x001a_6ca8, &[o, 1], &[]);
    set_tagged(m, o + 0x40, 2);
    let v = m.r64(o + 0x148) & 0xffff_fff7_ffff_ffff;
    m.w32(o + 0x148, v as u32);
    m.w32(o + 0x14c, (v >> 32) as u32);
}
/// FUN_001fc0a0 (C2d9c70_v1): --(s16 +0x4c); at 0 -> FUN_00258348(o, 0), FUN_001fc110(o)
pub fn c2d9c70_v1<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let s = m.r16(o + 0x4c) as i16;
    m.w16(o + 0x4c, s.wrapping_sub(1) as u16);
    if s == 1 && o != 0 {
        m.call(0x0025_8348, &[o, 0], &[]);
        m.call(0x001f_c110, &[o], &[]);
    }
}
/// FUN_0020c290 (C2daa10_v4)
pub fn c2daa10_v4<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32, p3: u32) {
    if p2 == 9 && m.r32(p3 + 8) == 0x22 {
        m.call(0x0020_c2f8, &[o, 1], &[]);
    }
    m.call(0x0013_3590, &[o, p2, p3], &[]);
}
/// FUN_0021c820 (C2db338_v5): (o, a1, a2, p4)
pub fn c2db338_v5<M: QMem + ?Sized>(m: &mut M, o: u32, a1: u32, a2: u32, p4: u32) {
    m.call(0x0012_cf88, &[o, a1, a2, p4], &[]);
    let v = m.r32(p4 + 0xa8);
    m.w32(o + 0x1c, v);
}
/// FUN_0021c858 (C2db338_v0)
pub fn c2db338_v0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0012_ce98, &[o], &[]);
    let p = m.r32(o + 8);
    let v = m.r32(o + 0x1c);
    m.w32(p + 0xa8, v);
}

/// FUN_00176708 (C2d04a0): FUN_001eef70(); memset(buf,0,0x70); buf[0]=0; FUN_00176750(o, buf)
pub fn c2d04a0_fn<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef70, &[], &[]);
    let sp = stack_push(m, 0x70);
    for i in 0..28 {
        m.w32(sp + 4 * i, 0);
    }
    m.call(0x0017_6750, &[o, sp], &[]);
    stack_pop(m, 0x70);
}
