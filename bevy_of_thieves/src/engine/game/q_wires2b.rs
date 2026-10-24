//! WIRE-S2: 1:1 QMem ports of live-frontier natives (shard 3/4).
#![allow(unused)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

const T: u32 = 0x002d_ded0;
const GS: u32 = 0x003d_d490;

fn bit60<M: QMem + ?Sized>(m: &mut M, p: u32) -> bool {
    (m.r64(p + 0x28) >> 60) & 1 != 0
}

/// FUN_00191090: init: +0=a1, +0x4fc/+8/+0xc = -1; FUN_0020b840(+0x180/+0x240, a1, 0x6f); FUN_001cc158(+0x440).
pub fn ee_sub_191090<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.w32(a0, a1);
    m.w32(a0 + 0x4fc, 0xffff_ffff);
    m.w32(a0 + 8, 0xffff_ffff);
    m.w32(a0 + 0xc, 0xffff_ffff);
    m.call(0x0020_b840, &[a0 + 0x180, a1, 0x6f], &[]);
    m.call(0x0020_b840, &[a0 + 0x240, a1, 0x6f], &[]);
    m.call(0x001c_c158, &[a0 + 0x440], &[]);
}

/// FUN_00153f18: FUN_00154180 == 0 and +0xfc < 8 -> push a1 into slot table, FUN_00154080.
pub fn ee_sub_153f18<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if m.call(0x0015_4180, &[a0, a1], &[]).0 != 0 {
        return;
    }
    let n = m.r32(a0 + 0xfc);
    if n < 8 {
        m.w32(a0 + 0xfc, n.wrapping_add(1));
        let p = a0.wrapping_add((n << 4).wrapping_add(0x100));
        m.w32(p, a1);
        m.call(0x0015_4080, &[a0, p, 0], &[]);
    }
}

/// FUN_002036e8: +0xec == 0 -> +0xf0 = a1; else FUN_00164640(+0xec, 2), player FUN_0019d948 if kind 0x34.
pub fn ee_sub_2036e8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let h = m.r32(a0 + 0xec);
    if h == 0 {
        m.w32(a0 + 0xf0, a1);
        return;
    }
    m.call(0x0016_4640, &[h, 2], &[]);
    let s = m.call(0x001c_4840, &[GS], &[]).0;
    if s != 0 && m.call(0x0013_2b10, &[s, 0x34], &[]).0 != 0 {
        m.call(0x0019_d948, &[s + 0xf24], &[]);
    }
}

/// FUN_002242e0: r = FUN_001f4fc8(a0, a1); state 0/4 and a1 kind 0x34 -> FUN_00134310(r, 1).
pub fn ee_sub_2242e0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let mut r = m.call(0x001f_4fc8, &[a0, a1], &[]).0;
    let s = m.r32(a0 + 0x150);
    if s != 0 && s != 4 {
        return r;
    }
    if m.call(0x0013_2b10, &[a1, 0x34], &[]).0 != 0 {
        r = m.call(0x0013_4310, &[r, 1], &[]).0;
    }
    r
}

/// FUN_00195950: init fields and float constants; +0xf0 = quad at 0x2bf740.
pub fn ee_sub_195950<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.w32(a0, a1);
    m.w32(a0 + 0x1c, 0xffff_ffff);
    m.w32(a0 + 4, 0xffff_ffff);
    for k in (0..16).step_by(4) {
        let w = m.r32(0x002b_f740 + k);
        m.w32(a0 + 0xf0 + k, w);
    }
    let f5 = f32::from_bits(0x4296_0000);
    m.wf(a0 + 0x164, f32::from_bits(0x3f80_0000));
    m.wf(a0 + 0x108, f32::from_bits(0xc270_0000));
    m.wf(a0 + 0x10c, f32::from_bits(0xc248_0000));
    m.wf(a0 + 0x114, f32::from_bits(0x4248_0000));
    m.wf(a0 + 0x11c, f5);
    m.wf(a0 + 0x118, f32::from_bits(0xc316_0000));
    m.wf(a0 + 0x110, f5);
}

/// FUN_002022d8: +0xb8 -> 1; any flagged child (+0x34[i]) whose +0x100 is null/unflagged -> 1; else 0.
pub fn ee_sub_2022d8<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    if m.r32(a0 + 0xb8) != 0 {
        return 1;
    }
    let n = m.r32(a0 + 0x30) as i32;
    let mut i = 0i32;
    while i < n {
        let p = m.r32(a0 + 0x34 + 4 * i as u32);
        if bit60(m, p) {
            let q = m.r32(p + 0x100);
            if q == 0 || !bit60(m, q) {
                return 1;
            }
        }
        i += 1;
    }
    0
}

