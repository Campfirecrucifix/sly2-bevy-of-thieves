//! WIRE-S2: 1:1 QMem ports of live-frontier natives (shard 3/4).
#![allow(unused)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

// SKIP 0x1a19a0: 64-bit pointer-to-member args (ldl/ldr), quadword vslot returns (pextuw), 0x60-byte quad copies; too heavy for 1:1 here.
// SKIP 0x202378: VU0 macro heavy (vsqrt/vdiv/vopmula cross products) with quadword args to 0x20faa0/0x14cc58.
// SKIP 0x19a4a0: VU0 macro + 128-bit GPR args/returns (pextuw/por) to 0x22d348/0x19a400/0x117120.

/// FUN_0021e718: printf conversion dispatch on +0x2c (jump table 0x2a9530, chars 0x25..0x78).
pub fn ee_sub_21e718<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, t0: u32) {
    let (s1, s2, t1) = (a0, a1, a2);
    let ch = m.r8(s1 + 0x2c) as u32;
    let next = |m: &mut M| -> u32 {
        let v = m.r32(t1);
        m.w32(t1, v.wrapping_add(8));
        v
    };
    let put = |m: &mut M, s: u32| {
        let n = m.call(0x0021_ed28, &[s], &[]).0;
        m.call(0x0020_d660, &[s2, s, n], &[]);
    };
    match ch {
        0x25 => {
            m.call(0x0020_d660, &[s2, s1 + 0x2c, 1], &[]);
        }
        0x63 => {
            let v = next(m);
            let b = m.r8(v);
            let sp = stack_push(m, 0x10);
            m.w8(sp, b);
            m.call(0x0020_d660, &[s2, sp, 1], &[]);
            stack_pop(m, 0x10);
        }
        0x73 => {
            let v = next(m);
            let s = m.r32(v);
            m.call(0x0021_d9f0, &[s1, s2, s], &[]);
        }
        0x62 | 0x64 | 0x69 | 0x6f | 0x70 | 0x75 | 0x78 => {
            if ch == 0x70 {
                m.w32(s1 + 0x1c, 8);
                m.w8(s1 + 0x2c, 0x78);
                m.w32(s1 + 0x20, 0);
            }
            let sz = m.r32(s1 + 0x20) as i32;
            // t2 uninitialised for sz < 0 / sz > 2: modelled as 0
            let (lo, hi) = if sz == 0 || sz == 1 {
                let v = next(m);
                (m.r32(v), 0)
            } else if sz == 2 {
                let v = next(m);
                (m.r32(v), m.r32(v + 4))
            } else {
                (0, 0)
            };
            m.call(0x0021_dd40, &[s1, s2], &[[f32::from_bits(lo), f32::from_bits(hi), 0.0, 0.0]]);
        }
        0x65 | 0x66 | 0x67 => {
            if m.r32(s1 + 0x24) == 0 {
                let v = next(m);
                let (lo, hi) = (m.r32(v), m.r32(v + 4));
                m.call(0x0021_e4b8, &[s1, s2], &[[f32::from_bits(lo), f32::from_bits(hi), 0.0, 0.0]]);
                return;
            }
            let a = next(m);
            let mut s3 = m.r32(a);
            let (s7, fp, s5, s6) = (0x002a_9510u32, 0x002a_9508u32, 0x002a_9518u32, 0x002a_9520u32);
            if m.r32(s1 + 0x28) as i32 >= 2 {
                put(m, 0x002a_9500);
            }
            let mut s0: i32 = 0;
            if m.r32(s1 + 0x28) as i32 > 0 {
                loop {
                    if s0 != 0 {
                        put(m, fp);
                    }
                    let s4 = s0 + 1;
                    put(m, s7);
                    let mut j: i32 = 0;
                    if m.r32(s1 + 0x24) as i32 > 0 {
                        loop {
                            if j != 0 {
                                put(m, s5);
                            }
                            let f = m.rf(s3);
                            j += 1;
                            s3 = s3.wrapping_add(4);
                            // 64-bit double return: only the low word crosses QMem::call
                            let d = m.call(0x0023_e290, &[], &[[f, 0.0, 0.0, 0.0]]).0;
                            m.call(0x0021_e4b8, &[s1, s2], &[[f32::from_bits(d), 0.0, 0.0, 0.0]]);
                            if j >= m.r32(s1 + 0x24) as i32 {
                                break;
                            }
                        }
                    }
                    put(m, s6);
                    s0 = s4;
                    if s0 >= m.r32(s1 + 0x28) as i32 {
                        break;
                    }
                }
            }
            if m.r32(s1 + 0x28) as i32 >= 2 {
                put(m, 0x002a_9500);
            }
        }
        0x6e => {
            let k = m.r32(s1 + 0x20);
            if k > 2 {
                return;
            }
            let v = next(m);
            let p = m.r32(v);
            let c = m.r32(s2 + 0x10);
            match k {
                0 => m.w32(p, c),
                1 => m.w16(p, c as u16),
                _ => {
                    m.w32(p, c);
                    m.w32(p + 4, if (c as i32) < 0 { u32::MAX } else { 0 });
                }
            }
        }
        _ => {
            m.call(0x0020_d660, &[s2, a3, t0], &[]);
        }
    }
}

