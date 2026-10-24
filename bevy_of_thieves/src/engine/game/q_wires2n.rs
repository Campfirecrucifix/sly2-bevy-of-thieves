//! WIRE-S2: 1:1 QMem ports of live-frontier natives (shard 3/4).
#![allow(unused)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

const GS: u32 = 0x003d_d490;

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

/// FUN_0017f5b0: once (+0x18c == 0 -> 1): FUN_00149a50(GS, a0, *(u64*)0x2a5630, a1).
pub fn ee_sub_17f5b0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if m.r32(a0 + 0x18c) == 0 {
        let lo = m.r32(0x002a_5630);
        let hi = m.r32(0x002a_5634);
        m.w32(a0 + 0x18c, 1);
        m.call(0x0014_9a50, &[GS, a0, lo, a1, hi], &[]);
    }
}

/// FUN_001d4050: FUN_001f4fb0(a0, a1, a2); s2 = *(a1 + (1-a2)*0x40 + 8); dir = a1+0x80 (negated if a2);
/// FUN_00132b10(s2,0x4d) || dir.y < 0.7 -> (+0x94,+0x98) = (0,0.1) else (0.5,0.1);
/// FUN_00132b10(s2,0x6c) -> FUN_00149ab8(GS, a0, *(u64*)0x2a7840, s2).
pub fn ee_sub_1d4050<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    m.call(0x001f_4fb0, &[a0, a1, a2], &[]);
    let idx = 1u32.wrapping_sub(a2) << 6;
    let s2 = m.r32(a1.wrapping_add(idx).wrapping_add(8));
    let mut q = m.rv(a1 + 0x80);
    if a2 != 0 {
        for c in q.iter_mut() {
            *c *= -1.0;
        }
    }
    let tenth = f32::from_bits(0x3dcc_cccd);
    let hit = m.call(0x0013_2b10, &[s2, 0x4d], &[]).0 != 0 || q[1] < f32::from_bits(0x3f33_3333);
    if hit {
        m.w32(a1 + 0x94, 0);
        m.wf(a1 + 0x98, tenth);
    } else {
        m.wf(a1 + 0x94, 0.5);
        m.wf(a1 + 0x98, tenth);
    }
    if m.call(0x0013_2b10, &[s2, 0x6c], &[]).0 != 0 {
        let lo = m.r32(0x002a_7840);
        let hi = m.r32(0x002a_7844);
        m.call(0x0014_9ab8, &[GS, a0, lo, s2, hi], &[]);
    }
}

// ---- FUN_00245b58 (powf wrapper, newlib w_powf) helpers ----
// soft-float callees computed natively: 0x23e290 __extendsfdf2, 0x23dfa0 __truncdfsf2, 0x23d7a0 __subdf3,
// 0x23d808 __muldf3, 0x23dd30 __cmpdf2; 0x249ce0 rint (pure math, round-half-even).
const LIB_VERSION: u32 = 0x002a_c138; // _LIB_VERSION (-1 _IEEE_, 0 _SVID_, 2 _POSIX_)

fn wd<M: QMem + ?Sized>(m: &mut M, a: u32, d: f64) {
    let b = d.to_bits();
    m.w32(a, b as u32);
    m.w32(a + 4, (b >> 32) as u32);
}
fn rd<M: QMem + ?Sized>(m: &mut M, a: u32) -> f64 {
    f64::from_bits(m.r64(a))
}
fn isnan<M: QMem + ?Sized>(m: &mut M, x: f32) -> bool {
    m.call(0x0024_a0c0, &[], &[f4(x)]).0 != 0
}
fn finite<M: QMem + ?Sized>(m: &mut M, x: f32) -> bool {
    m.call(0x0024_a098, &[], &[f4(x)]).0 != 0
}
/// exc header: type, name "powf" (0x2ab918), err = 0, arg1 = x, arg2 = y
fn exc_init<M: QMem + ?Sized>(m: &mut M, e: u32, ty: u32, x: f32, y: f32) {
    m.w32(e, ty);
    m.w32(e + 4, 0x002a_b918);
    m.w32(e + 0x20, 0);
    wd(m, e + 8, x as f64);
    wd(m, e + 0x10, y as f64);
}
/// 0x245fdc..: lib != 2 && matherr(e) -> skip errno set; else *__errno() = code; then err != 0 -> errno = err
fn exc_finish<M: QMem + ?Sized>(m: &mut M, e: u32, code: u32) -> f32 {
    let lib = m.r32(LIB_VERSION);
    let handled = lib != 2 && m.call(0x0024_9cb8, &[e], &[]).0 != 0;
    if !handled {
        let p = m.call(0x0024_01c8, &[], &[]).0;
        m.w32(p, code);
    }
    exc_tail(m, e)
}
/// 0x246004..: err != 0 -> *__errno() = err; return (float)retval
fn exc_tail<M: QMem + ?Sized>(m: &mut M, e: u32) -> f32 {
    let err = m.r32(e + 0x20);
    if err != 0 {
        let p = m.call(0x0024_01c8, &[], &[]).0;
        let err = m.r32(e + 0x20);
        m.w32(p, err);
    }
    rd(m, e + 0x18) as f32
}

