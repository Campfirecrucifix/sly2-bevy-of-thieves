//! WP-P24: glide camera (class-15 volume 0x5f4 -> follow track params), glider model 0x5f5 (SMA jt+0x1d28),
//! Sly pipe / run2 fields, Bentley / Murray cross-fade.

use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::world::{LoadOptions, World};
use std::sync::Arc;

fn tables() -> (LevelLoader, Vec<sly2_recomp::engine::disc::descriptor::Descriptor>) {
    let iso = Arc::new(IsoImage::open(concat!(env!("CARGO_MANIFEST_DIR"), "/../Sly 2 - Band of Thieves (USA).iso")).expect("ISO"));
    let t = sly2_recomp::engine::disc::descriptor::static_table(&iso.read_file("SCUS_973.16").unwrap()).unwrap();
    let l = LevelLoader::new(iso, ModOverlay::default());
    let boot = l.load_blocking(t[25]).unwrap();
    let d = boot.header.level_table.entries.iter().map(|e| e.desc).collect();
    (l, d)
}

#[test]
#[ignore]
fn p24_probe_glider() {
    use sly2_recomp::engine::playerdraw::{ee_sub_15e938_below, find_jt};
    let (l, ds) = tables();
    for (k, d) in ds.into_iter().enumerate() {
        let Ok(lv) = l.load_blocking(d) else { continue };
        let Ok(w) = World::load(&lv, LoadOptions::default()) else { continue };
        let Some(jt) = find_jt(&w) else { continue };
        eprintln!("== level {k}");
        for oid in [0x5f4, 0x5f5, 0x5f6] {
            let Some(o) = ee_sub_15e938_below(&w, jt, oid) else { eprintln!("  {oid:#x} none"); continue };
            let x = &w.objects[o];
            eprintln!("  {oid:#x} obj {o} class {} children {:?}", x.class, x.children.iter().map(|&c| (w.objects[c].class, w.objects[c].oid)).collect::<Vec<_>>());
            use sly2_recomp::engine::world::{loads_b4::B4Load, objects::{LoadData, Payload}};
            if let Payload::Loaded(LoadData::B4(b)) = &x.payload {
                if let B4Load::L220fd0 { props, nested, .. } = &**b {
                    eprintln!("     nested {nested:?}");
                    for p in props { eprintln!("     field {} {:?}", p.field, p.value); }
                }
                if let B4Load::L1efac0 { nested, pairs, links, link_f, .. } = &**b {
                    eprintln!("     sm nested {:?} pairs {pairs:?} links {links:?} f {link_f:?}", nested.iter().map(|&n| (w.objects[n].class, w.objects[n].oid)).collect::<Vec<_>>());
                }
            }
            if let Some(p) = sly2_recomp::engine::classes::props_of(&w, o) {
                for p in p {
                    eprintln!("     field {} {:?}", p.field, p.value);
                }
            }
        }
        if std::env::var("P24_ALL").is_err() {
            break;
        }
    }
}

/// WP-P24: the level-0 glider track 0x5f4 -> C2db858 view (FUN_00220fd0 props into entry 0).
#[test]
fn p24_glide_track_level0() {
    let (l, ds) = tables();
    let lv = l.load_blocking(ds[0]).unwrap();
    let w = World::load(&lv, LoadOptions::default()).unwrap();
    let (pv, t) = sly2_recomp::engine::playerdraw::glide_track(&w).expect("glide track");
    eprintln!("pv {pv} track {t:?}");
    assert_eq!(pv as usize - 1, sly2_recomp::engine::playerdraw::ee_sub_15e938_below(&w, sly2_recomp::engine::playerdraw::find_jt(&w).unwrap(), 0x5f4).unwrap());
    assert_eq!(t.params.s_dist, -400.0);
    assert!((t.fov - 1.0).abs() < 1e-6);
    assert!(t.simple(), "mode-4 track without path is the ported free-follow path");
    assert_eq!(t.f_128, 3, "field 200 -> push priority 5");
}

