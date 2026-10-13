//! WP-K23 tests.
use sly2_recomp::engine::classes::k15_101::{C2c9810, Cx};
use sly2_recomp::engine::classes::k23_102;
use sly2_recomp::engine::classes::plugin::EngineHost;
use sly2_recomp::engine::classes::ClassHost;

const DT: f32 = 1.0 / 60.0;

#[path = "engine_k20.rs"]
#[allow(dead_code)]
mod k20;

/// class-102 host: one guard (handle 7) at `gpos`, its hold component state, constraint scale, record links
#[derive(Default)]
struct H102 {
    gpos: [f32; 3],
    eligible: bool,
    comp: Option<i32>,
    scale: Vec<[f32; 3]>,
    linked: Vec<(u32, u32)>,
    unlinked: Vec<(u32, u32)>,
    vel: Vec<[f32; 3]>,
    brain: u32,
    world_1c6fd8: Vec<i32>,
    knock: u32,
    accel: Vec<[f32; 3]>,
}
impl ClassHost for H102 {
    fn query_15e9c8(&mut self, class: i32) -> Vec<u32> {
        if class == 0x38 { vec![7] } else { Vec::new() }
    }
    fn ee_sub_1a62e8(&mut self, obj: u32) -> [f32; 3] {
        if obj == 7 { self.gpos } else { [0.0; 3] }
    }
    fn k23_guard_206eb0(&mut self, _obj: u32, k: i32) -> bool {
        k == 0xb && self.eligible
    }
    fn k23_comp_b0(&mut self, _g: u32) -> Option<i32> {
        self.comp
    }
    fn k23_comp_b0_set(&mut self, _g: u32, s: i32) {
        self.comp = Some(s);
    }
    fn k23_guard_fc4(&mut self, _g: u32) -> u32 {
        9
    }
    fn k22_pin_12d108(&mut self, _h: u32, v: [f32; 3]) {
        self.scale.push(v);
    }
    fn k23_link_229710(&mut self, obj: u32, owner: u32) {
        self.linked.push((obj, owner));
    }
    fn ee_sub_229738(&mut self, obj: u32, rec: u32) {
        self.unlinked.push((obj, rec));
    }
    fn k22_speed_10c(&mut self, _obj: u32) -> [f32; 3] {
        [10.0, 20.0, 30.0]
    }
    fn vcall_11c_v(&mut self, _obj: u32, v: [f32; 3]) {
        self.vel.push(v);
    }
    fn k23_brain_209e28(&mut self, _g: u32, _a: i32) {
        self.brain += 1;
    }
    fn k23_world_1c6fd8(&mut self, kind: i32, _p: [f32; 3], _src: u32, _k: i32) {
        self.world_1c6fd8.push(kind);
    }
    fn vcall(&mut self, obj: u32, off: u32) {
        if obj == 7 && off == 0x4c {
            self.knock += 1;
        }
    }
    fn phys_e8_80(&mut self, _obj: usize) -> [f32; 3] {
        [0.0, 0.0, -980.0]
    }
    fn ee_sub_1fac30(&mut self, _obj: usize, a: [f32; 3]) {
        self.accel.push(a);
    }
}

fn bomb102(h: &mut dyn ClassHost) -> C2c9810 {
    let mut c = C2c9810 { k23: Some(Default::default()), ..Default::default() };
    k23_102::ee_sub_2252e0(&mut c, 0, h);
    c
}

