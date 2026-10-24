//! WP-Q2: class C2caf88 (world class 86) queue package: the ALO / SO core vtable slots the DB rooted at
//! C2caf88 that had no port_tracking row (K5's actor/alo.rs models the high-level state; these are the
//! raw-layout bodies), ported 1:1 against `QMem`. Layout: +0xc vtable, +0x10 parent, +0x24 next sibling,
//! +0x28 flags (u64: bit 56 frozen-root, 57 / 58 skip), +0x54 local xform (0x40 matrix + 0xa4 stamp),
//! +0x58 motion, +0x5c physics, +0x60 world matrix owner, +0x64 first child, +0x70 child-list sentinel,
//! +0xe0 / +0xe8 / +0xec collision / physics / list record, +0x134 pose override, +0x148 flags.
//! Out-of-package callees and parent (C2c94e0 / ALO) bodies go through `QMem::call` by address.
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

/// DAT_003e11ac: active physics record list; DAT_003e11e8: xform stamp; DAT_003e159c: clone list
pub const PHYS_LIST: u32 = 0x003e_11ac;
pub const XF_STAMP: u32 = 0x003e_11e8;
pub const CLONE_LIST: u32 = 0x003e_159c;
/// &DAT_003dd490: the world
pub const WORLD: u32 = 0x003d_d490;

fn bit<M: QMem + ?Sized>(m: &mut M, o: u32, off: u32, b: u32) -> bool {
    (m.r64(o + off) >> b) & 1 != 0
}

/// children of `obj` (+0x64, linked by +0x24) up to `end` (0 = null-terminated, else the +0x70 sentinel)
fn children<M: QMem + ?Sized>(m: &mut M, obj: u32, end: u32) -> Vec<u32> {
    let mut v = Vec::new();
    let mut c = m.r32(obj + 0x64);
    while c != end && c != 0 {
        v.push(c);
        c = m.r32(c + 0x24);
    }
    v
}

/// FUN_0012e3f0: allocate from the pool `*a0` (FUN_0012e238) and run the new object's init (descriptor
/// at +0x18, slot word +0x34 / this-adjust +0x30) with (a0, a1, a2).
pub fn ee_sub_12e3f0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) -> u32 {
    let pool = m.r32(a0);
    let u = m.call(0x0012_e238, &[pool], &[]).0;
    let d = m.r32(u + 0x18);
    let f = m.r32(d + 0x34);
    let adj = m.r16(d + 0x30) as i16 as i32 as u32;
    m.call(f, &[u.wrapping_add(adj), a0, a1, a2], &[]);
    u
}

/// FUN_001336f0: new 0x60-byte watch record for `obj` (pool world +0x3ce0, list +0x34 inited, +0x60 obj)
/// on world +0x3d10 (and +0x3d1c when obj +0x14 is in the world's +0x2c set); two 0xc-byte hooks
/// {0, rec, rec + 4k, 1, rec, rec + 0x10 + 4k} appended at world +0x3cd4 / +0x3cd8 / +0x3cdc [+0x3cd0], count += 2.
pub fn ee_sub_1336f0<M: QMem + ?Sized>(m: &mut M, w: u32, obj: u32) -> u32 {
    let r = m.call(0x0011_ccc0, &[w + 0x3ce0], &[]).0;
    m.call(0x0011_a1f8, &[r + 0x34, 0x60], &[]);
    m.w32(r + 0x60, obj);
    m.call(0x0011_a220, &[w + 0x3d10, r], &[]);
    let id = m.r32(obj + 0x14);
    if m.call(0x0011_a380, &[w + 0x2c, id], &[]).0 != 0 {
        m.call(0x0011_a220, &[w + 0x3d1c, r], &[]);
    }
    // asm: a3 = 2, loop `a3 -= 1 .. bgez a3` = 3 passes (hook arrays +0x3cd4 / +0x3cd8 / +0x3cdc)
    for k in 0..3u32 {
        let n = m.r32(w + 0x3cd0);
        let h = m.r32(w + 0x3cd4 + k * 4) + n * 0xc;
        m.w32(h + 8, r + k * 4);
        m.w32(h, 0);
        m.w32(h + 4, r);
        m.w32(h + 0x14, r + 0x10 + k * 4);
        m.w32(h + 0xc, 1);
        m.w32(h + 0x10, r);
    }
    let n = m.r32(w + 0x3cd0);
    m.w32(w + 0x3cd0, n + 2);
    r
}

