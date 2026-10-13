//! WP-K2 tests: ClassSet fed from level data + ClassesPlugin over all 43 levels. Needs the ISO at
//! "<root>/Sly 2 - Band of Thieves (USA).iso" (level list from the boot level table).
use sly2_recomp::engine::disc::descriptor::Descriptor;
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::LevelLoader;
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

use sly2_recomp::engine::classes::plugin::{Classes, ClassesLevel, ClassesPlugin, EngineHost};
use sly2_recomp::engine::classes::{ClassSet, Inst};
use sly2_recomp::engine::physics::driven::GameRng;

#[test]
#[ignore = "heavy"]
fn all_levels_build_init_120_frames() {
    let (loader, descs) = levels();
    let (mut fed33, mut n33, mut emits, mut sounds, mut parts) = (0, 0, 0, 0, 0i64);
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let mut app = bevy_app::App::new();
        app.add_plugins(ClassesPlugin);
        app.world_mut().resource_mut::<ClassesLevel>().0 = Some(w.clone());
        for _ in 0..120 {
            app.update();
        }
        let c = app.world().resource::<Classes>();
        assert_eq!(c.frames, 120);
        let n = w.objects.iter().filter(|o| [10u16, 33, 43, 48, 124].contains(&o.class)).count();
        assert_eq!(c.set.insts.len(), n);
        // slot 51 with a default request (flags 0: random count from +0xf4/+0xf8)
        let mut set: ClassSet = c.set.clone();
        let mut h = EngineHost::default();
        let mut rng = GameRng::default();
        set.emit_all(&[0; 15], &mut rng, &mut h);
        for (_, x) in &set.insts {
            if let Inst::C2ce4d0(k) = x {
                n33 += 1;
                if k.f_f8 > 0.0 && k.tpl.w.iter().any(|&v| v != 0) {
                    fed33 += 1;
                }
            }
        }
        emits += h.emits;
        sounds += h.sounds;
        parts += h.particles;
    }
    eprintln!("class33 {n33} fed {fed33} emits {emits} particles {parts} sounds {sounds}");
    assert!(n33 > 0 && fed33 * 2 > n33, "most class-33 objects get +0xf4 range and template fields");
    assert_eq!(emits, n33);
    assert!(parts > 0 && sounds > 0);
}

#[test]
fn slot51_request_flags() {
    use sly2_recomp::engine::classes::c2ce4d0::C2ce4d0;
    let mut c = C2ce4d0 { f_f4: 10.0, f_f8: 10.0, ..Default::default() };
    let mut h = EngineHost::default();
    let mut rng = GameRng::default();
    let mut r = [0u32; 15];
    r[0] = 0x80;
    r[14] = 7;
    c.ee_sub_15c690(3, &r, &mut rng, &mut h);
    assert_eq!((h.emits, h.particles, h.sounds), (1, 7, 0));
    // flag 8 with t[1] == 0: count scaled by r[2] (< 1 -> multiply), floored at f_f8 / 2
    r[0] = 0x88;
    r[2] = 0.5f32.to_bits();
    r[14] = 40;
    c.ee_sub_15c690(3, &r, &mut rng, &mut h);
    assert_eq!(h.particles, 7 + 20);
    r[14] = 4;
    c.f_f0 = Some([1, 2, 3, 4, 5]);
    c.ee_sub_15c690(3, &r, &mut rng, &mut h);
    assert_eq!((h.particles, h.sounds), (7 + 20 + 5, 1));
    // random count: lo == hi -> lo, no draw
    r[0] = 0;
    c.ee_sub_15c690(3, &r, &mut rng, &mut h);
    assert_eq!(h.particles, 7 + 20 + 5 + 10);
}
