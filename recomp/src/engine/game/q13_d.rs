//! WP-Q13 fourth part: runs 001fbbc8 .. 00256328 (2D-object ctor/registry, bone tables, misc wrappers).
use super::q13_b::vcl;
use super::q13_c::ee_sub_1fafd0;
use super::q_00114680::*;
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem, V4};

const POOL_CREATE: u32 = 0x0011_cca8;

/// FUN_001fbbc8
pub fn ee_sub_1fbbc8<M: QMem + ?Sized>(m: &mut M, p1: u32, p2: u32) -> u32 {
    ee_sub_1fafd0(m, p1);
    m.w32(p1 + 0x60, p2);
    m.w32(p1 + 0xc, 0x002d_9c70);
    m.call(0x001f_b658, &[p1, 1, 1], &[]);
    p1
}
/// FUN_001fc0f0
pub fn ee_sub_1fc0f0<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x0011_ccc0, &[0x003d_d05c], &[]);
}
/// FUN_001fc138
pub fn ee_sub_1fc138<M: QMem + ?Sized>(m: &mut M, p1: u32, p2: u32) -> u32 {
    if p1 == 0 {
        return 0;
    }
    let u = m.call(0x001f_ccf0, &[0x3c0], &[]).0;
    ee_sub_1fc190(m, u, p1, p2)
}
/// FUN_001fc190
pub fn ee_sub_1fc190<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32, p3: u32) -> u32 {
    ee_sub_1fafd0(m, o);
    m.w32(o + 0x60, p2);
    m.w32(o + 0xc, 0x002d_9be0);
    m.w32(o + 0x3b8, p3);
    for k in 0..6 {
        m.w32(o + 0x70 + 4 * k, 0x3f80_0000);
    }
    m.call(0x001f_b658, &[o, 0x801, 0x801], &[]);
    let k = m.r32(o + 0x60);
    let i = m.call(0x001f_d080, &[0x003d_d040, k], &[]).0;
    if m.rf(0x003d_d19c + i.wrapping_mul(4)) == 0.0 {
        m.call(0x001f_b018, &[o], &[]);
    }
    if m.r32(0x003d_d098) != 0 {
        m.call(0x001f_b018, &[o], &[]);
    }
    o
}
/// FUN_001fc380
pub fn ee_sub_1fc380<M: QMem + ?Sized>(m: &mut M, o: u32, idx: u64, ang: i32) {
    if idx < 6 {
        let mut a = ang % 0x168;
        if a < 0 {
            a += 0x168;
        }
        let p = o + 0x88 + (idx as u32) * 4;
        if a as u32 != m.r32(p) {
            let v = m.r32(o + 0x3bc) | 1 << (idx as u32 & 0x1f);
            m.w32(o + 0x3bc, v);
            m.w32(p, a as u32);
            m.call(0x001f_b658, &[o, 0x100, 0x100], &[]);
        }
    }
}
/// FUN_001fe478: remove (and return through smart pointer `out`) the entry of the table at +0x208 (count
/// +0x204) whose +0x60 equals FUN_001fcff8(o, key)
pub fn ee_sub_1fe478<M: QMem + ?Sized>(m: &mut M, out: u32, o: u32, key: u32) -> u32 {
    let want = m.call(0x001f_cff8, &[o, key], &[]).0;
    let cnt = m.r32(o + 0x204) as i32;
    let at = |m: &mut M, i: i32| m.r32(o + 0x208 + 4 * i as u32);
    let mut i = 0i32;
    if cnt > 0 {
        let e = at(m, 0);
        if m.r32(e + 0x60) != want {
            i = 1;
            while i < m.r32(o + 0x204) as i32 {
                let e = at(m, i);
                if m.r32(e + 0x60) == want {
                    break;
                }
                i += 1;
            }
        }
    }
    if i == m.r32(o + 0x204) as i32 {
        m.call(0x0025_9bb0, &[out, 0], &[]);
    } else {
        let sp = stack_push(m, 0x10);
        let e = at(m, i);
        m.call(0x0025_9bb0, &[sp, e], &[]);
        let n = m.r32(o + 0x204) as i32 - 1;
        m.w32(o + 0x204, n as u32);
        let mut j = i;
        while j < n {
            let nx = at(m, j + 1);
            m.call(0x0025_9b48, &[o + 0x208 + 4 * j as u32, nx], &[]);
            j += 1;
        }
        let t = m.r32(sp);
        m.call(0x0025_9bb0, &[out, t], &[]);
        if t != 0 {
            vcl(m, t, 0x10, 0x14, &[], &[]);
        }
        stack_pop(m, 0x10);
    }
    out
}
/// FUN_001ff108
pub fn ee_sub_1ff108<M: QMem + ?Sized>(m: &mut M, p1: u32, p4: i32) {
    let u = m.call(0x001f_e990, &[0x003d_d040], &[]).0;
    let x = m.r32(0x003d_d268);
    m.call(0x001f_c2e0, &[p1, x, u], &[]);
    ee_sub_1fc380(m, x, u as u64, p4);
}
/// FUN_001ff188: p2 == 0xffff: p1 == 0 destroy the 0x3dd040 singleton and its smart-pointer slots,
/// else construct it and create the six pools on first reference
pub fn ee_sub_1ff188<M: QMem + ?Sized>(m: &mut M, p1: u64, p2: u64) {
    if p2 != 0xffff {
        return;
    }
    if p1 == 0 {
        let mut p = 0x003d_d268u32;
        let o = m.r32(p);
        if o != 0 {
            vcl(m, o, 0x10, 0x14, &[], &[]);
        }
        loop {
            p -= 4;
            let o = m.r32(p);
            if o != 0 {
                vcl(m, o, 0x10, 0x14, &[], &[]);
            }
            if p == 0x003d_d248 {
                break;
            }
        }
        let o = m.r32(0x003d_d240);
        if o != 0 {
            vcl(m, o, 0x10, 0x14, &[], &[]);
        }
        m.call(0x0025_8348, &[0x003d_d040, 2], &[]);
    } else {
        m.call(0x001f_cd38, &[0x003d_d040], &[]);
        for (c, a, sz, n) in [
            (0x002d_d2e0u32, 0x0045_20a8u32, 0x14u32, 0x2000u32),
            (0x002d_d2e4, 0x0045_20d0, 0x18, 0x800),
            (0x002d_d2e8, 0x0045_20f8, 0x28, 0x200),
            (0x002d_d2ec, 0x0045_2120, 0x1c, 0x800),
            (0x002d_d2f0, 0x0045_2148, 0xc, 200),
            (0x002d_d2f4, 0x0045_2170, 0x14, 0x100),
        ] {
            let v = m.r32(c).wrapping_add(1);
            m.w32(c, v);
            if v == 1 {
                m.call(POOL_CREATE, &[a, sz, n, 0x10], &[]);
            }
        }
    }
}
/// FUN_001ff378
pub fn ee_sub_1ff378<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_1ff188(m, 1, 0xffff)
}
/// FUN_001ff398
pub fn ee_sub_1ff398<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_1ff188(m, 0, 0xffff)
}
/// FUN_001ff4f0: out = position(a0) - [p2 + 0x240]
pub fn ee_sub_1ff4f0<M: QMem + ?Sized>(m: &mut M, a0: u32, p2: u32, out: u32) {
    if out != 0 {
        let p = m.call(0x001a_62e8, &[a0], &[]).1;
        let c = m.rv(p2 + 0x240);
        m.wv(out, [p[0] - c[0], p[1] - c[1], p[2] - c[2], p[3] - c[3]]);
    }
}
/// FUN_001ff670
pub fn ee_sub_1ff670<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x001f_f538, &[1, 0xffff], &[]);
}
/// FUN_001ff7c8
pub fn ee_sub_1ff7c8<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x001f_f690, &[1, 0xffff], &[]);
}
/// FUN_001ff7e8
pub fn ee_sub_1ff7e8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.w32(o + 0xe85 * 4, 1);
    let u = m.call(0x0025_6dd0, &[], &[]).0;
    m.call(0x0025_6ee0, &[o, u], &[]);
    let k = m.r32(o);
    m.call(0x0025_9a88, &[k], &[]);
}
/// FUN_001ff828
pub fn ee_sub_1ff828<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.w32(o + 0x3a14, 0);
}
/// FUN_001ff830
pub fn ee_sub_1ff830<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0025_6ee0, &[o, 0], &[]);
    ee_sub_12ae28(m);
}
/// FUN_001ffcd8
pub fn ee_sub_1ffcd8<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x001f_fba0, &[1, 0xffff], &[]);
}

