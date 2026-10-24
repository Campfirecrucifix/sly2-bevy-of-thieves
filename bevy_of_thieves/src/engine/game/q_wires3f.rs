//! WIRE-S3: 1:1 QMem ports of live-frontier natives (shard 5/6/7).
//! QMem convention: integer / pointer args in `a` (stack args appended after the 8 register args),
//! quadword and float args in `v`; quadword returns in `.1`.
#![allow(unused)]
#![allow(clippy::too_many_arguments)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

fn vadd(a: V4, b: V4) -> V4 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2], a[3] + b[3]]
}

fn vsub(a: V4, b: V4) -> V4 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2], a[3] - b[3]]
}

fn vscale(a: V4, s: f32) -> V4 {
    [a[0] * s, a[1] * s, a[2] * s, a[3] * s]
}

/// vf00 = (0, 0, 0, 1)
const VF0: V4 = [0.0, 0.0, 0.0, 1.0];

fn dot3(a: V4) -> f32 {
    a[0] * a[0] + a[1] * a[1] + a[2] * a[2]
}

/// u64 -> f32 via FUN_0023c078 (the asm's halve-and-double trick for values with the top bit set)
fn u64tof<M: QMem + ?Sized>(m: &mut M, x: u64) -> f32 {
    if (x as i64) >= 0 {
        m.call(0x0023_c078, &[x as u32, (x >> 32) as u32], &[]).1[0]
    } else {
        let h = (x & 1) | (x >> 1);
        let f = m.call(0x0023_c078, &[h as u32, (h >> 32) as u32], &[]).1[0];
        f + f
    }
}

/// g++ pointer-to-member-function call through the 8-byte PMF at `pmf` on `obj` (no extra args).
fn pmf_call<M: QMem + ?Sized>(m: &mut M, pmf: u32, obj: u32) {
    let idx = m.r16(pmf + 2) as i16 as i32;
    let delta = m.r16(pmf) as i16 as i32;
    let (f, adj) = if idx >= 0 {
        let vo = m.r16(pmf + 4) as i16 as i32;
        let vt = m.r32(obj.wrapping_add(vo as u32));
        let e = vt.wrapping_add((idx as u32).wrapping_mul(8)).wrapping_sub(8);
        let d2 = m.r16(e) as i16 as i32;
        (m.r32(e + 4), d2.wrapping_add(delta))
    } else {
        (m.r32(pmf + 4), delta)
    };
    m.call(f, &[obj.wrapping_add(adj as u32)], &[]);
}

/// FUN_001fea60(o, ch, k, f12): set channel factor [k][ch] (via FUN_0011d948 clamp); k == 2 toggles
/// fire PMF on 0/1/2 targets when the value crosses 0; recompute the product over 8 factors and
/// notify FUN_002321e8(ch) when it changes at 1/1024 resolution.
pub fn ee_sub_1fea60<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, f12: f32) {
    if a2 >= 8 || (a1 as i32) < 0 || (a1 as i32) >= 4 {
        return;
    }
    let mut f20 = m.call(0x0011_d948, &[0x002b_bb60], &[f4(f12)]).1[0];
    let s6 = a1 << 2;
    let t = a2 << 4;
    let s7 = a0 + 0x13c;
    let s5 = a0 + 0x1c0;
    if m.rf(s7 + s6 + t) == f20 {
        return;
    }
    if a2 == 2 {
        let sp = stack_push(m, 0xd0);
        m.call(0x0023_33b4, &[sp, 0, 8], &[]);
        let mut copy = 0u32;
        if f20 == 0.0 {
            copy = 0x002a_8298;
        } else if m.rf(s6 + s7 + 0x20) == 0.0 {
            copy = 0x002a_82a0;
        }
        if copy != 0 {
            let w0 = m.r32(copy);
            let w1 = m.r32(copy + 4);
            m.w32(sp, w0);
            m.w32(sp + 4, w1);
        }
        let idx = m.r16(sp + 2) as i16;
        if idx != 0 {
            if a1 == 1 {
                let mut i = 0u32;
                while (i as i32) < (m.r32(a0 + 0x204) as i32) {
                    let obj = m.r32(a0 + 0x208 + i.wrapping_mul(4));
                    if obj != 0 {
                        pmf_call(m, sp, obj);
                    }
                    i += 1;
                }
            } else if a1 == 0 {
                let obj = m.r32(a0 + 0x228);
                if obj != 0 {
                    pmf_call(m, sp, obj);
                }
            } else if a1 == 2 {
                let obj = m.r32(a0 + 0x200);
                if obj != 0 {
                    pmf_call(m, sp, obj);
                }
            }
        }
        stack_pop(m, 0xd0);
    }
    m.wf(s7 + s6 + t, f20);
    f20 = 1.0;
    for k in 0..8u32 {
        f20 *= m.rf(s6 + s7 + k.wrapping_mul(16));
    }
    let v1 = s5 + s6;
    let a = (f20 * 1024.0) as i32;
    let b = (m.rf(v1) * 1024.0) as i32;
    if a == b {
        return;
    }
    m.wf(v1, f20);
    m.call(0x0023_21e8, &[a1], &[]);
}

