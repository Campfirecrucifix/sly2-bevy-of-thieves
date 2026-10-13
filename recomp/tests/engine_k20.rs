//! WP-K20: Murray leftovers (classes/k20_54.rs: per-state updates 5 / 6 / 9 / 0xb / 0xc, FUN_001b70a0,
//! FUN_001b7d20 / 1b7970 / 1b7ae8, slot 86, base update FUN_001403d0) + the K20 class ports.
use sly2_recomp::engine::classes::k18_54::C2d44c8;
use sly2_recomp::engine::classes::k18_55::Cx;
use sly2_recomp::engine::classes::k20_54::FUN_001d69f0;
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
const HELD: u32 = 0x500;
const X: u32 = 0x600;
const ALLY: u32 = 0x700;

/// Public build: every level descriptor, ISO only (no analysis DB). The level table embedded in the boot level
/// (FUN_0020f318) lists every file; rows that pass `level_check::check_level_row` are the levels, sorted by LSN.
pub fn levels() -> (LevelLoader, Vec<Descriptor>) {
    let iso = Arc::new(IsoImage::open(concat!(env!("CARGO_MANIFEST_DIR"), "/../Sly 2 - Band of Thieves (USA).iso")).expect("ISO"));
    let elf = iso.read_file("SCUS_973.16").unwrap();
    let table = sly2_recomp::engine::disc::descriptor::static_table(&elf).unwrap();
    let boot = *table.iter().skip(22).find(|d| d.size() != 0).unwrap();
    let loader = LevelLoader::new(iso, ModOverlay::default());
    let lt = loader.load_blocking(boot).unwrap().header.level_table;
    let mut descs: Vec<Descriptor> = vec![];
    for e in lt.entries {
        let d = e.desc;
        if !descs.iter().any(|x| x.same_file(&d)) && sly2_recomp::engine::game::level_check::check_level_row(&loader, d).is_ok() {
            descs.push(d);
        }
    }
    descs.sort_by_key(|d| d.lsn());
    (loader, descs)
}

/// test host: HELD +0x180 = X (in the world while `in_world`), ALLY positions, sm labels
#[derive(Default)]
struct H {
    done: bool,
    in_world: bool,
    calls: Vec<u32>,
    ee: Vec<(u32, Vec<u32>)>,
    vel: Vec<[f32; 3]>,
    vc: Vec<(u32, u32, i32)>,
    label: i32,
    ally_pos: [f32; 3],
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
    fn ee_call(&mut self, addr: u32, args: &[u32]) -> i64 {
        self.ee.push((addr, args.to_vec()));
        match addr {
            0x1f07e8 | 0x1f07c8 => self.label as i64,
            0x209a68 => 1,
            _ => 0,
        }
    }
    fn obj_i32(&mut self, obj: u32, off: u32) -> i32 {
        match (obj, off) {
            (HELD, 0x180) => X as i32,
            (ALLY, 0xfec) => 0x77,
            _ => 0,
        }
    }
    fn obj_flags_28(&mut self, obj: u32) -> u64 {
        if obj == X && self.in_world {
            1 << 60
        } else {
            0
        }
    }
    fn ee_sub_1a62e8(&mut self, obj: u32) -> [f32; 3] {
        if obj == ALLY {
            self.ally_pos
        } else {
            [0.0; 3]
        }
    }
    fn ee_sub_1eeee0_rec(&mut self, _obj: u32, _k: i32) -> Option<(u32, i32, u32)> {
        Some((0x800, 0, O as u32 + 1))
    }
    fn ee_sub_1f5860(&mut self, _obj: u32) -> [f32; 3] {
        [0.0, 0.0, -2000.0]
    }
    fn vcall_11c_v(&mut self, _obj: u32, v: [f32; 3]) {
        self.vel.push(v);
    }
    fn vcall_arg(&mut self, obj: u32, off: u32, a: i32) {
        self.vc.push((obj, off, a));
    }
    fn ee_sub_132b10(&mut self, _obj: u32, _class: i32) -> u32 {
        0
    }
    fn ee_sub_12f9b8(&mut self, _a: u32, _rate: f32, _anim: u32, _obj: u32, _flags: u32) -> u32 {
        0x99
    }
}

fn inited() -> (C2d44c8, ClassGlobals, GameRng) {
    let mut m = C2d44c8::default();
    let g = ClassGlobals { g_2c3b90: 400f32.to_bits(), t: 1.0, dt: DT, ..Default::default() };
    m.ee_sub_1b4728(O, None, &g, &mut NullHost);
    (m, g, GameRng::default())
}

