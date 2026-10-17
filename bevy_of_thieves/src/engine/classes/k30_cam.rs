//! K30: camera presets of the class code on g_cm 0x2ddf40 (the policies embedded in CM, ctor FUN_0014d3d8),
//! applied through classes/k29_cam.rs `K29Cam` with the camera/ pub API:
//! - 0x2de3e8 (CM +0x4a8, FUN_0012e760, vtable C2cab90): the class-44 object camera (C2ca9f0, cpp 10).
//!   Slots 1 FUN_0012e798, 2 FUN_0012e7f0, 5 FUN_0012e858 1:1 (`CplcyObj`).
//! - 0x2de530 (CM +0x5f0, FUN_001518c8, C2cd3c0): the attach policy (camera/policy.rs `CplcyAttach`) on the
//!   class-69 scope (C2dbd80, cpp 9): object = pso +0x1d0 (vtable +0xfc matrix), fov pso +0x1bc, smooth +0x1d4.
//! - 0x2de390 (CM +0x450, C2cd4e0 look): owned by playerdraw (SlyFollow::look); class revokes are run on it.
//! The policies read the objects' state the class code publishes each frame (`ClassHost::k30_cam_state`),
//! instead of dereferencing the object pointer (pv / pso).
#![allow(non_snake_case)]

use crate::engine::camera::cm::{Cm, CmClock, FUN_0014d8c0, FUN_0014e168, FUN_0014e1a8, FUN_0014f5a0};
use crate::engine::camera::math::M4;
use crate::engine::camera::policy::{CmObject, Cplcy, CplcyAttach, CplcyCtx, CplcySave, FUN_0014fcc8, FUN_0014fce8};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// CM +0x4a8: C2cab90 object camera (class 44).
pub const DESC_OBJ_2DE3E8: u32 = 0x2de3e8;
/// CM +0x5f0: C2cd3c0 attach (class 69).
pub const DESC_ATTACH_2DE530: u32 = 0x2de530;
/// CM +0x450: C2cd4e0 look (binocucom, playerdraw).
pub const DESC_LOOK_2DE390: u32 = 0x2de390;
/// 0x12e858: half the 36 mm film width (fov = 2 atan(17.995901 / focal length +0x128)).
pub const HALF_FILM: f32 = 17.995901;

/// State of a class-44 object (C2ca9f0) read by the C2cab90 policy.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ObjCam {
    /// FUN_001a6318: world orientation rows
    pub rows: [[f32; 4]; 3],
    /// FUN_001a62e8: world position
    pub pos: [f32; 3],
    /// +0xe0 look-at object position (FUN_001a62e8(+0xe0)), None when +0xe0 == 0
    pub target: Option<[f32; 3]>,
    /// +0x120: cut when the policy deactivates
    pub f_120: i32,
    /// +0x128: focal length (written through slot 42)
    pub f_128: f32,
    /// +0x12c: focal length users (slots 43 / 44)
    pub f_12c: i32,
    /// +0x130: fov on activation (0 = keep)
    pub f_130: f32,
}

/// Published camera state of a class object (key = the pv / pso handle the requests use).
#[derive(Clone, Copy, Debug)]
pub enum CamState {
    Obj(ObjCam),
    Attach(CmObject),
}

pub type CamObjs = Arc<Mutex<HashMap<u32, CamState>>>;

fn obj_of(objs: &CamObjs, k: u32) -> Option<ObjCam> {
    match objs.lock().unwrap().get(&k) {
        Some(CamState::Obj(o)) => Some(*o),
        _ => None,
    }
}

/// 0x0012e858 body: camera matrix of the object (no target) = rows (-r2, -r0, r1, pos), else look-at.
pub fn ee_sub_12e858(cm: &mut Cm, o: &ObjCam, clk: CmClock) {
    match o.target {
        None => {
            let r = &o.rows;
            let m: M4 = [
                [-r[2][0], -r[2][1], -r[2][2], 0.0],
                [-r[0][0], -r[0][1], -r[0][2], 0.0],
                [r[1][0], r[1][1], r[1][2], 0.0],
                // FUN_00256d18(pos), w = 1.0
                [o.pos[0], o.pos[1], o.pos[2], 1.0],
            ];
            FUN_0014e168(cm, &m, clk);
        }
        Some(t) => FUN_0014e1a8(cm, 0.0, 0.0, o.pos, t, clk),
    }
    if 0 < o.f_12c {
        let fov = ((HALF_FILM / o.f_128).atan() * 2.0).clamp(0.01, 3.0);
        FUN_0014f5a0(cm, fov, clk);
    }
}

