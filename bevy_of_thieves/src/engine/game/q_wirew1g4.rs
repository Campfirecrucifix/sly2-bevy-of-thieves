//! WIRE-W1G4: 1:1 queue ports (against `QMem`) of a batch of live-frontier natives, ported from the EE asm;
//! callees go through `m.call` / `vslot`. QMem convention: integer / pointer args in `a`, quadword and float
//! args in `v`.
#![allow(unused_variables, unused_mut, unused_assignments, clippy::all)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

const GS: u32 = 0x003d_d490; // game state / world

/// FUN_001ab040(p): FUN_001ab088(p) -> FUN_002348b0(p[1], 1); FUN_00234a60(p[0]); FUN_00234a40(p[0]).
pub fn ee_sub_1ab040<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    if m.call(0x001a_b088, &[a0], &[]).0 != 0 {
        let x = m.r32(a0 + 4);
        m.call(0x0023_48b0, &[x, 1], &[]);
    }
    let x = m.r32(a0);
    m.call(0x0023_4a60, &[x], &[]);
    let x = m.r32(a0);
    m.call(0x0023_4a40, &[x], &[]);
}

/// FUN_002342d0: case-insensitive strcmp via FUN_002345b8 (tolower).
pub fn ee_sub_2342d0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> i32 {
    let (mut s1, mut s2) = (a0, a1);
    loop {
        let c = m.r8(s1) as i8 as i32 as u32;
        if c == 0 {
            break;
        }
        let x = m.call(0x0023_45b8, &[c], &[]).0;
        let d = m.r8(s2) as i8 as i32 as u32;
        let y = m.call(0x0023_45b8, &[d], &[]).0;
        if x != y {
            break;
        }
        s1 = s1.wrapping_add(1);
        s2 = s2.wrapping_add(1);
    }
    let c = m.r8(s1) as u32;
    let x = m.call(0x0023_45b8, &[c], &[]).0;
    let d = m.r8(s2) as u32;
    let y = m.call(0x0023_45b8, &[d], &[]).0;
    x.wrapping_sub(y) as i32
}

/// FUN_0015a1c0(o): +0xe0 = 0; +0x150 -> FUN_001394c8; +0x14c -> FUN_001cde70; +0x130 = FUN_0011d448(+0xf8, +0xfc)
/// (x60 when +0xf0 == 3).
pub fn ee_sub_15a1c0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.w32(o + 0xe0, 0);
    let p = m.r32(o + 0x150);
    if p != 0 {
        let e = m.r32(o + 0xe4);
        m.call(0x0013_94c8, &[p, e], &[]);
    }
    let p = m.r32(o + 0x14c);
    if p != 0 {
        let e = m.r32(o + 0xe4);
        m.call(0x001c_de70, &[p, e], &[]);
    }
    let k = m.r32(o + 0xf0);
    let (x, y) = (m.rf(o + 0xf8), m.rf(o + 0xfc));
    let mut f = m.call(0x0011_d448, &[], &[f4(x), f4(y)]).1[0];
    if k == 3 {
        f = f * 60.0;
    }
    m.wf(o + 0x130, f);
}

/// FUN_001fe310(o, id): find the music entry, stop/restart via vtable.
pub fn ee_sub_1fe310<M: QMem + ?Sized>(m: &mut M, o: u32, id: u32) {
    let s = stack_push(m, 0x10);
    m.call(0x001f_e478, &[s, o, id], &[]);
    let p = m.r32(s);
    if p != 0 {
        let skip = m.r32(p + 0x40) == 0 && m.r16(p + 0x52) != 0;
        if !skip {
            m.call(0x001f_b098, &[p, p], &[f4(0.0)]);
            let n = m.r32(o + 0x204) as i32;
            let a1 = if n > 0 { m.r32(o.wrapping_add((n as u32) << 2) + 0x204) } else { 0 };
            m.call(0x0025_9b48, &[s, a1], &[]);
            let p = m.r32(s);
            if p != 0 {
                vslot(m, p, 6, &[], &[]);
            }
        }
    }
    let p = m.r32(s);
    if p != 0 {
        vslot(m, p, 1, &[], &[]);
    }
    stack_pop(m, 0x10);
}

/// FUN_0011ad30(n, src, w, out, f12): build n-entry 0x28-stride channel set, evaluate, copy +0xc floats to out.
pub fn ee_sub_11ad30<M: QMem + ?Sized>(m: &mut M, n: u32, a1: u32, a2: u32, a3: u32, f12: f32) {
    let s = stack_push(m, 0x80);
    m.call(0x001a_e478, &[s], &[]);
    m.call(0x0011_a550, &[s + 0x10, n], &[]);
    let ni = n as i32;
    if ni > 0 {
        let (mut s2, mut s4) = (a1, a2);
        let st = n << 2;
        for i in 0..n {
            let s0 = m.r32(s + 0x18).wrapping_add(i.wrapping_mul(0x28));
            m.wf(s0 + 8, f12);
            m.w32(s + 0x60, st);
            m.w32(s + 0x64, 0x10);
            let v = m.call(0x001a_e318, &[s + 0x60], &[]).0;
            m.w32(s0 + 0x18, v);
            m.call(0x001a_e5a0, &[v, s2, st], &[]);
            s2 = s2.wrapping_add(st);
            let f = m.rf(s4);
            s4 += 4;
            m.wf(s0 + 0x10, f);
            m.call(0x0011_a220, &[s + 0x40, s0], &[]);
        }
    }
    m.w32(s + 0x14, n);
    m.call(0x0011_abf8, &[s + 0x10], &[]);
    if ni > 0 {
        let b = m.r32(s + 0x18) + 0xc;
        for i in 0..n {
            let f = m.rf(b + i.wrapping_mul(0x28));
            m.wf(a3 + i.wrapping_mul(4), f);
        }
    }
    m.call(0x001a_e4a0, &[s, 2], &[]);
    stack_pop(m, 0x80);
}

/// FUN_001fe5f0(o, id, quiet): select music/ambient set; returns +0x200.
pub fn ee_sub_1fe5f0<M: QMem + ?Sized>(m: &mut M, o: u32, a1: u32, a2: u32) -> u32 {
    m.call(0x001f_e738, &[o, a1, a2], &[]);
    let s0 = m.call(0x001f_cfa0, &[o, a1], &[]).0;
    if s0 == 0 {
        return 0;
    }
    let v = m.call(0x001f_c138, &[s0, 0], &[]).0;
    m.call(0x0025_9b48, &[o + 0x200, v], &[]);
    m.w32(o + 0x244, 0);
    m.w32(o + 0x248, 0);
    m.w32(o + 0x24c, 0);
    if a2 == 0 {
        let k = m.r32(s0 + 0x48);
        m.call(0x0022_7728, &[0x0044_35f0, a1, k], &[]);
    }
    let k = m.r32(s0 + 0x48);
    if k != 0xffff_ffff && m.r32(s0 + 0x54) != 0 {
        let s3 = m.call(0x0015_ed78, &[GS, 5, k, 0], &[]).0;
        if s3 != 0 {
            for i in 0..6u32 {
                let p = o + 0x22c + 4 * i;
                if i == 3 {
                    m.w32(o + 0x238, s3);
                } else {
                    let v = m.call(0x001f_d0c0, &[o, i], &[]).0;
                    let v = m.call(0x001f_67e0, &[s3, v], &[]).0;
                    m.w32(p, v);
                }
                let q = m.r32(p);
                if q != 0 {
                    vslot(m, q, 0, &[], &[]);
                }
            }
        }
    }
    m.r32(o + 0x200)
}

fn release<M: QMem + ?Sized>(m: &mut M, s0: u32, a0: u32) {
    let c = m.r32(s0).wrapping_sub(1);
    m.w32(s0, c);
    if c == 0 {
        m.call(0x0025_6d40, &[a0, 2], &[]);
        m.call(0x0012_9ec8, &[s0 + 8], &[]);
        m.call(0x0025_50f8, &[s0], &[]);
        m.call(0x0011_cd98, &[0x0045_20a8, s0], &[]);
    }
}

/// FUN_00200da0(ret, n, ids, names): build a string list of n formatted entries into `ret`.
pub fn ee_sub_200da0<M: QMem + ?Sized>(m: &mut M, a0: u32, n: u32, a2: u32, a3: u32) -> u32 {
    if n == 0 {
        m.call(0x0012_a0b0, &[a0, 0], &[]);
        return a0;
    }
    let s = stack_push(m, 0x60);
    m.w32(s + 0x30, 0);
    m.call(0x0023_33b4, &[s, 0, 0x28], &[]);
    m.w32(s + 4, 2);
    let mut s6 = s + 0x30;
    if (n as i32) > 0 {
        let (mut s5, mut s4) = (a2, a3);
        for _ in 0..n {
            let v = m.r32(s5);
            let s1 = m.r32(s4);
            m.w32(s, v);
            let v = m.call(0x0025_6b48, &[], &[]).0;
            m.call(0x0025_6c38, &[s + 0x40, v], &[]);
            let s0 = m.r32(s + 0x40) + 8;
            m.call(0x001f_fd98, &[s + 0x50, s1, s], &[]);
            m.call(0x0012_9658, &[s0, s + 0x50], &[]);
            m.call(0x0012_9ec8, &[s + 0x50], &[]);
            let x = m.r32(s + 0x40);
            m.call(0x0025_6c58, &[s6, x], &[]);
            let s0 = m.r32(s + 0x40);
            s6 = s0.wrapping_add(0x10);
            if s0 != 0 {
                release(m, s0, s6);
            }
            s5 += 4;
            s4 += 4;
        }
    }
    let x = m.r32(s + 0x30);
    m.call(0x0012_9480, &[a0, x], &[]);
    let s0 = m.r32(s + 0x30);
    if s0 != 0 {
        release(m, s0, s0 + 0x10);
    }
    stack_pop(m, 0x60);
    a0
}

