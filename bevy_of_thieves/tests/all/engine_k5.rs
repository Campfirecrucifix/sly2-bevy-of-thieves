//! WP-K5 tests: K5 actor classes (ClassSet::k5) fed from level data + ClassesPlugin over all 43 levels. Needs the ISO at
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


use sly2_recomp::engine::classes::actor::alo::{Alo, Contact, Contacts, Touch};
use sly2_recomp::engine::classes::actor::c2caf88::C2caf88;
use sly2_recomp::engine::classes::actor::c2d4b78::{Act, C2d4b78, STAGE_FIRST, STAGE_LAST};
use sly2_recomp::engine::classes::actor::PORTED_K5;
use sly2_recomp::engine::classes::plugin::{Classes, ClassesLevel, ClassesPlugin};
use sly2_recomp::engine::classes::{ClassHost, Inst};
use std::collections::BTreeMap;

fn run_all() -> (BTreeMap<u16, usize>, BTreeMap<String, usize>) {
    let (loader, descs) = levels();
    let mut n = BTreeMap::new();
    let mut st: BTreeMap<String, usize> = BTreeMap::new();
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
        let expect = w.objects.iter().filter(|o| PORTED_K5.iter().any(|p| p.0 == o.class)).count();
        assert_eq!(c.set.k5.len(), expect, "lsn {}", d.lsn());
        let mut b = |k: &str, v: bool| *st.entry(k.into()).or_default() += v as usize;
        for (i, x) in &c.set.k5 {
            *n.entry(w.objects[*i].class).or_default() += 1;
            match x {
                Inst::C2caf88(k) => {
                    b("86 init (bit 34, +0x140 -1, alpha 1)", k.alo.so.f_148 >> 34 & 1 != 0 && k.alo.so.f_140 == -1 && k.alo.so.f_12c == 1.0);
                    b("86 oid filter", !k.f_154.is_empty());
                    b("86 class filter", !k.f_168.is_empty());
                    b("86 +0x178", k.f_178 != 0);
                    b("86 +0x17c", k.f_17c != 0);
                }
                Inst::C2d4b78(k) => {
                    b("98 init (+0x110 masks)", k.alo.f_110[1] == 8 && k.alo.f_110[0] == 0 && k.alo.f_110[19] == 0);
                    b("98 stages", !k.stages.is_empty());
                    b("98 actions", k.stages.iter().any(|s| !s.acts.is_empty()) || !k.pre.acts.is_empty());
                    b("98 filter", !k.f_154.is_empty());
                    b("98 at stage 0", k.f_15c == 0);
                    b("98 in 0x3e1450", c.set.g.l_3e1450.contains(&(*i as u32 + 1)));
                }
                _ => panic!("non-K5 inst in k5"),
            }
        }
    }
    (n, st)
}

#[test]
#[ignore = "heavy"]
fn k5_all_levels_build_init_120_frames() {
    let (n, st) = run_all();
    eprintln!("K5 n {n:?}\nK5 fed {st:#?}");
    let (n86, n98) = (n[&86], n[&98]);
    assert!(n86 > 0 && n98 > 0);
    assert_eq!(st["86 init (bit 34, +0x140 -1, alpha 1)"], n86);
    assert_eq!(st["98 init (+0x110 masks)"], n98);
    assert_eq!(st["98 at stage 0"], n98);
    assert_eq!(st["98 in 0x3e1450"], n98);
    assert!(st["98 stages"] * 2 > n98, "most class-98 switches carry stages");
    assert!(st["86 oid filter"] + st["86 class filter"] > 0, "class 86 filters fed");
}

/// Records the hook calls the actor slots make.
#[derive(Default)]
struct Rec {
    log: Vec<String>,
    classes: BTreeMap<u32, u16>,
    phys: bool,
    contacts: Vec<Contact>,
}
impl ClassHost for Rec {
    fn ee_sub_15e938(&mut self, _k: i32, oid: i32, _c: u32) -> u32 {
        if oid > 0 { oid as u32 + 100 } else { 0 }
    }
    fn vcall(&mut self, obj: u32, off: u32) {
        self.log.push(format!("v{off:x}({obj})"));
    }
    fn ee_sub_1f6978(&mut self, _o: usize, id: i32) {
        self.log.push(format!("show{id}"));
    }
    fn ee_sub_1f69f8(&mut self, _o: usize, id: i32) {
        self.log.push(format!("hide{id}"));
    }
    fn ee_sub_132b10(&mut self, obj: u32, class: i32) -> u32 {
        let c = self.classes.get(&obj).copied().unwrap_or(0) as i32;
        sly2_recomp::engine::player::p4::FUN_00132b10(c, class) as u32
    }
    fn ee_sub_149ab8(&mut self, _o: usize, pmf: u32, _a: i32) {
        self.log.push(format!("q{pmf:x}"));
    }
    fn phys_e8(&mut self, _o: usize) -> Option<sly2_recomp::engine::classes::actor::alo::PhysE8> {
        self.phys.then(Default::default)
    }
    fn contacts_ec(&mut self, _o: usize) -> Contacts {
        Contacts { head_6c: false, list_64: self.contacts.clone() }
    }
    fn vcall_1b4(&mut self, obj: u32, other: u32) {
        self.log.push(format!("t{obj}>{other}"));
    }
    fn vcall_ret(&mut self, obj: u32, _off: u32) -> i32 {
        (obj == 7) as i32
    }
}

