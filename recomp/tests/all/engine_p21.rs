//! WP-P21: Bentley / Murray movement, character switches, ledge hang offset.

use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
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

#[test]
#[ignore]
fn p21_probe_pose_extent() {
    use sly2_recomp::engine::playerdraw::{find_jt, JtAnimator, JtRig};
    let w = world(0);
    let jt = find_jt(&w).unwrap();
    let mut an = JtAnimator::new(JtRig::build(&w, jt));
    let ext = |wd: &[[[f32; 4]; 4]]| {
        let (mut lo, mut hi) = (f32::MAX, f32::MIN);
        for m in wd { lo = lo.min(m[3][2]); hi = hi.max(m[3][2]); }
        (lo, hi)
    };
    let rest = an.rig.pose.world();
    println!("rest z extent {:?} root {:?}", ext(&rest), rest[0][3]);
    an.request(&w, 0x414, 0x11);
    for _ in 0..30 { an.step(1.0 / 60.0); }
    let wd = an.rig.pose.world();
    println!("hang z extent {:?} root {:?}", ext(&wd), wd[0][3]);
    for i in 0..an.rig.keys.len() { println!("b{i:3} {:?} rest {:?} hang {:?}", an.rig.keys[i], rest[i][3], wd[i][3]); }
}

#[test]
#[ignore]
fn p21_probe_stand_height() {
    use sly2_recomp::engine::physics::geom::V3;
    use sly2_recomp::engine::player::{p6::P6Level, Jt, JtInput};
    let w = world(0);
    let cw = sly2_recomp::engine::player::p10::player_collision_world(&w);
    let lvl = P6Level::load(&w);
    let xy: Vec<f32> = std::env::var("P21_XY").unwrap_or("-18984.42,8000".into()).split(',').map(|v| v.parse().unwrap()).collect();
    let start: V3 = [xy[0], xy[1], 2600.0];
    let mut jt = Jt::new(start);
    lvl.apply(&mut jt);
    println!("hull {}", jt.hull.is_some());
    for f in 0..240 {
        jt.update(&cw, JtInput::default(), 1.0 / 60.0, &mut ());
        if f % 30 == 0 { println!("f{f} pos {:?} jts {} jtbs {}", jt.phys.pos, jt.jts, jt.jtbs); }
    }
    let p = jt.phys.pos;
    let hits = cw.clip_segment([p[0], p[1], p[2] + 1000.0], [p[0], p[1], p[2] - 300.0]);
    println!("floor hits {:?}", hits.iter().map(|h| (h.pos, h.t)).collect::<Vec<_>>());
    for dx in [-60.0f32, 60.0] {
        let h = cw.clip_segment([p[0] + dx, p[1], p[2] + 1000.0], [p[0] + dx, p[1], p[2] - 300.0]);
        println!("dx {dx} {:?}", h.iter().map(|h| (h.pos, h.t)).collect::<Vec<_>>());
    }
}

#[test]
#[ignore]
fn p21_probe_hull() {
    use sly2_recomp::engine::player::p6::{self, P6Level};
    let w = world(0);
    let h = p6::player_hull(&w, P6Level::load(&w).jt.unwrap()).unwrap();
    println!("radius {} n_hull {} i_p0 {} z_min {}", h.radius, h.n_hull, h.i_p0, h.z_min);
    println!("{:?}", h.geom.verts);
}

/// WP-P21 item 4: jt+0x240 = (0, 0, -75) (JT load 0x1885b8) is what 0x19b110 / 0x193a68 / 0x19faf8 read;
/// the port keeps the P5 hofs ZERO (with -75 Sly flies 150 above edges / tightropes: 0x193a68 and the
/// 0x19b110 tail both subtract it, while the JT model origin is at the feet; shots/P21).
#[test]
fn p21_hofs_240_kept_zero() {
    use sly2_recomp::engine::player::p5_hull::JT_240;
    use sly2_recomp::engine::player::p5_states::{FUN_0019faf8, P5};
    use sly2_recomp::engine::player::Jt;
    assert_eq!(JT_240, [0.0, 0.0, -75.0]);
    let st = P5::default();
    assert_eq!(st.hofs_240, [0.0; 3]);
    let jt = Jt::new([100.0, 200.0, 300.0]);
    assert_eq!(jt.p5.hofs_240, [0.0; 3]);
    // no safe point: 0x19faf8 falls back to pos + hofs
    assert_eq!(FUN_0019faf8(&jt, &st).pos, [100.0, 200.0, 300.0]);
}

