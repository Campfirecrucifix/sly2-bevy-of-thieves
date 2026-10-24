//! PORT2: VU0 macro-mode bodies of game-logic functions, ported 1:1 from the disassembly
//! (python tools/eedis.py <addr>). VU0 `vmula/vmadda/vmadd` broadcast chains are `bc3`; dest masks
//! (.xyz) keep the destination register's other lanes, as on the VU.
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem, V4};

/// c0 * v.x + c1 * v.y + c2 * v.z on .xyz; w = `keep_w` (dest register's previous w)
fn bc3(c: &[V4; 3], v: V4, keep_w: f32) -> V4 {
    let mut o: V4 = std::array::from_fn(|i| c[0][i] * v[0] + c[1][i] * v[1] + c[2][i] * v[2]);
    o[3] = keep_w;
    o
}
fn rows3<M: QMem + ?Sized>(m: &mut M, a: u32) -> [V4; 3] {
    [m.rv(a), m.rv(a + 0x10), m.rv(a + 0x20)]
}

/// FUN_001f4558 (C2caf88 vtable +0x10c): angular velocity of the ALO's physics record +0x5c.
/// rec == 0 or rec +0x94 != 6 -> the quadword at DAT_002a2dd0. Else T = FUN_0022d148(rec, 0) rows 0..2,
/// A = OrthonormalizeMatrix (0x11b3d0) of rec +0x40 rows, D = FUN_0011b340(rec +0x40) rows;
/// P_i = D_i * A, Q_i = T_i * P; result = (Q[1].z, Q[2].x, Q[0].y, w) with w = the sign-extension word of
/// `mfc1 Q[0].y` (pcpyld of the 64-bit GPR: 0xffffffff when Q[0].y is negative, else 0).
pub fn ee_sub_1f4558<M: QMem + ?Sized>(m: &mut M, obj: u32) -> V4 {
    // 0x001f4558
    let rec = m.r32(obj + 0x5c);
    if rec == 0 || m.r32(rec + 0x94) != 6 {
        return m.rv(0x002a_2dd0);
    }
    let sp = stack_push(m, 0x170);
    m.call(0x0022_d148, &[sp + 0x60, rec, 0], &[]);
    let t = rows3(m, sp + 0x60);
    for r in 0..3 {
        let v = m.rv(rec + 0x40 + r * 0x10);
        m.wv(sp + r * 0x10, v);
    }
    m.call(0x0011_b3d0, &[sp], &[]);
    m.call(0x0011_b340, &[sp + 0xd0, rec + 0x40], &[]);
    let a = rows3(m, sp);
    let d = rows3(m, sp + 0xd0);
    stack_pop(m, 0x170);
    let p = [bc3(&a, d[0], d[0][3]), bc3(&a, d[1], d[1][3]), bc3(&a, d[2], d[2][3])];
    let q = [bc3(&p, t[0], t[0][3]), bc3(&p, t[1], t[1][3]), bc3(&p, t[2], t[2][3])];
    let w = if (q[0][1].to_bits() as i32) < 0 { f32::from_bits(u32::MAX) } else { 0.0 };
    [q[1][2], q[2][0], q[0][1], w]
}

fn vmin(a: V4, b: V4) -> V4 {
    [a[0].min(b[0]), a[1].min(b[1]), a[2].min(b[2]), a[3]]
}
fn vmax(a: V4, b: V4) -> V4 {
    [a[0].max(b[0]), a[1].max(b[1]), a[2].max(b[2]), a[3]]
}
/// |a - b| over xyz (vsub / vmul.xyz / vadda.x + vmadd.x / vsqrt)
fn dist(a: V4, b: V4) -> f32 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}
/// c2 * p.x + c4 * p.y + c3 * p.z + c7 (vmulax/vmadday/vmaddaz/vmaddw vf00w)
fn xf_pt(r: [V4; 4], p: V4) -> V4 {
    std::array::from_fn(|i| if i < 3 { r[0][i] * p[0] + r[1][i] * p[1] + r[2][i] * p[2] + r[3][i] } else { p[3] })
}

