//! WP-P14: pole / swing-point targets from the level props (class 0x22 round poles in list G, class 0x26 attach
//! points in list I, class 0x13 swing points), picks against the real collision world, attach climbs.

use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::world::{LoadOptions, World};
use std::sync::Arc;

fn rows() -> (LevelLoader, Vec<sly2_recomp::engine::disc::descriptor::Descriptor>) {
    let iso = Arc::new(IsoImage::open(concat!(env!("CARGO_MANIFEST_DIR"), "/../Sly 2 - Band of Thieves (USA).iso")).expect("ISO"));
    let t = sly2_recomp::engine::disc::descriptor::static_table(&iso.read_file("SCUS_973.16").unwrap()).unwrap();
    let l = LevelLoader::new(iso, ModOverlay::default());
    let boot = l.load_blocking(t[25]).unwrap();
    let d = boot.header.level_table.entries.iter().map(|e| e.desc).collect();
    (l, d)
}

/// Probe (ignored): classes 0x22 / 0x26 / 0x13 with their props per row.
#[test]
#[ignore]
fn p14_probe_props() {
    let (l, descs) = rows();
    for (r, d) in descs.into_iter().enumerate() {
        let Ok(lv) = l.load_blocking(d) else { continue };
        let Ok(w) = World::load(&lv, LoadOptions::default()) else { continue };
        let mut n = 0;
        for (i, o) in w.objects.iter().enumerate() {
            if ![34u16, 38, 19, 11, 12, 13].contains(&o.class) {
                continue;
            }
            n += 1;
            if n > 400 {
                continue;
            }
            let props = sly2_recomp::engine::game::fields::object_props(o).unwrap_or(&[]);
            let ps: Vec<String> = props.iter().filter(|p| p.field >= 700).map(|p| format!("{}={:?}", p.field, p.value)).collect();
            println!("row {r} obj {i} class {} oid {} dis {} props {}", o.class, o.oid, o.disabled, ps.join(" "));
        }
    }
}

/// Probe (ignored): JT members 0xf50..0x1000 (jts 8 ASEGs) on row 14.
#[test]
#[ignore]
fn p14_probe_jts8_asegs() {
    let (l, descs) = rows();
    let lv = l.load_blocking(descs[14]).unwrap();
    let w = World::load(&lv, LoadOptions::default()).unwrap();
    let jt = sly2_recomp::engine::playerdraw::find_jt(&w).unwrap();
    let m = sly2_recomp::engine::playerdraw::jt_anim_members(&w, jt);
    let mut ks: Vec<_> = m.iter().filter(|(k, _)| (0xf50..0x1000).contains(*k)).collect();
    ks.sort();
    for (k, o) in ks {
        let a = sly2_recomp::engine::playerdraw::aseg_of(&w, *o);
        println!("{k:#x} obj {o} class {} {:?}", w.objects[*o].class, a.map(|a| (a.length, a.channels.len(), a.scalars.len(), a.props.iter().map(|p| (p.field, format!("{:?}", p.value))).collect::<Vec<_>>())));
        if let Some(a) = a {
            for c in &a.channels {
                println!("   ch oid {:#x} flag {} vec {}", c.oid, c.flag, c.vec.is_some());
            }
        }
    }
}

/// Probe (ignored): targets per category on row 14 and an air pick toward each hang point / pipe with the
/// real player collision world.
#[test]
#[ignore]
fn p14_probe_pick() {
    use sly2_recomp::engine::player::p5_target::TargetGeom;
    use sly2_recomp::engine::player::p6::P6Level;
    use sly2_recomp::engine::player::Jt;
    let (l, descs) = rows();
    let lv = l.load_blocking(descs[14]).unwrap();
    let w = World::load(&lv, LoadOptions::default()).unwrap();
    let p6 = P6Level::load(&w);
    let cw = sly2_recomp::engine::player::p10::player_collision_world(&w);
    let mut n = std::collections::BTreeMap::new();
    for t in &p6.targets {
        let k = format!("{:?}", t.obj.geom).split(' ').next().unwrap().to_string();
        *n.entry(k).or_insert(0) += 1;
    }
    println!("cats {n:?}");
    for t in p6.targets.iter().take(400) {
        let pt = match &t.obj.geom {
            TargetGeom::Spot0xd { pt } => *pt,
            TargetGeom::PathBody { path } => path.FUN_001e6f10(path.FUN_001e6648() * 0.5).0,
            _ => continue,
        };
        for d in [[-300.0f32, 0.0, -100.0], [300.0, 0.0, 100.0], [0.0, -300.0, 200.0], [0.0, 300.0, 0.0]] {
            let mut jt = Jt::new([pt[0] + d[0], pt[1] + d[1], pt[2] + d[2]]);
            p6.apply(&mut jt);
            jt.t = 10.0;
            jt.jts = 2;
            jt.jtbs = 4;
            jt.set_velocity_pub([-d[0] * 1.7, -d[1] * 1.7, 700.0]);
            let mut j2 = jt.clone();
            let e = j2.FUN_0019b110(&sly2_recomp::engine::physics::query::CollisionWorld::default(), None, &mut ());
            let ok = jt.FUN_0019b110(&cw, None, &mut ());
            if matches!(t.obj.geom, TargetGeom::Spot0xd { .. }) {
                let v = [-d[0] * 1.7, -d[1] * 1.7, 700.0];
                let r = sly2_recomp::engine::player::p5_target::FUN_0019a4a0(jt.phys.pos, jt.phys.gravity[2], v, [0.0; 3], pt, [0.0; 3]);
                println!("spot {} {pt:?} g {:?} v {v:?} t {} cost {} mask {} jmask {:x}", t.obj.id, jt.phys.gravity, r.t, r.cost, t.obj.mask, jt.mask_1000);
            }
            if e || ok {
                print!("empty-world {e} ({}) ", j2.jtbs);
                println!("obj {} at {pt:?} from {d:?}: jtbs {} att {:?} mask {:x} range {}", t.obj.id, jt.jtbs, jt.flight.target, t.obj.mask, t.obj.range);
            }
        }
    }
}

