//! WP-K6: guards (class 56 family, concrete 57-67) on the real levels.
use sly2_recomp::engine::classes::guard::patrol::Cmd;
use sly2_recomp::engine::classes::guard::{sensor, PLAYER_HANDLE, PORTED_K6};
use sly2_recomp::engine::classes::plugin::EngineHost;
use sly2_recomp::engine::classes::{ClassSet, Inst};
use sly2_recomp::engine::disc::descriptor::Descriptor;
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::gfx::scene::invert;
use sly2_recomp::engine::player::p5_attach::FUN_001e6f60;
use sly2_recomp::engine::world::{LoadOptions, World};
use std::sync::Arc;

fn levels() -> (LevelLoader, Vec<Descriptor>) {
    crate::common::levels() // public: ISO level table instead of the analysis DB
}

fn setup(w: &Arc<World>, sly: [f32; 3]) -> (ClassSet, EngineHost) {
    let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
    let mut s = ClassSet::build(w, &mut h);
    s.g.k6_players = vec![(PLAYER_HANDLE, sly)];
    s.post_load(&mut h);
    s.g.dt = 1.0 / 60.0;
    (s, h)
}
fn step(s: &mut ClassSet, h: &mut EngineHost) {
    s.g.t += s.g.dt;
    s.frame(h);
}
const FAR: [f32; 3] = [1.0e9; 3];

#[test]
#[ignore = "heavy"]
fn k6_all_levels_build_guards_300_frames() {
    let (loader, descs) = levels();
    let mut total = 0;
    let mut unspawned = 0;
    let mut at_origin = 0;
    let mut levels_with = 0;
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let template: std::collections::HashSet<usize> = w.entries.iter().flat_map(|e| e.instances.iter().copied()).collect();
        let all = w.objects.iter().filter(|o| PORTED_K6.iter().any(|p| p.0 == o.class)).count();
        let expect = (0..w.objects.len())
            .filter(|&i| PORTED_K6.iter().any(|p| p.0 == w.objects[i].class) && (!template.contains(&i) || w.life.placed.contains_key(&i)))
            .count();
        unspawned += all - expect;
        let (mut s, mut h) = setup(&w, FAR);
        assert_eq!(s.k6.len(), expect, "lsn {}", d.lsn());
        for _ in 0..300 {
            step(&mut s, &mut h);
        }
        for (i, x) in &s.k6 {
            let Inst::Guard(g) = x else { panic!() };
            assert!(g.body.pos.iter().all(|v| v.is_finite()), "lsn {} guard {i}", d.lsn());
            assert_eq!(g.f_38c, 0, "no target with Sly far away");
        }
        let origin = s.k6.iter().filter(|(_, x)| matches!(x, Inst::Guard(g) if g.f_300 == [0.0; 3])).count();
        // a few template copies are placed at the origin by the level data itself (staging / scripted)
        at_origin += origin;
        total += expect;
        levels_with += (expect > 0) as usize;
    }
    println!("K6: {total} guards on {levels_with} levels, {unspawned} unspawned template copies");
    println!("{at_origin} placed at the origin");
    assert_eq!(total + unspawned, 469);
    assert!(at_origin * 10 < total, "guards use their +0x12c placement");
}

