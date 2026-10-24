//! WP-P11: Sly's cane (jt+0x18d8 "jtcs" sub-state), the cane hit geom, jt+0x1010 decay constant.
//!
//! 1:1: FUN_0018f0a8 (cane input, called by the JT input 0x18b7e8 right after 0x18a8c8; Square 0x80 = ground
//! swing, Triangle 0x10 = jtcs 0x16 / 0x17), FUN_0018ee38 (swing chooser), FUN_0018f050, FUN_0018e540 (set
//! jtcs: chained follow-up table DAT_002bf448, cane geom force-disable FUN_001f2ad0, +0x18ec / +0x19b0 /
//! +0x18e4 / +0x18dc, start time of a chained swing = min(old ASEG time, new length)), the cane part of the
//! JT message handler FUN_00188f20 (msg 9 ASEG events 0x164 / 0x166, msg 3 ASEG done, msg 0xb hit feedback
//! recoil), the ASEG event executor FUN_00131290 kinds 0xf / 0x10 (FUN_002104a0 -> FUN_001f6ac8 / FUN_001f6b48
//! enable / disable the SO geoms with that id) and 0xb (msg 9 to the owner), FUN_001f7598's per-target dedupe
//! (list 0x3e15c0: one hit per src / geom / kind / target while the geom stays on); jt+0x1010 decay via
//! p5_attach::FUN_0011f088 with DAT_002bf820.
//! The swing's hit data is the level's: Sly's ALO collision part id 0x14c (jt+0x1a88, postload table
//! 0x2bf1b0 flags 0x220 = disabled at load) carries hit kind +0x18 = 2 and amount +0x14 = -10; the swing ASEGs
//! (0x1ce0..0x1d04, 0x1d14 / 0x1d18) enable it with event 0xf and disable it with event 0x10.
//! REPLACED: the contact between the cane geom and a guard (SO contact list -> JT slot 73 FUN_001f7410) is an
//! overlap test of the guard's position against a sphere around the geom's bone origin (rest pose, turned by
//! the facing yaw) with the part's bounding radius (ALO part +f); FUN_001f7530 accept = guards of the class
//! set; the hit's response bits (FUN_001f78d8 over target +0x110 per-kind bytes, not loaded) = 0x20 (damage).
//! FUN_0018ed98 (cane model jt+0x19a4 local y > 0: which side the cane ends on) = `Cane::side`, set by each
//! swing 0xc..0x17 to "the even swing ran last" (0x18f0a8 resets jtcs to -1 before choosing, so the side
//! test is what alternates a combo). FUN_0018ee38's sneak takedown branch
//! (guard jt+0x10dc within 400 / FUN_0011d910 250 / guard SM 0xb1) needs the guard SM: jt+0x10dc is not
//! tracked -> never taken.
//! UNPORTED (recorded in `Jt::calls`): jtcs 0..7 / 10 / 11 bodies (aim blends 0x18f560 / 0x18f740, 0x1ccac0,
//! 0x1f4bf8, cane model swap 0x19a0 / 0x19a8), the sounds 0x1f0400 / 0x1f09d8 (0x17b) (WP-P16: 0x1fbb90 0xb / 0xf / target +0x130 ported, p16),
//! event 0x165 (charged follow-up SetJtJts(5, 6 / 7): jts 5 has no body in the port). WP-P16: jts 0xd / 0xe bodies in p16.
#![allow(non_snake_case)]
use super::{Jt, JtAnim, JTS_STAND, JTBS_NIL};
use crate::engine::anim::aseg::{Aseg, EventData};
use crate::engine::classes::guard::slots::GuardHit;
use crate::engine::classes::guard::PLAYER_HANDLE;
use crate::engine::physics::geom::V3;
use crate::engine::world::World;
use std::collections::HashMap;