/// Collision part points of `s0` (part kinds without the sphere bit 49): the +0x94 point list
/// (count +0x90), then, when +0xa8 != +0xac, the +0x7c points [+0x1c, +0x90) through the part matrix
/// rows +0x20 / +0x30 / +0x40 / +0x50 (translation).
fn part_points<M: QMem + ?Sized>(m: &mut M, s0: u32, f: &mut dyn FnMut(V4)) {
    let n = m.r32(s0 + 0x90) as i32;
    let p = m.r32(s0 + 0x94);
    for i in 0..n.max(0) as u32 {
        let v = m.rv(p + i * 0x10);
        f(v);
    }
    xf_points(m, s0, f);
}
fn xf_points<M: QMem + ?Sized>(m: &mut M, s0: u32, f: &mut dyn FnMut(V4)) {
    if m.r32(s0 + 0xa8) == m.r32(s0 + 0xac) {
        return;
    }
    let n = m.r32(s0 + 0x90) as i32;
    let start = m.r32(s0 + 0x1c) as i32;
    if start >= n {
        return;
    }
    let r = [m.rv(s0 + 0x20), m.rv(s0 + 0x30), m.rv(s0 + 0x40), m.rv(s0 + 0x50)];
    let base = m.r32(s0 + 0x7c);
    for i in start..n {
        let v = m.rv(base.wrapping_add((i as u32) << 4));
        f(xf_pt(r, v));
    }
}

/// FUN_001f50e0 (C2caf88 slot 47): bounds of the collision parts (list *(obj +0xec) +0x34, next +0x60).
/// Pass 1 (parts without flag bit 50, each refreshed by FUN_001f38a8 first): AABB min / max from
/// DAT_002a2df0 / DAT_002a2e00 over sphere parts (bit 49: centre *(part) +0x70 -+ DAT_002a2de0 * radius
/// +8, plus vf00 (0,0,0,1)) or the part points. rec +0 = min, +0x10 = max, +0x20 = c = (min + max) / 2
/// (xyzw), +0x30 = r = max over every part (no bit-50 skip) of |c - centre| + radius / |c - point|.
/// Then DAT_003dd490 +0x3ccc = 1 and FUN_001f4ba8(obj, rec, 1).
pub fn ee_sub_1f50e0<M: QMem + ?Sized>(m: &mut M, obj: u32) {
    // 0x001f50e0
    let mut lo = m.rv(0x002a_2df0);
    let mut hi = m.rv(0x002a_2e00);
    let unit = m.rv(0x002a_2de0);
    let rec = m.r32(obj + 0xec);
    let mut s0 = m.r32(rec + 0x34);
    while s0 != 0 {
        if m.r32(s0 + 0x14) & (1 << 18) == 0 {
            m.call(0x001f_38a8, &[s0], &[]);
            if m.r32(s0 + 0x14) & (1 << 17) != 0 {
                let r = m.rf(s0 + 8);
                let c0 = m.r32(s0);
                let c = m.rv(c0 + 0x70);
                let a: V4 = std::array::from_fn(|i| c[i] + if i == 3 { 1.0 } else { 0.0 } + unit[i] * -r);
                lo = vmin(lo, a);
                let c = m.rv(c0 + 0x70);
                let b: V4 = std::array::from_fn(|i| c[i] + if i == 3 { 1.0 } else { 0.0 } + unit[i] * r);
                hi = vmax(hi, b);
                xf_points(m, s0, &mut |v| {
                    hi = vmax(hi, v);
                    lo = vmin(lo, v);
                });
            } else {
                part_points(m, s0, &mut |v| {
                    hi = vmax(hi, v);
                    lo = vmin(lo, v);
                });
            }
        }
        s0 = m.r32(s0 + 0x60);
    }
    let rec = m.r32(obj + 0xec);
    m.wv(rec, lo);
    let c: V4 = std::array::from_fn(|i| lo[i] * 0.5 + hi[i] * 0.5);
    m.wv(rec + 0x10, hi);
    let mut rad = 0.0f32;
    let mut s0 = m.r32(rec + 0x34);
    while s0 != 0 {
        let next = m.r32(s0 + 0x60);
        let sphere = m.r32(s0 + 0x14) & (1 << 17) != 0;
        if sphere {
            let r = m.rf(s0 + 8);
            let c0 = m.r32(s0);
            let d = dist(c, m.rv(c0 + 0x70)) + r;
            if rad < d {
                rad = d;
            }
        }
        let mut f = |v: V4| {
            let d = dist(c, v);
            if rad < d {
                rad = d;
            }
        };
        if sphere {
            xf_points(m, s0, &mut f);
        } else {
            part_points(m, s0, &mut f);
        }
        s0 = next;
    }
    let rec = m.r32(obj + 0xec);
    m.wv(rec + 0x20, c);
    m.wf(rec + 0x30, rad);
    m.w32(0x003d_d490 + 0x3ccc, 1);
    m.call(0x001f_4ba8, &[obj, rec, 1], &[]);
}

