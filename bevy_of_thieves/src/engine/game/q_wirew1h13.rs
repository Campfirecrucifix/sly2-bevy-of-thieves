//! WIRE-W1H13: 1:1 queue ports (against `QMem`) of a batch of live-frontier natives, ported from the EE asm;
//! callees go through `m.call` / `vslot`. QMem convention: integer / pointer args in `a`, quadword and float
//! args in `v`.
#![allow(unused_variables, unused_mut, unused_assignments, clippy::all)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

/// FUN_0018df50: *(a0+0x1d30) = a1
pub fn ee_sub_18df50<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.w32(a0.wrapping_add(0x1d30), a1);
}

/// FUN_002564c0: identity
pub fn ee_sub_2564c0<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    a0
}

/// FUN_001c9958: (short)a0
pub fn ee_sub_1c9958<M: QMem + ?Sized>(m: &mut M, a0: u32) -> i32 {
    a0 as u16 as i16 as i32
}

/// FUN_001cc158: +0x90 = +0xa4 = -1
pub fn ee_sub_1cc158<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.w32(a0 + 0x90, 0xffff_ffff);
    m.w32(a0 + 0xa4, 0xffff_ffff);
}

/// FUN_00229d78: tail of FUN_001f72f8(a0, a1)
pub fn ee_sub_229d78<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    m.call(0x001f_72f8, &[a0, a1], &[]).0
}

/// FUN_00232500: FUN_00232520(a0..t1, 0, 0)
pub fn ee_sub_232500<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, t0: u32, t1: u32) {
    m.call(0x0023_2520, &[a0, a1, a2, a3, t0, t1, 0, 0], &[]);
}

/// FUN_001a0b18: init (+0 = a1, +0xc = 0x28, +0x90 = +4 = -1, +0x9c = 2.0)
pub fn ee_sub_1a0b18<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.w32(a0, a1);
    m.w32(a0 + 0xc, 0x28);
    m.w32(a0 + 0x90, 0xffff_ffff);
    m.wf(a0 + 0x9c, 2.0);
    m.w32(a0 + 4, 0xffff_ffff);
}

/// FUN_002322f0: FUN_00232838(0x5c, 0x30, a0, 0, 0)
pub fn ee_sub_2322f0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0023_2838, &[0x5c, 0x30, a0, 0, 0], &[]);
}

/// FUN_0023e260: FUN_0023dff8(&{a0, a1, a2, a3})
pub fn ee_sub_23e260<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x10);
    m.w32(sp, a0);
    m.w32(sp + 4, a1);
    m.w32(sp + 8, a2);
    m.w32(sp + 0xc, a3);
    let r = m.call(0x0023_dff8, &[sp], &[]).0;
    stack_pop(m, 0x10);
    r
}

/// FUN_001de5a8: FUN_001f56f8(a0)->+0x70 |= 0x20 (64-bit)
pub fn ee_sub_1de5a8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let p = m.call(0x001f_56f8, &[a0], &[]).0;
    let x = m.r32(p + 0x70);
    m.w32(p + 0x70, x | 0x20);
}

/// FUN_0011d3c0: FUN_002342a0() % 0x35f15e13
pub fn ee_sub_11d3c0<M: QMem + ?Sized>(m: &mut M) -> i32 {
    let v = m.call(0x0023_42a0, &[], &[]).0 as i32;
    v.wrapping_rem(0x35f1_5e13)
}

/// FUN_001a2388: FUN_001f80d0(a0, a1, 1.0); FUN_00140eb0(a0)
pub fn ee_sub_1a2388<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.call(0x001f_80d0, &[a0, a1], &[f4(1.0)]);
    m.call(0x0014_0eb0, &[a0], &[]);
}

/// FUN_001524a0: FUN_001305b8(a1, a0, a2 | 1, a0 + 0x408, 0.0, 1.0)
pub fn ee_sub_1524a0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    m.call(0x0013_05b8, &[a1, a0, a2 | 1, a0.wrapping_add(0x408)], &[f4(0.0), f4(1.0)]);
}

/// FUN_001e00e8: a1 in (a0+0x88)[0 .. *(a0+0x84)]
pub fn ee_sub_1e00e8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let n = m.r32(a0 + 0x84) as i32;
    if n <= 0 {
        return 0;
    }
    let mut p = a0 + 0x88;
    let mut i = 0i32;
    loop {
        let v = m.r32(p);
        i += 1;
        if v == a1 {
            return 1;
        }
        if !(i < n) {
            return 0;
        }
        p = p.wrapping_add(4);
    }
}

/// FUN_001a6290
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

/// FUN_001af660
pub fn ee_sub_1af660<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    if m.r32(a0 + 0x1e8) != 7 {
        return;
    }
    if !((m.r32(a0 + 0x24) as i32) < 2) {
        m.w32(a0 + 0x24, 1);
    }
    m.call(0x001a_eaa8, &[a0, 8], &[]);
}

/// FUN_001a3b38
pub fn ee_sub_1a3b38<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    if m.r32(a0 + 0x160) == 0 {
        let g = 0x003d_d490u32;
        let x = m.r32(g + 0x3d5c).wrapping_add(1);
        m.w32(g + 0x3d5c, x);
    }
    if m.r32(a0 + 0x15c) == 0 && m.r32(a0 + 0xe0) != 0 {
        m.w32(0x003e_19e4, 1);
    }
}

/// FUN_0023dd30(double a, double b): FUN_0023d468(&a, t0); FUN_0023d468(&b, t1); FUN_0023dc18(t0, t1)
pub fn ee_sub_23dd30<M: QMem + ?Sized>(m: &mut M, a0: u64, a1: u64) -> u32 {
    let sp = stack_push(m, 0x50);
    m.w32(sp + 0x40, a0 as u32);
    m.w32(sp + 0x44, (a0 >> 32) as u32);
    m.w32(sp + 0x48, a1 as u32);
    m.w32(sp + 0x4c, (a1 >> 32) as u32);
    m.call(0x0023_d468, &[sp + 0x40, sp], &[]);
    m.call(0x0023_d468, &[sp + 0x48, sp + 0x20], &[]);
    let r = m.call(0x0023_dc18, &[sp, sp + 0x20], &[]).0;
    stack_pop(m, 0x50);
    r
}

/// FUN_00231598: FUN_0011c168(tmp, a1, a0+0x60); a0[0x60..0xa0] = tmp; FUN_002313f0(a0)
pub fn ee_sub_231598<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let sp = stack_push(m, 0x40);
    m.call(0x0011_c168, &[sp, a1, a0 + 0x60], &[]);
    for k in (0..0x40).step_by(4) {
        let w = m.r32(sp + k);
        m.w32(a0 + 0x60 + k, w);
    }
    stack_pop(m, 0x40);
    m.call(0x0023_13f0, &[a0], &[]);
}

/// FUN_001fb268: angle = a1 mod 360 (wrapped to >= 0); on change store +0x14 and FUN_001fb658(a0, 0x20, 0x20)
pub fn ee_sub_1fb268<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: i32) {
    let r = a1.wrapping_rem(0x168);
    let v = if -1 < r { r } else { r + 0x168 };
    if v as u32 == m.r32(a0 + 0x14) {
        return;
    }
    m.w32(a0 + 0x14, v as u32);
    m.call(0x001f_b658, &[a0, 0x20, 0x20], &[]);
}

/// FUN_001a3e88: release slot *(a1+0x118) in the bitmap at a0+0x4590
pub fn ee_sub_1a3e88<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let a2 = m.r32(a1 + 0x118) as i32;
    let q = (if -1 < a2 { a2 } else { a2.wrapping_add(0x1f) }) >> 5;
    let p = a0.wrapping_add(0x4590).wrapping_add((q << 2) as u32);
    let bit = 1u32 << ((a2.wrapping_sub(q << 5)) as u32 & 31);
    let w = m.r32(p);
    m.w32(p, w | bit);
    let s = m.r32(a1 + 0x118);
    m.w32(a0.wrapping_add(s << 2).wrapping_add(0x4154), 0);
    m.w32(a1 + 0x118, 0xffff_ffff);
    m.w32(a0 + 0x4554, 1);
}

/// FUN_00228250: FUN_00117968(a0, 0x40, buf); *a1 = buf[1] != buf[15] ? FUN_001ae238(&{0x40, 0x10}, buf) : 0
pub fn ee_sub_228250<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let sp = stack_push(m, 0x50);
    m.call(0x0011_7968, &[a0, 0x40, sp], &[]);
    if m.r32(sp + 4) != m.r32(sp + 0x3c) {
        m.w32(sp + 0x40, 0x40);
        m.w32(sp + 0x44, 0x10);
        let v = m.call(0x001a_e238, &[sp + 0x40, sp], &[]).0;
        m.w32(a1, v);
    } else {
        m.w32(a1, 0);
    }
    stack_pop(m, 0x50);
}

/// FUN_00191090: init
pub fn ee_sub_191090<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.w32(a0, a1);
    m.w32(a0 + 0x4fc, 0xffff_ffff);
    m.w32(a0 + 8, 0xffff_ffff);
    m.w32(a0 + 0xc, 0xffff_ffff);
    m.call(0x0020_b840, &[a0 + 0x180, a1, 0x6f], &[]);
    m.call(0x0020_b840, &[a0 + 0x240, a1, 0x6f], &[]);
    m.call(0x001c_c158, &[a0 + 0x440], &[]);
}

/// FUN_001c9968
pub fn ee_sub_1c9968<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let v = m.call(0x001c_9958, &[a0], &[]).0;
    if v == 0xffff_ffff {
        return 0xffff_ffff;
    }
    let i = m.call(0x001c_9950, &[a0], &[]).0;
    let s0 = m.r32(0x003e_1398).wrapping_add(i.wrapping_mul(0x6c));
    let v = m.call(0x001c_9958, &[a0], &[]).0;
    let t = m.r32(s0 + 0x18);
    m.r32((v << 2).wrapping_add(t))
}

/// FUN_00195950: init
pub fn ee_sub_195950<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.w32(a0, a1);
    m.w32(a0 + 0x1c, 0xffff_ffff);
    m.w32(a0 + 4, 0xffff_ffff);
    for k in (0..0x10).step_by(4) {
        let w = m.r32(0x002b_f740 + k);
        m.w32(a0 + 0xf0 + k, w);
    }
    m.wf(a0 + 0x164, 1.0);
    m.w32(a0 + 0x108, 0xc270_0000);
    m.w32(a0 + 0x10c, 0xc248_0000);
    m.w32(a0 + 0x114, 0x4248_0000);
    m.w32(a0 + 0x11c, 0x4296_0000);
    m.w32(a0 + 0x118, 0xc316_0000);
    m.w32(a0 + 0x110, 0x4296_0000);
}

fn bit60<M: QMem + ?Sized>(m: &mut M, o: u32) -> bool {
    (m.r32(o + 0x2c) >> 28) & 1 != 0
}

/// FUN_002022d8
pub fn ee_sub_2022d8<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    if m.r32(a0 + 0xb8) != 0 {
        return 1;
    }
    let n = m.r32(a0 + 0x30) as i32;
    if n <= 0 {
        return 0;
    }
    let mut i = 0i32;
    let mut p = a0 + 0x34;
    loop {
        let o = m.r32(p);
        if bit60(m, o) {
            let q = m.r32(o + 0x100);
            if q == 0 {
                return 1;
            }
            if !bit60(m, q) {
                return 1;
            }
        }
        i += 1;
        p = p.wrapping_add(4);
        if !(i < n) {
            return 0;
        }
    }
}

/// FUN_002590f0
pub fn ee_sub_2590f0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    if a1 == 1 {
        m.w32(a0 + 0xe0, a1);
        let p = m.r32(a0 + 0xe4);
        let c = m.r32(p) as i32;
        if !(c < 2) {
            m.w32(p, c.wrapping_sub(1) as u32);
            let sp = stack_push(m, 0x10);
            m.w32(sp, 0x100);
            let e = m.r32(a0 + 0xe4);
            m.w32(sp + 4, 0x10);
            let v = m.call(0x001a_e238, &[sp, e], &[]).0;
            stack_pop(m, 0x10);
            m.w32(a0 + 0xe4, v);
            m.w32(v, a1);
        }
    }
    m.r32(a0 + 0xe4)
}

/// FUN_00201f00: for each child with bit 60 set and a +0x100 partner, partner vslot 7
pub fn ee_sub_201f00<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    if (m.r32(a0 + 0x30) as i32) <= 0 {
        return;
    }
    let mut i = 0i32;
    let mut p = a0 + 0x34;
    loop {
        let o = m.r32(p);
        if bit60(m, o) {
            let q = m.r32(o + 0x100);
            if q != 0 {
                vslot(m, q, 7, &[], &[]);
            }
        }
        i += 1;
        p = p.wrapping_add(4);
        if !(i < m.r32(a0 + 0x30) as i32) {
            return;
        }
    }
}

/// FUN_001b1868
pub fn ee_sub_1b1868<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let s1 = vslot(m, a0, 30, &[], &[]).0;
    let v = vslot(m, a0, 30, &[], &[]).0;
    let f13 = m.rf(v);
    let f12 = m.rf(s1 + 4);
    let f0 = m.call(0x0024_58f0, &[], &[f4(f12), f4(f13)]).1[0];
    let f12 = m.rf(a0 + 0x1d8) - f0;
    let f0 = m.call(0x0011_cf60, &[], &[f4(f12)]).1[0];
    (f0.abs() < f32::from_bits(0x3c8e_fa36)) as u32
}

/// FUN_0023e1c8: float -> int via FUN_0023e108 unpack
pub fn ee_sub_23e1c8<M: QMem + ?Sized>(m: &mut M, f12: f32) -> u32 {
    let sp = stack_push(m, 0x20);
    m.wf(sp + 0x10, f12);
    m.call(0x0023_e108, &[sp + 0x10, sp], &[]);
    let cls = m.r32(sp);
    let sign = m.r32(sp + 4);
    let e = m.r32(sp + 8) as i32;
    let mant = m.r32(sp + 0xc);
    stack_pop(m, 0x20);
    if cls == 2 || cls < 2 {
        return 0;
    }
    if sign != 0 {
        return 0;
    }
    if cls != 4 {
        if e < 0 {
            return 0;
        }
        if e < 0x20 {
            if e < 0x1f {
                return mant >> ((0x1e - e) as u32 & 31);
            }
            return mant << ((e - 0x1e) as u32 & 31);
        }
    }
    0xffff_ffff
}

/// FUN_00181268
pub fn ee_sub_181268<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: i32) {
    let cur = m.r32(a0 + 0x1c0);
    if cur == a1 as u32 {
        return;
    }
    if cur == 0xffff_ffff {
        let x = m.r32(a0 + 0x180);
        m.w32(a0 + 0x184, x);
    }
    m.w32(a0 + 0x1c0, a1 as u32);
    let x = m.r32(0x002d_ded0);
    m.w32(a0 + 0x1c4, x);
    if a1 != 3 {
        let clear = if a1 < 4 { a1 >= 0 } else { a1 == 4 };
        if clear {
            m.w32(a0 + 0x214, 0);
            m.w32(a0 + 0x218, 0);
            m.w32(a0 + 0x210, 0);
        }
    }
    let p = if a1 == -1 { 0 } else { a0.wrapping_add((a1 as u32) << 4).wrapping_add(0x220) };
    m.call(0x0017_f4c0, &[a0, p], &[]);
}

