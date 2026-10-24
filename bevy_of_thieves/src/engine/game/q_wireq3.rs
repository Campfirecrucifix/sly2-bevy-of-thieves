//! WIRE-Q3: 1:1 queue ports (against `QMem`) of the functions whose port_tracking rust_name names a
//! different address with different code bytes (dropped from the live dispatch by the WIRE-Q2
//! identical-bytes rule). Callees go through `m.call` / `vslot` (the live dispatch routes ported ones).
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

const Z: V4 = [0.0; 4];

fn fv(f: f32) -> V4 {
    [f, 0.0, 0.0, 0.0]
}

/// FUN_00101f90(out, a1, a2, a3): 8-byte key DAT_002a3a80 on the stack, FUN_00201360(tmp, 2, a3, &key,
/// &x, &y), FUN_00129ec8 / 0012a0b0(tmp, 0x24), FUN_001fb4f0(a1, x, y), FUN_001291a8(out, tmp), tmp dtor
pub fn ee_sub_101f90<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x70);
    let k0 = m.r32(0x002a_3a80);
    let k1 = m.r32(0x002a_3a84);
    m.w32(sp, k0);
    m.w32(sp + 4, k1);
    let s0 = sp + 0x10;
    m.call(0x0020_1360, &[s0, 2, a3, sp, sp + 0x20, sp + 0x24], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_a0b0, &[s0, 0x24], &[]);
    let x = m.rf(sp + 0x20);
    let y = m.rf(sp + 0x24);
    m.call(0x001f_b4f0, &[a1], &[fv(x), fv(y)]);
    m.call(0x0012_91a8, &[a0, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0x70);
    a0
}

fn q132d(m: &mut (impl QMem + ?Sized), ctor: u32, a0: u32, a1: u32, a2: u32) {
    let sp = stack_push(m, 0x50);
    m.call(ctor, &[sp, a2], &[]);
    m.call(0x0020_0d10, &[sp + 0x10, a0, a1, sp], &[]);
    m.call(0x0012_9ec8, &[sp + 0x10], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x50);
}

/// FUN_00132d28: tmp = FUN_00129398(a2); FUN_00200d10(&r, a0, a1, &tmp); dtor both
pub fn ee_sub_132d28<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    q132d(m, 0x0012_9398, a0, a1, a2)
}

/// FUN_00132de0: tmp = FUN_00129248(a2); FUN_00200d10(&r, a0, a1, &tmp); dtor both
pub fn ee_sub_132de0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    q132d(m, 0x0012_9248, a0, a1, a2)
}

/// FUN_00133590: empty
pub fn ee_sub_133590<M: QMem + ?Sized>(_m: &mut M) {}

/// FUN_00134310: signed max(a0, a1)
pub fn ee_sub_134310<M: QMem + ?Sized>(_m: &mut M, a0: u32, a1: u32) -> u32 {
    if (a1 as i32) < (a0 as i32) {
        a0
    } else {
        a1
    }
}

/// FUN_001374e0: a1 +0x94 = DAT_002bc9b4
pub fn ee_sub_1374e0<M: QMem + ?Sized>(m: &mut M, _a0: u32, a1: u32) {
    let f = m.rf(0x002b_c9b4);
    m.wf(a1 + 0x94, f);
}

/// FUN_0013b7b8: sort the *a1 words at a2 (FUN_002338c0 qsort, cmp FUN_0013b7a0) and drop adjacent
/// duplicates; *a1 = unique count
pub fn ee_sub_13b7b8<M: QMem + ?Sized>(m: &mut M, _a0: u32, a1: u32, a2: u32) {
    let n = m.r32(a1) as i32;
    if n < 2 {
        return;
    }
    m.call(0x0023_38c0, &[a2, n as u32, 4, 0x0013_b7a0], &[]);
    let end = a2 + 4 * n as u32;
    let mut d = a2;
    let mut s = a2;
    while s < end {
        let v0 = m.r32(d);
        let v1 = m.r32(s);
        s += 4;
        if v0 != v1 {
            d += 4;
            m.w32(d, v1);
        }
    }
    m.w32(a1, ((d - a2) as i32 >> 2) as u32 + 1);
}

/// FUN_00140ed8: 1 when +0xf20 == 1, +0x2dc == 0x15 or FUN_001c40c0(DAT_003d4a54); else
/// FUN_001a2230(+0xdf0, a1)
pub fn ee_sub_140ed8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    if m.r32(a0 + 0xf20) == 1 || m.r32(a0 + 0x2dc) == 0x15 {
        return 1;
    }
    let p = m.r32(0x003d_4a54);
    if p != 0 && m.call(0x001c_40c0, &[p], &[]).0 != 0 {
        return 1;
    }
    m.call(0x001a_2230, &[a0 + 0xdf0, a1], &[]).0
}

/// FUN_00144638: *a0 = a1
pub fn ee_sub_144638<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.w32(a0, a1);
}

/// FUN_00148738: *a0 = a1
pub fn ee_sub_148738<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.w32(a0, a1);
}

/// FUN_00148668: for each {oid, ptr} pair of the counted list at a0: ptr = FUN_0015e938(0x3dd490, 4,
/// oid, a1); slot 8 on each found
pub fn ee_sub_148668<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let mut i = 0i32;
    let mut s0 = a0 + 4;
    while i < m.r32(a0) as i32 {
        let oid = m.r32(s0);
        let r = m.call(0x0015_e938, &[0x003d_d490, 4, oid, a1], &[]).0;
        m.w32(s0, r);
        if r != 0 {
            vslot(m, r, 8, &[], &[]);
        }
        i += 1;
        s0 += 8;
    }
}

/// FUN_001490a0: FUN_00148a78(a0 + 0x150)
pub fn ee_sub_1490a0<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    m.call(0x0014_8a78, &[a0 + 0x150], &[]).0
}

/// FUN_00153638: identity (f12 -> f0)
pub fn ee_sub_153638<M: QMem + ?Sized>(_m: &mut M, f: f32) -> f32 {
    f
}

/// FUN_00153e60: position of +0xe0 (FUN_001a62e8) lifted by 80 * DAT_002a2dc0 when it is-a 0x38;
/// own position when +0xe0 is unset
pub fn ee_sub_153e60<M: QMem + ?Sized>(m: &mut M, a0: u32) -> V4 {
    let e = m.r32(a0 + 0xe0);
    if e == 0 {
        return m.call(0x001a_62e8, &[a0], &[]).1;
    }
    if m.call(0x0013_2b10, &[e, 0x38], &[]).0 == 0 {
        return m.call(0x001a_62e8, &[e], &[]).1;
    }
    let p = m.call(0x001a_62e8, &[e], &[]).1;
    let u = m.rv(0x002a_2dc0);
    let mut r = Z;
    for k in 0..4 {
        r[k] = p[k] + u[k] * 80.0;
    }
    r
}

