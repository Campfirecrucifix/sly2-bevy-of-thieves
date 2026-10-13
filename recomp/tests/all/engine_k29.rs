//! WP-K29 tests: class camera requests on the CM (sequence camera moves 0x38 / 0x39), particle limits.
use sly2_recomp::engine::camera::cm::{CmClock, NoWorld, FUN_0014d478};
use sly2_recomp::engine::camera::policy::CplcyCtx;
use sly2_recomp::engine::camera::Cm;
use sly2_recomp::engine::classes::k29_cam::*;

use crate::common as k20;

fn clk(t: f32) -> CmClock {
    CmClock { t, dt: 1.0 / 60.0, dt_real: 1.0 / 60.0 }
}

fn pos(cm: &Cm) -> [f32; 3] {
    [cm.mat[3][0], cm.mat[3][1], cm.mat[3][2]]
}

/// Item 1: FUN_001e57a0 -> blend policy 0x2de440 at cpp 10: an unset key A (zeros) takes the camera's
/// current pose / fov, the camera then follows the blend curve to key B, FUN_001e5a98 revokes it.
#[test]
fn k29_seq_move_blends_camera() {
    let mut cm = Cm::new();
    let start = pos(&cm);
    let fov0 = cm.rad_fov;
    let mut st = K29Cam::default();
    let b = MoveKey { pos: [1000.0, 200.0, 300.0], rot: [0.5, 0.0, 0.0], fov: 0.9 };
    let mv = CmMove { a: MoveKey::default(), b, duration: 2.0, cut: 1, curve: 0 };
    let pv = 77;
    st.apply(
        &mut cm,
        &[
            CamReq::Move { pv, mv },
            CamReq::Revoke { grf: 2, cpp: 10, desc: DESC_BLEND_2DE440, pso: 0, pv, if_active: true },
            CamReq::Set { cpp: CPP_SEQ_MOVE, desc: DESC_BLEND_2DE440, pso: 0, pv },
        ],
        clk(1.0),
    );
    assert_eq!(cm.acpr[0].cpp, 10);
    assert_eq!(Some(cm.acpr[0].pcplcy), st.blend);
    let m = st.moves.lock().unwrap()[&pv];
    assert_eq!(m.a.pos, start, "unset key A = the camera position");
    assert_eq!(m.a.fov, fov0);
    let ctx = CplcyCtx::default();
    FUN_0014d478(&mut cm, &mut NoWorld, &ctx, clk(2.0));
    let half = pos(&cm);
    for i in 0..3 {
        let want = 0.5 * (start[i] + b.pos[i]);
        assert!((half[i] - want).abs() < 1e-2, "half {half:?} want {want}");
    }
    FUN_0014d478(&mut cm, &mut NoWorld, &ctx, clk(3.5));
    let end = pos(&cm);
    for i in 0..3 {
        assert!((end[i] - b.pos[i]).abs() < 1e-2, "end {end:?}");
    }
    // FUN_001e5a98: +0x90 = rec[0x11], revoke by policy -> the base policy is back on top
    st.apply(&mut cm, &[CamReq::MoveCut { pv, cut: 0 }, CamReq::Revoke { grf: 2, cpp: 10, desc: DESC_BLEND_2DE440, pso: 0, pv, if_active: false }], clk(3.6));
    assert_eq!(cm.acpr.len(), 1);
    assert_eq!(cm.acpr[0].cpp, 0);
    assert_eq!(st.moves.lock().unwrap()[&pv].cut, 0);
}

/// Item 1: FUN_0014fd50 gate: the revoke before a new move only runs while the blend is the active policy;
/// requests on unported descriptors are counted, not applied.
#[test]
fn k29_revoke_if_active_and_unported() {
    let mut cm = Cm::new();
    let mut st = K29Cam::default();
    let mv = CmMove { b: MoveKey { pos: [1.0, 2.0, 3.0], ..Default::default() }, duration: 1.0, ..Default::default() };
    st.apply(&mut cm, &[CamReq::Move { pv: 1, mv }, CamReq::Set { cpp: 10, desc: DESC_BLEND_2DE440, pso: 0, pv: 1 }], clk(0.0));
    // a higher-priority unported policy request is skipped (stack unchanged)
    st.apply(&mut cm, &[CamReq::Set { cpp: 12, desc: 0x2de2f0, pso: 1, pv: 0 }], clk(0.1));
    assert_eq!(st.skipped, 1);
    assert_eq!(cm.acpr.len(), 2);
    st.apply(&mut cm, &[CamReq::Revoke { grf: 2, cpp: 10, desc: DESC_BLEND_2DE440, pso: 0, pv: 1, if_active: true }], clk(0.2));
    assert_eq!(cm.acpr.len(), 1, "active blend revoked");
    st.apply(&mut cm, &[CamReq::Revoke { grf: 2, cpp: 10, desc: DESC_BLEND_2DE440, pso: 0, pv: 1, if_active: true }], clk(0.3));
    assert_eq!(cm.acpr.len(), 1);
}