/// FUN_001cd5d0
pub fn ee_sub_1cd5d0<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x001c_d498, &[1, 0xffff], &[]);
}
/// FUN_00204da8: clamp the new scale to >= 0.1, rescale the block (+0xe8) velocity-like quad (+0x80) by
/// new/old (+0x260) and store the new scale
pub fn ee_sub_204da8<M: QMem + ?Sized>(m: &mut M, p1: f32, o: u32) {
    let f = if p1 < 0.1 { f32::from_bits(0x3dcc_cccd) } else { p1 };
    let b = m.r32(o + 0xe8);
    let old = m.rf(o + 0x260);
    let a = m.rf(b + 0xd0) * old / f;
    super::q13_c::ee_sub_1f35c8(m, a, o);
    let b = m.r32(o + 0xe8);
    let v = m.rv(b + 0x80);
    let q = 1.0 / old;
    m.wv(b + 0x80, [v[0] * f * q, v[1] * f * q, v[2] * f * q, v[3] * f * q]);
    m.wf(o + 0x260, f);
}
/// FUN_00204ed8: query virtual slot (+0x1c of the entry vtable) of the type-6 child of `p3`; returns the
/// low 64 bits of the (default DAT_002a2dd0) result vector
pub fn ee_sub_204ed8<M: QMem + ?Sized>(m: &mut M, p1: u32, p2: u32, p3: u32) -> u64 {
    let sp = stack_push(m, 0x40);
    let loc = sp;
    let w = m.r32(0x002a_2dd0);
    m.w32(loc, w);
    let w = m.r32(0x002a_2dd4);
    m.w32(loc + 4, w);
    let w = m.r32(0x002a_2dd8);
    m.w32(loc + 8, w);
    let w = m.r32(0x002a_2ddc);
    m.w32(loc + 0xc, w);
    if p3 != 0 && m.call(0x0013_1968, &[p3], &[]).0 == 0 {
        let k = m.r32(p3 + 0x14);
        let l = m.call(0x0012_ff10, &[k, p2, 6, 1], &[]).0;
        if l != 0 {
            let p = m.r32(l + 4);
            let vt = m.r32(p);
            let adj = m.r16(vt + 0x18) as i16 as i32 as u32;
            let f = m.r32(vt + 0x1c);
            let a = m.r32(p3 + 0x40);
            m.call(f, &[p1, a, p.wrapping_add(adj), 0, loc], &[]);
        }
    }
    let r = m.r64(loc);
    stack_pop(m, 0x40);
    r
}
/// FUN_00204f78
pub fn ee_sub_204f78<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32, p3: u64) {
    let mut i2 = m.call(0x001a_5bf0, &[p2], &[]).0;
    while i2 != 0 && m.r32(i2 + 0xec) == 0 {
        i2 = m.r32(i2 + 0x10);
    }
    let i1 = m.r32(o + 0xec);
    if i1 != 0 && i2 != 0 && m.r32(i2 + 0xec) != 0 {
        let t = m.r32(i2 + 0xec);
        m.call(0x0013_3a38, &[0x003d_d490, i1, t, if p3 == 0 { 2 } else { 1 }], &[]);
    }
    m.call(0x001f_4bf8, &[o], &[]);
}
/// FUN_002137f0
pub fn ee_sub_2137f0<M: QMem + ?Sized>(m: &mut M, p1: u32) -> u32 {
    let mut n = m.r32(0x003e_1060);
    if n == 0 {
        return 0;
    }
    let mut i1 = m.r32(n + 0xa4) as i32;
    while i1 <= m.r32(p1 + 0xa4) as i32 {
        n = m.r32(n + 0xb4);
        if n == 0 {
            return 0;
        }
        i1 = m.r32(n + 0xa4) as i32;
    }
    1
}
/// FUN_00213830
pub fn ee_sub_213830<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32) {
    let a = m.r32(o + 0x84);
    m.call(0x0021_7ba0, &[a], &[]);
    vcl(m, o, 0xa8, 0xac, &[1], &[]);
    if m.r32(o + 0xac) != 0 {
        m.call(0x0021_4300, &[o, 0], &[]);
    }
    m.w32(o + 0x7c, p2);
}
/// FUN_00213898
pub fn ee_sub_213898<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let a = m.r32(o + 0x84);
    m.call(0x0021_8638, &[a], &[]);
    if m.r32(o + 0xac) != 0 {
        m.call(0x0021_4300, &[o, 0xffff_ffff], &[]);
    }
    vcl(m, o, 0xa8, 0xac, &[0], &[]);
    vcl(m, o, 0x18, 0x1c, &[6, 0, 0, 0, 0], &[]);
    m.w32(o + 0x88, 0);
    m.w32(o + 0x7c, 0xffff_ffff);
}
/// Find the (group, entry) whose entry key (+0x14 of 0x18-byte entries at group+4) equals `key`; groups are
/// 0x30 bytes at +0x84 (count u16 at +0x80). Returns (group index, group addr, entry index).
fn find_entry<M: QMem + ?Sized>(m: &mut M, p: u32, key: u32) -> Option<(u32, u32, u32)> {
    let ng = m.r16(p + 0x80) as u32;
    for gi in 0..ng {
        let g = m.r32(p + 0x84).wrapping_add(gi * 0x30);
        let ne = m.r16(g) as u32;
        for ei in 0..ne {
            let e = m.r32(g + 4).wrapping_add(ei * 0x18);
            if key == m.r32(e + 0x14) {
                return Some((gi, g, ei));
            }
        }
    }
    None
}
fn any_active<M: QMem + ?Sized>(m: &mut M, g: u32) -> bool {
    let n = m.r16(g + 2) as u32;
    let base = m.r32(g + 8);
    (0..n).any(|k| m.r16(base + 8 + k * 0x18) as i16 != -1)
}
/// FUN_00213928
pub fn ee_sub_213928<M: QMem + ?Sized>(m: &mut M, p: u32, key: u32) {
    if let Some((gi, g, _)) = find_entry(m, p, key) {
        if !any_active(m, g) {
            m.w16(g + 0x20, 0xffff);
            m.w16(g + 0x22, 0xffff);
        } else {
            m.call(0x0021_3f70, &[p, gi, 0, 0, 0], &[]);
        }
    }
}
/// FUN_00213a08: every matching entry (the scan continues after a hit)
pub fn ee_sub_213a08<M: QMem + ?Sized>(m: &mut M, p: u32, key: u32, p3: u32) {
    let ng = m.r16(p + 0x80) as u32;
    for gi in 0..ng {
        let g = m.r32(p + 0x84).wrapping_add(gi * 0x30);
        let mut ei = 0;
        while ei < m.r16(g) as u32 {
            let e = m.r32(g + 4).wrapping_add(ei * 0x18);
            if key == m.r32(e + 0x14) {
                m.call(0x0021_4100, &[p, gi, 0, p3, 0, 0], &[]);
            }
            ei += 1;
        }
    }
}
/// FUN_00213b00. PARTIAL: FUN_00213f70 is called without visible arguments in the decompile; passed
/// (p, group, 0, 0, 0) like 213928.
pub fn ee_sub_213b00<M: QMem + ?Sized>(m: &mut M, p: u32, key: i32) {
    if key == -1 {
        return;
    }
    if let Some((gi, g, ei)) = find_entry(m, p, key as u32) {
        if !any_active(m, g) {
            m.w32(g + 0x20, ei);
        } else {
            m.call(0x0021_3f70, &[p, gi, 0, 0, 0], &[]);
        }
    }
}

