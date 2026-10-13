//! WP-P19: animation selection gaps (ledge hang, paraglider + glider model), Circle ground cane attack,
//! ClassLink::jt_states drain, Bentley / Murray player layer.

use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::world::{LoadOptions, World};
use std::sync::Arc;

fn world(r: usize) -> World {
    let iso = Arc::new(IsoImage::open(concat!(env!("CARGO_MANIFEST_DIR"), "/../Sly 2 - Band of Thieves (USA).iso")).expect("ISO"));
    let t = sly2_recomp::engine::disc::descriptor::static_table(&iso.read_file("SCUS_973.16").unwrap()).unwrap();
    let l = LevelLoader::new(iso, ModOverlay::default());
    let boot = crate::common::load_blocking(&l, t[25]).unwrap();
    let d: Vec<_> = boot.header.level_table.entries.iter().map(|e| e.desc).collect();
    let lv = crate::common::load_blocking(&l, d[r]).unwrap();
    World::load(&lv, LoadOptions::default()).unwrap()
}

#[test]
#[ignore]
fn p19_probe_field() {
    use sly2_recomp::engine::playerdraw::{aseg_of, find_jt, JtAnimator, JtRig};
    let row: usize = std::env::var("P19_ROW").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    let field = u32::from_str_radix(&std::env::var("P19_FIELD").unwrap_or("414".into()), 16).unwrap();
    let w = world(row);
    let jt = find_jt(&w).unwrap();
    let mut an = JtAnimator::new(JtRig::build(&w, jt));
    let rest = an.rig.pose.world();
    match an.rig.members.get(&field) {
        Some(&o) => {
            let a = aseg_of(&w, o).unwrap();
            let b = &an.rig.bindings[&o];
            println!("field {field:#x} obj {o} oid {:#x} len {} chans {} bound {}", w.objects[o].oid, a.length, a.channels.len(), b.iter().filter(|x| x.is_some()).count());
            for (c, bb) in a.channels.iter().zip(b) {
                println!("  oid {:#x} kind {} -> {:?}", c.oid, c.kind, bb);
            }
        }
        None => println!("field {field:#x}: no member (blend {})", an.blends.contains_key(&field)),
    }
    an.request(&w, field, 0x11);
    let nf: usize = std::env::var("P19_N").ok().and_then(|v| v.parse().ok()).unwrap_or(30);
    for _ in 0..nf {
        an.step(1.0 / 60.0);
    }
    let wd = an.rig.pose.world();
    for i in 0..an.rig.keys.len().min(80) {
        let f = |m: &[[f32; 4]; 4]| format!("({:7.1},{:7.1},{:7.1})", m[3][0], m[3][1], m[3][2]);
        println!("b{i:3} {:?} kind {:2} rest {} now {}", an.rig.keys[i], an.rig.pose.bones[i].kind, f(&rest[i]), f(&wd[i]));
    }
}

/// WP-P19 item 2: Circle on the ground with a target in range -> FUN_001928f8 kind 6 really runs
/// FUN_0018e540(jt, 0) (cane state 0) and FUN_0018ed70 (recorded); without a target nothing (1:1).
#[test]
fn p19_circle_kind6_enters_cane_state_0() {
    use sly2_recomp::engine::player::Jt;
    let mut jt = Jt::new([0.0, 0.0, 35.0]);
    jt.t = 10.0;
    assert_eq!(jt.cane.jtcs, -1);
    jt.FUN_001928f8(6, None, 0, &mut ());
    assert_eq!(jt.cane.jtcs, 0, "kind 6 -> FUN_0018e540(jt, 0)");
    assert!(jt.calls.contains(&(0x0018ed70, 0)), "FUN_0018ed70 recorded");
}

