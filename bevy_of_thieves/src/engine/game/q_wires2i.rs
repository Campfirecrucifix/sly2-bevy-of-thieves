//! WIRE-S2: 1:1 QMem ports of live-frontier natives (shard 3/4).
#![allow(unused)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

const T: u32 = 0x002d_ded0; // g_clock t
const DT: u32 = 0x002d_ded4; // g_clock dt
const CM: u32 = 0x002d_df40; // camera manager
const GS: u32 = 0x003d_d490; // game state / world

/// vmul.xyz + vadday.x + vmaddz.x (with 1.0) dot product
fn dot3(a: V4, b: V4) -> f32 {
    (a[0] * b[0] + a[1] * b[1]) + a[2] * b[2]
}

/// vopmula/vopmsub: cross(a, b).xyz, w from `w`
fn cross(a: V4, b: V4, w: f32) -> V4 {
    [a[1] * b[2] - b[1] * a[2], a[2] * b[0] - b[2] * a[0], a[0] * b[1] - b[0] * a[1], w]
}

/// FUN_001f9410: build a 0x10-frame solver (FUN_001585d8) over t0 bodies at t1 and a2 constraints at a3, run it by a1 flags, then notify.
pub fn ee_sub_1f9410<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, t0: u32, t1: u32) {
    let sp = stack_push(m, 0x160);
    m.w32(sp + 0x80, a0);
    m.w32(sp + 0x88, a3);
    m.w32(sp + 0x84, a1);
    m.call(0x001a_e478, &[sp], &[]);
    m.call(0x0015_85d8, &[sp + 0x10, t0, t1, a2, a3], &[]);
    let n = t0 as i32;
    let na = a2 as i32;
    if n > 0 {
        let f22 = m.rf(0x002a_8280);
        let f20 = m.rf(0x002a_8284);
        let flag1 = a1 & 1;
        m.w32(sp + 0x8c, flag1);
        for i in 0..t0 {
            let base = m.r32(sp + 0x70);
            let s0 = m.r32(t1 + 4 * i);
            let s1 = m.r32(sp + 0x18).wrapping_add(0x28 * i);
            let r = base.wrapping_add(0x60 * i);
            m.w32(r + 0x50, s0);
            let q = m.rv(s0 + 0x80);
            m.wv(r + 0x40, q);
            let k = m.r32(s0 + 0x9c);
            if k == 0xb {
                m.wf(s1 + 4, f22);
                m.w32(s1, 1);
                m.wf(s1 + 8, f20);
            } else if k == 0xe {
                if flag1 == 0 {
                    m.w32(s1 + 4, 0);
                    m.wf(s1 + 8, f20);
                    m.w32(s1, 0);
                } else {
                    let d = m.rf(DT);
                    let x = m.rf(s0 + 0xa0) * d;
                    m.wf(s1 + 4, x);
                    let d = m.rf(DT);
                    let y = m.rf(s0 + 0xa4) * d;
                    m.wf(s1 + 8, y);
                    let neg = (m.rf(s0 + 0xa0) < 0.0) as u32;
                    m.w32(s1, neg);
                }
            } else {
                m.w32(s1, 0);
                m.w32(s1 + 4, 0);
                m.wf(s1 + 8, f20);
            }
            m.call(0x0015_8668, &[sp + 0x10, s1], &[]);
        }
    }
    if a1 & 4 != 0 {
        let mut f20 = 0.0f32;
        if n > 0 {
            for i in 0..t0 {
                let s1 = m.r32(sp + 0x18).wrapping_add(0x28 * i);
                let s0 = m.r32(t1 + 4 * i);
                let f0 = if m.r32(s1) == 1 { m.rf(s0 + 0x90) } else { m.rf(s0 + 0x90) + 1.0 };
                m.wf(s1 + 0x10, f0);
                let a = m.rf(s0 + 0x90).abs();
                if f20 < a {
                    f20 = a;
                }
            }
        }
        m.call(0x0015_8988, &[sp + 0x10, 4], &[]);
        m.call(0x0015_8e20, &[sp + 0x10], &[f4(f20 + f20), f4(f32::from_bits(0x40c9_0fdb))]);
        if na > 0 {
            for i in 0..a2 {
                let o = m.r32(a3 + 4 * i);
                vslot(m, o, 66, &[], &[]);
            }
        }
        if a1 == 4 {
            m.call(0x001a_e4a0, &[sp, 2], &[]);
            stack_pop(m, 0x160);
            return;
        }
        m.call(0x0011_a6b8, &[sp + 0x10], &[]);
    }
    let mut f20 = 0.0f32;
    let s7 = a1 & 1;
    if s7 != 0 {
        m.call(0x001a_7e60, &[a0, t0, t1], &[]);
        m.call(0x001a_7fc0, &[a0, t0, t1], &[]);
        if n > 0 {
            for i in 0..t0 {
                let s0 = m.r32(t1 + 4 * i);
                let s1 = m.r32(sp + 0x18).wrapping_add(0x28 * i);
                let v = m.rv(s0 + 0xb0);
                let p = m.rv(s0 + 0x80);
                m.wf(s1 + 0x10, dot3(v, p));
                let v = m.rv(s0 + 0xb0);
                let f0 = dot3(v, v);
                if f20 < f0 {
                    f20 = f0;
                }
            }
        }
    } else if n > 0 {
        for i in 0..t0 {
            let s1 = m.r32(sp + 0x18).wrapping_add(0x28 * i);
            let r = m
                .call(0x0011_d448, &[], &[f4(f32::from_bits(0x3f66_6666)), f4(f32::from_bits(0x3f8c_cccd))])
                .1[0];
            let s0 = m.r32(t1 + 4 * i);
            let f1 = m.rf(s0 + 0x98) * r + 1.0;
            let v = m.rv(s0 + 0xb0);
            let p = m.rv(s0 + 0x80);
            m.wf(s1 + 0x10, dot3(v, p) * f1);
            let v = m.rv(s0 + 0xb0);
            let f0 = dot3(v, v);
            if f20 < f0 {
                f20 = f0;
            }
        }
    }
    m.call(0x0015_8b20, &[sp + 0x10, a1], &[]);
    let f12 = f20.sqrt();
    m.call(0x0015_8e20, &[sp + 0x10], &[f4(f12 + f12), f4(50.0)]);
    if s7 == 0 {
        if na > 0 {
            for i in 0..a2 {
                let o = m.r32(a3 + 4 * i);
                vslot(m, o, 65, &[], &[]);
            }
        }
    } else {
        if na > 0 {
            for i in 0..a2 {
                let o = m.r32(a3 + 4 * i);
                vslot(m, o, 64, &[], &[]);
            }
        }
        let mut found = false;
        let lim = f32::from_bits(0x41c8_0000);
        let eps = f32::from_bits(0x3b23_d70b);
        for i in 0..(if na > 0 { a2 } else { 0 }) {
            let c = m.r32(a3 + 4 * i);
            let b0 = m.r32(c + 0x58);
            if m.r16(b0 + 0x98) == 2 {
                found = true;
                break;
            }
            let b1 = m.r32(c + 0x5c);
            if m.r16(b1 + 0x98) == 2 {
                found = true;
                break;
            }
            let e = m.r32(c + 0xe8);
            let v = m.rv(b0 + 0xb0);
            let mut hit = lim < dot3(v, v);
            if !hit {
                let b = m.rv(b1 + 0xb0);
                hit = eps < dot3(b, b);
            }
            if !hit {
                let v = m.rv(e + 0x60);
                hit = lim < dot3(v, v);
            }
            if !hit {
                let v = m.rv(e + 0x70);
                hit = eps < dot3(v, v);
            }
            if hit {
                found = true;
                break;
            }
        }
        if !found {
            if na > 0 {
                for i in 0..a2 {
                    let o = m.r32(a3 + 4 * i);
                    m.call(0x001f_9c48, &[o], &[]);
                }
            }
            m.call(0x001a_7e60, &[GS, t0, t1], &[]);
            m.call(0x001a_7fc0, &[GS, t0, t1], &[]);
        }
    }
    m.call(0x001a_e4a0, &[sp, 2], &[]);
    stack_pop(m, 0x160);
}

