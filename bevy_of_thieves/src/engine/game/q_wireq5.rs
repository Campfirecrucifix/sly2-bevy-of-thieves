//! WIRE-Q5: 1:1 queue ports (against `QMem`) of functions reached live that only had native
//! (non-QMem) ports, so the live dispatch ran them as 0-returning stubs. Ported from the EE asm.
//! Callees go through `m.call` / `vslot` (the live dispatch routes ported ones).
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

const WORLD: u32 = 0x003d_d490;

fn w64<M: QMem + ?Sized>(m: &mut M, a: u32, v: u64) {
    m.w32(a, v as u32);
    m.w32(a + 4, (v >> 32) as u32);
}

/// FUN_00110c98: o = FUN_001ae4f8(0x190); o+0xc = vtable 0x2d9040; o+0 = o+4 = 0; returns o.
pub fn ee_sub_110c98<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let o = m.call(0x001a_e4f8, &[0x190], &[]).0;
    m.w32(o + 0xc, 0x002d_9040);
    m.w32(o, 0);
    m.w32(o + 4, 0);
    o
}

/// FUN_001443d8: +0xf04 < 3 (unsigned).
pub fn ee_sub_1443d8<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    (m.r32(a0 + 0xf04) < 3) as u32
}

/// FUN_00162670: 0 if +0x50 set or +0xf0 is 2 / 3; else FUN_0020e040(WORLD) == 0.
pub fn ee_sub_162670<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    if m.r32(a0 + 0x50) != 0 {
        return 0;
    }
    let k = m.r32(a0 + 0xf0);
    if k == 2 || k == 3 {
        return 0;
    }
    (m.call(0x0020_e040, &[WORLD], &[]).0 == 0) as u32
}

/// FUN_001633e8: DAT_003e0f9c == a0.
pub fn ee_sub_1633e8<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    (m.r32(0x003e_0f9c) == a0) as u32
}

/// FUN_001a5800: +0x28 |= bit 60, then slot 3 (o, 0, o).
pub fn ee_sub_1a5800<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let f = m.r64(a0 + 0x28) | (1u64 << 60);
    w64(m, a0 + 0x28, f);
    vslot(m, a0, 3, &[0, a0], &[]);
}

/// FUN_001a5848: +0x28 &= !bit 60; if bit 59 set FUN_001a00d0(WORLD, o, 0); then slot 3 (o, 1, o).
pub fn ee_sub_1a5848<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let f = m.r64(a0 + 0x28) & 0xefff_ffff_ffff_ffff;
    w64(m, a0 + 0x28, f);
    if f & (1u64 << 59) != 0 {
        m.call(0x001a_00d0, &[WORLD, a0, 0], &[]);
    }
    vslot(m, a0, 3, &[1, a0], &[]);
}

/// FUN_001a61e0(o, ev): FUN_001331f8(o, ev); ev 5 / 6 -> FUN_001619a0(0x2deb30, 0x1a / 0x1b, o).
pub fn ee_sub_1a61e0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.call(0x0013_31f8, &[a0, a1], &[]);
    match a1 {
        5 => {
            m.call(0x0016_19a0, &[0x002d_eb30, 0x1a, a0], &[]);
        }
        6 => {
            m.call(0x0016_19a0, &[0x002d_eb30, 0x1b, a0], &[]);
        }
        _ => {}
    }
}

/// FUN_001a62e8: r = slot 30 (o); returns the quad at r + 0x30 (low word in v0).
pub fn ee_sub_1a62e8<M: QMem + ?Sized>(m: &mut M, a0: u32) -> (u32, V4) {
    let r = vslot(m, a0, 30, &[], &[]).0;
    (m.r32(r + 0x30), m.rv(r + 0x30))
}

/// FUN_001ae1c8: empty.
pub fn ee_sub_1ae1c8<M: QMem + ?Sized>(_m: &mut M) {}

/// FUN_001c42e8: +0x30 = 0, +0x10..+0x2c = 0.0, tail FUN_001c4738(o, 0xffff).
pub fn ee_sub_1c42e8<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    m.w32(a0 + 0x30, 0);
    for off in (0x10..0x30).step_by(4) {
        m.wf(a0 + off, 0.0);
    }
    m.call(0x001c_4738, &[a0, 0xffff], &[]).0
}

/// FUN_001f6030(o, id): first of +0xe0 records (0xb0 each, at +0xe4) whose +0xc == id, else 0.
pub fn ee_sub_1f6030<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let n = m.r32(a0 + 0xe0) as i32;
    let mut p = m.r32(a0 + 0xe4);
    for _ in 0..n.max(0) {
        if m.r32(p + 0xc) == a1 {
            return p;
        }
        p = p.wrapping_add(0xb0);
    }
    0
}

/// FUN_00210a88: DAT_003e1110.
pub fn ee_sub_210a88<M: QMem + ?Sized>(m: &mut M) -> u32 {
    m.r32(0x003e_1110)
}

/// FUN_002313f0: +0x54 = -1; if +0x10 == 0 and +0x28 bit 60: FUN_00147e88(WORLD, +0x14).
pub fn ee_sub_2313f0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.w32(a0 + 0x54, u32::MAX);
    if m.r32(a0 + 0x10) == 0 && (m.r64(a0 + 0x28) >> 60) & 1 != 0 {
        let a1 = m.r32(a0 + 0x14);
        m.call(0x0014_7e88, &[WORLD, a1], &[]);
    }
}

/// FUN_001d7da0(tbl, key, out): walks the 10-byte node list of tbl+4 (head index = *(u16 *)(nodes + 6)) matching key's u16 path (key+4, length key+0); out = {node, next id}.
pub fn ee_sub_1d7da0<M: QMem + ?Sized>(m: &mut M, t2: u32, t1: u32, a2: u32) -> u32 {
    let nodes = m.r32(t2 + 4);
    if m.r16(nodes + 6) == 0 {
        return 0;
    }
    let v1 = m.r16(nodes + 6) as u32;
    m.w32(a2 + 4, 0);
    let mut t0 = 0u32;
    let mut a3 = (v1.wrapping_mul(10)).wrapping_add(nodes);
    loop {
        let path = m.r32(t1 + 4);
        let a1 = m.r16(a3) as i16 as i32 as u32;
        let v1 = m.r16(path.wrapping_add(t0.wrapping_mul(2))) as u32;
        let next;
        if a1 != v1 {
            next = m.r16(a3 + 8) as u32;
            if next == 0 {
                return 0;
            }
        } else {
            t0 += 1;
            if t0 == m.r32(t1) {
                m.w32(a2, a3);
                return 1;
            }
            if m.r8(a3 + 3) != 0 {
                m.w32(a2, a3);
                let a0 = t0.wrapping_mul(2);
                t0 += 1;
                let p = m.r32(t1 + 4);
                let id = m.r16(a0.wrapping_add(p)) as u32;
                m.w32(a2 + 4, id);
                return (t0 == m.r32(t1)) as u32;
            }
            if m.r8(a3 + 2) as i8 != -1 {
                return 0;
            }
            next = m.r16(a3 + 6) as u32;
            if next == 0 {
                return 0;
            }
        }
        a3 = m.r32(t2 + 4).wrapping_add(next.wrapping_mul(10));
    }
}

/// FUN_001d8720(hash, name): h = FUN_001d8640(name); bucket chain hash[h % 53] (+0 h, +4 str, +8 value,
/// +0xc next); value of the first entry with equal hash and FUN_002342d0(str, name) == 0, else -1.
pub fn ee_sub_1d8720<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let h = m.call(0x001d_8640, &[a1], &[]).0;
    let mut s0 = m.r32(a0.wrapping_add((h % 0x35) * 4));
    while s0 != 0 {
        if m.r32(s0) == h {
            let s = m.r32(s0 + 4);
            if m.call(0x0023_42d0, &[s, a1], &[]).0 == 0 {
                return m.r32(s0 + 8);
            }
        }
        s0 = m.r32(s0 + 0xc);
    }
    u32::MAX
}

/// FUN_001f2fe0(o, bits): +0x40 |= bits (or *(+0x40) |= bits when +0x40 is an even pointer); each child
/// (+0x64 .. +0x70 via +0x24) gets slot 46 (child, bits).
pub fn ee_sub_1f2fe0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let v1 = m.r32(a0 + 0x40);
    if v1 & 1 == 0 {
        let w = m.r32(v1) | a1;
        m.w32(v1, w);
    } else {
        m.w32(a0 + 0x40, v1 | a1);
    }
    let mut s0 = m.r32(a0 + 0x64);
    while s0 != m.r32(a0 + 0x70) {
        vslot(m, s0, 46, &[a1], &[]);
        s0 = m.r32(s0 + 0x24);
    }
}

