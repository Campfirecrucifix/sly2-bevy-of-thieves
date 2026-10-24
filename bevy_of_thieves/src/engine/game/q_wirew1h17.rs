//! WIRE-W1H17: 1:1 queue ports (against `QMem`) of a batch of large EE functions, ported from the asm;
//! callees go through `m.call` / `vslot`. 64-bit GPR args: trailing ones are passed as two u32 words
//! (lo, hi) like q_002155c0; a non-trailing one is passed as a [lo, hi] bit quadword in `v`. `v` carries
//! quadword (128-bit GPR) args in register order, then float args (f12..); quadword results are `.1`.
#![allow(unused_variables, unused_mut, unused_assignments, clippy::all)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

fn fb(b: u32) -> f32 {
    f32::from_bits(b)
}

/// 64-bit GPR (`lw` / `mfc1`, sign-extended) moved to a quadword: upper 64 bits modelled as 0
fn gq(lo: u32) -> V4 {
    let hi = if lo & 0x8000_0000 != 0 { u32::MAX } else { 0 };
    [fb(lo), fb(hi), 0.0, 0.0]
}

fn gqf(x: f32) -> V4 {
    gq(x.to_bits())
}

fn add(a: V4, b: V4) -> V4 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2], a[3] + b[3]]
}

fn sub(a: V4, b: V4) -> V4 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2], a[3] - b[3]]
}

fn scl(a: V4, s: f32) -> V4 {
    [a[0] * s, a[1] * s, a[2] * s, a[3] * s]
}

/// vmul.xyz t,a,a ; vaddw.x ; vadday.x ; vmaddz.x -> |a.xyz|^2
fn dot3(a: V4) -> f32 {
    a[0] * a[0] + a[1] * a[1] + a[2] * a[2]
}

fn dotp(a: V4, b: V4) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// vopmula.xyz ACC,a,b ; vopmsub.xyz d,b,a -> d.xyz = a x b, w = d.w
fn cross(d: V4, a: V4, b: V4) -> V4 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0], d[3]]
}

/// vmulax/vmadday/vmaddz.xyz d, (r0,r1,r2), s -> xyz = r0*s.x + r1*s.y + r2*s.z, w = d.w
fn mx3(d: V4, r0: V4, r1: V4, r2: V4, s: V4) -> V4 {
    let mut o = d;
    for i in 0..3 {
        o[i] = r0[i] * s[0] + r1[i] * s[1] + r2[i] * s[2];
    }
    o
}

const VF0: V4 = [0.0, 0.0, 0.0, 1.0];
fn eps() -> f32 {
    fb(0x38d1_b717) // 1e-4
}