/// FUN_0021f2e0: init
pub fn ee_sub_21f2e0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.wf(a0 + 0x1c, 500.0);
    m.w32(a0 + 0x20, 0x44bb_8000);
    m.wf(a0 + 0x24, 1.5);
    m.w32(a0, a1);
    m.w32(a0 + 4, 0xffff_ffff);
    m.wf(a0 + 0x18, 1.0);
    m.wf(a0 + 0x14, 1.0);
    m.wf(a0 + 0x28, 1000.0);
    m.w32(a0 + 0x30, 0);
    m.wf(a0 + 0x38, 25.0);
    m.w32(a0 + 0x34, 0);
    m.wf(a0 + 0xc0, 1000.0);
    m.w32(a0 + 0xc8, 0);
    m.w32(a0 + 0xc4, 0);
    m.w32(a0 + 0xd0, 0x4228_0000);
    m.w32(a0 + 0xd8, 0xc248_0000);
    m.w32(a0 + 0xd4, 0xc292_0000);
}

/// FUN_0018f688
pub fn ee_sub_18f688<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let p = m.r32(a0 + 0x150);
    let mut f20 = if p != 0 && 0.5 < m.rf(p + 0x1c) { m.rf(p + 0x18) } else { m.rf(a0 + 0x1a8) };
    let sp = stack_push(m, 0x20);
    m.call(0x0018_f2b0, &[a0, 8, 0x002b_f4d0, sp + 0x10, sp], &[f4(f20), f4(0.25), f4(1.0)]);
    let v = m.r32(sp + 0x10);
    m.w32(a0 + 0x18c0, v);
    if v != 0 {
        let f12 = m.rf(sp + 4);
        let f13 = m.rf(sp);
        f20 = m.call(0x0024_58f0, &[], &[f4(f12), f4(f13)]).1[0];
    }
    stack_pop(m, 0x20);
    m.call(0x0020_4660, &[a0], &[f4(f20)]);
    m.call(0x0020_4ca8, &[a0], &[]);
}

/// FUN_001b82a0: switch on +0x13d0 (jump table 0x2a7480)
pub fn ee_sub_1b82a0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    match m.r32(a0 + 0x13d0) {
        0 | 1 | 4 | 5 | 7 | 8 | 9 | 10 | 0x12 | 0x15 => {
            let a1 = m.r32(a0 + 0x208);
            if a1 == 0 || !(f32::from_bits(0x3f33_3333) < m.rf(a1 + 0x88)) {
                return;
            }
            let o = m.r32(a1 + 0x4c);
            if m.r32(o + 0x148) & 0x2000_0000 != 0 {
                return;
            }
            if m.r32(a0 + 0x108c) != 0 {
                return;
            }
            let q = m.rv(a1 + 0x60);
            let x = m.r32(a1 + 0x40);
            m.call(0x0019_feb0, &[a0 + 0x9e0, x], &[q]);
        }
        0xd => {
            m.call(0x0019_42f0, &[a0 + 0x4c0], &[]);
        }
        0x14 => {
            m.call(0x0019_ea60, &[a0 + 0xf50], &[]);
        }
        _ => {}
    }
}

/// FUN_001ddb08
pub fn ee_sub_1ddb08<M: QMem + ?Sized>(m: &mut M) {
    let s1 = m.r32(0x003e_0fac);
    let sp = stack_push(m, 0x30);
    m.w32(sp, 0x16);
    m.w32(sp + 4, 0);
    m.call(0x0012_91a8, &[sp + 0x10, sp], &[]);
    m.call(0x0025_8200, &[sp + 0x20, s1, sp + 0x10, 0x003d_4a60], &[]);
    m.call(0x0012_9ec8, &[sp + 0x20], &[]);
    m.call(0x0012_9ec8, &[sp + 0x10], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x30);
    for g in [0x002e_1e40u32, 0x002d_d5bc, 0x002f_7900] {
        let a = m.r32(g);
        if a != 0 {
            m.call(0x0014_40d0, &[a, 1], &[]);
        }
    }
    m.call(0x0016_5e38, &[0x003d_4a60], &[]);
}

/// FUN_001c4768(o, v): t = rows(o+0x40..0x60) . v; u = (-t.y, t.z, 0, r0.w); normalised (or *0x2a2db0 if |u| < 1e-4)
pub fn ee_sub_1c4768<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: V4) -> V4 {
    let r0 = m.rv(a0 + 0x40);
    let r1 = m.rv(a0 + 0x50);
    let r2 = m.rv(a0 + 0x60);
    let d = |r: V4| r[0] * a1[0] + r[1] * a1[1] + r[2] * a1[2];
    let t = [d(r0), d(r1), d(r2), r0[3]];
    let u = [-t[1], t[2], 0.0, t[3]];
    let len = (u[0] * u[0] + u[1] * u[1] + u[2] * u[2]).sqrt();
    if len < f32::from_bits(0x38d1_b717) {
        return m.rv(0x002a_2db0);
    }
    let q = 1.0 / len;
    [u[0] * q, u[1] * q, u[2] * q, u[3] * q]
}

/// FUN_001c4af8(o, pos): nearest (squared xyz distance via FUN_001a62e8) bit-60 entry of the list at o+0x3ebc
pub fn ee_sub_1c4af8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: V4) -> u32 {
    let mut best = m.rf(0x002a_7708);
    let mut s2 = 0u32;
    if (m.r32(a0 + 0x3eb8) as i32) <= 0 {
        return s2;
    }
    let mut i = 0i32;
    let mut p = a0 + 0x3ebc;
    loop {
        let o = m.r32(p);
        if bit60(m, o) {
            let q = m.call(0x001a_62e8, &[o], &[]).1;
            let d = [a1[0] - q[0], a1[1] - q[1], a1[2] - q[2]];
            let f = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
            if f < best {
                best = f;
                s2 = o;
            }
        }
        i += 1;
        p = p.wrapping_add(4);
        if !(i < m.r32(a0 + 0x3eb8) as i32) {
            return s2;
        }
    }
}

/// FUN_00189530: FUN_00205d10(a0); the three +0x19a0.. objects get vslot 20 (a1 == 0)
pub fn ee_sub_189530<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.call(0x0020_5d10, &[a0], &[]);
    let x = (a1 == 0) as u32;
    for off in [0x19a0u32, 0x19a4, 0x19a8] {
        let o = m.r32(a0 + off);
        vslot(m, o, 20, &[x], &[]);
    }
}

/// FUN_00117030: Hermite-style segment eval (value -> *a0, derivative -> *a1)
pub fn ee_sub_117030<M: QMem + ?Sized>(
    m: &mut M,
    a0: u32,
    a1: u32,
    f12: f32,
    f13: f32,
    f14: f32,
    f15: f32,
    f16: f32,
    f17: f32,
    f18: f32,
) {
    let f13 = f13 / f12;
    let f8 = 1.0f32;
    let f7 = f17 - f15;
    let f6 = f8 - f13;
    if a0 != 0 {
        let mut f1 = f13 * f6;
        let mut f4 = 3.0f32;
        let mut f0 = f6 * f6;
        let mut f2 = f13 * f13;
        f1 = f1 * f6;
        f0 = f0 * f6;
        let mut f3 = f2 * f6;
        let f5 = f1 * f4;
        f1 = f1 * f12;
        f4 = f3 * f4;
        f0 = f0 + f5;
        f2 = f2 * f13;
        f1 = f1 * f16;
        f0 = f0 * f15;
        f3 = f3 * f12;
        f2 = f2 + f4;
        f0 = f0 + f1;
        f3 = f3 * f18;
        f2 = f2 * f17;
        f0 = f0 - f3;
        f0 = f0 + f2;
        m.wf(a0, f0);
    }
    if a1 != 0 {
        let mut f0 = f6 * 6.0;
        let mut f2 = f13 * 3.0;
        let mut f1 = 4.0f32;
        f0 = f13 * f0;
        f1 = f1 - f2;
        f2 = f2 + -2.0;
        f0 = f0 / f12;
        f1 = f13 * f1;
        f2 = f13 * f2;
        f1 = f8 - f1;
        f2 = f2 * f18;
        f0 = f0 * f7;
        f1 = f1 * f16;
        f0 = f0 + f1;
        f0 = f0 + f2;
        f0 = f0 * f14;
        m.wf(a1, f0);
    }
}

/// FUN_002584a8
pub fn ee_sub_2584a8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let s2 = m.call(0x0011_7c20, &[a1], &[]).0;
    if s2 == 0 {
        return;
    }
    let s0 = m.r32(a0 + 0xe4).wrapping_add(0xc0);
    let sp = stack_push(m, 0x10);
    m.w32(sp + 4, 0x10);
    m.w32(sp, (s2 << 2).wrapping_add(4));
    let s3 = m.call(0x001a_e1d0, &[sp], &[]).0;
    m.w32(s0 + 0x1c, s3);
    let v = m.call(0x0011_7bd0, &[a1], &[]).0;
    m.w32(s0 + 0xc, v);
    let n = s2 as i32;
    m.w32(s3, if 0x1f < n { 0x20 } else { s2 });
    if n > 0 {
        let mut s1 = 0i32;
        let mut p = s3 + 4;
        loop {
            let v = m.call(0x0011_7c80, &[a1], &[]).0;
            m.w32(sp, v);
            m.call(0x001e_d278, &[sp], &[]);
            let c = s1 < m.r32(s3) as i32;
            s1 += 1;
            if c {
                let w = m.r32(sp);
                m.w32(p, w);
            }
            p = p.wrapping_add(4);
            if !(s1 < n) {
                break;
            }
        }
    }
    stack_pop(m, 0x10);
}

/// FUN_001c8188
pub fn ee_sub_1c8188<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let sp = stack_push(m, 0x50);
    m.call(0x001a_e478, &[sp], &[]);
    let s1 = m.call(0x001c_8090, &[], &[]).0;
    if s1 == 0 {
        m.call(0x001a_e4a0, &[sp, 2], &[]);
        stack_pop(m, 0x50);
        return;
    }
    if a0 != 0 {
        m.call(0x0025_81c0, &[sp + 0x10, s1, 0x003d_4a60], &[]);
        let s0 = m.r32(sp + 0x14);
        m.call(0x0012_9ec8, &[sp + 0x10], &[]);
        if s0 == 0 {
            m.call(0x001a_bd98, &[0x002e_65e0, 0], &[]);
        }
    }
    m.w32(sp + 0x24, (a0 != 0) as u32);
    m.w32(sp + 0x20, 0x17);
    m.call(0x0012_91a8, &[sp + 0x30, sp + 0x20], &[]);
    m.call(0x0025_8200, &[sp + 0x40, s1, sp + 0x30, 0x003d_4a60], &[]);
    m.call(0x0012_9ec8, &[sp + 0x40], &[]);
    m.call(0x0012_9ec8, &[sp + 0x30], &[]);
    m.call(0x0012_9ec8, &[sp + 0x20], &[]);
    m.call(0x001a_e4a0, &[sp, 2], &[]);
    stack_pop(m, 0x50);
}

/// FUN_00152390
pub fn ee_sub_152390<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32) {
    if (m.r32(a0 + 0x2c4) as i32) <= 0 {
        return;
    }
    let mut s3 = 0i32;
    let mut s1 = a0 + 0x2c8;
    let mut s2 = a0.wrapping_add(a3.wrapping_mul(0xc).wrapping_add(0x2d4));
    loop {
        let mut hit = false;
        if a1 == 0xffff_ffff {
            let o = m.r32(a0 + 0x150);
            let e = m.r32(s1);
            hit = m.call(0x001c_4700, &[o, e], &[]).0 != 0;
        }
        if hit || m.r32(s1) == a1 {
            if a2 != 0xffff_ffff {
                let v = m.call(0x0015_2520, &[a0, a2], &[]).0;
                if v != 0 {
                    m.w32(s1 + 8, v);
                }
            }
            let x = m.r32(a0 + 0x58);
            m.call(0x0023_03a0, &[a0, x, 2], &[]);
            let x = m.r32(a0 + 0x5c);
            m.call(0x0023_03a0, &[a0, x, 2], &[]);
            let b = m.r32(s2);
            let a = m.r32(s2.wrapping_sub(4));
            m.call(0x0015_24a0, &[a0, a, b], &[]);
            return;
        }
        let n = m.r32(a0 + 0x2c4) as i32;
        s3 += 1;
        s2 = s2.wrapping_add(0x28);
        s1 = s1.wrapping_add(0x28);
        if !(s3 < n) {
            return;
        }
    }
}

/// FUN_0020a408
pub fn ee_sub_20a408<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    if m.call(0x0020_8d70, &[a0, a1], &[]).0 == 0 {
        return a1;
    }
    let mut s0 = m.r32(0x003e_148c);
    let mut s1 = a1;
    while s0 != 0 {
        if vslot(m, a0, 88, &[s0], &[]).0 != 0 {
            if s1 == 0 || (m.call(0x0020_a168, &[a0, s0, s1], &[]).0 as i32) < 0 {
                s1 = s0;
            }
        }
        s0 = m.r32(s0 + 0x58);
    }
    if s1 == 0 {
        return s1;
    }
    if m.r32(s1 + 0x54) != 4 {
        return s1;
    }
    let x = m.r32(a0 + 0xf64);
    let y = m.r32(s1 + 0x44);
    let r = m.call(0x0015_4180, &[x, y], &[]).0;
    if r == 0 {
        return s1;
    }
    if m.r32(r + 4) != 0 {
        return a1;
    }
    let x = m.r32(a0 + 0xf64);
    m.call(0x0015_4080, &[x, r, 2], &[]);
    let f = m.rf(0x002d_ded0) + m.rf(0x002c_3db0);
    m.wf(a0 + 0xf04, f);
    a1
}

/// FUN_001af4d8: push a 0x1c-byte frame at +0x28 + 0x1c * (+0x24)++ and run its entry callbacks
pub fn ee_sub_1af4d8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, t0: u32) {
    let mut s3 = a3;
    if s3 == 0 {
        let c = m.r32(a0 + 0x24) as i32;
        if c > 0 {
            s3 = m.r32((c - 1).wrapping_mul(0x1c) as u32 + a0 + 0x30);
        }
    }
    let c = m.r32(a0 + 0x24);
    m.w32(a0 + 0x24, c.wrapping_add(1));
    let s0 = a0.wrapping_add(c.wrapping_mul(0x1c).wrapping_add(0x28));
    m.call(0x0023_33b4, &[s0, 0, 0x1c], &[]);
    m.w32(s0 + 4, a2);
    m.w32(s0, a1);
    m.w32(s0 + 8, s3);
    m.w32(s0 + 0x18, t0);
    m.w32(s0 + 0xc, 0);
    m.call(0x001a_eaa8, &[a0, 1], &[]);
    if (m.r32(s0) as i32) > 0 {
        let mut s1 = 0i32;
        loop {
            let e = m.r32(s0 + 4).wrapping_add((s1 as u32) << 5);
            let f = m.r32(e + 0x14);
            if f != 0 {
                m.call(f, &[e], &[]);
            }
            s1 += 1;
            if !(s1 < m.r32(s0) as i32) {
                break;
            }
        }
    }
    if m.call(0x001b_0108, &[a0], &[]).0 != 0 {
        let v = m.r32(a0 + 0x24);
        m.w32(a0 + 0x1ec, v);
        m.w32(a0 + 0x24, v.wrapping_sub(1));
    }
}

/// FUN_0019caa0: first-use pool allocations
pub fn ee_sub_19caa0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if a1 != 0xffff || a0 == 0 {
        return;
    }
    for (g, a, b, c) in [
        (0x002d_d2e0u32, 0x0045_20a8u32, 0x14u32, 0x2000u32),
        (0x002d_d2e4, 0x0045_20d0, 0x18, 0x800),
        (0x002d_d2e8, 0x0045_20f8, 0x28, 0x200),
        (0x002d_d2ec, 0x0045_2120, 0x1c, 0x800),
        (0x002d_d2f0, 0x0045_2148, 0xc, 0xc8),
        (0x002d_d2f4, 0x0045_2170, 0x14, 0x100),
    ] {
        let v = m.r32(g).wrapping_add(1);
        m.w32(g, v);
        if v == 1 {
            m.call(0x0011_cca8, &[a, b, c, 0x10], &[]);
        }
    }
}

