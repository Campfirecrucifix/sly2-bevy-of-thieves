//! WIRE-W1H8: 1:1 queue ports (against `QMem`) of a batch of EE functions, ported from the asm;
//! callees go through `m.call` / `vslot`.
#![allow(unused_variables, unused_mut, unused_assignments, clippy::all)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

const GS: u32 = 0x003d_d490; // game state / world

/// FUN_0014fd48: empty.
pub fn ee_sub_14fd48<M: QMem + ?Sized>(m: &mut M) {}

/// FUN_0013c478(o): +0x158 ? *(+0x158) : -1.
pub fn ee_sub_13c478<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let p = m.r32(a0 + 0x158);
    if p == 0 {
        return 0xffff_ffff;
    }
    m.r32(p)
}

/// FUN_001e6630(o): ((f32*)+0xec)[+0xe8 - 1].
pub fn ee_sub_1e6630<M: QMem + ?Sized>(m: &mut M, a0: u32) -> f32 {
    let n = m.r32(a0 + 0xe8);
    let b = m.r32(a0 + 0xec);
    m.rf((n << 2).wrapping_add(b).wrapping_sub(4))
}

/// FUN_00256c38(dst, p): if p: ++*p; *dst = p.
pub fn ee_sub_256c38<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if a1 != 0 {
        let v = m.r32(a1).wrapping_add(1);
        m.w32(a1, v);
    }
    m.w32(a0, a1);
}

/// FUN_00165dc0(o, p): +0x3b0c = p; global 0x2deb40 = p ? p[0x18] : -1.
pub fn ee_sub_165dc0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.w32(a0 + 0x3b0c, a1);
    if a1 != 0 {
        let v = m.r32(a1 + 0x18);
        m.w32(0x002d_eb40, v);
    } else {
        m.w32(0x002d_eb40, 0xffff_ffff);
    }
}

pub fn ee_sub_1836c8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0023_1100, &[a0], &[]);
    m.call(0x0018_2c70, &[a0 + 0xe0], &[]);
}

pub fn ee_sub_1d1910<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001c_b518, &[a0], &[]);
    m.call(0x001d_14c0, &[a0], &[]);
}

pub fn ee_sub_1a3430<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5800, &[a0], &[]);
    m.call(0x001a_3968, &[a0], &[]);
}

pub fn ee_sub_1a3400<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5c48, &[a0], &[]);
    m.call(0x001a_3968, &[a0], &[]);
}

pub fn ee_sub_180638<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0017_f170, &[a0], &[]);
    m.call(0x0018_1268, &[a0, 0], &[]);
}

/// FUN_0014b1a0(o): FUN_00205eb8(o); global 0x2dd648 = o; +0x4a4 = 1.
pub fn ee_sub_14b1a0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0020_5eb8, &[a0], &[]);
    m.w32(0x002d_d648, a0);
    m.w32(a0 + 0x4a4, 1);
}

/// FUN_001f03c8(o, id): walk list +0x68 via +0x14 for node with +0x28 == id.
pub fn ee_sub_1f03c8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let mut p = m.r32(a0 + 0x68);
    while p != 0 {
        if m.r32(p + 0x28) == a1 {
            return p;
        }
        p = m.r32(p + 0x14);
    }
    0
}

pub fn ee_sub_1a3518<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_6178, &[a0], &[]);
    m.call(0x001a_3d00, &[GS, a0], &[]);
}

pub fn ee_sub_19c9e8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5800, &[a0], &[]);
    m.call(0x0011_a220, &[0x003e_1420, a0], &[]);
}

pub fn ee_sub_1bc050<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001e_5d80, &[a0], &[]);
    m.w32(a0 + 0x1e8, 0xffff_ffff);
    m.call(0x0011_a1f8, &[a0 + 0x1c4, 0x1d0], &[]);
}

pub fn ee_sub_185ce0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5c48, &[a0], &[]);
    let p = m.r32(a0 + 0x110);
    if p != 0 {
        m.call(0x0022_1178, &[p, 0], &[]);
    }
}

pub fn ee_sub_132100<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001f_36a8, &[a0], &[]);
    if m.r32(a0 + 0xe8) != 0 {
        m.call(0x0013_4320, &[GS, a0], &[]);
    }
}

pub fn ee_sub_225aa8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001f_36a8, &[a0], &[]);
    if m.r32(a0 + 0x1cc) != 0 {
        m.w32(a0 + 0x1cc, 0);
        m.call(0x0022_6370, &[a0, 1], &[]);
    }
}

pub fn ee_sub_1618b0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let s = m.r32(a0 + 0x3bb8);
    if s != 0 {
        vslot(m, s, 48, &[0xffff_ffff], &[]);
        m.call(0x0021_2a30, &[s], &[]);
    }
}

pub fn ee_sub_1b2070<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001b_1db8, &[a0], &[]);
    m.wf(a0 + 0x270, f32::from_bits(0x44fa_0000));
    m.wf(a0 + 0x274, f32::from_bits(0x459c_4000));
    m.w32(a0 + 0x264, 0);
}

pub fn ee_sub_152100<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0020_3a38, &[a0], &[]);
    m.call(0x0015_2578, &[a0], &[]);
    vslot(m, a0, 78, &[0], &[]);
}

/// ANIM-ALL R78: a1 (the orientation matrix) passes through to FUN_001f42e0 as in the original (it was
/// dropped: FUN_001f42e0 read rows at address 0 -> NaN JT bone 2 local -> NaN guest JT worlds).
pub fn ee_sub_2041f8<M: QMem + ?Sized>(m: &mut M, a0: u32, mat: u32) {
    m.call(0x001f_42e0, &[a0, mat], &[]);
    let f = m.call(0x0020_4740, &[a0], &[]).1[0];
    m.call(0x0020_4660, &[a0], &[f4(f)]);
    let p = m.r32(a0 + 0x150);
    if p != 0 {
        let f = m.rf(a0 + 0x1a8);
        m.wf(p + 0x18, f);
    }
}

/// FUN_0019cce8(o): FUN_0019cd38(o, 2) unless +0x38 non-null and *(+0x38) in {0, 8..10}.
pub fn ee_sub_19cce8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let p = m.r32(a0 + 0x38);
    if p != 0 {
        let v = m.r32(p) as i32;
        if v == 0 {
            return;
        }
        if v >= 0 && v < 11 && v >= 8 {
            return;
        }
    }
    m.call(0x0019_cd38, &[a0, 2], &[]);
}

pub fn ee_sub_15dc80<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5270, &[a0], &[]);
    m.call(0x001a_6ca8, &[a0, 1], &[]);
    let v = m.r32(a0 + 0x40);
    if (v ^ 1) & 1 != 0 {
        let w = m.r32(v) | 0xa;
        m.w32(v, w);
    } else {
        m.w32(a0 + 0x40, v | 0xa);
    }
}

pub fn ee_sub_173158<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5270, &[a0], &[]);
    m.wf(a0 + 0x80, f32::from_bits(0x45ab_e000));
    m.wf(a0 + 0xa4, f32::from_bits(0x4248_0000));
    m.w32(a0 + 0x84, 1);
    m.wf(a0 + 0xa0, f32::from_bits(0x41f0_0000));
    m.w32(a0 + 0x88, 1);
    m.w32(a0 + 0x8c, 1);
}

pub fn ee_sub_183f50<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5c48, &[a0], &[]);
    if m.r32(a0 + 0xfc) == 0 {
        m.w32(a0 + 0xfc, 7);
    }
    if m.rf(a0 + 0x38) == f32::from_bits(0x5015_02f9) {
        let f = m.rf(0x002b_e900);
        m.wf(a0 + 0x38, f);
    }
}

pub fn ee_sub_1aea40<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001d_9910, &[a0], &[]);
    m.w32(a0 + 0x24, 0);
    m.call(0x0014_d1d0, &[0x002d_ded0, 0], &[f4(1.0)]);
    m.call(0x0020_f030, &[GS, 0], &[]);
    m.call(0x0016_19a0, &[0x002d_eb30, 0x2c, 0], &[]);
}

pub fn ee_sub_1c6588<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001c_5e48, &[a0], &[]);
    if m.r32(a0 + 0x150) != 0 {
        return;
    }
    let f = m.rf(a0 + 0x238) - m.rf(0x002d_ded4);
    m.wf(a0 + 0x238, f);
    if f <= 0.0 {
        vslot(m, a0, 75, &[5], &[]);
    }
}

/// FUN_0014c6d8(o, p, a2, a3, t0, t1): p ? FUN_00118668(p, 0x7fffffff) : (bit 49 of +0x10 ?
/// FUN_0014c748(t0, t1; *(*o + 0x70), a2, a3, o[2]) : 0).
pub fn ee_sub_14c6d8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, t0: u32, t1: u32, a2: V4, a3: V4) -> u32 {
    if a1 != 0 {
        return m.call(0x0011_8668, &[a1, 0x7fff_ffff], &[]).0;
    }
    if m.r64(a0 + 0x10) & (0x8000u64 << 34) == 0 {
        return 0;
    }
    let p = m.r32(a0);
    let c = m.rv(p + 0x70);
    let r = m.rf(a0 + 8);
    m.call(0x0014_c748, &[t0, t1], &[c, a2, a3, f4(r)]).0
}

pub fn ee_sub_1d6a90<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5270, &[a0], &[]);
    let f2 = m.rf(0x002a_7874);
    m.w32(a0 + 0x70, 0);
    m.w32(a0 + 0x78, 0);
    m.w32(a0 + 0x74, 0);
    m.w32(a0 + 0x80, 0);
    m.wf(a0 + 0x84, f2);
    m.w32(a0 + 0x90, 0);
    m.wf(a0 + 0x94, 1.0);
    m.w32(a0 + 0x98, 0);
    m.wf(a0 + 0xb0, 0.5);
    m.wf(a0 + 0xa4, f2);
    m.w32(a0 + 0xa0, 0);
}

