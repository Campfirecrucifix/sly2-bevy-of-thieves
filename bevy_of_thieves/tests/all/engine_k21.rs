//! WP-K21 tests.
use sly2_recomp::engine::classes::plugin::EngineHost;
use sly2_recomp::engine::classes::{ClassHost, ClassSet};
use sly2_recomp::engine::disc::descriptor::Descriptor;
use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::world::{LoadOptions, World};
use std::sync::Arc;

use crate::common as k20;

pub fn levels() -> (LevelLoader, Vec<Descriptor>) {
    k20::levels()
}

#[test]
#[ignore]
fn k21_probe_119() {
    let (loader, descs) = levels();
    for (li, d) in descs.into_iter().enumerate() {
        let Ok(lv) = crate::common::load_blocking(&loader, d) else { continue };
        let Ok(w) = World::load(&lv, LoadOptions::default()) else { continue };
        let w = Arc::new(w);
        if !w.objects.iter().any(|o| o.class == 119) {
            continue;
        }
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        for (i, o) in w.objects.iter().enumerate() {
            if o.class != 119 {
                continue;
            }
            if let Some(ps) = sly2_recomp::engine::game::fields::object_props(&w.objects[o.clone_of.unwrap_or(i)]) {
                for x in ps { if (1320..1340).contains(&x.field) && x.field != 1325 { eprintln!("   field {} {:?}", x.field, x.value); } }
            }
            let c = s.k20_119(i).unwrap();
            eprintln!("L{li} obj {i} oid {} slots {} l94 {:?} l94o {:?} l9c {:?} ac {} 80 {} 84 {} 88 {} 8c {}", o.oid, c.m.i(0x5c), c.l94, c.l94_obj, c.l9c, c.m.i(0xac), c.m.f(0x80), c.m.i(0x84), c.m.i(0x88), c.m.i(0x8c));
            let mut hist = std::collections::BTreeMap::new();
            for p in &c.points {
                let hits: Vec<_> = w.find_oid(p.oid).iter().map(|&k| w.objects[k].class).collect();
                *hist.entry((p.kind, hits)).or_insert(0) += 1;
            }
            eprintln!("   points {:?}", hist);
            for &o in &c.l94 { eprintln!("   l94 {} -> {:?}", o, w.find_oid(o as i32).iter().map(|&k| w.objects[k].class).collect::<Vec<_>>()); }
        }
    }
}

const DT: f32 = 1.0 / 60.0;

/// G1: every level with class 119: camera 6000 units behind the first spawn point of the first spawner (all
/// probes beyond +0x80 = 5500 -> FUN_00173768 returns 2 = hidden), the player 20000 units above; the
/// spawner picks a point (FUN_00173f18), clones a guard (FUN_00174490 / FUN_001cb250); the guard then runs
/// 300 frames in the K6 set.
#[test]
fn k21_c119_spawns_guards() {
    use sly2_recomp::engine::classes::k21_119::CamView;
    let (loader, descs) = levels();
    let mut levels_spawned = 0;
    let mut total = 0;
    for (li, d) in descs.into_iter().enumerate() {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let sp: Vec<usize> = w.objects.iter().enumerate().filter(|x| x.1.class == 119).map(|x| x.0).collect();
        if sp.is_empty() {
            continue;
        }
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        s.post_load(&mut h);
        let pt = s.k20_119(sp[0]).unwrap().points[0].clone();
        let p = sly2_recomp::engine::game::fields::ee_sub_1a62e8(&w, pt.obj as usize - 1);
        let cm = sly2_recomp::engine::camera::cm::Cm::new();
        let mut mat = cm.mat;
        mat[0] = [1.0, 0.0, 0.0, 0.0];
        mat[1] = [0.0, 1.0, 0.0, 0.0];
        mat[2] = [0.0, 0.0, 1.0, 0.0];
        mat[3] = [p[0] - 6000.0, p[1], p[2], 1.0];
        h.cam = Some(CamView { mat, x_scale: cm.x_scale, y_scale: cm.y_scale });
        let pl = [p[0], p[1], p[2] + 20000.0];
        h.player_pos = pl;
        h.player_valid = true;
        s.g.k6_players = vec![(sly2_recomp::engine::classes::guard::PLAYER_HANDLE, pl)];
        let mut first = None;
        for f in 0..900 {
            s.g.dt = DT;
            s.g.t += DT;
            s.frame(&mut h);
            if first.is_none() && !s.k21.spawned.is_empty() {
                first = Some(f);
            }
            if first.is_some_and(|f0| f >= f0 + 300) {
                break;
            }
        }
        let n = s.k21.spawned.len();
        eprintln!("L{li}: spawners {} first spawn frame {:?} spawned {n} {:?}", sp.len(), first, &s.k21.spawned);
        if let Some(f0) = first {
            levels_spawned += 1;
            total += n;
            let g = s.k21.spawned[0];
            assert!(s.k6.iter().any(|x| x.0 == g), "L{li}: spawned guard {g} still in the K6 set after 300 frames (from {f0})");
            let c = s.k20_119(sp[0]).unwrap();
            assert!(c.m.i(0x58) >= 1 || sp.len() > 1);
        }
    }
    eprintln!("levels with spawns {levels_spawned}, guards {total}");
    assert!(levels_spawned > 0);
}

