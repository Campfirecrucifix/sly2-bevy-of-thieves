//! WIRE-W1H14: 1:1 queue ports (against `QMem`) of a batch of live-frontier natives, ported from the EE asm;
//! callees go through `m.call` / `vslot`. Integer / pointer args in `a`, quadword and float args in `v`.
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, clippy::all)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

const GS: u32 = 0x003d_d490; // game state / world

/// FUN_00205d10: empty.
pub fn ee_sub_205d10<M: QMem + ?Sized>(m: &mut M) {}

/// FUN_001c9950: a0 >> 16 (arith).
pub fn ee_sub_1c9950<M: QMem + ?Sized>(m: &mut M, a0: u32) -> i32 {
    (a0 as i32) >> 16
}

/// FUN_0018de08: returns 0x2a5ef0.
pub fn ee_sub_18de08<M: QMem + ?Sized>(m: &mut M) -> u32 {
    0x002a_5ef0
}

/// FUN_001fee90: ++*(a0+0x5c).
pub fn ee_sub_1fee90<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let v = m.r32(a0 + 0x5c).wrapping_add(1);
    m.w32(a0 + 0x5c, v);
}

/// FUN_0022d698: --DAT_004437a0.
pub fn ee_sub_22d698<M: QMem + ?Sized>(m: &mut M) {
    let v = m.r32(0x0044_37a0).wrapping_sub(1);
    m.w32(0x0044_37a0, v);
}

/// FUN_001441c0: if p=+0x12fc: *p = min(a1, +0x1300).
pub fn ee_sub_1441c0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let p = m.r32(a0 + 0x12fc);
    if p != 0 {
        let lim = m.r32(a0 + 0x1300);
        let v = if (a1 as i32) < (lim as i32) { a1 } else { lim };
        m.w32(p, v);
    }
}

/// FUN_001a2c98: FUN_001a2c48(&tmp, 0); return tmp.
pub fn ee_sub_1a2c98<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let sp = stack_push(m, 0x20);
    m.call(0x001a_2c48, &[sp, 0], &[]);
    let r = m.r32(sp);
    stack_pop(m, 0x20);
    r
}

/// FUN_001d5310: FUN_001d5340(a0, a1*0x104 + 0x2c2260).
pub fn ee_sub_1d5310<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let p = a1.wrapping_mul(0x104).wrapping_add(0x002c_2260);
    m.call(0x001d_5340, &[a0, p], &[]);
}

/// FUN_001bbe18: 1 if 0 <= +0x15c < +0x160 or +0x15c == -1.
pub fn ee_sub_1bbe18<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let i = m.r32(a0 + 0x15c) as i32;
    if i >= 0 && i < m.r32(a0 + 0x160) as i32 {
        return 1;
    }
    (i == -1) as u32
}

/// FUN_002323a8: tmp = a0; FUN_00232838(0x13, 4, &tmp, 0, 0).
pub fn ee_sub_2323a8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let sp = stack_push(m, 0x20);
    m.w32(sp, a0);
    m.call(0x0023_2838, &[0x13, 4, sp, 0, 0], &[]);
    stack_pop(m, 0x20);
}

/// FUN_00144038: (vtable +0x2c8 slot)(o) - 5 < 2 (unsigned).
pub fn ee_sub_144038<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let vt = m.r32(a0 + 0xc);
    let adj = m.r16(vt + 0x2c8) as i16 as i32 as u32;
    let f = m.r32(vt + 0x2cc);
    let r = m.call(f, &[a0.wrapping_add(adj)], &[]).0;
    (r.wrapping_sub(5) < 2) as u32
}

/// FUN_00203758: +0xec ? FUN_00164030(+0xec) : +0xf0.
pub fn ee_sub_203758<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let p = m.r32(a0 + 0xec);
    if p == 0 {
        m.r32(a0 + 0xf0)
    } else {
        m.call(0x0016_4030, &[p], &[]).0
    }
}

/// FUN_001aff88: FUN_001d8ff8(0x3d4a60); FUN_001aeaa8(a0, 9).
pub fn ee_sub_1aff88<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001d_8ff8, &[0x003d_4a60], &[]);
    m.call(0x001a_eaa8, &[a0, 9], &[]);
}

/// FUN_0022b7a0: FUN_0011a070(a0+0x8f0); *(a1+0xc) = a0.
pub fn ee_sub_22b7a0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.call(0x0011_a070, &[a0 + 0x8f0], &[]);
    m.w32(a1 + 0xc, a0);
}

/// FUN_001a2160: kind = a0[1]; clear a1[0xd] for kind 5 / 8..12; then a1[0] |= 1.
pub fn ee_sub_1a2160<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let k = m.r32(a0 + 4) as i32;
    if k == 5 || (k >= 8 && k < 0xd) {
        m.w32(a1 + 0x34, 0);
    }
    let v = m.r32(a1) | 1;
    m.w32(a1, v);
}

/// FUN_001baed8: +0x170 = a2; +0x16c = a1; FUN_00149ab8(GS, a0, DAT_002a7630, 0).
pub fn ee_sub_1baed8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    m.w32(a0 + 0x170, a2);
    m.w32(a0 + 0x16c, a1);
    let d = m.r32(0x002a_7630);
    m.call(0x0014_9ab8, &[GS, a0, d, 0], &[]);
}

/// FUN_0019de10: init.
pub fn ee_sub_19de10<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.w32(a0, a1);
    m.w32(a0 + 4, 0xffff_ffff);
    m.wf(a0 + 0x30, 45.0);
    m.wf(a0 + 0x38, 40.0);
    m.w32(a0 + 0x34, 0);
    m.wf(a0 + 0x40, 45.0);
    m.wf(a0 + 0x48, -20.0);
    m.w32(a0 + 0x44, 0);
}

/// FUN_0011e290: d = dot3(a, b); proj = b*d -> *a2; a - proj -> *a3; returns d.
pub fn ee_sub_11e290<M: QMem + ?Sized>(m: &mut M, a2: u32, a3: u32, a: V4, b: V4) -> f32 {
    let d = (a[0] * b[0] + a[1] * b[1]) + 1.0 * (a[2] * b[2]);
    let p = [b[0] * d, b[1] * d, b[2] * d, b[3] * d];
    if a2 != 0 {
        m.wv(a2, p);
    }
    if a3 != 0 {
        m.wv(a3, [a[0] - p[0], a[1] - p[1], a[2] - p[2], a[3] - p[3]]);
    }
    d
}

/// FUN_001a2c48: idx = DAT_002e1e58 (reset to 0 if > 6); *a0 = tbl[idx].0; *a1 = tbl[idx].1 (tbl 0x2bfbd0).
pub fn ee_sub_1a2c48<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if m.r32(0x002e_1e58) >= 7 {
        m.w32(0x002e_1e58, 0);
    }
    let e = (m.r32(0x002e_1e58) << 3).wrapping_add(0x002b_fbd0);
    if a0 != 0 {
        let v = m.r32(e);
        m.w32(a0, v);
    }
    if a1 != 0 {
        let v = m.r32(e + 4);
        m.w32(a1, v);
    }
}

fn fb(b: u32) -> f32 {
    f32::from_bits(b)
}

/// FUN_001d4c88: if +0x568: t = +0x560 / +0x55c (int->float), 1-t if +0x564; FUN_00212e40(+0x568, 0x34f; t).
pub fn ee_sub_1d4c88<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let p = m.r32(a0 + 0x568);
    if p == 0 {
        return;
    }
    let n = m.r32(a0 + 0x560) as i32 as f32;
    let d = m.r32(a0 + 0x55c) as i32 as f32;
    let mut t = n / d;
    if m.r32(a0 + 0x564) != 0 {
        t = 1.0 - t;
    }
    m.call(0x0021_2e40, &[p, 0x34f], &[f4(t)]);
}

/// FUN_00255d90: *a0 = 0; FUN_00255c00(a0, 0x2dd2c0); a0[1] bit31 = DAT_002dd598 bit0; returns a0.
pub fn ee_sub_255d90<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    m.w32(a0, 0);
    m.call(0x0025_5c00, &[a0, 0x002d_d2c0], &[]);
    let v = (m.r32(a0 + 4) & 0x7fff_ffff) | (m.r32(0x002d_d598) << 31);
    m.w32(a0 + 4, v);
    a0
}

/// FUN_002560c8: *a0 = 0; FUN_00255f38(a0, 0x2dd2d0); a0[1] bit31 = DAT_002dd598 bit0; returns a0.
pub fn ee_sub_2560c8<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    m.w32(a0, 0);
    m.call(0x0025_5f38, &[a0, 0x002d_d2d0], &[]);
    let v = (m.r32(a0 + 4) & 0x7fff_ffff) | (m.r32(0x002d_d598) << 31);
    m.w32(a0 + 4, v);
    a0
}

/// raw vtable-offset virtual call: (*(vt+off+4))(obj + *(short*)(vt+off), ...)
fn vcall<M: QMem + ?Sized>(m: &mut M, obj: u32, off: u32, a: &[u32], v: &[V4]) -> (u32, V4) {
    let vt = m.r32(obj + 0xc);
    let adj = m.r16(vt + off) as i16 as i32 as u32;
    let f = m.r32(vt + off + 4);
    let mut args = vec![obj.wrapping_add(adj)];
    args.extend_from_slice(a);
    m.call(f, &args, v)
}

/// FUN_001acbb8: +0x40 = -1; if +0x24: this->vt(0x58)(); (+0x24)->vt(0x190)().
pub fn ee_sub_1acbb8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.w32(a0 + 0x40, 0xffff_ffff);
    if m.r32(a0 + 0x24) == 0 {
        return;
    }
    vcall(m, a0, 0x58, &[], &[]);
    let o = m.r32(a0 + 0x24);
    vcall(m, o, 0x190, &[], &[]);
}

/// FUN_001e6660(a0; f): wrap f into [tbl[0], tbl[n-1]] via FUN_0011d898 (fmod) when outside.
pub fn ee_sub_1e6660<M: QMem + ?Sized>(m: &mut M, a0: u32, f: f32) -> f32 {
    if m.r32(a0 + 0xe4) == 0 {
        return f;
    }
    let t = m.r32(a0 + 0xec);
    let n = m.r32(a0 + 0xe8);
    let lo = m.rf(t);
    let hi = m.rf((n << 2).wrapping_add(t).wrapping_sub(4));
    if lo <= f && f <= hi {
        return f;
    }
    let r = m.call(0x0011_d898, &[], &[f4(f - lo), f4(hi - lo)]).1[0];
    lo + r
}

/// FUN_0011e9f8(a0 qword; f12): v = a0; if f12 < |v.xyz|: v *= f12/|v| (all lanes). Returns v (128-bit).
pub fn ee_sub_11e9f8<M: QMem + ?Sized>(m: &mut M, a: V4, f12: f32) -> V4 {
    let len = (a[0] * a[0] + a[1] * a[1] + 1.0 * (a[2] * a[2])).sqrt();
    if !(f12 < len) {
        return a;
    }
    let s = f12 / len;
    // (the sqc2 of the scalar to the frame is a dead scratch store)
    [a[0] * s, a[1] * s, a[2] * s, a[3] * s]
}

/// FUN_0018f740: FUN_0018f2b0(a0, 4, 0x2bf4e0, &found, &q; +0x1a8, 0.25, 3.15); +0x18c0 = found;
/// if found: FUN_002046b0(a0, q); FUN_00204ca8(a0).
pub fn ee_sub_18f740<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let sp = stack_push(m, 0x40);
    let f = m.rf(a0 + 0x1a8);
    m.call(0x0018_f2b0, &[a0, 4, 0x002b_f4e0, sp + 0x10, sp], &[f4(f), f4(0.25), f4(fb(0x4049_999a))]);
    let v = m.r32(sp + 0x10);
    m.w32(a0 + 0x18c0, v);
    if v != 0 {
        let q = m.rv(sp);
        m.call(0x0020_46b0, &[a0], &[q]);
        m.call(0x0020_4ca8, &[a0], &[]);
    }
    stack_pop(m, 0x40);
}

/// FUN_00190be0: p = (+0x18c4)->vt(0xf8)(); q = same; *a1 = FUN_002458f0(p[1], *q).
pub fn ee_sub_190be0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let o = m.r32(a0 + 0x18c4);
    let p = vcall(m, o, 0xf8, &[], &[]).0;
    let o = m.r32(a0 + 0x18c4);
    let q = vcall(m, o, 0xf8, &[], &[]).0;
    let x = m.rf(p + 4);
    let y = m.rf(q);
    let r = m.call(0x0024_58f0, &[], &[f4(x), f4(y)]).1[0];
    m.wf(a1, r);
}

/// FUN_002156b8: init.
pub fn ee_sub_2156b8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.w32(a0 + 0x44, 0x002a_3a10);
    m.w32(a0 + 0x4c, 0x7f7f_7f7f);
    m.w32(a0 + 0x50, 0xffff_ffff);
    m.w32(a0 + 0x64, 0xffff_ffff);
    m.w32(a0 + 0x30, 0);
    m.w32(a0 + 0x54, 0);
    m.w32(a0, 0x7f7f_7f7f);
    m.w32(a0 + 4, 0xffff_ffff);
    m.w32(a0 + 0x3c, 0x7f7f_7f7f);
    m.w32(a0 + 0x40, 0xffff_ffff);
    m.w32(a0 + 0x58, 0);
    m.w32(a0 + 0x34, 0);
    m.w32(a0 + 0x5c, 0);
    m.w32(a0 + 0x38, 0);
    m.w32(a0 + 0x48, 0);
    m.call(0x0011_a058, &[a0 + 0x68], &[]);
}

/// FUN_00172560: +0x38 = DAT_002dded0 + FUN_0011d448(o.e6c, o.e70); +0x3c = (o.e78 == 1) ?
/// DAT_002dded0 + DAT_002be5a0 : DAT_002a53b8 (o = +0x10).
pub fn ee_sub_172560<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let base = m.rf(0x002d_ded0);
    let o = m.r32(a0 + 0x10);
    let x = m.rf(o + 0xe6c);
    let y = m.rf(o + 0xe70);
    let r = m.call(0x0011_d448, &[], &[f4(x), f4(y)]).1[0];
    m.wf(a0 + 0x38, base + r);
    let o = m.r32(a0 + 0x10);
    let v = if m.r32(o + 0xe78) == 1 { m.rf(0x002d_ded0) + m.rf(0x002b_e5a0) } else { m.rf(0x002a_53b8) };
    m.wf(a0 + 0x3c, v);
}

/// FUN_001fe3f8: e = (+0x204 > 0) ? a0[+0x204 + n*4] : 0; e ? u64->float(e.68) (FUN_0023c078) * 3.390842e-9 : 0.
pub fn ee_sub_1fe3f8<M: QMem + ?Sized>(m: &mut M, a0: u32) -> f32 {
    let n = m.r32(a0 + 0x204);
    let e = if (n as i32) <= 0 { 0 } else { m.r32((n << 2).wrapping_add(a0) + 0x204) };
    if e == 0 {
        return 0.0;
    }
    let x = m.r64(e + 0x68);
    let f = if (x as i64) < 0 {
        let y = (x & 1) | (x >> 1);
        let r = m.call(0x0023_c078, &[], &[[fb(y as u32), fb((y >> 32) as u32), 0.0, 0.0]]).1[0];
        r + r
    } else {
        m.call(0x0023_c078, &[], &[[fb(x as u32), fb((x >> 32) as u32), 0.0, 0.0]]).1[0]
    };
    f * fb(0x3169_0453)
}

/// FUN_001b0078: if FUN_001d9880(a0) && +0x24 > 0: map last entry's +0x2c (0x2bfda8->0x1e, 0x2bfe08->0x1f,
/// 0x2bfe68->0x19) else -1.
pub fn ee_sub_1b0078<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    if m.call(0x001d_9880, &[a0], &[]).0 == 0 {
        return 0xffff_ffff;
    }
    let n = m.r32(a0 + 0x24);
    if (n as i32) <= 0 {
        return 0xffff_ffff;
    }
    let e = n.wrapping_sub(1).wrapping_mul(0x1c).wrapping_add(a0);
    let k = m.r32(e + 0x2c);
    if k == 0x002b_fda8 {
        0x1e
    } else if k == 0x002b_fe08 {
        0x1f
    } else if k == 0x002b_fe68 {
        0x19
    } else {
        0xffff_ffff
    }
}

/// FUN_001e01f8: allocate a 0x50 entry (cap +0x58, <0 -> 0x200) from +0x54/+0x50, zero it, [0]=a1, [1]=a2;
/// else returns 0x3da740.
pub fn ee_sub_1e01f8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) -> u32 {
    let mut cap = m.r32(a0 + 0x58);
    let n = m.r32(a0 + 0x50);
    if !(-1 < cap as i32) {
        cap = 0x200;
    }
    if !((n as i32) < cap as i32) {
        return 0x003d_a740;
    }
    let base = m.r32(a0 + 0x54);
    m.w32(a0 + 0x50, n.wrapping_add(1));
    let p = base.wrapping_add(n.wrapping_mul(0x50));
    m.call(0x0023_33b4, &[p, 0, 0x50], &[]);
    m.w32(p, a1);
    m.w32(p + 4, a2);
    p
}