/// FUN_001b1900
pub fn ee_sub_1b1900<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let s2 = m.r32(a0 + 0x220);
    let p = m.call(0x001a_62e8, &[a0], &[]).1;
    let s0 = m.call(0x001c_4af8, &[0x003d_d490], &[p]).0;
    if s0 == 0 {
        return s2;
    }
    let q = m.call(0x001a_62e8, &[s0], &[]).1;
    let d = [q[0] - p[0], q[1] - p[1], q[2] - p[2], q[3] - p[3]];
    let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    if m.rf(a0 + 0x154) < len {
        return s2;
    }
    if m.rf(a0 + 0x150) < len {
        let v = vslot(m, s0, 32, &[], &[f4(d[1])]).1;
        let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        if l < 150.0 {
            return s2;
        }
    }
    let f = m.call(0x0024_58f0, &[], &[f4(d[1]), f4(d[0])]).1[0];
    m.call(0x001b_1818, &[a0], &[f4(f)]);
    3
}

/// FUN_001fae18: identical copy of FUN_0019caa0
pub fn ee_sub_1fae18<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    ee_sub_19caa0(m, a0, a1)
}

/// FUN_00225880: identical copy of FUN_0019caa0
pub fn ee_sub_225880<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    ee_sub_19caa0(m, a0, a1)
}

/// FUN_001df260
pub fn ee_sub_1df260<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    if (m.r32(a0 + 0x198) as i32) <= 0 {
        return;
    }
    let sp = stack_push(m, 0x90);
    let mut s0 = 0i32;
    loop {
        let s1 = m.r32(a0 + 0x19c).wrapping_add((s0 as u32).wrapping_mul(0x150));
        m.call(0x001d_ef98, &[a0, s1, sp, sp + 0x80], &[]);
        let mut v = m.r32(s1 + 0x148);
        let s4 = s0 + 1;
        if v == 0 {
            let f = m.rf(sp + 0x80);
            m.wf(s1 + 0x140, f);
            m.wf(s1 + 0x144, f);
        } else {
            let mut f20 = m.rf(sp + 0x80);
            let mut s2 = 0u32;
            loop {
                let f = m.rf(v + 0xa0);
                if f < f20 {
                    f20 = f;
                    s2 = v;
                }
                v = m.r32(v + 0xac);
                if v == 0 {
                    break;
                }
            }
            let b = sp + 0x20;
            m.call(0x0023_33b4, &[b, 0, 0x60], &[]);
            m.w32(b, a0);
            let x = m.r32(s2);
            m.w32(b + 4, x);
            let x = m.r32(s2 + 0x4c);
            m.w32(b + 0xc, 0x1000);
            m.w32(b + 8, x);
            m.w32(b + 0x10, 0xffff_fff6);
            for k in (0..0x10).step_by(4) {
                let w = m.r32(s2 + 0x10 + k);
                m.w32(sp + 0x40 + k, w);
            }
            m.call(0x001f_7598, &[b], &[]);
            m.wf(s1 + 0x144, f20);
        }
        s0 = s4;
        if !(s0 < m.r32(a0 + 0x198) as i32) {
            break;
        }
    }
    stack_pop(m, 0x90);
}

/// FUN_001e7668: locate the key-time segment for FUN_001e66c8's time (cursor +0x178 into table +0xf0)
pub fn ee_sub_1e7668<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) -> i32 {
    let f1 = m.call(0x001e_66c8, &[a0], &[]).1[0];
    let mut i = m.r32(a0 + 0x178) as i32;
    let tbl = m.r32(a0 + 0xf0);
    let f0 = m.rf(((i as u32) << 2).wrapping_add(tbl));
    if f0 < f1 {
        let v1 = (m.r32(a0 + 0xe8) as i32).wrapping_sub(1);
        if !(i < v1) || f1 < f0 {
            i -= 1;
        } else {
            i += 1;
            loop {
                if !(i < v1) {
                    i -= 1;
                    break;
                }
                let t = m.r32(a0 + 0xf0);
                let f = m.rf(((i as u32) << 2).wrapping_add(t));
                if f1 < f {
                    i -= 1;
                    break;
                }
                i += 1;
            }
        }
    } else if i > 0 && !(f0 <= f1) {
        i -= 1;
        loop {
            if i <= 0 {
                break;
            }
            let t = m.r32(a0 + 0xf0);
            let f = m.rf(((i as u32) << 2).wrapping_add(t));
            if f <= f1 {
                break;
            }
            i -= 1;
        }
    }
    if a1 != 0 {
        let t = m.r32(a0 + 0xf0);
        let f = m.rf(((i as u32) << 2).wrapping_add(t));
        m.wf(a1, f1 - f);
    }
    if a2 != 0 {
        let t = m.r32(a0 + 0xf0);
        let p = ((i as u32) << 2).wrapping_add(t);
        let d = m.rf(p + 4) - m.rf(p);
        m.wf(a2, d);
    }
    m.w32(a0 + 0x178, i as u32);
    i
}

/// FUN_00238138: sceStdio close-style RPC (sema wait on a stack result word)
pub fn ee_sub_238138<M: QMem + ?Sized>(m: &mut M) -> i32 {
    let s2 = 0x0044_a680u32;
    let s0 = m.call(0x0023_76d0, &[], &[]).0;
    m.call(0x0023_7b50, &[1], &[]);
    if m.r32(0x002c_4a7c) == 0 || s0 == 0 || m.r32(s0 + 4) == 0 {
        m.call(0x0023_7b80, &[], &[]);
        return -9;
    }
    let sp = stack_push(m, 0x40);
    let x = m.r32(s0);
    m.w32(s2 + 0xc, x);
    m.w32(s2 + 0x10, ((s0.wrapping_sub(0x0044_bb80) as i32) >> 4) as u32);
    m.w32(sp + 0x14, 1);
    m.w32(sp + 0x24, 0x002a_aac8);
    m.w32(sp + 0x18, 0);
    let s1 = m.call(0x0023_4a20, &[sp + 0x10], &[]).0;
    m.w32(s2, s1);
    m.w32(s2 + 4, sp + 0x30);
    m.w32(s2 + 8, 4);
    m.w32(sp, 0);
    m.w32(sp + 8, 0);
    let s3 = 0x0044_b2c0u32;
    let v = m.call(0x0023_75e0, &[0x0044_bd80, 1, 0, s2, 0x14, s3, 4, 0], &[]).0 as i32;
    let r;
    if v >= 0 {
        m.w32(s0 + 4, 0);
        // uncached alias 0x2044b2c0 of the same word
        let w = m.r32(s3);
        m.call(0x0023_7b80, &[], &[]);
        if w == 0 {
            m.call(0x0023_4a30, &[s1], &[]);
            r = -0xb;
        } else {
            m.call(0x0023_4a60, &[s1], &[]);
            m.call(0x0023_4a30, &[s1], &[]);
            let v = m.r32(sp + 0x30) as i32;
            r = if -1 < v { 0 } else { v };
        }
    } else {
        m.call(0x0023_4a30, &[s1], &[]);
        m.call(0x0023_7b80, &[], &[]);
        r = -0xb;
    }
    stack_pop(m, 0x40);
    r
}

/// shared body of FUN_00255588 / FUN_00255c00: pool-index link packing (pool count `cnt`, base table `tbl`,
/// pool span `span`, element size via multiply-by-inverse `inv` >> `sh`)
fn pool_link<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, cnt: u32, tbl: u32, span: u32, inv: u32, sh: u32) {
    let find = |m: &mut M, x: u32| -> (u32, u32) {
        let n = m.r32(cnt);
        let mut i = 0u32;
        let mut p = tbl;
        if n != 0 {
            loop {
                let b = m.r32(p);
                if !(x < b) && x < b.wrapping_add(span) {
                    return ((x.wrapping_sub(b).wrapping_mul(inv) as i32 >> sh) as u32, i);
                }
                i += 1;
                p += 4;
                if !(i < n) {
                    break;
                }
            }
        }
        (0, 0)
    };
    let (e0, p0) = find(m, a0);
    let q = m.r32(a1);
    if q != 0 {
        let mut v = m.r32(q + 4) & 0xffff_e000 | e0 & 0x1fff;
        m.w32(q + 4, v);
        v = v & 0xffff_9fff | (p0 & 3) << 0xd;
        m.w32(q + 4, v);
    }
    let x = m.r32(a1);
    let (e1, p1) = find(m, x);
    let v = m.r32(a0 + 4) & 0xe000_ffff | (e1 & 0x1fff) << 0x10;
    let v = v & 0x9fff_ffff | (p1 & 3) << 0x1d;
    m.w32(a0 + 4, v);
    m.w32(a1, a0);
}

/// FUN_00255588: link a0 at the head of *a1 (0x28-byte pool elements)
pub fn ee_sub_255588<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    pool_link(m, a0, a1, 0x002d_d2a4, 0x0045_20e8, 0x5000, 0xcccc_cccd, 3);
}

/// FUN_00255c00: link a0 at the head of *a1 (0xc-byte pool elements)
pub fn ee_sub_255c00<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    pool_link(m, a0, a1, 0x002d_d2c4, 0x0045_2138, 0x960, 0xaaaa_aaab, 2);
}

/// FUN_001b1160
pub fn ee_sub_1b1160<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let mut f21 = 0.0f32;
    let s0 = match m.r32(a0 + 0x220) {
        1 => {
            f21 = m.rf(a0 + 0x1cc);
            m.r32(a0 + 0x16c)
        }
        4 => {
            f21 = m.rf(a0 + 0x1d0);
            m.r32(a0 + 0x170)
        }
        _ => 0,
    };
    if s0 == 0 {
        return;
    }
    let p = m.call(0x001a_62e8, &[a0], &[]).1;
    let f12 = m.rf(a0 + 0x174);
    m.call(0x001e_6ed8, &[s0, 0, 0, 0, a0 + 0x174, 0], &[p, f4(f12)]);
    let t = m.rf(a0 + 0x174) + m.rf(a0 + 0x178);
    let f20 = m.call(0x001e_6660, &[s0], &[f4(t)]).1[0];
    let lo = m.call(0x001e_6610, &[s0], &[]).1[0];
    let mut inside = false;
    if !(f20 < lo) {
        let hi = m.call(0x001e_6630, &[s0], &[]).1[0];
        inside = !(hi < f20);
    }
    if !inside {
        if m.r32(a0 + 0x220) == 1 {
            m.call(0x001b_1328, &[a0, 2], &[]);
        }
        return;
    }
    let sp = stack_push(m, 0x10);
    m.call(0x001e_67b8, &[s0, sp, 0], &[f4(f20)]);
    let b = m.rv(sp);
    stack_pop(m, 0x10);
    let d = [b[0] - p[0], b[1] - p[1], b[2] - p[2], b[3] - p[3]];
    let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    let mut v = m.rv(0x002a_2da0);
    if !(len < f32::from_bits(0x38d1_b717)) {
        let q = 1.0 / len;
        v = [d[0] * q, d[1] * q, d[2] * q, d[3] * q];
    }
    let v = [v[0] * f21, v[1] * f21, v[2] * f21, v[3] * f21];
    vslot(m, a0, 34, &[], &[v]);
}

/// FUN_001a3968
pub fn ee_sub_1a3968<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let f21 = 0.0f32;
    let f20 = m.rf(a0 + 0xf8).max(m.rf(a0 + 0x150));
    let f12 = m.rf(a0 + 0x154).max(30.0);
    m.call(0x001a_38a8, &[a0 + 0x124, a0 + 0x128], &[f4(f12), f4(f21), f4(f21), f4(f20)]);
    let sp = stack_push(m, 0x10);
    m.call(0x0011_dfa0, &[a0 + 0xf0, sp], &[]);
    m.call(0x001e_d2d8, &[sp], &[]);
    let mut f1 = m.rf(sp);
    let mut f2 = m.rf(sp + 4);
    let mut f3 = m.rf(sp + 8);
    stack_pop(m, 0x10);
    let mut f0 = m.rf(a0 + 0x150);
    m.wf(a0 + 0x130, f1);
    m.wf(a0 + 0x134, f2);
    m.wf(a0 + 0x138, f3);
    m.wf(a0 + 0x13c, f0);
    if f21 < f20 {
        f0 = f0 / f20;
        f1 = f1 / f20;
        f2 = f2 / f20;
        f3 = f3 / f20;
        m.wf(a0 + 0x13c, f0);
        m.wf(a0 + 0x130, f1);
        m.wf(a0 + 0x134, f2);
        m.wf(a0 + 0x138, f3);
    }
    let f = -m.rf(a0 + 0x128);
    let v1 = m.r32(a0 + 0xe0);
    m.w32(a0 + 0x120, v1);
    m.wf(a0 + 0x128, f);
    if v1 == 0 {
        m.wf(a0 + 0x104, f21);
        m.wf(a0 + 0x108, f21);
        m.wf(a0 + 0x110, 1.0);
        m.wf(a0 + 0x114, f21);
        m.wf(a0 + 0x100, f21);
        let v = vslot(m, a0, 30, &[], &[]).0;
        for k in [0u32, 4, 8] {
            let w = m.r32(v + k);
            m.w32(a0 + 0x140 + k, w);
        }
    } else if v1 == 1 {
        let q = m.call(0x001a_62e8, &[a0], &[]).1;
        m.wv(a0 + 0x100, q);
        for k in [0u32, 4, 8] {
            let w = m.r32(a0 + 0x100 + k);
            m.w32(a0 + 0x140 + k, w);
        }
        let x = m.rf(a0 + 0x164);
        let y = m.rf(a0 + 0x168);
        m.call(0x001a_3730, &[a0 + 0x12c, a0 + 0x14c], &[f4(x), f4(1.0), f4(y), f4(f21)]);
        let a = m.rf(a0 + 0x12c);
        let b = m.rf(a0 + 0x14c);
        m.wf(a0 + 0x110, a);
        m.wf(a0 + 0x114, b);
    }
    let f3 = m.rf(a0 + 0xf8);
    let f20 = m.rf(a0 + 0x150).max(f3);
    let a = m.rf(a0 + 0x110) * f20;
    let b = m.rf(a0 + 0x114) * f20;
    m.wf(a0 + 0x110, a);
    m.wf(a0 + 0x114, b);
    m.call(0x001a_3b38, &[a0], &[]);
}