/// C2cab90 (FUN_0012e760) at CM +0x4a8: +0x10 = the current object.
pub struct CplcyObj {
    pub objs: CamObjs,
    pub save: CplcySave,
    pub f_10: u32,
}

impl Cplcy for CplcyObj {
    /// 0x0012e798
    fn activate(&mut self, cm: &mut Cm, _pso: u32, pv: u32, clk: CmClock) {
        self.save = FUN_0014fcc8(cm);
        self.f_10 = pv;
        if let Some(o) = obj_of(&self.objs, pv).filter(|o| o.f_130 != 0.0) {
            FUN_0014f5a0(cm, o.f_130, clk);
        }
    }
    /// 0x0012e7f0
    fn deactivate(&mut self, cm: &mut Cm, _pso: u32, pv: u32, clk: CmClock) {
        if self.f_10 == pv {
            if obj_of(&self.objs, pv).is_some_and(|o| o.f_120 != 0) {
                FUN_0014d8c0(cm);
            }
            self.f_10 = 0;
        }
        FUN_0014fce8(cm, &self.save, clk);
    }
    /// 0x0012e858
    fn update(&mut self, cm: &mut Cm, _h: bool, _pad: &CplcyCtx, clk: CmClock) {
        // the original dereferences +0x10 unconditionally (never 0 while active)
        if let Some(o) = obj_of(&self.objs, self.f_10) {
            ee_sub_12e858(cm, &o, clk);
        }
    }
    fn set_saved_near(&mut self, s: f32) {
        self.save.near = s;
    }
}

/// C2cd3c0 at CM +0x5f0 with the pso's published object (FUN_0014fcc8 stores the pso at +0).
pub struct ClassAttach {
    pub objs: CamObjs,
    pub pso: u32,
    pub inner: CplcyAttach,
}

impl ClassAttach {
    pub fn new(objs: CamObjs) -> Self {
        let obj = CmObject { mat: crate::engine::camera::math::IDENTITY, fov: 1.0, smooth: false };
        Self { objs, pso: 0, inner: CplcyAttach::new(obj) }
    }
    fn sync(&mut self) {
        if let Some(CamState::Attach(o)) = self.objs.lock().unwrap().get(&self.pso) {
            self.inner.obj = *o;
        }
    }
}

impl Cplcy for ClassAttach {
    /// 0x00151900
    fn activate(&mut self, cm: &mut Cm, pso: u32, pv: u32, clk: CmClock) {
        self.pso = pso;
        self.sync();
        self.inner.activate(cm, pso, pv, clk);
    }
    fn deactivate(&mut self, cm: &mut Cm, pso: u32, pv: u32, clk: CmClock) {
        self.inner.deactivate(cm, pso, pv, clk);
    }
    /// 0x00151950
    fn update(&mut self, cm: &mut Cm, h: bool, pad: &CplcyCtx, clk: CmClock) {
        self.sync();
        self.inner.update(cm, h, pad, clk);
    }
    fn set_saved_near(&mut self, s: f32) {
        self.inner.set_saved_near(s);
    }
}

/// CM +0x520 (FUN_00150728, vtable C2cd498): the zoom look (binocucom mode 1, sequence actions 0x3d-0x41).
pub const DESC_ZOOM_2DE460: u32 = 0x2de460;
/// 0x2bd9b8: zoom approach (FUN_0011d018); 0x2bd9f8: point springs (FUN_0011f088); 0x2bd9d8: rotation
/// spring (FUN_0011c950); 0x2bd9e8: fov approach (FUN_0011d2d0).
pub const SMP_ZOOM: [f32; 4] = [1.0, 0.4, 1.5, 0.0];
pub const ZOOM_SPRING: [f32; 3] = [100.0, 40.0, 1.5];
pub const ZOOM_ROT: [f32; 3] = [1.0, 0.5, 0.5];
pub const SMP_ZOOM_FOV: [f32; 4] = [1.0, 0.4, 1.5, 0.0];
/// 0x2a5000: the camera 50 behind the centre
pub const ZOOM_BACK: [f32; 3] = [-50.0, 0.0, 0.0];
/// FUN_0011d4d0 tolerances of the getters (0x38d1b717 / 0x3c23d70a)
pub const ZOOM_EPS: f32 = 1.0e-4;
pub const ZOOM_EPS_U: f32 = 0.01;

