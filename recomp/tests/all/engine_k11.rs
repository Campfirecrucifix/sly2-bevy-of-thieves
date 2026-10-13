//! WP-K11: the guard's class-120 attack records (guard/attack.rs) and the K10-unported classes.
use sly2_recomp::engine::classes::guard::PLAYER_HANDLE;
use sly2_recomp::engine::classes::plugin::EngineHost;
use sly2_recomp::engine::classes::k11::{K11, C2c9a80};
use sly2_recomp::engine::classes::{ClassSet, Inst};
use sly2_recomp::engine::disc::descriptor::Descriptor;
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::player::plugin::{link_classes, PlayerHud};
use sly2_recomp::engine::player::{Jt, JtAnim};
use sly2_recomp::engine::world::{LoadOptions, World};
use std::sync::Arc;

struct Rec(Vec<(u32, i32, i32)>);
impl JtAnim for Rec {
    fn play_aseg(&mut self, f: u32, jts: i32, jtbs: i32) {
        self.0.push((f, jts, jtbs));
    }
}

fn setup(w: &Arc<World>) -> (ClassSet, EngineHost) {
    let mut h = EngineHost { world: Some(w.clone()), player_pos: [1.0e9; 3], ..Default::default() };
    let mut s = ClassSet::build(w, &mut h);
    s.g.k6_players = vec![(PLAYER_HANDLE, [1.0e9; 3])];
    s.post_load(&mut h);
    s.g.dt = 1.0 / 60.0;
    (s, h)
}

fn frame(jt: &mut Jt, s: &mut ClassSet, h: &mut EngineHost, hud: &mut PlayerHud, rec: &mut Rec) {
    link_classes(jt, s, h, rec, hud);
    s.g.k6_players = vec![(PLAYER_HANDLE, jt.phys.pos)];
    s.g.t += s.g.dt;
    s.frame(h);
}

/// Every level: the guards' attack records resolve (count / windows / hit geoms printed).
#[test]
#[ignore = "heavy"]
fn guard_attack_records_all_levels() {
    let (loader, descs) = levels();
    let (mut n, mut with) = (0, 0);
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let (s, _h) = setup(&w);
        for (gi, _) in &s.k6 {
            let g = s.k6_guard(*gi).unwrap();
            n += 1;
            if g.atk.recs.iter().any(|r| r.windows.iter().any(|x| g.atk.geoms.iter().any(|q| q.geom == x.2))) {
                with += 1;
            }
        }
    }
    println!("guards {n}, with a hitting attack record {with}");
    assert!(n > 0 && with * 2 > n, "{with} / {n}");
}

/// A real guard notices Sly standing in front of it, chases, starts its class-120 attack and its hit geom
/// damages Sly through K7 -> classes_link -> TakeDamage.
#[test]
fn guard_attack_damages_sly() {
    let (loader, descs) = levels();
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let (s0, _) = setup(&w);
        let gis: Vec<usize> = s0.k6.iter().map(|x| x.0).collect();
        for gi in gis {
            let (mut s, mut h) = setup(&w);
            let g = s.k6_guard(gi).unwrap();
            let Some(r) = g.atk.recs.iter().find(|r| r.windows.iter().any(|x| g.atk.geoms.iter().any(|q| q.geom == x.2))) else { continue };
            let reach = (r.rec.f_50[0] + 1.0).max(60.0).min(r.rec.f_50[1] - 1.0);
            let (p, yaw) = (g.body.pos, g.body.yaw);
            let mut jt = Jt::new([p[0] + yaw.cos() * reach, p[1] + yaw.sin() * reach, p[2] + 30.0]);
            let hp0 = jt.dmg.hp;
            let (mut hud, mut rec) = (PlayerHud::default(), Rec(vec![]));
            for f in 0..600 {
                frame(&mut jt, &mut s, &mut h, &mut hud, &mut rec);
                if hud.hits_taken > 0 {
                    let g = s.k6_guard(gi).unwrap();
                    println!("lsn {} guard {gi} class {}: attack started {} hit after {f} frames hp {hp0} -> {}", d.lsn(), g.class, g.atk.started, jt.dmg.hp);
                    assert!(g.atk.started > 0);
                    assert!(jt.dmg.hp < hp0);
                    return;
                }
            }
            let g = s.k6_guard(gi).unwrap();
            println!("lsn {} guard {gi}: no hit (child {:?}, started {})", d.lsn(), g.brain.child, g.atk.started);
        }
    }
    panic!("no guard attack hit Sly");
}