pub fn ee_sub_1f6ac8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if (m.r32(a0 + 0xe0) as i32) <= 0 {
        return;
    }
    let mut i: i32 = 0;
    let mut off: u32 = 0;
    loop {
        let e = m.r32(a0 + 0xe4).wrapping_add(off);
        if m.r32(e + 0xc) == a1 {
            m.call(0x001f_2a60, &[a0, e], &[]);
        }
        i += 1;
        off = off.wrapping_add(0xb0);
        if i >= m.r32(a0 + 0xe0) as i32 {
            break;
        }
    }
}

pub fn ee_sub_1811b8<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    if m.r32(a0 + 0x1c0) == 4 {
        return 0;
    }
    if m.call(0x0017_f1d0, &[a0], &[]).0 == 0 {
        return 0;
    }
    let v = m.r32(a0 + 0x184) as i32;
    if v < 0 {
        return 1;
    }
    if v <= 0 {
        m.call(0x0016_19a0, &[0x002d_eb30, 0x18, a0], &[]);
    }
    m.call(0x0018_1268, &[a0, 4], &[]);
    1
}

pub fn ee_sub_1adee8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001d_a808, &[a0], &[]);
    let p = m.r32(a0 + 0x24);
    if p == 0 {
        return;
    }
    if m.r32(p + 0x58) != 2 || m.r32(p + 0x64) != 2 {
        return;
    }
    let f = m.rf(0x002d_df0c) - m.rf(p + 0x5c);
    if !(1.0 < f) {
        return;
    }
    if m.call(0x001a_b088, &[0x002f_633c], &[]).0 != 0 {
        return;
    }
    m.call(0x001a_de70, &[a0], &[]);
}

/// FUN_001944c8(o, out): selects a field by +8 / +0xc; *out = 1 only for the +8 == 8, +0xc in 9..10 case.
pub fn ee_sub_1944c8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let mut a2 = 0u32;
    let k = m.r32(a0 + 8) as i32;
    let v = if k == 1 {
        m.r32(a0 + 0x58)
    } else if k < 2 {
        if k == 0 {
            if m.r32(a0 + 0xc) != 5 {
                m.r32(a0 + 0x54)
            } else {
                m.r32(a0 + 0x60)
            }
        } else {
            0
        }
    } else if k == 8 {
        let c = m.r32(a0 + 0xc) as i32;
        if c < 11 && c >= 9 {
            a2 = 1;
            m.r32(a0 + 0x64)
        } else {
            0
        }
    } else {
        0
    };
    m.w32(a1, a2);
    v
}

pub fn ee_sub_22d6b0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) -> u32 {
    let k = a0 & 0x3000;
    if k == 0x1000 {
        let b = m.r32(a1 + 0x54);
        return b.wrapping_add((a0 & 0xffff_0fff) << 8);
    }
    if k == 0 {
        if a2 == 0 {
            return 0;
        }
        return m.r32(a2 + 0x54).wrapping_add(a0 << 8);
    }
    if k == 0x2000 {
        let i = m.r32(0x0044_37a0).wrapping_sub(1);
        return m.r32((i << 2).wrapping_add(0x0044_37a8));
    }
    0
}

pub fn ee_sub_13bf50<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001f_36e0, &[a0], &[]);
    let s = m.r32(a0 + 0x184);
    if s == 4 {
        let p = m.r32(a0 + 0x158);
        if m.r32(p + 0xc) != 0 {
            let t = m.rf(0x002d_ded0);
            if m.rf(0x003e_1bfc) <= t {
                let x = m.r32(a0 + 0x160);
                m.call(0x0014_3380, &[x], &[]);
            }
        }
    } else if s == 5 {
        let f = m.rf(0x002d_ded0) - m.rf(a0 + 0x188);
        if 5.0 <= f {
            m.call(0x0013_c4e0, &[a0, 0], &[]);
        }
    }
}

/// FUN_00143220(o, out, f12): d = slot32(o) with its component along the 0x2a2dc0 axis removed;
/// out = {0, 150, 75, ..; +0x10 = FUN_001a62e8(o) + d * f12 (+w 1), +0x20 = d}.
pub fn ee_sub_143220<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, f12: f32) {
    let v3 = vslot(m, a0, 32, &[], &[]).1;
    let ax = m.rv(0x002a_2dc0);
    let d = v3[0] * ax[0] + v3[1] * ax[1] + v3[2] * ax[2];
    let mut v3d = [0.0f32; 4];
    for i in 0..4 {
        v3d[i] = v3[i] - ax[i] * d;
    }
    let sp = stack_push(m, 0x20);
    m.wv(sp + 0x10, v3d);
    let b = m.call(0x001a_62e8, &[a0], &[]).1;
    let v3d = m.rv(sp + 0x10);
    let one = [0.0f32, 0.0, 0.0, 1.0];
    let mut r = [0.0f32; 4];
    for i in 0..4 {
        r[i] = b[i] + one[i] + v3d[i] * f12;
    }
    m.wv(a1 + 0x20, v3d);
    m.wf(a1 + 4, 150.0);
    m.wf(a1 + 8, 75.0);
    m.wv(a1 + 0x10, r);
    m.wv(sp, f4(f12));
    m.w32(a1, 0);
    stack_pop(m, 0x20);
}

pub fn ee_sub_1a5ee8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, f12: f32, f13: f32) {
    let sp = stack_push(m, 0xc0);
    let s1 = sp + 0x40;
    let s2 = sp + 0x80;
    let v = vslot(m, a0, 30, &[], &[]).0;
    m.call(0x0011_c318, &[s2, v], &[]);
    m.call(0x0011_c168, &[s1, a1, s2], &[]);
    for k in 0..4u32 {
        let q = m.rv(s1 + 0x10 * k);
        m.wv(sp + 0x10 * k, q);
    }
    vslot(m, a0, 19, &[sp, 0], &[f4(f12), f4(f13)]);
    stack_pop(m, 0xc0);
}

pub fn ee_sub_130eb0<M: QMem + ?Sized>(m: &mut M, a2: u32) {
    let p = m.r32(a2 + 0x14);
    let n = m.r32(p + 0xa0) as i32;
    if n == 0 {
        return;
    }
    if 0.0 <= m.rf(a2 + 0x40) {
        let mut i: i32 = 0;
        if n > 0 {
            let f1 = m.rf(a2 + 0x1c);
            let e = m.r32(p + 0xa4);
            if !(f1 <= m.rf(e)) {
                i = 1;
                while i < n {
                    let e = m.r32(p + 0xa4).wrapping_add((i as u32) << 4);
                    if f1 <= m.rf(e) {
                        break;
                    }
                    i += 1;
                }
            }
        }
        m.w32(a2 + 0x50, i as u32);
    } else {
        let mut i = n - 1;
        while i >= 0 {
            let q = m.r32(a2 + 0x14);
            let f1 = m.rf(a2 + 0x1c);
            let e = m.r32(q + 0xa4).wrapping_add((i as u32) << 4);
            if m.rf(e) <= f1 {
                break;
            }
            i -= 1;
        }
        m.w32(a2 + 0x50, (i + 1) as u32);
    }
}

/// FUN_0019c8d0(o, k): jump table 0x2a6600 over k - 2.
pub fn ee_sub_19c8d0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    fn tail<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
        let c = m.r32(a0 + 0x1c) as i32;
        if c == 6 {
            0
        } else if c < 6 {
            1
        } else if c < 0x17 && c >= 9 {
            0
        } else {
            1
        }
    }
    match a1.wrapping_sub(2) {
        0 | 3 | 25 => tail(m, a0),
        1 => {
            if m.r32(a0 + 0x1c) != 0x11 {
                return 0;
            }
            let p = m.r32(a0);
            if m.r32(p + 0x12f0) == 0 {
                return 0;
            }
            if m.r32(0x002d_fbc0) & 2 == 0 {
                return 0;
            }
            (m.call(0x0014_fd50, &[0x002d_e2e0], &[]).0 == 0) as u32
        }
        10 => {
            if m.r32(a0 + 0x2c) != 0 {
                tail(m, a0)
            } else {
                0
            }
        }
        20 => {
            let v = m.r32(a0 + 4) as i32;
            if v >= 0x11 || v < 0xf {
                tail(m, a0)
            } else {
                0
            }
        }
        _ => 0,
    }
}

pub fn ee_sub_1526d8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5270, &[a0], &[]);
    let f4v = f32::from_bits(0xbfc0_0000);
    m.w32(0x002d_e570, a0);
    m.wf(a0 + 0x234, f4v);
    m.wf(a0 + 0x238, 1.0);
    let f2 = f32::from_bits(0x3ecc_cccd);
    m.wf(a0 + 0x5c, f2);
    m.w32(a0 + 0x280, 8);
    m.wf(a0 + 0x58, f2);
    let f0 = (1.0f32 - f4v).abs() * f32::from_bits(0x3eaa_aaab);
    m.wf(a0 + 0x28c, f0);
    m.wf(a0 + 0x294, 5.0);
    m.w32(a0 + 0x290, 0);
    m.wf(a0 + 0x29c, f32::from_bits(0xbf4c_cccd));
    m.wf(a0 + 0x2a0, f32::from_bits(0xbe4c_cccd));
    m.wf(a0 + 0x2a4, f4v);
    m.wf(a0 + 0x298, f4v);
    m.call(0x001a_6ca8, &[a0, 1], &[]);
    let v = m.r32(a0 + 0x40);
    if (v ^ 1) & 1 != 0 {
        let w = m.r32(v) | 0xa;
        m.w32(v, w);
    } else {
        m.w32(a0 + 0x40, v | 0xa);
    }
}

