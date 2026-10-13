//! WP-A4: SMA state-change signals (FUN_001f13a8) and the NPC animators (npcdraw).
use sly2_recomp::engine::anim::sm::{Sm, Sma, SmaMsg, SmaReceiver, MSGID_SMA_SCRIPT, MSGID_SMA_STATE};
use sly2_recomp::engine::classes::guard::{los, PLAYER_HANDLE};
use sly2_recomp::engine::classes::plugin::EngineHost;
use sly2_recomp::engine::classes::{ClassSet, Inst};
use sly2_recomp::engine::disc::descriptor::Descriptor;
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::npcdraw::*;
use sly2_recomp::engine::physics::driven::GameRng;
use sly2_recomp::engine::playerdraw::aseg_of;
use sly2_recomp::engine::world::{LoadOptions, World};
use std::sync::Arc;

fn levels() -> (LevelLoader, Vec<Descriptor>) {
    crate::common::levels() // public: ISO level table instead of the analysis DB
}

fn setup(w: &Arc<World>) -> (ClassSet, EngineHost) {
    let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
    let mut s = ClassSet::build(w, &mut h);
    s.g.k6_players = vec![(PLAYER_HANDLE, [1.0e9; 3])];
    s.post_load(&mut h);
    s.g.dt = 1.0 / 60.0;
    (s, h)
}

/// FUN_001f13a8: labels of old/new (FUN_001f04f8, -1 outside), message 10 to the owner then every
/// subscriber, message 8 to the script binding; `dispatch` drains the queued changes in order;
/// FUN_001f0660 retargets an idle SMA.
#[test]
fn sma_signals_1f13a8() {
    let pairs = [(10, 30), (20, -1), (30, 0x20)];
    let links = [[10, 20, 8 | 1, 0], [20, 30, 8, 0], [30, 30, 8, 0], [30, 10, 0x10, 0], [20, 10, 8, 0]];
    let link_f = [(1.0, 100), (1.0, 101), (1.0, 102), (1.0, 103), (1.0, 999)];
    let nested = [(5, 100), (6, 101), (7, 102), (8, 103)];
    let sm = Sm::ee_sub_1efac0(&pairs, &links, &link_f, &nested);
    let len = |o: usize| o as f32;
    let mut rng = GameRng::default();

    let mut log: Vec<(&str, i32, SmaMsg)> = vec![];
    {
        let log = std::cell::RefCell::new(&mut log);
        let mut owner = |m: i32, x: &SmaMsg| log.borrow_mut().push(("owner", m, *x));
        let mut sub = |m: i32, x: &SmaMsg| log.borrow_mut().push(("sub", m, *x));
        let mut script = |m: i32, x: &SmaMsg| log.borrow_mut().push(("script", m, *x));
        let r = Sma::ee_sub_1f13a8(&sm, 0, 7, Some(&mut owner), &mut [&mut sub as &mut dyn SmaReceiver], Some(&mut script));
        assert_eq!(r, SmaMsg { old: 10, new: -1 });
    }
    let m = SmaMsg { old: 10, new: -1 };
    assert_eq!(log, vec![("owner", MSGID_SMA_STATE, m), ("sub", MSGID_SMA_STATE, m), ("script", MSGID_SMA_SCRIPT, m)]);

    // start in A (label 10), walk A -> B -> C: queued (-1,A), (A,B), (B,C) arrive as labels in order
    let mut s = Sma::ee_sub_1f01b8(&sm, &mut rng, 10, 1, 8, &len);
    s.ee_sub_1f1078(&sm, &mut rng, &len);
    s.ee_sub_1f1078(&sm, &mut rng, &len);
    let mut got = vec![];
    let mut owner = |id: i32, x: &SmaMsg| got.push((id, x.old, x.new));
    let sent = s.dispatch(&sm, Some(&mut owner), &mut []);
    assert!(s.changes.is_empty());
    assert_eq!(got, vec![(10, -1, 10), (10, 10, 20), (10, 20, 30)]);
    assert_eq!(sent.len(), 3);

    // FUN_001f0660 on an idle SMA (B: next -1 stays): goal A walks at once, B -> A backwards
    let mut s = Sma::ee_sub_1f01b8(&sm, &mut rng, 20, 1, 8, &len);
    assert!(s.asega.is_none());
    s.ee_sub_1f0660(&sm, &mut rng, 10, &len);
    let p = s.asega.clone().unwrap();
    assert_eq!((p.aseg, p.svt, s.goal), (5, -1.0, 0));
    // same goal again: nothing
    s.ee_sub_1f0660(&sm, &mut rng, 10, &len);
    assert_eq!(s.asega.as_ref().unwrap().aseg, 5);
}

