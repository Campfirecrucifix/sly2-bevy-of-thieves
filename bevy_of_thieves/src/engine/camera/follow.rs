//! C2db858: the gameplay follow camera (vtable 0x002db858: v0 ctor 0x002219c8, v1 activate 0x00221a18,
//! v2 deactivate 0x00221cc0, v5 update 0x00221dd8; v3/v4/v6 inherited no-ops).
//!
//! Ported 1:1 for the free-follow case (`pv` == 0: no camera track object, params = DAT_002c4120, the
//! normal on-foot camera): auto-follow yaw from the focus velocity, pitch return, right-stick manual
//! orbit, distance/height springs, occlusion-cut check (0x002236e8), collision move (0x0014f110),
//! screen-offset framing and first-frame angle-offset continuity.
//! WP-P24: a class-15 track (`pv` != 0, `FollowWorld::track`) without a path (+0x10c == 0, +300 == 0) and
//! with mode bits +0x164 & 7 == 4 (+0x168 / +0x16c / +0x190 == 0) is ported: its +0x114 params entry (live,
//! like the original's pointer), the +0x188 fov target and the +0x184 cut (the glider camera 0x5f4).
//! FEEL R23: the activate-time focus FUN_00220dd8(.., track) is ported (no-path case). Not ported: the
//! entry selection 0x0015e988 over several +0x114 entries and the camera-track (path +0x10c / +300,
//! +0x164 mode bits 1|2) helpers 0x00221408/0x002215c0/0x00221ce8/0x001e6f60/0x001e7380 and the
//! (needs world objects that are not loaded yet; see port_tracking). WP-P17: the cut placement search
//! 0x00223a70 is ported (`FUN_00223a70`, push-out sweep via `FollowWorld::push_out`).
//! Sounds (0x001fbb90/0x001fb2f8, swing sound 3) go to `FollowWorld::emitters`, the activity message
//! 0x001619a0(0x2deb30, 6, 0) to `FollowWorld::activity`.
#![allow(non_snake_case)]

use super::cm::*;
use super::collide::{CmSegmentQuery, FUN_0014ea08};
use super::math::*;
use super::policy::{Cplcy, CplcyCtx, CplcySave, FUN_0014fcc8, FUN_0014fce8};
use super::follow_track::{self, TrackCtx};
use crate::engine::classes::q::q2mem::QMem;

/// Follow parameters (0x90 bytes; default instance DAT_002c4120, a track supplies its own).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FollowParams {
    /// +0x00
    pub grf: u32,
    /// +0x04: distance (negative = behind; |x| is the follow distance).
    pub s_dist: f32,
    /// +0x10, +0x20, +0x30: offsets used for the default pitch (FUN_00220d40).
    pub v10: V4,
    pub v20: V4,
    pub v30: V4,
    /// +0x40..+0x54: limits.
    pub lim: [f32; 6],
    /// +0x58: horizontal screen offset (atan(x * x_scale) yaw bias).
    pub x_offset: f32,
    /// +0x5c: vertical screen offset (atan(y * y_scale) pitch bias).
    pub y_offset: f32,
    /// +0x60: blend of the focus between CPDEFI pos_base (0) and pos_adjust (1).
    pub u_adjust: f32,
    /// +0x64: auto-follow gain target.
    pub g_follow: f32,
    /// +0x68: track blend gain.
    pub g_track: f32,
    pub unk_6c: f32,
    pub v70: V4,
    /// +0x80/+0x84: spring stiffness / damping scales.
    pub k_spring: f32,
    pub d_spring: f32,
    /// +0x88/+0x8c: pitch range (sentinel -FLT_MAX/FLT_MAX = computed by FUN_00220d40).
    pub pitch_range: [f32; 2],
}

/// WP-P24: class-15 camera track (C2db6b8) fields read by C2db858 (pv = object handle).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FollowTrack {
    /// +0x114 current params entry (FUN_00221818: DAT_002c4120 copy, grf 0x30, then the 0x2000 fields)
    pub params: FollowParams,
    /// +0x164 mode bits (field 197, default 7; bit 1 cleared without +0x10c)
    pub grf: u32,
    /// +0x168 / +0x16c / +0x190 / +0x184 (no load fields seen; 0 on the ported tracks)
    pub f_168: i32,
    pub f_16c: i32,
    pub f_190: i32,
    pub f_184: i32,
    /// +0x188 fov target (field 208; 0 = keep)
    pub fov: f32,
    /// +0x10c path / +300 ring present (unported track kinds)
    pub path: bool,
    /// +0x110 params entries (> 1: 0x0015e988 selection, unported)
    pub n_params: i32,
    /// +0x128 (field 200): FUN_00221178 pushes the follow at priority +0x128 + 2
    pub f_128: i32,
}

impl FollowTrack {
    /// Ported subset (see the module doc).
    pub fn simple(&self) -> bool {
        !self.path && self.grf & 7 == 4 && self.f_168 == 0 && self.f_16c == 0 && self.f_190 == 0
    }
}

/// DAT_002c4120 (read from the ELF).
pub const FOLLOW_PARAMS_DEFAULT: FollowParams = FollowParams {
    grf: 0x30,
    s_dist: -450.0,
    v10: [0.0, 1.0, 0.0, 0.0],
    v20: [0.0, 1.0, 0.0, 0.0],
    v30: [125.0, 1.0, 0.0, 0.0],
    lim: [-f32::MAX, f32::MAX, -f32::MAX, f32::MAX, -f32::MAX, f32::MAX],
    x_offset: 0.0,
    y_offset: -0.1,
    u_adjust: 1.0,
    g_follow: 1.0,
    g_track: 1.0,
    unk_6c: 0.0,
    v70: [1.0, 1.0, 1.0, 0.0],
    k_spring: 1.0,
    d_spring: 1.0,
    pitch_range: [-f32::MAX, f32::MAX],
};

impl FollowParams {
    /// WP-P24: the 0x90-byte entry as words (+0x08 / +0x0c padding read as 0).
    pub fn to_words(&self) -> [u32; 36] {
        let mut w = [0u32; 36];
        let f = |x: f32| x.to_bits();
        w[0] = self.grf;
        w[1] = f(self.s_dist);
        for i in 0..4 {
            w[4 + i] = f(self.v10[i]);
            w[8 + i] = f(self.v20[i]);
            w[12 + i] = f(self.v30[i]);
            w[28 + i] = f(self.v70[i]);
        }
        for i in 0..6 {
            w[16 + i] = f(self.lim[i]);
        }
        let t = [self.x_offset, self.y_offset, self.u_adjust, self.g_follow, self.g_track, self.unk_6c];
        for i in 0..6 {
            w[22 + i] = f(t[i]);
        }
        w[32] = f(self.k_spring);
        w[33] = f(self.d_spring);
        w[34] = f(self.pitch_range[0]);
        w[35] = f(self.pitch_range[1]);
        w
    }

