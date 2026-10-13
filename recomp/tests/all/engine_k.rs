//! WP-B5 tests: template instantiation (FUN_001ca370) and post-load activation (FUN_001a5630). Needs the ISO at
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

use sly2_recomp::engine::classes::{ClassHost, ClassSet, Inst, PORTED};
use std::collections::BTreeMap;

/// Counts calls so the test sees the overrides run.
#[derive(Default)]
struct CountHost {
    calls: usize,
}
impl ClassHost for CountHost {
    fn base(&mut self, _a: u32, _o: usize) {
        self.calls += 1;
    }
    fn ee_sub_1a5e80(&mut self, _o: usize) {
        self.calls += 1;
    }
}

#[test]
fn ported_classes_construct_and_run_120_frames() {
    let (loader, descs) = levels();
    let mut totals: BTreeMap<u16, usize> = BTreeMap::new();
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = World::load(&lv, LoadOptions::default()).unwrap_or_else(|e| panic!("lsn {}: {e}", d.lsn()));
        let mut h = CountHost::default();
        let mut s = ClassSet::build(&w, &mut h);
        s.g.g_2ddf54 = 0xff;
        s.post_load(&mut h);
        for _ in 0..120 {
            s.frame(&mut h);
        }
        for (i, x) in &s.insts {
            *totals.entry(w.objects[*i].class).or_default() += 1;
            if let Inst::C2d9978(c) = x {
                assert_eq!(c.f_148 & 0xffff, 0xecf6);
                assert_eq!(c.f_140, -1);
            }
        }
        assert!(s.insts.is_empty() || h.calls > 0);
    }
    eprintln!("K instance counts: {totals:?}");
    // per-class instance counts over the 43 levels (clones included)
    let expect: BTreeMap<u16, usize> = EXPECT.iter().copied().collect();
    assert_eq!(totals, expect);
    assert!(PORTED.iter().all(|(c, _)| totals.contains_key(c)));
}

const EXPECT: &[(u16, usize)] = &[(10, 3140), (33, 10306), (43, 1315), (48, 11195), (124, 2429)];
