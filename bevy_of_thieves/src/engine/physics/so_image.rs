//! FEEL R51 (hull port step 3a): EE-memory images of the runtime GEOM / BSPC layouts (see `geom` module doc:
//! FUN_00117fd0 / FUN_00118220) so the 1:1 QMem mesh/mesh chain (FUN_001aa1d8 -> FUN_001a8b50 -> 0x1a80d8 /
//! 0x1a81f8 / 0x1a8410 / 0x1a87f0 / 0x1a86b8 / FUN_00118668 / FUN_00118a88) can run on the port's `CGeom` /
//! `BspC` data. Layout written:
//! GEOM (at `base`): +0 vertex count, +4 vertices (16 B, w = 1), +8 surface count, +0xc surfaces (0x20 B: plane
//! +0, own edge list head +0x10, foreign edge list head +0x14, flags +0x18 = 0, first vertex +0x1c), +0x10 edge
//! count, +0x14 edges (0x10 B: u16 a, b, owner surface, other surface, +8 next in owner list, +0xc next in the
//! other surface's list).
//! BSPC (at `base`): +0 node count, +4 / +8 the two u32s, +0xc nodes (0xc B: +0 neg child*, +4 pos child*, +8
//! surface* of `surfs` or DAT_002bbaf0 for a 0xffff leaf).
//! In an SO the BSPC sits at +0x68 (+0x74 = nodes) and the world GEOM at +0x90 (+0x94 verts, +0x9c surfaces,
//! +0xa0 / +0xa4 edges), per FUN_001f70c0 / FUN_001191e8.
#![allow(non_snake_case)]

use super::geom::{BspC, CGeom, LEAF_PLANE};
use crate::engine::classes::q::q2mem::QMem;

/// DAT_002bbaf0: the plane every 0xffff BSP node points at.
pub const DAT_002BBAF0: u32 = 0x002b_baf0;

/// Scratch area handed out for FUN_001ae318 (the 0x118a88 candidate frame, <= 0x1555 * 12 bytes).
pub const FRAME_BUF: u32 = 0x0200_0000;

/// FUN_001a7c18's list side only (the record itself is built by the caller from the logged call): a node
/// {+0x9c kind, +0xa0 featA, +0xa4 featB, +0xac next} prepended to *list, so FUN_001a8b50's dedupe walk and
/// its "list head unchanged" 0xd-fallback test see the original list. Nodes live at NODE_BUF (count at -0x10).
pub fn list_add<M: QMem + ?Sized>(m: &mut M, a: &[u32]) -> u32 {
    let k = m.r32(NODE_BUF - 0x10);
    m.w32(NODE_BUF - 0x10, k + 1);
    let n = NODE_BUF + 0xb0 * k;
    let list = a[6];
    m.w32(n + 0x9c, a[3]);
    m.w32(n + 0xa0, a[4]);
    m.w32(n + 0xa4, a[5]);
    let head = m.r32(list);
    m.w32(n + 0xac, head);
    m.w32(list, n);
    0
}
pub const NODE_BUF: u32 = 0x0300_0000;

/// Bump allocator over a QMem address range (16-byte aligned).
pub struct Bump(pub u32);

impl Bump {
    pub fn take(&mut self, n: u32) -> u32 {
        let a = (self.0 + 15) & !15;
        self.0 = a + n;
        a
    }
}

fn wv<M: QMem + ?Sized>(m: &mut M, a: u32, v: [f32; 4]) {
    for (k, x) in v.iter().enumerate() {
        m.w32(a + 4 * k as u32, x.to_bits());
    }
}

