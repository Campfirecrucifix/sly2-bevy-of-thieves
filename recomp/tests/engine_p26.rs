//! WP-P26: the carry / throw machine FUN_002206c8 + its arc FUN_0021fb70 (player::p26) and the pickpocket loot
//! state machine FUN_0013c560 / FUN_00143678 (player::loot).

use sly2_recomp::engine::classes::k17_55::Mem;
use sly2_recomp::engine::classes::plugin::EngineHost;
use sly2_recomp::engine::classes::{ClassHost, ClassSet};
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::player::loot::{LootDef, LootSys, CLASS_GUARD, CLASS_LOOT};
use sly2_recomp::engine::player::p20::{apply_po, PoCurrent, CLASS_BENTLEY, CLASS_MURRAY};
use sly2_recomp::engine::player::p26::{ee_sub_21fb70, ee_sub_2206c8, CarryCx, CARRY_JT, CARRY_MURRAY};
use sly2_recomp::engine::world::{LoadOptions, World};
use std::sync::Arc;

fn tables() -> (LevelLoader, Vec<sly2_recomp::engine::disc::descriptor::Descriptor>) {
    let iso = Arc::new(IsoImage::open(concat!(env!("CARGO_MANIFEST_DIR"), "/../Sly 2 - Band of Thieves (USA).iso")).expect("ISO"));
    let t = sly2_recomp::engine::disc::descriptor::static_table(&iso.read_file("SCUS_973.16").unwrap()).unwrap();
    let l = LevelLoader::new(iso, ModOverlay::default());
    let boot = l.load_blocking(t[25]).unwrap();
    let d = boot.header.level_table.entries.iter().map(|e| e.desc).collect();
    (l, d)
}

/// Real level (table row 4): Murray carries Bentley. Entering state 3 makes Murray the controlled character,
/// releasing to -1 throws Bentley (velocity = horizontal dir(Bentley - Murray) * 400 + 600 up) and hands control
/// back to DAT_002dd5bc (Bentley).
#[test]
fn p26_switch_characters_real_level() {
    use sly2_recomp::engine::player::p23::apply_switches;
    let (l, ds) = tables();
    let lv = l.load_blocking(ds[4]).unwrap();
    let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
    let cls = |c: u16| w.objects.iter().position(|x| x.class == c).expect("class in level");
    let (b, m) = (cls(CLASS_BENTLEY), cls(CLASS_MURRAY));
    let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
    let mut set = ClassSet::build(&w, &mut h);
    set.post_load(&mut h);
    let mut cur = PoCurrent::default();
    assert!(apply_po(&w, w.objects[b].oid, &mut set, &mut h, &mut cur));
    assert_eq!(set.k18_active, Some(b));
    let _ = std::mem::take(&mut h.p23_switch);
    let (bh, mh) = (b as u32 + 1, m as u32 + 1);
    let (pb, pm) = (h.ee_sub_1a62e8(bh), h.ee_sub_1a62e8(mh));
    // Murray picks Bentley up: carried word [4] = Bentley, state 3 -> Murray controlled
    set.k18_murray(m).unwrap().b.m.set_u(CARRY_MURRAY + 0x10, bh);
    h.po_part_s(0x2206c8, m, CARRY_MURRAY, 3);
    let q = std::mem::take(&mut h.p23_switch);
    assert_eq!(apply_switches(&w, &mut set, &mut h, q), vec![Some(m)]);
    assert_eq!(set.k18_active, Some(m));
    // the release on the machine itself: throw + hand-back
    let cx = CarryCx { owner: mh, bentley: bh, bentley_1800: 0, t: 1.0, dt: 1.0 / 60.0, ..Default::default() };
    let mut mem = set.k18_murray(m).unwrap().b.m.clone();
    let r = ee_sub_2206c8(&mut mem, CARRY_MURRAY, -1, &cx, &mut h, &mut |_, _| None).unwrap();
    assert_eq!(r.slot78, vec![bh]);
    let (o, v) = r.throw.expect("Bentley thrown");
    assert_eq!(o, bh);
    let d = [pb[0] - pm[0], pb[1] - pm[1]];
    let l = (d[0] * d[0] + d[1] * d[1]).sqrt();
    let e = if l < 1e-4 { [400.0, 0.0] } else { [d[0] / l * 400.0, d[1] / l * 400.0] };
    assert!((v[0] - e[0]).abs() < 1e-2 && (v[1] - e[1]).abs() < 1e-2 && v[2] == 600.0, "{v:?} {e:?}");
    assert_eq!(mem.u(CARRY_MURRAY + 0x10), 0, "carried cleared");
    assert_eq!(mem.i(CARRY_MURRAY + 4), -1);
    // the same through the switch queue: control back to Bentley
    h.po_part_s(0x2206c8, m, CARRY_MURRAY, -1);
    let q = std::mem::take(&mut h.p23_switch);
    assert_eq!(apply_switches(&w, &mut set, &mut h, q), vec![Some(b)]);
    assert_eq!(set.k18_active, Some(b));
}