/// FUN_001e0070: remove first a1 from list +0x88[+0x84] (FUN_002332b8 memmove), count--.
pub fn ee_sub_1e0070<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let mut n = m.r32(a0 + 0x84);
    let mut i = 0u32;
    while (i as i32) < n as i32 {
        let p = a0 + 0x88 + 4 * i;
        if m.r32(p) == a1 {
            let len = (n.wrapping_sub(i) << 2).wrapping_sub(4);
            m.call(0x0023_32b8, &[p, p + 4, len], &[]);
            let c = m.r32(a0 + 0x84).wrapping_sub(1);
            m.w32(a0 + 0x84, c);
            return;
        }
        n = m.r32(a0 + 0x84);
        i += 1;
    }
}

/// FUN_001f1f38: FUN_00205d10(a0, a1); +0x494 -> vslot 20(a1 == 0).
pub fn ee_sub_1f1f38<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.call(0x0020_5d10, &[a0, a1], &[]);
    let o = m.r32(a0 + 0x494);
    if o != 0 {
        vslot(m, o, 20, &[(a1 == 0) as u32], &[]);
    }
}

/// FUN_00138370: find first (key, width) pair (width != 0) where wrap(f12 - key, f13) < width + eps; *a2 = d; returns index (n if none).
pub fn ee_sub_138370<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, f12: f32, f13: f32) -> u32 {
    let eps = f32::from_bits(0x38d1_b717);
    let mut p = a1;
    let mut i = 0u32;
    while (i as i32) < a0 as i32 {
        let w = m.rf(p + 4);
        if w != 0.0 {
            let mut d = f12 - m.rf(p);
            if d < 0.0 {
                d = d + f13;
            }
            if d < w + eps {
                if a2 != 0 {
                    m.wf(a2, d);
                }
                return i;
            }
        }
        p = p.wrapping_add(8);
        i = i.wrapping_add(1);
    }
    i
}

/// FUN_0013c4e0: state 5: detach +0x15c/+0x158, FUN_0013c560(a0, 0); if FUN_001a61a0(old) restore and FUN_0013c560(a0, 1).
pub fn ee_sub_13c4e0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    if m.r32(a0 + 0x184) != 5 {
        return;
    }
    let s1 = m.r32(a0 + 0x15c);
    m.w32(a0 + 0x15c, 0);
    let s2 = m.r32(a0 + 0x158);
    m.call(0x0013_c560, &[a0, 0], &[]);
    if s1 == 0 {
        return;
    }
    if m.call(0x001a_61a0, &[s1], &[]).0 != 0 {
        m.w32(a0 + 0x15c, s1);
        m.w32(a0 + 0x158, s2);
        m.call(0x0013_c560, &[a0, 1], &[]);
    }
}

/// FUN_00201f00: for each flagged child (+0x34[i]) with +0x100 -> vslot 7(+0x100).
pub fn ee_sub_201f00<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let mut i = 0i32;
    while i < m.r32(a0 + 0x30) as i32 {
        let p = m.r32(a0 + 0x34 + 4 * i as u32);
        if bit60(m, p) {
            let q = m.r32(p + 0x100);
            if q != 0 {
                vslot(m, q, 7, &[], &[]);
            }
        }
        i += 1;
    }
}

/// FUN_001b0d90: FUN_001b17b8(a0, 1.0); +0x16c -> FUN_001b1328(1), cache +0x174/+0x178, FUN_001e67b8 -> vslot 28(q); else FUN_001b1328(0).
pub fn ee_sub_1b0d90<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001b_17b8, &[a0], &[f4(1.0)]);
    if m.r32(a0 + 0x16c) == 0 {
        m.call(0x001b_1328, &[a0, 0], &[]);
        return;
    }
    m.call(0x001b_1328, &[a0, 1], &[]);
    let k = m.r32(a0 + 0x16c);
    let f = m.call(0x001e_6610, &[k], &[]).1[0];
    m.wf(a0 + 0x174, f);
    let k = m.r32(a0 + 0x16c);
    let f = m.call(0x001e_77e8, &[k], &[]).1[0];
    m.wf(a0 + 0x178, f);
    let sp = stack_push(m, 0x10);
    let k = m.r32(a0 + 0x16c);
    let f12 = m.rf(a0 + 0x174);
    m.call(0x001e_67b8, &[k, sp, 0], &[f4(f12)]);
    let q = m.rv(sp);
    vslot(m, a0, 28, &[], &[q]);
    stack_pop(m, 0x10);
}