/// FUN_00203d00(o, x, go): +0x25c = x; if go: M = FUN_0011c168(inv(+0x258 xf), +0x25c xf); transform the
/// +0x25c point list into the +0x258 one; FUN_002333b4(...); FUN_001f4bf8(o).
pub fn ee_sub_203d00<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    m.w32(a0 + 0x25c, a1);
    if a2 == 0 {
        return;
    }
    let sp = stack_push(m, 0xc0);
    let s0 = sp + 0x80;
    let s1 = sp + 0x40;
    let p = m.r32(a0 + 0x258);
    let p = m.r32(p);
    m.call(0x0011_c318, &[s0, p.wrapping_add(0x40)], &[]);
    let p = m.r32(a0 + 0x25c);
    let p = m.r32(p);
    m.call(0x0011_c168, &[s1, s0, p.wrapping_add(0x40)], &[]);
    for k in 0..4u32 {
        let q = m.rv(s1 + 0x10 * k);
        m.wv(sp + 0x10 * k, q);
    }
    let v1 = m.r32(a0 + 0x25c);
    let v0 = m.r32(a0 + 0x258);
    let mut n = (m.r32(v1 + 0x78) as i32).wrapping_sub(1);
    let mut dst = m.r32(v0 + 0x7c);
    let mut src = m.r32(v1 + 0x7c);
    while n >= 0 {
        let v = m.rv(src);
        n -= 1;
        src = src.wrapping_add(0x10);
        let r0 = m.rv(sp);
        let r1 = m.rv(sp + 0x10);
        let r2 = m.rv(sp + 0x20);
        let r3 = m.rv(sp + 0x30);
        let mut o = r0;
        for i in 0..3 {
            o[i] = r0[i] * v[0] + r1[i] * v[1] + r2[i] * v[2] + r3[i];
        }
        m.wv(dst, o);
        dst = dst.wrapping_add(0x10);
    }
    let p = m.r32(a0 + 0x258);
    let q = m.r32(p + 0x10);
    let b = m.r32(q);
    let sz = m.r32(p + 0x78) << 4;
    m.call(0x0023_33b4, &[b, 0, sz], &[]);
    m.call(0x001f_4bf8, &[a0], &[]);
    stack_pop(m, 0xc0);
}

pub fn ee_sub_190e18<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let sp = stack_push(m, 0xf0);
    let x = m.r32(a0 + 0x18c4);
    let v = m.call(0x001a_62e8, &[x], &[]).1;
    let q = m.r32(a0 + 0x19cc);
    let b = m.rv(q + 0x70);
    let d = [v[0] - b[0], v[1] - b[1], v[2] - b[2], v[3] - b[3]];
    let c = m.r32(a0 + 0x19c0);
    let k = m.r32(c + 0xa0);
    let r = m.call(0x0022_fcc8, &[0, k], &[d]).1;
    let ax = m.rv(0x002a_2da0);
    m.call(0x0011_ba08, &[sp + 0x60], &[ax, r]);
    let ax = m.rv(0x002a_2da0);
    let f = m.rf(0x002b_f594);
    m.call(0x0011_b500, &[sp + 0x90], &[ax, f4(f)]);
    let a = [m.rv(sp + 0x60), m.rv(sp + 0x70), m.rv(sp + 0x80)];
    let bm = [m.rv(sp + 0x90), m.rv(sp + 0xa0), m.rv(sp + 0xb0)];
    let mut out = [bm[0], bm[1], a[0]];
    for j in 0..3 {
        for i in 0..3 {
            out[j][i] = a[0][i] * bm[j][0] + a[1][i] * bm[j][1] + a[2][i] * bm[j][2];
        }
    }
    m.wv(sp, out[0]);
    m.wv(sp + 0x10, out[1]);
    m.wv(sp + 0x20, out[2]);
    m.wv(sp + 0xc0, out[0]);
    m.wv(sp + 0x30, out[0]);
    m.wv(sp + 0xd0, out[1]);
    m.wv(sp + 0x40, out[1]);
    m.wv(sp + 0xe0, out[2]);
    m.wv(sp + 0x50, out[2]);
    let q = m.call(0x0011_bff0, &[sp], &[]).1;
    let o = m.r32(a0 + 0x1a00);
    m.call(0x0012_d108, &[o], &[q]);
    let cur = m.rv(a0 + 0x1960);
    let tgt = m.rv(0x002a_2dd0);
    let dt = m.rf(0x002d_ded4);
    let r = m.call(0x0011_f088, &[0x002b_f598, 0], &[cur, tgt, f4(dt)]).1;
    m.wv(a0 + 0x1960, r);
    stack_pop(m, 0xf0);
}

/// FUN_00118220(rd, base, out): out[0..3] = 3 reads (FUN_00117c80); out[3] = alloc(out[0] * 12, 0x10);
/// per entry: +8 = base[3] + 32 * FUN_00117c20 (0x2bbaf0 for 0xffff), +0 / +4 = links (-1 = none).
pub fn ee_sub_118220<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    let sp = stack_push(m, 0x10);
    let v = m.call(0x0011_7c80, &[a0], &[]).0;
    m.w32(a2, v);
    let v = m.call(0x0011_7c80, &[a0], &[]).0;
    m.w32(a2 + 4, v);
    let v = m.call(0x0011_7c80, &[a0], &[]).0;
    let n = m.r32(a2);
    m.w32(a2 + 8, v);
    m.w32(sp + 4, 0x10);
    m.w32(sp, n.wrapping_mul(0xc));
    let v = m.call(0x001a_e2a0, &[sp], &[]).0;
    m.w32(a2 + 0xc, v);
    let mut i: i32 = 0;
    let mut off: u32 = 0;
    if (m.r32(a2) as i32) > 0 {
        loop {
            let s1 = m.r32(a2 + 0xc).wrapping_add(off);
            let k = m.call(0x0011_7c20, &[a0], &[]).0;
            if k == 0xffff {
                m.w32(s1 + 8, 0x002b_baf0);
            } else {
                let b = m.r32(a1 + 0xc);
                m.w32(s1 + 8, b.wrapping_add(k << 5));
                let v = m.call(0x0011_7c80, &[a0], &[]).0;
                if v != 0xffff_ffff {
                    let b = m.r32(a2 + 0xc);
                    m.w32(s1, b.wrapping_add(v.wrapping_mul(0xc)));
                }
                let v = m.call(0x0011_7c80, &[a0], &[]).0;
                if v != 0xffff_ffff {
                    let b = m.r32(a2 + 0xc);
                    m.w32(s1 + 4, b.wrapping_add(v.wrapping_mul(0xc)));
                }
            }
            i += 1;
            off = off.wrapping_add(0xc);
            if i >= m.r32(a2) as i32 {
                break;
            }
        }
    }
    stack_pop(m, 0x10);
}

pub fn ee_sub_1b4728<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0013_ff70, &[a0], &[]);
    let f20 = 50.0f32;
    m.wf(a0 + 0x220, f20);
    m.w32(a0 + 0x17f4, 0xffff_ffff);
    m.wf(a0 + 0x1410, f32::from_bits(0x44bb_8000));
    m.wf(a0 + 0x1414, f32::from_bits(0x455a_c000));
    m.wf(a0 + 0x103c, 60.0);
    m.wf(a0 + 0x1048, 125.0);
    for o in [0x13d8u32, 0x1810, 0x1840, 0x13d0, 0x144c, 0x1450, 0x1458] {
        m.w32(a0 + o, 0xffff_ffff);
    }
    m.call(0x0020_b840, &[a0 + 0x1320, a0, 0x15], &[]);
    m.call(0x0020_b840, &[a0 + 0x1460, a0, 0x15], &[]);
    m.call(0x0020_b840, &[a0 + 0x1630, a0, 0x15], &[]);
    m.call(0x0020_b840, &[a0 + 0x1540, a0, 0x15], &[]);
    m.call(0x0020_bd78, &[a0 + 0x1850], &[]);
    m.wf(a0 + 0x1434, -60.0);
    m.wf(a0 + 0x1098, -100.0);
    m.wf(a0 + 0x109c, 100.0);
    m.wf(a0 + 0x10a0, f20);
    m.wf(a0 + 0x10a4, -10.0);
    m.wf(a0 + 0x10a8, 250.0);
    m.wf(a0 + 0x1ac8, -60.0);
    m.wf(a0 + 0x10b4, f32::from_bits(0x458c_a000));
    m.wf(a0 + 0x10b0, 1.0);
    m.w32(a0 + 0x10ac, 0);
    m.call(0x0021_f2e0, &[a0 + 0x1910, a0], &[]);
    m.w32(a0 + 0x1000, 4);
    m.w32(0x002f_7900, a0);
}

