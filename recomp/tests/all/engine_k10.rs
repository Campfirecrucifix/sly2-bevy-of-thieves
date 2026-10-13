//! WP-K10: the K10 classes (35, 36, 27, 19, 38, 21, 40, 9, 12) on the real levels: all 43 levels build + 300
//! frames, and one behaviour test per class on a real-level instance.
use sly2_recomp::engine::classes::k10::{self, K10};
use sly2_recomp::engine::classes::plugin::EngineHost;
use sly2_recomp::engine::classes::{ClassHost, ClassSet, Inst};
use sly2_recomp::engine::disc::descriptor::Descriptor;
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::world::{LoadOptions, World};
use std::sync::Arc;

fn levels() -> (LevelLoader, Vec<Descriptor>) {
    crate::common::levels() // public: ISO level table instead of the analysis DB
}

/// Recording host: world lookups through `EngineHost`, K10 hooks logged.
#[derive(Default)]
struct Rec {
    e: EngineHost,
    log: Vec<String>,
    /// handle answering is-a 0x36 (Bentley)
    bentley: u32,
    pos: [f32; 3],
}
impl ClassHost for Rec {
    fn ee_sub_15e938(&mut self, kind: i32, oid: i32, ctx: u32) -> u32 {
        self.e.ee_sub_15e938(kind, oid, ctx)
    }
    fn ee_sub_132b10(&mut self, obj: u32, class: i32) -> u32 {
        if obj == self.bentley && class == 0x36 {
            1
        } else {
            self.e.ee_sub_132b10(obj, class)
        }
    }
    fn vcall_24(&mut self, obj: usize, a: i32, b: u32) {
        self.log.push(format!("vt24 {obj} {a} {b}"));
    }
    fn vcall_1c(&mut self, obj: usize, a: i32, b: i32) {
        self.log.push(format!("vt1c {obj} {a} {b}"));
    }
    fn ee_sub_20ff70(&mut self, obj: usize, e: &[[u32; 3]]) {
        self.log.push(format!("20ff70 {obj} {}", e.len()));
    }
    fn ee_sub_161f48_list(&mut self, obj: usize, e: &[[u32; 2]]) {
        self.log.push(format!("161f48 {obj} {}", e.len()));
    }
    fn ee_sub_1dfd58(&mut self, seq: u32, player: u32) {
        self.log.push(format!("seq {seq} {player}"));
    }
    fn ee_sub_1b7000(&mut self, user: u32, point: u32) {
        self.log.push(format!("1b7000 {user} {point}"));
    }
    fn ee_sub_1a62e8(&mut self, _obj: u32) -> [f32; 3] {
        self.pos
    }
    fn vol_centre_110(&mut self, _obj: usize) -> [f32; 3] {
        [1.0, 2.0, 2.0]
    }
    fn ee_sub_1a6f00_to_world(&mut self, _obj: usize, p: [f32; 3]) -> [f32; 3] {
        [p[0], p[1] + 1.0, p[2]]
    }
}

/// First level (in disc order) with an object of `class`, built + post-loaded with `h`.
fn level_with(class: u16, h: &mut Rec) -> (Arc<World>, ClassSet) {
    let (loader, descs) = levels();
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        if !w.objects.iter().any(|o| o.class == class) {
            continue;
        }
        h.e.world = Some(w.clone());
        let mut s = ClassSet::build(&w, h);
        s.post_load(h);
        s.g.dt = 1.0 / 60.0;
        return (w, s);
    }
    panic!("no level with class {class}");
}
fn k10s(s: &ClassSet) -> impl Iterator<Item = (usize, &K10)> {
    s.k10.iter().filter_map(|(i, x)| match x {
        Inst::K10(k) => Some((*i, k.as_ref())),
        _ => None,
    })
}
fn k10_mut(s: &mut ClassSet, i: usize) -> &mut K10 {
    match s.k10.iter_mut().find(|x| x.0 == i) {
        Some((_, Inst::K10(k))) => k.as_mut(),
        _ => panic!("object {i}"),
    }
}