/// FUN_001c8730: state change (+0x180) with exit / enter actions (jump table 0x2a77a0)
pub fn ee_sub_1c8730<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: i32) {
    if a1 as u32 == m.r32(a0 + 0x180) {
        return;
    }
    if m.r32(a0 + 0x1a4) != 0 {
        m.call(0x001f_e738, &[0x003d_d040], &[]);
        m.w32(a0 + 0x1a4, 0);
    }
    let p = m.r32(a0 + 0x19c);
    if p != 0 {
        m.call(0x001d_feb8, &[p], &[]);
    }
    let old = m.r32(a0 + 0x180) as i32;
    if old == 2 {
        m.call(0x001c_8918, &[a0, 0xffff_ffff], &[]);
        m.call(0x0020_f160, &[0x003d_d490], &[]);
    } else if old < 3 {
        if old == 0 && m.r32(a0 + 0x190) != 0 {
            m.call(0x0020_f198, &[0x003d_d490], &[]);
        }
    } else if old == 3 {
        let x = m.r32(a0 + 0x200);
        if x != 0xffff_ffff {
            m.call(0x001d_af38, &[x, 1], &[]);
        }
        let y = m.r32(a0 + 0x1fc);
        if y != 0 {
            let v = m.call(0x001f_e5f0, &[0x003d_d040, y, 1], &[]).0;
            m.w32(a0 + 0x1a4, v);
        }
    }
    m.w32(a0 + 0x180, a1 as u32);
    let x = m.r32(0x002d_ded0);
    m.w32(a0 + 0x184, x);
    match a1 {
        -1 => return,
        0 | 5 => {
            let o = m.r32(a0 + 0x190);
            if o != 0 {
                vslot(m, o, 78, &[1], &[]);
                m.w32(a0 + 0x190, 0);
                m.call(0x0020_f1a8, &[0x003d_d490], &[]);
            }
        }
        1 => {
            vslot(m, a0, 78, &[1], &[]);
        }
        2 => {
            m.w32(a0 + 0x194, 5);
            m.call(0x001c_8918, &[a0, 1], &[]);
            m.call(0x0020_f128, &[0x003d_d490], &[]);
            let y = m.r32(a0 + 0x1f4);
            if y != 0 {
                let v = m.call(0x001f_e5f0, &[0x003d_d040, y, 0], &[]).0;
                m.w32(a0 + 0x1a4, v);
            }
        }
        _ => {}
    }
    let s = m.r32(a0 + 0x180);
    let q = m.r32(a0.wrapping_add(s << 2).wrapping_add(0x20c));
    m.w32(a0 + 0x19c, q);
    if q != 0 {
        let b = m.r32(a0 + 0x190);
        m.call(0x001d_fd58, &[q, b], &[]);
    }
}

/// FUN_0019da78
pub fn ee_sub_19da78<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    const G: u32 = 0x002b_f920;
    let x = m.r32(a0 + 0x14);
    if x != 0 {
        m.call(0x0020_2d38, &[x], &[]);
    }
    let st = m.r32(a0 + 4) as i32;
    if st == 0 {
        let p2 = m.r32(a0 + 8);
        let b4 = p2 == 0 || m.r32(p2 + 0x58) == 2;
        let o = m.r32(a0);
        m.w32(G, 0xffff_ffff);
        let mut b3 = true;
        if m.r32(o + 0x20) == 1 {
            let p4 = m.r32(a0 + 0x10);
            b3 = p4 == 0 || m.r32(p4 + 0x58) == 2;
        }
        if b4 && b3 {
            m.call(0x0019_d660, &[a0, 1], &[]);
        }
    } else if st < 1 {
        if st == -1 {
            return;
        }
    } else if st == 2 {
        let p2 = m.r32(a0 + 8);
        let b4 = p2 == 0 || (m.r32(p2 + 0x58) as i32) < 2;
        let o = m.r32(a0);
        let mut b3 = true;
        if m.r32(o + 0x20) == 1 {
            let p4 = m.r32(a0 + 0x10);
            b3 = p4 == 0 || (m.r32(p4 + 0x58) as i32) < 2;
        }
        if b4 && b3 {
            m.call(0x0019_d660, &[a0, 0xffff_ffff], &[]);
        }
        return;
    }
    let a1 = m.r32(a0 + 8);
    if a1 == 0 {
        return;
    }
    if m.r32(a0 + 4) != 0 {
        let mut done = false;
        let o = { let t = m.r32(a0); m.r32(t) };
        if m.r32(o + 0x10ec) == 4 {
            if m.r32(G) != 4 {
                m.call(0x0021_3b00, &[a1, 0x353], &[]);
            }
            let o = { let t = m.r32(a0); m.r32(t) };
            if m.r32(o + 0x10ec) == 4 {
                let v = m.r32(o + 0x10e4);
                m.w32(G, v);
                done = true;
            }
        }
        if !done {
            if m.r32(G) == 4 {
                let p2 = m.r32(a0 + 8);
                m.call(0x0021_3928, &[p2, 0x353], &[]);
            }
            let o = { let t = m.r32(a0); m.r32(t) };
            let v = m.r32(o + 0x10e4);
            m.w32(G, v);
        }
    }
    let o = { let t = m.r32(a0); m.r32(t) };
    let f = (m.rf(o + 0x1a8) + f32::from_bits(0x4049_0fdb)) * f32::from_bits(0x3e22_f983);
    let p2 = m.r32(a0 + 8);
    m.call(0x0021_2e40, &[p2, 0x352], &[f4(f)]);
    let o = m.r32(a0);
    let p2 = m.r32(a0 + 8);
    let f = m.rf(o + 0x3c);
    m.call(0x0021_2e40, &[p2, 0x351], &[f4(f)]);
}

pub fn ee_sub_1c84d8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let s0 = a0;
    let s1 = m.r32(s0 + 0x188);
    loop {
        let p = m.r32(s0 + 0x150);
        if m.call(0x1c46c0, &[p, 0x30], &[[0.0, 0.0, 0.0, 0.0]]).0 != 0 {
            let o = m.r32(s0 + 0x19c);
            m.call(0x1e0020, &[o, 0x655], &[]);
            m.call(0x1c8918, &[s0, 3], &[]);
        }
        let st = m.r32(s0 + 0x188) as i32;
        if st == 1 {
            let w = m.r32(s0 + 0x204);
            if m.rf(w + 0xa8) <= m.rf(0x2dded0) {
                m.call(0x1c8918, &[s0, 0], &[]);
            }
        } else if st == 0 {
            let p = m.r32(s0 + 0x150);
            if m.call(0x1c46c0, &[p, 0xc0], &[[0.0, 0.0, 0.0, 0.0]]).0 != 0 {
                m.call(0x1c8918, &[s0, 2], &[]);
            } else {
                let p = m.r32(s0 + 0x150);
                let t0 = m.r32(s0 + 0x194);
                let (x, y) = (m.rf(p + 0x10), m.rf(p + 0x14));
                let f2 = m.rf(0x2c0fe8);
                let mut f1 = m.rf(0x2a7798);
                let mut best = u32::MAX;
                for i in 0..9u32 {
                    let t = m.rv(0x2c0ff0 + i.wrapping_mul(0x10));
                    let (dx, dy, dz) = (x - t[0], y - t[1], 0.0 - t[2]);
                    let mut d = (dx * dx + dy * dy + dz * dz).sqrt();
                    if i == t0 { d *= f2; }
                    if d < f1 { f1 = d; best = i; }
                }
                if best != t0 {
                    m.w32(s0 + 0x194, best);
                    m.call(0x1c8918, &[s0, 1], &[]);
                }
            }
        } else if st == 2 {
            let a = m.r32(s0 + 0x1a0);
            let go = a == 0 || m.call(0x131968, &[a], &[]).0 != 0;
            if go {
                let v1 = m.r32(s0 + 0x198) as i32;
                let v = (v1.wrapping_mul(10) % 1000).wrapping_add(1).wrapping_add({ let t = m.r32(s0 + 0x194); t } as i32) as u32;
                m.w32(s0 + 0x198, v);
                if v == m.r32(s0 + 0x1f0) {
                    let o = m.r32(s0 + 0x19c);
                    m.call(0x1e0020, &[o, 0x655], &[]);
                    m.call(0x1c8918, &[s0, 3], &[]);
                } else {
                    m.call(0x1c8918, &[s0, 0], &[]);
                }
            }
        }
        if m.r32(s0 + 0x188) == s1 { break; }
    }
}

pub fn ee_sub_1e0070<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let s0 = a0;
    let mut v1 = m.r32(s0 + 0x84) as i32;
    let mut i: i32 = 0;
    while i < v1 {
        let p = s0.wrapping_add(0x88).wrapping_add((i as u32) << 2);
        if m.r32(p) == a1 {
            let n = ((v1.wrapping_sub(i) as u32) << 2).wrapping_sub(4);
            m.call(0x2332b8, &[p, p + 4, n], &[]);
            let c = m.r32(s0 + 0x84).wrapping_sub(1);
            m.w32(s0 + 0x84, c);
            return;
        }
        v1 = m.r32(s0 + 0x84) as i32;
        i += 1;
    }
}

pub fn ee_sub_1acc38<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: i32) {
    if a1 as u32 == m.r32(a0 + 0x44) { return; }
    if a1 >= 4 || a1 >= 2 {
        if a1 < 4 { m.w32(0x2e65e0 + 0x8000 + 0x7d50, 0); }
    }
    m.w32(a0 + 0x44, a1 as u32);
    let f = m.rf(0x2ddf0c);
    m.wf(a0 + 0x48, f);
}

pub fn ee_sub_1f0598<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let s0 = a0;
    let r = m.call(0x1f0638, &[s0], &[]).0;
    if r != 0 { return r; }
    let p = m.r32(s0 + 0x30);
    if p != 0 {
        m.w32(s0 + 0x30, 0);
        m.call(0x130b18, &[p], &[]);
    }
    let q = m.r32(s0 + 0x24);
    m.call(0x11a2e8, &[q.wrapping_add(0x68), s0], &[]);
    m.call(0x11a2e8, &[0x3e126c, s0], &[]);
    vslot(m, s0, 2, &[3, 0, 0, 0, 0], &[]);
    vslot(m, s0, 1, &[], &[]).0
}

pub fn ee_sub_1d7ed8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32) -> u32 {
    let s2 = a0;
    let sp = stack_push(m, 0x20);
    m.w32(sp, u32::MAX);
    let ok = m.call(0x1d7da0, &[a1, a2, sp + 0x10], &[]).0;
    let e = m.r32(sp + 0x10);
    let kind = if ok != 0 { m.r8(e + 2) as i8 as i32 } else { -1 };
    if kind != -1 {
        let t0 = a3.wrapping_add(m.r16(e + 4) as u32);
        let idx = m.r32(sp + 0x14);
        match kind as u32 {
            0 | 1 | 2 | 10 | 9 => {
                let v = m.r32(t0.wrapping_add(idx << 2));
                m.call(0x129518, &[sp, v], &[]);
            }
            3 => {
                let f = m.rf(t0.wrapping_add(idx << 2));
                m.call(0x129558, &[sp], &[[f, 0.0, 0.0, 0.0]]);
            }
            4 => {
                let v = if m.r8(e + 3) != 0 {
                    let a2 = idx as i32;
                    let q = (if a2 >= 0 { a2 } else { a2.wrapping_add(7) }) >> 3;
                    let sh = a2.wrapping_sub(q << 3) as u32;
                    let bit = (1u32.wrapping_shl(sh & 31)) & 0xff;
                    ((m.r8(t0.wrapping_add(q as u32)) as u32 & bit) != 0) as u32
                } else {
                    m.r32(t0)
                };
                m.call(0x129598, &[sp, v], &[]);
            }
            5 => { m.call(0x1296a8, &[sp, t0.wrapping_add(idx << 4)], &[]); }
            6 => { m.call(0x129808, &[sp, t0.wrapping_add(idx << 4)], &[]); }
            7 => { m.call(0x129878, &[sp, t0.wrapping_add(idx << 3)], &[]); }
            8 => { m.call(0x1298f0, &[sp, idx.wrapping_mul(0xc).wrapping_add(t0)], &[]); }
            _ => {}
        }
    }
    m.call(0x1291a8, &[s2, sp], &[]);
    m.call(0x129ec8, &[sp], &[]);
    stack_pop(m, 0x20);
    s2
}

pub fn ee_sub_247330<M: QMem + ?Sized>(m: &mut M, f12: f32) -> f32 {
    let c = f32::from_bits;
    let x = f12;
    let hx = x.to_bits() as i32;
    let ix = hx & 0x7fffffff;
    let pq = |z: f32| -> (f32, f32) {
        let mut f1 = z * c(0x3811ef08) + c(0x3a4f7f04);
        f1 = z * f1 + c(0xbd241146);
        f1 = z * f1 + c(0x3e4e0aa8);
        f1 = z * f1 + c(0xbea6b090);
        f1 = z * f1 + c(0x3e2aaaab);
        let p = z * f1;
        let mut f2 = z * c(0x3d9dc62e) + c(0xbf303361);
        f2 = z * f2 + c(0x4001572d);
        f2 = z * f2 + c(0xc019d139);
        let q = z * f2 + c(0x3f800000);
        (p, q)
    };
    if ix == 0x3f800000 {
        return if hx > 0 { 0.0 } else { c(0x40490fdb) };
    }
    if ix > 0x3f800000 {
        return crate::engine::math::ps2f::ps2_div(x - x, x - x);
    }
    if ix <= 0x3effffff {
        if ix <= 0x23000000 {
            return c(0x3fc90fdb);
        }
        let (p, q) = pq(x * x);
        let r = p / q;
        return c(0x3fc90fda) - (x - (c(0x33a22168) - x * r));
    }
    if hx < 0 {
        let z = (x + 1.0) * 0.5;
        let (p, q) = pq(z);
        let s = m.call(0x248ca8, &[], &[[z, 0.0, 0.0, 0.0]]).1[0];
        let r = p / q;
        let w = r * s - c(0x33a22168);
        let t = s + w;
        return c(0x40490fda) - (t + t);
    }
    let z = (1.0 - x) * 0.5;
    let s = m.call(0x248ca8, &[], &[[z, 0.0, 0.0, 0.0]]).1[0];
    let df = c(s.to_bits() & 0xfffff000);
    let cc = (z - df * df) / (s + df);
    let (p, q) = pq(z);
    let r = p / q;
    let t = df + (r * s + cc);
    t + t
}

pub fn ee_sub_11d9e0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, f12: f32, f13: f32, f14: f32, f15: f32) -> f32 {
    let (step, lo, hi) = (f13, f14, f15);
    let x = if f12 < lo { lo } else if hi < f12 { hi } else { f12 };
    let mut ev = |m: &mut M, t: f32| m.call(a0, &[a1], &[[t, 0.0, 0.0, 0.0]]).1[0];
    let ax = lo.max(x - step);
    let mut a = (ax, ev(m, ax));
    let mut b = (x, ev(m, x));
    let cx = hi.min(x + step);
    let mut c = (cx, ev(m, cx));
    loop {
        if a.1 < b.1 {
            c = b;
            b = a;
            let t = lo.max(b.0 + b.0 - c.0);
            a = (t, ev(m, t));
        } else if c.1 < b.1 {
            a = b;
            b = c;
            let t = hi.min(b.0 + b.0 - a.0);
            c = (t, ev(m, t));
        } else {
            break;
        }
    }
    let mut s2: i32 = 9;
    loop {
        let t1 = (a.0 + b.0) * 0.5;
        let m1 = (t1, ev(m, t1));
        let t2 = (b.0 + c.0) * 0.5;
        let m2 = (t2, ev(m, t2));
        if m1.1 < b.1 {
            if m1.1 < m2.1 { c = b; b = m1; } else { a = b; b = m2; }
        } else if b.1 <= m2.1 {
            a = m1; c = m2;
        } else {
            a = b; b = m2;
        }
        s2 -= 1;
        if s2 < 0 { break; }
    }
    if a2 != 0 { m.wf(a2, b.0); }
    if a3 != 0 { m.wf(a3, b.1); }
    b.1
}

