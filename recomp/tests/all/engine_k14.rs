//! WP-K14: class 76 C2d0b30 (rail rider swarm; class-71 parts in classes/shared/c71.rs).
use sly2_recomp::engine::classes::guard::slots::GuardHit;
use sly2_recomp::engine::classes::k14::{self, C2d0b30, FUN_0011d1c0, FUN_0014c748};
use sly2_recomp::engine::classes::plugin::EngineHost;
use sly2_recomp::engine::classes::k7::{C2d7f40, K7};
use sly2_recomp::engine::classes::{ClassHost, ClassSet, Inst};
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

#[test]
fn k14_c76_init_and_helpers() {
    let mut c = C2d0b30::default();
    c.ee_sub_1817e8(3, &mut EngineHost::default());
    assert_eq!((c.b.f_180, c.b.f_1a0, c.b.f_1b0, c.b.f_40 & 0xa), (10, [300.0, 50.0, 0.0, 0.0], [0.0, 800.0], 0xa));
    assert_eq!(k14::slot5(76), Some(0x2bb0d8));
    for _ in 0..5 {
        c.ee_sub_182ab8(20.0, 1, 3, 2);
    }
    assert_eq!(c.groups.len(), 4, "at most 4 groups");
    assert_eq!((c.groups[0].hp_1c, c.groups[0].t_2c), (k14::D_2BE8AC, -1.0));
    // FUN_0011d1c0: the speed change is limited to 4500 / s
    let mut v = 0.0;
    let x = FUN_0011d1c0(0.0, 0.0, 1000.0, DT, k14::T_2BE888, &mut v);
    assert!(v <= 4500.0 * DT + 1e-3 && v > 0.0 && x > 0.0, "{x} {v}");
    // FUN_0014c748: segment through a unit sphere
    let h = FUN_0014c748(1.0, [0.0; 3], [-2.0, 0.0, 0.0], [2.0, 0.0, 0.0]).unwrap();
    assert!((h[0].0[0] + 1.0).abs() < 1e-4 && (h[1].0[0] - 1.0).abs() < 1e-4);
    assert_eq!(h[0].1, [-1.0, 0.0, 0.0]);
    assert!(FUN_0014c748(1.0, [0.0; 3], [-2.0, 3.0, 0.0], [2.0, 3.0, 0.0]).is_none());
}

/// every class-76 object built + fed on all 43 levels, slot 14 at load, 300 frames (slots 15 / 18): riders stay on
/// their rail inside [0, L], positions finite
#[test]
#[ignore = "heavy"]
fn k14_c76_all_levels_300_frames() {
    let (loader, descs) = levels();
    let (mut n, mut orig, mut nlv, mut riders) = (0, 0, 0, 0);
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let want = w.objects.iter().filter(|o| o.class == 76).count();
        if want == 0 {
            continue;
        }
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        s.post_load(&mut h);
        s.g.dt = DT;
        let ids: Vec<usize> = s.k14.iter().filter(|x| matches!(x.1, Inst::K14(_))).map(|x| x.0).collect();
        assert_eq!(ids.len(), want, "every class-76 object built");
        for _ in 0..300 {
            s.g.t += DT;
            s.frame(&mut h);
        }
        for &i in &ids {
            let c = s.k14_get(i).unwrap();
            for g in &c.groups {
                let l = g.path.as_ref().map(sly2_recomp::engine::player::p5_attach::FUN_001e6648).unwrap_or(0.0);
                assert_eq!(g.list.len() as i32, g.alive);
                for &r in &g.list {
                    let rd = &c.riders[r];
                    assert!(rd.pos.iter().all(|x| x.is_finite()) && rd.l_3c.is_finite());
                    assert!(rd.l_3c >= -1.0 && rd.l_3c <= l + 1.0, "on the rail {} / {l}", rd.l_3c);
                    assert_eq!(rd.state, 0);
                }
                riders += g.list.len();
            }
        }
        n += want;
        orig += w.objects.iter().filter(|o| o.class == 76 && o.clone_of.is_none()).count();
        nlv += 1;
    }
    println!("class 76: {n} instances ({orig} non-clone) on {nlv} levels, {riders} riders");
    assert_eq!(orig, 22, "class_port.md instance count (clones excluded)");
    assert!(riders > 0);
}

