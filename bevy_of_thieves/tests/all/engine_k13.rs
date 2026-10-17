//! WP-K13: default EngineHost player (FUN_001c4840 = Sly from PlayerFocus) and the K13 classes
//! (81 C2d3d20 projectile; callees in classes/shared/).
use sly2_recomp::engine::classes::k13::{self, C2d3d20};
use sly2_recomp::engine::classes::plugin::{Classes, ClassesLevel, ClassesPlugin, EngineHost};
use sly2_recomp::engine::classes::shared::act::ActPool;
use sly2_recomp::engine::classes::{ClassHost, ClassSet, Inst};
use sly2_recomp::engine::player::p8_hazard::Hazards;
use sly2_recomp::engine::disc::descriptor::Descriptor;
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::player::plugin::PlayerFocus;
use sly2_recomp::engine::world::{LoadOptions, World};
use std::sync::Arc;

const DT: f32 = 1.0 / 60.0;

fn levels() -> (LevelLoader, Vec<Descriptor>) {
    crate::common::levels() // public: ISO level table instead of the analysis DB
}

/// Step 0: in the plugin-driven engine (ClassesPlugin + PlayerFocus) a class-107 glow that targets Sly (0x34)
/// switches on when Sly walks up to it, and off again when he leaves.
#[test]
fn k13_plugin_player_switches_c107() {
    let (loader, descs) = levels();
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        if !w.objects.iter().any(|o| o.class == 107) {
            continue;
        }
        let mut app = bevy_app::App::new();
        app.add_plugins(ClassesPlugin);
        app.init_resource::<PlayerFocus>();
        app.world_mut().resource_mut::<ClassesLevel>().0 = Some(w.clone());
        app.update();
        let pick = {
            let mut c = app.world_mut().resource_mut::<Classes>();
            let c = &mut *c;
            let ids: Vec<usize> = c.set.k12.iter().map(|x| x.0).collect();
            let f9c = c.set.g.g_3e0f9c;
            ids.into_iter()
                .find(|&i| {
                    let g = c.set.k12_get(i).unwrap();
                    g.f_154 == 0x34 && g.f_164 == 0 && (f9c == 0 || !(g.f_150 == 0 || g.f_150 == 2))
                })
                .map(|i| (i, c.host.ee_sub_1a62e8(i as u32 + 1)))
        };
        let Some((i, at)) = pick else { continue };
        let walk = |app: &mut bevy_app::App, x: f32, valid: bool| {
            {
                let mut f = app.world_mut().resource_mut::<PlayerFocus>();
                f.valid = valid;
                f.pos[0] = at[0] + x;
                f.pos[1] = at[1];
                f.pos[2] = at[2];
            }
            app.update();
            app.world_mut().resource_mut::<Classes>().set.k12_get(i).unwrap().f_164
        };
        assert_eq!(walk(&mut app, 2000.0, true), 0, "far away: off");
        let mut on_at = None;
        for k in 0..40 {
            let x = 2000.0 - 50.0 * k as f32;
            if walk(&mut app, x, true) == 1 {
                on_at = Some(x);
                break;
            }
        }
        let on_at = on_at.expect("Sly walking up switches the glow on");
        assert!(on_at < 300.0, "on radius +0x158 = 300 ({on_at})");
        assert_eq!(walk(&mut app, 450.0, true), 1, "hysteresis: still on inside 500");
        assert_eq!(walk(&mut app, 600.0, true), 0, "off past 500");
        assert_eq!(walk(&mut app, 0.0, false), 0, "no player (PlayerFocus invalid): FUN_001c4840 = 0, stays off");
        println!("lsn {} class-107 obj {i} on at {on_at}", d.lsn());
        return;
    }
    panic!("no class-107 glow targeting Sly");
}

#[test]
fn k13_c81_init() {
    let mut c = C2d3d20::default();
    c.ee_sub_1b2070(4, &mut EngineHost::default());
    let p = &c.p;
    assert_eq!((p.f_15c, p.f_160, p.f_154, p.f_1e0, p.f_1ec, p.f_1f0, p.f_150, p.f_40 & 2, p.f_200), (200.0, 200.0, f32::MAX, 1, 0x100, -10, 0, 2, 5));
    assert_eq!((p.f_264, p.f_26c, p.f_260, p.f_270, p.f_274), (0, 1.0, 800.0, 2000.0, 5000.0));
    assert_eq!(k13::slot5(81), Some(0x2bb13c));
}

