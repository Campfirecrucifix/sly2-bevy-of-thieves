//! WP-K18: Bentley (class 55) rest of 55h in classes/k18_55.rs, Murray (class 54) in classes/k18_54.rs.
use sly2_recomp::engine::classes::k16_55::C2cb230;
use sly2_recomp::engine::classes::k18_54::C2d44c8;
use sly2_recomp::engine::classes::k18_55::Cx;
use sly2_recomp::engine::classes::plugin::EngineHost;
use sly2_recomp::engine::classes::{ClassGlobals, ClassHost, ClassSet, NullHost};
use sly2_recomp::engine::disc::descriptor::Descriptor;
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::physics::driven::GameRng;
use sly2_recomp::engine::player::Joy;
use sly2_recomp::engine::world::{LoadOptions, World};
use std::sync::Arc;

const DT: f32 = 1.0 / 60.0;

fn levels() -> (LevelLoader, Vec<Descriptor>) {
    crate::common::levels() // public: ISO level table instead of the analysis DB
}

/// a Bentley object after slot 6 (object index 7)
fn inited() -> (C2cb230, ClassGlobals) {
    let mut c = C2cb230::default();
    let mut g = ClassGlobals { g_2c3b90: 400f32.to_bits(), ..Default::default() };
    c.ee_sub_134648(7, None, &mut g, &mut NullHost);
    (c, g)
}

/// test host: object in the world, anims never finish, gadgets allowed (vtable +0x31c), a carry object 0x77
#[derive(Default)]
struct H {
    calls: Vec<(u32, u32)>,
    focus: Vec<u32>,
}
impl ClassHost for H {
    fn lo_28_bit60(&mut self, _obj: u32) -> bool {
        true
    }
    fn ee_sub_131968(&mut self, _a: u32) -> bool {
        false
    }
    fn vcall_ret_arg(&mut self, _obj: u32, _off: u32, _arg: i32) -> i64 {
        1
    }
    fn po_call(&mut self, addr: u32, _obj: usize, off: u32, _args: &[u32]) -> i64 {
        self.calls.push((addr, off));
        0
    }
    fn ee_sub_1cb250(&mut self, _host: u32, _k: i32) -> u32 {
        0x77
    }
    fn ee_sub_1c4940(&mut self, obj: u32) {
        self.focus.push(obj);
    }
}

fn joy(mag: f32, held: u32) -> Joy {
    Joy { angle: 0.5, mag, held, ..Default::default() }
}

fn update(c: &mut C2cb230, g: &mut ClassGlobals, rng: &mut GameRng, h: &mut H) {
    g.dt = DT;
    g.t += DT;
    let mut cx = Cx { g, rng, h };
    c.ee_sub_134da8(7, &mut cx);
}

/// FUN_00136a28 / FUN_00141ea8 / FUN_00141cd0
#[test]
fn k18_c55_gadget_helpers() {
    assert_eq!([4, 5, 7, 9, 0xb, 0xd, 3].map(C2cb230::ee_sub_136a28), [0x1860, 0x1824, 0x1830, 0x183c, 0x1848, 0x1854, 0]);
    let (mut c, mut g) = inited();
    c.m.set_i(0x1180, 0xc);
    c.m.set_i(0x118c, 4);
    c.m.set_i(0x1198, 0xc);
    assert_eq!((c.ee_sub_141ea8(0xc), c.ee_sub_141ea8(4), c.ee_sub_141ea8(9)), (4 | 2, 1, 0));
    c.ctr_12fc = Some(30);
    g.t = 5.0;
    c.ee_sub_141cd0(0xa, &g);
    assert_eq!((c.ee_sub_1441a0(), c.m.f(0x1304)), (15, 5.0));
}