/// class 102 state machine: 5 grabs the guard (link, velocity {x/2, y/2, 400}, brain message), the hold
/// component goes to 4 within 100, the scale shrinks, after 3 s the guard is knocked out and the bomb goes to
/// 3 (record unlinked, component 2, scale {1,1,1}); slot 70 cancels gravity and springs the guard
#[test]
fn k23_c102_grab_hold_knockout() {
    let mut h = H102 { gpos: [50.0, 0.0, 0.0], eligible: true, comp: Some(0), ..Default::default() };
    let mut c = bomb102(&mut h);
    assert_eq!((c.f_150, c.f_158, c.f_170), (-1, -1, 3000.0));
    assert_eq!(sly2_recomp::engine::classes::k15_101::slot5(102), Some(0x2bb2e0));
    let mut t = 1.0;
    c.slot75(0, 5, &mut Cx { t, dt: DT, h: &mut h });
    assert_eq!(c.f_150, 5);
    let e = c.k23.clone().unwrap();
    assert_eq!((e.f_1b4, e.f_1b8, e.f_1bc, e.grabs), (7, 1, 7, 1));
    assert_eq!(h.linked, vec![(7, 1)]);
    assert_eq!(h.vel, vec![[5.0, 10.0, 400.0]]);
    assert_eq!(h.brain, 1);
    let mut states = std::collections::BTreeSet::new();
    for _ in 0..240 {
        t += DT;
        let mut cx = Cx { t, dt: DT, h: &mut h };
        c.slot15(0, &mut cx);
        if c.k23.as_ref().unwrap().f_1b4 != 0 {
            k23_102::ee_sub_2255c8(&c, 0, DT, &mut h);
        }
        states.insert(c.f_150);
    }
    eprintln!("states {states:?} scale {:?}..{:?}", h.scale.first(), h.scale.last());
    assert!(states.contains(&5) && states.contains(&3), "{states:?}");
    assert_eq!(c.f_150, 3);
    assert_eq!(h.knock, 1);
    assert_eq!(h.world_1c6fd8, vec![0x5d]);
    assert_eq!(c.k23.as_ref().unwrap().knockouts, 1);
    assert_eq!(c.k23.as_ref().unwrap().f_1b4, 0);
    assert_eq!(h.unlinked, vec![(7, 1)]);
    assert_eq!(h.comp, Some(2));
    assert_eq!(*h.scale.last().unwrap(), [1.0; 3]);
    // the scale clamps to 1 at first (1.2 - f^2) and falls below 1 before the knock-out
    assert_eq!(h.scale[0], [1.0; 3]);
    assert!(h.scale.iter().any(|v| v[0] < 0.5));
    // slot 70: -gravity then a spring toward the anchor (the bomb at the origin)
    assert_eq!(h.accel[0], [0.0, 0.0, 980.0]);
    assert!(h.accel[1][0] < 0.0);
}

/// no eligible guard: state 5 holds nobody and slot 15 moves on to 3; a guard whose component is -1 -> 2
#[test]
fn k23_c102_no_guard() {
    let mut h = H102 { gpos: [50.0, 0.0, 0.0], eligible: false, comp: Some(0), ..Default::default() };
    let mut c = bomb102(&mut h);
    c.slot75(0, 5, &mut Cx { t: 1.0, dt: DT, h: &mut h });
    assert_eq!(c.k23.as_ref().unwrap().f_1b4, 0);
    c.slot15(0, &mut Cx { t: 1.1, dt: DT, h: &mut h });
    assert_eq!(c.f_150, 3);
    // too far (400)
    let mut h = H102 { gpos: [500.0, 0.0, 0.0], eligible: true, comp: Some(0), ..Default::default() };
    let mut c = bomb102(&mut h);
    c.slot75(0, 5, &mut Cx { t: 1.0, dt: DT, h: &mut h });
    assert_eq!(c.k23.as_ref().unwrap().f_1b4, 0);
    // component state -1 -> 2
    let mut h = H102 { gpos: [50.0, 0.0, 0.0], eligible: true, comp: Some(0), ..Default::default() };
    let mut c = bomb102(&mut h);
    c.slot75(0, 5, &mut Cx { t: 1.0, dt: DT, h: &mut h });
    h.comp = Some(-1);
    c.slot15(0, &mut Cx { t: 1.1, dt: DT, h: &mut h });
    assert_eq!(c.f_150, 2);
    assert_eq!(h.unlinked, vec![(7, 1)]);
    // the parent's virtual slot-75 calls dispatch to class 102 (slot 74 from state 2 -> 3)
    assert!(c.ee_sub_224190(0, &mut Cx { t: 1.2, dt: DT, h: &mut h }));
    assert_eq!(c.f_150, 3);
}