/// FUN_001b0d90: FUN_001b17b8(a0; 1.0); if +0x16c: FUN_001b1328(a0,1); +0x174 = FUN_001e6610(+0x16c);
/// +0x178 = FUN_001e77e8(+0x16c); FUN_001e67b8(+0x16c, &q, 0; +0x174); vt(0xe8)(q) else FUN_001b1328(a0,0).
pub fn ee_sub_1b0d90<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let sp = stack_push(m, 0x30);
    m.call(0x001b_17b8, &[a0], &[f4(1.0)]);
    if m.r32(a0 + 0x16c) != 0 {
        m.call(0x001b_1328, &[a0, 1], &[]);
        let p = m.r32(a0 + 0x16c);
        let f = m.call(0x001e_6610, &[p], &[]).1[0];
        m.wf(a0 + 0x174, f);
        let p = m.r32(a0 + 0x16c);
        let f = m.call(0x001e_77e8, &[p], &[]).1[0];
        m.wf(a0 + 0x178, f);
        let p = m.r32(a0 + 0x16c);
        let t = m.rf(a0 + 0x174);
        m.call(0x001e_67b8, &[p, sp, 0], &[f4(t)]);
        let q = m.rv(sp);
        vcall(m, a0, 0xe8, &[], &[q]);
    } else {
        m.call(0x001b_1328, &[a0, 0], &[]);
    }
    stack_pop(m, 0x30);
}

/// FUN_001fc2e0(a0, i): if i < 6: f = FUN_0011d948(0x2bbb60); if (int)(f*1024) != (int)(a0.70[i]*1024):
/// +0x3bc |= 1<<i; a0.70[i] = f; FUN_001fb658(a0, 0x80, 0x80).
pub fn ee_sub_1fc2e0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if a1 >= 6 {
        return;
    }
    let f = m.call(0x0011_d948, &[0x002b_bb60], &[]).1[0];
    let p = a0 + 0x70 + (a1 << 2);
    let old = m.rf(p);
    if (f * 1024.0) as i32 == (old * 1024.0) as i32 {
        return;
    }
    let v = m.r32(a0 + 0x3bc) | (1u32 << (a1 & 31));
    m.w32(a0 + 0x3bc, v);
    m.wf(p, f);
    m.call(0x001f_b658, &[a0, 0x80, 0x80], &[]);
}

/// FUN_0022bf70: weighted average of FUN_0022be60 over +0xc entries (stride 0x10 from +0x10), shaped by
/// FUN_0022bf40(avg, p[2]); returns lerp(p[0], p[1]).
pub fn ee_sub_22bf70<M: QMem + ?Sized>(m: &mut M, a0: u32) -> f32 {
    let mut acc = 0.0f32;
    let mut wsum = 0.0f32;
    let mut i = 0i32;
    if (m.r32(a0 + 0xc) as i32) > 0 {
        let mut e = a0 + 0x10;
        loop {
            let x = m.rf(e + 4);
            i += 1;
            let y = m.rf(e + 8);
            let z = m.rf(e + 0xc);
            let r = m.call(0x0022_be60, &[], &[f4(x), f4(y), f4(z)]).1[0];
            let w = m.rf(e);
            e = e.wrapping_add(0x10);
            wsum += w;
            acc += w * r;
            if !(i < m.r32(a0 + 0xc) as i32) {
                break;
            }
        }
    }
    let p2 = m.rf(a0 + 8);
    let r = m.call(0x0022_bf40, &[], &[f4(acc / wsum), f4(p2)]).1[0];
    let lo = m.rf(a0);
    let hi = m.rf(a0 + 4);
    lo + r * (hi - lo)
}

/// FUN_001f8dd8: FUN_00134040(a0); +0x3f1c = &i; for i < +0x3f14: e = +0x3f20[i]; if (e.0 & 0x24) == 4:
/// e.1->vt(0x88)(); +0x3f1c = 0; FUN_001f8f80(a0,1); FUN_00134040(a0); FUN_001f8f80(a0,6).
pub fn ee_sub_1f8dd8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let sp = stack_push(m, 0x40);
    m.call(0x0013_4040, &[a0], &[]);
    m.w32(a0 + 0x3f1c, sp);
    m.w32(sp, 0);
    if (m.r32(a0 + 0x3f14) as i32) > 0 {
        let mut i = 0u32;
        loop {
            let e = m.r32(a0 + 0x3f20).wrapping_add(i << 3);
            if m.r32(e) & 0x24 == 4 {
                let o = m.r32(e + 4);
                vcall(m, o, 0x88, &[], &[]);
            }
            let v = m.r32(sp).wrapping_add(1);
            i = v;
            m.w32(sp, v);
            if !((i as i32) < m.r32(a0 + 0x3f14) as i32) {
                break;
            }
        }
    }
    m.w32(a0 + 0x3f1c, 0);
    m.call(0x001f_8f80, &[a0, 1], &[]);
    m.call(0x0013_4040, &[a0], &[]);
    m.call(0x001f_8f80, &[a0, 6], &[]);
    stack_pop(m, 0x40);
}

/// FUN_001f3118(a0; t): +0x38 = t; if +0x140 != -1: FUN_001d6310(); FUN_001d6378(a0); if !+0x10 && bit60(+0x28):
/// FUN_00147e88(GS, +0x14); for c = +0x64 .. c.24: if t < c.38: c->vt(0x150)(; t).
pub fn ee_sub_1f3118<M: QMem + ?Sized>(m: &mut M, a0: u32, t: f32) {
    m.wf(a0 + 0x38, t);
    if m.r32(a0 + 0x140) != 0xffff_ffff {
        m.call(0x001d_6310, &[a0], &[]);
        m.call(0x001d_6378, &[a0], &[]);
    }
    if m.r32(a0 + 0x10) == 0 && (m.r32(a0 + 0x2c) >> 28) & 1 != 0 {
        let x = m.r32(a0 + 0x14);
        m.call(0x0014_7e88, &[GS, x], &[]);
    }
    let mut c = m.r32(a0 + 0x64);
    while c != 0 {
        if t < m.rf(c + 0x38) {
            vcall(m, c, 0x150, &[], &[f4(t)]);
        }
        c = m.r32(c + 0x24);
    }
}

/// FUN_0011b938(a0 xform, a1 out3x3?, a2 out?): FUN_0011ba08(&r, a0[2], DAT_002a2dc0) -> 3 rows to a1 (or frame);
/// if a2: a2[i] = rows applied to a0[i] (xyz; w kept).
pub fn ee_sub_11b938<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    let sp = stack_push(m, 0xd0);
    let s0 = if a1 != 0 { a1 } else { sp };
    let q1 = m.rv(a0 + 0x20);
    let q2 = m.rv(0x002a_2dc0);
    m.call(0x0011_ba08, &[sp + 0x30], &[q1, q2]);
    let r0 = m.rv(sp + 0x30);
    m.wv(s0, r0);
    let r1 = m.rv(sp + 0x40);
    m.wv(s0 + 0x10, r1);
    let r2 = m.rv(sp + 0x50);
    m.wv(s0 + 0x20, r2);
    if a2 != 0 {
        let xf = |v: V4| -> V4 {
            [
                r0[0] * v[0] + r1[0] * v[1] + r2[0] * v[2],
                r0[1] * v[0] + r1[1] * v[1] + r2[1] * v[2],
                r0[2] * v[0] + r1[2] * v[1] + r2[2] * v[2],
                v[3],
            ]
        };
        let v1 = xf(m.rv(a0 + 0x20));
        let v2 = xf(m.rv(a0));
        let v3 = xf(m.rv(a0 + 0x10));
        m.wv(a2 + 0x20, v1);
        m.wv(a2, v2);
        m.wv(a2 + 0x10, v3);
        m.wv(sp + 0x60, v2);
        m.wv(sp + 0x30, v2);
        m.wv(sp + 0x70, v3);
        m.wv(sp + 0x40, v3);
        m.wv(sp + 0x80, v1);
        m.wv(sp + 0x50, v1);
    }
    stack_pop(m, 0xd0);
}

/// FUN_001b1a38(a0, a1, a2): if a1 && (c = FUN_0012ff10(a1, a0, 1, 1)): p = c[1]; p->vt(0)(&x0, 0; 0, 1);
/// p->vt(0)(&x1, 0; a1.74, 1); *a2 = |x1 - x0| / a1.74 * a1.78.
pub fn ee_sub_1b1a38<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    if a1 == 0 {
        return;
    }
    let sp = stack_push(m, 0x70);
    let c = m.call(0x0012_ff10, &[a1, a0, 1, 1], &[]).0;
    if c != 0 {
        let p = m.r32(c + 4);
        let vt = m.r32(p);
        let adj = m.r16(vt + 0x18) as i16 as i32 as u32;
        let f = m.r32(vt + 0x1c);
        m.call(f, &[p.wrapping_add(adj), sp, 0, p], &[f4(0.0), f4(1.0)]);
        let p = m.r32(c + 4);
        let t = m.rf(a1 + 0x74);
        let vt = m.r32(p);
        let adj = m.r16(vt + 0x18) as i16 as i32 as u32;
        let f = m.r32(vt + 0x1c);
        m.call(f, &[p.wrapping_add(adj), sp + 0x10, 0, p], &[f4(t), f4(1.0)]);
        let d = (m.rf(sp + 0x10) - m.rf(sp)).abs();
        let r = d / m.rf(a1 + 0x74) * m.rf(a1 + 0x78);
        m.wf(a2, r);
    }
    stack_pop(m, 0x70);
}

/// FUN_001b1f40(a0, a1): r = FUN_0013bbf8(a0, a1); for each +0x23c[i] == a1.18: r = FUN_00134310(r, 1);
/// for each +0x250[i]: if FUN_00132b10(a1, it): r = FUN_00134310(r, 1). Returns r.
pub fn ee_sub_1b1f40<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let mut r = m.call(0x0013_bbf8, &[a0, a1], &[]).0;
    if (m.r32(a0 + 0x238) as i32) > 0 {
        let mut i = 0i32;
        let mut p = a0 + 0x23c;
        loop {
            if m.r32(p) == m.r32(a1 + 0x18) {
                r = m.call(0x0013_4310, &[r, 1], &[]).0;
            }
            i += 1;
            p += 4;
            if !(i < m.r32(a0 + 0x238) as i32) {
                break;
            }
        }
    }
    if (m.r32(a0 + 0x24c) as i32) > 0 {
        let mut i = 0i32;
        let mut p = a0 + 0x250;
        loop {
            let x = m.r32(p);
            if m.call(0x0013_2b10, &[a1, x], &[]).0 != 0 {
                r = m.call(0x0013_4310, &[r, 1], &[]).0;
            }
            i += 1;
            p += 4;
            if !(i < m.r32(a0 + 0x24c) as i32) {
                break;
            }
        }
    }
    r
}

/// FUN_001ad168: state +0x40 dispatch: 0x15 / 0x12 / 0x20 -> FUN_001aaf90(0x2f633c, DAT) + FUN_001acb58(a0, next);
/// 0x1c -> FUN_001ad3e0(a0).
pub fn ee_sub_1ad168<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let s = m.r32(a0 + 0x40);
    let (d, n) = match s {
        0x15 => (0x002a_6d88, 0x16),
        0x12 => (0x002a_6e88, 0x13),
        0x20 => (0x002a_6e90, 0x21),
        0x1c => {
            m.call(0x001a_d3e0, &[a0], &[]);
            return;
        }
        _ => return,
    };
    let x = m.r32(d);
    m.call(0x001a_af90, &[0x002f_633c, x], &[]);
    m.call(0x001a_cb58, &[a0, n], &[]);
}

/// FUN_001fc408(a0, i, a2, a3; f12): if i < 6 && cnt[i] (+0xa0) < 8: FUN_001fb658(a0, 0x200, 0x200);
/// e = a0 + i*0x80 + 0xb8 + cnt*16; cnt++; e = {a2, *(u64*)a3, f12}; if e.4 == e.8: e.8 += 1.
pub fn ee_sub_1fc408<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, f12: f32) {
    if a1 >= 6 {
        return;
    }
    let pc = a0 + 0xa0 + (a1 << 2);
    if !((m.r32(pc) as i32) < 8) {
        return;
    }
    m.call(0x001f_b658, &[a0, 0x200, 0x200], &[]);
    let c = m.r32(pc);
    let e = a0.wrapping_add((a1 << 7) + 0xb8).wrapping_add(c << 4);
    m.w32(pc, c.wrapping_add(1));
    m.w32(e, a2);
    m.wf(e + 0xc, f12);
    for k in 0..8 {
        let b = m.r8(a3 + k);
        m.w8(e + 4 + k, b);
    }
    let x = m.rf(e + 4);
    let y = m.rf(e + 8);
    if x == y {
        m.wf(e + 8, y + 1.0);
    }
}

/// FUN_00173928(a0, p, mode, flag): p[4] = mode; mode 0: p[2] = now; (*p)->vt(0x48)(); if !(flag&1): p[3].18 = now;
/// mode 1: p[2] = now + a0.a0; p[8..12] = FUN_00173a18(a0) (qword); if !(flag&1): p[3].18 = now + a0.a4;
/// then *p = 0; p[3].14 = 0; p[3] = 0; a0.58--.
pub fn ee_sub_173928<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32) {
    m.w32(a1 + 0x10, a2);
    if a2 == 0 {
        let now = m.rf(0x002d_ded0);
        let o = m.r32(a1);
        m.wf(a1 + 8, now);
        vcall(m, o, 0x48, &[], &[]);
        if (a3 ^ 1) & 1 != 0 {
            let f = m.rf(0x002d_ded0);
            let q = m.r32(a1 + 0xc);
            m.wf(q + 0x18, f);
        }
    } else if a2 == 1 {
        let x = m.rf(a0 + 0xa0);
        let now = m.rf(0x002d_ded0);
        m.wf(a1 + 8, now + x);
        let r = m.call(0x0017_3a18, &[a0], &[]).1;
        m.wv(a1 + 0x20, r);
        if (a3 ^ 1) & 1 != 0 {
            let f = m.rf(0x002d_ded0) + m.rf(a0 + 0xa4);
            let q = m.r32(a1 + 0xc);
            m.wf(q + 0x18, f);
        }
    }
    let q = m.r32(a1 + 0xc);
    m.w32(a1, 0);
    m.w32(q + 0x14, 0);
    m.w32(a1 + 0xc, 0);
    let v = m.r32(a0 + 0x58).wrapping_sub(1);
    m.w32(a0 + 0x58, v);
}

/// FUN_001dedb0: for 2 slots e (+0x1a0, stride 0x10): if e[0]: FUN_0015e9c8(GS, 2, 0x6d, e0, 1, &e[1]);
/// if e[1]: e[2] = FUN_001f6788(e0, e1.18) || FUN_00130488(e1, e0, 8; 0, 1); if e[2]:
/// c = FUN_0012ff10(e1, e0, 0xc, 1); if c: e[3] = c[1].
pub fn ee_sub_1dedb0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let mut e = a0 + 0x1a0;
    let mut s3 = a0 + 0x1a4;
    let mut i = 0u32;
    loop {
        let s1 = m.r32(e);
        'b: {
            if s1 == 0 {
                break 'b;
            }
            m.call(0x0015_e9c8, &[GS, 2, 0x6d, s1, 1, s3], &[]);
            let s0 = m.r32(e + 4);
            if s0 == 0 {
                break 'b;
            }
            let k = m.r32(s0 + 0x18);
            let v = m.call(0x001f_6788, &[s1, k], &[]).0;
            m.w32(e + 8, v);
            if v == 0 {
                let v = m.call(0x0013_0488, &[s0, s1, 8], &[f4(0.0), f4(1.0)]).0;
                m.w32(e + 8, v);
                if v == 0 {
                    break 'b;
                }
            }
            let c = m.call(0x0012_ff10, &[s0, s1, 0xc, 1], &[]).0;
            if c != 0 {
                let v = m.r32(c + 4);
                m.w32(e + 0xc, v);
            }
        }
        i += 1;
        s3 += 0x10;
        e += 0x10;
        if i >= 2 {
            break;
        }
    }
}

fn u64f<M: QMem + ?Sized>(m: &mut M, x: u64) -> f32 {
    if (x as i64) < 0 {
        let y = (x & 1) | (x >> 1);
        let r = m.call(0x0023_c078, &[], &[[fb(y as u32), fb((y >> 32) as u32), 0.0, 0.0]]).1[0];
        r + r
    } else {
        m.call(0x0023_c078, &[], &[[fb(x as u32), fb((x >> 32) as u32), 0.0, 0.0]]).1[0]
    }
}

/// FUN_001fe888: o = +0x200; !o -> 0; t = u64f(o.68)*k; if t == 0 && !o.40 && !o.52 && !(o.50&1): +0x24c;
/// else u64f(o.68)*k.
pub fn ee_sub_1fe888<M: QMem + ?Sized>(m: &mut M, a0: u32) -> f32 {
    let o = m.r32(a0 + 0x200);
    if o == 0 {
        return 0.0;
    }
    let x = m.r64(o + 0x68);
    let t = u64f(m, x) * fb(0x3169_0453);
    let o = m.r32(a0 + 0x200);
    if t == 0.0 {
        let c = m.r32(o + 0x40) == 0 && m.r16(o + 0x52) == 0 && m.r16(o + 0x50) & 1 == 0;
        if c {
            return m.rf(a0 + 0x24c);
        }
    }
    let x = m.r64(o + 0x68);
    u64f(m, x) * fb(0x3169_0453)
}

