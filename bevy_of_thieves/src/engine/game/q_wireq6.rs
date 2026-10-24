//! WIRE-Q6: 1:1 queue ports (against `QMem`) of functions the all-levels live run (tools/wireall.sh) reached
//! with only native ports (test_out/wireall_frontier.tsv). Ported from the EE asm; callees go through
//! `m.call` / `vslot` (the live dispatch routes ported ones).
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

const WORLD: u32 = 0x003d_d490;

/// FUN_00231368: refresh the cached world xform (+0xa0..+0xdf) of a bone-world record.
/// +0x50 parent: if +0x54 < parent+0xa4 { +0x54 = it; +0xa0 = FUN_0011c168(parent + 0x40, +0x60) }.
/// no parent: if +0x54 < 0 copy +0x60..+0x9f to +0xa0.
pub fn ee_sub_231368<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let p = m.r32(a0 + 0x50);
    if p != 0 {
        let s = m.r32(p + 0xa4);
        if (m.r32(a0 + 0x54) as i32) >= s as i32 {
            return;
        }
        m.w32(a0 + 0x54, s);
        let sp = stack_push(m, 0x40);
        m.call(0x0011_c168, &[sp, p + 0x40, a0 + 0x60], &[]);
        for k in 0..16 {
            let w = m.r32(sp + 4 * k);
            m.w32(a0 + 0xa0 + 4 * k, w);
        }
        stack_pop(m, 0x40);
    } else {
        if (m.r32(a0 + 0x54) as i32) >= 0 {
            return;
        }
        for k in 0..16 {
            let w = m.r32(a0 + 0x60 + 4 * k);
            m.w32(a0 + 0xa0 + 4 * k, w);
        }
    }
}

/// FUN_00182c70: +0x14 = -1, +8 = DAT_002a5680, +0x1c = 7; FUN_001a6ca8(vt(+0x20)[+0x34](o + vt[+0x30]), 3).
pub fn ee_sub_182c70<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let f = m.r32(0x002a_5680);
    m.w32(a0 + 0x14, u32::MAX);
    m.w32(a0 + 8, f);
    let vt = m.r32(a0 + 0x20);
    m.w32(a0 + 0x1c, 7);
    let adj = m.r16(vt + 0x30) as i16 as i32 as u32;
    let fp = m.r32(vt + 0x34);
    let r = m.call(fp, &[a0.wrapping_add(adj)], &[]).0;
    m.call(0x001a_6ca8, &[r, 3], &[]);
}

/// FUN_001f7918(o, mask, b): o+0x110+i = b for every set bit i < 0x1a of mask.
pub fn ee_sub_1f7918<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    for i in 0..0x1a {
        if a1 & (1 << i) != 0 {
            m.w8(a0 + 0x110 + i, a2 as u8);
        }
    }
}

/// FUN_0017f138: FUN_001f36a8(o); slot 82 (o).
pub fn ee_sub_17f138<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001f_36a8, &[a0], &[]);
    vslot(m, a0, 82, &[], &[]);
}

/// FUN_00140af8(o, wide): Jt +0x10ec == 4 && +0x10e4 != 3: horizontal distance from the +0x10e8 object
/// below r (DAT_002bcc7c if it is-a 0x3a else DAT_002bcc80) and dz in (lo, hi) -> 1 (wide widens both).
pub fn ee_sub_140af8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    if m.r32(a0 + 0x10ec) != 4 || m.r32(a0 + 0x10e4) == 3 {
        return 0;
    }
    let t = m.r32(a0 + 0x10e8);
    let mut r = if m.call(0x0013_2b10, &[t, 0x3a], &[]).0 != 0 { m.rf(0x002b_cc7c) } else { m.rf(0x002b_cc80) };
    let mut lo = m.rf(0x002b_cc84);
    let mut hi = m.rf(0x002b_cc88);
    if a1 != 0 {
        let d = m.rf(0x002b_cc90);
        hi += d;
        r += m.rf(0x002b_cc8c);
        lo -= d;
    }
    let p = m.call(0x001a_62e8, &[t], &[]).1;
    let q = m.call(0x001a_62e8, &[a0], &[]).1;
    let d = [p[0] - q[0], p[1] - q[1], p[2] - q[2]];
    let h = (d[0] * d[0] + d[1] * d[1]).sqrt();
    if h < r && lo < d[2] && d[2] < hi {
        1
    } else {
        0
    }
}