/// WP-P21 item 4 (row 14 edge 5954): an O flight from above attaches (jts 7) with the feet on the edge.
#[test]
fn p21_edge_attach_feet_on_edge() {
    use sly2_recomp::engine::player::p5_target::TargetGeom;
    use sly2_recomp::engine::player::{Jt, JtInput};
    let w = world(14);
    let p6 = sly2_recomp::engine::player::p6::P6Level::load(&w);
    let cw = sly2_recomp::engine::player::p10::player_collision_world(&w);
    let t = p6.targets.iter().find(|t| t.obj.id == 5954).expect("edge 5954");
    let TargetGeom::Rope { path } = &t.obj.geom else { panic!("5954 not category B") };
    let mut jt = Jt::new([2047.0, -8352.0, 1700.0]);
    p6.apply(&mut jt);
    let mut attached = None;
    for i in 0..120 {
        let o = (8..10).contains(&i);
        jt.feed_pad(&{
            let mut p = sly2_recomp::engine::input::pad::PadState::default();
            if o {
                p.buttons = sly2_recomp::engine::input::pad::BTN_CIRCLE as _;
                p.pressed = if i == 8 { p.buttons } else { 0 };
            }
            p
        });
        jt.update(&cw, JtInput::default(), 1.0 / 60.0, &mut ());
        if jt.jts == 7 {
            attached = Some(i);
            break;
        }
    }
    assert!(attached.is_some(), "no attach (jts {} jtbs {})", jt.jts, jt.jtbs);
    let cp = path.FUN_001e6f60(jt.phys.pos).cp;
    println!("attach pos {:?} edge {:?}", jt.phys.pos, cp);
    assert!((jt.phys.pos[2] - cp[2]).abs() < 30.0, "feet on the edge: {:?} vs {:?}", jt.phys.pos, cp);
}

/// WP-P21 item 5: death only publishes `PlayerRespawn` (game/ flow does the 1:1 FUN_0022b490 reload);
/// the player layer no longer teleports the JT in place.
#[test]
fn p21_death_only_publishes_respawn() {
    use sly2_recomp::engine::player::p7_damage::Respawn;
    use sly2_recomp::engine::player::plugin::{publish_respawn, JtAnimEvents, Player, PlayerLevel, PlayerRespawn};
    use sly2_recomp::engine::player::Jt;
    let mut app = bevy_app::App::new();
    app.init_resource::<PlayerRespawn>();
    app.insert_resource(PlayerLevel { spawn: Some([0.0, 0.0, 0.0]), ..Default::default() });
    app.add_systems(bevy_app::Update, publish_respawn);
    let mut jt = Jt::new([500.0, 600.0, 700.0]);
    jt.dmg.hp = 0;
    let r = Respawn { level: 0, spawn: 7, spawn_parent: -1, flags: 0, a: 0x30a };
    jt.respawn = Some(r);
    let e = app.world_mut().spawn((Player { jt }, JtAnimEvents::default())).id();
    app.update();
    assert_eq!(app.world().resource::<PlayerRespawn>().0, Some(r));
    let p = app.world().get::<Player>(e).unwrap();
    assert!(p.jt.respawn.is_none());
    assert_eq!(p.jt.phys.pos, [500.0, 600.0, 700.0], "not moved to the spawn in place");
    assert_eq!(p.jt.dmg.hp, 0, "no in-place refill");
}

/// Probe: edge 5954 (row 14) path points (z) at the hang arc lengths of p14_edge_moves_along.
#[test]
#[ignore]
fn p21_probe_edge_z() {
    use sly2_recomp::engine::player::p5_target::TargetGeom;
    let w = world(14);
    let p6 = sly2_recomp::engine::player::p6::P6Level::load(&w);
    let t = p6.targets.iter().find(|t| t.obj.id == 5954).unwrap();
    let TargetGeom::Rope { path } = &t.obj.geom else { panic!() };
    println!("L {}", path.FUN_001e6648());
    for s in [1263.0f32, 1276.0, 1557.0, 1603.0, 2485.0, 2509.0] {
        println!("s {s} p {:?}", path.FUN_001e67b8(s).0);
    }
}