/// Item 1 census (--ignored): sequence 0x38 / 0x39 records (class 114) over every level.
#[test]
#[ignore]
fn k29_probe_seq_camera_records() {
    let (loader, descs) = k20::levels();
    let (mut n38, mut n39, mut lv_with) = (0, 0, 0);
    for d in descs {
        let Ok(lv) = crate::common::load_blocking(&loader, d) else { continue };
        let Ok(w) = sly2_recomp::engine::world::World::load(&lv, Default::default()) else { continue };
        let mut host = sly2_recomp::engine::classes::plugin::EngineHost { world: Some(std::sync::Arc::new(w)), ..Default::default() };
        let w = host.world.clone().unwrap();
        let set = sly2_recomp::engine::classes::ClassSet::build(&w, &mut host);
        let mut here = 0;
        for (_, x) in &set.more {
            if let sly2_recomp::engine::classes::Inst::C2d8c48(c) = x {
                for r in &c.f_54 {
                    match r[0] {
                        0x38 => {
                            n38 += 1;
                            here += 1;
                        }
                        0x39 => n39 += 1,
                        _ => {}
                    }
                }
            }
        }
        lv_with += (here > 0) as usize;
    }
    eprintln!("K29 census: 0x38 {n38} 0x39 {n39} levels with moves {lv_with}");
}

/// Item 1 (--ignored): what the class-run ASEGAs drive through anim::bind (bones / objects)
#[test]
#[ignore]
fn k29_probe_binder() {
    use sly2_recomp::engine::classes::plugin::EngineHost;
    use sly2_recomp::engine::classes::ClassSet;
    let (loader, descs) = k20::levels();
    for d in descs {
        let Ok(lv) = crate::common::load_blocking(&loader, d) else { continue };
        let Ok(w) = sly2_recomp::engine::world::World::load(&lv, Default::default()) else { continue };
        let w = std::sync::Arc::new(w);
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        s.post_load(&mut h);
        for f in 0..300 {
            s.g.t += 1.0 / 60.0;
            s.g.dt = 1.0 / 60.0;
            s.frame(&mut h);
            if f == 299 && !h.k25.asegas.is_empty() {
                let b = h.k29_binder.lock().unwrap();
                for (k, x) in &h.k25.asegas {
                    let bd = b.asegas.get(&(*k as u64));
                    eprintln!("aseg {k} obj {} aseg {} alive {} bones {:?} objs {:?}", x.obj, x.aseg, x.st.alive, bd.map(|b| b.bones.iter().take(8).collect::<Vec<_>>()), bd.map(|b| b.objs.iter().take(8).collect::<Vec<_>>()));
                }
                eprintln!("posed {:?}", h.k26_posed.keys().collect::<Vec<_>>());
            }
        }
    }
}

/// Item 1: k26_bind on the D6 AnimBinder: the class-run ASEGAs of the first levels bind bone references and
/// their bones move (object roots stay: no live class ASEG drives a bone 0).
#[test]
fn k29_class_asegas_drive_bones() {
    use sly2_recomp::engine::classes::plugin::EngineHost;
    use sly2_recomp::engine::classes::ClassSet;
    use std::collections::HashMap;
    let (loader, descs) = k20::levels();
    let (mut bound, mut moved) = (0, 0);
    for d in descs.into_iter().take(3) {
        let Ok(lv) = crate::common::load_blocking(&loader, d) else { continue };
        let Ok(w) = sly2_recomp::engine::world::World::load(&lv, Default::default()) else { continue };
        let w = std::sync::Arc::new(w);
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        s.post_load(&mut h);
        let mut first: HashMap<(usize, usize), [f32; 3]> = HashMap::new();
        let mut mv = std::collections::HashSet::new();
        for _ in 0..120 {
            s.g.t += 1.0 / 60.0;
            s.g.dt = 1.0 / 60.0;
            s.frame(&mut h);
            for (k, m) in &h.k29_bones {
                let p = *first.entry(*k).or_insert(m[3]);
                if (0..3).any(|a| (p[a] - m[3][a]).abs() > 1e-3) {
                    mv.insert(*k);
                }
            }
        }
        bound += first.len();
        moved += mv.len();
    }
    eprintln!("K29 binder: bones {bound} moved {moved}");
    if bound > 0 {
        assert!(moved > 0, "bones {bound} moved {moved}");
    }
}

/// Item 2: FUN_001390d8 pool 0x3dd490 + 0x4070 (0x28 systems): unlinked requests past the cap spawn nothing;
/// a removed system frees its slot.
#[test]
fn k29_particle_system_pool_cap() {
    use sly2_recomp::engine::classes::c2ceb78::ee_sub_159118;
    use sly2_recomp::engine::classes::k28_emit::*;
    use sly2_recomp::engine::classes::plugin::EngineHost;
    use sly2_recomp::engine::classes::{ClassSet, Inst};
    let mut t = ee_sub_159118();
    t[0x30 / 4] = 10f32.to_bits();
    let mut set = ClassSet::default();
    let mut host = EngineHost::default();
    let live = |s: &ClassSet| s.k21c.iter().filter(|x| matches!(&x.1, Inst::K21C28(c) if !c.removed)).count();
    for i in 0..50 {
        let r = EmitReq { obj: i, count: 3, t, blk: [0; 5], link: false, frame: None };
        let parts = ee_sub_15b7e8(3, &t, 0.0, None, &mut set.rng);
        set.ee_sub_15b428(&r, parts, &mut host);
    }
    assert_eq!(live(&set), SYS_CAP);
    if let Inst::K21C28(c) = &mut set.k21c[0].1 {
        c.removed = true;
    }
    let r = EmitReq { obj: 99, count: 3, t, blk: [0; 5], link: false, frame: None };
    let parts = ee_sub_15b7e8(3, &t, 0.0, None, &mut set.rng);
    set.ee_sub_15b428(&r, parts, &mut host);
    assert_eq!(live(&set), SYS_CAP);
    assert_eq!(set.k21c.len(), SYS_CAP + 1);
}