/// every class-81 object built + fed on all 43 levels, 300 frames of the slot-69 physics part (at rest: stays put)
#[test]
#[ignore = "heavy"]
fn k13_c81_all_levels_300_frames() {
    let (loader, descs) = levels();
    let (mut n, mut orig, mut nlv) = (0, 0, 0);
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        s.post_load(&mut h);
        s.g.dt = DT;
        let want = w.objects.iter().filter(|o| o.class == 81).count();
        let ids: Vec<usize> = s.k13.iter().filter(|x| matches!(x.1, Inst::K13(_))).map(|x| x.0).collect();
        assert_eq!(ids.len(), want, "every class-81 object built");
        let p0: Vec<[f32; 3]> = ids.iter().map(|&i| s.k13_get(i).unwrap().p.phys.pos).collect();
        for _ in 0..300 {
            s.g.t += DT;
            s.frame(&mut h);
        }
        for (k, &i) in ids.iter().enumerate() {
            let c = s.k13_get(i).unwrap();
            assert!(c.p.f_270 > 0.0 && c.p.f_274 >= 0.0 && c.p.f_270.is_finite(), "fed speeds {:?}", (c.p.f_270, c.p.f_274));
            assert_eq!(c.p.phys.pos, p0[k], "at rest: no gravity, no drift");
        }
        n += want;
        orig += w.objects.iter().filter(|o| o.class == 81 && o.clone_of.is_none()).count();
        nlv += (want > 0) as usize;
    }
    println!("class 81: {n} instances ({orig} non-clone) on {nlv} levels");
    assert_eq!(orig, 132, "class_port.md instance count (clones excluded)");
}

/// A real class-81 instance: launch (slot 77) records the launcher / vector, plays +0x230 and adds the kind-4
/// xfAng act record; launched at 100 along +x the slot-69 hold accelerates it to its cruise speed +0x270 at
/// +0x274 per second, along the launch direction; +0x1e8 != 0 disables the hold.
#[test]
fn k13_c81_real_instance_launch() {
    let (loader, descs) = levels();
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        let Some(i) = s.k13.iter().map(|x| x.0).find(|&i| w.objects[i].clone_of.is_none()) else { continue };
        let mut c = s.k13_get(i).unwrap().clone();
        let mut pool = ActPool::default();
        c.p.f_264 = 1;
        c.ee_sub_1b1e88(i, 7, [1.0, 2.0, 3.0, 0.0], &mut pool, &mut h);
        assert_eq!((c.p.f_210, c.p.f_220), (7, [1.0, 2.0, 3.0, 0.0]));
        assert_eq!(c.p.xf_ang.list.len(), 1, "kind-4 act record on xfAng");
        assert_eq!(c.p.f_268, c.p.xf_ang.list[0].id);
        assert_eq!(c.p.xf_ang.list[0].kind(), 4);
        let (cruise, rate) = (c.p.f_270, c.p.f_274);
        c.p.f_1e8 = 0;
        c.p.phys.v = [100.0, 0.0, 0.0];
        let x0 = c.p.phys.pos;
        let mut last = 100.0f32;
        for _ in 0..600 {
            c.physics(i, DT, &mut h);
            let v = c.p.phys.v;
            let sp = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
            assert!(v[1].abs() < 1e-3 && v[2].abs() < 1e-3, "keeps the direction");
            assert!((sp - last).abs() <= rate * DT * 1.01 + 1e-2, "rate-limited {last} -> {sp}");
            last = sp;
        }
        assert!((last - cruise).abs() < 1.0, "cruise speed {cruise}, got {last}");
        assert!(c.p.phys.pos[0] > x0[0]);
        c.p.f_1e8 = 1;
        c.p.phys.v = [100.0, 0.0, 0.0];
        c.physics(i, DT, &mut h);
        assert_eq!(c.p.phys.v, [100.0, 0.0, 0.0], "+0x1e8 holds off the speed control");
        println!("lsn {} obj {i}: cruise {cruise} rate {rate}", d.lsn());
        return;
    }
    panic!("no class-81 instance");
}

