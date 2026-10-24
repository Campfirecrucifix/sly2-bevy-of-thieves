//! WIRE-Q7: 1:1 queue ports (against `QMem`) of frontier natives: functions the live dispatch reaches whose
//! only port is a state-struct / host one (tools/wired.py frontier). Ported from the EE asm; callees go
//! through `m.call` / `vslot` (the live dispatch routes ported ones).
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

const T_NOW: u32 = 0x002d_ded0;

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

/// FUN_001715f0(s, cmd): guard script command switch on cmd[0] (0..0xf); always ends +0x3c = DAT_002dded0.
/// cmd 5/6/7/0xd/0xe spawn a +0x10 actor task (FUN_00205438(actor, kind)) and run its slot 7, then +0x40 = 1.
pub fn ee_sub_1715f0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let sp = stack_push(m, 0x40);
    let p = |m: &mut M, k: u32| m.r32(a1 + 4 * k);
    let mut done = false; // -> +0x40 = 1
    match m.r32(a1) {
        0 => {
            let k = p(m, 1);
            let r = m.call(0x0017_15b8, &[a0, k], &[]).0;
            m.w32(a0 + 0x34, r);
        }
        1 | 2 => {
            let off = if m.r32(a1) == 1 { 0x2cc } else { 0x2d4 };
            let o = m.r32(a0 + 0x10);
            let (x, y) = (p(m, 2), p(m, 3));
            m.w32(o + off, x);
            m.w32(o + off + 4, y);
        }
        3 => {
            let k = p(m, 1);
            let r = m.call(0x0017_15b8, &[a0, k], &[]).0;
            m.call(0x0017_1af8, &[a0, r, sp, sp + 0x30], &[]);
            // a1 = the 128-bit quad at sp (a V4 arg), f12 = sp+0x30
            let q = m.rv(sp);
            let f = m.rf(sp + 0x30);
            m.call(0x0017_1b60, &[a0], &[q, f4(f)]);
        }
        4 => {
            m.w32(sp + 0x34, a1 + 4);
            vslot(m, a0, 2, &[4, 0, 1, sp + 0x34, 0], &[]);
        }
        5 => {
            let act = m.r32(a0 + 0x10);
            match m.r32(act + 0xf80) as i32 {
                2 | 3 => {
                    let kind = if m.r32(act + 0xf80) == 2 { 0xbb } else { 0xbc };
                    let t = m.call(0x0020_5438, &[act, kind], &[]).0;
                    vslot(m, t, 7, &[], &[]);
                    done = true;
                }
                -1 => {
                    let t = m.call(0x0020_5438, &[act, 0x9e], &[]).0;
                    vslot(m, t, 7, &[], &[]);
                    done = true;
                }
                _ => done = true,
            }
        }
        6 => {
            let sc = m.r32(a0 + 0x38);
            let k = m.r32(sc + 4);
            let s4 = m.call(0x0017_15b8, &[a0, k], &[]).0;
            if s4 != 0 {
                let act = m.r32(a0 + 0x10);
                match m.r32(act + 0xf80) as i32 {
                    0 => {
                        let t = m.call(0x0020_5438, &[act, 0xb8], &[]).0;
                        let sc = m.r32(a0 + 0x38);
                        let w = m.r32(sc + 8);
                        m.call(0x0017_cdb8, &[t, w], &[]);
                        vslot(m, t, 7, &[], &[]);
                    }
                    1 => {
                        let t = m.call(0x0020_5438, &[act, 0xb9], &[]).0;
                        let sc = m.r32(a0 + 0x38);
                        let w = m.r32(sc + 8);
                        m.call(0x0017_d110, &[t, w], &[]);
                        vslot(m, t, 7, &[], &[]);
                    }
                    -1 => {
                        let t = m.call(0x0020_5438, &[act, 0xc0], &[]).0;
                        m.w32(t + 0x30, s4);
                        let w = p(m, 2);
                        m.w32(t + 0x34, w);
                        let w = m.r32(a0 + 0x4c);
                        m.w32(t + 0x58, w);
                        if m.r32(a0 + 0x4c) == 0 {
                            m.call(0x001e_9c30, &[s4], &[]);
                        }
                        let q = m.r32(a0 + 0x44);
                        if q != 0 {
                            let w = m.r32(q + 8);
                            m.w32(t + 0x38, w);
                        }
                        let k = p(m, 3);
                        if k != u32::MAX {
                            let r = m.call(0x0017_15b8, &[a0, k], &[]).0;
                            m.w32(t + 0x50, r);
                        }
                        vslot(m, t, 7, &[], &[]);
                    }
                    _ => {}
                }
                done = true;
            }
        }
        7 => {
            m.call(0x0025_62a8, &[sp + 0x10], &[]);
            let sc = m.r32(a0 + 0x38);
            let k = m.r32(sc + 4);
            if k == u32::MAX {
                let src = m.r32(sc + 8);
                for i in 0..4 {
                    let w = m.r32(src + 4 * i);
                    m.w32(sp + 0x20 + 4 * i, w);
                }
                let w = m.r32(sc + 0xc);
                m.w32(a0 + 0x48, w);
            } else {
                let s0 = m.call(0x0017_15b8, &[a0, k], &[]).0;
                if s0 != 0 {
                    m.call(0x0017_1af8, &[a0, s0, 0, a0 + 0x48], &[]);
                    let isa = m.call(0x0013_2b10, &[s0, 0xa], &[]).0;
                    m.w32(if isa == 0 { sp + 0x14 } else { sp + 0x10 }, s0);
                }
            }
            let act = m.r32(a0 + 0x10);
            if m.r32(act + 0xf80) == u32::MAX {
                let t = m.call(0x0020_5438, &[act, 0xba], &[]).0;
                m.call(0x0017_dd38, &[t, sp + 0x10], &[]);
                let k = p(m, 4);
                if k != u32::MAX {
                    let r = m.call(0x0017_15b8, &[a0, k], &[]).0;
                    m.w32(t + 0xa8, r);
                }
                let w = p(m, 5);
                m.w32(t + 0xb0, w);
                let w = m.r32(a0 + 0x4c);
                m.w32(t + 0xbc, w);
                vslot(m, t, 7, &[], &[]);
            }
            done = true;
        }
        8 => {
            let k = p(m, 1);
            let r = m.call(0x0017_15b8, &[a0, k], &[]).0;
            m.call(0x0012_f9b8, &[r, 0, 0, sp + 0x38], &[f4(0.0), f4(1.0)]);
        }
        0xa => {
            let (o, k) = (m.r32(a0 + 0x30), p(m, 1));
            let r = m.call(0x0017_11f0, &[o, k], &[]).0;
            if r != 0 {
                m.w32(a0 + 0x38, r);
            }
        }
        0xb => {
            m.call(0x001e_efc0, &[a0], &[]);
        }
        0xc => {
            m.call(0x001e_f038, &[a0], &[]);
        }
        0xd => {
            let sc = m.r32(a0 + 0x38);
            let k = m.r32(sc + 4);
            let s0 = m.call(0x0017_15b8, &[a0, k], &[]).0;
            let act = m.r32(a0 + 0x10);
            let t = m.call(0x0020_5438, &[act, 0xac], &[]).0;
            m.w32(t + 0x60, s0);
            m.w32(t + 0x68, 1);
            let w = p(m, 2);
            m.w32(t + 0x64, w);
            vslot(m, t, 7, &[], &[]);
            done = true;
        }
        0xe => {
            let s0 = a0 + 0x50;
            let act = m.r32(a0 + 0x10);
            m.call(0x0020_90d0, &[s0, act], &[]);
            let (x, y) = (p(m, 1), p(m, 2));
            let act = m.r32(a0 + 0x10);
            m.w32(a0 + 0x54, x);
            m.w32(a0 + 0x5c, y);
            m.call(0x0020_9130, &[s0, act], &[]);
            let act = m.r32(a0 + 0x10);
            if m.r32(act + 0xf80) == u32::MAX {
                let t = m.call(0x0020_5438, &[act, 0x9d], &[]).0;
                m.w32(t + 0x40, s0);
                vslot(m, t, 7, &[], &[]);
                done = true;
            }
        }
        0xf => {
            let k = p(m, 1);
            let r = m.call(0x0017_15b8, &[a0, k], &[]).0;
            let w = p(m, 2);
            m.call(0x001e_0020, &[r, w], &[]);
        }
        _ => {}
    }
    if done {
        m.w32(a0 + 0x40, 1);
    }
    let t = m.r32(T_NOW);
    m.w32(a0 + 0x3c, t);
    stack_pop(m, 0x40);
}

/// FUN_0014d3d8: CM ctor: +0 = +4 = 0, +0xc = 0x2ca4d0 vtable, member ctors, +0x60c = 0x2cd378; returns o.
pub fn ee_sub_14d3d8<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    m.w32(a0, 0);
    m.w32(a0 + 4, 0);
    m.w32(a0 + 0xc, 0x002c_a4d0);
    for (f, w) in [
        (0x0014_fd68u32, 0xe8u32),
        (0x0022_1990, 0xec),
        (0x0014_fde0, 0x114),
        (0x0012_e760, 0x12a),
        (0x0015_1208, 0x130),
        (0x0015_1610, 0x140),
        (0x0015_0728, 0x148),
        (0x0015_18c8, 0x17c),
        (0x0014_fca8, 0x180),
    ] {
        m.call(f, &[a0 + 4 * w], &[]);
    }
    m.w32(a0 + 0x60c, 0x002c_d378);
    a0
}

