//! WP-P7: Sly's damage component (jt+0xdf0, "DMG"), health, death and respawn.
//!
//! Traced from the JT vtable (C2d2e98): slot 74 0x140f50 -> 0x1a19a0 TakeDamage(jt+0xdf0, hit), slot 75
//! 0x1a21a0 -> 0x140ed8 -> 0x1a2230 immunity test, slot 76 0x140f70 -> 0x1a23c0 damage status, slot 85
//! 0x144418 lethal-hit test, slot 92 0x1a2388 die (0x1f80d0 + 0x140eb0 = event 0x10 on the job manager
//! 0x2deb30). Health lives in the save variable "hpCur" (jt+0xe00 -> int, 0x1440d0 / 0x144188), maximum
//! jt+0xdfc = DMG+0xc = 0x28 (0x1a0b18). Sly 2 has no lives counter: death = event 0x10 -> 0x161a58: no job
//! active -> FUN_0022b490(world, current level, world+0x3c88, world+0x3c8c, DAT_003e12c8 ? 0x20 : 0, 0x30a)
//! = reload the current level at the spawn the level was entered with (world+0x3c88/+0x3c8c is copied from
//! the request's +0x3c38/+0x3c3c by 0x20e890 when the spawn object was found; 0x162a60 resets it to
//! 0x189 / -1 when a job ends). The reloaded JT runs 0x1400b8 -> 0x1440d0: hpCur == 0 -> hp = max.
//!
//! 1:1: 0x1a19a0 (lethal path -0x8000, slot 75 veto, +0x4c8 == 1 delegation reported, difficulty scale
//! 0x155228, heal 0x100000 incl. the lethal-heal -> kind 0x200 conversion, class-0x38 halving, jts 4 entry
//! vt+0x2c4(3) = 0x18d568 table, event 0xf, hit copy, health clamp, flash, kind switch and continuations),
//! 0x1a0b40 SetDmgState (state bookkeeping, ASEG pick +0xa0 / +0xc0 defaults, flags 1 / 0 / 0x21, case 8 ->
//! 9 soft landing test and yaw turn, timers), 0x1a16c8 knockback velocities (0x100 class 0x4f: 600 away +
//! 800 up; 0x200 / 0x8000 class 0x33: 1.25 * hit vector clamped to 1200 (0x11e9f8); others: hit vector),
//! 0x1a12f0 / 0x1a15d0 / 0x1a16a8 continuation dispatch, 0x1a1f68 / 0x1a1fa8 / 0x1a2028 / 0x1a2080,
//! 0x1a1358 flash alpha (jt+0xd0 = clamp(1 - .75 f^2) * clamp(.7 + .3 cos(10 t))) and state-7 turn / landing,
//! 0x1a2230 / 0x1a21a0 / 0x140ed8 immunity, 0x144418, 0x1a23c0, 0x1a23f8 ASEG fallback table 0x2bfb28,
//! 0x1a2100, 0x1a2160, 0x144188 / 0x1440d0 health, 0x155228, 0x18d568 (SetJtJts table), 0x161a58 case 0x10.
//! REPLACED: difficulty object 0x2de578 (+0x14 -> +0x20 / +0x24 lerp, factor 0x154c90 from the save data)
//! -> `DmgEnv::diff`; the state ASEGs DMG[0x28 + state] / +0xa0 / +0xc0 (loaded with the level, loader not
//! traced) -> `Dmg::aseg` oids, played through `JtAnim` with field 0xe90 + 4 * state; the source object's
//! is-a tests / +0xfbc / +0x148 / position -> `HitSrc`; 0x1a0b40 case 8 normal jt+0x1b0 = facing row (0x204660);
//! FUN_00204740 / FUN_00204660 yaw sync, 0x2046b0 face-camera (`Jt::cam_pos`), +0x19b8 / +0x19bc pose offsets live.
//! UNPORTED (recorded in `Jt::calls` / `DmgEv`): 0x143380, 0x1fbb90 / 0x1fb2f8 sounds, 0x156698 hit fx,
//! 0x1f4bf8, 0x1f80d0, job restart 0x162bb8 -> 0x165538 (jobs are not ported), 0x1a2460 / 0x1a2788 (jt+0x108c
//! hazard volume sampling that builds hits).
#![allow(non_snake_case)]
use super::{Jt, JtAnim};
use crate::engine::camera::math::FUN_0011d2d0;
use crate::engine::physics::geom::{dot, V3};

