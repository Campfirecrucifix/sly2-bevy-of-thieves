//! WIRE-W1H15: 1:1 queue ports (against `QMem`) of a batch of live-frontier natives, ported from the EE asm;
//! callees go through `m.call` / `vslot`. QMem convention: integer / pointer args in `a`, quadword and float
//! args in `v`.
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, clippy::all)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

/// FUN_001e6610(o): return *(float*)(o+0xec).
pub fn ee_sub_1e6610<M: QMem + ?Sized>(m: &mut M, a0: u32) -> f32 {
    let p = m.r32(a0 + 0xec);
    m.rf(p)
}

/// FUN_00136cb0: return &DAT_002a4ae8.
pub fn ee_sub_136cb0<M: QMem + ?Sized>(m: &mut M) -> u32 {
    0x002a_4ae8
}

/// FUN_001b80f0: empty.
pub fn ee_sub_1b80f0<M: QMem + ?Sized>(m: &mut M) {}

/// FUN_00204248: return 1.
pub fn ee_sub_204248<M: QMem + ?Sized>(m: &mut M) -> u32 {
    1
}

/// FUN_00212a30(o): +0x58 == 2 -> +0x5c = DAT_002ddf0c.
pub fn ee_sub_212a30<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    if m.r32(a0 + 0x58) == 2 {
        let f = m.rf(0x002d_df0c);
        m.wf(a0 + 0x5c, f);
    }
}

/// FUN_001f07c8(o): FUN_001f04f8(o+0x24 word, o+0x3c word).
pub fn ee_sub_1f07c8<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let a = m.r32(a0 + 0x24);
    let b = m.r32(a0 + 0x3c);
    m.call(0x001f_04f8, &[a, b], &[]).0
}

/// FUN_0020b840(o, a, b): o[0] = a; +0x94 = 0.6; +0x90 = b; +0x98 = 1.0.
pub fn ee_sub_20b840<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    m.w32(a0, a1);
    m.wf(a0 + 0x94, f32::from_bits(0x3f19_999a));
    m.w32(a0 + 0x90, a2);
    m.wf(a0 + 0x98, 1.0);
}

/// FUN_0022d670(x): push x onto the list at 0x4437a8 (count at 0x4437a0).
pub fn ee_sub_22d670<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let n = m.r32(0x0044_37a0);
    m.w32((n << 2).wrapping_add(0x0044_37a8), a0);
    m.w32(0x0044_37a0, n.wrapping_add(1));
}

fn send_232838<M: QMem + ?Sized>(m: &mut M, id: u32, words: &[u32]) -> u32 {
    let sp = stack_push(m, 0x20);
    for (i, w) in words.iter().enumerate() {
        m.w32(sp + 4 * i as u32, *w);
    }
    let r = m.call(0x0023_2838, &[id, 4 * words.len() as u32, sp, 0, 0], &[]).0;
    stack_pop(m, 0x20);
    r
}

/// FUN_002323d8(x): FUN_00232838(0x14, 4, &x, 0, 0).
pub fn ee_sub_2323d8<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    send_232838(m, 0x14, &[a0])
}

/// FUN_00232350(x): FUN_00232838(0x15, 4, &x, 0, 0).
pub fn ee_sub_232350<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    send_232838(m, 0x15, &[a0])
}

/// FUN_00232568(a,b,c,d): FUN_00232838(0x22, 0x10, {a,b,c,d}, 0, 0).
pub fn ee_sub_232568<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32) -> u32 {
    send_232838(m, 0x22, &[a0, a1, a2, a3])
}

/// FUN_001894f8(o): +0x18c8 in {0,1,5,10,12,24} -> 1 else 0 (jump table 0x2a5880).
pub fn ee_sub_1894f8<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    match m.r32(a0 + 0x18c8) {
        0 | 1 | 5 | 10 | 12 | 24 => 1,
        _ => 0,
    }
}

/// FUN_002022a0(o): for i < +0x30: (o+0x34)[i]->+0x108 = 0.
pub fn ee_sub_2022a0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    if (m.r32(a0 + 0x30) as i32) <= 0 {
        return;
    }
    let mut i: i32 = 0;
    let mut p = a0 + 0x34;
    loop {
        let e = m.r32(p);
        i += 1;
        m.w32(e.wrapping_add(0x108), 0);
        p = p.wrapping_add(4);
        if !(i < m.r32(a0 + 0x30) as i32) {
            break;
        }
    }
}

/// FUN_001a23c0(o): +4 != -1 -> (*(+0x10) > 0 ? 1 : 2); else +0x90 == 1.
pub fn ee_sub_1a23c0<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    if m.r32(a0 + 4) as i32 != -1 {
        let p = m.r32(a0 + 0x10);
        if (m.r32(p) as i32) > 0 {
            1
        } else {
            2
        }
    } else {
        (m.r32(a0 + 0x90) == 1) as u32
    }
}

/// FUN_001adea8(o): p = +0x24; !p -> 0; p+0x58 < 2 -> 1; else p+0x64 < 2 ? 0 : 1.
pub fn ee_sub_1adea8<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let p = m.r32(a0 + 0x24);
    if p == 0 {
        return 0;
    }
    if (m.r32(p + 0x58) as i32) >= 2 {
        return 1;
    }
    if (m.r32(p + 0x64) as i32) < 2 {
        0
    } else {
        1
    }
}

/// FUN_001acc80(o): by +0x40 - 0x11 (jump table 0x2a6d10): -> 1 / 2 / 0.
pub fn ee_sub_1acc80<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let i = m.r32(a0 + 0x40).wrapping_sub(0x11);
    match i {
        0 | 3 | 6 | 7 | 10 | 12 | 18 | 19 | 20 | 29 => 1,
        1 | 4 | 11 | 15 => 2,
        _ => 0,
    }
}

/// FUN_0011b7e0(dst): FUN_0011b830(&tmp, q=DAT_002a2dc0); FUN_0023320c(dst, &tmp, 0x30); return dst.
pub fn ee_sub_11b7e0<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let sp = stack_push(m, 0x30);
    let q = m.rv(0x002a_2dc0);
    m.call(0x0011_b830, &[sp], &[q]);
    m.call(0x0023_320c, &[a0, sp, 0x30], &[]);
    stack_pop(m, 0x30);
    a0
}

/// FUN_0019f120(o): +0x4748 && (DAT_002ddf30?-0x2130 - +0x474c) > 6.0 -> FUN_0019f170(o, 0).
pub fn ee_sub_19f120<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    if m.r32(a0 + 0x4748) == 0 {
        return;
    }
    let t = m.rf(0x002d_ded0) - m.rf(a0 + 0x474c);
    if 6.0 < t {
        m.call(0x0019_f170, &[a0, 0], &[]);
    }
}

