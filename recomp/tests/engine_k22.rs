//! WP-K22 tests.
use sly2_recomp::engine::classes::guard::brain::Child;
use sly2_recomp::engine::classes::guard::c56::Guard;
use sly2_recomp::engine::classes::guard::search::{ee_sub_20a168, ee_sub_20a408};
use sly2_recomp::engine::classes::guard::slots::Stimulus;
use sly2_recomp::engine::classes::plugin::EngineHost;

const DT: f32 = 1.0 / 60.0;

/// G1 class 171 (0xab search goal): a heard stimulus moves the brain to sub 2; the goal runs surprise (0) ->
/// go (1, stimulus at the guard: goto done at once) -> look (2) -> search points (3, mode +0xe78 = 1 after
/// 2 s) -> give up (4, dwell +0x38 = rand(5.5, 6.5)) -> done (-1) -> brain sub 1.
#[test]
fn k22_c171_search_states() {
    let mut g = Guard::default();
    let mut h = EngineHost::default();
    g.ee_sub_205eb8(0, &mut h);
    g.f_e64 = 1;
    g.f_e68 = 0;
    g.f_e78 = 1;
    g.body.f_fe8 = 2;
    g.stimuli = vec![Stimulus { level: 3, mode: 0, f_4c: 1.0, ..Default::default() }];
    let mut seen = std::collections::BTreeSet::new();
    let mut subs = std::collections::BTreeSet::new();
    let mut t = 1.0;
    for _ in 0..1200 {
        g.ee_sub_207350(0, t, DT, &[], &|_| None, &|_| false, false, &[], &mut h);
        subs.insert(g.brain.sub.code);
        if let Child::Search(s) = &g.brain.child {
            seen.insert(s.st);
            assert!(g.f_2c0 & (1 << 0x1f) != 0);
            if s.st >= 3 {
                g.stimuli.clear();
            }
        }
        t += DT;
    }
    eprintln!("subs {subs:?} search states {seen:?}");
    assert!(subs.contains(&2), "{subs:?}");
    // state 4 (give-up anim, length 0 headless) passes to -1 inside the frame it is entered
    for st in [0, 1, 2, 3] {
        assert!(seen.contains(&st), "{seen:?}");
    }
    assert!(!matches!(g.brain.child, Child::Search(_)));
    assert_eq!(g.brain.sub.code, 1);
    assert!(g.f_2c0 & (1 << 0x1f) == 0);
}

/// FUN_0020a168 / FUN_0020a408 ordering: kind, then level, then +0x4c, then distance
#[test]
fn k22_stimulus_order() {
    let a = Stimulus { level: 3, f_4c: 1.0, pos: [10.0, 0.0, 0.0], ..Default::default() };
    assert!(ee_sub_20a168([0.0; 3], &Stimulus { level: 4, ..a.clone() }, &a) < 0);
    assert!(ee_sub_20a168([0.0; 3], &Stimulus { f_4c: 2.0, ..a.clone() }, &a) < 0);
    assert!(ee_sub_20a168([0.0; 3], &Stimulus { pos: [5.0, 0.0, 0.0], ..a.clone() }, &a) < 0);
    assert!(ee_sub_20a168([0.0; 3], &Stimulus { mode: 1, ..a.clone() }, &a) > 0);
    let b = Stimulus { level: 4, ..a.clone() };
    assert_eq!(ee_sub_20a408([0.0; 3], &[a.clone(), b.clone()], None), Some(b.clone()));
    assert_eq!(ee_sub_20a408([0.0; 3], &[a.clone()], Some(&b)), Some(b));
}

#[path = "engine_k20.rs"]
#[allow(dead_code)]
mod k20;

#[test]
#[ignore]
fn k22_probe_class() {
    use sly2_recomp::engine::disc::descriptor::Descriptor;
    use sly2_recomp::engine::world::{LoadOptions, World};
    let cls: u16 = std::env::var("K22_CLASS").ok().and_then(|s| s.parse().ok()).unwrap_or(96);
    let (loader, descs) = k20::levels();
    let descs: Vec<Descriptor> = descs;
    for (li, d) in descs.into_iter().enumerate() {
        let Ok(lv) = loader.load_blocking(d) else { continue };
        let Ok(w) = World::load(&lv, LoadOptions::default()) else { continue };
        for (i, o) in w.objects.iter().enumerate() {
            if o.class != cls {
                continue;
            }
            eprintln!("L{li} obj {i} oid {} clone_of {:?} owner {:?}", o.oid, o.clone_of, o.owner);
            if let Some(ps) = sly2_recomp::engine::game::fields::object_props(&w.objects[o.clone_of.unwrap_or(i)]) {
                for x in ps {
                    let m = sly2_recomp::engine::game::fieldbind::FIELD_MEMBER.get(x.field as usize).copied().unwrap_or(0);
                    eprintln!("   field {} member {:#x} {:?}", x.field, m, x.value);
                }
            }
        }
    }
}