/// FUN_002236e8: periodic camera visibility probe of the target (a0->+0): box over 5 offset points, GS query, ray tests; hit -> +0x80 = t.
pub fn ee_sub_2236e8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let s4 = m.r32(a0);
    if m.r32(0x002d_fbc4) & 2 != 0 {
        let t = m.rf(T);
        m.wf(a0 + 0x80, t);
        return;
    }
    if m.r32(a0 + 0x10) != 0 && m.r32(0x002c_018c) != 0 {
        return;
    }
    if m.r32(0x002c_4118) != 0 {
        return;
    }
    if m.rf(T) - m.rf(a0 + 0x80) < m.rf(0x002c_4110) {
        return;
    }
    if m.r32(CM + 0x300) != 0 {
        return;
    }
    let sp = stack_push(m, 0x180);
    let c = m.rv(CM + 0x90);
    let up = m.rv(0x002a_2dc0);
    m.wv(sp, c);
    m.wv(sp + 0x10, c);
    m.wv(sp + 0xc0, up);
    m.wv(sp + 0xa0, up);
    let p = m.call(0x001a_62e8, &[s4], &[]).1;
    let c = m.rv(CM + 0x90);
    let d = [p[0] - c[0], p[1] - c[1], p[2] - c[2], p[3] - c[3]];
    let a = m.rv(sp + 0xc0);
    let v3 = cross(a, d, p[3]);
    let len = dot3(v3, v3).sqrt();
    let dflt = m.rv(0x002a_2da0);
    let side = if len < f32::from_bits(0x38d1_b717) {
        dflt
    } else {
        let q = 1.0 / len;
        [v3[0] * q, v3[1] * q, v3[2] * q, v3[3] * q]
    };
    let u = m.rv(sp + 0xa0);
    let fwd = cross(u, side, u[3]);
    m.wv(sp + 0x90, side);
    m.wv(sp + 0x80, fwd);
    for i in 0..5u32 {
        let p = m.call(0x001a_62e8, &[s4], &[]).1;
        let o = m.rv(0x002c_4290 + 0x10 * i);
        let v3 = m.rv(sp + 0x90);
        let v1 = m.rv(sp + 0xa0);
        let v5 = m.rv(sp + 0x80);
        let mut v4 = o;
        for k in 0..3 {
            v4[k] = (v5[k] * o[0] + v3[k] * o[1]) + v1[k] * o[2];
        }
        let q = [p[0] + v4[0], p[1] + v4[1], p[2] + v4[2], p[3] + v4[3]];
        let mut lo = m.rv(sp);
        for k in 0..3 {
            lo[k] = lo[k].min(q[k]);
        }
        let mut hi = p;
        for k in 0..3 {
            hi[k] = lo[k].max(q[k]);
        }
        m.wv(sp, lo);
        m.wv(sp + 0x10, hi);
        m.wv(sp + 0x30 + 0x10 * i, q);
    }
    m.call(0x001a_e478, &[sp + 0xb0], &[]);
    let lo = m.rv(sp);
    let hi = m.rv(sp + 0x10);
    m.call(0x0020_faa0, &[GS, 0x0014_f030, 0, sp + 0xb4, sp + 0xb8], &[lo, hi]);
    for i in 0..5u32 {
        let s0 = sp + 0x30 + 0x10 * i;
        let q = m.rv(s0);
        let v = m.call(0x0014_f308, &[CM], &[q]).1;
        m.wv(sp + 0x20, v);
        let z = m.rf(CM + 0x4c);
        m.wf(sp + 0x28, z);
        let a = m.rv(sp + 0x20);
        let v = m.call(0x0014_f268, &[CM], &[a]).1;
        m.wv(sp + 0x20, v);
        let k0 = m.r32(sp + 0xb4);
        let k1 = m.r32(sp + 0xb8);
        let q = m.rv(s0);
        let r = m.call(0x0014_ea08, &[k0, k1, 0, sp + 0xbc], &[q, v, f4(0.0)]).0;
        if r != 0 && 1.0 <= m.rf(sp + 0xbc) {
            let t = m.rf(T);
            m.wf(a0 + 0x80, t);
            break;
        }
    }
    if m.rf(0x002c_4114) < m.rf(T) - m.rf(a0 + 0x80) {
        m.call(0x0014_d8c0, &[CM], &[]);
    }
    m.call(0x001a_e4a0, &[sp + 0xb0, 2], &[]);
    stack_pop(m, 0x180);
}