/// FUN_00154528: FUN_0020fa10(a0) && FUN_00132ea0(a1 +4) == 0x31
pub fn ee_sub_154528<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    if m.call(0x0020_fa10, &[a0], &[]).0 == 0 {
        return 0;
    }
    let p = m.r32(a1 + 4);
    (m.call(0x0013_2ea0, &[p], &[]).0 == 0x31) as u32
}

/// FUN_00158e20(island, f12, f13): scratch frame (FUN_001ae478 / 001ae318 {n << 5, 0x10}); every
/// contact row (+8, 0x28 each) with impulse +0xc != 0 adds impulse * the two 0x20-strided axis pairs
/// of row +0x60 (0x60 each) into the body (+0x50 [j] +8) +0xf0 / +0x100; then FUN_0011e9f8 clamps each
/// body (+0x5c list, +0x58 count) +0xf0 by f12 and +0x100 by f13; FUN_001ae4a0(frame, 2)
pub fn ee_sub_158e20<M: QMem + ?Sized>(m: &mut M, a0: u32, f12: f32, f13: f32) {
    let sp = stack_push(m, 0x70);
    m.call(0x001a_e478, &[sp], &[]);
    let n = m.r32(a0 + 0x58);
    m.w32(sp + 0x10, n << 5);
    m.w32(sp + 0x14, 0x10);
    m.call(0x001a_e318, &[sp + 0x10], &[]);
    let mut i = 0i32;
    while i < m.r32(a0 + 4) as i32 {
        let a1 = m.r32(a0 + 8).wrapping_add((i.wrapping_mul(0x28)) as u32);
        if m.rf(a1 + 0xc) != 0.0 {
            let a3 = m.r32(a0 + 0x60).wrapping_add((i.wrapping_mul(0x60)) as u32);
            let mut ax = a3;
            for j in 0..2u32 {
                let b = m.r32(a3 + 0x50).wrapping_add(j << 6);
                let s0 = m.r32(b + 8);
                if s0 != 0 {
                    for (off, ao) in [(0xf0u32, 0u32), (0x100, 0x10)] {
                        let f = m.rf(a1 + 0xc);
                        let x = m.rv(s0 + off);
                        let d = m.rv(ax + ao);
                        let mut r = Z;
                        for k in 0..4 {
                            r[k] = x[k] + d[k] * f;
                        }
                        m.wv(s0 + off, r);
                    }
                }
                ax += 0x20;
            }
        }
        i += 1;
    }
    let mut k = 0i32;
    while k < m.r32(a0 + 0x58) as i32 {
        let s0 = { let t = m.r32(a0 + 0x5c); m.r32(t + 4 * k as u32) };
        k += 1;
        let v = m.rv(s0 + 0xf0);
        let r = m.call(0x0011_e9f8, &[], &[v, fv(f12)]).1;
        m.wv(s0 + 0xf0, r);
        let v = m.rv(s0 + 0x100);
        let r = m.call(0x0011_e9f8, &[], &[v, fv(f13)]).1;
        m.wv(s0 + 0x100, r);
    }
    m.call(0x001a_e4a0, &[sp, 2], &[]);
    stack_pop(m, 0x70);
}

/// FUN_00162908: FUN_00162a60(a0) when FUN_001633e8(a0)
pub fn ee_sub_162908<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    if m.call(0x0016_33e8, &[a0], &[]).0 != 0 {
        m.call(0x0016_2a60, &[a0], &[]);
    }
}

/// FUN_00178d80: +0x4c == 3
pub fn ee_sub_178d80<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    (m.r32(a0 + 0x4c) == 3) as u32
}

/// FUN_0017a9c0: +0x30 == -1
pub fn ee_sub_17a9c0<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    (m.r32(a0 + 0x30) == u32::MAX) as u32
}

/// FUN_0017dcf8: FUN_00178d80(+0x18) while 0 < +0x30 < 3, else 0
pub fn ee_sub_17dcf8<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let v = m.r32(a0 + 0x30) as i32;
    if v >= 3 || v <= 0 {
        return 0;
    }
    let p = m.r32(a0 + 0x18);
    m.call(0x0017_8d80, &[p], &[]).0
}

/// FUN_0017f3e0: a1 records (0x10 apart) at a2: first word = -1
pub fn ee_sub_17f3e0<M: QMem + ?Sized>(m: &mut M, _a0: u32, a1: u32, a2: u32) {
    let (mut n, mut p) = (a1 as i32, a2);
    if n <= 0 {
        return;
    }
    while n != 0 {
        m.w32(p, u32::MAX);
        n -= 1;
        p += 0x10;
    }
}

/// FUN_0019c8b0: *a1 |= 1 when +0x28
pub fn ee_sub_19c8b0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if m.r32(a0 + 0x28) != 0 {
        let v = m.r32(a1) | 1;
        m.w32(a1, v);
    }
}

/// FUN_0019e9c8: a1 +0x98 = a1 +0x94 = 0
pub fn ee_sub_19e9c8<M: QMem + ?Sized>(m: &mut M, _a0: u32, a1: u32) {
    m.w32(a1 + 0x98, 0);
    m.w32(a1 + 0x94, 0);
}

/// FUN_0019e9d8: FUN_001e6f10(+0xc, a1, 0, f = +0x10)
pub fn ee_sub_19e9d8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let f = m.rf(a0 + 0x10);
    let p = m.r32(a0 + 0xc);
    m.call(0x001e_6f10, &[p, a1, 0], &[fv(f)]).0
}

/// FUN_0019f3e8: *a0 = a1
pub fn ee_sub_19f3e8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.w32(a0, a1);
}

/// FUN_001a1f68: FUN_001a0b40(a0, *(+0x10) > 0 ? 0 : 8)
pub fn ee_sub_1a1f68<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let c = { let t = m.r32(a0 + 0x10); m.r32(t) } as i32;
    m.call(0x001a_0b40, &[a0, if c > 0 { 0 } else { 8 }], &[]).0
}

fn q1a1fa8(m: &mut (impl QMem + ?Sized), a0: u32) -> u32 {
    let c = { let t = m.r32(a0 + 0x10); m.r32(t) } as i32;
    if c <= 0 {
        return m.call(0x001a_0b40, &[a0, 8], &[]).0;
    }
    let v = m.r32(a0 + 0x30);
    if v != 0 && m.r64(v + 0x148) & (1u64 << 33) != 0 {
        return m.call(0x001a_16a8, &[a0], &[]).0;
    }
    let p = m.r32(a0);
    m.call(0x0019_5a00, &[p + 0x2c0, 0xb, 0, 0], &[]).0
}

/// FUN_001a1fa8: count 0 -> FUN_001a0b40(a0, 8); +0x30 flag bit 33 -> FUN_001a16a8; else
/// FUN_00195a00(*a0 + 0x2c0, 0xb, 0, 0)
pub fn ee_sub_1a1fa8<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    q1a1fa8(m, a0)
}

