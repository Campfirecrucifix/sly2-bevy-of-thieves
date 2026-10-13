//! WP-P22: Bentley / Murray own movement (class slot 15 FUN_0020bb58 + slot 69 FUN_00204108, player::p22).

use sly2_recomp::engine::classes::plugin::EngineHost;
use sly2_recomp::engine::classes::ClassSet;
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::player::p20::{apply_po, PoCurrent, PoHost, CLASS_BENTLEY, CLASS_MURRAY};
use sly2_recomp::engine::player::p22::{ee_sub_203fc0, ee_sub_204108, ee_sub_20bb58, po_move_of, LocoEntry, PoMove};
use sly2_recomp::engine::player::{Jt, Joy};
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

/// FUN_0020bb58: weights {1-u, u}, +0x190 = -(sum w_i v_i len_i / blend length); entry 1 absent: entry 0 only.
#[test]
fn p22_bb58_blend() {
    let e0 = LocoEntry { v: [100.0, 0.0, 0.0], len: 1.0 };
    let e1 = LocoEntry { v: [300.0, 0.0, 0.0], len: 1.0 };
    let v = ee_sub_20bb58(&[Some(e0), Some(e1)], 0.5, 1.0);
    assert!((v[0] + 200.0).abs() < 1e-3, "{v:?}");
    let v = ee_sub_20bb58(&[Some(e0), Some(e1)], 1.0, 1.0);
    assert!((v[0] + 300.0).abs() < 1e-3);
    let v = ee_sub_20bb58(&[Some(e0), None], 0.7, 2.0);
    assert!((v[0] + 200.0).abs() < 1e-3);
}

/// FUN_00203fc0 and FUN_00204108: facing error scales the contact motor; the body velocity goes to
/// -R(+0x1b0) . +0x190 on the ground, nothing linear in the air; the angular servo turns toward +0x1a8.
#[test]
fn p22_204108_forces() {
    assert_eq!(ee_sub_203fc0(0.0, 0.0), 1.0);
    assert_eq!(ee_sub_203fc0(2.0, 0.0), 0.0);
    assert!((ee_sub_203fc0(1.0, 0.0) - 0.5).abs() < 1e-5);
    let jt = Jt::new([0.0; 3]);
    let yaw = 0.5f32;
    let (s, c) = yaw.sin_cos();
    let m = PoMove {
        on: true,
        v190: [-200.0, 0.0, 0.0],
        f_1a8: yaw,
        rows_1b0: [[c, s, 0.0], [-s, c, 0.0], [0.0, 0.0, 1.0]],
        f_1a4: 0,
        w_1e0: [0.0; 3],
        mv: [10.0, 0.0, 0.25, 1.0],
    };
    let dt = 1.0 / 60.0;
    let a = ee_sub_204108(&jt.phys, &m, true, dt);
    // facing error 0.5 rad -> +0x180 = 1
    let v = [a.lin[0] * dt, a.lin[1] * dt];
    assert!((v[0] - 200.0 * c).abs() < 1e-2 && (v[1] - 200.0 * s).abs() < 1e-2, "{v:?}");
    assert!(a.ang[2] > 0.0, "servo turns toward +0x1a8: {:?}", a.ang);
    let air = ee_sub_204108(&jt.phys, &m, false, dt);
    assert_eq!(air.lin, [0.0; 3]);
    let off = ee_sub_204108(&jt.phys, &PoMove { on: false, ..m }, true, dt);
    assert_eq!(off.lin, [0.0; 3]);
    // the JT stand / run forces are the po's angular part (WP-P24: the linear drive is the E8 contact motor)
    let mut jt = jt;
    jt.grounded = true;
    jt.po_move = Some(m);
    assert_eq!(jt.forces(dt).lin, [0.0; 3]);
    assert_eq!(jt.forces(dt).ang, a.ang);
}