/// FUN_0019da78: sync state +4 (0 / 2 / other) of a0 with +8 / +0x10 handles: FUN_0019d660 transitions, sound 0x353 swap via DAT_002bf920, params 0x352 / 0x351.
pub fn ee_sub_19da78<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    const G: u32 = 0x002b_f920;
    let h = m.r32(a0 + 0x14);
    if h != 0 {
        m.call(0x0020_2d38, &[h], &[]);
    }
    let v1 = m.r32(a0 + 4);
    if v1 == 0 {
        let v = m.r32(a0 + 8);
        let a3 = v == 0 || m.r32(v + 0x58) == 2;
        let o = m.r32(a0);
        m.w32(G, 0xffff_ffff);
        let a2 = if m.r32(o + 0x20) != 1 {
            true
        } else {
            let v = m.r32(a0 + 0x10);
            v == 0 || m.r32(v + 0x58) == 2
        };
        if a3 && a2 {
            m.call(0x0019_d660, &[a0, 1], &[]);
        }
    } else if (v1 as i32) > 0 {
        if v1 == 2 {
            let v = m.r32(a0 + 8);
            let a1 = v == 0 || (m.r32(v + 0x58) as i32) < 2;
            let o = m.r32(a0);
            let a0f = if m.r32(o + 0x20) != 1 {
                true
            } else {
                let v = m.r32(a0 + 0x10);
                v == 0 || (m.r32(v + 0x58) as i32) < 2
            };
            if a1 && a0f {
                m.call(0x0019_d660, &[a0, 0xffff_ffff], &[]);
            }
            return;
        }
    } else if v1 == 0xffff_ffff {
        return;
    }
    let a1 = m.r32(a0 + 8);
    if a1 == 0 {
        return;
    }
    if m.r32(a0 + 4) != 0 {
        let o0 = m.r32(a0);
        let o1 = m.r32(o0);
        let s1 = m.r32(o1 + 0x10ec);
        let mut done = None;
        if s1 == 4 {
            if m.r32(G) != s1 {
                m.call(0x0021_3b00, &[a1, 0x353], &[]);
            }
            let o0 = m.r32(a0);
            let v1 = m.r32(o0);
            if m.r32(v1 + 0x10ec) == s1 {
                done = Some(m.r32(v1 + 0x10e4));
            }
        }
        let val = match done {
            Some(x) => x,
            None => {
                if m.r32(G) == 4 {
                    let h = m.r32(a0 + 8);
                    m.call(0x0021_3928, &[h, 0x353], &[]);
                }
                let o0 = m.r32(a0);
            let v1 = m.r32(o0);
                m.r32(v1 + 0x10e4)
            }
        };
        m.w32(G, val);
    }
    let o0 = m.r32(a0);
    let v2 = m.r32(o0);
    let f12 = (m.rf(v2 + 0x1a8) + f32::from_bits(0x4049_0fdb)) * f32::from_bits(0x3e22_f983);
    let h = m.r32(a0 + 8);
    m.call(0x0021_2e40, &[h, 0x352], &[f4(f12)]);
    let o = m.r32(a0);
    let h = m.r32(a0 + 8);
    let f = m.rf(o + 0x3c);
    m.call(0x0021_2e40, &[h, 0x351], &[f4(f)]);
}