/// FUN_001a2080: same body as FUN_001a1fa8
pub fn ee_sub_1a2080<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    q1a1fa8(m, a0)
}

/// FUN_001a2028: sound 0x6a (FUN_001fbb90 -> FUN_001fb2f8(s, 0)), FUN_001a0b40(a0, 6),
/// FUN_001a15b0(a0, DAT_002a6790)
pub fn ee_sub_1a2028<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let s = m.call(0x001f_bb90, &[0x6a], &[]).0;
    m.call(0x001f_b2f8, &[s, 0], &[]);
    m.call(0x001a_0b40, &[a0, 6], &[]);
    let k = m.r32(0x002a_6790);
    m.call(0x001a_15b0, &[a0, k], &[]).0
}

/// FUN_001ab7d0: first nonzero of FUN_001ac748(a0, 0x2a6bb0), FUN_001ac900(a0, 0x2a6bc8, 0x2a6bf8, 5),
/// FUN_001ac008(a0, 0x2a6bb0, 0x2a6bf8), FUN_001ac238(a0, 0x2a6bb0)
pub fn ee_sub_1ab7d0<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let (s1, s2) = (0x002a_6bb0, 0x002a_6bf8);
    let r = m.call(0x001a_c748, &[a0, s1], &[]).0;
    if r != 0 {
        return r;
    }
    let r = m.call(0x001a_c900, &[a0, 0x002a_6bc8, s2, 5], &[]).0;
    if r != 0 {
        return r;
    }
    let r = m.call(0x001a_c008, &[a0, s1, s2], &[]).0;
    if r != 0 {
        return r;
    }
    m.call(0x001a_c238, &[a0, s1], &[]).0
}

/// FUN_001b1b98(a0, oid): o = FUN_0015e938(0x3dd490, 1, oid, a0); slot 8 on it; returns o
pub fn ee_sub_1b1b98<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let o = m.call(0x0015_e938, &[0x003d_d490, 1, a1, a0], &[]).0;
    if o != 0 {
        vslot(m, o, 8, &[], &[]);
    }
    o
}

/// FUN_001bbe48: !FUN_001bbe18(a0)
pub fn ee_sub_1bbe48<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    (m.call(0x001b_be18, &[a0], &[]).0 == 0) as u32
}

/// FUN_001bbe68: FUN_001bbe18(a0) ? +0x164 + (+0x15c << 4) : 0
pub fn ee_sub_1bbe68<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    if m.call(0x001b_be18, &[a0], &[]).0 == 0 {
        return 0;
    }
    let i = m.r32(a0 + 0x15c);
    m.r32(a0 + 0x164).wrapping_add(i << 4)
}

/// FUN_001c0f30(o, n, stream): +0 = n, +8 / +0x10 = counts (FUN_00117c80); ActPool arrays +4 (n * 8),
/// +0xc (+8 * 16), +0x18 zeroed / +0x14 (+0x10 * 4); n records {+0xc + 16 * u32, u16, u16}
/// (FUN_00117c20), +8 vectors (FUN_00117f70), +0x10 words (FUN_00117db8)
pub fn ee_sub_1c0f30<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    let sp = stack_push(m, 0x60);
    m.w32(a0, a1);
    let v = m.call(0x0011_7c80, &[a2], &[]).0;
    m.w32(a0 + 8, v);
    let v = m.call(0x0011_7c80, &[a2], &[]).0;
    m.w32(a0 + 0x10, v);
    for (f, cnt, sh, dst) in [(0x001a_e1d0u32, 0u32, 3u32, 4u32), (0x001a_e1d0, 8, 4, 0xc), (0x001a_e2a0, 0x10, 2, 0x18), (0x001a_e1d0, 0x10, 2, 0x14)] {
        let n = m.r32(a0 + cnt);
        m.w32(sp, n << sh);
        m.w32(sp + 4, 0x10);
        let p = m.call(f, &[sp], &[]).0;
        m.w32(a0 + dst, p);
    }
    let mut i = 0i32;
    while i < m.r32(a0) as i32 {
        let s0 = m.r32(a0 + 4).wrapping_add((i as u32) << 3);
        i += 1;
        let v = m.call(0x0011_7c20, &[a2], &[]).0;
        let b = m.r32(a0 + 0xc).wrapping_add(v << 4);
        m.w32(s0, b);
        let v = m.call(0x0011_7c20, &[a2], &[]).0;
        m.w16(s0 + 4, v as u16);
        let v = m.call(0x0011_7c20, &[a2], &[]).0;
        m.w16(s0 + 6, v as u16);
    }
    let mut j = 0i32;
    while j < m.r32(a0 + 8) as i32 {
        let p = m.r32(a0 + 0xc).wrapping_add((j as u32) << 4);
        j += 1;
        m.call(0x0011_7f70, &[a2, p], &[]);
    }
    let mut k = 0i32;
    while k < m.r32(a0 + 0x10) as i32 {
        let v = m.call(0x0011_7db8, &[a2], &[]).0;
        let p = m.r32(a0 + 0x14).wrapping_add((k as u32) << 2);
        k += 1;
        m.w32(p, v);
    }
    stack_pop(m, 0x60);
}

/// FUN_001c4700: FUN_001c46c0(a0, a1, 0.0) != 0
pub fn ee_sub_1c4700<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    (m.call(0x001c_46c0, &[a0, a1], &[Z]).0 != 0) as u32
}

/// FUN_001c7a08: +0x244 = 0
pub fn ee_sub_1c7a08<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.w32(a0 + 0x244, 0);
}

/// FUN_001cc9a8(a0, a1, n): max(n, 3) when +0xa4 == 1 and FUN_001a5bf0(+0x84) +0xec (nonzero)
/// equals a1 +0xec; else n
pub fn ee_sub_1cc9a8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) -> u32 {
    if m.r32(a0 + 0xa4) == 1 {
        let p = m.r32(a0 + 0x84);
        let v = m.call(0x001a_5bf0, &[p], &[]).0;
        if v != 0 {
            let x = m.r32(v + 0xec);
            if x != 0 && m.r32(a1 + 0xec) == x {
                return m.call(0x0013_4310, &[a2, 3], &[]).0;
            }
        }
    }
    a2
}

/// FUN_001de2a0: FUN_001f2108(a0, ..); +0x148 |= 1 << 34
pub fn ee_sub_1de2a0<M: QMem + ?Sized>(m: &mut M, a0: u32, rest: &[u32]) {
    let mut a = vec![a0];
    a.extend_from_slice(rest);
    m.call(0x001f_2108, &a, &[]);
    let v = m.r32(a0 + 0x14c) | 4;
    m.w32(a0 + 0x14c, v);
}

