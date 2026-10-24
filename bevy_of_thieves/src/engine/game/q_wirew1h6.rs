//! WIRE-W1H6: 1:1 queue ports (against `QMem`) of a batch of live-frontier natives, ported from the EE asm;
//! callees go through `m.call` / `vslot`. QMem convention: integer / pointer args in `a`, quadword and float
//! args in `v`.
#![allow(unused_variables, unused_mut, unused_assignments, clippy::all)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

fn w64<M: QMem + ?Sized>(m: &mut M, a: u32, v: u64) {
    m.w32(a, v as u32);
    m.w32(a + 4, (v >> 32) as u32);
}

/// FUN_00187e78(o): +0x1028 = +0x1024 = 0.
pub fn ee_sub_187e78<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.w32(a0 + 0x1028, 0);
    m.w32(a0 + 0x1024, 0);
}

/// FUN_001f7588(_, p): p+0x44 != 0.
pub fn ee_sub_1f7588<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    (m.r32(a1 + 0x44) != 0) as u32
}

/// FUN_0013b5c8(o, f): +0x154 = min(k + f, +0x154).
pub fn ee_sub_13b5c8<M: QMem + ?Sized>(m: &mut M, a0: u32, f12: f32) {
    let cur = m.rf(a0 + 0x154);
    let k = m.rf(0x002d_ded0);
    m.wf(a0 + 0x154, (k + f12).min(cur));
}

/// FUN_001fbee0(o): FUN_002323a8(o+0x40).
pub fn ee_sub_1fbee0<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let p = m.r32(a0 + 0x40);
    m.call(0x0023_23a8, &[p], &[]).0
}

/// FUN_00229710(o, n): push n onto list +0x138 (link at n+8); FUN_001f5078(o, n).
pub fn ee_sub_229710<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let h = m.r32(a0 + 0x138);
    m.w32(a1 + 8, h);
    m.w32(a0 + 0x138, a1);
    m.call(0x001f_5078, &[a0, a1], &[]).0
}

/// FUN_001a6318(o): virtual slot 30 (vt+0xfc).
pub fn ee_sub_1a6318<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    vslot(m, a0, 30, &[], &[]).0
}

/// FUN_001a34b0(o): FUN_001a5848(o); FUN_001a3b38(o).
pub fn ee_sub_1a34b0<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    m.call(0x001a_5848, &[a0], &[]);
    m.call(0x001a_3b38, &[a0], &[]).0
}

/// FUN_00211d08(o): FUN_001a5270(o); +0xb0 = 1.
pub fn ee_sub_211d08<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5270, &[a0], &[]);
    m.w32(a0 + 0xb0, 1);
}

/// FUN_0011f2b8(n, p): vmax.xyz over n quadwords starting at p, seeded with the constant at 0x2a2e00.
pub fn ee_sub_11f2b8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> V4 {
    let mut r = m.rv(0x002a_2e00);
    let mut n = a0 as i32;
    let mut p = a1;
    if n > 0 {
        loop {
            let v = m.rv(p);
            n = n.wrapping_sub(1);
            for i in 0..3 {
                r[i] = v[i].max(r[i]);
            }
            p = p.wrapping_add(0x10);
            if n == 0 {
                break;
            }
        }
    }
    r
}

/// FUN_0019c9b8(o): FUN_00231100(o); +0xe8 = 100.
pub fn ee_sub_19c9b8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0023_1100, &[a0], &[]);
    m.wf(a0 + 0xe8, 100.0);
}

/// FUN_001ca788(o): FUN_00231100(o); FUN_001a6ca8(o, 0).
pub fn ee_sub_1ca788<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0023_1100, &[a0], &[]);
    m.call(0x001a_6ca8, &[a0, 0], &[]);
}

/// FUN_00182e80(o): FUN_001e5d80(o); FUN_00182c70(o+0x190); +0x1bc = 3.
pub fn ee_sub_182e80<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001e_5d80, &[a0], &[]);
    m.call(0x0018_2c70, &[a0 + 0x190], &[]);
    m.w32(a0 + 0x1bc, 3);
}

/// FUN_00182eb8(o): FUN_001a6178(o); FUN_0011a220(0x3e13f0, o).
pub fn ee_sub_182eb8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_6178, &[a0], &[]);
    m.call(0x0011_a220, &[0x003e_13f0, a0], &[]);
}

/// FUN_001badb8(o): FUN_001f3ea8(o); FUN_0011a220(0x3e1450, o).
pub fn ee_sub_1badb8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001f_3ea8, &[a0], &[]);
    m.call(0x0011_a220, &[0x003e_1450, a0], &[]);
}

/// FUN_001320c8(o): FUN_001f2108(o); +0x148 |= 1<<34.
pub fn ee_sub_1320c8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001f_2108, &[a0], &[]);
    let v = m.r64(a0 + 0x148) | (0x8000u64 << 0x13);
    w64(m, a0 + 0x148, v);
}

/// FUN_001da578(o, k)
pub fn ee_sub_1da578<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if a1 == u32::MAX {
        m.w32(a0 + 0x20, u32::MAX);
        m.w32(a0 + 0x1c, 0);
        return;
    }
    if a1 == m.r32(a0 + 0x18) {
        m.w32(a0 + 0x1c, 0);
        return;
    }
    if a1 == m.r32(a0 + 0x20) {
        m.w32(a0 + 0x20, u32::MAX);
    }
}

/// FUN_00141928(o)
pub fn ee_sub_141928<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001f_2d48, &[a0], &[]);
    m.call(0x0013_0658, &[a0 + 0xefc], &[]);
    m.call(0x0013_0658, &[a0 + 0xf14], &[]);
    m.call(0x001e_ecd0, &[a0 + 0x26c], &[]);
}

/// FUN_00143ff8(o): if FUN_00144038(o) -> virtual slot 87 (vt+0x2c4)(0).
pub fn ee_sub_143ff8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    if m.call(0x0014_4038, &[a0], &[]).0 != 0 {
        vslot(m, a0, 87, &[0], &[]);
    }
}

/// FUN_001b4438(o): FUN_001f2108(o); +0x148 &= !(1<<35).
pub fn ee_sub_1b4438<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001f_2108, &[a0], &[]);
    let v = m.r64(a0 + 0x148) & 0xffff_fff7_ffff_ffff;
    w64(m, a0 + 0x148, v);
}

/// FUN_00183f08(o)
pub fn ee_sub_183f08<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0023_1100, &[a0], &[]);
    m.call(0x0018_2c70, &[a0 + 0xe0], &[]);
    m.w32(a0 + 0xf0, 1);
    m.wf(a0 + 0x110, 200.0);
    m.w32(a0 + 0xfc, 0);
}

/// FUN_001cb908(o)
pub fn ee_sub_1cb908<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001e_5d80, &[a0], &[]);
    m.call(0x0018_2c70, &[a0 + 0x190], &[]);
    m.wf(a0 + 0x1c4, 1.0);
    m.wf(a0 + 0x1c0, 1000.0);
}

/// FUN_001bea40(o)
pub fn ee_sub_1bea40<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0023_1100, &[a0], &[]);
    m.wf(a0 + 0x11c, 1.0);
    m.wf(a0 + 0x124, 1000.0);
    m.wf(a0 + 0x120, 4.0);
}

/// FUN_00224190(o): if 0 < +0x150 < 3 -> virtual slot 75 (vt+0x264)(3); return 1.
pub fn ee_sub_224190<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let k = m.r32(a0 + 0x150) as i32;
    if k < 3 && k > 0 {
        vslot(m, a0, 75, &[3], &[]);
    }
    1
}

/// FUN_00144338(o, a1, a2): FUN_00205588(o, a1, a2); if a2 && --(+0x12f4) == 0 -> FUN_00143ff8(o).
pub fn ee_sub_144338<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    m.call(0x0020_5588, &[a0, a1, a2], &[]);
    if a2 != 0 {
        let n = m.r32(a0 + 0x12f4).wrapping_sub(1);
        m.w32(a0 + 0x12f4, n);
        if n == 0 {
            m.call(0x0014_3ff8, &[a0], &[]);
        }
    }
}

/// FUN_001ff3e8(o): FUN_001a5c48(o); if +0x38 == sentinel -> +0x38 = k.
pub fn ee_sub_1ff3e8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5c48, &[a0], &[]);
    if m.rf(a0 + 0x38) == f32::from_bits(0x5015_02f9) {
        let k = m.rf(0x002c_3c58);
        m.wf(a0 + 0x38, k);
    }
}

/// FUN_001c5df0(o, p): if +0x1c8 && (p+0x44 & 8) -> virtual slot 75 (vt+0x264)(1), return 1.
pub fn ee_sub_1c5df0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    if m.r32(a0 + 0x1c8) == 0 {
        return 0;
    }
    if m.r32(a1 + 0x44) & 8 == 0 {
        return 0;
    }
    vslot(m, a0, 75, &[1], &[]);
    1
}

/// FUN_001564e8(o)
pub fn ee_sub_1564e8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5270, &[a0], &[]);
    let f = m.rf(0x002b_dcd0);
    m.wf(a0 + 0x54, f);
    let lo = m.r32(0x002b_dcd8);
    let hi = m.r32(0x002b_dcdc);
    m.w32(a0 + 0x58, lo);
    m.w32(a0 + 0x5c, hi);
    m.wf(a0 + 0x60, 0.25);
    let v = m.r32(a0 + 0x40) | 0x40;
    m.w32(a0 + 0x40, v);
}

