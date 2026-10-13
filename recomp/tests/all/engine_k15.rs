//! WP-K15: class 73 C2d1080 (chaser; class-71 parts in classes/shared/c71.rs).
use sly2_recomp::engine::classes::plugin::EngineHost;
use sly2_recomp::engine::classes::{ClassSet, Inst};
use sly2_recomp::engine::disc::descriptor::Descriptor;
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::world::{LoadOptions, World};
use std::sync::Arc;

const DT: f32 = 1.0 / 60.0;

fn levels() -> (LevelLoader, Vec<Descriptor>) {
    crate::common::levels() // public: ISO level table instead of the analysis DB
}

/// every level x 300 frames: the class-73 instances stay finite and in a valid state
#[test]
#[ignore = "heavy"]
fn k15_73_all_levels() {
    let (loader, descs) = levels();
    let mut n = 0;
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        s.post_load(&mut h);
        for _ in 0..300 {
            s.g.dt = DT;
            s.g.t += DT;
            s.frame(&mut h);
        }
        for (_, x) in &s.k15 {
            let Inst::K15(c) = x else { continue };
            assert!(c.phys.pos.iter().chain(c.phys.v.iter()).all(|v| v.is_finite()));
            assert!((-1..=4).contains(&c.f_1c0));
            n += 1;
        }
    }
    assert!(n > 0, "no class-73 instance");
    eprintln!("class 73 instances: {n}");
}

/// a real chaser (state 3 enter range 800..9000): with the player 2000 away it picks state 3, replans a path
/// and closes in; a lethal hit takes it to state 4
#[test]
fn k15_73_chase_and_hit() {
    let (loader, descs) = levels();
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        s.post_load(&mut h);
        let Some(i) = s.k15.iter().find_map(|(i, x)| match x {
            Inst::K15(c) if c.ranges[3].has_d && c.ranges[3].d_in == [800.0, 9000.0] => Some(*i),
            _ => None,
        }) else {
            continue;
        };
        let c = s.k15_get(i).unwrap();
        c.f_1d4 = sly2_recomp::engine::classes::guard::PLAYER_HANDLE;
        let p0 = c.phys.pos;
        h.player_pos = [p0[0] + 2000.0, p0[1], p0[2]];
        let dist = |s: &mut ClassSet, h: &EngineHost| {
            let p = s.k15_get(i).unwrap().phys.pos;
            ((p[0] - h.player_pos[0]).powi(2) + (p[1] - h.player_pos[1]).powi(2) + (p[2] - h.player_pos[2]).powi(2)).sqrt()
        };
        let d0 = dist(&mut s, &h);
        for _ in 0..120 {
            s.g.dt = DT;
            s.g.t += DT;
            s.frame(&mut h);
        }
        let c = s.k15_get(i).unwrap();
        assert_eq!(c.f_1c0, 3);
        assert!(c.path.as_ref().map(|p| p.pts.len()) == Some(2));
        let d1 = dist(&mut s, &h);
        eprintln!("chase: {d0} -> {d1}");
        assert!(d1 < d0 - 100.0, "{d0} -> {d1}");
        // lethal hit (+0x44 bit 3)
        let hp = s.k15_get(i).unwrap().b.f_184;
        let hit = sly2_recomp::engine::classes::guard::slots::GuardHit { f_44: 8, amount: -hp, ..Default::default() };
        assert!(s.k15_hit(i, &hit, &mut h));
        assert_eq!(s.k15_get(i).unwrap().f_1c0, 4);
        return;
    }
    panic!("no chaser found");
}

/// every level x 300 frames: the class-101 bomb template is built from its rows and stays put (state -1)
#[test]
#[ignore = "heavy"]
fn k15_101_all_levels() {
    let (loader, descs) = levels();
    let mut n = 0;
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        s.post_load(&mut h);
        let p0: Vec<[f32; 3]> = s.k15.iter().filter_map(|(_, x)| if let Inst::K15b(c) = x { Some(c.phys.pos) } else { None }).collect();
        for _ in 0..300 {
            s.g.dt = DT;
            s.g.t += DT;
            s.frame(&mut h);
        }
        let mut k = 0;
        for (_, x) in &s.k15 {
            let Inst::K15b(c) = x else { continue };
            assert_eq!(c.f_178, -20);
            assert_eq!(c.f_170, 3000.0);
            assert_eq!(c.f_150, -1);
            assert_eq!(c.phys.pos, p0[k]);
            assert!(c.e8_d4 == 2.0 && c.e8_d8 == 2.0);
            k += 1;
            n += 1;
        }
    }
    assert!(n >= 40, "{n}");
    eprintln!("class 101 instances: {n}");
}