/// 0x1a0b18: DMG+0xc (jt+0xdfc) health maximum
pub const HP_MAX: i32 = 0x28;
/// 0x1a0b18: DMG+0x9c flash fade duration
pub const FLASH_DUR: f32 = 2.0;
/// jts of the damage component (0x18d568 table entry 3)
pub const JTS_DMG: i32 = 4;
/// hit kinds (+0xc)
pub const K_HEAL: i32 = 0x100000;
/// lethal hits (0x144418) TakeDamage delta
pub const LETHAL: i32 = -0x8000;
/// 0x2bfac0 SMP of the state-7 turn
pub const SMP_TURN: [f32; 4] = [3.5, 0.0, 0.5, 0.25];
/// 0x2bfb28 ASEG fallback chain of 0x1a23f8 (state -> next, -1 end)
pub const ASEG_NEXT: [i32; 13] = [-1, 0, 0, 0, -1, 0, 0, 0, 0, -1, 8, 8, 5];

/// 0x18d568: vt+0x2c4(jt, m) -> SetJtJts(jt, table[m], -1) (0x2a5d00).
pub fn FUN_0018d568(m: i32) -> Option<i32> {
    Some(match m {
        0 => 0,
        1 => 2,
        2 => 7,
        3 => 4,
        4 => 0x12,
        5 => 0x13,
        6 => 0x14,
        7 => 0x17,
        8 => 8,
        _ => return None,
    })
}

/// Source object of a hit (what 0x1a19a0 / 0x1a16c8 / 0x1a15d0 read from hit+0).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HitSrc {
    /// class ids the object is-a (FUN_00132b10 tests 0x33 / 0x36 / 0x38 / 0x4f / 0x51)
    pub isa: Vec<i32>,
    /// FUN_001a62e8 position
    pub pos: V3,
    /// FUN_001a6318 rotation rows of the source (None = identity; the 0x36 knockback frame of 0x1a16c8)
    pub rot: Option<[V3; 3]>,
    /// class 0x51: vt+0xfc direction
    pub dir_fc: Option<V3>,
    /// +0xfbc state (class 0x38: 8 / 0xd halve the damage)
    pub st_fbc: i32,
    /// +0x148 flags (bit 33: recover by warping to the safe point 0x1a16a8)
    pub flags_148: u64,
}

impl HitSrc {
    fn is(&self, c: i32) -> bool {
        self.isa.contains(&c)
    }
}

/// 0x60-byte hit record (TakeDamage param_2).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Hit {
    /// +0x00 source object (None = 0)
    pub src: Option<HitSrc>,
    /// +0x0c kind
    pub kind: i32,
    /// +0x10 amount (negative = damage, before the difficulty scale)
    pub amount: i32,
    /// +0x30 vector (knockback velocity)
    pub vec: V3,
}

/// Continuation PMF at DMG+0x1c (0x1a15b0); table 0x2a6768.. (8-byte entries).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Cont {
    /// 0x2a6768 null -> 0x1a15d0 default
    #[default]
    None,
    /// 0x2a6770
    C1a1f68,
    /// 0x2a6778
    C1a2028,
    /// 0x2a6780
    C1a1fa8,
    /// 0x2a6788
    C1a16a8,
    /// 0x2a6790
    C1a2080,
}

/// Engine effects of the damage component the JT does not own.
#[derive(Clone, Debug, PartialEq)]
pub enum DmgEv {
    /// FUN_001619a0(0x2deb30, k, obj): 0xf damaged, 0x10 died
    Event(i32),
    /// FUN_001fbb90(id) + FUN_001fb2f8: sound
    Sound(i32),
    /// FUN_001305b8(0, 1.0, aseg, jt, flags, jt+0x1030): state ASEG (oid None = no ASEG loaded)
    Aseg { state: i32, oid: Option<i32>, flags: u32 },
    /// the heal ASEG DMG[0x42]
    HealAseg(i32),
    /// +0x4c8 == 1: the hit goes to the attach partner +0x4d4 (slot 74, the TNT barrel C2d20d0 FUN_00183e08):
    /// the 0x60-byte message (kind +0xc, amount +0x10) for the engine to deliver; BOSS R21: the consumer is
    /// classes::q::barrel::deliver (the original returns the partner's slot-74 result; the port returns false)
    Delegated { partner: Option<usize>, kind: i32, amount: i32 },
    /// slot 92 0x1a2388 (death)
    Die,
    /// 0x161a58 case 0x10: level reload request
    Respawn(Respawn),
}

