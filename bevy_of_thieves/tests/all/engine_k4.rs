//! WP-K4 tests: K4 classes (ClassSet::k4) fed from level data + ClassesPlugin over all 43 levels. Needs the ISO at
//! "<root>/Sly 2 - Band of Thieves (USA).iso" (level list from the boot level table).
use sly2_recomp::engine::disc::descriptor::Descriptor;
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::world::{LoadOptions, World};
use std::path::PathBuf;
use std::sync::Arc;

fn real_root() -> PathBuf {
    let r = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    if r.join("Sly 2 - Band of Thieves (USA).iso").exists() { r } else { PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/..")) }
}

/// The 43 level descriptors (db disc_files + level_table, as in engine_a).
fn levels() -> (LevelLoader, Vec<Descriptor>) {
    crate::common::levels() // public: ISO level table instead of the analysis DB
}


use sly2_recomp::engine::classes::k4::PORTED_K4;
use sly2_recomp::engine::classes::plugin::{Classes, ClassesLevel, ClassesPlugin, EngineHost};
use sly2_recomp::engine::classes::{ClassGlobals, ClassHost, Inst};
use std::collections::BTreeMap;

#[derive(Default, Debug)]
struct Stats {
    n: BTreeMap<u16, usize>,
    fed: BTreeMap<u16, usize>,
    host: Vec<EngineHost>,
}

/// Per-instance "fed from level data / phases ran" predicate.
fn fed(x: &Inst) -> bool {
    match x {
        // class 105: slot 14 sized the ring from +0x150 / +0x158 and slot 16 wrote it
        Inst::C2db0d0(k) => k.f_158 >= 2 && k.f_15c.len() == k.f_158 as usize,
        // class 15: slot 6 ran (+0x164 = 7, +0x40 bit 1); slot 15 queues FUN_00221298 unless +0x13c == 0
        Inst::C2db6b8(k) => k.f_40 & 2 != 0 && k.f_164 == 7,
        // class 132: slot 6 ran, +0x68 group object resolved from field 1244
        Inst::C2cf378(k) => k.f_b4 == -1 && k.f_68 != 0,
        // class 7: slot 14 numbered it (DAT_003e1bf0 serial)
        Inst::C2d5dc8(k) => k.f_100 >= 0,
        // class 13: slot 6 (+ class-10 init) ran, slot 14 fixed the radius
        Inst::C2d56f8(k) => k.sub.base.f_178 == 1 && k.sub.f_1ac == 1 && k.sub.f_38 != 1e10,
        // class 113: its load-time method calls built the +0x50 path list
        Inst::C2cf880(k) => !k.f_50.is_empty(),
        // class 129: its method calls listed at least one object
        Inst::C2dad90(k) => !(k.f_110.is_empty() && k.f_194.is_empty()),
        // class 116: slot 6 ran
        Inst::C2cf058(k) => k.f_40 & 10 == 10,
        Inst::C2d52b0(k) => k.f_124 == -1,
        Inst::C2dd090(_) => true,
        Inst::C2ce2f8(k) => k.f_40 & 0x40 != 0,
        Inst::C2d9dd0(k) => k.f_38 != 1e10,
        Inst::C2ca9f0(k) => k.f_124 == 1,
        Inst::C2d6fe8(k) => k.base.f_178 == 1 && k.f_1ac == 7 && k.f_38 != 1e10,
        _ => false,
    }
}

fn run_all() -> Stats {
    let (loader, descs) = levels();
    let mut st = Stats::default();
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let mut app = bevy_app::App::new();
        app.add_plugins(ClassesPlugin);
        app.world_mut().resource_mut::<ClassesLevel>().0 = Some(w.clone());
        for _ in 0..120 {
            app.update();
        }
        let c = app.world().resource::<Classes>();
        assert_eq!(c.frames, 120);
        let expect = w.objects.iter().filter(|o| PORTED_K4.iter().any(|p| p.0 == o.class)).count();
        assert_eq!(c.set.k4.len(), expect, "lsn {}", d.lsn());
        for (i, x) in &c.set.k4 {
            let cls = w.objects[*i].class;
            *st.n.entry(cls).or_default() += 1;
            if fed(x) {
                *st.fed.entry(cls).or_default() += 1;
            }
        }
        st.host.push(c.host.clone());
    }
    st
}