/// FUN_00213bd8. PARTIAL: FUN_00214100 is called without visible arguments in the decompile; passed
/// (p, group, 0, 0, 0, 0).
pub fn ee_sub_213bd8<M: QMem + ?Sized>(m: &mut M, p: u32, key: u32) {
    if let Some((gi, _, _)) = find_entry(m, p, key) {
        ee_sub_214100(m, p, gi, 0, 0, 0, 0);
    }
}
fn copy32<M: QMem + ?Sized>(m: &mut M, dst: u32, src: u32) {
    for k in 0..8 {
        let w = m.r32(src + 4 * k);
        m.w32(dst + 4 * k, w);
    }
}
fn alloc_via<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let obj = m.r32(o + 0xc4);
    let vt = m.r32(obj);
    let adj = m.r16(vt + 8) as i16 as i32 as u32;
    let f = m.r32(vt + 0xc);
    m.call(f, &[obj.wrapping_add(adj), 0x20, 0x1f], &[]).0
}
/// FUN_00213c68
pub fn ee_sub_213c68<M: QMem + ?Sized>(m: &mut M, o: u32, src: u32) {
    let mut b = m.r32(o + 0xf8);
    if b == 0 {
        b = alloc_via(m, o);
        m.w32(o + 0xf8, b);
    }
    copy32(m, b, src);
}
/// FUN_00213cd8
pub fn ee_sub_213cd8<M: QMem + ?Sized>(m: &mut M, p: u32, key: u32, src: u32) {
    if let Some((_, g, _)) = find_entry(m, p, key) {
        let mut b = m.r32(g + 0x2c);
        if b == 0 {
            b = alloc_via(m, p);
            m.w32(g + 0x2c, b);
        }
        copy32(m, b, src);
    }
}
/// FUN_00213db0
pub fn ee_sub_213db0<M: QMem + ?Sized>(m: &mut M, p: u32, new: u32, ptr: u32) {
    let old = m.r32(ptr);
    if new != old {
        if old != 0xffff_ffff {
            ee_sub_213928(m, p, old);
        }
        m.w32(ptr, new);
        if new != 0xffff_ffff {
            ee_sub_213b00(m, p, new as i32);
        }
    }
}
/// first-level (count, array) of the table at (+0x84)+8 when `arr` is 0
fn root_tab<M: QMem + ?Sized>(m: &mut M, p: u32, cnt: i32, arr: u32) -> (i32, u32) {
    if arr == 0 {
        let t0 = m.r32(p + 0x84);
        let t = m.r32(t0 + 8);
        (m.r32(t) as i32, m.r32(t + 4))
    } else {
        (cnt, arr)
    }
}
/// child table (count, array) of the sub-group of `node` (node+0x58 selects the 0x18-byte record)
fn child_tab<M: QMem + ?Sized>(m: &mut M, p: u32, i2: u32, node: u32) -> (u32, u32) {
    let g = m.r32(i2 + 0x10);
    let base = m.r32(p + 0x84).wrapping_add(g.wrapping_mul(0x30));
    let pp = m.r32(node + 0x58).wrapping_mul(0x18).wrapping_add(m.r32(base + 8));
    (m.r32(pp), m.r32(pp + 4))
}
/// FUN_00213e20
pub fn ee_sub_213e20<M: QMem + ?Sized>(m: &mut M, p: u32, key: u32, p3: u32, p4: u32, p5: u32, cnt: i32, arr: u32) -> u32 {
    let (cnt, mut node) = root_tab(m, p, cnt, arr);
    for _ in 0..cnt.max(0) {
        let i2 = m.r32(node + 0x30);
        if i2 != 0 && m.r32(i2 + 4) == 4 {
            if key == m.r32(i2 + 0x10) {
                copy32(m, p3, node + 0x10);
                let (a, b) = (m.r32(node), m.r32(node + 4));
                m.w32(p4, a);
                m.w32(p5, b);
                return 1;
            }
            let (c, a) = child_tab(m, p, i2, node);
            if ee_sub_213e20(m, p, key, p3, p4, p5, c as i32, a) != 0 {
                return 1;
            }
        }
        node += 0x70;
    }
    0
}
fn start_voice<M: QMem + ?Sized>(m: &mut M, p: u32, gi: u32, p3: u32, node: u32, colour: u32, flag: u32) {
    let grp = m.r32(p + 0x84).wrapping_add(gi * 0x30);
    m.call(0x0021_8660, &[grp], &[]);
    m.call(0x0011_a058, &[node + 0x68], &[]);
    let u = m.call(0x0021_7ef8, &[grp], &[]).0;
    m.w32(node + 0x58, u);
    let sp = stack_push(m, 0x20);
    m.w32(sp, colour);
    m.w32(sp + 0x10, flag);
    m.call(0x0021_7d98, &[0, grp, p3, u, node + 0x10, sp, sp + 0x10], &[]);
    stack_pop(m, 0x20);
}
/// FUN_00213f70
pub fn ee_sub_213f70<M: QMem + ?Sized>(m: &mut M, p: u32, gi: u32, p3: u32, cnt: i32, arr: u32) -> u32 {
    let (cnt, mut node) = root_tab(m, p, cnt, arr);
    for _ in 0..cnt.max(0) {
        let i2 = m.r32(node + 0x30);
        if i2 != 0 && m.r32(i2 + 4) == 4 {
            if m.r32(i2 + 0x10) == gi {
                start_voice(m, p, gi, p3, node, 0x7f7f_7f7f, 0xffff_ffff);
                return 1;
            }
            let (c, a) = child_tab(m, p, i2, node);
            if ee_sub_213f70(m, p, gi, p3, c as i32, a) != 0 {
                return 1;
            }
        }
        node += 0x70;
    }
    0
}
/// FUN_00214100
pub fn ee_sub_214100<M: QMem + ?Sized>(m: &mut M, p: u32, gi: u32, p3: u32, p4: u32, cnt: i32, arr: u32) -> u32 {
    let (cnt, mut node) = root_tab(m, p, cnt, arr);
    for _ in 0..cnt.max(0) {
        let i2 = m.r32(node + 0x30);
        if i2 != 0 && m.r32(i2 + 4) == 4 {
            if m.r32(i2 + 0x10) == gi {
                if p4 == m.r32(node + 100) {
                    start_voice(m, p, gi, p3, node, 0x7f7f_7f7f, 0xffff_ffff);
                    return 1;
                }
            } else {
                let (c, a) = child_tab(m, p, i2, node);
                if ee_sub_214100(m, p, gi, p3, p4, c as i32, a) != 0 {
                    return 1;
                }
            }
        }
        node += 0x70;
    }
    0
}
/// FUN_002142b0
pub fn ee_sub_2142b0<M: QMem + ?Sized>(m: &mut M, g: u32, key: u32) -> u32 {
    if key == 0xffff_ffff {
        return 0;
    }
    let n = m.r16(g) as u32;
    let base = m.r32(g + 4);
    (0..n).any(|i| key == m.r32(base + 0x14 + 0x18 * i)) as u32
}
/// FUN_00214300
pub fn ee_sub_214300<M: QMem + ?Sized>(m: &mut M, o: u32, v: u32) {
    let cur = m.r32(o + 0xa8);
    if cur != v {
        if cur == 0 && m.call(0x0011_a380, &[0x003e_1060, o], &[]).0 != 0 {
            m.call(0x0011_a2e8, &[0x003e_1060, o], &[]);
        }
        m.w32(o + 0xa8, v);
        if v == 0 && m.call(0x0011_a380, &[0x003e_1060, o], &[]).0 == 0 {
            m.call(0x0011_a220, &[0x003e_1060, o], &[]);
        }
    }
}
/// FUN_0021a998
pub fn ee_sub_21a998<M: QMem + ?Sized>(m: &mut M, p1: u32, idx: u32, p3: u32) {
    let t0 = m.r32(p1 + 0x10);
    let t1 = m.r32(t0 + 8);
    let pp = idx.wrapping_mul(0x18).wrapping_add(t1);
    let mut n = m.r32(pp) as i32 - 1;
    if n >= 0 {
        let mut node = (n as u32).wrapping_mul(0x70).wrapping_add(m.r32(pp + 4));
        loop {
            let i1 = m.r32(node + 0x30);
            if i1 == 0 {
                if m.r32(node + 0x34) == 4 {
                    let a = m.r32(p1 + 0xc) + 0xfc;
                    m.call(0x0011_a098, &[a, node], &[]);
                }
            } else {
                m.call(0x0021_5960, &[i1, node, p3], &[]);
            }
            n -= 1;
            if n < 0 {
                break;
            }
            node -= 0x70;
        }
    }
}
/// FUN_0021bb68
pub fn ee_sub_21bb68<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x0021_aa60, &[1, 0xffff], &[]);
}

