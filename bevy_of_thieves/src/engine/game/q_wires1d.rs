//! WIRE-S1: 1:1 QMem queue ports of live-frontier natives (shard1/2).
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem};

/// FUN_00113ab0: returns static descriptor 0x2babb0.
pub fn ee_sub_113ab0<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_abb0
}

/// FUN_00113bc0: returns static descriptor 0x2bad04.
pub fn ee_sub_113bc0<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_ad04
}

/// FUN_00113c40: returns static descriptor 0x2bada4.
pub fn ee_sub_113c40<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_ada4
}

/// FUN_00113cd0: returns static descriptor 0x2bae58.
pub fn ee_sub_113cd0<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_ae58
}

/// FUN_00113f90: returns static descriptor 0x2bb1c8.
pub fn ee_sub_113f90<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_b1c8
}

/// FUN_00114160: returns static descriptor 0x2bb40c.
pub fn ee_sub_114160<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_b40c
}

/// FUN_00114270: returns static descriptor 0x2bb560.
pub fn ee_sub_114270<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_b560
}

/// FUN_00114300: returns static descriptor 0x2bb614.
pub fn ee_sub_114300<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_b614
}

/// FUN_001143f0: returns static descriptor 0x2bb740.
pub fn ee_sub_1143f0<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_b740
}

/// FUN_00114440: returns static descriptor 0x2bb7a4.
pub fn ee_sub_114440<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_b7a4
}

/// FUN_001144b0: returns static descriptor 0x2bb830.
pub fn ee_sub_1144b0<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_b830
}

/// FUN_001145d0: returns static descriptor 0x2bb998.
pub fn ee_sub_1145d0<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_b998
}

/// FUN_001179d8(s, a, b): BinaryInputStream set_lz: flags |= 2, +0x20 = a, +0x24 = b.
pub fn ee_sub_1179d8<M: QMem + ?Sized>(m: &mut M, s: u32, a: u32, b: u32) {
    let f = m.r32(s);
    m.w32(s + 0x24, b);
    m.w32(s + 0x20, a);
    m.w32(s, f | 2);
}

/// FUN_002321b8(x): snd command 6 (4-byte payload {x}) via FUN_00232838.
pub fn ee_sub_2321b8<M: QMem + ?Sized>(m: &mut M, x: u32) -> u32 {
    let sp = stack_push(m, 0x20);
    m.w32(sp, x);
    let r = m.call(0x0023_2838, &[6, 4, sp, 0, 0], &[]).0;
    stack_pop(m, 0x20);
    r
}

/// FUN_001177a8(s): BinaryInputStream ctor: zero fields, +0x30 = 0x2ca5c0; returns s.
pub fn ee_sub_1177a8<M: QMem + ?Sized>(m: &mut M, s: u32) -> u32 {
    m.w32(s, 0);
    m.w32(s + 0x30, 0x002c_a5c0);
    for o in [0x10, 4, 0x14, 8, 0x18, 0x1c, 0xc, 0x20, 0x24, 0x2c, 0x28] {
        m.w32(s + o, 0);
    }
    s
}

/// FUN_0017eeb8(p): rail "at target": 0 if owner busy (+0xe4), else |+0x38 - pos(owner)| <= limit.
pub fn ee_sub_17eeb8<M: QMem + ?Sized>(m: &mut M, p: u32) -> u32 {
    let o = m.r32(p + 0x30);
    if m.r32(o + 0xe4) != 0 {
        return 0;
    }
    let f2 = if m.r32(p + 0x34) != 0 {
        m.call(0x001e_6648, &[o], &[]).1[0]
    } else {
        m.call(0x001e_6620, &[o], &[]).1[0]
    };
    let d = (m.rf(p + 0x38) - f2).abs();
    if m.rf(0x002b_e7b8) < d {
        0
    } else {
        1
    }
}

/// FUN_001eae40(r): world read entry sections: count, alloc 32-byte entries, read each, count type-1
/// entries into 0x3dcf9c, then keep max(type1, rest).
pub fn ee_sub_1eae40<M: QMem + ?Sized>(m: &mut M, r: u32) {
    const N: u32 = 0x003d_cff8;
    const P: u32 = 0x003d_cffc;
    const C: u32 = 0x003d_cf9c;
    let n = m.call(0x0011_7c20, &[r], &[]).0;
    let sp = stack_push(m, 16);
    m.w32(sp, n << 5);
    m.w32(sp + 4, 0x10);
    m.w32(N, n);
    let a = m.call(0x001a_e378, &[sp], &[]).0;
    stack_pop(m, 16);
    m.w32(P, a);
    m.w32(C, 0);
    let mut i: i32 = 0;
    if (m.r32(N) as i32) > 0 {
        loop {
            let e = m.r32(P).wrapping_add((i as u32) << 5);
            m.call(0x001e_a8a8, &[r, e], &[]);
            let e = ((i as u32) << 5).wrapping_add(m.r32(P));
            if m.r8(e + 4) == 1 {
                let c = m.r32(C).wrapping_add(1);
                m.w32(C, c);
            }
            i += 1;
            if i >= m.r32(N) as i32 {
                break;
            }
        }
    }
    let c = m.r32(C) as i32;
    let rest = (m.r32(N) as i32).wrapping_sub(c);
    if c < rest {
        m.w32(C, rest as u32);
    }
}

