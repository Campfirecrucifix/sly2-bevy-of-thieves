//! WP-E: BSP segment clipping (ClsgClipEdgeToBsp, 0x00118668) and LSG list merge (0x0014c9f8).
#![allow(non_snake_case)]

use super::geom::{dot, BspC, CGeom, Plane, V3, LEAF_PLANE};

/// One 0x60-byte LSG record written by 0x00118668.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Lsg {
    /// +0x00 / +0x10: points at t_enter / t_exit
    pub pos: [V3; 2],
    /// +0x20 / +0x30: splitting planes at entry / exit (zero = segment start inside)
    pub plane: [Plane; 2],
    /// +0x40 / +0x44
    pub t: [f32; 2],
    /// +0x4c / +0x50: surface index of the entry / exit plane
    pub surf: [Option<usize>; 2],
}

fn plane_of(g: &CGeom, s: Option<usize>) -> Plane {
    match s {
        None => LEAF_PLANE,
        Some(i) => g.surfs.get(i).map(|s| s.plane).unwrap_or([0.0; 4]),
    }
}
/// plane distance (vmul, x+y+z, vmsubw)
fn dist(p: Plane, v: V3) -> f32 {
    dot([p[0], p[1], p[2]], v) - p[3]
}
/// PS2 FPU / VU results round toward zero (not IEEE nearest): exact f64 result truncated to f32.
fn rtz(x: f64) -> f32 {
    let y = x as f32;
    if (y as f64).abs() > x.abs() { f32::from_bits(y.to_bits() - 1) } else { y }
}
/// 0x118750 sub.s (1 - t) + vmulax ACC = a * u, vmaddx = ACC + b * t (each step rounded toward zero; IEEE
/// nearest put an exit point exactly on its plane, e.g. the guard drop probe FUN_001c6de8, WIRE-G)
fn lerp(a: V3, b: V3, t: f32) -> V3 {
    let u = rtz(1.0 - t as f64);
    std::array::from_fn(|i| rtz(rtz(a[i] as f64 * u as f64) as f64 + rtz(b[i] as f64 * t as f64) as f64))
}

struct Entry {
    node: Option<usize>,
    enter: Option<usize>,
    t: f32,
    surf: Option<usize>,
    plane: Plane,
}

/// 0x00118668: clip segment p1->p2 against the BSP rooted at `root`; node indices >= `limit` count as null
/// (the original compares node pointers against param_2; callers pass 0x7fffffff or absp + 0xc*cbsp).
/// Appends solid intervals to `out` (adjacent intervals sharing a t are merged), returns the number
/// written, or `max` as soon as a new record would exceed it. Null negative child = solid, null positive = empty.
pub fn FUN_00118668(g: &CGeom, bsp: &BspC, root: usize, limit: usize, p1: V3, p2: V3, max: usize, out: &mut Vec<Lsg>) -> usize {
    let lim = |i: Option<usize>| i.filter(|&i| i < limit && i < bsp.nodes.len());
    let base = out.len();
    let mut written = 0usize;
    let mut prev: Option<usize> = None;
    let mut stack = vec![Entry { node: None, enter: None, t: 1.0, surf: None, plane: [0.0; 4] }];
    let mut cur = lim(Some(root));
    let mut last: Option<usize> = None;
    let mut t0 = 0.0f32;
    let mut esurf: Option<usize> = None;
    let mut eplane: Plane = [0.0; 4];
    loop {
        while cur.is_none() {
            if last.is_some() {
                let top = stack.last().unwrap();
                match prev {
                    Some(p) if out[p].t[1] == t0 => {
                        let r = &mut out[p];
                        r.pos[1] = lerp(p1, p2, top.t);
                        r.plane[1] = top.plane;
                        r.t[1] = top.t;
                        r.surf[1] = top.surf;
                    }
                    _ => {
                        if max <= written {
                            return max;
                        }
                        written += 1;
                        out.push(Lsg {
                            pos: [lerp(p1, p2, t0), lerp(p1, p2, top.t)],
                            plane: [eplane, top.plane],
                            t: [t0, top.t],
                            surf: [esurf, top.surf],
                        });
                        prev = Some(out.len() - 1);
                    }
                }
            }
            if stack.len() <= 1 {
                debug_assert_eq!(out.len() - base, written);
                return written;
            }
            let e = stack.pop().unwrap();
            last = e.enter;
            t0 = e.t;
            esurf = e.surf;
            eplane = e.plane;
            cur = e.node;
        }
        let n = cur.unwrap();
        let node = bsp.nodes[n];
        let plane = plane_of(g, node.surf);
        let d1 = dist(plane, p1);
        let d2 = dist(plane, p2);
        let neg;
        if !(0.0 < d1) && !(0.0 < d2) {
            neg = true;
        } else if 0.0 < d1 && 0.0 < d2 {
            neg = false;
        } else {
            let tend = stack.last().unwrap().t;
            let t = d1 / (d1 - d2);
            let f = d2 * t0 + (1.0 - t0) * d1;
            if t0 < t && t < tend {
                let pos = 0.0 < f;
                stack.push(Entry {
                    node: lim(node.child[if pos { 0 } else { 1 }]),
                    enter: pos.then_some(n),
                    t,
                    surf: node.surf,
                    plane,
                });
            }
            neg = f < 0.0 || (f == 0.0 && d2 * tend + (1.0 - tend) * d1 <= 0.0);
        }
        if neg {
            last = Some(n);
            cur = lim(node.child[0]);
        } else {
            last = None;
            cur = lim(node.child[1]);
        }
    }
}

/// 0x0014c9f8: merge two sorted LSG lists. mode 0 = union (inside when >= 1 list is), 1 = intersection.
/// Quirk kept: stops once the output count exceeds `max` (the original then copies max+1 records).
pub fn FUN_0014c9f8(a: &[Lsg], b: &[Lsg], max: usize, mode: i32) -> Vec<Lsg> {
    let need = if mode == 0 { 1 } else { 2 };
    let (mut ia, mut ib) = (0usize, 0usize);
    let (mut sa, mut sb) = (0usize, 0usize);
    let mut inside = false;
    let mut out: Vec<Lsg> = Vec::new();
    let mut cur = Lsg::default();
    loop {
        if ia >= a.len() && ib >= b.len() {
            return out;
        }
        // DAT_002a4ff0 = FLT_MAX
        let ta = if ia < a.len() { a[ia].t[sa] } else { f32::MAX };
        let tb = if ib < b.len() { b[ib].t[sb] } else { f32::MAX };
        let (rec, old) = if ta <= tb {
            let old = sa;
            sa = 1 - sa;
            (a[ia], old)
        } else {
            let old = sb;
            sb = 1 - sb;
            (b[ib], old)
        };
        let now = need <= sa + sb;
        if now != inside {
            inside = now;
            if now {
                cur.t[0] = rec.t[0];
                cur.pos[0] = rec.pos[0];
                cur.plane[0] = rec.plane[0];
                cur.surf[0] = rec.surf[0];
            } else {
                cur.t[1] = rec.t[1];
                cur.pos[1] = rec.pos[1];
                cur.plane[1] = rec.plane[1];
                cur.surf[1] = rec.surf[1];
                out.push(cur);
                if max < out.len() {
                    return out;
                }
            }
        }
        if old == 1 {
            if ta <= tb { ia += 1 } else { ib += 1 }
        }
    }
}
