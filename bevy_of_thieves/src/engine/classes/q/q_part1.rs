//! PART-1: ports of rows moved from 'skipped' to 'unported' by the PART-1 audit.
//! VU0 macro sequences are written as plain f32 lane math (same lane order / masks as the EE code).
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem, V4};
use crate::engine::classes::q::q5util::f4;

const UP: u32 = 0x002a_2dc0;

// all lane ops saturate like the VU0 (no inf / NaN, denormals flush): math::vu0::fix
use crate::engine::math::vu0::fix;
fn sub(a: V4, b: V4) -> V4 {
    std::array::from_fn(|i| fix(a[i] - b[i]))
}
fn add(a: V4, b: V4) -> V4 {
    std::array::from_fn(|i| fix(a[i] + b[i]))
}
fn scale(a: V4, s: f32) -> V4 {
    a.map(|x| fix(x * s))
}
/// vmul + vaddabc + vmaddbc: x*x + y*y + z*z, then vsqrt / vdiv vf00w: 1 / |a|
fn inv_len(a: V4) -> f32 {
    crate::engine::math::vu0::vdiv(1.0, crate::engine::math::vu0::vsqrt(dot3(a, a)))
}
/// vopmula(s, t) / vopmsub(t, s): s x t on xyz, w kept from `w`
fn cross(s: V4, t: V4, w: f32) -> V4 {
    let c = |a: f32, b: f32, d: f32, e: f32| fix(fix(a * b) - fix(d * e));
    [c(s[1], t[2], t[1], s[2]), c(s[2], t[0], t[2], s[0]), c(s[0], t[1], t[0], s[1]), w]
}
fn dot3(a: V4, b: V4) -> f32 {
    fix(fix(fix(a[0] * b[0]) + fix(a[1] * b[1])) + fix(a[2] * b[2]))
}

/// FUN_00221408(track, q, out_local, out_dir): camera-track frame at the track's current
/// parameter (+0x140): path (+0x10c) -> FUN_001e6f10 point/tangent; no circle (+300 == 0) -> the
/// object's xf (FUN_001a6318) direction and FUN_001a6f00(0, track, q); circle -> radius +0x160 around
/// +0x150 with tangent up x r. out_local = (rows of FUN_0011b830(dir, up)) . (q - origin), w = q.w - origin.w.
pub fn ee_sub_221408<M: QMem + ?Sized>(m: &mut M, o: u32, q: V4, out: u32, dir: u32) {
    let sp = stack_push(m, 0x40);
    let (origin, mat);
    if m.r32(o + 0x10c) != 0 {
        let u = m.rf(o + 0x140);
        let path = m.r32(o + 0x10c);
        m.call(0x001e_6f10, &[path, sp, dir], &[f4(u)]);
        origin = m.rv(sp);
        let (d, up) = (m.rv(dir), m.rv(UP));
        m.call(0x0011_b830, &[sp + 0x10], &[d, up]);
        mat = sp + 0x10;
    } else if m.r32(o + 0x12c) == 0 {
        let xf = m.call(0x001a_6318, &[o], &[]).0;
        let v = m.rv(xf);
        m.wv(dir, scale(v, inv_len(v)));
        let r = m.call(0x001a_6f00, &[0, o], &[q]).1;
        m.wv(out, r);
        stack_pop(m, 0x40);
        return;
    } else {
        let rad = m.rf(o + 0x160);
        let u = m.rf(o + 0x140);
        let r = m.call(0x0011_e1a0, &[], &[f4(crate::engine::math::ps2f::ps2_div(u, rad)), f4(rad), f4(0.0)]).1;
        origin = add(m.rv(o + 0x150), r);
        let up = m.rv(UP);
        let t = cross(up, r, up[3]);
        let d = scale(t, inv_len(t));
        m.wv(dir, d);
        m.call(0x0011_b830, &[sp + 0x10], &[d, up]);
        mat = sp + 0x10;
    }
    let dv = sub(q, origin);
    let (r0, r1, r2) = (m.rv(mat), m.rv(mat + 0x10), m.rv(mat + 0x20));
    m.wv(out, [dot3(r0, dv), dot3(r1, dv), dot3(r2, dv), dv[3]]);
    stack_pop(m, 0x40);
}

/// FUN_002215c0(u, du, track, k[3], poly[3][4], clamp[3][2], mode, out): world point of a camera
/// offset on a track: a[i] = clamp(poly0 + k*(poly1 + k*poly2)); mode 2 negates a.x; +0x120 -> a =
/// FUN_0011e1a0(a) (polar). No path/circle: out = FUN_001a6f00(track, 0, a + (du, 0, 0)). Else the frame
/// at u +- du (mode 1: +) on the circle (+0x150, radius +0x160, FUN_00245a18 wrap) or path (+0x10c,
/// FUN_001e6f10); out = origin + m0*a.x + m1*a.y + m2*a.z (FUN_0011b830(dir, up)).
#[allow(clippy::too_many_arguments)]
pub fn ee_sub_2215c0<M: QMem + ?Sized>(m: &mut M, u: f32, du: f32, o: u32, k: u32, poly: u32, clamp: u32, mode: u32, out: u32) {
    let mut a = [0.0f32; 4];
    for i in 0..3u32 {
        let (kk, p) = (m.rf(k + 4 * i), poly + 16 * i);
        let v = m.rf(p) + kk * (m.rf(p + 4) + kk * m.rf(p + 8));
        let (lo, hi) = (m.rf(clamp + 8 * i), m.rf(clamp + 8 * i + 4));
        a[i as usize] = if lo <= v { if hi < v { hi } else { v } } else { lo };
    }
    if mode == 2 {
        a[0] = -a[0];
    }
    if m.r32(o + 0x120) != 0 {
        a = m.call(0x0011_e1a0, &[], &[f4(a[0]), f4(a[1]), f4(a[2])]).1;
    }
    let sp = stack_push(m, 0x60);
    let path = m.r32(o + 0x10c);
    if path == 0 && m.r32(o + 0x12c) == 0 {
        a[0] += du;
        let r = m.call(0x001a_6f00, &[o, 0], &[a]).1;
        m.wv(out, r);
        stack_pop(m, 0x60);
        return;
    }
    let u = if mode == 1 { u + du } else { u - du };
    let (origin, dir);
    if path == 0 {
        let rad = m.rf(o + 0x160);
        let w = m.call(0x0024_5a18, &[], &[f4(u), f4(rad * 6.283_185_5)]).1[0];
        let r = m.call(0x0011_e1a0, &[], &[f4(w / rad), f4(rad), f4(0.0)]).1;
        let c = m.rv(o + 0x150);
        origin = add(r, c);
        let up = m.rv(UP);
        let t = cross(up, r, c[3]);
        dir = scale(t, inv_len(t));
    } else {
        m.call(0x001e_6f10, &[path, sp, sp + 0x10], &[f4(u)]);
        origin = m.rv(sp);
        dir = m.rv(sp + 0x10);
    }
    let up = m.rv(UP);
    m.call(0x0011_b830, &[sp + 0x20], &[dir, up]);
    let (r0, r1, r2) = (m.rv(sp + 0x20), m.rv(sp + 0x30), m.rv(sp + 0x40));
    // vmulax/vmadday/vmaddz.xyz: w keeps a.w
    let mut v = add(add(scale(r0, a[0]), scale(r1, a[1])), scale(r2, a[2]));
    v[3] = a[3];
    m.wv(out, add(origin, v));
    stack_pop(m, 0x60);
}

/// FUN_001b7700(o, q, &t, &tmax, out_v0, out_v1): launch velocities toward `q`.
/// +0x144c == 0: out_v0 / out_v1 = FUN_0022fcc8(0, frame, up * 2500 / up * -2000).
/// +0x144c == 1: body b = (o+0x13f0)->+0x180, s = b->+0x58: g = FUN_0022fcc8(0, s+0xa0, (b+0xe8)->+0x80),
/// v = s+0xb0, p = s+0x30; t = last root of SolveQuadratic(g.z/2, v.z, p.z - q.z) clamped
/// (< DAT_002c09c8 -> DAT_002c09c8, > tmax -> tmax); FUN_00130d30(*t - t * (o+0x1030)->+0x40, o+0x1030, 0);
/// v1 = v + g t, p1 = p + (v + v1) t/2, d = (q - p1) / t: out_v0 = v + d, out_v1 = v1 + d, *tmax = t.
/// Other modes leave both outputs untouched (the original stores uninitialised stack / vf2).
pub fn ee_sub_1b7700<M: QMem + ?Sized>(m: &mut M, o: u32, q: V4, pt: u32, ptmax: u32, out0: u32, out1: u32) {
    let mode = m.r32(o + 0x144c);
    let b0 = m.r32(o + 0x13f0);
    let b = m.r32(b0 + 0x180);
    let s = m.r32(b + 0x58);
    if mode == 0 {
        let fr = m.r32(s + 0xa0);
        let up = m.rv(UP);
        let r0 = m.call(0x0022_fcc8, &[0, fr], &[scale(up, 2500.0)]).1;
        let r1 = m.call(0x0022_fcc8, &[0, fr], &[scale(up, -2000.0)]).1;
        m.wv(out0, r0);
        m.wv(out1, r1);
    } else if mode == 1 {
        let v = m.rv(s + 0xb0);
        let p = m.rv(s + 0x30);
        let ge = m.r32(b + 0xe8);
        let gi = m.rv(ge + 0x80);
        let fr = m.r32(s + 0xa0);
        let g = m.call(0x0022_fcc8, &[0, fr], &[gi]).1;
        let sp = stack_push(m, 0x20);
        let n = m.call(0x0011_d508, &[sp], &[f4(g[2] * 0.5), f4(v[2]), f4(p[2] - q[2])]).0 as i32;
        let lo = m.rf(0x002c_09c8);
        let mut t = lo;
        if n != 0 {
            let r = m.rf(sp + 4 * (n as u32 - 1));
            if lo <= r {
                t = m.rf(ptmax);
                if r <= t {
                    t = r;
                }
            }
        }
        stack_pop(m, 0x20);
        let a = m.r32(o + 0x1030);
        let x = m.rf(pt) - t * m.rf(a + 0x40);
        m.call(0x0013_0d30, &[a, 0], &[f4(x)]);
        let v1 = add(v, scale(g, t));
        m.wf(ptmax, t);
        let p1 = add(p, scale(add(v, v1), t * 0.5));
        let d = scale(sub(q, p1), 1.0 / t);
        m.wv(out0, add(v, d));
        m.wv(out1, add(v1, d));
    }
}