/// Every K10 class builds on all 43 levels and runs 300 frames; list membership, radii and slot 5 hold.
#[test]
#[ignore = "heavy"]
fn k10_classes_all_levels_300_frames() {
    let (loader, descs) = levels();
    assert_eq!(descs.len(), 43);
    let mut counts = std::collections::BTreeMap::<u16, usize>::new();
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        s.post_load(&mut h);
        s.g.dt = 1.0 / 60.0;
        for (i, o) in w.objects.iter().enumerate() {
            if k10::PORTED_K10.iter().any(|x| x.0 == o.class) {
                assert!(s.k10.iter().any(|x| x.0 == i), "class {} object {i} built", o.class);
                assert!(k10::slot5(o.class).is_some());
                *counts.entry(o.class).or_default() += 1;
            }
        }
        for (i, k) in k10s(&s) {
            let hd = i as u32 + 1;
            let listed = |a: u32| s.g.k10_lists.get(&a).is_some_and(|l| l.contains(&hd));
            match k {
                K10::C2d1ee8(c) => assert!(c.f_fc != 0 && c.f_38 != 1e10),
                K10::C2d1d00(c) => assert!(c.base.f_fc & 4 != 0 && listed(k10::L_3E1414)),
                K10::C2d1b60(c) => assert!(c.f_38.is_finite() && listed(k10::L_3E1438)),
                K10::C2d2978(c) => assert!(c.f_38 != 1e10 && listed(k10::L_3E13B4)),
                K10::C2d2378(c) => assert!(c.f_38 != 1e10 && listed(k10::L_3E1408)),
                K10::C2db378(_) => assert!(listed(k10::L_3E1474)),
                K10::C2d3200(_) => assert!(listed(k10::L_3E1420)),
                K10::C2d2b18(_) => assert!(listed(k10::L_3E142C)),
                K10::C2d2738(c) => assert!(c.f_38 != 1e10 && listed(k10::L_3E13F0)),
            }
        }
        for _ in 0..300 {
            s.g.t += s.g.dt;
            s.frame(&mut h);
        }
    }
    println!("K10 classes over 43 levels: {counts:?}");
    for c in [35, 36, 27, 19, 38, 21, 40, 9, 12] {
        assert!(counts.get(&c).copied().unwrap_or(0) > 0, "class {c} present");
    }
}

/// Class 35: the character mask comes from field 1074 (0x34 -> all) / slot 14 (0 -> 7); slot 50 records the
/// user, raises vt+0x24(6) / FUN_0020ff70 / FUN_00161f48 / vt+0x1c(5) and starts the +0x114 sequence.
#[test]
fn k10_c35_use_point() {
    let mut h = Rec::default();
    let (_w, mut s) = level_with(35, &mut h);
    let (i, c) = k10s(&s)
        .find_map(|(i, k)| match k {
            K10::C2d1ee8(c) => Some((i, c.clone())),
            _ => None,
        })
        .unwrap();
    assert!(c.f_fc != 0 && c.f_fc <= 7, "mask {}", c.f_fc);
    assert_eq!(c.f_38, 1000.0);
    let sly_ok = c.f_fc & 1 != 0;
    h.log.clear();
    assert_eq!(s.k10_use(i, 0x9999, 0x35, &mut h), sly_ok);
    if sly_ok {
        assert_eq!(h.log[0], format!("vt24 {i} 6 {}", i + 1));
        assert!(h.log.iter().any(|l| l == &format!("vt1c {i} 5 0")));
        assert_eq!(h.log.iter().any(|l| l.starts_with("seq ")), c.f_114 != 0);
        let K10::C2d1ee8(c2) = k10_mut(&mut s, i) else { panic!() };
        assert_eq!(c2.f_10c, 0x9999);
    }
    println!("class 35 object {i}: mask {} log {:?}", c.f_fc, h.log);
    // setter FUN_00184040: 0x36 -> Bentley bit, 0x37 -> Murray bit, 0x34 -> all
    let mut m = k10::C2d1ee8::default();
    m.ee_sub_184040(0x36);
    assert_eq!(m.f_fc, 4);
    m.ee_sub_184040(0x37);
    assert_eq!(m.f_fc, 6);
    m.ee_sub_184040(0x34);
    assert_eq!(m.f_fc, 7);
}

/// Class 36: Bentley-only point (slot 6 / field 1074 = 0x36); Sly is refused, Bentley's use ends in FUN_001b7000.
#[test]
fn k10_c36_bentley_point() {
    let mut h = Rec { bentley: 0x7777, ..Default::default() };
    let (_w, mut s) = level_with(36, &mut h);
    let (i, c) = k10s(&s)
        .find_map(|(i, k)| match k {
            K10::C2d1d00(c) => Some((i, c.clone())),
            _ => None,
        })
        .unwrap();
    assert_eq!(c.base.f_fc, 4, "Bentley only");
    assert_eq!(c.base.f_f0, -1);
    assert!(!s.k10_use(i, 0x9999, 0x35, &mut h));
    h.log.clear();
    assert!(s.k10_use(i, 0x7777, 0x36, &mut h));
    assert_eq!(h.log.last().unwrap(), &format!("1b7000 {} {}", 0x7777, i + 1));
    println!("class 36 links {:?} -> {:?}, +0x180 {:#x}", c.oid, c.obj, c.f_180);
}

/// Class 27: +0x38 = |position - world(bounding-sphere centre)| + 1000 at load-post.
#[test]
fn k10_c27_radius() {
    let mut h = Rec { pos: [4.0, 7.0, 2.0], ..Default::default() };
    let (_w, s) = level_with(27, &mut h);
    let c = k10s(&s)
        .find_map(|(_, k)| match k {
            K10::C2d1b60(c) => Some(c.clone()),
            _ => None,
        })
        .unwrap();
    // |(4,7,2) - (1,3,2)| = 5
    assert_eq!(c.f_38, 1005.0);
    assert_eq!(c.sub.f_1c != 0, true);
}

