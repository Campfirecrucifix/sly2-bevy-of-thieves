//! WP-Q8: file packages run_0011abf8 (list / pool / hash / VU0 matrix helpers) and run_00120c48.
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem, V4};
use crate::engine::classes::q::q8util::*;

/// DAT_002dd2fc: head of the safe-iteration chain
const ITER_CHAIN: u32 = 0x002d_d2fc;

/// FUN_0011abf8: move every type-1 node (word 0) of the list at +0x30 (safe iteration, link offset +0x38) to the
/// list at +0xc (+0x3c counts them); FUN_0011a770(this, 0); then repeatedly FUN_0011a770(this, n) for the node
/// with the smallest value (+0x10) below -0.01 (link +0x20) until none
pub fn ee_sub_11abf8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let it = stack_push(m, 0x10);
    let off = m.r32(o + 0x38);
    let mut c = m.r32(o + 0x30);
    m.w32(it, o + 0x30);
    m.w32(it + 4, c.wrapping_add(off));
    m.w32(it + 8, off);
    let prev = m.r32(ITER_CHAIN);
    m.w32(it + 0xc, prev);
    m.w32(ITER_CHAIN, it);
    while c != 0 {
        if m.r32(c) == 1 {
            m.call(0x0011_a2e8, &[o + 0x30, c], &[]);
            m.call(0x0011_a220, &[o + 0xc, c], &[]);
            let n = m.r32(o + 0x3c);
            m.w32(o + 0x3c, n.wrapping_add(1));
        }
        let nx = m.r32(it + 4);
        c = m.r32(nx);
        let off = m.r32(it + 8);
        m.w32(it + 4, c.wrapping_add(off));
    }
    m.call(0x0011_a770, &[o, 0], &[]);
    let prev = m.r32(it + 0xc);
    m.w32(ITER_CHAIN, prev);
    stack_pop(m, 0x10);
    loop {
        let mut best = 0u32;
        let mut thr = -0.01f32;
        let mut n = m.r32(o + 0x30);
        while n != 0 {
            let v = m.rf(n + 0x10);
            if v < thr {
                thr = v;
                best = n;
            }
            n = m.r32(n + 0x20);
        }
        if best == 0 {
            break;
        }
        m.call(0x0011_a770, &[o, best], &[]);
    }
}
/// FUN_0011ae88 / FUN_0011b050 / FUN_0011b1a8 (identical bodies): with (p1 != 0, p2 == 0xffff) reference-count the
/// six pools (counters DAT_002dd2e0..f4) and create each one on its first reference with FUN_0011cca8
pub fn pools_init<M: QMem + ?Sized>(m: &mut M, p1: u64, p2: u64) {
    if p2 == 0xffff && p1 != 0 {
        for (ctr, a0, a1, a2) in [
            (0x002d_d2e0, 0x0045_20a8, 0x14, 0x2000),
            (0x002d_d2e4, 0x0045_20d0, 0x18, 0x800),
            (0x002d_d2e8, 0x0045_20f8, 0x28, 0x200),
            (0x002d_d2ec, 0x0045_2120, 0x1c, 0x800),
            (0x002d_d2f0, 0x0045_2148, 0xc, 200),
            (0x002d_d2f4, 0x0045_2170, 0x14, 0x100),
        ] {
            let c = m.r32(ctr).wrapping_add(1);
            m.w32(ctr, c);
            if c == 1 {
                m.call(0x0011_cca8, &[a0, a1, a2, 0x10], &[]);
            }
        }
    }
}
pub fn ee_sub_11ae88<M: QMem + ?Sized>(m: &mut M, p1: u64, p2: u64) {
    pools_init(m, p1, p2)
}
pub fn ee_sub_11b050<M: QMem + ?Sized>(m: &mut M, p1: u64, p2: u64) {
    pools_init(m, p1, p2)
}
pub fn ee_sub_11b1a8<M: QMem + ?Sized>(m: &mut M, p1: u64, p2: u64) {
    pools_init(m, p1, p2)
}
/// FUN_0011afc0: FUN_0011ae88(1, 0xffff)
pub fn ee_sub_11afc0<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_11ae88(m, 1, 0xffff)
}
/// FUN_0011b188: FUN_0011b050(1, 0xffff)
pub fn ee_sub_11b188<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_11b050(m, 1, 0xffff)
}
/// FUN_0011b2e0: FUN_0011b1a8(1, 0xffff)
pub fn ee_sub_11b2e0<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_11b1a8(m, 1, 0xffff)
}
/// FUN_0011afe0: string hash h = h * 0x21 + c, lower case letters (ctype table 0x2aa6a1 bit 1) folded by -0x20;
/// null -> the empty default string at 0x2a4150
pub fn ee_sub_11afe0<M: QMem + ?Sized>(m: &mut M, s: u32) -> u32 {
    let mut p = if s == 0 { 0x002a_4150 } else { s };
    let mut h: u32 = 0;
    let mut b2 = m.r8(p);
    while b2 != 0 {
        let b = b2;
        p += 1;
        b2 = m.r8(p);
        let mut u = (b as u32).wrapping_sub(0x20);
        if m.r8(0x002a_a6a1 + b as u32) & 2 == 0 {
            u = b as u32;
        }
        h = h.wrapping_mul(0x21).wrapping_add(u);
    }
    h
}
/// FUN_0011b318: FUN_0011c500(4, 3, p1, p2)
pub fn ee_sub_11b318<M: QMem + ?Sized>(m: &mut M, p1: u32, p2: u32) {
    m.call(0x0011_c500, &[4, 3, p1, p2], &[]);
}
/// FUN_0011b628: euler angles (x, y, z of the quadword, FUN_0011d608 sin / cos each) -> matrix words
/// 0 1 2 / 4 5 6 / 8 9 10 of `out`; returns out
pub fn ee_sub_11b628<M: QMem + ?Sized>(m: &mut M, out: u32, ang: V4) -> u32 {
    let t = stack_push(m, 0x20);
    let mut sc = [0.0f32; 6];
    for i in 0..3u32 {
        m.call(0x0011_d608, &[t + 8 * i, t + 8 * i + 4], &[s(ang[i as usize])]);
        sc[2 * i as usize] = m.rf(t + 8 * i);
        sc[2 * i as usize + 1] = m.rf(t + 8 * i + 4);
    }
    stack_pop(m, 0x20);
    let [s1, c1, s2, c2, s3, c3] = sc;
    m.wf(out + 5 * 4, s1 * s2 * s3 + c1 * c3);
    m.wf(out + 4 * 4, s1 * s2 * c3 - c1 * s3);
    m.wf(out + 8 * 4, c1 * s2 * c3 + s1 * s3);
    m.wf(out + 9 * 4, c1 * s2 * s3 - s1 * c3);
    m.wf(out + 10 * 4, c1 * c2);
    m.wf(out + 2 * 4, -s2);
    m.wf(out, c2 * c3);
    m.wf(out + 4, c2 * s3);
    m.wf(out + 6 * 4, s1 * c2);
    out
}
/// FUN_0011b720: matrix = rotation(-p2 about DAT_002a2db0) composed with rotation(p1 about DAT_002a2dc0), each
/// built by FUN_0011b500; rows r_i = A0 * B_i.x + A1 * B_i.y + A2 * B_i.z; written to `out`; returns out
pub fn ee_sub_11b720<M: QMem + ?Sized>(m: &mut M, a1: f32, a2: f32, out: u32) -> u32 {
    let ma = stack_push(m, 0x30);
    let mb = stack_push(m, 0x30);
    m.call(0x0011_b500, &[ma], &[rov(0x002a_2dc0), s(a1)]);
    m.call(0x0011_b500, &[mb], &[rov(0x002a_2db0), s(-a2)]);
    let a = [m.rv(ma), m.rv(ma + 0x10), m.rv(ma + 0x20)];
    for i in 0..3u32 {
        let r = m.rv(mb + 0x10 * i);
        let v: V4 = std::array::from_fn(|k| a[0][k] * r[0] + a[1][k] * r[1] + a[2][k] * r[2]);
        m.wv(out + 0x10 * i, v);
    }
    stack_pop(m, 0x60);
    out
}
/// FUN_00120c48: p3[1] owner; p5 == 0x23 -> FUN_00129658(owner + 8, p3 + 8); else FUN_00256c58(owner + 0x10,
/// p3[2] == 0x1f ? p3[3] : 0); FUN_0012a0b0(a0, 0x24); returns a0
pub fn ee_sub_120c48<M: QMem + ?Sized>(m: &mut M, a0: u32, _a1: u32, p3: u32, _a3: u32, p5: u32) -> u32 {
    let ow = m.r32(p3 + 4);
    if p5 == 0x23 {
        m.call(0x0012_9658, &[ow + 8, p3 + 8], &[]);
    } else if m.r32(p3 + 8) == 0x1f {
        let v = m.r32(p3 + 0xc);
        m.call(0x0025_6c58, &[ow + 0x10, v], &[]);
    } else {
        m.call(0x0025_6c58, &[ow + 0x10, 0], &[]);
    }
    m.call(0x0012_a0b0, &[a0, 0x24], &[]);
    a0
}
