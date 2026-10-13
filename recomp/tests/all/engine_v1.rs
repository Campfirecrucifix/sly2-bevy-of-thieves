//! WP-V1 (verification sweep, tools/verify_sweep.sh + recomp/VERIFY.md): cheap headless subset of the sweep.
//! Sly (Jt) on real levels with the real SO collision set, scripted like the sweep pad (walk / jump).
//! Needs the ISO at the absolute path below; tests are skipped (pass) when it is absent.
use sly2_recomp::engine::disc::descriptor;
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::game::spawn;
use sly2_recomp::engine::input::pad::BTN_CROSS;
use sly2_recomp::engine::player::p6::P6Level;
use sly2_recomp::engine::player::{Jt, JtInput, JT_RADIUS};
use sly2_recomp::engine::world::{LoadOptions, World};
use std::sync::Arc;

const ISO: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../Sly 2 - Band of Thieves (USA).iso");
const DT: f32 = 1.0 / 60.0;

/// Level-table row of the boot level (same rows as `game --table`).
fn level(row: usize) -> Option<Arc<World>> {
    let iso = Arc::new(IsoImage::open(ISO).ok()?);
    let t = descriptor::static_table(&iso.read_file("SCUS_973.16").ok()?)?;
    let l = LevelLoader::new(iso, ModOverlay::default());
    let boot = crate::common::load_blocking(&l, t[25]).ok()?;
    let lv = crate::common::load_blocking(&l, boot.header.level_table.entries[row].desc).ok()?;
    Some(Arc::new(World::load(&lv, LoadOptions::default()).ok()?))
}

/// Spawn like player::plugin (first class-spawn candidate, + JT_RADIUS + 5); None = no spawn found.
fn spawn_of(w: &World) -> Option<[f32; 3]> {
    let (_, c) = spawn::ee_sub_15e9c8(w, None, spawn::CLASS_SPAWN);
    c.into_iter().find_map(|i| spawn::spawn_position(w, i)).map(|p| [p[0], p[1], p[2] + JT_RADIUS + 5.0])
}

/// The sweep's pad cycle (400 frames): forward, jump, right, jump, back-left, back, double jump.
fn pad(i: u32) -> ([f32; 2], bool) {
    let c = i % 400;
    let stick = match c {
        0..120 => [0.0, 1.0],
        130..220 => [1.0, 0.0],
        230..300 => [-0.7071, -0.7071],
        300..380 => [0.0, -1.0],
        _ => [0.0, 0.0],
    };
    (stick, matches!(c, 120 | 220 | 380 | 392))
}

/// Runs `frames` frames; returns (spawn, first frame with a non-finite position / velocity, lowest z - spawn z).
fn run(row: usize, frames: u32, at: Option<[f32; 3]>) -> Option<([f32; 3], Option<u32>, f32)> {
    let w = level(row)?;
    // as player::plugin: P10 world (no sensor parts) + P6 level data incl. the JT hull
    let cw = sly2_recomp::engine::player::p10::player_collision_world(&w);
    let p = at.map(|p| [p[0], p[1], p[2] + JT_RADIUS + 5.0]).or_else(|| spawn_of(&w)).unwrap_or([0.0; 3]);
    let mut jt = Jt::new(p);
    let lvl = P6Level::load(&w);
    lvl.apply(&mut jt);
    jt.hull = lvl.hull.clone();
    let (mut nan, mut dz) = (None, 0.0f32);
    for i in 0..frames {
        let (stick, jump) = pad(i);
        let inp = JtInput { stick, jump, ..Default::default() };
        let b = if jump { BTN_CROSS } else { 0 };
        jt.pad_pending = Some((b, b));
        jt.update(&cw, inp, DT, &mut ());
        if jt.phys.pos.iter().chain(jt.phys.v.iter()).any(|x| !x.is_finite()) {
            nan = Some(i);
            break;
        }
        dz = dz.min(jt.phys.pos[2] - p[2]);
    }
    Some((p, nan, dz))
}

/// V1 subset: Sly stays finite and on the ground for 600 sweep-pad frames on levels that passed the sweep.
#[test]
fn v1_sweep_pad_stays_finite() {
    for row in [2usize, 13, 34] {
        let Some((p, nan, dz)) = run(row, 600, None) else {
            eprintln!("V1: ISO not available, skipped");
            return;
        };
        eprintln!("V1 row {row}: spawn {p:?} nan {nan:?} min dz {dz:.0}");
        assert_eq!(nan, None, "row {row}: Sly position / velocity became NaN");
        assert!(dz > -3000.0, "row {row}: Sly fell {dz} below the spawn");
    }
}

/// V1 probe (open issue, see recomp/VERIFY.md): rows 15 / 22 / 32 / 38 hang the game (a busy loop after Sly's
/// position becomes NaN on player frame 2 in jts 2 / jtbs 4); 12 / 37 enter death-reload loops.
#[test]
#[ignore]
fn v1_probe_nan_rows() {
    // spawn points the game binary uses (transition spawn oid 393 -> found object; sweep logs `player: spawn`)
    let game_spawn: [(usize, [f32; 3]); 5] = [
        (15, [540.82007, -4041.7283, 456.90833]),
        (22, [-16019.648, -11488.91, 79.35658]),
        (32, [2555.919, -4075.4675, 596.0872]),
        (37, [837.0426, 0.0, 29.042377]),
        (38, [0.0, 0.0, 0.0]),
    ];
    for (row, at) in game_spawn {
        eprintln!("V1 row {row}: class spawn {:?} / game spawn {:?}", run(row, 3000, None), run(row, 3000, Some(at)));
    }
}