/// FUN_002149d8(o, s, a2, _, t0 = table): level-chunk loader. id = FUN_00117c20(s); e = table + id*0x40;
/// e+8 = a2, e[0] = id, e+4 = kind (FUN_00117c80); per kind (jump table 0x2a8b10) parse the stream and
/// build the entry (1: glyph DMA lists, 2: 0x18-byte records, 3: params, 4: u16, 6/7: GS sprite packet,
/// 8: two arrays via FUN_00214398 with 0x2c40a4 set). Scratch frame FUN_001ae478 / FUN_001ae4a0(sp, 2).
pub fn ee_sub_2149d8<M: QMem + ?Sized>(m: &mut M, o: u32, s: u32, a2: u32, a3: u32, t0: u32) {
    let sp = stack_push(m, 0x1b0);
    m.w32(sp + 0xe8, o);
    m.call(0x001a_e478, &[sp], &[]);
    let id = m.call(0x0011_7c20, &[s], &[]).0;
    let e = t0.wrapping_add(id << 6);
    m.w32(e + 8, a2);
    m.w32(e, id);
    let kind = m.call(0x0011_7c80, &[s], &[]).0;
    m.w32(e + 4, kind);
    match kind {
        4 => {
            let v = m.call(0x0011_7c20, &[s], &[]).0;
            m.w32(e + 0x10, v);
        }
        8 => {
            m.w32(0x002c_40a4, 1);
            m.call(0x0021_4398, &[o, s, a2, e + 0x10, e + 0x14], &[]);
            let n = m.call(0x0011_7c80, &[s], &[]).0;
            m.w32(e + 0x20, n);
            let h = m.r32(o + 0xc4);
            let p = m.call(0x0011_6918, &[h, n << 2, 0x13], &[]).0;
            m.w32(e + 0x24, p);
            if (m.r32(e + 0x20) as i32) > 0 {
                let mut i: u32 = 0;
                loop {
                    let v = m.call(0x0011_7c80, &[s], &[]).0;
                    let b = m.r32(e + 0x24);
                    m.w32((i << 2).wrapping_add(b), v);
                    i = i.wrapping_add(1);
                    if !((i as i32) < (m.r32(e + 0x20) as i32)) {
                        break;
                    }
                }
            }
            let n = m.call(0x0011_7c80, &[s], &[]).0;
            m.w32(e + 0x18, n);
            let h = m.r32(o + 0xc4);
            let p = m.call(0x0011_6918, &[h, n << 2, 0x13], &[]).0;
            m.w32(e + 0x1c, p);
            if (m.r32(e + 0x18) as i32) > 0 {
                let mut i: u32 = 0;
                loop {
                    let f = m.call(0x0011_7e38, &[s], &[]).1[0];
                    let b = m.r32(e + 0x1c);
                    m.wf((i << 2).wrapping_add(b), f);
                    i = i.wrapping_add(1);
                    if !((i as i32) < (m.r32(e + 0x18) as i32)) {
                        break;
                    }
                }
            }
            m.w32(0x002c_40a4, 0);
        }
        3 => {
            let v = m.call(0x0011_7c80, &[s], &[]).0;
            m.w32(e + 0x10, v);
            let v = m.call(0x0011_7c80, &[s], &[]).0;
            m.w32(e + 0x1c, v);
            let f = m.call(0x0011_7e38, &[s], &[]).1[0];
            m.wf(e + 0x20, f);
            let f = m.call(0x0011_7e38, &[s], &[]).1[0];
            m.wf(e + 0x28, f);
            let f = m.call(0x0011_7e38, &[s], &[]).1[0];
            m.wf(e + 0x24, f);
            let v = m.call(0x0011_7d00, &[s], &[]).0;
            m.w32(e + 0x2c, v);
            let v = m.call(0x0011_7d00, &[s], &[]).0;
            m.w32(e + 0x14, v);
            if v == 1 || v == 0 {
                let w = m.call(0x0011_7db8, &[s], &[]).0;
                m.w32(e + 0x18, w);
            }
            let a = m.r32(e + 0x14);
            if a != 1 || m.r32(e + 0x18) != 0x2f2 {
                m.w32(e + 0x30, 0);
            } else {
                m.w32(e + 0x30, a);
            }
        }
        2 => {
            m.call(0x0011_7ed0, &[s, sp + 0x10], &[]);
            for k in 0..3u32 {
                let q = m.rv(sp + 0x10 + k.wrapping_mul(0x10));
                m.wv(sp + 0x40 + k.wrapping_mul(0x10), q);
            }
            for (src, dst) in [(0x40u32, 0x10u32), (0x44, 0x14), (0x50, 0x18), (0x54, 0x1c), (0x60, 0x20), (0x64, 0x24)] {
                let f = m.rf(sp + src);
                m.wf(e + dst, f);
            }
            let v = m.call(0x0011_7c20, &[s], &[]).0;
            m.w16(e + 0x30, v as u16);
            let h = m.r32(o + 0xc4);
            let p = m.call(0x0011_6918, &[h, (v & 0xffff).wrapping_mul(0x18), 0x14], &[]).0;
            let n = (m.r16(e + 0x30) as u32).wrapping_mul(0x18);
            m.w32(e + 0x34, p);
            m.call(0x0011_7968, &[s, n, p], &[]);
        }
        6 | 7 => {
            for off in [0x40u32, 0x42, 0x50, 0x52] {
                let v = m.call(0x0011_7d50, &[s], &[]).0;
                m.w16(sp + off, v as u16);
            }
            let tex = m.call(0x0011_7db8, &[s], &[]).0;
            let fr = sp + 0x70;
            m.call(0x001a_e478, &[fr], &[]);
            let b = sp + 0x80;
            m.call(0x0016_6920, &[b], &[]);
            m.call(0x0015_72e0, &[b, 0x10], &[]);
            m.call(0x0016_6bd8, &[b, 0, 1, 0xe], &[]);
            m.w32(e + 0x10, tex);
            if m.r32(e + 4) == 6 {
                m.w32(e + 0x14, 0);
                m.call(0x0016_6e90, &[b, 0x47, 0x5000f, 0], &[]);
                m.call(0x0016_6e90, &[b, 0x42, 0xa8, 0x80], &[]);
            } else {
                m.call(0x0016_6e90, &[b, 0x42, 0x44, 0], &[]);
                m.call(0x0016_6e90, &[b, 0x47, 0x50000, 0], &[]);
            }
            // a3 = ((0xa4a4a4a4 << 3) | 1) = 0x5_2525_2521
            m.call(0x0016_6bd8, &[b, 0x54, 9, 0x2525_2521, 5], &[]);
            m.call(0x0016_6c70, &[b, 0x8080_8080], &[]);
            let h = |m: &mut M, off: u32| m.r16(sp + off) as i16 as i32 as u32;
            if m.r32(e + 4) == 6 {
                let pts = [(0.0f32, 0.0f32, 0x40u32, 0x42u32), (1.0, 0.0, 0x50, 0x42), (0.0, 1.0, 0x40, 0x52), (1.0, 1.0, 0x50, 0x52)];
                for (u, v, x, y) in pts {
                    m.call(0x0016_6d38, &[b], &[f4(u), f4(v), f4(0.0)]);
                    let (xx, yy) = (h(m, x), h(m, y));
                    m.call(0x0016_6da0, &[b, xx, yy, 0], &[]);
                }
                m.call(0x0016_6bd8, &[b, 0, 1, 0xe], &[]);
                m.call(0x0016_6e90, &[b, 0x42, 0x44, 0], &[]);
                m.call(0x0016_6e90, &[b, 0x47, 0x50000, 0], &[]);
            } else {
                let pts = [(0.0f32, 1.0f32, 0x40u32, 0x42u32), (1.0, 1.0, 0x50, 0x42), (0.0, 0.0, 0x40, 0x52), (1.0, 0.0, 0x50, 0x52)];
                for (u, v, x, y) in pts {
                    m.call(0x0016_6d38, &[b], &[f4(u), f4(v), f4(0.0)]);
                    let (xx, yy) = (h(m, x), h(m, y));
                    m.call(0x0016_6da0, &[b, xx, yy, 0], &[]);
                }
            }
            m.call(0x0016_69e0, &[b, 1], &[]);
            let d = sp + 0xb0;
            m.call(0x0022_6c10, &[d], &[]);
            m.call(0x0015_72e0, &[d, 0x14], &[]);
            m.call(0x0015_74b8, &[d], &[]);
            let st = m.r32(sp + 0x84);
            let n = m.r32(sp + 0x8c).wrapping_sub(st) >> 4;
            m.call(0x0022_6f28, &[d, 0xc, n, st, 7], &[]);
            m.call(0x0022_6ce0, &[d, 0x650], &[]);
            m.call(0x0015_76f0, &[d], &[]);
            m.call(0x0015_7620, &[d], &[]);
            let hh = m.r32(o + 0xc4);
            if m.r32(e + 4) == 6 {
                m.call(0x0015_73f8, &[d, hh, e + 0x18, 0, e + 0x1c], &[]);
            } else {
                m.call(0x0015_73f8, &[d, hh, e + 0x14, 0, e + 0x18], &[]);
                let (lo, hi) = (m.r32(0x002a_8b08), m.r32(0x002a_8b0c));
                m.call(0x0014_9a50, &[0x003d_d490, o, lo, e, hi], &[]); // WIRE-S2: 64-bit PMF = lo + a[4] hi
            }
            m.call(0x001a_e4a0, &[fr, 2], &[]);
        }
        1 => {
            let fp = sp + 0x40;
            m.w32(sp + 0xf4, e + 0x18);
            m.w32(sp + 0xfc, e + 0x1c);
            let n = m.r16(e + 0x10) as u32;
            m.w32(fp, n.wrapping_mul(0x24));
            m.w32(fp + 4, 0x10);
            let ga = m.call(0x001a_e378, &[fp], &[]).0;
            m.w32(sp + 0xec, ga);
            let n = m.r16(e + 0x10) as u32;
            m.w32(fp, n.wrapping_mul(0x14));
            m.w32(fp + 4, 0x10);
            let gb = m.call(0x001a_e378, &[fp], &[]).0;
            m.w32(sp + 0xf0, gb);
            let mut i: i32 = 0;
            if m.r16(e + 0x10) != 0 {
                loop {
                    let g = ga.wrapping_add((i as u32).wrapping_mul(0x24));
                    let next = i + 1;
                    let v = m.call(0x0011_7bd0, &[s], &[]).0;
                    m.w8(g + 0xc, v as u8);
                    let v = m.call(0x0011_7bd0, &[s], &[]).0;
                    m.w8(g + 0x14, v as u8);
                    let v = m.call(0x0011_7bd0, &[s], &[]).0;
                    m.w32(g, v);
                    let v = m.call(0x0011_7bd0, &[s], &[]).0;
                    m.w32(g + 4, v);
                    let v = m.call(0x0011_7bd0, &[s], &[]).0;
                    let c = m.r8(g + 0xc) as u32;
                    m.w8(g + 0x1c, v as u8);
                    m.w32(g + 0x10, c + 2);
                    m.w32(fp, c << 2);
                    m.w32(fp + 4, 0x10);
                    let p = m.call(0x001a_e378, &[fp], &[]).0;
                    let c = m.r8(g + 0xc) as u32;
                    m.w32(g + 0x18, p);
                    m.w32(fp, c);
                    m.w32(fp + 4, 0x10);
                    let p = m.call(0x001a_e378, &[fp], &[]).0;
                    m.w32(g + 0x20, p);
                    if m.r8(g + 0xc) != 0 {
                        let mut j: i32 = 0;
                        loop {
                            let q = m.r32(g + 0x18).wrapping_add((j as u32) << 2);
                            let v = m.call(0x0011_7c80, &[s], &[]).0;
                            m.w32(g + 8, v);
                            let v = m.call(0x0011_7bd0, &[s], &[]).0;
                            let t = m.r32(g + 0x20).wrapping_add(j as u32);
                            m.w8(t, v as u8);
                            j += 1;
                            let v = m.call(0x0011_7d50, &[s], &[]).0;
                            m.w16(q, v as u16);
                            let v = m.call(0x0011_7d50, &[s], &[]).0;
                            m.w16(q + 2, v as u16);
                            if !(j < m.r8(g + 0xc) as i32) {
                                break;
                            }
                        }
                    }
                    i = next;
                    if !(i < m.r16(e + 0x10) as i32) {
                        break;
                    }
                }
            }
            i = 0;
            m.w16(e + 0x12, 0);
            m.w32(e + 0x18, 0);
            m.w32(e + 0x14, 0);
            if m.r16(e + 0x10) != 0 {
                let mut t4 = ga;
                let mut t5: u32 = 0;
                loop {
                    let t3 = gb.wrapping_add((m.r16(e + 0x12) as u32).wrapping_mul(0x14));
                    m.w32(t3, 0);
                    m.w32(t3 + 8, 0);
                    m.w32(t3 + 4, 0);
                    m.w32(t3 + 0xc, t4);
                    if i < m.r16(e + 0x10) as i32 && (m.r32(t4 + 0x10) as i32) < 0xbd {
                        let mut a6 = t5.wrapping_add(ga);
                        let mut a1 = m.r32(a6 + 0x10);
                        loop {
                            i += 1;
                            let v1 = m.r32(t3 + 4);
                            t4 = t4.wrapping_add(0x24);
                            let a0 = m.r32(t3 + 8);
                            a1 = v1.wrapping_add(a1);
                            let v0 = m.r32(t3);
                            m.w32(t3 + 4, a1);
                            m.w32(t3, v0.wrapping_add(1));
                            let c = m.r8(a6 + 0xc) as u32;
                            a6 = a6.wrapping_add(0x24);
                            m.w32(t3 + 0x10, a6);
                            m.w32(t3 + 8, a0.wrapping_add(c));
                            t5 = t5.wrapping_add(0x24);
                            if !(i < m.r16(e + 0x10) as i32) {
                                break;
                            }
                            let nx = m.r32(a6 + 0x10);
                            if !((a1.wrapping_add(nx) as i32) < 0xbd) {
                                break;
                            }
                            a1 = nx;
                        }
                    }
                    let cnt = m.r32(t3) as i32;
                    let a2 = m.r32(t3 + 8) as i32;
                    let v1 = cnt.wrapping_mul(3);
                    let t1 = v1.wrapping_add(3);
                    let x0 = if a2 >= 0 { a2 } else { a2.wrapping_add(0xf) };
                    let x1 = if a2.wrapping_add(1) >= 0 { a2.wrapping_add(1) } else { a2.wrapping_add(4) };
                    let x2 = if t1 >= 0 { t1 } else { v1.wrapping_add(6) };
                    let tot = (x1 >> 2).wrapping_add(x0 >> 4).wrapping_add(x2 >> 2).wrapping_add(3);
                    let v = m.r32(e + 0x18).wrapping_add(tot as u32);
                    m.w32(e + 0x18, v);
                    let k = m.r16(e + 0x12).wrapping_add(1);
                    let more = i < m.r16(e + 0x10) as i32;
                    m.w16(e + 0x12, k);
                    let v = m.r32(e + 0x14).wrapping_add(m.r32(t3 + 8));
                    m.w32(e + 0x14, v);
                    if !more {
                        break;
                    }
                }
            }
            m.call(0x0022_6c10, &[fp], &[]);
            let n = m.r32(e + 0x18) << 1;
            m.call(0x0015_72e0, &[fp, n], &[]);
            m.call(0x0015_74b8, &[fp], &[]);
            m.call(0x0022_6d68, &[fp, 0x002a_2e10], &[]);
            if m.r16(e + 0x12) != 0 {
                let mut k: i32 = 0;
                loop {
                    let r = gb.wrapping_add((k as u32).wrapping_mul(0x14));
                    k += 1;
                    m.w32(sp + 0xf8, k as u32);
                    let mut s6: u32 = 0;
                    let c = m.r32(r);
                    m.w32(sp + 0x80, c);
                    m.call(0x0022_6f28, &[fp, 4, 1, sp + 0x80, 7], &[]);
                    m.call(0x0022_6d40, &[fp, 1, 2], &[]);
                    let c = m.r32(r);
                    m.call(0x0022_6e88, &[fp, 0xa, c, 8, 0, sp + 0xe0], &[]);
                    let c = m.r32(r);
                    m.call(0x0022_6e88, &[fp, 0xe, c, 9, 0, sp + 0xe4], &[]);
                    let c = m.r32(r);
                    let mut s1 = (c << 1).wrapping_add(8);
                    m.call(0x0022_6d40, &[fp, 1, 1], &[]);
                    let s7 = s1;
                    m.call(0x0022_6dd0, &[fp, 0x5050_5050], &[]);
                    let mut p = m.r32(r + 0xc);
                    if p != m.r32(r + 0x10) {
                        let mut s2: u32 = 0;
                        loop {
                            let d = m.r32(sp + 0xe0).wrapping_add(s2);
                            let b = m.r8(p);
                            m.w8(d, b);
                            let a3 = s6 << 2;
                            s6 += 1;
                            let d = m.r32(sp + 0xe0).wrapping_add(s2);
                            let b = m.r8(p + 0xc);
                            m.w8(d + 1, b);
                            let d = m.r32(sp + 0xe0).wrapping_add(s2);
                            let b = m.r8(p + 0x14);
                            m.w8(d + 2, b);
                            s2 = s2.wrapping_add(3);
                            let d = a3.wrapping_add(m.r32(sp + 0xe4));
                            let w = m.r32(p + 8);
                            m.w32(d, w);
                            let c = m.r8(p + 0xc) as u32;
                            let a = m.r32(p + 0x18);
                            m.call(0x0022_6f28, &[fp, 0x15, c, a, s1], &[]);
                            let c = m.r8(p + 0xc) as u32;
                            p = p.wrapping_add(0x24);
                            s1 = s1.wrapping_add(c);
                            if p == m.r32(r + 0x10) {
                                break;
                            }
                        }
                    }
                    m.call(0x0022_6dd0, &[fp, 0x3f3f_3f3f], &[]);
                    s1 = s7;
                    let mut p = m.r32(r + 0xc);
                    if p != m.r32(r + 0x10) {
                        loop {
                            if m.r8(p + 0x1c) != 0 {
                                let c = m.r8(p + 0xc) as u32;
                                let a = m.r32(p + 0x20);
                                m.call(0x0022_6f28, &[fp, 0x12, c, a, s1], &[]);
                            }
                            let c = m.r8(p + 0xc) as u32;
                            p = p.wrapping_add(0x24);
                            s1 = s1.wrapping_add(c);
                            if p == m.r32(r + 0x10) {
                                break;
                            }
                        }
                    }
                    m.call(0x0022_6dd0, &[fp, 0], &[]);
                    k = m.r32(sp + 0xf8) as i32;
                    m.call(0x0022_6ce0, &[fp, 0x140], &[]);
                    if !(k < m.r16(e + 0x12) as i32) {
                        break;
                    }
                }
            }
            m.call(0x0015_76f0, &[fp], &[]);
            m.call(0x0015_7620, &[fp], &[]);
            let hh = m.r32(o + 0xc4);
            let (x, y) = (m.r32(sp + 0xf4), m.r32(sp + 0xfc));
            m.call(0x0015_73f8, &[fp, hh, x, 0, y], &[]);
        }
        _ => {}
    }
    m.call(0x001a_e4a0, &[sp, 2], &[]);
    stack_pop(m, 0x1b0);
}

