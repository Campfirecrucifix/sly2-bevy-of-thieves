//! WP-K17: class 55 (Bentley) clusters 55f (init), 55e (slots 82 / 83), 55d (draw), 55c (small slots),
//! 55g (shared engine callees) and 55h (state machine) in classes/k17_55.rs.
use sly2_recomp::engine::classes::k16_55::C2cb230;
use sly2_recomp::engine::classes::plugin::EngineHost;
use sly2_recomp::engine::classes::{ClassGlobals, ClassHost, ClassSet, NullHost};
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

/// a Bentley object after slot 6 (object index 7)
fn inited() -> (C2cb230, ClassGlobals) {
    let mut c = C2cb230::default();
    let mut g = ClassGlobals { g_2c3b90: 400f32.to_bits(), ..Default::default() };
    c.ee_sub_134648(7, None, &mut g, &mut NullHost);
    (c, g)
}

/// 55f: slot 6 FUN_00134648 (+ class-52 base FUN_0013ff70, PO FUN_00203948, component inits)
#[test]
fn k17_c55f_init() {
    let (c, g) = inited();
    assert_eq!(g.g_2dd5bc, 8);
    assert_eq!((c.state, c.f_16b0, c.m.i(0x16bc), c.m.i(0x1800), c.m.i(0x1040)), (-1, -1, -1, -1, 0));
    // FUN_00134648 overrides the base's +0x103c (20) and the JMP's +0x3d8 (-150)
    assert_eq!((c.m.f(0x103c), c.m.f(0x3d8), c.f_1048), (45.0, -70.0, 125.0));
    // JMP +0x2c0
    assert_eq!((c.m.u(0x2c0), c.f_2dc, c.m.i(0x2c4)), (8, -1, -1));
    assert_eq!([c.m.f(0x3b0), c.m.f(0x3b4), c.m.f(0x3b8), c.m.f(0x3bc)], [37.0, 0.0, 25.0, 0.0]);
    assert_eq!([c.m.f(0x3c8), c.m.f(0x3cc), c.m.f(0x3d0), c.m.f(0x3d4), c.m.f(0x3dc), c.m.f(0x424)], [-60.0, -50.0, 75.0, 50.0, 75.0, 1.0]);
    // attach +0x4c0, its FUN_0020b840 sub-blocks (k 0x6f) and FUN_001cc158
    assert_eq!((c.att_4c8, c.att_4cc, c.m.i(0x4c0 + 0x4fc)), (-1, -1, -1));
    assert_eq!((c.m.u(0x640), c.m.u(0x6d0), c.m.f(0x6d4), c.m.f(0x6d8)), (8, 0x6f, f32::from_bits(0x3f19999a), 1.0));
    assert_eq!((c.m.u(0x700), c.m.u(0x790)), (8, 0x6f));
    assert_eq!((c.m.i(0x900 + 0x90), c.m.i(0x900 + 0xa4)), (-1, -1));
    // +0x1320 / +0x1600 (k 0x73)
    assert_eq!((c.m.u(0x1320), c.m.u(0x13b0), c.m.u(0x1600), c.m.u(0x1690)), (8, 0x73, 8, 0x73));
    // damage / binoc / path / +0x11b0
    assert_eq!(c.dmg, sly2_recomp::engine::player::p7_damage::Dmg::default());
    assert_eq!((c.binoc.state, c.binoc.mode), (-1, -1));
    assert_eq!((c.f_f54, c.m.f(0xf80), c.m.f(0xf88), c.m.f(0xf98)), (-1, 45.0, 40.0, -20.0));
    assert_eq!((c.m.f(0x11cc), c.m.f(0x11d0), c.m.f(0x1270), c.m.f(0x1284)), (500.0, 1500.0, 1000.0, -73.0));
    // PO
    assert_eq!((c.m.i(0x15c), c.m.u(0x168), c.alo.f_40 & 2), (1, 8, 2));
    assert_eq!((c.m.f(0x1f0), c.m.f(0x1f4), c.m.f(0x1f8), c.m.f(0x1fc)), (1.0, 25.0, 0.0, 0.25));
    assert_eq!((c.m.f(0x1b0), c.m.f(0x1c4), c.m.f(0x1d8)), (1.0, 1.0, 1.0));
    assert_eq!((c.m.f(0x220), c.m.f(0x214), c.m.i(0x264), c.m.u(0x26c)), (30.0, -50.0, -1, 8));
    // ALO: FUN_001f2108 + FUN_001f7918 (0 everywhere, then 8 on bits 8..15 and 20)
    assert_eq!(c.alo.so.f_140, -1);
    for k in 0..26 {
        assert_eq!(c.alo.f_110[k], if (8..16).contains(&k) || k == 20 { 8 } else { 0 }, "byte {k}");
    }
    // FUN_0013ff70 tail + FUN_00134648 tail
    assert_eq!((c.f_1300, c.m.f(0x1304), c.m.i(0x12f0), c.m.i(0x1318)), (100, -60.0, 1, -1));
    assert_eq!((c.m.f(0x10b4), c.m.i(0x1000), c.m.i(0x1824), c.m.i(0x1870)), (4500.0, 2, 5, 5));
}