/// a real class-76 swarm: riders spread evenly, hits take hp and kill (state 2, off the group, bands re-laid,
/// last one signals 0x3a), reset (slot 82) brings them back; the touch queues slot 81 which kills the nearest
#[test]
fn k14_c76_real_instance() {
    let (loader, descs) = levels();
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        s.post_load(&mut h);
        s.g.dt = DT;
        let Some(i) = s.k14.iter().find(|x| matches!(&x.1, Inst::K14(c) if c.groups.iter().any(|g| g.alive > 1))).map(|x| x.0) else { continue };
        let c = s.k14_get(i).unwrap().clone();
        let g = c.groups.iter().position(|g| g.alive > 1).unwrap();
        let n = c.groups[g].alive;
        let l = sly2_recomp::engine::player::p5_attach::FUN_001e6648(c.groups[g].path.as_ref().unwrap());
        for (k, &r) in c.groups[g].list.iter().enumerate() {
            assert!((c.riders[r].l_3c - l * (k as f32 + 0.5) / n as f32).abs() < 1e-3, "evenly spread");
            assert_eq!(c.riders[r].hp, k14::D_2BE8AC);
        }
        // hit the first rider: hp 30 - 10 -> state 1, then dead at -30
        let r0 = c.groups[g].list[0];
        let p = c.riders[r0].pos;
        let mk = |amount| GuardHit { amount, pos: [p[0], p[1], p[2], 0.0], vec: [0.0, 0.0, 0.0, 0.0], f_44: 8, ..Default::default() };
        assert!(!s.k14_hit(i, &GuardHit { f_44: 0, ..mk(-100) }, &mut h), "needs +0x44 bit 3");
        assert!(s.k14_hit(i, &mk(-10), &mut h));
        let c2 = s.k14_get(i).unwrap();
        let near = c2.ee_sub_182810(p).unwrap();
        assert_eq!((c2.riders[near].hp, c2.riders[near].state), (k14::D_2BE8AC - 10, 1));
        s.g.t += 0.2;
        s.frame(&mut h);
        assert_eq!(s.k14_get(i).unwrap().riders[near].state, 0, "state 1 ends after 0.15 s");
        let p2 = s.k14_get(i).unwrap().riders[near].pos;
        assert_eq!(s.k14_get(i).unwrap().ee_sub_182810(p2), Some(near));
        assert!(s.k14_hit(i, &GuardHit { pos: [p2[0], p2[1], p2[2], 0.0], ..mk(-30) }, &mut h));
        let c3 = s.k14_get(i).unwrap();
        assert_eq!(c3.riders[near].state, 2);
        assert_eq!((c3.groups[g].alive, c3.groups[g].list.len(), c3.dead.len()), (n - 1, n as usize - 1, 1));
        // reset
        let mut c4 = c3.clone();
        let mut rng = sly2_recomp::engine::physics::driven::GameRng(1);
        let mut cx = k14::Cx { t: 0.0, dt: DT, rng: &mut rng, h: &mut h };
        c4.ee_sub_1819e0(i, &mut cx);
        assert_eq!((c4.groups[g].alive, c4.groups[g].list.len(), c4.dead.len()), (n, n as usize, 0));
        // touch with damage: slot 54 queues slot 81 once, the frame runs it
        s.k14_get(i).unwrap().b.f_188 = 5;
        // FUN_00132b10(other, 0x47): only class-71-derived objects (72-76) are hurt; Sly (class 53) is not
        let player = sly2_recomp::engine::classes::guard::PLAYER_HANDLE;
        s.k14_touch(i, player, &mut h);
        assert!(s.k14_deferred.is_empty());
        let other = i as u32 + 1;
        s.k14_touch(i, other, &mut h);
        s.k14_touch(i, other, &mut h);
        assert_eq!(s.k14_deferred.len(), 1, "+0x18c: once");
        s.frame(&mut h);
        let c5 = s.k14_get(i).unwrap();
        assert_eq!((c5.b.f_18c, c5.groups.iter().map(|g| g.alive).sum::<i32>() < c.groups.iter().map(|g| g.alive).sum::<i32>() - 1), (0, true));
        println!("lsn {} obj {i}: {} groups, group {g} {n} riders over {l}", d.lsn(), c.groups.len());
        return;
    }
    panic!("no class-76 swarm");
}

