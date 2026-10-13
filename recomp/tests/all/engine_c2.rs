//! WP-C2 tests: VU1 cull flag counts over all 43 levels, fog/colour path unit tests. Needs the ISO at
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



use sly2_recomp::engine::gfx::fog::*;
use sly2_recomp::engine::gfx::material::vu1_cull_back;
use sly2_recomp::engine::gfx::mesh::convert;
use sly2_recomp::engine::world::alo::MeshData;
use sly2_recomp::engine::world::objects::{LoadData, Payload};

#[test]
fn cull_flag_from_code() {
    // FUN_0016a000: flags & 0x10 -> VU1 header |= 0x80 (no back-face test)
    assert!(vu1_cull_back(0));
    assert!(vu1_cull_back(0xef));
    assert!(!vu1_cull_back(0x10));
}

#[test]
#[ignore = "heavy"]
fn all_levels_cull_counts() {
    let (loader, descs) = levels();
    let (mut culled, mut double, mut prims_c, mut prims_d) = (0usize, 0usize, 0usize, 0usize);
    for d in descs {
        let (_lv, w) = load(&loader, d);
        for o in &w.objects {
            let Payload::Loaded(LoadData::Alo { alo, .. }) = &o.payload else { continue };
            for r in &alo.mesh.records {
                if let MeshData::Type0(m) = &r.data {
                    if m.flags & 0x10 == 0 { culled += 1 } else { double += 1 }
                    for p in convert(&r.data) {
                        assert_eq!(p.cull_back, vu1_cull_back(m.flags));
                        if p.cull_back { prims_c += 1 } else { prims_d += 1 }
                    }
                }
            }
        }
    }
    println!("type0 records culled {culled} double {double}; prims {prims_c} / {prims_d}");
    // WP-B4: + meshes of objects created by loads_b4 (was 33956, 1759)
    assert_eq!((culled, double), (38498, 2473)); // D2: +ALO-wrapper actors (was 24565, 1128)
    assert!(prims_c > 0 && prims_d > 0);
}

#[test]
fn fog_curve_from_code() {
    let cm = CmFog { start: 100.0, end: 1000.0, max: 0.75 };
    let c = ee_sub_14e270(f32::MAX, &cm); // FLT_MAX -> scale 1
    assert_eq!(c[2], 100.0);
    assert_eq!(c[3], c[0] + 8192.0);
    assert!((gs_fog_value(&c, 100.0) - 4080.0).abs() < 0.5);
    assert!((gs_fog_value(&c, 1000.0) - 0.25 * 4080.0).abs() < 0.5);
    assert!((fog_amount(1.0, &cm, 1000.0) - 0.75).abs() < 1e-3);
    assert_eq!(fog_amount(1.0, &cm, 50.0), 0.0);
    assert!(fog_amount(1.0, &cm, 300.0) > (300.0 - 100.0) / 900.0 * 0.75); // hyperbolic in 1/w
    // degenerate: start == end -> constant 4080
    assert_eq!(ee_sub_1a37d0(1.0, 4080.0, 1.0, 0.0), (4080.0, 0.0));
}

#[test]
fn fog_colour_from_code() {
    assert_eq!(ee_sub_1ed278([10, 20, 30, 0x80], false), [10, 20, 30, 0x80]);
    let g = ee_sub_1ed278([255, 255, 255, 7], true);
    assert_eq!(g, [255, 255, 255, 7]);
    let g = ee_sub_1ed278([255, 0, 0, 1], true); // L = 0.3, v = 0.3^0.6
    let v = (0.3f32.powf(0.6) * 255.0 + 0.5) as u8;
    assert_eq!(g, [v, v, v, 1]);
    assert_eq!(ee_sub_11dd90([0.0, 1.0, 1.0]), [1.0, 0.0, 0.0]);
    assert_eq!(ee_sub_11dd90([120.0, 1.0, 1.0]), [0.0, 1.0, 0.0]);
}
