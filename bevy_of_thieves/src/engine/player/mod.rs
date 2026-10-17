//! WP-P: Sly (JT class C2d2e98, vtable 0x2d2e98, size 7472) controller. Call tree: tools/engineport/jt_map.md.
//!
//! Per frame (original order): JT update slot15 0x18bce0 [input 0x18b7e8 -> 0x18a8c8 per-state input,
//! per-jts update switch, 0x18ed70, grab test, 0x189618 anim] -> physics driver 0x1f8dd8 whose per-object slot16
//! calls JT slot69 0x188960 (forces by jts: 0x204108 ground / 0x1f9b20 + 0x196df8 jump/air).
//!
//! 1:1 here: 0x188960 dispatch for jts 0/1/2 (others listed as unported), 0x204108 (+0x1a4 == 0 branch),
//! 0x001f59a0 over-speed damping term, 0x196df8 default case (air steering gains 2.5/3.5/2.5, 1/dt for jtbs 0/2,
//! terminal-fall drag below -750 * -2), 0x18c250 SetJtJts early-out + velocity clear on jts 0 entry +
//! ASEG field per state (anim hook), 0x195fb8 jtbs bookkeeping (+0x1c jtbs, +0x20 entry time).
//! REPLACED: 0x18a8c8 input (5.5 KB of target picking / ledge / rope / point logic) by the minimal stick ->
//! target velocity / X -> jump mapping below; 0x001f59a0's steering-to-target-frame part (FUN_0022fd38 /
//! FUN_0011bcf8 angle braking) by velocity-error steering with the same accel limit (s - p1)/p2; jump launch
//! speed = sqrt(2 g (DAT_002bf73c - DAT_002bf738)) taken from 0x19a750 case 6 (the normal-jump launch path is
//! in the unported input code); JT accelerations enter the driver as impulses a*dt (identical trapezoid result,
//! see physics::driver::FUN_0022de88 / FUN_001f9e70) because physics/ (WP-E6) owns slot16.
//! UNPORTED jts: 3 zap (0x1903b8/0x190e18), 4 (0x1a1148), 5 hide (0x1a0450), 6 pipe (0x137248), 7 edge
//! (0x194170), 8 celebrate (0x19df70), 10/0xc (0x1a0450/0x20bc08), 0xe steer, 0x15 (0x185e40), 0x16 (0x18df60),
//! 0x18 (0x189c40/0x189ec8); jtbs other than 0/1/2; 0x196aa8, 0x18ed70, 0x189618, grab test, double jump
//! (lives in 0x197500 air transitions, unported), ledge/rope/point "circle" actions.
//!
//! WP-P2: 0x197500 (JMP body-state update) ported for jtbs 0..4 and 0x11 (ledge hang). Sly 2 JMP numbering:
//! 0 jump init, 1 rise, 2 double-jump init, 3 double-jump rise, 4 fall (Sly 1's 2 = fall does NOT apply);
//! 0->1 / 2->3 after DAT_002bf818 / DAT_002bf81c = 0.15 s, 1->4 when v.z < 0, 3/4 land when grounded
//! (+0x208 normal z > 0.7; vtable +0x2c4 exit), else ledge search FUN_00199d48 -> 0x11; 0x11 + X after 0.25 s
//! -> FUN_00195a00(3) (jump off the ledge). Double jump = X pressed with JMP +0x28 set and jtbs not in
//! 9..0x16 -> FUN_00195a00(6): v.z = sqrt(-2 g (DAT_002bf73c - DAT_002bf738)) (0x19a750 case 6), jtbs 2;
//! +0x28 is set by FUN_00195a00 cases 0/1/2/3 and cleared by FUN_00195fb8 on entering jtbs 2/3.
//! REPLACED in P2: FUN_00199d48 ledge search (segment probes against CollisionWorld, `ledge_probe`); hang =
//! body frozen at the ledge (the original hangs through JMP +0xc8.. edge data, not traced); ground-jump launch
//! speed (0x19a750 cases 1/2 are VU code Ghidra lost: same formula as case 6 kept); FUN_00143d20 (0.1 s
//! action gate, treated as open); short-hop flag (+0x24 when X released in jtbs 0).
//! Still unported: jtbs 5..0x10, 0x12..0x16, jts 3..0x18 (pole/rope/point/pipe/hide/zap/edge),
//! FUN_0019b110 (O air attack), FUN_0019c828 / FUN_0019c688.
//!
//! WP-P3: 1:1 FUN_0019a750 (all launch kinds: 1/7/9/0xd/0xe/0xf height jumps sqrt(max(0, 2g(h_rel - h))) with
//! h 215/1200(9)/500(0xd)/250(0xe), 2/3/0x12 floors 700/800/900, 4 = clamp(700 + vz/2), 6 double jump with
//! the 0x11d508 quadratic; result max(.., vz)), FUN_00195a00 (field table per kind, SetVelocity), the joystick
//! object +0x150 press-time API FUN_001c4660 / FUN_001c4738 / FUN_001c46c0 (X buffered 0.1 s on the ground,
//! O 0.2 s), FUN_00143d20 action gate formula, FUN_0019c828, FUN_0019c688 dispatch, FUN_00197370 direction
//! pick, 0x197500 restart loop with cases 0/2 (short hop +0x24), 1/7, 3/4, 5, 6, 8, 9..0x10, 0x11, and the
//! 0x18a8c8 jts 0/1 part (walk off after 0.1 s without ground contact = +0x20c, ground jump kind 1, O ground
//! attack). Ground jump: h_rel = sphere centre - ground contact z (jt +0x218 / +0x238).
//! REPLACED in P3: joy +0x88 writer (pad `pressed` stamped with t), +0x238 contact z (centre - n.z r),
//! +0x1a8 yaw (last stick direction), FUN_00143d20's +0x248 (not traced, gate open) and FUN_00173610.
//! Recorded-only (Jt::calls): FUN_0019b110 air attack, FUN_001938c0 ground attack, FUN_001928f8 attaches,
//! FUN_00197370 ASEG play, FUN_00140c88, FUN_0011ef08, FUN_00135248. Still unported: 0x197f98 air steering
//! (consumer of the short hop), FUN_00199d48 1:1 (contact edge lists), jts 3..0x18 bodies (pole/rope/point/
//! pipe/hide/zap/edge/celebrate/death), jtbs 0x12..0x16 bodies, 0x194a40 target pick.
//!
//! WP-P5 (integration of p5_target / p5_attach / p5_states / p5_hull into `Jt`):
//! 1:1 wired: 0x197500 pre check (FUN_0019c828 target bit-60 -> p5_states::FUN_00197500_pre); jtbs 5, 6, 8,
//! 9..0x10, 0x12..0x16 bodies (p5_states::FUN_00197500_p5; 0..4 and 0x11 stay here) and the airborne no-ledge
//! tail of 3/4 (FUN_00197500_tail5: stuck test 0x19a098 -> safe-point warp 0x19f590); O in the air =
//! 0x19b110 target pick (p5_target, the CALL_AIR_ATTACK record is kept) + 0x195fb8 entries / exits of the
//! flight states; jtbs 9..0x10 / 0x12 with xf kind 0x11 are kinematic (0x22de88 case 0x11 sets pos / vel, the
//! physics driver is skipped; 0x197f98 abort -> jtbs 4; arrival -> 0x19c688); 0x19c688 attach cases ->
//! FUN_001928f8 -> p5_attach::attach_to (0x192bd8 entry) when the target has attach data in `p5_targets`;
//! 0xe / 0xf / 0x10 arrival through p5_states::FUN_0019c688_p5; SetJtJts = `set_jts` (p5 exit / enter actions,
//! 0x18c250_p5) while the raw `FUN_0018c250` is what p5 code calls; leaving jts 7 -> 0x192bd8(-1) detach;
//! input: jts 3 -> 0x190520, jts 7 -> 0x1911a8; update: jts 3 -> 0x190e18, jts 7 -> 0x191a10, others
//! 0x18bce0_p5; forces 0x188960 jts 3/4/6/8/0xe/0x18 from FUN_00188960_p5 (+0x204108 for 4/6/0xe/0x18);
//! FUN_00199d48 contacts from Sly's GEOM-199 hull (p5_hull, `Jt::hull`, with the JT side uVar5).
//! Adapters: `P5Target` registry (0x19b110 category object + attach data; `from_attach` builds rope / point /
//! fixed-member objects), `target_obj_from_p4` (P4 JtTarget -> TargetObj from its pos only: class 0x13 -> point,
//! 0x26 -> fixed-member point, others dropped, because P4 targets carry no rope path / pole axis / surface), attach Path -> target Path, `with_p5` (P5 / P5World
//! taken out of the JT while p5_states runs; `p5_busy` guards re-entry), events AttachEv / Fx -> JT
//! (SetJtJts table 0x18d568, velocity, ASEG via JtAnim, rest recorded in `calls`).
//! REPLACED: jts 7 forces = velocity driven to AttachExt::goal_world in one step with gravity cancelled
//! (+0x194170 for edge ropes, kind 3 f_1bc) instead of the contact motor 0x2044b0; body yaw 0x204740 (the
//! facing turns instantly); the stick no longer sets the yaw in jts 7.
//! Still unported / gaps: the game's category object lists and attach data (filled by the caller into
//! `p5_targets`; the plugin has none yet), AttachExt per-JT tuning (hang radius, grab offset, 8-way /
//! rope blend velocities from 0x1a0450 / ASEGs: zero by default, so attached JTs do not move until set),
//! Sly's model hull reaches the plugin only through the `PlayerHull` resource (nothing inserts it yet, so
//! the P4 sphere contacts remain the runtime fallback), P5World inputs other than contact normals (safe
//! points, partner, jmp target), jtbs 0x11 hang body position (P2 REPLACED: body frozen; WP-P8 ported the rest of
//! case 0x11 1:1: 0.03 s edge re-check FUN_001999d0 -> jtbs 4 when lost, facing FUN_002046b0(jt, -dir), attack
//! button 0x400 -> FUN_0019cc50(jt+0xf00) recorded; jt+0x1010 FUN_0011f088 decay: WP-P11).
#![allow(non_snake_case)]

pub mod plugin;
pub mod p4;
pub mod p5_target;
pub mod p5_attach;
pub mod p5_states;
pub mod p5_hull;
pub mod p6;
pub mod p7_damage;
pub mod p8_hazard;
pub mod p10;
pub mod p11_cane;
pub mod p13;
pub mod p14;
pub mod p15_anim;
pub mod p15;
pub mod p15_binoc;
pub mod p16;
pub mod p18;
pub mod p18_gadget;
pub mod p20;
pub mod p22;
pub mod p23;
pub mod p25;
pub mod p26;
pub mod loot;
pub mod pj_adapt;

pub use p4::{Attach, JtContact, JtTarget, LedgeHit, TargetCand};

use crate::engine::input::pad::{PadState, BTN_CIRCLE, BTN_CROSS};
use crate::engine::physics::driver::{self, AloPhys};
use crate::engine::physics::geom::{dot, V3};
use crate::engine::physics::CollisionWorld;

/// JT +0x18c8 main state (Sly 1 JTS numbering where Sly 2 agrees).
pub const JTS_STAND: i32 = 0;
pub const JTS_RUN: i32 = 1;
/// jt+0x4c0: the ATTACH sub-object (p5_attach `AttachEv::Anim` offsets are relative to it).
pub const ATTACH_BASE: u32 = 0x4c0;
pub const JTS_JUMP: i32 = 2;
/// JMP +0x1c body state (Sly 1 JTBS).
pub const JTBS_NIL: i32 = -1;
pub const JTBS_JUMP_INIT: i32 = 0;
pub const JTBS_JUMP_RISE: i32 = 1;
pub const JTBS_DJUMP_INIT: i32 = 2;
pub const JTBS_DJUMP_RISE: i32 = 3;
pub const JTBS_JUMP_FALL: i32 = 4;
pub const JTBS_LEDGE: i32 = 0x11;
/// DAT_002bf818 / DAT_002bf81c (0x197500 cases 0 / 2).
pub const T_JUMP_INIT: f32 = 0.15;
pub const T_DJUMP_INIT: f32 = 0.15;
/// 0x197500 case 0x11: hang time before X jumps off.
pub const T_LEDGE_JUMP: f32 = 0.25;
/// REPLACED FUN_00199d48 reach: probe distance beyond the sphere / height above the centre.
pub const LEDGE_FWD: f32 = 25.0;
pub const LEDGE_REACH: f32 = 60.0;