/// every class-102 instance on all levels builds (class 102 data, row 1113) and runs 300 frames
#[test]
fn k23_c102_levels() {
    use sly2_recomp::engine::classes::ClassSet;
    use sly2_recomp::engine::world::{LoadOptions, World};
    use std::sync::Arc;
    let (loader, descs) = k20::levels();
    let (mut n, mut levels, mut anchored) = (0, 0, 0);
    for d in descs {
        let Ok(lv) = loader.load_blocking(d) else { continue };
        let Ok(w) = World::load(&lv, LoadOptions::default()) else { continue };
        if !w.objects.iter().any(|o| o.class == 102) {
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
        for (i, o) in w.objects.iter().enumerate() {
            if o.class != 102 {
                continue;
            }
            let c = s.k15_101_get(i).unwrap_or_else(|| panic!("class 102 obj {i} not built"));
            let e = c.k23.as_ref().expect("class-102 members");
            anchored += (e.f_1b0 != 0) as usize;
            assert!(c.phys.pos.iter().all(|v| v.is_finite()));
            assert_eq!(c.f_170, 3000.0);
            n += 1;
        }
    }
    eprintln!("class 102: {n} instances on {levels} levels, {anchored} anchored");
    assert!(n >= 40 && levels >= 40, "class-102 instances {n} levels {levels}");
}

/// class 32 / 33 template: FUN_00159118 defaults, field writes, slot 14 resolves +0x20 (and kind-2 +0x30) into
/// the block and copy-on-writes when it changed; slot 40 adds a reference
#[test]
fn k23_c32_template_slot14() {
    use sly2_recomp::engine::classes::k23_32;
    #[derive(Default)]
    struct H32(Vec<(i32, i32, u32)>);
    impl ClassHost for H32 {
        fn ee_sub_15e938(&mut self, kind: i32, oid: i32, obj: u32) -> u32 {
            self.0.push((kind, oid, obj));
            oid as u32 + 100
        }
    }
    assert_eq!(k23_32::ee_sub_113c10(), 0x2bad68);
    let mut c = k23_32::C2ca098::default();
    assert_eq!((c.tpl.w[0], c.tpl.w[1], c.tpl.w[8]), (1, u32::MAX, u32::MAX));
    let mut h = H32::default();
    // nothing to resolve (t[1] = -1): block stays zero, no copy
    k23_32::ee_sub_259170(&mut c.tpl, &mut c.f_e0, 4, &mut h);
    assert!(h.0.is_empty());
    assert_eq!((c.f_e0, c.tpl.block), (0, [0; 5]));
    // source oid 7 at +0x20, two references -> resolved, private copy (refcount 1)
    c.tpl.w[8] = 7;
    k23_32::ee_sub_2590b8(&mut c.tpl, 4, &mut h);
    assert_eq!(c.tpl.w[0], 2);
    k23_32::ee_sub_259170(&mut c.tpl, &mut c.f_e0, 4, &mut h);
    assert_eq!(h.0, vec![(4, 7, 5)]);
    assert_eq!((c.f_e0, c.tpl.w[0], c.tpl.block[0]), (1, 1, 107));
    // kind 2: +0x30 replaces the source; emitter kind 1: +0xc8 / +0xcc / +0xd0
    c.tpl.w[4] = 2;
    c.tpl.w[12] = 9;
    c.tpl.w[1] = 1;
    c.tpl.w[50] = 1;
    c.tpl.w[51] = u32::MAX;
    c.tpl.w[52] = 3;
    k23_32::ee_sub_259170(&mut c.tpl, &mut c.f_e0, 4, &mut h);
    assert_eq!(c.tpl.block, [0, 109, 101, 0, 103]);
}

/// every class-32 / 33 instance on all levels builds and resolves its template in slot 14, 300 frames
#[test]
fn k23_c32_levels() {
    use sly2_recomp::engine::classes::ClassSet;
    use sly2_recomp::engine::world::{LoadOptions, World};
    use std::sync::Arc;
    let (loader, descs) = k20::levels();
    let (mut n, mut levels, mut src, mut n33) = (0, 0, 0, 0);
    for d in descs {
        let Ok(lv) = loader.load_blocking(d) else { continue };
        let Ok(w) = World::load(&lv, LoadOptions::default()) else { continue };
        n33 += w.objects.iter().filter(|o| o.class == 33).count();
        if !w.objects.iter().any(|o| o.class == 32) {
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
        for (i, o) in w.objects.iter().enumerate() {
            if o.class != 32 {
                continue;
            }
            let c = s.k23_32_get(i).unwrap_or_else(|| panic!("class 32 obj {i} not built"));
            src += (c.tpl.block[0] != 0 || c.tpl.block[1] != 0) as usize;
            n += 1;
        }
    }
    eprintln!("class 32: {n} instances on {levels} levels, {src} with a resolved source; class 33: {n33}");
    assert!(n >= 40, "class-32 instances {n}");
}

#[derive(Default)]
struct HSeq23 {
    clock: Vec<(f32, usize)>,
    streams: Vec<(i32, i32)>,
    playing: bool,
    other: Vec<u32>,
}
impl ClassHost for HSeq23 {
    fn ee_sub_15e938(&mut self, _kind: i32, oid: i32, _ctx: u32) -> u32 {
        oid as u32
    }
    fn k23_clock_14d1d0(&mut self, v: f32, index: usize) {
        self.clock.push((v, index));
    }
    fn ee_sub_1fe5f0(&mut self, id: i32, k: i32) -> u32 {
        self.streams.push((id, k));
        self.playing = true;
        1
    }
    fn k23_stream_1fe848(&mut self, _id: i32) -> bool {
        self.playing
    }
    fn k22_seq_action(&mut self, _owner: u32, _subj: u32, rec: &[u32; 20]) {
        self.other.push(rec[0]);
    }
}

/// task 2: sequence actions 0xc (context oid 40, then -1 -> owner), 0x10 (clock scale 0 -> owner bit 0x100,
/// then 1 -> cleared), 0x25 / 0x26, 0x45 (stream start, the node waits while it plays)
#[test]
fn k23_seq_actions() {
    use sly2_recomp::engine::classes::c2d8c48::C2d8c48;
    use sly2_recomp::engine::classes::k22_seq::ee_sub_1e1c40;
    use sly2_recomp::engine::game::methods::cls::ee_sub_1dfd58;
    let mut c = C2d8c48::default();
    let mut h = HSeq23::default();
    c.ee_sub_1dfad8(0, &mut h);
    c.feed(&[]);
    c.f_58 = -1;
    let mut add = |k: u32, b: u32, a: &[u32]| {
        let i = c.ee_sub_1e01f8(k, b).unwrap();
        c.f_54[i][4..4 + a.len()].copy_from_slice(a);
    };
    add(0xc, 1, &[40]);
    add(0x10, 1, &[0]);
    add(0x25, 1, &[]);
    add(0x45, 0, &[17, 1]);
    add(0x10, 1, &[1.0f32.to_bits()]);
    add(0x26, 1, &[]);
    add(0x2e, 1, &[1, 2]);
    add(0xc, 1, &[u32::MAX]);
    add(0x45, 0, &[18, 0]);
    let mut nodes = Vec::new();
    ee_sub_1dfd58(&mut c, &mut nodes, 0, 0, &mut h);
    let step = |c: &mut C2d8c48, h: &mut HSeq23| {
        for n in c.f_5c.clone() {
            ee_sub_1e1c40(c, 0, n, 0.0, h);
        }
    };
    step(&mut c, &mut h);
    assert_eq!(c.k22_nodes[0].ctx, 40);
    assert_eq!(h.clock, vec![(0.0, 6)]);
    assert_ne!(c.f_40 & 0x100, 0);
    assert_eq!(c.k22_nodes[0].f_e0, 1);
    assert_eq!(h.streams, vec![(17, 1)]);
    assert_eq!(c.k22_nodes[0].f_68, Some(4));
    step(&mut c, &mut h);
    assert_eq!(c.k22_nodes[0].cur, Some(4), "waits on the stream");
    h.playing = false;
    step(&mut c, &mut h);
    assert_eq!(h.clock, vec![(0.0, 6), (1.0, 6)]);
    assert_eq!(c.f_40 & 0x100, 0);
    assert_eq!(h.other, vec![0x2e]);
    assert_eq!(h.streams, vec![(17, 1), (18, 0)]);
    assert_eq!(c.k22_nodes[0].f_e0, 0);
    assert_eq!(c.k22_nodes[0].ctx, 1, "-1 -> the owner");
}