/// FUN_001c3458(o): FUN_001a5c48(o); if +0x38 == sentinel -> +0x38 = FUN_001e7d90(o) + k.
pub fn ee_sub_1c3458<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5c48, &[a0], &[]);
    if m.rf(a0 + 0x38) == f32::from_bits(0x5015_02f9) {
        let f = m.call(0x001e_7d90, &[a0], &[]).1[0];
        let k = m.rf(0x002c_0d00);
        m.wf(a0 + 0x38, f + k);
    }
}

/// FUN_001f69f8(o, id): flag every 0x28-byte entry of +0xc0 (count +0xb8) whose id matches.
pub fn ee_sub_1f69f8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if (m.r32(a0 + 0xb8) as i32) <= 0 {
        return;
    }
    let mut i: i32 = 0;
    let mut off: u32 = 0;
    loop {
        let e = m.r32(a0 + 0xc0).wrapping_add(off);
        if m.r32(e) == a1 {
            let v = m.r32(e + 0xc) | 0x800;
            m.w32(e + 0xc, v);
            let p = m.r32(e + 0x24);
            if p != 0 {
                let v = m.r64(p + 0x10) | (0x8000u64 << 35);
                w64(m, p + 0x10, v);
            }
        }
        i += 1;
        off = off.wrapping_add(0x28);
        if i >= m.r32(a0 + 0xb8) as i32 {
            break;
        }
    }
}

/// FUN_001c6ab8(o)
pub fn ee_sub_1c6ab8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001c_4e50, &[a0], &[]);
    let f3 = f32::from_bits(0x420c_0000);
    let f4v = f32::from_bits(0x3f19_999a);
    m.wf(a0 + 0x1dc, f3);
    m.wf(a0 + 0x1a0, 0.75);
    m.wf(a0 + 0x1d0, 500.0);
    m.wf(a0 + 0x1d4, 250.0);
    m.wf(a0 + 0x1e0, f4v);
    m.wf(a0 + 0x1d8, f3);
    m.wf(a0 + 0x1e4, f4v);
}

/// FUN_00177e80(o)
pub fn ee_sub_177e80<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001e_ef68, &[a0], &[]);
    let f2 = m.rf(0x002b_e62c);
    let f3 = m.rf(0x002b_e624);
    let f1 = m.rf(0x002b_e628);
    m.w32(a0 + 0xa0, 1);
    m.wf(a0 + 0x34, -1.0);
    m.wf(a0 + 0x64, f2);
    m.wf(a0 + 0x68, f3);
    m.wf(a0 + 0x6c, f1);
    m.w32(a0 + 0x4c, u32::MAX);
    m.wf(a0 + 0x48, -1.0);
    m.w32(a0 + 0x58, 0);
}

/// FUN_001de400(o)
pub fn ee_sub_1de400<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001d_e2a0, &[a0], &[]);
    let v = m.r64(a0 + 0x148) & 0xffff_fff7_ffff_ffff;
    let a1 = m.r32(a0 + 0x40);
    w64(m, a0 + 0x148, v);
    m.wf(a0 + 0x1c0, 1.0);
    if (a1 ^ 1) & 1 != 0 {
        let x = m.r32(a1) | 8;
        m.w32(a1, x);
    } else {
        m.w32(a0 + 0x40, a1 | 8);
    }
}

/// FUN_00217f38(o, i): walk the i-th 0x18-byte bucket of +8 (count, 0x70-byte entries).
pub fn ee_sub_217f38<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let e = a1.wrapping_mul(0x18).wrapping_add(m.r32(a0 + 8));
    let mut p = m.r32(e + 4);
    let n = m.r32(e) as i32;
    if n <= 0 {
        return;
    }
    let mut k = n;
    loop {
        let q = m.r32(p + 0x30);
        if q != 0 {
            m.call(0x0021_58c0, &[q, p], &[]);
        } else {
            m.call(0x0021_56b8, &[p], &[]);
        }
        p = p.wrapping_add(0x70);
        k = k.wrapping_sub(1);
        if k == 0 {
            break;
        }
    }
}

/// FUN_001f6978(o, id): clear the flags set by FUN_001f69f8.
pub fn ee_sub_1f6978<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if (m.r32(a0 + 0xb8) as i32) <= 0 {
        return;
    }
    let mut i: i32 = 0;
    let mut off: u32 = 0;
    loop {
        let e = m.r32(a0 + 0xc0).wrapping_add(off);
        if m.r32(e) == a1 {
            let v = m.r32(e + 0xc) & 0xffff_f7ff;
            m.w32(e + 0xc, v);
            let p = m.r32(e + 0x24);
            if p != 0 {
                let v = m.r64(p + 0x10) & 0xfffb_ffff_ffff_ffff;
                w64(m, p + 0x10, v);
            }
        }
        i += 1;
        off = off.wrapping_add(0x28);
        if i >= m.r32(a0 + 0xb8) as i32 {
            break;
        }
    }
}

/// FUN_001eed78(list, a1, a2): from the last element down, virtual slot 12 (vt+0x6c)(a1, a2) until nonzero.
pub fn ee_sub_1eed78<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    let mut i = m.r32(a0 + 4) as i32 - 1;
    while i >= 0 {
        let obj = m.r32(a0.wrapping_add(8).wrapping_add((i as u32) << 2));
        if vslot(m, obj, 12, &[a1, a2], &[]).0 != 0 {
            break;
        }
        i -= 1;
    }
}

/// FUN_00180500(o)
pub fn ee_sub_180500<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0017_f090, &[a0], &[]);
    let f2 = m.rf(0x002a_5658);
    let f1 = m.rf(0x002a_565c);
    m.wf(a0 + 0x340, 1.0);
    m.wf(a0 + 0x350, 1.0);
    m.wf(a0 + 0x364, 1.0);
    m.wf(a0 + 0x338, 250.0);
    m.wf(a0 + 0x370, f2);
    m.wf(a0 + 0x374, f1);
    m.call(0x0018_13c0, &[a0], &[f4(1.0)]);
    m.call(0x0018_13f0, &[a0], &[f4(1.0)]);
    m.call(0x0017_f3e0, &[a0, 5, a0 + 0x220], &[]);
    m.w32(a0 + 0x1c0, u32::MAX);
}

/// FUN_001c45a8(o): latch +0x30 into +0x78; reset the newly set bits' timers (+0x88..) and two side timers.
pub fn ee_sub_1c45a8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let old = m.r32(a0 + 0x78);
    let cur = m.r32(a0 + 0x30);
    let f1 = m.rf(0x002d_ded0);
    let mut bits = (cur & !old) as i32;
    m.w32(a0 + 0x78, cur);
    if bits != 0 {
        let mut p = a0 + 0x88;
        loop {
            if bits & 1 != 0 {
                m.wf(p, f1);
            }
            bits >>= 1;
            p = p.wrapping_add(4);
            if bits == 0 {
                break;
            }
        }
    }
    let o = if 0.0 < m.rf(a0 + 0x1c) { 4 } else { 0 };
    m.wf(a0 + 0xc8 + o, f1);
    let o = if 0.0 < m.rf(a0 + 0x2c) { 4 } else { 0 };
    m.wf(a0 + 0xd0 + o, f1);
}

/// FUN_001f13a8(o, a1, a2): build {o, FUN_001f04f8(+0x24, a1), FUN_001f04f8(+0x24, a2)} on the stack; virtual
/// slot 3 (vt+0x24)(10, frame); virtual slot 2 (vt+0x1c)(8, 0, 2, {&frame[1], &frame[2]}, 0).
pub fn ee_sub_1f13a8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    let sp = stack_push(m, 0x20);
    m.w32(sp, a0);
    let x = m.r32(a0 + 0x24);
    let v = m.call(0x001f_04f8, &[x, a1], &[]).0;
    m.w32(sp + 4, v);
    let x = m.r32(a0 + 0x24);
    let v = m.call(0x001f_04f8, &[x, a2], &[]).0;
    m.w32(sp + 8, v);
    vslot(m, a0, 3, &[0xa, sp], &[]);
    m.w32(sp + 0x14, sp + 8);
    m.w32(sp + 0x10, sp + 4);
    vslot(m, a0, 2, &[8, 0, 2, sp + 0x10, 0], &[]);
    stack_pop(m, 0x20);
}

/// FUN_001843f0(o): FUN_001a5c48(o); if +0x38 == sentinel -> +0x38 = |FUN_001a62e8(o) - FUN_001a6f00(o, 0, +0x110)| + k.
pub fn ee_sub_1843f0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5c48, &[a0], &[]);
    if m.rf(a0 + 0x38) != f32::from_bits(0x5015_02f9) {
        return;
    }
    let a = m.call(0x001a_62e8, &[a0], &[]).1;
    let q = m.rv(a0 + 0x110);
    let b = m.call(0x001a_6f00, &[a0, 0], &[q]).1;
    let k = m.rf(0x002b_e904);
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    let s = (d[0] * d[0] + d[1] * d[1]) + 1.0 * (d[2] * d[2]);
    m.wf(a0 + 0x38, s.sqrt() + k);
}