/// FUN_0017f5f8(o, k): k != 0 && +0x188 != 0: FUN_001f7598(&{o, _, k, 0x100, -(+0x188), pos = FUN_001a62e8(o),
/// slot 32 (o)}); +0x190 -> FUN_0015c150; then slot 8 (o).
pub fn ee_sub_17f5f8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if a1 == 0 || m.r32(a0 + 0x188) == 0 {
        return;
    }
    let sp = stack_push(m, 0x60);
    m.call(0x0023_33b4, &[sp, 0, 0x60], &[]);
    m.w32(sp + 8, a1);
    m.w32(sp, a0);
    let p = m.call(0x001a_62e8, &[a0], &[]).1;
    m.wv(sp + 0x20, p);
    let q = vslot(m, a0, 32, &[], &[]).1;
    m.wv(sp + 0x30, q);
    let n = m.r32(a0 + 0x188).wrapping_neg();
    m.w32(sp + 0xc, 0x100);
    m.w32(sp + 0x10, n);
    m.call(0x001f_7598, &[sp], &[]);
    stack_pop(m, 0x60);
    let s = m.r32(a0 + 0x190);
    if s != 0 {
        m.call(0x0015_c150, &[s], &[]);
    }
    vslot(m, a0, 8, &[], &[]);
}

/// FUN_0017f408(o, n, ents): for each of n 0x10-byte entries with id != -1: x = FUN_0015e938(WORLD, 4, id, o);
/// is-a 0x6d -> +4 = x, else is-a 0x6f -> +8 = x.
pub fn ee_sub_17f408<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    let mut n = a1 as i32;
    if n <= 0 {
        return;
    }
    let mut e = a2;
    while n != 0 {
        let id = m.r32(e);
        if id != u32::MAX {
            let x = m.call(0x0015_e938, &[WORLD, 4, id, a0], &[]).0;
            if x != 0 {
                if m.call(0x0013_2b10, &[x, 0x6d], &[]).0 != 0 {
                    m.w32(e + 4, x);
                } else if m.call(0x0013_2b10, &[x, 0x6f], &[]).0 != 0 {
                    m.w32(e + 8, x);
                }
            }
        }
        n -= 1;
        e += 0x10;
    }
}

/// FUN_001938c0(jt): first in-range (+8 <= DAT_002bf660) entry of the 0x3e1b10 list (count 0x3e1b08, stride
/// 0x30) whose object kind maps to an action -> FUN_001928f8(jt, action, obj, +4); returns 1, else 0.
pub fn ee_sub_1938c0<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let n = m.r32(WORLD + 0x4678) as i32;
    let mut e = WORLD + 0x4680;
    let mut i = 0;
    while i < n {
        if m.rf(0x002b_f660) < m.rf(e + 8) {
            return 0;
        }
        let o = m.r32(e);
        let isa = |m: &mut M, k: u32| m.call(0x0013_2b10, &[o, k], &[]).0 != 0;
        let act: i32 = if isa(m, 0x13) || isa(m, 0xb) {
            if m.r32(a0) == m.r32(0x002e_1e40) { 6 } else { 5 }
        } else if isa(m, 0x1b) {
            8
        } else if isa(m, 0xc) || isa(m, 0x22) {
            0
        } else if isa(m, 0x26) {
            let p = m.r32(e);
            m.r32(p + 0x10c) as i32
        } else if isa(m, 0x23) {
            7
        } else {
            -1
        };
        if act != -1 {
            let x = m.r32(e + 4);
            m.call(0x0019_28f8, &[a0, act as u32, o, x], &[]);
            return 1;
        }
        i += 1;
        e += 0x30;
    }
    0
}

