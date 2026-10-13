//! Camera policies C2cd4e0 (first-person look, ctor 0x0014fde0), C2cd450 (close-up on a spring,
//! ctor 0x00151208) and C2cd378 (two-object look-at, ctor 0x0014d3d8 tail). Ported 1:1; world objects,
//! the pad and the sound pool are reached through `LookWorld` (replaces the vtable/global calls).
#![allow(non_snake_case)]

use super::cm::*;
use super::math::*;
use super::policy::{Cplcy, CplcyCtx, CplcySave, FUN_0014fcc8, FUN_0014fce8};
use super::sound::{self, CmEmitter};
use crate::engine::audio::emitter::EmitterSystem;

/// World access of the look policies.
pub trait LookWorld: Send + Sync {
    /// FUN_00132b10(pso, 0x34): the object is a JT (Sly).
    fn is_jt(&mut self, _pso: u32) -> bool {
        false
    }
    /// FUN_001a62e8: world position of an SO.
    fn so_pos(&mut self, _pso: u32) -> V3 {
        [0.0; 3]
    }
    /// FUN_001a6318 (SO vtable +0xfc): world orientation of an SO (row 0 = forward).
    fn so_mat(&mut self, _pso: u32) -> M4 {
        IDENTITY
    }
    /// SO +0x4cc (JT state; 9 = the close-up idle jiggle).
    fn so_4cc(&mut self, _pso: u32) -> i32 {
        0
    }
    /// FUN_002581c0(.., *(0x3e0fdc), 0x3d4a60) result +4: inverted look option.
    fn invert_look(&mut self) -> bool {
        false
    }
    /// The sound pool 0x3dd05c (None: sounds are skipped).
    fn emitters(&mut self) -> Option<&mut EmitterSystem> {
        None
    }
    /// FUN_0011d448(lo, hi): random float in [lo, hi].
    fn rand_range(&mut self, lo: f32, _hi: f32) -> f32 {
        lo
    }
    /// DAT_003e1be0 = t (time of the last look-mode frame).
    fn set_t_look(&mut self, _t: f32) {}
    /// FUN_0012dc18(pso, obj, 1, 8, 3, 0x2a2dd0): create the close-up spring point; returns a handle.
    fn xp_new(&mut self, _pso: u32, _obj: u32) -> u32 {
        0
    }
    /// handle +0x20: spring point position.
    fn xp_pos(&mut self, _h: u32) -> V3 {
        [0.0; 3]
    }
    /// FUN_0012d108(handle, pos).
    fn xp_set(&mut self, _h: u32, _p: V3) {}
    /// FUN_0012e460(&handle): release.
    fn xp_free(&mut self, _h: u32) {}
    /// object +0x40: world matrix of the close-up camera object.
    fn obj_mat(&mut self, _obj: u32) -> M4 {
        IDENTITY
    }
}

pub struct NoLookWorld;
impl LookWorld for NoLookWorld {}

fn snd(w: &mut dyn LookWorld, f: impl FnOnce(&mut EmitterSystem)) {
    if let Some(es) = w.emitters() {
        f(es)
    }
}

/// 0x002bd950/0x002bd958: JT look limits (yaw -10..10, pitch -1..1); 0x002bd968: other (+-1, +-1).
pub const LOOK_LIM_JT: [f32; 4] = [-10.0, 10.0, -1.0, 1.0];
pub const LOOK_LIM_SO: [f32; 4] = [-1.0, 1.0, -1.0, 1.0];
/// 0x002bd960
pub const LOOK_PITCH_RANGE: [f32; 2] = [-1.5, 1.5];
/// 0x002bd980: zoom speed polynomial in u; 0x002bd990/94: stick yaw/pitch scale; 0x002bd998/9c: rate slew.
pub const LOOK_ZOOM_POLY: V3 = [2.0, 0.0, -1.0];
pub const LOOK_YAW_SCALE: f32 = 0.9;
pub const LOOK_PITCH_SCALE: f32 = 0.8;
pub const LOOK_SLEW: [f32; 2] = [10.0, 10.0];
/// 0x002bd9a0: limit approach profile; 0x002bd9b0: turn rate lerp by fov (0 .. 1.3).
pub const SMP_LOOK_LIMIT: Smp = [1.0, 0.0, 0.2, 0.0];
pub const LOOK_TURN: [f32; 2] = [0.0, 1.3];
/// 0x002a5000: eye -> camera offset along the view (camera 50 behind the eye).
pub const LOOK_BACK: V3 = [-50.0, 0.0, 0.0];
/// Eye height over a JT (0x0014fe58).
pub const LOOK_JT_EYE: f32 = 150.0;

