//! WIRE-W1G2: 1:1 queue ports (against `QMem`) of a batch of live-frontier natives, ported from the EE asm;
//! callees go through `m.call` / `vslot`. QMem convention: integer / pointer args in `a`, quadword
//! (128-bit GPR) and float args in `v` (register order).
#![allow(unused_variables, unused_mut, clippy::all)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}
fn fb(b: u32) -> f32 {
    f32::from_bits(b)
}
/// ACC = r0*v.x + r1*v.y + r2*v.z (xyz), w kept from v
fn mxyz(r0: V4, r1: V4, r2: V4, v: V4) -> V4 {
    [
        r0[0] * v[0] + r1[0] * v[1] + r2[0] * v[2],
        r0[1] * v[0] + r1[1] * v[1] + r2[1] * v[2],
        r0[2] * v[0] + r1[2] * v[1] + r2[2] * v[2],
        v[3],
    ]
}
/// vadday.x + vmaddz.x then sqrt: |v.xyz|
fn len3(v: V4) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}
fn sub4(a: V4, b: V4) -> V4 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2], a[3] - b[3]]
}
/// raw 16-bit-adjusted call through a vtable at `vt` with fn at `vt+off`, adjust at `vt+off-4`
fn vraw<M: QMem + ?Sized>(m: &mut M, obj: u32, vt: u32, off: u32, a: &[u32], v: &[V4]) -> (u32, V4) {
    let f = m.r32(vt + off);
    let adj = m.r16(vt + off - 4) as i16 as i32 as u32;
    let mut args = vec![obj.wrapping_add(adj)];
    args.extend_from_slice(a);
    m.call(f, &args, v)
}

/// FUN_00130d30(o, mode, f12, f13)
pub fn ee_sub_130d30<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, f12: f32, f13: f32) {
    if m.call(0x0013_1968, &[a0, a1], &[f4(f12), f4(f13)]).0 != 0 {
        return;
    }
    let s1 = a0;
    let p = m.rf(s1 + 0x48) * f13;
    m.wf(s1 + 0x44, f13);
    let p = p * m.rf(s1 + 0x4c);
    m.wf(s1 + 0x40, p);
    let f1 = m.rf(s1 + 0x1c);
    // mode outside 0..=2 leaves f22 uninitialised (callee-saved garbage); treated as unchanged
    let f22 = match a1 as i32 {
        1 => f1 + f12,
        0 => f12,
        2 => {
            let v = m.r32(s1 + 0x14);
            m.rf(v + 0x74) + f12
        }
        _ => f1,
    };
    let f0;
    if f22 == f1 {
        f0 = m.rf(s1 + 0x1c);
    } else {
        m.wf(s1 + 0x1c, f22);
        m.call(0x0013_17d8, &[s1, s1 + 0x1c, s1 + 0x40], &[]);
        m.call(0x0013_0eb0, &[s1], &[]);
        let mut s0 = m.r32(s1 + 0x34);
        while s0 != 0 {
            if m.call(0x0012_cee0, &[s0], &[]).0 == 0 {
                s0 = m.r32(s0 + 0x28);
                continue;
            }
            let v1 = m.r32(s0 + 8);
            let p = m.r32(v1 + 0x88);
            let lo = m.rf(v1 + 0x80);
            let x = m.rf(p);
            let fire = if x < lo { true } else { m.rf(v1 + 0x84) < x };
            if fire {
                let vt = m.r32(s0 + 0x18);
                vraw(m, s0, vt, 0x24, &[], &[]);
            }
            let a = m.r32(s0 + 4);
            let b = m.r32(s0 + 8);
            m.call(0x0023_0598, &[a, b], &[]);
            s0 = m.r32(s0 + 0x28);
        }
        m.call(0x0023_0cf0, &[s1, 0], &[]);
        f0 = m.rf(s1 + 0x1c);
    }
    m.wf(s1 + 0x20, f0);
}

/// FUN_00181060(o, msg, ev)
pub fn ee_sub_181060<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    m.call(0x001c_3e20, &[a0, a1, a2], &[]);
    if a1 != 9 {
        return;
    }
    let s1 = a0;
    let e0 = m.r32(a2);
    if e0 != m.r32(s1 + 0x194) {
        let v = m.r32(s1 + 0x198);
        if v == 0 || e0 != m.r32(v + 0x30) {
            return;
        }
    }
    match m.r32(a2 + 8) {
        0x27 => m.w32(s1 + 0x1cc, 1),
        0x25 => m.w32(s1 + 0x1c8, 1),
        0x26 => m.w32(s1 + 0x1c8, 0),
        0x28 => m.w32(s1 + 0x1cc, 0),
        0x15e => {
            let sp = stack_push(m, 0xa0);
            let ax = m.rv(0x002a_2dc0);
            let f = m.rf(s1 + 0x204);
            m.call(0x0011_b500, &[sp + 0x30], &[ax, f4(f)]);
            let r2 = m.rv(sp + 0x50);
            let r0 = m.rv(sp + 0x30);
            let r1 = m.rv(sp + 0x40);
            m.wv(sp + 0x20, r2);
            m.wv(sp, r0);
            m.wv(sp + 0x10, r1);
            let e0 = m.r32(a2);
            let f = m.rf(a2 + 4);
            let v = m.call(0x0020_4e38, &[s1, e0], &[f4(f)]).1;
            let r = mxyz(r0, r1, r2, v);
            m.wv(s1 + 0x210, r);
            vslot(m, s1, 34, &[], &[r]);
            stack_pop(m, 0xa0);
        }
        _ => {}
    }
}

fn unlink_255<M: QMem + ?Sized>(m: &mut M, a3: u32, tab: u32, stride: u32, g: u32) {
    let w = m.r32(a3 + 4);
    let lo = w & 0x1fff;
    let hi = (w >> 13) & 3;
    let t1 = if lo == 0 && hi == 0 { 0 } else { m.r32(tab + hi.wrapping_mul(4)).wrapping_add(lo.wrapping_mul(stride)) };
    let w = m.r32(a3 + 4);
    let lo = (w >> 16) & 0x1fff;
    let hi = (w >> 29) & 3;
    let t0 = if lo == 0 && hi == 0 { 0 } else { m.r32(tab + hi.wrapping_mul(4)).wrapping_add(lo.wrapping_mul(stride)) };
    if a3 == m.r32(g + 8) {
        m.w32(g + 8, t0);
    }
    if t1 != 0 {
        let a0 = ((m.r16(a3 + 6) as u32) & 0x1fff) << 16;
        let v1 = (m.r32(t1 + 4) & 0xe000_ffff) | a0;
        m.w32(t1 + 4, v1);
        let v1 = (v1 & 0x9fff_ffff) | (m.r32(a3 + 4) & 0x6000_0000);
        m.w32(t1 + 4, v1);
    } else if a3 == m.r32(g) {
        m.w32(g, t0);
    } else if a3 == m.r32(g + 0xc) {
        m.w32(g + 0xc, t0);
    }
    if t0 != 0 {
        let v1 = (m.r32(t0 + 4) & 0xffff_e000) | (m.r32(a3 + 4) & 0x1fff);
        m.w32(t0 + 4, v1);
        let v1 = (v1 & 0xffff_9fff) | (m.r32(a3 + 4) & 0x6000);
        m.w32(t0 + 4, v1);
    }
}

/// FUN_00255438(node): unlink from the 0x18-stride packed list (tables 0x4520c0, heads 0x2dd290)
pub fn ee_sub_255438<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    unlink_255(m, a0, 0x0045_20c0, 0x18, 0x002d_d290);
}

/// FUN_00256120(node): unlink from the 0x14-stride packed list (tables 0x452160, heads 0x2dd2d0)
pub fn ee_sub_256120<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    unlink_255(m, a0, 0x0045_2160, 0x14, 0x002d_d2d0);
}

