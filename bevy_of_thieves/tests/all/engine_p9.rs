//! WP-P9: Sly on real levels with the real CollisionWorld (fling reproduction / regression).
use sly2_recomp::engine::disc::descriptor;
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::game::spawn;
use sly2_recomp::engine::input::pad::BTN_CROSS;
use sly2_recomp::engine::physics::CollisionWorld;
use sly2_recomp::engine::player::p6::P6Level;
use sly2_recomp::engine::player::{Jt, JtInput, JT_RADIUS};
use sly2_recomp::engine::world::{LoadOptions, World};
use std::sync::Arc;

const ISO: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../Sly 2 - Band of Thieves (USA).iso");
const DT: f32 = 1.0 / 60.0;

fn level(row: usize) -> Option<Arc<World>> {
    let iso = Arc::new(IsoImage::open(ISO).ok()?);
    let t = descriptor::static_table(&iso.read_file("SCUS_973.16").ok()?)?;
    let l = LevelLoader::new(iso, ModOverlay::default());
    let boot = crate::common::load_blocking(&l, t[25]).ok()?;
    let lv = crate::common::load_blocking(&l, boot.header.level_table.entries[row].desc).ok()?;
    Some(Arc::new(World::load(&lv, LoadOptions::default()).ok()?))
}

fn spawn_of(w: &World) -> Option<[f32; 3]> {
    let (_, c) = spawn::ee_sub_15e9c8(w, None, spawn::CLASS_SPAWN);
    c.into_iter().find_map(|i| spawn::spawn_position(w, i))
}

/// One scripted run: idle 600 frames, run forward 2 s, jump at 720, double jump at 740, 160 more frames.
/// Returns (pos/vel/jts/jtbs per frame, max speed during the idle, max speed overall).
struct Trace {
    spawn: [f32; 3],
    f: Vec<([f32; 3], [f32; 3], i32, i32)>,
}

fn run(row: usize, old: bool) -> Option<Trace> {
    let w = level(row)?;
    let cw = if old { CollisionWorld::build(&w) } else { CollisionWorld::build_so(&w) };
    let p = spawn_of(&w)?;
    // same spawn as player::plugin (spawn + JT_RADIUS + 5)
    let mut jt = Jt::new([p[0], p[1], p[2] + JT_RADIUS + 5.0]);
    P6Level::load(&w).apply(&mut jt);
    let mut f = Vec::new();
    for i in 0..900 {
        let mut inp = JtInput::default();
        if (600..720).contains(&i) {
            inp.stick = [0.0, 1.0];
        }
        let pressed = if i == 720 || i == 740 { BTN_CROSS } else { 0 };
        inp.jump = pressed != 0;
        jt.pad_pending = Some((pressed, pressed));
        jt.update(&cw, inp, DT, &mut ());
        f.push((jt.phys.pos, jt.phys.v, jt.jts, jt.jtbs));
    }
    Some(Trace { spawn: p, f })
}