/// FUN_001d6848(from, to, axis): ballistic launch velocity reaching `to` from `from` (VU0 math inline).
pub fn ee_sub_1d6848<M: QMem + ?Sized>(m: &mut M, a0: V4, a1: V4, a2: V4) -> V4 {
    let mut f21 = m.call(0x0024_5000, &[], &[]).1[0];
    let mut d = [a1[0] - a0[0], a1[1] - a0[1], a1[2] - a0[2], a1[3] - a0[3]];
    let f24 = d[2];
    d[2] = 0.0;
    let f22 = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    let mut f12 = m.call(0x0024_58f0, &[], &[f4(f24), f4(f22)]).1[0];
    if f12 < 0.0 {
        f12 = 0.0;
    }
    let f0 = (f32::from_bits(0x3fc9_0fdb) - f12) * f21;
    let c = m.rv(0x002a_2dc0);
    let mut f20 = a2[0] * c[0] + a2[1] * c[1] + a2[2] * c[2];
    f21 = f12 + f0;
    let t = m.call(0x0024_50f0, &[], &[f4(f21)]).1[0];
    let den = f24 - f22 * t;
    f20 = f20 / (den + den);
    if f20 <= 0.0 {
        return m.rv(0x002a_2dd0);
    }
    let f0 = f20.sqrt();
    let cs = m.call(0x0024_4e10, &[], &[f4(f21)]).1[0];
    f20 = f22 * f0 / cs;
    let t = m.call(0x0024_50f0, &[], &[f4(f21)]).1[0];
    d[2] = f22 * t;
    let q = 1.0 / (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    [d[0] * q * f20, d[1] * q * f20, d[2] * q * f20, d[3] * q * f20]
}

const T: u32 = 0x002d_ded0; // g_clock t

/// FUN_001b6da8(o, state): grapple/attach state change (+0x13d8).
pub fn ee_sub_1b6da8<M: QMem + ?Sized>(m: &mut M, o: u32, a1: u32) {
    let v = m.r32(o + 0x13d8);
    if v == a1 {
        return;
    }
    if v == 0xffff_ffff {
        m.w32(o + 0x194, 0);
        m.w32(o + 0x198, 0);
        m.w32(o + 0x190, 0);
    }
    m.w32(o + 0x13d8, a1);
    let t = m.rf(T);
    m.wf(o + 0x13dc, t);
    match a1 as i32 {
        -1 => {
            let k = m.r32(o + 0x13e4);
            m.call(0x001a_7e10, &[GS, k], &[]);
            m.w32(o + 0x13e4, 0);
        }
        1 => {
            let v3 = vslot(m, o, 32, &[], &[]).1;
            let v2 = m.rv(0x002a_2dc0);
            let d = v3[0] * v2[0] + v3[1] * v2[1] + v3[2] * v2[2];
            let k = m.rf(0x002c_09a8);
            let mut r = [0.0f32; 4];
            for i in 0..4 {
                r[i] = (v3[i] - v2[i] * d) + v2[i] * k;
            }
            vslot(m, o, 34, &[], &[r]);
            let t = m.rf(T);
            m.wf(o + 0x131c, t);
        }
        2 => {
            m.w32(o + 0x194, 0);
            m.w32(o + 0x198, 0);
            m.w32(o + 0x190, 0);
            let mut s1 = m.r32(o + 0x13e4);
            let s = stack_push(m, 0x60);
            while s1 != 0 {
                let v0 = (m.r32(s1 + 8) != o) as u32;
                let s3 = s1 + ((1 - v0) << 6);
                let s4 = s1 + (v0 << 6);
                let k = m.r32(s3 + 0xc);
                let s0 = m.call(0x001f_7530, &[k, 4], &[]).0;
                if s0 != 0 {
                    m.call(0x0023_33b4, &[s, 0, 0x60], &[]);
                    m.w32(s, o);
                    let x = m.r32(s4);
                    m.w32(s + 8, s0);
                    m.w32(s + 4, x);
                    m.w32(s + 0xc, 4);
                    m.w32(s + 0x10, 0xffff_ffec);
                    let q = m.rv(s3 + 0x10);
                    m.wv(s + 0x20, q);
                    let q = m.rv(s4 + 0x30);
                    m.wv(s + 0x30, q);
                    m.call(0x001f_7598, &[s], &[]);
                }
                s1 = m.r32(s1 + 0xac);
            }
            stack_pop(m, 0x60);
            let k = m.r32(o + 0x13e4);
            m.call(0x001a_7e10, &[GS, k], &[]);
            m.w32(o + 0x13e4, 0);
            let p = m.r32(o + 0x1030);
            let p = m.r32(p + 0x14);
            let v = m.call(0x0012_f938, &[p, 0x1a5], &[]).0;
            let p = m.r32(o + 0x1030);
            if v != 0 {
                let f = m.rf(v);
                m.call(0x0013_0d30, &[p, 0], &[f4(f), f4(1.0)]);
            } else {
                m.call(0x0013_0b18, &[p, 0], &[]);
            }
        }
        _ => {}
    }
}

/// FUN_001f7598(e): register a 0x60-byte contact event against every matching receiver; returns the receiver.
pub fn ee_sub_1f7598<M: QMem + ?Sized>(m: &mut M, e: u32) -> u32 {
    let s = stack_push(m, 0x30);
    let (s2, s3, l30) = (s, s + 0x10, s + 0x20);
    let k = m.r32(e + 8);
    let v = m.call(0x001a_5bf0, &[k], &[]).0;
    m.w32(l30, v);
    let mut ret = 0;
    'outer: while m.r32(l30) != 0 {
        let r = m.r32(l30);
        let mut a0 = m.r32(0x003e_15c0);
        while a0 != 0 {
            if m.r32(a0) == m.r32(e)
                && m.r32(a0 + 4) == m.r32(e + 4)
                && m.r32(a0 + 0xc) == m.r32(e + 0xc)
                && m.r32(a0 + 0x48) == r
            {
                break 'outer;
            }
            a0 = m.r32(a0 + 0x50);
        }
        let k = m.r32(e + 0xc);
        let v = m.call(0x001f_78d8, &[r, k], &[]).0;
        m.w32(e + 0x44, v);
        if vslot(m, r, 74, &[e], &[]).0 == 0 {
            let n = m.r32(r + 0x10);
            m.w32(l30, n);
            continue;
        }
        m.w32(e + 0x48, r);
        let s0 = m.call(0x0011_ccc0, &[0x003e_15a8], &[]).0;
        for i in (0..0x60).step_by(16) {
            let q = m.rv(e + i);
            m.wv(s0 + i, q);
        }
        let t = m.rf(T);
        m.wf(s0 + 0x4c, t);
        m.call(0x0011_a210, &[s0 + 0x50], &[]);
        m.call(0x0011_a220, &[0x003e_15c0, s0], &[]);
        if m.r32(e + 0x44) & 0xf8 != 0 {
            let e0 = m.r32(e);
            m.w32(s2, r);
            m.w32(s2 + 4, e0);
            m.w32(s2 + 8, e);
            vslot(m, r, 3, &[0xb, s2], &[]);
            let e0 = m.r32(e);
            vslot(m, e0, 3, &[0xb, s2], &[]);
            m.w32(s3, l30);
            m.w32(s3 + 4, e + 0xc);
            let e0 = m.r32(e);
            vslot(m, e0, 2, &[0xa, 1, 2, s3, 0], &[]);
            m.w32(s3, e);
            let r = m.r32(l30);
            vslot(m, r, 2, &[0xb, 1, 2, s3, 0], &[]);
        }
        ret = m.r32(l30);
        break;
    }
    stack_pop(m, 0x30);
    ret
}