/// FUN_001f36e0(o): FUN_001a5e28(o); if (+0xe8)->+0xdc: FUN_001f37c8(o, FUN_001c0d70(FUN_001a62e8(o), 1));
/// fade: +0x12c += DAT_002dded4 * +0x130; <= 0 -> both 0 and slot 8 (o); >= 1 -> +0x130 = 0, +0x12c = 1.
pub fn ee_sub_1f36e0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5e28, &[a0], &[]);
    let e8 = m.r32(a0 + 0xe8);
    let v1 = if e8 == 0 { 0 } else { m.r32(e8 + 0xdc) };
    if v1 != 0 {
        let (r, rv) = m.call(0x001a_62e8, &[a0], &[]);
        let r2 = m.call(0x001c_0d70, &[r, 1], &[rv]).0;
        m.call(0x001f_37c8, &[a0, r2], &[]);
    }
    let f2 = m.rf(a0 + 0x130);
    if f2 == 0.0 {
        return;
    }
    let f1 = m.rf(a0 + 0x12c) + m.rf(0x002d_ded4) * f2;
    m.wf(a0 + 0x12c, f1);
    if f1 <= 0.0 {
        m.wf(a0 + 0x130, 0.0);
        m.wf(a0 + 0x12c, 0.0);
        vslot(m, a0, 8, &[], &[]);
    } else if 1.0 <= f1 {
        m.wf(a0 + 0x130, 0.0);
        m.wf(a0 + 0x12c, 1.0);
    }
}

/// FUN_00227c90(o, pos): p = FUN_001a6f00(0, o, pos); 0 if |p - o+0x110|^2 > +0x120; else
/// FUN_00118fb0(+0xec, 0x7fffffff, p) != 0.
pub fn ee_sub_227c90<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: V4) -> u32 {
    let p = m.call(0x001a_6f00, &[0, a0], &[a1]).1;
    let r = m.rf(a0 + 0x120);
    let c = m.rv(a0 + 0x110);
    let d = [p[0] - c[0], p[1] - c[1], p[2] - c[2]];
    let d2 = (d[0] * d[0] + d[1] * d[1]) + d[2] * d[2];
    if r < d2 {
        return 0;
    }
    let ec = m.r32(a0 + 0xec);
    (m.call(0x0011_8fb0, &[ec, 0x7fff_ffff], &[p]).0 != 0) as u32
}

/// FUN_0020aaa8(o): +0xefc = 0; for each of +0xed8 ids at +0xeb8: r = FUN_0015e938(WORLD, 4, id, o);
/// nonzero r appended to +0xedc[+0xefc++].
pub fn ee_sub_20aaa8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.w32(a0 + 0xefc, 0);
    let mut i = 0i32;
    while i < m.r32(a0 + 0xed8) as i32 {
        let id = m.r32(a0 + 0xeb8 + 4 * i as u32);
        let r = m.call(0x0015_e938, &[WORLD, 4, id, a0], &[]).0;
        if r != 0 {
            let n = m.r32(a0 + 0xefc);
            m.w32(a0 + 0xedc + 4 * n, r);
            m.w32(a0 + 0xefc, n + 1);
        }
        i += 1;
    }
}

fn fv(f: f32) -> V4 {
    [f, 0.0, 0.0, 0.0]
}

/// FUN_00140558(jt): unless +0x10e0: over the list DAT_003e1444 (next +0xfe0) d = FUN_00208e70(e, jt, &f):
/// nearest (min f) -> (best_d, best_e); highest d (ties: min f) -> (top_d, top_e); then slot 94
/// (jt, best_d, best_e, top_d, top_e).
pub fn ee_sub_140558<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    if m.r32(a0 + 0x10e0) != 0 {
        return;
    }
    let sp = stack_push(m, 0x90);
    let mut f21 = f32::from_bits(0x7f7f_ffff);
    let mut f20 = f21;
    let (mut s5, mut s4) = (0u32, 0u32);
    let (mut s1, mut s3) = (-1i32, -1i32);
    let mut s0 = m.r32(0x003e_1444);
    while s0 != 0 {
        let v1 = m.call(0x0020_8e70, &[s0, a0, sp + 4], &[]).0 as i32;
        let f0 = m.rf(sp + 4);
        if f0 < f20 {
            f20 = f0;
            s3 = v1;
            s4 = s0;
        }
        if s1 < v1 || (v1 == s1 && f0 < f21) {
            f21 = f0;
            s1 = v1;
            s5 = s0;
        }
        s0 = m.r32(s0 + 0xfe0);
    }
    stack_pop(m, 0x90);
    vslot(m, a0, 94, &[s3 as u32, s4, s1 as u32, s5], &[]);
}

/// FUN_001430c8(o, _, a2): when +0x10f8: p = FUN_001a6f00(0, o, FUN_001a62e8(+0x10f8)); a = FUN_002458f0(p.y,
/// p.x); a2 = |a| < 1.0 ? 0 : |a| > 2.5 ? 3 : a > 0 ? 1 : 2 (else a2 is the incoming register); returns
/// by +0x10fc + 1: 2 -> 0x10 << a2, 3 -> 0x100 << a2, 4 -> 0x1000, 5 -> 0x100, else 8.
pub fn ee_sub_1430c8<M: QMem + ?Sized>(m: &mut M, a0: u32, _a1: u32, a2: u32) -> u32 {
    let mut a2 = a2;
    let t = m.r32(a0 + 0x10f8);
    if t != 0 {
        let q = m.call(0x001a_62e8, &[t], &[]).1;
        let p = m.call(0x001a_6f00, &[0, a0], &[q]).1;
        let f2 = m.call(0x0024_58f0, &[], &[fv(p[1]), fv(p[0])]).1[0];
        let f1 = f2.abs();
        a2 = if f1 < 1.0 {
            0
        } else if 2.5 < f1 {
            3
        } else if 0.0 < f2 {
            1
        } else {
            2
        };
    }
    match m.r32(a0 + 0x10fc).wrapping_add(1) {
        2 => 0x10u32.wrapping_shl(a2),
        3 => 0x100u32.wrapping_shl(a2),
        4 => 0x1000,
        5 => 0x100,
        _ => 8,
    }
}

/// FUN_00141a50(jt): with +0x150, for each of the 3 gadget slots (0xc each at +0x1180, mask table
/// 0x2c3258, cost table 0x2bcd00 stride 0x20): state 0 -> when FUN_001c46c0(+0x150, mask, 0.15): slot 98
/// (jt, id) == 0 -> FUN_001fb2f8(FUN_001fbb90(0x2e), 0); else FUN_001441a0(jt) < cost+0xc -> same sound
/// and +0x1304 = DAT_002dded0; else FUN_00141c08(jt, slot, cost+0). State 1 -> mask in (+0x150)+0x30 and
/// slot 98 -> slot 100 (jt, slot); else FUN_00141c08(jt, slot, 0).
pub fn ee_sub_141a50<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    if m.r32(a0 + 0x150) == 0 {
        return;
    }
    for s3 in 0..3u32 {
        let a1 = m.r32(0x002c_3258 + 4 * s3);
        let s2 = a0 + 0x1180 + 0xc * s3;
        let v1 = m.r32(s2 + 4);
        let s1 = m.r32(s2);
        if v1 == 0 {
            let p = m.r32(a0 + 0x150);
            if m.call(0x001c_46c0, &[p, a1], &[fv(f32::from_bits(0x3e19_999a))]).0 == 0 {
                continue;
            }
            if vslot(m, a0, 98, &[s1], &[]).0 == 0 {
                let x = m.call(0x001f_bb90, &[0x2e], &[]).0;
                m.call(0x001f_b2f8, &[x, 0], &[]);
                continue;
            }
            let v = m.call(0x0014_41a0, &[a0], &[]).0 as i32;
            let c = 0x002b_cd00 + s1.wrapping_mul(32);
            if v < m.r32(c + 0xc) as i32 {
                let x = m.call(0x001f_bb90, &[0x2e], &[]).0;
                m.call(0x001f_b2f8, &[x, 0], &[]);
                let f = m.rf(0x002d_ded0);
                m.wf(a0 + 0x1304, f);
            } else {
                let cost = m.r32(c);
                m.call(0x0014_1c08, &[a0, s2, cost], &[]);
            }
        } else if v1 == 1 {
            let p = m.r32(a0 + 0x150);
            if m.r32(p + 0x30) & a1 != 0 && vslot(m, a0, 98, &[s1], &[]).0 != 0 {
                vslot(m, a0, 100, &[s2], &[]);
            } else {
                m.call(0x0014_1c08, &[a0, s2, 0], &[]);
            }
        }
    }
}

/// FUN_001f2d48(alo): FUN_001a5848; FUN_001f2ba8; +0x130 = 0, +0x12c = 1.0; if (+0xe8)->+0xdc:
/// FUN_00149b58(WORLD, 3, alo, pmf 0x001f3868_ffff0000 (low word passed), 0), FUN_001f37c8(alo, 0); then
/// with a DAT_002dd2fc iterator frame on the stack, slot 11 on each child (+0x64 list, link +8) until one
/// has +0x28 bit 56.
pub fn ee_sub_1f2d48<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5848, &[a0], &[]);
    m.call(0x001f_2ba8, &[a0], &[]);
    m.w32(a0 + 0x130, 0);
    m.wf(a0 + 0x12c, 1.0);
    let e8 = m.r32(a0 + 0xe8);
    let v1 = if e8 == 0 { 0 } else { m.r32(e8 + 0xdc) };
    if v1 != 0 {
        m.call(0x0014_9b58, &[WORLD, 3, a0, 0xffff_0000, 0], &[]);
        m.call(0x001f_37c8, &[a0, 0], &[]);
    }
    let sp = stack_push(m, 0x40);
    let lst = a0 + 0x64;
    m.w32(sp + 4, lst);
    m.w32(sp, lst);
    let prev = m.r32(0x002d_d2fc);
    let link = m.r32(lst + 8);
    m.w32(0x002d_d2fc, sp);
    m.w32(sp + 0xc, prev);
    let mut a2 = m.r32(a0 + 0x64);
    m.w32(sp + 8, link);
    m.w32(sp + 4, a2.wrapping_add(link));
    const B56: u64 = 1 << 56;
    if a2 != 0 && m.r64(a2 + 0x28) & B56 == 0 {
        loop {
            vslot(m, a2, 11, &[], &[]);
            let it = m.r32(sp + 4);
            let l = m.r32(sp + 8);
            let nx = m.r32(it);
            m.w32(sp + 4, nx.wrapping_add(l));
            a2 = nx;
            if a2 == 0 || m.r64(a2 + 0x28) & B56 != 0 {
                break;
            }
        }
    }
    let v = m.r32(sp + 0xc);
    m.w32(0x002d_d2fc, v);
    stack_pop(m, 0x40);
}

