//! WIRE-S2: 1:1 QMem ports of live-frontier natives (shard 3/4).
#![allow(unused)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

/// FUN_00113a30: returns 0x2bab10.
pub fn ee_sub_113a30<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x2bab10
}

/// FUN_00113a60: returns 0x2bab4c.
pub fn ee_sub_113a60<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x2bab4c
}

/// FUN_00113ae0: returns 0x2babec.
pub fn ee_sub_113ae0<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x2babec
}

/// FUN_00113b30: returns 0x2bac50.
pub fn ee_sub_113b30<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x2bac50
}

/// FUN_00113b70: returns 0x2baca0.
pub fn ee_sub_113b70<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x2baca0
}

/// FUN_00113b80: returns 0x2bacb4.
pub fn ee_sub_113b80<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x2bacb4
}

/// FUN_00113b90: returns 0x2bacc8.
pub fn ee_sub_113b90<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x2bacc8
}

/// FUN_00113c60: returns 0x2badcc.
pub fn ee_sub_113c60<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x2badcc
}

/// FUN_00113ca0: returns 0x2bae1c.
pub fn ee_sub_113ca0<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x2bae1c
}

/// FUN_00113cb0: returns 0x2bae30.
pub fn ee_sub_113cb0<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x2bae30
}

/// FUN_00113ce0: returns 0x2bae6c.
pub fn ee_sub_113ce0<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x2bae6c
}

/// FUN_00113d20: returns 0x2baebc.
pub fn ee_sub_113d20<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x2baebc
}

/// FUN_00113f00: returns 0x2bb114.
pub fn ee_sub_113f00<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x2bb114
}

/// FUN_00113f10: returns 0x2bb128.
pub fn ee_sub_113f10<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x2bb128
}

/// FUN_00113f30: returns 0x2bb150.
pub fn ee_sub_113f30<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x2bb150
}

/// FUN_00113f40: returns 0x2bb164.
pub fn ee_sub_113f40<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x2bb164
}

/// FUN_00113f70: returns 0x2bb1a0.
pub fn ee_sub_113f70<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x2bb1a0
}

/// FUN_00114050: returns 0x2bb2b8.
pub fn ee_sub_114050<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x2bb2b8
}

/// FUN_00114070: returns 0x2bb2e0.
pub fn ee_sub_114070<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x2bb2e0
}

/// FUN_00114110: returns 0x2bb3a8.
pub fn ee_sub_114110<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x2bb3a8
}

/// FUN_001141d0: returns 0x2bb498.
pub fn ee_sub_1141d0<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x2bb498
}

/// FUN_00114210: returns 0x2bb4e8.
pub fn ee_sub_114210<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x2bb4e8
}

/// FUN_00114470: returns 0x2bb7e0.
pub fn ee_sub_114470<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x2bb7e0
}

/// FUN_0012e740: ++*(a0 + 0x12c).
pub fn ee_sub_12e740<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let v = m.r32(a0 + 0x12c).wrapping_add(1);
    m.w32(a0 + 0x12c, v);
    v
}

/// FUN_00172550: *(a0 + 0x30) == -1.
pub fn ee_sub_172550<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    (m.r32(a0 + 0x30) == 0xffff_ffff) as u32
}

/// FUN_001cc158: +0x90 = +0xa4 = -1.
pub fn ee_sub_1cc158<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.w32(a0 + 0x90, 0xffff_ffff);
    m.w32(a0 + 0xa4, 0xffff_ffff);
}

/// FUN_00130488: FUN_0012f9b8(a0, a1, a2, &out); return out.
pub fn ee_sub_130488<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) -> u32 {
    let sp = stack_push(m, 0x10);
    m.call(0x0012_f9b8, &[a0, a1, a2, sp], &[]);
    let r = m.r32(sp);
    stack_pop(m, 0x10);
    r
}

/// FUN_001f03a8: FUN_001f01b8(a0, a1, a2, a3, t0, &out); return out.
pub fn ee_sub_1f03a8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, t0: u32) -> u32 {
    let sp = stack_push(m, 0x10);
    m.call(0x001f_01b8, &[a0, a1, a2, a3, t0, sp], &[]);
    let r = m.r32(sp);
    stack_pop(m, 0x10);
    r
}

