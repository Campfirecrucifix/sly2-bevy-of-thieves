//! WP-Q13 third part: runs 001ddd40 .. 001f58f0 (menu text setters, path follower, collision record helpers).
use super::q13_b::{rr, vcl};
use super::q_00114680::*;
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem, V4};

// ---- run_001ddd40 .. run_001e81c0
/// FUN_001ddd40: 0..=8 -> 0x6d + n, else -1
pub fn ee_sub_1ddd40(n: u64) -> u32 {
    if n <= 8 {
        0x6d + n as u32
    } else {
        0xffff_ffff
    }
}
/// FUN_001dddb8
pub fn ee_sub_1dddb8<M: QMem + ?Sized>(m: &mut M, a: u32) {
    let u = m.call(0x0021_0ae0, &[], &[]).0;
    let u = ee_sub_1ddd40(u as u64);
    let u = m.call(0x0013_e368, &[0x003d_d490, u], &[]).0;
    m.call(0x0021_ed58, &[a, u], &[]);
}
fn set_text<M: QMem + ?Sized>(m: &mut M, a: u32, key: u32) {
    let u = m.call(0x0013_e368, &[0x003d_d490, key], &[]).0;
    m.call(0x0021_ed58, &[a, u], &[]);
}
/// FUN_001dde10: text id of DAT_003da4d0 +0x70
pub fn ee_sub_1dde10<M: QMem + ?Sized>(m: &mut M, a: u32) {
    let d = m.r32(0x003d_a4d0);
    if d != 0 {
        let k = m.r32(d + 0x70);
        set_text(m, a, k);
    }
}
/// FUN_001dde58: text id of DAT_003da4d0 +0x6c
pub fn ee_sub_1dde58<M: QMem + ?Sized>(m: &mut M, a: u32) {
    let d = m.r32(0x003d_a4d0);
    if d != 0 {
        let k = m.r32(d + 0x6c);
        set_text(m, a, k);
    }
}
/// FUN_001ddea0: entry i (12-byte table at 0x3da40c, count 0x3da408): +0x6c of its record
pub fn ee_sub_1ddea0<M: QMem + ?Sized>(m: &mut M, a: u32, i: i32) {
    if i < m.r32(0x003d_a408) as i32 {
        let r = m.r32(0x003d_a40c_u32.wrapping_add((i as u32).wrapping_mul(0xc)));
        let k = m.r32(r + 0x6c);
        set_text(m, a, k);
    }
}
/// FUN_001ddf00: as 1ddea0 with +0x5c
pub fn ee_sub_1ddf00<M: QMem + ?Sized>(m: &mut M, a: u32, i: i32) {
    if i < m.r32(0x003d_a408) as i32 {
        let r = m.r32(0x003d_a40c_u32.wrapping_add((i as u32).wrapping_mul(0xc)));
        let k = m.r32(r + 0x5c);
        set_text(m, a, k);
    }
}
/// FUN_001ddf60
pub fn ee_sub_1ddf60<M: QMem + ?Sized>(m: &mut M, a: u32) {
    set_text(m, a, 0x40);
    m.call(0x0021_eda0, &[a, 0x002a_7aa8], &[]);
}
/// FUN_001ddfb0
pub fn ee_sub_1ddfb0<M: QMem + ?Sized>(m: &mut M, a: u32) {
    let u = m.call(0x001d_82b8, &[0x003e_0ea8, 0x003d_4a60], &[]).0;
    m.call(0x0021_ecd8, &[a, 0x002a_7ab8, u], &[]);
}
/// FUN_001de008: static init / destroy of the menu singletons (0x3d9e70 ..), then the six pool stores
pub fn ee_sub_1de008<M: QMem + ?Sized>(m: &mut M, p1: u64, p2: u64) {
    if p2 != 0xffff {
        return;
    }
    if p1 == 0 {
        for a in [0x3da530u32, 0x3da4f0, 0x3da3c0, 0x3da150, 0x3da0f0, 0x3da0c0, 0x3d9ed0, 0x3d9e80] {
            m.call(0x0025_8348, &[a, 2], &[]);
        }
    } else {
        m.call(0x001d_93d8, &[0x003d_9e70], &[]);
        m.w32(0x003d_9e8c, 0x002d_8668);
        m.w32(0x003d_9edc, 0x002d_37d0);
        m.w32(0x003d_a0cc, 0x002c_a490);
        for a in [0x3d9e80u32, 0x3d9ed0, 0x3da0c0, 0x3d9e84, 0x3d9ed4, 0x3da0c4] {
            m.w32(a, 0);
        }
        m.call(0x001d_aa08, &[0x003d_a0f0], &[]);
        m.call(0x001d_b7a8, &[0x003d_a150], &[]);
        m.call(0x001d_cc58, &[0x003d_a3c0], &[]);
        m.call(0x001d_d648, &[0x003d_a4f0], &[]);
        m.call(0x001d_d7d8, &[0x003d_a530], &[]);
        static_init_stores(m, 1, 0xffff);
    }
}
/// FUN_001de260
pub fn ee_sub_1de260<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_1de008(m, 1, 0xffff)
}
/// FUN_001de280
pub fn ee_sub_1de280<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_1de008(m, 0, 0xffff)
}