/// FUN_001a6dc8 (slot 45): +0x2d = `b`, FUN_001a6d28(child, b) for every child without flag bit 58
pub fn ee_sub_1a6dc8<M: QMem + ?Sized>(m: &mut M, obj: u32, b: u32) {
    m.w8(obj + 0x2d, b as u8);
    for c in children(m, obj, 0) {
        if !bit(m, c, 0x28, 58) {
            m.call(0x001a_6d28, &[c, b], &[]);
        }
    }
}

/// FUN_001ca308 (slot 24): parent FUN_001ca118, then slot 24(a, 0) on every child
pub fn ee_sub_1ca308<M: QMem + ?Sized>(m: &mut M, obj: u32, a: u32) {
    m.call(0x001c_a118, &[obj, a], &[]);
    for c in children(m, obj, 0) {
        vslot(m, c, 24, &[a, 0], &[]);
    }
}

/// FUN_001ed850: clone `p`: descriptor = slot 5, new = desc +0x10(), copy desc +0xc bytes, re-init the
/// lists +0x34 / +0x40, append to DAT_003e159c.
pub fn ee_sub_1ed850<M: QMem + ?Sized>(m: &mut M, p: u32) -> u32 {
    let d = vslot(m, p, 5, &[], &[]).0;
    let f = m.r32(d + 0x10);
    let u = m.call(f, &[], &[]).0;
    let n = m.r32(d + 0xc);
    m.call(0x001a_e5a0, &[u, p, n], &[]);
    m.call(0x0011_a060, &[u + 0x34], &[]);
    m.call(0x0011_a058, &[u + 0x40], &[]);
    m.call(0x0011_a070, &[CLONE_LIST, u], &[]);
    u
}

/// FUN_001f2e78 (slot 12): parent FUN_001a5b80, slot 12(a, b) on every child; then, with collision
/// (+0xe0) and not frozen (bit 56), when `a` is obj or an unfrozen ancestor and neither `b` nor obj's
/// parent leads to a frozen ancestor, FUN_001f2ba8 + FUN_001f2b20.
pub fn ee_sub_1f2e78<M: QMem + ?Sized>(m: &mut M, obj: u32, a: u32, b: u32) {
    m.call(0x001a_5b80, &[obj, a, b], &[]);
    for c in children(m, obj, 0) {
        vslot(m, c, 12, &[a, b], &[]);
    }
    if m.r32(obj + 0xe0) == 0 || bit(m, obj, 0x28, 56) {
        return;
    }
    let mut hit = obj == a;
    if !hit {
        // the original walks without a null check (a root always carries bit 56); stop at 0 here
        let mut p = m.r32(obj + 0x10);
        while p != 0 && !bit(m, p, 0x28, 56) {
            if p == a {
                hit = true;
                break;
            }
            p = m.r32(p + 0x10);
        }
    }
    if !hit {
        return;
    }
    let up = |m: &mut M, mut x: u32| {
        if x != 0 && !bit(m, x, 0x28, 56) {
            x = m.r32(x + 0x10);
            while x != 0 && !bit(m, x, 0x28, 56) {
                x = m.r32(x + 0x10);
            }
        }
        x
    };
    let pb = up(m, b);
    let p0 = m.r32(obj + 0x10);
    let pp = up(m, p0);
    if pb == 0 && pp == 0 {
        m.call(0x001f_2ba8, &[obj], &[]);
        m.call(0x001f_2b20, &[obj], &[]);
    }
}

/// FUN_001f3cc8 (slot 19): without an explicit pose `p5`, the +0x134 override (0x30 bytes) mapped by
/// FUN_001a4478(world, a1, ..) is used; FUN_00168638(f1, f2, obj + 0x80, a1, pose), then slot 19 on
/// every child.
pub fn ee_sub_1f3cc8<M: QMem + ?Sized>(m: &mut M, obj: u32, a1: u32, mut p5: u32, f1: f32, f2: f32) {
    let tmp = stack_push(m, 0x30);
    let ov = m.r32(obj + 0x134);
    if p5 == 0 && ov != 0 {
        for i in 0..12 {
            let w = m.r32(ov + i * 4);
            m.w32(tmp + i * 4, w);
        }
        m.call(0x001a_4478, &[WORLD, a1, tmp], &[]);
        p5 = tmp;
    }
    let fv = [[f1, 0.0, 0.0, 0.0], [f2, 0.0, 0.0, 0.0]];
    m.call(0x0016_8638, &[obj + 0x80, a1, p5], &fv);
    let end = m.r32(obj + 0x70);
    for c in children(m, obj, end) {
        vslot(m, c, 19, &[a1, p5], &fv);
    }
    stack_pop(m, 0x30);
}