/// Sequence-action setters of the zoom look (FUN_001e1e30 cases 0x3d-0x41).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ZoomOp {
    /// 0x3d FUN_00150e20: +0x10 centre object
    Center(u32),
    /// 0x3e FUN_00150e60: +0x14 look-at object
    Target(u32),
    /// 0x3f FUN_00150ea0: object mode key (+0x80 pos, +0x90 Euler, +0xa0 fov)
    Key { pos: [f32; 3], rot: [f32; 3], fov: f32 },
    /// 0x40 FUN_00150ee8: +0x18 zoom target (< 0 = free)
    Zoom(f32),
    /// 0x41 FUN_00150ef0
    Snap,
}

/// Positions the zoom look reads (FUN_001a62e8 of its objects, FUN_001c4a48(0x3dd490) the player),
/// resolved by the class host before the requests run.
#[derive(Clone, Debug, Default)]
pub struct ZoomPos {
    pub player: Option<[f32; 3]>,
    pub objs: HashMap<u32, [f32; 3]>,
}

/// C2cd498 fields by original offset. The zoom motor sounds (+0xc0 0x26 / +0xc4 0x25) are not modelled
/// (no emitter pool in the class camera system, as the default LookWorld); the look count DAT_002de3e4 and
/// DAT_003e1be0 = t written on activation are not read by ported code.
#[derive(Clone, Debug)]
pub struct Zoom {
    pub save: CplcySave,
    pub f_10: u32,
    pub f_14: u32,
    pub f_18: f32,
    /// +0x20 look-at point, +0x30 camera point (free mode)
    pub f_20: [f32; 3],
    pub f_30: [f32; 3],
    /// +0x40 object mode
    pub f_40: i32,
    /// +0x50 pos, +0x60 Euler, +0x70 fov (object mode, current); +0x80 / +0x90 / +0xa0 the key
    pub f_50: [f32; 3],
    pub f_60: [f32; 3],
    pub f_70: f32,
    pub f_80: [f32; 3],
    pub f_90: [f32; 3],
    pub f_a0: f32,
    /// +0xa4 yaw, +0xa8 pitch, +0xac zoom (0..1), +0xb0 zoom speed, +0xb4 factor, +0xb8 zoomed / +0xbc fov
    pub f_a4: f32,
    pub f_a8: f32,
    pub f_ac: f32,
    pub f_b0: f32,
    pub f_b4: f32,
    pub f_b8: f32,
    pub f_bc: f32,
    pub pos: ZoomPos,
}

impl Default for Zoom {
    /// FUN_00150728
    fn default() -> Self {
        Self {
            save: CplcySave::default(),
            f_10: 0,
            f_14: 0,
            f_18: -1.0,
            f_20: [0.0; 3],
            f_30: [0.0; 3],
            f_40: 0,
            f_50: [0.0; 3],
            f_60: [0.0; 3],
            f_70: 0.0,
            f_80: [0.0; 3],
            f_90: [0.0; 3],
            f_a0: 0.0,
            f_a4: 0.0,
            f_a8: 0.0,
            f_ac: 0.0,
            f_b0: 0.0,
            f_b4: 7.0,
            f_b8: 0.0,
            f_bc: 0.0,
            pos: ZoomPos::default(),
        }
    }
}

fn clamp01(v: f32) -> f32 {
    // 0x150a28: 0 unless 0 <= v (NaN -> 0), capped at 1
    if 0.0 <= v { v.min(1.0) } else { 0.0 }
}

impl Zoom {
    fn obj_pos(&self, h: u32) -> [f32; 3] {
        self.pos.objs.get(&h).copied().unwrap_or([0.0; 3])
    }

