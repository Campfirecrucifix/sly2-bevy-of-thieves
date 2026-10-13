//! WP-K32: guard hit reactions (brain slot 12 FUN_00176be8, substates 7..0x13 of FUN_00177618 / FUN_00176750)
//! on Episode 1 (Paris, level-table rows 2..7) guards: stagger -> recover, knock-out (permanent).
use sly2_recomp::engine::classes::guard::brain::Child;
use sly2_recomp::engine::classes::guard::c56::Guard;
use sly2_recomp::engine::classes::guard::slots::GuardHit;
use sly2_recomp::engine::classes::guard::PLAYER_HANDLE;
use sly2_recomp::engine::classes::plugin::EngineHost;
use sly2_recomp::engine::classes::ClassSet;
use sly2_recomp::engine::disc::descriptor::Descriptor;
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::physics::query::CollisionWorld;
use sly2_recomp::engine::player::p11_cane as cane;
use sly2_recomp::engine::player::p6::P6Level;
use sly2_recomp::engine::player::{Jt, JtInput};
use sly2_recomp::engine::world::{LoadOptions, World};
use std::sync::Arc;

const DT: f32 = 1.0 / 60.0;

fn levels() -> (LevelLoader, Vec<Descriptor>) {
    crate::common::levels() // public: ISO level table instead of the analysis DB
}

/// The first Episode 1 level (rows 2..7) with load-placed guards: (row, world, host, class set)
fn ep1_level() -> (usize, Arc<World>, EngineHost, ClassSet) {
    let (loader, descs) = levels();
    for row in 2..=7 {
        let lv = crate::common::load_blocking(&loader, descs[row]).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        s.g.k6_players = vec![(PLAYER_HANDLE, [1.0e9; 3])];
        s.post_load(&mut h);
        s.g.dt = DT;
        if !s.k6.is_empty() {
            return (row, w, h, s);
        }
    }
    panic!("no Episode 1 level with load-placed guards");
}

fn frame(s: &mut ClassSet, h: &mut EngineHost) {
    s.g.t += DT;
    s.frame(h);
}

fn cane_hit(gi: usize, g: &Guard) -> GuardHit {
    // FUN_001f7410_cane record (p11_cane): kind 2, amount -10, response +0x44 = 0x20, vector Sly -> guard
    let p = g.body.pos;
    GuardHit { src: PLAYER_HANDLE, target: gi as u32 + 1, kind: 2, amount: -10, pos: [p[0], p[1], p[2], 1.0], vec: [1.0, 0.0, 0.0, 0.0], f_44: 0x20 }
}

/// FUN_00176be8 on a default guard: lethal (slot 85: +0x338 <= -amount) -> knocked out; 0x10-only (flinch)
/// keeps the substate; kind masks gate hits while down.
#[test]
fn k32_hit_rules_pure() {
    let mut h = EngineHost::default();
    let mut g = Guard::default();
    g.init(0, &mut h);
    g.f_338 = 40;
    g.ee_sub_206518(0, &[], 0.0, &mut h);
    assert_eq!(g.f_33c, 40);
    g.anims = vec![5, 6, 7];
    // flinch only: +0x340 armour 0 -> 0x20 is added -> health hit + stagger
    let mut hit = GuardHit { src: 7, target: 1, kind: 2, amount: -10, f_44: 0x10, ..Default::default() };
    assert!(g.ee_sub_176be8(0, &hit, 1.0, DT, &mut h));
    assert_eq!(g.f_33c, 30);
    assert!(matches!(g.brain.sub.code, 9 | 10), "0x10 without armour -> stagger");
    // response 0 -> handled, no reaction
    let sub = g.brain.sub.code;
    hit.f_44 = 0;
    g.ee_sub_176be8(0, &hit, 1.1, DT, &mut h);
    assert_eq!((g.brain.sub.code, g.f_33c), (sub, 30));
    // +0x29c: no damage
    g.f_29c = 1;
    hit.f_44 = 0x20;
    g.ee_sub_176be8(0, &hit, 1.2, DT, &mut h);
    assert_eq!(g.f_33c, 30);
    g.f_29c = 0;
    // lethal
    let big = GuardHit { amount: -40, ..hit.clone() };
    g.ee_sub_176be8(0, &big, 1.3, DT, &mut h);
    assert!(g.knocked_out() && g.f_330 == 1);
    assert!(matches!(&g.brain.child, Child::Hit(c) if c.bit == 7 && c.hit.f_44 == 0x40));
    // down: kind 2 is not in the +0x358 mask (0xf800) -> ignored
    g.ee_sub_176be8(0, &hit, 1.4, DT, &mut h);
    assert!(g.knocked_out());
    assert_eq!(g.f_33c, 30);
}