/// FUN_0019c300(o, src, a2 (quad local point), out_d, out_hit, out_pos, f12 dt): predict o's position
/// over dt (A from FUN_001a62e8, B from vslot 32) and solve the radius (+0x1020) contact against src's
/// frame (vslot 31 -> FUN_002309e8 -> FUN_0011c168); writes distance / hit point / end position.
pub fn ee_sub_19c300<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: V4, a3: u32, t0: u32, t1: u32, f12: f32) {
    let sp = stack_push(m, 0x1c0);
    let f20 = f12;
    m.wv(sp + 0x130, a2);
    let o = m.r32(a0);
    let a = m.call(0x001a_62e8, &[o], &[]).1;
    m.wv(sp + 0xf0, a);
    let o = m.r32(a0);
    let b = vslot(m, o, 32, &[o], &[]).1;
    m.wv(sp + 0x100, b);
    let s1 = sp + 0x50;
    let o = m.r32(a0);
    let v1 = m.r32(o + 0xe8);
    let c = m.rv(v1 + 0x80);
    let h = f20 * 0.5;
    let e = vadd(vadd(b, VF0), vscale(c, f20));
    m.wv(sp + 0xe0, e);
    let g = vadd(vadd(vadd(a, VF0), vscale(b, h)), vscale(e, h));
    m.wv(sp + 0x120, g);
    let p = vslot(m, a1, 31, &[s1], &[]).0;
    m.wv(sp + 0x110, g);
    m.call(0x0023_09e8, &[p, 0, sp + 0x10, 0], &[f4(f20)]);
    m.call(0x0011_c168, &[sp + 0xa0, sp + 0x10, s1], &[]);
    let r0 = m.rv(sp + 0xa0);
    let r1 = m.rv(sp + 0xb0);
    let r2 = m.rv(sp + 0xc0);
    let r3 = m.rv(sp + 0xd0);
    let q = m.rv(sp + 0x130);
    let mut v5 = [0.0f32; 4];
    for i in 0..3 {
        v5[i] = r0[i] * q[0] + r1[i] * q[1] + r2[i] * q[2] + r3[i];
    }
    m.wv(sp + 0x10, r0);
    let gg = m.rv(sp + 0x110);
    m.wv(sp + 0x20, r1);
    let v5 = vsub(v5, gg);
    m.wv(sp + 0x30, r2);
    let d2 = dot3(v5);
    m.wv(sp + 0x40, r3);
    m.wv(sp, v5);
    let o = m.r32(a0);
    let f1 = m.rf(o + 0x1020);
    if d2 < f1 * f1 {
        if a3 != 0 {
            m.wf(a3, d2.sqrt() - f1);
        }
        if t0 != 0 {
            let v = m.rv(sp + 0x120);
            m.wv(t0, v);
        }
    } else {
        let z = m.rf(sp + 8);
        let f2 = z.abs();
        if f1 < f2 || f20 == 0.0 {
            if a3 != 0 {
                m.wf(a3, f2 + f32::from_bits(0x4976_9500));
            }
            if t0 != 0 {
                let v = m.rv(sp + 0xf0);
                m.wv(t0, v);
            }
        } else {
            m.wf(sp + 8, 0.0);
            let v3 = m.rv(sp);
            let len = dot3(v3).sqrt();
            if len < f32::from_bits(0x38d1_b717) {
                let v = m.rv(0x002a_2dd0);
                m.wv(sp, v);
            } else {
                m.wv(sp, vscale(v3, 1.0 / len));
            }
            let o = m.r32(a0);
            let f2 = f2 * f2;
            let r = m.rf(o + 0x1020);
            let f1 = (r * r - f2).sqrt();
            let f2 = len - f1;
            let n = m.rv(sp);
            let f0 = (f2 + f2) / (f20 * f20);
            let v1 = m.r32(o + 0xe8);
            let k = vscale(n, f0);
            let c2 = m.rv(v1 + 0x80);
            let b = m.rv(sp + 0x100);
            let e2 = vadd(vadd(vadd(b, VF0), vscale(c2, f20)), vscale(k, f20));
            m.wv(sp + 0xe0, e2);
            if a3 != 0 {
                let x = m.rf(0x002b_f910);
                let v = (x + x) * f2 / f20;
                m.wf(a3, v.min(f32::from_bits(0x4974_2400)));
            }
            if t0 != 0 {
                let a = m.rv(sp + 0xf0);
                let h = f20 * 0.5;
                let r = vadd(vadd(vadd(a, VF0), vscale(b, h)), vscale(e2, h));
                m.wv(t0, r);
            }
        }
    }
    if t1 != 0 {
        let v = m.rv(sp + 0xe0);
        m.wv(t1, v);
    }
    stack_pop(m, 0x1c0);
}