    pub fn from_words(w: &[u32; 36]) -> Self {
        let f = |i: usize| f32::from_bits(w[i]);
        let v = |i: usize| [f(i), f(i + 1), f(i + 2), f(i + 3)];
        Self {
            grf: w[0],
            s_dist: f(1),
            v10: v(4),
            v20: v(8),
            v30: v(12),
            lim: [f(16), f(17), f(18), f(19), f(20), f(21)],
            x_offset: f(22),
            y_offset: f(23),
            u_adjust: f(24),
            g_follow: f(25),
            g_track: f(26),
            unk_6c: f(27),
            v70: v(28),
            k_spring: f(32),
            d_spring: f(33),
            pitch_range: [f(34), f(35)],
        }
    }

    /// Field +0x10.x (+0x10), +0x20.x (+0x20), +0x30.x (+0x30) as read by FUN_00220d40.
    fn f10(&self) -> f32 {
        self.v10[0]
    }
}

/// 0x002c41b0: pitch clamp range.
pub const PITCH_RANGE: [f32; 2] = [-0.5, 1.5550884];
/// 0x002c4110 / 0x002c4114: occlusion check period and cut delay; 0x002c4118 = 0 (check enabled).
pub const OCCLUDE_RECHECK: f32 = 0.5;
pub const OCCLUDE_CUT: f32 = 2.0;
/// 0x002c4290: occlusion probe offsets (x along up x right, y along right, z along up).
pub const OCCLUDE_PROBES: [V3; 5] = [[0.0, 0.0, 0.0], [0.0, -50.0, 0.0], [0.0, 50.0, 0.0], [0.0, 0.0, 75.0], [0.0, 0.0, -50.0]];
/// 0x002c41d8 / 0x002c41dc
pub const T_MOVE_GRACE: f32 = 0.75;
pub const T_MANUAL_GRACE: f32 = 10.0;
/// 0x002c41e0: max yaw-rate factor; 0x002c41e4/e8: stick rate scales; 0x002c41ec: rate slew.
pub const FOLLOW_RATE_MAX: f32 = 2.5;
pub const STICK_YAW_RATE: f32 = 2.5;
pub const STICK_PITCH_RATE: f32 = 1.0;
pub const STICK_SLEW: f32 = 15.0;
/// 0x002c41f0..: left-stick y boost polynomial and clamp; 0x002c4220..: manual-grace ramp.
pub const STICK_BOOST: [f32; 3] = [1.0, -2.0, -2.0];
pub const STICK_BOOST_RANGE: [f32; 2] = [1.0, 2.0];
pub const MANUAL_RAMP: [f32; 3] = [0.0, 1.0, 0.0];
/// 0x002bbb60: [0, 1].
pub const UNIT_RANGE: [f32; 2] = [0.0, 1.0];
/// 0x002c4208: track-yaw approach profile (scaled), 0x002c4230: follow-gain profile,
/// 0x002c4240/0x002c4250: reset profiles (normal / JT +0x10e4 == 4), 0x002c4260: pitch return.
pub const SMP_TRACK: Smp = [0.5, 0.0, 0.5, 0.0];
pub const SMP_GAIN: Smp = [4.0, 0.0, 0.25, 0.0];
pub const SMP_RESET: Smp = [5.0, 0.0, 0.25, 35.0];
pub const SMP_RESET_FAST: Smp = [10.0, 0.0, 0.1, 100.0];
pub const SMP_PITCH: Smp = [0.75, 0.0, 2.0, 0.0];
/// 0x002c426c/0x002c4270: distance spring damping/stiffness; 0x002c4274..: angle-offset spring.
pub const SPRING_D: f32 = -8.0;
pub const SPRING_K: f32 = 15.0;
pub const OFFSET_K: f32 = -10.0;
pub const OFFSET_D: f32 = -5.0;
/// 0x002bd8d8/0x002bd8dc: collision sphere radius / extra move; 0x002bd8f0/0x002bd900: query box.
pub const COLLIDE_RADIUS: f32 = 125.0;
pub const COLLIDE_EXTRA: f32 = 1500.0;
pub const COLLIDE_BOX: f32 = 250.0;

/// JT (Sly) fields the follow camera reads.
#[derive(Clone, Copy, Debug, Default)]
pub struct FollowJt {
    /// +0x4f4
    pub state: i32,
    /// +0x500
    pub pos_500: V3,
    /// +0x10e4
    pub f_10e4: i32,
    /// FEEL R25: +0x4c8 / +0x4cc (attach kind / substate, -1 unattached) and +0x528 (track_blend's own-track
    /// test), and the ALO row 0 xy (0x001a6318, heading_weight) for a focus with no `pso_addr`
    pub a_4c8: u32,
    pub a_4cc: u32,
    pub t_528: u32,
    pub fwd: [f32; 2],
}

/// World access of the follow policy (replaces vtable/global calls).
pub trait FollowWorld: CmWorld + Send + Sync {
    /// FUN_00132b10(pso, 0x34): the focus is a JT.
    fn jt(&mut self, _pso: u32) -> Option<FollowJt> {
        None
    }
    /// FUN_001a62e8: world position of an SO.
    fn so_pos(&mut self, _pso: u32) -> V3 {
        [0.0; 3]
    }
    /// Geometry near the camera (FUN_0020faa0 gather + FUN_00118668 clip, WP-E).
    fn segments(&mut self) -> Option<&mut dyn CmSegmentQuery> {
        None
    }
    /// 0x0014eda0: sphere (radius) moving with `vel` from `start`; returns the corrected velocity, length
    /// limited to `max_len` (contact LCP 0x0011ad30 over FUN_001a93c8 contacts, physics = WP-E).
    fn sweep_sphere(&mut self, _radius: f32, _max_len: f32, _start: V3, vel: V3) -> V3 {
        vel
    }
    /// WP-P17: 0x0014eda0(Cm +0x48, Cm +0x48, Cm, objects, callback 0x00223a10, centre, centre) of the
    /// cut placement search: sphere at `centre`, b_i = n_i.centre - plane_i.w (0x00223a10), returns centre +
    /// the clamped push-out. Objects = the 0x0020faa0 gather of the box `lo`..`hi`.
    fn push_out(&mut self, _radius: f32, _lo: V3, _hi: V3, centre: V3) -> V3 {
        centre
    }
    /// The sound pool 0x3dd05c (None: the swing sound is skipped).
    fn emitters(&mut self) -> Option<&mut crate::engine::audio::emitter::EmitterSystem> {
        None
    }
    /// DAT_002ddee4 (clock read by the manual-camera activity message).
    fn t_2ddee4(&mut self) -> f32 {
        0.0
    }
    /// 0x001619a0(0x2deb30, kind, arg): activity message (hint/tutorial tracker; 6 = manual camera).
    fn activity(&mut self, _kind: i32, _arg: i32) {}
    /// WP-P24: the class-15 track object `pv` (None: unknown / not loaded).
    fn track(&mut self, _pv: u32) -> Option<FollowTrack> {
        None
    }
    /// WP-P24: write back the track's +0x114 params entry (FUN_00220d40 works on it in place).
    fn set_track_params(&mut self, _pv: u32, _p: &FollowParams) {}
    /// PART-3: EE object memory holding the track / focus objects (track modes 1|2 of 0x00221dd8; calls
    /// route through `QMem::call`). None: the struct-side track subset only.
    fn mem(&mut self) -> Option<&mut dyn QMem> {
        None
    }
    /// FEEL R24: `mem` address of track handle `pv` (None: not in memory). Default: pv is the address.
    fn track_addr(&mut self, pv: u32) -> Option<u32> {
        Some(pv)
    }
    /// FEEL R24: `mem` address of the focus `pso` (None: the focus is not in memory).
    fn pso_addr(&mut self, pso: u32) -> Option<u32> {
        Some(pso)
    }
}