/// FUN_00220388: v(p2+0x10) += [DAT_002ddfb0] * DAT_002c4108 (VU adda with vf0 adds 1 to the w lane); for kinds
/// 4..=6 also store unit_xy([p1+0xe0] - v - [2ddfb0] * f) * DAT_002c410c at p2+0x20 (z forced 0,
/// DAT_002a2dd0 in x when shorter than 1e-4)
pub fn ee_sub_220388<M: QMem + ?Sized>(m: &mut M, p1: u32, p2: u32) {
    let f = m.rf(0x002c_4108);
    let v = m.rv(p2 + 0x10);
    let cv = m.rv(0x002d_dfb0);
    let r = [v[0] + cv[0] * f, v[1] + cv[1] * f, v[2] + cv[2] * f, 1.0 + v[3] + cv[3] * f];
    m.wv(p2 + 0x10, r);
    let k = m.r32(p1 + 4) as i32;
    if k < 7 && 3 < k {
        let w = m.rv(p1 + 0xe0);
        let a = [
            1.0 + w[0] - r[0] - cv[0] * f,
            1.0 + w[1] - r[1] - cv[1] * f,
            0.0,
            1.0 + w[3] - r[3] - cv[3] * f,
        ];
        let c = m.rf(0x002c_410c);
        let len = (a[0] * a[0] + a[1] * a[1]).sqrt();
        let o = if len < 0.0001 {
            [m.rf(0x002a_2dd0) * c, 0.0, 0.0, 0.0]
        } else {
            let q = 1.0 / len;
            [a[0] * q * c, a[1] * q * c, a[2] * q * c, a[3] * q * c]
        };
        m.wv(p2 + 0x20, o);
    }
}
/// FUN_00220508
pub fn ee_sub_220508<M: QMem + ?Sized>(m: &mut M, p1: u32, p2: u32) {
    let k = m.r32(p1 + 4) as i32;
    if k < 7 && 3 < k {
        let pos = m.call(0x001a_62e8, &[p2], &[]).1;
        let w = m.rv(p1 + 0xe0);
        let d = [w[0] - pos[0], w[1] - pos[1], w[2] - pos[2]];
        m.wv(p1 + 0x110, pos);
        let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        m.wv(p1 + 0x100, w);
        let t = m.r32(0x002d_ded0);
        m.w32(p1 + 0x128, 1);
        m.w32(p1 + 0x120, t);
        let s = m.rf(p1 + 0x28);
        m.wf(p1 + 0x124, len / s);
    }
}
/// FUN_002205a8. PARTIAL: the 64-bit third argument of FUN_001a6fc0 is passed as its low word.
pub fn ee_sub_2205a8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let mut i2 = m.r32(o + 0x10);
    let mut skip = false;
    if i2 == m.r32(0x002d_d5bc) {
        if m.r32(i2 + 0x13d0) != 8 {
            skip = true;
        } else {
            m.call(0x0013_6530, &[i2, 1], &[]);
        }
    } else {
        m.call(0x001a_58d0, &[i2, 0, 3], &[]);
        let t = m.r32(o + 0x10);
        super::q13_c::ee_sub_1f5720(m, t, 0);
        let t = m.r32(o + 0x10);
        super::q13_c::ee_sub_1f57c0(m, t, 0);
    }
    if !skip {
        i2 = m.r32(o + 0x10);
        vcl(m, i2, 0x118, 0x11c, &[], &[]);
        let k = m.r32(o);
        let lo = m.r32(o + 0x30);
        // FUN_001a6fc0's 3rd arg is *(undefined8 *)(o+0x30), a 64-bit value: pass both words (lo at o+0x30,
        // hi at o+0x34) as the lo/hi pair, not the low word alone. [HARD2]
        let hi = m.r32(o + 0x34);
        let u = m.call(0x001a_6fc0, &[k, 0, lo, hi], &[]).0;
        vcl(m, i2, 0x120, 0x124, &[u], &[]);
        let t = m.r32(o + 0x10);
        if m.call(0x0013_2b10, &[t, 100], &[]).0 != 0 {
            vcl(m, t, 0x260, 0x264, &[1], &[]);
        }
    }
    let c = m.r32(o + 0x12c);
    if c != 0 {
        vcl(m, c, 0x48, 0x4c, &[], &[]);
    }
}
/// FUN_00221940
pub fn ee_sub_221940<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.w32(o + 0x118, 0);
}
/// FUN_00221990: C2db858 ctor
pub fn ee_sub_221990<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    super::q_0014cc58::ee_sub_14fca8(m, o);
    m.w32(o + 0xc, 0x002d_b858);
    o
}
/// FUN_00223f50
pub fn ee_sub_223f50<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x0022_3e18, &[1, 0xffff], &[]);
}
/// FUN_002259b8
pub fn ee_sub_2259b8<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x0022_5880, &[1, 0xffff], &[]);
}
/// FUN_00226530
pub fn ee_sub_226530<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x0022_63f8, &[1, 0xffff], &[]);
}

