//! WIRE-S2: 1:1 QMem ports of live-frontier natives (shard 3/4).
#![allow(unused)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

fn cpq<M: QMem + ?Sized>(m: &mut M, dst: u32, src: u32) {
    for k in (0..16).step_by(4) {
        let w = m.r32(src + k);
        m.w32(dst + k, w);
    }
}

/// FUN_001660f8: FUN_001a4e08(0x2e1e88).
pub fn ee_sub_1660f8<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x001a_4e08, &[0x002e_1e88], &[]);
}

// SKIP 0x17f5b0: passes a 64-bit (ldl/ldr) by-value arg in a2 to FUN_00149a50; not representable as u32.

/// FUN_0016f8b8: FUN_0021f060(0x2dfb40); flag=1; GS CSR |= 2; FUN_00240aa0(FUN_00240b68() & ~0x200);
/// FUN_00234ca0(0); FUN_00241728(a0, a1); FUN_00234a60(*0x2dfb50); FUN_0021f0c0(0x2dfb40).
pub fn ee_sub_16f8b8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let s0 = 0x002d_fb40;
    m.call(0x0021_f060, &[s0], &[]);
    m.w32(0x002d_fb54, 1);
    let c = m.r32(0x1200_1000);
    m.w32(0x1200_1000, c | 2);
    let v = m.call(0x0024_0b68, &[], &[]).0;
    m.call(0x0024_0aa0, &[v & 0xffff_fdff], &[]);
    m.call(0x0023_4ca0, &[0], &[]);
    m.call(0x0024_1728, &[a0, a1], &[]);
    let x = m.r32(0x002d_fb50);
    m.call(0x0023_4a60, &[x], &[]);
    m.call(0x0021_f0c0, &[s0], &[]);
}

// SKIP 0x21e0d8: soft-float double math (64-bit register args to __muldf3 etc.); not representable as u32.

// SKIP 0x1d4050: passes a 64-bit (ldl/ldr) by-value arg in a2 to FUN_00149ab8; not representable as u32.

/// FUN_00127838(a0, a1): dispatch on a1 node kind (+8) / sub-kind (+0xc->+8) to FUN_00126xxx builders; returns a0.
pub fn ee_sub_127838<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let k = m.r32(a1 + 8);
    if k == 0x18 {
        m.call(0x0012_6438, &[a0], &[]);
        return a0;
    }
    if k != 0x1f {
        // b 0x12797c with a1 unchanged
        m.call(0x0012_91a8, &[a0, a1.wrapping_add(8)], &[]);
        return a0;
    }
    let a3 = m.r32(a1 + 0xc);
    let s = m.r32(a3 + 8).wrapping_sub(1);
    let f = match s {
        0 => 0x0012_6490,
        1 => 0x0012_6540,
        4 => 0x0012_65f0,
        5 => 0x0012_66c8,
        6 => 0x0012_6758,
        7 => 0x0012_6840,
        8 => 0x0012_6928,
        9 => 0x0012_6a18,
        11 => 0x0012_6ba0,
        12 => 0x0012_6db8,
        13 => 0x0012_6f78,
        14 => 0x0012_7068,
        15 => 0x0012_7228,
        16 => {
            m.call(0x0012_a0b0, &[a0, 0xffff_ffff], &[]);
            return a0;
        }
        17 => {
            let p = m.r32(a3 + 0x10);
            m.call(0x0012_91a8, &[a0, p.wrapping_add(8)], &[]);
            return a0;
        }
        _ => 0x0012_72e0,
    };
    m.call(f, &[a0], &[]);
    a0
}