/// FUN_0018fc60(o): player follow/aim step. Base pos = FUN_001a62e8(o+0x18c4), dir to the jt (normalized,
/// fallback 0x2a2dc0), FUN_0011e290 / FUN_0011d1c0 solve, poly-clamped gains (0x2bf4f0.. / 0x2bbb60) build a
/// velocity into sp+0xf0, optional target (o+0x1900) correction; result -> FUN_0018f958(o, ..) or o+0x1910.
pub fn ee_sub_18fc60<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let sp = stack_push(m, 0x2a0);
    let t = m.r32(o + 0x18c4);
    let q0 = m.call(0x001a_62e8, &[t], &[]).1;
    m.wv(sp, q0);
    let t = m.r32(o + 0x18c4);
    let v = m.call(0x001a_6318, &[t], &[]).0;
    for k in 0..3u32 {
        let q = m.rv(v + k.wrapping_mul(0x10));
        m.wv(sp + 0x40 + k.wrapping_mul(0x10), q);
    }
    let a6 = m.r32(o + 0x18c4);
    let p = vslot(m, a6, 31, &[0], &[]).0;
    let q = m.rv(sp);
    let q = m.call(0x0022_d348, &[p, 0], &[q]).1;
    m.wv(sp + 0x20, q);
    // dir = (pos(o) - base) normalized
    let mut vf02 = m.call(0x001a_62e8, &[o], &[]).1;
    let mut vf01 = m.rv(sp);
    let mut vf03 = sub(vf02, vf01);
    vf01[0] = dot3(vf03);
    vf01[1] = vf03[1] * vf03[1];
    vf01[2] = vf03[2] * vf03[2];
    m.wv(sp + 0x30, vf03);
    vf02[0] = vf01[0].sqrt();
    m.wv(sp + 0x10, vf03);
    let len = vf02[0];
    if len < eps() {
        let q = m.rv(0x002a_2dc0);
        m.wv(sp + 0x30, q);
    } else {
        vf01 = scl(vf03, 1.0 / vf02[0]);
        m.wv(sp + 0x30, vf01);
    }
    let f22 = len;
    vf03 = vslot(m, o, 32, &[], &[]).1;
    vf01 = m.rv(sp + 0x20);
    vf02 = sub(vf03, vf01);
    m.wv(sp + 0x70, vf02);
    let q30 = m.rv(sp + 0x30);
    let f23 = m.call(0x0011_e290, &[sp + 0x90, sp + 0x80], &[vf02, q30]).1[0];
    vf02 = m.rv(sp + 0x30);
    vf01 = m.rv(sp + 0x80);
    vf03 = cross(vf03, vf02, vf01);
    let l = dot3(vf03).sqrt();
    let mut vf04 = m.rv(0x002a_2da0);
    if !(l < eps()) {
        vf04 = scl(vf03, 1.0 / l);
    }
    vf01 = m.rv(sp + 0x80);
    m.wv(sp + 0xa0, vf04);
    let l80 = dot3(vf01).sqrt();
    vf03 = m.rv(sp + 0x30);
    let v1 = m.r32(o + 0xe8);
    let f4v = l80 / f22;
    vf01 = gqf(f4v);
    vf02 = scl(vf04, vf01[0]);
    m.wv(sp + 0xc0, vf01);
    m.wv(sp + 0xb0, vf02);
    vf02 = m.rv(v1 + 0x80);
    let d = dotp(vf02, vf03);
    vf01 = scl(vf03, d);
    vf02 = sub(vf02, vf01);
    vf04 = cross(vf04, vf03, vf02);
    m.wv(sp + 0xc0, vf02);
    let l = dot3(vf04).sqrt();
    vf03 = m.rv(0x002a_2db0);
    if !(l < eps()) {
        vf03 = scl(vf04, 1.0 / l);
    }
    vf01 = m.rv(sp + 0xc0);
    let lc = dot3(vf01).sqrt();
    let f1 = f4v * m.rf(0x002b_f4f8);
    let f0 = lc / f22;
    let f2 = m.rf(0x002b_f4f4) + f1;
    let f3 = m.rf(0x002b_f4f0);
    let f5 = m.rf(0x002b_f500);
    let f1b = m.rf(0x002b_f504);
    let f2 = f4v * f2;
    m.wv(sp + 0xd0, vf03);
    vf02 = gqf(f0);
    let g = f3 + f2;
    vf01 = scl(vf03, vf02[0]);
    m.wv(sp + 0xe0, vf01);
    m.wv(sp + 0xf0, vf02);
    let f20 = if g < f5 {
        f5
    } else if f1b < g {
        f1b
    } else {
        g
    };
    let st = m.r32(o + 0x18d0);
    let f21 = if st == 0 || st == 2 { m.rf(0x002b_f508) } else { m.rf(0x002b_f50c) };
    vf01 = m.rv(sp + 0xe0);
    vf02 = m.rv(sp + 0xb0);
    let a1q = m.rv(sp + 0x50);
    let dtb = m.r32(0x002d_ded4);
    vf03 = gq(dtb);
    let acc = add(vf02, VF0);
    vf02 = add(acc, scl(vf01, vf03[0]));
    m.wv(sp + 0x120, vf03);
    m.wv(sp + 0xf0, vf02);
    m.call(0x0011_e290, &[sp + 0x100, sp + 0x110], &[vf02, a1q]);
    let f3 = m.rf(0x002d_ded4);
    let f1 = f20 * f3;
    let st = m.r32(o + 0x18d0);
    let f0 = f21 * f3;
    let vf04 = m.rv(sp + 0x100);
    vf03 = m.rv(sp + 0x110);
    let f1 = f1 + 1.0;
    let f0 = f0 + 1.0;
    vf01 = gqf(f1);
    vf02 = gqf(f0);
    m.wv(sp + 0x120, vf01);
    vf01 = add(scl(vf04, vf01[0]), scl(vf03, vf02[0]));
    m.wv(sp + 0x120, vf02);
    m.wv(sp + 0xf0, vf01);
    if st == 0 || st == 2 {
        let f0 = m.rf(0x002d_ded0) - m.rf(o + 0x18d4);
        if f0 < m.rf(0x002b_f510) {
            let mut f6 = f3 * m.rf(0x002b_f514);
            if st == 0 {
                f6 = -f6;
            }
            if m.r32(o + 0x18fc) != 0 {
                f6 = -f6;
            }
            let v0 = m.r32(o + 0xe8);
            let q = m.rv(sp + 0x80);
            let f3 = dot3(q).sqrt();
            let f5 = -m.rf(v0 + 0x88);
            let f1 = f3 * 0.5;
            let f0 = f5 * m.rf(sp + 0x18);
            let f3 = f3 * f1;
            let f7 = m.rf(0x002b_bb60);
            let f0 = (f0 + f3) / f5;
            let f4 = f0 * m.rf(0x002b_f528);
            let f2 = m.rf(0x002b_f524) + f4;
            let f0 = f0 * f2;
            let mut f1 = m.rf(0x002b_f520) + f0;
            let f8 = m.rf(0x002b_bb64);
            if f1 < f7 {
                f1 = f7;
            } else if f8 < f1 {
                f1 = f8;
            }
            f6 = f6 * f1;
            vf01 = m.rv(sp + 0xf0);
            vf02 = m.rv(sp + 0x50);
            vf03 = gqf(f6);
            vf01 = add(add(vf01, VF0), scl(vf02, vf03[0]));
            m.wv(sp + 0x120, vf03);
            m.wv(sp + 0xf0, vf01);
        }
    }
    let v0 = m.r32(o + 0x18c4);
    let f14 = m.rf(0x002b_f544) - m.rf(v0 + 0x134);
    let f15 = m.rf(0x002d_ded4);
    let f20 = m.call(0x0011_d1c0, &[0x002b_f530, sp + 0x1d0], &[f4(f22), f4(f23), f4(f14), f4(f15)]).1[0];
    let f1 = m.rf(0x002b_f544).min(f22);
    let f0 = m.rf(0x002b_f540).max(f20);
    vf02 = m.rv(sp + 0xf0);
    let f1 = f1 / f0;
    vf01 = gqf(f1);
    vf02 = scl(vf02, vf01[0]);
    m.wv(sp + 0x120, vf01);
    m.wv(sp + 0xf0, vf02);
    let tg = m.r32(o + 0x1900);
    if tg != 0 {
        let a0 = m.r32(tg + 4);
        let dtb = m.r32(0x002d_ded4);
        let g1f0 = gq(dtb);
        m.wv(sp + 0x1f0, g1f0);
        vf01 = m.rv(sp);
        vf02 = m.rv(0x002a_2dd0);
        let q20 = m.rv(sp + 0x20);
        m.wv(sp + 0x200, vf01);
        m.wv(sp + 0x130, vf02);
        m.wv(sp + 0x140, vf02);
        m.wv(sp + 0x210, vf02);
        m.wv(sp + 0x1e0, q20);
        m.wv(sp + 0x150, g1f0);
        vf03 = m.call(0x001a_62e8, &[a0], &[]).1;
        vf01 = sub(m.rv(sp + 0x200), vf03);
        vf02 = m.rv(sp + 0x210);
        vf01 = cross(vf01, vf02, vf01);
        let vf04 = m.rv(sp + 0x1e0);
        let vf05 = m.rv(sp + 0x1f0);
        let acc = add(add(vf04, VF0), scl(vf02, vf05[0]));
        vf01 = add(acc, scl(vf01, vf05[0]));
        m.wv(sp + 0x120, vf01);
    } else {
        let q = m.rv(sp + 0x20);
        m.wv(sp + 0x120, q);
    }
    let f0 = m.rf(0x002d_ded4) * 0.5;
    vf01 = m.rv(sp + 0xf0);
    vf02 = add(m.rv(sp + 0xb0), vf01);
    vf03 = gqf(f0);
    vf01 = scl(vf02, vf03[0]);
    m.wv(sp + 0x1c0, vf03);
    m.call(0x0011_b570, &[sp + 0x190], &[vf01]);
    let vf10 = m.rv(sp + 0x190);
    let vf09 = m.rv(sp + 0x1a0);
    let vf08 = m.rv(sp + 0x1b0);
    let mut vf04 = m.rv(sp + 0x30);
    let vf07 = gqf(f20);
    vf04 = mx3(vf04, vf10, vf09, vf08, vf04);
    let v1 = m.r32(sp + 0x1d0);
    let vf06 = scl(vf04, vf07[0]);
    vf02 = m.rv(sp + 0xf0);
    let vf11 = gq(v1);
    vf02 = cross(vf02, vf02, vf06);
    vf03 = m.rv(sp + 0x120);
    let vf05 = scl(vf04, vf11[0]);
    vf03 = add(vf03, vf02);
    vf01 = m.rv(sp);
    vf03 = add(vf03, vf05);
    vf01 = add(vf01, vf06);
    m.wv(sp + 0x140, vf01);
    m.wv(sp + 0x150, vf03);
    m.wv(sp + 0x160, vf10);
    m.wv(sp + 0x170, vf09);
    m.wv(sp + 0x180, vf08);
    m.wv(sp + 0x130, vf04);
    m.wv(sp + 0x190, vf07);
    m.wv(sp + 0x220, vf03);
    m.wv(sp + 0x190, vf11);
    vf01 = vslot(m, o, 32, &[], &[]).1;
    let s0 = m.r32(0x002d_ded4);
    let tg = m.r32(o + 0x1900);
    vf02 = gq(s0);
    m.wv(sp + 0x190, vf02);
    vf03 = sub(m.rv(sp + 0x220), vf01);
    let v0 = m.r32(o + 0xe8);
    vf02 = scl(vf03, 1.0 / vf02[0]);
    vf01 = sub(vf02, m.rv(v0 + 0x80));
    m.wv(sp + 0xc0, vf01);
    if tg != 0 {
        let a1 = m.r32(tg + 4);
        m.call(0x0018_f958, &[o, a1, sp, sp + 0xc0, o + 0x1910], &[]);
    } else {
        m.wv(o + 0x1910, vf01);
    }
    stack_pop(m, 0x2a0);
}