#[derive(Default)]
struct Rec {
    calls: Vec<(u32, i64, i64)>,
    isa: i32,
    have_link: bool,
}
impl ClassHost for Rec {
    fn ee_sub_135248(&mut self, obj: usize, state: i32) {
        self.calls.push((0x135248, obj as i64, state as i64));
    }
    fn ee_sub_15e938(&mut self, kind: i32, oid: i32, ctx: u32) -> u32 {
        self.calls.push((0x15e938, kind as i64, oid as i64));
        assert_eq!(ctx, 8);
        if oid == -1 {
            0
        } else {
            1000 + oid as u32
        }
    }
    fn ee_sub_1f03c8(&mut self, sm: u32, obj: u32) -> u32 {
        self.calls.push((0x1f03c8, sm as i64, obj as i64));
        if self.have_link {
            sm + 1
        } else {
            0
        }
    }
    fn ee_sub_1f01b8(&mut self, sm: u32, obj: u32, a: i32, b: u32) -> u32 {
        self.calls.push((0x1f01b8, sm as i64, (a as i64) * 100 + b as i64));
        assert_eq!(obj, 8);
        sm + 1
    }
    fn ee_sub_1f0598_link(&mut self, rec: u32) {
        self.calls.push((0x1f0598, rec as i64, 0));
    }
    fn ee_sub_1f0380_link(&mut self, sm: u32, obj: u32, a: i32, b: u32) -> u32 {
        self.calls.push((0x1f0380, sm as i64, b as i64));
        assert_eq!((obj, a), (8, -1));
        77
    }
    fn ee_sub_132b10(&mut self, _obj: u32, class: i32) -> u32 {
        (class == self.isa) as u32
    }
    fn ee_sub_19cd38_po(&mut self, obj: usize, s: i32) {
        self.calls.push((0x19cd38, obj as i64, s as i64));
    }
    fn ee_sub_12f9b8(&mut self, _a: u32, _rate: f32, anim: u32, _obj: u32, flags: u32) -> u32 {
        self.calls.push((0x12f9b8, anim as i64, flags as i64));
        55
    }
    fn vcall(&mut self, obj: u32, off: u32) {
        self.calls.push((off, obj as i64, 0));
    }
}