/// FUN_001e8008: random value in [lo, hi] around `c` (p1) +- `h` (p2) shaped by DAT_002c3650 (0 linear,
/// 1 x*|x|, 2 smoothstep-like, 3 sqrt); with +0xe4 == 0 the range is clamped to [0, FUN_001e6648]
pub fn ee_sub_1e8008<M: QMem + ?Sized>(m: &mut M, mut p1: f32, p2: f32, o: u32) {
    let mut hi;
    if m.r32(o + 0xe4) == 0 {
        let lim = m.call(0x001e_6648, &[o], &[]).1[0];
        let mut f3 = p2;
        if p2 <= p1 {
            f3 = lim - p2;
            if p1 <= lim - p2 {
                f3 = p1;
            }
        }
        let f1 = f3 + p2;
        hi = if f1 < lim { f1 } else { lim };
        p1 = if 0.0 < f3 - p2 { f3 - p2 } else { 0.0 };
    } else {
        hi = p1 + p2;
        p1 -= p2;
    }
    let mut r = m.call(0x0011_d448, &[(-1.0f32).to_bits(), 1.0f32.to_bits()], &[]).1[0];
    let mode = m.r32(0x002c_3650) as i32;
    if mode == 1 {
        r *= r.abs();
    } else if mode > 1 {
        if mode == 2 {
            let c = if 0.0 < r { 3.0 } else { -3.0 };
            r = r * r * (c - (r + r));
        } else if mode == 3 {
            r = if 0.0 < r { r.sqrt() } else { -(-r).sqrt() };
        }
    }
    r = r * 0.5 + 0.5;
    if m.r32(o + 0xe4) != 0 {
        let v = (1.0 - r) * p1 + r * hi;
        m.call(0x001e_66c8, &[v.to_bits(), o], &[]);
    }
    let _ = &mut hi;
}
/// FUN_001e81c0: init the path follower: transform the source points (obj2 +0xf4, count +0xe8) by the matrix
/// of its vslot +0xf8 into a fresh array (+0x54), keep first (+0x20) / last (+0x30) point
pub fn ee_sub_1e81c0<M: QMem + ?Sized>(m: &mut M, t: u32, o2: u32) -> u32 {
    m.w32(t + 0x48, o2);
    m.w32(t + 0x4c, 0xffff_ffff);
    let n = m.r32(o2 + 0xe8) as i32;
    let sp = stack_push(m, 0x10);
    for (dst, shift) in [(0x54u32, 4u32), (0x58, 4), (0x5c, 2)] {
        m.w32(sp + 4, 0x10);
        m.w32(sp, (n as u32) << shift);
        let a = m.call(0x001a_e2a0, &[sp], &[]).0;
        m.w32(t + dst, a);
        if dst == 0x5c {
            m.w32(t + 8, 0xc4da_c000);
        }
    }
    stack_pop(m, 0x10);
    m.w32(t + 0x60, 0x4040_0000);
    m.w32(t + 0x6c, 0x43fa_0000);
    m.w32(t + 0x70, 0x40a0_0000);
    m.w32(t, 0);
    m.w32(t + 4, 0);
    let i9 = m.r32(t + 0x48);
    let c = m.r32(i9 + 0xe8);
    m.w32(t + 0x88, 0x4396_0000);
    m.w32(t + 0x84, c);
    let mat = vcl(m, i9, 0xf8, 0xfc, &[], &[]).0;
    let cols = [m.rv(mat), m.rv(mat + 0x10), m.rv(mat + 0x20), m.rv(mat + 0x30)];
    let mut i = 0i32;
    while i < rr(m, t + 0x48, 0xe8) as i32 {
        let src = rr(m, t + 0x48, 0xf4);
        let v = m.rv(src + (i as u32) * 0x10);
        let mut o = [0f32; 4];
        for k in 0..4 {
            o[k] = cols[0][k] * v[0] + cols[1][k] * v[1] + cols[2][k] * v[2] + cols[3][k];
        }
        let dst = m.r32(t + 0x54);
        m.wv(dst + (i as u32) * 0x10, o);
        i += 1;
    }
    m.w32(t + 0x40, 0x002c_44f0);
    m.w32(t + 0x44, 0x002c_44f0);
    if 0 < rr(m, t + 0x48, 0xe8) as i32 {
        let pts = m.r32(t + 0x54);
        let first = m.rv(pts);
        m.wv(t + 0x20, first);
        let cnt = rr(m, t + 0x48, 0xe8);
        let last = m.rv(pts + cnt * 0x10 - 0x10);
        m.wv(t + 0x30, last);
    }
    m.call(0x001e_9828, &[t], &[]);
    t
}

