//! WP-B3 tests: ALO load chain (FUN_001f31e0) consumes exactly each record; mesh counts. Needs the ISO at
//! "<root>/Sly 2 - Band of Thieves (USA).iso" (level list from the boot level table).
use sly2_recomp::engine::disc::descriptor::Descriptor;
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::{LevelLoader, LoadedLevel};
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


use sly2_recomp::engine::world::objects::LoadData;

/// (lsn, ALO objects loaded, parsed meshes, mesh vertices, ALO children)
/// WP-B4: re-blessed (objects created by loads_b4: class-6 records + nested lists, 53/54 ALOs).
// WP-B5: last column (ALO children) re-blessed: FUN_001ca370 reparents template copies to
// the class-6 record's owner (vtable +0x54), so owning ALOs gain those children.
const EXPECT: &[(u32, usize, usize, usize, usize)] = &[
    (335488, 421, 2183, 163364, 2099),
    (350144, 2, 2, 61, 0),
    (353856, 439, 2388, 175084, 2672),
    (377568, 246, 1514, 96374, 1761),
    (388768, 295, 1687, 123208, 1901),
    (402976, 237, 1205, 83175, 1630),
    (409312, 200, 945, 66270, 1658),
    (417504, 203, 1085, 62118, 1553),
    (427328, 385, 2044, 175339, 2273),
    (444896, 233, 1343, 100093, 1854),
    (453760, 227, 1185, 78070, 1815),
    (465056, 202, 1105, 84336, 2239),
    (482752, 438, 2041, 168910, 3023),
    (509056, 314, 1371, 101825, 2125),
    (521344, 456, 2094, 158461, 2644),
    (540160, 294, 1411, 87821, 2065),
    (552768, 124, 582, 44228, 1227),
    (561312, 434, 2167, 161594, 2694),
    (578848, 142, 686, 49177, 1326),
    (589920, 322, 1526, 102490, 2172),
    (603648, 171, 872, 67550, 1531),
    (615360, 180, 748, 55254, 1385),
    (626688, 152, 656, 52048, 1372),
    (637920, 161, 683, 50425, 1366),
    (649056, 155, 663, 51574, 1391),
    (660320, 150, 648, 47580, 1348),
    (671392, 350, 2047, 135265, 2431),
    (687488, 217, 965, 61624, 1800),
    (698912, 263, 1167, 83644, 1981),
    (710528, 276, 1228, 88521, 2138),
    (720608, 230, 1112, 80997, 1740),
    (730464, 427, 2232, 142921, 2410),
    (749312, 195, 869, 56961, 1496),
    (761664, 192, 940, 61552, 1472),
    (773120, 196, 936, 60249, 1597),
    (784544, 138, 631, 44708, 1226),
    (795296, 167, 822, 55201, 1519),
    (808192, 458, 2286, 153125, 2767),
    (823200, 285, 1336, 93494, 2025),
    (834976, 175, 731, 52395, 1376),
    (845728, 197, 949, 63769, 1652),
    (856896, 177, 746, 56120, 1388),
    (867648, 470, 1755, 159065, 2778),
];

#[test]
#[ignore = "heavy"]
fn alo_loads_consume_record_size() {
    let (loader, descs) = levels();
    let mut rows = Vec::new();
    for d in descs {
        // World::load errors (SizeMismatch / Eof) if an ALO load does not consume its record
        let (_lv, w) = load(&loader, d);
        let (mut n, mut m, mut v, mut c) = (0, 0, 0, 0);
        for o in &w.objects {
            if let Payload::Loaded(LoadData::Alo { alo, .. }) = &o.payload {
                n += 1;
                let (a, b) = alo.stats();
                m += a;
                v += b;
                c += o.children.len();
            }
        }
        rows.push((d.lsn(), n, m, v, c));
    }
    if std::env::var("ENGINE_B3_BLESS").is_ok() || EXPECT.is_empty() {
        for r in &rows { println!("BLESS     {r:?},"); }
        assert!(!EXPECT.is_empty(), "bless EXPECT");
        return;
    }
    assert_eq!(rows, EXPECT.to_vec());
}
