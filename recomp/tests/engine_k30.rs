//! WP-K30 tests: class camera presets 0x2de3e8 (object camera), 0x2de530 (attach), 0x2de390 (look, idle
//! without playerdraw) and 0x2de460 (zoom look, sequence actions 0x3d-0x41) on the CM.
use sly2_recomp::engine::camera::cm::{CmClock, NoWorld, FUN_0014d478};
use sly2_recomp::engine::camera::policy::{CmObject, CplcyCtx};
use sly2_recomp::engine::camera::Cm;
use sly2_recomp::engine::classes::k29_cam::*;
use sly2_recomp::engine::classes::k30_cam::*;

fn clk(t: f32) -> CmClock {
    CmClock { t, dt: 1.0 / 60.0, dt_real: 1.0 / 60.0 }
}

fn pos(cm: &Cm) -> [f32; 3] {
    [cm.mat[3][0], cm.mat[3][1], cm.mat[3][2]]
}

fn near(a: [f32; 3], b: [f32; 3], eps: f32) -> bool {
    (0..3).all(|i| (a[i] - b[i]).abs() < eps)
}

/// FUN_0012e858: the object camera takes the object's position; focal length users set the fov
/// 2 atan(17.995901 / +0x128); deactivation with +0x120 cuts and restores the base policy.
#[test]
fn k30_object_camera_follows_object() {
    let mut cm = Cm::new();
    let mut st = K29Cam::default();
    let o = ObjCam {
        rows: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0]],
        pos: [100.0, 200.0, 300.0],
        target: None,
        f_120: 1,
        f_128: 35.0,
        f_12c: 1,
        f_130: 0.0,
    };
    st.objs.lock().unwrap().insert(9, CamState::Obj(o));
    st.apply(&mut cm, &[CamReq::Set { cpp: 10, desc: DESC_OBJ_2DE3E8, pso: 0, pv: 9 }], clk(1.0));
    assert_eq!(Some(cm.acpr[0].pcplcy), st.obj_cam);
    let ctx = CplcyCtx::default();
    FUN_0014d478(&mut cm, &mut NoWorld, &ctx, clk(1.1));
    assert!(near(pos(&cm), o.pos, 1e-3), "{:?}", pos(&cm));
    let fov = (HALF_FILM / 35.0).atan() * 2.0;
    assert!((cm.rad_fov_target - fov).abs() < 1e-4, "{} {}", cm.rad_fov_target, fov);
    st.apply(&mut cm, &[CamReq::Revoke { grf: 0xf, cpp: 10, desc: DESC_OBJ_2DE3E8, pso: 0, pv: 9, if_active: false }], clk(1.2));
    assert_eq!(cm.acpr.len(), 1);
    assert_eq!(st.skipped, 0);
}

/// 0x2de390 without playerdraw's look policy: requests are no-ops (not counted as unported).
#[test]
fn k30_look_idle_without_playerdraw() {
    let mut cm = Cm::new();
    let mut st = K29Cam::default();
    st.apply(&mut cm, &[CamReq::Set { cpp: 7, desc: DESC_LOOK_2DE390, pso: 0, pv: 0 }, CamReq::Revoke { grf: 2, cpp: -1, desc: DESC_LOOK_2DE390, pso: 0, pv: 0, if_active: false }], clk(1.0));
    assert_eq!(cm.acpr.len(), 1);
    assert_eq!(st.skipped, 0);
}

/// 0x2de530: the attach policy runs on the published object of its pso.
#[test]
fn k30_attach_uses_published_object() {
    let mut cm = Cm::new();
    let mut st = K29Cam::default();
    let mut mat = sly2_recomp::engine::camera::math::IDENTITY;
    mat[3] = [500.0, -40.0, 60.0, 1.0];
    st.objs.lock().unwrap().insert(4, CamState::Attach(CmObject { mat, fov: 0.7, smooth: false }));
    st.apply(&mut cm, &[CamReq::Set { cpp: 9, desc: DESC_ATTACH_2DE530, pso: 4, pv: 0 }], clk(1.0));
    assert_eq!(Some(cm.acpr[0].pcplcy), st.attach);
    let ctx = CplcyCtx::default();
    for i in 0..4 {
        FUN_0014d478(&mut cm, &mut NoWorld, &ctx, clk(1.0 + i as f32 / 60.0));
    }
    assert!(near(pos(&cm), [500.0, -40.0, 60.0], 1e-2), "{:?}", pos(&cm));
    assert_eq!(st.skipped, 0);
}

