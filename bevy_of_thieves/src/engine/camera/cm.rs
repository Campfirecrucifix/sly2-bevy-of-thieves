//! CM (game camera) core, ported 1:1 from SCUS_973.16 0x0014d3d8..0x0014f898.
//! The single camera instance lives at 0x002ddf40 (`g_cm`, also the target of the level's property list
//! FUN_0013f810(0x2ddf40) and of Splice `find-camera-object`). Field names follow the Sly 1 CM layout
//! where the offsets match; the original offset is kept in each field's comment.
//!
//! Camera space (FUN_0014d8f8): mat column 0 = forward (view depth), column 1 = left, column 2 = up
//! (world is Z-up). Camera x (screen right) = -left, camera y (screen down) = -up, z = forward.
#![allow(non_snake_case)]

use super::math::*;
use super::policy::{Cplcy, CplcyCtx};

/// Global clock values the camera reads (DAT_002dded0 = t, DAT_002dded4 = dt, DAT_002ddee0 = dt used by
/// the stereo/collision code (unscaled frame time)).
#[derive(Clone, Copy, Debug, Default)]
pub struct CmClock {
    pub t: f32,
    pub dt: f32,
    pub dt_real: f32,
}

/// CPDEFI (0x50 bytes, filled by the focus object's vtable +0x194 into cm+0x270).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Cpdefi {
    /// +0x00..+0x0c
    pub grfcpd: [u32; 4],
    /// +0x10
    pub pos_base: V4,
    /// +0x20
    pub v_base: V4,
    /// +0x30
    pub rad_home: f32,
    /// +0x34
    pub s_adjust: f32,
    /// +0x38, +0x3c
    pub t_move_last: f32,
    pub unk_3c: f32,
    /// +0x40
    pub pos_adjust: V4,
}

/// One policy stack entry (cm+0x310, 16 bytes): {cpp priority, CPLCY*, param (focus SO), pv}.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cpr {
    pub cpp: i32,
    /// index into `Cm::policies`
    pub pcplcy: usize,
    pub pso: u32,
    pub pv: u32,
}

/// Queries the camera makes into the world (objects are owned by WP-B; these replace the vtable calls).
pub trait CmWorld {
    /// focus SO vtable +0x194: fill the CPDEFI.
    fn get_cpdefi(&mut self, pso: u32) -> Cpdefi;
    /// FUN_00132b10(pso, 0x34) (is a JT = Sly) -> Some(jt+0x4f4 state), else None.
    fn jt_state(&mut self, pso: u32) -> Option<i32>;
}

/// A world with no objects (tests / title screen).
pub struct NoWorld;
impl CmWorld for NoWorld {
    fn get_cpdefi(&mut self, _pso: u32) -> Cpdefi {
        Cpdefi::default()
    }
    fn jt_state(&mut self, _pso: u32) -> Option<i32> {
        None
    }
}

pub const CM_MAX_POLICIES: usize = 8;

