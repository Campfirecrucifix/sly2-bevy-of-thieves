//! WP-P11: Sly's cane (0x18f0a8 / 0x18ee38 / 0x18e540, ASEG events 0xf / 0x10 on geom 0x14c, REPLACED contact
//! -> FUN_001f7598 -> guard slot 74 0x207468 through the class set hook `p11_cane::cane_hits_to_guards`),
//! target defaults 0x182c70, FUN_0011f088.
use sly2_recomp::engine::classes::guard::PLAYER_HANDLE;
use sly2_recomp::engine::classes::plugin::EngineHost;
use sly2_recomp::engine::classes::ClassSet;
use sly2_recomp::engine::disc::descriptor::Descriptor;
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::physics::query::CollisionWorld;
use sly2_recomp::engine::player::p11_cane::{self as cane, CaneAseg, CaneData, CaneGeom};
use sly2_recomp::engine::player::p6::{FUN_00182c70_defaults, P6Level};
use sly2_recomp::engine::player::{Jt, JtInput, JTS_STAND};
use sly2_recomp::engine::world::{LoadOptions, World};
use std::sync::Arc;

const DT: f32 = 1.0 / 60.0;

fn levels() -> (LevelLoader, Vec<Descriptor>) {
    crate::common::levels() // public: ISO level table instead of the analysis DB
}

/// The 0x1ce0 swing as Sly's data has it (geom on 0.133, off 0.333, combo window 0.5, length 0.833).
fn synthetic() -> CaneData {
    let a = CaneAseg { len: 0.8333333, ev: vec![(0.13333334, 0xf, 0x14c), (0.33333334, 0x10, 0x14c), (0.5, 0xb, 0x164)] };
    let mut d = CaneData::default();
    for f in (0x1ce0..=0x1d04).step_by(4).chain([0x1d14, 0x1d18]) {
        d.asegs.insert(f, a.clone());
    }
    d.geom = Some(CaneGeom { kind: 2, amount: -10, center: [0.0, 0.0, 19.6], radius: 211.9 });
    d
}

fn standing() -> Jt {
    let mut jt = Jt::new([0.0, 0.0, 35.0]);
    jt.t = 10.0;
    jt.t_ground = 1.0e9; // never walks off (no floor in the empty collision world)
    jt.cane.data = synthetic();
    jt
}

fn step(jt: &mut Jt, held: u32, pressed: u32) {
    jt.pad_pending = Some((held, pressed));
    jt.update(&CollisionWorld::default(), JtInput::default(), DT, &mut ());
}

#[test]
fn p11_square_swing_geom_window_and_combo() {
    let mut jt = standing();
    step(&mut jt, 0x80, 0x80);
    assert_eq!((jt.jts, jt.cane.jtcs), (JTS_STAND, 0xc), "Square standing -> jtcs 0xc (FUN_0018ee38)");
    assert!(jt.cane.f_18ec && !jt.cane.geom_on);
    let mut on = vec![];
    let mut combo_at = None;
    for f in 1..60 {
        step(&mut jt, 0, 0);
        if jt.cane.geom_on {
            on.push(f);
        }
        if combo_at.is_none() && jt.cane.f_18e4 {
            combo_at = Some(f);
        }
        if jt.cane.jtcs == -1 {
            break;
        }
    }
    println!("geom on frames {on:?} combo {combo_at:?}");
    assert!(on.len() >= 10 && on.len() <= 14, "geom active ~0.2 s");
    assert!(!jt.cane.f_18ec, "Square released -> +0x18ec cleared");
    assert_eq!(jt.cane.jtcs, -1, "ASEG done -> jtcs -1");
    // combo: second Square inside the window alternates to the odd swing
    let mut jt = standing();
    step(&mut jt, 0x80, 0x80);
    while !jt.cane.f_18e4 {
        step(&mut jt, 0, 0);
    }
    step(&mut jt, 0x80, 0x80);
    assert_eq!(jt.cane.jtcs, 0xd, "combo follow-up 0xc -> 0xd");
    // Triangle -> 0x16
    let mut jt = standing();
    step(&mut jt, 0x10, 0x10);
    assert_eq!(jt.cane.jtcs, 0x16);
}

#[test]
fn p11_cane_hit_dedupe_and_range() {
    let mut jt = standing();
    jt.yaw = 0.0;
    let feet = [0.0, 0.0, 35.0 - 30.0];
    let near = (5u32, [feet[0] + 150.0, feet[1], feet[2]]);
    let far = (6u32, [feet[0] + 400.0, feet[1], feet[2]]);
    assert!(jt.FUN_001f7410_cane(&[near, far]).is_empty(), "geom off -> no hit");
    jt.cane.geom_on = true;
    let h = jt.FUN_001f7410_cane(&[near, far]);
    assert_eq!(h.len(), 1);
    assert_eq!((h[0].src, h[0].target, h[0].kind, h[0].amount), (PLAYER_HANDLE, 5, 2, -10));
    assert!(jt.FUN_001f7410_cane(&[near]).is_empty(), "0x3e15c0 dedupe while the geom stays on");
    jt.cane.geom_off();
    jt.cane.geom_on = true;
    assert_eq!(jt.FUN_001f7410_cane(&[near]).len(), 1, "a new swing hits again");
}

