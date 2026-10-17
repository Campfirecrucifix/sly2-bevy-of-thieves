//! WP-P15: camera collision (clip broadphase, follow sweep), climb / attach animation blends, owner-chain
//! swing points.

use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::world::{LoadOptions, World};
use std::sync::Arc;

fn rows() -> (
    LevelLoader,
    Vec<sly2_recomp::engine::disc::descriptor::Descriptor>,
) {
    let iso = Arc::new(
        IsoImage::open(concat!(env!("CARGO_MANIFEST_DIR"), "/../Sly 2 - Band of Thieves (USA).iso"))
            .expect("ISO"),
    );
    let t =
        sly2_recomp::engine::disc::descriptor::static_table(&iso.read_file("SCUS_973.16").unwrap())
            .unwrap();
    let l = LevelLoader::new(iso, ModOverlay::default());
    let boot = crate::common::load_blocking(&l, t[25]).unwrap();
    let d = boot
        .header
        .level_table
        .entries
        .iter()
        .map(|e| e.desc)
        .collect();
    (l, d)
}

fn world(r: usize) -> World {
    let (l, descs) = rows();
    let lv = crate::common::load_blocking(&l, descs[r]).unwrap();
    World::load(&lv, LoadOptions::default()).unwrap()
}

/// Probe (ignored): owner chains of the non-ride (mode != 3) swing points.
#[test]
#[ignore]
fn p15_probe_swing_owners() {
    let (l, descs) = rows();
    for r in 0..descs.len() {
        let Ok(lv) = crate::common::load_blocking(&l, descs[r]) else {
            continue;
        };
        let Ok(w) = World::load(&lv, LoadOptions::default()) else {
            continue;
        };
        let p6 = sly2_recomp::engine::player::p6::P6Level::load(&w);
        for s in &p6.p14.swing {
            if s.1 == 3 {
                continue;
            }
            let mut chain = vec![];
            let mut o = Some(s.0);
            while let Some(i) = o {
                let lo = &w.objects[i];
                let x = sly2_recomp::engine::gfx::attach::BoneWorlds::default()
                    .xfm_world(&w, i)
                    .map(|m| m[3]);
                chain.push(format!(
                    "{i}(c{} oid {} slot {} clone {:?} x {:?})",
                    lo.class, lo.oid, lo.parent_slot, lo.clone_of, x
                ));
                o = lo.owner;
            }
            println!("row {r} pt {} mode {}: {}", s.0, s.1, chain.join(" <- "));
            let mut bw = sly2_recomp::engine::gfx::attach::BoneWorlds::default();
            for (id, m) in sly2_recomp::engine::player::p15::instance_frames(&w, &mut bw, s.0) {
                println!("   inst {id} pos {:?}", m[3]);
            }
            if r == 14 || r == 33 || r == 32 {
                for (j, o) in w.objects.iter().enumerate() {
                    let hit = o
                        .clone_of
                        .map_or(false, |c| c == s.0 || Some(c) == w.objects[s.0].owner)
                        || (o.oid == w.objects[s.0].oid && j != s.0);
                    if hit {
                        let x = sly2_recomp::engine::gfx::attach::BoneWorlds::default()
                            .xfm_world(&w, j)
                            .map(|m| m[3]);
                        println!(
                            "   rel {j} c{} oid {} owner {:?} clone {:?} x {:?}",
                            o.class, o.oid, o.owner, o.clone_of, x
                        );
                    }
                }
            }
        }
    }
}

fn row14_cw() -> (
    World,
    sly2_recomp::engine::player::p6::P6Level,
    Arc<sly2_recomp::engine::physics::CollisionWorld>,
) {
    let w = world(14);
    let p6 = sly2_recomp::engine::player::p6::P6Level::load(&w);
    let cw = Arc::new(sly2_recomp::engine::player::p10::player_collision_world(&w));
    (w, p6, cw)
}

fn pipe_pt(p6: &sly2_recomp::engine::player::p6::P6Level, f: f32) -> [f32; 3] {
    use sly2_recomp::engine::player::p5_target::TargetGeom;
    let t = p6
        .targets
        .iter()
        .find(|t| t.obj.id == 5947)
        .expect("pipe 5947");
    let TargetGeom::PathBody { path } = &t.obj.geom else {
        panic!("5947 not category F")
    };
    path.FUN_001e6f10(path.FUN_001e6648() * f).0
}

