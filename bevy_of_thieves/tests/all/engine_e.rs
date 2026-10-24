//! WP-E tests: collision derived data (FUN_00117fd0/FUN_00119308/FUN_00227bb8), BSP clip (0x00118668),
//! LSG merge (0x0014c9f8), camera probe through CmSegmentQuery. Level part needs the ISO + db (as engine_b3).
use sly2_recomp::engine::camera::collide::{CmSegmentQuery, FUN_0014ea08, FUN_0014eb70};
use sly2_recomp::engine::disc::descriptor::Descriptor;
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::physics::bsp::{FUN_00118668, FUN_0014c9f8, Lsg};
use sly2_recomp::engine::physics::geom::*;
use sly2_recomp::engine::physics::CollisionWorld;
use sly2_recomp::engine::world::loads::{Bsp, BspNode, Geom, Surface};
use sly2_recomp::engine::world::{LoadOptions, World};
use std::path::PathBuf;
use std::sync::Arc;

fn real_root() -> PathBuf {
    let r = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    if r.join("Sly 2 - Band of Thieves (USA).iso").exists() { r } else { PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/..")) }
}

fn levels() -> (LevelLoader, Vec<Descriptor>) {
    crate::common::levels() // public: ISO level table instead of the analysis DB
}

/// unit cube [0,1]^3 as 6 planes in a BSP chain: neg child = next plane, last neg = solid leaf.
fn cube() -> (CGeom, BspC) {
    let verts = vec![
        [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0, 0.0],
        [0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [0.0, 1.0, 1.0], [1.0, 1.0, 1.0],
    ];
    // outward normals (cross((p1-p0),(p2-p0)))
    let tris: [[u16; 3]; 6] = [[1, 3, 5], [0, 4, 2], [2, 6, 3], [0, 1, 4], [4, 5, 6], [0, 2, 1]];
    let surfaces = tris.iter().map(|&v| Surface { verts: v, discarded: [0.0; 4], edges: vec![] }).collect();
    let g = Geom { verts, surfaces, edge_pool: 0 };
    let mut nodes: Vec<BspNode> = (0..6).map(|i| BspNode { surface: i as u16, children: Some([i + 1, -1]) }).collect();
    nodes.push(BspNode { surface: 0xffff, children: None });
    let b = Bsp { a: 7, b: 0, nodes };
    (build_geom(&g), build_bsp(&b))
}

#[test]
fn plane_and_sphere() {
    // 0x00119308: (1,0,0)x(0,1,0) = +z, w = n.p0
    let p = FUN_00119308([0.0, 0.0, 2.0], [1.0, 0.0, 2.0], [0.0, 1.0, 2.0]);
    assert_eq!(p, [0.0, 0.0, 1.0, 2.0]);
    let (g, _) = cube();
    assert_eq!(g.surfs[0].plane, [1.0, 0.0, 0.0, 1.0]);
    assert_eq!(g.surfs[1].plane, [-1.0, 0.0, 0.0, 0.0]);
    // FUN_00227bb8: centre = .5 min + .5 max, +0x120 = |c - min|^2 (no sqrt in the listing)
    let (c, r) = bounding_sphere_227bb8(&g);
    assert_eq!(c, [0.5, 0.5, 0.5]);
    assert_eq!(r, 0.75);
    assert_eq!(bounding_sphere_227bb8(&CGeom { verts: vec![], surfs: vec![], edges: vec![] }).1, -f32::MAX);
}

#[test]
fn bsp_clip_and_merge() {
    let (g, b) = cube();
    let mut out = Vec::new();
    let n = FUN_00118668(&g, &b, 0, 0x7fffffff, [-1.0, 0.5, 0.5], [3.0, 0.5, 0.5], 8, &mut out);
    assert_eq!(n, 1);
    assert_eq!(out[0].t, [0.25, 0.5]);
    assert_eq!(out[0].pos, [[0.0, 0.5, 0.5], [1.0, 0.5, 0.5]]);
    assert_eq!(out[0].surf, [Some(1), Some(0)]);
    // start inside: enter plane zero, t0 = 0
    out.clear();
    assert_eq!(FUN_00118668(&g, &b, 0, 7, [0.5, 0.5, 0.5], [0.5, 0.5, 3.5], 8, &mut out), 1);
    assert_eq!((out[0].t[0], out[0].surf[0], out[0].t[1]), (0.0, None, 1.0 / 6.0));
    // miss
    out.clear();
    assert_eq!(FUN_00118668(&g, &b, 0, 7, [-1.0, 2.0, 0.5], [3.0, 2.0, 0.5], 8, &mut out), 0);
    // max reached -> returns max
    assert_eq!(FUN_00118668(&g, &b, 0, 7, [-1.0, 0.5, 0.5], [3.0, 0.5, 0.5], 0, &mut out), 0);
    // union / intersection
    let l = |a: f32, z: f32| Lsg { t: [a, z], ..Default::default() };
    let u = FUN_0014c9f8(&[l(0.1, 0.3), l(0.6, 0.7)], &[l(0.2, 0.4)], 0x40, 0);
    assert_eq!(u.iter().map(|x| x.t).collect::<Vec<_>>(), vec![[0.1, 0.4], [0.6, 0.7]]);
    let i = FUN_0014c9f8(&[l(0.1, 0.3), l(0.6, 0.7)], &[l(0.2, 0.4)], 0x40, 1);
    assert_eq!(i.iter().map(|x| x.t).collect::<Vec<_>>(), vec![[0.2, 0.3]]);
}

#[test]
fn camera_probe() {
    let (g, b) = cube();
    let mut cw = CollisionWorld {
        colliders: vec![sly2_recomp::engine::physics::query::Collider {
            object: 0,
            source: sly2_recomp::engine::physics::query::Source::Solid,
            geom: g,
            bsp: b,
            sphere: None,
            bbox: None,
        }],
    };
    assert_eq!(cw.solid_intervals([-1.0, 0.5, 0.5], [3.0, 0.5, 0.5]), vec![(0.25, 0.5)]);
    // 0x0014ea08: first gap > 0.2 is [0, 0.25]
    assert_eq!(FUN_0014ea08(0.2, &mut cw, [-1.0, 0.5, 0.5], [3.0, 0.5, 0.5]), Some((0.0, 0.25)));
    let p = FUN_0014eb70(10.0, &mut cw, [-1.0, 0.5, 0.5], [9.0; 3]);
    assert!(p.iter().all(|v| v.is_finite()));
}

/// (lsn, colliders, surfaces, BSP nodes) from CollisionWorld::build (Solid / FUN_00227380 / ALO parts).
const EXPECT: &[(u32, usize, usize, usize)] = &[
    (335488, 229, 26942, 79440),
    (350144, 8, 48, 48),
    (353856, 267, 25616, 78196),
    (377568, 152, 11906, 32338),
    (388768, 166, 14225, 41149),
    (402976, 122, 8223, 22844),
    (409312, 123, 3843, 9125),
    (417504, 138, 8666, 23724),
    (427328, 276, 39780, 127782),
    (444896, 163, 16771, 60788),
    (453760, 147, 5944, 15902),
    (465056, 130, 1775, 2448),
    (482752, 281, 36218, 97437),
    (509056, 193, 15244, 45519),
    (521344, 301, 36452, 121482),
    (540160, 212, 17859, 52330),
    (552768, 75, 1539, 3117),
    (561312, 317, 32446, 96212),
    (578848, 88, 3648, 8858),
    (589920, 199, 10278, 27626),
    (603648, 119, 5031, 12896),
    (615360, 99, 4552, 12132),
    (626688, 99, 2231, 4945),
    (637920, 123, 2643, 5705),
    (649056, 100, 3097, 7763),
    (660320, 99, 2468, 5069),
    (671392, 229, 25394, 70678),
    (687488, 119, 4169, 11040),
    (698912, 160, 9569, 28004),
    (710528, 175, 10374, 32850),
    (720608, 130, 10100, 30230),
    (730464, 278, 39600, 112438),
    (749312, 123, 6355, 18605),
    (761664, 121, 7142, 21969),
    (773120, 126, 5169, 18607),
    (784544, 91, 2828, 5560),
    (795296, 107, 5980, 19184),
    (808192, 267, 22654, 64101),
    (823200, 169, 4076, 7148),
    (834976, 95, 3097, 7466),
    (845728, 122, 3247, 7187),
    (856896, 94, 2867, 7103),
    (867648, 266, 11748, 22058),
];

#[test]
#[ignore = "heavy"]
fn all_levels_collision() {
    let (loader, descs) = levels();
    assert_eq!(descs.len(), 43);
    let mut total = (0usize, 0usize, 0usize);
    let mut hits = 0usize;
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = World::load(&lv, LoadOptions::default()).unwrap();
        let cw = CollisionWorld::build(&w);
        let surfs: usize = cw.colliders.iter().map(|c| c.geom.surfs.len()).sum();
        let nodes: usize = cw.colliders.iter().map(|c| c.bsp.nodes.len()).sum();
        assert!(EXPECT.contains(&(d.lsn(), cw.colliders.len(), surfs, nodes)), "lsn {} {} {} {}", d.lsn(), cw.colliders.len(), surfs, nodes);
        total = (total.0 + cw.colliders.len(), total.1 + surfs, total.2 + nodes);
        for c in &cw.colliders {
            for s in &c.geom.surfs {
                assert!(s.plane.iter().all(|v| v.is_finite()) || c.geom.verts.is_empty(), "lsn {} non-finite plane", d.lsn());
            }
            // rays through the bounding box centre along 3 axes: hits finite, ordered, endpoints on the segment
            if c.geom.verts.is_empty() || c.bsp.nodes.is_empty() {
                continue;
            }
            let mn = FUN_0011f288(&c.geom.verts);
            let mx = FUN_0011f2b8(&c.geom.verts);
            let ce = [(mn[0] + mx[0]) * 0.5, (mn[1] + mx[1]) * 0.5, (mn[2] + mx[2]) * 0.5];
            for ax in 0..3 {
                let (mut p1, mut p2) = (ce, ce);
                p1[ax] = mn[ax] - 10.0;
                p2[ax] = mx[ax] + 10.0;
                let mut out = Vec::new();
                let n = FUN_00118668(&c.geom, &c.bsp, 0, c.bsp.nodes.len(), p1, p2, 0x40, &mut out);
                let mut tprev = 0.0f32;
                for l in &out[..n] {
                    assert!(l.t[0].is_finite() && l.t[1].is_finite());
                    assert!(tprev <= l.t[0] && l.t[0] <= l.t[1] && l.t[1] <= 1.0, "lsn {} {:?}", d.lsn(), l.t);
                    tprev = l.t[1];
                    hits += 1;
                }
            }
        }
    }
    // WP-B4: + colliders of objects created by loads_b4 (class-6 nested lists, 53/54 ALOs) (was 5386, 497576, 1464865)
    assert_eq!(total, (6898, 511814, 1479103)); // D2: +ALO-wrapper actor parts (was 3638, 475683, 1438249)
    assert!(hits > 1000, "hits {hits}");
}