// ---- run_001f35c8 .. run_001f4b58
/// FUN_001f35c8: +0xd0 of the data block (+0xe8) rescaled by `p1 / old` (= p1 for a non-zero old value) then
/// 1f35f8. PARTIAL: the register arguments of the tail call (a0/a1 pass-through) are taken as (o, o).
pub fn ee_sub_1f35c8<M: QMem + ?Sized>(m: &mut M, p1: f32, o: u32) {
    let b = m.r32(o + 0xe8);
    let f = m.rf(b + 0xd0);
    // VER5: FUN_001f35f8(o, $f12 = p1 / f) (PS2 div.s)
    let q = crate::engine::math::ps2f::ps2_div(p1, f);
    m.wf(b + 0xd0, f * q);
    ee_sub_1f35f8(m, o, q);
}
/// FUN_001f35f8: scale the three 4-float rows of the block (+0xe8) by `s` (bits), then clear the 0x30 bytes
/// after it when FUN_0011b318(block, block + 0x30) is 0
/// VER5: ABI per asm: o = a0, s = $f12
pub fn ee_sub_1f35f8<M: QMem + ?Sized>(m: &mut M, o: u32, s: f32) {
    let b = m.r32(o + 0xe8);
    for r in 0..3 {
        let mut v = m.rv(b + 0x10 * r);
        for k in 0..4 {
            v[k] *= s;
        }
        m.wv(b + 0x10 * r, v);
    }
    if m.call(0x0011_b318, &[b, b + 0x30], &[]).0 == 0 {
        m.call(0x0023_33b4, &[b + 0x30, 0, 0x30], &[]);
    }
}
/// FUN_001f36a8
pub fn ee_sub_1f36a8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001a_5c48, &[o], &[]);
    m.call(0x0022_faa0, &[o], &[]);
    m.call(0x0016_7b68, &[o + 0x80], &[]);
}
/// FUN_001f37c8: move the record (+0xe8 -> +0xdc) between the owner lists at owner+0x118
pub fn ee_sub_1f37c8<M: QMem + ?Sized>(m: &mut M, o: u32, new_owner: u32) {
    let rec = rr(m, o + 0xe8, 0xdc);
    let old = m.r32(rec + 0x10);
    if old == new_owner {
        return;
    }
    if old != 0 && m.call(0x0011_a380, &[old + 0x118, rec], &[]).0 != 0 {
        m.call(0x0011_a2e8, &[old + 0x118, rec], &[]);
    }
    if new_owner != 0 {
        if m.call(0x0011_a380, &[new_owner + 0x118, rec], &[]).0 != 0 {
            m.w32(rec + 0x10, new_owner);
            return;
        }
        m.call(0x0011_a220, &[new_owner + 0x118, rec], &[]);
        m.call(0x001c_1440, &[new_owner, rec + 0x20], &[]);
    }
    m.w32(rec + 0x10, new_owner);
}
/// FUN_001f3868: PARTIAL: argument registers of 1c0d70 / 1f37c8 (a0/a1 pass-through) assumed (q, 1) / (o, a1=0)
pub fn ee_sub_1f3868<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let q = m.call(0x001a_62e8, &[o], &[]);
    m.call(0x001c_0d70, &[q.0, 1], &[q.1]);
    ee_sub_1f37c8(m, o, 0);
}
/// FUN_001f38a8
pub fn ee_sub_1f38a8<M: QMem + ?Sized>(m: &mut M, p: u32) {
    let s = m.r32(p);
    if m.r32(p + 0xa8) != m.r32(s + 0xa4) {
        m.call(0x0016_64a0, &[s + 0x40, p + 0x78, p + 0x90], &[]);
        let v = m.r32(s + 0xa4);
        m.w32(p + 0xa8, v);
        let a = m.r32(p + 4);
        m.call(0x001f_4bf8, &[a], &[]);
    }
}
/// FUN_001f3900: refresh the cached matrix (+0x20..0x5c) and stamp (+0xac) of every 0xb0-byte entry whose
/// flag bit 50 is clear from its source object (entry +0)
pub fn ee_sub_1f3900<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let n = m.r32(o + 0xe0) as i32;
    for i in 0..n.max(0) {
        let e = m.r32(o + 0xe4).wrapping_add(i as u32 * 0xb0);
        if m.r64(e + 0x10) & 0x4_0000_0000_0000 == 0 {
            let s = m.r32(e);
            for k in 0..16 {
                let w = m.r32(s + 0x40 + 4 * k);
                m.w32(e + 0x20 + 4 * k, w);
            }
            let st = m.r32(s + 0xa4);
            m.w32(e + 0xac, st);
        }
    }
}
/// FUN_001f4848 / 1f4870: quaternion-style combine of the two slots (+0x58 / +0x5c) with +0x60
pub fn ee_sub_1f4848<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let s = m.r32(o + 0x58);
    let (a, b, c) = (m.r32(s + 0xa0), m.r32(o + 0x60), m.r32(s + 0xb0));
    m.call(0x0022_fcc8, &[a, b, c], &[]);
}
pub fn ee_sub_1f4870<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let s = m.r32(o + 0x5c);
    let (a, b, c) = (m.r32(s + 0xa0), m.r32(o + 0x60), m.r32(s + 0xb0));
    m.call(0x0022_fcc8, &[a, b, c], &[]);
}
fn set_slot<M: QMem + ?Sized>(m: &mut M, o: u32, q: V4, off: u32) {
    let s = m.r32(o + off);
    let id = m.r16(s + 0x98) as u32;
    if m.call(0x0023_0310, &[id], &[]).0 == 0 {
        m.call(0x0023_03a0, &[o, s, 1], &[]);
    }
    let s = m.r32(o + off);
    let (a, b) = (m.r32(o + 0x60), m.r32(s + 0xa0));
    let r = m.call(0x0022_fcc8, &[a, b], &[q]);
    m.wv(s + 0xb0, r.1);
    m.call(0x0023_0598, &[o, s], &[]);
    m.call(0x001f_4bf8, &[o], &[]);
    m.call(0x001f_5078, &[o], &[]);
    m.call(0x001f_8668, &[o, 0], &[]);
}
/// FUN_001f4898 / 1f4930. PARTIAL: the 128-bit result of FUN_0022fcc8 is stored from its vector return
/// (the decompile shows only the low 64 bits plus unnamed registers).
pub fn ee_sub_1f4898<M: QMem + ?Sized>(m: &mut M, o: u32, q: V4) {
    set_slot(m, o, q, 0x58)
}
pub fn ee_sub_1f4930<M: QMem + ?Sized>(m: &mut M, o: u32, q: V4) {
    set_slot(m, o, q, 0x5c)
}
/// FUN_001f4b58: mark (+0x70 bit 0) and register the object at +0xec in the list 0x3e11b8
pub fn ee_sub_1f4b58<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let i = m.r32(o + 0xec);
    if i != 0 && m.r64(i + 0x70) & 1 == 0 {
        let v = m.r32(i + 0x70) | 1;
        m.w32(i + 0x70, v);
        let k = m.r32(o + 0xec);
        m.call(0x0011_a220, &[0x003e_11b8, k], &[]);
    }
}

