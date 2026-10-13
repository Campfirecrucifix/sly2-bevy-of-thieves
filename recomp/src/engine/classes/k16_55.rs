//! WP-K16: class 55 C2cb230 = Bentley (playable character, parent class 52; size 0x1880). Its vtable is the DB
//! root of the class-52 player-character base: slots 11/18/28/57/58/60/67/71/74/76-79/81-83/92/95/96/100/101/103
//! are shared with C2d2e98 (class 53 JT / Sly, ported in player/), C2cbe68 and C2d44c8; the rest are Bentley's
//! own. Split into call clusters in tools/engineport/class_port.md ("K16 class-55 split"); ported here one
//! cluster at a time, smallest first.
//! Cluster 55a (1-function slots), 1:1: own 5 FUN_00113d80, 69 FUN_00135940 (physics forces by state), 87
//! FUN_00135c20 (state index -> FUN_00135248), 88 FUN_00135d08 (state -> index), 90 FUN_00135e30, 91
//! FUN_00135e38, 93 FUN_00135ff0, 102 FUN_00136cb0; base 74 FUN_00140f50, 76 FUN_00140f70, 92 FUN_00140eb0,
//! 95 FUN_00143220, 101 FUN_00141e28, 103 FUN_00141e30.
//! Hooks (their ports are player-side or in a later cluster): FUN_00135248 Bentley SetState (cluster 55h),
//! FUN_001f9b20 / FUN_00204108 / FUN_001fac30 (ALO / PO forces of a non-JT object), the embedded JT-layout
//! components +0x2c0 FUN_00196df8, +0x4c0 FUN_00194170, +0xdf0 FUN_001a1148, +0xf50 FUN_0019df70 and the +0xdf0
//! TakeDamage FUN_001a19a0 (player::p7_damage runs them on a `Jt`; Bentley has no `Jt`), vtable +0x10c (velocity
//! getter). +0xdf0 damage status (slot 76) runs player::p7_damage::Dmg::FUN_001a23c0 on Bentley's own `Dmg`.
//! Cluster 55b (2-function slots), 1:1: own 54 FUN_00135a48 (contact record, + FUN_0019e9c8), 75 FUN_00135f78
//! (hit veto, + FUN_00194640), 84 FUN_00136cc0 (+ FUN_00143c78), 100 FUN_00141dd0 (+ FUN_001441e8 / 1441a0 /
//! 1441c0 counter drain); base 77 FUN_00144480 (+ FUN_001c3ff0), 79 FUN_00144090 (+ FUN_001c41b8), 96
//! FUN_00141588 (+ FUN_0019e9d8). REPLACED: the +0x12fc counter pointer -> `ctr_12fc` (the counter value).
//! Hooks: FUN_00204250 (PO contact part on a non-JT), FUN_00140ed8 (JT-side immunity chain), FUN_00221268
//! (volume state), vtable +0x34c / +0x1c query 0xd (FUN_00129ec8 message destructor dropped), DAT_002de1a0.
#![allow(non_snake_case)]
use super::guard::slots::GuardHit;
use super::*;
use crate::engine::player::p5_hull::ContactRec;
use crate::engine::player::p7_damage::Dmg;

/// class 55 (Bentley)
pub const PORTED_K16: &[(ClassId, u32)] = &[(55, 0x2cb230)];

/// DAT_002bc990: state-6 drag factor (slot 69)
pub const R_2BC990: f32 = 2.5;
/// DAT_002a2dc0: up axis (slot 95)
pub const UP_2A2DC0: [f32; 3] = [0.0, 0.0, 1.0];
/// s_bentley_002a4ae8
pub const NAME: &str = "bentley";

/// slot 5 FUN_00113d80 (class descriptor getter)
pub fn slot5(class: ClassId) -> Option<u32> {
    match class {
        55 => Some(0x2baf34),
        _ => None,
    }
}

