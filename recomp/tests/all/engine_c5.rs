//! WP-C5 tests: skinned-mesh hookup and ink outline segments on all 43 levels. Needs the ISO at
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

use sly2_recomp::engine::anim::skin_mesh::skin_attributes;
use sly2_recomp::engine::gfx::attach::BoneWorlds;
use sly2_recomp::engine::gfx::ink::*;
use sly2_recomp::engine::gfx::mesh::{ee_vu1_edge_silhouette, MeshRole};
use sly2_recomp::engine::gfx::scene::{mul, LevelScene};
use sly2_recomp::engine::gfx::skin::*;
use sly2_recomp::engine::world::objects::{LoadData, Payload};

#[test]
fn eye_rule_matches_projected_rule() {
    // eye far along +z looking down = orthographic projection onto xy: same result as the VU1 rule
    let eye = [0.3, 0.2, 1.0e6];
    let pts = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.1], [1.0, 1.0, 0.3], [0.5, 0.5, -0.2], [2.0, 0.5, 0.4], [0.0, 2.0, 1.0]];
    for a in pts {
        for d in pts {
            let (b, c) = (pts[1], pts[2]);
            assert_eq!(edge_silhouette_eye(a, b, c, d, eye), ee_vu1_edge_silhouette(a, b, c, d), "{a:?} {d:?}");
        }
    }
}

/// (lsn, skinned actors, skinned prims, joints, ink prims, ink edges, ink segments from above)
type Row = (u32, usize, usize, usize, usize, usize, usize);

#[test]
#[ignore = "heavy"]
fn all_levels_skin_and_ink() {
    let (loader, descs) = levels();
    let mut rows: Vec<Row> = Vec::new();
    for d in descs {
        let (lv, w) = load(&loader, d);
        let sc = LevelScene::build(&lv, &w);
        let mut bw = BoneWorlds::default();
        let mut row: Row = (d.lsn(), 0, 0, 0, 0, 0, 0);
        for (si, so) in sc.objects.iter().enumerate() {
            let Payload::Loaded(LoadData::Alo { alo, .. }) = &w.objects[so.object].payload else { unreachable!() };
            let ow = sc.world_of(si);
            let mut skinned = false;
            for p in &so.prims {
                match p.prim.role {
                    MeshRole::Surface => {
                        let Some(a) = skin_attributes(&p.prim) else { continue };
                        let j = skin_joints(&w, &mut bw, so.object, record_weight_refs(&alo.mesh.records[p.record].data));
                        assert!(skin_valid(&p.prim, &j), "lsn {} obj {} rec {}: invalid joints {:?}", d.lsn(), so.object, p.record, j.refs);
                        assert!(a.joints.iter().flatten().all(|&x| (x as usize) < j.refs.len()));
                        // at rest the skinned prim lands where the rigid one did: joint * ibp == prim local
                        for wj in j.rest_world.iter().flatten() {
                            let jl = joint_local(Some(wj), &ow, &p.local);
                            let back = mul(&inverse_bindpose(&p.local, &jl), &jl);
                            let err = (0..4).flat_map(|r| (0..3).map(move |c| (r, c))).map(|(r, c)| (back[r][c] - p.local[r][c]).abs() / (1.0 + p.local[r][c].abs())).fold(0.0f32, f32::max);
                            assert!(err < 1e-2, "lsn {} rest mismatch {err}", d.lsn());
                        }
                        skinned = true;
                        row.2 += 1;
                        row.3 += j.refs.len();
                    }
                    MeshRole::InkEdges => {
                        row.4 += 1;
                        row.5 += p.prim.edges.len();
                        let mut seg = Vec::new();
                        ink_segments(&p.prim, [0.0, 0.0, 1.0e7], &mut seg);
                        assert!(seg.len() % 2 == 0 && seg.len() / 2 <= p.prim.edges.len());
                        row.6 += seg.len() / 2;
                    }
                    _ => {}
                }
            }
            row.1 += skinned as usize;
        }
        println!("{row:?},");
        rows.push(row);
    }
    let tot = rows.iter().fold([0usize; 6], |t, r| [t[0] + r.1, t[1] + r.2, t[2] + r.3, t[3] + r.4, t[4] + r.5, t[5] + r.6]);
    println!("totals {tot:?}");
    assert_eq!(rows, EXPECT.to_vec());
}

// totals: 381 skinned actors, 18508 skinned prims, 297289 joints, 14216 ink prims,
// 1144849 ink edges, 292045 segments seen from (0,0,1e7) prim-local
const EXPECT: &[Row] = &[
    (335488, 12, 611, 9760, 324, 25703, 6087),
    (350144, 0, 0, 0, 0, 0, 0),
    (353856, 16, 669, 10729, 431, 33050, 8411),
    (377568, 7, 308, 5483, 267, 21714, 5467),
    (388768, 11, 346, 5411, 296, 22901, 5850),
    (402976, 7, 280, 4951, 247, 21152, 5338),
    (409312, 9, 348, 5571, 279, 23408, 6057),
    (417504, 4, 307, 5869, 208, 18585, 4523),
    (427328, 15, 567, 7894, 433, 33666, 8525),
    (444896, 10, 473, 6961, 339, 27720, 7063),
    (453760, 9, 471, 6957, 368, 28139, 7284),
    (465056, 14, 1025, 15621, 590, 44170, 10915),
    (482752, 21, 1075, 14720, 601, 45538, 11335),
    (509056, 9, 400, 6581, 331, 26276, 6718),
    (521344, 14, 538, 8024, 432, 33219, 9313),
    (540160, 8, 385, 6402, 318, 25208, 6647),
    (552768, 3, 230, 4201, 218, 17670, 4394),
    (561312, 15, 679, 10014, 497, 38824, 9945),
    (578848, 7, 306, 5257, 248, 21753, 5453),
    (589920, 11, 692, 10713, 462, 35133, 9030),
    (603648, 7, 328, 6053, 301, 23494, 6025),
    (615360, 6, 280, 5157, 239, 20925, 5249),
    (626688, 6, 280, 5157, 241, 20967, 5259),
    (637920, 6, 280, 5157, 241, 20967, 5259),
    (649056, 6, 280, 5157, 241, 20967, 5259),
    (660320, 6, 280, 5157, 241, 20967, 5259),
    (671392, 14, 737, 10992, 488, 39954, 10427),
    (687488, 8, 383, 6473, 334, 26018, 6627),
    (698912, 8, 459, 7117, 350, 28640, 7316),
    (710528, 11, 578, 8923, 406, 31869, 7875),
    (720608, 10, 491, 7361, 345, 28554, 7280),
    (730464, 14, 618, 8801, 447, 34218, 9122),
    (749312, 7, 286, 4925, 256, 21944, 5473),
    (761664, 6, 284, 4917, 281, 22291, 5663),
    (773120, 6, 333, 5769, 281, 23520, 5860),
    (784544, 5, 274, 4529, 249, 20943, 5254),
    (795296, 7, 341, 5825, 273, 23146, 5751),
    (808192, 12, 486, 8350, 498, 40949, 11448),
    (823200, 10, 463, 7195, 482, 35794, 9485),
    (834976, 4, 232, 4205, 199, 17156, 4142),
    (845728, 7, 312, 5389, 291, 22466, 5493),
    (856896, 4, 232, 4205, 218, 17670, 4394),
    (867648, 9, 561, 9356, 425, 37601, 9770),
];