#[test]
#[ignore = "heavy"]
fn k4_all_levels_build_init_120_frames() {
    let st = run_all();
    eprintln!("K4 n {:?}\nK4 fed {:?}", st.n, st.fed);
    for (cls, _) in PORTED_K4 {
        assert!(st.n.get(cls).copied().unwrap_or(0) > 0, "class {cls} has instances");
    }
    assert_eq!(st.fed.get(&105), st.n.get(&105), "class 105 ring");
    assert_eq!(st.fed.get(&15), st.n.get(&15), "class 15 init");
    assert_eq!(st.fed.get(&7), st.n.get(&7), "class 7 serial");
    assert_eq!(st.fed.get(&13), st.n.get(&13), "class 13 init");
    assert_eq!(st.fed.get(&11), st.n.get(&11), "class 11 init");
    // 9 of 915 class-113 objects carry no method calls
    assert!(st.fed[&113] * 100 > st.n[&113] * 98, "class 113 path list");
    assert_eq!(st.fed.get(&129), st.n.get(&129), "class 129 lists");
    assert_eq!(st.fed.get(&116), st.n.get(&116), "class 116 init");
    for c in [17, 3, 117, 39, 44] {
        assert_eq!(st.fed.get(&c), st.n.get(&c), "class {c} init");
    }
    assert!(st.fed.get(&132).copied().unwrap_or(0) * 2 > st.n[&132], "class 132 group refs");
}

#[test]
fn c2db0d0_trail() {
    use sly2_recomp::engine::anim::pose::{Mat4, IDENTITY};
    use sly2_recomp::engine::classes::c2db0d0::{C2db0d0, Group105, Sub105};
    struct H(f32);
    impl ClassHost for H {
        fn vcall_fc(&mut self, _o: u32) -> Mat4 {
            let mut m = IDENTITY;
            m[3][0] = self.0;
            m
        }
        fn mesh_dc(&mut self, _o: u32) -> Vec<Group105> {
            let verts = vec![[0.0, 0.0, 0.0, 1.0], [10.0, 0.0, 0.0, 1.0]];
            vec![Group105 { idx: 0, subs: vec![Sub105 { n: 2, verts, f_14: Some(vec![0xff00_0000; 2]), ..Default::default() }] }]
        }
    }
    let mut h = H(0.0);
    let mut g = ClassGlobals { dt: 1.0 / 60.0, t: 1.0, ..Default::default() };
    let mut c = C2db0d0::default();
    c.ee_sub_21bb88(0, &mut h);
    assert_eq!((c.f_170, c.f_184), ([1.0, 0.0, 0.0, 0.0], 100.0));
    c.f_168 = 2.0;
    c.f_12c = 1.0;
    c.ee_sub_21bd18(0, &g, &mut h);
    // +0x150 0 -> 0.5 s -> 32 entries; phase 0..1 along the axis; stretch 2 along x
    assert_eq!((c.f_150, c.f_158, c.f_15c.len(), c.f_40 & 2), (0.5, 32, 32, 2));
    let r = c.groups[0].subs[0].recs[1];
    assert_eq!((f32::from_bits(r[0]), f32::from_bits(r[4])), (20.0, 1.0));
    // the object moves +1/frame: vertex 0 (phase 0) follows the newest entry, vertex 1 (phase 1) lags
    for k in 1..=40 {
        h.0 = k as f32;
        g.t += g.dt;
        c.ee_sub_21c0d8(0, &g, &mut h);
        c.ee_sub_21c1c8(0, &g, &mut h);
        c.ee_sub_21c340(0, &g, &mut h);
    }
    let v = &c.groups[0].subs[0].verts;
    assert_eq!(v[0][0], 40.0);
    assert_eq!(v[1][0], 20.0 + 10.0);
    // alpha fade on the colour stream: newest entry full alpha
    assert_eq!(c.groups[0].subs[0].f_14.as_ref().unwrap()[0] >> 24, 0xff);
}