/// GEOM image at `base`; returns the surfaces address (for BSP node +8).
pub fn write_geom<M: QMem + ?Sized>(m: &mut M, base: u32, g: &CGeom, heap: &mut Bump) -> u32 {
    let verts = heap.take(16 * g.verts.len() as u32);
    for (i, v) in g.verts.iter().enumerate() {
        wv(m, verts + 16 * i as u32, [v[0], v[1], v[2], 1.0]);
    }
    let surfs = heap.take(0x20 * g.surfs.len() as u32);
    let edges = heap.take(0x10 * g.edges.len() as u32);
    let ea = |k: usize| edges + 0x10 * k as u32;
    for (i, e) in g.edges.iter().enumerate() {
        m.w16(ea(i), e.a);
        m.w16(ea(i) + 2, e.b);
        m.w16(ea(i) + 4, e.owner);
        m.w16(ea(i) + 6, e.other);
    }
    for (i, s) in g.surfs.iter().enumerate() {
        let sa = surfs + 0x20 * i as u32;
        wv(m, sa, s.plane);
        // own list: +0x10 head, linked by +8; foreign list: +0x14 head, linked by +0xc
        for (head, link, list) in [(0x10u32, 8u32, &s.own_edges), (0x14, 0xc, &s.other_edges)] {
            let mut prev = sa + head;
            for &k in list.iter() {
                m.w32(prev, ea(k));
                prev = ea(k) + link;
            }
            m.w32(prev, 0);
        }
        m.w32(sa + 0x18, 0);
        m.w32(sa + 0x1c, s.v0 as u32);
    }
    m.w32(base, g.verts.len() as u32);
    m.w32(base + 4, verts);
    m.w32(base + 8, g.surfs.len() as u32);
    m.w32(base + 0xc, surfs);
    m.w32(base + 0x10, g.edges.len() as u32);
    m.w32(base + 0x14, edges);
    surfs
}

/// SCUS_973.16 rodata the mesh/mesh chain reads: DAT_002a6ad0 = -FLT_MAX (0xff7fffff, FUN_001a87f0 best-face
/// start), DAT_002a2dc0 = (0, 0, 1, 0) (FUN_001a8b50 0xd fallback normal), DAT_002bbaf0 = leaf plane.
pub fn write_rodata<M: QMem + ?Sized>(m: &mut M) {
    m.w32(0x002a_6ad0, 0xff7f_ffff);
    wv(m, 0x002a_2dc0, [0.0, 0.0, 1.0, 0.0]);
    wv(m, DAT_002BBAF0, LEAF_PLANE);
    // FEEL R89 (phantom contact root, R85 'random teleport'): FUN_00118a88's pruned-node sentinels. DAT_002bbb10 =
    // node {null, null, &DAT_002bbad0 (0,0,0,-1e8)} = EMPTY leaf (a pruned child wholly on a node's empty side),
    // DAT_002bbb20 = {null, null, &DAT_002bbaf0} = SOLID leaf. Unwritten (all zero) the empty sentinel read a
    // zero plane -> d = 0 -> negative null child = SOLID: hull edges / vertices in empty space clipped as inside
    // the level (deep kind-1 records against a far splitting plane, e.g. EP1 hub surf 8975). SLY2_FEEL_R89SENT=0 = old.
    if std::env::var("SLY2_FEEL_R89SENT").map_or(true, |v| v != "0") {
        wv(m, 0x002b_bad0, [0.0, 0.0, 0.0, -100000000.0]);
        for (a, pl) in [(0x002b_bb10u32, 0x002b_bad0u32), (0x002b_bb20, DAT_002BBAF0)] {
            m.w32(a, 0);
            m.w32(a + 4, 0);
            m.w32(a + 8, pl);
        }
    }
}