fn clamp(v: f32, lo: f32, hi: f32) -> f32 {
    if v < lo {
        lo
    } else if hi < v {
        hi
    } else {
        v
    }
}

/// ballistic point p + v*t + g*tt as the asm builds it (`vadda ACC, p, vf00` adds vf00.w = 1 to w)
fn arc(p: V4, v: V4, g: V4, t: f32, tt: f32) -> V4 {
    add(add(add(p, VF0), scl(v, t)), scl(g, tt))
}

/// FUN_0021fb70(o, upd): projectile/throw arc aim. Launch P (FUN_001a6f00), gravity G (o+0x10 -> +0xe8 -> +0x80),
/// velocity V (FUN_001d6848); collides the arc against the world (FUN_0020faa0 / FUN_0014cc58 / FUN_0011d508),
/// eases o+0x130, writes the arc samples into the trail (o+0x12c, FUN_0021c5f8) and blends the four aim
/// weights (o+0x68/0x74/0x80/0x8c) from the yaw error and range.
pub fn ee_sub_21fb70<M: QMem + ?Sized>(m: &mut M, o: u32, upd: u32) {
    let sp = stack_push(m, 0x3d0);
    let a0 = m.r32(o);
    let qd0 = m.rv(o + 0xd0);
    let p = m.call(0x001a_6f00, &[a0, 0], &[qd0]).1;
    m.wv(sp + 0x280, p);
    let v0 = m.r32(o + 0x10);
    let mut vf01 = m.rv(o + 0xe0);
    let v1 = m.r32(v0 + 0xe8);
    m.wv(sp + 0x2e0, vf01);
    let g = m.rv(v1 + 0x80);
    m.wv(sp + 0x290, g);
    let a0 = m.r32(o);
    let mut vf02 = m.call(0x001a_62e8, &[a0], &[]).1;
    let f2 = m.rf(o + 0x1c);
    let f1 = m.rf(o + 0x20);
    vf01 = sub(m.rv(sp + 0x2e0), vf02);
    let d = (vf01[0] * vf01[0] + vf01[1] * vf01[1]).sqrt();
    let f0 = (d - f2) / (f1 - f2);
    let mut f25 = if f0 < 0.0 {
        0.0
    } else if 1.0 < f0 {
        1.0
    } else {
        f0
    };
    let f12 = m.rf(0x002c_40f0) + f25 * (m.rf(0x002c_40f4) + f25 * m.rf(0x002c_40f8));
    let (qp, qe0, qg) = (m.rv(sp + 0x280), m.rv(o + 0xe0), m.rv(sp + 0x290));
    let vq = m.call(0x001d_6848, &[], &[qp, qe0, qg, f4(f12)]).1;
    m.wv(sp + 0x2a0, vq);
    let f0 = (-m.rv(sp + 0x2a0)[2] / m.rv(sp + 0x290)[2]).max(0.0);
    let f1 = f0 * 0.5;
    vf02 = gqf(f0);
    m.wv(sp, vf02);
    vf01 = gqf(f0 * f1);
    let (p, v, g) = (m.rv(sp + 0x280), m.rv(sp + 0x2a0), m.rv(sp + 0x290));
    let vf03 = arc(p, v, g, vf02[0], vf01[0]);
    m.wv(sp, vf01);
    m.wv(sp + 0x2b0, vf03);
    let r = m.call(0x001d_aea8, &[0x24], &[]).0;
    let mut f20 = if r != 0 { m.rf(0x002c_40e4) * m.rf(o + 0x24) } else { m.rf(o + 0x24) };
    let mut s0: u32 = 1;
    let f22: f32 = 0.5;
    vf01 = gqf(f20);
    let f0 = f20 * 0.5;
    m.wv(sp, vf01);
    let f0 = f20 * f0;
    let f24 = m.rv(sp + 0x290)[2];
    vf02 = gqf(f0);
    let (p, v, g) = (m.rv(sp + 0x280), m.rv(sp + 0x2a0), m.rv(sp + 0x290));
    vf01 = arc(p, v, g, vf01[0], vf02[0]);
    m.wv(sp, vf02);
    m.wv(sp + 0x2e0, vf01);
    m.call(0x001a_e478, &[sp + 0x10], &[]);
    let f23 = m.rv(sp + 0x280)[2];
    vf01 = m.rv(sp + 0x2e0);
    let vf03 = m.rv(sp + 0x2b0);
    for i in 0..3 {
        vf02[i] = vf03[i].max(vf01[i]);
        vf01[i] = vf03[i].min(vf01[i]);
    }
    let mut vf04 = m.rv(sp + 0x280);
    let mut vf05 = m.rv(sp + 0x280);
    for i in 0..3 {
        vf04[i] = vf04[i].min(vf01[i]);
        vf05[i] = vf05[i].max(vf02[i]);
    }
    let t = m.r32(o + 0x10);
    let t0 = m.r32(t + 0xec);
    m.call(0x0020_faa0, &[0x003d_d490, 0x0021_fb18, t0, sp + 0x270, sp + 0x274], &[vf04, vf05]);
    let q = m.rv(sp + 0x2a0);
    m.wv(o + 0xf0, q);
    let s3 = sp + 0x200;
    let f21 = f20 * fb(0x3d0d_3dcb);
    let mut a1q = m.rv(sp + 0x280);
    loop {
        let (p, v, g) = (m.rv(sp + 0x280), m.rv(sp + 0x2a0), m.rv(sp + 0x290));
        f20 = (s0 as i32 as f32) * f21;
        let a3 = m.r32(sp + 0x270);
        let t0 = m.r32(sp + 0x274);
        let f0 = f20 * (f20 * f22);
        let q2 = gqf(f20);
        m.wv(sp, q2);
        let q1 = gqf(f0);
        let pt = arc(p, v, g, q2[0], q1[0]);
        m.wv(sp, q1);
        m.wv(sp + 0x2f0, pt);
        let hit = m.call(0x0014_cc58, &[1, a3, t0, s3], &[a1q, pt]).0;
        vf02 = m.rv(sp + 0x2f0);
        if hit != 0 {
            let f14 = f23 - m.rf(sp + 0x208);
            let f12 = f24 * f22;
            let f13 = m.rv(sp + 0x2a0)[2];
            let n = m.call(0x0011_d508, &[sp], &[f4(f12), f4(f13), f4(f14)]).0;
            let mut f1 = 0.0f32;
            let mut f3 = m.rf(0x002b_96a0);
            if (n as i32) > 0 {
                let vf07 = m.rv(sp + 0x200);
                let mut vf03 = m.rv(sp + 0x260);
                let (p, v, g) = (m.rv(sp + 0x280), m.rv(sp + 0x2a0), m.rv(sp + 0x290));
                let mut k = n;
                let mut a = sp;
                loop {
                    let f2 = m.rf(a);
                    if !(f2 < 0.0) {
                        let q1 = gqf(f2);
                        vf03 = gqf(f2 * (f2 * 0.5));
                        let d = sub(arc(p, v, g, q1[0], vf03[0]), vf07);
                        let dist = dot3(d).sqrt();
                        if dist < f3 {
                            f3 = dist;
                            f1 = f2;
                        }
                    }
                    k = k.wrapping_sub(1);
                    a += 4;
                    if k == 0 {
                        break;
                    }
                }
                m.wv(sp + 0x260, vf03);
            }
            f20 = f1;
            break;
        }
        s0 += 1;
        if !(s0 < 0x1d) {
            break;
        }
        a1q = vf02;
    }
    let f0 = m.rf(0x002c_4100) * m.rf(0x002d_ded4);
    let f2 = m.rf(o + 0x130);
    let (hi, lo) = (f2 + f0, f2 - f0);
    let f2 = clamp(f20, lo, hi);
    f20 = f2;
    m.wf(o + 0x130, f2);
    let tenth = fb(0x3dcc_cccd);
    s0 = 0;
    if !(f20 < tenth) {
        let (p, v, g) = (m.rv(sp + 0x280), m.rv(sp + 0x2a0), m.rv(sp + 0x290));
        let mut a = sp + 0x20;
        let mut f0 = f20 * 0.5;
        let mut q2: V4 = [0.0; 4];
        loop {
            s0 += 1;
            let q1 = gqf(f20);
            f0 = f20 * f0;
            q2 = gqf(f0);
            let pt = arc(p, v, g, q1[0], q2[0]);
            m.wv(a, pt);
            a += 0x10;
            if !(s0 < 0x1d) {
                break;
            }
            f20 = m.rf(o + 0x130) - (s0 as i32 as f32) * f21;
            if f20 < tenth {
                break;
            }
            f0 = f20 * 0.5;
        }
        m.wv(sp, q2);
    }
    let q = m.rv(sp + 0x280);
    m.wv((s0 << 4).wrapping_add(sp) + 0x20, q);
    let s5 = s0 + 1;
    let tr = m.r32(o + 0x12c);
    if tr != 0 {
        let n = m.r32(tr + 0x158);
        let mut s4 = sp + 0x20;
        let mut s1: u32 = 0;
        let vf04 = gqf(2.0);
        let base = m.r32(tr + 0x15c);
        let vf03 = gqf(-1.0);
        m.wv(sp, vf04);
        let mut s0p = base.wrapping_add(n.wrapping_mul(0x50));
        let mut vf01 = add(scl(m.rv(sp + 0x20), vf04[0]), scl(m.rv(sp + 0x30), vf03[0]));
        m.wv(sp, vf03);
        if (n as i32) > 0 {
            loop {
                s0p = s0p.wrapping_sub(0x50);
                if (s1 as i32) < (s5 as i32) {
                    let q = m.rv(s4);
                    s4 += 0x10;
                    let d = sub(q, vf01);
                    m.wv(sp + 0x2c0, q);
                    let up = m.rv(0x002a_2dc0);
                    m.call(0x0011_b830, &[sp + 0x240], &[d, up]);
                    let q = m.rv(sp + 0x2c0);
                    m.call(0x0025_6d18, &[sp + 0x230], &[q]);
                    for k in 0..3u32 {
                        let q = m.rv(sp + 0x240 + k.wrapping_mul(0x10));
                        m.wv(sp + 0x200 + k.wrapping_mul(0x10), q);
                    }
                    vf01 = m.rv(sp + 0x2c0);
                    m.w32(sp + 0x22c, 0);
                    m.w32(sp + 0x21c, 0);
                    m.w32(sp + 0x20c, 0);
                    m.wf(s3 + 0x3c, 1.0);
                    m.wf(s0p + 4, 1.0);
                } else {
                    m.w32(s0p + 4, 0);
                }
                let f0 = m.rf(0x002d_ded0);
                s1 += 1;
                m.w32(s0p + 8, 0);
                m.wf(s0p, f0);
                for k in 0..4u32 {
                    let q = m.rv(sp + 0x200 + k.wrapping_mul(0x10));
                    m.wv(s0p + 0x10 + k.wrapping_mul(0x10), q);
                }
                let tr = m.r32(o + 0x12c);
                if !((s1 as i32) < (m.r32(tr + 0x158) as i32)) {
                    break;
                }
            }
        }
        let tr = m.r32(o + 0x12c);
        let q = m.rv(sp + 0x280);
        m.call(0x0021_c5f8, &[tr], &[q]);
    }
    let q = m.rv(o + 0xe0);
    m.wv(sp + 0x2d0, q);
    let a0 = m.r32(o);
    let vf03 = m.call(0x001a_62e8, &[a0], &[]).1;
    let d = sub(m.rv(sp + 0x2d0), vf03);
    m.wv(sp, d);
    let (y, x) = (m.rf(sp + 4), m.rf(sp));
    let f20 = m.call(0x0024_58f0, &[], &[f4(y), f4(x)]).1[0];
    let a0 = m.r32(o);
    let f0 = m.call(0x0020_4740, &[a0], &[]).1[0];
    let f0 = m.call(0x0011_cf60, &[], &[f4(f20 - f0)]).1[0];
    let f0 = clamp(f0 / m.rf(o + 0x18), -1.0, 1.0);
    let mut f3 = f0 * 0.5 + 0.5;
    if upd != 0 {
        let f2 = m.rf(o + 0xb0);
        let f5 = if 0.5 < f2 { m.rf(o + 0x8c) / f2 } else { m.rf(o + 0x74) / (1.0 - f2) };
        let f0 = m.rf(0x002c_4104) * m.rf(0x002d_ded4);
        f25 = clamp(f25, f2 - f0, f2 + f0);
        f3 = clamp(f3, f5 - f0, f5 + f0);
    }
    let a0 = m.r32(o + 0x98);
    let f20 = 1.0 - f25;
    let f0 = 1.0 - f3;
    m.wf(o + 0x8c, f3 * f25);
    m.wf(o + 0x74, f3 * f20);
    m.wf(o + 0x80, f0 * f25);
    m.wf(o + 0x68, f0 * f20);
    m.call(0x0013_84c0, &[a0], &[]);
    let a0 = m.r32(o + 0xbc);
    m.wf(o + 0xa4, f20);
    m.wf(o + 0xb0, f25);
    m.call(0x0013_84c0, &[a0], &[]);
    m.call(0x001a_e4a0, &[sp + 0x10, 2], &[]);
    stack_pop(m, 0x3d0);
}