/// slot 87 jump table 0x2a4910 (index -> state passed to FUN_00135248)
pub const STATE_OF_INDEX: [i32; 9] = [0, 2, 3, 4, 9, 10, 11, 12, 13];

fn handle(obj: usize) -> u32 {
    obj as u32 + 1
}

/// slot 95 FUN_00143220 output record
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Lead {
    /// +0
    pub kind: i32,
    /// +4 / +8
    pub r_4: f32,
    pub r_8: f32,
    /// +0x10 position + horizontal velocity * t
    pub pos: [f32; 3],
    /// +0x20 horizontal velocity (vtable +0x10c minus its DAT_002a2dc0 component)
    pub vel: [f32; 3],
}

/// Bentley instance state (the fields the ported clusters use; JT-layout offsets below +0x1320)
#[derive(Clone, Debug, Default)]
pub struct C2cb230 {
    /// +0x13d0 state
    pub state: i32,
    /// +0x16b0
    pub f_16b0: i32,
    /// +0xdf0 damage block (JT layout)
    pub dmg: Dmg,
    /// +0x16c volume object (handle, 0 none)
    pub f_16c: u32,
    /// +0x204 / +0x208
    pub f_204: i32,
    pub f_208: i32,
    /// +0x238 / +0x248
    pub f_238: f32,
    pub f_248: f32,
    /// +0x2dc (JMP +0x1c)
    pub f_2dc: i32,
    /// +0x4c0 attach block +8 kind / +0xc sub kind
    pub att_4c8: i32,
    pub att_4cc: i32,
    /// +0xf50 block: +4 (-1 off), +0xc path (handle), +0x10 distance
    pub f_f54: i32,
    pub f_f5c: u32,
    pub f_f60: f32,
    /// +0x1048 / +0x104c heights
    pub f_1048: f32,
    pub f_104c: f32,
    /// +0x12fc counter pointer (REPLACED: the pointed counter itself, None = null), +0x1300 its maximum
    pub ctr_12fc: Option<i32>,
    pub f_1300: i32,
    /// +0x1308 fractional drain
    pub f_1308: f32,
    /// +0x1310 anim handle, +0x1314
    pub f_1310: u32,
    pub f_1314: i32,
    /// K17: the other object words (classes/k17_55.rs `Mem`)
    pub m: super::k17_55::Mem,
    /// K17: ALO core (+0x10.. +0x148, +0x40 flags, +0x110 per-kind bytes)
    pub alo: super::actor::alo::Alo,
    /// K17: +0xf00 binocucom block (FUN_0019cbf8 layout)
    pub binoc: crate::engine::player::p15_binoc::Binoc,
    /// K18: +0x150 joystick (REPLACED pointer: the JOY by value, fed by `ClassSet::k18_feed_joy`)
    pub joy: crate::engine::player::Joy,
    /// K18: joystick +0x80 control mode (None = no joystick: FUN_001c4080 returns -1) and +0x78 mask
    pub joy_80: Option<i32>,
    pub joy_78: u32,
}