/// 55e: slots 82 / 83 (+0x2a0 per-kind counters, +0x12f4 nesting -> pose 0x60b / back to index 0)
#[test]
fn k17_c55e_slots_82_83() {
    let (mut c, _) = inited();
    // kind 0 and kind 3 (oid -1: nothing found)
    let mut r = Rec { isa: 0x6d, ..Default::default() };
    c.ee_sub_1442d0(7, 0b1001, true, &mut r);
    assert_eq!((c.m.i(0x2a0), c.m.i(0x2ac), c.m.i(0x12f4)), (1, 1, 1));
    assert_eq!(
        r.calls,
        vec![
            (0x15e938, 4, 0x60c),
            (0x1f03c8, 1000 + 0x60c, 8),
            (0x1f01b8, 1000 + 0x60c, -100 + 9),
            (0x15e938, 4, -1),
            // FUN_00143ee0(self, 0x60b, 9, 5): anim (is-a 0x6d) -> index 5 = state 10, +0x1034 stopped, anim played
            (0x15e938, 1, 0x60b),
            (0x135248, 7, 10),
            (0x12f9b8, 1000 + 0x60b, 9),
            (0xc, 55, 0),
            (0x19cd38, 7, -1),
        ]
    );
    assert_eq!(c.m.u(0x1030), 55);
    // nested enable: counter 2, no link, no pose
    let mut r = Rec::default();
    c.ee_sub_1442d0(7, 1, true, &mut r);
    assert_eq!((c.m.i(0x2a0), c.m.i(0x12f4)), (2, 2));
    assert!(r.calls.is_empty());
    // disable twice; the second removes the link and (in state 10) goes back to index 0
    c.state = 10;
    let mut r = Rec { have_link: true, ..Default::default() };
    c.ee_sub_144338(7, 1, true, &mut r);
    assert!(r.calls.is_empty());
    c.ee_sub_144338(7, 1, true, &mut r);
    assert_eq!(r.calls, vec![(0x15e938, 4, 0x60c), (0x1f03c8, 1000 + 0x60c, 8), (0x1f0598, 1001 + 0x60c, 0), (0x135248, 7, 0)]);
    assert_eq!((c.m.i(0x2a0), c.m.i(0x12f4)), (0, 0));
    // a zero counter is not decremented
    let mut r = Rec::default();
    c.ee_sub_144338(7, 1, false, &mut r);
    assert_eq!(c.m.i(0x2a0), 0);
    assert!(r.calls.is_empty());
    // state-machine pose (is-a 0x6f): anim slot dropped, link record into +0x1034
    let mut r = Rec { isa: 0x6f, ..Default::default() };
    c.m.set_u(0x1034, 0);
    c.ee_sub_143ee0(7, 0x700, 3, 2, &mut r);
    assert_eq!(c.m.u(0x1034), 77);
    assert_eq!(r.calls[1], (0x135248, 7, 3));
    assert!(r.calls.contains(&(0x1f0380, 1000 + 0x700, 3)));
}

/// all levels: every class-55 object is built + inited, runs 300 frames with the engine host
#[test]
#[ignore = "heavy"]
fn k17_c55_all_levels_300_frames() {
    let (loader, descs) = levels();
    let mut n = 0;
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        let objs: Vec<usize> = w.objects.iter().enumerate().filter(|x| x.1.class == 55).map(|x| x.0).collect();
        assert_eq!(objs.len(), s.k16.len());
        for &i in &objs {
            let c = s.k16_get(i).unwrap();
            assert_eq!((c.state, c.f_1048, c.m.f(0x3d8), c.m.u(0x2c0)), (-1, 125.0, -70.0, i as u32 + 1));
        }
        if let Some(&last) = objs.last() {
            assert_eq!(s.g.g_2dd5bc, last as u32 + 1);
        }
        s.post_load(&mut h);
        for _ in 0..300 {
            s.g.dt = DT;
            s.g.t += DT;
            s.frame(&mut h);
        }
        n += objs.len();
    }
    assert_eq!(n, 42);
}

#[derive(Default)]
struct DrawRec(Vec<(usize, f32, f32, i32)>, u32);
impl ClassHost for DrawRec {
    fn ee_sub_1f3c10(&mut self, _obj: usize) {
        self.1 += 1;
    }
    fn ee_sub_168638_2(&mut self, obj: usize, a: f32, b: f32, k: i32) {
        self.0.push((obj, a, b, k));
    }
}