/// slot 15: the joystick drives the state transitions (FUN_00134850): 0 idle, 1 run (held 8), 7 sneak, 5
/// (FUN_00140af8 says so: hook 0 here, so never), 6 carry on the 0x10 press, 0xe (slot 99 kind 10) and back to 0
#[test]
fn k18_c55_update_reaches_states() {
    let (mut c, mut g) = inited();
    let mut rng = GameRng::default();
    let mut h = H::default();
    g.t = 1.0;
    c.ee_sub_135248(7, 0, &g, &mut rng, &mut h);
    assert_eq!(c.state, 0);
    c.joy = joy(1.0, 8);
    update(&mut c, &mut g, &mut rng, &mut h);
    assert_eq!(c.state, 1);
    c.joy = joy(1.0, 0);
    update(&mut c, &mut g, &mut rng, &mut h);
    assert_eq!(c.state, 7);
    c.joy = joy(0.0, 0);
    update(&mut c, &mut g, &mut rng, &mut h);
    assert_eq!(c.state, 0);
    // 0x10 pressed now: carry the +0x1870 gadget (5 -> record +0x1824)
    let mut j = joy(0.0, 0);
    j.press_t[4] = g.t + DT;
    c.joy = j;
    update(&mut c, &mut g, &mut rng, &mut h);
    assert_eq!((c.state, c.m.u(0x16b4)), (6, 0x1824));
    // the press was consumed
    assert_eq!(c.joy.press_t[4], sly2_recomp::engine::player::PRESS_NONE);
    // slot 99 kind 10: pay, state 0xe; 2 s later back to 0
    c.ctr_12fc = Some(50);
    {
        let mut cx = Cx { g: &g, rng: &mut rng, h: &mut h };
        c.ee_sub_136a90(7, 10, &mut cx);
    }
    assert_eq!((c.state, c.ee_sub_1441a0()), (0xe, 35));
    for _ in 0..130 {
        update(&mut c, &mut g, &mut rng, &mut h);
    }
    assert_eq!(c.state, 0);
    // slot 99 kind 4: carry record +0x1860, pay, state 6
    {
        let mut cx = Cx { g: &g, rng: &mut rng, h: &mut h };
        c.ee_sub_136a90(7, 4, &mut cx);
    }
    assert_eq!((c.state, c.m.u(0x16b4)), (6, 0x1860));
    // slot 99 kind 7: spawned carry object 0x77 kept in +0x11c0, state 0xc, +0x11b0 told 4
    h.calls.clear();
    {
        let mut cx = Cx { g: &g, rng: &mut rng, h: &mut h };
        c.ee_sub_136a90(7, 7, &mut cx);
    }
    assert_eq!((c.state, c.m.u(0x11c0)), (0xc, 0x77));
    assert!(h.calls.contains(&(0x2206c8, 0x11b0)));
    // slot 98: no gadget while in the binocucom / with +0x1818
    c.m.set_i(0x1818, 1);
    assert!(!c.ee_sub_136870(7, 8, &mut h));
}

/// slot 14 load-post: aim block (+0x1710), state 0, +0x10b8 child
#[test]
fn k18_c55_load_post() {
    let (mut c, mut g) = inited();
    let mut rng = GameRng::default();
    let mut h = H::default();
    g.t = 2.0;
    let mut cx = Cx { g: &g, rng: &mut rng, h: &mut h };
    c.ee_sub_1347a8(7, &mut cx);
    assert_eq!((c.state, c.m.u(0x1710), c.m.i(0x1714)), (0, 8, -1));
    assert_eq!((c.m.f(0x1740), c.m.f(0x1744), c.m.f(0x1730)), (-50.0, 50.0, 175.0));
}

/// FUN_001c3ed8 / slot 78: control focus moves between characters
#[test]
fn k18_c55_control_switch() {
    let (mut c, _) = inited();
    assert_eq!(c.ee_sub_1c3ed8(1, false), Some(true));
    assert_eq!((c.joy_80, c.joy_78), (Some(1), 0xffff));
    assert_eq!(c.ee_sub_1c3ed8(1, true), None);
    assert_eq!(c.ee_sub_1c3ed8(0, true), Some(false));
    assert_eq!(c.ee_sub_1c3ed8(2, false), None);
}