/// FUN_001f4ba8: clear bit 0 of the +0xec record's flags and unlink it from list 0x3e11b8
pub fn ee_sub_1f4ba8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let i = m.r32(o + 0xec);
    if i != 0 && m.r64(i + 0x70) & 1 != 0 {
        let v = m.r32(i + 0x70) & !1;
        m.w32(i + 0x70, v);
        let k = m.r32(o + 0xec);
        m.call(0x0011_a2e8, &[0x003e_11b8, k], &[]);
    }
}
/// FUN_001f4bf8: 1f4b58 + bit 1 / list 0x3e11c4
pub fn ee_sub_1f4bf8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    ee_sub_1f4b58(m, o);
    let i = m.r32(o + 0xec);
    if i != 0 && m.r64(i + 0x70) & 2 == 0 {
        let v = m.r32(i + 0x70) | 2;
        m.w32(i + 0x70, v);
        let k = m.r32(o + 0xec);
        m.call(0x0011_a220, &[0x003e_11c4, k], &[]);
    }
}
/// FUN_001f4d50
pub fn ee_sub_1f4d50<M: QMem + ?Sized>(m: &mut M, o: u32) {
    ee_sub_1f4ba8(m, o);
    let i = m.r32(o + 0xec);
    if i != 0 && m.r64(i + 0x70) & 2 != 0 {
        let v = m.r32(i + 0x70) & !2;
        m.w32(i + 0x70, v);
        let k = m.r32(o + 0xec);
        m.call(0x0011_a2e8, &[0x003e_11c4, k], &[]);
    }
}
/// FUN_001f4da8: unlink every node of the chain (next at +0xac) whose slot (+8, or +0x48 when +8 == a) equals
/// b, collect them reversed, then 1f5078(a) + free via FUN_001a7e10
pub fn ee_sub_1f4da8<M: QMem + ?Sized>(m: &mut M, a: u32, b: u32, mut plist: u32) {
    let mut acc = 0u32;
    loop {
        let n = m.r32(plist);
        if n == 0 {
            break;
        }
        let sel = if m.r32(n + 8) == a { 0x40 } else { 0 };
        if m.r32(n + sel + 8) == b {
            let nx = m.r32(n + 0xac);
            m.w32(plist, nx);
            m.w32(n + 0xac, acc);
            acc = n;
        } else {
            plist = n + 0xac;
        }
    }
    if acc != 0 {
        ee_sub_1f5078(m, a);
        m.call(0x001a_7e10, &[0x003d_d490, acc], &[]);
    }
}
/// FUN_001f4e28: release the record at +0xec (child list at +0x64 -> detach both ends via vslot +0x1b4,
/// free the nodes, then vslot +0x1b4(this, 0) and free the +0x68 buffer)
pub fn ee_sub_1f4e28<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.r32(o + 0xec) == 0 {
        return;
    }
    ee_sub_1f4bf8(m, o);
    let mut n = rr(m, o + 0xec, 0x64);
    if n != 0 {
        loop {
            let rec = m.r32(o + 0xec);
            let a = m.r32(rec + 0x60);
            let n3 = m.r32(n + 0xc);
            vcl(m, a, 0x1b0, 0x1b4, &[n3], &[]);
            let n3 = m.r32(n + 0xc);
            let r60 = rr(m, o + 0xec, 0x60);
            vcl(m, n3, 0x1b0, 0x1b4, &[r60], &[]);
            let k = m.r32(n);
            m.call(0x001a_7e10, &[0x003d_d490, k], &[]);
            let back = m.r32(n + 8);
            m.w32(back, 0);
            m.w32(n, 0);
            n = m.r32(n + 4);
            if n == 0 {
                break;
            }
        }
    }
    vcl(m, o, 0x1b0, 0x1b4, &[0], &[]);
    let b = rr(m, o + 0xec, 0x68);
    m.call(0x001a_7e10, &[0x003d_d490, b], &[]);
    let r = m.r32(o + 0xec);
    m.w32(r + 0x68, 0);
}
/// FUN_001f5078
pub fn ee_sub_1f5078<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.r32(o + 0xec) != 0 {
        vcl(m, o, 0x178, 0x17c, &[4], &[]);
        if m.r32(o + 0xe8) != 0 {
            let a = m.r32(o + 0x58);
            m.call(0x0023_0598, &[o, a], &[]);
            let b = m.r32(o + 0x5c);
            m.call(0x0023_0598, &[o, b], &[]);
        }
    }
}
/// FUN_001f5480: grow the box of the record (+0xec): min(.., p - r), max(.., p + r), and the radius (+0x30)
/// to the distance from the centre (+0x20). `r` (f12) is splatted; `p` is the a1 128-bit register.
/// VER4: o = a0, p = a1 (qword), r = f12 (caller order `&[o], &[p, s(f)]`); vmini/vmax are .xyz (w kept)
pub fn ee_sub_1f5480<M: QMem + ?Sized>(m: &mut M, o: u32, p: V4, r: f32) {
    let b = m.r32(o + 0xec);
    let (mut lo, mut hi) = (m.rv(b), m.rv(b + 0x10));
    for k in 0..3 {
        lo[k] = lo[k].min(p[k] - r);
        hi[k] = hi[k].max(p[k] + r);
    }
    m.wv(b, lo);
    m.wv(b + 0x10, hi);
    let c = m.rv(b + 0x20);
    let d = [p[0] - c[0], p[1] - c[1], p[2] - c[2]];
    let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    if m.rf(b + 0x30) < l {
        m.wf(b + 0x30, l);
    }
}
/// FUN_001f5510: out[0..4] = unit(rows(rec) . weights[idx]) (or DAT_002a2dd0 when shorter than 1e-4),
/// out[+0x10] = DAT_002c3c20[idx]
/// (a0 unused; rec = a1, idx = a2, out = a3; xyz from the rows, w lane = rec+0x2c (vmaddz.xyz keeps vf03.w))
pub fn ee_sub_1f5510<M: QMem + ?Sized>(m: &mut M, _o: u32, rec: u32, idx: u32, out: u32) {
    let k = m.r32(0x002c_3c20 + idx * 4);
    m.w32(out + 0x10, k);
    let (r0, r1, r2) = (m.rv(rec), m.rv(rec + 0x10), m.rv(rec + 0x20));
    let w = m.rv(0x002c_3ba0 + idx * 0x10);
    let mut v = [0f32; 4];
    for i in 0..3 {
        v[i] = r0[i] * w[0] + r1[i] * w[1] + r2[i] * w[2];
    }
    v[3] = r2[3];
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    let o = if l < 0.0001 {
        m.rv(0x002a_2dd0)
    } else {
        let inv = 1.0 / l;
        [v[0] * inv, v[1] * inv, v[2] * inv, v[3] * inv]
    };
    m.wv(out, o);
}
/// FUN_001f55c8: re-parent the sub-records of nearby objects (kind 0x30, up to 0x40) to the shared record
pub fn ee_sub_1f55c8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if (m.r64(o + 0x28) >> 0x3c) & 1 == 0 {
        return;
    }
    let sp = stack_push(m, 0x100);
    let r = m.call(0x001f_2798, &[o], &[]).0;
    if r != rr(m, o + 0xec, 0x60) {
        let n = m.call(0x0015_e9c8, &[0x003d_d490, 1, 0x30, o, 0x40, sp], &[]).0 as i32;
        for k in 0..n.max(0) {
            let i1 = m.r32(sp + 4 * k as u32);
            if m.r32(i1 + 0xec) == 0 {
                continue;
            }
            let i3 = m.call(0x001f_2798, &[i1], &[]).0;
            if i3 == rr(m, i1 + 0xec, 0x60) {
                continue;
            }
            m.call(0x001f_2ba8, &[i1], &[]);
            let mut i5 = m.r32(i3 + 0xec);
            if i5 == 0 {
                let u = vcl(m, i3, 0x1a8, 0x1ac, &[], &[]).0;
                m.w32(i3 + 0xec, u);
                ee_sub_134320(m, 0x003d_d490, i3);
                i5 = m.r32(i3 + 0xec);
            }
            m.w32(i1 + 0xec, i5);
            m.call(0x001f_2b20, &[i1], &[]);
        }
    }
    stack_pop(m, 0x100);
    ee_sub_134320(m, 0x003d_d490, o);
    ee_sub_1f5078(m, o);
}
fn slot_update<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32, slot: u32, outoff: u32, chk: u32) {
    if m.r32(o + 0xe8) == 0 {
        return;
    }
    let s = m.r32(o + slot);
    if m.r32(s + 0x90) == 0 && m.r16(s + 0x98) as i16 != 2 {
        m.call(0x0023_03a0, &[o, s, 1], &[]);
    }
    let b = m.r32(o + 0xe8);
    ee_sub_1f5510(m, o, s, k, b + outoff);
    ee_sub_1f55c8(m, o);
    if rr(m, o + 0xe8, chk) != 3 {
        m.call(0x001f_8668, &[o, 0], &[]);
    }
}
/// FUN_001f5720
pub fn ee_sub_1f5720<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32) {
    slot_update(m, o, k, 0x58, 0xb0, 0xc0)
}
/// FUN_001f57c0
pub fn ee_sub_1f57c0<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32) {
    slot_update(m, o, k, 0x5c, 0x90, 0xa0)
}
/// FUN_001f5860: the 64-bit at block+0x80 (or DAT_002a2dd0 without a block)
pub fn ee_sub_1f5860<M: QMem + ?Sized>(m: &mut M, o: u32) -> u64 {
    let b = m.r32(o + 0xe8);
    if b != 0 {
        // VFIX: the asm is `lq` (128-bit load ignores the low 4 address bits); low dword returned
        return m.r64((b + 0x80) & !0xf);
    }
    m.r64(0x002a_2dd0 & !0xf)
}
/// FUN_001f5880
pub fn ee_sub_1f5880<M: QMem + ?Sized>(m: &mut M, o: u32, q: V4) {
    if m.r32(o + 0xe8) == 0 {
        return;
    }
    let s = m.r32(o + 0x58);
    if m.r32(s + 0x90) == 0 && m.r16(s + 0x98) as i16 != 2 {
        m.call(0x0023_03a0, &[o, s, 1], &[]);
    }
    let b = m.r32(o + 0xe8);
    m.wv(b + 0x80, q);
    ee_sub_1f5078(m, o);
}
/// FUN_001f58f0: PARTIAL: the high word of the 64-bit result of FUN_0022d348 (stored at out+0x24) is not
/// visible in the decompile; it is written as 0.
pub fn ee_sub_1f58f0<M: QMem + ?Sized>(m: &mut M, o: u32, out: u32) {
    let i1 = m.r32(o + 0x60);
    let q = m.rv(i1 + 0x70);
    for k in 0..4 {
        m.wf(out + 0x10 + 4 * k as u32, q[k]);
    }
    // VFIX3 per asm: `jal 0x22d348; sq v0, 16(s1)` then `sq v0, 32(s1)` = the full 128-bit return (FUN_0022d348
    // ends `lq v0, 144(sp)`), not x/y + the input z/w
    let r = m.call(0x0022_d348, &[i1, 0], &[q]);
    m.wv(out + 0x20, r.1);
    // BOSS R29: FUN_0011b938 writes 3 quadwords at param_3 (the original's fStack_60 frame slot); a 0x10
    // reservation let the +0x20 row land on the emulated SP word (q2mem stack_push overflow panics)
    let sp = stack_push(m, 0x30);
    m.call(0x0011_b938, &[i1 + 0x40, 0, sp], &[]);
    let (f0, f1) = (m.rf(sp), m.rf(sp + 4));
    let a = m.call(0x0024_58f0, &[(-f1).to_bits(), (-f0).to_bits()], &[]).1[0];
    stack_pop(m, 0x30);
    m.wf(out + 0x30, a);
    m.w32(out + 0x34, 0x42c8_0000);
    m.w32(out, 0);
    m.w32(out + 0x3c, 0);
    let t = m.r32(0x002d_ded0);
    m.w32(out + 0x38, t);
}

