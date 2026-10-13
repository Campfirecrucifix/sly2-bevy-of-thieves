//! WP-A14: the four HUD slots world+0x3ba0 (FUN_0020f710 / FUN_0020f7e0, `game::world_update`) and the
//! class-126 label state machine of their screen objects.
//!
//! Slots (DAT_002c3e70 oids, kind 0x405): 0 = 0x371 the HUD meter screen (FUN_001da2e0, `ui::meter`),
//! 1 = 0x36d coins (+0x3bac, FUN_00161868), 2 = 0x36e clue bottles (+0x3bb8, FUN_001618b0), 3 = 0x36f
//! countdown (+0x3bc4, FUN_001618f8 start / FUN_00161958 stop, world+0x476c end time).
//!
//! 1:1: FUN_002128e8 (vt+0x18c show), FUN_002129e8 (vt+0x194 hide), FUN_00212a30 (restart the hold),
//! FUN_00212a50 (request), FUN_00212a70 (state change: 0 off, 1 idle, 2 hold anim +0x70, 3 enter anim
//! +0x74, 4 exit anim +0x78), FUN_00212c50 (vt slot 15 per frame), FUN_002132d0 (shown), FUN_00212298
//! anim defaults (0x310 / 0x30f / 0x311), FUN_00161868 / FUN_001618b0 / FUN_001618f8 / FUN_00161958.
//! REPLACED: the label ASEG animations: "anim playing" (FUN_002121e8 / FUN_00218738 / FUN_00218710) is an
//! anim timer of fixed length (`ANIM_ENTER` / `ANIM_HOLD` / `ANIM_EXIT`), FUN_002142b0 (anim present) is
//! true for 0x30f / 0x310 / 0x311; FUN_002137f0 (a higher screen of DAT_003e1060 covers the label) is the
//! caller's `covered` (a menu is open). NOT modelled: the DAT_003e107c / DAT_002dd318 counters and their
//! FUN_0014d1d0 bus duck, FUN_002124f0 / FUN_00212780 / FUN_00213898 / FUN_00213830 screen-element
//! setup (vt+0xac / vt+0x1c messages), the +0x8c hold time property (`hold`, default 0 = the hold anim
//! decides, as the property is not loaded by the port).
#![allow(non_snake_case)]
use crate::engine::game::world_update::{self, Slot, SlotHost};

/// REPLACED: label animation lengths (enter 0x30f, hold 0x310 / 0x314, exit 0x311), seconds.
pub const ANIM_ENTER: f32 = 0.25;
pub const ANIM_HOLD: f32 = 3.0;
pub const ANIM_EXIT: f32 = 0.25;

/// One class-126 label (the fields FUN_00212c50 reads).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Label {
    /// +0x58 state, +0x5c state time, +0x60 state frame (DAT_002be538)
    pub state: i32,
    pub t_state: f32,
    /// +0x64 request, +0x68 request time
    pub req: i32,
    pub t_req: f32,
    /// +0x70 hold / +0x74 enter / +0x78 exit anims (-1 none)
    pub anims: [i32; 3],
    /// +0x7c playing anim (-1 none), +0x88 its time
    pub anim: i32,
    pub t_anim: f32,
    /// +0x8c hold time (0 = until the hold anim ends)
    pub hold: f32,
    /// +0xf8 -> +0x14 slide offset (FUN_0020f7e0)
    pub offset: f32,
}

impl Default for Label {
    fn default() -> Self {
        // FUN_00212298 (screen object: state 1) + ctor FUN_00211c58
        Self { state: 1, t_state: 0.0, req: 0, t_req: -1.0, anims: [0x310, 0x30f, 0x311], anim: -1, t_anim: 0.0, hold: 0.0, offset: 0.0 }
    }
}

/// REPLACED FUN_002142b0: the label owns the default anims.
fn has_anim(id: i32) -> bool {
    matches!(id, 0x30f..=0x311)
}

