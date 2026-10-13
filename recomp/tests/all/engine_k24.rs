//! WP-K24 tests: the guard 0xb0 lifted goal (guard/lift.rs) and the class-102 guard hooks routed to the
//! ported guards (classes/k24.rs `K24Host`).
use sly2_recomp::engine::classes::guard::brain::Child;
use sly2_recomp::engine::classes::guard::lift::{C2d0420, ANIMS_2A55A0, BIT_16};
use sly2_recomp::engine::classes::plugin::EngineHost;
use sly2_recomp::engine::classes::{ClassHost, ClassSet, Inst, NullHost};
use sly2_recomp::engine::world::{LoadOptions, World};
use std::sync::Arc;

const DT: f32 = 1.0 / 60.0;

use crate::common as k20;

/// C2d0420 setstate / update: anims 18..21, +0x32c (state < 2), 2 / 3 end at ground contact, noises every 1 s
/// in 1 / 4, deactivate -> -1 and bit 0x16 cleared
#[test]
fn k24_b0_states() {
    let mut h = NullHost;
    let mut b = C2d0420::new();
    assert_eq!(b.f_30, -1);
    b.ee_sub_17a730(0, 1.0, &mut h);
    assert!(b.active);
    // slot 6 init: +0x30 = -1; activate's setstate(0) pushes anim 18, +0x32c = 1
    assert_eq!(b.f_30, 0);
    assert_eq!(b.anim, Some((18, 1.0)));
    assert_eq!(b.w_32c.take(), Some(1));
    for s in 1..5 {
        b.ee_sub_17a8c8(s, 2.0);
        assert_eq!(b.anim, Some((ANIMS_2A55A0[s as usize], 2.0)));
        assert_eq!(b.w_32c.take(), Some((s < 2) as i32));
    }
    assert_eq!(ANIMS_2A55A0, [18, 19, 20, 21, 19]);
    // state 4: a noise each second
    b.f_38 = 2.0;
    for k in 0..120 {
        b.ee_sub_17a7f8(false, 2.0 + k as f32 * DT);
    }
    assert_eq!(b.noises, 1);
    // 3 (knocked) ends at ground contact only
    b.ee_sub_17a8c8(3, 5.0);
    b.ee_sub_17a7f8(false, 5.1);
    assert_eq!(b.f_30, 3);
    b.ee_sub_17a7f8(true, 5.2);
    assert_eq!((b.f_30, b.anim, b.w_32c), (-1, None, Some(0)));
    assert!(b.ee_sub_17a9c0());
    b.ee_sub_17a7b8(6.0);
    assert!(!b.active);
    assert_eq!(b.w_clear, BIT_16);
}

/// a level with class-102 bombs and liftable guards (class 57 / 58: FUN_00206eb0(0xb))
fn level() -> Option<(Arc<World>, Vec<usize>)> {
    let (loader, descs) = k20::levels();
    for d in descs {
        let Ok(lv) = crate::common::load_blocking(&loader, d) else { continue };
        let Ok(w) = World::load(&lv, LoadOptions::default()) else { continue };
        let bombs: Vec<usize> = w.objects.iter().enumerate().filter(|o| o.1.class == 102).map(|o| o.0).collect();
        if !bombs.is_empty() && w.objects.iter().any(|o| o.class == 57 || o.class == 58) {
            return Some((Arc::new(w), bombs));
        }
    }
    None
}

fn guard_mut(s: &mut ClassSet, i: usize) -> &mut sly2_recomp::engine::classes::guard::c56::Guard {
    match s.k6.iter_mut().find(|x| x.0 == i) {
        Some((_, Inst::Guard(g))) => g,
        _ => panic!("guard {i}"),
    }
}