fn len3(v: V4) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// FUN_00179328(o): enter/reset state; face velocity or target, sync attachments, schedule timers.
pub fn ee_sub_179328<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0017_4950, &[o], &[]);
    let v = m.rv(o + 0x80);
    let mut cond = false;
    if f32::from_bits(0x38d1_b717) < len3(v) {
        cond = m.r32(o + 0x5c) != 0x40000;
    }
    let f = if cond {
        let (y, x) = (m.rf(o + 0x84), m.rf(o + 0x80));
        m.call(0x0024_58f0, &[], &[f4(-y), f4(-x)]).1[0]
    } else {
        let k = m.r32(o + 0x10);
        let p = m.call(0x001a_6318, &[k], &[]).0;
        let k = m.r32(o + 0x10);
        let q = m.call(0x001a_6318, &[k], &[]).0;
        let (y, x) = (m.rf(p + 4), m.rf(q));
        m.call(0x0024_58f0, &[], &[f4(y), f4(x)]).1[0]
    };
    let k = m.r32(o + 0x10);
    m.call(0x0020_9818, &[k + 0xfd0], &[f4(f)]);
    let p = m.r32(o + 0x40);
    if p != 0 && m.r32(p) == 2 {
        let k = m.r32(o + 0x10);
        let a = m.rf(k + 0x1a8) + f32::from_bits(0x4049_0fdb);
        let f = m.call(0x0011_cf60, &[], &[f4(a)]).1[0];
        let p = m.r32(o + 0x40);
        let q = m.r32(p + 4);
        let g = m.rf(q + 0x1c);
        let r = m.call(0x0011_e1a0, &[], &[f4(f), f4(g), f4(0.0)]).1;
        let k = m.r32(o + 0x10);
        let w = m.call(0x001a_62e8, &[k], &[]).1;
        let p = m.r32(o + 0x40);
        let x = m.r32(p + 8);
        let s = [w[0] + r[0], w[1] + r[1], w[2] + r[2], w[3] + r[3]];
        m.call(0x0020_c6f8, &[x], &[s]);
    }
    let k = m.r32(o + 0x10);
    let z = m.rv(0x002a_2dd0);
    m.call(0x0020_ab68, &[k, 0, 0], &[z]);
    if m.r32(o + 0x30) == 0 {
        m.call(0x001e_efc0, &[o], &[]);
    }
    let k = m.r32(o + 0x10);
    let w = m.call(0x001a_62e8, &[k], &[]).1;
    let k = m.r32(o + 0x10);
    m.call(0x0015_6c80, &[GS, 1, k, k + 0x35c], &[w]);
    let f = m.rf(0x002a_5584);
    let k = m.r32(o + 0x10);
    m.wf(o + 0xb0, f);
    m.wf(o + 0xb4, f);
    m.w32(k + 0xf04, 0);
    let k = m.r32(o + 0x10);
    if m.r32(k + 0x334) == 0 {
        return;
    }
    let s0 = m.call(0x001c_4840, &[GS], &[]).0;
    if s0 != 0 {
        let k = m.r32(o + 0x50);
        if m.call(0x001a_6af8, &[k, s0], &[]).0 != 0 {
            if s0 == m.r32(0x002f_7900) || m.r32(o + 0x5c) & 0x40000 != 0 {
                m.call(0x0018_7e28, &[0x002d_fc00, 3], &[f4(f32::from_bits(0x3e19_999a))]);
                let t = m.rf(T) + f32::from_bits(0x3d4c_ccce);
                m.wf(o + 0xb0, t);
            }
        }
    }
    let t = m.rf(T) + f32::from_bits(0x3e08_8889);
    m.wf(o + 0xb4, t);
}

const CM: u32 = 0x002d_df40; // camera manager

fn cp8<M: QMem + ?Sized>(m: &mut M, d: u32, s: u32) {
    let a = m.r32(s);
    let b = m.r32(s + 4);
    m.w32(d, a);
    m.w32(d + 4, b);
}

/// FUN_0014ff38(o): camera mode enter: resolve the two sound/gadget handles, init pitch/yaw window, FOV.
pub fn ee_sub_14ff38<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let (s3, s4) = (o + 0x10, o + 0x14);
    m.call(0x0014_fcc8, &[o], &[]);
    let s1 = m.call(0x001f_bb90, &[0x26], &[]).0;
    m.call(0x001f_b2f8, &[s1, 0], &[]);
    m.call(0x0025_8040, &[o + 0x34, s1], &[]);
    let s1 = m.call(0x001f_bb90, &[0x25], &[]).0;
    m.call(0x001f_b2f8, &[s1, 0], &[]);
    m.call(0x0025_8040, &[o + 0x38, s1], &[]);
    let k = m.r32(o + 0x34);
    m.call(0x001f_b018, &[k], &[]);
    m.call(0x0011_bc88, &[CM + 0x60, s3, s4], &[]);
    let k = m.r32(o);
    if m.call(0x0013_2b10, &[k, 0x34], &[]).0 != 0 {
        cp8(m, o + 0x3c, 0x002b_d950);
        cp8(m, o + 0x44, 0x002b_d958);
    } else {
        cp8(m, o + 0x3c, 0x002b_d968);
        cp8(m, o + 0x44, 0x002b_d970);
        let f1 = m.rf(o + 0x14);
        let f0 = m.rf(o + 0x10);
        m.wf(o + 0x50, f1);
        m.wf(o + 0x4c, f0);
        let k = m.r32(o);
        let v = m.call(0x001a_6318, &[k], &[]).0;
        m.call(0x0011_bc88, &[v, s3, s4], &[]);
        let c = 0x002b_d960;
        let x = m.rf(o + 0x14);
        let f0 = m.call(0x0011_d948, &[c], &[f4(x)]).1[0];
        let f5 = m.rf(o + 0x10);
        let (a, b, cc, d) = (m.rf(o + 0x3c), m.rf(o + 0x40), m.rf(o + 0x44), m.rf(o + 0x48));
        m.wf(o + 0x14, f0);
        m.wf(o + 0x3c, a + f5);
        m.wf(o + 0x40, b + f5);
        m.wf(o + 0x44, cc + f0);
        m.wf(o + 0x48, d + f0);
        let g = m.call(0x0011_d948, &[c], &[f4(f0)]).1[0];
        m.wf(o + 0x44, g);
        let x = m.rf(o + 0x14);
        let g = m.call(0x0011_d948, &[c], &[f4(x)]).1[0];
        m.wf(o + 0x48, g);
    }
    m.w32(o + 0x18, 0);
    m.w32(o + 0x1c, 0);
    m.w32(o + 0x20, 0);
    m.w32(o + 0x24, 0);
    let f = m.rf(CM + 0x24) / m.rf(o + 0x28);
    m.wf(o + 0x2c, f);
    let f = m.rf(CM + 0x24);
    m.wf(o + 0x30, f);
    m.call(0x0014_f610, &[CM], &[f4(49.0)]);
    let n = m.r32(o + 0x54).wrapping_add(1);
    m.w32(o + 0x54, n);
    let t = m.rf(T);
    m.wf(0x003e_1be0, t);
}

/// FUN_0012c570(o, rd): load a keyframe track (times at +8, up to 3 optional float2 channels at +0xc..).
pub fn ee_sub_12c570<M: QMem + ?Sized>(m: &mut M, o: u32, rd: u32) {
    let s = stack_push(m, 0x10);
    m.call(0x0011_7eb0, &[rd, o + 0x20], &[]);
    let n = m.call(0x0011_7c20, &[rd], &[]).0;
    m.w32(s + 4, 0x10);
    m.w32(s, n << 2);
    m.w32(o + 4, n);
    let v = m.call(0x001a_e1d0, &[s], &[]).0;
    m.w32(o + 8, v);
    let mut i = 0i32;
    if (m.r32(o + 4) as i32) > 0 {
        loop {
            let x = m.call(0x0011_7c20, &[rd], &[]).0 as i32 as f32;
            let b = m.r32(o + 8);
            m.wf(((i as u32) << 2).wrapping_add(b), x * f32::from_bits(0x3c88_8889));
            i += 1;
            if i >= m.r32(o + 4) as i32 {
                break;
            }
        }
    }
    let mask = m.call(0x0011_7bd0, &[rd], &[]).0;
    for k in 0..3u32 {
        if (1u32 << k) & mask == 0 {
            continue;
        }
        let n = m.r32(o + 4);
        m.w32(s + 4, 0x10);
        m.w32(s, n << 3);
        let mut p = m.call(0x001a_e1d0, &[s], &[]).0;
        m.w32(o + 0xc + k.wrapping_mul(4), p);
        let mut i = 0i32;
        if (m.r32(o + 4) as i32) > 0 {
            loop {
                i += 1;
                let f = m.call(0x0011_7e38, &[rd], &[]).1[0];
                m.wf(p, f);
                let f = m.call(0x0011_7e38, &[rd], &[]).1[0];
                m.wf(p + 4, f);
                p += 8;
                if i >= m.r32(o + 4) as i32 {
                    break;
                }
            }
        }
    }
    stack_pop(m, 0x10);
}

/// FUN_0011d018(p, out_vel, x, target, dt): accelerate-limited approach of x toward target; returns new x.
pub fn ee_sub_11d018<M: QMem + ?Sized>(m: &mut M, p: u32, out: u32, x: f32, f21: f32, dt: f32) -> f32 {
    let mut f20 = dt;
    let f3 = m.rf(p);
    if f3 == 0.0 {
        if out != 0 {
            m.wf(out, 0.0);
        }
        return x;
    }
    let mut f14 = x - f21;
    let mut neg = false;
    if f14 < 0.0 {
        f14 = -f14;
        neg = true;
    }
    let p1 = m.rf(p + 4);
    let f4v = m.rf(p + 8);
    let f1 = (p1 + f3) * f4v * 0.5;
    let (f2, f12);
    let mut direct = false;
    let (mut d2, mut d12) = (0.0f32, 0.0f32);
    if f1 <= f14 {
        let f0 = (f14 - f1) / f3;
        if f20 <= f0 {
            d2 = -f3;
            d12 = f14 - f3 * f20;
            direct = true;
        } else {
            f20 = f20 - f0;
            f14 = f1;
        }
    }
    if direct {
        f2 = d2;
        f12 = d12;
    } else {
        let f0 = m.rf(p + 4);
        let a = (m.rf(p) - f0) / (f4v + f4v);
        let s = stack_push(m, 0x10);
        let n = m.call(0x0011_d508, &[s], &[f4(a), f4(f0), f4(-f14)]).0;
        let r = m.rf(s.wrapping_add(n.wrapping_sub(1) << 2));
        stack_pop(m, 0x10);
        let f14 = r - f20;
        if 0.0 < f14 {
            let q2 = m.rf(p + 4);
            let q0 = m.rf(p);
            let q1 = m.rf(p + 8);
            let g4 = q2 * f14;
            let g0 = (q0 - q2) * f14;
            let g1 = g0 / q1;
            let g0 = g0 * f14 / (q1 + q1);
            f2 = -(q2 + g1);
            f12 = g0 + g4;
        } else {
            let q = m.rf(p + 4);
            f2 = -q;
            f12 = 0.0;
        }
    }
    if neg {
        let r = f21 - f12;
        if out != 0 {
            m.wf(out, -f2);
        }
        r
    } else {
        let r = f12 + f21;
        if out != 0 {
            m.wf(out, f2);
        }
        r
    }
}