/// FUN_001b7d20: FUN_0012e460 on 12 handle slots; if +0x13f0: vt(0x18)(6,0,0,0,0); a = +0x13f0.180;
/// FUN_002303a0(a, a.58, +0x1404); FUN_002303a0(a, a.5c, +0x1408); FUN_001f6860(a,0); FUN_001a58d0(a,0,0xf);
/// +0x1430 = a; +0x1434 = now; FUN_00134320(GS, a0).
pub fn ee_sub_1b7d20<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    for off in [0x13f4, 0x13f8, 0x13fc, 0x1400, 0x1600, 0x1604, 0x1608, 0x160c, 0x1618, 0x161c, 0x1620, 0x1624] {
        m.call(0x0012_e460, &[a0 + off], &[]);
    }
    let o = m.r32(a0 + 0x13f0);
    if o == 0 {
        return;
    }
    vcall(m, o, 0x18, &[6, 0, 0, 0, 0], &[]);
    let o = m.r32(a0 + 0x13f0);
    let s0 = m.r32(o + 0x180);
    let (x, y) = (m.r32(s0 + 0x58), m.r32(a0 + 0x1404));
    m.call(0x0023_03a0, &[s0, x, y], &[]);
    let (x, y) = (m.r32(s0 + 0x5c), m.r32(a0 + 0x1408));
    m.call(0x0023_03a0, &[s0, x, y], &[]);
    m.call(0x001f_6860, &[s0, 0], &[]);
    m.call(0x001a_58d0, &[s0, 0, 0xf], &[]);
    m.w32(a0 + 0x1430, s0);
    let f = m.rf(0x002d_ded0);
    m.wf(a0 + 0x1434, f);
    m.call(0x0013_4320, &[GS, a0], &[]);
}

/// FUN_001e7538(a0, out_frac, out_span; t): t' = FUN_001e6660(a0; t); walk cached key index +0x178 in key table
/// +0xec (count +0xe8) to the segment holding t'; writes t'-key[i], key[i+1]-key[i]; returns/stores i.
pub fn ee_sub_1e7538<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, t: f32) -> i32 {
    let f1 = m.call(0x001e_6660, &[a0], &[f4(t)]).1[0];
    let mut i = m.r32(a0 + 0x178) as i32;
    let tbl = m.r32(a0 + 0xec);
    let p = ((i as u32) << 2).wrapping_add(tbl);
    let f0 = m.rf(p);
    if f0 < f1 {
        let lim = (m.r32(a0 + 0xe8) as i32).wrapping_sub(1);
        if i < lim {
            if f1 < f0 {
                i -= 1;
            } else {
                let mut q = p;
                i += 1;
                loop {
                    if !(i < lim) {
                        break;
                    }
                    q = q.wrapping_add(4);
                    let v = m.rf(q);
                    if f1 < v {
                        break;
                    }
                    i += 1;
                }
                i -= 1;
            }
        } else {
            i -= 1;
        }
    } else if i > 0 && !(f0 <= f1) {
        let mut q = p;
        loop {
            i -= 1;
            if i <= 0 {
                break;
            }
            q = q.wrapping_sub(4);
            let v = m.rf(q);
            if v <= f1 {
                break;
            }
        }
    }
    let o = (i as u32) << 2;
    let f2 = m.rf(o.wrapping_add(tbl));
    if a1 != 0 {
        m.wf(a1, f1 - f2);
    }
    if a2 != 0 {
        let t2 = m.r32(a0 + 0xec);
        let n = m.rf(o.wrapping_add(t2) + 4);
        m.wf(a2, n - f2);
    }
    m.w32(a0 + 0x178, i as u32);
    i
}

/// shared body of FUN_001a2a78 / 001b45d0 / 001ff690 / 002289f0 (identical code): on (a0 != 0, a1 == 0xffff)
/// bump six pool refcounts DAT_002dd2e0..f4 and FUN_0011cca8-init each pool on first use.
fn pool_init<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if a1 != 0xffff || a0 == 0 {
        return;
    }
    const P: [(u32, u32, u32, u32); 6] = [
        (0x002d_d2e0, 0x0045_20a8, 0x14, 0x2000),
        (0x002d_d2e4, 0x0045_20d0, 0x18, 0x800),
        (0x002d_d2e8, 0x0045_20f8, 0x28, 0x200),
        (0x002d_d2ec, 0x0045_2120, 0x1c, 0x800),
        (0x002d_d2f0, 0x0045_2148, 0xc, 0xc8),
        (0x002d_d2f4, 0x0045_2170, 0x14, 0x100),
    ];
    for (c, pool, sz, n) in P {
        let v = m.r32(c).wrapping_add(1);
        m.w32(c, v);
        if v == 1 {
            m.call(0x0011_cca8, &[pool, sz, n, 0x10], &[]);
        }
    }
}

/// FUN_001a2a78: static pool init (see `pool_init`).
pub fn ee_sub_1a2a78<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    pool_init(m, a0, a1)
}

/// FUN_001b45d0: static pool init (see `pool_init`).
pub fn ee_sub_1b45d0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    pool_init(m, a0, a1)
}

/// FUN_001ff690: static pool init (see `pool_init`).
pub fn ee_sub_1ff690<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    pool_init(m, a0, a1)
}

/// FUN_002289f0: static pool init (see `pool_init`).
pub fn ee_sub_2289f0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    pool_init(m, a0, a1)
}

/// FUN_0019faf8(a0, a1): pick the best (lowest FUN_0019f7d0 score) 0x20-byte entry of a0[4..] into a1;
/// none found -> zero a1, a1+0x10 = FUN_001a62e8(*a0) + (*a0)+0x240.
pub fn ee_sub_19faf8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let sp = stack_push(m, 0x40);
    let s4 = m.call(0x0015_e9c8, &[GS, 0x505, 0x33, 0, 0x10, sp], &[]).0;
    let mut best = m.rf(0x002a_66a0);
    let mut i: i32 = 0;
    if (m.r32(a0 + 4) as i32) > 0 {
        let mut s0 = a0 + 0x10;
        loop {
            let f = m.call(0x0019_f7d0, &[a0, s0, s4, sp], &[]).1[0];
            if f < best {
                best = f;
                let q = m.rv(s0);
                m.wv(a1, q);
                let q = m.rv(s0 + 0x10);
                m.wv(a1 + 0x10, q);
                if f == 0.0 {
                    stack_pop(m, 0x40);
                    return;
                }
            }
            i += 1;
            s0 = s0.wrapping_add(0x20);
            if i >= m.r32(a0 + 4) as i32 {
                break;
            }
        }
    }
    if best == m.rf(0x002a_66a0) {
        m.call(0x0023_33b4, &[a1, 0, 0x20], &[]);
        let o = m.r32(a0);
        let p = m.call(0x001a_62e8, &[o], &[]).1;
        let o = m.r32(a0);
        let b = m.rv(o + 0x240);
        m.wv(a1 + 0x10, [p[0] + b[0], p[1] + b[1], p[2] + b[2], p[3] + b[3]]);
    }
    stack_pop(m, 0x40);
}

/// FUN_00197370(a0): classify the heading delta into a0[2] (1..4), pick an anim from a0[0x48..]/a0[0x4d..]
/// and FUN_001f15b8 / FUN_001305b8 it onto *a0.
pub fn ee_sub_197370<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.w32(a0 + 0x30, 1);
    let o = m.r32(a0);
    let p = m.r32(o + 0x60);
    let (x, y) = (m.rf(p + 0x44), m.rf(p + 0x40));
    let h = m.call(0x0024_58f0, &[], &[f4(x), f4(y)]).1[0];
    let o = m.r32(a0);
    let p = m.r32(o + 0x150);
    let d = m.rf(p + 0x18) - h;
    let f = m.call(0x0011_cf60, &[], &[f4(d)]).1[0];
    let af = f.abs();
    let k = if af < fb(0x3f8c_be4c) {
        3
    } else if fb(0x4002_b0b4) < af {
        4
    } else if 0.0 < f {
        2
    } else {
        1
    };
    m.w32(a0 + 8, k);
    let o = m.r32(a0);
    let a1 = m.r32(a0 + 0x1c);
    let s2 = m.r32(o + 0x1030);
    let idx = m.r32(a0 + 8) << 2;
    let mut s0 = if a1 == 2 { m.r32(a0.wrapping_add(idx) + 0x134) } else { m.r32(a0.wrapping_add(idx) + 0x120) };
    if s0 == 0 {
        s0 = if a1 != 2 { m.r32(a0 + 0x120) } else { m.r32(a0 + 0x134) };
        if s0 == 0 {
            s0 = m.r32(a0 + 0x120);
        }
    }
    m.call(0x001f_15b8, &[o + 0x1034], &[]);
    let o = m.r32(a0);
    let (x, y) = (m.rf(s2 + 0x1c), m.rf(s2 + 0x40));
    m.call(0x0013_05b8, &[s0, o, 1, o + 0x1030], &[f4(x), f4(y)]);
}

/// FUN_001da5c8(a0, a1): state change of an anim/sfx driver (+0x10 state, +0x14 time, +0x28 target).
pub fn ee_sub_1da5c8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let mut s1 = a1;
    let cur = m.r32(a0 + 0x10);
    if s1 == cur {
        return;
    }
    if cur == 3 {
        if s1 == 1 && m.rf(a0 + 0x14) == m.rf(0x002d_ded0) {
            s1 = 2;
        }
    } else if cur == 4 {
        let v = m.r32(a0 + 0x20);
        let f = m.rf(a0 + 0x24);
        m.w32(a0 + 0x18, v);
        m.wf(a0 + 0x1c, f);
        m.w32(a0 + 0x20, 0xffff_ffff);
        m.wf(a0 + 0x24, -1.0);
    }
    if s1 == 0 {
        m.w32(a0 + 0x18, 0xffff_ffff);
    }
    let p = m.r32(a0 + 0x28);
    if p != 0 {
        if m.r32(a0 + 0x10) == 4 && s1 == 2 {
            m.call(0x0021_3830, &[p, 0x310], &[]);
        }
        if s1 == 3 {
            let o = m.r32(a0 + 0x28);
            vcall(m, o, 0x190, &[], &[]);
        } else if (s1 as i32) < 4 {
            if s1 == 1 {
                let o = m.r32(a0 + 0x28);
                m.call(0x0021_2880, &[o, 1, a0 + 0x18], &[]);
            }
        } else if s1 == 4 {
            let o = m.r32(a0 + 0x28);
            m.call(0x0021_3830, &[o, 0x312], &[]);
        }
    }
    m.w32(a0 + 0x10, s1);
    let t = m.rf(0x002d_ded0);
    m.wf(a0 + 0x14, t);
}

/// FUN_00210cf0(a0, a1): only for a1 == 0xffff: a0 == 0 -> FUN_00258348(GS, 2); else FUN_0020d9d0(GS),
/// reset the 0x3e1c20 header and bump six pool refcounts (first ref -> FUN_0011cca8 pool init).
pub fn ee_sub_210cf0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if a1 != 0xffff {
        return;
    }
    if a0 == 0 {
        m.call(0x0025_8348, &[GS, 2], &[]);
        return;
    }
    m.call(0x0020_d9d0, &[GS], &[]);
    let c = m.r32(0x002d_d2e0).wrapping_add(1);
    m.w32(0x003e_1c20, 0);
    m.w32(0x003e_1c2c, 0x003e_1c60);
    m.w32(0x003e_1c24, 0x002c_3eb8);
    m.w32(0x002d_d2e0, c);
    m.w32(0x003e_1c28, 0);
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
        let c = m.r32(cnt).wrapping_add(1);
        m.w32(cnt, c);
        if c == 1 {
            m.call(0x0011_cca8, &[pool, sz, n, 0x10], &[]);
        }
    }
}

/// FUN_00186670(a0, a1): n = FUN_00117bd0(a1); if n: set bit 36 of the 64-bit +0x148 flags, then n times
/// allocate a 0xe0 record (FUN_001ae2a0), fill it from the stream a1 via vslot +0x198 lookups and
/// FUN_00117eb0, FUN_00186930 it and FUN_0012dd28-link it twice.
pub fn ee_sub_186670<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let sp = stack_push(m, 0x10);
    let n = m.call(0x0011_7bd0, &[a1], &[]).0;
    if n != 0 {
        let hi = m.r32(a0 + 0x14c) | 0x10;
        m.w32(a0 + 0x14c, hi);
        if (n as i32) > 0 {
            let mut s5 = n;
            loop {
                m.w32(sp, 0xe0);
                m.w32(sp + 4, 0x10);
                s5 = s5.wrapping_sub(1);
                let s2 = m.call(0x001a_e2a0, &[sp], &[]).0;
                for off in [4u32, 8, 0xc] {
                    let vt = m.r32(a0 + 0xc);
                    let adj = m.r16(vt + 0x198) as i16 as i32 as u32;
                    let x = m.call(0x0011_7d50, &[a1], &[]).0;
                    let f = m.r32(vt + 0x19c);
                    let r = m.call(f, &[a0.wrapping_add(adj), x], &[]).0;
                    m.w32(s2 + off, r);
                }
                m.call(0x0011_7eb0, &[a1, s2 + 0xb0], &[]);
                let vt = m.r32(a0 + 0xc);
                let adj = m.r16(vt + 0x198) as i16 as i32 as u32;
                let x = m.call(0x0011_7d50, &[a1], &[]).0;
                let f = m.r32(vt + 0x19c);
                let r = m.call(f, &[a0.wrapping_add(adj), x], &[]).0;
                m.w32(s2, r);
                m.call(0x0011_7eb0, &[a1, s2 + 0xa0], &[]);
                m.call(0x0018_6930, &[s2], &[]);
                let x = m.r32(s2 + 4);
                m.call(0x0012_dd28, &[a0, x, 0xffff_ffff, s2], &[]);
                let x = m.r32(s2 + 8);
                m.call(0x0012_dd28, &[a0, x, 0xffff_ffff, s2], &[]);
                if s5 == 0 {
                    break;
                }
            }
        }
    }
    stack_pop(m, 0x10);
}

/// FUN_001b8b70(a0): +0x190 = -FUN_00204e38(+0x1030.+0x1c); with +0x1818 blend it toward
/// -FUN_001a6fc0(0, a0, FUN_0011e1a0(..)) by a clamped quadratic of the anim phase; then FUN_002046f0 when
/// +0x10dc && +0x10e4 == 4.
pub fn ee_sub_1b8b70<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let p = m.r32(a0 + 0x1030);
    if p != 0 {
        let x = m.rf(p + 0x1c);
        let v = m.call(0x0020_4e38, &[], &[f4(x)]).1;
        m.wv(a0 + 0x190, [v[0] * -1.0, v[1] * -1.0, v[2] * -1.0, v[3] * -1.0]);
        if m.r32(a0 + 0x1818) != 0 {
            let p = m.r32(a0 + 0x1030);
            let x = m.r32(p + 0x14);
            let s0 = m.call(0x0012_f938, &[x, 0x240], &[]).0;
            let p = m.r32(a0 + 0x1030);
            let x = m.r32(p + 0x14);
            let q = m.call(0x0012_f938, &[x, 0x242], &[]).0;
            let p = m.r32(a0 + 0x1030);
            let lo = m.rf(s0);
            let t = (m.rf(p + 0x1c) - lo) / (m.rf(q) - lo);
            let c = m.rf(0x002c_0a20) + t * (m.rf(0x002c_0a24) + t * m.rf(0x002c_0a28));
            let (mn, mx) = (m.rf(0x002b_bb60), m.rf(0x002b_bb64));
            let w = if c < mn {
                mn
            } else if mx < c {
                mx
            } else {
                c
            };
            let o = m.r32(a0 + 0x150);
            let u = m.rv(a0 + 0x1700);
            let len = (u[0] * u[0] + u[1] * u[1]).sqrt();
            let (h, s) = (m.rf(o + 0x18), m.rf(o + 0x1c));
            let r = m.call(0x0011_e1a0, &[], &[f4(h), f4(s * len), f4(0.0)]).1;
            let g = m.call(0x001a_6fc0, &[0, a0], &[r]).1;
            let ng = [g[0] * -1.0, g[1] * -1.0, g[2] * -1.0, g[3] * -1.0];
            let b = m.rv(a0 + 0x190);
            let k = 1.0 - w;
            m.wv(a0 + 0x190, [b[0] * k + ng[0] * w, b[1] * k + ng[1] * w, b[2] * k + ng[2] * w, b[3] * k + ng[3] * w]);
        }
    }
    if m.r32(a0 + 0x10dc) != 0 && m.r32(a0 + 0x10e4) == 4 {
        m.call(0x0020_46f0, &[a0], &[]);
    }
}

/// FUN_00165e38(a0): look up a0 in the 0x3e0fac table (FUN_002581c0 / FUN_00129ec8); dispatch a
/// FUN_0022b490 message (or FUN_001d54b8 + FUN_001dd888 when FUN_001d9880(0x3da530) and the value is 0).
pub fn ee_sub_165e38<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let sp = stack_push(m, 0x10);
    let t = m.r32(GS + 0x3b1c);
    m.call(0x0025_81c0, &[sp, t, a0], &[]);
    let f20 = m.rf(sp + 4);
    m.call(0x0012_9ec8, &[sp], &[]);
    let s0 = 0x003d_a530;
    let r = m.call(0x001d_9880, &[s0], &[]).0;
    if r != 0 {
        if f20 == 0.0 {
            m.call(0x001d_54b8, &[], &[]);
            m.call(0x001d_d888, &[s0], &[]);
            stack_pop(m, 0x10);
            return;
        }
    } else if f20 == 0.0 {
        m.w32(0x002d_eb4c, a0);
        m.w32(0x002d_eb48, 0x62f);
        m.w32(0x002d_eb44, 0xffff_ffff);
        m.w32(0x002d_eb40, 0xffff_ffff);
        let v = m.call(0x0021_0b98, &[0], &[]).0;
        m.call(0x0022_b490, &[GS, v, 0x189, 0xffff_ffff, 0x48, 0xffff_ffff], &[]);
        stack_pop(m, 0x10);
        return;
    }
    let t = m.r32(GS + 0x3b28);
    m.w32(0x002d_eb48, 0xffff_ffff);
    m.w32(0x002d_eb4c, a0);
    m.w32(0x002d_eb40, 0xffff_ffff);
    m.w32(0x002d_eb44, 0xffff_ffff);
    m.call(0x0025_81c0, &[sp, t, a0], &[]);
    let k = m.r32(sp + 4);
    m.call(0x0012_9ec8, &[sp], &[]);
    let mut s2 = m.call(0x0021_0b98, &[k], &[]).0;
    let e = m.r32((GS + 0x3b7c).wrapping_add(k << 2));
    let mut s0: u32 = 0;
    if e != 0 {
        m.call(0x0025_81c0, &[sp, e, a0], &[]);
        s0 = m.r32(sp + 4);
        m.call(0x0012_9ec8, &[sp], &[]);
    }
    if !((s0 as i32) < 100) || s2 == 0 {
        s2 = 0x002a_3828;
    }
    m.call(0x0022_b490, &[GS, s2, 0x189, 0xffff_ffff, 0x40, 0xffff_ffff], &[]);
    stack_pop(m, 0x10);
}