/// 55d: slot 18 FUN_001425b0 (fades +0x10d4 / +0x10d8, ALO draw hook)
#[test]
fn k17_c55d_draw() {
    let (mut c, mut g) = inited();
    g.dt = DT;
    // not in the draw mask
    let mut r = DrawRec::default();
    c.alo.so.f_2d = 0;
    g.g_2ddf54 = 1;
    c.ee_sub_1425b0(7, &g, &mut r);
    assert_eq!((r.1, r.0.len()), (0, 0));
    c.alo.so.f_2d = 1;
    c.alo.so.f_12c = 2.0;
    // attached as kind 2: alphas fade from 1 towards 0.15 / 0
    c.att_4c8 = 2;
    let mut last = (1.0, 1.0);
    for _ in 0..200 {
        c.ee_sub_1425b0(7, &g, &mut r);
        let (a, b) = (c.m.f(0x10d4), c.m.f(0x10d8));
        assert!(a <= last.0 && b <= last.1 && a >= 0.15 && b >= 0.0);
        last = (a, b);
    }
    assert_eq!(r.1, 200);
    assert_eq!(last, (0.15, 0.0));
    let e = *r.0.last().unwrap();
    assert_eq!((e.0, e.1, e.2), (7, 0.3, 0.0));
    // detached: back to 1
    c.att_4c8 = -1;
    for _ in 0..200 {
        c.ee_sub_1425b0(7, &g, &mut r);
    }
    assert_eq!((c.m.f(0x10d4), c.m.f(0x10d8)), (1.0, 1.0));
    // binoc state 1 (mode != 2): ALO pre-draw only
    c.binoc.state = 1;
    c.binoc.mode = 0;
    let n = r.0.len();
    c.ee_sub_1425b0(7, &g, &mut r);
    assert_eq!((r.0.len(), r.1), (n, 401));
}

#[derive(Default)]
struct CRec {
    calls: Vec<(u32, i64, i64)>,
    z: f32,
    focus: [f32; 3],
    vel: [f32; 3],
    lead: bool,
    in_world: bool,
    isa30: bool,
    words: Vec<(u32, u32, i32)>,
    pushes: Vec<([f32; 3], u32)>,
}
impl ClassHost for CRec {
    fn base(&mut self, addr: u32, obj: usize) {
        self.calls.push((addr, obj as i64, -1));
    }
    fn po_part(&mut self, addr: u32, obj: usize, off: u32) {
        self.calls.push((addr, obj as i64, off as i64));
    }
    fn po_part_rec(&mut self, addr: u32, obj: usize, off: u32, rec: &mut [u32; 16]) {
        self.calls.push((addr, obj as i64, off as i64));
        rec[0] |= 0x100;
    }
    fn ee_sub_1f58f0(&mut self, _obj: usize, rec: &mut [u32; 16]) {
        rec[13] = 100f32.to_bits();
    }
    fn ee_sub_1a62e8(&mut self, _obj: u32) -> [f32; 3] {
        [0.0, 0.0, self.z]
    }
    fn vcall_30c(&mut self, _obj: usize) -> [f32; 3] {
        self.focus
    }
    fn vcall_114(&mut self, _obj: usize) -> [f32; 3] {
        self.vel
    }
    fn g_2de1a0(&mut self) -> u32 {
        if self.lead { 8 } else { 0 }
    }
    fn ee_sub_203d00_bones(&mut self, obj: usize) {
        self.calls.push((0x203d00, obj as i64, 1));
    }
    fn lo_28_bit60(&mut self, _obj: u32) -> bool {
        self.in_world
    }
    fn ee_sub_205008(&mut self, obj: usize, other: u32, yaw: f32) {
        self.calls.push((0x205008, obj as i64, other as i64 * 1000 + yaw as i64));
    }
    fn ee_sub_2041c0(&mut self, obj: usize, v: [f32; 3]) {
        self.calls.push((0x2041c0, obj as i64, v[0] as i64));
    }
    fn ee_sub_1fad00(&mut self, _obj: usize, push: [f32; 3], side: u32) {
        self.pushes.push((push, side));
    }
    fn ee_sub_132b10(&mut self, _obj: u32, class: i32) -> u32 {
        (class == 0x30 && self.isa30) as u32
    }
    fn obj_i32(&mut self, obj: u32, off: u32) -> i32 {
        self.words.iter().find(|w| w.0 == obj && w.1 == off).map_or(0, |w| w.2)
    }
}