/// FUN_0011bc88(v, yaw, pitch): *yaw = FUN_002458f0(v.y, v.x); *pitch = FUN_002458f0(v.z, sqrt(x²+y²)).
pub fn ee_sub_11bc88<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    let y = m.rf(a0 + 4);
    let x = m.rf(a0);
    let r = m.call(0x0024_58f0, &[a0, a1, a2], &[f4(y), f4(x)]).1[0];
    m.wf(a1, r);
    let x = m.rf(a0);
    let y = m.rf(a0 + 4);
    let h = (x * x + y * y).sqrt();
    let z = m.rf(a0 + 8);
    let r = m.call(0x0024_58f0, &[a0, a1, a2], &[f4(z), f4(h)]).1[0];
    m.wf(a2, r);
}

/// FUN_001f2a60(o, p): p+0x10 bit 50 set -> clear it; o+0x28 bit 60 -> FUN_001f2870(o, p).
pub fn ee_sub_1f2a60<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let hi = m.r32(a1 + 0x14);
    if hi & (1 << 18) == 0 {
        return;
    }
    m.w32(a1 + 0x14, hi & !(1 << 18));
    if (m.r32(a0 + 0x2c) >> 28) & 1 != 0 {
        m.call(0x001f_2870, &[a0, a1], &[]);
    }
}

/// FUN_001b90b8(o): +0x1840 in {1,3} -> +0x190 = -FUN_00204e38(o, (+0x1030)->+0x1c); +0x180 = 1.0.
pub fn ee_sub_1b90b8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let s = m.r32(a0 + 0x1840);
    if s != 1 && s != 3 {
        return;
    }
    let p = m.r32(a0 + 0x1030);
    let f = m.rf(p + 0x1c);
    let v = m.call(0x0020_4e38, &[a0, p], &[f4(f)]).1;
    m.wv(a0 + 0x190, [v[0] * -1.0, v[1] * -1.0, v[2] * -1.0, v[3] * -1.0]);
    m.wf(a0 + 0x180, 1.0);
}

/// FUN_001c1f28(o) -> qword: o[0] -> FUN_001e6f10(o[0], &out, 0, FUN_001e6648(o[0]) * 0.5); else o[1] ->
/// FUN_001a62e8(o[1]); else o+0x10.
pub fn ee_sub_1c1f28<M: QMem + ?Sized>(m: &mut M, a0: u32) -> V4 {
    let p = m.r32(a0);
    if p != 0 {
        let f = m.call(0x001e_6648, &[p], &[]).1[0];
        let sp = stack_push(m, 0x10);
        let p = m.r32(a0);
        m.call(0x001e_6f10, &[p, sp, 0], &[f4(f * 0.5)]);
        let r = m.rv(sp);
        stack_pop(m, 0x10);
        return r;
    }
    let p = m.r32(a0 + 4);
    if p != 0 {
        m.call(0x001a_62e8, &[p], &[]).1
    } else {
        m.rv(a0 + 0x10)
    }
}

/// FUN_00258658(o, n): n == 1 -> +0xe0 = 1; (*(+0xe4) > 1) -> --*(+0xe4); +0xe4 = FUN_001ae238({0x100,0x10},
/// +0xe4); *(+0xe4) = 1. return +0xe4.
pub fn ee_sub_258658<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    if a1 == 1 {
        m.w32(a0 + 0xe0, a1);
        let p = m.r32(a0 + 0xe4);
        let c = m.r32(p);
        if (c as i32) >= 2 {
            m.w32(p, c.wrapping_sub(1));
            let sp = stack_push(m, 0x10);
            m.w32(sp, 0x100);
            m.w32(sp + 4, 0x10);
            let e = m.r32(a0 + 0xe4);
            let v = m.call(0x001a_e238, &[sp, e], &[]).0;
            stack_pop(m, 0x10);
            m.w32(a0 + 0xe4, v);
            m.w32(v, a1);
        }
    }
    m.r32(a0 + 0xe4)
}

/// FUN_001b8450(jt, x): r = FUN_00144388(jt, x); +0x1430 && FUN_001a6af8(x, +0x1430) -> FUN_00134310(r, 3);
/// else +0x13d0 == 15 -> FUN_00220498(jt+0x1910, x, r).
pub fn ee_sub_1b8450<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let r = m.call(0x0014_4388, &[a0, a1], &[]).0;
    let h = m.r32(a0 + 0x1430);
    if h != 0 && m.call(0x001a_6af8, &[a1, h], &[]).0 != 0 {
        return m.call(0x0013_4310, &[r, 3], &[]).0;
    }
    if m.r32(a0 + 0x13d0) != 0xf {
        return r;
    }
    m.call(0x0022_0498, &[a0 + 0x1910, a1, r], &[]).0
}

/// FUN_0023e108(&f, out): unpack float -> out {class, sign, exp, mant}.
pub fn ee_sub_23e108<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let w = m.r32(a0);
    let mant = w & 0x7f_ffff;
    let e = (w >> 23) & 0xff;
    m.w32(a1 + 4, w >> 31);
    if e == 0 {
        m.w32(a1, 2);
        return;
    }
    if e == 0xff {
        if mant == 0 {
            m.w32(a1, 4);
            return;
        }
        m.w32(a1, if mant & 0x10_0000 != 0 { 1 } else { 0 });
        m.w32(a1 + 0xc, mant);
        return;
    }
    m.w32(a1 + 0xc, (mant << 7) | 0x4000_0000);
    m.w32(a1 + 8, e.wrapping_sub(0x7f));
    m.w32(a1, 3);
}

/// FUN_00220d40(o): +0x88 == -FLT_MAX && +0x8c == FLT_MAX -> both = FUN_0011d948(0x2c41b0,
/// FUN_002458f0(+0x30, sqrt((+4 + +0x10)² + +0x20²))).
pub fn ee_sub_220d40<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    if m.rf(a0 + 0x88) != f32::from_bits(0xff7f_ffff) {
        return;
    }
    if m.rf(a0 + 0x8c) != f32::from_bits(0x7f7f_ffff) {
        return;
    }
    let a = m.rf(a0 + 4) + m.rf(a0 + 0x10);
    let b = m.rf(a0 + 0x20);
    let h = (a * a + b * b).sqrt();
    let y = m.rf(a0 + 0x30);
    let r = m.call(0x0024_58f0, &[a0], &[f4(y), f4(h)]).1[0];
    let r = m.call(0x0011_d948, &[0x002c_41b0], &[f4(r)]).1[0];
    m.wf(a0 + 0x88, r);
    m.wf(a0 + 0x8c, r);
}

