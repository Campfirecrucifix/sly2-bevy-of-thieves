#![allow(non_snake_case)]
//! WP-A12: HUD meter (health + gadget power), object 0x3d9e80 of the HUD set FUN_001d9520 (init) /
//! FUN_001d9648 (per frame).
//!
//! 1:1: FUN_001d9968 (init), FUN_001d9df8 (update state machine), FUN_001d9b60 (state change) -- shown /
//! target fractions of health (*(jt+0xe00) / jt+0xdfc) and power (FUN_001441a0 / jt+0x1300), rising
//! values jump, falling ones run down with FUN_0011d018(cur, target, DAT_002dded4, 0x2c3248 = {100, 0,
//! 0.5, 5}), equality FUN_0011d4d0(.., 1e-4); the power fill 0x350 is shown only when a gadget of the
//! character's gadget-menu list (Sly 0x2c2de8, Murray 0x2c3028, Bentley 0x2c3148) is acquired and costs
//! power (DAT_002bcd0c > 0). Animation ids per character: Sly 0x33d.., Bentley 0x342.., Murray 0x347..
//! REPLACED: the meter is the level's screen object 0x36b (FUN_0015e938(0x3dd490, 0x405, 0x36b)) whose
//! fills 0x34f / 0x350 are timelines scrubbed to 1 - fraction (FUN_00212e40) and whose state animations
//! play through vt+0x18c; the port keeps the fractions / shown flags / last animation id and `ui::hud::
//! meter` draws two bars. FUN_002121e8 (animation still playing) = false (no screen timelines), jt+0x10c8
//! = -1 (not tracked by the port JT).
//! The original HUD has no refusal flash and no L1/L2/R2 slot icons: jt+0x1304 is read only by the gadget
//! code (0x13ff70 / 0x141a50 / 0x141cd0) and the slots jt+0x1180 only by the gadget menu FUN_001dac58.
use crate::engine::camera::math::{FUN_0011d018, Smp};
use crate::engine::player::p18_gadget::GADGETS;
use crate::engine::player::p4::FUN_00132b10;

/// DAT_002c3248: approach parameters of the falling meters.
pub const DAT_002C3248: Smp = [100.0, 0.0, 0.5, 5.0];
/// FUN_0011d4d0 epsilon 0x38d1b717.
pub const METER_EPS: f32 = 1.0e-4;
/// Gadget ids of the gadget-menu lists (first word of the 0x20-byte entries).
pub const DAT_002C2DE8: [i32; 0x12] = [23, 24, 25, 26, 29, 30, 33, 34, 35, 37, 38, 39, 27, 28, 40, 41, 42, 31];
pub const DAT_002C3028: [i32; 9] = [7, 9, 11, 13, 10, 12, 8, 14, 36];
pub const DAT_002C3148: [i32; 8] = [15, 22, 20, 19, 16, 21, 17, 18];

/// What the meter reads from the player JT (FUN_001c4840 focus, is-a 0x34).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MeterJt {
    /// JT identity (the +0x34 pointer; changes on a character switch)
    pub id: u32,
    /// class id for FUN_00132b10 (Sly 0x35, Bentley 0x36, Murray 0x37)
    pub cls: i32,
    /// *(jt+0xe00) / jt+0xdfc
    pub hp: i32,
    pub hp_max: i32,
    /// FUN_001441a0 / jt+0x1300
    pub mp: i32,
    pub mp_max: i32,
    /// jt+0x10ec focus mode, jt+0x10c8
    pub f_10ec: i32,
    pub f_10c8: i32,
    /// FUN_001daea8 bits ("mppukfAcquired")
    pub acquired: u64,
}

impl MeterJt {
    fn health(&self) -> f32 {
        self.hp as f32 / self.hp_max as f32
    }
    fn power(&self) -> f32 {
        self.mp as f32 / self.mp_max as f32
    }
    fn is(&self, id: i32) -> bool {
        FUN_00132b10(self.cls, id)
    }
}

/// Object 0x3d9e80.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HudMeter {
    /// +0x20 / +0x24 health shown / target, +0x28 / +0x2c power shown / target
    pub health: f32,
    pub health_t: f32,
    pub power: f32,
    pub power_t: f32,
    /// +0x30 state, +0x34 jt, +0x38 screen object present
    pub state: i32,
    pub jt: Option<MeterJt>,
    pub screen: bool,
    /// +0x3c..+0x4c animation ids
    pub anims: [u32; 5],
    /// port view of the screen: fills 0x34f / 0x350 shown (FUN_00213b00 / FUN_00213928), last animation
    pub health_shown: bool,
    pub power_shown: bool,
    pub anim: Option<u32>,
}

/// FUN_0011d4d0: |a - b| / max(1, |a|) < eps (relative; physics::driver's copy is absolute -- see A12 Findings).
pub fn FUN_0011d4d0(a: f32, b: f32, eps: f32) -> bool {
    let m = a.abs();
    let d = if m < 1.0 { 1.0 } else { m };
    (a - b).abs() / d < eps
}