/// FUN_001f0598: FUN_001f0638 == 0 -> release +0x30 (FUN_00130b18), unlink from two lists, vslot 2(3,0,0,0,0), vslot 1().
pub fn ee_sub_1f0598<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    if m.call(0x001f_0638, &[a0], &[]).0 != 0 {
        return;
    }
    let h = m.r32(a0 + 0x30);
    if h != 0 {
        m.w32(a0 + 0x30, 0);
        m.call(0x0013_0b18, &[h], &[]);
    }
    let l = m.r32(a0 + 0x24);
    m.call(0x0011_a2e8, &[l.wrapping_add(0x68), a0], &[]);
    m.call(0x0011_a2e8, &[0x003e_126c, a0], &[]);
    vslot(m, a0, 2, &[3, 0, 0, 0, 0], &[]);
    vslot(m, a0, 1, &[], &[]);
}

/// FUN_00201f90: +0xb8 = 0; FUN_0015e9c8(GS, 0x505, 0x19, 0, 16, buf); any FUN_00227c90(buf[i], +0x20) -> +0xb8 = 1.
pub fn ee_sub_201f90<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.w32(a0 + 0xb8, 0);
    let sp = stack_push(m, 0x40);
    let n = m.call(0x0015_e9c8, &[GS, 0x505, 0x19, 0, 0x10, sp], &[]).0 as i32;
    let mut i = 0i32;
    while i < n {
        let e = m.r32(sp + 4 * i as u32);
        let q = m.rv(a0 + 0x20);
        if m.call(0x0022_7c90, &[e], &[q]).0 != 0 {
            m.w32(a0 + 0xb8, 1);
            break;
        }
        i += 1;
    }
    stack_pop(m, 0x40);
}

/// FUN_00208728: remove every a1 from list +0x398[+0x394] (FUN_002332b8 memmove), index keeps advancing.
pub fn ee_sub_208728<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let mut i = 0u32;
    let mut n = m.r32(a0 + 0x394);
    while (i as i32) < n as i32 {
        let p = a0 + 0x398 + 4 * i;
        if m.r32(p) == a1 {
            let len = (n.wrapping_sub(i) << 2).wrapping_sub(4);
            m.call(0x0023_32b8, &[p, p + 4, len], &[]);
            let c = m.r32(a0 + 0x394).wrapping_sub(1);
            m.w32(a0 + 0x394, c);
        }
        n = m.r32(a0 + 0x394);
        i += 1;
    }
}

/// FUN_0021f2e0: init: +0 = a1, +4 = -1, float constants.
pub fn ee_sub_21f2e0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let one = f32::from_bits(0x3f80_0000);
    let k1000 = f32::from_bits(0x447a_0000);
    m.wf(a0 + 0x1c, f32::from_bits(0x43fa_0000));
    m.wf(a0 + 0x20, f32::from_bits(0x44bb_8000));
    m.wf(a0 + 0x24, f32::from_bits(0x3fc0_0000));
    m.w32(a0, a1);
    m.w32(a0 + 4, 0xffff_ffff);
    m.wf(a0 + 0x18, one);
    m.wf(a0 + 0x14, one);
    m.wf(a0 + 0x28, k1000);
    m.w32(a0 + 0x30, 0);
    m.wf(a0 + 0x38, f32::from_bits(0x41c8_0000));
    m.w32(a0 + 0x34, 0);
    m.wf(a0 + 0xc0, k1000);
    m.w32(a0 + 0xc8, 0);
    m.w32(a0 + 0xc4, 0);
    m.wf(a0 + 0xd0, f32::from_bits(0x4228_0000));
    m.wf(a0 + 0xd8, f32::from_bits(0xc248_0000));
    m.wf(a0 + 0xd4, f32::from_bits(0xc292_0000));
}