/// FUN_00118380(a, out, alloc, d): n = FUN_00117c20(a, out, alloc, d); !n -> *out = 0; else buf =
/// alloc->vslot0(n+1, d); FUN_00117968(a, n, buf); buf[n] = 0; *out = buf.
pub fn ee_sub_118380<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32) {
    let n = m.call(0x0011_7c20, &[a0, a1, a2, a3], &[]).0;
    if n == 0 {
        m.w32(a1, 0);
        return;
    }
    let b = vslot(m, a2, 0, &[n.wrapping_add(1), a3], &[]).0;
    m.call(0x0011_7968, &[a0, n, b], &[]);
    m.w8(b.wrapping_add(n), 0);
    m.w32(a1, b);
}

/// FUN_00208728(o, x): remove every x from the list +0x398 (count +0x394) via FUN_002332b8 (memmove).
pub fn ee_sub_208728<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let mut n = m.r32(a0 + 0x394);
    if (n as i32) <= 0 {
        return;
    }
    let mut i: u32 = 0;
    let mut s3 = a0 + 0x39c;
    let mut s0 = a0 + 0x398;
    loop {
        if m.r32(s0) == a1 {
            let len = (n.wrapping_sub(i) << 2).wrapping_sub(4);
            m.call(0x0023_32b8, &[s0, s3, len], &[]);
            let c = m.r32(a0 + 0x394).wrapping_sub(1);
            m.w32(a0 + 0x394, c);
        }
        n = m.r32(a0 + 0x394);
        i = i.wrapping_add(1);
        s3 = s3.wrapping_add(4);
        s0 = s0.wrapping_add(4);
        if !((i as i32) < n as i32) {
            break;
        }
    }
}

/// FUN_001f0ad0(a0, n, list): weighted pick: t = FUN_0011d448(0, sum(list[i]+0x14)); return first list[i]
/// where t -= w_i <= 0, else 0.
pub fn ee_sub_1f0ad0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) -> u32 {
    let n = a1 as i32;
    let mut s = 0.0f32;
    if n > 0 {
        let mut p = a2;
        let mut c = a1;
        loop {
            let e = m.r32(p);
            c = c.wrapping_sub(1);
            p = p.wrapping_add(4);
            s += m.rf(e + 0x14);
            if c == 0 {
                break;
            }
        }
    }
    let mut t = m.call(0x0011_d448, &[a0, a1, a2], &[f4(0.0), f4(s)]).1[0];
    if n <= 0 {
        return 0;
    }
    let mut i: i32 = 0;
    let mut p = a2;
    loop {
        let e = m.r32(p);
        t -= m.rf(e + 0x14);
        if t <= 0.0 {
            return e;
        }
        i += 1;
        p = p.wrapping_add(4);
        if !(i < n) {
            break;
        }
    }
    0
}

/// FUN_0011b570(dst, v): |v.xyz| < 1e-4 -> dst = identity (0x2a2d00); else FUN_0011b500(&tmp, v/|v|, |v|);
/// dst = tmp (0x30 bytes). return dst.
pub fn ee_sub_11b570<M: QMem + ?Sized>(m: &mut M, a0: u32, v: V4) -> u32 {
    let l = (v[0] * v[0] + v[1] * v[1] + 1.0 * (v[2] * v[2])).sqrt();
    if l < f32::from_bits(0x38d1_b717) {
        m.call(0x0023_320c, &[a0, 0x002a_2d00, 0x30], &[]);
        return a0;
    }
    let q = 1.0 / l;
    let sp = stack_push(m, 0x40);
    m.wv(sp + 0x30, [l, 0.0, 0.0, 0.0]);
    m.call(0x0011_b500, &[sp], &[[v[0] * q, v[1] * q, v[2] * q, v[3] * q], f4(l)]);
    m.call(0x0023_320c, &[a0, sp, 0x30], &[]);
    stack_pop(m, 0x40);
    a0
}

/// FUN_001a12f0(o): loop { s = o[1]; FUN_00131968(o[0]->+0x1030) -> FUN_001a15d0(o); } until o[1] == -1 or
/// o[1] == s.
pub fn ee_sub_1a12f0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    loop {
        let s = m.r32(a0 + 4);
        let p = m.r32(a0);
        let x = m.r32(p + 0x1030);
        if m.call(0x0013_1968, &[x], &[]).0 != 0 {
            m.call(0x001a_15d0, &[a0], &[]);
        }
        let v = m.r32(a0 + 4);
        if v == 0xffff_ffff || v == s {
            break;
        }
    }
}

/// FUN_00129a10(o, x): x -> x.vslot0(); FUN_00129ec8(o); o[1] = x; o[0] = 0x21; return o.
pub fn ee_sub_129a10<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    if a1 != 0 {
        vslot(m, a1, 0, &[], &[]);
    }
    m.call(0x0012_9ec8, &[a0], &[]);
    m.w32(a0 + 4, a1);
    m.w32(a0, 0x21);
    a0
}

/// FUN_0020a5f8(o, x): +0xc = x; +0x14 = 1; +0x18 = 2; +8 = +0x10 = -1; FUN_0011a220(0x3e145c, o); return o.
pub fn ee_sub_20a5f8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    m.w32(a0 + 0xc, a1);
    m.wf(a0 + 0x14, 1.0);
    m.wf(a0 + 0x18, 2.0);
    m.w32(a0 + 8, 0xffff_ffff);
    m.w32(a0 + 0x10, 0xffff_ffff);
    m.call(0x0011_a220, &[0x003e_145c, a0], &[]);
    a0
}

/// FUN_00255718(o): o[0] = 0; FUN_00255588(o, 0x2dd2a0); o[1] bit31 = DAT_002dd598; return o.
pub fn ee_sub_255718<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    m.w32(a0, 0);
    m.call(0x0025_5588, &[a0, 0x002d_d2a0], &[]);
    let w = m.r32(a0 + 4);
    let b = m.r32(0x002d_d598);
    m.w32(a0 + 4, (w & 0x7fff_ffff) | (b << 31));
    a0
}

/// FUN_001af608(o): +0x1e8 == 7 -> FUN_001aeaa8(o, 2); FUN_001b0108(o) -> +0x1ec = +0x24 = +0x24 - 1.
pub fn ee_sub_1af608<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    if m.r32(a0 + 0x1e8) != 7 {
        return;
    }
    m.call(0x001a_eaa8, &[a0, 2], &[]);
    if m.call(0x001b_0108, &[a0], &[]).0 != 0 {
        let v = m.r32(a0 + 0x24).wrapping_sub(1);
        m.w32(a0 + 0x1ec, v);
        m.w32(a0 + 0x24, v);
    }
}

const T: u32 = 0x002d_ded0; // frame clock (float)

