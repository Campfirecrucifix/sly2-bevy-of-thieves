//! WIRE-W1G3: 1:1 queue ports (against `QMem`) of a batch of live-frontier EE natives, ported from the
//! EE asm (tools/eedis.py); callees go through `m.call` / `vslot` (the live dispatch routes ported ones).
//! QMem convention: integer / pointer args in `a`, quadword (128-bit GPR) and float args in `v`.
#![allow(clippy::too_many_arguments)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

fn cpn<M: QMem + ?Sized>(m: &mut M, dst: u32, src: u32, n: u32) {
    for k in (0..n).step_by(4) {
        let w = m.r32(src + k);
        m.w32(dst + k, w);
    }
}

/// FUN_002315f0(o, a1, a2, fl): snapshot world matrix (slot 30 if +0x28 bit 60, else +0x60.. folded through
/// the +0x50 parent chain via FUN_0011c168); set +0x50 parent; FUN_001a5900; fl&1 -> slot 28(row 3 qword);
/// fl&2 -> slot 29(&matrix).
pub fn ee_sub_2315f0<M: QMem + ?Sized>(m: &mut M, o: u32, a1: u32, a2: u32, fl: u32) {
    let sp = stack_push(m, 0x80);
    if fl & 3 != 0 {
        if (m.r64(o + 0x28) >> 60) & 1 != 0 {
            let p = vslot(m, o, 30, &[], &[]).0;
            cpn(m, sp, p, 0x40);
        } else {
            cpn(m, sp, o + 0x60, 0x40);
            let mut s0 = m.r32(o + 0x50);
            while s0 != 0 {
                m.call(0x0011_c168, &[sp + 0x40, s0, sp], &[]);
                cpn(m, sp, sp + 0x40, 0x40);
                s0 = m.r32(s0 + 0xa0);
            }
        }
    }
    if a2 != 0 {
        m.w32(o + 0x50, a2);
    } else if a1 == 0 {
        m.w32(o + 0x50, 0);
    } else {
        let v = m.r32(a1 + 0x60);
        m.w32(o + 0x50, v);
    }
    m.call(0x001a_5900, &[o, a1, a2, fl], &[]);
    if fl & 1 != 0 {
        let q = m.rv(sp + 0x30);
        vslot(m, o, 28, &[], &[q]);
    }
    if fl & 2 != 0 {
        vslot(m, o, 29, &[sp], &[]);
    }
    stack_pop(m, 0x80);
}

/// FUN_001b98a0(jt, msg) -> bool: FUN_001443d8 gate; by +0x13d0 state sub-handler, then by msg.
pub fn ee_sub_1b98a0<M: QMem + ?Sized>(m: &mut M, o: u32, a1: u32) -> u32 {
    if m.call(0x0014_43d8, &[o, a1], &[]).0 != 0 {
        return 0;
    }
    let r = match m.r32(o + 0x13d0) {
        0 | 1 | 9 | 10 => 1,
        2 => m.call(0x0019_c8d0, &[o + 0x2c0, a1], &[]).0,
        5 => {
            if a1 == 3 {
                return 1;
            }
            if a1 != 0x13 {
                return 0;
            }
            let p = m.r32(o + 0x13f0);
            let p = m.r32(p + 0x180);
            return (m.call(0x0013_2b10, &[p, 0x38], &[]).0 != 0) as u32;
        }
        0xd => m.call(0x0019_4558, &[o + 0x4c0, a1], &[]).0,
        0x14 => m.call(0x0019_eef0, &[o + 0xf50, a1], &[]).0,
        _ => 0,
    };
    if r == 0 {
        return 0;
    }
    match a1 {
        1 | 2 | 0xf | 0x10 | 0x14 | 0x15 => 1,
        3 => {
            if m.r32(o + 0x12f0) == 0 || m.r32(0x002d_fbc0) & 2 == 0 {
                0
            } else {
                (m.call(0x0014_fd50, &[0x002d_e2e0], &[]).0 == 0) as u32
            }
        }
        0x16 => {
            let v = m.r32(o + 0x2c4);
            (v != 0 && v != 0xb && v != 0xffff_ffff) as u32
        }
        _ => 0,
    }
}

fn link_ptr<M: QMem + ?Sized>(m: &mut M, idx: u32, bank: u32) -> u32 {
    if idx == 0 && bank == 0 {
        0
    } else {
        m.r32(0x0045_2138u32.wrapping_add(bank.wrapping_mul(4))).wrapping_add(idx.wrapping_mul(0xc))
    }
}

/// FUN_00255de8(n): unlink node `n` (packed 13-bit prev/next indices + 2-bit banks at +4) from its list,
/// fixing the head/tail/cursor globals 0x2dd2c0 / 0x2dd2cc / 0x2dd2c8.
pub fn ee_sub_255de8<M: QMem + ?Sized>(m: &mut M, a3: u32) {
    let w = m.r32(a3 + 4);
    let t1 = link_ptr(m, w & 0x1fff, (w >> 13) & 3);
    let w = m.r32(a3 + 4);
    let t0 = link_ptr(m, (w >> 16) & 0x1fff, (w >> 29) & 3);
    if a3 == m.r32(0x002d_d2c8) {
        m.w32(0x002d_d2c8, t0);
    }
    if t1 != 0 {
        let a0 = (m.r16(a3 + 6) as u32 & 0x1fff) << 16;
        let mut v1 = (m.r32(t1 + 4) & 0xe000_ffff) | a0;
        m.w32(t1 + 4, v1);
        v1 &= 0x9fff_ffff;
        v1 |= m.r32(a3 + 4) & 0x6000_0000;
        m.w32(t1 + 4, v1);
    } else if a3 == m.r32(0x002d_d2c0) {
        m.w32(0x002d_d2c0, t0);
    } else if a3 == m.r32(0x002d_d2cc) {
        m.w32(0x002d_d2cc, t0);
    }
    if t0 != 0 {
        let lo = m.r32(a3 + 4) & 0x1fff;
        let mut v1 = (m.r32(t0 + 4) & 0xffff_e000) | lo;
        m.w32(t0 + 4, v1);
        v1 &= 0xffff_9fff;
        v1 |= m.r32(a3 + 4) & 0x6000;
        m.w32(t0 + 4, v1);
    }
}