/// the real bomb template: thrown (state 1) it falls; a contact makes it stick (state 2, normal kept); a hit
/// lights the fuse (state 3); thrown by kind 0x1a it rights itself
#[test]
fn k15_101_throw_stick_fuse() {
    use sly2_recomp::engine::classes::k15_101::Contact57;
    let (loader, descs) = levels();
    let lv = crate::common::load_blocking(&loader, descs[0].clone()).unwrap();
    let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
    let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
    let mut s = ClassSet::build(&w, &mut h);
    s.post_load(&mut h);
    let i = s.k15.iter().find_map(|(i, x)| matches!(x, Inst::K15b(_)).then_some(*i)).unwrap();
    s.k15_101_with(i, &mut h, |c, cx| {
        c.f_15c = 5;
        c.ee_sub_224ec8(i, 1, cx);
        c.phys.v = [300.0, 0.0, 300.0];
    });
    let z0 = s.k15_101_get(i).unwrap().phys.pos[2];
    for _ in 0..60 {
        s.g.dt = DT;
        s.g.t += DT;
        s.frame(&mut h);
    }
    let c = s.k15_101_get(i).unwrap();
    assert_eq!(c.f_150, 1);
    assert!(c.phys.v[2] < 0.0 && c.phys.v[0] == 300.0, "{:?}", c.phys);
    eprintln!("flight: z {z0} -> {}", c.phys.pos[2]);
    // a kind-8 contact does not stick; a wall contact on the second half does
    s.k15_101_with(i, &mut h, |c, cx| {
        c.ee_sub_224380(i, &[Contact57 { other: 0, kind: 8, normal: [0.0, 0.0, 1.0], second: false }], cx);
        assert_eq!(c.f_158, -1);
        c.ee_sub_224380(i, &[Contact57 { other: 7, kind: 0, normal: [1.0, 0.0, 0.0], second: true }], cx);
        assert_eq!((c.f_158, c.f_1a0, c.f_190), (2, 7, [-1.0, 0.0, 0.0]));
    });
    s.g.t += DT;
    s.frame(&mut h);
    let c = s.k15_101_get(i).unwrap();
    assert_eq!((c.f_150, c.f_158), (2, -1));
    assert_eq!(c.phys.w, [0.0; 3]);
    // a hit lights the fuse; a second one changes nothing
    assert_eq!(s.k15_101_with(i, &mut h, |c, cx| (c.ee_sub_224190(i, cx), c.f_150)), Some((true, 3)));
    assert_eq!(s.k15_101_with(i, &mut h, |c, cx| (c.ee_sub_224190(i, cx), c.f_150)), Some((true, 3)));
    // kind 0x1a: tilted 60 degrees, the servo rights it
    s.k15_101_with(i, &mut h, |c, cx| {
        c.ee_sub_224ec8(i, 0, cx);
        c.f_15c = 0x1a;
        c.ee_sub_224ec8(i, 1, cx);
        let (sn, cs) = (60f32.to_radians().sin(), 60f32.to_radians().cos());
        c.phys.rot = [[1.0, 0.0, 0.0], [0.0, cs, sn], [0.0, -sn, cs]];
        c.phys.v = [0.0; 3];
    });
    for _ in 0..120 {
        s.g.dt = DT;
        s.g.t += DT;
        s.frame(&mut h);
    }
    let c = s.k15_101_get(i).unwrap();
    eprintln!("upright: up z = {}", c.phys.rot[2][2]);
    assert!(c.phys.rot[2][2] > 0.99, "{:?}", c.phys.rot);
}

