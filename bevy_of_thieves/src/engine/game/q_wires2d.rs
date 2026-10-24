//! WIRE-S2: 1:1 QMem ports of live-frontier natives (shard 3/4).
#![allow(unused)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

const CM: u32 = 0x002d_df40; // camera manager

/// FUN_001a3d00(o, e): if e+0x118 < 0, allocate the lowest free bit of the 8-word mask o+0x4590 as e's slot
/// (o+0x4154[slot] = e); then unless o+0x4554 / e+0x15c, either FUN_001a3b80(o, o+0x455c, pos(e), slot, e+0x168)
/// (e+0xe0 && !e+0x160 && o+0x455c) or set bit slot in o+0x4570.
pub fn ee_sub_1a3d00<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let s1 = a0;
    let s0 = a1;
    if (m.r32(s0 + 0x118) as i32) < 0 {
        let mut found = false;
        for i in 0..8u32 {
            let t2 = s1 + 0x4590 + i.wrapping_mul(4);
            let t1 = m.r32(t2);
            if t1 == 0 {
                continue;
            }
            let low = t1 & t1.wrapping_neg();
            let idx = i.wrapping_mul(32) + low.trailing_zeros();
            let w = m.r32(t2) & !low;
            m.w32(t2, w);
            m.w32(s1.wrapping_add(idx << 2) + 0x4154, s0);
            m.w32(s0 + 0x118, idx);
            found = true;
            break;
        }
        if !found {
            return;
        }
    }
    if m.r32(s1 + 0x4554) != 0 || m.r32(s0 + 0x15c) != 0 {
        return;
    }
    if m.r32(s0 + 0xe0) != 0 && m.r32(s0 + 0x160) == 0 && m.r32(s1 + 0x455c) != 0 {
        let q = m.call(0x001a_62e8, &[s0], &[]).1;
        let a1 = m.r32(s1 + 0x455c);
        let a3 = m.r32(s0 + 0x118);
        let f12 = m.rf(s0 + 0x168);
        m.call(0x001a_3b80, &[s1, a1, a3], &[q, f4(f12)]);
        return;
    }
    let a0 = m.r32(s0 + 0x118) as i32;
    let word = (if a0 > -1 { a0 } else { a0.wrapping_add(0x1f) }) >> 5;
    let addr = (s1 + 0x4570).wrapping_add((word << 2) as u32);
    let v = m.r32(addr) | (1u32 << (a0 as u32 & 31));
    m.w32(addr, v);
}

/// pool lookup: index i of the block table `tab` (count at `cnt`) whose [base, base+span) holds x;
/// returns ((x - base) * magic as i32 >> sh, i) or (0, 0).
fn pool_find<M: QMem + ?Sized>(m: &mut M, x: u32, cnt: u32, tab: u32, span: u32, magic: u32, sh: u32) -> (u32, u32) {
    let n = m.r32(cnt);
    let mut i = 0u32;
    while i < n {
        let b = m.r32(tab + i.wrapping_mul(4));
        if x >= b && x < b.wrapping_add(span) {
            return (((x.wrapping_sub(b).wrapping_mul(magic) as i32) >> sh) as u32, i);
        }
        i += 1;
    }
    (0, 0)
}

fn pool_link<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, cnt: u32, tab: u32, span: u32, magic: u32, sh: u32) {
    let (r0, i0) = pool_find(m, a0, cnt, tab, span, magic, sh);
    let a2 = m.r32(a1);
    if a2 != 0 {
        let v = (m.r32(a2 + 4) & 0xffff_e000) | (r0 & 0x1fff);
        m.w32(a2 + 4, v);
        let v = (v & 0xffff_9fff) | ((i0 & 3) << 13);
        m.w32(a2 + 4, v);
    }
    let x = m.r32(a1);
    let (r1, i1) = pool_find(m, x, cnt, tab, span, magic, sh);
    let v = (m.r32(a0 + 4) & 0xe000_ffff) | ((r1 & 0x1fff) << 16);
    let v = (v & 0x9fff_ffff) | ((i1 & 3) << 29);
    m.w32(a0 + 4, v);
    m.w32(a1, a0);
}

