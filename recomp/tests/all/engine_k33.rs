//! WP-K33 tests: pickpocket (JT FUN_001938c0 -> attach kind 7 -> target slot 50 = C2cb9f8 FUN_0013d020), the
//! class-103 loot def rows of the level load (Splice fields 1152 / 1153 / 1155 / 1161 / 1163) and the
//! timed-loot countdown HUD requests (FUN_001618f8 / FUN_00161958).
use sly2_recomp::engine::classes::plugin::EngineHost;
use sly2_recomp::engine::classes::{ClassHost, ClassSet};
use sly2_recomp::engine::input::pad::PadState;
use sly2_recomp::engine::physics::CollisionWorld;
use sly2_recomp::engine::player::loot::{defs_of_props, LootSys};
use sly2_recomp::engine::player::p4::*;
use sly2_recomp::engine::player::*;
use sly2_recomp::engine::world::{LoadOptions, World};
use std::sync::Arc;
use crate::common::{levels, load_blocking};


/// O near a class-37 use point (DAT_003e1414, category 0x80): attach kind 7, dropped at once, slot 50 queued.
#[test]
fn k33_jt_pickpocket_queues_slot50() {
    assert!(FUN_00132b10(37, 0x23));
    let cand = TargetCand { obj: 5, class: 37, f_10c: 0, radius: None, member: 0xe0, prio: 0, group: 0, mask: 7, sub: 0, pos: [100.0, 0.0, 0.0], category: 0x80, f_e8: false };
    let mut jt = Jt::new([0.0, 0.0, JT_RADIUS]);
    jt.t_ground = 0.0;
    jt.FUN_00194a40(std::slice::from_ref(&cand), 0xffff);
    assert_eq!(jt.targets.len(), 1);
    jt.feed_pad(&PadState { buttons: 0x20, pressed: 0x20, ..Default::default() });
    jt.update(&CollisionWorld::default(), JtInput::default(), 1.0 / 60.0, &mut ());
    assert!(jt.calls.contains(&(CALL_ATTACH, 7)), "{:?}", jt.calls);
    assert_eq!((jt.attach.target, jt.k7_pending), (Some(5), true));
    assert!(jt.slot50.is_empty());
    // next frame: 0x1911a8 kind 7 -> drop + slot 50
    jt.feed_pad(&PadState::default());
    jt.update(&CollisionWorld::default(), JtInput::default(), 1.0 / 60.0, &mut ());
    assert_eq!(jt.slot50, vec![5]);
    assert_eq!((jt.attach.sub, jt.attach.target), (-1, None));
    // out of range: nothing
    let mut far = Jt::new([400.0, 0.0, JT_RADIUS]);
    far.t_ground = 0.0;
    far.FUN_00194a40(std::slice::from_ref(&cand), 0xffff);
    far.feed_pad(&PadState { buttons: 0x20, pressed: 0x20, ..Default::default() });
    far.update(&CollisionWorld::default(), JtInput::default(), 1.0 / 60.0, &mut ());
    far.update(&CollisionWorld::default(), JtInput::default(), 1.0 / 60.0, &mut ());
    assert!(far.slot50.is_empty());
}

