//! WIRE-S3 batch 3
#![allow(unused)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

/// FUN_001f4fb0: empty stub.
pub fn ee_sub_1f4fb0<M: QMem + ?Sized>(m: &mut M) {}

/// FUN_002040f8: +0x204 != 0 ? 0 : 7.
pub fn ee_sub_2040f8<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    if m.r32(a0 + 0x204) != 0 { 0 } else { 7 }
}

/// FUN_0024a780: copysign(f12, f13).
pub fn ee_sub_24a780<M: QMem + ?Sized>(m: &mut M, f12: f32, f13: f32) -> f32 {
    f32::from_bits((f12.to_bits() & 0x7fff_ffff) | (f13.to_bits() & 0x8000_0000))
}

/// FUN_00180ff0: FUN_0017f1a0(a0, a1, a2); +0x188 != 0 && a1[(1 - a2) * 0x40 + 8] == *0x2dfb80 -> FUN_0017f5b0(a0).
pub fn ee_sub_180ff0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    m.call(0x0017_f1a0, &[a0, a1, a2], &[]);
    if m.r32(a0 + 0x188) == 0 {
        return;
    }
    let e = m.r32(a1.wrapping_add(1u32.wrapping_sub(a2) << 6) + 8);
    if e == m.r32(0x002d_fb80) {
        m.call(0x0017_f5b0, &[a0], &[]);
    }
}

/// FUN_0018ed98: a = FUN_001a6f00(0, o, FUN_001a62e8(+0x19a4)); b = FUN_001a6f00(0, o, *(+0x19c0)+0x70); 0 < 0.5*(a.y+b.y).
pub fn ee_sub_18ed98<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let h = m.r32(a0 + 0x19a4);
    let p = m.call(0x001a_62e8, &[h], &[]).1;
    let a = m.call(0x001a_6f00, &[0, a0], &[p]).1;
    let q = m.r32(a0 + 0x19c0);
    let qv = m.rv(q + 0x70);
    let b = m.call(0x001a_6f00, &[0, a0], &[qv]).1;
    let y = a[1] * 0.5 + b[1] * 0.5;
    if 0.0 < y { 1 } else { 0 }
}