/// No world: no collision, focus not a JT.
pub struct NoFollowWorld;
impl CmWorld for NoFollowWorld {
    fn get_cpdefi(&mut self, _pso: u32) -> Cpdefi {
        Cpdefi::default()
    }
    fn jt_state(&mut self, _pso: u32) -> Option<i32> {
        None
    }
}
impl FollowWorld for NoFollowWorld {}

/// The policy object (fields by original offset).
pub struct CplcyFollow {
    pub save: CplcySave,
    /// +0x00 (focus SO, set by activate)
    pub pso: u32,
    /// +0x10 track (only 0 supported) / +0x14 params
    pub track: u32,
    pub params: FollowParams,
    /// +0x18 yaw, +0x1c pitch, +0x20 distance, +0x24 vertical screen offset
    pub yaw: f32,
    pub pitch: f32,
    pub dist: f32,
    pub y_off: f32,
    /// +0x28 reset in progress, +0x2c/+0x30 reset yaw/pitch targets (set by script/game code)
    pub reset: bool,
    pub reset_yaw: f32,
    pub reset_pitch: f32,
    /// +0x34 mode (1 track A, 2 track B, 4 free), +0x38 first update, +0x3c manual/track latched
    pub mode: i32,
    pub first: bool,
    pub latched: bool,
    /// +0x40/+0x44: stick yaw/pitch rates (also reset velocities)
    pub rate_yaw: f32,
    pub rate_pitch: f32,
    /// +0x48: time of the last manual input (- T_MANUAL_GRACE at activation)
    pub t_manual: f32,
    /// +0x4c track yaw, +0x50..+0x5c previous track angles
    pub track_yaw: f32,
    pub prev: [f32; 4],
    /// +0x60 latest CPDEFI move time, +0x64 follow gain
    pub t_move: f32,
    pub g_follow: f32,
    /// +0x68/+0x6c distance / offset spring velocities
    pub v_dist: f32,
    pub v_off: f32,
    /// +0x70 camera position (after collision)
    pub pos: V4,
    /// +0x80 last time the focus was visible
    pub t_visible: f32,
    /// +0x84/+0x88 angle offsets (yaw, pitch), +0x8c/+0x90 their velocities
    pub off_yaw: f32,
    pub off_pitch: f32,
    pub v_off_yaw: f32,
    pub v_off_pitch: f32,
    /// DAT_002deb34 (global 0x2deb30 +4): time of the last manual-camera activity message.
    pub t_msg: f32,
    pub world: Box<dyn FollowWorld>,
}

impl CplcyFollow {
    /// 0x002219c8 ctor (resets DAT_002c41a8/ac = params pitch range to the sentinel).
    pub fn new(world: Box<dyn FollowWorld>) -> Self {
        CplcyFollow {
            save: CplcySave::default(),
            pso: 0,
            track: 0,
            params: FOLLOW_PARAMS_DEFAULT,
            yaw: 0.0,
            pitch: 0.0,
            dist: 0.0,
            y_off: 0.0,
            reset: false,
            reset_yaw: 0.0,
            reset_pitch: 0.0,
            mode: 0,
            first: false,
            latched: false,
            rate_yaw: 0.0,
            rate_pitch: 0.0,
            t_manual: 0.0,
            track_yaw: 0.0,
            prev: [0.0; 4],
            t_move: 0.0,
            g_follow: 0.0,
            v_dist: 0.0,
            v_off: 0.0,
            pos: [0.0; 4],
            t_visible: 0.0,
            off_yaw: 0.0,
            off_pitch: 0.0,
            v_off_yaw: 0.0,
            v_off_pitch: 0.0,
            t_msg: 0.0,
            world,
        }
    }
}

/// 0x00220d40: when the pitch range is the sentinel, set both ends to the params' natural pitch
/// clamp(atan2(+0x30, |(+0x04 + +0x10, +0x20)|), PITCH_RANGE).
pub fn FUN_00220d40(p: &mut FollowParams) {
    if p.pitch_range[0] == -f32::MAX && p.pitch_range[1] == f32::MAX {
        let a = p.s_dist + p.f10();
        let x = p.v30[0].atan2((a * a + p.v20[0] * p.v20[0]).sqrt());
        let x = FUN_0011d948(x, PITCH_RANGE);
        p.pitch_range = [x, x];
    }
}

/// 0x00220dd8 (track == 0): focus target = lerp(CPDEFI +0x10, +0x40, params +0x60).
pub fn FUN_00220dd8(p: &FollowParams, c: &Cpdefi) -> V3 {
    let u = p.u_adjust;
    add(scale(v3(c.pos_base), 1.0 - u), scale(v3(c.pos_adjust), u))
}

/// 0x0014f110: move the camera from `from` to `to` as a 125-unit sphere over dt_real (DAT_002ddee0).
pub fn FUN_0014f110(world: &mut dyn FollowWorld, from: V3, to: V3, dt_real: f32) -> V3 {
    let v = scale(sub(to, from), 1.0 / dt_real);
    let v2 = world.sweep_sphere(COLLIDE_RADIUS, len(v) + COLLIDE_EXTRA, from, v);
    add(from, scale(v2, dt_real))
}