/// G2 host for class 96: the creature (obj 0) moves with the velocity it sets, faces its target yaw at once.
struct H96 {
    pos: [f32; 3],
    vel: [f32; 3],
    yaw: f32,
    tgt: Option<[f32; 3]>,
    path: Option<std::sync::Arc<sly2_recomp::engine::player::p5_target::Path>>,
    bursts: u32,
    msgs: Vec<(u32, i32, u32)>,
}
impl sly2_recomp::engine::classes::ClassHost for H96 {
    fn ee_sub_1a62e8(&mut self, obj: u32) -> [f32; 3] {
        if obj == 1 { self.pos } else { self.tgt.unwrap_or([0.0; 3]) }
    }
    fn k22_targets_3eb8(&mut self) -> Vec<u32> {
        self.tgt.map(|_| 0x8000_0000).into_iter().collect()
    }
    fn vcall_fc(&mut self, _obj: u32) -> sly2_recomp::engine::anim::pose::Mat4 {
        let (s, c) = self.yaw.sin_cos();
        [[c, s, 0.0, 0.0], [-s, c, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [self.pos[0], self.pos[1], self.pos[2], 1.0]]
    }
    fn vcall_11c_v(&mut self, _obj: u32, v: [f32; 3]) {
        self.vel = v;
    }
    fn k22_path(&mut self, obj: u32) -> Option<std::sync::Arc<sly2_recomp::engine::player::p5_target::Path>> {
        if obj == 77 { self.path.clone() } else { None }
    }
    fn ee_sub_1fbb90(&mut self, _obj: usize, id: i32) {
        if id == 0x14 {
            self.bursts += 1;
        }
    }
    fn vcall_19c_msg(&mut self, target: u32, a: i32, src: u32) {
        self.msgs.push((target, a, src));
    }
}

/// G2 class 96: wander path 0 -> 400 (state 1 at the start, 2 at the path end, turn back -> 1); a target 300
/// away -> 3 (turn) -> facing -> 4 (charge at 1000 toward it, FUN_001f7bf8 after 2 s) -> flag-8 contact -> 5
/// (burst 0x14 + message 6 to +0x1c8).
#[test]
fn k22_c96_states() {
    use sly2_recomp::engine::classes::actor::alo::Touch;
    use sly2_recomp::engine::classes::k22_96::C2d3838;
    use sly2_recomp::engine::player::p5_target::{Path, M4_IDENT};
    let mut h = H96 { pos: [0.0; 3], vel: [0.0; 3], yaw: 0.0, tgt: None, path: None, bursts: 0, msgs: vec![] };
    h.path = Some(std::sync::Arc::new(Path::polyline(vec![[0.0, 0.0, 0.0], [400.0, 0.0, 0.0]], M4_IDENT)));
    let mut c = C2d3838::default();
    c.ee_sub_1b0b78(0, None, 0, &mut h);
    assert_eq!((c.st, c.f_150, c.f_154, c.f_1cc, c.f_1d0), (-1, 400.0, 800.0, 300.0, 1000.0));
    c.f_16c = 77;
    c.f_1c8 = 9;
    let mut now = 0.0f32;
    c.ee_sub_1b0d08(0, now, &mut h);
    assert_eq!(c.st, 1, "wander path -> state 1");
    let mut seen = std::collections::BTreeSet::new();
    let mut step = |c: &mut C2d3838, h: &mut H96, now: &mut f32, seen: &mut std::collections::BTreeSet<i32>| {
        *now += DT;
        c.ee_sub_1b0e28(0, *now, DT, h);
        for k in 0..3 {
            h.pos[k] += h.vel[k] * DT;
        }
        if c.st == 2 || c.st == 3 {
            h.yaw = c.f_1d8;
        }
        seen.insert(c.st);
    };
    for _ in 0..200 {
        step(&mut c, &mut h, &mut now, &mut seen);
    }
    assert!(seen.contains(&2), "path end -> turn round (2): {seen:?}");
    assert!(seen.contains(&1));
    assert!(h.pos[0] > 100.0 && h.pos[0] <= 410.0, "walked the path: {:?}", h.pos);
    // a target inside the near radius
    h.tgt = Some([h.pos[0], h.pos[1] + 300.0, 0.0]);
    for _ in 0..3 {
        step(&mut c, &mut h, &mut now, &mut seen);
    }
    assert!(seen.contains(&3) && c.st == 4, "spotted -> turn (3) -> charge (4): {seen:?} {}", c.st);
    assert!((h.vel[1] - 1000.0).abs() < 1.0, "charge toward the target: {:?}", h.vel);
    assert!((c.f_22c - (c.f_224 + 2.0)).abs() < 1e-3);
    let mut t = Touch::default();
    t.w_44 = 8;
    assert!(c.ee_sub_1b0f88(0, &t, now, &mut h));
    assert_eq!(c.st, 5);
    assert_eq!((h.bursts, h.msgs.clone()), (1, vec![(9, 6, 1)]));
    // far target, no path: state 0 idles
    let mut c = C2d3838::default();
    c.ee_sub_1b0b78(0, None, 0, &mut h);
    c.ee_sub_1b0d08(0, now, &mut h);
    assert_eq!(c.st, 0);
    h.tgt = Some([h.pos[0] + 2000.0, h.pos[1], 0.0]);
    for _ in 0..600 {
        now += DT;
        c.ee_sub_1b0e28(0, now, DT, &mut h);
        assert_eq!(c.st, 0);
    }
}

/// G2 class 96 on every level: instances load (fields 965 / 971 / 972) and run 300 frames.
#[test]
fn k22_c96_levels() {
    use sly2_recomp::engine::classes::{ClassSet, Inst};
    use sly2_recomp::engine::world::{LoadOptions, World};
    use std::sync::Arc;
    let (loader, descs) = k20::levels();
    let mut n = 0;
    for d in descs {
        let Ok(lv) = loader.load_blocking(d) else { continue };
        let Ok(w) = World::load(&lv, LoadOptions::default()) else { continue };
        if !w.objects.iter().any(|o| o.class == 96) {
            continue;
        }
        let w = Arc::new(w);
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        s.post_load(&mut h);
        for _ in 0..300 {
            s.g.t += DT;
            s.g.dt = DT;
            s.frame(&mut h);
        }
        for (i, o) in w.objects.iter().enumerate() {
            if o.class != 96 {
                continue;
            }
            let Some(Inst::K22C96(c)) = s.k22c(i) else { panic!("class 96 obj {i} not built") };
            assert_eq!(c.st, 0, "no wander path, no player: idles");
            if o.clone_of.is_some() || w.objects[i].clone_of.is_none() {
                assert!(c.f_158 == 2.0 && c.f_15c == 10.0 || c.f_158 == 2.0 && c.f_15c == 6.0, "fields {} {}", c.f_158, c.f_15c);
            }
            n += 1;
        }
    }
    assert!(n >= 3, "class-96 instances {n}");
}

/// G3 class 115 (WR warp): records from the loader fields, circle / bend matrices, the glob bake.
#[test]
fn k22_c115_warp() {
    use sly2_recomp::engine::classes::k22_115::*;
    let mut h = EngineHost::default();
    let mut c = C2dcf00::default();
    c.ee_sub_22b720(0, &mut h);
    assert_eq!((c.cur, c.f_5c, c.f_50), (-1, -1, -1.0));
    // two circles share matrix 0 (rows 0-1, 2-3), the bend gets the next matrix count but word 1 = 0
    c.ee_sub_22c3b0();
    c.ee_sub_22c518(100.0);
    c.recs[0][8..11].copy_from_slice(&[1.0, 0.0, 0.0]);
    c.recs[0][0x10..0x13].copy_from_slice(&[0.0, 1.0, 0.0]);
    c.recs[0][0x1a] = 0.25;
    c.ee_sub_22c3b0();
    assert_eq!((c.recs[1][1].to_bits(), c.recs[1][0x18].to_bits(), c.f_58, c.f_5c), (0, 2, 1, -1));
    assert!((c.ee_sub_22c570() - 0.0).abs() < 1e-6, "record 1 has no wavelength");
    assert!(c.dirty);
    c.ee_sub_22c028(0, 1.0, &mut h); // angle = 0.25 * 2pi -> (s, c) = (1, 0)
    assert!(!c.dirty && c.f_50 == 1.0);
    let m = c.mats[0];
    assert!((m[0][1] - 1.0).abs() < 1e-4 && m[0][0].abs() < 1e-4 && (m[1][0] + 1.0).abs() < 1e-4, "{m:?}");
    // bend with one noise term on each struct
    let mut b = C2dcf00::default();
    b.ee_sub_22b720(0, &mut h);
    b.ee_sub_22c430();
    {
        let r = b.ee_sub_22c4a0(true).unwrap();
        ee_sub_22c5b8([1.0, 0.7, 0.0, 1.0], &mut r[0xc..]);
        ee_sub_22c5b8([1.0, 0.3, 0.0, 1.0], &mut r[0x20..]);
        r[0xc] = -0.3;
        r[0xd] = 0.3;
        r[0x20] = -0.3;
        r[0x21] = 0.3;
    }
    let mut seen = 0;
    for f in 0..300 {
        b.ee_sub_22c028(0, f as f32 / 60.0, &mut h);
        let m = b.mats[0];
        assert!(m.iter().flatten().all(|x| x.is_finite()));
        if m[0][1].abs() > 1e-3 {
            seen += 1;
        }
    }
    assert!(seen > 0, "bend moved");
    for x in 0..64u32 {
        let v = ee_sub_22be08(x);
        assert!((0.0..1.0).contains(&v));
    }
    assert_eq!(ee_sub_22bf40(0.5, 0.5), 0.5);
    // bake: a circle wave along x, ramp kind 4 (x) 0 -> 100 maps weight 0 -> 1
    c.ee_sub_22b7a0(WrGlob {
        kind: 4,
        ramp: [0.0, 0.0, 100.0, 1.0],
        mat: sly2_recomp::engine::anim::pose::IDENTITY,
        xf: sly2_recomp::engine::anim::pose::IDENTITY,
        chunks: vec![WrChunk { verts: vec![[50.0, 0.0, 0.0, 1.0], [200.0, 0.0, 0.0, 1.0]], ..Default::default() }],
        ..Default::default()
    });
    c.ee_sub_22b7d8(&mut h);
    let o = &c.globs[0].chunks[0].out[0];
    // record 0 (rows 0-1): wavelength 100 -> phase x / 100 turns; record 1 (rows 2-3): wavelength 0
    assert!((o[0][0] + 0.5).abs() < 1e-3 && o[0][1].abs() < 1e-3, "{o:?}");
    assert!((o[1][0] - 1.0).abs() < 1e-3 && (o[1][2] - 1.0).abs() < 1e-3, "{o:?}");
}

/// G3 class 115 on every level: instances load their warp records and run 300 frames.
#[test]
fn k22_c115_levels() {
    use sly2_recomp::engine::classes::{ClassSet, Inst};
    use sly2_recomp::engine::world::{LoadOptions, World};
    use std::sync::Arc;
    let (loader, descs) = k20::levels();
    let (mut n, mut circles, mut bends) = (0, 0, 0);
    for d in descs {
        let Ok(lv) = loader.load_blocking(d) else { continue };
        let Ok(w) = World::load(&lv, LoadOptions::default()) else { continue };
        if !w.objects.iter().any(|o| o.class == 115) {
            continue;
        }
        let w = Arc::new(w);
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        s.post_load(&mut h);
        for _ in 0..300 {
            s.g.t += DT;
            s.g.dt = DT;
            s.frame(&mut h);
        }
        for (i, o) in w.objects.iter().enumerate() {
            if o.class != 115 {
                continue;
            }
            let t = s.g.t;
            let Some(Inst::K22C115(c)) = s.k22c(i) else { panic!("class 115 obj {i} not built") };
            assert!(c.n >= 1 && c.f_50 == t, "obj {i}: {} records, t {}", c.n, c.f_50);
            for r in &c.recs[..c.n as usize] {
                match r[0].to_bits() {
                    0 => circles += 1,
                    1 => bends += 1,
                    k => panic!("kind {k}"),
                }
            }
            n += 1;
        }
    }
    eprintln!("class 115: {n} instances, {circles} circles, {bends} bends");
    assert!(n >= 40 && circles > 0 && bends > 0, "class-115 instances {n}");
}

/// K22 class 69 host: a user with sticks / a fire press, a spawn pool
#[derive(Default)]
struct H69 {
    stick: Option<[f32; 4]>,
    press: bool,
    pins: Vec<[f32; 3]>,
    msgs: u32,
}

impl sly2_recomp::engine::classes::ClassHost for H69 {
    fn k22_stick(&mut self, _obj: u32) -> Option<[f32; 4]> {
        self.stick
    }
    fn k9_pressed(&mut self, _obj: u32, mask: u32) -> u32 {
        if std::mem::take(&mut self.press) { mask } else { 0 }
    }
    fn k22_pin_12dc18(&mut self, _obj: u32, v: [f32; 3]) -> u32 {
        self.pins.push(v);
        7
    }
    fn k22_pin_12d108(&mut self, _h: u32, v: [f32; 3]) {
        self.pins.push(v);
    }
    fn ee_sub_1cb250(&mut self, _host: u32, _k: i32) -> u32 {
        99
    }
    fn vcall_1c(&mut self, _obj: usize, a: i32, _b: i32) {
        if a == 5 {
            self.msgs += 1;
        }
    }
}

#[test]
fn k22_c69_turret() {
    use sly2_recomp::engine::classes::k22_69::C2dbd80;
    let mut h = H69::default();
    let mut c = C2dbd80::default();
    c.ee_sub_2259d8(3, &mut h);
    assert_eq!((c.f_1ac, c.f_1b0, c.f_1bc, c.f_1a4, c.f_19c, c.f_1cc), (0.1, 1.0, 1.0, [-1.5, 1.5], [-0.7, 0.7], 1));
    // slot 14: constraint created once
    c.ee_sub_225aa8(3, &mut h);
    assert_eq!((c.f_1cc, c.f_1d8, h.pins.len()), (1, 7, 1));
    // unused: nothing moves
    c.ee_sub_225b60(3, DT, &mut h);
    assert_eq!((c.f_18c, c.f_190, c.f_1b4), (0.0, 0.0, 0.0));
    // full left stick right: yaw decreases, clamped at -1.5; right stick up zooms in to 1
    h.stick = Some([1.0, 0.0, 0.0, 1.0]);
    for _ in 0..3000 {
        c.ee_sub_225b60(3, DT, &mut h);
    }
    assert_eq!(c.f_18c, -1.5);
    assert_eq!(c.f_1b4, 1.0);
    assert!((c.f_1bc - 0.1).abs() < 1e-6, "fov {}", c.f_1bc);
    assert_eq!(*h.pins.last().unwrap(), [0.0, -c.f_190, -1.5]);
    // stick up (setting 0 = inverted): pitch rises to its limit
    h.stick = Some([0.0, 1.0, 0.0, 0.0]);
    for _ in 0..600 {
        c.ee_sub_225b60(3, DT, &mut h);
    }
    assert_eq!(c.f_190, -0.7);
    // fire without a template: message 5; with one: spawned + message
    h.press = true;
    c.ee_sub_225b60(3, DT, &mut h);
    assert_eq!((h.msgs, c.fired), (1, 1));
    c.f_1c0 = 5;
    h.press = true;
    c.ee_sub_225b60(3, DT, &mut h);
    assert_eq!((h.msgs, c.f_1c8), (2, 99));
    // FUN_00226370(0) releases
    c.ee_sub_226370(3, 0, &mut h);
    assert_eq!((c.f_1cc, c.f_1d8), (0, 0));
    assert_eq!(sly2_recomp::engine::classes::k22_69::ee_sub_226310(0, Some(4), 4), 1);
}

#[test]
fn k22_c69_levels() {
    use sly2_recomp::engine::classes::{ClassSet, Inst};
    use sly2_recomp::engine::world::{LoadOptions, World};
    use std::sync::Arc;
    let (loader, descs) = k20::levels();
    let mut n = 0;
    for d in descs {
        let Ok(lv) = loader.load_blocking(d) else { continue };
        let Ok(w) = World::load(&lv, LoadOptions::default()) else { continue };
        if !w.objects.iter().any(|o| o.class == 69) {
            continue;
        }
        let w = Arc::new(w);
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        s.post_load(&mut h);
        for _ in 0..300 {
            s.g.t += DT;
            s.g.dt = DT;
            s.frame(&mut h);
        }
        for (i, o) in w.objects.iter().enumerate() {
            if o.class != 69 {
                continue;
            }
            let Some(Inst::K22C69(c)) = s.k22c(i) else { panic!("class 69 obj {i} not built") };
            eprintln!(
                "obj {i}: yaw {:?} pitch {:?} fov {}..{} tgt {} tmpl {} launcher {} cc {}",
                c.f_1a4, c.f_19c, c.f_1ac, c.f_1b0, c.f_180, c.f_1c0, c.f_1c4, c.f_1cc
            );
            assert!(c.f_1a4[0] <= c.f_18c && c.f_18c <= c.f_1a4[1] && c.f_19c[0] <= c.f_190 && c.f_190 <= c.f_19c[1]);
            assert!(c.f_1bc.is_finite());
            n += 1;
        }
    }
    eprintln!("class 69: {n} instances");
    assert!(n >= 1, "class-69 instances {n}");
}

/// G6 host for class 121: music time, the dancer's presses (FUN_001c4660 / 001c4700), live crowd links.
#[derive(Default)]
struct H121 {
    press: u32,
    held: u32,
    sw: Vec<(usize, i32)>,
    skill: f32,
}
impl sly2_recomp::engine::classes::ClassHost for H121 {
    fn ee_call(&mut self, addr: u32, args: &[u32]) -> i64 {
        match addr {
            0x1c4660 => self.press as i64,
            0x1c4700 => (self.held & args[1] != 0) as i64,
            _ => 0,
        }
    }
    fn ee_sub_15e938(&mut self, _kind: i32, oid: i32, _ctx: u32) -> u32 {
        oid as u32 + 1000
    }
    fn ee_sub_1f0380_link(&mut self, sm: u32, _obj: u32, _a: i32, _b: u32) -> u32 {
        sm + 1
    }
    fn ee_sub_1f0660(&mut self, obj: usize, anim: i32) {
        self.sw.push((obj, anim));
    }
    fn ee_sub_154c90(&mut self) -> f32 {
        self.skill
    }
}

/// G6 class 121 (rhythm manager): init values, crowd binding / thresholds, demo state 1 plays cues, player
/// state 2 judges presses (on beat ~1, early -1.5, wrong -0.8, missed -1.5; skill 0 scales misses by 1), score mean /
/// smoothing, crowd cheer level, beat cursor.
#[test]
fn k22_c121_rhythm() {
    use sly2_recomp::engine::classes::k21_68::C2cd748;
    use sly2_recomp::engine::classes::k22_121::*;
    use sly2_recomp::engine::classes::Inst;
    let mut h = H121::default();
    let mut m = C2cd5b8::default();
    m.ee_sub_1526d8(0, &mut h);
    assert_eq!((m.f_234, m.f_58, m.f_5c, m.f_280, m.f_2a4), ([-1.5, 1.0], 0.4, 0.4, 8, -1.5));
    m.ee_sub_153810(5, 0x627);
    m.ee_sub_152810(0, &mut h);
    let c = m.crowd[0];
    assert_eq!((c.link, c.level), (0x625 + 1000 + 1, 1));
    assert!(c.thr[0] > c.thr[1] && c.thr[2] == -1.5, "{:?}", c.thr);
    // dancer: obj 1 with one move of mask 0x10
    let mut d = C2cd748::default();
    d.m.0.resize(0x420 / 4, 0);
    d.ee_sub_1524d8(0x10, 0, 0, 0, 0);
    let mut ds = vec![(1usize, Inst::K21C68(Box::new(d)))];
    // demo
    m.ee_sub_1531a8(1, &[[1.0; 12], { let mut b = [0.0; 12]; b[0] = 3.0; b }], 0.0, &mut h);
    assert_eq!((m.st, m.beats[0][4], m.beats[1][4]), (1, 100.0, 540.0));
    m.ee_sub_1532d8(1.0, 2, 0x10, -1, 0.0);
    m.ee_sub_152970(0, 0.5, DT, &mut ds, &mut h);
    assert_eq!(m.cues[0].f_14, 0);
    m.ee_sub_152970(0, 1.1, DT, &mut ds, &mut h);
    assert_eq!(m.cues[0].f_14, 1);
    // player: on beat
    m.ee_sub_153128(2, 2.0, &mut h);
    assert!(m.cues.is_empty());
    m.ee_sub_1532d8(3.0, 2, 0x10, -1, 2.0);
    let Inst::K21C68(d) = &mut ds[0].1 else { unreachable!() };
    h.press = 0x10;
    h.held = 0x10;
    m.ee_sub_1533b0(0, 1, d, 3.0, &mut h);
    assert_eq!((m.cues[0].f_1c, m.hist.clone(), m.beats[1][8]), (1, vec![1.0], 1.0));
    // pressed again: -0.2 (x clamp(1.2 - 0.6 * skill 0) = 1)
    m.ee_sub_1533b0(0, 1, d, 3.05, &mut h);
    assert!((m.hist[1] + 0.2).abs() < 1e-5, "{:?}", m.hist);
    // too early on a new cue: -1.5
    m.cues[0].f_20 = 1; // the first cue is over (slot 15 past t + 0.4)
    m.ee_sub_1532d8(5.0, 2, 0x10, -1, 3.0);
    m.ee_sub_1533b0(0, 1, d, 4.0, &mut h);
    assert_eq!(m.hist[2], -1.5);
    // wrong button: -0.8
    h.held = 0x20;
    m.ee_sub_1533b0(0, 1, d, 4.9, &mut h);
    assert!((m.hist[3] + 0.8).abs() < 1e-5);
    // missed cue: prompt posted at t, over after t + 0.4
    m.ee_sub_1532d8(6.0, 2, 0x10, -1, 5.0);
    m.ee_sub_152970(0, 6.1, DT, &mut ds, &mut h);
    assert_eq!(m.f_230, 0x10);
    m.ee_sub_152970(0, 6.5, DT, &mut ds, &mut h);
    assert_eq!(m.hist.len(), 5);
    assert_eq!(m.hist[4], -1.5);
    let mean = m.hist.iter().sum::<f32>() / 8.0;
    assert!((m.f_284 - mean).abs() < 1e-5);
    // the smoothed score walks to the mean; the crowd level follows it
    for _ in 0..600 {
        m.ee_sub_152970(0, 7.0, DT, &mut ds, &mut h);
    }
    assert!((m.f_288 - m.f_284).abs() < 0.05, "{} {}", m.f_288, m.f_284);
    let lv = (0..3).find(|&i| m.crowd[0].thr[i] <= m.f_288).unwrap();
    assert_eq!((h.sw.last().unwrap().1, m.crowd[0].level), (0x626 + lv as i32, lv as i32));
    // 16 scores kept
    for _ in 0..20 {
        m.ee_sub_153640(0, 0.5, &mut h);
    }
    assert_eq!((m.hist.len(), m.f_284), (16, 0.5));
    // the cursor launches 0.4 s before beat 0 and lands on it
    m.ee_sub_1531a8(1, &[[1.0; 12], { let mut b = [0.0; 12]; b[0] = 3.0; b }], 0.0, &mut h);
    let mut t = 0.5;
    while t < 1.0 {
        m.ee_sub_152c58(t, DT);
        t += DT;
    }
    assert_eq!(m.f_190, 1);
    assert!((m.f_1a0[0] - 100.0).abs() < 2.0, "{:?}", m.f_1a0);
    m.ee_sub_152c58(1.01, DT);
    assert_eq!((m.f_190, m.f_194), (0, 1));
}

/// G6 class 121 on its level (11): built, crowd bound, 300 frames with the dancer (class 68) wired to it.
#[test]
fn k22_c121_levels() {
    use sly2_recomp::engine::classes::{ClassSet, Inst};
    use sly2_recomp::engine::world::{LoadOptions, World};
    use std::sync::Arc;
    let (loader, descs) = k20::levels();
    let mut n = 0;
    for d in descs {
        let Ok(lv) = loader.load_blocking(d) else { continue };
        let Ok(w) = World::load(&lv, LoadOptions::default()) else { continue };
        if !w.objects.iter().any(|o| o.class == 121) {
            continue;
        }
        let w = Arc::new(w);
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        s.post_load(&mut h);
        for _ in 0..300 {
            s.g.t += DT;
            s.g.dt = DT;
            s.frame(&mut h);
        }
        for (i, o) in w.objects.iter().enumerate() {
            if o.class != 121 {
                continue;
            }
            let Some(Inst::K22C121(c)) = s.k22c(i) else { panic!("class 121 obj {i} not built") };
            eprintln!("obj {i}: windows {} {} crowd {} st {} 288 {}", c.f_58, c.f_5c, c.crowd.len(), c.st, c.f_288);
            assert!(c.crowd.len() > 50 && c.f_288.is_finite());
            n += 1;
        }
        assert!(s.k22_rhythm().is_some());
    }
    eprintln!("class 121: {n} instances");
    assert!(n >= 1);
}

/// Item 2: level 0's title sequence (class 114, oid 1672) loads its records from fields 625..708 and,
/// started (FUN_001dfd58), runs to action 0x44 (FUN_001dd820(0x3da530)), waits while the title is shown
/// and then runs to its end (the sequence stops, message 6).
#[test]
fn k22_seq_1672_title() {
    use sly2_recomp::engine::classes::{ClassSet, Inst};
    use sly2_recomp::engine::game::methods::cls::ee_sub_1dfd58;
    use sly2_recomp::engine::world::{LoadOptions, World};
    use std::sync::Arc;
    let (loader, descs) = k20::levels();
    let mut done = false;
    for d in descs {
        let Ok(lv) = loader.load_blocking(d) else { continue };
        let Ok(w) = World::load(&lv, LoadOptions::default()) else { continue };
        let Some(i) = w.objects.iter().position(|o| o.class == 114 && o.oid == 1672) else { continue };
        let w = Arc::new(w);
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        s.post_load(&mut h);
        let kinds = |s: &ClassSet| -> Vec<u32> {
            s.more.iter().find_map(|(j, x)| match x {
                Inst::C2d8c48(c) if *j == i => Some(c.f_54.iter().map(|r| r[0]).collect()),
                _ => None,
            }).unwrap()
        };
        let k = kinds(&s);
        eprintln!("1672 record kinds {k:x?}");
        assert_eq!(k[0], 0);
        assert!(k.contains(&0x44) && k.contains(&0x2c) && k.contains(&0x27));
        let mut nodes = Vec::new();
        for (j, x) in &mut s.more {
            if let Inst::C2d8c48(c) = x {
                if *j == i {
                    ee_sub_1dfd58(c, &mut nodes, i, 0, &mut h);
                    assert_eq!(c.f_74, 1);
                }
            }
        }
        let running = |s: &ClassSet| -> bool {
            s.more.iter().any(|(j, x)| matches!(x, Inst::C2d8c48(c) if *j == i && c.f_74 != 0))
        };
        for f in 0..30 {
            s.g.t += DT;
            s.g.dt = DT;
            s.frame(&mut h);
            if f == 29 {
                assert_eq!(h.k22_title_shows, 1, "action 0x44 ran once");
                assert!(running(&s), "waits on the title");
            }
        }
        eprintln!("unported actions before 0x44: {:x?}", h.k22_seq_actions.iter().map(|a| a.2[0]).collect::<Vec<_>>());
        h.k22_title_active = false;
        for _ in 0..300 {
            s.g.t += DT;
            s.g.dt = DT;
            s.frame(&mut h);
        }
        eprintln!("after: {:x?}", h.k22_seq_actions.iter().map(|a| a.2[0]).collect::<Vec<_>>());
        assert!(!running(&s), "the sequence ran to its end");
        assert_eq!(h.k22_title_shows, 1);
        done = true;
        break;
    }
    assert!(done, "level 0 / sequence 1672 not found");
}

#[derive(Default)]
struct HSeq {
    shows: u32,
    title: bool,
    po: Vec<u32>,
    other: Vec<u32>,
}

impl sly2_recomp::engine::classes::ClassHost for HSeq {
    fn ee_sub_15e938(&mut self, _kind: i32, oid: i32, _ctx: u32) -> u32 {
        oid as u32
    }
    fn ee_sub_132b10(&mut self, obj: u32, class: i32) -> u32 {
        (class == 0x32 && obj == 55) as u32
    }
    fn k22_title_show_1dd820(&mut self) {
        self.shows += 1;
        self.title = true;
    }
    fn k22_title_active_1d9880(&mut self) -> bool {
        self.title
    }
    fn k22_po_switch(&mut self, po: u32) {
        self.po.push(po);
    }
    fn k22_seq_action(&mut self, _owner: u32, _subj: u32, rec: &[u32; 20]) {
        self.other.push(rec[0]);
    }
}

/// Item 2 synthetic: [open, 7 po 55, 7 non-po 9, open {0x2e}, close, 0x44 (waits), 0x53 wait 0.5 s, 5 signal 3,
/// 4 wait signal 3 (consume), close]: the slot-78 switch, the nested block, the title wait, the timer wait,
/// the signal, the end.
#[test]
fn k22_seq_actions() {
    use sly2_recomp::engine::classes::c2d8c48::C2d8c48;
    use sly2_recomp::engine::classes::k22_seq::ee_sub_1e1c40;
    use sly2_recomp::engine::game::methods::cls::ee_sub_1dfd58;
    let mut c = C2d8c48::default();
    let mut h = HSeq::default();
    c.ee_sub_1dfad8(0, &mut h);
    c.feed(&[]);
    c.f_58 = -1;
    let mut add = |k: u32, b: u32, a: &[u32]| {
        let i = c.ee_sub_1e01f8(k, b).unwrap();
        c.f_54[i][4..4 + a.len()].copy_from_slice(a);
    };
    add(7, 1, &[55]);
    add(7, 1, &[9]);
    add(0, 1, &[u32::MAX]);
    add(0x2e, 1, &[1, 2]);
    add(1, 1, &[]);
    add(0x44, 0, &[]);
    add(0x53, 0, &[0, 0.5f32.to_bits()]);
    add(5, 1, &[3, u32::MAX]);
    add(4, 0, &[3, 1]);
    let mut nodes = Vec::new();
    ee_sub_1dfd58(&mut c, &mut nodes, 0, 0, &mut h);
    assert_eq!(c.f_5c.len(), 1);
    let mut t = 0.0;
    let step = |c: &mut C2d8c48, h: &mut HSeq, t: f32| {
        for n in c.f_5c.clone() {
            ee_sub_1e1c40(c, 0, n, t, h);
        }
    };
    step(&mut c, &mut h, t);
    assert_eq!(h.po, vec![55]);
    assert_eq!(h.other, vec![0x2e]);
    assert_eq!(h.shows, 1);
    step(&mut c, &mut h, t);
    assert_eq!(c.k22_nodes[0].cur, Some(6), "waits on the title");
    h.title = false;
    t = 0.1;
    step(&mut c, &mut h, t);
    assert_eq!(c.k22_nodes[0].cur, Some(7), "timer started");
    t = 0.5;
    step(&mut c, &mut h, t);
    assert_eq!(c.k22_nodes[0].cur, Some(7));
    t = 0.7;
    step(&mut c, &mut h, t);
    assert!(c.f_5c.is_empty(), "signal consumed, block ended");
    assert!(c.k22_sig.is_empty());
}

#[test]
fn k22_set_3dd4bc() {
    use sly2_recomp::engine::classes::plugin::EngineHost;
    use sly2_recomp::engine::classes::ClassHost;
    let mut h = EngineHost::default();
    assert!(!h.ee_sub_1a61a0(5));
    h.base(0x11a220, 4);
    h.base(0x11a220, 4);
    assert_eq!(h.set_3dd4bc, vec![5]);
    assert!(h.ee_sub_11a380(5) && h.ee_sub_1a61a0(5));
    h.ee_sub_11a2e8(5);
    assert!(!h.ee_sub_11a380(5));
}