/// Probe (ignored): what blocks the straight approach to the row-14 hang point 3228.
#[test]
#[ignore]
fn p14_probe_block() {
    use sly2_recomp::engine::physics::query::CollisionWorld;
    let (l, descs) = rows();
    let lv = l.load_blocking(descs[14]).unwrap();
    let w = World::load(&lv, LoadOptions::default()).unwrap();
    let cw = sly2_recomp::engine::player::p10::player_collision_world(&w);
    let pt = [5966.51f32, -9437.557, 651.3698];
    for c in cw.colliders.iter().filter(|c| c.object == 3577 || c.object == 5350) {
        let one = CollisionWorld { colliders: vec![c.clone()] };
        for (dx, dy) in [(0.0f32, 0.0f32), (-300.0, 0.0), (2000.0, 0.0), (0.0, 3000.0)] {
            let iv: Vec<_> = one.clip_segment([pt[0] + dx, pt[1] + dy, pt[2] + 5000.0], [pt[0] + dx, pt[1] + dy, pt[2] - 5000.0]).iter().map(|h| (h.t, h.pos[0][2])).collect();
            println!("obj {} src {:?} col ({dx},{dy}): {iv:?}", c.object, c.source, );
        }
    }
    let o = &w.objects[3577];
    println!("3577 oid {} class {} payload {}", o.oid, o.class, { let s = format!("{:?}", o.payload); let i = s.find("parts").unwrap_or(0); s[i..(i + 1500).min(s.len())].to_string() });
}

/// Probe (ignored): which colliders (after the broadphase) hit the straight approach segments to the hang points.
#[test]
#[ignore]
fn p14_probe_block2() {
    use sly2_recomp::engine::physics::query::CollisionWorld;
    use sly2_recomp::engine::player::p5_target::{ClearFilter, ClearWorld, TargetGeom};
    use sly2_recomp::engine::player::p6::P6Level;
    let (l, descs) = rows();
    let lv = l.load_blocking(descs[14]).unwrap();
    let w = World::load(&lv, LoadOptions::default()).unwrap();
    let cw = sly2_recomp::engine::player::p10::player_collision_world(&w);
    let p6 = P6Level::load(&w);
    for t in &p6.targets {
        let TargetGeom::Spot0xd { pt } = t.obj.geom else { continue };
        for d in [[-300.0f32, 0.0], [300.0, 0.0], [0.0, -300.0], [0.0, 300.0]] {
            let a = [pt[0] + d[0], pt[1] + d[1], pt[2] - 100.0];
            let mut hits = vec![];
            for c in &cw.colliders {
                let one = CollisionWorld { colliders: vec![c.clone()] };
                if let Some((p, f)) = one.seg_hit(a, pt, ClearFilter::Layer) {
                    hits.push((c.object, w.objects[c.object].class, f, p));
                }
            }
            println!("spot {} {d:?}: {hits:?}", t.obj.id);
        }
    }
}

/// Probe (ignored): owner relations of the row-14 blocker 5350 vs the hang points' 818 refs.
#[test]
#[ignore]
fn p14_probe_5350() {
    let (l, descs) = rows();
    let lv = l.load_blocking(descs[14]).unwrap();
    let w = World::load(&lv, LoadOptions::default()).unwrap();
    for i in [5350usize, 3070, 3228, 3230] {
        let o = &w.objects[i];
        let props = sly2_recomp::engine::game::fields::object_props(o).unwrap_or(&[]);
        let ps: Vec<String> = props.iter().map(|p| format!("{}={:?}", p.field, p.value)).collect();
        println!("obj {i} class {} oid {} ({:#x}) owner {:?} props {}", o.class, o.oid, o.oid, o.owner, ps.join(" "));
    }
    for (i, o) in w.objects.iter().enumerate() {
        if o.oid == 0x15aae || o.oid == 88750 { println!("oid match obj {i} class {}", o.class); }
    }
}