#[derive(bevy_ecs::prelude::Resource)]
pub struct Cm {
    /// +0x10 / +0x14: fade-target object and its +0xe0 (FUN_0014e148).
    pub pso_fade: u32,
    pub fade_e0: u32,
    /// +0x18: MRD ratio (SetCmMrdRatio FUN_0014f730).
    pub r_mrd_ratio: f32,
    /// +0x1c: max(r_mrd_ratio / fov, 1).
    pub r_mrd: f32,
    /// +0x20: vertical field of view (radians). The original FOV is VERTICAL (tan(fov/2) -> +0x44).
    pub rad_fov: f32,
    /// +0x24: target fov (approached by FUN_0011d1c0 when no cut).
    pub rad_fov_target: f32,
    /// +0x28: fov velocity.
    pub drad_fov: f32,
    /// +0x2c..+0x38: fov approach profile (0.5, 0, 0.1, 10).
    pub smp_fov: Smp,
    /// +0x3c: aspect = 640/492.8 (= 1.2987013, 0x3fa63bda) on PS2.
    pub r_aspect: f32,
    /// +0x40: tan(fov/2) * aspect (horizontal half-extent at depth 1).
    pub x_scale: f32,
    /// +0x44: tan(fov/2) (vertical half-extent at depth 1).
    pub y_scale: f32,
    /// +0x48: radius of the near-clip rectangle + 1.
    pub s_near_radius: f32,
    /// +0x4c / +0x50
    pub s_near_clip: f32,
    pub s_far_clip: f32,
    /// +0x60..+0x9c: orientation (3 columns) and position (+0x90).
    pub mat: M4,
    /// +0xa0..+0xcc: the 4 frustum plane normals, transposed (x row, y row, z row).
    pub anormal_frustrum_transpose: [V4; 3],
    /// +0xd0..+0xfc: the wide frustum (x 7.93, y 18.1) transposed.
    pub anormal_frustrum_wide_transpose: [V4; 3],
    /// +0x100: projection.
    pub mat_proj: M4,
    /// +0x140: world -> clip (with shake).
    pub mat_world_to_clip: M4,
    /// +0x180: clip -> world.
    pub mat_clip_to_world: M4,
    /// +0x1c0: frustum plane normals (left, right, top, bottom; outward).
    pub anormal_frustrum: [V4; 4],
    /// +0x200: wide frustum normals.
    pub anormal_frustrum_wide: [V4; 4],
    /// +0x240/+0x244/+0x248/+0x24c: fog near, far, max, rgba (fog GS setup FUN_0014e270 -> +0x250 is WP-C).
    pub s_fog_near: f32,
    pub s_fog_far: f32,
    pub u_fog_max: f32,
    pub rgba_fog: u32,
    /// +0x260: focus SO whose CPDEFI is cached in +0x270.
    pub pso_cpdefi: u32,
    /// +0x270
    pub cpdefi: Cpdefi,
    /// +0x2c0/+0x2d0, +0x2e0/+0x2f0: adjust springs (pos, vel).
    pub pos_adjust0: V3,
    pub v_adjust0: V3,
    pub pos_adjust1: V3,
    pub v_adjust1: V3,
    /// +0x300: cut level (0 none, 1 cut, 2 cut with params +0x304/+0x308).
    pub cut: i32,
    pub cut_p0: u32,
    pub cut_p1: u32,
    /// +0x30c/+0x310: policy stack (max 8, sorted by cpp descending; [0] is active).
    pub acpr: Vec<Cpr>,
    /// +0x390: time the active policy was activated.
    pub t_activate: f32,
    /// Policy objects embedded in CM at +0x3a0..+0x60c (CPLCY subclasses).
    pub policies: Vec<Option<Box<dyn Cplcy>>>,
    /// +0x618: shake start time (-10 = none), +0x61c shake kind.
    pub t_shake: f32,
    pub shake_kind: i32,
    /// +0x620/+0x624: stereo eye separation and its target (3D mode, DAT_003e12c8).
    pub s_stereo: f32,
    pub s_stereo_target: f32,
    /// DAT_003e12c8: stereo rendering enabled.
    pub f_stereo: bool,
    /// Hor+ (port addition, not in the original): multiplier on tan(fov/2). 1.0 = original behaviour.
    /// See `widescreen.rs`.
    pub hor_plus_y_scale: f32,
}

impl Default for Cm {
    fn default() -> Self {
        Self::new()
    }
}

/// 0x002bd81c..: fog defaults (500, 20000, 0.5).
pub const FOG_DEFAULTS: [f32; 3] = [500.0, 20000.0, 0.5];
/// 0x3fa63bda: PS2 aspect 640 / 492.8 (448 lines * 1.1 display stretch).
pub const R_ASPECT_PS2: f32 = 1.2987013;
/// 0x002bd810: stereo separation approach profile.
pub const SMP_STEREO: Smp = [20.0, 10.0, 0.5, 500.0];
/// 0x002bd850/0x002bd854: shake duration and angular frequency.
pub const SHAKE_DURATION: f32 = 0.5;
pub const SHAKE_FREQ: f32 = 50.0;
/// 0x002bd860..68: shake envelope polynomial 1 + u*(-2 + u*1) = (1-u)^2, clamped to [0x2bbb60, 0x2bbb64] = [0, 1].
pub const SHAKE_ENV: [f32; 3] = [1.0, -2.0, 1.0];
/// 0x002bd870: shake translation axis (zero in the retail ELF), 0x002bd880 rotation vector (roll 0.05 rad),
/// 0x002bd890 projection scale delta (zero), 0x002bd8a0 projection centre delta (zero).
pub const SHAKE_TRANS: V3 = [0.0, 0.0, 0.0];
pub const SHAKE_ROT: V3 = [0.05, 0.0, 0.0];
pub const SHAKE_PROJ_SCALE: V3 = [0.0, 0.0, 0.0];
pub const SHAKE_PROJ_CENTER: V3 = [0.0, 0.0, 0.0];

