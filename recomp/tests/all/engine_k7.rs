//! WP-K7: guard occlusion (blockers / LOS / concealment), the remaining class-56 slots, guard hits on Sly,
//! anim root motion, and the K7 classes on the real levels.
use sly2_recomp::engine::classes::guard::los::{self, Cover};
use sly2_recomp::engine::classes::guard::slots::{Contact, GuardHit, Stimulus};
use sly2_recomp::engine::classes::guard::{c56::Guard, sensor, PLAYER_HANDLE};
use sly2_recomp::engine::classes::plugin::EngineHost;
use sly2_recomp::engine::classes::{ClassHost, ClassSet, Inst};
use sly2_recomp::engine::disc::descriptor::Descriptor;
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::gfx::scene::invert;
use sly2_recomp::engine::player::p7_damage as dmg;
use sly2_recomp::engine::player::Jt;
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
fn k7_slots_pure() {
    let mut g = Guard::default();
    let hit = |kind, amount| GuardHit { kind, amount, ..Default::default() };
    // slot 85: +0x338 <= -amount, kinds 0 / 1 / 0x80000 / 0x100000 never lethal
    g.f_338 = 20;
    assert!(g.ee_sub_207530(&hit(0x200, -20)));
    assert!(!g.ee_sub_207530(&hit(0x200, -19)));
    assert!(!g.ee_sub_207530(&hit(0x80000, -100)));
    // slot 75 / 76 / 84 behaviour bits
    assert!(!g.ee_sub_207598(0));
    g.f_2c0 = 1 << 0x2a;
    assert!(g.ee_sub_207598(0));
    g.f_2c0 = 1 << 5;
    assert_eq!(g.ee_sub_20b3d0(), 1);
    g.f_2c0 = 1 << 7 | 1 << 5;
    assert_eq!(g.ee_sub_20b3d0(), 2);
    g.f_2c0 = 1 << 0x13;
    assert_eq!(g.ee_sub_20b090(|| 7), 0);
    g.f_2c0 = 1 << 0x14 | 1 << 0x13;
    assert_eq!(g.ee_sub_20b090(|| 7), 1);
    g.f_2c0 = 0;
    assert_eq!(g.ee_sub_20b090(|| 7), 7);
    // slot 60
    assert_eq!(g.ee_sub_206380(4, false, None), 4);
    g.f_2c0 = 1 << 7;
    assert_eq!(g.ee_sub_206380(4, false, None), 5);
    assert_eq!(g.ee_sub_206380(4, true, None), 4);
    g.f_2c0 = 1 << 0x17;
    assert_eq!(g.ee_sub_206380(4, false, Some((true, 0))), 5);
    assert_eq!(g.ee_sub_206380(4, false, Some((true, 2))), 4);
    assert_eq!(g.ee_sub_206380(4, false, Some((true, 5))), 5);
    assert_eq!(g.ee_sub_206380(4, false, Some((false, 5))), 4);
    // slot 74: non-lethal, not immune, +0x44 == 0 -> refused; +0x44 -> taken (goal message)
    let mut h = EngineHost::default();
    g.f_2c0 = 0;
    assert!(!g.ee_sub_207468(0, &hit(0x200, -1), false, &mut h));
    assert!(g.ee_sub_207468(0, &hit(0x200, -1), true, &mut h));
    assert!(g.f_damage.is_empty());
    assert!(g.ee_sub_207468(0, &GuardHit { kind: 0x200, amount: -1, f_44: 1, ..Default::default() }, false, &mut h));
    assert!(g.ee_sub_207468(0, &hit(0x200, -30), false, &mut h)); // lethal
    assert_eq!(g.f_damage.len(), 2);
    // slot 4 tracker weapon, slots 21 / 22 guard list
    g.ee_sub_2075d0(0, 4, 77, 1, 5, &mut h);
    assert_eq!(g.f_4a8, 77);
    g.ee_sub_2075d0(0, 5, 78, 1, 0, &mut h);
    assert_eq!(g.f_4a8, 77);
    g.ee_sub_2075d0(0, 5, 77, 1, 0, &mut h);
    assert_eq!(g.f_4a8, 0);
    let mut live = Vec::new();
    g.ee_sub_2079e0(0, &mut live, &mut h);
    g.ee_sub_2079e0(0, &mut live, &mut h);
    assert_eq!((live.clone(), g.f_ff8), (vec![1], f32::MAX));
    g.ee_sub_2079a0(0, &mut live, 3.5, &mut h);
    assert_eq!((live.len(), g.f_ff8), (0, 3.5));
    // slot 88: gate, level, own stimulus, sound -> hear
    g.f_e68 = 2;
    let st = Stimulus { level: 3, mode: 0, ..Default::default() };
    assert!(g.ee_sub_208be0(0, &st, true, [0.0; 3], || true, |_, _| false));
    assert!(!g.ee_sub_208be0(0, &st, false, [0.0; 3], || true, |_, _| false));
    assert!(!g.ee_sub_208be0(0, &Stimulus { level: 1, ..st.clone() }, true, [0.0; 3], || true, |_, _| false));
    assert!(!g.ee_sub_208be0(0, &Stimulus { src: 1, ..st.clone() }, true, [0.0; 3], || true, |_, _| false));
    assert!(!g.ee_sub_208be0(0, &Stimulus { mode: 1, ..st.clone() }, true, [0.0; 3], || true, |_, _| false)); // no vision volume
    // slot 73: only attack parts with an accepting target
    let mut out = Vec::new();
    let cs = [Contact { kind: 0, other: PLAYER_HANDLE, ..Default::default() }, Contact { kind: 0x200, amount: -10, other: 9, ..Default::default() }, Contact { kind: 0x200, amount: -10, other: PLAYER_HANDLE, ..Default::default() }];
    Guard::ee_sub_1f7410(4, &cs, &mut |o, _| if o == PLAYER_HANDLE { o } else { 0 }, &mut |x| out.push(x));
    assert_eq!(out, vec![GuardHit { src: 5, target: PLAYER_HANDLE, kind: 0x200, amount: -10, ..Default::default() }]);
}