/// FUN_00202540: refresh +0xa8 owner (slot 96), advance +0x28 by 20; pick nearest visible candidate of +0x34[+0x30] within DAT_002a8550 (2D) -> +0x8c, change -> FUN_00202768(a0, 2).
pub fn ee_sub_202540<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let sp = stack_push(m, 0xd0);
    let o0 = m.r32(a0 + 0xa8);
    let o = m.r32(o0);
    vslot(m, o, 96, &[a0 + 0x20], &[]);
    let f0 = m.rf(a0 + 0x28) + 20.0;
    let mut f21 = m.rf(0x002a_8550);
    m.wf(a0 + 0x28, f0);
    let mut s4 = 0u32;
    if (m.r32(a0 + 0x30) as i32) > 0 {
        let mut i = 0u32;
        loop {
            let s0 = m.r32(a0 + 0x34 + 4 * i);
            if m.call(0x0020_3790, &[s0], &[]).0 != 0 {
                let p = m.call(0x001a_62e8, &[s0], &[]).1;
                m.wv(sp + 0x20, p);
                let e = m.r32(s0 + 0xe0);
                let c = m.rv(CM + 0x90);
                let d = [p[0] - c[0], p[1] - c[1], p[2] - c[2], p[3] - c[3]];
                if m.call(0x0014_f850, &[CM, e], &[d]).0 != 0 {
                    let p2 = m.call(0x001a_62e8, &[s0], &[]).1;
                    let f12 = m.rf(s0 + 0x38);
                    if m.call(0x0014_f898, &[CM, sp + 0x10], &[p2, f4(f12)]).0 != 0 {
                        let p = m.rv(sp + 0x20);
                        let c = m.rv(CM + 0x90);
                        let dx = p[0] - c[0];
                        let dy = p[1] - c[1];
                        let f20 = (dx * dx + dy * dy).sqrt();
                        if f20 < f21 {
                            m.call(0x0020_2378, &[a0, s0], &[]);
                            if m.r32(s0 + 0x108) != 2 {
                                s4 = s0;
                                f21 = f20;
                            }
                        }
                    }
                }
            }
            i += 1;
            if (i as i32) >= (m.r32(a0 + 0x30) as i32) {
                break;
            }
        }
    }
    if s4 != m.r32(a0 + 0x8c) {
        m.w32(a0 + 0x8c, s4);
        m.call(0x0020_2768, &[a0, 2], &[]);
    }
    stack_pop(m, 0xd0);
}

