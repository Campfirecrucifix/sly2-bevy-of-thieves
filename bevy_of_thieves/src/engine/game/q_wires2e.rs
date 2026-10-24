//! WIRE-S2: 1:1 QMem ports of live-frontier natives (shard 3/4).
#![allow(unused)]
use crate::engine::classes::q::q2mem::{vslot, QMem, V4};

const T: u32 = 0x002d_ded0; // g_clock t

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

fn clamp(x: f32, lo: f32, hi: f32) -> f32 {
    if x < lo {
        lo
    } else if hi < x {
        hi
    } else {
        x
    }
}

fn swap_words<M: QMem + ?Sized>(m: &mut M, a: u32, b: u32, n: u32) {
    for i in 0..n {
        let x = m.r32(a + i.wrapping_mul(4));
        let y = m.r32(b + i.wrapping_mul(4));
        m.w32(a + i.wrapping_mul(4), y);
        m.w32(b + i.wrapping_mul(4), x);
    }
}

/// FUN_001a8410: bucket `n` 0x60-byte entries (base a1) by their u16 key (+0x54)[+0x58] into
/// keys a3[] / heads t0[] lists (+0x5c link); flipped entries (+0x4c set, +0x50 clear) are swapped;
/// dead entries chained on fp / *t1; then buckets whose key is not covered by the fp chain are
/// moved to *t1 and removed. *a2 = bucket count.
pub fn ee_sub_1a8410<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, t0: u32, t1: u32) {
    let n = a0;
    let base = a1;
    let keys = a3;
    let heads = t0;
    let head = t1;
    let mut fp: u32 = 0;
    let mut cnt: i32 = 0;
    m.w32(head, 0);
    let mut i: i32 = 0;
    while i < n as i32 {
        let e = (i as u32).wrapping_mul(0x60).wrapping_add(base);
        i += 1;
        let v4c = m.r32(e + 0x4c);
        let v50 = m.r32(e + 0x50);
        if v4c == 0 {
            if v50 == 0 {
                m.w32(e + 0x5c, fp);
                fp = e;
                continue;
            }
        } else if v50 != 0 {
            let h = m.r32(head);
            m.w32(e + 0x5c, h);
            m.w32(head, e);
            continue;
        } else {
            swap_words(m, e, e + 0x10, 4);
            swap_words(m, e + 0x20, e + 0x30, 4);
            swap_words(m, e + 0x40, e + 0x44, 1);
            swap_words(m, e + 0x4c, e + 0x50, 1);
            m.w32(e + 0x58, 1);
        }
        let t2 = m.r32(e + 0x54);
        let t9 = m.r32(e + 0x58);
        let kaddr = t2.wrapping_add(t9 << 1);
        let key = m.r16(kaddr) as u32;
        let mut s1: i32 = 0;
        while s1 < cnt {
            if key == m.r32(keys + (s1 as u32) * 4) {
                break;
            }
            s1 += 1;
        }
        let off = (s1 as u32) * 4;
        if s1 < cnt {
            let h = m.r32(heads + off);
            m.w32(e + 0x5c, h);
        } else {
            m.w32(keys + off, key);
            cnt += 1;
        }
        m.w32(heads + off, e);
    }
    let mut s1: i32 = 0;
    while s1 < cnt {
        let off = (s1 as u32) * 4;
        let b = m.r32(heads + off);
        if m.r32(b + 0x5c) != 0 {
            s1 += 1;
            continue;
        }
        let k = m.r32(keys + off);
        let mut a = fp;
        let mut covered = false;
        while a != 0 {
            let p = m.r32(a + 0x54);
            if m.r16(p) as u32 == k || m.r16(p + 2) as u32 == k {
                covered = true;
                break;
            }
            a = m.r32(a + 0x5c);
        }
        if covered {
            s1 += 1;
            continue;
        }
        let sz = (((cnt - s1) as u32) << 2).wrapping_sub(4);
        let h = m.r32(head);
        m.w32(b + 0x5c, h);
        m.w32(head, b);
        m.call(0x0023_32b8, &[heads + off, heads + off + 4, sz], &[]);
        m.call(0x0023_32b8, &[keys + off, keys + off + 4, sz], &[]);
        cnt -= 1;
    }
    m.w32(a2, cnt as u32);
}