/// FUN_001c6de8(pos (u128 a0), a1, a2, out, down f12, up f13, min f14): ground / ceiling probe.
/// FUN_001ae478(&tmr) .. FUN_001ae4a0(&tmr, 2) bracket the body. dir = DAT_002a2dc0;
/// A = pos + vf00 + dir * up, B = pos + vf00 + dir * down (xyzw). a2 == 0 -> FUN_0020faa0(0x3dd490,
/// pos, A, 0x1c5570, 0, &a1, &a2). up > 0: FUN_0014cc58(0, pos, A, a1, a2, hit) != 0 with hit +0x40 == 0,
/// +0x44 != 1.0 and !(hit +0x38 < min) -> *out = hit +0x10, 1. Else down < 0: FUN_0014cc58(0, pos, B, ..)
/// != 0 with hit +0x40 != 0 and !(hit +0x28 < min) -> *out = hit +0, 1; failure -> 0. down >= 0 ->
/// *out = pos, 1.
#[allow(clippy::too_many_arguments)]
pub fn ee_sub_1c6de8<M: QMem + ?Sized>(m: &mut M, pos: V4, a1: u32, a2: u32, out: u32, down: f32, up: f32, min: f32) -> u32 {
    // 0x001c6de8
    let sp = stack_push(m, 0x110);
    let tmr = sp + 0x60;
    m.w32(sp + 0x80, a1);
    m.w32(sp + 0x84, a2);
    m.call(0x001a_e478, &[tmr], &[]);
    let dir = m.rv(0x002a_2dc0);
    let off = |k: f32| -> V4 { std::array::from_fn(|i| pos[i] + if i == 3 { 1.0 } else { 0.0 } + dir[i] * k) };
    let (b, a) = (off(down), off(up));
    if m.r32(sp + 0x84) == 0 {
        m.call(0x0020_faa0, &[0x003d_d490, 0x001c_5570, 0, sp + 0x80, sp + 0x84], &[pos, a]);
    }
    let mut res = None;
    let mut fail = false;
    if 0.0 < up {
        let (s80, s84) = (m.r32(sp + 0x80), m.r32(sp + 0x84));
        let r = m.call(0x0014_cc58, &[0, s80, s84, sp], &[pos, a]).0;
        if r != 0 && m.rf(sp + 0x40) == 0.0 {
            if m.rf(sp + 0x44) == 1.0 || m.rf(sp + 0x38) < min {
                fail = true;
            } else {
                res = Some(m.rv(sp + 0x10));
            }
        }
    }
    if res.is_none() && !fail {
        if down < 0.0 {
            let (s80, s84) = (m.r32(sp + 0x80), m.r32(sp + 0x84));
            let r = m.call(0x0014_cc58, &[0, s80, s84, sp], &[pos, b]).0;
            if r != 0 && m.rf(sp + 0x40) != 0.0 && !(m.rf(sp + 0x28) < min) {
                res = Some(m.rv(sp));
            }
        } else {
            res = Some(pos);
        }
    }
    m.call(0x001a_e4a0, &[tmr, 2], &[]);
    stack_pop(m, 0x110);
    match res {
        Some(v) => {
            m.wv(out, v);
            1
        }
        None => 0,
    }
}

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