/// FUN_001928f8(jt, kind, a, b): FUN_00192bd8(jt, -1); +8 = kind, +4 = DAT_002dded0, +0x14 = a, +0x18 = b;
/// slot at vt+0x2c4 of *jt (2); kind -> substate (0/0xd: 5, 8: 7 + FUN_00192a28, 1: FUN_00192a28, 5/9: +0x4c4 =
/// a + FUN_001ccac0(+0x440, 0), 6: (*jt)+0x1854 = a, FUN_0018e540(*jt, 0), FUN_0018ed70(*jt), 0xb/0xc: +0x500 = 1)
/// then FUN_00192bd8(jt, substate).
pub fn ee_sub_1928f8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32) {
    m.call(0x0019_2bd8, &[a0, u32::MAX], &[]);
    m.w32(a0 + 8, a1);
    let o = m.r32(a0);
    let t = m.r32(T_NOW);
    m.w32(a0 + 0x18, a3);
    m.w32(a0 + 4, t);
    m.w32(a0 + 0x14, a2);
    let vt = m.r32(o + 0xc);
    let adj = m.r16(vt + 0x2c0) as i16 as i32 as u32;
    let fp = m.r32(vt + 0x2c4);
    m.call(fp, &[o.wrapping_add(adj), 2], &[]);
    let s = match a1 {
        0 | 0xd => 5,
        1 => {
            m.call(0x0019_2a28, &[a0], &[]);
            0
        }
        8 => {
            m.call(0x0019_2a28, &[a0], &[]);
            7
        }
        5 | 9 => {
            let w = m.r32(a0 + 0x14);
            m.w32(a0 + 0x4c4, w);
            m.call(0x001c_cac0, &[a0 + 0x440, 0], &[]);
            0
        }
        6 => {
            let o = m.r32(a0);
            m.w32(o + 0x1854, a2);
            m.call(0x0018_e540, &[o, 0], &[]);
            m.call(0x0018_ed70, &[o], &[]);
            0
        }
        0xb | 0xc => {
            m.w32(a0 + 0x500, 1);
            0
        }
        _ => 0,
    };
    m.call(0x0019_2bd8, &[a0, s], &[]);
}

/// FUN_00144388(o, x): FUN_00220498(o + 0x11b0, x, FUN_001f4fc8(o)) -> FUN_001cc9a8(o + 0x900, x, that).
pub fn ee_sub_144388<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let r = m.call(0x001f_4fc8, &[a0, a1], &[]).0;
    let r = m.call(0x0022_0498, &[a0 + 0x11b0, a1, r], &[]).0;
    m.call(0x001c_c9a8, &[a0 + 0x900, a1, r], &[]);
}

/// FUN_002075d0(o, msg, d): FUN_001c3e20(o, msg, d); msg 4 from o (d+4) with (*d)+0x184: FUN_0020d460(+0x49c, *d);
/// msg 5 from o with *d == (+0x49c)+0xc: FUN_0020d460(+0x49c, 0); then FUN_001eed78(o + 0x26c, msg, d).
pub fn ee_sub_2075d0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    m.call(0x001c_3e20, &[a0, a1, a2], &[]);
    if a1 == 4 {
        if m.r32(a2 + 4) == a0 {
            let d = m.r32(a2);
            if m.r32(d + 0x184) != 0 {
                let x = m.r32(a0 + 0x49c);
                m.call(0x0020_d460, &[x, d], &[]);
            }
        }
    } else if a1 == 5 && m.r32(a2 + 4) == a0 {
        let x = m.r32(a0 + 0x49c);
        if m.r32(a2) == m.r32(x + 0xc) {
            m.call(0x0020_d460, &[x, 0], &[]);
        }
    }
    m.call(0x001e_ed78, &[a0 + 0x26c, a1, a2], &[]);
}

/// FUN_001384c0(a): FUN_001383f0(a); scale the +0x50 chain's +0x1c by 1/+0x74, FUN_001380f0 for each of +0xcc
/// 12-byte +0xd0 entries (or once with zeros), scale back; with a +0xd8 source copy its +0xa4 column scaled by
/// +0x74 / src+0x74 into +0xa4 (+0xa0 rows, stride 0x10).
pub fn ee_sub_1384c0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0013_83f0, &[a0], &[]);
    let mut q = m.r32(a0 + 0x50);
    while q != 0 {
        let v = m.rf(q + 0x1c) / m.rf(a0 + 0x74);
        m.wf(q + 0x1c, v);
        q = m.r32(q + 0x2c);
    }
    let mut i = 0;
    while (i as i32) < m.r32(a0 + 0xcc) as i32 {
        let e = m.r32(a0 + 0xd0) + 0xc * i;
        let (x, y, z) = (m.r32(e), m.r32(e + 4), m.r32(e + 8));
        m.call(0x0013_80f0, &[a0, x, y, z, a0 + 0x74], &[]);
        i += 1;
    }
    if m.r32(a0 + 0xcc) == 0 {
        m.call(0x0013_80f0, &[a0, 0, 0, 0, a0 + 0x74], &[]);
    }
    let mut q = m.r32(a0 + 0x50);
    while q != 0 {
        let v = m.rf(q + 0x1c) * m.rf(a0 + 0x74);
        m.wf(q + 0x1c, v);
        q = m.r32(q + 0x2c);
    }
    let s = m.r32(a0 + 0xd8);
    if s != 0 {
        let k = m.rf(a0 + 0x74) / m.rf(s + 0x74);
        let mut i = 0;
        while (i as i32) < m.r32(a0 + 0xa0) as i32 {
            let sd = m.r32(a0 + 0xd8);
            let src = m.r32(sd + 0xa4) + 0x10 * i;
            let dst = m.r32(a0 + 0xa4) + 0x10 * i;
            let v = k * m.rf(src);
            m.wf(dst, v);
            i += 1;
        }
    }
}

/// FUN_00177618(b): guard brain 0x9c substate machine. FUN_001eef98(b); loop while !FUN_001ef0a8(b):
/// snapshot +0x30..+0x9f, switch on state +0x30 (0..0x13) -> maybe FUN_00176750(b, {state, .., +0x10 target,
/// +0x14}) / FUN_00177ba0(b), until FUN_00177dc0(b + 0x30, snapshot) says the state settled.
pub fn ee_sub_177618<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let sp = stack_push(m, 0xe0);
    let (snap, buf) = (sp, sp + 0x70);
    m.call(0x001e_ef98, &[a0], &[]);
    let clr = |m: &mut M| {
        m.call(0x0023_33b4, &[buf, 0, 0x70], &[]);
    };
    // the `goto LAB_00177a28` tail: buf[0] = st, buf+0x10 = tgt, FUN_00176750(b, buf)
    let post = |m: &mut M, st: u32, tgt: u32| {
        m.w32(buf, st);
        m.w32(buf + 0x10, tgt);
        m.call(0x0017_6750, &[a0, buf], &[]);
    };
    // the shared "has a target t": 3 (plain) or 0x12 + FUN_00208bb0 (o).
    let target = |m: &mut M, o: u32, t: u32| {
        if m.call(0x0020_8b38, &[o, t], &[]).0 == 0 {
            post(m, 3, t);
        } else {
            let w = m.call(0x0020_8bb0, &[o], &[]).0;
            m.w32(buf + 0x14, w);
            post(m, 0x12, t);
        }
    };
    let idle = |m: &mut M| {
        clr(m);
        post(m, 1, 0);
    };
    let busy = |m: &mut M| {
        clr(m);
        post(m, 0xc, 0);
    };
    while m.call(0x001e_f0a8, &[a0], &[]).0 == 0 {
        for i in 0..0x1c {
            let w = m.r32(a0 + 0x30 + 4 * i);
            m.w32(snap + 4 * i, w);
        }
        let o = m.r32(a0 + 0x10);
        let s18 = m.r32(a0 + 0x18);
        // Some(r): `if r != 0 { FUN_00177ba0 }`
        let mut fin: Option<u32> = None;
        match m.r32(a0 + 0x30) {
            0 => idle(m),
            1 | 0x10 | 0x11 => {
                clr(m);
                if m.call(0x0017_7d20, &[a0], &[]).0 != 0 {
                    post(m, 0xc, 0);
                } else {
                    let t = vslot(m, o, 87, &[], &[]).0;
                    if t != 0 {
                        target(m, o, t);
                    } else {
                        let t = m.call(0x0020_a408, &[o, 0], &[]).0;
                        if t != 0 {
                            post(m, 2, t);
                        } else if m.r32(a0 + 0x30) == 0x11 && m.rf(a0 + 0xa4) > m.rf(T_NOW) {
                            post(m, 0x11, 0);
                        } else if m.call(0x0017_7d48, &[a0], &[]).0 != 0 {
                            post(m, 0x10, 0);
                        } else {
                            post(m, 1, 0);
                        }
                    }
                }
            }
            2 => {
                if m.call(0x0017_7d20, &[a0], &[]).0 != 0 {
                    busy(m);
                } else {
                    let t = vslot(m, o, 87, &[], &[]).0;
                    if t != 0 {
                        clr(m);
                        target(m, o, t);
                    } else {
                        let w = m.r32(s18 + 0x44);
                        let r = m.call(0x0020_a408, &[o, w], &[]).0;
                        if r != w {
                            m.call(0x0017_25e0, &[s18, r], &[]);
                        } else if m.call(0x0017_2550, &[s18], &[]).0 != 0 {
                            idle(m);
                        }
                    }
                }
            }
            3 => {
                if m.call(0x0017_7d20, &[a0], &[]).0 != 0 {
                    busy(m);
                } else if m.call(0x0017_4fb8, &[s18], &[]).0 != 0 {
                    clr(m);
                    let st = if m.call(0x0013_2b10, &[o, 0x3b], &[]).0 == 0 { 4 } else { 0xd };
                    let t = m.r32(a0 + 0x40);
                    post(m, st, t);
                }
            }
            4 => {
                if m.call(0x0017_7d20, &[a0], &[]).0 != 0 {
                    busy(m);
                } else if m.call(0x0017_5810, &[s18], &[]).0 != 0 {
                    idle(m);
                } else {
                    let t = vslot(m, o, 87, &[], &[]).0;
                    if t != m.r32(o + 0x38c) {
                        clr(m);
                        if t == 0 {
                            m.w32(buf, 1);
                            let q = m.r32(s18 + 0x30);
                            if q != 0
                                && m.call(0x0013_2b10, &[q, 0x34], &[]).0 != 0
                                && vslot(m, q, 76, &[], &[]).0 == 2
                            {
                                let e = m.r32(q + 0xe20);
                                let x = m.call(0x0017_4898, &[e], &[]).0;
                                if x != 0 {
                                    m.w32(buf, 0x11);
                                    m.w32(buf + 0x10, x);
                                }
                            }
                            let (st, tg) = (m.r32(buf), m.r32(buf + 0x10));
                            post(m, st, tg);
                        } else {
                            target(m, o, t);
                        }
                    }
                }
            }
            5 => fin = Some(m.call(0x0017_1458, &[s18], &[]).0),
            6 => fin = Some(m.call(0x0017_a9c0, &[s18], &[]).0),
            7 => {
                if m.call(0x0017_4fb8, &[s18], &[]).0 != 0 {
                    let k = m.r32(s18 + 0x50);
                    if k == u32::MAX {
                        let w = m.r32(s18 + 0x5c);
                        m.call(0x0017_6b80, &[a0, (-50i32) as u32, w], &[]);
                        if m.r32(a0 + 0x30) == 7 {
                            fin = Some(1);
                        }
                    } else if k == 3 {
                        fin = Some(1);
                    }
                }
            }
            8 => {
                if m.call(0x0017_4fb8, &[s18], &[]).0 != 0 {
                    let pos = m.call(0x001a_62e8, &[o], &[]).1;
                    let h = m.call(0x0015_6b98, &[0x003d_d490, 0, o], &[pos]).0;
                    clr(m);
                    m.w32(buf, 2);
                    m.w32(buf + 0x10, h);
                    m.call(0x0017_6750, &[a0, buf], &[]);
                    m.call(0x0015_6780, &[h], &[]);
                }
            }
            9 | 10 => fin = Some(m.call(0x0017_4fb8, &[s18], &[]).0),
            0xc => {
                if m.call(0x0017_7d20, &[a0], &[]).0 == 0 {
                    idle(m);
                }
            }
            0x12 => {
                if m.call(0x0017_de40, &[s18], &[]).0 != 0 {
                    m.call(0x0017_7ba0, &[a0], &[]);
                }
            }
            0x13 => fin = Some(m.call(0x0017_eae8, &[s18], &[]).0),
            _ => {}
        }
        if fin.unwrap_or(0) != 0 {
            m.call(0x0017_7ba0, &[a0], &[]);
        }
        if m.call(0x0017_7dc0, &[a0 + 0x30, snap], &[]).0 != 0 {
            break;
        }
    }
    stack_pop(m, 0xe0);
}