/// FUN_00215f40(t): rich text layout: word-wrap pass inserting '\n' when width (+0x74) is exceeded;
/// returns the line count.
pub fn ee_sub_215f40<M: QMem + ?Sized>(m: &mut M, t: u32) -> u32 {
    let mut lines: u32 = 1;
    let mut w: f32 = 0.0;
    let zero: f32 = 0.0;
    m.call(0x0021_5ca8, &[t, 0], &[]);
    loop {
        let c = m.call(0x0021_5d40, &[t], &[]).0;
        if c == 0 {
            return lines;
        }
        if c == 10 {
            w = 0.0;
            lines += 1;
        } else {
            let f = m.r32(t + 8);
            let cw = m.call(0x0021_6b80, &[f, c], &[]).1[0];
            w += cw * m.rf(t + 0x88);
        }
        let lim = m.rf(t + 0x74);
        if lim == zero || !(lim < w) {
            continue;
        }
        let cur = m.r32(t + 4);
        let start = m.r32(t);
        let mut a = cur.wrapping_sub(1);
        if a >= start && m.r8(cur.wrapping_sub(1)) != 10 {
            loop {
                let b = m.r8(a);
                if b == 0x20 || b == 9 {
                    let n = m.r8(a + 1) as u32;
                    if m.r8(n + 0x002a_a6a1) & 0x10 == 0 {
                        lines += 1;
                        m.w32(t + 4, a + 1);
                        w = 0.0;
                        m.w8(a, 10);
                        break;
                    }
                }
                a = a.wrapping_sub(1);
                if a < start || m.r8(a) == 10 {
                    break;
                }
            }
        }
        if w != zero {
            let p = m.r32(t + 4);
            if m.r8(p) != 0 {
                m.w8(p, 10);
                lines += 1;
                w = 0.0;
            }
        }
    }
}

/// FUN_00231c48(): snd_FlushSoundCommands: run/cancel pending command callbacks, flush, finish
/// stop/load completion callbacks; returns 1 while work is still pending.
pub fn ee_sub_231c48<M: QMem + ?Sized>(m: &mut M) -> u32 {
    if m.r32(0x002c_4640) != 0 && m.call(0x0023_1e90, &[], &[]).0 != 0 {
        if m.r32(0x002c_465c) != 0 {
            let f = m.r32(0x002c_46a4);
            if f != 0 {
                let a0 = m.r32(0x0044_3844);
                let a1 = m.r32(0x002c_46a8);
                m.call(f, &[a0, a1], &[]);
            }
            m.w32(0x002c_46a4, 0);
            m.w32(0x002c_465c, 0);
        } else {
            let o = if m.r32(0x002c_4680) ^ 1 != 0 { 4 } else { 0 };
            let lst = 0x002c_4660 + o;
            let l0 = m.r32(lst);
            if (m.r32(l0) as i32) > 0 {
                let mut i: i32 = 0;
                loop {
                    let e = ((i as u32) << 4).wrapping_add(m.r32(0x002c_4670 + o));
                    let f = m.r32(e);
                    if f != 0 {
                        let a1 = m.r32(e + 8);
                        let b = m.r32(0x002c_4678 + o);
                        let a0 = m.r32(((i as u32) << 2).wrapping_add(b) + 4);
                        m.call(f, &[a0, a1], &[]);
                    }
                    i += 1;
                    let l0 = m.r32(lst);
                    if i >= m.r32(l0) as i32 {
                        break;
                    }
                }
            }
        }
    }
    if m.r32(0x002c_4688) != 0 {
        m.call(0x0023_4ca0, &[0], &[]);
        let a0 = m.r32(0x0044_8480);
        if a0 != 0xffff_ffff {
            let f = m.r32(0x002c_4690);
            if f != 0 {
                let a1 = m.r32(0x002c_4698);
                m.w32(0x002c_4690, 0);
                m.w32(0x002c_4698, 0);
                m.w32(0x002c_469c, 0);
                m.call(f, &[a0, a1], &[]);
            }
            m.w32(0x0044_8480, 0);
            m.w32(0x002c_4688, 0);
        }
    }
    if m.r32(0x002c_4640) == 0 {
        let idx = m.r32(0x002c_4680);
        let l = m.r32((idx << 2).wrapping_add(0x002c_4660));
        if m.r32(l) != 0 && m.r32(0x002c_4684) == 0 {
            m.call(0x0023_2bf8, &[], &[]);
        }
    }
    if m.r32(0x002c_4654) != 0 {
        m.call(0x0023_2da8, &[1], &[]);
        if m.r32(0x002c_4658) != 0 {
            m.w32(0x002c_4654, 0);
            let f = m.r32(0x002c_4650);
            m.w32(0x002c_4658, 0);
            if f != 0 {
                m.call(f, &[1], &[]);
            }
        }
    }
    (m.r32(0x002c_4640) != 0 || m.r32(0x002c_4688) != 0) as u32
}
