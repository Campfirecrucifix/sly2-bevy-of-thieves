//! WP-Q4: run 0x001f10f0..0x001f2ba8 - animation chain timing (FUN_001f10f0 / 1f11a8: +0x24 graph, +0x30
//! current segment, +0x34 / +0x40 / +0x48), the guard's pending action queue (0x10-byte records
//! {+0 id, +4, +8 arg, +0xc} at +0x404, count +0x484, cursor +0x488, done flag +0x48c, frozen +0x490,
//! owner +0x494, state +0x3fc / +0x400 with the two alarm lights +0x3f4 / +0x3f8), the bounding sphere
//! records of ALO hierarchies (+0x134, pool 0x3e12d4: {+0 owner, +4 centre, +8 refs, +0x10 / +0x1c
//! radii, +0x20}) and the SO part lists (+0xec merged body {+0x34 part list, +0x60 root}, parts +0xe4 x
//! 0xb0, count +0xe0).
//! 1:1 against `QMem` (classes::q::q2mem); callees outside the run go through `QMem::call` by address.
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};
use crate::engine::game::q_001e9090::ee_sub_1ea410;

/// DAT_002dded0: game clock
pub const NOW: u32 = 0x002d_ded0;
/// bounding sphere record pool (FUN_0011ccc0 / 11cd98)
pub const SPHERE_POOL: u32 = 0x003e_12d4;
/// world
pub const WORLD: u32 = 0x003d_d490;

fn f(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

/// FUN_001f10f0(t, a): unless FUN_001f0638(a): slot 0, then advance the segment chain (+0x30) by t:
/// segments finished within t are retracted (FUN_00130b18) and t reduced by their remaining time
/// (+0x14 +0x74 - +0x1c); the one that is left gets FUN_00130d30(seg, 1, t, 1.0); slot 1
pub fn ee_sub_1f10f0<M: QMem + ?Sized>(m: &mut M, t: f32, a: u32) {
    if m.call(0x001f_0638, &[a], &[]).0 != 0 {
        return;
    }
    vslot(m, a, 0, &[], &[]);
    let mut t = t;
    loop {
        let s = m.r32(a + 0x30);
        if s == 0 {
            break;
        }
        let seg = m.r32(s + 0x14);
        let rem = m.rf(seg + 0x74) - m.rf(s + 0x1c);
        if t < rem {
            m.call(0x0013_0d30, &[s, 1], &[f(t), f(1.0)]);
            break;
        }
        t -= rem;
        m.call(0x0013_0b18, &[s], &[]);
    }
    vslot(m, a, 1, &[], &[]);
}

/// FUN_001f11a8(a, target): time to reach graph node FUN_001f0438(+0x24, target): the current segment's
/// remaining time (/ its rate +0x40; 0 for rate 0; skipped when +0x40 +0xc bit 1) then each edge of
/// FUN_001f0b80(+0x24, +0x48, from, to, 1): length +0x74 / (+0x78 * clamp(c0 + r (c1 + r c2), +0x90,
/// +0x94)), r = FUN_00154c90(0x2de578); 0 while FUN_001f0638
pub fn ee_sub_1f11a8<M: QMem + ?Sized>(m: &mut M, a: u32, target: u32) -> f32 {
    if m.call(0x001f_0638, &[a], &[]).0 != 0 {
        return 0.0;
    }
    let g = m.r32(a + 0x24);
    let to = m.call(0x001f_0438, &[g, target], &[]).0;
    let mut acc = 0.0f32;
    let s = m.r32(a + 0x30);
    let mut node;
    if s == 0 {
        node = m.r32(a + 0x34);
    } else {
        let e = m.r32(a + 0x40);
        if m.r32(e + 0xc) & 2 == 0 {
            let rate = m.rf(s + 0x40);
            if rate < 0.0 {
                acc = -m.rf(s + 0x1c);
            } else {
                if rate <= 0.0 {
                    return 0.0;
                }
                let seg = m.r32(s + 0x14);
                acc = m.rf(seg + 0x74) - m.rf(s + 0x1c);
            }
            acc /= rate;
        }
        node = m.r32(e + 4);
        if node == m.r32(a + 0x34) {
            node = m.r32(e + 8);
        }
    }
    while node != to {
        let (g, h) = (m.r32(a + 0x24), m.r32(a + 0x48));
        let ed = m.call(0x001f_0b80, &[g, h, node, to, 1], &[]).0;
        if ed == 0 {
            break;
        }
        let c = m.r32(ed);
        if c != 0 {
            let r = m.call(0x0015_4c90, &[0x002d_e578], &[]).1[0];
            let v = m.rf(c + 0x80) + r * (m.rf(c + 0x84) + r * m.rf(c + 0x88));
            let (lo, hi) = (m.rf(c + 0x90), m.rf(c + 0x94));
            let k = if v < lo {
                lo
            } else if hi < v {
                hi
            } else {
                v
            };
            acc += m.rf(c + 0x74) / (m.rf(c + 0x78) * k);
        }
        let nx = m.r32(ed + 4);
        node = if nx == node { m.r32(ed + 8) } else { nx };
    }
    acc
}

/// FUN_001f1610 / 1f1fb0 (identical static pool inits) = FUN_001ea410
pub fn ee_sub_1f1610<M: QMem + ?Sized>(m: &mut M, init: u32, prio: u32) {
    ee_sub_1ea410(m, init, prio);
}
/// FUN_001f1748 / 1f20e8: (1, 0xffff)
pub fn ee_sub_1f1748<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_1ea410(m, 1, 0xffff);
}