/// FUN_001f3e00 (slot 21, remove): parent FUN_001a6150, FUN_001d6310, slot 21 on every child; an owned
/// physics record (+0xec, +0x60 == obj) gets slot 48 and leaves DAT_003e11ac.
pub fn ee_sub_1f3e00<M: QMem + ?Sized>(m: &mut M, obj: u32) {
    m.call(0x001a_6150, &[obj], &[]);
    m.call(0x001d_6310, &[obj], &[]);
    let end = m.r32(obj + 0x70);
    for c in children(m, obj, end) {
        vslot(m, c, 21, &[], &[]);
    }
    let x = m.r32(obj + 0xec);
    if x != 0 && m.r32(x + 0x60) == obj {
        vslot(m, obj, 48, &[], &[]);
        let x = m.r32(obj + 0xec);
        m.call(0x0011_a2e8, &[PHYS_LIST, x], &[]);
    }
}

/// FUN_001f3ea8 (slot 22, add): parent FUN_001a6178; +0x148 bit 35 with tag bit 3 of +0x40 (inline when
/// odd, else through the pointer) -> FUN_001d6378; slot 22 on every child; an owned physics record joins
/// DAT_003e11ac (unless FUN_0011a380 finds it; a1 assumed = the record) and slot 48 runs.
pub fn ee_sub_1f3ea8<M: QMem + ?Sized>(m: &mut M, obj: u32) {
    m.call(0x001a_6178, &[obj], &[]);
    if bit(m, obj, 0x148, 35) {
        let mut p = m.r32(obj + 0x40);
        if p & 1 == 0 {
            p = m.r32(p);
        }
        if p & 8 != 0 {
            m.call(0x001d_6378, &[obj], &[]);
        }
    }
    let end = m.r32(obj + 0x70);
    for c in children(m, obj, end) {
        vslot(m, c, 22, &[], &[]);
    }
    let x = m.r32(obj + 0xec);
    if x != 0 && m.r32(x + 0x60) == obj {
        if m.call(0x0011_a380, &[PHYS_LIST, x], &[]).0 == 0 {
            m.call(0x0011_a220, &[PHYS_LIST, x], &[]);
        }
        vslot(m, obj, 48, &[], &[]);
    }
}

/// FUN_001f3fa0 (slot 30): world matrix address (+0x60 +0x40)
pub fn ee_sub_1f3fa0<M: QMem + ?Sized>(m: &mut M, obj: u32) -> u32 {
    m.r32(obj + 0x60) + 0x40
}

/// FUN_001f3fb0 (slot 31): identity into `out` (if any); returns +0x60
pub fn ee_sub_1f3fb0<M: QMem + ?Sized>(m: &mut M, obj: u32, out: u32) -> u32 {
    if out != 0 {
        m.call(0x0023_33b4, &[out, 0, 0x40], &[]);
        for i in [0u32, 5, 10, 15] {
            m.w32(out + i * 4, 0x3f80_0000);
        }
    }
    m.r32(obj + 0x60)
}

/// FUN_001f4010 (slot 36): local xform = FUN_0011c168(a1, xform), new DAT_003e11e8 stamp at +0xa4,
/// slot 17, FUN_00186820 (IK rebuild)
pub fn ee_sub_1f4010<M: QMem + ?Sized>(m: &mut M, obj: u32, a1: u32) {
    let p = m.r32(obj + 0x54);
    let tmp = stack_push(m, 0x40);
    m.call(0x0011_c168, &[tmp, a1, p], &[]);
    let r = m.rm(tmp);
    stack_pop(m, 0x40);
    m.wm(p, r);
    let s = m.r32(XF_STAMP).wrapping_add(1);
    m.w32(XF_STAMP, s);
    let p = m.r32(obj + 0x54);
    m.w32(p + 0xa4, s);
    vslot(m, obj, 17, &[], &[]);
    m.call(0x0018_6820, &[obj], &[]);
}