/// FUN_001b49e0(o)
pub fn ee_sub_1b49e0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0014_00b8, &[a0], &[]);
    m.call(0x0020_b868, &[a0 + 0x1320], &[]);
    m.call(0x0020_b868, &[a0 + 0x1460], &[]);
    m.call(0x0020_b868, &[a0 + 0x1630], &[]);
    m.call(0x0020_b868, &[a0 + 0x1540], &[]);
    m.call(0x001a_0300, &[a0, a0 + 0x16f0, a0 + 0x17f0], &[]);
    m.call(0x0020_bd88, &[a0 + 0x1850, a0], &[]);
    m.call(0x0021_f388, &[a0 + 0x1910], &[]);
    m.call(0x0014_26e8, &[a0, 0x16], &[]);
    let x = m.r32(a0 + 0x18);
    let p = m.call(0x001f_4c58, &[a0, x, 0xc], &[]).0;
    if p != 0 {
        let q = m.rv(0x002a_2de0);
        let v = m.call(0x0012_dc18, &[a0, p, 0, 7, 2], &[q]).0;
        m.w32(a0 + 0x1acc, v);
    }
    m.call(0x001b_6078, &[a0, 0], &[]);
}

/// FUN_001e50e0(o, on, id): switch the active GS entry (+0x38 flag, +0x3c id).
pub fn ee_sub_1e50e0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    const GS: u32 = 0x003d_d490;
    if a1 == 0 {
        if m.r32(a0 + 0x38) == 0 {
            return;
        }
        let x = m.r32(a0 + 0x3c);
        m.call(0x0020_f030, &[GS, x], &[]);
        m.call(0x0020_f0f0, &[GS], &[]);
        m.w32(a0 + 0x3c, a2);
    } else {
        if m.r32(a0 + 0x38) != 0 {
            let x = m.r32(a0 + 0x3c);
            if a2 == x {
                return;
            }
            m.call(0x0020_f030, &[GS, x], &[]);
            m.call(0x0020_f0f0, &[GS], &[]);
        }
        m.call(0x0020_efc0, &[GS, a2], &[]);
        m.call(0x0020_f0c0, &[GS], &[]);
        m.w32(a0 + 0x3c, a2);
    }
    m.w32(a0 + 0x38, a1);
}

/// FUN_0012ec50(o): FUN_001a5c48(o); +0xec = FUN_001ae2a0({+0xe4 * 4, 0x10}); then for each 0x10-byte entry of
/// +0xf4 (count +0xf0): FUN_001fc408(*0x3dd268, FUN_001fe990(0x3dd040, e[0]), o, e+4, e[3]).
pub fn ee_sub_12ec50<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5c48, &[a0], &[]);
    let sp = stack_push(m, 0x10);
    let n = m.r32(a0 + 0xe4);
    m.w32(sp + 4, 0x10);
    m.w32(sp, n << 2);
    let v = m.call(0x001a_e2a0, &[sp], &[]).0;
    stack_pop(m, 0x10);
    m.w32(a0 + 0xec, v);
    let s3 = m.r32(0x003d_d268);
    if s3 == 0 || (m.r32(a0 + 0xf0) as i32) <= 0 {
        return;
    }
    let mut i: u32 = 0;
    loop {
        let e = m.r32(a0 + 0xf4).wrapping_add(i << 4);
        i = i.wrapping_add(1);
        let k = m.r32(e);
        let v = m.call(0x001f_e990, &[0x003d_d040, k], &[]).0;
        let f = m.rf(e + 0xc);
        m.call(0x001f_c408, &[s3, v, a0, e + 4], &[f4(f)]);
        if (i as i32) >= m.r32(a0 + 0xf0) as i32 {
            break;
        }
    }
}

/// FUN_00229f88(o): FUN_001f36a8(o); clear flag 8 on the +0x40 field of +0x16c / +0x170 / +0x174.
pub fn ee_sub_229f88<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001f_36a8, &[a0], &[]);
    for off in [0x16c, 0x170, 0x174] {
        let p = m.r32(a0 + off);
        if p == 0 {
            continue;
        }
        let a = m.r32(p + 0x40);
        if (a ^ 1) & 1 != 0 {
            let v = m.r32(a) & !8;
            m.w32(a, v);
        } else {
            m.w32(p + 0x40, a & !8);
        }
    }
}

/// FUN_00163b48(o)
pub fn ee_sub_163b48<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5e28, &[a0], &[]);
    if m.r32(a0 + 0x54) == 1 {
        let f2 = m.rf(a0 + 0x78);
        if 0.0 < f2 {
            let f1 = m.rf(0x002d_ded0) - m.rf(a0 + 0x58);
            if f1 < f2 {
                let f0 = f2 + m.rf(0x002d_ded4);
                if f0 <= f1 {
                    m.call(0x0016_5118, &[a0, 0], &[]);
                }
            }
        }
    }
    let a1 = m.r32(a0 + 0x80);
    if a1 == 0 {
        return;
    }
    let c = m.r32(a0 + 0x54) == 1 && m.r32(0x003e_0f9c) == 0;
    if c {
        vslot(m, a1, 7, &[], &[]);
    } else {
        vslot(m, a1, 8, &[], &[]);
    }
}

/// FUN_00256b48(): allocate an entry from the pool at 0x4520a8 (refilling it when empty).
pub fn ee_sub_256b48<M: QMem + ?Sized>(m: &mut M) -> u32 {
    const POOL: u32 = 0x0045_20a8;
    const N: u32 = 0x002d_d284;
    if m.r32(POOL + 0xc) == 0 {
        if m.r32(0x003e_0ea4) == 0 || m.r32(N) >= 4 {
            m.call(0x0012_86e0, &[0x002d_d598], &[]);
        }
        if m.r32(POOL + 0xc) == 0 {
            let s2 = m.call(0x0011_ccc0, &[POOL], &[]).0;
            if m.r32(N) != 0 {
                m.call(0x0011_cd98, &[POOL, s2], &[]);
            }
            let n = m.r32(N);
            m.w32((n << 2).wrapping_add(0x0045_2098), s2);
            m.w32(N, n.wrapping_add(1));
        }
    }
    let s0 = m.call(0x0011_ccc0, &[POOL], &[]).0;
    m.call(0x0025_50a0, &[s0], &[]);
    m.w32(s0 + 8, u32::MAX);
    m.w32(s0 + 0x10, 0);
    s0
}

/// FUN_001dfc50(o): FUN_001a5e28(o); when +0x74: +0x70 += dt; walk the +0x5c list (next at +8) through
/// FUN_001e1c40 under an iterator frame chained at 0x2dd2fc; FUN_0011a3b0(o+0x5c) -> FUN_001dfeb8(o).
pub fn ee_sub_1dfc50<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5e28, &[a0], &[]);
    if m.r32(a0 + 0x74) == 0 {
        return;
    }
    let mut v1 = m.r32(a0 + 0x40);
    if (v1 ^ 1) & 1 != 0 {
        v1 = m.r32(v1);
    }
    let f1 = if v1 & 0x100 != 0 { m.rf(0x002d_df10) } else { m.rf(0x002d_ded4) };
    let f0 = m.rf(a0 + 0x70);
    m.wf(a0 + 0x70, f0 + f1);
    let sp = stack_push(m, 0x10);
    let l = a0 + 0x5c;
    m.w32(sp + 4, l);
    m.w32(sp, l);
    let old = m.r32(0x002d_d2fc);
    let link = m.r32(l + 8);
    m.w32(0x002d_d2fc, sp);
    m.w32(sp + 0xc, old);
    let mut v0 = m.r32(a0 + 0x5c);
    m.w32(sp + 8, link);
    m.w32(sp + 4, v0.wrapping_add(link));
    while v0 != 0 {
        m.call(0x001e_1c40, &[v0], &[]);
        let p = m.r32(sp + 4);
        let k = m.r32(sp + 8);
        v0 = m.r32(p);
        m.w32(sp + 4, v0.wrapping_add(k));
    }
    if m.call(0x0011_a3b0, &[l], &[]).0 != 0 {
        m.call(0x001d_feb8, &[a0], &[]);
    }
    let old = m.r32(sp + 0xc);
    m.w32(0x002d_d2fc, old);
    stack_pop(m, 0x10);
}

/// FUN_001b7970(o)
pub fn ee_sub_1b7970<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001b_7d20, &[a0], &[]);
    let p = m.r32(a0 + 0x13f0);
    let s2 = m.r32(p + 0x180);
    m.w32(a0 + 0x13f0, 0);
    if (m.r64(s2 + 0x28) >> 60) & 1 == 0 {
        return;
    }
    if m.call(0x0013_2b10, &[s2, 0x38], &[]).0 != 0 {
        m.call(0x0020_9fb8, &[s2], &[]);
    }
    let v3 = m.call(0x001a_62e8, &[s2], &[]).1;
    let v2 = m.rv(a0 + 0x1420);
    let f0 = m.rf(a0 + 0x1414);
    let dx = v3[0] - v2[0];
    let dy = v3[1] - v2[1];
    let f20 = (dx * dx + dy * dy).sqrt() / f0;
    let s0 = m.call(0x001a_62e8, &[s2], &[]).1;
    let s1 = m.rv(a0 + 0x1420);
    let v = m.call(0x001f_5860, &[s2], &[]).1;
    let r = m.call(0x001d_69f0, &[], &[s0, s1, v, f4(f20)]).1;
    vslot(m, s2, 34, &[], &[r]);
    m.call(0x001f_7a68, &[s2, 2], &[]);
}