/// real level: a class-102 bomb set to state 5 grabs the nearest ported guard through `K24Host` (link +0x138,
/// lift velocity, brain message 6 -> the 0xb0 goal), holds it (component 4 near the anchor), then knocks it
/// out after 3 s (component 2, record unlinked, LO Remove: the guard leaves the world and its updates stop)
#[test]
fn k24_c102_lifts_real_guard() {
    let (w, bombs) = level().expect("a level with class 102 and guards");
    let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
    let mut s = ClassSet::build(&w, &mut h);
    s.post_load(&mut h);
    for _ in 0..30 {
        s.g.t += DT;
        s.g.dt = DT;
        s.frame(&mut h);
    }
    let bomb = bombs[0];
    let anchor = s.k15_101_get(bomb).and_then(|c| c.k23.as_ref()).map(|e| e.anchor(bomb)).expect("class-102 bomb");
    let ap = h.ee_sub_1a62e8(anchor);
    // every guard but one liftable far away; the chosen one 50 from the anchor
    let gi = s
        .k6
        .iter()
        .find(|x| matches!(&x.1, Inst::Guard(g) if g.class == 57 || g.class == 58))
        .map(|x| x.0)
        .expect("class 57/58 guard");
    let ids: Vec<usize> = s.k6.iter().map(|x| x.0).collect();
    for i in ids {
        if let Some((_, Inst::Guard(g))) = s.k6.iter_mut().find(|x| x.0 == i) {
            g.body.pos = if i == gi { [ap[0] + 50.0, ap[1], ap[2]] } else { [ap[0] + 1.0e5, ap[1], ap[2]] };
        }
    }
    s.with_k24(&mut h, |s, h| {
        s.k15_101_with(bomb, h, |c, cx| c.slot75(bomb, 5, cx));
    });
    let e = s.k15_101_get(bomb).unwrap().k23.clone().unwrap();
    assert_eq!((e.f_1b4, e.grabs), (gi as u32 + 1, 1), "the bomb grabbed the ported guard");
    {
        let g = guard_mut(&mut s, gi);
        assert_eq!(g.f_138, vec![bomb as u32 + 1]);
        assert!(g.po.as_ref().is_some_and(|p| p.vel[2] == 400.0), "{:?}", g.po);
        assert!(g.body.k24_air);
        assert_eq!(g.brain.sub.code, 6);
        assert!(matches!(g.brain.child, Child::Lifted));
        let b = g.brain.b0.as_ref().expect("0xb0 component");
        assert_eq!((b.f_30, b.active), (0, true));
        assert_eq!(g.f_32c, 1);
        assert_eq!(g.f_f80, -1);
    }
    let z0 = guard_mut(&mut s, gi).body.pos[2];
    let (mut states, mut zmax, mut gone_at) = (std::collections::BTreeSet::new(), z0, None);
    for f in 0..300 {
        s.g.t += DT;
        s.g.dt = DT;
        s.frame(&mut h);
        let g = guard_mut(&mut s, gi);
        zmax = zmax.max(g.body.pos[2]);
        if let Some(b) = &g.brain.b0 {
            states.insert(b.f_30);
        }
        if g.removed && gone_at.is_none() {
            gone_at = Some(f);
        }
    }
    let g = guard_mut(&mut s, gi);
    eprintln!("0xb0 states {states:?}, lifted {:.1}, knocked out at frame {gone_at:?}", zmax - z0);
    assert!(zmax > z0 + 10.0, "the guard was lifted");
    assert!(states.contains(&4), "held close: {states:?}");
    assert!(g.removed, "knocked out (vtable +0x4c)");
    assert!(g.f_138.is_empty(), "record unlinked");
    // the bomb's setstate(2) on knock-out (the removed guard's goal stays in state 2: no more updates)
    assert_eq!(g.k24_comp_b0(), Some(2));
    let e = s.k15_101_get(bomb).unwrap().k23.clone().unwrap();
    assert_eq!((e.knockouts, e.f_1b4), (1, 0));
    // FUN_0015e9c8(0x38) through the routing host no longer lists it
    let mut got = Vec::new();
    s.with_k24(&mut h, |_, h| got = h.query_15e9c8(0x38));
    assert!(!got.contains(&(gi as u32 + 1)));
    assert!(!got.is_empty());
}