/// unaligned word copy (`lwl`/`lwr` + `swl`/`swr`)
fn cpw<M: QMem + ?Sized>(m: &mut M, dst: u32, src: u32) {
    for k in 0..4 {
        let b = m.r8(src + k);
        m.w8(dst + k, b);
    }
}

fn cpq<M: QMem + ?Sized>(m: &mut M, dst: u32, src: u32) {
    let q = m.rv(src);
    m.wv(dst, q);
}

/// per-channel byte lerp of FUN_00219fe0 (bias 0x3f7fbe77 = 0.999, `cvt.w.s` truncation)
fn lerp_b(a: u8, b: u8, t: f32, bias: f32) -> u8 {
    let (a, b) = (a as i32, b as i32);
    let v = if a == b {
        b
    } else if a < b {
        a.wrapping_add((t * ((b as f32 + bias) - a as f32)) as i32)
    } else {
        b.wrapping_add(((1.0 - t) * ((a as f32 + bias) - b as f32)) as i32)
    };
    v as u8
}

fn lerp_w<M: QMem + ?Sized>(m: &mut M, sp: u32, t: f32, bias: f32) {
    for c in 0..4 {
        let (a, b) = (m.r8(sp + 0x80 + c), m.r8(sp + 0x90 + c));
        m.w8(sp + 0xa0 + c, lerp_b(a, b, t, bias));
    }
}