#[test]
#[ignore]
fn p24_probe_glider_aseg() {
    use sly2_recomp::engine::playerdraw::{aseg_of, ee_sub_15e938_below, find_jt, subtree, vis::JtVis};
    let (l, ds) = tables();
    let lv = l.load_blocking(ds[0]).unwrap();
    let w = World::load(&lv, LoadOptions::default()).unwrap();
    let jt = find_jt(&w).unwrap();
    let v = JtVis::load(&w, jt);
    let g = ee_sub_15e938_below(&w, jt, 0x5f5).unwrap();
    eprintln!("0x5f5 obj {g} owner {:?} parent chain drawn {} removed-in-subtree {:?}", w.objects[g].owner, v.drawn(g), subtree(&w, g).iter().map(|&o| (o, w.objects[o].class, w.objects[o].oid, v.drawn(o))).collect::<Vec<_>>());
    eprintln!("hidden recs of glider subtree: {:?}", v.hidden.iter().filter(|h| subtree(&w, g).contains(&h.0)).collect::<Vec<_>>());
    for (k, &o) in subtree(&w, jt).iter().enumerate() {
        let _ = k;
        if let Some(a) = aseg_of(&w, o) {
            if [5517, 5518].contains(&w.objects[o].oid) || w.objects[o].oid == 0x1528 {
                eprintln!("aseg {} oid {} len {} ch {:?}", o, w.objects[o].oid, a.length, a.channels.iter().map(|c| (((c.oid << 12) as i32) >> 12, c.kind)).collect::<Vec<_>>());
            }
        }
    }
    let sm = ee_sub_15e938_below(&w, jt, 0x5f6).unwrap();
    eprintln!("sm {sm} children {:?}", w.objects[sm].children.iter().map(|&c| (c, w.objects[c].class, w.objects[c].oid)).collect::<Vec<_>>());
    use sly2_recomp::engine::world::{loads_b4::B4Load, objects::{LoadData, Payload}};
    if let Payload::Loaded(LoadData::B4(b)) = &w.objects[sm].payload {
        if let B4Load::L1efac0 { nested, .. } = &**b {
            for &n in nested {
                if let Some(a) = aseg_of(&w, n) { eprintln!("  full {:?}", (&a.channels, &a.scalars, &a.events0, &a.events1, &a.events2, &a.props)); }
                eprintln!(" nested {n} class {} oid {} aseg {:?}", w.objects[n].class, w.objects[n].oid, aseg_of(&w, n).map(|a| (a.length, a.channels.iter().map(|c| (((c.oid << 12) as i32) >> 12, c.kind)).collect::<Vec<_>>())));
            }
        }
    }
}

/// WP-P24: glider SMA (JT +0x1d28): 0x5f7 plays ASEG 5517 (kind 7 shows 0x5f5, scale 0 -> 1), 0x5f8 plays
/// 5518 (kind 8 removes it).
#[test]
fn p24_glider_sma_level0() {
    use sly2_recomp::engine::playerdraw::{ee_sub_15e938_below, find_jt, glider::apply_e7, vis::JtVis, JtAnimator, JtRig};
    let (l, ds) = tables();
    let lv = l.load_blocking(ds[0]).unwrap();
    let w = World::load(&lv, LoadOptions::default()).unwrap();
    let jt = find_jt(&w).unwrap();
    let g = ee_sub_15e938_below(&w, jt, 0x5f5).unwrap();
    let mut vis = JtVis::load(&w, jt);
    assert!(!vis.drawn(g), "glider removed after load");
    let mut an = JtAnimator::new(JtRig::build(&w, jt));
    an.glider_init(&w);
    let sma = &an.glider.as_ref().expect("glider sma").sma;
    eprintln!("init sma cur {} goal {} asega {:?}", sma.cur, sma.goal, sma.asega);
    let bones: Vec<usize> = an.rig.keys.iter().enumerate().filter(|(_, k)| k.0 == g).map(|(i, _)| i).collect();
    eprintln!("glider bones {bones:?}");
    an.glider_goal(&w, 0x5f7);
    let mut shown = false;
    for f in 0..20 {
        an.glider_tick(&w);
        an.step(1.0 / 60.0);
        shown |= apply_e7(&w, &mut vis, &an.fired);
        let s = an.glider.as_ref().unwrap();
        if f % 4 == 0 {
            eprintln!("  binding {:?} locals {:?}", an.rig.bindings.get(&1839), bones.iter().map(|&b| (an.rig.pose.bones[b].local[0][0], an.rig.pose.bones[b].local[1][1])).collect::<Vec<_>>());
            eprintln!("f{f} cur {} goal {} play {:?} fired {:?} drawn {} scale {:?}", s.sma.cur, s.sma.goal, s.play.as_ref().map(|p| (p.0, p.1.state.t)), an.fired, vis.drawn(g), bones.first().map(|&b| an.rig.pose.bones[b].local[0]));
        }
    }
    assert!(shown && vis.drawn(g), "0x5f7 shows the glider");
    // kind 0xd (7, 3775) / (7, 3784): the JT's glider mesh records (u1 bit 0 = hidden at load) are shown
    let glider_recs = |vis: &JtVis| (0..3).filter(|&r| vis.record_drawn(jt, r, None)).count();
    assert_eq!(glider_recs(&vis), 3, "JT records 0..2 (words 3775 / 3784) drawn while gliding");
    let b = an.rig.bindings[&1839][0].expect("5517 channel binds a glider bone");
    assert!(bones.contains(&b) && (an.rig.pose.bones[b].local[0][0] - 1.0).abs() < 1e-3, "scale channel ends at 1");
    an.glider_goal(&w, 0x5f8);
    for _ in 0..20 {
        an.glider_tick(&w);
        an.step(1.0 / 60.0);
        apply_e7(&w, &mut vis, &an.fired);
    }
    assert!(!vis.drawn(g), "0x5f8 removes the glider");
    assert_eq!(glider_recs(&vis), 0, "kind 0xe hides the glider records again");
}

