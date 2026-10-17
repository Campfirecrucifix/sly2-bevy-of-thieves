//! WP-P13: jts 5 (the stance next to the focus object jt+0x10e8, e.g. a guard Sly can sneak on / pick):
//! entry from 0x0018a8c8 jts 0/1/10/0xc, its own 0x0018a8c8 input case, the cane charged follow-up
//! (msg 9 event 0x165 -> SetJtJts(5, 6 / 7)), the jtbs 8 / 9 ends (msg 3 of the jt+0x1030 ASEG -> (5, 4)),
//! the focus setter FUN_00140678 (jt+0x10dc / +0x10e4 / +0x10e8 / +0x10ec) and its range test FUN_00140af8,
//! the Splice-side flag FUN_0018df50 (jt+0x1d30 -> SetJtJts(0x11, -1)).
//!
//! REPLACED: the jt+0x1030 ASEG messages (msg 9 events / msg 3 done) of the jts 5 jtbs 6..9 bodies are
//! produced here from the JT's own ASEG data (`CaneData::asegs`, keyed by JT member) and the jtbs clock
//! (JMP +0x20), because the port's anim driver does not send messages back to the JT.
//! Focus selection FUN_00140558 (from the JT update 0x1403d0) and its score FUN_001428d8 are 1:1; the per-guard
//! priority FUN_00208e70 is guard code (behaviour SM / vision volumes) and is REPLACED by `guard_focus_prio`.
//! UNPORTED (recorded in `Jt::calls`): FUN_001431d0 (focus change cleanup), vtable +0x31c(jt, 3) (assumed
//! true, as in the ledge hang) / FUN_0019cc50 (binocucom look), jts 0x11 body (scripted state).

#![allow(non_snake_case)]

use super::{Jt, JtAnim, BTN_CIRCLE, BTN_CROSS, JTS_STAND};
use crate::engine::input::pad::BTN_R1;
use crate::engine::physics::geom::V3;

/// DAT_002bcc7c / DAT_002bcc80: FUN_00140af8 horizontal range (class 0x3a object / other).
pub const DAT_002BCC7C: f32 = 1700.0;
pub const DAT_002BCC80: f32 = 1000.0;
/// DAT_002bcc84 / DAT_002bcc88: dz window (low, high); DAT_002bcc8c / DAT_002bcc90: hysteresis widening.
pub const DAT_002BCC84: f32 = -750.0;
pub const DAT_002BCC88: f32 = 750.0;
pub const DAT_002BCC8C: f32 = 500.0;
pub const DAT_002BCC90: f32 = 500.0;

/// Main ASEG members of the jts 5 jtbs (0x0018c250 jtbs switch): 6 -> 0x1d08, 7 -> 0x1d0c, 8 -> 0x1d10,
/// 9 -> 0x1fe8.
pub fn jts5_aseg(jtbs: i32) -> Option<u32> {
    match jtbs {
        6 => Some(0x1d08),
        7 => Some(0x1d0c),
        8 => Some(0x1d10),
        9 => Some(0x1fe8),
        _ => None,
    }
}

/// jt+0x10dc..+0x10ec focus fields (FUN_00140678) and jt+0x1d30.
#[derive(Clone, Debug, PartialEq)]
pub struct JtP13 {
    /// +0x10dc focus target (0 = none; the sneak takedown target)
    pub f_10dc: u32,
    /// +0x10e4 focus kind (-1 after init 0x13ff70)
    pub f_10e4: i32,
    /// +0x10ec focus mode (-1 after init; 4 = stance object)
    pub f_10ec: i32,
    /// +0x10e8 focus object: (FUN_001a62e8 world position, FUN_00132b10(obj, 0x3a))
    pub obj_10e8: Option<(V3, bool)>,
    /// +0x1d30 (FUN_0018df50, written by the Splice method 0x10b598; SetJtJts case 0x11 clears it)
    pub f_1d30: i32,
    /// WP-P25: FUN_001a62e8 world position of the +0x10dc target (read by jts 6: 0x137218 / 0x1374f0 / 0x137248)
    pub pos_10dc: Option<V3>,
}