/// FUN_00129ec8 (CRef::Release)(ref): by ref kind (+0) - 0x1a: kinds 0..4 decrement a counter of ref+4
/// (+0x10 / +0x40 / +0x10 / +8 / +0xc) and free it to the pool 0x452188 / 0x4521a0 / 0x4521b8 / 0x4521d0 /
/// 0x4521e8 (FUN_0011cd98) at 0; 5: rc +0 -> FUN_00256d40(p+0x10, 2), release(p+8), FUN_002550f8(p),
/// pool 0x4520a8; 6: rc -> FUN_00257148(p, 3); 7: slot 1 (p); 8: rc -> FUN_00256120(p), pool 0x452170.
pub fn ee_sub_129ec8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let k = m.r32(a0).wrapping_sub(0x1a);
    if k >= 9 {
        return;
    }
    let p = m.r32(a0 + 4);
    let dec = |m: &mut M, at: u32| -> bool {
        let v = m.r32(at).wrapping_sub(1);
        m.w32(at, v);
        v == 0
    };
    match k {
        0..=4 => {
            let (off, pool) = [(0x10, 0x0045_2188), (0x40, 0x0045_21a0), (0x10, 0x0045_21b8), (8, 0x0045_21d0), (0xc, 0x0045_21e8)][k as usize];
            if dec(m, p + off) {
                m.call(0x0011_cd98, &[pool, p], &[]);
            }
        }
        5 => {
            if dec(m, p) && p != 0 {
                m.call(0x0025_6d40, &[p + 0x10, 2], &[]);
                m.call(0x0012_9ec8, &[p + 8], &[]);
                m.call(0x0025_50f8, &[p], &[]);
                m.call(0x0011_cd98, &[0x0045_20a8, p], &[]);
            }
        }
        6 => {
            if dec(m, p) && p != 0 {
                m.call(0x0025_7148, &[p, 3], &[]);
            }
        }
        7 => {
            if p != 0 {
                vslot(m, p, 1, &[], &[]);
            }
        }
        _ => {
            if dec(m, p) && p != 0 {
                m.call(0x0025_6120, &[p], &[]);
                m.call(0x0011_cd98, &[0x0045_2170, p], &[]);
            }
        }
    }
}

/// FUN_001a5900 (SetParent)(o, parent, a2, a3): unlinks o from its old parent's child list
/// (FUN_001a5448 / FUN_0011a2e8) and links it into the new one (bit 56 of +0x28: FUN_0011a220 append +
/// first-child fixup, else FUN_0011a280 insert); with FUN_001a61a0: slot 21 before / slot 22 after
/// (and the 0x3dd4bc list unlink); +0x10 = parent, slot 12 (o, old); slot 9 (parent, a2, a3) on each
/// slot-27 sub-object; in-world change (s4 vs bit 60) -> slot 10 / 11; out-of-world root -> FUN_00148300.
pub fn ee_sub_1a5900<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32) {
    const B56: u64 = 1 << 56;
    let (s0, s3, s6, s5) = (a0, a1, a2, a3);
    let v1 = m.r64(s0 + 0x28);
    let s7 = ((v1 >> 60) & 1) as u32;
    let mut s4 = 0u32;
    if s3 == 0 || (m.r64(s3 + 0x28) >> 60) & 1 != 0 {
        s4 = (v1 & B56 == 0) as u32;
    }
    let sp = stack_push(m, 0xb0);
    let old = m.r32(s0 + 0x10);
    m.call(0x001a_5448, &[old, sp, sp + 4], &[]);
    m.call(0x001a_5448, &[s3, sp + 8, sp + 0xc], &[]);
    if m.r64(s0 + 0x28) & B56 != 0 {
        let v1 = m.r32(sp + 4);
        if m.r32(v1) == s0 {
            let n = m.r32(s0 + 0x24);
            m.w32(v1, n);
        }
        let l = m.r32(sp);
        m.call(0x0011_a2e8, &[l, s0], &[]);
        let l = m.r32(sp + 8);
        m.call(0x0011_a220, &[l, s0], &[]);
        let v1 = m.r32(sp + 0xc);
        if m.r32(v1) == 0 {
            m.w32(v1, s0);
        }
    } else {
        let l = m.r32(sp);
        m.call(0x0011_a2e8, &[l, s0], &[]);
        let c = m.r32(sp + 0xc);
        let first = m.r32(c);
        let l = m.r32(sp + 8);
        m.call(0x0011_a280, &[l, first, s0], &[]);
    }
    if m.call(0x001a_61a0, &[s0], &[]).0 != 0 {
        vslot(m, s0, 21, &[], &[]);
        let a1 = m.r32(s0 + 0x14);
        if m.r32(a1) == s0 {
            m.call(0x0011_a2e8, &[0x003d_d4bc, a1], &[]);
        }
    }
    let old = m.r32(s0 + 0x10);
    m.w32(s0 + 0x10, s3);
    vslot(m, s0, 12, &[s0, old], &[]);
    let n = vslot(m, s0, 27, &[sp + 0x10], &[]).0 as i32;
    for i in 0..n.max(0) as u32 {
        let arr = m.r32(sp + 0x10);
        let t0 = m.r32(arr.wrapping_add(4 * i));
        vslot(m, t0, 9, &[s3, s6, s5], &[]);
    }
    if s4 != s7 {
        vslot(m, s0, if s4 != 0 { 10 } else { 11 }, &[], &[]);
    }
    if m.call(0x001a_61a0, &[s0], &[]).0 != 0 {
        vslot(m, s0, 22, &[], &[]);
    } else if m.r32(s0 + 0x10) == 0 && s4 != 0 {
        m.call(0x0014_8300, &[WORLD, s0], &[]);
    }
    stack_pop(m, 0xb0);
}

/// FUN_001a64e8(o, n, tbl): resolves n 12-byte member records {flags, id, off} into o+off: 0x10 ->
/// o's +0x54 table entry (0x100 each; the asm compares only the last of +0x50 entries, n times) with
/// +0x9c == id; 0x20 -> FUN_001f6030 (0x200: FUN_001f2ad0); 0x40 -> FUN_001f5ff8 (0x200: FUN_001f69f8);
/// else FUN_0015e938(WORLD, kind 4 / 5 / 1 (| 0x400), id, o) (0x200: slot 8 on the hit).
pub fn ee_sub_1a64e8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    let (s3, s4, s5) = (a0, a1 as i32, a2);
    let mut s2 = 0i32;
    while s2 < s4 {
        let s1 = s5.wrapping_add(12 * s2 as u32);
        let v1 = m.r32(s1);
        s2 += 1;
        let id = m.r32(s1 + 4);
        if v1 & 0x10 != 0 {
            let n = m.r32(s3 + 0x50) as i32;
            let a2 = (n << 8).wrapping_sub(0x100) as u32;
            let mut a3 = n - 1;
            while a3 >= 0 {
                let e = a2.wrapping_add(m.r32(s3 + 0x54));
                if m.r32(e + 0x9c) == id {
                    let off = m.r32(s1 + 8);
                    m.w32(s3.wrapping_add(off), e);
                    break;
                }
                a3 -= 1;
            }
        } else if v1 & 0x20 != 0 {
            let r = m.call(0x001f_6030, &[s3, id], &[]).0;
            if r != 0 {
                let off = m.r32(s1 + 8);
                m.w32(s3.wrapping_add(off), r);
                if m.r32(s1) & 0x200 != 0 {
                    m.call(0x001f_2ad0, &[s3, r], &[]);
                }
            }
        } else if v1 & 0x40 != 0 {
            let r = m.call(0x001f_5ff8, &[s3, id], &[]).0;
            if r != 0 {
                let off = m.r32(s1 + 8);
                m.w32(s3.wrapping_add(off), r);
                if m.r32(s1) & 0x200 != 0 {
                    let id = m.r32(s1 + 4);
                    m.call(0x001f_69f8, &[s3, id], &[]);
                }
            }
        } else {
            let t0 = if v1 & 2 != 0 { 4 } else if v1 & 1 != 0 { 5 } else { 1 };
            let f = m.r32(s1);
            let k = if f & 0x400 != 0 { t0 | 0x400 } else { t0 };
            let s0 = m.call(0x0015_e938, &[WORLD, k, id, s3], &[]).0;
            if s0 != 0 {
                if m.r32(s1) & 0x200 != 0 {
                    vslot(m, s0, 8, &[], &[]);
                }
                let off = m.r32(s1 + 8);
                m.w32(s3.wrapping_add(off), s0);
            }
        }
    }
}

