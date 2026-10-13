//! WP-K12: guard damage message (K32: brain slot 12 FUN_00176be8 hit reactions) and the K12 classes
//! (107 C2dcc98 player-proximity glow).
use sly2_recomp::engine::classes::guard::c56::Guard;
use sly2_recomp::engine::classes::guard::slots::GuardHit;
use sly2_recomp::engine::classes::guard::PLAYER_HANDLE;
use sly2_recomp::engine::classes::plugin::EngineHost;
use sly2_recomp::engine::classes::k12::C2dcc98;
use sly2_recomp::engine::classes::{ClassGlobals, ClassHost, ClassSet};
use sly2_recomp::engine::player::p4::FUN_00132b10;
use sly2_recomp::engine::disc::descriptor::Descriptor;
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::physics::query::CollisionWorld;
use sly2_recomp::engine::player::p11_cane as cane;
use sly2_recomp::engine::player::p6::P6Level;
use sly2_recomp::engine::player::{Jt, JtInput};
use sly2_recomp::engine::world::{LoadOptions, World};
use std::sync::Arc;

const DT: f32 = 1.0 / 60.0;

fn step(jt: &mut Jt, held: u32, pressed: u32) {
    jt.pad_pending = Some((held, pressed));
    jt.update(&CollisionWorld::default(), JtInput::default(), DT, &mut ());
}

#[test]
fn k12_stun_pure() {
    let mut g = Guard::default();
    let mut h = EngineHost::default();
    g.init(0, &mut h);
    g.f_338 = 40;
    g.ee_sub_206518(0, &[], 0.0, &mut h);
    assert!(g.ee_sub_207468(0, &GuardHit { kind: 2, amount: -10, f_44: 0x20, ..Default::default() }, false, &mut h));
    assert!(!g.staggered());
    g.ee_sub_207350(0, 1.0, DT, &[], &|_| None, &|_| false, false, &[], &mut h);
    // K32: queued message -> FUN_00176be8 -> stagger (no anims: the goal ends at once, recovered next tick)
    assert!(g.f_damage.is_empty() && g.f_33c == 30 && g.hit_fx == 1);
}

/// A cane hit on a real-level guard staggers it (K32): it stops moving for the stagger anim, then resumes.
#[test]
fn k12_cane_hit_stuns_guard_on_real_level() {
    let (loader, descs) = levels();
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let p6 = P6Level::load(&w);
        if p6.cane.geom.is_none() {
            continue;
        }
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        s.g.k6_players = vec![(PLAYER_HANDLE, [1.0e9; 3])];
        s.post_load(&mut h);
        s.g.dt = DT;
        let Some(gi) = s.k6.first().map(|x| x.0) else { continue };
        let gpos = s.k6_guard(gi).unwrap().body.pos;
        let mut jt = Jt::new([gpos[0] - 120.0, gpos[1], gpos[2] + 30.0]);
        p6.apply(&mut jt);
        jt.t = 10.0;
        jt.t_ground = 1.0e9;
        step(&mut jt, 0x80, 0x80);
        let mut hit = false;
        for _ in 0..60 {
            hit |= cane::cane_hits_to_guards(&mut jt, &mut s, &mut h, &mut ()).iter().any(|x| x.target == gi as u32 + 1);
            if jt.cane.jtcs == -1 {
                break;
            }
            step(&mut jt, 0, 0);
            jt.phys.pos = [gpos[0] - 120.0, gpos[1], gpos[2] + 30.0];
        }
        assert!(hit, "the swing hit the guard");
        s.g.t += DT;
        s.frame(&mut h);
        let g = s.k6_guard(gi).unwrap();
        assert!(g.staggered(), "cane hit -> stagger (K32)");
        let p0 = g.body.pos;
        let mut n = 0;
        while s.k6_guard(gi).unwrap().staggered() {
            assert_eq!(s.k6_guard(gi).unwrap().body.pos, p0, "frozen while staggered");
            s.g.t += DT;
            s.frame(&mut h);
            n += 1;
            assert!(n < 600, "stagger wears off");
        }
        println!("lsn {} guard {gi} staggered for {n} frames", d.lsn());
        return;
    }
    panic!("no level with Sly and guards");
}
fn levels() -> (LevelLoader, Vec<Descriptor>) {
    crate::common::levels() // public: ISO level table instead of the analysis DB
}