pub fn ee_sub_21c888<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let (s7, s5) = (a0, a1);
    let sp = stack_push(m, 0x20);
    let n = m.call(0x117d00, &[s5], &[]).0 as i32;
    m.w32(sp + 0x10, n as u32);
    let mut s0: i32 = 0;
    if n > 0 {
        loop {
            s0 += 1;
            m.w32(sp + 4, 0x10);
            m.w32(sp, 0x50);
            m.w32(sp + 0x14, s0 as u32);
            let s3 = m.call(0x1ae2a0, &[sp], &[]).0;
            let v = m.call(0x117bd0, &[s5], &[]).0;
            m.w32(s3 + 8, v);
            for off in [0xcu32, 0x10, 0x14, 0x18] {
                let f = m.call(0x117e38, &[s5], &[]).1[0];
                m.wf(s3 + off, f);
            }
            m.call(0x117eb0, &[s5, s3 + 0x20], &[]);
            m.call(0x117eb0, &[s5, s3 + 0x30], &[]);
            let cnt = m.call(0x117d00, &[s5], &[]).0;
            m.w32(s3, cnt);
            m.w32(sp + 4, 0x10);
            m.w32(sp, cnt.wrapping_mul(0x70));
            let arr = m.call(0x1ae2a0, &[sp], &[]).0;
            m.w32(s3 + 4, arr);
            let mut s4: i32 = 0;
            if (m.r32(s3) as i32) > 0 {
                let mut s6: u32 = 0;
                loop {
                    let s2 = m.r32(s3 + 4).wrapping_add(s6);
                    s4 += 1;
                    m.w32(s2 + 0x60, s3);
                    s6 = s6.wrapping_add(0x70);
                    let k = m.call(0x117d50, &[s5], &[]).0;
                    let r = vslot(m, s7, 50, &[k], &[]).0;
                    m.w32(s2 + 0x64, r);
                    for q in [0u32, 0x10, 0x20] {
                        let t = m.rv(r + q);
                        m.wv(s2 + q, t);
                    }
                    let o = m.call(0x12e1f0, &[0x20], &[]).0;
                    m.w32(o, 3);
                    m.w32(o + 0x18, 0x2db338);
                    m.w32(o + 4, s7);
                    let p = m.r32(s2 + 0x64);
                    m.w32(o + 0xc, u32::MAX);
                    m.w32(o + 8, p);
                    m.w32(o + 0x1c, s2);
                    m.w32(o + 0x14, 0x10);
                    let p = m.r32(s2 + 0x64);
                    m.call(0x230018, &[s7, p, o], &[]);
                    let p = m.r32(s2 + 0x64);
                    m.w32(p + 0xa8, s2);
                    if s4 >= m.r32(s3) as i32 { break; }
                }
            }
            let mut a = m.r32(s3) as i32;
            if a > 0 {
                s4 = 0;
                let mut so: u32 = 0;
                loop {
                    let s2 = m.r32(s3 + 4).wrapping_add(so);
                    let v = if s4 < a - 1 {
                        let nx = m.r32(s2 + 0xd4);
                        let p = m.r32(s2 + 0x64);
                        let q = m.rv(nx + 0x70);
                        m.call(0x22fc48, &[0, p], &[q]).1
                    } else if s4 <= 0 {
                        m.rv(s3 + 0x30)
                    } else {
                        let pa = m.r32(s2.wrapping_sub(0xc));
                        let p = m.r32(s2 + 0x64);
                        let q = m.rv(s2.wrapping_sub(0x40));
                        m.call(0x22fcc8, &[pa, p], &[q]).1
                    };
                    m.wv(s2 + 0x30, v);
                    let p = m.r32(s2 + 0x64);
                    let q = m.rv(s2 + 0x30);
                    s4 += 1;
                    so = so.wrapping_add(0x70);
                    let v = m.call(0x22fc48, &[p, 0], &[q]).1;
                    m.wv(s2 + 0x40, v);
                    a = m.r32(s3) as i32;
                    if s4 >= a { break; }
                }
            }
            s0 = m.r32(sp + 0x14) as i32;
            if s0 >= m.r32(sp + 0x10) as i32 { break; }
        }
    }
    stack_pop(m, 0x20);
}

pub fn ee_sub_225ba0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let s2 = a0;
    let sp = stack_push(m, 0x80);
    let a = m.r32(s2 + 0x180);
    if a != 0 && m.r32(s2 + 0x184) != 0 {
        let v = m.call(0x1a62e8, &[a], &[]).1;
        m.wv(sp + 0x70, v);
        let w = m.call(0x1a62e8, &[s2], &[]).1;
        let v = m.rv(sp + 0x70);
        let d = [v[0] - w[0], v[1] - w[1], v[2] - w[2], v[3] - w[3]];
        m.call(0x11b7e0, &[sp + 0x30], &[d]);
        for q in [0u32, 0x10, 0x20] {
            let t = m.rv(sp + 0x30 + q);
            m.wv(sp + q, t);
        }
        m.call(0x11bc88, &[sp, sp + 0x60, sp + 0x64], &[]);
        let (f2, f3) = (m.rf(sp + 0x60), m.rf(sp + 0x64));
        let (o1, o0) = (m.rf(s2 + 0x18c), m.rf(s2 + 0x190));
        m.wf(s2 + 0x18c, f2);
        m.wf(s2 + 0x190, f3);
        m.wf(s2 + 0x194, f2 - o1);
        m.wf(s2 + 0x198, f3 - o0);
    } else {
        let p = m.r32(s2 + 0x150);
        if p != 0 {
            let z = 0.0f32;
            let x = m.rf(p + 0x10) + m.rf(p + 0x20);
            let f21a = m.call(0x11cfe0, &[], &[[x, 0.0, 0.0, 0.0], [1.0, 0.0, 0.0, 0.0]]).1[0];
            let p = m.r32(s2 + 0x150);
            let y = m.rf(p + 0x14);
            let f6 = m.call(0x11cfe0, &[], &[[y, 0.0, 0.0, 0.0], [1.0, 0.0, 0.0, 0.0]]).1[0];
            let f3 = m.rf(s2 + 0x1bc);
            let f20a = (1.0 - f3) * 0.5 + f3;
            let f12 = f20a * 10.0;
            let f21 = ((-f21a) * f21a * f21a) * f12;
            let f23 = f3 * f32::from_bits(0x3fa66666);
            if f21 * m.rf(s2 + 0x194) <= z { m.wf(s2 + 0x194, z); }
            let mut f20 = (f6 * f6 * f6) * f12;
            let g = m.r32(0x3e0fe0);
            m.call(0x2581c0, &[sp, g, 0x3d4a60], &[]);
            let s0 = m.r32(sp + 4);
            m.call(0x129ec8, &[sp], &[]);
            if s0 == 0 { f20 = -f20; }
            if f20 * m.rf(s2 + 0x198) <= z { m.wf(s2 + 0x198, z); }
            let s1 = 0x2dded0u32;
            for off in [0x194u32, 0x198] {
                let v = m.rf(s2 + off);
                let dt = m.rf(s1 + 4);
                let r = m.call(0x11d018, &[0x2c4408, 0], &[[v, 0.0, 0.0, 0.0], [z, 0.0, 0.0, 0.0], [dt, 0.0, 0.0, 0.0]]).1[0];
                m.wf(s2 + off, r);
            }
            let v = m.rf(s2 + 0x194) + f21 * m.rf(s1 + 4);
            let r = m.call(0x11cfe0, &[], &[[v, 0.0, 0.0, 0.0], [f23 * f32::from_bits(0x3f666666), 0.0, 0.0, 0.0]]).1[0];
            m.wf(s2 + 0x194, r);
            let v = m.rf(s2 + 0x198) + f20 * m.rf(s1 + 4);
            let r = m.call(0x11cfe0, &[], &[[v, 0.0, 0.0, 0.0], [f23 * f32::from_bits(0x3f4ccccd), 0.0, 0.0, 0.0]]).1[0];
            m.wf(s2 + 0x198, r);
            let v = m.rf(s2 + 0x18c) + m.rf(s2 + 0x194) * m.rf(s1 + 4);
            m.wf(s2 + 0x18c, v);
            let v = m.rf(s2 + 0x190) + r * m.rf(s1 + 4);
            m.wf(s2 + 0x190, v);
        }
    }
    let f = m.rf(s2 + 0x18c);
    let r = m.call(0x11d948, &[s2 + 0x1a4], &[[f, 0.0, 0.0, 0.0]]).1[0];
    m.wf(s2 + 0x18c, r);
    let f = m.rf(s2 + 0x190);
    let r = m.call(0x11d948, &[s2 + 0x19c], &[[f, 0.0, 0.0, 0.0]]).1[0];
    m.wf(s2 + 0x190, r);
    if m.r32(s2 + 0x1cc) != 0 {
        let y = -r;
        let x = m.rf(s2 + 0x18c);
        let hi = if (x.to_bits() as i32) < 0 { u32::MAX } else { 0 };
        let v = [0.0, y, x, f32::from_bits(hi)];
        let o = m.r32(s2 + 0x1d8);
        m.wv(sp, v);
        m.call(0x12d108, &[o], &[v]);
    }
    stack_pop(m, 0x80);
}

pub fn ee_sub_1aeaa8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let (s1, s2) = (a0, a1);
    let old = m.r32(s1 + 0x1e8);
    if old == s2 { return; }
    let cnt = m.r32(s1 + 0x24) as i32;
    let s0 = if cnt > 0 { s1.wrapping_add((cnt as u32).wrapping_mul(0x1c).wrapping_add(0xc)) } else { 0 };
    match old {
        0 => { vslot(m, s1, 9, &[], &[]); }
        5 if s0 != 0 => {
            let v = m.r32(s0 + 0x14).wrapping_add(1);
            m.w32(s0 + 0x14, v);
            if !((v as i32) < m.r32(s0) as i32) { m.w32(s0 + 0x14, 0); }
        }
        6 if s0 != 0 => {
            let v = m.r32(s0 + 0x14).wrapping_sub(1);
            m.w32(s0 + 0x14, v);
            if (v as i32) < 0 {
                let n = m.r32(s0).wrapping_sub(1);
                m.w32(s0 + 0x14, n);
            }
        }
        _ => {}
    }
    m.w32(s1 + 0x1e8, s2);
    if s2 >= 0xc { return; }
    if s2 == 0 {
        vslot(m, s1, 10, &[], &[]);
        return;
    }
    if s0 == 0 { return; }
    let o = m.r32(s0 + 8);
    if o == 0 { return; }
    let t = m.r32(s0 + 0x18);
    match s2 {
        1 | 2 | 8 | 7 => {
            let (va, vb) = match s2 { 1 => (0, 0xc), 7 => (4, 0x10), _ => (8, 0x14) };
            let x = m.r32(t + va);
            vslot(m, o, 48, &[x], &[]);
            let t = m.r32(s0 + 0x18);
            let o = m.r32(s0 + 8);
            let x = m.r32(t + vb);
            m.call(0x213b00, &[o, x], &[]);
        }
        3 | 4 | 5 | 6 => {
            let off = match s2 { 3 => 0x14, 4 => 0xc, 5 => 0x18, _ => 0x1c };
            let x = m.r32(t + off);
            m.call(0x213b00, &[o, x], &[]);
        }
        9 => { m.call(0x213b00, &[o, 0x332], &[]); }
        11 => { m.call(0x213b00, &[o, 0x331], &[]); }
        _ => {
            let i = m.r32(s0 + 0x14);
            let b = m.r32(s0 + 4);
            let k = m.r32(b.wrapping_add(i << 5));
            if k == 0x67 || k == 0x68 {
                let (c, fa) = if k == 0x67 { (0x333, 0x3dd190) } else { (0x334, 0x3dd18c) };
                m.call(0x213b00, &[o, c], &[]);
                let o = m.r32(s0 + 8);
                m.call(0x213b00, &[o, 0x315], &[]);
                let o = m.r32(s0 + 8);
                let f = m.rf(fa);
                m.call(0x212e40, &[o, 0x315], &[[f, 0.0, 0.0, 0.0]]);
            } else if k == 0x3f {
                m.call(0x213b00, &[o, 0x335], &[]);
                let w = m.r32(0x2deb98);
                m.wf(0x2f6468, (w & 0xfff) as i32 as f32);
                let lo = m.r32(0x2deb98);
                m.wf(0x2f646c, ((lo >> 12) & 0x7ff) as i32 as f32);
            }
        }
    }
}

pub fn ee_sub_194688<M: QMem + ?Sized>(m: &mut M, _a0: u32, a1: u32, a3: u32, t0: u32, t1: u32, t2: u32, a2: V4) {
    let (s0, s5, s4, s3, s2) = (a1, a3, t0, t1 as i32, t2);
    let d3 = |a: V4, b: V4| { let (x, y, z) = (a[0] - b[0], a[1] - b[1], a[2] - b[2]); x * x + y * y + z * z };
    if m.r32(s5 + 0x10) != 0 {
        let p = m.call(0x1a62e8, &[s5], &[]).1;
        let r = m.rf(s5 + 0x38);
        if r * r < d3(p, a2) { return; }
    }
    let a2r = m.r32(s0 + 0x4d4);
    if s5 == a2r || s5 == m.r32(s0 + 0x2cc) { return; }
    let v1 = m.r32(s4 + 0x18);
    if v1 != 0 {
        let q = m.r32(s0 + 0x4d8);
        if q != 0 && v1 == m.r32(q + 0x18) { return; }
    }
    if s5 == m.r32(s0 + 0x37c) { return; }
    let a1r = m.r32(s0 + 0xf5c);
    if s5 == a1r { return; }
    let a0r = m.r32(0x2e1e40);
    if s0 == a0r && (s5 == m.r32(s0 + 0x18c4) || s5 == m.r32(s0 + 0x1f18)) { return; }
    if m.r32(s0 + 0x1000) & m.r32(s4 + 0x1c) == 0 { return; }
    if m.r32(0x2e1e4c) != 0 {
        if a2r != 0 || a1r != 0 { return; }
        if s0 == a0r && (m.r32(s0 + 0x18c4) != 0 || m.r32(s0 + 0x1f18) != 0) { return; }
    }
    let sp = stack_push(m, 0x30);
    let vt = m.r32(s4 + 0x20);
    let (f, adj) = (m.r32(vt + 0x14), m.r16(vt + 0x10) as i16 as u32);
    m.call(f, &[s4.wrapping_add(adj), s0, sp + 0x18], &[]);
    let vt = m.r32(s4 + 0x20);
    let (f, adj) = (m.r32(vt + 0x1c), m.r16(vt + 0x18) as i16 as u32);
    let f12 = m.rf(sp + 0x18);
    m.call(f, &[s4.wrapping_add(adj), s0, sp + 0x20, 0], &[[f12, 0.0, 0.0, 0.0]]);
    let pt = m.rv(sp + 0x20);
    let pr = m.r32(s4 + 0x10);
    let vt = m.r32(s4 + 0x20);
    m.w32(sp + 0xc, pr);
    m.wf(sp + 8, d3(a2, pt).sqrt());
    let (f, adj) = (m.r32(vt + 0x24), m.r16(vt + 0x20) as i16 as u32);
    m.call(f, &[s4.wrapping_add(adj), sp + 8], &[]);
    if m.rf(0x2bf664) < m.rf(sp + 8) { stack_pop(m, 0x30); return; }
    let a1 = m.r32(s4 + 0x18);
    if a1 != 0 && s3 > 0 && m.r32(s2 + 4) != 0 {
        let mut s1: i32 = 0;
        let mut a2c = s3 - 1;
        let mut e = s2;
        loop {
            let v0 = m.r32(e + 4);
            if m.r32(v0 + 0x18) == a1 {
                let e = s2.wrapping_add((s1 as u32).wrapping_mul(0x30));
                if !(m.rf(sp + 8) < m.rf(e + 8)) { stack_pop(m, 0x30); return; }
                m.call(0x1ae5a0, &[e, e + 0x30, (a2c as u32).wrapping_mul(0x30)], &[]);
                let last = s2.wrapping_add((s3 as u32).wrapping_mul(0x30)).wrapping_sub(0x30);
                m.call(0x2333b4, &[last, 0, 0x30], &[]);
                let f = m.rf(0x2bf664);
                m.wf(last + 8, f);
                break;
            }
            s1 += 1;
            e = e.wrapping_add(0x30);
            if s1 >= s3 { break; }
            a2c -= 1;
            if m.r32(e + 4) == 0 { break; }
        }
    }
    let mut s1 = s3;
    let pr = m.r32(sp + 0xc) as i32;
    let f1 = m.rf(sp + 8);
    let mut a0 = s2.wrapping_add((s3 as u32).wrapping_mul(0x30)).wrapping_sub(0x30);
    loop {
        let v1 = m.r32(a0 + 0xc) as i32;
        s1 -= 1;
        if pr < v1 || (v1 == pr && m.rf(a0 + 8) < f1) { s1 += 1; break; }
        if s1 == 0 { break; }
        a0 = a0.wrapping_sub(0x30);
    }
    if s1 != s3 {
        let e = s2.wrapping_add((s1 as u32).wrapping_mul(0x30));
        let n = ((s3 - s1) as u32).wrapping_mul(0x30).wrapping_sub(0x30);
        m.call(0x2332b8, &[e + 0x30, e, n], &[]);
        let t = m.rf(0x2dded0);
        m.w32(sp, s5);
        m.w32(sp + 4, s4);
        m.wf(sp + 0x14, t);
        m.w32(sp + 0x10, 0);
        m.w32(sp + 0x1c, 0);
        for q in [0u32, 0x20, 0x10] {
            let v = m.rv(sp + q);
            m.wv(e + q, v);
        }
    }
    stack_pop(m, 0x30);
}

