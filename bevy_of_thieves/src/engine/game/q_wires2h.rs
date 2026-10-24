//! WIRE-S2: 1:1 QMem ports of live-frontier natives (shard 3/4).
#![allow(unused)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

// SKIP 0x1a93c8: VU0 macro heavy (vmadd/vsqrt/vdiv, MMI madd, 0x3760-byte stack frame of quad buffers).
// SKIP 0x21e1b0: soft-float double formatter; a3 and callee args/returns are 64-bit doubles (not representable as u32).

/// FUN_0018ee38: pick facing/anim state index for object.
pub fn ee_sub_18ee38<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let s1 = a0;
    let t = m.r32(s1 + 0x10dc);
    if t != 0 {
        let p = m.call(0x001a_62e8, &[t], &[]).1;
        let q = m.call(0x001a_62e8, &[s1], &[]).1;
        let d = [p[0] - q[0], p[1] - q[1], p[2] - q[2], p[3] - q[3]];
        let len = (d[0] * d[0] + d[1] * d[1]).sqrt();
        let mut ok = false;
        if len < m.rf(0x002b_f4ac) {
            ok = m.call(0x0011_d910, &[0x002b_f4b0], &[f4(d[2])]).0 != 0;
        }
        if ok {
            let t = m.r32(s1 + 0x10dc);
            let r = m.call(0x001e_eee0, &[t.wrapping_add(0x26c), 0xb1], &[]).0;
            if r != 0 && m.r32(r + 0x5c) == s1 {
                let st = m.r32(r + 0x50) as i32;
                if st < 2 && st >= 0 {
                    m.call(0x0018_c250, &[s1, 0x10, 0xffff_ffff], &[]);
                    return 0xffff_ffff;
                }
            }
        }
    }
    let mut s0: u32;
    if m.r32(s1 + 0x18d0) == 5 {
        let a = vslot(m, s1, 30, &[], &[]).0;
        let b = vslot(m, s1, 30, &[], &[]).0;
        let (fa, fb) = (m.rf(a + 4), m.rf(b));
        let ang = m.call(0x0024_58f0, &[], &[f4(fa), f4(fb)]).1[0];
        let h = m.r32(s1 + 0x150);
        let x = m.rf(h + 0x18) - ang;
        let x = m.call(0x0011_cf60, &[], &[f4(x)]).1[0];
        let ax = x.abs();
        if ax < f32::from_bits(0x3f49_0fdb) {
            s0 = 0xe;
        } else if f32::from_bits(0x4016_cbe4) < ax {
            s0 = 0x10;
        } else if 0.0 < x {
            s0 = 0x12;
        } else {
            s0 = 0x14;
        }
    } else {
        s0 = if m.r32(s1 + 0x18c8) == 0 { 0xc } else { 0xe };
    }
    let k = m.r32(s1 + 0x18d8).wrapping_sub(0xc);
    if k < 10 {
        // jump table 0x2a6010: even -> +1, odd -> unchanged
        if k & 1 == 0 {
            s0 = s0.wrapping_add(1);
        }
    } else if m.call(0x0018_ed98, &[s1], &[]).0 != 0 {
        s0 = s0.wrapping_add(1);
    }
    s0
}

/// pool lookup: (offset/20, pool index) of `p` among `*n_addr` pools at `tbl`, else (0, 0).
fn find_blk<M: QMem + ?Sized>(m: &mut M, n_addr: u32, tbl: u32, span: u32, p: u32) -> (u32, u32) {
    let n = m.r32(n_addr);
    let mut i = 0u32;
    while i < n {
        let b = m.r32(tbl + i.wrapping_mul(4));
        if !(p < b) && p < b.wrapping_add(span) {
            let v = (p.wrapping_sub(b) as i32 as i64 * 0xcccc_cccd_u32 as i32 as i64) as i32;
            return ((v >> 2) as u32, i);
        }
        i += 1;
    }
    (0, 0)
}

fn link_pack<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, n_addr: u32, tbl: u32, span: u32) {
    let (lo, hi) = find_blk(m, n_addr, tbl, span, a0);
    let h = m.r32(a1);
    if h != 0 {
        let mut v = m.r32(h + 4) & 0xffff_e000 | (lo & 0x1fff);
        m.w32(h + 4, v);
        v = v & 0xffff_9fff | ((hi & 3) << 13);
        m.w32(h + 4, v);
    }
    let (lo2, hi2) = find_blk(m, n_addr, tbl, span, h);
    let v = m.r32(a0 + 4) & 0xe000_ffff | ((lo2 & 0x1fff) << 16);
    let v = v & 0x9fff_ffff | ((hi2 & 3) << 29);
    m.w32(a0 + 4, v);
    m.w32(a1, a0);
}

/// FUN_00254f00: push node a0 onto packed list head *a1 (pool table 0x452098, span 0x28000).
pub fn ee_sub_254f00<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    link_pack(m, a0, a1, 0x002d_d284, 0x0045_2098, 0x28000);
}