impl Cm {
    /// 0x0014d3d8 (ctor) + 0x0014d6d8 (InitCm) with the default policy set.
    pub fn new() -> Self {
        let mut cm = Cm {
            pso_fade: 0,
            fade_e0: 0,
            r_mrd_ratio: 0.0,
            r_mrd: 0.0,
            rad_fov: 0.0,
            rad_fov_target: 0.0,
            drad_fov: 0.0,
            smp_fov: [0.0; 4],
            r_aspect: 0.0,
            x_scale: 0.0,
            y_scale: 0.0,
            s_near_radius: 0.0,
            s_near_clip: 0.0,
            s_far_clip: 0.0,
            mat: IDENTITY,
            anormal_frustrum_transpose: [[0.0; 4]; 3],
            anormal_frustrum_wide_transpose: [[0.0; 4]; 3],
            mat_proj: IDENTITY,
            mat_world_to_clip: IDENTITY,
            mat_clip_to_world: IDENTITY,
            anormal_frustrum: [[0.0; 4]; 4],
            anormal_frustrum_wide: [[0.0; 4]; 4],
            s_fog_near: 0.0,
            s_fog_far: 0.0,
            u_fog_max: 0.0,
            rgba_fog: 0,
            pso_cpdefi: 0,
            cpdefi: Cpdefi::default(),
            pos_adjust0: [0.0; 3],
            v_adjust0: [0.0; 3],
            pos_adjust1: [0.0; 3],
            v_adjust1: [0.0; 3],
            cut: 0,
            cut_p0: 0,
            cut_p1: 0,
            acpr: Vec::new(),
            t_activate: 0.0,
            policies: Vec::new(),
            t_shake: -10.0,
            shake_kind: -1,
            s_stereo: 0.0,
            s_stereo_target: 10.0,
            f_stereo: false,
            hor_plus_y_scale: 1.0,
        };
        FUN_0014d6d8(&mut cm, CmClock::default());
        cm
    }

    pub fn pos(&self) -> V3 {
        v3(self.mat[3])
    }
}

/// 0x0014d6d8 InitCm: defaults, look from (0,-2000,500) at the origin, push the base policy (cpp 0).
pub fn FUN_0014d6d8(cm: &mut Cm, clk: CmClock) {
    // DAT_002ddf34/30: one-time default fog colour 0x80303030.
    cm.s_near_clip = 100.0; // 0x42c80000
    cm.s_fog_near = FOG_DEFAULTS[0];
    cm.s_fog_far = FOG_DEFAULTS[1];
    cm.u_fog_max = FOG_DEFAULTS[2];
    cm.r_aspect = R_ASPECT_PS2;
    cm.s_far_clip = 100000.0; // 0x47c35000
    cm.r_mrd = 1.0;
    cm.r_mrd_ratio = 1.0;
    cm.rad_fov_target = 1.0;
    cm.rad_fov = 1.0;
    cm.rgba_fog = 0x80303030;
    cm.smp_fov = [0.5, 0.0, 0.1, 10.0]; // +0x2c, +0x30, +0x34, +0x38
    cm.mat = IDENTITY;
    FUN_0014db70(cm, clk);
    FUN_0014e1a8(cm, 0.0, 0.0, [0.0, -2000.0, 500.0], [0.0, 0.0, 0.0], clk); // 0x2bd830 / 0x2bd840
    FUN_0014d8c0(cm);
    cm.acpr.clear();
    cm.policies.clear();
    // +0x3a0 base policy (C2cd528, FUN_0014fda0 init) is the only one pushed here; the other embedded
    // policies (+0x3b0 C2db858 follow, +0x450.. ) are created by their owners via `add_policy`.
    let base = add_policy(cm, Box::new(super::policy::CplcyBase));
    cm.pso_fade = 0;
    cm.fade_e0 = 0xff;
    let _ = FUN_0014e328(cm, 0, base, 0, 0, clk);
    cm.shake_kind = -1;
    cm.t_shake = -10.0;
}

/// Registers a policy object (the original embeds them in CM; their address is the CPLCY*).
pub fn add_policy(cm: &mut Cm, p: Box<dyn Cplcy>) -> usize {
    cm.policies.push(Some(p));
    cm.policies.len() - 1
}

/// 0x0014d650 ResetCm: reset every active policy (vtable slot 6), stereo separation 0 -> target 10.
pub fn FUN_0014d650(cm: &mut Cm) {
    let ids: Vec<usize> = cm.acpr.iter().map(|c| c.pcplcy).collect();
    for id in ids {
        with_policy(cm, id, |p, cm| p.reset(cm));
    }
    cm.s_stereo = 0.0;
    cm.s_stereo_target = 10.0;
}

/// 0x0014d8c0 SetCmCut(level >= 1).
pub fn FUN_0014d8c0(cm: &mut Cm) {
    cm.cut = cm.cut.max(1);
}

/// 0x0014d8d8: SetCmCut with parameters (level >= 2).
pub fn FUN_0014d8d8(cm: &mut Cm, p0: u32, p1: u32) {
    cm.cut_p1 = p1;
    cm.cut_p0 = p0;
    cm.cut = cm.cut.max(2);
}

