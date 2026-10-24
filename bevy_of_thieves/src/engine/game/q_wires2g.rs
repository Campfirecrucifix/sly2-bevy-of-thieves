//! WIRE-S2: 1:1 QMem ports of live-frontier natives (shard 3/4).
#![allow(unused)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

const GS: u32 = 0x003d_d490;

// SKIP 0x202d38: 2 KB body with a computed `jr $a0` jump table; not ported here.

/// FUN_001a9108: widening ray probe (f20 doubles up to 52000), marks/clears 0x1000 hit flags, emits nearest hit.
pub fn ee_sub_1a9108<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, t0: u32, q: V4, f12: f32) {
    let b = stack_push(m, 0x700);
    let mut f20 = if f12 > 100.0 { f12 } else { 100.0 };
    m.call(0x001a_e478, &[b], &[]);
    m.w32(b + 0x10, 0xfffc);
    m.w32(b + 0x14, 0x10);
    let s2 = m.call(0x001a_e318, &[b + 0x10], &[]).0;
    while f20 < 52000.0 {
        m.w32(b + 0x624, s2);
        m.w32(b + 0x620, 0x1555);
        let r = m.call(0x0011_8a88, &[a0, a1, b + 0x620, b + 0x624], &[q, f4(f20)]).0 as i32;
        if r == 2 {
            let diff = m.r32(b + 0x624).wrapping_sub(s2);
            let n = (diff.wrapping_mul(0xaaaa_aaab) as i32) >> 2;
            m.w32(b + 0x620, n as u32);
            let mut f21 = 0.0f32;
            let mut i = 0i32;
            while i < m.r32(b + 0x620) as i32 {
                let e = m.r32(s2 + 8 + 12 * i as u32);
                let fl = m.r32(e + 0x18);
                if fl & 0x1000 == 0 {
                    let s = f20 + 1.0;
                    m.w32(e + 0x18, fl | 0x1000);
                    let v = m.rv(e);
                    let p: V4 = [q[0] + v[0] * s, q[1] + v[1] * s, q[2] + v[2] * s, q[3] + v[3] * s];
                    let a0x = m.r32(b + 0x624).wrapping_sub(0xc);
                    let hit = m.call(0x0011_8668, &[a0x, 0x7fff_ffff, 0x10, b + 0x20], &[p, q]).0 as i32;
                    if hit > 0 {
                        let f0 = m.rf(b + 0x60);
                        if f21 < f0 {
                            f21 = f0;
                            let d: V4 = [p[0] - q[0], p[1] - q[1], p[2] - q[2], p[3] - q[3]];
                            let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                            let k = 1.0 / len;
                            let dir: V4 = [d[0] * k, d[1] * k, d[2] * k, d[3] * k];
                            let h = m.rv(b + 0x20);
                            m.wv(t0, h);
                            m.call(0x0011_9390, &[t0 + 0x10], &[dir, h]);
                            m.w32(t0 + 0x20, 0xc);
                        }
                    }
                }
                i += 1;
            }
            let mut i = 0i32;
            while i < m.r32(b + 0x620) as i32 {
                let e = m.r32(s2 + 8 + 12 * i as u32);
                let fl = m.r32(e + 0x18);
                m.w32(e + 0x18, fl & !0x1000);
                i += 1;
            }
            if 0.0 < f21 {
                m.call(0x001a_e4a0, &[b, 2], &[]);
                stack_pop(m, 0x700);
                return;
            }
        }
        f20 = f20 + f20;
    }
    m.call(0x001a_e4a0, &[b, 2], &[]);
    stack_pop(m, 0x700);
}