    /// 0x00150780: centre = +0x10 object, else the player, else the camera position (DAT_002ddfd0)
    pub fn ee_sub_150780(&self, cm: &Cm) -> [f32; 3] {
        if self.f_10 != 0 {
            self.obj_pos(self.f_10)
        } else if let Some(p) = self.pos.player {
            p
        } else {
            [cm.mat[3][0], cm.mat[3][1], cm.mat[3][2]]
        }
    }

    /// 0x001507f8: camera point = centre + R(yaw, pitch) (-50, 0, 0); with a look-at object, 50 short of
    /// the centre away from it (+0xa4 = its yaw)
    pub fn ee_sub_1507f8(&mut self, cm: &Cm) -> [f32; 3] {
        use crate::engine::camera::math::*;
        if self.f_14 == 0 {
            let m = FUN_0011b720(self.f_a4, self.f_a8);
            add(self.ee_sub_150780(cm), v3(mul_v4(&m, v4(ZOOM_BACK, 0.0))))
        } else {
            let c = self.ee_sub_150780(cm);
            let d = sub(self.obj_pos(self.f_14), c);
            let l = len(d);
            // REPLACED: the original divides by 0 when the objects coincide (NaN camera)
            let dir = if l > 0.0 { scale(d, 1.0 / l) } else { [1.0, 0.0, 0.0] };
            self.f_a4 = dir[1].atan2(dir[0]);
            add(c, scale(dir, -len(ZOOM_BACK)))
        }
    }

    /// 0x00150ef0: snap the springs to their targets
    pub fn ee_sub_150ef0(&mut self, cm: &Cm) {
        self.f_20 = self.ee_sub_150780(cm);
        self.f_30 = self.ee_sub_1507f8(cm);
        if 0.0 <= self.f_18 {
            self.f_ac = self.f_18;
        }
        self.f_50 = self.f_80;
        self.f_70 = self.f_a0;
        self.f_60 = self.f_90;
    }

    /// 0x00150e20 / 0x00150e60 / 0x00150ea0 / 0x00150ee8 / 0x00150ef0
    pub fn set(&mut self, cm: &Cm, op: ZoomOp) {
        match op {
            ZoomOp::Center(o) | ZoomOp::Target(o) => {
                if matches!(op, ZoomOp::Center(_)) {
                    self.f_10 = o;
                } else {
                    self.f_14 = o;
                }
                if self.f_40 != 0 {
                    self.ee_sub_150ef0(cm);
                }
                self.f_40 = 0;
            }
            ZoomOp::Key { pos, rot, fov } => {
                self.f_80 = pos;
                self.f_90 = rot;
                self.f_a0 = fov;
                if self.f_40 == 0 {
                    self.ee_sub_150ef0(cm);
                }
                self.f_40 = 1;
            }
            ZoomOp::Zoom(u) => self.f_18 = u,
            ZoomOp::Snap => self.ee_sub_150ef0(cm),
        }
    }

    /// Wait conditions 0x3d / 0x3e / 0x40 (FUN_001e3890): FUN_00150f58 look-at point settled, FUN_00151018
    /// camera point settled, FUN_001510d8 zoom reached
    pub fn waits(&self, cm: &Cm) -> [bool; 3] {
        use crate::engine::physics::driver::FUN_0011d4d0;
        let near = |a: [f32; 3], b: [f32; 3]| (0..3).all(|i| FUN_0011d4d0(a[i], b[i], ZOOM_EPS));
        // FUN_001507f8 writes +0xa4: evaluated on a copy (the getter runs only while a wait polls it)
        let cam = self.clone().ee_sub_1507f8(cm);
        [
            near(self.f_20, self.ee_sub_150780(cm)),
            near(self.f_30, cam),
            !(0.0 <= self.f_18) || FUN_0011d4d0(self.f_ac, self.f_18, ZOOM_EPS_U),
        ]
    }
}

pub type ZoomRef = Arc<Mutex<Zoom>>;

/// C2cd498 instance at CM +0x520 (state shared with the class host's setters).
pub struct CplcyZoom {
    pub z: ZoomRef,
}

