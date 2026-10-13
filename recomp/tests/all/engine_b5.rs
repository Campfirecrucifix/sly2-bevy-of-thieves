//! WP-B5 tests: template instantiation (FUN_001ca370) and post-load activation (FUN_001a5630). Needs the ISO at
//! "<root>/Sly 2 - Band of Thieves (USA).iso" (level list from the boot level table).
use sly2_recomp::engine::disc::descriptor::Descriptor;
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::{LevelLoader, LoadedLevel};
use sly2_recomp::engine::world::loads_b4::B4Load;
use sly2_recomp::engine::world::objects::{LoadData, Payload};
use sly2_recomp::engine::world::loads_b5::live_objects;
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



/// (lsn, template records, instantiated copy objects, nested objects, entries with copies
///  not yet loaded at instantiation, overflowing records, active objects after load)
const EXPECT: &[(u32, usize, usize, usize, usize, usize, usize)] = &[
    (335488, 593, 681, 1, 0, 0, 93),
    (350144, 1, 31, 0, 0, 0, 4),
    (353856, 545, 600, 20, 0, 0, 307),
    (377568, 408, 460, 0, 0, 0, 101),
    (388768, 515, 625, 0, 0, 0, 107),
    (402976, 348, 387, 0, 0, 0, 103),
    (409312, 275, 316, 0, 0, 0, 107),
    (417504, 312, 364, 0, 0, 0, 53),
    (427328, 623, 783, 77, 0, 0, 309),
    (444896, 563, 619, 4, 0, 0, 103),
    (453760, 359, 451, 1, 0, 0, 112),
    (465056, 355, 396, 0, 0, 0, 114),
    (482752, 625, 716, 16, 0, 0, 279),
    (509056, 512, 671, 1, 0, 0, 107),
    (521344, 585, 741, 10, 0, 1, 329),
    (540160, 600, 715, 1, 0, 0, 227),
    (552768, 90, 145, 0, 0, 0, 49),
    (561312, 464, 599, 5, 0, 0, 430),
    (578848, 213, 300, 0, 0, 0, 107),
    (589920, 513, 793, 0, 0, 1, 162),
    (603648, 212, 339, 0, 0, 0, 109),
    (615360, 188, 385, 0, 0, 0, 137),
    (626688, 139, 179, 0, 0, 0, 110),
    (637920, 145, 239, 0, 0, 0, 108),
    (649056, 177, 326, 0, 0, 0, 110),
    (660320, 165, 223, 0, 0, 0, 116),
    (671392, 598, 805, 9, 0, 0, 199),
    (687488, 408, 466, 0, 0, 0, 101),
    (698912, 427, 542, 12, 0, 0, 201),
    (710528, 419, 536, 0, 0, 0, 218),
    (720608, 381, 494, 2, 0, 0, 109),
    (730464, 666, 732, 37, 0, 0, 229),
    (749312, 261, 307, 4, 0, 0, 118),
    (761664, 236, 315, 3, 0, 0, 91),
    (773120, 292, 440, 4, 0, 0, 106),
    (784544, 120, 157, 0, 0, 0, 86),
    (795296, 193, 240, 0, 0, 0, 106),
    (808192, 683, 984, 12, 0, 0, 330),
    (823200, 407, 457, 2, 0, 1, 104),
    (834976, 175, 217, 2, 0, 0, 52),
    (845728, 282, 324, 2, 0, 0, 117),
    (856896, 162, 199, 2, 0, 0, 45),
    (867648, 770, 1075, 112, 0, 0, 127),
];

