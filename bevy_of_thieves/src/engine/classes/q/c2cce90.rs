//! WP-Q5: class C2cce90 (vtable 0x2cce90) queue package: a brain behaviour that walks its owner along a
//! patrol track toward the best view of the player (+0x10 owner, +0x18 brain node, +0x30 state -1..4,
//! +0x34 state time, +0x38 track record {+0x50 dir, +0x54 n, +0x58 0xc-byte stops (+8 obj), +0x5c track
//! {+0xf0 f32 params, +0xf4 quad points}}, +0x3c current stop, +0x40 best stop, +0x44 chosen hide spot,
//! +0x48 time the player last moved); ported 1:1 against `QMem`.
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};
use crate::engine::classes::q::q5util::{dist_xy, f4, pos};
use std::f32::consts::{FRAC_PI_2, PI};

pub const CLOCK: u32 = 0x002d_ded0;
/// &DAT_003dd490 world, &DAT_003dd040
pub const WORLD: u32 = 0x003d_d490;
/// FUN_001c4840(world): the player
fn player<M: QMem + ?Sized>(m: &mut M) -> u32 {
    m.call(0x001c_4840, &[WORLD], &[]).0
}

/// C2cce90_v5 (slot 5): class descriptor
pub fn ee_sub_114570() -> u32 {
    0x002b_b920
}

/// C2cce90_v6 (slot 6): parent init, register with 0x3dd040 (FUN_001fee90), owner flag 0x24 on,
/// stops -1, state -1
pub fn ee_sub_14a1b0<M: QMem + ?Sized>(m: &mut M, s: u32) {
    m.call(0x001e_ef68, &[s], &[]);
    m.call(0x001f_ee90, &[0x003d_d040], &[]);
    let o = m.r32(s + 0x10);
    m.call(0x0020_9890, &[o, 0x24], &[]);
    m.w32(s + 0x3c, 0xffff_ffff);
    m.w32(s + 0x30, 0xffff_ffff);
    m.w32(s + 0x40, 0xffff_ffff);
}

/// C2cce90_v7 (slot 7): activate: state 0, player-moved time = now, owner +0x4a4 = 0
pub fn ee_sub_14a200<M: QMem + ?Sized>(m: &mut M, s: u32) {
    m.call(0x001e_ef70, &[s], &[]);
    ee_sub_14ae38(m, s, 0);
    let c = m.r32(CLOCK);
    m.w32(s + 0x48, c);
    let o = m.r32(s + 0x10);
    m.w32(o + 0x4a4, 0);
}

/// C2cce90_v8 (slot 8): deactivate: state -1, owner flag 0x24 off, unregister, owner +0x4a4 = 1
pub fn ee_sub_14a248<M: QMem + ?Sized>(m: &mut M, s: u32) {
    m.call(0x001e_ef78, &[s], &[]);
    ee_sub_14ae38(m, s, 0xffff_ffff);
    let o = m.r32(s + 0x10);
    m.call(0x0020_9970, &[o, 0x24], &[]);
    m.call(0x001f_eea0, &[0x003d_d040], &[]);
    let o = m.r32(s + 0x10);
    m.w32(o + 0x4a4, 1);
}

/// C2cce90_v9 (slot 9): aim the head (FUN_0014a4d0) then settle the state machine: 0 look -> 2 (player
/// hidden, FUN_0014a478) / -> 3 hide spot at the best stop / step toward it (FUN_0014ad70); 1 walk -> 0
/// when arrived (FUN_00178d00) else keep facing the player; 2 / 3 -> 0 when their node finishes; any
/// state -> 4 while the owner is alerted (FUN_00209098), 4 -> 0 after.
pub fn ee_sub_14a2a0<M: QMem + ?Sized>(m: &mut M, s: u32) {
    m.call(0x001e_ef98, &[s], &[]);
    ee_sub_14a4d0(m, s);
    loop {
        if m.call(0x001e_f0a8, &[s], &[]).0 != 0 {
            return;
        }
        let st = m.r32(s + 0x30);
        let o = m.r32(s + 0x10);
        let alert = |m: &mut M| m.call(0x0020_9098, &[o], &[]).0 != 0;
        match st as i32 {
            0 => {
                if alert(m) {
                    ee_sub_14ae38(m, s, 4);
                } else if ee_sub_14a478(m, s) {
                    ee_sub_14ae38(m, s, 2);
                } else {
                    let b = ee_sub_14aa30(m, s);
                    m.w32(s + 0x40, b as u32);
                    if b as u32 == m.r32(s + 0x3c) {
                        let p = player(m);
                        let h = m.call(0x0017_58a0, &[o, p, 0], &[]).0;
                        m.w32(s + 0x44, h);
                        if h != 0 {
                            ee_sub_14ae38(m, s, 3);
                        }
                    } else {
                        ee_sub_14ad70(m, s);
                    }
                }
            }
            1 => {
                if alert(m) {
                    ee_sub_14ae38(m, s, 4);
                } else if ee_sub_14a478(m, s) {
                    ee_sub_14ae38(m, s, 2);
                } else {
                    let n = m.r32(s + 0x18);
                    if m.call(0x0017_8d00, &[n], &[]).0 == 0 {
                        let p = player(m);
                        m.call(0x0017_58a0, &[o, p, 0], &[]);
                    } else {
                        ee_sub_14ae38(m, s, 0);
                    }
                }
            }
            2 | 3 => {
                if alert(m) {
                    ee_sub_14ae38(m, s, 4);
                } else {
                    let n = m.r32(s + 0x18);
                    let f = if st == 2 { 0x0017_4fb8 } else { 0x0017_5748 };
                    if m.call(f, &[n], &[]).0 != 0 {
                        ee_sub_14ae38(m, s, 0);
                    }
                }
            }
            4 => {
                if !alert(m) {
                    ee_sub_14ae38(m, s, 0);
                }
            }
            _ => {}
        }
        if m.r32(s + 0x30) == st {
            return;
        }
    }
}