/// A guard runs a level routine (FUN_00171cf8, as the level script does) whose rail patrol walks it along the
/// rail: its position changes and follows the rail.
#[test]
fn k6_guard_walks_patrol_rail() {
    let (loader, descs) = levels();
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let (mut s, mut h) = setup(&w, FAR);
        // nearest (guard, routine with a rail command) pair, guard able to walk (+0xfe8 & 2)
        let mut best: Option<(f32, usize, usize)> = None;
        for (gi, x) in &s.k6 {
            let Inst::Guard(g) = x else { continue };
            if g.body.f_fe8 & 2 == 0 {
                continue;
            }
            for (ri, c) in s.k6l.cmds.iter().enumerate() {
                for cmd in c {
                    if let Cmd::Rail(Some(p), ..) = cmd {
                        let hit = FUN_001e6f60(p, g.body.pos);
                        let e = (0..3).map(|k| (hit.cp[k] - g.body.pos[k]).powi(2)).sum::<f32>().sqrt();
                        if best.is_none_or(|b| e < b.0) {
                            best = Some((e, *gi, ri));
                        }
                    }
                }
            }
        }
        let Some((e, gi, ri)) = best else { continue };
        if e > 300.0 {
            continue;
        }
        let rail = s.k6l.cmds[ri].iter().find_map(|c| if let Cmd::Rail(Some(p), ..) = c { Some(p.clone()) } else { None }).unwrap();
        let p0 = s.k6_guard(gi).unwrap().body.pos;
        s.k6_run_routine(gi, ri, &mut h);
        let mut max_off: f32 = 0.0;
        for f in 0..300 {
            step(&mut s, &mut h);
            if f > 60 {
                let p = s.k6_guard(gi).unwrap().body.pos;
                let hit = FUN_001e6f60(&rail, p);
                max_off = max_off.max(((hit.cp[0] - p[0]).powi(2) + (hit.cp[1] - p[1]).powi(2)).sqrt());
            }
        }
        let g = s.k6_guard(gi).unwrap();
        let moved = ((g.body.pos[0] - p0[0]).powi(2) + (g.body.pos[1] - p0[1]).powi(2)).sqrt();
        println!("lsn {} guard {gi} routine {ri}: start off-rail {e:.1}, moved {moved:.1}, max off-rail {max_off:.1}, sub {:?}", d.lsn(), g.brain.sub.code);
        assert_eq!(g.brain.sub.code, 5);
        assert!(moved > 200.0, "guard walked: {moved}");
        assert!(max_off < 150.0, "guard stays on its rail: {max_off}");
        return;
    }
    panic!("no level with a guard routine rail");
}

/// A Sly placed inside a guard's vision volume is seen (sensor entry state 1) and the guard brain switches
/// from patrol (substate 1) to the spotted reaction (3) and on to chase (4) with Sly as target.
#[test]
fn k6_guard_spots_sly_in_vision_volume() {
    let (loader, descs) = levels();
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let (mut s, mut h) = setup(&w, FAR);
        for _ in 0..30 {
            step(&mut s, &mut h);
        }
        // a guard with a vision volume and a point inside it
        let mut pick = None;
        for (gi, x) in &s.k6 {
            let Inst::Guard(g) = x else { continue };
            let Some(v) = &g.sensor.f_e8 else { continue };
            let mw = invert(&v.inv);
            let to_w = |l: [f32; 3]| [0, 1, 2].map(|j| l[0] * mw[0][j] + l[1] * mw[1][j] + l[2] * mw[2][j] + mw[3][j]);
            let mut cands = vec![v.c];
            for k in 0..v.geom.verts.len().min(32) {
                let q = v.geom.verts[k];
                cands.push([0, 1, 2].map(|j| (q[j] + v.c[j]) * 0.5));
            }
            if let Some(l) = cands.into_iter().find(|l| sensor::ee_sub_227c90(v, to_w(*l))) {
                let p = to_w(l);
                if g.sensor.f_ec.as_ref().is_none_or(|o| sensor::ee_sub_227c90(o, p)) {
                    pick = Some((*gi, p));
                    break;
                }
            }
        }
        let Some((gi, p)) = pick else { continue };
        let g = s.k6_guard(gi).unwrap();
        assert_eq!(g.brain.sub.code, 1, "patrolling before");
        assert_eq!(g.f_38c, 0);
        // the guard has not moved (home routine at its spawn point): the rest-space test is the world test
        s.g.k6_players = vec![(PLAYER_HANDLE, p)];
        step(&mut s, &mut h);
        let g = s.k6_guard(gi).unwrap();
        let e = g.sensor.entries.iter().find(|e| e.obj == PLAYER_HANDLE).expect("Sly static entry");
        println!("lsn {} guard {gi} at {:?} class {}: Sly at {p:?} -> sensor state {}, brain sub {}", d.lsn(), g.body.pos, g.class, e.state, g.brain.sub.code);
        assert_eq!(e.state, 1, "seen");
        assert_eq!(g.f_38c, PLAYER_HANDLE, "target = Sly");
        assert!(matches!(g.brain.sub.code, 3 | 4), "spotted -> reaction / chase");
        step(&mut s, &mut h);
        assert_eq!(s.k6_guard(gi).unwrap().brain.sub.code, 4);
        // Sly leaves both volumes: lost -> back to patrol
        s.g.k6_players = vec![(PLAYER_HANDLE, FAR)];
        step(&mut s, &mut h);
        step(&mut s, &mut h);
        let g = s.k6_guard(gi).unwrap();
        assert_eq!(g.sensor.entries.iter().find(|e| e.obj == PLAYER_HANDLE).unwrap().state, 0);
        assert_eq!(g.brain.sub.code, 1);
        return;
    }
    panic!("no guard with a vision volume");
}