/// clip_segment broadphase: a short open-air segment beside pipe 5947 is not inside any solid (before the
/// bbox reject the closed class-48 mesh far above classified every segment as inside); a segment driven
/// down through the ground is clipped.
#[test]
fn p15_clip_segment_bbox() {
    let (_w, p6, cw) = row14_cw();
    let p = pipe_pt(&p6, 0.3);
    let a = [p[0] + 250.0, p[1], p[2] + 100.0];
    let b = [p[0] + 400.0, p[1] + 50.0, p[2] + 150.0];
    let open = cw.clip_segment(a, b);
    println!("open {:?}", open.iter().map(|l| l.t).collect::<Vec<_>>());
    assert!(
        !open.iter().any(|l| l.t[0] <= 0.0 && l.t[1] >= 1.0),
        "open air segment fully inside a solid"
    );
    let down = cw.clip_segment(a, [a[0], a[1], a[2] - 20000.0]);
    println!("down {:?}", down.iter().map(|l| l.t).collect::<Vec<_>>());
    assert!(!down.is_empty(), "no ground below the pipe");
}

/// Follow camera collision input: CwQuery over the shared world and the 0x14f110 sweep shorten a camera move
/// into the ground, and leave an open-air move alone.
#[test]
fn p15_cam_sweep_hits_ground() {
    use sly2_recomp::engine::camera::collide::CmSegmentQuery;
    use sly2_recomp::engine::player::p15::{CamCollide, CwQuery, cam_sweep_14f110};
    let (_w, p6, cw) = row14_cw();
    let p = pipe_pt(&p6, 0.3);
    let a = [p[0] + 250.0, p[1], p[2] + 100.0];
    let mut q = CwQuery(cw.clone());
    let iv = q.solid_intervals(a, [a[0], a[1], a[2] - 20000.0]);
    assert!(!iv.is_empty());
    let c = CamCollide {
        cw: Some(cw.clone()),
        near_radius: 30.0,
        dt_real: 1.0 / 60.0,
    };
    // ground surface = first solid entry below `a`; start the camera 100 above it and move 300 down
    let g = a[2] - iv[0].0 * 20000.0;
    let s0 = [a[0], a[1], g + 100.0];
    let v = [0.0, 0.0, -300.0];
    let r = cam_sweep_14f110(&c, 30.0, 300.0, s0, v);
    println!("sweep {v:?} -> {r:?} (first solid t {:?})", iv.first());
    assert!(r[2] > v[2] + 1.0, "sweep went through the ground: {r:?}");
    let small = [5.0, 0.0, 0.0];
    let r2 = cam_sweep_14f110(&c, 10.0, 100.0, [a[0], a[1], a[2] + 2000.0], small);
    assert!((r2[0] - 5.0).abs() < 0.5, "open air move altered: {r2:?}");
    // no world -> pass-through
    assert_eq!(
        cam_sweep_14f110(&CamCollide::default(), 30.0, 100.0, a, v),
        v
    );
}

/// jts 8 climb blend jt+0xfc0 (0x137a30 over jt+0xfa4 / jt+0xfb4) and the attach blends are built for the
/// row 14 JT, and the drive re-weights the climb blend from Jt::p14.wt.
#[test]
fn p15_climb_blends_built() {
    use sly2_recomp::engine::player::p15_anim::{
        FIELD_ATT_HIDE, FIELD_ATT_ROPE, FIELD_PIPE_CLIMB, jt_blends_build, jt_blends_drive,
    };
    use sly2_recomp::engine::playerdraw::{JtAnimator, JtPlay, JtRig, find_jt};
    let w = world(14);
    let jt = find_jt(&w).unwrap();
    let mut an = JtAnimator::new(JtRig::build(&w, jt));
    jt_blends_build(&w, &mut an);
    let b = an.blends.get(&FIELD_PIPE_CLIMB).expect("0xfc0 blend");
    println!(
        "0xfc0 entries {} len {} chans {}",
        b.entries.len(),
        b.length,
        b.channels.len()
    );
    assert_eq!(b.entries.len(), 2);
    assert!(b.entries.iter().all(|e| e.aseg.is_some()));
    assert!(!b.channels.is_empty());
    for f in [FIELD_ATT_HIDE, FIELD_ATT_ROPE] {
        println!("{f:#x} built {}", an.blends.contains_key(&f));
    }
    let mut j = sly2_recomp::engine::player::Jt::new([0.0; 3]);
    j.jts = 8;
    j.p5.s8.state = 3;
    j.p14.wt = [0.25, 0.75];
    an.current = Some(JtPlay::Blend {
        field: FIELD_PIPE_CLIMB,
        t: 0.0,
        svt: 1.0,
    });
    jt_blends_drive(&mut an, &j);
    let Some(JtPlay::Blend { svt, .. }) = an.current else {
        panic!()
    };
    assert_eq!(svt, -1.0, "state 3 plays the blend backwards");
    let b = &an.blends[&FIELD_PIPE_CLIMB];
    assert_eq!((b.entries[0].weight, b.entries[1].weight), (0.25, 0.75));
}