pub fn ee_sub_259170<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let sp = stack_push(m, 0x30);
    let mut s2: u32 = 0xffff_ffff;
    m.call(0x001a_5c48, &[a0], &[]);
    let p = m.r32(a0 + 0xe4);
    let mut s1 = m.r32(p + 0x20);
    if m.r32(p + 0x10) == 2 {
        s2 = m.r32(p + 0x30);
    }
    m.call(0x0023_33b4, &[sp, 0, 0x14], &[]);
    let s3: u32 = 0xffff_ffff;
    if s1 != s3 {
        let v = m.call(0x0015_e938, &[GS, 4, s1, a0], &[]).0;
        m.w32(sp, v);
    }
    if s2 != s3 {
        let v = m.call(0x0015_e938, &[GS, 4, s2, a0], &[]).0;
        m.w32(sp, 0);
        m.w32(sp + 4, v);
    }
    let p = m.r32(a0 + 0xe4);
    let k = m.r32(p + 4);
    if k == 0 {
        let x = m.r32(p + 0xd4);
        if x != s3 {
            let v = m.call(0x0015_e938, &[GS, 4, x, a0], &[]).0;
            m.w32(sp + 8, v);
        }
    } else if k == 1 {
        let x = m.r32(p + 0xc8);
        s2 = m.r32(p + 0xd0);
        s1 = m.r32(p + 0xcc);
        if x != s3 {
            let v = m.call(0x0015_e938, &[GS, 4, x, a0], &[]).0;
            m.w32(sp + 8, v);
        }
        if s1 != s3 {
            let v = m.call(0x0015_e938, &[GS, 4, s1, a0], &[]).0;
            m.w32(sp + 0xc, v);
        }
        if s2 != s3 {
            let v = m.call(0x0015_e938, &[GS, 4, s2, a0], &[]).0;
            m.w32(sp + 0x10, v);
        }
    }
    let p = m.r32(a0 + 0xe4);
    if m.r32(p + 8) == 0 {
        m.w32(p + 8, 0x002c_e4b8);
    }
    let p = m.r32(a0 + 0xe4);
    let q = m.r32(p + 8);
    if m.call(0x0023_3178, &[q, sp, 0x14], &[]).0 != 0 {
        m.call(0x0025_90f0, &[a0, 1], &[]);
        m.w32(sp + 0x20, 0x14);
        m.w32(sp + 0x24, 0x10);
        let v = m.call(0x001a_e238, &[sp + 0x20, sp], &[]).0;
        let p = m.r32(a0 + 0xe4);
        m.w32(p + 8, v);
    }
    stack_pop(m, 0x30);
}

pub fn ee_sub_1dd920<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001d_a808, &[a0], &[]);
    if m.r32(a0 + 0x24) == 0 {
        return;
    }
    let x = m.r32(a0 + 0x10);
    if x == 0 {
        return;
    }
    if m.call(0x0018_7d80, &[x, 0x800], &[]).0 != 0 {
        let s1: u32 = 0x002e_65e0 + 0x8000;
        let v = m.r32(s1 + 0x7d54);
        m.w32(s1 + 0x7d50, v);
        if v != 0 {
            m.call(0x001a_cc38, &[0x002e_6550, 1], &[]);
            let v = m.r32(s1 + 0x7d50);
            m.call(0x0016_5e38, &[v], &[]);
        } else {
            m.call(0x001d_db08, &[a0], &[]);
        }
        return;
    }
    let x = m.r32(a0 + 0x10);
    if m.call(0x0018_7dd0, &[x, 0x100], &[]).0 != 0 {
        let x = m.r32(a0 + 0x10);
        if m.r32(x + 0x10b4) & 0x108 == 0x100 {
            m.call(0x0018_7de0, &[x, 0x100], &[]);
            m.call(0x001b_04e0, &[0], &[]);
            return;
        }
    }
    if (m.call(0x001a_2c98, &[], &[]).0 as i32) < 2 {
        return;
    }
    let x = m.r32(a0 + 0x10);
    if m.call(0x0018_7d80, &[x, 0x10], &[]).0 == 0 {
        return;
    }
    let s3 = m.call(0x0015_e938, &[GS, 5, 0x5d6, 0], &[]).0;
    if s3 == 0 {
        return;
    }
    let sp = stack_push(m, 0x10);
    m.call(0x001a_2c48, &[sp, sp | 4], &[]);
    let mut i: i32 = 0;
    if (m.r32(sp) as i32) > 0 {
        let mut s1: u32 = 0x003d_a578;
        loop {
            let t = m.r32(sp + 4).wrapping_add((i as u32) << 2);
            let s0 = m.r32(t);
            m.call(0x0023_33b4, &[s1, 0, 0x20], &[]);
            i += 1;
            m.w32(s1 + 8, s0);
            let v = m.r32((s0 << 2).wrapping_add(0x002c_35b8));
            m.w32(s1 + 4, 0x001d_d8e0);
            m.w32(s1, v);
            s1 = s1.wrapping_add(0x20);
            if i >= m.r32(sp) as i32 {
                break;
            }
        }
    }
    let n = m.r32(sp);
    m.call(0x001a_f4d8, &[0x003d_9ed0, n, 0x003d_a578, s3, 0x002c_35f0], &[]);
    stack_pop(m, 0x10);
}

pub fn ee_sub_19f590<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let s3 = a0;
    let sp = stack_push(m, 0x30);
    let src = 'main: {
        if a1 != 0 {
            break 'main a1;
        }
        m.call(0x0019_faf8, &[s3 + 0x9e0, sp], &[]);
        if m.call(0x001c_4840, &[GS], &[]).0 != s3 {
            break 'main sp;
        }
        let r = m.call(0x0019_f3f8, &[s3, sp], &[f4(0.0)]).1;
        m.wv(sp + 0x20, r);
        let q = m.r32(s3 + 0x58);
        let b = m.rv(q + 0x70);
        let r = m.rv(sp + 0x20);
        let d = [r[0] - b[0], r[1] - b[1], r[2] - b[2], r[3] - b[3]];
        let q = m.r32(s3 + 0xe8);
        let a1q = m.rv(q + 0x80);
        let f = m.call(0x0019_6928, &[], &[d, a1q, f4(1000.0), f4(-1000.0), f4(1.0), f4(-1.0)]).1[0];
        if !(m.rf(0x002b_f940) < f) {
            break 'main sp;
        }
        let s2 = s3 + 0x2c0;
        let b = m.rv(s3 + 0x240);
        let r = m.rv(sp + 0x20);
        let d2 = [r[0] - b[0], r[1] - b[1], r[2] - b[2], r[3] - b[3]];
        let s0 = m.r32(s3 + 0x9e4);
        vslot(m, s3, 28, &[], &[d2]);
        m.w32(s3 + 0x9e4, s0);
        let s1: u32 = 0x002d_df40;
        let q = m.rv(s1 + 0x60);
        m.call(0x0020_46b0, &[s3], &[q]);
        vslot(m, s3, 29, &[s3 + 0x1b0], &[]);
        m.call(0x0014_d8c0, &[s1], &[]);
        m.call(0x0019_5a00, &[s2, 0xb, 0, 0], &[]);
        if m.r32(sp + 4) != 0 {
            let mut hit = false;
            for k in [0x13u32, 0xb, 0xd, 0x27, 0xc, 0x22, 0x26] {
                let x = m.r32(sp);
                if m.call(0x0013_2b10, &[x, k], &[]).0 != 0 {
                    hit = true;
                    break;
                }
            }
            if hit {
                let x = m.r32(sp);
                m.call(0x0014_8300, &[GS, x], &[]);
                let x = m.r32(sp);
                m.call(0x0019_b110, &[s2, x], &[]);
            }
        }
        let x = m.r32(GS + 0x3e60);
        m.call(0x0015_e1d8, &[x], &[]);
        stack_pop(m, 0x30);
        return;
    };
    let q = m.rv(src);
    m.wv(s3 + 0x490, q);
    let q = m.rv(src + 0x10);
    m.wv(s3 + 0x4a0, q);
    m.call(0x0019_5a00, &[s3 + 0x2c0, 0xa, 0, 0], &[]);
    stack_pop(m, 0x30);
}

pub fn ee_sub_173a68<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    const T: u32 = 0x002d_ded0;
    let sp = stack_push(m, 0x60);
    'b: {
        m.call(0x001a_5e28, &[a0], &[]);
        let pos = m.call(0x0017_3a18, &[a0], &[]).1;
        m.wv(sp + 0x50, pos);
        m.call(0x0017_3d18, &[a0], &[pos]);
        let t = m.rf(T);
        if t < m.rf(a0 + 0xa8) {
            break 'b;
        }
        let f = t + m.rf(0x002b_e5d0);
        m.wf(a0 + 0xa8, f);
        if m.r32(a0 + 0x58) == m.r32(a0 + 0x5c) {
            let b = m.rv(a0 + 0x70);
            let p = m.rv(sp + 0x50);
            let dx = p[0] - b[0];
            let dy = p[1] - b[1];
            let len = (dx * dx + dy * dy).sqrt();
            if len < m.rf(0x002b_e5d4) {
                break 'b;
            }
        }
        let mut s2 = 0u32;
        let v = m.call(0x001c_4840, &[GS], &[]).0;
        if v != 0 {
            s2 = m.call(0x001f_8240, &[v, 0], &[]).0;
            if s2 != 0 {
                if m.r32(s2 + 0x28) == 0 {
                    s2 = 0;
                } else {
                    let k = m.r32(a0 + 0xac) as i32;
                    if k == 1 {
                        let d = m.rf(T) - m.rf(s2);
                        if m.rf(0x002b_e5cc) < d {
                            break 'b;
                        }
                    } else if k == 0 {
                        break 'b;
                    }
                }
            }
        }
        let mut a1 = 0u32;
        if s2 != 0 && m.r32(s2 + 0x28) == 1 {
            a1 = m.r32(s2 + 0x2c);
        }
        let s1 = m.call(0x0017_3e78, &[a0, a1], &[]).0;
        if s1 == 0 {
            break 'b;
        }
        let pos = m.rv(sp + 0x50);
        let s3 = m.call(0x0017_3f18, &[a0, sp, sp + 0x10, sp + 0x40], &[pos]).0;
        if s3 == 0 {
            break 'b;
        }
        let pos = m.rv(sp + 0x50);
        m.wv(a0 + 0x70, pos);
        if m.r32(s1) != 0 {
            m.call(0x0017_3928, &[a0, s1, 0, 1], &[]);
        }
        let t0 = m.r32(sp + 0x40);
        let v = m.call(0x0017_4490, &[a0, s3, sp, sp + 0x10, t0], &[]).0;
        m.w32(sp + 0x44, v);
        if v == 0 {
            break 'b;
        }
        m.call(0x001f_7d10, &[v], &[f4(f32::from_bits(0x3c23_d70a))]);
        let v = m.r32(sp + 0x44);
        m.call(0x001f_7c68, &[v], &[f4(0.5)]);
        if s2 != 0 {
            let v = m.r32(sp + 0x44);
            let x = m.r32(v + 0xf64);
            let y = m.r32(s2 + 0xc);
            m.call(0x0015_4038, &[x, y, 2], &[]);
        }
        let v = m.r32(sp + 0x44);
        let f1 = m.rf(0x002b_e5d8);
        m.w32(s1, v);
        let t = m.rf(T);
        m.w32(s1 + 0xc, s3);
        m.wf(s1 + 4, t);
        let n = m.r32(a0 + 0x58).wrapping_add(1);
        m.w32(a0 + 0x58, n);
        m.w32(s3 + 0x14, 1);
        m.w32(sp + 0x48, sp + 0x44);
        let t = m.rf(T);
        m.wf(a0 + 0xa8, t + f1);
        vslot(m, a0, 2, &[0x19, 0, 1, sp + 0x48, 0, 1], &[]);
    }
    stack_pop(m, 0x60);
}