/// FUN_00140f90(jt, msg, d): Jt message handler (1 drop, 3 lost, 4/5 attach/detach notify, 9 anim event,
/// 0xb release), then FUN_001eed78(jt + 0x26c, msg, d) and FUN_001c3e20(jt, msg, d).
pub fn ee_sub_140f90<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    let sp = stack_push(m, 0x20);
    match a1 {
        1 => {
            if m.r32(a0 + 0x4c8) == 0xb && m.r32(a0 + 0x4d4) == a2 {
                m.call(0x0013_3448, &[a2, a0], &[]);
                m.w32(a0 + 0x4d4, 0);
            }
        }
        3 => {
            let hit = a2 == m.r32(a0 + 0x1030) || {
                let q = m.r32(a0 + 0x1034);
                q != 0 && a2 == m.r32(q + 0x30)
            };
            if hit {
                let k = m.r32(a0 + 0xdf4);
                m.w32(a0 + 0x1038, u32::MAX);
                if k.wrapping_sub(8) < 5 {
                    m.call(0x001a_15d0, &[a0 + 0xdf0], &[]);
                } else {
                    let s = m.r32(a0 + 0x11b4) as i32;
                    if s > 0 && s < 3 {
                        m.call(0x0022_06c8, &[a0 + 0x11b0, 0], &[]);
                    } else if s > 4 && s < 7 {
                        m.call(0x0022_06c8, &[a0 + 0x11b0, 4], &[]);
                    }
                }
            }
        }
        4 | 5 => {
            if m.r32(a2 + 4) == a0 && (a1 == 4 || m.r32(a2) == m.r32(a0 + 0x108c)) {
                let w = if a1 == 4 { m.r32(a2) } else { 0 };
                m.w32(a0 + 0x108c, w);
                let mut i = 0;
                while (i as i32) < m.r32(a0 + 0x1094) as i32 {
                    let e = m.r32(a0 + 0x1090) + 8 * i;
                    let o = m.r32(e);
                    vslot(m, o, if a1 == 4 { 7 } else { 8 }, &[], &[]);
                    if a1 == 4 {
                        let e = m.r32(a0 + 0x1090) + 8 * i;
                        m.w32(e + 4, 0);
                    }
                    i += 1;
                }
            }
        }
        9 => {
            let h = m.r32(a2);
            if m.r32(h + 0x18) == a0 {
                msg9(m, a0, a2, sp);
            }
        }
        0xb => {
            if m.r32(a2 + 4) == a0 {
                m.call(0x0013_0658, &[a0 + 0x1310], &[]);
                m.w32(a0 + 0x1314, u32::MAX);
            }
        }
        _ => {}
    }
    m.call(0x001e_ed78, &[a0 + 0x26c, a1, a2], &[]);
    m.call(0x001c_3e20, &[a0, a1, a2], &[]);
    stack_pop(m, 0x20);
}

fn msg9<M: QMem + ?Sized>(m: &mut M, a0: u32, a2: u32, sp: u32) {
    match m.r32(a2 + 8) {
        0x1c7 => {
            if m.r32(a0 + 0x4c8) == 0xc {
                let x = m.r32(a0 + 0x1030);
                m.call(0x0013_1a00, &[x], &[f4(1.0)]);
                let y = m.r32(a0 + 0x4d4);
                m.call(0x001c_d098, &[a0 + 0x900, y], &[]);
            }
        }
        0x161 => m.w32(a0 + 0x1044, 1),
        0x15e => {
            let s = m.r32(a2 + 4);
            let v = m.call(0x0020_4e38, &[s, a0], &[]).1;
            m.wv(a0 + 0x190, [-v[0], -v[1], -v[2], -v[3]]);
            let (r0, r1, r2) = (m.rv(a0 + 0x1b0), m.rv(a0 + 0x1c0), m.rv(a0 + 0x1d0));
            let mut o = [0.0f32; 4];
            for k in 0..3 {
                o[k] = r0[k] * v[0] + r1[k] * v[1] + r2[k] * v[2];
            }
            m.wv(sp, [-1.0, 0.0, 0.0, 0.0]);
            vslot(m, a0, 34, &[], &[o]);
        }
        0x162 => {
            let q = m.rv(a0 + 0x1060);
            m.w32(a0 + 0x1044, 0);
            let p = m.call(0x001a_62e8, &[a0], &[]).1;
            m.wv(a0 + 0x1070, [q[0] - p[0], q[1] - p[1], q[2] - p[2], q[3] - p[3]]);
            m.w32(a0 + 0x1078, 0);
        }
        0x27b => {
            if m.r32(a0 + 0x11b4) != u32::MAX {
                m.call(0x0022_05a8, &[a0 + 0x11b0], &[]);
            }
        }
        0x1c8 => {
            if m.r32(a0 + 0x4c8) == 0xc {
                let y = m.r32(a0 + 0x4d4);
                m.call(0x001c_cc88, &[a0 + 0x900, y], &[]);
            }
        }
        0x27c => {
            if m.r32(a0 + 0x4c8) != 0xb {
                return;
            }
            let t = m.r32(a0 + 0x4d4);
            if t != 0 && m.call(0x0013_2b10, &[t, 100], &[]).0 != 0 && m.call(0x0022_4590, &[t], &[]).0 != 0 {
                m.call(0x0022_45b8, &[t], &[]);
            }
            if m.r32(a0 + 0x130c) == 0 {
                if m.r32(a0 + 0x9bc) != u32::MAX {
                    m.call(0x0014_1cd0, &[a0], &[]);
                    match m.r32(a0 + 0x9bc) {
                        0x15 => {
                            m.call(0x0019_f220, &[0x003d_d490, 0x15], &[]);
                        }
                        0xe => {
                            m.call(0x0019_f170, &[0x003d_d490, 1], &[]);
                        }
                        0x17 => {
                            let g = m.r32(0x002e_1e40);
                            let g = m.r32(g + 0x2010);
                            let l = m.call(0x001c_b250, &[g, 0], &[]).0;
                            if l != 0 {
                                m.call(0x001a_58d0, &[l, 0, 3], &[]);
                            }
                            // safe list walk over DAT_003e148c (frame pushed on DAT_002dd2fc)
                            let head = m.r32(0x003e_148c);
                            let stride = m.r32(0x003e_1494);
                            let prev = m.r32(0x002d_d2fc);
                            m.w32(sp, 0x003e_148c);
                            m.w32(sp + 0xc, prev);
                            m.w32(sp + 4, head.wrapping_add(stride));
                            m.w32(sp + 8, stride);
                            m.w32(0x002d_d2fc, sp);
                            let mut n = head;
                            while n != 0 {
                                if m.r32(n + 0x54) == 4 && m.r32(n + 0x44) == a0 {
                                    m.call(0x0015_6780, &[n], &[]);
                                }
                                let nx = m.r32(sp + 4);
                                let st = m.r32(sp + 8);
                                n = m.r32(nx);
                                m.w32(sp + 4, n.wrapping_add(st));
                            }
                            let prev = m.r32(sp + 0xc);
                            m.w32(0x002d_d2fc, prev);
                            let t = m.rf(T_NOW) + m.rf(0x002b_cc94);
                            m.wf(0x003e_1be8, t);
                        }
                        _ => {}
                    }
                }
                m.w32(a0 + 0x130c, 0);
            } else {
                if m.r32(a0 + 0x1314) == m.r32(a0 + 0x9bc) {
                    m.call(0x0013_0658, &[a0 + 0x1310], &[]);
                    m.w32(a0 + 0x1314, u32::MAX);
                } else {
                    m.call(0x0014_1cd0, &[a0], &[]);
                    let k = m.r32(a0 + 0x130c);
                    m.call(0x0013_05b8, &[k, a0, 8, a0 + 0x1310], &[f4(0.0), f4(1.0)]);
                    let w = m.r32(a0 + 0x9bc);
                    m.w32(a0 + 0x1314, w);
                }
                m.w32(a0 + 0x130c, 0);
            }
            m.w32(a0 + 0x9bc, u32::MAX);
        }
        _ => {}
    }
}

