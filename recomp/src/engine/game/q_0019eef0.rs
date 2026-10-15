//! WP-Q5: run_0019eef0 queue package: player-side "recent contacts" ring (+0x9e0: count +4, 0x20-byte
//! entries from +0x10, at most 0x20) fed by the ground / grab / rail states (FUN_0019fc30), the
//! drop-landing danger estimate FUN_0019f7d0 and its target-position helper FUN_0019f3f8, plus small
//! glue (FUN_0019eef0 revoke check, FUN_0019f0a8 broadcast). 1:1 against `QMem`.
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};
use crate::engine::classes::q::q5util::{dist3, f4};

const CLOCK: u32 = 0x002d_ded0;

/// libc memmove (FUN_002332b8) on emulated memory
fn mmove<M: QMem + ?Sized>(m: &mut M, dst: u32, src: u32, n: u32) {
    if dst > src {
        for i in (0..n).rev() {
            let b = m.r8(src + i);
            m.w8(dst + i, b);
        }
    } else {
        for i in 0..n {
            let b = m.r8(src + i);
            m.w8(dst + i, b);
        }
    }
}

fn zero<M: QMem + ?Sized>(m: &mut M, a: u32, n: u32) {
    for i in 0..n {
        m.w8(a + i, 0);
    }
}

/// FUN_0019eef0: message 3 with a +0x12f0 owner and DAT_002dfbc0 bit 1: Revoke(0x2de2e0) == 0
pub fn ee_sub_19eef0<M: QMem + ?Sized>(m: &mut M, p: u32, msg: u32) -> bool {
    if msg != 3 {
        return false;
    }
    let o = m.r32(p);
    if m.r32(o + 0x12f0) == 0 || m.r32(0x002d_fbc0) & 2 == 0 {
        return false;
    }
    m.call(0x0014_fd50, &[0x002d_e2e0], &[]).0 == 0
}

/// FUN_0019f0a8: every class-5 / flag-0x34 object (FUN_0015e9c8 on 0x3dd490, at most 8) gets slot 103
pub fn ee_sub_19f0a8<M: QMem + ?Sized>(m: &mut M) {
    let sp = stack_push(m, 0x20);
    let n = m.call(0x0015_e9c8, &[0x003d_d490, 5, 0x34, 0, 8, sp], &[]).0 as i32;
    for i in 0..n.max(0) as u32 {
        let o = m.r32(sp + 4 * i);
        vslot(m, o, 103, &[], &[]);
    }
    stack_pop(m, 0x20);
}

/// FUN_0019f3f0: empty
pub fn ee_sub_19f3f0() {}

/// FUN_0019f3f8: world target of contact `p` {owner, attach, frame-holder, .., +0x10 local quad / t}
/// at time `t`: with an attachment, its vtable (+0x20) slot +0x1c evaluates the local point (time
/// +0x10) into a frame transformed by FUN_0022fc48 into the owner's (slot +0x104) space; else the
/// holder's (*+8) object and the stored quad. With an object and t > 0 the point goes through the
/// FUN_002309e8 matrix (w from its row 2), else FUN_0022fc48(obj, 0, q). Without an attachment the
/// result is relative to `s`+0x240.
pub fn ee_sub_19f3f8<M: QMem + ?Sized>(m: &mut M, t: f32, s: u32, p: u32) -> V4 {
    let sp = stack_push(m, 0xb0);
    let at = m.r32(p + 4);
    let (obj, q);
    if at != 0 {
        let vt = m.r32(at + 0x20);
        let adj = m.r16(vt + 0x18) as i16 as i32 as u32;
        let f = m.r32(vt + 0x1c);
        let tm = m.rf(p + 0x10);
        m.call(f, &[at.wrapping_add(adj), s, sp, 0], &[f4(tm)]);
        let o = m.r32(p);
        let vt = m.r32(o + 0xc);
        let adj = m.r16(vt + 0x100) as i16 as i32 as u32;
        let f = m.r32(vt + 0x104);
        obj = m.call(f, &[o.wrapping_add(adj), 0], &[]).0;
        let l = m.rv(sp);
        q = m.call(0x0022_fc48, &[0, obj], &[l]).1;
    } else {
        let h = m.r32(p + 8);
        obj = if h != 0 { m.r32(h) } else { 0 };
        q = m.rv(p + 0x10);
    }
    let mut r;
    if obj != 0 && 0.0 < t {
        m.call(0x0023_09e8, &[obj, 0, sp + 0x10, 0], &[f4(t)]);
        let mm = m.rm(sp + 0x10);
        r = mm[2];
        for c in 0..3 {
            r[c] = mm[0][c] * q[0] + mm[1][c] * q[1] + mm[2][c] * q[2] + mm[3][c];
        }
    } else {
        r = m.call(0x0022_fc48, &[obj, 0], &[q]).1;
    }
    if m.r32(p + 4) == 0 {
        let b = m.rv(s + 0x240);
        for c in 0..4 {
            r[c] -= b[c];
        }
    }
    stack_pop(m, 0xb0);
    r
}