/// FUN_00244b68: atanf(f12) (fdlibm, tables at DAT_002ab860 hi / 002ab870 lo / 002ab880 aT).
pub fn ee_sub_244b68<M: QMem + ?Sized>(m: &mut M, f12: f32) -> f32 {
    let s1 = f12.to_bits() as i32;
    let s0 = s1 & 0x7fff_ffff;
    let mut x = f12;
    let id: i32;
    if s0 > 0x507f_ffff {
        if s0 > 0x7f80_0000 {
            return x + x;
        }
        return if s1 > 0 {
            m.rf(0x002a_b86c) + m.rf(0x002a_b87c)
        } else {
            -m.rf(0x002a_b86c) - m.rf(0x002a_b87c)
        };
    }
    if s0 <= 0x3edf_ffff {
        if s0 <= 0x30ff_ffff && 1.0 < x + f32::from_bits(0x7149_f2ca) {
            return x;
        }
        id = -1;
    } else {
        x = m.call(0x0024_4ef8, &[], &[f4(x)]).1[0];
        if s0 <= 0x3f97_ffff {
            if s0 <= 0x3f2f_ffff {
                id = 0;
                x = (x + x - 1.0) / (x + 2.0);
            } else {
                id = 1;
                x = (x - 1.0) / (x + 1.0);
            }
        } else if s0 <= 0x401b_ffff {
            id = 2;
            x = (x - 1.5) / (x * 1.5 + 1.0);
        } else {
            id = 3;
            x = -1.0 / x;
        }
    }
    let at = |m: &mut M, i: u32| m.rf(0x002a_b880 + i.wrapping_mul(4));
    let z = x * x;
    let w = z * z;
    let f5 = at(m, 8) + w * at(m, 10);
    let f1 = at(m, 7) + w * at(m, 9);
    let f6 = at(m, 6) + w * f5;
    let f2 = at(m, 5) + w * f1;
    let f7 = at(m, 4) + w * f6;
    let f3 = at(m, 3) + w * f2;
    let f8 = at(m, 2) + w * f7;
    let f4_ = at(m, 1) + w * f3;
    let s2 = w * f4_;
    let f9 = at(m, 0) + w * f8;
    let s1f = z * f9;
    if id < 0 {
        return x - x * (s1f + s2);
    }
    let lo = m.rf(0x002a_b870 + (id as u32) * 4);
    let hi = m.rf(0x002a_b860 + (id as u32) * 4);
    let r = hi - ((x * (s1f + s2) - lo) - x);
    if s1 < 0 {
        -r
    } else {
        r
    }
}