/// FUN_001cf6b0(emitter, a1, a2, p (u128 a3), d (u128 t0)): spawn a class-10 particle (FUN_001ce888(0x3dd490,
/// 10, 0)) with its basis. a1 == 0 or no particle -> 0. v = FUN_0022fc48(0, a2, p); u = (DAT_002dded0 - e +0x14)
/// / e +0x18, speed = (e +0x20 + u * (e +0x24 + u * e +0x28)) * FUN_0011d448(0.8, 1.4); (*(q +0xec) +0xc)(q +
/// adj, v, 0, speed); q +0x1c = a1, +0x18 = 5.0, +0x28 = -speed, +0xf0 = a2. B = FUN_0011e2d8(d, DAT_002a2da0,
/// DAT_002a2db0) rows, R = FUN_0011b628((0, 0, FUN_0011d448(0, 2pi))) rows; q +0x40 / +0x50 / +0x60 =
/// B * R0 / R1 / R2 (.xyz; the w lanes of vf04 / vf06 are stale registers in the original: 0 here, +0x50 w =
/// B0.w). |d . DAT_002a2dc0| < 0.7 -> q +0x90 = FUN_0011ba08((d.x, d.y, 0, d.w), d) rows * (DAT_002a2dc0 * -50)
/// (.xyz, w = -50 dir.w). q +0x78 += *(e +8) +0x158. Returns q.
pub fn ee_sub_1cf6b0<M: QMem + ?Sized>(m: &mut M, e: u32, a1: u32, a2: u32, p: V4, d: V4) -> u32 {
    // 0x001cf6b0
    if a1 == 0 {
        return 0;
    }
    let q = m.call(0x001c_e888, &[0x003d_d490, 10, 0], &[]).0;
    if q == 0 {
        return 0;
    }
    let v = m.call(0x0022_fc48, &[0, a2], &[p]).1;
    let u = (m.rf(0x002d_ded0) - m.rf(e + 0x14)) / m.rf(e + 0x18);
    let mut s = m.rf(e + 0x20) + u * (m.rf(e + 0x24) + u * m.rf(e + 0x28));
    s *= m.call(0x0011_d448, &[], &[f4(0.8), f4(1.4)]).1[0];
    let mo = m.r32(q + 0xec);
    let adj = m.r16(mo + 8) as i16 as i32;
    let f = m.r32(mo + 0xc);
    m.call(f, &[q.wrapping_add(adj as u32), 0], &[v, f4(s)]);
    m.w32(q + 0x1c, a1);
    m.wf(q + 0x18, 5.0);
    m.wf(q + 0x28, -s);
    m.w32(q + 0xf0, a2);
    let sp = stack_push(m, 0x190);
    m.wv(sp + 0x30, d);
    let (r0, r1) = (m.rv(0x002a_2da0), m.rv(0x002a_2db0));
    m.call(0x0011_e2d8, &[sp + 0x10], &[d, r0, r1]);
    let ang = m.call(0x0011_d448, &[], &[f4(0.0), f4(std::f32::consts::TAU)]).1[0];
    let w4 = m.rf(sp + 0x4c);
    m.call(0x0011_b628, &[sp + 0x80], &[[0.0, 0.0, ang, w4]]);
    let b = rows3(m, sp + 0x10);
    let r = rows3(m, sp + 0x80);
    m.wv(q + 0x40, bc3(&b, r[0], 0.0));
    m.wv(q + 0x50, bc3(&b, r[1], b[0][3]));
    m.wv(q + 0x60, bc3(&b, r[2], 0.0));
    let dir = m.rv(0x002a_2dc0);
    if (d[0] * dir[0] + d[1] * dir[1] + d[2] * dir[2]).abs() < 0.7 {
        m.call(0x0011_ba08, &[sp + 0xe0], &[[d[0], d[1], 0.0, d[3]], d]);
        let mm = rows3(m, sp + 0xe0);
        let k: V4 = std::array::from_fn(|i| dir[i] * -50.0);
        m.wv(q + 0x90, bc3(&mm, k, k[3]));
    }
    stack_pop(m, 0x190);
    let o = m.r32(e + 8);
    let x = m.rf(q + 0x78) + m.rf(o + 0x158);
    m.wf(q + 0x78, x);
    q
}
