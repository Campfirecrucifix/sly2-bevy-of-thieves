//! WIRE-W1G5: 1:1 queue ports (against `QMem`) of a batch of EE functions, ported from the EE asm;
//! callees go through `m.call` / `vslot`. QMem convention: integer / pointer args in `a` (stack args
//! appended after the 8 register args), quadword and float args in `v`.
#![allow(clippy::too_many_arguments)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

const GS: u32 = 0x003d_d490; // game state / world

/// FUN_0015bd20(a0, a1, a2)
pub fn ee_sub_15bd20<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    if a0 == 0 {
        return;
    }
    let sp = stack_push(m, 0x50);
    m.call(0x001a_e478, &[sp + 0x30], &[]);
    m.w32(sp + 0x20, 1);
    m.call(0x0015_b7e8, &[a0, a1, sp + 0x10], &[]);
    let (w0, w1, w2, w3) = (m.r32(sp + 0x10), m.r32(sp + 0x14), m.r32(sp + 0x18), m.r32(sp + 0x1c));
    if m.r32(a1 + 4) == 1 {
        m.call(0x0015_b1b0, &[a1, a2, a0, w0, w1, w2, w3], &[]);
    } else {
        let v0 = if m.r32(a1 + 0xc4) == 2 {
            m.call(0x0015_b9f0, &[a0, a1, sp + 0x10, sp + 0x40], &[]);
            m.r32(sp + 0x44)
        } else {
            m.w32(sp + 0x40, 0);
            m.w32(sp + 0x44, 0);
            0
        };
        let (w0, w1, w2, w3) = (m.r32(sp + 0x10), m.r32(sp + 0x14), m.r32(sp + 0x18), m.r32(sp + 0x1c));
        let t3 = m.r32(sp + 0x40);
        m.call(0x0015_b428, &[a1, a2, a0, w0, w1, w2, w3, t3, v0], &[]);
    }
    m.call(0x001a_e4a0, &[sp + 0x30, 2], &[]);
    stack_pop(m, 0x50);
}

/// FUN_0012bf30(o, bis): reads a key-time table and up to 3 vector channels.
pub fn ee_sub_12bf30<M: QMem + ?Sized>(m: &mut M, o: u32, bis: u32) {
    let sp = stack_push(m, 0x10);
    m.call(0x0011_7eb0, &[bis, o + 0x20], &[]);
    let n = m.call(0x0011_7c20, &[bis], &[]).0;
    m.w32(sp + 4, 0x10);
    m.w32(sp, n << 2);
    m.w32(o + 4, n);
    let p = m.call(0x001a_e1d0, &[sp], &[]).0;
    m.w32(o + 8, p);
    let k = f32::from_bits(0x3c88_8889);
    let mut i = 0i32;
    while i < m.r32(o + 4) as i32 {
        let v = m.call(0x0011_7c20, &[bis], &[]).0;
        let a = ((i as u32) << 2).wrapping_add(m.r32(o + 8));
        m.wf(a, v as i32 as f32 * k);
        i += 1;
    }
    let mask = m.call(0x0011_7bd0, &[bis], &[]).0;
    let mut bit = 1u32;
    for j in 0..3u32 {
        if bit & mask != 0 {
            let n = m.r32(o + 4);
            m.w32(sp + 4, 0x10);
            m.w32(sp, n.wrapping_mul(0xc));
            let mut q = m.call(0x001a_e1d0, &[sp], &[]).0;
            m.w32(o + 0xc + j * 4, q);
            let mut i = 0i32;
            while i < m.r32(o + 4) as i32 {
                i += 1;
                for k in 0..3 {
                    let f = m.call(0x0011_7e38, &[bis], &[]).1[0];
                    m.wf(q + k * 4, f);
                }
                q += 0xc;
            }
        }
        bit <<= 1;
    }
    stack_pop(m, 0x10);
}

/// FUN_001e4530(a0, p) -> f32
pub fn ee_sub_1e4530<M: QMem + ?Sized>(m: &mut M, a0: u32, p: u32) -> f32 {
    let mut f20 = 0.0f32;
    if m.r32(p + 4) != 0 {
        return f20;
    }
    match m.r32(p) {
        10 => {
            let a1 = m.r32(p + 0x10);
            let s = m.call(0x001e_1db8, &[a0, a1], &[]).0;
            if s != 0 {
                let f0 = m.call(0x0015_4c90, &[0x002d_e578], &[]).1[0];
                let f1 = f0 * m.rf(s + 0x88);
                let f2 = m.rf(s + 0x84) + f1;
                let mut f3 = m.rf(s + 0x80) + f0 * f2;
                let f4v = m.rf(s + 0x90);
                let f5 = m.rf(s + 0x94);
                if f3 < f4v {
                    f3 = f4v;
                } else if f5 < f3 {
                    f3 = f5;
                }
                return m.rf(s + 0x74) / (m.rf(s + 0x78) * f3);
            }
        }
        0xd => {
            if m.r32(p + 0x18) != u32::MAX {
                let a3 = m.r32(a0 + 0x18);
                let a2 = m.r32(p + 0x10);
                let o = m.call(0x0015_e938, &[GS, 4, a2, a3], &[]).0;
                if o == 0 {
                    return f20;
                }
                let a1 = m.r32(p + 0x14);
                let o = m.call(0x001f_67e0, &[o, a1], &[]).0;
                if o == 0 {
                    return f20;
                }
                let a1 = m.r32(p + 0x18);
                f20 = m.call(0x001f_11a8, &[o, a1], &[]).1[0];
            }
        }
        0x23 | 0x33 => f20 = m.rf(p + 0x10),
        0x2c | 0x2d | 0x53 => f20 = m.rf(p + 0x14),
        0x32 => f20 = m.rf(0x002b_d850),
        0x45 => {
            let a1 = m.r32(p + 0x10);
            f20 = m.call(0x001f_cfd0, &[0x003d_d040, a1], &[]).1[0];
        }
        _ => {}
    }
    f20
}

/// FUN_001fe150(o, a1) -> handle
pub fn ee_sub_1fe150<M: QMem + ?Sized>(m: &mut M, o: u32, a1: u32) -> u32 {
    let sp = stack_push(m, 0x20);
    m.call(0x001f_e478, &[sp, o, a1], &[]);
    let n = m.r32(o + 0x204);
    let top = if (n as i32) <= 0 { 0 } else { m.r32((n << 2).wrapping_add(o) + 0x204) };
    if top != 0 {
        m.call(0x001f_b098, &[top], &[f4(0.0)]);
    }
    let h = m.r32(sp);
    if h != 0 {
        vslot(m, h, 6, &[], &[]);
    } else {
        let r = m.call(0x001f_cff8, &[o, a1], &[]).0;
        if r == 0 || m.r32(o + 0x204) == 8 {
            let h = m.r32(sp);
            if h != 0 {
                vslot(m, h, 1, &[], &[]);
            }
            stack_pop(m, 0x20);
            return 0;
        }
        let x = m.call(0x001f_c138, &[r, 4], &[]).0;
        m.call(0x0025_9b48, &[sp, x], &[]);
        m.wf(sp + 0x10, f32::from_bits(0x44fa_0000));
        m.wf(sp + 0x14, f32::from_bits(0x459c_4000));
        for (k, g) in [(3u32, 0x002d_d5bcu32), (3, 0x002f_7900), (4, 0x002e_1e40)] {
            let h = m.r32(sp);
            let a2 = m.r32(g);
            m.call(0x001f_c408, &[h, k, a2, sp + 0x10], &[f4(1.0)]);
        }
    }
    let n = m.r32(o + 0x204);
    let s0 = (n << 2).wrapping_add(o) + 0x208;
    m.w32(o + 0x204, n.wrapping_add(1));
    let h = m.r32(sp);
    m.call(0x0025_9b48, &[s0, h], &[]);
    let h = m.r32(sp);
    let ret = m.r32(s0);
    if h != 0 {
        vslot(m, h, 1, &[], &[]);
    }
    stack_pop(m, 0x20);
    ret
}

