//! WP-P20: Bentley / Murray player layer (po selection FUN_0020e890, k18 activation, po_call from the
//! shadow JT).

use sly2_recomp::engine::classes::plugin::EngineHost;
use sly2_recomp::engine::classes::{ClassHost, ClassSet};
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::player::p20::{apply_po, ee_sub_20e890_po, PoCurrent, PoHost, CLASS_BENTLEY, CLASS_MURRAY, CLASS_SLY};
use sly2_recomp::engine::player::Jt;
use sly2_recomp::engine::world::{LoadOptions, World};
use std::sync::Arc;

fn tables() -> (LevelLoader, Vec<sly2_recomp::engine::disc::descriptor::Descriptor>) {
    let iso = Arc::new(IsoImage::open(concat!(env!("CARGO_MANIFEST_DIR"), "/../Sly 2 - Band of Thieves (USA).iso")).expect("ISO"));
    let t = sly2_recomp::engine::disc::descriptor::static_table(&iso.read_file("SCUS_973.16").unwrap()).unwrap();
    let l = LevelLoader::new(iso, ModOverlay::default());
    let boot = crate::common::load_blocking(&l, t[25]).unwrap();
    let d = boot.header.level_table.entries.iter().map(|e| e.desc).collect();
    (l, d)
}

fn world(l: &LevelLoader, d: sly2_recomp::engine::disc::descriptor::Descriptor) -> Option<World> {
    let lv = crate::common::load_blocking(&l, d).ok()?;
    World::load(&lv, LoadOptions::default()).ok()
}

fn first(w: &World, c: u16) -> Option<usize> {
    w.objects.iter().position(|o| o.class == c)
}

/// level-table rows with Sly / Bentley / Murray objects (for the screenshots)
#[test]
#[ignore]
fn p20_probe_rows() {
    let (l, ds) = tables();
    for (r, d) in ds.into_iter().enumerate() {
        let Some(w) = world(&l, d) else { continue };
        let n = |c: u16| w.objects.iter().filter(|o| o.class == c).count();
        let oid = |c: u16| first(&w, c).map(|o| w.objects[o].oid);
        eprintln!("row {r}: sly {} bentley {} murray {} oids {:?} {:?}", n(CLASS_SLY), n(CLASS_BENTLEY), n(CLASS_MURRAY), oid(CLASS_BENTLEY), oid(CLASS_MURRAY));
    }
}

/// children of Bentley / Murray (row 0)
#[test]
#[ignore]
fn p20_probe_children() {
    let (l, ds) = tables();
    let w = world(&l, ds[0]).unwrap();
    for c in [CLASS_BENTLEY, CLASS_MURRAY] {
        let o = first(&w, c).unwrap();
        for (d, k) in sly2_recomp::engine::playerdraw::subtree(&w, o).into_iter().map(|k| (0, k)) {
            let lo = &w.objects[k];
            let depth = { let mut n = 0; let mut p = lo.owner; while let Some(q) = p { if q == o { break; } n += 1; p = w.objects[q].owner; } n };
            let _ = d;
            eprintln!("{c}: {}{} class {} oid {} disabled {} kids {}", "  ".repeat(depth), k, lo.class, lo.oid, lo.disabled, lo.children.len());
        }
    }
}

/// FUN_0020e890: oid -1 falls back to Sly, then Bentley, then Murray; an oid picks its object; apply_po makes
/// a Bentley / Murray the k18 active character and puts the shadow JT at it; po_call answers from the JT.
#[test]
fn p20_po_selection_and_activation() {
    let (l, ds) = tables();
    let mut seen = 0;
    for d in ds {
        let Some(w) = world(&l, d) else { continue };
        let (s, b, m) = (first(&w, CLASS_SLY), first(&w, CLASS_BENTLEY), first(&w, CLASS_MURRAY));
        assert_eq!(ee_sub_20e890_po(&w, -1), s.or(b).or(m));
        for o in [b, m].into_iter().flatten() {
            let oid = w.objects[o].oid;
            let pick = ee_sub_20e890_po(&w, oid).unwrap();
            assert_eq!(w.objects[pick].oid, oid);
            let w = Arc::new(w.clone());
            let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
            let mut set = ClassSet::build(&w, &mut h);
            set.post_load(&mut h);
            let jt = Jt::new([0.0; 3]);
            let mut cur = PoCurrent::default();
            assert!(apply_po(&w, oid, &mut set, &mut h, &mut cur));
            if w.objects[pick].class == CLASS_BENTLEY || w.objects[pick].class == CLASS_MURRAY {
                assert_eq!(cur.obj, Some(pick));
                assert_eq!(set.k18_active, Some(pick));
                h.po = Some(PoHost { obj: pick, jt: Box::new(jt.clone()) });
                assert_eq!(h.po_call(0x140af8, pick, 0, &[0]), jt.FUN_00140af8(false) as i64);
                assert_eq!(h.po_call(0x19c8d0, pick, 0x2c0, &[3]), jt.FUN_0019c8d0(3) as i64);
                assert_eq!(h.po_call(0x194558, pick, 0x4c0, &[3]), jt.FUN_00194558(3) as i64);
                assert_eq!(h.po_call(0x140af8, pick + 1, 0, &[0]), 0, "other objects: no shadow JT");
                for _ in 0..60 {
                    set.k18_feed_joy(jt.joy);
                    set.frame(&mut h);
                }
                seen += 1;
            }
        }
        if seen >= 4 {
            break;
        }
    }
    assert!(seen > 0, "no level with Bentley / Murray");
}