/// FUN_001c0d70(pt, a1): walk the BSP at *0x3e1a5c (planes: dot3(plane, pt) - plane.w > 0 picks child);
/// leaf -> region table entry; fell off -> *0x3e1a58 (a1 != 0 or tree +0x10 == 0) else 0.
pub fn ee_sub_1c0d70<M: QMem + ?Sized>(m: &mut M, a1: u32, pt: V4) -> u32 {
    let s1 = m.r32(0x003e_1a5c);
    if s1 == 0 {
        return 0;
    }
    let n = m.r32(s1);
    let s0b = m.r32(s1 + 4);
    let s4 = m.r32(s1 + 0x18);
    let s2 = s0b.wrapping_add(n << 3);
    let mut s0 = s0b;
    if s0 != s2 {
        let sp = stack_push(m, 0x40);
        m.wv(sp + 0x20, pt);
        loop {
            let pl_ = m.r32(s0);
            let pl = m.rv(pl_);
            m.wv(sp + 0x30, pl);
            m.call(0x0025_64b8, &[sp], &[]);
            m.call(0x0025_64b8, &[sp + 0x10], &[]);
            let p = m.rv(sp + 0x30);
            let q = m.rv(sp + 0x20);
            let d = p[0] * q[0] + p[1] * q[1] + 1.0 * (p[2] * q[2]) - 1.0 * p[3];
            m.wv(sp + 0x10, [1.0, 0.0, 0.0, 1.0]);
            m.wv(sp, [d, p[1] * q[1], p[2] * q[2], p[3]]);
            let k = (0.0 < d) as u32;
            let idx = m.r16(s0 + 4 + k.wrapping_mul(2)) as u32;
            if idx & 0x8000 != 0 {
                stack_pop(m, 0x40);
                return m.r32(((idx & 0x7fff) << 2).wrapping_add(s4));
            }
            s0 = s0b.wrapping_add(idx << 3);
            if s0 == s2 {
                break;
            }
        }
        stack_pop(m, 0x40);
    }
    if a1 != 0 || m.r32(s1 + 0x10) == 0 {
        m.r32(0x003e_1a58)
    } else {
        0
    }
}

/// FUN_00212298(o, bis): +0x70..0x78 = 0x310/0x30f/0x311; FUN_001a5c28(o); read flag / f32 / 3 words from
/// the stream; flag -> allocate + register a 0x50/0x10 helper, else load the inline data.
pub fn ee_sub_212298<M: QMem + ?Sized>(m: &mut M, o: u32, bis: u32) {
    m.w32(o + 0x78, 0x311);
    m.w32(o + 0x74, 0x30f);
    m.w32(o + 0x70, 0x310);
    m.call(0x001a_5c28, &[o], &[]);
    let s0 = m.call(0x0011_7d00, &[bis], &[]).0;
    let f = m.call(0x0011_7e38, &[bis], &[]).1[0];
    m.wf(o + 0x90, f);
    let v = m.call(0x0011_7c80, &[bis], &[]).0;
    m.w32(o + 0x94, v);
    let v = m.call(0x0011_7c80, &[bis], &[]).0;
    m.w32(o + 0x98, v);
    let v = m.call(0x0011_7c80, &[bis], &[]).0;
    m.w32(o + 0xbc, v);
    if s0 != 0 {
        m.w32(o + 0x58, 0);
        let sp = stack_push(m, 0x10);
        m.w32(sp + 4, 0x10);
        m.w32(sp, 0x50);
        m.w32(o + 0xc4, 0x003e_1c70);
        let p = m.call(0x001a_e2a0, &[sp], &[]).0;
        stack_pop(m, 0x10);
        m.w32(o + 0xc0, p);
        m.w32(p + 0x40, o);
        let h = m.r32(o + 0xc0);
        m.call(0x0011_8458, &[bis, h], &[]);
        let h = m.r32(o + 0xc0);
        m.call(0x0011_a070, &[0x003e_106c, h], &[]);
    } else {
        m.w32(o + 0x58, 1);
        m.w32(o + 0xc4, 0x003e_1c80);
        m.call(0x0021_4398, &[o, bis, o, o + 0x50, o + 0x54], &[]);
        m.call(0x0021_4460, &[o, bis, o], &[]);
    }
    m.w32(o + 0x7c, 0xffff_ffff);
}