/// BSPC image at `base` whose nodes point into the surfaces at `surfs`; also writes DAT_002bbaf0.
pub fn write_bspc<M: QMem + ?Sized>(m: &mut M, base: u32, b: &BspC, surfs: u32, heap: &mut Bump) {
    wv(m, DAT_002BBAF0, LEAF_PLANE);
    let nodes = heap.take(0xc * b.nodes.len() as u32);
    let na = |k: usize| nodes + 0xc * k as u32;
    for (i, n) in b.nodes.iter().enumerate() {
        m.w32(na(i), n.child[0].map_or(0, na));
        m.w32(na(i) + 4, n.child[1].map_or(0, na));
        m.w32(na(i) + 8, n.surf.map_or(DAT_002BBAF0, |s| surfs + 0x20 * s as u32));
    }
    m.w32(base, b.nodes.len() as u32);
    m.w32(base + 4, b.a);
    m.w32(base + 8, b.b);
    m.w32(base + 0xc, if b.nodes.is_empty() { 0 } else { nodes });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::classes::q::q2mem::FlatMem;
    use crate::engine::physics::geom::{BspNode, Edge, Surf};

    #[test]
    fn so_image_layout() {
        let g = CGeom {
            verts: vec![[1., 2., 3.], [4., 5., 6.]],
            surfs: vec![
                Surf { plane: [0., 0., 1., 2.], own_edges: vec![0, 1], other_edges: vec![], v0: 1 },
                Surf { plane: [1., 0., 0., 0.], own_edges: vec![], other_edges: vec![1, 0], v0: 0 },
            ],
            edges: vec![Edge { a: 0, b: 1, owner: 0, other: 1 }, Edge { a: 1, b: 0, owner: 0, other: 1 }],
        };
        let b = BspC { a: 7, b: 9, nodes: vec![BspNode { child: [None, Some(1)], surf: Some(1) }, BspNode { child: [None, None], surf: None }] };
        let mut m = FlatMem::default();
        let mut h = Bump(0x10_0000);
        let so = 0x1000;
        let surfs = write_geom(&mut m, so + 0x90, &g, &mut h);
        write_bspc(&mut m, so + 0x68, &b, surfs, &mut h);
        assert_eq!(m.r32(so + 0x90), 2);
        assert_eq!(m.r32(so + 0x9c), surfs);
        assert_eq!(m.r32(so + 0xa0), 2);
        let (v, e) = (m.r32(so + 0x94), m.r32(so + 0xa4));
        assert_eq!(m.rv(v + 16), [4., 5., 6., 1.]);
        assert_eq!(m.rv(surfs), [0., 0., 1., 2.]);
        // own list of surf 0: e0 -> e1 via +8; foreign list of surf 1: e1 -> e0 via +0xc
        assert_eq!(m.r32(surfs + 0x10), e);
        assert_eq!(m.r32(e + 8), e + 0x10);
        assert_eq!(m.r32(e + 0x18), 0);
        assert_eq!(m.r32(surfs + 0x20 + 0x14), e + 0x10);
        assert_eq!(m.r32(e + 0x10 + 0xc), e);
        assert_eq!((m.r16(e + 0x10), m.r16(e + 0x12), m.r16(e + 0x14), m.r16(e + 0x16)), (1, 0, 0, 1));
        assert_eq!(m.r32(surfs + 0x1c), 1);
        let n = m.r32(so + 0x74);
        assert_eq!((m.r32(so + 0x68), m.r32(so + 0x6c), m.r32(so + 0x70)), (2, 7, 9));
        assert_eq!((m.r32(n), m.r32(n + 4), m.r32(n + 8)), (0, n + 0xc, surfs + 0x20));
        assert_eq!(m.r32(n + 0xc + 8), DAT_002BBAF0);
        assert_eq!(m.rv(DAT_002BBAF0), LEAF_PLANE);
    }
}

#[cfg(test)]
pub(crate) mod e2e {
    use super::*;
    use crate::engine::classes::q::q2mem::FlatMem;
    use crate::engine::physics::geom::{build_geom, FUN_00119308};
    use crate::engine::world::loads::{Geom, Surface};
    type V4 = [f32; 4];

    /// cube of half size h at c: verts i = x | y << 1 | z << 2, 6 quads with outward FUN_00119308 normals.
    pub fn cube(c: [f32; 3], h: f32) -> (CGeom, BspC) {
        let verts: Vec<[f32; 3]> = (0..8).map(|i| [c[0] + if i & 1 != 0 { h } else { -h }, c[1] + if i & 2 != 0 { h } else { -h }, c[2] + if i & 4 != 0 { h } else { -h }]).collect();
        let quads: [[u16; 4]; 6] = [[1, 3, 7, 5], [0, 4, 6, 2], [2, 6, 7, 3], [0, 1, 5, 4], [4, 5, 7, 6], [0, 2, 3, 1]];
        let has = |q: &[u16; 4], a: u16, b: u16| q.contains(&a) && q.contains(&b);
        let surfaces = quads
            .iter()
            .enumerate()
            .map(|(i, q)| Surface {
                verts: [q[0], q[1], q[2]],
                discarded: [0.0; 4],
                edges: (0..4)
                    .map(|k| {
                        let (a, b) = (q[k], q[(k + 1) % 4]);
                        let o = quads.iter().enumerate().position(|(j, r)| j != i && has(r, a, b)).unwrap() as u16;
                        [a, b, o]
                    })
                    .collect(),
            })
            .collect();
        let g = build_geom(&Geom { verts, surfaces, edge_pool: 24 });
        for s in &g.surfs {
            let _ = FUN_00119308; // planes already built by build_geom
            assert!(s.plane[0].abs() + s.plane[1].abs() + s.plane[2].abs() > 0.9);
        }
        // convex chain: node i tests surface i; negative (inside) side continues, positive side = null
        let nodes = (0..6).map(|i| crate::engine::physics::geom::BspNode { child: [(i < 5).then_some(i + 1), None], surf: Some(i) }).collect();
        (g, BspC { a: 6, b: 0, nodes })
    }