/// every level x 300 frames: the class 79 / 80 / 82 projectiles (and the K13 class 81 through the inherited
/// slots) stay finite; resting ones in the world never explode
#[test]
#[ignore = "heavy"]
fn k15_79_all_levels() {
    let (loader, descs) = levels();
    let mut n = [0usize; 3];
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        s.post_load(&mut h);
        for _ in 0..300 {
            s.g.dt = DT;
            s.g.t += DT;
            s.frame(&mut h);
        }
        for (_, x) in &s.k15 {
            let Inst::K15c(c) = x else { continue };
            assert!(c.p.phys.pos.iter().chain(c.p.phys.v.iter()).all(|v| v.is_finite()));
            n[match c.cls {
                79 => 0,
                80 => 1,
                _ => 2,
            }] += 1;
        }
    }
    eprintln!("class 79 / 80 / 82 instances: {n:?}");
    assert!(n.iter().sum::<usize>() > 0, "no class 79 / 80 / 82 instance");
}

/// projectile helpers on a default `Proj`: fuse, explosion record (once), slot 74 bit 3 fuse, slot 60
/// ignore-oid, class-82 homing closes in on its target
#[test]
fn k15_79_helpers_and_homing() {
    use sly2_recomp::engine::classes::k15_79::{C2cb780, Cx};
    use sly2_recomp::engine::classes::shared::proj::Proj;
    let mut h = EngineHost::default();
    let mut p = Proj::default();
    p.ee_sub_13a9e8(0, &mut h);
    assert_eq!(p.f_154, f32::MAX);
    p.ee_sub_13b5c8(2.0, 10.0);
    assert_eq!(p.f_154, 12.0);
    p.ee_sub_13b5c8(5.0, 10.0);
    assert_eq!(p.f_154, 12.0);
    p.phys.v = [100.0, 0.0, 0.0];
    p.f_170 = 1;
    p.ee_sub_13b428([1.0, 2.0, 3.0], [0.0, 0.0, 1.0]);
    assert_eq!(p.f_180, [1.0, 2.0, 3.0]);
    assert_eq!(p.f_174, 1);
    p.ee_sub_13b428([9.0, 9.0, 9.0], [0.0, 0.0, 1.0]);
    assert_eq!(p.f_180, [1.0, 2.0, 3.0]);
    // slot 74 with bit 3: fuse now
    let mut q = Proj::default();
    q.ee_sub_13a9e8(0, &mut h);
    let hit = sly2_recomp::engine::classes::guard::slots::GuardHit { f_44: 8, vec: [0.0, 0.0, 5.0, 0.0], ..Default::default() };
    assert!(q.ee_sub_13b2f0(&hit, &mut Cx { t: 3.0, dt: DT, h: &mut h }));
    assert_eq!(q.f_154, 3.0);
    assert_eq!(q.f_174, 1);
    // class 82 homing: 1500 away, arrival in 1 s
    let mut c = C2cb780 { cls: 82, ..Default::default() };
    c.p.ee_sub_1b21f0(0, &mut h);
    assert_eq!(c.p.f_274, 1500.0);
    c.p.phys.kind = 1;
    c.p.f_220 = [1500.0, 0.0, 0.0, 0.0];
    c.p.f_278 = 1.0;
    let mut t = 0.0;
    for _ in 0..60 {
        t += DT;
        c.physics(0, t, DT, &mut h);
    }
    let d = (c.p.phys.pos[0] - 1500.0).abs();
    eprintln!("homing miss: {d} pos {:?}", c.p.phys.pos);
    assert!(d < 150.0, "{d}");
}

