//! WP-P17: pipe-climb pose (JT member binding of the STEP table 0x2bc9f0), swing camera, pole stick, launch blend.

use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::world::{LoadOptions, World};
use std::sync::Arc;

fn world(r: usize) -> World {
    let iso = Arc::new(IsoImage::open(concat!(env!("CARGO_MANIFEST_DIR"), "/../Sly 2 - Band of Thieves (USA).iso")).expect("ISO"));
    let t = sly2_recomp::engine::disc::descriptor::static_table(&iso.read_file("SCUS_973.16").unwrap()).unwrap();
    let l = LevelLoader::new(iso, ModOverlay::default());
    let boot = l.load_blocking(t[25]).unwrap();
    let d: Vec<_> = boot.header.level_table.entries.iter().map(|e| e.desc).collect();
    let lv = l.load_blocking(d[r]).unwrap();
    World::load(&lv, LoadOptions::default()).unwrap()
}

#[test]
#[ignore]
fn p17_probe_members() {
    use sly2_recomp::engine::playerdraw::{aseg_of, ee_sub_15e938_below, find_jt, JT_TABLE};
    use sly2_recomp::engine::playerdraw::vis::{JT_BIND_2BC9F0, JT_BIND_2BF1B0};
    let w = world(14);
    let jt = find_jt(&w).unwrap();
    for (name, t) in [("2be990", &JT_TABLE[..]), ("2bc9f0", &JT_BIND_2BC9F0[..]), ("2bf1b0", &JT_BIND_2BF1B0[..])] {
        for &(fl, oid, m) in t {
            if let Some(o) = ee_sub_15e938_below(&w, jt, oid) {
                let a = aseg_of(&w, o);
                println!("{name} {fl:#x} oid {oid:#x} -> {m:#x}: obj {o} class {} aseg len {:?} chans {:?}", w.objects[o].class, a.map(|a| a.length), a.map(|a| a.channels.len()));
            }
        }
    }
}

#[test]
#[ignore]
fn p17_probe_climb_pose() {
    use sly2_recomp::engine::player::p15_anim::{jt_blends_build, jt_blends_drive, FIELD_PIPE_CLIMB};
    use sly2_recomp::engine::playerdraw::{find_jt, JtAnimator, JtRig};
    let w = world(14);
    let jt = find_jt(&w).unwrap();
    let mut an = JtAnimator::new(JtRig::build(&w, jt));
    jt_blends_build(&w, &mut an);
    let b = &an.blends[&FIELD_PIPE_CLIMB];
    println!("chans {} unbound {} len {} w {:?}", b.channels.len(), b.channels.iter().filter(|c| c.bone.is_none()).count(), b.length, b.entries.iter().map(|e| (e.aseg, e.weight, e.sub_length)).collect::<Vec<_>>());
    let rest: Vec<_> = an.rig.pose.bones.iter().map(|b| b.local).collect();
    let diff = |an: &JtAnimator| -> f32 {
        an.rig.pose.bones.iter().zip(&rest).map(|(b, r)| (0..4).flat_map(|i| (0..4).map(move |j| (i, j))).map(|(i, j)| (b.local[i][j] - r[i][j]).abs()).fold(0.0, f32::max)).fold(0.0, f32::max)
    };
    let mut j = sly2_recomp::engine::player::Jt::new([0.0; 3]);
    j.jts = 8;
    j.p5.s8.state = 2;
    j.p14.wt = [1.0, 0.0];
    an.request(&w, FIELD_PIPE_CLIMB, 8);
    for f in 0..40 {
        jt_blends_drive(&mut an, &j);
        an.step(1.0 / 60.0);
        if f % 5 == 0 {
            println!("blend f{f} cur {:?} diff {}", an.current_field(), diff(&an));
        }
    }
    let mut an2 = JtAnimator::new(JtRig::build(&w, jt));
    an2.request(&w, 0xfa4, 8);
    for f in 0..40 {
        an2.step(1.0 / 60.0);
        if f % 5 == 0 {
            println!("aseg f{f} cur {:?} diff {}", an2.current_field(), diff(&an2));
        }
    }
    let d: f32 = an.rig.pose.bones.iter().zip(&an2.rig.pose.bones).map(|(a, b)| (0..3).map(|i| (0..3).map(|k| (a.local[i][k] - b.local[i][k]).abs()).fold(0.0, f32::max)).fold(0.0, f32::max)).fold(0.0, f32::max);
    println!("blend vs aseg rot diff {d}");
}