/// FUN_0012f590(_, rd, timed, out_n, out_p): read a count + array of 16-byte keyed records (jump table 0x2a4660).
pub fn ee_sub_12f590<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, t0: u32) {
    let rd = a1;
    let k = f32::from_bits(0x3c88_8889);
    let sp = stack_push(m, 0x30);
    let n = m.call(0x0011_7bd0, &[rd], &[]).0;
    m.w32(a3, n);
    m.w32(sp, n << 4);
    m.w32(sp + 4, 0x10);
    let s5 = m.call(0x001a_e2a0, &[sp], &[]).0;
    m.w32(t0, s5);
    let n = n as i32;
    let mut i: i32 = 0;
    while i < n {
        let e = s5.wrapping_add((i as u32) << 4);
        let v = m.call(0x0011_7d00, &[rd], &[]).0;
        m.w32(e + 4, v);
        if a2 != 0 {
            let v = m.call(0x0011_7d50, &[rd], &[]).0 as i32;
            m.wf(e, v as f32 * k);
        }
        match m.r32(e + 4) {
            0 => {
                let v = m.call(0x0011_7d50, &[rd], &[]).0;
                m.w32(sp + 0x10, v);
                let v = m.call(0x0011_7db8, &[rd], &[]).0;
                m.w32(sp + 0x14, v);
                let v = m.call(0x0011_7db8, &[rd], &[]).0;
                m.w32(sp + 0x18, v);
                let v = m.call(0x0011_7db8, &[rd], &[]).0;
                m.w32(sp + 0x1c, v);
                let f = m.call(0x0011_7e38, &[rd], &[]).1[0];
                m.wf(sp + 0x28, f);
                let f = m.call(0x0011_7e38, &[rd], &[]).1[0];
                m.wf(sp + 0x20, f);
                let f = m.call(0x0011_7e38, &[rd], &[]).1[0];
                m.wf(sp + 0x24, f);
                m.w32(sp, 0x1c);
                m.w32(sp + 4, 0x10);
                let v = m.call(0x001a_e238, &[sp, sp + 0x10], &[]).0;
                m.w32(e + 8, v);
            }
            1 | 2 | 21 => {
                let v = m.call(0x0011_7d50, &[rd], &[]).0;
                m.w32(e + 8, v);
            }
            3 => {
                let v = m.call(0x0011_7d50, &[rd], &[]).0 as i32;
                m.wf(e + 0xc, v as f32 * k);
                let v = m.call(0x0011_7d50, &[rd], &[]).0;
                m.w32(e + 8, v);
            }
            4 | 11 | 12 | 22 => {
                let v = m.call(0x0011_7db8, &[rd], &[]).0;
                m.w32(e + 8, v);
            }
            5 => {
                let v = m.call(0x0011_7d50, &[rd], &[]).0 as i32;
                m.wf(e + 8, v as f32 * k);
                let f = m.call(0x0011_7e38, &[rd], &[]).1[0];
                m.wf(e + 0xc, f);
            }
            7 | 8 => {
                let v = m.call(0x0011_7db8, &[rd], &[]).0;
                m.w32(e + 8, v);
                let f = m.call(0x0011_7e38, &[rd], &[]).1[0];
                m.wf(e + 0xc, f);
            }
            13..=16 => {
                let v = m.call(0x0011_7db8, &[rd], &[]).0;
                m.w32(e + 8, v);
                let v = m.call(0x0011_7db8, &[rd], &[]).0;
                m.w32(e + 0xc, v);
            }
            17 => {
                m.w32(sp + 4, 0x10);
                m.w32(sp, 0x10);
                let v = m.call(0x001a_e2a0, &[sp], &[]).0;
                m.w32(e + 8, v);
                let v = m.call(0x0011_7db8, &[rd], &[]).0;
                let p = m.r32(e + 8);
                m.w32(p, v);
                let v = m.call(0x0011_7db8, &[rd], &[]).0;
                let p = m.r32(e + 8);
                m.w32(p + 4, v);
                let v = m.call(0x0011_7db8, &[rd], &[]).0;
                let p = m.r32(e + 8);
                m.w32(p + 8, v);
                let f = m.call(0x0011_7e38, &[rd], &[]).1[0];
                let p = m.r32(e + 8);
                m.wf(p + 0xc, f);
            }
            20 => {
                let v = m.call(0x0011_7db8, &[rd], &[]).0;
                m.w32(sp, v);
                let v = m.call(0x0011_7db8, &[rd], &[]).0;
                m.w32(sp + 4, v);
                let f = m.call(0x0011_7e38, &[rd], &[]).1[0];
                m.wf(sp + 8, f);
                m.w32(sp + 0x10, 0xc);
                m.w32(sp + 0x14, 0x10);
                let v = m.call(0x001a_e238, &[sp + 0x10, sp], &[]).0;
                m.w32(e + 8, v);
            }
            24 => {
                m.w32(sp, 0x18);
                m.w32(sp + 4, 0x10);
                let s0 = m.call(0x001a_e1d0, &[sp], &[]).0;
                m.call(0x0011_7fb0, &[rd, s0], &[]);
                m.call(0x0011_7fb0, &[rd, s0 + 8], &[]);
                m.call(0x0011_7fb0, &[rd, s0 + 0x10], &[]);
                m.w32(e + 8, s0);
            }
            _ => {}
        }
        i += 1;
    }
    stack_pop(m, 0x30);
}

pub fn ee_sub_1394c8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let (s2, s3) = (a0, a1);
    let sp = stack_push(m, 0x10);
    let mut f0 = 1.0f32;
    let v = m.r32(s3 + 0xc0);
    m.w32(s2 + 0x43c, v);
    if v == 1 {
        f0 = m.rf(s3 + 0xf0);
    }
    m.wf(s2 + 0x440, f0);
    let p = m.r32(s3 + 0xdc);
    if p != 0 {
        let n = m.r32(p);
        m.w32(s2 + 0x3b0, n);
        if (n as i32) > 0 {
            let mut i: i32 = 0;
            loop {
                let n = m.r32(s2 + 0x3b0);
                let mut src = ((i as u32) << 2).wrapping_add(p + 4);
                let mut dst = (n.wrapping_sub(i as u32).wrapping_sub(1) << 2).wrapping_add(s2 + 0x3b4);
                for _ in 0..4 {
                    let b = m.r8(src) as u32;
                    src = src.wrapping_add(1);
                    m.w8(dst, ((b + 1) >> 1) as u8);
                    dst = dst.wrapping_add(1);
                }
                i += 1;
                if i >= m.r32(s2 + 0x3b0) as i32 {
                    break;
                }
            }
        }
        let v = m.r32(s3 + 0xcc);
        m.w32(s2 + 0x434, v);
    }
    let v = m.r32(s3 + 0xd0);
    m.call(0x0013_9370, &[s2, v], &[]);
    m.call(0x0023_33b4, &[s2 + 0x140, 0, 0x10], &[]);
    let k = m.r32(s2 + 0x1a0) as i32;
    if k >= 2 {
        let mut d = m.rf(s3 + 0xd8);
        if d == 0.0 {
            d = m.rf(s3 + 0x8c);
        }
        m.wf(s2 + 0x144, k as f32 / d);
    }
    let f = 1.0f32 / m.rf(s3 + 0x8c);
    m.call(0x0011_d5b0, &[s3 + 0xb0, s2 + 0x130], &[f4(f), f4(0.0)]);
    let f = 1.0f32 / m.rf(s3 + 0x8c);
    m.call(0x0011_d5b0, &[s3 + 0xe0, s2 + 0x110], &[f4(f), f4(0.0)]);
    let r;
    if m.rf(s2 + 0x114) == 0.0 && m.rf(s2 + 0x118) == 0.0 {
        r = m.rf(s2 + 0x110).abs();
    } else {
        let f3 = m.rf(s3 + 0x8c);
        let c2 = m.rf(s2 + 0x118);
        let c1 = m.rf(s2 + 0x114);
        m.w32(sp + 4, 0);
        let c0 = m.rf(s2 + 0x110);
        m.wf(sp, c0);
        m.wf(sp + 8, c0 + f3 * (c1 + f3 * c2));
        if c2 != 0.0 {
            let c1 = m.rf(s2 + 0x114);
            let f2 = -c1 / (c2 + c2);
            if 0.0 < f2 && f2 < f3 {
                let c2 = m.rf(s2 + 0x118);
                let c1 = m.rf(s2 + 0x114);
                m.wf(sp + 4, c0 + f2 * (c1 + f2 * c2));
            }
        }
        let a = m.rf(sp).abs();
        let b = m.rf(sp + 4).abs();
        let c = m.rf(sp + 8).abs();
        r = if b < a && c < a {
            a
        } else if c < b {
            b
        } else {
            c
        };
    }
    m.wf(s2 + 0x120, r);
    let f0 = m.rf(s3 + 0x8c);
    let mut s5 = false;
    m.w32(s2 + 0x150, 0);
    m.w32(s2 + 0x158, 0);
    m.wf(s2 + 0x154, 1.0f32 / f0);
    let v = m.r32(s3 + 0xc4);
    m.w32(s2 + 0x160, v);
    if v == 0 {
        let q1 = m.rv(s3 + 0x60);
        let q0 = m.rv(0x002a_2dd0);
        let eps = f4(f32::from_bits(0x38d1_b717));
        let mut s4 = false;
        if m.call(0x0011_d4d0, &[], &[f4(q0[0]), f4(q1[0]), eps]).0 != 0
            && m.call(0x0011_d4d0, &[], &[f4(q0[1]), f4(q1[1]), eps]).0 != 0
        {
            s4 = m.call(0x0011_d4d0, &[], &[f4(q0[2]), f4(q1[2]), eps]).0 != 0;
        }
        if s4 && m.rf(s3 + 0x58) == 0.0 && m.rf(s3 + 0x80) == 0.0 {
            s5 = true;
        }
    }
    if s5 {
        m.w32(s2 + 0x160, 1);
    }
    m.call(0x0023_33b4, &[s2 + 0x170, 0, 0x30], &[]);
    if m.r32(s2 + 0x160) == 0 {
        m.call(0x0015_bf40, &[s2 + 0x170, s3], &[]);
    }
    stack_pop(m, 0x10);
}

