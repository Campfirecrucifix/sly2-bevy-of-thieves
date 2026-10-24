//! WP-P15: binocucom controller jt+0xf00 (R3 look), 1:1 for the JT side:
//! 0x19cbf8 init (+0 jt, +4 state -1, +0x20 mode -1), 0x19cc50 request, 0x19cd38 SetBinocState, 0x19cce8 leave,
//! 0x19d330 update, 0x18b2c0 per-frame hook (from the input step 0x18b7e8, before 0x18a8c8), 0x1443d8 active
//! test (state < 3 unsigned), vt+0x2f4 0x1894f8 (states that play the enter / exit ASEG), vt+0x2bc 0x189530
//! (hide Sly while looking: `hidden`).
//! Layout: +4 state, +8 state time, +0xc enter ASEG (jt+0xf0c, JT field 0xeb), +0x10 exit ASEG (jt+0xf10, field
//! 0xec), +0x14 ASEG handle, +0x18 suspended jt+0x1030 body anim, +0x20 mode (0 own look, 1 looking through
//! object +0x40, 2 object +0x40 without hiding Sly), +0x24.. HUD sub-object (0x19d660 state, 0x19da78 update,
//! 0x19dca0 / 0x19d540 HUD text), +0x3c HUD param, +0x40 / +0x44 request args.
//! RECORDED (`calls`): camera policy push 0x14e328 / pop 0x14e4c8 + 0x14d8c0 (CM 0x2ddf40, policy 7 own / 8
//! object), 0x204660 yaw from the binoc camera (DAT_002de3a0 / DAT_002de504), 0x1fea60 sound bus volume
//! (0.6 while looking, 1.0 after), HUD 0x19d660 / 0x19da78 / 0x19dca0, 0x20f198/0x20f0c0/0x20f1a8/0x20f0f0
//! (0x3dd490 game state, mode 1), object vt+0x29c / 0x2a4 / 0x1f5720 (mode 1), jt vt+0x29c / 0x2a4 (0x3f
//! flags), body anim suspend / resume 0x131a00 + vt+0xc / vt+0x14.
//! REPLACED: 0x131968(handle) = the ASEG played longer than `len` (enter / exit ASEG lengths set by the
//! anim side; 0 -> null handle, as 0x1305b8 with no ASEG); 0x18b2c0 exit mask = 0x4f0 (jt+0x1d40 != 7: the
//! class 0x35 sub-object +0x1d40 is not ported); 0x19cce8 +0x38 (unmapped pointer) = null -> always leaves;
//! joy +0x80 == 1 assumed for the stick clear.
#![allow(non_snake_case)]

use super::{Jt, JtAnim};

pub const CALL_CAM_PUSH: u32 = 0x0014e328;
pub const CALL_CAM_POP: u32 = 0x0014e4c8;
pub const CALL_HUD_STATE: u32 = 0x0019d660;
pub const CALL_SOUND_VOL: u32 = 0x001fea60;
pub const CALL_YAW_CAM: u32 = 0x00204660;
pub const CALL_BODY_SUSPEND: u32 = 0x00131a00;

/// jt+0xf00
#[derive(Clone, Debug)]
pub struct Binoc {
    /// +0x04
    pub state: i32,
    /// +0x08 (DAT_002dded0 at the change)
    pub t_state: f32,
    /// +0x14: Some(time the ASEG started) while the enter / exit ASEG plays
    pub handle: Option<f32>,
    /// REPLACED 0x131968: lengths of the +0xc / +0x10 ASEGs (0 = no ASEG)
    pub len: [f32; 2],
    pub cur_len: f32,
    /// +0x18 suspended body anim
    pub suspended: bool,
    /// +0x20
    pub mode: i32,
    /// +0x40 / +0x44
    pub arg: [u32; 2],
    /// +0x28 HUD state (0x19d660)
    pub hud: i32,
    /// vt+0x2bc 0x189530 (Sly hidden while looking)
    pub hidden: bool,
}

impl Default for Binoc {
    /// 0x0019cbf8
    fn default() -> Self {
        Self { state: -1, t_state: 0.0, handle: None, len: [0.0; 2], cur_len: 0.0, suspended: false, mode: -1, arg: [0; 2], hud: -1, hidden: false }
    }
}