fn push_rec<M: QMem + ?Sized>(m: &mut M, g: u32, id: u32, arg3: u32) {
    let n = m.r32(g + 0x484);
    m.w32(g + 0x484, n.wrapping_add(1));
    let r = g + n.wrapping_mul(0x10) + 0x404;
    m.w32(r, id);
    m.w32(r + 4, 0);
    m.w32(r + 8, u32::MAX);
    m.w32(r + 0xc, arg3);
}

/// FUN_001f1768(g, v): frozen (+0x490) -> queue {0x2b3, 0, -1, v}
pub fn ee_sub_1f1768<M: QMem + ?Sized>(m: &mut M, g: u32, v: u32) {
    if m.r32(g + 0x490) != 0 {
        push_rec(m, g, 0x2b3, v);
    }
}

/// FUN_001f17b0(g, id): empty queue or frozen -> queue {id, 0, -1, 1}
pub fn ee_sub_1f17b0<M: QMem + ?Sized>(m: &mut M, g: u32, id: u32) {
    if (m.r32(g + 0x484) as i32) < 1 || m.r32(g + 0x490) != 0 {
        push_rec(m, g, id, 1);
    }
}

/// FUN_001f17f8(g, s): state +0x3fc changes -> +0x400 = now; s -1 / 0 slot 9 of the lights +0x3f4 /
/// +0x3f8, 1 / 2 slot 8; then the owner's (+0x494) sound +0x16c gets DAT_002c3738[s + 1] (FUN_001f0730)
pub fn ee_sub_1f17f8<M: QMem + ?Sized>(m: &mut M, g: u32, s: u32) {
    if s == m.r32(g + 0x3fc) {
        return;
    }
    m.w32(g + 0x3fc, s);
    let t = m.r32(NOW);
    m.w32(g + 0x400, t);
    let si = s as i32;
    if (-1..=2).contains(&si) {
        let slot = if si < 1 { 9 } else { 8 };
        for off in [0x3f4, 0x3f8] {
            let l = m.r32(g + off);
            if l != 0 {
                vslot(m, l, slot, &[], &[]);
            }
        }
    }
    let o = m.r32(g + 0x494);
    let h = m.r32(o + 0x16c);
    if h != 0 {
        let st = m.r32(g + 0x3fc);
        let v = m.r32(st.wrapping_add(1).wrapping_mul(4).wrapping_add(0x002c_3738));
        m.call(0x001f_0730, &[h, v], &[]);
    }
}

