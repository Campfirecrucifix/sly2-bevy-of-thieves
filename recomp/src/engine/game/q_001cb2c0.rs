//! WP-Q4: run 0x001cb2c0..0x001ccc88 - C2caf88 (path / rail ALO) helpers: the attachment record (+0xe0
//! {kind, ids, count, first}, +0xe8 attachment list), base slots 25 / 26 (add to / remove from the world
//! and forward to the children +0x64 / +0x24), the path sampling helpers (FUN_001e6f10 / 1e6f60 / 1e8008)
//! and the player rope / zipline aim record (p +0 player, +4, +8 / +0x1c blend, +0xc / +0x20 two curves
//! {curve, out0, out1} stride 0x14, +0x30 anim, +0x34 / +0x38 skeleton node, +0x40 matrix, +0x80 handle,
//! +0x84 target, +0x88 param, +0x8c target object, +0x90 mode, +0x94 / +0x9c handles, +0xa4 state,
//! +0xa8 state time).
//! 1:1 against `QMem` (classes::q::q2mem); callees outside the run go through `QMem::call` by address.
//! VU0 notes: `adda vf0, x; madd` adds vf0.w = 1 to w; opmula / opmsub = cross product (xyz only).
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};
use crate::engine::game::q_001e9090::ee_sub_1ea410;

/// DAT_002c1ec4: sample step for FUN_001e8008 (FUN_001cbba8)
pub const SPRAY_STEP: u32 = 0x002c_1ec4;
/// DAT_002c1ec8: angular spread (FUN_001cbba8)
pub const SPRAY_SPREAD: u32 = 0x002c_1ec8;
/// DAT_002c1ecc: radial spread (FUN_001cbba8)
pub const SPRAY_RADIUS: u32 = 0x002c_1ecc;
/// DAT_002a2dc0: up vector (FUN_001cbba8)
pub const UP: u32 = 0x002a_2dc0;
/// DAT_002a2da0: fallback side vector
pub const DEFAULT_DIR: u32 = 0x002a_2da0;
/// DAT_002c1ed0: point count of the aim curves (FUN_001cc5e8)
pub const CURVE_POINTS: u32 = 0x002c_1ed0;

fn cross(a: V4, b: V4, w: f32) -> V4 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0], w]
}
fn f(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

/// FUN_001cb2c0(o): FUN_001cade0(o, 2) != 0
pub fn ee_sub_1cb2c0<M: QMem + ?Sized>(m: &mut M, o: u32) -> bool {
    m.call(0x001c_ade0, &[o, 2], &[]).0 != 0
}

/// FUN_001cb2e0(o): first attachment's id (record +0xe0 ids [FUN_001ca930(o, +0xe8)]), 0 without a list
pub fn ee_sub_1cb2e0<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let l = m.r32(o + 0xe8);
    if l == 0 {
        return 0;
    }
    let i = m.call(0x001c_a930, &[o, l], &[]).0;
    let r = m.r32(o + 0xe0);
    let ids = m.r32(r + 4);
    m.r32(i.wrapping_mul(4).wrapping_add(ids))
}

/// FUN_001cb328(o): FUN_001cb250(o, 1) when it has no attachment
pub fn ee_sub_1cb328<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if ee_sub_1cb2e0(m, o) == 0 {
        m.call(0x001c_b250, &[o, 1], &[]);
    }
}

/// FUN_001cb360(ret, o): up to 32 attachment ids (list +0xe8, next +4) -> FUN_00200f58(ret, n, 0x2002, ids)
pub fn ee_sub_1cb360<M: QMem + ?Sized>(m: &mut M, ret: u32, o: u32) -> u32 {
    let buf = stack_push(m, 0x80);
    let mut n = 0u32;
    let mut l = m.r32(o + 0xe8);
    if l != 0 {
        loop {
            let i = m.call(0x001c_a930, &[o, l], &[]).0;
            n += 1;
            let r = m.r32(o + 0xe0);
    let ids = m.r32(r + 4);
            let v = m.r32(i.wrapping_mul(4).wrapping_add(ids));
            m.w32(buf + 4 * (n - 1), v);
            l = m.r32(l + 4);
            if l == 0 || n >= 0x20 {
                break;
            }
        }
    }
    m.call(0x0020_0f58, &[ret, n, 0x2002, buf], &[]);
    stack_pop(m, 0x80);
    ret
}