impl Binoc {
    /// 0x001443d8: state 0..2 (unsigned < 3).
    pub fn active(&self) -> bool {
        (self.state as u32) < 3
    }
}

/// ANIM-ALL R34: FUN_0019cd38 mode-1 player vtable calls +0x29c (slot 82, FUN_001442d0) / +0x2a4 (slot 83,
/// FUN_00144338) with mask 0x3f; arg `SLOT_SELF` = the JT (nest 0), `SLOT_PARTNER` = +0x40 (nest 1)
pub const CALL_SLOT_82: u32 = 0x0014_42d0;
pub const CALL_SLOT_83: u32 = 0x0014_4338;
pub const SLOT_SELF: i32 = 0;
pub const SLOT_PARTNER: i32 = 1;

/// ANIM-ALL R34: apply the recorded slot 82 / 83 calls to the class set, in order. `me` = the JT's object
/// (the po, else Sly); the partner is the +0x40 handle (`binoc.arg[0]`, 0 none).
pub fn binoc_slots(set: &mut crate::engine::classes::ClassSet, h: &mut dyn crate::engine::classes::ClassHost, me: Option<usize>, jt: &mut Jt) {
    use crate::engine::classes::k22_seq::GuardOp;
    let partner = jt.binoc.arg[0];
    let mut ops = Vec::new();
    // CRAWL R97: FUN_0019cd38's world counter calls (mode 1): FUN_0020f198 / 0020f1a8 (+0x3d84, the guards'
    // FUN_00209098 patrol freeze) and FUN_0020f0c0 / 0020f0f0 (+0x3d7c pad lock), executed in order (SLY2_NO_R97_3D84)
    if std::env::var_os("SLY2_NO_R97_3D84").is_none() {
        jt.calls.retain(|&(c, _)| {
            match c {
                0x0020f198 => h.world_3d84_add(1),
                0x0020f1a8 => h.world_3d84_add(-1),
                0x0020f0c0 => h.k27_world_20f0c0(),
                0x0020f0f0 => h.k27_world_20f0f0(),
                _ => return true,
            }
            false
        });
    }
    jt.calls.retain(|&(c, a)| {
        if c != CALL_SLOT_82 && c != CALL_SLOT_83 {
            return true;
        }
        let (o, nest) = if a == SLOT_PARTNER { (partner, 1) } else { (me.map_or(0, |o| o as u32 + 1), 0) };
        if o != 0 {
            ops.push(if c == CALL_SLOT_82 { GuardOp::Slot82(o, 0x3f, nest) } else { GuardOp::Slot83(o, 0x3f, nest) });
        }
        false
    });
    if !ops.is_empty() {
        for op in ops {
            set.k27_guard_op(op, h);
        }
    }
}

/// JT field of the enter (+0xc) / exit (+0x10) ASEG.
pub const FIELD_BINOC_ENTER: u32 = 0xf0c;
pub const FIELD_BINOC_EXIT: u32 = 0xf10;

impl Jt {
    /// vt+0x2f4 0x001894f8: jts 0, 1, 5, 10, 0xc, 0x18 play the binoc ASEG.
    fn FUN_001894f8(&self) -> bool {
        matches!(self.jts, 0 | 1 | 5 | 10 | 0xc | 0x18)
    }

    fn binoc_hud(&mut self, s: i32) {
        // 0x19d660(+0x24, s)
        if self.binoc.hud != s {
            self.binoc.hud = s;
            self.calls.push((CALL_HUD_STATE, s));
        }
    }

    /// 0x0019cc50(jt+0xf00, mode, a, b)
    pub fn FUN_0019cc50(&mut self, mode: i32, a: u32, b: u32, anim: &mut dyn JtAnim) {
        if self.binoc.state != -1 {
            if self.binoc.mode == 1 || mode != 1 {
                return;
            }
            self.FUN_0019cd38(2, anim);
            self.FUN_0019cd38(-1, anim);
        }
        self.binoc.mode = mode;
        self.binoc.arg = [a, b];
        self.FUN_0019cd38(0, anim);
    }

    /// 0x0019cce8: leave unless +0x38 holds a state in 8..10 (REPLACED: +0x38 null).
    pub fn FUN_0019cce8(&mut self, anim: &mut dyn JtAnim) {
        self.FUN_0019cd38(2, anim);
    }