/// 0x0014d8f8 CombineEyeLookAtProj: world->clip = proj * inverse(eye basis
/// [-left, -up, forward, pos]).
pub fn FUN_0014d8f8(mat: &M4, proj: &M4) -> M4 {
    let mut t = IDENTITY;
    for k in 0..3 {
        t[0][k] = -mat[1][k];
        t[1][k] = -mat[2][k];
        t[2][k] = mat[0][k];
        t[3][k] = mat[3][k];
    }
    let inv = FUN_0011c318(&t);
    FUN_0011c168(proj, &inv)
}

/// 0x0014da08 BuildFrustrum: outward unit normals of the left, right, top and bottom planes through the eye.
/// ORIGINAL CULL CODE (documented for Hor+): planes are built from x_scale (+0x40 = tan(fov/2)*aspect)
/// and y_scale (+0x44); objects are tested by FUN_0014f850 against all four. Hor+ only changes
/// x_scale (aspect) so culling widens with the real horizontal FOV automatically.
pub fn FUN_0014da08(x: f32, y: f32, mat: &M4) -> [V4; 4] {
    let (f, l, u) = (v3(mat[0]), v3(mat[1]), v3(mat[2]));
    let nrm = |v: V3| scale(v, 1.0 / len(v));
    let a = nrm(add(f, scale(l, x)));
    let n0 = cross(u, a);
    let a = nrm(add(f, scale(l, -x)));
    let n1 = cross(a, u);
    let a = nrm(add(f, scale(u, y)));
    let n2 = cross(a, l);
    let a = nrm(add(f, scale(u, -y)));
    let n3 = cross(l, a);
    [v4(n0, 0.0), v4(n1, 0.0), v4(n2, 0.0), v4(n3, 0.0)]
}

fn transpose3(p: &[V4; 4]) -> [V4; 3] {
    std::array::from_fn(|k| [p[0][k], p[1][k], p[2][k], p[3][k]])
}

/// 0x0014db70 RecalcCmFrustrum.
pub fn FUN_0014db70(cm: &mut Cm, clk: CmClock) {
    // Hor+: hor_plus_y_scale == 1.0 reproduces the original exactly.
    let y = (cm.rad_fov * 0.5).tan() * cm.hor_plus_y_scale;
    let a = cm.r_aspect;
    cm.y_scale = y;
    cm.x_scale = y * a;
    // original: conditional move = max(ratio/fov, 1.0)
    cm.r_mrd = (cm.r_mrd_ratio * (1.0 / cm.rad_fov)).max(1.0);
    cm.s_near_radius = y * (a * a + 1.0).sqrt() * cm.s_near_clip + 1.0;
    cm.mat_proj = FUN_0011c448(512.0 / (y * a * 4096.0), 224.0 / (y * 4096.0), 0.0, 0.0, cm.s_near_clip, cm.s_far_clip);
    // FUN_0014e270(1.0, cm, cm+0x250): GS fog registers -> WP-C (renderer).
    FUN_0014dc58(cm, clk);
}

/// 0x0014dc58 UpdateCmMat4: world->clip (with the 0.5 s roll shake), clip->world, frustum planes.
pub fn FUN_0014dc58(cm: &mut Cm, clk: CmClock) {
    let ds = clk.t - cm.t_shake;
    if SHAKE_DURATION < ds {
        cm.mat_world_to_clip = FUN_0014d8f8(&cm.mat, &cm.mat_proj);
    } else {
        let u = ds / SHAKE_DURATION;
        let env = (SHAKE_ENV[0] + u * (SHAKE_ENV[1] + u * SHAKE_ENV[2])).clamp(0.0, 1.0);
        let s = (ds * SHAKE_FREQ).sin() * env;
        let mut shake = FUN_0011b570(scale(SHAKE_ROT, s));
        shake[0][3] = 0.0;
        shake[1][3] = 0.0;
        shake[2][3] = 0.0;
        shake[3] = v4(scale(SHAKE_TRANS, s), 1.0);
        let m = FUN_0011c168(&cm.mat, &shake);
        let sc = add([1.0, 1.0, 1.0], scale(SHAKE_PROJ_SCALE, s)); // 0x2a2de0 + 0x2bd890 * s
        let c = scale(SHAKE_PROJ_CENTER, s);
        let proj = FUN_0011c448(
            sc[0] * 512.0 / (cm.x_scale * 4096.0),
            sc[1] * 224.0 / (cm.y_scale * 4096.0),
            c[0],
            c[1],
            cm.s_near_clip,
            cm.s_far_clip,
        );
        cm.mat_world_to_clip = FUN_0014d8f8(&m, &proj);
    }
    cm.mat_clip_to_world = FUN_0011c2f0(&cm.mat_world_to_clip);
    cm.anormal_frustrum = FUN_0014da08(cm.x_scale, cm.y_scale, &cm.mat);
    cm.anormal_frustrum_transpose = transpose3(&cm.anormal_frustrum);
    cm.anormal_frustrum_wide = FUN_0014da08(cm.x_scale * 7.93, cm.y_scale * 18.1, &cm.mat);
    cm.anormal_frustrum_wide_transpose = transpose3(&cm.anormal_frustrum_wide);
    // FUN_0014e078: notifies every class-0x105 object (FUN_0015e9c8 world query) of the camera position
    // and may set the fade target (FUN_0014e148) -> world-side hook, done by WP-B/J.
}