/// FUN_0021fb70 through SetState(4): the arc starts at the target side and ends at the launch point, the
/// launch velocity reaches the target, the aim weights sum to 1; a blocked segment shortens the arc.
#[test]
fn p26_throw_arc() {
    let mut h = EngineHost::default();
    let off = CARRY_JT;
    let mk = || {
        let mut m = Mem::default();
        m.set_i(off + 4, -1);
        m.set_f(off + 0x18, 1.0); // [6] yaw range
        m.set_f(off + 0x1c, 0.0); // [7]
        m.set_f(off + 0x20, 1000.0); // [8]
        m.set_f(off + 0x24, 1.0); // [9] flight time
        m.set_v3(off + 0xc0, [500.0, 0.0, 0.0]);
        m.set_v3(off + 0xd0, [0.0, 0.0, 100.0]);
        m
    };
    let cx = CarryCx { owner: 1, gravity: [0.0, 0.0, -1000.0], t: 2.0, dt: 1.0, ..Default::default() };
    let mut m = mk();
    let r = ee_sub_2206c8(&mut m, off, 4, &cx, &mut h, &mut |_, _| None).unwrap();
    assert_eq!(m.i(off + 4), 4);
    assert_eq!(m.f(off + 8), 2.0);
    assert_eq!(*r.arc.last().unwrap(), [0.0, 0.0, 100.0]);
    assert!(r.arc.len() > 2);
    let v = m.v3(off + 0xf0);
    assert!(v[0] > 0.0 && v[1].abs() < 1e-3, "{v:?}");
    let ws: f32 = [0x1a, 0x1d, 0x20, 0x23].iter().map(|k| m.f(off + 4 * k)).sum();
    assert!((ws - 1.0).abs() < 1e-4);
    assert!((m.f(off + 4 * 0x29) + m.f(off + 4 * 0x2c) - 1.0).abs() < 1e-5);
    // a wall at x = 100: the arc stops there
    let mut m2 = mk();
    m2.set_i(off + 4, 4);
    m2.set_v3(off + 0xe0, [500.0, 0.0, 0.0]);
    let a = ee_sub_21fb70(&mut m2, off, false, &cx, &mut h, &mut |a, b| if a[0] < 100.0 && b[0] >= 100.0 { Some([100.0, 0.0, b[2]]) } else { None });
    let full = m.f(off + 4 * 0x4c);
    assert!(m2.f(off + 4 * 0x4c) < full, "{} {}", m2.f(off + 4 * 0x4c), full);
    // the landing time is the z root nearest the hit point (one step past the wall at most)
    assert!(a.iter().all(|p| p[0] <= 120.0), "{a:?}");
}

fn defs() -> Vec<LootDef> {
    vec![
        LootDef { id: 0x2bd, key: 3, timed: 0, allowed: 1, kind: 0, mesh: -1 },
        LootDef { id: 0x2be, key: -1, timed: 1, allowed: 0, kind: 1, mesh: 2 },
    ]
}