/// FUN_00226250(o): +0x150 && FUN_001c4700(+0x150, 0xff) -> [+0x1c0 -> v = FUN_001cb250(+0x1c0, 0), !v ->
/// return; +0x1c8 = v; +0x1c4 -> FUN_001d6b00(+0x1c4, v)]; o.vslot2(5, 0, 0, 0, 0).
pub fn ee_sub_226250<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let p = m.r32(a0 + 0x150);
    if p == 0 {
        return;
    }
    if m.call(0x001c_4700, &[p, 0xff], &[]).0 == 0 {
        return;
    }
    let q = m.r32(a0 + 0x1c0);
    if q != 0 {
        let v = m.call(0x001c_b250, &[q, 0], &[]).0;
        if v == 0 {
            return;
        }
        let r = m.r32(a0 + 0x1c4);
        m.w32(a0 + 0x1c8, v);
        if r != 0 {
            m.call(0x001d_6b00, &[r, v], &[]);
        }
    }
    vslot(m, a0, 2, &[5, 0, 0, 0, 0], &[]);
}

/// FUN_001ed2d8(c): DAT_003e12c8 -> l = (0.3,0.6,0.1)·c.xyz; tmp = {60, 0, FUN_0011d948(0x2bbb60,
/// FUN_00245b58(l, 0.6))}; FUN_0011dd90(&tmp, c).
pub fn ee_sub_1ed2d8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    if m.r32(0x003e_12c8) == 0 {
        return;
    }
    let sp = stack_push(m, 0x20);
    m.wf(sp + 0x10, f32::from_bits(0x3e99_999a));
    m.wf(sp + 0x18, f32::from_bits(0x3dcc_cccd));
    m.wf(sp + 0x14, f32::from_bits(0x3f19_999a));
    let c = m.rv(a0);
    let w = m.rv(sp + 0x10);
    let l = c[0] * w[0] + c[1] * w[1] + 1.0 * (c[2] * w[2]);
    let f0 = m.rf(0x002c_3690);
    let f1 = m.rf(0x002c_3694);
    let f13 = m.rf(0x002c_3698);
    m.wf(sp, f0);
    m.wf(sp + 4, f1);
    let r = m.call(0x0024_5b58, &[], &[f4(l), f4(f13)]).1[0];
    let r = m.call(0x0011_d948, &[0x002b_bb60], &[f4(r)]).1[0];
    m.wf(sp + 8, r);
    m.call(0x0011_dd90, &[sp, a0], &[]);
    stack_pop(m, 0x20);
}

/// FUN_0020a298(o, x, mask): +0x3d0 -> 0; FUN_00132b10(x, 0x36) && !FUN_00132b10(o, 0x3a) &&
/// !(+0x2c0 & FUN_00207a20(6)) -> 1; p = FUN_00154180(+0xf64, x); !p || !p[1] || T - p[3] < 0.2 -> 1;
/// return (+0x2c0 & mask) != 0.
pub fn ee_sub_20a298<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) -> u32 {
    if m.r32(a0 + 0x3d0) != 0 {
        return 0;
    }
    if m.call(0x0013_2b10, &[a1, 0x36], &[]).0 != 0 {
        if m.call(0x0013_2b10, &[a0, 0x3a], &[]).0 == 0 {
            let k = m.call(0x0020_7a20, &[6], &[]).0 as u64;
            if m.r64(a0 + 0x2c0) & k == 0 {
                return 1;
            }
        }
    }
    let f = m.r32(a0 + 0xf64);
    let p = m.call(0x0015_4180, &[f, a1], &[]).0;
    if p == 0 {
        return 1;
    }
    if m.r32(p + 4) != 0 {
        let d = m.rf(T) - m.rf(p + 0xc);
        if d < m.rf(0x002c_3dac) {
            return 1;
        }
        if m.r64(a0 + 0x2c0) & (a2 as u64) == 0 {
            return 0;
        }
    }
    1
}

/// FUN_0014db70(cm): camera projection setup from fov +0x20, aspect +0x3c, +0x18, +0x4c, +0x50 ->
/// FUN_0011c448(cm+0x100, ..); FUN_0014e270(cm, cm+0x250, 1.0); FUN_0014dc58(cm).
pub fn ee_sub_14db70<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let f20 = 1.0f32;
    let fov = m.rf(a0 + 0x20);
    let t = m.call(0x0024_50f0, &[a0], &[f4(fov * 0.5)]).1[0];
    let f2 = f20 / m.rf(a0 + 0x20);
    let asp = m.rf(a0 + 0x3c);
    let f1 = m.rf(a0 + 0x18);
    let f3 = asp * asp + f20;
    m.wf(a0 + 0x44, t);
    let f12 = t * asp;
    m.wf(a0 + 0x40, f12);
    let f1 = f20.max(f1 * f2);
    m.wf(a0 + 0x1c, f1);
    let f2 = f3.sqrt();
    let f2 = t * f2;
    let p4c = m.rf(a0 + 0x4c);
    let f0 = t * 4096.0;
    let f12 = f12 * 4096.0;
    let f17 = m.rf(a0 + 0x50);
    let f13 = 224.0 / f0;
    let f12 = 512.0 / f12;
    let f2 = f2 * p4c + f20;
    m.wf(a0 + 0x48, f2);
    m.call(
        0x0011_c448,
        &[a0 + 0x100],
        &[f4(f12), f4(f13), f4(0.0), f4(0.0), f4(p4c), f4(f17)],
    );
    m.call(0x0014_e270, &[a0, a0 + 0x250], &[f4(f20)]);
    m.call(0x0014_dc58, &[a0], &[]);
}

/// FUN_00172f10(o): +0xd0 < 4 -> [+0x3c == FLT_MAX && FUN_0017dcb8(+0x18) -> +0x3c = T + 1]; T >= +0x3c ->
/// q = o+0x50+32*+0xd0; +0xd0 = (+0xd0+1) % 4; FUN_002562a8(&tmp); tmp.+0x10 = *q; FUN_0017dd38(+0x18, &tmp);
/// +0x3c = FLT_MAX.
pub fn ee_sub_172f10<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    if m.r32(a0 + 0xd0) >= 4 {
        return;
    }
    let big = m.rf(0x002a_53d4);
    let s2 = m.r32(a0 + 0x18);
    if m.rf(a0 + 0x3c) == big {
        if m.call(0x0017_dcb8, &[s2], &[]).0 != 0 {
            let t = m.rf(T);
            m.wf(a0 + 0x3c, t + 1.0);
        }
    }
    if m.rf(T) < m.rf(a0 + 0x3c) {
        return;
    }
    let v = m.r32(a0 + 0xd0);
    let s1 = a0.wrapping_add((v << 5).wrapping_add(0x50));
    let v = v.wrapping_add(1);
    m.w32(a0 + 0xd0, v);
    if v >= 4 {
        m.w32(a0 + 0xd0, 0);
    }
    let sp = stack_push(m, 0x20);
    m.call(0x0025_62a8, &[sp], &[]);
    let q = m.rv(s1);
    m.wv(sp + 0x10, q);
    m.call(0x0017_dd38, &[s2, sp], &[]);
    stack_pop(m, 0x20);
    let big = m.rf(0x002a_53d4);
    m.wf(a0 + 0x3c, big);
}