/// C2cd4e0: first-person look (pso = viewer). Fields by original offset.
pub struct CplcyLook {
    pub save: CplcySave,
    pub pso: u32,
    /// +0x10 yaw, +0x14 pitch, +0x18/+0x1c yaw/pitch rates
    pub yaw: f32,
    pub pitch: f32,
    pub rate_yaw: f32,
    pub rate_pitch: f32,
    /// +0x20 zoom (0..1), +0x24 zoom speed
    pub zoom: f32,
    pub v_zoom: f32,
    /// +0x28 zoom factor (7), +0x2c zoomed fov, +0x30 normal fov
    pub zoom_factor: f32,
    pub fov_zoom: f32,
    pub fov: f32,
    /// +0x34 zoom motor loop (0x26), +0x38 second loop (0x25)
    pub snd_zoom: CmEmitter,
    pub snd_25: CmEmitter,
    /// +0x3c..+0x48: yaw lo/hi, pitch lo/hi
    pub lim: [f32; 4],
    /// +0x4c/+0x50: camera yaw/pitch at activation (restored as a cut when the viewer is not a JT)
    pub yaw_saved: f32,
    pub pitch_saved: f32,
    /// +0x54: activation count (C2cd498 activate also bumps it)
    pub count: u32,
    pub world: Box<dyn LookWorld>,
}

impl CplcyLook {
    /// 0x0014fde0
    pub fn new(world: Box<dyn LookWorld>) -> Self {
        CplcyLook {
            save: CplcySave::default(),
            pso: 0,
            yaw: 0.0,
            pitch: 0.0,
            rate_yaw: 0.0,
            rate_pitch: 0.0,
            zoom: 0.0,
            v_zoom: 0.0,
            zoom_factor: 7.0,
            fov_zoom: 0.0,
            fov: 0.0,
            snd_zoom: CmEmitter::default(),
            snd_25: CmEmitter::default(),
            lim: [0.0; 4],
            yaw_saved: 0.0,
            pitch_saved: 0.0,
            count: 0,
            world,
        }
    }

    /// 0x0014fe58: eye = SO position (+150 up for a JT).
    pub fn FUN_0014fe58(&mut self) -> V3 {
        let p = self.world.so_pos(self.pso);
        if self.world.is_jt(self.pso) { add(p, [0.0, 0.0, LOOK_JT_EYE]) } else { p }
    }

    /// 0x0014fec8: camera position = eye + R(yaw, pitch) * (-50, 0, 0).
    pub fn FUN_0014fec8(&mut self) -> V3 {
        let m = FUN_0011b720(self.yaw, self.pitch);
        let e = self.FUN_0014fe58();
        add(e, v3(mul_v4(&m, v4(LOOK_BACK, 0.0))))
    }
}