/// FUN_001f1900(g): light state from the owner (+0x494): -1 without its +0x158; 0 while the guard's
/// +0x2c0 meets FUN_00207a20(0x1d / 0x20 / 5 / 6 / 7 / 0x17 / 0x16); else 2 when the last queued
/// action is not 0x2b3, FUN_001c4840(world, 0) and FUN_001c4840(world, 1) == DAT_002e1e40; else 1
pub fn ee_sub_1f1900<M: QMem + ?Sized>(m: &mut M, g: u32) {
    let o = m.r32(g + 0x494);
    if o == 0 {
        return;
    }
    let mut s = u32::MAX;
    if m.r32(o + 0x158) != 0 {
        let mut mask = 0u64;
        for k in [0x1d, 0x20, 5, 6, 7, 0x17, 0x16] {
            m.call(0x0020_7a20, &[k, u32::MAX], &[]);
            // FUN_00207a20(k) = (1L << k) (0 when k == -1): a 64-bit mask. k == 0x20 sets bit 32, which lives in
            // the high return word the (u32, V4) call ABI does not expose, so reconstruct the full u64 locally
            // from the function's exact semantics. [HARD2]
            mask |= 1u64 << k;
        }
        s = 0;
        if m.r64(g + 0x2c0) & mask == 0 {
            s = 1;
            let n = m.r32(g + 0x484);
            if n != 0
                && m.r32(g + n.wrapping_sub(1).wrapping_mul(0x10) + 0x404) != 0x2b3
                && m.call(0x001c_4840, &[WORLD, 0], &[]).0 != 0
                && m.call(0x001c_4840, &[WORLD, 1], &[]).0 == m.r32(0x002e_1e40)
            {
                s = 2;
            }
        }
    }
    ee_sub_1f17f8(m, g, s);
}