/// 0x161a58 case 0x10 -> FUN_0022b490(world, 0, DAT_003e1118, DAT_003e111c, flags, 0x30a).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Respawn {
    /// 0 = the current level (world+0x3c40)
    pub level: i32,
    /// world+0x3c88 spawn oid
    pub spawn: i32,
    /// world+0x3c8c spawn parent oid
    pub spawn_parent: i32,
    /// DAT_003e12c8 ? 0x20 : 0
    pub flags: u32,
    pub a: u32,
}

impl Respawn {
    /// FUN_0022b490 -> FUN_0022b220: the world+0x3bf4 request (level 0 = the current descriptor +0x3c40).
    /// The flags word gets bit 1 when the level is the current one (0x22b220); the 0x30a argument (fade /
    /// transition style of FUN_0022b090) is not part of the request record.
    pub fn level_request(&self, current: crate::engine::disc::descriptor::Descriptor) -> crate::engine::game::transition::LevelRequest {
        let mut r = crate::engine::game::transition::LevelRequest::new(current);
        r.flags = self.flags | 2;
        r.spawn = self.spawn;
        r.spawn_parent = self.spawn_parent;
        r
    }
}

/// World-side inputs of the damage code.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DmgEnv {
    /// 0x155228: (1 - f) * lo + f * hi (REPLACED difficulty object)
    pub diff: [f32; 3],
    /// DAT_003e0f9c != 0 (a job is active: the job restarts instead, unported)
    pub job: bool,
    /// world+0x3c88 / +0x3c8c
    pub spawn: (i32, i32),
    /// DAT_003e12c8
    pub flag_12c8: bool,
    /// DAT_002e1e00 & 1 (cheat invulnerability)
    pub cheat: bool,
    /// FUN_0020f0b0(world) != 0
    pub world_busy: bool,
    /// DAT_003d4a54 && FUN_001c40c0()
    pub immune_4a54: bool,
}

impl Default for DmgEnv {
    fn default() -> Self {
        Self { diff: [1.0, 1.0, 0.0], job: false, spawn: (0x189, -1), flag_12c8: false, cheat: false, world_busy: false, immune_4a54: false }
    }
}

/// jt+0xdf0 (0x110 bytes).
#[derive(Clone, Debug, PartialEq)]
pub struct Dmg {
    /// +0x04 state (-1 none)
    pub state: i32,
    /// +0x08 time the state was entered
    pub t_state: f32,
    /// +0x0c (jt+0xdfc) maximum health
    pub hp_max: i32,
    /// *(+0x10) = "hpCur"
    pub hp: i32,
    /// +0x1c continuation
    pub cont: Cont,
    /// +0x30 copy of the last hit (+0x40 overwritten with the applied delta)
    pub hit: Option<Hit>,
    pub delta: i32,
    /// +0x90 flash (-1 none, 0 heal, 1 hurt)
    pub flash: i32,
    /// +0x94 flash start, +0x98 fade start, +0x9c fade duration
    pub t_flash: f32,
    pub t_fade: f32,
    pub fade_dur: f32,
    /// +0xa0 / +0xc0 default ASEGs (< 8 / >= 8), +0xa0 + 4*state (DMG[0x28+state]) per state, DMG[0x42] heal
    pub aseg_def: [Option<i32>; 2],
    pub aseg: [Option<i32>; 13],
    pub aseg_heal: Option<i32>,
    /// jt+0xd0 (flash alpha)
    pub alpha: f32,
    /// end time of the state ASEG at jt+0x1030 (FUN_00131968 done test). REPLACED: set on state entry to
    /// f32::MAX when the state has an ASEG (the anim driver writes the real end) or state 7 (ended by the
    /// landing in 0x1a1358); a state without ASEG is done at once.
    pub aseg_end: f32,
    /// jt+0x298 invulnerable, jt+0x29c no damage, jt+0xf20 == 1 immune (BOSS R21: +0x4c8 is `jt.attach.kind`)
    pub f_298: bool,
    pub f_29c: bool,
    pub f_f20: bool,
    /// jt+0x15c
    pub f_15c: bool,
    /// engine effects of this frame
    pub ev: Vec<DmgEv>,
}