/// G1: the alert record list DAT_003e145c (FUN_001f8240 / FUN_001f8278 / FUN_0020a5f8 / FUN_001f82b8)
#[test]
fn k21_alert_list() {
    use sly2_recomp::engine::classes::k21_119::*;
    let mut l = Vec::new();
    assert_eq!(ee_sub_1f8240(&l, 7), None);
    assert_eq!(ee_sub_1f8278(&mut l, 7), 0);
    assert_eq!(ee_sub_1f8278(&mut l, 9), 1);
    assert_eq!(ee_sub_1f8278(&mut l, 7), 0);
    ee_sub_1f82b8(&mut l, 9, 3, Some([4.0, 5.0]));
    assert_eq!((l[1].f_10, l[1].f_14, l[1].f_18), (3, 4.0, 5.0));
    assert_eq!(ee_sub_1f8240(&l, 9), Some(1));
    assert!(ee_sub_173588(1.25, [1.0, -1.2, 600.0]) && !ee_sub_173588(1.25, [1.3, 0.0, 600.0]) && !ee_sub_173588(1.7, [0.0, 0.0, 0.0]));
    assert_eq!(ee_sub_1ea3b0(5.0, &[[0.0, 1.0], [4.0, 6.0]]), Some(1));
}

#[test]
#[ignore]
fn k21_probe_77() {
    let (loader, descs) = levels();
    for (li, d) in descs.into_iter().enumerate() {
        let Ok(lv) = crate::common::load_blocking(&loader, d) else { continue };
        let Ok(w) = World::load(&lv, LoadOptions::default()) else { continue };
        for (i, o) in w.objects.iter().enumerate() {
            if o.class != 77 {
                continue;
            }
            for oid in [39024, 39022, 39014, 27891, 37817, 28576, 39051, 39060, 39062, 39085, 39039] {
                eprintln!("  oid {oid} -> {:?}", w.find_oid(oid).iter().map(|&k| (k, w.objects[k].class, w.objects[k].owner)).collect::<Vec<_>>());
            }
            eprintln!("L{li} obj {i} oid {} clone {:?} children {:?}", o.oid, o.clone_of, o.children.iter().map(|&c| (c, w.objects[c].class, w.objects[c].oid)).collect::<Vec<_>>());
            if let Some(ps) = sly2_recomp::engine::game::fields::object_props(&w.objects[o.clone_of.unwrap_or(i)]) {
                for x in ps {
                    eprintln!("   field {} {:?}", x.field, x.value);
                }
            }
        }
    }
}

// ---- class 77 leftovers (classes/k21_77.rs)
use sly2_recomp::engine::classes::k20_77::C2d7ca8;
use sly2_recomp::engine::classes::k21_77::{ee_sub_1d2dd0, ee_sub_1d4d30, PartRec};