pub fn ee_sub_19e1e0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let s1 = a0;
    let fv = |x: f32| -> V4 { [x, 0.0, 0.0, 0.0] };
    let d3 = |a: V4, b: V4| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    let s0 = m.r32(s1);
    let q0 = m.rv(s0 + 0x1010);
    let q1 = m.rv(0x2a2dd0);
    let dt = m.rf(0x2dded4);
    let r = m.call(0x11f088, &[0x2bf928, 0], &[q0, q1, fv(dt)]).1;
    m.wv(s0 + 0x1010, r);
    let v1 = m.r32(s1 + 0xc);
    let a = m.r32(s1);
    let s2 = m.r32(a + 0x150);
    if (m.r32(v1 + 0x2c) >> 28) & 1 == 0 {
        m.call(0x195a00, &[a + 0x2c0, 0, 0, 0], &[]);
        return;
    }
    if m.r32(s1 + 4) != 0 && m.call(0x1c46c0, &[s2, 0x40], &[fv(0.25)]).0 != 0 {
        let a = m.r32(s1);
        m.call(0x195a00, &[a + 0x2c0, 2, 0, 0], &[]);
        return;
    }
    if m.call(0x1c4660, &[s2, 0x400], &[fv(f32::from_bits(0x3e19999a))]).0 != 0 {
        let a2 = m.r32(s1);
        if vslot(m, a2, 98, &[3], &[]).0 != 0 {
            m.call(0x1c4738, &[s2, 0x400], &[]);
            let a = m.r32(s1);
            m.call(0x19cc50, &[a + 0xf00, 0, 0, 0], &[]);
        }
    }
    let mut s0 = m.r32(s1 + 4);
    let v1 = loop {
        if s0 == 0 || s0 == 4 {
            let a = m.r32(s1);
            let t = m.r32(a + 0x1030);
            if m.call(0x131968, &[t], &[]).0 != 0 {
                m.call(0x19eab0, &[s1, 1], &[]);
            }
        }
        let v1 = m.r32(s1 + 4);
        if v1 == s0 { break v1; }
        s0 = m.r32(s1 + 4);
    };
    if v1 == 0 || v1 == 4 { return; }
    let sp = stack_push(m, 0x50);
    let o = m.r32(s1 + 0xc);
    let x = m.rf(s1 + 0x10) + 50.0;
    let p = m.call(0x1e6728, &[o], &[fv(x)]).1[0];
    let o = m.r32(s1 + 0xc);
    m.call(0x1e6f10, &[o, sp, 0], &[fv(p)]);
    let o = m.r32(s1 + 0xc);
    let x = m.rf(s1 + 0x10) - 50.0;
    let p = m.call(0x1e6728, &[o], &[fv(x)]).1[0];
    let o = m.r32(s1 + 0xc);
    m.call(0x1e6f10, &[o, sp + 0x10, 0], &[fv(p)]);
    let (pa, pb) = (m.rv(sp), m.rv(sp + 0x10));
    let d = [pa[0] - pb[0], pa[1] - pb[1], pa[2] - pb[2], pa[3] - pb[3]];
    let len = d3(d, d).sqrt();
    let dir = if len < f32::from_bits(0x38d1b717) {
        m.rv(0x2a2dc0)
    } else {
        let q = 1.0 / len;
        [d[0] * q, d[1] * q, d[2] * q, d[3] * q]
    };
    let nz = dir[2];
    let call4 = 'c: {
        if nz.abs() > 0.5 {
            let s18 = m.r32(s1 + 0x18);
            if 0.0 < nz {
                if s18 != 0 { break 'c true; }
            } else if s18 == 0 {
                break 'c true;
            }
        }
        let f2 = m.rf(s1 + 0x1c);
        if f32::from_bits(0x3ecccccd) < f2 && m.r32(s1 + 0x18) != 0 && f32::from_bits(0xbe99999a) < nz {
            break 'c true;
        }
        if f2 < f32::from_bits(0xbecccccd) && m.r32(s1 + 0x18) == 0 && nz < f32::from_bits(0x3e99999a) {
            break 'c true;
        }
        false
    };
    if call4 {
        m.call(0x19eab0, &[s1, 4], &[]);
        stack_pop(m, 0x50);
        return;
    }
    let (pa, pb) = (m.rv(sp), m.rv(sp + 0x10));
    let d = [pa[0] - pb[0], pa[1] - pb[1], pa[2] - pb[2], pa[3] - pb[3]];
    let v = m.call(0x1c4768, &[s2], &[d]).1;
    let w = [m.rf(s2 + 0x10), m.rf(s2 + 0x14), 0.0, 0.0];
    let f4 = m.rf(s2 + 0x1c);
    m.wv(sp + 0x20, w);
    let mut f3 = d3(v, w);
    let a1 = m.r32(s1 + 0x20) as i32;
    match a1 {
        0 => {
            if f4 < 0.25 { m.w32(s1 + 0x20, u32::MAX); }
            else if f3 < f32::from_bits(0xbf333333) { m.w32(s1 + 0x20, 1); }
        }
        1 => {
            if f4 < 0.25 { m.w32(s1 + 0x20, u32::MAX); }
            else if f32::from_bits(0x3f333333) < f3 { m.w32(s1 + 0x20, 0); }
        }
        -1 => {
            if f32::from_bits(0x3eb33333) < f4 {
                m.w32(s1 + 0x20, if 0.0 <= f3 { 0 } else { 1 });
            }
        }
        _ => {}
    }
    let v1 = m.r32(s1 + 0x20) as i32;
    let s18 = m.r32(s1 + 0x18);
    let mut s2v: i32 = -1;
    match v1 {
        0 => { s2v = if s18 != 0 { 3 } else { 2 }; f3 = f4; }
        1 => { s2v = if s18 != 0 { 2 } else { 3 }; f3 = f4; }
        -1 => { s2v = 1; f3 = 0.0; }
        _ => {}
    }
    let mut f1 = m.rf(s1 + 0x6c);
    let mut f20;
    if f32::from_bits(0x3c23d70a) < f3 {
        f20 = (1.0 - f3) * m.rf(s1 + 0x5c) + f3 * f1;
        if s2v == 3 { f20 = -f20; }
    } else {
        f20 = 0.0;
        s2v = 1;
        f1 = m.rf(s1 + 0x6c);
    }
    if s18 != 0 { f20 = -f20; }
    let s3 = 0x2dded0u32;
    let f5 = f1.abs();
    let dt = m.rf(s3 + 4);
    let f4b = m.rf(s1 + 0x1c);
    m.wf(sp + 0x20, f5);
    m.wf(sp + 0x28, f32::from_bits(0x3e4ccccd));
    m.w32(sp + 0x24, 0);
    m.wf(s1 + 0x1c, ((-dt) * f20) / f5 + (1.0 - dt) * f4b);
    let o = m.r32(s1 + 0xc);
    if m.r32(o + 0xe4) == 0 {
        m.call(0x19e110, &[s1, sp + 0x30], &[]);
        for (lim, out) in [(0x30u32, 0x40u32), (0x34, 0x44)] {
            let (x, l, dt) = (m.rf(s1 + 0x10), m.rf(sp + lim), m.rf(s3 + 4));
            m.call(0x11d018, &[sp + 0x20, sp + out], &[fv(x), fv(l), fv(dt)]);
        }
        let f1 = -m.rf(sp + 0x44);
        let f0 = -m.rf(sp + 0x40);
        if f20 < f1 { f20 = f1; } else if f0 < f20 { f20 = f0; }
    }
    let f2 = -f20;
    m.wf(s1 + 0x14, f2);
    if f20 != 0.0 {
        let x = m.rf(s1 + 0x10) + f2 * m.rf(s3 + 4);
        let a = m.r32(s1);
        m.wf(s1 + 0x10, x);
        m.call(0x1f4bf8, &[a], &[]);
    }
    let o = m.r32(s1 + 0xc);
    if m.r32(o + 0xe4) != 0 {
        let r = m.call(0x1e6648, &[o], &[]).1[0];
        let x = m.rf(s1 + 0x10);
        let r = m.call(0x11d898, &[], &[fv(x), fv(r)]).1[0];
        m.wf(s1 + 0x10, r);
    }
    let f2 = f20.abs();
    if f2 < f32::from_bits(0x3c23d70a) {
        s2v = 1;
    } else {
        let lo = m.rf(s1 + 0x5c).abs();
        let hi = m.rf(s1 + 0x6c).abs();
        let mut f3 = (f2 - lo) / (hi - lo);
        if f3 < 0.0 { f3 = 0.0; } else if 1.0 < f3 { f3 = 1.0; }
        let o = m.r32(s1 + 0x70);
        m.wf(s1 + 0x60, f3);
        m.wf(s1 + 0x50, 1.0 - f3);
        m.call(0x1384c0, &[o], &[]);
    }
    m.call(0x19eab0, &[s1, s2v as u32], &[]);
    stack_pop(m, 0x50);
}

pub fn ee_sub_1728b0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let s6 = a0;
    let fv = |x: f32| -> V4 { [x, 0.0, 0.0, 0.0] };
    let len3 = |a: V4| (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
    let eps = f32::from_bits(0x38d1b717);
    let sp = stack_push(m, 0xef0);
    let (s3, s4, s5, s7) = (sp + 0x850, sp + 0xc50, sp + 0xcd0, sp + 0x20);
    let o = m.r32(s6 + 0x10);
    let org = m.call(0x1a62e8, &[o], &[]).1;
    let o = m.r32(s6 + 0x10);
    m.wv(sp, org);
    let f = m.call(0x204740, &[o], &[]).1[0];
    m.wf(sp + 0x10, f);
    m.w32(sp + 0x14, 0);
    // list 1: radius-offset points
    let mut s0 = m.r32(0x3e13fc);
    while s0 != 0 {
        let p = m.call(0x1a62e8, &[s0], &[]).1;
        let o = m.rv(sp);
        m.wv(sp + 0x830, [o[0] - p[0], o[1] - p[1], o[2] - p[2], o[3] - p[3]]);
        m.w32(sp + 0x838, 0);
        let d = m.rv(sp + 0x830);
        let rb = m.r32(s0 + 0x10c);
        let r = f32::from_bits(rb);
        let l = len3(d);
        m.wv(sp + 0x840, [r, f32::from_bits(if (rb as i32) < 0 { u32::MAX } else { 0 }), 0.0, 0.0]);
        let n = if l < eps { m.rv(0x2a2da0) } else { let q = 1.0 / l; [d[0] * q, d[1] * q, d[2] * q, d[3] * q] };
        let v = [p[0] + n[0] * r, p[1] + n[1] * r, p[2] + n[2] * r, p[3] + 1.0 + n[3] * r];
        m.wv(sp + 0x820, v);
        m.call(0x172798, &[sp], &[]);
        s0 = m.r32(s0 + 0x104);
    }
    let pos_list = |m: &mut M, head: u32, next: u32| {
        let mut s0 = m.r32(head);
        while s0 != 0 {
            let p = m.call(0x1a62e8, &[s0], &[]).1;
            m.wv(sp + 0x820, p);
            m.call(0x172798, &[sp], &[]);
            s0 = m.r32(s0 + next);
        }
    };
    let near_list = |m: &mut M, head: u32| {
        let mut s0 = m.r32(head);
        while s0 != 0 {
            let o = m.rv(sp);
            m.call(0x1e6f60, &[s0, 0, sp + 0x820, 0, 0, 0], &[o]);
            m.call(0x172798, &[sp], &[]);
            s0 = m.r32(s0 + 0x1b4);
        }
    };
    near_list(m, 0x3e13f0);
    pos_list(m, 0x3e1408, 0x104);
    pos_list(m, 0x3e13e4, 0x174);
    near_list(m, 0x3e13c0);
    pos_list(m, 0x3e13d8, 0x104);
    near_list(m, 0x3e13cc);
    let mut s0 = m.r32(0x3e13b4);
    while s0 != 0 {
        if m.r32(s0 + 0xe8) & 1 != 0 && m.call(0x132b10, &[s0, 0x13], &[]).0 != 0 {
            let p = m.call(0x1a62e8, &[s0], &[]).1;
            m.wv(sp + 0x820, p);
            m.call(0x172798, &[sp], &[]);
        }
        s0 = m.r32(s0 + 0xe0);
    }
    for (k, cap, buf) in [(0x23u32, 0x100u32, s3), (0x67, 0x20, s4), (0x1b, 0x80, s5)] {
        let n = m.call(0x15e9c8, &[0x3dd490, 0x505, k, 0, cap, buf], &[]).0 as i32;
        let mut p = buf;
        let mut c = n;
        if c > 0 {
            loop {
                let a = m.r32(p);
                c -= 1;
                p += 4;
                let v = m.call(0x1a62e8, &[a], &[]).1;
                m.wv(sp + 0x820, v);
                m.call(0x172798, &[sp], &[]);
                if c == 0 { break; }
            }
        }
    }
    pos_list(m, 0x3e1450, 0x174);
    let cnt = m.r32(sp + 0x14);
    m.call(0x2338c0, &[s7, cnt, 0x20, 0x172888], &[]);
    let mut f21 = 0.0f32;
    let mut s1: u32 = 0;
    let cnt = m.r32(sp + 0x14) as i32;
    if cnt > 0 {
        let o = m.rv(sp);
        let mut a3: u32 = 0;
        let mut a2: i32 = 0;
        let mut a0 = s7;
        let mut a1 = s6 + 0x50;
        loop {
            let e = m.rv(a0);
            let up = m.rv(0x2a2dc0);
            let d = [e[0] - o[0], e[1] - o[1], e[2] - o[2], e[3] - o[3]];
            let k = d[0] * up[0] + d[1] * up[1] + d[2] * up[2];
            let mut v = [d[0] - up[0] * k, d[1] - up[1] * k, d[2] - up[2] * k, d[3] - up[3] * k];
            let l = len3(v);
            if l < eps { v = m.rv(0x2a2da0); } else { let q = 1.0 / l; v = [v[0] * q, v[1] * q, v[2] * q, v[3] * q]; }
            let p = [o[0] + v[0] * 200.0, o[1] + v[1] * 200.0, o[2] + v[2] * 200.0, o[3] + 1.0 + v[3] * 200.0];
            m.wv(sp + 0x830, [200.0, 0.0, 0.0, 0.0]);
            let skip = if a3 != 0 {
                let q = m.rv(a3);
                let (dx, dy) = (p[0] - q[0], p[1] - q[1]);
                (dx * dx + dy * dy).sqrt() < 150.0
            } else { false };
            if !skip {
                a3 = a1;
                f21 = m.rf(a0 + 0x10);
                a1 += 0x20;
                s1 += 1;
                m.wv(a1 - 0x10, e);
                m.wv(a1 - 0x20, p);
            }
            a2 += 1;
            if a2 >= cnt { break; }
            a0 += 0x20;
            if s1 >= 4 { break; }
        }
    }
    let o = m.rv(sp);
    let f12 = m.rf(sp + 0x10) + f32::from_bits(0x40490fdb);
    m.wv(sp + 0xed0, o);
    let v = m.call(0x11e1a0, &[], &[fv(f12), fv(f32::from_bits(0xc3af0000)), fv(0.0)]).1;
    let base = [o[0] + v[0], o[1] + v[1], o[2] + v[2], o[3] + v[3]];
    m.wv(sp + 0xee0, base);
    if s1 < 4 {
        let mut s0 = s6 + 0x50 + (s1 << 5);
        loop {
            let (a, b) = (m.rf(0x2be5a8), m.rf(0x2be5ac));
            s1 += 1;
            let r1 = m.call(0x11d448, &[], &[fv(a), fv(b)]).1[0];
            f21 += r1;
            let f20 = f21 + m.rf(sp + 0x10);
            let (a, b) = (m.rf(0x2be5b0), m.rf(0x2be5b4));
            let r2 = m.call(0x11d448, &[], &[fv(a), fv(b)]).1[0];
            let v = m.call(0x11e1a0, &[], &[fv(f20), fv(r2), fv(0.0)]).1;
            let base = m.rv(sp + 0xee0);
            let w = [base[0] + v[0], base[1] + v[1], base[2] + v[2], base[3] + v[3]];
            m.wv(s0 + 0x10, w);
            m.wv(s0, w);
            s0 += 0x20;
            if s1 >= 4 { break; }
        }
    }
    m.w32(s6 + 0xd0, 0);
    let f = m.rf(0x2a53d0);
    m.wf(s6 + 0x3c, f);
    stack_pop(m, 0xef0);
}