/// FUN_0019f7d0: landing risk for contact `p2` of player *`p1`: probe the floor (FUN_0020fc60 gather +
/// FUN_0014cc58 segment casts) under the target: no floor / steep (+0x28 < 0.7) / no +0x40 -> 2000,
/// blocked from above -> at least 2000, any of the 8 DAT_002bf950 offsets without floor -> at least
/// 1500; then the max with the clamped DAT_002bf9d0 curve of the distance to each other player in
/// `others[..n]` (their +0x218 position)
pub fn ee_sub_19f7d0<M: QMem + ?Sized>(m: &mut M, p1: u32, p2: u32, n: i32, others: u32) -> f32 {
    let mut r = 0.0f32;
    let me = m.r32(p1);
    let mut q = ee_sub_19f3f8(m, 0.0, me, p2);
    if m.r32(p2 + 8) != 0 {
        let sp = stack_push(m, 0x140);
        let (a, b, hit, nb, lb) = (sp + 0x10, sp + 0x20, sp + 0x30, sp + 0x90, sp + 0x94);
        m.call(0x001a_e478, &[sp], &[]);
        let me = m.r32(p1);
        let ec = m.r32(me + 0xec);
        let k = m.rf(0x002b_f944);
        m.call(0x0020_fc60, &[0x003d_d490, 0x0020_f948, ec, nb, lb], &[f4(k), q]);
        let cast = |m: &mut M, q: V4, up: f32, down: f32| {
            m.wv(a, [q[0], q[1], q[2] + up, q[3]]);
            m.wv(b, [q[0], q[1], q[2] + down, q[3]]);
            let (n, l) = (m.r32(nb), m.r32(lb));
            let (va, vb) = (m.rv(a), m.rv(b));
            m.call(0x0014_cc58, &[0, n, l, hit], &[va, vb]).0
        };
        if cast(m, q, 125.0, -175.0) == 0 || m.rf(hit + 0x40) == 0.0 || m.rf(hit + 0x28) < 0.7 {
            r = 2000.0;
        } else {
            q = m.rv(hit);
        }
        if cast(m, q, 180.0, 5.0) != 0 && r < 2000.0 {
            r = 2000.0;
        }
        for i in 0..8u32 {
            let o = m.rv(0x002b_f950 + 0x10 * i);
            let c = [q[0] + o[0], q[1] + o[1], q[2] + o[2], q[3] + o[3]];
            if cast(m, c, 50.0, -30.0) == 0 {
                if r < 1500.0 {
                    r = 1500.0;
                }
                break;
            }
        }
        m.call(0x001a_e4a0, &[sp, 2], &[]);
        stack_pop(m, 0x140);
    }
    for i in 0..n.max(0) as u32 {
        let o = m.r32(others + 4 * i);
        if o != m.r32(p1) {
            let pp = m.r32(o + 0x218);
            let d = dist3(m.rv(pp), q);
            let v = m.rf(0x002b_f9d0) + d * (m.rf(0x002b_f9d4) + d * m.rf(0x002b_f9d8));
            let (lo, hi) = (m.rf(0x002b_f9e0), m.rf(0x002b_f9e4));
            let c = if lo <= v { if hi < v { hi } else { v } } else { lo };
            if r < c {
                r = c;
            }
        }
    }
    r
}