fn lerp(a: V4, b: V4, t: f32) -> V4 {
    add(scale(a, 1.0 - t), scale(b, t))
}
fn neg(a: V4) -> V4 {
    [-a[0], -a[1], -a[2], -a[3]]
}

/// FUN_001bc688(hull, p0, p1, max, out): clip segment p0 -> p1 against the convex hull's planes
/// (+0x1ec, +0xe8 - 1 of them, (n, d) with f = n.p - d + DAT_002c0ab4) after FUN_001bd400(hull).
/// Records of 0x60 bytes (memset FUN_002333b4): disjoint (tout <= tin, or both ends in front of a
/// plane): {+0 p0, +0x10 p1, +0x40 0, +0x44 1.0}; else an entry {+0 p0, +0x10 lerp(tin), +0x30 -n_in,
/// +0x40 0, +0x44 tin, +0x50 hull} when tin > 0 and an exit {+0 lerp(tout), +0x10 p1, +0x20 -n_out,
/// +0x40 tout, +0x44 1.0, +0x4c hull} when tout < 1, at most `max`. Returns the record count.
pub fn ee_sub_1bc688<M: QMem + ?Sized>(m: &mut M, o: u32, p0: V4, p1: V4, max: i32, out: u32) -> i32 {
    m.call(0x001b_d400, &[o], &[]);
    let (mut tin, mut tout) = (0.0f32, 1.0f32);
    let (mut n_in, mut n_out) = ([0.0f32; 4], [0.0f32; 4]);
    let mut i = 0i32;
    let disjoint = loop {
        if m.r32(o + 0xe8) as i32 - 1 <= i {
            break tout <= tin;
        }
        let pa = m.r32(o + 0x1ec);
        let pl = m.rv(pa + 16 * i as u32);
        let eps = m.rf(0x002c_0ab4);
        let f0 = dot3(pl, p0) - pl[3] + eps;
        let f1 = dot3(pl, p1) - pl[3] + eps;
        i += 1;
        if f0 < 0.0 && f1 < 0.0 {
            continue;
        }
        if 0.0 < f0 && 0.0 < f1 {
            tin = 0.0;
            tout = 0.0;
            break true;
        }
        let t = f0 / (f0 - f1);
        if f0 < 0.0 {
            tout = tout.min(t);
            n_out = pl;
        } else {
            n_in = pl;
            tin = tin.max(t);
        }
        if tout <= tin {
            break true;
        }
    };
    if disjoint && 0 < max {
        m.call(0x0023_33b4, &[out, 0, 0x60], &[]);
        m.wv(out, p0);
        m.wf(out + 0x44, 1.0);
        m.w32(out + 0x40, 0);
        m.wv(out + 0x10, p1);
        return 1;
    }
    let mut n = 0i32;
    if 0.0 < tin && 0 < max {
        let r = out;
        m.call(0x0023_33b4, &[r, 0, 0x60], &[]);
        n = 1;
        m.wv(r, p0);
        m.wv(r + 0x10, lerp(p0, p1, tin));
        m.wv(r + 0x30, neg(n_in));
        m.wf(r + 0x40, 0.0);
        m.wf(r + 0x44, tin);
        m.w32(r + 0x50, o);
    }
    if tout < 1.0 && n < max {
        let r = out + n as u32 * 0x60;
        m.call(0x0023_33b4, &[r, 0, 0x60], &[]);
        n += 1;
        m.w32(r + 0x4c, o);
        m.wv(r, lerp(p0, p1, tout));
        m.wv(r + 0x20, neg(n_out));
        m.wv(r + 0x10, p1);
        m.wf(r + 0x40, tout);
        m.wf(r + 0x44, 1.0);
    }
    n
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::classes::q::q2mem::FlatMem;

    /// FUN_001a8b50 (disassembly 0x1a8b50-0x1a90d0): one A edge whose B faces have no BSP path (0x1a86b8 = 0) ->
    /// the face with the smaller plane distance to [A]+0x70, its vertex u16[e+0x54 + sel*2], kind 1 contact
    /// (d = plane . v - w); n1 = 0 and the list head unchanged -> the kind 0xd fallback at [A]+0x70 / DAT_002a2dc0.
    #[test]
    fn mesh_1a8b50_vertex_face_and_fallback() {
        fn route(m: &mut FlatMem, addr: u32, a: &[u32], _v: &[V4]) -> Option<(u32, V4)> {
            match addr {
                0x001a_80d8 => Some((1, [0.0; 4])),
                0x001a_8410 if a[0] == 1 => {
                    m.w32(a[5], 0x5000);
                    Some((0, [0.0; 4]))
                }
                _ => None,
            }
        }
        let mut m = FlatMem { route: Some(route), ..Default::default() };
        let (a, b, o, pts, e, ee, fa, fb, list) = (0x1000, 0x2000, 0x1800, 0x3000, 0x5000, 0x5800, 0x6000, 0x6020, 0x7000);
        m.w32(a, o);
        m.wv(o + 0x70, [0., 0., 0., 1.]);
        m.w32(a + 0x94, pts);
        m.wv(pts, [1., 2., 3., 1.]);
        m.wv(pts + 16, [4., 5., 6., 1.]);
        m.w32(b + 0x9c, fa);
        m.w32(e + 0x4c, fa);
        m.w32(e + 0x50, fb);
        m.w32(e + 0x54, ee);
        m.w16(ee + 2, 1);
        m.wv(e + 0x20, [0., 0., 1., 5.]); // d1 = -5
        m.wv(e + 0x30, [0., 0., 1., -2.]); // d2 = 2 -> face +0x50, sel = +0x58 = 0
        m.wv(fb, [1., 0., 0., 0.5]);
        m.wv(0x002a_2dc0, [0., 0., 1., 0.]);
        ee_sub_1a8b50(&mut m, 0x100, a, b, 0, 0, 0, list);
        let c: Vec<_> = m.calls.iter().filter(|c| c.0 == 0x001a_7c18).cloned().collect();
        assert_eq!(c.len(), 2);
        assert_eq!(c[0].1, vec![0x100, a, b, 1, pts, fb, list]);
        assert_eq!(c[0].2, vec![[0.5, 0., 0., 0.], [1., 2., 3., 1.], [1., 0., 0., 0.5]]);
        assert_eq!(c[1].1, vec![0x100, a, b, 0xd, 0, 0, list]);
        assert_eq!(c[1].2, vec![[0.0; 4], [0., 0., 0., 1.], [0., 0., 1., 0.]]);
        let ord: Vec<u32> = m.called().into_iter().filter(|&x| x != 0x0025_64b8).collect();
        assert_eq!(ord, vec![0x001a_80d8, 0x001a_81f8, 0x001a_8410, 0x001a_8410, 0x001a_87f0, 0x001a_87f0, 0x001a_86b8, 0x001a_7c18, 0x001a_7c18]);
    }

    /// FUN_001a8b50 edge / edge: 0x1a86b8 finds one B segment, FUN_0011ea60 st = (0.5, 0.25) -> pa = lerp(A seg),
    /// pb = lerp(B seg), kind 3 contact with depth -|pa - pb|, pos pa, the 0x11ea60 normal; no fallback (n1 = 0 but
    /// the list head is FUN_001a7c18's job: FlatMem leaves it, so the 0xd add follows).
    #[test]
    fn mesh_1a8b50_edge_edge() {
        fn route(m: &mut FlatMem, addr: u32, a: &[u32], _v: &[V4]) -> Option<(u32, V4)> {
            match addr {
                0x001a_80d8 => Some((1, [0.0; 4])),
                0x001a_8410 if a[0] == 1 => {
                    m.w32(a[5], 0x5000);
                    Some((0, [0.0; 4]))
                }
                0x001a_86b8 => {
                    m.w32(a[4], 0x5100);
                    Some((1, [0.0; 4]))
                }
                0x0011_ea60 => {
                    m.wf(a[0], 0.5);
                    m.wf(a[0] + 4, 0.25);
                    m.wv(a[1], [0., 1., 0., 0.]);
                    Some((1, [0.0; 4]))
                }
                _ => None,
            }
        }
        let mut m = FlatMem { route: Some(route), ..Default::default() };
        let (a, b, pa_, pb_, list) = (0x1000, 0x2000, 0x3000, 0x3800, 0x7000);
        m.w32(a + 0x94, pa_);
        m.w32(b + 0x94, pb_);
        m.wv(pa_, [0., 0., 0., 1.]);
        m.wv(pa_ + 16, [2., 0., 0., 1.]);
        m.wv(pb_, [0., 3., 0., 1.]);
        m.wv(pb_ + 16, [4., 3., 0., 1.]);
        m.w32(b + 0x9c, 0x6000);
        m.w32(0x5000 + 0x4c, 0x6000);
        m.w32(0x5000 + 0x50, 0x6040);
        m.w32(0x5000 + 0x54, 0x5800);
        m.w16(0x5802, 1);
        m.w32(0x5100 + 0x54, 0x5900);
        m.w16(0x5902, 1);
        ee_sub_1a8b50(&mut m, 0x100, a, b, 0, 0, 0, list);
        let i = m.calls.iter().position(|c| c.0 == 0x001a_86b8).unwrap();
        assert_eq!(m.calls[i].1, vec![0x6000, 0, 2, 0, m.calls[i].1[4]]);
        let c: Vec<_> = m.calls.iter().filter(|c| c.0 == 0x001a_7c18).cloned().collect();
        assert_eq!(c[0].1, vec![0x100, a, b, 3, 0x5800, 0x5900, list]);
        assert_eq!(c[0].2, vec![[-3.0, 0., 0., 0.], [1., 0., 0., 1.], [0., 1., 0., 0.]]);
        assert_eq!(m.calls.iter().find(|c| c.0 == 0x0011_ea60).unwrap().2, vec![[0., 0., 0., 1.], [2., 0., 0., 1.], [0., 3., 0., 1.], [4., 3., 0., 1.]]);
    }

    /// FUN_001a87f0 (decomp text_001a0000.c): vertex P = b+0x94[verts[0]]; chain faces F1 (dmin 1) / F2 (dmin 0.5)
    /// -> F1 kept (largest min distance over P and the chain's far vertices u16[+0x54 + (1 - sel) * 2]); visited bit
    /// 8 cleared; kind 1 (d0 = 1, P, F1) then kind 2 for each chain point +0x10 below d0 (node 1 only).
    #[test]
    fn mesh_1a87f0_pick_face() {
        let mut m = FlatMem::default();
        let (b, pts, verts, chains, n1, n2, f1, f2, e1, e2) = (0x1000, 0x3000, 0x4000, 0x4100, 0x5000, 0x5100, 0x6000, 0x6040, 0x5800, 0x5900);
        m.w32(b + 0x94, pts);
        m.wv(pts, [0., 0., 0., 1.]);
        m.wv(pts + 16, [1., 0., 0., 1.]);
        m.wv(pts + 32, [0., 1., 0., 1.]);
        m.wf(0x002a_6ad0, -1.0e30);
        m.w32(chains, n1);
        for (n, f, e, nx, q) in [(n1, f1, e1, n2, [0., 0., -0.5, 1.]), (n2, f2, e2, 0, [0., 0., 5., 1.])] {
            m.w32(n + 0x50, f);
            m.w32(n + 0x54, e);
            m.w32(n + 0x5c, nx);
            m.wv(n + 0x10, q);
        }
        m.w16(e1 + 2, 1);
        m.w16(e2 + 2, 2);
        m.wv(f1, [0., 0., 1., -1.]);
        m.wv(f2, [1., 0., 0., -0.5]);
        ee_sub_1a87f0(&mut m, 0x100, b, 0x200, 1, verts, chains, 0x7000);
        let c: Vec<_> = m.calls.iter().filter(|c| c.0 == 0x001a_7c18).cloned().collect();
        assert_eq!(c.len(), 2);
        assert_eq!(c[0].1, vec![0x100, b, 0x200, 1, pts, f1, 0x7000]);
        assert_eq!(c[0].2, vec![[1., 0., 0., 0.], [0., 0., 0., 1.], [0., 0., 1., -1.]]);
        assert_eq!(c[1].1, vec![0x100, b, 0x200, 2, e1, f1, 0x7000]);
        assert_eq!(c[1].2, vec![[0.5, 0., 0., 0.], [0., 0., -0.5, 1.], [0., 0., 1., -1.]]);
        assert_eq!(m.r32(f1 + 0x18) & 8, 0);
        assert_eq!(m.r32(f2 + 0x18) & 8, 0);
    }

    /// FUN_001aa1d8 (disassembly 0x1aa1d8): body a1 radius 10 < a2 radius 20 -> fp = a2 (outer loop), s5 = a1.
    /// Plain SOs (no bit 48 / 49, no geom +0x18, +0xa8 == +0xac): FUN_00118a88(fp SO bsp +0x74, 0x7fffffff, &n,
    /// &end) against s5's sphere (+0x20, +0x30) returns 2 with end = buf + 24 -> sp40 = 2, sp3c = buf, sp24 = end - 12;
    /// mesh/mesh FUN_001a8b50(w, s0, s1, 2, buf, end - 12, head) (seen as its 0x1a80d8 / 0x1a81f8 calls); then
    /// FUN_001aa0b0(w, s5, fp, s1, +0x6c, +0x74, +0x74, head).
    #[test]
    fn mesh_1aa1d8_plain_pair_routes_mesh_mesh() {
        fn route(m: &mut FlatMem, addr: u32, a: &[u32], _v: &[V4]) -> Option<(u32, V4)> {
            if addr == 0x0011_8a88 {
                let buf = m.r32(a[3]);
                m.w32(a[3], buf + 24);
                return Some((2, [0.0; 4]));
            }
            None
        }
        let mut m = FlatMem { route: Some(route), ..Default::default() };
        let (b1, b2, s0, s1) = (0x1000, 0x2000, 0x3000, 0x4000);
        m.wf(b1 + 0x30, 10.0);
        m.wf(b2 + 0x30, 20.0);
        m.wv(b1 + 0x20, [1., 2., 3., 1.]);
        m.w32(b1 + 0x34, s0);
        m.w32(b2 + 0x34, s1);
        m.w32(s1 + 0x6c, 4);
        m.w32(s1 + 0x74, 0x5000);
        let r = ee_sub_1aa1d8(&mut m, 0x100, b1, b2);
        assert_eq!(r, 0);
        let c = |m: &FlatMem, x: u32| m.calls.iter().find(|c| c.0 == x).cloned().unwrap();
        let q = c(&m, 0x0011_8a88);
        assert_eq!(&q.1[..2], &[0x5000, 0x7fff_ffff]);
        assert_eq!(q.2, vec![[1., 2., 3., 1.], [10., 0., 0., 0.]]);
        let f = c(&m, 0x001a_81f8);
        let buf = f.1[4];
        assert_eq!(&f.1[..5], &[0x100, s0, s1, 2, buf]);
        assert_eq!(&c(&m, 0x001a_80d8).1[..4], &[0x100, s1, s0, buf + 12]);
        let t = c(&m, 0x001a_a0b0);
        assert_eq!(&t.1[..7], &[0x100, b1, b2, s1, 4, 0x5000, 0x5000]);
    }

    /// unit cube |x|,|y|,|z| <= 1 as 6 planes (n, d); segment along x from -3 to 3: enter t=1/3, exit t=2/3
    #[test]
    fn clip_1bc688_through_cube() {
        let mut m = FlatMem::default();
        let (o, pl, out) = (0x1000, 0x2000, 0x3000);
        m.w32(o + 0xe8, 7);
        m.w32(o + 0x1ec, pl);
        let ns: [V4; 6] = [[1., 0., 0., 1.], [-1., 0., 0., 1.], [0., 1., 0., 1.], [0., -1., 0., 1.], [0., 0., 1., 1.], [0., 0., -1., 1.]];
        for (i, n) in ns.iter().enumerate() {
            m.wv(pl + 16 * i as u32, *n);
        }
        let n = ee_sub_1bc688(&mut m, o, [-3., 0., 0., 1.], [3., 0., 0., 1.], 2, out);
        assert_eq!(n, 2);
        assert!((m.rf(out + 0x44) - 1.0 / 3.0).abs() < 1e-6);
        assert!((m.rf(out + 0x10) + 1.0).abs() < 1e-5);
        assert_eq!(m.rv(out + 0x30), [1., 0., 0., -1.]);
        assert!((m.rf(out + 0x60 + 0x40) - 2.0 / 3.0).abs() < 1e-6);
        assert_eq!(m.r32(out + 0x60 + 0x4c), o);
        // segment entirely outside (+x side): disjoint record
        let n = ee_sub_1bc688(&mut m, o, [2., 0., 0., 1.], [3., 0., 0., 1.], 2, out);
        assert_eq!((n, m.rf(out + 0x44), m.rf(out + 0x40)), (1, 1.0, 0.0));
    }

    /// mode 1: g = (0,0,-10) (FUN_0022fcc8 canned), v = (0,0,10), p = 0, q = (5,0,0) and root t = 2
    #[test]
    fn launch_1b7700_hits_target() {
        let mut m = FlatMem::default();
        let (o, b, s, e) = (0x1000, 0x2000, 0x3000, 0x4000);
        m.w32(o + 0x144c, 1);
        m.w32(o + 0x13f0, 0x5000);
        m.w32(0x5000 + 0x180, b);
        m.w32(b + 0x58, s);
        m.w32(b + 0xe8, e);
        m.w32(o + 0x1030, 0x6000);
        m.wv(s + 0xb0, [0., 0., 10., 0.]);
        m.wf(0x7004, 100.0);
        m.ret.insert(0x0022_fcc8, (0, [0., 0., -10., 0.]));
        m.route = Some(|m, a, args, _| {
            (a == 0x0011_d508).then(|| {
                m.wf(args[0], 2.0);
                (1, [0.0; 4])
            })
        });
        ee_sub_1b7700(&mut m, o, [5., 0., 0., 0.], 0x7000, 0x7004, 0x8000, 0x8010);
        assert_eq!(m.rf(0x7004), 2.0);
        // p1 = v t + g t^2 / 2 = 0 -> d = (2.5, 0, 0)
        assert_eq!(m.rv(0x8000), [2.5, 0., 10., 0.]);
        assert_eq!(m.rv(0x8010), [2.5, 0., -10., 0.]);
    }
}