/// FUN_0014a478: owner not engaged (+0x208 == 0) and FUN_00173610(100, -250, owner pos, 1) finds nothing
pub fn ee_sub_14a478<M: QMem + ?Sized>(m: &mut M, s: u32) -> bool {
    let o = m.r32(s + 0x10);
    if m.r32(o + 0x208) != 0 {
        return false;
    }
    let p = pos(m, o);
    let buf = stack_push(m, 0x10);
    m.wv(buf, p);
    let r = m.call(0x0017_3610, &[buf, 1, 0], &[f4(100.0), f4(-250.0)]).0;
    stack_pop(m, 0x10);
    r == 0
}

/// FUN_0014a4d0: head look target (owner +0x1010 slot 28, FUN_00208540). Player idle longer than a random
/// 4..6 s (and no FUN_0020f0b0 alarm): look at the player (kind-0x34 player in mode 0x11: the highest of
/// its +0x258 points). Else glance at a random point (FUN_0011e208) near a 0x3e1450 lookout (or one of
/// its 0x12 children with +0xe8 & 0x17e) within 600 of the player, accepted when closer (xy) than the
/// player or within 10 deg of the owner->player bearing (FUN_0011e4b0); else ahead of a moving player
/// (pos - 200 * dir) or beside a still one (FUN_0011e1a0(bearing +-90deg, 200, -75)).
pub fn ee_sub_14a4d0<M: QMem + ?Sized>(m: &mut M, s: u32) {
    let p = player(m);
    let pp = pos(m, p);
    let o = m.r32(s + 0x10);
    let op = pos(m, o);
    let d: V4 = [pp[0] - op[0], pp[1] - op[1], pp[2] - op[2], pp[3] - op[3]];
    let vel = vslot(m, p, 32, &[], &[]).1;
    let sp3 = (vel[0] * vel[0] + vel[1] * vel[1] + vel[2] * vel[2]).sqrt();
    if m.rf(0x002b_d460) < sp3 {
        let c = m.r32(CLOCK);
        m.w32(s + 0x48, c);
    }
    let idle = m.rf(CLOCK) - m.rf(s + 0x48);
    let (lo, hi) = (m.rf(0x002b_d468), m.rf(0x002b_d46c));
    let r = m.call(0x0011_d448, &[], &[f4(lo), f4(hi)]).1[0];
    let target: V4;
    if !(r < idle) || m.call(0x0020_f0b0, &[WORLD], &[]).0 != 0 {
        let mut hit = None;
        let mut l = m.r32(0x003e_1450);
        if l != 0 {
            let hd = dist_xy(d, [0.0; 4]);
            while l != 0 {
                if m.call(0x001b_be48, &[l], &[]).0 == 0 {
                    let mut q = pos(m, l);
                    let x = m.r32(l + 0xec);
                    let mut rad = m.rf(0x002b_d478);
                    if x != 0 {
                        rad = m.rf(x + 0x30);
                        q = m.rv(x + 0x20);
                    }
                    let buf = stack_push(m, 0x20);
                    let n = m.call(0x0015_e9c8, &[WORLD, 1, 0x12, l, 8, buf], &[]).0 as i32;
                    let mut k = 0u32;
                    for i in 0..n.max(0) as u32 {
                        let c = m.r32(buf + 4 * i);
                        if m.r32(c + 0xe8) & 0x17e != 0 {
                            m.w32(buf + 4 * k, c);
                            k += 1;
                        }
                    }
                    if k != 0 {
                        let i = m.call(0x0011_d3f8, &[0, k - 1], &[]).0;
                        let c = m.r32(buf + 4 * i);
                        q = pos(m, c);
                        rad = m.rf(c + 0xec);
                    }
                    stack_pop(m, 0x20);
                    let lim = m.rf(0x002b_d478);
                    let e: V4 = [q[0] - pp[0], q[1] - pp[1], q[2] - pp[2], 0.0];
                    if e[0] * e[0] + e[1] * e[1] + e[2] * e[2] <= lim * lim {
                        let a = m.call(0x0011_d448, &[], &[f4(-PI), f4(PI)]).1[0];
                        let b = m.call(0x0011_d448, &[], &[f4(-PI), f4(PI)]).1[0];
                        let c = m.call(0x0011_d448, &[], &[f4(0.0), f4(rad)]).1[0];
                        let off = m.call(0x0011_e208, &[], &[f4(a), f4(b), f4(c)]).1;
                        let t: V4 = [q[0] + off[0], q[1] + off[1], q[2] + off[2], q[3] + off[3]];
                        let e: V4 = [t[0] - op[0], t[1] - op[1], t[2] - op[2], t[3] - op[3]];
                        if dist_xy(e, [0.0; 4]) < hd {
                            hit = Some(t);
                            break;
                        }
                        let ang = m.call(0x0011_e4b0, &[0], &[d, e]).1[0];
                        if m.rf(0x002b_d47c) < ang.abs() * 57.295_776 {
                            hit = Some(t);
                            break;
                        }
                    }
                }
                l = m.r32(l + 0x174);
            }
        }
        target = match hit {
            Some(t) => t,
            None => {
                let k = m.rf(0x002b_d470);
                if dist_xy(vel, [0.0; 4]) < m.rf(0x002b_d460) {
                    let e: V4 = [op[0] - pp[0], op[1] - pp[1], op[2] - pp[2], op[3] - pp[3]];
                    let ang = m.call(0x0024_58f0, &[], &[f4(e[1]), f4(e[0])]).1[0];
                    let pp2 = pos(m, p);
                    let a = m.call(0x0011_d448, &[], &[f4(ang - FRAC_PI_2), f4(ang + FRAC_PI_2)]).1[0];
                    let z = m.rf(0x002b_d474);
                    let off = m.call(0x0011_e1a0, &[], &[f4(a), f4(k), f4(z)]).1;
                    [pp2[0] + off[0], pp2[1] + off[1], pp2[2] + off[2], pp2[3] + off[3]]
                } else {
                    let q = 1.0 / sp3;
                    let n = [vel[0] * q, vel[1] * q, vel[2] * q, vel[3] * q];
                    [pp[0] - n[0] * k, pp[1] - n[1] * k, pp[2] - n[2] * k, 1.0 + pp[3] - n[3] * k]
                }
            }
        };
    } else {
        let mut look = None;
        if m.call(0x0013_2b10, &[p, 0x34], &[]).0 != 0 && m.r32(p + 0x2dc) == 0x11 {
            let r = m.r32(p + 600);
            let mut t = pos(m, p);
            let n = m.r32(r + 0x90) as i32;
            let a = m.r32(r + 0x94);
            for i in 0..n.max(0) as u32 {
                let q = m.rv(a + 0x10 * i);
                if t[2] < q[2] {
                    t = q;
                }
            }
            look = Some(t);
        }
        match look {
            Some(t) => target = t,
            None => {
                m.call(0x0020_8540, &[o, p], &[]);
                return;
            }
        }
    }
    let h = m.r32(o + 0x1010);
    vslot(m, h, 28, &[], &[target]);
    let o = m.r32(s + 0x10);
    let h = m.r32(o + 0x1010);
    m.call(0x0020_8540, &[o, h], &[]);
}