impl Default for Dmg {
    fn default() -> Self {
        // 0x1a0b18 + 0x1440d0 (hpCur 0 -> max)
        Self {
            state: -1,
            t_state: 0.0,
            hp_max: HP_MAX,
            hp: HP_MAX,
            cont: Cont::None,
            hit: None,
            delta: 0,
            flash: -1,
            t_flash: 0.0,
            t_fade: 0.0,
            fade_dur: FLASH_DUR,
            aseg_def: [None; 2],
            aseg: [None; 13],
            aseg_heal: None,
            alpha: 0.0,
            aseg_end: f32::MAX,
            f_298: false,
            f_29c: false,
            f_f20: false,
            f_15c: false,
            ev: Vec::new(),
        }
    }
}

impl Dmg {
    /// 0x00144188: hp = min(v, max)
    pub fn FUN_00144188(&mut self, v: i32) {
        self.hp = v.min(self.hp_max);
    }
    /// 0x001440d0 (hp part): hpCur == 0 or !keep -> hp = max
    pub fn FUN_001440d0(&mut self, keep: bool) {
        if self.hp == 0 || !keep {
            self.FUN_00144188(self.hp_max);
        }
    }
    /// 0x001a23c0 (slot 76): 0 none / 1 alive / 2 dying; idle: flash == 1
    pub fn FUN_001a23c0(&self) -> i32 {
        if self.state == -1 {
            return (self.flash == 1) as i32;
        }
        if self.hp > 0 {
            1
        } else {
            2
        }
    }
    /// 0x00144418 (slot 85): lethal hit
    pub fn FUN_00144418(&self, h: &Hit) -> bool {
        self.hp_max <= -h.amount && !matches!(h.kind, 0 | 1 | 0x80000 | K_HEAL)
    }
    /// 0x001a23f8: state ASEG with the 0x2bfb28 fallback chain
    pub fn FUN_001a23f8(&self) -> Option<i32> {
        if self.state == -1 {
            return None;
        }
        let mut i = self.state;
        loop {
            if let Some(a) = self.aseg[i as usize] {
                return Some(a);
            }
            let n = ASEG_NEXT[i as usize];
            if n == -1 {
                return None;
            }
            i = n;
        }
    }
    /// 0x001a2100: (+0x94, +0x98) of the camera param block by state (None = unchanged)
    pub fn FUN_001a2100(&self) -> Option<[f32; 2]> {
        match self.state {
            0 | 6 | 7 => None,
            4 | 0xc => Some([1.0, 0.0]),
            _ => Some([1.0, 0.3]),
        }
    }
    /// 0x001a2160: flags word bit 0 set except states 5 / 8..0xc which clear +0x34 instead
    pub fn FUN_001a2160(&self, flags: &mut u32, w34: &mut u32) {
        match self.state {
            5 | 8..=0xc => *w34 = 0,
            _ => *flags |= 1,
        }
    }
}

/// 0x00155228(0x2de578, amount)
pub fn FUN_00155228(env: &DmgEnv, amount: i32) -> i32 {
    let [lo, hi, f] = env.diff;
    (((1.0 - f) * lo + f * hi) * amount as f32) as i32
}

/// 0x001a2230: immunity of the damage component
pub fn FUN_001a2230(jt: &Jt, env: &DmgEnv, kind: i32) -> bool {
    let d = &jt.dmg;
    if d.f_298 {
        return true;
    }
    if kind == 0x2000 || kind == 0x4000 {
        return match d.state {
            3 | 5 | 10 | 0xb => true,
            -1 => match jt.attach.kind {
                1 | 2 | 5 | 6 | 9 => false,
                -1 => {
                    // +0x2dc = JMP +0x1c (jtbs)
                    match jt.jtbs {
                        -1 => false,
                        b if b < -1 => false,
                        b if b < 8 => false,
                        b if b < 0x12 => kind == 0x4000,
                        _ => false,
                    }
                }
                _ => true,
            },
            _ => false,
        };
    }
    d.state != -1 || d.flash == 1 || env.cheat || env.world_busy
}

/// 0x001a21a0 (slot 75) -> 0x00140ed8: hit kind vetoed
pub fn FUN_001a21a0(jt: &Jt, env: &DmgEnv, kind: i32) -> bool {
    match jt.jts {
        3 | 8 | 0x15 | 0x16 if kind == 0x4000 => return true,
        // 0x194640(jt+0x4c0): attach kind 8 sub 9/10 or kind 0xc -> immune.
        7 if crate::engine::classes::k16_55::ee_sub_194640(jt.attach.kind, jt.attach.sub) => {
            return true
        }
        _ => {}
    }
    // 0x140ed8 (+0x2dc = JMP jtbs)
    if jt.dmg.f_f20 || jt.jtbs == 0x15 || env.immune_4a54 {
        return true;
    }
    FUN_001a2230(jt, env, kind)
}