/// FUN_00205478(o, mask): for i in 0..6 with mask bit i, bump o+0x2a0[i]; first ref -> look up the
/// effect (FUN_001fd0c0 / FUN_0015e938) and attach it to `o` unless already attached.
pub fn ee_sub_205478<M: QMem + ?Sized>(m: &mut M, o: u32, mask: u32) {
    let sp = stack_push(m, 0x10);
    let mut s1 = o + 0x2a0;
    let mut i = 0u32;
    loop {
        let bit = if i < 6 { 1u32 << i } else { 0 };
        if mask & bit != 0 {
            let v = m.r32(s1).wrapping_add(1);
            m.w32(s1, v);
            if (v as i32) < 2 {
                let id = m.call(0x001f_d0c0, &[0x003d_d040, i], &[]).0;
                let s0 = m.call(0x0015_e938, &[0x003d_d490, 4, id, o], &[]).0;
                if s0 != 0 {
                    let r = m.call(0x001f_03c8, &[s0, o], &[]).0;
                    m.w32(sp, r);
                    if r == 0 {
                        m.call(0x001f_0198, &[s0, o, 0xffff_ffff, 9, sp], &[]);
                    }
                }
            }
        }
        i += 1;
        s1 += 4;
        if (i as i32) >= 6 {
            break;
        }
    }
    stack_pop(m, 0x10);
}

/// FUN_0012d108(o, pos): +0x20 = pos; if the owner's (+8) active entry (+0x90) is `o` and the
/// delta pos - old differs from the 0x2a2dd0 vector (per x/y/z, FUN_0011d4d0 with eps 1e-4), push
/// pos to owner +0xc0 and FUN_00230598(+4, +8).
pub fn ee_sub_12d108<M: QMem + ?Sized>(m: &mut M, o: u32, pos: V4) {
    let old = m.rv(o + 0x20);
    m.wv(o + 0x20, pos);
    let d = [pos[0] - old[0], pos[1] - old[1], pos[2] - old[2], pos[3] - old[3]];
    let own = m.r32(o + 8);
    if m.r32(own + 0x90) != o {
        return;
    }
    let r = m.rv(0x002a_2dd0);
    let eps = f32::from_bits(0x38d1_b717);
    let mut same = false;
    if m.call(0x0011_d4d0, &[], &[f4(r[0]), f4(d[0]), f4(eps)]).0 != 0
        && m.call(0x0011_d4d0, &[], &[f4(r[1]), f4(d[1]), f4(eps)]).0 != 0
    {
        same = m.call(0x0011_d4d0, &[], &[f4(r[2]), f4(d[2]), f4(eps)]).0 != 0;
    }
    if !same {
        let own = m.r32(o + 8);
        m.wv(own + 0xc0, pos);
        let a1 = m.r32(o + 8);
        let a0 = m.r32(o + 4);
        m.call(0x0023_0598, &[a0, a1], &[]);
    }
}

/// FUN_0022ccd0(out, _, mat, v): out = skew(v) (cross-product matrix) then out row i = skew * mat row i
/// (xyz; w words cleared). Returns `out`.
pub fn ee_sub_22ccd0<M: QMem + ?Sized>(m: &mut M, o: u32, _a1: u32, a2: u32, v: V4) -> u32 {
    m.call(0x0023_33b4, &[o, 0, 0x40], &[]);
    m.wf(o + 4, v[2]);
    m.wf(o + 0x20, v[1]);
    m.wf(o + 0x24, -v[0]);
    m.wf(o + 0x10, -v[2]);
    m.wf(o + 0x18, v[0]);
    m.wf(o + 8, -v[1]);
    let r1 = m.rv(o + 0x10);
    let m2 = m.rv(a2 + 0x20);
    let m0 = m.rv(a2);
    let r0 = m.rv(o);
    let r2 = m.rv(o + 0x20);
    let m1 = m.rv(a2 + 0x10);
    let mul = |q: V4| -> V4 {
        let mut z = q;
        for i in 0..3 {
            z[i] = r0[i] * q[0] + r1[i] * q[1] + r2[i] * q[2];
        }
        z
    };
    let n2 = mul(m2);
    let n0 = mul(m0);
    let n1 = mul(m1);
    m.wv(o + 0x20, n2);
    m.wv(o, n0);
    m.wv(o + 0x10, n1);
    m.w32(o + 0x2c, 0);
    m.w32(o + 0x1c, 0);
    m.w32(o + 0xc, 0);
    o
}

/// FUN_0012eb78(o, bis): frame-alloc scope; allocate +0xe8 (0xa0) / +0xf4 (0x80) scratch, FUN_00231170
/// load, then shrink them to +0xe4*0x14 / +0xf0*0x10 (FUN_001ae238).
pub fn ee_sub_12eb78<M: QMem + ?Sized>(m: &mut M, o: u32, bis: u32) {
    let sp = stack_push(m, 0x20);
    let s0 = sp + 0x10;
    m.call(0x001a_e478, &[sp], &[]);
    m.w32(s0, 0xa0);
    m.w32(s0 + 4, 0x10);
    let v = m.call(0x001a_e378, &[s0], &[]).0;
    m.w32(o + 0xe8, v);
    m.w32(s0, 0x80);
    m.w32(s0 + 4, 0x10);
    let v = m.call(0x001a_e378, &[s0], &[]).0;
    m.w32(o + 0xf4, v);
    m.call(0x0023_1170, &[o, bis], &[]);
    let n = m.r32(o + 0xe4).wrapping_mul(0x14);
    let p = m.r32(o + 0xe8);
    m.w32(s0, n);
    m.w32(s0 + 4, 0x10);
    let v = m.call(0x001a_e238, &[s0, p], &[]).0;
    m.w32(o + 0xe8, v);
    let n = m.r32(o + 0xf0) << 4;
    let p = m.r32(o + 0xf4);
    m.w32(s0, n);
    m.w32(s0 + 4, 0x10);
    let v = m.call(0x001a_e238, &[s0, p], &[]).0;
    m.w32(o + 0xf4, v);
    m.call(0x001a_e4a0, &[sp, 2], &[]);
    stack_pop(m, 0x20);
}