/// FUN_00255588(node, head): push node (40-byte pool @0x4520e8, count 0x2dd2a4) onto the packed
/// (index:13, block:2) linked list *head.
pub fn ee_sub_255588<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    pool_link(m, a0, a1, 0x002d_d2a4, 0x0045_20e8, 0x5000, 0xcccc_cccd, 3);
}

/// FUN_00255c00(node, head): as FUN_00255588 for the 12-byte pool @0x452138 (count 0x2dd2c4).
pub fn ee_sub_255c00<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    pool_link(m, a0, a1, 0x002d_d2c4, 0x0045_2138, 0x960, 0xaaaa_aaab, 2);
}

/// FUN_0012e858(a0): o = a0+0x10; o+0xe0 -> FUN_0014e1a8(CM, pos(o), pos(o+0xe0), 0, 0); else build a matrix
/// (-row2, -row0, row1, pos(o)) from FUN_001a6318(o) and FUN_0014e168(CM, &mat). Then o+0x12c > 0 ->
/// FUN_0014f5a0(CM, clamp(2 * FUN_00244b68(17.9959 / o+0x128), 0.01, 3)).
pub fn ee_sub_12e858<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let s2 = m.r32(a0 + 0x10);
    if m.r32(s2 + 0xe0) != 0 {
        let q1 = m.call(0x001a_62e8, &[s2], &[]).1;
        let k = m.r32(s2 + 0xe0);
        let q2 = m.call(0x001a_62e8, &[k], &[]).1;
        m.call(0x0014_e1a8, &[CM], &[q1, q2, f4(0.0), f4(0.0)]);
    } else {
        let sp = stack_push(m, 0x100);
        let p = m.call(0x001a_6318, &[s2], &[]).0;
        let r0 = m.rv(p);
        let r1 = m.rv(p + 16);
        let r2 = m.rv(p + 32);
        let neg = |v: V4| [-v[0], -v[1], -v[2], -v[3]];
        m.wv(sp, r0);
        m.wv(sp + 0x10, r1);
        m.wv(sp + 0x20, r2);
        m.wv(sp + 0x30, neg(r2));
        m.wv(sp + 0x40, neg(r0));
        m.wv(sp + 0x50, r1);
        m.wv(sp + 0xa0, f4(-1.0));
        let q = m.call(0x001a_62e8, &[s2], &[]).1;
        m.call(0x0025_6d18, &[sp + 0x90], &[q]);
        let s0 = sp + 0x60;
        let v = m.rv(sp + 0x30);
        m.wv(s0, v);
        let v = m.rv(sp + 0x40);
        m.wv(s0 + 0x10, v);
        let v = m.rv(sp + 0x50);
        m.wv(s0 + 0x20, v);
        m.w32(sp + 0x8c, 0);
        m.w32(sp + 0x7c, 0);
        m.w32(sp + 0x6c, 0);
        m.wf(s0 + 0x3c, 1.0);
        m.call(0x0014_e168, &[CM, s0], &[]);
        stack_pop(m, 0x100);
    }
    if (m.r32(s2 + 0x12c) as i32) <= 0 {
        return;
    }
    let f12 = f32::from_bits(0x418f_f79b) / m.rf(s2 + 0x128);
    let f0 = m.call(0x0024_4b68, &[], &[f4(f12)]).1[0];
    let mut f12 = f0 + f0;
    let lo = f32::from_bits(0x3c23_d70a);
    if f12 < lo {
        f12 = lo;
    } else if 3.0 < f12 {
        f12 = 3.0;
    }
    m.call(0x0014_f5a0, &[CM], &[f4(f12)]);
}

