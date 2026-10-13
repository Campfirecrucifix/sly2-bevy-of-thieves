//! WP-A7 (render side): class-81 projectiles, class-89 water volumes, class-76 swarm riders drawn from
//! their real ALO data; effect paths.
use sly2_recomp::engine::classes::guard::PLAYER_HANDLE;
use sly2_recomp::engine::classes::plugin::EngineHost;
use sly2_recomp::engine::classes::k7::K7;
use sly2_recomp::engine::classes::{ClassSet, Inst};
use sly2_recomp::engine::gfx::attach::BoneWorlds;
use sly2_recomp::engine::gfx::scene::LevelScene;
use sly2_recomp::engine::npcdraw::{objdata, template_of};
use sly2_recomp::engine::disc::descriptor::Descriptor;
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::world::{LoadOptions, World};
use std::sync::Arc;

fn levels() -> (LevelLoader, Vec<Descriptor>) {
    crate::common::levels() // public: ISO level table instead of the analysis DB
}

fn setup(w: &Arc<World>) -> (ClassSet, EngineHost) {
    let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
    let mut s = ClassSet::build(w, &mut h);
    s.g.k6_players = vec![(PLAYER_HANDLE, [1.0e9; 3])];
    s.post_load(&mut h);
    (s, h)
}

const DT: f32 = 1.0 / 60.0;

fn rows(loader: &LevelLoader, descs: &[Descriptor]) -> Vec<u32> {
    let l = crate::common::load_blocking(&loader, descs[0]).unwrap();
    l.header.level_table.entries.iter().map(|e| e.desc.lsn()).collect()
}

fn prims(scene: &LevelScene, o: usize) -> usize {
    scene.objects.iter().find(|so| so.object == o).map(|so| so.prims.len()).unwrap_or(0)
}

/// class 81 on every level: not in flight at load; FUN_001d6b00 (k14_fire) puts it in flight and the drawn
/// body follows the physics (template mesh from the level scene)
#[test]
#[ignore = "heavy"]
fn a7_projectiles_drawn_in_flight() {
    let (loader, descs) = levels();
    let (mut n, mut fired, mut meshes) = (0, 0, 0);
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let (mut s, mut h) = setup(&w);
        let projs: Vec<usize> = s.k13.iter().filter(|x| matches!(x.1, Inst::K13(_))).map(|x| x.0).collect();
        if projs.is_empty() {
            continue;
        }
        let scene = LevelScene::build(&lv, &w);
        for &p in &projs {
            n += 1;
            assert!(!objdata::proj_in_flight(p, &s.k13_get(p).unwrap().p), "not in flight at load");
            if prims(&scene, template_of(&w, p)) > 0 {
                meshes += 1;
            }
        }
        let Some(l) = s.k7.iter().find(|x| matches!(x.1, Inst::K7(K7::C2d7f40(_)))).map(|x| x.0) else { continue };
        let p = projs[0];
        s.k13_get(p).unwrap().p.phys.pos = [100.0, 200.0, 300.0];
        s.g.dt = DT;
        s.k14_fire(l, p, &mut h).expect("launch");
        let c = s.k13_get(p).unwrap();
        assert!(objdata::proj_in_flight(p, &c.p));
        let m0 = objdata::proj_world(&c.p);
        let r = m0[0];
        assert!(((r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt() - 1.0).abs() < 1e-3, "orientation rows FUN_0011b7e0");
        for _ in 0..10 {
            s.g.t += DT;
            s.frame(&mut h);
        }
        let c = s.k13_get(p).unwrap();
        let m1 = objdata::proj_world(&c.p);
        assert_eq!(m1[3], c.p.phys.pos);
        assert!(m1[3] != m0[3], "the drawn body moves");
        fired += 1;
    }
    println!("{n} class-81 instances, {meshes} with template meshes, {fired} levels fired");
    assert!(n > 0 && fired > 0 && meshes > 0);
}