/// FUN_001f8dd8: FUN_00134040; for each entry +0x3f20[i] (flags&0x24 == 4) vslot 16(obj), counter on stack (+0x3f1c);
/// FUN_001f8f80(1); FUN_00134040; FUN_001f8f80(6).
pub fn ee_sub_1f8dd8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0013_4040, &[a0], &[]);
    let sp = stack_push(m, 0x10);
    let n = m.r32(a0 + 0x3f14) as i32;
    m.w32(a0 + 0x3f1c, sp);
    m.w32(sp, 0);
    if n > 0 {
        let mut i = 0u32;
        loop {
            let e = m.r32(a0 + 0x3f20).wrapping_add(i << 3);
            if m.r32(e) & 0x24 == 4 {
                let o = m.r32(e + 4);
                vslot(m, o, 16, &[], &[]);
            }
            let v = m.r32(sp).wrapping_add(1);
            let n = m.r32(a0 + 0x3f14) as i32;
            m.w32(sp, v);
            i = v;
            if !((i as i32) < n) {
                break;
            }
        }
    }
    m.w32(a0 + 0x3f1c, 0);
    stack_pop(m, 0x10);
    m.call(0x001f_8f80, &[a0, 1], &[]);
    m.call(0x0013_4040, &[a0], &[]);
    m.call(0x001f_8f80, &[a0, 6], &[]);
}

/// FUN_0022cad0(a, b): sincos(a.y) -> (s1,c1), sincos(a.z) -> (s2,c2); returns
/// (c1*c2*b.x - s2*b.y, c1*s2*b.x + c2*b.y, -s1*b.x + b.z, sext).
pub fn ee_sub_22cad0<M: QMem + ?Sized>(m: &mut M, a0: V4, a1: V4) -> V4 {
    let sp = stack_push(m, 0x10);
    m.call(0x0011_d608, &[sp, sp + 4], &[f4(a0[1])]);
    m.call(0x0011_d608, &[sp + 8, sp + 0xc], &[f4(a0[2])]);
    let f1 = m.rf(sp + 4);
    let f4_ = m.rf(sp + 0xc);
    let f3 = m.rf(sp + 8);
    let f0 = m.rf(sp);
    stack_pop(m, 0x10);
    let (f5, f6, f7) = (a1[0], a1[1], a1[2]);
    let x = f1 * f4_ * f5 - f3 * f6;
    let y = f1 * f3 * f5 + f4_ * f6;
    let z = -f0 * f5 + f7;
    let w = if z.to_bits() & 0x8000_0000 != 0 { 0xffff_ffff } else { 0 };
    [x, y, z, f32::from_bits(w)]
}

/// FUN_00206380: r = FUN_001f4fc8(a0, a1); FUN_00209a68(7) -> vslot 61(a1) == 0 -> FUN_00134310(r, 1);
/// else FUN_00209a68(0x17) and matching FUN_001eeee0 entry with state 0/4/5 -> FUN_00134310(r, 1).
pub fn ee_sub_206380<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let r = m.call(0x001f_4fc8, &[a0, a1], &[]).0;
    let bump = if m.call(0x0020_9a68, &[a0, 7], &[]).0 != 0 {
        vslot(m, a1, 61, &[], &[]).0 == 0
    } else {
        if m.call(0x0020_9a68, &[a0, 0x17], &[]).0 == 0 {
            return r;
        }
        let e = m.call(0x001e_eee0, &[a0 + 0x26c, 0xb1], &[]).0;
        let p = m.r32(e + 0x5c);
        if m.r32(a1 + 0xec) != m.r32(p + 0xec) {
            return r;
        }
        let s = m.r32(e + 0x50) as i32;
        s == 0 || (s >= 4 && s < 6)
    };
    if bump {
        m.call(0x0013_4310, &[r, 1], &[]).0
    } else {
        r
    }
}

/// FUN_001b1a38(a0, a1, out): FUN_0012ff10(a1, a0, 1, 1) -> curve; sample at 0 and +0x74 (vtable@+0 slot +0x1c);
/// *out = |v(+0x74) - v(0)| / +0x74 * +0x78.
pub fn ee_sub_1b1a38<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    if a1 == 0 {
        return;
    }
    let s0 = m.call(0x0012_ff10, &[a1, a0, 1, 1], &[]).0;
    if s0 == 0 {
        return;
    }
    let sp = stack_push(m, 0x20);
    let one = f32::from_bits(0x3f80_0000);
    let o = m.r32(s0 + 4);
    let vt = m.r32(o);
    let adj = m.r16(vt + 0x18) as i16 as i32 as u32;
    let f = m.r32(vt + 0x1c);
    m.call(f, &[o.wrapping_add(adj), sp, 0], &[f4(0.0), f4(one)]);
    let o = m.r32(s0 + 4);
    let f12 = m.rf(a1 + 0x74);
    let vt = m.r32(o);
    let adj = m.r16(vt + 0x18) as i16 as i32 as u32;
    let f = m.r32(vt + 0x1c);
    m.call(f, &[o.wrapping_add(adj), sp + 0x10, 0], &[f4(f12), f4(one)]);
    let d = (m.rf(sp + 0x10) - m.rf(sp)).abs() / m.rf(a1 + 0x74) * m.rf(a1 + 0x78);
    m.wf(a2, d);
    stack_pop(m, 0x20);
}

