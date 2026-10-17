//! WP-C4 tests: mesh types 1/2/3, live light updates, light calibration. Needs the ISO at
//! "<root>/Sly 2 - Band of Thieves (USA).iso" (level list from the boot level table).
use sly2_recomp::engine::disc::descriptor::Descriptor;
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::{LevelLoader, LoadedLevel};
use sly2_recomp::engine::world::{LoadOptions, World};
use std::path::PathBuf;
use std::sync::Arc;

fn real_root() -> PathBuf {
    let r = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    if r.join("Sly 2 - Band of Thieves (USA).iso").exists() { r } else { PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/..")) }
}

/// The 43 level descriptors (db disc_files + level_table, as in engine_a).
fn levels() -> (LevelLoader, Vec<Descriptor>) {
    crate::common::levels() // public: ISO level table instead of the analysis DB
}

fn load(loader: &LevelLoader, d: Descriptor) -> (LoadedLevel, World) {
    let lv = crate::common::load_blocking(&loader, d).unwrap();
    let w = World::load(&lv, LoadOptions::default()).unwrap_or_else(|e| panic!("lsn {}: {e}", d.lsn()));
    (lv, w)
}

use sly2_recomp::engine::gfx::light::*;
use sly2_recomp::engine::gfx::mesh::*;
use sly2_recomp::engine::gfx::scene::{LevelScene, IDENTITY};
use sly2_recomp::engine::world::alo::MeshData;
use sly2_recomp::engine::world::objects::{LoadData, Payload};
use std::collections::BTreeMap;

#[test]
fn silhouette_rule() {
    let (a, b, c) = ([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]);
    // flat fold, d on the other side of edge (b, c): same facing -> not drawn
    assert!(!ee_vu1_edge_silhouette(a, b, c, [2.0, 0.5, 0.0]));
    // d on the same side as a: faces opposite -> silhouette
    assert!(ee_vu1_edge_silhouette(a, b, c, [0.5, 0.5, 0.0]));
    // boundary edge (a == d) is always a silhouette
    assert!(ee_vu1_edge_silhouette(a, b, c, a));
    // degenerate product 0 -> not drawn
    assert!(!ee_vu1_edge_silhouette(a, b, b, a));
    assert_eq!(edge_faces(&[[0, 1, 2, 3], [1, 2, 0, 3]]), vec![[0, 1, 2], [2, 1, 3], [0, 2, 3]]);
}

#[test]
#[ignore = "heavy"]
fn all_levels_types_123() {
    let (loader, descs) = levels();
    let (mut n, mut edges, mut prims, mut faces) = ([0usize; 4], [0usize; 4], [0usize; 4], [0usize; 4]);
    let mut kinds: BTreeMap<(i8, u8), usize> = BTreeMap::new();
    for d in descs {
        let (lv, w) = load(&loader, d);
        let sc = LevelScene::build(&lv, &w);
        for o in &w.objects {
            let Payload::Loaded(LoadData::Alo { alo, .. }) = &o.payload else { continue };
            for r in &alo.mesh.records {
                let t = r.kind.max(0) as usize;
                let ps = convert(&r.data);
                match &r.data {
                    MeshData::Type1(m) | MeshData::Type2(m) => {
                        n[t] += 1;
                        let role = if t == 1 { MeshRole::InkEdges } else { MeshRole::ShadowVolume };
                        // every strip converts (all indices in range); stored edge count kept
                        assert_eq!(ps.len(), m.strips.len());
                        for (p, s) in ps.iter().zip(&m.strips) {
                            assert_eq!(p.edges.len(), s.v4_8.len());
                            assert_eq!(p.positions.len(), s.positions.len());
                            assert_eq!((p.role, p.cull, p.cull_back), (role, Vu1Cull::Silhouette, false));
                            edges[t] += p.edges.len();
                            if let Some(k) = s.material.and_then(|x| sc.textures[o.entry].shaders.get(x as usize)) {
                                *kinds.entry((r.kind, k.kind)).or_insert(0) += 1;
                            }
                        }
                    }
                    MeshData::Type3(m) => {
                        n[3] += 1;
                        for p in &ps {
                            assert_eq!((p.role, p.cull), (MeshRole::ShadowVolume, Vu1Cull::None));
                        }
                        for s in &m.strips {
                            edges[3] += s.indices.len();
                            if let Some(k) = sc.textures[o.entry].shaders.get(s.material as usize) {
                                *kinds.entry((3, k.kind)).or_insert(0) += 1;
                            }
                        }
                    }
                    MeshData::Type0(m) => {
                        for p in &ps {
                            assert_eq!(p.role, MeshRole::Surface);
                            assert_eq!(p.cull, if vu1_cull(m.flags) { Vu1Cull::Back } else { Vu1Cull::None });
                        }
                    }
                    _ => continue,
                }
                for p in &ps {
                    // Solari-compatible: every attribute per vertex, U32 triangle list
                    let nv = p.positions.len();
                    assert!(p.normals.len() == nv && p.uvs.len() == nv && p.tangents.len() == nv && p.colors.len() == nv);
                    assert!(p.indices.len() % 3 == 0 && p.indices.iter().all(|&i| (i as usize) < nv));
                    prims[t] += 1;
                    faces[t] += p.indices.len() / 3;
                }
            }
        }
    }
    println!("records {n:?} edges/indices {edges:?} prims {prims:?} faces {faces:?} shader kinds {kinds:?}");
    // re-blessed in the merged tree: WP-B4 instantiates more objects (C4 counted on the pre-B4 world)
    assert_eq!(&n[1..], &[9325, 1082, 2208]);
    assert_eq!(&edges[1..], &[1139474, 94697, 58250]);
    assert_eq!(&prims[1..], &[14062, 6655, 2208]);
    // types 2/3 only ever reference shadow shaders (kind 2)
    assert!(kinds.keys().filter(|k| k.0 >= 2).all(|k| k.1 == 2));
}

fn vu1_cull(flags: u8) -> bool {
    sly2_recomp::engine::gfx::material::vu1_cull_back(flags)
}

#[test]
fn light_setters_rederive() {
    let mut p = ee_sub_1a3360();
    assert!(set_field(&mut p, 151, &1u32.to_le_bytes()));
    let hsv: Vec<u8> = [0.0f32, 0.0, 255.0].iter().flat_map(|f| f.to_le_bytes()).collect();
    assert!(set_field(&mut p, 152, &hsv));
    let fall: Vec<u8> = [10.0f32, 110.0].iter().flat_map(|f| f.to_le_bytes()).collect();
    assert!(set_field(&mut p, 155, &fall));
    assert!(!set_field(&mut p, 12, &[0]));
    let mut m = IDENTITY;
    m[3] = [5.0, 6.0, 7.0];
    let d = ee_sub_1a3968(&p, &m, false);
    assert_eq!(d.vec, [5.0, 6.0, 7.0]);
    assert!((d.du_fall - 255.0 * 1.1).abs() < 1e-3 && (d.ru_fall + 255.0 / 100.0).abs() < 1e-4);
    // 157/158 mutually exclusive
    set_field(&mut p, 158, &[1]);
    set_field(&mut p, 157, &[1]);
    assert!(p.excluded && !p.dynamic);
}

#[test]
fn light_unit_from_exposure() {
    let e = ps2_light_unit_lux(BEVY_DEFAULT_EV100);
    assert!((e - 3135.6).abs() < 1.0, "{e}");
    // Lambert: albedo * E / PI * 2^-ev100 / 1.2 == albedo for one PS2 unit
    assert!((e / std::f32::consts::PI * (-BEVY_DEFAULT_EV100).exp2() / 1.2 - 1.0).abs() < 1e-5);
}