/// FUN_00176750(o, msg): unless FUN_00177dc0(msg, o + 0x30) — store msg (0x70 bytes) and post the matching
/// event object (FUN_00205438(o[0x10], code), jump table 0x2a5430 over *msg - 1) through its slot 7.
pub fn ee_sub_176750<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let (s2, s1) = (a0, a1);
    if m.call(0x0017_7dc0, &[s1, s2 + 0x30], &[]).0 != 0 {
        return;
    }
    m.call(0x001e_f0d8, &[s2 + 0x18], &[]);
    for k in 0..7u32 {
        let q = m.rv(s1 + 0x10 * k);
        m.wv(s2 + 0x30 + 0x10 * k, q);
    }
    let t = m.rf(0x002d_ded0);
    m.wf(s2 + 0xa0, t);
    let mut s4 = false;
    let code = m.r32(s1).wrapping_sub(1);
    let ev = |m: &mut M, c: u32| -> u32 {
        let x = m.r32(s2 + 0x10);
        m.call(0x0020_5438, &[x, c], &[]).0
    };
    match code {
        0 => {
            s4 = true;
            let e = ev(m, 0xaa);
            vslot(m, e, 7, &[], &[]);
        }
        1 => {
            s4 = true;
            let e = ev(m, 0xab);
            let x = m.r32(s1 + 0x10);
            m.call(0x0017_25e0, &[e, x], &[]);
            vslot(m, e, 7, &[], &[]);
        }
        2 => {
            let e = ev(m, 0x9d);
            m.w32(e + 0x3c, 0xb);
            let x = m.r32(s1 + 0x10);
            m.w32(e + 0x38, x);
            vslot(m, e, 7, &[], &[]);
        }
        3 => {
            let e = ev(m, 0xa5);
            let x = m.r32(s1 + 0x10);
            m.w32(e + 0x30, x);
            vslot(m, e, 7, &[], &[]);
        }
        4 => {
            s4 = true;
            let e = ev(m, 0xa9);
            let x = m.r32(s1 + 0x10);
            m.call(0x0017_1258, &[e, x, 1, 0], &[]);
            vslot(m, e, 7, &[], &[]);
        }
        5 => {
            let e = ev(m, 0xb0);
            let x = m.r32(s2 + 0x40);
            m.w32(e + 0x3c, x);
            vslot(m, e, 7, &[], &[]);
        }
        6 => {
            let e = ev(m, 0xb1);
            let x = m.r32(s1 + 0x10);
            m.w32(e + 0x5c, x);
            vslot(m, e, 7, &[], &[]);
        }
        7 => {
            let e = ev(m, 0xaf);
            vslot(m, e, 7, &[], &[]);
        }
        8 | 9 | 10 => {
            let e = ev(m, [0xa1, 0xa2, 0xa3][(code - 8) as usize]);
            for k in 0..6u32 {
                let q = m.rv(s1 + 0x10 + 0x10 * k);
                m.wv(e + 0x50 + 0x10 * k, q);
            }
            if code == 10 {
                s4 = true;
            }
            vslot(m, e, 7, &[], &[]);
        }
        11 => {
            let e = ev(m, 0xb2);
            vslot(m, e, 7, &[], &[]);
        }
        12 | 13 => {
            let e = ev(m, if code == 12 { 0xb4 } else { 0xb6 });
            let x = m.r32(s1 + 0x10);
            m.w32(e + 0x38, x);
            vslot(m, e, 7, &[], &[]);
        }
        14 => {
            let e = ev(m, 0xae);
            let p = m.r32(s2 + 0x10);
            let x = m.r32(p + 0x108c);
            m.w32(e + 0x38, x);
            vslot(m, e, 7, &[], &[]);
        }
        15 => {
            s4 = true;
            let e = ev(m, 0xb7);
            let p = m.r32(s2 + 0x10);
            let x = m.r32(p + 0xf68);
            m.w32(e + 0x30, x);
            let p = m.r32(s2 + 0x10);
            let q = m.rv(p + 0xf70);
            m.wv(e + 0x40, q);
            vslot(m, e, 7, &[], &[]);
        }
        16 => {
            let e = ev(m, 0x9e);
            m.w32(e + 0x3c, 0x1a);
            let x = m.r32(s1 + 0x10);
            m.w32(e + 0x38, x);
            vslot(m, e, 7, &[], &[]);
            let t = m.rf(0x002d_ded0);
            let (lo, hi) = (m.rf(0x002b_e618), m.rf(0x002b_e61c));
            let f = m.call(0x0011_d448, &[], &[f4(lo), f4(hi)]).1[0];
            m.wf(s2 + 0xa4, t + f);
        }
        17 => {
            let e = ev(m, 0xbd);
            let x = m.r32(s1 + 0x10);
            m.w32(e + 0x38, x);
            let f = m.rf(s1 + 0x14);
            m.wf(e + 0x40, f);
            vslot(m, e, 7, &[], &[]);
        }
        18 => {
            let e = ev(m, 0xbf);
            let p = m.r32(s2 + 0x10);
            let q = m.rv(p + 0x4b0);
            m.wv(e + 0x40, q);
            vslot(m, e, 7, &[], &[]);
        }
        _ => {}
    }
    if s4 {
        let x = m.r32(s2 + 0x10);
        m.call(0x0020_8540, &[x, 0], &[]);
    }
}

fn h8_chain<M: QMem + ?Sized>(m: &mut M, a0: u32, off: u32) -> u32 {
    let p = m.r32(a0);
    let q = m.r32(p + off);
    let r = m.r32(q + 0xa8);
    m.r32(r + 0x60)
}

fn h8_clr3<M: QMem + ?Sized>(m: &mut M, p: u32) {
    m.w32(p + 0x24, 0);
    m.w32(p + 0x28, 0);
    m.w32(p + 0x20, 0);
}