/// FUN_00258f40(o, rd): n = FUN_00117c20(rd); n -> e = (+0xe4)+0xc0; arr = FUN_001ae1d0({4n+4, 0x10});
/// e+0x1c = arr; e+0xc = FUN_00117bd0(rd); arr[0] = min(n, 32); for i < n: w = FUN_00117c80(rd);
/// FUN_001ed278(&w); i < arr[0] -> arr[1+i] = w.
pub fn ee_sub_258f40<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let n = m.call(0x0011_7c20, &[a1], &[]).0;
    if n == 0 {
        return;
    }
    let e = m.r32(a0 + 0xe4).wrapping_add(0xc0);
    let sp = stack_push(m, 0x10);
    m.w32(sp + 4, 0x10);
    m.w32(sp, (n << 2).wrapping_add(4));
    let arr = m.call(0x001a_e1d0, &[sp], &[]).0;
    m.w32(e + 0x1c, arr);
    let v = m.call(0x0011_7bd0, &[a1], &[]).0;
    m.w32(e + 0xc, v);
    let c = if 0x1f < n as i32 { 0x20 } else { n };
    m.w32(arr, c);
    if (n as i32) > 0 {
        let mut i: i32 = 0;
        let mut d = arr.wrapping_add(4);
        loop {
            let w = m.call(0x0011_7c80, &[a1], &[]).0;
            m.w32(sp, w);
            m.call(0x001e_d278, &[sp], &[]);
            let lim = m.r32(arr) as i32;
            let hit = i < lim;
            i += 1;
            if hit {
                let w = m.r32(sp);
                m.w32(d, w);
            }
            d = d.wrapping_add(4);
            if !(i < n as i32) {
                break;
            }
        }
    }
    stack_pop(m, 0x10);
}

/// FUN_002283d0(o): +0x13c == 3 -> +0x134 ? (FUN_00228358(o, 0); n = FUN_002280d0(GS); n -> FUN_002283d0(n))
/// : (FUN_00161f48(0x2deb30, +0x164, o+0x168, o); FUN_0014e4c8 x2; FUN_00228358(o, 4); +0x158 = T + +0x160;
/// o.vslot2(5, 0, 0, 0, 0)).
pub fn ee_sub_2283d0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    if m.r32(a0 + 0x13c) != 3 {
        return;
    }
    if m.r32(a0 + 0x134) != 0 {
        m.call(0x0022_8358, &[a0, 0], &[]);
        let n = m.call(0x0022_80d0, &[0x003d_d490], &[]).0;
        if n != 0 {
            m.call(0x0022_83d0, &[n], &[]);
        }
        return;
    }
    let a1 = m.r32(a0 + 0x164);
    m.call(0x0016_1f48, &[0x002d_eb30, a1, a0 + 0x168, a0], &[]);
    let s0 = 0x002d_df40u32;
    m.call(0x0014_e4c8, &[s0, 2, 0xffff_ffff, s0 + 0x450, 0, 0], &[]);
    m.call(0x0014_e4c8, &[s0, 2, 0xffff_ffff, s0 + 0x520, 0, 0], &[]);
    m.call(0x0022_8358, &[a0, 4], &[]);
    let t = m.rf(T) + m.rf(a0 + 0x160);
    m.wf(a0 + 0x158, t);
    vslot(m, a0, 2, &[5, 0, 0, 0, 0], &[]);
}

/// FUN_001f72f8(o, pos): any entry (+0xe4, stride 0xb0, count +0xe0) without bit50 of +0x10 that contains pos
/// (bit49: |pos - e[0]->+0x70| < e+8; or e+0x74 && FUN_00118fb0(e+0x74, 0x7fffffff, pos)) -> 1; else 0.
pub fn ee_sub_1f72f8<M: QMem + ?Sized>(m: &mut M, a0: u32, v: V4) -> u32 {
    if (m.r32(a0 + 0xe0) as i32) <= 0 {
        return 0;
    }
    let mut i: i32 = 0;
    let mut off: u32 = 0;
    loop {
        let e = m.r32(a0 + 0xe4).wrapping_add(off);
        let d = m.r64(e + 0x10);
        if d & (1u64 << 50) == 0 {
            let mut hit = false;
            if d & (1u64 << 49) != 0 {
                let p = m.r32(e);
                let r = m.rf(e + 8);
                let q = m.rv(p + 0x70);
                let (x, y, z) = (v[0] - q[0], v[1] - q[1], v[2] - q[2]);
                let dd = x * x + y * y + 1.0 * (z * z);
                hit = dd < r * r;
            }
            if hit {
                return 1;
            }
            let q = m.r32(e + 0x74);
            if q != 0 && m.call(0x0011_8fb0, &[q, 0x7fff_ffff], &[v]).0 != 0 {
                return 1;
            }
        }
        i += 1;
        off = off.wrapping_add(0xb0);
        if !(i < m.r32(a0 + 0xe0) as i32) {
            break;
        }
    }
    0
}

/// FUN_001a2230(o, kind): o[0]+0x298 -> 1; kind 0x2000/0x4000 -> by o[0]+0xdf4 / +0x4c8 / +0x2dc tables
/// (0x2a6830 / 0x2a6870); else o[1] != -1 || +0x90 == 1 || DAT_002e1e00 & 1 -> 1; else FUN_0020f0b0(GS) != 0.
pub fn ee_sub_1a2230<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let p = m.r32(a0);
    if m.r32(p + 0x298) != 0 {
        return 1;
    }
    if a1 == 0x2000 || a1 == 0x4000 {
        let i = m.r32(p + 0xdf4).wrapping_add(1);
        match i {
            0 => {
                let p = m.r32(a0);
                let j = m.r32(p + 0x4c8).wrapping_add(1);
                match j {
                    0 => {
                        let v = m.r32(p + 0x2dc) as i32;
                        if v == -1 || v < -1 || !(v < 0x12) || v < 8 {
                            return 0;
                        }
                        (a1 == 0x4000) as u32
                    }
                    1 | 4 | 5 | 8 | 9 => 1,
                    j if j >= 11 => 1,
                    _ => 0,
                }
            }
            4 | 6 | 11 | 12 => 1,
            _ => 0,
        }
    } else {
        if m.r32(a0 + 4) as i32 != -1 {
            return 1;
        }
        if m.r32(a0 + 0x90) == 1 {
            return 1;
        }
        if m.r32(0x002e_1e00) & 1 != 0 {
            return 1;
        }
        (m.call(0x0020_f0b0, &[0x003d_d490], &[]).0 != 0) as u32
    }
}