/// FUN_001e0120: +0x84 = 0
pub fn ee_sub_1e0120<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.w32(a0 + 0x84, 0);
}

/// FUN_001e02c0: FUN_001e01f8(a0, 1, 1)
pub fn ee_sub_1e02c0<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    m.call(0x001e_01f8, &[a0, 1, 1], &[]).0
}

/// FUN_001e1d48: when +0x40 and FUN_001e3890(a0, +0x40) and (+0x40 kind != 0x1d or its +0x10 == -1):
/// FUN_001e4eb8(a0, 0, 0)
pub fn ee_sub_1e1d48<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let a1 = m.r32(a0 + 0x40);
    if a1 == 0 || m.call(0x001e_3890, &[a0, a1], &[]).0 == 0 {
        return;
    }
    let p = m.r32(a0 + 0x40);
    let ok = m.r32(p) != 0x1d || m.r32(p + 0x10) == u32::MAX;
    if ok {
        m.call(0x001e_4eb8, &[a0, 0, 0], &[]);
    }
}

fn len2(d: V4) -> f32 {
    d[0] * d[0] + d[1] * d[1] + 1.0 * (d[2] * d[2])
}

/// FUN_001e69a0(ctx, t): point = FUN_001e67b8(*ctx, &pt, 0, t); squared distance to ctx +0x10,
/// through FUN_001fa970(ctx +4, d) when ctx +4 is set
pub fn ee_sub_1e69a0<M: QMem + ?Sized>(m: &mut M, a0: u32, f12: f32) -> f32 {
    let sp = stack_push(m, 0x30);
    let c = m.r32(a0);
    m.call(0x001e_67b8, &[c, sp, 0], &[fv(f12)]);
    let p = m.r32(a0 + 4);
    let q = m.rv(a0 + 0x10);
    let s = m.rv(sp);
    let r = if p == 0 {
        len2([q[0] - s[0], q[1] - s[1], q[2] - s[2], q[3] - s[3]])
    } else {
        let d = [s[0] - q[0], s[1] - q[1], s[2] - q[2], s[3] - q[3]];
        let v = m.call(0x001f_a970, &[p], &[d]).1;
        len2(v)
    };
    stack_pop(m, 0x30);
    r
}

/// FUN_001e6db8(path, target, a2, out_pt, out_dir, out_t, out_f, t): minimise
/// FUN_0011d9e0(FUN_001e69a0, {path, a2, target}) over [lo, hi] (knots +0xec, or DAT_002a8130 / 34
/// when +0xe4), step (k1 - k0) * 0.1; t = FUN_001e6660(t); point FUN_001e67b8 to out_pt / out_dir;
/// *out_t = t; *out_f = FUN_001e63c0(t)
#[allow(clippy::too_many_arguments)]
pub fn ee_sub_1e6db8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: V4, a2: u32, a3: u32, t0: u32, t1: u32, t2: u32, f12: f32) {
    let sp = stack_push(m, 0x90);
    m.w32(sp, a0);
    m.w32(sp + 4, a2);
    m.wv(sp + 0x10, a1);
    let ec = m.r32(a0 + 0xec);
    let k0 = m.rf(ec);
    let k1 = m.rf(ec + 4);
    let f13 = (k1 - k0) * f32::from_bits(0x3dcc_cccd);
    let (f14, f15) = if m.r32(a0 + 0xe4) != 0 {
        (m.rf(0x002a_8130), m.rf(0x002a_8134))
    } else {
        let n = m.r32(a0 + 0xe8);
        (k0, m.rf(ec.wrapping_add(n << 2).wrapping_sub(4)))
    };
    m.call(0x0011_d9e0, &[0x001e_69a0, sp, sp + 0x20, 0], &[fv(f12), fv(f13), fv(f14), fv(f15)]);
    let t = m.rf(sp + 0x20);
    let f0 = m.call(0x001e_6660, &[a0], &[fv(t)]).1[0];
    m.wf(sp + 0x20, f0);
    if a3 != 0 || t0 != 0 {
        m.call(0x001e_67b8, &[a0, a3, t0], &[fv(f0)]);
    }
    if t1 != 0 {
        let v = m.rf(sp + 0x20);
        m.wf(t1, v);
    }
    if t2 != 0 {
        let t = m.rf(sp + 0x20);
        let r = m.call(0x001e_63c0, &[a0], &[fv(t)]).1[0];
        m.wf(t2, r);
    }
    stack_pop(m, 0x90);
}

/// FUN_001eec60: *a0 = a1
pub fn ee_sub_1eec60<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.w32(a0, a1);
}

/// FUN_001f0198(a0..a3, t0): FUN_001f01b8(a0, a1, a2, a3, 4, t0)
pub fn ee_sub_1f0198<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, t0: u32) -> u32 {
    m.call(0x001f_01b8, &[a0, a1, a2, a3, 4, t0], &[]).0
}

/// FUN_001f0fd8(o, a1, a2): slot 0(a1, a2); +0x28 slot 4(a1, a2); FUN_00133508(o, a1, a2); slot 1
pub fn ee_sub_1f0fd8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    vslot(m, a0, 0, &[a1, a2], &[]);
    let h = m.r32(a0 + 0x28);
    if h != 0 {
        vslot(m, h, 4, &[a1, a2], &[]);
    }
    m.call(0x0013_3508, &[a0, a1, a2], &[]);
    vslot(m, a0, 1, &[a0], &[]);
}

/// FUN_001f68a8(o, on): bit 48 of +0x10 of each +0xe4 record (0xb0 each, +0xe0 count) = on & 1
pub fn ee_sub_1f68a8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let mut i = 0i32;
    while i < m.r32(a0 + 0xe0) as i32 {
        let p = m.r32(a0 + 0xe4).wrapping_add(i as u32 * 0xb0);
        let v = (m.r32(p + 0x14) & !0x1_0000) | ((a1 & 1) << 16);
        m.w32(p + 0x14, v);
        i += 1;
    }
}

/// FUN_001f8e90(world, body, &n, list): flood the contact graph: flag body +0x148 bit 27, append it to
/// list; recurse into each +0xec +0x64 contact's other body (+0xc) not yet flagged whose +0xe8 island
/// is not kind 3 / 3 (+0xc0 / +0xa0)
pub fn ee_sub_1f8e90<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32) {
    let v = m.r32(a1 + 0x148) | 0x0800_0000;
    m.w32(a1 + 0x148, v);
    let n = m.r32(a2);
    m.w32(a3.wrapping_add(n << 2), a1);
    let n = m.r32(a2).wrapping_add(1);
    m.w32(a2, n);
    let mut s0 = { let t = m.r32(a1 + 0xec); m.r32(t + 0x64) };
    while s0 != 0 {
        if m.r32(s0) != 0 {
            let b = m.r32(s0 + 0xc);
            if m.r32(b + 0x148) & 0x0800_0000 == 0 {
                let e = m.r32(b + 0xe8);
                if e != 0 && !(m.r32(e + 0xc0) == 3 && m.r32(e + 0xa0) == 3) {
                    ee_sub_1f8e90(m, a0, b, a2, a3);
                }
            }
        }
        s0 = m.r32(s0 + 4);
    }
}