/// the class-81 launcher (FUN_001d6b00 on a class-112 launcher record, as the Splice method 468 / class 69
/// call it): a real projectile is aimed at the launcher's target, gets +0x260 along it, slot 77 records the
/// launcher frame / target, and the slot-69 hold carries it to the target (closest approach within its speed
/// step: it travels through the target point)
#[test]
#[ignore = "heavy"]
fn k14_c81_launch_travels_and_hits() {
    let (loader, descs) = levels();
    let (mut nl, mut fired, mut hits) = (0, 0, 0);
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        s.post_load(&mut h);
        s.g.dt = DT;
        let ls: Vec<(usize, C2d7f40)> = s
            .k7
            .iter()
            .filter_map(|x| match &x.1 {
                Inst::K7(K7::C2d7f40(c)) => Some((x.0, c.clone())),
                _ => None,
            })
            .collect();
        nl += ls.len();
        let Some(pi) = s.k13.iter().find(|x| matches!(x.1, Inst::K13(_))).map(|x| x.0) else { continue };
        let p0 = s.k13_get(pi).unwrap().p.phys.pos;
        // the launcher whose target is farthest from the projectile
        let tgt = |h: &mut EngineHost, li: usize, l: &C2d7f40| {
            let f = if l.f_60 == 0 { 0 } else { h.ee_sub_15e938(0x2002, l.f_60, li as u32 + 1) };
            let o = if f == 0 { [0.0; 3] } else { h.ee_sub_1a62e8(f) };
            (f, [o[0] + l.f_50[0], o[1] + l.f_50[1], o[2] + l.f_50[2]])
        };
        let Some((li, l, (frame, target))) = ls
            .into_iter()
            .map(|(li, l)| {
                let t = tgt(&mut h, li, &l);
                (li, l, t)
            })
            .max_by(|a, b| {
                let d2 = |t: [f32; 3]| (t[0] - p0[0]).powi(2) + (t[1] - p0[1]).powi(2) + (t[2] - p0[2]).powi(2);
                d2(a.2 .1).total_cmp(&d2(b.2 .1))
            })
        else {
            continue;
        };
        let dist = |a: [f32; 3]| ((a[0] - target[0]).powi(2) + (a[1] - target[1]).powi(2) + (a[2] - target[2]).powi(2)).sqrt();
        let out = s.k14_fire(li, pi, &mut h).expect("class 81 launch");
        fired += 1;
        let c = s.k13_get(pi).unwrap();
        assert_eq!(c.p.f_200, li as u32 + 1, "FUN_0013bc60 owner");
        assert_eq!((c.p.f_210, c.p.f_220), (frame, l.f_50), "slot 77 ran");
        let sp = (out.v[0].powi(2) + out.v[1].powi(2) + out.v[2].powi(2)).sqrt();
        assert!((sp - c.p.f_260).abs() < 1e-2, "launch speed +0x260");
        let d0 = dist(p0);
        let mut best = d0;
        let slow = { let c = s.k13_get(pi).unwrap(); c.p.f_260.min(c.p.f_270).max(1.0) };
        let frames = ((d0 / (slow * DT)) as usize + 120).min(20000);
        for _ in 0..frames {
            s.g.t += DT;
            s.frame(&mut h);
            let p = s.k13_get(pi).unwrap().p.phys.pos;
            best = best.min(dist(p));
        }
        let c = s.k13_get(pi).unwrap();
        let step = c.p.f_270.max(c.p.f_260) * DT;
        println!("lsn {} launcher {li} proj {pi}: d0 {d0} closest {best} (step {step}) speed {sp}", d.lsn());
        if d0 > 1.0 && l.f_a0[1] > 0.0 {
            assert!(best <= step + 1.0, "reaches the target: {best}");
            hits += 1;
        }
    }
    println!("{nl} class-112 launchers, {fired} launches, {hits} aimed away from the projectile and reached");
    assert!(hits > 0, "a real launcher fired at a distant target");
}
