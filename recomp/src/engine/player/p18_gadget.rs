//! WP-P18: gadgets ("puk" in the save data) on the JT.
//!
//! Table DAT_002bcd00 (0x2b entries x 0x20, `GADGETS`): +0 button mode (2 one-shot, 1 hold, -1 not on a slot
//! button), +4 owner character (7 Sly, 8 Bentley, 9 Murray), +8 shop price, +0xc power cost (DAT_002bcd0c),
//! +0x10 shop order, +0x14 / +0x18 name / description string ids, +0x1c icon.
//! Slots jt+0x1180 3 x {+0 gadget, +4 state, +8 time} on the buttons DAT_002c3258 = {L1 4, L2 1, R2 2}; the
//! equipped ids are the save variables "apukCur[%d]" (0x141ef8 load / 0x141fc8 equip, written by the gadget
//! menu C2dc018 slot 7 0x1dac58 = ui), acquired = "powerUps" / "mppukfAcquired[%d]" (FUN_001daea8).
//! Power ("mpCur", jt+0x12fc pool pointer, max jt+0x1300 = 100 from 0x13ff70, refill 0x1440d0) is spent by
//! FUN_00141cd0; jt+0x1304 = last use / refusal time (HUD flash), jt+0x1308 fractional drain.
//! Per frame (0x1403d0 JT parent update) FUN_00141a50 polls the slots: state 0 + button within 0.15 s ->
//! vt+0x31c can-use (JT slot 98 0x18d760) ? (power < cost ? sound 0x2e + +0x1304 : FUN_00141c08(mode)) :
//! sound 0x2e; state 1 (hold) -> button held && can-use ? vt+0x32c (slot 100 0x141dd0 drain) :
//! FUN_00141c08(0). FUN_00141c08: leaving 1 -> vt+0x334 (slot 101 0x18dd78 release); entering 1 -> vt+0x324
//! (slot 99 0x18da48 use); 2 -> use, back to 0.
//! Air R1 (0x197500 head): jt == DAT_002e1e40 && acquired(0x1b) && !FUN_00143d20(0.3) -> spend, consume 8,
//! use(0x1b) = paraglider 0x195fb8(jmp, 0x13). Glide: 0x197f98 case 0x14 (stick turning 5 / 12 rad/s, speed
//! targets jmp+0x1c0 / +0x1c4) + 0x196df8 case 0x14 force (`glide_force`).
//! REPLACED: the scene / partner tests of can-use (FUN_0014fd50, jt+0x12f0, jt+0x2010 FUN_001cb2c0, carried
//! object 0x18d6f8 + 0x1cade0 / 0x224590, jmp+0x2c) = false; the clock rate 0x14d1d0 of gadgets 0x1d / 0x2a
//! is stored in `Gadgets::time_scale` (DAT_002ddeec, the game clock owner must apply it); sounds are queued
//! ids in `Gadgets::sfx`; event 0x3c on 0x2deb30 and the carry / throw bodies are recorded in `Jt::calls`.
//! The glide forward axis is the facing (FUN_001a6fc0 body frame and the half-step angular prediction).
use super::{Jt, JtAnim};
use crate::engine::physics::geom::V3;

/// DAT_002bcd00 entry.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GadgetDef {
    pub mode: i32,
    pub owner: i32,
    pub price: i32,
    pub cost: i32,
    pub order: i32,
    pub name: u32,
    pub desc: u32,
    pub icon: u32,
}

const fn g(mode: i32, owner: i32, price: i32, cost: i32, order: i32, name: u32, desc: u32, icon: u32) -> GadgetDef {
    GadgetDef { mode, owner, price, cost, order, name, desc, icon }
}

