use sly2_recomp::engine::camera::cm::*;
use sly2_recomp::engine::camera::hud::*;
use sly2_recomp::engine::camera::policy::*;
use sly2_recomp::engine::camera::widescreen::*;
use sly2_recomp::engine::camera::CmClock;

fn clk() -> CmClock {
    CmClock { t: 1.0, dt: 1.0 / 60.0, dt_real: 1.0 / 60.0 }
}

fn approx(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-4 * (1.0 + a.abs().max(b.abs()))
}

#[test]
fn init_defaults_match_decomp() {
    let cm = Cm::new();
    assert_eq!(cm.s_near_clip, 100.0);
    assert_eq!(cm.s_far_clip, 100000.0);
    assert_eq!(cm.rad_fov, 1.0);
    assert_eq!(cm.rgba_fog, 0x80303030);
    assert!(approx(cm.y_scale, (0.5f32).tan()));
    assert!(approx(cm.x_scale, (0.5f32).tan() * R_ASPECT_PS2));
    assert_eq!(cm.pos(), [0.0, -2000.0, 500.0]);
    assert_eq!(cm.acpr.len(), 1);
    // looking at the origin: the origin projects to the screen centre
    let s = FUN_0014f308(&cm, [0.0, 0.0, 0.0]);
    assert!(s[0].abs() < 1e-4 && s[1].abs() < 1e-4 && s[2] > 0.0);
    let back = FUN_0014f268(&cm, [0.3, -0.2, 1000.0]);
    let s2 = FUN_0014f308(&cm, back);
    assert!(approx(s2[0], 0.3) && approx(s2[1], -0.2) && approx(s2[2], 1000.0));
}

#[test]
fn fov_math_hor_plus() {
    let mut cm = Cm::new();
    let mut v = vec![];
    let mut h = vec![];
    for (w, hh) in [(1440.0, 1080.0), (1920.0, 1080.0), (2560.0, 1080.0), (3840.0, 1080.0)] {
        apply_hor_plus(&mut cm, w, hh, clk());
        v.push(vertical_fov(&cm));
        h.push(horizontal_fov(&cm));
    }
    for x in &v {
        assert!(approx(*x, v[0]));
    }
    assert!(h.windows(2).all(|p| p[1] > p[0]));
    // 4:3 Hor+ reproduces the original horizontal FOV exactly
    let mut o = Cm::new();
    apply_original(&mut o, clk());
    assert!(approx(h[0], horizontal_fov(&o)));
}

#[test]
fn culling_32_9_keeps_edge_objects() {
    let mut cm = Cm::new();
    apply_hor_plus(&mut cm, 3840.0, 1080.0, clk());
    // a point near the right screen edge at depth 1000
    let p = FUN_0014f268(&cm, [0.95, 0.0, 1000.0]);
    let rel = [p[0] - cm.mat[3][0], p[1] - cm.mat[3][1], p[2] - cm.mat[3][2]];
    assert!(FUN_0014f850(&cm, rel, 1.0));
    let mut o = Cm::new();
    apply_original(&mut o, clk());
    let rel_o = [p[0] - o.mat[3][0], p[1] - o.mat[3][1], p[2] - o.mat[3][2]];
    assert!(!FUN_0014f850(&o, rel_o, 1.0)); // the original frustum would cull it
    // behind the camera is culled
    assert!(!FUN_0014f850(&cm, [0.0, -2000.0, 0.0], 1.0));
}

#[test]
fn hud_rect_mapping() {
    let l = HudLayout::new(3840.0, 1080.0);
    assert_eq!(l.safe, Rect { x: 960.0, y: 0.0, w: 1920.0, h: 1080.0 });
    assert_eq!(l.map(0.0, 0.0, Anchor::Left), (960.0, 0.0));
    assert_eq!(l.map(640.0, 448.0, Anchor::Right), (2880.0, 1080.0));
    assert_eq!(l.map(320.0, 224.0, Anchor::Center), (1920.0, 540.0));
    let t = HudLayout::new(1440.0, 1080.0);
    assert_eq!(t.safe.w, 1440.0);
    assert!(approx(t.safe.h, 810.0) && approx(t.safe.y, 135.0));
    assert_eq!(HudLayout::auto_anchor(10.0), Anchor::Left);
}

#[test]
fn policy_stack_and_blend() {
    let mut cm = Cm::new();
    let c = clk();
    let k = |x: f32| CmKey { pos: [x, 0.0, 0.0], rot: [0.0; 3], fov: 1.0 };
    let b = add_policy(&mut cm, Box::new(CplcyBlend::new(CmBlend { a: k(0.0), b: k(100.0), duration: 2.0, curve: 0, cut_on_end: false })));
    assert!(FUN_0014e328(&mut cm, 5, b, 0, 0, c));
    assert_eq!(cm.acpr[0].pcplcy, b);
    let mut c2 = c;
    c2.t += 1.0;
    FUN_0014d478(&mut cm, &mut NoWorld, &CplcyCtx::default(), c2);
    assert!(approx(cm.pos()[0], 50.0));
    cm.s_near_clip = 7.0;
    FUN_0014e4c8(&mut cm, 2, 0, b, 0, 0, c2);
    assert_eq!(cm.acpr.len(), 1);
    assert_eq!(cm.s_near_clip, 100.0); // restored by base deactivate 0x0014fce8
}

#[test]
fn spring_and_mrd() {
    let (mut p, mut v) = ([0.0; 3], [0.0; 3]);
    FUN_0014e6d0(0, 5.0, -3.5, &mut p, &mut v, [10.0, 0.0, 0.0], 0.1);
    assert!(approx(v[0], 5.0) && approx(p[0], 0.5));
    FUN_0014e6d0(1, 5.0, -3.5, &mut p, &mut v, [10.0, 0.0, 0.0], 0.1);
    assert_eq!((p, v), ([10.0, 0.0, 0.0], [0.0; 3]));
    let cm = Cm::new();
    assert_eq!(FUN_0014f898(&cm, 1000.0, [100.0, 0.0, 0.0]), Some(1.0));
    assert_eq!(FUN_0014f898(&cm, 1000.0, [1001.0, 0.0, 0.0]), None);
    let f = FUN_0014f898(&cm, 1000.0, [800.0, 0.0, 0.0]).unwrap();
    assert!(f > 0.0 && f < 1.0);
}