#[test]
fn c2db6b8_volume_state() {
    use sly2_recomp::engine::classes::c2db6b8::{C2db6b8, PMF_221298};
    #[derive(Default)]
    struct H(bool, Vec<(i32, i32)>, usize);
    impl ClassHost for H {
        fn ee_sub_1c4a48(&mut self) -> u32 {
            9
        }
        fn ee_sub_118fb0_at(&mut self, _o: usize, off: u32, _p: [f32; 3]) -> bool {
            assert_eq!(off, 0x108);
            self.0
        }
        fn vcall_1c(&mut self, _o: usize, a: i32, b: i32) {
            self.1.push((a, b));
        }
        fn ee_sub_14e328(&mut self, k: i32, _d: u32, t: u32, _o: u32) {
            assert_eq!((k, t), (5, 9));
            self.2 += 1;
        }
    }
    let mut h = H::default();
    let g = ClassGlobals { t: 2.0, ..Default::default() };
    let mut c = C2db6b8::default();
    c.ee_sub_220ed0(0, &mut h);
    c.f_128 = 3;
    let mut q = Vec::new();
    c.ee_sub_221388(0, &mut q, &mut h);
    assert_eq!(q, vec![(0, PMF_221298)]);
    // outside: new focus object -> state 0 already, nothing fired
    c.ee_sub_221298(0, &g, &mut h);
    assert_eq!((c.f_e0, c.f_134, h.1.len()), (9, 0, 0));
    // inside -> state 1 (camera request + vtable +0x1c (5, 1)), then outside -> 0 (6, 1)
    h.0 = true;
    c.ee_sub_221298(0, &g, &mut h);
    assert_eq!((c.f_134, c.f_138, h.2), (1, 2.0, 1));
    h.0 = false;
    c.ee_sub_221298(0, &g, &mut h);
    assert_eq!((c.f_134, h.1.clone()), (0, vec![(5, 1), (6, 1)]));
    // slot 11 drops the queued call
    c.ee_sub_220f88(0, &mut q, &mut h);
    assert!(q.is_empty());
}

#[test]
fn c2cf378_states() {
    use sly2_recomp::engine::classes::c2cf378::{C133View, C2cf378};
    #[derive(Default)]
    struct H(bool, Vec<String>);
    impl ClassHost for H {
        fn ee_sub_2121e8(&mut self, seq: u32, id: i32) -> bool {
            self.1.push(format!("run {seq} {id}"));
            self.0
        }
        fn ee_sub_213b00(&mut self, seq: u32, id: i32) {
            self.1.push(format!("start {seq} {id}"));
        }
        fn ee_sub_164640(&mut self, obj: u32, k: i32) {
            self.1.push(format!("164640 {obj} {k}"));
        }
        fn ee_sub_15e938(&mut self, kind: i32, oid: i32, _c: u32) -> u32 {
            if kind == 0x405 { oid as u32 } else { 0 }
        }
    }
    let views = |o: u32| match o {
        // group 50: members 60 (+0xc4 unset), 61 (+0xc4 set); 61's +0x98 list: 50 (skipped), 62 (state 2 -> ok)
        50 => Some(C133View { f_a4: vec![60, 61], ..Default::default() }),
        60 => Some(C133View::default()),
        61 => Some(C133View { f_c4: 7, f_98: vec![50, 62], ..Default::default() }),
        62 => Some(C133View { f_54: 2, ..Default::default() }),
        _ => None,
    };
    let mut h = H::default();
    let mut g = ClassGlobals { t: 3.0, ..Default::default() };
    let mut c = C2cf378::default();
    c.ee_sub_162448(0, &mut h);
    assert_eq!((c.f_9c, c.f_a0, c.f_b4, c.f_ac), (480.0, 720.0, -1, -1));
    c.f_ac = 11;
    c.f_68 = 50;
    c.f_b0 = 4;
    c.f_b4 = 5;
    c.ee_sub_162780(0, &g, &mut h);
    assert_eq!((c.f_b8, c.f_bc), (11, 0x379));
    // 0 -> 1: picks group member 61, starts sequence +0xb0
    c.ee_sub_163170(0, 1, &g, &views, &mut h);
    assert_eq!((c.f_c0, c.f_c4, c.f_c8, c.f_40), (61, 1, 3.0, 0x102));
    // sequence done -> state 3 (member chosen) -> +0xbc 0x30f done -> hand over to member 61
    c.ee_sub_163078(0, &mut g, &views, &mut h);
    assert_eq!(c.f_c4, 3);
    c.ee_sub_163078(0, &mut g, &views, &mut h);
    assert_eq!(h.1, vec!["start 11 4", "run 11 4", "run 889 783", "164640 61 4"]);
}