/// FUN_00219fe0(o, idx, xf, col0, col1, f12): advance animation track `idx` (o+0x10 -> +8, 0x18 stride) to
/// time key f22 (keys at o+4, 0x20 stride, count *(u16 *)o): per passed key FUN_00219190 / FUN_002190f8 /
/// FUN_00219348 / FUN_00219400; then for each of its 0x70-byte elements (+0x30 live) interpolate the 6-float
/// rect and two colours between keyframes (+0x44 / +0x54, +0x3c..+0x50), transform by the 2x3 `xf`,
/// FUN_00218790 / FUN_00215850 and store back (+0x5c = time), FUN_0011a058(e+0x68). First pass with
/// key index 0 takes FUN_002158c0 instead.
pub fn ee_sub_219fe0<M: QMem + ?Sized>(m: &mut M, o: u32, idx: u32, xf: u32, a3: u32, t0: u32, f12: f32) {
    let sp = stack_push(m, 0x1a0);
    let f24 = f12;
    let k = idx.wrapping_mul(0x18);
    m.w32(sp + 0xc4, k);
    m.w32(sp + 0xb0, a3);
    m.w32(sp + 0xb4, t0);
    let v1 = m.r32(o.wrapping_add(0x10));
    let e = k.wrapping_add(m.r32(v1.wrapping_add(8)));
    let x = m.r32(e.wrapping_add(4));
    m.w32(sp + 0xbc, x);
    let x = m.r32(e);
    m.w32(sp + 0xb8, x);
    let f23 = m.rf(e.wrapping_add(0xc));
    let mut f22 = f23;
    let mut s2 = m.r16(e.wrapping_add(0xa)) as u32;
    m.w32(sp + 0xc0, (s2 < 1) as u32);
    let bias = fb(0x3f7f_be77);
    loop {
        // 0x21a078
        let mut keys = (s2 as i32) < (m.r16(o) as i32);
        if keys {
            let kp = (s2 << 5).wrapping_add(m.r32(o.wrapping_add(4)));
            let f0 = m.rf(kp);
            keys = f0 <= f23;
            if keys {
                f22 = f0;
                m.call(0x0021_9190, &[o, s2, idx], &[]);
                m.call(0x0021_90f8, &[o, s2, idx], &[]);
                m.call(0x0021_9348, &[o, s2, idx], &[]);
                let p = m.r32(sp + 0xb0);
                cpw(m, sp + 0x40, p);
                let p = m.r32(sp + 0xb4);
                cpw(m, sp + 0x50, p);
                m.call(0x0021_9400, &[o, s2, idx, xf, sp + 0x40, sp + 0x50], &[f4(f22), f4(f24)]);
                s2 = s2.wrapping_add(1);
                let v = m.r32(o.wrapping_add(0x10));
                let e = m.r32(sp + 0xc4).wrapping_add(m.r32(v + 8));
                let c = m.r16(e.wrapping_add(0xa)).wrapping_add(1);
                m.w16(e.wrapping_add(0xa), c);
                let v = m.r32(o.wrapping_add(0x10));
                let e = m.r32(sp + 0xc4).wrapping_add(m.r32(v + 8));
                let n = m.r16(o);
                if (n as u32) < (m.r16(e.wrapping_add(0xa)) as u32) {
                    m.w16(e.wrapping_add(0xa), n);
                }
            }
        }
        // 0x21a160
        let cnt = m.r32(sp + 0xb8);
        if (cnt as i32) > 0 {
            let mut i: u32 = 0;
            loop {
                let s0 = m.r32(sp + 0xbc).wrapping_add(i.wrapping_mul(0x70));
                let s7 = i.wrapping_add(1);
                let a0 = m.r32(s0.wrapping_add(0x30));
                if a0 != 0 && m.rf(s0.wrapping_add(0x5c)) != f22 {
                    if m.r32(sp + 0xc0) != 0 {
                        m.call(0x0021_58c0, &[a0, s0], &[]);
                    } else {
                        let s5 = sp + 0x70;
                        let s4 = sp + 0x80;
                        let s6 = s0.wrapping_add(0x68);
                        let nx = m.r32(s0.wrapping_add(0x54));
                        if m.r8(s0.wrapping_add(0x60)) != 0 && nx != 0 {
                            let f0 = m.rf(s0.wrapping_add(0x38));
                            let f1 = m.rf(s0.wrapping_add(0x48));
                            let mut pa = nx;
                            let mut pb = m.r32(s0.wrapping_add(0x44));
                            let f4v = (f22 - f0) / (f1 - f0);
                            let f3 = 1.0 - f4v;
                            let mut a1 = sp + 0x90;
                            let mut c: i32 = 5;
                            loop {
                                let x = f3 * m.rf(pb);
                                let y = f4v * m.rf(pa);
                                c -= 1;
                                pb += 4;
                                pa += 4;
                                m.wf(a1, x + y);
                                a1 += 4;
                                if c < 0 {
                                    break;
                                }
                            }
                            let (q1, q2) = (m.rv(sp + 0x90), m.rv(sp + 0xa0));
                            m.wv(sp, q1);
                            m.wv(sp + 0x80, q2);
                            m.wv(sp + 0x70, q1);
                            m.wv(sp + 0x10, q2);
                            cpw(m, sp + 0x80, s0.wrapping_add(0x3c));
                            cpw(m, sp + 0x90, s0.wrapping_add(0x4c));
                            lerp_w(m, sp, f4v, bias);
                            cpw(m, sp + 0x70, sp + 0xa0);
                            cpw(m, sp + 0x40, sp + 0x70);
                            cpw(m, sp + 0x80, s0.wrapping_add(0x40));
                            cpw(m, sp + 0x90, s0.wrapping_add(0x50));
                            lerp_w(m, sp, f4v, bias);
                            cpw(m, sp + 0x70, sp + 0xa0);
                            cpw(m, sp + 0x60, sp + 0x70);
                        } else {
                            let v1 = m.r32(s0.wrapping_add(0x44));
                            cpq(m, sp, v1);
                            cpq(m, sp + 0x10, v1.wrapping_add(0x10));
                            cpw(m, sp + 0x40, s0.wrapping_add(0x3c));
                            cpw(m, sp + 0x60, s0.wrapping_add(0x40));
                        }
                        // 0x21a794: 2x3 affine transform of the rect
                        let (m0, m1, m2, m3, m4, m5) =
                            (m.rf(xf), m.rf(xf + 4), m.rf(xf + 8), m.rf(xf + 0xc), m.rf(xf + 0x10), m.rf(xf + 0x14));
                        let (x0, x1, x2, x3, x4, x5) =
                            (m.rf(sp), m.rf(sp + 4), m.rf(sp + 8), m.rf(sp + 0xc), m.rf(sp + 0x10), m.rf(sp + 0x14));
                        m.wf(sp + 0xa0, x4 * m0 + x5 * m2 + m4);
                        m.wf(sp + 0x90, x0 * m0 + x1 * m2);
                        m.wf(sp + 0x94, x0 * m1 + x1 * m3);
                        m.wf(sp + 0x98, x2 * m0 + x3 * m2);
                        m.wf(sp + 0x9c, x2 * m1 + x3 * m3);
                        m.wf(sp + 0xa4, x4 * m1 + x5 * m3 + m5);
                        let (q1, q2) = (m.rv(sp + 0x90), m.rv(sp + 0xa0));
                        m.wv(sp, q1);
                        m.wv(sp + 0x10, q2);
                        m.wv(sp + 0x70, q1);
                        m.wv(sp + 0x80, q2);
                        let p = m.r32(sp + 0xb4);
                        cpw(m, sp + 0x70, p);
                        let p = m.r32(sp + 0xb0);
                        cpw(m, sp + 0x80, p);
                        cpw(m, sp + 0x90, sp + 0x60);
                        cpw(m, sp + 0xa0, sp + 0x40);
                        m.call(0x0021_8790, &[s5, s4, sp + 0x90, sp + 0xa0, sp + 0x30, sp + 0x20], &[]);
                        cpw(m, sp + 0x70, sp + 0x20);
                        cpw(m, sp + 0x80, sp + 0x30);
                        let a0 = m.r32(s0.wrapping_add(0x30));
                        m.call(0x0021_5850, &[a0, s0, sp, s5, s4], &[f4(f24)]);
                        m.wf(s0.wrapping_add(0x5c), f22);
                        cpq(m, s0.wrapping_add(0x10), sp);
                        cpq(m, s0.wrapping_add(0x20), sp + 0x10);
                        cpw(m, s0, sp + 0x20);
                        cpw(m, s0.wrapping_add(4), sp + 0x30);
                        m.call(0x0011_a058, &[s6], &[]);
                    }
                }
                i = s7;
                if !((i as i32) < (m.r32(sp + 0xb8) as i32)) {
                    break;
                }
            }
        }
        // 0x21a920
        m.w32(sp + 0xc0, 0);
        if !((s2 as i32) < (m.r16(o) as i32)) {
            break;
        }
        let kp = (s2 << 5).wrapping_add(m.r32(o.wrapping_add(4)));
        if !(m.rf(kp) <= f23) {
            break;
        }
    }
    stack_pop(m, 0x1a0);
}