fn static_pools_init<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if a1 != 0xffff || a0 == 0 {
        return;
    }
    const P: [(u32, u32, u32, u32); 6] = [
        (0x002d_d2e0, 0x0045_20a8, 0x14, 0x2000),
        (0x002d_d2e4, 0x0045_20d0, 0x18, 0x800),
        (0x002d_d2e8, 0x0045_20f8, 0x28, 0x200),
        (0x002d_d2ec, 0x0045_2120, 0x1c, 0x800),
        (0x002d_d2f0, 0x0045_2148, 0xc, 0xc8),
        (0x002d_d2f4, 0x0045_2170, 0x14, 0x100),
    ];
    for (cnt, pool, sz, n) in P {
        let v = m.r32(cnt).wrapping_add(1);
        m.w32(cnt, v);
        if v == 1 {
            m.call(0x0011_cca8, &[pool, sz, n, 0x10], &[]);
        }
    }
}

/// FUN_00190f38(init, prio): static-init: (1, 0xffff) -> ref-counted FUN_0011cca8 pool inits (0x4520a8..).
pub fn ee_sub_190f38<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    static_pools_init(m, a0, a1)
}

/// FUN_001a2ec8: same static-init body as FUN_00190f38.
pub fn ee_sub_1a2ec8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    static_pools_init(m, a0, a1)
}

/// FUN_001cd498: same static-init body as FUN_00190f38.
pub fn ee_sub_1cd498<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    static_pools_init(m, a0, a1)
}

/// FUN_001ffba0: same static-init body as FUN_00190f38.
pub fn ee_sub_1ffba0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    static_pools_init(m, a0, a1)
}

/// FUN_001d84b0(o, dst): dst = copy of 0x5408 bytes at 0x3d4a60; id = FUN_001d8720(o+8, 0x2a78f8); id != -1 ->
/// for each 10-byte entry e of o[1] (count o[0]) with (s16)e[0] == id: dst[(u16)e[4]] = 1.
pub fn ee_sub_1d84b0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let src = 0x003d_4a60u32;
    let mut k = 0;
    while k < 0x5408 {
        let w = m.r32(src + k);
        m.w32(a1.wrapping_add(k), w);
        k += 4;
    }
    let id = m.call(0x001d_8720, &[a0 + 8, 0x002a_78f8], &[]).0;
    if id == 0xffff_ffff {
        return;
    }
    if (m.r32(a0) as i32) <= 0 {
        return;
    }
    let mut i: i32 = 0;
    let mut off: u32 = 0;
    loop {
        let e = m.r32(a0 + 4).wrapping_add(off);
        if m.r16(e) as i16 as i32 as u32 == id {
            let j = m.r16(e + 4) as u32;
            m.w8(a1.wrapping_add(j), 1);
        }
        i += 1;
        off = off.wrapping_add(10);
        if !(i < m.r32(a0) as i32) {
            break;
        }
    }
}

/// FUN_001b04e0(o): filter the 11 0x20-byte entries at 0x2bfec8 (kind 0x3c needs FUN_001d9880(0x3da530) != 0,
/// kind 0x3e needs it == 0) into 0x2f6470; o -> FUN_001af390(0x3d9ed0, n, 0x2f6470), o+0x1c |= 2; else
/// FUN_001af4d8(0x3d9ed0, n, 0x2f6470, *(0x3d9ef0), 0x2bfd38).
pub fn ee_sub_1b04e0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let mut s0 = 0x002b_fec8u32;
    let mut s1 = 0x002f_6470u32;
    let mut i = 0u32;
    loop {
        let v1 = m.r32(s0);
        let mut ok = true;
        if v1 == 0x3c {
            ok = m.call(0x001d_9880, &[0x003d_a530], &[]).0 != 0;
        } else if v1 == 0x3e {
            ok = m.call(0x001d_9880, &[0x003d_a530], &[]).0 == 0;
        }
        i += 1;
        if ok {
            for k in 0..8 {
                let w = m.r32(s0 + 4 * k);
                m.w32(s1 + 4 * k, w);
            }
            s1 = s1.wrapping_add(0x20);
        }
        s0 = s0.wrapping_add(0x20);
        if i >= 0xb {
            break;
        }
    }
    let n = ((s1.wrapping_sub(0x002f_6470) as i32) >> 5) as u32;
    if a0 != 0 {
        m.call(0x001a_f390, &[0x003d_9ed0, n, 0x002f_6470], &[]);
        let v = m.r32(a0 + 0x1c) | 2;
        m.w32(a0 + 0x1c, v);
    } else {
        let a3 = m.r32(0x003d_9ef0);
        m.call(0x001a_f4d8, &[0x003d_9ed0, n, 0x002f_6470, a3, 0x002b_fd38], &[]);
    }
}

/// FUN_00199408(_, o, edge, out): pick the face with weight > 0.7 whose neighbour is < 0.55; out = unit
/// edge normal (-dy, dx, 0) (or DAT_002a2da0 if degenerate) flipped to face that face's normal; return 1/0.
pub fn ee_sub_199408<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32) -> u32 {
    let t0 = a1 + 0x90;
    let arr = m.r32(t0 + 0xc);
    let h4 = m.r16(a2 + 4) as u32;
    let h6 = m.r16(a2 + 6) as u32;
    let v1 = arr.wrapping_add(h4 << 5);
    let p = arr.wrapping_add(h6 << 5);
    let hi = f32::from_bits(0x3f33_3333);
    let lo = f32::from_bits(0x3f0c_cccd);
    let t1 = if hi < m.rf(v1 + 8) && m.rf(p + 8) < lo {
        p
    } else if m.rf(v1 + 8) < lo && hi < m.rf(p + 8) {
        v1
    } else {
        return 0;
    };
    let verts = m.r32(t0 + 4);
    let h2 = m.r16(a2 + 2) as u32;
    let h0 = m.r16(a2) as u32;
    let a = m.rv(verts.wrapping_add(h2 << 4));
    let b = m.rv(verts.wrapping_add(h0 << 4));
    let sp = stack_push(m, 0x30);
    m.w32(sp + 0x18, 0);
    m.wf(sp + 0x14, a[0] - b[0]);
    m.wf(sp + 0x10, -(a[1] - b[1]));
    let mut v = m.rv(sp + 0x10);
    let len = (v[0] * v[0] + v[1] * v[1] + 1.0 * (v[2] * v[2])).sqrt();
    if len < f32::from_bits(0x38d1_b717) {
        v = m.rv(0x002a_2da0);
    } else {
        let q = 1.0 / len;
        v = [v[0] * q, v[1] * q, v[2] * q, v[3] * q];
    }
    let n = m.rv(t1);
    let d = v[0] * n[0] + v[1] * n[1] + 1.0 * (v[2] * n[2]);
    if d < 0.0 {
        v = [-v[0], -v[1], -v[2], -v[3]];
    }
    m.wv(a3, v);
    stack_pop(m, 0x30);
    1
}