fn clamp2<M: QMem + ?Sized>(m: &mut M, f0: f32) -> f32 {
    let lo = m.rf(0x002b_bb60);
    let hi = m.rf(0x002b_bb64);
    if f0 < lo {
        lo
    } else if hi < f0 {
        hi
    } else {
        f0
    }
}

/// FUN_001a1358(d): damage flash / knockback driver: while +0x90 != -1: with +4 == -1 the fade ends
/// after +0x9c (t = age / +0x9c) -> +0x90 = -1, (+0)->+0xd0 = 0, FUN_001f4bf8; else (+0)->+0xd0 =
/// clamp(poly_0x2bfaa0(t)) * clamp(poly_0x2bfab0(FUN_00244e10((now - +0x94) * 10))). State 7: face away
/// from the slot-32 vector (FUN_00204660(FUN_0011d2d0(0x2bfac0, 0, +0x1a8, atan2 + pi, dt))) and end via
/// FUN_001a15d0 after 0.25 s with +0x208 or when FUN_0019a098(o, 0, +8).
pub fn ee_sub_1a1358<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let s1 = a0;
    let clock = 0x002d_ded0u32;
    'fade: {
        if m.r32(s1 + 0x90) == u32::MAX {
            break 'fade;
        }
        let a0 = m.r32(s1 + 4);
        let f20;
        if a0 == u32::MAX {
            let f0 = m.rf(clock) - m.rf(s1 + 0x98);
            let f2 = m.rf(s1 + 0x9c);
            if f2 < f0 {
                let o = m.r32(s1);
                m.w32(s1 + 0x90, a0);
                m.w32(o + 0xd0, 0);
                m.call(0x001f_4bf8, &[o], &[]);
                break 'fade;
            }
            f20 = f0 / f2;
        } else {
            f20 = 0.0;
        }
        let f12 = (m.rf(clock) - m.rf(s1 + 0x94)) * 10.0;
        let f6 = m.call(0x0024_4e10, &[], &[fv(f12)]).1[0];
        let (a0c, a1c, a2c) = (m.rf(0x002b_faa0), m.rf(0x002b_faa4), m.rf(0x002b_faa8));
        let f3 = clamp2(m, a0c + f20 * (a1c + f20 * a2c));
        let (b0, b1, b2) = (m.rf(0x002b_fab0), m.rf(0x002b_fab4), m.rf(0x002b_fab8));
        let f0 = clamp2(m, b0 + f6 * (b1 + f6 * b2));
        let o = m.r32(s1);
        m.wf(o + 0xd0, f3 * f0);
    }
    if m.r32(s1 + 4) != 7 {
        return;
    }
    let o = m.r32(s1);
    let q = vslot(m, o, 32, &[], &[]).1;
    let ang = m.call(0x0024_58f0, &[], &[fv(q[1]), fv(q[0])]).1[0];
    let f13 = ang + f32::from_bits(0x4049_0fdb);
    let f14 = m.rf(clock + 4);
    let f12 = m.rf(o + 0x1a8);
    let r = m.call(0x0011_d2d0, &[0x002b_fac0, 0], &[fv(f12), fv(f13), fv(f14)]).1[0];
    m.call(0x0020_4660, &[o], &[fv(r)]);
    let o = m.r32(s1);
    if 0.25 < m.rf(clock) - m.rf(s1 + 8) && m.r32(o + 0x208) != 0 {
        m.call(0x001a_15d0, &[s1], &[]);
        return;
    }
    let f = m.rf(s1 + 8);
    if m.call(0x0019_a098, &[o, 0], &[fv(f)]).0 != 0 {
        m.call(0x001a_15d0, &[s1], &[]);
    }
}

/// FUN_00203fc0(guard): FUN_001f36e0; FUN_00203e08; +0x1a4 == 0 -> +0x180 = clamp(poly_0x2c3c80(|FUN_0011cf60(
/// +0x1a8 - atan2(p.y, p.x))|)) with p = slot 30 (o); +0x1a4 == +-1 -> +0x180 = 1.0; +0x208 -> +0x20c = now.
pub fn ee_sub_203fc0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let s1 = a0;
    m.call(0x001f_36e0, &[s1], &[]);
    m.call(0x0020_3e08, &[s1], &[]);
    let v1 = m.r32(s1 + 0x1a4) as i32;
    if v1 == 0 {
        let s0 = vslot(m, s1, 30, &[], &[]).0;
        let v0 = vslot(m, s1, 30, &[], &[]).0;
        let (y, x) = (m.rf(s0 + 4), m.rf(v0));
        let ang = m.call(0x0024_58f0, &[], &[fv(y), fv(x)]).1[0];
        let d = m.rf(s1 + 0x1a8) - ang;
        let f0 = m.call(0x0011_cf60, &[], &[fv(d)]).1[0].abs();
        let (c0, c1, c2) = (m.rf(0x002c_3c80), m.rf(0x002c_3c84), m.rf(0x002c_3c88));
        let f3 = clamp2(m, c0 + f0 * (c1 + f0 * c2));
        m.wf(s1 + 0x180, f3);
    } else if v1 == 1 || v1 == -1 {
        m.wf(s1 + 0x180, 1.0);
    }
    if m.r32(s1 + 0x208) != 0 {
        let t = m.rf(0x002d_ded0);
        m.wf(s1 + 0x20c, t);
    }
}

/// FUN_0011c168(out, a, b): out = a * b (VU0 microprogram 0x60; same replacement as
/// anim::pose::ee_sub_11c168, column-vector product, b applied first). Returns out.
pub fn ee_sub_11c168<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) -> u32 {
    let a = m.rm(a1);
    let b = m.rm(a2);
    let o = crate::engine::anim::pose::ee_sub_11c168(&a, &b);
    m.wm(a0, o);
    a0
}

/// FUN_001a2460(jt): hazard contact with +0x108c: p = pos (FUN_001a62e8) with z += +0x10a0 (- 10 in
/// states 3 / 10); slot 72 (hazard, p) == 0 -> done. Hazard +0x184 and (slot-32 z < 0, or +0x2dc == -1 and
/// state != 7) -> FUN_001f7598 on a zeroed 0x60 record {hazard, jt, +0x188, +0x184, pos, slot 32}. With
/// +0x174 and FUN_001a2788(jt, hazard, &p): every +0x1090 contact (8 each, +0x1094, rearm 0.05 s) whose
/// slot-31 matrix (FUN_002309e8 / FUN_0011c168) z straddles p.z sends a kind-4 message (slot 50 on +0x174).
pub fn ee_sub_1a2460<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let s1 = a0;
    let clock = 0x002d_ded0u32;
    if m.r32(s1 + 0x108c) == 0 {
        return;
    }
    let sp = stack_push(m, 0x1a0);
    'body: {
        let q = m.call(0x001a_62e8, &[s1], &[]).1;
        m.wv(sp, q);
        let f1 = m.rf(sp + 8) + m.rf(s1 + 0x10a0);
        m.wf(sp + 8, f1);
        let st = m.r32(s1 + 0xdf4);
        if st == 3 || st == 10 {
            m.wf(sp + 8, f1 - 10.0);
        }
        let hz = m.r32(s1 + 0x108c);
        let p = m.rv(sp);
        if vslot(m, hz, 72, &[], &[p]).0 == 0 {
            break 'body;
        }
        let hz = m.r32(s1 + 0x108c);
        if m.r32(hz + 0x184) != 0 {
            let q = vslot(m, s1, 32, &[], &[]).1;
            let s0 = q[2] < 0.0 || (m.r32(s1 + 0x2dc) == u32::MAX && m.r32(s1 + 0xdf4) != 7);
            if s0 {
                m.call(0x0023_33b4, &[sp + 0x10, 0, 0x60], &[]);
                let a1 = m.r32(s1 + 0x108c);
                m.w32(sp + 0x18, s1);
                m.w32(sp + 0x10, a1);
                let x = m.r32(a1 + 0x188);
                m.w32(sp + 0x1c, x);
                let x = m.r32(a1 + 0x184);
                m.w32(sp + 0x20, x);
                let q = m.call(0x001a_62e8, &[s1], &[]).1;
                m.wv(sp + 0x30, q);
                let q = vslot(m, s1, 32, &[], &[]).1;
                m.wv(sp + 0x40, q);
                m.call(0x001f_7598, &[sp + 0x10], &[]);
            }
        }
        let a1 = m.r32(s1 + 0x108c);
        if m.r32(a1 + 0x174) == 0 {
            break 'body;
        }
        if m.call(0x001a_2788, &[s1, a1, sp], &[]).0 == 0 {
            break 'body;
        }
        let f20 = f32::from_bits(0x3d4c_cccd);
        let mut s3 = 0i32;
        while s3 < m.r32(s1 + 0x1094) as i32 {
            let s2 = m.r32(s1 + 0x1090).wrapping_add(8 * s3 as u32);
            s3 += 1;
            if m.rf(clock) - m.rf(s2 + 4) < f20 {
                continue;
            }
            let s0 = m.r32(s2);
            let q = m.call(0x001a_62e8, &[s0], &[]).1;
            m.wv(sp + 0x10, q);
            let r = vslot(m, s0, 31, &[sp + 0x70], &[]).0;
            let dt = m.rf(clock + 4);
            m.call(0x0023_09e8, &[r, 0, sp + 0x20, 0], &[fv(dt)]);
            m.call(0x0011_c168, &[sp + 0xb0, sp + 0x20, sp + 0x70], &[]);
            for k in 0..4 {
                let w = m.rv(sp + 0xb0 + 16 * k);
                m.wv(sp + 0x20 + 16 * k, w);
            }
            let f1 = m.rf(sp + 8);
            let f0 = m.rf(sp + 0x18);
            let z = m.rf(sp + 0x58);
            let hit = if f1 <= f0 { z <= f1 } else { f1 <= z };
            if hit {
                let q = m.rv(sp + 0x10);
                m.wv(sp + 0xd0, q);
                let f = m.rf(sp + 8);
                m.wf(sp + 0xd8, f);
                m.w32(sp + 0xb0, 4);
                let hz = m.r32(s1 + 0x108c);
                let a2 = m.r32(hz + 0x174);
                vslot(m, a2, 50, &[sp + 0xb0], &[]);
                let t = m.rf(clock);
                m.wf(s2 + 4, t);
            }
        }
    }
    stack_pop(m, 0x1a0);
}