fn sxw(bits: u32) -> f32 {
    if bits & 0x8000_0000 != 0 { f32::from_bits(0xffff_ffff) } else { 0.0 }
}
fn qf(x: f32) -> V4 {
    [x, sxw(x.to_bits()), 0.0, 0.0]
}
fn vsub4(a: V4, b: V4) -> V4 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2], a[3] - b[3]]
}
/// a + vf00 + b * s  (vadda ACC, a, vf00; vmaddx b, s)
fn vmadd1(a: V4, b: V4, s: f32) -> V4 {
    [a[0] + b[0] * s, a[1] + b[1] * s, a[2] + b[2] * s, a[3] + 1.0 + b[3] * s]
}
fn len3(v: V4) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}
/// rows r0..r3 * v (xyz, + r3), w kept from `keep`
fn mat4w(r: [V4; 4], v: V4, keep: f32) -> V4 {
    let mut o = [0.0; 4];
    for i in 0..3 {
        o[i] = r[0][i] * v[0] + r[1][i] * v[1] + r[2][i] * v[2] + r[3][i];
    }
    o[3] = keep;
    o
}
fn mat3w(r: [V4; 3], v: V4, keep: f32) -> V4 {
    let mut o = [0.0; 4];
    for i in 0..3 {
        o[i] = r[0][i] * v[0] + r[1][i] * v[1] + r[2][i] * v[2];
    }
    o[3] = keep;
    o
}
fn accept_197f98<M: QMem + ?Sized>(m: &mut M, sp: u32, f1: f32, f2: f32, alt: f32, f21: f32) -> bool {
    if !(f1 < f2) && alt != 0.0 {
        return true;
    }
    !(f1 * f21 < f2) && !(0.0 < m.rf(sp + 0x28))
}

