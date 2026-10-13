//! WP-P25 tests: FUN_00228750 spawn activation on real levels (player::p25), spawn list adders.

use sly2_recomp::engine::camera::Cm;
use sly2_recomp::engine::classes::plugin::EngineHost;
use sly2_recomp::engine::classes::{ClassSet, Inst};
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::game::spawn::{ee_sub_227fe8, SpawnAliases};
use sly2_recomp::engine::player::p25::{self, SpawnLists};
use sly2_recomp::engine::world::{LoadOptions, World};
use std::sync::Arc;

fn tables() -> (LevelLoader, Vec<sly2_recomp::engine::disc::descriptor::Descriptor>) {
    let iso = Arc::new(IsoImage::open(concat!(env!("CARGO_MANIFEST_DIR"), "/../Sly 2 - Band of Thieves (USA).iso")).expect("ISO"));
    let t = sly2_recomp::engine::disc::descriptor::static_table(&iso.read_file("SCUS_973.16").unwrap()).unwrap();
    let l = LevelLoader::new(iso, ModOverlay::default());
    let boot = crate::common::load_blocking(&l, t[25]).unwrap();
    let d = boot.header.level_table.entries.iter().map(|e| e.desc).collect();
    (l, d)
}

#[test]
fn p25_spawn_lists_adders() {
    let mut l = SpawnLists::default();
    for i in 0..6 {
        l.FUN_00228958(3, i, i + 100);
        l.FUN_00228920(3, i, 1);
    }
    assert_eq!(l.pairs[&3].len(), 4);
    assert_eq!(l.tasks[&3].len(), 4);
    // FUN_0020fec0: flag 0 = no-op; existing {oid, sub} updated; count + 1 < 4 -> only 3 records
    assert!(!l.FUN_002288d8(3, 0, 7, 1, 5));
    assert!(l.FUN_002288d8(3, 1, 7, 1, 5));
    assert!(l.FUN_002288d8(3, 1, 7, 1, 6));
    assert_eq!(l.recs[&3], vec![[7, 1, 6, -1]]);
    assert!(l.FUN_002288d8(3, 1, 8, -1, 2));
    assert!(l.FUN_002288d8(3, 1, 9, -1, 2));
    assert!(!l.FUN_002288d8(3, 1, 10, -1, 2));
    assert_eq!(l.recs[&3].len(), 3);
}

#[test]
fn p25_spawn_activate_real_levels() {
    let (l, ds) = tables();
    let mut checked = 0;
    for d in ds {
        let Ok(lv) = crate::common::load_blocking(&l, d) else { continue };
        let Ok(w) = World::load(&lv, LoadOptions::default()) else { continue };
        let Some(spawn) = ee_sub_227fe8(&w, &SpawnAliases::default(), 0x189, -1) else { continue };
        let vols = p25::warp_volumes(&w);
        if vols.len() < 2 {
            continue;
        }
        let w = Arc::new(w);
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut set = ClassSet::build(&w, &mut h);
        set.post_load(&mut h);
        // states: first volume 2 (kept), the others 3 (reset to 0)
        let mut want = Vec::new();
        for (k, &v) in vols.iter().enumerate() {
            if let Some((_, Inst::C2dc6f0(c))) = set.more.iter_mut().find(|x| x.0 == v && matches!(x.1, Inst::C2dc6f0(_))) {
                c.f_13c = if k == 0 { 2 } else { 3 };
                if k != 0 {
                    want.push(v);
                }
            }
        }
        if want.is_empty() {
            continue;
        }
        want.reverse();
        // a pair of two objects with unique oids, one record and one task entry
        let uniq: Vec<usize> = (0..w.objects.len()).filter(|&i| w.objects[i].oid > 0 && w.find_oid(w.objects[i].oid).len() == 1).take(2).collect();
        let mut lists = SpawnLists::default();
        lists.FUN_00228958(spawn, w.objects[uniq[0]].oid, w.objects[uniq[1]].oid);
        lists.FUN_00228958(spawn, -12345, w.objects[uniq[1]].oid); // a not found -> skipped
        lists.FUN_00228920(spawn, w.objects[uniq[0]].oid, 1);
        h.k27_script_msgs.clear();
        h.k28_task_calls.clear();
        let mut cm = Cm::new();
        let r = p25::FUN_00228750(&w, spawn, &lists, &mut set, &mut h, Some(&mut cm));
        eprintln!("level spawn {spawn} warps {} reset {:?} placed {:?}", r.warps, r.warp_reset, r.placed);
        assert_eq!(r.warp_reset, want);
        for (x, i) in &set.more {
            if let Inst::C2dc6f0(c) = i {
                if vols.contains(x) && *x != vols[0] {
                    assert_eq!(c.f_13c, 0);
                }
            }
        }
        assert_eq!(cm.cut, 1);
        assert!(r.cut);
        assert_eq!(r.placed.len(), 1);
        assert!(h.k27_script_msgs.iter().any(|m| m.0 == spawn as u32 + 1 && m.1 == 5));
        assert!(h.k28_task_calls.iter().any(|c| *c == (0x1619a0, spawn as u32 + 1, 0x1a, 0)));
        assert!(h.k28_task_calls.iter().any(|c| c.0 == 0x164640 && c.2 == 1 && c.3 == spawn as u32 + 1));
        checked += 1;
        if checked >= 2 {
            break;
        }
    }
    assert!(checked >= 1, "no level with spawn 0x189 and warp volumes");
}