/// FUN_0011c500(stride, n, src, dst): Gauss-Jordan inverse of the n x n float matrix src (row stride
/// `stride` words) into dst on a heap copy (FUN_001ae478 mark / FUN_001ae318 alloc / FUN_001ae5a0 copy /
/// FUN_001ae4a0(mark, 2) release). Pivots along the row (column swap), |pivot| < 1e-4 -> 0; else 1.
pub fn ee_sub_11c500<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32) -> u32 {
    let (s4, s2, s1, s5) = (a0, a1 as i32, a2, a3);
    let sp = stack_push(m, 0xa0);
    m.call(0x001a_e478, &[sp], &[]);
    let s0 = (s2 as u32).wrapping_mul(s4) << 2;
    m.w32(sp + 0x10, s0);
    m.w32(sp + 0x14, 0x10);
    let s3 = m.call(0x001a_e318, &[sp + 0x10], &[]).0;
    m.call(0x001a_e5a0, &[s3, s1, s0], &[]);
    let n = s2.max(0) as u32;
    let at = |base: u32, r: u32, c: u32| base.wrapping_add((r.wrapping_mul(s4).wrapping_add(c)) << 2);
    for t0 in 0..n {
        for c in 0..n {
            m.wf(at(s5, t0, c), if t0 == c { 1.0 } else { 0.0 });
        }
    }
    let eps = f32::from_bits(0x38d1_b717);
    let mut ok = 1u32;
    'inv: {
        for t0 in 0..n {
            let mut p = t0;
            let mut f1 = m.rf(at(s3, t0, t0)).abs();
            for c in t0 + 1..n {
                let f2 = m.rf(at(s3, t0, c)).abs();
                if f1 < f2 {
                    f1 = f2;
                    p = c;
                }
            }
            if f1 < eps {
                ok = 0;
                break 'inv;
            }
            if p != t0 {
                for k in 0..n {
                    for base in [s3, s5] {
                        let x = m.rf(at(base, k, p));
                        let y = m.rf(at(base, k, t0));
                        m.wf(at(base, k, t0), x);
                        m.wf(at(base, k, p), y);
                    }
                }
            }
            for c in t0 + 1..n {
                let f2 = m.rf(at(s3, t0, c)) / m.rf(at(s3, t0, t0));
                for k in 0..n {
                    for base in [s3, s5] {
                        let v = m.rf(at(base, k, c)) - f2 * m.rf(at(base, k, t0));
                        m.wf(at(base, k, c), v);
                    }
                }
            }
        }
        for t0 in (0..n).rev() {
            for c in 0..t0 {
                let f2 = m.rf(at(s3, t0, c)) / m.rf(at(s3, t0, t0));
                for k in 0..n {
                    for base in [s3, s5] {
                        let v = m.rf(at(base, k, c)) - f2 * m.rf(at(base, k, t0));
                        m.wf(at(base, k, c), v);
                    }
                }
            }
        }
        for t0 in 0..n {
            let f2 = m.rf(at(s3, t0, t0));
            for k in 0..n {
                let v = m.rf(at(s5, k, t0)) / f2;
                m.wf(at(s5, k, t0), v);
            }
        }
    }
    m.call(0x001a_e4a0, &[sp, 2], &[]);
    stack_pop(m, 0xa0);
    ok
}

/// FUN_00132b00: empty.
pub fn ee_sub_132b00<M: QMem + ?Sized>(_m: &mut M) {}

/// FUN_001d8640(s): h = h * 33 + upper(c) over the bytes (ctype table 0x2aa6a1 bit 2 = lower -> c - 0x20).
pub fn ee_sub_1d8640<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let mut p = a0;
    let mut h = 0u32;
    loop {
        let c = m.r8(p) as u32;
        if c == 0 {
            return h;
        }
        p += 1;
        let up = if m.r8(0x002a_a6a1 + c) & 2 != 0 { c.wrapping_sub(0x20) } else { c };
        h = h.wrapping_mul(33).wrapping_add(up);
    }
}

/// FUN_001e5d80(o): FUN_00231100(o); +0x178 = 1, +0x148 = -1.
pub fn ee_sub_1e5d80<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0023_1100, &[a0], &[]);
    m.w32(a0 + 0x178, 1);
    m.w32(a0 + 0x148, u32::MAX);
}

/// FUN_001f0638(o): FUN_0011a380(0x3e126c, o) == 0.
pub fn ee_sub_1f0638<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    (m.call(0x0011_a380, &[0x003e_126c, a0], &[]).0 == 0) as u32
}

/// FUN_0020e040: DAT_003e1080 != 0 -> 1; else DAT_00443740 == 1.
pub fn ee_sub_20e040<M: QMem + ?Sized>(m: &mut M) -> u32 {
    if m.r32(0x003e_1080) != 0 {
        return 1;
    }
    (m.r32(0x0044_3740) == 1) as u32
}

/// FUN_001ca8e0(o): FUN_001a6178(o); with +0xe4 == 0: FUN_00149ab8(WORLD, o, pmf @0x2a77c0 (low word
/// passed), 0).
pub fn ee_sub_1ca8e0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_6178, &[a0], &[]);
    if m.r32(a0 + 0xe4) == 0 {
        let pmf = m.r32(0x002a_77c0);
        m.call(0x0014_9ab8, &[WORLD, a0, pmf, 0], &[]);
    }
}

/// FUN_001599a8 / FUN_0015c5c8 body: on event 1 for a handle held in +s / +s+4: FUN_00133448(h, o) and
/// clear the slot.
fn drop_handle<M: QMem + ?Sized>(m: &mut M, o: u32, ev: u32, h: u32, off: u32) {
    if ev != 1 {
        return;
    }
    if h == m.r32(o + off) {
        m.call(0x0013_3448, &[h, o], &[]);
        m.w32(o + off, 0);
    } else if h == m.r32(o + off + 4) {
        m.call(0x0013_3448, &[h, o], &[]);
        m.w32(o + off + 4, 0);
    }
}

/// FUN_001599a8(o, ev, h): handles +0x14c / +0x150.
pub fn ee_sub_1599a8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    drop_handle(m, a0, a1, a2, 0x14c);
}

/// FUN_0015c5c8(o, ev, h): handles +0x100 / +0x104.
pub fn ee_sub_15c5c8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    drop_handle(m, a0, a1, a2, 0x100);
}

/// FUN_00140678(jt, a1, a2, a3, t0): a2 != +0x10dc -> FUN_001431d0(jt, ..); +0x10ec = a3, +0x10dc = a2,
/// +0x10e4 = a1, +0x10e8 = t0.
pub fn ee_sub_140678<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, t0: u32) {
    if a2 != m.r32(a0 + 0x10dc) {
        m.call(0x0014_31d0, &[a0, a1, a2, a3, t0], &[]);
    }
    m.w32(a0 + 0x10ec, a3);
    m.w32(a0 + 0x10dc, a2);
    m.w32(a0 + 0x10e4, a1);
    m.w32(a0 + 0x10e8, t0);
}

/// FUN_0011cf60(a): wraps a into [-pi, pi]: outside -> FUN_0011d898(a + pi, 2 pi) - pi.
pub fn ee_sub_11cf60<M: QMem + ?Sized>(m: &mut M, f12: f32) -> f32 {
    let pi = f32::from_bits(0x4049_0fdb);
    if f12 < f32::from_bits(0xc049_0fdb) || pi < f12 {
        let r = m.call(0x0011_d898, &[], &[fv(f12 + pi), fv(f32::from_bits(0x40c9_0fdb))]).1[0];
        return r - pi;
    }
    f12
}

/// FUN_0011c318(out, m): affine inverse (VU0; same as anim::pose::ee_sub_11c318). Returns out.
pub fn ee_sub_11c318<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let a = m.rm(a1);
    let o = crate::engine::anim::pose::ee_sub_11c318(&a);
    m.wm(a0, o);
    a0
}

fn xform(mt: &[V4; 4], p: V4) -> V4 {
    let mut o = p;
    for k in 0..3 {
        o[k] = ((mt[0][k] * p[0] + mt[1][k] * p[1]) + mt[2][k] * p[2]) + mt[3][k];
    }
    o
}

