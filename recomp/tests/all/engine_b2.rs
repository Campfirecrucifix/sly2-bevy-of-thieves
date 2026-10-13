//! WP-B tests: every level's object stream loads to its end. Needs the ISO at
//! "<root>/Sly 2 - Band of Thieves (USA).iso" (level list from the boot level table).
use sly2_recomp::engine::disc::descriptor::Descriptor;
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::{LevelLoader, LoadedLevel};
use sly2_recomp::engine::world::class::CLASSES;
use sly2_recomp::engine::world::objects::Payload;
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

fn load(loader: &LevelLoader, d: Descriptor) -> (LoadedLevel, World) {
    let lv = crate::common::load_blocking(&loader, d).unwrap();
    let w = World::load(&lv, LoadOptions::default()).unwrap_or_else(|e| panic!("lsn {}: {e}", d.lsn()));
    (lv, w)
}


/// (lsn, objects whose class load is ported and consumed exactly its record size, still-unported objects)
/// WP-B4: re-blessed (objects created by loads_b4: class-6 records + nested lists, 53/54 ALOs).
const EXPECT: &[(u32, usize, usize)] = &[
    (335488, 3320, 0),
    (350144, 43, 0),
    (353856, 4808, 0),
    (377568, 2637, 0),
    (388768, 2952, 0),
    (402976, 2374, 0),
    (409312, 2244, 0),
    (417504, 2417, 0),
    (427328, 4528, 0),
    (444896, 2911, 0),
    (453760, 2616, 0),
    (465056, 2915, 0),
    (482752, 5155, 0),
    (509056, 3198, 0),
    (521344, 5267, 0),
    (540160, 3343, 0),
    (552768, 1632, 0),
    (561312, 5253, 0),
    (578848, 1890, 0),
    (589920, 3117, 0),
    (603648, 2089, 0),
    (615360, 2005, 0),
    (626688, 1813, 0),
    (637920, 1831, 0),
    (649056, 1878, 0),
    (660320, 1860, 0),
    (671392, 4258, 0),
    (687488, 2423, 0),
    (698912, 2693, 0),
    (710528, 2878, 0),
    (720608, 2398, 0),
    (730464, 4601, 0),
    (749312, 2259, 0),
    (761664, 2206, 0),
    (773120, 2479, 0),
    (784544, 1758, 0),
    (795296, 2142, 0),
    (808192, 4530, 0),
    (823200, 2772, 0),
    (834976, 1837, 0),
    (845728, 2200, 0),
    (856896, 1846, 0),
    (867648, 3512, 0),
];

#[test]
#[ignore = "heavy"]
fn ported_loads_consume_record_size() {
    let (loader, descs) = levels();
    let mut rows = Vec::new();
    let mut tot: std::collections::BTreeMap<u16, usize> = Default::default();
    for d in descs {
        // World::load errors (SizeMismatch / Eof) if a ported load does not consume its record
        let (_lv, w) = load(&loader, d);
        let loaded = w.objects.iter().filter(|o| matches!(o.payload, Payload::Loaded(_))).count();
        let unp: usize = w.unported_classes().values().sum();
        for (c, n) in w.unported_classes() { *tot.entry(c).or_default() += n; }
        rows.push((d.lsn(), loaded, unp));
    }
    for (c, n) in &tot { println!("UNP class {c} load {:#x} n {n}", CLASSES[*c as usize].load); }
    if std::env::var("ENGINE_B2_BLESS").is_ok() || EXPECT.is_empty() {
        for r in &rows { println!("BLESS     {r:?},"); }
        assert!(!EXPECT.is_empty(), "bless EXPECT");
        return;
    }
    assert_eq!(rows, EXPECT.to_vec());
}