/// list methods, part defaults / damage / lookup, deepest contact, slot 18 spin, the slot-15 aim
#[test]
fn k21_c77_records() {
    let mut c = C2d7ca8::default();
    for k in 0..9 {
        c.ee_sub_1d4b48(20.0, 100 + k, [-10.0, 5.0]);
        c.ee_sub_1d4c48(200 + k, [-5.0, 5.0]);
        c.ee_sub_1d4b90(3, 300 + k, 1, 2, -1);
    }
    assert_eq!((c.wheels.len(), c.treads.len(), c.parts.len()), (8, 4, 8));
    assert_eq!(c.ee_sub_1d3fa8(305), Some(5));
    let mut p = PartRec::default();
    ee_sub_1d4d30(&mut p, 4);
    assert_eq!((p.owner, p.pitch, p.speed, p.g, p.f_9c), (4, [-1.57, 1.57], [250.0, 2500.0], -1750.0, 1.5));
    c.parts[1].stage = 1;
    c.parts[1].thresh = 30;
    c.m.set_i(0x580, 50);
    c.ee_sub_1d4e30(1);
    c.ee_sub_1d4e30(1);
    assert_eq!(c.m.i(0x580), 0);
    assert_eq!(ee_sub_1d2dd0(&[]), None);
    assert_eq!(ee_sub_1d2dd0(&[1.0, 3.0, 3.0, 2.0]), Some(1));
    // slot 18: no contact, rear wheels spin up toward speed / radius
    let mut h = sly2_recomp::engine::classes::NullHost;
    c.m.set_f(0x550, 200.0);
    for _ in 0..120 {
        c.ee_sub_1d41f8(3, 0.0, 1.0 / 60.0, &mut h);
    }
    assert_eq!(c.wheels[0].rate, 0.0);
    assert!((c.wheels[2].rate - 10.0).abs() < 1e-3, "{}", c.wheels[2].rate);
    // contact: rate = forward speed / radius
    c.wheels[3].contacts = vec![0.5];
    c.ee_sub_1d41f8(3, 100.0, 1.0 / 60.0, &mut h);
    assert_eq!(c.wheels[3].rate, 5.0);
    // aim: stationary, target 1000 ahead -> on target within the ranges
    c.m.set_f(0x550, 0.0);
    for r in &mut c.part_recs {
        r.bone = 1;
    }
    let id = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    c.ee_sub_1d19a8_tail(3, &id, [0.0; 3], [1000.0, 0.0, 0.0], [0.0; 3], 1.0 / 60.0, &mut h);
    let r = &c.part_recs[0];
    assert_eq!(r.on, 1);
    assert!(r.aim > 0.0 && r.aim < 1.57 && r.aim_speed > 250.0, "{r:?}");
}

/// every class-77 instance resolves its wheel / tread / part lists from the field rows, runs 300 frames
/// driven (state 2) and the wheels / treads move
#[test]
#[ignore = "heavy"]
fn k21_c77_lists_all_levels_300_frames() {
    let (loader, descs) = levels();
    let mut n = 0;
    for d in descs {
        let Ok(lv) = crate::common::load_blocking(&loader, d) else { continue };
        let Ok(w) = World::load(&lv, LoadOptions::default()) else { continue };
        let w = Arc::new(w);
        if !w.objects.iter().any(|o| o.class == 77) {
            continue;
        }
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        s.post_load(&mut h);
        let objs: Vec<usize> = w.objects.iter().enumerate().filter(|o| o.1.class == 77).map(|o| o.0).collect();
        for &o in &objs {
            let c = s.k20_77(o).unwrap();
            eprintln!("obj {o}: wheels {} treads {} parts {} 4a0 {}", c.wheels.len(), c.treads.len(), c.parts.len(), c.m.f(0x4a0));
            assert!(!c.wheels.is_empty() && !c.treads.is_empty() && !c.parts.is_empty());
            assert!(c.m.f(0x4a0) > 0.0);
        }
        let t = s.g.t;
        s.k20_77(objs[0]).unwrap().ee_sub_1d1940(objs[0], 1, t, &mut h);
        s.k20_77(objs[0]).unwrap().joy.angle = 1.0;
        for _ in 0..300 {
            s.g.dt = 1.0 / 60.0;
            s.g.t += 1.0 / 60.0;
            let t = s.g.t;
            let c = s.k20_77(objs[0]).unwrap();
            c.joy.feed(t, 0x40, 0);
            c.joy.angle = 1.0;
            s.frame(&mut h);
        }
        let c = s.k20_77(objs[0]).unwrap();
        assert_eq!(c.state(), 2);
        assert!(c.wheels.iter().any(|w| w.steer != 0.0) && c.wheels.iter().any(|w| w.rate != 0.0));
        eprintln!("wheel steer {:?} tread {:?}", c.wheels.iter().map(|w| w.steer).collect::<Vec<_>>(), c.treads.iter().map(|t| t.ang).collect::<Vec<_>>());
        n += objs.len();
    }
    assert!(n > 0);
}

