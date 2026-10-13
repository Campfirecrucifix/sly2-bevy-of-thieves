//! WP-P10: player polish on real levels (ledge clearance 0x1997d8, spawn, ...).
use sly2_recomp::engine::disc::descriptor;
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::game::spawn;
use sly2_recomp::engine::physics::CollisionWorld;
use sly2_recomp::engine::player::p6::P6Level;
use sly2_recomp::engine::player::{Jt, JtInput, JT_RADIUS};
use sly2_recomp::engine::world::{LoadOptions, World};
use std::sync::Arc;

const ISO: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../Sly 2 - Band of Thieves (USA).iso");
const DT: f32 = 1.0 / 60.0;
const ROWS: [usize; 6] = [0, 2, 3, 10, 20, 27];

fn level(row: usize) -> Option<Arc<World>> {
    let iso = Arc::new(IsoImage::open(ISO).ok()?);
    let t = descriptor::static_table(&iso.read_file("SCUS_973.16").ok()?)?;
    let l = LevelLoader::new(iso, ModOverlay::default());
    let boot = l.load_blocking(t[25]).ok()?;
    let lv = l.load_blocking(boot.header.level_table.entries[row].desc).ok()?;
    Some(Arc::new(World::load(&lv, LoadOptions::default()).ok()?))
}

fn spawn_of(w: &World) -> Option<[f32; 3]> {
    let (_, c) = spawn::ee_sub_15e9c8(w, None, spawn::CLASS_SPAWN);
    c.into_iter().find_map(|i| spawn::spawn_position(w, i))
}

/// Sly spawned as player::plugin does, idling `n` frames: (pos, jts, jtbs) per frame.
fn idle(w: &World, cw: &CollisionWorld, n: usize) -> Option<([f32; 3], Vec<([f32; 3], i32, i32)>)> {
    let p = spawn_of(w)?;
    let mut jt = Jt::new([p[0], p[1], p[2] + JT_RADIUS + 5.0]);
    P6Level::load(w).apply(&mut jt);
    let mut f = Vec::new();
    for _ in 0..n {
        jt.update(cw, JtInput::default(), DT, &mut ());
        f.push((jt.phys.pos, jt.jts, jt.jtbs));
    }
    Some((p, f))
}

/// 0x1995d8 segment set in the identity frame at the origin: the 11 segments of the JMP init values.
#[test]
fn clearance_segments() {
    use sly2_recomp::engine::player::p4::{FUN_0011e2d8, FUN_001995d8};
    let (s, u) = FUN_0011e2d8([1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0, 0.0]);
    assert!((s[1] - 1.0).abs() < 1e-6 && (u[2] - 1.0).abs() < 1e-6, "{s:?} {u:?}");
    let p = FUN_001995d8(&[[1.0, 0.0, 0.0], s, u, [0.0; 3]]);
    assert_eq!(p[0], [50.0, 0.0, 75.0]);
    assert_eq!(p[1], [50.0, 0.0, -150.0]);
    assert_eq!(p[2], [-60.0, 0.0, 75.0]);
    assert_eq!(p[3], [50.0, 0.0, 75.0]);
    // middle fan segment (k = 0) straight behind, the outer ones at +-60 degrees, radius 60
    assert!((p[12][0] + 60.0).abs() < 1e-4 && p[12][1].abs() < 1e-4 && p[13][2] == -50.0);
    for i in (4..22).step_by(2) {
        let r = (p[i][0] * p[i][0] + p[i][1] * p[i][1]).sqrt();
        assert!((r - 60.0).abs() < 1e-3 && p[i][2] == 75.0 && p[i + 1][2] == -50.0);
    }
    assert!((p[4][1].atan2(p[4][0]).abs() - 2.0944).abs() < 1e-3);
}

/// WP-P10: Sly idles grounded (not hanging, jtbs 0x11) at the spawn of rows 0, 2, 3, 10, 20, 27.
#[test]
fn spawn_idle_grounded() {
    for row in ROWS {
        let Some(w) = level(row) else { eprintln!("ISO missing"); return };
        let cw = sly2_recomp::engine::player::p10::player_collision_world(&w);
        let (sp, f) = idle(&w, &cw, 300).unwrap();
        let hang = f.iter().filter(|x| x.2 == 0x11).count();
        let maxz = f.iter().map(|x| x.0[2]).fold(f32::MIN, f32::max);
        let (p, jts, jtbs) = *f.last().unwrap();
        eprintln!("row {row}: spawn {sp:?} end {p:?} jts {jts}/{jtbs} hang frames {hang} max z {maxz}");
        assert_eq!(hang, 0, "row {row}: hangs at spawn");
        assert_eq!(jts, 0, "row {row}: not standing");
        assert!(maxz < sp[2] + JT_RADIUS + 50.0, "row {row}: spawn pop to z {maxz}");
    }
}