impl Label {
    /// REPLACED FUN_002121e8(o, id): `id` is the playing anim and its timer has not run out.
    pub fn FUN_002121e8(&self, id: i32) -> bool {
        // 0x002121e8
        if id == -1 || self.anim == -1 || id != self.anim {
            return false;
        }
        let len = match id {
            0x30f => ANIM_ENTER,
            0x311 => ANIM_EXIT,
            _ => ANIM_HOLD,
        };
        self.t_anim < len
    }

    /// FUN_00212a50(o, r): new request, stamped with the time.
    pub fn FUN_00212a50(&mut self, r: i32, now: f32) {
        // 0x00212a50
        if self.req != r {
            self.req = r;
            self.t_req = now;
        }
    }

    /// FUN_00212a30(o): restart the hold while in state 2.
    pub fn FUN_00212a30(&mut self, now: f32) {
        // 0x00212a30
        if self.state == 2 {
            self.t_state = now;
        }
    }

    /// FUN_00212a70(o, s): enter state `s` (anim 0x70 / 0x74 / 0x78 for 2 / 3 / 4, none for 1).
    pub fn FUN_00212a70(&mut self, s: i32, now: f32) {
        // 0x00212a70
        if self.state == s {
            return;
        }
        self.state = s;
        self.t_state = now;
        match s {
            // FUN_00212780 / FUN_00213898: no anim
            0 | 1 => {
                self.anim = -1;
                self.t_anim = 0.0;
            }
            // FUN_00213830(o, anim)
            2..=4 => {
                self.anim = self.anims[(s - 2) as usize];
                self.t_anim = 0.0;
            }
            _ => {}
        }
    }

    /// FUN_002128e8(o, anim) (vt+0x18c): show with hold anim `anim` (-1 = the defaults).
    pub fn FUN_002128e8(&mut self, anim: i32, now: f32) {
        // 0x002128e8
        if self.state == 0 {
            self.FUN_00212a70(1, now);
        }
        self.FUN_00212a50(2, now);
        self.anims = [anim, -1, -1];
        if anim == -1 {
            self.anims[0] = 0x314;
            if has_anim(0x30f) {
                self.anims[1] = 0x30f;
            }
            if has_anim(0x310) {
                self.anims[0] = 0x310;
            }
            if has_anim(0x311) {
                self.anims[2] = 0x311;
            }
        }
        // state 1: vt+0x84 (FUN_00212c50) at once
        if self.state == 1 {
            self.FUN_00212c50(now, 0.0);
        }
    }

    /// FUN_002129e8(o) (vt+0x194): hide.
    pub fn FUN_002129e8(&mut self, now: f32) {
        // 0x002129e8
        self.FUN_00212a50(1, now);
        if self.anims[2] == -1 {
            self.FUN_00212a70(1, now);
        }
    }