/// FUN_001ca118(a0, a1, a2): rebuild a0's +0x3c id list with a1 prepended (FUN_001c99d8), move ownership
/// (+0x10) of the entries from the old to the new list; with a2, notify every child (vtable +0xe0 list)
/// through vtable +0xc8 (a1, 1) inside a FUN_001ae478 / FUN_001ae4a0 bracket.
pub fn ee_sub_1ca118<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    let sp = stack_push(m, 0x50);
    let mut s0 = m.r32(a0 + 0x3c);
    let mut n: u32 = 0x10;
    if s0 != 0 {
        let mut v = m.r32(s0);
        if v != 0xffff_ffff {
            let mut p = sp + 0x40;
            loop {
                p = p.wrapping_sub(4);
                m.w32(p, v);
                s0 = m.r32(s0 + 4);
                n = n.wrapping_sub(1);
                if s0 == 0 {
                    break;
                }
                v = m.r32(s0);
                if v == 0xffff_ffff {
                    break;
                }
            }
        }
    }
    n = n.wrapping_sub(1);
    let p = sp.wrapping_add(n << 2);
    let t = m.r32(GS + 0x3f0c);
    m.w32(p, a1);
    let s6 = m.call(0x001c_99d8, &[t, 0x10u32.wrapping_sub(n), p], &[]).0;
    let mut s0 = m.r32(a0 + 0x3c);
    while m.r32(s0) != 0xffff_ffff && m.r32(s0 + 0x10) == a0 {
        m.w32(s0 + 0x10, 0);
        s0 = m.r32(s0 + 4);
    }
    let mut s0 = s6;
    loop {
        let id = m.r32(s0);
        if id == 0xffff_ffff {
            break;
        }
        let i = m.call(0x001c_9950, &[id], &[]).0;
        let tab = m.r32(GS + 0x3f08);
        if m.r32(tab.wrapping_add(i.wrapping_mul(0x6c))) != 1 {
            break;
        }
        if m.r32(s0 + 0x10) != 0 {
            break;
        }
        m.w32(s0 + 0x10, a0);
        s0 = m.r32(s0 + 4);
    }
    m.w32(a0 + 0x3c, s6);
    if a2 != 0 {
        m.call(0x001a_e478, &[sp + 0x40], &[]);
        let cnt = vcall(m, a0, 0xe0, &[sp + 0x44], &[]).0;
        let mut i: i32 = 0;
        if (cnt as i32) > 0 {
            loop {
                let arr = m.r32(sp + 0x44);
                let o = m.r32(arr.wrapping_add((i as u32) << 2));
                i += 1;
                vcall(m, o, 0xc8, &[a1, 1], &[]);
                if !(i < cnt as i32) {
                    break;
                }
            }
        }
        m.call(0x001a_e4a0, &[sp + 0x40, 2], &[]);
    }
    stack_pop(m, 0x50);
}

/// FUN_001997d8(a0; a1q, a2q): build a probe (FUN_0011e2d8 / FUN_00256d18 / FUN_001995d8), then test every
/// child's collision list via FUN_0014c6a0 over 11 directions; 0 on an early hit, else (hits > 0.7) >= 2.
pub fn ee_sub_1997d8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: V4, a2: V4) -> u32 {
    let sp = stack_push(m, 0x240);
    let q1 = m.rv(0x002a_2dc0);
    let q2 = m.rv(0x002a_2db0);
    m.wv(sp, a2);
    m.call(0x0011_e2d8, &[sp + 0x10], &[a2, q1, q2]);
    let s0 = sp + 0x30;
    m.call(0x0025_6d18, &[sp + 0x60], &[a1]);
    for o in [0u32, 0x10, 0x20] {
        let q = m.rv(sp + o);
        m.wv(s0 + o, q);
    }
    m.w32(sp + 0x5c, 0);
    m.w32(sp + 0x4c, 0);
    m.w32(sp + 0x3c, 0);
    m.wf(s0 + 0xc, 1.0);
    m.call(0x0019_95d8, &[a0, s0, 0x16, sp + 0x70], &[]);
    let o = m.r32(a0);
    let p = m.r32(o + 0xec);
    let mut s4 = m.r32(p + 0x64);
    let mut fp: i32 = 0;
    if s4 == 0 {
        stack_pop(m, 0x240);
        return ((fp < 2) as u32) ^ 1;
    }
    loop {
        let c = m.r32(s4 + 0xc);
        let e = m.r32(c + 0xec);
        if m.r32(e + 0x70) & 4 == 0 {
            let mut s2: u32 = 0;
            let mut s3 = sp + 0x70;
            loop {
                let c = m.r32(s4 + 0xc);
                let e = m.r32(c + 0xec);
                let mut s0 = m.r32(e + 0x34);
                if s0 != 0 {
                    let s1 = (sp + 0x80).wrapping_add(s2 << 4);
                    let s5 = (s2 as i32) < 4;
                    loop {
                        if m.r32(s0 + 0x14) & 0x1_0000 == 0 {
                            let qa = m.rv(s3);
                            let qb = m.rv(s1);
                            let r = m.call(0x0014_c6a0, &[s0, 1, sp + 0x1d0], &[qa, qb]).0;
                            if (r as i32) > 0 {
                                if s5 {
                                    stack_pop(m, 0x240);
                                    return 0;
                                }
                                if m.rf(sp + 0x210) == 0.0 {
                                    stack_pop(m, 0x240);
                                    return 0;
                                }
                                if fb(0x3f33_3333) < m.rf(sp + 0x1f8) {
                                    fp += 1;
                                }
                            }
                        }
                        s0 = m.r32(s0 + 0x60);
                        if s0 == 0 {
                            break;
                        }
                    }
                }
                s2 += 2;
                s3 = s3.wrapping_add(0x20);
                if s2 >= 0x16 {
                    break;
                }
            }
        }
        s4 = m.r32(s4 + 4);
        if s4 == 0 {
            break;
        }
    }
    stack_pop(m, 0x240);
    ((fp < 2) as u32) ^ 1
}

/// FUN_001fd830(a0): per-frame update: +0x1f8 = +0x1f0 (u64), +0x1f0 = FUN_0014d240(0x2dded0) (u64: low = v0,
/// high = the callee's +0x58 wrap count), sub-updates, enable/disable the three focus objects
/// (FUN_001fb060 / FUN_001fb018) on DAT_002bd4c0 == 3, FUN_0011d018 fade into FUN_001fea60 x3, then
/// FUN_001fb8e0 the +0x4c chain and FUN_00231c48.
pub fn ee_sub_1fd830<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let sp = stack_push(m, 0x10);
    let (lo, hi) = (m.r32(a0 + 0x1f0), m.r32(a0 + 0x1f4));
    m.w32(a0 + 0x1f8, lo);
    m.w32(a0 + 0x1fc, hi);
    let v = m.call(0x0014_d240, &[0x002d_ded0], &[]).0;
    let hi = m.r32(0x002d_ded0 + 0x58);
    m.w32(a0 + 0x1f0, v);
    m.w32(a0 + 0x1f4, hi);
    m.call(0x001f_da48, &[a0], &[]);
    m.call(0x001f_dba8, &[a0], &[]);
    let k = m.r32(a0 + 0x204);
    let a1 = if (k as i32) <= 0 { 0 } else { m.r32((k << 2).wrapping_add(a0) + 0x204) };
    let st = m.r32(0x002b_d4c0);
    let f58 = m.r32(a0 + 0x58);
    if st == 3 {
        if f58 != 0 {
            if a1 != 0 {
                m.call(0x001f_b060, &[a1], &[]);
            }
            let p = m.r32(a0 + 0x228);
            if p != 0 {
                m.call(0x001f_b060, &[p], &[]);
            }
            let p = m.r32(a0 + 0x200);
            if p != 0 {
                m.call(0x001f_b060, &[p], &[]);
            }
            m.w32(a0 + 0x58, 0);
        }
    } else if f58 == 0 {
        if a1 != 0 {
            m.call(0x001f_b018, &[a1], &[]);
        }
        let p = m.r32(a0 + 0x228);
        if p != 0 {
            m.call(0x001f_b018, &[p], &[]);
        }
        let p = m.r32(a0 + 0x200);
        if p != 0 {
            m.call(0x001f_b018, &[p], &[]);
        }
        m.w32(a0 + 0x58, 1);
    }
    let mut f21 = 1.0f32;
    m.call(0x001f_dc08, &[a0], &[]);
    m.call(0x001f_dec0, &[a0], &[]);
    m.call(0x001f_dec8, &[a0], &[]);
    for o in [0u32, 4, 8] {
        let w = m.r32(0x002a_8288 + o);
        m.w32(sp + o, w);
    }
    if m.call(0x001f_e5e0, &[a0], &[]).0 != 0 && m.rf(0x002d_ded4) == 0.0 {
        f21 = 0.0;
    }
    let x = m.rf(a0 + 0x15c);
    let z = m.rf(0x002d_df10);
    let f20 = m.call(0x0011_d018, &[sp, 0], &[f4(x), f4(f21), f4(z)]).1[0];
    m.call(0x001f_ea60, &[a0, 0, 2], &[f4(f20)]);
    m.call(0x001f_ea60, &[a0, 1, 2], &[f4(f20)]);
    m.call(0x001f_ea60, &[a0, 2, 2], &[f4(f21)]);
    let mut s0 = m.r32(a0 + 0x4c);
    while s0 != 0 {
        let a = s0;
        s0 = m.r32(s0 + 0x38);
        m.call(0x001f_b8e0, &[a], &[]);
    }
    m.call(0x0023_1c48, &[], &[]);
    stack_pop(m, 0x10);
}

/// r0*p.x + r1*p.y + r2*p.z + r3 on xyz (VU `vmaddw.xyz`); w is kept from `w`.
fn xf3(r: &[V4; 4], p: V4, w: f32) -> V4 {
    let mut o = [0.0f32; 4];
    for i in 0..3 {
        o[i] = r[0][i] * p[0] + r[1][i] * p[1] + r[2][i] * p[2] + r[3][i];
    }
    o[3] = w;
    o
}

/// FUN_001bc2f8(a0): classify the +0xf4 polygon (+0xe8 points) under the vtable +0xf8 matrix via
/// FUN_001bc208 / FUN_001bc278 winding tests: 0 = self-crossing / degenerate, 1..4 shape class.
/// (The w lanes of vf07/vf08 are stale register contents in the asm; they are passed as 0 here.)
pub fn ee_sub_1bc2f8<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    if (m.r32(a0 + 0xe8) as i32) < 2 {
        return 2;
    }
    let sp = stack_push(m, 0x70);
    let (mut s6, mut s5, mut s3): (i32, i32, i32) = (0, 0, 0);
    let mp = vcall(m, a0, 0xf8, &[], &[]).0;
    let mut r = [[0.0f32; 4]; 4];
    for i in 0..4u32 {
        r[i as usize] = m.rv(mp + i.wrapping_mul(0x10));
        m.wv(sp + i.wrapping_mul(0x10), r[i as usize]);
    }
    let pts = m.r32(a0 + 0xf4);
    let mut w7 = 0.0f32;
    let mut w8 = 0.0f32;
    let v7 = xf3(&r, m.rv(pts + 0x10), w7);
    let v8 = xf3(&r, m.rv(pts), w8);
    let mut s4 = m.call(0x001b_c208, &[], &[v8, v7]).0 as i32;
    let mut ret = 2u32;
    if (m.r32(a0 + 0xe8) as i32).wrapping_sub(1) > 0 {
        loop {
            let s1 = m.call(0x001b_c530, &[a0, s5 as u32], &[]).0;
            let s0 = m.call(0x001b_c530, &[a0, s1.wrapping_add(1)], &[]).0;
            let v = m.call(0x001b_c530, &[a0, s0.wrapping_add(1)], &[]).0;
            let pts = m.r32(a0 + 0xf4);
            let mut r = [[0.0f32; 4]; 4];
            for i in 0..4u32 {
                r[i as usize] = m.rv(sp + i.wrapping_mul(0x10));
            }
            let v7 = xf3(&r, m.rv((v << 4).wrapping_add(pts)), w7);
            let p6 = m.rv((s0 << 4).wrapping_add(pts));
            let v6 = xf3(&r, p6, p6[3]);
            let v8 = xf3(&r, m.rv((s1 << 4).wrapping_add(pts)), w8);
            m.wv(sp + 0x50, v6);
            m.wv(sp + 0x60, v7);
            m.wv(sp + 0x40, v8);
            let a = m.call(0x001b_c208, &[], &[v6, v7]).0 as i32;
            let v6 = m.rv(sp + 0x50);
            let v7 = m.rv(sp + 0x60);
            if a != 0 && a == s4.wrapping_neg() {
                s6 += 1;
            }
            s4 = a;
            let q = m.rv(sp + 0x40);
            let b = m.call(0x001b_c278, &[], &[q, v6, v7]).0 as i32;
            if b != 0 {
                let neg = s3.wrapping_neg();
                s3 = b;
                if b == neg {
                    stack_pop(m, 0x70);
                    return 0;
                }
            }
            s5 += 1;
            if !(s5 < (m.r32(a0 + 0xe8) as i32).wrapping_sub(1)) {
                break;
            }
        }
    }
    stack_pop(m, 0x70);
    if !(s6 < 3) {
        return (s3 == 0) as u32;
    }
    if s3 == -1 {
        4
    } else if s3 == 1 {
        3
    } else {
        2
    }
}

/// FUN_001d8d80(a0, a1): save-reset: stash 11 named save entries (keyed by the GS+0x3b2c.. id table)
/// into strings (FUN_001d7ed8 / FUN_00129658) when a1 && FUN_001d7d30(0x3e0ea8, a0), clear the 0x5400 save
/// block (FUN_002333b4) + FUN_001d54b8, then write them back (FUN_001291a8 / FUN_00258200); all inside a
/// FUN_001ae478 / FUN_001ae4a0 bracket, destroying the 11 temporaries (FUN_00129ec8) in reverse.
pub fn ee_sub_1d8d80<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let sp = stack_push(m, 0xd0);
    m.call(0x001a_e478, &[sp], &[]);
    let s6 = sp + 0x70;
    let fp = sp + 0xc8;
    for (i, o) in [0x3b2cu32, 0x3b48, 0x3b54, 0x3b4c, 0x3b50, 0x3b30, 0x3b34, 0x3b38, 0x3b3c, 0x3b40, 0x3b44]
        .iter()
        .enumerate()
    {
        let w = m.r32(GS + o);
        m.w32(sp + 0x40 + 4 * i as u32, w);
    }
    for i in 0..11u32 {
        let w = m.r32(sp + 0x40 + 4 * i);
        m.w32(sp + 0x10 + 4 * i, w);
    }
    let mut t6 = s6;
    let mut t7: i32 = 10;
    loop {
        m.w32(t6, 0xffff_ffff);
        t7 -= 1;
        t6 = t6.wrapping_add(8);
        if t7 == -1 {
            break;
        }
    }
    let mut s4 = a1;
    if s4 != 0 {
        if m.call(0x001d_7d30, &[0x003e_0ea8, a0], &[]).0 == 0 {
            s4 = 0;
        }
        if s4 != 0 {
            let (mut s3, mut s1, mut s0) = (0u32, sp + 0x10, s6);
            loop {
                let id = m.r32(s1);
                m.call(0x001d_7ed8, &[sp + 0x40, 0x003e_0ea8, id, a0], &[]);
                s3 += 1;
                m.call(0x0012_9658, &[s0, sp + 0x40], &[]);
                s1 += 4;
                m.call(0x0012_9ec8, &[sp + 0x40], &[]);
                s0 += 8;
                if s3 >= 0xb {
                    break;
                }
            }
        }
    }
    m.call(0x0023_33b4, &[a0, 0, 0x5400], &[]);
    m.call(0x001d_54b8, &[], &[]);
    if s4 != 0 {
        let (mut s3, mut s1, mut s0) = (0u32, sp + 0x10, s6);
        loop {
            m.call(0x0012_91a8, &[sp + 0x40, s0], &[]);
            s3 += 1;
            let id = m.r32(s1);
            m.call(0x0025_8200, &[sp + 0x50, id, sp + 0x40, a0], &[]);
            s1 += 4;
            m.call(0x0012_9ec8, &[sp + 0x50], &[]);
            s0 += 8;
            m.call(0x0012_9ec8, &[sp + 0x40], &[]);
            if s3 >= 0xb {
                break;
            }
        }
    }
    let mut s0 = fp;
    while s6 != s0 {
        s0 -= 8;
        m.call(0x0012_9ec8, &[s0], &[]);
    }
    m.call(0x001a_e4a0, &[sp, 2], &[]);
    stack_pop(m, 0xd0);
}

