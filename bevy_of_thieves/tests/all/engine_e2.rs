//! WP-E2 tests: point-in-BSP (0x00118fb0 / 0x00119090) vs segment clip (0x00118668), sphere prune
//! (0x00118a88), point-cloud surface collection (0x00118d08); expectations derived from the listings.
use sly2_recomp::engine::physics::bsp::FUN_00118668;
use sly2_recomp::engine::physics::geom::*;
use sly2_recomp::engine::physics::point::*;
use sly2_recomp::engine::world::loads::{Bsp, BspNode, Geom, Surface};

/// unit cube [0,1]^3: 6 outward planes chained on the negative side, last = solid leaf (as engine_e.rs).
fn cube() -> (CGeom, BspC) {
    let verts = vec![
        [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0, 0.0],
        [0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [0.0, 1.0, 1.0], [1.0, 1.0, 1.0],
    ];
    let tris: [[u16; 3]; 6] = [[1, 3, 5], [0, 4, 2], [2, 6, 3], [0, 1, 4], [4, 5, 6], [0, 2, 1]];
    let surfaces = tris.iter().map(|&v| Surface { verts: v, discarded: [0.0; 4], edges: vec![] }).collect();
    let g = Geom { verts, surfaces, edge_pool: 0 };
    let mut nodes: Vec<BspNode> = (0..6).map(|i| BspNode { surface: i as u16, children: Some([i + 1, -1]) }).collect();
    nodes.push(BspNode { surface: 0xffff, children: None });
    (build_geom(&g), build_bsp(&Bsp { a: 7, b: 0, nodes }))
}

#[test]
fn point_in_bsp() {
    let (g, b) = cube();
    // inside: every plane d < 1e-4 -> chain to node 6 (DAT_002bbaf0 plane: d = -1e8, neg child null)
    assert_eq!(FUN_00118fb0(&g, &b, 0, 0x7fffffff, [0.5, 0.5, 0.5]), Some(6));
    assert_eq!(FUN_00118fb0(&g, &b, 0, 0x7fffffff, [1.5, 0.5, 0.5]), None);
    // on the face x=1 (d = 0 < 1e-4) counts as inside
    assert_eq!(FUN_00118fb0(&g, &b, 0, 0x7fffffff, [1.0, 0.5, 0.5]), Some(6));
    // limit: node 6 cut off -> node 5's neg child null -> node 5 returned
    assert_eq!(FUN_00118fb0(&g, &b, 0, 6, [0.5, 0.5, 0.5]), Some(5));
    assert_eq!(FUN_00119090(&g, &b, 0, 0x7fffffff, [0.5, 0.5, 0.5]), Some(6));
    assert_eq!(FUN_00119090(&g, &b, 0, 0x7fffffff, [1.00005, 0.5, 0.5]), Some(6));
    assert_eq!(FUN_00119090(&g, &b, 0, 0x7fffffff, [1.1, 0.5, 0.5]), None);
}

#[test]
fn point_vs_clip_consistency() {
    let (g, b) = cube();
    let (p1, p2) = ([-1.0, 0.3, 0.7], [3.0, 0.6, 0.2]);
    let mut out = Vec::new();
    let n = FUN_00118668(&g, &b, 0, 0x7fffffff, p1, p2, 8, &mut out);
    for i in 0..=100 {
        let t = i as f32 / 100.0;
        let p = [p1[0] + (p2[0] - p1[0]) * t, p1[1] + (p2[1] - p1[1]) * t, p1[2] + (p2[2] - p1[2]) * t];
        let near = out[..n].iter().any(|l| (t - l.t[0]).abs() < 0.02 || (t - l.t[1]).abs() < 0.02);
        if near {
            continue;
        }
        let clip = out[..n].iter().any(|l| l.t[0] < t && t < l.t[1]);
        assert_eq!(FUN_00118fb0(&g, &b, 0, 0x7fffffff, p).is_some(), clip, "t={t}");
    }
}

#[test]
fn sphere_prune() {
    let (g, b) = cube();
    let mut left = 16;
    let mut out = Vec::new();
    // fully inside: every node d < -r -> chain to the solid leaf, whose neg child is null -> 0 solid
    assert_eq!(FUN_00118a88(&g, &b, 0, 0x7fffffff, [0.5, 0.5, 0.5], 0.1, &mut left, &mut out), SOLID);
    assert!(out.is_empty());
    // fully outside past x=1: d > r -> positive child null -> 1 empty
    assert_eq!(FUN_00118a88(&g, &b, 0, 0x7fffffff, [3.0, 0.5, 0.5], 0.1, &mut left, &mut out), EMPTY);
    // straddling x=1 only: one pruned node (neg = solid subtree -> Null, pos = empty -> Null)
    assert_eq!(FUN_00118a88(&g, &b, 0, 0x7fffffff, [1.0, 0.5, 0.5], 0.1, &mut left, &mut out), MIXED);
    assert_eq!(out, vec![PNode { child: [PChild::Null, PChild::Null], surf: Some(0) }]);
    assert_eq!(left, 15);
    // corner: three planes straddled -> 3 nodes, children first
    out.clear();
    assert_eq!(FUN_00118a88(&g, &b, 0, 0x7fffffff, [1.0, 1.0, 1.0], 0.1, &mut left, &mut out), MIXED);
    assert_eq!(out.len(), 3);
    assert_eq!(out[2].surf, Some(0));
    // budget exhausted on a mixed node -> 1
    let mut zero = 0;
    assert_eq!(FUN_00118a88(&g, &b, 0, 0x7fffffff, [1.0, 0.5, 0.5], 0.1, &mut zero, &mut out), EMPTY);
}

#[test]
fn point_cloud_surfaces() {
    let (g, b) = cube();
    let mut marked = Vec::new();
    let mut out = Vec::new();
    let mut left = 4;
    let box_ = [[0.9, 0.4, 0.4], [1.1, 0.6, 0.6], [0.9, 0.6, 0.4]];
    assert_eq!(FUN_00118d08(&g, &b, 0, 0x7fffffff, &box_, &mut left, &mut marked, &mut out), MIXED);
    assert_eq!(out, vec![0]);
    // second call: surface already flagged (+0x18 & 0x4000) -> not collected again
    assert_eq!(FUN_00118d08(&g, &b, 0, 0x7fffffff, &box_, &mut left, &mut marked, &mut out), MIXED);
    assert_eq!(out, vec![0]);
    let inside = [[0.5, 0.5, 0.5], [0.6, 0.5, 0.5]];
    assert_eq!(FUN_00118d08(&g, &b, 0, 0x7fffffff, &inside, &mut left, &mut marked, &mut out), SOLID);
}