const SLY: u32 = 0x002d_d5bc;

/// FUN_002206c8(p22, mode): switch the active character mode (-1 = none, 0..7). Leaves -1 (tear down the
/// previous body: FUN_001cd098 unless it is Sly, vtable +0x264(0) when FUN_00132b10(body, 100)) or 3
/// (Sly vtable +0x27c(1) unless entering 4); +4 = mode, +8 = DAT_002dded0, FUN_00134320 (0x3dd490).
/// Mode 7 -> FUN_002046b0(self, +0xe0 - pos); -1 -> FUN_00204660(FUN_00204740()); else +0x1e0 = 0x2a2dd0.
/// Per mode the anim (+0x58.. ) goes to FUN_001305b8; -1 restores the controlled body (Sly: push away
/// along the ground plane: dir*400 + up*600, FUN_00136530(1), vtable +0x11c).
pub fn ee_sub_2206c8<M: QMem + ?Sized>(m: &mut M, p: u32, mode: u32) {
    use crate::engine::classes::q::q6::vcall;
    let prev = m.r32(p + 4);
    if mode == prev {
        return;
    }
    let sly = m.r32(SLY);
    if prev == u32::MAX {
        let b = m.r32(p + 0x10);
        if b != sly {
            let o = m.r32(p);
            m.call(0x001c_d098, &[o + 0x900], &[]);
        }
        let b = m.r32(p + 0x10);
        if m.call(0x0013_2b10, &[b, 100], &[]).0 != 0 {
            vcall(m, b, 0xc, 0x264, &[0], &[]);
        }
        let o = m.r32(p);
        for f in [0x194, 0x198, 0x190] {
            m.w32(o + f, 0);
        }
    } else if prev == 3 && mode != 4 {
        vcall(m, sly, 0xc, 0x27c, &[1], &[]);
    }
    m.w32(p + 4, mode);
    let t = m.r32(0x002d_ded0);
    m.w32(p + 8, t);
    let o = m.r32(p);
    m.call(0x0013_4320, &[0x003d_d490, o], &[]);
    if mode == u32::MAX {
        let u = m.call(0x0020_4740, &[o], &[]).1[0];
        m.call(0x0020_4660, &[o], &[super::q5util::f4(u)]);
        m.w32(o + 0x1a4, 0);
    } else if mode == 7 {
        let pos = m.call(0x001a_62e8, &[o], &[]).1;
        let v = sub(m.rv(p + 0xe0), pos);
        m.call(0x0020_46b0, &[o], &[v]);
        m.w32(o + 0x1a4, 0);
    } else {
        let v = m.rv(0x002a_2dd0);
        m.wv(o + 0x1e0, v);
        m.w32(o + 0x1a4, 1);
    }
    let (mut anim, mut kind) = (0u32, 1u32);
    match mode as i32 {
        0 => {
            if prev.wrapping_sub(1) >= 2 {
                m.wf(p + 0x40, 0.5);
                m.wf(p + 0x4c, 0.5);
                let a = m.r32(p + 0x58);
                m.call(0x0013_84c0, &[a], &[]);
            }
            anim = m.r32(p + 0x58);
            kind = 9;
        }
        1 => anim = m.r32(p + 0x5c),
        2 => anim = m.r32(p + 0x60),
        3 => {
            anim = m.r32(p + 0x64);
            vcall(m, o, 0xc, 0x27c, &[1], &[]);
        }
        4 => {
            let w = m.r32(p + 0x12c);
            if w != 0 {
                m.w32(w + 0x154, 1);
                m.w32(w + 0x160, 0);
                vcall(m, w, 0xc, 0x44, &[], &[]);
            }
            if prev.wrapping_sub(5) >= 2 {
                m.w32(p + 0x130, 0);
                let off = m.rv(p + 0xc0);
                let r = m.call(0x001a_6f00, &[o, 0], &[off]).1;
                m.wv(p + 0xe0, r);
                m.call(0x0021_fb70, &[p, 0], &[]);
            }
            anim = m.r32(p + 0x98);
            kind = 9;
            m.w32(p + 0x128, 0);
        }
        5 => anim = m.r32(p + 0x9c),
        6 => anim = m.r32(p + 0xa0),
        7 => {
            anim = m.r32(p + 0xbc);
            let b = m.r32(p + 0x10);
            if m.call(0x0013_2b10, &[b, 100], &[]).0 != 0 {
                let id = m.r32(b + 0x15c);
                m.call(0x0014_1cd0, &[o, id], &[]);
            }
        }
        -1 => {
            let b = m.r32(p + 0x10);
            if b == sly && m.r32(b + 0x1800) == 0 {
                let a = m.call(0x001a_62e8, &[sly], &[]).1;
                let c = m.call(0x001a_62e8, &[o], &[]).1;
                let up = m.rv(UP);
                let mut d = sub(a, c);
                d = sub(d, scale(up, dot3(d, up)));
                let len = dot3(d, d).sqrt();
                let dir = if len < 1.0e-4 { m.rv(0x002a_2da0) } else { scale(d, 1.0 / len) };
                let sp = stack_push(m, 0x40);
                m.wv(sp + 0x30, add(scale(dir, 400.0), scale(up, 600.0)));
                m.call(0x0013_6530, &[sly, 1], &[]);
                stack_pop(m, 0x40);
                vcall(m, sly, 0xc, 0x11c, &[], &[]);
            } else if { let c = m.r32(p + 0xc); m.r32(b + 0x10) == m.r32(c + 0x10) } {
                if m.call(0x0013_2b10, &[b, 100], &[]).0 == 0 {
                    m.call(0x001a_58d0, &[b, 0, 0xf], &[]);
                    m.call(0x001f_7bf8, &[b], &[f4(0.25)]);
                } else {
                    vcall(m, b, 0xc, 0x264, &[4], &[]);
                }
            }
            m.call(0x001c_d2c0, &[o + 0x900], &[]);
            let w = m.r32(p + 0x12c);
            if w != 0 {
                vcall(m, w, 0xc, 0x4c, &[], &[]);
            }
            m.w32(p + 0x10, 0);
            return;
        }
        _ => {}
    }
    m.call(0x001f_15b8, &[o + 0x1034], &[]);
    m.call(0x0013_05b8, &[anim, o, kind, o + 0x1030], &[f4(0.0), f4(1.0)]); // PART r16 ABI: $f12 = 0.0, $f13 = 1.0
    vcall(m, o, 0xc, 0x2d4, &[], &[]);
    m.call(0x0013_4320, &[0x003d_d490, o], &[]);
}

