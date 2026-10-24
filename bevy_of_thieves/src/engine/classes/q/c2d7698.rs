//! WP-Q7: class C2d7698 (parent C2d7468; particle with bounce) queue package: slots 1 / 5 / 7, the
//! shared bounce response C2d7468 slot 7 (FUN_001ceb08) and its fade helper FUN_001ceaa8. Written from
//! the disassembly (VU0 macro code). Particle layout: +0 kind, +0x14 / +0x18 life start / length,
//! +0x20.. size curve, +0x40 colour, +0x70 position, +0x90 velocity, +0x98 alpha floor, +0xa0 accel,
//! +0xd0 spin, +0xf0..+0xf8 bounce params (scale, restitution, friction).
//! VU quirk kept: `vadda ACC, vf0, x` (ACC = x + vf0) adds 1.0 to the w lane of every such sum; the w lane
//! of cross-product temporaries (`vopmula/vopmsub` keep the stale register w) is modelled as 0.
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem, V4};
use crate::engine::classes::q::q7util::*;

pub fn dot3(a: V4, b: V4) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
pub fn cross(a: V4, b: V4) -> V4 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0], 0.0]
}
/// `vadda ACC, vf0, a` then `vmaddx d, b, s`: a + vf0 + b * s
pub fn acc1(a: V4, b: V4, s: f32) -> V4 {
    let mut r = add(a, scale(b, s));
    r[3] += 1.0;
    r
}

/// FUN_001ceaa8: shorten the remaining life of particle `p` to at most `x` seconds keeping its fraction
pub fn ee_sub_1ceaa8<M: QMem + ?Sized>(m: &mut M, x: f32, p: u32) {
    let clk = m.rf(CLOCK);
    let len = m.rf(p + 0x18);
    let rem = len - (clk - m.rf(p + 0x14));
    if x < rem {
        // VER5: PS2 FPU semantics (div.s x/0 = +-max, overflow clamps), as the EE computes it
        use crate::engine::math::ps2f::{ps2_div, ps2_mul, ps2_sub};
        let r = ps2_div(rem, len);
        let nl = ps2_div(x, r);
        m.wf(p + 0x18, nl);
        m.wf(p + 0x14, ps2_sub(clk, ps2_mul(1.0 - r, nl)));
    }
}

/// FUN_001ceb08 (C2d7468 slot 7): bounce of particle `p` at contact point `cp` with normal `n`
/// (ctx +0 surface, +4 collider). Collider with flag 0x59: stick (FUN_00228c18 attach point, velocity
/// lerp DAT_002c1f64, accel over DAT_002c1f60, fade). Else rigid-body impulse with restitution / friction
/// (radius max(scale * size curve, 10), inertia 2.5 / r^2), split between the two candidate impulses by
/// FUN_0011ee08, spin update, alpha floor, push out of the surface. Returns 1.
pub fn ee_sub_1ceb08<M: QMem + ?Sized>(m: &mut M, p: u32, ctx: u32, cp: V4, n: V4) -> u32 {
    let (mut r, rest, fric) = if m.r32(p) == 8 {
        (m.rf(p + 0xf0), m.rf(p + 0xf4), m.rf(p + 0xf8))
    } else {
        (100.0, f32::from_bits(0x3f33_3333), f32::from_bits(0x3f4c_cccd))
    };
    let t = (m.rf(CLOCK) - m.rf(p + 0x14)) / m.rf(p + 0x18);
    r *= m.rf(p + 0x20) + t * (m.rf(p + 0x24) + t * m.rf(p + 0x28));
    let col = m.r32(ctx + 4);
    if m.call(0x0013_2b10, &[col, 0x59], &[]).0 != 0 {
        let sp = stack_push(m, 0x10);
        m.call(0x0022_8c18, &[col, sp, 0], &[cp]);
        let k = m.rf(0x002c_1f64);
        let v = m.rv(p + 0x90);
        let a = m.rv(sp);
        let nv = acc1(v, sub(a, v), k);
        m.wv(p + 0x90, nv);
        let c = m.rf(0x002c_1f60);
        m.wv(p + 0xa0, scale(sub(a, nv), 1.0 / c));
        ee_sub_1ceaa8(m, c, p);
        stack_pop(m, 0x10);
        return 1;
    }
    let sp = stack_push(m, 0x50);
    let surf = m.r32(ctx);
    let arm = scale(n, -r); // vf10
    let t1 = cross(arm, n);
    let c = m.call(0x0022_d348, &[surf, 0], &[cp]).1;
    let r = r.max(10.0);
    let k1 = 2.5 / (r * r);
    let spin = m.rv(p + 0xd0);
    let mut v1 = add(m.rv(p + 0x90), cross(spin, arm));
    v1[3] = m.rf(p + 0x90 + 12) + spin[3];
    let rel = sub(v1, c); // vf9
    let d1 = dot3(rel, n);
    // normal impulse j1
    let kn = acc1(n, cross(t1, arm), k1);
    let j1 = d1 * (-1.0 - rest) / dot3(kn, n);
    let v2 = acc1(rel, kn, j1);
    let tv = sub(v2, scale(n, dot3(v2, n)));
    let len = dot3(tv, tv).sqrt();
    let dir = if len < f32::from_bits(0x38d1_b717) { m.rv(0x002a_2da0) } else { scale(tv, 1.0 / len) };
    // friction impulse direction D, j2
    let d = acc1(n, dir, -fric);
    let t2 = cross(arm, d);
    let kd = acc1(d, cross(t2, arm), k1);
    let j2 = d1 * (-1.0 - rest) / dot3(kd, n);
    let v3 = acc1(rel, kd, j2);
    let tv2 = sub(v3, scale(n, dot3(v3, n)));
    let q = m.rv(0x002a_2dd0);
    m.call(0x0011_ee08, &[0, sp + 0x40, sp + 0x44], &[q, tv, tv2]);
    let s = m.rf(sp + 0x40);
    let (a, b) = ((1.0 - s) * j1, s * j2);
    let imp = add(scale(n, a), scale(d, b));
    let v = m.rv(p + 0x90);
    m.wv(p + 0x90, add(v, imp));
    let w = add(scale(t1, a), scale(t2, b));
    let sp0 = m.rv(p + 0xd0);
    m.wv(p + 0xd0, acc1(sp0, w, k1));
    let fl = m.rf(0x002c_1f68);
    if m.rf(p + 0x98) < fl {
        m.wf(p + 0x98, fl);
        let x = m.rf(0x002c_1f6c);
        ee_sub_1ceaa8(m, x, p);
    }
    let pos = m.rv(p + 0x70);
    let pen = dot3(sub(cp, pos), n).max(0.0) + 1.0;
    m.wv(p + 0x70, acc1(pos, n, pen));
    stack_pop(m, 0x50);
    1
}