#[test]
#[ignore = "heavy"]
fn b5_templates_and_activation_all_levels() {
    let (loader, descs) = levels();
    let mut rows = Vec::new();
    for d in descs {
        let (_lv, w) = load(&loader, d);
        let mut recs = 0;
        for (i, o) in w.objects.iter().enumerate() {
            if let Payload::Loaded(LoadData::B4(b)) = &o.payload {
                if let B4Load::Class6 { in_place: false, template, .. } = **b {
                    recs += 1;
                    // every template record resolves its template (FUN_001cb770 != -1)
                    let t = w.template_instances.iter().find(|t| t.record == i).expect("FUN_001ca370 call");
                    let e = t.entry.unwrap_or_else(|| panic!("lsn {} template {template} not in directory", d.lsn()));
                    assert_eq!(w.entries[e].dir.c, template);
                    assert_eq!(t.key, ((e as u32) << 16) | (t.copy as u32 & 0xffff));
                    assert!(o.owner.is_none(), "record bookkeeping Lo has no parent");
                }
            }
        }
        assert_eq!(recs, w.template_instances.len());
        let (mut objs, mut nested, mut early, mut over) = (0, 0, 0, 0);
        for t in &w.template_instances {
            objs += t.objects.len();
            nested += t.nested.len();
            let e = t.entry.unwrap();
            if w.entries[e].start_offset > w.objects[t.record].payload_offset {
                early += 1;
            }
            if t.overflow {
                let en = &w.entries[e];
                println!("OVER lsn {} entry {e} n {} a {} m {} copy {} inst {} name {}", d.lsn(), en.dir.n, en.dir.a, en.objects.len(), t.copy, en.instances.len(), en.dir.name());
                over += 1;
            }
            for &o in &t.objects {
                // correct parent: the record's owner (+0x54), key pushed (+0xcc), placed (+0x12c)
                // an object handed out again later (template of a template) moves again
                let last = w.template_instances.iter().rev().find(|u| u.objects.contains(&o)).unwrap();
                assert_eq!(w.objects[o].owner, last.owner, "lsn {} obj {o}", d.lsn());
                if let Some(p) = last.owner {
                    assert!(w.objects[p].children.contains(&o));
                }
                assert!(w.life.keys[o].contains(&t.key));
                assert!(w.life.placed.contains_key(&o));
                assert!(w.entries[e].instances.contains(&o));
            }
        }
        // copies used per entry never exceed +0x0c - 1 and oids are recorded
        for e in &w.entries {
            for (k, &oid) in e.inst_oids.iter().enumerate() {
                if oid != -1 {
                    assert!((k as i32) < e.load_count);
                }
            }
        }
        // activation: every a == 0 entry instance is active; template entries' copies are not
        // unless an a == 0 entry collected them
        for e in w.entries.iter().filter(|e| e.dir.instantiate_now()) {
            for &o in &e.instances {
                // FUN_001ca370 removes (+0x44) objects it hands out; they stay inactive unless
                // they end up in an a == 0 entry's top-level collector
                let moved = w.template_instances.iter().any(|t| t.objects.contains(&o));
                assert!(moved || w.is_active(o));
            }
        }
        // list invariants: active objects are exactly the list tails from the first-active ptr
        let mut in_lists = 0;
        for l in w.life.lists.values().chain(std::iter::once(&w.life.root_list)) {
            for &o in l.active() {
                assert!(w.is_active(o), "lsn {} obj {o} after head but inactive", d.lsn());
            }
            in_lists += l.active().len();
        }
        let live = live_objects(&w).count();
        assert_eq!(in_lists, live);
        rows.push((d.lsn(), recs, objs, nested, early, over, live));
    }
    if std::env::var("ENGINE_B5_BLESS").is_ok() || EXPECT.is_empty() {
        for r in &rows {
            println!("BLESS     {r:?},");
        }
        assert!(!EXPECT.is_empty(), "bless EXPECT");
        return;
    }
    assert_eq!(rows, EXPECT.to_vec());
    // B4 counted 16005 template (FUN_001ca370) records on the disc
    assert_eq!(rows.iter().map(|r| r.1).sum::<usize>(), 16005);
}

// ---------------------------------------------------------------- WP-B5 (world loader gaps): alias table

use sly2_recomp::engine::splice::eval::Vm;
use sly2_recomp::engine::splice::frame::{lookup, FrameRef};
use sly2_recomp::engine::splice::serialize::read_script_block;
use sly2_recomp::engine::splice::value::{list_items, Ref};
use std::collections::{BTreeSet, HashSet};