/// FUN_0014e328(o, key, obj, a3, t0): sorted insert into the +0x30c list (max 8).
pub fn ee_sub_14e328<M: QMem + ?Sized>(m: &mut M, o: u32, key: u32, obj: u32, a3: u32, t0: u32) {
    let n = m.r32(o + 0x30c);
    if n >= 8 {
        return;
    }
    let mut i = 0u32;
    if (n as i32) > 0 && (m.r32(o + 0x310) as i32) >= (key as i32) {
        i = 1;
        let mut p = o + 0x310;
        while (i as i32) < (n as i32) {
            p += 0x10;
            if (m.r32(p) as i32) < (key as i32) {
                break;
            }
            i += 1;
        }
    }
    let off = i << 4;
    let b = off.wrapping_add(o);
    m.call(0x0023_32b8, &[b + 0x320, b + 0x310, n.wrapping_sub(i) << 4], &[]);
    m.w32(o + 0x310 + off, key);
    m.w32(o + 0x314 + off, obj);
    m.w32(o + 0x318 + off, a3);
    m.w32(o + 0x31c + off, t0);
    let c = m.r32(o + 0x30c).wrapping_add(1);
    m.w32(o + 0x30c, c);
    vslot(m, obj, 3, &[t0], &[]);
    if i == 0 {
        if (m.r32(o + 0x30c) as i32) >= 2 {
            let x = m.r32(o + 0x324);
            let (a1, a2) = (m.r32(o + 0x328), m.r32(o + 0x32c));
            vslot(m, x, 2, &[a1, a2], &[]);
        }
        let t = m.rf(0x002d_ded0);
        m.wf(o + 0x390, t);
        vslot(m, obj, 1, &[a3, t0], &[]);
    }
}

/// FUN_001fbc58(o)
pub fn ee_sub_1fbc58<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let s1 = 0x003d_d040u32;
    let k = m.r32(o + 0x60);
    let s2 = m.call(0x001f_d028, &[s1, k], &[]).0;
    if s2 == 0 || m.r32(s1 + 0xe8) == 0 {
        m.call(0x001f_b658, &[o, 0x1fff, 0], &[]);
        return;
    }
    if m.r16(o + 0x50) & 2 != 0 {
        m.call(0x001f_b710, &[o], &[]);
        return;
    }
    let sp = stack_push(m, 0x30);
    m.call(0x0023_33b4, &[sp, 0, 0x30], &[]);
    let v = m.r32(s1 + 0xe8);
    m.w32(sp + 0x28, 3);
    m.w32(sp, v);
    let r = m.call(0x001f_d070, &[s1, s2], &[]).0;
    m.w16(sp + 0x10, r as u16);
    if m.r16(o + 0x50) & 0x10 != 0 {
        let f = m.rf(o + 0x44) * 1024.0;
        let w = m.r32(sp + 0x28) | 4;
        m.w32(sp + 0x28, w);
        m.w32(sp + 8, f as i32 as u32);
    }
    if m.r16(o + 0x50) & 0x20 != 0 {
        let x = m.r32(o + 0x14);
        let w = m.r32(sp + 0x28) | 8;
        m.w32(sp + 0x28, w);
        m.w32(sp + 0xc, x);
    }
    if m.r16(o + 0x50) & 0x40 != 0 {
        let f = m.rf(o + 0x18) * 1524.0;
        let w = m.r32(sp + 0x28) | 0x10;
        m.w32(sp + 0x28, w);
        m.w16(sp + 0x20, f as i32 as u16);
    }
    if m.r32(s2) == 2 {
        m.w16(o + 0x52, 1);
        vslot(m, o, 0, &[], &[]);
        m.call(0x0023_2320, &[sp, 0x001f_baf8, o], &[]);
        m.call(0x001f_b658, &[o, 0x7f, 0], &[]);
    } else {
        m.call(0x0023_22f0, &[sp], &[]);
        m.call(0x001f_b658, &[o, 0x1fff, 0], &[]);
        let id = m.r32(o + 0x60) as i32;
        let g = match id {
            0x6f => m.r32(0x002e_1e40),
            0x15 => m.r32(0x002f_7900),
            0x73 => m.r32(0x002d_d5bc),
            _ => 0,
        };
        if g == m.call(0x001c_4840, &[GS], &[]).0 {
            m.call(0x001f_eed0, &[s1], &[]);
        }
    }
    stack_pop(m, 0x30);
}

/// FUN_001baba0(o, bis): load + relocate the +0x154 / +0x164 tables into the level heap.
pub fn ee_sub_1baba0<M: QMem + ?Sized>(m: &mut M, o: u32, bis: u32) {
    let sp = stack_push(m, 0x20);
    let req = sp + 0x10;
    m.call(0x001a_e478, &[sp], &[]);
    m.w32(req, 0x40);
    m.w32(req + 4, 0x10);
    let v = m.call(0x001a_e318, &[req], &[]).0;
    m.w32(o + 0x154, v);
    m.w32(req, 0x200);
    m.w32(req + 4, 0x10);
    let v = m.call(0x001a_e378, &[req], &[]).0;
    m.w32(o + 0x164, v.wrapping_add(0x10));
    m.call(0x001f_31e0, &[o, bis], &[]);
    let n = m.r32(o + 0x150);
    let a1 = m.r32(o + 0x154);
    m.w32(o + 0x15c, 0);
    m.w32(req, n << 3);
    m.w32(req + 4, 0x10);
    let v = m.call(0x001a_e238, &[req, a1], &[]).0;
    m.w32(o + 0x154, v);
    let c = m.r32(o + 0x160);
    if c == 0 {
        m.w32(o + 0x164, 0);
    } else {
        let a1 = m.r32(o + 0x164).wrapping_sub(0x10);
        m.w32(req + 4, 0x10);
        m.w32(req, (c << 4).wrapping_add(0x10));
        let v = m.call(0x001a_e238, &[req, a1], &[]).0;
        m.w32(o + 0x164, v.wrapping_add(0x10));
        let mut i: i32 = -1;
        while i < m.r32(o + 0x160) as i32 {
            let s1 = m.r32(o + 0x164).wrapping_add((i << 4) as u32);
            i += 1;
            let k = m.r32(s1 + 8);
            let a1 = m.r32(s1 + 0xc);
            m.w32(req + 4, 0x10);
            m.w32(req, k << 3);
            let v = m.call(0x001a_e238, &[req, a1], &[]).0;
            m.w32(s1 + 0xc, v);
            let mut j = 0i32;
            while j < m.r32(s1 + 8) as i32 {
                let e = m.r32(s1 + 0xc).wrapping_add((j as u32) << 3);
                if m.r32(e) == 0xc {
                    let a1 = m.r32(e + 4);
                    m.w32(req, 8);
                    m.w32(req + 4, 0x10);
                    let v = m.call(0x001a_e238, &[req, a1], &[]).0;
                    m.w32(e + 4, v);
                }
                j += 1;
            }
        }
    }
    m.call(0x001a_e4a0, &[sp, 2], &[]);
    stack_pop(m, 0x20);
}