/// 55c: slot 89 FUN_00135d80 (base ASEG by state)
#[test]
fn k17_c55c_slot_89() {
    let (mut c, _) = inited();
    c.m.set_u(0x268, 0x500);
    let mut r = CRec::default();
    c.state = 0;
    c.ee_sub_135d80(7, &mut r);
    assert_eq!(c.m.u(0x25c), 0x500);
    // JMP: +0x2dc < 0xb -> +0x380, 0xb..0xd -> owner +0x520, 0x11 -> +0x384
    c.state = 2;
    c.m.set_u(0x380, 0x601);
    c.m.set_u(0x384, 0x602);
    c.m.set_u(0x520, 0x603);
    for (s, a) in [(0, 0x601), (0xc, 0x603), (0x11, 0x602), (0x12, 0x601)] {
        c.f_2dc = s;
        c.ee_sub_135d80(7, &mut r);
        assert_eq!(c.m.u(0x25c), a, "jmp {s}");
    }
    // attach kind 8 sub 9: +0x524 with the bone flag
    c.state = 3;
    c.att_4c8 = 8;
    c.att_4cc = 9;
    c.m.set_u(0x524, 0x700);
    c.ee_sub_135d80(7, &mut r);
    assert_eq!(c.m.u(0x25c), 0x700);
    assert_eq!(r.calls, vec![(0x203d00, 7, 1)]);
    // attach kind 0 sub 5 -> +0x520; kind 8 sub 11 -> none -> +0x268
    c.att_4c8 = 0;
    c.att_4cc = 5;
    c.ee_sub_135d80(7, &mut r);
    assert_eq!(c.m.u(0x25c), 0x603);
    c.att_4c8 = 8;
    c.att_4cc = 11;
    c.ee_sub_135d80(7, &mut r);
    assert_eq!(c.m.u(0x25c), 0x500);
    // damage state -1: default
    c.state = 4;
    c.ee_sub_135d80(7, &mut r);
    assert_eq!(c.m.u(0x25c), 0x500);
    assert_eq!(r.calls.len(), 1);
}

/// 55c: slot 49 FUN_001358a8 / FUN_00141640 (camera target)
#[test]
fn k17_c55c_slot_49_camera() {
    let (mut c, mut g) = inited();
    g.dt = DT;
    g.t = 12.5;
    let f = |x: u32| f32::from_bits(x);
    // not the lead object: height = focus z clamped to [z - 275, z + 150] (+0x434 == 0)
    let mut r = CRec { z: 100.0, focus: [10.0, 20.0, 1000.0], ..Default::default() };
    let mut rec = [0u32; 16];
    c.state = 2;
    c.att_4cc = 0;
    c.ee_sub_1358a8(7, &mut rec, &g, &mut r);
    assert_eq!(c.m.f(0x1080), 0.0);
    assert_eq!((f(rec[4]), f(rec[5]), f(rec[6])), (10.0, 20.0, 250.0));
    assert_eq!((c.f_104c, c.m.f(0x1050)), (250.0, 250.0));
    assert_eq!(c.m.v3(0x1060), [10.0, 20.0, 250.0]);
    // speed 0 -> scale clamp(1.5) = 1
    assert_eq!(f(rec[13]), 100.0);
    // below the window
    r.focus[2] = -1000.0;
    c.ee_sub_1358a8(7, &mut rec, &g, &mut r);
    assert_eq!(f(rec[6]), -175.0);
    // the lead object (DAT_002de1a0 == self) smooths the height towards +0x104c
    r.lead = true;
    r.focus[2] = 200.0;
    c.att_4cc = -1;
    c.m.set_f(0x1050, 0.0);
    let mut rec = [0u32; 16];
    c.ee_sub_1358a8(7, &mut rec, &g, &mut r);
    assert_eq!(c.m.f(0x1080), 12.5);
    assert_eq!(c.f_104c, 200.0);
    let z1 = f(rec[6]);
    assert!(z1 > 0.0 && z1 < 200.0, "{z1}");
    // speed 2 -> scale 0.5; damage state -1, JMP +0x28 == 0: flags 0
    r.vel = [2.0, 0.0, 0.0];
    let mut rec = [0u32; 16];
    c.ee_sub_1358a8(7, &mut rec, &g, &mut r);
    assert_eq!((f(rec[13]), rec[0]), (50.0, 0));
    // state 5 -> flags |= 1; JMP +0x28 -> |= 1; attach -> FUN_0018a4e8 part; state 0xd -> FUN_0019ea00 part
    c.state = 0xd;
    c.att_4cc = 2;
    r.calls.clear();
    let mut rec = [0u32; 16];
    c.ee_sub_1358a8(7, &mut rec, &g, &mut r);
    assert_eq!(r.calls, vec![(0x18a4e8, 7, 0x4c0), (0x19ea00, 7, 0xf50)]);
    // +0x1044 set: the stored +0x1060 position is used, +0x1070 offset decays towards 0
    c.m.set_i(0x1044, 1);
    c.m.set_v3(0x1060, [1.0, 2.0, 3.0]);
    c.m.set_v3(0x1070, [100.0, 0.0, 0.0]);
    let mut rec = [0u32; 16];
    c.ee_sub_1358a8(7, &mut rec, &g, &mut r);
    let ox = c.m.f(0x1070);
    assert!(ox < 100.0 && ox > 0.0);
    assert_eq!((f(rec[4]), f(rec[6]), c.f_104c), (1.0 + ox, 3.0, 3.0));
}