/// FUN_00188f20(jt, msg, d): FUN_00140f90(jt, msg, d), then the Jt-level handling of msg 1 (drop: release
/// the cane parts holding d), 3 (lost target), 9 (anim events 0x164..0x174) and 0xb (recoil).
pub fn ee_sub_188f20<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    m.call(0x0014_0f90, &[a0, a1, a2], &[]);
    match a1 {
        1 => {
            let k = m.r32(a0 + 0x18c8);
            let chk = if k == 7 {
                let s = m.r32(a0 + 0x4c8) as i32;
                if s > 2 && s < 5 && m.r32(a0 + 0x4d4) == a2 {
                    m.call(0x0019_5a00, &[a0 + 0x2c0, 0, 0, 0], &[]);
                }
                false
            } else {
                k == 3 || k == 0x16
            };
            if chk && m.r32(a0 + 0x18c4) == a2 {
                m.call(0x0019_5a00, &[a0 + 0x2c0, 0, 0, 0], &[]);
            }
            for i in 0..4 {
                let e = a0 + 0x1fb0 + 0xc * i;
                if m.r32(e + 8) == a2 {
                    m.call(0x0018_d698, &[a0, e, 0], &[]);
                }
            }
        }
        3 => {
            if a2 == m.r32(a0 + 0x18e0) {
                let s = if m.r32(a0 + 0x18d8) == 10 { 2 } else { u32::MAX };
                m.call(0x0018_e540, &[a0, s], &[]);
                return;
            }
            if a2 != m.r32(a0 + 0x1030) {
                return;
            }
            match m.r32(a0 + 0x18c8) {
                5 => {
                    let s = m.r32(a0 + 0x18d0) as i32;
                    if s == 8 || s == 9 {
                        m.call(0x0018_c250, &[a0, 5, 4], &[]);
                    }
                }
                7 | 0x15 => m.w32(a0 + 0x1038, u32::MAX),
                9 | 0xb => {
                    m.call(0x0018_c250, &[a0, 0, u32::MAX], &[]);
                }
                _ => {}
            }
        }
        9 => {
            let src = m.r32(a2);
            let ok = src == m.r32(a0 + 0x1030) || src == m.r32(a0 + 0x18e0) || {
                let q = m.r32(a0 + 0x1034);
                q != 0 && src == m.r32(q + 0x30)
            };
            if !ok {
                return;
            }
            match m.r32(a2 + 8) {
                0x164 => m.w32(a0 + 0x18e4, 1),
                0x165 => {
                    if m.r32(a0 + 0x18ec) != 0 {
                        match m.r32(a0 + 0x18d8) {
                            0xc | 0xe | 0x10 | 0x12 | 0x14 => {
                                m.call(0x0018_c250, &[a0, 5, 6], &[]);
                            }
                            0xd | 0xf | 0x11 | 0x13 | 0x15 => {
                                m.call(0x0018_c250, &[a0, 5, 7], &[]);
                            }
                            _ => {}
                        }
                    }
                }
                0x166 => m.w32(a0 + 0x18e8, 1),
                0x167 => {
                    let o = m.r32(a0 + 0x10dc);
                    if o == 0 || m.r32(a0 + 0x1bd4) == 0 {
                        return;
                    }
                    let p = m.call(0x001a_62e8, &[a0], &[]).1;
                    let t = m.call(0x001f_1e20, &[o], &[p]).0;
                    if t != 0
                        && m.call(0x0013_2b10, &[t, 0x5b], &[]).0 == 0
                        && m.call(0x0013_2b10, &[t, 0x5d], &[]).0 == 0
                    {
                        let u = m.call(0x0015_e938, &[0x003d_d490, 1, 0x282, a0], &[]).0;
                        m.w32(a0 + 0x1bcc, t);
                        m.call(0x0014_24e0, &[a0, u, t, u32::MAX], &[]);
                    }
                }
                0x16d => {
                    let p = m.call(0x001a_62e8, &[a0], &[]).1;
                    m.call(0x0015_6b98, &[0x003d_d490, 0, a0], &[p]);
                }
                0x16e => {
                    let (x, y) = (m.r32(a0 + 0x18e0), m.r32(a0 + 0x10b8));
                    m.call(0x0013_1b10, &[x, a0, y], &[]);
                }
                0x16f | 0x170 => {
                    let x = m.r32(a0 + 0x18e0);
                    let t = if m.r32(a2 + 8) == 0x16f { 0x002b_f2a0 } else { 0x002b_f2b8 };
                    m.call(0x0013_1b60, &[x, a0, 3, t], &[]);
                }
                0x173 => {
                    m.w32(a0 + 0x1d2c, 1);
                    m.w32(a0 + 0x194, 0);
                    m.w32(a0 + 0x198, 0);
                    m.w32(a0 + 0x190, 0);
                    let v = m.rv(a0 + 0x190);
                    vslot(m, a0, 34, &[], &[v]);
                    let o = m.r32(a0 + 0x10dc);
                    let t = m.call(0x0020_5458, &[o, 0xb1], &[]).0;
                    if t != 0 {
                        m.call(0x0017_b1a8, &[t, 4], &[]);
                    }
                }
                0x174 => {
                    let o = m.r32(a0 + 0x10dc);
                    let t = m.call(0x0020_5458, &[o, 0xb1], &[]).0;
                    if t != 0 {
                        m.call(0x0017_b1a8, &[t, 6], &[]);
                    }
                }
                _ => {}
            }
        }
        0xb => {
            if m.r32(a2 + 4) != a0 {
                return;
            }
            let r = m.r32(a2 + 8);
            if m.r32(r + 0x44) & 2 == 0 {
                return;
            }
            let u = m.r32(r + 0xc);
            let s = if u & 2 == 0 {
                if u & 4 != 0 { 8 } else { return }
            } else {
                match m.r32(a0 + 0x18d8) {
                    4 | 6 | 0xc | 0xe | 0x10 | 0x12 | 0x14 => 8,
                    5 | 7 | 0xd | 0xf | 0x11 | 0x13 | 0x15 => 9,
                    _ => return,
                }
            };
            m.call(0x0018_c250, &[a0, 0, u32::MAX], &[]);
            m.call(0x0018_e540, &[a0, s], &[]);
        }
        _ => {}
    }
}

fn scale190<M: QMem + ?Sized>(m: &mut M, a0: u32, k: u32) {
    let s = m.rf(k);
    let v = m.rv(a0 + 0x190);
    m.wv(a0 + 0x190, [v[0] * s, v[1] * s, v[2] * s, v[3] * s]);
}

/// FUN_0018bce0(jt): JtUpdate. FUN_001403d0; only with +0x28 bit 60: FUN_0018b7e8 if +0x150, per-state
/// (+0x18c8) update, FUN_0018ed70, then (no +0x18d8 lock) the pipe / rail grab test against +0x1854
/// (FUN_001e7380 / FUN_0011e7c8 segment distance < +0x19ac -> FUN_0018c250), FUN_00189618.
pub fn ee_sub_18bce0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0014_03d0, &[a0], &[]);
    if (m.r64(a0 + 0x28) >> 60) & 1 == 0 {
        return;
    }
    let sp = stack_push(m, 0x60);
    if m.r32(a0 + 0x150) != 0 {
        m.call(0x0018_b7e8, &[a0], &[]);
    }
    match m.r32(a0 + 0x18c8) {
        0 => {
            m.call(0x0018_a0c8, &[a0], &[]);
        }
        1 => {
            let p = m.r32(a0 + 0x150);
            let f = m.rf(p + 0x18);
            m.call(0x0020_4660, &[a0], &[f4(f)]);
            m.call(0x0020_bb58, &[a0 + 0x1320], &[f4(1.0)]);
            m.call(0x0020_bc08, &[a0 + 0x1320], &[]);
            scale190(m, a0, 0x002b_f3d4);
        }
        3 => {
            m.call(0x0019_0e18, &[a0], &[]);
        }
        5 => {
            if m.r32(a0 + 0x18d0) == 5 {
                let w = m.r32(a0 + 0x14d0);
                m.call(0x001a_0450, &[a0, a0 + 0x13d0, w], &[]);
                scale190(m, a0, 0x002b_f3d8);
                m.wf(a0 + 0x180, 1.0);
            }
        }
        10 => {
            let w = m.r32(a0 + 0x1bc0);
            m.call(0x001a_0450, &[a0, a0 + 0x1ac0, w], &[]);
        }
        0xc => {
            m.call(0x0020_bc08, &[a0 + 0x1c30], &[]);
        }
        0x15 => {
            m.call(0x0018_5e40, &[a0], &[]);
        }
        0x16 => {
            m.call(0x0018_df60, &[a0], &[]);
        }
        0x18 => {
            let n0 = m.r32(a0 + 0x19f0);
            if n0 != 0 {
                let f = m.rf(0x002b_f3dc);
                let (x, y) = (m.r32(a0 + 0x19d8), m.r32(a0 + 0x19e0));
                m.call(0x0018_9c40, &[a0, x, y, n0, a0 + 0x1520], &[f4(f)]);
                let f = m.rf(0x002b_f3dc);
                let (x, y, n1) = (m.r32(a0 + 0x19d4), m.r32(a0 + 0x19dc), m.r32(a0 + 0x19f4));
                m.call(0x0018_9c40, &[a0, x, y, n1, a0 + 0x1620], &[f4(f)]);
                let n = m.r32(a0 + 0x19f0);
                m.call(0x0018_9ec8, &[a0, a0 + 0x1520, n], &[]);
                let n = m.r32(a0 + 0x19f4);
                m.call(0x0018_9ec8, &[a0, a0 + 0x1620, n], &[]);
            }
        }
        _ => {}
    }
    m.call(0x0018_ed70, &[a0], &[]);
    'grab: {
        if m.r32(a0 + 0x18d8) != 0 {
            break 'grab;
        }
        let t = m.r32(a0 + 0x1854);
        if m.call(0x0013_2b10, &[t, 0xb], &[]).0 != 0 {
            let t = m.r32(a0 + 0x1854);
            for i in 0..3 {
                let q = m.rv(a0 + 0x1890 + 0x10 * i);
                let f = m.rf(a0 + 0x1858);
                m.call(0x001e_7380, &[t, 0, sp + 0x10 + 0x10 * i, 0, 0, 0], &[q, f4(f)]);
            }
        } else {
            let t = m.r32(a0 + 0x1854);
            if m.call(0x0013_2b10, &[t, 0x13], &[]).0 == 0 {
                break 'grab;
            }
            let t = m.r32(a0 + 0x1854);
            let p = m.call(0x001a_62e8, &[t], &[]).1;
            for k in 0..3 {
                m.wv(sp + 0x10 + 0x10 * k, p);
            }
        }
        let mut i = 1;
        while (i as i32) < m.r32(a0 + 0x1880) as i32 {
            let a = m.rv(a0 + 0x1890 + 0x10 * (i - 1));
            let b = m.rv(a0 + 0x1890 + 0x10 * i);
            let c = m.rv(sp + 0x10 * i);
            let d = m.rv(sp + 0x10 * i + 0x10);
            m.call(0x0011_e7c8, &[sp, sp + 0x50, sp + 0x40], &[a, b, c, d]);
            if m.rf(sp + 0x50) < m.rf(a0 + 0x19ac) {
                let t = m.r32(a0 + 0x1854);
                if m.call(0x0013_2b10, &[t, 0x13], &[]).0 == 0 {
                    let t = m.r32(a0 + 0x1854);
                    let f = m.r32(a0 + 0x1858);
                    m.w32(a0 + 0xf5c, t);
                    m.w32(a0 + 0xf60, f);
                    m.call(0x0018_c250, &[a0, 8, u32::MAX], &[]);
                } else {
                    let t = m.r32(a0 + 0x1854);
                    m.w32(a0 + 0x18c4, t);
                    if m.r32(t + 0x124) == 3 {
                        m.call(0x0018_c250, &[a0, 0x16, u32::MAX], &[]);
                    } else {
                        m.call(0x0018_c250, &[a0, 3, 1], &[]);
                    }
                }
                break;
            }
            i += 1;
        }
    }
    m.call(0x0018_9618, &[a0], &[]);
    stack_pop(m, 0x60);
}

