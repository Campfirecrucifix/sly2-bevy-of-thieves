//! WP-Q10: class packages C2ce2b8, C2cf7f8, C2cfe20, C2cfea0, C2cffa0, C2d0520, C2d0620.
//! Argument lists were checked against the disassembly where the decompile drops them (e.g. the
//! second argument of FUN_00209890 / FUN_00209970 in C2cfe20, quadword a3 of FUN_0020ac28).
use crate::engine::classes::q::q10_a::ref_dec;
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem};
use crate::engine::classes::q::q7util::{f, CLOCK};

const NEG1: u32 = u32::MAX;

/// FUN_001566c8 (C2ce2b8_v1, pool 0x3e1498)
pub fn c2ce2b8_v1<M: QMem + ?Sized>(m: &mut M, o: u32) {
    ref_dec(m, o, 0x3e_1498)
}

// ---- C2cf7f8 --------------------------------------------------------------------------------------

/// FUN_00171228 (v6)
pub fn c2cf7f8_v6<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef68, &[o], &[]);
    let p = m.r32(o + 0x10);
    m.call(0x0020_9890, &[p, 0x21], &[]);
}
/// FUN_00171398 (v8)
pub fn c2cf7f8_v8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef78, &[o], &[]);
    m.call(0x001e_f038, &[o], &[]);
    m.call(0x0017_1258, &[o, 0, 0, 0], &[]);
    let p = m.r32(o + 0x10);
    m.call(0x0020_9970, &[p, 0x21], &[]);
}

// ---- C2cfe20 --------------------------------------------------------------------------------------

/// FUN_0017d360 (v6)
pub fn c2cfe20_v6<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef68, &[o], &[]);
    let c = m.r32(0x002a_5604);
    m.w32(o + 0x38, NEG1);
    m.w32(o + 0xa0, c);
    m.w32(o + 0x30, NEG1);
    let g = m.r32(0x003e_1a5c);
    let v = m.r32(g + 0x28);
    m.w32(o + 0xa4, v);
}
/// FUN_0017d3b0 (v7)
pub fn c2cfe20_v7<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let a1 = m.r32(o + 0x38);
    if a1 != NEG1 {
        let p = m.r32(o + 0x10);
        m.call(0x0020_9890, &[p, a1], &[]);
    }
    m.call(0x001e_ef70, &[o], &[]);
    m.call(0x0017_d450, &[o, 4], &[]);
    m.call(0x0017_da28, &[o], &[]);
}
/// FUN_0017d408 (v8)
pub fn c2cfe20_v8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef78, &[o], &[]);
    m.call(0x0017_d450, &[o, NEG1], &[]);
    let a1 = m.r32(o + 0x38);
    if a1 != NEG1 {
        let p = m.r32(o + 0x10);
        m.call(0x0020_9970, &[p, a1], &[]);
    }
}

// ---- C2cfea0 --------------------------------------------------------------------------------------

/// FUN_0017d2b0 (v6)
pub fn c2cfea0_v6<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef68, &[o], &[]);
    let p = m.r32(o + 0x10);
    m.call(0x0020_9890, &[p, 0x29], &[]);
}
/// FUN_0017d2e0 (v7)
pub fn c2cfea0_v7<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef70, &[o], &[]);
    let p = m.r32(o + 0x10);
    let n = m.call(0x0020_5438, &[p, 0x9d], &[]).0;
    m.w32(n + 0x3c, 2);
    vslot(m, n, 7, &[], &[]);
}
/// FUN_0017d330 (v8)
pub fn c2cfea0_v8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef78, &[o], &[]);
    let p = m.r32(o + 0x10);
    m.call(0x0020_9970, &[p, 0x29], &[]);
}

// ---- C2cffa0 --------------------------------------------------------------------------------------