impl C2cb230 {
    /// slot 69 FUN_00135940: physics forces of the current state
    pub fn ee_sub_135940(&self, obj: usize, h: &mut dyn ClassHost) {
        if self.state == 0xd {
            h.ee_sub_1f9b20(obj);
        } else {
            h.ee_sub_204108(obj);
        }
        match self.state {
            2 => h.po_part(0x196df8, obj, 0x2c0),
            3 => h.po_part(0x194170, obj, 0x4c0),
            4 => h.po_part(0x1a1148, obj, 0xdf0),
            6 => {
                if self.f_16b0 != 2 {
                    return;
                }
                let v = h.vcall_10c(obj);
                let k = -R_2BC990;
                h.ee_sub_1fac30(obj, [v[0] * k, v[1] * k, v[2] * k]);
            }
            0xd => h.po_part(0x19df70, obj, 0xf50),
            _ => {}
        }
    }
    /// slot 87 FUN_00135c20: FUN_00135248(self, state of index `i`); i >= 9 does nothing
    pub fn ee_sub_135c20(&self, obj: usize, i: u32, h: &mut dyn ClassHost) {
        if let Some(&s) = STATE_OF_INDEX.get(i as usize) {
            h.ee_sub_135248(obj, s);
        }
    }
    /// slot 88 FUN_00135d08: index of the current state (-1 none)
    pub fn ee_sub_135d08(&self) -> i32 {
        match self.state {
            0 => 0,
            2 => 1,
            3 => 2,
            4 => 3,
            9 => 4,
            10 => 5,
            0xb => 6,
            0xc => 7,
            0xd => 8,
            _ => -1,
        }
    }
    /// slot 90 FUN_00135e30
    pub fn ee_sub_135e30(&self) {}
    /// slot 91 FUN_00135e38
    pub fn ee_sub_135e38(&self) -> i32 {
        0
    }
    /// slot 93 FUN_00135ff0: state 0, 1, 5 or 7
    pub fn ee_sub_135ff0(&self) -> i32 {
        let s = self.state;
        if s == 5 {
            return 1;
        }
        if s < 6 {
            (0..2).contains(&s) as i32
        } else {
            (s == 7) as i32
        }
    }
    /// slot 102 FUN_00136cb0
    pub fn ee_sub_136cb0(&self) -> &'static str {
        NAME
    }
    /// slot 74 FUN_00140f50: FUN_001a19a0(self + 0xdf0, hit)
    pub fn ee_sub_140f50(&self, obj: usize, hit: &GuardHit, h: &mut dyn ClassHost) -> bool {
        h.ee_sub_1a19a0_po(obj, hit)
    }
    /// slot 76 FUN_00140f70: FUN_001a23c0(self + 0xdf0)
    pub fn ee_sub_140f70(&self) -> i32 {
        self.dmg.FUN_001a23c0()
    }
    /// slot 92 FUN_00140eb0: FUN_001619a0(0x2deb30, 0x10, self)
    pub fn ee_sub_140eb0(&self, obj: usize, h: &mut dyn ClassHost) {
        h.ee_sub_1619a0(0x10, handle(obj));
    }
    /// slot 95 FUN_00143220(self, out, t): lead record = position + horizontal velocity * t
    pub fn ee_sub_143220(&self, obj: usize, t: f32, h: &mut dyn ClassHost) -> Lead {
        let v = h.vcall_10c(obj);
        let a = UP_2A2DC0;
        let d = v[0] * a[0] + v[1] * a[1] + v[2] * a[2];
        let vel = [v[0] - a[0] * d, v[1] - a[1] * d, v[2] - a[2] * d];
        let p = h.ee_sub_1a62e8(handle(obj));
        Lead { kind: 0, r_4: 150.0, r_8: 75.0, pos: [p[0] + vel[0] * t, p[1] + vel[1] * t, p[2] + vel[2] * t], vel }
    }
    /// slot 101 FUN_00141e28
    pub fn ee_sub_141e28(&self) {}
    /// slot 103 FUN_00141e30: (vtable +0x1c)(self, 0x1a, 0, 0, 0, 0)
    pub fn ee_sub_141e30(&self, obj: usize, h: &mut dyn ClassHost) {
        h.vcall_1c(obj, 0x1a, 0);
    }
}

/// FUN_00194640(attach block): kind 8 with sub kind 9 / 10, or kind 0xc
pub fn ee_sub_194640(kind: i32, sub: i32) -> bool {
    if kind == 8 {
        (sub.wrapping_sub(9) as u32) < 2
    } else {
        kind == 0xc
    }
}

/// DAT_002bd260 / DAT_002bd264: slot-100 drain rates (message 0x1d / 0x2a)
pub const R_2BD260: f32 = 2.0;
pub const R_2BD264: f32 = 1.0;