/// FUN_001731b8(o, a1)
pub fn ee_sub_1731b8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let sp = stack_push(m, 0x80);
    let s0 = sp + 0x10;
    m.call(0x001a_e478, &[sp], &[]);
    m.w32(sp + 0x10, 0xe00);
    m.w32(s0 + 4, 0x10);
    let v = m.call(0x001a_e378, &[s0], &[]).0;
    m.w32(a0 + 0x54, v);
    m.w32(sp + 0x10, 0x100);
    m.w32(s0 + 4, 0x10);
    let v = m.call(0x001a_e318, &[s0], &[]).0;
    m.w32(sp + 0x10, 0x20);
    m.w32(a0 + 0x9c, v);
    m.w32(s0 + 4, 0x10);
    let v = m.call(0x001a_e318, &[s0], &[]).0;
    m.w32(a0 + 0x94, v);
    let q = m.rv(0x002a_2dd0);
    m.call(0x0017_33c8, &[a0], &[q]);
    m.call(0x001a_5c28, &[a0, a1], &[]);
    let n = m.r32(a0 + 0x50).wrapping_mul(0x1c);
    let p = m.r32(a0 + 0x54);
    m.w32(sp + 0x10, n);
    m.w32(s0 + 4, 0x10);
    let v = m.call(0x001a_e238, &[s0, p], &[]).0;
    let n = m.r32(a0 + 0x98) << 4;
    let p = m.r32(a0 + 0x9c);
    m.w32(a0 + 0x54, v);
    m.w32(sp + 0x10, n);
    m.w32(s0 + 4, 0x10);
    let v = m.call(0x001a_e238, &[s0, p], &[]).0;
    let p = m.r32(a0 + 0x94);
    m.w32(sp + 0x10, 0x20);
    m.w32(a0 + 0x9c, v);
    m.w32(s0 + 4, 0x10);
    let v = m.call(0x001a_e238, &[s0, p], &[]).0;
    m.w32(a0 + 0x94, v);
    m.call(0x001a_e4a0, &[sp, 2], &[]);
    stack_pop(m, 0x80);
}

/// FUN_00150610(o, a1, a2)
pub fn ee_sub_150610<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    let s0 = a0;
    let s1 = s0.wrapping_add(0x34);
    if s1 != 0 && m.r32(s0 + 0x34) != 0 {
        let a = m.r32(s0 + 0x34);
        m.call(0x001f_b098, &[a], &[f4(0.0)]);
        m.call(0x0025_8040, &[s1, 0], &[]);
    }
    let s1 = s0.wrapping_add(0x38);
    if s1 != 0 && m.r32(s0 + 0x38) != 0 {
        let a = m.r32(s0 + 0x38);
        m.call(0x001f_b098, &[a], &[f4(0.5)]);
        m.call(0x0025_8040, &[s1, 0], &[]);
    }
    let a = m.r32(s0);
    let pi = fb(0x4049_0fdb);
    let (x, y) = if m.call(0x0013_2b10, &[a, 0x34], &[]).0 != 0 { (0x10, 0x14) } else { (0x4c, 0x50) };
    let f12 = m.rf(s0 + x) + pi;
    let f13 = -m.rf(s0 + y);
    m.call(0x0014_d8d8, &[0x002d_df40], &[f4(f12), f4(f13)]);
    m.call(0x0014_fce8, &[s0, a1, a2], &[]);
}

/// FUN_00152250(o, msg, ev)
pub fn ee_sub_152250<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    m.call(0x001c_3e20, &[a0, a1, a2], &[]);
    if a1 != 9 {
        return;
    }
    let e0 = m.r32(a2);
    let s0 = m.r32(e0 + 0x18);
    if s0 != a0 {
        return;
    }
    match m.r32(a2 + 8) {
        0x28 => m.w32(s0 + 0x40c, 0),
        0x27 => m.w32(s0 + 0x40c, 1),
        0x15e => {
            let sp = stack_push(m, 0x50);
            let f = m.rf(a2 + 4);
            let v = m.call(0x0020_4e38, &[s0, e0], &[f4(f)]).1;
            m.wv(sp, [-1.0, fb(0xffff_ffff), 0.0, 0.0]);
            m.wv(s0 + 400, [-v[0], -v[1], -v[2], -v[3]]);
            let r0 = m.rv(s0 + 0x1b0);
            let r2 = m.rv(s0 + 0x1d0);
            let r1 = m.rv(s0 + 0x1c0);
            let r = mxyz(r0, r1, r2, v);
            vslot(m, s0, 34, &[], &[r]);
            stack_pop(m, 0x50);
        }
        _ => {}
    }
}

/// FUN_001da858(o)
pub fn ee_sub_1da858<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let v1 = m.r32(a0 + 0x10);
    if v1 == 0 {
        return;
    }
    let f1 = m.rf(v1 + 0x1044);
    if m.r32(a0 + 0x28) as i32 == -1 {
        if fb(0x3f4c_cccd) < f1 {
            let f13 = m.rf(v1 + 0x103c);
            let f12 = m.rf(v1 + 0x1040);
            let f0 = m.call(0x0024_58f0, &[], &[f4(f12), f4(f13)]).1[0];
            let v = (f0 * fb(0x3fa2_f983) + 8.5) as i32;
            let a = if -1 < v { v } else { v.wrapping_add(7) };
            let r = v.wrapping_sub((a >> 3) << 3);
            vslot(m, a0, 13, &[r as u32], &[]);
        }
    } else if f1 < 0.5 {
        vslot(m, a0, 13, &[0xffff_ffff], &[]);
    }
}

/// FUN_00205588(o, mask)
pub fn ee_sub_205588<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let mut p = a0 + 0x2a0;
    for i in 0..6u32 {
        if a1 & (1 << i) != 0 {
            let v = m.r32(p);
            if v != 0 {
                let v = v.wrapping_sub(1);
                m.w32(p, v);
                if (v as i32) <= 0 {
                    let r = m.call(0x001f_d0c0, &[0x003d_d040, i], &[]).0;
                    let r = m.call(0x0015_e938, &[0x003d_d490, 4, r, a0], &[]).0;
                    if r != 0 {
                        let r = m.call(0x001f_03c8, &[r, a0], &[]).0;
                        if r != 0 {
                            m.call(0x001f_0598, &[r], &[]);
                        }
                    }
                }
            }
        }
        p += 4;
    }
}

/// FUN_002240b0(o)
pub fn ee_sub_2240b0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let sp = stack_push(m, 0x30);
    m.call(0x001f_36e0, &[a0], &[]);
    if m.r32(a0 + 0x150) == 1 && m.r32(a0 + 0x158) as i32 != -1 {
        let a = m.r32(a0 + 0x1a0);
        m.call(0x0022_4258, &[a0, a], &[]);
    }
    if m.r32(a0 + 0x16c) != 0 {
        let p = m.call(0x001a_62e8, &[a0], &[]).1;
        let o = m.r32(a0 + 0x16c);
        m.wv(sp, p);
        let q = m.call(0x001a_62e8, &[o], &[]).1;
        let f1 = m.rf(a0 + 0x170);
        let p = m.rv(sp);
        let d = len3(sub4(p, q));
        if f1 < d {
            vslot(m, a0, 74, &[0], &[]);
        }
    }
    stack_pop(m, 0x30);
}

/// FUN_00227bb8(o, a1)
pub fn ee_sub_227bb8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let sp = stack_push(m, 0x60);
    m.call(0x0023_1170, &[a0, a1], &[]);
    let s1 = a0 + 0xf0;
    m.call(0x0011_7fd0, &[a1, s1], &[]);
    m.call(0x0011_8220, &[a1, s1, a0 + 0xe0], &[]);
    let n = m.r32(a0 + 0xf0);
    if (n as i32) <= 0 {
        let f = m.rf(0x002a_9880);
        m.wf(a0 + 0x120, f);
    } else {
        let b = m.r32(a0 + 0xf4);
        let p = m.call(0x0011_f288, &[n, b], &[]).1;
        let n = m.r32(a0 + 0xf0);
        let b = m.r32(a0 + 0xf4);
        m.wv(sp + 0x10, p);
        let q = m.call(0x0011_f2b8, &[n, b], &[]).1;
        let p = m.rv(sp + 0x10);
        let c = [p[0] * 0.5 + q[0] * 0.5, p[1] * 0.5 + q[1] * 0.5, p[2] * 0.5 + q[2] * 0.5, p[3] * 0.5 + q[3] * 0.5];
        m.wv(sp, [0.5, 0.0, 0.0, 0.0]);
        let d = sub4(c, p);
        m.wv(a0 + 0x110, c);
        m.wf(a0 + 0x120, d[0] * d[0] + d[1] * d[1] + d[2] * d[2]);
    }
    stack_pop(m, 0x60);
}