/// FUN_001f4440 (slot 32): motion (+0x58) in mode 1 -> FUN_0022d348(motion, 0, motion +0x70)
pub fn ee_sub_1f4440<M: QMem + ?Sized>(m: &mut M, obj: u32) {
    let i = m.r32(obj + 0x58);
    if i != 0 && m.r32(i + 0x94) == 1 {
        let v = m.rv(i + 0x70);
        m.call(0x0022_d348, &[i, 0], &[v]);
    }
}

/// FUN_001f4ca0 (slot 51): remap a 0x100-stride record pointer `*pp` from `b`'s table (+0x54 / +0x50,
/// else b's parent +0x10's) to obj's (resp. obj +0x10's), fixing its +0x88 self links (the original
/// writes +0x88 for the +0x8c match too).
pub fn ee_sub_1f4ca0<M: QMem + ?Sized>(m: &mut M, a: u32, b: u32, pp: u32) {
    let x = m.r32(pp);
    if x == 0 {
        return;
    }
    let idx = (x.wrapping_sub(m.r32(b + 0x54)) as i32) >> 8;
    let mut rec = 0u32;
    if idx < 0 || m.r32(a + 0x50) as i32 <= idx {
        let b10 = m.r32(b + 0x10);
        let i3 = (x.wrapping_sub(m.r32(b10 + 0x54)) as i32) >> 8;
        if i3 >= 0 && i3 < m.r32(b10 + 0x50) as i32 {
            let a10 = m.r32(a + 0x10);
            rec = m.r32(a10 + 0x54).wrapping_add((i3 as u32) << 8);
        }
    } else {
        rec = m.r32(a + 0x54).wrapping_add((idx as u32) << 8);
    }
    if m.r32(rec + 0x88) == x.wrapping_add(0x80) {
        m.w32(rec + 0x88, rec + 0x80);
    }
    if m.r32(rec + 0x8c) == x.wrapping_add(0x84) {
        m.w32(rec + 0x88, rec + 0x84);
    }
    m.w32(pp, rec);
}

/// FUN_001f4fa8 / FUN_001f4fb8 / FUN_001f4fc0 / FUN_001f5988 / FUN_001f5990 / FUN_002297f0
/// (slots 58 / 55 / 56 / 53 / 59 / 70): empty
pub fn ee_nop<M: QMem + ?Sized>(_m: &mut M, _obj: u32) {}

/// FUN_001f5998 (slot 62): 0
pub fn ee_sub_1f5998<M: QMem + ?Sized>(_m: &mut M, _obj: u32) -> u32 {
    0
}

/// FUN_001f56f8 (slot 52): FUN_001336f0(world, obj)
pub fn ee_sub_1f56f8<M: QMem + ?Sized>(m: &mut M, obj: u32) -> u32 {
    ee_sub_1336f0(m, WORLD, obj)
}

/// FUN_001f66f0 (slot 39): slot 40 on obj, then FUN_0013e510(child, obj) for `other`'s children
/// without flag bit 57
pub fn ee_sub_1f66f0<M: QMem + ?Sized>(m: &mut M, obj: u32, other: u32) {
    vslot(m, obj, 40, &[], &[]);
    let end = m.r32(other + 0x70);
    let mut c = m.r32(other + 0x64);
    while c != end && c != 0 {
        if !bit(m, c, 0x28, 57) {
            m.call(0x0013_e510, &[c, obj], &[]);
        }
        c = m.r32(c + 0x24);
    }
}

fn cross(a: V4, b: V4) -> V4 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0], 0.0]
}

/// FUN_001fab48: velocity of world point `p` on the body: v + w x (p - physics +0x70) (rec = {v, w})
pub fn ee_sub_1fab48<M: QMem + ?Sized>(m: &mut M, obj: u32, rec: u32, p: V4) -> V4 {
    let ph = m.r32(obj + 0x5c);
    let c = m.rv(ph + 0x70);
    let r = [p[0] - c[0], p[1] - c[1], p[2] - c[2], p[3] - c[3]];
    let w = m.rv(rec + 0x10);
    let v = m.rv(rec);
    let x = cross(w, r);
    [v[0] + x[0], v[1] + x[1], v[2] + x[2], v[3]]
}