/// Probe (ignored): 5350 geometry around hang point 3228.
#[test]
#[ignore]
fn p14_probe_5350_geom() {
    let (l, descs) = rows();
    let lv = l.load_blocking(descs[14]).unwrap();
    let w = World::load(&lv, LoadOptions::default()).unwrap();
    let cw = sly2_recomp::engine::player::p10::player_collision_world(&w);
    let pt = [5966.51f32, -9437.557, 651.3698];
    for c in cw.colliders.iter().filter(|c| c.object == 5350) {
        let near: Vec<_> = c.geom.verts.iter().filter(|v| (0..3).all(|k| (v[k] - pt[k]).abs() < 400.0)).map(|v| [(v[0] - pt[0]) as i32, (v[1] - pt[1]) as i32, (v[2] - pt[2]) as i32]).collect();
        println!("src {:?} nverts {} near {:?}", c.source, c.geom.verts.len(), near);
    }
}

/// Probe (ignored): pipe (class 11 PathBody) / rope picks from 16 directions with the real collision world.
#[test]
#[ignore]
fn p14_probe_pipe_pick() {
    use sly2_recomp::engine::player::p5_target::TargetGeom;
    use sly2_recomp::engine::player::p6::P6Level;
    use sly2_recomp::engine::player::Jt;
    let (l, descs) = rows();
    let lv = l.load_blocking(descs[14]).unwrap();
    let w = World::load(&lv, LoadOptions::default()).unwrap();
    let p6 = P6Level::load(&w);
    let cw = sly2_recomp::engine::player::p10::player_collision_world(&w);
    for t in &p6.targets {
        let (path, kind) = match &t.obj.geom {
            TargetGeom::PathBody { path } => (path, "pipe"),
            TargetGeom::Rope { path, .. } => (path, "rope"),
            _ => continue,
        };
        let l = path.FUN_001e6648();
        let mut ok = vec![];
        for f in [0.3f32, 0.5, 0.7] {
            let (pt, dir) = path.FUN_001e6f10(l * f);
            for k in 0..16 {
                let a = k as f32 * std::f32::consts::PI / 8.0;
                let d = [a.cos() * 250.0, a.sin() * 250.0];
                let dz = if k % 2 == 0 { -50.0 } else { 150.0 };
                let mut jt = Jt::new([pt[0] + d[0], pt[1] + d[1], pt[2] + dz]);
                p6.apply(&mut jt);
                jt.t = 10.0;
                jt.jts = 2;
                jt.jtbs = 4;
                jt.set_velocity_pub([-d[0] * 1.7, -d[1] * 1.7, 600.0]);
                if jt.FUN_0019b110(&cw, None, &mut ()) && jt.flight.target == Some(t.obj.id) {
                    ok.push((f, k, jt.jtbs));
                }
            }
            if f == 0.5 {
                println!("{kind} {} len {l} mid {pt:?} dir {dir:?}", t.obj.id);
            }
        }
        println!("  ok {ok:?}");
    }
}

fn row14() -> (World, sly2_recomp::engine::player::p6::P6Level, sly2_recomp::engine::physics::query::CollisionWorld) {
    let (l, descs) = rows();
    let lv = l.load_blocking(descs[14]).unwrap();
    let w = World::load(&lv, LoadOptions::default()).unwrap();
    let p6 = sly2_recomp::engine::player::p6::P6Level::load(&w);
    let cw = sly2_recomp::engine::player::p10::player_collision_world(&w);
    (w, p6, cw)
}