fn with<R>(m: &mut C2d44c8, g: &ClassGlobals, rng: &mut GameRng, h: &mut H, f: impl FnOnce(&mut C2d44c8, &mut Cx) -> R) -> R {
    let mut cx = Cx { g, rng, h };
    f(m, &mut cx)
}

fn press(m: &mut C2d44c8, t: f32, mask: u32, held: u32, mag: f32) {
    m.b.joy = Joy { mag, held, ..Default::default() };
    m.b.joy.feed(t, held, mask);
}

/// state 5: object gone -> 0; Triangle-family 0xa0 -> throw 7; 0x10 with +0x1438 -> 8; stick -> sub-states
#[test]
fn k20_c54_state5_holding() {
    let (mut m, g, mut rng) = inited();
    let mut h = H { in_world: true, ..Default::default() };
    m.b.m.set_u(0x13f0, HELD);
    with(&mut m, &g, &mut rng, &mut h, |m, cx| m.ee_sub_1b6078(O, 5, cx));
    assert_eq!((m.b.state, m.b.m.i(0x1458)), (5, 0));
    press(&mut m, g.t, 0, 0, 1.0);
    with(&mut m, &g, &mut rng, &mut h, |m, cx| m.ee_sub_1b5380(O, cx));
    assert_eq!(m.b.m.i(0x1458), 1, "stick -> walk (no 8 held)");
    press(&mut m, g.t, 0, 8, 1.0);
    with(&mut m, &g, &mut rng, &mut h, |m, cx| m.ee_sub_1b5380(O, cx));
    assert_eq!(m.b.m.i(0x1458), 1, "8 held but gadget 0x12 not owned -> walk");
    press(&mut m, g.t, 0, 0, 0.0);
    with(&mut m, &g, &mut rng, &mut h, |m, cx| m.ee_sub_1b5380(O, cx));
    assert_eq!(m.b.m.i(0x1458), 0);
    press(&mut m, g.t, 0x10, 0, 0.0);
    m.b.m.set_u(0x1438, 1);
    m.b.m.set_u(0x143c, 5);
    with(&mut m, &g, &mut rng, &mut h, |m, cx| m.ee_sub_1b5380(O, cx));
    assert_eq!((m.b.state, m.b.m.u(0x143c)), (8, 0));
    // back to 5, 0x20 (in 0xa0) -> 7
    m.b.m.set_u(0x13f0, HELD);
    with(&mut m, &g, &mut rng, &mut h, |m, cx| m.ee_sub_1b6078(O, 5, cx));
    press(&mut m, g.t, 0x20, 0, 0.0);
    with(&mut m, &g, &mut rng, &mut h, |m, cx| m.ee_sub_1b5380(O, cx));
    assert_eq!(m.b.state, 7);
    // the object left the world: FUN_001b7d20 (12 acts) + SetState 0
    m.b.m.set_u(0x13f0, HELD);
    with(&mut m, &g, &mut rng, &mut h, |m, cx| m.ee_sub_1b6078(O, 5, cx));
    h.in_world = false;
    h.calls.clear();
    with(&mut m, &g, &mut rng, &mut h, |m, cx| m.ee_sub_1b5380(O, cx));
    assert_eq!((m.b.state, m.b.m.u(0x13f0)), (0, 0));
    assert!(h.calls.iter().filter(|&&a| a == 0x12e460).count() >= 12);
    assert_eq!(m.b.m.u(0x1430), X, "+0x1430 = released object");
}

/// state 6: label countdown and the DAT_002c0908 / 2c0948 tables
#[test]
fn k20_c54_state6_labels() {
    let (mut m, g, mut rng) = inited();
    let mut h = H { label: 0x1cb, ..Default::default() };
    m.b.m.set_u(0x1034, 0x44);
    m.b.m.set_f(0x1524, 0.01);
    m.b.m.set_i(0x152c, 2);
    m.b.m.set_i(0x1534, 1);
    with(&mut m, &g, &mut rng, &mut h, |m, cx| m.ee_sub_1b5820(cx));
    // 0.01 - dt <= 0, +0x152c >= 1 -> DAT_002c0948[(0x1cb - 0x1cb) + 5] = 458
    let sw: Vec<_> = h.ee.iter().filter(|e| e.0 == 0x1f0660).collect();
    assert_eq!(sw.last().unwrap().1, vec![0x44, 458]);
    h.label = 0x1c0;
    h.ee.clear();
    with(&mut m, &g, &mut rng, &mut h, |m, cx| m.ee_sub_1b5820(cx));
    assert!(h.ee.iter().all(|e| e.0 != 0x1f0660), "outside 0x1cb..0x1cf nothing happens");
}

