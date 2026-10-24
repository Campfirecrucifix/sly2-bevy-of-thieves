//! WP-Q10: class packages C2d8668, C2d9240, C2d92a0, C2d9360, C2d9768 (+ the shared refcount pair is in q10_a).
//! Float expressions follow the disassembly operation order (single precision, R5900 truncating cvt.w.s).
use crate::engine::classes::q::q10_a::ref_dec;
use crate::engine::classes::q::q2mem::{vslot, QMem};
use crate::engine::classes::q::q7util::{f, CLOCK};

/// 1/4096 (0x39800000)
const K: f32 = f32::from_bits(0x3980_0000);
const DT: u32 = CLOCK + 4;

/// FUN_001f0538 (C2d9768_v1, pool 0x3e1278)
pub fn c2d9768_v1<M: QMem + ?Sized>(m: &mut M, o: u32) {
    ref_dec(m, o, 0x3e_1278)
}

// ---- C2d8668 --------------------------------------------------------------------------------------

/// FUN_001d95b8(list): list[0] is the wanted entry, list[3] the current one; swap them, notifying both through
/// virtual slot 8 (0x4c): old -> (0, new) with +0x10 cleared, new -> (1, old) with +0x10 = &DAT_002dfc00
pub fn ee_sub_1d95b8<M: QMem + ?Sized>(m: &mut M, list: u32) {
    let cur = m.r32(list);
    let old = m.r32(list + 12);
    if cur == old {
        return;
    }
    if old != 0 {
        m.w32(old + 0x10, 0);
        vslot(m, old, 8, &[0, cur], &[]);
    }
    let old2 = m.r32(list + 12);
    m.w32(list + 12, cur);
    if cur != 0 {
        m.w32(cur + 0x10, 0x002d_fc00);
        vslot(m, cur, 8, &[1, old2], &[]);
    }
}
/// FUN_001d98b0 (v9)
pub fn c2d8668_v9<M: QMem + ?Sized>(m: &mut M, o: u32) {
    const L: u32 = 0x003d_9e70;
    if m.call(0x0011_a380, &[L, o], &[]).0 != 0 {
        m.call(0x0011_a2e8, &[L, o], &[]);
    }
    m.call(0x0011_a250, &[L, o], &[]);
    ee_sub_1d95b8(m, L);
}
/// FUN_001d9910 (v10)
pub fn c2d8668_v10<M: QMem + ?Sized>(m: &mut M, o: u32) {
    const L: u32 = 0x003d_9e70;
    if m.call(0x0011_a380, &[L, o], &[]).0 != 0 {
        m.call(0x0011_a2e8, &[L, o], &[]);
        ee_sub_1d95b8(m, L);
    }
}

// ---- C2d9240 --------------------------------------------------------------------------------------

/// FUN_001ee480 (v6): copy of the init record p (+0xc..+0x18) to +0x50..+0x5c, +0x60 = +0x64 = 1.0
pub fn c2d9240_v6<M: QMem + ?Sized>(m: &mut M, o: u32, p: u32) {
    m.call(0x001e_d8e0, &[o, p], &[]);
    for i in 0..4 {
        let v = m.r32(p + 0xc + 4 * i);
        m.w32(o + 0x50 + 4 * i, v);
    }
    m.w32(o + 0x60, 0x3f80_0000);
    m.w32(o + 0x64, 0x3f80_0000);
}
/// FUN_001ee4e0 (v7)
pub fn c2d9240_v7<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.call(0x0011_a1e8, &[o + 0x34], &[]).0 != 0 {
        return;
    }
    let dt = m.rf(DT);
    let sx = m.r16(o + 0x24) as i16 as i32 as f32;
    let sy = m.r16(o + 0x26) as i16 as i32 as f32;
    let (a, b) = (m.rf(o + 0x50), m.rf(o + 0x60));
    let c = m.rf(o + 0x58);
    let x = m.call(0x0024_5a18, &[], &[f(sx * K + a * b * dt), f(c)]).1[0];
    let (a, b) = (m.rf(o + 0x54), m.rf(o + 0x64));
    let c = m.rf(o + 0x5c);
    let y = m.call(0x0024_5a18, &[], &[f(sy * K + a * b * dt), f(c)]).1[0];
    m.call(0x001e_dad8, &[o], &[f(x), f(y)]);
}
/// FUN_001ee5b8 (v8): float result
pub fn c2d9240_v8<M: QMem + ?Sized>(m: &mut M, o: u32) -> f32 {
    let a = if m.rf(o + 0x50) == 0.0 {
        0.5
    } else {
        ps2_div(m.r16(o + 0x24) as i16 as i32 as f32 * K, m.rf(o + 0x58)) * 0.5
    };
    let b = if m.rf(o + 0x54) == 0.0 {
        0.5
    } else {
        ps2_div(m.r16(o + 0x26) as i16 as i32 as f32 * K, m.rf(o + 0x5c)) * 0.5
    };
    ps2_clamp(a + b)
}
use crate::engine::math::ps2f::{ps2_clamp, ps2_div};

// ---- C2d92a0 --------------------------------------------------------------------------------------