/// DAT_002bf7f0 / DAT_002bf7f4 / DAT_002bf7f8 (0x196df8 air steering gains).
pub const AIR_GAIN_X_SLOW: f32 = 2.5;
pub const AIR_GAIN_X_FAST: f32 = 3.5;
pub const AIR_GAIN_Y: f32 = 2.5;
/// DAT_002bf810 / DAT_002bf814 (0x196df8 terminal fall).
pub const FALL_LIMIT: f32 = -750.0;
pub const FALL_DRAG: f32 = -2.0;
/// DAT_002bf738 / DAT_002bf73c (0x19a750 case 6 heights).
pub const JUMP_H0: f32 = 215.0;
pub const JUMP_H1: f32 = 315.0;
/// DAT_002bf730 = 515.0: REPLACED role guess = run speed (the JT move params +0x1f4/+0x1f8/+0x1fc come from
/// 0x18c250 / 0x180668 per state, not traced).
pub const RUN_SPEED: f32 = 515.0;
/// Sly collision sphere radius (REPLACED: JT uses its ALO collision parts; not resolved here).
pub const GROUND_GRACE: f32 = 0.1;
pub const JT_RADIUS: f32 = 30.0;

/// Anim hook (WP-D): 0x18c250 plays the state's ASEG through FUN_0012f9b8 / FUN_001f03a8.
pub trait JtAnim {
    /// `field` = JT member holding the ASEG (0x1324 run, 0x1994 jump jtbs 0..2, 0x14d0 hide, ...).
    fn play_aseg(&mut self, field: u32, jts: i32, jtbs: i32);
}
impl JtAnim for () {
    fn play_aseg(&mut self, _: u32, _: i32, _: i32) {}
}

/// Controller intent (output of the replaced input step 0x18b7e8/0x18a8c8).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct JtInput {
    /// left stick, x right / y forward, magnitude 0..1 (PadState.left after 0x187768)
    pub stick: [f32; 2],
    /// X pressed this frame (PadState.pressed & 0x40)
    pub jump: bool,
}

/// pad -> intent. Camera yaw rotates the stick into world XY (Z-up, I: camera col0 = forward).
pub fn input_from_pad(pad: &PadState) -> JtInput {
    JtInput {
        stick: [pad.left.x * pad.left.mag, pad.left.y * pad.left.mag],
        jump: pad.pressed & BTN_CROSS != 0,
    }
}

/// 0x0018b7e8 stick part: camera-relative stick. `cam_fwd` = CM matrix column 0 (forward, Z-up); stick y
/// moves along the camera's horizontal forward, stick x along its right (-column 1). Returned in the stick
/// convention `Jt` uses (stick y = world +x, stick x = world -y). Vertical camera: unrotated.
pub fn input_from_pad_cm(pad: &PadState, cam_fwd: V3) -> JtInput {
    let mut i = input_from_pad(pad);
    let l = (cam_fwd[0] * cam_fwd[0] + cam_fwd[1] * cam_fwd[1]).sqrt();
    if l < 1e-4 {
        return i;
    }
    let f = [cam_fwd[0] / l, cam_fwd[1] / l];
    let r = [f[1], -f[0]];
    let w = [i.stick[1] * f[0] + i.stick[0] * r[0], i.stick[1] * f[1] + i.stick[0] * r[1]];
    i.stick = [-w[1], w[0]];
    i
}

/// WP-P12: the JT field FUN_00195fb8 (JMP = jt+0x2c0) plays on entering `jtbs` (0x00196844 tail): jtbs 2/3
/// (double jump) JMP[idx + 0x4d] = jt+0x3f4 + 4*idx (JMP +8 = `jmp_idx`; the original falls back to
/// JMP[0x4d] then JMP[0x48] when that member is null, which the anim driver does by skipping unbound fields),
/// jtbs 0x11 (ledge hang) JMP[0x55] = jt+0x414 (mode 9). Other jtbs: None here (0/1/4/5/7 JMP[idx + 0x48]
/// = jt+0x3e0 + 4*idx are left to the jump blend 0x1994 of the P2 port; the param_1[1] == 0xe/0xd/0xf/0x10
/// overrides are unported).
pub fn jmp_body_aseg_195fb8(jtbs: i32, idx: i32) -> Option<u32> {
    match jtbs {
        JTBS_DJUMP_INIT | JTBS_DJUMP_RISE => Some(0x3f4 + 4 * idx.clamp(0, 4) as u32),
        JTBS_LEDGE => Some(0x414),
        // WP-P19: 0x13 JMP[0x5e] = jt+0x438 (glide open), 0x14 JMP[0x6e] = jt+0x478 (mode 9; unbound for Sly)
        0x13 => Some(0x438),
        0x14 => Some(0x478),
        _ => None,
    }
}

/// REPLACED FUN_00199d48 (ledge search): a wall ahead of the sphere at centre height, then a downward segment
/// from `LEDGE_REACH` above the probe point onto the wall top (start must be free). Returns the ledge top.
pub fn ledge_probe(cw: &CollisionWorld, pos: V3, dir: [f32; 2]) -> Option<V3> {
    let ahead = add(pos, scale([dir[0], dir[1], 0.0], JT_RADIUS + LEDGE_FWD));
    cw.clip_segment(pos, ahead).iter().find(|l| l.t[0] > 0.0)?;
    let top = [ahead[0], ahead[1], pos[2] + LEDGE_REACH];
    let hits = cw.clip_segment(top, [ahead[0], ahead[1], pos[2]]);
    let h = hits.first()?;
    (h.t[0] > 0.0).then_some(h.pos[0])
}

/// The JT's joystick object (jt +0x150, "JOY"): +0x18 stick angle, +0x1c stick magnitude, +0x30 held
/// buttons, +0x88 per-button press time (16 floats, DAT_002dded0 of the last press; consumed = -FLT_MAX).
/// REPLACED writer: the original fill of +0x88 was not traced; `Jt` stamps the pad's `pressed` bits with the
/// clock time each frame (same edge source as input::pad, H).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Joy {
    /// +0x18 stick angle (world yaw, rad)
    pub angle: f32,
    /// +0x1c stick magnitude 0..1
    pub mag: f32,
    /// +0x30 held buttons (H bit layout)
    pub held: u32,
    /// +0x88 press times
    pub press_t: [f32; 16],
}

/// DAT_002a76f0 = -FLT_MAX (consumed press time).
pub const PRESS_NONE: f32 = -f32::MAX;

impl Default for Joy {
    fn default() -> Self {
        Self { angle: 0.0, mag: 0.0, held: 0, press_t: [PRESS_NONE; 16] }
    }
}

impl Joy {
    /// 0x001c4660(window, joy, mask): bit k set when button k (in mask) was pressed within `window` s of t.
    pub fn FUN_001c4660(&self, t: f32, window: f32, mask: u32) -> u32 {
        let mut out = 0;
        let (mut m, mut k) = (mask, 0usize);
        while m != 0 && k < 16 {
            if m & 1 != 0 && t - self.press_t[k] <= window {
                out |= 1 << k;
            }
            m >>= 1;
            k += 1;
        }
        out
    }
    /// 0x001c4738(joy, mask): consume = press time -FLT_MAX.
    pub fn FUN_001c4738(&mut self, mask: u32) {
        let (mut m, mut k) = (mask, 0usize);
        while m != 0 && k < 16 {
            if m & 1 != 0 {
                self.press_t[k] = PRESS_NONE;
            }
            m >>= 1;
            k += 1;
        }
    }
    /// 0x001c46c0(window, joy, mask): test then consume what was found.
    pub fn FUN_001c46c0(&mut self, t: f32, window: f32, mask: u32) -> u32 {
        let r = self.FUN_001c4660(t, window, mask);
        self.FUN_001c4738(r);
        r
    }
    /// REPLACED +0x88 writer: stamp `pressed` bits with t, `held` = buttons.
    pub fn feed(&mut self, t: f32, held: u32, pressed: u32) {
        self.held = held;
        for k in 0..16 {
            if pressed & (1 << k) != 0 {
                self.press_t[k] = t;
            }
        }
    }
}

/// 0x0011e1a0(angle, s, z): (cos a * s, sin a * s, z) via the 0x11d608 sin/cos.
pub fn FUN_0011e1a0(angle: f32, s: f32, z: f32) -> V3 {
    [angle.cos() * s, angle.sin() * s, z]
}

/// 0x0019a750 jump launch velocity for JMP kind `kind` (switch on kind-1, table 0x2a6570): returns the new v.z,
/// always max(computed, vz). `g` = e8+0x88 gravity z, `h_rel` = (jt+0x218)->z - jt+0x238 (collision-sphere
/// centre z minus ground contact z).
pub fn FUN_0019a750(kind: i32, vz: f32, g: f32, h_rel: f32) -> f32 {
    let r = match kind {
        2 => DAT_002BF8C8,
        0x12 => DAT_002BF8CC,
        3 => DAT_002BF8D0,
        1 | 7 | 9 | 0xd | 0xe | 0xf => {
            let h = match kind {
                9 => DAT_002BF8D4,
                0xe => DAT_002BF8D8,
                0xd => DAT_002BF8DC,
                _ => JUMP_H0,
            };
            // max.s f2 = max(0, 2g (h_rel - h)); sqrt.s
            ((g + g) * (h_rel - h)).max(0.0).sqrt()
        }
        4 => {
            // 700 + vz * (0.5 + vz * 0), clamped to [-FLT_MAX, 700] (0x2bf8b0..0x2bf8c4)
            let f = 700.0 + vz * (0.5 + vz * 0.0);
            if f < -f32::MAX {
                -f32::MAX
            } else if 700.0 < f {
                700.0
            } else {
                f
            }
        }
        6 => {
            let v0 = (g * (JUMP_H0 * -2.0)).sqrt(); // f20
            let f27 = -v0;
            let f24 = f27 / g; // time to apex
            let vzc = v0.min(vz); // min.s f21
            let f23 = (vzc - v0) / g;
            let f20 = f23 * (v0 + g * (f23 * 0.5)); // height risen since launch
            let b = (g * ((JUMP_H1 - JUMP_H0) * -2.0)).sqrt();
            let mut out = [0.0f32; 2]; // sp+0x20 (uninitialised on the PS2 when < 2 roots)
            crate::engine::physics::dynamics::FUN_0011d508(g * 0.5, b, JUMP_H0, &mut out);
            let f1 = out[1] - f24;
            if 0.0 < vzc {
                (g * ((JUMP_H1 - f20) * -2.0)).sqrt()
            } else {
                let f20 = if vzc < f27 { 0.0 } else { f20 };
                let tt = f24 + f24 + f1 - f23;
                -f20 / tt - g * (tt * 0.5)
            }
        }
        _ => vz,
    };
    r.max(vz) // max.s f0, f0, f12 (0x19aa14)
}
/// 0x2bf8c8.. launch constants of 0x19a750.
pub const DAT_002BF8C8: f32 = 700.0;
pub const DAT_002BF8CC: f32 = 900.0;
pub const DAT_002BF8D0: f32 = 800.0;
pub const DAT_002BF8D4: f32 = 1200.0;
pub const DAT_002BF8D8: f32 = 250.0;
pub const DAT_002BF8DC: f32 = 500.0;
/// DAT_002bf754 (FUN_00195a00 case 3 push), DAT_002bf760 (case 0xe stick boost * 515).
pub const DAT_002BF754: f32 = 100.0;
pub const DAT_002BF760: f32 = 1.75;
/// DAT_002bf82c / DAT_002bf830 (0x197500 case 0x16 timers).
pub const DAT_002BF82C: f32 = 0.5;

/// Calls into code that is not ported (recorded in `Jt::calls` so tests/game can see them).
pub const CALL_AIR_ATTACK: u32 = 0x0019b110;
pub const CALL_GROUND_ATTACK: u32 = 0x001938c0;
pub const CALL_ANIM_DIR: u32 = 0x00197370;
pub const CALL_ATTACH: u32 = 0x001928f8;