/// FUN_001cb410(o, id): record +0xe0 of kind 1 holding `id` in [+0xc, +8) -> FUN_001ca958(o), else 0
pub fn ee_sub_1cb410<M: QMem + ?Sized>(m: &mut M, o: u32, id: u32) -> u32 {
    let r = m.r32(o + 0xe0);
    if m.r32(r) != 1 {
        return 0;
    }
    let mut i = m.r32(r + 0xc) as i32;
    while i < m.r32(r + 8) as i32 {
        let ids = m.r32(r + 4);
        if m.r32((i as u32).wrapping_mul(4).wrapping_add(ids)) == id {
            return m.call(0x001c_a958, &[o], &[]).0;
        }
        i += 1;
    }
    0
}

/// FUN_001cb488(o, id): attachment of `id` +8 = 1
pub fn ee_sub_1cb488<M: QMem + ?Sized>(m: &mut M, o: u32, id: u32) {
    let r = ee_sub_1cb410(m, o, id);
    if r != 0 {
        m.w32(r + 8, 1);
    }
}

/// FUN_001cb4b8(o, id): attachment of `id` +8 = 0
pub fn ee_sub_1cb4b8<M: QMem + ?Sized>(m: &mut M, o: u32, id: u32) {
    let r = ee_sub_1cb410(m, o, id);
    if r != 0 {
        m.w32(r + 8, 0);
    }
}

/// FUN_001cb518(o, a) = C2caf88 slot 25: base FUN_001cb4e0, FUN_001f7a68(o, 0), +0xcc = -1; with an
/// attachment list (+0xe8 +0xdc) FUN_001f37c8(o, FUN_001c0d70(pos FUN_001a62e8(o), 1)); then slot 25 of
/// every child (+0x64, next +0x24) with `a`
pub fn ee_sub_1cb518<M: QMem + ?Sized>(m: &mut M, o: u32, a: u32) {
    m.call(0x001c_b4e0, &[o, a], &[]);
    m.call(0x001f_7a68, &[o, 0], &[]);
    m.w32(o + 0xcc, u32::MAX);
    let l = m.r32(o + 0xe8);
    let att = if l != 0 { m.r32(l + 0xdc) } else { 0 };
    if att != 0 {
        let pos = m.call(0x001a_62e8, &[o], &[]).1;
        let vol = m.call(0x001c_0d70, &[1], &[pos]).0;
        m.call(0x001f_37c8, &[o, vol], &[]);
    }
    let mut c = m.r32(o + 0x64);
    while c != 0 {
        vslot(m, c, 25, &[a], &[]);
        c = m.r32(c + 0x24);
    }
}

/// FUN_001cb5d0(o, a) = C2caf88 slot 26: base FUN_001cb5c8, FUN_001f37c8(o, 0) with an attachment list,
/// then slot 26 of every child
pub fn ee_sub_1cb5d0<M: QMem + ?Sized>(m: &mut M, o: u32, a: u32) {
    m.call(0x001c_b5c8, &[o, a], &[]);
    let l = m.r32(o + 0xe8);
    let att = if l != 0 { m.r32(l + 0xdc) } else { 0 };
    if att != 0 {
        m.call(0x001f_37c8, &[o, 0], &[]);
    }
    let mut c = m.r32(o + 0x64);
    while c != 0 {
        vslot(m, c, 26, &[a], &[]);
        c = m.r32(c + 0x24);
    }
}