/// FUN_00131290(ev, e): anim event dispatch: slot 0 (begin) on ev, +0x1c = e[0], switch e[1] (0..0x18:
/// sound / effect / message / camera / scripted calls), slot 1 (end). `flip` = e lies inside the owner's
/// +0xa4 table (+0xa0 entries of 0x10) while +0x40 < 0 (played backwards): the on/off pairs swap.
pub fn ee_sub_131290<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    const W: u32 = 0x003d_d490;
    let sp = stack_push(m, 0x40);
    vslot(m, a0, 0, &[], &[]);
    let s4 = m.r32(a0 + 0x14);
    let e0 = m.r32(a1);
    m.w32(a0 + 0x1c, e0);
    let mut flip = 0u32;
    if m.rf(a0 + 0x40) < 0.0 {
        let base = m.r32(s4 + 0xa4);
        if a1 >= base {
            let n = m.r32(s4 + 0xa0);
            flip = (a1 < base.wrapping_add(n << 4)) as u32;
        }
    }
    let k = m.r32(a1 + 4);
    let p = |m: &mut M, i: u32| m.r32(a1 + 4 * i);
    let pf = |m: &mut M, i: u32| m.rf(a1 + 4 * i);
    match k {
        0 => {
            let u = m.call(0x0013_1270, &[a0], &[]).0;
            let t = p(m, 2);
            let e: Vec<u32> = (0..4).map(|i| m.r32(t + 4 * i)).collect();
            let f: Vec<f32> = (4..7).map(|i| m.rf(t + 4 * i)).collect();
            let s18 = m.r32(a0 + 0x18);
            m.call(0x0021_0740, &[W, e[0], e[1], e[2], e[3], s18, u, s4, a0 + 0x5c], &[f4(f[0]), f4(f[1]), f4(f[2])]);
        }
        1 => {
            let x = p(m, 2);
            m.call(0x0021_08e8, &[W, x, s4, a0 + 0x5c], &[]);
        }
        2 => {
            let x = p(m, 2);
            m.call(0x0014_f750, &[0x002d_df40, x], &[]);
        }
        3 => {
            let x = p(m, 2);
            let f = pf(m, 3);
            m.call(0x0018_7e28, &[0x002d_fc00, x], &[f4(f)]);
        }
        4 => {
            let u = m.call(0x0013_1270, &[a0], &[]).0;
            let x = p(m, 2);
            let s0 = m.call(0x0015_e938, &[W, 4, x, u], &[]).0;
            if s0 != 0 {
                match m.call(0x0013_2ea0, &[s0], &[]).0 {
                    0x6d => {
                        m.call(0x0012_f9b8, &[s0, 0, 0, 0], &[f4(0.0), f4(1.0)]);
                    }
                    0x72 => {
                        m.call(0x001d_fd58, &[s0, 0], &[]);
                    }
                    _ => {}
                }
            }
        }
        5 => {
            let (f, g) = (pf(m, 2), pf(m, 3));
            m.call(0x0013_0d30, &[a0, 0], &[f4(f), f4(g)]);
        }
        6 => {
            m.call(0x0013_0b18, &[a0], &[]);
        }
        7 | 8 => {
            let fl = ((k == 7) as u32) ^ flip;
            let u = m.call(0x0013_1270, &[a0], &[]).0;
            let f = pf(m, 3);
            let x = p(m, 2);
            m.call(0x0021_0338, &[W, fl, x, u, s4], &[f4(f)]);
        }
        9 | 10 => {
            let fl = ((k == 9) as u32) ^ flip;
            m.call(0x0013_1990, &[a0, fl], &[]);
        }
        0xb => {
            let f = m.r32(a1);
            let x = p(m, 2);
            m.w32(sp + 0x20, a0);
            m.w32(sp + 0x24, f);
            m.w32(sp + 0x28, x);
            vslot(m, a0, 3, &[9, sp + 0x20], &[]);
        }
        0xc => {
            m.w32(sp + 0x30, a1 + 8);
            vslot(m, a0, 2, &[4, 0, 1, sp + 0x30, 0], &[]);
        }
        0xd..=0x10 => {
            let on = if k <= 0xe { 0xd } else { 0xf };
            let fl = ((k == on) as u32) ^ flip;
            let u = m.call(0x0013_1270, &[a0], &[]).0;
            let (x, y, s18) = (p(m, 2), p(m, 3), m.r32(a0 + 0x18));
            let f = if k <= 0xe { 0x0021_0408 } else { 0x0021_04a0 };
            m.call(f, &[W, fl, x, y, s18, u, s4], &[]);
        }
        0x11 => {
            let t = p(m, 2);
            let u = m.call(0x0013_1270, &[a0], &[]).0;
            let (x, y, z, f) = (m.r32(t), m.r32(t + 4), m.r32(t + 8), m.rf(t + 0xc));
            m.call(0x0021_0538, &[W, x, y, z, u, s4], &[f4(f)]);
        }
        0x12 => {
            let f = m.rf(a0 + 0x1c);
            m.call(0x0016_18f8, &[W], &[f4(f)]);
        }
        0x13 => {
            m.call(0x0016_1958, &[W], &[]);
        }
        0x14 => {
            let t = p(m, 2);
            let u = m.call(0x0013_1270, &[a0], &[]).0;
            let (x, y, f) = (m.r32(t), m.r32(t + 4), m.rf(t + 8));
            m.call(0x0021_0678, &[W, x, y, u, s4], &[f4(f)]);
        }
        0x15 => {
            let x = p(m, 2);
            m.call(0x0023_0cf0, &[a0, x], &[]);
        }
        0x16 | 0x17 => {
            let o = m.r32(a0 + 0x18);
            let x = if k == 0x16 { p(m, 2) } else { u32::MAX };
            m.call(0x001f_81e0, &[o, x], &[]);
        }
        0x18 => {
            let o = m.r32(a0 + 0x18);
            if o != 0 && m.call(0x0013_2b10, &[o, 0x34], &[]).0 != 0 {
                let x = p(m, 2);
                let o = m.r32(a0 + 0x18);
                m.call(0x0014_3090, &[o, x, x + 8, x + 0x10], &[]);
            }
        }
        _ => {}
    }
    vslot(m, a0, 1, &[], &[]);
    stack_pop(m, 0x40);
}

/// FUN_0013b250(o, x): +0x1e8 = x; FUN_001a58d0(o, x+0x18 == 1 ? 0 : x, 3); FUN_001f4898/FUN_001f4930(o, q2a2dd0);
/// FUN_001f5720(o, 1); FUN_001f57c0(o, 1); slot 2 (vt+0x1c) msg 0x1b.
pub fn ee_sub_13b250<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.w32(a0 + 0x1e8, a1);
    let x = if m.r32(a1 + 0x18) == 1 { 0 } else { a1 };
    m.call(0x001a_58d0, &[a0, x, 3], &[]);
    let q = m.rv(0x002a_2dd0);
    m.call(0x001f_4898, &[a0], &[q]);
    let q = m.rv(0x002a_2dd0);
    m.call(0x001f_4930, &[a0], &[q]);
    m.call(0x001f_5720, &[a0, 1], &[]);
    m.call(0x001f_57c0, &[a0, 1], &[]);
    vslot(m, a0, 2, &[0x1b, 0, 0, 0, 0], &[]);
}