    /// 0x0019cd38 SetBinocState.
    pub fn FUN_0019cd38(&mut self, new: i32, anim: &mut dyn JtAnim) {
        let old = self.binoc.state;
        if new == old {
            return;
        }
        let mode = self.binoc.mode;
        match old {
            -1 => {
                if mode == 1 {
                    self.calls.push((0x0020f198, 0));
                    self.calls.push((0x0020f0c0, 0));
                }
                // +0x18 = jt+0x1030 (or +0x1034 +0x30): vt+0xc suspend + 0x131a00(0, anim)
                self.binoc.suspended = true;
                self.calls.push((CALL_BODY_SUSPEND, 0));
            }
            1 => {
                match mode {
                    1 => {
                        self.calls.push((CALL_CAM_POP, 8));
                        // ANIM-ALL R34: +0x40 partner vt+0x2a4 (FUN_00144338)(o, 0x3f, 1), FUN_001f5720(o, 0), then
                        // jt vt+0x2a4(jt, 0x3f, 0) (applied by `binoc_slots`)
                        self.calls.push((CALL_SLOT_83, SLOT_PARTNER));
                        self.calls.push((0x001f5720, 0));
                        self.calls.push((CALL_SLOT_83, SLOT_SELF));
                    }
                    0 | 2 => self.calls.push((CALL_CAM_POP, 7)),
                    _ => {}
                }
                self.calls.push((0x0014d8c0, 0));
                self.calls.push((CALL_SOUND_VOL, 10)); // 1.0 on buses 0 / 1
                if mode != 2 {
                    self.binoc.hidden = false; // vt+0x2bc(jt, 0)
                }
            }
            _ => {}
        }
        self.binoc.state = new;
        self.binoc.t_state = self.t;
        let mut aseg = None;
        match new {
            0 => aseg = Some((FIELD_BINOC_ENTER, self.binoc.len[0])),
            1 => {
                match mode {
                    1 => {
                        self.calls.push((CALL_CAM_PUSH, 8));
                        self.calls.push((CALL_YAW_CAM, 1));
                        // ANIM-ALL R34: +0x40 partner vt+0x29c (FUN_001442d0)(o, 0x3f, 1) before FUN_001f5720(o, 1);
                        // jt vt+0x29c(jt, 0x3f, 0) after the HUD (applied by `binoc_slots`)
                        self.calls.push((CALL_SLOT_82, SLOT_PARTNER));
                        self.calls.push((0x001f5720, 1));
                        self.calls.push((0x0019dca0, 0));
                        self.binoc_hud(0);
                        self.calls.push((CALL_SLOT_82, SLOT_SELF));
                    }
                    0 => {
                        self.calls.push((CALL_CAM_PUSH, 7));
                        self.calls.push((CALL_YAW_CAM, 0));
                        self.binoc_yaw_204660();
                        self.binoc_hud(0);
                    }
                    2 => {
                        self.calls.push((CALL_CAM_PUSH, 7));
                        self.binoc_hud(0);
                    }
                    _ => {}
                }
                // +0x190 = 0; vt+0x11c SetVelocity(0); vt+0x124 SetAngularVelocity(0)
                self.v_target = [0.0; 3];
                self.set_velocity_pub([0.0; 3]);
                self.calls.push((0x0014d8c0, 0));
                self.calls.push((CALL_SOUND_VOL, 6)); // 0.6 on buses 0 / 1
                if mode != 2 {
                    self.binoc.hidden = true; // vt+0x2bc(jt, 1)
                }
            }
            2 => {
                self.binoc_hud(2);
                aseg = Some((FIELD_BINOC_EXIT, self.binoc.len[1]));
            }
            -1 => {
                self.binoc_hud(-1);
                if mode == 1 {
                    self.calls.push((0x0020f1a8, 0));
                    self.calls.push((0x0020f0f0, 0));
                }
                if self.binoc.suspended {
                    // 0x131a00(1.0, +0x18); vt+0x14 release; +0x18 = 0
                    self.binoc.suspended = false;
                    self.calls.push((CALL_BODY_SUSPEND, 1));
                }
                self.binoc.mode = -1;
            }
            _ => {}
        }
        // vt+0x2f4 gate, then 0x1305b8(0, 1.0, aseg, jt, 1, &+0x14)
        let aseg = aseg.filter(|_| self.FUN_001894f8());
        self.binoc.handle = None;
        if let Some((f, len)) = aseg {
            anim.play_aseg(f, self.jts, self.jtbs);
            if len > 0.0 {
                self.binoc.handle = Some(self.t);
                self.binoc.cur_len = len;
            }
        }
    }