impl CplcyFollow {
    /// 0x002236e8: if none of 5 probe points around the focus has a clear line to the camera plane for
    /// OCCLUDE_CUT seconds, cut (FUN_0014d8c0). Checked at most every OCCLUDE_RECHECK seconds.
    pub fn FUN_002236e8(&mut self, cm: &mut Cm, input_locked: bool, clk: CmClock) {
        if input_locked {
            self.t_visible = clk.t;
            return;
        }
        if !(self.track == 0 && OCCLUDE_RECHECK <= clk.t - self.t_visible && cm.cut == 0) {
            return;
        }
        let up = [0.0, 0.0, 1.0];
        let so = self.world.so_pos(self.pso);
        let d = sub(so, v3(cm.mat[3]));
        let c = cross(up, d);
        let l = len(c);
        let right = if l < 0.0001 { [1.0, 0.0, 0.0] } else { scale(c, 1.0 / l) };
        let side = cross(up, right);
        let mut pts = [[0.0f32; 3]; 5];
        for (i, o) in OCCLUDE_PROBES.iter().enumerate() {
            let so = self.world.so_pos(self.pso);
            pts[i] = add(so, add(add(scale(side, o[0]), scale(right, o[1])), scale(up, o[2])));
        }
        for p in pts {
            // FUN_0014f308 returns (x, y) in v0 only; z (depth) is lost -> 0 = point on the camera plane.
            let s = FUN_0014f308(cm, p);
            let e = FUN_0014f268(cm, [s[0], s[1], 0.0]);
            let Some(q) = self.world.segments() else { break };
            if let Some((_, hi)) = FUN_0014ea08(0.0, q, p, e) {
                if 1.0 <= hi {
                    self.t_visible = clk.t;
                    break;
                }
            }
        }
        if OCCLUDE_CUT < clk.t - self.t_visible {
            FUN_0014d8c0(cm);
        }
    }
}

/// WP-P17: immediates of 0x00223a70 (probe radius of its 0x0014eb70 call, yaw step, cost bias).
pub const CUT_PROBE_RADIUS: f32 = 500.0;
pub const CUT_YAW_STEP: f32 = 0.28559935;
pub const CUT_YAW_BIAS: f32 = 0.47123894;

impl CplcyFollow {
    /// 0x00223a70 (cut placement search). Box = min/max(focus, want, SO); s = FUN_0014eb70(500, SO) (a clear
    /// point beside the focus SO); clear fraction of s -> want (FUN_0014ea08, gap >= 0). Occluded (< 1): mode
    /// (+0x34) = 4, (yaw, r, z) = FUN_0011ef08(want - focus) and yaw offsets 0, +k, -k, +2k, ... (k =
    /// 0.28559935) while (|off| + 0.4712) * 0.25 <= best cost and off < pi: candidate focus +
    /// FUN_0011e1a0(yaw + off, r, z), cost (1.25 - hi) * (|off| + 0.4712), keep lerp(s, cand, hi) of the
    /// cheapest. Then the push-out 0x0014eda0 / 0x00223a10 at that point P -> r; if P -> r is not clear
    /// (hi != 1) the result is lerp(P, r, (lo + hi) / 2) of its first gap, else r.
    pub fn FUN_00223a70(&mut self, cm: &Cm, focus: V3, want: V3) -> V3 {
        let so = self.world.so_pos(self.pso);
        let lo = std::array::from_fn(|k| focus[k].min(want[k]).min(so[k]));
        let hi_box = std::array::from_fn(|k| focus[k].max(want[k]).max(so[k]));
        let Some(q) = self.world.segments() else { return want };
        let s = super::collide::FUN_0014eb70(CUT_PROBE_RADIUS, q, so, so);
        let gap_hi = |q: &mut dyn CmSegmentQuery, a: V3, b: V3| FUN_0014ea08(0.0, q, a, b).map(|g| g.1).unwrap_or(0.0);
        let mut p = want;
        if gap_hi(q, s, want) < 1.0 {
            self.mode = 4;
            let (yaw, r, z) = FUN_0011ef08(sub(want, focus));
            let mut best = f32::MAX;
            let mut off = 0.0f32;
            loop {
                let cand = add(focus, FUN_0011e1a0(yaw + off, r, z));
                let Some(q) = self.world.segments() else { break };
                let h = gap_hi(q, s, cand);
                let cost = (1.25 - h) * (off.abs() + CUT_YAW_BIAS);
                if cost < best {
                    p = add(scale(s, 1.0 - h), scale(cand, h));
                    best = cost;
                }
                let up = off <= 0.0;
                off = -off;
                if up {
                    off += CUT_YAW_STEP;
                }
                if !((off.abs() + CUT_YAW_BIAS) * 0.25 <= best && off < 3.1415927) {
                    break;
                }
            }
        }
        let r = self.world.push_out(cm.s_near_radius, lo, hi_box, p);
        let Some(q) = self.world.segments() else { return r };
        match FUN_0014ea08(0.0, q, p, r) {
            Some((a, b)) if b != 1.0 => {
                let f = (a + b) * 0.5;
                add(scale(p, 1.0 - f), scale(r, f))
            }
            Some(_) => r,
            // no gap at all: the original reads the stale stack interval; keep r
            None => r,
        }
    }
}

