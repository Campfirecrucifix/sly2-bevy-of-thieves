//! WP-K19: Murray (class 54) own state machine in classes/k19_54.rs (SetState FUN_001b6078 + sub-states,
//! slots 4 / 14 / 15 / 98 / 99, FUN_001b7ba8, FUN_001b6cc0).
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
use std::collections::BTreeSet;
use std::sync::Arc;

const DT: f32 = 1.0 / 60.0;
const O: usize = 9;

fn levels() -> (LevelLoader, Vec<Descriptor>) {
    crate::common::levels() // public: ISO level table instead of the analysis DB
}

/// test host: object in the world, anims finish when `done`, every po hook returns 0
#[derive(Default)]
struct H {
    done: bool,
    calls: Vec<u32>,
    vel: Vec<[f32; 3]>,
}
impl ClassHost for H {
    fn lo_28_bit60(&mut self, _obj: u32) -> bool {
        true
    }
    fn ee_sub_131968(&mut self, _a: u32) -> bool {
        self.done
    }
    fn po_call(&mut self, addr: u32, _obj: usize, _off: u32, _args: &[u32]) -> i64 {
        self.calls.push(addr);
        0
    }
    fn vcall_10c(&mut self, _obj: usize) -> [f32; 3] {
        [3.0, 4.0, 5.0]
    }
    fn vcall_11c_v(&mut self, _obj: u32, v: [f32; 3]) {
        self.vel.push(v);
    }
    fn ee_sub_12f9b8(&mut self, _a: u32, _rate: f32, _anim: u32, _obj: u32, _flags: u32) -> u32 {
        0x99
    }
}

fn inited() -> (C2d44c8, ClassGlobals, GameRng) {
    let mut m = C2d44c8::default();
    let g = ClassGlobals { g_2c3b90: 400f32.to_bits(), t: 1.0, ..Default::default() };
    m.ee_sub_1b4728(O, None, &g, &mut NullHost);
    (m, g, GameRng::default())
}

fn ss(m: &mut C2d44c8, s: i32, g: &ClassGlobals, rng: &mut GameRng, h: &mut H) {
    let mut cx = Cx { g, rng, h };
    m.ee_sub_1b6078(O, s, &mut cx);
}

fn update(m: &mut C2d44c8, g: &mut ClassGlobals, rng: &mut GameRng, h: &mut H) {
    g.dt = DT;
    g.t += DT;
    let mut cx = Cx { g, rng, h };
    m.ee_sub_1b50b0(O, &mut cx);
}

/// SetState: +0x13d0 / +0x13d4, sub-state setters entered with 0 and left with -1
#[test]
fn k19_c54_setstate_substates() {
    let (mut m, mut g, mut rng) = inited();
    let mut h = H::default();
    ss(&mut m, 0, &g, &mut rng, &mut h);
    assert_eq!((m.b.state, m.b.m.f(0x13d4)), (0, 1.0));
    // 3: jump sub-state 0, 1 (anim event 0x1a4): velocity = horizontal + DAT_002c09a8 up
    g.t = 2.0;
    ss(&mut m, 3, &g, &mut rng, &mut h);
    assert_eq!((m.b.state, m.b.m.i(0x13d8), m.b.m.f(0x13d4)), (3, 0, 2.0));
    {
        let mut cx = Cx { g: &g, rng: &mut rng, h: &mut h };
        m.ee_sub_1b68b8(O, 9, &[0, 0, 0x1a4], &mut cx);
    }
    assert_eq!((m.b.m.i(0x13d8), h.vel.last().copied()), (1, Some([3.0, 4.0, -500.0])));
    ss(&mut m, 5, &g, &mut rng, &mut h);
    assert_eq!((m.b.m.i(0x13d8), m.b.m.i(0x1458), m.b.m.f(0x145c)), (-1, 0, 2.0));
    ss(&mut m, 9, &g, &mut rng, &mut h);
    assert_eq!((m.b.m.i(0x1458), m.b.m.i(0x17f4)), (-1, 0));
    // 4 with nothing held: carry sub-state 0
    ss(&mut m, 4, &g, &mut rng, &mut h);
    assert_eq!((m.b.state, m.b.m.i(0x17f4), m.b.m.i(0x1450)), (4, -1, 0));
    // carry sub-state 1 without a held object: back to state 0
    {
        let mut cx = Cx { g: &g, rng: &mut rng, h: &mut h };
        m.ee_sub_1b93e8(O, 1, &mut cx);
    }
    assert_eq!((m.b.state, m.b.m.i(0x1450)), (0, -1));
    // 0xb: ally lift sub-state from FUN_001b8508; 0xc push sub-states via slot 99 kind 0x14
    {
        let mut cx = Cx { g: &g, rng: &mut rng, h: &mut h };
        m.ee_sub_1b8508(O, &mut cx);
    }
    assert_eq!((m.b.state, m.b.m.i(0x1810)), (0xb, 1));
    {
        let mut cx = Cx { g: &g, rng: &mut rng, h: &mut h };
        m.ee_sub_1b9a08(O, 0x14, &mut cx);
    }
    assert_eq!((m.b.state, m.b.m.i(0x1810), m.b.m.i(0x1840)), (0xc, -1, 1));
    ss(&mut m, 0x15, &g, &mut rng, &mut h);
    assert_eq!((m.b.state, m.b.m.i(0x1840), m.b.m.i(0x1908)), (0x15, -1, 0));
    // slot 87 index 3 -> SetState 0xe
    {
        let mut cx = Cx { g: &g, rng: &mut rng, h: &mut h };
        m.ee_sub_1b7e40_cx(O, 3, &mut cx);
    }
    assert_eq!(m.b.state, 0xe);
}