/// FUN_001b86b0(a0, a1): switch a0's pose to a1 (+0x1810): reset blend state, optionally re-face the
/// +0x10dc target, start the +0x17fc[a1] anim (FUN_001f15b8 / FUN_001305b8 / FUN_00131a90) and compute the
/// +0x1830 anchor (from the anim's root track or FUN_001a62e8), then FUN_001431d0.
pub fn ee_sub_1b86b0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if a1 == m.r32(a0 + 0x1810) {
        return;
    }
    m.w32(a0 + 0x1810, a1);
    let t = m.rf(0x002d_ded0);
    m.wf(a0 + 0x1814, t);
    if a1 == 0xffff_ffff {
        return;
    }
    let sp = stack_push(m, 0x30);
    m.w32(a0 + 0x1820, 0);
    m.w32(a0 + 0x1818, 0);
    let p = m.r32(a0 + 0x190c);
    m.w32(a0 + 0x181c, 0);
    if p != 0 {
        m.call(0x0013_0658, &[a0 + 0x190c], &[]);
    }
    let q = m.rv(0x002a_2dd0);
    let s1 = m.r32(a0.wrapping_add(a1 << 2) + 0x17fc);
    let tg = m.r32(a0 + 0x10dc);
    m.wv(a0 + 0x190, q);
    if tg != 0 && !((m.r32(a0 + 0x10e4) as i32) < 2) {
        let c = m.r32(a0 + 0x150);
        if m.r32(c + 0x30) & 8 == 0 {
            let a = m.call(0x001a_62e8, &[tg], &[]).1;
            m.wv(sp + 0x20, a);
            let b = m.call(0x001a_62e8, &[a0], &[]).1;
            let a = m.rv(sp + 0x20);
            let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2], a[3] - b[3]];
            let f20 = m.call(0x0024_58f0, &[], &[f4(d[1]), f4(d[0])]).1[0];
            let h = m.rf(a0 + 0x1a8);
            let r = m.call(0x0011_cf60, &[], &[f4(f20 - h)]).1[0];
            if r.abs() < m.rf(0x002c_09f0) {
                m.call(0x0020_4660, &[a0], &[f4(f20)]);
                m.call(0x0020_4ca8, &[a0], &[]);
            }
        }
    }
    let mut f20 = 0.0f32;
    m.call(0x001f_15b8, &[a0 + 0x1034], &[]);
    m.call(0x0013_05b8, &[s1, a0, 1, a0 + 0x1030], &[f4(f20), f4(1.0)]);
    if s1 != 0 {
        let v = m.call(0x0012_f938, &[s1, 0x1ac], &[]).0;
        if v != 0 {
            f20 = m.rf(v);
        }
    }
    let f21 = 0.0f32;
    if f20 != f21 {
        let p = m.r32(a0 + 0x1030);
        let t = m.rf(0x002d_ded0);
        let d = m.rf(p + 0x40);
        m.call(0x0013_1a90, &[p], &[f4(t + f20 / d)]);
    }
    let p = m.r32(a0 + 0x1030);
    if p != 0 {
        let x = m.r32(p + 0x14);
        let s1 = m.call(0x0012_ff10, &[x, a0, 1, 1], &[]).0;
        if s1 != 0 {
            let o = m.r32(s1 + 4);
            let vt = m.r32(o);
            let adj = m.r16(vt + 0x18) as i16 as i32 as u32;
            let f = m.r32(vt + 0x1c);
            m.call(f, &[o.wrapping_add(adj), sp, 0], &[f4(f21), f4(f21)]);
            let o = m.r32(s1 + 4);
            let p = m.r32(a0 + 0x1030);
            let vt = m.r32(o);
            let c = m.r32(p + 0x14);
            let adj = m.r16(vt + 0x18) as i16 as i32 as u32;
            let f = m.r32(vt + 0x1c);
            let z = m.rf(c + 0x74);
            m.call(f, &[o.wrapping_add(adj), sp + 0x10, 0], &[f4(z), f4(f21)]);
            let b = m.call(0x001a_62e8, &[a0], &[]).1;
            let u = m.rv(sp);
            let w = m.rv(sp + 0x10);
            let d = [w[0] - u[0], w[1] - u[1], w[2] - u[2], w[3] - u[3]];
            let r = [m.rv(a0 + 0x1b0), m.rv(a0 + 0x1c0), m.rv(a0 + 0x1d0)];
            let mut v = d;
            for i in 0..3 {
                v[i] = r[0][i] * d[0] + r[1][i] * d[1] + r[2][i] * d[2];
            }
            m.wv(a0 + 0x1830, [b[0] + v[0], b[1] + v[1], b[2] + v[2], b[3] + v[3]]);
        } else {
            let b = m.call(0x001a_62e8, &[a0], &[]).1;
            m.wv(a0 + 0x1830, b);
        }
    }
    m.call(0x0014_31d0, &[a0], &[]);
    stack_pop(m, 0x30);
}

/// FUN_001a3f78: walk the BSP (root a0+0x455c, leaf masks a0+0x4564) from point `v` to OR the
/// leaf visibility mask into a copy of a0+0x4570, then for every set bit pick item
/// *(a0+0x4154+4*idx) and insert (score, item) into the descending array a3[a2] (8-byte entries).
pub fn ee_sub_1a3f78<M: QMem + ?Sized>(m: &mut M, a0: u32, a2: u32, a3: u32, v: V4) {
    let sp = stack_push(m, 0xf0);
    m.wv(sp + 0x40, v);
    m.call(0x0023_33b4, &[a3, 0, a2 << 3], &[]);
    let q0 = m.rv(a0 + 0x4570);
    let q1 = m.rv(a0 + 0x4580);
    m.wv(sp, q0);
    m.wv(sp + 0x10, q1);
    let mut node = m.r32(a0 + 0x455c);
    if node != 0 {
        let leaf;
        loop {
            let p = m.rv(node);
            m.wv(sp + 0x50, p);
            m.call(0x0025_64b8, &[sp + 0x20], &[]);
            m.call(0x0025_64b8, &[sp + 0x30], &[]);
            let p = m.rv(sp + 0x50);
            let vv = m.rv(sp + 0x40);
            let (x, y, z) = (p[0] * vv[0], p[1] * vv[1], p[2] * vv[2]);
            let d = x + y + z - p[3];
            m.wf(sp + 0x30, 1.0);
            m.wv(sp + 0x20, [d, y, z, p[3]]);
            let (co, lo) = if d < 0.0 { (0x10, 0x18) } else { (0x14, 0x1c) };
            let c = m.r32(node + co);
            if c != 0 {
                node = c;
            } else {
                leaf = m.r32(node + lo);
                break;
            }
        }
        let s2 = m.r32(a0 + 0x4564).wrapping_add(leaf << 5);
        for i in 0..8u32 {
            let w = m.r32(sp + 4 * i) | m.r32(s2 + 4 * i);
            m.w32(sp + 4 * i, w);
        }
    }
    let vv = m.rv(sp + 0x40);
    let n = a2 as i32;
    for i in 0..8u32 {
        let mut w = m.r32(sp + 4 * i);
        while w != 0 {
            let rest = w & w.wrapping_sub(1);
            let low = w & !rest;
            let idx = (i << 5) + low.trailing_zeros();
            let it = m.r32(a0 + 0x4154 + (idx << 2));
            let c = m.rv(it + 0x100);
            let (dx, dy, dz) = (vv[0] - c[0], vv[1] - c[1], vv[2] - c[2]);
            let dist = (dx * dx + dy * dy + dz * dz).sqrt();
            let f2 = m.rf(it + 0x110) + m.rf(it + 0x114) * dist;
            let mut t0 = n;
            if t0 > 0 && !(f2 < m.rf(a3.wrapping_add((n as u32) << 3).wrapping_sub(8))) {
                let mut p = a3.wrapping_add((n as u32) << 3);
                loop {
                    let shift = t0 < n;
                    t0 -= 1;
                    if shift {
                        let lo = m.r32(p - 8);
                        let hi = m.r32(p - 4);
                        m.w32(p, lo);
                        m.w32(p + 4, hi);
                    }
                    if t0 <= 0 {
                        break;
                    }
                    p -= 8;
                    if f2 < m.rf(p - 8) {
                        break;
                    }
                }
            }
            if t0 < n {
                let e = a3.wrapping_add((t0 as u32) << 3);
                m.w32(e + 4, it);
                m.wf(e, f2);
            }
            w = rest;
        }
    }
    stack_pop(m, 0xf0);
}

/// FUN_001af6d0: message handler on *a2: 0x67/0x68 adjust an option level (0x3dd190 / 0x3dd18c)
/// and play the result; 0x3f nudges/clamps 0x2f6468/0x2f646c and packs them into the GS regs
/// at 0x2debc0 / 0x2deb98.
pub fn ee_sub_1af6d0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    let id = m.r32(a2) as i32;
    if id == 0x67 || id == 0x68 {
        let o = m.r32(a0 + 0x10);
        let n = m.call(0x0018_7e88, &[o], &[]).0 as i32 as f32;
        if n == 0.0 {
            return;
        }
        let base = m.rf(if id == 0x67 { 0x003d_d190 } else { 0x003d_d18c });
        let x = base + n * m.rf(0x002d_df10) * 1.5;
        let f20 = m.call(0x0011_d948, &[0x002b_bb60], &[f4(x)]).1[0];
        if id == 0x68 {
            m.call(0x001f_ea60, &[0x003d_d040, 0, 1], &[f4(f20)]);
        }
        m.call(0x001f_ea60, &[0x003d_d040, if id == 0x67 { 1 } else { 3 }, 1], &[f4(f20)]);
        let o = m.r32(a1 + 8);
        m.call(0x0021_2e40, &[o, 0x315], &[f4(f20)]);
        return;
    }
    if id != 0x3f {
        return;
    }
    let o = m.r32(a0 + 0x10);
    let f20 = m.call(0x0018_7e88, &[o], &[]).0 as i32 as f32;
    let o = m.r32(a0 + 0x10);
    let f4v = m.call(0x0018_80d8, &[o], &[]).0 as i32 as f32;
    if f20 == 0.0 && f4v == 0.0 {
        return;
    }
    let v = m.rf(0x002f_6468) + f20 * m.rf(0x002c_01c8);
    let a = if v < 424.0 { 424.0 } else if 848.0 < v { 848.0 } else { v };
    let w = m.rf(0x002f_646c) + f4v * m.rf(0x002c_01cc);
    m.wf(0x002f_6468, a);
    let b = if w < 10.0 { 10.0 } else if 90.0 < w { 90.0 } else { w };
    m.wf(0x002f_646c, b);
    let a = m.rf(0x002f_6468);
    let r = m.call(0x0023_e1c8, &[], &[f4(a)]).0 & 0xfff;
    let s1 = (m.r32(0x002d_ebc0) & 0xffff_f000) | r;
    let s0 = (m.r32(0x002d_eb98) & 0xffff_f000) | r;
    m.w32(0x002d_ebc0, s1);
    m.w32(0x002d_eb98, s0);
    let r2 = (m.call(0x0023_e1c8, &[], &[f4(b)]).0 & 0x7fe) << 12;
    m.w32(0x002d_ebc0, (s1 & 0xff80_0fff) | r2);
    m.w32(0x002d_eb98, (s0 & 0xff80_0fff) | r2);
}

/// r0*v.x + r1*v.y + r2*v.z on xyz (w from `w`)
fn h14_rot(r: &[V4; 4], v: V4, w: f32) -> V4 {
    let mut o = [0.0; 4];
    for k in 0..3 {
        o[k] = r[0][k] * v[0] + r[1][k] * v[1] + r[2][k] * v[2];
    }
    o[3] = w;
    o
}

/// FUN_001e6a30: closest point on a polyline path (segments at *(a0+0xf4), 16-byte points, count
/// *(a0+0xe8), closed if *(a0+0xe4)) to point `v` in world space (matrix = vslot 30).
/// Outputs: a3 = world point, t0 = unit world tangent, t1 = interpolated param (*(a0+0xec)),
/// t2 = FUN_001e63c0(a0, param).
pub fn ee_sub_1e6a30<M: QMem + ?Sized>(
    m: &mut M,
    a0: u32,
    a2: u32,
    a3: u32,
    t0: u32,
    t1: u32,
    t2: u32,
    v: V4,
) {
    let sp = stack_push(m, 0x130);
    m.wv(sp + 0x70, v);
    m.w32(sp + 0x68, t1);
    m.w32(sp + 0x6c, t2);
    let mut s0 = m.call(0x001e_7538, &[a0, sp + 0x60, sp + 0x64], &[]).0 as i32;
    let p = vslot(m, a0, 30, &[], &[]).0;
    for k in 0..4 {
        let q = m.rv(p + 16 * k);
        m.wv(sp + 0x10 + 16 * k, q);
    }
    let mut s2 = 0i32;
    let mut s5 = false;
    let mut s6 = false;
    let mut f12: f32;
    loop {
        let r = m.rm(sp + 0x10);
        let seg = m.r32(a0 + 0xf4).wrapping_add((s0 as u32) << 4);
        let p0 = m.rv(seg);
        let p1 = m.rv(seg + 16);
        let d = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2], p1[3] - p0[3]];
        let dir = h14_rot(&r, d, d[3]);
        let mut wp = h14_rot(&r, p0, r[0][3]);
        for k in 0..3 {
            wp[k] += r[3][k];
        }
        m.wv(sp, dir);
        let pt = m.rv(sp + 0x70);
        let mut rel = [pt[0] - wp[0], pt[1] - wp[1], pt[2] - wp[2], pt[3] - wp[3]];
        if a2 != 0 {
            rel = m.call(0x001f_a970, &[a2], &[rel]).1;
            m.wv(sp + 0x80, rel);
            let d0 = m.rv(sp);
            let r2 = m.call(0x001f_a970, &[a2], &[d0]).1;
            rel = m.rv(sp + 0x80);
            m.wv(sp, r2);
        }
        let d2 = m.rv(sp);
        let len = (d2[0] * d2[0] + d2[1] * d2[1] + d2[2] * d2[2]).sqrt();
        let q = 1.0 / len;
        let u = [d2[0] * q, d2[1] * q, d2[2] * q, d2[3] * q];
        m.wv(sp + 0x50, d2);
        let f0 = u[0] * rel[0] + u[1] * rel[1] + u[2] * rel[2];
        m.wv(sp, u);
        f12 = 0.0;
        if !(f0 < 0.0) {
            if len < f0 {
                if s6 {
                    s0 = s2;
                    break;
                }
                if s0 < (m.r32(a0 + 0xe8) as i32).wrapping_sub(2) {
                    s0 += 1;
                } else if m.r32(a0 + 0xe4) != 0 {
                    s0 = 0;
                } else {
                    f12 = 1.0;
                    break;
                }
                s5 = true;
                s2 = s0;
                continue;
            }
            f12 = f0 / len;
            break;
        }
        if s5 {
            f12 = 1.0;
            s0 = s2;
            break;
        }
        if s0 > 0 {
            s0 -= 1;
        } else {
            if m.r32(a0 + 0xe4) == 0 {
                break;
            }
            s0 = (m.r32(a0 + 0xe8) as i32).wrapping_sub(2);
        }
        s6 = true;
        s2 = s0;
    }
    if a3 != 0 {
        let r = m.rm(sp + 0x10);
        let u = 1.0 - f12;
        let seg = m.r32(a0 + 0xf4).wrapping_add((s0 as u32) << 4);
        let p0 = m.rv(seg);
        let p1 = m.rv(seg + 16);
        let mut b = [0.0; 4];
        for k in 0..4 {
            b[k] = p0[k] * u + p1[k] * f12;
        }
        let mut o = h14_rot(&r, b, r[0][3]);
        for k in 0..3 {
            o[k] += r[3][k];
        }
        m.wv(a3, o);
    }
    if t0 != 0 {
        let r = m.rm(sp + 0x10);
        let seg = m.r32(a0 + 0xf4).wrapping_add((s0 as u32) << 4);
        let p0 = m.rv(seg);
        let p1 = m.rv(seg + 16);
        let d = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2], p1[3] - p0[3]];
        let dv = h14_rot(&r, d, d[3]);
        let q = 1.0 / (dv[0] * dv[0] + dv[1] * dv[1] + dv[2] * dv[2]).sqrt();
        m.wv(t0, [dv[0] * q, dv[1] * q, dv[2] * q, dv[3] * q]);
    }
    let e = m.r32(a0 + 0xec).wrapping_add((s0 as u32) << 2);
    let f12 = (1.0 - f12) * m.rf(e) + f12 * m.rf(e + 4);
    let o1 = m.r32(sp + 0x68);
    if o1 != 0 {
        m.wf(o1, f12);
    }
    let o2 = m.r32(sp + 0x6c);
    if o2 != 0 {
        let f0 = m.call(0x001e_63c0, &[a0], &[f4(f12)]).1[0];
        let o2 = m.r32(sp + 0x6c);
        m.wf(o2, f0);
    }
    stack_pop(m, 0x130);
}

fn h14_dot3(a: V4, b: V4) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// `vadda ACC, a, vf00; vmaddx d, b, s` = a + (0,0,0,1) + b*s
fn h14_adda(a: V4, b: V4, s: f32) -> V4 {
    [a[0] + b[0] * s, a[1] + b[1] * s, a[2] + b[2] * s, a[3] + 1.0 + b[3] * s]
}