/// FUN_00136338(o): binoc-state (+0x16bc) driven update, repeated while the state changes.
pub fn ee_sub_136338<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let mut s = m.r32(a0 + 0x16bc) as i32;
    loop {
        if s == -1 {
            if m.call(0x0014_43d8, &[a0], &[]).0 != 0 && m.r32(a0 + 0xf20) != 2 {
                let f = m.rf(0x002d_ded0) - m.rf(a0 + 0x1700);
                let k = if m.rf(0x002b_c994) < f { 0 } else { 1 };
                m.call(0x0013_62f8, &[a0, k], &[]);
            }
            let k: i32 = match m.r32(a0 + 0x13d0) {
                0 => {
                    m.call(0x0013_6dc0, &[a0 + 0x1710, 0], &[]);
                    if m.r32(a0 + 0x181c) != 0 && m.r32(a0 + 0x1714) != u32::MAX && m.r32(a0 + 0x1034) != 0 {
                        let c = m.r32(a0 + 0x1028);
                        if (m.call(0x001f_0400, &[c, 0x17b], &[]).0 as i32) >= 0 {
                            let c = m.r32(a0 + 0x1034);
                            m.call(0x001f_09d8, &[c, 0x17b], &[]);
                        }
                    }
                    i32::MIN
                }
                1 | 2 | 3 | 7 => 1,
                5 => 2,
                _ => -1,
            };
            if k != i32::MIN {
                m.call(0x0013_6dc0, &[a0 + 0x1710, k as u32], &[]);
            }
        } else if (0..2).contains(&s) {
            if m.call(0x0014_43d8, &[a0], &[]).0 == 0 {
                m.call(0x0013_62f8, &[a0, u32::MAX], &[]);
            } else if m.r32(a0 + 0x16bc) == 1 {
                let f = m.rf(0x002d_ded0) - m.rf(a0 + 0x1700);
                if m.rf(0x002b_c994) < f {
                    m.call(0x0013_62f8, &[a0, 0], &[]);
                }
            } else {
                let c = m.r32(a0 + 0x150);
                if m.call(0x001c_46c0, &[c, 8], &[[0.25, 0.0, 0.0, 0.0]]).0 != 0 {
                    m.call(0x0013_57c0, &[a0, 0x8_0000], &[]);
                }
            }
        }
        let v = m.r32(a0 + 0x16bc) as i32;
        if v == s {
            return;
        }
        s = v;
    }
}

/// FUN_00153de0(sensor): for each of +0xfc 0x10-byte targets at +0x100: FUN_00154080(o, e, FUN_001541d8(o, e[0], e[1])).
pub fn ee_sub_153de0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let mut i = 0;
    let mut e = a0 + 0x100;
    while i < m.r32(a0 + 0xfc) as i32 {
        let (x, y) = (m.r32(e), m.r32(e + 4));
        let v = m.call(0x0015_41d8, &[a0, x, y], &[]).0;
        m.call(0x0015_4080, &[a0, e, v], &[]);
        i += 1;
        e += 0x10;
    }
}

/// FUN_00136a28(o, k): gadget slot record for button k (k-4 < 10): 4 -> +0x1860, 5 -> +0x1824,
/// 7 -> +0x1830, 9 -> +0x183c, 11 -> +0x1848, 13 -> +0x1854, else 0.
pub fn ee_sub_136a28<M: QMem + ?Sized>(_m: &mut M, a0: u32, a1: u32) -> u32 {
    match a1 {
        4 => a0 + 0x1860,
        5 => a0 + 0x1824,
        7 => a0 + 0x1830,
        9 => a0 + 0x183c,
        11 => a0 + 0x1848,
        13 => a0 + 0x1854,
        _ => 0,
    }
}