/// FUN_001ee090 (v6)
pub fn c2d92a0_v6<M: QMem + ?Sized>(m: &mut M, o: u32, p: u32) {
    m.call(0x001e_d8e0, &[o, p], &[]);
    let v = m.r32(p + 0x1c);
    m.w32(o + 0x60, v);
    for i in 0..4 {
        let v = m.r32(p + 0xc + 4 * i);
        m.w32(o + 0x50 + 4 * i, v);
    }
    m.w32(o + 0x64, u32::MAX);
    m.call(0x001e_e100, &[o, 0], &[]);
}
/// FUN_001ee228 (v7): blink state machine on +0x64 (0 wait, 1 advance frame +0x74 at rate +0x70, 2 hold,
/// 3 retract), frame count +0x1c
pub fn c2d92a0_v7<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.call(0x0011_a1e8, &[o + 0x34], &[]).0 != 0 {
        return;
    }
    let n = m.r32(o + 0x1c) as i32;
    if n < 2 {
        return;
    }
    let mut st = m.r32(o + 0x64) as i32;
    let dt = m.rf(DT);
    match st {
        1 => {
            let d = m.rf(o + 0x70) * dt;
            let v = m.rf(o + 0x74) + d;
            m.wf(o + 0x74, v);
            if (n as f32) <= v {
                let v = v - d;
                m.wf(o + 0x74, v);
                st = if v as i32 == n - 1 { 2 } else { 3 };
            }
        }
        0 => {
            if m.rf(o + 0x6c) <= m.rf(CLOCK) - m.rf(o + 0x68) {
                st = 1;
            }
        }
        2 => {
            if m.rf(o + 0x78) < 1.0 {
                st = 3;
            }
        }
        3 => {
            let v = m.rf(o + 0x74) - m.rf(o + 0x70) * dt;
            m.wf(o + 0x74, v);
            if v <= n as f32 * m.rf(o + 0x78) {
                st = 0;
            }
        }
        _ => {}
    }
    m.call(0x001e_e100, &[o, st as u32], &[]);
    let fr = m.rf(o + 0x74) as i32;
    m.call(0x001e_da68, &[o, fr as u32], &[]);
}
/// FUN_001ee420 (v9)
pub fn c2d92a0_v9<M: QMem + ?Sized>(m: &mut M, o: u32, a1: u32) -> u32 {
    if m.call(0x001e_db80, &[o, a1], &[]).0 != 0 || m.r32(a1) == m.r32(o + 0x60) {
        1
    } else {
        0
    }
}

// ---- C2d9360 --------------------------------------------------------------------------------------

fn d9360_reset<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let (a, b) = (m.rf(o + 0x50), m.rf(o + 0x54));
    let d = m.call(0x0011_d448, &[], &[f(a), f(b)]).1[0];
    let n = (m.r32(o + 0x1c) as i32).wrapping_shl(1);
    m.wf(o + 0x60, crate::engine::math::ps2f::ps2_div(n as f32, d)); // PS2 div.s: x/0 = +-FLT_MAX
    let (a, b) = (m.rf(o + 0x58), m.rf(o + 0x5c));
    let u = m.call(0x0011_d448, &[], &[f(a), f(b)]).1[0];
    m.wf(o + 0x6c, u);
    m.wf(o + 0x68, u);
}
/// FUN_001edd60 (v6)
pub fn c2d9360_v6<M: QMem + ?Sized>(m: &mut M, o: u32, p: u32) {
    m.call(0x001e_d8e0, &[o, p], &[]);
    for i in 0..4 {
        let v = m.r32(p + 0xc + 4 * i);
        m.w32(o + 0x50 + 4 * i, v);
    }
    let h = m.r16(p + 0x1c) as f32;
    m.wf(o + 0x64, h);
    d9360_reset(m, o);
}
/// FUN_001ede10 (v7)
pub fn c2d9360_v7<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.call(0x0011_a1e8, &[o + 0x34], &[]).0 != 0 {
        return;
    }
    let n = m.r32(o + 0x1c) as i32;
    if n < 2 {
        return;
    }
    let dt = m.rf(DT);
    let w = m.rf(o + 0x6c);
    if 0.0 < w {
        m.wf(o + 0x6c, w - dt);
        return;
    }
    let rate = m.rf(o + 0x60);
    let d = rate * dt;
    let v = m.rf(o + 0x64) + d;
    m.wf(o + 0x64, v);
    if n as f32 <= v {
        m.wf(o + 0x64, v - d);
        m.wf(o + 0x60, -rate);
    }
    if m.rf(o + 0x64) < 0.0 {
        m.wf(o + 0x64, 0.0);
        d9360_reset(m, o);
    }
    let fr = m.rf(o + 0x64) as i32;
    m.call(0x001e_da68, &[o, fr as u32], &[]);
}
/// FUN_001edf28 (v8): float result
pub fn c2d9360_v8<M: QMem + ?Sized>(m: &mut M, o: u32) -> f32 {
    let mut rate = m.rf(o + 0x60);
    let n = (m.r32(o + 0x1c) as i32).wrapping_shl(1) as f32;
    let num = if rate < 0.0 {
        rate = -rate;
        ps2_clamp(n - m.rf(o + 0x64))
    } else {
        m.rf(o + 0x64)
    };
    // VFIX: PS2 DIV.S semantics (x/0 = +-max) when +0x60 == 0
    ps2_div(ps2_div(num, rate), ps2_clamp(ps2_div(n, rate) + m.rf(o + 0x68)))
}