/// FUN_001e7bb8(o, mn, mx): FUN_00231368(o, mn, mx); world-space AABB of the curve points (+0xf4, count
/// +0xe8; +0xe0 -> include Bezier tangent handles +0x144 / +0x140 scaled by knot deltas +0xec / 3).
/// The w lanes of the stored min/max are register leftovers in the asm; written as 0 here.
pub fn ee_sub_1e7bb8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    m.call(0x0023_1368, &[a0, a1, a2], &[]);
    let r0 = m.rv(a0 + 0xa0);
    let r1 = m.rv(a0 + 0xb0);
    let r2 = m.rv(a0 + 0xc0);
    let tr = m.rv(a0 + 0xd0);
    let xf = |p: V4, t: bool| -> V4 {
        let mut o = [0.0f32; 4];
        for k in 0..3 {
            o[k] = r0[k] * p[0] + r1[k] * p[1] + r2[k] * p[2] + if t { tr[k] } else { 0.0 };
        }
        o
    };
    let mx3 = |a: V4, b: V4| [a[0].max(b[0]), a[1].max(b[1]), a[2].max(b[2]), a[3]];
    let mn3 = |a: V4, b: V4| [a[0].min(b[0]), a[1].min(b[1]), a[2].min(b[2]), a[3]];
    let mut pts = m.r32(a0 + 0xf4);
    let mut v5 = xf(m.rv(pts), true);
    let mut v6 = v5;
    if m.r32(a0 + 0xe0) == 0 {
        let n = m.r32(a0 + 0xe8) as i32;
        if 1 < n {
            let mut c = n - 1;
            let mut p = pts + 0x10;
            loop {
                let v = xf(m.rv(p), true);
                c -= 1;
                p = p.wrapping_add(0x10);
                v5 = mx3(v5, v);
                v6 = mn3(v6, v);
                if c == 0 {
                    break;
                }
            }
        }
    } else {
        let n = (m.r32(a0 + 0xe8) as i32).wrapping_sub(1);
        if n > 0 {
            let mut ta = m.r32(a0 + 0x140).wrapping_add(0x10);
            let mut kn = m.r32(a0 + 0xec);
            let mut tb = m.r32(a0 + 0x144);
            let mut p = pts.wrapping_add(0x10);
            let mut v7 = v5;
            let mut c = n;
            loop {
                let f0 = m.rf(kn);
                let v4p = v7;
                let f1 = m.rf(kn + 4) - f0;
                c -= 1;
                kn = kn.wrapping_add(4);
                v7 = xf(m.rv(p), true);
                p = p.wrapping_add(0x10);
                let c3 = f1 * f32::from_bits(0x3eaa_aaab);
                let c2 = f1 * f32::from_bits(0xbeaa_aaab);
                let t = xf(m.rv(tb), false);
                tb = tb.wrapping_add(0x10);
                let v4 = [v4p[0] + t[0] * c3, v4p[1] + t[1] * c3, v4p[2] + t[2] * c3, 0.0];
                v5 = mx3(v5, v4);
                v6 = mn3(v6, v4);
                let t = xf(m.rv(ta), false);
                ta = ta.wrapping_add(0x10);
                let v2 = [v7[0] + t[0] * c2, v7[1] + t[1] * c2, v7[2] + t[2] * c2, 0.0];
                v5 = mx3(v5, v2);
                v6 = mn3(v6, v2);
                v6 = mn3(v6, v7);
                v5 = mx3(v5, v7);
                if c == 0 {
                    break;
                }
            }
        }
    }
    m.wv(a1, [v6[0], v6[1], v6[2], 0.0]);
    m.wv(a2, [v5[0], v5[1], v5[2], 0.0]);
}

/// FUN_0018f0a8(o): per-state (+0x18c8, jump table 0x2a6040) re-evaluation of the player's stance; when the
/// stance is "free" and +0x18d8 is -1 or in 12..23 with +0x18e4, re-pick via FUN_0018f050 / FUN_0018ee38.
pub fn ee_sub_18f0a8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let v = m.r32(a0 + 0x18d8) as i32;
    if v < 0x16 && !(v < 0xc) {
        let p = m.r32(a0 + 0x150);
        if m.r32(p + 0x30) & 0x80 == 0 {
            m.w32(a0 + 0x18ec, 0);
        }
    }
    let mut b = false;
    match m.r32(a0 + 0x18c8) {
        0 | 1 | 10 | 12 => b = true,
        3 | 0x15 | 0x16 => {
            let p = m.r32(a0 + 0x150);
            if m.call(0x001c_46c0, &[p, 0x80], &[f4(0.0)]).0 != 0 {
                m.call(0x0019_5a00, &[a0 + 0x2c0, 0, 0, 0], &[]);
            }
        }
        5 => {
            let i = m.r32(a0 + 0x18d0) as i32;
            if i < 6 {
                b = true;
            } else if i < 8 {
                let p = m.r32(a0 + 0x150);
                if m.r32(p + 0x30) & 0x80 == 0 {
                    let k = if m.r32(a0 + 0x18e8) != 0 { 8 } else { 4 };
                    m.call(0x0018_c250, &[a0, 5, k], &[]);
                }
            } else {
                if i < 10 {
                    return;
                }
                b = true;
            }
        }
        7 => {
            let i = m.r32(a0 + 0x4c8) as i32;
            if i < 5 {
                b = 2 < i;
            }
        }
        _ => {}
    }
    if !b {
        return;
    }
    let i = m.r32(a0 + 0x18d8) as i32;
    if i != -1 {
        if i < -1 || !(i < 0x18) || i < 0xc || m.r32(a0 + 0x18e4) == 0 {
            return;
        }
    }
    let f = f32::from_bits(0x3eb3_3333);
    let p = m.r32(a0 + 0x150);
    if m.call(0x001c_46c0, &[p, 0x10], &[f4(f)]).0 != 0 {
        m.call(0x0018_e540, &[a0, 0xffff_ffff], &[]);
        let r = m.call(0x0018_f050, &[a0], &[]).0;
        m.call(0x0018_e540, &[a0, r], &[]);
        return;
    }
    let p = m.r32(a0 + 0x150);
    if m.call(0x001c_46c0, &[p, 0x80], &[f4(f)]).0 == 0 {
        return;
    }
    m.call(0x0018_e540, &[a0, 0xffff_ffff], &[]);
    let r = m.call(0x0018_ee38, &[a0], &[]).0;
    m.call(0x0018_e540, &[a0, r], &[]);
}