/// FUN_001f3980(o): FUN_001f3900(o); tick active parts (FUN_001f6ef8(part, dt)); sync anchors +0x58/+0x5c into
/// +0xe8; reset their motion if FUN_00230380 says so and call vslot 69; FUN_0022de88(o); FUN_001f38a8 on parts
/// lacking flag bit 50 of +0x10.
pub fn ee_sub_1f3980<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let s0 = a0;
    m.call(0x001f_3900, &[s0], &[]);
    let mut i: i32 = 0;
    let mut off: u32 = 0;
    while i < m.r32(s0 + 0xe0) as i32 {
        let p = m.r32(s0 + 0xe4).wrapping_add(off);
        if m.r32(p + 0x10) != 0 {
            let dt = m.rf(0x002d_ded4);
            m.call(0x001f_6ef8, &[p], &[f4(dt)]);
        }
        i += 1;
        off = off.wrapping_add(0xb0);
    }
    let e8 = m.r32(s0 + 0xe8);
    if e8 != 0 {
        let p58 = m.r32(s0 + 0x58);
        if p58 != 0 {
            cpq(m, e8 + 0x60, p58 + 0xb0);
        }
        let p5c = m.r32(s0 + 0x5c);
        if p5c != 0 {
            let e8 = m.r32(s0 + 0xe8);
            cpq(m, e8 + 0x70, p5c + 0xb0);
        }
        let v1 = m.r32(s0 + 0x58);
        let mut go = (m.r16(v1 + 0x98) as u32).wrapping_sub(1) < 2;
        if !go {
            let p5c = m.r32(s0 + 0x5c);
            let h = m.r16(p5c + 0x98) as i16 as i32;
            go = h == 1 || h == 2;
        }
        if go {
            let h = m.r16(v1 + 0x98) as i16 as i32 as u32;
            if m.call(0x0023_0380, &[h], &[]).0 != 0 {
                let q = m.r32(s0 + 0x58);
                m.w32(q + 0xf4, 0);
                m.w32(q + 0xf8, 0);
                m.w32(q + 0xf0, 0);
            }
            let p5c = m.r32(s0 + 0x5c);
            let h = m.r16(p5c + 0x98) as i16 as i32 as u32;
            if m.call(0x0023_0380, &[h], &[]).0 != 0 {
                let q = m.r32(s0 + 0x5c);
                m.w32(q + 0xf4, 0);
                m.w32(q + 0xf8, 0);
                m.w32(q + 0xf0, 0);
            }
            vslot(m, s0, 69, &[], &[]);
        }
    }
    m.call(0x0022_de88, &[s0], &[]);
    let mut i: i32 = 0;
    let mut off: u32 = 0;
    while i < m.r32(s0 + 0xe0) as i32 {
        let p = m.r32(s0 + 0xe4).wrapping_add(off);
        if m.r32(p + 0x14) & (1 << 18) == 0 {
            m.call(0x001f_38a8, &[p], &[]);
        }
        i += 1;
        off = off.wrapping_add(0xb0);
    }
}