/// FUN_001cb7b0 / FUN_001cbd58 / FUN_001cc000 (identical static pool inits) = FUN_001ea410
pub fn ee_sub_1cb7b0<M: QMem + ?Sized>(m: &mut M, init: u32, prio: u32) {
    ee_sub_1ea410(m, init, prio);
}
/// FUN_001cb8e8 / FUN_001cbe90 / FUN_001cc138: (1, 0xffff)
pub fn ee_sub_1cb8e8<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_1ea410(m, 1, 0xffff);
}

/// FUN_001cba68(path, out): closed path (+0xe4) -> out = [FUN_001e6620, FUN_001e6648], true
pub fn ee_sub_1cba68<M: QMem + ?Sized>(m: &mut M, path: u32, out: u32) -> bool {
    if m.r32(path + 0xe4) == 0 {
        return false;
    }
    let a = m.call(0x001e_6620, &[path], &[]).1[0];
    m.wf(out, a);
    let b = m.call(0x001e_6648, &[path], &[]).1[0];
    m.wf(out + 4, b);
    true
}

/// FUN_001cbac0(path, obj, out): closest path parameter to obj's position (*(obj +0x218)) via
/// FUN_001e6f60 into *out; an open path clamps it to [50, length - 50]
pub fn ee_sub_1cbac0<M: QMem + ?Sized>(m: &mut M, path: u32, obj: u32, out: u32) {
    let sp = stack_push(m, 0x10);
    let pp = m.r32(obj + 0x218);
    let pos = m.rv(pp);
    m.call(0x001e_6f60, &[path, 0, sp, 0, 0, out], &[pos]);
    stack_pop(m, 0x10);
    if m.r32(path + 0xe4) == 0 {
        let len = m.call(0x001e_6648, &[path], &[]).1[0];
        let mut t = m.rf(out);
        if t < 50.0 {
            t = 50.0;
        } else if len - 50.0 < t {
            t = len - 50.0;
        }
        m.wf(out, t);
    }
}

/// FUN_001cbb58(path, obj, out, t): out = point(t) (FUN_001e6f10) - obj +0x240
pub fn ee_sub_1cbb58<M: QMem + ?Sized>(m: &mut M, path: u32, obj: u32, out: u32, t: f32) {
    let sp = stack_push(m, 0x10);
    m.call(0x001e_6f10, &[path, sp, 0], &[f(t)]);
    let p = m.rv(sp);
    stack_pop(m, 0x10);
    if out != 0 {
        let o = m.rv(obj + 0x240);
        m.wv(out, [p[0] - o[0], p[1] - o[1], p[2] - o[2], p[3] - o[3]]);
    }
}