/// FUN_001b0e28(o): state machine on +0x220 (jump table 0x2a70f0), then FUN_001b1328(o, s); FUN_001b1160(o).
pub fn ee_sub_1b0e28<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    const T: u32 = 0x002d_ded0;
    m.call(0x001f_36e0, &[a0], &[]);
    let mut s0 = m.r32(a0 + 0x220);
    match s0 {
        0 => {
            s0 = m.call(0x001b_1900, &[a0], &[]).0;
            if s0 != 3 && m.rf(a0 + 0x228) <= m.rf(T) {
                if m.r32(a0 + 0x16c) != 0 {
                    s0 = 1;
                }
            }
        }
        1 => {
            s0 = m.call(0x001b_1900, &[a0], &[]).0;
            if s0 != 3 && m.rf(a0 + 0x228) <= m.rf(T) {
                s0 = 0;
            }
        }
        2 => {
            s0 = m.call(0x001b_1900, &[a0], &[]).0;
            if s0 != 3 {
                if m.call(0x001b_1868, &[a0], &[]).0 != 0 {
                    s0 = 1;
                }
            }
        }
        3 => {
            if m.r32(a0 + 0x1c4) == 0 {
                if m.call(0x001b_1868, &[a0], &[]).0 != 0 {
                    s0 = 4;
                }
            }
        }
        4 => {
            if m.rf(a0 + 0x22c) <= m.rf(T) {
                let f = m.rf(a0 + 0x168);
                m.call(0x001f_7bf8, &[a0], &[f4(f)]);
                let k = m.rf(0x002a_70e0);
                m.wf(a0 + 0x22c, k);
            }
        }
        _ => {}
    }
    m.call(0x001b_1328, &[a0, s0], &[]);
    m.call(0x001b_1160, &[a0], &[]);
}

/// FUN_001e5ae0(owner, desc, parent): allocate + init a 0xf0-byte node (vtable 0x2d8c08), find the matching
/// close marker in the 0x50-byte descriptor array, link into owner+0x5c, virtual slot 0.
pub fn ee_sub_1e5ae0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) -> u32 {
    let s0 = m.call(0x0011_ccc0, &[0x003e_1560], &[]).0;
    m.call(0x0023_33b4, &[s0, 0, 0xf0], &[]);
    m.w32(s0, 0);
    m.w32(s0 + 4, 0);
    m.w32(s0 + 0xc, 0x002d_8c08);
    m.w32(s0 + 0xec, 0);
    m.w32(s0 + 0x14, a0);
    if a2 != 0 {
        let v = m.r32(a2 + 0x18);
        m.w32(s0 + 0x18, v);
    } else {
        m.w32(s0 + 0x18, a0);
    }
    let k = m.r32(a1 + 0x10);
    if k != u32::MAX {
        let v = m.call(0x0015_e938, &[0x003d_d490, 5, k, 0], &[]).0;
        m.w32(s0 + 0x1c, v);
    }
    m.w32(s0 + 0x28, a1);
    let mut p = a1.wrapping_add(0x50);
    let end = m.r32(a0 + 0x54).wrapping_add(m.r32(a0 + 0x50).wrapping_mul(0x50).wrapping_sub(0x50));
    let mut depth: i32 = 1;
    let mut found = false;
    while end >= p {
        let v = m.r32(p);
        if v == 0 {
            depth += 1;
        } else if v == 1 {
            depth -= 1;
            if depth == 0 {
                m.w32(s0 + 0x2c, p);
                found = true;
                break;
            }
        }
        p = p.wrapping_add(0x50);
    }
    if !found {
        let v = m.r32(s0 + 0x28);
        m.w32(s0 + 0x2c, v);
    }
    m.call(0x0011_a250, &[a0 + 0x5c, s0], &[]);
    vslot(m, s0, 0, &[], &[]);
    s0
}

/// FUN_001c99d8(root, n, ids): descend the child chain (+0x18, sibling +0x14) matching ids; create the missing
/// tail (0x28-byte nodes from FUN_001ae2a0) and return the last node.
pub fn ee_sub_1c99d8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) -> u32 {
    let (mut s3, mut s1, mut s2) = (a0, a1, a2);
    let mut s0;
    'outer: loop {
        s0 = m.r32(s3 + 0x18);
        while s0 != 0 {
            if m.r32(s0) == m.r32(s2) {
                s3 = s0;
                if s1 == 1 {
                    return s0;
                }
                s1 = s1.wrapping_sub(1);
                s2 = s2.wrapping_add(4);
                continue 'outer;
            }
            s0 = m.r32(s0 + 0x14);
        }
        break;
    }
    let sp = stack_push(m, 0x10);
    m.w32(sp + 4, 0x10);
    m.w32(sp, s1.wrapping_mul(0x28));
    let mut v = m.call(0x001a_e2a0, &[sp], &[]).0;
    stack_pop(m, 0x10);
    let mut s4 = s3;
    if (s1 as i32) <= 0 {
        return s0;
    }
    let mut ids = s2;
    let mut cnt = s1;
    let mut q = v.wrapping_add(0x18);
    loop {
        s0 = v;
        m.call(0x0011_a050, &[q, 0x14], &[]);
        q = q.wrapping_add(0x28);
        cnt = cnt.wrapping_sub(1);
        let id = m.r32(ids);
        ids = ids.wrapping_add(4);
        m.w32(s0, id);
        let r = m.call(0x001c_9968, &[id], &[]).0;
        m.w32(s0 + 8, r);
        let r2 = m.call(0x0015_e3c0, &[0x003d_d490, r], &[]).0;
        m.call(0x0011_a070, &[r2, s0], &[]);
        m.w32(s0 + 4, s4);
        m.call(0x0011_a070, &[s4 + 0x18, s0], &[]);
        s4 = s0;
        if cnt == 0 {
            break;
        }
        v = s0.wrapping_add(0x28);
    }
    s0
}

/// FUN_00134648(o)
pub fn ee_sub_134648<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0013_ff70, &[a0], &[]);
    m.w32(a0 + 0x1040, 0);
    m.w32(a0 + 0x1800, u32::MAX);
    m.wf(a0 + 0x103c, 45.0);
    m.wf(a0 + 0x1048, 125.0);
    m.w32(a0 + 0x13d0, u32::MAX);
    m.w32(a0 + 0x16b0, u32::MAX);
    m.w32(a0 + 0x16bc, u32::MAX);
    m.call(0x0020_b840, &[a0 + 0x1320, a0, 0x73], &[]);
    m.call(0x0020_b840, &[a0 + 0x1600, a0, 0x73], &[]);
    let v = a0 + 0x10a8;
    m.wf(a0 + 0x3d8, -70.0);
    m.wf(a0 + 0x1098, -100.0);
    m.wf(a0 + 0x109c, 100.0);
    m.wf(a0 + 0x10a0, 50.0);
    m.wf(a0 + 0x10a4, -10.0);
    m.wf(a0 + 0x10a8, 250.0);
    m.wf(v + 0xc, f32::from_bits(0x458c_a000));
    m.wf(v + 8, 1.0);
    m.w32(v + 4, 0);
    m.w32(a0 + 0x1000, 2);
    m.w32(a0 + 0x1830, 7);
    m.w32(a0 + 0x183c, 9);
    m.w32(a0 + 0x1848, 0xb);
    m.w32(a0 + 0x1854, 0xd);
    m.w32(a0 + 0x1860, 4);
    m.w32(a0 + 0x1870, 5);
    m.w32(a0 + 0x1824, 5);
    m.w32(0x002d_d5bc, a0);
}

/// FUN_00117fd0(rd, out): read a mesh (verts, tris, edges) via the FUN_00117c20 reader.
pub fn ee_sub_117fd0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let sp = stack_push(m, 0x10);
    let mut s0: i32 = 0;
    let n = m.call(0x0011_7c20, &[a0], &[]).0;
    m.w32(sp + 4, 0x10);
    m.w32(sp, n << 4);
    m.w32(a1, n);
    let v = m.call(0x001a_e1d0, &[sp], &[]).0;
    m.w32(a1 + 4, v);
    if (m.r32(a1) as i32) > 0 {
        loop {
            let b = m.r32(a1 + 4);
            m.call(0x0011_7eb0, &[a0, b.wrapping_add((s0 as u32) << 4)], &[]);
            s0 += 1;
            if s0 >= m.r32(a1) as i32 {
                break;
            }
        }
    }
    let n2 = m.call(0x0011_7c20, &[a0], &[]).0;
    m.w32(sp + 4, 0x10);
    m.w32(sp, n2 << 5);
    m.w32(a1 + 8, n2);
    let v = m.call(0x001a_e2a0, &[sp], &[]).0;
    m.w32(a1 + 0xc, v);
    let n3 = m.call(0x0011_7c20, &[a0], &[]).0;
    m.w32(sp + 4, 0x10);
    m.w32(a1 + 0x10, n3);
    m.w32(sp, n3 << 4);
    let mut s7 = m.call(0x001a_e2a0, &[sp], &[]).0;
    m.w32(a1 + 0x14, s7);
    if (m.r32(a1 + 8) as i32) > 0 {
        let mut s6: i32 = 0;
        loop {
            let s5 = m.r32(a1 + 0xc).wrapping_add((s6 as u32) << 5);
            let fp = s6 + 1;
            let i0 = m.call(0x0011_7c20, &[a0], &[]).0;
            m.w32(sp, i0);
            let i1 = m.call(0x0011_7c20, &[a0], &[]).0;
            m.w32(sp + 4, i1);
            let i2 = m.call(0x0011_7c20, &[a0], &[]).0;
            m.w32(sp + 8, i2);
            let vb = m.r32(a1 + 4);
            let (j0, j1) = (m.r32(sp), m.r32(sp + 4));
            let q0 = m.rv(vb.wrapping_add(j0 << 4));
            let q1 = m.rv(vb.wrapping_add(j1 << 4));
            let q2 = m.rv(vb.wrapping_add(i2 << 4));
            m.call(0x0011_9308, &[s5], &[q0, q1, q2]);
            let j0 = m.r32(sp);
            m.w32(s5 + 0x1c, j0);
            let mut k: i32 = 3;
            loop {
                m.call(0x0011_7e38, &[a0], &[]);
                k -= 1;
                if k < 0 {
                    break;
                }
            }
            let s4 = m.call(0x0011_7bd0, &[a0], &[]).0 as i32;
            let mut s1: i32 = 0;
            if s4 > 0 {
                loop {
                    let e = s7;
                    let h = m.call(0x0011_7c20, &[a0], &[]).0;
                    s7 = e.wrapping_add(0x10);
                    m.w16(e, h as u16);
                    let h = m.call(0x0011_7c20, &[a0], &[]).0;
                    m.w16(e + 4, s6 as u16);
                    m.w16(e + 2, h as u16);
                    let h = m.call(0x0011_7c20, &[a0], &[]).0;
                    m.w16(e + 6, h as u16);
                    if s1 == 0 {
                        m.w32(s5 + 0x10, e);
                    } else {
                        m.w32(e.wrapping_sub(8), e);
                    }
                    let idx = m.r16(e + 6) as u32;
                    s1 += 1;
                    let t = m.r32(a1 + 0xc).wrapping_add(idx << 5);
                    let x = m.r32(t + 0x14);
                    m.w32(e + 0xc, x);
                    m.w32(t + 0x14, e);
                    if s1 >= s4 {
                        break;
                    }
                }
            }
            s6 = fp;
            if s6 >= m.r32(a1 + 8) as i32 {
                break;
            }
        }
    }
    stack_pop(m, 0x10);
}