/// FUN_002289b0
pub fn ee_sub_2289b0<M: QMem + ?Sized>(m: &mut M, p: u32) {
    m.call(0x0022_8988, &[p, 0x189], &[]);
}
/// FUN_002289d0
pub fn ee_sub_2289d0<M: QMem + ?Sized>(m: &mut M, p: u32) {
    m.call(0x0022_8988, &[p, 0x18a], &[]);
}
/// FUN_00228b28
pub fn ee_sub_228b28<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x0022_89f0, &[1, 0xffff], &[]);
}
/// FUN_00228c18: build the orientation basis (three constant rows from 0x2a2d00/10/20, the quad from
/// FUN_00256d18(FUN_001a6f00(0, p1, p2))), run FUN_00230cd0 and return its translation row (p3) / packed
/// rotation (p4). PARTIAL: the 128-bit a1 half of p2 and the 64-bit result of FUN_0011bfb8 are carried as
/// their low words only.
pub fn ee_sub_228c18<M: QMem + ?Sized>(m: &mut M, p1: u32, p2: u32, p3: u32, p4: u32) {
    let sp = stack_push(m, 0xf0);
    let u = m.call(0x001a_6f00, &[0, p1, p2], &[]).0;
    m.call(0x0025_6d18, &[sp + 0x30, u], &[]);
    for (dst, src, n) in [(0u32, 0x002a_2d00u32, 3u32), (0x10, 0x002a_2d10, 3), (0x20, 0x002a_2d20, 3)] {
        for k in 0..n {
            let w = m.r32(src + 4 * k);
            m.w32(sp + dst + 4 * k, w);
        }
        m.w32(sp + dst + 0xc, 0);
    }
    m.w32(sp + 0x3c, 0x3f80_0000);
    m.call(0x0023_33b4, &[sp + 0x40, 0, 0x40], &[]);
    for k in 0..3 {
        let w = m.r32(p1 + 0x160 + 4 * k);
        m.w32(sp + 0x70 + 4 * k, w);
    }
    let k = m.r32(p1 + 0x60);
    m.call(0x0023_0cd0, &[k, 0, sp, sp + 0x40], &[]);
    if p3 != 0 {
        for k in 0..4 {
            let w = m.r32(sp + 0x70 + 4 * k);
            m.w32(p3 + 4 * k, w);
        }
    }
    if p4 != 0 {
        let r = m.call(0x0011_bfb8, &[sp + 0x40], &[]);
        // FUN_0011bfb8 returns a 64-bit = two floats (x, y) of a projected vector; p4 is a [x, y, z, w] record
        // (z/w from sp+0x78/0x7c). The decompile stored the full undefined8, so keep the y float (r.1[1])
        // instead of 0 for the high word. [HARD2]
        m.w32(p4, r.0);
        m.w32(p4 + 4, r.1[1].to_bits());
        let (a, b) = (m.r32(sp + 0x78), m.r32(sp + 0x7c));
        m.w32(p4 + 8, a);
        m.w32(p4 + 0xc, b);
    }
    stack_pop(m, 0xf0);
}
/// FUN_00228d20: walk the list at *pp (next at +0xc, target at +4, flag word +0x148 bit 31 = want, bit 32 =
/// has): sync via 229710 / 229738, unlink the entries that end up without bit 32 onto a chain handed to
/// FUN_002296c0, clear both bits
pub fn ee_sub_228d20<M: QMem + ?Sized>(m: &mut M, mut pp: u32) {
    let mut acc = 0u32;
    let mut cur = m.r32(pp);
    while cur != 0 {
        let t = m.r32(cur + 4);
        let u = m.r64(t + 0x148);
        let nxt = m.r32(cur + 0xc);
        if (u >> 31) & 1 != (u >> 32) & 1 {
            if u & 0x8000_0000 == 0 {
                m.call(0x0022_9710, &[t, cur], &[]);
            } else {
                m.call(0x0022_9738, &[t, cur], &[]);
            }
        }
        let t = m.r32(cur + 4);
        if m.r64(t + 0x148) & 0x1_0000_0000 == 0 {
            m.w32(pp, nxt);
            m.w32(cur + 0xc, acc);
            acc = cur;
        } else {
            pp = cur + 0xc;
        }
        let t = m.r32(cur + 4);
        let v = m.r64(t + 0x148) & 0xffff_fffe_7fff_ffff;
        m.w32(t + 0x148, v as u32);
        m.w32(t + 0x14c, (v >> 32) as u32);
        cur = nxt;
    }
    m.call(0x0022_96c0, &[0x003d_d490, acc], &[]);
}
/// FUN_0022b020
pub fn ee_sub_22b020<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x0022_aee8, &[1, 0xffff], &[]);
}
/// FUN_0022b5c0
pub fn ee_sub_22b5c0<M: QMem + ?Sized>(m: &mut M, p1: u64, p2: u64) {
    if p2 == 0xffff && p1 != 0 {
        m.call(0x0022_b050, &[0x0044_3740], &[]);
        for (c, a, sz, n) in [
            (0x002d_d2e0u32, 0x0045_20a8u32, 0x14u32, 0x2000u32),
            (0x002d_d2e4, 0x0045_20d0, 0x18, 0x800),
            (0x002d_d2e8, 0x0045_20f8, 0x28, 0x200),
            (0x002d_d2ec, 0x0045_2120, 0x1c, 0x800),
            (0x002d_d2f0, 0x0045_2148, 0xc, 200),
            (0x002d_d2f4, 0x0045_2170, 0x14, 0x100),
        ] {
            let v = m.r32(c).wrapping_add(1);
            m.w32(c, v);
            if v == 1 {
                m.call(POOL_CREATE, &[a, sz, n, 0x10], &[]);
            }
        }
    }
}
/// FUN_0022b700
pub fn ee_sub_22b700<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_22b5c0(m, 1, 0xffff)
}
/// FUN_002562b8
pub fn ee_sub_2562b8<M: QMem + ?Sized>(m: &mut M, o: u32, v: f32) {
    let b = m.r32(o + 0x24);
    m.wf(b + 8, v);
    let t = m.r32(o + 0x20);
    super::q13_c::ee_sub_1f4bf8(m, t);
}
/// FUN_002562f0
pub fn ee_sub_2562f0<M: QMem + ?Sized>(m: &mut M, o: u32, v: u16) {
    let b = m.r32(o + 0x24);
    m.w16(b + 0x14, v);
    let t = m.r32(o + 0x20);
    super::q13_c::ee_sub_1f4bf8(m, t);
}
/// FUN_00256328
pub fn ee_sub_256328<M: QMem + ?Sized>(m: &mut M, o: u32, v: u32) {
    let b = m.r32(o + 0x24);
    m.w32(b + 0x18, v);
    let t = m.r32(o + 0x20);
    super::q13_c::ee_sub_1f4bf8(m, t);
}