impl Default for JtP13 {
    fn default() -> Self {
        // 0x0013ff70: +0x10ec = +0x10e4 = -1
        Self { f_10dc: 0, f_10e4: -1, f_10ec: -1, obj_10e8: None, f_1d30: 0, pos_10dc: None }
    }
}

impl Jt {
    /// 0x00140678(jt, kind, target, mode, obj) 1:1 (FUN_001431d0 on a target change is recorded).
    pub fn FUN_00140678(&mut self, kind: i32, target: u32, mode: i32, obj: Option<(V3, bool)>) {
        if target != self.p13.f_10dc {
            self.calls.push((0x001431d0, target as i32));
        }
        self.p13.f_10ec = mode;
        self.p13.f_10dc = target;
        self.p13.f_10e4 = kind;
        self.p13.obj_10e8 = obj;
    }

    /// 0x0018df50(jt, v): jt+0x1d30 = v.
    pub fn FUN_0018df50(&mut self, v: i32) {
        self.p13.f_1d30 = v;
    }

    /// 0x00140af8(jt, wide) 1:1: focus mode 4 (kind != 3) with the object within the horizontal range and
    /// the dz window (both widened when `wide`, the stay test of jts 5).
    pub fn FUN_00140af8(&self, wide: bool) -> bool {
        let p = &self.p13;
        if p.f_10ec != 4 || p.f_10e4 == 3 {
            return false;
        }
        let Some((o, is_3a)) = p.obj_10e8 else { return false };
        let mut r = if is_3a { DAT_002BCC7C } else { DAT_002BCC80 };
        let mut lo = DAT_002BCC84;
        let mut hi = DAT_002BCC88;
        if wide {
            hi = DAT_002BCC88 + DAT_002BCC90;
            r += DAT_002BCC8C;
            lo = DAT_002BCC84 - DAT_002BCC90;
        }
        let d = [o[0] - self.phys.pos[0], o[1] - self.phys.pos[1], o[2] - self.phys.pos[2]];
        let h = (d[0] * d[0] + d[1] * d[1]).sqrt();
        h < r && lo < d[2] && d[2] < hi
    }

    /// 0x0018a8c8 jts 0/1/10/0xc head (before the X test): +0x1d30 -> SetJtJts(0x11, -1); cane recoil
    /// (jtcs 8/9 with the combo window closed) blocks the input. Returns true when the case is done.
    pub fn FUN_0018a8c8_stand_head(&mut self, anim: &mut dyn JtAnim) -> bool {
        if self.p13.f_1d30 != 0 {
            self.set_jts(0x11, -1, anim);
            return true;
        }
        (8..10).contains(&self.cane.jtcs) && !self.cane.f_18e4
    }

    /// 0x0018a8c8 jts 0/1/10/0xc after the X test: R1 (joy +0x30 & 8) not held and FUN_00140af8(jt, 0) ->
    /// SetJtJts(5, 4). Returns true when taken.
    pub fn FUN_0018a8c8_stand_jts5(&mut self, anim: &mut dyn JtAnim) -> bool {
        if self.joy.held & BTN_R1 == 0 && self.FUN_00140af8(false) {
            self.set_jts(5, 4, anim);
            return true;
        }
        false
    }