/// FUN_00185150(o, bis)
pub fn ee_sub_185150<M: QMem + ?Sized>(m: &mut M, o: u32, bis: u32) {
    let sp = stack_push(m, 0x10);
    m.call(0x0023_1768, &[o, bis], &[]);
    let n = m.call(0x0011_7d50, &[bis], &[]).0;
    m.w32(sp + 4, 0x10);
    m.w32(sp, n << 4);
    m.w32(o + 0xe0, n);
    let v = m.call(0x001a_e1d0, &[sp], &[]).0;
    m.w32(o + 0xe4, v);
    let mut i = 0i32;
    while i < m.r32(o + 0xe0) as i32 {
        let a = m.r32(o + 0xe4).wrapping_add((i as u32) << 4);
        i += 1;
        m.call(0x0011_7eb0, &[bis, a], &[]);
    }
    let n = m.call(0x0011_7d50, &[bis], &[]).0;
    m.w32(sp + 4, 0x10);
    m.w32(sp, n.wrapping_mul(0xc));
    m.w32(o + 0xe8, n);
    let v = m.call(0x001a_e1d0, &[sp], &[]).0;
    m.w32(o + 0xec, v);
    let mut i = 0i32;
    while i < m.r32(o + 0xe8) as i32 {
        let p = m.r32(o + 0xec).wrapping_add((i as u32).wrapping_mul(0xc));
        i += 1;
        for k in 0..3 {
            let v = m.call(0x0011_7d50, &[bis], &[]).0;
            m.w32(p + k * 4, v);
        }
    }
    let n = m.r32(o + 0xe8);
    m.w32(sp, n.wrapping_mul(0xc));
    m.w32(sp + 4, 0x10);
    let v = m.call(0x001a_e1d0, &[sp], &[]).0;
    m.w32(o + 0xf0, v);
    let mut i = 0i32;
    while i < m.r32(o + 0xe8) as i32 {
        let off = (i as u32).wrapping_mul(0xc);
        i += 1;
        for k in 0..3 {
            let v = m.call(0x0011_7d50, &[bis], &[]).0;
            let a = m.r32(o + 0xf0).wrapping_add(off + k * 4);
            m.w32(a, v);
        }
    }
    m.call(0x0013_f810, &[o, bis], &[]);
    stack_pop(m, 0x10);
}

fn req<M: QMem + ?Sized>(m: &mut M, r: u32, n: u32) {
    m.w32(r, n);
    m.w32(r + 4, 0x10);
}

/// FUN_001635f8(o, bis)
pub fn ee_sub_1635f8<M: QMem + ?Sized>(m: &mut M, o: u32, bis: u32) {
    let sp = stack_push(m, 0x20);
    let r = sp + 0x10;
    m.call(0x001a_e478, &[sp], &[]);
    req(m, r, 0x200);
    let v = m.call(0x001a_e318, &[r], &[]).0;
    m.w32(o + 0xac, v);
    req(m, r, 0x300);
    let v = m.call(0x001a_e318, &[r], &[]).0;
    m.w32(o + 0xb4, v);
    m.call(0x001a_5c28, &[o, bis], &[]);
    let n = m.r32(o + 0xa8);
    req(m, r, n << 4);
    let a1 = m.r32(o + 0xac);
    let v = m.call(0x001a_e238, &[r, a1], &[]).0;
    let n = m.r32(o + 0xb0);
    m.w32(o + 0xac, v);
    req(m, r, n.wrapping_mul(0x18));
    let a1 = m.r32(o + 0xb4);
    let v = m.call(0x001a_e238, &[r, a1], &[]).0;
    m.w32(o + 0xb4, v);
    for (cnt, arr, ptr) in [(0x94u32, 0x98u32, 0x90u32), (0xa0, 0xa4, 0x9c)] {
        let n = m.call(0x0011_7bd0, &[bis], &[]).0;
        m.w32(o + cnt, n);
        req(m, r, n << 2);
        let v = m.call(0x001a_e2a0, &[r], &[]).0;
        m.w32(o + arr, v);
        let n = m.r32(o + cnt);
        req(m, r, n << 2);
        let v = m.call(0x001a_e1d0, &[r], &[]).0;
        m.w32(o + ptr, v);
        let mut i = 0i32;
        while i < m.r32(o + cnt) as i32 {
            let v = m.call(0x0011_7db8, &[bis], &[]).0;
            let a = ((i as u32) << 2).wrapping_add(m.r32(o + ptr));
            i += 1;
            m.w32(a, v);
        }
    }
    m.call(0x0011_8420, &[bis], &[]);
    m.call(0x001a_e4a0, &[sp, 2], &[]);
    stack_pop(m, 0x20);
}

/// FUN_0022a060(o): proximity trigger on the player; toggles +0x164 and fires vslot 2 (5 on / 6 off).
pub fn ee_sub_22a060<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001f_36e0, &[o], &[]);
    let pl = m.call(0x001c_4840, &[GS], &[]).0;
    let mut s2 = 0u32;
    if pl != 0 {
        let c = m.r32(o + 0x154);
        if m.call(0x0013_2b10, &[pl, c], &[]).0 != 0 {
            let f20 = if m.r32(o + 0x164) == 0 { m.rf(o + 0x158) } else { m.rf(o + 0x15c) };
            let a = m.call(0x001a_62e8, &[pl], &[]).1;
            let b = m.call(0x001a_62e8, &[o], &[]).1;
            let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
            let dist = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            if dist < f20 {
                s2 = 1;
            }
        }
    }
    let st = m.r32(o + 0x150);
    if (st == 0 || st == 2) && m.r32(GS + 0x3b0c) != 0 {
        s2 = 0;
    }
    let v0;
    if s2 == 0 {
        v0 = m.r32(o + 0x164);
    } else if m.call(0x0013_2b10, &[pl, 0x34], &[]).0 == 0 {
        v0 = m.r32(o + 0x164);
    } else if m.r32(pl + 0x2dc) != u32::MAX {
        s2 = m.r32(o + 0x164);
        v0 = m.r32(o + 0x164);
    } else if vslot(m, pl, 76, &[], &[]).0 != 2 {
        v0 = m.r32(o + 0x164);
    } else {
        s2 = m.r32(o + 0x164);
        v0 = m.r32(o + 0x164);
    }
    if s2 != v0 {
        m.w32(o + 0x164, s2);
        let ev = if s2 != 0 { 5 } else { 6 };
        vslot(m, o, 2, &[ev, 0, 0, 0, 0], &[]);
    }
}