    fn route(m: &mut FlatMem, addr: u32, a: &[u32], v: &[V4]) -> Option<(u32, V4)> {
        // BOSS R35: + 0x2332b8 (memmove): FUN_001a8410 compacts its key / chain-head tables with it; unrouted it was a
        // no-op and a removed singleton chain could be pushed twice onto the +0x5c head list (self-loop -> FUN_001a86b8 hang)
        const R: [u32; 12] = [0x1a80d8, 0x1a81f8, 0x1a8410, 0x1a87f0, 0x1a86b8, 0x11ea60, 0x118668, 0x118a88, 0x2564b8, 0x2564c0, 0x2333b4, 0x2332b8];
        if addr == 0x1a7c18 {
            return Some((list_add(m, a), [0.0; 4]));
        }
        if addr == 0x1ae318 {
            // FUN_001ae318 frame allocation (one per FUN_001aa1d8 call): a fixed scratch area
            return Some((FRAME_BUF, [0.0; 4]));
        }
        if !R.contains(&addr) {
            return None;
        }
        let t = crate::engine::classes::q::live_dispatch::TABLE;
        let i = t.binary_search_by_key(&addr, |e| e.0).ok()?;
        Some((t[i].1)(m as &mut dyn QMem, a, v))
    }

    /// body image: +0x20 centre, +0x30 radius, +0x34 SO chain; SO: [so] owner (+0x70 centre), +0x6c BSPC a.
    fn body(m: &mut FlatMem, b: u32, so: u32, owner: u32, c: [f32; 3], r: f32, g: &CGeom, bsp: &BspC, h: &mut Bump) {
        m.wv(b + 0x20, [c[0], c[1], c[2], 1.0]);
        m.wf(b + 0x30, r);
        m.w32(b + 0x34, so);
        m.w32(so, owner);
        m.wv(owner + 0x70, [c[0], c[1], c[2], 1.0]);
        m.wv(so + 0x20, [c[0], c[1], c[2], 1.0]);
        m.wf(so + 0x30, r);
        m.wf(so + 8, r);
        let s = write_geom(m, so + 0x90, g, h);
        write_bspc(m, so + 0x68, bsp, s, h);
    }

    /// FEEL R51 step 3b: two overlapping unit cubes (A at 0, B at x = 1.5, overlap 0.5) through the 1:1 chain
    /// FUN_001aa1d8 -> FUN_001a8b50 -> callees on so_image data; FUN_001a7c18 left as a logged stub.
    #[test]
    fn so_image_cube_cube_contacts() {
        let mut m = FlatMem { route: Some(route), ..Default::default() };
        let mut h = Bump(0x10_0000);
        write_rodata(&mut m);
        let (ga, ba) = cube([0.0; 3], 1.0);
        let (gb, bb) = cube([1.5, 0.0, 0.0], 1.0);
        body(&mut m, 0x1000, 0x2000, 0x2800, [0.0; 3], 1.8, &ga, &ba, &mut h);
        body(&mut m, 0x3000, 0x4000, 0x4800, [1.5, 0.0, 0.0], 1.9, &gb, &bb, &mut h);
        let _ = crate::engine::classes::q::q_part1::ee_sub_1aa1d8(&mut m, 0x100, 0x1000, 0x3000);
        let mut seen: Vec<u32> = m.calls.iter().map(|c| c.0).collect();
        seen.sort();
        seen.dedup();
        eprintln!("called: {:x?}", seen);
        for c in m.calls.iter().filter(|c| c.0 == 0x001a_7c18) {
            eprintln!("7c18 kind {} a {:x?} v {:?}", c.1[3], &c.1[..7], c.2);
        }
        // 4 corners of A at x = 1 inside B (B face -x, w -0.5) and 4 of B at x = 0.5 inside A (A face +x, w 1),
        // all vertex / face (kind 1) at depth -0.5
        let c: Vec<_> = m.calls.iter().filter(|c| c.0 == 0x001a_7c18).collect();
        assert_eq!(c.len(), 8);
        for (i, c) in c.iter().enumerate() {
            assert_eq!(c.1[3], 1);
            assert_eq!(c.2[0][0], -0.5);
            let (side, pl, x) = if i < 4 { (0x2000, [-1.0, 0.0, 0.0, -0.5], 1.0) } else { (0x4000, [1.0, 0.0, 0.0, 1.0], 0.5) };
            assert_eq!(c.1[1], side);
            assert_eq!(c.2[2], pl);
            assert_eq!(c.2[1][0], x);
        }
    }
}

