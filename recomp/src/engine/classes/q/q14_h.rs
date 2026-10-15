//! WP-Q14 part H: the small functions of the later file runs (static-init wrappers, accessors, state setters).
//! 1:1 from ghidra_functions(_v2).body on `QMem`.
#![allow(unused_variables, unused_mut, dead_code)]
use crate::engine::classes::q::q2mem::QMem;
use crate::engine::classes::q::q5util::pos;
use crate::engine::classes::q::q6::{vcall, CLOCK};

fn rr<M: QMem + ?Sized>(m: &mut M, a: u32, off: u32) -> u32 {
    let p = m.r32(a);
    m.r32(p + off)
}

macro_rules! init_wrap {
    ($($name:ident, $target:expr);* $(;)?) => {$(
        pub fn $name<M: QMem + ?Sized>(m: &mut M) { m.call($target, &[1, 0xffff], &[]); }
    )*};
}
init_wrap! {
    ee_sub_1cd5d0, 0x001c_d498; ee_sub_1ffcd8, 0x001f_fba0; ee_sub_205e50, 0x0020_5d18;
    ee_sub_223f50, 0x0022_3e18; ee_sub_2259b8, 0x0022_5880; ee_sub_226530, 0x0022_63f8;
    ee_sub_229ed0, 0x0022_9d98; ee_sub_22b020, 0x0022_aee8;
}