/// FUN_0011ccc0(pool): fixed-size pool alloc (+0 size, +4 count, +8 align, +0xc free list, +0x10 used,
/// +0x14 block); lazily builds the free list; returns a zeroed entry.
pub fn ee_sub_11ccc0<M: QMem + ?Sized>(m: &mut M, p: u32) -> u32 {
    if m.r32(p + 0xc) == 0 {
        let sp = stack_push(m, 0x10);
        let n = m.r32(p + 4).wrapping_mul(m.r32(p));
        let al = m.r32(p + 8);
        m.w32(sp + 4, al);
        m.w32(sp, n);
        let a3 = m.call(0x001a_e1d0, &[sp], &[]).0;
        stack_pop(m, 0x10);
        let mut a1 = 1u32;
        while (a1 as i32) < m.r32(p + 4) as i32 {
            let sz = m.r32(p);
            let nx = a1.wrapping_mul(sz).wrapping_add(a3);
            m.w32(a1.wrapping_sub(1).wrapping_mul(sz).wrapping_add(a3), nx);
            a1 = a1.wrapping_add(1);
        }
        let sz = m.r32(p);
        m.w32(a1.wrapping_sub(1).wrapping_mul(sz).wrapping_add(a3), 0);
        m.w32(p + 0x14, a3);
        m.w32(p + 0xc, a3);
    }
    let s0 = m.r32(p + 0xc);
    let sz = m.r32(p);
    let nx = m.r32(s0);
    m.w32(p + 0xc, nx);
    m.call(0x0023_33b4, &[s0, 0, sz], &[]);
    let u = m.r32(p + 0x10).wrapping_add(1);
    m.w32(p + 0x10, u);
    s0
}

/// FUN_0022cc00(out, kind, v): out = 0 matrix, then v.xyz scattered by the +0x94 kind
/// (1/4/5/9/10 -> row 3; 0xb -> [4],[8],[9]; 0xc -> [0],[5],[10]). Returns `out`.
pub fn ee_sub_22cc00<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32, v: V4) -> u32 {
    m.call(0x0023_33b4, &[o, 0, 0x40], &[]);
    let (a, b, c) = match m.r32(k + 0x94) {
        1 | 4 | 5 | 9 | 10 => (0x30, 0x34, 0x38),
        0xb => (0x10, 0x20, 0x24),
        0xc => (0x00, 0x14, 0x28),
        _ => return o,
    };
    m.wf(o + a, v[0]);
    m.wf(o + c, v[2]);
    m.wf(o + b, v[1]);
    o
}

/// FUN_0022ca18(kind, mat) -> qword: inverse of FUN_0022cc00 (6/7 -> FUN_0011bff0(mat)).
pub fn ee_sub_22ca18<M: QMem + ?Sized>(m: &mut M, k: u32, a1: u32) -> V4 {
    let (a, b, c) = match m.r32(k + 0x94) {
        1 | 4 | 5 | 9 | 10 => (0x30, 0x34, 0x38),
        6 | 7 => return m.call(0x0011_bff0, &[a1], &[]).1,
        0xb => (0x10, 0x20, 0x24),
        0xc => (0x00, 0x14, 0x28),
        _ => return [0.0; 4],
    };
    let x = m.rf(a1 + a);
    let y = m.rf(a1 + b);
    let z = m.rf(a1 + c);
    // pcpyld of the sign-extended mfc1: w = sign word of z
    let w = f32::from_bits(if (z.to_bits() as i32) < 0 { 0xffff_ffff } else { 0 });
    [x, y, z, w]
}

/// FUN_00135d80(jt): per +0x13d0 state (2 / 3 / 4) ask the sub-mode for an anim target (+out word);
/// fallback +0x268; FUN_00203d00(jt, target, out).
pub fn ee_sub_135d80<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let sp = stack_push(m, 0x10);
    m.w32(sp, 0);
    let mut v = match m.r32(o + 0x13d0) {
        3 => m.call(0x0019_44c8, &[o + 0x4c0, sp], &[]).0,
        2 => m.call(0x0019_c870, &[o + 0x2c0, sp], &[]).0,
        4 => m.call(0x001a_23f8, &[o + 0xdf0, sp], &[]).0,
        _ => 0,
    };
    if v == 0 {
        v = m.r32(o + 0x268);
    }
    let a2 = m.r32(sp);
    stack_pop(m, 0x10);
    m.call(0x0020_3d00, &[o, v, a2], &[]);
}