/// FUN_001f9c48(o): integrate anchors +0x58 (position by -dt/2 * vel) and +0x5c (rotation by -dt/2 * angvel),
/// bump frame stamps, zero motion, vslot 17, FUN_001f3900, FUN_001f38a8 on the +0xec chain, FUN_001f4d50.
pub fn ee_sub_1f9c48<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let s6 = a0;
    let sp = stack_push(m, 0x160);
    let s0 = m.r32(s6 + 0x58);
    let s1 = m.r32(s6 + 0x5c);
    let f20: f32 = -0.5;
    m.call(0x0023_0630, &[s6, s0], &[]);
    let s4 = s0 + 0xb0;
    m.call(0x0023_0630, &[s6, s1], &[]);
    let s5 = s1 + 0xb0;
    let t = 0x002d_ded0u32;
    let s3 = 0x003d_d490u32;
    // vf04 = (+0x30 + vf00) + (+0xb0) * (dt * -0.5)
    let k = m.rf(t + 4) * f20;
    m.w32(sp, k.to_bits());
    let p = s0 + 0x30;
    let mut v: V4 = [0.0; 4];
    for j in 0..4u32 {
        let a = m.rf(p + 4 * j) + if j == 3 { 1.0 } else { 0.0 };
        v[j as usize] = a + m.rf(s0 + 0xb0 + 4 * j) * k;
    }
    m.call(0x0025_6d18, &[p], &[v]);
    m.wf(s0 + 0x3c, 1.0);
    let c = m.r32(s3 + 0x3d58).wrapping_add(1);
    m.w32(s3 + 0x3d58, c);
    m.w32(s0 + 0xa4, c);
    m.w32(s4 + 4, 0);
    m.w32(s4 + 8, 0);
    m.w32(s0 + 0xb0, 0);
    let k = m.rf(t + 4) * f20;
    m.w32(sp, k.to_bits());
    let mut w: V4 = [0.0; 4];
    for j in 0..4u32 {
        w[j as usize] = m.rf(s1 + 0xb0 + 4 * j) * k;
    }
    m.call(0x0011_b570, &[sp + 0x70], &[w]);
    let r0: V4 = m.rv(sp + 0x70);
    let r1: V4 = m.rv(sp + 0x80);
    let r2: V4 = m.rv(sp + 0x90);
    let mut rows = [[0u32; 4]; 3];
    for i in 0..3u32 {
        let x = m.rf(s1 + 0x10 * i);
        let y = m.rf(s1 + 0x10 * i + 4);
        let z = m.rf(s1 + 0x10 * i + 8);
        let ww = m.r32(s1 + 0x10 * i + 0xc);
        for c in 0..3 {
            rows[i as usize][c] = (r0[c] * x + r1[c] * y + r2[c] * z).to_bits();
        }
        rows[i as usize][3] = ww;
    }
    let put = |m: &mut M, a: u32, r: &[u32; 4]| {
        for c in 0..4u32 {
            m.w32(a + 4 * c, r[c as usize]);
        }
    };
    put(m, sp + 0x40, &rows[0]);
    put(m, sp + 0xa0, &rows[0]);
    put(m, sp + 0x50, &rows[1]);
    put(m, sp + 0xb0, &rows[1]);
    put(m, sp + 0x10, &rows[0]);
    put(m, sp + 0x20, &rows[1]);
    put(m, sp + 0x30, &rows[2]);
    put(m, sp + 0xc0, &rows[2]);
    put(m, sp + 0x60, &rows[2]);
    m.call(0x0011_b3d0, &[sp + 0x10], &[]);
    cpq(m, s1, sp + 0x10);
    cpq(m, s1 + 0x10, sp + 0x20);
    cpq(m, s1 + 0x20, sp + 0x30);
    m.w32(s1 + 0x1c, 0);
    m.w32(s1 + 0x2c, 0);
    m.w32(s1 + 0xc, 0);
    let c = m.r32(s3 + 0x3d58).wrapping_add(1);
    m.w32(s3 + 0x3d58, c);
    m.w32(s1 + 0xa4, c);
    m.w32(s5 + 4, 0);
    m.w32(s5 + 8, 0);
    m.w32(s1 + 0xb0, 0);
    vslot(m, s6, 17, &[], &[]);
    m.call(0x001f_3900, &[s6], &[]);
    let a = m.r32(s6 + 0xec);
    if a != 0 {
        let mut p = m.r32(a + 0x34);
        while p != 0 {
            if m.r32(p + 0x14) & (1 << 18) == 0 {
                m.call(0x001f_38a8, &[p], &[]);
            }
            p = m.r32(p + 0x60);
        }
    }
    m.call(0x001f_4d50, &[s6], &[]);
    stack_pop(m, 0x160);
}

