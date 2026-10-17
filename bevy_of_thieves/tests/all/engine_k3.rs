//! WP-K3 tests: K3 classes (ClassSet::more) fed from level data + ClassesPlugin over all 43 levels. Needs the ISO at
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

use sly2_recomp::engine::classes::plugin::{Classes, ClassesLevel, ClassesPlugin, EngineHost};
use sly2_recomp::engine::classes::{Inst, PORTED_K3};
use std::collections::BTreeMap;

/// Per-class statistics over all levels.
#[derive(Default, Debug)]
struct Stats {
    n: BTreeMap<u16, usize>,
    fed: BTreeMap<u16, usize>,
    created: usize,
    c111_74: usize,
    host: Vec<EngineHost>,
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
        let expect = w.objects.iter().filter(|o| PORTED_K3.iter().any(|p| p.0 == o.class)).count();
        let own = c.set.more.iter().filter(|x| x.0 < w.objects.len()).count();
        assert_eq!(own, expect, "lsn {}", d.lsn());
        st.created += c.set.more.len() - own;
        for (i, x) in &c.set.more {
            let cls = w.objects.get(*i).map(|o| o.class).unwrap_or(0xffff);
            *st.n.entry(cls).or_default() += 1;
            let fed = match x {
                Inst::C2ceb78(k) => k.f_f8 != 10.0 || k.f_110 != 0.0 || k.f_118 != -1 || k.tpl.w[20] != 750.0f32.to_bits(),
                // class 6: no method table ported -> FUN_001a5c48 parks the object (+0x30 = FLT_MAX)
                Inst::C2d6e08(k) => k.f_30 == f32::MAX,
                // class 133: a reference / timer prop or an oid list that resolved
                Inst::C2cf1e8(k) => k.f_80 != 0 || k.f_7c != 0 || k.f_78 != 0.0 || !k.f_98.is_empty() || !k.f_c8.is_empty(),
                // class 114: load record 0 {0, 1, .., +0x10 = -1} (K22: then the sequence records), capacity > count
                Inst::C2d8c48(k) => !k.f_54.is_empty() && k.f_54[0][4] == u32::MAX && k.f_58 > k.f_54.len() as i32,
                // class 31: attached class-30 list from the load
                Inst::C2ce828(k) => !k.f_e4.is_empty(),
                // class 4: registered in the light table by slot 22, FUN_001a3968 ran (+0x120 = kind)
                Inst::C2d33a0(k) => k.f_118 >= 0 && k.f_120 == k.p.kind,
                // class 111: slot 6 ran (+0x40 bit 6)
                Inst::C2d97a8(k) => k.f_40 & 0x40 != 0,
                // class 26: slot 6 defaults overwritten by a warp field (+0x150 level / +0x13c state / delays)
                Inst::C2dc6f0(k) => k.f_150 != 0x189 || k.f_154 != -1 || k.f_15c != 0.0 || k.f_160 != 0.0 || k.f_13c != 0,
                // class 120: bound to a class-0x38 owner (+0xac) by slot 14
                Inst::C2da840(k) => k.f_ac != 0 && k.f_ac == k.anc,
                _ => false,
            };
            if let Inst::C2d97a8(k) = x {
                st.c111_74 += k.f_74.is_some() as usize;
            }
            if fed {
                *st.fed.entry(cls).or_default() += 1;
            }
        }
        st.host.push(c.host.clone());
    }
    st
}

#[test]
#[ignore = "heavy"]
fn k3_all_levels_build_init_120_frames() {
    let st = run_all();
    let sum = |f: fn(&EngineHost) -> i64| st.host.iter().map(f).sum::<i64>();
    let (emits, parts, found, deferred) = (sum(|h| h.emits as i64), sum(|h| h.particles), sum(|h| h.found as i64), sum(|h| h.deferred as i64));
    eprintln!("K3 111 with +0x74: {}", st.c111_74);
    eprintln!("K3 n {:?} fed {:?} created {} emits {emits} particles {parts} found {found} deferred {deferred}", st.n, st.fed, st.created);
    // class 29: most emitters carry rate / template props; they emit in 120 frames
    let n29 = st.n[&29];
    assert!(st.fed[&29] * 2 > n29, "class 29 fed from props");
    assert!(emits > 0 && parts > 0, "class 29 slot 15 emits");
    assert_eq!(st.fed[&6], st.n[&6], "class 6 slot 14 ran on every instance");
    assert_eq!(st.fed[&114], st.n[&114], "class 114 load tail");
    assert!(st.fed.get(&31).copied().unwrap_or(0) * 2 > st.n.get(&31).copied().unwrap_or(0), "class 31 attached lists");
    assert_eq!(st.fed.get(&4), st.n.get(&4), "class 4 lights registered");
    assert_eq!(st.fed.get(&111), st.n.get(&111), "class 111 init");
    assert_eq!(st.fed.get(&26), st.n.get(&26), "class 26 warp fields");
    eprintln!("K3 class 120 bound {:?} / {:?}", st.fed.get(&120), st.n.get(&120));
    assert!(st.fed.get(&120).copied().unwrap_or(0) * 2 > st.n.get(&120).copied().unwrap_or(0), "class 120 ancestor");
    let vcalls = sum(|h| h.vcalls as i64);
    eprintln!("vcalls {vcalls}");
    assert!(st.fed.get(&133).copied().unwrap_or(0) > 0 && vcalls > 0, "class 133 fed + slot 15 drives its +0x80 object");
}

