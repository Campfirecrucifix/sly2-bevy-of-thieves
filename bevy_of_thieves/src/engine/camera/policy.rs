//! Camera policies (CPLCY subclasses, vtable base C2cab90 0x002cab90). Slots: 1 activate (+0x14),
//! 2 deactivate (+0x1c), 3 push (+0x24), 4 revoke (+0x2c), 5 update (+0x34), 6 reset (+0x3c).
//! Ported: base (C2cd528/C2cd570 = 0x0014fcc8/0x0014fce8), blend (C2cd408 0x00151648..0x00151700,
//! cutscene/transition interpolation), attach (C2cd3c0 0x00151900/0x00151950, camera glued to an
//! animated camera object). The follow policy C2db858 is in follow.rs; C2cd4e0, C2cd450, C2cd378 in looks.rs.
//! C2cd498 (zoom look) is not ported yet (see port_tracking).
#![allow(non_snake_case)]

use super::cm::*;
use super::math::*;

/// Per-update input the original reads from globals (pad 0x2dfc00/0x2e0d00 is passed to slot 5).
#[derive(Clone, Copy, Debug, Default)]
pub struct CplcyCtx {
    pub stick_x: f32,
    pub stick_y: f32,
    /// pad +0x103c/+0x1040: left stick (x, y) (read by the follow policy C2db858).
    pub left: [f32; 2],
    /// pad +0x1070/+0x1074/+0x1078: right stick (x, y, magnitude after dead zone).
    pub right: [f32; 3],
    /// DAT_002dfbc4 & 2: input lock (the follow policy's occlusion check is skipped).
    pub input_locked: bool,
}

pub trait Cplcy: Send + Sync {
    fn activate(&mut self, cm: &mut Cm, pso: u32, pv: u32, clk: CmClock);
    fn deactivate(&mut self, cm: &mut Cm, pso: u32, pv: u32, clk: CmClock);
    fn push(&mut self, _cm: &mut Cm, _pv: u32) {}
    fn revoke(&mut self, _cm: &mut Cm, _pv: u32) {}
    fn update(&mut self, cm: &mut Cm, has_cpdefi: bool, pad: &CplcyCtx, clk: CmClock);
    fn reset(&mut self, _cm: &mut Cm) {}
    /// +8 of the policy (written by FUN_0014f630).
    fn set_saved_near(&mut self, _s: f32) {}
}

/// Saved state of the CPLCY base (+4 saved fov target, +8 saved near clip).
#[derive(Clone, Copy, Debug, Default)]
pub struct CplcySave {
    pub fov_target: f32,
    pub near: f32,
}

/// 0x0014fcc8: base activate: remember near clip and fov target.
pub fn FUN_0014fcc8(cm: &Cm) -> CplcySave {
    CplcySave { near: cm.s_near_clip, fov_target: cm.rad_fov_target }
}

/// 0x0014fce8: base deactivate: restore near clip and fov target.
pub fn FUN_0014fce8(cm: &mut Cm, s: &CplcySave, clk: CmClock) {
    FUN_0014f610(cm, s.near, clk);
    cm.rad_fov_target = s.fov_target;
}

/// The base policy pushed by InitCm (cpp 0): keeps the camera where it is.
pub struct CplcyBase;
impl Cplcy for CplcyBase {
    fn activate(&mut self, _cm: &mut Cm, _pso: u32, _pv: u32, _clk: CmClock) {}
    fn deactivate(&mut self, _cm: &mut Cm, _pso: u32, _pv: u32, _clk: CmClock) {}
    fn update(&mut self, _cm: &mut Cm, _h: bool, _pad: &CplcyCtx, _clk: CmClock) {}
}

/// Blend key (pos +0x00, Euler rotation +0x10, fov +0x20).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CmKey {
    pub pos: V3,
    pub rot: V3,
    pub fov: f32,
}

/// Blend descriptor passed as `pv` to C2cd408: {key A*, key B*, ?, duration +8 -> here, curve +0x10,
/// +0x0c cut on deactivate, +0x14 owner object (start/stop hooks)}.
#[derive(Clone, Copy, Debug, Default)]
pub struct CmBlend {
    pub a: CmKey,
    pub b: CmKey,
    pub duration: f32,
    pub curve: usize,
    pub cut_on_end: bool,
}

/// 0x002bda70: blend curves, dot((u^3, u^2, u), coef). 0 linear, 1 -u^3+2u^2, 2 -u^3-u^2+u (as in the ELF,
/// ends at -1), 3 smoothstep.
pub const BLEND_CURVES: [V3; 4] = [[0.0, 0.0, 1.0], [-1.0, 2.0, 0.0], [-1.0, -1.0, 1.0], [-2.0, 3.0, 0.0]];

/// C2cd408 (0x00151610 ctor): interpolate the camera between two keys over `duration` from activation.
#[derive(Default)]
pub struct CplcyBlend {
    pub save: CplcySave,
    pub blend: Option<CmBlend>,
}

impl CplcyBlend {
    /// Set the descriptor that the next SetCmPolicy (`pv`) refers to.
    pub fn new(b: CmBlend) -> Self {
        Self { save: CplcySave::default(), blend: Some(b) }
    }
}

