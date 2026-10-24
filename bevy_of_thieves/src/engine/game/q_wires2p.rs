//! WIRE-S2: 1:1 QMem ports of live-frontier natives (shard 3/4).
#![allow(unused)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

fn cpy<M: QMem + ?Sized>(m: &mut M, d: u32, s: u32, n: u32) {
    for k in (0..n).step_by(4) {
        let v = m.r32(s + k);
        m.w32(d + k, v);
    }
}

/// per-byte colour lerp of words at sp+0x80 / sp+0x90 by t, result word at sp+0xa0
fn lerp_rgba<M: QMem + ?Sized>(m: &mut M, sp: u32, t: f32) {
    let k: f32 = f32::from_bits(0x3f7f_be77);
    for i in 0..4 {
        let a = m.r8(sp + 0x80 + i) as i32;
        let b = m.r8(sp + 0x90 + i) as i32;
        let r = if a == b {
            b
        } else if a < b {
            a.wrapping_add((t * ((b as f32 + k) - a as f32)) as i32)
        } else {
            b.wrapping_add(((1.0 - t) * ((a as f32 + k) - b as f32)) as i32)
        };
        m.w8(sp + 0xa0 + i, r as u8);
    }
}

/// FUN_00219400: per-channel keyframe sample/blend of an animation track into 0x70-byte slot records.
pub fn ee_sub_219400<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, t0: u32, t1: u32, f12: f32, f13: f32) {
    let s4 = a0;
    let s5 = a3;
    let (f23, f22) = (f12, f13);
    let v1 = m.r32(s4 + 0x10);
    let tab = m.r32(v1 + 8);
    let idx = a1;
    let d = m.r32(s4 + 4).wrapping_add(idx << 5);
    let cc = m.r32(tab.wrapping_add(a2.wrapping_mul(0x18)) + 4);
    if m.r8(d + 4) == 0 {
        return;
    }
    let sp = stack_push(m, 0xc0);
    let (e0, e4) = (0x7f7f_7f7fu32, 0xffff_ffffu32);
    let mut j: u32 = 0;
    loop {
        let s1 = m.r32(d + 0xc).wrapping_add(j << 6);
        let d4 = m.r16(s1 + 2) as u32;
        let d8 = m.r16(s1 + 4) as u32;
        let dc = m.r16(s1) as u32;
        let s0 = cc.wrapping_add(d4.wrapping_mul(0x70));
        let h = m.r16(s1 + 6) as i16 as i32;
        let mut p = 0u32;
        if h >= 0 {
            p = m.r32(d + 0x10).wrapping_add((h << 3) as u32);
        }
        m.w32(s0 + 0x3c, e0);
        m.w32(sp + 0x40, e0);
        m.w32(s0 + 0x40, e4);
        m.w32(sp + 0x50, e4);
        if p != 0 {
            let v = m.r32(p + 4);
            m.w32(s0 + 0x3c, v);
            m.w32(sp + 0x40, v);
            let v = m.r32(p);
            m.w32(s0 + 0x40, v);
            m.w32(sp + 0x50, v);
        }
        j = j.wrapping_add(1);
        m.w32(s0 + 0x44, s1 + 0x10);
        let fd = m.rf(d);
        m.w32(s0 + 0x54, 0);
        m.wf(s0 + 0x38, fd);
        m.w32(s0 + 0x48, 0);
        let mut blended = false;
        if m.r8(s1 + 0x30) != 0 {
            // search following tracks for a key on the same slot
            let mut a1 = idx.wrapping_add(1);
            while (a1 as i32) < (m.r16(s4) as i32) {
                let a2 = m.r32(s4 + 4).wrapping_add(a1 << 5);
                let next = a1.wrapping_add(1);
                let mut k: u32 = 0;
                if m.r8(a2 + 4) != 0 {
                    loop {
                        let off = k << 6;
                        let v1 = off.wrapping_add(m.r32(a2 + 0xc));
                        if m.r16(v1 + 2) as u32 == d4 {
                            m.w32(s0 + 0x54, v1 + 0x10);
                            let f = m.rf(a2);
                            m.wf(s0 + 0x48, f);
                            let v0 = m.r32(a2 + 0xc).wrapping_add(off);
                            let hh = m.r16(v0 + 6) as i16 as i32;
                            if hh >= 0 {
                                let q = ((hh << 3) as u32).wrapping_add(m.r32(a2 + 0x10));
                                let v = m.r32(q + 4);
                                m.w32(s0 + 0x4c, v);
                                let v = m.r32(q);
                                m.w32(s0 + 0x50, v);
                            } else {
                                let v = m.r32(s0 + 0x3c);
                                m.w32(s0 + 0x4c, v);
                                let v = m.r32(s0 + 0x40);
                                m.w32(s0 + 0x50, v);
                            }
                            break;
                        }
                        k = k.wrapping_add(1);
                        if !((k as i32) < (m.r8(a2 + 4) as i32)) {
                            break;
                        }
                    }
                }
                a1 = next;
                if !((a1 as i32) < (m.r16(s4) as i32)) || m.r32(s0 + 0x54) != 0 {
                    break;
                }
            }
            if m.r8(s1 + 0x30) != 0 && m.r32(s0 + 0x54) != 0 && m.rf(s0 + 0x38) < m.rf(s0 + 0x48) {
                blended = true;
                m.call(0x0023_33b4, &[sp + 0x70, 0, 8], &[]);
                let f = m.rf(s0 + 0x38);
                m.wf(sp + 0x70, f);
                let f = m.rf(s0 + 0x48);
                m.wf(sp + 0x74, f);
                cpy(m, sp + 0x60, sp + 0x70, 8);
                let c = m.call(0x0011_d948, &[sp + 0x60], &[f4(f22)]).1[0];
                let f2 = m.rf(s0 + 0x38);
                let f1 = m.rf(s0 + 0x48) - f2;
                let t = (c - f2) / f1;
                let u = 1.0 - t;
                let mut pa = m.r32(s0 + 0x44);
                let mut pb = m.r32(s0 + 0x54);
                for i in 0..6 {
                    let x = u * m.rf(pa) + t * m.rf(pb);
                    m.wf(sp + 0xa0 + i * 4, x);
                    pa += 4;
                    pb += 4;
                }
                cpy(m, sp + 0x10, sp + 0xb0, 16);
                cpy(m, sp + 0x80, sp + 0xa0, 16);
                cpy(m, sp + 0x90, sp + 0xb0, 16);
                cpy(m, sp, sp + 0xa0, 16);
                let v = m.r32(s0 + 0x3c);
                m.w32(sp + 0x80, v);
                let v = m.r32(s0 + 0x4c);
                m.w32(sp + 0x90, v);
                lerp_rgba(m, sp, t);
                let v = m.r32(sp + 0xa0);
                m.w32(sp + 0x70, v);
                m.w32(sp + 0x40, v);
                let v = m.r32(s0 + 0x40);
                m.w32(sp + 0x80, v);
                let v = m.r32(s0 + 0x50);
                m.w32(sp + 0x90, v);
                lerp_rgba(m, sp, t);
                let v = m.r32(sp + 0xa0);
                m.w32(sp + 0x70, v);
                m.w32(sp + 0x50, v);
            }
        }
        if !blended {
            let v0 = m.r32(s0 + 0x44);
            cpy(m, sp, v0, 32);
            let v = m.r32(s0 + 0x3c);
            m.w32(sp + 0x40, v);
            let v = m.r32(s0 + 0x40);
            m.w32(sp + 0x50, v);
        }
        // 2x3 affine transform by s5
        let f11 = m.rf(s5);
        let f10 = m.rf(s5 + 8);
        let f0 = m.rf(s5 + 4);
        let f7 = m.rf(s5 + 0xc);
        let fx = m.rf(s5 + 0x10);
        let fy = m.rf(s5 + 0x14);
        let (x5, x6) = (m.rf(sp + 0x10), m.rf(sp + 0x14));
        let (x2, x3, x4, x1) = (m.rf(sp), m.rf(sp + 4), m.rf(sp + 8), m.rf(sp + 0xc));
        m.wf(sp + 0x80, x2 * f11 + x3 * f10);
        m.wf(sp + 0x84, x2 * f0 + x3 * f7);
        m.wf(sp + 0x88, x4 * f11 + x1 * f10);
        m.wf(sp + 0x8c, x4 * f0 + x1 * f7);
        m.wf(sp + 0x90, (x5 * f11 + x6 * f10) + fx);
        m.wf(sp + 0x94, (x5 * f0 + x6 * f7) + fy);
        cpy(m, sp, sp + 0x80, 16);
        cpy(m, sp + 0x60, sp + 0x80, 16);
        cpy(m, sp + 0x10, sp + 0x90, 16);
        cpy(m, sp + 0x70, sp + 0x90, 16);
        let v = m.r32(t1);
        m.w32(sp + 0x60, v);
        let v = m.r32(t0);
        m.w32(sp + 0x70, v);
        let v = m.r32(sp + 0x50);
        m.w32(sp + 0x80, v);
        let v = m.r32(sp + 0x40);
        m.w32(sp + 0x90, v);
        m.call(0x0021_8790, &[sp + 0x60, sp + 0x70, sp + 0x80, sp + 0x90, sp + 0x30, sp + 0x20], &[]);
        let c20 = m.r32(sp + 0x20);
        let c30 = m.r32(sp + 0x30);
        if (dc as i32) > 0 {
            let o = m.r32(s0 + 0x30);
            if o != 0 && m.r32(o) == dc {
                m.w32(sp + 0x60, c20);
                m.w32(sp + 0x70, c30);
                let o = m.r32(s0 + 0x30);
                m.call(0x0021_5850, &[o, s0, sp, sp + 0x60, sp + 0x70], &[f4(f22)]);
            } else {
                if o != 0 {
                    m.call(0x0021_58c0, &[o, s0], &[f4(f22)]);
                    let c20 = m.r32(sp + 0x20);
                    m.w32(sp + 0x60, c20);
                } else {
                    m.w32(sp + 0x60, c20);
                }
                let c30 = m.r32(sp + 0x30);
                m.w32(sp + 0x70, c30);
                let w = m.r32(s4 + 0xc);
                let base = m.r32(w + 0x54);
                m.call(0x0021_5730, &[base.wrapping_add(dc << 6), s0, sp, sp + 0x60, sp + 0x70], &[]);
            }
        } else {
            m.w32(sp + 0x60, c20);
            m.w32(sp + 0x70, c30);
            let o = m.r32(s0 + 0x30);
            m.call(0x0021_5850, &[o, s0, sp, sp + 0x60, sp + 0x70], &[f4(f22)]);
        }
        m.wf(s0 + 0x5c, f23);
        cpy(m, s0 + 0x10, sp, 32);
        let v = m.r32(sp + 0x20);
        m.w32(s0, v);
        let v = m.r32(sp + 0x30);
        m.w32(s0 + 4, v);
        let b = m.r8(s1 + 0x30);
        m.w8(s0 + 0x60, b);
        let b = m.r8(s1 + 0x31);
        m.w8(s0 + 0x61, b);
        let v = m.r32(s1 + 0x34);
        m.w32(s0 + 0x64, v);
        m.call(0x0011_a058, &[s0 + 0x68], &[]);
        if d8 != 0 {
            let q = cc.wrapping_add(d4.wrapping_add(d8).wrapping_mul(0x70));
            m.w32(s0 + 0x34, 1);
            m.w32(q + 0x34, 4);
        } else if m.r32(s0 + 0x34) != 4 {
            m.w32(s0 + 0x34, 0);
        }
        if !((j as i32) < (m.r8(d + 4) as i32)) {
            break;
        }
    }
    stack_pop(m, 0xc0);
}