/// FUN_0017b750: state change; leaving state 0 clears +0xf60 on the owner, entering 0 plays brain action 0x9d
pub fn ee_sub_17b750<M: QMem + ?Sized>(m: &mut M, o: u32, st: i32) {
    if st == m.r32(o + 0x30) as i32 {
        return;
    }
    m.call(0x001e_f0d8, &[o + 0x18], &[]);
    if m.r32(o + 0x30) == 0 {
        let p = m.r32(o + 0x10);
        m.w32(p + 0xf60, 0);
    }
    m.w32(o + 0x30, st as u32);
    let t = m.r32(CLOCK);
    m.w32(o + 0x34, t);
    if st == 0 {
        let p = m.r32(o + 0x10);
        let b = m.call(0x0020_5438, &[p, 0x9d], &[]).0;
        m.w32(b + 0x3c, 0x11);
        let t = m.r32(o + 0x38);
        m.w32(b + 0x38, t);
        vcall(m, b, 0xc, 0x44, &[], &[]);
        let p = m.r32(o + 0x10);
        m.w32(p + 0xf60, 1);
    }
}
/// FUN_001800a0: append (a, b) to the 4-entry list (count +0x230, 0x18-byte entries at +0x234)
pub fn ee_sub_1800a0<M: QMem + ?Sized>(m: &mut M, o: u32, a: u32, b: u32) {
    let n = m.r32(o + 0x230);
    if n < 4 {
        m.w32(o + 0x230, n + 1);
        let e = o.wrapping_add(n * 0x18).wrapping_add(0x234);
        m.w32(e + 4, b);
        m.w32(e, a);
    }
}
/// FUN_001800d8
pub fn ee_sub_1800d8<M: QMem + ?Sized>(m: &mut M, o: u32, i: u64, a: u32, b: u32) {
    if i < 5 {
        let p = o.wrapping_add((i as u32).wrapping_mul(0x10));
        m.w32(p + 0x1c8, a);
        m.w32(p + 0x1d4, b);
    }
}
/// FUN_00180100: health-bar style update (1 - hp/max clamped to 0..1)
pub fn ee_sub_180100<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let h = m.r32(o + 0x2a0);
    if h != 0 {
        let mut n = m.r32(o + 0x184) as i32;
        if n < 1 {
            n = 0;
        }
        let mut f = 1.0f32 - n as f32 / (m.r32(o + 0x180) as i32) as f32;
        if f < 0.0 {
            f = 0.0;
        } else if 1.0 < f {
            f = 1.0;
        }
        m.call(0x0021_2e40, &[h, 0x34f], &[crate::engine::classes::q::q5util::f4(f)]);
    }
}
/// FUN_00180478
pub fn ee_sub_180478<M: QMem + ?Sized>(m: &mut M, o: u32, v: i32) {
    if v != m.r32(o + 0x1c8) as i32 {
        m.w32(o + 0x1c8, v as u32);
        let e = m.r32(o + 0xec);
        if m.r32(e + 0x60) == o {
            m.call(0x0013_4320, &[0x003d_d490, o], &[]);
        }
    }
    let f = m.rf(o + 0x1c4);
    let mut r = m.rf(0x002a_5654);
    if f != 0.0 {
        r = m.rf(CLOCK) + f;
    }
    m.wf(o + 0x1c0, r);
}
/// FUN_001b7000 / FUN_001b7050 (mode 0 / 1)
fn b7_set<M: QMem + ?Sized>(m: &mut M, o: u32, t: u32, mode: u32) {
    if m.r32(o + 0x13ec) != 0 {
        m.w32(o + 0x13f0, t);
        m.w32(o + 0x144c, mode);
        m.call(0x001b_6078, &[o, 4], &[]);
        let k = rr(m, o + 0x13f0, 0x180);
        m.call(0x0020_46f0, &[o, k], &[]);
    }
}
pub fn ee_sub_1b7000<M: QMem + ?Sized>(m: &mut M, o: u32, t: u32) {
    b7_set(m, o, t, 0)
}
pub fn ee_sub_1b7050<M: QMem + ?Sized>(m: &mut M, o: u32, t: u32) {
    b7_set(m, o, t, 1)
}
/// FUN_001b7a78
pub fn ee_sub_1b7a78<M: QMem + ?Sized>(m: &mut M, o: u32, tgt: u32) {
    m.call(0x001b_6078, &[o, 7], &[]);
    if tgt != 0 {
        let p = pos(m, tgt);
        m.wv(o + 0x1420, p);
        let q = pos(m, o);
        let x = p[0] - q[0];
        m.call(0x0020_46b0, &[o, x.to_bits()], &[]);
    }
}
/// FUN_001ca8c0
pub fn ee_sub_1ca8c0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001c_aec8, &[o, 2], &[]);
}
/// FUN_001ca930: (p - base) / 0x50 + first (0x50-byte records, mul by the 1/80 magic)
pub fn ee_sub_1ca930<M: QMem + ?Sized>(m: &mut M, o: u32, p: i32) -> i32 {
    let t = m.r32(o + 0xe0);
    let base = m.r32(t + 0x68) as i32;
    (p.wrapping_sub(base).wrapping_mul(-0x3333_3333) >> 4).wrapping_add(m.r32(t + 0xc) as i32)
}
/// FUN_001ca958
pub fn ee_sub_1ca958<M: QMem + ?Sized>(m: &mut M, o: u32, i: i32) -> i32 {
    let t = m.r32(o + 0xe0);
    (m.r32(t + 0x68) as i32).wrapping_add(i.wrapping_sub(m.r32(t + 0xc) as i32).wrapping_mul(0x50))
}
/// FUN_001d3ff0(o, e)
pub fn ee_sub_1d3ff0<M: QMem + ?Sized>(m: &mut M, o: u32, e: u32) {
    if m.r32(e + 0x150) != 2 {
        let p = o.wrapping_add(m.r32(e + 0x158).wrapping_mul(0xc)).wrapping_add(0x574);
        let a = (m.r32(p) as i32).wrapping_add(m.r32(e + 0x15c) as i32);
        let cap = m.r32(p + 4) as i32;
        m.w32(p, if a <= cap { a } else { cap } as u32);
        m.call(0x001d_4fa0, &[e, 2], &[]);
    }
}
/// FUN_00205e70: create the object-list once (DAT_003dd48c)
pub fn ee_sub_205e70<M: QMem + ?Sized>(m: &mut M) {
    if m.r32(0x003d_d48c) == 0 {
        let r = m.call(0x0013_e3c8, &[0x7d, 0, 0x656, 0xffff_ffff], &[]).0;
        m.w32(0x003d_d48c, r);
    }
}
/// FUN_0021fb18
pub fn ee_sub_21fb18<M: QMem + ?Sized>(m: &mut M, a: u32, b: u32) -> u32 {
    let t = m.r32(b + 4);
    if m.call(0x0013_2b10, &[t, 0x59], &[]).0 == 0 {
        m.call(0x0020_f948, &[a, b], &[]).0
    } else {
        1
    }
}

