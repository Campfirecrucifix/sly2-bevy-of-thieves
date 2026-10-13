//! WP-P16: jts 0xd / 0xe / 0x11 bodies (0x18a8c8 / 0x18c250), FUN_0018e540 swing sounds through the audio
//! emitter path, jts 0x11 job events.

use sly2_recomp::engine::audio::emitter::EmitterSystem;
use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::jobs::JobMgr;
use sly2_recomp::engine::physics::CollisionWorld;
use sly2_recomp::engine::player::p16::{self, ASEG_11, ASEG_D, ASEG_E};
use sly2_recomp::engine::player::p6::P6Level;
use sly2_recomp::engine::player::{Jt, JtAnim, JtInput, JT_RADIUS};
use sly2_recomp::engine::world::{LoadOptions, World};
use std::sync::Arc;

fn world(r: usize) -> World {
    let iso = Arc::new(IsoImage::open(concat!(env!("CARGO_MANIFEST_DIR"), "/../Sly 2 - Band of Thieves (USA).iso")).expect("ISO"));
    let t = sly2_recomp::engine::disc::descriptor::static_table(&iso.read_file("SCUS_973.16").unwrap()).unwrap();
    let l = LevelLoader::new(iso, ModOverlay::default());
    let boot = crate::common::load_blocking(&l, t[25]).unwrap();
    let d: Vec<_> = boot.header.level_table.entries.iter().map(|e| e.desc).collect();
    let lv = crate::common::load_blocking(&l, d[r]).unwrap();
    World::load(&lv, LoadOptions::default()).unwrap()
}

#[derive(Default)]
struct Rec(Vec<u32>);
impl JtAnim for Rec {
    fn play_aseg(&mut self, field: u32, _: i32, _: i32) {
        self.0.push(field);
    }
}

/// Row 0 JT standing at the spawn after 2 s.
fn standing() -> (Jt, CollisionWorld) {
    use sly2_recomp::engine::game::spawn;
    let w = world(0);
    let cw = sly2_recomp::engine::player::p10::player_collision_world(&w);
    let (_, c) = spawn::ee_sub_15e9c8(&w, None, spawn::CLASS_SPAWN);
    let p = c.into_iter().find_map(|i| spawn::spawn_position(&w, i)).unwrap();
    let mut jt = Jt::new([p[0], p[1], p[2] + JT_RADIUS + 5.0]);
    P6Level::load(&w).apply(&mut jt);
    for _ in 0..120 {
        jt.update(&cw, JtInput::default(), 1.0 / 60.0, &mut ());
    }
    assert_eq!(jt.jts, 0, "standing");
    (jt, cw)
}

/// FUN_0018e540 sounds: jtcs 0 -> 0xf, 3..7 -> 0xb at the JT, none when chained; leaving jtcs 1 plays the
/// jt+0x18c4 target's +0x130 sound. The plugin drain creates positioned emitters (0x200 set).
#[test]
fn p16_swing_sounds() {
    let mut jt = Jt::new([10.0, 20.0, 30.0]);
    jt.FUN_0018e540(4, &mut ());
    assert_eq!(jt.p16.sfx, vec![(0xb, [10.0, 20.0, 30.0])]);
    jt.FUN_0018e540(-1, &mut ());
    jt.FUN_0018e540(0, &mut ());
    assert_eq!(jt.p16.sfx.last().unwrap().0, 0xf);
    jt.FUN_0018e540(-1, &mut ());
    let n = jt.p16.sfx.len();
    jt.FUN_0018e540(8, &mut ());
    jt.FUN_0018e540(9, &mut ());
    assert_eq!(jt.p16.sfx.len(), n, "jtcs 8 / 9: no sound");
    jt.FUN_0018e540(-1, &mut ());
    jt.p16.snd_18c4 = 0x55;
    jt.FUN_0018e540(1, &mut ());
    jt.FUN_0018e540(-1, &mut ());
    assert_eq!(jt.p16.sfx.last().unwrap().0, 0x55, "leaving jtcs 1");
    let mut es = EmitterSystem::default();
    let mut jobs = JobMgr::default();
    p16::drain(&mut jt, Some(&mut es), &mut jobs);
    assert!(jt.p16.sfx.is_empty());
    assert_eq!(es.active.len(), 3); // 0xb, 0xf, 0x55
    for &id in &es.active {
        let e = es.get(id).unwrap();
        assert_eq!(e.pos, [10.0, 20.0, 30.0]);
        assert_eq!(e.flags & 0x201, 0x201, "created + positioned");
    }
}