fn len(v: V3) -> f32 {
    dot(v, v).sqrt()
}

/// 0x0011e9f8: clamp |v| to m
pub fn FUN_0011e9f8(m: f32, v: V3) -> V3 {
    let l = len(v);
    if m < l {
        let k = m / l;
        [v[0] * k, v[1] * k, v[2] * k]
    } else {
        v
    }
}

/// FUN_00204660(yaw, jt) on the JT (jt+0x1a8 / jt+0x1b0 rows).
fn face_yaw(jt: &mut Jt, yaw: f32) {
    let mut x = std::mem::take(&mut jt.att);
    super::p5_attach::FUN_00204660(jt, &mut x, yaw);
    jt.att = x;
}

/// FUN_00204660(FUN_00204740(jt), jt); (jt+0x150)+0x18 = jt+0x1a8.
fn sync_yaw(jt: &mut Jt) {
    face_yaw(jt, super::p22::body_yaw(&jt.phys));
    jt.joy.angle = jt.yaw;
}

/// FUN_002046b0(jt, DAT_002ddfd0 - FUN_001a62e8(jt)): face the camera (prot3w + FUN_002458f0 -> 0x204660).
fn face_camera(jt: &mut Jt) {
    match jt.cam_pos {
        Some(c) => {
            let p = jt.phys.pos;
            face_yaw(jt, (c[1] - p[1]).atan2(c[0] - p[0]));
        }
        None => jt.calls.push((0x002046b0, 0)),
    }
}

/// 0x001a0b40 SetDmgState.
pub fn FUN_001a0b40(jt: &mut Jt, mut s: i32, anim: &mut dyn JtAnim) {
    let old = jt.dmg.state;
    if s == old {
        return;
    }
    let now = jt.t;
    if old == 3 || old == 10 {
        // class 0x35 (the JT): pose offset +0x19b8 -> (0, 0, 0)
        jt.pose_19b8[..3].fill(0.0);
    } else if old == -1 {
        // pose offsets +0x19b8 / +0x19bc = DAT_002bfa00 (0, 0, -750, 0), vt+0x2dc = 0x18d548 (recorded)
        jt.pose_19b8 = [0.0, 0.0, -750.0, 0.0];
        jt.pose_19bc = [0.0, 0.0, -750.0, 0.0];
        jt.calls.push((0x0018e540, -1));
        jt.v_target = [0.0; 3]; // +0x190..+0x198
        jt.dmg.f_15c = false;
        jt.dmg.t_fade = f32::MAX; // DAT_002a6720
    }
    let mut oid = None;
    let mut flags = 1u32;
    let mut has = true;
    match s {
        0 | 6 | 7 => {}
        1 | 2 => {
            sync_yaw(jt);
            flags = 0;
        }
        4 | 0xc => {
            jt.calls.push((0x001f4488, 0)); // vt+0x124
            sync_yaw(jt);
            if s == 0xc {
                flags = 0x21;
            }
        }
        5 | 0xb => {
            face_camera(jt);
            if s == 0xb {
                flags = 0x21;
            }
        }
        8 | 9 => {
            if s == 8 {
                let v = jt.velocity_pub();
                // jt+0x1b0 . v (the facing row written by FUN_00204660)
                let n = jt.att.rows[0];
                if -250.0 < v[0] * n[0] + v[1] * n[1] + v[2] * n[2] {
                    s = 9;
                } else {
                    let sp = (v[0] * v[0] + v[1] * v[1]).sqrt();
                    let k = (-1.0 + sp * (0.004 + sp * 0.0)).clamp(0.0, 1.0);
                    let d = super::p5_attach::FUN_0011cf60(v[1].atan2(v[0]) + std::f32::consts::PI - jt.yaw);
                    face_yaw(jt, jt.yaw + d * k);
                    // FUN_00204ca8: REPLACED, the JT body turns instantly to jt+0x1a8 (see Jt::update)
                    jt.calls.push((0x00204ca8, 0));
                }
            }
            flags = 0x21;
        }
        10 | 3 => {
            if s == 10 {
                flags = 0x21;
                face_camera(jt);
            }
            jt.pose_19b8 = [0.0, 0.0, 500.0, 0.0]; // DAT_002bfa10
        }
        -1 => {
            jt.pose_19b8[..3].fill(0.0);
            jt.pose_19bc[..3].fill(0.0);
            jt.dmg.f_15c = true;
            jt.dmg.t_fade = now;
            has = false;
        }
        _ => {}
    }
    if s >= 0 && (s as usize) < 13 {
        oid = jt.dmg.aseg[s as usize];
    }
    jt.calls.push((0x001f4bf8, 0));
    jt.dmg.state = s;
    jt.dmg.t_state = now;
    jt.p5.kind_df4 = s;
    if has {
        let oid = oid.or(jt.dmg.aseg_def[(s >= 8) as usize]);
        jt.dmg.aseg_end = if oid.is_some() || s == 7 { f32::MAX } else { now };
        jt.dmg.ev.push(DmgEv::Aseg { state: s, oid, flags });
        anim.play_aseg(0xe90 + 4 * s as u32, jt.jts, jt.jtbs);
        jt.calls.push((0x0018c060, 0)); // vt+0x2d4
    }
}