fn dot3(a: V4, b: V4) -> f32 {
    let p = [a[0] * b[0], a[1] * b[1], a[2] * b[2]];
    p[0] + p[1] + 1.0 * p[2]
}

/// FUN_001f9240(world, flags, a2, a3): n = FUN_001f90d0(.., 0x40, bodies); flag 1 kept only with
/// bodies; flag 4 kept only when a body moves (kind 0xb: |+0x90| > 0.1, else +0x90 < -2.0); flag 2
/// only when a body's +0xb0 . +0x80 is beyond 5.0 (kind 0xb: abs; else < -5.0); then
/// FUN_001f9410(world, flags, a2, a3, n, bodies) when any flag is left
pub fn ee_sub_1f9240<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32) {
    let sp = stack_push(m, 0x150);
    let n = m.call(0x001f_90d0, &[a0, a1, a2, a3, 0x40, sp], &[]).0 as i32;
    let mut s0 = a1;
    if s0 & 1 != 0 {
        if n == 0 {
            s0 = 0;
        }
    } else {
        if s0 & 4 != 0 {
            let mut found = false;
            for i in 0..n.max(0) as u32 {
                let p = m.r32(sp + 4 * i);
                let f = m.rf(p + 0x90);
                found = if m.r32(p + 0x9c) == 0xb { f32::from_bits(0x3dcc_cccd) < f.abs() } else { f < -2.0 };
                if found {
                    break;
                }
            }
            if !found {
                s0 &= !4;
            }
        }
        if s0 & 2 != 0 {
            let mut found = false;
            for i in 0..n.max(0) as u32 {
                let p = m.r32(sp + 4 * i);
                let d = dot3(m.rv(p + 0xb0), m.rv(p + 0x80));
                found = if m.r32(p + 0x9c) == 0xb { 5.0 < d.abs() } else { d < -5.0 };
                if found {
                    break;
                }
            }
            if !found {
                s0 &= !2;
            }
        }
    }
    if s0 != 0 {
        m.call(0x001f_9410, &[a0, s0, a2, a3, n as u32, sp], &[]);
    }
    stack_pop(m, 0x150);
}

/// FUN_001feea0: +0x5c -= 1
pub fn ee_sub_1feea0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let v = m.r32(a0 + 0x5c).wrapping_sub(1);
    m.w32(a0 + 0x5c, v);
}

/// FUN_001feeb0: +0x60 += 1
pub fn ee_sub_1feeb0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let v = m.r32(a0 + 0x60).wrapping_add(1);
    m.w32(a0 + 0x60, v);
}

/// FUN_001feec0: +0x60 -= 1
pub fn ee_sub_1feec0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let v = m.r32(a0 + 0x60).wrapping_sub(1);
    m.w32(a0 + 0x60, v);
}

/// FUN_00201360(out, n, a2, a3, ...): varargs: the n words after a3 (t0..) copied to a frame array,
/// FUN_00201110(out, n, a2, a3, array); returns out
pub fn ee_sub_201360<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, rest: &[u32]) -> u32 {
    let sp = stack_push(m, 0x100);
    let n = a1 as i32;
    for i in 0..n.max(0) as usize {
        let w = rest.get(i).copied().unwrap_or(0);
        m.w32(sp + 4 * i as u32, w);
    }
    m.call(0x0020_1110, &[a0, a1, a2, a3, sp], &[]);
    stack_pop(m, 0x100);
    a0
}

/// FUN_00206d48: is-a any of the 5 class ids at DAT_002c3d10 (FUN_00132b10)
pub fn ee_sub_206d48<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    for i in 0..5 {
        let k = m.r32(0x002c_3d10 + 4 * i);
        if m.call(0x0013_2b10, &[a0, k], &[]).0 != 0 {
            return 1;
        }
    }
    0
}

/// FUN_0020bad8: DAT_002a2dd0 + sum over the two 0x40 records at a0 +0x10 of
/// +0x10 * (+0 * (+0x24 / (+4 +0xd0 +8)->+4))
pub fn ee_sub_20bad8<M: QMem + ?Sized>(m: &mut M, a0: u32) -> V4 {
    let a1 = m.r32(a0 + 4);
    let v1 = m.r32(a1 + 0xd0);
    let mut acc = m.rv(0x002a_2dd0);
    let f2 = { let t = m.r32(v1 + 8); m.rf(t + 4) };
    let mut p = a0 + 0x10;
    for _ in 0..2 {
        let f1 = m.rf(p + 0x24) / f2;
        let f0 = m.rf(p) * f1;
        let d = m.rv(p + 0x10);
        for k in 0..4 {
            acc[k] += d[k] * f0;
        }
        p += 0x40;
    }
    acc
}

/// FUN_0020bd68: +0x9c = +0xa0 = 0
pub fn ee_sub_20bd68<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.w32(a0 + 0x9c, 0);
    m.w32(a0 + 0xa0, 0);
}

/// FUN_0020bd78: *a0 = -1
pub fn ee_sub_20bd78<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.w32(a0, u32::MAX);
}

/// FUN_00220498(o, who, n): max(n, 3) when who == +0x10 and +4 (state) >= 3, or < 3 (not -1) with
/// who the player (DAT_002dd5bc) in state 0x12 (+0x2dc); else n
pub fn ee_sub_220498<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) -> u32 {
    if a1 != m.r32(a0 + 0x10) {
        return a2;
    }
    let x = m.r32(a0 + 4) as i32;
    if x == -1 {
        return a2;
    }
    if x < 3 && (a1 != m.r32(0x002d_d5bc) || m.r32(a1 + 0x2dc) != 0x12) {
        return a2;
    }
    m.call(0x0013_4310, &[a2, 3], &[]).0
}

/// FUN_0022c4d8: FUN_0022c4a0(a0, a1, 0)
pub fn ee_sub_22c4d8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    m.call(0x0022_c4a0, &[a0, a1, 0], &[]).0
}

/// FUN_0022c4f8: FUN_0022c4a0(a0, a1, 1)
pub fn ee_sub_22c4f8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    m.call(0x0022_c4a0, &[a0, a1, 1], &[]).0
}