/// Pipe 5947 (row 14, class 0xb path): O air pick with the real collision world -> jtbs 0xf, arrival ->
/// jts 8; holding the stick along the pipe climbs it (u and z grow, body on the path), X jumps off.
#[test]
fn p14_pipe_pick_and_climb() {
    use sly2_recomp::engine::player::p5_target::TargetGeom;
    use sly2_recomp::engine::player::{Jt, JtInput};
    let (_w, p6, cw) = row14();
    assert!(p6.p14.sp[0] < 0.0 && p6.p14.sp[1] < p6.p14.sp[0], "climb speeds {:?}", p6.p14.sp);
    let t = p6.targets.iter().find(|t| t.obj.id == 5947).expect("pipe 5947");
    let TargetGeom::PathBody { path } = &t.obj.geom else { panic!("5947 not category F") };
    let l = path.FUN_001e6648();
    let (pt, _) = path.FUN_001e6f10(l * 0.3);
    println!("pipe pt {pt:?} len {l}");
    let mut jt = Jt::new([pt[0] + 250.0, pt[1], pt[2] - 50.0]);
    p6.apply(&mut jt);
    jt.t = 10.0;
    jt.jts = 2;
    jt.jtbs = 4;
    jt.set_velocity_pub([-250.0 * 1.7, 0.0, 600.0]);
    assert!(jt.FUN_0019b110(&cw, None, &mut ()), "pick failed");
    assert_eq!(jt.jtbs, 0xf);
    assert_eq!(jt.flight.target, Some(5947));
    let dt = 1.0 / 60.0;
    for _ in 0..240 {
        jt.update(&cw, JtInput::default(), dt, &mut ());
        if jt.jts == 8 {
            break;
        }
    }
    assert_eq!(jt.jts, 8, "never reached jts 8 (jtbs {})", jt.jtbs);
    // let the enter ASEG finish
    for _ in 0..120 {
        jt.update(&cw, JtInput::default(), dt, &mut ());
    }
    assert_eq!(jt.p5.s8.state, 1, "idle on the pipe");
    let (u0, z0) = (jt.p5.s8.u, jt.phys.pos[2]);
    jt.p5w.stick = [0.0, 1.0];
    for _ in 0..120 {
        jt.update(&cw, JtInput { stick: [0.0, 1.0], jump: false }, dt, &mut ());
    }
    assert_eq!(jt.jts, 8);
    let (u1, z1) = (jt.p5.s8.u, jt.phys.pos[2]);
    println!("climb u {u0} -> {u1}, z {z0} -> {z1}, state {}", jt.p5.s8.state);
    assert!(u1 > u0 + 100.0, "u {u0} -> {u1}");
    assert!(z1 > z0 + 50.0, "z {z0} -> {z1}");
    assert!(matches!(jt.p5.s8.state, 2 | 3));
    let q = path.FUN_001e6f10(u1).0;
    let d = [q[0] - jt.phys.pos[0], q[1] - jt.phys.pos[1], q[2] - jt.phys.pos[2]];
    assert!((d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() < 60.0, "off path {d:?}");
    // releasing the stick stops, X jumps off
    jt.p5w.stick = [0.0, 0.0];
    for _ in 0..30 {
        jt.update(&cw, JtInput::default(), dt, &mut ());
    }
    assert_eq!(jt.p5.s8.state, 1);
    jt.update(&cw, JtInput { stick: [0.0, 0.0], jump: true }, dt, &mut ());
    jt.update(&cw, JtInput::default(), dt, &mut ());
    assert_ne!(jt.jts, 8, "X leaves the pipe");
}

/// Probe (ignored): P6 target categories per row.
#[test]
#[ignore]
fn p14_probe_cats() {
    let (l, descs) = rows();
    for (r, d) in descs.into_iter().enumerate() {
        let Ok(lv) = l.load_blocking(d) else { continue };
        let Ok(w) = World::load(&lv, LoadOptions::default()) else { continue };
        let p6 = sly2_recomp::engine::player::p6::P6Level::load(&w);
        let mut n = std::collections::BTreeMap::new();
        for t in &p6.targets {
            let k = format!("{:?}", t.obj.geom).split(|c: char| !c.is_alphanumeric()).next().unwrap().to_string();
            *n.entry(k).or_insert(0) += 1;
        }
        println!("row {r}: {n:?}");
    }
}

/// Rope 5966 (row 14, class 0xd, category B): real-world pick -> jtbs 10 -> attach kind 3 (jts 7); the
/// stick along the rope moves Sly along the path (edge forces 0x194170).
#[test]
fn p14_rope_pick_and_move_along() {
    use sly2_recomp::engine::player::p5_target::TargetGeom;
    use sly2_recomp::engine::player::{Jt, JtInput};
    let (_w, p6, cw) = row14();
    let t = p6.targets.iter().find(|t| t.obj.id == 5966).expect("rope 5966");
    let TargetGeom::Rope { path } = &t.obj.geom else { panic!("5966 not category B") };
    let l = path.FUN_001e6648();
    let (pt, dir) = path.FUN_001e6f10(l * 0.5);
    let mut jt = Jt::new([pt[0] + 250.0, pt[1], pt[2] - 50.0]);
    p6.apply(&mut jt);
    jt.t = 10.0;
    jt.jts = 2;
    jt.jtbs = 4;
    jt.set_velocity_pub([-250.0 * 1.7, 0.0, 600.0]);
    assert!(jt.FUN_0019b110(&cw, None, &mut ()), "pick failed");
    assert_eq!((jt.jtbs, jt.flight.target), (10, Some(5966)));
    let dt = 1.0 / 60.0;
    for _ in 0..240 {
        jt.update(&cw, JtInput::default(), dt, &mut ());
        if jt.jts == 7 {
            break;
        }
    }
    assert_eq!(jt.jts, 7, "attach (jtbs {})", jt.jtbs);
    assert_eq!(jt.attach.kind, 3);
    for _ in 0..60 {
        jt.update(&cw, JtInput::default(), dt, &mut ());
    }
    let p0 = jt.phys.pos;
    // stick along the rope direction (Jt stick convention: y = world +x, x = world -y)
    let st = [-dir[1], dir[0]];
    for _ in 0..120 {
        jt.update(&cw, JtInput { stick: st, jump: false }, dt, &mut ());
    }
    let p1 = jt.phys.pos;
    let along = (p1[0] - p0[0]) * dir[0] + (p1[1] - p0[1]) * dir[1] + (p1[2] - p0[2]) * dir[2];
    println!("rope prof v0 {} vmax {} blend {:?}", jt.att.rope_v0, jt.att.rope_vmax, jt.att.rope_blend);
    println!("rope jts {} jtbs {} attach sub {} p0 {p0:?} p1 {p1:?} along {along}", jt.jts, jt.jtbs, jt.attach.sub);
    assert_eq!(jt.jts, 7);
    assert!(along > 100.0, "moved {along} along the rope");
}

fn row(r: usize) -> (World, sly2_recomp::engine::player::p6::P6Level, sly2_recomp::engine::physics::query::CollisionWorld) {
    let (l, descs) = rows();
    let lv = l.load_blocking(descs[r]).unwrap();
    let w = World::load(&lv, LoadOptions::default()).unwrap();
    let p6 = sly2_recomp::engine::player::p6::P6Level::load(&w);
    let cw = sly2_recomp::engine::player::p10::player_collision_world(&w);
    (w, p6, cw)
}

/// Air pick of a class 0x13 swing point from 16 approach directions; returns (jt, point id) of the first
/// pick that reaches jtbs 0xe in the real collision world.
fn swing_pick(p6: &sly2_recomp::engine::player::p6::P6Level, cw: &sly2_recomp::engine::physics::query::CollisionWorld, mode: Option<i32>) -> Option<(sly2_recomp::engine::player::Jt, usize)> {
    use sly2_recomp::engine::player::p5_target::TargetGeom;
    use sly2_recomp::engine::player::Jt;
    for t in &p6.targets {
        let TargetGeom::Point { pos, .. } = t.obj.geom else { continue };
        if mode.is_some_and(|m| p6.p14.swing.iter().find(|s| s.0 == t.obj.id).map(|s| s.1) != Some(m)) {
            continue;
        }
        for k in 0..32 {
            let a = (k % 16) as f32 * std::f32::consts::TAU / 16.0;
            let (dz, vz) = if k < 16 { (-100.0, 500.0) } else { (0.0, 300.0) };
            let (c, s) = (a.cos(), a.sin());
            let mut jt = Jt::new([pos[0] + 300.0 * c, pos[1] + 300.0 * s, pos[2] + dz]);
            p6.apply(&mut jt);
            jt.t = 10.0;
            jt.jts = 2;
            jt.jtbs = 4;
            jt.set_velocity_pub([-c * 500.0, -s * 500.0, vz]);
            if jt.FUN_0019b110(cw, None, &mut ()) && jt.jtbs == 0xe && jt.flight.target == Some(t.obj.id) {
                println!("swing pick pt {} pos {pos:?} start {:?} k {k}", t.obj.id, jt.phys.pos);
                return Some((jt, t.obj.id));
            }
        }
    }
    None
}

/// Probe (ignored): swing point picks per row.
#[test]
#[ignore]
fn p14_probe_swing() {
    use sly2_recomp::engine::player::p5_target::TargetGeom;
    for r in [27usize, 30, 32, 36] {
        let (_w, p6, cw) = row(r);
        let n = p6.targets.iter().filter(|t| matches!(t.obj.geom, TargetGeom::Point { .. })).count();
        let s = swing_pick(&p6, &cw, None).map(|(jt, id)| (id, jt.phys.pos));
        println!("row {r}: {n} points, pick {s:?} swing props {:?}", &p6.p14.swing[..p6.p14.swing.len().min(4)]);
    }
}

#[test]
#[ignore]
fn p14_probe_swing_empty() {
    use sly2_recomp::engine::player::p5_target::TargetGeom;
    let (_w, p6, _cw) = row(27);
    let e = sly2_recomp::engine::physics::query::CollisionWorld::default();
    for t in p6.targets.iter().filter(|t| matches!(t.obj.geom, TargetGeom::Point { .. })).take(5) {
        println!("pt {} mask {} range {} geom {:?}", t.obj.id, t.obj.mask, t.obj.range, t.obj.geom);
    }
    println!("empty pick {:?}", swing_pick(&p6, &e, None).map(|(j, id)| (id, j.phys.pos)));
}

#[test]
#[ignore]
fn p14_probe_swing_one() {
    use sly2_recomp::engine::player::Jt;
    let (_w, p6, _cw) = row(27);
    let e = sly2_recomp::engine::physics::query::CollisionWorld::default();
    let pos = [-10197.173, 693.1112, 874.4892];
    for (dz, vz) in [(-250.0, 700.0), (-100.0, 500.0), (0.0, 300.0), (-400.0, 900.0)] {
        let mut jt = Jt::new([pos[0] + 300.0, pos[1], pos[2] + dz]);
        p6.apply(&mut jt);
        jt.t = 10.0;
        jt.jts = 2;
        jt.jtbs = 4;
        jt.set_velocity_pub([-500.0, 0.0, vz]);
        let ok = jt.FUN_0019b110(&e, None, &mut ());
        println!("dz {dz} vz {vz}: ok {ok} jtbs {} tgt {:?} mask {}", jt.jtbs, jt.flight.target, jt.mask_1000);
    }
}

/// Swing point (class 0x13) on a real level with the real collision world: O air pick -> jtbs 0xe ->
/// arrival -> jts 3 swing (mode 1) / jts 0x16 ride (mode 3, +0x124); the JT stays at the rope length from
/// the pivot while swinging with the stick held, and X releases into the air (swing completes).
fn swing_run(r: usize, mode: i32, want_jts: i32, force_mode: Option<i32>) {
    use sly2_recomp::engine::player::JtInput;
    let (_w, mut p6, cw) = row(r);
    let (mut jt, id) = swing_pick(&p6, &cw, Some(mode)).expect("no swing pick");
    if let Some(m) = force_mode {
        p6.p14.swing.iter_mut().filter(|s| s.0 == id).for_each(|s| s.1 = m);
        jt.p14.swing = p6.p14.swing.clone();
    }
    let pivot = p6.targets.iter().find(|t| t.obj.id == id).unwrap().attach.as_ref().unwrap().pos;
    let dt = 1.0 / 60.0;
    for _ in 0..240 {
        jt.update(&cw, JtInput::default(), dt, &mut ());
        if jt.jts == want_jts {
            break;
        }
    }
    assert_eq!(jt.jts, want_jts, "row {r} pt {id}: arrival (jtbs {})", jt.jtbs);
    println!("arrive pos {:?} swing {:?}", jt.phys.pos, jt.p5w.swing.map(|s| s.pos));
    let mut dmin = f32::MAX;
    let mut dmax = 0.0f32;
    let mut zs = (f32::MAX, f32::MIN);
    for i in 0..180 {
        let st = if (i / 40) % 2 == 0 { [0.0, 1.0] } else { [0.0, -1.0] };
        jt.update(&cw, JtInput { stick: st, jump: false }, dt, &mut ());
        let p = jt.phys.pos;
        let d = ((p[0] - pivot[0]).powi(2) + (p[1] - pivot[1]).powi(2) + (p[2] - pivot[2]).powi(2)).sqrt();
        dmin = dmin.min(d);
        dmax = dmax.max(d);
        zs = (zs.0.min(p[2]), zs.1.max(p[2]));
    }
    println!("row {r} pt {id} pivot {pivot:?}: jts {} jtbs {} dist {dmin}..{dmax} z {zs:?}", jt.jts, jt.jtbs);
    assert_eq!(jt.jts, want_jts, "still swinging");
    assert!(dmax < 400.0, "left the pivot: {dmax}");
    jt.update(&cw, JtInput { stick: [0.0, 1.0], jump: true }, dt, &mut ());
    for _ in 0..3 {
        jt.update(&cw, JtInput::default(), dt, &mut ());
    }
    println!("after X: jts {} jtbs {} v {:?}", jt.jts, jt.jtbs, jt.phys.v);
    assert_ne!(jt.jts, want_jts, "X releases");
}

/// jts 3 pendulum swing. Every mode-1 point in the levels sits in owner-local coordinates (owner chain
/// unported: e.g. row 36 obj 341 at (501, -18, -157) inside the level geometry), so the ride point 4834 of
/// row 27 (real position, real collision) is used with +0x124 forced to 1.
#[test]
fn p14_swing_point_completes() {
    swing_run(27, 3, 3, Some(1));
}

#[test]
fn p14_swing_ride_completes() {
    swing_run(27, 3, 0x16, None);
}

#[test]
#[ignore]
fn p14_probe_swing_modes() {
    use sly2_recomp::engine::player::p5_target::TargetGeom;
    let (l, descs) = rows();
    for r in 0..descs.len() {
        let Ok(lv) = l.load_blocking(descs[r]) else { continue };
        let Ok(w) = World::load(&lv, LoadOptions::default()) else { continue };
        let p6 = sly2_recomp::engine::player::p6::P6Level::load(&w);
        for s in &p6.p14.swing {
            if s.1 == 3 {
                continue;
            }
            let pos = p6.targets.iter().find(|t| t.obj.id == s.0).and_then(|t| if let TargetGeom::Point { pos, .. } = t.obj.geom { Some(pos) } else { None });
            println!("row {r} pt {} mode {} f134 {} pos {pos:?} dis {}", s.0, s.1, s.2, w.objects[s.0].disabled);
        }
    }
}

#[test]
#[ignore]
fn p14_probe_edges() {
    use sly2_recomp::engine::player::p5_target::TargetGeom;
    let (l, descs) = rows();
    for r in [14usize, 15, 17, 19, 27] {
        let Ok(lv) = l.load_blocking(descs[r]) else { continue };
        let Ok(w) = World::load(&lv, LoadOptions::default()) else { continue };
        let p6 = sly2_recomp::engine::player::p6::P6Level::load(&w);
        let mut n = (0, 0);
        for t in &p6.targets {
            let Some(a) = &t.attach else { continue };
            if let TargetGeom::Rope { path } = &t.obj.geom {
                if a.f_1bc != 0 { n.0 += 1; if n.0 < 4 { println!("row {r} edge {} f1bc {} c0 {} c4 {} L {} p0 {:?}", t.obj.id, a.f_1bc, a.f_1c0, a.f_1c4, path.FUN_001e6648(), path.FUN_001e6f10(0.0).0); } } else { n.1 += 1; }
            }
        }
        println!("row {r}: edges {} ropes {}", n.0, n.1);
    }
}

/// Edge: row 14 class 0xd edge 5954 (+0x1bc = 1, prop 858; +0x1c0 / +0x1c4 = FUN_001cb908 defaults 1000 / 1):
/// O air pick in the real collision world -> jtbs 10 -> jts 7 attach kind 3 on the edge; the JT moves along it
/// (gravity + the 0x194170 force; the edge descends, so it slides down).
#[test]
fn p14_edge_moves_along() {
    use sly2_recomp::engine::player::p5_target::TargetGeom;
    use sly2_recomp::engine::player::{Jt, JtInput};
    let (_w, p6, cw) = row14();
    let t = p6.targets.iter().find(|t| t.obj.id == 5954).expect("edge 5954");
    assert_eq!(t.attach.as_ref().unwrap().f_1bc, 1);
    let TargetGeom::Rope { path } = &t.obj.geom else { panic!("5954 not category B") };
    let l = path.FUN_001e6648();
    let mut picked = None;
    'o: for f in [0.5f32, 0.3, 0.7] {
        let (pt, dir) = path.FUN_001e6f10(l * f);
        let side = [-dir[1], dir[0]];
        for sgn in [1.0f32, -1.0] {
            let mut jt = Jt::new([pt[0] + side[0] * 250.0 * sgn, pt[1] + side[1] * 250.0 * sgn, pt[2] - 50.0]);
            p6.apply(&mut jt);
            jt.t = 10.0;
            jt.jts = 2;
            jt.jtbs = 4;
            jt.set_velocity_pub([-side[0] * 425.0 * sgn, -side[1] * 425.0 * sgn, 600.0]);
            if jt.FUN_0019b110(&cw, None, &mut ()) && jt.flight.target == Some(5954) {
                picked = Some((jt, dir));
                break 'o;
            }
        }
    }
    let (mut jt, dir) = picked.expect("edge pick failed");
    let dt = 1.0 / 60.0;
    for _ in 0..240 {
        jt.update(&cw, JtInput::default(), dt, &mut ());
        if jt.jts == 7 {
            break;
        }
    }
    assert_eq!((jt.jts, jt.attach.kind), (7, 3), "attach (jtbs {})", jt.jtbs);
    for _ in 0..60 {
        jt.update(&cw, JtInput::default(), dt, &mut ());
    }
    let (p0, u0) = (jt.phys.pos, jt.att.s20);
    let st = [-dir[1], dir[0]];
    for _ in 0..120 {
        jt.update(&cw, JtInput { stick: st, jump: false }, dt, &mut ());
    }
    let p1 = jt.phys.pos;
    let along = (p1[0] - p0[0]) * dir[0] + (p1[1] - p0[1]) * dir[1] + (p1[2] - p0[2]) * dir[2];
    println!("edge jts {} sub {} s20 {u0} -> {} p0 {p0:?} p1 {p1:?} along {along}", jt.jts, jt.attach.sub, jt.att.s20);
    assert_eq!(jt.jts, 7);
    assert!(along.abs() > 100.0, "moved {along} along the edge");
}