/// DAT_002bcd00 (0x2b x 0x20, read from SCUS_973.16).
pub const GADGETS: [GadgetDef; 0x2b] = [
    g(-1, 0, 0, 0, 0, 0, 0, 0),
    g(-1, 0, 0, 0, 0, 0, 0, 0),
    g(-1, 0, 0, 0, 0, 0, 0, 0),
    g(-1, 0, 0, 0, 0, 0, 0, 0),
    g(2, 0, 0, 0, 0, 0, 0, 0),
    g(2, 0, 0, 0, 0, 0, 0, 0),
    g(1, 0, 0, 0, 0, 0, 0, 0),
    g(2, 8, 200, 0, 1, 0x81, 0x82, 0xd1),
    g(2, 8, 500, 5, 2, 0x83, 0x84, 0xcd),
    g(2, 8, 550, 5, 3, 0x85, 0x86, 0xcc),
    g(2, 8, 1000, 15, 4, 0x87, 0x88, 0xcd),
    g(2, 8, 1050, 20, 5, 0x89, 0x8a, 0xd1),
    g(2, 8, 1200, 1, 6, 0x8b, 0x8c, 0xd0),
    g(2, 8, 1700, 20, 7, 0x8d, 0x8e, 0xcc),
    g(2, 8, 2000, 25, 8, 0x8f, 0x90, 0xcd),
    g(2, 9, 400, 10, 1, 0x93, 0x94, 0xcd),
    g(2, 9, 400, 0, 2, 0x95, 0x96, 0xd4),
    g(-1, 9, 750, 0, 3, 0x97, 0x98, 0xce),
    g(-1, 9, 900, 0, 4, 0x99, 0x9a, 0xce),
    g(2, 9, 2100, 20, 8, 0x9b, 0x9c, 0xd6),
    g(2, 9, 1400, 10, 6, 0x9d, 0x9e, 0xcd),
    g(2, 9, 1500, 15, 7, 0x9f, 0xa0, 0xcd),
    g(2, 9, 1150, 25, 5, 0xa1, 0xa2, 0xcd),
    g(2, 7, 300, 30, 1, 0xa3, 0xa4, 0xcd),
    g(2, 7, 600, 0, 2, 0xa5, 0xa6, 0xd7),
    g(2, 7, 650, 1, 3, 0xa7, 0xa8, 0xd0),
    g(2, 7, 600, 5, 4, 0xa9, 0xaa, 0xd8),
    g(-1, 7, 600, 0, 5, 0xab, 0xac, 0xd3),
    g(-1, 7, 1300, 0, 6, 0xad, 0xae, 0xd2),
    g(1, 7, 1600, 2, 7, 0xaf, 0xb0, 0xd0),
    g(2, 7, 1900, 0, 8, 0xb1, 0xb2, 0xcd),
    g(2, 7, 0, 0, -1, 0xb3, 0xb4, 0xcd),
    g(-1, 7, 0, 0, -1, 0xb5, 0xb6, 0xcd),
    g(2, 7, 0, 10, -1, 0xb7, 0xb8, 0xcd),
    g(2, 7, 0, 10, -1, 0xb9, 0xba, 0xcd),
    g(2, 7, 0, 10, -1, 0xbb, 0xbc, 0xcd),
    g(-1, 7, 0, 0, -1, 0xbd, 0xbe, 0xce),
    g(2, 7, 0, 25, -1, 0xbf, 0xc0, 0xd8),
    g(2, 7, 0, 15, -1, 0xc1, 0xc2, 0xd8),
    g(2, 7, 0, 20, -1, 0xc3, 0xc4, 0xcd),
    g(2, 7, 0, 3, -1, 0xc5, 0xc6, 0xd0),
    g(2, 7, 0, 5, -1, 0xc7, 0xc8, 0xd1),
    g(1, 7, 0, 1, -1, 0xc9, 0xca, 0xd0),
];

/// DAT_002c3258: slot buttons L1, L2, R2.
pub const DAT_002C3258: [u32; 3] = [0x4, 0x1, 0x2];
/// Paraglider (R1 in the air, 0x197500).
pub const GADGET_PARAGLIDER: i32 = 0x1b;
/// FUN_001fbb90(0x2e): refusal sound.
pub const SND_REFUSE: u32 = 0x2e;
/// DAT_002bd260 / DAT_002bd264: power drain per second while 0x1d / 0x2a are held.
pub const DAT_002BD260: f32 = 2.0;
pub const DAT_002BD264: f32 = 1.0;
/// DAT_002bf3ec / DAT_002bf3f0: clock rates of 0x1d / 0x2a (0x14d1d0).
pub const DAT_002BF3EC: f32 = 0.25;
pub const DAT_002BF3F0: f32 = 1.35;
/// 0x197f98 case 0x14: speed targets (DAT_002bf850..85c), turn rates (DAT_002bf870 / 874).
pub const DAT_002BF850: [f32; 4] = [300.0, 800.0, -200.0, -400.0];
pub const DAT_002BF870: [f32; 2] = [5.0, 12.0];
/// 0x196df8 case 0x14 approach params 0x2bf7a8 / 0x2bf7b8 / 0x2bf7c8 ({vmax, v0, accel}) and |a| limit 0x2bf7d4.
pub const GLIDE_PRM: [[f32; 3]; 3] = [[800.0, 0.0, 0.1], [800.0, 0.0, 0.1], [5000.0, 0.0, 0.1]];
pub const DAT_002BF7D4: f32 = 8000.0;