/// FUN_002282b0(o, bis): FUN_00227bb8; read +0x144, count -> +0x148, alloc +0x14c[count] and read each.
pub fn ee_sub_2282b0<M: QMem + ?Sized>(m: &mut M, o: u32, bis: u32) {
    m.call(0x0022_7bb8, &[o, bis], &[]);
    m.call(0x0022_8250, &[bis, o + 0x144], &[]);
    let n = m.call(0x0011_7bd0, &[bis], &[]).0;
    let sp = stack_push(m, 0x10);
    m.w32(sp + 4, 0x10);
    m.w32(sp, n << 2);
    m.w32(o + 0x148, n);
    let p = m.call(0x001a_e1d0, &[sp], &[]).0;
    stack_pop(m, 0x10);
    m.w32(o + 0x14c, p);
    let mut i = 0u32;
    if (m.r32(o + 0x148) as i32) > 0 {
        loop {
            let b = m.r32(o + 0x14c);
            m.call(0x0022_8250, &[bis, b.wrapping_add(i << 2)], &[]);
            i += 1;
            if (i as i32) >= m.r32(o + 0x148) as i32 {
                break;
            }
        }
    }
}

/// FUN_001358a8(jt, flags): stamp +0x1080 = t (state not 0 / 9 with +0x4cc); FUN_00141640; state 5 ->
/// *flags |= 1; state 0xd -> FUN_0019ea00(jt + 0xf50, flags).
pub fn ee_sub_1358a8<M: QMem + ?Sized>(m: &mut M, o: u32, fl: u32) {
    let s = m.r32(o + 0x13d0);
    if s != 0 && s != 9 && m.r32(o + 0x4cc) != 0 {
        let t = m.rf(0x002d_ded0);
        m.wf(o + 0x1080, t);
    }
    m.call(0x0014_1640, &[o, fl], &[]);
    match m.r32(o + 0x13d0) {
        5 => {
            let v = m.r32(fl) | 1;
            m.w32(fl, v);
        }
        0xd => {
            m.call(0x0019_ea00, &[o + 0xf50, fl], &[]);
        }
        _ => {}
    }
}

/// FUN_00210408(gs, on, id, arg, obj, ctx): obj = id != -1 ? FUN_0015e938(gs, 4, id, ctx) : obj; if it is
/// kind 0x30 -> on ? FUN_001f6978(obj, arg) : FUN_001f69f8(obj, arg).
pub fn ee_sub_210408<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, t0: u32, t1: u32) {
    let s0 = if a2 != 0xffff_ffff { m.call(0x0015_e938, &[a0, 4, a2, t1], &[]).0 } else { t0 };
    if s0 == 0 || m.call(0x0013_2b10, &[s0, 0x30], &[]).0 == 0 {
        return;
    }
    if a1 != 0 {
        m.call(0x001f_6978, &[s0, a3], &[]);
    } else {
        m.call(0x001f_69f8, &[s0, a3], &[]);
    }
}

/// FUN_001a21a0(o, msg): +0x18c8 state 3/8/0x15/0x16 accept msg 0x4000; 7 asks FUN_00194640; else
/// FUN_00140ed8(o, msg).
pub fn ee_sub_1a21a0<M: QMem + ?Sized>(m: &mut M, o: u32, a1: u32) -> u32 {
    match m.r32(o + 0x18c8) {
        3 | 8 | 0x15 | 0x16 => {
            if a1 == 0x4000 {
                return 1;
            }
        }
        7 => {
            if m.call(0x0019_4640, &[o + 0x4c0, a1], &[]).0 != 0 {
                return 1;
            }
        }
        _ => {}
    }
    m.call(0x0014_0ed8, &[o, a1], &[]).0
}

/// FUN_002035a8(o, bis): defaults (+0xe4 0, +0xe0 75, +0x104 5000, +0xe8 0.25); FUN_00231170 load;
/// +0xe4 == 0 -> +0xe0; +0xf8 == -2 -> +0xf4.
pub fn ee_sub_2035a8<M: QMem + ?Sized>(m: &mut M, o: u32, a1: u32) {
    m.w32(o + 0xe4, 0);
    m.w32(o + 0xe0, 0x4296_0000);
    m.w32(o + 0x104, 0x459c_4000);
    m.w32(o + 0xe8, 0x3e80_0000);
    m.call(0x0023_1170, &[o, a1], &[]);
    if m.rf(o + 0xe4) == 0.0 {
        let v = m.rf(o + 0xe0);
        m.wf(o + 0xe4, v);
    }
    if m.r32(o + 0xf8) == 0xffff_fffe {
        let v = m.r32(o + 0xf4);
        m.w32(o + 0xf8, v);
    }
}

/// FUN_00178d00(o): +0x3c set and the xy distance of FUN_001a62e8(+0x10) from +0xb0 is <= +0x64.
pub fn ee_sub_178d00<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    if m.r32(o + 0x3c) == 0 {
        return 0;
    }
    let a = m.r32(o + 0x10);
    let p = m.call(0x001a_62e8, &[a], &[]).1;
    let r = m.rf(o + 0x64);
    let q = m.rv(o + 0xb0);
    let dx = p[0] - q[0];
    let dy = p[1] - q[1];
    let d = (dx * dx + dy * dy).sqrt();
    (d <= r) as u32
}

/// FUN_00135f78(jt, msg): state 3 -> FUN_00194640(jt + 0x4c0, msg); state 0xd accepts 0x4000;
/// else FUN_00140ed8(jt, msg).
pub fn ee_sub_135f78<M: QMem + ?Sized>(m: &mut M, o: u32, a1: u32) -> u32 {
    match m.r32(o + 0x13d0) {
        3 => {
            if m.call(0x0019_4640, &[o + 0x4c0, a1], &[]).0 != 0 {
                return 1;
            }
        }
        0xd => {
            if a1 == 0x4000 {
                return 1;
            }
        }
        _ => {}
    }
    m.call(0x0014_0ed8, &[o, a1], &[]).0
}