#[test]
fn c2d4b78_stage_runner() {
    let mut c = C2d4b78::default();
    let mut h = Rec::default();
    c.ee_sub_1bab50(0, None, 0, &mut h);
    // no stage yet: field 1040 runs now
    c.ee_sub_1bbbb0(0, 9, &mut h);
    assert_eq!(h.log, ["show9"]);
    h.log.clear();
    c.ee_sub_1bb950(0x11);
    c.ee_sub_1bbbb0(0, 3, &mut h); // action 2 (show 3)
    c.ee_sub_1bbc98(0, 4, &mut h); // action 3 (hide 4)
    c.ee_sub_1bb950(0x22);
    c.ee_sub_1bbdb0(0).unwrap().arg = 5; // vcall +0x44 on oid 5
    c.ee_sub_1bbdb0(1).unwrap().arg = 0; // unknown oid: skipped
    assert!(h.log.is_empty());
    assert_eq!(c.f_15c, 1);
    c.f_15c = 0; // slot 13 tail
    c.ee_sub_1baf18(0, STAGE_LAST, &mut h);
    assert_eq!(c.f_15c, 2);
    assert_eq!(h.log, ["show3", "hide4", "v44(105)"]);
    h.log.clear();
    // back to stage 0x11 (index 0): stage 1 then stage 0 actions reversed + inverted
    c.ee_sub_1baf18(0, 0x11, &mut h);
    assert_eq!(c.f_15c, 0);
    assert_eq!(h.log, ["v4c(105)", "show4", "hide3"]);
    h.log.clear();
    c.ee_sub_1baf18(0, STAGE_FIRST, &mut h);
    assert!(h.log.is_empty());
    assert_eq!(c.stages[1].acts[0], Act { kind: 0, arg: 5, pair: [0, 0] });
    // slot 25 resets to the first stage
    c.f_15c = 2;
    c.ee_sub_1bbea8(0, &mut h);
    assert_eq!(c.f_15c, 0);
}

#[test]
fn c2d4b78_touch_and_contacts() {
    let mut c = C2d4b78::default();
    let mut h = Rec::default();
    c.ee_sub_1bb918(53, 1); // filter: class 53 (player) enabled
    h.classes.insert(9, 53);
    h.classes.insert(10, 48);
    // stage invalid (no stages, +0x15c = 0) -> nothing
    let t = Touch { other: 9, w_c: 77, flags_40: 0x8_0000_0000, w_44: 1 };
    assert_eq!(c.ee_sub_1badf0(0, &t, &mut h), 0);
    c.ee_sub_1bb950(1);
    c.f_15c = 0;
    assert_eq!(c.ee_sub_1badf0(0, &t, &mut h), 1);
    assert_eq!((c.f_16c, c.f_170), (9, 77));
    assert_eq!(c.ee_sub_1badf0(0, &Touch { other: 10, ..t }, &mut h), 0);
    assert_eq!(c.ee_sub_1badf0(0, &Touch { flags_40: 0, ..t }, &mut h), 1, "wrong flags: taken without latch");
    // contact pass: the flagged player contact latches and notifies both sides
    h.log.clear();
    c.f_16c = 0;
    h.contacts = vec![Contact { rec: 5, flag: true, other: 9 }, Contact { rec: 6, flag: false, other: 9 }, Contact { rec: 7, flag: true, other: 10 }];
    let peers = c.ee_sub_1bb250(0, &|_| None, &mut h);
    assert!(peers.is_empty());
    assert_eq!(c.f_16c, 9);
    assert_eq!(h.log, ["q2a7630", "t1>9", "t9>1"]);
}

#[test]
fn c2caf88_filter_slot60() {
    let mut c = C2caf88::default();
    let mut h = Rec::default();
    c.ee_sub_1320c8(0, Some(false), 0, &mut h);
    assert!(c.alo.so.f_148 >> 34 & 1 != 0 && !c.alo.so.bit35());
    h.classes.insert(3, 53);
    h.classes.insert(7, 48);
    c.ee_sub_1323f0(52); // player subtree
    for k in 0..6 {
        c.ee_sub_1323c8(k);
    }
    assert_eq!(c.f_154.len(), 4);
    assert_eq!(c.ee_sub_132140(0, 3, &mut h), 2);
    assert_eq!(c.ee_sub_132140(0, 7, &mut h), 1);
    c.f_178 = 1;
    assert_eq!(c.ee_sub_132140(0, 3, &mut h), 3);
    // FUN_001f4fc8: neither +0x148 bit 26 nor own vcall +500 -> 0
    assert_eq!(c.ee_sub_132140(0, 7, &mut h), 0);
    c.alo.so.f_148 |= 0x400_0000;
    assert_eq!(c.ee_sub_132140(0, 7, &mut h), 1);
}

#[test]
fn alo_core_fade_and_flags() {
    let mut a = Alo::default();
    let mut h = Rec::default();
    a.so.f_12c = 1.0;
    a.f_130 = -2.0;
    a.ee_sub_1f36e0(4, 0.25, &mut h);
    assert_eq!(a.so.f_12c, 0.5);
    a.ee_sub_1f36e0(4, 0.25, &mut h);
    assert_eq!((a.so.f_12c, a.f_130), (0.0, 0.0));
    assert_eq!(h.log, ["v4c(5)"]);
    a.f_130 = 4.0;
    a.ee_sub_1f36e0(4, 0.5, &mut h);
    assert_eq!((a.so.f_12c, a.f_130), (1.0, 0.0));
    a.ee_sub_1f3b28(4, true, &mut h);
    assert_eq!(a.f_40 & 8, 8);
    a.ee_sub_1f2fe0(4, 0x30, &mut h);
    a.ee_sub_1f3078(4, 0x10, &mut h);
    assert_eq!(a.f_40, 0x28);
    a.ee_sub_1f7918(0x80001, 3);
    assert_eq!((a.f_110[0], a.f_110[19], a.f_110[1]), (3, 3, 0));
    assert!(a.ee_sub_1f5040(4, &mut h));
    h.phys = true;
    assert!(!a.ee_sub_1f5040(4, &mut h));
}