/// FUN_001fdec8(o): sequencer tick on +0x200 track: skip when paused/held; advance +0x24c by
/// DAT_002ddf10; either finish (FUN_001fe738) or fire due events (FUN_001fe0e8) at 1/60-s steps.
pub fn ee_sub_1fdec8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let p = m.r32(a0 + 0x200);
    if p == 0 {
        return;
    }
    if m.r16(p + 0x52) != 0 || (m.r16(p + 0x50) & 1) != 0 {
        return;
    }
    let p = m.r32(a0 + 0x200);
    if (m.r16(p + 0x4e) as i16) > 0 {
        return;
    }
    let s2 = m.r32(p + 0x60);
    let v = m.rf(a0 + 0x24c) + m.rf(0x002d_df10);
    m.wf(a0 + 0x24c, v);
    let flag = m.r32(p + 0x40) == 0 && m.r16(p + 0x52) == 0 && (m.r16(p + 0x50) & 1) == 0;
    let f21 = f32::from_bits(0x3169_0453);
    if flag {
        let q = m.r32(a0 + 0x200);
        let x = m.r64(q + 0x68);
        let f0 = u64tof(m, x) * f21;
        if 0.0 < f0 || m.rf(s2 + 0x50) <= m.rf(a0 + 0x24c) {
            m.call(0x001f_e738, &[a0], &[]);
        }
        return;
    }
    let f22 = f32::from_bits(0x3c88_8889);
    let mut v1 = m.r32(a0 + 0x244);
    while (v1 as i32) < (m.r32(s2 + 0x54) as i32) {
        let s1 = m.r32(s2 + 0x58).wrapping_add(v1 << 3);
        let q = m.r32(a0 + 0x200);
        let f20 = (m.r16(s1) as i16 as i32 as f32) * f22;
        let x = m.r64(q + 0x68);
        let f1 = u64tof(m, x) * f21;
        let f0 = m.rf(a0 + 0x248) + f20;
        if f1 < f0 {
            return;
        }
        m.call(0x001f_e0e8, &[a0, s1], &[]);
        let f0 = m.rf(a0 + 0x248) + f20;
        v1 = m.r32(a0 + 0x244).wrapping_add(1);
        m.w32(a0 + 0x244, v1);
        m.wf(a0 + 0x248, f0);
    }
}

/// FUN_001507f8(o) -> quad: target point. +0x14 set: dir = norm(FUN_001a62e8(+0x14) - FUN_00150780(o)),
/// +0xa4 = FUN_002458f0(dir.y, dir.x), result FUN_00150780(o) - dir * |DAT_002a5000|; else
/// FUN_00150780(o) + FUN_0011b720(+0xa4, +0xa8) * DAT_002a5000.
pub fn ee_sub_1507f8<M: QMem + ?Sized>(m: &mut M, a0: u32) -> V4 {
    let sp = stack_push(m, 0x90);
    let p = m.r32(a0 + 0x14);
    let r = if p != 0 {
        let a = m.call(0x001a_62e8, &[p], &[]).1;
        m.wv(sp + 0x60, a);
        let b = m.call(0x0015_0780, &[a0], &[]).1;
        let d = vsub(m.rv(sp + 0x60), b);
        let len = dot3(d).sqrt();
        let n = vscale(d, 1.0 / len);
        m.wv(sp, n);
        let f12 = m.rf(sp + 4);
        let f13 = m.rf(sp);
        let f0 = m.call(0x0024_58f0, &[], &[f4(f12), f4(f13)]).1[0];
        m.wf(a0 + 0xa4, f0);
        let c = m.call(0x0015_0780, &[a0], &[]).1;
        let n = m.rv(sp);
        let w = m.rv(0x002a_5000);
        let l = dot3(w).sqrt();
        m.wv(sp + 0x10, f4(-l));
        vadd(c, vscale(n, -l))
    } else {
        let f12 = m.rf(a0 + 0xa4);
        let f13 = m.rf(a0 + 0xa8);
        let w = m.rv(0x002a_5000);
        m.wv(sp + 0x50, w);
        m.call(0x0011_b720, &[sp + 0x20], &[f4(f12), f4(f13)]);
        let c = m.call(0x0015_0780, &[a0], &[]).1;
        let r1 = m.rv(sp + 0x30);
        let r0 = m.rv(sp + 0x20);
        let mut v = m.rv(sp + 0x40);
        let w = m.rv(sp + 0x50);
        for i in 0..3 {
            v[i] = r0[i] * w[0] + r1[i] * w[1] + v[i] * w[2];
        }
        vadd(c, v)
    };
    stack_pop(m, 0x90);
    r
}