/// Probe (ignored): directory entry / template records of the row 14 swing prefab root 1823.
#[test]
#[ignore]
fn p15_probe_entry_1823() {
    let w = world(14);
    for (ei, e) in w.entries.iter().enumerate() {
        if !e.objects.contains(&1823) && !e.instances.contains(&1825) {
            continue;
        }
        println!(
            "entry {ei} {:?} a {} n {} objs {:?} inst {:?} load_count {}",
            e.dir.name(),
            e.dir.a,
            e.dir.n,
            e.objects,
            e.instances,
            e.load_count
        );
        for t in w.template_instances.iter().filter(|t| t.entry == Some(ei)) {
            println!(
                "  rec {} copy {} owner {:?} bone {} objs {:?} x {:?}",
                t.record,
                t.copy,
                t.owner,
                t.bone,
                t.objects,
                sly2_recomp::engine::gfx::attach::affine_from_bytes(&t.xform)[3]
            );
        }
    }
    for k in 1823..1835 {
        println!(
            "{k} placed {:?} owner {:?}",
            w.life
                .placed
                .get(&k)
                .map(|x| sly2_recomp::engine::gfx::attach::affine_from_bytes(x)[3]),
            w.objects[k].owner
        );
    }
}

/// Owner chain: the row 14 "p_hook" prefab (entry a = 0, n = 18, root 1823 with swing point 1824) yields one
/// mode-1 swing target per placed copy at the class-6 record positions (none at the origin any more).
#[test]
fn p15_owner_chain_swing_points() {
    use sly2_recomp::engine::player::p5_target::TargetGeom;
    let w = world(14);
    let p6 = sly2_recomp::engine::player::p6::P6Level::load(&w);
    let mut bw = sly2_recomp::engine::gfx::attach::BoneWorlds::default();
    let f = sly2_recomp::engine::player::p15::instance_frames(&w, &mut bw, 1824);
    assert_eq!(f.len(), 18, "copies {f:?}");
    assert_eq!(f[0].0, 1824);
    assert!((f[1].1[3][0] - 4571.84).abs() < 0.1 && (f[1].1[3][2] - 911.93).abs() < 0.1, "copy 1825 {:?}", f[1].1[3]);
    let pts: Vec<_> = p6.targets.iter().filter_map(|t| if let TargetGeom::Point { pos, .. } = t.obj.geom { Some((t.obj.id, pos)) } else { None }).collect();
    assert!(pts.len() >= 18);
    assert!(pts.iter().all(|(_, p)| p[0].abs() + p[1].abs() > 100.0), "point at the origin: {pts:?}");
    for (id, _) in &f {
        assert!(pts.iter().any(|p| p.0 == *id), "no target for copy {id}");
        assert_eq!(p6.p14.swing.iter().find(|s| s.0 == *id).map(|s| s.1), Some(1), "copy {id} mode");
    }
}

/// Mode-1 pendulum swing on a real owner-chain point with the real collision world: row 14 O pick -> jtbs 0xe
/// -> jts 3; the body swings around the pivot (stays within the rope length, z changes), X releases.
#[test]
fn p15_pendulum_swing_in_world() {
    use sly2_recomp::engine::player::p5_target::TargetGeom;
    use sly2_recomp::engine::player::{Jt, JtInput};
    let (_w, p6, cw) = row14_cw();
    let mut found = None;
    'o: for t in &p6.targets {
        let TargetGeom::Point { pos, .. } = t.obj.geom else { continue };
        if p6.p14.swing.iter().find(|s| s.0 == t.obj.id).map(|s| s.1) != Some(1) {
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
            if jt.FUN_0019b110(&cw, None, &mut ()) && jt.jtbs == 0xe && jt.flight.target == Some(t.obj.id) {
                found = Some((jt, t.obj.id, pos));
                break 'o;
            }
        }
    }
    let (mut jt, id, pivot) = found.expect("no mode-1 swing pick on row 14");
    let dt = 1.0 / 60.0;
    for _ in 0..240 {
        jt.update(&cw, JtInput::default(), dt, &mut ());
        if jt.jts == 3 {
            break;
        }
    }
    assert_eq!(jt.jts, 3, "pt {id}: arrival (jtbs {})", jt.jtbs);
    let (mut dmax, mut zs) = (0.0f32, (f32::MAX, f32::MIN));
    for i in 0..180 {
        let st = if (i / 40) % 2 == 0 { [0.0, 1.0] } else { [0.0, -1.0] };
        jt.update(&cw, JtInput { stick: st, jump: false }, dt, &mut ());
        let p = jt.phys.pos;
        dmax = dmax.max(((p[0] - pivot[0]).powi(2) + (p[1] - pivot[1]).powi(2) + (p[2] - pivot[2]).powi(2)).sqrt());
        zs = (zs.0.min(p[2]), zs.1.max(p[2]));
    }
    println!("pt {id} pivot {pivot:?}: jts {} dist max {dmax} z {zs:?}", jt.jts);
    assert_eq!(jt.jts, 3, "still swinging");
    assert!(dmax < 400.0, "left the pivot: {dmax}");
    assert!(zs.1 - zs.0 > 5.0, "no pendulum motion: z {zs:?}");
    jt.update(&cw, JtInput { stick: [0.0, 1.0], jump: true }, dt, &mut ());
    for _ in 0..3 {
        jt.update(&cw, JtInput::default(), dt, &mut ());
    }
    assert_ne!(jt.jts, 3, "X releases");
}