/// FUN_00151340: FUN_0014fd40(a0, a1, a2); then compute a target (a2 stick +0x1070/+0x1074 when
/// +0x1078 > 0, else a spring-damper step when (*a0)[+0x4cc] == 9), FUN_0012d108(+0x14, tgt);
/// +0x30 = vel; FUN_0014e168(0x2ddf40, +0x10 + 0x40).
pub fn ee_sub_151340<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    let s1 = a0;
    let s0 = a2;
    m.call(0x0014_fd40, &[a0, a1, a2], &[]);
    let f21: f32 = 0.0;
    let vf00: V4 = [0.0, 0.0, 0.0, 1.0];
    let vf01: V4;
    let vf03: V4;
    if f21 < m.rf(s0 + 0x1078) {
        let f2 = m.rf(s0 + 0x1070);
        let f7: f32 = if f21 < f2 { -1.0 } else { 1.0 };
        let f0 = m.rf(s0 + 0x1074);
        let f8: f32 = if f21 < f0 { 1.0 } else { -1.0 };
        let x = f2.abs();
        let p = m.rf(0x002b_da10) + x * (m.rf(0x002b_da14) + x * m.rf(0x002b_da18));
        let f4v = clamp(p, m.rf(0x002b_da20), m.rf(0x002b_da24));
        let x = f0.abs();
        let p = m.rf(0x002b_da30) + x * (m.rf(0x002b_da34) + x * m.rf(0x002b_da38));
        let f4v = f7 * f4v;
        let c = clamp(p, m.rf(0x002b_da40), m.rf(0x002b_da44));
        let f1 = f8 * c;
        // pcpyld of a sign-extended mfc1: w = upper half of f4's sign extension
        let w = if (f4v.to_bits() as i32) < 0 { u32::MAX } else { 0 };
        vf01 = [0.0, f1, f4v, f32::from_bits(w)];
        vf03 = [0.0; 4];
    } else {
        let v0 = m.r32(s1);
        if m.r32(v0 + 0x4cc) != 9 {
            vf01 = [0.0; 4];
            vf03 = [0.0; 4];
        } else {
            if m.rf(s1 + 0x18) < m.rf(T) {
                let (c12, c13) = (m.rf(0x002b_da48), m.rf(0x002b_da4c));
                let f20 = m.call(0x0011_d448, &[], &[f4(c12), f4(c13)]).1[0];
                let (c12, c13) = (m.rf(0x002b_da50), m.rf(0x002b_da54));
                let f0 = m.call(0x0011_d448, &[], &[f4(c12), f4(c13)]).1[0];
                m.wf(s1 + 0x20, f21);
                m.wf(s1 + 0x28, f0);
                m.wf(s1 + 0x24, f20);
                let t = m.rf(T);
                let (c12, c13) = (m.rf(0x002b_da58), m.rf(0x002b_da5c));
                let f0 = m.call(0x0011_d448, &[], &[f4(c12), f4(c13)]).1[0];
                m.wf(s1 + 0x18, t + f0);
            }
            let k = m.rf(0x002b_da60);
            let nk = -m.rf(0x002b_da64);
            let p = m.r32(s1 + 0x14);
            let dt = m.rf(T + 4);
            let base = m.rv(p + 0x20);
            let a = m.rv(s1 + 0x20);
            let v2 = m.rv(s1 + 0x30);
            let h = dt * 0.5;
            let mut v3 = [0.0f32; 4];
            let mut v1 = [0.0f32; 4];
            for i in 0..4 {
                let d = a[i] - base[i];
                let acc = d * k;
                let t3 = acc + v2[i] * nk;
                let acc = v2[i] + vf00[i];
                v3[i] = acc + t3 * dt;
                let acc = base[i] + vf00[i];
                let acc = acc + v2[i] * h;
                v1[i] = acc + v3[i] * h;
            }
            vf01 = v1;
            vf03 = v3;
        }
    }
    let a0p = m.r32(s1 + 0x14);
    m.call(0x0012_d108, &[a0p], &[vf01]);
    m.wv(s1 + 0x30, vf03);
    let a1p = m.r32(s1 + 0x10).wrapping_add(0x40);
    m.call(0x0014_e168, &[0x002d_df40, a1p], &[]);
}

// SKIP 0x1a93c8: 0x3760-byte frame of 128-bit lq/sq records, VU0 macro heavy (vsqrt/vdiv Q,
// ACC chains, quad args via qmfc2 to FUN_00118668/FUN_00119390) — not ported.