/// Probe (ignored): class 0xc rails (category H) per row and real-world picks.
#[test]
#[ignore]
fn p14_probe_rails() {
    use sly2_recomp::engine::player::p5_target::TargetGeom;
    for r in [2usize, 3, 4, 9, 12, 17] {
        let (_w, p6, cw) = row(r);
        let rails: Vec<_> = p6.targets.iter().filter(|t| matches!(t.obj.geom, TargetGeom::Rail { .. })).collect();
        let mut ok = vec![];
        for t in &rails {
            if rail_pick(&p6, &cw, t.obj.id).is_some() {
                ok.push(t.obj.id);
            }
        }
        println!("row {r}: rails {} picks {ok:?}", rails.len());
    }
}

fn rail_pick(p6: &sly2_recomp::engine::player::p6::P6Level, cw: &sly2_recomp::engine::physics::query::CollisionWorld, id: usize) -> Option<(sly2_recomp::engine::player::Jt, [f32; 3])> {
    use sly2_recomp::engine::player::p5_target::TargetGeom;
    use sly2_recomp::engine::player::Jt;
    let t = p6.targets.iter().find(|t| t.obj.id == id)?;
    let TargetGeom::Rail { path, .. } = &t.obj.geom else { return None };
    let l = path.FUN_001e6648();
    for f in [0.5f32, 0.3, 0.7] {
        let (pt, dir) = path.FUN_001e6f10(l * f);
        let side = [-dir[1], dir[0]];
        for (sgn, dz, vz) in [(1.0f32, -50.0, 600.0), (-1.0, -50.0, 600.0), (1.0, 50.0, 300.0), (-1.0, 50.0, 300.0)] {
            let mut jt = Jt::new([pt[0] + side[0] * 250.0 * sgn, pt[1] + side[1] * 250.0 * sgn, pt[2] + dz]);
            p6.apply(&mut jt);
            jt.t = 10.0;
            jt.jts = 2;
            jt.jtbs = 4;
            jt.set_velocity_pub([-side[0] * 425.0 * sgn, -side[1] * 425.0 * sgn, vz]);
            if jt.FUN_0019b110(cw, None, &mut ()) && jt.flight.target == Some(id) {
                return Some((jt, dir));
            }
        }
    }
    None
}