/// FUN_00197f98: steering / target-acquire update (state +0x1c, phase +0x40)
pub fn ee_sub_197f98<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let s2 = a0;
    let sp = stack_push(m, 0x190);
    let tnow = 0x2dded0u32;
    let o0 = m.r32(s2);
    let s3 = m.r32(o0 + 0x150);
    let ph = m.r32(s2 + 0x40) as i32;
    let mut cond = false;
    if ph == 1 {
        let f1 = m.rf(s3 + 0x1c);
        if f1 < f32::from_bits(0x3f666666) {
            m.w32(s2 + 0x40, 2);
            let f2 = m.rf(0x2bf838);
            let t = m.rf(tnow);
            let f3 = m.rf(s2 + 0x48);
            m.wf(s2 + 0x4c, t);
            let t = m.rf(tnow);
            cond = t - f3 < f2;
        }
    } else if ph == 0 {
        let f1 = m.rf(s3 + 0x1c);
        if f32::from_bits(0x3f666666) < f1 {
            m.w32(s2 + 0x40, 1);
            let t = m.rf(tnow);
            m.wf(s2 + 0x48, t);
        } else {
            let f2 = m.rf(s2 + 0x44);
            let t = m.rf(tnow);
            let k = m.rf(0x2bf834);
            cond = k < t - f2;
        }
    } else if ph == 3 {
        let f1 = m.rf(s3 + 0x1c);
        cond = f32::from_bits(0x3e4ccccd) < f1;
    }
    if cond {
        m.w32(s2 + 0x24, 1);
    }
    let o0 = m.r32(s2);
    let org = m.call(0x1a62e8, &[o0], &[]).1;
    m.wv(sp + 0x10, org);
    let o0 = m.r32(s2);
    let vel = vslot(m, o0, 32, &[], &[]).1;
    m.wv(sp + 0x20, vel);
    let st = m.r32(s2 + 0x1c);
    let search = st >= 0x17 || matches!(st, 1 | 3 | 4 | 5 | 17);
    let mut f21: f32 = 0.0;
    let mut f20: f32 = 0.0;
    let mut f23: f32 = 0.0;
    if search {
        let mut s4: u32 = 0;
        let mut s6: u32 = 0;
        let mut f22 = f32::from_bits(0x44bb8000);
        let s1 = 0x2a2dd0u32;
        let mut s0 = m.r32(0x3e13e4);
        if s0 != 0 {
            loop {
                if m.r32(s0 + 0x17c) == 0 {
                    'body: {
                        let p = m.call(0x1a62e8, &[s0], &[]).1;
                        let o = m.rv(sp + 0x10);
                        let d = vsub4(p, o);
                        let d2 = d[0] * d[0] + d[1] * d[1];
                        if f32::from_bits(0x48afc800) < d2 {
                            break 'body;
                        }
                        let h = p[2] + m.rf(s0 + 0x158);
                        if h < m.rf(sp + 0x18) {
                            break 'body;
                        }
                        let ob = m.r32(s2);
                        let base = m.rv(ob + 0x240);
                        let a1 = m.rv(sp + 0x20);
                        let t0 = m.rv(s1);
                        m.wv(sp + 0x180, p);
                        m.call(0x19a4a0, &[s2, s0, sp + 0x120, sp + 0x124, sp + 0x30, sp + 0x50], &[a1, vsub4(p, base), t0]);
                        if f22 < m.rf(sp + 0x124) {
                            break 'body;
                        }
                        let p = m.rv(sp + 0x180);
                        let o = m.rv(sp + 0x10);
                        let dd = vsub4(o, p);
                        f21 = f32::from_bits(0x3e800000);
                        m.wv(sp + 0x80, dd);
                        let v = m.rv(sp + 0x20);
                        f20 = 0.0;
                        m.wv(sp + 0x180, v);
                        let w = vslot(m, s0, 32, &[], &[]).1;
                        let f0 = f21.min(m.rf(sp + 0x120));
                        m.wf(sp + 0x88, f20);
                        let v = m.rv(sp + 0x180);
                        let dv = vsub4(v, w);
                        let r = vmadd1(dd, dv, f0);
                        let a0q = m.rv(s1);
                        let a1q = m.rv(sp + 0x80);
                        m.wv(sp + 0xa0, r);
                        m.wf(sp + 0xa8, f20);
                        m.wv(sp + 0x90, dv);
                        let a2q = m.rv(sp + 0xa0);
                        m.wv(sp + 0xb0, qf(f0));
                        m.call(0x11ee08, &[0, sp + 0x128, sp + 0x12c], &[a0q, a1q, a2q]);
                        let f2 = m.rf(sp + 0x12c);
                        let f1 = m.rf(0x2bf83c);
                        let alt = m.rf(sp + 0x128);
                        if accept_197f98(m, sp, f1, f2, alt, f21) {
                            s4 = s0;
                            let v1 = m.rv(sp + 0x30);
                            let v0 = m.rv(sp + 0x50);
                            s6 = 0;
                            m.wv(sp + 0x40, v1);
                            m.wv(sp + 0x60, v0);
                            f23 = m.rf(sp + 0x120);
                            f22 = m.rf(sp + 0x124);
                        }
                    }
                }
                s0 = m.r32(s0 + 0x174);
                if s0 == 0 {
                    break;
                }
            }
        }
        let mut s0 = m.r32(0x3e1420);
        while s0 != 0 {
            'body2: {
                let p = m.call(0x1a62e8, &[s0], &[]).1;
                let o = m.rv(sp + 0x10);
                let d = vsub4(o, p);
                m.wv(sp + 0x80, p);
                let lim = m.rf(0x2bf840);
                let d2 = d[0] * d[0] + d[1] * d[1];
                if lim * lim < d2 {
                    break 'body2;
                }
                let ob = m.r32(s2);
                let base = m.rv(ob + 0x240);
                let a1 = m.rv(sp + 0x20);
                let t0 = m.rv(s1);
                m.call(0x19a4a0, &[s2, s0, sp + 0x120, sp + 0x124, sp + 0x30, sp + 0x50], &[a1, vsub4(p, base), t0]);
                if f22 < m.rf(sp + 0x124) {
                    break 'body2;
                }
                f21 = f32::from_bits(0x3e800000);
                f20 = 0.0;
                let g = vslot(m, s0, 31, &[0], &[]).0;
                let p = m.rv(sp + 0x80);
                let r = m.call(0x22d348, &[g, 0], &[p]).1;
                let f0 = f21.min(m.rf(sp + 0x120));
                let o = m.rv(sp + 0x10);
                let v = m.rv(sp + 0x20);
                let dd = vsub4(o, p);
                let dv = vsub4(v, r);
                m.wv(sp + 0xa0, dd);
                let q = vmadd1(dd, dv, f0);
                m.wf(sp + 0xa8, f20);
                m.wv(sp + 0xc0, q);
                m.wf(sp + 0xc8, f20);
                let a1q = m.rv(sp + 0xa0);
                let a0q = m.rv(s1);
                let a2q = m.rv(sp + 0xc0);
                m.wv(sp + 0x90, r);
                m.wv(sp + 0xb0, dv);
                m.wv(sp + 0xd0, qf(f0));
                m.call(0x11ee08, &[0, sp + 0x130, sp + 0x134], &[a0q, a1q, a2q]);
                let f2 = m.rf(sp + 0x134);
                let f1 = m.rf(s0 + 0xe8);
                let alt = m.rf(sp + 0x130);
                if accept_197f98(m, sp, f1, f2, alt, f21) {
                    s6 = s0;
                    let v1 = m.rv(sp + 0x30);
                    let v0 = m.rv(sp + 0x50);
                    s4 = 0;
                    m.wv(sp + 0x40, v1);
                    m.wv(sp + 0x60, v0);
                    f23 = m.rf(sp + 0x120);
                    f22 = m.rf(sp + 0x124);
                }
            }
            s0 = m.r32(s0 + 0xe0);
        }
        if s4 != 0 || s6 != 0 {
            let ob = m.r32(s2);
            let t = m.rf(tnow);
            let s0 = m.r32(ob + 0x58);
            let a1 = m.r32(s0 + 0xa0);
            m.wf(s0 + 0xa8, t + f23);
            let q = m.rv(sp + 0x40);
            let r = m.call(0x22fc48, &[0, a1], &[q]).1;
            m.wv(s0 + 0xc0, r);
            let a1 = m.r32(s0 + 0xa0);
            let ob = m.r32(s2);
            let f1 = m.rf(sp + 0x28);
            let e8 = m.r32(ob + 0xe8);
            let k = m.rf(e8 + 0x88);
            let f1 = f1 + f23 * k;
            let x = m.rf(sp + 0x60);
            let y = m.rf(sp + 0x64);
            let a2 = [x, y, f1, sxw(f1.to_bits())];
            let r = m.call(0x22fcc8, &[0, a1], &[a2]).1;
            m.wv(sp + 0x80, a2);
            m.wv(s0 + 0xd0, r);
            if s4 != 0 {
                m.call(0x195fb8, &[s2, 6, s4], &[]);
            } else {
                m.call(0x195fb8, &[s2, 8, s6], &[]);
            }
        }
    }
    // 0x1985bc
    let ob = m.r32(s2);
    let st = m.r32(s2 + 0x1c);
    let h = m.rf(ob + 0x1a8);
    m.wf(sp + 0x138, h);
    let idx = st.wrapping_sub(6);
    if idx < 0xd {
        // jump table 0x2a64d0: 0 = 0x1985f0, 1 = 0x19878c, 2 = vfn(+0x190), 3 = vfn(+0xe0), 4 = 0x19877c
        let k = match idx {
            0 | 2 => 0,
            4 | 5 => 2,
            3 | 6 | 7 => 3,
            12 => 4,
            _ => 1,
        };
        if k != 1 {
            let a2 = match k {
                0 => {
                    let c = m.r32(s2 + 0xc);
                    let p = m.call(0x1a62e8, &[c], &[]).1;
                    let ob = m.r32(s2);
                    let b = m.rv(ob + 0x240);
                    m.wv(sp + 0x30, vsub4(p, b));
                    m.r32(s2 + 0xc)
                }
                4 => {
                    let c = m.r32(s2 + 0xc);
                    let p = m.call(0x1a62e8, &[c], &[]).1;
                    m.wv(sp + 0x30, p);
                    m.r32(s2 + 0xc)
                }
                _ => {
                    let v1 = m.r32(s2 + 0xc);
                    let off = if k == 2 { 0x190 } else { 0xe0 };
                    let t0 = if v1 != 0 { v1.wrapping_add(off) } else { 0 };
                    let vv = m.r32(t0 + 0x20);
                    let adj = m.r16(vv + 0x18) as i16 as i32 as u32;
                    let f = m.r32(vv + 0x1c);
                    let a1 = m.r32(s2);
                    let f12 = m.rf(s2 + 0x14);
                    m.call(f, &[t0.wrapping_add(adj), a1, sp + 0x30, sp + 0x138], &[f4(f12)]);
                    m.r32(s2 + 0xc)
                }
            };
            // 0x198610
            let ob = m.r32(s2);
            let t = m.rf(tnow);
            let s1 = m.r32(ob + 0x58);
            f20 = m.rf(s1 + 0xa8) - t;
            let s0 = vslot(m, a2, 31, &[0], &[]).0;
            m.call(0x2309e8, &[s0, 0, sp + 0x80, sp + 0xe0], &[f4(f20)]);
            let q = m.rv(sp + 0x30);
            let r = m.call(0x22fc48, &[0, s0], &[q]).1;
            m.wv(sp + 0x150, r);
            let rows = [m.rv(sp + 0x80), m.rv(sp + 0x90), m.rv(sp + 0xa0), m.rv(sp + 0xb0)];
            let v5 = mat4w(rows, r, r[3]);
            let a1 = m.r32(s1 + 0xa0);
            let r2 = m.call(0x22fc48, &[0, a1], &[v5]).1;
            m.wv(s1 + 0xc0, r2);
            if m.r32(s2 + 0x1c) != 10 {
                let rows = [m.rv(sp + 0xe0), m.rv(sp + 0xf0), m.rv(sp + 0x100), m.rv(sp + 0x110)];
                let r = m.rv(sp + 0x150);
                let v2 = mat4w(rows, r, rows[0][3]);
                m.wv(sp + 0xc0, v2);
                let ob = m.r32(s2);
                let f1 = m.rf(sp + 0x28);
                let e8 = m.r32(ob + 0xe8);
                let k = m.rf(e8 + 0x88);
                m.wf(sp + 0xc8, f1 + f20 * k);
                let a1 = m.r32(s1 + 0xa0);
                let q = m.rv(sp + 0xc0);
                let r3 = m.call(0x22fcc8, &[0, a1], &[q]).1;
                m.wv(s1 + 0xd0, r3);
            }
        }
    }
    // 0x19878c
    let v1 = m.r32(s2 + 0x1c) as i32;
    if v1 == 6 || (8..=16).contains(&v1) || v1 == 0x12 {
        let ob = m.r32(s2);
        let t = m.rf(tnow);
        let s0 = m.r32(ob + 0x58);
        let t4 = m.rf(tnow + 4);
        f20 = m.rf(s0 + 0xa8) - t + t4;
        f21 = 1.0;
        let (q0, q1, q2, q3) = (m.rv(s0 + 0x30), m.rv(s0 + 0xb0), m.rv(s0 + 0xc0), m.rv(s0 + 0xd0));
        m.call(0x117120, &[0, 0, sp + 0x40], &[q0, q1, q2, q3, f4(f20), f4(0.0), f4(f21)]);
        let (q0, q1, q2, q3) = (m.rv(s0 + 0x30), m.rv(s0 + 0xb0), m.rv(s0 + 0xc0), m.rv(s0 + 0xd0));
        m.call(0x117120, &[0, 0, sp + 0x50], &[q0, q1, q2, q3, f4(f20), f4(f20), f4(f21)]);
        let l1 = len3(m.rv(sp + 0x40));
        let l2 = len3(m.rv(sp + 0x50));
        let lim = m.rf(0x2bf844);
        f20 *= 0.5;
        if lim < (l1 + l2) * f20 {
            m.call(0x195fb8, &[s2, 4, 0], &[]);
        }
    }
    // 0x1988d0
    let v0 = m.r32(s2 + 8);
    f21 = m.rf(0x2bf780u32.wrapping_add(v0 << 2));
    let a0o = m.r32(s2);
    let st = m.r32(s2 + 0x1c);
    f20 = m.rf(a0o + 0x1a8);
    // jump table 0x2a6510
    match st {
        0 => {
            let f1 = m.rf(s3 + 0x1c);
            if f32::from_bits(0x3e4ccccd) < f1 {
                f20 = m.rf(s3 + 0x18) + f21;
            }
        }
        11 | 12 | 13 => f20 = m.rf(sp + 0x138),
        17 | 19 => {}
        20 => {
            f23 = 1.0;
            let ob = m.r32(s2);
            let f24 = m.call(0x204740, &[ob], &[]).1[0];
            let a = m.rf(s3 + 0x18) - f24;
            let f0 = m.call(0x11cf60, &[], &[f4(a)]).1[0];
            m.call(0x11d608, &[sp + 0x13c, sp + 0x140], &[f4(f0)]);
            let f3 = m.rf(s3 + 0x1c);
            let f22 = m.rf(sp + 0x140) * f3;
            let f21b = m.rf(sp + 0x13c) * f3;
            let fz = (f22 + f23) * 0.5;
            let a0 = m.r32(s2 + 0x1bc);
            if a0 != 0 {
                let f1 = (f23 - fz) * m.rf(0x2bf798);
                let f13 = fz * m.rf(0x2bf79c);
                let s0 = m.r32(a0 + 0x114);
                let f14 = m.rf(tnow + 4);
                let f12 = m.rf(s0 + 4);
                let r = m.call(0x11d018, &[0x2bf860, 0], &[f4(f12), f4(f1 + f13), f4(f14)]).1[0];
                m.wf(s0 + 4, r);
            }
            let f2 = f23 - fz;
            let c0 = f2 * m.rf(0x2bf850) + fz * m.rf(0x2bf854);
            let f1 = f2 * m.rf(0x2bf870) + fz * m.rf(0x2bf874);
            let f5 = m.rf(tnow + 4);
            m.wf(s2 + 0x1c0, c0);
            let c4 = f2 * m.rf(0x2bf858) + fz * m.rf(0x2bf85c);
            let f1 = f1 * f21b * f5;
            m.wf(s2 + 0x1c4, c4);
            let g = m.rf(s3 + 0x1c);
            m.wf(s2 + 0x17c, f23 - g);
            let f2 = m.rf(s3 + 0x1c);
            f20 = f24 + f1;
            if f2 == 0.0 {
                m.wf(s2 + 0x1ac, 0.0);
                m.wf(s2 + 0x188, 0.0);
                m.wf(s2 + 0x194, 0.0);
                m.wf(s2 + 0x1a0, 0.0);
            } else {
                let f1 = f2 / (f21b.abs() + f22.abs());
                if f21b < 0.0 {
                    m.wf(s2 + 0x188, 0.0);
                    m.wf(s2 + 0x194, -f21b * f1);
                } else {
                    m.wf(s2 + 0x194, 0.0);
                    m.wf(s2 + 0x188, f21b * f1);
                }
                if 0.0 < f22 {
                    m.wf(s2 + 0x1ac, 0.0);
                    m.wf(s2 + 0x1a0, f22 * f1);
                } else {
                    m.wf(s2 + 0x1a0, 0.0);
                    m.wf(s2 + 0x1ac, -f22 * f1);
                }
            }
            let b8 = m.r32(s2 + 0x1b8);
            m.call(0x1384c0, &[b8], &[]);
        }
        14 | 15 => {
            if st == 0xf {
                let c = m.r32(s2 + 0xc);
                let f12 = m.rf(s2 + 0x10);
                m.call(0x1e67b8, &[c, sp + 0x40, 0], &[f4(f12)]);
            } else {
                let c = m.r32(s2 + 0xc);
                let p = m.call(0x1a62e8, &[c], &[]).1;
                m.wv(sp + 0x40, p);
            }
            let a0 = m.r32(s2);
            if a0 != m.r32(0x2e1e40) {
                m.call(0x1cc2e8, &[a0.wrapping_add(0x900)], &[]);
            }
            let d = vsub4(m.rv(sp + 0x40), m.rv(sp + 0x10));
            let a0 = m.r32(s2);
            m.call(0x2046b0, &[a0], &[d]);
            let ob = m.r32(s2);
            f20 = m.rf(ob + 0x1a8);
        }
        22 => {
            let a0 = m.r32(s2);
            let k = m.rf(0x2bf878);
            m.call(0x1441e8, &[a0], &[f4(k)]);
            let f12 = m.rf(s3 + 0x18) + f32::from_bits(0x40490fdb);
            let f13 = m.rf(s3 + 0x1c) * m.rf(0x2bf87c);
            let f14 = m.rf(0x2bf880);
            let s0 = m.r32(s2);
            let v = m.call(0x11e1a0, &[], &[f4(f12), f4(f13), f4(f14)]).1;
            m.wv(s0 + 0x190, v);
            let vv = m.rv(sp + 0x20);
            let l = (vv[0] * vv[0] + vv[1] * vv[1]).sqrt();
            if m.rf(0x2bf848) < l {
                let (y, x) = (m.rf(sp + 0x24), m.rf(sp + 0x20));
                let hh = m.call(0x2458f0, &[], &[f4(y), f4(x)]).1[0] + f21;
                let ob = m.r32(s2);
                m.wf(ob + 0x1a8, hh);
            }
            m.wv(sp + 0x80, qf(0.5));
            m.wv(sp + 0x80, qf(-0.5));
            let s1 = 0x2a2dc0u32;
            let v4 = m.rv(sp + 0x20);
            let u = m.rv(s1);
            let ob = m.r32(s2);
            let v5 = m.rv(ob + 0x190);
            let d4 = v4[0] * u[0] + v4[1] * u[1] + v4[2] * u[2];
            let d5 = v5[0] * u[0] + v5[1] * u[1] + v5[2] * u[2];
            let p4 = vsub4(v4, [u[0] * d4, u[1] * d4, u[2] * d4, u[3] * d4]);
            let p5 = vsub4(v5, [u[0] * d5, u[1] * d5, u[2] * d5, u[3] * d5]);
            let c = m.rv(0x2bf890);
            let a2 = [
                c[0] + p4[0] * 0.5 + p5[0] * -0.5,
                c[1] + p4[1] * 0.5 + p5[1] * -0.5,
                c[2] + p4[2] * 0.5 + p5[2] * -0.5,
                c[3] + 1.0 + p4[3] * 0.5 + p5[3] * -0.5,
            ];
            m.call(0x11ba08, &[sp + 0x80], &[u, a2]);
            let a1 = m.rv(sp + 0x80);
            let v0 = m.rv(sp + 0x90);
            let v1 = m.rv(sp + 0xa0);
            m.wv(sp + 0x50, v0);
            m.wv(sp + 0x60, v1);
            m.wv(sp + 0x40, a1);
            let u = m.rv(s1);
            let s0 = m.r32(s2);
            let f12 = m.rf(s0 + 0x1a8);
            m.call(0x11b500, &[sp + 0xb0], &[u, f4(f12)]);
            let rows = [m.rv(sp + 0x40), m.rv(sp + 0x50), m.rv(sp + 0x60)];
            let b5 = m.rv(sp + 0xb0);
            let b3 = m.rv(sp + 0xc0);
            let b6 = m.rv(sp + 0xd0);
            let v5 = mat3w(rows, b5, b5[3]);
            let v3 = mat3w(rows, b3, b3[3]);
            let v4 = mat3w(rows, b6, rows[0][3]);
            m.wv(sp + 0x90, v3);
            m.wv(sp + 0xa0, v4);
            m.wv(sp + 0xe0, v5);
            m.wv(sp + 0x80, v5);
            m.wv(sp + 0xf0, v3);
            m.wv(sp + 0x100, v4);
            m.wv(s0 + 0x1b0, v5);
            let q = m.rv(sp + 0x90);
            m.wv(s0 + 0x1c0, q);
            let q = m.rv(sp + 0xa0);
            m.wv(s0 + 0x1d0, q);
        }
        _ => {
            let r = vslot(m, a0o, 91, &[], &[]).0;
            if r != 0 {
                let ob = m.r32(s2);
                f20 = m.rf(ob + 0x1a8);
            } else {
                let vv = m.rv(sp + 0x20);
                let l = (vv[0] * vv[0] + vv[1] * vv[1]).sqrt();
                if m.rf(0x2bf848) < l {
                    let (y, x) = (m.rf(sp + 0x24), m.rf(sp + 0x20));
                    f20 = m.call(0x2458f0, &[], &[f4(y), f4(x)]).1[0] + f21;
                }
            }
        }
    }
    // 0x198dcc
    if m.r32(s2 + 0x1c) != 0x16 {
        let a0 = m.r32(s2);
        m.call(0x204660, &[a0], &[f4(f20)]);
    }
    let v1 = m.r32(s2 + 0x1c) as i32;
    if v1 >= 0 && (v1 < 6 || v1 == 7) {
        if v1 == 5 {
            let f1 = m.rf(s3 + 0x1c);
            if f32::from_bits(0x3e4ccccd) < f1 {
                let a = m.rf(s3 + 0x18) - m.rf(s2 + 0xb0);
                let mut f0 = m.call(0x244e10, &[], &[f4(a)]).1[0];
                let f1 = m.rf(s2 + 0xb4);
                if f0 < 0.0 {
                    f0 = 0.0;
                } else if f1 < f0 {
                    f0 = f1;
                }
                m.wf(s2 + 0xb4, f0);
            }
            let f2 = m.rf(s2 + 0xb8);
            let f12 = m.rf(0x2bf730);
            f21 = f12;
            if f12 < f2 {
                let f0 = m.rf(s2 + 0xb4);
                f21 = (1.0 - f0) * f12 + f0 * f2;
            }
        } else if (2..4).contains(&v1) {
            f21 = m.rf(0x2bf734);
        } else {
            f21 = m.rf(0x2bf730);
        }
        // 0x198ee4
        f21 *= m.rf(s2 + 0x38);
        let f13 = m.rf(s3 + 0x1c) * f21;
        let f12 = m.rf(s3 + 0x18);
        f20 = 0.0;
        let v = m.call(0x11e1a0, &[], &[f4(f12), f4(f13), f4(f20)]).1;
        m.wv(sp + 0x40, v);
        let ob = m.r32(s2);
        let r0 = m.rv(ob + 0x1b0);
        let r1 = m.rv(ob + 0x1c0);
        let r2 = m.rv(ob + 0x1d0);
        m.wv(sp + 0x50, qf(-1.0));
        let x = r0[0] * v[0] + r0[1] * v[1] + r0[2] * v[2];
        let y = r1[1] * v[1] + r1[0] * v[0] + r1[2] * v[2];
        let z = r2[2] * v[2] + r2[0] * v[0] + r2[1] * v[1];
        m.wv(ob + 0x190, [-x, -y, -z, -r0[3]]);
        let go = if m.r32(s2 + 0x1c) != 0 || m.r32(s2 + 0x40) == 3 {
            m.r32(s2 + 0x24) == 0
        } else {
            false
        };
        if go {
            let ob = m.r32(s2);
            let w = vslot(m, ob, 32, &[], &[]).1;
            m.wv(sp + 0x50, w);
            m.wf(sp + 0x58, f20);
            let v3 = m.rv(sp + 0x50);
            let l = len3(v3);
            if l < f32::from_bits(0x38d1b717) {
                let q = m.rv(0x2a2da0);
                m.wv(sp + 0x50, q);
            } else {
                let q = 1.0 / l;
                m.wv(sp + 0x50, [v3[0] * q, v3[1] * q, v3[2] * q, v3[3] * q]);
            }
            f20 = l;
            let a0q = m.rv(sp + 0x40);
            let a1q = m.rv(sp + 0x50);
            let r = m.call(0x11e290, &[0, sp + 0x60], &[a0q, a1q]).1[0];
            if r < f20 && f20 * f32::from_bits(0xbe800000) < r {
                let p = m.rv(sp + 0x60);
                let d = m.rv(sp + 0x50);
                let q = vmadd1(p, d, f20);
                m.wv(sp + 0x80, qf(f20));
                m.wv(sp + 0x40, q);
                let v = m.call(0x11e9f8, &[], &[q, f4(f21)]).1;
                m.wv(sp + 0x40, v);
            }
        }
        let a1 = m.r32(s2 + 0x160);
        if a1 != 0 {
            let v0 = m.rv(sp + 0x40);
            let a0 = m.r32(s2 + 0x164);
            m.wv(sp + 0x170, [f32::from_bits(a0), sxw(a0), 0.0, 0.0]);
            m.wv(sp + 0x160, v0);
            let w = vslot(m, a1, 32, &[], &[]).1;
            let q = m.rv(sp + 0x170);
            m.wv(sp + 0x50, q);
            let b = m.rv(sp + 0x160);
            let s = m.rf(sp + 0x170);
            m.wv(sp + 0x40, vmadd1(b, w, s));
        }
        let q = m.rv(sp + 0x40);
        m.wv(s2 + 0xa0, q);
    }
    // 0x1990f0
    let f1 = m.rf(sp + 0x18);
    if m.rf(s2 + 0x3c) < f1 {
        m.wf(s2 + 0x3c, f1);
    }
    if m.r32(s2 + 0x174) != 0 {
        let v0 = m.r32(s2 + 0x16c);
        if v0 != 0 && m.r32(s2 + 0x170) != 0 && m.r32(v0 + 0x134) == 1 {
            let f1 = m.rf(sp + 0x28);
            if f1 < m.rf(0x2bf8a0) {
                let a0 = m.r32(s2);
                m.call(0x140c88, &[a0], &[]);
            }
        }
    }
    stack_pop(m, 0x190);
}
