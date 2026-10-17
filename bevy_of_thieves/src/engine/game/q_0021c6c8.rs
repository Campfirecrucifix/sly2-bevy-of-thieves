//! WP-Q2: run 0x0021c6c8..0x0021f2c0 - per-translation-unit static initialisers (the six shared pool
//! registrations), the small string helpers and the format-spec helpers of the engine's printf.
//! 1:1 against `QMem` (classes::q::q2mem); FUN_0011cca8 (pool register) and FUN_001ae5a0 (memcpy) are
//! routed through `QMem::call`.
//! REPLACED: the printf core (FUN_0021d6b0 spec parser jump table, FUN_0021d9f0 %s, FUN_0021db38 sign,
//! FUN_0021dc70 / 0021dd40 integer, FUN_0021e048 / 0021e0d8 / 0021e1b0 / 0021e4b8 float, FUN_0021e718
//! dispatch, FUN_0021eac8 loop, FUN_0021ebb8 / 0021ec18 printf, FUN_0021ec60 / 0021ecd8 sprintf):
//! std::fmt (`format!`) at the Rust call sites (e.g. ui::bind).
use crate::engine::classes::q::q2mem::QMem;

/// FUN_0021c6c8 / FUN_0021d120 / FUN_0021d4e0 / FUN_0021ee78 / FUN_0021f188 (identical bodies, one per
/// TU): `(init, prio)`: when prio == 0xffff and init != 0 each counter is bumped and the first bump
/// registers its pool.
pub fn ee_sub_21c6c8<M: QMem + ?Sized>(m: &mut M, init: u32, prio: u32) {
    crate::engine::game::q_001e9090::ee_sub_1ea410(m, init, prio);
}

/// FUN_0021c800 / FUN_0021d258 / FUN_0021d618 / FUN_0021efb0 / FUN_0021f2c0: `static_init(1, 0xffff)`
pub fn ee_sub_21c800<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_21c6c8(m, 1, 0xffff)
}

/// FUN_0021d638: parse an optional '-' and decimal digits at `p`; `*out = value * sign`; returns the
/// number of characters consumed.
pub fn ee_sub_21d638<M: QMem + ?Sized>(m: &mut M, mut p: u32, out: u32) -> u32 {
    let sign: i32 = if m.r8(p) == b'-' { -1 } else { 1 };
    if sign == -1 {
        p += 1;
    }
    let mut n = (sign == -1) as u32;
    let mut v: i32 = 0;
    while m.r8(p).wrapping_sub(b'0') < 10 {
        n += 1;
        v = v.wrapping_mul(10).wrapping_add(m.r8(p) as i32 - 0x30);
        p += 1;
    }
    m.w32(out, v.wrapping_mul(sign) as u32);
    n
}

/// FUN_0021dc18: radix of a conversion character (b/B 2, o 8, x 16, else 10)
pub fn ee_sub_21dc18(c: u8) -> u32 {
    match c {
        b'B' | b'b' => 2,
        b'o' => 8,
        b'x' => 16,
        _ => 10,
    }
}

/// FUN_0021ed28: strlen
pub fn ee_sub_21ed28<M: QMem + ?Sized>(m: &mut M, mut p: u32) -> u32 {
    let mut n = 0;
    while m.r8(p) != 0 {
        p += 1;
        n += 1;
    }
    n
}

/// FUN_0021ed58: strcpy = FUN_001ae5a0(dst, src, strlen(src) + 1); returns dst
pub fn ee_sub_21ed58<M: QMem + ?Sized>(m: &mut M, dst: u32, src: u32) -> u32 {
    let n = ee_sub_21ed28(m, src);
    m.call(0x001a_e5a0, &[dst, src, n + 1], &[]);
    dst
}

/// FUN_0021eda0: strcat = FUN_001ae5a0(dst + strlen(dst), src, strlen(src) + 1); returns dst
pub fn ee_sub_21eda0<M: QMem + ?Sized>(m: &mut M, dst: u32, src: u32) -> u32 {
    let a = ee_sub_21ed28(m, dst);
    let b = ee_sub_21ed28(m, src);
    m.call(0x001a_e5a0, &[dst + a, src, b + 1], &[]);
    dst
}

/// FUN_0021edf8: strchr (the terminator itself is not matched unless c == 0)
/// VFIX: `c` is compared as the full register (beq lbu, a1): c > 0xff never matches
pub fn ee_sub_21edf8<M: QMem + ?Sized>(m: &mut M, mut p: u32, c: u32) -> u32 {
    loop {
        let b = m.r8(p);
        if b as u32 == c {
            return p;
        }
        p += 1;
        if b == 0 {
            return 0;
        }
    }
}

/// FUN_0021ee30: strupr through the ctype table at 0x002aa6a1 (bit 2 = lower case -> c - 0x20)
pub fn ee_sub_21ee30<M: QMem + ?Sized>(m: &mut M, mut p: u32) {
    let mut c = m.r8(p);
    while c != 0 {
        let lower = m.r8(0x002a_a6a1 + c as u32) & 2 != 0;
        m.w8(p, if lower { c.wrapping_sub(0x20) } else { c });
        p += 1;
        c = m.r8(p);
    }
}