impl Cplcy for CplcyLook {
    /// 0x0014ff38
    fn activate(&mut self, cm: &mut Cm, pso: u32, _pv: u32, clk: CmClock) {
        self.save = FUN_0014fcc8(cm);
        self.pso = pso;
        let (mut a, mut b) = (self.snd_zoom, self.snd_25);
        snd(self.world.as_mut(), |es| {
            sound::new_emitter(es, &mut a, 0x26, 0);
            sound::new_emitter(es, &mut b, 0x25, 0);
            sound::FUN_001fb018(es, &a);
        });
        self.snd_zoom = a;
        self.snd_25 = b;
        let (y, p) = FUN_0011bc88(v3(cm.mat[0]));
        self.yaw = y;
        self.pitch = p;
        if !self.world.is_jt(pso) {
            self.lim = LOOK_LIM_SO;
            self.pitch_saved = self.pitch;
            self.yaw_saved = self.yaw;
            let (y, p) = FUN_0011bc88(v3(self.world.so_mat(pso)[0]));
            self.yaw = y;
            let p = FUN_0011d948(p, LOOK_PITCH_RANGE);
            self.pitch = p;
            self.lim[0] += y;
            self.lim[1] += y;
            // quirk: the pitch limits are overwritten with the clamped pitch itself (pitch locked)
            self.lim[2] = FUN_0011d948(p, LOOK_PITCH_RANGE);
            self.lim[3] = FUN_0011d948(self.pitch, LOOK_PITCH_RANGE);
        } else {
            self.lim = LOOK_LIM_JT;
        }
        self.rate_yaw = 0.0;
        self.rate_pitch = 0.0;
        self.zoom = 0.0;
        self.v_zoom = 0.0;
        self.fov_zoom = cm.rad_fov_target / self.zoom_factor;
        self.fov = cm.rad_fov_target;
        FUN_0014f610(cm, 49.0, clk);
        self.count += 1;
        self.world.set_t_look(clk.t);
    }

    /// 0x00150610
    fn deactivate(&mut self, cm: &mut Cm, _pso: u32, _pv: u32, clk: CmClock) {
        let (mut a, mut b) = (self.snd_zoom, self.snd_25);
        snd(self.world.as_mut(), |es| {
            if a.id.is_some() {
                sound::FUN_001fb098(es, 0.0, &mut a);
                sound::FUN_00258040(es, &mut a, None);
            }
            if b.id.is_some() {
                sound::FUN_001fb098(es, 0.5, &mut b);
                sound::FUN_00258040(es, &mut b, None);
            }
        });
        self.snd_zoom = a;
        self.snd_25 = b;
        if !self.world.is_jt(self.pso) {
            FUN_0014d8d8(cm, (self.yaw_saved + PI).to_bits(), (-self.pitch_saved).to_bits());
        } else {
            FUN_0014d8d8(cm, (self.yaw + PI).to_bits(), (-self.pitch).to_bits());
        }
        FUN_0014fce8(cm, &self.save, clk);
    }