/// FUN_00152970(o): fire the timed entries (+0x1c4, 0x24 bytes, count +0x1c0) per mode +0x50; ease +0x288.
pub fn ee_sub_152970<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5e28, &[a0], &[]);
    let f20 = m.call(0x001f_e3f8, &[0x003d_d040], &[]).1[0];
    let k = m.r32(a0 + 0x50);
    let s4 = a0 + 0x28c;
    if k == 1 {
        let n = m.r32(a0 + 0x1c0) as i32;
        let mut i: i32 = 0;
        while i < n {
            let e = a0.wrapping_add((i as u32).wrapping_mul(0x24)).wrapping_add(0x1c4);
            if m.r32(e + 0x14) == 0 && m.rf(e + 0xc) < f20 {
                let (x, y, z) = (m.r32(e), m.r32(e + 4), m.r32(e + 8));
                m.call(0x0015_2390, &[x, y, z, 0], &[]);
                m.w32(e + 0x14, 1);
                break;
            }
            i += 1;
        }
    } else if k == 2 {
        if (m.r32(a0 + 0x1c0) as i32) > 0 {
            let mut s2: i32 = 0;
            let mut e = a0 + 0x1c4;
            loop {
                if m.r32(e + 0x20) == 0 {
                    if m.r32(e + 0x18) == 0 && m.rf(e + 0xc) < f20 {
                        let v = m.r32(e + 4);
                        m.w32(a0 + 0x230, v);
                        m.call(0x0016_19a0, &[0x002d_eb30, 0x34, a0], &[]);
                        m.w32(e + 0x18, 1);
                    }
                    if m.rf(e + 0xc) + m.rf(a0 + 0x5c) < f20 {
                        m.w32(e + 0x20, 1);
                        if m.r32(e + 0x1c) == 0 {
                            let (x, y, z) = (m.r32(e), m.r32(e + 4), m.r32(e + 8));
                            m.call(0x0015_2390, &[x, y, z, 2], &[]);
                            let f = m.rf(a0 + 0x2a4);
                            m.call(0x0015_3640, &[a0], &[f4(f)]);
                        }
                        break;
                    }
                }
                s2 += 1;
                e = e.wrapping_add(0x24);
                if s2 >= m.r32(a0 + 0x1c0) as i32 {
                    break;
                }
            }
        }
    }
    let (f12, f13, f14) = (m.rf(a0 + 0x288), m.rf(a0 + 0x284), m.rf(0x002d_ded4));
    let f = m.call(0x0011_d018, &[s4, 0], &[f4(f12), f4(f13), f4(f14)]).1[0];
    m.wf(a0 + 0x288, f);
    m.call(0x0015_2b58, &[a0], &[]);
}

/// FUN_001accc8(o): menu/state dispatch on +0x40 (jump table 0x2a6d90, cases 0x11..0x2e).
pub fn ee_sub_1accc8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    const SAVE: u32 = 0x002f_633c;
    const FLAG: u32 = 0x002f_634c;
    let v1 = m.r32(a0 + 0x24);
    if v1 == 0 || m.r32(v1 + 0x58) != 2 {
        return;
    }
    let s1 = m.r32(v1 + 0x64);
    if s1 != 2 {
        return;
    }
    if m.call(0x0020_e040, &[0x003d_d490], &[]).0 != 0 {
        return;
    }
    let v1 = m.r32(a0 + 0x24);
    if m.rf(0x002d_df0c) - m.rf(v1 + 0x5c) <= f32::from_bits(0x3dcc_cccd) {
        return;
    }
    let r = m.call(0x001a_cc80, &[a0], &[]).0;
    if r == 1 {
        let p = m.r32(a0 + 0x10);
        if p != 0 && m.call(0x0018_7d80, &[p, 0x80], &[]).0 != 0 {
            m.call(0x001a_cfe0, &[a0], &[]);
            return;
        }
    } else if r == s1 {
        let p = m.r32(a0 + 0x10);
        if p != 0 {
            if m.call(0x0018_7d80, &[p, 0x80], &[]).0 != 0 {
                m.call(0x001a_d168, &[a0], &[]);
                return;
            }
            let p = m.r32(a0 + 0x10);
            if p != 0 && m.call(0x0018_7d80, &[p, 0x10], &[]).0 != 0 {
                m.call(0x001a_cfe0, &[a0], &[]);
                return;
            }
        }
    }
    let x = m.r32(a0 + 0x40).wrapping_sub(0x11);
    match x {
        0 | 1 | 4 | 7 | 29 => {
            if m.r32(0x002e_65e8) == 6 {
                m.call(0x001a_cbb8, &[a0], &[]);
            }
        }
        2 => {
            if m.call(0x001a_b088, &[SAVE], &[]).0 != 0 {
                return;
            }
            if m.r32(FLAG) == 0 {
                // 64-bit a1 = *(u64 *)0x2a6d88 (pointer-to-member pair), passed as raw bits
                let lo = m.r32(0x002a_6d88);
                let hi = m.r32(0x002a_6d8c);
                m.call(0x001a_af90, &[SAVE], &[[f32::from_bits(lo), f32::from_bits(hi), 0.0, 0.0]]);
                m.call(0x001a_cb58, &[a0, 0x16], &[]);
            } else {
                m.call(0x001a_cb58, &[a0, 0x14], &[]);
            }
        }
        5 => {
            if m.call(0x001a_b088, &[SAVE], &[]).0 != 0 {
                return;
            }
            if m.r32(FLAG) == 0 {
                m.call(0x001a_cbb8, &[a0], &[]);
            } else {
                m.call(0x001a_cb58, &[a0, 0x17], &[]);
            }
        }
        9 => {
            if m.call(0x001a_b088, &[SAVE], &[]).0 != 0 {
                return;
            }
            if m.r32(FLAG) != 0 {
                if m.r32(a0 + 0x4c) == 3 {
                    m.call(0x001a_cb58, &[a0, 0x23], &[]);
                } else {
                    m.call(0x001a_cb58, &[a0, 0x1d], &[]);
                }
            } else {
                m.call(0x001a_cbb8, &[a0], &[]);
                let k = m.r32(a0 + 0x4c);
                if k == 1 {
                    m.call(0x0016_60f8, &[k], &[]);
                } else if k == 2 {
                    m.call(0x001a_f660, &[0x003d_9ed0], &[]);
                }
            }
        }
        15 | 17 => {
            if m.r32(a0 + 0x44) != 1 {
                m.call(0x001a_cbb8, &[a0], &[]);
            }
        }
        16 => {
            if m.call(0x001a_b088, &[SAVE], &[]).0 != 0 {
                return;
            }
            m.w32(0x002f_6338, 0);
            if m.r32(FLAG) != 0 {
                m.call(0x001a_cb58, &[a0, 0x1d], &[]);
            } else {
                m.call(0x001a_cbb8, &[a0], &[]);
            }
        }
        _ => {}
    }
}