/// FUN_00255f38: push node a0 onto packed list head *a1 (pool table 0x452160, span 0x1400).
pub fn ee_sub_255f38<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    link_pack(m, a0, a1, 0x002d_d2d4, 0x0045_2160, 0x1400);
}

fn copy4<M: QMem + ?Sized>(m: &mut M, dst: u32, src: u32) {
    for k in 0..4 {
        let b = m.r8(src + k);
        m.w8(dst + k, b);
    }
}

/// FUN_00215730: bind a0 into a1 (+0x30); type 4 -> lookup + FUN_00217ca8; type 6 -> resolve +0x14.
pub fn ee_sub_215730<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, t0: u32) {
    m.w32(a1 + 0x30, a0);
    let ty = m.r32(a0 + 4);
    if ty == 4 {
        let idx = m.r32(a0 + 0x10).wrapping_mul(0x30);
        let b8 = m.r32(a0 + 8); let base = m.r32(b8 + 0x84);
        let v = m.call(0x0021_7ef8, &[base.wrapping_add(idx)], &[]).0;
        m.w32(a1 + 0x58, v);
        let sp = stack_push(m, 0x20);
        copy4(m, sp, a3);
        copy4(m, sp + 0x10, t0);
        let idx = m.r32(a0 + 0x10).wrapping_mul(0x30);
        let b8 = m.r32(a0 + 8); let base = m.r32(b8 + 0x84);
        m.call(0x0021_7ca8, &[base.wrapping_add(idx), 0x314, v, a2, sp, sp + 0x10], &[]);
        stack_pop(m, 0x20);
    } else if ty == 6 {
        let k = m.r32(a0 + 0x10);
        if k != 0xffff_ffff {
            let v = m.call(0x001b_eed8, &[k], &[]).0;
            if v != 0 {
                let r = m.call(0x001b_eb28, &[v], &[]).0;
                m.w32(a0 + 0x14, r);
            }
        }
    }
}

/// FUN_0021eac8: printf core: copy literal runs via FUN_0020d660, expand %-specs; returns out +0x10.
pub fn ee_sub_21eac8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) -> u32 {
    let s3 = a0;
    let mut s1 = a1;
    let sp = stack_push(m, 0x50);
    let s4 = sp + 0x40;
    m.w32(s4, a2);
    if m.r8(s1) != 0 {
        loop {
            let mut s2 = m.call(0x0021_edf8, &[s1, 0x25], &[]).0;
            let mut s0 = s2.wrapping_sub(s1);
            if s2 == 0 {
                s0 = m.call(0x0021_ed28, &[s1], &[]).0;
                s2 = s1.wrapping_add(s0);
            }
            if s0 != 0 {
                let v = m.call(0x0020_d660, &[s3, s1, s0], &[]).0;
                s1 = s2;
                if (v as i32) < (s0 as i32) {
                    break;
                }
            }
            let mut c = m.r8(s1);
            if c == 0x25 {
                let n = m.call(0x0021_d6b0, &[s1, sp, s4], &[]).0;
                m.call(0x0021_e718, &[sp, s3, s4, s1, n], &[]);
                s1 = s1.wrapping_add(n);
                c = m.r8(s1);
            }
            if c == 0 {
                break;
            }
        }
    }
    stack_pop(m, 0x50);
    m.r32(s3 + 0x10)
}

/// FUN_001a5078: if a1 live (+4 != +0x3c), alloc FUN_001a4fb0(a0, 2) and fill +8=a1, +0xc=a2, +0x10=a3.
pub fn ee_sub_1a5078<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32) -> u32 {
    if a1 == 0 || m.r32(a1 + 4) == m.r32(a1 + 0x3c) {
        return 0;
    }
    let r = m.call(0x001a_4fb0, &[a0, 2], &[]).0;
    m.w32(r + 0x10, a3);
    m.w32(r + 8, a1);
    m.w32(r + 0xc, a2);
    r
}

/// FUN_001be1e8: drain list at 0x3e1a4c: while !FUN_0011a3b0(head) { FUN_0011a2e8(head, x); FUN_001bd040(x) }.
pub fn ee_sub_1be1e8<M: QMem + ?Sized>(m: &mut M) {
    let h = 0x003e_1a4c;
    while m.call(0x0011_a3b0, &[h], &[]).0 == 0 {
        let x = m.r32(h);
        m.call(0x0011_a2e8, &[h, x], &[]);
        m.call(0x001b_d040, &[x], &[]);
    }
}

/// FUN_00240aa0: r = FUN_00234d40(a0); FUN_00234d60(a0); r.
pub fn ee_sub_240aa0<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let r = m.call(0x0023_4d40, &[a0], &[]).0;
    m.call(0x0023_4d60, &[a0], &[]);
    r
}