/// FUN_00144068: +0x270 > 0 || +0x1084 != 0.
pub fn ee_sub_144068<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    if m.r32(a0 + 0x270) as i32 > 0 {
        return 1;
    }
    (m.r32(a0 + 0x1084) != 0) as u32
}

/// FUN_00208700: if +0x394 < 3: +0x398[n++] = a1.
pub fn ee_sub_208700<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let n = m.r32(a0 + 0x394);
    if n < 3 {
        m.w32(a0 + 0x394, n.wrapping_add(1));
        m.w32((n << 2).wrapping_add(a0) + 0x398, a1);
    }
}

/// FUN_0020b840: +0 = a1; +0x94 = 0.6; +0x90 = a2; +0x98 = 1.0.
pub fn ee_sub_20b840<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    m.w32(a0, a1);
    m.wf(a0 + 0x94, f32::from_bits(0x3f19_999a));
    m.w32(a0 + 0x90, a2);
    m.wf(a0 + 0x98, f32::from_bits(0x3f80_0000));
}

/// FUN_00144038: (vslot 88(a0) - 5) < 2.
pub fn ee_sub_144038<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let v = vslot(m, a0, 88, &[], &[]).0;
    (v.wrapping_sub(5) < 2) as u32
}

/// FUN_001733c8: if +0x98 < 16: (+0x9c)[n++] = q (quadword).
pub fn ee_sub_1733c8<M: QMem + ?Sized>(m: &mut M, a0: u32, q: V4) {
    let n = m.r32(a0 + 0x98);
    if (n as i32) < 0x10 {
        let b = m.r32(a0 + 0x9c);
        m.w32(a0 + 0x98, n.wrapping_add(1));
        m.wv((n << 4).wrapping_add(b), q);
    }
}

/// FUN_00151208: FUN_0014fca8(a0); +0xc = vtable 0x2cd450; return a0.
pub fn ee_sub_151208<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    m.call(0x0014_fca8, &[a0], &[]);
    m.w32(a0 + 0xc, 0x002c_d450);
    a0
}

/// FUN_002022a0: for i < +0x30: (+0x34)[i]->+0x108 = 0.
pub fn ee_sub_2022a0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    if m.r32(a0 + 0x30) as i32 <= 0 {
        return;
    }
    let mut i: i32 = 0;
    let mut p = a0 + 0x34;
    loop {
        let e = m.r32(p);
        i += 1;
        m.w32(e + 0x108, 0);
        p = p.wrapping_add(4);
        if i >= m.r32(a0 + 0x30) as i32 {
            break;
        }
    }
}

/// FUN_00203758: +0xec ? FUN_00164030(+0xec) : +0xf0.
pub fn ee_sub_203758<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let p = m.r32(a0 + 0xec);
    if p == 0 {
        return m.r32(a0 + 0xf0);
    }
    m.call(0x0016_4030, &[p], &[]).0
}

/// FUN_001e00e8: a1 in (+0x88)[0..+0x84].
pub fn ee_sub_1e00e8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let n = m.r32(a0 + 0x84) as i32;
    let mut i: i32 = 0;
    let mut p = a0 + 0x88;
    if n <= 0 {
        return 0;
    }
    loop {
        let v = m.r32(p);
        i += 1;
        if v == a1 {
            return 1;
        }
        p = p.wrapping_add(4);
        if i >= n {
            return 0;
        }
    }
}

/// FUN_001f3590: (+0xe8): +0x80 = 0, +0x88 = -1750.0, +0x84 = 0, +0xc0 = 3, +0xa0 = 3.
pub fn ee_sub_1f3590<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let v = m.r32(a0 + 0xe8);
    m.w32(v + 0x80, 0);
    m.wf(v + 0x88, f32::from_bits(0xc4da_c000));
    m.w32(v + 0x84, 0);
    let v = m.r32(a0 + 0xe8);
    m.w32(v + 0xc0, 3);
    let v = m.r32(a0 + 0xe8);
    m.w32(v + 0xa0, 3);
}

/// FUN_001fb2c0: if f12 != +0x18: +0x18 = f12; FUN_001fb658(a0, 0x40, 0x40).
pub fn ee_sub_1fb2c0<M: QMem + ?Sized>(m: &mut M, a0: u32, f12: f32) {
    if f12 == m.rf(a0 + 0x18) {
        return;
    }
    m.wf(a0 + 0x18, f12);
    m.call(0x001f_b658, &[a0, 0x40, 0x40], &[[f12, 0.0, 0.0, 0.0]]);
}