/// 0x001a16c8 knockback: SetVelocity + state 7.
pub fn FUN_001a16c8(jt: &mut Jt, anim: &mut dyn JtAnim) {
    let h = jt.dmg.hit.clone().unwrap_or_default();
    let mut v = h.vec;
    let src = h.src.clone().unwrap_or_default();
    let has = h.src.is_some();
    match h.kind {
        0x100 if has && src.is(0x4f) => {
            let mut d = match src.dir_fc {
                Some(d) if src.is(0x51) => d,
                _ => {
                    let p = jt.phys.pos;
                    [p[0] - src.pos[0], p[1] - src.pos[1], p[2] - src.pos[2]]
                }
            };
            d[2] = 0.0;
            let l = len(d);
            let n = if 0.0001 <= l { [d[0] / l, d[1] / l, 0.0] } else { [0.0; 3] };
            v = [n[0] * 600.0, n[1] * 600.0, 800.0];
        }
        0x40 if has && src.is(0x36) => {
            // v = jt pos - (src pos + R.row0 * -150); FUN_001a6fc0(0, src, v) (x, y of R v, z kept), prot3w,
            // FUN_001a6fc0(src, 0, ..) (full R^T... rows combined as p * R)
            let r = src.rot.unwrap_or([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]);
            let p = jt.phys.pos;
            let k = r[0];
            let d = [
                p[0] - (src.pos[0] - 150.0 * k[0]),
                p[1] - (src.pos[1] - 150.0 * k[1]),
                p[2] - (src.pos[2] - 150.0 * k[2]),
            ];
            let l = [dot(r[0], d), dot(r[1], d), d[2]];
            let q = [l[1], l[2], l[0]]; // PROT3W
            v = [0, 1, 2].map(|j| q[0] * r[0][j] + q[1] * r[1][j] + q[2] * r[2][j]);
        }
        0x200 | 0x8000 if has && src.is(0x33) => {
            v = FUN_0011e9f8(1200.0, [v[0] * 1.25, v[1] * 1.25, v[2] * 1.25]);
        }
        _ => {}
    }
    jt.set_velocity_pub(v); // vt+0x11c
    FUN_001a0b40(jt, 7, anim);
}

/// 0x001a19a0 TakeDamage (slot 74 0x140f50). Returns 1 when the hit was taken.
pub fn FUN_001a19a0(jt: &mut Jt, h: &Hit, env: &DmgEnv, anim: &mut dyn JtAnim) -> bool {
    let mut h = h.clone();
    let delta;
    if jt.dmg.FUN_00144418(&h) {
        if (8..=12).contains(&jt.dmg.state) {
            return false;
        }
        delta = LETHAL;
    } else {
        if FUN_001a21a0(jt, env, h.kind) {
            return false;
        }
        if jt.attach.kind == 1 {
            jt.dmg.ev.push(DmgEv::Delegated { partner: jt.attach.target, kind: h.kind, amount: h.amount });
            return false;
        }
        let mut d = 0;
        if !jt.dmg.f_29c && jt.dmg.flash != 1 {
            d = FUN_00155228(env, h.amount);
        }
        if h.kind == K_HEAL {
            if jt.dmg.hp + d >= 1 {
                if jt.dmg.flash != 1 {
                    jt.dmg.flash = 0;
                    jt.dmg.t_flash = jt.t;
                    jt.dmg.t_fade = jt.t;
                }
                jt.dmg.hp += d;
                if let Some(a) = jt.dmg.aseg_heal {
                    jt.dmg.ev.push(DmgEv::HealAseg(a));
                }
                jt.calls.push((0x00156698, 0));
                return true;
            }
            // a heal that kills: becomes kind 0x200 with no vector
            h.kind = 0x200;
            h.vec = [0.0; 3];
        }
        if let Some(s) = &h.src {
            if s.is(0x38) && (s.st_fbc == 8 || s.st_fbc == 0xd) {
                d /= 2;
            }
        }
        delta = d;
    }
    take(jt, h, delta, anim);
    true
}