// ---- Murray FUN_001b93e8 (classes/k21_54.rs)
use sly2_recomp::engine::classes::k18_54::C2d44c8;
use sly2_recomp::engine::classes::k18_55::Cx;
use sly2_recomp::engine::classes::ClassGlobals;
use sly2_recomp::engine::physics::driven::GameRng;

/// carried 0x50 holds 0x60 (in the world); 0x60 +0x58 / +0x5c bones 0x70 / 0x71 (+0x98 ids 7 / 8)
#[derive(Default)]
struct HM {
    ee: Vec<u32>,
}
impl ClassHost for HM {
    fn obj_i32(&mut self, obj: u32, off: u32) -> i32 {
        match (obj, off) {
            (0x50, 0x180) => 0x60,
            (0x60, 0x58) => 0x70,
            (0x60, 0x5c) => 0x71,
            (0x70, 0x98) => 7,
            (0x71, 0x98) => 8,
            (0x50, 0x190) => 0x80,
            _ => 0,
        }
    }
    fn obj_flags_28(&mut self, obj: u32) -> u64 {
        ((obj == 0x60) as u64) << 60
    }
    fn ee_call(&mut self, addr: u32, _args: &[u32]) -> i64 {
        self.ee.push(addr);
        match addr {
            0x12de58 => 0x91,
            0x12dc18 => 0x92,
            0x12dd28 => 0x93,
            0x12ddb0 => 0x94,
            _ => 0,
        }
    }
}

#[test]
fn k21_c54_carry_setup() {
    let mut m = C2d44c8::default();
    let g = ClassGlobals { t: 1.0, ..Default::default() };
    let mut rng = GameRng::default();
    let mut h = HM::default();
    m.b.m.set_u(0x13f0, 0x50);
    m.b.m.set_i(0x1450, 0);
    {
        let mut cx = Cx { g: &g, rng: &mut rng, h: &mut h };
        m.ee_sub_1b93e8(3, 1, &mut cx);
    }
    assert_eq!((m.b.m.i(0x1450), m.b.m.i(0x1404), m.b.m.i(0x1408)), (1, 7, 8));
    assert_eq!((m.b.m.u(0x13fc), m.b.m.u(0x1400)), (0x91, 0x92));
    assert_eq!(h.ee[..4], [0x1f5720, 0x1a58d0, 0x1f6860, 0x12f938]);
    h.ee.clear();
    m.b.m.set_v3(0x190, [1.0; 3]);
    {
        let mut cx = Cx { g: &g, rng: &mut rng, h: &mut h };
        m.ee_sub_1b93e8(3, 2, &mut cx);
    }
    assert_eq!((m.b.m.u(0x13f8), m.b.m.u(0x13f4), m.b.m.u(0x1618), m.b.m.u(0x161c), m.b.m.u(0x1604)), (0x92, 0x92, 0x93, 0x93, 0x94));
    assert_eq!((m.b.m.u(0x1620), m.b.m.v3(0x190)), (0, [0.0; 3]));
}