/// FUN_001fe990: index of a1 in (+0x1d4)[0..+0x1d0], else -1.
pub fn ee_sub_1fe990<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let n = m.r32(a0 + 0x1d0) as i32;
    let mut i: i32 = 0;
    let mut p = a0 + 0x1d4;
    if n <= 0 {
        return 0xffff_ffff;
    }
    loop {
        if m.r32(p) == a1 {
            return i as u32;
        }
        i += 1;
        p = p.wrapping_add(4);
        if i >= n {
            return 0xffff_ffff;
        }
    }
}

/// FUN_0022b7a0: FUN_0011a070(a0 + 0x8f0, a1); a1->+0xc = a0.
pub fn ee_sub_22b7a0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.call(0x0011_a070, &[a0 + 0x8f0, a1], &[]);
    m.w32(a1 + 0xc, a0);
}

/// FUN_001a6290: a1 == +0x18, or a1 found walking list +0x3c (node +4 next, +0x10 owner == a0, +8 value).
pub fn ee_sub_1a6290<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    if m.r32(a0 + 0x18) == a1 {
        return 1;
    }
    let mut v1 = m.r32(a0 + 0x3c);
    loop {
        if v1 == 0 {
            return 0;
        }
        if m.r32(v1 + 0x10) != a0 {
            return 0;
        }
        if m.r32(v1 + 8) == a1 {
            return 1;
        }
        v1 = m.r32(v1 + 4);
    }
}

/// FUN_00143380: if +0x11a8: FUN_00143678(a0, 2, 0); FUN_0013c560(+0x11a8, 5).
pub fn ee_sub_143380<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    if m.r32(a0 + 0x11a8) == 0 {
        return;
    }
    m.call(0x0014_3678, &[a0, 2, 0], &[]);
    let p = m.r32(a0 + 0x11a8);
    m.call(0x0013_c560, &[p, 5], &[]);
}

/// FUN_0014d1f8: +8 = product of +0x18[0..9]; +0xc = +0x38.
pub fn ee_sub_14d1f8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.wf(a0 + 8, 1.0);
    let mut p = a0 + 0x18;
    for _ in 0..9 {
        let f0 = m.rf(p);
        let f1 = m.rf(a0 + 8) * f0;
        p += 4;
        m.wf(a0 + 8, f1);
    }
    let f = m.rf(a0 + 0x38);
    m.wf(a0 + 0xc, f);
}

/// FUN_0019de10: init: +0 = a1, +4 = -1, +0x30 = (45, 0, 40), +0x40 = (45, 0, -20); returns a0 + 0x30.
pub fn ee_sub_19de10<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let f45 = f32::from_bits(0x4234_0000);
    m.w32(a0, a1);
    m.w32(a0 + 4, 0xffff_ffff);
    m.wf(a0 + 0x30, f45);
    m.wf(a0 + 0x38, f32::from_bits(0x4220_0000));
    m.w32(a0 + 0x34, 0);
    m.wf(a0 + 0x40, f45);
    m.wf(a0 + 0x48, f32::from_bits(0xc1a0_0000));
    m.w32(a0 + 0x44, 0);
    a0 + 0x30
}

/// FUN_00140c30(a0, a1, a2, q): bit 60 of a2+0x28 ? FUN_00205008(a0, a1, a2, q - a0[+0x1010])
/// : FUN_00195a00(a0 + 0x2c0, 0, 0, 0).
pub fn ee_sub_140c30<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, q: V4) -> u32 {
    if (m.r64(a2 + 0x28) >> 60) & 1 == 0 {
        return m.call(0x0019_5a00, &[a0 + 0x2c0, 0, 0, 0], &[]).0;
    }
    let b = m.rv(a0 + 0x1010);
    let d = [q[0] - b[0], q[1] - b[1], q[2] - b[2], q[3] - b[3]];
    m.call(0x0020_5008, &[a0, a1, a2], &[d]).0
}

/// FUN_00147ee8: FUN_0011a2e8(a0 + 0x20, a1) (BOSS R21: a1 is passed through unchanged, 0x147efc keeps it in s1
/// and the jal at 0x147f08 leaves $a1 as is; the decompile drops it); +0x39a4 == 0 -> +0x39a4 = a1; FUN_0011a220(a0 + 0x20, a1).
pub fn ee_sub_147ee8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let s2 = a0 + 0x20;
    m.call(0x0011_a2e8, &[s2, a1], &[]);
    if m.r32(a0 + 0x39a4) == 0 {
        m.w32(a0 + 0x39a4, a1);
    }
    m.call(0x0011_a220, &[s2, a1], &[]);
}