#[test]
fn c2ceb78_modes() {
    use sly2_recomp::engine::classes::c2ceb78::{C2ceb78, Group};
    use sly2_recomp::engine::classes::ClassGlobals;
    use sly2_recomp::engine::physics::driven::GameRng;
    let mut h = EngineHost::default();
    let mut rng = GameRng::default();
    let mut g = ClassGlobals { dt: 1.0 / 60.0, ..Default::default() };
    let mut c = C2ceb78::default();
    c.ee_sub_1591b0(0, &mut h);
    assert_eq!((c.f_f0, c.f_f4, c.f_118, c.f_f8, c.f_148), (-1, -1, -1, 10.0, -1.0));
    // mode picked from +0x104/+0x114/+0x110: all zero -> 0 (rate mode), +0x110 -> 1
    c.f_f8 = 60.0;
    c.f_fc = 60.0;
    assert_eq!(c.ee_sub_159500(0, &mut rng, &g, &mut h), Group::None);
    assert_eq!((c.f_f0, c.f_130, c.f_134, c.f_138), (0, 60.0, 1.0, 1.0));
    // 60/s at dt 1/60 -> one particle per frame
    for _ in 0..10 {
        g.t += g.dt;
        c.ee_sub_159c80(0, &mut rng, &g, &mut h);
    }
    assert_eq!((h.emits, h.particles), (10, 10));
    // budget +0xf4 stops it
    c.f_f4 = 3;
    for _ in 0..10 {
        g.t += g.dt;
        c.ee_sub_159c80(0, &mut rng, &g, &mut h);
    }
    assert_eq!((h.particles, c.f_f4), (13, 0));
    // group oid -> join request, object disabled
    let mut d = C2ceb78::default();
    d.ee_sub_1591b0(1, &mut h);
    d.f_118 = 77;
    assert_eq!(d.ee_sub_159500(1, &mut rng, &g, &mut h), Group::Join(77));
    assert_eq!(d.f_40 & 2, 0);
    // mode 1: keep +0x110 * +0x13c particles alive
    let mut e = C2ceb78::default();
    e.ee_sub_1591b0(2, &mut h);
    e.f_110 = 5.0;
    e.ee_sub_159500(2, &mut rng, &g, &mut h);
    assert_eq!(e.f_f0, 1);
    let p0 = h.particles;
    e.ee_sub_159c80(2, &mut rng, &g, &mut h);
    assert_eq!(h.particles - p0, 5);
}