/// FUN_00211f90(o): count completed sub-goals (lists +0x194/+0x19c and +0x110/+0x118, stride 8).
pub fn ee_sub_211f90<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let (mut s3, mut s4, mut s5) = (0i32, 0i32, 0i32);
    m.call(0x0021_2c50, &[o], &[]);
    let mut i = 0i32;
    let mut p = o + 0x19c;
    while i < m.r32(o + 0x194) as i32 {
        let e = m.r32(p);
        if e != 0 {
            if m.r32(e + 0x54) == 1 {
                s3 += 1;
            } else if m.call(0x0016_4030, &[e], &[]).0 != 0 {
                s4 += 1;
                s5 += 1;
            }
        }
        i += 1;
        p += 8;
    }
    let mut i = 0i32;
    let mut p = o + 0x118;
    while i < m.r32(o + 0x110) as i32 {
        let e = m.r32(p);
        if e != 0 {
            if m.r32(e + 0x54) == 1 {
                s3 += 1;
            } else if m.call(0x0016_4030, &[e], &[]).0 != 0 {
                s4 += 1;
            }
        }
        i += 1;
        p += 8;
    }
    if m.call(0x0021_21e8, &[o, 0x314], &[]).0 != 0 {
        let (a, b) = (0x002c_3ec0u32, 0x002a_8aa0u32);
        m.call(0x0021_ecd8, &[a, b, s5 as u32], &[]);
        let n = m.r32(o + 0x194);
        m.call(0x0021_ecd8, &[a + 0x30, b, n], &[]);
        let n = m.r32(o + 0x194);
        m.call(0x0021_ecd8, &[a + 0x60, b, n.wrapping_sub(s5 as u32)], &[]);
    }
    let tot = m.r32(o + 0x194).wrapping_add(m.r32(o + 0x110));
    if s4 as u32 == tot || s3 == 0 {
        vslot(m, o, 49, &[], &[]);
    } else if s3 > 0 {
        vslot(m, o, 48, &[u32::MAX], &[]);
    }
}

/// `(*(*(obj) + 0xc))(obj + *(short *)(*obj + 8), ..)` (vtable pointer at +0, first slot)
fn vcall0<M: QMem + ?Sized>(m: &mut M, obj: u32, a: &[u32]) -> (u32, V4) {
    let vt = m.r32(obj);
    let f = m.r32(vt + 0xc);
    let adj = m.r16(vt + 8) as i16 as i32;
    let mut args = vec![obj.wrapping_add(adj as u32)];
    args.extend_from_slice(a);
    m.call(f, &args, &[])
}

/// FUN_0012f2a8(o, bis)
pub fn ee_sub_12f2a8<M: QMem + ?Sized>(m: &mut M, o: u32, bis: u32) {
    let sp = stack_push(m, 0x10);
    let f = m.call(0x0011_7e38, &[bis], &[]).1[0];
    m.wf(o + 0x74, f);
    let n = m.call(0x0011_7d50, &[bis], &[]).0;
    req(m, sp, n << 3);
    m.w32(o + 0x5c, n);
    let v = m.call(0x001a_e1d0, &[sp], &[]).0;
    m.w32(o + 0x60, v);
    let mut i = 0i32;
    while i < m.r32(o + 0x5c) as i32 {
        let e = m.r32(o + 0x60).wrapping_add((i as u32) << 3);
        let v = m.call(0x0011_7db8, &[bis], &[]).0;
        let w = (m.r32(e) & 0xfff0_0000) | (v & 0x000f_ffff);
        m.w32(e, w);
        let v = m.call(0x0011_7d50, &[bis], &[]).0;
        let w = (m.r32(e) & 0xc00f_ffff) | ((v & 0x3ff) << 20);
        m.w32(e, w);
        let k = m.call(0x0011_7d00, &[bis], &[]).0;
        if k != u32::MAX {
            let p = m.call(0x0012_b3b0, &[k], &[]).0;
            m.w32(e + 4, p);
            vcall0(m, p, &[bis]);
            let w = m.r32(e) & 0x7fff_ffff;
            m.w32(e, w);
        }
        let k = m.call(0x0011_7d00, &[bis], &[]).0;
        if k != u32::MAX {
            let p = m.call(0x0012_cb98, &[k], &[]).0;
            m.w32(e + 4, p);
            vcall0(m, p, &[bis]);
            let w = m.r32(e) | 0x8000_0000;
            m.w32(e, w);
        }
        i += 1;
        let b = m.call(0x0011_7bd0, &[bis], &[]).0;
        let w = (m.r32(e) & 0xbfff_ffff) | (((b != 0) as u32) << 30);
        m.w32(e, w);
    }
    let n = m.call(0x0011_7bd0, &[bis], &[]).0;
    req(m, sp, n.wrapping_mul(0xc));
    m.w32(o + 0x64, n);
    let v = m.call(0x001a_e2a0, &[sp], &[]).0;
    m.w32(o + 0x68, v);
    let mut i = 0i32;
    while i < m.r32(o + 0x64) as i32 {
        let e = m.r32(o + 0x68).wrapping_add((i as u32).wrapping_mul(0xc));
        i += 1;
        let v = m.call(0x0011_7bd0, &[bis], &[]).0;
        m.w32(e, v);
        let v = m.call(0x0011_7db8, &[bis], &[]).0;
        m.w32(e + 4, v);
        let k = m.call(0x0011_7d00, &[bis], &[]).0;
        let p = m.call(0x0012_cb98, &[k], &[]).0;
        m.w32(e + 8, p);
        vcall0(m, p, &[bis]);
    }
    m.call(0x0012_f590, &[o, bis, 0, o + 0x98, o + 0x9c], &[]);
    m.call(0x0012_f590, &[o, bis, 1, o + 0xa0, o + 0xa4], &[]);
    m.call(0x0012_f590, &[o, bis, 0, o + 0xa8, o + 0xac], &[]);
    m.call(0x0013_f810, &[o, bis], &[]);
    stack_pop(m, 0x10);
}