/// K21 G3 probe: K21_CLS=<class> prints every instance with its field rows
#[test]
#[ignore]
fn k21_probe_class() {
    let cls: u32 = std::env::var("K21_CLS").ok().and_then(|s| s.parse().ok()).unwrap_or(68);
    let (loader, descs) = levels();
    for (li, d) in descs.into_iter().enumerate() {
        let Ok(lv) = crate::common::load_blocking(&loader, d) else { continue };
        let Ok(w) = World::load(&lv, LoadOptions::default()) else { continue };
        for (i, o) in w.objects.iter().enumerate() {
            if o.class as u32 != cls {
                continue;
            }
            eprintln!("L{li} obj {i} oid {} clone {:?} owner {:?} children {:?}", o.oid, o.clone_of, o.owner, o.children.iter().map(|&c| (c, w.objects[c].class, w.objects[c].oid)).collect::<Vec<_>>());
            if o.clone_of.is_some() { continue; }
            if let Some(ps) = sly2_recomp::engine::game::fields::object_props(&w.objects[i]) {
                for x in ps {
                    eprintln!("   field {} {:?}", x.field, x.value);
                }
            }
        }
    }
}

#[test]
fn k21_c28_particles() {
    use sly2_recomp::engine::classes::k21_28::{C2cb5d8, Particle};
    let mut c = C2cb5d8::default();
    c.m.set_i(0x160, 1);
    c.m.set_i(0x43c, 2);
    let p = |x: f32, die: f32| Particle { pos: [x, 0.0, 0.0, 2.0], vel: [10.0, 0.0, 0.0, die], spin: [0.0, 0.0, 1.0, 0.5], rec: [0; 8] };
    c.add_group(vec![p(0.0, 5.0), p(100.0, 1.0)]);
    c.add_group(vec![p(0.0, 1.0)]);
    assert_eq!(c.m.i(0x444), 3);
    let id = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0; 4]];
    c.ee_sub_139dc0(2.0, 0.5, &id);
    // two died (death time <= now), the second group freed, the survivor moved by v * dt and spun
    assert_eq!((c.m.i(0x444), c.groups.len(), c.groups[0].parts.len(), c.groups[0].cur), (1, 1, 1, 1));
    assert_eq!(c.groups[0].parts[0].pos[0], 5.0);
    assert_eq!(c.groups[0].parts[0].spin[3], 1.0);
    assert_eq!(c.m.v3(0xf0), [5.0, 0.0, 0.0]);
    assert!(!c.removed);
    c.ee_sub_139dc0(6.0, 0.5, &id);
    assert!(c.removed && c.groups.is_empty());
}

#[test]
#[ignore = "heavy"]
fn k21_c68_c49_levels_300_frames() {
    use sly2_recomp::engine::classes::Inst;
    let (loader, descs) = levels();
    let (mut n, mut n49s) = (0, 0);
    for d in descs {
        let Ok(lv) = crate::common::load_blocking(&loader, d) else { continue };
        let Ok(w) = World::load(&lv, LoadOptions::default()) else { continue };
        let w = Arc::new(w);
        if !w.objects.iter().any(|o| matches!(o.class, 68 | 28 | 49)) {
            continue;
        }
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        s.post_load(&mut h);
        for _ in 0..300 {
            s.g.dt = 1.0 / 60.0;
            s.g.t += 1.0 / 60.0;
            s.frame(&mut h);
        }
        let n49 = w.objects.iter().filter(|o| o.class == 49).count();
        let k49: Vec<_> = s.k21c.iter().filter_map(|(_, x)| if let Inst::C2d4260(c) = x { Some(c) } else { None }).collect();
        assert_eq!(k49.len(), n49);
        assert!(k49.iter().all(|c| c.alo.so.f_148 >> 35 & 1 == 0 && c.alo.so.f_140 == -1 && c.alo.so.f_12c == 1.0));
        n49s += n49;
        for (i, x) in &s.k21c {
            if let Inst::K21C68(c) = x {
                eprintln!("68 obj {i}: idle {:#x} moves {:?}", c.m.u(0x2c0), c.moves);
                assert!(!c.moves.is_empty() && c.m.i(0x2c4) == c.moves.len() as i32);
                assert!(c.moves.iter().all(|m| m.mask != 0));
                n += 1;
            }
        }
    }
    assert_eq!(n, 1);
    eprintln!("class 49 instances {n49s}");
    assert!(n49s > 0);
}