// ---- run_001fa970 .. run_001fb710
/// FUN_001fa970: split `v` against the unit direction `n` (16 bytes at p1, mode word at p1+0x10):
/// mode 2 -> n * (v . n) (component along), 1 -> v - n * (v . n) (component across), 3 -> 0, else v
pub fn ee_sub_1fa970<M: QMem + ?Sized>(m: &mut M, p1: u32, v: V4) -> V4 {
    let mode = m.r32(p1 + 0x10) as i32;
    let n = m.rv(p1);
    let d = v[0] * n[0] + v[1] * n[1] + v[2] * n[2];
    match mode {
        2 => [n[0] * d, n[1] * d, n[2] * d, n[3] * d],
        1 => [v[0] - n[0] * d, v[1] - n[1] * d, v[2] - n[2] * d, v[3] - n[3] * d],
        3 => [0.0; 4],
        _ => v,
    }
}
/// FUN_001faa00: v / (block +0xd0) -> FUN_001fa810(block + 0xb0, slot +0x58, scaled)
pub fn ee_sub_1faa00<M: QMem + ?Sized>(m: &mut M, o: u32, v: V4) {
    let b = m.r32(o + 0xe8);
    let q = 1.0 / m.rf(b + 0xd0);
    let s = [v[0] * q, v[1] * q, v[2] * q, v[3] * q];
    let slot = m.r32(o + 0x58);
    m.call(0x001f_a810, &[b + 0xb0, slot], &[s]);
}
/// FUN_001faa48: cross((a1 - position(this)), a2) -> 1faa98. PARTIAL: the w lane of the product (VU OPMSUB
/// leaves it unspecified) is passed as 0.
pub fn ee_sub_1faa48<M: QMem + ?Sized>(m: &mut M, o: u32, a1: V4, a2: V4) {
    let p = m.call(0x001a_62e8, &[o], &[]).1;
    let d = [a1[0] - p[0], a1[1] - p[1], a1[2] - p[2]];
    let c = [d[1] * a2[2] - a2[1] * d[2], d[2] * a2[0] - a2[2] * d[0], d[0] * a2[1] - a2[0] * d[1], 0.0];
    ee_sub_1faa98(m, o, c);
}
fn row_mul(v: V4, r0: V4, r1: V4, r2: V4) -> V4 {
    let mut o = [0f32; 4];
    for k in 0..4 {
        o[k] = r0[k] * v[0] + r1[k] * v[1] + r2[k] * v[2];
    }
    o
}
/// FUN_001faa98: carry a vector through the slot +0x5c matrix (FUN_0011b340 at +0x40), the block (+0xe8) rows
/// +0x30/+0x40/+0x50 and the slot rows +0x40/+0x50/+0x60, then FUN_001fa810(block + 0x90, slot, result)
pub fn ee_sub_1faa98<M: QMem + ?Sized>(m: &mut M, o: u32, v: V4) {
    let slot = m.r32(o + 0x5c);
    let sp = stack_push(m, 0x40);
    m.call(0x0011_b340, &[sp, slot + 0x40], &[]);
    let b = m.r32(o + 0xe8);
    let (r0, r1, r2) = (m.rv(sp), m.rv(sp + 0x10), m.rv(sp + 0x20));
    let t1 = row_mul(v, r0, r1, r2);
    let (a0, a1, a2) = (m.rv(b + 0x30), m.rv(b + 0x40), m.rv(b + 0x50));
    let t2 = row_mul(t1, a0, a1, a2);
    let (s0, s1, s2) = (m.rv(slot + 0x40), m.rv(slot + 0x50), m.rv(slot + 0x60));
    let t3 = row_mul(t2, s0, s1, s2);
    stack_pop(m, 0x40);
    let s = m.r32(o + 0x5c);
    m.call(0x001f_a810, &[b + 0x90, s], &[t3]);
}
/// FUN_001fac98: add FUN_0022fcc8(0, slot +0xa0, q) to the slot's +0xf0 quad when FUN_00230380(slot id) is set.
/// PARTIAL: the 128-bit result of 22fcc8 is taken from its vector return.
pub fn ee_sub_1fac98<M: QMem + ?Sized>(m: &mut M, o: u32, q: V4) {
    let slot = m.r32(o + 0x5c);
    let id = m.r16(slot + 0x98) as u32;
    if m.call(0x0023_0380, &[id], &[]).0 != 0 {
        let old = m.rv(slot + 0xf0);
        let k = m.r32(slot + 0xa0);
        let u = m.call(0x0022_fcc8, &[0, k], &[q]).1;
        m.wv(slot + 0xf0, [old[0] + u[0], old[1] + u[1], old[2] + u[2], old[3] + u[3]]);
    }
}
/// FUN_001faf50
pub fn ee_sub_1faf50<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x001f_ae18, &[1, 0xffff], &[]);
}
/// FUN_001fafd0
pub fn ee_sub_1fafd0<M: QMem + ?Sized>(m: &mut M, p: u32) -> u32 {
    m.w32(p + 0xc, 0x002d_9d00);
    m.w32(p + 0x44, 0x3f80_0000);
    m.w32(p + 0x30, 0x4396_0000);
    m.w32(p + 0x34, 0x453b_8000);
    m.w32(p, 0);
    m.w32(p + 4, 0);
    m.w32(p + 0x10, 0x3f80_0000);
    p
}
/// FUN_001fb2f8: set flag +0x1c; when cleared and the vector at +0x20 is (almost, 1e-4 = 0x38d1b717) equal to
/// DAT_002a2dd0 call FUN_001fb658(this, 0x200, 0); when set call it with (0x200, 0x200)
pub fn ee_sub_1fb2f8<M: QMem + ?Sized>(m: &mut M, o: u32, flag: u32) {
    m.w32(o + 0x1c, flag);
    if flag != 0 {
        m.call(0x001f_b658, &[o, 0x200, 0x200], &[]);
        return;
    }
    let (c, v) = (m.rv(0x002a_2dd0), m.rv(o + 0x20));
    let mut eq = true;
    for k in 0..3 {
        if m.call(0x0011_d4d0, &[], &[[c[k], 0.0, 0.0, 0.0], [v[k], 0.0, 0.0, 0.0], [f32::from_bits(0x38d1_b717), 0.0, 0.0, 0.0]]).0 == 0 {
            eq = false;
            break;
        }
    }
    if eq {
        m.call(0x001f_b658, &[o, 0x200, 0], &[]);
    }
}
/// FUN_001fb658(o, mask, val): +0x50 = +0x50 & !mask | val & mask; nonzero -> FUN_001fb6a8 (register)
/// else FUN_001fb710 (unregister)
pub fn ee_sub_1fb658<M: QMem + ?Sized>(m: &mut M, o: u32, mask: u32, val: u32) {
    let (mask, val) = (mask & 0xffff, val & 0xffff);
    let f = (m.r16(o + 0x50) as u32 & !mask | val & mask) & 0xffff;
    m.w16(o + 0x50, f as u16);
    if f != 0 {
        m.call(0x001f_b6a8, &[o], &[]);
    } else {
        m.call(0x001f_b710, &[o], &[]);
    }
}
/// FUN_001fb520: empty virtual
pub fn ee_sub_1fb520() {}
/// FUN_001fb6a8: first registration in list 0x3dd08c calls the object's vslot (+8 adj, +0xc fn)
pub fn ee_sub_1fb6a8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.call(0x0011_a380, &[0x003d_d08c, o], &[]).0 == 0 {
        vcl(m, o, 8, 0xc, &[], &[]);
        m.call(0x0011_a220, &[0x003d_d08c, o], &[]);
    }
}
/// FUN_001fb710: unregister from list 0x3dd08c and call vslot (+0x10 adj, +0x14 fn)
pub fn ee_sub_1fb710<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.call(0x0011_a380, &[0x003d_d08c, o], &[]).0 != 0 {
        m.call(0x0011_a2e8, &[0x003d_d08c, o], &[]);
        vcl(m, o, 0x10, 0x14, &[], &[]);
    }
}