/// FUN_001fadd0 (slot 68): point velocity into `out` (zero xyz without a physics record +0xe8)
pub fn ee_sub_1fadd0<M: QMem + ?Sized>(m: &mut M, obj: u32, rec: u32, p: V4, out: u32) {
    if m.r32(obj + 0xe8) == 0 {
        m.w32(out, 0);
        m.w32(out + 8, 0);
        m.w32(out + 4, 0);
    } else {
        let v = ee_sub_1fab48(m, obj, rec, p);
        m.wv(out, v);
    }
}

/// FUN_0022fbe0 (slot 17): FUN_0022faa0 (pose compose), then slot 17 on every child
pub fn ee_sub_22fbe0<M: QMem + ?Sized>(m: &mut M, obj: u32) {
    m.call(0x0022_faa0, &[obj], &[]);
    let end = m.r32(obj + 0x70);
    for c in children(m, obj, end) {
        vslot(m, c, 17, &[], &[]);
    }
}

/// c0 * v.x + c1 * v.y + c2 * v.z (VU mula/madda/madd broadcast)
fn bc3(c: &[V4], v: V4) -> V4 {
    std::array::from_fn(|i| c[0][i] * v[0] + c[1][i] * v[1] + c[2][i] * v[2])
}
fn rows3<M: QMem + ?Sized>(m: &mut M, a: u32) -> [V4; 3] {
    [m.rv(a), m.rv(a + 0x10), m.rv(a + 0x20)]
}
/// orthonormalised (OrthonormalizeMatrix 0x11b3d0) copy of the 3 rows at `src` (identity when 0),
/// inverted by FUN_0011b340
fn inv_orth<M: QMem + ?Sized>(m: &mut M, src: u32) -> [V4; 3] {
    let t = stack_push(m, 0x60);
    if src == 0 {
        m.call(0x0023_33b4, &[t, 0, 0x30], &[]);
        for o in [0u32, 0x14, 0x28] {
            m.w32(t + o, 0x3f80_0000);
        }
    } else {
        for r in 0..3 {
            let v = m.rv(src + r * 0x10);
            m.wv(t + r * 0x10, v);
        }
        m.call(0x0011_b3d0, &[t], &[]);
    }
    m.call(0x0011_b340, &[t + 0x30, t], &[]);
    let r = rows3(m, t + 0x30);
    stack_pop(m, 0x60);
    r
}

/// FUN_001f42e0 (vslot 28, set orientation): rows of `mat` into parent-orthonormal-inverse space ->
/// physics +0x5c rows 0..2 (w = 0), new stamp, slot 17, FUN_001f4e28, FUN_00230598, FUN_001f3900.
pub fn ee_sub_1f42e0<M: QMem + ?Sized>(m: &mut M, obj: u32, mat: u32) {
    let rec = m.r32(obj + 0x5c);
    if rec == 0 {
        return;
    }
    let par = m.r32(rec + 0xa0);
    let inv = inv_orth(m, if par == 0 { 0 } else { par + 0x40 });
    for r in 0..3 {
        let p = m.rv(mat + r * 0x10);
        let mut o = bc3(&inv, p);
        o[3] = 0.0;
        m.wv(rec + r * 0x10, o);
    }
    let s = m.r32(XF_STAMP).wrapping_add(1);
    m.w32(XF_STAMP, s);
    m.w32(rec + 0xa4, s);
    vslot(m, obj, 17, &[], &[]);
    m.call(0x001f_4e28, &[obj], &[]);
    m.call(0x0023_0598, &[obj, rec], &[]);
    m.call(0x001f_3900, &[obj], &[]);
}

fn ensure_active<M: QMem + ?Sized>(m: &mut M, obj: u32, rec: u32) {
    let id = m.r16(rec + 0x98) as u32;
    if m.call(0x0023_0310, &[id], &[]).0 == 0 {
        m.call(0x0023_03a0, &[obj, rec, 1], &[]);
    }
}