/// FUN_00201e88(o): for each of +0x30 entries at +0x34[i], if entry +0x100 is set -> its slot 8.
pub fn ee_sub_201e88<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if (m.r32(o + 0x30) as i32) <= 0 {
        return;
    }
    let mut s0 = o + 0x34;
    let mut i = 0i32;
    loop {
        let e = m.r32(s0);
        let a1 = m.r32(e + 0x100);
        if a1 != 0 {
            vslot(m, a1, 8, &[], &[]);
        }
        i += 1;
        s0 += 4;
        if i >= m.r32(o + 0x30) as i32 {
            break;
        }
    }
}

/// FUN_0012b520(src, dst): init dst +0x80.. (count 2, 0 / 1 pair), copy src+4..+0x10 to dst +0xc0 and
/// +0xe0 (qword), clear +0xd0/+0xf0 xyz.
pub fn ee_sub_12b520<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.w16(a1 + 0x9a, 2);
    m.w32(a1 + 0x84, 0x3f80_0000);
    m.w32(a1 + 0x88, a1 + 0x80);
    m.w32(a1 + 0x8c, a1 + 0x84);
    m.w32(a1 + 0x80, 0);
    let mut b = [0u8; 8];
    for (k, x) in b.iter_mut().enumerate() {
        *x = m.r8(a0 + 4 + k as u32);
    }
    let v3 = m.r32(a0 + 0xc);
    for (k, x) in b.iter().enumerate() {
        m.w8(a1 + 0xc0 + k as u32, *x);
    }
    m.w32(a1 + 0xc8, v3);
    let q = m.rv(a1 + 0xc0);
    m.wv(a1 + 0xe0, q);
    m.w32(a1 + 0xd4, 0);
    m.w32(a1 + 0xd8, 0);
    m.w32(a1 + 0xd0, 0);
    m.w32(a1 + 0xf4, 0);
    m.w32(a1 + 0xf8, 0);
    m.w32(a1 + 0xf0, 0);
}

/// FUN_0022cb90(out, a1, mat, a, b): v = FUN_0022cad0(a, b); FUN_0022ccd0(tmp, a1, mat, v); copy to out.
pub fn ee_sub_22cb90<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, qa: V4, qb: V4) -> u32 {
    let v = m.call(0x0022_cad0, &[], &[qa, qb]).1;
    let sp = stack_push(m, 0x40);
    m.call(0x0022_ccd0, &[sp, a1, a2], &[v]);
    m.call(0x0023_320c, &[a0, sp, 0x40], &[]);
    stack_pop(m, 0x40);
    a0
}

/// FUN_0020b3d0(o): +0x2c0 (u64 flags) & bit(7) -> 2; & (bit(5) | bit(6)) -> 1; else 0.
pub fn ee_sub_20b3d0<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let b7 = m.call(0x0020_7a20, &[7], &[]).0 as u64;
    if m.r64(o + 0x2c0) & b7 != 0 {
        return 2;
    }
    let b5 = m.call(0x0020_7a20, &[5], &[]).0 as u64;
    let b6 = m.call(0x0020_7a20, &[6], &[]).0 as u64;
    (m.r64(o + 0x2c0) & (b5 | b6) != 0) as u32
}

/// FUN_00117d50(bis) -> i16: read a little-endian short from the stream (fast path from the buffer,
/// else FUN_00117968(bis, 2, &tmp)).
pub fn ee_sub_117d50<M: QMem + ?Sized>(m: &mut M, a3: u32) -> i32 {
    let n = m.r32(a3 + 8);
    if (n as i32) < 2 {
        let sp = stack_push(m, 0x10);
        m.call(0x0011_7968, &[a3, 2, sp], &[]);
        let v = m.r16(sp) as i16 as i32;
        stack_pop(m, 0x10);
        return v;
    }
    let p = m.r32(a3 + 4);
    let hi = m.r8(p + 1) as u32;
    let lo = m.r8(p) as u32;
    m.w32(a3 + 8, n.wrapping_sub(2));
    m.w32(a3 + 4, p.wrapping_add(2));
    (lo | (((hi << 24) as i32 >> 16) as u32)) as i32
}

/// FUN_00229738(o, e): unlink `e` from the +0x138 singly linked list (next at +8), then FUN_001f5078(o, e).
pub fn ee_sub_229738<M: QMem + ?Sized>(m: &mut M, o: u32, a1: u32) {
    let h = m.r32(o + 0x138);
    if h != 0 {
        if h == a1 {
            let n = m.r32(a1 + 8);
            m.w32(o + 0x138, n);
            m.w32(a1 + 8, 0);
        } else {
            let mut v1 = h;
            loop {
                let v0 = m.r32(v1 + 8);
                if v0 == 0 {
                    break;
                }
                if v0 == a1 {
                    let n = m.r32(a1 + 8);
                    m.w32(v1 + 8, n);
                    m.w32(a1 + 8, 0);
                    break;
                }
                v1 = v0;
            }
        }
    }
    m.call(0x001f_5078, &[o, a1], &[]);
}

/// FUN_001e66c8(path, t): looped paths (+0xe4) with t outside [0, last knot] -> FUN_0011d898 wrap.
pub fn ee_sub_1e66c8<M: QMem + ?Sized>(m: &mut M, o: u32, t: f32) -> f32 {
    if m.r32(o + 0xe4) == 0 {
        return t;
    }
    let n = m.r32(o + 0xe8);
    let b = m.r32(o + 0xf0);
    let last = m.rf((n << 2).wrapping_add(b).wrapping_sub(4));
    if 0.0 <= t && t <= last {
        return t;
    }
    m.call(0x0011_d898, &[o], &[f4(t)]).1[0]
}

