//! WP-E8: PO contact motor (0x204ac8 / 0x204250 / 0x2044b0) + tangential contact rows (0x158b20) on the
//! single-body 0x1f9410 solve, constraint following (0x1fa810 / 0x1fac30 / 0x1f9b20), relative 0x11d4d0.
use sly2_recomp::engine::classes::plugin::EngineHost;
use sly2_recomp::engine::classes::ClassHost;
use sly2_recomp::engine::physics::driven::Attachment;
use sly2_recomp::engine::physics::driver::*;
use sly2_recomp::engine::physics::dynamics::FUN_0011d018;
use sly2_recomp::engine::physics::geom::dot;
use sly2_recomp::engine::physics::motor::*;
use sly2_recomp::engine::physics::CollisionWorld;
use sly2_recomp::engine::world::{LoadOptions, World};
use std::sync::Arc;

mod k20 {
    use sly2_recomp::engine::disc::descriptor::Descriptor;
    use sly2_recomp::engine::disc::iso::IsoImage;
    use sly2_recomp::engine::disc::mods::ModOverlay;
    use sly2_recomp::engine::disc::LevelLoader;
    use std::sync::Arc;
    /// every level of the disc (as tests/engine_k20.rs levels())
    pub fn levels() -> (LevelLoader, Vec<Descriptor>) {
        crate::common::levels() // public: ISO level table instead of the analysis DB
    }
}

const DT: f32 = 1.0 / 60.0;

fn finite(v: [f32; 3]) -> bool {
    v.iter().all(|x| x.is_finite())
}

#[test]
fn e8_relative_eps_11d4d0() {
    // relative to max(1, |a|)
    assert!(FUN_0011d4d0(1000.0, 1000.05, 1e-4));
    assert!(!FUN_0011d4d0(1000.0, 1000.2, 1e-4));
    assert!(FUN_0011d4d0(0.0, 5e-5, 1e-4));
    assert!(!FUN_0011d4d0(0.0, 2e-4, 1e-4));
    assert!(FUN_0011d4d0(-0.5, -0.50005, 1e-4));
}

#[test]
fn e8_motor_2044b0_unit() {
    let m = PoMotor { v190: [100.0, 0.0, 0.0], k180: 2.0, k210: -20.0, k248: 5.0, ..Default::default() };
    let mut x = Xp { n: [0.0, 0.0, 1.0], gap: 10.0, kind: 0xe, vel: [[3.0, 4.0, -100.0], [0.0; 3]], ..Default::default() };
    let mut a = 0.0;
    let mut b = 0.0;
    FUN_0011d018(10.0, -20.0, DT, DAT_002C3C98, Some(&mut a));
    FUN_0011d018(10.0, 0.0, DT, DAT_002C3CA8, Some(&mut b));
    let f3 = (-100.0f32).max(a);
    let f2 = if f3 < b { (DT * 9000.0).min(b - f3) } else { (DT * -4500.0).max(b - f3) };
    FUN_002044b0(&m, &mut x, 0, DT);
    assert!((x.vel[0][2] - (-100.0 - (f3 + f2))).abs() < 1e-3);
    assert!((x.vel[0][0] - (3.0 + 200.0)).abs() < 1e-3 && (x.vel[0][1] - 4.0).abs() < 1e-4);
    assert!((x.gap - (10.0 - 25.0)).abs() < 1e-5);
    // other kinds untouched
    let mut y = Xp { kind: 3, vel: [[1.0; 3], [0.0; 3]], ..Default::default() };
    FUN_002044b0(&m, &mut y, 0, DT);
    assert_eq!(y.vel[0], [1.0; 3]);
    // 0x204250 bounds
    let mm = PoMotor { mass: 2.0, ..Default::default() };
    let mut f = Xp { kind: 0xe, n: [0.0, 0.0, 1.0], ..Default::default() };
    FUN_00204250(&mm, &mut f);
    assert_eq!((f.f94, f.fa0, f.fa4), (1.5, -9000.0, 18000.0));
    let mut s = Xp { kind: 0xe, n: [0.6, 0.0, 0.6], ..Default::default() };
    FUN_00204250(&mm, &mut s);
    assert_eq!((s.f94, s.fa0, s.fa4), (0.0, 0.0, 18000.0));
}