// ---------------------------------------------------------------- run_0017fd08 / run_0018b348
use crate::engine::classes::q::q2mem::{stack_pop, stack_push};
use crate::engine::classes::q::q5util::f4;

/// FUN_0017fd08(o, new): mode change (+0x1c0); mode 3 sets up the two links (+0x298 / +0x29c), mode 4 starts the
/// follow-world activity, -1 tears the emitter down
pub fn ee_sub_17fd08<M: QMem + ?Sized>(m: &mut M, o: u32, new: i32) {
    let old = m.r32(o + 0x1c0) as i32;
    if new == old {
        return;
    }
    let mut skip_set = false;
    if old == 1 {
        let n = m.r32(o + 0x230) as i32;
        let mut i = 0;
        while i < n {
            let e = m.r32(o + 0x248 + 24 * i as u32);
            if e != 0 {
                m.call(0x001f_0660, &[e, 0x66], &[]);
            }
            i += 1;
        }
    } else if old < 2 {
        if old == -1 {
            let h = m.r32(o + 0x2a0);
            if h != 0 {
                vcall(m, h, 0xc, 0x18c, &[u32::MAX], &[]);
                let h2 = m.r32(o + 0x2a0);
                m.call(0x0021_3b00, &[h2, 0x34f], &[]);
                ee_sub_180100(m, o);
                if m.r32(o + 0x2a8) as i32 != -1 {
                    let u = m.call(0x001f_bb90, &[], &[]).0;
                    m.call(0x001f_b2f8, &[u, 0], &[]);
                    m.call(0x0025_8040, &[o + 0x2a4, u], &[]);
                    let e = m.r32(o + 0x2a4);
                    m.call(0x001f_b018, &[e], &[]);
                }
            }
        }
        m.w32(o + 0x1c0, new as u32);
        skip_set = true;
    } else if old != 3 {
        m.w32(o + 0x1c0, new as u32);
        skip_set = true;
    } else {
        m.call(0x0012_e460, &[o + 0x298], &[]);
        m.call(0x0012_e460, &[o + 0x29c], &[]);
        let a = m.r32(o + 0x58);
        m.call(0x0023_03a0, &[o, a, 2], &[]);
        let a = m.r32(o + 0x5c);
        m.call(0x0023_03a0, &[o, a, 2], &[]);
        m.call(0x001a_58d0, &[o, 0, 0xf], &[]);
        m.call(0x001f_5720, &[o, 5], &[]);
        m.call(0x001f_57c0, &[o, 4], &[]);
        m.call(0x001f_68a8, &[o, 0], &[]);
        m.w32(o + 0x294, 0);
    }
    if !skip_set {
        m.w32(o + 0x1c0, new as u32);
    }
    let t = m.r32(CLOCK);
    m.w32(o + 0x1c4, t);
    if new == 3 {
        let sp = stack_push(m, 0x80);
        let (e0, a0) = (sp, sp + 0x40);
        let l = m.r32(o + 0x294);
        let u = vcall(m, l, 0xc, 0x104, &[e0], &[]).0;
        let (x, y) = (rr(m, o + 0x54, 0xa0), rr(m, o + 0x58, 0xa0));
        m.call(0x0022_ff40, &[a0, x, y, e0], &[]);
        for i in 0..4 {
            let v = m.rv(a0 + 16 * i);
            m.wv(e0 + 16 * i, v);
        }
        let l = m.r32(o + 0x294);
        let k = m.r32(l + 0x10);
        vcall(m, o, 0xc, 0x54, &[k, u, 3], &[]);
        let q = m.rv(0x002a_2dd0);
        vcall(m, o, 0xc, 0x11c, &[], &[q]);
        vcall(m, o, 0xc, 0x124, &[], &[q]);
        let a = m.r32(o + 0x58);
        let r = m.call(0x0012_dc18, &[o, a, 1, 7, 2], &[]).0;
        m.w32(o + 0x298, r);
        let v = m.call(0x0011_bff0, &[e0], &[]).1;
        let a = m.r32(o + 0x5c);
        let r = m.call(0x0012_dc18, &[o, a, 1, 7, 2], &[v]).0;
        m.w32(o + 0x29c, r);
        m.call(0x001f_5720, &[o, 1], &[]);
        m.call(0x001f_57c0, &[o, 1], &[]);
        m.call(0x001f_68a8, &[o, 1], &[]);
        stack_pop(m, 0x80);
    } else if new < 4 {
        if new == -1 {
            let h = m.r32(o + 0x2a0);
            if h != 0 {
                vcall(m, h, 0xc, 0x194, &[], &[]);
            }
            if m.r32(o + 0x2a4) != 0 {
                m.call(0x001f_b098, &[0], &[]);
                m.call(0x0025_8040, &[o + 0x2a4, 0], &[]);
            }
        }
    } else if new == 4 {
        m.call(0x0016_19a0, &[0x002d_eb30, 0x10, o], &[]);
    }
    let arg = if new == -1 { 0 } else { o.wrapping_add((new as u32).wrapping_mul(0x10)).wrapping_add(0x1c8) };
    m.call(0x0017_f4c0, &[o, arg], &[]);
}