/// FUN_002309e8(a0, a1, mat, mat2, f): mat = identity, mat2 = zero (when given; FUN_002333b4);
/// FUN_00230a98(a0, a1, mat, mat2, f)
pub fn ee_sub_2309e8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, f12: f32) {
    if a2 != 0 {
        m.call(0x0023_33b4, &[a2, 0, 0x40], &[]);
        for o in [0, 0x3c, 0x28, 0x14] {
            m.wf(a2 + o, 1.0);
        }
    }
    if a3 != 0 {
        m.call(0x0023_33b4, &[a3, 0, 0x40], &[]);
    }
    m.call(0x0023_0a98, &[a0, a1, a2, a3], &[fv(f12)]);
}

/// FUN_0024a7b0: language tag DAT_002c8378 (14 bytes) read once from file DAT_002ac270
/// (FUN_00237ea8 open / 002382b8 read / 00238138 close); returns the tag
pub fn ee_sub_24a7b0<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let s2 = 0x002c_8378;
    if m.r8(s2) == 0 {
        let fd = m.call(0x0023_7ea8, &[0x002a_c270, 1], &[]).0;
        if (fd as i32) >= 0 {
            m.call(0x0023_82b8, &[fd, s2, 0xe], &[]);
            m.call(0x0023_8138, &[fd], &[]);
        }
    }
    s2
}

/// FUN_0024a820: tag (FUN_0024a7b0) byte 4 == 'T'
pub fn ee_sub_24a820<M: QMem + ?Sized>(m: &mut M) -> u32 {
    if m.r8(0x002c_8378) == 0 {
        m.call(0x0024_a7b0, &[], &[]);
    }
    (m.r8(0x002c_837c) == 0x54) as u32
}

/// FUN_00259d98: x of the quadword at a0
pub fn ee_sub_259d98<M: QMem + ?Sized>(m: &mut M, a0: u32) -> f32 {
    m.rf(a0)
}

/// FUN_00259da8: y of the quadword at a0 (PROT3W)
pub fn ee_sub_259da8<M: QMem + ?Sized>(m: &mut M, a0: u32) -> f32 {
    m.rf(a0 + 4)
}

/// FUN_00259dc0: z of the quadword at a0 (PEXTUW)
pub fn ee_sub_259dc0<M: QMem + ?Sized>(m: &mut M, a0: u32) -> f32 {
    m.rf(a0 + 8)
}

/// FUN_00119ba0: static init thunk FUN_00119a68(1, 0xffff)
pub fn ee_sub_119ba0<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x00119a68, &[1, 0xffff], &[]);
}

/// FUN_00119d80: static init thunk FUN_00119c48(1, 0xffff)
pub fn ee_sub_119d80<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x00119c48, &[1, 0xffff], &[]);
}

/// FUN_00119ed8: static init thunk FUN_00119da0(1, 0xffff)
pub fn ee_sub_119ed8<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x00119da0, &[1, 0xffff], &[]);
}

/// FUN_0011a030: static init thunk FUN_00119ef8(1, 0xffff)
pub fn ee_sub_11a030<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x00119ef8, &[1, 0xffff], &[]);
}

/// FUN_0011a530: static init thunk FUN_0011a3f8(1, 0xffff)
pub fn ee_sub_11a530<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x0011a3f8, &[1, 0xffff], &[]);
}

/// FUN_0014b640: static init thunk FUN_0014b508(1, 0xffff)
pub fn ee_sub_14b640<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x0014b508, &[1, 0xffff], &[]);
}

/// FUN_0014b7d8: static init thunk FUN_0014b6a0(1, 0xffff)
pub fn ee_sub_14b7d8<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x0014b6a0, &[1, 0xffff], &[]);
}

/// FUN_0014be88: static init thunk FUN_0014bd50(1, 0xffff)
pub fn ee_sub_14be88<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x0014bd50, &[1, 0xffff], &[]);
}

/// FUN_0014c528: static init thunk FUN_0014c3f0(1, 0xffff)
pub fn ee_sub_14c528<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x0014c3f0, &[1, 0xffff], &[]);
}

/// FUN_0014c680: static init thunk FUN_0014c548(1, 0xffff)
pub fn ee_sub_14c680<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x0014c548, &[1, 0xffff], &[]);
}

/// FUN_001571e0: static init thunk FUN_001570a8(1, 0xffff)
pub fn ee_sub_1571e0<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x001570a8, &[1, 0xffff], &[]);
}

/// FUN_001578d8: static init thunk FUN_001577a0(1, 0xffff)
pub fn ee_sub_1578d8<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x001577a0, &[1, 0xffff], &[]);
}

/// FUN_00157a30: static init thunk FUN_001578f8(1, 0xffff)
pub fn ee_sub_157a30<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x001578f8, &[1, 0xffff], &[]);
}

/// FUN_00160398: static init thunk FUN_00160260(1, 0xffff)
pub fn ee_sub_160398<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x00160260, &[1, 0xffff], &[]);
}

/// FUN_00160780: static init thunk FUN_00160648(1, 0xffff)
pub fn ee_sub_160780<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x00160648, &[1, 0xffff], &[]);
}

/// FUN_00166250: static init thunk FUN_00166118(1, 0xffff)
pub fn ee_sub_166250<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x00166118, &[1, 0xffff], &[]);
}

/// FUN_00166900: static init thunk FUN_001667c8(1, 0xffff)
pub fn ee_sub_166900<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x001667c8, &[1, 0xffff], &[]);
}

/// FUN_001672f0: static init thunk FUN_001671b8(1, 0xffff)
pub fn ee_sub_1672f0<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x001671b8, &[1, 0xffff], &[]);
}

/// FUN_0019f088: static init thunk FUN_0019ef50(1, 0xffff)
pub fn ee_sub_19f088<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x0019ef50, &[1, 0xffff], &[]);
}

/// FUN_0019f3c8: static init thunk FUN_0019f290(1, 0xffff)
pub fn ee_sub_19f3c8<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x0019f290, &[1, 0xffff], &[]);
}

/// FUN_001a02e0: static init thunk FUN_001a01a8(1, 0xffff)
pub fn ee_sub_1a02e0<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x001a01a8, &[1, 0xffff], &[]);
}

/// FUN_001a4de8: static init thunk FUN_001a4cb0(1, 0xffff)
pub fn ee_sub_1a4de8<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x001a4cb0, &[1, 0xffff], &[]);
}

/// FUN_001a5250: static init thunk FUN_001a5118(1, 0xffff)
pub fn ee_sub_1a5250<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x001a5118, &[1, 0xffff], &[]);
}

/// FUN_001ae7b0: static init thunk FUN_001ae678(1, 0xffff)
pub fn ee_sub_1ae7b0<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x001ae678, &[1, 0xffff], &[]);
}

/// FUN_001ae908: static init thunk FUN_001ae7d0(1, 0xffff)
pub fn ee_sub_1ae908<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x001ae7d0, &[1, 0xffff], &[]);
}

/// FUN_001bf0f0: static init thunk FUN_001befb8(1, 0xffff)
pub fn ee_sub_1bf0f0<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x001befb8, &[1, 0xffff], &[]);
}