/// v.xyz = r0*p.x + r1*p.y + r2*p.z + t (VMULAx/VMADDAy/VMADDAz/VMADDw vf00w); w kept from `w`
fn xform(m: [V4; 4], p: V4, w: f32) -> V4 {
    let mut o = [0.0; 4];
    for i in 0..3 {
        o[i] = m[0][i] * p[0] + m[1][i] * p[1] + m[2][i] * p[2] + m[3][i];
    }
    o[3] = w;
    o
}
fn rmat<M: QMem + ?Sized>(m: &mut M, a: u32) -> [V4; 4] {
    [m.rv(a), m.rv(a + 0x10), m.rv(a + 0x20), m.rv(a + 0x30)]
}

/// FUN_001a9d38(a0, b1, b2, stk) (IntersectBodies: b1's vertex list vs b2's BSP, from the disassembly).
/// For i in b1[+0x1c]..b1[+0x78]: p = b1 matrix (+0x20) * verts[i] (+0x7c); when b2[+0xa8] != b2[+0xac]:
/// p = b2[0] matrix (+0x40) * (FUN_0011c318(b2 + 0x20) inverse * p). n = FUN_0014c6a0(b2, p, nrm[i] (+0x94), 2,
/// rec); skip unless n > (rec+0x40 == 0.0). dir = (p - nrm[i]) / |..|; depth = min(0, (verts' - rec.pt) . dir over
/// b1[+0x90] points at +0x94, plus (b1[0]+0x70 - rec.pt) . dir - b1[+8] when b1[+0x10] bit 49);
/// FUN_001a7c18(depth, a0, b1, b2, rec.pt, dir, b2 bit49 ? 9 : 8, 0, bit49 ? 0 : rec[+0x4c], stk).
pub fn ee_sub_1a9d38<M: QMem + ?Sized>(m: &mut M, a0: u32, b1: u32, b2: u32, stk: u32) {
    const B49: u64 = 1 << 49;
    let sp = stack_push(m, 0x100);
    let (rec, inv) = (sp, sp + 0x80);
    let mut p: V4 = [0.0; 4];
    let mut i = m.r32(b1 + 0x1c);
    while (i as i32) < m.r32(b1 + 0x78) as i32 {
        let vtx = m.r32(b1 + 0x7c) + i * 16;
        let v = m.rv(vtx);
        let mb = rmat(m, b1 + 0x20);
        p = xform(mb, v, p[3]);
        if m.r32(b2 + 0xa8) != m.r32(b2 + 0xac) {
            m.call(0x0011_c318, &[inv, b2 + 0x20], &[]);
            let mi = rmat(m, inv);
            p = xform(mi, p, p[3]);
            let o = m.r32(b2);
            let mo = rmat(m, o + 0x40);
            p = xform(mo, p, p[3]);
        }
        let nrm = m.r32(b1 + 0x94) + i * 16;
        let q = m.rv(nrm);
        i += 1;
        let n = m.call(0x0014_c6a0, &[b2, 2, rec], &[p, q]).0 as i32;
        if n == 0 {
            continue;
        }
        let z = (m.rf(rec + 0x40) == 0.0) as i32;
        if z >= n {
            continue;
        }
        let d = sub(p, q);
        let dir = scale(d, inv_len(d));
        let pt = m.rv(rec);
        let f0 = (m.r32(b1 + 0x10) as u64) | (m.r32(b1 + 0x14) as u64) << 32;
        let f2 = (m.r32(b2 + 0x10) as u64) | (m.r32(b2 + 0x14) as u64) << 32;
        let k5c = m.r32(rec + 0x4c);
        let mut depth = 0.0f32;
        let (cnt, list) = (m.r32(b1 + 0x90) as i32, m.r32(b1 + 0x94));
        for j in 0..cnt.max(0) as u32 {
            let x = dot3(sub(m.rv(list + 16 * j), pt), dir);
            if x < depth {
                depth = x;
            }
        }
        if f0 & B49 != 0 {
            let o = m.r32(b1);
            let x = dot3(sub(m.rv(o + 0x70), pt), dir) - m.rf(b1 + 8);
            if x < depth {
                depth = x;
            }
        }
        let hit = f2 & B49 != 0;
        let (kind, t2) = if hit { (9, 0) } else { (8, k5c) };
        m.call(0x001a_7c18, &[a0, b1, b2, kind, t2, 0, stk], &[f4(depth), pt, dir]);
    }
    stack_pop(m, 0x100);
}

/// plane.xyz . p - plane.w (VMUL/VADDAy/VMADDAz/VMSUBw with vf02.x = 1)
fn pdist(pl: V4, p: V4) -> f32 {
    dot3(pl, p) - pl[3]
}