/// every class-89 water built + fed on all 43 levels (defaults / props 876, slot 63 physics block, slot 72
/// volume = the P8 water), 300 frames
#[test]
#[ignore = "heavy"]
fn k13_c89_all_levels_300_frames() {
    let (loader, descs) = levels();
    let (mut n, mut orig, mut nlv, mut vols) = (0, 0, 0, 0);
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        s.post_load(&mut h);
        s.g.dt = DT;
        let hz = Hazards::load(&w);
        let ids: Vec<usize> = s.k13.iter().filter(|x| matches!(x.1, Inst::K13Water(_))).map(|x| x.0).collect();
        let want = w.objects.iter().filter(|o| o.class == 89).count();
        assert_eq!(ids.len(), want, "every class-89 object built");
        for _ in 0..300 {
            s.g.t += DT;
            s.frame(&mut h);
        }
        for &i in &ids {
            let c = s.k13_water(i).unwrap();
            assert_eq!((c.f_180, c.f_188, c.f_40 & 2, c.parts_bit48, c.e8_d4, c.e8_d8), (1, 0x2000, 2, true, 1.0, 1.0));
            assert!(c.contacts.is_empty());
            if let Some(p8) = hz.waters.iter().find(|x| x.obj == i) {
                assert_eq!(c.f_184, p8.amount, "prop 876 as P8");
                vols += 1;
                assert!(c.vol.as_ref().unwrap().parts.iter().all(|p| p.flags & (1 << 48) != 0));
            }
            assert_eq!(k13::slot5(89), Some(0x2bb1dc));
        }
        n += want;
        orig += w.objects.iter().filter(|o| o.class == 89 && o.clone_of.is_none()).count();
        nlv += (want > 0) as usize;
    }
    println!("class 89: {n} instances ({orig} non-clone, {vols} with a P8 volume) on {nlv} levels");
    assert_eq!(orig, 58, "class_port.md instance count (clones excluded)");
}

#[derive(Default)]
struct H89 {
    unlinked: Vec<(u32, u32)>,
    msgs: Vec<(usize, i32, u32)>,
    v244: Vec<(u32, u32)>,
}
impl ClassHost for H89 {
    fn ee_sub_229738(&mut self, obj: u32, water: u32) {
        self.unlinked.push((obj, water));
    }
    fn vcall_24(&mut self, obj: usize, a: i32, b: u32) {
        self.msgs.push((obj, a, b));
    }
    fn vcall_244(&mut self, obj: u32, water: u32) {
        self.v244.push((obj, water));
    }
}

/// A real class-89 water: slot 72 holds points inside its parts (same answer as the P8 water test over a
/// grid around it); slot 4 msg 1 from a contact drops its record and sends it msg 5 {water}; unknown senders /
/// other messages do nothing; slot 70 calls the contact's +0x244; slot 11 drops every record.
#[test]
fn k13_c89_real_instance() {
    let (loader, descs) = levels();
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let mut eh = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut eh);
        let ids: Vec<usize> = s.k13.iter().filter(|x| matches!(x.1, Inst::K13Water(_))).map(|x| x.0).collect();
        let Some(i) = ids.into_iter().find(|&i| s.k13_water(i).unwrap().vol.as_ref().is_some_and(|v| !v.parts.is_empty())) else { continue };
        let mut c = s.k13_water(i).unwrap().clone();
        let v = c.vol.clone().unwrap();
        let mut inside = 0;
        for p in &v.parts {
            for dx in [-200.0, -50.0, 0.0, 50.0, 200.0] {
                for dz in [-100.0, -20.0, 0.0, 20.0, 100.0] {
                    let q = [p.centre[0] + dx, p.centre[1] + dx * 0.5, p.centre[2] + dz];
                    let r = c.ee_sub_229d78(q);
                    assert_eq!(r, v.FUN_001f72f8(q));
                    inside += r as usize;
                }
            }
        }
        if inside == 0 {
            continue;
        }
        let me = i as u32 + 1;
        let mut h = H89::default();
        c.contacts = vec![10, 20, 30];
        c.ee_sub_229370(i, 1, 20, &mut h);
        assert_eq!(c.contacts, vec![10, 30]);
        assert_eq!(h.unlinked, vec![(20, me)]);
        assert_eq!(h.msgs, vec![(19, 5, me)], "msg 5 {{water}} to the leaving object");
        c.ee_sub_229370(i, 1, 99, &mut h);
        c.ee_sub_229370(i, 2, 10, &mut h);
        c.ee_sub_229370(i, 1, me, &mut h);
        assert_eq!((c.contacts.len(), h.msgs.len()), (2, 1), "no record / other message / self: nothing");
        c.ee_sub_2292d8(i, 30, &mut h);
        assert_eq!(h.v244, vec![(30, me)]);
        c.ee_sub_229308(i, &mut h);
        assert!(c.contacts.is_empty());
        assert_eq!(h.unlinked, vec![(20, me), (10, me), (30, me)]);
        println!("lsn {} water {i}: {} parts, {inside} grid points inside, amount {}", d.lsn(), v.parts.len(), c.f_184);
        return;
    }
    panic!("no class-89 water with a volume");
}