/// FUN_001b68b8(jt, msg, p): message handler.
pub fn ee_sub_1b68b8<M: QMem + ?Sized>(m: &mut M, o: u32, msg: u32, p: u32) {
    m.call(0x0014_0f90, &[o, msg, p], &[]);
    if msg == 9 {
        let s = m.r32(p);
        if s != m.r32(o + 0x1030) && s != m.r32(o + 0x190c) {
            return;
        }
        match m.r32(p + 8) as i32 {
            0x1c8 => {
                if m.r32(o + 0x1450) == 0 {
                    let pos = m.call(0x001a_62e8, &[o], &[]).1;
                    m.call(0x0015_6b98, &[GS, 0, o], &[pos]);
                    m.call(0x001b_93e8, &[o, 1], &[]);
                }
            }
            0x1a5 => {
                let pos = m.call(0x001a_62e8, &[o], &[]).1;
                m.call(0x0015_6b98, &[GS, 0, o], &[pos]);
            }
            0x1a4 => {
                m.call(0x001b_6da8, &[o, 1], &[]);
            }
            0x1c6 => m.w32(o + 0x1448, 1),
            0x1c7 => {
                if m.r32(o + 0x1450) == 1 {
                    m.call(0x001b_93e8, &[o, 2], &[]);
                }
            }
            0x240 => m.w32(o + 0x1818, 1),
            0x241 => m.w32(o + 0x181c, 1),
            0x242 => m.w32(o + 0x1820, 1),
            0x1eb => match m.r32(o + 0x13d0) as i32 {
                8 => {
                    m.call(0x001b_7ae8, &[o], &[]);
                }
                7 => {
                    m.call(0x001b_7970, &[o], &[]);
                }
                0xf => {
                    m.call(0x0022_05a8, &[o + 0x1910], &[]);
                }
                _ => {}
            },
            _ => {}
        }
    } else if msg == 10 {
        if m.r32(p) != m.r32(o + 0x1034) || m.r32(o + 0x13d0) != 6 {
            return;
        }
        let v = m.r32(p + 8);
        if v == m.r32(p + 4) {
            return;
        }
        match v as i32 {
            0x1cb..=0x1cf => {
                let i = m.r32(o + 0x1534);
                let f0 = m.rf((i << 2).wrapping_add(0x002c_0990));
                let f1 = m.rf(0x002c_099c) * f0;
                m.wf(o + 0x1528, f0);
                m.wf(o + 0x1524, f0);
                m.w32(o + 0x152c, f1 as i32 as u32);
            }
            0x1ca | 0x1d0 => {
                m.call(0x001b_6078, &[o, 0], &[]);
            }
            _ => {}
        }
    }
}

/// FUN_00176458(o, st): state change.
pub fn ee_sub_176458<M: QMem + ?Sized>(m: &mut M, o: u32, st: u32) {
    if st == m.r32(o + 0x34) {
        return;
    }
    m.call(0x001e_f0d8, &[o + 0x18], &[]);
    if m.r32(o + 0x34) == 1 {
        let q = m.r32(o + 0x3c);
        let t = m.rf(0x002d_ded0);
        let (a, b) = (m.rf(q + 0x60), m.rf(q + 0x64));
        let f = m.call(0x0011_d448, &[], &[f4(a), f4(b)]).1[0];
        let p = m.r32(o + 0x10);
        m.wf(p + 0xf04, t + f);
        m.w32(o + 0x3c, 0);
    }
    m.w32(o + 0x34, st);
    let t = m.rf(0x002d_ded0);
    m.wf(o + 0x38, t);
    let own = m.r32(o + 0x10);
    match st {
        0 => {
            let s = m.call(0x0020_5438, &[own, 0xb3], &[]).0;
            vslot(m, s, 7, &[], &[]);
        }
        1 => {
            let s = m.call(0x0020_5438, &[own, 0xa4], &[]).0;
            let v = m.r32(o + 0x30);
            m.w32(s + 0x30, v);
            let v = m.r32(o + 0x3c);
            m.w32(s + 0x3c, v);
            let f = m.rf(0x002a_5408);
            let p = m.r32(o + 0x10);
            m.wf(p + 0xf04, f);
            vslot(m, s, 7, &[], &[]);
            let p = m.r32(o + 0x10);
            let a0 = m.r32(p + 0x3cc);
            m.call(0x0020_a9a8, &[a0, 3], &[]);
        }
        2 => {
            let s = m.call(0x0020_5438, &[own, 0xad], &[]).0;
            let v = m.r32(o + 0x30);
            m.w32(s + 0x38, v);
            vslot(m, s, 7, &[], &[]);
            let p = m.r32(o + 0x10);
            let a0 = m.r32(p + 0x3cc);
            m.call(0x0020_a9a8, &[a0, 4], &[]);
        }
        3 => {
            let s = m.call(0x0020_5438, &[own, 0xba], &[]).0;
            m.w32(s + 0xc0, 1);
            m.w32(s + 0x38, 0x1c);
            let a = m.r32(o + 0x30);
            let pos = m.call(0x001a_62e8, &[a], &[]).1;
            let a = m.r32(o + 0x30);
            m.call(0x001a_62e8, &[a], &[]);
            let sp = stack_push(m, 0x20);
            m.call(0x0025_62a8, &[sp], &[]);
            m.call(0x0017_dd38, &[s, sp], &[]);
            stack_pop(m, 0x20);
            let v = m.r32(o + 0x30);
            m.w32(s + 0xac, 1);
            m.w32(s + 0xa8, v);
            vslot(m, s, 7, &[], &[]);
            m.wv(o + 0x50, pos);
            let r = m.call(0x001c_0d70, &[1], &[pos]).0;
            m.w32(o + 0x44, r);
        }
        4 => {
            let s = m.call(0x0020_5438, &[own, 0x9d], &[]).0;
            m.w32(s + 0x3c, 0xd);
            vslot(m, s, 7, &[], &[]);
            let p = m.r32(o + 0x10);
            let a1 = m.r32(o + 0x30);
            let a0 = m.r32(p + 0xf64);
            m.call(0x0015_4038, &[a0, a1, 0], &[]);
            let a0 = m.r32(o + 0x10);
            m.call(0x0020_8540, &[a0, 0], &[]);
        }
        5 => {
            let s = m.call(0x0020_5438, &[own, 0x9e], &[]).0;
            m.w32(s + 0x3c, 1);
            vslot(m, s, 7, &[], &[]);
        }
        _ => {}
    }
}

fn setbit14<M: QMem + ?Sized>(m: &mut M, a: u32, bit: u32, v: u32) {
    let w = (m.r32(a + 0x14) & !(1 << bit)) | ((v & 1) << bit);
    m.w32(a + 0x14, w);
}

