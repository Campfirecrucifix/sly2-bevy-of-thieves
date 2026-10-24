//! WIRE-S2: 1:1 QMem ports of live-frontier natives (shard 3/4).
//! Soft-float libgcc callees (0x23d748 __adddf3, 0x23d7a0 __subdf3, 0x23d808 __muldf3, 0x23dd80 __floatsidf,
//! 0x23de38 __fixdfsi) are computed natively with f64. 64-bit (double) args to m.call: low words in normal
//! slots, high words as trailing ints starting at a[4]. Double returns read as (v0 lo, f32 bits hi in .1[0]).
#![allow(unused)]
use crate::engine::classes::q::q2mem::{vslot, QMem, V4};

const DIGITS: u32 = 0x002c_40c8; // digit char table "0123456789..."
const TEN: u64 = 0x4024_0000_0000_0000; // 10.0
const TENTH: u64 = 0x3fb9_9999_9999_999a; // 0.1
const HALF: u64 = 0x3fe0_0000_0000_0000; // 0.5
const TWO: u64 = 0x4000_0000_0000_0000; // 2.0

fn fd(b: u64) -> f64 {
    f64::from_bits(b)
}
fn lo(b: u64) -> u32 {
    b as u32
}
fn hi(b: u64) -> u32 {
    (b >> 32) as u32
}
fn ret64(r: (u32, V4)) -> u64 {
    ((r.1[0].to_bits() as u64) << 32) | r.0 as u64
}
fn floatsidf(i: u32) -> u64 {
    (i as i32 as f64).to_bits()
}
fn fixdfsi(b: u64) -> u32 {
    fd(b) as i32 as u32
}
fn mul(a: u64, b: u64) -> u64 {
    (fd(a) * fd(b)).to_bits()
}
/// FUN_002452c0 (libm pow) via m.call
fn pow<M: QMem + ?Sized>(m: &mut M, a: u64, b: u64) -> u64 {
    ret64(m.call(0x0024_52c0, &[lo(a), lo(b), 0, 0, hi(a), hi(b)], &[]))
}
/// FUN_0011d740 (double truncate) via m.call
fn trunc<M: QMem + ?Sized>(m: &mut M, a: u64) -> u64 {
    ret64(m.call(0x0011_d740, &[lo(a), 0, 0, 0, hi(a)], &[]))
}
/// FUN_0021e048 (decimal exponent of double) via m.call
fn exp10<M: QMem + ?Sized>(m: &mut M, a: u64) -> u32 {
    m.call(0x0021_e048, &[lo(a), 0, 0, 0, hi(a)], &[]).0
}

/// FUN_0021e0d8: digit char of double x (a0 lo, a0_hi) at decimal position a2 given exponent a1.
pub fn ee_sub_21e0d8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a0_hi: u32) -> u32 {
    let x = ((a0_hi as u64) << 32) | a0 as u64;
    let mut v0 = 0u32;
    if !((a1 as i32) < (a2 as i32)) && ((a1.wrapping_sub(15) as i32) < (a2 as i32)) {
        let p = pow(m, TEN, floatsidf(a2.wrapping_neg()));
        let t = trunc(m, mul(x, p));
        let u = trunc(m, mul(t, TENTH));
        let r = (fd(t) - fd(mul(u, TEN))).to_bits();
        v0 = fixdfsi(r);
    }
    m.r8(v0.wrapping_add(DIGITS)) as u32
}