/// FUN_001cbba8(path, out_pos, out_dir): random spray from the path: t = FUN_001e8008(path,
/// DAT_002c1ec4), point / tangent FUN_001e6f10; side = normalize(up x tangent) (DAT_002a2da0 under 1e-4),
/// normal = tangent x side; r = FUN_0011d448(-1, 1), dir = FUN_0011e1a0(r |r| spread, 1, 0) in the
/// (normal, side, tangent) frame; out_pos = point + dir * |r| FUN_0011d448(0, radius) (w + 1)
pub fn ee_sub_1cbba8<M: QMem + ?Sized>(m: &mut M, path: u32, out_pos: u32, out_dir: u32) {
    let step = m.rf(SPRAY_STEP);
    let t = m.call(0x001e_8008, &[path], &[f(step)]).1[0];
    let sp = stack_push(m, 0x40);
    m.call(0x001e_6f10, &[path, sp, sp + 0x30], &[f(t)]);
    let pos = m.rv(sp);
    let tan = m.rv(sp + 0x30);
    stack_pop(m, 0x40);
    let up = m.rv(UP);
    // vf3.w is not written by opmsub.xyz: 0 assumed (the w lane never reaches xyz)
    let c = cross(up, tan, 0.0);
    let l = (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt();
    let side = if l < 0.0001 {
        m.rv(DEFAULT_DIR)
    } else {
        let q = 1.0 / l;
        [c[0] * q, c[1] * q, c[2] * q, c[3] * q]
    };
    let nrm = cross(tan, side, tan[3]);
    let r = m.call(0x0011_d448, &[], &[f(-1.0), f(1.0)]).1[0];
    let a = r.abs();
    let spread = m.rf(SPRAY_SPREAD);
    let e = m.call(0x0011_e1a0, &[], &[f(r * a * spread), f(1.0), f(0.0)]).1;
    let dir: V4 = [
        nrm[0] * e[0] + side[0] * e[1] + tan[0] * e[2],
        nrm[1] * e[0] + side[1] * e[1] + tan[1] * e[2],
        nrm[2] * e[0] + side[2] * e[1] + tan[2] * e[2],
        e[3],
    ];
    if out_pos != 0 {
        let rad = m.rf(SPRAY_RADIUS);
        let k = a * m.call(0x0011_d448, &[], &[f(0.0), f(rad)]).1[0];
        m.wv(
            out_pos,
            [pos[0] + dir[0] * k, pos[1] + dir[1] * k, pos[2] + dir[2] * k, 1.0 + pos[3] + dir[3] * k],
        );
    }
    if out_dir != 0 {
        m.wv(out_dir, dir);
    }
}

fn aim_seg<M: QMem + ?Sized>(m: &mut M, p: u32, v: V4, o1: u32, o2: u32, t: f32, end: u32) {
    let sp = stack_push(m, 0x20);
    let c = m.r32(p + 0x14 + 4 * end);
    m.call(0x001e_67b8, &[c, sp, 0], &[f(t)]);
    let c = m.r32(p + 0x28 + 4 * end);
    m.call(0x001e_67b8, &[c, sp + 0x10, 0], &[f(t)]);
    let (a, b) = (m.rv(sp), m.rv(sp + 0x10));
    stack_pop(m, 0x20);
    m.call(0x0011_ee08, &[0, o1, o2], &[v, a, b]);
}

/// FUN_001cc168(p, v, o1, o2, t): points of the two first curves (+0x14 / +0x28) at t (FUN_001e67b8) ->
/// FUN_0011ee08(v, a, b, 0, o1, o2)
pub fn ee_sub_1cc168<M: QMem + ?Sized>(m: &mut M, p: u32, v: V4, o1: u32, o2: u32, t: f32) {
    aim_seg(m, p, v, o1, o2, t, 0);
}
/// FUN_001cc1f8: FUN_001cc168 with the second curves (+0x18 / +0x2c)
pub fn ee_sub_1cc1f8<M: QMem + ?Sized>(m: &mut M, p: u32, v: V4, o1: u32, o2: u32, t: f32) {
    aim_seg(m, p, v, o1, o2, t, 1);
}
/// FUN_001cc288(t, ctx): FUN_001cc168(*ctx, ctx +0x10, 0, &r, t) -> r (root-finder callback)
pub fn ee_sub_1cc288<M: QMem + ?Sized>(m: &mut M, t: f32, ctx: u32) -> f32 {
    let r = stack_push(m, 0x10);
    let (p, v) = (m.r32(ctx), m.rv(ctx + 0x10));
    ee_sub_1cc168(m, p, v, 0, r, t);
    let x = m.rf(r);
    stack_pop(m, 0x10);
    x
}
/// FUN_001cc2b8(t, ctx): FUN_001cc1f8 variant of FUN_001cc288
pub fn ee_sub_1cc2b8<M: QMem + ?Sized>(m: &mut M, t: f32, ctx: u32) -> f32 {
    let r = stack_push(m, 0x10);
    let (p, v) = (m.r32(ctx), m.rv(ctx + 0x10));
    ee_sub_1cc1f8(m, p, v, 0, r, t);
    let x = m.rf(r);
    stack_pop(m, 0x10);
    x
}

/// FUN_001cc2e8(p): aim update (state +0xa4 != -1). Target by mode +0x90: 0 = path +0x84 nearest point to
/// the player (FUN_001e7380, param +0x88), 1 = position of +0x8c, 2 = node matrix of +0x8c (slot 30) *
/// inverse(+0x40) (row 3 the target, rows 0..2 kept). The target in player space (FUN_001a6f00) is
/// flattened to [hypot(x, y), 0, z, w] and back to world; FUN_0011d9e0(0.5, 0.1, 0, 1, FUN_001cc288 /
/// 1cc2b8, ctx) solves the curve blend, FUN_001cc168 / 1cc1f8 its weight; mode 2 also aims handle +0x80
/// (FUN_0022fd38 / 11bff0 / 12d108). Then anim time FUN_00130d30(t * anim +0x74), +0x1c = w, +8 = 1 - w,
/// FUN_001384c0, FUN_002046b0(player, target - player pos), FUN_00204ca8(player).
pub fn ee_sub_1cc2e8<M: QMem + ?Sized>(m: &mut M, p: u32) {
    if m.r32(p + 0xa4) == u32::MAX {
        return;
    }
    let sp = stack_push(m, 0x170);
    let mode = m.r32(p + 0x90) as i32;
    if mode == 1 {
        let o = m.r32(p + 0x8c);
        let v = m.call(0x001a_62e8, &[o], &[]).1;
        m.wv(sp, v);
    } else if mode == 0 {
        let pl = m.r32(p);
        let pos = m.call(0x001a_62e8, &[pl], &[]).1;
        let path = m.r32(p + 0x84);
        let t = m.rf(p + 0x88);
        m.call(0x001e_7380, &[path, 0, sp, 0, 0, p + 0x88], &[pos, f(t)]);
    } else if mode == 2 {
        let o = m.r32(p + 0x8c);
        let mat = vslot(m, o, 30, &[], &[]).0;
        m.call(0x0011_c318, &[sp + 0xc0, p + 0x40], &[]);
        m.call(0x0011_c168, &[sp + 0x80, mat, sp + 0xc0], &[]);
        let r = m.rm(sp + 0x80);
        m.wv(sp + 0x50, r[1]);
        m.wv(sp + 0x60, r[2]);
        m.wv(sp, r[3]);
        m.wv(sp + 0x10, r[0]);
        m.wv(sp + 0x40, r[0]);
        m.wv(sp + 0x70, r[3]);
        m.wv(sp + 0x20, r[1]);
        m.wv(sp + 0x30, r[2]);
    }
    let tgt = m.rv(sp);
    let pl = m.r32(p);
    m.w32(sp + 0x40, p);
    let l = m.call(0x001a_6f00, &[0, pl], &[tgt]).1;
    let h = (l[0] * l[0] + l[1] * l[1]).sqrt();
    let flat = [h, 0.0, l[2], l[3]];
    m.wv(sp + 0x60, flat);
    let pl = m.r32(p);
    let w = m.call(0x001a_6f00, &[pl, 0], &[flat]).1;
    m.wv(sp + 0x50, w);
    let ctx = sp + 0x40;
    if m.r32(p + 0x90) == 2 {
        m.call(0x0011_d9e0, &[0x001c_c2b8, ctx, sp + 0x100, sp + 0x104], &[f(0.5), f(0.1), f(0.0), f(1.0)]);
        let t = m.rf(sp + 0x100);
        let v = m.rv(sp + 0x50);
        ee_sub_1cc1f8(m, p, v, sp + 0x108, 0, t);
        let nd = m.r32(p + 0x38);
        let node = m.r32(nd + 0xa0);
        m.call(0x0022_fd38, &[sp + 0xa0, 0, node, sp + 0x10], &[]);
        for k in 0..3 {
            let r = m.rv(sp + 0xa0 + 0x10 * k);
            m.wv(sp + 0x70 + 0x10 * k, r);
        }
        let e = m.call(0x0011_bff0, &[sp + 0x70], &[]).1;
        let hd = m.r32(p + 0x80);
        m.call(0x0012_d108, &[hd], &[e]);
    } else {
        m.call(0x0011_d9e0, &[0x001c_c288, ctx, sp + 0x100, sp + 0x104], &[f(0.5), f(0.1), f(0.0), f(1.0)]);
        let t = m.rf(sp + 0x100);
        let v = m.rv(sp + 0x50);
        ee_sub_1cc168(m, p, v, sp + 0x108, 0, t);
    }
    let an = m.r32(p + 0x30);
    let t = m.rf(sp + 0x100) * m.rf(an + 0x74);
    let a0 = m.r32(p + 4);
    let a0 = m.r32(a0);
    m.call(0x0013_0d30, &[a0, 0], &[f(t), f(0.0)]);
    let wgt = m.rf(sp + 0x108);
    m.wf(p + 0x1c, wgt);
    m.wf(p + 8, 1.0 - wgt);
    let an = m.r32(p + 0x30);
    m.call(0x0013_84c0, &[an], &[]);
    let tgt = m.rv(sp);
    let pl = m.r32(p);
    let pos = m.call(0x001a_62e8, &[pl], &[]).1;
    let d = [tgt[0] - pos[0], tgt[1] - pos[1], tgt[2] - pos[2], tgt[3] - pos[3]];
    let pl = m.r32(p);
    m.call(0x0020_46b0, &[pl], &[d]);
    let pl = m.r32(p);
    m.call(0x0020_4ca8, &[pl], &[]);
    stack_pop(m, 0x170);
}

/// FUN_001cc5e8(p, player, b): aim record setup: +0 player, +4 b; with a skeleton (+0x34): +0x40 = its
/// slot-31 matrix, +0x38 = node (walked up +0xa0 to kind (+0x94) 6, +0x40 = node * +0x40 per step); for each
/// curve (+0xc / +0x20, stride 0x14) two rope objects (class 10, FUN_0013e3c8(10, player, 0xb2, -1), each
/// DAT_002c1ed0 points: +0xec t = i / (n - 1), +0xf4 = FUN_001300b8(t * curve +0x74, ..) matrix * base
/// row 3) for the root (slot-31 matrix) / the walked node (identity), slot 8 + FUN_001e6180, stored at
/// +0x14 / +0x18 of the curve; both curves -> +8 = 1, FUN_00137a30(player, 0x14, 2, p +8, 0, 0, 0, p +0x30)
pub fn ee_sub_1cc5e8<M: QMem + ?Sized>(m: &mut M, p: u32, player: u32, b: u32) {
    m.w32(p, player);
    let sk = m.r32(p + 0x34);
    m.w32(p + 4, b);
    if sk == 0 {
        return;
    }
    let sp = stack_push(m, 0x190);
    let root = vslot(m, sk, 31, &[sp], &[]).0;
    m.w32(sp + 0xc0, root);
    let mat = m.rm(sp);
    m.wm(p + 0x40, mat);
    m.w32(p + 0x38, root);
    if m.r32(root + 0x94) != 6 {
        loop {
            let n = m.r32(p + 0x38);
            m.call(0x0011_c168, &[sp + 0x40, n, p + 0x40], &[]);
            let r = m.rm(sp + 0x40);
            m.wm(p + 0x40, r);
            let nd = m.r32(p + 0x38);
            let up = m.r32(nd + 0xa0);
            m.w32(p + 0x38, up);
            if m.r32(up + 0x94) == 6 {
                break;
            }
        }
    }
    let mut cnt = 0;
    for k in 0..2u32 {
        let curve = m.r32(p + 0xc + 0x14 * k);
        if curve == 0 {
            continue;
        }
        cnt += 1;
        for j in 0..2u32 {
            let pl = m.r32(p);
            let o = m.call(0x0013_e3c8, &[10, pl, 0xb2, u32::MAX], &[]).0;
            let n = m.r32(CURVE_POINTS);
            m.w32(o + 0xe0, 0);
            m.w32(o + 0xe8, n);
            for (off, sh) in [(0xec, 2), (0xf0, 2), (0xf4, 4)] {
                let n = m.r32(CURVE_POINTS);
                m.w32(sp + 0x40, n << sh);
                m.w32(sp + 0x44, 0x10);
                let a = m.call(0x001a_e1d0, &[sp + 0x40], &[]).0;
                m.w32(o + off, a);
            }
            let node = if j == 0 {
                let r = m.rm(sp);
                m.wm(sp + 0x40, r);
                m.r32(sp + 0xc0)
            } else {
                let node = m.r32(p + 0x38);
                m.call(0x0023_33b4, &[sp + 0x40, 0, 0x40], &[]);
                for off in [0x7c, 0x68, 0x54, 0x40] {
                    m.wf(sp + off, 1.0);
                }
                node
            };
            let mut i = 0;
            while i < m.r32(CURVE_POINTS) as i32 {
                let n = m.r32(CURVE_POINTS) as i32;
                let t = i as f32 / (n - 1) as f32;
                let pl = m.r32(p);
                let a3 = m.r32(pl + 0x60);
                let len = m.rf(curve + 0x74);
                m.call(0x0013_00b8, &[curve, pl, node, a3, sp + 0x80, 0], &[f(t * len), f(0.0)]);
                let ta = m.r32(o + 0xec);
                m.wf(ta + 4 * i as u32, t);
                let x = m.rv(sp + 0x70);
                let r = m.rm(sp + 0x80);
                let pt: V4 = [
                    r[0][0] * x[0] + r[1][0] * x[1] + r[2][0] * x[2] + r[3][0],
                    r[0][1] * x[0] + r[1][1] * x[1] + r[2][1] * x[2] + r[3][1],
                    r[0][2] * x[0] + r[1][2] * x[1] + r[2][2] * x[2] + r[3][2],
                    r[0][3],
                ];
                let pa = m.r32(o + 0xf4);
                m.wv(pa + 0x10 * i as u32, pt);
                i += 1;
            }
            vslot(m, o, 8, &[], &[]);
            m.call(0x001e_6180, &[o], &[]);
            m.w32(p + 0x14 + 0x14 * k + 4 * j, o);
        }
    }
    if cnt == 2 {
        m.wf(p + 8, 1.0);
        let pl = m.r32(p);
        m.call(0x0013_7a30, &[pl, 0x14, 2, p + 8, 0, 0, 0, p + 0x30], &[]);
    }
    stack_pop(m, 0x190);
}

/// FUN_001cca20(_, p): FUN_001cc2e8(p)
pub fn ee_sub_1cca20<M: QMem + ?Sized>(m: &mut M, _a: u32, p: u32) {
    ee_sub_1cc2e8(m, p);
}

/// FUN_001cca40(player, o): first of up to 4 objects of FUN_0015e9c8(0x3dd490, 0x401, 0x14, o) whose +0xe0
/// mask meets player +0x1000
pub fn ee_sub_1cca40<M: QMem + ?Sized>(m: &mut M, pl: u32, o: u32) -> u32 {
    let buf = stack_push(m, 0x10);
    let n = m.call(0x0015_e9c8, &[0x003d_d490, 0x401, 0x14, o, 4, buf], &[]).0 as i32;
    let mut r = 0;
    for i in 0..n.max(0) as u32 {
        let x = m.r32(buf + 4 * i);
        if m.r32(x + 0xe0) & m.r32(pl + 0x1000) != 0 {
            r = x;
            break;
        }
    }
    stack_pop(m, 0x10);
    r
}

/// FUN_001ccac0(p, s): aim state change (+0xa4; leaving 1 frees +0x94 / +0x9c, +0xa8 = DAT_002dded0).
/// -1: free +0x80, mode -1, target cleared. 0 / 1: target +0x84 -> mode 0 (kind 10: path, param +0x88 =
/// FUN_001e6f60 nearest to the player), 2 (kind 0x14 matching player +0x1000, or kind 0x30 with a
/// FUN_001cca40 hit as +0x8c) else 1; mode 2 grabs handle +0x80 = FUN_0012dc18(player, node, 5, 7, 3,
/// FUN_0022ca18(node)); +0x1c = +8 = 0.5, FUN_001384c0(+0x30), FUN_00149ab8(0x3dd490, player, DAT_002a77c8, p)
pub fn ee_sub_1ccac0<M: QMem + ?Sized>(m: &mut M, p: u32, s: u32) {
    let old = m.r32(p + 0xa4);
    if s == old {
        return;
    }
    if old == 1 {
        m.call(0x0012_e460, &[p + 0x94], &[]);
        m.call(0x0012_e460, &[p + 0x9c], &[]);
    }
    m.w32(p + 0xa4, s);
    let t = m.r32(0x002d_ded0);
    m.w32(p + 0xa8, t);
    let si = s as i32;
    if si == -1 {
        m.call(0x0012_e460, &[p + 0x80], &[]);
        m.w32(p + 0x90, s);
        m.w32(p + 0x8c, 0);
        m.w32(p + 0x84, 0);
        m.w32(p + 0x88, 0);
        return;
    }
    if !(-1..2).contains(&si) {
        return;
    }
    let tg = m.r32(p + 0x84);
    m.w32(p + 0x90, 1);
    m.w32(p + 0x8c, tg);
    if m.call(0x0013_2b10, &[tg, 10], &[]).0 != 0 {
        let pl = m.r32(p);
        let pos = m.call(0x001a_62e8, &[pl], &[]).1;
        let tg = m.r32(p + 0x84);
        m.call(0x001e_6f60, &[tg, 0, 0, 0, 0, p + 0x88], &[pos]);
        m.w32(p + 0x90, 0);
    } else {
        let tg = m.r32(p + 0x84);
        let pl = m.r32(p);
        let mut two = m.call(0x0013_2b10, &[tg, 0x14], &[]).0 != 0
            && m.r32(tg + 0xe0) & m.r32(pl + 0x1000) != 0;
        if !two && m.call(0x0013_2b10, &[tg, 0x30], &[]).0 != 0 {
            let pl = m.r32(p);
            let tg = m.r32(p + 0x84);
            let h = ee_sub_1cca40(m, pl, tg);
            if h != 0 {
                m.w32(p + 0x8c, h);
                two = true;
            }
        }
        if two {
            m.w32(p + 0x90, 2);
        }
    }
    if m.r32(p + 0x90) == 2 {
        let nd = m.r32(p + 0x38);
        let q = m.call(0x0022_ca18, &[nd, nd], &[]).1;
        let pl = m.r32(p);
        let nd = m.r32(p + 0x38);
        let h = m.call(0x0012_dc18, &[pl, nd, 5, 7, 3], &[q]).0;
        m.w32(p + 0x80, h);
    }
    m.wf(p + 0x1c, 0.5);
    m.wf(p + 8, 0.5);
    let an = m.r32(p + 0x30);
    m.call(0x0013_84c0, &[an], &[]);
    let pl = m.r32(p);
    // a2 = 64-bit unaligned load of DAT_002a77c8: low word passed (callers use the 32-bit flag word)
    let k = m.r32(0x002a_77c8);
    m.call(0x0014_9ab8, &[0x003d_d490, pl, k, p], &[]);
}