/// class 89: clones (no entities of their own) are drawn at their own placement, templates have meshes
#[test]
#[ignore = "heavy"]
fn a7_water_clones_placed() {
    let (loader, descs) = levels();
    let (mut clones, mut moved) = (0, 0);
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let (s, _h) = setup(&w);
        let cs: Vec<usize> = s.k13.iter().filter(|x| matches!(x.1, Inst::K13Water(_)) && w.objects[x.0].clone_of.is_some()).map(|x| x.0).collect();
        if cs.is_empty() {
            continue;
        }
        let scene = LevelScene::build(&lv, &w);
        let mut bw = BoneWorlds::default();
        for o in cs {
            let t = template_of(&w, o);
            assert_eq!(prims(&scene, o), 0, "clone has no entities of its own");
            clones += 1;
            let (a, b) = (objdata::clone_world(&w, &mut bw, o), objdata::clone_world(&w, &mut bw, t));
            if (0..3).any(|k| (a[3][k] - b[3][k]).abs() > 1.0) {
                moved += 1;
            }
            println!("lsn {} water clone {o} of {t} ({} prims) at {:?} (template {:?})", d.lsn(), prims(&scene, t), a[3], b[3]);
        }
    }
    assert!(clones > 0 && moved > 0);
}

/// class 76: one drawn rider per alive rider (slot 18 matrices), the +0x1c4 model has meshes
#[test]
fn a7_swarm_riders_one_copy_each() {
    let (loader, descs) = levels();
    let rs = rows(&loader, &descs);
    let mut swarms = 0;
    for row in [14usize, 15, 22, 32, 38] {
        let d = descs.iter().find(|d| d.lsn() == rs[row]).copied().unwrap();
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let (mut s, mut h) = setup(&w);
        s.g.k6_players = vec![(PLAYER_HANDLE, [0.0; 3])];
        s.g.dt = DT;
        for _ in 0..30 {
            s.g.t += DT;
            s.frame(&mut h);
        }
        let scene = LevelScene::build(&lv, &w);
        for (o, x) in &s.k14 {
            let Inst::K14(c) = x else { continue };
            let m = objdata::swarm_model(&w, c).expect("model +0x1c4");
            assert!(prims(&scene, m) > 0, "model {m} has meshes");
            let ws = objdata::rider_worlds(c, *o, &mut h);
            assert_eq!(ws.len() as i32, c.groups.iter().map(|g| g.list.len() as i32).sum::<i32>());
            assert!(ws.len() <= c.riders.len(), "copies are made per rider record");
            swarms += 1;
        }
    }
    assert!(swarms >= 5);
}

// ---------------------------------------------------------------- group 2/4: collect effect (type 9) 1:1

use sly2_recomp::engine::gfx::fx::{
    collect_desc, collect_size_alpha, ee_sub_14f268, ee_sub_14f308, ee_sub_1d04e8, ee_sub_1d05a0, ee_sub_1d0630_rot, CamFrame, M3, COLLECT_LIFE, DESC_COIN, DESC_HEALTH,
};

fn elf() -> Vec<u8> {
    let mut root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    if !root.join("Sly 2 - Band of Thieves (USA).iso").exists() {
        root = concat!(env!("CARGO_MANIFEST_DIR"), "/..").into();
    }
    IsoImage::open(root.join("Sly 2 - Band of Thieves (USA).iso")).expect("ISO").read_file("SCUS_973.16").unwrap()
}

fn cam() -> CamFrame {
    // forward (+x level) tilted down, left, up
    let (c, s) = (0.3f32.cos(), 0.3f32.sin());
    CamFrame { pos: [100.0, -50.0, 300.0], rot: [[c, 0.0, -s], [0.0, 1.0, 0.0], [s, 0.0, c]], x_scale: 0.9, y_scale: 0.5 }
}

fn close(a: [f32; 3], b: [f32; 3], eps: f32) -> bool {
    (0..3).all(|k| (a[k] - b[k]).abs() <= eps)
}