/// state 9 / 0xb / 0xc updates reach their sub-states; FUN_001b70a0 lifts the ally
#[test]
fn k20_c54_states_9_b_c() {
    let (mut m, mut g, mut rng) = inited();
    let mut h = H::default();
    with(&mut m, &g, &mut rng, &mut h, |m, cx| m.ee_sub_1b6078(O, 9, cx));
    assert_eq!((m.b.state, m.b.m.i(0x17f4)), (9, 0));
    // 8 held -> leave the crouch
    press(&mut m, g.t, 0, 8, 0.0);
    with(&mut m, &g, &mut rng, &mut h, |m, cx| m.ee_sub_1b5990(O, cx));
    assert_eq!(m.b.state, 0, "8 held / FUN_00140af8 0 -> SetState 0");
    // ally lift through FUN_001b70a0: in reach, 0x80 pressed
    m.b.m.set_u(0x10dc, ALLY);
    m.b.m.set_u(0x180c, 1);
    h.ally_pos = [100.0, 0.0, 50.0];
    press(&mut m, g.t, 0x80, 0, 0.0);
    let r = with(&mut m, &g, &mut rng, &mut h, |m, cx| m.ee_sub_1b70a0(O, cx));
    assert!(r);
    assert_eq!((m.b.state, m.b.m.i(0x1810)), (0xb, 4));
    // pickup branch: z in (100, 500), 0x20
    with(&mut m, &g, &mut rng, &mut h, |m, cx| m.ee_sub_1b6078(O, 0, cx));
    h.ally_pos = [100.0, 0.0, 200.0];
    h.ee.clear();
    press(&mut m, g.t, 0x20, 0, 0.0);
    assert!(with(&mut m, &g, &mut rng, &mut h, |m, cx| m.ee_sub_1b70a0(O, cx)));
    assert!(h.ee.iter().any(|e| e.0 == 0x17b1a8));
    // 0xb: anim over -> 0
    with(&mut m, &g, &mut rng, &mut h, |m, cx| m.ee_sub_1b8508(O, cx));
    assert_eq!(m.b.state, 0xb);
    m.b.m.set_u(0x1030, 0x33);
    h.done = true;
    with(&mut m, &g, &mut rng, &mut h, |m, cx| m.ee_sub_1b8988(O, cx));
    assert_eq!(m.b.state, 0);
    // 0xc push: 0 (anim over) -> 1 -> (timeout) 3 -> stick -> state 0
    h.done = false;
    m.b.m.set_u(0x184c, 0x55);
    m.b.m.set_u(0x1904, 0x56);
    with(&mut m, &g, &mut rng, &mut h, |m, cx| m.ee_sub_1b9a08(O, 0x14, cx));
    assert_eq!((m.b.state, m.b.m.i(0x1840)), (0xc, 1));
    m.b.m.set_u(0x1030, 0x33);
    g.t += 1.0;
    with(&mut m, &g, &mut rng, &mut h, |m, cx| m.ee_sub_1b8ec0(O, cx));
    assert_eq!(m.b.m.i(0x1840), 3);
    m.b.m.set_u(0x1030, 0x33);
    m.b.joy_80 = Some(1);
    press(&mut m, g.t + 1.0, 0, 0, 1.0);
    g.t += 1.0;
    with(&mut m, &g, &mut rng, &mut h, |m, cx| m.ee_sub_1b8ec0(O, cx));
    assert_eq!(m.b.state, 0);
}