/// FUN_001a0b40(st, s): state machine transition (jump table 0x2a6730 over s + 1).
pub fn ee_sub_1a0b40<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let s0 = a0;
    let s2 = a1;
    let cur = m.r32(s0 + 4);
    if s2 == cur {
        return;
    }
    let sp = stack_push(m, 0x30);
    let o = |m: &mut M| m.r32(s0);
    let cur = cur as i32;
    if cur == 3 || cur == 10 {
        let x = o(m);
        if m.call(0x0013_2b10, &[x, 0x35], &[]).0 != 0 {
            let p = h8_chain(m, s0, 0x19b8);
            h8_clr3(m, p);
        }
    } else if cur == -1 {
        let x = o(m);
        if m.call(0x0013_2b10, &[x, 0x35], &[]).0 != 0 {
            let q = m.rv(0x002b_fa00);
            let p = h8_chain(m, s0, 0x19b8);
            m.wv(p + 0x20, q);
            let q = m.rv(0x002b_fa00);
            let p = h8_chain(m, s0, 0x19bc);
            m.wv(p + 0x20, q);
        }
        let x = o(m);
        vslot(m, x, 90, &[], &[]);
        let p = o(m);
        m.w32(p + 0x194, 0);
        m.w32(p + 0x198, 0);
        m.w32(p + 0x190, 0);
        let p = o(m);
        m.w32(p + 0x15c, 0);
        let f = m.rf(0x002a_6720);
        m.wf(s0 + 0x98, f);
    }
    let mut s1 = 0u32;
    let mut s3 = 1u32;
    let slot_of = |m: &mut M, s2: u32| m.r32(s0.wrapping_add(s2 << 2) + 0xa0);
    let mut st = s2;
    match st.wrapping_add(1) {
        0 => {
            let x = o(m);
            if m.call(0x0013_2b10, &[x, 0x35], &[]).0 != 0 {
                let p = h8_chain(m, s0, 0x19b8);
                h8_clr3(m, p);
                let p = h8_chain(m, s0, 0x19bc);
                h8_clr3(m, p);
            }
            let p = o(m);
            m.w32(p + 0x15c, 1);
            let f = m.rf(0x002d_ded0);
            m.wf(s0 + 0x98, f);
        }
        1 | 7 | 8 => {
            s1 = slot_of(m, st);
        }
        2 | 3 => {
            s3 = 0;
            let x = o(m);
            let f = m.call(0x0020_4740, &[x], &[]).1[0];
            let x = o(m);
            m.call(0x0020_4660, &[x], &[f4(f)]);
            let p = o(m);
            let f = m.rf(p + 0x1a8);
            let q = m.r32(p + 0x150);
            m.wf(q + 0x18, f);
            s1 = slot_of(m, st);
        }
        5 | 13 => {
            let x = o(m);
            let z = m.rv(0x002a_2dd0);
            vslot(m, x, 35, &[], &[z]);
            let x = o(m);
            let f = m.call(0x0020_4740, &[x], &[]).1[0];
            let x = o(m);
            m.call(0x0020_4660, &[x], &[f4(f)]);
            let p = o(m);
            let f = m.rf(p + 0x1a8);
            if st == 0xc {
                s3 = 0x21;
            }
            let q = m.r32(p + 0x150);
            m.wf(q + 0x18, f);
            s1 = slot_of(m, st);
        }
        6 | 12 => {
            let q = m.rv(0x002d_dfd0);
            m.wv(sp + 0x20, q);
            let x = o(m);
            let v = m.call(0x001a_62e8, &[x], &[]).1;
            let q = m.rv(sp + 0x20);
            let d = [q[0] - v[0], q[1] - v[1], q[2] - v[2], q[3] - v[3]];
            let x = o(m);
            m.call(0x0020_46b0, &[x], &[d]);
            if st == 0xb {
                s3 = 0x21;
            }
            s1 = slot_of(m, st);
        }
        4 | 11 => {
            if st == 10 {
                s3 = 0x21;
                let q = m.rv(0x002d_dfd0);
                m.wv(sp + 0x10, q);
                let x = o(m);
                let v = m.call(0x001a_62e8, &[x], &[]).1;
                let q = m.rv(sp + 0x10);
                let d = [q[0] - v[0], q[1] - v[1], q[2] - v[2], q[3] - v[3]];
                let x = o(m);
                m.call(0x0020_46b0, &[x], &[d]);
            }
            s1 = slot_of(m, st);
            let x = o(m);
            if m.call(0x0013_2b10, &[x, 0x35], &[]).0 != 0 {
                let q = m.rv(0x002b_fa10);
                let p = h8_chain(m, s0, 0x19b8);
                m.wv(p + 0x20, q);
            }
        }
        9 | 10 => {
            if st == 8 {
                let x = o(m);
                let v = vslot(m, x, 32, &[], &[]).1;
                let x = o(m);
                let n = m.rv(x + 0x1b0);
                let dot = v[0] * n[0] + v[1] * n[1] + v[2] * n[2];
                if -250.0 < dot {
                    st = 9;
                } else {
                    let x = o(m);
                    let v2 = vslot(m, x, 32, &[], &[]).1;
                    let len = (v2[0] * v2[0] + v2[1] * v2[1]).sqrt();
                    let (c0, c1, c2) = (m.rf(0x002b_fa20), m.rf(0x002b_fa24), m.rf(0x002b_fa28));
                    let f3 = c0 + len * (c1 + len * c2);
                    let lo = m.rf(0x002b_bb60);
                    let hi = m.rf(0x002b_bb64);
                    m.wv(sp, v2);
                    let f20 = if f3 < lo {
                        lo
                    } else if hi < f3 {
                        hi
                    } else {
                        f3
                    };
                    let (y, xx) = (m.rf(sp + 4), m.rf(sp));
                    let a = m.call(0x0024_58f0, &[], &[f4(y), f4(xx)]).1[0];
                    let a = a + f32::from_bits(0x4049_0fdb);
                    let x = o(m);
                    let h = m.rf(x + 0x1a8);
                    let d = m.call(0x0011_cf60, &[], &[f4(a - h)]).1[0];
                    let x = o(m);
                    let h = m.rf(x + 0x1a8);
                    m.call(0x0020_4660, &[x], &[f4(h + d * f20)]);
                    let x = o(m);
                    m.call(0x0020_4ca8, &[x], &[]);
                }
            }
            s3 = 0x21;
            s1 = slot_of(m, st);
        }
        _ => {}
    }
    let s2 = st;
    let x = o(m);
    m.call(0x001f_4bf8, &[x], &[]);
    m.w32(s0 + 4, s2);
    let t = m.rf(0x002d_ded0);
    m.wf(s0 + 8, t);
    if s2 != 0xffff_ffff {
        if s1 == 0 {
            let off = if (s2 as i32) < 8 { 0 } else { 0x20 };
            s1 = m.r32(s0 + 0xa0 + off);
        }
        let x = o(m);
        m.call(0x0013_0658, &[x + 0x1030], &[]);
        let x = o(m);
        m.call(0x001f_15b8, &[x + 0x1034], &[]);
        let x = o(m);
        m.call(0x0013_05b8, &[s1, x, s3, x + 0x1030], &[f4(0.0), f4(1.0)]);
        let x = o(m);
        m.call(0x001f_1448, &[0, x, 0xffff_ffff, 0, x + 0x1034], &[]);
        let x = o(m);
        vslot(m, x, 89, &[], &[]);
    }
    stack_pop(m, 0x30);
}