/// FUN_0019fc30: record the current contact by player state (+0x18c8): ground states take the
/// +0x208 surface (normal z > 0.7, its +0x4c object without flag bit 29, not +0x108c) through
/// FUN_0019feb0; rail states 3 / 0x16 the +0x18c4 rail (unless +0x10c) through FUN_0019fd30; 7 / 8
/// forward to FUN_001942f0(+0x4c0) / FUN_0019ea60(+0xf50)
pub fn ee_sub_19fc30<M: QMem + ?Sized>(m: &mut M, s: u32) {
    match m.r32(s + 0x18c8) {
        0 | 1 | 5 | 6 | 9 | 10 | 0xb | 0xc | 0xf | 0x10 | 0x11 | 0x14 | 0x17 | 0x18 => {
            let g = m.r32(s + 0x208);
            if g != 0 && 0.7 < m.rf(g + 0x88) {
                let o = m.r32(g + 0x4c);
                if m.r64(o + 0x148) & 0x2000_0000 == 0 && m.r32(s + 0x108c) == 0 {
                    let x = m.r32(g + 0x40);
                    let q = m.rv(g + 0x60);
                    ee_sub_19feb0(m, s + 0x9e0, x, q);
                }
            }
        }
        3 | 0x16 => {
            let r = m.r32(s + 0x18c4);
            if m.r32(r + 0x10c) == 0 {
                let a = if r == 0 { 0 } else { r + 0x100 };
                ee_sub_19fd30(m, 0.0, s + 0x9e0, a);
            }
        }
        7 => {
            m.call(0x0019_42f0, &[s + 0x4c0], &[]);
        }
        8 => {
            m.call(0x0019_ea60, &[s + 0xf50], &[]);
        }
        _ => {}
    }
}

/// shared tail of FUN_0019fd30 / FUN_0019feb0: entry `i` (>= 1) found -> move it to slot 1 (entries
/// 1..i shift up); none -> push everything down one (count + 1, at most 0x20); clear entry 0
fn make_room<M: QMem + ?Sized>(m: &mut M, r: u32, found: Option<u32>) {
    match found {
        Some(1) => {}
        Some(i) => {
            let e = r + 0x10 + 0x20 * i;
            let mut save = [0u8; 0x20];
            for (k, b) in save.iter_mut().enumerate() {
                *b = m.r8(e + k as u32);
            }
            mmove(m, r + 0x50, r + 0x30, 0x20 * (i - 1));
            for (k, b) in save.iter().enumerate() {
                m.w8(r + 0x30 + k as u32, *b);
            }
        }
        None => {
            let n = m.r32(r + 4);
            if n < 0x20 {
                m.w32(r + 4, n + 1);
            }
            let n = m.r32(r + 4);
            mmove(m, r + 0x30, r + 0x10, (n * 0x20).wrapping_sub(0x20));
        }
    }
    zero(m, r + 0x10, 0x20);
}