// ---------------------------------------------------------------- jts 6 (gadget 0x18)

use sly2_recomp::engine::physics::CollisionWorld;
use sly2_recomp::engine::player::p25::{ASEG_6, DAT_002BC9B8};
use sly2_recomp::engine::player::p6::P6Level;
use sly2_recomp::engine::player::{Jt, JtAnim, JtInput, JT_RADIUS};

#[derive(Default)]
struct Rec(Vec<u32>);
impl JtAnim for Rec {
    fn play_aseg(&mut self, field: u32, _: i32, _: i32) {
        self.0.push(field);
    }
}

fn standing() -> (Jt, CollisionWorld, World) {
    use sly2_recomp::engine::game::spawn;
    let (l, ds) = tables();
    let w = World::load(&crate::common::load_blocking(&l, ds[0]).unwrap(), LoadOptions::default()).unwrap();
    let cw = sly2_recomp::engine::player::p10::player_collision_world(&w);
    let (_, c) = spawn::ee_sub_15e9c8(&w, None, spawn::CLASS_SPAWN);
    let p = c.into_iter().find_map(|i| spawn::spawn_position(&w, i)).unwrap();
    let mut jt = Jt::new([p[0], p[1], p[2] + JT_RADIUS + 5.0]);
    P6Level::load(&w).apply(&mut jt);
    for _ in 0..120 {
        jt.update(&cw, JtInput::default(), 1.0 / 60.0, &mut ());
    }
    assert_eq!(jt.jts, 0, "standing");
    (jt, cw, w)
}

/// 0x1374f0 side pick + 0x1375d0 enter / exit + 0x1371c8 leave when the side ASEG is done + 0x137218 facing.
#[test]
fn p25_jts6_side_and_exit() {
    let (mut jt, cw, _w) = standing();
    assert!(ASEG_6.iter().all(|f| jt.cane.data.asegs.contains_key(f)), "side ASEGs bound (oids 0x119 / 0x11a)");
    let dt = 1.0 / 60.0;
    let tg = [jt.phys.pos[0] + 250.0, jt.phys.pos[1], jt.phys.pos[2]];
    jt.p13.f_10dc = 7;
    jt.p13.pos_10dc = Some(tg);
    // no stick: the current side [3] (0)
    assert_eq!(jt.FUN_001374f0(), 0);
    // stick toward +y: dir(angle - pi/2) = +x, target at +x -> dot > 0 -> side 0; stick toward -y -> side 1
    jt.joy.mag = 1.0;
    jt.joy.angle = std::f32::consts::FRAC_PI_2;
    assert_eq!(jt.FUN_001374f0(), 0);
    jt.joy.angle = -std::f32::consts::FRAC_PI_2;
    assert_eq!(jt.FUN_001374f0(), 1);
    let mut an = Rec::default();
    jt.set_jts(6, -1, &mut an);
    assert_eq!(jt.jts, 6);
    assert_eq!((jt.j6.side, jt.j6.cur), (1, 1));
    assert_eq!(an.0.last(), Some(&ASEG_6[1]));
    assert!(jt.yaw.abs() < 1e-4, "faces the target at +x (yaw {})", jt.yaw);
    // same side again: no-op
    let n = an.0.len();
    jt.FUN_001375d0(1, &mut an);
    assert_eq!(an.0.len(), n);
    let len = jt.cane.data.asegs[&ASEG_6[1]].len;
    let mut steps = 0;
    while jt.jts == 6 && steps < 600 {
        jt.update(&cw, JtInput::default(), dt, &mut an);
        steps += 1;
    }
    assert_ne!(jt.jts, 6, "left jts 6 when the side ASEG ended");
    let t = steps as f32 * dt;
    assert!((t - len).abs() < 0.1, "left after {t} s, ASEG {len} s");
    assert_eq!(jt.j6.side, -1, "exit 0x1375d0(-1)");
    let _ = DAT_002BC9B8;
}

/// 0x1375d0(-1): velocity * 0.5.
#[test]
fn p25_jts6_exit_velocity() {
    let mut jt = Jt::new([0.0; 3]);
    jt.j6.side = 0;
    jt.set_velocity_pub([100.0, -40.0, 8.0]);
    jt.FUN_001375d0(-1, &mut ());
    let v = [0, 1, 2].map(|i| jt.phys.v[i] + jt.phys.imp_lin[i]);
    assert!((v[0] - 50.0).abs() < 1e-3 && (v[1] + 20.0).abs() < 1e-3 && (v[2] - 4.0).abs() < 1e-3, "{v:?}");
}
