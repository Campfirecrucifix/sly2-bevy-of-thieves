//! K29: camera requests of the class code (FUN_0014e328 SetCmPolicy / FUN_0014e4c8 RevokeCmPolicy on g_cm
//! 0x2ddf40) applied to the ported CM (camera/cm.rs), and the sequence camera move (actions 0x38 / 0x39,
//! FUN_001e57a0 / FUN_001e5a98) on the blend policy CM +0x500 (0x2de440, C2cd408: camera/policy.rs CplcyBlend).
//! The class host queues the requests (`EngineHost::k29_cam`); `apply` runs them in order on the CM.
//! UNPORTED descriptors (requests counted in `K29Cam::skipped`): 0x2de2f0 follow (playerdraw / I5 TrackSet own
//! it). K30: 0x2de3e8 / 0x2de530 / 0x2de390 / 0x2de460 in classes/k30_cam.rs.
#![allow(non_snake_case)]

use crate::engine::camera::cm::{add_policy, Cm, CmClock, FUN_0014e328, FUN_0014e4c8};
use crate::engine::camera::math::FUN_0011bff0;
use crate::engine::physics::driver::FUN_0011d4d0;
use crate::engine::camera::policy::{CmBlend, CmKey, Cplcy, CplcyBlend, CplcyCtx};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// CM +0x500: the blend policy (C2cd408) the sequence camera moves push (cpp 10).
pub const DESC_BLEND_2DE440: u32 = 0x2de440;
/// Priority of the sequence camera move (FUN_0014e328(g_cm, 10, 0x2de440, 0, node + 0x84)).
pub const CPP_SEQ_MOVE: i32 = 10;
/// FUN_0011d4d0 tolerance of the "unset" key test in FUN_001e57a0 (0x38d1b717).
pub const KEY_EPS: f32 = 1.0e-4;

/// One camera key of a 0x38 / 0x39 record (record +0x10 pos, +0x20 Euler rotation, +0x30 fov).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MoveKey {
    pub pos: [f32; 3],
    pub rot: [f32; 3],
    pub fov: f32,
}

impl MoveKey {
    /// record words 4..=0xc
    pub fn from_rec(r: &[u32]) -> Self {
        let f = |i: usize| f32::from_bits(r[i]);
        Self { pos: [f(4), f(5), f(6)], rot: [f(8), f(9), f(0xa)], fov: f(0xc) }
    }
}

/// The node's blend descriptor (node +0x84: key A*, key B*, +0x8c duration, +0x90 cut on end, +0x94 curve).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CmMove {
    pub a: MoveKey,
    pub b: MoveKey,
    pub duration: f32,
    pub cut: u32,
    pub curve: u32,
}

/// A queued camera request of the class code.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CamReq {
    /// FUN_0014e328(g_cm, cpp, desc, pso, pv)
    Set { cpp: i32, desc: u32, pso: u32, pv: u32 },
    /// FUN_0014e4c8(g_cm, grf, cpp, desc, pso, pv); `if_active`: only when FUN_0014fd50(desc) (desc is the
    /// active policy, DAT_002de254)
    Revoke { grf: u32, cpp: i32, desc: u32, pso: i32, pv: u32, if_active: bool },
    /// FUN_001e57a0: the node's descriptor (keys still unresolved)
    Move { pv: u32, mv: CmMove },
    /// FUN_001e5a98: node +0x90 = cut
    MoveCut { pv: u32, cut: u32 },
    /// K30: sequence actions 0x3d-0x41 on the zoom look 0x2de460 (classes/k30_cam.rs)
    Zoom(super::k30_cam::ZoomOp),
}

/// FUN_001e57a0 key fix-ups (per key): pos == 0 -> camera pos (DAT_002ddfd0, CM +0x90), rotation == 0 ->
/// FUN_0011bff0(CM +0x60), fov == 0 -> CM +0x20.
pub fn ee_sub_1e57a0_key(cm: &Cm, k: &mut MoveKey) {
    let unset = |v: [f32; 3]| v.iter().all(|&x| FUN_0011d4d0(0.0, x, KEY_EPS));
    if unset(k.pos) {
        k.pos = [cm.mat[3][0], cm.mat[3][1], cm.mat[3][2]];
    }
    if unset(k.rot) {
        k.rot = FUN_0011bff0(&cm.mat);
    }
    if k.fov == 0.0 {
        k.fov = cm.rad_fov;
    }
}

/// The blend policy instance of the class code (CM +0x500): CplcyBlend driven by the descriptor of its pv.
pub struct ClassBlend {
    pub moves: Arc<Mutex<HashMap<u32, CmMove>>>,
    pub inner: CplcyBlend,
}

impl ClassBlend {
    fn sync(&mut self, pv: u32) {
        let m = self.moves.lock().unwrap().get(&pv).copied();
        self.inner.blend = m.map(|m| CmBlend {
            a: CmKey { pos: m.a.pos, rot: m.a.rot, fov: m.a.fov },
            b: CmKey { pos: m.b.pos, rot: m.b.rot, fov: m.b.fov },
            duration: m.duration,
            curve: m.curve as usize,
            cut_on_end: m.cut != 0,
        });
    }
}