/// The swing (jts 3) enter / exit drive jtcs 1 through the real FUN_0018e540.
#[test]
fn p16_swing_jts3_cane_state() {
    use sly2_recomp::engine::player::p5_states::SwingTarget;
    let mut jt = Jt::new([0.0, 0.0, 0.0]);
    jt.p5w.swing = Some(SwingTarget { pos: [0.0, 0.0, 230.0], ..Default::default() });
    jt.p16.snd_18c4 = 0x77;
    jt.set_jts(3, 1, &mut ());
    assert_eq!(jt.cane.jtcs, 1);
    jt.set_jts(2, 4, &mut ());
    assert_eq!(jt.cane.jtcs, -1);
    assert!(jt.p16.sfx.iter().any(|s| s.0 == 0x77), "{:?}", jt.p16.sfx);
}

/// jts 0x11: +0x1d30 -> SetJtJts(0x11, -1) (ASEG jt+0x1d34, job event 0x37); stays while +0x1d30 holds for
/// at most DAT_002bf360 = 5 s, then jts 0 with +0x1d30 cleared and event 0x38.
#[test]
fn p16_jts11_timeout() {
    let (mut jt, cw) = standing();
    let mut an = Rec::default();
    let dt = 1.0 / 60.0;
    jt.p13.f_1d30 = 1;
    jt.update(&cw, JtInput::default(), dt, &mut an);
    assert_eq!(jt.jts, 0x11);
    assert!(an.0.contains(&ASEG_11));
    assert_eq!(jt.p16.jobs, vec![0x37]);
    for _ in 0..(4.5 / dt) as usize {
        jt.update(&cw, JtInput::default(), dt, &mut an);
    }
    assert_eq!(jt.jts, 0x11, "held");
    for _ in 0..(1.0 / dt) as usize {
        jt.update(&cw, JtInput::default(), dt, &mut an);
    }
    assert_ne!(jt.jts, 0x11, "5 s limit");
    assert_eq!(jt.p13.f_1d30, 0);
    assert_eq!(&jt.p16.jobs[..2], &[0x37, 0x38]);
    let mut jobs = JobMgr::default();
    p16::drain(&mut jt, None, &mut jobs);
    assert!(jobs.calls.contains(&(0x00154a90, 0x38)));
}

/// jts 0xe on the ground: ASEG jt+0x1a84, after 0.4 s the clock jumps to event 0x16d, ends in jts 0 when
/// the ASEG is done. jts 0xd on the ground: ASEG jt+0x1a80, jts 0 after 0.4 s (or the 0xb launch when its
/// ASEG is shorter).
#[test]
fn p16_jts_d_e_ground() {
    let (mut jt, cw) = standing();
    let dt = 1.0 / 60.0;
    let lens: Vec<_> = [ASEG_D, ASEG_E, ASEG_11].iter().map(|f| jt.cane.data.asegs.get(f).map(|a| (a.len, a.ev.clone()))).collect();
    println!("ASEG 0x1a80 / 0x1a84 / 0x1d34: {lens:?}");
    let le = lens[1].as_ref().expect("ASEG 0x1a84").0;
    let mut an = Rec::default();
    jt.set_jts(0xe, -1, &mut an);
    assert!(an.0.contains(&ASEG_E));
    let t0 = jt.t;
    let mut seen_seek = false;
    for _ in 0..((le + 1.0) / dt) as usize {
        jt.update(&cw, JtInput::default(), dt, &mut an);
        seen_seek |= jt.p16.t_off > 0.0;
        if jt.jts != 0xe {
            break;
        }
    }
    println!("0xe left after {} s (seek {seen_seek})", jt.t - t0);
    assert_eq!(jt.jts, 0, "0xe done -> stand");
    let tev = lens[1].as_ref().unwrap().1.iter().find(|e| e.1 == 0xb && e.2 == 0x16d).map(|e| e.0);
    if tev.is_some_and(|t| t > 0.45) {
        assert!(seen_seek, "seek to 0x16d");
    }
    let mut an = Rec::default();
    jt.set_jts(0xd, -1, &mut an);
    assert!(an.0.contains(&ASEG_D));
    let t0 = jt.t;
    for _ in 0..60 {
        jt.update(&cw, JtInput::default(), dt, &mut an);
        if jt.jts != 0xd {
            break;
        }
    }
    println!("0xd -> jts {} jtbs {} after {} s", jt.jts, jt.jtbs, jt.t - t0);
    let ld = lens[0].as_ref().map_or(0.0, |a| a.0);
    if ld > 0.45 {
        assert_eq!(jt.jts, 0, "ground after 0.4 s");
        assert!(jt.t - t0 > 0.4);
    } else {
        assert_eq!(jt.jts, 2, "short ASEG: FUN_00195a00(0xb)");
    }
}