#[test]
fn e8_tangential_rows_drive_body() {
    // body on flat ground at gap 0, falling at -29 (one frame of gravity); motor wants -R v190 k180
    let m = PoMotor { v190: [300.0, 0.0, 0.0], ..Default::default() };
    let mut v = [0.0, 0.0, -1750.0 * DT];
    let mut speeds = Vec::new();
    for _ in 0..60 {
        let mut x = Xp { n: [0.0, 0.0, 1.0], gap: 0.0, kind: 0xe, point: [[0.0; 3]; 2], ..Default::default() };
        FUN_00204250(&m, &mut x);
        FUN_001a7fc0(&m, &mut x, v, DT);
        let s = FUN_001f9410_po(&[x], m.mass, -1750.0, 1, DT);
        // the tangential row exists and is bounded by mu * max(|lambda_n|, 1750 m dt)
        let t = s.rows.iter().find(|r| r.tangential).copied();
        if let Some(t) = t {
            let ln = s.rows[t.link.unwrap()].lambda.abs();
            assert!((t.hi - 1.5 * ln.max(1750.0 * DT)).abs() < 1e-3);
            assert!(t.lambda.abs() <= t.hi + 1e-3);
        }
        for k in 0..3 {
            v[k] += s.dv[k];
        }
        speeds.push(v[0]);
        // next frame gravity
        v[2] -= 1750.0 * DT;
    }
    // no fall: the normal row holds the body on the ground (z speed back to ~0 after the solve)
    assert!((v[2] + 1750.0 * DT).abs() < 1.0, "vz {}", v[2]);
    // friction-limited acceleration (mu g = 2625 / s^2 -> 43.75 per frame) towards -300
    assert!((speeds[0] + 1.5 * 1750.0 * DT).abs() < 1.0, "first {}", speeds[0]);
    assert!((speeds[59] + 300.0).abs() < 1.0, "final {}", speeds[59]);
}

#[test]
fn e8_constraint_following() {
    // line constraint: gravity projected on the line, body stays on it, impulses projected
    let d = [std::f32::consts::FRAC_1_SQRT_2, 0.0, std::f32::consts::FRAC_1_SQRT_2];
    let mut b = AloPhys { ckind: 2, cnorm: d, ..Default::default() };
    for f in 0..120 {
        if f == 30 {
            b.imp_lin = [0.0, 500.0, 200.0];
        }
        FUN_001f3980(&mut b, DT);
        FUN_001f9e70(&mut b, DT);
        let c = [b.pos[1] * d[2] - b.pos[2] * d[1], b.pos[2] * d[0] - b.pos[0] * d[2], b.pos[0] * d[1] - b.pos[1] * d[0]];
        assert!(dot(c, c).sqrt() < 1e-2, "frame {f} off the line {:?}", b.pos);
    }
    assert!(b.pos[2] < -100.0, "slides down the line {:?}", b.pos);
    // plane constraint with up normal: gravity removed
    let mut p = AloPhys { ckind: 1, cnorm: [0.0, 0.0, 1.0], ..Default::default() };
    for _ in 0..60 {
        FUN_001f3980(&mut p, DT);
    }
    assert_eq!(p.pos, [0.0; 3]);
    // locked: nothing
    let mut l = AloPhys { ckind: 3, cnorm: [0.0, 0.0, 1.0], v: [0.0; 3], ..Default::default() };
    l.imp_lin = [10.0, 10.0, 10.0];
    FUN_001f3980(&mut l, DT);
    FUN_001f9e70(&mut l, DT);
    assert_eq!(l.pos, [0.0; 3]);
    // parent frame (xf+0xa0 rotated 90 deg about z): constraint n = local x = world y, accel in parent frame
    let r = FUN_0011b500([0.0, 0.0, 1.0], std::f32::consts::FRAC_PI_2);
    let mut q = AloPhys { ckind: 2, cnorm: [1.0, 0.0, 0.0], gravity: [0.0, -1750.0, 0.0], parent: Some(r), ..Default::default() };
    FUN_001f9b20(&mut q);
    let gw = FUN_001fa810_xf(2, [1.0, 0.0, 0.0], Some(&r), [0.0, -1750.0, 0.0]);
    assert!((gw[1] + 1750.0).abs() < 1e-2 && gw[0].abs() < 1e-2);
    assert!((q.a[0] + 1750.0).abs() < 1e-2 && q.a[1].abs() < 1e-2, "local accel {:?}", q.a);
    // zero gravity: 0x1f9b20 adds nothing; +0x138 tail places attachments
    let mut z = AloPhys { gravity: [0.0; 3], pos: [5.0, 0.0, 0.0], ..Default::default() };
    let mut placed = Vec::new();
    FUN_001f9b20_att(&mut z, &[Attachment { child: 7, rot: IDENT, pos: [1.0, 2.0, 3.0] }], |c, _, p| placed.push((c, p)));
    assert_eq!(z.a, [0.0; 3]);
    assert_eq!(placed, vec![(7, [6.0, 2.0, 3.0])]);
}