/// FUN_00207030(o, k): actor substate switch +0xfbc (exit old / enter new; gated by FUN_00206eb0); +0xfc0 = now.
pub fn ee_sub_207030<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if a1 == m.r32(a0 + 0xfbc) {
        return;
    }
    if m.call(0x0020_6eb0, &[a0, a1], &[]).0 == 0 {
        return;
    }
    let old = m.r32(a0 + 0xfbc);
    m.w32(a0 + 0xfbc, a1);
    let t = m.rf(T_NOW);
    m.wf(a0 + 0xfc0, t);
    match old {
        8 | 0xd => {
            let p = m.r32(a0 + 0xfc4);
            if p != 0 {
                let q = m.rv(0x002a_2de0);
                m.call(0x0012_d108, &[p], &[q]);
            }
        }
        0xe => {
            let t = m.call(0x0020_5458, &[a0, 0x9d], &[]).0;
            if t != 0 {
                let x = m.r32(t + 0x40);
                let v = m.call(0x0020_9518, &[x], &[]).0;
                if v != 0 {
                    let w = m.r32(v + 0x14);
                    let f = m.rf(w + 0x78);
                    m.call(0x0013_1a20, &[v], &[f4(f)]);
                }
            }
        }
        0x15 => {
            let s = m.call(0x0020_5458, &[a0, 0x9c], &[]).0;
            if s != 0 && m.r32(s + 0x30) == 0x12 {
                let g = m.call(0x001c_4840, &[0x003d_d490], &[]).0;
                if m.r32(s + 0x40) == g && m.rf(s + 0x44) == m.rf(0x002a_8698) {
                    m.call(0x0017_7ba0, &[s], &[]);
                }
            }
        }
        0x22 | 0x25 => {
            m.call(0x0020_8728, &[a0, 0x38], &[]);
            m.call(0x0020_6e50, &[a0], &[]);
        }
        _ => {}
    }
    match a1 {
        8 | 0xd => {
            let p = m.r32(a0 + 0xfc4);
            if p != 0 {
                let q = m.rv(0x002c_3d30);
                m.call(0x0012_d108, &[p], &[q]);
            }
        }
        0xe => {
            let t = m.call(0x0020_5458, &[a0, 0x9d], &[]).0;
            if t != 0 {
                let x = m.r32(t + 0x40);
                let v = m.call(0x0020_9518, &[x], &[]).0;
                if v != 0 {
                    m.call(0x0013_1a20, &[v], &[f4(0.0)]);
                }
            }
            m.w32(a0 + 0x194, 0);
            m.w32(a0 + 0x198, 0);
            m.w32(a0 + 0x190, 0);
        }
        0x15 => {
            let s = m.call(0x0020_5458, &[a0, 0x9c], &[]).0;
            if s != 0 && matches!(m.r32(s + 0x30), 1..=4 | 0xc | 0x10..=0x12) {
                m.call(0x0017_7ba0, &[s], &[]);
            }
        }
        0x22 | 0x25 => {
            m.call(0x0020_8700, &[a0, 0x38], &[]);
            m.call(0x0020_6db0, &[a0], &[]);
        }
        _ => {}
    }
}

/// FUN_0020bb58(p, f12): +0x54 ? (+0x50 = f, +0x10 = 1-f) : (+0x50 = 0, +0x10 = 1); FUN_001384c0(+4);
/// (*p)+0x190 = -FUN_0020bad8(p) * (FUN_0020bb48(p))+0x40.
pub fn ee_sub_20bb58<M: QMem + ?Sized>(m: &mut M, a0: u32, f: f32) {
    let w = if m.r32(a0 + 0x54) != 0 {
        m.wf(a0 + 0x50, f);
        1.0 - f
    } else {
        m.w32(a0 + 0x50, 0);
        1.0
    };
    m.wf(a0 + 0x10, w);
    let p = m.r32(a0 + 4);
    m.call(0x0013_84c0, &[p], &[]);
    let s0 = m.r32(a0);
    let b = m.call(0x0020_bb48, &[a0], &[]).0;
    let k = m.rf(b + 0x40);
    let v = m.call(0x0020_bad8, &[a0], &[]).1;
    for i in 0..4 {
        m.wf(s0 + 0x190 + 4 * i as u32, (v[i] * -1.0) * k);
    }
}

/// FUN_00188608(o): post-load: FUN_001400b8, FUN_001a64e8(o, 0x12, 0x2bf1b0), find-children (FUN_001f4c58) into
/// +0x10b8 / +0x19c0..+0x19e0, sub-inits, channel lookups (FUN_0012dbb0), FUN_0012dc18 / FUN_00230100, poses
/// (FUN_00137a30), FUN_001cc5e8, FUN_0018c250(o, 0, -1), FUN_001a6ca8(o, 1), optional FUN_001f0380 -> +0x1d28.
pub fn ee_sub_188608<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0014_00b8, &[a0], &[]);
    m.call(0x001a_64e8, &[a0, 0x12, 0x002b_f1b0], &[]);
    for (k, n, off) in [
        (0x138u32, 1u32, 0x10b8u32),
        (0x18, 6, 0x19e0),
        (0x19, 6, 0x19dc),
        (0x13, 1, 0x19d8),
        (0x12, 1, 0x19d4),
        (0x1c, 6, 0x19d0),
        (0x1d, 6, 0x19cc),
        (0xd, 1, 0x19c8),
        (0xc, 1, 0x19c4),
        (0xc, 6, 0x19c0),
    ] {
        let r = m.call(0x001f_4c58, &[a0, k, n], &[]).0;
        m.w32(a0 + off, r);
    }
    m.call(0x0020_b868, &[a0 + 0x1320], &[]);
    m.call(0x0020_b868, &[a0 + 0x1c30], &[]);
    m.call(0x001a_0300, &[a0, a0 + 0x1ac0, a0 + 0x1bc0], &[]);
    m.call(0x0018_f908, &[a0, a0 + 0x1970, a0 + 0x1994], &[]);
    m.call(0x001a_0300, &[a0, a0 + 0x1e10, a0 + 0x1f10], &[]);
    for (src, k, dst) in [
        (0x10b8u32, 5u32, 0x19e4u32),
        (0x19c8, 5, 0x19e8),
        (0x19c4, 5, 0x19ec),
        (0x19d8, 7, 0x19f0),
        (0x19d4, 7, 0x19f4),
    ] {
        let x = m.r32(a0 + src);
        let r = m.call(0x0012_dbb0, &[a0, x, k], &[]).0;
        m.w32(a0 + dst, r);
    }
    let o2 = m.r32(a0 + 0x19a0);
    let x = m.r32(o2 + 0x58);
    let r = m.call(0x0012_dbb0, &[o2, x, 5], &[]).0;
    m.w32(a0 + 0x19fc, r);
    let x = m.r32(a0 + 0x19c0);
    let r = m.call(0x0012_dbb0, &[a0, x, 5], &[]).0;
    m.w32(a0 + 0x19f8, r);
    let x = m.r32(a0 + 0x19c0);
    let q = m.rv(0x002a_2dd0);
    let r = m.call(0x0012_dc18, &[a0, x, 5, 5, 2], &[q]).0;
    m.w32(a0 + 0x1a00, r);
    let x = m.r32(a0 + 0x19c0);
    m.call(0x0023_0100, &[a0, x, r], &[]);
    m.call(0x0014_26e8, &[a0, 0x17], &[]);
    m.call(0x0018_b348, &[a0], &[]);
    for (n, a, b) in [
        (2u32, 0x1a04u32, 0x1a1cu32),
        (2, 0x1a20, 0x1a38),
        (2, 0x1a3c, 0x1a54),
        (2, 0x1a58, 0x1a70),
    ] {
        m.call(0x0013_7a30, &[a0, 0xc, n, a0 + a, 0, 0, 0, a0 + b], &[]);
    }
    m.call(0x001a_0300, &[a0, a0 + 0x13d0, a0 + 0x14d0], &[]);
    m.call(0x0013_7a30, &[a0, 0xc, 4, a0 + 0x1f60, 0, 0, 0, a0 + 0x1f90], &[]);
    m.call(0x0013_7a30, &[a0, 0xc, 3, a0 + 0x2014, 0, 0, 0, a0 + 0x2038], &[]);
    let w = m.r32(a0 + 0x19a4);
    m.w32(a0 + 0x1804, w);
    m.call(0x001c_c5e8, &[a0 + 0x17d0, a0, a0 + 0x18e0], &[]);
    m.call(0x0018_c250, &[a0, 0, u32::MAX], &[]);
    m.call(0x001a_6ca8, &[a0, 1], &[]);
    let p = m.r32(a0 + 0x1d24);
    if p != 0 {
        let x = m.r32(a0 + 0x1d20);
        let r = m.call(0x001f_0380, &[p, x, u32::MAX, 1], &[]).0;
        m.w32(a0 + 0x1d28, r);
    }
}