/// jt+0x1180 entry.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GadgetSlot {
    pub id: i32,
    pub state: i32,
    pub t: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Gadgets {
    pub slots: [GadgetSlot; 3],
    /// *(jt+0x12fc) "mpCur" (None = no pool: 0x1441a0 reads the max)
    pub mp: Option<i32>,
    /// jt+0x1300
    pub mp_max: i32,
    /// jt+0x1304 (init -60.0)
    pub t_1304: f32,
    /// jt+0x1308
    pub frac: f32,
    /// "mppukfAcquired[%d]" bits
    pub acquired: u64,
    /// DAT_002ddeec clock rate set by 0x14d1d0 (1 = normal)
    pub time_scale: f32,
    /// DAT_003e1bec
    pub f_3e1bec: f32,
    /// jt+0x2044 looping sound (0x258040)
    pub loop_snd: Option<u32>,
    /// jt+0x9bc / jt+0x130c (0x22 / 0x23 / 0x141d38 attach 0xb param)
    pub f_9bc: i32,
    /// queued FUN_001fbb90 sound ids
    pub sfx: Vec<u32>,
    /// gadgets used (vt+0x324), for diagnostics / tests
    pub used: Vec<i32>,
}

impl Default for Gadgets {
    fn default() -> Self {
        Gadgets {
            slots: [GadgetSlot { id: -1, state: 0, t: 0.0 }; 3],
            mp: Some(100),
            mp_max: 100,
            t_1304: -60.0,
            frac: 0.0,
            acquired: 0,
            time_scale: 1.0,
            f_3e1bec: 0.0,
            loop_snd: None,
            f_9bc: 0,
            sfx: vec![],
            used: vec![],
        }
    }
}

impl Gadgets {
    /// 0x001441a0
    pub fn FUN_001441a0(&self) -> i32 {
        self.mp.unwrap_or(self.mp_max)
    }
    /// 0x001441c0
    pub fn FUN_001441c0(&mut self, v: i32) {
        if let Some(m) = self.mp.as_mut() {
            *m = v.min(self.mp_max);
        }
    }
    /// 0x001441e8(rate): +0x1308 += rate * dt; whole units leave the pool.
    pub fn FUN_001441e8(&mut self, rate: f32, dt: f32) {
        self.frac += rate * dt;
        let n = self.frac as i32;
        if n != 0 {
            self.frac -= n as f32;
            let v = (self.FUN_001441a0() - n).max(0);
            self.FUN_001441c0(v);
        }
    }
    /// 0x00141cd0: spend the cost of `g` (-1: none), stamp +0x1304.
    pub fn FUN_00141cd0(&mut self, g: i32, t: f32) {
        if g != -1 {
            let v = self.FUN_001441a0() - GADGETS[g as usize].cost;
            self.FUN_001441c0(v);
        }
        self.t_1304 = t;
    }
    /// 0x00141e78: slot holding `g` or -1.
    pub fn FUN_00141e78(&self, g: i32) -> i32 {
        self.slots.iter().position(|s| s.id == g).map_or(-1, |i| i as i32)
    }
    /// 0x00141ea8: button mask of the slots holding `g`.
    pub fn FUN_00141ea8(&self, g: i32) -> u32 {
        (0..3).filter(|&i| self.slots[i].id == g).fold(0, |m, i| m | DAT_002C3258[i])
    }
    /// 0x001daea8: acquired flag ("powerUps" / "mppukfAcquired[g]"), -1 -> 0.
    pub fn FUN_001daea8(&self, g: i32) -> bool {
        (0..64).contains(&g) && self.acquired >> g & 1 != 0
    }
    /// 0x001440d0 power part: refill when hp is 0 or `force`.
    pub fn refill(&mut self, force: bool, hp_zero: bool) {
        if hp_zero || force {
            let m = self.mp_max;
            self.FUN_001441c0(m);
            self.frac = 0.0;
        }
    }
}