/// 0x0014e148: set the fade-target object.
pub fn FUN_0014e148(cm: &mut Cm, pso: u32, pso_e0: u32) {
    if pso != cm.pso_fade {
        cm.pso_fade = pso;
        cm.fade_e0 = pso_e0;
    }
}

/// 0x0014e168 SetCmPosMat: set the full 4x4 (orientation + position) and rebuild.
pub fn FUN_0014e168(cm: &mut Cm, m: &M4, clk: CmClock) {
    cm.mat = *m;
    FUN_0014dc58(cm, clk);
}

/// 0x0014f428 SetCmMat: orientation from a 3x3 (rows of 4 floats), keep the position.
pub fn FUN_0014f428(cm: &mut Cm, rot: &[V4; 3], clk: CmClock) {
    let mut m = [v4(v3(rot[0]), 0.0), v4(v3(rot[1]), 0.0), v4(v3(rot[2]), 0.0), cm.mat[3]];
    m[3][3] = cm.mat[3][3];
    FUN_0014e168(cm, &m, clk);
}

/// 0x0014e1a8 SetCmLookAt: aim from `pos` at `target`, with yaw/pitch offsets.
pub fn FUN_0014e1a8(cm: &mut Cm, dyaw: f32, dpitch: f32, pos: V3, target: V3, clk: CmClock) {
    let (yaw, pitch, _) = FUN_0011efb0(sub(target, pos));
    let r = FUN_0011b720(yaw + dyaw, pitch + dpitch);
    cm.mat[0] = v4(v3(r[0]), 0.0);
    cm.mat[1] = v4(v3(r[1]), 0.0);
    cm.mat[2] = v4(v3(r[2]), 0.0);
    cm.mat[3] = v4(pos, 1.0);
    FUN_0014dc58(cm, clk);
}

/// 0x0014e328 SetCmPolicy: insert (cpp, policy, pso, pv) keeping the stack sorted by cpp (new entries go
/// below existing ones of equal priority). Max 8 entries (silently ignored beyond).
pub fn FUN_0014e328(cm: &mut Cm, cpp: i32, pcplcy: usize, pso: u32, pv: u32, clk: CmClock) -> bool {
    if cm.acpr.len() >= CM_MAX_POLICIES {
        return false;
    }
    let mut i = 0;
    if !cm.acpr.is_empty() && cm.acpr[0].cpp >= cpp {
        i = 1;
        while i < cm.acpr.len() && cpp <= cm.acpr[i].cpp {
            i += 1;
        }
    }
    cm.acpr.insert(i, Cpr { cpp, pcplcy, pso, pv });
    with_policy(cm, pcplcy, |p, cm| p.push(cm, pv)); // slot 3
    if i == 0 {
        if cm.acpr.len() > 1 {
            let prev = cm.acpr[1];
            with_policy(cm, prev.pcplcy, |p, cm| p.deactivate(cm, prev.pso, prev.pv, clk)); // slot 2
        }
        cm.t_activate = clk.t;
        with_policy(cm, pcplcy, |p, cm| p.activate(cm, pso, pv, clk)); // slot 1
    }
    true
}

/// 0x0014e4c8 RevokeCmPolicy: remove every entry matching the fields selected by `grf`
/// (1 cpp, 2 policy, 4 pso, 8 pv); if the active one was removed, deactivate it and activate the new top.
pub fn FUN_0014e4c8(cm: &mut Cm, grf: u32, cpp: i32, pcplcy: usize, pso: u32, pv: u32, clk: CmClock) {
    let old = std::mem::take(&mut cm.acpr);
    let mut removed_first: Option<Cpr> = None;
    let mut first = true;
    for (k, c) in old.iter().enumerate() {
        let keep = (grf & 1 != 0 && cpp != c.cpp)
            || (grf & 2 != 0 && pcplcy != c.pcplcy)
            || (grf & 4 != 0 && pso != c.pso)
            || (grf & 8 != 0 && pv != c.pv);
        if keep {
            cm.acpr.push(*c);
        } else {
            // quirk kept: "first" is the first *iteration* (k == 0), i.e. only the active entry counts.
            if k == 0 && first {
                removed_first = Some(*c);
            }
            with_policy(cm, c.pcplcy, |p, cm| p.revoke(cm, c.pv)); // slot 4
        }
        first = false;
    }
    if let Some(c) = removed_first {
        with_policy(cm, c.pcplcy, |p, cm| p.deactivate(cm, c.pso, c.pv, clk));
        if let Some(top) = cm.acpr.first().copied() {
            cm.t_activate = clk.t;
            with_policy(cm, top.pcplcy, |p, cm| p.activate(cm, top.pso, top.pv, clk));
        }
    }
}