/// FUN_0018d760(o, k) -> bool: can the player use action k in the current +0x18c8 mode.
pub fn ee_sub_18d760<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    if m.call(0x0014_43d8, &[a0, a1], &[]).0 != 0 {
        return 0;
    }
    let k = a1 as i32;
    let mode = m.r32(a0 + 0x18c8) as i32;
    if k == 0x1d || k == 0x2a {
        if mode == 4 {
            return 0;
        }
        let t = m.rf(0x002d_deec);
        if t == 1.0 {
            return 1;
        }
        let f = m.rf(if k == 0x1d { 0x002b_f3ec } else { 0x002b_f3f0 });
        return (t == f) as u32;
    }
    let ok = match mode {
        2 => m.call(0x0019_c8d0, &[a0 + 0x2c0, a1], &[]).0 != 0,
        3 => {
            if k != 3 {
                return 0;
            }
            true
        }
        0 | 1 | 5 | 10 | 0xc => true,
        7 => m.call(0x0019_4558, &[a0 + 0x4c0, a1], &[]).0 != 0,
        8 => m.call(0x0019_eef0, &[a0 + 0xf50, a1], &[]).0 != 0,
        0x17 => {
            if m.r32(a0 + 0x11b4).wrapping_sub(4) < 3 {
                if k < 0x27 {
                    if k < 0x25 && k != 0x1a {
                        return 0;
                    }
                } else if k != 0x29 {
                    return 0;
                }
                let p = m.r32(a0 + 0x11c0);
                return (m.r32(p + 0x15c) as i32 == k) as u32;
            }
            return 0;
        }
        _ => return 0,
    };
    if !ok {
        return 0;
    }
    let r = match k {
        1 | 2 | 0x19 | 0x1e | 0x1f | 0x21 | 0x22 | 0x23 | 0x27 => true,
        3 => {
            m.r32(a0 + 0x12f0) != 0
                && (m.r32(0x002d_fbc0) & 2) != 0
                && m.call(0x0014_fd50, &[0x002d_e2e0], &[]).0 == 0
        }
        0x17 => {
            let p = m.r32(a0 + 0x2010);
            p != 0 && m.call(0x001c_b2c0, &[p], &[]).0 != 0
        }
        0x18 => mode == 5,
        0x1a | 0x25 | 0x26 | 0x29 => {
            let e = m.call(0x0018_d6f8, &[a0, a1], &[]).0;
            let x = m.r32(e + 8);
            if x == 0 {
                let y = m.r32(e + 4);
                y != 0 && m.call(0x001c_ade0, &[y, 2], &[]).0 != 0
            } else {
                m.call(0x0022_4590, &[x], &[]).0 != 0
            }
        }
        0x1b => mode == 2,
        0x28 => {
            if m.r32(a0 + 0x1d30) != 0 {
                return 0;
            }
            if m.r32(a0 + 0x10ec) == 4 || m.r32(a0 + 0x18d8) != u32::MAX {
                false
            } else {
                matches!(mode, 0 | 1 | 10 | 0xc)
            }
        }
        _ => false,
    };
    r as u32
}

/// FUN_00172018(s): guard alert-state machine on +0x30 (FUN_00172320 transitions), loops until the state settles
/// or FUN_001ef0a8(s) says stop; FUN_001eef98(s) first.
pub fn ee_sub_172018<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001e_ef98, &[a0], &[]);
    loop {
        if m.call(0x001e_f0a8, &[a0], &[]).0 != 0 {
            return;
        }
        let st = m.r32(a0 + 0x30);
        let set = |m: &mut M, k: u32| {
            m.call(0x0017_2320, &[a0, k], &[]);
        };
        let clr = |m: &mut M, k: u32| {
            m.call(0x0017_2320, &[a0, k], &[]);
            m.w32(a0 + 0x48, 0);
        };
        let p10 = m.r32(a0 + 0x10);
        let p18 = m.r32(a0 + 0x18);
        match st {
            0 => {
                if m.call(0x0017_4fb8, &[p18], &[]).0 != 0 {
                    set(m, 1);
                }
            }
            1 => {
                if m.call(0x0020_9098, &[p10], &[]).0 != 0 {
                    set(m, 5);
                } else if m.r32(a0 + 0x48) == 0 {
                    if m.call(0x0017_2658, &[a0], &[]).0 != 0
                        || m.call(0x0017_dcb8, &[p18], &[]).0 != 0
                        || m.call(0x0017_dcf8, &[p18], &[]).0 != 0
                    {
                        set(m, 2);
                    }
                } else if m.call(0x0017_2658, &[a0], &[]).0 != 0 {
                    clr(m, 2);
                } else {
                    let v = m.call(0x0017_2760, &[a0], &[]).1;
                    let sp = stack_push(m, 0x20);
                    m.call(0x0025_62a8, &[sp], &[]);
                    for i in 0..4 {
                        m.wf(sp + 0x10 + 4 * i as u32, v[i]);
                    }
                    m.call(0x0017_dd38, &[p18, sp], &[]);
                    stack_pop(m, 0x20);
                    m.w32(a0 + 0x48, 0);
                }
            }
            2 => {
                if m.call(0x0020_9098, &[p10], &[]).0 != 0 {
                    set(m, 5);
                } else if m.r32(a0 + 0x48) == 0 {
                    if m.rf(a0 + 0x38) < m.rf(T_NOW) || m.call(0x0017_4fb8, &[p18], &[]).0 != 0 {
                        set(m, 4);
                    } else if m.rf(a0 + 0x3c) < m.rf(T_NOW) {
                        set(m, 3);
                    } else {
                        let v = m.call(0x0017_2760, &[a0], &[]).1;
                        let w = m.call(0x001a_62e8, &[p10], &[]).1;
                        let (dx, dy) = (v[0] - w[0], v[1] - w[1]);
                        let ang = m.call(0x0024_58f0, &[], &[f4(dy), f4(dx)]).1[0];
                        m.call(0x0020_9818, &[p10 + 0xfd0], &[f4(ang)]);
                    }
                } else if m.call(0x0017_2658, &[a0], &[]).0 != 0 {
                    m.call(0x0017_2560, &[a0], &[]);
                    m.w32(a0 + 0x48, 0);
                } else {
                    clr(m, 1);
                }
            }
            3 => {
                if m.call(0x0020_9098, &[p10], &[]).0 != 0 {
                    set(m, 5);
                } else if m.r32(a0 + 0x48) == 0 {
                    if m.rf(a0 + 0x38) < m.rf(T_NOW) {
                        set(m, 4);
                    } else {
                        m.call(0x0017_2f10, &[a0], &[]);
                    }
                } else if m.call(0x0017_2658, &[a0], &[]).0 != 0 {
                    clr(m, 2);
                } else {
                    clr(m, 1);
                }
            }
            4 => {
                if m.r32(a0 + 0x48) == 0 {
                    if m.call(0x0017_4fb8, &[p18], &[]).0 != 0 {
                        set(m, u32::MAX);
                    }
                } else if m.call(0x0017_2658, &[a0], &[]).0 != 0 {
                    clr(m, 2);
                } else {
                    clr(m, 1);
                }
            }
            5 => {
                if m.call(0x0020_9098, &[p10], &[]).0 == 0 {
                    set(m, 1);
                }
            }
            _ => {}
        }
        if m.r32(a0 + 0x30) == st {
            return;
        }
    }
}

/// FUN_002082d0(o) -> best target: scans the (+0xf64) candidate list (+0x100 entries of 0x10, count +0xfc) for the
/// nearest eligible entity (FUN_002089f0 rejects, FUN_00208b38 forces, FUN_002087e8 priority, +0x38c / class 0x38
/// distance remaps); skipped when +0x390 is set and flagged.
pub fn ee_sub_2082d0<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let c = m.r32(a0 + 0x390);
    if c != 0 && (m.r32(c + 0x2c) >> 28) & 1 != 0 {
        return c;
    }
    let mut best = m.rf(0x002a_886c);
    let me = m.call(0x001a_62e8, &[a0], &[]).1;
    let mut res = 0u32;
    let mut i = 0u32;
    loop {
        let l = m.r32(a0 + 0xf64);
        if i as i32 >= m.r32(l + 0xfc) as i32 {
            break;
        }
        let e = m.r32(l + 0x100 + 0x10 * i);
        let flag = m.r32(l + 0x104 + 0x10 * i);
        i += 1;
        if e == 0 {
            continue;
        }
        let st = m.r32(a0 + 0xfbc);
        if flag == 0 {
            if !(st == 0x22 || st == 0x25) || m.call(0x0013_2b10, &[e, 0x38], &[]).0 == 0 {
                continue;
            }
        }
        if m.call(0x0020_89f0, &[a0, e], &[]).0 != 0 {
            continue;
        }
        if m.call(0x0020_8b38, &[a0, e], &[]).0 != 0 {
            return e;
        }
        let p = m.call(0x001a_62e8, &[e], &[]).1;
        let (dx, dy, dz) = (me[0] - p[0], me[1] - p[1], me[2] - p[2]);
        let mut d = (dx * dx + dy * dy + dz * dz).sqrt();
        if e == m.r32(a0 + 0x38c) {
            d = m.rf(0x002c_3d70) + d * (m.rf(0x002c_3d74) + d * m.rf(0x002c_3d78));
        }
        let st = m.r32(a0 + 0xfbc);
        if (st == 0x22 || st == 0x25) && m.call(0x0013_2b10, &[e, 0x38], &[]).0 != 0 {
            d = m.rf(0x002c_3d80) + d * (m.rf(0x002c_3d84) + d * m.rf(0x002c_3d88));
        }
        let pe = m.call(0x0020_87e8, &[a0, e], &[]).0 != 0;
        if !pe {
            if res != 0 && m.call(0x0020_87e8, &[a0, res], &[]).0 != 0 {
                continue;
            }
        } else if res != 0 && m.call(0x0020_87e8, &[a0, res], &[]).0 == 0 {
            best = d;
            res = e;
            continue;
        }
        if d < best {
            best = d;
            res = e;
        }
    }
    res
}

/// FUN_002039c0(o): FUN_001c3dc8(o); +0x240 = FUN_001a6f00(+0x10, o, q2a2dd0); FUN_001f8300(o).
pub fn ee_sub_2039c0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001c_3dc8, &[a0], &[]);
    let p = m.r32(a0 + 0x10);
    let q = m.rv(0x002a_2dd0);
    let v = m.call(0x001a_6f00, &[p, a0], &[q]).1;
    for i in 0..4 {
        m.wf(a0 + 0x240 + 4 * i as u32, v[i]);
    }
    m.call(0x001f_8300, &[a0], &[]);
}

/// FUN_00203948(o): FUN_001c3d70(o); +0x264 = -1, +0x260 = +0x200 = 1.0; FUN_00204d48(o, 1.0);
/// FUN_00204660(o, 0.0); +0x220 = 30.0, +0x214 = -50.0; FUN_001eec60(o + 0x26c, o).
pub fn ee_sub_203948<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001c_3d70, &[a0], &[]);
    m.w32(a0 + 0x264, u32::MAX);
    m.wf(a0 + 0x260, 1.0);
    m.wf(a0 + 0x200, 1.0);
    m.call(0x0020_4d48, &[a0], &[f4(1.0)]);
    m.call(0x0020_4660, &[a0], &[f4(0.0)]);
    m.wf(a0 + 0x220, 30.0);
    m.wf(a0 + 0x214, -50.0);
    m.call(0x001e_ec60, &[a0 + 0x26c, a0], &[]);
}