/// FUN_001c2a58: static init thunk FUN_001c2920(1, 0xffff)
pub fn ee_sub_1c2a58<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x001c2920, &[1, 0xffff], &[]);
}

/// FUN_001c2d00: static init thunk FUN_001c2bc8(1, 0xffff)
pub fn ee_sub_1c2d00<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x001c2bc8, &[1, 0xffff], &[]);
}

/// FUN_001c3370: static init thunk FUN_001c3238(1, 0xffff)
pub fn ee_sub_1c3370<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x001c3238, &[1, 0xffff], &[]);
}

/// FUN_001c8ca0: static init thunk FUN_001c8b68(1, 0xffff)
pub fn ee_sub_1c8ca0<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x001c8b68, &[1, 0xffff], &[]);
}

/// FUN_001cbe90: static init thunk FUN_001cbd58(1, 0xffff)
pub fn ee_sub_1cbe90<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x001cbd58, &[1, 0xffff], &[]);
}

/// FUN_001cc138: static init thunk FUN_001cc000(1, 0xffff)
pub fn ee_sub_1cc138<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x001cc000, &[1, 0xffff], &[]);
}

/// FUN_001d5788: static init thunk FUN_001d5650(1, 0xffff)
pub fn ee_sub_1d5788<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x001d5650, &[1, 0xffff], &[]);
}

/// FUN_001d6828: static init thunk FUN_001d66f0(1, 0xffff)
pub fn ee_sub_1d6828<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x001d66f0, &[1, 0xffff], &[]);
}

/// FUN_001ee958: static init thunk FUN_001ee820(1, 0xffff)
pub fn ee_sub_1ee958<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x001ee820, &[1, 0xffff], &[]);
}

/// FUN_001eec40: static init thunk FUN_001eeb08(1, 0xffff)
pub fn ee_sub_1eec40<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x001eeb08, &[1, 0xffff], &[]);
}

/// FUN_001ef830: static init thunk FUN_001ef6f8(1, 0xffff)
pub fn ee_sub_1ef830<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x001ef6f8, &[1, 0xffff], &[]);
}

/// FUN_001efa70: static init thunk FUN_001ef938(1, 0xffff)
pub fn ee_sub_1efa70<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x001ef938, &[1, 0xffff], &[]);
}

/// FUN_001f20e8: static init thunk FUN_001f1fb0(1, 0xffff)
pub fn ee_sub_1f20e8<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x001f1fb0, &[1, 0xffff], &[]);
}

/// FUN_0021d258: static init thunk FUN_0021d120(1, 0xffff)
pub fn ee_sub_21d258<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x0021d120, &[1, 0xffff], &[]);
}

/// FUN_0021d618: static init thunk FUN_0021d4e0(1, 0xffff)
pub fn ee_sub_21d618<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x0021d4e0, &[1, 0xffff], &[]);
}

/// FUN_0021efb0: static init thunk FUN_0021ee78(1, 0xffff)
pub fn ee_sub_21efb0<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x0021ee78, &[1, 0xffff], &[]);
}

/// FUN_0021f2c0: static init thunk FUN_0021f188(1, 0xffff)
pub fn ee_sub_21f2c0<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x0021f188, &[1, 0xffff], &[]);
}

/// FUN_00226bf0: static init thunk FUN_00226ab8(1, 0xffff)
pub fn ee_sub_226bf0<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x00226ab8, &[1, 0xffff], &[]);
}

/// FUN_00227330: static init thunk FUN_002271f8(1, 0xffff)
pub fn ee_sub_227330<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x002271f8, &[1, 0xffff], &[]);
}

/// FUN_00227598: static init thunk FUN_00227460(1, 0xffff)
pub fn ee_sub_227598<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x00227460, &[1, 0xffff], &[]);
}

/// FUN_00227b98: static init thunk FUN_00227a60(1, 0xffff)
pub fn ee_sub_227b98<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x00227a60, &[1, 0xffff], &[]);
}

/// FUN_00227fc8: static init thunk FUN_00227e90(1, 0xffff)
pub fn ee_sub_227fc8<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x00227e90, &[1, 0xffff], &[]);
}

/// FUN_002310e0: static init thunk FUN_00230fa8(1, 0xffff)
pub fn ee_sub_2310e0<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x00230fa8, &[1, 0xffff], &[]);
}

// ---- frontier natives (callees of live-reached queue ports) ----

/// FUN_001433c8: with +0x11a8: FUN_00143678(a0, 1, 0); FUN_0013c560(+0x11a8, 0)
pub fn ee_sub_1433c8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    if m.r32(a0 + 0x11a8) == 0 {
        return;
    }
    m.call(0x0014_3678, &[a0, 1, 0], &[]);
    let h = m.r32(a0 + 0x11a8);
    m.call(0x0013_c560, &[h, 0], &[]);
}

/// FUN_0020efc0(world, mask): +0x3d78 += 1 (first: FUN_00187078(2)); mask 1 FUN_0020f128, mask 2
/// FUN_0020f198
pub fn ee_sub_20efc0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let n = m.r32(a0 + 0x3d78).wrapping_add(1);
    m.w32(a0 + 0x3d78, n);
    if n == 1 {
        m.call(0x0018_7078, &[2], &[]);
    }
    if a1 & 1 != 0 {
        m.call(0x0020_f128, &[a0], &[]);
    }
    if a1 & 2 != 0 {
        m.call(0x0020_f198, &[a0], &[]);
    }
}

/// FUN_0020f030(world, mask): +0x3d78 -= 1 (last: FUN_001870a0(2), FUN_00187e08(0x2dfc00, 8)); mask 1
/// FUN_0020f160, mask 2 FUN_0020f1a8
pub fn ee_sub_20f030<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let n = m.r32(a0 + 0x3d78).wrapping_sub(1);
    m.w32(a0 + 0x3d78, n);
    if n == 0 {
        m.call(0x0018_70a0, &[2], &[]);
        m.call(0x0018_7e08, &[0x002d_fc00, 8], &[]);
    }
    if a1 & 1 != 0 {
        m.call(0x0020_f160, &[a0], &[]);
    }
    if a1 & 2 != 0 {
        m.call(0x0020_f1a8, &[a0], &[]);
    }
}

/// FUN_001c02d0(o, a1, a2, path): n = FUN_001bff28(o + 0xe0, a1, a2, 0x10, path +0xf4); n >= 2:
/// path +0xe8 = n, FUN_001e6180(path), 1; else 0
pub fn ee_sub_1c02d0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32) -> u32 {
    let t0 = m.r32(a3 + 0xf4);
    let n = m.call(0x001b_ff28, &[a0 + 0xe0, a1, a2, 0x10, t0], &[]).0;
    if (n as i32) < 2 {
        return 0;
    }
    m.w32(a3 + 0xe8, n);
    m.call(0x001e_6180, &[a3], &[]);
    1
}