/// FUN_00139318(o): FUN_0011a118(FUN_0015e3a8(GS, 0x1c), o); FUN_0011cd98(0x3e1500, o).
pub fn ee_sub_139318<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let l = m.call(0x0015_e3a8, &[0x003d_d490, 0x1c], &[]).0;
    m.call(0x0011_a118, &[l, o], &[]);
    m.call(0x0011_cd98, &[0x003e_1500, o], &[]);
}

/// FUN_00154180(o, key): find the 0x10-stride entry at +0x100 (count +0xfc) whose first word is `key`.
pub fn ee_sub_154180<M: QMem + ?Sized>(m: &mut M, o: u32, a1: u32) -> u32 {
    if a1 == 0 {
        return 0;
    }
    let n = m.r32(o + 0xfc) as i32;
    let mut e = o + 0x100;
    let mut i = 0i32;
    while i < n {
        if m.r32(e) == a1 {
            return e;
        }
        i += 1;
        e += 0x10;
    }
    0
}

/// FUN_00139900(o, tgt): FUN_001333c0(o, tgt); tgt kind 0x1d -> +0xe0 = tgt, +0x38 = tgt +0x38.
pub fn ee_sub_139900<M: QMem + ?Sized>(m: &mut M, o: u32, a1: u32) {
    m.call(0x0013_33c0, &[o, a1], &[]);
    if m.call(0x0013_2b10, &[a1, 0x1d], &[]).0 != 0 {
        m.w32(o + 0xe0, a1);
        let v = m.rf(a1 + 0x38);
        m.wf(o + 0x38, v);
    }
}

/// FUN_00221388(o): FUN_001a5e28(o); +0x13c -> FUN_00149a50(GS, o, *0x2a96f0, 0).
pub fn ee_sub_221388<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001a_5e28, &[o], &[]);
    if m.r32(o + 0x13c) != 0 {
        let a2 = m.r32(0x002a_96f0);
        m.call(0x0014_9a50, &[0x003d_d490, o, a2, 0], &[]);
    }
}

/// FUN_001bc088(o): FUN_001e5df8(o); +0x1ec = alloc(+0xe8 * 0x10 - 0x10, align 0x10).
pub fn ee_sub_1bc088<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_5df8, &[o], &[]);
    let sp = stack_push(m, 0x10);
    let n = (m.r32(o + 0xe8) << 4).wrapping_sub(0x10);
    m.w32(sp + 4, 0x10);
    m.w32(sp, n);
    let p = m.call(0x001a_e2a0, &[sp], &[]).0;
    stack_pop(m, 0x10);
    m.w32(o + 0x1ec, p);
}

/// FUN_00151bf0(o, a1, a2): FUN_0014fcc8(o, a1, a2); +0x10 = a2; +0x14 = a1.
pub fn ee_sub_151bf0<M: QMem + ?Sized>(m: &mut M, o: u32, a1: u32, a2: u32) {
    m.call(0x0014_fcc8, &[o, a1, a2], &[]);
    m.w32(o + 0x10, a2);
    m.w32(o + 0x14, a1);
}

/// FUN_0021ecd8(a0, fmt, ...): spill the variadic GPRs (a2..t3, 64-bit slots) and f12/f14/f16/f18,
/// then FUN_0021ec60(a0, fmt, &va).
pub fn ee_sub_21ecd8<M: QMem + ?Sized>(
    m: &mut M,
    a0: u32,
    a1: u32,
    a2: u32,
    a3: u32,
    t0: u32,
    t1: u32,
    t2: u32,
    t3: u32,
    f12: f32,
    f14: f32,
    f16: f32,
    f18: f32,
) {
    let sp = stack_push(m, 0x40);
    for (k, r) in [a2, a3, t0, t1, t2, t3].iter().enumerate() {
        let p = sp + 0x10 + 8 * k as u32;
        m.w32(p, *r);
        m.w32(p + 4, ((*r as i32) >> 31) as u32);
    }
    m.wf(sp, f12);
    m.wf(sp + 4, f14);
    m.wf(sp + 8, f16);
    m.wf(sp + 0xc, f18);
    m.call(0x0021_ec60, &[a0, a1, sp + 0x10], &[]);
    stack_pop(m, 0x40);
}

/// FUN_001fb018(o): ++(s16)+0x4e == 1 -> FUN_001fb658(o, 6, 2).
pub fn ee_sub_1fb018<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let v = m.r16(o + 0x4e).wrapping_add(1);
    m.w16(o + 0x4e, v);
    if v as i16 == 1 {
        m.call(0x001f_b658, &[o, 6, 2], &[]);
    }
}

/// FUN_00224068(o): FUN_001f2d48(o); slot 75(-1); FUN_001f15b8(o + 0x168).
pub fn ee_sub_224068<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001f_2d48, &[o], &[]);
    vslot(m, o, 75, &[0xffff_ffff], &[]);
    m.call(0x001f_15b8, &[o + 0x168], &[]);
}

