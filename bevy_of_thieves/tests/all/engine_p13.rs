//! WP-P13: jts 5 (stance next to the focus object), the cane charged follow-up (event 0x165 -> (5, 6/7)),
//! the jtbs 8/9 ends, the focus setter / range test FUN_00140678 / FUN_00140af8, FUN_00140558 selection,
//! the Splice flag FUN_0018df50.

use sly2_recomp::engine::classes::plugin::EngineHost;
use sly2_recomp::engine::classes::{ClassSet, Inst};
use sly2_recomp::engine::disc::descriptor::Descriptor;
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::physics::query::CollisionWorld;
use sly2_recomp::engine::player::p11_cane::{CaneAseg, CaneData};
use sly2_recomp::engine::player::p13::{guard_focus_prio, FocusCand};
use sly2_recomp::engine::player::p6::P6Level;
use sly2_recomp::engine::player::{Jt, JtInput, JTS_STAND};
use sly2_recomp::engine::world::{LoadOptions, World};
use std::sync::Arc;

const DT: f32 = 1.0 / 60.0;

fn levels() -> (LevelLoader, Vec<Descriptor>) {
    crate::common::levels() // public: ISO level table instead of the analysis DB
}

/// Swing ASEGs with the hold event 0x165 at 0.3 s; jts 5 bodies of 0.5 s (0x1d08 sets +0x18e8 at 0.2 s).
fn synthetic() -> CaneData {
    let a = CaneAseg { len: 0.8333333, ev: vec![(0.3, 0xb, 0x165), (0.5, 0xb, 0x164)] };
    let mut d = CaneData::default();
    for f in (0x1ce0..=0x1d04).step_by(4).chain([0x1d14, 0x1d18]) {
        d.asegs.insert(f, a.clone());
    }
    d.asegs.insert(0x1d08, CaneAseg { len: 0.5, ev: vec![(0.2, 0xb, 0x166)] });
    for f in [0x1d0c, 0x1d10, 0x1fe8] {
        d.asegs.insert(f, CaneAseg { len: 0.5, ev: vec![] });
    }
    d
}

fn standing() -> Jt {
    let mut jt = Jt::new([0.0, 0.0, 35.0]);
    jt.t = 10.0;
    jt.t_ground = 1.0e9;
    jt.cane.data = synthetic();
    jt
}

fn focus(jt: &mut Jt, x: f32, z: f32) {
    jt.FUN_00140678(0, 1, 4, Some(([x, 0.0, 35.0 + z], false)));
}

fn step(jt: &mut Jt, held: u32, pressed: u32) {
    jt.pad_pending = Some((held, pressed));
    jt.update(&CollisionWorld::default(), JtInput::default(), DT, &mut ());
    jt.phys.pos = [0.0, 0.0, 35.0]; // no floor
}

#[test]
fn p13_focus_range_and_hysteresis() {
    let mut jt = Jt::new([0.0, 0.0, 0.0]);
    assert!(!jt.FUN_00140af8(false), "init: mode -1");
    jt.FUN_00140678(0, 1, 4, Some(([900.0, 0.0, 0.0], false)));
    assert!(jt.FUN_00140af8(false));
    jt.FUN_00140678(0, 1, 4, Some(([1200.0, 0.0, 0.0], false)));
    assert!(!jt.FUN_00140af8(false) && jt.FUN_00140af8(true), "1000 / 1500 range");
    jt.FUN_00140678(0, 1, 4, Some(([1200.0, 0.0, 0.0], true)));
    assert!(jt.FUN_00140af8(false), "class 0x3a: 1700");
    jt.FUN_00140678(0, 1, 4, Some(([0.0, 0.0, 800.0], false)));
    assert!(!jt.FUN_00140af8(false) && jt.FUN_00140af8(true), "dz window 750 / 1250");
    jt.FUN_00140678(0, 1, 4, Some(([0.0, 0.0, -800.0], false)));
    assert!(!jt.FUN_00140af8(false) && jt.FUN_00140af8(true));
    jt.FUN_00140678(3, 1, 4, Some(([0.0, 0.0, 0.0], false)));
    assert!(!jt.FUN_00140af8(true), "kind 3 never");
    jt.FUN_00140678(0, 1, 2, Some(([0.0, 0.0, 0.0], false)));
    assert!(!jt.FUN_00140af8(true), "mode != 4 never");
}