/// FUN_001b50b0(o): per-frame player update; state +0x13d0 dispatch via jump table 0x2a71e0.
pub fn ee_sub_1b50b0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    const GS: u32 = 0x003d_d490;
    const T: u32 = 0x002d_ded0;
    m.call(0x0014_03d0, &[a0], &[]);
    if (m.r64(a0 + 0x28) >> 60) & 1 == 0 {
        return;
    }
    let p = m.r32(a0 + 0x190c);
    if p != 0 && m.call(0x0013_1968, &[p], &[]).0 != 0 {
        m.call(0x0013_06a8, &[a0 + 0x190c], &[]);
    }
    m.call(0x0019_4a40, &[GS, a0, 0xf8fa], &[]);
    m.call(0x001b_5038, &[a0], &[]);
    m.call(0x001b_4aa0, &[a0], &[]);
    match m.r32(a0 + 0x13d0) {
        0 => {
            let v = m.call(0x0014_30c8, &[a0], &[]).0;
            let x = m.r32(a0 + 0x1034);
            m.call(0x001f_1350, &[x, v], &[]);
        }
        1 => {
            let q = m.r32(a0 + 0x150);
            let f = m.rf(q + 0x18);
            m.call(0x0020_4660, &[a0], &[f4(f)]);
            let s0 = a0 + 0x1320;
            m.call(0x0020_bb58, &[s0], &[f4(1.0)]);
            m.call(0x0020_bc08, &[s0], &[]);
        }
        10 => {
            let q = m.r32(a0 + 0x150);
            let f = m.rf(q + 0x18);
            m.call(0x0020_4660, &[a0], &[f4(f)]);
            let s0 = a0 + 0x1540;
            let q = m.r32(a0 + 0x150);
            let f = m.rf(q + 0x1c);
            m.call(0x0020_bb58, &[s0], &[f4(f)]);
            m.call(0x0020_bc08, &[s0], &[]);
        }
        2 => {
            m.call(0x0019_7f98, &[a0 + 0x2c0], &[]);
        }
        13 => {
            m.call(0x0019_1a10, &[a0 + 0x4c0], &[]);
        }
        11 => {
            m.call(0x001b_8b70, &[a0], &[]);
        }
        12 => {
            m.call(0x001b_90b8, &[a0], &[]);
        }
        17 | 18 => {
            let a1 = m.r32(a0 + 0x1030);
            let f = m.rf(a1 + 0x1c);
            let r = m.call(0x0020_4e38, &[a0, a1], &[f4(f)]).1;
            m.wv(a0 + 0x190, [-r[0], -r[1], -r[2], -r[3]]);
            if m.r32(a0 + 0x13d0) == 18 {
                let x = m.r32(a0 + 0x1030);
                let q = m.r32(a0 + 0x150);
                let f = m.rf(q + 0x1c);
                m.call(0x0013_1a00, &[x], &[f4(f)]);
                let q = m.r32(a0 + 0x150);
                let f = m.rf(q + 0x18);
                m.call(0x0020_4660, &[a0], &[f4(f)]);
            }
        }
        _ => {}
    }
    m.call(0x001b_82a0, &[a0], &[]);
    m.call(0x001a_1358, &[a0 + 0xdf0], &[]);
    let a2 = m.r32(a0 + 0x1430);
    if a2 != 0 {
        if m.rf(a0 + 0x1434) + f32::from_bits(0x3dcc_cccd) < m.rf(T) {
            let q = m.r32(a2 + 0xec);
            if q == 0 {
                m.w32(a0 + 0x1430, 0);
            } else {
                let x = m.r32(a0 + 0xec);
                if m.call(0x0013_3d08, &[GS, x, q], &[]).0 == 0 {
                    m.w32(a0 + 0x1430, 0);
                }
            }
            if m.r32(a0 + 0x1430) == 0 {
                m.call(0x0013_4320, &[GS, a0], &[]);
            }
        }
    }
    let p = m.r32(a0 + 0x1acc);
    if p != 0 {
        let d = m.rf(T) - m.rf(a0 + 0x1ac8);
        let q = if d < m.rf(0x002c_08e4) { m.rv(0x002c_08f0) } else { m.rv(0x002a_2de0) };
        m.call(0x0012_d108, &[p], &[q]);
    }
}

/// FUN_00181f60(o): per-frame update of the 0x30-byte limbs (+0x1cc, count +0x1c8) and their spring chains.
pub fn ee_sub_181f60<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    const T: u32 = 0x002d_ded0;
    m.call(0x001f_36e0, &[a0], &[]);
    if m.r32(a0 + 0x1c0) == 0 || (m.r32(a0 + 0x1c8) as i32) <= 0 {
        return;
    }
    let sp = stack_push(m, 0x10);
    let mut s1: i32 = 0;
    loop {
        let e = a0.wrapping_add((s1 as u32).wrapping_mul(0x30)).wrapping_add(0x1cc);
        let s3 = m.r32(e + 0x24);
        m.call(0x001e_6648, &[s3], &[]);
        if m.rf(e + 0x2c) < m.rf(T) {
            let c = m.r32(0x002d_fb80);
            let q = m.call(0x001a_62e8, &[c], &[]).1;
            m.call(0x001e_6f60, &[s3, 0, 0, sp, 0, e + 0x28], &[q]);
            let t = m.rf(T);
            let f = m.call(0x0011_d448, &[], &[f4(0.5), f4(0.75)]).1[0];
            m.wf(e + 0x2c, t + f);
        } else {
            let c = m.r32(0x002d_fb80);
            let q = m.call(0x001a_62e8, &[c], &[]).1;
            let f = m.rf(e + 0x28);
            m.call(0x001e_7380, &[s3, 0, 0, sp, 0, e + 0x28], &[q, f4(f)]);
        }
        m.call(0x0018_1cb0, &[a0, e], &[]);
        let mut s0 = m.r32(e + 0x10);
        let s4 = s1 + 1;
        while s0 != 0 {
            let (f13, f14, f12, f15) = (m.rf(s0 + 0x40), m.rf(s0 + 0x44), m.rf(s0 + 0x3c), m.rf(T + 4));
            let f = m.call(0x0011_d1c0, &[0x002b_e888, s0 + 0x40], &[f4(f12), f4(f13), f4(f14), f4(f15)]).1[0];
            m.wf(s0 + 0x3c, f);
            m.call(0x001e_6f10, &[s3, s0, s0 + 0x10], &[f4(f)]);
            let c0 = m.rf(0x002b_e898);
            let f7 = m.rf(s0 + 0x50);
            let c1 = m.rf(0x002b_e89c);
            let f3 = m.rf(s0 + 0x58);
            let f8 = m.rf(T + 4);
            let c2 = m.rf(0x002b_e8a0);
            let c3 = m.rf(0x002b_e8a4);
            let f5 = m.rf(s0 + 0x5c);
            let f2 = m.rf(s0 + 0x54);
            let mut f0 = c0 * f7 + c1 * f3;
            let f4v = c3 * f5;
            let mut f1 = c2 * f2 + f4v;
            f0 = f3 + f8 * f0;
            let f3b = (f3 + f0) * 0.5;
            m.wf(s0 + 0x58, f0);
            let mut f7b = f7 + f8 * f3b;
            m.wf(s0 + 0x50, f7b);
            let dt = m.rf(T + 4);
            f1 = f5 + dt * f1;
            let f5b = (f5 + f1) * 0.5;
            m.wf(s0 + 0x5c, f1);
            f7b = f7b + dt * f5b;
            m.wf(s0 + 0x50, f7b);
            m.call(0x0018_1e10, &[a0, s0], &[]);
            s0 = m.r32(s0 + 0x60);
        }
        s1 = s4;
        if s1 >= m.r32(a0 + 0x1c8) as i32 {
            break;
        }
    }
    stack_pop(m, 0x10);
}

fn cp16<M: QMem + ?Sized>(m: &mut M, dst: u32, src: u32) {
    for k in (0..16).step_by(4) {
        let w = m.r32(src + k);
        m.w32(dst + k, w);
    }
}