/// 55c: base slots 57 / 58 / 67 / 81 / 28 / 60
#[test]
fn k17_c55c_base_slots() {
    let (mut c, g) = inited();
    let mut r = CRec::default();
    // 57 / 58 with no active components: base only
    c.ee_sub_140d70(7, &mut r);
    c.ee_sub_140d00(7, &mut r);
    assert_eq!(r.calls, vec![(0x204768, 7, -1), (0x1f4fa8, 7, -1)]);
    r.calls.clear();
    c.f_2dc = 3;
    c.att_4cc = 1;
    c.f_f54 = 0;
    c.ee_sub_140d70(7, &mut r);
    c.ee_sub_140d00(7, &mut r);
    assert_eq!(r.calls, vec![(0x204768, 7, -1), (0x199298, 7, 0x2c0), (0x1f4fa8, 7, -1), (0x1991a0, 7, 0x2c0), (0x193de8, 7, 0x4c0), (0x19e950, 7, 0xf50)]);
    // 67: push from Bentley (DAT_002dd5bc) scaled by 0.01 unless world +0x150 +0x80 == 1 (host default 0)
    c.ee_sub_140db0(7, [100.0, 0.0, -50.0], g.g_2dd5bc, 1, &g, &mut r);
    c.ee_sub_140db0(7, [100.0, 0.0, -50.0], 99, 0, &g, &mut r);
    assert_eq!(r.pushes, vec![([1.0, 0.0, -0.5], 1), ([100.0, 0.0, -50.0], 0)]);
    // 81: not in the world -> JMP start; in the world -> PO attach with yaw - +0x1010
    r.calls.clear();
    c.ee_sub_140c30(7, 50, 3.0, &mut r);
    r.in_world = true;
    c.m.set_f(0x1010, 1.0);
    c.ee_sub_140c30(7, 50, 3.0, &mut r);
    // 28: translate + clear +0x9e4
    c.m.set_u(0x9e4, 5);
    c.ee_sub_143cf8(7, [4.0, 0.0, 0.0], &mut r);
    assert_eq!(r.calls, vec![(0x195a00, 7, 0x2c0), (0x205008, 7, 50002), (0x2041c0, 7, 4)]);
    assert_eq!(c.m.u(0x9e4), 0);
    // 60: +0x11b0 block (target +0x11c0, kind +0x11b4 > 2) and +0x900 block (same +0xec) raise to 3
    let mut r = CRec::default();
    assert_eq!(c.ee_sub_144388(7, 60, &g, &mut r), 0);
    c.m.set_u(0x11c0, 60);
    c.m.set_i(0x11b4, 2);
    assert_eq!(c.ee_sub_144388(7, 60, &g, &mut r), 0);
    c.m.set_i(0x11b4, 3);
    assert_eq!(c.ee_sub_144388(7, 60, &g, &mut r), 3);
    c.m.set_i(0x11b4, -1);
    c.m.set_i(0x9a4, 1);
    c.m.set_u(0x984, 70);
    r.words = vec![(70, 0x10, 71), (71, 0xec, 5), (60, 0xec, 5)];
    assert_eq!(c.ee_sub_144388(7, 60, &g, &mut r), 3);
    r.isa30 = true;
    assert_eq!(c.ee_sub_144388(7, 60, &g, &mut r), 0);
}