/// FUN_001f31e0(o, bis): SO load.
pub fn ee_sub_1f31e0<M: QMem + ?Sized>(m: &mut M, o: u32, bis: u32) {
    let sp = stack_push(m, 0x10);
    m.call(0x0022_d7a0, &[o, bis], &[]);
    let fp = m.call(0x0011_7c80, &[bis], &[]).0;
    m.call(0x0016_7320, &[o + 0x80, o, bis], &[]);
    let extra = vslot(m, o, 62, &[], &[]).0;
    let n = m.call(0x0011_7d50, &[bis], &[]).0;
    m.w32(sp + 4, 0x10);
    m.w32(o + 0xe0, n);
    m.w32(sp, n.wrapping_add(extra).wrapping_mul(0xb0));
    let v = m.call(0x001a_e2a0, &[sp], &[]).0;
    m.w32(o + 0xe4, v);
    let mut i = 0i32;
    while i < m.r32(o + 0xe0) as i32 {
        let e = m.r32(o + 0xe4).wrapping_add((i as u32).wrapping_mul(0xb0));
        m.w32(e + 4, o);
        let k = m.call(0x0011_7d50, &[bis], &[]).0;
        let v = vslot(m, o, 50, &[k], &[]).0;
        m.w32(e, v);
        let v = m.call(0x0011_7db8, &[bis], &[]).0;
        m.w32(e + 0xc, v);
        let v = m.call(0x0011_7d00, &[bis], &[]).0;
        setbit14(m, e, 16, v);
        let v = m.call(0x0011_7d00, &[bis], &[]).0;
        setbit14(m, e, 17, v);
        let f = m.call(0x0011_7e38, &[bis], &[]).1[0];
        m.wf(e + 8, f);
        m.call(0x0011_7fd0, &[bis, e + 0x78], &[]);
        if m.r32(e) == 0 {
            m.w32(e, 0x002c_44f0);
            for k in 0..6 {
                let v = m.r32(e + 0x78 + k * 4);
                m.w32(e + 0x90 + k * 4, v);
            }
        } else {
            m.call(0x0016_6270, &[e + 0x78, 0, e + 0x90], &[]);
        }
        m.call(0x0011_8220, &[bis, e + 0x90, e + 0x68], &[]);
        i += 1;
        let v = m.call(0x0011_7c80, &[bis], &[]).0;
        m.w32(e + 0x18, v);
        let v = m.call(0x0011_7d50, &[bis], &[]).0;
        m.w16(e + 0x14, v as u16);
        let v = m.call(0x0011_7d50, &[bis], &[]).0;
        m.w32(e + 0x1c, v);
        let v = m.call(0x0011_7d00, &[bis], &[]).0;
        setbit14(m, e, 18, v);
    }
    let mut i = m.r32(o + 0xe0) as i32;
    while i < m.r32(o + 0xe0).wrapping_add(extra) as i32 {
        let e = m.r32(o + 0xe4).wrapping_add((i as u32).wrapping_mul(0xb0));
        i += 1;
        m.w16(e + 0x14, 0xfff6);
    }
    m.call(0x001f_2b20, &[o], &[]);
    if (m.r32(o + 0xe0) as i32) > 0 {
        m.w32(sp + 4, 0x10);
        m.w32(sp, 0xe0);
        let p = m.call(0x001a_e2a0, &[sp], &[]).0;
        m.w32(o + 0xe8, p);
        let f = m.call(0x0011_7e38, &[bis], &[]).1[0];
        m.wf(p + 0xd0, f);
        m.call(0x0011_7ed0, &[bis, p], &[]);
        m.call(0x001f_35f8, &[o], &[f4(1.0)]);
        vslot(m, o, 63, &[], &[]);
    }
    m.call(0x0013_f810, &[o, bis], &[]);
    if m.r32(o + 0xb8) != 0 {
        let v = m.r32(o + 0x40);
        if v & 1 != 0 {
            m.w32(o + 0x40, v | 8);
        } else {
            let w = m.r32(v) | 8;
            m.w32(v, w);
        }
    }
    m.call(0x001f_2500, &[o], &[]);
    m.call(0x0013_fb40, &[o, bis, 0], &[]);
    m.call(0x001a_6d28, &[o, fp], &[]);
    stack_pop(m, 0x10);
}

/// FUN_001e67b8(o, pos_out, dir_out): sample the path at the current param, transformed by vslot 30's matrix.
pub fn ee_sub_1e67b8<M: QMem + ?Sized>(m: &mut M, o: u32, s1: u32, s3: u32) {
    let sp = stack_push(m, 0x10);
    let i = m.call(0x001e_7538, &[o, sp, sp + 4], &[]).0;
    let mp = vslot(m, o, 30, &[], &[]).0;
    let mx = [m.rv(mp), m.rv(mp + 0x10), m.rv(mp + 0x20), m.rv(mp + 0x30)];
    let (fa, fb) = (m.rf(sp), m.rf(sp + 4));
    stack_pop(m, 0x10);
    let xf3 = |v: V4| -> [f32; 3] {
        [
            mx[0][0] * v[0] + mx[1][0] * v[1] + mx[2][0] * v[2],
            mx[0][1] * v[0] + mx[1][1] * v[1] + mx[2][1] * v[2],
            mx[0][2] * v[0] + mx[1][2] * v[1] + mx[2][2] * v[2],
        ]
    };
    let mut d: V4;
    if m.r32(o + 0xe0) == 0 {
        if s1 != 0 {
            let t = fa / fb;
            let u = 1.0 - t;
            let p = (i << 4).wrapping_add(m.r32(o + 0xf4));
            let (a, b) = (m.rv(p), m.rv(p + 0x10));
            let l: V4 = [a[0] * u + b[0] * t, a[1] * u + b[1] * t, a[2] * u + b[2] * t, a[3] * u + b[3] * t];
            let r = xf3(l);
            m.wv(s1, [r[0] + mx[3][0], r[1] + mx[3][1], r[2] + mx[3][2], l[3]]);
        }
        if s3 == 0 {
            return;
        }
        let p = (i << 4).wrapping_add(m.r32(o + 0xf4));
        let (a, b) = (m.rv(p), m.rv(p + 0x10));
        d = [b[0] - a[0], b[1] - a[1], b[2] - a[2], b[3] - a[3]];
    } else {
        let off = i << 4;
        let t3 = off.wrapping_add(m.r32(o + 0xf4));
        let a3p = off.wrapping_add(m.r32(o + 0x140));
        let a1p = off.wrapping_add(m.r32(o + 0x144));
        let (q0, q1, q2, q3) = (m.rv(t3), m.rv(a1p), m.rv(t3 + 0x10), m.rv(a3p + 0x10));
        m.call(0x0011_7120, &[s1, s3, 0], &[q0, q1, q2, q3, f4(fb), f4(fa), f4(1.0)]);
        if s1 != 0 {
            let v = m.rv(s1);
            let r = xf3(v);
            m.wv(s1, [r[0] + mx[3][0], r[1] + mx[3][1], r[2] + mx[3][2], mx[0][3]]);
        }
        if s3 == 0 {
            return;
        }
        d = m.rv(s3);
    }
    let r = xf3(d);
    d = [r[0], r[1], r[2], d[3]];
    let q = 1.0 / (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    m.wv(s3, [d[0] * q, d[1] * q, d[2] * q, d[3] * q]);
}

fn vadd(a: V4, b: V4) -> V4 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2], a[3] + b[3]]
}

fn vsub(a: V4, b: V4) -> V4 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2], a[3] - b[3]]
}

/// FUN_001b5e98(jt): pick a target point (+0x1420) from the 0x3e13b4 list in front of the actor.
pub fn ee_sub_1b5e98<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let mut s2 = 0u32;
    let mut f22 = m.rf(o + 0x1410) * m.rf(0x002c_0984);
    let mut s0 = m.r32(0x003e_13b4);
    while s0 != 0 {
        if m.r32(s0 + 0xe8) & 0x10000 != 0 {
            let a0 = m.r32(o + 0x13f0);
            if m.call(0x0018_4380, &[a0, s0], &[]).0 == 0 {
                let a = m.call(0x001a_62e8, &[s0], &[]).1;
                let b = m.call(0x001a_62e8, &[o], &[]).1;
                let d = vsub(a, b);
                let dist = (d[0] * d[0] + d[1] * d[1]).sqrt();
                if !(f22 < dist) && !(dist < 150.0) && !(500.0 < d[2]) {
                    let f20 = m.call(0x0024_58f0, &[], &[f4(d[1]), f4(d[0])]).1[0];
                    let f0 = m.call(0x0020_4740, &[o], &[]).1[0];
                    let r = m.call(0x0011_cf60, &[], &[f4(f20 - f0)]).1[0];
                    if !(m.rf(0x002c_0988) < r.abs()) {
                        s2 = s0;
                        f22 = dist;
                        let p = m.call(0x001a_62e8, &[s2], &[]).1;
                        m.wv(o + 0x1420, p);
                    }
                }
            }
        }
        s0 = m.r32(s0 + 0xe0);
    }
    if s2 == 0 {
        let p = m.call(0x001a_62e8, &[o], &[]).1;
        let (a, b) = (m.rf(o + 0x1a8), m.rf(o + 0x1410));
        let q = m.call(0x0011_e1a0, &[], &[f4(a), f4(b), f4(0.0)]).1;
        m.wv(o + 0x1420, vadd(p, q));
    }
}