/// FUN_0020ab68(o, a1, a2, q)
pub fn ee_sub_20ab68<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: V4) {
    let v0 = m.r32(a0 + 0xf00);
    if v0 != 0 {
        let a = m.r32(v0 + 0xc0);
        m.call(0x0020_9360, &[a], &[]);
        let v0 = m.r32(a0 + 0xf00);
        let f20 = m.rf(0x002d_ded0);
        let f13 = m.rf(v0 + 0x64);
        let f12 = m.rf(v0 + 0x60);
        let f0 = m.call(0x0011_d448, &[], &[f4(f12), f4(f13)]).1[0];
        m.w32(a0 + 0xf00, 0);
        m.wf(a0 + 0xf04, f20 + f0);
    }
    m.w32(a0 + 0xf00, a1);
    if a1 != 0 {
        let p = m.r32(a1 + 0xc0);
        let f = m.r32(p + 0xc) | 1;
        m.call(0x0020_9278, &[p, a0, f], &[f4(0.0)]);
        let f = m.rf(0x002a_88c4);
        m.wf(a0 + 0xf04, f);
    }
    m.call(0x0020_ac28, &[a0, a1, a2], &[a3]);
}

/// FUN_001819e0(o)
pub fn ee_sub_1819e0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0017_f6b0, &[a0], &[]);
    let mut s1 = m.r32(a0 + 0x28c);
    while s1 != 0 {
        let s0 = m.r32(s1 + 0x60);
        m.call(0x0011_a210, &[s1 + 0x60], &[]);
        let a = m.r32(s1 + 0x20);
        m.call(0x0011_a220, &[a.wrapping_add(0x10), s1], &[]);
        s1 = s0;
    }
    m.call(0x0011_a200, &[a0 + 0x28c], &[]);
    if (m.r32(a0 + 0x1c8) as i32) > 0 {
        let mut s0 = a0 + 0x1cc;
        let mut i = 0i32;
        loop {
            let v = m.r32(s0 + 4);
            i += 1;
            m.w32(s0 + 0xc, v);
            m.call(0x0018_1a90, &[a0, s0], &[]);
            s0 += 0x30;
            if i >= m.r32(a0 + 0x1c8) as i32 {
                break;
            }
        }
    }
}

/// FUN_0014a008(o, a1)
pub fn ee_sub_14a008<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let sp = stack_push(m, 0x70);
    let s0 = sp + 0x10;
    m.call(0x001a_e478, &[sp], &[]);
    m.w32(sp + 0x10, 0xc00);
    m.w32(s0 + 4, 0x10);
    let v = m.call(0x001a_e378, &[s0], &[]).0;
    m.w32(a0 + 0x58, v);
    m.call(0x001a_5c28, &[a0, a1], &[]);
    let n = m.r32(a0 + 0x54).wrapping_mul(0xc);
    let p = m.r32(a0 + 0x58);
    m.w32(sp + 0x10, n);
    m.w32(s0 + 4, 0x10);
    let v = m.call(0x001a_e238, &[s0, p], &[]).0;
    m.w32(a0 + 0x58, v);
    m.call(0x001a_e4a0, &[sp, 2], &[]);
    stack_pop(m, 0x70);
}

/// FUN_00204ca8(o)
pub fn ee_sub_204ca8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let sp = stack_push(m, 0x40);
    let v1 = m.r32(a0 + 0x5c);
    let q = m.rv(v1 + 0xb0);
    m.wv(sp, q);
    let s1 = vslot(m, a0, 30, &[], &[]).0;
    let v0 = vslot(m, a0, 30, &[], &[]).0;
    let f13 = m.rf(v0);
    let f12 = m.rf(s1 + 4);
    let f0 = m.call(0x0024_58f0, &[], &[f4(f12), f4(f13)]).1[0];
    let f13 = m.rf(a0 + 0x1a8);
    m.call(0x0011_d2d0, &[a0 + 0x1f4, sp | 8], &[f4(f0), f4(f13), f4(0.0)]);
    let q = m.rv(sp);
    vslot(m, a0, 35, &[], &[q]);
    stack_pop(m, 0x40);
}

/// FUN_001a6fc0(from, to, v) -> v transformed by from's rotation then by to's inverse
pub fn ee_sub_1a6fc0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: V4) -> V4 {
    let sp = stack_push(m, 0x60);
    m.wv(sp + 0x30, a2);
    if a0 != 0 {
        let p = m.call(0x001a_6318, &[a0], &[]).0;
        let r2 = m.rv(p + 0x20);
        let r0 = m.rv(p);
        let r1 = m.rv(p + 0x10);
        let v = m.rv(sp + 0x30);
        m.wv(sp + 0x30, mxyz(r0, r1, r2, v));
    }
    if a1 != 0 {
        let p = m.call(0x001a_6318, &[a1], &[]).0;
        m.call(0x0011_b340, &[sp, p], &[]);
        let v = m.rv(sp + 0x30);
        let r0 = m.rv(sp);
        let r1 = m.rv(sp + 0x10);
        let r2 = m.rv(sp + 0x20);
        m.wv(sp + 0x30, mxyz(r0, r1, r2, v));
    }
    let r = m.rv(sp + 0x30);
    stack_pop(m, 0x60);
    r
}

/// FUN_0017a730(o)
pub fn ee_sub_17a730<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001e_ef70, &[a0], &[]);
    m.call(0x0017_a8c8, &[a0, 0], &[]);
    let a = m.r32(a0 + 0x10);
    m.call(0x0020_b210, &[a], &[]);
    let a = m.r32(a0 + 0x10);
    let q = m.rv(0x002a_2dd0);
    m.call(0x0020_ab68, &[a, 0, 0], &[q]);
    let a = m.r32(a0 + 0x10);
    m.call(0x001f_4e28, &[a], &[]);
    m.call(0x001e_efc0, &[a0], &[]);
    let v = m.r32(a0 + 0x10);
    let a = m.r32(v + 0xf64);
    m.call(0x0015_3ff8, &[a], &[]);
    let a1 = m.r32(a0 + 0x3c);
    if a1 != 0 {
        let v = m.r32(a0 + 0x10);
        let a = m.r32(v + 0xf64);
        m.call(0x0015_4038, &[a, a1, 2], &[]);
    }
}

fn maff(r0: V4, r1: V4, r2: V4, t: V4, v: V4) -> V4 {
    let x = mxyz(r0, r1, r2, v);
    [x[0] + t[0], x[1] + t[1], x[2] + t[2], v[3]]
}

/// FUN_0022fc48(from, to, v) -> point transformed by from's world matrix then by to's inverse
pub fn ee_sub_22fc48<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: V4) -> V4 {
    let mut v = a2;
    if a0 != 0 {
        let (r0, r1, r2, t) = (m.rv(a0 + 0x40), m.rv(a0 + 0x50), m.rv(a0 + 0x60), m.rv(a0 + 0x70));
        v = maff(r0, r1, r2, t, v);
    }
    if a1 != 0 {
        let sp = stack_push(m, 0x60);
        m.wv(sp + 0x40, v);
        m.call(0x0011_c318, &[sp, a1 + 0x40], &[]);
        let (r0, r1, r2, t) = (m.rv(sp), m.rv(sp + 0x10), m.rv(sp + 0x20), m.rv(sp + 0x30));
        let w = m.rv(sp + 0x40);
        v = maff(r0, r1, r2, t, w);
        stack_pop(m, 0x60);
    }
    v
}

/// FUN_00231220(a0, a1, pp): rebase *pp from a1's block array into a0's
pub fn ee_sub_231220<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    let t0 = m.r32(a2);
    if t0 == 0 {
        return;
    }
    let b = m.r32(a1 + 0x10);
    let i = (t0.wrapping_sub(m.r32(b + 0x54)) as i32) >> 8;
    let mut a3 = 0u32;
    if i >= 0 && i < m.r32(b + 0x50) as i32 {
        let v = m.r32(a0 + 0x10);
        a3 = m.r32(v + 0x54).wrapping_add((i << 8) as u32);
    }
    if m.r32(a3.wrapping_add(0x88)) == t0.wrapping_add(0x80) {
        m.w32(a3.wrapping_add(0x88), a3.wrapping_add(0x80));
    }
    if m.r32(a3.wrapping_add(0x8c)) == t0.wrapping_add(0x84) {
        m.w32(a3.wrapping_add(0x88), a3.wrapping_add(0x84));
    }
    m.w32(a2, a3);
}

/// FUN_001f4fc8(a, b)
pub fn ee_sub_1f4fc8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    if m.r32(a0 + 0x148) & 0x0400_0000 == 0 && vslot(m, a0, 61, &[], &[]).0 == 0 {
        return 0;
    }
    (vslot(m, a1, 61, &[], &[]).0 != 0) as u32
}