impl Cplcy for ClassBlend {
    fn activate(&mut self, cm: &mut Cm, pso: u32, pv: u32, clk: CmClock) {
        self.sync(pv);
        self.inner.activate(cm, pso, pv, clk);
    }
    fn deactivate(&mut self, cm: &mut Cm, pso: u32, pv: u32, clk: CmClock) {
        // +0x90 may have been rewritten by FUN_001e5a98 since the activation
        self.sync(pv);
        self.inner.deactivate(cm, pso, pv, clk);
    }
    fn update(&mut self, cm: &mut Cm, h: bool, pad: &CplcyCtx, clk: CmClock) {
        self.inner.update(cm, h, pad, clk);
    }
    fn set_saved_near(&mut self, s: f32) {
        self.inner.set_saved_near(s);
    }
}

/// Bridge state: the class policies registered in the CM.
#[derive(Default)]
pub struct K29Cam {
    pub blend: Option<usize>,
    pub moves: Arc<Mutex<HashMap<u32, CmMove>>>,
    /// requests on unported descriptors
    pub skipped: usize,
    /// K30: C2cab90 0x2de3e8 / C2cd3c0 0x2de530 instances, and the class objects' published camera state
    pub obj_cam: Option<usize>,
    pub attach: Option<usize>,
    pub objs: super::k30_cam::CamObjs,
    /// K30: playerdraw's 0x2de390 look policy (SlyFollow::look), set before `apply`
    pub look: Option<usize>,
    /// K30: the 0x2de460 zoom look (C2cd498) instance, its state, and its getters after the last `apply`
    /// (wait conditions 0x3d / 0x3e / 0x40)
    pub zoom_cam: Option<usize>,
    pub zoom: super::k30_cam::ZoomRef,
    pub zoom_waits: [bool; 3],
}

impl K29Cam {
    fn policy(&mut self, cm: &mut Cm, desc: u32) -> Option<usize> {
        use super::k30_cam::*;
        let objs = self.objs.clone();
        match desc {
            DESC_BLEND_2DE440 => {
                let moves = self.moves.clone();
                Some(*self.blend.get_or_insert_with(|| add_policy(cm, Box::new(ClassBlend { moves, inner: CplcyBlend::default() }))))
            }
            DESC_OBJ_2DE3E8 => Some(*self.obj_cam.get_or_insert_with(|| add_policy(cm, Box::new(CplcyObj { objs, save: Default::default(), f_10: 0 })))),
            DESC_ATTACH_2DE530 => Some(*self.attach.get_or_insert_with(|| add_policy(cm, Box::new(ClassAttach::new(objs))))),
            DESC_LOOK_2DE390 => self.look,
            DESC_ZOOM_2DE460 => {
                let z = self.zoom.clone();
                Some(*self.zoom_cam.get_or_insert_with(|| add_policy(cm, Box::new(CplcyZoom { z }))))
            }
            _ => None,
        }
    }

    /// K30: requests on 0x2de390 while playerdraw has no look policy are no-ops (nothing can be pushed on it)
    fn idle(&self, desc: u32) -> bool {
        desc == super::k30_cam::DESC_LOOK_2DE390 && self.look.is_none()
    }

    /// Run the queued requests in order.
    pub fn apply(&mut self, cm: &mut Cm, reqs: &[CamReq], clk: CmClock) {
        for r in reqs {
            match *r {
                CamReq::Set { cpp, desc, pso, pv } => match self.policy(cm, desc) {
                    Some(id) => {
                        FUN_0014e328(cm, cpp, id, pso, pv, clk);
                    }
                    None if self.idle(desc) => {}
                    None => self.skipped += 1,
                },
                CamReq::Revoke { grf, cpp, desc, pso, pv, if_active } => match self.policy(cm, desc) {
                    // FUN_0014fd50: DAT_002de254 (acpr[0].pcplcy) == desc
                    Some(id) if if_active && cm.acpr.first().map(|c| c.pcplcy) != Some(id) => {}
                    Some(id) => {
                        FUN_0014e4c8(cm, grf, cpp, id, pso as u32, pv, clk);
                    }
                    None if self.idle(desc) => {}
                    None => self.skipped += 1,
                },
                CamReq::Move { pv, mut mv } => {
                    ee_sub_1e57a0_key(cm, &mut mv.a);
                    ee_sub_1e57a0_key(cm, &mut mv.b);
                    self.moves.lock().unwrap().insert(pv, mv);
                }
                CamReq::MoveCut { pv, cut } => {
                    if let Some(m) = self.moves.lock().unwrap().get_mut(&pv) {
                        m.cut = cut;
                    }
                }
                CamReq::Zoom(op) => self.zoom.lock().unwrap().set(cm, op),
            }
        }
        self.zoom_waits = self.zoom.lock().unwrap().waits(cm);
    }
}