/// FUN_001efac0(o, rd): load nodes (+0x50/+0x54), edges (+0x58/+0x5c), per-mask groups (+0x60/+0x64) and
/// build each group's n x n hop-count matrix by repeated relaxation.
pub fn ee_sub_1efac0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let (s3, s4) = (a0, a1);
    let sp = stack_push(m, 0xb0);
    let s6 = sp + 0x20;
    m.call(0x0011_a1f8, &[s3 + 0x68, 0x14], &[]);
    m.call(0x001a_e478, &[sp], &[]);
    m.call(0x0013_f888, &[sp + 0x10], &[]);
    let x = m.r32(s3 + 0x10);
    m.call(0x0013_fb40, &[x, s4, sp + 0x10], &[]);
    if (m.r32(sp + 0x18) as i32) > 0 {
        let mut i: u32 = 0;
        loop {
            let arr = m.r32(sp + 0x1c);
            let o = m.r32((i << 2).wrapping_add(arr));
            i += 1;
            vslot(m, o, 8, &[], &[]);
            if (i as i32) >= m.r32(sp + 0x18) as i32 {
                break;
            }
        }
    }
    // nodes
    let n = m.call(0x0011_7bd0, &[s4], &[]).0;
    m.w32(s3 + 0x50, n);
    m.w32(s6, n << 3);
    m.w32(s6 + 4, 0x10);
    let v = m.call(0x001a_e1d0, &[s6], &[]).0;
    m.w32(s3 + 0x54, v);
    let mut i: i32 = 0;
    while i < m.r32(s3 + 0x50) as i32 {
        let e = m.r32(s3 + 0x54).wrapping_add((i as u32) << 3);
        i += 1;
        let v = m.call(0x0011_7db8, &[s4], &[]).0;
        m.w32(e, v);
        let v = m.call(0x0011_7db8, &[s4], &[]).0;
        m.w32(e + 4, v);
    }
    let mut i: i32 = 0;
    while i < m.r32(s3 + 0x50) as i32 {
        let e = m.r32(s3 + 0x54).wrapping_add((i as u32) << 3);
        let v = m.r32(e + 4);
        if v == 0xffff_ffff {
            m.w32(e + 4, v);
        } else if v == 0x20 {
            m.w32(e + 4, 0xffff_fffe);
        } else {
            let r = m.call(0x001f_0438, &[s3, v], &[]).0;
            m.w32(e + 4, r);
        }
        i += 1;
    }
    // edges
    let mut s7: u32 = 0;
    let n = m.call(0x0011_7bd0, &[s4], &[]).0;
    m.w32(s3 + 0x58, n);
    m.w32(s6, n.wrapping_mul(0x18));
    m.w32(s6 + 4, 0x10);
    let v = m.call(0x001a_e378, &[s6], &[]).0;
    m.w32(s3 + 0x5c, v);
    let mut s0: i32 = 0;
    let mut s5: u32 = 0;
    if (m.r32(s3 + 0x58) as i32) > 0 {
        loop {
            let s2 = m.r32(s3 + 0x5c).wrapping_add(s5);
            let v = m.call(0x0011_7db8, &[s4], &[]).0;
            let r = m.call(0x001f_0438, &[s3, v], &[]).0;
            m.w32(s2 + 4, r);
            let v = m.call(0x0011_7db8, &[s4], &[]).0;
            let r = m.call(0x001f_0438, &[s3, v], &[]).0;
            m.w32(s2 + 8, r);
            let v = m.call(0x0011_7db8, &[s4], &[]).0;
            m.w32(s2 + 0xc, v);
            let v = m.call(0x0011_7db8, &[s4], &[]).0;
            m.w32(s2 + 0x10, v);
            let f = m.call(0x0011_7e38, &[s4], &[]).1[0];
            m.wf(s2 + 0x14, f);
            let id = m.call(0x0011_7db8, &[s4], &[]).0;
            let mut drop = false;
            if id != 0xffff_ffff {
                let cnt = m.r32(sp + 0x18) as i32;
                let arr = m.r32(sp + 0x1c);
                let mut j: i32 = 0;
                while j < cnt {
                    let a2 = m.r32(((j as u32) << 2).wrapping_add(arr));
                    if m.r32(a2 + 0x18) == id {
                        m.w32(s2, a2);
                        break;
                    }
                    j += 1;
                }
                if m.r32(s2) == 0 {
                    drop = true;
                }
            }
            if drop {
                let c = m.r32(s3 + 0x58).wrapping_sub(1);
                s5 = s5.wrapping_sub(0x18);
                s0 -= 1;
                m.w32(s3 + 0x58, c);
            } else {
                s7 |= m.r32(s2 + 0xc) & 0xfffc;
            }
            s0 += 1;
            s5 = s5.wrapping_add(0x18);
            if s0 >= m.r32(s3 + 0x58) as i32 {
                break;
            }
        }
    }
    let c = m.r32(s3 + 0x58);
    let p = m.r32(s3 + 0x5c);
    m.w32(s6, c.wrapping_mul(0x18));
    m.w32(s6 + 4, 0x10);
    let v = m.call(0x001a_e238, &[s6, p], &[]).0;
    m.w32(s3 + 0x5c, v);
    m.call(0x0013_f810, &[s3, s4], &[]);
    // masks
    let t0 = sp + 0x30;
    let mut a2: u32 = 0;
    let mut a3: u32 = 4;
    loop {
        if a3 & s7 != 0 {
            let mut seen = false;
            for k in 0..a2 {
                if a3 & m.r32(t0 + 4 * k) != 0 {
                    seen = true;
                    break;
                }
            }
            if !seen {
                let n = m.r32(s3 + 0x58) as i32;
                let mut a1 = s7;
                if n > 0 {
                    let mut p = m.r32(s3 + 0x5c) + 0xc;
                    for _ in 0..n {
                        let v = m.r32(p);
                        p = p.wrapping_add(0x18);
                        if v & a3 != 0 {
                            a1 &= v;
                        }
                    }
                    let mut p = m.r32(s3 + 0x5c) + 0xc;
                    for _ in 0..n {
                        let v = m.r32(p);
                        p = p.wrapping_add(0x18);
                        if v & a1 != 0 && v & a3 == 0 {
                            a1 &= !v;
                        }
                    }
                }
                m.w32(t0 + (a2 << 2), a1);
                a2 += 1;
            }
        }
        a3 <<= 1;
        if (a3 as i32) >= 0x1001 {
            break;
        }
    }
    m.w32(s6, a2 << 3);
    m.w32(s6 + 4, 0x10);
    m.w32(s3 + 0x60, a2);
    let v = m.call(0x001a_e1d0, &[s6], &[]).0;
    m.w32(s3 + 0x64, v);
    let mut g: i32 = 0;
    while g < m.r32(s3 + 0x60) as i32 {
        let s2 = m.r32(s3 + 0x64).wrapping_add((g as u32) << 3);
        let msk = m.r32(sp + 0x30 + ((g as u32) << 2));
        m.w32(s2, msk);
        m.w32(s6 + 4, 0x10);
        let n = m.r32(s3 + 0x50);
        m.w32(s6, n.wrapping_mul(n));
        let v = m.call(0x001a_e1d0, &[s6], &[]).0;
        m.w32(s2 + 4, v);
        let t6 = g + 1;
        let mut r: i32 = 0;
        while r < m.r32(s3 + 0x50) as i32 {
            let mut c: i32 = 0;
            loop {
                let a1 = m.r32(s3 + 0x50);
                if c >= a1 as i32 {
                    break;
                }
                let idx = (r as u32).wrapping_mul(a1).wrapping_add(c as u32);
                let p = m.r32(s2 + 4).wrapping_add(idx);
                m.w8(p, if r == c { 0 } else { 0xff });
                c += 1;
            }
            r += 1;
        }
        let mut t3: i32 = 1;
        loop {
            let mut t1 = false;
            let mut j: i32 = 0;
            while j < m.r32(s3 + 0x58) as i32 {
                let e = m.r32(s3 + 0x5c).wrapping_add((j as u32).wrapping_mul(0x18));
                let msk = m.r32(s2);
                if m.r32(e + 0xc) & msk != 0 && (m.r32(s3 + 0x50) as i32) > 0 {
                    let t2 = t3 - 1;
                    let mut s1: i32 = 0;
                    loop {
                        let nn = m.r32(s3 + 0x50);
                        let a1 = (s1 as u32).wrapping_mul(nn);
                        let base = m.r32(s2 + 4);
                        let t0e = m.r32(e + 4);
                        let mut done = false;
                        if m.r8(base.wrapping_add(a1).wrapping_add(t0e)) as i8 as i32 == t2 {
                            let q = base.wrapping_add(a1).wrapping_add(m.r32(e + 8));
                            if (m.r8(q) as i8) < 0 {
                                m.w8(q, t3 as u8);
                                t1 = true;
                                done = true;
                            }
                        }
                        if !done && m.r32(e + 0xc) & 1 != 0 {
                            let nn = m.r32(s3 + 0x50);
                            let a0x = (s1 as u32).wrapping_mul(nn);
                            let q = base.wrapping_add(a0x).wrapping_add(m.r32(e + 8));
                            if m.r8(q) as i8 as i32 == t2 {
                                let p = base.wrapping_add(a0x).wrapping_add(t0e);
                                if (m.r8(p) as i8) < 0 {
                                    m.w8(p, t3 as u8);
                                    t1 = true;
                                }
                            }
                        }
                        s1 += 1;
                        if s1 >= m.r32(s3 + 0x50) as i32 {
                            break;
                        }
                    }
                }
                j += 1;
            }
            t3 += 1;
            if !t1 {
                break;
            }
        }
        g = t6;
    }
    m.call(0x001a_e4a0, &[sp, 2], &[]);
    stack_pop(m, 0xb0);
}

/// FUN_001e0020(o, x): add x to o's set at +0x88 (count +0x84, max 8) unless already present
pub fn ee_sub_1e0020<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let a3 = m.r32(a0 + 0x84) as i32;
    if a3 > 0 {
        let mut a2: i32 = 0;
        let mut v1 = a0 + 0x88;
        loop {
            if m.r32(v1) == a1 {
                return;
            }
            a2 += 1;
            v1 = v1.wrapping_add(4);
            if a2 >= a3 {
                break;
            }
        }
    }
    let v1 = m.r32(a0 + 0x84);
    if v1 < 8 {
        m.w32(a0.wrapping_add(v1 << 2) + 0x88, a1);
        m.w32(a0 + 0x84, v1.wrapping_add(1));
    }
}

/// FUN_00149ab8(w, o, k, x): skip if node {o, k, x} already in list w+0x3ca8, else FUN_00149a50.
/// `k` is a 64-bit key; the u32 ABI carries the low word, high word = sign extension.
/// WIRE-S2: 64-bit a2 = `a2` (low word) + `a2_hi` (a[4]; 0 when a caller passes only 4 args).
pub fn ee_sub_149ab8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, a2_hi: u32) {
    let mut t0 = m.r32(a0 + 0x3ca8);
    if t0 != 0 {
        let hi = a2_hi;
        let v1 = (a2 >> 16) as i16 as i32;
        let a6 = hi as i16 as i32;
        let t1 = hi;
        loop {
            'next: {
                if m.r32(t0) != a1 {
                    break 'next;
                }
                let h = m.r16(t0 + 6) as i16 as i32;
                if h != v1 {
                    break 'next;
                }
                let v0;
                if h == -1 || m.r16(t0 + 8) as i16 as i32 != a6 {
                    if m.r32(t0 + 8) != t1 {
                        break 'next;
                    }
                    v0 = m.r32(t0 + 0xc);
                } else {
                    v0 = m.r32(t0 + 0xc);
                }
                if v0 == a3 {
                    return;
                }
            }
            t0 = m.r32(t0 + 0x10);
            if t0 == 0 {
                break;
            }
        }
    }
    m.call(0x0014_9a50, &[a0, a1, a2, a3, a2_hi], &[]);
}

/// FUN_001da4b8(o, s, f): state transition on o+0x10 (jump table 0x2a7970)
pub fn ee_sub_1da4b8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, f12: f32) {
    let s0 = a0;
    let s1 = a1;
    if s1 == 0xffff_ffff {
        m.call(0x001d_a578, &[s0, 0xffff_ffff], &[]);
        return;
    }
    let st = m.r32(s0 + 0x10);
    match st {
        0 | 1 | 3 => {
            if st == 1 {
                m.call(0x001d_a5c8, &[s0, 0xffff_ffff], &[]);
            }
            m.w32(s0 + 0x18, s1);
            m.wf(s0 + 0x1c, f12);
            m.call(0x001d_a5c8, &[s0, 1], &[]);
        }
        2 => {
            if m.r32(s0 + 0x18) == s1 {
                m.wf(s0 + 0x1c, f12);
            } else {
                m.wf(s0 + 0x24, f12);
                m.w32(s0 + 0x20, s1);
            }
        }
        4 => {
            m.wf(s0 + 0x24, f12);
            m.w32(s0 + 0x20, s1);
        }
        _ => {}
    }
}

/// FUN_001d54b8: if DAT_002c2d8c is 2/3 build string id {0x17, 0/1} and FUN_00258200 with DAT_003e0fd8, 0x3d4a60
pub fn ee_sub_1d54b8<M: QMem + ?Sized>(m: &mut M) {
    let a0 = m.r32(0x002c_2d8c) as i32;
    let k = match a0 {
        2 => 0,
        3 => 1,
        _ => return,
    };
    let s1 = m.r32(0x003e_0fd8);
    let sp = stack_push(m, 0x60);
    m.w32(sp, 0x17);
    m.w32(sp + 4, k);
    m.call(0x0012_91a8, &[sp + 0x10, sp], &[]);
    m.call(0x0025_8200, &[sp + 0x20, s1, sp + 0x10, 0x003d_4a60], &[]);
    m.call(0x0012_9ec8, &[sp + 0x20], &[]);
    m.call(0x0012_9ec8, &[sp + 0x10], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x60);
}