    /// 0x00150160
    fn update(&mut self, cm: &mut Cm, _h: bool, pad: &CplcyCtx, clk: CmClock) {
        let u = self.zoom;
        let ry = pad.right[1];
        let zv = (LOOK_ZOOM_POLY[0] + u * (LOOK_ZOOM_POLY[1] + u * LOOK_ZOOM_POLY[2])) * ry * ry.abs();
        self.v_zoom = zv;
        let u = (u + zv * clk.dt).clamp(0.0, 1.0);
        self.zoom = u;
        let mut a = self.snd_zoom;
        snd(self.world.as_mut(), |es| {
            if zv != 0.0 && 0.0 < u && u < 1.0 {
                sound::FUN_001fb2c0(es, FUN_0011d948(zv.abs(), [-1.0, 1.0]), &mut a);
                sound::FUN_001fb1f8(es, FUN_0011d948(zv.abs() * 0.5 + 0.5, [0.0, 1.0]), &a);
                if sound::playing(es, &a) != 0 {
                    sound::FUN_001fb060(es, &a);
                }
            } else if sound::playing(es, &a) == 0 {
                sound::FUN_001fb018(es, &a);
            }
        });
        self.snd_zoom = a;
        let u = self.zoom;
        FUN_0014f5a0(cm, (1.0 - (1.0 - u)) * self.fov_zoom + (1.0 - u) * self.fov, clk);
        let mut tp = -pad.left[1].abs() * pad.left[1] * LOOK_PITCH_SCALE;
        let ty = -pad.left[0].abs() * pad.left[0] * LOOK_YAW_SCALE;
        if self.world.invert_look() {
            tp = -tp;
        }
        let ry = self.rate_yaw + FUN_0011cfe0(ty - self.rate_yaw, LOOK_SLEW[0] * clk.dt);
        let rp = self.rate_pitch + FUN_0011cfe0(tp - self.rate_pitch, LOOK_SLEW[1] * clk.dt);
        let h = clk.dt * 0.5;
        let lo = FUN_0011d018(self.yaw, self.lim[0], h, &SMP_LOOK_LIMIT).1;
        let hi = FUN_0011d018(self.yaw, self.lim[1], h, &SMP_LOOK_LIMIT).1;
        self.rate_yaw = if lo <= ry { if ry <= hi { ry } else { hi } } else { lo };
        let lo = FUN_0011d018(self.pitch, self.lim[2], h, &SMP_LOOK_LIMIT).1;
        let hi = FUN_0011d018(self.pitch, self.lim[3], h, &SMP_LOOK_LIMIT).1;
        self.rate_pitch = if lo <= rp { if rp <= hi { rp } else { hi } } else { lo };
        let k = (1.0 - cm.rad_fov) * LOOK_TURN[0] + cm.rad_fov * LOOK_TURN[1];
        self.yaw += k * self.rate_yaw * clk.dt;
        self.pitch += k * self.rate_pitch * clk.dt;
        if 6.2831855 <= self.lim[1] - self.lim[0] {
            self.yaw = FUN_0011cf60(self.yaw);
        }
        let pos = self.FUN_0014fec8();
        let eye = self.FUN_0014fe58();
        FUN_0014e1a8(cm, 0.0, 0.0, pos, eye, clk);
        self.world.set_t_look(clk.t);
    }

    /// 0x001505d8: drop both emitters.
    fn reset(&mut self, _cm: &mut Cm) {
        let (mut a, mut b) = (self.snd_zoom, self.snd_25);
        snd(self.world.as_mut(), |es| {
            sound::FUN_00258040(es, &mut b, None);
            sound::FUN_00258040(es, &mut a, None);
        });
        self.snd_zoom = a;
        self.snd_25 = b;
    }

    fn set_saved_near(&mut self, s: f32) {
        self.save.near = s;
    }
}

/// 0x002bda10/20, 0x002bda30/40: right-stick offset polynomials and clamps of the close-up.
pub const CLOSEUP_X_POLY: V3 = [0.0, 1.1, -0.3];
pub const CLOSEUP_X_RANGE: [f32; 2] = [0.0, 0.5];
pub const CLOSEUP_Y_POLY: V3 = [0.0, 0.5, -0.1];
pub const CLOSEUP_Y_RANGE: [f32; 2] = [0.0, 0.4];
/// 0x002bda48..: idle jiggle target ranges (y, z), re-pick period range; 0x002bda60/64 spring k, d.
pub const CLOSEUP_JIGGLE: [[f32; 2]; 3] = [[-0.01, 0.01], [-0.04, 0.04], [1.5, 2.0]];
pub const CLOSEUP_K: f32 = 5.0;
pub const CLOSEUP_D: f32 = 1.0;

/// C2cd450: close-up on a camera object (pv) whose position point rides a spring (stick nudges it).
pub struct CplcyCloseup {
    pub save: CplcySave,
    pub pso: u32,
    /// +0x10 camera object, +0x14 spring point handle, +0x18 next jiggle time
    pub obj: u32,
    pub xp: u32,
    pub t_next: f32,
    /// +0x20 jiggle target, +0x30 spring velocity
    pub target: V3,
    pub vel: V3,
    pub world: Box<dyn LookWorld>,
}

impl CplcyCloseup {
    pub fn new(world: Box<dyn LookWorld>) -> Self {
        CplcyCloseup { save: CplcySave::default(), pso: 0, obj: 0, xp: 0, t_next: 0.0, target: [0.0; 3], vel: [0.0; 3], world }
    }
}