/// slot 15: the joystick and anim ends drive the state machine (FUN_001b4aa0)
#[test]
fn k19_c54_update_reaches_states() {
    let (mut m, mut g, mut rng) = inited();
    let mut h = H::default();
    {
        let mut cx = Cx { g: &g, rng: &mut rng, h: &mut h };
        m.ee_sub_1b49e0(O, &mut cx);
    }
    assert_eq!(m.b.state, 0, "load-post enters state 0");
    let mut seen = BTreeSet::new();
    m.b.joy = Joy { angle: 0.5, mag: 1.0, held: 8, ..Default::default() };
    update(&mut m, &mut g, &mut rng, &mut h);
    seen.insert(m.b.state);
    assert_eq!(m.b.state, 1);
    m.b.joy.held = 0;
    update(&mut m, &mut g, &mut rng, &mut h);
    assert_eq!(m.b.state, 10);
    m.b.joy.mag = 0.0;
    update(&mut m, &mut g, &mut rng, &mut h);
    assert_eq!(m.b.state, 0);
    // slot 99 kind 0x16: jump (state 3, anim +0x13e0 playing), anim over -> 0
    m.b.m.set_u(0x13e0, 0x44);
    {
        let mut cx = Cx { g: &g, rng: &mut rng, h: &mut h };
        m.ee_sub_1b9a08(O, 0x16, &mut cx);
    }
    assert_eq!((m.b.state, m.b.m.i(0x13e8)), (3, 1));
    update(&mut m, &mut g, &mut rng, &mut h);
    assert_eq!(m.b.state, 3);
    // air steering from the stick in sub-state 0
    m.b.joy.mag = 1.0;
    update(&mut m, &mut g, &mut rng, &mut h);
    assert!(m.b.m.v3(0x190)[0].abs() + m.b.m.v3(0x190)[1].abs() > 1.0);
    m.b.joy.mag = 0.0;
    h.done = true;
    update(&mut m, &mut g, &mut rng, &mut h);
    assert_eq!(m.b.state, 0);
    h.done = false;
    // kind 0x13: state 8, anim over -> 0
    {
        let mut cx = Cx { g: &g, rng: &mut rng, h: &mut h };
        m.ee_sub_1b9a08(O, 0x13, &mut cx);
    }
    assert_eq!((m.b.state, m.b.m.i(0x143c)), (8, 1));
    h.done = true;
    update(&mut m, &mut g, &mut rng, &mut h);
    assert_eq!(m.b.state, 0);
    h.done = false;
    // kind 0x10: state 0x15, button 0x10 gadget not held -> 0
    {
        let mut cx = Cx { g: &g, rng: &mut rng, h: &mut h };
        m.ee_sub_1b9a08(O, 0x10, &mut cx);
    }
    assert_eq!(m.b.state, 0x15);
    update(&mut m, &mut g, &mut rng, &mut h);
    assert_eq!(m.b.state, 0);
    // the 0x80 press with no lift anim playing: state 0xb lift
    let mut j = Joy::default();
    j.press_t[7] = g.t + DT;
    m.b.joy = j;
    update(&mut m, &mut g, &mut rng, &mut h);
    // K20: FUN_001b8988 is ported; with no lift anim slot (+0x1030 == 0 in this host) it returns to 0
    assert_eq!((m.b.state, m.b.m.i(0x1810)), (0, -1));
    assert!(!h.calls.contains(&0x1b8988));
}