impl Cplcy for CplcyZoom {
    /// 0x00150938
    fn activate(&mut self, cm: &mut Cm, _pso: u32, _pv: u32, clk: CmClock) {
        use crate::engine::camera::math::{v3, FUN_0011bc88};
        let mut z = self.z.lock().unwrap();
        z.save = FUN_0014fcc8(cm);
        z.f_a4 = FUN_0011bc88(v3(cm.mat[0])).0;
        z.f_a8 = 0.0;
        z.f_ac = 0.0;
        z.f_b0 = 0.0;
        z.f_b8 = cm.rad_fov_target / z.f_b4;
        z.f_bc = cm.rad_fov_target;
        crate::engine::camera::cm::FUN_0014f610(cm, 49.0, clk);
        z.ee_sub_150ef0(cm);
    }

    /// 0x00151130
    fn deactivate(&mut self, cm: &mut Cm, _pso: u32, _pv: u32, clk: CmClock) {
        let mut z = self.z.lock().unwrap();
        z.f_10 = 0;
        z.f_14 = 0;
        z.f_40 = 0;
        z.f_18 = -1.0;
        FUN_0014d8c0(cm);
        FUN_0014fce8(cm, &z.save, clk);
    }

    /// 0x00150a28
    fn update(&mut self, cm: &mut Cm, _h: bool, _pad: &CplcyCtx, clk: CmClock) {
        use crate::engine::camera::math::{v3, FUN_0011b628, FUN_0011bc88, FUN_0011bff0, FUN_0011d018, FUN_0011d2d0};
        use crate::engine::player::p5_attach::FUN_0011f088;
        use crate::engine::player::p5_states::FUN_0011c950;
        let dt = clk.dt;
        if dt == 0.0 {
            return;
        }
        let mut z = self.z.lock().unwrap();
        if 0.0 <= z.f_18 {
            let u = clamp01(FUN_0011d018(z.f_ac, z.f_18, dt, &SMP_ZOOM).0);
            let old = z.f_ac;
            z.f_ac = u;
            z.f_b0 = (u - old) / dt;
        }
        let mut v = [0.0; 3];
        if z.f_40 == 0 {
            let w = 1.0 - z.f_ac;
            FUN_0014f5a0(cm, (1.0 - w) * z.f_b8 + w * z.f_bc, clk);
            let t = z.ee_sub_1507f8(cm);
            z.f_30 = FUN_0011f088(dt, z.f_30, t, ZOOM_SPRING, &mut v);
            let c = z.ee_sub_150780(cm);
            z.f_20 = FUN_0011f088(dt, z.f_20, c, ZOOM_SPRING, &mut v);
            FUN_0014e1a8(cm, 0.0, 0.0, z.f_30, z.f_20, clk);
        } else {
            // the original's argument registers are lost in the decompile: +0x50 springs to +0x80, the Euler
            // +0x60 rotates to +0x90, the fov +0x70 approaches +0xa0 (as the free mode / FUN_00150ef0 pairs)
            z.f_50 = FUN_0011f088(dt, z.f_50, z.f_80, ZOOM_SPRING, &mut v);
            let r3 = |m: M4| [v3(m[0]), v3(m[1]), v3(m[2])];
            let (r, _) = FUN_0011c950(dt, &r3(FUN_0011b628(z.f_60)), &r3(FUN_0011b628(z.f_90)), ZOOM_ROT);
            let p = z.f_50;
            let m: M4 = [
                [r[0][0], r[0][1], r[0][2], 0.0],
                [r[1][0], r[1][1], r[1][2], 0.0],
                [r[2][0], r[2][1], r[2][2], 0.0],
                [p[0], p[1], p[2], 1.0],
            ];
            z.f_60 = FUN_0011bff0(&m);
            z.f_70 = FUN_0011d2d0(z.f_70, z.f_a0, dt, &SMP_ZOOM_FOV);
            FUN_0014e168(cm, &m, clk);
            FUN_0014f5a0(cm, z.f_70, clk);
            let (y, pi) = FUN_0011bc88(v3(m[0]));
            z.f_a4 = y;
            z.f_a8 = pi;
            let f = if z.f_b8 == z.f_bc { 0.0 } else { (z.f_70 - z.f_b8) / (z.f_bc - z.f_b8) };
            z.f_ac = crate::engine::camera::math::FUN_0011d948(f, [0.0, 1.0]);
        }
    }

    /// 0x00150de8: drops the two sounds (not modelled)
    fn reset(&mut self, _cm: &mut Cm) {}

    fn set_saved_near(&mut self, s: f32) {
        self.z.lock().unwrap().save.near = s;
    }
}