#[derive(Clone, Debug)]
pub struct Jt {
    pub phys: AloPhys,
    /// +0x18c8
    pub jts: i32,
    /// +0x18d0 / JMP +0x1c
    pub jtbs: i32,
    /// JMP +0x20 time the body state was entered (DAT_002dded0)
    pub t_jtbs: f32,
    /// JMP +0xa0 target velocity (world)
    pub v_target: V3,
    /// JMP +0x34 force scale
    pub air_scale: f32,
    /// +0x1f4 / +0x1f8 / +0x1fc / +0x200 move params (0x204d48 layout)
    pub mv: [f32; 4],
    /// WP-P22: the controlled Bentley / Murray's PO fields (None: Sly). Set each frame by the player layer
    /// from the class memory; the JT stand / run forces are then the class's FUN_00204108 (p22).
    pub po_move: Option<p22::PoMove>,
    pub grounded: bool,
    /// +0x20c clock time of the last ground contact (0x204768). 0x18a8c8 jts 0/1: walk off when
    /// t - +0x20c > 0.1 (FUN_00195a00(0)).
    pub t_ground: f32,
    /// JMP +0x28: double jump available
    pub f_28: bool,
    /// ledge top while hanging (jtbs 0x11)
    pub ledge: Option<V3>,
    /// recorded (jts, jtbs) transitions
    pub history: Vec<(i32, i32)>,
    pub t: f32,
    // ---- WP-P3 ----
    /// jt +0x150 joystick object
    pub joy: Joy,
    /// pad bits for the next update (held, pressed) set by `feed_pad`; None = derive from JtInput
    pub pad_pending: Option<(u32, u32)>,
    /// JMP +0x04 jump kind (FUN_00195a00 param_2 unless 6)
    pub kind: i32,
    /// JMP +0x08 (FUN_00195a00 param_3; 0x197370 animation direction 1..4)
    pub jmp_idx: i32,
    /// JMP +0x18 launch time
    pub t_launch: f32,
    /// JMP +0x24 short hop (0x197500 case 0: X released before 0.15 s), consumed by 0x197f98 (unported)
    pub f_24: bool,
    /// JMP +0x2c = +0x28 at launch
    pub f_2c: bool,
    /// JMP +0x30 direction animation chosen (0x197370)
    pub f_30: bool,
    /// JMP +0x38 second scale (raw bits kept: case 0xe stores (int)1.75 = 1 = 1.4e-45, quirk)
    pub s_38: f32,
    /// JMP +0x3c launch z (FUN_001a62e8 position z)
    pub z_launch: f32,
    /// JMP +0x40 (3 = no-stick push / boost, -1 case 10)
    pub f_40: i32,
    /// JMP +0x44 / +0x48 / +0x4c timers
    pub t_44: [f32; 3],
    /// jt +0x1a8 facing yaw (REPLACED source: last stick direction)
    pub yaw: f32,
    /// jt +0x238: ground contact z (+0x230 vector, 0x204768 = contact +0x50). REPLACED: centre - n*r of
    /// the best ground normal from the driver.
    pub z_contact: f32,
    /// jt +0x208 ground contact normal z (None = no contact this frame)
    pub ground_nz: Option<f32>,
    /// jt +0x248 height used by the action gate FUN_00143d20 (writer not traced: f32::MAX = gate open)
    pub h_248: f32,
    /// (jt+0x58)+0xa8: end time of the current state ASEG (set by the anim driver; f32::MAX = never)
    pub t_aseg_end: f32,
    /// unported callees reached (address, arg)
    pub calls: Vec<(u32, i32)>,
    // ---- WP-P4 ----
    /// DAT_003e1b08 / DAT_003e1b10 target list (FUN_00194a40)
    pub targets: Vec<JtTarget>,
    /// WP-K33: the category-list objects of 0x18b7e8's FUN_00194a40(world, jt, 0xffff), filled by the game
    /// layer (None = not driven: `targets` is left as set, headless tests)
    pub cands: Option<Vec<p4::TargetCand>>,
    /// WP-K33: target vt+0x19c (slot 50)(target, jt) calls of 0x1911a8 kind 7 (pickpocket / use point), drained
    /// by the game layer (`player::plugin::loot_link`)
    pub slot50: Vec<usize>,
    /// WP-K33: a recorded kind-7 attach waiting for its first jts-7 frame (0x1911a8)
    pub k7_pending: bool,
    /// WP-PJ1 jt +0x270: count of the attached-item list (+0x274..), FUN_00144068 busy test
    pub n_270: i32,
    /// WP-PJ1 jt +0x1084: held / carried object (FUN_00143de8 sets it), FUN_00144068 busy test
    pub held_1084: Option<usize>,
    /// jt+0x4c0 attach object
    pub attach: Attach,
    /// jt+0x1000 target category mask (TARGET +0x1c test)
    pub mask_1000: u32,
    /// jt == DAT_002e1e40 (the player Sly)
    pub is_player: bool,
    /// last ledge found by FUN_00199d48
    pub ledge_hit: Option<LedgeHit>,
    // ---- WP-P5 ----
    /// last step (DAT_002dded4)
    pub dt: f32,
    /// port-owned fields of the P5 states (p5_states) and the engine-side inputs filled per frame
    pub p5: p5_states::P5,
    pub p5w: p5_states::P5World,
    /// p5 / p5w are taken out of the JT while a p5_states function runs (see `with_p5`)
    pub p5_busy: bool,
    /// attach fields beyond `attach` (p5_attach) and the inputs of 0x1911a8 the port does not own
    pub att: p5_attach::AttachExt,
    pub att_in: p5_attach::AttachInput,
    /// target data of the current attach (copied from `p5_targets` at attach time; refreshed per frame)
    pub att_tg: Option<p5_attach::AttachTarget>,
    /// attach events of this frame (engine side effects applied by the JT are also listed here)
    pub att_ev: Vec<p5_attach::AttachEv>,
    /// JMP flight fields (0x195fb8 / xf kind 0x11)
    pub flight: p5_target::JmpFlight,
    /// air / attach target registry (category-list objects of 0x19b110 + attach data per object)
    pub p5_targets: Vec<P5Target>,
    /// Sly's convex hull (GEOM 199, p5_hull); None = P4 sphere contacts
    pub hull: Option<p5_hull::JtHull>,
    /// jt+0x1020 reach radius / jt+0x1f30 surface standoff of 0x19b110 (REPLACED: writers not traced)
    pub reach: f32,
    pub standoff: f32,
    // ---- WP-P7 ----
    /// jt+0xdf0 damage component (health, damage states, flash)
    pub dmg: p7_damage::Dmg,
    /// world-side inputs of the damage code (difficulty, spawn, job)
    pub dmg_env: p7_damage::DmgEnv,
    /// level reload requested by the death (0x161a58 case 0x10), consumed by the game
    pub respawn: Option<p7_damage::Respawn>,
    // ---- WP-P8 ----
    /// jt+0x108c / +0x10a0 / ALO +0x110 (hazards)
    pub p8: p8_hazard::JtP8,
    // ---- WP-P11 ----
    /// jt+0x18c0..+0x19b0 cane sub-state, cane ASEG and hit geom (p11_cane)
    pub cane: p11_cane::Cane,
    // ---- WP-P13 ----
    /// jt+0x10dc..+0x10ec focus object, jt+0x1d30 (p13)
    pub p13: p13::JtP13,
    /// WP-P14: jts 8 pipe climb
    pub p14: p14::JtP14,
    /// WP-P15: jt+0xf00 binocucom (0x19cd38 state machine)
    pub binoc: p15_binoc::Binoc,
    /// WP-P16: jts 0xd / 0xe / 0x11 state clock, cane swing sounds and job events for the plugin
    pub p16: p16::JtP16,
    /// WP-P18: cane aim targets / jt+0x18c0
    pub p18: p18::JtP18,
    /// WP-P25: jt+0x1ff0 jts 6 block (FUN_001371b8)
    pub j6: p25::Jts6,
    /// jt+0x1010 vector decayed by FUN_0011f088 in the ledge hang (writer of non-zero values not traced)
    pub v_1010: V3,
    /// WP-P18: gadget slots / power (jt+0x1180, +0x12fc..+0x1308)
    pub gad: p18_gadget::Gadgets,
    /// JMP +0x1c0 / +0x1c4 glide speed targets (0x197f98 case 0x14)
    pub gad_glide: [f32; 2],
    /// WP-P24: jt+0x1024 = jmp[0x6f] (glider camera track 0x5f4, FUN_00140c88) while gliding (0x195fb8 0x13 / 0x14)
    pub cam_glide: bool,
    /// WP-P24: DAT_002de3a0 (C2cd4e0 look yaw, CM 0x2ddf40 + 0x460) as read by the binocucom (0x204660):
    /// the yaw of the CM forward row, set by the game layer each frame (REPLACED: the look policy's own
    /// +0x10 is not shared; activate copies it from that row and the look drives the row)
    pub cam_yaw: Option<f32>,
    /// WP-PJ2b: jt+0x19b8 / +0x19bc class-0x35 sub objects: ((obj+0xa8)->+0x60)+0x20 pose offset vector
    /// (written by SetDmgState 0x1a0b40)
    pub pose_19b8: [f32; 4],
    pub pose_19bc: [f32; 4],
    /// WP-SCR2: FUN_00143ee0 slot-87 pose - jt+0x1030 anim handle (FUN_001305b8), jt+0x1034 state-machine
    /// link record (FUN_001f1448), and the slot-87 current index (vt+0x2c4 writes; FUN_00144038 is
    /// (idx - 5) < 2). The k22 wait condition reads `pose_87_idx` in {5, 6} for the slot-88 10 / 11 test;
    /// Bentley mirrors it on +0x1084 but Sly reads it from the same field as slot-87.
    pub pose_1030: u32,
    pub pose_1034: u32,
    pub pose_87_idx: i32,
    /// WP-PJ2b: DAT_002ddfd0 camera position (CM +0x90), set by the game layer each frame (None = unknown)
    pub cam_pos: Option<V3>,
}

/// One object of the 0x19b110 category lists with its attach data (REPLACED: the game's object lists).
#[derive(Clone, Debug)]
pub struct P5Target {
    pub obj: p5_target::TargetObj,
    /// None: the object cannot be attached to (FUN_001928f8 stays recorded)
    pub attach: Option<p5_attach::AttachTarget>,
}

impl P5Target {
    /// Category list object from attach data: path + is-a 10 -> rope list (kind 0xa), is-a 0x26 -> angle list
    /// with a fixed member point (kind 0xc -> attach kind 0), is-a 0x13 -> point list (kind 0xe swing).
    /// Other classes (poles 0x22 need the member axis, rails 0xc / 0xb, surfaces) return None: build the
    /// `TargetObj` explicitly.
    pub fn from_attach(id: usize, mask: u32, range: f32, tg: p5_attach::AttachTarget) -> Option<Self> {
        use p5_target::TargetGeom;
        let c = tg.class;
        let geom = if let (Some(p), true) = (tg.path.as_ref(), p4::FUN_00132b10(c, 10)) {
            TargetGeom::Rope { path: target_path(p) }
        } else if p4::FUN_00132b10(c, 0x26) {
            // WP-P14: class 0x26 registers in DAT_003e1408 (0x1836c8) = category I (kind 0xd), member u = 0
            TargetGeom::Spot0xd { pt: tg.pos }
        } else if p4::FUN_00132b10(c, 0x13) {
            TargetGeom::Point { pos: tg.pos, f_e8: true, is_13: true }
        } else {
            return None;
        };
        let obj = p5_target::TargetObj { id, mask, range, vel: [0.0; 3], geom };
        Some(Self { obj, attach: Some(tg) })
    }
}

/// Adapter: p5_attach path (rows + translation) -> p5_target path (M4 rows 0..2 + translation row).
pub fn target_path(p: &p5_attach::Path) -> p5_target::Path {
    let mut t = p5_target::Path::polyline(p.pts.clone(), [p.rot[0], p.rot[1], p.rot[2], p.pos]);
    t.spline = p.kind == 1;
    t.looped = p.looped;
    t.s = p.s.clone();
    t.l = p.l.clone();
    t
}

/// Adapter: P4 target slot (position + class only) -> 0x19b110 category object. P4 slots carry no member
/// geometry, so only point-like classes convert (0x13 point list, 0x26 fixed member point); ropes / poles /
/// rails / surfaces need `P5Target` data (GAP: TargetCand has no geometry).
pub fn target_obj_from_p4(t: &JtTarget) -> Option<p5_target::TargetObj> {
    use p5_target::TargetGeom;
    let geom = if p4::FUN_00132b10(t.class, 0x13) {
        TargetGeom::Point { pos: t.pos, f_e8: true, is_13: true }
    } else if p4::FUN_00132b10(t.class, 0x26) {
        TargetGeom::Spot0xd { pt: t.pos }
    } else {
        return None;
    };
    Some(p5_target::TargetObj { id: t.obj, mask: u32::MAX, range: p4::DAT_002BF664, vel: [0.0; 3], geom })
}

fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn scale(a: V3, s: f32) -> V3 {
    [a[0] * s, a[1] * s, a[2] * s]
}

