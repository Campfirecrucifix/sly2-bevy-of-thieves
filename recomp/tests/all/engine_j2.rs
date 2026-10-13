//! WP-J2 headless tests: spawn alias list / FUN_00227fe8, Splice host BIFs, startup FUN_001a7970.

use sly2_recomp::engine::audio::emitter::EmitterSystem;
use sly2_recomp::engine::disc::descriptor;
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::game::clock::Clock;
use sly2_recomp::engine::game::main_loop::STARTUP_FUNCS;
use sly2_recomp::engine::game::save::GameData;
use sly2_recomp::engine::game::spawn::{self, SpawnAliases, MAX_ALIASES};
use sly2_recomp::engine::game::splice_host::GameSpliceHost;
use sly2_recomp::engine::game::startup::{self, StartupHost, STARTUP_TABLE};
use sly2_recomp::engine::splice::value::Ref;
use sly2_recomp::engine::splice::SpliceHost;
use sly2_recomp::engine::world::{LoadOptions, World};
use std::path::PathBuf;
use std::sync::Arc;

fn iso() -> Option<Arc<IsoImage>> {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../Sly 2 - Band of Thieves (USA).iso");
    if !p.exists() {
        p = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../Sly 2 - Band of Thieves (USA).iso"));
    }
    IsoImage::open(&p).ok().map(Arc::new)
}

#[test]
fn alias_list_caps_at_four() {
    let mut a = SpawnAliases::default();
    for i in 0..6 {
        a.ee_sub_228988(7, 100 + i);
    }
    assert_eq!(a.get(7), &[100, 101, 102, 103][..MAX_ALIASES]);
    assert!(a.get(8).is_empty());
}

#[test]
fn splice_host_time_and_void_write() {
    let mut em = EmitterSystem::default();
    let mut data = GameData::default();
    let mut h = GameSpliceHost { t: 12.5, cm: None, emitters: &mut em, vars: None, data: &mut data };
    assert!(matches!(h.ref_op_current_time(&[]).unwrap(), Ref::F32(t) if t == 12.5));
    assert!(matches!(h.ref_op_start_rumble(&[]).unwrap(), Ref::Void));
    assert!(matches!(h.ref_op_stop_sound(&[]).unwrap(), Ref::Nil));
    assert!(h.ref_op_near_clip_center(&[]).is_err());
}

#[derive(Default)]
struct Rec {
    bars: Vec<u32>,
    pad: u32,
    sound: u32,
}
impl StartupHost for Rec {
    fn loading_bar(&mut self, r: u32) {
        self.bars.push(r);
    }
    fn init_pad(&mut self) {
        self.pad += 1;
    }
    fn init_sound(&mut self, _g: &startup::SoundGlobals) {
        self.sound += 1;
    }
}

#[test]
fn startup_runs_table_once() {
    assert_eq!(STARTUP_TABLE.len(), STARTUP_FUNCS as usize);
    let mut r = Rec::default();
    let mut clk = Clock::default();
    startup::ee_str_0813_0032_1a7970(&mut clk, 0.0, &mut r);
    assert_eq!((r.pad, r.sound), (1, 1));
    assert_eq!(r.bars.last(), Some(&0));
    assert!(r.bars.windows(2).all(|w| w[0] > w[1]));
}

/// Real disc: every level-table level has spawn objects and FUN_00227fe8 finds each by its own oid.
#[test]
fn spawn_lookup_on_disc_levels() {
    let Some(iso) = iso() else { return eprintln!("ISO not found; skipped") };
    let table = descriptor::static_table(&iso.read_file("SCUS_973.16").unwrap()).unwrap();
    let loader = LevelLoader::new(iso, Default::default());
    let boot = crate::common::load_blocking(&loader, table[22]).unwrap();
    let mut checked = 0;
    for e in boot.header.level_table.entries.iter().take(6).filter(|e| e.desc.size() != 0) {
        let l = crate::common::load_blocking(&loader, e.desc).unwrap();
        let w = World::load(&l, LoadOptions::default()).unwrap();
        let (n, c) = spawn::ee_sub_15e9c8(&w, None, spawn::CLASS_SPAWN);
        assert!(c.len() <= n.min(spawn::MAX_CANDIDATES));
        for &i in &c {
            let found = spawn::ee_sub_227fe8(&w, &SpawnAliases::default(), w.objects[i].oid, -1).unwrap();
            assert_eq!(w.objects[found].oid, w.objects[i].oid);
        }
        assert!(!spawn::active_objects(&w).is_empty());
        checked += 1;
    }
    assert!(checked > 0);
}