/// FUN_001f1a40(g): FUN_001c2db8(owner); unless frozen: refill the queue from the alert table
/// (DAT_002c3968 for class 0x3a else DAT_002c3748, 0x3c-byte rows indexed by FUN_00210ae0 (-1 -> 1)):
/// a queued head whose target (FUN_0015e938(world, 4, id, owner)) is no busy kind-0x5c object is kept as
/// the trailing action; else with probability row +8 a weighted pick ({weight, value} x 6 at +0xc,
/// FUN_0011d3f8) becomes {0x2b5, 0, value, 1}. Then rand(row +0, row +4) seconds are spread over
/// rand(2, 3) waits {0x2b3, 0, -, share}, and the trailing action appended when its +0xc > 0.
pub fn ee_sub_1f1a40<M: QMem + ?Sized>(m: &mut M, g: u32) {
    let o = m.r32(g + 0x494);
    m.call(0x001c_2db8, &[o], &[]);
    if m.r32(g + 0x490) != 0 {
        return;
    }
    let v = m.call(0x0021_0ae0, &[], &[]).0;
    let idx = if v == u32::MAX { 1 } else { v };
    let base = if m.call(0x0013_2b10, &[g, 0x3a], &[]).0 != 0 { 0x002c_3968 } else { 0x002c_3748 };
    let row = idx.wrapping_mul(0x3c).wrapping_add(base);
    let sp = stack_push(m, 0x10);
    m.call(0x0023_33b4, &[sp, 0, 0x10], &[]);
    let mut kept = false;
    if 0 < m.r32(g + 0x484) as i32 {
        let (id, o) = (m.r32(g + 0x404), m.r32(g + 0x494));
        let t = m.call(0x0015_e938, &[WORLD, 4, id, o], &[]).0;
        if t != 0 && m.call(0x0013_2b10, &[t, 0x5c], &[]).0 != 0 && m.r32(t + 0x230) != 0 {
            let b = m.r32(t + 0x230);
            if m.call(0x0016_4030, &[b], &[]).0 != 0 {
                m.w32(g + 0x484, 0);
            }
        }
        if 0 < m.r32(g + 0x484) as i32 {
            for k in 0..4 {
                let w = m.r32(g + 0x404 + 4 * k);
                m.w32(sp + 4 * k, w);
            }
            kept = true;
        }
    }
    if !kept {
        let r = m.call(0x0011_d448, &[], &[f(0.0), f(1.0)]).1[0];
        if m.rf(row + 8) < r {
            m.w32(sp, u32::MAX);
            m.w32(sp + 8, u32::MAX);
        } else {
            let tot: u32 = (0..6).map(|i| m.r32(row + 0xc + 8 * i)).fold(0u32, |a, b| a.wrapping_add(b));
            let mut x = m.call(0x0011_d3f8, &[0, tot.wrapping_sub(1)], &[]).0 as i32;
            for i in 0..6 {
                let e = row + 0xc + 8 * i;
                let w = m.r32(e) as i32;
                if x < w {
                    let v = m.r32(e + 4);
                    m.w32(sp + 8, v);
                    break;
                }
                x = x.wrapping_sub(w);
            }
            m.w32(sp + 0xc, 1);
            m.w32(sp, 0x2b5);
        }
    }
    m.w32(g + 0x484, 0);
    let (lo, hi) = (m.r32(row), m.r32(row + 4));
    let mut total = m.call(0x0011_d3f8, &[lo, hi], &[]).0 as i32;
    let k = m.call(0x0011_d3f8, &[2, 3], &[]).0 as i32;
    let mut j = 0;
    while j < k {
        let n = m.r32(g + 0x484);
        m.w32(g + 0x484, n + 1);
        let r = g + n * 0x10 + 0x404;
        m.w32(r, 0x2b3);
        m.w32(r + 4, 0);
        let d = total / (k - j);
        let v = if 0 < d { d } else { 1 };
        total -= v;
        m.w32(r + 0xc, v as u32);
        if total == 0 {
            break;
        }
        j += 1;
    }
    if 0 < m.r32(sp + 0xc) as i32 {
        let n = m.r32(g + 0x484);
        for kk in 0..4 {
            let w = m.r32(sp + 4 * kk);
            m.w32(g + n * 0x10 + 0x404 + 4 * kk, w);
        }
        m.w32(g + 0x484, n + 1);
    }
    stack_pop(m, 0x10);
}

/// FUN_001f1cf8(g): run the queue from +0x488: the first entry with a live target (FUN_0015e938) is
/// started (FUN_001c2dc8(owner, target, +8, 0x2b0 / 0x2b1 for kind-0x5c targets whose +0x230 +0x54 is 1;
/// +0x54 -1 / 0 -> done) and +0x48c = done; then FUN_001f1900
pub fn ee_sub_1f1cf8<M: QMem + ?Sized>(m: &mut M, g: u32) {
    m.w32(g + 0x48c, 0);
    let mut i = m.r32(g + 0x488) as i32;
    while i < m.r32(g + 0x484) as i32 {
        let r = g + i as u32 * 0x10 + 0x404;
        let (id, o) = (m.r32(r), m.r32(g + 0x494));
        let t = m.call(0x0015_e938, &[WORLD, 4, id, o], &[]).0;
        if t != 0 {
            let arg = m.r32(r + 8);
            let mut kind = 0x2b0;
            let mut go = true;
            if m.call(0x0013_2b10, &[t, 0x5c], &[]).0 != 0 {
                let b = m.r32(t + 0x230);
                if b != 0 {
                    let st = m.r32(b + 0x54);
                    if st.wrapping_add(1) < 2 {
                        m.w32(g + 0x48c, 1);
                        break;
                    }
                    if st != 1 {
                        go = false;
                    }
                }
                kind = 0x2b1;
            }
            if go {
                let o = m.r32(g + 0x494);
                if m.call(0x001c_2dc8, &[o, t, arg, kind], &[]).0 == 1 {
                    m.w32(g + 0x48c, 1);
                }
                break;
            }
        }
        i = m.r32(g + 0x488) as i32 + 1;
        m.w32(g + 0x488, i as u32);
    }
    ee_sub_1f1900(m, g);
}