/// FUN_00136870(o, k): may button k act now (per Jt state +0x13d0 and the gadget bound to it).
pub fn ee_sub_136870<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    if m.r64(a0 + 0x1818) != 0 {
        return 0;
    }
    if m.call(0x0014_43d8, &[a0], &[]).0 != 0 {
        return 0;
    }
    let sub = |m: &mut M, f: u32, off: u32| m.call(f, &[a0 + off, a1], &[]).0;
    match m.r32(a0 + 0x13d0) {
        0 | 1 | 5 | 7 => {}
        2 => {
            if sub(m, 0x0019_c8d0, 0x2c0) == 0 {
                return 0;
            }
        }
        3 => {
            if sub(m, 0x0019_4558, 0x4c0) == 0 {
                return 0;
            }
        }
        0xd => {
            if sub(m, 0x0019_eef0, 0xf50) == 0 {
                return 0;
            }
        }
        0xc => {
            if m.r32(a0 + 0x11b4).wrapping_sub(4) >= 3 {
                return 0;
            }
            return match a1 {
                4 | 7 | 9 | 11 | 13 => {
                    let p = m.r32(a0 + 0x11c0);
                    (m.r32(p + 0x15c) == a1) as u32
                }
                _ => 0,
            };
        }
        _ => return 0,
    }
    match a1 {
        1 | 2 | 8 | 10 | 12 | 14 => 1,
        3 => {
            if m.r32(a0 + 0x12f0) == 0 || m.r32(0x002d_fbc0) & 2 == 0 {
                0
            } else {
                (m.call(0x0014_fd50, &[0x002d_e2e0], &[]).0 == 0) as u32
            }
        }
        4 | 5 | 7 | 9 | 11 | 13 => {
            let r = m.call(0x0013_6a28, &[a0, a1], &[]).0;
            let g = m.r32(r + 8);
            if g != 0 {
                (m.call(0x0022_4590, &[g], &[]).0 != 0) as u32
            } else {
                let c = m.r32(r + 4);
                if c == 0 {
                    0
                } else {
                    (m.call(0x001c_ade0, &[c, 2], &[]).0 != 0) as u32
                }
            }
        }
        _ => 0,
    }
}

/// FUN_0017f4c0(o, p): FUN_001f15b8(o+0x198); FUN_00130658(o+0x194); with p: FUN_001305b8(p[1], o, p[3],
/// o+0x194, 0.0, 1.0); FUN_001f1448(p[2], o, -1, p[3], o+0x198).
pub fn ee_sub_17f4c0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.call(0x001f_15b8, &[a0 + 0x198], &[]);
    m.call(0x0013_0658, &[a0 + 0x194], &[]);
    if a1 == 0 {
        return;
    }
    let (p1, p3) = (m.r32(a1 + 4), m.r32(a1 + 0xc));
    m.call(0x0013_05b8, &[p1, a0, p3, a0 + 0x194], &[[0.0; 4], [1.0, 0.0, 0.0, 0.0]]);
    let (p2, p3) = (m.r32(a1 + 8), m.r32(a1 + 0xc));
    m.call(0x001f_1448, &[p2, a0, u32::MAX, p3, a0 + 0x198], &[]);
}

/// FUN_0017f550(o): +0x194 -> FUN_00131968(it); else +0x198 == 0 -> 1; else FUN_001f07e8 == FUN_001f07c8 (+0x198).
pub fn ee_sub_17f550<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let a = m.r32(a0 + 0x194);
    if a != 0 {
        return m.call(0x0013_1968, &[a], &[]).0;
    }
    let b = m.r32(a0 + 0x198);
    if b == 0 {
        return 1;
    }
    let x = m.call(0x001f_07e8, &[b], &[]).0;
    let b = m.r32(a0 + 0x198);
    (x == m.call(0x001f_07c8, &[b], &[]).0) as u32
}

/// FUN_00110b10: o = FUN_001ae4f8(0xe0); o+0xc = vtable 0x2dd090; o+0 = o+4 = 0; returns o.
pub fn ee_sub_110b10<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let o = m.call(0x001a_e4f8, &[0xe0], &[]).0;
    m.w32(o + 0xc, 0x002d_d090);
    m.w32(o, 0);
    m.w32(o + 4, 0);
    o
}

/// quad copy of DAT_002a2dd0 to jt+0x190 (the default blend weights)
fn jt_190<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let q = m.rv(0x002a_2dd0);
    m.wv(a0 + 0x190, q);
}