    /// FUN_00212c50(o) (slot 15): `now` = DAT_002ddf0c, `dt` = DAT_002ddf10.
    pub fn FUN_00212c50(&mut self, now: f32, dt: f32) {
        // 0x00212c50
        let s = self.state;
        let mut next = s;
        let to = |a: i32, yes: i32, no: i32| if a == -1 { no } else { yes };
        if s == 1 {
            if self.req == 2 {
                next = to(self.anims[1], 3, 2);
            }
        } else if self.t_req != now {
            match s {
                3 => {
                    if !self.FUN_002121e8(self.anims[1]) {
                        match self.req {
                            1 => next = to(self.anims[2], 4, 1),
                            2 => next = 2,
                            _ => {}
                        }
                    }
                }
                2 => {
                    if !self.FUN_002121e8(self.anim) {
                        self.FUN_00212a50(1, now);
                    }
                    if 0.0 < self.hold && self.hold < now - self.t_state {
                        self.FUN_00212a50(1, now);
                    }
                    if self.req == 1 {
                        next = to(self.anims[2], 4, 1);
                    }
                }
                4 => {
                    if !self.FUN_002121e8(self.anims[2]) {
                        match self.req {
                            1 => next = 1,
                            2 => next = to(self.anims[1], 3, 2),
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
        self.FUN_00212a70(next, now);
        if self.anim != -1 {
            self.t_anim += dt;
        }
    }

    /// FUN_002132d0(o): an anim plays and no higher screen covers the label.
    pub fn FUN_002132d0(&self, covered: bool) -> bool {
        // 0x002132d0
        self.anim != -1 && !covered
    }
}

/// The world+0x3ba0 slots with their labels; slot 0 (the meter screen) is driven by `ui::meter`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HudSlots {
    pub slots: [Slot; 4],
    pub labels: [Label; 4],
    /// world+0x476c countdown end, +0x4770 shown seconds
    pub end: f32,
    pub shown_secs: u32,
    /// slot 0 shown (the meter object decides)
    pub meter_shown: bool,
    /// FUN_001fb2f8 sound requests of FUN_0020f7e0 (0x30 / 0x31 tick-tock), drained by the caller
    pub sounds: Vec<u32>,
}

struct Host<'a> {
    labels: &'a mut [Label; 4],
    meter_shown: bool,
    covered: bool,
    sounds: &'a mut Vec<u32>,
}

impl SlotHost for Host<'_> {
    fn shown(&self, obj: u32) -> bool {
        let i = obj as usize - 1;
        if i == 0 {
            return self.meter_shown && !self.covered;
        }
        self.labels[i].FUN_002132d0(self.covered)
    }
    fn offset(&mut self, obj: u32) -> &mut f32 {
        &mut self.labels[obj as usize - 1].offset
    }
    fn sound(&mut self, id: u32) {
        self.sounds.push(id);
    }
}

impl HudSlots {
    /// FUN_0020f710 on level start: the four slot objects exist on every level (oids 0x371 / 0x36d /
    /// 0x36e / 0x36f, class 126); handle = slot index + 1.
    pub fn FUN_0020f710(&mut self) {
        let mut i = 0;
        world_update::ee_sub_20f710(&mut self.slots, |_| {
            i += 1;
            Some(i)
        });
        self.labels = Default::default();
        self.end = 0.0;
    }

    /// FUN_00161868(world): coin counter pops up (vt+0x18c(-1) + FUN_00212a30).
    pub fn FUN_00161868(&mut self, now: f32) {
        // 0x00161868
        let l = &mut self.labels[1];
        l.FUN_002128e8(-1, now);
        l.FUN_00212a30(now);
    }

    /// FUN_001618b0(world): clue-bottle counter pops up.
    pub fn FUN_001618b0(&mut self, now: f32) {
        // 0x001618b0
        let l = &mut self.labels[2];
        l.FUN_002128e8(-1, now);
        l.FUN_00212a30(now);
    }

    /// FUN_001618f8(t, world): countdown of `t` seconds (hold time 0 = until stopped).
    pub fn FUN_001618f8(&mut self, t: f32, now: f32) {
        // 0x001618f8
        let l = &mut self.labels[3];
        l.hold = 0.0;
        l.FUN_002128e8(-1, now);
        self.end = now + t;
    }

    /// FUN_00161958(world): countdown off.
    pub fn FUN_00161958(&mut self, now: f32) {
        // 0x00161958
        self.labels[3].FUN_002129e8(now);
        self.end = 0.0;
    }

    /// Per frame: label slot 15 updates (FUN_00212c50, part of the object update) then FUN_0020f7e0.
    pub fn update(&mut self, now: f32, dt: f32, covered: bool) {
        for l in &mut self.labels[1..] {
            l.FUN_00212c50(now, dt);
            // the countdown label holds while the countdown runs (anim 0x310 loops)
        }
        if self.end > now && self.labels[3].state == 2 {
            self.labels[3].t_anim = 0.0;
        }
        let mut h = Host { labels: &mut self.labels, meter_shown: self.meter_shown, covered, sounds: &mut self.sounds };
        world_update::FUN_0020f7e0(&mut self.slots, &mut h, now, dt, self.end, &mut self.shown_secs);
    }

    /// Slot `i` drawn this frame and its slide offset.
    pub fn shown(&self, i: usize, covered: bool) -> Option<f32> {
        let on = if i == 0 { self.meter_shown && !covered } else { self.labels[i].FUN_002132d0(covered) };
        (on && self.slots[i].obj != 0).then_some(self.labels[i].offset)
    }
}