/// FUN_001997d8(o, pos (128 a1), dir (128 a2)): build a frame (FUN_0011e2d8 + FUN_00256d18), FUN_001995d8(o, &mat,
/// 0x16, &out); for each linked object (o[0]+0xec+0x64 list, skipping flag 4) and each of 11 out pairs, ray-test
/// FUN_0014c6a0 against its collision list; any hit in the first 2 pairs or with zero hit-t -> 0; else count
/// hits with normal.y > 0.7; returns count >= 2.
pub fn ee_sub_1997d8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: V4, a2: V4) -> u32 {
    let sp = stack_push(m, 0x2f0);
    let r = (|| {
        let qa = m.rv(0x002a_2dc0);
        let qb = m.rv(0x002a_2db0);
        m.wv(sp, a2);
        m.call(0x0011_e2d8, &[sp + 0x10], &[a2, qa, qb]);
        let s0 = sp + 0x30;
        m.call(0x0025_6d18, &[sp + 0x60], &[a1]);
        let v = m.rv(sp);
        m.wv(sp + 0x30, v);
        let v = m.rv(sp + 0x10);
        m.wv(s0 + 0x10, v);
        let v = m.rv(sp + 0x20);
        m.wv(s0 + 0x20, v);
        m.w32(sp + 0x5c, 0);
        m.w32(sp + 0x4c, 0);
        m.w32(sp + 0x3c, 0);
        m.wf(s0 + 0x3c, 1.0);
        m.call(0x0019_95d8, &[a0, s0, 0x16, sp + 0x70], &[]);
        let mut fp: i32 = 0;
        let k = m.r32(a0);
        let k = m.r32(k + 0xec);
        let mut s4 = m.r32(k + 0x64);
        while s4 != 0 {
            let k = m.r32(s4 + 0xc);
            let k = m.r32(k + 0xec);
            if m.r64(k + 0x70) & 4 == 0 {
                let mut s2 = 0u32;
                let mut s3 = sp + 0x70;
                while s2 < 0x16 {
                    let k = m.r32(s4 + 0xc);
                    let k = m.r32(k + 0xec);
                    let mut s0 = m.r32(k + 0x34);
                    if s0 != 0 {
                        let s1 = sp + 0x80 + (s2 << 4);
                        let s5 = (s2 as i32) < 4;
                        while s0 != 0 {
                            if m.r64(s0 + 0x10) & (1u64 << 48) == 0 {
                                let va = m.rv(s3);
                                let vb = m.rv(s1);
                                let hit = m.call(0x0014_c6a0, &[s0, 1, sp + 0x1d0], &[va, vb]).0;
                                if (hit as i32) > 0 {
                                    if s5 {
                                        return 0;
                                    }
                                    if m.rf(sp + 0x210) == 0.0 {
                                        return 0;
                                    }
                                    if f32::from_bits(0x3f33_3333) < m.rf(sp + 0x1f8) {
                                        fp += 1;
                                    }
                                }
                            }
                            s0 = m.r32(s0 + 0x60);
                        }
                    }
                    s2 += 2;
                    s3 += 0x20;
                }
            }
            s4 = m.r32(s4 + 4);
        }
        (fp >= 2) as u32
    })();
    stack_pop(m, 0x2f0);
    r
}

/// FUN_001a7c18(o, a1, a2, t1, t2, t3, *list (stack), pos (128 a3), dir (128 t0), len): alloc a node
/// (FUN_0011ccc0(o+0x3d40)) holding pos / pos+(0,0,0,1)-dir*len / dir and two endpoints (a1, a2) with
/// FUN_0022fc48 data; fill params, notify endpoints (vslot 54), FUN_001a7fc0(o, 1, &node), append the list
/// to the node's +0xac chain and make the node the list head.
pub fn ee_sub_1a7c18<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, t1: u32, t2: u32, t3: u32, list: u32, a3: V4, t0: V4, f12: f32) {
    let sp = stack_push(m, 0x20);
    let f20 = f12;
    let v = m.call(0x0011_ccc0, &[a0 + 0x3d40], &[]).0;
    m.w32(sp + 0x10, v);
    m.wv(v + 0x10, a3);
    let nf = -f20;
    let e = [a3[0] + t0[0] * nf, a3[1] + t0[1] * nf, a3[2] + t0[2] * nf, a3[3] + 1.0 + t0[3] * nf];
    m.wv(v + 0x50, e);
    m.wv(v + 0x80, t0);
    for s1 in 0..2u32 {
        let base = m.r32(sp + 0x10);
        let s0 = base.wrapping_add(s1 << 6);
        m.w32(s0, if s1 == 0 { a1 } else { a2 });
        let v1 = m.r32(s0);
        if v1 != 0 {
            let x = m.r32(v1);
            m.w32(s0 + 4, x);
            let o = m.r32(v1 + 4);
            m.w32(s0 + 0xc, o);
            let k = m.r32(o + 0xec);
            let x = m.r32(k + 0x60);
            m.w32(s0 + 8, x);
        }
        let k = m.r32(s0 + 4);
        let q = m.rv(s0 + 0x10);
        let r = m.call(0x0022_fc48, &[0, k], &[q]).1;
        m.wv(s0 + 0x20, r);
    }
    let f1 = m.rf(0x002b_fd30);
    let f0 = m.rf(0x002b_fd34);
    let v = m.r32(sp + 0x10);
    m.w32(v + 0xa4, t3);
    m.wf(v + 0x90, f20);
    m.w32(v + 0x9c, t1);
    m.wf(v + 0x94, f1);
    m.wf(v + 0x98, f0);
    m.w32(v + 0xa0, t2);
    for s1 in 0..2u32 {
        let base = m.r32(sp + 0x10);
        let s0 = base.wrapping_add(s1 << 6);
        let o = m.r32(s0 + 0xc);
        if o != 0 {
            vslot(m, o, 54, &[base, s1], &[]);
        }
    }
    m.call(0x001a_7fc0, &[a0, 1, sp + 0x10], &[]);
    let mut v1 = m.r32(sp + 0x10);
    while m.r32(v1 + 0xac) != 0 {
        v1 = m.r32(v1 + 0xac);
    }
    let x = m.r32(list);
    m.w32(v1 + 0xac, x);
    let n = m.r32(sp + 0x10);
    m.w32(list, n);
    stack_pop(m, 0x20);
}