impl Cplcy for CplcyCloseup {
    /// 0x00151240: near clip 25, fov 1.15, spring point on the object.
    fn activate(&mut self, cm: &mut Cm, pso: u32, pv: u32, clk: CmClock) {
        self.save = FUN_0014fcc8(cm);
        self.pso = pso;
        self.obj = pv;
        FUN_0014f610(cm, 25.0, clk);
        FUN_0014f5a0(cm, f32::from_bits(0x3f933333), clk);
        self.xp = self.world.xp_new(self.pso, self.obj);
        self.t_next = -1.0;
    }
    /// 0x001512e0
    fn deactivate(&mut self, cm: &mut Cm, _pso: u32, _pv: u32, clk: CmClock) {
        self.world.xp_free(self.xp);
        self.xp = 0;
        FUN_0014d8c0(cm);
        FUN_0014fce8(cm, &self.save, clk);
    }
    /// 0x00151340
    fn update(&mut self, cm: &mut Cm, _h: bool, pad: &CplcyCtx, clk: CmClock) {
        let (pos, vel);
        if 0.0 < pad.right[2] {
            let sx = if 0.0 < pad.right[0] { -1.0 } else { 1.0 };
            let sy = if 0.0 < pad.right[1] { 1.0 } else { -1.0 };
            let ax = pad.right[0].abs();
            let x = FUN_0011d948(CLOSEUP_X_POLY[0] + ax * (CLOSEUP_X_POLY[1] + ax * CLOSEUP_X_POLY[2]), CLOSEUP_X_RANGE);
            let ay = pad.right[1].abs();
            let y = FUN_0011d948(CLOSEUP_Y_POLY[0] + ay * (CLOSEUP_Y_POLY[1] + ay * CLOSEUP_Y_POLY[2]), CLOSEUP_Y_RANGE);
            pos = [sx * x, sy * y, 0.0];
            vel = [0.0; 3];
        } else if self.world.so_4cc(self.pso) == 9 {
            if self.t_next < clk.t {
                let a = self.world.rand_range(CLOSEUP_JIGGLE[0][0], CLOSEUP_JIGGLE[0][1]);
                let b = self.world.rand_range(CLOSEUP_JIGGLE[1][0], CLOSEUP_JIGGLE[1][1]);
                self.target = [0.0, a, b];
                let t = clk.t;
                self.t_next = t + self.world.rand_range(CLOSEUP_JIGGLE[2][0], CLOSEUP_JIGGLE[2][1]);
            }
            let x = self.world.xp_pos(self.xp);
            let acc = sub(scale(sub(self.target, x), CLOSEUP_K), scale(self.vel, CLOSEUP_D));
            let v2 = add(self.vel, scale(acc, clk.dt));
            let h = clk.dt * 0.5;
            pos = add(add(x, scale(self.vel, h)), scale(v2, h));
            vel = v2;
        } else {
            pos = [0.0; 3];
            vel = [0.0; 3];
        }
        self.world.xp_set(self.xp, pos);
        self.vel = vel;
        let m = self.world.obj_mat(self.obj);
        FUN_0014e168(cm, &m, clk);
    }
    fn set_saved_near(&mut self, s: f32) {
        self.save.near = s;
    }
}

/// 0x002bdacc: focus weight of object A; 0x002bdad0.. height / back / rise / fov polynomials in the
/// horizontal A-B distance, each with its clamp.
pub const LOOKAT_W: f32 = 0.45;
pub const LOOKAT_EYE_Z: f32 = 150.0;
pub const LOOKAT_HEIGHT: (V3, [f32; 2]) = ([200.0, 0.22, 0.0], [225.0, 650.0]);
pub const LOOKAT_BACK: (V3, [f32; 2]) = ([600.0, 0.33, 0.0], [600.0, 1500.0]);
pub const LOOKAT_RISE: (V3, [f32; 2]) = ([200.0, 0.225, 0.0], [0.0, 500.0]);
pub const LOOKAT_FOV: (V3, [f32; 2]) = ([0.6, 0.00045, 0.0], [0.7, 1.2]);