/// Calls a policy with mutable access to the camera (the original passes `this` + global g_cm).
pub fn with_policy<R: Default>(cm: &mut Cm, id: usize, f: impl FnOnce(&mut dyn Cplcy, &mut Cm) -> R) -> R {
    let Some(mut p) = cm.policies.get_mut(id).and_then(|p| p.take()) else { return R::default() };
    let r = f(p.as_mut(), cm);
    cm.policies[id] = Some(p);
    r
}

/// 0x0014e698 CalcCmAdjust: cylindrical offset (radHome + pi, sAdjust, 0).
pub fn FUN_0014e698(cpdefi: &Cpdefi) -> V3 {
    FUN_0011e1a0(cpdefi.rad_home + PI, cpdefi.s_adjust, 0.0)
}

/// 0x0014e6d0 SpringCm: damped spring (stiffness k, damping d) of pos/vel towards target; snaps on cut.
pub fn FUN_0014e6d0(cm_cut: i32, k: f32, d: f32, pos: &mut V3, vel: &mut V3, target: V3, dt: f32) {
    if cm_cut == 0 {
        let acc = add(scale(sub(target, *pos), k), scale(*vel, d));
        *vel = add(*vel, scale(acc, dt));
        *pos = add(*pos, scale(*vel, dt));
    } else {
        *pos = target;
        *vel = [0.0; 3];
    }
}

/// 0x0014e768 AdjustCmCpdefi: spring the adjust offset; stiffness depends on the JT (Sly) state.
pub fn FUN_0014e768(cm: &mut Cm, jt_state: Option<i32>, clk: CmClock) {
    let target = FUN_0014e698(&cm.cpdefi);
    let cut = cm.cut;
    let dt = clk.dt;
    // FEEL R25: the +0x2e0 spring call passes no FUN_0014e698 target (Ghidra drops arg 6): it springs to 0 --
    // every rec5 EE state has cm +0x2e0 / +0x2f0 = 0 while +0x2c0 = the 100-unit adjust. A/B SLY2_FEEL_ADJ1OLD=1.
    let t1 = if std::env::var_os("SLY2_FEEL_ADJ1OLD").is_some() { target } else { [0.0; 3] };
    let (mut p0, mut v0, mut p1, mut v1) = (cm.pos_adjust0, cm.v_adjust0, cm.pos_adjust1, cm.v_adjust1);
    match jt_state {
        None => {
            FUN_0014e6d0(cut, 5.0, -3.5, &mut p0, &mut v0, target, dt);
            FUN_0014e6d0(cut, 5.0, -3.5, &mut p1, &mut v1, t1, dt);
        }
        Some(0) => {
            FUN_0014e6d0(cut, 5.0, -3.5, &mut p0, &mut v0, target, dt);
            FUN_0014e6d0(cut, 5.0, -3.5, &mut p1, &mut v1, t1, dt);
        }
        Some(1) => {
            FUN_0014e6d0(cut, 1.0, -10.0, &mut p0, &mut v0, target, dt);
            FUN_0014e6d0(cut, 0.0, -10.0, &mut p1, &mut v1, t1, dt);
        }
        Some(2) => FUN_0014e6d0(cut, 5.0, -3.5, &mut p0, &mut v0, target, dt),
        Some(_) => {}
    }
    (cm.pos_adjust0, cm.v_adjust0, cm.pos_adjust1, cm.v_adjust1) = (p0, v0, p1, v1);
    cm.cpdefi.pos_adjust = v4(add(v3(cm.cpdefi.pos_base), p0), cm.cpdefi.pos_base[3]);
}

/// 0x0014e920 ResetCmAdjust.
pub fn FUN_0014e920(cm: &mut Cm) {
    cm.pos_adjust0 = FUN_0014e698(&cm.cpdefi);
    cm.v_adjust0 = [0.0; 3];
    cm.v_adjust1 = [0.0; 3];
}