#[test]
fn c2d5dc8_attach_host() {
    use sly2_recomp::engine::classes::c2d5dc8::{C2d5dc8, PMF_2A77C0};
    #[derive(Default)]
    struct H(Vec<String>);
    impl ClassHost for H {
        fn ee_sub_1c7bb8(&mut self, s: i32) -> bool {
            s == 1
        }
        fn vcall(&mut self, o: u32, off: u32) {
            self.0.push(format!("vcall {o} {off:#x}"));
        }
        fn ee_sub_149ab8(&mut self, o: usize, pmf: u32, _a: i32) {
            assert_eq!(pmf, PMF_2A77C0);
            self.0.push(format!("defer {o}"));
        }
        fn ee_sub_1caa60(&mut self, _o: usize, x: u32) {
            self.0.push(format!("detach {x}"));
        }
        fn ee_sub_127e90(&mut self, r: u32, o: u32) -> u32 {
            r * 100 + o
        }
    }
    let mut h = H::default();
    let mut g = ClassGlobals { g_3e1bf0: 0, ..Default::default() };
    let mut a = C2d5dc8::default();
    let mut b = C2d5dc8::default();
    a.ee_sub_1c8030(0, &mut g, &mut h);
    b.ee_sub_1c8030(1, &mut g, &mut h);
    assert_eq!((a.f_100, b.f_100, g.g_3e1bf0), (0, 1, 2));
    b.ee_sub_1ca8e0(1, &mut h);
    b.f_e4 = true;
    b.ee_sub_1ca8e0(1, &mut h);
    b.f_e8 = vec![7, 8];
    b.ee_sub_1cb6d8(1, &mut h);
    assert!(b.f_e8.is_empty());
    assert_eq!(h.0, vec!["vcall 2 0x4c", "defer 1", "detach 7", "detach 8"]);
    // clone sharing the table: copied and rebound
    b.f_f0 = vec![[0; 6], [0, 0, 3, 0, 0, 0]];
    b.ee_sub_1ca7b8(1, 0, &mut h);
    assert_eq!((b.f_f0[0][2], b.f_f0[1][2], b.f_f0_id), (0, 302, 0x8000_0002));
}

#[test]
fn c2d6fe8_radius_list() {
    use sly2_recomp::engine::classes::c2d6fe8::C2d6fe8;
    struct H;
    impl ClassHost for H {
        fn ee_sub_1e7d90(&mut self, _o: usize) -> f32 {
            50.0
        }
    }
    let mut g = ClassGlobals::default();
    let mut c = C2d6fe8::default();
    c.ee_sub_1cb908(3, &mut H);
    assert_eq!((c.f_1c4, c.f_1c0, c.f_1a4, c.f_1ac, c.base.f_148), (1.0, 1000.0, -1, 7, -1));
    c.ee_sub_1cb950(3, &mut H);
    assert_eq!(c.f_38, 1300.0);
    c.ee_sub_1cb9b0(3, &mut g, &mut H);
    assert_eq!(g.l_3e13cc, vec![4]);
    c.ee_sub_1cb9e8(3, &mut g, &mut H);
    assert!(g.l_3e13cc.is_empty());
    c.f_1b4 = 9;
    c.base.f_17c = 5;
    c.ee_sub_1cba20(3, &mut H);
    assert_eq!((c.f_1b4, c.base.f_17c), (9, 0));
}

#[test]
fn c2d56f8_init_list() {
    use sly2_recomp::engine::classes::c2d56f8::C2d56f8;
    struct H;
    impl ClassHost for H {}
    let mut g = ClassGlobals::default();
    let mut c = C2d56f8::default();
    c.ee_sub_1c3390(0, &mut H);
    assert_eq!((c.sub.f_1ac, c.f_1c0, c.f_1d0, c.f_1d4, c.sub.f_1a4), (1, [1.0, 0.0, 0.0, 0.0], 8, 9, -1));
    c.ee_sub_1c3458(0, &mut H);
    assert_eq!(c.sub.f_38, 1250.0);
    c.ee_sub_1c33e8(0, &mut g, &mut H);
    c.ee_sub_1c3420(0, &mut g, &mut H);
    assert!(g.l_3e13c0.is_empty());
}