impl CplcyFollow {
    /// 0x00221dd8 mode 4 with track bits 1|2: blend the auto-follow (ya, pa) towards the track point by the
    /// weight w (1 no path/ring, 3 the JT's own +0x528 track, else the heading weight) and the gain f18
    /// (focus speed (JT only, x 1/600 or 1/200) x params +0x68, raised by the +0x16c / +0x168 ramps);
    /// may latch modes 1 / 2 (+0x3c) when the camera crosses a track point.
    #[allow(clippy::too_many_arguments)]
    fn track_blend(&mut self, x: &TrackCtx, vel: V3, yaw0: f32, pitch0: f32, ya: f32, pa: f32, ta: [f32; 3], tb: [f32; 3], clk: CmClock) -> (f32, f32) {
        let jt = self.world.jt(self.pso).is_some();
        let now = clk.t;
        let mut w;
        let mut use_b = false;
        if x.path == 0 && x.ring == 0 {
            w = 1.0;
        } else {
            let pa = self.world.pso_addr(self.pso);
            let j = self.world.jt(self.pso);
            let t528 = match pa {
                Some(a) => self.world.mem().map(|m| m.r32(a + 0x528)),
                None => j.map(|j| j.t_528),
            };
            let own = jt && t528 == Some(x.t);
            if own {
                w = 3.0;
            } else if self.mode == 1 {
                w = 1.0;
            } else if self.mode == 2 {
                w = 1.0;
                use_b = true;
            } else if let Some((m, a)) = pa.and_then(|a| self.world.mem().map(|m| (m, a))) {
                let (hw, b) = follow_track::heading_weight(m, x, a);
                w = hw;
                use_b = b;
            } else if let (Some(j), Some(m)) = (j.filter(|_| pa.is_none()), self.world.mem()) {
                let (hw, b) = follow_track::heading_weight_dir(m, x, j.fwd);
                w = hw;
                use_b = b;
            } else {
                w = 1.0;
            }
        }
        let (ty, tp) = if use_b { (tb[0], tb[1]) } else { (ta[0], ta[1]) };
        let mut f18 = 0.0;
        if jt {
            let pa = self.world.pso_addr(self.pso);
            let (a, b) = match pa.and_then(|p| self.world.mem().map(|m| (m, p))) {
                Some((m, p)) => (m.r32(p + 0x4cc), m.r32(p + 0x4c8)),
                None => self.world.jt(self.pso).map_or((u32::MAX, 0), |j| (j.a_4cc, j.a_4c8)),
            };
            let k = if a == u32::MAX || b != 0 { 0.0016666667 } else { 0.005 };
            f18 = ((vel[0] * vel[0] + vel[1] * vel[1]).sqrt() * k).min(1.0);
        }
        f18 *= self.params.g_track;
        if x.i108 != 0 {
            let mut f22 = 0.0;
            let a = now - (self.t_move + T_MOVE_GRACE);
            let b = now - (self.t_manual + T_MANUAL_GRACE);
            if 0.0 < x.f178 {
                f22 = FUN_0011d948(a / x.f178, UNIT_RANGE) * FUN_0011d948(b / x.f178, UNIT_RANGE);
            } else if 0.0 < a && 0.0 < b {
                f22 = 1.0;
            }
            let f21 = x.f17c;
            if f18 * w < f22 * f21 {
                w = 1.0;
                f18 = f22 * f21;
            }
        }
        if x.i10c != 0 && !self.latched {
            let r = if 0.0 < x.f170 { FUN_0011d948((now - x.t138) / x.f170, UNIT_RANGE) } else { 1.0 };
            let f22 = r * x.f174;
            if f18 * w < f22 {
                w = 1.0;
                f18 = f22;
            }
        }
        let p = [SMP_TRACK[0] * f18, SMP_TRACK[1] * f18, SMP_TRACK[2], 0.0];
        let mut ny = FUN_0011d2d0(yaw0, FUN_0011cf60(yaw0 + FUN_0011cf60(ty - yaw0)), clk.dt, &p);
        let mut np = FUN_0011d2d0(pitch0, FUN_0011cf60(pitch0 + FUN_0011cf60(tp - pitch0)), clk.dt, &p);
        if FUN_0011cf60(ty - ya).abs() < FUN_0011cf60(ty - ny).abs() {
            ny = ya;
        }
        if FUN_0011cf60(tp - pa).abs() < FUN_0011cf60(tp - np).abs() {
            np = pa;
        }
        let iw = 1.0 - w;
        let yaw = FUN_0011cf60(yaw0 + iw * FUN_0011cf60(ya - yaw0) + w * FUN_0011cf60(ny - yaw0));
        let pitch = FUN_0011cf60(pitch0 + iw * FUN_0011cf60(pa - pitch0) + w * FUN_0011cf60(np - pitch0));
        if self.first || (x.path != 0 && x.path_e0 == 0 && 2 < x.path_e8) {
            return (yaw, pitch);
        }
        const HALF_PI: f32 = 1.5707964;
        let crossed = |prev: f32, cur: f32, now_: f32, t: f32| {
            let a = FUN_0011cf60(prev - cur);
            let d = FUN_0011cf60(now_ - t);
            a * d <= 0.0 || d.abs() < 0.0001
        };
        if x.grf & 1 != 0
            && crossed(self.yaw, self.prev[0], yaw, ta[0])
            && FUN_0011cf60(yaw - ta[0]).abs() < HALF_PI
            && crossed(self.pitch, self.prev[1], pitch, ta[1])
            && FUN_0011cf60(pitch - ta[1]).abs() < HALF_PI
        {
            self.latched = true;
            self.mode = 1;
            return (yaw, pitch);
        }
        if x.grf & 2 != 0
            && crossed(self.yaw, self.prev[2], yaw, tb[0])
            && FUN_0011cf60(yaw - tb[0]).abs() < HALF_PI
            && crossed(self.pitch, self.prev[3], pitch, tb[1])
            && FUN_0011cf60(pitch - tb[1]).abs() < HALF_PI
        {
            self.mode = 2;
            self.latched = true;
        }
        (yaw, pitch)
    }
}

impl Cplcy for CplcyFollow {
    fn g_follow(&self) -> Option<f32> {
        Some(self.g_follow)
    }
    /// 0x00221a18 (pv == 0 path).
    fn activate(&mut self, cm: &mut Cm, pso: u32, pv: u32, clk: CmClock) {
        self.save = FUN_0014fcc8(cm);
        self.pso = pso;
        self.track = pv;
        // pv == 0: params = DAT_002c4120; WP-P24: a track supplies its +0x114 entry (one entry: no 0x15e988 pick)
        let track = if pv != 0 { self.world.track(pv) } else { None };
        if let Some(t) = &track {
            self.params = t.params;
            if std::env::var_os("SLY2_FEEL_CAMLOG").is_some() {
                eprintln!("FEEL cam activate track {pv:#x} simple={} {:?}", t.simple(), t);
            }
        }
        self.first = true;
        FUN_00220d40(&mut self.params);
        if track.is_some() {
            self.world.set_track_params(pv, &self.params);
        }
        let c = FUN_0014e968(cm, pso, self.world.as_mut());
        let mut target = FUN_00220dd8(&self.params, &c);
        // FEEL R23: the track branch FUN_00220dd8(.., track) of the initial focus (0x00221a18:1233-1248)
        if pv != 0 && std::env::var_os("SLY2_FEEL_NOTRKACT").is_none() {
            let prm = self.params;
            let ta = self.world.track_addr(pv).unwrap_or(0);
            if let Some(m) = self.world.mem().filter(|_| ta != 0) {
                if std::env::var_os("SLY2_FEEL_CAMLOG").is_some() {
                    eprintln!("FEEL cam activate track pv {pv:#x} @ {ta:#x} path={:#x} ring={:#x} grf={:#x} i168={} i16c={}", m.r32(ta + 0x10c), m.r32(ta + 300), m.r32(ta + 0x164), m.r32(ta + 0x168), m.r32(ta + 0x16c));
                }
                if let Some(t) = follow_track::activate_target(m, ta, &prm, &c) {
                    if std::env::var_os("SLY2_FEEL_CAMLOG").is_some() {
                        eprintln!("FEEL cam activate target {t:?} (track-less {target:?})");
                    }
                    target = t;
                }
            }
        }
        let focus = add(target, cm.pos_adjust1);
        let (yaw, pitch, dist) = FUN_0011efb0(sub(v3(cm.mat[3]), focus));
        self.yaw = yaw;
        self.pitch = pitch;
        self.dist = dist - cm.s_near_clip;
        self.pos = v4(add(focus, FUN_0011e208(self.yaw, self.pitch, self.dist)), 0.0);
        self.mode = 0;
        self.y_off = self.params.y_offset;
        self.t_visible = clk.t;
        self.t_move = clk.t;
        self.latched = false;
        self.t_manual = clk.t - T_MANUAL_GRACE;
        if track.is_some_and(|t| t.f_184 != 0) {
            FUN_0014d8c0(cm);
        }
        // DAT_002ddf64 (= Cm +0x24 fov target): unchanged without a track or with track +0x188 == 0
        if let Some(t) = track {
            if t.fov != 0.0 {
                cm.rad_fov_target = t.fov;
            }
        }
    }