impl Jt {
    pub fn new(pos: V3) -> Self {
        Self {
            phys: AloPhys { pos, ..Default::default() },
            jts: JTS_STAND,
            jtbs: JTBS_NIL,
            t_jtbs: 0.0,
            v_target: [0.0; 3],
            air_scale: 1.0,
            // 0x204d48 defaults scaled by RUN_SPEED / 25 (REPLACED: real per-state values not traced)
            mv: [RUN_SPEED, 0.0, 0.25 * (25.0f32 / RUN_SPEED).sqrt(), 1.0],
            po_move: None,
            grounded: false,
            t_ground: f32::MIN,
            f_28: false,
            ledge: None,
            history: vec![(JTS_STAND, JTBS_NIL)],
            t: 0.0,
            joy: Joy::default(),
            pad_pending: None,
            kind: 0,
            jmp_idx: 0,
            t_launch: 0.0,
            f_24: false,
            f_2c: false,
            f_30: false,
            s_38: 1.0,
            z_launch: pos[2],
            f_40: 0,
            t_44: [0.0; 3],
            yaw: 0.0,
            z_contact: pos[2] - JT_RADIUS,
            ground_nz: None,
            h_248: f32::MAX,
            t_aseg_end: f32::MAX,
            calls: Vec::new(),
            targets: Vec::new(),
            cands: None,
            slot50: Vec::new(),
            k7_pending: false,
            n_270: 0,
            held_1084: None,
            attach: Attach::default(),
            mask_1000: u32::MAX,
            is_player: true,
            ledge_hit: None,
            dt: 1.0 / 60.0,
            p5: p5_states::P5::default(),
            p5w: p5_states::P5World::default(),
            p5_busy: false,
            att: p5_attach::AttachExt::default(),
            att_in: p5_attach::AttachInput::default(),
            att_tg: None,
            att_ev: Vec::new(),
            flight: p5_target::JmpFlight::default(),
            p5_targets: Vec::new(),
            hull: None,
            reach: JT_RADIUS,
            standoff: 20.0,
            dmg: p7_damage::Dmg::default(),
            dmg_env: p7_damage::DmgEnv::default(),
            respawn: None,
            p8: p8_hazard::JtP8::default(),
            cane: p11_cane::Cane::default(),
            p13: p13::JtP13::default(),
            p14: p14::JtP14 { mode: -1, wt: [1.0, 0.0], ..Default::default() },
            binoc: p15_binoc::Binoc::default(),
            p16: p16::JtP16::default(),
            p18: p18::JtP18::default(),
            j6: p25::Jts6::default(),
            v_1010: [0.0; 3],
            gad: p18_gadget::Gadgets::default(),
            gad_glide: [0.0; 2],
            cam_glide: false,
            cam_yaw: None,
            pose_19b8: [0.0; 4],
            pose_19bc: [0.0; 4],
            pose_1030: 0,
            pose_1034: 0,
            pose_87_idx: 0,
            cam_pos: None,
        }
    }

    /// Pad bits (buttons held, pressed this frame) for the next `update` (plugin path).
    pub fn feed_pad(&mut self, pad: &PadState) {
        self.pad_pending = Some((pad.buttons, pad.pressed));
    }

    /// 0x0018c250 SetJtJts(jt, jts, jtbs): early-out when unchanged; jts 0 entry clears +0x190..+0x198;
    /// jtbs change -> 0x195fb8; anim field per state.
    pub fn FUN_0018c250(&mut self, jts: i32, jtbs: i32, anim: &mut dyn JtAnim) {
        if self.jts == jts && self.jtbs == jtbs {
            return;
        }
        if self.jts == 7 && jts != 7 {
            // WP-P5: leaving the attach state -> FUN_00192bd8(a, -1) (detach)
            self.detach();
        }
        if self.jts != jts {
            if jts == JTS_STAND {
                // +0x190/+0x194/+0x198 = 0 (velocity goal)
                self.v_target = [0.0; 3];
            }
            self.jts = jts;
            self.p14.t_18cc = self.t;
            if jts == JTS_RUN {
                anim.play_aseg(0x1324, jts, jtbs);
            }
        }
        if self.jtbs != jtbs {
            self.FUN_00195fb8(jtbs, anim);
            if (0..=2).contains(&jtbs) {
                anim.play_aseg(0x1994, jts, jtbs);
            }
            // WP-P12: 0x00195fb8 tail (LAB_00196844 -> FUN_001305b8 into jt+0x1030), issued after the jump
            // blend so the anim driver ends on the JMP body ASEG like the original's last write
            if let Some(f) = jmp_body_aseg_195fb8(jtbs, self.jmp_idx) {
                anim.play_aseg(f, self.jts, jtbs);
            }
        }
        self.history.push((self.jts, self.jtbs));
    }

    /// 0x00195fb8 (JMP set body state) bookkeeping: +0x1c = jtbs, +0x20 = clock t. WP-P5: exits of 8..0x10 /
    /// 0x12 and entries of 9..0x10 / 0x12 run p5_target::FUN_00195fb8 (xf kind 0x11, kind-4 launch, ASEG);
    /// an O pick (`FUN_0019b110`) already ran it through `apply_pick` with the target.
    fn FUN_00195fb8(&mut self, jtbs: i32, anim: &mut dyn JtAnim) {
        if jtbs == self.jtbs {
            return;
        }
        // WP-P24: exit 0x13 / 0x14 (not to 0x13 / 0x14): glider SM jt+0x1d28 -> 0x5f8 (FUN_001f0660), FUN_00140c88(jt, 0)
        if matches!(self.jtbs, 0x13 | 0x14) && !matches!(jtbs, 0x13 | 0x14) {
            self.calls.push((0x001f0660, 0x5f8));
            self.cam_glide = false;
        }
        // entry 0x13: FUN_00140c88(jt, jmp[0x6f]) + track params +4 (follow_camera), glider SM -> 0x5f7
        if jtbs == 0x13 {
            self.cam_glide = true;
            self.calls.push((0x001f0660, 0x5f7));
        }
        let old = self.flight.jtbs;
        let flight = |s: i32| matches!(s, 8..=0x10 | 0x12);
        if old != jtbs && (flight(old) || (flight(jtbs) && jtbs != 8)) {
            let v = self.velocity();
            let fx = p5_target::FUN_00195fb8(&mut self.flight, jtbs, None, v, self.phys.gravity[2], self.t, self.is_player);
            self.apply_fx(&fx, jtbs, anim);
        } else {
            self.flight.jtbs = jtbs;
            self.flight.t_jtbs = self.t;
        }
        if matches!(jtbs, 9..=0x10 | 0x12) {
            // jmp+0x28 = 0, +0x8 = 0 (0x195fb8 entries)
            self.f_28 = false;
            self.jmp_idx = 0;
        }
        self.jtbs = jtbs;
        self.t_jtbs = self.t;
        if jtbs == JTBS_DJUMP_INIT || jtbs == JTBS_DJUMP_RISE {
            self.f_28 = false;
        }
    }

    /// 0x00195fb8 reached from JMP code while in jts 2 (state stays the jump).
    fn set_jtbs(&mut self, jtbs: i32, anim: &mut dyn JtAnim) {
        if self.jts == JTS_JUMP {
            self.FUN_0018c250(JTS_JUMP, jtbs, anim);
        } else {
            // vtable +0x2c4(jt, 1): full SetJtJts (exit actions of the old state)
            self.set_jts(JTS_JUMP, jtbs, anim);
        }
    }

    /// jt +0x218 z - jt +0x238 (see FUN_0019a750).
    fn h_rel(&self) -> f32 {
        self.phys.pos[2] - self.z_contact
    }

    /// vtable +0x11c SetVelocity: velocity delta through the pending impulse (consumed by the driver 0x1f9e70).
    pub fn set_velocity_pub(&mut self, v: V3) {
        self.set_velocity(v)
    }
    pub(crate) fn velocity_pub(&self) -> V3 {
        self.velocity()
    }
    fn set_velocity(&mut self, v: V3) {
        self.phys.imp_lin = add(self.phys.imp_lin, sub(v, add(self.phys.v, self.phys.imp_lin)));
    }
    fn velocity(&self) -> V3 {
        add(self.phys.v, self.phys.imp_lin)
    }

    /// 0x00195a00(jmp, kind, idx, _): start a jump of `kind` (see Findings P3 for the case table).
    pub fn FUN_00195a00(&mut self, kind: i32, idx: i32, anim: &mut dyn JtAnim) {
        if kind != 6 {
            self.kind = kind;
        }
        self.jmp_idx = idx;
        // vtable +0x2c4(jt, 1): enter the jump state (jts 2)
        if kind != 6 {
            self.f_40 = 0;
            self.t_44 = [self.t; 3];
        }
        // 0x19a750(jmp, kind): kind passed through ($a1), incl. 6 for the double jump
        let mut v = self.velocity();
        v[2] = FUN_0019a750(kind, v[2], self.phys.gravity[2], self.h_rel());
        self.set_velocity(v);
        if kind != 6 {
            self.t_launch = self.t;
        }
        let jtbs;
        match kind {
            0 => {
                jtbs = 4;
                (self.f_28, self.s_38, self.f_24, self.air_scale) = (true, 1.0, false, 1.0);
            }
            1 | 2 | 0x12 => {
                jtbs = 0;
                (self.f_28, self.s_38, self.f_24, self.air_scale) = (true, 1.0, false, 1.0);
            }
            3 => {
                if self.joy.mag < 0.2 {
                    self.f_40 = 3;
                    let p = FUN_0011e1a0(self.yaw, DAT_002BF754, 0.0);
                    self.set_velocity(add(v, p));
                }
                jtbs = 0;
                (self.f_28, self.air_scale, self.s_38, self.f_24) = (true, 1.0, 0.6, false);
            }
            6 => {
                jtbs = if self.kind < 0xf && 0xc < self.kind { 3 } else { 2 };
            }
            7 => {
                jtbs = 7;
                (self.f_28, self.air_scale, self.f_24, self.s_38) = (true, 1.0, false, 1.0);
            }
            8 => {
                // FUN_0011ef08(v, +0xb0, +0xb8, 0) not ported; +0xb4 = 1.0
                self.calls.push((0x0011ef08, 8));
                jtbs = 5;
                (self.f_24, self.air_scale, self.s_38, self.f_28) = (false, 1.0, 1.0, true);
            }
            9 => {
                // +0x168 = jt+0x260, FUN_00204da8(0.75), FUN_00140c88(jt, +0x16c), +0x174 = 1: not ported
                self.calls.push((0x00140c88, 9));
                jtbs = 0;
                (self.f_24, self.f_28, self.s_38, self.air_scale) = (false, true, 1.5, 1.2);
            }
            10 => {
                self.f_40 = -1;
                jtbs = 0x15;
                (self.f_24, self.f_28, self.air_scale, self.s_38) = (false, false, 0.0, 0.0);
            }
            0xb => {
                jtbs = 4;
                (self.f_24, self.s_38, self.f_28, self.air_scale) = (true, 1.0, false, 1.0);
            }
            0xc => {
                jtbs = 1;
                self.f_40 = 3;
                (self.f_28, self.s_38, self.f_24, self.air_scale) = (true, 1.0, false, 1.0);
            }
            0xd => {
                jtbs = 1;
                (self.f_28, self.s_38, self.air_scale, self.f_24) = (true, 1.0, 1.0, false);
            }
            0xe => {
                jtbs = 1;
                // piVar10[0xe] = (int)DAT_002bf760: integer 1 stored into a float slot (quirk kept)
                (self.f_28, self.air_scale, self.f_24, self.s_38) = (true, 1.0, false, f32::from_bits(1));
                if self.joy.mag < 0.2 {
                    self.f_40 = 3;
                    let p = FUN_0011e1a0(self.joy.angle, RUN_SPEED * DAT_002BF760, 0.0);
                    self.set_velocity(add(v, p));
                }
            }
            0xf => {
                jtbs = 0;
                (self.f_24, self.s_38, self.f_28, self.air_scale) = (false, 1.0, false, 1.0);
            }
            0x10 => {
                jtbs = 4;
                (self.f_24, self.s_38, self.f_28, self.air_scale) = (false, 1.0, false, 1.0);
            }
            0x11 => {
                // +0x1c == -1 branch (velocity projection on DAT_002a2dc0 + DAT_002bf770): not reached by the port
                jtbs = 0x16;
                (self.f_24, self.s_38, self.f_2c, self.air_scale) = (false, 1.0, false, 1.0);
            }
            _ => {
                jtbs = self.jtbs;
            }
        }
        if kind != 6 {
            if kind != 0x11 {
                self.f_2c = self.f_28;
            }
            // FUN_00132b10(jt, 0x35) -> +0x18f0 = -1 (class test, no effect on the port)
            self.z_launch = self.phys.pos[2];
        }
        self.set_jtbs(jtbs, anim);
    }

    /// 0x00143d20(dt, jt): action gate. h = +0x248 + dt vz + dt^2/2 g; h <= 0 -> FUN_00173610(0, h, &pos, 0, 0)
    /// (WP-P4: ground within the predicted drop -> true = action blocked, e.g. no double jump just before landing).
    pub fn FUN_00143d20(&self, cw: &CollisionWorld, d: f32) -> bool {
        let h = self.h_248 + d * self.velocity()[2] + d * d * 0.5 * self.phys.gravity[2];
        if h <= 0.0 {
            let mut p = self.phys.pos;
            return Self::FUN_00173610(cw, 0.0, h, &mut p, false);
        }
        false
    }

    /// 0x0019c828: target lost -> jump off (0x11) or fall.
    pub fn FUN_0019c828(&mut self, anim: &mut dyn JtAnim) {
        if self.jtbs == JTBS_LEDGE {
            self.FUN_00195a00(3, 0, anim);
        } else {
            self.set_jtbs(4, anim);
        }
    }