/// WP-P19 item 3 (J12): script SetJtJts requests (method 0x10b028 -> FUN_0018d568) queued on
/// `ClassLink::jt_states` reach the player's JT; requests on other objects are dropped.
#[test]
fn p19_drain_jt_states() {
    use sly2_recomp::engine::player::plugin::drain_jt_states;
    use sly2_recomp::engine::player::p7_damage::FUN_0018d568;
    use sly2_recomp::engine::player::Jt;
    let mut jt = Jt::new([0.0, 0.0, 35.0]);
    jt.t = 10.0;
    let before = jt.jts;
    drain_jt_states(&mut jt, vec![(7, FUN_0018d568(1).unwrap())], Some(3), &mut ());
    assert_eq!(jt.jts, before, "other object: dropped");
    drain_jt_states(&mut jt, vec![(3, FUN_0018d568(1).unwrap())], Some(3), &mut ());
    assert_eq!(jt.jts, 2, "0x10b028 i=1 -> table 0x2a5d00[1] = jts 2");
}

#[test]
#[ignore]
fn p19_probe_oids() {
    let row: usize = std::env::var("P19_ROW").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    let w = world(row);
    let want: Vec<i32> = std::env::var("P19_OIDS").unwrap_or("5ee,5f4".into()).split(',').map(|s| i32::from_str_radix(s, 16).unwrap()).collect();
    for (i, o) in w.objects.iter().enumerate() {
        if want.contains(&(o.oid as i32)) {
            println!("obj {i} oid {:#x} class {} owner {:?}", o.oid, o.class, o.owner);
        }
    }
}

#[test]
#[ignore]
fn p19_probe_aseg_obj() {
    use sly2_recomp::engine::playerdraw::aseg_of;
    let row: usize = std::env::var("P19_ROW").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    // T1: probe needs P19_OBJ; skip under the full tier (--include-ignored) when unset
    let Ok(o) = std::env::var("P19_OBJ") else { return eprintln!("P19_OBJ unset; probe skipped") };
    let o: usize = o.parse().unwrap();
    let w = world(row);
    let a = aseg_of(&w, o).unwrap();
    println!("obj {o} oid {:#x} class {} len {} chans {}", w.objects[o].oid, w.objects[o].class, a.length, a.channels.len());
    println!("ev0 {:?}
ev1 {:?}
ev2 {:?}", a.events0, a.events1, a.events2);
    let mut seen = std::collections::BTreeSet::new();
    for c in &a.channels {
        if seen.insert(c.oid) {
            let objs: Vec<_> = w.objects.iter().enumerate().filter(|(_, x)| x.oid as i32 == c.oid as i32).map(|(i, x)| (i, x.class, x.owner)).take(3).collect();
            println!("  chan oid {:#x} kind {} objs {:?}", c.oid, c.kind, objs);
        }
    }
}

#[test]
#[ignore]
fn p19_probe_vis() {
    use sly2_recomp::engine::playerdraw::{find_jt, subtree, vis::JtVis};
    let w = world(0);
    let jt = find_jt(&w).unwrap();
    let v = JtVis::load(&w, jt);
    for o in subtree(&w, jt) {
        let x = &w.objects[o];
        if [0x5f4, 0x5f5, 0x5f6, 0x5ee].contains(&(x.oid as i32)) || std::env::var("P19_ALL").is_ok() {
            println!("obj {o} oid {:#x} class {} owner {:?} drawn {}", x.oid, x.class, x.owner, v.drawn(o));
        }
    }
}

/// WP-P19 item 1: paraglider entry 0x195fb8 case 0x13 plays JMP[0x5e] = jt+0x438 (post-load table
/// 0x2bc9f0 oid 0x5ee, 0.25 s, animates the JT bones); case 0x14 JMP[0x6e] = jt+0x478 is unbound for Sly.
#[test]
fn p19_glide_open_aseg_bound() {
    use sly2_recomp::engine::player::jmp_body_aseg_195fb8;
    use sly2_recomp::engine::playerdraw::{aseg_of, find_jt, jt_anim_members};
    assert_eq!(jmp_body_aseg_195fb8(0x13, 0), Some(0x438));
    assert_eq!(jmp_body_aseg_195fb8(0x14, 0), Some(0x478));
    let w = world(0);
    let m = jt_anim_members(&w, find_jt(&w).unwrap());
    let o = m[&0x438];
    assert_eq!(w.objects[o].oid, 0x5ee);
    assert!((aseg_of(&w, o).unwrap().length - 0.25).abs() < 1e-3);
    assert!(!m.contains_key(&0x478));
    assert!(m.contains_key(&0x414), "JT_TABLE members kept");
}