/// 0x2de460: free mode looks at the centre object from 50 back, zooms to fov / 7 on action 0x40; the key
/// (action 0x3f) switches to object mode and springs to it; the getters (waits 0x3d / 0x3e / 0x40) settle.
#[test]
fn k30_zoom_look_free_and_key() {
    let mut cm = Cm::new();
    let mut st = K29Cam::default();
    let c = [1000.0, 0.0, 100.0];
    st.zoom.lock().unwrap().pos.objs.insert(5, c);
    let fov0 = cm.rad_fov_target;
    st.apply(&mut cm, &[CamReq::Zoom(ZoomOp::Center(5)), CamReq::Zoom(ZoomOp::Zoom(1.0)), CamReq::Set { cpp: 8, desc: DESC_ZOOM_2DE460, pso: 0, pv: 0 }], clk(1.0));
    assert_eq!(Some(cm.acpr[0].pcplcy), st.zoom_cam);
    assert_eq!(st.skipped, 0);
    let ctx = CplcyCtx::default();
    let mut t = 1.0;
    for _ in 0..240 {
        t += 1.0 / 60.0;
        FUN_0014d478(&mut cm, &mut NoWorld, &ctx, clk(t));
    }
    st.apply(&mut cm, &[], clk(t));
    assert_eq!(st.zoom_waits, [true, true, true]);
    {
        let z = st.zoom.lock().unwrap();
        assert_eq!(z.f_ac, 1.0);
        assert!((z.f_b8 - fov0 / 7.0).abs() < 1e-5);
        assert!((sly2_recomp::engine::camera::math::len(sly2_recomp::engine::camera::math::sub(z.f_30, c)) - 50.0).abs() < 1e-2);
    }
    assert!((cm.rad_fov_target - fov0 / 7.0).abs() < 1e-4, "{} {}", cm.rad_fov_target, fov0);
    // object mode
    let key = [200.0, 300.0, 400.0];
    st.apply(&mut cm, &[CamReq::Zoom(ZoomOp::Key { pos: key, rot: [0.0, 0.0, 0.5], fov: 0.6 })], clk(t));
    assert_eq!(st.zoom.lock().unwrap().f_40, 1);
    for _ in 0..600 {
        t += 1.0 / 60.0;
        FUN_0014d478(&mut cm, &mut NoWorld, &ctx, clk(t));
    }
    assert!(near(pos(&cm), key, 1e-2), "{:?}", pos(&cm));
    assert!((cm.rad_fov_target - 0.6).abs() < 1e-3);
    // deactivation (FUN_00151130) clears the objects and the zoom target
    st.apply(&mut cm, &[CamReq::Revoke { grf: 0xf, cpp: 8, desc: DESC_ZOOM_2DE460, pso: 0, pv: 0, if_active: false }], clk(t));
    let z = st.zoom.lock().unwrap();
    assert_eq!((z.f_10, z.f_40, z.f_18), (0, 0, -1.0));
}