/// FUN_001fd830: per-frame update: time stamp, sub-updates, enable/disable 3 children on mode, 3x FUN_001fea60, child list, FUN_00231c48.
pub fn ee_sub_1fd830<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let lo = m.r32(a0 + 0x1f0);
    let hi = m.r32(a0 + 0x1f4);
    m.w32(a0 + 0x1f8, lo);
    m.w32(a0 + 0x1fc, hi);
    let v = m.call(0x0014_d240, &[0x002d_ded0], &[]).0;
    m.w32(a0 + 0x1f0, v);
    m.w32(a0 + 0x1f4, ((v as i32) >> 31) as u32);
    m.call(0x001f_da48, &[a0], &[]);
    m.call(0x001f_dba8, &[a0], &[]);
    let n = m.r32(a0 + 0x204) as i32;
    let a1 = if n > 0 { m.r32(a0.wrapping_add((n as u32) << 2) + 0x204) } else { 0 };
    let on = m.r32(a0 + 0x58);
    if m.r32(0x002b_d4c0) == 3 {
        if on != 0 {
            if a1 != 0 {
                m.call(0x001f_b060, &[a1], &[]);
            }
            let c = m.r32(a0 + 0x228);
            if c != 0 {
                m.call(0x001f_b060, &[c], &[]);
            }
            let c = m.r32(a0 + 0x200);
            if c != 0 {
                m.call(0x001f_b060, &[c], &[]);
            }
            m.w32(a0 + 0x58, 0);
        }
    } else if on == 0 {
        if a1 != 0 {
            m.call(0x001f_b018, &[a1], &[]);
        }
        let c = m.r32(a0 + 0x228);
        if c != 0 {
            m.call(0x001f_b018, &[c], &[]);
        }
        let c = m.r32(a0 + 0x200);
        if c != 0 {
            m.call(0x001f_b018, &[c], &[]);
        }
        m.w32(a0 + 0x58, 1);
    }
    let mut f21 = 1.0f32;
    m.call(0x001f_dc08, &[a0], &[]);
    m.call(0x001f_dec0, &[a0], &[]);
    m.call(0x001f_dec8, &[a0], &[]);
    let sp = stack_push(m, 0x10);
    for k in 0..3 {
        let w = m.r32(0x002a_8288 + 4 * k);
        m.w32(sp + 4 * k, w);
    }
    if m.call(0x001f_e5e0, &[a0], &[]).0 != 0 && m.rf(0x002d_ded4) == 0.0 {
        f21 = 0.0;
    }
    let f12 = m.rf(a0 + 0x15c);
    let f14 = m.rf(0x002d_df10);
    let f20 = m.call(0x0011_d018, &[sp, 0], &[f4(f12), f4(f21), f4(f14)]).1[0];
    stack_pop(m, 0x10);
    m.call(0x001f_ea60, &[a0, 0, 2], &[f4(f20)]);
    m.call(0x001f_ea60, &[a0, 1, 2], &[f4(f20)]);
    m.call(0x001f_ea60, &[a0, 2, 2], &[f4(f21)]);
    let mut s = m.r32(a0 + 0x4c);
    while s != 0 {
        let nx = m.r32(s + 0x38);
        m.call(0x001f_b8e0, &[s], &[]);
        s = nx;
    }
    m.call(0x0023_1c48, &[0], &[]);
}

/// FUN_001488d8: re-bind slot a1's entries (FUN_00130598), then FUN_0020ff70; slot 0 also FUN_001dfd58/FUN_00161f48.
pub fn ee_sub_1488d8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let off = a1.wrapping_mul(0x44);
    let s7 = a0 + 4 + off;
    let mut s2 = a0 + 0xc + off;
    let mut i = 0i32;
    if (m.r32(s7) as i32) > 0 {
        loop {
            let s1 = m.r32(s2 - 4);
            if s1 != 0 {
                let v0 = m.r32(s1 + 0x14);
                let mut x = m.r32(a0);
                let v1 = m.r32(v0);
                let ax = m.r32(x + 0x18);
                let a6 = m.r32(v1 + 0x18);
                let s3 = m.r32(s2);
                if ax != a6 {
                    x = if a6 == 0xffff_ffff { 0 } else { m.call(0x0015_e938, &[GS, 4, a6, x], &[]).0 };
                }
                let f13 = m.rf(a0 + 0x8c);
                let a2 = if m.r32(a0 + 0xa0) != 0 { if f13 < 0.0 { 2 } else { 0 } } else { 1 };
                m.call(0x0013_0598, &[s1, x, a2, s3], &[f4(0.0), f4(f13)]);
            }
            i += 1;
            s2 += 8;
            if i >= m.r32(s7) as i32 {
                break;
            }
        }
    }
    let k = m.r32(a0 + 0xb0);
    let o = m.r32(a0);
    m.call(0x0020_ff70, &[GS, k, a0 + 0xb4, o, (a1 == 0) as u32], &[]);
    if a1 == 0 {
        let c = m.r32(a0 + 0xac);
        if c != 0 {
            let d = m.r32(a0 + 0xa8);
            m.call(0x001d_fd58, &[c, d], &[]);
        }
        let k = m.r32(a0 + 0x134);
        let o = m.r32(a0);
        m.call(0x0016_1f48, &[0x002d_eb30, k, a0 + 0x138, o], &[]);
    }
}

fn setbit<M: QMem + ?Sized>(m: &mut M, a0: u32, row: u32, bit: u32) {
    let b = bit as i32;
    let w = (if b > -1 { b } else { b.wrapping_add(31) }) >> 5;
    let a = (row << 5).wrapping_add(m.r32(a0 + 0x4564)).wrapping_add((w << 2) as u32);
    let v = m.r32(a);
    m.w32(a, v | (1u32 << (bit & 31)));
}