/// FUN_0022d148(out, from, to) -> out: accumulate the bone/joint chain transform from `from` up to `to`.
pub fn ee_sub_22d148<M: QMem + ?Sized>(m: &mut M, out: u32, from: u32, to: u32) -> u32 {
    let sp = stack_push(m, 0x140);
    m.call(0x0023_33b4, &[out, 0, 0x40], &[]);
    let mut s3 = from;
    while s3 != to {
        let k = m.r16(s3 + 0x98) as i16 as i32;
        if !matches!(k, 0 | 3 | 10 | 12 | 15 | 19) {
            let s5 = sp + 0x40;
            m.call(0x0022_cdc0, &[s5, s3], &[]);
            for w in 0..16 {
                let v = m.r32(s5 + w * 4);
                m.w32(sp + w * 4, v);
            }
            let par = m.r32(s3 + 0xa0);
            let src = if par != 0 {
                m.call(0x0011_c168, &[sp + 0xc0, par + 0x40, sp], &[]);
                m.call(0x0011_c318, &[sp + 0x100, s3 + 0x40], &[]);
                m.call(0x0011_c168, &[sp + 0x80, sp + 0xc0, sp + 0x100], &[]);
                sp + 0x80
            } else {
                m.call(0x0011_c318, &[sp + 0xc0, s3 + 0x40], &[]);
                m.call(0x0011_c168, &[sp + 0x80, sp, sp + 0xc0], &[]);
                sp + 0x80
            };
            for r in 0..4 {
                let v = vadd(m.rv(out + r * 0x10), m.rv(src + r * 0x10));
                m.wv(s5 + r * 0x10, v);
            }
            m.call(0x0023_320c, &[out, s5, 0x40], &[]);
        }
        s3 = m.r32(s3 + 0xa0);
    }
    if to != 0 {
        m.call(0x0011_c318, &[sp + 0x80, to + 0x40], &[]);
        m.call(0x0011_c168, &[sp + 0x40, sp + 0x80, out], &[]);
        m.call(0x0023_320c, &[out, sp + 0x40, 0x40], &[]);
    }
    stack_pop(m, 0x140);
    out
}

/// FUN_0012f9b8(asd, a1, flags, out, f12, f13): instantiate an animation (ASEG) into a new 0x60 SAA.
pub fn ee_sub_12f9b8<M: QMem + ?Sized>(m: &mut M, asd: u32, a1: u32, flags: u32, out: u32, f12: f32, f13: f32) {
    let sp = stack_push(m, 0x20);
    let s1 = m.call(0x0011_ccc0, &[0x003e_1224], &[]).0;
    m.call(0x0023_33b4, &[s1, 0, 0x60], &[]);
    m.w32(s1, 0);
    m.w32(s1 + 4, 0);
    m.w32(s1 + 0xc, 0x002c_af48);
    m.w32(s1 + 0x5c, 0);
    m.w32(s1 + 0x14, asd);
    m.w32(s1 + 0x18, a1);
    m.wf(s1 + 0x1c, f12);
    m.w32(s1 + 0x3c, flags);
    let f = m.rf(asd + 0x78);
    m.wf(s1 + 0x48, f);
    let f0 = m.call(0x0015_4c90, &[0x002d_e578], &[]).1[0];
    let f1 = f0 * m.rf(asd + 0x88);
    let f2 = m.rf(asd + 0x84) + f1;
    let mut f3 = m.rf(asd + 0x80) + f0 * f2;
    let (lo, hi) = (m.rf(asd + 0x90), m.rf(asd + 0x94));
    if f3 < lo {
        f3 = lo;
    } else if hi < f3 {
        f3 = hi;
    }
    m.wf(s1 + 0x4c, f3);
    m.call(0x0013_1a00, &[s1], &[f4(f13)]);
    m.call(0x0011_a220, &[0x003e_1218, s1], &[]);
    m.call(0x0011_a220, &[asd + 0x50, s1], &[]);
    vslot(m, s1, 0, &[], &[]);
    m.w32(sp, s1);
    vslot(m, s1, 0, &[], &[]);
    let a = m.r32(asd);
    m.call(0x0025_6ee0, &[s1, a], &[]);
    if out != 0 {
        m.w32(out, s1);
    }
    m.w32(sp + 0x18, ((flags as i32) >> 2) as u32);
    m.w32(sp + 0x14, flags & 2);
    let kind = if flags & 1 != 0 { 7 } else { 5 };
    let mut i = 0i32;
    while i < m.r32(asd + 0x5c) as i32 {
        let e = m.r32(asd + 0x60).wrapping_add((i as u32) << 3);
        if m.r32(e) & 0x4000_0000 == 0 {
            m.call(0x0012_fe58, &[asd, a1, e, sp + 4, sp + 8], &[]);
            if m.r32(sp + 8) != 0 {
                let a0 = m.r32(sp + 4);
                m.call(0x001f_8668, &[a0, 0], &[]);
                let p = m.call(0x0012_e1f0, &[0x2c], &[]).0;
                m.w32(p + 0x18, 0x002c_af08);
                m.w32(p, 0);
                let v = m.r32(e + 4);
                if m.r32(e) & 0x8000_0000 != 0 {
                    m.w32(p + 0x20, v);
                } else {
                    m.w32(p + 0x1c, v);
                }
                let v = m.r32(sp + 4);
                m.w32(p + 4, v);
                let v = m.r32(sp + 8);
                m.w32(p + 8, v);
                let v = m.r32(asd + 0x6c);
                m.w32(p + 0x14, kind);
                m.w32(p + 0xc, v);
                m.w32(p + 0x24, s1);
                let v = m.r32(s1 + 0x34);
                m.w32(p + 0x28, v);
                m.w32(s1 + 0x34, p);
                let (x, y) = (m.r32(sp + 4), m.r32(sp + 8));
                m.call(0x0023_0018, &[x, y, p], &[]);
            }
        }
        i += 1;
    }
    let mut i = 0i32;
    while i < m.r32(asd + 0x64) as i32 {
        let e = m.r32(asd + 0x68).wrapping_add((i as u32).wrapping_mul(0xc));
        m.call(0x0013_0010, &[asd, a1, e, sp + 0xc, sp + 0x10], &[]);
        if m.r32(sp + 0x10) != 0 {
            let p = m.call(0x0012_ce50, &[0x14], &[]).0;
            let v = m.r32(e + 8);
            m.w32(p, v);
            let v = m.r32(sp + 0x10);
            m.w32(p + 4, v);
            let v = m.r32(sp + 0xc);
            m.w32(p + 8, v);
            let v = m.r32(e);
            m.w32(p + 0xc, v);
            let v = m.r32(s1 + 0x38);
            m.w32(p + 0x10, v);
            m.w32(s1 + 0x38, p);
            let tgt = m.r32(sp + 0xc);
            let v = m.r32(e);
            vslot(m, tgt, 43, &[v], &[]);
        }
        i += 1;
    }
    let v = m.r32(asd + 0x70);
    m.call(0x0013_1990, &[s1, v], &[]);
    m.wf(s1 + 0x20, f12);
    let mut i = 0i32;
    while i < m.r32(asd + 0x98) as i32 {
        let p = m.r32(asd + 0x9c).wrapping_add((i as u32) << 4);
        i += 1;
        m.call(0x0013_1290, &[s1, p], &[]);
    }
    let b = m.r32(sp + 0x18) & 1;
    let f = m.rf(s1 + 0x20);
    m.wf(s1 + 0x1c, f);
    m.call(0x0023_0cf0, &[s1, b], &[]);
    if m.r32(sp + 0x14) != 0 {
        if m.rf(s1 + 0x40) < 0.0 {
            let v = m.r32(asd + 0xa0);
            m.w32(s1 + 0x50, v);
        }
        m.call(0x0013_1100, &[s1], &[]);
    } else {
        m.call(0x0013_0eb0, &[s1], &[]);
    }
    vslot(m, s1, 2, &[2, 0, 0, 0, 0], &[]);
    let s = m.r32(sp);
    vslot(m, s, 1, &[], &[]);
    stack_pop(m, 0x20);
}