#[test]
#[ignore]
fn p17_probe_climb_bones() {
    use sly2_recomp::engine::player::p15_anim::{jt_blends_build, FIELD_PIPE_CLIMB};
    use sly2_recomp::engine::playerdraw::{find_jt, JtAnimator, JtRig, FIELD_RUN};
    let w = world(14);
    let jt = find_jt(&w).unwrap();
    let mut an = JtAnimator::new(JtRig::build(&w, jt));
    jt_blends_build(&w, &mut an);
    let n = an.rig.pose.bones.len();
    let set = |f: u32| -> std::collections::BTreeSet<usize> { an.blends[&f].channels.iter().filter_map(|c| c.bone).collect() };
    let (c, r) = (set(FIELD_PIPE_CLIMB), set(FIELD_RUN));
    println!("bones {n} climb {} run {}", c.len(), r.len());
    for i in 0..n {
        let (o, k) = an.rig.keys[i];
        println!("bone {i} obj {o} oid {:#x} k {k} parent {:?} climb {} run {}", w.objects[o].oid, an.rig.pose.bones[i].parent, c.contains(&i), r.contains(&i));
    }
    for c in &an.blends[&FIELD_PIPE_CLIMB].channels {
        println!("chan key {:#x} bone {:?}", c.key, c.bone);
    }
}

#[test]
#[ignore]
fn p17_probe_climb_world() {
    use sly2_recomp::engine::player::p15_anim::{jt_blends_build, jt_blends_drive, FIELD_PIPE_CLIMB};
    use sly2_recomp::engine::playerdraw::{find_jt, JtAnimator, JtRig, FIELD_RUN};
    let w = world(14);
    let jt = find_jt(&w).unwrap();
    let mk = || { let mut a = JtAnimator::new(JtRig::build(&w, jt)); jt_blends_build(&w, &mut a); a };
    let mut an = mk();
    an.rig.pose.compose(&sly2_recomp::engine::anim::pose::IDENTITY);
    let rest = an.rig.pose.world();
    let mut j = sly2_recomp::engine::player::Jt::new([0.0; 3]);
    j.jts = 8;
    j.p5.s8.state = 2;
    j.p14.wt = [0.0, 1.0];
    an.request(&w, FIELD_PIPE_CLIMB, 8);
    for _ in 0..20 { jt_blends_drive(&mut an, &j); an.step(1.0 / 60.0); }
    let climb = an.rig.pose.world();
    let mut rn = mk();
    rn.request(&w, FIELD_RUN, 6);
    for _ in 0..20 { rn.step(1.0 / 60.0); }
    let run = rn.rig.pose.world();
    for i in 0..139 {
        let f = |m: &[[f32; 4]; 4]| format!("({:7.1},{:7.1},{:7.1})", m[3][0], m[3][1], m[3][2]);
        println!("b{i:3} kind {:2} rest {} climb {} run {}", an.rig.pose.bones[i].kind, f(&rest[i]), f(&climb[i]), f(&run[i]));
    }
}

#[test]
#[ignore]
fn p17_probe_aseg_chans() {
    use sly2_recomp::engine::playerdraw::{aseg_of, find_jt, JtAnimator, JtRig};
    let w = world(14);
    let jt = find_jt(&w).unwrap();
    let an = JtAnimator::new(JtRig::build(&w, jt));
    for f in [0xfb4u32, 0x1334] {
        let o = an.rig.members[&f];
        let a = aseg_of(&w, o).unwrap();
        println!("== field {f:#x} obj {o} len {} chans {} scalar list {}", a.length, a.channels.len(), a.scalars.len());
        for c in &a.channels {
            println!("  oid {:#x} kind {} vec {} scalar {} flag {}", c.oid, c.kind, c.vec.is_some(), c.scalar.is_some(), c.flag);
        }
    }
}