/// FEEL R51 step 3c: one FUN_001aa1d8 IntersectBodies pair (1:1 QMem chain) between a PO hull SO (world-space
/// `hull` geom + its BSP, body sphere `hc` / `hr`) and a level collider (world geom). Returns the FUN_001a7c18
/// records as (side A is the hull, gap, P, n, kind). FUN_001a7c18 itself (pool / notify) is the caller's
/// (motor `Xp` / p5 `ContactRec` builders); FUN_001aa0b0 / FUN_001a9fe0 / FUN_001aa020 are logged no-ops here
/// (static level body: no SO +0x6c lists).
const HS: u32 = 0x2000;

#[allow(clippy::too_many_arguments)]
fn write_body<M: QMem + ?Sized>(m: &mut M, b: u32, so: u32, owner: u32, c: [f32; 3], r: f32, g: &CGeom, bsp: &BspC, h: &mut Bump) {
    wv(m, b + 0x20, [c[0], c[1], c[2], 1.0]);
    m.w32(b + 0x30, r.to_bits());
    m.w32(b + 0x34, so);
    m.w32(so, owner);
    wv(m, owner + 0x70, [c[0], c[1], c[2], 1.0]);
    wv(m, so + 0x20, [c[0], c[1], c[2], 1.0]);
    m.w32(so + 0x30, r.to_bits());
    m.w32(so + 8, r.to_bits());
    let s = write_geom(m, so + 0x90, g, h);
    write_bspc(m, so + 0x68, bsp, s, h);
}

type Image = std::sync::Arc<std::collections::HashMap<u32, u8>>;

/// Byte memory for the image writers (no calls).
#[derive(Default)]
struct ImgMem(std::collections::HashMap<u32, u8>);
impl QMem for ImgMem {
    fn r8(&mut self, a: u32) -> u8 {
        self.0.get(&a).copied().unwrap_or(0)
    }
    fn w8(&mut self, a: u32, v: u8) {
        self.0.insert(a, v);
    }
    fn call(&mut self, _: u32, _: &[u32], _: &[[f32; 4]]) -> (u32, [f32; 4]) {
        (0, [0.0; 4])
    }
}

/// The level body image (body 0x3000, SO 0x4000, owner 0x4800, heap 0x0400_0000), cached per thread by a
/// hash of the collider's data (static level parts never change; a moved part rebuilds).
fn level_image(lvl: &CGeom, lbsp: &BspC, lc: [f32; 3], lr: f32) -> Image {
    use std::hash::{Hash, Hasher};
    let mut hs = std::collections::hash_map::DefaultHasher::new();
    for v in lvl.verts.iter().chain(std::iter::once(&lc)) {
        v.map(f32::to_bits).hash(&mut hs);
    }
    (lvl.surfs.len(), lvl.edges.len(), lbsp.nodes.len(), lr.to_bits()).hash(&mut hs);
    let key = hs.finish();
    thread_local! {
        static CACHE: std::cell::RefCell<Vec<(u64, Image)>> = const { std::cell::RefCell::new(Vec::new()) };
    }
    CACHE.with(|c| {
        if let Some((_, img)) = c.borrow().iter().find(|e| e.0 == key) {
            return img.clone();
        }
        let mut m = ImgMem::default();
        write_body(&mut m, 0x3000, 0x4000, 0x4800, lc, lr, lvl, lbsp, &mut Bump(0x0400_0000));
        let img: Image = std::sync::Arc::new(m.0);
        let mut c = c.borrow_mut();
        if c.len() >= 16 {
            c.remove(0);
        }
        c.push((key, img.clone()));
        img
    })
}

/// Overlay memory: reads the per-call writes first, then the shared level image.
struct LayeredMem {
    base: Image,
    over: std::collections::HashMap<u32, u8>,
    calls: Vec<(u32, Vec<u32>, Vec<[f32; 4]>)>,
    route: fn(&mut LayeredMem, u32, &[u32], &[[f32; 4]]) -> Option<(u32, [f32; 4])>,
}
impl QMem for LayeredMem {
    fn scrub_popped_stack(&self) -> bool {
        false
    }
    fn r8(&mut self, a: u32) -> u8 {
        self.over.get(&a).or_else(|| self.base.get(&a)).copied().unwrap_or(0)
    }
    fn w8(&mut self, a: u32, v: u8) {
        self.over.insert(a, v);
    }
    fn call(&mut self, addr: u32, a: &[u32], v: &[[f32; 4]]) -> (u32, [f32; 4]) {
        self.calls.push((addr, a.to_vec(), v.to_vec()));
        (self.route)(self, addr, a, v).unwrap_or((0, [0.0; 4]))
    }
}