/// FUN_001a87f0(a0, b, a2, n, verts, chains, stk) (IntersectBodies mesh: per contact vertex, pick the separating
/// face; from the disassembly). For k < n: P = b[+0x94][verts[k]]; over the face chain chains[k] (link +0x5c, face
/// +0x50, plane at face[0], visited bit 8 of face[+0x18]): d0 = dist(plane, P), dmin = min(d0, dist(plane, vtx
/// u16 (+0x54)[1 - (+0x58)] of each chain node)); keep the face with the largest dmin (> DAT_002a6ad0). Clear the
/// visited bits; FUN_001a7c18(d0*, a0, b, a2, P, plane*, 1, &b[+0x94][verts[k]], face*, stk); then for each chain
/// node with d = dist(plane*, node+0x10) < d0*: FUN_001a7c18(d, a0, b, a2, node+0x10, plane*, 2, node[+0x54], face*, stk).
/// FUN_002564b8 / FUN_002564c0 temp ctor/dtor calls kept.
#[allow(clippy::too_many_arguments)]
pub fn ee_sub_1a87f0<M: QMem + ?Sized>(m: &mut M, a0: u32, b: u32, a2: u32, n: u32, verts: u32, chains: u32, stk: u32) {
    let sp = stack_push(m, 0x90);
    for k in 0..(n as i32).max(0) as u32 {
        let pts = m.r32(b + 0x94);
        let vi = m.r32(verts + 4 * k);
        let s3 = m.r32(chains + 4 * k);
        let mut best_min = m.rf(0x002a_6ad0);
        let p = m.rv(pts + vi * 16);
        let (mut face, mut best, mut d0b) = (0u32, [0.0f32; 4], 0.0f32);
        let mut s0 = s3;
        while s0 != 0 {
            let s2 = m.r32(s0 + 0x50);
            let fl = m.r32(s2 + 0x18);
            if fl & 8 == 0 {
                m.w32(s2 + 0x18, fl | 8);
                let pl = m.rv(s2);
                m.call(0x0025_64b8, &[sp + 0x10], &[]);
                m.call(0x0025_64b8, &[sp + 0x20], &[]);
                let d0 = pdist(pl, p);
                let mut dmin = d0;
                let mut s1 = s3;
                while s1 != 0 {
                    let sel = 1u32.wrapping_sub(m.r32(s1 + 0x58));
                    let e = m.r32(s1 + 0x54).wrapping_add(sel.wrapping_mul(2));
                    let ix = m.r16(e) as u32;
                    let q = m.rv(pts + ix * 16);
                    m.call(0x0025_64b8, &[sp + 0x30], &[]);
                    m.call(0x0025_64b8, &[sp + 0x40], &[]);
                    let d = pdist(pl, q);
                    if d < dmin {
                        dmin = d;
                    }
                    s1 = m.r32(s1 + 0x5c);
                }
                if best_min < dmin {
                    best = pl;
                    face = s2;
                    best_min = dmin;
                    d0b = d0;
                }
            }
            s0 = m.r32(s0 + 0x5c);
        }
        let mut s0 = s3;
        while s0 != 0 {
            let f = m.r32(s0 + 0x50);
            let v = m.r32(f + 0x18);
            m.w32(f + 0x18, v & !8);
            s0 = m.r32(s0 + 0x5c);
        }
        m.call(0x0025_64c0, &[sp + 0x50], &[]);
        let vp = m.r32(b + 0x94) + vi * 16;
        m.call(0x001a_7c18, &[a0, b, a2, 1, vp, face, stk], &[f4(d0b), p, best]);
        let mut s1 = s3;
        while s1 != 0 {
            let q = m.rv(s1 + 0x10);
            m.call(0x0025_64b8, &[sp + 0x60], &[]);
            m.call(0x0025_64b8, &[sp + 0x70], &[]);
            let d = pdist(best, q);
            if d < d0b {
                m.call(0x0025_64c0, &[sp + 0x80], &[]);
                let e = m.r32(s1 + 0x54);
                m.call(0x001a_7c18, &[a0, b, a2, 2, e, face, stk], &[f4(d), q, best]);
            }
            s1 = m.r32(s1 + 0x5c);
        }
    }
    stack_pop(m, 0x90);
}


/// FUN_001a8b50(a0, A, B, a3, t0, t1, list) (IntersectBodies mesh vs mesh; from the disassembly; stack 0x6ac0).
/// n0 = FUN_001a80d8(a0, B, A, t1, 0x80, recs0); n1 = FUN_001a81f8(a0, A, B, a3, t0, 0x80, recs1) (0x60-byte
/// records); both 0 -> return. FUN_001a8410(n_i, recs_i, &cnt_i, verts_i, chains_i, &edges_i) for i = 0, 1;
/// FUN_001a87f0 for (A, B) and (B, A). For each edge e of edges0 (link +0x5c) with faces (+0x4c, +0x50):
/// k = FUN_001a86b8(B[+0x9c], idx(+0x4c), idx(+0x50), edges1, out) (32-byte face index); k > 0 -> per matched
/// edge f: (s, t) = FUN_0011ea60(A seg e, B seg f, &st, &n) closest points; FUN_001a7c18(-|pa - pb|, a0, A, B, pa,
/// n, 3, e[+0x54], f[+0x54], list). k <= 0 -> unless a recs1 record (+0x4c/+0x50) names either face of e: pick
/// the face with the smaller plane distance to A centre ([A]+0x70), vertex = A pts[e verts[1 - sel or sel]];
/// skip when list already holds {+0x9c = 1, +0xa0 = vtx, +0xa4 = face} (chain +0xac); else FUN_001a7c18(plane .
/// vtx - w, a0, A, B, vtx, face plane, 1, vtx, face, list). Finally, if no contact was added and n0 == 0 or
/// n1 == 0: FUN_001a7c18(0, a0, A, B, [A]+0x70, DAT_002a2dc0, 0xd, 0, 0, list).
#[allow(clippy::too_many_arguments)]
pub fn ee_sub_1a8b50<M: QMem + ?Sized>(m: &mut M, a0: u32, ba: u32, bb: u32, a3: u32, t0: u32, t1: u32, list: u32) {
    let sp = stack_push(m, 0x6ac0);
    let head0 = m.r32(list);
    let n0 = m.call(0x001a_80d8, &[a0, bb, ba, t1, 0x80, sp + 0x20], &[]).0;
    m.w32(sp + 0x10, n0);
    let n1 = m.call(0x001a_81f8, &[a0, ba, bb, a3, t0, 0x80, sp + 0x3020], &[]).0;
    m.w32(sp + 0x14, n1);
    if n0 == 0 && n1 == 0 {
        stack_pop(m, 0x6ac0);
        return;
    }
    for i in 0..2u32 {
        let n = m.r32(sp + 0x10 + 4 * i);
        m.call(0x001a_8410, &[n, sp + 0x20 + 0x3000 * i, sp + 0x6020 + 4 * i, sp + 0x6030 + 0x200 * i, sp + 0x6430 + 0x200 * i, sp + 0x6830 + 4 * i], &[]);
    }
    let (c0, c1) = (m.r32(sp + 0x6020), m.r32(sp + 0x6024));
    m.call(0x001a_87f0, &[a0, ba, bb, c0, sp + 0x6030, sp + 0x6430, list], &[]);
    m.call(0x001a_87f0, &[a0, bb, ba, c1, sp + 0x6230, sp + 0x6630, list], &[]);
    // edges of the A-side chain (sp+0x6830, link +0x5c)
    let mut e = m.r32(sp + 0x6830);
    while e != 0 {
        let fa = m.r32(e + 0x4c);
        let fb = m.r32(e + 0x50);
        if fa == 0 || fb == 0 {
            e = m.r32(e + 0x5c);
            continue;
        }
        let base = m.r32(bb + 0x9c);
        let segs1 = m.r32(sp + 0x6834);
        let ia = (fa.wrapping_sub(base) as i32 >> 5) as u32;
        let ib = (fb.wrapping_sub(base) as i32 >> 5) as u32;
        let k = m.call(0x001a_86b8, &[base, ia, ib, segs1, sp + 0x6840], &[]).0 as i32;
        if k > 0 {
            // edge / edge: closest points of A seg e and B seg f (FUN_0011ea60), depth -|pa - pb|, kind 3
            for j in 0..k as u32 {
                let f = m.r32(sp + 0x6840 + 4 * j);
                let (ee, fe) = (m.r32(e + 0x54), m.r32(f + 0x54));
                let (pts_a, pts_b) = (m.r32(ba + 0x94), m.r32(bb + 0x94));
                let e0 = vat(m, pts_a, ee);
                let e1 = vat(m, pts_a, ee + 2);
                let f0 = vat(m, pts_b, fe);
                let f1 = vat(m, pts_b, fe + 2);
                m.call(0x0011_ea60, &[sp + 0x68c0, sp + 0x68d0], &[e0, e1, f0, f1]);
                let (s, t) = (m.rf(sp + 0x68c0), m.rf(sp + 0x68c4));
                let pa = add(scale(e0, 1.0 - s), scale(e1, s));
                let pb = add(scale(f0, 1.0 - t), scale(f1, t));
                let dv = sub(pa, pb);
                let d = crate::engine::math::vu0::vsqrt(dot3(dv, dv));
                let n = m.rv(sp + 0x68d0);
                m.call(0x001a_7c18, &[a0, ba, bb, 3, ee, fe, list], &[f4(-d), pa, n]);
            }
            e = m.r32(e + 0x5c);
            continue;
        }
        // no B path: skip when a recs1 record names either A face of e
        let n1 = m.r32(sp + 0x14) as i32;
        let ee = m.r32(e + 0x54);
        let fbase = m.r32(ba + 0x9c);
        let ga = fbase.wrapping_add((m.r16(ee + 4) as u32) << 5);
        let gb = fbase.wrapping_add((m.r16(ee + 6) as u32) << 5);
        let mut skip = false;
        for r in 0..n1.max(0) as u32 {
            let rec = sp + 0x3020 + 0x60 * r;
            let (x, y) = (m.r32(rec + 0x4c), m.r32(rec + 0x50));
            if x == ga || x == gb || y == ga || y == gb {
                skip = true;
                break;
            }
        }
        if !skip {
            // face with the smaller plane distance to A's centre ([A]+0x70) and its vertex
            let o = m.r32(ba);
            let p1 = m.rv(e + 0x20);
            let c = m.rv(o + 0x70);
            m.call(0x0025_64b8, &[sp + 0x6910], &[]);
            m.call(0x0025_64b8, &[sp + 0x6920], &[]);
            let d1 = pdist(p1, c);
            let o = m.r32(ba);
            let c = m.rv(o + 0x70);
            let p2 = m.rv(e + 0x30);
            m.call(0x0025_64b8, &[sp + 0x6930], &[]);
            m.call(0x0025_64b8, &[sp + 0x6940], &[]);
            let d2 = pdist(p2, c);
            let (face, sel) = if d2 < d1 { (m.r32(e + 0x4c), 1u32.wrapping_sub(m.r32(e + 0x58))) } else { (m.r32(e + 0x50), m.r32(e + 0x58)) };
            let ia = m.r32(e + 0x54).wrapping_add(sel.wrapping_mul(2));
            let vi = m.r16(ia) as u32;
            let vp = m.r32(ba + 0x94).wrapping_add(vi << 4);
            let mut n = m.r32(list);
            while n != 0 {
                if m.r32(n + 0x9c) == 1 && m.r32(n + 0xa0) == vp && m.r32(n + 0xa4) == face {
                    break;
                }
                n = m.r32(n + 0xac);
            }
            if n == 0 {
                let v = m.rv(vp);
                let pl = m.rv(face);
                m.call(0x0025_64b8, &[sp + 0x6950], &[]);
                m.call(0x0025_64b8, &[sp + 0x6960], &[]);
                let d = pdist(pl, v);
                m.call(0x001a_7c18, &[a0, ba, bb, 1, vp, face, list], &[f4(d), v, pl]);
            }
        }
        e = m.r32(e + 0x5c);
    }
    if m.r32(list) == head0 && (m.r32(sp + 0x10) == 0 || m.r32(sp + 0x14) == 0) {
        let o = m.r32(ba);
        let c = m.rv(o + 0x70);
        let dir = m.rv(0x002a_2dc0);
        m.call(0x001a_7c18, &[a0, ba, bb, 0xd, 0, 0, list], &[f4(0.0), c, dir]);
    }
    stack_pop(m, 0x6ac0);
}