fn climb_animator(w: &World) -> sly2_recomp::engine::playerdraw::JtAnimator {
    use sly2_recomp::engine::player::p15_anim::{jt_blends_build, jt_blends_drive, FIELD_PIPE_CLIMB};
    use sly2_recomp::engine::playerdraw::{find_jt, JtAnimator, JtRig};
    let jt = find_jt(w).unwrap();
    let mut an = JtAnimator::new(JtRig::build(w, jt));
    jt_blends_build(w, &mut an);
    let mut j = sly2_recomp::engine::player::Jt::new([0.0; 3]);
    j.jts = 8;
    j.p5.s8.state = 2;
    j.p14.wt = [0.0, 1.0];
    an.request(w, FIELD_PIPE_CLIMB, 8);
    for _ in 0..20 {
        jt_blends_drive(&mut an, &j);
        an.step(1.0 / 60.0);
    }
    an
}

/// WP-P17: the JT ALO carries four FUN_00186670 two-bone IK records (2 legs, 2 arms) and
/// FUN_00186a50 moves the limb bones so the chain end reaches the animated goal.
#[test]
fn p17_ik_records_and_reach() {
    let w = world(14);
    let an = climb_animator(&w);
    let iks = an.ik.iks.clone();
    assert_eq!(iks.len(), 4, "IK records {:?}", iks.iter().map(|k| (k.b1, k.b2, k.b3, k.goal)).collect::<Vec<_>>());
    let rest = an.rig.rest_world.clone();
    let pt = |m: &[[f32; 4]; 4], v: [f32; 3]| -> [f32; 3] { std::array::from_fn(|j| v[0] * m[0][j] + v[1] * m[1][j] + v[2] * m[2][j] + m[3][j]) };
    let dist = |a: [f32; 3], b: [f32; 3]| ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
    let wv = an.rig.pose.world();
    for k in &iks {
        let end = pt(&wv[k.b3], k.end_off);
        let goal = pt(&wv[k.goal], k.goal_off);
        let moved = dist([wv[k.b2][3][0], wv[k.b2][3][1], wv[k.b2][3][2]], [rest[k.b2][3][0], rest[k.b2][3][1], rest[k.b2][3][2]]);
        println!("ik b1 {} b2 {} b3 {} goal {} l {:.1}/{:.1} a0 {:.3} sw {:.3} end-goal {:.2} b2 moved {:.1}", k.b1, k.b2, k.b3, k.goal, k.l1, k.l2, k.angle0, k.swivel, dist(end, goal), moved);
        assert!(moved > 1.0, "limb not posed");
        let reach = k.l1 + k.l2;
        let s = [wv[k.b1][3][0], wv[k.b1][3][1], wv[k.b1][3][2]];
        if dist(goal, s) < reach * 0.98 {
            assert!(dist(end, goal) < 1.0, "end {end:?} goal {goal:?}");
        }
    }
}

/// FUN_0011e4b0 angle/axis and FUN_0011b500 rotation (row vectors): rotating a by the angle
/// about the axis gives b's direction.
#[test]
fn p17_ik_math() {
    use sly2_recomp::engine::playerdraw::ik::{ee_sub_11b500, ee_sub_11e4b0};
    let (a, b) = ([1.0, 0.0, 0.0], [0.0, 2.0, 0.0]);
    let (ang, ax) = ee_sub_11e4b0(a, b);
    assert!((ang - std::f32::consts::FRAC_PI_2).abs() < 1e-5 && (ax[2] - 1.0).abs() < 1e-6);
    let m = ee_sub_11b500(ang, ax);
    let r: [f32; 3] = std::array::from_fn(|j| a[0] * m[0][j] + a[1] * m[1][j] + a[2] * m[2][j]);
    assert!((r[0]).abs() < 1e-5 && (r[1] - 1.0).abs() < 1e-5, "{r:?}");
    assert_eq!(ee_sub_11e4b0([0.0; 3], b), (0.0, [1.0, 0.0, 0.0]));
}