/// every level with guards and class-102 bombs runs 300 frames through the routing host (guards and bombs
/// stay finite); each guard is listed by FUN_0015e9c8(0x38) and accepts / rejects hit 0xb by class
#[test]
#[ignore = "heavy"]
fn k24_levels() {
    let (loader, descs) = k20::levels();
    let (mut levels, mut guards, mut liftable) = (0, 0, 0);
    for d in descs {
        let Ok(lv) = crate::common::load_blocking(&loader, d) else { continue };
        let Ok(w) = World::load(&lv, LoadOptions::default()) else { continue };
        if !w.objects.iter().any(|o| (57..=67).contains(&o.class)) {
            continue;
        }
        levels += 1;
        let w = Arc::new(w);
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        s.post_load(&mut h);
        for _ in 0..300 {
            s.g.t += DT;
            s.g.dt = DT;
            s.frame(&mut h);
        }
        let mut listed = Vec::new();
        let mut ok = Vec::new();
        let ids: Vec<u32> = s.k6.iter().map(|x| x.0 as u32 + 1).collect();
        s.with_k24(&mut h, |_, h| {
            listed = h.query_15e9c8(0x38);
            ok = ids.iter().map(|&o| h.k23_guard_206eb0(o, 0xb)).collect();
        });
        for (x, ok) in s.k6.iter().zip(ok) {
            let Inst::Guard(g) = &x.1 else { continue };
            guards += 1;
            assert!(listed.contains(&(x.0 as u32 + 1)));
            assert!(g.body.pos.iter().all(|v| v.is_finite()));
            assert!(g.po.is_none() && !g.removed);
            if ok {
                assert!(g.class == 57 || g.class == 58, "class {} accepts hit 0xb", g.class);
                liftable += 1;
            }
        }
    }
    eprintln!("K24: {guards} guards on {levels} levels, {liftable} liftable by class 102");
    assert!(levels >= 20 && guards >= 100, "levels {levels} guards {guards}");
}

/// EngineHost world-data hooks wired by K24: parent_10 = `Lo::owner`, vtable +0xfc / FUN_001a6318 /
/// FUN_001a6f00(obj, 0, p) agree with FUN_001a62e8, vol_centre_110 = the class 22-27 volume sphere centre
#[test]
fn k24_enginehost_world_hooks() {
    let (loader, descs) = k20::levels();
    let (mut levels, mut owned, mut vols, mut turned) = (0, 0, 0, 0);
    for d in descs.into_iter().take(12) {
        let Ok(lv) = crate::common::load_blocking(&loader, d) else { continue };
        let Ok(w) = World::load(&lv, LoadOptions::default()) else { continue };
        levels += 1;
        let w = Arc::new(w);
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        for (i, o) in w.objects.iter().enumerate() {
            let me = i as u32 + 1;
            assert_eq!(h.parent_10(me), o.owner.map_or(0, |p| p as u32 + 1));
            owned += o.owner.is_some() as usize;
            let p = h.ee_sub_1a62e8(me);
            let m = h.vcall_fc(me);
            let r = h.ee_sub_1a6318(me);
            assert_eq!([m[3][0], m[3][1], m[3][2]], p);
            for k in 0..3 {
                assert_eq!([m[k][0], m[k][1], m[k][2]], [r[k][0], r[k][1], r[k][2]]);
            }
            turned += (r[0] != [1.0, 0.0, 0.0, 0.0]) as usize;
            assert_eq!(h.ee_sub_1a6f00_to_world(i, [0.0; 3]), p);
            if (22..=27).contains(&o.class) {
                let c = h.vol_centre_110(i);
                assert!(c.iter().all(|v| v.is_finite()));
                vols += (c != [0.0; 3]) as usize;
            }
        }
    }
    eprintln!("K24 hooks: {levels} levels, {owned} owned objects, {turned} turned, {vols} volume centres");
    assert!(levels >= 10 && owned > 100 && turned > 10 && vols > 10);
}