/// An Episode 1 guard runs 300 frames: routine (sub 0 -> 1), a cane hit at frame 100 -> heavy stagger
/// (sub 10, anim 6, bit 6, frozen) -> recovers through FUN_00177ba0 (chase 4 / routine 1).
#[test]
fn k32_ep1_guard_300_frames_hit_and_recover() {
    let (row, _w, mut h, mut s) = ep1_level();
    let gi = s.k6[0].0;
    assert_eq!(s.k6_guard(gi).unwrap().brain.sub.code, 0);
    let mut subs = vec![0];
    let mut hit_at = None;
    let mut rec_at = None;
    let len6 = {
        use sly2_recomp::engine::classes::ClassHost;
        h.anim_len(gi, 6)
    };
    for f in 0..300 {
        if f == 100 {
            let g = s.k6_guard(gi).unwrap();
            assert_eq!(g.brain.sub.code, 1, "patrol routine before the hit");
            let hit = cane_hit(gi, g);
            assert!(s.ee_sub_1f7598(&hit, &mut h), "slot 74 takes the hit");
        }
        frame(&mut s, &mut h);
        let g = s.k6_guard(gi).unwrap();
        let c = g.brain.sub.code;
        if *subs.last().unwrap() != c {
            subs.push(c);
        }
        if f == 100 {
            assert_eq!(c, 10, "cane hit -> heavy stagger (FUN_0020a298 -> 0x80, anim 6 in the table)");
            assert!(matches!(&g.brain.child, Child::Hit(x) if x.bit == 6));
            assert_eq!(g.f_33c, 30, "health 40 - 10");
            assert!(g.f_2c0 & (1 << 6) != 0 && g.hit_fx == 1);
            hit_at = Some((s.g.t, g.body.pos));
        }
        if let Some((t0, p0)) = hit_at {
            if rec_at.is_none() && c == 10 {
                assert_eq!(g.body.f_190, [0.0; 3]);
                assert_eq!(g.body.pos, p0, "no walk while staggered");
            } else if rec_at.is_none() {
                rec_at = Some(s.g.t - t0);
                assert!(matches!(c, 1 | 4), "recover -> chase / routine, got {c}");
            }
        }
    }
    let rec = rec_at.expect("stagger ends");
    assert!((rec - len6).abs() < 2.0 * DT + 1.0e-3, "stagger lasts anim 6 ({len6} s), got {rec}");
    assert!(!s.k6_guard(gi).unwrap().knocked_out());
    println!("row {row} guard {gi}: substates {subs:?}, stagger {rec:.3} s (anim 6 {len6:.3} s)");
    assert!(subs.starts_with(&[0, 1, 10]), "{subs:?}");
}