/// HUD view of the gadget state (published by the player plugin; ui reads it).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GadgetHud {
    pub mp: i32,
    pub mp_max: i32,
    /// slot gadget ids (L1, L2, R2) and their icon ids (GADGETS +0x1c)
    pub slots: [i32; 3],
    pub icons: [u32; 3],
    /// jt+0x1304 (last use / refusal time) and the JT clock, for the power-bar flash
    pub t_1304: f32,
    pub t: f32,
    pub time_scale: f32,
}

impl Jt {
    pub fn gadget_hud(&self) -> GadgetHud {
        let gd = &self.gad;
        let ids = gd.slots.map(|s| s.id);
        GadgetHud {
            mp: gd.FUN_001441a0(),
            mp_max: gd.mp_max,
            slots: ids,
            icons: ids.map(|i| if i >= 0 { GADGETS[i as usize].icon } else { 0 }),
            t_1304: gd.t_1304,
            t: self.t,
            time_scale: gd.time_scale,
        }
    }

    /// 0x00141fc8(slot, g): equip (unequips `g` from the other slots; save "apukCur[%d]").
    pub fn FUN_00141fc8(&mut self, slot: usize, g: i32, anim: &mut dyn JtAnim) {
        if slot >= 3 {
            return;
        }
        self.FUN_00141c08(slot, -1, anim);
        self.gad.slots[slot] = GadgetSlot { id: g, state: 0, t: self.t };
        for i in 0..3 {
            if i != slot && self.gad.slots[i].id == g {
                self.FUN_00141c08(i, -1, anim);
                self.gad.slots[i].id = -1;
            }
        }
    }

    /// 0x00141c08(slot, state)
    pub fn FUN_00141c08(&mut self, slot: usize, s: i32, anim: &mut dyn JtAnim) {
        let cur = self.gad.slots[slot];
        if cur.state == s {
            return;
        }
        if cur.state == 1 {
            self.FUN_0018dd78(cur.id);
        }
        self.gad.slots[slot].state = s;
        self.gad.slots[slot].t = self.t;
        if s == 1 {
            self.FUN_0018da48(cur.id, anim);
        } else if s == 2 {
            self.FUN_0018da48(cur.id, anim);
            self.gad.slots[slot].state = 0;
        }
    }

    /// 0x00141a50 (from the JT parent update 0x1403d0): slot buttons.
    pub fn FUN_00141a50(&mut self, anim: &mut dyn JtAnim) {
        for i in 0..3 {
            let GadgetSlot { id, state, .. } = self.gad.slots[i];
            if state == 0 {
                if self.joy.FUN_001c46c0(self.t, 0.15, DAT_002C3258[i]) == 0 {
                    continue;
                }
                if !self.FUN_0018d760(id) {
                    self.gad.sfx.push(SND_REFUSE);
                } else if self.gad.FUN_001441a0() < GADGETS[id as usize].cost {
                    self.gad.sfx.push(SND_REFUSE);
                    self.gad.t_1304 = self.t;
                } else {
                    self.FUN_00141c08(i, GADGETS[id as usize].mode, anim);
                }
            } else if state == 1 {
                if self.joy.held & DAT_002C3258[i] == 0 || !self.FUN_0018d760(id) {
                    self.FUN_00141c08(i, 0, anim);
                } else {
                    self.FUN_00141dd0(id);
                }
            }
        }
    }