/// DAT_002bf448: chained follow-up jtcs per jtcs (-1 = none).
pub const DAT_002BF448: [i32; 24] = [-1, -1, -1, -1, 6, 7, 4, 5, -1, -1, -1, -1, 14, 15, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1];
/// DAT_002bf440: +0x19b0 delay of the swings 0xc..0x17; DAT_002bf43c (blend time of the ASEG start)
pub const DAT_002BF440: f32 = 0.13333334;
pub const DAT_002BF43C: f32 = 0.06666667;
/// 0x3eb33333: press window of FUN_0018f0a8
pub const CANE_WINDOW: f32 = 0.35;
/// buttons of FUN_0018f0a8 (joy +0x30 / press bits)
pub const BTN_CANE: u32 = 0x80;
pub const BTN_CANE2: u32 = 0x10;
/// jt+0x1a88: Sly's cane hit geom id
pub const GEOM_CANE: u32 = 0x14c;
/// POOL1 R65: the air attack hit part (ALO part id 0x14d), turned on / off by the jts 0xd ASEG jt+0x1a80
/// events 0xf / 0x10 (id 333)
pub const GEOM_AIR: u32 = 0x14d;
/// FUN_00188f20 msg 9 event ids
pub const EV_COMBO: i32 = 0x164;
pub const EV_HOLD: i32 = 0x165;
pub const EV_18E8: i32 = 0x166;
/// REPLACED response bits of a cane hit (module doc)
pub const CANE_RESPONSE: i32 = 0x20;

/// JT member of the jtcs ASEG (FUN_0018e540); None = no direct member.
pub fn jtcs_field(n: i32) -> Option<u32> {
    match n {
        8 => Some(0x1a74),
        9 => Some(0x1a78),
        0xc..=0x15 => Some(0x1ce0 + 4 * (n - 0xc) as u32),
        0x16 => Some(0x1d14),
        0x17 => Some(0x1d18),
        _ => None,
    }
}

/// The timed events of a cane ASEG the port executes: (time, kind, arg) for kinds 0xb (msg 9 id), 0xf / 0x10
/// (geom id).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CaneAseg {
    pub len: f32,
    pub ev: Vec<(f32, u8, i32)>,
}

impl CaneAseg {
    pub fn from_aseg(a: &Aseg) -> Self {
        let ev = a
            .events1
            .iter()
            .filter_map(|e| {
                let t = e.time?;
                match (e.kind, &e.data) {
                    (0xb, EventData::S32(x)) => Some((t, 0xb, *x)),
                    (0xf | 0x10, EventData::S32x2(_, id)) => Some((t, e.kind, *id)),
                    _ => None,
                }
            })
            .collect();
        Self { len: a.length, ev }
    }
}

/// Sly's cane hit geom (ALO part id 0x14c): hit kind +0x18, amount +0x14, sphere in the JT frame (REPLACED).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CaneGeom {
    pub kind: i32,
    pub amount: i16,
    pub center: V3,
    pub radius: f32,
}

/// Level data of the cane: ASEGs by JT member, hit geom.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CaneData {
    pub asegs: HashMap<u32, CaneAseg>,
    pub geom: Option<CaneGeom>,
    /// POOL1 R65: part 0x14d (air attack)
    pub geom_air: Option<CaneGeom>,
}

impl CaneData {
    pub fn load(w: &World, jt: usize) -> Self {
        use crate::engine::playerdraw::{aseg_of, jt_anim_members};
        let members = jt_anim_members(w, jt);
        let asegs = (8..=0x17)
            .filter_map(jtcs_field)
            .chain([0x1d08, 0x1d0c, 0x1d10, 0x1fe8]) // WP-P13: jts 5 jtbs 6..9 bodies
            .chain([0x1a80, 0x1a84, 0x1d34]) // WP-P16: jts 0xd / 0xe / 0x11 state ASEGs
            .chain([0x438]) // WP-P19: glide open (0x195fb8 case 0x13)
            .chain([0x2000, 0x2004]) // WP-P25: jts 6 side ASEGs (oids 0x119 / 0x11a)
            .filter_map(|f| Some((f, CaneAseg::from_aseg(aseg_of(w, *members.get(&f)?)?))))
            .collect();
        Self { asegs, geom: cane_geom(w, jt), geom_air: cane_geom_id(w, jt, GEOM_AIR) }
    }
}