/// FUN_001c55e0: sweep the actor (radius *(a0+0x1d8)) against nearby statics (FUN_0020faa0 query):
/// find the deepest penetration with a contact normal opposing the velocity *(a0+0x180), then push
/// position *(a0+0x170) out and redirect / slide the velocity.
pub fn ee_sub_1c55e0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let s1 = a0;
    let sp = stack_push(m, 0x290);
    m.call(0x001a_e478, &[sp], &[]);
    let mut f20: f32 = 0.0;
    let pos = m.call(0x001a_62e8, &[s1], &[]).1;
    let c = m.rv(0x002a_2de0);
    let r = m.rf(s1 + 0x1d8);
    let a = h14_adda(pos, c, -r);
    m.wv(sp + 0x10, f4(-r));
    m.wv(sp + 0x200, a);
    let pos = m.call(0x001a_62e8, &[s1], &[]).1;
    let r = m.rf(s1 + 0x1d8);
    let c = m.rv(0x002a_2de0);
    let b = h14_adda(pos, c, r);
    let a = m.rv(sp + 0x200);
    m.wv(sp + 0x10, f4(r));
    m.call(0x0020_faa0, &[0x003d_d490, 0x001c_5570, 0, sp + 0x1f0, sp + 0x1f4], &[a, b]);
    let mut s2 = 0i32;
    if (m.r32(sp + 0x1f0) as i32) > 0 {
        loop {
            let lst = m.r32(sp + 0x1f4);
            let s0 = m.r32(lst.wrapping_add((s2 as u32) << 2));
            if m.r64(s0 + 0x10) & (1u64 << 49) != 0 {
                let o = m.r32(s0);
                let q = m.rv(o + 0x70);
                m.wv(sp + 0x210, q);
                let p = m.call(0x001a_62e8, &[s1], &[]).1;
                let f2a = m.rf(s0 + 8);
                let q = m.rv(sp + 0x210);
                let d = [q[0] - p[0], q[1] - p[1], q[2] - p[2], 0.0];
                let f1 = m.rf(s1 + 0x1d8);
                let f2 = h14_dot3(d, d).sqrt() - f2a - f1;
                if f2 < f20 {
                    let pa = m.rv(s1 + 0x170);
                    let o = m.r32(s0);
                    let pb = m.rv(o + 0x70);
                    let mut n = [pa[0] - pb[0], pa[1] - pb[1], pa[2] - pb[2], pa[3] - pb[3]];
                    let len = h14_dot3(n, n).sqrt();
                    if len < f32::from_bits(0x38d1_b717) {
                        n = m.rv(0x002a_2da0);
                    } else {
                        let q = 1.0 / len;
                        n = [n[0] * q, n[1] * q, n[2] * q, n[3] * q];
                    }
                    let vel = m.rv(s1 + 0x180);
                    m.wv(sp + 0x10, n);
                    if h14_dot3(vel, n) < 0.0 {
                        f20 = f2;
                        m.wv(sp + 0x20, n);
                    }
                }
            } else if m.r32(s0 + 0x74) != 0 {
                let p = m.call(0x001a_62e8, &[s1], &[]).1;
                let r = m.rf(s1 + 0x1d8);
                let mut n = m.call(0x001a_93c8, &[s0, 0x7fff_ffff, 0, 0, 8, sp + 0x30], &[p, f4(r)]).0 as i32;
                if n > 0 {
                    let mut v1 = sp + 0x30;
                    loop {
                        let pa = m.rv(v1);
                        let nb = m.rv(v1 + 16);
                        let f1 = h14_dot3(pa, nb) - m.rf(v1 + 0x1c);
                        if f1 < f20 {
                            let vel = m.rv(s1 + 0x180);
                            if h14_dot3(vel, nb) < 0.0 {
                                m.wv(sp + 0x20, nb);
                                f20 = f1;
                            }
                        }
                        n -= 1;
                        v1 += 0x30;
                        if n == 0 {
                            break;
                        }
                    }
                }
            }
            s2 += 1;
            if !(s2 < m.r32(sp + 0x1f0) as i32) {
                break;
            }
        }
    }
    if f20 < 0.0 {
        let f1 = m.rf(sp + 0x28);
        if f32::from_bits(0x3f33_3333) < f1 && m.r32(s1 + 0x150) == 4 {
            let vel = m.rv(s1 + 0x180);
            let up = m.rv(0x002a_2dc0);
            let f4r = m.call(0x0011_e290, &[0, sp + 0x1c0], &[vel, up]).1[0];
            let v3 = m.rv(sp + 0x1c0);
            let len = h14_dot3(v3, v3).sqrt();
            if len < f32::from_bits(0x38d1_b717) {
                let d = m.rv(0x002a_2dd0);
                m.wv(sp + 0x1c0, d);
            } else {
                let q = 1.0 / len;
                m.wv(sp + 0x1c0, [v3[0] * q, v3[1] * q, v3[2] * q, v3[3] * q]);
            }
            let k0 = (m.rf(s1 + 0x1e4) * len).min(m.rf(0x002c_0d74));
            let k1 = (-m.rf(s1 + 0x1e0) * f4r).min(m.rf(0x002c_0d78));
            let v5 = m.rv(0x002a_2dc0);
            let v4 = m.rv(sp + 0x1c0);
            let mut out = [0.0; 4];
            for k in 0..4 {
                out[k] = v4[k] * k0 + v5[k] * k1;
            }
            m.wv(sp + 0x1d0, f4(k1));
            m.wv(sp + 0x1b0, out);
            if m.r32(s1 + 0x1c4) == 0 {
                let n = m.rv(sp + 0x20);
                if h14_dot3(out, n) < m.rf(s1 + 0x1d0) {
                    let k2 = m.rf(s1 + 0x1d4);
                    for k in 0..4 {
                        out[k] = v4[k] * k0 + v5[k] * k2;
                    }
                    m.wv(sp + 0x1b0, out);
                    m.w32(s1 + 0x1c4, 1);
                    m.wv(sp + 0x1d0, f4(k2));
                }
            }
        } else {
            let vel = m.rv(s1 + 0x180);
            let n = m.rv(sp + 0x20);
            m.call(0x0011_e290, &[sp + 0x1c0, sp + 0x1d0], &[vel, n]);
            let k1 = -m.rf(s1 + 0x1e0);
            let k2 = m.rf(s1 + 0x1e4);
            let a = m.rv(sp + 0x1c0);
            let b = m.rv(sp + 0x1d0);
            let mut out = [0.0; 4];
            for k in 0..4 {
                out[k] = a[k] * k1 + b[k] * k2;
            }
            m.wv(sp + 0x1b0, out);
            m.wv(sp + 0x1e0, f4(k2));
        }
        let p = m.rv(s1 + 0x170);
        let n = m.rv(sp + 0x20);
        let np = h14_adda(p, n, -f20);
        m.wv(s1 + 0x170, np);
        let o = m.rv(sp + 0x1b0);
        m.wv(s1 + 0x180, o);
        m.wv(sp + 0x1c0, f4(-f20));
    }
    m.call(0x001a_e4a0, &[sp, 2], &[]);
    stack_pop(m, 0x290);
}

/// FUN_001a19a0: player damage-message handler (a1 = 0x60-byte damage message, kind at +0xc):
/// health delta, hit reactions per damage kind, stores the message at a0+0x30; returns 1 if handled.
pub fn ee_sub_1a19a0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let s1 = a0;
    let mut s2 = a1;
    let sp = stack_push(m, 0xb0);
    let mut s0: i32;
    'pre: {
        if m.call(0x001a_2358, &[s1], &[]).0 != 0 {
            if m.r32(s1 + 4).wrapping_sub(8) < 5 {
                stack_pop(m, 0xb0);
                return 0;
            }
            s0 = -0x8000;
            break 'pre;
        }
        let o = m.r32(s1);
        let k = m.r32(s2 + 0xc);
        if vslot(m, o, 75, &[k], &[]).0 != 0 {
            stack_pop(m, 0xb0);
            return 0;
        }
        let v1 = m.r32(s1);
        if m.r32(v1 + 0x4c8) == 1 {
            let o = m.r32(v1 + 0x4d4);
            let r = vslot(m, o, 74, &[s2], &[]).0;
            stack_pop(m, 0xb0);
            return r;
        }
        s0 = 0;
        if m.r32(v1 + 0x29c) == 0 && m.r32(s1 + 0x90) != 1 {
            let x = m.r32(s2 + 0x10);
            s0 = m.call(0x0015_5228, &[0x002d_e578, x], &[]).0 as i32;
        }
        if m.r32(s2 + 0xc) == 0x0010_0000 {
            let p = m.r32(s1 + 0x10);
            if (m.r32(p) as i32).wrapping_add(s0) > 0 {
                if m.r32(s1 + 0x90) != 1 {
                    m.w32(s1 + 0x90, 0);
                    let f = m.rf(0x002d_ded0);
                    m.wf(s1 + 0x94, f);
                    m.wf(s1 + 0x98, f);
                }
                let p = m.r32(s1 + 0x10);
                let v = m.r32(p).wrapping_add(s0 as u32);
                m.w32(p, v);
                let x = m.r32(s1 + 0x108);
                if x != 0 {
                    let o = m.r32(s1);
                    m.call(0x0013_05b8, &[x, o, 1, s1 + 0x10c], &[f4(0.0), f4(1.0)]);
                }
                h14_1a19a0_tail(m, s1);
                stack_pop(m, 0xb0);
                return 1;
            }
            for k in 0..6 {
                let q = m.rv(s2 + 16 * k);
                m.wv(sp + 16 * k, q);
            }
            m.w32(sp + 0xc, 0x200);
            s2 = sp;
            m.w32(sp + 0x34, 0);
            m.w32(sp + 0x38, 0);
            m.w32(sp + 0x30, 0);
        }
        let o = m.r32(s2);
        if o != 0 && m.call(0x0013_2b10, &[o, 0x38], &[]).0 != 0 {
            let o = m.r32(s2);
            let k = m.r32(o + 0xfbc);
            if k == 8 || k == 0xd {
                s0 /= 2;
            }
        }
    }
    let o = m.r32(s1);
    vslot(m, o, 87, &[3], &[]);
    let a = m.r32(s1);
    let v = m.r32(a + 0x11a8);
    if v != 0 {
        let w = m.r32(v + 0x158);
        let call = if m.r32(w + 8) != 0 {
            true
        } else {
            let p = m.r32(s1 + 0x10);
            !((m.r32(p) as i32).wrapping_add(s0) > 0)
        };
        if call {
            m.call(0x0014_3380, &[a], &[]);
        }
    }
    let x = m.r32(s2);
    m.call(0x0016_19a0, &[0x002d_eb30, 0xf, x], &[]);
    for k in 0..6 {
        let q = m.rv(s2 + 16 * k);
        m.wv(s1 + 0x30 + 16 * k, q);
    }
    m.w32(s1 + 0x40, s0 as u32);
    let k = m.r32(0x002a_6768);
    m.call(0x001a_15b0, &[s1, k], &[]);
    let p = m.r32(s1 + 0x10);
    let mut v = (m.r32(p) as i32).wrapping_add(m.r32(s1 + 0x40) as i32);
    if !(-1 < v) {
        v = 0;
    }
    m.w32(p, v as u32);
    let p = m.r32(s1 + 0x10);
    if (m.r32(p) as i32) > 0 {
        m.w32(s1 + 0x90, 1);
        let f = m.rf(0x002d_ded0);
        m.wf(s1 + 0x94, f);
    }
    let hp = |m: &mut M| {
        let p = m.r32(s1 + 0x10);
        (m.r32(p) as i32) > 0
    };
    match m.r32(s2 + 0xc) {
        0x800 | 0x1000 => {
            let o = m.r32(s1);
            let v = m.rv(0x002a_2dd0);
            vslot(m, o, 34, &[], &[v]);
            let o = m.r32(s1);
            let v = m.rv(0x002a_2dd0);
            vslot(m, o, 35, &[], &[v]);
            let a = if m.r32(s2 + 0xc) ^ 0x800 != 0 { 2 } else { 1 };
            m.call(0x001a_0b40, &[s1, a], &[]);
            let k = m.r32(0x002a_6780);
            m.call(0x001a_15b0, &[s1, k], &[]);
        }
        0x100 | 0x200 => {
            m.call(0x001a_16c8, &[s1], &[]);
            let o = m.r32(s1);
            let y = vslot(m, o, 32, &[], &[]).1[1];
            if 400.0 < y {
                let o = m.r32(s1);
                let c = m.call(0x0013_2ea0, &[o], &[]).0;
                let snd: u32 = match c {
                    0x36 => 0x84,
                    0x35 => 0x69,
                    0x37 => 0x83,
                    _ => 0xffff_ffff,
                };
                let o = m.r32(s1);
                let s = m.call(0x001f_bb90, &[snd], &[]).0;
                m.call(0x001f_b2f8, &[s, o], &[]);
            }
            let k = m.r32(0x002a_6770);
            m.call(0x001a_15b0, &[s1, k], &[]);
        }
        0x8000 => {
            m.call(0x001a_0b40, &[s1, 7], &[]);
            let k = m.r32(0x002a_6778);
            m.call(0x001a_15b0, &[s1, k], &[]);
        }
        0x2000 => {
            if hp(m) {
                m.call(0x001a_0b40, &[s1, 3], &[]);
                let k = m.r32(0x002a_6788);
                m.call(0x001a_15b0, &[s1, k], &[]);
            } else {
                m.call(0x001a_0b40, &[s1, 0xa], &[]);
            }
        }
        0x400 => {
            let o = m.r32(s1);
            let v = m.rv(0x002b_fb10);
            vslot(m, o, 34, &[], &[v]);
            let a = if hp(m) { 4 } else { 0xc };
            m.call(0x001a_0b40, &[s1, a], &[]);
        }
        0x4000 => {
            if hp(m) {
                m.call(0x001a_0b40, &[s1, 5], &[]);
                let k = m.r32(0x002a_6788);
                m.call(0x001a_15b0, &[s1, k], &[]);
            } else {
                m.call(0x001a_0b40, &[s1, 0xb], &[]);
            }
        }
        0x40 => {
            m.call(0x001a_16c8, &[s1], &[]);
            let k = m.r32(0x002a_6770);
            m.call(0x001a_15b0, &[s1, k], &[]);
        }
        _ => {}
    }
    h14_1a19a0_tail(m, s1);
    stack_pop(m, 0xb0);
    1
}

fn h14_1a19a0_tail<M: QMem + ?Sized>(m: &mut M, s1: u32) {
    let o = m.r32(s1);
    let pos = m.call(0x001a_62e8, &[o], &[]).1;
    let w = m.r32(0x003d_d490 + 0x4038);
    let o = m.r32(s1);
    m.call(0x0015_6698, &[w, o], &[pos]);
}

fn h14_bounds<M: QMem + ?Sized>(m: &mut M, sp: u32, p: V4) {
    let mut mn = m.rv(sp + 0x60);
    let mut mx = m.rv(sp + 0x70);
    for k in 0..3 {
        mn[k] = mn[k].min(p[k]);
        mx[k] = mx[k].max(p[k]);
    }
    m.wv(sp + 0x70, mx);
    m.wv(sp + 0x60, mn);
}