/// FUN_00130918(o): |o+0x40| >= 1e-4 -> vslot 0; o+0x1c += o+0x40 * rate (flag 0x100 picks DAT_002ddf10 /
/// DAT_002dded4); FUN_00130f88 / FUN_00131968 / FUN_001317d8 step logic with vslot 3(3, o); then FUN_00130b18,
/// notify o+0x38 listeners (their vtable +0x14 at word 0) with (+4, 0, o+0x1c, o+0x40); vslot 1.
pub fn ee_sub_130918<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let s1 = a0;
    if m.rf(s1 + 0x40).abs() < f32::from_bits(0x38d1_b717) {
        return;
    }
    vslot(m, s1, 0, &[], &[]);
    let k = m.r32(s1 + 0x14);
    let mut a = m.r32(k + 0x40);
    if (a ^ 1) & 1 != 0 {
        a = m.r32(a);
    }
    let f2 = if a & 0x100 != 0 { m.rf(0x002d_df10) } else { m.rf(0x002d_ded4) };
    let f1 = m.rf(s1 + 0x40) * f2;
    let f0 = m.rf(s1 + 0x1c) + f1;
    m.wf(s1 + 0x1c, f0);
    m.call(0x0013_0f88, &[s1], &[]);
    if m.call(0x0013_1968, &[s1], &[]).0 != 0 {
        vslot(m, s1, 1, &[], &[]);
        return;
    }
    let s0 = m.call(0x0013_17d8, &[s1, s1 + 0x1c, s1 + 0x40], &[]).0;
    if s0 != 0 {
        if m.r32(s1 + 0x3c) & 8 != 0 {
            let v = if m.rf(s1 + 0x40) <= 0.0 {
                let k = m.r32(s1 + 0x14);
                m.r32(k + 0xa0)
            } else {
                0
            };
            m.w32(s1 + 0x50, v);
        }
        vslot(m, s1, 3, &[3, s1], &[]);
        if m.call(0x0013_1968, &[s1], &[]).0 != 0 {
            vslot(m, s1, 1, &[], &[]);
            return;
        }
    }
    m.call(0x0013_0f88, &[s1], &[]);
    if m.call(0x0013_1968, &[s1], &[]).0 != 0 {
        vslot(m, s1, 1, &[], &[]);
        return;
    }
    if s0 != 0 && m.r32(s1 + 0x3c) & 0x38 == 0 {
        m.call(0x0013_0b18, &[s1], &[]);
    }
    let mut s0 = m.r32(s1 + 0x38);
    while s0 != 0 {
        let a3 = m.r32(s0);
        let a1 = m.r32(s0 + 4);
        let vt = m.r32(a3);
        let f12 = m.rf(s1 + 0x1c);
        let adj = m.r16(vt + 0x10) as i16 as i32;
        let f = m.r32(vt + 0x14);
        let f13 = m.rf(s1 + 0x40);
        m.call(f, &[a3.wrapping_add(adj as u32), a1, 0], &[f4(f12), f4(f13)]);
        s0 = m.r32(s0 + 0x10);
    }
    vslot(m, s1, 1, &[], &[]);
}