/// FUN_001a6f00(from, to, p): p from `from`'s space (slot-30 matrix) to world, then into `to`'s space
/// (FUN_0011c318 inverse of its slot-30 matrix); 0 = world. Returns the quad.
pub fn ee_sub_1a6f00<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: V4) -> (u32, V4) {
    let mut p = a2;
    if a0 != 0 {
        let x = vslot(m, a0, 30, &[], &[]).0;
        let mt = m.rm(x);
        p = xform(&mt, p);
    }
    if a1 != 0 {
        let x = vslot(m, a1, 30, &[], &[]).0;
        let sp = stack_push(m, 0x70);
        m.call(0x0011_c318, &[sp, x], &[]);
        let inv = m.rm(sp);
        stack_pop(m, 0x70);
        p = xform(&inv, p);
    }
    (p[0].to_bits(), p)
}

/// FUN_00118fb0(node, lim, p): BSP point query: plane (node+8) d = n.p - w; d < 1e-4 -> front child +0
/// (returns the node when the child is not below lim), else back child +4 (0 when not below lim).
pub fn ee_sub_118fb0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: V4) -> u32 {
    let mut s0 = a0;
    if s0 == 0 {
        return 0;
    }
    let eps = f32::from_bits(0x38d1_b717);
    loop {
        let pa = m.r32(s0 + 8);
        let pl = m.rv(pa);
        let d = ((pl[0] * a2[0] + pl[1] * a2[1]) + pl[2] * a2[2]) - pl[3];
        if d < eps {
            let v1 = m.r32(s0);
            if v1 < a1 && v1 != 0 {
                s0 = v1;
            } else {
                return s0;
            }
        } else {
            let v1 = m.r32(s0 + 4);
            if v1 < a1 && v1 != 0 {
                s0 = v1;
            } else {
                return 0;
            }
        }
    }
}

/// FUN_001620b0(_, kind, n, tbl): s4 = FUN_00161fc8(kind); s2 = FUN_00161ff0 over the 2 (kinds 4 / 5,
/// @0x2a5220) or 4 (@0x2a5228) state words copied to the stack; then each 0x10 record {a, b, mask, level}
/// with mask & s2 (and level -1 or the current FUN_00210a88): o = FUN_0015e938(WORLD, b-found ? 4 : 5, a,
/// FUN_0015e938(WORLD, 5, b, 0)) gets slot 7 (mask & s4) or slot 8.
pub fn ee_sub_1620b0<M: QMem + ?Sized>(m: &mut M, _a0: u32, a1: u32, a2: u32, a3: u32) {
    let s4 = m.call(0x0016_1fc8, &[a1], &[]).0;
    let sp = stack_push(m, 0xa0);
    let s2 = if a1.wrapping_sub(4) < 2 {
        for k in 0..2 {
            let w = m.r32(0x002a_5220 + 4 * k);
            m.w32(sp + 4 * k, w);
        }
        m.call(0x0016_1ff0, &[2, sp], &[]).0
    } else {
        for k in 0..4 {
            let w = m.r32(0x002a_5228 + 4 * k);
            m.w32(sp + 0x10 + 4 * k, w);
        }
        m.call(0x0016_1ff0, &[4, sp + 0x10], &[]).0
    };
    stack_pop(m, 0xa0);
    let mut s0 = a3;
    let mut n = a2 as i32;
    while n > 0 {
        let e = s0;
        s0 = s0.wrapping_add(0x10);
        n -= 1;
        if m.r32(e + 8) & s2 == 0 {
            continue;
        }
        let lv = m.r32(e + 0xc);
        if lv != u32::MAX && lv != m.call(0x0021_0a88, &[], &[]).0 {
            continue;
        }
        let b = m.r32(e + 4);
        let p = m.call(0x0015_e938, &[WORLD, 5, b, 0], &[]).0;
        let a = m.r32(e);
        let o = m.call(0x0015_e938, &[WORLD, if p != 0 { 4 } else { 5 }, a, p], &[]).0;
        if o == 0 {
            continue;
        }
        let slot = if m.r32(e + 8) & s4 != 0 { 7 } else { 8 };
        vslot(m, o, slot, &[], &[]);
    }
}

/// FUN_0019d330(b): binoc state machine, re-run while the state (+4) changes: 0 / 2 -> clear the jt
/// +0x190 quad (0x2a2dd0), unless +0x14 and !FUN_00131968(+0x14) step FUN_0019cd38(b, +1 / -1); 1 -> clear,
/// +0x20 0 / 1: FUN_00204660(jt, 0x2ddf40+0x460 / +0x5c4), +0x3c = +0x470 / +0x5cc, FUN_0019da78(b+0x24).
/// Then with +0x20 == 0 and (jt+0x150)+0x80 == 1 its +0x10..+0x2c are zeroed.
pub fn ee_sub_19d330<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let s0 = a0;
    let s2 = 0x002d_df40u32;
    let zero = m.rv(0x002a_2dd0);
    loop {
        let s1 = m.r32(s0 + 4);
        match s1 {
            0 | 2 => {
                let v1 = m.r32(s0);
                m.wv(v1 + 0x190, zero);
                let a = m.r32(s0 + 0x14);
                if a == 0 || m.call(0x0013_1968, &[a], &[]).0 != 0 {
                    m.call(0x0019_cd38, &[s0, if s1 == 0 { 1 } else { u32::MAX }], &[]);
                }
            }
            1 => {
                let v1 = m.r32(s0);
                m.wv(v1 + 0x190, zero);
                let k = m.r32(s0 + 0x20);
                if k == 0 || k == 1 {
                    let (fa, fb) = if k == 0 { (0x460, 0x470) } else { (0x5c4, 0x5cc) };
                    let jt = m.r32(s0);
                    let f = m.rf(s2 + fa);
                    m.call(0x0020_4660, &[jt], &[fv(f)]);
                    let f0 = m.rf(s2 + fb);
                    m.wf(s0 + 0x3c, f0);
                    m.call(0x0019_da78, &[s0 + 0x24], &[]);
                }
            }
            _ => {}
        }
        if m.r32(s0 + 4) == s1 {
            break;
        }
    }
    if m.r32(s0 + 0x20) != 0 {
        return;
    }
    let jt = m.r32(s0);
    let v0 = m.r32(jt + 0x150);
    if m.r32(v0 + 0x80) == 1 {
        for off in (0x10..0x30).step_by(4) {
            m.wf(v0 + off, 0.0);
        }
    }
}

/// FUN_0013c1b8(o, ev, arg): FUN_00133590(o, ev, arg); ev 4 in state (+0x184) 5 -> FUN_0013c560(o, 0);
/// ev 8 with arg == +0x18c: state 2 -> FUN_0013c560(o, 4), state 3 -> FUN_0013c560(o, 1).
pub fn ee_sub_13c1b8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    m.call(0x0013_3590, &[a0, a1, a2], &[]);
    if a1 == 4 {
        if m.r32(a0 + 0x184) == 5 {
            m.call(0x0013_c560, &[a0, 0], &[]);
        }
    } else if a1 == 8 && a2 == m.r32(a0 + 0x18c) {
        match m.r32(a0 + 0x184) {
            2 => {
                m.call(0x0013_c560, &[a0, 4], &[]);
            }
            3 => {
                m.call(0x0013_c560, &[a0, 1], &[]);
            }
            _ => {}
        }
    }
}

/// FUN_00244e10 cosf (fdlibm s_cosf): |x| < pi/4 -> __kernel_cosf(x, 0); inf/nan -> x - x; else quadrant
/// kernels after __rem_pio2f (0x2488c8).
pub fn ee_sub_244e10<M: QMem + ?Sized>(m: &mut M, x: f32) -> f32 {
    let ix = x.to_bits() & 0x7fff_ffff;
    let k = |z: f32| [z, 0.0, 0.0, 0.0];
    if ix < 0x3f49_0fd9 {
        return m.call(0x0024_8de0, &[], &[k(x), k(0.0)]).1[0];
    }
    if ix >= 0x7f80_0000 {
        return x - x;
    }
    let sp = stack_push(m, 0x20);
    let n = m.call(0x0024_88c8, &[sp], &[k(x)]).0 & 3;
    let (y0, y1) = (m.rf(sp), m.rf(sp + 4));
    let r = match n {
        0 => m.call(0x0024_8de0, &[], &[k(y0), k(y1)]).1[0],
        1 => -m.call(0x0024_9888, &[1], &[k(y0), k(y1)]).1[0],
        2 => -m.call(0x0024_8de0, &[], &[k(y0), k(y1)]).1[0],
        _ => m.call(0x0024_9888, &[1], &[k(y0), k(y1)]).1[0],
    };
    stack_pop(m, 0x20);
    r
}

/// FUN_002458f0 (atan2f, replaced by Rust f32 math like the other libm rows): atan2(y, x).
pub fn ee_sub_2458f0<M: QMem + ?Sized>(m: &mut M, f12: f32, f13: f32) -> f32 {
    // newlib w_atan2f (WIRE-GAP 1:1): __ieee754_atan2f = FUN_00247b00; y == x == 0 -> domain error (retval 0)
    let z = m.call(0x0024_7b00, &[], &[[f12, 0.0, 0.0, 0.0], [f13, 0.0, 0.0, 0.0]]).1[0];
    if m.r32(0x002a_c138) == 0xffff_ffff
        || m.call(0x0024_a0c0, &[], &[[f13, 0.0, 0.0, 0.0]]).0 != 0
        || m.call(0x0024_a0c0, &[], &[[f12, 0.0, 0.0, 0.0]]).0 != 0
        || f13 != 0.0
        || f12 != 0.0
    {
        return z;
    }
    crate::engine::game::q_wireq8::libm_domain(m, 0x002a_b900, f12, f13, 0.0)
}

