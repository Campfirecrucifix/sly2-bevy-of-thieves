//! WP-Q2 tests: the JT's class-0x6e blends (FUN_00137a30 / FUN_0020b868 / FUN_001384c0 /
//! FUN_001380f0 / FUN_00137fb8) on synthetic data and on Sly in real levels; run (0x18bce0
//! FUN_0020bb58(1.0)) and jump (FUN_00190520) weight updates; idle (jump blend held) pose.

use sly2_recomp::engine::disc::descriptor::{self, Descriptor};
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::playerdraw::jtblend::*;
use sly2_recomp::engine::playerdraw::*;
use sly2_recomp::engine::world::{LoadOptions, World};
use std::path::PathBuf;
use std::sync::Arc;

fn root() -> PathBuf {
    let r = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    if r.join("Sly 2 - Band of Thieves (USA).iso").exists() { r } else { PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/..")) }
}
fn loader() -> Option<(LevelLoader, Vec<Descriptor>)> {
    let iso = Arc::new(IsoImage::open(root().join("Sly 2 - Band of Thieves (USA).iso")).ok()?);
    let table = descriptor::static_table(&iso.read_file("SCUS_973.16").ok()?)?;
    Some((LevelLoader::new(iso, ModOverlay::default()), table))
}

fn entry(w: f32, len: f32, phases: Vec<(f32, f32)>) -> JtBlendEntry {
    JtBlendEntry { aseg: Some(0), sub_length: len, weight: w, phases: vec![phases] }
}

#[test]
fn phase_table_1380f0() {
    // two entries, 2 phases each: lengths weighted, phase 0 start = fmod(circular mean) * total
    let e = [entry(0.5, 1.0, vec![(0.1, 0.4), (0.5, 0.6)]), entry(0.5, 2.0, vec![(0.4, 1.0), (1.4, 1.0)])];
    let (p, total) = ee_sub_1380f0(&e, 0, 2);
    assert!((p[0].1 - 0.7).abs() < 1e-6 && (p[1].1 - 0.8).abs() < 1e-6);
    assert!((total - 1.5).abs() < 1e-6);
    // first weighted entry unweighted: 0.1 + 0.5 * (0.2 - 0.1) = 0.15
    assert!((p[0].0 - 0.15 * 1.5).abs() < 1e-5, "{p:?}");
    assert!((p[1].0 - (0.15 * 1.5 + 0.7)).abs() < 1e-5);
    // wrap of the difference into (-0.5, 0.5]: 0.9 vs 0.1 -> -0.2
    let e = [entry(1.0, 1.0, vec![(0.1, 1.0)]), entry(1.0, 1.0, vec![(0.9, 1.0)])];
    let (p, total) = ee_sub_1380f0(&e, 0, 1);
    assert!((p[0].0 - 0.9 * total).abs() < 1e-5, "{p:?}");
    // zero-weight entries do not count
    let e = [entry(1.0, 1.0, vec![(0.25, 1.0)]), entry(0.0, 1.0, vec![(0.75, 3.0)])];
    let (p, total) = ee_sub_1380f0(&e, 0, 1);
    assert_eq!((p[0], total), ((0.25, 1.0), 1.0));
}

#[test]
fn select_and_length_1384c0() {
    let mut b = JtBlend { entries: vec![entry(1.0, 2.0, vec![]), entry(0.0, 4.0, vec![])], tables: vec![], length: 2.0, select: -1, channels: vec![] };
    let mut t = [1.0f32];
    b.ee_sub_1384c0(&mut t);
    assert_eq!((b.select, b.length, t[0]), (0, 2.0, 1.0));
    // FUN_0020bb58(0.25): w1 = 0.25, w0 = 0.75 -> blend all, length = 0.75*2 + 0.25*4, time keeps its phase
    b.ee_sub_20bb58(0.25, &mut t);
    assert_eq!((b.select, b.length), (-1, 2.5));
    assert!((t[0] - 1.25).abs() < 1e-6);
    // FUN_0020bb58(1.0): entry 1 alone, selected
    b.ee_sub_20bb58(1.0, &mut t);
    assert_eq!((b.select, b.length), (1, 4.0));
    // without entry 1: w0 = 1
    b.entries[1].aseg = None;
    b.entries[1].weight = 0.0;
    b.ee_sub_20bb58(0.3, &mut t);
    assert_eq!((b.entries[0].weight, b.select), (1.0, 0));
}

#[test]
fn jump_weights_190520() {
    let mut b = JtBlend { entries: (0..3).map(|_| entry(0.0, 1.0, vec![])).collect(), tables: vec![], length: 1.0, select: -1, channels: vec![] };
    let mut jw = JumpWeights::default();
    let mut t = 0.5;
    // jtbs 0: targets (1, 1) -> entry 0 alone; time steered to L/2
    let svt = ee_sub_190520(&mut b, &mut jw, 0, 1.0 / 60.0, 0.0, &mut t).unwrap();
    assert_eq!((b.entries[0].weight, b.select), (1.0, 0));
    assert_eq!(svt, 0.0);
    // jtbs 2: a -> 1, b -> 0: entry 1 takes over
    for _ in 0..120 {
        ee_sub_190520(&mut b, &mut jw, 2, 1.0 / 60.0, 0.0, &mut t);
    }
    assert!(b.entries[1].weight > 0.99 && b.select == 1, "{:?}", jw);
    // jtbs 1: a -> 0: entry 2
    for _ in 0..120 {
        ee_sub_190520(&mut b, &mut jw, 1, 1.0 / 60.0, 0.0, &mut t);
    }
    assert!(b.entries[2].weight > 0.99, "{:?}", jw);
    // jtbs 3: no update
    assert!(ee_sub_190520(&mut b, &mut jw, 3, 1.0 / 60.0, 0.0, &mut t).is_none());
    // lean f = 1 steers time to the end at |svt| <= 3
    let svt = ee_sub_190520(&mut b, &mut jw, 1, 1.0 / 60.0, 1.0, &mut t).unwrap();
    assert_eq!(svt, JUMP_SVT_MAX);
}

#[test]
fn sly_blends_in_levels() {
    let Some((l, t)) = loader() else { return eprintln!("ISO not found; skipped") };
    let boot = crate::common::load_blocking(&l, t[25]).unwrap();
    for r in [3usize, 10, 20] {
        let lv = crate::common::load_blocking(&l, boot.header.level_table.entries[r].desc).unwrap();
        let w = World::load(&lv, LoadOptions::default()).unwrap();
        let jt = find_jt(&w).expect("JT object");
        let rig = JtRig::build(&w, jt);
        let blends = rig.build_blends(&w);
        let run = &blends[&FIELD_RUN];
        let jump = &blends[&FIELD_JUMP];
        let bound = |b: &JtBlend| b.channels.iter().filter(|c| c.bone.is_some()).count();
        let phased = run.channels.iter().filter(|c| c.table.is_some()).count();
        eprintln!(
            "row {r}: blends {:?}; run {} ch ({} bound, {phased} phased) len {} sel {} tables {:?}; jump {} ch len {} sel {}",
            blends.keys().map(|k| format!("{k:#x}")).collect::<Vec<_>>(),
            run.channels.len(),
            bound(run),
            run.length,
            run.select,
            run.tables,
            jump.channels.len(),
            jump.length,
            jump.select
        );
        assert_eq!(run.entries.len(), 2);
        assert!(run.channels.len() > 30 && bound(run) * 10 >= run.channels.len() * 9);
        assert!(phased > 0, "row {r}: run legs use the label phase tables");
        assert_eq!(run.tables.len(), 2);
        assert!(run.length > 0.0 && jump.length > 0.0);
        assert_eq!((jump.entries.len(), jump.select), (3, 0));
        // run as the JT plays it: FUN_0020bb58(1.0) every frame
        let mut an = JtAnimator::new(rig);
        an.request(&w, FIELD_RUN, 0);
        let mut moved = 0.0f32;
        let w0 = an.rig.pose.world();
        for _ in 0..30 {
            an.step(1.0 / 60.0);
        }
        for (a, b) in an.rig.pose.world().iter().zip(&w0) {
            moved = moved.max((0..3).map(|j| (a[3][j] - b[3][j]).abs()).fold(0.0, f32::max));
        }
        let finite = an.rig.pose.world().iter().all(|m| m.iter().flatten().all(|x| x.is_finite()));
        assert!(finite && moved > 0.1 && moved < 300.0, "row {r}: run moved {moved}");
        let rb = &an.blends[&FIELD_RUN];
        if rb.entries[1].aseg.is_some() {
            assert_eq!(rb.select, 1, "row {r}: run = entry 1");
        }
        // jump / stand (jtbs 0): blend held at L/2
        an.request(&w, FIELD_JUMP, 0);
        for _ in 0..30 {
            an.step(1.0 / 60.0);
        }
        let Some(JtPlay::Blend { field, t, .. }) = &an.current else { panic!("jump blend") };
        assert_eq!(*field, FIELD_JUMP);
        assert!((t - an.blends[&FIELD_JUMP].length * 0.5).abs() < 1e-3);
        assert!(an.rig.pose.world().iter().all(|m| m.iter().flatten().all(|x| x.is_finite())));
    }
}

#[test]
fn sly_visibility_in_levels() {
    use sly2_recomp::engine::playerdraw::vis::*;
    use sly2_recomp::engine::anim::aseg::EventData;
    let Some((l, t)) = loader() else { return eprintln!("ISO not found; skipped") };
    let boot = crate::common::load_blocking(&l, t[25]).unwrap();
    for r in [3usize, 10, 20] {
        let lv = crate::common::load_blocking(&l, boot.header.level_table.entries[r].desc).unwrap();
        let w = World::load(&lv, LoadOptions::default()).unwrap();
        let jt = find_jt(&w).unwrap();
        let sub = subtree(&w, jt);
        let alos: Vec<usize> = sub.iter().copied().filter(|&o| o != jt && sly2_recomp::engine::gfx::attach::alo_of(&w, o).is_some()).collect();
        let by_oid = |oid: i32| sub.iter().copied().find(|&o| w.objects[o].oid == oid);
        let mut v = JtVis::load(&w, jt);
        let drawn: Vec<String> = alos.iter().filter(|&&o| v.drawn(o)).map(|&o| format!("{:#x}", w.objects[o].oid)).collect();
        let jt_alo = sly2_recomp::engine::gfx::attach::alo_of(&w, jt).unwrap();
        let hidden: Vec<String> = v.hidden.iter().filter(|k| k.0 == jt && k.2.is_none()).map(|k| format!("{:#x}", jt_alo.mesh.records[k.1].word)).collect();
        // ASEG mesh show/hide events (0xd/0xe) in the JT subtree
        let mut ev = vec![];
        for &o in &sub {
            if let Some(a) = aseg_of(&w, o) {
                for e in a.events0.iter().chain(&a.events1).chain(&a.events2).filter(|e| e.kind == 0xd || e.kind == 0xe) {
                    if let EventData::S32x2(x, y) = e.data {
                        ev.push((w.objects[o].oid, e.kind, x, y));
                    }
                }
            }
        }
        eprintln!("row {r}: {} attached ALOs, drawn {drawn:?}; JT hidden records {hidden:?}; mesh events {ev:?}", alos.len());
        // 0x13e (+0x19a8) removed by 0x2bf1b0, 0x13c (+0x19a0) kept
        let (a, b) = (by_oid(OID_19A0).unwrap(), by_oid(OID_19A8).unwrap());
        assert!(v.drawn(a) && !v.drawn(b), "row {r}");
        assert!(drawn.len() < alos.len());
        // u1 & 1 records of the JT (paraglider / gadget parts) start hidden
        for oid in [0xebf, 0x142, 0x143, 0x144] {
            assert!(jt_alo.mesh.records.iter().enumerate().filter(|(_, x)| x.word == oid).all(|(i, _)| !v.record_drawn(jt, i, None)), "row {r}: {oid:#x}");
        }
        assert!(jt_alo.mesh.records.iter().enumerate().any(|(i, _)| v.record_drawn(jt, i, None)));
        // FUN_0018e540(2): 0x13e added, 0x13c removed; back to -1 restores
        v.ee_sub_18e540(&w, 2);
        assert!(!v.drawn(a) && v.drawn(b));
        v.ee_sub_18e540(&w, 0xb);
        assert!(!v.drawn(a) && v.drawn(b));
        v.ee_sub_18e540(&w, -1);
        assert!(v.drawn(a) && !v.drawn(b));
        // every JT blend field builds (FUN_00188608)
        let rig = JtRig::build(&w, jt);
        let blends = rig.build_blends(&w);
        for (f, _, ms) in blend_fields() {
            let any = ms.iter().any(|m| rig.members.contains_key(m));
            assert_eq!(blends.contains_key(&f), any, "row {r}: field {f:#x}");
        }
        eprintln!("row {r}: blends {:?}", { let mut k: Vec<String> = blends.iter().map(|(k, b)| format!("{k:#x}:{}e/{}ch", b.entries.len(), b.channels.len())).collect(); k.sort(); k });
    }
}

#[test]
fn sly_stand_sm_walk() {
    let Some((l, t)) = loader() else { return eprintln!("ISO not found; skipped") };
    let boot = crate::common::load_blocking(&l, t[25]).unwrap();
    for r in [3usize, 10, 20] {
        let lv = crate::common::load_blocking(&l, boot.header.level_table.entries[r].desc).unwrap();
        let w = World::load(&lv, LoadOptions::default()).unwrap();
        let jt = find_jt(&w).expect("JT object");
        let mut an = JtAnimator::new(JtRig::build(&w, jt));
        an.request_sm(&w, SM_LABEL_STAND);
        let Some(JtPlay::Aseg(first, _)) = &an.current else { panic!("row {r}: stand SM plays no ASEG") };
        let first = *first;
        let mut seen = vec![w.objects[first].oid];
        for _ in 0..1200 {
            an.sm_tick(&w);
            an.step(1.0 / 60.0);
            if let Some(JtPlay::Aseg(o, _)) = &an.current {
                if seen.last() != Some(&w.objects[*o].oid) {
                    seen.push(w.objects[*o].oid);
                }
            }
        }
        eprintln!("row {r}: stand SM ASEGs {:x?}", seen);
        assert!(an.rig.pose.world().iter().all(|m| m.iter().flatten().all(|x| x.is_finite())));
        assert!(an.sm.is_some());
    }
}