/// Every class-103 object's load writes its def rows; Paris (row 2): all 8 loot have rows, the holders take
/// the default id 700 row, the pickpocket flies it to Sly.
#[test]
#[ignore = "heavy"]
fn k33_loot_defs_and_pickpocket_paris() {
    let (loader, descs) = levels();
    let mut with_defs = 0;
    for row in [2usize, 3] {
        let lv = load_blocking(&loader, descs[row]).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut set = ClassSet::build(&w, &mut h);
        set.post_load(&mut h);
        let n = w.objects.len() as u32;
        let ps: Vec<[f32; 3]> = (1..=n).map(|o| h.ee_sub_1a62e8(o)).collect();
        let pos = move |o: u32| ps.get(o as usize - 1).copied().unwrap_or([0.0; 3]);
        let mut s = LootSys::from_world(&w, &set.k31, &pos);
        for l in &s.loots {
            println!("row {row} loot {}: {} defs", l.obj, l.defs.len());
            assert!(l.defs.iter().all(|d| d.mesh != 0));
            with_defs += !l.defs.is_empty() as usize;
        }
        for g in &s.guards {
            let par = h.parent_10(g.obj);
            if par != 0 {
                // K33: an XFM holder on a placed owner's bone sits at the owner (FUN_001a62e8 = vt+0xfc world)
                let (a, b) = (pos(par), pos(g.obj));
                assert!((0..3).map(|k| (a[k] - b[k]).powi(2)).sum::<f32>() < 1000.0f32.powi(2), "{a:?} {b:?}");
                println!("  parent {} class {} pos {:?} parent {}", par, w.objects[par as usize - 1].class, pos(par), h.parent_10(par));
            }
            println!("row {row} holder {} alert {} kind {} pos {:?} parent {} loot {:?} sly {:?}", g.obj, g.alert, g.kind, pos(g.obj), h.parent_10(g.obj), g.loot, w.objects.iter().position(|o| o.class == sly2_recomp::engine::player::p20::CLASS_SLY).map(|i| pos(i as u32 + 1)));
        }
        if row != 2 {
            continue;
        }
        assert_eq!((s.guards.len(), s.loots.len()), (2, 8));
        assert!(s.loots.iter().all(|l| !l.defs.is_empty()));
        assert_eq!(s.guards.iter().map(|g| g.timer).collect::<Vec<_>>(), set.k31.iter().map(|x| x.1.f_18c).collect::<Vec<_>>());
        let gi = s.guards.iter().position(|g| g.loot.is_some()).expect("a holder got its loot");
        let l = s.guards[gi].loot.unwrap();
        assert_eq!(s.loots[l].defs[s.loots[l].def.unwrap()].id, s.guards[gi].booty);
        let sly = w.objects.iter().position(|o| o.class == sly2_recomp::engine::player::p20::CLASS_SLY).unwrap() as u32 + 1;
        s.now = 10.0;
        assert!(s.ee_sub_13d020(s.guards[gi].obj, sly, true, &pos));
        assert_eq!(s.loots[l].state, 2);
        let timed = s.loots[l].defs[s.loots[l].def.unwrap()].timed != 0;
        assert_eq!(s.hud.len(), timed as usize);
        if timed {
            assert_eq!(s.hud[0], Some(s.guards[gi].timer + 0.99));
            assert_eq!(s.t_3e1bfc, 10.0 + s.guards[gi].timer + 0.99);
        }
        let end = s.loots[l].fx_end;
        s.update(end, &pos, &|_| true);
        assert_eq!(s.held_by(sly).map(|x| x.0), Some(l));
    }
    assert!(with_defs >= 8);
}

/// Row fields (flags 0x2080) go to the current row; 1153 ends it; 0x20 rows max.
#[test]
fn k33_defs_of_props_rows() {
    use sly2_recomp::engine::world::props::{ArrayItem, Prop, PropValue};
    let arr = |v: &[i32]| PropValue::Array(v.iter().map(|x| ArrayItem::One(6, x.to_le_bytes().to_vec())).collect());
    let sc = |v: i32| PropValue::Scalar(v.to_le_bytes().to_vec());
    let p = |field: i16, value: PropValue| Prop { field, value };
    let props = vec![
        p(1152, arr(&[700, 36, -1])),
        p(1154, arr(&[0x1abc])),
        p(1155, sc(0x1aaf)),
        p(1161, PropValue::Scalar(vec![1])),
        p(1163, PropValue::Scalar(vec![1])),
        p(1153, PropValue::Array(vec![])),
        p(1155, sc(9)), // no current row: ignored
        p(1152, arr(&[701, -1, 2])),
    ];
    let d = defs_of_props(&props);
    assert_eq!(d.len(), 2);
    assert_eq!((d[0].id, d[0].key, d[0].kind, d[0].mesh, d[0].timed, d[0].allowed), (700, 36, -1, 0x1aaf, 1, 1));
    assert_eq!((d[1].id, d[1].key, d[1].kind, d[1].mesh, d[1].timed), (701, -1, 2, -1, 0));
    let many: Vec<Prop> = (0..40).map(|i| p(1152, arr(&[i, 0, 0]))).collect();
    assert_eq!(defs_of_props(&many).len(), 0x20);
}