/// FUN_0018d568(o, k): k < 9 -> FUN_0018c250(o, [0,2,7,4,0x12,0x13,0x14,0x17,8][k], -1).
pub fn ee_sub_18d568<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32) {
    const T: [u32; 9] = [0, 2, 7, 4, 0x12, 0x13, 0x14, 0x17, 8];
    if k < 9 {
        m.call(0x0018_c250, &[o, T[k as usize], 0xffff_ffff], &[]);
    }
}

/// FUN_00135c20(o, k): k < 9 -> FUN_00135248(o, [0,2,3,4,9,10,11,12,13][k]).
pub fn ee_sub_135c20<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32) {
    const T: [u32; 9] = [0, 2, 3, 4, 9, 10, 11, 12, 13];
    if k < 9 {
        m.call(0x0013_5248, &[o, T[k as usize]], &[]);
    }
}

/// FUN_001fe848(o, id): +0x200 set and (id == -1 or its +0x60 -> +0x40 == id).
pub fn ee_sub_1fe848<M: QMem + ?Sized>(m: &mut M, o: u32, a1: u32) -> u32 {
    let p = m.r32(o + 0x200);
    if p == 0 {
        return 0;
    }
    let v1 = m.r32(p + 0x60);
    if a1 == 0xffff_ffff {
        return 1;
    }
    (m.r32(v1 + 0x40) == a1) as u32
}

/// FUN_002041c0(o): FUN_001f40a0(o); +0x230 = FUN_001a6f00(o, 0, +0x240 qword).
pub fn ee_sub_2041c0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001f_40a0, &[o], &[]);
    let q = m.rv(o + 0x240);
    let r = m.call(0x001a_6f00, &[o, 0], &[q]).1;
    m.wv(o + 0x230, r);
}

/// FUN_001d0158(o): +0x80 = +0x70; +0x70 = FUN_001a62e8(+0xf0).
pub fn ee_sub_1d0158<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let v = m.rv(o + 0x70);
    let a = m.r32(o + 0xf0);
    m.wv(o + 0x80, v);
    let r = m.call(0x001a_62e8, &[a], &[]).1;
    m.wv(o + 0x70, r);
}

/// FUN_0021d428(o): FUN_001a6150(o); FUN_0011a2e8(0x3e1474, o).
pub fn ee_sub_21d428<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001a_6150, &[o], &[]);
    m.call(0x0011_a2e8, &[0x003e_1474, o], &[]);
}

/// FUN_001bad80(o): FUN_001f3e00(o); FUN_0011a2e8(0x3e1450, o).
pub fn ee_sub_1bad80<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001f_3e00, &[o], &[]);
    m.call(0x0011_a2e8, &[0x003e_1450, o], &[]);
}

/// FUN_00182ef0(o): FUN_001a6150(o); FUN_0011a2e8(0x3e13f0, o).
pub fn ee_sub_182ef0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001a_6150, &[o], &[]);
    m.call(0x0011_a2e8, &[0x003e_13f0, o], &[]);
}

/// FUN_00171fa0(o): FUN_001eef70(o); FUN_00172320(o, 0).
pub fn ee_sub_171fa0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef70, &[o], &[]);
    m.call(0x0017_2320, &[o, 0], &[]);
}

/// FUN_002213d8(o): FUN_001a6150(o); FUN_00221178(o, 0).
pub fn ee_sub_2213d8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001a_6150, &[o], &[]);
    m.call(0x0022_1178, &[o, 0], &[]);
}

/// FUN_00209818(o): +4 == 0 -> FUN_00204660(+0xc).
pub fn ee_sub_209818<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.r32(o + 4) == 0 {
        let a = m.r32(o + 0xc);
        m.call(0x0020_4660, &[a], &[]);
    }
}

/// FUN_0017f110(o): FUN_001f2d48(o); +0x18c = 0.
pub fn ee_sub_17f110<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001f_2d48, &[o], &[]);
    m.w32(o + 0x18c, 0);
}

/// FUN_00249cb8(o): FUN_0023dd30(+8, +8); returns 0.
pub fn ee_sub_249cb8<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let a = m.r32(o + 8);
    m.call(0x0023_dd30, &[a, a], &[]);
    0
}

/// FUN_00205458(o): FUN_001eeee0(o + 0x26c).
pub fn ee_sub_205458<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_eee0, &[o + 0x26c], &[]);
}

/// FUN_001bea88(o, bis): FUN_00231170(o, bis).
pub fn ee_sub_1bea88<M: QMem + ?Sized>(m: &mut M, o: u32, a1: u32) {
    m.call(0x0023_1170, &[o, a1], &[]);
}

/// FUN_001a6120: returns the qword at 0x2a2dd0.
pub fn ee_sub_1a6120<M: QMem + ?Sized>(m: &mut M) -> V4 {
    m.rv(0x002a_2dd0)
}

/// FUN_001a6138: returns the qword at 0x2a2dd0.
pub fn ee_sub_1a6138<M: QMem + ?Sized>(m: &mut M) -> V4 {
    m.rv(0x002a_2dd0)
}

/// FUN_001eda50: empty.
pub fn ee_sub_1eda50<M: QMem + ?Sized>(_m: &mut M) {}

/// FUN_0014fcc0: empty.
pub fn ee_sub_14fcc0<M: QMem + ?Sized>(_m: &mut M) {}

/// FUN_00182cc0: returns 0.
pub fn ee_sub_182cc0<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0
}

/// FUN_001ef0b8: returns 0.
pub fn ee_sub_1ef0b8<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0
}

/// FUN_001c3ed0: returns 0.
pub fn ee_sub_1c3ed0<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0
}
