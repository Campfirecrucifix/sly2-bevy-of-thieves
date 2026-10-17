//! WP-K8: the pickup family (classes 91-95; classes/k8.rs) on the real levels: build + 300 frames on all 43
//! levels, a clue-bottle pickup incrementing the bottle counter DAT_003e1bf4, coin expiry, hit-filter collect.
use sly2_recomp::engine::classes::guard::PLAYER_HANDLE;
use sly2_recomp::engine::classes::k8::{self, PickRec, Pickup};
use sly2_recomp::engine::classes::plugin::EngineHost;
use sly2_recomp::engine::classes::{ClassSet, Inst};
use sly2_recomp::engine::disc::descriptor::Descriptor;
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::world::{LoadOptions, World};
use std::sync::Arc;

fn levels() -> (LevelLoader, Vec<Descriptor>) {
    crate::common::levels() // public: ISO level table instead of the analysis DB
}

fn setup(w: &Arc<World>, sly: [f32; 3]) -> (ClassSet, EngineHost) {
    let mut h = EngineHost { world: Some(w.clone()), player_pos: sly, ..Default::default() };
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
fn pickups(s: &ClassSet) -> Vec<(usize, Pickup)> {
    s.k8.iter().filter_map(|(i, x)| if let Inst::K8(c) = x { Some((*i, (**c).clone())) } else { None }).collect()
}
const FAR: [f32; 3] = [1.0e9; 3];

/// Every K8 class builds on all 43 levels (init / load-post state 0) and runs 300 frames.
#[test]
#[ignore = "heavy"]
fn k8_classes_all_levels_300_frames() {
    let (loader, descs) = levels();
    assert_eq!(descs.len(), 43);
    let mut counts = std::collections::BTreeMap::<u16, usize>::new();
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let (mut s, mut h) = setup(&w, FAR);
        for (i, o) in w.objects.iter().enumerate() {
            if k8::PORTED_K8.iter().any(|x| x.0 == o.class) {
                assert!(s.k8.iter().any(|x| x.0 == i), "class {} object {i} built", o.class);
                assert!(k8::slot5(o.class).is_some());
            }
        }
        for (_, c) in pickups(&s) {
            assert_eq!(c.f_150, 0, "class {} starts idle", c.class);
            assert_eq!(c.f_1c8, 1);
            assert_eq!(c.f_168, Some(k8::ANIM_IDLE));
            *counts.entry(c.class).or_default() += 1;
        }
        for _ in 0..300 {
            step(&mut s, &mut h);
        }
        for (_, c) in pickups(&s) {
            assert!(c.f_170.iter().all(|v| v.is_finite()));
            // nobody collected anything with Sly far away
            assert_ne!(c.f_150, 2);
        }
        assert_eq!(s.g.g_3e1bf4, 0);
    }
    println!("K8 instances: {counts:?}");
    for c in [91, 92, 93, 94, 95] {
        assert!(counts.get(&c).copied().unwrap_or(0) > 0, "class {c} present");
    }
}

/// Clue bottle (class 95) attached to a class-7 host: Sly walking into it collects it (FUN_001c5d18 ->
/// SetState(1) FUN_001c7ee8) and the bottle counter DAT_003e1bf4 counts the host serial bits.
#[test]
fn k8_clue_bottle_pickup_counts() {
    let (loader, descs) = levels();
    let d = descs.into_iter().find(|d| d.lsn() == 353856).unwrap();
    let lv = crate::common::load_blocking(&loader, d).unwrap();
    let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
    let (mut s, mut h) = setup(&w, FAR);
    h.player_rec = Some(PickRec { alive: 1, radius: 60.0, obj: 0 });
    let hosts: Vec<usize> = w.objects.iter().enumerate().filter(|(_, o)| o.class == 7).map(|(i, _)| i).collect();
    let bottles: Vec<usize> = pickups(&s).into_iter().filter(|(_, c)| c.class == 95).map(|(i, _)| i).collect();
    assert!(hosts.len() >= 2 && bottles.len() >= 2, "hosts {} bottles {}", hosts.len(), bottles.len());
    for (n, (&b, &host)) in bottles.iter().zip(hosts.iter()).take(2).enumerate() {
        s.k8_attach(b, host as u32 + 1, &mut h);
        let c = s.k8_pickup(b).unwrap();
        assert!(c.in_world);
        assert_eq!(c.owner_100, Some(n as i32), "class-7 serials from DAT_003e1bf0");
        // Sly out of reach: nothing happens
        step(&mut s, &mut h);
        assert_eq!(s.g.g_3e1bf4, n as i32);
        // Sly at the bottle
        let p = s.k8_pickup(b).unwrap().xf_pos;
        let sly = [p[0] + 30.0, p[1], p[2]];
        s.g.k6_players = vec![(PLAYER_HANDLE, sly)];
        h.player_pos = sly;
        step(&mut s, &mut h);
        let c = s.k8_pickup(b).unwrap();
        assert_eq!(c.f_150, 2, "collected");
        assert!(!c.in_world);
        assert_eq!(c.f_168, None, "anim stopped");
        assert_eq!(s.g.g_3e1bf4, n as i32 + 1, "bottle counter");
        h.player_pos = FAR;
        s.g.k6_players = vec![(PLAYER_HANDLE, FAR)];
    }
    // the same serial again does not count twice
    k8::ee_sub_1c7c30(&mut s.g, 0, true);
    assert_eq!(s.g.g_3e1bf4, 2);
}

/// Coin (class 91): idle timer rand(5, 5) -> vanish state 5 after 5 s; a hit with +0x44 bit 3 (slot 74)
/// collects at once.
#[test]
fn k8_coin_expiry_and_hit() {
    let (loader, descs) = levels();
    let d = descs.into_iter().find(|d| d.lsn() == 353856).unwrap();
    let lv = crate::common::load_blocking(&loader, d).unwrap();
    let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
    let (mut s, mut h) = setup(&w, FAR);
    // coins are all template clones outside the world until dropped
    let all = pickups(&s);
    let live = all.iter().find(|(_, c)| c.class == 91).expect("coin").0;
    let other = all.iter().find(|(i, c)| c.class == 91 && *i != live).expect("second coin").0;
    // a dropper puts the coin into the world (slot 25 FUN_001cb518, no host)
    s.k8_attach(live, 0, &mut h);
    assert!(s.k8_pickup(live).unwrap().in_world);
    let coins = [live, other];
    assert!((s.k8_pickup(coins[0]).unwrap().f_238 - 5.0).abs() < 1e-4);
    for _ in 0..290 {
        step(&mut s, &mut h);
    }
    assert_eq!(s.k8_pickup(coins[0]).unwrap().f_150, 0);
    for _ in 0..20 {
        step(&mut s, &mut h);
    }
    let c = s.k8_pickup(coins[0]).unwrap();
    assert_eq!(c.f_150, 5);
    assert_eq!(c.f_168, Some(k8::ANIM_VANISH));
    // slot 74 hit filter
    let mut c = s.k8_pickup(coins[1]).unwrap().clone();
    let mut cx = k8::Cx { g: &mut s.g, rng: &mut s.rng };
    assert!(!c.ee_sub_1c5df0(coins[1], 0, &mut cx, &mut h));
    assert!(c.ee_sub_1c5df0(coins[1], 8, &mut cx, &mut h));
    assert_eq!(c.f_150, 2);
}