/// FUN_0015b428(o, pp, n, pos, vel, f0s, f1s, q3, q4 (stack arg)): spawn n particles into the emitter's
/// 0x2b-slot blocks (+0x448 chain, next +0x1090; new blocks via FUN_00138ff8).
pub fn ee_sub_15b428<M: QMem + ?Sized>(
    m: &mut M,
    a0: u32,
    a1: u32,
    a2: u32,
    a3: u32,
    t0: u32,
    t1: u32,
    t2: u32,
    t3: u32,
    a8: u32,
) {
    const GS: u32 = 0x003d_d490;
    let n = a2 as i32;
    let mut s4: u32 = 0;
    if a1 != 0 {
        let v = m.r32(a1);
        if v != 0 {
            s4 = m.r32(v);
        }
    }
    let mut s1;
    if s4 != 0 {
        s1 = m.r32(s4 + 0x448);
    } else {
        s4 = m.call(0x0013_90d8, &[GS], &[]).0;
        if s4 == 0 {
            return;
        }
        if a1 != 0 {
            let v = m.r32(a1);
            if v != 0 {
                m.w32(v, s4);
            }
        }
        m.call(0x0013_94c8, &[s4, a0], &[]);
        if a1 != 0 {
            let x = m.r32(a1 + 8);
            if x != 0 {
                vslot(m, s4, 50, &[x], &[]);
            }
        }
        s1 = m.r32(s4 + 0x448);
    }
    let mut i: i32 = 0;
    loop {
        if i >= n {
            return;
        }
        if s1 == 0 {
            s1 = m.call(0x0013_8ff8, &[GS, s4], &[]).0;
            if s1 == 0 {
                return;
            }
        }
        let mut s3 = m.r32(s1 + 0x1084);
        if s3 < 0x2b && i < n {
            let iu = i as u32;
            let mut s2 = (s3 << 4).wrapping_add(s1 + 0x840);
            let mut s0 = (s3 << 4).wrapping_add(0x560).wrapping_add(s1);
            let mut p24 = (iu << 2).wrapping_add(t1);
            let mut p28 = (iu << 4).wrapping_add(a3);
            let mut p20 = (iu << 4).wrapping_add(a8);
            let mut fp = (iu << 4).wrapping_add(t3);
            let mut s7 = (iu << 2).wrapping_add(t2);
            let mut s6 = (iu << 4).wrapping_add(t0);
            loop {
                cp16(m, s0.wrapping_sub(0x560), p28);
                let f = m.rf(p24);
                m.wf(s0.wrapping_sub(0x554), f);
                cp16(m, s0.wrapping_sub(0x2b0), s6);
                let f = m.rf(s7);
                m.wf(s0.wrapping_sub(0x2a4), f);
                m.call(0x0023_33b4, &[s0, 0, 0x10], &[]);
                let k = m.r32(s4 + 0x3b0).wrapping_sub(1);
                let v = m.call(0x0011_d3f8, &[0, k], &[]).0;
                m.w32(s0, v);
                if m.r32(s4 + 0x434) != 0 {
                    m.w32(s0, v & 0xffff_fffe);
                }
                if m.r32(a0 + 0xc8) != 0 {
                    let k = m.r32(s4 + 0x1a0).wrapping_sub(1);
                    let v = m.call(0x0011_d3f8, &[0, k], &[]).0;
                    m.w32(s0 + 4, v);
                }
                if m.r32(s4 + 0x43c) == 2 {
                    let (x, y) = (m.rf(a0 + 0xf8), m.rf(a0 + 0xfc));
                    let f2 = m.call(0x0011_d448, &[], &[f4(x), f4(y)]).1[0];
                    m.wf(s0 + 8, f2);
                    if m.r32(a0 + 0xf0) == 0 {
                        let v = m.rv(s0.wrapping_sub(0x2b0));
                        let c = m.rv(0x002d_dfb0);
                        let d = (c[0] * v[0] + c[1] * v[1]) + 1.0 * (c[2] * v[2]);
                        if 0.0 < d {
                            m.wf(s0 + 8, -f2);
                        }
                    }
                    if m.r32(a0 + 0xf4) != 0 {
                        let f = m.call(
                            0x0011_d448,
                            &[],
                            &[f4(f32::from_bits(0xc049_0fdb)), f4(f32::from_bits(0x4049_0fdb))],
                        )
                        .1[0];
                        m.wf(s0 + 0xc, f);
                    }
                }
                if m.r32(s4 + 0x160) == 2 {
                    let a = m.r32(s1 + 0x1088).wrapping_add(s3 << 5);
                    cp16(m, a, fp);
                    cp16(m, a + 0x10, p20);
                }
                s3 = s3.wrapping_add(1);
                fp = fp.wrapping_add(0x10);
                p20 = p20.wrapping_add(0x10);
                s7 = s7.wrapping_add(4);
                s6 = s6.wrapping_add(0x10);
                cp16(m, s2, s0.wrapping_sub(0x560));
                p24 = p24.wrapping_add(4);
                p28 = p28.wrapping_add(0x10);
                i += 1;
                cp16(m, s2 + 0x2b0, s0.wrapping_sub(0x2b0));
                cp16(m, s2 + 0x560, s0);
                s0 = s0.wrapping_add(0x10);
                s2 = s2.wrapping_add(0x10);
                let c = m.r32(s1 + 0x1084).wrapping_add(1);
                m.w32(s1 + 0x1084, c);
                let c = m.r32(s4 + 0x444).wrapping_add(1);
                m.w32(s4 + 0x444, c);
                if s3 >= 0x2b || i >= n {
                    break;
                }
            }
        }
        s1 = m.r32(s1 + 0x1090);
    }
}

/// FUN_00118668(root, limit, p0 (128-bit a2), p1 (128-bit a3), max, out): clip segment p0-p1 through the BSP
/// (node: [0] front, [4] back, [8] plane); emit up to `max` 0x60-byte leaf spans to `out`; returns the count.
/// Frame stack of 0x20-byte entries at sp+0x20 (current entry at sp+0).
pub fn ee_sub_118668<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, t0: u32, t1: u32, p0: V4, p1: V4) -> u32 {
    let sp = stack_push(m, 0x4a0);
    let r = ee_sub_118668_body(m, sp, a0, a1, t0, t1, p0, p1);
    stack_pop(m, 0x4a0);
    r
}

fn ee_sub_118668_body<M: QMem + ?Sized>(
    m: &mut M,
    sp: u32,
    a0: u32,
    s3: u32,
    s7: u32,
    mut s4: u32,
    p0: V4,
    p1: V4,
) -> u32 {
    let lerp = |a: f32, b: f32| -> V4 {
        [p0[0] * a + p1[0] * b, p0[1] * a + p1[1] * b, p0[2] * a + p1[2] * b, p0[3] * a + p1[3] * b]
    };
    let lim = |v: u32| if v < s3 { v } else { 0 };
    let mut s1 = sp + 0x20;
    let fp = s1;
    m.wv(sp + 0x470, p0);
    m.wv(sp + 0x480, p1);
    m.call(0x0023_33b4, &[sp, 0, 0x20], &[]);
    m.call(0x0023_33b4, &[s1, 0, 0x20], &[]);
    let f21: f32 = 1.0;
    m.w32(sp, lim(a0));
    m.wf(s1 + 8, 1.0);
    let (mut s2, mut s6, mut s5): (u32, u32, u32) = (0, 0, 0);
    let mut s0: u32 = 0;
    loop {
        // L708
        if m.r32(sp) == 0 {
            s0 = s4;
            // L720 .. L850 loop: emit leaves / pop frames while the current node is a leaf
            loop {
                if m.r32(sp + 4) != 0 {
                    if s2 != 0 && m.rf(sp + 8) == m.rf(s2 + 0x44) {
                        let t = m.rf(s1 + 8);
                        let v = lerp(1.0 - t, t);
                        m.wv(s2 + 0x10, v);
                        cp16(m, s2 + 0x30, s1 + 0x10);
                        let f = m.rf(s1 + 8);
                        m.wf(s2 + 0x44, f);
                        let x = m.r32(s1 + 0xc);
                        m.w32(s2 + 0x50, x);
                    } else {
                        if ((s5 as i32) >> 5) >= s7 as i32 {
                            return s7;
                        }
                        s2 = s0;
                        m.call(0x0023_33b4, &[s2, 0, 0x60], &[]);
                        s0 = s0.wrapping_add(0x60);
                        let f3 = m.rf(sp + 8);
                        s6 = s6.wrapping_add(0x20);
                        let f2 = m.rf(s1 + 8);
                        s5 = s5.wrapping_add(0x20);
                        s4 = s4.wrapping_add(0x60);
                        let va = lerp(1.0 - f3, f3);
                        let vb = lerp(1.0 - f2, f2);
                        let a4 = m.r32(s1 + 0xc);
                        let a5 = m.r32(sp + 0xc);
                        m.wv(s2, va);
                        m.wv(s2 + 0x10, vb);
                        cp16(m, s2 + 0x20, sp + 0x10);
                        cp16(m, s2 + 0x30, s1 + 0x10);
                        m.wf(s2 + 0x40, f3);
                        m.wf(s2 + 0x44, f2);
                        m.w32(s2 + 0x4c, a5);
                        m.w32(s2 + 0x50, a4);
                    }
                }
                // L84c / L850: pop
                if fp >= s1 {
                    return ((s6 as i32) >> 5) as u32;
                }
                cp16(m, sp, s1);
                cp16(m, sp + 0x10, s1 + 0x10);
                s1 = s1.wrapping_sub(0x20);
                if m.r32(sp) != 0 {
                    break;
                }
            }
        }
        // L878: split against the node plane
        let node = m.r32(sp);
        let pl = m.r32(node + 8);
        let plane = m.rv(pl);
        m.wv(sp + 0x490, plane);
        m.call(0x0025_64b8, &[sp + 0x430], &[]);
        m.call(0x0025_64b8, &[sp + 0x440], &[]);
        let f20 = (plane[0] * p0[0] + plane[1] * p0[1]) + 1.0 * (plane[2] * p0[2]) - 1.0 * plane[3];
        m.call(0x0025_64b8, &[sp + 0x450], &[]);
        m.call(0x0025_64b8, &[sp + 0x460], &[]);
        let f6 = (plane[0] * p1[0] + plane[1] * p1[1]) + 1.0 * (plane[2] * p1[2]) - 1.0 * plane[3];
        let v1;
        if f20 <= 0.0 && f6 <= 0.0 {
            m.w32(sp + 4, node);
            v1 = m.r32(node);
        } else if 0.0 < f20 && 0.0 < f6 {
            m.w32(sp + 4, 0);
            v1 = m.r32(node + 4);
        } else {
            let f3d = f20 - f6;
            let f1 = m.rf(sp + 8);
            let f5 = m.rf(s1 + 8);
            let f2 = (f21 - f1) * f20;
            let f7 = f20 / f3d;
            let f0 = (f21 - f5) * f20;
            let f4 = f6 * f5 + f0;
            let f3 = f6 * f1 + f2;
            if f1 < f7 && f7 < f5 {
                s1 = s1.wrapping_add(0x20);
                m.wf(s1 + 8, f7);
                cp16(m, s1 + 0x10, sp + 0x490);
                let x = m.r32(node + 8);
                m.w32(s1 + 0xc, x);
                if f3 <= 0.0 {
                    m.w32(s1 + 4, 0);
                    let v = m.r32(node + 4);
                    m.w32(s1, lim(v));
                } else {
                    m.w32(s1 + 4, node);
                    let v = m.r32(node);
                    m.w32(s1, lim(v));
                }
            }
            if f3 < 0.0 || (f3 == 0.0 && f4 <= 0.0) {
                m.w32(sp + 4, node);
                v1 = m.r32(node);
            } else {
                m.w32(sp + 4, 0);
                v1 = m.r32(node + 4);
            }
        }
        m.w32(sp, lim(v1));
    }
}