    /// 0x00221cc0
    fn deactivate(&mut self, cm: &mut Cm, _pso: u32, _pv: u32, clk: CmClock) {
        self.track = 0;
        FUN_0014fce8(cm, &self.save, clk);
    }

    /// 0x00221dd8 (track == 0: mode bits uVar7 = 4).
    fn update(&mut self, cm: &mut Cm, _h: bool, pad: &CplcyCtx, clk: CmClock) {
        let dt_real = clk.dt_real;
        let jt = self.world.jt(self.pso);
        self.FUN_002236e8(cm, pad.input_locked, clk);
        // WP-P24: iVar1 = +0x10 track: uVar7 = +0x164, params = the live +0x114 entry (ported: `simple`)
        let mut uvar7 = 4u32;
        // PART-3: with the track object in EE memory: +0x164 bits, 0x00221ce8, 0x00220dd8(.., track)
        let mut trk: Option<TrackCtx> = None;
        let mut target_t = None;
        if self.track != 0 {
            let ta = self.world.track_addr(self.track);
            if let Some((m, ta)) = ta.and_then(|a| self.world.mem().map(|m| (m, a))) {
                let (x, f, tg) = follow_track::begin(m, ta, &mut self.params, &cm.cpdefi);
                if std::env::var_os("SLY2_FEEL_CAMLOG").is_some() {
                    eprintln!("FEEL cam tick track @ {ta:#x} grf={:#x} target {tg:?} base {:?} a {:?} b {:?}", x.grf, cm.cpdefi.pos_base, x.a, x.b);
                }
                uvar7 = x.grf;
                target_t = Some((tg, f));
                trk = Some(x);
            } else if let Some(t) = self.world.track(self.track) {
                self.params = t.params;
                if t.simple() {
                    uvar7 = t.grf;
                }
            }
        }
        let mut frame = None;
        let target = match target_t {
            Some((tg, f)) => {
                frame = Some(f);
                tg
            }
            None => FUN_00220dd8(&self.params, &cm.cpdefi),
        };
        if let Some(j) = jt {
            if j.state == 2 {
                let (mut p, mut v) = (cm.pos_adjust1, cm.v_adjust1);
                FUN_0014e6d0(cm.cut, 8.0, -6.0, &mut p, &mut v, sub(j.pos_500, target), clk.dt);
                cm.pos_adjust1 = p;
                cm.v_adjust1 = v;
            }
        }
        let focus = add(target, cm.pos_adjust1);
        if trk.is_some_and(|x| x.f190 != 0) {
            cm.cpdefi.grfcpd[0] &= !1;
        }
        let g = if cm.cpdefi.grfcpd[0] & 1 == 0 { self.params.g_follow } else { 0.0 };
        self.g_follow = FUN_0011d018(self.g_follow, g, clk.dt, &SMP_GAIN).0;
        self.t_move = self.t_move.max(cm.cpdefi.t_move_last);
        if let (Some(x), Some(f)) = (trk.as_mut(), frame) {
            if let Some(m) = self.world.mem() {
                follow_track::angles(m, x, &f, focus, self.first, &mut self.track_yaw);
                follow_track::end(m, f);
            }
        }
        let (mut ta, mut tb) = trk.map(|x| (x.a, x.b)).unwrap_or(([0.0; 3], [0.0; 3]));
        if self.first {
            self.mode = 4;
        }
        if cm.cut != 0 {
            if cm.cut == 2 {
                self.mode = 4;
                self.yaw = f32::from_bits(cm.cut_p0);
                let o = (self.params.y_offset * cm.y_scale).atan();
                self.pitch = FUN_0011d948(f32::from_bits(cm.cut_p1) - o, PITCH_RANGE);
            } else if uvar7 & 3 == 0 {
                self.mode = 4;
                self.yaw = cm.cpdefi.rad_home;
                self.pitch = self.params.pitch_range[0];
            } else if uvar7 & 2 == 0 {
                self.mode = 1;
            } else if uvar7 & 1 == 0 {
                self.mode = 2;
            } else {
                let a = FUN_0011cf60(ta[0] - cm.cpdefi.rad_home);
                let b = FUN_0011cf60(tb[0] - cm.cpdefi.rad_home);
                self.mode = if a.abs() < b.abs() { 1 } else { 2 };
            }
            self.t_move = clk.t;
            self.t_manual = clk.t - T_MANUAL_GRACE;
        }
        let m = self.mode as u32;
        if m & uvar7 == 0 && 0 < self.mode && self.mode < 3 {
            self.mode = 4;
        }
        let vel = match (trk, self.world.mem()) {
            (Some(x), Some(m)) if x.f190 != 0 => follow_track::look_vel(m, &x, &cm.cpdefi),
            _ => v3(cm.cpdefi.v_base),
        };
        let yaw0 = self.yaw;
        let mut pitch = self.pitch;
        let mut yaw;
        let manual = uvar7 & 4 != 0 && (self.reset || 0.0 < pad.right[2] || self.rate_yaw != 0.0 || self.rate_pitch != 0.0);
        if !manual {
            // LAB_002225d8, mode 4: auto follow.
            if self.mode == 4 {
                let st = [pad.left[0], pad.left[1], 0.0];
                let sl = len(st);
                let stick = if sl < 0.0001 { [1.0, 0.0, 0.0] } else { scale(st, 1.0 / sl) };
                let dv = sub(focus, v3(cm.mat[3]));
                let dl = len(dv);
                let dir = if dl < 0.0001 { [1.0, 0.0, 0.0] } else { scale(dv, 1.0 / dl) };
                // track +0x180 (!= -1) overrides CPDEFI +0x3c
                let nm = match trk {
                    Some(x) if x.f180 != u32::MAX => x.f180,
                    _ => cm.cpdefi.unk_3c.to_bits(),
                };
                let n = match nm {
                    0 => cross([0.0, 0.0, 1.0], dir),
                    1 => dir,
                    // the original reads an uninitialised stack vector here
                    _ => [0.0; 3],
                };
                let nl = len(n);
                let n = if nl < 0.0001 { [1.0, 0.0, 0.0] } else { scale(n, 1.0 / nl) };
                let w = FUN_0011cfe0(dot(vel, n) / dl, FOLLOW_RATE_MAX);
                // quirk: uses the NORMALISED left-stick y (0 when the stick is centred)
                let sy = stick[1];
                let b = FUN_0011d948(STICK_BOOST[0] + sy * (STICK_BOOST[1] + sy * STICK_BOOST[2]), STICK_BOOST_RANGE);
                let tm = clk.t - self.t_manual;
                let r = FUN_0011d948(MANUAL_RAMP[0] + tm * (MANUAL_RAMP[1] + tm * MANUAL_RAMP[2]), UNIT_RANGE);
                yaw = FUN_0011cf60(yaw0 + w * self.g_follow * b * r * clk.dt);
                // speed = |v.xy| (single vaddbc: x+y only)
                let s = ((vel[0] * vel[0] + vel[1] * vel[1]).sqrt() * 0.0016666667).min(1.0);
                if 0.1 < s {
                    let smp = [SMP_PITCH[0] * s, SMP_PITCH[1] * s, SMP_PITCH[2], 0.0];
                    let t = FUN_0011d948(pitch, self.params.pitch_range);
                    pitch = FUN_0011d2d0(pitch, t, clk.dt, &smp);
                }
                if let Some(x) = trk.filter(|_| uvar7 & 3 != 0) {
                    let (y, p) = self.track_blend(&x, vel, yaw0, self.pitch, yaw, pitch, ta, tb, clk);
                    yaw = y;
                    pitch = p;
                }
            } else {
                yaw = yaw0;
            }
        } else {
            // manual (right stick) or reset
            if self.mode != 4 {
                // FUN_001fbb90(3) + FUN_001fb2f8(e, 0): fire-and-forget swing sound (no reference kept)
                if let Some(es) = self.world.emitters() {
                    let id = es.new_emitter_1fbb90(3);
                    es.set_unpositioned_1fb2f8(id, 0);
                }
            }
            if 0.0 < pad.right[2] {
                self.reset = false;
            }
            let (ny, np);
            if !self.reset {
                let slew = STICK_SLEW * dt_real;
                let ty = pad.right[0] * pad.right[0].abs() * STICK_YAW_RATE;
                let tp = pad.right[1] * pad.right[1].abs() * STICK_PITCH_RATE;
                self.rate_yaw = FUN_0011d948(ty, [self.rate_yaw - slew, self.rate_yaw + slew]);
                let (lo, hi) = (self.rate_pitch - slew, self.rate_pitch + slew);
                self.rate_pitch = if lo <= tp { if tp <= hi { tp } else { hi } } else { lo };
                let a = pad.right[1].abs().atan2(pad.right[0].abs());
                if a < 0.2617994 {
                    self.rate_pitch = 0.0;
                } else if 1.3089969 < a {
                    self.rate_yaw = 0.0;
                }
                ny = FUN_0011cf60(yaw0 + self.rate_yaw * dt_real);
                np = FUN_0011d948(FUN_0011cf60(pitch + self.rate_pitch * dt_real), PITCH_RANGE);
                let t = self.world.t_2ddee4();
                if 5.0 < t - self.t_msg {
                    self.t_msg = t;
                    self.world.activity(6, 0);
                }
            } else {
                let smp = if jt.map(|j| j.f_10e4 == 4).unwrap_or(false) { &SMP_RESET_FAST } else { &SMP_RESET };
                let (a, va) = FUN_0011d340(yaw0, self.rate_yaw, self.reset_yaw, dt_real, smp);
                let (b, vb) = FUN_0011d340(pitch, self.rate_pitch, self.reset_pitch, dt_real, smp);
                self.rate_yaw = va;
                self.rate_pitch = vb;
                ny = a;
                np = b;
                if a == self.reset_yaw && b == self.reset_pitch {
                    self.reset = false;
                }
            }
            yaw = ny;
            pitch = np;
            self.mode = 4;
            self.latched = true;
            self.t_manual = clk.t;
        }
        // mode 4, (uVar7 & 3) == 0
        let (mut s_yaw, mut s_pitch, mut s_dist) = (yaw, pitch, 0.0);
        let (mut f30, mut f31) = (0.0, 0.0);
        if self.mode == 2 {
            (s_yaw, s_pitch, s_dist) = (tb[0], tb[1], tb[2]);
            f31 = -self.params.x_offset;
            f30 = self.params.y_offset;
        } else if self.mode == 1 {
            (s_yaw, s_pitch, s_dist) = (ta[0], ta[1], ta[2]);
            f31 = self.params.x_offset;
            f30 = self.params.y_offset;
        } else if self.mode == 4 && uvar7 & 3 == 0 {
            f30 = self.params.y_offset;
            s_dist = self.params.s_dist.abs();
            f31 = self.params.x_offset;
        } else if self.mode == 4 {
            // blend the two track distances / screen offsets by the yaw between the two track points
            let w = if uvar7 & 2 == 0 {
                1.0
            } else if uvar7 & 1 == 0 {
                0.0
            } else {
                let y = self.yaw;
                const TAU: f32 = 6.2831855;
                if y < ta[0] && tb[0] > y {
                    if tb[0] < ta[0] {
                        ta[0] -= TAU;
                    } else {
                        tb[0] -= TAU;
                    }
                } else if ta[0] < y && tb[0] < y {
                    if tb[0] <= ta[0] {
                        tb[0] += TAU;
                    } else {
                        ta[0] += TAU;
                    }
                }
                let k = (y - tb[0]) / (ta[0] - tb[0]);
                k * k * (3.0 - (k + k))
            };
            f30 = self.params.y_offset;
            s_dist = w * ta[2] + (1.0 - w) * tb[2];
            f31 = w * self.params.x_offset + (1.0 - w) * -self.params.x_offset;
        } else {
            s_yaw = 0.0;
            s_pitch = 0.0;
        }
        // LAB_00223324: distance / height springs
        if cm.cut == 0 {
            let k = self.params.k_spring * SPRING_K * clk.dt;
            let d = self.params.d_spring * SPRING_D * clk.dt + 1.0;
            let v1 = d * self.v_dist + (s_dist - self.dist) * k;
            let v2 = d * self.v_off + (f30 - self.y_off) * k;
            self.v_dist = v1;
            self.v_off = v2;
            s_dist = self.dist + v1 * clk.dt;
            f30 = self.y_off + v2 * clk.dt;
        } else {
            self.v_dist = 0.0;
            self.v_off = 0.0;
        }
        let off = FUN_0011e208(s_yaw, s_pitch, s_dist);
        let mut want = add(focus, off);
        if std::env::var_os("SLY2_FEEL_CAMLOG").is_some() {
            eprintln!("FEEL cam want cut {} mode {} yaw {s_yaw} pitch {s_pitch} dist {s_dist} focus {focus:?} want {want:?}", cm.cut, self.mode);
        }
        if trk.is_some_and(|x| x.f18c != 0) {
            // track +0x18c: no collision / cut placement
        } else if cm.cut == 0 {
            want = FUN_0014f110(self.world.as_mut(), v3(self.pos), want, dt_real);
        } else {
            // WP-P17: cut -> FUN_00223a70 occlusion-free placement search
            want = self.FUN_00223a70(cm, focus, want);
        }
        let d = sub(want, focus);
        let (y2, p2, l2) = FUN_0011efb0(d);
        s_yaw = y2;
        s_pitch = p2;
        s_dist = l2;
        self.pos = v4(want, 0.0);
        // 0x00221dd8 tail: (fStack_120 + DAT_002ddf8c) / fStack_120. WP-EP1c: a track with distance 0 (+4 = 0, rows
        // 15 22 32 38 at spawn) divides by 0: the PS2 FPU gives +-FMAX (no inf/NaN) and d = 0, so campos = focus.
        let k = if s_dist == 0.0 { f32::MAX.copysign(s_dist + cm.s_near_clip) } else { (s_dist + cm.s_near_clip) / s_dist };
        let campos = add(focus, scale(d, k));
        let a_yaw = (f31 * cm.x_scale).atan();
        let a_pitch = (f30 * cm.y_scale).atan();
        if !self.first {
            let v = self.v_off_yaw + (self.off_yaw * OFFSET_K + self.v_off_yaw * OFFSET_D) * clk.dt;
            self.v_off_yaw = v;
            let w = self.v_off_pitch + (self.off_pitch * OFFSET_K + self.v_off_pitch * OFFSET_D) * clk.dt;
            self.v_off_pitch = w;
            self.off_yaw += v * clk.dt;
            self.off_pitch += w * clk.dt;
        } else {
            if cm.cut == 0 {
                let (_, horiz, z) = FUN_0011ef08(d);
                let (fy, fp) = FUN_0011bc88(v3(cm.mat[0]));
                let ly = FUN_0011cf60(s_yaw + 3.1415927 + a_yaw);
                let lp = FUN_0011cf60((-z).atan2(horiz) + -a_pitch);
                self.off_yaw = FUN_0011cf60(fy - ly);
                self.off_pitch = FUN_0011cf60(fp - lp);
            } else {
                self.off_yaw = 0.0;
                self.off_pitch = 0.0;
            }
            self.v_off_yaw = 0.0;
            self.v_off_pitch = 0.0;
        }
        FUN_0014e1a8(cm, a_yaw + self.off_yaw, -a_pitch + self.off_pitch, campos, focus, clk);
        self.y_off = f30;
        self.yaw = s_yaw;
        self.pitch = s_pitch;
        self.dist = s_dist;
        self.prev = [ta[0], ta[1], tb[0], tb[1]];
        self.first = false;
    }

