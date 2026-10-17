//! WP-E: derived collision data the loaders skip (GEOM runtime layout of FUN_00117fd0 / FUN_00118220).
//! GEOM: +0 vertex count, +4 vertices (16 B each), +8/+0xc surfaces (0x20 B: plane xyzw +0, own edge
//! list +0x10, foreign edge list +0x14, first vertex +0x1c), +0x10/+0x14 edge pool (0x10 B: u16 a, u16 b,
//! u16 owner surface, u16 other surface, +8 next in owner list, +0xc next in other surface's list).
//! BSPC (FUN_00118220): +0 node count, +4/+8 two u32s, +0xc nodes (0xc B: +0 neg child, +4 pos child,
//! +8 surface*; 0xffff -> DAT_002bbaf0 = plane (0,0,0,1e8), i.e. a solid leaf).
#![allow(non_snake_case)]

use crate::engine::world::loads::{Bsp, Geom};

pub type V3 = [f32; 3];
pub type Plane = [f32; 4];

/// DAT_002bbaf0: plane of a 0xffff (leaf) BSP node.
pub const LEAF_PLANE: Plane = [0.0, 0.0, 0.0, 100000000.0];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Edge {
    pub a: u16,
    pub b: u16,
    /// +4: owning surface index
    pub owner: u16,
    /// +6: other surface index
    pub other: u16,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Surf {
    /// +0: unit normal, w = distance (FUN_00119308)
    pub plane: Plane,
    /// +0x10 list (stream order)
    pub own_edges: Vec<usize>,
    /// +0x14 list (prepended by FUN_00117fd0 -> reverse insertion order)
    pub other_edges: Vec<usize>,
    /// +0x1c: first vertex index
    pub v0: u16,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CGeom {
    pub verts: Vec<V3>,
    pub surfs: Vec<Surf>,
    pub edges: Vec<Edge>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BspNode {
    /// +0 (negative side), +4 (positive side); -1 / unset = null
    pub child: [Option<usize>; 2],
    /// +8: surface index; None = DAT_002bbaf0
    pub surf: Option<usize>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BspC {
    pub a: u32,
    pub b: u32,
    pub nodes: Vec<BspNode>,
}

pub fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
/// VU0 dot idiom (vmul, then x + y + z)
pub fn dot(a: V3, b: V3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// 0x00119308: plane through three points: n = normalize((p1-p0) x (p2-p0)), w = n.p0.
pub fn FUN_00119308(p0: V3, p1: V3, p2: V3) -> Plane {
    let a = sub(p1, p0);
    let b = sub(p2, p0);
    // vopmula a,b ; vopmsub b,a
    let c = [a[1] * b[2] - b[1] * a[2], a[2] * b[0] - b[2] * a[0], a[0] * b[1] - b[0] * a[1]];
    let q = 1.0 / dot(c, c).sqrt();
    let n = [c[0] * q, c[1] * q, c[2] * q];
    [n[0], n[1], n[2], dot(n, p0)]
}

/// 0x00119390: plane from a normal and a point.
pub fn FUN_00119390(n: V3, p: V3) -> Plane {
    [n[0], n[1], n[2], dot(n, p)]
}

/// 0x0011f288: component-wise min, starting from DAT_002a2df0 (FLT_MAX).
pub fn FUN_0011f288(v: &[V3]) -> V3 {
    v.iter().fold([f32::MAX; 3], |m, p| [m[0].min(p[0]), m[1].min(p[1]), m[2].min(p[2])])
}
/// 0x0011f2b8: component-wise max, starting from DAT_002a2e00 (-FLT_MAX).
pub fn FUN_0011f2b8(v: &[V3]) -> V3 {
    v.iter().fold([-f32::MAX; 3], |m, p| [m[0].max(p[0]), m[1].max(p[1]), m[2].max(p[2])])
}

/// FUN_00227bb8 derived tail: (+0x110 centre, +0x120 value). Quirk kept: the listing has no vsqrt, so
/// +0x120 is |centre - min|^2; empty geometry -> DAT_002a9880 (-FLT_MAX) and the centre untouched (zero).
pub fn bounding_sphere_227bb8(g: &CGeom) -> (V3, f32) {
    if g.verts.is_empty() {
        return ([0.0; 3], -f32::MAX);
    }
    let mn = FUN_0011f288(&g.verts);
    let mx = FUN_0011f2b8(&g.verts);
    let c = [mn[0] * 0.5 + mx[0] * 0.5, mn[1] * 0.5 + mx[1] * 0.5, mn[2] * 0.5 + mx[2] * 0.5];
    let d = sub(c, mn);
    (c, dot(d, d))
}

/// 0x00117fd0 derived part: planes recomputed with FUN_00119308 (stream floats discarded), edge records
/// linked into the owner's +0x10 list and prepended to the other surface's +0x14 list.
pub fn build_geom(g: &Geom) -> CGeom {
    let verts = g.verts.clone();
    let mut surfs: Vec<Surf> = Vec::with_capacity(g.surfaces.len());
    let mut edges = Vec::new();
    let mut other: Vec<Vec<usize>> = vec![Vec::new(); g.surfaces.len()];
    for (i, s) in g.surfaces.iter().enumerate() {
        let v = |k: u16| verts.get(k as usize).copied().unwrap_or([0.0; 3]);
        let plane = FUN_00119308(v(s.verts[0]), v(s.verts[1]), v(s.verts[2]));
        let mut own = Vec::new();
        for e in &s.edges {
            let k = edges.len();
            edges.push(Edge { a: e[0], b: e[1], owner: i as u16, other: e[2] });
            own.push(k);
            if let Some(o) = other.get_mut(e[2] as usize) {
                o.insert(0, k);
            }
        }
        surfs.push(Surf { plane, own_edges: own, other_edges: Vec::new(), v0: s.verts[0] });
    }
    for (s, o) in surfs.iter_mut().zip(other) {
        s.other_edges = o;
    }
    CGeom { verts, surfs, edges }
}

/// 0x00118220 runtime nodes (child index -1 stays null; surface 0xffff -> DAT_002bbaf0).
pub fn build_bsp(b: &Bsp) -> BspC {
    let nodes = b
        .nodes
        .iter()
        .map(|n| {
            let c = |i: i32| (i != -1).then_some(i as usize);
            match n.children {
                None => BspNode { child: [None, None], surf: None },
                Some([x, y]) => BspNode { child: [c(x), c(y)], surf: Some(n.surface as usize) },
            }
        })
        .collect();
    BspC { a: b.a, b: b.b, nodes }
}

/// 0x00166270: copy a GEOM, transforming it by rows r0..r3 (v' = r0 x + r1 y + r2 z + r3) when `m` is set;
/// normals use the 3x3 part, are renormalised (1/sqrt) and re-anchored at vertex +0x1c (FUN_00119390).
pub fn FUN_00166270(src: &CGeom, m: Option<&[[f32; 3]; 4]>) -> CGeom {
    let mut g = src.clone();
    if let Some(m) = m {
        for v in &mut g.verts {
            let p = *v;
            for k in 0..3 {
                v[k] = m[0][k] * p[0] + m[1][k] * p[1] + m[2][k] * p[2] + m[3][k];
            }
        }
        for s in &mut g.surfs {
            let n0 = [s.plane[0], s.plane[1], s.plane[2]];
            let mut n = [0.0f32; 3];
            for k in 0..3 {
                n[k] = m[0][k] * n0[0] + m[1][k] * n0[1] + m[2][k] * n0[2];
            }
            let q = 1.0 / dot(n, n).sqrt();
            let n = [n[0] * q, n[1] * q, n[2] * q];
            s.plane = FUN_00119390(n, g.verts.get(s.v0 as usize).copied().unwrap_or([0.0; 3]));
        }
    }
    g
}