/// The ALO part 0x14c of the JT: bone origin (rest pose, JT frame) + bounding radius.
pub fn cane_geom(w: &World, jt: usize) -> Option<CaneGeom> {
    cane_geom_id(w, jt, GEOM_CANE)
}

/// ALO part `id` of the JT as a hit geom (see `cane_geom`).
pub fn cane_geom_id(w: &World, jt: usize, id: u32) -> Option<CaneGeom> {
    use crate::engine::gfx::attach::{alo_of, BoneWorlds};
    let p = alo_of(w, jt)?.parts.iter().find(|p| p.w0 == id as i32)?;
    let m = BoneWorlds::default().get(w, jt, p.r#ref as usize);
    let root = BoneWorlds::default().get(w, jt, 0);
    let c = [0, 1, 2].map(|j| m[3][j] - root[3][j]);
    Some(CaneGeom { kind: p.w1 as i32, amount: p.s0, center: c, radius: p.f })
}

/// jt+0x18c0..+0x19b0 cane fields + the running cane ASEG (jt+0x18e0).
#[derive(Clone, Debug, PartialEq)]
pub struct Cane {
    /// +0x18d8
    pub jtcs: i32,
    /// +0x18e4 combo window open / +0x18e8 / +0x18ec button still held
    pub f_18e4: bool,
    pub f_18e8: bool,
    pub f_18ec: bool,
    /// +0x19b0 / +0x18dc (start time)
    pub t_19b0: f32,
    pub t_18dc: f32,
    /// running cane ASEG (JT member) and its local time
    pub field: Option<u32>,
    pub t_aseg: f32,
    /// jt+0x1a88 geom enabled (bit 50 clear)
    pub geom_on: bool,
    /// POOL1 R65: part 0x14d enabled
    pub geom_air_on: bool,
    /// 0x3e15c0 entries of this JT's cane: targets already hit while the geom is on
    pub hit_done: Vec<u32>,
    /// REPLACED FUN_0018ed98 (cane model on the other side after an even swing)
    pub side: bool,
    pub data: CaneData,
    /// CRAWL R26: jt+0x1bd4 pickpocket hand in reach (FUN_0018b460, jts 0xb; unported -> set by callers),
    /// jt+0x1bcc caught item; `pick_167` = FUN_001f1e20(jt+0x10dc) calls of event 0x167 for the class set
    pub f_1bd4: bool,
    pub f_1bcc: u32,
    pub pick_167: Vec<u32>,
}

impl Default for Cane {
    fn default() -> Self {
        Self {
            jtcs: -1,
            f_18e4: false,
            f_18e8: false,
            f_18ec: false,
            t_19b0: 0.0,
            t_18dc: 0.0,
            field: None,
            t_aseg: 0.0,
            geom_on: false,
            geom_air_on: false,
            f_1bd4: false,
            f_1bcc: 0,
            pick_167: Vec::new(),
            hit_done: vec![],
            side: false,
            data: CaneData::default(),
        }
    }
}

impl Cane {
    /// FUN_001f2ad0(jt, jt+0x1a88) / FUN_001f6b48: disable (removes the geom's 0x3e15c0 entries with it)
    pub fn geom_off(&mut self) {
        self.geom_on = false;
        self.hit_done.clear();
    }

    /// POOL1 R65: FUN_001f6ac8 / FUN_001f6b48 on SO geom `id` (0x14c cane, 0x14d air attack); disabling one
    /// drops the 0x3e15c0 entries
    pub fn geom_set(&mut self, id: u32, on: bool) {
        match (id, on) {
            (GEOM_CANE, true) => self.geom_on = true,
            (GEOM_CANE, false) => self.geom_off(),
            (GEOM_AIR, true) => self.geom_air_on = true,
            (GEOM_AIR, false) => {
                self.geom_air_on = false;
                self.hit_done.clear();
            }
            _ => {}
        }
    }
}

/// 0x0018f050: Triangle swing 0x16 / 0x17 alternate.
pub fn FUN_0018f050(jt: &Jt) -> i32 {
    match jt.cane.jtcs {
        0x16 => 0x17,
        0x17 => 0x16,
        _ if jt.cane.side => 0x17,
        _ => 0x16,
    }
}

/// 0x0011cf60: angle wrapped to (-pi, pi].
fn wrap(a: f32) -> f32 {
    let tau = std::f32::consts::TAU;
    let mut a = a % tau;
    if a > std::f32::consts::PI {
        a -= tau;
    } else if a <= -std::f32::consts::PI {
        a += tau;
    }
    a
}

/// 0x0018ee38: ground swing chooser (takedown branch never taken, module doc).
pub fn FUN_0018ee38(jt: &Jt) -> i32 {
    let mut n = if jt.jtbs == 5 {
        // vt+0xfc vector -> FUN_002458f0 yaw (REPLACED: the facing yaw), stick angle joy +0x18
        let a = wrap(jt.joy.angle - jt.yaw);
        if a.abs() < 0.7853982 {
            0xe
        } else if a.abs() <= 2.3561945 {
            0x10
        } else if 0.0 < a {
            0x12
        } else {
            0x14
        }
    } else if jt.jts == JTS_STAND {
        0xc
    } else {
        0xe
    };
    match jt.cane.jtcs {
        0xc | 0xe | 0x10 | 0x12 | 0x14 => n += 1,
        0xd | 0xf | 0x11 | 0x13 | 0x15 => {}
        _ if jt.cane.side => n += 1,
        _ => {}
    }
    n
}

impl Jt {
    /// 0x0018f0a8: cane input (after 0x18a8c8 in 0x18b7e8).
    pub fn FUN_0018f0a8(&mut self, anim: &mut dyn JtAnim) {
        let c = self.cane.jtcs;
        if (0xc..0x16).contains(&c) && self.joy.held & BTN_CANE == 0 {
            self.cane.f_18ec = false;
        }
        let ok = match self.jts {
            0 | 1 | 10 | 0xc => true,
            3 | 0x15 | 0x16 => {
                if self.joy.FUN_001c46c0(self.t, 0.0, BTN_CANE) != 0 {
                    self.FUN_00195a00(0, 0, anim); // FUN_00195a00(jt+0x2c0, 0, 0, 0)
                }
                false
            }
            5 => match self.jtbs {
                j if j < 6 => true,
                6 | 7 => {
                    if self.joy.held & BTN_CANE == 0 {
                        let b = if self.cane.f_18e8 { 8 } else { 4 };
                        self.set_jts(5, b, anim); // WP-P13: full SetJtJts (jtbs 8 plays 0x1d10)
                    }
                    false
                }
                8 | 9 => return,
                _ => true,
            },
            // jt+0x4c8 = attach +8 kind
            7 => (3..5).contains(&self.attach.kind),
            _ => false,
        };
        if !ok {
            return;
        }
        let c = self.cane.jtcs;
        if c == -1 || ((0xc..0x18).contains(&c) && self.cane.f_18e4) {
            if self.joy.FUN_001c46c0(self.t, CANE_WINDOW, BTN_CANE2) != 0 {
                self.FUN_0018e540(-1, anim);
                let n = FUN_0018f050(self);
                self.FUN_0018e540(n, anim);
            } else if self.joy.FUN_001c46c0(self.t, CANE_WINDOW, BTN_CANE) != 0 {
                self.FUN_0018e540(-1, anim);
                let n = FUN_0018ee38(self);
                self.FUN_0018e540(n, anim);
            }
        }
    }

    /// 0x0018e540(jt, jtcs): set the cane sub-state (module doc for the unported parts).
    pub fn FUN_0018e540(&mut self, n: i32, anim: &mut dyn JtAnim) {
        let cur = self.cane.jtcs;
        if cur == n {
            return;
        }
        let chained = n != -1 && cur != -1 && DAT_002BF448.get(cur as usize) == Some(&n);
        if !chained {
            self.cane.geom_off(); // FUN_001f2ad0(jt, jt+0x1a88)
        }
        if (0..=2).contains(&cur) {
            self.calls.push((0x0018e540, cur)); // 0x1ccac0 / 0x1f4bf8 exits (unported)
        }
        // WP-P16: leaving jtcs 1 plays the swing target's +0x130 sound (any chain state)
        if cur == 1 {
            let snd = self.p16.snd_18c4;
            self.FUN_0018e540_sound(snd);
        }
        match n {
            0xc..=0x15 => {
                self.cane.f_18ec = true;
                self.cane.t_19b0 = self.t + DAT_002BF440;
                if n <= 0xd && self.jts == JTS_STAND {
                    self.calls.push((0x001f0400, 0x17b)); // stop the idle cane sound
                }
            }
            0x16 | 0x17 => {
                self.cane.f_18ec = false;
                self.cane.t_19b0 = self.t + DAT_002BF440;
                if self.jts == JTS_STAND {
                    self.calls.push((0x001f0400, 0x17b));
                }
            }
            // WP-P18: case -1 clears +0x18c0
            -1 => self.p18.aim_18c0 = None,
            8 | 9 => {}
            // WP-P18: cases 4..7: FUN_0018f560 aim blend, +0x19b0 = t + DAT_002bf434 (4, 6) / DAT_002bf438 (5, 7)
            4..=7 => {
                self.FUN_0018f560();
                self.cane.t_19b0 = self.t + if n & 1 == 0 { 0.16666667 } else { 0.11666667 };
                self.calls.push((0x0018e540, n)); // rest of the body (sounds 0x1f0400 / 0x1f09d8) recorded
            }
            _ => self.calls.push((0x0018e540, n)), // jtcs 0..3, 10, 11 bodies unported
        }
        // WP-P16: the swing sound of the new jtcs (0 -> 0xf, 3..7 -> 0xb), not when chained
        if !chained {
            self.FUN_0018e540_sound(match n {
                0 => 0xf,
                3..=7 => 0xb,
                _ => -1,
            });
        }
        let field = jtcs_field(n);
        let len = field.and_then(|f| self.cane.data.asegs.get(&f)).map_or(0.0, |a| a.len);
        // chained: the new ASEG starts at min(old time, new length); else the combo window closes
        let t0 = if chained { self.cane.t_aseg.min(len) } else { 0.0 };
        if !chained {
            self.cane.f_18e4 = false;
        }
        if (0xc..=0x17).contains(&n) {
            self.cane.side = n & 1 == 0;
        }
        self.cane.jtcs = n;
        self.cane.t_18dc = self.t;
        self.cane.field = field;
        self.cane.t_aseg = t0;
        if let Some(f) = field {
            // FUN_001305b8(t0, rate, aseg, jt, 1, jt+0x18e0)
            anim.play_aseg(f, self.jts, self.jtbs);
            if !chained {
                self.cane_events(-1.0, 0.0, anim); // events at time 0
            }
        }
    }

    /// FUN_00131290 for the cane ASEG events in (t0, t1] (kinds 0xb / 0xf / 0x10).
    fn cane_events(&mut self, t0: f32, t1: f32, anim: &mut dyn JtAnim) {
        let Some(a) = self.cane.field.and_then(|f| self.cane.data.asegs.get(&f)).cloned() else { return };
        for (t, k, x) in a.ev {
            if !(t0 < t && t <= t1) {
                continue;
            }
            match k {
                0xf if x as u32 == GEOM_CANE => self.cane.geom_on = true, // FUN_001f6ac8
                0x10 if x as u32 == GEOM_CANE => self.cane.geom_off(),  // FUN_001f6b48
                0xb => self.FUN_00188f20_msg9(x, anim),
                _ => {}
            }
        }
    }

    /// FUN_00188f20 msg 9 (ASEG event) for the cane ids.
    pub fn FUN_00188f20_msg9(&mut self, id: i32, _anim: &mut dyn JtAnim) {
        match id {
            EV_COMBO => self.cane.f_18e4 = true,
            EV_HOLD => {
                // WP-P13: SetJtJts(5, 6 / 7) by jtcs (p13)
                self.FUN_00188f20_ev165(_anim);
            }
            EV_18E8 => self.cane.f_18e8 = true,
            // CRAWL R26: 0x167 (pickpocket anim): focus guard jt+0x10dc and +0x1bd4 -> FUN_001f1e20(guard),
            // run by the class set (player::plugin::loot_link); the result's 0x282 / FUN_001424e0 catch in
            // `FUN_00188f20_ev167_done`
            0x167 => {
                if self.p13.f_10dc != 0 && self.cane.f_1bd4 {
                    self.cane.pick_167.push(self.p13.f_10dc);
                }
            }
            _ => self.calls.push((0x00188f20, id)),
        }
    }

    /// FUN_00188f20 event 0x167 tail: FUN_001f1e20's item `obj` (class `cls`); coins (is-a 0x5b / 0x5d)
    /// return, else jt+0x1bcc = obj and FUN_001424e0(jt, FUN_0015e938(1, 0x282, jt), obj, -1) (catch attach,
    /// recorded: the attach kind-0xc body is unported)
    pub fn FUN_00188f20_ev167_done(&mut self, obj: u32, cls: i32) {
        use super::p4::FUN_00132b10;
        if obj == 0 || FUN_00132b10(cls, 0x5b) || FUN_00132b10(cls, 0x5d) {
            return;
        }
        self.cane.f_1bcc = obj;
        self.calls.push((0x001424e0, obj as i32));
    }

    /// FUN_00188f20 msg 0xb (hit feedback, p[1] == jt): kind 2 recoil by side, kind 4 recoil 8.
    pub fn FUN_00188f20_msgb(&mut self, hit: &GuardHit, anim: &mut dyn JtAnim) {
        if hit.f_44 & 2 == 0 {
            return;
        }
        if hit.kind & 2 != 0 {
            if self.cane.jtcs & 1 == 0 {
                self.FUN_0018c250(JTS_STAND, JTBS_NIL, anim);
                self.FUN_0018e540(8, anim);
            } else {
                self.FUN_0018e540(9, anim);
            }
        } else if hit.kind & 4 != 0 {
            self.FUN_0018e540(8, anim);
        }
    }

    /// Advance the cane ASEG by dt (events, msg 3 at the end -> jtcs -1).
    pub fn cane_step(&mut self, dt: f32, anim: &mut dyn JtAnim) {
        let Some(f) = self.cane.field else { return };
        let len = self.cane.data.asegs.get(&f).map_or(0.0, |a| a.len);
        let (t0, t1) = (self.cane.t_aseg, self.cane.t_aseg + dt);
        self.cane.t_aseg = t1;
        self.cane_events(t0, t1, anim);
        if self.cane.field == Some(f) && len <= t1 {
            // msg 3 (ASEG done): jtcs 10 -> 2, else -1
            let n = if self.cane.jtcs == 10 { 2 } else { -1 };
            self.FUN_0018e540(n, anim);
            self.cane.field = None;
        }
    }

    /// REPLACED JT slot 73 FUN_001f7410 for the cane geom (module doc): hits on `guards` (handle, position).
    pub fn FUN_001f7410_cane(&mut self, guards: &[(u32, V3)]) -> Vec<GuardHit> {
        let geoms: Vec<CaneGeom> = [(self.cane.data.geom.clone(), self.cane.geom_on), (self.cane.data.geom_air.clone(), self.cane.geom_air_on)]
            .into_iter()
            .filter_map(|(g, on)| g.filter(|g| on && g.kind != 0))
            .collect();
        let mut out = vec![];
        for g in geoms {
            out.extend(self.FUN_001f7410_geom(&g, guards));
        }
        out
    }

    fn FUN_001f7410_geom(&mut self, g: &CaneGeom, guards: &[(u32, V3)]) -> Vec<GuardHit> {
        let (s, c) = self.yaw.sin_cos();
        let feet = [self.phys.pos[0], self.phys.pos[1], self.phys.pos[2] - super::JT_RADIUS];
        let cw = [feet[0] + c * g.center[0] - s * g.center[1], feet[1] + s * g.center[0] + c * g.center[1], feet[2] + g.center[2]];
        let mut out = vec![];
        for &(h, p) in guards {
            let d = [p[0] - cw[0], p[1] - cw[1], p[2] - cw[2]];
            let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            if g.radius < l || self.cane.hit_done.contains(&h) {
                continue;
            }
            self.cane.hit_done.push(h); // FUN_001f7598 dedupe (0x3e15c0)
            let n = if l > 1e-4 { [d[0] / l, d[1] / l, d[2] / l] } else { [1.0, 0.0, 0.0] };
            out.push(GuardHit {
                src: PLAYER_HANDLE,
                target: h,
                kind: g.kind,
                amount: g.amount as i32,
                pos: [p[0], p[1], p[2], 1.0],
                vec: [n[0], n[1], n[2], 0.0],
                f_44: CANE_RESPONSE,
            });
        }
        out
    }
}

/// DAT_002bf820: smp of the jt+0x1010 decay in the ledge hang (0x197500 case 0x11).
pub const DAT_002BF820: [f32; 3] = [1000.0, 0.0, 0.25];

/// WP-P11 hook into the class set (REPLACED contact path, module doc): cane hits on the set's guards, each
/// delivered by FUN_001f7598 (`ClassSet::ee_sub_1f7598` -> guard slot 74 FUN_00207468); an accepted hit with
/// response & 0xf8 sends msg 0xb back to the JT (FUN_00188f20). Returns the hits built this frame.
pub fn cane_hits_to_guards(
    jt: &mut Jt,
    set: &mut crate::engine::classes::ClassSet,
    h: &mut dyn crate::engine::classes::ClassHost,
    anim: &mut dyn JtAnim,
) -> Vec<GuardHit> {
    use crate::engine::classes::Inst;
    if !jt.cane.geom_on && !jt.cane.geom_air_on {
        return vec![];
    }
    let guards: Vec<(u32, V3)> =
        set.k6.iter().filter_map(|(i, x)| if let Inst::Guard(g) = x { Some((*i as u32 + 1, g.body.pos)) } else { None }).collect();
    // WIRE-EP8: class-98 staged switches take the cane hit too (slot 74 FUN_001badf0; cellar exit 382).
    // REPLACED: FUN_001f7410's geometry query -> sphere vs the object origin, as for the guards.
    let mut guards = guards;
    for (i, x) in &set.k5 {
        if matches!(x, Inst::C2d4b78(_)) {
            guards.push((*i as u32 + 1, h.ee_sub_1a62e8(*i as u32 + 1)));
        }
    }
    // WIRE-EP8: class-83 switches in hit mode (+0x2a8 1 / 2) take it too (slot 74 FUN_00148f20; hub 142627)
    for (i, x) in &set.k15 {
        if matches!(x, Inst::K15d(c) if c.f_2a8 == 1 || c.f_2a8 == 2) {
            guards.push((*i as u32 + 1, h.ee_sub_1a62e8(*i as u32 + 1)));
        }
    }
    let hits = jt.FUN_001f7410_cane(&guards);
    for x in &hits {
        if set.ee_sub_1f7598(x, h) && x.f_44 & 0xf8 != 0 {
            jt.FUN_00188f20_msgb(x, anim);
        }
    }
    hits
}