/// FUN_00227380(o, a1)
pub fn ee_sub_227380<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.call(0x0023_1768, &[a0, a1], &[]);
    let v = m.call(0x0011_7c80, &[a1], &[]).0;
    m.w32(a0 + 0xe0, v);
    m.call(0x0011_7fd0, &[a1, a0 + 0xf8], &[]);
    m.call(0x0011_8220, &[a1, a0 + 0xf8, a0 + 0xe8], &[]);
    m.call(0x0013_f810, &[a0, a1], &[]);
}

/// FUN_00151698(o, a1, ev)
pub fn ee_sub_151698<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    m.call(0x0014_fce8, &[a0, a1, a2], &[]);
    if m.r32(a2 + 0xc) != 0 {
        m.call(0x0014_d8c0, &[0x002d_df40], &[]);
    }
    let v = m.r32(a0 + 0x10);
    let o = m.r32(v + 0x14);
    vslot(m, o, 1, &[], &[]);
    m.w32(a0 + 0x10, 0);
}

/// FUN_00258040(slot, obj): retain obj, release old, store
pub fn ee_sub_258040<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if a1 != 0 {
        vslot(m, a1, 0, &[], &[]);
    }
    let o = m.r32(a0);
    if o != 0 {
        vslot(m, o, 1, &[], &[]);
    }
    m.w32(a0, a1);
}

/// FUN_00141cd0(o, idx)
pub fn ee_sub_141cd0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if a1 as i32 != -1 {
        let v = m.call(0x0014_41a0, &[a0, a1], &[]).0;
        let t = m.r32(0x002b_cd00u32.wrapping_add(a1 << 5) + 0xc);
        m.call(0x0014_41c0, &[a0, v.wrapping_sub(t)], &[]);
    }
    let f = m.r32(0x002d_ded0);
    m.w32(a0 + 0x1304, f);
}

/// FUN_00211ca8(o)
pub fn ee_sub_211ca8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    if m.r32(a0 + 0x7c) as i32 != -1 {
        return;
    }
    m.call(0x0021_28e8, &[a0], &[]);
    m.call(0x0020_efc0, &[0x003d_d490, 0], &[]);
    m.w32(a0 + 0x110, 1);
    m.call(0x0021_3b00, &[a0, 0x33a], &[]);
}

/// FUN_002312a0(o, v)
pub fn ee_sub_2312a0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: V4) {
    let a = m.r32(a0 + 0x50);
    let v = m.call(0x0022_fc48, &[0, a], &[a1]).1;
    m.call(0x0025_6d18, &[a0 + 0x90], &[v]);
    m.wf(a0 + 0x9c, 1.0);
    m.call(0x0023_13f0, &[a0], &[]);
}

/// FUN_001dc4b0(o, state)
pub fn ee_sub_1dc4b0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if a1 == m.r32(a0 + 0x28) {
        return;
    }
    m.w32(a0 + 0x28, a1);
    let f = m.r32(0x002d_df0c);
    m.w32(a0 + 0x2c, f);
    let up = match a1 {
        0 => {
            m.call(0x001d_c260, &[a0, 1], &[]);
            return;
        }
        4 => {
            m.call(0x001d_c260, &[a0, 0], &[]);
            return;
        }
        1..=3 => {
            let i = m.r32(a0 + 0x24c) as i32;
            if i > 0 {
                m.call(0x001d_bbe0, &[a0, i.wrapping_sub(1) as u32], &[]);
                true
            } else {
                false
            }
        }
        5..=7 => {
            let n = (m.r32(a0 + 0x48) as i32).wrapping_sub(1);
            let i = m.r32(a0 + 0x24c) as i32;
            if i < n {
                m.call(0x001d_bbe0, &[a0, i.wrapping_add(1) as u32], &[]);
                true
            } else {
                false
            }
        }
        _ => return,
    };
    let v = m.call(0x001f_bb90, &[if up { 0x35 } else { 0x36 }], &[]).0;
    m.call(0x001f_b2f8, &[v, 0], &[]);
}

/// FUN_001e6f10(a0, a1, a2)
pub fn ee_sub_1e6f10<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    let f = m.call(0x001e_64c8, &[a0, a1, a2], &[]).1[0];
    m.call(0x001e_67b8, &[a0, a1, a2], &[f4(f)]);
}

/// FUN_001f2ad0(o, p)
pub fn ee_sub_1f2ad0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let hi = m.r32(a1 + 0x14);
    if hi & 0x0004_0000 != 0 {
        return;
    }
    m.w32(a1 + 0x14, hi | 0x0004_0000);
    if m.r32(a0 + 0x2c) & 0x1000_0000 != 0 {
        m.call(0x001f_2930, &[a0, a1], &[]);
    }
}

/// FUN_0011d3f8(lo, hi): random int in [lo, hi]
pub fn ee_sub_11d3f8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let r = m.call(0x0011_d3c0, &[a0, a1], &[]).0 as i32;
    let d = (a1 as i32).wrapping_sub((a0 as i32).wrapping_sub(1));
    let h = if d == 0 { 0 } else { r.wrapping_rem(d) };
    a0.wrapping_add(h as u32)
}

/// FUN_0017f6b0(o)
pub fn ee_sub_17f6b0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let v = m.r32(a0 + 0x180);
    m.w32(a0 + 0x184, v);
    if m.r32(a0 + 0x18c) != 0 {
        let s = m.r32(0x002a_5638);
        m.call(0x0014_9b58, &[0x003d_d490, 1, a0, s, 0], &[]);
    }
}

/// FUN_00220f88(o)
pub fn ee_sub_220f88<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5848, &[a0], &[]);
    let s = m.r32(0x002a_96e8);
    m.call(0x0014_9b58, &[0x003d_d490, 1, a0, s, 0], &[]);
}

/// FUN_002129e8(o)
pub fn ee_sub_2129e8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0021_2a50, &[a0, 1], &[]);
    if m.r32(a0 + 0x78) as i32 == -1 {
        m.call(0x0021_2a70, &[a0, 1], &[]);
    }
}

/// FUN_00161868(o)
pub fn ee_sub_161868<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let s0 = m.r32(a0 + 0x3bac);
    if s0 != 0 {
        vslot(m, s0, 48, &[0xffff_ffff], &[]);
        m.call(0x0021_2a30, &[s0], &[]);
    }
}

/// FUN_0015e988(idx, v): is v in list idx
pub fn ee_sub_15e988<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let mut p = a0.wrapping_mul(0x14).wrapping_add(0x002b_aae8);
    while p != 0 {
        if m.r32(p + 4) == a1 {
            return 1;
        }
        p = m.r32(p);
    }
    0
}

/// FUN_002064d8(o)
pub fn ee_sub_2064d8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001f_2d48, &[a0], &[]);
    m.call(0x0020_8540, &[a0, 0], &[]);
    m.call(0x001e_ecd0, &[a0 + 0x26c], &[]);
    m.call(0x0013_0658, &[a0 + 0x34c], &[]);
}

/// FUN_001eecd0(o)
pub fn ee_sub_1eecd0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    if (m.r32(a0 + 4) as i32) > 0 {
        let sp = stack_push(m, 0x10);
        let v = m.r32(a0 + 8);
        m.w32(sp, v);
        m.call(0x001e_f0d8, &[sp], &[]);
        stack_pop(m, 0x10);
    }
    m.w32(a0 + 4, 0);
}

/// FUN_00165c98(out, outbuf)
pub fn ee_sub_165c98<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let sp = stack_push(m, 0x30);
    m.w32(a0, 0);
    let n = m.r32(0x003e_0f8c);
    m.w32(sp + 4, 0x10);
    m.w32(sp, n << 2);
    let v = m.call(0x001a_e378, &[sp], &[]).0;
    m.w32(a1, v);
    m.call(0x001a_e478, &[sp + 0x10], &[]);
    let n = m.r32(0x003e_0f8c);
    m.w32(sp + 4, 0x10);
    m.w32(sp, n << 2);
    let buf = m.call(0x001a_e378, &[sp], &[]).0;
    let n = m.r32(0x003e_0f8c);
    let cnt = m.call(0x0015_e9c8, &[0x003d_d490, 5, 0x85, 0, n, buf], &[]).0 as i32;
    if cnt > 0 {
        for i in 0..cnt as u32 {
            let o = m.r32(buf + 4 * i);
            m.w32(o + 0xfc, 0);
        }
        for i in 0..cnt as u32 {
            let o = m.r32(buf + 4 * i);
            if m.r32(o + 0x94) == 0 {
                let b = m.r32(a1);
                m.call(0x0016_4100, &[o, a0, b, 0, 0], &[]);
            }
        }
    }
    m.call(0x001a_e4a0, &[sp + 0x10, 2], &[]);
    stack_pop(m, 0x30);
}