fn finish_vel<M: QMem + ?Sized>(m: &mut M, obj: u32, rec: u32, d: V4) {
    let par = m.r32(rec + 0xa0);
    let r = m.call(0x0022_fcc8, &[0, par], &[d]).1;
    m.wv(rec + 0xb0, r);
    m.call(0x0023_0598, &[obj, rec], &[]);
    m.call(0x001f_4bf8, &[obj], &[]);
    m.call(0x001f_5078, &[obj], &[]);
    m.call(0x001f_8668, &[obj, 0], &[]);
}

/// FUN_001f4488 (vslot 34, set linear velocity): motion +0x58 in mode 1: velocity delta v - slot 32()
/// into the parent frame (FUN_0022fcc8) -> +0xb0, then the motion refresh chain.
pub fn ee_sub_1f4488<M: QMem + ?Sized>(m: &mut M, obj: u32, v: V4) {
    let rec = m.r32(obj + 0x58);
    if rec == 0 || m.r32(rec + 0x94) != 1 {
        return;
    }
    ensure_active(m, obj, rec);
    for o in [0xb4, 0xb8, 0xb0] {
        m.w32(rec + o, 0);
    }
    let cur = vslot(m, obj, 32, &[], &[]).1;
    let d = [v[0] - cur[0], v[1] - cur[1], v[2] - cur[2], v[3] - cur[3]];
    finish_vel(m, obj, rec, d);
}

/// FUN_001f46d8 (vslot 35, set angular velocity): physics +0x5c in mode 6: delta v - slot 33() mapped by
/// rows(+0x40) x inverse(orthonormal rows(+0x40)), then as FUN_001f4488.
pub fn ee_sub_1f46d8<M: QMem + ?Sized>(m: &mut M, obj: u32, v: V4) {
    let rec = m.r32(obj + 0x5c);
    if rec == 0 || m.r32(rec + 0x94) != 6 {
        return;
    }
    ensure_active(m, obj, rec);
    for o in [0xb4, 0xb8, 0xb0] {
        m.w32(rec + o, 0);
    }
    let cur = vslot(m, obj, 33, &[], &[]).1;
    let inv = inv_orth(m, rec + 0x40);
    let r = rows3(m, rec + 0x40);
    let mm = [bc3(&r, inv[0]), bc3(&r, inv[1]), bc3(&r, inv[2])];
    let d = [v[0] - cur[0], v[1] - cur[1], v[2] - cur[2], v[3] - cur[3]];
    let x = bc3(&mm, d);
    finish_vel(m, obj, rec, x);
}

/// FUN_001f49c8 (vslot 36, copy motion): base FUN_001a6b88, then with physics (+0xe8) linear / angular
/// velocity of `other` (slots 32 / 33) -> slots 34 / 35.
pub fn ee_sub_1f49c8<M: QMem + ?Sized>(m: &mut M, obj: u32, other: u32) {
    m.call(0x001a_6b88, &[obj, other], &[]);
    if m.r32(obj + 0xe8) != 0 {
        let v = vslot(m, other, 32, &[], &[]).1;
        vslot(m, obj, 34, &[], &[v]);
        let w = vslot(m, other, 33, &[], &[]).1;
        vslot(m, obj, 35, &[], &[w]);
    }
}

/// FUN_001f4a80 (vslot 37): base FUN_001a6bc8, then with physics the velocity matrix
/// FUN_0022d148(other): linear = M * (other +0x70, 1) -> slot 34, angular (M1.z, M2.x, M0.y) -> slot 35.
pub fn ee_sub_1f4a80<M: QMem + ?Sized>(m: &mut M, obj: u32, other: u32) {
    m.call(0x001a_6bc8, &[obj, other], &[]);
    if m.r32(obj + 0xe8) == 0 {
        return;
    }
    let t = stack_push(m, 0x40);
    m.call(0x0022_d148, &[t, other, 0], &[]);
    let mm = m.rm(t);
    stack_pop(m, 0x40);
    let p = m.rv(other + 0x70);
    let lin: V4 = std::array::from_fn(|i| bc3(&mm[..3], p)[i] + mm[3][i]);
    let ang = [mm[1][2], mm[2][0], mm[0][1], 0.0];
    vslot(m, obj, 34, &[], &[lin]);
    vslot(m, obj, 35, &[], &[ang]);
}