/// FUN_001880d8(o): latch / repeat the +0x1028 direction (-1/+1) from input FUN_00187d80(o, 0x5000) and the
/// stick (+0x103c/+0x1040/+0x1044), with auto-repeat timer +0x1034 / interval +0x1038; return +0x1028.
pub fn ee_sub_1880d8<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let third = f32::from_bits(0x3eaa_a64c);
    let f;
    if m.r32(a0 + 0x1028) == 0 {
        let r = m.call(0x0018_7d80, &[a0, 0x5000], &[]).0;
        let mut set: Option<u32> = None;
        if r & 0x1000 != 0 {
            set = Some(0xffff_ffff);
        } else if r & 0x4000 != 0 {
            set = Some(1);
        } else if f32::from_bits(0x3f4c_cccd) <= m.rf(a0 + 0x1044) {
            let y = m.rf(a0 + 0x1040);
            if m.rf(a0 + 0x103c).abs() < y.abs() {
                set = Some(if 0.0 < y { 0xffff_ffff } else { 1 });
            }
        }
        if let Some(v) = set {
            m.w32(a0 + 0x1028, v);
        }
        if m.r32(a0 + 0x1028) == 0 {
            return m.r32(a0 + 0x1028);
        }
        m.wf(a0 + 0x1038, third);
        f = m.rf(0x002d_df0c);
        m.wf(a0 + 0x1034, f + third);
        return m.r32(a0 + 0x1028);
    }
    let mut b = false;
    let neg = (m.r32(a0 + 0x1028) as i32) < 0;
    let bit = if neg { 0x1000 } else { 0x4000 };
    if m.r32(a0 + 0x10b4) & bit != 0 {
        b = true;
    } else if 0.25 <= m.rf(a0 + 0x1044) {
        let y = m.rf(a0 + 0x1040);
        if m.rf(a0 + 0x103c).abs() < y.abs() {
            b = if neg { 0.0 < y } else { y < 0.0 };
        }
    }
    if !b {
        m.w32(a0 + 0x1028, 0);
        return m.r32(a0 + 0x1028);
    }
    if !(m.rf(a0 + 0x1034) <= m.rf(0x002d_df0c)) {
        return 0;
    }
    let x = m.rf(a0 + 0x1038);
    let g = (x + x) * f32::from_bits(0x3eaa_aaab);
    let lo = f32::from_bits(0x3dcc_cccd);
    let v = if g < lo { lo } else if third < g { third } else { g };
    m.wf(a0 + 0x1038, v);
    let t = m.rf(0x002d_df0c);
    m.wf(a0 + 0x1034, t + v);
    m.r32(a0 + 0x1028)
}

/// FUN_0018df60(o): copy the 0x18c4 target's matrix (vslot 30) rotation into o+0x1b0; smooth orientation
/// (FUN_0011c950 -> vslot 35) and position (FUN_0011f088 -> vslot 34) toward it; derive the lean blend
/// weights +0x1f60..+0x1f98 from the normalized FUN_001a6fc0 direction's y; FUN_001384c0(+0x1f90).
/// The w lane of the transformed target point is a register leftover in the asm (0 here).
pub fn ee_sub_18df60<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let half = 0.5f32;
    let t = m.r32(a0 + 0x18c4);
    let mp = vslot(m, t, 30, &[], &[]).0;
    let r0 = m.rv(mp);
    let r1 = m.rv(mp + 0x10);
    let r2 = m.rv(mp + 0x20);
    let r3 = m.rv(mp + 0x30);
    m.wv(a0 + 0x1b0, r0);
    m.wv(a0 + 0x1c0, r1);
    m.wv(a0 + 0x1d0, r2);
    let sp = stack_push(m, 0x60);
    let v = m.call(0x001a_6318, &[a0], &[]).0;
    let f12 = m.rf(0x002d_ded4) * half;
    m.call(0x0011_c950, &[v, a0 + 0x1b0, 0x002b_f3f8, 0, sp], &[f4(f12)]);
    let q = m.rv(sp);
    vslot(m, a0, 35, &[], &[q]);
    let d = m.rv(a0 + 0x1fa0);
    let d = [-d[0], -d[1], -d[2], -d[3]];
    let mut tp = [0.0f32; 4];
    for k in 0..3 {
        tp[k] = r0[k] * d[0] + r1[k] * d[1] + r2[k] * d[2] + r3[k];
    }
    let cur = m.call(0x001a_62e8, &[a0], &[]).1;
    let f12 = m.rf(0x002d_ded4) * half;
    m.call(0x0011_f088, &[0x002b_f408, sp + 0x10], &[cur, tp, f4(f12)]);
    let q = m.rv(sp + 0x10);
    vslot(m, a0, 34, &[], &[q]);
    let p = m.r32(a0 + 0x150);
    let x = [0.0f32, -m.rf(p + 0x10), m.rf(p + 0x14), 0.0];
    let a = m.rv(0x002d_dfa0);
    let b = m.rv(0x002d_dfb0);
    let c = m.rv(0x002d_dfc0);
    let mut w = [0.0f32; 4];
    for k in 0..3 {
        w[k] = a[k] * x[0] + b[k] * x[1] + c[k] * x[2];
    }
    let t = m.r32(a0 + 0x18c4);
    let u = m.call(0x001a_6fc0, &[0, t], &[w]).1;
    let len = (u[0] * u[0] + u[1] * u[1] + 1.0 * (u[2] * u[2])).sqrt();
    let y = if len < f32::from_bits(0x38d1_b717) {
        let z = m.rv(0x002a_2dc0);
        z[1] * -1.0
    } else {
        u[1] * (1.0 / len)
    };
    stack_pop(m, 0x60);
    let f4p = y.max(0.0);
    let f1 = (-y).max(0.0);
    let f3 = 1.0 - f4p - f1;
    let p = m.r32(a0 + 0x150);
    m.wf(a0 + 0x1f94, y);
    let s = m.rf(p + 0x1c);
    m.wf(a0 + 0x1f98, s);
    m.wf(a0 + 0x1f6c, f1 * s);
    m.wf(a0 + 0x1f60, 1.0 - s);
    m.wf(a0 + 0x1f84, f4p * s);
    m.wf(a0 + 0x1f78, f3 * s);
    let h = m.r32(a0 + 0x1f90);
    m.call(0x0013_84c0, &[h], &[]);
}