/// FUN_001de5d8: resolve the attach targets of each of the *(a0+0x198) slot pairs (0x150 each at
/// *(a0+0x19c), 2 x 0xa0 entries): path targets (class 0xa) build a speed profile (linear / ramp /
/// s-curve), others take the target / own position; accumulate the world bounds and create the
/// bounds proxy at *(a0+0xe4).
pub fn ee_sub_1de5d8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let s4 = a0;
    let sp = stack_push(m, 0x140);
    m.call(0x001f_2ba8, &[s4], &[]);
    let v = m.rv(0x002a_2df0);
    m.wv(sp + 0x60, v);
    let v = m.rv(0x002a_2e00);
    m.wv(sp + 0x70, v);
    let mut a1 = 0i32;
    if (m.r32(s4 + 0x198) as i32) > 0 {
        loop {
            let s7 = m.r32(s4 + 0x19c).wrapping_add((a1 as u32).wrapping_mul(0x150));
            for s3 in 0..2u32 {
                let s0 = s7 + s3.wrapping_mul(0xa0);
                let a2 = m.r32(s0);
                m.w32(s0 + 4, 0xffff_ffff);
                if a2 != 0xffff_ffff {
                    let s1 = m.call(0x0015_e938, &[0x003d_d490, 1, a2, s4], &[]).0;
                    if s1 != 0 {
                        if m.call(0x0013_2b10, &[s1, 0xa], &[]).0 != 0 {
                            h14_1de5d8_path(m, s4, s0, s1, sp);
                        } else {
                            let hit = if m.call(0x0013_2b10, &[s1, 3], &[]).0 != 0 {
                                true
                            } else {
                                m.call(0x0013_2b10, &[s1, 0x30], &[]).0 != 0
                            };
                            if hit {
                                m.w32(s0 + 4, 0);
                                let q = m.call(0x001a_62e8, &[s1], &[]).1;
                                m.wv(s0 + 0x10, q);
                            }
                        }
                    }
                }
                if m.r32(s0 + 4) == 0xffff_ffff {
                    m.w32(s0 + 4, 0);
                    let q = m.call(0x001a_62e8, &[s4], &[]).1;
                    m.wv(s0 + 0x10, q);
                }
                match m.r32(s0 + 4) {
                    0 => {
                        let p = m.rv(s0 + 0x10);
                        h14_bounds(m, sp, p);
                        let w = m.r32(s4 + 0x60);
                        let q = m.call(0x0022_fc48, &[0, w], &[p]).1;
                        m.wv(s0 + 0x10, q);
                    }
                    1 => {
                        let o = m.r32(s0 + 0x10);
                        let p = vslot(m, o, 30, &[], &[]).0;
                        for k in 0..4 {
                            let q = m.rv(p + 16 * k);
                            m.wv(sp + 0x20 + 16 * k, q);
                        }
                        let r = m.rm(sp + 0x20);
                        let a1p = m.r32(s0 + 0x10);
                        let mut t0 = 0i32;
                        if (m.r32(a1p + 0xe8) as i32) > 0 {
                            loop {
                                let e = m.r32(a1p + 0xf4).wrapping_add((t0 as u32) << 4);
                                let p = m.rv(e);
                                t0 += 1;
                                let mut w = h14_rot(&r, p, p[3]);
                                for k in 0..3 {
                                    w[k] += r[3][k];
                                }
                                h14_bounds(m, sp, w);
                                if !(t0 < m.r32(a1p + 0xe8) as i32) {
                                    break;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            a1 += 1;
            if !(a1 < m.r32(s4 + 0x198) as i32) {
                break;
            }
        }
    }
    if m.r32(s4 + 0x198) != 0 {
        m.w32(s4 + 0xe0, 1);
        m.w32(sp, 0xb0);
        m.w32(sp + 4, 0x10);
        let s0 = m.call(0x001a_e2a0, &[sp], &[]).0;
        m.w32(s4 + 0xe4, s0);
        let w = m.r32(s4 + 0x60);
        m.w32(s0, w);
        m.w32(s0 + 4, s4);
        let f = m.r32(s0 + 0x14) | (1 << 16);
        let x = m.r32(s4 + 0x18);
        m.w32(s0 + 0x78, 2);
        m.w32(s0 + 0xc, x);
        m.w32(s0 + 0x14, f);
        m.w32(sp + 4, 0x10);
        m.w32(sp, 0x20);
        let b = m.call(0x001a_e2a0, &[sp], &[]).0;
        m.w32(s0 + 0x7c, b);
        m.call(0x0016_6270, &[s0 + 0x78, 0, s0 + 0x90], &[]);
        let mn = m.rv(sp + 0x60);
        let p = m.r32(s0 + 0x94);
        m.wv(p, mn);
        let mx = m.rv(sp + 0x70);
        let p = m.r32(s0 + 0x94);
        m.wv(p + 16, mx);
        let w = m.r32(s4 + 0x60);
        let s1 = m.r32(s0 + 0x7c);
        let q = m.call(0x0022_fc48, &[0, w], &[mn]).1;
        m.wv(s1, q);
        let mx = m.rv(sp + 0x70);
        let w = m.r32(s4 + 0x60);
        let s1 = m.r32(s0 + 0x7c);
        let q = m.call(0x0022_fc48, &[0, w], &[mx]).1;
        m.wv(s1 + 16, q);
    }
    if (m.r32(s4 + 0x2c) >> 28) & 1 != 0 {
        m.call(0x001f_2b20, &[s4], &[]);
    }
    stack_pop(m, 0x140);
}

/// FUN_001de5d8 path-target branch (class 0xa): speed profile from (frac, speed, accel, kind) at s0+0x18.
fn h14_1de5d8_path<M: QMem + ?Sized>(m: &mut M, s4: u32, s0: u32, s1: u32, sp: u32) {
    for k in 0..4 {
        let w = m.r32(s0 + 0x18 + 4 * k);
        m.w32(sp + 4 * k, w);
    }
    let mut s5 = false;
    let a = m.rf(sp);
    let f1 = if a < 0.0 { 0.0 } else if 1.0 < a { 1.0 } else { a };
    let b = m.rf(sp + 4);
    m.wf(sp, f1);
    if b < 0.0 {
        s5 = true;
        m.wf(sp + 4, -b);
    }
    let c = m.rf(sp + 8).abs();
    m.wf(sp + 8, c);
    let k = m.r32(sp + 0xc);
    if k != 8 && k != 0x10 && k != 0 {
        m.w32(sp + 0xc, 8);
    }
    let f20 = m.call(0x001e_6648, &[s1], &[]).1[0];
    let c = m.rf(sp + 8);
    if 0.0 < c {
        m.wf(sp + 4, f20 / c);
    } else {
        let b = m.rf(sp + 4);
        if 0.0 < b {
            m.wf(sp + 8, f20 / b);
        }
    }
    let f21 = m.rf(sp) * f20;
    if m.rf(sp + 4) == 0.0 || m.rf(sp + 8) == 0.0 {
        m.w32(s0 + 4, 0);
        m.call(0x001e_6f10, &[s1, s0 + 0x10, 0], &[f4(f21)]);
        return;
    }
    m.w32(s0 + 0x10, s1);
    m.w32(s0 + 4, 1);
    m.wf(s0 + 0x14, f21);
    let s1 = s0 + 0x1c;
    m.call(0x0023_33b4, &[s1, 0, 0x78], &[]);
    let kind = m.r32(sp + 0xc);
    let lin = |m: &mut M| {
        let v1 = s1;
        let (f1, f0);
        if s5 {
            m.wf(s0 + 0x1c, f20);
            let b = m.rf(sp + 4);
            m.wf(v1 + 4, -b);
            f1 = 1.0 - m.rf(sp);
            f0 = m.rf(sp + 8);
        } else {
            let b = m.rf(sp + 4);
            m.wf(v1 + 4, b);
            f1 = m.rf(sp);
            f0 = m.rf(sp + 8);
        }
        m.wf(v1 + 0x10, f1 * f0);
        let c = m.rf(sp + 8);
        m.wf(s1 + 0xc, c);
    };
    match kind {
        8 => {
            m.w32(s0 + 0x18, 1);
            lin(m);
            m.w32(s0 + 0x94, 0);
        }
        0 => {
            m.w32(s0 + 0x18, 2);
            lin(m);
            let v1 = s0 + 0x30;
            if s5 {
                m.w32(s0 + 0x30, 0);
            } else {
                m.wf(s0 + 0x30, f20);
            }
            m.w32(v1 + 0x10, 0);
            m.w32(v1 + 4, 0);
            m.w32(s0 + 0x94, 0);
        }
        0x10 => {
            let f1 = m.rf(0x002c_3610);
            if f1 * 3.0 <= f20 {
                m.wf(sp + 0x10, f1);
            } else {
                m.wf(sp + 0x10, f20 * f32::from_bits(0x3e4c_cccd));
            }
            let f1 = m.rf(sp + 0x10);
            let mut f5 = m.rf(sp + 8);
            let f3 = f1 + f1;
            m.wf(sp + 0x14, f20 - f1);
            let f4v = f3 * f5 / (f20 + f3);
            f5 = f5 - (f4v + f4v);
            let f0 = (f20 - f3) / f5;
            let f2 = f0 * f0 / f3;
            m.wf(sp + 4, f0);
            m.w32(s0 + 0x18, 6);
            m.wf(s1 + 0xc, f4v);
            m.wf(s1 + 8, f2);
            let f3 = -f2;
            let x = m.rf(sp + 0x10);
            m.wf(s0 + 0x30, x);
            let y = m.rf(sp + 4);
            m.wf(s0 + 0x3c, f5);
            m.wf(s0 + 0x34, y);
            let x = m.rf(sp + 0x14);
            m.wf(s0 + 0x44, x);
            let y = m.rf(sp + 4);
            m.wf(s0 + 0x50, f4v);
            m.wf(s0 + 0x48, y);
            m.wf(s0 + 0x4c, f3);
            m.wf(s0 + 0x58, f20);
            m.wf(s0 + 0x64, f4v);
            m.wf(s0 + 0x60, f3);
            let x = m.rf(sp + 0x14);
            m.wf(s0 + 0x6c, x);
            let y = m.rf(sp + 4);
            m.wf(s0 + 0x78, f5);
            m.wf(s0 + 0x70, -y);
            let x = m.rf(sp + 0x10);
            m.wf(s0 + 0x80, x);
            let y = m.rf(sp + 4);
            m.wf(s0 + 0x8c, f4v);
            m.wf(s0 + 0x88, f2);
            m.wf(s0 + 0x84, -y);
            m.w32(s0 + 0x94, 0);
            loop {
                let a0 = m.r32(s0 + 0x94) as i32;
                let v1 = s0.wrapping_add((a0 as u32).wrapping_mul(0x14)) + 0x1c;
                if f21 < m.rf(v1 + 0x14) {
                    let f12 = (2.0 * (f21 - m.rf(v1))) / (m.rf(v1 + 4) + m.rf(v1 + 0x18));
                    if s5 {
                        let v0 = 5i32.wrapping_sub(a0);
                        m.w32(s0 + 0x94, v0 as u32);
                        let v1 = s0.wrapping_add((v0 as u32).wrapping_mul(0x14)) + 0x1c;
                        let x = m.rf(v1 + 0xc) - f12;
                        m.wf(v1 + 0x10, x);
                    } else {
                        m.wf(v1 + 0x10, f12);
                    }
                    break;
                }
                let n = a0 + 1;
                m.w32(s0 + 0x94, n as u32);
                if !(n < 3) {
                    break;
                }
            }
        }
        _ => {}
    }
    let v1 = m.r32(s4 + 0x40);
    if (v1 ^ 1) & 1 != 0 {
        let v = m.r32(v1) | 2;
        m.w32(v1, v);
    } else {
        m.w32(s4 + 0x40, v1 | 2);
    }
}

/// embedded-object virtual call: obj at `base`, vtable word at `base + vtw`, fn at vt+0x1c, adj at vt+0x18
fn h14_emb<M: QMem + ?Sized>(m: &mut M, base: u32, vtw: u32, a: &[u32], v: &[V4]) -> (u32, V4) {
    let vt = m.r32(base + vtw);
    let adj = m.r16(vt + 0x18) as i16 as i32;
    let f = m.r32(vt + 0x1c);
    let mut args = vec![base.wrapping_add(adj as u32)];
    args.extend_from_slice(a);
    m.call(f, &args, v)
}

/// FUN_0019b110 helper: `|q - *(sp+0x10)|`
fn h14_dist<M: QMem + ?Sized>(m: &mut M, sp: u32, q: V4) -> f32 {
    let p = m.rv(sp + 0x10);
    let d = [q[0] - p[0], q[1] - p[1], q[2] - p[2], 0.0];
    h14_dot3(d, d)
}

/// FUN_0019b110 helper: cost gate at `slot` (f12 = *slot): false if 0x2bf8e0 <= cost or the
/// FUN_0019b088-adjusted cost (k = *(kaddr)) is not below `best`
fn h14_gate<M: QMem + ?Sized>(m: &mut M, s1: u32, s0: u32, slot: u32, kaddr: u32, best: f32) -> bool {
    let f12 = m.rf(slot);
    if m.rf(0x002b_f8e0) <= f12 {
        return false;
    }
    let k = m.rf(kaddr);
    let f0 = m.call(0x0019_b088, &[s1, s0], &[f4(f12), f4(k)]).1[0];
    m.wf(slot, f0);
    !(best <= f0)
}

/// FUN_0019b110 helper: FUN_0019ae00(s1, a1, a2, t3; *(sp+0x10), t0, *(sp+0x30), *(sp+0x50), *(sp+0xe0))
fn h14_ae00<M: QMem + ?Sized>(m: &mut M, sp: u32, s1: u32, a1: u32, a2: u32, t3: u32, t0: V4) -> u32 {
    let a3 = m.rv(sp + 0x10);
    let t1 = m.rv(sp + 0x30);
    let t2 = m.rv(sp + 0x50);
    let f = m.rf(sp + 0xe0);
    m.call(0x0019_ae00, &[s1, a1, a2, t3], &[a3, t0, t1, t2, f4(f)]).0
}

/// FUN_0019b110 helper: *(sp+0x58) = *(sp+off) + *(sp+0xe0) * (*(*s1+0xe8)+0x88)
fn h14_h58<M: QMem + ?Sized>(m: &mut M, sp: u32, s1: u32, off: u32) {
    let o = m.r32(s1);
    let g = m.r32(o + 0xe8);
    let x = m.rf(sp + off) + m.rf(sp + 0xe0) * m.rf(g + 0x88);
    m.wf(sp + 0x58, x);
}

/// FUN_0019b110 helper: accept = *(sp+0x40) = *(sp+0x30) (+ *(*s1+0x240) if `add`), *(sp+0x60) = *(sp+0x50)
fn h14_keep<M: QMem + ?Sized>(m: &mut M, sp: u32, s1: u32, add: bool) {
    let mut a = m.rv(sp + 0x30);
    if add {
        let o = m.r32(s1);
        let b = m.rv(o + 0x240);
        for k in 0..4 {
            a[k] += b[k];
        }
    }
    let b = m.rv(sp + 0x50);
    m.wv(sp + 0x40, a);
    m.wv(sp + 0x60, b);
}

/// FUN_0019b110: pick the best traversal target (jump surfaces, rails, ropes, ledges, poles, ...)
/// for the player state a0 (a1 = current target or 0), cheapest cost wins; commits via
/// FUN_00195fb8(a0, kind, target).
pub fn ee_sub_19b110<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let s1 = a0;
    let s3 = a1;
    let sp = stack_push(m, 0x1f0);
    let mut s4: u32 = 0;
    let mut s5: i32 = -1;
    let mut f26: f32 = 0.0;
    let mut f27: f32 = 0.0;
    let o = m.r32(s1);
    let p = m.call(0x001a_62e8, &[o], &[]).1;
    m.wv(sp + 0x10, p);
    let o = m.r32(s1);
    let q = vslot(m, o, 32, &[], &[]).1;
    m.wv(sp + 0x20, q);
    let o = m.r32(s1);
    let s6 = m.r32(o + 0x1000);
    m.call(0x0023_33b4, &[sp + 0x70, 0, 0xc], &[]);
    let mut f23 = m.rf(if s3 != 0 { 0x002a_65b8 } else { 0x002a_65bc });
    let q = m.call(0x0019_a750, &[s1, 4], &[]).1;
    m.wv(sp + 0x80, q);
    let mut f24: f32 = 0.0;
    let o = m.r32(s1);
    let unit = |m: &mut M| m.rv(0x002a_2dd0);
    let below = |m: &mut M, r: u32| m.rf(r) < m.rf(sp + 0x18) - m.rf(sp + 0x98);
    if m.call(0x0013_2b10, &[o, 0x35], &[]).0 != 0 {
        // jump surfaces (0x3e13d8)
        let mut s0 = m.r32(0x003e_13d8);
        while s0 != 0 {
            'n: {
                if m.r32(s0 + 0xfc) & s6 == 0 {
                    break 'n;
                }
                let o = m.r32(s1);
                h14_emb(m, s0 + 0xe0, 0x20, &[o, sp + 0x90, 0], &[f4(0.0)]);
                if s0 != s3 && below(m, s0 + 0xe8) {
                    break 'n;
                }
                let (va, vb, vc) = (m.rv(sp + 0x80), m.rv(sp + 0x90), unit(m));
                m.call(0x0019_a4a0, &[s1, s0, sp + 0xe0, sp + 0xe4, sp + 0x30, sp + 0x50], &[va, vb, vc]);
                h14_h58(m, sp, s1, 0x88);
                if s0 != s3 {
                    if !h14_gate(m, s1, s0, sp + 0xe4, 0x002b_f8e4, f23) {
                        break 'n;
                    }
                    let o = m.r32(s1);
                    let a2 = m.r32(o + 0xec);
                    let t0 = m.rv(sp + 0x80);
                    if h14_ae00(m, sp, s1, 0x0020_f948, a2, 0, t0) != 0 {
                        break 'n;
                    }
                }
                s5 = 9;
                s4 = s0;
                h14_keep(m, sp, s1, false);
                f23 = f23.min(m.rf(sp + 0xe4));
                f24 = m.rf(sp + 0xe0);
            }
            s0 = m.r32(s0 + 0x104);
        }
        // rails (0x3e13cc)
        let mut s0 = m.r32(0x003e_13cc);
        while s0 != 0 {
            'n: {
                if m.r32(s0 + 0x1ac) & s6 == 0 {
                    break 'n;
                }
                if s0 != s3 {
                    let o = m.r32(s1);
                    let w = m.r32(o + 0x218);
                    let v = m.rv(w);
                    m.call(0x001e_6f60, &[s0, 0, sp + 0x90, 0, 0, 0], &[v]);
                    if below(m, s0 + 0x198) {
                        break 'n;
                    }
                }
                let o = m.r32(s1);
                m.w32(sp + 0xa4, o + 0x2c0);
                m.w32(sp + 0xa0, o + 0x4c0);
                let v = m.rv(sp + 0x80);
                m.wv(sp + 0xb0, v);
                m.w32(sp + 0xa8, s0);
                m.call(0x001e_7410, &[s0, 0x0019_aa58, sp + 0xa0, 0, 0, sp + 0xe8, 0], &[]);
                let f22 = m.rf(sp + 0xe8);
                let f21 = m.call(0x001e_77e8, &[s0], &[]).1[0];
                let f20 = m.call(0x001e_64c8, &[s0], &[f4(50.0)]).1[0];
                let l = m.call(0x001e_6648, &[s0], &[]).1[0];
                let f0 = m.call(0x001e_64c8, &[s0], &[f4(l - 50.0)]).1[0];
                m.call(0x0011_d9e0, &[0x0019_aa58, sp + 0xa0, sp + 0xec, sp + 0xe4], &[f4(f22), f4(f21), f4(f20), f4(f0)]);
                if s0 != s3 && !h14_gate(m, s1, s0, sp + 0xe4, 0x002b_f8e8, f23) {
                    break 'n;
                }
                let a = m.r32(sp + 0xa0);
                let t = m.rf(sp + 0xec);
                m.call(0x0019_3a68, &[a, s0, sp + 0x90, sp + 0xc0], &[f4(t)]);
                let (va, vb, vc) = (m.rv(sp + 0x80), m.rv(sp + 0x90), m.rv(sp + 0xc0));
                m.call(0x0019_a4a0, &[s1, s0, sp + 0xe0, 0, sp + 0x30, sp + 0x50], &[va, vb, vc]);
                h14_h58(m, sp, s1, 0x88);
                if s0 != s3 {
                    let o = m.r32(s1);
                    let a2 = m.r32(o + 0xec);
                    let t0 = m.rv(sp + 0x80);
                    if h14_ae00(m, sp, s1, 0x0020_f948, a2, 0, t0) != 0 {
                        break 'n;
                    }
                }
                f23 = f23.min(m.rf(sp + 0xe4));
                s4 = s0;
                f27 = m.rf(sp + 0xec);
                s5 = 0xa;
                h14_keep(m, sp, s1, false);
                f24 = m.rf(sp + 0xe0);
                f26 = m.call(0x001e_63c0, &[s4], &[f4(f27)]).1[0];
            }
            s0 = m.r32(s0 + 0x1b4);
        }
        // swing targets (0x3e13b4)
        let mut s0 = m.r32(0x003e_13b4);
        while s0 != 0 {
            'n: {
                if s0 != s3 {
                    if s3 != 0 || m.r32(s0 + 0xe8) & 1 == 0 {
                        break 'n;
                    }
                    if m.call(0x0013_2b10, &[s0, 0x13], &[]).0 == 0 {
                        break 'n;
                    }
                }
                if m.r32(s0 + 0x11c) & s6 == 0 {
                    break 'n;
                }
                let p = m.call(0x001a_62e8, &[s0], &[]).1;
                m.wv(sp + 0x90, p);
                if s0 != s3 {
                    if below(m, s0 + 0x108) {
                        break 'n;
                    }
                    if f32::from_bits(0x4974_2400) < h14_dist(m, sp, p) {
                        break 'n;
                    }
                }
                let u = unit(m);
                m.wv(sp + 0xb0, u);
                m.w32(sp + 0xa0, s1);
                m.w32(sp + 0xa4, s0);
                m.call(0x0011_d9e0, &[0x0019_abb8, sp + 0xa0, sp + 0xe0, sp + 0xe4], &[f4(0.5), f4(0.25), f4(0.25), f4(3.0)]);
                let (a, b) = (m.r32(sp + 0xa0), m.r32(sp + 0xa4));
                let (v, t) = (m.rv(sp + 0xb0), m.rf(sp + 0xe0));
                m.call(0x0019_c300, &[a, b, 0, sp + 0x30, sp + 0x50], &[v, f4(t)]);
                if s0 != s3 {
                    if !h14_gate(m, s1, s0, sp + 0xe4, 0x002b_f8ec, f23) {
                        break 'n;
                    }
                    let o = m.r32(s1);
                    let a2 = m.r32(o + 0xec);
                    let t0 = m.rv(sp + 0x20);
                    if h14_ae00(m, sp, s1, 0x0020_f948, a2, sp + 0x90, t0) != 0 {
                        break 'n;
                    }
                }
                s5 = 0xe;
                s4 = s0;
                h14_keep(m, sp, s1, true);
                f23 = m.rf(sp + 0xe4).min(f23);
                f24 = m.rf(sp + 0xe0);
            }
            s0 = m.r32(s0 + 0xe0);
        }
        // 0x3e142c list
        let mut s0 = m.r32(0x003e_142c);
        while s0 != 0 {
            'n: {
                let v = m.rv(sp + 0x10);
                m.call(0x0018_5310, &[s0, 0, 0, sp + 0x90, 0], &[v]);
                if s0 != s3 {
                    if below(m, s0 + 0x108) {
                        break 'n;
                    }
                    let q = m.rv(sp + 0x90);
                    if 1000.0 < h14_dist(m, sp, q).sqrt() {
                        break 'n;
                    }
                }
                let v = m.rv(sp + 0x80);
                m.wv(sp + 0xc0, v);
                m.w32(sp + 0xb0, s1);
                m.w32(sp + 0xb4, s0);
                m.call(0x0011_d9e0, &[0x0019_ac88, sp + 0xb0, sp + 0xe0, sp + 0xf0], &[f4(1.0), f4(0.25), f4(0.25), f4(2.0)]);
                if s0 != s3 && !h14_gate(m, s1, s0, sp + 0xf0, 0x002b_f908, f23) {
                    break 'n;
                }
                let o = m.r32(s1);
                let g = m.r32(o + 0xe8);
                let (va, vb, vc) = (m.rv(sp + 0x10), m.rv(sp + 0x80), m.rv(g + 0x80));
                let (fa, fb) = (m.rf(o + 0x1f30), m.rf(sp + 0xe0));
                m.call(0x0018_5d18, &[s0, sp + 0xa0, sp + 0x30, sp + 0x50], &[va, vb, vc, f4(fa), f4(fb)]);
                if s0 != s3 {
                    let o = m.r32(s1);
                    let a2 = m.r32(o + 0xec);
                    let t0 = m.rv(sp + 0x20);
                    if h14_ae00(m, sp, s1, 0x0020_f948, a2, 0, t0) != 0 {
                        break 'n;
                    }
                }
                s5 = 0x10;
                s4 = s0;
                h14_keep(m, sp, s1, true);
                f23 = m.rf(sp + 0xf0).min(f23);
                f24 = m.rf(sp + 0xe0);
                for k in 0..3 {
                    let w = m.r32(sp + 0xa0 + 4 * k);
                    m.w32(sp + 0x70 + 4 * k, w);
                }
            }
            s0 = m.r32(s0 + 0x100);
        }
    }
    let o = m.r32(s1);
    if m.call(0x0013_2b10, &[o, 0x37], &[]).0 != 0 {
        let a = m.r32(0x002f_7900);
        if a != 0 && m.r32(a + 0x13d0) == 0xf && m.r32(a + 0x1914) == 0 {
            'n: {
                let t = m.r32(a + 0x191c);
                let p = m.call(0x001a_62e8, &[t], &[]).1;
                m.wv(sp + 0x90, p);
                let t = m.r32(0x002f_7900);
                let tt = m.r32(t + 0x191c);
                let (va, vc) = (m.rv(sp + 0x80), unit(m));
                m.call(0x0019_a4a0, &[s1, tt, sp + 0xe0, sp + 0xe4, sp + 0x30, sp + 0x50], &[va, p, vc]);
                h14_h58(m, sp, s1, 0x88);
                let a1 = m.r32(0x002f_7900);
                if m.r32(a1 + 0x191c) != s3 {
                    if !h14_gate(m, s1, a1, sp + 0xe4, 0x002b_f904, f23) {
                        break 'n;
                    }
                    let o = m.r32(s1);
                    let t = m.r32(0x002f_7900);
                    let x = m.r32(o + 0xec);
                    m.w32(sp + 0xa0, x);
                    let x = m.r32(t + 0xec);
                    m.w32(sp + 0xa4, x);
                    m.w32(sp + 0xb0, 2);
                    m.w32(sp + 0xb4, sp + 0xa0);
                    let t0 = m.rv(sp + 0x80);
                    if h14_ae00(m, sp, s1, 0x0019_b0a8, sp + 0xb0, 0, t0) != 0 {
                        break 'n;
                    }
                }
                s5 = 0x12;
                h14_keep(m, sp, s1, false);
                let t = m.r32(0x002f_7900);
                s4 = m.r32(t + 0x191c);
                f24 = m.rf(sp + 0xe0);
                f23 = f23.min(m.rf(sp + 0xe4));
            }
        }
    }
    // ropes (0x3e13c0)
    let mut s0 = m.r32(0x003e_13c0);
    while s0 != 0 {
        'n: {
            let o = m.r32(s1);
            if m.r32(s0 + 0x1ac) & m.r32(o + 0x1000) == 0 {
                break 'n;
            }
            let v = m.rv(sp + 0x10);
            m.call(0x001e_6f60, &[s0, 0, sp + 0x90, 0, 0, sp + 0xf4], &[v]);
            if s0 != s3 {
                if below(m, s0 + 0x198) {
                    break 'n;
                }
                let q = m.rv(sp + 0x90);
                if 1000.0 < h14_dist(m, sp, q).sqrt() {
                    break 'n;
                }
            }
            let f20 = m.rf(sp + 0xf4);
            m.w32(sp + 0xa0, s1);
            m.w32(sp + 0xa4, s0);
            let l = m.call(0x001e_6648, &[s0], &[]).1[0];
            m.call(0x0011_d9e0, &[0x0019_abf0, sp + 0xa0, sp + 0xf8, sp + 0xfc], &[f4(f20), f4(100.0), f4(50.0), f4(l - 50.0)]);
            if s0 != s3 && !h14_gate(m, s1, s0, sp + 0xfc, 0x002b_f8f0, f23) {
                break 'n;
            }
            let t = m.rf(sp + 0xf8);
            m.w32(sp + 0xb0, s1);
            m.w32(sp + 0xb4, s0);
            m.call(0x001e_6f10, &[s0, sp + 0x90, 0], &[f4(t)]);
            let v = m.rv(sp + 0x90);
            let q = m.call(0x001a_6f00, &[0, s0], &[v]).1;
            m.wv(sp + 0xc0, q);
            m.call(0x0011_d9e0, &[0x0019_abb8, sp + 0xb0, sp + 0xe0, 0], &[f4(0.5), f4(0.25), f4(0.25), f4(3.0)]);
            let (a, b) = (m.r32(sp + 0xb0), m.r32(sp + 0xb4));
            let (v, t) = (m.rv(sp + 0xc0), m.rf(sp + 0xe0));
            m.call(0x0019_c300, &[a, b, 0, sp + 0x30, sp + 0x50], &[v, f4(t)]);
            if s0 != s3 {
                let o = m.r32(s1);
                let a2 = m.r32(o + 0xec);
                let t0 = m.rv(sp + 0x20);
                if h14_ae00(m, sp, s1, 0x0020_f948, a2, sp + 0x90, t0) != 0 {
                    break 'n;
                }
            }
            s5 = 0xf;
            s4 = s0;
            h14_keep(m, sp, s1, true);
            f23 = m.rf(sp + 0xfc).min(f23);
            f24 = m.rf(sp + 0xe0);
            f26 = m.rf(sp + 0xf8);
        }
        s0 = m.r32(s0 + 0x1b4);
    }
    // poles (0x3e13fc)
    let mut s0 = m.r32(0x003e_13fc);
    while s0 != 0 {
        'n: {
            if m.r32(s0 + 0xfc) & s6 == 0 {
                break 'n;
            }
            let p = m.call(0x001a_62e8, &[s0], &[]).1;
            m.wv(sp + 0x90, p);
            if below(m, s0 + 0xe8) {
                break 'n;
            }
            let a = m.rv(sp + 0x10);
            m.wv(sp + 0xc0, [a[0] - p[0], a[1] - p[1], a[2] - p[2], a[3] - p[3]]);
            let b = m.rv(sp + 0x20);
            m.wv(sp + 0xb0, b);
            m.w32(sp + 0xa0, s1);
            m.w32(sp + 0xa4, s0);
            let (y, x) = (m.rf(sp + 0xc4), m.rf(sp + 0xc0));
            let ang = m.call(0x0024_58f0, &[], &[f4(y), f4(x)]).1[0];
            let f12 = ang * m.rf(s0 + 0x10c);
            let (f13, f14, f15) = (m.rf(0x002b_f914), m.rf(0x002a_65b8), m.rf(0x002a_65bc));
            m.call(0x0011_d9e0, &[0x0019_aab8, sp + 0xa0, sp + 0x100, sp + 0xe4], &[f4(f12), f4(f13), f4(f14), f4(f15)]);
            let c = m.rf(sp + 0xe4) * m.rf(0x002b_f90c);
            m.wf(sp + 0xe4, c);
            if s0 != s3 && !h14_gate(m, s1, s0, sp + 0xe4, 0x002b_f8f8, f23) {
                break 'n;
            }
            let o = m.r32(s1);
            let t = m.rf(sp + 0x100);
            h14_emb(m, s0 + 0xe0, 0x20, &[o, sp + 0xd0, 0], &[f4(t)]);
            let (va, vb, vc) = (m.rv(sp + 0x20), m.rv(sp + 0xd0), unit(m));
            m.call(0x0019_a4a0, &[s1, s0, sp + 0xe0, 0, sp + 0x30, sp + 0x50], &[va, vb, vc]);
            h14_h58(m, sp, s1, 0x28);
            if s0 != s3 {
                let o = m.r32(s1);
                let a2 = m.r32(o + 0xec);
                let t0 = m.rv(sp + 0x20);
                if h14_ae00(m, sp, s1, 0x0020_f948, a2, 0, t0) != 0 {
                    break 'n;
                }
            }
            s5 = 0xc;
            s4 = s0;
            h14_keep(m, sp, s1, false);
            f23 = m.rf(sp + 0xe4).min(f23);
            f24 = m.rf(sp + 0xe0);
            f26 = m.rf(sp + 0x100);
        }
        s0 = m.r32(s0 + 0x104);
    }
    // ledges (0x3e13f0)
    let mut s0 = m.r32(0x003e_13f0);
    while s0 != 0 {
        'n: {
            if m.r32(s0 + 0x1ac) & s6 == 0 {
                break 'n;
            }
            let v = m.rv(sp + 0x10);
            m.call(0x001e_6f60, &[s0, 0, sp + 0x90, 0, 0, sp + 0x104], &[v]);
            if below(m, s0 + 0x198) {
                break 'n;
            }
            let b = m.rv(sp + 0x20);
            m.wv(sp + 0xb0, b);
            let f20 = m.rf(sp + 0x104);
            m.w32(sp + 0xa0, s1);
            m.w32(sp + 0xa4, s0);
            let l = m.call(0x001e_6648, &[s0], &[]).1[0];
            let f13 = m.rf(0x002b_f918);
            m.call(0x0011_d9e0, &[0x0019_ab38, sp + 0xa0, sp + 0x108, sp + 0xe4], &[f4(f20), f4(f13), f4(30.0), f4(l - 30.0)]);
            let c = m.rf(sp + 0xe4) * m.rf(0x002b_f90c);
            m.wf(sp + 0xe4, c);
            if s0 != s3 && !h14_gate(m, s1, s0, sp + 0xe4, 0x002b_f8f4, f23) {
                break 'n;
            }
            let o = m.r32(s1);
            let t = m.rf(sp + 0x108);
            h14_emb(m, s0 + 0x190, 0x20, &[o, sp + 0xc0, 0], &[f4(t)]);
            let (va, vb, vc) = (m.rv(sp + 0x20), m.rv(sp + 0xc0), unit(m));
            m.call(0x0019_a4a0, &[s1, s0, sp + 0xe0, 0, sp + 0x30, sp + 0x50], &[va, vb, vc]);
            h14_h58(m, sp, s1, 0x28);
            if s0 != s3 {
                let o = m.r32(s1);
                let a2 = m.r32(o + 0xec);
                let t0 = m.rv(sp + 0x20);
                if h14_ae00(m, sp, s1, 0x0020_f948, a2, 0, t0) != 0 {
                    break 'n;
                }
            }
            s5 = 0xb;
            s4 = s0;
            h14_keep(m, sp, s1, false);
            f23 = m.rf(sp + 0xe4).min(f23);
            f24 = m.rf(sp + 0xe0);
            f26 = m.rf(sp + 0x108);
        }
        s0 = m.r32(s0 + 0x1b4);
    }
    // 0x3e1408 list (like jump surfaces, facing-based)
    let mut s0 = m.r32(0x003e_1408);
    while s0 != 0 {
        'n: {
            if m.r32(s0 + 0xfc) & s6 == 0 {
                break 'n;
            }
            let o = m.r32(s1);
            h14_emb(m, s0 + 0xe0, 0x20, &[o, sp + 0x90, 0], &[f4(0.0)]);
            if s0 != s3 && below(m, s0 + 0xe8) {
                break 'n;
            }
            let (va, vb, vc) = (m.rv(sp + 0x20), m.rv(sp + 0x90), unit(m));
            m.call(0x0019_a4a0, &[s1, s0, sp + 0xe0, sp + 0xe4, sp + 0x30, sp + 0x50], &[va, vb, vc]);
            h14_h58(m, sp, s1, 0x28);
            if s0 != s3 {
                if !h14_gate(m, s1, s0, sp + 0xe4, 0x002b_f8fc, f23) {
                    break 'n;
                }
                let o = m.r32(s1);
                let a2 = m.r32(o + 0xec);
                let t0 = m.rv(sp + 0x20);
                if h14_ae00(m, sp, s1, 0x0020_f948, a2, 0, t0) != 0 {
                    break 'n;
                }
            }
            s5 = 0xd;
            s4 = s0;
            f23 = f23.min(m.rf(sp + 0xe4));
            h14_keep(m, sp, s1, false);
            f24 = m.rf(sp + 0xe0);
        }
        s0 = m.r32(s0 + 0x104);
    }
    if s5 != -1 {
        let o = m.r32(s1);
        let v1 = m.r32(o + 0x58);
        let x = m.rf(0x002d_ded0) + f24;
        m.wf(v1 + 0xa8, x);
        let o = m.r32(s1);
        let a = m.rv(sp + 0x40);
        let b = m.rv(o + 0x240);
        m.wv(v1 + 0xc0, [a[0] - b[0], a[1] - b[1], a[2] - b[2], a[3] - b[3]]);
        let c = m.rv(sp + 0x60);
        m.wv(v1 + 0xd0, c);
        m.wf(s1 + 0x10, f27);
        m.wf(s1 + 0x14, f26);
        m.call(0x0019_5fb8, &[s1, s5 as u32, s4], &[]);
    }
    stack_pop(m, 0x1f0);
}