/// FUN_0018b348: cache the three joint transforms (+0x1be0 / +0x1bf0 / +0x1c00 translations), the length-vector
/// FUN_0011bff0 of the third (+0x1c10) and the inverse-bind-pose transform of the first (+0x1c20)
pub fn ee_sub_18b348<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let t = m.r32(o + 0x1bc8);
    if t == 0 {
        return;
    }
    let sp = stack_push(m, 0x100);
    let (b1, b2, b3, inv) = (sp, sp + 0x40, sp + 0x80, sp + 0xc0);
    let u = m.call(0x0012_f990, &[t, 0x167], &[]).0;
    m.w32(o + 0x1bd8, u);
    let k = rr(m, o + 0x19a4, 0x60);
    let n = m.r32(o + 0x60);
    m.call(0x0013_00b8, &[u, 0, t, o, k, n, b1, 0], &[]);
    let v = m.rv(b1 + 0x30);
    m.wv(o + 0x1be0, v);
    let (u, k, n) = (m.r32(o + 0x1bd8), m.r32(o + 0x19cc), m.r32(o + 0x60));
    m.call(0x0013_00b8, &[u, 0, t, o, k, n, b2, 0], &[]);
    let v = m.rv(b2 + 0x30);
    m.wv(o + 0x1bf0, v);
    let (u, k, n) = (m.r32(o + 0x1bd8), m.r32(o + 0x19c0), m.r32(o + 0x60));
    m.call(0x0013_00b8, &[u, 0, t, o, k, n, b3, 0], &[]);
    let v = m.rv(b3 + 0x30);
    m.wv(o + 0x1c00, v);
    let l = m.call(0x0011_bff0, &[b3], &[]).1;
    m.wv(o + 0x1c10, l);
    m.call(0x0011_c318, &[inv, b3], &[]);
    let (r0, r1, r2, r3) = (m.rv(inv), m.rv(inv + 0x10), m.rv(inv + 0x20), m.rv(inv + 0x30));
    let v = m.rv(o + 0x1be0);
    let mut out = [0f32; 4];
    for i in 0..4 {
        out[i] = r0[i] * v[0] + r1[i] * v[1] + r2[i] * v[2] + r3[i];
    }
    m.wv(o + 0x1c20, out);
    stack_pop(m, 0x100);
}