/// Real level (table row 4): a class-37 guard gets its loot from the class-103 pool, Sly pickpockets it
/// (flight -> held), drops it on a hit (5 s later it is back on the guard), steals it again and delivers it
/// ("mplootkc" counted, guard path cleared).
#[test]
fn p26_pickpocket_real_level() {
    let (l, ds) = tables();
    let lv = l.load_blocking(ds[4]).unwrap();
    let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
    let mut s = LootSys::from_objects(w.objects.iter().enumerate().map(|(i, o)| (i as u32 + 1, o.class)), &defs());
    assert!(!s.guards.is_empty() && s.loots.len() >= 2, "guards {} loot {}", s.guards.len(), s.loots.len());
    assert_eq!(w.objects[s.guards[0].obj as usize - 1].class, CLASS_GUARD);
    assert_eq!(w.objects[s.loots[0].obj as usize - 1].class, CLASS_LOOT);
    let sly = w.objects.iter().position(|o| o.class == sly2_recomp::engine::player::p20::CLASS_SLY).unwrap() as u32 + 1;
    let mut hst = EngineHost { world: Some(w.clone()), ..Default::default() };
    let ps: Vec<[f32; 3]> = (1..=w.objects.len() as u32).map(|o| hst.ee_sub_1a62e8(o)).collect();
    let pos = move |h: u32| ps.get(h as usize - 1).copied().unwrap_or([0.0; 3]);
    let g = s.guards[0].obj;
    s.guards[0].booty = 0x2c0; // remapped to 0x2bd
    s.ee_sub_13cf38(0, &pos);
    let gl = s.guards[0].loot.expect("guard has loot");
    assert_eq!(s.loots[gl].state, 1);
    assert_eq!(s.ee_sub_13c478(gl), 0x2bd);
    // pickpocket: flight (state 2), then held (state 4)
    assert!(s.ee_sub_13d020(g, sly, true, &pos));
    assert_eq!(s.loots[gl].state, 2);
    assert_eq!(s.guards[0].loot, None);
    assert_eq!(s.oid_booty[&sly], 0x2bd);
    assert_eq!(s.guard_save[&0], sly as i32);
    let end = s.loots[gl].fx_end;
    assert!(end >= 0.5 - 1e-6);
    s.update(end, &pos, &|_| true);
    assert_eq!(s.loots[gl].state, 4);
    assert_eq!(s.held_by(sly), Some((gl, 0x2bd)));
    // second pickpocket: guard has nothing, Sly's loot kind matches -> it flies back (state 3 -> 1)
    assert!(s.ee_sub_13d020(g, sly, true, &pos));
    assert_eq!(s.loots[gl].state, 3);
    assert_eq!(s.held_by(sly), None);
    let end = s.loots[gl].fx_end;
    s.update(end, &pos, &|_| true);
    assert_eq!(s.loots[gl].state, 1);
    assert_eq!(s.guards[0].loot, Some(gl));
    // steal again, hit -> drop (state 5), path back to the loot id, back on the guard after 5 s
    assert!(s.ee_sub_13d020(g, sly, true, &pos));
    let end = s.loots[gl].fx_end;
    s.update(end, &pos, &|_| true);
    assert_eq!(s.loots[gl].state, 4);
    s.ee_sub_143380(sly, &pos);
    assert_eq!(s.loots[gl].state, 5);
    assert_eq!(s.guard_save[&0], 0x2bd);
    assert_eq!(s.loots[gl].guard, g);
    let t5 = s.loots[gl].t;
    s.update(t5 + 4.9, &pos, &|_| true);
    assert_eq!(s.loots[gl].state, 5);
    s.update(t5 + 5.0, &pos, &|_| true);
    assert_eq!(s.loots[gl].state, 1);
    assert_eq!(s.guards[0].loot, Some(gl));
    // alerted guard: no pickpocket (activity 0x32)
    s.guards[0].alert = 1;
    assert!(!s.ee_sub_13d020(g, sly, true, &pos));
    s.guards[0].alert = 0;
    // steal and deliver: mplootkc[3] += 1, guard path cleared, loot freed
    assert!(s.ee_sub_13d020(g, sly, true, &pos));
    let end = s.loots[gl].fx_end;
    s.update(end, &pos, &|_| true);
    s.ee_sub_1433c8(sly, &pos);
    assert_eq!(s.mplootkc.get(&3), Some(&1));
    assert_eq!(s.guard_save[&0], -1);
    assert_eq!(s.ee_sub_13d248(0), -1);
    assert_eq!(s.loots[gl].state, 0);
    assert_eq!(s.held_by(sly), None);
    // FUN_001432e0: give loot 0x2be directly (state 6 -> 4)
    s.ee_sub_1432e0(sly, 0x2be, &pos);
    let (h2, id) = s.held_by(sly).unwrap();
    assert_eq!(id, 0x2be);
    assert_eq!(s.loots[h2].state, 4);
    // a timed loot is dropped at DAT_003e1bfc
    s.t_3e1bfc = s.now + 1.0;
    let n = s.now;
    s.update(n + 1.0, &pos, &|_| true);
    assert_eq!(s.loots[h2].state, 5);
}