#[test]
fn c2cf880_path_walk() {
    use sly2_recomp::engine::classes::c2cf880::{C2cf880, Entry};
    #[derive(Default)]
    struct H {
        calls: Vec<(i32, i32, u32)>,
        bad: Vec<u32>,
    }
    impl ClassHost for H {
        fn ee_sub_15e938(&mut self, kind: i32, oid: i32, ctx: u32) -> u32 {
            self.calls.push((kind, oid, ctx));
            oid as u32 + 100
        }
        fn ee_sub_132b10(&mut self, obj: u32, class: i32) -> u32 {
            (obj == 107 && class == 0xb) as u32
        }
        fn ee_sub_1e9c30(&mut self, obj: u32) {
            self.bad.push(obj);
        }
    }
    let mut h = H::default();
    let mut c = C2cf880::default();
    c.ee_sub_1709f8(2, &mut h);
    c.ee_sub_170e00(0, 5);
    c.ee_sub_170e00(6, 7);
    c.ee_sub_170e00(6, 8);
    c.ee_sub_170e00(3, 9);
    assert_eq!(c.f_50[1], Entry { kind: 6, id: 7, ..Default::default() });
    c.ee_sub_170a28(2, &mut h);
    assert_eq!(h.calls, vec![(4, 5, 3), (4, 7, 105), (4, 8, 105)]);
    assert_eq!(h.bad, vec![108]);
}

#[test]
fn c2dad90_counts() {
    use sly2_recomp::engine::classes::c2cf378::C133View;
    use sly2_recomp::engine::classes::c2dad90::C2dad90;
    #[derive(Default)]
    struct H {
        v: Vec<(u32, u32, i32)>,
        txt: Vec<(u32, i32)>,
    }
    impl ClassHost for H {
        fn ee_sub_15e938(&mut self, _k: i32, oid: i32, _c: u32) -> u32 {
            oid as u32
        }
        fn ee_sub_2121e8(&mut self, _s: u32, id: i32) -> bool {
            id == 0x314
        }
        fn ee_sub_21ecd8(&mut self, t: u32, v: i32) {
            self.txt.push((t, v));
        }
        fn vcall(&mut self, o: u32, off: u32) {
            self.v.push((o, off, 0));
        }
        fn vcall_arg(&mut self, o: u32, off: u32, a: i32) {
            self.v.push((o, off, a));
        }
    }
    let mut c = C2dad90::default();
    for k in 0..20 {
        c.ee_sub_211e98(10 + k);
    }
    assert_eq!(c.f_194.len(), 16);
    c.f_194.truncate(2);
    c.ee_sub_211e70(30);
    let mut h = H::default();
    c.ee_sub_211ec0(0, &mut h);
    assert_eq!((c.f_194[1], c.f_110[0]), ((11, 11), (30, 30)));
    // 10 done, 11 state 1, 30 pending -> fail (+0x18c)
    let view = |o: u32| -> Option<C133View> { Some(C133View { f_54: match o { 10 => 2, 11 => 1, _ => 0 }, ..Default::default() }) };
    c.ee_sub_211f90(0, &view, &mut h);
    assert_eq!(h.v, vec![(1, 0x18c, -1)]);
    assert_eq!(h.txt, vec![(0x2c3ec0, 1), (0x2c3ef0, 2), (0x2c3f20, 1)]);
    // all done -> +0x194
    let all = |_o: u32| -> Option<C133View> { Some(C133View { f_54: 3, ..Default::default() }) };
    c.ee_sub_211f90(0, &all, &mut h);
    assert_eq!(h.v[1], (1, 0x194, 0));
}