#[test]
fn p11_target_defaults_and_11f088() {
    assert_eq!(FUN_00182c70_defaults(0xb), Some((1, f32::MAX)));
    assert_eq!(FUN_00182c70_defaults(0xc), Some((7, f32::MAX)));
    assert_eq!(FUN_00182c70_defaults(0xd), Some((7, f32::MAX)));
    assert_eq!(FUN_00182c70_defaults(0xa), None);
    let mut v = [100.0, 0.0, 0.0];
    let mut last = 100.0;
    for _ in 0..120 {
        v = sly2_recomp::engine::player::p5_attach::FUN_0011f088(DT, v, [0.0; 3], cane::DAT_002BF820, &mut [0.0; 3]);
        assert!(v[0] <= last && v[0] >= 0.0);
        last = v[0];
    }
    assert!(v[0] < 1e-3, "decayed to the goal: {v:?}");
}

/// Sly's real cane data + a real guard: Square next to the guard -> the swing's geom window -> one hit kind 2
/// / -10 delivered to the guard's slot 74 (pending SM event 0xc in `f_damage`). Hook: the class set is driven
/// directly (as the Bevy `cane_hits` system does with the `Classes` resource).
#[test]
fn p11_cane_attack_hits_guard_on_real_level() {
    let (loader, descs) = levels();
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let p6 = P6Level::load(&w);
        let Some(g) = p6.cane.geom.clone() else { continue };
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        s.post_load(&mut h);
        s.g.dt = DT;
        let Some(gi) = s.k6.first().map(|x| x.0) else { continue };
        assert!(p6.cane.asegs.contains_key(&0x1ce0), "swing ASEG 0x1ce0 loaded");
        assert_eq!((g.kind, g.amount), (2, -10), "part 0x14c hit kind / amount");
        let gpos = s.k6_guard(gi).unwrap().body.pos;
        let mut jt = Jt::new([gpos[0] - 120.0, gpos[1], gpos[2] + 30.0]);
        p6.apply(&mut jt);
        jt.t = 10.0;
        jt.t_ground = 1.0e9;
        let n0 = s.k6_guard(gi).unwrap().f_damage.len();
        let mut hits = vec![];
        step(&mut jt, 0x80, 0x80);
        for _ in 0..60 {
            hits.extend(cane::cane_hits_to_guards(&mut jt, &mut s, &mut h, &mut ()));
            if jt.cane.jtcs == -1 {
                break;
            }
            step(&mut jt, 0, 0);
            jt.phys.pos = [gpos[0] - 120.0, gpos[1], gpos[2] + 30.0]; // no floor: hold Sly in place
        }
        println!("lsn {} guard {gi} at {gpos:?}: hits {hits:?}", d.lsn());
        let hits: Vec<_> = hits.into_iter().filter(|x| x.target == gi as u32 + 1).collect();
        assert_eq!(hits.len(), 1, "one hit per swing on the guard");
        assert_eq!((hits[0].kind, hits[0].amount), (2, -10));
        assert_eq!(s.k6_guard(gi).unwrap().f_damage.len(), n0 + 1, "guard slot 74 took the hit");
        return;
    }
    panic!("no level with Sly and guards");
}

/// Real level row 3: the rope target carries the FUN_00182c70 defaults (class 13: mask 7, range FLT_MAX)
/// and is picked by Sly's jt+0x1000 = 1.
#[test]
fn p11_real_rope_target_defaults() {
    let iso = Arc::new(IsoImage::open(concat!(env!("CARGO_MANIFEST_DIR"), "/../Sly 2 - Band of Thieves (USA).iso")).expect("ISO"));
    let t = sly2_recomp::engine::disc::descriptor::static_table(&iso.read_file("SCUS_973.16").unwrap()).unwrap();
    let l = LevelLoader::new(iso, ModOverlay::default());
    let boot = crate::common::load_blocking(&l, t[25]).unwrap();
    let lv = crate::common::load_blocking(&l, boot.header.level_table.entries[3].desc).unwrap();
    let w = World::load(&lv, LoadOptions::default()).unwrap();
    let ts = sly2_recomp::engine::player::p6::level_targets(&w);
    let r = ts.iter().find(|t| t.obj.id == 2967).expect("rope 2967");
    assert_eq!((r.obj.mask, r.obj.range), (7, f32::MAX));
    let mut kinds = std::collections::BTreeMap::new();
    for t in &ts {
        *kinds.entry(w.objects[t.obj.id].class).or_insert(0) += 1;
    }
    println!("targets by class {kinds:?}");
    assert!(ts.iter().all(|t| t.obj.mask & 1 != 0), "every path target matches Sly's jt+0x1000 = 1");
}