/// FUN_0021e1b0: printf %e/%f/%g float conversion of double (a3 lo, a3_hi) into buf a0 with spec a2; returns length.
pub fn ee_sub_21e1b0<M: QMem + ?Sized>(m: &mut M, a0: u32, a2: u32, a3: u32, a3_hi: u32) -> u32 {
    let x = ((a3_hi as u64) << 32) | a3 as u64;
    let s6 = a0;
    let s4 = a2;
    if x & 0x7ff0_0000_0000_0000 == 0x7ff0_0000_0000_0000 {
        let s = if x & 0x000f_ffff_ffff_ffff != 0 { 0x002a_94f0 } else { 0x002a_94f8 };
        m.call(0x0021_ed58, &[s6, s], &[]);
        return m.call(0x0021_ed28, &[s6], &[]).0;
    }
    let mut s1 = s6;
    let mut s3 = exp10(m, x);
    let c = m.r8(s4.wrapping_add(0x2c)) as u32;
    let mut s7 = c == 0x65;
    if c == 0x67 {
        let w = m.r32(s4.wrapping_add(0x1c));
        if (s3 as i32) < -4 || !((s3 as i32) < (w as i32)) {
            s7 = true;
            m.w32(s4.wrapping_add(0x1c), w.wrapping_sub(1));
        }
    }
    let prec = m.r32(s4.wrapping_add(0x1c));
    let a = if s7 { s3 } else { 0 };
    let p = pow(m, TEN, floatsidf(a.wrapping_sub(prec)));
    let h = mul(p, HALF);
    let s5 = (fd(x) + fd(h)).to_bits();
    let e0 = exp10(m, s5);
    let e1 = exp10(m, s5);
    if (s3 as i32) < (e1 as i32) {
        s3 = e0;
    }
    let (mut s0, mut s2);
    if s7 {
        s0 = 1u32;
        s2 = s3;
    } else if (s3 as i32) >= 0 {
        s2 = s3;
        s0 = s3.wrapping_add(1);
    } else {
        s0 = 1;
        s2 = 0;
    }
    while (s0 as i32) > 0 {
        s0 = s0.wrapping_sub(1);
        let d = ee_sub_21e0d8(m, lo(s5), s3, s2, hi(s5));
        s2 = s2.wrapping_sub(1);
        m.w8(s1, d as u8);
        s1 = s1.wrapping_add(1);
    }
    if m.r32(s4.wrapping_add(0x14)) == 0 {
        m.w8(s1, 0x2e);
        s1 = s1.wrapping_add(1);
        let mut n = m.r32(s4.wrapping_add(0x1c));
        while (n as i32) > 0 {
            n = n.wrapping_sub(1);
            let d = ee_sub_21e0d8(m, lo(s5), s3, s2, hi(s5));
            s2 = s2.wrapping_sub(1);
            m.w8(s1, d as u8);
            s1 = s1.wrapping_add(1);
        }
        if m.r8(s4.wrapping_add(0x2c)) == 0x67 && s6 < s1 {
            let mut ok = true;
            while m.r8(s1.wrapping_sub(1)) == 0x30 {
                s1 = s1.wrapping_sub(1);
                if !(s6 < s1) {
                    ok = false;
                    break;
                }
            }
            if ok && s6 < s1 && m.r8(s1.wrapping_sub(1)) == 0x2e {
                s1 = s1.wrapping_sub(1);
            }
        }
    }
    if s7 {
        m.w8(s1, 0x65);
        s1 = s1.wrapping_add(1);
        let sign = if (s3 as i32) >= 0 {
            0x2b
        } else {
            s3 = s3.wrapping_neg();
            0x2d
        };
        m.w8(s1, sign);
        s1 = s1.wrapping_add(1);
        let a2 = fixdfsi(pow(m, TEN, TWO));
        let mut a1 = 2i32;
        loop {
            let (q, r) = if a2 == 0 {
                (0, 0)
            } else {
                ((s3 as i32).wrapping_div(a2 as i32) as u32, (s3 as i32).wrapping_rem(a2 as i32) as u32)
            };
            a1 -= 1;
            let ch = m.r8(q.wrapping_add(DIGITS));
            s3 = r.wrapping_mul(10);
            m.w8(s1, ch);
            s1 = s1.wrapping_add(1);
            if a1 < 0 {
                break;
            }
        }
    }
    s1.wrapping_sub(s6)
}