#[test]
fn k7_concealment_154770() {
    let c = |st, sub| Cover { st, sub, dir: [1.0, 0.0, 0.0], ..Default::default() };
    assert!(!los::ee_sub_154770(&c(2, -1), [0.0; 3], [0.0; 3]));
    assert!(los::ee_sub_154770(&c(2, 0), [0.0; 3], [0.0; 3]));
    assert!(los::ee_sub_154770(&c(1, 0), [0.0; 3], [0.0; 3]));
    assert!(!los::ee_sub_154770(&c(1, 1), [0.0; 3], [0.0; 3]));
    assert!(los::ee_sub_154770(&c(8, 10), [0.0; 3], [0.0; 3]));
    assert!(!los::ee_sub_154770(&c(8, 11), [0.0; 3], [0.0; 3]));
    // state 0 against a cover object (is-a 0xc, +0x194 == 0): concealed when the sensor is in front of the facing
    let mut k = Cover { st: 0, sub: 0, att_c: true, dir: [1.0, 0.0, 0.0], ..Default::default() };
    assert!(!los::ee_sub_154770(&k, [0.0; 3], [100.0, 0.0, 0.0])); // d = -100 <= 0, b true -> true != true
    assert!(los::ee_sub_154770(&k, [0.0; 3], [-100.0, 0.0, 0.0]));
    k.f_1040 = 1;
    assert!(!los::ee_sub_154770(&k, [0.0; 3], [-100.0, 0.0, 0.0]));
    assert!(los::ee_sub_154770(&k, [0.0; 3], [100.0, 0.0, 0.0]));
    k.att_v = 3;
    assert!(!los::ee_sub_154770(&k, [0.0; 3], [-100.0, 0.0, 0.0]));
    // no cover object: +0xe4 == 0
    assert!(los::ee_sub_154770(&Cover { st: 0xd, sub: 0, ..Default::default() }, [0.0; 3], [0.0; 3]));
}