/// FUN_001a5478(lo): detach from parent / remove from world (bit 56 of +0x28).
pub fn ee_sub_1a5478<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let hi = m.r32(o + 0x2c);
    if hi & (1 << 24) == 0 {
        return;
    }
    m.w32(o + 0x2c, hi & !(1 << 24));
    let sp = stack_push(m, 0x10);
    let par = m.r32(o + 0x10);
    m.call(0x001a_5448, &[par, sp, sp + 4], &[]);
    let pp = m.r32(sp + 4);
    if o == m.r32(pp) {
        let v = m.r32(o + 0x24);
        m.w32(pp, v);
    } else {
        let a0 = m.r32(sp);
        m.call(0x0011_a2e8, &[a0, o], &[]);
        let a0 = m.r32(sp);
        let pp = m.r32(sp + 4);
        let a1 = m.r32(pp);
        m.call(0x0011_a280, &[a0, a1, o], &[]);
    }
    let par = m.r32(o + 0x10);
    if par == 0 || (m.r32(par + 0x2c) >> 28) & 1 != 0 {
        vslot(m, o, 10, &[], &[]);
    }
    if m.r32(o + 0x10) == 0 && (m.r32(o + 0x2c) >> 20) & 0xf != 2 {
        let a1 = m.r32(o + 0x14);
        m.call(0x0014_7e88, &[GS, a1], &[]);
    }
    let n = vslot(m, o, 27, &[sp + 8], &[]).0 as i32;
    let mut i = 0i32;
    while i < n {
        let arr = m.r32(sp + 8);
        let c = m.r32(((i as u32) << 2).wrapping_add(arr));
        i += 1;
        vslot(m, c, 7, &[], &[]);
    }
    if m.call(0x001a_61a0, &[o], &[]).0 != 0 {
        vslot(m, o, 22, &[], &[]);
    } else if m.r32(o + 0x10) == 0 {
        m.call(0x0014_8300, &[GS, o], &[]);
    }
    stack_pop(m, 0x10);
}

/// FUN_00225300(o): state-5 update (carry / hand-off to the +0x1b4 actor).
pub fn ee_sub_225300<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0022_40b0, &[o], &[]);
    let mut s3 = m.r32(o + 0x150);
    loop {
        if s3 == 5 {
            let a0 = m.r32(o + 0x1b4);
            if a0 == 0 {
                vslot(m, o, 75, &[3], &[]);
            } else {
                let s1 = m.call(0x0020_5458, &[a0, 0xb0], &[]).0;
                if s1 == 0 || m.call(0x0017_a9c0, &[s1], &[]).0 != 0 {
                    vslot(m, o, 75, &[2], &[]);
                } else {
                    let a = m.r32(o + 0x1b0);
                    let a = if a == 0 { o } else { a };
                    let p = m.call(0x001a_6318, &[a], &[]).0;
                    let sp = stack_push(m, 0x30);
                    for k in 0..3 {
                        let v = m.rv(p + k * 0x10);
                        m.wv(sp + k * 0x10, v);
                    }
                    let t = m.r32(o + 0x1b4);
                    m.call(0x0020_9840, &[t + 0xfd0, sp], &[]);
                    stack_pop(m, 0x30);
                    if m.r32(s1 + 0x30) != 4 {
                        let a = m.r32(o + 0x1b0);
                        let a = if a == 0 { o } else { a };
                        let pa = m.call(0x001a_62e8, &[a], &[]).1;
                        let t = m.r32(o + 0x1b4);
                        let pb = m.call(0x001a_62e8, &[t], &[]).1;
                        let d = vsub(pa, pb);
                        let dist = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                        if dist < 100.0 {
                            m.call(0x0017_a8c8, &[s1, 4], &[]);
                        }
                    }
                    let v1 = m.r32(o + 0x1b4);
                    if m.r32(v1 + 0xfc4) != 0 {
                        let f0 = (m.rf(0x002d_ded0) - m.rf(o + 0x154)) / m.rf(0x002c_43b0);
                        let mut f4v =
                            m.rf(0x002c_43c0) + f0 * (m.rf(0x002c_43c4) + f0 * m.rf(0x002c_43c8));
                        let (lo, hi) = (m.rf(0x002b_bb60), m.rf(0x002b_bb64));
                        if f4v < lo {
                            f4v = lo;
                        } else if hi < f4v {
                            f4v = hi;
                        }
                        let a0 = m.r32(v1 + 0xfc4);
                        m.call(0x0012_d108, &[a0], &[[f4v; 4]]);
                    }
                    let el = m.rf(0x002d_ded0) - m.rf(o + 0x154);
                    if m.rf(0x002c_43b0) < el {
                        let a = m.r32(o + 0x1b0);
                        let pos = m.call(0x001a_62e8, &[a], &[]).1;
                        m.call(0x001c_6fd8, &[GS, 0x5d, o, 1], &[pos]);
                        let t = m.r32(o + 0x1b4);
                        vslot(m, t, 8, &[], &[]);
                        let t = m.r32(o + 0x1b4);
                        m.call(0x0016_19a0, &[0x002d_eb30, 0x18, t], &[]);
                        vslot(m, o, 75, &[3], &[]);
                    }
                }
            }
        }
        let v = m.r32(o + 0x150);
        if v == s3 {
            return;
        }
        s3 = v;
    }
}