/// FUN_001f2c30(o)
pub fn ee_sub_1f2c30<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5800, &[a0], &[]);
    m.call(0x0022_faa0, &[a0], &[]);
    m.call(0x001f_2b20, &[a0], &[]);
    let v1 = m.r32(a0 + 0x40);
    if (v1 ^ 1) & 1 == 0 {
        m.w32(a0 + 0x40, v1 | 0x14);
    } else {
        let w = m.r32(v1) | 0x14;
        m.w32(v1, w);
    }
    let e = m.r32(a0 + 0xe8);
    let f = if e == 0 { 0 } else { m.r32(e + 0xdc) };
    if f != 0 {
        let g = m.r32(0x003e_1a5c);
        if g != 0 && m.r32(g + 0x2c) != 0 {
            let q = m.call(0x001a_62e8, &[a0], &[]).1;
            let r = m.call(0x001c_0d70, &[1], &[q]).0;
            m.call(0x001f_37c8, &[a0, r], &[]);
        } else {
            let s = m.r32(0x002a_8270);
            m.call(0x0014_9a50, &[0x003d_d490, a0, s, 0], &[]);
        }
    }
    let mut s0 = m.r32(a0 + 0x64);
    while s0 != m.r32(a0 + 0x70) {
        vslot(m, s0, 10, &[], &[]);
        s0 = m.r32(s0 + 0x24);
    }
}

/// FUN_0017eb88(o)
pub fn ee_sub_17eb88<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001e_ef70, &[a0], &[]);
    let f1 = m.rf(a0 + 0x38);
    let o = m.r32(a0 + 0x10);
    let q = m.call(0x001a_62e8, &[o], &[]).1;
    let p = m.r32(a0 + 0x30);
    let ints = [p, 0x003d_d020, a0 + 0x40, 0, 0, a0 + 0x38];
    if 0.0 <= f1 {
        let f12 = m.rf(a0 + 0x38);
        m.call(0x001e_7380, &ints, &[q, f4(f12)]);
    } else {
        m.call(0x001e_6f60, &ints, &[q]);
    }
    let o = m.r32(a0 + 0x10);
    let s0 = m.call(0x0020_5438, &[o, 0xba], &[]).0;
    let v = m.r32(a0 + 0x50);
    m.w32(s0 + 0xa8, v);
    let v = m.r32(a0 + 0x54);
    m.w32(s0 + 0xac, v);
    let v = m.r32(a0 + 0x58);
    m.call(0x0017_ecf0, &[a0, 1], &[]);
    m.w32(s0 + 0xbc, v);
    vslot(m, s0, 7, &[], &[]);
}

fn dot3(a: V4, b: V4) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// FUN_0011ee08(p, a, b, out_pt, out_t, out_dist): closest point on segment a..b to p
pub fn ee_sub_11ee08<M: QMem + ?Sized>(m: &mut M, a3: u32, t0: u32, t1: u32, a0: V4, a1: V4, a2: V4) {
    let d = sub4(a2, a1);
    let e = sub4(a0, a1);
    let mut t = dot3(e, d) / dot3(d, d);
    if t < 0.0 {
        t = 0.0;
    } else if 1.0 < t {
        t = 1.0;
    }
    if t0 != 0 {
        m.wf(t0, t);
    }
    if a3 != 0 || t1 != 0 {
        let p = [a1[0] + d[0] * t, a1[1] + d[1] * t, a1[2] + d[2] * t, a1[3] + d[3] * t];
        if a3 != 0 {
            m.wv(a3, p);
        }
        if t1 != 0 {
            m.wf(t1, len3(sub4(a0, p)));
        }
    }
}

/// FUN_00150938(o)
pub fn ee_sub_150938<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0014_fcc8, &[a0], &[]);
    let s = m.call(0x001f_bb90, &[0x26], &[]).0;
    m.call(0x001f_b2f8, &[s, 0], &[]);
    m.call(0x0025_8040, &[a0 + 0xc0, s], &[]);
    let s = m.call(0x001f_bb90, &[0x25], &[]).0;
    m.call(0x001f_b2f8, &[s, 0], &[]);
    m.call(0x0025_8040, &[a0 + 0xc4, s], &[]);
    let v = m.r32(a0 + 0xc0);
    m.call(0x001f_b018, &[v], &[]);
    m.call(0x0011_bc88, &[0x002d_dfa0, a0 + 0xa4, a0 + 0xa8], &[]);
    m.w32(a0 + 0xa8, 0);
    m.w32(a0 + 0xac, 0);
    m.w32(a0 + 0xb0, 0);
    let f = m.rf(0x002d_df64) / m.rf(a0 + 0xb4);
    m.wf(a0 + 0xb8, f);
    let f = m.rf(0x002d_df64);
    m.wf(a0 + 0xbc, f);
    m.call(0x0014_f610, &[0x002d_df40], &[f4(fb(0x4244_0000))]);
    m.call(0x0015_0ef0, &[a0], &[]);
    let c = m.r32(0x002d_e3e4).wrapping_add(1);
    m.w32(0x002d_e3e4, c);
    let f = m.r32(0x002d_ded0);
    m.w32(0x003e_1be0, f);
}

/// FUN_0017dd38(o, src)
pub fn ee_sub_17dd38<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let v = m.rv(a1);
    m.wv(a0 + 0x40, v);
    let f = m.r32(0x002a_562c);
    let w = m.rv(a1 + 0x10);
    m.w32(a0 + 0xa0, f);
    m.wv(a0 + 0x50, w);
    m.call(0x0017_da28, &[a0, a1], &[]);
}

/// FUN_001b1e00(o)
pub fn ee_sub_1b1e00<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001f_31e0, &[a0], &[]);
    m.call(0x001a_64e8, &[a0, 1, 0x002c_0258], &[]);
}

/// FUN_001fbc20(o)
pub fn ee_sub_1fbc20<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let k = m.r32(a0 + 0x60);
    let p = m.call(0x001f_d028, &[0x003d_d040, k], &[]).0;
    if p == 0 {
        0xffff_ffff
    } else {
        m.r32(p)
    }
}

/// FUN_001f5040(o)
pub fn ee_sub_1f5040<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let v1 = m.r32(a0 + 0xe8);
    if v1 == 0 {
        return 1;
    }
    (m.r32(v1 + 0xc0) == 3 && m.r32(v1 + 0xa0) == 3) as u32
}

/// FUN_001cb9e8(o)
pub fn ee_sub_1cb9e8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_6150, &[a0], &[]);
    m.call(0x0011_a2e8, &[0x003e_13cc, a0], &[]);
}

/// FUN_00183ff0(o)
pub fn ee_sub_183ff0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_6150, &[a0], &[]);
    m.call(0x0011_a2e8, &[0x003e_1414, a0], &[]);
}

/// FUN_0021d358(o)
pub fn ee_sub_21d358<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_6178, &[a0], &[]);
    m.call(0x0011_a220, &[0x003e_13b4, a0], &[]);
}

/// FUN_00209840(o, mat3)
pub fn ee_sub_209840<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if m.r32(a0 + 4) != 0 {
        return;
    }
    let d = m.r32(a0 + 0xc);
    for i in 0..3 {
        let v = m.rv(a1 + 0x10 * i);
        m.wv(d + 0x1b0 + 0x10 * i, v);
    }
}

/// FUN_00131968(o)
pub fn ee_sub_131968<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    (m.call(0x0011_a380, &[0x003e_1218, a0], &[]).0 == 0) as u32
}

/// FUN_0012f990(...) -> *FUN_0012f938() or 0.0
pub fn ee_sub_12f990<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> f32 {
    let p = m.call(0x0012_f938, &[a0, a1], &[]).0;
    if p != 0 {
        m.rf(p)
    } else {
        0.0
    }
}

/// FUN_001fb098(o, f)
pub fn ee_sub_1fb098<M: QMem + ?Sized>(m: &mut M, a0: u32, f12: f32) {
    m.wf(a0 + 0x48, f12);
    m.call(0x001f_b658, &[a0, 0x1fff, 8], &[]);
}

/// FUN_001adf78(o, state)
pub fn ee_sub_1adf78<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if a1 != m.r32(a0 + 0x28) {
        m.w32(a0 + 0x28, a1);
        let f = m.r32(0x002d_df0c);
        m.w32(a0 + 0x2c, f);
    }
}