/// Points inside a guard's vision volume: one with a clear eye line is seen; one with level geometry (or a
/// class-23 blocker) between the eye and Sly is not, and the guard keeps patrolling.
#[test]
fn k7_guard_behind_wall_does_not_see_sly() {
    let (loader, descs) = levels();
    let (mut clear_done, mut wall_done) = (false, false);
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let (mut s, mut h) = setup(&w, FAR);
        step(&mut s, &mut h);
        let mut picks = Vec::new();
        for (gi, x) in &s.k6 {
            let Inst::Guard(g) = x else { continue };
            let Some(v) = &g.sensor.f_e8 else { continue };
            if g.sensor.f_f8 & 2 != 0 || g.sensor.f_f0 != 0 {
                continue;
            }
            let mw = invert(&v.inv);
            let to_w = |l: [f32; 3]| [0, 1, 2].map(|j| l[0] * mw[0][j] + l[1] * mw[1][j] + l[2] * mw[2][j] + mw[3][j]);
            let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
            for q in &v.geom.verts {
                for k in 0..3 {
                    lo[k] = lo[k].min(q[k]);
                    hi[k] = hi[k].max(q[k]);
                }
            }
            let eye = [g.body.pos[0], g.body.pos[1], g.body.pos[2] + 80.0];
            let (mut clear, mut wall) = (None, None);
            for a in 0..8 {
                for b in 0..8 {
                    for c in 0..4 {
                        let l = [lo[0] + (hi[0] - lo[0]) * (a as f32 + 0.5) / 8.0, lo[1] + (hi[1] - lo[1]) * (b as f32 + 0.5) / 8.0, lo[2] + (hi[2] - lo[2]) * (c as f32 + 0.5) / 4.0];
                        let p = to_w(l);
                        if !sensor::ee_sub_227c90(v, p) || !g.sensor.f_ec.as_ref().is_none_or(|o| sensor::ee_sub_227c90(o, p)) {
                            continue;
                        }
                        let blk = h.ee_sub_118668_blockers(eye, p) || h.ee_sub_1545f0(*gi, eye, p, g.sensor.f_f8);
                        if blk && wall.is_none() {
                            wall = Some(p);
                        }
                        if !blk && clear.is_none() {
                            clear = Some(p);
                        }
                    }
                }
            }
            picks.push((*gi, clear, wall));
        }
        for (gi, clear, wall) in picks {
            let run = |p: [f32; 3]| {
                let (mut s, mut h) = setup(&w, FAR);
                step(&mut s, &mut h);
                s.g.k6_players = vec![(PLAYER_HANDLE, p)];
                step(&mut s, &mut h);
                step(&mut s, &mut h);
                let g = s.k6_guard(gi).unwrap();
                (g.sensor.entries.iter().find(|e| e.obj == PLAYER_HANDLE).unwrap().state, g.brain.sub.code, g.f_38c)
            };
            if let (Some(p), false) = (wall, wall_done) {
                let (st, sub, t) = run(p);
                println!("lsn {} guard {gi}: Sly behind geometry at {p:?} -> state {st}, sub {sub}", d.lsn());
                assert_eq!((st, t), (0, 0), "not seen through the wall");
                assert_ne!(sub, 4);
                wall_done = true;
            }
            if let (Some(p), false) = (clear, clear_done) {
                let (st, sub, t) = run(p);
                println!("lsn {} guard {gi}: Sly in clear view at {p:?} -> state {st}, sub {sub}", d.lsn());
                assert_eq!((st, t), (1, PLAYER_HANDLE), "seen with a clear eye line");
                clear_done = true;
            }
        }
        if clear_done && wall_done {
            return;
        }
    }
    panic!("no guard with both a clear and an occluded point in its vision volume (clear {clear_done}, wall {wall_done})");
}

/// The global no-detection counter (world +0x3d84, FUN_0020f1b8) hides Sly from every guard.
#[test]
fn k7_no_detection_flag() {
    let mut s = sensor::Sensor { f_f0: 1, ..Default::default() };
    s.ee_sub_153f18(PLAYER_HANDLE, 0.0);
    let tgt = |_: u32| Some(sensor::Tgt { active: true, c33: true, c34: true, ..Default::default() });
    let id = |p: [f32; 3]| p;
    let see = sensor::See { now: 0.0, eye: [0.0; 3], pos: [0.0; 3], to_rest: &id, tgt: &tgt, player_c46: false };
    let mut h = EngineHost::default();
    assert_eq!(s.ee_sub_1541d8(0, PLAYER_HANDLE, 0, &see, &mut h), 1);
    h.world_3d84 = 1;
    assert_eq!(s.ee_sub_1541d8(0, PLAYER_HANDLE, 0, &see, &mut h), 0);
    // sensor state change raises the 0x1c script event
    h.world_3d84 = 0;
    s.ee_sub_153de0(0, &see, &mut h);
    assert_eq!((s.entries[0].state, h.sensor_events), (1, 1));
}

/// A guard attack part touching Sly (slot 73 contact) delivers a hit that lands in the player queue and
/// damages Sly through P7's TakeDamage FUN_001a19a0.
#[test]
fn k7_guard_attack_damages_sly() {
    let (loader, descs) = levels();
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let (mut s, mut h) = setup(&w, FAR);
        let Some(gi) = s.k6.first().map(|x| x.0) else { continue };
        step(&mut s, &mut h);
        h.k7_contacts.push((gi, Contact { kind: 0x200, amount: -10, other: PLAYER_HANDLE, vec: [300.0, 0.0, 400.0, 0.0], ..Default::default() }));
        h.k7_contacts.push((gi, Contact { kind: 0, amount: -10, other: PLAYER_HANDLE, ..Default::default() }));
        step(&mut s, &mut h);
        assert_eq!(h.player_hits.len(), 1, "one hit from the attack part");
        assert!(h.k7_contacts.is_empty());
        let x = h.player_hits.pop().unwrap();
        assert_eq!((x.src, x.target, x.kind, x.amount), (gi as u32 + 1, PLAYER_HANDLE, 0x200, -10));
        let g = s.k6_guard(gi).unwrap();
        let mut jt = Jt::new([0.0, 0.0, 0.0]);
        let hp0 = jt.dmg.hp;
        let env = dmg::DmgEnv::default();
        assert!(dmg::FUN_001a19a0(&mut jt, &x.to_p7(g.body.pos, g.f_fbc), &env, &mut ()));
        println!("lsn {} guard {gi}: hit {x:?} -> Sly hp {hp0} -> {}", d.lsn(), jt.dmg.hp);
        assert!(jt.dmg.hp < hp0, "Sly damaged");
        return;
    }
    panic!("no level with guards");
}