/// 0x0014e968 GetCmCpdefi: cached CPDEFI if `pso` is the cached focus, else a fresh one with the
/// (unsprung) adjust applied.
pub fn FUN_0014e968(cm: &Cm, pso: u32, world: &mut dyn CmWorld) -> Cpdefi {
    if pso == cm.pso_cpdefi {
        cm.cpdefi
    } else {
        let mut c = world.get_cpdefi(pso);
        let a = FUN_0014e698(&c);
        c.pos_adjust = v4(add(v3(c.pos_base), a), c.pos_base[3]);
        c
    }
}

/// 0x0014d478 UpdateCm: run the active policy (re-running if it changed the stack), then the fov
/// approach (or snap on cut) and the stereo separation approach.
pub fn FUN_0014d478(cm: &mut Cm, world: &mut dyn CmWorld, pad: &CplcyCtx, clk: CmClock) {
    let mut cut;
    if cm.acpr.is_empty() {
        cut = cm.cut;
    } else {
        let mut cur_pso = 0u32;
        let mut prev_policy: Option<usize> = None;
        loop {
            let top = cm.acpr[0];
            let has_cpdefi = if top.pso == 0 {
                cm.pso_cpdefi = 0;
                cur_pso = 0;
                false
            } else {
                if top.pso != cur_pso {
                    cm.cpdefi = world.get_cpdefi(top.pso);
                    if prev_policy.is_some() || top.pso != cm.pso_cpdefi {
                        FUN_0014e920(cm);
                    }
                    let js = world.jt_state(top.pso);
                    FUN_0014e768(cm, js, clk);
                    cm.pso_cpdefi = top.pso;
                    cur_pso = top.pso;
                }
                true
            };
            with_policy(cm, top.pcplcy, |p, cm| p.update(cm, has_cpdefi, pad, clk)); // slot 5
            if cm.acpr.is_empty() {
                break;
            }
            if cm.acpr[0].pcplcy == top.pcplcy {
                break;
            }
            prev_policy = Some(top.pcplcy);
        }
        cut = cm.cut;
    }
    if cut == 0 {
        if cm.rad_fov_target != cm.rad_fov {
            let (f, v) = FUN_0011d1c0(cm.rad_fov, cm.drad_fov, cm.rad_fov_target, clk.dt, &cm.smp_fov);
            cm.rad_fov = f;
            cm.drad_fov = v;
            FUN_0014db70(cm, clk);
        }
    } else {
        FUN_0014f5a0(cm, cm.rad_fov_target, clk);
    }
    cut = 0;
    cm.cut = cut;
    if cm.f_stereo {
        cm.s_stereo = FUN_0011d018(cm.s_stereo, cm.s_stereo_target, clk.dt_real, &SMP_STEREO).0;
    }
}

/// 0x0014f268 ConvertCmScreenToWorld: (sx, sy) in [-1,1] (y up), depth d -> world point.
pub fn FUN_0014f268(cm: &Cm, s: V3) -> V3 {
    let d = s[2];
    let (mut y, mut z) = (-s[0], s[1]);
    if 0.0001 < d {
        y *= d;
        z *= d;
    }
    let local = [d, y * cm.x_scale, z * cm.y_scale];
    xform_point(&cm.mat, local)
}

/// 0x0014f308 ConvertCmWorldToScreen: world point -> (x, y) in [-1,1] (x right, y up) and depth.
pub fn FUN_0014f308(cm: &Cm, p: V3) -> V3 {
    let inv = FUN_0011c318(&cm.mat);
    let l = xform_point(&inv, p);
    let mut y = l[2] / cm.y_scale;
    let mut x = -l[1] / cm.x_scale;
    if 0.0001 < l[0] {
        y /= l[0];
        x /= l[0];
    }
    [x, y, l[0]]
}

/// 0x0014f478: camera orientation as integer degrees (x, y, z) (debug display).
pub fn FUN_0014f478(cm: &Cm) -> [i32; 3] {
    let e = FUN_0011bff0(&cm.mat);
    [(e[0] * 57.295776) as i32, (e[1] * 57.295776) as i32, (e[2] * 57.295776) as i32]
}

/// 0x0014f5a0 SetCmFov: set fov and target, zero the velocity; recalc only if it changed by > 1e-4.
pub fn FUN_0014f5a0(cm: &mut Cm, fov: f32, clk: CmClock) {
    let old = cm.rad_fov;
    cm.rad_fov_target = fov;
    cm.rad_fov = fov;
    cm.drad_fov = 0.0;
    if 0.0001 < (fov - old).abs() {
        FUN_0014db70(cm, clk);
    }
}

/// 0x0014f610 SetCmNearClip.
pub fn FUN_0014f610(cm: &mut Cm, s: f32, clk: CmClock) {
    cm.s_near_clip = s;
    FUN_0014db70(cm, clk);
}