/// remap `x` through the 2-point ramp at `at` ({lo, hi}): (x - lo) / (hi - lo), 0 when lo == hi
fn ramp<M: QMem + ?Sized>(m: &mut M, at: u32, x: f32) -> f32 {
    let (lo, hi) = (m.rf(at), m.rf(at + 4));
    if lo == hi {
        0.0
    } else {
        (x - lo) / (hi - lo)
    }
}
/// FUN_0011d910 range test then ramp up over `up` / down over `down` (0 outside both)
fn band<M: QMem + ?Sized>(m: &mut M, up: u32, down: u32, x: f32) -> f32 {
    if m.call(0x0011_d910, &[up], &[f4(x)]).0 != 0 {
        ramp(m, up, x)
    } else if m.call(0x0011_d910, &[down], &[f4(x)]).0 != 0 {
        1.0 - ramp(m, down, x)
    } else {
        0.0
    }
}

/// FUN_0014aa30: best stop: for every stop with an object, score 0.5 * track-distance band (0x2bd480 /
/// 0x2bd488, signed by +0x50) + 0.15 * screen-depth band (0x2bd490 / 0x2bd498 of FUN_0014f308 z) +
/// 0.35 * screen-centre falloff (0x2bd4a0); highest score > -1 wins (-1 = none).
pub fn ee_sub_14aa30<M: QMem + ?Sized>(m: &mut M, s: u32) -> i32 {
    let p = player(m);
    let tr = m.r32(s + 0x38);
    let pp = pos(m, p);
    let buf = stack_push(m, 0x10);
    let track = m.r32(tr + 0x5c);
    m.call(0x001e_6f60, &[track, 0, 0, 0, 0, buf], &[pp]);
    let tpar = m.rf(buf);
    stack_pop(m, 0x10);
    let mut best = -1i32;
    let mut bf = -1.0f32;
    let mut i = 0i32;
    loop {
        let tr = m.r32(s + 0x38);
        if i >= m.r32(tr + 0x54) as i32 {
            break;
        }
        let st = m.r32(tr + 0x58) + 0xc * i as u32;
        if m.r32(st + 8) != 0 {
            let t = m.r32(tr + 0x5c);
            let pa = m.r32(t + 0xf0);
            let mut x = tpar - m.rf(pa + 4 * i as u32);
            if m.r32(tr + 0x50) == 1 {
                x = -x;
            }
            let a = if m.call(0x0011_d910, &[0x002b_d480], &[f4(x)]).0 != 0 {
                ramp(m, 0x002b_d480, x)
            } else if m.call(0x0011_d910, &[0x002b_d488], &[f4(x)]).0 != 0 {
                1.0 - ramp(m, 0x002b_d488, x)
            } else {
                0.0
            };
            let tr = m.r32(s + 0x38);
            let t = m.r32(tr + 0x5c);
            let qa = m.r32(t + 0xf4);
            let q = m.rv(qa + 0x10 * i as u32);
            let sc = m.call(0x0014_f308, &[0x002d_df40], &[q]).1;
            let b = band(m, 0x002b_d490, 0x002b_d498, sc[2]);
            let r = (sc[0] * sc[0] + sc[1] * sc[1]).sqrt();
            let c = if m.call(0x0011_d910, &[0x002b_d4a0], &[f4(r)]).0 == 0 { 0.0 } else { 1.0 - ramp(m, 0x002b_d4a0, r) };
            let f = a * 0.5 + b * 0.15 + c * 0.35;
            if bf < f {
                bf = f;
                best = i;
            }
        }
        i += 1;
    }
    best
}