#[test]
#[ignore]
fn diag_row10() {
    use sly2_recomp::engine::physics::bsp::FUN_00118668;
    let w = level(10).unwrap();
    let cw = CollisionWorld::build_so(&w);
    let sp = spawn_of(&w).unwrap();
    let (a, b) = ([sp[0], sp[1], sp[2] - 300.0], [sp[0], sp[1], sp[2] + 900.0]);
    for o in [1102usize, 1103] {
        let lo = &w.objects[o];
        let own = lo.owner.map(|x| (x, w.objects[x].class, w.objects[x].oid));
        eprintln!("obj {o}: oid {} owner {:?} filter {:?} disabled {} clone_of {:?} entry {}", lo.oid, own, lo.filter, lo.disabled, lo.clone_of, lo.entry);
        for c in cw.colliders.iter().filter(|c| c.object == o) {
            let (mut lo_, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
            for v in &c.geom.verts { for k in 0..3 { lo_[k] = lo_[k].min(v[k]); hi[k] = hi[k].max(v[k]); } }
            eprintln!("  part bbox {lo_:?} {hi:?} nverts {}", c.geom.verts.len());
        }
    }
    for c in &cw.colliders {
        let mut out = Vec::new();
        let n = FUN_00118668(&c.geom, &c.bsp, 0, c.bsp.nodes.len(), a, b, 8, &mut out);
        for l in out.iter().take(n) {
            let fl = match (&w.objects[c.object].payload, c.source) { (sly2_recomp::engine::world::objects::Payload::Loaded(sly2_recomp::engine::world::objects::LoadData::Alo { alo, .. }), sly2_recomp::engine::physics::query::Source::AloPart(k)) => format!("ref {} b0 {} b1 {} b2 {} w0 {:#x} w1 {:#x} s0 {} s1 {}", alo.parts[k].r#ref, alo.parts[k].b0, alo.parts[k].b1, alo.parts[k].b2, alo.parts[k].w0, alo.parts[k].w1, alo.parts[k].s0, alo.parts[k].s1), _ => String::new() };
            eprintln!("obj {} class {:?} src {:?} {fl}: z {} .. {} nz {} {}", c.object, w.objects.get(c.object).map(|o| o.class), c.source, l.pos[0][2], l.pos[1][2], l.plane[0][2], l.plane[1][2]);
        }
    }
}

#[test]
#[ignore]
fn diag_compare_so() {
    for row in ROWS {
        let w = level(row).unwrap();
        let a = CollisionWorld::build_so(&w);
        let b = sly2_recomp::engine::player::p10::player_collision_world(&w);
        let (_, fa) = idle(&w, &a, 300).unwrap();
        let (_, fb) = idle(&w, &b, 300).unwrap();
        eprintln!("row {row}: sensors {} so end {:?} {}/{} | p10 end {:?}", a.colliders.len() - b.colliders.len(), fa[299].0, fa[299].1, fa[299].2, fb[299].0);
    }
}

#[test]
#[ignore]
fn diag_row27_run() {
    use sly2_recomp::engine::input::pad::BTN_CROSS;
    let w = level(27).unwrap();
    for (name, cw) in [("so", CollisionWorld::build_so(&w)), ("p10", sly2_recomp::engine::player::p10::player_collision_world(&w))] {
        let p = spawn_of(&w).unwrap();
        let mut jt = Jt::new([p[0], p[1], p[2] + JT_RADIUS + 5.0]);
        P6Level::load(&w).apply(&mut jt);
        for i in 0..900 {
            let mut inp = JtInput::default();
            if (600..720).contains(&i) { inp.stick = [0.0, 1.0]; }
            let pressed = if i == 720 || i == 740 { BTN_CROSS } else { 0 };
            inp.jump = pressed != 0;
            jt.pad_pending = Some((pressed, pressed));
            jt.update(&cw, inp, DT, &mut ());
            if i % 20 == 0 && i >= 580 { eprintln!("{name} f{i}: {:?} v {:?} jts {}/{}", jt.phys.pos, jt.phys.v, jt.jts, jt.jtbs); }
        }
    }
}