fn take(jt: &mut Jt, h: Hit, delta: i32, anim: &mut dyn JtAnim) {
    // vt+0x2c4(3) -> SetJtJts(jt, 4, -1)
    jt.set_jts(JTS_DMG, super::JTBS_NIL, anim);
    // jt+0x11a8 (carried object) && (... || hp + delta < 1) -> FUN_00143380: no carried objects in the port
    jt.dmg.ev.push(DmgEv::Event(0xf));
    let kind = h.kind;
    jt.dmg.hit = Some(h);
    jt.dmg.delta = delta;
    jt.dmg.cont = Cont::None;
    jt.dmg.hp = (jt.dmg.hp + delta).max(0);
    if jt.dmg.hp > 0 {
        jt.dmg.flash = 1;
        jt.dmg.t_flash = jt.t;
    }
    let hp = jt.dmg.hp;
    match kind {
        0x800 | 0x1000 => {
            jt.set_velocity_pub([0.0; 3]); // vt+0x11c (argument lost in the decompile: stop)
            jt.calls.push((0x001f4488, 0));
            FUN_001a0b40(jt, if kind == 0x800 { 1 } else { 2 }, anim);
            jt.dmg.cont = Cont::C1a1fa8;
        }
        0x100 | 0x200 => {
            FUN_001a16c8(jt, anim);
            let v = jt.velocity_pub();
            if 400.0 < v[2] {
                // sound by the JT class: 0x35 -> 0x69, 0x36 -> 0x84, 0x37 -> 0x83 (Sly = 0x35)
                jt.dmg.ev.push(DmgEv::Sound(0x69));
            }
            jt.dmg.cont = Cont::C1a1f68;
        }
        0x40 => {
            FUN_001a16c8(jt, anim);
            jt.dmg.cont = Cont::C1a1f68;
        }
        0x400 => {
            jt.set_velocity_pub([0.0; 3]);
            FUN_001a0b40(jt, if hp < 1 { 0xc } else { 4 }, anim);
        }
        0x2000 => {
            if hp < 1 {
                FUN_001a0b40(jt, 10, anim);
            } else {
                FUN_001a0b40(jt, 3, anim);
                jt.dmg.cont = Cont::C1a16a8;
            }
        }
        0x4000 => {
            if hp > 0 {
                FUN_001a0b40(jt, 5, anim);
                jt.dmg.cont = Cont::C1a16a8;
            } else {
                FUN_001a0b40(jt, 0xb, anim);
            }
        }
        0x8000 => {
            FUN_001a0b40(jt, 7, anim);
            jt.dmg.cont = Cont::C1a2028;
        }
        _ => {}
    }
    jt.calls.push((0x00156698, 0));
}

/// 0x001a15d0: run (and clear) the continuation.
pub fn FUN_001a15d0(jt: &mut Jt, env: &DmgEnv, anim: &mut dyn JtAnim) {
    let c = std::mem::take(&mut jt.dmg.cont);
    let hp = jt.dmg.hp;
    let recover = |jt: &mut Jt, anim: &mut dyn JtAnim| {
        let warp = jt.dmg.hit.as_ref().and_then(|h| h.src.as_ref()).is_some_and(|s| s.flags_148 & 0x2_0000_0000 != 0);
        if warp {
            FUN_001a16a8(jt, anim);
        } else {
            jt.FUN_00195a00(0xb, 0, anim);
        }
    };
    match c {
        Cont::None => {
            if (8..=12).contains(&jt.dmg.state) {
                FUN_001a2388(jt, env);
            } else {
                recover(jt, anim);
            }
        }
        Cont::C1a1f68 => FUN_001a0b40(jt, if hp < 1 { 8 } else { 0 }, anim),
        Cont::C1a1fa8 | Cont::C1a2080 => {
            if hp < 1 {
                FUN_001a0b40(jt, 8, anim);
            } else {
                recover(jt, anim);
            }
        }
        Cont::C1a2028 => {
            jt.dmg.ev.push(DmgEv::Sound(0x6a));
            FUN_001a0b40(jt, 6, anim);
            jt.dmg.cont = Cont::C1a2080;
        }
        Cont::C1a16a8 => FUN_001a16a8(jt, anim),
    }
}