/// a real class-79 instance launched ballistically (FUN_001d6848) at a point 1000 away flies under its gravity,
/// its fuse explodes it (FUN_0013b860: fuse reset, record cleared, back at the explosion point)
#[test]
fn k15_79_real_instance_ballistic_fuse() {
    use sly2_recomp::engine::classes::shared::launch::FUN_001d6848;
    let (loader, descs) = levels();
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        s.post_load(&mut h);
        let Some(i) = s.k15.iter().find_map(|(i, x)| match x {
            Inst::K15c(c) if c.cls == 79 && c.p.phys.gravity[2] < 0.0 => Some(*i),
            _ => None,
        }) else {
            continue;
        };
        let t0 = s.g.t;
        let c = s.k15_79_get(i).unwrap();
        let p0 = c.p.phys.pos;
        let aim = [p0[0] + 1000.0, p0[1], p0[2]];
        let v = FUN_001d6848(0.5, p0, aim, c.p.phys.gravity).expect("ballistic solution");
        assert!(v[2] > 0.0);
        c.p.phys.v = v;
        c.p.phys.kind = 1;
        c.p.f_158 = 0f32.to_bits();
        c.p.in_play = true;
        c.p.ee_sub_13b5c8(0.5, t0);
        for _ in 0..20 {
            s.g.dt = DT;
            s.g.t += DT;
            s.frame(&mut h);
        }
        let c = s.k15_79_get(i).unwrap();
        assert!(c.p.phys.pos[0] > p0[0] + 50.0, "{:?} -> {:?}", p0, c.p.phys.pos);
        assert!(c.p.f_154 < f32::MAX);
        for _ in 0..20 {
            s.g.dt = DT;
            s.g.t += DT;
            s.frame(&mut h);
        }
        let c = s.k15_79_get(i).unwrap();
        assert_eq!(c.p.f_154, f32::MAX, "fuse not reset by the explosion");
        assert_eq!(c.p.f_174, 0);
        eprintln!("class 79 obj {i}: {:?} -> {:?}", p0, c.p.phys.pos);
        return;
    }
    panic!("no class-79 instance with gravity");
}

/// every level x 300 frames: the class-83 switches stay in a valid state
#[test]
#[ignore = "heavy"]
fn k15_83_all_levels() {
    let (loader, descs) = levels();
    let (mut n, mut modes) = (0, [0; 5]);
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        s.post_load(&mut h);
        for _ in 0..300 {
            s.g.dt = DT;
            s.g.t += DT;
            s.frame(&mut h);
        }
        for (_, x) in &s.k15 {
            let Inst::K15d(c) = x else { continue };
            assert!((0..=4).contains(&c.tg.state), "state {}", c.tg.state);
            assert!(c.tg.anims.iter().all(|l| l.len() <= 8) && c.tg.sms.len() < 8 && c.tg.sigs.len() <= 4);
            modes[(c.f_2a8.clamp(0, 4)) as usize] += 1;
            n += 1;
        }
    }
    assert!(n > 0, "no class-83 instance");
    eprintln!("class 83 instances: {n}, modes {modes:?}");
}