impl Cplcy for CplcyBlend {
    /// 0x00151648 (owner hook +0x14 vtable +0xc is the caller's business).
    fn activate(&mut self, cm: &mut Cm, _pso: u32, _pv: u32, _clk: CmClock) {
        self.save = FUN_0014fcc8(cm);
    }
    /// 0x00151698
    fn deactivate(&mut self, cm: &mut Cm, _pso: u32, _pv: u32, clk: CmClock) {
        FUN_0014fce8(cm, &self.save, clk);
        if self.blend.map(|b| b.cut_on_end).unwrap_or(false) {
            FUN_0014d8c0(cm);
        }
    }
    /// 0x00151700
    fn update(&mut self, cm: &mut Cm, _h: bool, _pad: &CplcyCtx, clk: CmClock) {
        let Some(b) = self.blend else { return };
        let u = ((clk.t - cm.t_activate) / b.duration).clamp(0.0, 1.0);
        let w = dot([u * u * u, u * u, u], BLEND_CURVES[b.curve & 3]);
        let pos = add(scale(b.a.pos, 1.0 - w), scale(b.b.pos, w));
        let rot = add(scale(b.a.rot, 1.0 - w), scale(b.b.rot, w));
        let r = FUN_0011b628(rot);
        let m = [v4(v3(r[0]), 0.0), v4(v3(r[1]), 0.0), v4(v3(r[2]), 0.0), v4(pos, 1.0)];
        FUN_0014e168(cm, &m, clk);
        FUN_0014f5a0(cm, (1.0 - w) * b.a.fov + w * b.b.fov, clk);
    }
    fn set_saved_near(&mut self, s: f32) {
        self.save.near = s;
    }
}

/// Camera object state read by C2cd3c0 (object vtable +0xfc matrix, +0x1bc fov, +0x1d4 smooth flag).
#[derive(Clone, Copy, Debug)]
pub struct CmObject {
    pub mat: M4,
    pub fov: f32,
    pub smooth: bool,
}

/// 0x002bdab0 / 0x002bdac0: rotation and position catch-up profiles of the attach policy.
pub const SMP_ATTACH_ROT: Smp = [5.0, 0.0, 0.5, 0.0];
pub const SMP_ATTACH_POS: Smp = [2000.0, 0.0, 1.0, 0.45];

/// C2cd3c0 (0x001518c8 ctor): the camera follows an animated camera object (cutscenes).
pub struct CplcyAttach {
    pub save: CplcySave,
    pub obj: CmObject,
}

impl CplcyAttach {
    pub fn new(obj: CmObject) -> Self {
        Self { save: CplcySave::default(), obj }
    }
}

/// 0x00151950 core: catch up from the current camera matrix to the object's.
pub fn FUN_00151950(cm: &mut Cm, obj: &CmObject, clk: CmClock) {
    let mut m = obj.mat;
    if cm.cut == 0 && obj.smooth {
        // FUN_0011bc00: relative rotation object -> camera (world frame) = cam * obj^T
        let mut objt = IDENTITY;
        for i in 0..3 {
            for k in 0..3 {
                objt[i][k] = obj.mat[k][i];
            }
        }
        let mut cam = cm.mat;
        cam[3] = [0.0, 0.0, 0.0, 1.0];
        let rel = FUN_0011c168(&cam, &objt);
        let (ang, axis) = FUN_0011bcf8(&rel);
        if !(ang < 0.0001) {
            let a = FUN_0011d018(ang, 0.0, clk.dt, &SMP_ATTACH_ROT).0;
            let r = FUN_0011b500(a, axis);
            for i in 0..3 {
                m[i] = v4(v3(mul_v4(&r, v4(v3(obj.mat[i]), 0.0))), 0.0);
            }
        }
        let opos = v3(obj.mat[3]);
        let d = sub(v3(cm.mat[3]), opos);
        let l = len(d);
        let mut p = opos;
        if 0.0001 <= l {
            let nl = FUN_0011d018(l, 0.0, clk.dt, &SMP_ATTACH_POS).0;
            p = add(opos, scale(d, nl / l));
        }
        m[3] = v4(p, 1.0);
        m[0][3] = 0.0;
        m[1][3] = 0.0;
        m[2][3] = 0.0;
    }
    FUN_0014e168(cm, &m, clk);
    if cm.rad_fov_target == cm.rad_fov {
        FUN_0014f5a0(cm, obj.fov, clk);
    } else {
        cm.rad_fov_target = obj.fov;
    }
}

impl Cplcy for CplcyAttach {
    /// 0x00151900: near clip 49, fov target = object fov.
    fn activate(&mut self, cm: &mut Cm, _pso: u32, _pv: u32, clk: CmClock) {
        self.save = FUN_0014fcc8(cm);
        FUN_0014f610(cm, 49.0, clk);
        cm.rad_fov_target = self.obj.fov;
    }
    fn deactivate(&mut self, cm: &mut Cm, _pso: u32, _pv: u32, clk: CmClock) {
        FUN_0014fce8(cm, &self.save, clk);
    }
    fn update(&mut self, cm: &mut Cm, _h: bool, _pad: &CplcyCtx, clk: CmClock) {
        let o = self.obj;
        FUN_00151950(cm, &o, clk);
    }
    fn set_saved_near(&mut self, s: f32) {
        self.save.near = s;
    }
}