impl C2cb230 {
    /// slot 54 FUN_00135a48(self, rec, side): contact record part (skipped when the contacted object's +0x10
    /// bit 48 is set)
    pub fn ee_sub_135a48(&self, obj: usize, rec: &mut ContactRec, side: usize, other_bit48: bool, h: &mut dyn ClassHost) {
        if other_bit48 {
            return;
        }
        match self.state {
            4 => {
                // FUN_001a2100(self + 0xdf0, rec)
                if let Some([a, b]) = self.dmg.FUN_001a2100() {
                    rec.f94 = a;
                    rec.f98 = b;
                }
            }
            // FUN_0019e9c8(self + 0xf50, rec)
            0xd => {
                rec.f98 = 0.0;
                rec.f94 = 0.0;
            }
            _ => h.ee_sub_204250(obj, rec, side),
        }
    }
    /// slot 75 FUN_00135f78(self, kind): hit of `kind` vetoed
    pub fn ee_sub_135f78(&self, obj: usize, kind: i32, h: &mut dyn ClassHost) -> bool {
        if self.state == 3 {
            if ee_sub_194640(self.att_4c8, self.att_4cc) {
                return true;
            }
        } else if self.state == 0xd && kind == 0x4000 {
            return true;
        }
        h.ee_sub_140ed8(obj, kind)
    }
    /// slot 84 FUN_00136cc0
    pub fn ee_sub_136cc0(&self) -> i32 {
        if self.f_204 == 0 && self.state != 0xd {
            return match self.state {
                8 => 5,
                0xe => 0xf,
                _ => self.ee_sub_143c78(),
            };
        }
        0
    }
    /// FUN_00143c78
    pub fn ee_sub_143c78(&self) -> i32 {
        if self.f_204 != 0 {
            return 0;
        }
        if (3..5).contains(&self.att_4c8) {
            return 4;
        }
        if self.f_2dc == -1 {
            return match self.dmg.state {
                4 | 7 | 8 | 0xc => 1,
                -1 | 0 | 6 => 7,
                _ => 0,
            };
        }
        1
    }
    /// slot 100 FUN_00141dd0(self, msg): message 0x1d / 0x2a drains the counter
    pub fn ee_sub_141dd0(&mut self, msg: i32, dt: f32) {
        match msg {
            0x1d => self.ee_sub_1441e8(R_2BD260, dt),
            0x2a => self.ee_sub_1441e8(R_2BD264, dt),
            _ => {}
        }
    }
    /// FUN_001441e8(rate, self): +0x1308 += rate * dt; whole units come off the counter (floor 0)
    pub fn ee_sub_1441e8(&mut self, rate: f32, dt: f32) {
        let f = self.f_1308 + rate * dt;
        let n = f as i32;
        self.f_1308 = f;
        if n != 0 {
            self.f_1308 = f - n as f32;
            let v = self.ee_sub_1441a0() - n;
            self.ee_sub_1441c0(v.max(0));
        }
    }
    /// FUN_001441a0: counter (or +0x1300 when there is none)
    pub fn ee_sub_1441a0(&self) -> i32 {
        self.ctr_12fc.unwrap_or(self.f_1300)
    }
    /// FUN_001441c0: counter = min(v, +0x1300)
    pub fn ee_sub_1441c0(&mut self, v: i32) {
        let m = self.f_1300;
        if let Some(c) = &mut self.ctr_12fc {
            *c = v.min(m);
        }
    }
    /// slot 77 FUN_00144480(self, on)
    pub fn ee_sub_144480(&mut self, obj: usize, on: bool, h: &mut dyn ClassHost) {
        self.ee_sub_1c3ff0(obj, on, h);
        h.vcall(handle(obj), 0x34c);
        if !on {
            super::shared::anim::ee_sub_130658(&mut self.f_1310, h);
            self.f_1314 = -1;
        }
    }
    /// FUN_001c3ff0(self, on): camera request 0x2de2f0 (no +0x16c volume) or the volume's forced state
    pub fn ee_sub_1c3ff0(&self, obj: usize, on: bool, h: &mut dyn ClassHost) {
        if self.f_16c == 0 {
            if on {
                h.ee_sub_14e328(1, 0x2de2f0, handle(obj), 0);
            } else {
                h.ee_sub_14e4c8_k(0xf, 1, 0x2de2f0, handle(obj) as i32, 0);
            }
        } else {
            h.ee_sub_221268(self.f_16c, on as i32);
        }
    }
    /// slot 79 FUN_00144090
    pub fn ee_sub_144090(&self, obj: usize, h: &mut dyn ClassHost) -> bool {
        Self::ee_sub_1c41b8(obj, h) != 0 && self.dmg.FUN_001a23c0() != 2
    }
    /// FUN_001c41b8(obj): (vtable +0x1c)(obj, 0xd, 0, 0, 0, &{0x17, 1}) answer (1 when the message kind changed)
    pub fn ee_sub_1c41b8(obj: usize, h: &mut dyn ClassHost) -> i32 {
        let (k, v) = h.vcall_1c_query(obj, 0xd, (0x17, 1));
        if k == 0x17 {
            v
        } else {
            1
        }
    }
    /// slot 96 FUN_00141588(self, out): aim point
    pub fn ee_sub_141588(&self, obj: usize, g: &ClassGlobals, h: &mut dyn ClassHost) -> [f32; 3] {
        if self.f_f54 != -1 {
            // FUN_0019e9d8 -> FUN_001e6f10(+0xf60, +0xf5c, out, 0)
            return h.path_point_1e6f10(self.f_f5c, self.f_f60);
        }
        let mut p = h.ee_sub_1a62e8(handle(obj));
        if self.f_208 != 0 {
            p[2] = self.f_238 + self.f_1048;
            return p;
        }
        if self.f_2dc != 0x14 && g.g_2de240 == 0 && h.g_2de1a0() == handle(obj) {
            p[2] = self.f_104c;
            return p;
        }
        p[2] += self.f_248 + self.f_1048;
        p
    }
}