/// FUN_001fa328 (vslot 64): pending linear (+0xf0, motion +0x58) / angular (+0x100, physics +0x5c)
/// impulses not FFloatsNear(1e-4) DAT_002a2dd0 are mapped (FUN_0022fcc8) and added to the record's
/// +0xb0, cleared, FUN_001f4bf8.
pub fn ee_sub_1fa328<M: QMem + ?Sized>(m: &mut M, obj: u32) {
    if m.r32(obj + 0xe8) == 0 {
        return;
    }
    for (off, ro) in [(0xf0u32, 0x58u32), (0x100, 0x5c)] {
        let z = m.rv(0x002a_2dd0);
        let v = m.rv(obj + off);
        let mut near = true;
        for i in 0..3 {
            if m.call(0x0011_d4d0, &[], &[[z[i]; 4], [v[i]; 4], [1e-4; 4]]).0 == 0 {
                near = false;
                break;
            }
        }
        if near {
            continue;
        }
        let rec = m.r32(obj + ro);
        m.call(0x0023_0598, &[obj, rec], &[]);
        let par = m.r32(rec + 0xa0);
        let r = m.call(0x0022_fcc8, &[0, par], &[v]).1;
        for o in [4, 8, 0] {
            m.w32(obj + off + o, 0);
        }
        let b = m.rv(rec + 0xb0);
        m.wv(rec + 0xb0, std::array::from_fn(|i| b[i] + r[i]));
        m.call(0x001f_4bf8, &[obj], &[]);
    }
}

/// FUN_001fad00 (vslot 66): impulse record `rec` (+0x40 direction, negated for side 1; +0x50 contact
/// points stride 0x40): with physics, linear response FUN_001faa00(dir) -> rec + 0x20 side unless
/// physics +0xc0 == 3, angular FUN_001faa48(point +0x10, dir) -> +0x10 unless +0xa0 == 3.
pub fn ee_sub_1fad00<M: QMem + ?Sized>(m: &mut M, obj: u32, rec: u32, side: u32) {
    let ph = m.r32(obj + 0xe8);
    if ph == 0 {
        return;
    }
    let mut d = m.rv(rec + 0x40);
    if side != 0 {
        d = d.map(|x| x * -1.0);
    }
    let out = rec + side * 0x20;
    if m.r32(ph + 0xc0) != 3 {
        let r = m.call(0x001f_aa00, &[obj], &[d]).1;
        m.wv(out, r);
    }
    if m.r32(ph + 0xa0) != 3 {
        let p = m.r32(rec + 0x50) + side * 0x40;
        let q = m.rv(p + 0x10);
        let r = m.call(0x001f_aa48, &[obj], &[q, d]).1;
        m.wv(out + 0x10, r);
    }
}

fn near_dat<M: QMem + ?Sized>(m: &mut M, v: V4) -> bool {
    let z = m.rv(0x002a_2dd0);
    (0..3).all(|i| m.call(0x0011_d4d0, &[], &[[z[i]; 4], [v[i]; 4], [1e-4; 4]]).0 != 0)
}

/// FUN_001fa528 (vslot 65): apply pending displacements: linear +0xf0 (mapped by FUN_0022fcc8) added
/// to motion +0x58's position +0x30 (w = 1); rotation +0x100 (mapped) -> MatrixFromRotationVector
/// (0x11b570) R, physics +0x5c rows i = R * row i (w = 0); each with a new stamp; then slot 17 +
/// FUN_001f4bf8 if anything moved.
pub fn ee_sub_1fa528<M: QMem + ?Sized>(m: &mut M, obj: u32) {
    if m.r32(obj + 0xe8) == 0 {
        return;
    }
    let mut moved = false;
    let v = m.rv(obj + 0xf0);
    if !near_dat(m, v) {
        moved = true;
        let rec = m.r32(obj + 0x58);
        m.call(0x0023_0598, &[obj, rec], &[]);
        let par = m.r32(rec + 0xa0);
        let r = m.call(0x0022_fcc8, &[0, par], &[v]).1;
        for o in [4, 8, 0] {
            m.w32(obj + 0xf0 + o, 0);
        }
        let p = m.rv(rec + 0x30);
        m.wv(rec + 0x30, [p[0] + r[0], p[1] + r[1], p[2] + r[2], 1.0]);
        let s = m.r32(XF_STAMP).wrapping_add(1);
        m.w32(XF_STAMP, s);
        m.w32(rec + 0xa4, s);
    }
    let w = m.rv(obj + 0x100);
    if !near_dat(m, w) {
        moved = true;
        let rec = m.r32(obj + 0x5c);
        m.call(0x0023_0598, &[obj, rec], &[]);
        let par = m.r32(rec + 0xa0);
        let r = m.call(0x0022_fcc8, &[0, par], &[w]).1;
        for o in [4, 8, 0] {
            m.w32(obj + 0x100 + o, 0);
        }
        let t = stack_push(m, 0x40);
        m.call(0x0011_b570, &[t], &[r]);
        let rm = rows3(m, t);
        stack_pop(m, 0x40);
        for i in 0..3 {
            let row = m.rv(rec + i * 0x10);
            let mut o = bc3(&rm, row);
            o[3] = 0.0;
            m.wv(rec + i * 0x10, o);
        }
        let s = m.r32(XF_STAMP).wrapping_add(1);
        m.w32(XF_STAMP, s);
        m.w32(rec + 0xa4, s);
    }
    if moved {
        vslot(m, obj, 17, &[], &[]);
        m.call(0x001f_4bf8, &[obj], &[]);
    }
}