/// FUN_00158988: retire state-1 entries of the +0x30 list (FUN_00158728 when unresolved), then repeatedly resolve the most negative (< -0.01) entry via FUN_0011a770.
pub fn ee_sub_158988<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    const ITER: u32 = 0x002d_d2fc;
    let sp = stack_push(m, 0x80);
    let l = a0 + 0x30;
    m.w32(sp + 4, l);
    m.w32(sp, l);
    let old = m.r32(ITER);
    let stride = m.r32(l + 8);
    m.w32(ITER, sp);
    m.w32(sp + 0xc, old);
    let head = m.r32(a0 + 0x30);
    m.w32(sp + 8, stride);
    m.w32(sp + 4, head.wrapping_add(stride));
    let mut s0 = head;
    while s0 != 0 {
        if m.r32(s0) == 1 {
            if m.r32(s0 + 0x18) == 0 {
                let v = m.call(0x0015_8958, &[a0, s0], &[]).0;
                m.call(0x0015_8728, &[a0, a1, s0, v], &[]);
            }
            m.call(0x0011_a2e8, &[a0 + 0x30, s0], &[]);
            m.call(0x0011_a220, &[a0 + 0xc, s0], &[]);
            let n = m.r32(a0 + 0x3c).wrapping_add(1);
            m.w32(a0 + 0x3c, n);
        }
        let v0 = m.r32(sp + 4);
        let nx = m.r32(v0);
        let st = m.r32(sp + 8);
        m.w32(sp + 4, nx.wrapping_add(st));
        s0 = nx;
    }
    m.call(0x0011_a770, &[a0, 0], &[]);
    let old = m.r32(sp + 0xc);
    m.w32(ITER, old);
    loop {
        let mut v = m.r32(a0 + 0x30);
        let mut f1 = f32::from_bits(0xbc23_d70a);
        let mut s0 = 0u32;
        while v != 0 {
            let f0 = m.rf(v + 0x10);
            if f0 < f1 {
                f1 = f0;
                s0 = v;
            }
            v = m.r32(v + 0x20);
        }
        if s0 == 0 {
            break;
        }
        if m.r32(s0 + 0x18) == 0 {
            let v = m.call(0x0015_8958, &[a0, s0], &[]).0;
            m.call(0x0015_8728, &[a0, a1, s0, v], &[]);
        }
        m.call(0x0011_a770, &[a0, s0], &[]);
    }
    stack_pop(m, 0x80);
}