/// FUN_001cfab8 (slot 1, clone from `src`): parent slot 1 (FUN_001ce9b8), life *= random
/// [DAT_002c1f78, DAT_002c1f7c], copy the bounce params, random spin +-10 * clamp(speed curve).
pub fn c2d7698_v1<M: QMem + ?Sized>(m: &mut M, p: u32, src: u32) {
    m.call(0x001c_e9b8, &[p, src], &[]);
    let (a, b) = (m.rf(0x002c_1f78), m.rf(0x002c_1f7c));
    let k = fr(m.call(0x0011_d448, &[], &[f(a), f(b)]));
    let v = m.rv(p + 0x90);
    let sp = dot3(v, v).sqrt();
    let l = m.rf(p + 0x18) * k;
    m.wf(p + 0x18, l);
    copy(m, p + 0xf0, src + 0xf0, 0xc);
    let c = m.rf(0x002c_1f80) + sp * (m.rf(0x002c_1f84) + sp * m.rf(0x002c_1f88));
    let (lo, hi) = (m.rf(0x002b_bb60), m.rf(0x002b_bb64));
    let c = if c < lo {
        lo
    } else if hi < c {
        hi
    } else {
        c
    };
    let x = c * 10.0;
    for i in 0..3 {
        let r = fr(m.call(0x0011_d448, &[], &[f(-x), f(x)]));
        m.wf(p + 0xd0 + 4 * i, r);
    }
}

/// FUN_001cfbe0 (slot 7): collider (ctx +4 -> +0xec -> +0x60) with flag 0x32: ignore (0); flag 0x59:
/// FUN_001ce980 (1); else the bounce FUN_001ceb08.
pub fn c2d7698_v7<M: QMem + ?Sized>(m: &mut M, p: u32, ctx: u32, cp: V4, n: V4) -> u32 {
    let c = m.r32(ctx + 4);
    let e = m.r32(c + 0xec);
    let u = m.r32(e + 0x60);
    if m.call(0x0013_2b10, &[u, 0x32], &[]).0 != 0 {
        0
    } else if m.call(0x0013_2b10, &[u, 0x59], &[]).0 != 0 {
        m.call(0x001c_e980, &[p], &[]);
        1
    } else {
        // jal 0x1ceb08 (C2d7468_v7) through the call table
        m.call(0x001c_eb08, &[p, ctx], &[cp, n]).0
    }
}

/// FUN_001cfc88 (slot 5): FUN_001cf338(p, pos + vf0 + DAT_002a2dc0 * +0xf0, p + 0x40)
pub fn c2d7698_v5<M: QMem + ?Sized>(m: &mut M, p: u32) {
    let k = m.rf(p + 0xf0);
    let off = m.rv(0x002a_2dc0);
    let pos = m.rv(p + 0x70);
    m.call(0x001c_f338, &[p, p + 0x40], &[acc1(pos, off, k)]);
}