/// FUN_001c77e8(o, st): state change (+0x150); state 1 builds a "...%d" label and notifies, then forces st 2.
pub fn ee_sub_1c77e8<M: QMem + ?Sized>(m: &mut M, o: u32, a1: u32) {
    if m.r32(o + 0x150) == a1 {
        return;
    }
    let mut st = a1;
    if a1 == 0 {
        let (x, y) = (m.rf(o + 0x230), m.rf(o + 0x234));
        let f = m.call(0x0011_d448, &[], &[f4(x), f4(y)]).1[0];
        m.wf(o + 0x238, f);
    } else if a1 == 1 {
        let s = stack_push(m, 0x50);
        let v1 = m.r32(o + 0x244);
        m.w32(s, 0xffff_ffff);
        let (s3, s1, s4) = (s + 0x10, s + 0x20, s + 0x30);
        let k = m.r32(v1 + 4);
        m.call(0x0021_ecd8, &[s3, 0x002a_7768, k], &[]);
        let (s0, s2) = (0x002a_7770, 0x002a_7778);
        m.call(0x001d_8a38, &[s1, s0, s2, s3, 0], &[]);
        m.call(0x0012_9658, &[s, s1], &[]);
        m.call(0x0012_9ec8, &[s1], &[]);
        let n = m.r32(s + 4).wrapping_add(1);
        m.w32(s + 0x30, 0x15);
        m.w32(s + 0x34, n);
        m.call(0x001d_8b20, &[s1, s4, s0, s2, s3, 0], &[]);
        m.call(0x0012_9ec8, &[s1], &[]);
        m.call(0x0012_9ec8, &[s4], &[]);
        let a2 = m.r32(o + 0x210);
        if a2 != 0 {
            m.w32(s + 0x40, a1);
            m.w32(s + 0x44, o);
            vslot(m, a2, 50, &[s + 0x40], &[]);
        }
        vslot(m, o, 2, &[5, 0, 0, 0, 0], &[]);
        st = 2;
        m.call(0x0012_9ec8, &[s], &[]);
        stack_pop(m, 0x50);
    }
    m.call(0x001c_60b8, &[o, st], &[]);
}

fn lin4(a: V4, x: f32, b: V4, y: f32, c: V4, z: f32) -> V4 {
    let mut r = [0.0; 4];
    for i in 0..4 {
        r[i] = a[i] * x + b[i] * y + c[i] * z;
    }
    r
}

/// FUN_00117120(out_pos, out_vel, out_acc, p0, v0, p1, v1, dur, t, scale): cubic Hermite evaluation (VU0 inline).
pub fn ee_sub_117120<M: QMem + ?Sized>(
    m: &mut M,
    t0: u32,
    t1: u32,
    t2: u32,
    p0: V4,
    v0: V4,
    p1: V4,
    v1: V4,
    f12: f32,
    f13: f32,
    f14: f32,
) {
    let u = f13 / f12;
    let w = 1.0 - u;
    if t0 != 0 {
        let uww = u * w * w;
        let uu = u * u;
        let www = w * w * w;
        let uuw = uu * w;
        let a = www + uww * 3.0;
        let b = uww * f12;
        let c = -uuw * f12;
        let d = uu * u + uuw * 3.0;
        let mut r = [0.0; 4];
        for i in 0..4 {
            r[i] = p0[i] * a + v0[i] * b + v1[i] * c + p1[i] * d;
        }
        m.wv(t0, r);
    }
    if t1 != 0 {
        let d = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2], p1[3] - p0[3]];
        let a = u * (6.0 - u * 6.0) / f12;
        let b = 1.0 - u * (4.0 - u * 3.0);
        let c = u * (u * 3.0 - 2.0);
        let r = lin4(d, a, v0, b, v1, c);
        m.wv(t1, [r[0] * f14, r[1] * f14, r[2] * f14, r[3] * f14]);
    }
    if t2 != 0 {
        let d = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2], p1[3] - p0[3]];
        let k = f14 * f14 / f12;
        let a = (6.0 - u * 12.0) / f12;
        let b = u * 6.0 - 4.0;
        let c = u * 6.0 - 2.0;
        let r = lin4(d, a, v0, b, v1, c);
        m.wv(t2, [r[0] * k, r[1] * k, r[2] * k, r[3] * k]);
    }
}

fn alloc16<M: QMem + ?Sized>(m: &mut M, s: u32, n: u32) -> u32 {
    m.w32(s + 4, 0x10);
    m.w32(s, n);
    m.call(0x001a_e1d0, &[s], &[]).0
}

fn read_vecs<M: QMem + ?Sized>(m: &mut M, o: u32, rd: u32, base_off: u32) {
    let mut i = 0i32;
    if (m.r32(o + 0xe8) as i32) > 0 {
        loop {
            let b = m.r32(o + base_off);
            m.call(0x0011_7eb0, &[rd, b.wrapping_add((i as u32) << 4)], &[]);
            i += 1;
            if i >= m.r32(o + 0xe8) as i32 {
                break;
            }
        }
    }
}

/// FUN_001e5df8(o, rd): load a path/rail object (counts, index + point tables, optional tangents).
pub fn ee_sub_1e5df8<M: QMem + ?Sized>(m: &mut M, o: u32, rd: u32) {
    let s = stack_push(m, 0x10);
    m.call(0x0023_1768, &[o, rd], &[]);
    let v = m.call(0x0011_7d00, &[rd], &[]).0;
    m.w32(o + 0xe0, v);
    let v = m.call(0x0011_7d00, &[rd], &[]).0;
    m.w32(o + 0xe4, v);
    let n = m.call(0x0011_7c20, &[rd], &[]).0;
    m.w32(o + 0xe8, n);
    let v = alloc16(m, s, n << 2);
    m.w32(o + 0xec, v);
    let n = m.r32(o + 0xe8);
    let v = alloc16(m, s, n << 2);
    m.w32(o + 0xf0, v);
    let n = m.r32(o + 0xe8);
    let v = alloc16(m, s, n << 4);
    m.w32(o + 0xf4, v);
    let n = m.r32(o + 0xe8);
    let p = m.r32(o + 0xec);
    m.call(0x0011_7968, &[rd, n << 2, p], &[]);
    read_vecs(m, o, rd, 0xf4);
    if m.r32(o + 0xe0) != 0 {
        let n = m.r32(o + 0xe8);
        let v = alloc16(m, s, n << 4);
        m.w32(o + 0x140, v);
        let n = m.r32(o + 0xe8);
        let v = alloc16(m, s, n << 4);
        m.w32(o + 0x144, v);
        read_vecs(m, o, rd, 0x140);
        read_vecs(m, o, rd, 0x144);
    }
    m.call(0x0013_f810, &[o, rd], &[]);
    m.call(0x001e_6180, &[o], &[]);
    stack_pop(m, 0x10);
}

/// FUN_0017d450(o, st): state change (+0x30); spawns the per-state effect (jump table 0x2a5610).
pub fn ee_sub_17d450<M: QMem + ?Sized>(m: &mut M, o: u32, s1: u32) {
    let v1 = m.r32(o + 0x30);
    if s1 == v1 {
        return;
    }
    if !(s1 < 3 && (v1 as i32) >= 0 && (v1 as i32) < 3) {
        m.call(0x001e_f0d8, &[o + 0x18], &[]);
    }
    m.w32(o + 0x30, s1);
    let t = m.rf(T);
    m.wf(o + 0x34, t);
    let k = m.r32(o + 0x10);
    match s1.wrapping_add(1) {
        2 | 3 => {
            let s = m.r32(o + 0x18);
            if s != 0 {
                let k = m.r32(o + 0x10);
                let x = m.r32(k + 0x294);
                m.call(0x0017_8d90, &[s, x, 0], &[]);
            } else {
                let s = m.call(0x0020_5438, &[k, 0xa7], &[]).0;
                let x = m.r32(o + 0xa8);
                m.w32(s + 0xc8, x);
                let x = m.r32(o + 0xbc);
                m.w32(s + 0x40, x);
                let mut v = 9;
                if m.r32(o + 0xa8) != 0 && m.r32(o + 0xac) == 0 {
                    let k = m.r32(o + 0x10);
                    if m.r32(k + 0xfe8) & 4 != 0 {
                        v = 8;
                    }
                }
                m.w32(s + 0xc0, v);
                let k = m.r32(o + 0x10);
                let x = m.r32(k + 0x294);
                m.call(0x0017_8d90, &[s, x, 0], &[]);
                vslot(m, s, 7, &[], &[]);
            }
        }
        4 => {
            let v = m.call(0x0020_5438, &[k, 0xbe], &[]).0;
            let x = m.r32(o + 0xb8);
            m.w32(v + 0x30, x);
            vslot(m, v, 7, &[], &[]);
        }
        5 => {
            let v = m.call(0x0020_5438, &[k, 0x9e], &[]).0;
            let x = m.r32(o + 0xa8);
            m.w32(v + 0x38, x);
            vslot(m, v, 7, &[], &[]);
        }
        6 => {
            let v = m.call(0x0020_5438, &[k, 0xbf], &[]).0;
            let q = m.rv(o + 0x80);
            m.wv(v + 0x30, q);
            let q = m.rv(o + 0x90);
            m.wv(v + 0x40, q);
            vslot(m, v, 7, &[], &[]);
        }
        _ => {}
    }
}

