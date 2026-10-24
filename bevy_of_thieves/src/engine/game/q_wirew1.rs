//! WIRE-W1: 1:1 queue ports (against `QMem`) of live-frontier natives: the pickup family (class 91 coin,
//! 93 health, base SetState, slot-13 load). Ported from the EE asm (tools/eedis.py); callees go through
//! `m.call` / `vslot` (the live dispatch routes ported ones).
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

const GS: u32 = 0x003d_d490; // game state

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

fn cp16<M: QMem + ?Sized>(m: &mut M, dst: u32, src: u32) {
    for k in (0..16).step_by(4) {
        let w = m.r32(src + k);
        m.w32(dst + k, w);
    }
}

/// FUN_001c4ef8(p): pickup load (slot 13): base load, +0xf0 flag of FUN_001f7b98, two 0x15 bindings
/// for +0x58/+0x5c, anim binding table 0x2c0d40.
pub fn ee_sub_1c4ef8<M: QMem + ?Sized>(m: &mut M, p: u32) {
    m.call(0x001f_31e0, &[p], &[]);
    let r = m.call(0x001f_7b98, &[p], &[]).0;
    m.w32(r + 0xf0, 1);
    let a = m.r32(p + 0x58);
    m.call(0x0023_03a0, &[p, a, 0x15], &[]);
    let a = m.r32(p + 0x5c);
    m.call(0x0023_03a0, &[p, a, 0x15], &[]);
    m.call(0x001a_64e8, &[p, 1, 0x002c_0d40], &[]);
}

/// FUN_001c60b8(p, s): pickup base SetState. +0x150 state, +0x154 t, +0x15c anim override, +0x160 fade,
/// +0x164 anim set, +0x168 anim handle, +0x16c linked object, +0x170/+0x180 saved xform rows, +0x190 color.
pub fn ee_sub_1c60b8<M: QMem + ?Sized>(m: &mut M, p: u32, s: i32) {
    let cur = m.r32(p + 0x150) as i32;
    if cur == s {
        return;
    }
    if s != 0 && s != 5 {
        m.call(0x001a_58d0, &[p, 0, 0], &[]);
    }
    let cur = m.r32(p + 0x150) as i32;
    if cur == -1 || cur == 2 {
        let x = m.r32(p + 0x58);
        cp16(m, p + 0x170, x + 0x30);
        cp16(m, p + 0x180, x + 0xb0);
    }
    let mut anim: i32 = -1;
    match s {
        0 => {
            anim = 0x184;
            let o = m.r32(p + 0x15c) as i32;
            if o != -1 {
                anim = o;
            }
            cp16(m, p + 0x190, 0x002a_2dd0);
        }
        2 => {
            let t = m.r32(0x002a_773c);
            m.w32(p + 0x1cc, t);
            vslot(m, p, 8, &[], &[]);
            let l = m.call(0x001c_b658, &[p], &[]).0;
            if l != 0 {
                m.call(0x001c_b4b8, &[l, p], &[]);
            }
        }
        3 | 4 => {
            anim = 0x183;
            cp16(m, p + 0x190, 0x002c_0d60);
        }
        5 => {
            anim = 0x186;
            cp16(m, p + 0x190, 0x002a_2dd0);
        }
        _ => {}
    }
    let l = m.r32(p + 0x16c);
    if l != 0 {
        let slot = if s == 0 || s == 4 || s == 5 { 7 } else { 8 };
        vslot(m, l, slot, &[], &[]);
    }
    m.w32(p + 0x150, s as u32);
    let t = m.r32(0x002d_ded0);
    m.w32(p + 0x154, t);
    let h = m.r32(p + 0x168);
    if anim == -1 {
        if h != 0 {
            m.call(0x001f_0598, &[h], &[]);
            let h = m.r32(p + 0x168);
            vslot(m, h, 1, &[], &[]);
            m.w32(p + 0x168, 0);
        }
    } else if h == 0 {
        let set = m.r32(p + 0x164);
        let h = m.call(0x001f_0380, &[set, p, anim as u32, 0], &[]).0;
        m.w32(p + 0x168, h);
        vslot(m, h, 0, &[], &[]);
        if 0.0 < m.rf(p + 0x160) {
            let h = m.r32(p + 0x168);
            m.call(0x001f_10f0, &[h], &[]);
        }
    } else {
        m.call(0x001f_0660, &[h, anim as u32], &[]);
    }
}