/// FUN_001380f0(o, src, n, out, *total): weighted blend over o's channel table (count +0xc4, stride +0xc0,
/// base +0xc8: weight @0, clip @4 with length +0x74). n == 0: *total = sum w*len. Else per out slot i
/// accumulate out[i].y = sum w * src[i] and total; out[0].x = FUN_0011d898(wrapped phase average, 1) * total;
/// out[i].x = out[i-1].x + out[i-1].y wrapped by total; *total = total.
pub fn ee_sub_1380f0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, t0: u32) {
    let s0 = a2;
    let s1 = a3;
    let s2 = t0;
    let mut f20: f32 = 0.0;
    if s0 == 0 {
        if s2 == 0 {
            return;
        }
        let c4 = m.r32(a0 + 0xc4);
        if (c4 as i32) > 0 {
            let mut v1 = m.r32(a0 + 0xc8);
            let stride = m.r32(a0 + 0xc0);
            let mut n = c4;
            loop {
                let f1 = m.rf(v1);
                if f1 != 0.0 {
                    let k = m.r32(v1 + 4);
                    f20 += m.rf(k + 0x74) * f1;
                }
                n = n.wrapping_sub(1);
                v1 = v1.wrapping_add(stride);
                if n == 0 {
                    break;
                }
            }
        }
        m.wf(s2, f20);
        return;
    }
    if (s0 as i32) > 0 {
        let mut i = 0u32;
        while (i as i32) < (s0 as i32) {
            let t1 = i << 3;
            let v1 = t1.wrapping_add(s1);
            m.w32(v1 + 4, 0);
            if (m.r32(a0 + 0xc4) as i32) > 0 {
                let f3 = m.rf(v1 + 4);
                let mut j = 0u32;
                loop {
                    let c0 = m.r32(a0 + 0xc0);
                    let c8 = m.r32(a0 + 0xc8);
                    let off = j.wrapping_mul(c0);
                    let f2 = m.rf(c8.wrapping_add(off));
                    if f2 != f3 {
                        let p = t1.wrapping_add(off.wrapping_add(a1));
                        let f1 = m.rf(v1 + 4) + m.rf(p + 4) * f2;
                        m.wf(v1 + 4, f1);
                    }
                    j += 1;
                    if (j as i32) >= (m.r32(a0 + 0xc4) as i32) {
                        break;
                    }
                }
            }
            f20 += m.rf(t1.wrapping_add(s1) + 4);
            i += 1;
        }
    }
    // f12 is only defined once a non-zero channel is seen (asm leaves the incoming register otherwise)
    let mut f12: f32 = 0.0;
    let c4 = m.r32(a0 + 0xc4);
    if (c4 as i32) > 0 {
        let mut v1 = m.r32(a0 + 0xc8);
        let stride = m.r32(a0 + 0xc0);
        let mut pa = v1.wrapping_add(a1);
        let mut first = true;
        let mut f4v: f32 = 0.0;
        let mut n = c4;
        loop {
            let w = m.rf(v1);
            if w != 0.0 {
                let k = m.r32(v1 + 4);
                if first {
                    first = false;
                    f12 = m.rf(pa) / m.rf(k + 0x74);
                    f4v = f12;
                } else {
                    let mut f1 = m.rf(pa) / m.rf(k + 0x74) - f4v;
                    if f1 <= -0.5 {
                        f1 += 1.0;
                    } else if 0.5 < f1 {
                        f1 -= 1.0;
                    }
                    f12 += f1 * m.rf(v1);
                }
            }
            pa = pa.wrapping_add(stride);
            n = n.wrapping_sub(1);
            v1 = v1.wrapping_add(stride);
            if n == 0 {
                break;
            }
        }
    }
    let f0 = m.call(0x0011_d898, &[], &[f4(f12), f4(1.0)]).1[0];
    m.wf(s1, f0 * f20);
    let mut i: i32 = 1;
    while i < s0 as i32 {
        let a3 = s1.wrapping_add((i as u32) << 3);
        let f0 = m.rf(a3 - 8) + m.rf(a3 - 4);
        m.wf(a3, f0);
        if f20 <= f0 {
            m.wf(a3, f0 - f20);
        }
        i += 1;
    }
    if s2 != 0 {
        m.wf(s2, f20);
    }
}