/// FUN_00135248(jt, st): SetJtState — leave the old +0x13d0 state, enter `st`, restart the anim/sm blends.
pub fn ee_sub_135248<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let old = m.r32(a0 + 0x13d0);
    if a1 == old {
        return;
    }
    let (n58, n5c) = (0x58, 0x5c);
    let lods = |m: &mut M, k: u32| {
        let a = m.r32(a0 + n58);
        m.call(0x0023_03a0, &[a0, a, k], &[]);
        let b = m.r32(a0 + n5c);
        m.call(0x0023_03a0, &[a0, b, k], &[]);
    };
    let s1 = a1 as i32;
    match old {
        0 => {
            lods(m, 2);
            m.call(0x0014_2880, &[a0], &[]);
        }
        1 | 7 => {
            let c = m.r32(a0 + 0x1030);
            let d = m.r32(c + 0x14);
            let f = m.rf(c + 0x1c) / m.rf(d + 0x74);
            m.wf(a0 + 0x1a0, f);
        }
        2 => {
            if m.r32(a0 + 0x108c) == 0 {
                let s0 = if s1 < 0 {
                    0
                } else if s1 < 2 {
                    m.r32(a0 + 0x16d0)
                } else if s1 == 3 {
                    let v = m.r32(a0 + 0x4c8) as i32;
                    if v == 1 {
                        0
                    } else if v <= 0 || v >= 5 || v < 3 {
                        m.r32(a0 + 0x16d0)
                    } else {
                        m.r32(a0 + 0x16d4)
                    }
                } else {
                    0
                };
                if s0 != 0 {
                    vslot(m, s0, 7, &[], &[]);
                    m.call(0x0015_c150, &[s0], &[]);
                    vslot(m, s0, 8, &[], &[]);
                }
            }
            m.call(0x0019_5fb8, &[a0 + 0x2c0, u32::MAX, 0], &[]);
        }
        3 => {
            m.call(0x0019_2bd8, &[a0 + 0x4c0, u32::MAX], &[]);
        }
        4 => {
            m.call(0x001a_0b40, &[a0 + 0xdf0, u32::MAX], &[]);
        }
        6 => {
            let g = m.r32(a0 + 0x186c);
            if g != 0 {
                vslot(m, g, 75, &[1], &[]);
                m.call(0x0013_6298, &[a0], &[]);
            }
            m.call(0x0013_6040, &[a0, u32::MAX], &[]);
        }
        8 => {
            m.call(0x0013_6530, &[a0, u32::MAX], &[]);
            vslot(m, a0, 78, &[1], &[]);
        }
        9..=0xb => lods(m, 2),
        0xc => {
            m.call(0x0022_06c8, &[a0 + 0x11b0, u32::MAX], &[]);
        }
        0xd => {
            m.call(0x0019_eab0, &[a0 + 0xf50, u32::MAX], &[]);
        }
        _ => {}
    }
    let prev = m.r32(a0 + 0x13d0);
    m.w32(a0 + 0x13d0, a1);
    let (mut s3, mut s6, mut s5, mut s4, mut s7) = (0u32, 0u32, 1u32, u32::MAX, 4u32);
    let (mut f20, mut f21) = (0.0f32, 1.0f32);
    let t = m.r32(0x002d_ded0);
    m.w32(a0 + 0x13d4, t);
    let anim = |m: &mut M, s3: u32, a: u32| {
        let f = m.rf(a0 + 0x1a0) * m.rf(s3 + 0x74);
        m.call(0x0020_bd68, &[a], &[]);
        f
    };
    match a1 {
        0 => {
            lods(m, 1);
            if m.r32(a0 + 0x181c) != 0 {
                s3 = m.r32(a0 + 0x1820);
                s5 = 9;
            } else {
                s6 = m.r32(a0 + 0x1028);
                s4 = if prev == 2 {
                    0x178
                } else if prev == 7 {
                    0x176
                } else {
                    0x175
                };
                if (m.call(0x001f_0400, &[s6, s4], &[]).0 as i32) < 0 {
                    s4 = u32::MAX;
                }
            }
            jt_190(m, a0);
            s7 = m.call(0x0014_30c8, &[a0], &[]).0;
            m.call(0x0014_2868, &[a0, 1], &[]);
        }
        1 => {
            s3 = m.r32(a0 + 0x1324);
            s5 = 9;
            f20 = anim(m, s3, a0 + 0x1320);
        }
        4 => {
            m.call(0x0019_cd38, &[a0 + 0xf00, u32::MAX], &[]);
        }
        5 => {
            s3 = m.r32(a0 + 0x14e0);
            s5 = 9;
        }
        6 => {
            m.call(0x0013_6040, &[a0, 0], &[]);
            return;
        }
        7 => {
            s3 = m.r32(a0 + 0x1604);
            s5 = 9;
            f20 = anim(m, s3, a0 + 0x1600);
        }
        8 => {
            m.call(0x0013_6530, &[a0, 0], &[]);
            return;
        }
        9..=0xb => {
            if a1 == 9 {
                s6 = m.r32(a0 + 0x102c);
            }
            lods(m, 1);
            jt_190(m, a0);
        }
        0xc => return,
        0xd => {
            m.call(0x0019_eab0, &[a0 + 0xf50, 0], &[]);
            return;
        }
        0xe => {
            s3 = m.r32(a0 + 0x1878);
            s5 = 9;
            if s3 == 0 {
                s3 = m.r32(a0 + 0x1374);
                f21 = 1.5;
            }
        }
        _ => {}
    }
    m.call(0x0013_0658, &[a0 + 0x1030], &[]);
    m.call(0x001f_15b8, &[a0 + 0x1034], &[]);
    m.call(0x0013_05b8, &[s3, a0, s5, a0 + 0x1030], &[[f20, 0.0, 0.0, 0.0], [f21, 0.0, 0.0, 0.0]]);
    m.call(0x001f_14f8, &[s6, a0, s4, s5, s7, a0 + 0x1034], &[]);
    vslot(m, a0, 89, &[], &[]);
}