/// FUN_001a3b80: BSP plane walk: d = n.xyz.p - w; recurse/mark leaf bits on each side within radius f12.
pub fn ee_sub_1a3b80<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a3: u32, q: V4, f12: f32) {
    let sp = stack_push(m, 0x20);
    let n = m.rv(a1);
    m.call(0x0025_64b8, &[sp], &[]);
    m.call(0x0025_64b8, &[sp + 0x10], &[]);
    stack_pop(m, 0x20);
    let f21 = n[0] * q[0] + n[1] * q[1] + n[2] * q[2] - n[3];
    if f21 < f12 {
        let c = m.r32(a1 + 0x10);
        if c != 0 {
            m.call(0x001a_3b80, &[a0, c, a3], &[q, f4(f12)]);
        } else {
            let row = m.r32(a1 + 0x18);
            setbit(m, a0, row, a3);
        }
    }
    if -f12 < f21 {
        let c = m.r32(a1 + 0x14);
        if c != 0 {
            m.call(0x001a_3b80, &[a0, c, a3], &[q, f4(f12)]);
        } else {
            let row = m.r32(a1 + 0x1c);
            setbit(m, a0, row, a3);
        }
    }
}

/// FUN_00140db0: FUN_001fad00 wrapper that scales a2+0x40 by 0.01 for certain targets, restoring it after.
pub fn ee_sub_140db0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, t0: u32) {
    let saved = m.rv(a2 + 0x40);
    let e0 = m.r32(a2 + 0x50); let e = m.r32(e0.wrapping_add(1u32.wrapping_sub(t0) << 6) + 8);
    let hit = e == m.r32(0x002e_1e40) || e == m.r32(0x002d_d5bc) || e == m.r32(0x002f_7900);
    let p150 = m.r32(a0 + 0x150);
    if hit && (m.r32(p150 + 0x80) != 1 || m.call(0x0014_4068, &[a0], &[]).0 != 0) {
        let k = f32::from_bits(0x3c23_d70a);
        m.wv(a2 + 0x40, [saved[0] * k, saved[1] * k, saved[2] * k, saved[3] * k]);
    }
    m.call(0x001f_ad00, &[a0, a1, a2, a3, t0], &[]);
    m.wv(a2 + 0x40, saved);
}

/// FUN_001487a0: vslot 2(5,0,0,0,0); +0x9c == 0 -> FUN_001fb2f8(FUN_001fbb90(2), *a0); FUN_001488d8(a0, 0); +0xa4 == 0 -> FUN_00148a78(a0, 2); vslot 3(6, a0).
pub fn ee_sub_1487a0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let o = m.r32(a0);
    vslot(m, o, 2, &[5, 0, 0, 0, 0], &[]);
    if m.r32(a0 + 0x9c) == 0 {
        let s0 = m.r32(a0);
        let h = m.call(0x001f_bb90, &[2], &[]).0;
        m.call(0x001f_b2f8, &[h, s0], &[]);
    }
    m.call(0x0014_88d8, &[a0, 0], &[]);
    if m.r32(a0 + 0xa4) == 0 {
        m.call(0x0014_8a78, &[a0, 2], &[]);
    }
    let o = m.r32(a0);
    vslot(m, o, 3, &[6, a0], &[]);
}

/// FUN_001a4e08: init: clear +0, +0x1c4; FUN_001a5020/1a50f8/1a5078/1a4e80.
pub fn ee_sub_1a4e08<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.w32(a0, 0);
    m.w32(a0 + 0x1c4, 0);
    m.call(0x001a_5020, &[a0, 0x002a_3868], &[]);
    m.call(0x001a_50f8, &[a0], &[]);
    m.call(0x001a_5078, &[a0, 0x002a_38a8, 0x189, 1], &[]);
    m.call(0x001a_4e80, &[a0], &[]);
}

/// FUN_0018f050: toggle +0x18d8 between 0x16/0x17; else FUN_0018ed98 ? 0x17 : 0x16.
pub fn ee_sub_18f050<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    match m.r32(a0 + 0x18d8) {
        0x16 => 0x17,
        0x17 => 0x16,
        _ => {
            if m.call(0x0018_ed98, &[a0], &[]).0 != 0 { 0x17 } else { 0x16 }
        }
    }
}

/// FUN_0017f1a0: FUN_001f4fb0(a0, a1); a1+0x98 = a1+0x94 = 0.
pub fn ee_sub_17f1a0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.call(0x001f_4fb0, &[a0, a1], &[]);
    m.w32(a1 + 0x98, 0);
    m.w32(a1 + 0x94, 0);
}

/// FUN_00240b68: tail wrapper -> FUN_00234d40(a0..a3).
pub fn ee_sub_240b68<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32) -> u32 {
    m.call(0x0023_4d40, &[a0, a1, a2, a3], &[]).0
}