/// Every level: the K11 classes build, then 300 frames run (attack driver included); world matrices stay finite.
#[test]
#[ignore = "heavy"]
fn k11_classes_all_levels_300_frames() {
    let (loader, descs) = levels();
    let mut total = 0;
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let (mut s, mut h) = setup(&w);
        let n20 = w.objects.iter().filter(|o| o.class == 20 || o.class == 14).count();
        assert_eq!(s.k11.len(), n20, "lsn {}", d.lsn());
        total += n20;
        for _ in 0..300 {
            s.g.t += s.g.dt;
            s.frame(&mut h);
        }
        for (_, x) in &s.k11 {
            let Inst::K11(k) = x else { panic!() };
            let K11::C2c9a80(c) = &**k else { continue };
            assert!(c.f_a0.iter().flatten().all(|v| v.is_finite()));
        }
    }
    println!("class 20 instances {total}");
    assert!(total > 0);
}

/// A real class-20 node: its local matrix is the loaded one; slot 28 places it at a world position (the
/// world getter slot 30 returns it), slot 36 composes, slot 9 re-parents keeping the world position.
#[test]
fn k11_c20_transform_node() {
    let (loader, descs) = levels();
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let (mut s, mut h) = setup(&w);
        let Some(&(i, _)) = s.k11.iter().find(|x| w.objects[x.0].class == 20) else { continue };
        let m = sly2_recomp::engine::gfx::attach::xfm_of(&w, w.objects[i].clone_of.unwrap_or(i)).unwrap().1;
        let Some(K11::C2c9a80(c)) = s.k11_get(i) else { panic!() };
        assert_eq!(c.f_60[3][..3], m[3][..], "loaded translation");
        c.ee_sub_2312a0(i, [100.0, 200.0, 300.0], &mut h);
        assert_eq!(c.f_54, -1, "dirty");
        let wm = c.ee_sub_231440(&mut h);
        assert_eq!(wm[3][..3], [100.0, 200.0, 300.0]);
        let mut t = sly2_recomp::engine::anim::pose::IDENTITY;
        t[3] = [1.0, 2.0, 3.0, 1.0];
        c.ee_sub_231598(i, &t, &mut h);
        let p = c.f_60[3];
        assert!((p[0] - 101.0).abs() < 1e-3 && (p[2] - 303.0).abs() < 1e-3, "{p:?}");
        let before = c.f_60[3];
        let mut d2 = C2c9a80::default();
        d2.ee_sub_2311b0(i, c, &mut h);
        assert_eq!(d2.f_60, c.f_60, "clone copies");
        c.ee_sub_2315f0(i, 0, 0, 1, &mut h);
        assert_eq!(c.f_50, 0);
        assert_eq!(c.f_60[3], before, "world kept (identity ALO)");
        let (_, alo) = c.ee_sub_231550();
        assert_eq!(alo, sly2_recomp::engine::classes::k11::DAT_002C44F0);
        println!("lsn {} class-20 object {i} oid {:#x}", d.lsn(), w.objects[i].oid);
        return;
    }
    panic!("no class-20 object");
}

/// Class 14 outlines of the real levels: load-post leaves no clockwise outline (kind 4 reversed to 3, the
/// reversed knots then classify as 3) and lists them in DAT_003e1a4c; a synthetic square checks the winding.
#[test]
fn k11_c14_outline_winding() {
    use sly2_recomp::engine::classes::k11::{C2d4de0, L_3E1A4C};
    use sly2_recomp::engine::player::p5_attach::Path;
    let sq = |cw: bool| {
        let mut p = vec![[0.0, 0.0, 0.0], [100.0, 0.0, 0.0], [100.0, 100.0, 0.0], [0.0, 100.0, 0.0], [0.0, 0.0, 0.0]];
        if cw {
            p.reverse();
        }
        let mut c = C2d4de0::default();
        c.ee_sub_1bc088(Some(Path::polyline(p, true)));
        c
    };
    let mut h = EngineHost::default();
    assert_eq!(sq(false).ee_sub_1bc2f8(), 3);
    let mut c = sq(true);
    assert_eq!(c.ee_sub_1bc2f8(), 4);
    c.ee_sub_1bc570(0, &mut h);
    assert_eq!(c.f_190, 3);
    assert_eq!(c.ee_sub_1bc2f8(), 3, "reversed knots wind +1");
    assert_eq!(c.f_1ec, 0x40);
    let (loader, descs) = levels();
    let mut kinds = [0usize; 5];
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let (mut s, _h) = setup(&w);
        let list = s.g.list(L_3E1A4C).clone();
        for (i, x) in &s.k11 {
            let Inst::K11(k) = x else { panic!() };
            let K11::C2d4de0(c) = &**k else { continue };
            assert!(list.contains(&(*i as u32 + 1)));
            assert_ne!(c.f_190, 4);
            if c.f_190 == 3 {
                assert_eq!(c.ee_sub_1bc2f8(), 3);
            }
            kinds[c.f_190 as usize] += 1;
        }
    }
    println!("class-14 winding kinds {kinds:?}");
    assert!(kinds.iter().sum::<usize>() > 0);
}

fn levels() -> (LevelLoader, Vec<Descriptor>) {
    crate::common::levels() // public: ISO level table instead of the analysis DB
}