fn ctor_2550<M: QMem + ?Sized>(m: &mut M, a0: u32, f: u32, a1: u32) -> u32 {
    m.w32(a0, 0);
    m.call(f, &[a0, a1], &[]);
    let w = m.r32(a0 + 4);
    let g = m.r32(0x002d_d598);
    m.w32(a0 + 4, (w & 0x7fff_ffff) | (g << 31));
    a0
}

/// FUN_002550a0: +0 = 0; FUN_00254f00(a0, 0x2dd280); +4 bit31 = DAT_002dd598; return a0.
pub fn ee_sub_2550a0<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    ctor_2550(m, a0, 0x0025_4f00, 0x002d_d280)
}

/// FUN_002553e0: +0 = 0; FUN_00255248(a0, 0x2dd290); +4 bit31 = DAT_002dd598; return a0.
pub fn ee_sub_2553e0<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    ctor_2550(m, a0, 0x0025_5248, 0x002d_d290)
}

/// FUN_00255718: +0 = 0; FUN_00255588(a0, 0x2dd2a0); +4 bit31 = DAT_002dd598; return a0.
pub fn ee_sub_255718<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    ctor_2550(m, a0, 0x0025_5588, 0x002d_d2a0)
}

/// FUN_00255d90: +0 = 0; FUN_00255c00(a0, 0x2dd2c0); +4 bit31 = DAT_002dd598; return a0.
pub fn ee_sub_255d90<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    ctor_2550(m, a0, 0x0025_5c00, 0x002d_d2c0)
}

/// FUN_002560c8: +0 = 0; FUN_00255f38(a0, 0x2dd2d0); +4 bit31 = DAT_002dd598; return a0.
pub fn ee_sub_2560c8<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    ctor_2550(m, a0, 0x0025_5f38, 0x002d_d2d0)
}

/// FUN_001a3e88: k = a1->+0x118; set bit k in a0+0x4590 bitset; a0[+0x4154 + 4k] = 0; a1->+0x118 = -1; +0x4554 = 1.
pub fn ee_sub_1a3e88<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let k = m.r32(a1 + 0x118) as i32;
    let q = k / 32;
    let a = (a0 + 0x4590).wrapping_add((q << 2) as u32);
    let r = k.wrapping_sub(q << 5) as u32;
    let w = m.r32(a) | (1u32 << (r & 31));
    m.w32(a, w);
    let k2 = m.r32(a1 + 0x118);
    m.w32(a0.wrapping_add(k2 << 2).wrapping_add(0x4154), 0);
    m.w32(a1 + 0x118, 0xffff_ffff);
    m.w32(a0 + 0x4554, 1);
}

/// FUN_00228250: FUN_00117968(a0, 0x40, buf); buf[1] == buf[15] ? *a1 = 0 : *a1 = FUN_001ae238(&{0x40, 0x10}, buf).
pub fn ee_sub_228250<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let sp = stack_push(m, 0x50);
    m.call(0x0011_7968, &[a0, 0x40, sp], &[]);
    if m.r32(sp + 4) == m.r32(sp + 0x3c) {
        m.w32(a1, 0);
    } else {
        let p = sp + 0x40;
        m.w32(p, 0x40);
        m.w32(p + 4, 0x10);
        let v = m.call(0x001a_e238, &[p, sp], &[]).0;
        m.w32(a1, v);
    }
    stack_pop(m, 0x50);
}

/// FUN_0013bbf8: r = FUN_001f4fc8(a0); if **(a1+0x14) == **(a0->+0x200 + 0x14): r = FUN_00134310(r, 1); return r.
pub fn ee_sub_13bbf8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let mut r = m.call(0x001f_4fc8, &[a0], &[]).0;
    let a2 = m.r32(a0 + 0x200);
    let v1 = m.r32(a1 + 0x14);
    let p = m.r32(a2 + 0x14);
    let x = m.r32(v1);
    if x == m.r32(p) {
        r = m.call(0x0013_4310, &[r, 1], &[]).0;
    }
    r
}