#[derive(Default)]
struct SRec {
    calls: Vec<(u32, i64, i64)>,
    phase: f32,
    len: f32,
    label_ok: bool,
}
impl ClassHost for SRec {
    fn po_part_s(&mut self, addr: u32, obj: usize, off: u32, s: i32) {
        self.calls.push((addr, obj as i64 * 0x10000 + off as i64, s as i64));
    }
    fn ee_sub_2303a0_off(&mut self, _obj: u32, off: u32, k: i32) {
        self.calls.push((0x2303a0, off as i64, k as i64));
    }
    fn ee_sub_12f9b8(&mut self, a: u32, rate: f32, anim: u32, _obj: u32, flags: u32) -> u32 {
        self.calls.push((0x12f9b8, anim as i64, ((f32::from_bits(a) * 10.0) as i64) * 10000 + (rate * 10.0) as i64 * 100 + flags as i64));
        anim + 1
    }
    fn ee_sub_1f03a8_link(&mut self, sm: u32, _obj: u32, label: i32, flags: u32, mask: i32) -> u32 {
        self.calls.push((0x1f03a8, sm as i64, label as i64 * 1_000_000 + flags as i64 * 10000 + mask as i64));
        sm + 1
    }
    fn ee_sub_1f0400(&mut self, _sm: u32, _label: i32) -> i32 {
        if self.label_ok { 1 } else { -1 }
    }
    fn anim_slot_phase(&mut self, _slot: u32) -> f32 {
        self.phase
    }
    fn anim_74(&mut self, _anim: u32) -> f32 {
        self.len
    }
    fn ee_sub_19cd38_po(&mut self, obj: usize, s: i32) {
        self.calls.push((0x19cd38, obj as i64, s as i64));
    }
    fn ee_sub_136530(&mut self, obj: usize, s: i32) {
        self.calls.push((0x136530, obj as i64, s as i64));
    }
    fn vcall(&mut self, obj: u32, off: u32) {
        if off == 0x2d4 {
            self.calls.push((0x2d4, obj as i64, 0));
        }
    }
}