fn poly_clamp(l: f32, p: &(V3, [f32; 2])) -> f32 {
    let x = p.0[0] + l * (p.0[1] + l * p.0[2]);
    FUN_0011d948(x, p.1)
}

/// 0x0011e390: a vector perpendicular to `v` (smallest component zeroed, the other two swapped/negated).
pub fn FUN_0011e390(v: V3) -> V3 {
    let (x, y, z) = (v[0], v[1], v[2]);
    if x.abs() <= y.abs() {
        if x.abs() <= z.abs() { [0.0, z, -y] } else { [y, -x, 0.0] }
    } else if z.abs() < y.abs() {
        [y, -x, 0.0]
    } else {
        [z, 0.0, -x]
    }
}

/// 0x0011b830 (via 0x0011b7e0): orientation looking along `dir` with +Z up: rows forward, left, up.
pub fn FUN_0011b7e0(dir: V3) -> [V3; 3] {
    let l = len(dir);
    let f = if l < 0.0001 { [1.0, 0.0, 0.0] } else { scale(dir, 1.0 / l) };
    let c = cross([0.0, 0.0, 1.0], f);
    let cl = len(c);
    let left = if cl < 0.0001 { FUN_0011e390(f) } else { scale(c, 1.0 / cl) };
    [f, left, cross(f, left)]
}

/// C2cd378: frame two objects (pv = A at +0x10, pso = B at +0x14) from the side.
pub struct CplcyLookAt {
    pub save: CplcySave,
    pub a: u32,
    pub b: u32,
    pub world: Box<dyn LookWorld>,
}

impl CplcyLookAt {
    pub fn new(world: Box<dyn LookWorld>) -> Self {
        CplcyLookAt { save: CplcySave::default(), a: 0, b: 0, world }
    }
}

impl Cplcy for CplcyLookAt {
    /// 0x00151bf0
    fn activate(&mut self, cm: &mut Cm, pso: u32, pv: u32, _clk: CmClock) {
        self.save = FUN_0014fcc8(cm);
        self.a = pv;
        self.b = pso;
    }
    /// inherited 0x0014fce8
    fn deactivate(&mut self, cm: &mut Cm, _pso: u32, _pv: u32, clk: CmClock) {
        FUN_0014fce8(cm, &self.save, clk);
    }
    /// 0x00151c38
    fn update(&mut self, cm: &mut Cm, _h: bool, _pad: &CplcyCtx, clk: CmClock) {
        let pa = self.world.so_pos(self.a);
        let pb = self.world.so_pos(self.b);
        let mut d = sub(pa, pb);
        d[2] = 0.0;
        let l = len(d);
        let pa = self.world.so_pos(self.a);
        let pb = self.world.so_pos(self.b);
        let mut mid = add(scale(pa, LOOKAT_W), scale(pb, 1.0 - LOOKAT_W));
        mid[2] = LOOKAT_EYE_Z;
        let mut eye = mid;
        eye[2] += poly_clamp(l, &LOOKAT_HEIGHT);
        let n = cross(d, [0.0, 0.0, 1.0]);
        let n = scale(n, 1.0 / len(n));
        let mut cam = add(mid, scale(n, poly_clamp(l, &LOOKAT_BACK)));
        cam[2] += poly_clamp(l, &LOOKAT_RISE);
        let r = FUN_0011b7e0(sub(eye, cam));
        let m = [v4(r[0], 0.0), v4(r[1], 0.0), v4(r[2], 0.0), v4(cam, 1.0)];
        FUN_0014e168(cm, &m, clk);
        FUN_0014f5a0(cm, poly_clamp(l, &LOOKAT_FOV), clk);
    }
    fn set_saved_near(&mut self, s: f32) {
        self.save.near = s;
    }
}