/// FUN_001f1e20(g): next queued action: FUN_001619a0(0x2deb30, 0x19, g), r = FUN_001c2ef0(owner,
/// entry +0xc), state -1, cursor + 1, FUN_001f1cf8; 0 when the queue is done
pub fn ee_sub_1f1e20<M: QMem + ?Sized>(m: &mut M, g: u32) -> u32 {
    if !((m.r32(g + 0x488) as i32) < m.r32(g + 0x484) as i32) {
        return 0;
    }
    m.call(0x0016_19a0, &[0x002d_eb30, 0x19, g], &[]);
    let (o, c) = (m.r32(g + 0x494), m.r32(g + 0x488));
    let v = m.r32(c.wrapping_mul(0x10).wrapping_add(g) + 0x410);
    let r = m.call(0x001c_2ef0, &[o, v], &[]).0;
    ee_sub_1f17f8(m, g, u32::MAX);
    let c = m.r32(g + 0x488);
    m.w32(g + 0x488, c.wrapping_add(1));
    ee_sub_1f1cf8(m, g);
    r
}

/// FUN_001f1eb8(ret, g): the pending ids (cursor .. count, up to 8) -> FUN_00200f58(ret, n, 6, ids)
pub fn ee_sub_1f1eb8<M: QMem + ?Sized>(m: &mut M, ret: u32, g: u32) -> u32 {
    let buf = stack_push(m, 0x20);
    let mut n = 0u32;
    let c = m.r32(g + 0x488) as i32;
    let cnt = m.r32(g + 0x484) as i32;
    for i in c..cnt.max(c) {
        let v = m.r32(i as u32 * 0x10 + 0x404 + g);
        m.w32(buf + 4 * n, v);
        n += 1;
    }
    m.call(0x0020_0f58, &[ret, n, 6, buf], &[]);
    stack_pop(m, 0x20);
    ret
}

fn for_kind30_children<M: QMem + ?Sized>(m: &mut M, o: u32, f: fn(&mut M, u32)) {
    let mut c = m.r32(o + 0x64);
    while c != m.r32(o + 0x70) {
        if m.call(0x0013_2b10, &[c, 0x30], &[]).0 != 0 {
            f(m, c);
        }
        c = m.r32(c + 0x24);
    }
}

/// FUN_001f2468(o): release the bounding sphere +0x134 (refs - 1, freed to 0x3e12d4 at 0) and recurse
/// into the class-0x30 children (+0x64 .. +0x70, next +0x24)
pub fn ee_sub_1f2468<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let s = m.r32(o + 0x134);
    if s != 0 {
        let n = m.r32(s + 8).wrapping_sub(1);
        m.w32(s + 8, n);
        if n == 0 {
            m.call(0x0011_cd98, &[SPHERE_POOL, s], &[]);
        }
        m.w32(o + 0x134, 0);
    }
    for_kind30_children(m, o, ee_sub_1f2468);
}