    /// JT slot 98 0x0018d760: may gadget `g` be used now.
    pub fn FUN_0018d760(&self, g: i32) -> bool {
        if self.binoc.active() || g < 0 {
            return false;
        }
        if g == 0x1d || g == 0x2a {
            if self.jts == super::p7_damage::JTS_DMG {
                return false;
            }
            let ts = self.gad.time_scale;
            return ts == 1.0 || ts == if g == 0x1d { DAT_002BF3EC } else { DAT_002BF3F0 };
        }
        let pre = match self.jts {
            2 => self.FUN_0019c8d0(g),
            3 => g == 3,
            0 | 1 | 5 | 10 | 0xc => true,
            7 => self.FUN_00194558(g),
            8 => false, // 0x19eef0: only g 3 with the REPLACED scene test
            _ => false, // jts 0x17 carry: REPLACED (no carried objects)
        };
        if !pre {
            return false;
        }
        match g {
            1 | 2 | 0x19 | 0x1e | 0x1f | 0x21 | 0x22 | 0x23 | 0x27 => true,
            0x18 => self.jts == 5,
            0x1b => self.jts == 2,
            0x28 => {
                // +0x1d30 / +0x10ec gates (+0x18d8 == -1 not tracked: REPLACED true)
                self.p13.f_1d30 == 0 && self.p13.f_10ec != 4 && matches!(self.jts, 0 | 1 | 10 | 0xc)
            }
            _ => false, // 3 / 0x17 / 0x1a / 0x25 / 0x26 / 0x29: REPLACED scene / partner tests
        }
    }

    /// 0x0019c8d0(jmp, g)
    pub fn FUN_0019c8d0(&self, g: i32) -> bool {
        let b = self.jtbs;
        let test = b != 6 && !(9..=0x16).contains(&b);
        match g {
            2 | 5 | 0x1b => test,
            0x16 if self.kind >= 0x11 => test,
            _ => false, // 3 (jtbs 0x11 + scene test), 0xc / 0x16 (jmp+0x2c): REPLACED
        }
    }

    /// 0x00194558(attach, g)
    pub fn FUN_00194558(&self, g: i32) -> bool {
        if g == 3 {
            return false; // REPLACED scene test
        }
        match self.attach.kind {
            1 | 8 | 0xb | 0xc => false,
            3 | 4 => matches!(g, 1 | 2 | 0x1f),
            _ => true,
        }
    }

    /// JT slot 99 0x0018da48: use gadget `g`.
    pub fn FUN_0018da48(&mut self, g: i32, anim: &mut dyn JtAnim) {
        self.gad.used.push(g);
        self.calls.push((0x001619a0, 0x3c)); // event 0x3c on 0x2deb30
        let t = self.t;
        match g {
            0x18 => {
                self.gad.FUN_00141cd0(g, t);
                self.set_jts(6, super::JTBS_NIL, anim);
            }
            0x19 => {
                self.gad.FUN_00141cd0(g, t);
                self.set_jts(0x18, 0xc, anim);
            }
            0x1a | 0x25 | 0x26 | 0x29 => self.calls.push((0x0018d6f8, g)), // carried object throw: REPLACED
            0x1b => {
                self.gad.FUN_00141cd0(g, t);
                if self.jts == super::JTS_JUMP {
                    self.set_jtbs(0x13, anim);
                }
            }
            0x1d | 0x2a => {
                self.gad.FUN_00141cd0(g, t);
                let (ts, f, snd) = if g == 0x1d { (DAT_002BF3EC, -1.0, 0x67) } else { (DAT_002BF3F0, 0.7, 0x66) };
                self.gad.time_scale = ts;
                self.gad.f_3e1bec = f;
                self.gad.sfx.push(snd);
                self.gad.loop_snd = Some(snd);
            }
            0x1e => {
                self.gad.FUN_00141cd0(g, t);
                self.FUN_00195a00(0xe, 0, anim);
            }
            0x1f => {
                self.gad.FUN_00141cd0(g, t);
                self.FUN_00195a00(9, 0, anim);
            }
            0x21 => {
                self.gad.FUN_00141cd0(g, t);
                self.set_jts(9, super::JTBS_NIL, anim);
            }
            0x22 | 0x23 => {
                self.gad.f_9bc = g;
                self.FUN_001928f8(0xb, None, 0, anim);
            }
            0x27 => {
                self.gad.FUN_00141cd0(g, t);
                self.set_jts(5, 9, anim);
            }
            0x28 => {
                self.gad.FUN_00141cd0(g, t);
                self.FUN_001928f8(2, None, 0, anim);
            }
            // parent slot 99 0x00141d38
            0xc => {
                self.gad.FUN_00141cd0(g, t);
                self.FUN_00195a00(0x11, 0, anim);
            }
            0xe | 0x15 | 0x17 => {
                self.gad.f_9bc = g;
                self.FUN_001928f8(0xb, None, 0, anim);
            }
            _ => {}
        }
    }