/// FUN_002586d8(o): resolve up to five object ids of descriptor +0xe4 into a 0x14 table; re-intern if changed.
pub fn ee_sub_2586d8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001a_5c48, &[o], &[]);
    let s = stack_push(m, 0x30);
    let d = m.r32(o + 0xe4);
    let s1 = m.r32(d + 0x20);
    let mut s2 = 0xffff_ffff;
    if m.r32(d + 0x10) == 2 {
        s2 = m.r32(d + 0x30);
    }
    m.call(0x0023_33b4, &[s, 0, 0x14], &[]);
    let look = |m: &mut M, id: u32| m.call(0x0015_e938, &[GS, 4, id, o], &[]).0;
    if s1 != 0xffff_ffff {
        let v = look(m, s1);
        m.w32(s, v);
    }
    if s2 != 0xffff_ffff {
        let v = look(m, s2);
        m.w32(s, 0);
        m.w32(s + 4, v);
    }
    let d = m.r32(o + 0xe4);
    match m.r32(d + 4) {
        0 => {
            let a = m.r32(d + 0xd4);
            if a != 0xffff_ffff {
                let v = look(m, a);
                m.w32(s + 8, v);
            }
        }
        1 => {
            let (a, b, c) = (m.r32(d + 0xc8), m.r32(d + 0xcc), m.r32(d + 0xd0));
            if a != 0xffff_ffff {
                let v = look(m, a);
                m.w32(s + 8, v);
            }
            if b != 0xffff_ffff {
                let v = look(m, b);
                m.w32(s + 0xc, v);
            }
            if c != 0xffff_ffff {
                let v = look(m, c);
                m.w32(s + 0x10, v);
            }
        }
        _ => {}
    }
    let d = m.r32(o + 0xe4);
    if m.r32(d + 8) == 0 {
        m.w32(d + 8, 0x002c_e4a0);
    }
    let d = m.r32(o + 0xe4);
    let p = m.r32(d + 8);
    if m.call(0x0023_3178, &[p, s, 0x14], &[]).0 != 0 {
        m.call(0x0025_8658, &[o, 1], &[]);
        m.w32(s + 0x20, 0x14);
        m.w32(s + 0x24, 0x10);
        let v = m.call(0x001a_e238, &[s + 0x20, s], &[]).0;
        let d = m.r32(o + 0xe4);
        m.w32(d + 8, v);
    }
    stack_pop(m, 0x30);
}

/// FUN_0014e4c8(o, mask, a, b, c, d): remove matching 0x10 entries from the +0x310 list (count +0x30c),
/// notifying each removed one; re-notify the head-entry owner and the new head.
pub fn ee_sub_14e4c8<M: QMem + ?Sized>(m: &mut M, o: u32, fl: u32, a2: u32, a3: u32, t0: u32, t1: u32) {
    let mut s1 = o + 0x310;
    let mut s0 = s1;
    let (mut s4, mut s5, mut l10) = (0u32, 0u32, 0u32);
    let mut i = 0i32;
    if (m.r32(o + 0x30c) as i32) > 0 {
        loop {
            let ok = (fl & 1 == 0 || a2 == m.r32(s0))
                && (fl & 2 == 0 || a3 == m.r32(s0 + 4))
                && (fl & 4 == 0 || t0 == m.r32(s0 + 8))
                && (fl & 8 == 0 || t1 == m.r32(s0 + 0xc));
            if ok {
                let e1 = m.r32(s0 + 4);
                if i == 0 {
                    s4 = e1;
                    l10 = m.r32(s0 + 8);
                    s5 = m.r32(s0 + 0xc);
                }
                let e3 = m.r32(s0 + 0xc);
                vslot(m, e1, 4, &[e3], &[]);
            } else {
                let (x, y) = (m.r64(s0), m.r64(s0 + 8));
                m.w32(s1, x as u32);
                m.w32(s1 + 4, (x >> 32) as u32);
                m.w32(s1 + 8, y as u32);
                m.w32(s1 + 0xc, (y >> 32) as u32);
                s1 += 0x10;
            }
            i += 1;
            s0 += 0x10;
            if i >= m.r32(o + 0x30c) as i32 {
                break;
            }
        }
    }
    let n = ((s1.wrapping_sub(0x310).wrapping_sub(o)) as i32) >> 4;
    m.w32(o + 0x30c, n as u32);
    if s4 != 0 {
        vslot(m, s4, 2, &[l10, s5], &[]);
        if (m.r32(o + 0x30c) as i32) > 0 {
            let a3 = m.r32(o + 0x314);
            let t = m.rf(T);
            let a2 = m.r32(o + 0x31c);
            m.wf(o + 0x390, t);
            let a1 = m.r32(o + 0x318);
            vslot(m, a3, 1, &[a1, a2], &[]);
        }
    }
}

fn dot3(a: V4, b: V4) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// FUN_0011ba08(out, a, b): rotation matrix (3x4 -> 0x30 bytes) taking direction a onto b.
pub fn ee_sub_11ba08<M: QMem + ?Sized>(m: &mut M, out: u32, a: V4, b: V4) -> u32 {
    let eps = f32::from_bits(0x38d1_b717);
    let la = len3(a);
    let lb = len3(b);
    if la < eps || lb < eps {
        m.call(0x0023_33b4, &[out, 0, 0x30], &[]);
        m.wf(out, 1.0);
        m.wf(out + 0x28, 1.0);
        m.wf(out + 0x14, 1.0);
        return out;
    }
    let mut c = [a[1] * b[2] - b[1] * a[2], a[2] * b[0] - b[2] * a[0], a[0] * b[1] - b[0] * a[1], 0.0];
    let lc = len3(c);
    if f32::from_bits(0x322b_cc77) < lc {
        let q = 1.0 / lc;
        c = [c[0] * q, c[1] * q, c[2] * q, c[3] * q];
    } else {
        c = m.call(0x0011_e390, &[], &[a]).1;
    }
    let d = dot3(a, b);
    let f0 = m.call(0x0011_cfe0, &[], &[f4(d / (la * lb)), f4(1.0)]).1[0];
    let sn = (1.0 - f0 * f0).sqrt();
    let s = stack_push(m, 0x30);
    m.call(0x0011_b480, &[s], &[f4(sn), f4(f0), c]);
    m.call(0x0023_320c, &[out, s, 0x30], &[]);
    stack_pop(m, 0x30);
    out
}

/// FUN_001a7c18(o, a, b, t1, t2, t3, head, pos, nrm, d): allocate a 0xb0 two-ended link (pool o+0x3d40),
/// bind both ends, notify, register via FUN_001a7fc0 and prepend the chain to *head.
pub fn ee_sub_1a7c18<M: QMem + ?Sized>(
    m: &mut M,
    o: u32,
    a1: u32,
    a2: u32,
    t1: u32,
    t2: u32,
    t3: u32,
    head: u32,
    pos: V4,
    nrm: V4,
    f20: f32,
) {
    let s = stack_push(m, 0x10);
    let v = m.call(0x0011_ccc0, &[o + 0x3d40], &[]).0;
    m.w32(s, v);
    m.wv(v + 0x10, pos);
    let k = -f20;
    let mut r = [0.0; 4];
    for i in 0..4 {
        r[i] = pos[i] + if i == 3 { 1.0 } else { 0.0 } + nrm[i] * k;
    }
    m.wv(v + 0x50, r);
    m.wv(v + 0x80, nrm);
    for i in 0..2u32 {
        let b = m.r32(s);
        let s0 = b.wrapping_add(i << 6);
        m.w32(s0, if i == 0 { a1 } else { a2 });
        let p = m.r32(s0);
        if p != 0 {
            let x = m.r32(p);
            m.w32(s0 + 4, x);
            let a0 = m.r32(p + 4);
            m.w32(s0 + 0xc, a0);
            let q = m.r32(a0 + 0xec);
            let x = m.r32(q + 0x60);
            m.w32(s0 + 8, x);
        }
        let x = m.r32(s0 + 4);
        let q = m.rv(s0 + 0x10);
        let rv = m.call(0x0022_fc48, &[0, x], &[q]).1;
        m.wv(s0 + 0x20, rv);
    }
    let f1 = m.rf(0x002b_fd30);
    let f0 = m.rf(0x002b_fd34);
    let v = m.r32(s);
    m.w32(v + 0xa4, t3);
    m.wf(v + 0x90, f20);
    m.w32(v + 0x9c, t1);
    m.wf(v + 0x94, f1);
    m.wf(v + 0x98, f0);
    m.w32(v + 0xa0, t2);
    for i in 0..2u32 {
        let a1 = m.r32(s);
        let s0 = a1.wrapping_add(i << 6);
        let a3 = m.r32(s0 + 0xc);
        if a3 != 0 {
            vslot(m, a3, 54, &[a1, i], &[]);
        }
    }
    m.call(0x001a_7fc0, &[o, 1, s], &[]);
    let mut v1 = m.r32(s);
    if m.r32(v1 + 0xac) != 0 {
        v1 = m.r32(v1 + 0xac);
        while m.r32(v1 + 0xac) != 0 {
            v1 = m.r32(v1 + 0xac);
        }
    }
    let h = m.r32(head);
    m.w32(v1 + 0xac, h);
    let x = m.r32(s);
    m.w32(head, x);
    stack_pop(m, 0x10);
}