#[test]
fn c2cf058_fade() {
    use sly2_recomp::engine::classes::c2cf058::C2cf058;
    #[derive(Default)]
    struct H {
        v: Vec<u32>,
        d: Vec<(u32, i32)>,
    }
    impl ClassHost for H {
        fn vcall(&mut self, _o: u32, off: u32) {
            self.v.push(off);
        }
        fn ee_sub_15dd48_draw(&mut self, c: u32, k: i32) {
            self.d.push((c, k));
        }
    }
    let mut h = H::default();
    let mut g = ClassGlobals::default();
    let mut c = C2cf058 { f_50: 0x0000_0000, f_54: 0x80ff_ff00, f_58: 0, f_5c: 1.0, f_60: 1.0, f_64: 2.0, f_6c: 1, ..Default::default() };
    c.ee_sub_15dc80(0, &mut h);
    g.t = 10.0;
    c.ee_sub_15e1d8(0, &g, &mut h);
    c.ee_sub_15e1d8(0, &g, &mut h);
    assert_eq!((c.f_68, h.v.clone()), (10.0, vec![0x44]));
    g.t = 10.5;
    assert_eq!(c.colour(g.t), u32::from_le_bytes([0, 127, 127, 64]));
    assert_eq!(c.colour(11.5), 0x80ff_ff00);
    assert_eq!(c.colour(13.0), u32::from_le_bytes([0, 127, 127, 64]));
    c.ee_sub_15dd48(0, &g, &mut h);
    assert_eq!(h.d[0].1, 0xe);
    g.t = 14.5;
    c.ee_sub_15dcd8(0, &g, &mut h);
    assert!(!c.active);
    assert_eq!(h.v, vec![0x44, 0x4c]);
}

#[test]
fn small_k4_inits() {
    use sly2_recomp::engine::classes::{c2ce2f8, c2d52b0, c2dd090};
    struct H;
    impl ClassHost for H {}
    assert_eq!((c2dd090::ee_sub_113a40(), c2d52b0::ee_sub_113b20(), c2ce2f8::ee_sub_114160()), (0x2bab24, 0x2bac3c, 0x2bb40c));
    let mut a = c2d52b0::C2d52b0::default();
    a.ee_sub_1c0378(0, &mut H);
    assert_eq!((a.f_110, a.f_124), (-1, -1));
    let mut b = c2ce2f8::C2ce2f8::default();
    b.ee_sub_1564e8(0, &mut H);
    assert_eq!((b.f_54, b.f_58, b.f_60, b.f_40), (1500.0, [-200.0, 500.0], 0.25, 0x40));
}

#[test]
fn c2d9dd0_list() {
    use sly2_recomp::engine::classes::c2d9dd0::C2d9dd0;
    struct H;
    impl ClassHost for H {}
    let mut g = ClassGlobals::default();
    let mut c = C2d9dd0::default();
    c.ee_sub_1ff3b8(4, &mut H);
    assert_eq!((c.f_f4, c.f_fc), (-1, 7));
    c.ee_sub_1ff3e8(4, &mut H);
    assert_eq!(c.f_38, 1250.0);
    c.ee_sub_1ff438(4, &mut g, &mut H);
    assert_eq!(g.l_3e13d8, vec![5]);
    c.ee_sub_1ff470(4, &mut g, &mut H);
    assert!(g.l_3e13d8.is_empty());
    c.f_104 = 3;
    c.ee_sub_1ff4a8(4, &mut H);
    assert_eq!(c.f_104, 3);
}

#[test]
fn c2ca9f0_camera_request() {
    use sly2_recomp::engine::classes::c2ca9f0::C2ca9f0;
    #[derive(Default)]
    struct H(Vec<(char, i32, u32)>);
    impl ClassHost for H {
        fn ee_sub_14e328(&mut self, k: i32, _d: u32, _t: u32, o: u32) {
            self.0.push(('+', k, o));
        }
        fn ee_sub_14e4c8_k(&mut self, k: i32, a: i32, _d: u32, _b: i32, o: u32) {
            self.0.push(('-', k * 100 + a, o));
        }
    }
    let mut h = H::default();
    let mut c = C2ca9f0::default();
    c.ee_sub_12e600(1, &mut h);
    c.ee_sub_12e6a0(1, &mut h); // before load-post: no request
    c.ee_sub_12e648(1, &mut h);
    c.ee_sub_12e6e8(1, &mut h);
    c.ee_sub_12e740();
    c.ee_sub_12e740();
    c.ee_sub_12e750();
    assert_eq!(h.0, vec![('+', 10, 2), ('-', 1510, 2)]);
    assert_eq!((c.f_12c, c.f_110), (1, [0.0, 0.0, 1.0, 0.0]));
}
