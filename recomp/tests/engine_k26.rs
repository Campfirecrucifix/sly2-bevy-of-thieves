//! WP-K26 tests ('world alive'): level sounds (class 43), anim events / SM messages for every object,
//! world bodies, plus the 43-level x 600-frame census.
use sly2_recomp::engine::classes::plugin::EngineHost;
use sly2_recomp::engine::classes::k26::MusicRt;
use sly2_recomp::engine::classes::ClassSet;
use sly2_recomp::engine::world::{LoadOptions, World};
use std::sync::Arc;

const DT: f32 = 1.0 / 60.0;

#[path = "engine_k20.rs"]
#[allow(dead_code)]
mod k20;

/// dump the property lists of class-43 objects (sound entry lists)
#[test]
#[ignore]
fn k26_probe_c43_props() {
    let (loader, descs) = k20::levels();
    let (mut f99, mut f100, mut n43) = (0, 0, 0);
    for d in descs.into_iter() {
        let Ok(lv) = loader.load_blocking(d) else { continue };
        let Ok(w) = World::load(&lv, LoadOptions::default()) else { continue };
        for (i, o) in w.objects.iter().enumerate().filter(|o| o.1.class == 43) {
            n43 += 1;
            for p in sly2_recomp::engine::classes::props_of(&w, o.clone_of.unwrap_or(i)).unwrap_or(&[]) {
                f99 += (p.field == 99) as usize;
                f100 += (p.field == 100) as usize;
            }
        }
        for (i, o) in w.objects.iter().enumerate().filter(|o| o.1.class == 43).take(3) {
            eprintln!("obj {i} owner {:?} props {:?}", o.owner, sly2_recomp::engine::classes::props_of(&w, o.clone_of.unwrap_or(i)));
        }
    }
    eprintln!("class 43: {n43} objects, field 99 x{f99}, field 100 x{f100}");
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Census {
    pub levels: usize,
    pub asegas: usize,
    pub smas: usize,
    pub moving: usize,
    pub bodies: usize,
    pub sounds: usize,
    pub events: usize,
    pub dispatched: usize,
    pub sma_msgs: usize,
    pub delivered: usize,
    pub played: usize,
    pub posed: usize,
    pub targets: usize,
    pub moved_posed: usize,
    pub music_srcs: usize,
}

/// `routines`: stand-in for the level scripts' FUN_00171cf8 calls (method field 317, run by J10 in the game):
/// every guard able to walk runs the routine with the rail nearest to it (npcdraw::patrol_routines, 64 units)
pub fn census(frames: usize, routines: bool) -> Census {
    let (loader, descs) = k20::levels();
    let mut c = Census::default();
    for d in descs {
        let Ok(lv) = loader.load_blocking(d) else { continue };
        let Ok(w) = World::load(&lv, LoadOptions::default()) else { continue };
        c.levels += 1;
        let w = Arc::new(w);
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        h.k26_music = MusicRt::from_block(&lv.header.block_3dd040);
        let mut s = ClassSet::build(&w, &mut h);
        s.g.g_3dd268 = h.k26_music.inst;
        s.post_load(&mut h);
        c.music_srcs += h.k26_music.layers.iter().map(|l| l.len()).sum::<usize>();
        if routines {
            for (g, r) in sly2_recomp::engine::npcdraw::patrol_routines(&s, 64.0) {
                s.k6_run_routine(g, r, &mut h);
            }
        }
        let mut moved = std::collections::HashSet::new();
        let mut first: std::collections::HashMap<usize, [f32; 3]> = Default::default();
        let mut mp = std::collections::HashSet::new();
        for _ in 0..frames {
            s.g.t += DT;
            s.g.dt = DT;
            s.frame(&mut h);
            h.k26_due_sounds();
            for (o, m) in &h.k26_posed {
                let p = *first.entry(*o).or_insert(m[3]);
                if (0..3).any(|a| (p[a] - m[3][a]).abs() > 1e-3) {
                    mp.insert(*o);
                }
            }
            for (i, b) in s.k25_bodies() {
                if b.v.iter().any(|c| *c != 0.0) {
                    moved.insert(i);
                }
            }
        }
        c.bodies += h.k25_phys.bodies.len();
        c.moving += moved.len();
        c.moved_posed += mp.len();
        c.asegas += h.k25.started;
        c.smas += h.k25.sma_started;
        c.sounds += h.k25_snd.plays.len();
        c.events += h.k25.events;
        c.sma_msgs += h.k25.sma_msgs;
        c.dispatched += h.k26.dispatched;
        c.delivered += h.k26.delivered;
        c.played += h.k26_out.played;
        c.posed += h.k26.posed;
        c.targets += h.k25_phys.targets.values().filter(|v| v.iter().any(|x| *x != 0.0)).count();
        eprintln!("level {} -> {c:?}", c.levels);
    }
    c
}

/// 43 levels x 600 frames: anims started, SMs, moving bodies, sounds, anim events
#[test]
#[ignore]
fn k26_probe_census() {
    let c = census(600, false);
    eprintln!("K26 census 600: {c:?}");
    let c = census(600, true);
    eprintln!("K26 census 600 (guard routines started): {c:?}");
}

/// bodies by kind: count / gravity / ckind / pos moved over 300 frames (first 10 levels)
#[test]
#[ignore]
fn k26_probe_bodies() {
    let (loader, descs) = k20::levels();
    let mut kinds: std::collections::BTreeMap<String, (usize, usize, usize)> = Default::default();
    for d in descs {
        let Ok(lv) = loader.load_blocking(d) else { continue };
        let Ok(w) = World::load(&lv, LoadOptions::default()) else { continue };
        let w = Arc::new(w);
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        s.post_load(&mut h);
        let b0: std::collections::HashMap<usize, [f32; 3]> = s.k25_bodies().into_iter().map(|(i, b)| (i, b.pos)).collect();
        for _ in 0..300 {
            s.g.t += DT;
            s.g.dt = DT;
            s.frame(&mut h);
        }
        for (i, b) in s.k25_bodies() {
            let k = format!("class {} ck {} g {:?}", w.objects[i].class, b.ckind, b.gravity);
            let e = kinds.entry(k).or_default();
            e.0 += 1;
            if b0.get(&i).is_some_and(|p| (0..3).any(|a| (p[a] - b.pos[a]).abs() > 1e-3)) {
                e.1 += 1;
            }
            if b.v.iter().any(|c| *c != 0.0) {
                e.2 += 1;
            }
        }
    }
    for (k, v) in kinds {
        eprintln!("{k}: n {} moved {} v {}", v.0, v.1, v.2);
    }
}

/// posed objects of level 0: rest vs posed position
#[test]
#[ignore]
fn k26_probe_posed() {
    let (loader, descs) = k20::levels();
    let d = descs.into_iter().next().unwrap();
    let lv = loader.load_blocking(d).unwrap();
    let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
    let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
    let mut s = ClassSet::build(&w, &mut h);
    s.post_load(&mut h);
    for f in 0..120 {
        s.g.t += DT;
        s.g.dt = DT;
        s.frame(&mut h);
        if f % 40 == 0 {
            for (o, m) in &h.k26_posed {
                let ob = &w.objects[*o];
                eprintln!("f {f} obj {o} class {} oid {} owner {:?} posed {:?}", ob.class, ob.oid, ob.owner, m[3]);
            }
            for x in h.k25.asegas.values() {
                let a = sly2_recomp::engine::playerdraw::aseg_of(&w, x.aseg).unwrap();
                eprintln!("  asega aseg {} target {} t {} chans {:?}", x.aseg, x.obj, x.st.t, a.channels.iter().map(|c| (c.oid, c.kind, c.vec.is_some())).collect::<Vec<_>>());
            }
        }
    }
}

/// world-object xf constraint targets (FUN_0012d108) after 120 frames: owner class, xf offset, target
#[test]
#[ignore]
fn k26_probe_targets() {
    let (loader, descs) = k20::levels();
    for d in descs {
        let Ok(lv) = loader.load_blocking(d) else { continue };
        let Ok(w) = World::load(&lv, LoadOptions::default()) else { continue };
        let w = Arc::new(w);
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        s.post_load(&mut h);
        for _ in 0..120 {
            s.g.t += DT;
            s.g.dt = DT;
            s.frame(&mut h);
        }
        for (x, v) in &h.k25_phys.targets {
            let (owner, off) = (x / 0x100, x % 0x100);
            let cls = (owner as usize).checked_sub(1).and_then(|i| w.objects.get(i)).map(|o| o.class);
            eprintln!("xf {x:#x} owner {owner:#x} class {cls:?} off {off:#x} target {v:?}");
        }
    }
}

/// asserted minimums over the 43 levels x 300 frames (K27: no test-side guard routines; the original's
/// script / sequence starts are covered by engine_k27's `k27_guards_patrol_from_scripts_and_sequences`):
/// class-43 / anim-event sounds reach the host audio, anim events dispatched, SMA messages delivered, bodies and
/// ASEG-bound objects move, class-43 music sources attached
#[test]
fn k26_world_alive_minimums() {
    let c = census(300, false);
    eprintln!("{c:?}");
    assert!(c.levels >= 40, "{c:?}");
    assert!(c.played > 0 && c.sounds > 0, "sounds {c:?}");
    assert!(c.events > 0 && c.dispatched == c.events, "anim events {c:?}");
    assert!(c.sma_msgs > 0 && c.delivered == c.sma_msgs, "sma msgs {c:?}");
    // moving bodies (guards walking): asserted by engine_k27 with the routines started the original way
    // K29: k26_bind is the D6 AnimBinder (1:1 bone references): the live class ASEGAs drive non-root bones,
    // so object worlds no longer move; the moving bones are asserted by engine_k29 `k29_class_asegas_drive_bones`
    let _ = c.moved_posed;
    assert!(c.music_srcs > 0, "music sources {c:?}");
}

/// FUN_001fe990 / FUN_001fc408 / FUN_001fb658 / FUN_001fcfa0 / FUN_001fe5f0 / FUN_001fe738 on MusicRt
#[test]
fn k26_music_rt() {
    let mut m = MusicRt { keys: vec![7, 9], inst: 1, recs: vec![3, 5, 3], ..Default::default() };
    assert_eq!(m.ee_sub_1fe990(9), 1);
    assert_eq!(m.ee_sub_1fe990(4), -1);
    let f = |x: f32| x.to_bits() as i32;
    m.ee_sub_1fc408(2, 1, 5, [f(10.0), f(10.0)]);
    assert_eq!(m.layers[1][0].far, 11.0);
    assert_eq!(m.f_50 & 0x200, 0x200);
    m.ee_sub_1fc408(2, -1, 5, [f(1.0), f(2.0)]);
    m.ee_sub_1fc408(2, 6, 5, [f(1.0), f(2.0)]);
    for _ in 0..10 {
        m.ee_sub_1fc408(2, 0, 5, [f(1.0), f(2.0)]);
    }
    assert_eq!(m.layers[0].len(), 8);
    assert_eq!(m.layers.iter().map(|l| l.len()).sum::<usize>(), 9);
    // the head of the +0xec list is the last loaded record
    assert_eq!(m.ee_sub_1fcfa0(3), Some(2));
    assert_eq!(m.ee_sub_1fe5f0(4, 1), 0);
    let h = m.ee_sub_1fe5f0(5, 1);
    assert_eq!(h, sly2_recomp::engine::classes::k26::MUSIC_TAG | 2);
    m.ee_sub_1fe5f0(3, 0);
    assert_eq!((m.plays, m.stops), (2, 1));
    m.ee_sub_1fe738();
    assert_eq!((m.cur, m.stops), (0, 2));
}