/// shared collect fx of 1c65f8 / 1c68f0: FUN_001ce888(GS, 9, 0) effect (+0xf0 def, +0xf4 owner, slot 0 of
/// the +0xec table with the pickup position at 1.0), then +0x40..+0x70 = pickup xform; `none` when no fx.
fn collect_fx<M: QMem + ?Sized>(m: &mut M, p: u32, def: u32, none: u32) {
    let e = m.call(0x001c_e888, &[GS, 9, 0], &[]).0;
    if e == 0 {
        m.call(none, &[p, 0], &[]);
        return;
    }
    let t = m.r32(e + 0xec);
    m.w32(e + 0xf4, p);
    m.w32(e + 0xf0, def);
    m.w32(e + 0xf8, 0);
    let adj = m.r16(t + 8) as i16 as i32;
    let pos = m.call(0x001a_62e8, &[p], &[]).0;
    let f = m.r32(t + 0xc);
    m.call(f, &[e.wrapping_add(adj as u32), pos, 0], &[f4(1.0)]);
    m.w32(e + 0x1c, p);
    let x = m.call(0x001a_6318, &[p], &[]).0;
    cp16(m, e + 0x40, x);
    cp16(m, e + 0x50, x + 0x10);
    cp16(m, e + 0x60, x + 0x20);
}

/// `+0x210` listener message {1, p} (slot 50) then own vt+0x1c (slot 2) event 5.
fn notify_collected<M: QMem + ?Sized>(m: &mut M, p: u32) {
    let l = m.r32(p + 0x210);
    if l != 0 {
        let sp = stack_push(m, 16);
        m.w32(sp, 1);
        m.w32(sp + 4, p);
        vslot(m, l, 50, &[sp], &[]);
        stack_pop(m, 16);
    }
    vslot(m, p, 2, &[5, 0, 0, 0, 0], &[]);
}

/// FUN_001c65f8(p, s): coin (class 91) SetState. 0: +0x238 = rand(+0x230, +0x234); 1: collect fx
/// (def 0x2c0da0), sound 0x29, listener + event 5, HUD coin FUN_00161868, event 0x12 -> state 2.
pub fn ee_sub_1c65f8<M: QMem + ?Sized>(m: &mut M, p: u32, s: i32) {
    if m.r32(p + 0x150) as i32 == s {
        return;
    }
    let mut s = s;
    if s == 0 {
        let lo = m.rf(p + 0x230);
        let hi = m.rf(p + 0x234);
        let r = m.call(0x0011_d448, &[], &[f4(lo), f4(hi)]).1[0];
        m.wf(p + 0x238, r);
    } else if s == 1 {
        collect_fx(m, p, 0x002c_0da0, 0x001c_67a0);
        let snd = m.call(0x001f_bb90, &[0x29], &[]).0;
        m.call(0x001f_b2f8, &[snd, 0], &[]);
        s = 2;
        notify_collected(m, p);
        m.call(0x0016_1868, &[GS], &[]);
        m.call(0x0016_19a0, &[0x002d_eb30, 0x12, p], &[]);
    }
    ee_sub_1c60b8(m, p, s);
}

/// FUN_001c68f0(p, s): health (class 93) SetState. 1: player (FUN_001c4840) slot 79 can-take test, else
/// keep state; collect fx (def 0x2c0e10), slot 80 gift (+0x23c, +0x240), listener + event 5 -> state 2.
pub fn ee_sub_1c68f0<M: QMem + ?Sized>(m: &mut M, p: u32, s: i32) {
    if m.r32(p + 0x150) as i32 == s {
        return;
    }
    let mut s = s;
    if s == 0 {
        let lo = m.rf(p + 0x230);
        let hi = m.rf(p + 0x234);
        let r = m.call(0x0011_d448, &[], &[f4(lo), f4(hi)]).1[0];
        m.wf(p + 0x238, r);
    } else if s == 1 {
        let pl = m.call(0x001c_4840, &[GS], &[]).0;
        if vslot(m, pl, 79, &[], &[]).0 == 0 {
            s = m.r32(p + 0x150) as i32;
        } else {
            collect_fx(m, p, 0x002c_0e10, 0x001c_6ab0);
            let a = m.r32(p + 0x23c);
            let b = m.r32(p + 0x240);
            vslot(m, pl, 80, &[a, b], &[]);
            s = 2;
            notify_collected(m, p);
        }
    }
    ee_sub_1c60b8(m, p, s);
}