impl ClassSet {
    /// K16: construct the class-55 (Bentley) instances
    pub(super) fn build_k16(&mut self, w: &World, h: &mut dyn ClassHost) {
        for (i, o) in w.objects.iter().enumerate() {
            if o.class == 55 {
                // K17: slot 6 FUN_00134648 (owner +0x148 bit 35 from the host, as for C2d9978)
                let mut c: Box<C2cb230> = Box::default();
                c.alo.so.owner = o.owner;
                let owner_bit = o.owner.map(|ow| self.bit35_of(ow).unwrap_or_else(|| h.owner_bit35(ow)));
                c.ee_sub_134648(i, owner_bit, &mut self.g, h);
                self.k16.push((i, Inst::K16(c)));
            }
        }
    }
    /// K17: SetState FUN_00135248 on the class-55 instance at object index `obj` (false: none there)
    pub fn k17_set_state(&mut self, obj: usize, s: i32, h: &mut dyn ClassHost) -> bool {
        let Some(c) = self.k16.iter_mut().find(|x| x.0 == obj).and_then(|x| if let Inst::K16(c) = &mut x.1 { Some(&mut **c) } else { None }) else {
            return false;
        };
        c.ee_sub_135248(obj, s, &self.g, &mut self.rng, h);
        true
    }
    /// K18: slot 87 FUN_00135c20 with the in-set SetState (k17 `ee_sub_135248`); i >= 9 does nothing
    pub fn k18_slot87(&mut self, obj: usize, i: u32, h: &mut dyn ClassHost) -> bool {
        match STATE_OF_INDEX.get(i as usize) {
            Some(&s) => self.k17_set_state(obj, s, h),
            None => self.k16_get(obj).is_some(),
        }
    }
    /// K16 class-55 state at object index `obj`
    pub fn k16_get(&mut self, obj: usize) -> Option<&mut C2cb230> {
        self.k16.iter_mut().find(|x| x.0 == obj).and_then(|x| if let Inst::K16(c) = &mut x.1 { Some(&mut **c) } else { None })
    }
}