/// FUN_0011cd98(pool, node): push free node
pub fn ee_sub_11cd98<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let n = m.r32(a0 + 0x10).wrapping_sub(1);
    let h = m.r32(a0 + 0xc);
    m.w32(a0 + 0x10, n);
    m.w32(a1, h);
    m.w32(a0 + 0xc, a1);
}

/// FUN_00117968(...) -> FUN_00117898(...)
pub fn ee_sub_117968<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32) {
    m.call(0x0011_7898, &[a0, a1, a2, a3], &[]);
}

/// FUN_0014f750(o, v)
pub fn ee_sub_14f750<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let f = m.r32(0x002d_ded0);
    m.w32(a0 + 0x61c, a1);
    m.w32(a0 + 0x618, f);
}

/// FUN_001a62d8() -> &DAT_002a2d30
pub fn ee_sub_1a62d8<M: QMem + ?Sized>(m: &mut M) -> u32 {
    0x002a_2d30
}

pub fn ee_sub_1d98a8<M: QMem + ?Sized>(m: &mut M) {}
pub fn ee_sub_1cf570<M: QMem + ?Sized>(m: &mut M) {}
pub fn ee_sub_116a90<M: QMem + ?Sized>(m: &mut M) {}
pub fn ee_sub_182d70<M: QMem + ?Sized>(m: &mut M) {}
pub fn ee_sub_141c00<M: QMem + ?Sized>(m: &mut M) -> u32 {
    0
}

/// FUN_00196928(p, q, f12, f13, f14, f15) -> f32
pub fn ee_sub_196928<M: QMem + ?Sized>(m: &mut M, a0: V4, a1: V4, f12: f32, f13: f32, f14: f32, f15: f32) -> f32 {
    let sp = stack_push(m, 0x30);
    let f22 = a1[2] * 0.5;
    let f24 = -f22;
    let f21 = a0[2];
    let mut f20 = 0.0f32;
    m.wv(sp + 0x20, a0);
    let n = m.call(0x0011_d508, &[sp], &[f4(f24), f4(-f12), f4(f21)]).0 as i32;
    if n > 0 {
        f20 = m.rf(sp + ((n - 1) as u32) * 4).max(f20);
    }
    let n = m.call(0x0011_d508, &[sp + 0x10], &[f4(f22), f4(-f13), f4(f21)]).0 as i32;
    if n > 0 {
        f20 = m.rf(sp + 0x10 + ((n - 1) as u32) * 4).max(f20);
    }
    let v = m.rv(sp + 0x20);
    let l = (v[0] * v[0] + v[1] * v[1]).sqrt();
    let f0 = (f14 * l - f21) / f24;
    if 0.0 < f0 {
        f20 = f0.sqrt().max(f20);
    }
    let f0 = (f15 * l - f21) / f22;
    if 0.0 < f0 {
        f20 = f0.sqrt().max(f20);
    }
    stack_pop(m, 0x30);
    f20
}

/// FUN_001b4888(o)
pub fn ee_sub_1b4888<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let sp = stack_push(m, 0x10);
    m.call(0x0020_39c0, &[a0], &[]);
    m.call(0x001a_64e8, &[a0, 0x89, 0x002c_0270], &[]);
    for (off, k) in [(0x1610u32, 0x200u32), (0x1614, 0x203)] {
        m.w32(sp, 0xe0);
        m.w32(sp + 4, 0x10);
        let v = m.call(0x001a_e2a0, &[sp], &[]).0;
        m.w32(a0 + off, v);
        let v = m.call(0x001f_4c58, &[a0, k, 6], &[]).0;
        let p = m.r32(a0 + off);
        m.w32(p + 4, v);
        let v = m.call(0x001f_4c58, &[a0, k + 1, 6], &[]).0;
        let p = m.r32(a0 + off);
        m.w32(p + 8, v);
        let v = m.call(0x001f_4c58, &[a0, k + 2, 1], &[]).0;
        let p = m.r32(a0 + off);
        m.w32(p + 0xc, v);
        let p = m.r32(a0 + off);
        m.call(0x0018_6930, &[p], &[]);
    }
    let p = m.r32(a0 + 0x1610);
    let v = m.r32(p + 0xc);
    m.w32(a0 + 0x15f0, v);
    let v = m.call(0x001f_4c58, &[a0, 0x202, 6], &[]).0;
    m.w32(a0 + 0x15f4, v);
    let p = m.r32(a0 + 0x1614);
    let v = m.r32(p + 0xc);
    m.w32(a0 + 0x15f8, v);
    let v = m.call(0x001f_4c58, &[a0, 0x205, 6], &[]).0;
    m.w32(a0 + 0x15fc, v);
    stack_pop(m, 0x10);
}

/// FUN_0022c8c0(dst, src, v) -> dst
pub fn ee_sub_22c8c0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: V4) -> u32 {
    let k = m.r32(a1 + 0x94);
    let xyz = |m: &mut M, x: u32, y: u32, z: u32| {
        m.call(0x0023_320c, &[a0, a1, 0x40], &[]);
        m.wf(a0 + x, a2[0]);
        m.wf(a0 + z, a2[2]);
        m.wf(a0 + y, a2[1]);
    };
    match k {
        1 | 4 | 5 | 9 | 10 => xyz(m, 0x30, 0x34, 0x38),
        2 | 3 => {
            m.call(0x0023_320c, &[a0, a1, 0x40], &[]);
        }
        6 | 7 => {
            let sp = stack_push(m, 0x30);
            m.call(0x0011_b628, &[sp], &[a2]);
            let q = m.rv(a1 + 0x30);
            m.call(0x0025_6d18, &[a0 + 0x30], &[q]);
            for i in 0..3 {
                let v = m.rv(sp + 0x10 * i);
                m.wv(a0 + 0x10 * i, v);
            }
            m.wf(a0 + 0x3c, 1.0);
            m.w32(a0 + 0x2c, 0);
            m.w32(a0 + 0x1c, 0);
            m.w32(a0 + 0xc, 0);
            stack_pop(m, 0x30);
        }
        11 => xyz(m, 0x10, 0x20, 0x24),
        12 => xyz(m, 0, 0x14, 0x28),
        _ => {}
    }
    a0
}

/// FUN_00255ab0(node): unlink from the 0x1c-stride packed list (tables 0x452110, heads 0x2dd2b0)
pub fn ee_sub_255ab0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    unlink_255(m, a0, 0x0045_2110, 0x1c, 0x002d_d2b0);
}

/// FUN_001f7c68(o, t)
pub fn ee_sub_1f7c68<M: QMem + ?Sized>(m: &mut M, a0: u32, f12: f32) {
    if f12 <= 0.0 {
        m.call(0x001f_7d90, &[a0], &[f4(0.0)]);
        m.call(0x001f_7d10, &[a0], &[f4(1.0)]);
        return;
    }
    if m.r32(a0 + 0x2c) & 0x0100_0000 != 0 {
        vslot(m, a0, 7, &[], &[]);
        m.wf(a0 + 0x12c, 0.0);
    }
    m.call(0x001f_7d90, &[a0], &[f4(1.0 / f12)]);
}

/// FUN_001c6b28(o, state)
pub fn ee_sub_1c6b28<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if m.r32(a0 + 0x150) == a1 {
        return;
    }
    let mut s0 = a1;
    if a1 == 1 {
        let v = m.call(0x001f_bb90, &[0], &[]).0;
        m.call(0x001f_b2f8, &[v, 0], &[]);
        let p = m.r32(a0 + 0x230);
        if p != 0 {
            m.call(0x0016_4640, &[p, 2, a0], &[]);
        }
        s0 = 2;
        vslot(m, a0, 2, &[5, 0, 0, 0, 0], &[]);
    }
    m.call(0x001c_60b8, &[a0, s0], &[]);
}

/// FUN_0019cc50(o, mode, a, b)
pub fn ee_sub_19cc50<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32) {
    if m.r32(a0 + 4) as i32 != -1 {
        if m.r32(a0 + 0x20) == 1 || a1 != 1 {
            return;
        }
        m.call(0x0019_cd38, &[a0, 2], &[]);
        m.call(0x0019_cd38, &[a0, 0xffff_ffff], &[]);
    }
    m.w32(a0 + 0x20, a1);
    m.w32(a0 + 0x40, a2);
    m.w32(a0 + 0x44, a3);
    m.call(0x0019_cd38, &[a0, 0], &[]);
}