/// FUN_001d69f0 + FUN_001b7970 throw: reaches +0x1420 in |d.xy| / +0x1414 s under gravity
#[test]
fn k20_c54_throw() {
    let v = FUN_001d69f0(2.0, [0.0; 3], [200.0, 0.0, 0.0], [0.0, 0.0, -1000.0]);
    assert_eq!(v, [100.0, 0.0, 1000.0]);
    let (mut m, g, mut rng) = inited();
    let mut h = H { in_world: true, ..Default::default() };
    m.b.m.set_u(0x13f0, HELD);
    m.b.m.set_v3(0x1420, [400.0, 0.0, 0.0]);
    m.b.m.set_f(0x1414, 200.0);
    with(&mut m, &g, &mut rng, &mut h, |m, cx| m.ee_sub_1b7970(O, cx.g.t, cx.h));
    assert_eq!(m.b.m.u(0x13f0), 0);
    assert_eq!(h.vel.last().copied(), Some([200.0, 0.0, 2000.0]));
    // slot 86: hide (vt+0xac 0) then show (1)
    m.b.m.set_u(0x13f0, HELD);
    m.ee_sub_1b81b8(O, 1, &mut h);
    assert_eq!((m.b.m.u(0xf1c), h.vc.last().copied()), (X, Some((X, 0xac, 0))));
    m.ee_sub_1b81b8(O, 0, &mut h);
    assert_eq!((m.b.m.u(0xf1c), h.vc.last().copied()), (0, Some((X, 0xac, 1))));
}

/// FUN_001403d0 base update order (Bentley and Murray slot 15 run it)
#[test]
fn k20_base_update_1403d0() {
    let (mut m, g, mut rng) = inited();
    let mut h = H::default();
    m.b.dmg.state = -1;
    with(&mut m, &g, &mut rng, &mut h, |m, cx| m.b.ee_sub_1403d0(O, cx));
    let want = [0x203fc0, 0x141a50, 0x144640, 0x1449c8, 0x140558, 0x1a2460, 0x140750, 0x142af8, 0x1449c8, 0x1eed10];
    assert_eq!(h.calls, want);
}