/// FUN_001b1f40: r = FUN_0013bbf8(a0, a1); per +0x23c[i] == a1+0x18 and per +0x250[i] kind of a1 -> r = FUN_00134310(r, 1).
pub fn ee_sub_1b1f40<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let mut r = m.call(0x0013_bbf8, &[a0, a1], &[]).0;
    let mut i = 0i32;
    while i < m.r32(a0 + 0x238) as i32 {
        if m.r32(a0 + 0x23c + 4 * i as u32) == m.r32(a1 + 0x18) {
            r = m.call(0x0013_4310, &[r, 1], &[]).0;
        }
        i += 1;
    }
    let mut i = 0i32;
    while i < m.r32(a0 + 0x24c) as i32 {
        let k = m.r32(a0 + 0x250 + 4 * i as u32);
        if m.call(0x0013_2b10, &[a1, k], &[]).0 != 0 {
            r = m.call(0x0013_4310, &[r, 1], &[]).0;
        }
        i += 1;
    }
    r
}

/// FUN_0015b0c8(a0..t3, stk): FUN_0015af08(a1, a0, t1, a3, t2, ..; +0x38, +0xc); *t3 = t - FUN_0011d448(+0x50, +0x54);
/// *stk = *t3 + +0x3c; k = FUN_0011d448(+0x48, +0x4c) != 0 -> *t0 = *t0 + vf0 + *t2 * k (VU0 vadda/vmaddx).
pub fn ee_sub_15b0c8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32, t0: u32, t1: u32, t2: u32, t3: u32, stk: u32) {
    let f12 = m.rf(a0 + 0x38);
    let f13 = m.rf(a0 + 0xc);
    m.call(0x0015_af08, &[a1, a0, t1, a3, t2, t1, t2, t3], &[f4(f12), f4(f13)]);
    let t = m.rf(T);
    let x = m.rf(a0 + 0x50);
    let y = m.rf(a0 + 0x54);
    let mut f20 = t - m.call(0x0011_d448, &[], &[f4(x), f4(y)]).1[0];
    m.wf(t3, f20);
    f20 = f20 + m.rf(a0 + 0x3c);
    m.wf(stk, f20);
    let x = m.rf(a0 + 0x48);
    let y = m.rf(a0 + 0x4c);
    let k = m.call(0x0011_d448, &[], &[f4(x), f4(y)]).1[0];
    if k == 0.0 {
        return;
    }
    let v2 = m.rv(t2);
    let v1 = m.rv(t0);
    let r = [
        v1[0] + 0.0 + v2[0] * k,
        v1[1] + 0.0 + v2[1] * k,
        v1[2] + 0.0 + v2[2] * k,
        v1[3] + 1.0 + v2[3] * k,
    ];
    m.wv(t0, r);
}

/// FUN_001b81b8: FUN_00205d10(a0, a1); a1 != 0: +0xf1c = +0x13f0[+0x180], vslot 20(0), kind 0x33 -> vslot 86(a1);
/// a1 == 0: +0xf1c -> vslot 20(1), kind 0x33 -> vslot 86(0), +0xf1c = 0.
pub fn ee_sub_1b81b8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.call(0x0020_5d10, &[a0, a1], &[]);
    if a1 == 0 {
        let o = m.r32(a0 + 0xf1c);
        if o == 0 {
            return;
        }
        vslot(m, o, 20, &[1], &[]);
        let o = m.r32(a0 + 0xf1c);
        if m.call(0x0013_2b10, &[o, 0x33], &[]).0 != 0 {
            let o = m.r32(a0 + 0xf1c);
            vslot(m, o, 86, &[0], &[]);
        }
        m.w32(a0 + 0xf1c, 0);
    } else {
        let p = m.r32(a0 + 0x13f0);
        if p == 0 {
            return;
        }
        let o = m.r32(p + 0x180);
        if o == 0 {
            return;
        }
        m.w32(a0 + 0xf1c, o);
        vslot(m, o, 20, &[0], &[]);
        let o = m.r32(a0 + 0xf1c);
        if m.call(0x0013_2b10, &[o, 0x33], &[]).0 != 0 {
            let o = m.r32(a0 + 0xf1c);
            vslot(m, o, 86, &[a1], &[]);
        }
    }
}
