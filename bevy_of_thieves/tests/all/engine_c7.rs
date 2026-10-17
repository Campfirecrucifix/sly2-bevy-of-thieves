//! WP-C7 tests: gfx visual parity (per-level fog from the CM property list, env pass, sky,
//! translucent sort, screen effects). Needs the ISO at "<root>/Sly 2 - Band of Thieves (USA).iso"
//! (level list from the boot level table).
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

fn levels() -> (LevelLoader, Vec<Descriptor>) {
    crate::common::levels() // public: ISO level table instead of the analysis DB
}

fn load(loader: &LevelLoader, d: Descriptor) -> (LoadedLevel, World) {
    let lv = crate::common::load_blocking(&loader, d).unwrap();
    let w = World::load(&lv, LoadOptions::default()).unwrap_or_else(|e| panic!("lsn {}: {e}", d.lsn()));
    (lv, w)
}

#[test]
#[ignore]
fn c7_probe_cm_props() {
    let (loader, descs) = levels();
    for (i, d) in descs.into_iter().enumerate() {
        let (_lv, w) = load(&loader, d);
        let f: Vec<String> = w.props_2ddf40.iter().map(|p| format!("{}={:?}", p.field, p.value)).collect();
        println!("t{i} lsn {} cm: {}", d.lsn(), f.join(" "));
    }
}

#[test]
#[ignore]
fn c7_probe_fog_scale() {
    use sly2_recomp::engine::world::objects::{LoadData, Payload};
    let (loader, descs) = levels();
    for (i, d) in descs.into_iter().enumerate() {
        let (_lv, w) = load(&loader, d);
        let mut h: std::collections::BTreeMap<String, usize> = Default::default();
        for o in &w.objects {
            let Payload::Loaded(LoadData::Alo { alo, .. }) = &o.payload else { continue };
            for r in &alo.mesh.records {
                *h.entry(format!("k{} f1={} cls{}", r.kind, r.f1, if r.f1 != 1.0 { o.class as i32 } else { -1 })).or_default() += 1;
            }
        }
        println!("t{i}: {:?}", h);
    }
}

/// Per-level fog: the CM property list sets near / far / max / colour in 42 of 43 levels (t1 keeps
/// the InitCm defaults); every setting is sane (near < far, max in 0..1, colour alpha 0x80).
#[test]
fn c7_level_fog_from_cm_props() {
    use sly2_recomp::engine::gfx::fog::{level_fog, LevelFog};
    let (loader, descs) = levels();
    let mut set = 0;
    for (i, d) in descs.into_iter().enumerate() {
        let (_lv, w) = load(&loader, d);
        let f = level_fog(&w.props_2ddf40);
        if f != LevelFog::default() {
            set += 1;
        }
        assert!(f.fog.start < f.fog.end, "t{i} {f:?}");
        assert!((0.0..=1.0).contains(&f.fog.max), "t{i} {f:?}");
        assert_eq!(f.rgba[3], 0x80, "t{i} {f:?}");
    }
    assert_eq!(set, 42);
}

/// Fog curve (FUN_0014e270 / FUN_001a37d0): none at `start`, `max * scale` at `end`; record
/// scale FLT_MAX = 1, scale 0 = fog off.
#[test]
fn c7_fog_curve_and_record_scale() {
    use sly2_recomp::engine::gfx::fog::*;
    let cm = CmFog { start: 500.0, end: 20000.0, max: 0.5 };
    assert!(fog_amount(1.0, &cm, 500.0).abs() < 1e-3);
    assert!((fog_amount(1.0, &cm, 20000.0) - 0.5).abs() < 1e-3);
    assert!((fog_amount(0.5, &cm, 20000.0) - 0.25).abs() < 1e-3);
    assert_eq!(record_fog_scale(FLT_MAX_SENTINEL), 1.0);
    assert!(fog_enabled_for(record_fog_scale(FLT_MAX_SENTINEL)));
    assert!(!fog_enabled_for(record_fog_scale(0.0)));
}

/// Env-only (kind 3) light volume: unlit tint drawn from the back faces, level material invisible.
#[cfg(feature = "game")]
#[test]
fn c7_env_volume_material() {
    use bevy::prelude::*;
    use sly2_recomp::engine::gfx::envvol::{hide_material, volume_material};
    let mut m = StandardMaterial { base_color: Color::srgb(1.0, 0.5, 0.0), alpha_mode: AlphaMode::Add, ..default() };
    let v = volume_material(&m);
    assert!(v.base.unlit);
    assert_eq!(v.base.cull_mode, Some(bevy::render::render_resource::Face::Front));
    assert_eq!(v.base.alpha_mode, AlphaMode::Add);
    assert_eq!(v.base.base_color, m.base_color);
    hide_material(&mut m);
    assert_eq!(m.base_color.alpha(), 0.0);
    assert_eq!(m.alpha_mode, AlphaMode::Add);
}