/// FUN_001c7d08(): count set bits over the save-data flag words of the current level's entries
/// (FUN_00210a88 level id, FUN_001d7da0 / FUN_001d8720 / FUN_001d7e80 lookups in DAT_003e0ea8).
pub fn ee_sub_1c7d08<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let a0 = m.call(0x0021_0a88, &[], &[]).0;
    if a0 == u32::MAX {
        return 0;
    }
    let sp = stack_push(m, 0xa0);
    let mut s2 = 0u32;
    let p = m.call(0x0021_09b8, &[a0], &[]).0;
    let a1 = m.r32(p + 4);
    m.call(0x001a_e5a0, &[sp, a1, 6], &[]);
    let s0 = 0x003e_0ea8u32;
    let s5 = sp + 0x10;
    m.w32(sp + 0x14, sp);
    m.w32(sp + 0x10, 3);
    m.call(0x001d_7da0, &[s0, s5, sp + 0x20], &[]);
    m.w32(sp + 0x10, 5);
    let v = m.call(0x001d_8720, &[s0 + 8, 0x002a_7788], &[]).0;
    m.w16(sp + 8, v as u16);
    let v1 = m.r32(sp + 0x20);
    let mut n = m.r16(v1 + 6) as u32;
    while n != 0 {
        let base = m.r32(0x003e_0eac);
        let e = base.wrapping_add(n.wrapping_mul(10));
        let h = m.r16(e);
        m.w16(sp + 6, h);
        let v = m.call(0x001d_7e80, &[0x003e_0ea8, s5, 0x003d_4a60], &[]).0;
        let mut x = m.r32(v);
        while x != 0 {
            x &= x.wrapping_sub(1);
            s2 += 1;
        }
        n = m.r16(e + 8) as u32;
    }
    stack_pop(m, 0xa0);
    s2
}

/// FUN_001d8b20(out, a1, fmt, ...): varargs formatter: spill a3..t3 / f12,f14,f16,f18, then
/// FUN_001ae478 / FUN_001d87c0 / FUN_001d80c8 / FUN_00129658 / FUN_001291a8 into out; returns out.
pub fn ee_sub_1d8b20<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, t0: u32, t1: u32, t2: u32, t3: u32, f12: f32, f14: f32, f16: f32, f18: f32) -> u32 {
    let sp = stack_push(m, 0x110);
    for (i, r) in [a3, t0, t1, t2, t3].into_iter().enumerate() {
        let a = sp + 0xe8 + i as u32 * 8;
        m.w32(a, r);
        m.w32(a + 4, ((r as i32) >> 31) as u32);
    }
    m.wf(sp + 0xd8, f12);
    m.wf(sp + 0xdc, f14);
    m.wf(sp + 0xe0, f16);
    m.wf(sp + 0xe4, f18);
    m.w32(sp, u32::MAX);
    let s3 = sp + 0x10;
    m.call(0x001a_e478, &[s3], &[]);
    m.call(0x001d_87c0, &[a2, sp + 0xe8, sp + 0x20], &[]);
    m.call(0x001d_80c8, &[sp + 0x30, 0x003e_0ea8, sp + 0x20, a1, 0x003d_4a60], &[]);
    m.call(0x0012_9658, &[sp, sp + 0x30], &[]);
    m.call(0x0012_9ec8, &[sp + 0x30], &[]);
    m.call(0x0012_91a8, &[a0, sp], &[]);
    m.call(0x001a_e4a0, &[s3, 2], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x110);
    a0
}

/// FUN_002375e0(a0..t3, s0, s1 (stack args)): retry FUN_00237288 up to 101 times while it fails
/// (negative) and s1 == 0, sleeping FUN_00235518(ms) with exponential backoff; returns failure bit.
pub fn ee_sub_2375e0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, t0: u32, t1: u32, t2: u32, t3: u32, s0: u32, s1: u32) -> u32 {
    let mut n = 0i32;
    let mut d = 1i32;
    loop {
        let v = m.call(0x0023_7288, &[a0, a1, a2, a3, t0, t1, t2, t3, s0], &[]).0;
        let fail = v >> 31;
        if fail == 0 || s1 != 0 {
            return fail;
        }
        m.call(0x0023_5518, &[d.wrapping_mul(1000) as u32], &[]);
        if d < 0x7f {
            d <<= 1;
        }
        n += 1;
        if n >= 0x65 {
            return fail;
        }
    }
}