#[test]
fn p13_enter_stay_leave_jts5() {
    let mut jt = standing();
    focus(&mut jt, 400.0, 0.0);
    step(&mut jt, 0x8, 0); // R1 held: no entry
    assert_eq!(jt.jts, JTS_STAND);
    step(&mut jt, 0, 0);
    assert_eq!((jt.jts, jt.jtbs), (5, 4), "focus in range -> SetJtJts(5, 4)");
    focus(&mut jt, 1400.0, 0.0);
    step(&mut jt, 0, 0);
    assert_eq!(jt.jts, 5, "wide range keeps jts 5");
    focus(&mut jt, 1600.0, 0.0);
    step(&mut jt, 0, 0);
    assert_eq!(jt.jts, JTS_STAND, "out of the wide range -> SetJtJts(0, -1)");
    focus(&mut jt, 400.0, 0.0);
    step(&mut jt, 0, 0);
    assert_eq!(jt.jts, 5);
    step(&mut jt, 0x8, 0);
    assert_eq!(jt.jts, JTS_STAND, "R1 leaves jts 5");
}

#[test]
fn p13_charged_followup_and_end() {
    let mut jt = standing();
    focus(&mut jt, 400.0, 0.0);
    step(&mut jt, 0, 0);
    assert_eq!((jt.jts, jt.jtbs), (5, 4));
    step(&mut jt, 0x80, 0x80);
    assert!((0xc..0x16).contains(&jt.cane.jtcs) && jt.cane.jtcs % 2 == 0, "Square in jts 5 jtbs 4 swings (even jtcs -> follow-up 6)");
    let mut f = 0;
    while jt.jtbs != 6 && f < 40 {
        step(&mut jt, 0x80, 0);
        f += 1;
    }
    println!("event 0x165 -> jtbs 6 after {f} frames");
    assert_eq!((jt.jts, jt.jtbs, jt.cane.jtcs), (5, 6, -1), "held through 0x165 -> SetJtJts(5, 6), cane ended");
    for _ in 0..15 {
        step(&mut jt, 0x80, 0);
    }
    assert!(jt.cane.f_18e8, "0x1d08 event 0x166 -> +0x18e8");
    step(&mut jt, 0, 0);
    assert_eq!(jt.jtbs, 8, "released with +0x18e8 -> (5, 8)");
    for _ in 0..40 {
        step(&mut jt, 0, 0);
        if jt.jtbs == 4 {
            break;
        }
    }
    assert_eq!((jt.jts, jt.jtbs), (5, 4), "0x1d10 done (msg 3) -> (5, 4)");
    // released before 0x165: no follow-up
    let mut jt = standing();
    focus(&mut jt, 400.0, 0.0);
    step(&mut jt, 0, 0);
    step(&mut jt, 0x80, 0x80);
    for _ in 0..40 {
        step(&mut jt, 0, 0);
    }
    assert_eq!((jt.jts, jt.jtbs), (5, 4), "+0x18ec cleared -> 0x165 ignored");
}

#[test]
fn p13_splice_flag_1d30() {
    let mut jt = standing();
    jt.FUN_0018df50(1);
    step(&mut jt, 0, 0);
    assert_eq!(jt.jts, 0x11, "+0x1d30 -> SetJtJts(0x11, -1)");
}