    /// 0x0018a8c8 case 5 (0x18ad4c..0x18ae7c, 0x18b05c, 0x18b090) 1:1. `m` = joy +0x1c magnitude.
    pub fn FUN_0018a8c8_jts5(&mut self, m: f32, anim: &mut dyn JtAnim) {
        // first switch (0x18a910): walk off after 0.1 s without ground (jtcs 8/9 recoil only with +0x18e4)
        if (!(8..10).contains(&self.cane.jtcs) || self.cane.f_18e4) && 0.1 < self.t - self.t_ground {
            self.FUN_00195a00(0, 0, anim);
            return;
        }
        if self.p13.f_1d30 != 0 {
            self.set_jts(0x11, -1, anim); // 0x18ad40 -> 0x18b060 (a1 0x11, a2 -1)
            return;
        }
        if (6..10).contains(&self.jtbs) {
            return; // charged hold / follow-up bodies run on their ASEG (0x18f0a8, msg 3)
        }
        if self.joy.held & BTN_R1 != 0 || !self.FUN_00140af8(true) {
            self.set_jts(JTS_STAND, -1, anim); // 0x18b05c: SetJtJts(0, -1)
            return;
        }
        if self.joy.FUN_001c46c0(self.t, 0.1, BTN_CROSS) != 0 {
            self.FUN_00195a00(1, 0, anim); // 0x18b090: FUN_00195a00(jt+0x2c0, 1, 0, 0)
            return;
        }
        if self.joy.FUN_001c4660(self.t, 0.15, super::p5_attach::JOY_ATTACK) != 0 && !self.binoc.active() {
            // vt+0x31c(jt, 3) (false while 0x1443d8) -> FUN_001c4738(joy, 0x400); FUN_0019cc50(jt+0xf00, 0, 0, 0)
            self.binoc_r3(anim); // WP-P15
            return;
        }
        if self.joy.FUN_001c46c0(self.t, 0.2, BTN_CIRCLE) != 0 {
            // FUN_001938c0(jt+0x4c0, 0xffff, 0)
            self.calls.push((super::CALL_GROUND_ATTACK, 0xffff));
            self.FUN_001938c0(anim);
            return;
        }
        // 0x18ae40: joy +0x1c > 0 -> (5, 5) else (5, 4)
        let jtbs = if 0.0 < m { 5 } else { 4 };
        self.set_jts(5, jtbs, anim);
    }

    /// FUN_00188f20 msg 9 event 0x165 (needs +0x18ec): jtcs 0xc/0xe/0x10/0x12/0x14 -> SetJtJts(5, 6),
    /// 0xd/0xf/0x11/0x13/0x15 -> SetJtJts(5, 7); other jtcs nothing.
    pub fn FUN_00188f20_ev165(&mut self, anim: &mut dyn JtAnim) {
        if !self.cane.f_18ec {
            return;
        }
        match self.cane.jtcs {
            0xc | 0xe | 0x10 | 0x12 | 0x14 => self.set_jts(5, 6, anim),
            0xd | 0xf | 0x11 | 0x13 | 0x15 => self.set_jts(5, 7, anim),
            _ => {}
        }
    }

    /// REPLACED jt+0x1030 messages for jts 5 jtbs 6..9 (module doc): msg 9 events of the body ASEG in
    /// (t - dt, t] since the jtbs started, then msg 3 at its end: FUN_00188f20 case 5 jtbs 8/9 -> (5, 4).
    pub fn jts5_main_step(&mut self, dt: f32, anim: &mut dyn JtAnim) {
        if self.jts != 5 {
            return;
        }
        let Some(f) = jts5_aseg(self.jtbs) else { return };
        let Some(a) = self.cane.data.asegs.get(&f).cloned() else { return };
        let jtbs = self.jtbs;
        let t1 = self.t - self.t_jtbs;
        let t0 = t1 - dt;
        for (t, k, x) in a.ev {
            if k == 0xb && t0 < t && t <= t1 && self.jts == 5 && self.jtbs == jtbs {
                self.FUN_00188f20_msg9(x, anim);
            }
        }
        if self.jts == 5 && self.jtbs == jtbs && a.len <= t1 && (8..10).contains(&jtbs) {
            self.set_jts(5, 4, anim);
        }
    }
}

/// DAT_002bd280.. (FUN_001428d8 polynomials): current-target bonus on h / dz, stick penalty.
pub const DAT_002BD280: [f32; 3] = [-200.0, -0.2, 0.0];
pub const DAT_002BD290: [f32; 3] = [0.0, -0.2, 0.0];
pub const DAT_002BD2A0: [f32; 3] = [0.5, -0.5, 0.0];
pub const DAT_002BD2B0: [f32; 3] = [-500.0, -0.5, 0.0];
/// DAT_002bbb60 / DAT_002bbb64: clamp of the stick term.
pub const DAT_002BBB60: f32 = 0.0;
pub const DAT_002BBB64: f32 = 1.0;