/// FUN_001f2500(o): with sphere bits (+0xd0 bits 32 / 33): find / create the sphere record shared up the
/// parent chain (+0x10) to the first kind-1 node (+0x148 bits 20..23 of the low word) or an ancestor
/// that has one (refs + 1); a new one (pool 0x3e12d4) sits at that node: owner o, centre = node +0x60
/// +0x70, refs 1 (+1 more when it is not o itself), +0x20 = o +0x13c. Then the radius |pos - centre| +
/// o +0xa0 grows +0x10 (bit 32) / +0x1c (bit 33)
pub fn ee_sub_1f2500<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let bits = m.r64(o + 0xd0);
    if bits & 0x3_0000_0000 == 0 {
        return;
    }
    let s = m.r32(o + 0x134);
    'found: {
        if s != 0 {
            let n = m.r32(s + 8);
            m.w32(s + 8, n.wrapping_add(1));
            break 'found;
        }
        let mut node = o;
        let mut par = m.r32(o + 0x10);
        while par != 0 && (((m.r32(node + 0x148) << 8) as i32) >> 0x1c) != 1 {
            let ps = m.r32(par + 0x134);
            if ps != 0 {
                m.w32(o + 0x134, ps);
                let n = m.r32(ps + 8);
                m.w32(ps + 8, n.wrapping_add(1));
                break 'found;
            }
            node = par;
            par = m.r32(par + 0x10);
        }
        let r = m.call(0x0011_ccc0, &[SPHERE_POOL], &[]).0;
        m.w32(node + 0x134, r);
        m.w32(o + 0x134, r);
        let c = m.r32(node + 0x60).wrapping_add(0x70);
        m.w32(r + 4, c);
        let n = m.r32(r + 8);
        m.w32(r + 8, n.wrapping_add(1));
        let v = m.r32(o + 0x13c);
        m.w32(r + 0x20, v);
        m.w32(r, o);
        if node != o {
            let n = m.r32(r + 8);
            m.w32(r + 8, n.wrapping_add(1));
        }
    }
    let p = m.call(0x001a_62e8, &[o], &[]).1;
    let s = m.r32(o + 0x134);
    let cp = m.r32(s + 4);
    let c = m.rv(cp);
    let d = [p[0] - c[0], p[1] - c[1], p[2] - c[2]];
    let r = m.rf(o + 0xa0) + (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    if bits & 0x1_0000_0000 != 0 && m.rf(s + 0x10) < r {
        m.wf(s + 0x10, r);
    }
    if m.r64(o + 0xd0) & 0x2_0000_0000 != 0 {
        let s = m.r32(o + 0x134);
        if m.rf(s + 0x1c) < r {
            m.wf(s + 0x1c, r);
        }
    }
}

/// FUN_001f26a8(o): FUN_001f2500(o) over o and its class-0x30 children, recursively
pub fn ee_sub_1f26a8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    ee_sub_1f2500(m, o);
    for_kind30_children(m, o, ee_sub_1f26a8);
}

/// FUN_001f2798(o): walk up (+0x10) to the first node with a merge flag (+0x148 bit 34) or a physics
/// record +0xe8 not in state 3 / 3 (+0xc0 / +0xa0); the topmost node when none
pub fn ee_sub_1f2798<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let mut prev = 0;
    let mut n = o;
    loop {
        if n == 0 {
            return prev;
        }
        let e = m.r32(n + 0xe8);
        if e != 0 && (m.r32(e + 0xc0) != 3 || m.r32(e + 0xa0) != 3) {
            return n;
        }
        if m.r64(n + 0x148) & 0x4_0000_0000 != 0 {
            return n;
        }
        prev = n;
        n = m.r32(n + 0x10);
    }
}

/// FUN_001f27f0(o): merged body +0xec from the merge root (FUN_001f2798): created by its slot 52 and
/// registered (FUN_00134320(world, root)) when missing
pub fn ee_sub_1f27f0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.r32(o + 0xec) != 0 {
        return;
    }
    let r = ee_sub_1f2798(m, o);
    if m.r32(r + 0xec) == 0 {
        let b = vslot(m, r, 52, &[], &[]).0;
        m.w32(r + 0xec, b);
        m.call(0x0013_4320, &[WORLD, r], &[]);
    }
    let b = m.r32(r + 0xec);
    m.w32(o + 0xec, b);
}