#[test]
#[ignore]
fn p14_probe_rails_empty() {
    use sly2_recomp::engine::player::p5_target::TargetGeom;
    let e = sly2_recomp::engine::physics::query::CollisionWorld::default();
    for r in [2usize, 17] {
        let (_w, p6, _cw) = row(r);
        for t in p6.targets.iter().filter(|t| matches!(t.obj.geom, TargetGeom::Rail { .. })) {
            let TargetGeom::Rail { path, .. } = &t.obj.geom else { continue };
            println!("row {r} rail {} mask {} range {} L {} p {:?} empty pick {}", t.obj.id, t.obj.mask, t.obj.range, path.FUN_001e6648(), path.FUN_001e6f10(0.0).0, rail_pick(&p6, &e, t.obj.id).is_some());
        }
    }
}

#[test]
#[ignore]
fn p14_probe_rail_cost() {
    use sly2_recomp::engine::player::p5_target::{FUN_0019a4a0, TargetGeom};
    let (_w, p6, _cw) = row(2);
    let t = p6.targets.iter().find(|t| t.obj.id == 4622).unwrap();
    let TargetGeom::Rail { path, .. } = &t.obj.geom else { panic!() };
    let (pt, dir) = path.FUN_001e6f10(path.FUN_001e6648() * 0.5);
    let side = [-dir[1], dir[0]];
    let p = [pt[0] + side[0] * 250.0, pt[1] + side[1] * 250.0, pt[2] - 100.0];
    let v = [-side[0] * 425.0, -side[1] * 425.0, 500.0];
    for u in [30.0, 500.0, 951.0, 1400.0] {
        let q = path.FUN_001e6f10(u).0;
        let r = FUN_0019a4a0(p, -1750.0, v, [0.0; 3], q, [0.0; 3]);
        println!("u {u} q {q:?} cost {} t {}", r.cost, r.t);
    }
    println!("pc {:?} pt {pt:?} dir {dir:?}", path.FUN_001e6f60(p).l);
}