fn speed(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

const ROWS: [usize; 6] = [0, 2, 3, 10, 20, 27];
fn rows() -> Vec<usize> { std::env::var("P9_ROWS").ok().map(|v| v.split(',').filter_map(|x| x.parse().ok()).collect()).unwrap_or(ROWS.to_vec()) }

/// WP-P9 regression: with the SO collision set Sly never gets flung (speed stays within what the JT states
/// produce: run 515, jump 805/615, fall), stays near the spawn while idle and comes to rest (on the ground or
/// hanging from a ledge, jtbs 0x11) after idle and after the run + jump + double jump.
#[test]
fn no_fling_idle_run_jump() {
    for row in rows() {
        let Some(t) = run(row, false) else { eprintln!("ISO missing"); return };
        let max = t.f.iter().map(|x| speed(x.1)).fold(0.0f32, f32::max);
        let (p6, v6, jts6, jtbs6) = t.f[599];
        let (p9, v9, jts9, jtbs9) = t.f[899];
        eprintln!("row {row}: spawn {:?} f600 {p6:?} jts {jts6}/{jtbs6} end {p9:?} jts {jts9}/{jtbs9} max |v| {max}", t.spawn);
        assert!(t.f.iter().all(|x| x.0.iter().chain(x.1.iter()).all(|c| c.is_finite())), "row {row}: NaN");
        assert!(max < 1500.0, "row {row}: flung, max |v| {max}");
        // idle: at rest, within 50 horizontally of the spawn and not far below / above it
        assert!(speed(v6) < 1.0, "row {row}: not at rest after idle {v6:?}");
        let dh = ((p6[0] - t.spawn[0]).powi(2) + (p6[1] - t.spawn[1]).powi(2)).sqrt();
        assert!(dh < 50.0, "row {row}: drifted {dh} while idle");
        assert!(p6[2] > t.spawn[2] - 600.0 && p6[2] < t.spawn[2] + 700.0, "row {row}: idle z {}", p6[2]);
        assert!(jts6 == 0 || jtbs6 == 0x11, "row {row}: idle state {jts6}/{jtbs6}");
        // run moved Sly (from the ground; row 20 lands next to a floor edge and hangs from it at frame 0, see
        // ENGINE_PORT P9 / REPLACED 0x1997d8), the jumps went up, and he came back down to rest
        let run = ((t.f[719].0[0] - p6[0]).powi(2) + (t.f[719].0[1] - p6[1]).powi(2)).sqrt();
        // WP-P10: with the 1:1 clearance 0x1997d8 row 27 idles on the ground facing a wall (+x): the run is
        // blocked (the jump then hangs on the wall top, jtbs 0x11)
        assert!(jts6 != 0 || run > 100.0 || row == 27, "row {row}: run moved only {run}");
        let apex = t.f[720..].iter().map(|x| x.0[2]).fold(f32::MIN, f32::max);
        assert!(apex > t.f[719].0[2] + 100.0, "row {row}: no jump");
        assert!(speed(v9) < 1.0 && (jts9 == 0 || jtbs9 == 0x11), "row {row}: not landed {v9:?} {jts9}/{jtbs9}");
        assert!(p9[2] > t.spawn[2] - 700.0, "row {row}: fell through ({})", p9[2]);
    }
}

/// The pre-P9 world (`CollisionWorld::build`: class-5 / 22-27 volumes as solids, bone parts untransformed):
/// prints the fling for the record (run with --ignored).
#[test]
#[ignore]
fn trace_old_world() {
    for row in rows() {
        let Some(t) = run(row, true) else { return };
        let max = t.f.iter().map(|x| speed(x.1)).fold(0.0f32, f32::max);
        eprintln!("row {row} OLD: f600 {:?} end {:?} max |v| {max}", t.f[599].0, t.f[899].0);
    }
}

/// P8's water test with the REAL collision world (`build_so`; P8 had to use an empty one): Sly dropped 150
/// above a point inside a class-89 water volume falls through the surface (water parts are not SO collision)
/// and takes the water hit (0x1a2460 -> 0x1f7598).
#[test]
fn real_water_with_real_collision() {
    use sly2_recomp::engine::player::p7_damage::{DmgEv, HP_MAX};
    use sly2_recomp::engine::player::p8_hazard::{self as hz, Hazards, Water, JT_10A0, WATER_KIND};
    fn inside_points(w: &Water) -> Vec<[f32; 3]> {
        let mut out = Vec::new();
        for p in &w.parts {
            let Some((g, _)) = &p.bsp else { continue };
            let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
            for v in &g.verts {
                for k in 0..3 {
                    lo[k] = lo[k].min(v[k]);
                    hi[k] = hi[k].max(v[k]);
                }
            }
            let n = 12;
            for i in 1..n {
                for j in 1..n {
                    let f = |a: usize, d: usize| lo[d] + (hi[d] - lo[d]) * a as f32 / n as f32;
                    if let Some(q) = (1..=4 * n).rev().map(|k| [f(i, 0), f(j, 1), lo[2] + (hi[2] - lo[2]) * k as f32 / (4 * n) as f32 - 1.0]).find(|q| w.FUN_001f72f8(*q)) {
                        out.push(q);
                    }
                }
            }
        }
        out
    }
    let mut done = false;
    for row in [2usize, 4, 8, 10] {
        let Some(w) = level(row) else { return };
        let haz = Hazards::load(&w);
        let cw = CollisionWorld::build_so(&w);
        'w: for wi in (0..haz.waters.len()).filter(|&i| haz.waters[i].amount != 0) {
            for p in inside_points(&haz.waters[wi]) {
                if haz.waters[wi].FUN_001f72f8([p[0], p[1], p[2] + 150.0]) {
                    continue;
                }
                // a clear drop: no solid between the start (and its hull top) and the water point
                if !cw.clip_segment([p[0], p[1], p[2] + 250.0], p).is_empty() {
                    continue;
                }
                let mut h2 = haz.clone();
                let mut jt = Jt::new([p[0], p[1], p[2] - JT_10A0 + 150.0]);
                // Sly's hull (as in the game); the P4 sphere fallback is not the original's contact source
                P6Level::load(&w).apply(&mut jt);
                let mut t = 0.0;
                let mut maxv = 0.0f32;
                for f in 0..240 {
                    jt.update(&cw, JtInput::default(), DT, &mut ());
                    t += DT;
                    maxv = maxv.max(speed(jt.phys.v));
                    hz::hazard_step(&mut jt, &mut h2, t, &mut ());
                    if jt.dmg.hp < HP_MAX || jt.dmg.ev.contains(&DmgEv::Event(0x10)) {
                        if f == 0 {
                            break;
                        }
                        // overlapping volumes: the hit comes from whichever water Sly is in (jt+0x108c)
                        let wj = jt.p8.water.expect("jt+0x108c");
                        assert_eq!((h2.hits[0].src, h2.hits[0].kind), (wj, WATER_KIND));
                        assert!(maxv < 1500.0, "row {row}: flung ({maxv})");
                        eprintln!("row {row}: water {wi}/{wj} hit after {f} frames at {:?} (real collision)", jt.phys.pos);
                        done = true;
                        break 'w;
                    }
                }
            }
        }
        if done {
            break;
        }
    }
    assert!(done, "no water hit with the real collision world");
}