/// FUN_0019fd30: contact with rail `a` at parameter `t`: an older entry for it within 50 of `t` moves
/// to slot 1; entry 0 = {slot +0x34 object, a, clock, t}, the object's +0x28 bit 59 set
pub fn ee_sub_19fd30<M: QMem + ?Sized>(m: &mut M, t: f32, r: u32, a: u32) {
    let mut found = None;
    let mut i = 1u32;
    while (i as i32) < m.r32(r + 4) as i32 {
        let e = r + 0x10 + 0x20 * i;
        if a == m.r32(e + 4) && (t - m.rf(e + 0x10)).abs() < 50.0 {
            found = Some(i);
            break;
        }
        i += 1;
    }
    make_room(m, r, found);
    let vt = m.r32(a + 0x20);
    let adj = m.r16(vt + 0x30) as i16 as i32 as u32;
    let f = m.r32(vt + 0x34);
    let o = m.call(f, &[a.wrapping_add(adj)], &[]).0;
    m.w32(r + 0x10, o);
    m.w32(r + 0x14, a);
    m.wf(r + 0x20, t);
    let c = m.r32(CLOCK);
    m.w32(r + 0x1c, c);
    let o = m.r32(r + 0x10);
    let x = m.r64(o + 0x28) | 0x0800_0000_0000_0000;
    m.w32(o + 0x28, x as u32);
    m.w32(o + 0x2c, (x >> 32) as u32);
}

/// FUN_0019feb0: contact with surface `g` at `q`: an older entry for it within 50 moves to slot 1;
/// entry 0 = {g+4 object, g, clock, q}, g's +0x10 bit 51 set
pub fn ee_sub_19feb0<M: QMem + ?Sized>(m: &mut M, r: u32, g: u32, q: V4) {
    let mut found = None;
    let mut i = 1u32;
    while (i as i32) < m.r32(r + 4) as i32 {
        let e = r + 0x10 + 0x20 * i;
        if g == m.r32(e + 8) && dist3(q, m.rv(e + 0x10)) < 50.0 {
            found = Some(i);
            break;
        }
        i += 1;
    }
    make_room(m, r, found);
    if g != 0 {
        let o = m.r32(g + 4);
        m.w32(r + 0x10, o);
        m.w32(r + 0x18, g);
        let x = m.r64(g + 0x10) | 0x0008_0000_0000_0000;
        m.w32(g + 0x10, x as u32);
        m.w32(g + 0x14, (x >> 32) as u32);
    }
    m.wv(r + 0x20, q);
    let c = m.r32(CLOCK);
    m.w32(r + 0x1c, c);
}

/// FUN_001a0048: compact ring `r`, keeping the entries whose object is not `o` and whose surface is
/// not `g` (a 0 filter matches nothing)
pub fn ee_sub_1a0048<M: QMem + ?Sized>(m: &mut M, r: u32, o: u32, g: u32) {
    let end = r + 0x10 + m.r32(r + 4).wrapping_mul(0x20);
    let mut w = r + 0x10;
    let mut e = w;
    while e < end {
        if (o == 0 || o != m.r32(e)) && (g == 0 || g != m.r32(e + 8)) {
            if e != w {
                mmove(m, w, e, 0x20);
            }
            w += 0x20;
        }
        e += 0x20;
    }
    m.w32(r + 4, ((w.wrapping_sub(0x10).wrapping_sub(r)) as i32 >> 5) as u32);
}

/// FUN_001a00d0: forget object `o` / surface `g` in the three players' rings (DAT_002e1e40,
/// DAT_002dd5bc, DAT_002f7900 + 0x9e0) and clear their contact bits (o+0x28 bit 59, g+0x10 bit 51)
pub fn ee_sub_1a00d0<M: QMem + ?Sized>(m: &mut M, _a0: u32, o: u32, g: u32) {
    for p in [0x002e_1e40u32, 0x002d_d5bc, 0x002f_7900] {
        let x = m.r32(p);
        if x != 0 {
            ee_sub_1a0048(m, x + 0x9e0, o, g);
        }
    }
    if o != 0 {
        let x = m.r64(o + 0x28) & 0xf7ff_ffff_ffff_ffff;
        m.w32(o + 0x28, x as u32);
        m.w32(o + 0x2c, (x >> 32) as u32);
    }
    if g != 0 {
        let x = m.r64(g + 0x10) & 0xfff7_ffff_ffff_ffff;
        m.w32(g + 0x10, x as u32);
        m.w32(g + 0x14, (x >> 32) as u32);
    }
}