#[test]
fn a7_collect_descriptors_match_elf() {
    use sly2_recomp::engine::disc::descriptor::elf_bytes;
    let elf = elf();
    for (va, d) in [(DESC_COIN, collect_desc(DESC_COIN).unwrap()), (DESC_HEALTH, collect_desc(DESC_HEALTH).unwrap())] {
        let b = elf_bytes(&elf, va, 0x70).unwrap();
        let f = |o: usize| f32::from_le_bytes(b[o..o + 4].try_into().unwrap());
        let v = |o: usize| [f(o), f(o + 4), f(o + 8)];
        assert_eq!(v(0x00), d.target);
        assert_eq!(v(0x10), d.target_vel);
        assert_eq!(v(0x20), d.vel_min);
        assert_eq!(v(0x30), d.vel_max);
        assert_eq!(v(0x40), d.spin_axis);
        assert_eq!(v(0x50), d.size);
        assert_eq!(u32::from_le_bytes(b[0x64..0x68].try_into().unwrap()), d.arrive);
    }
    // DAT_002c2000 life, DAT_002bbb50 alpha curve
    assert_eq!(f32::from_le_bytes(elf_bytes(&elf, 0x002c_2000, 4).unwrap().try_into().unwrap()), COLLECT_LIFE);
    assert_eq!(f32::from_le_bytes(elf_bytes(&elf, 0x002b_bb50, 4).unwrap().try_into().unwrap()), 1.0);
}

#[test]
fn a7_screen_projection_roundtrip() {
    let c = cam();
    let p = [600.0, 40.0, 250.0];
    let s = ee_sub_14f308(&c, p);
    assert!(s[2] > 0.0 && s[0] < 0.0, "point left of the view axis is left on screen: {s:?}");
    assert!(close(ee_sub_14f268(&c, s), p, 1e-2));
    // same as the camera core port (camera::cm) on an equivalent Cm
    let mut cm = sly2_recomp::engine::camera::cm::Cm::default();
    cm.mat = [[c.rot[0][0], c.rot[0][1], c.rot[0][2], 0.0], [c.rot[1][0], c.rot[1][1], c.rot[1][2], 0.0], [c.rot[2][0], c.rot[2][1], c.rot[2][2], 0.0], [c.pos[0], c.pos[1], c.pos[2], 1.0]];
    cm.x_scale = c.x_scale;
    cm.y_scale = c.y_scale;
    assert!(close(sly2_recomp::engine::camera::cm::FUN_0014f308(&cm, p), s, 1e-4));
}

#[test]
fn a7_collect_flight_reaches_hud_target() {
    let c = cam();
    let d = collect_desc(DESC_COIN).unwrap();
    let mut draws = vec![];
    let mut st = ee_sub_1d04e8(d, &c, [600.0, 40.0, 250.0], |lo, hi| {
        draws.push((lo, hi));
        (lo + hi) * 0.5
    });
    assert_eq!(draws, vec![(-1.0, 1.0), (2.0, 3.0), (0.0, 0.0)]);
    assert_eq!(st.vel, [0.0, 2.5, 0.0]);
    let (dt, mut age) = (1.0 / 60.0, 0.0f32);
    let mut first = None;
    while age + dt <= COLLECT_LIFE + 1e-6 {
        age += dt;
        let p = ee_sub_1d05a0(&mut st, d, &c, COLLECT_LIFE - age, dt);
        first.get_or_insert(p);
    }
    // the screen point ends on the HUD target at depth 100, moving up first (start velocity +y)
    assert!(close(st.screen, d.target, 2e-3), "{:?}", st.screen);
    assert!(close(ee_sub_14f268(&c, st.screen), ee_sub_14f268(&c, d.target), 0.5));
    let (s1, a1) = collect_size_alpha(d, 1.0);
    assert!((s1 - 0.1).abs() < 1e-6 && a1 == 1.0);
}

#[test]
fn a7_collect_rotation_object_to_camera() {
    let c = cam();
    let (cz, sz) = (1.1f32.cos(), 1.1f32.sin());
    let obj: M3 = [[cz, sz, 0.0], [-sz, cz, 0.0], [0.0, 0.0, 1.0]];
    let r0 = ee_sub_1d0630_rot(&obj, &c.rot, 0.0, [0.0, 0.0, 1.0]);
    let r1 = ee_sub_1d0630_rot(&obj, &c.rot, 1.0, [0.0, 0.0, 1.0]);
    for k in 0..3 {
        assert!(close(r0[k], obj[k], 1e-4), "u=0 {r0:?}");
        assert!(close(r1[k], c.rot[k], 1e-3), "u=1 {r1:?}");
    }
    // half way: still orthonormal
    let h = ee_sub_1d0630_rot(&obj, &c.rot, 0.5, [0.0, 0.0, 1.0]);
    let dot = |a: [f32; 3], b: [f32; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    assert!((dot(h[0], h[0]) - 1.0).abs() < 1e-4 && dot(h[0], h[1]).abs() < 1e-4);
}