/// Murray slot 6 FUN_001b4728 and the small slots
#[test]
fn k18_c54_init_small_slots() {
    let mut m = C2d44c8::default();
    let g = ClassGlobals { g_2c3b90: 400f32.to_bits(), ..Default::default() };
    assert_eq!(m.ee_sub_1b4728(9, None, &g, &mut NullHost), 10);
    let b = &m.b;
    assert_eq!((b.state, b.m.f(0x220), b.m.f(0x103c), b.f_1048, b.m.i(0x1000)), (-1, 50.0, 60.0, 125.0, 4));
    assert_eq!((b.m.f(0x1410), b.m.f(0x1414), b.m.f(0x10b4), b.m.f(0x10a8), b.m.f(0x1ac8)), (1500.0, 3500.0, 4500.0, 250.0, -60.0));
    assert_eq!((b.m.u(0x1320), b.m.u(0x13b0), b.m.u(0x1460), b.m.u(0x1630), b.m.u(0x1540), b.m.i(0x1850)), (10, 0x15, 10, 10, 10, -1));
    // +0x1910 FUN_0021f2e0
    assert_eq!((b.m.u(0x1910), b.m.i(0x1914), b.m.f(0x192c), b.m.f(0x19e4)), (10, -1, 500.0, -73.0));
    assert_eq!((m.ee_sub_113d70(), m.ee_sub_1b9b60(), m.ee_sub_1b80f8()), (0x2baf20, "murray", 0));
    for (s, hud, b93) in [(0, 0, true), (2, 1, false), (9, -1, true), (0x14, 8, false), (0x11, 5, false)] {
        m.b.state = s;
        assert_eq!((m.ee_sub_1b7f90(), m.ee_sub_1b8180()), (hud, b93), "state {s}");
    }
    m.b.state = 0xf;
    assert!(m.ee_sub_1b83b8(9, 1, &mut NullHost));
    // +0x13d8 -1: FUN_00143c78 (damage state -1 -> 7)
    assert_eq!(m.ee_sub_1b8368(), 7);
    m.b.state = 0xc;
    assert_eq!(m.ee_sub_1b8100(&mut NullHost), 1);
    // slot 87: index 3 -> FUN_001b7ba8 then SetState 0xe
    let mut h = H::default();
    m.ee_sub_1b7e40(9, 3, &mut h);
    assert_eq!(h.calls, vec![(0x1b7ba8, 0), (0x1b6078, 0)]);
    h.calls.clear();
    m.ee_sub_1b7e40(9, 9, &mut h);
    assert!(h.calls.is_empty());
}

/// every level: Bentley / Murray instances load + init + load-post; each one in turn is made the controlled
/// character and runs 300 frames with a moving stick
#[test]
#[ignore = "heavy"]
fn k18_all_levels_300_frames() {
    let (loader, descs) = levels();
    let (mut nb, mut nm) = (0, 0);
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let bs: Vec<usize> = w.objects.iter().enumerate().filter(|x| x.1.class == 55).map(|x| x.0).collect();
        let ms: Vec<usize> = w.objects.iter().enumerate().filter(|x| x.1.class == 54).map(|x| x.0).collect();
        if bs.is_empty() && ms.is_empty() {
            continue;
        }
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        assert_eq!((s.k16.len(), s.k18.len()), (bs.len(), ms.len()));
        for &i in &ms {
            assert_eq!(s.k18_murray(i).unwrap().b.m.f(0x1414), 3500.0);
        }
        s.post_load(&mut h);
        for &i in &bs {
            assert_eq!(s.k16_get(i).unwrap().state, 0, "load-post enters state 0");
        }
        for &o in bs.iter().chain(ms.iter()) {
            s.k18_set_active(Some(o), &mut h);
            assert_eq!(s.k18_active, Some(o));
            for f in 0..300 {
                s.g.dt = DT;
                s.g.t += DT;
                let mag = if (f / 60) % 2 == 0 { 1.0 } else { 0.0 };
                s.k18_feed_joy(Joy { angle: f as f32 * 0.01, mag, held: 8, ..Default::default() });
                s.frame(&mut h);
            }
        }
        s.k18_set_active(None, &mut h);
        assert_eq!(s.k18_active, None);
        nb += bs.len();
        nm += ms.len();
    }
    assert_eq!(nb, 42);
    assert!(nm > 0);
    eprintln!("bentley {nb} murray {nm}");
}