/// Wall hug / ledge shimmy: row 2 class 0xc rail 4622 (category H): O air pick in the real collision world ->
/// jtbs 0xb -> jts 7 attach kind 0 hanging on the rail (member = path point); the stick along the rail moves the
/// JT along it (0x191a10 kind 0 lateral speed from the 8-way blend velocity).
#[test]
fn p14_wall_hug_moves_along_rail() {
    use sly2_recomp::engine::player::JtInput;
    let (_w, p6, cw) = row(2);
    let (mut jt, dir) = rail_pick(&p6, &cw, 4622).expect("rail pick failed");
    assert_eq!(jt.jtbs, 0xb);
    let dt = 1.0 / 60.0;
    use sly2_recomp::engine::input::pad::BTN_CIRCLE;
    for _ in 0..240 {
        jt.pad_pending = Some((BTN_CIRCLE, 0));
        jt.update(&cw, JtInput::default(), dt, &mut ());
        if jt.jts == 7 {
            break;
        }
    }
    assert_eq!((jt.jts, jt.attach.kind), (7, 0), "attach (jtbs {})", jt.jtbs);
    for _ in 0..60 {
        jt.pad_pending = Some((BTN_CIRCLE, 0));
        jt.update(&cw, JtInput { stick: [0.0; 2], jump: false }, dt, &mut ());
    }
    let p0 = jt.phys.pos;
    let st = [-dir[1], dir[0]];
    for k in 0..120 {
        jt.pad_pending = Some((BTN_CIRCLE, 0));
        jt.update(&cw, JtInput { stick: st, jump: false }, dt, &mut ());
        if k % 20 == 0 {
            println!("k {k} v190 {:?} s20 {} yaw {} dir {dir:?} pos {:?} v {:?} joy {} {}", jt.att.v190, jt.att.s20, jt.yaw, jt.phys.pos, jt.phys.v, jt.joy.angle, jt.joy.mag);
        }
    }
    let p1 = jt.phys.pos;
    let along = (p1[0] - p0[0]) * dir[0] + (p1[1] - p0[1]) * dir[1] + (p1[2] - p0[2]) * dir[2];
    println!("rail jts {} sub {} p0 {p0:?} p1 {p1:?} along {along} b8 {:?} {:?}", jt.jts, jt.attach.sub, jt.att.b8_vel_c0, jt.att.b8_vel_100);
    assert_eq!(jt.jts, 7);
    assert!(along.abs() > 100.0, "moved {along} along the rail");
}