/// every level with Murray: load + load-post + 300 frames controlled (the K20 state updates run)
#[test]
fn k20_murray_all_levels_300_frames() {
    let (loader, descs) = levels();
    let mut nm = 0;
    let mut seen = BTreeSet::new();
    for d in descs {
        let lv = loader.load_blocking(d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let ms: Vec<usize> = w.objects.iter().enumerate().filter(|x| x.1.class == 54).map(|x| x.0).collect();
        if ms.is_empty() {
            continue;
        }
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        s.post_load(&mut h);
        for &o in &ms {
            s.k18_set_active(Some(o), &mut h);
            for f in 0..300 {
                s.g.dt = DT;
                s.g.t += DT;
                let mag = if (f / 30) % 2 == 0 { 1.0 } else { 0.0 };
                let pressed = [0, 0x40, 0x20, 0x80, 0x10, 0xa0][(f / 50) % 6];
                let held = if (f / 75) % 2 == 0 { 0 } else { 8 };
                let mut j = Joy { angle: f as f32 * 0.01, mag, held, ..Default::default() };
                j.feed(s.g.t, held, pressed);
                s.k18_feed_joy(j);
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

// ---- class 77 (classes/k20_77.rs)
use sly2_recomp::engine::classes::k20_77::{C2d7ca8, Part};

#[derive(Default)]
struct H77 {
    ee: Vec<(u32, Vec<u32>)>,
    vc: Vec<(u32, u32, i32)>,
    vel: Vec<[f32; 3]>,
    spin: Vec<[f32; 3]>,
}
impl ClassHost for H77 {
    fn ee_call(&mut self, addr: u32, args: &[u32]) -> i64 {
        self.ee.push((addr, args.to_vec()));
        if addr == 0x1fbb90 { 0x99 } else { 0 }
    }
    fn vcall_arg(&mut self, obj: u32, off: u32, a: i32) {
        self.vc.push((obj, off, a));
    }
    fn vcall_11c_v(&mut self, _obj: u32, v: [f32; 3]) {
        self.vel.push(v);
    }
    fn vcall_124_v(&mut self, _obj: u32, v: [f32; 3]) {
        self.spin.push(v);
    }
}

/// state machine 0 -> 2 (slot 77) -> 0 -> 3 (hit points out), the sm labels, the part labels in state 2
#[test]
fn k20_c77_states() {
    let mut c = C2d7ca8::default();
    let mut h = H77::default();
    let mut rng = GameRng::default();
    c.ee_sub_1d1370(O, &mut h);
    assert_eq!((c.state(), c.m.i(0x55c), c.m.f(0x51c)), (-1, 100, 500.0));
    c.m.set_u(0x570, 0x40);
    c.m.set_i(0x59c, 0x1a9);
    c.m.set_f(0x4a0, 100.0);
    c.ee_sub_1d14c0(O, 0.0, &mut h);
    assert_eq!((c.state(), c.m.i(0x560)), (0, 100));
    assert!(h.ee.iter().any(|e| e == &(0x1f0730, vec![0x40, 0x73])));
    // slot 77: driven; the engine sound starts
    c.ee_sub_1d1940(O, 1, 1.0, &mut h);
    assert_eq!((c.state(), c.m.u(0x598), c.m.f(0x184)), (2, 0x99, 1.0));
    c.parts.push(Part { obj: 0x10, sm: 0x20, mask: 0x8, button: 0x20, stage: 0, thresh: 0 });
    c.joy.feed(1.0, 0x40, 0x20);
    c.joy.angle = 1.0;
    c.ee_sub_1d19a8(O, 0.0, 0.0, 1.0, &mut h);
    assert_eq!((c.m.f(0x550), c.m.f(0x544)), (500.0, 1.0));
    assert!((c.m.f(0x548) - 0.5).abs() < 1e-6, "steer 0.8 * 1 clamped to 0.5");
    assert!(h.ee.iter().any(|e| e == &(0x1f0660, vec![0x20, 0x68])));
    c.ee_sub_1d1940(O, 0, 2.0, &mut h);
    assert_eq!((c.state(), c.m.u(0x598)), (0, 0));
    // hit: -60 then -60 -> 0 -> state 3 (message 5); 0x100 kicks
    assert!(c.ee_sub_1d4568(O, true, 0, 0x100, -60, 3.0, &mut rng, &mut h));
    assert_eq!((c.m.i(0x560), c.state()), (40, 0));
    let v = h.vel[0];
    let r = (v[0] * v[0] + v[1] * v[1]).sqrt();
    assert!((50.0..=100.0).contains(&r) && (300.0..=400.0).contains(&v[2]));
    c.ee_sub_1d4568(O, true, 0, 0, -60, 3.0, &mut rng, &mut h);
    assert_eq!((c.m.i(0x560), c.state()), (0, 3));
    assert!(h.vc.contains(&(O as u32 + 1, 0x1c, 5)));
    assert!(h.ee.iter().any(|e| e == &(0x1f0730, vec![0x40, 0x76])));
}

/// every class-77 instance: build + load-post + 300 frames
#[test]
fn k20_c77_all_levels_300_frames() {
    let (loader, descs) = levels();
    let mut n = 0;
    for d in descs {
        let lv = loader.load_blocking(d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        if !w.objects.iter().any(|o| o.class == 77) {
            continue;
        }
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        s.post_load(&mut h);
        let objs: Vec<usize> = s.k20.iter().map(|x| x.0).collect();
        for &o in &objs {
            assert_eq!(s.k20_77(o).unwrap().state(), 0);
            assert_eq!(s.k20_77(o).unwrap().m.i(0x59c), 0x1a9);
        }
        let t = s.g.t;
        s.k20_77(objs[0]).unwrap().ee_sub_1d1940(objs[0], 1, t, &mut h);
        for _ in 0..300 {
            s.g.dt = DT;
            s.g.t += DT;
            s.frame(&mut h);
        }
        assert_eq!(s.k20_77(objs[0]).unwrap().state(), 2);
        n += objs.len();
    }
    assert!(n > 0);
}

// ---- class 119 (classes/k20_119.rs)
use sly2_recomp::engine::classes::k20_119::C2cfa90;

/// spawner host: player 0x900 at `pp`, guards 0xa00.. in the world unless `dead`, point objects in the world
#[derive(Default)]
struct H119 {
    pp: [f32; 3],
    next: u32,
    dead: Vec<u32>,
    gone: Vec<u32>,
    spawned: Vec<u32>,
}
impl ClassHost for H119 {
    fn ee_call(&mut self, addr: u32, _args: &[u32]) -> i64 {
        if addr == 0x1c4840 { 0x900 } else { 0 }
    }
    fn ee_sub_1a62e8(&mut self, _obj: u32) -> [f32; 3] {
        self.pp
    }
    fn ee_sub_15e938(&mut self, _k: i32, oid: i32, _c: u32) -> u32 {
        0x100 + oid as u32
    }
    fn po_call(&mut self, addr: u32, _obj: usize, _off: u32, _args: &[u32]) -> i64 {
        match addr {
            0x173f18 => 1,
            0x174490 => {
                self.next += 1;
                let g = 0xa00 + self.next;
                self.spawned.push(g);
                g as i64
            }
            _ => 0,
        }
    }
    fn obj_i32(&mut self, obj: u32, off: u32) -> i32 {
        (off == 0x330 && self.dead.contains(&obj)) as i32
    }
    fn obj_flags_28(&mut self, obj: u32) -> u64 {
        if self.gone.contains(&obj) { 0 } else { 1 << 60 }
    }
}

/// slot 15: spawns into free slots every DAT_002be5d8, frees dead / vanished guards, recycles nothing while
/// every slot is live and the player stands still
#[test]
fn k20_c119_spawner() {
    let mut c = C2cfa90::default();
    let mut h = H119::default();
    c.ee_sub_173158(3, &mut h);
    assert_eq!((c.m.f(0x80), c.m.f(0xa0), c.m.f(0xa4)), (5500.0, 30.0, 50.0));
    c.add_point(0, 7);
    c.add_point(2, 8);
    c.m.set_i(0x5c, 2);
    c.m.set_i(0xac, 2);
    c.ee_sub_1733f8(3, &mut h);
    assert_eq!((c.slots.len(), c.points[1].obj), (2, 0x108));
    let mut t = 1.0;
    c.ee_sub_173a68(3, t, &mut h);
    assert_eq!((c.m.i(0x58), h.spawned.len(), c.m.f(0xa8)), (1, 1, t + 2.5));
    t += 2.6;
    c.ee_sub_173a68(3, t, &mut h);
    assert_eq!(c.m.i(0x58), 2);
    t += 2.6;
    c.ee_sub_173a68(3, t, &mut h);
    assert_eq!(h.spawned.len(), 2, "all live, player still: no spawn");
    // guard 1 dies (+0x330): slot state 1 waits +0xa0 = 30 s; guard 2 leaves the world: state 0, free now
    h.dead.push(h.spawned[0]);
    h.gone.push(h.spawned[1]);
    t += 1.0;
    c.ee_sub_173a68(3, t, &mut h);
    assert_eq!(h.spawned.len(), 2, "freed this frame: free time == t is not < t");
    t += 1.0;
    c.ee_sub_173a68(3, t, &mut h);
    assert_eq!(h.spawned.len(), 3, "the vanished guard's slot respawns on the next retry");
    assert_eq!(c.slots.iter().filter(|s| s.guard == 0 && s.state == 1).count(), 1);
    assert_eq!(c.m.i(0x58), 2 - 2 + 1);
    // the dead slot re-arms when the player is > 3500 away
    h.pp = [4000.0, 0.0, 0.0];
    t += 0.1;
    c.ee_sub_173a68(3, t, &mut h);
    assert!(c.slots.iter().all(|s| s.state == 0 || s.guard != 0));
    // slot 11 frees everything quietly
    c.ee_sub_1746b8(3, t, &mut h);
    assert!(c.slots.iter().all(|s| s.guard == 0));
    assert_eq!(c.m.i(0x58), 0);
}

/// FUN_00173e78: a free slot wins; else the guard idle longest beyond 4 s that is not `keep`
#[test]
fn k20_c119_slot_choice() {
    let mut c = C2cfa90::default();
    c.slots = vec![Default::default(); 3];
    for (i, s) in c.slots.iter_mut().enumerate() {
        s.guard = 0x10 + i as u32;
        s.active = i as f32;
    }
    assert_eq!(c.ee_sub_173e78(0, 10.0), Some(0));
    assert_eq!(c.ee_sub_173e78(0x10, 10.0), Some(1));
    assert_eq!(c.ee_sub_173e78(0, 4.0), None);
    c.slots[2].guard = 0;
    c.slots[2].free = 9.0;
    assert_eq!(c.ee_sub_173e78(0, 10.0), Some(2));
}

/// every level with class 119: load + post_load + 300 frames (default host: no spawns)
#[test]
fn k20_c119_all_levels() {
    let (loader, descs) = levels();
    let mut n = 0;
    for d in descs {
        let lv = loader.load_blocking(d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        if !w.objects.iter().any(|o| o.class == 119) {
            continue;
        }
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        s.post_load(&mut h);
        for (i, o) in w.objects.iter().enumerate() {
            if o.class == 119 {
                let c = s.k20_119(i).unwrap();
                assert!(c.m.i(0x5c) >= 1 && !c.points.is_empty(), "obj {i}: slots / points from the field rows");
                assert_eq!(c.slots.len(), c.m.i(0x5c) as usize);
                n += 1;
            }
        }
        for _ in 0..300 {
            s.g.dt = DT;
            s.g.t += DT;
            s.frame(&mut h);
        }
    }
    eprintln!("class 119: {n}");
    assert!(n > 0);
}