fn anims_of(jt: &MeterJt, first: i32) -> Option<[u32; 5]> {
    let set = |b: u32| [b, b + 1, b + 2, b + 3, b + 4];
    match first {
        // FUN_001d9968 order: 0x37, 0x36, else Sly
        0 => Some(if jt.is(0x37) { set(0x347) } else if jt.is(0x36) { set(0x342) } else { set(0x33d) }),
        // FUN_001d9df8 order: 0x35, 0x37, 0x36, else unchanged
        _ => {
            if jt.is(0x35) {
                Some(set(0x33d))
            } else if jt.is(0x37) {
                Some(set(0x347))
            } else if jt.is(0x36) {
                Some(set(0x342))
            } else {
                None
            }
        }
    }
}

impl HudMeter {
    /// vt+0x18c on the screen object: play animation `id`.
    fn play(&mut self, id: u32) {
        self.anim = Some(id);
    }

    /// FUN_002121e8(screen, anim): animation still running. REPLACED: no screen timelines -> false.
    fn playing(&self, _id: u32) -> bool {
        false
    }

    /// 0x001d9968
    pub fn FUN_001d9968(&mut self, jt: Option<MeterJt>, screen: bool) {
        self.state = 0;
        self.jt = None;
        self.screen = screen;
        if let Some(j) = jt.filter(|j| j.is(0x34)) {
            self.jt = Some(j);
        }
        let Some(j) = self.jt else {
            (self.power, self.health_t, self.health, self.power_t) = (0.0, 0.0, 0.0, 0.0);
            return;
        };
        self.health = j.health();
        self.health_t = self.health;
        self.power = j.power();
        self.power_t = self.power;
        self.anims = anims_of(&j, 0).unwrap();
        if self.screen {
            self.play(self.anims[0]);
            self.health_shown = true;
            self.power_shown = true;
        }
    }

    /// 0x001d9b60
    pub fn FUN_001d9b60(&mut self, s: i32) {
        if self.state == s || !self.screen {
            return;
        }
        if self.state == 2 || self.state == 6 {
            self.play(self.anims[0]);
        }
        self.state = s;
        let a = match s {
            0 | 5 => self.anims[2],
            2 => self.anims[1],
            6 => self.anims[3],
            7 => {
                self.play(self.anims[4]);
                return;
            }
            1 => {
                self.play(self.anims[0]);
                self.health_shown = true;
                let Some(j) = self.jt else { return };
                let list: &[i32] = if j.is(0x35) {
                    &DAT_002C2DE8
                } else if j.is(0x37) {
                    &DAT_002C3028
                } else if j.is(0x36) {
                    &DAT_002C3148
                } else {
                    &[]
                };
                self.power_shown = list.iter().any(|&g| g >= 0 && j.acquired >> g & 1 != 0 && GADGETS[g as usize].cost > 0);
                return;
            }
            _ => return,
        };
        self.play(a);
    }

    /// 0x001d9df8 (`dt` = DAT_002dded4)
    pub fn FUN_001d9df8(&mut self, jt: Option<MeterJt>, dt: f32) {
        if !self.screen {
            return;
        }
        let prev = self.jt.map(|j| j.id);
        self.jt = jt.filter(|j| j.is(0x34));
        let Some(j) = self.jt else {
            (self.power, self.health_t, self.health, self.power_t) = (0.0, 0.0, 0.0, 0.0);
            if prev.is_some() {
                self.FUN_001d9b60(0);
            }
            return;
        };
        let st = if prev == Some(j.id) {
            self.state
        } else {
            self.FUN_001d9b60(if prev.is_none() { 3 } else { 5 });
            if let Some(a) = anims_of(&j, 1) {
                self.anims = a;
            }
            self.health = j.health();
            self.health_t = self.health;
            self.power = j.power();
            self.power_t = self.power;
            self.state
        };
        match st {
            0 => {
                self.health_t = j.health();
                self.power_t = j.power();
                if self.health_t <= self.health {
                    return;
                }
                self.health = self.health_t;
                self.power = self.power_t;
            }
            1 | 6 | 7 => {
                self.health_t = j.health();
                self.power_t = j.power();
                if j.hp < 1 {
                    return self.FUN_001d9b60(0);
                }
                if self.health_t < self.health {
                    return self.FUN_001d9b60(2);
                }
                if self.health < self.health_t {
                    return self.FUN_001d9b60(3);
                }
                if !FUN_0011d4d0(self.power, self.power_t, METER_EPS) {
                    return self.FUN_001d9b60(4);
                }
                if j.f_10ec == 4 {
                    return self.FUN_001d9b60(7);
                }
                if j.f_10c8 != -1 {
                    return self.FUN_001d9b60(6);
                }
            }
            2 => {
                if self.playing(self.anims[1]) {
                    return;
                }
                return self.FUN_001d9b60(3);
            }
            3 => {
                self.health_t = j.health();
                if self.health < self.health_t {
                    self.health = self.health_t;
                } else {
                    self.health = FUN_0011d018(self.health, self.health_t, dt, &DAT_002C3248).0;
                }
                if !FUN_0011d4d0(self.health, self.health_t, METER_EPS) {
                    return;
                }
            }
            4 => {
                self.power_t = j.power();
                if self.power < self.power_t {
                    self.power = self.power_t;
                } else {
                    self.power = FUN_0011d018(self.power, self.power_t, dt, &DAT_002C3248).0;
                }
                if !FUN_0011d4d0(self.power, self.power_t, METER_EPS) {
                    return;
                }
            }
            5 => {
                if self.playing(self.anims[2]) {
                    return;
                }
            }
            _ => return,
        }
        self.FUN_001d9b60(1);
    }
}