/// WP-P24: binocucom FUN_00189530 hide switches JT +0x19a0/+0x19a4/+0x19a8 off and back on.
#[test]
fn p24_binoc_hide_level0() {
    use sly2_recomp::engine::playerdraw::{ee_sub_15e938_below, find_jt, vis::{JtVis, OID_19A4}};
    let (l, ds) = tables();
    let lv = l.load_blocking(ds[0]).unwrap();
    let w = World::load(&lv, LoadOptions::default()).unwrap();
    let jt = find_jt(&w).unwrap();
    let Some(o) = ee_sub_15e938_below(&w, jt, OID_19A4) else { eprintln!("no 0x13d on level 0"); return };
    let mut vis = JtVis::load(&w, jt);
    let before = vis.drawn(o);
    vis.ee_sub_189530(&w, true);
    assert!(!vis.drawn(o), "hidden while binocucom is up");
    vis.ee_sub_189530(&w, false);
    assert_eq!(vis.drawn(o), before, "restored");
}

/// WP-P24 item 3: a Bentley / Murray shadow JT on the ground moves through the E8 contact motor
/// (`p22::po_motor` + `motor::po_step`): it heads along -R(+0x1b0) . +0x190 (P22's old velocity target) and
/// keeps the ground.
#[test]
fn p24_po_contact_motor_real_level() {
    use sly2_recomp::engine::physics::CollisionWorld;
    use sly2_recomp::engine::physics::motor::FUN_0014c6d8;
    use sly2_recomp::engine::player::p20::{CLASS_BENTLEY, CLASS_MURRAY};
    use sly2_recomp::engine::player::p22::PoMove;
    use sly2_recomp::engine::player::{Jt, JtInput, JT_RADIUS};
    let (l, ds) = tables();
    let mut done = 0;
    for d in ds {
        let Ok(lv) = l.load_blocking(d) else { continue };
        let Ok(w) = World::load(&lv, LoadOptions::default()) else { continue };
        let Some(o) = w.objects.iter().position(|x| x.class == CLASS_BENTLEY || x.class == CLASS_MURRAY) else { continue };
        let cw = CollisionWorld::build(&w);
        let mut h = sly2_recomp::engine::classes::plugin::EngineHost { world: Some(Arc::new(w.clone())), ..Default::default() };
        let p = sly2_recomp::engine::classes::ClassHost::ee_sub_1a62e8(&mut h, o as u32);
        let Some(g) = FUN_0014c6d8(&cw, [p[0], p[1], p[2] + 200.0], [p[0], p[1], p[2] - 4000.0]) else { continue };
        if g.normal[2] < 0.7 {
            continue;
        }
        let mut jt = Jt::new([p[0], p[1], g.point[2] + JT_RADIUS]);
        jt.grounded = true;
        let yaw = jt.phys.rot[0][1].atan2(jt.phys.rot[0][0]);
        let (s, c) = yaw.sin_cos();
        jt.po_move = Some(PoMove { on: true, v190: [-150.0, 0.0, 0.0], f_1a8: yaw, rows_1b0: [[c, s, 0.0], [-s, c, 0.0], [0.0, 0.0, 1.0]], ..Default::default() });
        let start = jt.phys.pos;
        let mut on = 0;
        // 25 frames: both starts walk off a ledge after ~30 (correct: no ground -> no motor)
        for f in 0..25 {
            jt.update(&cw, JtInput::default(), 1.0 / 60.0, &mut ());
            on += jt.grounded as usize;
            if std::env::var("P24_TRACE").is_ok() && f % 5 == 0 {
                let p = jt.phys.pos;
                let gz = FUN_0014c6d8(&cw, [p[0], p[1], p[2] + 200.0], [p[0], p[1], p[2] - 4000.0]).map(|g| g.point[2]);
                eprintln!("  f{f} pos {p:?} v {:?} jts {} jtbs {} ground {gz:?} grounded {}", jt.phys.v, jt.jts, jt.jtbs, jt.grounded);
            }
            if f >= 5 {
                let v = jt.phys.v[0] * c + jt.phys.v[1] * s;
                assert!((v - 150.0).abs() < 5.0, "f{f}: motor speed {v} (goal -R . +0x190 = 150)");
            }
        }
        let dv = [jt.phys.pos[0] - start[0], jt.phys.pos[1] - start[1]];
        let along = dv[0] * c + dv[1] * s;
        eprintln!("class {} obj {o}: moved {dv:?} along {along:.1}, grounded {on}/25, jts {}, dz {:.1}", w.objects[o].class, jt.jts, jt.phys.pos[2] - start[2]);
        assert!(jt.phys.pos.iter().all(|x| x.is_finite()));
        assert!(along > 50.0, "drives along -R . +0x190: {dv:?}");
        assert!(on == 25, "kept the ground {on}/25");
        done += 1;
        if done >= 2 {
            break;
        }
    }
    assert!(done >= 1);
}