/// Start of a character on a level: the class object's position, raised by `lift`.
fn start_of(w: &Arc<World>, class: i32) -> Option<(usize, [f32; 3])> {
    let o = w.objects.iter().position(|o| o.class as i32 == class)?;
    let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
    Some((o, h.ee_sub_1a62e8(o as u32)))
}

/// Ground under `p` (probe from 200 above down 4000).
fn ground(cw: &CollisionWorld, p: [f32; 3]) -> Option<GroundHit> {
    FUN_0014c6d8(cw, [p[0], p[1], p[2] + 200.0], [p[0], p[1], p[2] - 4000.0])
}

/// Drives a PO body for `frames` from `start` with a turning goal; returns (frames with the motor contact,
/// horizontal path length, min gap seen while grounded, all finite, max speed).
fn drive(cw: &CollisionWorld, start: [f32; 3], frames: usize) -> (usize, f32, f32, bool, f32) {
    let mut b = AloPhys { pos: start, kind: 2, ..Default::default() };
    let mut m = PoMotor { k180: 1.0, ..Default::default() };
    let (mut on, mut path, mut min_gap, mut ok, mut vmax) = (0, 0.0f32, f32::MAX, true, 0.0f32);
    for f in 0..frames {
        // circle of radius ~60 (speed 200, 3.3 rad/s) around the start
        let ang = f as f32 * 0.055;
        m.v190 = [200.0 * ang.cos(), 200.0 * ang.sin(), 0.0];
        let p = b.pos;
        let st = po_step(cw, &mut b, &m, [0.0; 3], (50.0, 60.0), Some(([0.0, 0.0, 60.0], 50.0)), [0.0; 3], DT);
        ok &= finite(b.pos) && finite(b.v);
        vmax = vmax.max(dot(b.v, b.v).sqrt());
        if let Some(x) = st.motor {
            on += 1;
            min_gap = min_gap.min(dot([b.pos[0] - x.point[1][0], b.pos[1] - x.point[1][1], b.pos[2] - x.point[1][2]], x.n));
        }
        if std::env::var("E8_TRACE").is_ok() && f < 40 {
            eprintln!("f{f} pos {:?} v {:?} motor {:?} n_xps {}", b.pos, b.v, st.motor.map(|x| (x.gap, x.n)), st.xps.len());
        }
        path += ((b.pos[0] - p[0]).powi(2) + (b.pos[1] - p[1]).powi(2)).sqrt();
    }
    (on, path, min_gap, ok, vmax)
}

#[test]
fn e8_bentley_murray_drive_real_level() {
    let (loader, descs) = k20::levels();
    let mut done = 0;
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let cw = CollisionWorld::build(&w);
        for class in [55i32, 54, 53] {
            let Some((o, p)) = start_of(&w, class) else { continue };
            let Some(g) = ground(&cw, p) else { continue };
            if g.normal[2] < 0.7 {
                continue;
            }
            // start on the ground under the instance
            let start = [p[0], p[1], g.point[2]];
            let (on, path, min_gap, ok, vmax) = drive(&cw, start, 300);
            eprintln!("class {class} obj {o} start {start:?}: motor {on}/300, path {path:.0}, min gap {min_gap:.2}, vmax {vmax:.0}");
            assert!(ok, "NaN");
            assert!(on >= 270, "lost the ground: {on}/300");
            assert!(min_gap > -10.0, "sank into the floor: {min_gap}");
            assert!(path > 300.0, "did not drive: {path}");
            done += 1;
        }
        if done >= 2 {
            break;
        }
    }
    assert!(done >= 1, "no Bentley / Murray on walkable ground");
}

#[test]
#[ignore = "heavy"]
fn e8_all_levels_no_nan() {
    let (loader, descs) = k20::levels();
    let mut n = 0;
    for (i, d) in descs.into_iter().enumerate() {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let cw = CollisionWorld::build(&w);
        let Some((_, p)) = [53i32, 55, 54].iter().find_map(|&c| start_of(&w, c)) else { continue };
        let start = match ground(&cw, p) {
            Some(g) => [p[0], p[1], g.point[2]],
            None => p,
        };
        let (on, path, _, ok, vmax) = drive(&cw, start, 300);
        eprintln!("level {i}: motor {on}/300 path {path:.0} vmax {vmax:.0}");
        assert!(ok, "level {i}: NaN");
        assert!(vmax < 10000.0, "level {i}: explosion {vmax}");
        n += 1;
    }
    assert!(n >= 40, "levels run {n}");
}