fn tr(m: &[[f32; 4]; 4]) -> [f32; 3] {
    [m[3][0], m[3][1], m[3][2]]
}
fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// Every skinned NPC ALO (guards and pickups) on 6 levels runs 600 frames of class update + npcdraw: bone
/// to parent distances stay within 20% of the rest length; a patrolling guard plays its walk anim at the
/// K7 root-motion rate; pickup SMAs start and signal their owner.
#[test]
#[ignore = "heavy"]
fn npc_limbs_and_walk_6_levels() {
    let (loader, descs) = levels();
    let (mut nlev, mut walkers, mut npcs, mut pickups_sm) = (0, 0, 0, 0);
    let mut worst = (0.0f32, 0usize, 0usize);
    for d in descs {
        if nlev == 6 {
            break;
        }
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let (mut s, mut h) = setup(&w);
        if s.k6.is_empty() {
            continue;
        }
        nlev += 1;
        // the level script's guard routines (FUN_00171cf8) start the rail patrols
        for (gi, ri) in patrol_routines(&s, 300.0) {
            s.k6_run_routine(gi, ri, &mut h);
        }
        let mut set = NpcSet::default();
        let mut walked = std::collections::HashSet::new();
        for f in 0..600 {
            s.g.t += s.g.dt;
            s.frame(&mut h);
            set.step(&w, &mut s, &mut h, 1.0 / 60.0);
            for (&o, a) in &set.npcs {
                if let NpcAnim::Loop { id: GUARD_ANIM_WALK, rate } = a.anim {
                    assert!(rate.is_finite() && rate > 0.0 && rate < 10.0, "guard {o} walk rate {rate}");
                    let walk = anim_object(&w, o, GUARD_ANIM_WALK);
                    if a.aseg().is_some() {
                        assert_eq!(a.aseg(), walk, "guard {o} plays its walk ASEG");
                        walked.insert(o);
                    }
                }
                if f % 10 != 0 {
                    continue;
                }
                let rest = &a.rig.rest_world;
                let wm = a.rig.pose.world();
                for (i, b) in a.rig.pose.bones.iter().enumerate() {
                    let Some(p) = b.parent else { continue };
                    if matches!(b.kind, 1 | 4 | 5 | 9 | 10) {
                        continue;
                    }
                    let (d0, d1) = (dist(tr(&rest[i]), tr(&rest[p])), dist(tr(&wm[i]), tr(&wm[p])));
                    let e = (d1 - d0).abs() / (d0 + 1.0);
                    if e > worst.0 {
                        worst = (e, o, i);
                    }
                }
            }
        }
        npcs += set.npcs.len();
        walkers += walked.len();
        pickups_sm += set.npcs.values().filter(|a| a.sm.is_some()).count();
        println!("lsn {}: npcs {} guards {} walking {} sm {}", d.lsn(), set.npcs.len(), s.k6.len(), walked.len(), pickups_sm);
    }
    println!("levels {nlev} npcs {npcs} walkers {walkers} worst limb change {worst:?}");
    assert_eq!(nlev, 6);
    assert!(npcs > 20);
    assert!(walkers > 0, "a patrolling guard plays its walk anim");
    assert!(worst.0 < 0.2, "limb length changed {worst:?}");
}

/// The walk ASEG of a guard is the anim-table entry 9 the K7 root motion reads (los::anim_oid), and the
/// animator's rate makes its root speed equal the body speed.
#[test]
fn guard_walk_rate_matches_root_motion() {
    let (loader, descs) = levels();
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let (s, _) = setup(&w);
        for (gi, x) in &s.k6 {
            let Inst::Guard(g) = x else { continue };
            let Some(a) = anim_object(&w, *gi, GUARD_ANIM_WALK) else { continue };
            if aseg_of(&w, a).is_none() || los::anim_oid(&w, *gi, GUARD_ANIM_WALK) != Some(w.objects[a].oid) {
                continue;
            }
            let mut g = g.clone();
            let r = los::root_velocity(&w, *gi, GUARD_ANIM_WALK, 0.2, 1.0);
            let r1 = (r[0] * r[0] + r[1] * r[1]).sqrt();
            if r1 < 1.0 {
                continue;
            }
            // body moving at 1.5x the anim's own root speed -> rate 1.5
            g.body.f_190 = [r[0] * 1.5, r[1] * 1.5, 0.0];
            let NpcAnim::Loop { id, rate } = guard_anim(&w, *gi, &g, 0.0, 0.2) else { panic!() };
            assert_eq!(id, GUARD_ANIM_WALK);
            assert!((rate - 1.5).abs() < 1.0e-3, "rate {rate}");
            return;
        }
    }
    panic!("no guard with a walk anim");
}