/// FUN_0011d898(x, m): fmodf(x, m), + m when negative (as player::p5_target::FUN_0011d898).
pub fn ee_sub_11d898<M: QMem + ?Sized>(m: &mut M, f12: f32, f13: f32) -> f32 {
    let r = m.call(0x0024_5a18, &[], &[[f12, 0.0, 0.0, 0.0], [f13, 0.0, 0.0, 0.0]]).1[0];
    if r < 0.0 {
        r + f13
    } else {
        r
    }
}

/// FUN_00206900(guard): unless +0x2c0 hits the cached state mask (DAT_003dd480, built once from
/// FUN_00207a20 of 5 / 6 / 0x17 / 7 / 0x10 / 1 / 0x16 / 0x2a): gathers up to 64 objects of the +0x398
/// lists (FUN_0015e9c8 kind 0x505) within the 2-D radius DAT_002c3d04 into 0x18 records {d, list, obj,
/// memo id, memo f} (memo from the last +0x3a4 entries of +0xf64), sorts them (FUN_002338c0, cmp
/// 0x2068d8), re-adds the FUN_002085d0 accepted ones; FUN_00153de0(+0xf64); then the +0x3cc target via
/// FUN_00154180: kind 1 -> FUN_0020a9a8(t, 1), kind 2 within DAT_002c3d08 (xy) / DAT_002c3d0c (|z|) -> 2.
pub fn ee_sub_206900<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let s7 = a0;
    if m.r32(0x003d_d488) == 0 {
        let mut mask = 0u64;
        for k in [5i64, 6, 0x17, 7, 0x10, 1, 0x16, 0x2a] {
            mask |= crate::engine::classes::q::q12_e::ee_sub_207a20(k) as u64;
        }
        m.w32(0x003d_d488, 1);
        w64(m, 0x003d_d480, mask);
    }
    let mask = m.r64(0x003d_d480);
    if m.r64(s7 + 0x2c0) & mask != 0 {
        return;
    }
    let pos = |m: &mut M, o: u32| m.call(0x001a_62e8, &[o], &[]).1;
    if m.r32(s7 + 0x394) != 0 {
        let sp = stack_push(m, 0xad0);
        let mut s1 = 0u32;
        let mut s0 = 0i32;
        while s0 < m.r32(s7 + 0x394) as i32 {
            let s4 = s7 + 0x398 + 4 * s0 as u32;
            s0 += 1;
            let id = m.r32(s4);
            let n = m.call(0x0015_e9c8, &[WORLD, 0x505, id, 0, 0x100, sp + 0x600], &[]).0 as i32;
            let f20 = m.rf(0x002a_8614);
            for i in 0..n.max(0) as u32 {
                let e = m.r32(sp + 0x600 + 4 * i);
                if e == s7 {
                    continue;
                }
                let p1 = pos(m, s7);
                let p2 = pos(m, e);
                let (dx, dy) = (p1[0] - p2[0], p1[1] - p2[1]);
                let d = (dx * dx + dy * dy).sqrt();
                if !(d < m.rf(0x002c_3d04)) || s1 >= 0x40 {
                    continue;
                }
                let rec = sp + 0x18 * s1;
                m.wf(rec, d);
                m.w32(rec + 0xc, u32::MAX);
                m.w32(rec + 4, s4);
                m.w32(rec + 8, e);
                m.wf(rec + 0x10, f20);
                s1 += 1;
                let t = m.r32(s7 + 0xf64);
                let mut j = (m.r32(t + 0xfc) as i32).wrapping_sub(m.r32(s7 + 0x3a4) as i32);
                loop {
                    let t = m.r32(s7 + 0xf64);
                    if j >= m.r32(t + 0xfc) as i32 {
                        break;
                    }
                    let ent = t.wrapping_add(0x100 + 16 * j as u32);
                    if m.r32(ent) == e {
                        let id = m.r32(ent + 4);
                        m.w32(rec + 0xc, id);
                        let f = m.rf(ent + 8);
                        m.wf(rec + 0x10, f);
                        break;
                    }
                    j += 1;
                }
            }
        }
        m.call(0x0023_38c0, &[sp, s1, 0x18, 0x0020_68d8], &[]);
        let t = m.r32(s7 + 0xf64);
        let c = m.r32(t + 0xfc).wrapping_sub(m.r32(s7 + 0x3a4));
        m.w32(t + 0xfc, c);
        m.w32(s7 + 0x3a4, 0);
        for k in 0..s1 {
            let rec = sp + 0x18 * k;
            let (obj, lst) = (m.r32(rec + 8), m.r32(rec + 4));
            if m.call(0x0020_85d0, &[s7, obj, lst], &[]).0 == 0 {
                continue;
            }
            let n = m.r32(s7 + 0x3a4) + 1;
            m.w32(s7 + 0x3a4, n);
            let id = m.r32(rec + 0xc);
            if id != u32::MAX {
                let t = m.r32(s7 + 0xf64);
                let e = t.wrapping_add((m.r32(t + 0xfc) << 4) + 0xf0);
                m.w32(e + 4, id);
                let f = m.rf(rec + 0x10);
                m.wf(e + 8, f);
            }
        }
        stack_pop(m, 0xad0);
    }
    let t = m.r32(s7 + 0xf64);
    m.call(0x0015_3de0, &[t], &[]);
    let v = m.r32(s7 + 0x3cc);
    if v == 0 {
        return;
    }
    let t = m.r32(s7 + 0xf64);
    let tg = m.r32(v + 0xc);
    let r = m.call(0x0015_4180, &[t, tg], &[]).0;
    if r == 0 {
        return;
    }
    match m.r32(r + 4) {
        1 => {
            let v = m.r32(s7 + 0x3cc);
            m.call(0x0020_a9a8, &[v, 1], &[]);
        }
        2 => {
            let v = m.r32(s7 + 0x3cc);
            let tg = m.r32(v + 0xc);
            let p1 = pos(m, tg);
            let p2 = pos(m, s7);
            let d = [p1[0] - p2[0], p1[1] - p2[1], p1[2] - p2[2]];
            let dist = (d[0] * d[0] + d[1] * d[1]).sqrt();
            if dist < m.rf(0x002c_3d08) && d[2].abs() < m.rf(0x002c_3d0c) {
                let v = m.r32(s7 + 0x3cc);
                m.call(0x0020_a9a8, &[v, 2], &[]);
            }
        }
        _ => {}
    }
}

/// FUN_001f0d88(sma, a1): state-machine advance loop (anim::sm::Sma): while not done (FUN_001f0638) and
/// no transition is playing (+0x30): pick the transition from +0x34 toward goal +0x3c (FUN_001f0b80, or
/// FUN_001f0cf0 for a -2 "any" link of the +0x24 state table, ending at a -1 link) into +0x40; start its
/// motion (FUN_0012f9b8(trans, +0x28, flags, &+0x30, start, speed)); an instant one (zero length, or a1 == 1
/// on the first pass) switches the state (FUN_00130b18 old motion, FUN_001f13a8(sma, old, new)); otherwise
/// FUN_001333c0(motion, sma) waits on it.
pub fn ee_sub_1f0d88<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let s2 = a0;
    let mut s3 = a1;
    if m.call(0x001f_0638, &[s2], &[]).0 != 0 {
        return;
    }
    loop {
        // L3c
        let a2 = m.r32(s2 + 0x34);
        let a3 = m.r32(s2 + 0x3c);
        let s0;
        if a2 != a3 {
            let (t, g) = (m.r32(s2 + 0x24), m.r32(s2 + 0x48));
            s0 = m.call(0x001f_0b80, &[t, g, a2, a3, 1], &[]).0;
        } else {
            let a0 = m.r32(s2 + 0x24);
            let tbl = m.r32(a0 + 0x54);
            let v0 = m.r32(tbl.wrapping_add(a2.wrapping_mul(8)) + 4);
            if v0 == u32::MAX {
                return;
            }
            let a1 = m.r32(s2 + 0x48);
            s3 = 0;
            if v0 == 0xffff_fffe {
                s0 = m.call(0x001f_0cf0, &[a0, a1, a2, 0], &[]).0;
                if s0 != 0 {
                    let mut x = m.r32(s0 + 4);
                    if x == m.r32(s2 + 0x34) {
                        x = m.r32(s0 + 8);
                    }
                    m.w32(s2 + 0x3c, x);
                }
            } else {
                m.w32(s2 + 0x3c, v0);
                s0 = m.call(0x001f_0b80, &[a0, a1, a2, v0, 1], &[]).0;
            }
        }
        // L18
        m.w32(s2 + 0x40, s0);
        if s0 == 0 {
            return;
        }
        let mut s1 = 0u32;
        let tr = m.r32(s0);
        if tr != 0 {
            let v1 = m.r32(s0 + 4);
            let (mut f12, f13);
            if v1 != m.r32(s2 + 0x34) {
                m.w32(s2 + 0x38, v1);
                f13 = -m.rf(s2 + 0x44);
                f12 = m.rf(tr + 0x74);
            } else {
                let x = m.r32(s0 + 8);
                m.w32(s2 + 0x38, x);
                f12 = 0.0;
                f13 = m.rf(s2 + 0x44);
            }
            let mut a2 = m.r32(s2 + 0x2c);
            if s3 == 1 {
                a2 = (a2 | 2) & !1;
                s1 = 1;
                f12 = m.rf(tr + 0x74) - f12;
            } else if m.rf(tr + 0x74) == 0.0 {
                s1 = 1;
            }
            let a1 = m.r32(s2 + 0x28);
            m.call(0x0012_f9b8, &[tr, a1, a2, s2 + 0x30], &[fv(f12), fv(f13)]);
        }
        let a0 = m.r32(s2 + 0x30);
        if a0 == 0 {
            let a1 = m.r32(s2 + 0x34);
            let mut v0 = m.r32(s0 + 4);
            if v0 == a1 {
                v0 = m.r32(s0 + 8);
            }
            m.w32(s2 + 0x34, v0);
            m.call(0x001f_13a8, &[s2, a1, v0], &[]);
        } else if s1 != 0 {
            let s1 = m.r32(s2 + 0x38);
            let old = m.r32(s2 + 0x34);
            m.w32(s2 + 0x34, s1);
            m.w32(s2 + 0x38, u32::MAX);
            m.call(0x0013_0b18, &[a0], &[]);
            m.w32(s2 + 0x30, 0);
            m.call(0x001f_13a8, &[s2, old, s1], &[]);
        } else {
            m.call(0x0013_33c0, &[a0, s2], &[]);
            return;
        }
        if m.call(0x001f_0638, &[s2], &[]).0 != 0 {
            return;
        }
        if m.r32(s2 + 0x30) != 0 {
            return;
        }
    }
}