fn vat<M: QMem + ?Sized>(m: &mut M, pts: u32, ia: u32) -> V4 {
    let i = m.r16(ia) as u32;
    m.rv(pts + i * 16)
}

fn ld64<M: QMem + ?Sized>(m: &mut M, a: u32) -> u64 {
    (m.r32(a) as u64) | (m.r32(a + 4) as u64) << 32
}

/// FUN_001aa1d8(world, b1, b2) IntersectBodies (from the disassembly): returns the contact list head built for
/// the two bodies' SO chains (+0x34, link +0x60). The body with the larger +0x30 radius becomes `fp`. For each so1 of
/// fp: bit 48 (+0x10) and no geom (+0x18) -> skip; BSP candidates (FUN_00118a88 against the other body's sphere
/// +0x20/+0x30 into a frame buffer of min(max +0x6c, 0x1555) 12-byte entries) unless bit 49 or other+0x70 & 0x20;
/// for each so0 of the other body: geom filter FUN_001f7530; vertex/BSP (FUN_001a9d38 both ways when either
/// +0xa8 != +0xac), then sphere/sphere (0x1a9c40), sphere/BSP (0x1a9b28), mesh/mesh (0x1a8b50); contacts of
/// filtered pairs are moved by FUN_001a9fe0 / FUN_001aa020. Every non-bit-48 SO also runs FUN_001aa0b0.
pub fn ee_sub_1aa1d8<M: QMem + ?Sized>(m: &mut M, w: u32, a1: u32, a2: u32) -> u32 {
    use crate::engine::classes::q::hard1::{ee_sub_1a9b28, ee_sub_1a9c40};
    use crate::engine::classes::q::q8util::{frame_alloc, frame_begin, frame_end};
    const B48: u64 = 1 << 48;
    const B49: u64 = 1 << 49;
    let (mut s5, mut fp) = (a1, a2);
    if m.rf(a2 + 0x30) < m.rf(a1 + 0x30) {
        (s5, fp) = (a2, a1);
    }
    let sp = stack_push(m, 0x40);
    let (head, aux) = (sp + 0x28, sp + 0x2c); // +0x20 n, +0x24 end
    m.w32(head, 0);
    let mut mx = 0i32;
    let mut s = m.r32(fp + 0x34);
    while s != 0 {
        mx = mx.max(m.r32(s + 0x6c) as i32);
        s = m.r32(s + 0x60);
    }
    let fr = frame_begin(m);
    let cnt = if mx > 0x1554 { 0x1555 } else { mx };
    let buf = frame_alloc(m, (cnt * 12) as u32, 0x10);
    let mut s1 = m.r32(fp + 0x34);
    while s1 != 0 {
        let has1 = m.r32(s1 + 0x18) != 0;
        let f1 = ld64(m, s1 + 0x10);
        if f1 & B48 != 0 && !has1 {
            s1 = m.r32(s1 + 0x60);
            continue;
        }
        let (mut sp24, mut sp3c, mut sp40, mut sp44) = (0u32, 0u32, 0u32, 2u32);
        if f1 & B49 == 0 && ld64(m, s5 + 0x70) & 0x20 == 0 {
            m.w32(sp + 0x20, cnt as u32);
            m.w32(sp + 0x24, buf);
            sp3c = buf;
            let bsp = m.r32(s1 + 0x74);
            let (c, r) = (m.rv(s5 + 0x20), m.rf(s5 + 0x30));
            sp44 = m.call(0x0011_8a88, &[bsp, 0x7fff_ffff, sp + 0x20, sp + 0x24], &[c, f4(r)]).0;
            if sp44 == 1 {
                s1 = m.r32(s1 + 0x60);
                continue;
            }
            if sp44 == 0 {
                sp24 = 0x002b_bb20;
                sp3c = 0x002b_bb20;
            } else {
                let e = m.r32(sp + 0x24);
                sp40 = (e.wrapping_sub(sp3c) as i32 / 12) as u32;
                sp24 = e.wrapping_sub(12);
            }
        }
        let mut s0 = m.r32(s5 + 0x34);
        while s0 != 0 {
            let s6 = m.r32(s0 + 0x18) != 0;
            let f0 = ld64(m, s0 + 0x10);
            let f1 = ld64(m, s1 + 0x10);
            let next = |m: &mut M, s0: u32| m.r32(s0 + 0x60);
            if (f0 & B48 != 0 && !s6) || (f0 & B48 != 0 && f1 & B48 != 0) {
                s0 = next(m, s0);
                continue;
            }
            m.w32(aux, 0);
            let mut s2 = head;
            if has1 {
                let (x, y) = (m.r32(s0 + 4), m.r32(s1 + 0x18));
                let v = m.call(0x001f_7530, &[x, y], &[]).0 != 0;
                if f1 & B48 != 0 {
                    if !v {
                        s0 = next(m, s0);
                        continue;
                    }
                    s2 = aux;
                } else if v {
                    s2 = if f0 & B48 != 0 { head } else { aux };
                }
            }
            if s6 {
                let (x, y) = (m.r32(s1 + 4), m.r32(s0 + 0x18));
                let v = m.call(0x001f_7530, &[x, y], &[]).0 != 0;
                if f0 & B48 != 0 {
                    if !v {
                        s0 = next(m, s0);
                        continue;
                    }
                    s2 = aux;
                } else if v && f1 & B48 == 0 {
                    s2 = aux;
                }
            }
            let s4 = m.r32(s2);
            if m.r32(s0 + 0xac) != m.r32(s0 + 0xa8) || m.r32(s1 + 0xac) != m.r32(s1 + 0xa8) {
                if (m.r32(s0 + 0x1c) as i32) < m.r32(s0 + 0x78) as i32 {
                    ee_sub_1a9d38(m, w, s0, s1, s2);
                }
                if (m.r32(s1 + 0x1c) as i32) < m.r32(s1 + 0x78) as i32 {
                    ee_sub_1a9d38(m, w, s1, s0, s2);
                }
            }
            if m.r32(s2) == s4 {
                if f1 & B49 != 0 {
                    if f0 & B49 != 0 {
                        ee_sub_1a9c40(m, w, s0, s1, s2);
                    } else {
                        ee_sub_1a9b28(m, w, s1, s0, 0, 0, s2);
                    }
                } else if f0 & B49 != 0 {
                    let far = sp44 == 0 || {
                        let o = m.r32(s1);
                        let d = sub(m.rv(o + 0x70), m.rv(s5 + 0x20));
                        1.0 < dot3(d, d) || 1.0 < (m.rf(s1 + 8) - m.rf(s5 + 0x30)).abs()
                    };
                    let (x, y) = if far { (0, 0) } else { (sp40, sp3c) };
                    ee_sub_1a9b28(m, w, s0, s1, x, y, s2);
                } else {
                    ee_sub_1a8b50(m, w, s0, s1, sp40, sp3c, sp24, s2);
                }
            }
            let l = m.r32(aux);
            if l != 0 {
                let mut mv = false;
                if has1 {
                    if f1 & B48 != 0 {
                        m.call(0x001a_9fe0, &[w, l, fp + 0x6c], &[]);
                    } else if f0 & B48 == 0 {
                        mv = true;
                        let r = m.call(0x001a_a020, &[w, l], &[]).0;
                        m.call(0x001a_9fe0, &[w, r, fp + 0x6c], &[]);
                    }
                }
                if s6 {
                    let l = m.r32(aux);
                    if f0 & B48 != 0 {
                        m.call(0x001a_9fe0, &[w, l, s5 + 0x6c], &[]);
                    } else if f1 & B48 == 0 {
                        mv = true;
                        let r = m.call(0x001a_a020, &[w, l], &[]).0;
                        m.call(0x001a_9fe0, &[w, r, s5 + 0x6c], &[]);
                    }
                }
                if mv {
                    let l = m.r32(aux);
                    m.call(0x001a_9fe0, &[w, l, head], &[]);
                }
            }
            s0 = next(m, s0);
        }
        if ld64(m, s1 + 0x10) & B48 == 0 {
            let (t0, t1) = (m.r32(s1 + 0x6c), m.r32(s1 + 0x74));
            m.call(0x001a_a0b0, &[w, s5, fp, s1, t0, t1, t1, head], &[]);
        }
        s1 = m.r32(s1 + 0x60);
    }
    let mut s0 = m.r32(s5 + 0x34);
    while s0 != 0 {
        if ld64(m, s0 + 0x10) & B48 == 0 {
            let (t0, t1) = (m.r32(s0 + 0x6c), m.r32(s0 + 0x74));
            m.call(0x001a_a0b0, &[w, fp, s5, s0, t0, t1, t1, head], &[]);
        }
        s0 = m.r32(s0 + 0x60);
    }
    frame_end(m, fr, 2);
    let r = m.r32(head);
    stack_pop(m, 0x40);
    r
}