/// slot 4 messages, slot 98, FUN_001b7ba8, FUN_001b6cc0
#[test]
fn k19_c54_messages_gadgets() {
    let (mut m, g, mut rng) = inited();
    let mut h = H::default();
    {
        let mut cx = Cx { g: &g, rng: &mut rng, h: &mut h };
        m.ee_sub_1b6078(O, 0, &mut cx);
        for (ev, off) in [(0x240, 0x1818), (0x241, 0x181c), (0x242, 0x1820), (0x1c6, 0x1448)] {
            m.ee_sub_1b68b8(O, 9, &[0, 0, ev], &mut cx);
            assert_eq!(m.b.m.i(off), 1, "event {ev:#x}");
        }
        // state 6 state-machine label 0x1cc: throw speeds by +0x1534; 0x1ca ends the state
        m.ee_sub_1b6078(O, 6, &mut cx);
        m.ee_sub_1b68b8(O, 10, &[0, 0, 0x1cc], &mut cx);
        assert_eq!((m.b.m.f(0x1524), m.b.m.i(0x152c)), (1.5, 6));
        m.ee_sub_1b68b8(O, 10, &[0, 0, 0x1ca], &mut cx);
        assert_eq!(m.b.state, 0);
    }
    for (kind, ok) in [(1, true), (0x15, true), (3, false), (7, false)] {
        assert_eq!(m.ee_sub_1b98a0(O, kind, &mut h), ok, "kind {kind:#x}");
    }
    m.b.state = 5;
    assert!(m.ee_sub_1b98a0(O, 3, &mut h));
    assert!(!m.ee_sub_1b98a0(O, 1, &mut h));
    // FUN_001b7ba8 in carry sub-state 0: message 6 to the held object, dropped
    m.b.state = 4;
    m.b.m.set_i(0x1450, 0);
    m.b.m.set_u(0x13f0, 0x55);
    m.ee_sub_1b7ba8(O, &mut h);
    assert_eq!(m.b.m.u(0x13f0), 0);
    // FUN_001b6cc0 runs only in jump sub-states 0 / 1
    m.b.m.set_i(0x13d8, 2);
    m.ee_sub_1b6cc0(O, &mut h);
}

/// every level: Murray instances load + init + load-post (state 0); each one in turn is the controlled
/// character and runs 300 frames with a moving stick through the real host
#[test]
#[ignore = "heavy"]
fn k19_all_levels_300_frames() {
    let (loader, descs) = levels();
    let mut nm = 0;
    let mut seen = BTreeSet::new();
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let ms: Vec<usize> = w.objects.iter().enumerate().filter(|x| x.1.class == 54).map(|x| x.0).collect();
        if ms.is_empty() {
            continue;
        }
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        assert_eq!(s.k18.len(), ms.len());
        s.post_load(&mut h);
        for &o in &ms {
            assert_eq!(s.k18_murray(o).unwrap().b.state, 0, "load-post enters state 0");
        }
        for &o in &ms {
            s.k18_set_active(Some(o), &mut h);
            for f in 0..300 {
                s.g.dt = DT;
                s.g.t += DT;
                let mag = if (f / 60) % 2 == 0 { 1.0 } else { 0.0 };
                let held = if (f / 120) % 2 == 0 { 8 } else { 0 };
                s.k18_feed_joy(Joy { angle: f as f32 * 0.01, mag, held, ..Default::default() });
                s.frame(&mut h);
                seen.insert(s.k18_murray(o).unwrap().b.state);
            }
        }
        s.k18_set_active(None, &mut h);
        nm += ms.len();
    }
    assert!(nm > 0);
    eprintln!("murray {nm} states {seen:?}");
    assert!(seen.contains(&0));
}