    /// 0x0019c688: JMP state timer expired (jtbs 6, 9..0x10, 0x12). Attach cases run FUN_001928f8 (jt+0x4c0,
    /// kind, jmp+0xc target, member): real p5_attach::attach_to when the target is in `p5_targets` with attach
    /// data, else recorded. 0xe / 0xf / 0x10 go through p5_states::FUN_0019c688_p5.
    pub fn FUN_0019c688(&mut self, anim: &mut dyn JtAnim) {
        let tgt = self.flight.target;
        match self.jtbs {
            // WP-P14: category F path -> jts 8 ride (0x19eab0 / 0x19e110 with the target path)
            0xf if !self.p5_busy => self.jts8_enter(anim),
            0xe if !self.p5_busy => {
                self.p14_swing_target();
                let dt = self.dt;
                self.with_p5(|jt, st, w| p5_states::FUN_0019c688_p5(jt, st, w, dt, &mut *anim));
            }
            0xe | 0xf | 0x10 if !self.p5_busy => {
                let dt = self.dt;
                self.with_p5(|jt, st, w| p5_states::FUN_0019c688_p5(jt, st, w, dt, &mut *anim));
            }
            6 => self.FUN_001928f8(1, tgt, 0, anim),
            9 => self.FUN_001928f8(4, tgt, 0, anim),
            10 => self.FUN_001928f8(3, tgt, 0, anim),
            0xb | 0xc => self.FUN_001928f8(0, tgt, 0, anim),
            0xd => {
                // kind = target +0x10c
                let k = self.target_attach(tgt).map_or(-1, |t| t.f_10c);
                self.FUN_001928f8(k, tgt, 0, anim)
            }
            0xe => {
                // +0x18c4 = target; target +0x124 == 3 -> SetJtJts(0x16, -1) else SetJtJts(3, 1)
                self.FUN_0018c250(3, 1, anim);
            }
            0xf => self.calls.push((0x2c4, 8)), // +0xf5c/+0xf60 = target, vtable +0x2c4(jt, 8)
            0x10 => self.FUN_0018c250(0x15, 10, anim),
            0x12 => self.calls.push((0x00135248, 8)),
            _ => self.set_jtbs(4, anim),
        }
    }

    /// 0x00197370 (direction animation pick): +0x30 = 1, +0x8 = 1..4 from the stick angle relative to
    /// the facing (|d| < 1.0995574 front 3, > 2.042035 back 4, d > 0 2, else 1). The ASEG play is recorded.
    fn FUN_00197370(&mut self) {
        self.f_30 = true;
        let mut d = self.joy.angle - self.yaw;
        // FUN_0011cf60: wrap to [-pi, pi]
        while d > std::f32::consts::PI {
            d -= std::f32::consts::TAU;
        }
        while d < -std::f32::consts::PI {
            d += std::f32::consts::TAU;
        }
        self.jmp_idx = if d.abs() < 1.0995574 {
            3
        } else if 2.042035 < d.abs() {
            4
        } else if 0.0 < d {
            2
        } else {
            1
        };
        self.calls.push((CALL_ANIM_DIR, self.jmp_idx));
    }

    /// Input step 0x18b7e8 -> 0x18a8c8 (jts 0/1/2 parts, see module doc).
    fn input(&mut self, cw: &CollisionWorld, mut inp: JtInput, anim: &mut dyn JtAnim) {
        // WP-K33: 0x18b7e8 -> FUN_00194a40(&DAT_003dd490, jt, 0xffff) before 0x18b2c0
        if let Some(c) = self.cands.take() {
            self.FUN_00194a40(&c, 0xffff);
            self.cands = Some(c);
        }
        // WP-K33: 0x1911a8 kind 7 of a recorded attach (the attach state body is unported, see p4 FUN_001928f8):
        // drop (SetJtJts(jt, 0) -> FUN_00192bd8(-1)) + target vt+0x19c (slot 50)(target, jt), queued in `slot50`
        if std::mem::take(&mut self.k7_pending) && self.attach.kind == 7 {
            if let Some(t) = self.attach.target.take() {
                self.slot50.push(t);
            }
            self.calls.push((0x00192bd8, -1));
            self.attach.sub = -1;
        }
        // WP-P15: 0x18b7e8 -> 0x18b2c0 binocucom before 0x18a8c8 (mode 0 clears the stick)
        if self.FUN_0018b2c0(anim) {
            inp.stick = [0.0; 2];
        }
        let m = (inp.stick[0] * inp.stick[0] + inp.stick[1] * inp.stick[1]).sqrt().min(1.0);
        let dir = if m > 1e-4 { [inp.stick[1] / m, -inp.stick[0] / m] } else { [0.0, 0.0] };
        // Z-up: stick y = +x (forward), stick x = -y (left axis is +y)
        self.v_target = [dir[0] * m * self.mv[0], dir[1] * m * self.mv[0], 0.0];
        // joy +0x18 / +0x1c (REPLACED: from the camera-relative stick)
        self.joy.mag = m;
        if m > 1e-4 {
            self.joy.angle = dir[1].atan2(dir[0]);
            // WP-P18: jtbs 0x11 / 0x13 / 0x14 steer in 0x197f98 (no snap; 0x14 = glide_steer)
            let glide = self.jts == JTS_JUMP && matches!(self.jtbs, 0x13 | 0x14);
            if self.jts != 7 && self.jts != 8 && self.jts != p7_damage::JTS_DMG && !glide {
                // jts 7: the facing is owned by 0x191a10 (attach rows)
                self.yaw = self.joy.angle;
            }
        }
        // WP-P18: 0x18a0c8 (jts 0) / 0x197f98 (jts 2) vt+0x2e4 = FUN_00189458: an aim target owns the facing
        if self.jts == JTS_STAND || self.jts == JTS_JUMP {
            self.FUN_00189458();
        }
        if self.jts == JTS_JUMP && self.jtbs == 0x14 {
            self.glide_steer(); // WP-P18: 0x197f98 case 0x14
        }
        // WP-P7: 0x18b7e8 -> 0x1a1358 damage flash / state-7 turn
        let env = self.dmg_env;
        p7_damage::FUN_001a1358(self, &env, anim);
        p7_damage::exit_sync(self, anim);
        // 0x18a8c8 restarts its switch while jts/jtbs change
        for _ in 0..8 {
            let (jts0, jtbs0) = (self.jts, self.jtbs);
            match self.jts {
                JTS_STAND | JTS_RUN => {
                    // +0x18d8 - 8 > 1 (not grabbing) and t - +0x20c > 0.1 -> FUN_00195a00(0) walk off
                    if 0.1 < self.t - self.t_ground {
                        self.FUN_00195a00(0, 0, anim);
                    } else if self.FUN_0018a8c8_stand_head(anim) {
                        // WP-P13: +0x1d30 -> (0x11, -1) / cane recoil blocks the input
                    } else if self.joy.FUN_001c46c0(self.t, 0.1, BTN_CROSS) != 0 {
                        self.FUN_00195a00(1, 0, anim);
                    } else if self.FUN_0018a8c8_stand_jts5(anim) {
                        // WP-P13: R1 up + FUN_00140af8(jt, 0) -> SetJtJts(5, 4)
                    } else if self.joy.FUN_001c4660(self.t, 0.15, p5_attach::JOY_ATTACK) != 0 && !self.binoc.active() {
                        // WP-P15: R3 + vt+0x31c(jt, 3) -> 0x1c4738(0x400), FUN_0019cc50(jt+0xf00, 0, 0, 0)
                        self.binoc_r3(anim);
                    } else if self.joy.FUN_001c46c0(self.t, 0.2, BTN_CIRCLE) != 0 {
                        // FUN_001938c0(jt+0x4c0, 0xffff, 0): ground cane attack over the target list (WP-P4)
                        self.calls.push((CALL_GROUND_ATTACK, 0xffff));
                        self.FUN_001938c0(anim);
                    } else {
                        let min = if self.jts == JTS_STAND { 0.1 } else { 0.0 };
                        // REPLACED: jts 0xc (no R1) / 1 (R1 held) both drive the 0x204108 run here
                        self.set_jts(if m > min { JTS_RUN } else { JTS_STAND }, JTBS_NIL, anim);
                    }
                }
                JTS_JUMP => self.FUN_00197500(cw, m, dir, anim),
                // WP-P14: 0x18a8c8 jts 8 -> 0x19e1e0 (pipe climb input)
                8 if !self.p5_busy => self.FUN_0019e1e0(anim),
                // WP-P14: 0x18a8c8 case 0x15 / 0x16 (X jump off the surface / ride point)
                0x15 => self.FUN_0018a8c8_15(anim),
                0x16 => self.FUN_0018a8c8_16(anim),
                // WP-P13: 0x18a8c8 case 5
                5 => self.FUN_0018a8c8_jts5(m, anim),
                // WP-P5: 0x18a8c8 jts 3 -> 0x190520 (swing input), jts 7 -> 0x1911a8 (attach input)
                // WP-P7: 0x18a8c8 jts 4 -> 0x1a12f0 (continuations when the state ASEG is done)
                p7_damage::JTS_DMG => p7_damage::FUN_001a12f0(self, &env, anim),
                // WP-P14: 0x18a8c8 case 3 head: 0.25 s after entering, X -> FUN_00195a00(2)
                3 if !self.p5_busy && 0.25 < self.t - self.p14.t_18cc && self.joy.FUN_001c46c0(self.t, 0.25, BTN_CROSS) != 0 => {
                    self.FUN_00195a00(2, 0, anim);
                }
                3 if !self.p5_busy => {
                    let dt = self.dt;
                    self.with_p5(|jt, st, w| p5_states::FUN_00190520(jt, st, w, dt, &mut *anim));
                }
                7 => {
                    let tg = self.cur_attach_target();
                    let mut x = std::mem::take(&mut self.att);
                    let inp = self.att_in;
                    let mut ev = Vec::new();
                    p5_attach::FUN_001911a8(self, &mut x, tg.as_ref(), &inp, &mut ev);
                    self.att = x;
                    self.apply_attach_ev(ev, anim);
                }
                // WP-P16: 0x18a8c8 cases 0xd / 0xe / 0x11
                0xd => self.FUN_0018a8c8_d(anim),
                0xe => self.FUN_0018a8c8_e(anim),
                0x11 => self.FUN_0018a8c8_11(anim),
                // WP-P25: 0x18a8c8 case 6 -> 0x1371c8
                6 => self.FUN_001371c8(anim),
                _ => {}
            }
            if self.jts == jts0 && self.jtbs == jtbs0 {
                break;
            }
            if self.jts == JTS_JUMP && jts0 == JTS_JUMP {
                break; // 0x197500 runs its own restart loop
            }
        }
        // WP-P11: 0x18b7e8 -> 0x18f0a8 cane input after 0x18a8c8
        self.FUN_0018f0a8(anim);
        // WP-P7: leaving jts 4 clears the damage state (0x18c250 case 4)
        p7_damage::exit_sync(self, anim);
        // WP-EP1f: 0x18b7e8 -> 0x19fc30 (after 0x1a1358) records the safe points 0x19f590 warps back to
        self.FUN_0019fc30();
    }

    /// WP-EP1f: 0x0019fc30. jts 0 1 5 6 9 10 0xb 0xc 0xf 0x10 0x11 0x14 0x17 0x18: a ground contact (+0x208)
    /// with normal z > 0.7, SO +0x148 bit 29 clear and no water (+0x108c) -> 0x19feb0(jt+0x9e0, contact SO,
    /// contact point +0x60). REPLACED: the contact SO / its +0x148 bit are not modelled (object None, bit
    /// clear); contact point = (x, y, +0x238). UNPORTED (recorded): jts 3 / 0x16 0x19fd30 (rail / pipe
    /// parameter points), jts 7 0x1942f0, jts 8 0x19ea60.
    pub fn FUN_0019fc30(&mut self) {
        match self.jts {
            0 | 1 | 5 | 6 | 9 | 10 | 0xb | 0xc | 0xf | 0x10 | 0x11 | 0x14 | 0x17 | 0x18 => {
                if self.ground_nz.is_some_and(|z| 0.7 < z) && self.p8.water.is_none() {
                    let p = [self.phys.pos[0], self.phys.pos[1], self.z_contact];
                    p5_states::FUN_0019feb0(&mut self.p5.safe_points, None, p);
                }
            }
            3 | 0x16 => self.calls.push((0x0019fd30, self.jts)),
            7 => self.calls.push((0x001942f0, 0)),
            8 => self.calls.push((0x0019ea60, 0)),
            _ => {}
        }
    }