/// Scripted cane hits -> stagger -> recover, then hits until health 0 -> knocked out (sub 0xb, anim 7,
/// stun birds flag), down for good: FUN_00177618 has no case 0xb and later cane hits are masked.
#[test]
fn k32_ep1_scripted_hits_knock_out() {
    let (row, _w, mut h, mut s) = ep1_level();
    let gi = s.k6[0].0;
    for _ in 0..10 {
        frame(&mut s, &mut h);
    }
    let hit = |s: &mut ClassSet, h: &mut EngineHost| {
        let x = cane_hit(gi, s.k6_guard(gi).unwrap());
        s.ee_sub_1f7598(&x, h);
        frame(s, h);
    };
    // a light stagger (sub 9) ignores kind 2 (not in +0x350, +0x110 byte 0): every hit waits for the recovery
    let mut frames = vec![];
    for k in 0..4 {
        let mut n = 0;
        while s.k6_guard(gi).unwrap().staggered() {
            frame(&mut s, &mut h);
            n += 1;
            assert!(n < 600, "stagger never ends");
        }
        if k > 0 {
            frames.push(n);
            assert!(matches!(s.k6_guard(gi).unwrap().brain.sub.code, 1 | 4), "recovered");
        }
        hit(&mut s, &mut h);
        let g = s.k6_guard(gi).unwrap();
        assert_eq!(g.f_33c, 30 - 10 * k);
        assert_eq!(g.knocked_out(), k == 3, "hit {k}");
        assert_eq!(g.staggered(), k < 3, "hit {k}");
        if k == 0 {
            assert_eq!(g.brain.sub.code, 10, "first hit: no sensor entry -> heavy");
        }
    }
    let n = frames;
    let g = s.k6_guard(gi).unwrap();
    assert!(matches!(&g.brain.child, Child::Hit(x) if x.bit == 7) && g.f_330 == 1 && g.f_2c0 & (1 << 7) != 0);
    assert!(g.sensor.entries.iter().all(|e| e.state == 0), "FUN_00153ff8");
    let p0 = g.body.pos;
    for _ in 0..300 {
        frame(&mut s, &mut h);
    }
    hit(&mut s, &mut h);
    let g = s.k6_guard(gi).unwrap();
    assert!(g.knocked_out() && g.body.pos == p0, "stays down");
    println!("row {row} guard {gi}: stagger frames {n:?}, knocked out on the 4th cane hit");
}

/// The P11 cane swing (FUN_001f7410_cane -> FUN_001f7598 -> slot 74) knocks an Episode 1 guard into its
/// hit reaction, and repeated swings knock it out.
#[test]
fn k32_ep1_cane_swing_knocks_out() {
    let (row, w, mut h, mut s) = ep1_level();
    let p6 = P6Level::load(&w);
    assert!(p6.cane.geom.is_some(), "Sly's cane geom in row {row}");
    let gi = s.k6[0].0;
    let step = |jt: &mut Jt, held: u32, pressed: u32| {
        jt.pad_pending = Some((held, pressed));
        jt.update(&CollisionWorld::default(), JtInput::default(), DT, &mut ());
    };
    let mut swings = 0;
    while !s.k6_guard(gi).unwrap().knocked_out() {
        swings += 1;
        assert!(swings <= 8, "not knocked out after 8 swings");
        let mut n = 0;
        while s.k6_guard(gi).unwrap().staggered() {
            frame(&mut s, &mut h);
            n += 1;
            assert!(n < 600);
        }
        let hp = s.k6_guard(gi).unwrap().f_33c;
        let gpos = s.k6_guard(gi).unwrap().body.pos;
        let at = [gpos[0] - 120.0, gpos[1], gpos[2] + 30.0];
        let mut jt = Jt::new(at);
        p6.apply(&mut jt);
        jt.t = 10.0;
        jt.t_ground = 1.0e9;
        step(&mut jt, 0x80, 0x80);
        let mut hit = false;
        let mut react = false;
        for _ in 0..60 {
            hit |= cane::cane_hits_to_guards(&mut jt, &mut s, &mut h, &mut ()).iter().any(|x| x.target == gi as u32 + 1);
            if jt.cane.jtcs == -1 {
                break;
            }
            step(&mut jt, 0, 0);
            jt.phys.pos = at;
            frame(&mut s, &mut h);
            let g = s.k6_guard(gi).unwrap();
            react |= g.staggered() || g.knocked_out();
        }
        assert!(hit, "swing {swings} hit the guard");
        frame(&mut s, &mut h);
        let g = s.k6_guard(gi).unwrap();
        assert!(react || g.staggered() || g.knocked_out(), "swing {swings}: reaction, sub {}", g.brain.sub.code);
        assert_eq!(g.f_33c, hp - 10, "swing {swings}: one hit per swing");
    }
    println!("row {row} guard {gi}: knocked out after {swings} cane swings");
}

#[test]
fn k32_probe_ep1_guards() {
    let (row, _w, _h, s) = ep1_level();
    for (o, _) in &s.k6 {
        let g = s.k6_guard(*o).unwrap();
        println!(
            "row {row} guard {o} class {} f_338 {} f_33c {} f_340 {} masks {:x}/{:x}/{:x} f_29c {} f_4a4 {} anims {:?} sub {}",
            g.class, g.f_338, g.f_33c, g.f_340, g.f_350, g.f_354, g.f_358, g.f_29c, g.f_4a4, g.anims, g.brain.sub.code
        );
    }
}