/// a real hit switch (mode 1 or 2): an accepted hit turns it on (half-0 lists fired, state 2 unless it stays in
/// 1), mode 2 hits it off again; the hold time brings state 2 to 4 and back on
#[test]
fn k15_83_real_switch_hit() {
    use sly2_recomp::engine::classes::guard::slots::GuardHit;
    let (loader, descs) = levels();
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        s.post_load(&mut h);
        let Some(i) = s.k15.iter().find_map(|(i, x)| match x {
            Inst::K15d(c) if c.f_2a8 == 1 || c.f_2a8 == 2 => Some(*i),
            _ => None,
        }) else {
            continue;
        };
        // an accepted hit source: the listed object, else an object of a listed class
        let c = s.k15_83_get(i).unwrap();
        let (objs, classes) = (c.objs.clone(), c.classes.clone());
        let src = match objs.first() {
            Some(&o) => o,
            None => {
                use sly2_recomp::engine::classes::ClassHost;
                let p = sly2_recomp::engine::classes::guard::PLAYER_HANDLE;
                if classes.iter().all(|&k| h.ee_sub_132b10(p, k) != 0) {
                    p
                } else {
                    match (1..=w.objects.len() as u32).find(|&o| classes.iter().all(|&k| h.ee_sub_132b10(o, k) != 0)) {
                        Some(o) => o,
                        None => continue,
                    }
                }
            }
        };
        assert_eq!(s.k15_83_get(i).unwrap().tg.state, 0);
        let sm0 = h.sm_states.len();
        let hit = GuardHit { src, target: i as u32 + 1, f_44: 1, ..Default::default() };
        assert!(s.k15_83_hit(i, &hit, &mut h).unwrap());
        let c = s.k15_83_get(i).unwrap();
        let (mode, stay1, hold, sms) = (c.f_2a8, c.tg.stay1, c.tg.hold, c.tg.sms.len());
        assert_eq!(c.tg.state, if stay1 == 0 { 2 } else { 1 });
        if sms > 0 {
            assert!(h.sm_states.len() >= sm0, "state machines");
        }
        // a hit flagged +0x44 bit 2 is ignored
        let hit4 = GuardHit { f_44: 4 | 1, ..hit };
        let st = s.k15_83_get(i).unwrap().tg.state;
        s.k15_83_hit(i, &hit4, &mut h);
        assert_eq!(s.k15_83_get(i).unwrap().tg.state, st);
        if stay1 == 0 && hold >= 0.0 {
            let frames = (hold / DT) as usize + 3;
            for _ in 0..frames {
                s.g.dt = DT;
                s.g.t += DT;
                s.frame(&mut h);
            }
            // 2 -> 4 -> (mode 1: 0, mode 2: 1 -> 2)
            let st = s.k15_83_get(i).unwrap().tg.state;
            if mode == 1 {
                assert_eq!(st, 0);
            } else {
                assert!(st == 1 || st == 2 || st == 4, "state {st}");
            }
        }
        if mode == 2 {
            let c = s.k15_83_get(i).unwrap();
            c.tg.ee_sub_148a78(1, 0.0, &mut h);
            assert!(s.k15_83_hit(i, &hit, &mut h).unwrap());
            assert_eq!(s.k15_83_get(i).unwrap().tg.state, 0, "mode 2 hit off");
        }
        eprintln!("class 83 obj {i}: mode {mode}, stay1 {stay1}, hold {hold}, sms {sms}");
        return;
    }
    panic!("no hit-mode class-83 instance");
}

/// every level x 300 frames: the class-85 beams stay finite, clipped within their length, 64 max
#[test]
#[ignore = "heavy"]
fn k15_85_all_levels() {
    let (loader, descs) = levels();
    let (mut n, mut beams, mut kinds, mut modes) = (0, 0, [0; 3], std::collections::BTreeMap::new());
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        s.post_load(&mut h);
        for _ in 0..300 {
            s.g.dt = DT;
            s.g.t += DT;
            s.frame(&mut h);
        }
        for (_, x) in &s.k15 {
            let Inst::K15e(c) = x else { continue };
            assert!(c.beams.len() <= 0x40);
            for b in &c.beams {
                assert!(b.len.is_finite() && b.clip <= b.len + 1e-3, "len {} clip {}", b.len, b.clip);
                for e in &b.ends {
                    assert!((0..=1).contains(&e.kind), "kind {}", e.kind);
                    assert!(e.local.iter().all(|v| v.is_finite()) && e.d.is_finite());
                    kinds[e.kind as usize] += 1;
                    *modes.entry(e.mode).or_insert(0) += 1;
                }
                beams += 1;
            }
            n += 1;
        }
    }
    assert!(n > 0, "no class-85 instance");
    eprintln!("class 85 instances: {n}, beams {beams}, endpoint kinds {kinds:?}, row-541 modes {modes:?}");
}