/// Class 19: slot 6 defaults over class 18, fields 1011-1016, +0x38 = +0xec + 1500.
#[test]
fn k10_c19_defaults() {
    let mut h = Rec::default();
    let (_w, s) = level_with(19, &mut h);
    let c = k10s(&s)
        .find_map(|(_, k)| match k {
            K10::C2d2978(c) => Some(c.clone()),
            _ => None,
        })
        .unwrap();
    assert_eq!(c.f_38, c.base.f_ec + 1500.0);
    let mut d = k10::C2d2978::default();
    d.ee_sub_184f50(0, &mut h);
    assert_eq!((d.base.f_e8, d.f_130, d.f_12c, d.sub.f_14, d.sub.f_1c, d.base.f_ec), (1, 9, 8, -1, 7, 25.0));
    println!("class 19: +0xe8 {} +0xec {} +0x124 {} +0x12c {} +0x130 {} +0x138 {}", c.base.f_e8, c.base.f_ec, c.f_124, c.f_12c, c.f_130, c.f_138);
}

/// Classes 38 / 12: the +0 sub-block volume (a class-15 C2db6b8) is forced to state 0 (FUN_00221268).
#[test]
fn k10_c38_c12_force_volume() {
    for class in [38u16, 12] {
        let mut h = Rec::default();
        let (_w, s) = level_with(class, &mut h);
        let mut forced = 0;
        for (_, k) in k10s(&s) {
            let (vol, r) = match k {
                K10::C2d2378(c) if class == 38 => (c.sub.f_00, c.f_38 == 1250.0),
                K10::C2d2738(c) if class == 12 => (c.sub.f_00, c.f_38 >= 1250.0),
                _ => continue,
            };
            assert!(r);
            if vol == 0 {
                continue;
            }
            if let Some((_, Inst::C2db6b8(v))) = s.k4.iter().find(|x| x.0 as u32 + 1 == vol) {
                assert_eq!(v.f_13c, 0);
                forced += 1;
            }
        }
        println!("class {class}: {forced} class-15 volumes forced");
        if class == 38 {
            assert!(forced > 0);
        }
    }
}

/// Class 21: slot 6 defaults overridden by fields 219 (+0xec range) / 220 / 221; slot 21 leaves DAT_003e1474.
#[test]
fn k10_c21_fields_and_list() {
    let mut h = Rec::default();
    let (_w, mut s) = level_with(21, &mut h);
    let (i, c) = k10s(&s)
        .find_map(|(i, k)| match k {
            K10::C2db378(c) => Some((i, c.clone())),
            _ => None,
        })
        .unwrap();
    assert!(c.f_e8 > 0.0 && c.f_ec[0] <= c.f_ec[1]);
    println!("class 21: +0xe8 {} +0xec {:?} +0xf4 {}", c.f_e8, c.f_ec, c.f_f4);
    let mut d = k10::C2db378::default();
    d.ee_sub_21d3d8(0, &mut h);
    assert_eq!((d.f_e8, d.f_ec, d.f_f4), (2000.0, [-800.0, 800.0], 1));
    assert!(s.g.list(k10::L_3E1474).contains(&(i as u32 + 1)));
    let mut g = std::mem::take(&mut s.g);
    let K10::C2db378(c) = k10_mut(&mut s, i) else { panic!() };
    c.ee_sub_21d428(i, &mut g, &mut h);
    assert!(!g.list(k10::L_3E1474).contains(&(i as u32 + 1)));
}

/// Class 40: field 870 sets +0xe8 (slot 6: 100.0); slots 10 / 11 add / remove DAT_003e1420.
#[test]
fn k10_c40_list() {
    let mut h = Rec::default();
    let (_w, mut s) = level_with(40, &mut h);
    let (i, c) = k10s(&s)
        .find_map(|(i, k)| match k {
            K10::C2d3200(c) => Some((i, c.clone())),
            _ => None,
        })
        .unwrap();
    assert_eq!(c.f_e8, 200.0, "field 870");
    assert_eq!(s.g.list(k10::L_3E1420), &vec![i as u32 + 1]);
    let mut g = std::mem::take(&mut s.g);
    let K10::C2d3200(c) = k10_mut(&mut s, i) else { panic!() };
    c.ee_sub_19ca20(i, &mut g, &mut h);
    assert!(g.list(k10::L_3E1420).is_empty());
}

/// Class 9: +0x110 resolves to a class-15 volume (FUN_00221178(vol, 0) at load-post), +0x108 set.
#[test]
fn k10_c9_volume() {
    let mut h = Rec::default();
    let (_w, s) = level_with(9, &mut h);
    let mut linked = 0;
    for (_, k) in k10s(&s) {
        let K10::C2d2b18(c) = k else { continue };
        assert!(c.f_108 > 0.0);
        if c.f_110 != 0 && s.k4.iter().any(|x| x.0 as u32 + 1 == c.f_110 && matches!(x.1, Inst::C2db6b8(_))) {
            linked += 1;
        }
    }
    println!("class 9: {linked} linked class-15 volumes");
    assert!(linked > 0);
}