/// FUN_00130658(ref): p = *ref; if p { FUN_00130b18(p); slot 1 (*ref); *ref = 0 }.
pub fn ee_sub_130658<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let p = m.r32(a0);
    if p == 0 {
        return;
    }
    m.call(0x0013_0b18, &[p], &[]);
    let p = m.r32(a0);
    vslot(m, p, 1, &[], &[]);
    m.w32(a0, 0);
}

/// FUN_001f15b8(ref): p = *ref; if p { FUN_001f0598(p); slot 1 (*ref); *ref = 0 }.
pub fn ee_sub_1f15b8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let p = m.r32(a0);
    if p == 0 {
        return;
    }
    m.call(0x001f_0598, &[p], &[]);
    let p = m.r32(a0);
    vslot(m, p, 1, &[], &[]);
    m.w32(a0, 0);
}

/// FUN_001305b8(a, o, k, ref, f12, f13): FUN_00130658(ref); with a: FUN_0012f9b8(a, o, k, ref, f12, f13);
/// slot 0 (*ref).
pub fn ee_sub_1305b8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, f12: f32, f13: f32) {
    m.call(0x0013_0658, &[a3], &[]);
    if a0 == 0 {
        return;
    }
    m.call(0x0012_f9b8, &[a0, a1, a2, a3], &[[f12, 0.0, 0.0, 0.0], [f13, 0.0, 0.0, 0.0]]);
    let p = m.r32(a3);
    vslot(m, p, 0, &[], &[]);
}

/// FUN_0012e460(ref): p = *ref; if p { FUN_0012d020(p); FUN_0012e210(p); *ref = 0 }.
pub fn ee_sub_12e460<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let p = m.r32(a0);
    if p == 0 {
        return;
    }
    m.call(0x0012_d020, &[p], &[]);
    m.call(0x0012_e210, &[p], &[]);
    m.w32(a0, 0);
}

/// FUN_0011e1a0(ang, r, z): FUN_0011d608(ang, &a, &b); returns {b * r, a * r, z, sign-extension of z}.
pub fn ee_sub_11e1a0<M: QMem + ?Sized>(m: &mut M, f12: f32, f13: f32, f14: f32) -> V4 {
    let sp = stack_push(m, 0x10);
    m.call(0x0011_d608, &[sp, sp + 4], &[[f12, 0.0, 0.0, 0.0]]);
    let (a, b) = (m.rf(sp), m.rf(sp + 4));
    stack_pop(m, 0x10);
    let hi = if (f14.to_bits() as i32) < 0 { u32::MAX } else { 0 };
    [b * f13, a * f13, f14, f32::from_bits(hi)]
}

/// FUN_0011d608(&sin, &cos, f12 angle): QMem side of the polynomial sincos (anim::motion::ee_sub_11d608).
pub fn ee_sub_11d608<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, f12: f32) {
    let (s, c) = crate::engine::anim::motion::ee_sub_11d608(f12);
    m.wf(a0, s);
    m.wf(a1, c);
}