/// FUN_001aeee0(o): menu page update: run the current page's (+0x24 -> +0xc + 0x1c*n) item callbacks, then the
/// input state machine on +0x1e8 (jump table 0x2a6f50).
pub fn ee_sub_1aeee0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001d_9870, &[a0], &[]);
    let n = m.r32(a0 + 0x24) as i32;
    if n <= 0 {
        return;
    }
    let s2 = (n as u32).wrapping_mul(0x1c).wrapping_add(0xc).wrapping_add(a0);
    if (m.r32(s2) as i32) > 0 {
        let mut s0: i32 = 0;
        loop {
            let e = m.r32(s2 + 4).wrapping_add((s0 as u32) << 5);
            let f = m.r32(e + 0x10);
            if f != 0 {
                m.call(f, &[e], &[]);
            }
            s0 += 1;
            if s0 >= m.r32(s2) as i32 {
                break;
            }
        }
    }
    let menu_chk = |m: &mut M, a1: u32| -> bool {
        let x = m.r32(s2 + 8);
        m.call(0x0021_21e8, &[x, a1], &[]).0 != 0
    };
    let back = |m: &mut M| {
        let a1 = m.r32(s2 + 8);
        vslot(m, a1, 49, &[], &[]);
        m.call(0x001a_eaa8, &[a0, 0], &[]);
    };
    let dir = |m: &mut M| {
        let p = m.r32(a0 + 0x10);
        let v = m.call(0x0018_80d8, &[p], &[]).0 as i32;
        if v < 0 {
            m.call(0x001a_eaa8, &[a0, 6], &[]);
        } else if v > 0 {
            m.call(0x001a_eaa8, &[a0, 5], &[]);
        }
    };
    let st = m.r32(a0 + 0x1e8);
    match st {
        1 => {
            if menu_chk(m, 0x314) {
                return;
            }
            let v = m.r32(a0 + 0x1ec);
            if v != u32::MAX {
                m.w32(a0 + 0x24, v);
                m.w32(a0 + 0x1ec, u32::MAX);
            }
            m.call(0x001a_eaa8, &[a0, 7], &[]);
        }
        2 => {
            if menu_chk(m, 0x314) {
                return;
            }
            let v = m.r32(a0 + 0x1ec);
            if v == u32::MAX {
                let c = m.r32(a0 + 0x24).wrapping_sub(1);
                m.w32(a0 + 0x24, c);
            } else {
                m.w32(a0 + 0x1ec, u32::MAX);
            }
            if m.r32(a0 + 0x24) == 0 {
                back(m);
            } else {
                m.call(0x001a_eaa8, &[a0, 7], &[]);
            }
        }
        3 => {
            let p = m.r32(s2 + 0x18);
            let a1 = m.r32(p + 0x14);
            if menu_chk(m, a1) {
                return;
            }
            let v = m.r32(a0 + 0x1ec);
            m.w32(a0 + 0x1ec, u32::MAX);
            m.w32(a0 + 0x24, v);
            m.call(0x001a_eaa8, &[a0, 4], &[]);
        }
        4 => {
            let p = m.r32(s2 + 0x18);
            let a1 = m.r32(p + 0xc);
            if menu_chk(m, a1) {
                return;
            }
            m.call(0x001a_eaa8, &[a0, 7], &[]);
        }
        8 => {
            if menu_chk(m, 0x314) {
                return;
            }
            back(m);
        }
        5 | 6 => {
            let p = m.r32(s2 + 0x18);
            let a1 = m.r32(p + if st == 5 { 0x18 } else { 0x1c });
            if menu_chk(m, a1) {
                return;
            }
            m.call(0x001a_eaa8, &[a0, 7], &[]);
            if m.r32(a0 + 0x10) == 0 {
                return;
            }
            dir(m);
        }
        7 => {
            let p = m.r32(a0 + 0x10);
            if p == 0 {
                return;
            }
            let v1 = m.call(0x0018_7d80, &[p, 0x870], &[]).0;
            if v1 & 0x800 != 0 {
                m.call(0x001a_f660, &[a0], &[]);
                return;
            }
            // L124 (by +0xc) vs L134 (by +0x24)
            let by_c = if v1 & 0x10 != 0 { true } else if m.r32(s2) != 0 { false } else { v1 != 0 };
            if by_c {
                if m.r32(s2 + 0xc) != 0 {
                    m.call(0x001a_f490, &[a0], &[]);
                } else {
                    m.call(0x001a_f608, &[a0], &[]);
                }
                return;
            }
            if (m.r32(a0 + 0x24) as i32) > 0 && v1 & 0x40 != 0 {
                let s0 = m.r32(s2 + 4).wrapping_add(m.r32(s2 + 0x14) << 5);
                let f = m.r32(s0 + 4);
                if f != 0 {
                    m.call(f, &[s0], &[]);
                }
                let fl = m.r32(s0 + 0x1c);
                if fl & 2 != 0 {
                    return;
                }
                if fl & 1 == 0 {
                    m.call(0x001a_f660, &[a0], &[]);
                    return;
                }
                if m.r32(s2 + 0xc) != 0 {
                    m.call(0x001a_f490, &[a0], &[]);
                } else {
                    m.call(0x001a_f608, &[a0], &[]);
                }
                return;
            }
            if v1 & 0x20 != 0 {
                let r = m.call(0x001b_0078, &[a0], &[]).0;
                if r != 0x19 && r != 0x1e {
                    return;
                }
                let s0 = m.r32(s2 + 0x14).wrapping_mul(0x5464).wrapping_add(0x002e_6604);
                let sp = stack_push(m, 0x10);
                let x = m.r32(0x003e_0fac);
                m.call(0x0025_81c0, &[sp, x, s0], &[]);
                let f20 = m.rf(sp + 4);
                m.call(0x0012_9ec8, &[sp], &[]);
                stack_pop(m, 0x10);
                if f20 == 0.0 {
                    return;
                }
                m.w32(0x002f_6338, s0);
                m.call(0x001a_cb58, &[0x002e_6550, 0x20], &[]);
                return;
            }
            if s2 == 0 {
                return;
            }
            dir(m);
        }
        11 => {
            if menu_chk(m, 0x331) {
                return;
            }
            m.call(0x001a_eaa8, &[a0, 0xa], &[]);
        }
        9 => {
            if menu_chk(m, 0x332) {
                return;
            }
            m.call(0x001a_eaa8, &[a0, 7], &[]);
        }
        10 => {
            let p = m.r32(a0 + 0x10);
            if p == 0 {
                return;
            }
            let s0 = m.r32(s2 + 4).wrapping_add(m.r32(s2 + 0x14) << 5);
            let v1 = m.call(0x0018_7d80, &[p, 0x850], &[]).0;
            if v1 & 0x800 != 0 {
                m.call(0x001a_ff88, &[a0, s0], &[]);
                m.call(0x001a_f660, &[a0], &[]);
            } else if v1 & 0x10 != 0 {
                m.call(0x001a_ff88, &[a0, s0], &[]);
            } else if v1 & 0x40 != 0 {
                m.call(0x001a_fb88, &[a0, s0], &[]);
            } else {
                m.call(0x001a_f6d0, &[a0, s2, s0], &[]);
            }
        }
        _ => {}
    }
}

/// FUN_0021aa60(init, prio): static initializer: build the 0x3e1c70 table (FUN_00116a98), copy 107 12-byte
/// records from 0x2a8b90 (stride 0x10) to 0x3e1c88 (stride 0xc) plus one stack-built record, then bump the
/// six pool refcounts at 0x2dd2e0.. and create each pool (FUN_0011cca8) on its first reference.
pub fn ee_sub_21aa60<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if a1 != 0xffff || a0 == 0 {
        return;
    }
    m.call(0x0011_6a98, &[0x003e_1c70, 0x60000, 0x003e_21a0], &[]);
    let sp = stack_push(m, 0x10);
    let cp12 = |m: &mut M, d: u32, s: u32| {
        for k in [0u32, 4, 8] {
            let w = m.r32(s + k);
            m.w32(d + k, w);
        }
    };
    for k in 0u32..107 {
        cp12(m, 0x003e_1c88 + k.wrapping_mul(0xc), 0x002a_8b90 + k.wrapping_mul(0x10));
        if k == 10 {
            m.w32(0x003e_1c80, 0x002c_a578);
        } else if k == 11 {
            m.w32(0x003e_1c84, u32::MAX);
        }
    }
    m.call(0x0023_33b4, &[sp, 0, 0xc], &[]);
    let a2 = m.r32(0x002c_40a0);
    let c = m.r32(0x002d_d2e0).wrapping_add(1);
    m.w32(sp + 4, 0x0021_0eb0);
    m.w32(sp, 0x5c1);
    m.w32(sp + 8, a2);
    m.w32(0x002d_d2e0, c);
    cp12(m, 0x003e_1c88 + 107 * 0xc, sp);
    stack_pop(m, 0x10);
    if c == 1 {
        m.call(0x0011_cca8, &[0x0045_20a8, 0x14, 0x2000, 0x10], &[]);
    }
    for (cnt, pool, sz, n) in [
        (0x002d_d2e4u32, 0x0045_20d0u32, 0x18u32, 0x800u32),
        (0x002d_d2e8, 0x0045_20f8, 0x28, 0x200),
        (0x002d_d2ec, 0x0045_2120, 0x1c, 0x800),
        (0x002d_d2f0, 0x0045_2148, 0xc, 0xc8),
        (0x002d_d2f4, 0x0045_2170, 0x14, 0x100),
    ] {
        let v = m.r32(cnt).wrapping_add(1);
        m.w32(cnt, v);
        if v == 1 {
            m.call(0x0011_cca8, &[pool, sz, n, 0x10], &[]);
        }
    }
}