/// FUN_00228538(o): per-frame update by state +0x13c (jump table 0x2a9890): proximity transitions, timed fire.
pub fn ee_sub_228538<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001a_5e28, &[o], &[]);
    let st = m.r32(o + 0x13c);
    match st {
        0 | 1 | 3 => {
            let v = m.call(0x001c_4840, &[GS], &[]).0;
            if v == 0 {
                return;
            }
            let q = m.call(0x001a_62e8, &[v], &[]).1;
            let r = m.call(0x001a_6f00, &[0, o], &[q]).1;
            let k = m.r32(o + 0xec);
            let a0 = m.call(0x0011_8fb0, &[k, 0x7fff_ffff], &[r]).0 != 0;
            match m.r32(o + 0x13c) {
                1 => {
                    if a0 {
                        m.call(0x0022_8358, &[o, 0], &[]);
                    } else if m.rf(o + 0x15c) < m.rf(T) - m.rf(o + 0x140) {
                        m.call(0x0022_8358, &[o, 3], &[]);
                    }
                }
                0 => {
                    if !a0 {
                        m.call(0x0022_8358, &[o, 1], &[]);
                    }
                }
                3 => {
                    if a0 {
                        m.call(0x0022_83d0, &[o], &[]);
                    }
                }
                _ => {}
            }
        }
        4 => {
            let f1 = m.rf(o + 0x158);
            if !(0.0 < f1) || !(f1 <= m.rf(T)) {
                return;
            }
            let mut i = 0i32;
            if (m.r32(o + 0x148) as i32) > 0 {
                loop {
                    let b = m.r32(o + 0x14c);
                    let x = m.r32(((i as u32) << 2).wrapping_add(b));
                    i += 1;
                    m.call(0x001b_3a48, &[0x002f_66b0, x], &[]);
                    if i >= m.r32(o + 0x148) as i32 {
                        break;
                    }
                }
            }
            let t0 = if m.r32(o + 0x138) != 0 { 0x20 } else { 0 };
            let (a1, a2, t1) = (m.r32(o + 0x144), m.r32(o + 0x150), m.r32(o + 0x154));
            m.call(0x0022_b490, &[GS, a1, a2, 0xffff_ffff, t0, t1], &[]);
            m.w32(o + 0x158, 0);
        }
        _ => {}
    }
}

fn dist3(a: V4, b: V4) -> f32 {
    len3([a[0] - b[0], a[1] - b[1], a[2] - b[2], 0.0])
}

/// FUN_00225660(o, st): state change; leaving 5 releases the grabbed actor, entering 5 grabs the nearest
/// (< 400) qualifying actor and launches it.
pub fn ee_sub_225660<M: QMem + ?Sized>(m: &mut M, o: u32, a1: u32) {
    let v1 = m.r32(o + 0x150);
    if a1 == v1 {
        return;
    }
    if v1 == 5 {
        let a = m.r32(o + 0x1b4);
        if a != 0 {
            m.call(0x0022_9738, &[a, o + 0x1b8], &[]);
            let a = m.r32(o + 0x1b4);
            let v = m.call(0x0020_5458, &[a, 0xb0], &[]).0;
            if v != 0 {
                m.call(0x0017_a8c8, &[v, 2], &[]);
            }
            let a = m.r32(o + 0x1b4);
            let a = m.r32(a + 0xfc4);
            if a != 0 {
                let q = m.rv(0x002a_2de0);
                m.call(0x0012_d108, &[a], &[q]);
            }
            m.w32(o + 0x1b4, 0);
        }
    }
    m.call(0x0022_4ec8, &[o, a1], &[]);
    if a1 != 5 {
        return;
    }
    let a = m.r32(o + 0x1b0);
    let me = if a != 0 { m.call(0x001a_62e8, &[a], &[]).1 } else { m.call(0x001a_62e8, &[o], &[]).1 };
    let s = stack_push(m, 0x80);
    let n = m.call(0x0015_e9c8, &[GS, 0x105, 0x38, 0, 0x20, s], &[]).0;
    let mut best = 400.0f32;
    let mut s2 = 0u32;
    if (n as i32) > 0 {
        for i in 0..n {
            let s0 = m.r32(s + i.wrapping_mul(4));
            if m.call(0x0020_6eb0, &[s0, 0xb], &[]).0 != 0 {
                let p = m.call(0x001a_62e8, &[s0], &[]).1;
                let d = dist3(p, me);
                if d < best {
                    best = d;
                    s2 = s0;
                }
            }
        }
    }
    stack_pop(m, 0x80);
    if s2 == 0 {
        return;
    }
    m.w32(o + 0x1b4, s2);
    m.w32(o + 0x1b8, o);
    m.w32(o + 0x1bc, s2);
    m.call(0x0022_9710, &[s2, o + 0x1b8], &[]);
    let v = vslot(m, s2, 32, &[], &[]).1;
    let k = m.rf(0x002c_43f0);
    let mut r = [v[0] * k, v[1] * k, v[2] * k, v[3] * k];
    r[2] = m.rf(0x002c_43f4);
    vslot(m, s2, 34, &[], &[r]);
    m.call(0x0020_9e28, &[s2, 0], &[]);
}

fn mulrows(r0: V4, r1: V4, r2: V4, v: V4, w: f32) -> V4 {
    [
        r0[0] * v[0] + r1[0] * v[1] + r2[0] * v[2],
        r0[1] * v[0] + r1[1] * v[1] + r2[1] * v[2],
        r0[2] * v[0] + r1[2] * v[1] + r2[2] * v[2],
        w,
    ]
}

/// FUN_0015c3c0(o, msg): forward a 0x40-byte transform message to every child (+0xe0 count, +0xe4 list),
/// re-expressed in the child's frame (vslot 50).
pub fn ee_sub_15c3c0<M: QMem + ?Sized>(m: &mut M, o: u32, msg: u32) {
    let s = stack_push(m, 0x140);
    for i in (0..0x40).step_by(16) {
        let q = m.rv(msg + i);
        m.wv(s + i, q);
    }
    let a3 = m.r32(s);
    m.w32(s + 0x34, s + 0x40);
    m.w32(s, a3 | 0x40);
    if a3 & 1 == 0 {
        m.w32(s + 4, o);
        m.w32(s, a3 | 0x41);
    }
    let (s6, s5, s1) = (s + 0xe0, s + 0x120, s + 0xa0);
    let mut i = 0i32;
    if (m.r32(o + 0xe0) as i32) > 0 {
        loop {
            let b = m.r32(o + 0xe4);
            let s0 = m.r32(((i as u32) << 2).wrapping_add(b));
            m.call(0x0011_c318, &[s5, o + 0x60], &[]);
            m.call(0x0011_c168, &[s6, s5, s0 + 0x60], &[]);
            let v1 = m.rv(s + 0xe0);
            let v7 = m.rv(s + 0x110);
            let q = m.rv(s + 0xf0);
            m.wv(s + 0xb0, q);
            let q = m.rv(s + 0x100);
            m.wv(s + 0xc0, q);
            m.wv(s + 0xa0, v1);
            m.wv(s + 0xd0, v7);
            if m.r32(msg) & 0x40 != 0 {
                let a0 = m.r32(msg + 0x34);
                let (r0, r1, r2) = (m.rv(a0 + 0x10), m.rv(a0 + 0x20), m.rv(a0 + 0x30));
                let v5 = mulrows(r0, r1, r2, v1, 0.0);
                m.wv(s + 0xe0, v5);
                m.wv(s + 0x110, v5);
                let q = m.rv(s1 + 0x10);
                let v2 = mulrows(r0, r1, r2, q, q[3]);
                m.wv(s + 0xf0, v2);
                m.wv(s + 0x120, v2);
                let q = m.rv(s1 + 0x20);
                let v3 = mulrows(r0, r1, r2, q, r0[3]);
                m.wv(s + 0x50, v5);
                m.wv(s + 0x60, v2);
                m.wv(s + 0x70, v3);
                m.wv(s + 0x100, v3);
                m.wv(s + 0x130, v3);
                let (r0, r1, r2, p) = (m.rv(a0 + 0x10), m.rv(a0 + 0x30), m.rv(a0 + 0x20), m.rv(a0));
                let t = mulrows(r0, r2, r1, v7, r0[3]);
                m.wv(s + 0x40, [p[0] + t[0], p[1] + t[1], p[2] + t[2], p[3] + t[3]]);
                let q = m.rv(a0 + 0x40);
                m.wv(s + 0x80, q);
                let q = m.rv(a0 + 0x50);
                m.wv(s + 0x90, q);
            } else {
                m.wv(s + 0x50, v1);
                let q = m.rv(s1 + 0x10);
                m.wv(s + 0x60, q);
                let q = m.rv(s1 + 0x20);
                m.wv(s + 0x40, v7);
                m.wv(s + 0x70, q);
                for k in [0x88, 0x84, 0x80, 0x98, 0x94, 0x90] {
                    m.w32(s + k, 0);
                }
            }
            i += 1;
            vslot(m, s0, 50, &[s], &[]);
            if i >= m.r32(o + 0xe0) as i32 {
                break;
            }
        }
    }
    stack_pop(m, 0x140);
}