/// class 107 test host: Sly (class 53) at `player`, the glow at `me`; records vtable +0x1c and +0x40 clears
#[derive(Default)]
struct H107 {
    player: [f32; 3],
    me: [f32; 3],
    free: bool,
    states: Vec<i32>,
    cleared: Vec<u32>,
}
impl ClassHost for H107 {
    fn ee_sub_1c4840(&mut self) -> u32 {
        PLAYER_HANDLE
    }
    fn ee_sub_132b10(&mut self, obj: u32, class: i32) -> u32 {
        (obj == PLAYER_HANDLE && FUN_00132b10(53, class)) as u32
    }
    fn ee_sub_1a62e8(&mut self, obj: u32) -> [f32; 3] {
        if obj == PLAYER_HANDLE { self.player } else { self.me }
    }
    fn jt_free_2dc_26c(&mut self, _p: u32) -> bool {
        self.free
    }
    fn vcall_1c(&mut self, _obj: usize, a: i32, _b: i32) {
        self.states.push(a);
    }
    fn obj_and_40(&mut self, obj: u32, mask: u32) {
        assert_eq!(mask, !8);
        self.cleared.push(obj);
    }
}

#[test]
#[ignore = "heavy"]
fn k12_c107_all_levels_300_frames() {
    let (loader, descs) = levels();
    let (mut n, mut nlv, mut orig) = (0, 0, 0);
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        s.post_load(&mut h);
        s.g.dt = DT;
        let want = w.objects.iter().filter(|o| o.class == 107).count();
        assert_eq!(s.k12.len(), want, "every class-107 object built");
        for _ in 0..300 {
            s.g.t += DT;
            s.frame(&mut h);
        }
        for (i, _) in s.k12.clone() {
            let c = s.k12_get(i).unwrap();
            assert_eq!((c.f_158, c.f_15c, c.f_178, c.f_184, c.f_40 & 10), (300.0, 500.0, -1, -1, 10));
            assert!(c.f_154 == 0x32 || c.f_154 == 0x34);
            assert!(c.f_16c.iter().all(|&o| o as usize <= w.objects.len()));
            assert_eq!(c.f_164, 0, "EngineHost has no FUN_001c4840 player: stays off");
        }
        n += want;
        orig += w.objects.iter().filter(|o| o.class == 107 && o.clone_of.is_none()).count();
        nlv += (want > 0) as usize;
    }
    println!("class 107: {n} instances ({orig} non-clone) on {nlv} levels");
    assert_eq!(orig, 55, "class_port.md instance count (clones excluded)");
}

/// A real class-107 instance: load-post clears the glow objects' +0x40 bit 3; Sly walking in / out switches
/// +0x164 with the 300 / 500 hysteresis (vtable +0x1c 5 / 6); a busy JT keeps the state; mode 0 / 2 is off
/// while DAT_003e0f9c.
#[test]
fn k12_c107_real_instance_proximity() {
    let (loader, descs) = levels();
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let mut eh = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut eh);
        let ids: Vec<usize> = s.k12.iter().map(|x| x.0).filter(|&i| w.objects[i].clone_of.is_none()).collect();
        let Some(i) = ids.into_iter().find(|&i| s.k12_get(i).unwrap().f_154 == 0x34) else { continue };
        let mut c: C2dcc98 = s.k12_get(i).unwrap().clone();
        assert_eq!(c.f_154, 0x34, "field 1313: Sly's class 0x34");
        assert!(c.f_16c.iter().any(|&o| o != 0), "fields 1318-1320 resolved");
        let mut h = H107 { free: true, ..Default::default() };
        c.ee_sub_229f88(i, &mut h);
        assert_eq!(h.cleared, c.f_16c.iter().copied().filter(|&o| o != 0).collect::<Vec<_>>());
        let mut g = ClassGlobals::default();
        let at = |c: &mut C2dcc98, h: &mut H107, g: &ClassGlobals, x: f32| {
            h.player = [x, 0.0, 0.0];
            c.ee_sub_22a060(i, g, h);
            c.f_164
        };
        assert_eq!(at(&mut c, &mut h, &g, 400.0), 0);
        assert_eq!(at(&mut c, &mut h, &g, 299.0), 1);
        assert_eq!(at(&mut c, &mut h, &g, 450.0), 1, "500 off radius once on");
        assert_eq!(at(&mut c, &mut h, &g, 501.0), 0);
        assert_eq!(h.states, vec![5, 6]);
        h.free = false;
        assert_eq!(at(&mut c, &mut h, &g, 10.0), 0, "busy JT keeps the state");
        h.free = true;
        g.g_3e0f9c = 1;
        let off = c.f_150 == 0 || c.f_150 == 2;
        assert_eq!(at(&mut c, &mut h, &g, 10.0), (!off) as i32, "mode {} with DAT_003e0f9c", c.f_150);
        println!("lsn {} obj {i}: {c:?}", d.lsn());
        return;
    }
    panic!("no class-107 instance");
}

#[test]
fn k12_c107_init() {
    let mut c = C2dcc98 { bit35: true, ..Default::default() };
    c.ee_sub_229ef0(0, &mut EngineHost::default());
    assert_eq!((c.f_154, c.f_158, c.f_15c, c.f_178, c.f_184, c.f_40, c.bit35), (0x32, 300.0, 500.0, -1, -1, 10, false));
}