/// WP-P9 jobs: task state machine 0x164df0 over a synthetic chain (props for the prerequisite / dependent
/// lists are not mapped, so real levels have none): finishing A (state 2) closes its prerequisite P (3 =
/// other / no job), opens dependent B (0 -> 1 via 0x165b40), publishes the save variables (0x1643b0 /
/// 0x163460) and the completion percentage 0x161660; 0x163c28 makes B current and 0x163ce0 leaves it.
#[test]
fn task_state_machine() {
    use sly2_recomp::engine::jobs::{Job, JobMgr, Task, VarOwner};
    let mut m = JobMgr { cur_row: -1, ..Default::default() };
    m.jobs.push(Job { tasks: vec![0, 1], f_50: 0, ..Default::default() });
    let mk = |job: Option<usize>, state: i32| Task { job, state, spawn: -1, level: -1, c0: -1, f0: -1, f_5c: -1, ..Default::default() };
    m.tasks = vec![mk(Some(0), 1), mk(Some(0), 0), mk(None, 0)];
    m.tasks[0].prereq = vec![2];
    m.tasks[0].deps = vec![1];
    m.tasks[1].prereq = vec![0];
    m.tasks[2].deps = vec![0];
    m.jobs[0].start = Some(0);
    m.jobs[0].end = Some(1);
    m.FUN_00164df0(0, 2);
    assert_eq!((m.tasks[0].state, m.tasks[1].state, m.tasks[2].state), (2, 1, 3));
    assert!(m.tasks.iter().all(|t| t.f0 == -1) && !m.guard_deb28);
    assert_eq!(m.tasks[0].n_68, 1);
    assert!(m.vars.contains(&(VarOwner::Task(0), 0xe0, 0x15, 2)));
    assert!(m.vars.iter().any(|v| v.0 == VarOwner::Job(0) && v.1 == 0xd8));
    // 2 tasks with dependents (0 and 2), one of them (2) in state 3 -> 50 %
    assert_eq!(m.FUN_00161660(), 50);
    assert!(m.calls.contains(&(0x00161660, 50)));
    m.FUN_00163c28(1);
    assert_eq!(m.cur_task, Some(1));
    assert_eq!(m.tasks[1].n_64, 1);
    m.FUN_00163ce0(1);
    assert_eq!(m.cur_task, None);
    // job end 0x162ea0 resets the counters (0x162940)
    m.jobs[0].n_7c = 3;
    m.FUN_00162940(0);
    assert_eq!((m.jobs[0].n_7c, m.tasks[0].n_64, m.tasks[1].n_64), (0, 0, 0));
}

/// Real row 27 jobs: the restart path now runs 0x164df0(start, 1) + 0x163c28 (the start task becomes the
/// current task) before the level reload.
#[test]
fn row27_restart_enters_start_task() {
    use sly2_recomp::engine::jobs::JobMgr;
    let Some(w) = level(27) else { return };
    let mut m = JobMgr::load(&w);
    let Some(j) = m.jobs.iter().position(|j| j.start.is_some()) else { panic!("no job with a start task") };
    let t = m.jobs[j].start.unwrap();
    assert!(m.jobs[j].tasks.contains(&t));
    m.FUN_00165538(Some(t));
    assert_eq!(m.tasks[t].state, 1);
    assert_eq!(m.cur_task, Some(t));
    assert_eq!(m.reloads.len(), 1);
}