/// FUN_001f2870(o, part): add a part to the merged body list (+0xec +0x34) once: its +0x20..+0x60 = the
/// part's source (+0) +0x40..+0x80, +0xa8 = -1, +0xac = source +0xa4; FUN_001f4bf8(o), FUN_001f38a8(part),
/// FUN_001f5078(o) when slot 61 is 0
pub fn ee_sub_1f2870<M: QMem + ?Sized>(m: &mut M, o: u32, part: u32) {
    ee_sub_1f27f0(m, o);
    let l = m.r32(o + 0xec) + 0x34;
    if m.call(0x0011_a380, &[l, part], &[]).0 != 0 {
        return;
    }
    m.call(0x0011_a220, &[l, part], &[]);
    let src = m.r32(part);
    m.w32(part + 0xa8, u32::MAX);
    for k in 0..16 {
        let v = m.r32(src + 0x40 + 4 * k);
        m.w32(part + 0x20 + 4 * k, v);
    }
    let v = m.r32(src + 0xa4);
    m.w32(part + 0xac, v);
    m.call(0x001f_4bf8, &[o], &[]);
    m.call(0x001f_38a8, &[part], &[]);
    if vslot(m, o, 61, &[], &[]).0 == 0 {
        m.call(0x001f_5078, &[o], &[]);
    }
}

/// FUN_001f2930(o, part): remove a listed part from the merged body: unlink, FUN_001a00d0(world, 0, part)
/// for parts with bit 51 of +0x10, FUN_001f4e28(o); an empty list (FUN_0011a3b0) detaches root and o
/// (FUN_001337d0(world)), else o keeps the body only while it is the root or still has a listed part;
/// FUN_001f5078(o) when slot 61 is 0
pub fn ee_sub_1f2930<M: QMem + ?Sized>(m: &mut M, o: u32, part: u32) {
    let b = m.r32(o + 0xec);
    if b == 0 || m.call(0x0011_a380, &[b + 0x34, part], &[]).0 == 0 {
        return;
    }
    m.call(0x0011_a2e8, &[b + 0x34, part], &[]);
    if m.r64(part + 0x10) & 0x8_0000_0000_0000 != 0 {
        m.call(0x001a_00d0, &[WORLD, 0, part], &[]);
    }
    m.call(0x001f_4e28, &[o], &[]);
    let b = m.r32(o + 0xec);
    if m.call(0x0011_a3b0, &[b + 0x34], &[]).0 == 0 {
        if m.r32(b + 0x60) != o {
            let mut i = m.r32(o + 0xe0) as i32;
            loop {
                i -= 1;
                if i < 0 {
                    m.w32(o + 0xec, 0);
                    break;
                }
                let b = m.r32(o + 0xec);
                let p = m.r32(o + 0xe4).wrapping_add(i as u32 * 0xb0);
                if m.call(0x0011_a380, &[b + 0x34, p], &[]).0 != 0 {
                    break;
                }
            }
        }
    } else {
        let root = m.r32(b + 0x60);
        m.w32(root + 0xec, 0);
        m.w32(o + 0xec, 0);
        m.call(0x0013_37d0, &[WORLD], &[]);
    }
    if vslot(m, o, 61, &[], &[]).0 == 0 {
        m.call(0x001f_5078, &[o], &[]);
    }
}

/// FUN_001f2ba8(o): FUN_001f2930(o, part) for every part without bit 50 of +0x10
pub fn ee_sub_1f2ba8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let mut i = 0;
    while i < m.r32(o + 0xe0) as i32 {
        let p = m.r32(o + 0xe4).wrapping_add(i as u32 * 0xb0);
        if m.r64(p + 0x10) & 0x4_0000_0000_0000 == 0 {
            ee_sub_1f2930(m, o, p);
        }
        i += 1;
    }
}