/// Records the JT ASEG requests.
#[derive(Default)]
struct Rec(Vec<u32>);
impl sly2_recomp::engine::player::JtAnim for Rec {
    fn play_aseg(&mut self, field: u32, _: i32, _: i32) {
        self.0.push(field);
    }
}

/// Binocucom jt+0xf00 (0x19cc50 / 0x19cd38 / 0x19d330 / 0x18b2c0) on row 0 at the spawn: R3 while standing
/// -> state 0 (enter ASEG jt+0xf0c), its length later state 1 (Sly hidden, stick cleared: no movement), X
/// (0x40 in the 0x4f0 exit mask) -> state 2 (exit ASEG jt+0xf10) -> -1 (shown again).
#[test]
fn p15_binocucom_cycle() {
    use sly2_recomp::engine::game::spawn;
    use sly2_recomp::engine::input::pad::PadState;
    use sly2_recomp::engine::player::p15_binoc::{FIELD_BINOC_ENTER, FIELD_BINOC_EXIT};
    use sly2_recomp::engine::player::{Jt, JtInput, JT_RADIUS};
    let w = world(0);
    let cw = sly2_recomp::engine::player::p10::player_collision_world(&w);
    let (_, c) = spawn::ee_sub_15e9c8(&w, None, spawn::CLASS_SPAWN);
    let p = c.into_iter().find_map(|i| spawn::spawn_position(&w, i)).unwrap();
    let mut jt = Jt::new([p[0], p[1], p[2] + JT_RADIUS + 5.0]);
    let lvl = sly2_recomp::engine::player::p6::P6Level::load(&w);
    lvl.apply(&mut jt);
    println!("binoc lens {:?}", jt.binoc.len);
    assert!(jt.binoc.len[0] > 0.0 && jt.binoc.len[1] > 0.0, "binoc ASEGs {:?}", jt.binoc.len);
    let dt = 1.0 / 60.0;
    let mut an = Rec::default();
    for _ in 0..120 {
        jt.update(&cw, JtInput::default(), dt, &mut an);
    }
    assert_eq!(jt.jts, 0, "standing");
    assert_eq!(jt.binoc.state, -1);
    jt.feed_pad(&PadState { buttons: 0x400, pressed: 0x400, ..Default::default() });
    jt.update(&cw, JtInput::default(), dt, &mut an);
    assert_eq!(jt.binoc.state, 0, "R3 enters the binocucom");
    assert!(an.0.contains(&FIELD_BINOC_ENTER));
    let n = (jt.binoc.len[0] / dt) as usize + 5;
    let fwd = JtInput { stick: [0.0, 1.0], jump: false };
    for _ in 0..n {
        jt.update(&cw, fwd, dt, &mut an);
    }
    assert_eq!(jt.binoc.state, 1, "enter ASEG done -> looking");
    assert!(jt.binoc.hidden);
    let p0 = jt.phys.pos;
    for _ in 0..60 {
        jt.update(&cw, fwd, dt, &mut an);
    }
    let d = ((jt.phys.pos[0] - p0[0]).powi(2) + (jt.phys.pos[1] - p0[1]).powi(2)).sqrt();
    assert!(d < 5.0, "moved {d} while looking");
    assert_eq!(jt.binoc.state, 1);
    jt.feed_pad(&PadState { buttons: 0x40, pressed: 0x40, ..Default::default() });
    jt.update(&cw, JtInput::default(), dt, &mut an);
    assert_eq!(jt.binoc.state, 2, "X leaves");
    assert!(!jt.binoc.hidden);
    assert!(an.0.contains(&FIELD_BINOC_EXIT));
    assert_eq!(jt.jts, 0, "the X press was consumed by the binocucom, no jump");
    for _ in 0..((jt.binoc.len[1] / dt) as usize + 5) {
        jt.update(&cw, JtInput::default(), dt, &mut an);
    }
    assert_eq!(jt.binoc.state, -1);
    assert_eq!(jt.binoc.mode, -1);
    for _ in 0..30 {
        jt.update(&cw, fwd, dt, &mut an);
    }
    let d2 = ((jt.phys.pos[0] - p0[0]).powi(2) + (jt.phys.pos[1] - p0[1]).powi(2)).sqrt();
    assert!(d2 > 20.0, "walks again after leaving ({d2})");
}