#[test]
fn p13_focus_selection_on_real_guards() {
    let (loader, descs) = levels();
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let p6 = P6Level::load(&w);
        if p6.cane.geom.is_none() {
            continue;
        }
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        s.post_load(&mut h);
        let mut cands: Vec<FocusCand> = s
            .k6
            .iter()
            .filter_map(|(i, x)| match x {
                Inst::Guard(g) => Some(FocusCand { handle: *i as u32 + 1, pos: g.body.pos, is_3a: false, prio: guard_focus_prio(g.f_2c0) }),
                _ => None,
            })
            .collect();
        if cands.len() < 2 {
            continue;
        }
        let g = cands[1];
        let mut jt = Jt::new([g.pos[0] - 300.0, g.pos[1], g.pos[2]]);
        p6.apply(&mut jt);
        jt.FUN_00140558(&cands);
        println!("lsn {}: {} guards, nearest {} prio {:?}", d.lsn(), cands.len(), jt.p13.f_10dc, cands.iter().map(|c| c.prio).collect::<Vec<_>>());
        assert_eq!(jt.p13.f_10dc, g.handle, "lowest score = nearest guard");
        // a pursuing guard (bit 5) wins the mode/object slot with priority 4
        let far = cands.len() - 1;
        cands[far].prio = guard_focus_prio(1 << 5);
        jt.FUN_00140558(&cands);
        assert_eq!(jt.p13.f_10ec, 4);
        assert_eq!(jt.p13.obj_10e8.map(|o| o.0), Some(cands[far].pos));
        return;
    }
    panic!("no level with Sly and 2 guards");
}

/// Probe (ignored): poles (class 0x26) / swing points (0x13) per level-table row, for the pad-script shots.
#[test]
#[ignore]
fn p13_probe_pole_rows() {
    let iso = Arc::new(IsoImage::open(concat!(env!("CARGO_MANIFEST_DIR"), "/../Sly 2 - Band of Thieves (USA).iso")).expect("ISO"));
    let t = sly2_recomp::engine::disc::descriptor::static_table(&iso.read_file("SCUS_973.16").unwrap()).unwrap();
    let l = LevelLoader::new(iso, ModOverlay::default());
    let boot = crate::common::load_blocking(&l, t[25]).unwrap();
    for (r, e) in boot.header.level_table.entries.iter().enumerate() {
        let Ok(lv) = crate::common::load_blocking(&l, e.desc) else { continue };
        let Ok(w) = World::load(&lv, LoadOptions::default()) else { continue };
        let ts = sly2_recomp::engine::player::p6::level_point_targets(&w);
        let poles: Vec<_> = ts.iter().filter(|t| t.obj.mask == 1).map(|t| format!("{} {:.110}", t.obj.id, format!("{:?}", t.obj.geom))).take(3).collect();
        let pts: Vec<_> = ts.iter().filter(|t| t.obj.mask == 7).map(|t| format!("{} {:.110}", t.obj.id, format!("{:?}", t.obj.geom))).take(3).collect();
        if !ts.is_empty() { println!("row {r} lsn {} n {} poles {poles:?}
   points {pts:?}", e.desc.lsn(), ts.len()); }
    }
}

/// Probe (ignored): O in the air past the row-14 pole (empty collision world).
#[test]
#[ignore]
fn p13_probe_pole_pick() {
    let iso = Arc::new(IsoImage::open(concat!(env!("CARGO_MANIFEST_DIR"), "/../Sly 2 - Band of Thieves (USA).iso")).expect("ISO"));
    let t = sly2_recomp::engine::disc::descriptor::static_table(&iso.read_file("SCUS_973.16").unwrap()).unwrap();
    let l = LevelLoader::new(iso, ModOverlay::default());
    let boot = crate::common::load_blocking(&l, t[25]).unwrap();
    let lv = crate::common::load_blocking(&l, boot.header.level_table.entries[14].desc).unwrap();
    let w = World::load(&lv, LoadOptions::default()).unwrap();
    let p6 = P6Level::load(&w);
    let cw = sly2_recomp::engine::player::p10::player_collision_world(&w);
    for x0 in [5700.0f32, 5800.0, 5900.0] {
        for z in [700.0f32, 800.0, 900.0] {
            let mut jt = Jt::new([x0, -9437.6, z]);
            p6.apply(&mut jt);
            jt.t = 10.0;
            jt.jts = 2;
            jt.jtbs = 4;
            jt.set_velocity_pub([515.0, 0.0, 0.0]);
            let ok = jt.FUN_0019b110(&cw, None, &mut ());
            println!("x {x0} z {z}: {ok} jts {} jtbs {} att {:?} mask {:x}", jt.jts, jt.jtbs, jt.attach.target, jt.mask_1000);
        }
    }
    let poles: Vec<_> = p6.targets.iter().filter(|t| t.obj.mask == 1).map(|t| (t.obj.id, t.obj.range)).collect();
    println!("poles {}", poles.len());
}