/// rows of `q` times `r` (VMULAx/VMADDAy/VMADDz.xyz); w lanes as left by the EE code (q0.w, q1.w, r0.w)
fn mulrows(q: [V4; 3], r: [V4; 3]) -> [V4; 3] {
    let row = |v: V4, w: f32| {
        let mut o = [0.0; 4];
        for i in 0..3 {
            o[i] = r[0][i] * v[0] + r[1][i] * v[1] + r[2][i] * v[2];
        }
        o[3] = w;
        o
    };
    [row(q[0], q[0][3]), row(q[1], q[1][3]), row(q[2], r[0][3])]
}
fn r3<M: QMem + ?Sized>(m: &mut M, a: u32) -> [V4; 3] {
    [m.rv(a), m.rv(a + 0x10), m.rv(a + 0x20)]
}
fn w3<M: QMem + ?Sized>(m: &mut M, a: u32, v: [V4; 3]) {
    for (i, r) in v.into_iter().enumerate() {
        m.wv(a + 0x10 * i as u32, r);
    }
}

/// FUN_00117308(q0, w0, q1, w1, out_rot, out_vel; f12 = T, f13 = t, f14 = k) hermite rotation blend (from the
/// disassembly; f12/f13/f14 are the float args). u = t / T. out_rot (when non-null): M1 = q0 * R(w0 * u(1-u)^2 T),
/// M2 = q1 * R(w1 * -u^2(1-u) T) (FUN_0011b570 rotation from vector); a = FUN_0011bfb8(FUN_0011bc00(M1, M2));
/// out_rot = M1 * R(a * (u^2(1-u)(3 - T) + u^3)). out_vel (when non-null): a = FUN_0011bfb8(FUN_0011bc00(q0, q1));
/// out_vel = (a * 6u(1-u)/T + w0 * (1 - u(4-3u)) + w1 * u(3u-2)) * k.
#[allow(clippy::too_many_arguments)]
pub fn ee_sub_117308<M: QMem + ?Sized>(m: &mut M, q0: u32, w0: V4, q1: u32, w1: V4, out: u32, vel: u32, t_: f32, t: f32, k: f32) {
    let sp = stack_push(m, 0x130);
    let u = t / t_;
    let f1 = 1.0 - u;
    if out != 0 {
        let c0 = u * f1 * f1 * t_;
        let f20 = u * u * f1;
        let f21 = u * u * u;
        m.call(0x0011_b570, &[sp + 0x90], &[scale(w0, c0)]);
        let (r, q) = (r3(m, sp + 0x90), r3(m, q0));
        let m1 = mulrows(q, r);
        w3(m, sp, m1);
        m.call(0x0011_b570, &[sp + 0x90], &[scale(w1, -f20 * t_)]);
        let (r, q) = (r3(m, sp + 0x90), r3(m, q1));
        let m2 = mulrows(q, r);
        w3(m, sp + 0x30, m2);
        m.call(0x0011_bc00, &[sp + 0x60, sp, sp + 0x30], &[]);
        let a = m.call(0x0011_bfb8, &[sp + 0x60], &[]).1;
        let s = f20 * (3.0 - t_) + f21;
        m.call(0x0011_b570, &[sp + 0x90], &[scale(a, s)]);
        let r = r3(m, sp + 0x90);
        let o = mulrows(m1, r);
        w3(m, out, o);
    }
    if vel != 0 {
        let kk = u * (6.0 - u * 6.0) / t_;
        m.call(0x0011_bc00, &[sp, q0, q1], &[]);
        let a = m.call(0x0011_bfb8, &[sp], &[]).1;
        let c1 = 1.0 - u * (4.0 - u * 3.0);
        let c2 = u * (u * 3.0 - 2.0);
        let v = add(add(scale(a, kk), scale(w0, c1)), scale(w1, c2));
        m.wv(vel, scale(v, k));
    }
    stack_pop(m, 0x130);
}

fn callf<M: QMem + ?Sized>(m: &mut M, f: u32, a: &[u32], v: &[V4]) -> f32 {
    m.call(f, a, v).1[0]
}

/// FUN_0018b460(jt) (from the disassembly): head/aim look-at blend. u = (+0x1bd8 - anim[+0x1c]) / anim[+0x40]
/// (anim = +0x1030). With a target (+0x10dc, its +0x494 rig) and u > 0: FUN_002046f0(jt); M = FUN_002309e8(rig
/// [+0x150], 0, &M, 0, min(u, DAT_002bf380)); p = M * DAT_002bf390; l = FUN_0022fc48(0, jt[+0x60], p);
/// yaw = FUN_0011d948(&DAT_002bf3a8, atan2(-l.z, DAT_002bf3a0 * l.x)) -> a = (0, yaw, 0, 0); N = euler(+0x1c10 + a)
/// (FUN_0011b628) with translation FUN_00256d18(+0x1c00); d = l - N * (+0x1c20); pitch = FUN_0011d948(
/// &DAT_002bf3b0, FUN_0022fcc8(jt[+0x60], bone, d).x) -> b = (pitch, 0, 0, 0); e = d - FUN_0022fcc8(bone, jt[+0x60],
/// b); when |+0x1c00 + e - +0x1bf0| < DAT_002bf3b8: FUN_0012da30 on +0x19f8 (a), +0x19ec (e), +0x19fc (b),
/// +0x1bd4 = 1, +0x1bdc = DAT_002dded0; return. Otherwise blend out: f = (+0x1bdc + DAT_002bf3bc - dt0 - dt1) /
/// (+0x1bdc + DAT_002bf3bc - dt0) (0 when <= 0); each of +0x19f8/+0x19ec/+0x19fc gets FUN_0012da30(p, p[+0x20] * f);
/// +0x1bd4 = 0. (bone = jt[+0x19a0][+0x58][+0xa0].)
pub fn ee_sub_18b460<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let sp = stack_push(m, 0xe0);
    let an = m.r32(o + 0x1030);
    let u = (m.rf(o + 0x1bd8) - m.rf(an + 0x1c)) / m.rf(an + 0x40);
    let tg = m.r32(o + 0x10dc);
    if tg != 0 && m.r32(tg + 0x494) != 0 && 0.0 < u {
        m.call(0x0020_46f0, &[o], &[]);
        let tg = m.r32(o + 0x10dc);
        let rig = m.r32(tg + 0x494);
        let lim = m.rf(0x002b_f380);
        let r150 = m.r32(rig + 0x150);
        m.call(0x0023_09e8, &[r150, 0, sp, 0], &[f4(u.min(lim))]);
        let mm = rmat(m, sp);
        let p = xform(mm, m.rv(0x002b_f390), 0.0);
        let j60 = m.r32(o + 0x60);
        let l = m.call(0x0022_fc48, &[0, j60], &[p]).1;
        let k = m.rf(0x002b_f3a0);
        let at = callf(m, 0x0024_58f0, &[], &[f4(-l[2]), f4(k * l[0])]);
        let yaw = callf(m, 0x0011_d948, &[0x002b_f3a8], &[f4(at)]);
        let a = [0.0, yaw, 0.0, 0.0];
        let e0 = add(m.rv(o + 0x1c10), a);
        m.call(0x0011_b628, &[sp + 0x90], &[e0]);
        let t = m.rv(o + 0x1c00);
        m.call(0x0025_6d18, &[sp + 0x70], &[t]);
        for i in 0..3 {
            let r = m.rv(sp + 0x90 + 0x10 * i);
            m.wv(sp + 0x40 + 0x10 * i, [r[0], r[1], r[2], 0.0]);
        }
        m.wf(sp + 0x7c, 1.0);
        let bone = {
            let v = m.r32(o + 0x19a0);
            let v = m.r32(v + 0x58);
            m.r32(v + 0xa0)
        };
        let n = rmat(m, sp + 0x40);
        let q = xform(n, m.rv(o + 0x1c20), n[3][3]);
        let d = sub(l, q);
        let j60 = m.r32(o + 0x60);
        let pr = m.call(0x0022_fcc8, &[j60, bone], &[d]).1[0];
        let pitch = callf(m, 0x0011_d948, &[0x002b_f3b0], &[f4(pr)]);
        let b = [pitch, 0.0, 0.0, 0.0];
        let j60 = m.r32(o + 0x60);
        let v = m.call(0x0022_fcc8, &[bone, j60], &[b]).1;
        let e = sub(d, v);
        let x = sub(add(m.rv(o + 0x1c00), e), m.rv(o + 0x1bf0));
        if dot3(x, x).sqrt() < m.rf(0x002b_f3b8) {
            for (off, v) in [(0x19f8, a), (0x19ec, e), (0x19fc, b)] {
                let p = m.r32(o + off);
                m.call(0x0012_da30, &[p], &[v]);
            }
            m.w32(o + 0x1bd4, 1);
            let t = m.rf(0x002d_ded0);
            m.wf(o + 0x1bdc, t);
            stack_pop(m, 0xe0);
            return;
        }
    }
    let f0 = m.rf(o + 0x1bdc) + m.rf(0x002b_f3bc) - m.rf(0x002d_ded0);
    let f2 = f0 - m.rf(0x002d_ded4);
    let f = if 0.0 < f2 { f2 / f0 } else { 0.0 };
    for off in [0x19f8, 0x19ec, 0x19fc] {
        let p = m.r32(o + off);
        let v = scale(m.rv(p + 0x20), f);
        m.call(0x0012_da30, &[p], &[v]);
    }
    m.w32(o + 0x1bd4, 0);
    stack_pop(m, 0xe0);
}