/// WP-P24 item 2 probe: which levels carry the pipe (jt+0xfa4/0xfb4/0xfc4/0xfc8/0xfcc) and run2 (jt+0x1c44 /
/// 0x1c84) ASEG oids below the JT, and pipe objects (class 0xb).
#[test]
#[ignore]
fn p24_probe_pipe_run2() {
    use sly2_recomp::engine::playerdraw::{ee_sub_15e938_below, find_jt};
    let (l, ds) = tables();
    for (k, d) in ds.into_iter().enumerate() {
        let Ok(lv) = l.load_blocking(d) else { continue };
        let Ok(w) = World::load(&lv, LoadOptions::default()) else { continue };
        let pipes = w.objects.iter().filter(|o| o.class == 0xb).count();
        let Some(jt) = find_jt(&w) else { eprintln!("level {k}: no jt, pipes {pipes}"); continue };
        let found: Vec<(i32, Option<usize>)> = [0xaa, 0xab, 0xac, 0xad, 0xae, 0x2a6, 0x2a7, 0x2a8, 0x2a9, 0x2aa, 0x114, 0x115]
            .iter().map(|&o| (o, ee_sub_15e938_below(&w, jt, o))).collect();
        eprintln!("level {k}: pipes {pipes} {:?}", found.iter().filter(|f| f.1.is_some()).map(|f| format!("{:#x}={}c{}", f.0, f.1.unwrap(), w.objects[f.1.unwrap()].class)).collect::<Vec<_>>());
    }
}