    fn set_saved_near(&mut self, s: f32) {
        self.save.near = s;
    }
}

/// WP-P24: the C2db858 view of a class-15 track (FUN_00220fd0 load): FUN_00221818(this, 0x30) makes entry 0 a
/// copy of DAT_002c4120 with grf 0x30, the property fields with desc flag 0x2000 write into that entry at their
/// desc offset, the others into the object (197 +0x164, 198 +0x168, 199 +0x16c, 207 +0x184, 208 +0x188, 210
/// +0x190); +0x164 starts at 7 (slot 6 FUN_00220ed0) and loses bit 1 without a +0x10c path. WP-I5: field 190
/// (0x00221960) sets the entry's +0x70/+0x74 = v, +0x78 = 1; field 176 (another entry, `n_params` > 1) is not
/// in any level (probe over 43 levels), so the 0x0015e988 entry pick is not ported.
pub fn follow_track(props: &[crate::engine::world::props::Prop], path: bool) -> FollowTrack {
    use crate::engine::classes::k4::word;
    use crate::engine::world::props::PropValue;
    let mut w = FOLLOW_PARAMS_DEFAULT.to_words();
    w[0] = 0x30;
    let mut t = FollowTrack { params: FOLLOW_PARAMS_DEFAULT, grf: 7, f_168: 0, f_16c: 0, f_190: 0, f_184: 0, fov: 0.0, path, n_params: 1, f_128: 0 };
    for p in props {
        let PropValue::Scalar(b) = &p.value else { continue };
        let Some(d) = crate::engine::world::props::field(p.field) else { continue };
        if d.flags & 0x2000 != 0 {
            for (k, c) in b.chunks_exact(4).enumerate() {
                if let Some(x) = w.get_mut(d.count as usize / 4 + k) {
                    *x = u32::from_le_bytes([c[0], c[1], c[2], c[3]]);
                }
            }
            continue;
        }
        match p.field {
            197 => t.grf = word(b),
            198 => t.f_168 = word(b) as i32,
            199 => t.f_16c = word(b) as i32,
            207 => t.f_184 = word(b) as i32,
            200 => t.f_128 = word(b) as i32,
            208 => t.fov = f32::from_bits(word(b)),
            210 => t.f_190 = word(b) as i32,
            // WP-I5: 190 setter 0x00221960 on the current entry: +0x70 = +0x74 = v, +0x78 = 1.0
            190 => {
                w[28] = word(b);
                w[29] = word(b);
                w[30] = 1.0f32.to_bits();
            }
            // 176 setter FUN_00221818(this, key): append / select another entry (never in the level data)
            176 => t.n_params = 2,
            _ => {}
        }
    }
    if !path {
        t.grf &= !2;
    }
    t.params = FollowParams::from_words(&w);
    t
}