pub fn mesh_pair(hull: &CGeom, hbsp: &BspC, hc: [f32; 3], hr: f32, lvl: &CGeom, lbsp: &BspC, lc: [f32; 3], lr: f32) -> Vec<(bool, f32, [f32; 3], [f32; 3], i32)> {
    type V4 = [f32; 4];
    fn route(m: &mut LayeredMem, addr: u32, a: &[u32], v: &[V4]) -> Option<(u32, V4)> {
        // BOSS R35: + 0x2332b8 (memmove): FUN_001a8410 compacts its key / chain-head tables with it; unrouted it was a
        // no-op and a removed singleton chain could be pushed twice onto the +0x5c head list (self-loop -> FUN_001a86b8 hang)
        const R: [u32; 12] = [0x1a80d8, 0x1a81f8, 0x1a8410, 0x1a87f0, 0x1a86b8, 0x11ea60, 0x118668, 0x118a88, 0x2564b8, 0x2564c0, 0x2333b4, 0x2332b8];
        if addr == 0x1a7c18 {
            return Some((list_add(m, a), [0.0; 4]));
        }
        if addr == 0x1ae318 {
            // FUN_001ae318 frame allocation (one per FUN_001aa1d8 call): a fixed scratch area
            return Some((FRAME_BUF, [0.0; 4]));
        }
        if !R.contains(&addr) {
            return None;
        }
        let t = crate::engine::classes::q::live_dispatch::TABLE;
        let i = t.binary_search_by_key(&addr, |e| e.0).ok()?;
        Some((t[i].1)(m as &mut dyn QMem, a, v))
    }
    // FEEL R53: the level body image (static per collider) is built once and shared read-only; the hull body
    // and every chain write go to the per-call overlay. Level image heap at 0x0400_0000 (R51-R52 used one heap
    // from 0x10_0000 for both: a large level mesh (obj 4570, 15700 verts) overran the rodata at 0x2a2dc0 /
    // 0x2a6ad0 / 0x2bbaf0 and the chain found no pairs).
    let base = level_image(lvl, lbsp, lc, lr);
    let mut m = LayeredMem { base, over: Default::default(), calls: Vec::new(), route };
    let mut h = Bump(0x10_0000);
    write_rodata(&mut m);
    write_body(&mut m, 0x1000, HS, 0x2800, hc, hr, hull, hbsp, &mut h);
    let _ = crate::engine::classes::q::q_part1::ee_sub_1aa1d8(&mut m, 0x100, 0x1000, 0x3000);
    let bad = std::env::var_os("SLY2_MESH_DBGBAD").is_some() && m.calls.iter().any(|c| c.0 == 0x001a_7c18 && c.1[3] == 1 && c.2[0][0] < -40.0);
    if bad || std::env::var_os("SLY2_MESH_DBG").is_some() {
        for c in &m.calls {
            eprintln!("call {:x} {:x?} {:?}", c.0, c.1, c.2);
        }
    }
    m.calls
        .iter()
        .filter(|c| c.0 == 0x001a_7c18)
        .map(|c| (c.1[1] == HS, c.2[0][0], [c.2[1][0], c.2[1][1], c.2[1][2]], [c.2[2][0], c.2[2][1], c.2[2][2]], c.1[3] as i32))
        .collect()
}

#[cfg(test)]
mod pair_tests {
    use super::*;

    /// hull cube (half 1 at z = 0.8) resting 0.2 into a floor cube (top z = 0): the hull's 4 bottom corners
    /// against the floor top face (gap -0.2, n +z), plus the floor's top corners are outside the hull (none).
    #[test]
    fn mesh_pair_cube_on_floor() {
        let (g, b) = super::e2e::cube([0.0, 0.0, 0.8], 1.0);
        let (fg, fb) = super::e2e::cube([0.0, 0.0, -5.0], 5.0);
        let r = mesh_pair(&g, &b, [0.0, 0.0, 0.8], 1.8, &fg, &fb, [0.0, 0.0, -5.0], 8.7);
        assert_eq!(r.len(), 4, "{r:?}");
        for (a, gap, p, n, kind) in r {
            assert!(a);
            assert_eq!(kind, 1);
            assert!((gap + 0.2).abs() < 1e-5);
            assert!((p[2] + 0.2).abs() < 1e-5);
            assert_eq!(n, [0.0, 0.0, 1.0]);
        }
    }
}