/// FUN_00209618 root motion from the guard's walk ASEG (anim 9) replaces the K6 stand-in speed.
#[test]
fn k7_root_motion_from_walk_anim() {
    let (loader, descs) = levels();
    let mut n = 0;
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let (s, _) = setup(&w, FAR);
        for (gi, _) in &s.k6 {
            if los::anim_oid(&w, *gi, 9).is_none() {
                continue;
            }
            let sp: f32 = (0..20).map(|k| {
                let v = los::root_velocity(&w, *gi, 9, k as f32 * 0.05, 1.0);
                (v[0] * v[0] + v[1] * v[1]).sqrt()
            }).sum::<f32>() / 20.0;
            if n < 3 {
                println!("lsn {} guard {gi} class {}: walk root speed {sp:.1}", d.lsn(), w.objects[*gi].class);
            }
            if sp > 1.0 {
                n += 1;
            }
        }
    }
    println!("guards with walk root motion: {n}");
    assert!(n > 10);
}

/// Every K7 class (PORTED_K7 + stateless) builds on all 43 levels and runs 300 frames; class 45 groups resolve
/// and a highlight activation / release round-trips the two global slots.
#[test]
#[ignore = "heavy"]
fn k7_classes_all_levels_300_frames() {
    use sly2_recomp::engine::classes::k7::{self, K7};
    let (loader, descs) = levels();
    assert_eq!(descs.len(), 43);
    let mut counts = std::collections::BTreeMap::<u16, usize>::new();
    let (mut groups, mut members, mut toggled) = (0, 0, false);
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let (mut s, mut h) = setup(&w, FAR);
        for (i, o) in w.objects.iter().enumerate() {
            if k7::PORTED_K7_STATELESS.iter().any(|x| x.0 == o.class) {
                assert!(k7::slot5(o.class).is_some());
                *counts.entry(o.class).or_default() += 1;
            }
            if k7::PORTED_K7.iter().any(|x| x.0 == o.class) {
                assert!(s.k7.iter().any(|x| x.0 == i), "class {} object {i} built", o.class);
            }
        }
        for (_, x) in &s.k7 {
            let Inst::K7(k) = x else { panic!() };
            let c = match k {
                K7::C2dc070(c) => {
                    assert_eq!(c.f_110, -1);
                    5
                }
                K7::C2db518(c) => {
                    assert!(c.f_ec >= 0.0);
                    18
                }
                K7::C2d4f80(c) => {
                    groups += 1;
                    members += c.f_e4.len();
                    45
                }
                K7::C2d7f40(c) => {
                    assert!(c.f_b0.is_finite());
                    112
                }
                K7::C2daa50(c) => {
                    assert_eq!(c.f_b0, 1);
                    128
                }
            };
            *counts.entry(c).or_default() += 1;
        }
        if !toggled {
            let g = s.k7.iter().find_map(|(i, x)| matches!(x, Inst::K7(K7::C2d4f80(c)) if !c.f_e4.is_empty()).then_some(*i));
            if let Some(g) = g {
                assert_eq!(s.k7_highlight(g, true, &mut h), Some(0));
                assert_eq!(s.g.g_3e1a78, [g as u32 + 1, 0]);
                step(&mut s, &mut h);
                s.k7_highlight(g, false, &mut h);
                assert_eq!(s.g.g_3e1a78, [0, 0]);
                toggled = true;
            }
        }
        for _ in 0..300 {
            step(&mut s, &mut h);
        }
    }
    println!("K7 classes over 43 levels: {counts:?}; class-45 groups {groups}, members {members}");
    for c in [2, 5, 18, 22, 23, 24, 41, 42, 45, 100, 112, 128] {
        assert!(counts.get(&c).copied().unwrap_or(0) > 0, "class {c} present");
    }
    assert!(toggled, "a class-45 group with members");
}