/// FUN_0014c748(c, p0, p1, n, out, r): segment p0->p1 vs sphere (c, r); writes entry/exit points, normals
/// (FUN_00119390) and params to out; returns 1 on hit.
pub fn ee_sub_14c748<M: QMem + ?Sized>(m: &mut M, a3: u32, out: u32, c: V4, p0: V4, p1: V4, r: f32) -> u32 {
    if (a3 as i32) <= 0 {
        return 0;
    }
    let d1 = [p0[0] - c[0], p0[1] - c[1], p0[2] - c[2], p0[3] - c[3]];
    let cc = (dot3(d1, d1) - r * r) * 4.0;
    let e = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2], p1[3] - p0[3]];
    let b = dot3(e, d1);
    let b2 = b + b;
    let a = dot3(e, e);
    let disc = b2 * b2 - a * cc;
    let eps = f32::from_bits(0x38d1_b717);
    if disc < eps {
        return 0;
    }
    let sq = disc.sqrt();
    let a2 = a + a;
    let t1 = (-b2 - sq) / a2;
    let t2 = (-b2 + sq) / a2;
    if t1 < 0.0 && t2 < 0.0 {
        return 0;
    }
    if 1.0 < t1 && 1.0 < t2 {
        return 0;
    }
    let ts = [t1.max(0.0), t2.min(1.0)];
    for k in 0..2u32 {
        let t = ts[k as usize];
        let u = 1.0 - t;
        m.wf(out + 0x40 + 4 * k, t);
        let p = [p1[0] * t + p0[0] * u, p1[1] * t + p0[1] * u, p1[2] * t + p0[2] * u, p1[3] * t + p0[3] * u];
        m.wv(out + 0x10 * k, p);
        let dv = [p[0] - c[0], p[1] - c[1], p[2] - c[2], p[3] - c[3]];
        let l = len3(dv);
        let n = if l < eps {
            m.rv(0x002a_2da0)
        } else {
            let q = 1.0 / l;
            [dv[0] * q, dv[1] * q, dv[2] * q, dv[3] * q]
        };
        m.call(0x0011_9390, &[out + 0x20 + 0x10 * k], &[n, p]);
    }
    m.w32(out + 0x48, 1);
    1
}

fn feq3<M: QMem + ?Sized>(m: &mut M, a: V4, b: V4) -> bool {
    let eps = f4(f32::from_bits(0x38d1_b717));
    for i in 0..3 {
        if m.call(0x0011_d4d0, &[], &[f4(a[i]), f4(b[i]), eps]).0 == 0 {
            return false;
        }
    }
    true
}

/// FUN_0012bbd0(track, o): sample keyframe pair at *(o+0x88) into o+0x80..0xff; +0x9a = 2 when the two
/// positions coincide and both tangents are zero, else 1.
pub fn ee_sub_12bbd0<M: QMem + ?Sized>(m: &mut M, a0: u32, s3: u32) {
    let p = m.r32(s3 + 0x88);
    let f = m.rf(p);
    let idx = m.call(0x0012_bb70, &[a0], &[f4(f)]).0;
    let o = idx << 5;
    let b = m.r32(a0 + 8).wrapping_add(o);
    let x = m.r32(b.wrapping_sub(0x14));
    m.w32(s3 + 0x80, x);
    for k in 0..3 {
        let x = m.r32(b.wrapping_sub(0x20) + 4 * k);
        m.w32(s3 + 0xc0 + 4 * k, x);
    }
    let q = m.rv(b.wrapping_sub(0x10));
    m.wv(s3 + 0xd0, q);
    let x = m.r32(b + 0xc);
    m.w32(s3 + 0x84, x);
    for k in 0..3 {
        let x = m.r32(b + 4 * k);
        m.w32(s3 + 0xe0 + 4 * k, x);
    }
    let q = m.rv(b + 0x10);
    m.wv(s3 + 0xf0, q);
    let (c, e) = (m.rv(s3 + 0xc0), m.rv(s3 + 0xe0));
    let mut s5 = false;
    if feq3(m, c, e) {
        let z = m.rv(0x002a_2dd0);
        let d = m.rv(s3 + 0xd0);
        if feq3(m, z, d) {
            let d = m.rv(s3 + 0xf0);
            let z = m.rv(0x002a_2dd0);
            s5 = feq3(m, z, d);
        }
    }
    m.w16(s3 + 0x9a, if s5 { 2 } else { 1 });
}

/// FUN_001a0450(o, tbl, a2): distribute the velocity o+0x190 over the 8 directional blend entries of `tbl`
/// (0x20 stride: weight, dir at +0x10), rebuild the velocity from the blend, then FUN_001384c0(a2).
pub fn ee_sub_1a0450<M: QMem + ?Sized>(m: &mut M, o: u32, a1: u32, a2: u32) {
    if a2 == 0 {
        return;
    }
    let p = m.r32(o + 0x1030);
    let f4v = m.rf(p + 0x40);
    if m.r32(a1 + 0xc4) == 0 || m.r32(a1 + 0xe4) == 0 {
        m.w32(o + 0x1038, 0xffff_ffff);
    } else if m.r32(o + 0x1038) == 0xffff_ffff {
        let d = dot3(m.rv(o + 0x190), m.rv(a1 + 0x10));
        m.w32(o + 0x1038, if d < 0.0 { 1 } else { 0 });
    }
    for k in 0..8 {
        m.w32(a1 + 0xe0 - 0x20 * k, 0);
    }
    let a3 = m.r32(o + 0x1038);
    let (sl, t1): ([u32; 5], usize) = match a3 as i32 {
        0 => ([6, 2, 4, 0, 0], 4),
        1 => ([7, 3, 5, a3, 0], 4),
        -1 => {
            if m.r32(a1 + 0xe4) != 0 {
                ([7, 3, 5, 0, 1], 5)
            } else {
                ([6, 2, 4, 0, 1], 5)
            }
        }
        _ => ([0; 5], 0),
    };
    let mut f2 = 0.0f32;
    if 0.0 < f4v && 1 < t1 {
        for i in 1..t1 {
            let e = a1.wrapping_add(sl[i] << 5);
            let v5 = m.rv(o + 0x190);
            let dir = m.rv(e + 0x10);
            let w = (dot3(v5, dir) / (f4v * dot3(dir, dir))).max(0.0);
            m.wf(e, w);
            f2 += w;
        }
    }
    let e0 = a1.wrapping_add(sl[0] << 5);
    if f2 < 1.0 {
        m.wf(e0, 1.0 - f2);
    } else {
        m.w32(e0, 0);
        for i in 1..t1 {
            let e = (sl[i] << 5).wrapping_add(a1);
            let w = m.rf(e) / f2;
            m.wf(e, w);
        }
    }
    let mut acc = [0.0f32; 4];
    for k in 0..8u32 {
        let e = a1 + 0x20 * k;
        let w = m.rf(e);
        let d = m.rv(e + 0x10);
        for i in 0..4 {
            acc[i] = acc[i] + if i == 3 { 1.0 } else { 0.0 } + d[i] * w;
        }
    }
    m.wv(o + 0x190, [acc[0] * f4v, acc[1] * f4v, acc[2] * f4v, acc[3] * f4v]);
    m.call(0x0013_84c0, &[a2], &[]);
}

fn add4(a: V4, b: V4) -> V4 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2], a[3] + b[3]]
}

/// x*r0 + y*r1 + z*r2 (+ r3 when given); w = r0.w
fn xform(r0: V4, r1: V4, r2: V4, r3: Option<V4>, v: V4) -> V4 {
    let t = r3.unwrap_or([0.0; 4]);
    let mut o = r0;
    for i in 0..3 {
        o[i] = r0[i] * v[0] + r1[i] * v[1] + r2[i] * v[2] + t[i];
    }
    o
}