/// a real class-85 instance with its real beams; endpoint 1 rides a 1000-unit straight path (test host):
/// the motion stays on the path, an overlapping object crossing the beam clips it and is zapped (kind 0x1000,
/// -10), the overlap end restores the full beam
#[test]
fn k15_85_real_beam_motion_and_zap() {
    use sly2_recomp::engine::classes::guard::slots::GuardHit;
    use sly2_recomp::engine::classes::k15_85::{HIT_AMOUNT, HIT_KIND};
    use sly2_recomp::engine::classes::ClassHost;
    #[derive(Default)]
    struct H {
        path: u32,
        over: Vec<u32>,
        hits: Vec<GuardHit>,
        draws: usize,
    }
    impl ClassHost for H {
        fn ee_sub_15e938(&mut self, _k: i32, oid: i32, _c: u32) -> u32 {
            if oid == -1 { 0 } else { oid as u32 + 1 }
        }
        fn ee_sub_132b10(&mut self, o: u32, c: i32) -> u32 {
            (o == self.path && c == 10) as u32
        }
        fn path_len_1e6648(&mut self, _p: u32) -> f32 {
            1000.0
        }
        fn path_point_1e6f10(&mut self, _p: u32, d: f32) -> [f32; 3] {
            [d, 0.0, 100.0]
        }
        fn beam_hit_14c6a0(&mut self, _o: u32, p0: [f32; 3], p1: [f32; 3]) -> Option<[f32; 3]> {
            Some(std::array::from_fn(|i| (p0[i] + p1[i]) * 0.5))
        }
        fn so_overlaps_85(&mut self, _obj: usize) -> Vec<u32> {
            self.over.clone()
        }
        fn ee_sub_1f7598(&mut self, hit: &GuardHit) {
            self.hits.push(hit.clone());
        }
        fn beam_draw_85(&mut self, _obj: usize, segs: &[([f32; 3], [f32; 3])]) {
            self.draws += segs.len();
        }
    }
    let (loader, descs) = levels();
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let mut e = EngineHost { world: Some(w.clone()), ..Default::default() };
        let s0 = ClassSet::build(&w, &mut e);
        let Some((i, oid1)) = s0.k15.iter().find_map(|(i, x)| match x {
            Inst::K15e(c) => c.beams.iter().find(|b| b.ends[1].oid != -1 && (b.ends[1].prm[1] != 0.0 || b.ends[1].prm[2] != 0.0)).map(|b| (*i, b.ends[1].oid)),
            _ => None,
        }) else {
            continue;
        };
        let mut h = H { path: oid1 as u32 + 1, ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        s.post_load(&mut h);
        let c = s.k15_85_get(i).unwrap();
        let bi = c.beams.iter().position(|b| b.ends[1].oid == oid1 && (b.ends[1].prm[1] != 0.0 || b.ends[1].prm[2] != 0.0)).unwrap();
        let (kind, mode, nseg) = (c.beams[bi].ends[1].kind, c.beams[bi].ends[1].mode, c.beams[bi].ends[1].segs.len());
        eprintln!("class 85 obj {i}: beams {}, end-1 kind {kind}, mode {mode}, segs {nseg}, prm {:?}", c.beams.len(), c.beams[bi].ends[1].prm);
        let mut ds = Vec::new();
        for _ in 0..240 {
            s.g.dt = DT;
            s.g.t += DT;
            s.frame(&mut h);
            let b = &s.k15_85_get(i).unwrap().beams[bi];
            assert!((b.len - b.clip).abs() < 1e-3);
            ds.push(b.ends[1].d);
        }
        assert!(h.draws > 0);
        assert_eq!(kind, 1, "moving endpoint");
        {
            assert!(ds.iter().all(|d| (-1e-2..=1000.01).contains(d)), "d {ds:?}");
            assert!(ds.windows(2).any(|w| w[0] != w[1]), "endpoint does not move");
        }
        // an object overlaps: slot 59 contacts, the deferred FUN_001df260 clips and zaps
        h.over = vec![7];
        s.frame(&mut h);
        let b = s.k15_85_get(i).unwrap().beams[bi].clone();
        assert_eq!(b.contacts.len(), 1);
        assert!((b.clip - b.len * 0.5).abs() < 1e-2 * b.len.max(1.0), "clip {} len {}", b.clip, b.len);
        let hit = h.hits.iter().find(|x| x.target == 7).expect("zap");
        assert_eq!((hit.src, hit.kind, hit.amount), (i as u32 + 1, HIT_KIND, HIT_AMOUNT));
        // overlap ends: slot 53 drops the contacts, full length again
        h.over.clear();
        s.frame(&mut h);
        let b = &s.k15_85_get(i).unwrap().beams[bi];
        assert!(b.contacts.is_empty() && (b.clip - b.len).abs() < 1e-3);
        return;
    }
    panic!("no class-85 beam with a moving endpoint");
}
