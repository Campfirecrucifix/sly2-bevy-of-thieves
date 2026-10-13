//! WP-EP1c: VERIFY.md blockers. Expected values derived from the decompiled code (comments).
use sly2_recomp::engine::camera::cm::*;
use sly2_recomp::engine::camera::follow::*;
use sly2_recomp::engine::camera::math::*;
use sly2_recomp::engine::camera::policy::*;
use sly2_recomp::engine::camera::CmClock;

const DT: f32 = 1.0 / 60.0;

/// Issue 1: a class-15 track whose params +4 distance is 0 (rows 15 22 32 38 at spawn). 0x00221dd8 tail
/// (dist + near) / dist divides by 0; the PS2 FPU yields FMAX (no NaN) and the offset d is 0 -> campos = focus.
#[test]
fn ep1c_follow_zero_distance_stays_finite() {
    struct W;
    impl CmWorld for W {
        fn get_cpdefi(&mut self, _pso: u32) -> Cpdefi {
            Cpdefi { pos_base: v4([100.0, 200.0, 300.0], 1.0), ..Default::default() }
        }
        fn jt_state(&mut self, _pso: u32) -> Option<i32> {
            None
        }
    }
    let mut cm = Cm::new();
    let mut f = CplcyFollow::new(Box::new(NoFollowWorld));
    f.params.s_dist = 0.0;
    let id = add_policy(&mut cm, Box::new(f));
    cm.cpdefi = W.get_cpdefi(1);
    cm.pso_cpdefi = 1;
    assert!(FUN_0014e328(&mut cm, 1, id, 1, 0, CmClock { t: 1.0, dt: DT, dt_real: DT }));
    for i in 0..120 {
        let c = CmClock { t: 1.0 + DT * (i + 1) as f32, dt: DT, dt_real: DT };
        FUN_0014d478(&mut cm, &mut W, &CplcyCtx::default(), c);
        let p = cm.pos();
        assert!(p.iter().all(|x| x.is_finite()), "frame {i}: {p:?}");
        assert!(cm.mat.iter().flatten().all(|x| x.is_finite()), "frame {i}: {:?}", cm.mat);
    }
}

/// Issue 2: the death reload (0x161a58 case 0x10 -> FUN_0022b490 -> FUN_0020e890) builds a fresh JT at the
/// spawn exactly once per reload; a re-fed `PlayerLevel` (same level, new load) must replace the dying JT,
/// otherwise the old jts-4 JT dies again and the game reloads forever.
#[test]
fn ep1c_reload_replaces_dying_jt() {
    use bevy_app::App;
    use sly2_recomp::engine::physics::CollisionWorld;
    use sly2_recomp::engine::player::p7_damage::HP_MAX;
    use sly2_recomp::engine::player::plugin::{Player, PlayerLevel, PlayerPlugin};
    use std::sync::Arc;
    let mut app = App::new();
    app.add_plugins(PlayerPlugin);
    app.insert_resource(PlayerLevel { cw: Arc::new(CollisionWorld::default()), spawn: Some([0.0, 0.0, 0.0]) });
    app.update();
    {
        let mut q = app.world_mut().query::<&mut Player>();
        let mut p = q.single_mut(app.world_mut()).expect("one player");
        p.jt.dmg.hp = 0;
        p.jt.jts = 4;
        p.jt.phys.pos = [5000.0, 0.0, -9000.0];
    }
    // same level reloaded: the game layer re-feeds PlayerLevel
    app.insert_resource(PlayerLevel { cw: Arc::new(CollisionWorld::default()), spawn: Some([10.0, 20.0, 30.0]) });
    app.update();
    let mut q = app.world_mut().query::<&Player>();
    let ps: Vec<_> = q.iter(app.world()).collect();
    assert_eq!(ps.len(), 1, "one player after the reload");
    let jt = &ps[0].jt;
    assert_ne!(jt.jts, 4);
    // 0x1440d0 on JT init refills a 0 hp
    assert_eq!(jt.dmg.hp, HP_MAX);
    assert!((jt.phys.pos[0] - 10.0).abs() < 1.0 && (jt.phys.pos[1] - 20.0).abs() < 1.0, "{:?}", jt.phys.pos);
    // no further reload: the plugin does not respawn again on an unchanged level
    app.update();
    let mut q = app.world_mut().query::<&Player>();
    assert_eq!(q.iter(app.world()).count(), 1);
}

const ISO: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../Sly 2 - Band of Thieves (USA).iso");

/// Boot level (static row 25) level-table rows (None without the ISO).
fn level_rows() -> Option<(sly2_recomp::engine::disc::LevelLoader, Vec<sly2_recomp::engine::disc::descriptor::Descriptor>)> {
    use sly2_recomp::engine::disc::{descriptor, iso::IsoImage, mods::ModOverlay, LevelLoader};
    let iso = std::sync::Arc::new(IsoImage::open(ISO).ok()?);
    let table = descriptor::static_table(&iso.read_file("SCUS_973.16").ok()?)?;
    let l = LevelLoader::new(iso, ModOverlay::default());
    let rows = l.load_blocking(table[25]).ok()?.header.level_table.entries.iter().map(|e| e.desc).collect();
    Some((l, rows))
}

/// Issue 3: rows 16 / 38 spawn at the origin. Row 38's default spawn 0x189 (obj 2329) and row 16's JT 465 /
/// only spawn 77108 have identity load transforms (placeholders); row 38 has task spawns (debug entry).
#[test]
fn ep1c_origin_placeholder_spawns() {
    use sly2_recomp::engine::game::spawn::*;
    use sly2_recomp::engine::world::{LoadOptions, World};
    let Some((l, rows)) = level_rows() else { return eprintln!("ISO not found; skipped") };
    let w = World::load(&l.load_blocking(rows[38]).unwrap(), LoadOptions::default()).unwrap();
    let al = SpawnAliases::default();
    let s = ee_sub_227fe8(&w, &al, 0x189, -1).expect("row 38 has spawn 0x189");
    assert!(at_origin(&w, s));
    let (t, oid) = debug_entry_spawn(&w, &al, -1).expect("row 38 task spawn");
    assert!(!at_origin(&w, t), "task spawn {oid} at {:?}", spawn_position(&w, t));
    let w = World::load(&l.load_blocking(rows[16]).unwrap(), LoadOptions::default()).unwrap();
    assert_eq!(ee_sub_227fe8(&w, &al, 0x189, -1), None);
    assert!(player_object(&w).is_some_and(|j| at_origin(&w, j)));
    assert_eq!(debug_entry_spawn(&w, &al, -1), None);
    // an ordinary level: the Ep1 hub's default spawn is placed
    let w = World::load(&l.load_blocking(rows[2]).unwrap(), LoadOptions::default()).unwrap();
    assert!(ee_sub_227fe8(&w, &al, 0x189, -1).is_some_and(|s| !at_origin(&w, s)));
}

/// Issue 9: `--table 1` (menu level abandon_icon: no JT, no spawn) is refused; levels still pass.
#[test]
fn ep1c_menu_level_rejected() {
    use sly2_recomp::engine::game::level_check::{check_level_row, NotALevel};
    let Some((l, rows)) = level_rows() else { return eprintln!("ISO not found; skipped") };
    assert!(matches!(check_level_row(&l, rows[1]), Err(NotALevel::NoPlayer { .. })), "{:?}", check_level_row(&l, rows[1]));
    for r in [0, 2, 16, 38] {
        assert_eq!(check_level_row(&l, rows[r]), Ok(()), "row {r}");
    }
}