/// FUN_0022d348(node, stop, p): accumulate the world position of local point p up the +0xa0 parent chain
/// until `stop`, then express it in stop's frame.
pub fn ee_sub_22d348<M: QMem + ?Sized>(m: &mut M, a0: u32, s2: u32, l: V4) -> V4 {
    let s = stack_push(m, 0xa0);
    let mut acc = [0.0f32; 4];
    let mut t160 = [0.0f32; 4];
    let mut s0 = a0;
    while s0 != s2 {
        let k = m.r16(s0 + 0x98) as i16 as i32 as u32;
        let a1h = m.r16(s0 + 0x98) as u32;
        let skip = k < 20 && matches!(k, 0 | 3 | 10 | 12 | 15 | 19);
        if !skip {
            let v = m.r32(s0 + 0x94).wrapping_sub(1);
            if v < 10 && matches!(v, 0 | 3 | 4 | 8 | 9) {
                let j = a1h.wrapping_sub(1) as u16 as i16 as i32 as u32;
                let mut store = None;
                if j < 21 {
                    match j {
                        0 | 1 | 6 | 7 | 8 | 10 | 16 | 17 | 19 | 20 => store = Some(m.rv(s0 + 0xb0)),
                        4 | 5 => {
                            let p = m.r32(s0 + 0x90);
                            if p != 0 {
                                let x = m.r32(p + 4);
                                m.call(0x0022_dd00, &[x, s0], &[]);
                            }
                            let k2 = m.r16(s0 + 0x9a) as i16 as i32 as u32;
                            store = Some(match k2 {
                                3 | 4 => m.rv(s0 + 0xd0),
                                1 => {
                                    let p88 = m.r32(s0 + 0x88);
                                    let f0 = m.rf(s0 + 0x80);
                                    let f13 = m.rf(p88) - f0;
                                    let f12 = m.rf(s0 + 0x84) - f0;
                                    let p8c = m.r32(s0 + 0x8c);
                                    let f14 = m.rf(p8c);
                                    let (q0, q1, q2, q3) =
                                        (m.rv(s0 + 0xc0), m.rv(s0 + 0xd0), m.rv(s0 + 0xe0), m.rv(s0 + 0xf0));
                                    m.call(
                                        0x0011_7120,
                                        &[0, s + 0x40, 0],
                                        &[q0, q1, q2, q3, f4(f12), f4(f13), f4(f14)],
                                    );
                                    m.rv(s + 0x40)
                                }
                                _ => m.rv(0x002a_2dd0),
                            });
                        }
                        _ => {}
                    }
                }
                if let Some(t) = store {
                    t160 = t;
                }
                let p = m.r32(s0 + 0xa0);
                if p != 0 {
                    let (r0, r1, r2) = (m.rv(p + 0x40), m.rv(p + 0x50), m.rv(p + 0x60));
                    acc = add4(acc, xform(r0, r1, r2, None, t160));
                } else {
                    acc = add4(acc, t160);
                }
            } else {
                let p = m.r32(s0 + 0x90);
                if p != 0 {
                    let x = m.r32(p + 4);
                    m.call(0x0022_dd00, &[x, s0], &[]);
                }
                m.call(0x0022_cdc0, &[s + 0x50, s0], &[]);
                for i in (0..0x40).step_by(16) {
                    let q = m.rv(s + 0x50 + i);
                    m.wv(s + i, q);
                }
                let s1 = m.r32(s0 + 0xa0);
                m.call(0x0011_c318, &[s + 0x40, s0 + 0x40], &[]);
                let n = [m.rv(s + 0x40), m.rv(s + 0x50), m.rv(s + 0x60), m.rv(s + 0x70)];
                let v4 = xform(n[0], n[1], n[2], Some(n[3]), l);
                let mm = [m.rv(s), m.rv(s + 0x10), m.rv(s + 0x20), m.rv(s + 0x30)];
                let v3 = xform(mm[0], mm[1], mm[2], Some(mm[3]), v4);
                if s1 != 0 {
                    let (p0, p1, p2) = (m.rv(s1 + 0x40), m.rv(s1 + 0x50), m.rv(s1 + 0x60));
                    let mut r = xform(p0, p1, p2, None, v3);
                    r[3] = v3[3];
                    acc = add4(acc, r);
                } else {
                    acc = add4(acc, v3);
                }
            }
        }
        s0 = m.r32(s0 + 0xa0);
    }
    if s2 != 0 {
        m.call(0x0011_c318, &[s + 0x40, s2 + 0x40], &[]);
        let (r0, r1, r2) = (m.rv(s + 0x40), m.rv(s + 0x50), m.rv(s + 0x60));
        let mut r = xform(r0, r1, r2, None, acc);
        r[3] = acc[3];
        acc = r;
    }
    stack_pop(m, 0xa0);
    acc
}

fn sub4(a: V4, b: V4) -> V4 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2], a[3] - b[3]]
}

/// FUN_0019eab0(o, st): rail/rope attach state machine (+4), jump table 0x2a6680.
pub fn ee_sub_19eab0<M: QMem + ?Sized>(m: &mut M, o: u32, st: u32) {
    let s0 = m.r32(o + 4);
    if st == s0 {
        return;
    }
    if s0 == 0xffff_ffff {
        let s = stack_push(m, 0x40);
        let t2 = m.r32(o + 0xc);
        vslot(m, t2, 2, &[5, 0, 0, 0, 0], &[]);
        let (a, b) = (m.r32(o + 0xc), m.r32(o));
        m.call(0x0013_33c0, &[a, b], &[]);
        let (a, b) = (m.r32(o), m.r32(o + 0xc));
        m.call(0x0020_4f78, &[a, b, 1], &[]);
        let x = m.rf(o + 0x10) + 50.0;
        let c = m.r32(o + 0xc);
        m.w32(o + 0x20, s0);
        let f = m.call(0x001e_6728, &[c], &[f4(x)]).1[0];
        let c = m.r32(o + 0xc);
        m.call(0x001e_6f10, &[c, s, 0], &[f4(f)]);
        let x = m.rf(o + 0x10) - 50.0;
        let c = m.r32(o + 0xc);
        let f = m.call(0x001e_6728, &[c], &[f4(x)]).1[0];
        let c = m.r32(o + 0xc);
        m.call(0x001e_6f10, &[c, s + 0x10, 0], &[f4(f)]);
        let d = sub4(m.rv(s), m.rv(s + 0x10));
        let l = len3(d);
        let mut v = m.rv(0x002a_2dc0);
        if !(l < f32::from_bits(0x38d1_b717)) {
            let q = 1.0 / l;
            v = [d[0] * q, d[1] * q, d[2] * q, d[3] * q];
        }
        m.w32(o + 0x18, (v[2] < -0.5) as u32);
        m.call(0x0019_e110, &[o, s + 0x20], &[]);
        let x = m.rf(o + 0x10);
        let f = m.call(0x0011_d948, &[s + 0x20], &[f4(x)]).1[0];
        let c = m.r32(o + 0xc);
        m.wf(o + 0x10, f);
        m.call(0x001e_6f10, &[c, s + 0x30, 0], &[f4(f)]);
        let a = m.r32(o);
        let q = m.rv(o + 0x30);
        let r = m.call(0x001a_6f00, &[a, 0], &[q]).1;
        let p = m.r32(o);
        let s30 = m.rv(s + 0x30);
        m.wv(p + 0x1010, sub4(s30, r));
        let p = m.r32(o);
        let q = m.rv(p + 0x1060);
        m.wv(p + 0x1070, sub4(q, s30));
        let p = m.r32(o);
        m.w32(p + 0x1078, 0);
        let c = m.r32(o + 0xc);
        let (a, b) = (m.r32(o), m.r32(c + 0x190));
        m.call(0x0014_0c88, &[a, b], &[]);
        let c = m.r32(o + 0xc);
        let a = m.r32(c + 0x1d0);
        if a != 0 {
            let s0 = m.call(0x001f_bb90, &[a], &[]).0;
            m.call(0x001f_b2f8, &[s0, 0], &[]);
            let q = m.rv(s + 0x30);
            m.call(0x001f_b3f0, &[s0], &[q]);
        }
        stack_pop(m, 0x40);
    } else if s0 == 0 {
        let a = m.r32(o);
        if a == m.r32(0x002e_1e40) {
            let a1 = if m.r32(a + 0x14d4) == 0 { 2 } else { 0xa };
            m.call(0x0018_e540, &[a, a1], &[]);
        }
    }
    m.w32(o + 4, st);
    let t = m.rf(T);
    m.wf(o + 8, t);
    let (mut s0, mut s2, mut f21, mut f20) = (0u32, 1u32, 0.0f32, 1.0f32);
    match st.wrapping_add(1) {
        0 => {
            let t2 = m.r32(o + 0xc);
            vslot(m, t2, 2, &[6, 0, 0, 0, 0], &[]);
            let (a, b) = (m.r32(o + 0xc), m.r32(o));
            m.call(0x0013_3448, &[a, b], &[]);
            let (a, b) = (m.r32(o), m.r32(o + 0xc));
            m.call(0x0020_4f78, &[a, b, 0], &[]);
            let s0 = m.r32(o);
            let q = m.rv(s0 + 0x1060);
            let w = m.call(0x001a_62e8, &[s0], &[]).1;
            m.wv(s0 + 0x1070, sub4(q, w));
            let p = m.r32(o);
            m.w32(p + 0x1078, 0);
            let a = m.r32(o);
            m.call(0x0014_0c88, &[a, 0], &[]);
            let c = m.r32(o + 0xc);
            let a = m.r32(c + 0x1d4);
            if a != 0 {
                let s0 = m.call(0x001f_bb90, &[a], &[]).0;
                m.call(0x001f_b2f8, &[s0, 0], &[]);
                let a = m.r32(o);
                let w = m.call(0x001a_62e8, &[a], &[]).1;
                m.call(0x001f_b3f0, &[s0], &[w]);
            }
            m.w32(o + 0xc, 0);
            return;
        }
        1 => {
            s0 = m.r32(o + 0x74);
            if s0 == 0 {
                s0 = m.r32(o + 0x7c);
                f20 = 0.5;
                f21 = m.rf(s0 + 0x74) * 0.5;
            }
            m.w32(o + 0x1c, 0);
        }
        2 => {
            s0 = m.r32(o + 0x78);
            s2 = 9;
        }
        3 => {
            s0 = m.r32(o + 0x70);
            s2 = 9;
        }
        4 => {
            s0 = m.r32(o + 0x70);
            f20 = -1.0;
            s2 = 9;
        }
        5 => {
            let v = m.r32(o + 0x18);
            s0 = m.r32(o + 0x7c);
            m.w32(o + 0x18, (v == 0) as u32);
        }
        _ => {}
    }
    let a = m.r32(o);
    m.call(0x001f_15b8, &[a + 0x1034], &[]);
    let a1 = m.r32(o);
    m.call(0x0013_05b8, &[s0, a1, s2, a1 + 0x1030], &[f4(f21), f4(f20)]);
    let p = m.r32(o);
    m.w32(p + 0x194, 0);
    m.w32(p + 0x198, 0);
    m.w32(p + 0x190, 0);
}
