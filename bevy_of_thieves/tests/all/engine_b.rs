//! WP-B tests: every level's object stream loads to its end. Needs the ISO at
//! "<root>/Sly 2 - Band of Thieves (USA).iso" (level list from the boot level table).
use sly2_recomp::engine::disc::descriptor::Descriptor;
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::{LevelLoader, LoadedLevel};
use sly2_recomp::engine::world::class::{self, CLASSES};
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

#[test]
fn class_table() {
    assert_eq!(CLASSES.len(), 199);
    for (i, c) in CLASSES.iter().enumerate() {
        assert_eq!(c.id as usize, i);
        assert!(c.id_end as usize > i && c.id_end <= 199);
        if c.parent >= 0 {
            let p = &CLASSES[c.parent as usize];
            // DFS ids: child subtree nested in the parent's
            assert!(p.id < c.id && c.id_end <= p.id_end);
        }
    }
    assert!(class::ee_sub_15e988(7, 6) && class::ee_sub_15e988(131, 2) && !class::ee_sub_15e988(2, 6));
    assert_eq!(CLASSES[2].load, class::LOAD_LO);
}

/// (lsn, objects created, entries, end offset, tail bytes, distinct classes)
/// WP-B4: re-blessed (objects created by loads_b4: class-6 records + nested lists, 53/54 ALOs).
const EXPECT: &[(u32, usize, usize, u32, u32, usize)] = &[
    (335488, 3320, 239, 17420048, 934, 67),
    (350144, 43, 3, 4084032, 567, 6),
    (353856, 4808, 287, 18744160, 2177, 84),
    (377568, 2637, 172, 11400864, 1995, 64),
    (388768, 2952, 213, 13390048, 2145, 73),
    (402976, 2374, 161, 10016368, 2003, 63),
    (409312, 2244, 182, 8586016, 2007, 62),
    (417504, 2417, 133, 9071520, 1988, 60),
    (427328, 4528, 243, 19254192, 1893, 82),
    (444896, 2911, 194, 13360992, 1765, 68),
    (453760, 2616, 195, 10467440, 1928, 69),
    (465056, 2915, 162, 11148048, 1846, 72),
    (482752, 5155, 287, 20132352, 1847, 86),
    (509056, 3198, 227, 13033648, 1878, 81),
    (521344, 5267, 284, 18933168, 1921, 92),
    (540160, 3343, 197, 12090944, 1834, 78),
    (552768, 1632, 85, 6574240, 2000, 50),
    (561312, 5253, 251, 18501120, 2468, 88),
    (578848, 1890, 127, 7305120, 2092, 62),
    (589920, 3117, 206, 12510496, 2207, 77),
    (603648, 2089, 139, 9194512, 2294, 64),
    (615360, 2005, 125, 7956992, 2126, 60),
    (626688, 1813, 118, 7692144, 2097, 62),
    (637920, 1831, 116, 7463744, 2103, 62),
    (649056, 1878, 127, 7745568, 2101, 60),
    (660320, 1860, 126, 7205664, 2100, 61),
    (671392, 4258, 242, 16161552, 1660, 80),
    (687488, 2423, 157, 9012208, 1766, 68),
    (698912, 2693, 191, 11147888, 1710, 68),
    (710528, 2878, 207, 11605472, 1632, 71),
    (720608, 2398, 179, 10876960, 1620, 65),
    (730464, 4601, 281, 17946720, 2351, 94),
    (749312, 2259, 146, 8292944, 2125, 69),
    (761664, 2206, 147, 8788352, 2201, 72),
    (773120, 2479, 172, 8763872, 2163, 70),
    (784544, 1758, 94, 6544880, 2110, 60),
    (795296, 2142, 145, 8312704, 2150, 63),
    (808192, 4530, 326, 17099936, 1620, 88),
    (823200, 2772, 214, 11092160, 1624, 76),
    (834976, 1837, 120, 7661808, 1603, 57),
    (845728, 2200, 158, 8621600, 1625, 63),
    (856896, 1846, 112, 7593248, 1768, 58),
    (867648, 3512, 263, 16034752, 1623, 70),
];

#[test]
#[ignore = "heavy"]
fn all_levels_load_object_stream() {
    let (loader, descs) = levels();
    let mut rows = Vec::new();
    let mut loaded_total = 0;
    for d in descs {
        let (lv, w) = load(&loader, d);
        // consumed every directory entry; only the never-read tail remains (< 4 KB)
        assert_eq!(w.entries.len(), lv.header.directory.len());
        assert!(w.tail_len < 4096, "lsn {} tail {}", d.lsn(), w.tail_len);
        assert_eq!(w.end_offset % 16, 0);
        // ported loads consumed exactly their recorded size (checked in the loader); count them
        let loaded = w.objects.iter().filter(|o| matches!(o.payload, Payload::Loaded(_))).count();
        loaded_total += loaded;
        // n copies made per entry
        for e in &w.entries {
            assert_eq!(e.instances.len(), e.objects.len() * e.dir.n.max(1) as usize);
        }
        let counts = w.class_counts();
        let total: usize = counts.values().sum();
        let name = lv.header.directory.first().map(|e| e.name()).unwrap_or_default();
        println!(
            "lsn {:7} {:40} entries {:3} objects {:5} (loaded {:4}) end {:9} tail {:5} classes {:?}",
            d.lsn(), name, w.entries.len(), total, loaded, w.end_offset, w.tail_len, counts
        );
        rows.push((d.lsn(), total, w.entries.len(), w.end_offset, w.tail_len, counts.len()));
    }
    assert!(loaded_total > 0, "no ported class load exercised");
    if std::env::var("ENGINE_B_BLESS").is_ok() {
        for r in &rows {
            println!("BLESS     {r:?},");
        }
        return;
    }
    assert_eq!(rows, EXPECT.to_vec());
}

#[test]
fn mission_filter_changes_counts() {
    let (loader, descs) = levels();
    // nightclub exterior: mission lists 2..7 are present
    let d = descs.into_iter().find(|d| d.lsn() == 353856).unwrap();
    let lv = crate::common::load_blocking(&loader, d).unwrap();
    let all = World::load(&lv, LoadOptions { mission: -1 }).unwrap();
    let m2 = World::load(&lv, LoadOptions { mission: 2 }).unwrap();
    assert_eq!(all.end_offset, m2.end_offset);
    assert!(m2.objects.len() < all.objects.len());
    // lookups
    let o = &all.objects[all.entries[0].objects[0]];
    assert!(all.find_oid(o.oid).contains(&all.entries[0].objects[0]));
    assert!(all.entry_by_name(&all.entries[1].dir.name()).is_some());
}