/// 0x001a16a8: FUN_0019f590(jt, 0) warp to the safe point
pub fn FUN_001a16a8(jt: &mut Jt, anim: &mut dyn JtAnim) {
    jt.with_p5(|jt, st, w| {
        super::p5_states::FUN_0019f590(jt, st, w, None, anim);
    });
}

/// 0x001a2388 (slot 92): die -> event 0x10 -> 0x161a58.
pub fn FUN_001a2388(jt: &mut Jt, env: &DmgEnv) {
    jt.calls.push((0x001f80d0, 1));
    jt.dmg.ev.push(DmgEv::Die);
    jt.dmg.ev.push(DmgEv::Event(0x10));
    // job current (DAT_003e0f9c != 0): 0x161a58 does nothing, the job half (0x162bb8 -> 0x165538) runs from the
    // Event(0x10) in `p8_hazard::job_events`; else 0x161a58 case 0x10 reloads the level
    if !env.job {
        let r = Respawn { level: 0, spawn: env.spawn.0, spawn_parent: env.spawn.1, flags: if env.flag_12c8 { 0x20 } else { 0 }, a: 0x30a };
        jt.dmg.ev.push(DmgEv::Respawn(r));
        jt.respawn = Some(r);
    }
}

/// 0x001a12f0: while the state ASEG (jt+0x1030) is done, run the continuation (done = jt.t >=
/// `Dmg::aseg_end`).
pub fn FUN_001a12f0(jt: &mut Jt, env: &DmgEnv, anim: &mut dyn JtAnim) {
    loop {
        let s0 = jt.dmg.state;
        if jt.t >= jt.dmg.aseg_end {
            FUN_001a15d0(jt, env, anim);
        }
        let s = jt.dmg.state;
        if s == -1 || s == s0 {
            break;
        }
    }
}

/// 0x001a1358: flash alpha and the state-7 turn / landing (from the 0x18b7e8 input step).
pub fn FUN_001a1358(jt: &mut Jt, env: &DmgEnv, anim: &mut dyn JtAnim) {
    let now = jt.t;
    let d = &mut jt.dmg;
    if d.flash != -1 {
        let mut f = 0.0;
        let mut off = false;
        if d.state == -1 {
            if d.fade_dur < now - d.t_fade {
                d.flash = -1;
                d.alpha = 0.0;
                off = true;
            } else {
                f = (now - d.t_fade) / d.fade_dur;
            }
        }
        if off {
            jt.calls.push((0x001f4bf8, 0));
        } else {
            let g = ((now - d.t_flash) * 10.0).cos();
            let a = (1.0 + f * (0.0 + f * -0.75)).clamp(0.0, 1.0);
            let b = (0.7 + g * (0.3 + g * 0.0)).clamp(0.0, 1.0);
            d.alpha = a * b;
        }
    }
    if jt.dmg.state == 7 {
        let v = jt.velocity_pub();
        let target = v[1].atan2(v[0]) + std::f32::consts::PI;
        jt.yaw = FUN_0011d2d0(jt.yaw, target, jt.dt, &SMP_TURN);
        if 0.25 < now - jt.dmg.t_state && jt.ground_nz.is_some() {
            FUN_001a15d0(jt, env, anim);
            return;
        }
        let normals = jt.p5w.contact_normals.clone();
        if super::p5_states::FUN_0019a098(jt, jt.dmg.t_state, None, &normals) {
            FUN_001a15d0(jt, env, anim);
        }
    }
}

/// 0x18c250 case 4: leaving jts 4 -> 0x1a0b40(-1). Every damage state is entered after vt+0x2c4(3) put
/// the JT into jts 4, so a state != -1 outside jts 4 means the JT left it.
pub fn exit_sync(jt: &mut Jt, anim: &mut dyn JtAnim) {
    if jt.jts != JTS_DMG && jt.dmg.state != -1 {
        FUN_001a0b40(jt, -1, anim);
    }
}