fn pow_body<M: QMem + ?Sized>(m: &mut M, e: u32, x: f32, y: f32, z: f32) -> f32 {
    let lib = m.r32(LIB_VERSION);
    if lib == 0xffff_ffff {
        return z;
    }
    if isnan(m, y) {
        return z;
    }
    if isnan(m, x) {
        if y != 0.0 {
            return z;
        }
        exc_init(m, e, 1, x, y);
        wd(m, e + 0x18, x as f64);
        if lib == 2 {
            wd(m, e + 0x18, 1.0);
            return exc_tail(m, e);
        }
        return exc_finish(m, e, 0x21);
    }
    if x == 0.0 {
        if y == 0.0 {
            exc_init(m, e, 1, x, y);
            wd(m, e + 0x18, 0.0);
            if lib != 0 {
                wd(m, e + 0x18, 1.0);
                return exc_tail(m, e);
            }
            return exc_finish(m, e, 0x21);
        }
        if !finite(m, y) || !(y < 0.0) {
            return z;
        }
        exc_init(m, e, 1, x, y);
        let r = if lib == 0 { 0.0 } else { 0.0 - rd(m, 0x002a_c130) };
        wd(m, e + 0x18, r);
        return exc_finish(m, e, 0x21);
    }
    if !finite(m, z) && finite(m, x) && finite(m, y) {
        if isnan(m, z) {
            exc_init(m, e, 1, x, y);
            let r = if lib == 0 { 0.0 } else { rd(m, 0x002a_b920) };
            wd(m, e + 0x18, r);
            return exc_finish(m, e, 0x21);
        }
        // overflow
        exc_init(m, e, 3, x, y);
        let (big, neg) = if lib == 0 {
            (rd(m, 0x002a_b928), None)
        } else {
            (rd(m, 0x002a_c130), Some(()))
        };
        let negv = if lib == 0 { rd(m, 0x002a_b930) } else { 0.0 - big };
        let _ = neg;
        wd(m, e + 0x18, big);
        let h = ((y as f64) * 0.5) as f32;
        if (x as f64) < 0.0 {
            let hd = h as f64;
            if hd.round_ties_even() != hd {
                wd(m, e + 0x18, negv);
            }
        }
        return exc_finish(m, e, 0x22);
    }
    if z != 0.0 || !finite(m, x) || !finite(m, y) {
        return z;
    }
    // underflow
    exc_init(m, e, 4, x, y);
    wd(m, e + 0x18, 0.0);
    exc_finish(m, e, 0x22)
}

/// FUN_00245b58: powf(x, y) wrapper: z = FUN_00248038(x, y) (__ieee754_powf) plus SVID/POSIX error
/// handling via exception struct, matherr (FUN_00249cb8) and errno (FUN_002401c8).
pub fn ee_sub_245b58<M: QMem + ?Sized>(m: &mut M, x: f32, y: f32) -> f32 {
    let z = m.call(0x0024_8038, &[], &[f4(x), f4(y)]).1[0];
    let e = stack_push(m, 0x28);
    let r = pow_body(m, e, x, y, z);
    stack_pop(m, 0x28);
    r
}