/// K30 G2: FUN_0015a808 shape kind 1 (emitter box), shape kind 2 (class-10 path, tangent direction),
/// direction kind 4 (camera rows into the source frame) and the FUN_0015ad48 source velocity add.
#[test]
fn k30_emit_shapes_and_source_velocity() {
    use sly2_recomp::engine::classes::c2ceb78::ee_sub_159118;
    use sly2_recomp::engine::classes::k28_emit::*;
    use sly2_recomp::engine::physics::driven::GameRng;
    use sly2_recomp::engine::player::p5_target::Path;
    let mut rng = GameRng::default();
    // shape kind 1: inside the box, no advance (+0x24 / +0x28 = 0)
    let mut t = ee_sub_159118();
    t[4] = 1;
    t[0x24 / 4] = 0;
    t[0x28 / 4] = 0;
    let bx = [-10.0, 0.0, 5.0, 0.0, 10.0, 20.0, 6.0, 0.0];
    let mut src = ShapeSrc { bx, ..Default::default() };
    for (p, ..) in ee_sub_15b7e8_src(40, &t, 0.0, None, &mut src, &mut rng) {
        assert!((-10.0..=10.0).contains(&p[0]) && (0.0..=20.0).contains(&p[1]) && (5.0..=6.0).contains(&p[2]), "{p:?}");
    }
    // shape kind 2 on a 100-long straight path along x, direction kind 2 (tangent): particle i in [10i, 10i + 10]
    t[4] = 2;
    t[5] = 2;
    let path = Path {
        spline: false,
        looped: false,
        s: vec![0.0, 1.0],
        l: vec![0.0, 100.0],
        pts: vec![[0.0, 0.0, 0.0], [100.0, 0.0, 0.0]],
        tan_in: vec![],
        tan_out: vec![],
        xf: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [0.0; 3]],
    };
    let mut src = ShapeSrc { path: Some(std::sync::Arc::new(path)), ..Default::default() };
    let mut dir = [0.0; 3];
    for i in 0..10 {
        let p = ee_sub_15a808_src(&t, i, 10, &src, None, &mut dir, &mut rng);
        assert!(p[0] >= 10.0 * i as f32 - 1e-3 && p[0] <= 10.0 * i as f32 + 10.0 + 1e-3 && p[1] == 0.0, "{i} {p:?}");
        assert_eq!(dir, [1.0, 0.0, 0.0]);
    }
    // no path object: position 0, the direction buffer untouched
    src.path = None;
    let mut dir = [0.0, 0.0, 1.0];
    assert_eq!(ee_sub_15a808_src(&t, 0, 1, &src, None, &mut dir, &mut rng), [0.0; 3]);
    assert_eq!(dir, [0.0, 0.0, 1.0]);
    // direction kind 4: +0x40 = (1, 0, 0) through camera row 0 = (0, 0, 1) -> world +z; source frame rotated
    // 90 degrees about z leaves z unchanged
    t[4] = 0;
    t[5] = 4;
    t[0x40 / 4] = 1f32.to_bits();
    t[0x44 / 4] = 0;
    t[0x48 / 4] = 0;
    let src = ShapeSrc { cam: Some([[0.0, 0.0, 1.0, 0.0], [1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0]]), ..Default::default() };
    let frame = [[0.0, 1.0, 0.0], [-1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0; 3]];
    ee_sub_15a808_src(&t, 0, 1, &src, Some(&frame), &mut dir, &mut rng);
    assert!((dir[2] - 1.0).abs() < 1e-6, "{dir:?}");
    // source velocity: +0x18 = 0 adds FUN_0022d348 at the source origin
    let mut t = ee_sub_159118();
    t[6] = 0;
    let frame = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [7.0, 8.0, 9.0]];
    let mut at = Vec::new();
    let mut f = |p: [f32; 3]| {
        at.push(p);
        [0.0, 0.0, 1000.0]
    };
    let mut src = ShapeSrc { vel: Some(&mut f), ..Default::default() };
    let ps = ee_sub_15b7e8_src(4, &t, 0.0, Some(&frame), &mut src, &mut rng);
    drop(src);
    assert_eq!(at, vec![[7.0, 8.0, 9.0]; 4]);
    for (_, v, ..) in &ps {
        let s = (v[0] * v[0] + v[1] * v[1] + (v[2] - 1000.0) * (v[2] - 1000.0)).sqrt();
        assert!((s - 750.0).abs() < 1.0, "{v:?}");
    }
}

/// K30 G3: anim event 0x11 request (FUN_00210538) queued on the host reaches a class-33 emitter's
/// FUN_0015c690 (vtable +0x19c) and spawns a class-28 system in `k28_spawn`.
#[test]
fn k30_anim_event_11_emits() {
    use sly2_recomp::engine::classes::c2ce4d0::C2ce4d0;
    use sly2_recomp::engine::classes::c2ceb78::ee_sub_159118;
    use sly2_recomp::engine::classes::plugin::EngineHost;
    use sly2_recomp::engine::classes::{ClassSet, Inst};
    let mut set = ClassSet::default();
    let mut c = C2ce4d0::default();
    c.tpl.w = ee_sub_159118();
    set.insts.push((5, Inst::C2ce4d0(c)));
    let mut host = EngineHost::default();
    // flags 1 (source) | 0x80 (count r[14])
    let mut r = [0u32; 15];
    r[0] = 0x81;
    r[1] = 6;
    r[14] = 4;
    host.k30_reqs.push((6, r));
    set.k28_spawn(&mut host);
    assert!(host.k30_reqs.is_empty());
    let n: usize = set.k21c.iter().map(|x| match &x.1 {
        Inst::K21C28(c) => c.groups.iter().map(|g| g.parts.len()).sum(),
        _ => 0,
    }).sum();
    assert_eq!(n, 4);
}