/// FUN_001532d8(o, a1, a2, a3, f12): push an entry (ring of 3, 0x24 bytes at +0x1c4, count +0x1c0;
/// full -> memmove down) {a1, a2, a3, f12, FUN_001fe3f8(DAT_003dd040), 0, 0, 0, 0}.
pub fn ee_sub_1532d8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, f12: f32) {
    let n = m.r32(a0 + 0x1c0);
    let s0 = if n == 3 {
        m.call(0x0023_32b8, &[a0 + 0x1c4, a0 + 0x1e8, 0x48], &[]);
        let n = m.r32(a0 + 0x1c0);
        a0.wrapping_add(n.wrapping_mul(0x24).wrapping_add(0x1a0))
    } else {
        m.w32(a0 + 0x1c0, n.wrapping_add(1));
        a0.wrapping_add(n.wrapping_mul(0x24).wrapping_add(0x1c4))
    };
    m.w32(s0, a1);
    m.w32(s0 + 4, a2);
    m.w32(s0 + 8, a3);
    m.wf(s0 + 0xc, f12);
    let f0 = m.call(0x001f_e3f8, &[0x003d_d040], &[]).1[0];
    m.wf(s0 + 0x10, f0);
    m.w32(s0 + 0x20, 0);
    m.w32(s0 + 0x14, 0);
    m.w32(s0 + 0x18, 0);
    m.w32(s0 + 0x1c, 0);
}

/// FUN_00175ff0(o): +0x3c = FUN_001758a0(+0x10, +0x30, +0x10[+0x3cc]); found -> 1; else push
/// +0x10[+0xf04] to t + DAT_002be610 when already past, -> 0.
pub fn ee_sub_175ff0<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let p = m.r32(a0 + 0x10);
    let a1 = m.r32(a0 + 0x30);
    let a2 = m.r32(p + 0x3cc);
    let v = m.call(0x0017_58a0, &[p, a1, a2], &[]).0;
    m.w32(a0 + 0x3c, v);
    if v != 0 {
        return 1;
    }
    let p = m.r32(a0 + 0x10);
    let t = m.rf(0x002d_ded0);
    if m.rf(p + 0xf04) < t {
        let v = t + m.rf(0x002b_e610);
        m.wf(p + 0xf04, v);
    }
    0
}

/// FUN_001fdba8(o): once (+0xa0 = 1) post FUN_002330d0('RKUS', 0xc, 0, 0, 0, 0, 0, FUN_001fcf88, o).
pub fn ee_sub_1fdba8<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let v = m.r32(a0 + 0xa0);
    if v != 0 {
        return v;
    }
    m.w32(a0 + 0xa0, 1);
    m.call(0x0023_30d0, &[0x524b_5553, 0xc, 0, 0, 0, 0, 0, 0x001f_cf88, a0], &[]).0
}

/// FUN_00171da8(a0, a1): e = FUN_00205458(a0, 0x9c); e && e[+0x30] == 5 && e[+0x40] == a1.
pub fn ee_sub_171da8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let e = m.call(0x0020_5458, &[a0, 0x9c], &[]).0;
    if e == 0 || m.r32(e + 0x30) != 5 {
        return 0;
    }
    (m.r32(e + 0x40) == a1) as u32
}

/// FUN_00237b50: FUN_00237af8(); FUN_00234a60(DAT_002c4a84); -> 0.
pub fn ee_sub_237b50<M: QMem + ?Sized>(m: &mut M) -> u32 {
    m.call(0x0023_7af8, &[], &[]);
    let a0 = m.r32(0x002c_4a84);
    m.call(0x0023_4a60, &[a0], &[]);
    0
}

/// FUN_00147ec8(o): FUN_0011a2e8(o + 0x20).
/// BOSS R24: Ghidra drops the pass-through a1 (the node; FUN_00147f40 calls FUN_00147ec8(o, n)).
pub fn ee_sub_147ec8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    m.call(0x0011_a2e8, &[a0 + 0x20, a1], &[]).0
}

/// FUN_00113b20: -> 0x2bac3c.
pub fn ee_sub_113b20<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_ac3c
}

/// FUN_00113d50: -> 0x2baef8.
pub fn ee_sub_113d50<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_aef8
}

/// FUN_001140a0: -> 0x2bb31c.
pub fn ee_sub_1140a0<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_b31c
}

/// FUN_00114170: -> 0x2bb420.
pub fn ee_sub_114170<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_b420
}

/// FUN_00114530: -> 0x2bb8d0.
pub fn ee_sub_114530<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_b8d0
}