fn poly(c: [f32; 3], x: f32) -> f32 {
    c[0] + x * (c[1] + x * c[2])
}

/// One focus candidate of FUN_00140558 (REPLACED: the guard list DAT_003e1444 linked by +0xfe0).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FocusCand {
    /// object handle (non-zero)
    pub handle: u32,
    /// FUN_001a62e8 world position
    pub pos: V3,
    /// FUN_00132b10(obj, 0x3a)
    pub is_3a: bool,
    /// FUN_00208e70 priority (0..4)
    pub prio: i32,
}

impl Jt {
    /// 0x001428d8(jt, obj, &jt+0x10dc, &score, &dz) 1:1. `cur` = (jt == *(jt+0x10dc)): the original compares
    /// the JT itself (a0) with the focus target, so from 0x208e70 the bonus never applies. Returns (score, dz).
    pub fn FUN_001428d8(&self, obj: V3, cur: bool) -> (f32, f32) {
        let d = [self.phys.pos[0] - obj[0], self.phys.pos[1] - obj[1], self.phys.pos[2] - obj[2]];
        let h = (d[0] * d[0] + d[1] * d[1]).sqrt();
        let dz = d[2];
        let (mut b, bz) = if cur { (poly(DAT_002BD280, h), poly(DAT_002BD290, dz)) } else { (0.0, 0.0) };
        // jt+0x150 joystick is always present on the player JT
        if 0.0001 < h {
            let dir = super::FUN_0011e1a0(self.joy.angle, self.joy.mag, 0.0);
            let x = (dir[0] * d[0] + dir[1] * d[1] + dir[2] * d[2]) / h;
            let f = poly(DAT_002BD2A0, x);
            let f = if f < DAT_002BBB60 { DAT_002BBB60 } else if DAT_002BBB64 < f { DAT_002BBB64 } else { f };
            b += f * poly(DAT_002BD2B0, h);
        }
        (h + b, dz + bz)
    }

    /// 0x00140558 1:1 (runs only while jt+0x10e0 == 0): over the candidates, the lowest-score one gives
    /// (+0x10e4 kind = its priority, +0x10dc target), the highest priority (ties: lower score) gives
    /// (+0x10ec mode, +0x10e8 object); then vt+0x2fc = FUN_00140678.
    pub fn FUN_00140558(&mut self, cands: &[FocusCand]) {
        let (mut best_p, mut best_s, mut best) = (-1i32, f32::MAX, None::<FocusCand>);
        let (mut near_p, mut near_s, mut near) = (-1i32, f32::MAX, None::<FocusCand>);
        for c in cands {
            let (s, _) = self.FUN_001428d8(c.pos, false);
            if s < near_s {
                (near_p, near_s, near) = (c.prio, s, Some(*c));
            }
            if best_p < c.prio || (c.prio == best_p && s < best_s) {
                (best_p, best_s, best) = (c.prio, s, Some(*c));
            }
        }
        let _ = (near_s, best_s);
        self.FUN_00140678(near_p, near.map_or(0, |c| c.handle), best_p, best.map(|c| (c.pos, c.is_3a)));
        self.p13.pos_10dc = near.map(|c| c.pos); // WP-P25
    }
}

/// REPLACED FUN_00208e70 (guard focus priority): 4 when the guard runs one of the pursuit behaviours whose
/// target the original checks against the JT (bits 0x1d / 5 / 6 of +0x2c0; the player is the only JT), else 0.
/// The vision (3), hearing (2) and pickpocket-zone (1) tests need the guard's +0x364..+0x3f8 volumes (unported).
pub fn guard_focus_prio(bits_2c0: u64) -> i32 {
    if [0x1d, 5, 6].iter().any(|b| bits_2c0 >> b & 1 != 0) {
        4
    } else {
        0
    }
}