/// FUN_0017ce68 (v6)
pub fn c2cffa0_v6<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef68, &[o], &[]);
    m.w32(o + 0x30, NEG1);
    let p = m.r32(o + 0x10);
    m.call(0x0020_9890, &[p, 0x27], &[]);
}
/// FUN_0017cea0 (v7)
pub fn c2cffa0_v7<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef70, &[o], &[]);
    m.call(0x0017_d058, &[o, 0], &[]);
}
/// FUN_0017ced0 (v8)
pub fn c2cffa0_v8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef78, &[o], &[]);
    m.call(0x0017_d058, &[o, NEG1], &[]);
    let p = m.r32(o + 0x10);
    m.call(0x0020_9970, &[p, 0x27], &[]);
}
/// FUN_0017cf10 (v9): in state 0 either FUN_0017d188 -> state 1, or steer the owner's heading value
/// (+0xf88) towards +0x38 (direction flag +0xf8c, rate +0x198, smoothing FUN_0011d018 with {|rate|, 0, 0.1});
/// loop while the state changes; finally FUN_001c3500(heading, +0xf84, dir, owner + 0x1b0)
pub fn c2cffa0_v9<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef98, &[o], &[]);
    let mut s1 = m.r32(o + 0x30);
    loop {
        if s1 == 0 {
            if m.call(0x0017_d188, &[o], &[]).0 != 0 {
                m.call(0x0017_d058, &[o, 1], &[]);
            } else {
                let p = m.r32(o + 0x10);
                let dir = (m.rf(p + 0xf88) < m.rf(o + 0x38)) as u32;
                m.w32(p + 0xf8c, dir);
                let rate = m.rf(p + 0x198);
                let val = if m.r32(p + 0xf8c) == 0 { rate } else { -rate };
                m.wf(0x002d_fb7c, val);
                let dt = m.rf(CLOCK + 4);
                let sp = stack_push(m, 0x10);
                m.wf(sp, val.abs());
                m.wf(sp + 4, 0.0);
                m.w32(sp + 8, 0x3dcc_cccd); // 0.1
                let cur = m.rf(o + 0x3c);
                let r = m.call(0x0011_d018, &[sp, 0], &[f(cur), f(val), f(dt)]).1[0];
                stack_pop(m, 0x10);
                m.wf(o + 0x3c, r);
                let p = m.r32(o + 0x10);
                let h = m.rf(p + 0xf88) + r * dt;
                let a0 = m.r32(p + 0xf84);
                let nh = m.call(0x001e_66c8, &[a0], &[f(h)]).1[0];
                m.wf(p + 0xf88, nh);
            }
        }
        if m.r32(o + 0x30) == s1 {
            break;
        }
        s1 = m.r32(o + 0x30);
    }
    let p = m.r32(o + 0x10);
    let (h, a0, a1) = (m.rf(p + 0xf88), m.r32(p + 0xf84), m.r32(p + 0xf8c));
    m.call(0x001c_3500, &[a0, a1, p + 0x1b0], &[f(h)]);
}

// ---- C2d0520 --------------------------------------------------------------------------------------

/// FUN_00179a58 (v6)
pub fn c2d0520_v6<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef68, &[o], &[]);
    let c = m.r32(0x002a_558c);
    m.w32(o + 0x38, NEG1);
    m.w32(o + 0x30, c);
    m.w32(o + 0x34, c);
    let p = m.r32(o + 0x10);
    m.call(0x0020_9890, &[p, 0x1e], &[]);
}
/// FUN_00179b60 (v8)
pub fn c2d0520_v8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0017_9d30, &[o, NEG1], &[]);
    m.call(0x001e_ef78, &[o], &[]);
    let p = m.r32(o + 0x10);
    m.call(0x0020_9970, &[p, 0x1e], &[]);
}

// ---- C2d0620 --------------------------------------------------------------------------------------