    /// 0x00197500 JMP update (see module doc). Restarts while jtbs changes (do/while on +0x1c).
    fn FUN_00197500(&mut self, cw: &CollisionWorld, m: f32, _dir: [f32; 2], anim: &mut dyn JtAnim) {
        let ground = self.ground_nz.is_some_and(|z| 0.7 < z); // bVar24: +0x208 normal z > 0.7
        for _ in 0..8 {
            let jtbs0 = self.jtbs;
            // +0xc target bit-60 test -> FUN_0019c828 (p5_states pre check, P5World::jmp_target_active)
            if !self.p5_busy && self.with_p5(|jt, _, w| p5_states::FUN_00197500_pre(jt, w, &mut *anim)) {
                if self.jtbs == jtbs0 || self.jtbs == -1 || self.jts != JTS_JUMP {
                    return;
                }
                continue;
            }
            if !(9..0x17).contains(&self.jtbs) {
                if self.f_28 && self.joy.FUN_001c4660(self.t, 0.0, BTN_CROSS) != 0 && !self.FUN_00143d20(cw, 0.1) {
                    self.joy.FUN_001c4738(BTN_CROSS);
                    let idx = self.jmp_idx;
                    self.FUN_00195a00(6, idx, anim);
                }
                if self.joy.FUN_001c4660(self.t, 0.0, BTN_CIRCLE) != 0 {
                    let j = self.jtbs;
                    // FUN_0019b110(jmp, 0): O air target pick (WP-P5 p5_target; the call stays recorded)
                    self.calls.push((CALL_AIR_ATTACK, 0));
                    self.FUN_0019b110(cw, None, anim);
                    if self.jtbs != j {
                        self.joy.FUN_001c4738(BTN_CIRCLE);
                    }
                }
                // WP-P18: R1 (8) with jt == DAT_002e1e40 -> paraglider 0x1b (FUN_001daea8, FUN_00143d20(0.3))
                self.FUN_00197500_r1(cw, anim);
            }
            let dt_state = self.t - self.t_jtbs;
            match self.jtbs {
                0 | 2 => {
                    let t_init = if self.jtbs == 0 { T_JUMP_INIT } else { T_DJUMP_INIT };
                    if t_init < dt_state {
                        if self.jtbs == 0 && self.joy.held & BTN_CROSS == 0 && self.f_40 != 3 {
                            self.f_24 = true; // short hop
                        }
                        let n = if self.jtbs == 0 { JTBS_JUMP_RISE } else { JTBS_DJUMP_RISE };
                        self.set_jtbs(n, anim);
                    } else if !self.f_30 && 0.5 < self.joy.mag {
                        self.FUN_00197370();
                    }
                }
                1 | 7 => {
                    if self.velocity()[2] < 0.0 {
                        self.set_jtbs(JTBS_JUMP_FALL, anim);
                    }
                }
                3 | 4 => {
                    if !ground {
                        // FUN_00199d48 (WP-P4: real contact-edge search, kinds 0xf/0x10 never grab)
                        if let Some(h) = self.FUN_00199d48(cw) {
                            self.ledge = Some(h.contact.pos);
                            if p15_anim::anim_log() {
                                eprintln!("P17 ledge: pos {:?} v {:?} hit {:?}", self.phys.pos, self.phys.v, h);
                            }
                            // WP-P17: FUN_00195fb8 case 0x11: jt+0x1010 = hang point - SO position (the
                            // FUN_0011f088-decayed offset of the hang update)
                            self.v_1010 = sub(p4::ledge_hang_point(h.contact.pos, h.dir), self.phys.pos);
                            self.ledge_hit = Some(h);
                            self.set_jtbs(JTBS_LEDGE, anim);
                        } else if !self.p5_busy {
                            // airborne without a ledge -> case-5 tail (stuck test 0x19a098 -> warp 0x19f590)
                            self.with_p5(|jt, st, w| p5_states::FUN_00197500_tail5(jt, st, w, &mut *anim));
                        }
                    } else if self.kind == 0xe {
                        // FUN_001928f8(jt+0x4c0, 10, 0, 0)
                        self.FUN_001928f8(10, None, 0, anim);
                    } else {
                        // vtable +0x2c4(jt, 0): leave the jump
                        self.set_jts(if m > 1e-4 { JTS_RUN } else { JTS_STAND }, JTBS_NIL, anim);
                    }
                }
                JTBS_LEDGE => {
                    if T_LEDGE_JUMP < dt_state && self.joy.FUN_001c46c0(self.t, 0.25, BTN_CROSS) != 0 {
                        self.ledge = None;
                        self.FUN_00195a00(3, 0, anim);
                    } else if let Some(h) = self.ledge_hit {
                        // WP-P8: rest of 0x197500 case 0x11 (0x197a6c..0x197b94). JMP +0xc8 contact object +4 ->
                        // +0xec != 0 (still in the contact list): the port's colliders are static -> always true.
                        // WP-P11: jt+0x1010 = FUN_0011f088(dt, jt+0x1010, DAT_002a2dd0, 0x2bf820) decay.
                        self.v_1010 = p5_attach::FUN_0011f088(self.dt, self.v_1010, [0.0; 3], p11_cane::DAT_002BF820, &mut [0.0; 3]);
                        // FUN_0022fc48(obj, 0, jmp+0xd0): static colliders -> local == world.
                        let mut dir = h.dir;
                        let mut held = true;
                        if 0.03 < dt_state {
                            // FUN_001999d0(jmp, +0xc8, pos, +0x100 kind, +0x104 feature, &+0xe0 dir) == 0 -> jtbs 4
                            if let Some(d) = self.FUN_001999d0(cw, &h.contact) {
                                dir = d;
                                self.ledge_hit = Some(LedgeHit { dir: d, ..h });
                            } else {
                                held = false;
                                self.ledge = None;
                                self.set_jtbs(4, anim); // LAB_00197f18: FUN_00195fb8(jmp, 4, 0)
                            }
                        }
                        if held {
                            // FUN_002046b0(jt, -dir): prot3w + FUN_002458f0 -> 0x204660 yaw = atan2(-dir.y, -dir.x)
                            self.yaw = (-dir[1]).atan2(-dir[0]);
                            self.att.rows = crate::engine::physics::driver::FUN_0011b500([0.0, 0.0, 1.0], self.yaw);
                            if self.joy.FUN_001c4660(self.t, 0.15, p5_attach::JOY_ATTACK) != 0 && !self.binoc.active() {
                                // vt+0x31c(jt + vt+0x318) can-leave (false while 0x1443d8)
                                self.binoc_r3(anim); // WP-P15: FUN_0019cc50(jt+0xf00, 0, 0, 0)
                            }
                        }
                    }
                }
                // WP-P5: 5, 6, 8, 9..0x10, 0x12..0x16 (p5_states, 1:1 bodies)
                _ if !self.p5_busy => {
                    let dt = self.dt;
                    self.with_p5(|jt, st, w| p5_states::FUN_00197500_p5(jt, st, w, dt, &mut *anim));
                }
                _ => {}
            }
            if self.jtbs == jtbs0 || self.jtbs == -1 || self.jts != JTS_JUMP {
                return;
            }
        }
    }

    /// 0x00188960 JT slot69 forces -> world acceleration (gravity is added by the driver's 0x1f9b20).
    pub fn FUN_00188960(&self, dt: f32) -> V3 {
        self.forces(dt).lin
    }

    /// 0x00188960 with the angular part (WP-P5): jts 3 / 4 / 6 / 8 / 0xe / 0x18 bodies from p5_states (+ the
    /// 0x204108 base for 4 / 6 / 0xe / 0x18), jts 7 from the attach goal (REPLACED contact motor 0x2044b0:
    /// velocity driven to `AttachExt::goal_world` this step, gravity cancelled) + 0x194170 for kind 3 edge ropes.
    pub fn forces(&self, dt: f32) -> p5_states::Acc {
        let lin = |v: V3| p5_states::Acc { lin: v, ..Default::default() };
        // WP-P22: Bentley / Murray slot 69: FUN_00204108 of the class PO fields on the ground states; the
        // jump component (+0x2c0, same code as the JT's 0x196df8) adds its force to the angular servo
        if let Some(m) = &self.po_move {
            let po = p22::ee_sub_204108(&self.phys, m, self.grounded, dt);
            match self.jts {
                // WP-P24: the linear part is the E8 contact motor (`p22::po_motor` in the driver step)
                JTS_STAND | JTS_RUN => return p5_states::Acc { ang: po.ang, set_w: po.set_w, ..Default::default() },
                JTS_JUMP => {
                    let mut a = lin(if self.jtbs == 0x14 { self.glide_force(dt) } else { self.FUN_00196df8(dt) });
                    a.ang = po.ang;
                    return a;
                }
                _ => {}
            }
        }
        match self.jts {
            // WP-P18: 0x196df8 case 0x14 (paraglider)
            JTS_JUMP if self.jtbs == 0x14 => lin(self.glide_force(dt)),
            JTS_JUMP => lin(self.FUN_00196df8(dt)),
            JTS_STAND | JTS_RUN => lin(self.FUN_00204108(dt)),
            3 | 4 | 6 | 8 | 0xe | 0x18 => {
                let mut a = p5_states::FUN_00188960_p5(self, &self.p5, &self.p5w, dt).unwrap_or_default();
                if matches!(self.jts, 4 | 6 | 0xe | 0x18) {
                    a.lin = add(a.lin, self.FUN_00204108(dt));
                }
                a
            }
            7 => {
                let goal = self.att.goal_world();
                let mut a = sub(scale(sub(goal, self.phys.v), 1.0 / dt), self.phys.gravity);
                if let Some(tg) = self.cur_attach_target().filter(|t| t.f_1bc != 0) {
                    let d = p5_attach::rope_dir(&self.att, &tg);
                    let e = self.FUN_00194170(d, tg.f_1c0, tg.f_1c4, dt);
                    // WP-P14 REPLACED (edge / slide rope): the attach constraint of the original only holds the
                    // JT on the path, so the velocity goal acts across the edge direction only and the motion
                    // along it is gravity + the 0x194170 force (along-edge accel = clamp(t1c4 * g.d) by speed)
                    if self.attach.kind == 3 {
                        let ad = a[0] * d[0] + a[1] * d[1] + a[2] * d[2];
                        a = sub(a, scale(d, ad));
                    }
                    a = add(a, e);
                }
                lin(a)
            }
            // 0x188960: 0x15 / 0x16 nothing (driven by 0x185e40 / 0x18df60)
            0x15 | 0x16 => lin([0.0; 3]),
            _ => lin(self.FUN_00204108(dt)),
        }
    }

    /// 0x00204108 (+0x1a4 == 0): FUN_001f59a0(jt, +0x1b0, {+0x1f4*+0x200, +0x1f8*+0x200, +0x1fc}).
    fn FUN_00204108(&self, dt: f32) -> V3 {
        let p = [self.mv[0] * self.mv[3], self.mv[1] * self.mv[3], self.mv[2]];
        let s = p[0];
        let accel = (s - p[1]) / p[2]; // fVar5
        let v = self.phys.v;
        let hv = [v[0], v[1], 0.0];
        let sp = dot(v, v).sqrt();
        let mut a = [0.0; 3];
        // 0x1f59a0: over-speed damping a += v * (s - |v|) * 2.5 / |v|
        if s < sp {
            a = scale(v, (s - sp) * 2.5 / sp);
        }
        // REPLACED steering: velocity error, magnitude limited to accel (1/dt when it can be reached this frame)
        let e = sub(self.v_target, add(hv, scale(a, dt)));
        let el = dot(e, e).sqrt();
        let k = if el < accel * dt { 1.0 / dt } else { accel / el };
        add(a, scale(e, k))
    }

    /// 0x00196df8 default case: local frame from the horizontal velocity direction (0x11b7e0), target velocity
    /// in that frame, gains, scale +0x34 (kind 0xc: time polynomial, unported), terminal fall drag, 0x1fac30.
    /// Cases 6, 8..0x13, 0x15 add nothing; 0x14 / 0x16 (attached states) unported.
    fn FUN_00196df8(&self, dt: f32) -> V3 {
        if matches!(self.jtbs, 6 | 8..=0x16) {
            return [0.0; 3];
        }
        let v = self.phys.v;
        let s = (v[0] * v[0] + v[1] * v[1]).sqrt();
        let fwd = if s >= 1e-4 { [v[0] / s, v[1] / s, 0.0] } else { [1.0, 0.0, 0.0] };
        let left = [-fwd[1], fwd[0], 0.0];
        let lx = dot(fwd, self.v_target);
        let ly = dot(left, self.v_target);
        let (mut gx, mut gy) = (1.0 / dt, 1.0 / dt);
        if self.jtbs != 0 && self.jtbs != 2 {
            gx = if s <= lx { AIR_GAIN_X_SLOW } else { AIR_GAIN_X_FAST };
            gy = AIR_GAIN_Y;
        }
        let f = [gx * (lx - s) * self.air_scale, gy * ly * self.air_scale];
        let mut a = add(scale(fwd, f[0]), scale(left, f[1]));
        if self.jtbs != 5 && v[2] < FALL_LIMIT {
            a[2] += FALL_DRAG * (v[2] - FALL_LIMIT);
        }
        a
    }