/// FUN_00127068: build a handle (FUN_00256f48 / FUN_00257038) for a1's +0xc->+0x10 info and a2, count its list until type 0x14, FUN_001294a0(a0, h), release; returns a0.
pub fn ee_sub_127068<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) -> u32 {
    let sp = stack_push(m, 0x60);
    let o0 = m.r32(a1 + 0xc);
    let s1 = m.r32(o0 + 0x10);
    let s2 = m.r32(s1 + 0x10);
    let v = m.call(0x0025_6f48, &[], &[]).0;
    m.call(0x0025_7038, &[sp, v], &[]);
    let p = m.r32(sp);
    m.call(0x0025_6ee0, &[p.wrapping_add(8), a2], &[]);
    let p = m.r32(sp);
    if m.r32(s1 + 8) == 0 {
        m.call(0x0025_6c58, &[p.wrapping_add(0x14), 0], &[]);
    } else {
        let k = m.r32(s1 + 0xc);
        m.call(0x0025_6c58, &[p.wrapping_add(0x14), k], &[]);
    }
    let p = m.r32(sp);
    m.call(0x0025_6c58, &[p.wrapping_add(0x18), s2], &[]);
    let p = m.r32(sp);
    let mut v1 = m.r32(p + 0x14);
    let mut cnt = 0u32;
    let mut hit = 0u32;
    while v1 != 0 {
        if m.r32(v1 + 8) == 0x14 {
            hit = 1;
            break;
        }
        cnt = cnt.wrapping_add(1);
        v1 = m.r32(v1 + 0x10);
    }
    let p = m.r32(sp);
    m.w32(p + 0xc, cnt);
    let p = m.r32(sp);
    m.w32(p + 0x10, hit);
    let p = m.r32(sp);
    m.call(0x0012_94a0, &[a0, p], &[]);
    let h = m.r32(sp);
    if h != 0 {
        let n = m.r32(h).wrapping_sub(1);
        m.w32(h, n);
        if n == 0 {
            m.call(0x0025_7148, &[h, 3], &[]);
        }
    }
    stack_pop(m, 0x60);
    a0
}

/// FUN_0014db70: recompute projection: +0x44 = FUN_002450f0(+0x20 / 2), +0x40, +0x1c, +0x48; FUN_0011c448(+0x100, ...); FUN_0014e270(a0, +0x250, 1); FUN_0014dc58(a0).
pub fn ee_sub_14db70<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let f20 = 1.0f32;
    let fov = m.rf(a0 + 0x20) * 0.5;
    let f0 = m.call(0x0024_50f0, &[], &[f4(fov)]).1[0];
    let f2 = f20 / m.rf(a0 + 0x20);
    let a = m.rf(a0 + 0x3c);
    let f1 = m.rf(a0 + 0x18);
    let mut f3 = a * a;
    m.wf(a0 + 0x44, f0);
    let f12 = f0 * a;
    f3 = f3 + f20;
    m.wf(a0 + 0x40, f12);
    let f1 = f20.max(f1 * f2);
    m.wf(a0 + 0x1c, f1);
    let f2 = f3.sqrt();
    let f2 = f0 * f2;
    let f3 = m.rf(a0 + 0x4c);
    let f0s = f0 * 4096.0;
    let f12s = f12 * 4096.0;
    let f2 = f2 * f3;
    let f17 = m.rf(a0 + 0x50);
    let f13 = 224.0 / f0s;
    let f16 = f3;
    let f12 = 512.0 / f12s;
    let f2 = f2 + f20;
    m.wf(a0 + 0x48, f2);
    m.call(
        0x0011_c448,
        &[a0 + 0x100],
        &[f4(f12), f4(f13), f4(0.0), f4(0.0), f4(f16), f4(f17)],
    );
    m.call(0x0014_e270, &[a0, a0 + 0x250], &[f4(f20)]);
    m.call(0x0014_dc58, &[a0], &[]);
}

/// FUN_0021ec60: format into a 0x420 stack stream: FUN_0020d5f0(&s, a0, 0); n = FUN_0021eac8(&s, a1, a2); a0[n] = 0; FUN_0020d618(&s, 2); returns n.
pub fn ee_sub_21ec60<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) -> u32 {
    let sp = stack_push(m, 0x460);
    m.call(0x0020_d5f0, &[sp, a0, 0], &[]);
    let n = m.call(0x0021_eac8, &[sp, a1, a2], &[]).0;
    m.w8(a0.wrapping_add(n), 0);
    m.call(0x0020_d618, &[sp, 2], &[]);
    stack_pop(m, 0x460);
    n
}

/// FUN_00255a58: +0 = 0; FUN_002558c0(a0, 0x2dd2b0); +4 bit31 = DAT_002dd598 bit0; returns a0.
pub fn ee_sub_255a58<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    m.w32(a0, 0);
    m.call(0x0025_58c0, &[a0, 0x002d_d2b0], &[]);
    let v = (m.r32(a0 + 4) & 0x7fff_ffff) | (m.r32(0x002d_d598) << 31);
    m.w32(a0 + 4, v);
    a0
}

/// FUN_00224350: FUN_001f4fb0(a0, a1); a1 +0x98 = 0.25.
pub fn ee_sub_224350<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.call(0x001f_4fb0, &[a0, a1], &[]);
    m.wf(a1 + 0x98, 0.25);
}