/// FUN_0014ad70: step the current stop one toward the best stop and walk (stop kind +0 == 0) or wait
/// (kind 1) there
pub fn ee_sub_14ad70<M: QMem + ?Sized>(m: &mut M, s: u32) {
    let c = m.r32(s + 0x3c) as i32;
    let tr = m.r32(s + 0x38);
    let base = m.r32(tr + 0x58);
    let k = if c < m.r32(s + 0x40) as i32 {
        m.w32(s + 0x3c, (c + 1) as u32);
        m.r32(base.wrapping_add(((c + 1) * 0xc) as u32))
    } else {
        let k = m.r32(base.wrapping_add((c * 0xc) as u32));
        m.w32(s + 0x3c, (c - 1) as u32);
        k
    };
    match k {
        0 => ee_sub_14ae38(m, s, 1),
        1 => ee_sub_14ae38(m, s, 2),
        _ => {}
    }
}

/// FUN_0014ae18: object of the current stop
pub fn ee_sub_14ae18<M: QMem + ?Sized>(m: &mut M, s: u32) -> u32 {
    let c = m.r32(s + 0x3c);
    let tr = m.r32(s + 0x38);
    let base = m.r32(tr + 0x58);
    m.r32(base.wrapping_add(c.wrapping_mul(0xc)) + 8)
}