/// FUN_001fbb90(kind): FUN_001fbbc8(FUN_001fc0f0(0x70), kind)
pub fn ee_sub_1fbb90<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let p = m.call(0x001f_c0f0, &[0x70], &[]).0;
    m.call(0x001f_bbc8, &[p, a0], &[]).0
}

/// FUN_001c3ed8(o, state): no-op when FUN_001c4080(o) == state; first use registers o in the
/// 0x3dd490 +0x3ebc list (max 16) and builds +0x150 = FUN_001c4280(FUN_001ae4f8(0xe0), o); state 1
/// makes o the focus (+0x3efc: old focus slot 78(0), FUN_001c4940); state -1 / 0 drops it if focused;
/// then FUN_001c4640(+0x150, state)
pub fn ee_sub_1c3ed8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if m.call(0x001c_4080, &[a0], &[]).0 == a1 {
        return;
    }
    let base = 0x003d_d490;
    if m.r32(a0 + 0x150) == 0 {
        let c = m.r32(base + 0x3eb8);
        if c >= 0x10 {
            return;
        }
        m.w32(base + 0x3ebc + 4 * c, a0);
        m.w32(base + 0x3eb8, c + 1);
        let n = m.call(0x001a_e4f8, &[0xe0], &[]).0;
        let r = m.call(0x001c_4280, &[n, a0], &[]).0;
        m.w32(a0 + 0x150, r);
    }
    let s = a1 as i32;
    if s >= -1 {
        if s <= 0 {
            if a0 == m.r32(base + 0x3efc) {
                m.call(0x001c_4940, &[base, 0], &[]);
            }
        } else if s == 1 {
            let f = m.r32(base + 0x3efc);
            if a0 != f {
                if f != 0 {
                    vslot(m, f, 78, &[0], &[]);
                }
                m.call(0x001c_4940, &[base, a0], &[]);
            }
        }
    }
    let p = m.r32(a0 + 0x150);
    m.call(0x001c_4640, &[p, a1], &[]);
}

fn w64<M: QMem + ?Sized>(m: &mut M, a: u32, v: u64) {
    m.w32(a, v as u32);
    m.w32(a + 4, (v >> 32) as u32);
}

/// FUN_001be398(tex, src): 128x128 render-target texture: TEX0 at +8 (TBP0 = FUN_00170548(src, 0x100),
/// TBW 2, PSMCT32, TW / TH = log2 (FUN_001703c0), TCC); GS packet (FUN_00166920 / 00157330 /
/// 00166bd8) drawing src (FRAME 0x4c, ZBUF 0x4e, XYOFFSET, SCISSOR, TEST, PRIM 6 sprites in 32-pixel
/// strips via FUN_00166da0), closed into +0x10 / +0x14 (FUN_00166968 / 001573c0); second packet sets
/// TEX0 / CLAMP 0x60 / TEX1 5 into +0x18 / +0x1c. 64-bit register data goes straight to the u64 ports.
pub fn ee_sub_1be398<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    use crate::engine::game::q_00166008::{ee_sub_166bd8 as prim, ee_sub_166da0 as xy, ee_sub_166e90 as reg};
    let sp = stack_push(m, 0x4d0);
    m.w32(a0 + 4, 0x80);
    m.w32(a0, 0x80);
    let tb = m.call(0x0017_0548, &[a1, 0x100], &[]).0 as i32;
    let mut v = m.r64(a0 + 8);
    v = (v & !0x3fffu64) | (tb as u32 & 0x3fff) as u64;
    v &= 0xffff_ffff_fff0_3fff;
    v |= 0x8000;
    v &= 0xffff_ffff_fc0f_ffff;
    w64(m, a0 + 8, v);
    let s2 = (tb >> 5) as i64;
    let w0 = m.r32(a0);
    let r = (m.call(0x0017_03c0, &[w0], &[]).0 & 0xf) as u64;
    let v = (m.r64(a0 + 8) & 0xffff_ffff_c3ff_ffff) | (r << 26);
    w64(m, a0 + 8, v);
    let h0 = m.r32(a0 + 4);
    let r = (m.call(0x0017_03c0, &[h0], &[]).0 & 0xf) as u64;
    let v = (m.r64(a0 + 8) & 0xffff_fffc_3fff_ffff) | (r << 30) | (1u64 << 34);
    w64(m, a0 + 8, v);
    let s0 = ((m.r32(a1 + 8) as i32).wrapping_sub(0x100) >> 5) as i64;
    m.call(0x0016_6920, &[sp], &[]);
    let w = m.r32(a0) as i32;
    let h = m.r32(a0 + 4) as i32;
    let s7 = 0x800 - w / 2;
    let s6 = 0x800 - h / 2;
    let fp = sp + 0x30;
    let mut s4 = s7;
    m.call(0x0015_7330, &[sp, 0x40, fp], &[]);
    prim(m, sp, 0, 1, 0xe);
    let w = m.r32(a0) as i32;
    let fw = if 0x40 < w { w } else { 0x40 };
    let fw = (if -1 < fw { fw } else { fw + 0x3f }) >> 6;
    reg(m, sp, 0x4c, (((fw as i64) << 16) | s2) as u64);
    reg(m, sp, 0x4e, (s0 | 0x3000_0000) as u64);
    reg(m, sp, 0x18, ((((s6 << 4) as i64) << 32) | (s7 << 4) as i64) as u64);
    let w = m.r32(a0) as i32;
    let h = m.r32(a0 + 4) as i32;
    reg(m, sp, 0x40, ((((w - 1) as i64) << 16) | (((h - 1) as i64) << 48)) as u64);
    reg(m, sp, 0x47, 0x3_0000);
    reg(m, sp, 1, 0);
    prim(m, sp, 6, 1, 5);
    while s4 < s7.wrapping_add(m.r32(a0) as i32) {
        xy(m, sp, (s4 << 4) as u32, (s6 << 4) as u32, 0);
        s4 += 0x20;
        let h = m.r32(a0 + 4) as i32;
        xy(m, sp, (s4 << 4) as u32, ((s6 + h) << 4) as u32, 0);
    }
    m.call(0x0016_6968, &[sp], &[]);
    m.call(0x0015_73c0, &[sp, a0 + 0x10, 0, a0 + 0x14], &[]);
    m.call(0x0015_7330, &[sp, 0x40, fp], &[]);
    prim(m, sp, 0, 1, 0xe);
    let t = m.r64(a0 + 8);
    reg(m, sp, 6, t);
    reg(m, sp, 0x14, 0x60);
    reg(m, sp, 8, 5);
    m.call(0x0016_6968, &[sp], &[]);
    m.call(0x0015_73c0, &[sp, a0 + 0x18, 0, a0 + 0x1c], &[]);
    stack_pop(m, 0x4d0);
}