/// The alias table (FUN_001c99d8 key tree + world+0x1838 buckets, built by FUN_001ca118):
/// a template copy whose entry has exactly one object is found by the copy's oid (the class-6
/// record's oid); the record itself is never returned by an oid search.
#[test]
fn b5_alias_table() {
    let (l, descs) = levels();
    let (mut single, mut single_found, mut multi, mut multi_found, mut nodes) = (0, 0, 0, 0, 0);
    for d in descs {
        let (_, w) = load(&l, d);
        let t = &w.life.alias;
        nodes += t.nodes.len() - 1;
        for (i, n) in t.nodes.iter().enumerate().skip(1) {
            // FUN_001c99d8 never creates two children with the same key
            let p = &t.nodes[n.parent.unwrap()];
            assert_eq!(p.children.iter().filter(|&&c| t.nodes[c].key == n.key).count(), 1);
            assert!(t.bucket[&n.oid].contains(&i));
            if let Some(o) = n.object {
                // +0x10 is only set for single-object entries
                assert_eq!(w.entries[(n.key >> 16) as usize].objects.len(), 1, "lsn {}", d.lsn());
                assert!(w.life.keys[o].contains(&n.key));
            }
        }
        for ti in &w.template_instances {
            assert!(!w.find_oid(w.objects[ti.record].oid).contains(&ti.record), "lsn {}: record returned", d.lsn());
            if ti.oid == -1 || ti.objects.is_empty() {
                continue;
            }
            let found = w.find_oid(ti.oid).contains(&ti.objects[0]);
            if ti.objects.len() == 1 {
                single += 1;
                single_found += found as usize;
            } else {
                multi += 1;
                multi_found += found as usize;
            }
        }
    }
    eprintln!("B5 alias: nodes {nodes}, single-object copies {single_found}/{single} found, multi {multi_found}/{multi}");
    assert_eq!((nodes, single, single_found, multi, multi_found), (16455, 14855, 14855, 1147, 0));
}

fn walk_calls(r: &Ref, g: &FrameRef, out: &mut Vec<(u32, i32)>, seen: &mut HashSet<usize>) {
    let Ref::Pair(p) = r else { return };
    if !seen.insert(std::rc::Rc::as_ptr(p) as usize) {
        return;
    }
    let items = list_items(Some(p.clone()));
    let bif = match items.first() {
        Some(Ref::Bifk(k)) => Some(*k),
        Some(Ref::Symid(s)) => match lookup(g, *s, false) {
            Some(Ref::Bifk(k)) => Some(k),
            _ => None,
        },
        _ => None,
    };
    // BIF 112 (FUN_0015e958) / 115 (FUN_0015e958 mode 4) with a literal oid
    if let (Some(b @ (112 | 115)), Some(Ref::S32(o))) = (bif, items.get(1)) {
        out.push((b, *o));
    }
    for x in &items {
        walk_calls(x, g, out, seen);
    }
}

/// Every oid level scripts look up literally (BIF 112/115) resolves in its level, except
/// references to objects of other levels (episode-wide script entries such as
/// `f_nightclub_macro` name every level's objects) and 6 oids no level carries.
#[test]
fn b5_script_oids_resolve() {
    let (l, descs) = levels();
    let vm = Vm::new();
    let mut per = Vec::new();
    for d in descs {
        let (lv, w) = load(&l, d);
        let mut calls = Vec::new();
        for e in &w.entries {
            let Some(data) = lv.data.get(e.start_offset as usize + 12..) else { continue };
            let Ok((blk, _)) = read_script_block(data, e.sections.ext) else { continue };
            let mut seen = HashSet::new();
            for p in blk.iter().flatten() {
                walk_calls(&Ref::Pair(p.clone()), &vm.global, &mut calls, &mut seen);
            }
        }
        let have: HashSet<i32> = calls.iter().map(|c| c.1).filter(|&o| !w.find_oid(o).is_empty()).collect();
        let all: HashSet<i32> = w.objects.iter().map(|o| o.oid).collect();
        per.push((calls, have, all));
    }
    let (mut found, mut other, mut nowhere) = (0, 0, BTreeSet::new());
    let mut nowhere_refs = 0;
    for (i, (calls, have, _)) in per.iter().enumerate() {
        for &(_, o) in calls {
            if have.contains(&o) {
                found += 1;
            } else if per.iter().enumerate().any(|(j, p)| j != i && p.2.contains(&o)) {
                other += 1;
            } else {
                nowhere.insert(o);
                nowhere_refs += 1;
            }
        }
    }
    eprintln!("B5 script oids: found {found}, other level {other}, nowhere {nowhere_refs} refs {nowhere:?}");
    // before B5 (template records in the oid bucket, no alias bucket): found 13479, of which
    // 8 = BIF 112 on oid 6786, a 4-object template copy (no alias object in the original)
    assert_eq!((found, other, nowhere_refs), (13471, 3976, 9));
    assert_eq!(nowhere.into_iter().collect::<Vec<_>>(), [17204, 30821, 76923, 105257, 105264, 123705]);
}