    /// One frame: input, JT forces, physics driver (WP-E5), ground state from contact normals.
    pub fn update(&mut self, cw: &CollisionWorld, inp: JtInput, dt: f32, anim: &mut dyn JtAnim) {
        self.t += dt;
        self.dt = dt;
        self.att_ev.clear();
        let (held, pressed) = match self.pad_pending.take() {
            Some(p) => p,
            None => {
                let b = if inp.jump { BTN_CROSS } else { 0 };
                (b, b)
            }
        };
        let pressed = if inp.jump { pressed | BTN_CROSS } else { pressed };
        self.joy.feed(self.t, held, pressed);
        // WP-P18: 0x1403d0 (JT parent update, before the input step) -> 0x141a50 gadget slot buttons
        self.FUN_00141a50(anim);
        // P5World inputs of jtbs 0x13 / 0x14 / 0x16: FUN_00141ea8(jt, 0x1b), FUN_001441a0 power
        self.p5w.mask_1b = self.gad.FUN_00141ea8(p18_gadget::GADGET_PARAGLIDER);
        self.p5w.f_1441a0 = self.gad.FUN_001441a0() as f32;
        self.p5w.mask_c = self.gad.FUN_00141ea8(0xc);
        // WP-P25: jt+0x10dc orbit target of 0x137248 (REPLACED: position only, velocity 0)
        self.p5w.orbit = self.p13.pos_10dc.filter(|_| self.p13.f_10dc != 0).map(|pos| p5_states::Kin { pos, v: [0.0; 3] });
        // 0x197500 case 0x13 reads jt+0x1030 (nothing else writes this view)
        // WP-P19: FUN_00131968(jt+0x1030) of the glide-open ASEG jt+0x438 (done at once when not loaded)
        self.p5w.anim.done = self.jts == JTS_JUMP
            && self.jtbs == 0x13
            && self.cane.data.asegs.get(&0x438).map_or(true, |a| a.len <= self.t - self.t_jtbs);
        self.input(cw, inp, anim);
        // WP-P11: cane ASEG (jt+0x18e0) events / end
        self.cane_step(dt, anim);
        // WP-P13: jts 5 body ASEG messages (REPLACED jt+0x1030 msg 9 / msg 3)
        self.jts5_main_step(dt, anim);
        // WP-P5 0x18bce0 per-jts update (jts 3: 0x190e18 here, its 0x190520 input ran in `input`)
        match self.jts {
            3 => {
                let dt = self.dt;
                self.with_p5(|jt, st, w| p5_states::FUN_00190e18(jt, st, w, dt));
            }
            7 => {
                // 0x18b7e8 -> 0x191a10 attach update (REPLACED body yaw 0x204740: the facing turns instantly)
                let tg = self.cur_attach_target();
                let mut x = std::mem::take(&mut self.att);
                x.body_yaw = self.yaw;
                let mut ev = Vec::new();
                p5_attach::FUN_00191a10(self, &mut x, tg.as_ref(), dt, &mut ev);
                self.att = x;
                self.apply_attach_ev(ev, anim);
            }
            _ => {
                // WP-P25: 0x18b7e8 case 6 -> 0x137218 (face the jt+0x10dc target)
                if self.jts == 6 {
                    self.FUN_00137218();
                }
                let dt = self.dt;
                self.with_p5(|jt, st, w| p5_states::FUN_0018bce0_p5(jt, st, w, None, dt, &mut *anim));
            }
        }
        // WP-P14: jts 8 rides the path (REPLACED constraint, no dynamic step)
        if self.jts8_step() {
            return;
        }
        if self.jts == JTS_JUMP && matches!(self.jtbs, 9..=0x10 | 0x12) && self.flight.xf.kind == 0x11 {
            // xf kind 0x11 Hermite drive (0x22de88 case 0x11) replaces the dynamic step
            self.flight.xf.pos = self.phys.pos;
            self.flight.xf.vel = self.velocity();
            if p5_target::FUN_00197f98_abort(&self.flight.xf, self.t, dt) {
                self.set_jtbs(JTBS_JUMP_FALL, anim);
            } else {
                let (p, v) = p5_target::FUN_0022de88_case11(&mut self.flight.xf, self.t, dt);
                (self.phys.pos, self.phys.v, self.phys.imp_lin) = (p, v, [0.0; 3]);
                self.ground_nz = None;
                self.grounded = false;
                if p5_target::arrived(&self.flight.xf, self.t) {
                    self.FUN_0019c688(anim);
                }
                return;
            }
        }
        if self.jtbs == JTBS_LEDGE {
            // WP-P17: JT slot 0x140d00 -> FUN_001991a0: vtable +0x294(jt, 7, contact object, hang point, 0, SO pos)
            // holds the body at the hang point (REPLACED: the type-7 constraint by placing the SO there; the
            // colliders are static). Before P17 the body was frozen where the ledge was found (mid-air).
            // WP-P18: vtable +0x294 = JT slot 81 FUN_00140c30(jt, 7, obj, pt): contact object +0x28 bit 60 clear
            // -> FUN_00195a00(0) (static colliders: always set), else FUN_00205008(jt, 7, obj, pt - jt+0x1010, pos)
            // = x/y/z constraints (0x1a7c18) at pt - jt+0x1010. jt+0x1010 (= hang - pos at 0x195fb8 entry) decays to
            // 0 in 0x197500 case 0x11 (0x11f088), so the body eases from the grab position onto the hang point.
            // It is not a render offset (readers: 0x140c30, 0x195fb8, 0x197500, pipe 0x19e1e0 / 0x19eab0).
            if let Some(h) = self.ledge_hit {
                self.phys.pos = sub(p4::ledge_hang_point(h.contact.pos, h.dir), self.v_1010);
            }
            self.phys.v = [0.0; 3];
            self.phys.imp_lin = [0.0; 3];
            return;
        }
        // REPLACED +0x248 writer: height of the sphere bottom above the floor below (FUN_00143d20)
        let bottom = self.phys.pos[2] - JT_RADIUS;
        self.h_248 = cw
            .clip_segment([self.phys.pos[0], self.phys.pos[1], bottom], [self.phys.pos[0], self.phys.pos[1], bottom - 5000.0])
            .iter()
            .find(|l| l.t[0] > 0.0)
            .map_or(f32::MAX, |l| bottom - l.pos[0][2]);
        let acc = self.forces(dt);
        let a = acc.lin;
        let motor = self.po_move.map(|m| p22::po_motor(&self.phys, &m));
        if motor.is_none() {
            self.phys.imp_lin = add(self.phys.imp_lin, scale(a, dt));
        }
        self.phys.imp_ang = add(self.phys.imp_ang, scale(acc.ang, dt));
        if let Some(w) = acc.set_w {
            self.phys.imp_ang = sub(w, self.phys.w);
        }
        if self.phys.kind == 3 {
            self.phys.kind = 1; // the player never sleeps while controlled (0x230598 wake)
        }
        let normals = match motor {
            // WP-P24: Bentley / Murray: one 0x1f8dd8 frame through the E8 PO contact motor (0x204ac8 kind-0xe
            // record + 0x2044b0 + 0x1f9410 mode 1); P0 = jt+0x218 = the sphere bottom, level hull = the JT sphere
            // (REPLACED hull source, as the JT's own driver step). The slot-69 linear force goes in as accel.
            Some(m) => {
                let st = crate::engine::physics::motor::po_step(cw, &mut self.phys, &m, [0.0, 0.0, -JT_RADIUS], p22::PO_PROBE, Some(([0.0; 3], JT_RADIUS)), a, dt);
                st.xps.iter().map(|x| x.n).collect()
            }
            None => driver::FUN_001f8dd8(cw, &mut self.phys, JT_RADIUS, dt),
        };
        // 0x204768: +0x208 best ground contact, +0x20c = t, +0x230 = contact point
        let best = normals.iter().copied().fold(None::<V3>, |b, n| match b {
            Some(b) if b[2] >= n[2] => Some(b),
            _ => Some(n),
        });
        self.ground_nz = best.map(|n| n[2]);
        if let Some(n) = best.filter(|n| n[2] > 0.7) {
            self.t_ground = self.t;
            self.z_contact = self.phys.pos[2] - n[2] * JT_RADIUS;
        }
        self.grounded = self.t - self.t_ground <= GROUND_GRACE;
        // contact normals of this step for the next 0x19a098 stuck test (P5World input)
        self.p5w.contact_normals = normals;
    }
}

// ============================================================================================ WP-P5 glue
impl Jt {
    /// Runs a p5_states function with the JT's P5 state / world taken out (`p5_busy` while it runs: nested
    /// SetJtJts from inside p5 code fall back to the raw bookkeeping, as p5_states expects).
    pub fn with_p5<R>(&mut self, f: impl FnOnce(&mut Jt, &mut p5_states::P5, &p5_states::P5World) -> R) -> R {
        let mut st = std::mem::take(&mut self.p5);
        let w = std::mem::take(&mut self.p5w);
        self.p5_busy = true;
        let r = f(self, &mut st, &w);
        self.p5_busy = false;
        self.p5 = st;
        self.p5w = w;
        r
    }

    /// 0x0018c250 SetJtJts with the P5 exit / enter / jtbs actions (p5_states::FUN_0018c250_p5); the raw
    /// bookkeeping `FUN_0018c250` is what p5 code itself calls.
    pub fn set_jts(&mut self, jts: i32, jtbs: i32, anim: &mut dyn JtAnim) {
        if self.p5_busy {
            self.FUN_0018c250(jts, jtbs, anim);
            return;
        }
        let dt = self.dt;
        self.with_p5(|jt, st, w| p5_states::FUN_0018c250_p5(jt, st, w, jts, jtbs, dt, &mut *anim));
    }

    /// WP-SCR2: FUN_00143ee0 for Sly (class 53) - one of the three branches resolved by
    /// `EngineHost::k22_pose_143ee0`. REPLACED: the slot-87 index set (vt+0x2c4) is kept as the `pose_87_idx`
    /// field here - Sly's per-state slot-87 writer (states 10 / 11 enter / exit) is not traced.
    pub fn ee_sub_143ee0(
        &mut self,
        req: &crate::engine::classes::plugin::PlayerPoseReq,
        h: &mut dyn crate::engine::classes::ClassHost,
    ) {
        use crate::engine::classes::guard::PLAYER_HANDLE;
        use crate::engine::classes::plugin::PlayerPoseKind;
        use crate::engine::classes::shared::anim::{ee_sub_130658, ee_sub_1305b8};
        match req.kind {
            PlayerPoseKind::Clear => {
                // FUN_00143ff8: slot-88 state 10 / 11 -> slot-87 idx 0
                if (self.pose_87_idx as u32).wrapping_sub(5) < 2 {
                    self.pose_87_idx = 0;
                }
            }
            PlayerPoseKind::Sm { obj, flags } => {
                self.pose_87_idx = req.idx as i32;
                ee_sub_130658(&mut self.pose_1030, h);
                crate::engine::classes::k17_55::ee_sub_1f1448(
                    obj,
                    PLAYER_HANDLE,
                    -1,
                    flags,
                    &mut self.pose_1034,
                    h,
                );
            }
            PlayerPoseKind::Anim { obj, flags } => {
                self.pose_87_idx = req.idx as i32;
                h.ee_sub_1f15b8(&mut self.pose_1034);
                ee_sub_1305b8(0, 1.0, obj, PLAYER_HANDLE, flags, &mut self.pose_1030, h);
            }
        }
    }

    /// WP-SCR2: FUN_00144038(Sly) - slot-88 index 5 / 6 (states 10 / 11): the pose is active. Mirrored
    /// onto `pose_87_idx` here.
    pub fn posing_144038(&self) -> bool {
        (self.pose_87_idx as u32).wrapping_sub(5) < 2
    }

    /// Attach data of a registry object.
    pub fn target_attach(&self, id: Option<usize>) -> Option<p5_attach::AttachTarget> {
        let id = id?;
        self.p5_targets.iter().find(|t| t.obj.id == id).and_then(|t| t.attach.clone())
    }

    /// Current attach target (registry entry of `attach.target`, else the copy taken at attach time).
    pub fn cur_attach_target(&self) -> Option<p5_attach::AttachTarget> {
        self.target_attach(self.attach.target).or_else(|| self.att_tg.clone())
    }

    /// FUN_001928f8 with the real FUN_00192bd8 (p5_attach::attach_to) and its events applied.
    pub fn attach_real(&mut self, kind: i32, target: Option<usize>, member: usize, tg: p5_attach::AttachTarget, anim: &mut dyn JtAnim) {
        let mut x = std::mem::take(&mut self.att);
        let mut ev = Vec::new();
        p5_attach::attach_to(self, &mut x, Some(&tg), kind, target, member, &mut ev);
        self.att = x;
        self.att_tg = Some(tg);
        self.apply_attach_ev(ev, anim);
    }

    /// Leaving jts 7: FUN_00192bd8(a, -1). Its events are side effects only (no state changes).
    fn detach(&mut self) {
        if self.attach.sub == -1 && self.attach.target.is_none() {
            return;
        }
        let tg = self.cur_attach_target();
        let mut x = std::mem::take(&mut self.att);
        let mut ev = Vec::new();
        p5_attach::FUN_00192bd8(self, &mut x, tg.as_ref(), -1, &mut ev);
        self.att = x;
        self.att_tg = None;
        for e in ev {
            self.att_ev.push(e);
            self.record_attach_ev(e);
        }
    }