/// FUN_001f40a0 (vslot 27, set world position `v`): A = (+0x60 world rotation, translation v);
/// C = inv(motion local +0x40) * (+0x60 +0x40); P = motion parent (+0xa0) +0x40 or identity;
/// motion +0x30 = translation of inv(P) * A * inv(C) (w = 1); new stamp, slot 17, FUN_001f4e28,
/// FUN_00230598, FUN_001f3900; with a physics +0xdc hook FUN_001c0d70(pos, 1) + FUN_001f37c8.
/// (4x4 inverse = 0x11c318, product = FUN_0011c168(out, a, b).)
pub fn ee_sub_1f40a0<M: QMem + ?Sized>(m: &mut M, obj: u32, v: V4) {
    let mot = m.r32(obj + 0x58);
    if mot == 0 {
        return;
    }
    let w = m.r32(obj + 0x60);
    let t = stack_push(m, 0x180);
    let (a, inv, c, p, x, y) = (t, t + 0x40, t + 0x80, t + 0xc0, t + 0x100, t + 0x140);
    for r in 0..3 {
        let row = m.rv(w + 0x40 + r * 0x10);
        m.wv(a + r * 0x10, [row[0], row[1], row[2], 0.0]);
    }
    m.wv(a + 0x30, [v[0], v[1], v[2], 1.0]);
    m.call(0x0011_c318, &[inv, mot + 0x40], &[]);
    m.call(0x0011_c168, &[c, inv, w + 0x40], &[]);
    let par = m.r32(mot + 0xa0);
    if par == 0 {
        m.call(0x0023_33b4, &[p, 0, 0x40], &[]);
        for o in [0u32, 0x14, 0x28, 0x3c] {
            m.w32(p + o, 0x3f80_0000);
        }
    } else {
        let pm = m.rm(par + 0x40);
        m.wm(p, pm);
    }
    m.call(0x0011_c318, &[x, p], &[]);
    m.call(0x0011_c168, &[y, x, a], &[]);
    m.call(0x0011_c318, &[x, c], &[]);
    m.call(0x0011_c168, &[inv, y, x], &[]);
    let tr = m.rv(inv + 0x30);
    stack_pop(m, 0x180);
    m.wv(mot + 0x30, [tr[0], tr[1], tr[2], 1.0]);
    let s = m.r32(XF_STAMP).wrapping_add(1);
    m.w32(XF_STAMP, s);
    m.w32(mot + 0xa4, s);
    vslot(m, obj, 17, &[], &[]);
    m.call(0x001f_4e28, &[obj], &[]);
    m.call(0x0023_0598, &[obj, mot], &[]);
    m.call(0x001f_3900, &[obj], &[]);
    let ph = m.r32(obj + 0xe8);
    if ph != 0 && m.r32(ph + 0xdc) != 0 {
        let pos = m.call(0x001a_62e8, &[obj], &[]).1;
        m.call(0x001c_0d70, &[1], &[pos]);
        m.call(0x001f_37c8, &[obj], &[]);
    }
}