/// FUN_001f90d0: collect matching child nodes of each object into out[], up to max; returns count.
pub fn ee_sub_1f90d0<M: QMem + ?Sized>(m: &mut M, a2: u32, a3: u32, t0: u32, t1: u32) -> u32 {
    let mut s3: i32 = 0;
    let mut s7 = a3;
    let mut i: i32 = 0;
    while i < a2 as i32 {
        let s4 = m.r32(s7);
        let f = m.r32(s4 + 0x148) | 0x1000_0000;
        m.w32(s4 + 0x148, f);
        let e = m.r32(s4 + 0xec);
        let mut s1 = m.r32(e + 0x64);
        loop {
            let s0start;
            loop {
                if s1 == 0 {
                    let e = m.r32(s4 + 0xec);
                    s0start = m.r32(e + 0x68);
                    break;
                }
                let o = m.r32(s1 + 0xc);
                if m.r32(o + 0x148) & 0x1000_0000 != 0 {
                    s1 = m.r32(s1 + 4);
                    continue;
                }
                s0start = m.r32(s1);
                break;
            }
            let mut s0 = s0start;
            let mut s2 = t1.wrapping_add((s3 as u32) << 2);
            while s0 != 0 {
                let k = m.r32(s0 + 0x9c);
                let mut take = k != 13;
                if k == 14 {
                    let o = m.r32(s0 + 8);
                    take = vslot(m, o, 84, &[], &[]).0 & 4 != 0;
                }
                if take {
                    if s3 >= t0 as i32 {
                        return s3 as u32;
                    }
                    m.w32(s0 + 0xa8, s3 as u32);
                    m.w32(s2, s0);
                    s3 += 1;
                    s2 = s2.wrapping_add(4);
                }
                s0 = m.r32(s0 + 0xac);
            }
            if s1 == 0 {
                break;
            }
            s1 = m.r32(s1 + 4);
        }
        i += 1;
        s7 = s7.wrapping_add(4);
    }
    s3 as u32
}

fn q2558c0_find<M: QMem + ?Sized>(m: &mut M, x: u32) -> (u32, u32) {
    let n = m.r32(0x002d_d2b4);
    for i in 0..n {
        let b = m.r32(0x0045_2110 + i.wrapping_mul(4));
        if x < b {
            continue;
        }
        if x < b.wrapping_add(0xe000) {
            let q = (x.wrapping_sub(b).wrapping_mul(0xb6db_6db7) as i32) >> 2;
            return (q as u32, i);
        }
    }
    (0, 0)
}

/// FUN_002558c0: link node a0 after *a1, packing pool indices into the +4 words; *a1 = a0.
pub fn ee_sub_2558c0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let (i0, p0) = q2558c0_find(m, a0);
    let h = m.r32(a1);
    if h != 0 {
        let v = (m.r32(h + 4) & 0xffff_e000) | (i0 & 0x1fff);
        m.w32(h + 4, v);
        let v = (v & 0xffff_9fff) | ((p0 & 3) << 13);
        m.w32(h + 4, v);
    }
    let h = m.r32(a1);
    let (i1, p1) = q2558c0_find(m, h);
    let mut v = m.r32(a0 + 4) & 0xe000_ffff;
    v |= (i1 & 0x1fff) << 16;
    v = (v & 0x9fff_ffff) | ((p1 & 3) << 29);
    m.w32(a0 + 4, v);
    m.w32(a1, a0);
}

/// FUN_00243870: if FUN_00243270(..)+0x72 == 1, copy 6 bytes into slot entry, return FUN_00242f70(a0,a1) == 1.
pub fn ee_sub_243870<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) -> u32 {
    let v = m.call(0x0024_3270, &[a0, a1, a2], &[]).0;
    if m.r8(v + 0x72) != 1 {
        return 0;
    }
    let e = 0x0045_05d0u32.wrapping_add(a1.wrapping_mul(0x1c)).wrapping_add(a0.wrapping_mul(0x70));
    let p = m.r32(e + 4);
    for i in 0..6 {
        let b = m.r8(a2 + i);
        m.w8(p + 0xc + i, b);
    }
    m.w32(p + 4, 1);
    m.w32(p + 8, 6);
    (m.call(0x0024_2f70, &[a0, a1], &[]).0 == 1) as u32
}

/// FUN_001d5340: if +8 and *a1 > 0 and FUN_001d55b8 -> set up +0xc/+0x10/+0x14/+0x18, +8 = -1, FUN_001d53c0(a0, 2).
pub fn ee_sub_1d5340<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, f12: f32) {
    if m.r32(a0 + 8) == 0 || m.r32(a1) as i32 <= 0 {
        return;
    }
    if m.call(0x001d_55b8, &[a0, a1], &[[f12, 0.0, 0.0, 0.0]]).0 == 0 {
        return;
    }
    m.wf(a0 + 0x14, f12);
    m.w32(a0 + 0xc, a1);
    m.w32(a0 + 0x10, 0);
    let f = m.rf(a1 + 8);
    m.w32(a0 + 8, u32::MAX);
    m.wf(a0 + 0x18, f);
    m.call(0x001d_53c0, &[a0, 2], &[]);
}

/// FUN_001a3460: FUN_001a5e80(a0); k = +0x50 ? +0x50[+0xa4] : 0; +0xe4 < k -> +0xe4 = k, FUN_001a3968(a0).
pub fn ee_sub_1a3460<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5e80, &[a0], &[]);
    let p = m.r32(a0 + 0x50);
    let k = if p != 0 { m.r32(p + 0xa4) } else { 0 };
    if (m.r32(a0 + 0xe4) as i32) < k as i32 {
        m.w32(a0 + 0xe4, k);
        m.call(0x001a_3968, &[a0], &[]);
    }
}

/// FUN_001d5310: FUN_001d5340(a0, 0x2c2260 + a1 * 0x104, f12).
pub fn ee_sub_1d5310<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, f12: f32) {
    let p = a1.wrapping_mul(0x104).wrapping_add(0x002c_2260);
    m.call(0x001d_5340, &[a0, p], &[[f12, 0.0, 0.0, 0.0]]);
}

/// FUN_001a50f8: FUN_001a4fb0(a0, 5).
pub fn ee_sub_1a50f8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_4fb0, &[a0, 5], &[]);
}