/// 0x00223a70: clear line -> want kept (mode untouched); a wall on the line -> mode 4 and a rotated
/// placement whose line from the focus is clear.
#[test]
fn p17_cut_placement_223a70() {
    use sly2_recomp::engine::camera::cm::{Cm, CmWorld, Cpdefi};
    use sly2_recomp::engine::camera::collide::CmSegmentQuery;
    use sly2_recomp::engine::camera::follow::{CplcyFollow, FollowWorld};
    /// solid slab 40 <= x <= 60 for y < 30 (a pillar edge)
    struct Wall;
    impl CmSegmentQuery for Wall {
        fn solid_intervals(&mut self, a: [f32; 3], b: [f32; 3]) -> Vec<(f32, f32)> {
            let dx = b[0] - a[0];
            if dx.abs() < 1e-6 {
                return vec![];
            }
            let (t0, t1) = (((40.0 - a[0]) / dx).clamp(0.0, 1.0), ((60.0 - a[0]) / dx).clamp(0.0, 1.0));
            let (t0, t1) = (t0.min(t1), t0.max(t1));
            let ym = a[1] + (b[1] - a[1]) * (t0 + t1) * 0.5;
            if t1 > t0 && ym < 30.0 { vec![(t0, t1)] } else { vec![] }
        }
    }
    struct W(Wall, bool);
    impl CmWorld for W {
        fn get_cpdefi(&mut self, _: u32) -> Cpdefi {
            Cpdefi::default()
        }
        fn jt_state(&mut self, _: u32) -> Option<i32> {
            None
        }
    }
    impl FollowWorld for W {
        fn segments(&mut self) -> Option<&mut dyn CmSegmentQuery> {
            if self.1 { Some(&mut self.0) } else { None }
        }
    }
    let cm = Cm::default();
    let mut f = CplcyFollow::new(Box::new(W(Wall, true)));
    // clear: want on the focus side of the wall
    let r = f.FUN_00223a70(&cm, [0.0; 3], [-100.0, 0.0, 0.0]);
    assert_eq!(r, [-100.0, 0.0, 0.0]);
    assert_ne!(f.mode, 4);
    // occluded: want behind the wall
    let r = f.FUN_00223a70(&cm, [0.0; 3], [100.0, 0.0, 0.0]);
    assert_eq!(f.mode, 4);
    let mut q = Wall;
    assert!(q.solid_intervals([0.0; 3], r).is_empty(), "placement {r:?} still occluded");
    assert!(r[1] >= 30.0 || r[0] < 40.0, "{r:?}");
}

/// FUN_001991a0 hang point: contact + 37 * dir - 25 * z (JMP +0xf0 = DAT_002bf740 (37, 0, 25)).
#[test]
fn p17_ledge_hang_point() {
    use sly2_recomp::engine::player::p4::ledge_hang_point;
    let p = ledge_hang_point([100.0, 200.0, 300.0], [0.0, 1.0, 0.0]);
    for (a, b) in p.iter().zip([100.0, 237.0, 275.0]) {
        assert!((a - b).abs() < 1e-3, "{p:?}");
    }
}

/// FUN_001d6848 applies sinf (FUN_00245000) to the blend: player and K15 copies agree.
#[test]
fn p17_launch_blend_sinf() {
    let (from, to, g) = ([0.0; 3], [500.0, 0.0, 100.0], [0.0, 0.0, -2000.0]);
    let a = sly2_recomp::engine::player::p14::FUN_001d6848(0.5, from, to, g);
    let b = sly2_recomp::engine::classes::shared::launch::FUN_001d6848(0.5, from, to, g).unwrap();
    for k in 0..3 {
        assert!((a[k] - b[k]).abs() < 1e-3, "{a:?} {b:?}");
    }
}