/// FUN_00235b08: vprintf core: a0 = putc callback(a1 ctx, &a2 slot, ch), a3 = fmt, t0 = va_list (8-byte slots); returns sum of callback results.
pub fn ee_sub_235b08<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, t0: u32) -> u32 {
    fn put<M: QMem + ?Sized>(m: &mut M, cb: u32, ctx: u32, p: u32, ch: u32) -> u32 {
        m.call(cb, &[ctx, p, ch], &[]).0
    }
    fn sx(b: u8) -> u32 {
        b as i8 as i32 as u32
    }
    let sp = stack_push(m, 0x120);
    let (cb, ctx) = (a0, a1);
    m.w32(sp + 0x40, a2);
    let p5c = sp + 0x40;
    let bufe = sp + 0x1f;
    let mut fp = t0;
    let mut s6 = a3;
    let mut cnt: u32 = 0;
    let mut c = m.r8(s6);
    while c != 0 {
        let mut s7 = s6.wrapping_add(1);
        if c != b'%' {
            cnt = cnt.wrapping_add(put(m, cb, ctx, p5c, sx(c)));
            s6 = s7;
            c = m.r8(s6);
            continue;
        }
        let mut s5: u32 = 0;
        let mut width: i32 = 0;
        let mut prec: i32 = 0;
        let mut plen: i32 = 0;
        // (start s2, end-exclusive s1) of the body, or None = no tail
        let mut body: Option<(u32, u32)> = None;
        loop {
            s6 = s6.wrapping_add(1);
            let mut ch = m.r8(s6);
            match ch {
                b'-' => s5 |= 1,
                b'+' => s5 |= 2,
                b'0' => s5 |= 4,
                b' ' => s5 |= 8,
                b'#' => s5 |= 0x10,
                b'l' => s5 |= 0x40,
                b'h' => s5 |= 0x80,
                b'1'..=b'9' | b'.' => {
                    let is_prec = ch == b'.';
                    if is_prec {
                        s6 = s6.wrapping_add(1);
                        s5 |= 0x20;
                        ch = m.r8(s6);
                        if ch == b'*' {
                            fp = fp.wrapping_add(8);
                            let s0 = m.r32(fp.wrapping_sub(8)) as i32;
                            prec = if -2 < s0 { s0 } else { -1 };
                            continue;
                        }
                    }
                    let mut s0: i32 = 0;
                    while ch.wrapping_sub(0x30) < 10 {
                        s0 = s0.wrapping_mul(10).wrapping_sub(0x30).wrapping_add(ch as i8 as i32);
                        s6 = s6.wrapping_add(1);
                        ch = m.r8(s6);
                    }
                    if is_prec {
                        prec = if -2 < s0 { s0 } else { -1 };
                    } else {
                        width = s0;
                    }
                    s7 = s6.wrapping_add(1);
                    if ch == 0 {
                        break; // s6 stays on the NUL
                    }
                    s6 = s6.wrapping_sub(1);
                }
                b'o' | b'x' | b'X' | b'u' | b'p' => {
                    let tab = if ch == b'X' { 0x002a_a858u32 } else { 0x002a_a840u32 };
                    if s5 & 0x10 != 0 && (ch == b'x' || ch == b'X') {
                        m.w8(sp + 0x20, b'0');
                        m.w8(sp + 0x21, if ch == b'x' { b'x' } else { b'X' });
                        plen = 2;
                    }
                    fp = fp.wrapping_add(8);
                    let mut s0: u64 = if ch == b'p' {
                        m.r32(fp.wrapping_sub(8)) as u64
                    } else if s5 & 0x40 != 0 {
                        m.r64(fp.wrapping_sub(8))
                    } else if s5 & 0x80 != 0 {
                        m.r16(fp.wrapping_sub(8)) as u64
                    } else {
                        m.r32(fp.wrapping_sub(8)) as u64
                    };
                    let mut s1 = bufe;
                    s7 = s6.wrapping_add(1);
                    if s0 == 0 {
                        if ch == b'x' || ch == b'X' {
                            plen = 0;
                        }
                        if !(s5 & 0x20 != 0 && prec == 0) {
                            m.w8(s1, b'0');
                            s1 = s1.wrapping_sub(1);
                        }
                    } else {
                        while s0 != 0 {
                            let d = match ch {
                                b'o' => {
                                    let d = (s0 & 7) as u8 + 0x30;
                                    s0 >>= 3;
                                    d
                                }
                                b'u' => {
                                    // libgcc __umoddi3/__udivdi3 (FUN_0023cdf8 / FUN_0023c828) on 64-bit a0
                                    let d = (s0 % 10) as u8 + 0x30;
                                    s0 /= 10;
                                    d
                                }
                                _ => {
                                    let d = m.r8(tab + (s0 & 0xf) as u32);
                                    s0 >>= 4;
                                    d
                                }
                            };
                            m.w8(s1, d);
                            s1 = s1.wrapping_sub(1);
                        }
                        if ch == b'o' && s5 & 0x10 != 0 {
                            m.w8(s1, b'0');
                            s1 = s1.wrapping_sub(1);
                        }
                    }
                    if ch == b'p' {
                        m.w8(sp + 0x20, b'0');
                        m.w8(sp + 0x21, b'x');
                        plen = 2;
                    }
                    body = Some((s1.wrapping_add(1), bufe.wrapping_add(1)));
                    break;
                }
                b'd' | b'i' => {
                    fp = fp.wrapping_add(8);
                    let mut s0: i64 = if s5 & 0x40 != 0 {
                        m.r64(fp.wrapping_sub(8)) as i64
                    } else if s5 & 0x80 != 0 {
                        m.r16(fp.wrapping_sub(8)) as i16 as i64
                    } else {
                        m.r32(fp.wrapping_sub(8)) as i32 as i64
                    };
                    let sign = if s0 < 0 {
                        Some(b'-')
                    } else if s5 & 2 != 0 {
                        Some(b'+')
                    } else if s5 & 8 != 0 {
                        Some(b' ')
                    } else {
                        None
                    };
                    if let Some(sc) = sign {
                        plen = 1;
                        m.w8(sp + 0x20, sc);
                    }
                    let mut s1 = bufe;
                    s7 = s6.wrapping_add(1);
                    if s0 == 0 {
                        if !(s5 & 0x20 != 0 && prec == 0) {
                            m.w8(s1, b'0');
                            s1 = s1.wrapping_sub(1);
                        }
                    } else {
                        if !(-1 < s0) {
                            s0 = s0.wrapping_neg();
                        }
                        while s0 != 0 {
                            // libgcc __moddi3/__divdi3 (FUN_0023c158 / FUN_0023b7f8) on 64-bit a0
                            let d = (s0.wrapping_rem(10) as u64).wrapping_add(0x30) as u8;
                            m.w8(s1, d);
                            s0 = s0.wrapping_div(10);
                            s1 = s1.wrapping_sub(1);
                        }
                    }
                    body = Some((s1.wrapping_add(1), bufe.wrapping_add(1)));
                    break;
                }
                b'e' | b'f' => {
                    fp = fp.wrapping_add(8);
                    let d = m.r64(fp.wrapping_sub(8));
                    // 64-bit double in a0, passed as lo/hi words
                    let f12 = m.call(0x0023_dfa0, &[d as u32, (d >> 32) as u32], &[]).1[0];
                    s7 = s6.wrapping_add(1);
                    if f12 == 0.0 {
                        cnt = cnt.wrapping_add(put(m, cb, ctx, p5c, 0x30));
                    } else {
                        let v = m.call(0x0023_e290, &[], &[[f12, 0.0, 0.0, 0.0]]).0;
                        let r = m.call(0x0023_5930, &[cb, ctx, p5c, v], &[]).0;
                        cnt = cnt.wrapping_add(r);
                    }
                    break;
                }
                b's' => {
                    fp = fp.wrapping_add(8);
                    let v1 = m.r32(fp.wrapping_sub(8));
                    s7 = s6.wrapping_add(1);
                    if v1 == 0 {
                        for &b in b"(null)" {
                            put(m, cb, ctx, p5c, b as u32);
                        }
                        break;
                    }
                    let mut n: u32 = 0;
                    if s5 & 0x20 != 0 {
                        if prec > 0 {
                            while (n as i32) < prec && m.r8(v1.wrapping_add(n)) != 0 {
                                n += 1;
                            }
                        }
                    } else {
                        while m.r8(v1.wrapping_add(n)) != 0 {
                            n += 1;
                        }
                    }
                    body = Some((v1, v1.wrapping_add(n)));
                    break;
                }
                b'c' => {
                    fp = fp.wrapping_add(8);
                    let b = m.r8(fp.wrapping_sub(8));
                    s5 |= 0x100;
                    prec = 1;
                    s7 = s6.wrapping_add(1);
                    m.w8(bufe, b);
                    body = Some((bufe, bufe.wrapping_add(1)));
                    break;
                }
                _ => {
                    s7 = s6.wrapping_add(1);
                    if ch == 0 {
                        break;
                    }
                    s5 |= 0x100;
                    prec = 1;
                    m.w8(bufe, ch);
                    body = Some((bufe, bufe.wrapping_add(1)));
                    break;
                }
            }
        }
        if let Some((mut s2, s1)) = body {
            let len = s1.wrapping_sub(s2) as i32;
            let mut zeros = prec.wrapping_sub(len);
            if !(len < prec) {
                zeros = 0;
            }
            width = width.wrapping_sub(len).wrapping_sub(plen).wrapping_sub(zeros);
            let zp = if s5 & 0x100 != 0 { s5 & 4 != 0 } else { s5 & 0x24 == 4 };
            let a3: u32 = if zp && width >= 1 { 0x30 } else { 0x20 };
            if s5 & 1 != 0 {
                for i in 0..plen.max(0) as u32 {
                    let b = m.r8(sp + 0x20 + i);
                    cnt = cnt.wrapping_add(put(m, cb, ctx, p5c, sx(b)));
                }
                for _ in 0..zeros.max(0) {
                    cnt = cnt.wrapping_add(put(m, cb, ctx, p5c, 0x30));
                }
                while s2 < s1 {
                    let b = m.r8(s2);
                    s2 = s2.wrapping_add(1);
                    cnt = cnt.wrapping_add(put(m, cb, ctx, p5c, sx(b)));
                }
                for _ in 0..width.max(0) {
                    cnt = cnt.wrapping_add(put(m, cb, ctx, p5c, 0x20));
                }
            } else {
                let zfirst = s5 & 4 != 0 && a3 == 0x30;
                if !zfirst {
                    for _ in 0..width.max(0) {
                        cnt = cnt.wrapping_add(put(m, cb, ctx, p5c, a3));
                    }
                }
                for i in 0..plen.max(0) as u32 {
                    let b = m.r8(sp + 0x20 + i);
                    cnt = cnt.wrapping_add(put(m, cb, ctx, p5c, sx(b)));
                }
                if zfirst {
                    for _ in 0..width.max(0) {
                        cnt = cnt.wrapping_add(put(m, cb, ctx, p5c, a3));
                    }
                }
                for _ in 0..zeros.max(0) {
                    cnt = cnt.wrapping_add(put(m, cb, ctx, p5c, 0x30));
                }
                while s2 < s1 {
                    let b = m.r8(s2);
                    s2 = s2.wrapping_add(1);
                    cnt = cnt.wrapping_add(put(m, cb, ctx, p5c, sx(b)));
                }
            }
        }
        if m.r8(s6) != 0 {
            s6 = s7;
        }
        c = m.r8(s6);
    }
    put(m, cb, ctx, p5c, 0);
    stack_pop(m, 0x120);
    cnt
}