/// FUN_0012ede8(o)
pub fn ee_sub_12ede8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_6150, &[a0], &[]);
    let mut i = 0i32;
    while i < m.r32(a0 + 0xe4) as i32 {
        let p = m.r32(a0 + 0xec).wrapping_add((i as u32) << 2);
        if p != 0 {
            let o = m.r32(p);
            if o != 0 {
                m.call(0x001f_b098, &[o], &[f4(0.0)]);
                m.call(0x0025_8040, &[p, 0], &[]);
            }
        }
        i += 1;
    }
}

/// FUN_00259030(o, a1)
pub fn ee_sub_259030<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let sp = stack_push(m, 0x10);
    m.w32(sp + 4, 0x10);
    m.w32(sp, 0x100);
    let v = m.call(0x001a_e1d0, &[sp], &[]).0;
    m.w32(a0 + 0xe4, v);
    m.call(0x0015_9118, &[v], &[]);
    m.call(0x0023_1170, &[a0, a1], &[]);
    let x = m.r32(a0 + 0xe4);
    if m.r32(x + 0x10) == 3 {
        let y = m.r32(x + 0x30);
        m.call(0x0015_9240, &[y, a1], &[]);
    }
    m.call(0x0025_8f40, &[a0, a1], &[]);
    stack_pop(m, 0x10);
}

/// FUN_001d4178(o)
pub fn ee_sub_1d4178<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001f_50e0, &[a0], &[]);
    let mut i = 0i32;
    let mut s0 = a0 + 0x1a0;
    while i < m.r32(a0 + 0x190) as i32 {
        let q = m.rv(s0 + 0x50);
        let v = m.call(0x001a_6f00, &[a0, 0], &[q]).1;
        i += 1;
        let f = m.rf(s0 + 0x20);
        m.call(0x001f_5480, &[a0], &[v, f4(f)]);
        s0 += 0x60;
    }
}

/// FUN_002241d8(o, msg, ev)
pub fn ee_sub_2241d8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    m.call(0x0013_3590, &[a0, a1, a2], &[]);
    if a1 != 4 {
        return;
    }
    let o = m.r32(a2 + 4);
    if o != a0 {
        return;
    }
    if m.r32(o + 0x150).wrapping_sub(1) < 2 {
        vslot(m, o, 75, &[3], &[]);
    }
}

/// FUN_00117db8(stream) -> u32 (little-endian read)
pub fn ee_sub_117db8<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let a3 = m.r32(a0 + 8);
    if (a3 as i32) < 4 {
        let sp = stack_push(m, 0x10);
        m.call(0x0011_7968, &[a0, 4, sp, a3], &[]);
        let v = m.r32(sp);
        stack_pop(m, 0x10);
        return v;
    }
    let p = m.r32(a0 + 4);
    let v = (m.r8(p) as u32)
        .wrapping_add((m.r8(p + 1) as u32) << 8)
        .wrapping_add((m.r8(p + 2) as u32) << 16)
        .wrapping_add(((m.r8(p + 3) as i8 as i32) << 24) as u32);
    m.w32(a0 + 8, a3.wrapping_sub(4));
    m.w32(a0 + 4, p + 4);
    v
}

/// FUN_001b10d0(o, msg, ev)
pub fn ee_sub_1b10d0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    m.call(0x0013_3590, &[a0, a1, a2], &[]);
    if a1 == 2 && a2 == m.r32(a0 + 0x1c4) && m.r32(a2 + 0x14) == m.r32(a0 + 0x1a4) {
        m.call(0x001b_1328, &[a0, 4], &[]);
    }
}

/// FUN_0012e7f0(o, a1, ev)
pub fn ee_sub_12e7f0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    if m.r32(a0 + 0x10) == a2 {
        if m.r32(a2 + 0x120) != 0 {
            m.call(0x0014_d8c0, &[0x002d_df40], &[]);
        }
        m.w32(a0 + 0x10, 0);
    }
    m.call(0x0014_fce8, &[a0, a1, a2], &[]);
}

/// FUN_001fbfb0(o)
pub fn ee_sub_1fbfb0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    if m.r16(a0 + 0x50) & 0x400 == 0 {
        return;
    }
    vslot(m, a0, 0, &[], &[]);
    let p = m.r32(a0 + 0x40);
    m.call(0x0023_2498, &[p, 0x001f_bb30, a0], &[]);
    m.call(0x001f_b658, &[a0, 0x400, 0], &[]);
}

/// FUN_00229308(o)
pub fn ee_sub_229308<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001f_2d48, &[a0], &[]);
    let mut s0 = m.r32(a0 + 0x150);
    while s0 != 0 {
        let a = m.r32(s0 + 4);
        m.call(0x0022_9738, &[a, s0], &[]);
        s0 = m.r32(s0 + 0xc);
    }
    let l = m.r32(a0 + 0x150);
    m.call(0x0022_96c0, &[0x003d_d490, l], &[]);
    m.w32(a0 + 0x150, 0);
}

/// FUN_001fbe80(o)
pub fn ee_sub_1fbe80<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let f1 = m.rf(a0 + 0x48);
    let p = m.r32(a0 + 0x40);
    if 0.0 < f1 {
        let n = (f1 * 240.0) as i32 as u32;
        m.call(0x0023_2568, &[p, 0xffff_fffc, n, 2], &[]);
    } else {
        m.call(0x0023_2350, &[p], &[]);
    }
}

/// FUN_0012e798(o, a1, ev)
pub fn ee_sub_12e798<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    m.call(0x0014_fcc8, &[a0, a1, a2], &[]);
    m.w32(a0 + 0x10, a2);
    let f = m.rf(a2 + 0x130);
    if f != 0.0 {
        m.call(0x0014_f5a0, &[0x002d_df40], &[f4(f)]);
    }
}

/// FUN_00142880(o)
pub fn ee_sub_142880<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let (x, y) = if m.r32(a0 + 0x1148) == 0 { (0x1140, 0x1144) } else { (0x1138, 0x113c) };
    let f12 = m.rf(a0 + x);
    let f13 = m.rf(a0 + y);
    let t = m.rf(0x002d_ded0);
    let f0 = m.call(0x0011_d448, &[a0], &[f4(f12), f4(f13)]).1[0];
    m.wf(a0 + 0x114c, t + f0);
}

/// FUN_00129658(dst, src) -> dst
pub fn ee_sub_129658<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    m.call(0x0012_9df8, &[a1], &[]);
    m.call(0x0012_9ec8, &[a0], &[]);
    let v = m.r32(a1);
    m.w32(a0, v);
    let v = m.r32(a1 + 4);
    m.w32(a0 + 4, v);
    a0
}

/// FUN_001885b8(o)
pub fn ee_sub_1885b8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0020_39c0, &[a0], &[]);
    m.w32(a0 + 0x240, 0);
    m.wf(a0 + 0x248, fb(0xc296_0000));
    m.w32(a0 + 0x244, 0);
    m.call(0x001a_64e8, &[a0, 0xad, 0x002b_e990], &[]);
}

/// FUN_002296c0(pool, list): free every node of list
pub fn ee_sub_2296c0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let mut s0 = a1;
    while s0 != 0 {
        let n = m.r32(s0 + 0xc);
        m.call(0x0011_cd98, &[a0 + 0x40e8, s0], &[]);
        s0 = n;
    }
}

/// FUN_00223fe0(o)
pub fn ee_sub_223fe0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001f_31e0, &[a0], &[]);
    let v = m.r32(a0 + 0xe8);
    if v != 0 {
        m.wf(v + 0xd4, 2.0);
        let v = m.r32(a0 + 0xe8);
        m.wf(v + 0xd8, 2.0);
    }
}

/// FUN_00204c60(o, a1)
pub fn ee_sub_204c60<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.call(0x001f_5988, &[a0, a1], &[]);
    let v = m.r32(a0 + 0x208);
    if v != 0 && m.r32(v + 0x48) == a1 {
        m.w32(a0 + 0x208, 0);
    }
}

/// FUN_00187e28(o)
pub fn ee_sub_187e28<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    if m.r32(a0 + 0x10e8) == 0 || (m.r32(a0 + 0x1010) as i32) < 3 || m.r32(a0 + 0x100c) != 3 {
        return;
    }
    m.call(0x001d_5310, &[a0 + 0x10cc], &[]);
}