#[test]
fn c2d6e08_slot14() {
    use sly2_recomp::engine::classes::c2d6e08::C2d6e08;
    use sly2_recomp::engine::classes::ClassHost;
    struct H(Vec<&'static str>);
    impl ClassHost for H {
        fn ee_sub_127db8(&mut self, _o: usize, _id: i32, _d: i32) -> bool {
            true
        }
        fn ee_sub_1a6ca8(&mut self, _o: usize, _a: i32) {
            self.0.push("1a6ca8");
        }
    }
    let mut h = H(vec![]);
    let mut c = C2d6e08 { f_28: 2 << 52, ..Default::default() };
    c.ee_sub_1a5c48(0, &mut h);
    assert_eq!((c.f_30, c.f_40, h.0.len()), (0.0, 2, 1));
}

#[test]
fn c2d8c48_run_stop() {
    use sly2_recomp::engine::classes::c2d8c48::C2d8c48;
    use sly2_recomp::engine::classes::{ClassGlobals, ClassHost};
    #[derive(Default)]
    struct H(usize, usize, usize);
    impl ClassHost for H {
        fn ee_sub_1e1c40(&mut self, _n: u32) {
            self.0 += 1;
        }
        fn ee_sub_1e0120(&mut self, _o: u32) {
            self.1 += 1;
        }
        fn vcall_1c(&mut self, _o: usize, a: i32, _b: i32) {
            self.2 += a as usize;
        }
    }
    let g = ClassGlobals { dt: 0.5, ..Default::default() };
    let mut h = H::default();
    let mut c = C2d8c48::default();
    c.ee_sub_1dfad8(0, &mut h);
    c.feed(&[]);
    assert_eq!((c.f_54.len(), c.f_58), (1, 2));
    c.f_74 = 1;
    c.f_5c = vec![5, 6];
    c.ee_sub_1dfc50(0, &g, &mut h);
    // K22: nodes 5 / 6 have no interpreter state (not script-started) -> nothing runs, they stay linked
    assert_eq!((c.f_70, h.0, c.f_74, c.f_5c.len()), (0.5, 0, 1, 2));
    c.f_5c.clear();
    c.ee_sub_1dfc50(0, &g, &mut h);
    assert_eq!((c.f_70, c.f_74, h.1, h.2), (0.0, 0, 1, 6));
    // table full at capacity
    assert_eq!(c.ee_sub_1e01f8(1, 1), Some(1));
    assert_eq!(c.ee_sub_1e01f8(2, 1), None);
}

#[test]
fn c2ce828_forward() {
    use sly2_recomp::engine::anim::pose::{Mat4, IDENTITY};
    use sly2_recomp::engine::classes::c2ce828::{C2ce828, ReqBlock};
    use sly2_recomp::engine::classes::ClassHost;
    #[derive(Default)]
    struct H(Vec<(u32, [u32; 15], ReqBlock)>);
    impl ClassHost for H {
        fn obj_mat(&mut self, obj: u32, _off: u32) -> Mat4 {
            let mut m = IDENTITY;
            m[3] = [obj as f32, 0.0, 0.0, 1.0];
            m
        }
        fn vcall_19c(&mut self, obj: u32, req: &[u32; 15], m: &ReqBlock) {
            self.0.push((obj, *req, *m));
        }
    }
    let mut c = C2ce828::default();
    c.feed(&[4, 9]);
    let mut h = H::default();
    c.ee_sub_15c3c0(0, &[0; 15], &[[0.0; 4]; 6], &mut h);
    // obj handle 1 at x = 1; children 5 / 10 -> relative x 4 / 9; flags |= 0x41, source = this
    assert_eq!(h.0.len(), 2);
    assert_eq!((h.0[0].0, h.0[0].1[0], h.0[0].1[1]), (5, 0x41, 1));
    assert_eq!(h.0[1].2[0], [9.0, 0.0, 0.0, 1.0]);
    assert_eq!(h.0[1].2[1], [1.0, 0.0, 0.0, 0.0]);
    // composed with a request block (scale 2, offset 100)
    let mut m: ReqBlock = [[0.0; 4]; 6];
    m[0] = [100.0, 0.0, 0.0, 0.0];
    m[1] = [2.0, 0.0, 0.0, 0.0];
    m[2] = [0.0, 2.0, 0.0, 0.0];
    m[3] = [0.0, 0.0, 2.0, 0.0];
    m[4] = [7.0; 4];
    h.0.clear();
    c.ee_sub_15c3c0(0, &[0x41, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], &m, &mut h);
    assert_eq!((h.0[0].1[0], h.0[0].1[1]), (0x41, 3));
    assert_eq!(h.0[0].2[0][0], 108.0);
    assert_eq!(h.0[0].2[1][0], 2.0);
    assert_eq!(h.0[0].2[4], [7.0; 4]);
}

#[test]
fn c2d33a0_light_table() {
    use sly2_recomp::engine::classes::c2d33a0::{C2d33a0, LightTable};
    use sly2_recomp::engine::classes::{ClassGlobals, NullHost};
    let mut t = LightTable::default();
    let mut g = ClassGlobals::default();
    let mut h = NullHost;
    let mut a = C2d33a0::default();
    let mut b = C2d33a0::default();
    b.p.kind = 1;
    b.p.hsv = [0.0, 0.0, 255.0];
    a.ee_sub_1a3400(0, &mut g, &mut h);
    b.ee_sub_1a3400(1, &mut g, &mut h);
    assert_eq!((g.g_3e11ec, g.g_3e19e4, b.d.g), (2, 1, 255.0));
    a.ee_sub_1a3518(0, &mut t, &mut h);
    b.ee_sub_1a3518(1, &mut t, &mut h);
    assert_eq!((a.f_118, b.f_118, t.slots[1], t.global[0]), (0, 1, 2, 3));
    a.ee_sub_1a34e0(0, &mut t, &mut h);
    assert_eq!((a.f_118, t.slots[0], t.dirty, t.free[0] & 3), (-1, 0, 1, 1));
    // re-register takes slot 0 again, but the dirty table skips the global insert
    t.global = [0; 8];
    a.ee_sub_1a3518(0, &mut t, &mut h);
    assert_eq!((a.f_118, t.global[0]), (0, 0));
}

#[test]
fn c2cad78_slot5() {
    use sly2_recomp::engine::classes::{c2cad78, PORTED_K3_STATELESS};
    assert_eq!(c2cad78::ee_sub_1140e0(), 0x2bb36c);
    assert!(PORTED_K3_STATELESS.contains(&(109, 0x2cad78)));
}

#[test]
fn c2d97a8_slot14() {
    use sly2_recomp::engine::classes::c2d97a8::C2d97a8;
    use sly2_recomp::engine::classes::ClassHost;
    #[derive(Default)]
    struct H(Vec<(usize, u32, u32, u32)>);
    impl ClassHost for H {
        fn ee_sub_15e938(&mut self, _k: i32, oid: i32, _c: u32) -> u32 {
            oid as u32 + 100
        }
        fn ee_sub_1f0380(&mut self, obj: usize, t: u32, a: u32, b: u32) {
            self.0.push((obj, t, a, b));
        }
    }
    let mut h = H::default();
    let mut c = C2d97a8::default();
    c.ee_sub_1efa90(3, &mut h);
    c.ee_sub_1f00c8(3, &mut h);
    assert!(h.0.is_empty() && c.f_40 == 0x40);
    c.ee_sub_1f0128(7, 8, 9);
    c.ee_sub_1f00c8(3, &mut h);
    assert_eq!(h.0, vec![(3, 107, 8, 9)]);
}

#[test]
fn c2dc6f0_warp() {
    use sly2_recomp::engine::classes::c2dc6f0::C2dc6f0;
    use sly2_recomp::engine::classes::{ClassGlobals, ClassHost};
    #[derive(Default)]
    struct H {
        inside: bool,
        warps: Vec<(i32, i32, i32)>,
        lists: i32,
    }
    impl ClassHost for H {
        fn ee_sub_1c4840(&mut self) -> u32 {
            1
        }
        fn ee_sub_118fb0(&mut self, _o: usize, _p: [f32; 3]) -> bool {
            self.inside
        }
        fn ee_sub_1b3a48(&mut self, _o: usize, _k: i32) {
            self.lists += 1;
        }
        fn ee_sub_22b490(&mut self, _o: usize, l: i32, f: i32, a: i32) {
            self.warps.push((l, f, a));
        }
    }
    let mut g = ClassGlobals::default();
    let mut h = H::default();
    let mut c = C2dc6f0::default();
    c.ee_sub_2281e8(2, &mut h);
    c.f_148 = 2;
    c.f_15c = 1.0;
    c.f_160 = 0.5;
    c.ee_sub_228538(2, &mut g, &mut h); // 0, outside -> 1
    assert_eq!(c.f_13c, 1);
    g.t = 2.0;
    c.ee_sub_228538(2, &mut g, &mut h); // 1, outside, delay passed -> 3
    assert_eq!(c.f_13c, 3);
    h.inside = true;
    c.ee_sub_228538(2, &mut g, &mut h); // 3, inside -> warp (state 4, time t + 0.5)
    assert_eq!((c.f_13c, c.f_158, g.g_3e1114), (4, 2.5, 3));
    c.ee_sub_228538(2, &mut g, &mut h);
    assert!(h.warps.is_empty());
    g.t = 3.0;
    c.ee_sub_228538(2, &mut g, &mut h);
    assert_eq!((h.warps.clone(), h.lists, c.f_158), (vec![(0x189, 0, -1)], 2, 0.0));
}

#[test]
fn c2da840_slot14() {
    use sly2_recomp::engine::classes::c2da840::C2da840;
    use sly2_recomp::engine::classes::ClassHost;
    #[derive(Default)]
    struct H(Vec<i32>, usize);
    impl ClassHost for H {
        fn ee_sub_1f4c58(&mut self, _a: u32, oid: i32, _k: i32) -> u32 {
            self.0.push(oid);
            oid as u32
        }
        fn ee_sub_1300b8(&mut self, _an: u32, _k: i32, _alo: u32, _b: u32, _a: u32) -> [u32; 4] {
            [1, 2, 3, 4]
        }
        fn ee_sub_209130(&mut self, _o: usize, _a: u32) {
            self.1 += 1;
        }
    }
    let mut h = H::default();
    let mut c = C2da840::default();
    c.ee_sub_20ad38(0, &mut h);
    assert_eq!((c.f_50[1], c.f_a8, c.f_7c), (300.0, 1, -1));
    c.ee_sub_20ae18(0, 9, 0x2cc, &mut h);
    assert!(!c.f_c0_self && h.0.is_empty() && h.1 == 1 && c.f_ac == 9);
    c.ee_sub_20af28(5, 6, 7);
    c.ee_sub_20ae18(0, 9, 1, &mut h);
    assert_eq!((h.0.clone(), c.f_a0, c.f_a4, c.f_90, c.f_c0_self), (vec![5, 6], 5, 6, [1, 2, 3, 4], true));
}