/// Real levels: the Bentley / Murray blend entries have root motion; slot 15 calls FUN_0020bb58 while the stick
/// is held (recorded by the host) and `po_move_of` turns it into a +0x190 the body follows.
#[test]
fn p22_class_drives_po_move() {
    let (l, ds) = tables();
    let mut seen = [false; 2];
    for d in ds {
        let Ok(lv) = crate::common::load_blocking(&l, d) else { continue };
        let Ok(w) = World::load(&lv, LoadOptions::default()) else { continue };
        for (k, cls) in [CLASS_BENTLEY, CLASS_MURRAY].into_iter().enumerate() {
            if seen[k] {
                continue;
            }
            let Some(o) = w.objects.iter().position(|x| x.class == cls) else { continue };
            let w = Arc::new(w.clone());
            let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
            let mut set = ClassSet::build(&w, &mut h);
            set.post_load(&mut h);
            let mut cur = PoCurrent::default();
            assert!(apply_po(&w, w.objects[o].oid, &mut set, &mut h, &mut cur));
            if cur.obj != Some(o) {
                continue;
            }
            assert!(!cur.loco.is_empty());
            let e = &cur.loco[0].1;
            eprintln!("class {cls}: loco {:?}", cur.loco);
            assert!(e[0].is_some_and(|x| x.len > 0.0), "entry ASEG 0x1a0 below the po");
            let jt = Jt::new([0.0; 3]);
            h.po = Some(PoHost { obj: o, jt: Box::new(jt.clone()) });
            let mut last = None;
            for f in 0..90 {
                let mut joy = Joy::default();
                joy.angle = 0.3;
                joy.mag = if f < 5 { 0.0 } else { 1.0 };
                set.k18_feed_joy(joy);
                set.frame(&mut h);
                let bb = std::mem::take(&mut h.po_bb58);
                if let Some(m) = po_move_of(&mut set, o, cls, &cur.loco, &bb) {
                    if !bb.is_empty() {
                        last = Some((bb.clone(), m));
                    }
                }
            }
            let Some((bb, m)) = last else {
                eprintln!("class {cls}: no 0x20bb58 call (state never reached locomotion)");
                continue;
            };
            eprintln!("class {cls}: bb58 {bb:?} move {m:?}");
            let sp = (m.v190[0] * m.v190[0] + m.v190[1] * m.v190[1]).sqrt();
            assert!(sp > 1.0, "own locomotion speed from the class's ASEG root motion: {m:?}");
            assert!((m.f_1a8 - 0.3).abs() < 1e-4, "slot 15 FUN_00204660(joy angle)");
            seen[k] = true;
        }
        if seen.iter().all(|&s| s) {
            break;
        }
    }
    assert!(seen.iter().any(|&s| s), "no Bentley / Murray locomotion");
}

/// Item 2: Bentley / Murray drawn state (vis::JtVis::load_po): their slot-13 table + FUN_001400b8 remove the
/// gadget / costume parts.
#[test]
fn p22_po_vis_removes_parts() {
    use sly2_recomp::engine::playerdraw::vis::{JtVis, PO_BIND_2BC390, PO_BIND_2C0270};
    let (l, ds) = tables();
    let w = World::load(&crate::common::load_blocking(&l, ds[0]).unwrap(), LoadOptions::default()).unwrap();
    for (cls, tab) in [(CLASS_BENTLEY, &PO_BIND_2BC390[..]), (CLASS_MURRAY, &PO_BIND_2C0270[..])] {
        let o = w.objects.iter().position(|x| x.class == cls).unwrap();
        let v = JtVis::load_po(&w, o, tab);
        let sub = sly2_recomp::engine::playerdraw::subtree(&w, o);
        eprintln!("class {cls}: subtree {} removed {} hidden records {}", sub.len(), v.removed.len(), v.hidden.len());
        for &k in &sub {
            let x = &w.objects[k];
            if v.removed.contains(&k) && sly2_recomp::engine::gfx::attach::alo_of(&w, k).is_some_and(|a| !a.mesh.records.is_empty()) {
                eprintln!("  removed {k} oid {:#x} class {}", x.oid, x.class);
            }
        }
        assert!(!v.removed.is_empty());
        assert!(v.drawn(o));
    }
}