/// `c0 + x * (c1 + x * c2)` from the 3-float table at `t`
fn poly<M: QMem + ?Sized>(m: &mut M, t: u32, x: f32) -> f32 {
    let (c0, c1, c2) = (m.rf(t), m.rf(t.wrapping_add(4)), m.rf(t.wrapping_add(8)));
    c0 + x * (c1 + x * c2)
}

/// bit 60 of the u64 at p+0x28 (`ld` ; `dsrl32 0x1c` ; `andi 1`)
fn bit60<M: QMem + ?Sized>(m: &mut M, p: u32) -> bool {
    (m.r64(p.wrapping_add(0x28)) >> 60) & 1 != 0
}

/// FUN_00180668(o): guard/turret look-at update. FUN_001f36e0(o); state o+0x1c0 (2/4 -> FUN_0017f550 check;
/// target o+0x1d4 lost -> FUN_00181268(o, 0)); with a live target re-pick the zone (0x28-byte bands from o+0x270 /
/// o+0x310, FUN_0011d910 on range / yaw) -> FUN_00181268(o, n). Then by state (jump table 0x2a5660): 1/2 aim yaw
/// poly (o+0x360) clamped to o+0x370/0x374 -> o+0x204; 3 track the target path (FUN_001c02d0 / FUN_001e7380),
/// weight it, push away from nearby actors (FUN_0015e9c8), facing -> o+0x210 (FUN_0011e9f8) and o+0x204.
pub fn ee_sub_180668<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let sp = stack_push(m, 0x250);
    m.call(0x001f_36e0, &[o], &[]);
    let mut st = m.r32(o + 0x1c0);
    if st == 2 || st == 4 {
        if m.call(0x0017_f550, &[o], &[]).0 != 0 {
            m.call(0x0018_1268, &[o, 0], &[]);
        }
        st = m.r32(o + 0x1c0);
    }
    if st != 0 && st != 4 {
        let t = m.r32(o + 0x1d4);
        if t == 0 || !bit60(m, t) {
            m.call(0x0018_1268, &[o, 0], &[]);
        }
    }
    'pick: {
        if m.r32(o + 0x1cc) != 0 {
            break 'pick;
        }
        let a0 = m.r32(o + 0x1d4);
        if a0 == 0 || !bit60(m, a0) {
            break 'pick;
        }
        let s0 = o.wrapping_add(m.r32(o + 0x1c0).wrapping_mul(0x28)).wrapping_add(0x270);
        let q1 = m.call(0x001a_62e8, &[a0], &[]).1;
        m.wv(sp + 0x190, q1);
        let q2 = m.call(0x001a_62e8, &[o], &[]).1;
        let d = sub(m.rv(sp + 0x190), q2);
        let f21 = dot3(d).sqrt();
        let r = m.call(0x001a_6fc0, &[0, o], &[d]).1;
        let f0 = m.call(0x0024_58f0, &[], &[f4(r[1]), f4(r[0])]).1[0];
        let f20 = m.call(0x0011_cf60, &[], &[f4(f0)]).1[0];
        let v0 = if m.r32(s0) != 0 { m.call(0x0011_d910, &[s0 + 0x10], &[f4(f21)]).0 } else { m.r32(s0 + 4) };
        if v0 != 0 {
            if m.r32(s0 + 4) == 0 {
                break 'pick;
            }
            if m.call(0x0011_d910, &[s0 + 0x20], &[f4(f20)]).0 != 0 {
                break 'pick;
            }
        }
        let mut s3: i32 = 4;
        let mut s2 = o + 0x310;
        while s3 > 0 {
            let s0 = s2;
            let v0 = if m.r32(s0) != 0 { m.call(0x0011_d910, &[s0 + 8], &[f4(f21)]).0 } else { m.r32(s0 + 4) };
            if v0 != 0 {
                if m.r32(s0 + 4) == 0 {
                    break;
                }
                if m.call(0x0011_d910, &[s0 + 0x18], &[f4(f20)]).0 != 0 {
                    break;
                }
            }
            s2 = s2.wrapping_sub(0x28);
            s3 -= 1;
        }
        m.call(0x0018_1268, &[o, s3 as u32], &[]);
    }
    match m.r32(o + 0x1c0) {
        1 | 2 => {
            if m.r32(o + 0x1c8) == 0 {
                let t = m.r32(o + 0x1d4);
                let s0 = m.r32(o + 0x5c);
                let q = m.call(0x001a_62e8, &[t], &[]).1;
                m.wv(sp + 0x150, q);
                let q2 = m.call(0x001a_62e8, &[o], &[]).1;
                let d = sub(m.rv(sp + 0x150), q2);
                let a1 = m.r32(s0 + 0xa0);
                let r = m.call(0x0022_fcc8, &[0, a1], &[d]).1;
                let f0 = m.call(0x0024_58f0, &[], &[f4(r[1]), f4(r[0])]).1[0];
                let f0 = m.call(0x0011_cf60, &[], &[f4(f0)]).1[0];
                let f3 = poly(m, o + 0x360, f0);
                let (lo, hi) = (m.rf(o + 0x370), m.rf(o + 0x374));
                m.wf(o + 0x204, clamp(f3, lo, hi));
            }
        }
        3 => {
            let t = m.r32(o + 0x1d4);
            let qt = m.call(0x001a_62e8, &[t], &[]).1;
            m.wv(sp + 0x160, qt);
            let qo = m.call(0x001a_62e8, &[o], &[]).1;
            m.wv(sp + 0x170, qo);
            vslot(m, o, 32, &[], &[]);
            let f1 = m.rf(0x002b_e820);
            let mut redo = f1 < dot3(sub(m.rv(sp + 0x160), m.rv(o + 0x1f0))).sqrt();
            if !redo {
                redo = f1 < dot3(sub(m.rv(sp + 0x170), m.rv(o + 0x1e0))).sqrt();
            }
            if !redo {
                redo = m.rf(0x002b_e824) < m.rf(0x002d_ded0) - m.rf(o + 0x200);
            }
            if redo {
                let (a0, a3) = (m.r32(o + 0x1d0), m.r32(o + 0x1d8));
                let (qo, qt) = (m.rv(sp + 0x170), m.rv(sp + 0x160));
                m.call(0x001c_02d0, &[a0, a3], &[qo, qt]);
                m.w32(o + 0x1dc, 0);
                let q = m.rv(sp + 0x160);
                m.wv(o + 0x1f0, q);
                let q = m.rv(sp + 0x170);
                m.wv(o + 0x1e0, q);
                let f0 = m.rf(0x002d_ded0);
                m.wf(o + 0x200, f0);
            }
            let a0 = m.r32(o + 0x1d8);
            let qo = m.rv(sp + 0x170);
            let f12 = m.rf(o + 0x1dc);
            m.call(0x001e_7380, &[a0, 0, sp, sp + 0x10, 0, o + 0x1dc], &[qo, f4(f12)]);
            let vf01 = m.rv(sp + 0x10);
            let vf03 = gq(m.r32(0x002b_e828));
            let vf02 = add(add(m.rv(sp), VF0), scl(vf01, vf03[0]));
            let vf04 = sub(vf02, m.rv(sp + 0x170));
            let l = dot3(vf04).sqrt();
            m.wv(sp + 0x30, vf03);
            let dir = if l < eps() { m.rv(0x002a_2dd0) } else { scl(vf04, 1.0 / l) };
            m.wv(sp + 0x20, dir);
            let a0 = m.r32(o + 0x1d8);
            let f0 = m.call(0x001e_6648, &[a0], &[]).1[0];
            let f20 = f0 - m.rf(o + 0x1dc);
            let q = m.rv(sp + 0x20);
            let r = m.call(0x001a_6fc0, &[0, o], &[q]).1;
            let f21 = m.call(0x0024_58f0, &[], &[f4(r[1]), f4(r[0])]).1[0];
            let f7 = m.call(0x0015_4c90, &[0x002d_e578], &[]).1[0];
            let (lo, hi) = (m.rf(0x002b_bb60), m.rf(0x002b_bb64));
            let f6 = clamp(poly(m, o + 0x340, f20), lo, hi);
            let f3 = clamp(poly(m, o + 0x350, f21), lo, hi);
            let f2 = clamp(poly(m, 0x002b_e830, f7), lo, hi);
            let f20 = f6 * f3 * f2 * m.rf(o + 0x338);
            let vf01 = gqf(f20);
            let vf03 = scl(m.rv(sp + 0x20), vf01[0]);
            m.wv(sp + 0x30, vf01);
            m.wv(sp + 0x180, vf03);
            let n = m.call(0x0015_e9c8, &[0x003d_d490, 0x105, 0x49, 0, 0x40, sp + 0x40], &[]).0;
            if (n as i32) > 0 {
                let mut s2 = sp + 0x40;
                let mut s0 = n;
                loop {
                    let a0 = m.r32(s2);
                    if a0 != o && m.r32(a0 + 0xec) != m.r32(o + 0xec) {
                        let vf03 = m.call(0x001a_62e8, &[a0], &[]).1;
                        let d = sub(m.rv(sp + 0x170), vf03);
                        m.wv(sp + 0x30, d);
                        m.w32(sp + 0x38, 0);
                        let vf03 = m.rv(sp + 0x30);
                        let len = dot3(vf03).sqrt();
                        if len < eps() {
                            let q = m.rv(0x002a_2da0);
                            m.wv(sp + 0x30, q);
                        } else {
                            m.wv(sp + 0x30, scl(vf03, 1.0 / len));
                        }
                        let (lo, hi) = (m.rf(0x002b_e850), m.rf(0x002b_e854));
                        let f6 = clamp(poly(m, 0x002b_e840, len), lo, hi);
                        let dd = dotp(m.rv(sp + 0x30), m.rv(sp + 0x20));
                        let (lo, hi) = (m.rf(0x002b_bb60), m.rf(0x002b_bb64));
                        let f0 = f6 * clamp(poly(m, 0x002b_e860, dd), lo, hi);
                        let vf01 = gqf(f0);
                        let vf03 = add(add(m.rv(sp + 0x180), VF0), scl(m.rv(sp + 0x30), vf01[0]));
                        m.wv(sp + 0x140, vf01);
                        m.wv(sp + 0x180, vf03);
                    }
                    s0 = s0.wrapping_sub(1);
                    s2 += 4;
                    if s0 == 0 {
                        break;
                    }
                }
            }
            let q = m.rv(sp + 0x180);
            let r = m.call(0x0011_e9f8, &[], &[q, f4(f20)]).1;
            m.wv(o + 0x210, r);
            let (y, x) = (m.rf(sp + 0x24), m.rf(sp + 0x20));
            let f0 = m.call(0x0024_58f0, &[], &[f4(y), f4(x)]).1[0];
            m.wf(o + 0x204, f0);
        }
        _ => {}
    }
    stack_pop(m, 0x250);
}