/// FUN_00141ea8(o, key) -> OR of masks for matching entries
pub fn ee_sub_141ea8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let mut r = 0;
    for i in 0..3 {
        if m.r32(a0 + 0x1180 + 0xc * i) == a1 {
            r |= m.r32(0x002c_3258 + 4 * i);
        }
    }
    r
}

/// FUN_00224028(o)
pub fn ee_sub_224028<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001f_2c30, &[a0], &[]);
    let p = m.r32(a0 + 0x164);
    if p != 0 {
        m.call(0x001f_1448, &[p, a0, 0xffff_ffff, 0, a0 + 0x168], &[]);
    }
}

/// FUN_0017a7b8(o)
pub fn ee_sub_17a7b8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001e_ef78, &[a0], &[]);
    m.call(0x0017_a8c8, &[a0, 0xffff_ffff], &[]);
    let p = m.r32(a0 + 0x10);
    m.call(0x0020_9970, &[p, 0x16], &[]);
}

/// FUN_0017ecb0(o)
pub fn ee_sub_17ecb0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001e_ef98, &[a0], &[]);
    let p = m.r32(a0 + 0x18);
    if m.call(0x0017_dcb8, &[p], &[]).0 != 0 {
        m.call(0x0017_ecf0, &[a0, 0], &[]);
    }
}

/// FUN_00225b60(o)
pub fn ee_sub_225b60<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001f_36e0, &[a0], &[]);
    m.call(0x0022_5ba0, &[a0], &[]);
    m.call(0x0022_5e98, &[a0], &[]);
    m.call(0x0022_6250, &[a0], &[]);
}

/// FUN_00134770(o)
pub fn ee_sub_134770<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0020_39c0, &[a0], &[]);
    m.call(0x001a_64e8, &[a0, 0x76, 0x002b_c390], &[]);
}

/// FUN_00209098(o)
pub fn ee_sub_209098<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    if m.r32(a0 + 0xfc8) != 0 {
        return 1;
    }
    m.call(0x0020_f1b8, &[0x003d_d490], &[]).0
}

fn reg_add<M: QMem + ?Sized>(m: &mut M, a0: u32, list: u32) {
    m.call(0x001a_6150, &[a0], &[]);
    m.call(0x0011_a2e8, &[list, a0], &[]);
}

pub fn ee_sub_227e58<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    reg_add(m, a0, 0x003e_1480);
}
pub fn ee_sub_1c3420<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    reg_add(m, a0, 0x003e_13c0);
}
pub fn ee_sub_183730<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    reg_add(m, a0, 0x003e_1408);
}

/// FUN_001ff438(o)
pub fn ee_sub_1ff438<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_6178, &[a0], &[]);
    m.call(0x0011_a220, &[0x003e_13d8, a0], &[]);
}

/// FUN_00228720(o, a1)
pub fn ee_sub_228720<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.call(0x0023_1170, &[a0, a1], &[]);
    m.call(0x0011_8420, &[a1], &[]);
}

/// FUN_00228140(o, a1)
pub fn ee_sub_228140<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.call(0x0023_1170, &[a0, a1], &[]);
    m.call(0x0011_8420, &[a1], &[]);
}

/// FUN_0022d750(o, a1)
pub fn ee_sub_22d750<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let v = m.r32(a0 + 0x10);
    m.call(0x0022_d6b0, &[a1, v, 0], &[]).0
}

/// FUN_001d4e08(o)
pub fn ee_sub_1d4e08<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let i = m.r32(a0 + 0x6c).wrapping_mul(0xc);
    let p = m.r32(a0 + 0x10).wrapping_add(i);
    ((m.r32(p + 0x574) as i32) >= (m.r32(a0 + 0x70) as i32)) as u32
}

/// FUN_0024a0c0(f) -> isnan(f)
pub fn ee_sub_24a0c0<M: QMem + ?Sized>(m: &mut M, f12: f32) -> u32 {
    0x7f80_0000u32.wrapping_sub(f12.to_bits() & 0x7fff_ffff) >> 31
}

/// FUN_001acc18(o, state)
pub fn ee_sub_1acc18<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if a1 != m.r32(a0 + 0x28) {
        m.w32(a0 + 0x28, a1);
        let f = m.r32(0x002d_df0c);
        m.w32(a0 + 0x2c, f);
    }
}

/// FUN_00205438(o, id)
pub fn ee_sub_205438<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    m.call(0x001e_ee00, &[a0 + 0x26c, a1], &[]).0
}

/// FUN_001a5c28(o, a1)
pub fn ee_sub_1a5c28<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.call(0x0013_f810, &[a0, a1], &[]);
}

/// FUN_00142868(o, v)
pub fn ee_sub_142868<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.w32(a0 + 0x1148, a1);
    let f = m.r32(0x002a_4d64);
    m.w32(a0 + 0x114c, f);
}

/// FUN_002562a8(p)
pub fn ee_sub_2562a8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.w32(a0 + 4, 0);
    m.w32(a0, 0);
}

pub fn ee_sub_1d9878<M: QMem + ?Sized>(m: &mut M) {}
pub fn ee_sub_182cd0<M: QMem + ?Sized>(m: &mut M) {}
pub fn ee_sub_1a6130<M: QMem + ?Sized>(m: &mut M) {}
pub fn ee_sub_14fd30<M: QMem + ?Sized>(m: &mut M) {}
pub fn ee_sub_1eefb8<M: QMem + ?Sized>(m: &mut M) {}

/// FUN_0017a7f8(o)
pub fn ee_sub_17a7f8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001e_ef98, &[a0], &[]);
    let mut s0 = m.r32(a0 + 0x30) as i32;
    loop {
        if s0 < 4 && s0 >= 2 {
            let o = m.r32(a0 + 0x10);
            if m.r32(o + 0x208) != 0 {
                m.call(0x0017_a8c8, &[a0, 0xffff_ffff], &[]);
            }
        }
        let v = m.r32(a0 + 0x30) as i32;
        if s0 == v {
            break;
        }
        s0 = v;
    }
    if s0 != 1 && s0 != 4 {
        return;
    }
    let t = m.rf(0x002d_ded0);
    let f0 = t - m.rf(a0 + 0x38);
    if m.rf(0x002b_e6c0) < f0 {
        let o = m.r32(a0 + 0x10);
        m.wf(a0 + 0x38, t);
        let q = m.call(0x001a_62e8, &[o], &[]).1;
        let g = m.r32(0x003e_14b0);
        let o = m.r32(a0 + 0x10);
        let r = m.call(0x0015_6698, &[g, o], &[q]).0;
        m.w32(r + 0x54, 3);
    }
}

/// FUN_00230100(a0, owner, node): remove node from owner+0x90 list (link +0x10)
pub fn ee_sub_230100<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    let s0 = m.r32(a1 + 0x90);
    let mut v1 = a1 + 0x90;
    let mut v0 = s0;
    if v0 == 0 {
        return;
    }
    while v0 != a2 {
        v1 = v0 + 0x10;
        v0 = m.r32(v0 + 0x10);
        if v0 == 0 {
            return;
        }
    }
    let n = m.r32(v0 + 0x10);
    m.w32(v1, n);
    if m.r32(a1 + 0x90) == s0 {
        if s0 != 0 {
            let vt = m.r32(s0 + 0x18);
            vraw(m, s0, vt, 0x1c, &[], &[]);
        }
        return;
    }
    m.call(0x0022_c840, &[a0, a1], &[]);
    let vt = m.r32(s0 + 0x18);
    vraw(m, s0, vt, 0x14, &[], &[]);
    let h = m.r32(a1 + 0x90);
    if h != 0 {
        let vt = m.r32(h + 0x18);
        vraw(m, h, vt, 0xc, &[], &[]);
    }
}

fn cross(a: V4, b: V4) -> V4 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0], 0.0]
}

/// FUN_0011e2d8(fwd, up, alt_up, out[2]): orthonormal side / up vectors
pub fn ee_sub_11e2d8<M: QMem + ?Sized>(m: &mut M, a3: u32, a0: V4, a1: V4, a2: V4) {
    let mut c = cross(a1, a0);
    if dot3(c, c) < fb(0x38d1_b717) {
        c = cross(a2, a0);
    }
    let u = cross(a0, c);
    let qc = 1.0 / dot3(c, c).sqrt();
    let qu = 1.0 / dot3(u, u).sqrt();
    m.wv(a3, [c[0] * qc, c[1] * qc, c[2] * qc, c[3] * qc]);
    m.wv(a3 + 0x10, [u[0] * qu, u[1] * qu, u[2] * qu, u[3] * qu]);
}