/// Item 3: switch API: Bentley -> Murray -> Sly through `set_active_character`; `follow_active` keeps one body per
/// character (switching back restores where that character was left).
#[test]
fn p22_switch_characters() {
    use sly2_recomp::engine::player::p22::{follow_active, set_active_character};
    let (l, ds) = tables();
    let w = Arc::new(World::load(&crate::common::load_blocking(&l, ds[0]).unwrap(), LoadOptions::default()).unwrap());
    let b = w.objects.iter().position(|x| x.class == CLASS_BENTLEY).unwrap();
    let m = w.objects.iter().position(|x| x.class == CLASS_MURRAY).unwrap();
    let sly = w.objects.iter().position(|x| x.class == sly2_recomp::engine::player::p20::CLASS_SLY).unwrap();
    let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
    let mut set = ClassSet::build(&w, &mut h);
    set.post_load(&mut h);
    let mut cur = PoCurrent::default();
    let mut jt = Jt::new([1.0, 2.0, 3.0]);
    set_active_character(&w, &mut set, &mut h, Some(b));
    assert_eq!(set.k18_active, Some(b));
    assert!(follow_active(&w, set.k18_active, &mut cur, &mut jt));
    assert_eq!((cur.obj, cur.class), (Some(b), CLASS_BENTLEY));
    assert!(!cur.loco.is_empty());
    jt.phys.pos = [100.0, 0.0, 0.0];
    set_active_character(&w, &mut set, &mut h, Some(m));
    assert_eq!(set.k18_active, Some(m));
    assert!(follow_active(&w, set.k18_active, &mut cur, &mut jt));
    assert_eq!(cur.class, CLASS_MURRAY);
    jt.phys.pos = [200.0, 0.0, 0.0];
    set_active_character(&w, &mut set, &mut h, Some(sly));
    assert_eq!(set.k18_active, None);
    assert!(follow_active(&w, set.k18_active, &mut cur, &mut jt));
    assert_eq!(cur.obj, None);
    assert_eq!(jt.phys.pos, [1.0, 2.0, 3.0], "Sly's body where he was left");
    set_active_character(&w, &mut set, &mut h, Some(b));
    assert!(follow_active(&w, set.k18_active, &mut cur, &mut jt));
    assert_eq!(jt.phys.pos, [100.0, 0.0, 0.0], "Bentley's body where he was left");
    assert!(!follow_active(&w, set.k18_active, &mut cur, &mut jt));
    // K22 sequence action 7 to Sly: the host turns it into a release
    use sly2_recomp::engine::classes::ClassHost;
    h.k22_po_switch(sly as u32 + 1);
    h.k22_po_switch(m as u32 + 1);
    assert_eq!(h.k22_take_po_switches(), vec![m as u32 + 1]);
    assert!(h.po_release);
}

/// Item 4: the hang ASEG JMP[0x55] (jt+0x414) leaves Sly's pelvis at +75 over the feet origin with the wrists at
/// ~+145, so the drawn tree is shifted until the wrists' midpoint lies on the ledge contact JMP+0xf0.
#[test]
fn p22_hang_grip_offset() {
    use sly2_recomp::engine::playerdraw::{aseg_of, bind_channels, find_jt, hang_grip_offset, jt_anim_members, JtRig, JT_WRIST_BONES};
    let (l, ds) = tables();
    let w = World::load(&crate::common::load_blocking(&l, ds[0]).unwrap(), LoadOptions::default()).unwrap();
    let jt = find_jt(&w).unwrap();
    let a = aseg_of(&w, jt_anim_members(&w, jt)[&0x414]).unwrap();
    let mut rig = JtRig::build(&w, jt);
    let b = bind_channels(&w, jt, a, &rig.index);
    rig.pose.bind(&sly2_recomp::engine::anim::play::bind_segments(a, 0.0), &b, 0.0, 1.0);
    rig.pose.compose(&sly2_recomp::engine::anim::pose::IDENTITY);
    let ws = rig.pose.world();
    let o = hang_grip_offset(&rig, &ws).unwrap();
    assert!(o[2] < -100.0, "body lowered below the ledge: {o:?}");
    let c = sly2_recomp::engine::player::p4::JMP_F0;
    for i in 0..3 {
        let g: f32 = JT_WRIST_BONES.iter().map(|&k| ws[rig.index[&(jt, k)]][3][i] * 0.5).sum();
        assert!((g + o[i] - c[i]).abs() < 1e-3);
    }
}