    /// JT slot 100 0x00141dd0 (hold): power drain of 0x1d / 0x2a.
    pub fn FUN_00141dd0(&mut self, g: i32) {
        let dt = self.dt;
        match g {
            0x1d => self.gad.FUN_001441e8(DAT_002BD260, dt),
            0x2a => self.gad.FUN_001441e8(DAT_002BD264, dt),
            _ => {}
        }
    }

    /// JT slot 101 0x0018dd78 (release).
    pub fn FUN_0018dd78(&mut self, g: i32) {
        if g == 0x1d || g == 0x2a {
            self.gad.time_scale = 1.0;
            self.gad.loop_snd = None;
            self.gad.f_3e1bec = 0.0;
        }
    }

    /// 0x00197500 head, R1 (8): paraglider. Returns true when it fired.
    pub fn FUN_00197500_r1(&mut self, cw: &crate::engine::physics::CollisionWorld, anim: &mut dyn JtAnim) -> bool {
        if self.joy.FUN_001c4660(self.t, 0.0, 8) == 0 || !self.is_player || !self.gad.FUN_001daea8(GADGET_PARAGLIDER) {
            return false;
        }
        if self.FUN_00143d20(cw, 0.3) {
            return false;
        }
        let t = self.t;
        self.gad.FUN_00141cd0(GADGET_PARAGLIDER, t);
        self.joy.FUN_001c4738(8);
        self.FUN_0018da48(GADGET_PARAGLIDER, anim);
        true
    }

    /// 0x00197f98 case 0x14 (glide steering): yaw turn and the speed targets jmp+0x1c0 / +0x1c4.
    pub fn glide_steer(&mut self) {
        let mut d = self.joy.angle - self.yaw;
        d = super::p5_attach::FUN_0011cf60(d);
        let (s, c) = d.sin_cos();
        let (s, c) = (s * self.joy.mag, c * self.joy.mag);
        let f41 = (c + 1.0) * 0.5;
        let f43 = 1.0 - f41;
        self.gad_glide = [
            f43 * DAT_002BF850[0] + f41 * DAT_002BF850[1],
            f43 * DAT_002BF850[2] + f41 * DAT_002BF850[3],
        ];
        self.yaw += (f43 * DAT_002BF870[0] + f41 * DAT_002BF870[1]) * s * self.dt;
        self.att.rows = crate::engine::physics::driver::FUN_0011b500([0.0, 0.0, 1.0], self.yaw);
        self.calls.push((0x001384c0, 0x14)); // glider blend weights jmp+0x17c.. (draw side)
    }

    /// 0x00196df8 case 0x14: body-frame velocity approach, |a| <= 8000, no downward push.
    pub fn glide_force(&self, dt: f32) -> V3 {
        use crate::engine::physics::dynamics::FUN_0011d018 as appr;
        let v = self.phys.v;
        let (s, c) = self.yaw.sin_cos();
        let l = [c * v[0] + s * v[1], -s * v[0] + c * v[1], v[2]];
        let n = [
            appr(l[0], self.gad_glide[0], dt, GLIDE_PRM[0], None),
            appr(l[1], 0.0, dt, GLIDE_PRM[1], None),
            appr(l[2], self.gad_glide[1], dt, GLIDE_PRM[2], None),
        ];
        let w = [c * n[0] - s * n[1], s * n[0] + c * n[1], n[2]];
        let mut a = [(w[0] - v[0]) / dt, (w[1] - v[1]) / dt, (w[2] - v[2]) / dt];
        let m = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
        if DAT_002BF7D4 < m {
            a = a.map(|x| x * DAT_002BF7D4 / m);
        }
        if a[2] < 0.0 {
            a[2] = 0.0;
        }
        a
    }
}