/// FUN_001753d8 (v6)
pub fn c2d0620_v6<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef68, &[o], &[]);
    let p = m.r32(o + 0x10);
    m.call(0x0020_9890, &[p, 0x18], &[]);
    m.w32(o + 0x34, NEG1);
}
/// FUN_00175410 (v7)
pub fn c2d0620_v7<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef70, &[o], &[]);
    c2d0620_set_state(m, o, 0);
    m.call(0x001e_efc0, &[o], &[]);
}
/// FUN_00175448 (v8)
pub fn c2d0620_v8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef78, &[o], &[]);
    c2d0620_set_state(m, o, NEG1);
    let p = m.r32(o + 0x10);
    m.call(0x0020_9970, &[p, 0x18], &[]);
}
/// FUN_00175488: state change (+0x34): 0 spawns class 0x9f child, 1 class 0x9d child (+0x3c = 4), -1 FUN_001ef038
pub fn c2d0620_set_state<M: QMem + ?Sized>(m: &mut M, o: u32, st: u32) {
    let cur = m.r32(o + 0x34);
    if st == cur {
        return;
    }
    let p10 = m.r32(o + 0x10);
    if cur == 0 {
        let p30 = m.r32(o + 0x30);
        m.call(0x0013_3448, &[p30, p10], &[]);
        m.w32(o + 0x30, 0);
        let q = m.rv(0x002a_2dd0);
        m.call(0x0020_ac28, &[p10, 0, 0], &[q]);
    }
    m.call(0x001e_f0d8, &[o + 0x18], &[]);
    m.w32(o + 0x34, st);
    let clk = m.r32(CLOCK);
    m.w32(o + 0x38, clk);
    if st == 0 {
        let p30 = m.r32(o + 0x30);
        m.call(0x0013_33c0, &[p30, p10], &[]);
        let n = m.call(0x0020_5438, &[p10, 0x9f], &[]).0;
        let p30 = m.r32(o + 0x30);
        m.w32(n + 0x38, p30);
        let p3c = m.r32(o + 0x3c);
        let v = m.r32(p3c + 0xc0);
        m.w32(n + 0x40, v);
        if m.r32(p3c + 0xa0) != 0 {
            let sp = stack_push(m, 0x40);
            let r = vslot(m, p30, 0x1f, &[sp], &[]).0;
            let q = m.rv(sp + 0x30);
            m.call(0x0020_ac28, &[p10, p3c, r], &[q]);
            stack_pop(m, 0x40);
        }
        vslot(m, n, 7, &[], &[]);
    } else if (st as i32) < 1 {
        if st == NEG1 {
            m.call(0x001e_f038, &[o], &[]);
        }
    } else if st == 1 {
        let n = m.call(0x0020_5438, &[p10, 0x9d], &[]).0;
        m.w32(n + 0x3c, 4);
        let p30 = m.r32(o + 0x30);
        m.w32(n + 0x38, p30);
        vslot(m, n, 7, &[], &[]);
    }
}
/// FUN_00175600 (v9)
pub fn c2d0620_v9<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef98, &[o], &[]);
    loop {
        if m.call(0x001e_f0a8, &[o], &[]).0 != 0 {
            return;
        }
        let s = m.r32(o + 0x34);
        let p18 = m.r32(o + 0x18);
        if s == 0 {
            if m.r32(o + 0x40) == 0 {
                if m.call(0x0017_4fb8, &[p18], &[]).0 != 0 {
                    c2d0620_set_state(m, o, NEG1);
                }
            } else {
                c2d0620_set_state(m, o, 1);
            }
        } else if s == 1 && m.call(0x0017_4fb8, &[p18], &[]).0 != 0 {
            c2d0620_set_state(m, o, NEG1);
        }
        if m.r32(o + 0x34) == s {
            return;
        }
    }
}
/// FUN_001756c0 (v12): message 0xb from the tracked object (a2+4 == +0x30) whose FUN_00174898(a2+4) equals the
/// owner (+0x10) sets +0x40 and returns 1; else FUN_001ef0b8
pub fn c2d0620_v12<M: QMem + ?Sized>(m: &mut M, o: u32, msg: u32, a2: u32) -> u32 {
    if msg == 0xb && m.r32(a2 + 4) == m.r32(o + 0x30) && {
        let a = m.r32(a2 + 4);
        m.call(0x0017_4898, &[a], &[]).0 == m.r32(o + 0x10)
    } {
        m.w32(o + 0x40, 1);
        1
    } else {
        m.call(0x001e_f0b8, &[o, msg, a2], &[]).0
    }
}
