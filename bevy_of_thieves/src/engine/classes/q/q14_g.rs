//! WP-Q14 part G: file package run_0016ef00 (GS packet helpers + the free-list allocator FUN_001704b8/170548).
//! The GS display / vsync interrupt part (FUN_0016f2d0 .. 0016f8b8, 00170278, 001700d0, 00170348) is hardware
//! register + semaphore code replaced by the Bevy frame loop (see port_tracking). 1:1 from
//! ghidra_functions(_v2).body on `QMem`.
#![allow(unused_variables, unused_mut, dead_code)]
use crate::engine::classes::q::q10_e::pool_static_init;
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem};
use crate::engine::game::q_00166008::{
    ee_sub_166920, ee_sub_166968, ee_sub_1669e0, ee_sub_166bd8, ee_sub_166e90, ee_sub_167150,
};

/// FUN_0016f960(dst, rgba*, mode): full-screen sprite strip clear to the colour at `c` (mode 1 / 2 / 3 picks the
/// FRAME-style register 0x47 value; other modes use an uninitialised register in the original -> 0)
pub fn ee_sub_16f960<M: QMem + ?Sized>(m: &mut M, dst: u32, c: u32, mode: u32) {
    let pk = stack_push(m, 0x90);
    ee_sub_166920(m, pk);
    m.call(0x0015_7330, &[pk, 0x25, dst], &[]);
    let prim = if (m.r8(c + 3) as i8) < 0 { 6 } else { 0x46 };
    ee_sub_166bd8(m, pk, prim, 1, 0xe);
    let rgba = m.r8(c) as u64 | (m.r8(c + 1) as u64) << 8 | (m.r8(c + 2) as u64) << 16 | (m.r8(c + 3) as u64) << 24;
    ee_sub_166e90(m, pk, 1, rgba);
    let v = match mode {
        2 => 0x3_2001u64,
        1 => 0x3_1001,
        3 => 0x3_0000,
        _ => 0,
    };
    ee_sub_166e90(m, pk, 0x47, v);
    ee_sub_166e90(m, pk, 0x42, 0x44);
    let (mut s2, mut s1, mut s0) = (0x7200u32, 0x7000u32, 0x1e0i32);
    loop {
        ee_sub_166e90(m, pk, 5, (s1 | 0x7900_0000) as u64);
        s1 += 0x200;
        ee_sub_166e90(m, pk, 5, (s2 | 0x8700_0000) as u64);
        s2 += 0x200;
        s0 -= 0x20;
        if s0 < 0 {
            break;
        }
    }
    ee_sub_166e90(m, pk, 0x47, 0x7_0000);
    ee_sub_166968(m, pk);
    stack_pop(m, 0x90);
}
/// FUN_0016faf8(a, b, c)
pub fn ee_sub_16faf8<M: QMem + ?Sized>(m: &mut M, a: u32, b: u32, c: u32) {
    let pk = stack_push(m, 0x70);
    ee_sub_166920(m, pk);
    m.call(0x0015_7330, &[pk, a, b], &[]);
    ee_sub_166bd8(m, pk, 0, 1, 0xe);
    ee_sub_166e90(m, pk, 0x4a, c as u64);
    ee_sub_1669e0(m, pk, 1);
    stack_pop(m, 0x70);
}
fn bufs<M: QMem + ?Sized>(m: &mut M) -> (u32, u32) {
    if m.r32(0x002d_edb0) & 1 == 0 {
        (0x002d_ecd0, 0x002d_ebe0)
    } else {
        (0x002d_ebe0, 0x002d_ecd0)
    }
}
/// FUN_0016fb78: DISPFB-style register 7 value of both frame buffers
pub fn ee_sub_16fb78<M: QMem + ?Sized>(m: &mut M) {
    let (p1, p2) = bufs(m);
    let sp = stack_push(m, 0x60);
    for (pk, tag, src) in [(sp, 0x002d_fb00u32, p1), (sp + 0x30, 0x002d_fb20, p2)] {
        ee_sub_166920(m, pk);
        m.call(0x0015_7330, &[pk, 2, tag], &[]);
        ee_sub_166bd8(m, pk, 0, 1, 0xe);
        let v = m.r64(src);
        ee_sub_166e90(m, pk, 7, (v & 0x1ff) << 5 | ((v >> 16) & 0x3f) << 14 | 0x6_a800_0000);
        ee_sub_166968(m, pk);
    }
    stack_pop(m, 0x60);
}
/// FUN_0016fca8(a, b, c, d, e)
pub fn ee_sub_16fca8<M: QMem + ?Sized>(m: &mut M, a: u32, b: u32, c: u32, d: u32, e: u32) {
    let (p3, p2) = if m.r32(0x002d_edb0) & 1 == 0 { (0x002d_ecd0u32, 0x002d_ebe0u32) } else { (0x002d_ebe0, 0x002d_ecd0) };
    let sp = stack_push(m, 0x90);
    let (pk, w, pk2) = (sp, sp + 0x30, sp + 0x50);
    ee_sub_166920(m, pk);
    m.call(0x0015_7330, &[pk, a, b], &[]);
    ee_sub_166bd8(m, pk, 0, 1, 0xe);
    let a0 = m.r64(p2);
    let hi = (e as u64) << 32;
    ee_sub_166e90(m, pk, 0x4c, a0 & 0x3f3f_01ff | hi);
    let v = m.r64(p2 + 0x10);
    ee_sub_166e90(m, pk, 0x4e, v);
    let v = m.r64(p2 + 0x20);
    ee_sub_166e90(m, pk, 0x18, v);
    let v = m.r64(p2 + 0x30);
    ee_sub_166e90(m, pk, 0x40, v);
    let v = m.r64(p2 + 0x70);
    ee_sub_166e90(m, pk, 0x47, v);
    ee_sub_166bd8(m, pk, 0x200, 1, 0xe);
    let a0 = m.r64(p2);
    ee_sub_166e90(m, pk, 0x4d, a0 & 0x3f3f_01ff | hi);
    let v = m.r64(p2 + 0x10);
    ee_sub_166e90(m, pk, 0x4f, v);
    let v = m.r64(p2 + 0x20);
    ee_sub_166e90(m, pk, 0x19, v);
    let v = m.r64(p2 + 0x30);
    ee_sub_166e90(m, pk, 0x41, v);
    let v = m.r64(p2 + 0x70);
    ee_sub_166e90(m, pk, 0x48, v);
    ee_sub_166968(m, pk);
    m.call(0x0023_33b4, &[w, 0, 0x20], &[]);
    let u = m.r64(p2);
    let mut d8 = (((u & 0x1ff) << 5) as u32) | ((((u >> 16) & 0x3f) << 14) as u32);
    let e8 = d8 | 0xa800_0000;
    let f0 = u as u32;
    let e0 = f0 & 0xc0ff_ffff | 0x200_0000;
    d8 |= 0xa820_0000;
    let u = m.r64(p3);
    let ec = u as u32;
    let dc = ec & 0xc0ff_ffff | 0x200_0000;
    let e4 = (((u & 0x1ff) << 5) as u32) | 0xa800_0000 | ((((u >> 16) & 0x3f) << 14) as u32);
    let d4 = e4 | 0x20_0000;
    for (i, v) in [f0, ec, e8, e4, e0, dc, d8, d4].into_iter().enumerate() {
        m.w32(w + 4 * i as u32, v);
    }
    m.call(0x0022_6c10, &[pk2], &[]);
    m.call(0x0015_7330, &[pk2, c, d], &[]);
    m.call(0x0015_74b8, &[pk2], &[]);
    m.call(0x0022_7148, &[pk2, a, b, 0], &[]);
    m.call(0x0022_6ca8, &[pk2, 0, 0], &[]);
    // VFIX3 per asm: FUN_00226f28(pk2, 0xc, 2, w, 0xb) (a0 = s0, a1 = 0xc, a2 = 2, a3 = s1, t0 = 0xb)
    m.call(0x0022_6f28, &[pk2, 0xc, 2, w, 0xb], &[]);
    m.call(0x0015_76f0, &[pk2], &[]);
    m.call(0x0015_7620, &[pk2], &[]);
    stack_pop(m, 0x90);
}
/// FUN_00170048
pub fn ee_sub_170048<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_16fca8(m, 0xc, 0x002d_f740, 8, 0x002d_f800, 0);
    let e = m.r32(0x002b_e598);
    ee_sub_16fca8(m, 0xc, 0x002d_f880, 8, 0x002d_f940, e);
    let e = m.r32(0x002b_e59c);
    ee_sub_16fca8(m, 0xc, 0x002d_f9c0, 8, 0x002d_fa80, e);
    ee_sub_16fb78(m);
}
/// FUN_001703c0: ceil(log2(n)) (0 for n < 2, capped at 0x1f)
pub fn ee_sub_1703c0(n: u32) -> u32 {
    let mut r = 0;
    if 1 < n {
        let mut v = 2u32;
        loop {
            r += 1;
            if n <= v {
                return r;
            }
            v <<= 1;
            if r >= 0x1f {
                break;
            }
        }
    }
    r
}
/// FUN_001703f8(p, a, b)
pub fn ee_sub_1703f8<M: QMem + ?Sized>(m: &mut M, p: u32, a: u32, b: u32) {
    m.call(0x0023_33b4, &[p, 0, 0x630], &[]);
    m.w32(p + 8, b);
    m.w32(p, a);
    m.w32(p + 4, a);
}
/// FUN_00170448: free-list size class of a chunk of `n` bytes (0xffffffff for 0)
pub fn ee_sub_170448(n: i64) -> u32 {
    if 8 < n {
        if n < 0x21 {
            return if n < 0x11 { 4 } else { 5 };
        }
        return if n < 0x41 { 6 } else { 7 };
    }
    if 2 < n {
        return if n < 5 { 2 } else { 3 };
    }
    if n != 0 {
        return (n >= 2) as u32;
    }
    0xffff_ffff
}
/// FUN_001704b8(a): put the tail up to the next 0x80 boundary on the free list
pub fn ee_sub_1704b8<M: QMem + ?Sized>(m: &mut M, a: u32) {
    if m.r32(a + 4) & 0x7f != 0 {
        let i = m.r32(a + 0x2c);
        m.w32(a + 0x2c, i.wrapping_add(1));
        let e = a.wrapping_add(i.wrapping_mul(0xc)).wrapping_add(0x30);
        let cur = m.r32(a + 4);
        m.w32(e, cur);
        let n = cur.wrapping_add(0x7f) & 0xffff_ff80;
        m.w32(a + 4, n);
        m.w32(e + 4, n);
        let c = ee_sub_170448(n.wrapping_sub(cur) as i32 as i64);
        let slot = a.wrapping_add(0xc).wrapping_add(c.wrapping_mul(4));
        let h = m.r32(slot);
        m.w32(e + 8, h);
        m.w32(slot, e);
    }
}
/// FUN_00170548(a, n): allocate n bytes (free lists 0..7, else bump with 0x80-block refill)
pub fn ee_sub_170548<M: QMem + ?Sized>(m: &mut M, a: u32, n: i32) -> u32 {
    let mut n = n;
    let mut c = ee_sub_170448(n as i64 - 1);
    loop {
        c = c.wrapping_add(1);
        if 7 < c {
            let cur = m.r32(a + 4);
            let r = if cur.wrapping_add(n as u32) & 0xffff_ff80 == cur & 0xffff_ff80 {
                cur
            } else {
                ee_sub_1704b8(m, a);
                m.r32(a + 4)
            };
            m.w32(a + 4, r.wrapping_add(n as u32));
            return r;
        }
        let slot = a.wrapping_add(0xc).wrapping_add(c * 4);
        let e = m.r32(slot);
        if e == 0 {
            continue;
        }
        let r = m.r32(e);
        n = (r as i32).wrapping_add(n);
        m.w32(e, n as u32);
        if n as u32 == m.r32(e + 4) {
            let nx = m.r32(e + 8);
            m.w32(slot, nx);
        } else {
            let c2 = ee_sub_170448(m.r32(e + 4) as i32 as i64 - n as i64);
            if c2 != c {
                let s2 = a.wrapping_add(0xc).wrapping_add(c2.wrapping_mul(4));
                let nx = m.r32(e + 8);
                m.w32(slot, nx);
                let h = m.r32(s2);
                m.w32(e + 8, h);
                m.w32(s2, e);
            }
        }
        return r;
    }
}
/// FUN_00170650(a, b, c, d, e, f, pk): BITBLTBUF / TRXPOS / TRXREG / TRXDIR transfer set-up
pub fn ee_sub_170650<M: QMem + ?Sized>(m: &mut M, a: i64, b: i64, c: i64, d: u64, e: i64, f: u64, pk: u32) {
    ee_sub_166bd8(m, pk, 0, 1, 0xe);
    ee_sub_166e90(m, pk, 0x50, (a << 0x20 | b << 0x30 | c << 0x38) as u64);
    ee_sub_166e90(m, pk, 0x51, 0);
    ee_sub_166e90(m, pk, 0x52, d | (e << 0x20) as u64);
    ee_sub_166e90(m, pk, 0x53, 0);
    ee_sub_167150(m, pk, f);
}
/// FUN_00170730(a)
pub fn ee_sub_170730<M: QMem + ?Sized>(m: &mut M, a: u32) {
    let sp = stack_push(m, 0x80);
    let (pk, s40, s3c) = (sp, sp + 0x30, sp + 0x34);
    ee_sub_166920(m, pk);
    m.call(0x001a_e478, &[s40], &[]);
    let n = m.r32(0x003d_cf9c).wrapping_mul(3).wrapping_add(m.r32(0x003d_cf98)).wrapping_mul(8) | 4;
    m.call(0x0015_72e0, &[pk, n], &[]);
    m.call(0x0015_74b8, &[pk], &[]);
    if m.call(0x001e_cc28, &[1, a, pk], &[]).0 != 0 {
        ee_sub_166bd8(m, pk, 0, 1, 0xe);
        ee_sub_166e90(m, pk, 0x3f, 0);
        ee_sub_166e90(m, pk, 0x61, 0);
        ee_sub_166968(m, pk);
        m.call(0x0015_76b0, &[pk], &[]);
        m.call(0x0015_7348, &[pk, 0, s3c], &[]);
        let h = m.r32(0x002d_e598);
        let t = m.r32(s3c);
        m.call(0x0016_f8b8, &[h, t], &[]);
    }
    m.call(0x001a_e4a0, &[s40, 2], &[]);
    stack_pop(m, 0x80);
}
/// FUN_00170828
pub fn ee_sub_170828<M: QMem + ?Sized>(m: &mut M, a: u32) {
    m.call(0x001e_cc28, &[1, a, 0], &[]);
}