/// FUN_001e6ed8(o): +0xe0 ? FUN_001e6db8(o) : FUN_001e6a30(o).
pub fn ee_sub_1e6ed8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let f = if m.r32(a0 + 0xe0) == 0 { 0x001e_6a30 } else { 0x001e_6db8 };
    m.call(f, &[a0], &[]);
}

/// FUN_001e1db8(o, k): k == -1 -> 0; else FUN_0015e938(0x3dd490, 4, k, +0x18), unwrapped via FUN_001cb2e0 if class 6.
pub fn ee_sub_1e1db8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    if a1 == u32::MAX {
        return 0;
    }
    let x = m.r32(a0 + 0x18);
    let r = m.call(0x0015_e938, &[0x003d_d490, 4, a1, x], &[]).0;
    if r != 0 && m.call(0x0013_2b10, &[r, 6], &[]).0 != 0 {
        return m.call(0x001c_b2e0, &[r], &[]).0;
    }
    r
}

/// FUN_001df3a0(o): FUN_001f36e0(o); FUN_001df3d0(o).
pub fn ee_sub_1df3a0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001f_36e0, &[a0], &[]);
    m.call(0x001d_f3d0, &[a0], &[]);
}

/// FUN_001f2b20(o): FUN_001f2870(o, e) for each of +0xe0 0xb0-byte +0xe4 entries without flag bit 50 of +0x10.
pub fn ee_sub_1f2b20<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let mut i = 0u32;
    while (i as i32) < m.r32(a0 + 0xe0) as i32 {
        let e = m.r32(a0 + 0xe4) + 0xb0 * i;
        if m.r32(e + 0x14) & (1 << 18) == 0 {
            m.call(0x001f_2870, &[a0, e], &[]);
        }
        i += 1;
    }
}

/// FUN_002128e8(o, k): pick anim set: +0x70 = k (or 0x314/0x310 by FUN_002142b0 probes), +0x74/+0x78 optional.
pub fn ee_sub_2128e8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if m.r32(a0 + 0x58) == 0 {
        m.call(0x0021_2a70, &[a0, 1], &[]);
    }
    m.call(0x0021_2a50, &[a0, 2], &[]);
    m.w32(a0 + 0x70, a1);
    m.w32(a0 + 0x74, u32::MAX);
    m.w32(a0 + 0x78, u32::MAX);
    if a1 == u32::MAX {
        m.w32(a0 + 0x70, 0x314);
        for (id, off) in [(0x30fu32, 0x74u32), (0x310, 0x70), (0x311, 0x78)] {
            let x = m.r32(a0 + 0x84);
            if m.call(0x0021_42b0, &[a0, x, id], &[]).0 != 0 {
                m.w32(a0 + off, id);
            }
        }
        if m.r32(a0 + 0x58) == 1 {
            vslot(m, a0, 15, &[], &[]);
        }
    } else {
        m.call(0x0021_2a70, &[a0, 1], &[]);
        m.call(0x0021_2a70, &[a0, 2], &[]);
    }
}

/// FUN_00206eb0(o, k) -> bool: may actor substate k be entered.
pub fn ee_sub_206eb0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    if a1 == u32::MAX {
        return 1;
    }
    if m.call(0x0020_9a68, &[a0, 7], &[]).0 != 0 {
        return 0;
    }
    let t = m.call(0x0020_5458, &[a0, 0xa9], &[]).0;
    if t != 0 && m.r32(t + 0x4c) != 0 {
        return 0;
    }
    let isa = |m: &mut M, c: u32| m.call(0x0013_2b10, &[a0, c], &[]).0 != 0;
    let c = match a1 {
        8 | 0xd | 0xe => return 1,
        0xb => {
            for c in [0x3b, 0x3d, 0x40, 0x3e, 0x41, 0x3f] {
                if isa(m, c) {
                    return 0;
                }
            }
            0x3c
        }
        0x15 => 0x3e,
        0x22 | 0x25 => return (m.call(0x0020_6d48, &[a0], &[]).0 == 0) as u32,
        _ => return 0,
    };
    (!isa(m, c) && !isa(m, 0x42) && !isa(m, 0x43)) as u32
}

/// FUN_00211d38(o): unless FUN_002121c8(o): FUN_00212c50(o); in mode 2 after 0.5s with no blocking UI and
/// FUN_00187d80(0x2dfc00, 0xf0): FUN_0020f030(0x3dd490, 0), optional FUN_00162df8 count, slot 49; +0x110 = 1.
pub fn ee_sub_211d38<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    if m.call(0x0021_21c8, &[a0], &[]).0 != 0 {
        return;
    }
    m.call(0x0021_2c50, &[a0], &[]);
    if m.r32(a0 + 0x58) == 2
        && 0.5 < m.rf(0x002d_df0c) - m.rf(a0 + 0x5c)
        && m.call(0x001d_9880, &[0x002e_6550], &[]).0 == 0
        && m.call(0x001d_9880, &[0x003d_a4f0], &[]).0 == 0
        && m.call(0x0018_7d80, &[0x002d_fc00, 0xf0], &[]).0 != 0
    {
        m.call(0x0020_f030, &[0x003d_d490, 0], &[]);
        if m.r32(a0 + 0x114) == 0 {
            let p = m.r32(0x003e_0f9c);
            if p != 0 {
                let n = m.r32(0x002d_d318).wrapping_add(1);
                m.w32(0x002d_d318, n);
                m.call(0x0016_2df8, &[p], &[]);
            }
        }
        vslot(m, a0, 49, &[], &[]);
    }
    m.w32(a0 + 0x110, 1);
}

/// FUN_001dfd58(o, x): start once: +0x6c = x, +0x74 = 1, +0x78 = 0, slot 2 msg 5, then resolve/advance.
pub fn ee_sub_1dfd58<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if m.r32(a0 + 0x74) != 0 {
        return;
    }
    m.w32(a0 + 0x6c, a1);
    m.w32(a0 + 0x74, 1);
    m.w32(a0 + 0x78, 0);
    vslot(m, a0, 2, &[5, 0, 0, 0, 0], &[]);
    if m.r32(a0 + 0x74) == 0 {
        return;
    }
    if m.r32(a0 + 0x7c) != 0 {
        m.call(0x001e_02c0, &[a0], &[]);
        m.w32(a0 + 0x7c, 0);
    }
    let x = m.r32(a0 + 0x54);
    let r = m.call(0x001e_5ae0, &[a0, x, 0], &[]).0;
    m.call(0x001e_1c40, &[r], &[]);
    if m.r32(a0 + 0x74) == 0 {
        return;
    }
    if m.call(0x0011_a3b0, &[a0 + 0x5c], &[]).0 != 0 {
        m.call(0x001d_feb8, &[a0], &[]);
        return;
    }
    let p = m.r32(a0 + 0x40);
    let w = if p & 1 == 0 { m.r32(p) } else { p };
    if w & 2 != 0 {
        return;
    }
    m.call(0x001a_6ca8, &[a0, 1], &[]);
    let p = m.r32(a0 + 0x40);
    if p & 1 != 0 {
        m.w32(a0 + 0x40, p | 2);
    } else {
        let w = m.r32(p) | 2;
        m.w32(p, w);
    }
    m.w32(a0 + 0x78, 1);
}

/// FUN_00212c50(o): FUN_001a5e28(o); next mode from +0x58/+0x64 and anim-done probes (FUN_002121e8) ->
/// FUN_00212a70(o, n); with +0x7c != -1: +0x88 += dt, FUN_00218220(+0x84, dt).
pub fn ee_sub_212c50<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    const T: u32 = 0x002d_df0c;
    m.call(0x001a_5e28, &[a0], &[]);
    let s = m.r32(a0 + 0x58) as i32;
    let want = m.r32(a0 + 0x64) as i32;
    let pick = |x: u32, a: i32, b: i32| if x == u32::MAX { a } else { b };
    let n = if s == 1 {
        if want != 2 {
            s
        } else {
            pick(m.r32(a0 + 0x74), 2, 3)
        }
    } else if m.rf(a0 + 0x68) == m.rf(T) {
        s
    } else if s == 3 {
        let x = m.r32(a0 + 0x74);
        if m.call(0x0021_21e8, &[a0, x], &[]).0 != 0 {
            s
        } else if want == 1 {
            pick(m.r32(a0 + 0x78), 1, 4)
        } else if want == 2 {
            2
        } else {
            s
        }
    } else if s < 4 {
        if s != 2 {
            s
        } else {
            let x = m.r32(a0 + 0x7c);
            if m.call(0x0021_21e8, &[a0, x], &[]).0 == 0 {
                m.call(0x0021_2a50, &[a0, 1], &[]);
            }
            let f = m.rf(a0 + 0x8c);
            if 0.0 < f && f < m.rf(T) - m.rf(a0 + 0x5c) {
                m.call(0x0021_2a50, &[a0, 1], &[]);
            }
            if m.r32(a0 + 0x64) != 1 {
                s
            } else {
                pick(m.r32(a0 + 0x78), 1, 4)
            }
        }
    } else {
        let x = m.r32(a0 + 0x78);
        if s != 4 || m.call(0x0021_21e8, &[a0, x], &[]).0 != 0 {
            s
        } else if want == 1 {
            1
        } else if want == 2 {
            pick(m.r32(a0 + 0x74), 2, 3)
        } else {
            s
        }
    };
    m.call(0x0021_2a70, &[a0, n as u32], &[]);
    if m.r32(a0 + 0x7c) != u32::MAX {
        let dt = m.rf(0x002d_df10);
        let v = m.rf(a0 + 0x88) + dt;
        m.wf(a0 + 0x88, v);
        let p = m.r32(a0 + 0x84);
        m.call(0x0021_8220, &[p], &[f4(dt)]);
    }
}