/// 55h: SetState FUN_00135248 (exit / enter by state, anim + state-machine slots, idle timer)
#[test]
fn k17_c55h_set_state() {
    use sly2_recomp::engine::physics::driven::GameRng;
    let (mut c, mut g) = inited();
    let mut rng = GameRng::default();
    g.t = 3.0;
    // -1 -> 0: idle state machine +0x1028 label 0x175 (found), mask 8 (+0x10fc 0), timer parked at FLT_MAX
    c.m.set_u(0x1028, 0x900);
    let mut r = SRec { label_ok: true, ..Default::default() };
    c.ee_sub_135248(7, 0, &g, &mut rng, &mut r);
    assert_eq!((c.state, c.m.f(0x13d4), c.m.i(0x1148), c.m.f(0x114c)), (0, 3.0, 1, f32::MAX));
    assert_eq!(r.calls, vec![(0x2303a0, 0x58, 1), (0x2303a0, 0x5c, 1), (0x1f03a8, 0x900, 0x175 * 1_000_000 + 10000 + 8), (0x2d4, 8, 0)]);
    assert_eq!(c.m.u(0x1034), 0x901);
    // same state: nothing
    let mut r = SRec::default();
    c.ee_sub_135248(7, 0, &g, &mut rng, &mut r);
    assert!(r.calls.is_empty());
    // 0 -> 7 (+0x1600 block anim, start = +0x1a0 * len): idle timer = t + rand(+0x1138, +0x113c) = 3 + [1.5, 2.5]
    c.m.set_u(0x1604, 0x40);
    c.m.set_f(0x1a0, 0.5);
    c.m.set_u(0x1600 + 0x9c, 9);
    let mut r = SRec { len: 2.0, ..Default::default() };
    c.ee_sub_135248(7, 7, &g, &mut rng, &mut r);
    let t = c.m.f(0x114c);
    assert!((4.5..=5.5).contains(&t), "{t}");
    assert_eq!(c.m.u(0x1600 + 0x9c), 0);
    assert_eq!(&r.calls[2..], &[(0x12f9b8, 0x40, 10 * 10000 + 1000 + 9), (0x2d4, 8, 0)]);
    assert_eq!(c.m.u(0x1030), 0x41);
    // 7 -> 2: +0x1a0 = phase of +0x1030; 2 has no enter call (the JMP part runs it)
    let mut r = SRec { phase: 0.25, ..Default::default() };
    c.ee_sub_135248(7, 2, &g, &mut rng, &mut r);
    assert_eq!((c.state, c.m.f(0x1a0)), (2, 0.25));
    // 2 -> 4: JMP told -1, damage enter = binoc -1
    let mut r = SRec::default();
    c.ee_sub_135248(7, 4, &g, &mut rng, &mut r);
    assert_eq!(&r.calls[..2], &[(0x195fb8, 7 * 0x10000 + 0x2c0, -1), (0x19cd38, 7, -1)]);
    // 4 -> 0xe: damage told -1; anim +0x1878 missing -> +0x1374 at rate 1.5
    c.m.set_u(0x1374, 0x50);
    let mut r = SRec::default();
    c.ee_sub_135248(7, 0xe, &g, &mut rng, &mut r);
    assert_eq!(r.calls[0], (0x1a0b40, 7 * 0x10000 + 0xdf0, -1));
    assert!(r.calls.contains(&(0x12f9b8, 0x50, 1500 + 9)));
    // -> 6: state-6 sub-state 0 (+0x14e4)
    c.m.set_u(0x14e4, 0x60);
    let mut r = SRec::default();
    c.ee_sub_135248(7, 6, &g, &mut rng, &mut r);
    assert_eq!((c.state, c.f_16b0), (6, 0));
    assert_eq!(r.calls, vec![(0x12f9b8, 0x60, 1000 + 1), (0x2d4, 8, 0)]);
    // 6 -> 0xd: sub-state -1, path part enter 0, no anim
    let mut r = SRec::default();
    c.ee_sub_135248(7, 0xd, &g, &mut rng, &mut r);
    assert_eq!(c.f_16b0, -1);
    assert_eq!(*r.calls.last().unwrap(), (0x19eab0, 7 * 0x10000 + 0xf50, 0));
    // 0xd -> 0xc: path part -1, 0xc plays nothing
    let mut r = SRec::default();
    c.ee_sub_135248(7, 0xc, &g, &mut rng, &mut r);
    assert_eq!(r.calls, vec![(0x19eab0, 7 * 0x10000 + 0xf50, -1)]);
    assert_eq!(c.state, 0xc);
}

/// real level: a Bentley instance is built + inited, runs frames, reaches states via SetState
#[test]
fn k17_c55_real_instance_set_state() {
    let (loader, descs) = levels();
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let Some(obj) = w.objects.iter().position(|o| o.class == 55) else { continue };
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        s.post_load(&mut h);
        // K18: load-post slot 14 FUN_001347a8 enters state 0
        assert_eq!(s.k16_get(obj).unwrap().state, 0);
        for i in 0..120 {
            s.g.dt = DT;
            s.g.t += DT;
            s.frame(&mut h);
            if i == 30 {
                assert!(s.k17_set_state(obj, 0, &mut h));
            }
            if i == 60 {
                assert!(s.k17_set_state(obj, 5, &mut h));
            }
        }
        let c = s.k16_get(obj).unwrap();
        assert_eq!(c.state, 5);
        assert!((c.m.f(0x13d4) - 61.0 * DT).abs() < 1e-4);
        // the idle timer was started when leaving state 0
        assert!(c.m.f(0x114c) > 61.0 * DT && c.m.f(0x114c) < 1e30);
        assert!(!s.k17_set_state(0, 0, &mut h) || w.objects[0].class == 55);
        return;
    }
    panic!("no class-55 object");
}
