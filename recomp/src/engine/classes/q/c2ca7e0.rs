//! WP-Q8: C2ca7e0 (look-at / follow object: target +8, params +0x20..+0x5c, ctx +0x6c..+0x74).
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem};
use crate::engine::classes::q::q8util::*;

/// FUN_0012d210: +0x74 = 0, +0x70 = 0
pub fn ee_sub_12d210<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.w32(o + 0x74, 0);
    m.w32(o + 0x70, 0);
}
/// FUN_0012d220 (update): distance of the target (+8) to the handle (+0x70 / FUN_001c4af8(world)) selects
/// the mode (+0x68 near / +0x64 far); on a change FUN_002301d0; then writes the target's +0xc0 (angles from
/// FUN_0022fcc8 / FUN_0011e958 through two cubics clamped to +0x50..+0x5c, or the saved +0x20) and +0xd0
/// (velocity), +0x9a = 3.
/// PARTIAL: VU0 register traffic read from the decomp: velocity divide taken as 1/dt (DAT_002dded4); the
/// 4th lane of the angle vector is uninitialised stack in the original (0 here).
pub fn ee_sub_12d220<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.r32(o + 0x70) == 0 && m.r32(o + 0x6c) != u32::MAX {
        let a = [WORLD, 0x104, m.r32(o + 0x6c), m.r32(o + 4)];
        let v = m.call(0x0015_e938, &a, &[]).0;
        m.w32(o + 0x70, v);
    }
    let tgt = m.r32(o + 8);
    let pos = m.rv(tgt + 0x70);
    let mut h = m.r32(o + 0x70);
    if h == 0 {
        h = m.call(0x001c_4af8, &[WORLD], &[]).0;
    }
    let mut d = [0.0f32; 4];
    let dist;
    if h == 0 {
        dist = rof(0x002a_45a8);
    } else {
        let p = m.call(0x001a_62e8, &[h], &[]).1;
        d = std::array::from_fn(|i| p[i] - pos[i]);
        dist = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    }
    let lim = m.rf(o + 0x60);
    let mode = if lim < dist { m.r32(o + 0x68) } else { m.r32(o + 0x64) };
    if mode != m.r32(o + 0xc) {
        m.w32(o + 0xc, mode);
        let (a, b) = (m.r32(o + 4), m.r32(o + 8));
        m.call(0x0023_01d0, &[a, b], &[]);
        let t1 = m.r32(o + 8);
        let t2 = m.r32(t1 + 0x90);
        if t2 != o {
            vcall(m, t2, 0x18, 0x24, &[], &[]);
            return;
        }
    }
    let lim = m.rf(o + 0x60);
    let (pos_v, vel);
    if lim < dist {
        pos_v = [m.rf(o + 0x20), m.rf(o + 0x24), m.rf(o + 0x28), m.rf(o + 0x2c)];
        vel = rov(0x002a_2dd0);
        m.w32(o + 0x74, 0);
    } else {
        let tg = m.r32(o + 8);
        let a1 = m.r32(tg + 0xa0);
        let q = m.call(0x0022_fcc8, &[0, a1], &[d]).1;
        let out = stack_push(m, 8);
        m.call(0x0011_e958, &[out, out + 4], &[q]);
        let (yaw, pitch) = (m.rf(out), m.rf(out + 4));
        stack_pop(m, 8);
        let z = cubic_clamp(m, o, 0x30, yaw, 0x50, 0x54);
        let y = -cubic_clamp(m, o, 0x40, pitch, 0x58, 0x5c);
        let nv = [0.0, y, z, 0.0];
        pos_v = nv;
        if m.r32(o + 0x74) == 0 {
            vel = rov(0x002a_2dd0);
        } else {
            let t1 = m.r32(o + 8);
            let cur = m.rv(t1 + 0xc0);
            let q = 1.0 / m.rf(0x002d_ded4);
            vel = std::array::from_fn(|i| (nv[i] - cur[i]) * q);
        }
        m.w32(o + 0x74, 1);
    }
    let tg = m.r32(o + 8);
    m.wv(tg + 0xc0, pos_v);
    m.wv(tg + 0xd0, vel);
    m.w16(tg + 0x9a, 3);
}
/// c0 + x * (c1 + x * c2) from o+base, clamped to [o+lo, o+hi] (lo checked first)
fn cubic_clamp<M: QMem + ?Sized>(m: &mut M, o: u32, base: u32, x: f32, lo: u32, hi: u32) -> f32 {
    let v = m.rf(o + base) + x * (m.rf(o + base + 4) + x * m.rf(o + base + 8));
    let (l, h) = (m.rf(o + lo), m.rf(o + hi));
    if v < l {
        l
    } else if h < v {
        h
    } else {
        v
    }
}
/// FUN_0012d4d8: copies the target's +0xc0 to *p2 and +0xd0 to *p3 (when non-null)
pub fn ee_sub_12d4d8<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32, p3: u32) {
    if p2 != 0 {
        let t = m.r32(o + 8);
        cpy4(m, p2, t + 0xc0);
    }
    if p3 != 0 {
        let t = m.r32(o + 8);
        cpy4(m, p3, t + 0xd0);
    }
}