/// FUN_0014ae38: enter state `st` (exit actions of the old one, reset the node, stamp +0x34):
/// 0 behaviour 0x9e watching the player (zero velocity / spin, +0x190..+0x198 cleared); 1 walk a 2-point
/// path to the current stop (behaviour 0xa7, speed 25); 2 behaviour 0xac at the stop; 3 hide (behaviour
/// 0xa4 with the spot, +0xf04 = FLT_MAX); 4 behaviour 0x9e alerted; -1 / 1 toggle FUN_001f5720/57c0.
pub fn ee_sub_14ae38<M: QMem + ?Sized>(m: &mut M, s: u32, st: u32) {
    let now = m.rf(CLOCK);
    let old = m.r32(s + 0x30) as i32;
    if st as i32 == old {
        return;
    }
    let o = m.r32(s + 0x10);
    match old {
        0 => {
            m.call(0x0020_ac28, &[o, 0, 0], &[[0.0; 4]]);
        }
        -1 | 1 => {
            m.call(0x001f_5720, &[o, 1], &[]);
            let o = m.r32(s + 0x10);
            m.call(0x001f_57c0, &[o, 1], &[]);
        }
        3 => {
            let h = m.r32(s + 0x44);
            let (a, b) = (m.rf(h + 0x60), m.rf(h + 100));
            let r = m.call(0x0011_d448, &[], &[f4(a), f4(b)]).1[0];
            let o = m.r32(s + 0x10);
            m.wf(o + 0xf04, now + r);
            m.w32(s + 0x44, 0);
        }
        _ => {}
    }
    m.call(0x001e_f0d8, &[s + 0x18], &[]);
    m.w32(s + 0x30, st);
    let c = m.r32(CLOCK);
    m.w32(s + 0x34, c);
    let o = m.r32(s + 0x10);
    match st as i32 {
        0 => {
            let b = m.call(0x0020_5438, &[o, 0x9e], &[]).0;
            let p = player(m);
            m.w32(b + 0x38, p);
            vslot(m, b, 7, &[], &[]);
            let o = m.r32(s + 0x10);
            vslot(m, o, 34, &[], &[[0.0; 4]]);
            let o = m.r32(s + 0x10);
            vslot(m, o, 35, &[], &[[0.0; 4]]);
            let o = m.r32(s + 0x10);
            m.w32(o + 0x194, 0);
            m.w32(o + 0x198, 0);
            m.w32(o + 400, 0);
        }
        1 => {
            let mv = m.r32(o + 0x294);
            m.w32(mv + 0xe8, 2);
            let path = m.r32(mv + 0xf4);
            let op = pos(m, o);
            m.wv(path, op);
            let path = m.r32(mv + 0xf4);
            let so = m.call(0x0014_ae18, &[s], &[]).0; // jal 0x14ae18
            let sp = pos(m, so);
            m.wv(path + 0x10, sp);
            m.call(0x001e_6180, &[mv], &[]);
            let o = m.r32(s + 0x10);
            let b = m.call(0x0020_5438, &[o, 0xa7], &[]).0;
            m.call(0x0017_8d90, &[b, mv, 0], &[]);
            m.wf(b + 100, 25.0);
            m.w32(b + 0x40, 1);
            m.w32(b + 0xc0, 8);
            let p = player(m);
            m.w32(b + 200, p);
            vslot(m, b, 7, &[], &[]);
            let o = m.r32(s + 0x10);
            let c = m.r32(CLOCK);
            m.w32(o + 0xf04, c);
            m.call(0x001f_5720, &[o, 0], &[]);
            let o = m.r32(s + 0x10);
            m.call(0x001f_57c0, &[o, 0], &[]);
        }
        2 => {
            let b = m.call(0x0020_5438, &[o, 0xac], &[]).0;
            let so = m.call(0x0014_ae18, &[s], &[]).0; // jal 0x14ae18
            let sp = pos(m, so);
            m.wv(b + 0x50, sp);
            m.w32(b + 0x68, 1);
            vslot(m, b, 7, &[], &[]);
        }
        3 => {
            let b = m.call(0x0020_5438, &[o, 0xa4], &[]).0;
            let x = m.r32(o + 0x38c);
            m.w32(b + 0x30, x);
            let h = m.r32(s + 0x44);
            m.w32(b + 0x3c, h);
            vslot(m, b, 7, &[], &[]);
            let o = m.r32(s + 0x10);
            let f = m.r32(0x002a_4ec4);
            m.w32(o + 0xf04, f);
        }
        4 => {
            let b = m.call(0x0020_5438, &[o, 0x9e], &[]).0;
            let p = player(m);
            m.w32(b + 0x38, p);
            m.w32(b + 0x3c, 1);
            vslot(m, b, 7, &[], &[]);
        }
        -1 => {
            m.call(0x001f_5720, &[o, 0], &[]);
            let o = m.r32(s + 0x10);
            m.call(0x001f_57c0, &[o, 0], &[]);
        }
        _ => {}
    }
}