/// FUN_00205910(jt, params, n, pts) ground-align (from the disassembly): M = rotation about DAT_002a2dc0 by
/// jt[+0x1a8] (FUN_0011b500) with translation pos(jt) (FUN_001a62e8 / FUN_00256d18); M0 = M. r = max(|pts[i]|,
/// DAT_002a85f8); in a frame: FUN_0020fc60(0x3dd490, M.t, 0x20fa50, jt[+0xec], &n, &list; f12 = r). Repeat
/// DAT_002c3ce4 times: w_i = M * pts[i]; FUN_00173610(&w_i, 0, 0, params[1], params[0]) == 0 -> w_i += (0,0,0,1) +
/// M.row2 * DAT_002c3ce0. nrm = normalize((w1 - w0) x (w2 - w0)) (DAT_002a2dc0 when |..| < 1e-4); R =
/// FUN_0011ba08(M0.row2 -> nrm); (ang, axis) = FUN_0011bcf8(R); ang > DAT_002c3ce8 -> R = FUN_0011b500(axis,
/// DAT_002c3ce8); M rows = M0 rows * R (w = 0). jt[+0x1b0..+0x1d0] = M rows; frame closed (2).
pub fn ee_sub_205910<M: QMem + ?Sized>(m: &mut M, o: u32, prm: u32, n: u32, pts: u32) {
    use crate::engine::classes::q::q8util::{frame_begin, frame_end};
    let sp = stack_push(m, 0x180);
    let up = m.rv(UP);
    let ang = m.rf(o + 0x1a8);
    m.call(0x0011_b500, &[sp + 0x80], &[up, f4(ang)]);
    let pos = m.call(0x001a_62e8, &[o], &[]).1;
    m.call(0x0025_6d18, &[sp + 0x30], &[pos]);
    for i in 0..3 {
        let r = m.rv(sp + 0x80 + 0x10 * i);
        m.wv(sp + 0x10 * i, [r[0], r[1], r[2], 0.0]);
    }
    m.wf(sp + 0x3c, 1.0);
    for i in 0..4 {
        let r = m.rv(sp + 0x10 * i);
        m.wv(sp + 0x40 + 0x10 * i, r);
    }
    let n = (n as i32).max(0) as u32;
    let mut rad = m.rf(0x002a_85f8);
    for i in 0..n {
        let p = m.rv(pts + 16 * i);
        rad = dot3(p, p).sqrt().max(rad);
    }
    let fr = frame_begin(m);
    let ec = m.r32(o + 0xec);
    let t = m.rv(sp + 0x30);
    m.call(0x0020_fc60, &[0x003d_d490, 0x0020_fa50, ec, sp + 0x170, sp + 0x174], &[t, f4(rad)]);
    let mut it = 0i32;
    while it < m.r32(0x002c_3ce4) as i32 {
        it += 1;
        let r2 = m.rv(sp + 0x20);
        for i in 0..n {
            let mm = rmat(m, sp);
            let w = xform(mm, m.rv(pts + 16 * i), mm[0][3]);
            m.wv(sp + 0xc0, w);
            let (p1, p0) = (m.rf(prm + 4), m.rf(prm));
            let hit = m.call(0x0017_3610, &[sp + 0xc0, 0, 0], &[f4(p1), f4(p0)]).0 != 0;
            let out = if hit {
                w
            } else {
                let k = m.rf(0x002c_3ce0);
                add(add(w, [0.0, 0.0, 0.0, 1.0]), scale(r2, k))
            };
            m.wv(sp + 0x80 + 16 * i, out);
        }
        let p0 = m.rv(sp + 0x80);
        let (e1, e2) = (sub(m.rv(sp + 0x90), p0), sub(m.rv(sp + 0xa0), p0));
        let c = cross(e1, e2, p0[3]);
        let l = dot3(c, c).sqrt();
        let nrm = if l < f32::from_bits(0x38d1_b717) { m.rv(UP) } else { scale(c, 1.0 / l) };
        let r0 = m.rv(sp + 0x60);
        m.call(0x0011_ba08, &[sp + 0x110], &[r0, nrm]);
        for i in 0..3 {
            let r = m.rv(sp + 0x110 + 0x10 * i);
            m.wv(sp + 0xe0 + 0x10 * i, r);
        }
        m.call(0x0011_bcf8, &[sp + 0xe0, sp + 0x178, sp + 0xc0], &[]);
        let lim = m.rf(0x002c_3ce8);
        if lim < m.rf(sp + 0x178) {
            let ax = m.rv(sp + 0xc0);
            m.call(0x0011_b500, &[sp + 0x110], &[ax, f4(lim)]);
            for i in 0..3 {
                let r = m.rv(sp + 0x110 + 0x10 * i);
                m.wv(sp + 0xe0 + 0x10 * i, r);
            }
        }
        let r = r3(m, sp + 0xe0);
        let m0 = r3(m, sp + 0x40);
        let nm = mulrows(m0, r);
        for (i, row) in nm.into_iter().enumerate() {
            m.wv(sp + 0x10 * i as u32, [row[0], row[1], row[2], 0.0]);
        }
    }
    for i in 0..3 {
        let r = m.rv(sp + 0x10 * i);
        m.wv(o + 0x1b0 + 0x10 * i, r);
    }
    frame_end(m, fr, 2);
    stack_pop(m, 0x180);
}

fn mcopy<M: QMem + ?Sized>(m: &mut M, dst: u32, src: u32, rows: u32) {
    for i in 0..rows {
        let r = m.rv(src + 0x10 * i);
        m.wv(dst + 0x10 * i, r);
    }
}

/// FUN_001b72b0(jt, bone, world, out_rot, out_pos) (from the disassembly): local euler rotation / position of
/// `bone` for world matrix `world`. Rig r = jt[+0x13f0]. No IK root (r[+0x188] == 0): L = FUN_0022fd38(bone,
/// r[+0x180][+0x5c][+0xa0], world); *out_rot = FUN_0011bff0(L); *out_pos = FUN_0022fc48(bone, r[+0x180][+0x58]
/// [+0xa0], world.t). Otherwise (s2 = r[+0x180], root = r[+0x188]): W' = inverse(world); chain = parent links
/// (+0xa0) from root[+0x50] up to `bone`, noting the indices of s2[+0x58] (i58) and s2[+0x5c] (i5c); A =
/// FUN_0022ff40(root[+0x50] -> s2[+0x5c]), B = FUN_0022ff40(s2[+0x5c][+0xa0] -> bone) (DAT_002a2d30);
/// F = (A * (root + 0x60)) * W' * B; rot = euler(inverse(F)) -> *out_rot; P = [rot(F^-1) | chain[i5c].t];
/// fold chain matrices i58+1..n-1 (P at i5c) into world, then 0..i58-1 into W'' = (A*(root+0x60))*W', then
/// (root + 0x60), W', fp; *out_pos = inverse(..).t (FUN_0011c168 multiply, FUN_0011c318 inverse).
pub fn ee_sub_1b72b0<M: QMem + ?Sized>(m: &mut M, o: u32, bone: u32, world: u32, out_rot: u32, out_pos: u32) {
    let sp = stack_push(m, 0x300);
    let rig = m.r32(o + 0x13f0);
    if m.r32(rig + 0x188) == 0 {
        let v1 = m.r32(rig + 0x180);
        let v0 = m.r32(v1 + 0x5c);
        let pa = m.r32(v0 + 0xa0);
        m.call(0x0022_fd38, &[sp + 0x30, bone, pa, world], &[]);
        mcopy(m, sp, sp + 0x30, 3);
        let r = m.call(0x0011_bff0, &[sp], &[]).1;
        m.wv(out_rot, r);
        let rig = m.r32(o + 0x13f0);
        let v0 = m.r32(rig + 0x180);
        let v1 = m.r32(v0 + 0x58);
        let a1 = m.r32(v1 + 0xa0);
        let t = m.rv(world + 0x30);
        let p = m.call(0x0022_fc48, &[bone, a1], &[t]).1;
        m.wv(out_pos, p);
        stack_pop(m, 0x300);
        return;
    }
    let s2 = m.r32(rig + 0x180);
    m.call(0x0011_c318, &[sp + 0x60, world], &[]);
    let (chain, a, b, p, s7, fp, s5) = (sp + 0x2c0, sp + 0xa0, sp + 0xe0, sp + 0x120, sp + 0x1a0, sp + 0x160, sp + 0x1e0);
    let (mut i58, mut i5c, mut n) = (-1i32, -1i32, 0i32);
    let root = m.r32(rig + 0x188);
    let mut a0 = m.r32(root + 0x50);
    while a0 != bone && n < 12 {
        m.w32(chain + 4 * n as u32, a0);
        if a0 == m.r32(s2 + 0x58) {
            i58 = n;
        }
        if a0 == m.r32(s2 + 0x5c) {
            i5c = n;
        }
        a0 = m.r32(a0 + 0xa0);
        n += 1;
    }
    let r50 = m.r32(root + 0x50);
    let k5c = m.r32(s2 + 0x5c);
    m.call(0x0022_ff40, &[a, r50, k5c, 0x002a_2d30], &[]);
    let k5c = m.r32(s2 + 0x5c);
    let pa = m.r32(k5c + 0xa0);
    m.call(0x0022_ff40, &[b, pa, bone, 0x002a_2d30], &[]);
    m.call(0x0011_c168, &[p, a, root + 0x60], &[]);
    mcopy(m, a, p, 4);
    m.call(0x0011_c168, &[s7, a, sp + 0x60], &[]);
    m.call(0x0011_c168, &[fp, s7, b], &[]);
    m.call(0x0011_c318, &[p, fp], &[]);
    let rot = m.call(0x0011_bff0, &[p], &[]).1;
    m.wv(sp + 0x40, rot);
    m.call(0x0011_b628, &[fp], &[rot]);
    let c5c = m.r32(chain + 4 * i5c.max(0) as u32);
    let t = m.rv(c5c + 0x30);
    m.call(0x0025_6d18, &[p + 0x30], &[t]);
    for i in 0..3 {
        let r = m.rv(fp + 0x10 * i);
        m.wv(p + 0x10 * i, [r[0], r[1], r[2], 0.0]);
    }
    m.wf(p + 0x3c, 1.0);
    mcopy(m, s7, world, 4);
    mcopy(m, fp, world, 4);
    let mut s1 = n - 1;
    while i58 < s1 {
        let x = if s1 == i5c { p } else { m.r32(chain + 4 * s1 as u32) };
        m.call(0x0011_c168, &[s5, fp, x], &[]);
        mcopy(m, fp, s5, 4);
        s1 -= 1;
    }
    let mut s0 = i58 - 1;
    while s0 >= 0 {
        let x = if s0 == i5c { p } else { m.r32(chain + 4 * s0 as u32) };
        m.call(0x0011_c168, &[s5, s7, x], &[]);
        mcopy(m, s7, s5, 4);
        s0 -= 1;
    }
    m.call(0x0011_c168, &[s5, s7, root + 0x60], &[]);
    mcopy(m, s7, s5, 4);
    m.call(0x0011_c168, &[sp + 0x260, s7, sp + 0x60], &[]);
    m.call(0x0011_c168, &[sp + 0x220, sp + 0x260, fp], &[]);
    m.call(0x0011_c318, &[s5, sp + 0x220], &[]);
    let t = m.rv(s5 + 0x30);
    m.wv(sp + 0x50, t);
    if out_rot != 0 {
        m.wv(out_rot, rot);
    }
    if out_pos != 0 {
        m.wv(out_pos, t);
    }
    stack_pop(m, 0x300);
}