/// WP-P24 item 2: Sly's run2 blend jt+0x1c34 (entries jt+0x1c44 / jt+0x1c84 from the 0x2bf1b0 rows, oids
/// 0x114 / 0x115) and the pipe climb blend jt+0xfc0 (jt+0xfa4 / jt+0xfb4, oids 0xad / 0xae) build on the
/// audit rows and on pipe levels; jt+0xfc4 (oid 0xaa) has no object on any level (p14 plays jt+0xfcc).
#[test]
fn p24_run2_pipe_blends_built() {
    use sly2_recomp::engine::player::p15_anim::{jt_blends_build, FIELD_PIPE_CLIMB};
    use sly2_recomp::engine::playerdraw::{find_jt, JtAnimator, JtRig, FIELD_RUN2};
    let (l, ds) = tables();
    for row in [0usize, 2, 4, 12, 14] {
        let w = World::load(&l.load_blocking(ds[row]).unwrap(), LoadOptions::default()).unwrap();
        let rig = JtRig::build(&w, find_jt(&w).unwrap());
        assert!(rig.members.contains_key(&0x1c44) && rig.members.contains_key(&0x1c84), "row {row}: run2 entries");
        assert!(!rig.members.contains_key(&0xfc4), "row {row}: 0xfc4 now bound?");
        let mut an = JtAnimator::new(rig);
        jt_blends_build(&w, &mut an);
        for f in [FIELD_RUN2, FIELD_PIPE_CLIMB] {
            let b = an.blends.get(&f).unwrap_or_else(|| panic!("row {row}: blend {f:#x}"));
            eprintln!("row {row} blend {f:#x}: {} entries", b.entries.len());
            assert_eq!(b.entries.len(), 2);
        }
    }
}

/// WP-P24 item 2: Bentley / Murray cross-fade: switching the driving anim (run block -> an ASEGA) fades the
/// bone locals over BLEND_TIME from the last pose (REPLACED FUN_0012f9b8 blend-in, as the JT).
#[test]
fn p24_po_crossfade() {
    use sly2_recomp::engine::player::p20::{CLASS_BENTLEY, CLASS_MURRAY};
    use sly2_recomp::engine::playerdraw::po::PoAnimator;
    use sly2_recomp::engine::playerdraw::{aseg_of, subtree, BLEND_TIME};
    let (l, ds) = tables();
    let mut done = 0;
    for d in ds {
        let Ok(lv) = l.load_blocking(d) else { continue };
        let Ok(w) = World::load(&lv, LoadOptions::default()) else { continue };
        for cls in [CLASS_BENTLEY, CLASS_MURRAY] {
            let Some(po) = w.objects.iter().position(|x| x.class == cls) else { continue };
            let mut an = PoAnimator::new(&w, po);
            let Some(&blk) = an.blends.keys().min() else { continue };
            let Some(a) = subtree(&w, po).into_iter().find(|&o| aseg_of(&w, o).is_some_and(|a| a.channels.len() > 3)) else { continue };
            let dt = 1.0 / 60.0;
            for _ in 0..10 {
                an.step(&w, &[], Some((blk, 0.5)), dt);
            }
            let run: Vec<_> = an.rig.pose.bones.iter().map(|b| b.local).collect();
            an.step(&w, &[(a, 0.0)], None, dt);
            assert!(an.fade > 0.0, "fade started");
            let mid: Vec<_> = an.rig.pose.bones.iter().map(|b| b.local).collect();
            for _ in 0..(BLEND_TIME / dt) as usize + 2 {
                an.step(&w, &[(a, 0.0)], None, dt);
            }
            assert_eq!(an.fade, 0.0);
            let end: Vec<_> = an.rig.pose.bones.iter().map(|b| b.local).collect();
            let diff = |x: &Vec<sly2_recomp::engine::anim::pose::Mat4>, y: &Vec<sly2_recomp::engine::anim::pose::Mat4>| {
                x.iter().zip(y).map(|(p, q)| p.iter().flatten().zip(q.iter().flatten()).map(|(u, v)| (u - v).abs()).fold(0.0f32, f32::max)).fold(0.0f32, f32::max)
            };
            let (dr, de) = (diff(&mid, &run), diff(&mid, &end));
            eprintln!("class {cls} po {po} blk {blk:#x} aseg {a}: |mid-run| {dr:.3} |mid-end| {de:.3} |run-end| {:.3}", diff(&run, &end));
            assert!(de > 1e-3 && diff(&run, &end) > de, "first frame after the switch is between the two poses");
            done += 1;
        }
        if done >= 2 {
            break;
        }
    }
    assert!(done >= 1);
}