    fn record_attach_ev(&mut self, e: p5_attach::AttachEv) {
        use p5_attach::AttachEv as E;
        match e {
            E::Call(a, k) => self.calls.push((a, k)),
            E::Notify(m) => self.calls.push((0x1c, m)),
            E::Snd { off, .. } => self.calls.push((0x001f1448, off as i32)),
            E::Vis(m) => self.calls.push((0x0018c120, m as i32)),
            E::Rider(on) => self.calls.push((if on { 0x001333c0 } else { 0x00133448 }, 0)),
            E::IgnoreCollision(on) => self.calls.push((0x00204f78, on as i32)),
            _ => {}
        }
    }

    /// Engine side of the attach events (`AttachEv`): SetJtJts table 0x18d568 (`p7_damage::FUN_0018d568`, P7;
    /// unknown m -> m), FUN_00195a00 jumps, velocity / translate sets, ASEG starts through `JtAnim`; the rest is recorded.
    pub fn apply_attach_ev(&mut self, ev: Vec<p5_attach::AttachEv>, anim: &mut dyn JtAnim) {
        use p5_attach::AttachEv as E;
        for e in ev {
            self.att_ev.push(e);
            match e {
                E::SetJtJts { m } => {
                    let jts = p7_damage::FUN_0018d568(m).unwrap_or(m);
                    self.set_jts(jts, JTBS_NIL, anim);
                }
                E::Jump { kind, idx } => self.FUN_00195a00(kind, idx, anim),
                E::SetVelocity(v) => self.set_velocity(v),
                E::SetAngVelocity(w) => self.phys.imp_ang = sub(w, self.phys.w),
                E::Translate(d) => self.phys.pos = add(self.phys.pos, d),
                // WP-P15: 0x1911a8 R3 -> FUN_0019cc50(jt+0xf00, 0, 0, 0)
                E::Call(0x0019cc50, _) => self.FUN_0019cc50(0, 0, 0, anim),
                // WP-P15: `off` is attach-relative (a = jt+0x4c0) unless jt_rel; the JtAnim field is a JT offset
                E::Anim { off, jt_rel, .. } => anim.play_aseg(if jt_rel { off } else { ATTACH_BASE + off }, self.jts, self.jtbs),
                e => self.record_attach_ev(e),
            }
        }
    }

    /// Side effects of p5_target::FUN_00195fb8.
    fn apply_fx(&mut self, fx: &[p5_target::Fx], jtbs: i32, anim: &mut dyn JtAnim) {
        use p5_target::Fx;
        for f in fx {
            match *f {
                Fx::XfKind(_) => {} // kept in `flight.xf.kind`
                Fx::Clear190 => {
                    self.v_target = [0.0; 3];
                    self.att.v190 = [0.0; 3];
                }
                Fx::SetVelocity(v) => self.set_velocity(v),
                // WP-P23: a JMP-relative field is a JT member at JMP (jt+0x2c0) + field (0x120 -> jt+0x3e0,
                // 0x148 -> +0x408, 0x14c -> +0x40c); before P23 it was requested as a JT offset (unbound)
                Fx::Anim { field, jmp_field, .. } => anim.play_aseg(if jmp_field { 0x2c0 + field } else { field }, self.jts, jtbs),
                Fx::Call(a, k) => self.calls.push((a, k)),
                Fx::CallObj(a, o) => self.calls.push((a, o as i32)),
                Fx::ExitUnported(_) | Fx::EntryUnported(_) => {}
            }
        }
    }

    /// Category objects for 0x19b110: the registry, then P4 target slots not in it (`target_obj_from_p4`).
    pub fn pick_candidates(&self) -> Vec<p5_target::TargetObj> {
        let mut v: Vec<_> = self.p5_targets.iter().map(|t| t.obj.clone()).collect();
        for t in &self.targets {
            if t.member != usize::MAX && !v.iter().any(|o| o.id == t.obj) {
                v.extend(target_obj_from_p4(t));
            }
        }
        v
    }

    /// 0x0019b110(jmp, forced): O air target pick (p5_target, 1:1) + its tail (xf aim / deadline, 0x195fb8).
    /// Collision clearance through the level `CollisionWorld`. Returns true when a flight started.
    pub fn FUN_0019b110(&mut self, cw: &CollisionWorld, forced: Option<usize>, anim: &mut dyn JtAnim) -> bool {
        let objs = self.pick_candidates();
        if objs.is_empty() && forced.is_none() {
            return false;
        }
        let vel = self.velocity();
        let inp = p5_target::PickIn {
            t: self.t,
            dt: self.dt,
            pos: self.phys.pos,
            vel,
            centre: self.phys.pos,
            hofs: self.p5.hofs_240,
            g: self.phys.gravity,
            reach: self.reach,
            standoff: self.standoff,
            mask: self.mask_1000,
            is_35: true,
            is_37: false,
            last: self.flight.last,
            rope: p5_target::RopeIn { a2a0: self.att.rope_vmax, a260: self.att.rope_v0, joy_angle: self.joy.angle, joy_mag: self.joy.mag },
            partner: None,
            targets: &objs,
        };
        let Some(out) = p5_target::FUN_0019b110(&inp, forced, cw) else { return false };
        self.flight.jtbs = self.jtbs;
        let fx = p5_target::apply_pick(&mut self.flight, &out, vel, self.phys.gravity[2], self.t, self.is_player);
        self.flight.xf.pos = self.phys.pos;
        self.apply_fx(&fx, out.kind, anim);
        // flight already entered: the bookkeeping 0x195fb8 sees flight.jtbs == kind
        self.FUN_0018c250(JTS_JUMP, out.kind, anim);
        true
    }
}

#[cfg(test)]
mod p5_integration_tests {
    use super::*;

    const DT: f32 = 1.0 / 60.0;

    fn step(jt: &mut Jt, cw: &CollisionWorld, stick: [f32; 2], held: u32, pressed: u32) {
        jt.pad_pending = Some((held, pressed));
        jt.update(cw, JtInput { stick, jump: false }, DT, &mut ());
    }

    fn airborne(pos: V3, v: V3) -> Jt {
        let mut jt = Jt::new(pos);
        jt.t = 10.0;
        jt.set_jts(JTS_JUMP, JTBS_JUMP_FALL, &mut ());
        jt.set_velocity(v);
        jt
    }

    /// (a) O in the air near a point (class 0x26, list I since WP-P14) -> jtbs 0xd flight -> arrives
    /// at the deadline -> FUN_0019c688 -> FUN_001928f8(target +0x10c = 0) -> jts 7 hanging on the point while O is held.
    #[test]
    fn land_on_point() {
        let cw = CollisionWorld::default();
        let mut jt = airborne([0.0, 0.0, 200.0], [300.0, 0.0, 200.0]);
        jt.mask_1000 = 1;
        let mut tg = p5_attach::AttachTarget::new(0x26, [150.0, 0.0, 150.0]);
        tg.f_10c = 0;
        jt.p5_targets.push(P5Target::from_attach(7, 1, 500.0, tg).unwrap());
        step(&mut jt, &cw, [0.0; 2], BTN_CIRCLE, BTN_CIRCLE);
        assert!(jt.calls.contains(&(CALL_AIR_ATTACK, 0)));
        assert_eq!((jt.jts, jt.jtbs), (JTS_JUMP, 0xd), "O pick enters the point flight");
        assert_eq!(jt.flight.xf.kind, 0x11);
        let mut n = 0;
        while jt.jts == JTS_JUMP {
            step(&mut jt, &cw, [0.0; 2], BTN_CIRCLE, 0);
            n += 1;
            assert!(n < 400, "flight never arrived: jtbs {}", jt.jtbs);
        }
        assert_eq!(jt.jts, 7, "arrival attaches");
        assert_eq!(jt.attach.target, Some(7));
        // hang: the attach goal pulls the JT onto the hang point below the point while O is held
        for _ in 0..60 {
            step(&mut jt, &cw, [0.0; 2], BTN_CIRCLE, 0);
        }
        assert_eq!(jt.jts, 7);
        let d = sub(jt.phys.pos, [150.0, 0.0, 150.0]);
        assert!(dot(d, d).sqrt() < 120.0, "hanging near the point: {:?}", jt.phys.pos);
    }

    /// (b) attach to a rope, ride it with the stick, X jumps off (jts 7 -> 2, detach).
    #[test]
    fn ride_rope() {
        let cw = CollisionWorld::default();
        let mut tg = p5_attach::AttachTarget::new(10, [0.0; 3]);
        tg.path = Some(p5_attach::Path::polyline(vec![[0.0, 0.0, 100.0], [400.0, 0.0, 100.0]], false));
        let mut jt = Jt::new([50.0, 0.0, 100.0]);
        jt.t = 10.0;
        jt.att.rope_vmax = 300.0;
        // GAP: the kind-3 blend velocities come from the rope ASEG (0x1a0450 unported); supplied here
        jt.att.rope_blend = p5_attach::Blend2 { on: true, k: [1.0, 1.0], vel: [[0.0; 3], [200.0, 0.0, 0.0]], aseg: true, ..Default::default() };
        jt.p5_targets.push(P5Target { obj: P5Target::from_attach(3, 1, 500.0, tg.clone()).unwrap().obj, attach: Some(tg) });
        jt.FUN_001928f8(3, Some(3), 0, &mut ());
        assert_eq!(jt.jts, 7);
        assert_eq!(jt.attach.target, Some(3));
        let x0 = jt.phys.pos[0];
        for _ in 0..60 {
            step(&mut jt, &cw, [0.0, 1.0], 0, 0);
            assert_eq!(jt.jts, 7);
        }
        assert!(jt.phys.pos[0] > x0 + 20.0, "moved along the rope: {:?}", jt.phys.pos);
        assert!(jt.phys.pos[1].abs() < 5.0 && (jt.phys.pos[2] - 100.0).abs() < 30.0, "{:?}", jt.phys.pos);
        step(&mut jt, &cw, [0.0; 2], BTN_CROSS, BTN_CROSS);
        assert_eq!(jt.jts, JTS_JUMP, "X jumps off");
        assert_eq!((jt.attach.kind, jt.attach.target), (-1, None), "detached");
    }

    /// (c) pole climb: attach to a round pole (class 0x22), the stick moves the JT around it on the hang circle.
    #[test]
    fn climb_pole() {
        let cw = CollisionWorld::default();
        let mut tg = p5_attach::AttachTarget::new(0x22, [0.0, 0.0, 100.0]);
        tg.r_10c = 20.0;
        let mut jt = Jt::new([50.0, 0.0, 100.0]);
        jt.t = 10.0;
        jt.att.hang_r = 30.0;
        jt.att.b8_vel_100 = [100.0, 0.0, 0.0];
        jt.p5_targets.push(P5Target { obj: p5_target::TargetObj { id: 1, mask: 1, range: 500.0, vel: [0.0; 3], geom: p5_target::TargetGeom::Point { pos: tg.pos, f_e8: true, is_13: false } }, attach: Some(tg) });
        jt.FUN_001928f8(0, Some(1), 0, &mut ());
        assert_eq!(jt.jts, 7);
        let s0 = jt.att.s20;
        for _ in 0..40 {
            step(&mut jt, &cw, [-1.0, 0.0], BTN_CIRCLE, 0);
        }
        assert_eq!(jt.jts, 7);
        assert!(jt.att.s20 > s0, "moved around the pole: s {} -> {}", s0, jt.att.s20);
        let r = (jt.phys.pos[0].powi(2) + jt.phys.pos[1].powi(2)).sqrt();
        assert!((r - 50.0).abs() < 10.0, "stays on the hang circle: {r}");
    }

    /// (d) wedged in a V in jtbs 5 (no ground): the 0x19a098 stuck test fires 1 s after entry and the JT is
    /// warped to the safe point (0x19f590 teleport branch), falling (jtbs 4).
    #[test]
    fn stuck_respawn() {
        let cw = CollisionWorld::default();
        let mut jt = Jt::new([0.0, 0.0, 100.0]);
        jt.t = 10.0;
        jt.FUN_00195a00(8, 0, &mut ());
        assert_eq!((jt.jts, jt.jtbs), (JTS_JUMP, 5));
        jt.p5.safe_points = vec![p5_states::SafePoint { pos: [0.0, 0.0, 2100.0], ..Default::default() }];
        let mut warped = false;
        for _ in 0..90 {
            // the driver's contacts of the V walls (no level geometry here)
            jt.phys.v = [0.0; 3];
            jt.phys.pos = if warped { jt.phys.pos } else { [0.0, 0.0, 100.0] };
            jt.p5w.contact_normals = vec![[0.8, 0.0, 0.6], [-0.8, 0.0, 0.6]];
            step(&mut jt, &cw, [0.0; 2], 0, 0);
            if jt.calls.contains(&(0x0014d8c0, 0)) {
                warped = true;
                break;
            }
        }
        assert!(warped, "stuck test never fired: jtbs {}", jt.jtbs);
        assert_eq!((jt.jts, jt.jtbs), (JTS_JUMP, JTBS_JUMP_FALL));
        assert!(jt.phys.pos[2] > 2000.0, "{:?}", jt.phys.pos);
    }
}