/// FUN_00134850(jt): per-state input dispatcher, re-run while +0x13d0 changes. Pre: states 5 / 7 / 12 /
/// out of 2..14 (and 3 unless +0x4c8 is 1 / 12) cancel the +0x2c0 action after 0.1 s since +0x20c
/// (FUN_00195a00). Then by state: 0 / 1 / 7 ground, 5 air (button tests on the pad +0x150 via FUN_001c46c0
/// / FUN_001c4660, slot 98 gadget checks) -> FUN_00195a00 / FUN_0019cc50 / gadget FUN_00136a28 + state 6
/// / FUN_001938c0 / FUN_00135248 transitions; 2 / 3 / 4 / 6 / 8 / 12 / 13 per-state updates; 10 ends
/// on +0x1030 / +0x1034 completion; 14 times out after DAT_002bc930. FUN_00136338 after each pass.
pub fn ee_sub_134850<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    #[derive(PartialEq)]
    enum L {
        End,
        B34,
        B88,
        C24,
        Bcc,
        Cc8,
        Be0,
    }
    let s0 = a0;
    let clock = 0x002d_ded0u32;
    let f01 = f32::from_bits(0x3dcc_cccd);
    let f015 = f32::from_bits(0x3e19_999a);
    let f02 = f32::from_bits(0x3e4c_cccd);
    let pad = |m: &mut M| m.r32(s0 + 0x150);
    let held = |m: &mut M, b: u32, f: f32| {
        let p = m.r32(s0 + 0x150);
        m.call(0x001c_46c0, &[p, b], &[fv(f)]).0 != 0
    };
    let press = |m: &mut M, b: u32, f: f32| {
        let p = m.r32(s0 + 0x150);
        m.call(0x001c_4660, &[p, b], &[fv(f)]).0 != 0
    };
    let mut s1 = m.r32(s0 + 0x13d0);
    loop {
        let timed = match s1 {
            3 => {
                let v1 = m.r32(s0 + 0x4c8);
                v1 != 1 && v1 != 0xc
            }
            5 | 7 | 12 => true,
            2..=14 => false,
            _ => true,
        };
        if timed && f01 < m.rf(clock) - m.rf(s0 + 0x20c) {
            m.call(0x0019_5a00, &[s0 + 0x2c0, 0, 0, 0], &[]);
        }
        let st = m.r32(s0 + 0x13d0);
        let mut l = L::End;
        match st {
            0 | 1 | 7 => 'a: {
                let p = pad(m);
                if m.r32(p + 0x30) & 8 == 0 && m.call(0x0014_0af8, &[s0, 0], &[]).0 != 0 {
                    m.call(0x0013_5248, &[s0, 5], &[]);
                    break 'a;
                }
                if held(m, 0x40, f01) {
                    l = L::B34;
                    break 'a;
                }
                if press(m, 0x400, f015) && vslot(m, s0, 98, &[3], &[]).0 != 0 {
                    l = L::B88;
                    break 'a;
                }
                if press(m, 0x10, f015) {
                    let g = m.r32(s0 + 0x1870);
                    if vslot(m, s0, 98, &[g], &[]).0 != 0 {
                        l = L::C24;
                        break 'a;
                    }
                }
                if held(m, 0x20, f02) {
                    l = L::Bcc;
                    break 'a;
                }
                let p = pad(m);
                let f1 = m.rf(p + 0x1c);
                let go = if m.r32(s0 + 0x13d0) == 0 { f01 < f1 } else { 0.0 < f1 };
                if !go {
                    l = L::Cc8;
                    break 'a;
                }
                let k = if m.r32(p + 0x30) & 8 != 0 { 1 } else { 7 };
                m.call(0x0013_5248, &[s0, k], &[]);
            }
            2 => {
                m.call(0x0019_7500, &[s0 + 0x2c0], &[]);
                l = L::Be0;
            }
            3 => {
                m.call(0x0019_11a8, &[s0 + 0x4c0], &[]);
            }
            4 => {
                m.call(0x001a_12f0, &[s0 + 0xdf0], &[]);
            }
            5 => 'b: {
                let p = pad(m);
                if m.r32(p + 0x30) & 8 != 0 || m.call(0x0014_0af8, &[s0, 1], &[]).0 == 0 {
                    l = L::Cc8;
                    break 'b;
                }
                if held(m, 0x40, f01) {
                    l = L::B34;
                    break 'b;
                }
                if press(m, 0x400, f015) && vslot(m, s0, 98, &[3], &[]).0 != 0 {
                    l = L::B88;
                    break 'b;
                }
                if held(m, 0x20, f02) {
                    l = L::Bcc;
                    break 'b;
                }
                l = L::Be0;
            }
            6 => {
                m.call(0x0013_6118, &[s0], &[]);
            }
            8 => {
                m.call(0x0013_67d8, &[s0], &[]);
            }
            10 => 'c: {
                let a = m.r32(s0 + 0x1030);
                if a != 0 && m.call(0x0013_1968, &[a], &[]).0 != 0 {
                    l = L::Cc8;
                    break 'c;
                }
                let a = m.r32(s0 + 0x1034);
                if a != 0 && m.call(0x001f_0638, &[a], &[]).0 != 0 {
                    l = L::Cc8;
                }
            }
            12 => {
                m.call(0x0021_f418, &[s0 + 0x11b0], &[]);
            }
            13 => {
                m.call(0x0019_e1e0, &[s0 + 0xf50], &[]);
            }
            14 => {
                if m.rf(0x002b_c930) < m.rf(clock) - m.rf(s0 + 0x13d4) {
                    l = L::Cc8;
                }
            }
            _ => {}
        }
        if l == L::Be0 && press(m, 0x10, f015) {
            let g = m.r32(s0 + 0x1870);
            if vslot(m, s0, 98, &[g], &[]).0 != 0 {
                l = L::C24;
            }
        }
        match l {
            L::B34 => {
                m.call(0x0019_5a00, &[s0 + 0x2c0, 1, 0, 0], &[]);
            }
            L::B88 => {
                let p = pad(m);
                m.call(0x001c_4738, &[p, 0x400], &[]);
                m.call(0x0019_cc50, &[s0 + 0xf00, 0, 0, 0], &[]);
            }
            L::C24 => {
                let p = pad(m);
                m.call(0x001c_4738, &[p, 0x10], &[]);
                let g = m.r32(s0 + 0x1870);
                let v = m.call(0x0013_6a28, &[s0, g], &[]).0;
                m.w32(s0 + 0x16b4, v);
                m.call(0x0013_5248, &[s0, 6], &[]);
            }
            L::Bcc => {
                m.call(0x0019_38c0, &[s0 + 0x4c0, 0xf8fe, 0], &[]);
            }
            L::Cc8 => {
                m.call(0x0013_5248, &[s0, 0], &[]);
            }
            L::End | L::Be0 => {}
        }
        m.call(0x0013_6338, &[s0], &[]);
        let v = m.r32(s0 + 0x13d0);
        if v == s1 {
            return;
        }
        s1 = v;
    }
}

/// FUN_00161fc8(k): -1 -> 0; else (u32)(1 << (k & 63)) (dsllv, low word).
pub fn ee_sub_161fc8<M: QMem + ?Sized>(_m: &mut M, a0: u32) -> u32 {
    if a0 == u32::MAX {
        return 0;
    }
    (1u64 << (a0 & 63)) as u32
}

/// FUN_00161ff0(n, ids): OR of FUN_00161fc8 over the n words.
pub fn ee_sub_161ff0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let mut r = 0;
    for i in 0..(a0 as i32).max(0) as u32 {
        let k = m.r32(a1 + 4 * i);
        r |= m.call(0x0016_1fc8, &[k], &[]).0;
    }
    r
}

/// FUN_00231440(o): FUN_00231368(o); returns o + 0xa0.
pub fn ee_sub_231440<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    m.call(0x0023_1368, &[a0], &[]);
    a0 + 0xa0
}