/// FUN_00167320(o, owner, stream): load a curve/track set from a binary stream: header, two pointer tables,
/// then +0x38 entries (0x14 / 0x28 / 0x18 strided) each dispatched by kind to FUN_0016a000/16e438/16caa0/16cf78;
/// finally FUN_00167870(o, stream).
pub fn ee_sub_167320<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    let sp = stack_push(m, 0x1d0);
    let s4 = a0;
    let s2 = a2;
    m.w32(sp + 0x110, a1);
    m.w32(s4, a1);
    let has = m.call(0x0011_7d00, &[s2], &[]).0;
    m.w32(sp + 0x114, has);
    m.call(0x0011_7eb0, &[s2, s4 + 0x10], &[]);
    let f = m.call(0x0011_7e38, &[s2], &[]).1[0];
    m.wf(s4 + 0x20, f);
    let n = m.call(0x0011_7bd0, &[s2], &[]).0;
    m.w32(s4 + 0x24, n);
    m.w32(sp, n << 2);
    m.w32(sp + 4, 0x10);
    let v = m.call(0x001a_e2a0, &[sp], &[]).0;
    m.w32(s4 + 0x28, v);
    let mut i: i32 = 0;
    while i < m.r32(s4 + 0x24) as i32 {
        let v = m.call(0x001e_d7a8, &[1, s2], &[]).0;
        let b = m.r32(s4 + 0x28);
        m.w32(b.wrapping_add((i as u32) << 2), v);
        i += 1;
    }
    let n = m.call(0x0011_7bd0, &[s2], &[]).0;
    m.w32(s4 + 0x2c, n);
    m.w32(sp, n << 2);
    m.w32(sp + 4, 0x10);
    let v = m.call(0x001a_e2a0, &[sp], &[]).0;
    m.w32(s4 + 0x30, v);
    m.w32(s4 + 0x34, 0);
    let mut i: i32 = 0;
    while i < m.r32(s4 + 0x2c) as i32 {
        let v = m.call(0x0011_7db8, &[s2], &[]).0;
        let b = m.r32(s4 + 0x30);
        m.w32(b.wrapping_add((i as u32) << 2), v);
        i += 1;
    }
    let n = m.call(0x0011_7d50, &[s2], &[]).0;
    m.w32(s4 + 0x38, n);
    m.w32(sp + 4, 0x10);
    m.w32(sp, n.wrapping_mul(0x14));
    let v = m.call(0x001a_e2a0, &[sp], &[]).0;
    m.w32(s4 + 0x3c, v);
    let n = m.r32(s4 + 0x38);
    m.w32(sp + 4, 0x10);
    m.w32(sp, n.wrapping_mul(0x28));
    let v = m.call(0x001a_e2a0, &[sp], &[]).0;
    m.w32(s4 + 0x40, v);
    if m.r32(sp + 0x114) == 0 {
        let n = m.r32(s4 + 0x38);
        m.w32(sp + 4, 0x10);
        m.w32(sp, n.wrapping_mul(0x18));
        let v = m.call(0x001a_e2a0, &[sp], &[]).0;
        m.w32(s4 + 0x44, v);
    }
    let fmax = f32::from_bits(m.r32(0x002a_5270));
    let mut s7: i32 = 0;
    if (m.r32(s4 + 0x38) as i32) > 0 {
        m.w32(sp + 0x118, sp + 0x10);
        loop {
            'body: {
                let s3 = m.r32(s4 + 0x3c).wrapping_add((s7 as u32).wrapping_mul(0x14));
                let s5 = m.r32(s4 + 0x40).wrapping_add((s7 as u32).wrapping_mul(0x28));
                let t0 = m.r32(s4 + 0x44);
                let s6 = if t0 != 0 { (s7 as u32).wrapping_mul(0x18).wrapping_add(t0) } else { 0 };
                m.call(0x0023_33b4, &[sp, 0, 0xc], &[]);
                let s0 = m.call(0x0011_7d00, &[s2], &[]).0;
                if s0 != 0 {
                    let buf = m.r32(sp + 0x118);
                    m.call(0x0011_7968, &[s2, s0 << 1, buf], &[]);
                    let s1 = m.call(0x0011_7db8, &[s2], &[]).0;
                    let buf = m.r32(sp + 0x118);
                    if m.call(0x0013_fad0, &[s0, buf], &[]).0 == 0 {
                        m.call(0x0011_79b8, &[s2, s1], &[]);
                        s7 -= 1;
                        let c = m.r32(s4 + 0x38).wrapping_sub(1);
                        m.w32(s4 + 0x38, c);
                        break 'body;
                    }
                }
                let v = m.call(0x0011_7d00, &[s2], &[]).0;
                m.w8(s3 + 3, v as u8);
                let fp = m.call(0x0011_7c20, &[s2], &[]).0;
                let own = m.r32(sp + 0x110);
                let k = m.call(0x0011_7d50, &[s2], &[]).0;
                let r = vslot(m, own, 50, &[k], &[]).0;
                m.w32(s5 + 4, r);
                let v = m.call(0x0011_7d00, &[s2], &[]).0;
                m.w8(s3, v as u8);
                let f1 = m.call(0x0011_7e38, &[s2], &[]).1[0];
                let f0 = f32::from_bits(m.r32(0x002a_5270));
                if f1 == f0 {
                    m.wf(s3 + 4, f0);
                } else {
                    m.wf(s3 + 4, f1 * f1.abs());
                }
                let f = m.call(0x0011_7e38, &[s2], &[]).1[0];
                m.wf(sp, f);
                let v = m.call(0x0011_7db8, &[s2], &[]).0;
                m.w32(s5, v);
                let v = m.call(0x0011_7d00, &[s2], &[]).0;
                m.w32(sp + 8, v);
                let v = m.call(0x0011_7d00, &[s2], &[]).0;
                let w = (m.r32(s5 + 0xc) & 0xffff_f7ff) | ((v & 1) << 11);
                m.w32(s5 + 0xc, w);
                if m.r32(sp + 0x114) == 0 {
                    m.call(0x0011_7eb0, &[s2, s6], &[]);
                    let f = m.call(0x0011_7e38, &[s2], &[]).1[0];
                    m.wf(s6 + 0xc, f);
                    let v = m.call(0x0011_7c80, &[s2], &[]).0;
                    m.w32(s6 + 0x14, v);
                    let f = m.call(0x0011_7e38, &[s2], &[]).1[0];
                    m.wf(s6 + 0x10, f);
                    if f == fmax {
                        let kd = m.r8(s3 + 3) as i8 as i32;
                        if kd == 1 {
                            m.wf(s6 + 0x10, f32::from_bits(0x44fa_0000));
                        } else if kd == 0 || kd == 2 || kd == 3 {
                            m.wf(s6 + 0x10, f32::from_bits(0x5015_02f9));
                        }
                    }
                }
                let kd = m.r8(s3 + 3) as i8 as i32 as u32;
                let w = (m.r32(s5 + 0xc) & 0xffff_fff8) | (kd & 7);
                m.w32(s5 + 0xc, w);
                let b0 = m.r8(s3) as i8 as i32 as u32;
                let w = (w & 0xffff_ff07) | ((b0 & 0x1f) << 3);
                m.w32(s5 + 0xc, w);
                if m.rf(s3 + 4) != fmax {
                    m.w32(s5 + 0xc, w | 0x400);
                }
                let kd = m.r8(s3 + 3) as i8 as i32;
                let f = match kd {
                    0 => Some(0x0016_a000),
                    1 => Some(0x0016_e438),
                    2 => Some(0x0016_caa0),
                    3 => Some(0x0016_cf78),
                    _ => None,
                };
                if let Some(f) = f {
                    m.call(f, &[s4, s3, s5, sp, s6, fp, s2], &[]);
                }
                s7 = s7.wrapping_add(fp as i32);
            }
            s7 += 1;
            if s7 >= m.r32(s4 + 0x38) as i32 {
                break;
            }
        }
    }
    m.call(0x0016_7870, &[s4, s2], &[]);
    stack_pop(m, 0x1d0);
}

// SKIP 0x245b58: libm powf wrapper doing soft-float double arithmetic (64-bit register/stack doubles,
// struct exception); not representable with u32 args.
