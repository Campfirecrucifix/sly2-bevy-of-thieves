//! WP-K31 tests: class 37 C2cb9f8 (guard loot holder) and the class-35 slot 21 / 22 list entry on the real
//! levels (build, load-post, 300 frames).
use sly2_recomp::engine::classes::k10::{self, K10};
use sly2_recomp::engine::classes::k31::{self, C2cb9f8};
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

/// Recording host: world lookups through `EngineHost`, deferred calls logged.
#[derive(Default)]
struct Rec {
    e: EngineHost,
    log: Vec<String>,
}
impl ClassHost for Rec {
    fn ee_sub_15e938(&mut self, kind: i32, oid: i32, ctx: u32) -> u32 {
        self.e.ee_sub_15e938(kind, oid, ctx)
    }
    fn ee_sub_132b10(&mut self, obj: u32, class: i32) -> u32 {
        self.e.ee_sub_132b10(obj, class)
    }
    fn ee_sub_149ab8(&mut self, obj: usize, pmf: u32, arg: i32) {
        self.log.push(format!("149ab8 {obj} {pmf:#x} {arg}"));
    }
}

/// Class 37 on every level that has it: slot 6 defaults / fields, slot 14 index counter (DAT_003e1acc in
/// load order), slot 22 list entry + FUN_00149ab8(DAT_002a4bb0), class-35 points listed too, 300 frames,
/// then slot 21 leaves the list.
#[test]
#[ignore = "heavy"]
fn k31_c37_guard_loot_holder_all_levels() {
    let (loader, descs) = levels();
    let mut n37 = 0;
    let mut n35 = 0;
    let mut fed = 0;
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let mut h = Rec::default();
        h.e.world = Some(w.clone());
        let mut s = ClassSet::build(&w, &mut h);
        s.post_load(&mut h);
        s.g.dt = 1.0 / 60.0;
        let objs: Vec<usize> = w.objects.iter().enumerate().filter(|(_, o)| o.class == 37).map(|(i, _)| i).collect();
        assert_eq!(s.k31.iter().map(|x| x.0).collect::<Vec<_>>(), objs);
        let l = s.g.k10_lists.get(&k10::L_3E1414).cloned().unwrap_or_default();
        for (k, (i, c)) in s.k31.iter().enumerate() {
            assert_eq!(c.f_180, k as i32, "guard index in load order");
            assert!(c.base.f_fc != 0 && c.base.f_38 != 1e10);
            assert!(l.contains(&(*i as u32 + 1)));
            assert!(h.log.contains(&format!("149ab8 {i} {:#x} 0", k31::PMF_2A4BB0)));
            println!("class 37 object {i}: +0x184 {} +0x188 {} +0x18c {} +0x190 {} +0x194 {:#x} m1168 {:?}", c.f_184, c.f_188, c.f_18c, c.f_190, c.f_194, c.m_1168);
            n37 += 1;
            fed += (c.f_190 != 0 || c.f_184 != 700) as usize;
        }
        assert_eq!(s.g.g_3e1acc, s.k31.len() as i32);
        for (i, x) in &s.k10 {
            if let Inst::K10(k) = x {
                if let K10::C2d1ee8(_) = k.as_ref() {
                    assert!(l.contains(&(*i as u32 + 1)), "class 35 slot 22");
                    n35 += 1;
                }
            }
        }
        let before: Vec<C2cb9f8> = s.k31.iter().map(|x| x.1.clone()).collect();
        for _ in 0..300 {
            s.g.t += s.g.dt;
            s.frame(&mut h);
        }
        assert_eq!(s.k31.len(), before.len());
        if let Some(&(i, _)) = s.k31.first() {
            s.unload_k31(i, &mut h);
            assert!(!s.g.k10_lists[&k10::L_3E1414].contains(&(i as u32 + 1)), "slot 21 leaves 0x3e1414");
            assert!(s.k31.iter().all(|x| x.0 != i));
        }
    }
    println!("class 37 objects {n37}, class 35 points {n35}");
    assert!(n37 > 0 && n35 > 0);
    assert!(fed > 0, "fields 1167-1173 fed on some guard");
}

/// Slot 6 defaults (+0x184 = 700, +0x18c = 15.0, +0x188 = -1) and the field setters.
#[test]
fn k31_c37_defaults() {
    let mut h = Rec::default();
    let mut c = C2cb9f8::default();
    c.ee_sub_13ceb8(0, &mut h);
    assert_eq!((c.f_184, c.f_18c, c.f_188), (700, 15.0, -1));
    assert_eq!(c.base.f_110, 200.0);
    assert_eq!(c.base.f_f0, 1);
    assert_eq!(k31::SLOT5_37, 0x2badcc);
}
