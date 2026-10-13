//! WP-P6: Sly's moves on a real level (level-table row 3): JT blend tuning (0x1a0300) from his ASEGs, his
//! GEOM-199 hull from his ALO, path targets from the level's objects, attach / ride / release on a real rope,
//! and the Bevy wiring (World -> PlayerWorldData -> the spawned JT).
use bevy_app::App;
use sly2_recomp::engine::disc::descriptor;
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::input::pad::BTN_CROSS;
use sly2_recomp::engine::physics::CollisionWorld;
use sly2_recomp::engine::player::p5_attach::Blend2;
use sly2_recomp::engine::player::p5_target::TargetGeom;
use sly2_recomp::engine::player::p6::{self, P6Level};
use sly2_recomp::engine::player::plugin::{Player, PlayerLevel, PlayerPlugin};
use sly2_recomp::engine::player::{Jt, JtInput, JTS_JUMP};
use sly2_recomp::engine::world::{LoadOptions, World};
use std::sync::{Arc, OnceLock};

const ISO: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../Sly 2 - Band of Thieves (USA).iso");
/// row-3 rope (class 13, C2d6fe8, 8-point polyline)
const ROPE: usize = 2967;

fn world() -> Option<&'static World> {
    static W: OnceLock<Option<World>> = OnceLock::new();
    W.get_or_init(|| {
        let iso = Arc::new(IsoImage::open(ISO).ok()?);
        let t = descriptor::static_table(&iso.read_file("SCUS_973.16").ok()?)?;
        let l = LevelLoader::new(iso, ModOverlay::default());
        let boot = crate::common::load_blocking(&l, t[25]).ok()?;
        let lv = crate::common::load_blocking(&l, boot.header.level_table.entries[3].desc).ok()?;
        World::load(&lv, LoadOptions::default()).ok()
    })
    .as_ref()
}

macro_rules! need_world {
    () => {
        match world() {
            Some(w) => w,
            None => {
                eprintln!("ISO missing: skipped");
                return;
            }
        }
    };
}

#[test]
fn tuning_from_sly_asegs() {
    let w = need_world!();
    let lvl = P6Level::load(w);
    let t = lvl.tuning.expect("JT found");
    // entry 0 of +0x13d0 (oid 0x7f): root (0,0,75) at t=0 -> (0,-500,75) at T=0.5 -> (0,1000,0)
    assert_eq!(t.blend_13d0.vel[0], [0.0, 1000.0, 0.0]);
    assert_eq!(t.blend_13d0.vel[2], [-1000.0, 0.0, 0.0]);
    assert!(t.blend_13d0.en.iter().all(|&e| e), "all 8 entry ASEGs resolved");
    assert_eq!(t.blend_1ac0.vel[0], [0.0, 300.0, 0.0]);
    assert_eq!(t.present, [true, true, true]);
}

#[test]
fn hull_from_sly_alo() {
    let w = need_world!();
    let h = p6::player_hull(w, P6Level::load(w).jt.unwrap()).expect("GEOM 199");
    assert_eq!(h.radius, 105.0);
    assert_eq!(h.n_hull, 8);
}

#[test]
fn targets_from_level_paths() {
    let w = need_world!();
    let ts = p6::level_targets(w);
    let r = ts.iter().find(|t| t.obj.id == ROPE).expect("rope 2967 is a target");
    assert!(matches!(r.obj.geom, TargetGeom::Rope { .. }));
    let p = r.attach.as_ref().unwrap().path.as_ref().unwrap();
    assert_eq!(p.pts.len(), 8);
    assert!(p.l[7] > 0.0 && p.l.windows(2).all(|x| x[0] <= x[1]));
}

fn step(jt: &mut Jt, cw: &CollisionWorld, stick: [f32; 2], held: u32, pressed: u32) {
    jt.pad_pending = Some((held, pressed));
    jt.update(cw, JtInput { stick, jump: false }, 1.0 / 60.0, &mut ());
}

/// attach to the real rope, ride it with the stick, X releases (jts 7 -> 2, detached).
#[test]
fn ride_real_rope() {
    let w = need_world!();
    let lvl = P6Level::load(w);
    let tg = lvl.targets.iter().find(|t| t.obj.id == ROPE).unwrap().attach.clone().unwrap();
    let path = tg.path.clone().unwrap();
    let wp = |q: [f32; 3]| [0, 1, 2].map(|j| q[0] * path.rot[0][j] + q[1] * path.rot[1][j] + q[2] * path.rot[2][j] + path.pos[j]);
    let (a, b) = (wp(path.pts[0]), wp(path.pts[1]));
    let start = [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0, (a[2] + b[2]) / 2.0];
    let cw = CollisionWorld::default();
    let mut jt = Jt::new(start);
    lvl.apply(&mut jt);
    assert!(jt.hull.is_some());
    jt.t = 10.0;
    jt.att.rope_vmax = 300.0;
    // GAP (unported): the kind-3 rope blend a+0x240 velocities (rope ASEG); supplied as in P5's ride_rope
    jt.att.rope_blend = Blend2 { on: true, k: [1.0, 1.0], vel: [[0.0; 3], [200.0, 0.0, 0.0]], aseg: true, ..Default::default() };
    jt.FUN_001928f8(3, Some(ROPE), 0, &mut ());
    assert_eq!(jt.jts, 7);
    assert_eq!(jt.attach.target, Some(ROPE));
    let p0 = jt.phys.pos;
    let u0 = jt.att.u;
    for _ in 0..60 {
        step(&mut jt, &cw, [1.0, 0.0], 0, 0);
        assert_eq!(jt.jts, 7, "still riding");
    }
    let d = [jt.phys.pos[0] - p0[0], jt.phys.pos[1] - p0[1], jt.phys.pos[2] - p0[2]];
    assert!((d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() > 10.0, "moved along the rope: {:?} -> {:?}", p0, jt.phys.pos);
    assert!(jt.att.u > u0 + 10.0, "rope param advanced: {} -> {}", u0, jt.att.u);
    step(&mut jt, &cw, [0.0; 2], BTN_CROSS, BTN_CROSS);
    assert_eq!(jt.jts, JTS_JUMP, "X jumps off");
    assert_eq!((jt.attach.kind, jt.attach.target), (-1, None), "detached");
}

/// Bevy: a World resource feeds the spawned JT (hull, targets, tuning).
#[test]
fn plugin_applies_level_data() {
    let w = need_world!();
    let mut app = App::new();
    app.add_plugins(PlayerPlugin);
    app.insert_resource(w.clone());
    app.insert_resource(PlayerLevel { cw: Arc::new(CollisionWorld::default()), spawn: Some([0.0, 0.0, 0.0]) });
    app.update();
    app.update();
    let mut q = app.world_mut().query::<&Player>();
    let p = q.iter(app.world()).next().expect("player spawned");
    assert!(p.jt.hull.is_some(), "hull contacts replace the sphere");
    assert!(p.jt.p5_targets.iter().any(|t| t.obj.id == ROPE));
    assert_eq!(p.jt.p5.blend_13d0.vel[0], [0.0, 1000.0, 0.0]);
}
