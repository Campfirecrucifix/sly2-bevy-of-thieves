//! WP-B tests: every level's object stream loads to its end. Needs the ISO at
//! "<root>/Sly 2 - Band of Thieves (USA).iso" (level list from the boot level table).
use sly2_recomp::engine::disc::descriptor::Descriptor;
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::{LevelLoader, LoadedLevel};
use sly2_recomp::engine::world::class::CLASSES;
use sly2_recomp::engine::world::loads_b4::B4Load;
use sly2_recomp::engine::world::objects::{LoadData, Payload};
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



/// (lsn, B4-ported objects (each consumed exactly its record size), still-unported objects)
const EXPECT: &[(u32, usize, usize)] = &[
    (335488, 1404, 0),
    (350144, 33, 0),
    (353856, 2161, 0),
    (377568, 1161, 0),
    (388768, 1402, 0),
    (402976, 1028, 0),
    (409312, 948, 0),
    (417504, 1025, 0),
    (427328, 2116, 0),
    (444896, 1343, 0),
    (453760, 1097, 0),
    (465056, 1058, 0),
    (482752, 2180, 0),
    (509056, 1389, 0),
    (521344, 2556, 0),
    (540160, 1513, 0),
    (552768, 559, 0),
    (561312, 2517, 0),
    (578848, 749, 0),
    (589920, 1338, 0),
    (603648, 816, 0),
    (615360, 788, 0),
    (626688, 672, 0),
    (637920, 679, 0),
    (649056, 734, 0),
    (660320, 705, 0),
    (671392, 1922, 0),
    (687488, 1011, 0),
    (698912, 1206, 0),
    (710528, 1260, 0),
    (720608, 1047, 0),
    (730464, 2021, 0),
    (749312, 903, 0),
    (761664, 841, 0),
    (773120, 1005, 0),
    (784544, 604, 0),
    (795296, 778, 0),
    (808192, 2323, 0),
    (823200, 1209, 0),
    (834976, 740, 0),
    (845728, 940, 0),
    (856896, 724, 0),
    (867648, 1682, 0),
];

#[test]
fn b4_loads_consume_record_size() {
    let (loader, descs) = levels();
    let mut rows = Vec::new();
    let mut tot: std::collections::BTreeMap<u16, usize> = Default::default();
    let mut class6 = (0usize, 0usize);
    for d in descs {
        // World::load errors (SizeMismatch / Eof) if a ported load does not consume its record
        let (_lv, w) = load(&loader, d);
        let b4 = w.objects.iter().filter(|o| matches!(o.payload, Payload::Loaded(LoadData::B4(_)))).count();
        for o in &w.objects {
            if let Payload::Loaded(LoadData::B4(b)) = &o.payload {
                if let B4Load::Class6 { in_place, .. } = **b {
                    if in_place { class6.0 += 1 } else { class6.1 += 1 }
                }
            }
        }
        assert!(!w.objects.iter().any(|o| o.payload == Payload::Handler1c9b08));
        let unp: usize = w.unported_classes().values().sum();
        for (c, n) in w.unported_classes() { *tot.entry(c).or_default() += n; }
        rows.push((d.lsn(), b4, unp));
    }
    for (c, n) in &tot { println!("UNP class {c} load {:#x} n {n}", CLASSES[*c as usize].load); }
    println!("class-6 records: in place {} / template {}", class6.0, class6.1);
    if std::env::var("ENGINE_B4_BLESS").is_ok() || EXPECT.is_empty() {
        for r in &rows { println!("BLESS     {r:?},"); }
        assert!(!EXPECT.is_empty(), "bless EXPECT");
        return;
    }
    assert_eq!(rows, EXPECT.to_vec());
}