/// 0x0014f630: SetCmNearClip and store it into the active policy's saved near clip (+8).
pub fn FUN_0014f630(cm: &mut Cm, s: f32, clk: CmClock) {
    FUN_0014f610(cm, s, clk);
    if let Some(top) = cm.acpr.first().copied() {
        with_policy(cm, top.pcplcy, |p, _| p.set_saved_near(s));
    }
}

/// 0x0014f670 SetCmFarClip.
pub fn FUN_0014f670(cm: &mut Cm, s: f32, clk: CmClock) {
    cm.s_far_clip = s;
    FUN_0014db70(cm, clk);
}

/// 0x0014f690 / 0x0014f6b0 / 0x0014f6d0: fog near / far / max.
pub fn FUN_0014f690(cm: &mut Cm, s: f32, clk: CmClock) {
    cm.s_fog_near = s;
    FUN_0014db70(cm, clk);
}
pub fn FUN_0014f6b0(cm: &mut Cm, s: f32, clk: CmClock) {
    cm.s_fog_far = s;
    FUN_0014db70(cm, clk);
}
pub fn FUN_0014f6d0(cm: &mut Cm, u: f32, clk: CmClock) {
    cm.u_fog_max = u;
    FUN_0014db70(cm, clk);
}

/// 0x0014f6f0 SetCmRgbaFog (FUN_001ed278 converts the colour for the GS -> WP-C).
pub fn FUN_0014f6f0(cm: &mut Cm, rgba: u32, clk: CmClock) {
    cm.rgba_fog = rgba;
    FUN_0014db70(cm, clk);
}

/// 0x0014f730 SetCmMrdRatio.
pub fn FUN_0014f730(cm: &mut Cm, r: f32, clk: CmClock) {
    cm.r_mrd_ratio = r;
    FUN_0014db70(cm, clk);
}

/// 0x0014f750 StartCmShake.
pub fn FUN_0014f750(cm: &mut Cm, kind: i32, clk: CmClock) {
    cm.shake_kind = kind;
    cm.t_shake = clk.t;
}

/// 0x0014f768: per-eye world->clip (eye 0 centre, 1 = +sep along left, 2 = -sep), optional projection.
pub fn FUN_0014f768(cm: &Cm, eye: i32, proj: Option<&M4>) -> (M4, M4) {
    let sep = match eye {
        1 => cm.s_stereo,
        2 => -cm.s_stereo,
        _ => 0.0,
    };
    let mut m = cm.mat;
    m[3] = v4(add(v3(cm.mat[3]), scale(v3(cm.mat[1]), sep)), 1.0);
    m[0][3] = 0.0;
    m[1][3] = 0.0;
    m[2][3] = 0.0;
    let p = proj.copied().unwrap_or(cm.mat_proj);
    (FUN_0014d8f8(&m, &p), m)
}

/// 0x0014f850 FInsideCmFrustrum: sphere (centre relative to the camera position, radius) is at least
/// partly inside all four planes (VU MAC sign flags == 0xf0: every n.c - r < 0).
pub fn FUN_0014f850(cm: &Cm, c: V3, r: f32) -> bool {
    let t = &cm.anormal_frustrum_transpose;
    (0..4).all(|k| t[0][k] * c[0] + t[1][k] * c[1] + t[2][k] * c[2] - r < 0.0)
}

/// 0x0014f898 FInsideCmMrd: within the max render distance `s * r_mrd` of the camera; returns the fade
/// factor (1 inside, ramps to 0 over the last 400 * r_mrd units).
pub fn FUN_0014f898(cm: &Cm, s: f32, d: V3) -> Option<f32> {
    let dd = dot(d, d);
    let s = s * cm.r_mrd;
    let s2 = s * s;
    if s2 < dd {
        return None;
    }
    let inner = s - cm.r_mrd * 400.0;
    let inner2 = inner * inner;
    Some(if inner2 < dd { (s2 - dd) / (s2 - inner2) } else { 1.0 })
}

/// 0x00125168 (Splice BIF 128 `RefOpNearClipCenter`): world point at the centre of the near plane.
pub fn near_clip_center(cm: &Cm) -> V3 {
    xform_point(&cm.mat, [cm.s_near_clip, 0.0, 0.0])
}

/// 0x001cd780 (label projection): world point -> 640 x 492.8 PS2 screen pixels, None if nearer than the
/// near clip. The HUD then maps those through `hud::HudLayout`.
pub fn screen_px_ps2(cm: &Cm, p: V3) -> Option<(f32, f32)> {
    let s = FUN_0014f308(cm, p);
    if cm.s_near_clip <= s[2] {
        Some(((s[0] * 0.5 + 0.5) * 640.0, (0.5 - s[1] * 0.5) * 492.80002))
    } else {
        None
    }
}