    /// 0x131968 on +0x14 (REPLACED: played time >= ASEG length).
    /// WP-P24: FUN_00204660(DAT_002de3a0, jt): Sly faces the binocucom look yaw (mode 0; mode 1 reads the
    /// C2cd498 zoom +0x10 DAT_002de504, unported).
    fn binoc_yaw_204660(&mut self) {
        if let Some(y) = self.cam_yaw {
            let mut att = self.att.clone();
            super::p5_attach::FUN_00204660(self, &mut att, y);
            self.att = att;
        }
    }

    fn binoc_anim_done(&self) -> bool {
        self.binoc.handle.is_none_or(|t0| self.t - t0 >= self.binoc.cur_len)
    }

    /// 0x0019d330 update. Returns true when the joystick was cleared (mode 0, joy +0x80 == 1).
    pub fn FUN_0019d330(&mut self, anim: &mut dyn JtAnim) -> bool {
        loop {
            let s = self.binoc.state;
            match s {
                1 => {
                    self.v_target = [0.0; 3];
                    match self.binoc.mode {
                        0 | 1 => {
                            self.calls.push((CALL_YAW_CAM, self.binoc.mode));
                            if self.binoc.mode == 0 {
                                self.binoc_yaw_204660();
                            }
                            // +0x3c = DAT_002de3b0 / DAT_002de50c; 0x19da78(+0x24) HUD update
                            self.calls.push((0x0019da78, 0));
                            // ANIM-ALL R35: REPLACED: the HUD widgets (+0x2c / +0x34) are unported = null, so
                            // 0x19da78 moves HUD 0 -> 1 at once (read by the sequence action-0x3b wait)
                            if self.binoc.hud == 0 {
                                self.binoc_hud(1);
                            }
                        }
                        _ => {}
                    }
                }
                0 => {
                    self.v_target = [0.0; 3];
                    if self.binoc_anim_done() {
                        self.FUN_0019cd38(1, anim);
                    }
                }
                2 => {
                    self.v_target = [0.0; 3];
                    if self.binoc_anim_done() {
                        self.FUN_0019cd38(-1, anim);
                    }
                }
                _ => {}
            }
            if s == self.binoc.state {
                // mode 0 && joy +0x80 == 1: joy +0x10..+0x2c = 0 (stick)
                if self.binoc.mode == 0 {
                    self.joy.mag = 0.0;
                    self.joy.angle = 0.0;
                    return true;
                }
                return false;
            }
        }
    }

    /// 0x0018b2c0 (from 0x18b7e8 before 0x18a8c8): while active, state 1 own look leaves on a 0x4f0 press within
    /// 0.25 s, then 0x19d330. Returns true when the stick was cleared.
    pub fn FUN_0018b2c0(&mut self, anim: &mut dyn JtAnim) -> bool {
        if !self.binoc.active() {
            return false;
        }
        if self.binoc.state == 1 && self.binoc.mode == 0 && self.joy.FUN_001c46c0(self.t, 0.25, 0x4f0) != 0 {
            self.FUN_0019cce8(anim);
        }
        self.FUN_0019d330(anim)
    }

    /// R3 request sites (0x18a8c8 stand / ledge, 0x19e1e0, 0x190520, 0x1911a8, jts 5):
    /// 0x1c4738(joy, 0x400) then FUN_0019cc50(jt+0xf00, 0, 0, 0).
    pub fn binoc_r3(&mut self, anim: &mut dyn JtAnim) {
        self.joy.FUN_001c4738(super::p5_attach::JOY_ATTACK);
        self.FUN_0019cc50(0, 0, 0, anim);
    }
}

/// Lengths of the JT members +0xf0c / +0xf10 ASEGs (0 when absent).
pub fn binoc_lens(w: &crate::engine::world::World, jt: usize) -> [f32; 2] {
    let m = crate::engine::playerdraw::jt_anim_members(w, jt);
    [FIELD_BINOC_ENTER, FIELD_BINOC_EXIT].map(|f| m.get(&f).and_then(|&o| crate::engine::playerdraw::aseg_of(w, o)).map_or(0.0, |a| a.length))
}
