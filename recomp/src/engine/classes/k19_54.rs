//! WP-K19: class 54 C2d44c8 = Murray, own state machine (continued from classes/k18_54.rs).
//! 1:1: SetState FUN_001b6078 with its sub-state setters FUN_001b6da8 (+0x13d8, state 3), FUN_001b5658
//! (+0x1458, state 5), FUN_001b5d30 (+0x17f4, state 9), FUN_001b86b0 (+0x1810, state 0xb), FUN_001b8d30
//! (+0x1840, state 0xc), FUN_001b5e98 (state-7 target), FUN_001b7ba8 (drop the held object), FUN_001b6cc0
//! (state-3 forces); slot 4 FUN_001b68b8 (messages), slot 14 FUN_001b49e0 (load-post), slot 15 FUN_001b50b0
//! (update: FUN_001b5038 binoc, FUN_001b4aa0 transitions, FUN_001b6ba8 state 3, FUN_001b92f8 state 4,
//! FUN_001b8b70 state 0xb, FUN_001b90b8 state 0xc, FUN_001b82a0 ground part, FUN_001b84e0, FUN_001b8508,
//! FUN_001b85d0), slot 98 FUN_001b98a0 (gadget usable), slot 99 FUN_001b9a08 (gadget use).
//! FUN_001b93e8 (+0x1450 carry sub-state): finished in K21 (classes/k21_54.rs: the attach / throw setup).
//! Hooks (`ClassHost::po_call`, unported 55g hub / JT-component / carry code): FUN_001b5380 (state 5),
//! FUN_001b5820 (6), FUN_001b5990 (9), FUN_001b8988 (0xb), FUN_001b8ec0 (0xc), FUN_001b70a0, FUN_001b7d20,
//! FUN_001b7970 / FUN_001b7ae8 (anim event 0x1eb), FUN_001431d0, FUN_00143d20, FUN_0019a098 / FUN_0019f590,
//! FUN_001f7598 (landing impacts), FUN_00184380, FUN_001daea8, FUN_001f7a68 / FUN_00209ee0, FUN_0012d108,
//! FUN_00133d08, FUN_00130d30, FUN_0020bd88 / FUN_0021f388 / FUN_0020bf80 / FUN_0020c080, and the
//! JT-layout component calls already hooked by Bentley (K18).
#![allow(non_snake_case)]
use super::k16_55::C2cb230;
use super::k18_54::C2d44c8;
use super::k18_55::Cx;
use super::*;
use crate::engine::camera::math::FUN_0011cf60;
use crate::engine::player::FUN_0011e1a0;

fn handle(obj: usize) -> u32 {
    obj as u32 + 1
}

/// DAT_002c09a8: state-3 jump vertical speed
pub const F_2C09A8: f32 = -500.0;
/// DAT_002c09a4: state-3 air control
pub const F_2C09A4: f32 = 2.5;
/// DAT_002c09a0 * DAT_002bf730: state-3 air speed
pub const F_2C09A0: f32 = 0.75;
pub const F_2BF730: f32 = 515.0;
/// DAT_002c09f0: state-0xb face-the-ally tolerance
pub const F_2C09F0: f32 = 0.4;
/// DAT_002c0984 / DAT_002c0988: state-7 target reach factor / angle tolerance
pub const F_2C0984: f32 = 1.3;
pub const F_2C0988: f32 = 1.0;
/// DAT_002c0990 (by +0x1534) / DAT_002c099c: state-6 throw speeds
pub const F_2C0990: [f32; 3] = [1.5, 1.5, 1.5];
pub const F_2C099C: f32 = 4.0;
/// DAT_002c08e4: +0x1acc highlight time, DAT_002c08f0 / DAT_002a2de0 its scales
pub const T_2C08E4: f32 = 5.0;
pub const S_2C08F0: [f32; 3] = [1.25; 3];
pub const S_2A2DE0: [f32; 3] = [1.0; 3];
/// DAT_002c09e0: state-5 drop velocity (object space)
pub const V_2C09E0: [f32; 3] = [-400.0, 0.0, 600.0];
/// DAT_002c0a20 / 24 / 28 and the DAT_002bbb60 / 64 clamp: state-0xb blend polynomial
pub const POLY_2C0A20: [f32; 3] = [0.5, 0.5, 0.0];
pub const CLAMP_2BBB60: [f32; 2] = [0.0, 1.0];

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn neg(a: [f32; 3]) -> [f32; 3] {
    [-a[0], -a[1], -a[2]]
}
/// minus the DAT_002a2dc0 (0, 0, 1) component
fn horiz(a: [f32; 3]) -> [f32; 3] {
    [a[0], a[1], 0.0]
}

impl C2cb230 {
    /// +0x190.. = DAT_002a2dd0 (zero)
    fn zero_190(&mut self) {
        self.m.set_v3(0x190, [0.0; 3]);
        self.m.set_f(0x19c, 0.0);
    }
    /// FUN_001a6fc0(0, self, v): world vector -> object space (rows +0x1b0 / +0x1c0 / +0x1d0)
    fn ee_sub_1a6fc0_in(&self, v: [f32; 3]) -> [f32; 3] {
        let d = |r: [f32; 3]| r[0] * v[0] + r[1] * v[1] + r[2] * v[2];
        [d(self.m.v3(0x1b0)), d(self.m.v3(0x1c0)), d(self.m.v3(0x1d0))]
    }
    /// FUN_001a6fc0(self, 0, v): object vector -> world
    fn ee_sub_1a6fc0_out(&self, v: [f32; 3]) -> [f32; 3] {
        let (a, b, c) = (self.m.v3(0x1b0), self.m.v3(0x1c0), self.m.v3(0x1d0));
        [0, 1, 2].map(|i| a[i] * v[0] + b[i] * v[1] + c[i] * v[2])
    }
    /// FUN_0020bd68(blk)
    fn ee_sub_20bd68(&mut self, blk: u32) {
        self.m.set_u(blk + 0x9c, 0);
        self.m.set_u(blk + 0xa0, 0);
    }
    /// FUN_00130658(+0x1030) + FUN_001f15b8(+0x1034) + FUN_001305b8(start, 1.0, anim, self, flags, +0x1030)
    fn replay_1030(&mut self, obj: usize, start: f32, anim: u32, flags: u32, stop: bool, h: &mut dyn ClassHost) {
        if stop {
            let mut sl = self.m.u(0x1030);
            super::shared::anim::ee_sub_130658(&mut sl, h);
            self.m.set_u(0x1030, sl);
        }
        let mut sl = self.m.u(0x1034);
        h.ee_sub_1f15b8(&mut sl);
        self.m.set_u(0x1034, sl);
        let mut sl = self.m.u(0x1030);
        super::shared::anim::ee_sub_1305b8(start.to_bits(), 1.0, anim, handle(obj), flags, &mut sl, h);
        self.m.set_u(0x1030, sl);
    }
    /// anim attribute 0x1ac (event time) of `anim` -> FUN_00131a90(+0x1030, t + time / rate)
    fn sync_1ac(&mut self, anim: u32, t: f32, h: &mut dyn ClassHost) {
        let f = if anim == 0 { 0.0 } else { h.anim_attr(anim, 0x1ac).map_or(0.0, f32::from_bits) };
        if f != 0.0 {
            let sl = self.m.u(0x1030);
            let rate = h.anim_slot_f(sl, 0x40);
            h.ee_sub_131a90(sl, t + f / rate);
        }
    }
    /// the held object +0x13f0 (+0x180, LO flag bit 60 = still in the world); None: nothing held / gone
    fn held_180(&self, h: &mut dyn ClassHost) -> Option<u32> {
        let c = self.m.u(0x13f0);
        if c == 0 {
            return None;
        }
        let x = h.obj_i32(c, 0x180) as u32;
        (h.obj_flags_28(x) >> 60 & 1 != 0).then_some(x)
    }
}

impl C2d44c8 {
    /// FUN_001b6078(self, s): Murray SetState. Exit of the old state (sub-state setters told -1, components
    /// +0x2c0 / +0x4c0 / +0xdf0 / +0x1910 / +0xf50 told -1), +0x13d0 = s, +0x13d4 = t, enter (anim +0x1030
    /// and state machine +0x1034 by state; 5 / 9 / 0xf / 0x14 hand over).
    pub fn ee_sub_1b6078(&mut self, obj: usize, s: i32, cx: &mut Cx) {
        let o = handle(obj);
        if s == self.b.state {
            return;
        }
        let old = self.b.state;
        match old {
            0 => {
                cx.h.ee_sub_2303a0_off(o, 0x58, 2);
                cx.h.ee_sub_2303a0_off(o, 0x5c, 2);
                self.b.ee_sub_142880(cx.g, cx.rng);
            }
            1 | 10 => {
                let ph = cx.h.anim_slot_phase(self.b.m.u(0x1030));
                self.b.m.set_f(0x1a0, ph);
            }
            2 => {
                let b = &self.b;
                if b.m.i(0x108c) == 0 {
                    let x = match s {
                        0 | 1 | 4 | 5 | 9 | 0xb | 0xc => b.m.u(0x1a50),
                        0xd => match b.att_4c8 {
                            1 => 0,
                            k if !(1..=4).contains(&k) || k < 3 => b.m.u(0x1a50),
                            _ => b.m.u(0x1a54),
                        },
                        _ => 0,
                    };
                    if x != 0 {
                        cx.h.vcall(x, 0x44);
                        cx.h.ee_sub_15c150(x, o);
                        cx.h.vcall(x, 0x4c);
                    }
                }
                cx.h.po_part_s(0x195fb8, obj, 0x2c0, -1);
            }
            3 => self.ee_sub_1b6da8(obj, -1, cx),
            4 => self.ee_sub_1b93e8(obj, -1, cx),
            5 => self.ee_sub_1b5658(obj, -1, cx),
            6 => self.b.m.set_u(0x1530, 0),
            9 => self.ee_sub_1b5d30(obj, -1, cx),
            0xb => self.ee_sub_1b86b0(obj, -1, cx),
            0xc => self.ee_sub_1b8d30(obj, -1, cx),
            0xd => cx.h.po_part_s(0x192bd8, obj, 0x4c0, -1),
            0xe => cx.h.po_part_s(0x1a0b40, obj, 0xdf0, -1),
            0xf => {
                cx.h.vcall(self.b.m.u(0x191c), 0x4c);
                cx.h.po_part_s(0x2206c8, obj, 0x1910, -1);
                cx.h.ee_sub_1f5720(obj, 0);
            }
            0x10..=0x12 => {
                cx.h.ee_sub_2303a0_off(o, 0x58, 2);
                cx.h.ee_sub_2303a0_off(o, 0x5c, 2);
            }
            0x14 => cx.h.po_part_s(0x19eab0, obj, 0xf50, -1),
            _ => {}
        }
        // the sub-state setters may have re-entered SetState: the old state is the one read now
        let old = self.b.state;
        self.b.state = s;
        let t = cx.g.t;
        self.b.m.set_f(0x13d4, t);
        let (mut anim, mut sm, mut flags, mut label, mut mask) = (0u32, 0u32, 1u32, -1i32, 4i32);
        let b = &mut self.b;
        match s {
            0 => {
                cx.h.ee_sub_2303a0_off(o, 0x58, 1);
                cx.h.ee_sub_2303a0_off(o, 0x5c, 1);
                sm = b.m.u(0x1028);
                b.zero_190();
                label = match old {
                    2 => 0x178,
                    10 => 0x176,
                    _ => 0x175,
                };
                if cx.h.ee_sub_1f0400(sm, label) < 0 {
                    label = -1;
                }
                mask = b.ee_sub_1430c8(obj, cx.h);
                b.ee_sub_142868(1);
            }
            1 | 10 => {
                let blk = if s == 1 { 0x1320 } else { 0x1540 };
                anim = b.m.u(blk + 4);
                flags = 9;
                b.ee_sub_20bd68(blk);
            }
            3 => {
                self.ee_sub_1b6da8(obj, 0, cx);
                let b = &self.b;
                anim = b.m.u(0x1ae8);
                if anim == 0 || b.m.i(0x13e8) == 0 {
                    anim = b.m.u(0x13e0);
                }
            }
            4 => {
                anim = match b.m.i(0x144c) {
                    0 => b.m.u(0x1440),
                    1 => b.m.u(0x1444),
                    _ => 0,
                };
                b.zero_190();
                self.ee_sub_1b93e8(obj, 0, cx);
            }
            5 | 9 => {
                if s == 5 {
                    self.ee_sub_1b5658(obj, 0, cx);
                } else {
                    self.ee_sub_1b5d30(obj, 0, cx);
                }
                cx.h.vcall(o, 0x2d4);
                return;
            }
            6 => sm = b.m.u(0x1520),
            7 => {
                anim = if b.m.u(0x1ae0) != 0 && cx.h.po_call(0x1daea8, obj, 0, &[0x11]) != 0 { b.m.u(0x1ae0) } else { b.m.u(0x140c) };
                b.zero_190();
                self.ee_sub_1b5e98(obj, cx.h);
                let d = sub(self.b.m.v3(0x1420), cx.h.ee_sub_1a62e8(o));
                cx.h.ee_sub_2046b0(obj, d);
            }
            8 => {
                anim = b.m.u(0x1ae4);
                if anim == 0 || b.m.i(0x143c) == 0 {
                    anim = b.m.u(0x1438);
                }
                b.zero_190();
            }
            0xe => cx.h.ee_sub_19cd38_po(obj, -1),
            0xf => {
                cx.h.vcall(b.m.u(0x191c), 0x44);
                b.m.set_u(0x1920, cx.g.g_2dd5bc);
                cx.h.po_part_s(0x2206c8, obj, 0x1910, 0);
                cx.h.ee_sub_1f5720(obj, 1);
                cx.h.vcall(o, 0x11c);
                return;
            }
            0x10..=0x12 => {
                if s == 0x10 {
                    sm = b.m.u(0x102c);
                }
                cx.h.ee_sub_2303a0_off(o, 0x58, 1);
                cx.h.ee_sub_2303a0_off(o, 0x5c, 1);
                b.zero_190();
            }
            0x14 => {
                cx.h.po_part_s(0x19eab0, obj, 0xf50, 0);
                return;
            }
            0x15 => {
                anim = b.m.u(0x1aec);
                b.m.set_v3(0x190, [0.0; 3]);
                flags = 9;
            }
            _ => {}
        }
        let b = &mut self.b;
        b.replay_1030(obj, 0.0, anim, flags, true, cx.h);
        // FUN_001f14f8(sm, self, label, flags, mask, +0x1034)
        let mut sl = b.m.u(0x1034);
        cx.h.ee_sub_1f15b8(&mut sl);
        if sm != 0 {
            sl = cx.h.ee_sub_1f03a8_link(sm, o, label, flags, mask);
            if sl != 0 {
                cx.h.vcall(sl, 0xc);
            }
        }
        b.m.set_u(0x1034, sl);
        if anim != 0 || sm != 0 {
            b.sync_1ac(anim, t, cx.h);
        }
        cx.h.vcall(o, 0x2d4);
    }

    /// FUN_001b6da8(self, s): state-3 (jump) sub-state +0x13d8 (-1 contact record freed; 1 velocity = its
    /// horizontal part + DAT_002c09a8 up, +0x131c = t; 2 landing: impacts, record freed, anim to its 0x1a5 pose)
    pub fn ee_sub_1b6da8(&mut self, obj: usize, s: i32, cx: &mut Cx) {
        let o = handle(obj);
        let b = &mut self.b;
        let old = b.m.i(0x13d8);
        if old == s {
            return;
        }
        if old == -1 {
            b.m.set_v3(0x190, [0.0; 3]);
        }
        b.m.set_i(0x13d8, s);
        b.m.set_f(0x13dc, cx.g.t);
        match s {
            -1 => {
                cx.h.ee_sub_1a7e10(b.m.u(0x13e4));
                b.m.set_u(0x13e4, 0);
                self.saved_13e4.clear();
            }
            1 => {
                let v = horiz(cx.h.vcall_10c(obj));
                cx.h.vcall_11c_v(o, [v[0], v[1], F_2C09A8]);
                b.m.set_f(0x131c, cx.g.t);
            }
            2 => {
                b.m.set_v3(0x190, [0.0; 3]);
                if b.m.u(0x13e4) != 0 {
                    // contact list walk: FUN_001f7530 surface sound + FUN_001f7598 impact per contact
                    cx.h.po_call(0x1f7598, obj, 0x13e4, &[]);
                }
                cx.h.ee_sub_1a7e10(b.m.u(0x13e4));
                b.m.set_u(0x13e4, 0);
                self.saved_13e4.clear();
                let sl = b.m.u(0x1030);
                let anim = cx.h.obj_i32(sl, 0x14) as u32;
                match cx.h.anim_attr(anim, 0x1a5) {
                    Some(a) => {
                        cx.h.po_call(0x130d30, obj, 0, &[a, 1f32.to_bits(), sl, 0]);
                    }
                    None => cx.h.ee_sub_130b18(sl),
                }
            }
            _ => {}
        }
    }

    /// FUN_001b93e8(self, s): state-4 (carry) sub-state +0x1450. PARTIAL: 1 / 2 need a held object still in
    /// the world (else FUN_001b7d20 + SetState 0); its attach (1) / throw (2) setup is the hook. +0x1088 +0x1c
    /// = sub-state in 0..2.
    pub fn ee_sub_1b93e8(&mut self, obj: usize, s: i32, cx: &mut Cx) {
        let old = self.b.m.i(0x1450);
        if old == s {
            return;
        }
        if old == 1 {
            // FUN_0012e460(+0x13fc), FUN_0012e460(+0x1400): drop the attach acts
            cx.h.po_call(0x12e460, obj, 0x13fc, &[]);
            cx.h.po_call(0x12e460, obj, 0x1400, &[]);
        }
        self.b.m.set_i(0x1450, s);
        self.b.m.set_f(0x1454, cx.g.t);
        match s {
            1 | 2 => {
                if self.b.held_180(cx.h).is_none() {
                    self.ee_sub_1b7d20(obj, cx.g.t, cx.h);
                    self.b.m.set_u(0x13f0, 0);
                    self.ee_sub_1b6078(obj, 0, cx);
                } else if let Some(x) = self.b.held_180(cx.h) {
                    // K21: the attach (1) / throw (2) setup (super::k21_54)
                    self.k21_1b93e8_setup(obj, s, x, cx);
                }
            }
            0 => self.b.m.set_i(0x1448, 0),
            _ => {}
        }
        let cur = self.b.m.i(0x1450);
        cx.h.obj_set_u32(self.b.m.u(0x1088), 0x1c, (0..2).contains(&cur) as u32);
    }

    /// FUN_001b5658(self, s): state-5 sub-state +0x1458 (0 stand: sm +0x1510; 1 walk +0x1464 / 2 +0x151c
    /// from the saved phase +0x1a0)
    pub fn ee_sub_1b5658(&mut self, obj: usize, s: i32, cx: &mut Cx) {
        let o = handle(obj);
        let b = &mut self.b;
        let old = b.m.i(0x1458);
        if s == old {
            return;
        }
        if old == 0 {
            cx.h.ee_sub_2303a0_off(o, 0x58, 2);
            cx.h.ee_sub_2303a0_off(o, 0x5c, 2);
        } else if (0..=2).contains(&old) {
            let ph = cx.h.anim_slot_phase(b.m.u(0x1030));
            b.m.set_f(0x1a0, ph);
        }
        b.m.set_i(0x1458, s);
        b.m.set_f(0x145c, cx.g.t);
        let (mut anim, mut sm, mut flags, mut start) = (0u32, 0u32, 1u32, 0f32);
        match s {
            1 | 2 => {
                anim = b.m.u(if s == 1 { 0x1464 } else { 0x151c });
                flags = 9;
                start = b.m.f(0x1a0) * cx.h.anim_74(anim);
                if s == 1 {
                    b.ee_sub_20bd68(0x1320);
                }
            }
            0 => {
                cx.h.ee_sub_2303a0_off(o, 0x58, 1);
                cx.h.ee_sub_2303a0_off(o, 0x5c, 1);
                sm = b.m.u(0x1510);
                b.zero_190();
            }
            _ => {}
        }
        b.replay_1030(obj, start, anim, flags, true, cx.h);
        let mut sl = b.m.u(0x1034);
        super::k17_55::ee_sub_1f1448(sm, o, -1, flags, &mut sl, cx.h);
        b.m.set_u(0x1034, sl);
    }

    /// FUN_001b5d30(self, s): state-9 sub-state +0x17f4 (0 sm +0x16e0, 1 anim +0x1634, 2 anim +0x17f0)
    pub fn ee_sub_1b5d30(&mut self, obj: usize, s: i32, cx: &mut Cx) {
        let o = handle(obj);
        let b = &mut self.b;
        let old = b.m.i(0x17f4);
        if s == old {
            return;
        }
        if old == 0 {
            cx.h.ee_sub_2303a0_off(o, 0x58, 2);
            cx.h.ee_sub_2303a0_off(o, 0x5c, 2);
        }
        b.m.set_i(0x17f4, s);
        b.m.set_f(0x17f8, cx.g.t);
        let (mut anim, mut sm, mut flags) = (0u32, 0u32, 1u32);
        match s {
            1 => {
                anim = b.m.u(0x1634);
                b.ee_sub_20bd68(0x1630);
                flags = 9;
            }
            2 => {
                anim = b.m.u(0x17f0);
                flags = 9;
            }
            0 => {
                cx.h.ee_sub_2303a0_off(o, 0x58, 1);
                cx.h.ee_sub_2303a0_off(o, 0x5c, 1);
                sm = b.m.u(0x16e0);
                b.zero_190();
            }
            _ => {}
        }
        b.replay_1030(obj, 0.0, anim, flags, true, cx.h);
        let mut sl = b.m.u(0x1034);
        super::k17_55::ee_sub_1f1448(sm, o, -1, flags, &mut sl, cx.h);
        b.m.set_u(0x1034, sl);
    }

    /// FUN_001b86b0(self, s): state-0xb (ally lift) sub-state +0x1810: anim +0x17fc[s], face the ally
    /// +0x10dc when close to its direction, +0x1830 = where the anim's root ends
    pub fn ee_sub_1b86b0(&mut self, obj: usize, s: i32, cx: &mut Cx) {
        let o = handle(obj);
        let t = cx.g.t;
        let b = &mut self.b;
        if s == b.m.i(0x1810) {
            return;
        }
        b.m.set_i(0x1810, s);
        b.m.set_f(0x1814, t);
        if s == -1 {
            return;
        }
        for off in [0x1820, 0x1818, 0x181c] {
            b.m.set_u(off, 0);
        }
        if b.m.u(0x190c) != 0 {
            let mut sl = b.m.u(0x190c);
            super::shared::anim::ee_sub_130658(&mut sl, cx.h);
            b.m.set_u(0x190c, sl);
        }
        let anim = b.m.u((0x17fc + s * 4) as u32);
        b.zero_190();
        let ally = b.m.u(0x10dc);
        if ally != 0 && 1 < b.m.i(0x10e4) && b.joy.held & 8 == 0 {
            let d = sub(cx.h.ee_sub_1a62e8(ally), cx.h.ee_sub_1a62e8(o));
            let yaw = d[1].atan2(d[0]);
            if FUN_0011cf60(yaw - b.m.f(0x1a8)).abs() < F_2C09F0 {
                b.ee_sub_204660(yaw);
                cx.h.ee_sub_204ca8_po(obj);
            }
        }
        b.replay_1030(obj, 0.0, anim, 1, false, cx.h);
        b.sync_1ac(anim, t, cx.h);
        let sl = b.m.u(0x1030);
        if sl != 0 {
            let a = cx.h.obj_i32(sl, 0x14) as u32;
            let p = cx.h.ee_sub_1a62e8(o);
            let end = match cx.h.root_track_12ff10(a, obj) {
                None => p,
                Some((p0, p1)) => {
                    let d = b.ee_sub_1a6fc0_out(sub(p1, p0));
                    [p[0] + d[0], p[1] + d[1], p[2] + d[2]]
                }
            };
            b.m.set_v3(0x1830, end);
        }
        cx.h.po_call(0x1431d0, obj, 0, &[]);
    }

    /// FUN_001b8d30(self, s): state-0xc (push) sub-state +0x1840 (0 +0x1848, 1 +0x184c, 2 the +0x1850 act
    /// on the +0x1890 record, 3 +0x1904; -1 clears +0x1908)
    pub fn ee_sub_1b8d30(&mut self, obj: usize, s: i32, cx: &mut Cx) {
        let o = handle(obj);
        let b = &mut self.b;
        let old = b.m.i(0x1840);
        if s == old {
            return;
        }
        if old == 2 {
            // FUN_0020c080(+0x1900) + (+0x1900 vtable +0x14): release the push act
            cx.h.po_call(0x20c080, obj, 0x1890, &[]);
            b.m.set_u(0x1900, 0);
        }
        b.m.set_i(0x1840, s);
        b.m.set_f(0x1844, cx.g.t);
        let (anim, flags) = match s {
            0 => {
                b.m.set_v3(0x190, [0.0; 3]);
                (b.m.u(0x1848), 1)
            }
            1 => (b.m.u(0x184c), 9),
            2 => {
                b.m.set_u(0x1908, 0);
                // +0x1900 = self + 0x1890 (kept as its byte offset)
                b.m.set_u(0x1900, 0x1890);
                cx.h.po_call(0x20bf80, obj, 0x1850, &[1, 0x1890]);
                let p = cx.h.ee_sub_1a62e8(o);
                let r = cx.h.ee_sub_1a6318(o)[0];
                let k = -b.m.f(0x186c);
                b.m.set_v3(0x1890 + 0x40, [p[0] + r[0] * k, p[1] + r[1] * k, p[2] + r[2] * k]);
                b.zero_190();
                (0, 0)
            }
            3 => (b.m.u(0x1904), 1),
            -1 => {
                b.m.set_u(0x1908, 0);
                return;
            }
            _ => (0, 0),
        };
        b.replay_1030(obj, 0.0, anim, flags, false, cx.h);
    }

    /// FUN_001b5e98(self): state-7 throw target +0x1420: the nearest DAT_003e13b4 object with +0xe8 bit 16 in
    /// 150 .. 1.3 * +0x1410, height <= 500, within 1 rad of the facing; none: +0x1410 ahead
    pub fn ee_sub_1b5e98(&mut self, obj: usize, h: &mut dyn ClassHost) {
        let o = handle(obj);
        let b = &mut self.b;
        let mut found = false;
        let mut r = b.m.f(0x1410) * F_2C0984;
        let held = b.m.u(0x13f0);
        for x in h.list_3e13b4() {
            if h.obj_i32(x, 0xe8) as u32 & 0x10000 == 0 || h.po_call(0x184380, obj, 0, &[held, x]) != 0 {
                continue;
            }
            let d = sub(h.ee_sub_1a62e8(x), h.ee_sub_1a62e8(o));
            let n = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            if n <= r && 150.0 <= n && d[2] <= 500.0 {
                let a = FUN_0011cf60(d[1].atan2(d[0]) - h.ee_sub_204740(obj));
                if a.abs() <= F_2C0988 {
                    b.m.set_v3(0x1420, h.ee_sub_1a62e8(x));
                    found = true;
                    r = n;
                }
            }
        }
        if !found {
            let p = h.ee_sub_1a62e8(o);
            let v = FUN_0011e1a0(b.m.f(0x1a8), b.m.f(0x1410), 0.0);
            b.m.set_v3(0x1420, [p[0] + v[0], p[1] + v[1], p[2] + v[2]]);
        }
    }

    /// FUN_001b7ba8(self): let go of the held object +0x13f0 (state 6: sm +0x1530 to 0x1ca; carry sub-state 0
    /// of state 4: message 6; else FUN_001b7d20, release it and give it the state's velocity)
    pub fn ee_sub_1b7ba8(&mut self, obj: usize, h: &mut dyn ClassHost) {
        self.drop_held(obj, None, h);
    }
    /// FUN_001b7ba8 body; `t` None: FUN_001b7d20 stays the hook (K19 form), Some: K20 port
    pub(super) fn drop_held(&mut self, obj: usize, t: Option<f32>, h: &mut dyn ClassHost) {
        if self.b.state == 6 {
            if self.b.m.u(0x1530) != 0 {
                h.ee_sub_1f09d8(self.b.m.u(0x1530), 0x1ca);
            }
            return;
        }
        let c = self.b.m.u(0x13f0);
        if c == 0 {
            return;
        }
        if self.b.state == 4 && self.b.m.i(0x1450) == 0 {
            h.vcall_arg(c, 0x1c, 6);
            self.b.m.set_u(0x13f0, 0);
            return;
        }
        match t {
            Some(t) => self.ee_sub_1b7d20(obj, t, h),
            None => {
                h.po_call(0x1b7d20, obj, 0, &[]);
            }
        }
        let b = &mut self.b;
        let x = h.obj_i32(b.m.u(0x13f0), 0x180) as u32;
        if h.ee_sub_132b10(x, 0x38) == 0 {
            h.po_call(0x1f7a68, obj, 0, &[x, 1]);
        } else {
            h.po_call(0x209ee0, obj, 0, &[x]);
        }
        match b.state {
            2 | 4 | 7 | 8 => h.vcall(x, 0x11c),
            5 => h.vcall_11c_v(x, b.ee_sub_1a6fc0_out(V_2C09E0)),
            _ => {}
        }
        b.m.set_u(0x13f0, 0);
    }

    /// slot 87 FUN_001b7e40(self, i), 1:1: FUN_001b7ba8 first (indices 3 / 5..8, and 1 unless +0x2c4 is 0xf /
    /// 0x10), then SetState of jump table 0x2a73f0 (index 0: 5 with +0x13f0 held); i >= 9 does nothing
    pub fn ee_sub_1b7e40_cx(&mut self, obj: usize, i: u32, cx: &mut Cx) {
        let Some(&s) = super::k18_54::STATE_OF_INDEX.get(i as usize) else {
            return;
        };
        let s = if i == 0 && self.b.m.u(0x13f0) != 0 { 5 } else { s };
        let drop = match i {
            1 => !(0xf..0x11).contains(&self.b.m.i(0x2c4)),
            3 | 5 | 6 | 7 | 8 => true,
            _ => false,
        };
        if drop {
            self.ee_sub_1b7ba8_t(obj, cx.g.t, cx.h);
        }
        self.ee_sub_1b6078(obj, s, cx);
    }

    /// FUN_001b6cc0(self): state-3 forces (sub-state 0: cancel gravity; 0 / 1: steer the horizontal velocity
    /// to -(+0x190) at DAT_002c09a4)
    pub fn ee_sub_1b6cc0(&self, obj: usize, h: &mut dyn ClassHost) {
        match self.b.m.i(0x13d8) {
            0 => {
                let g = h.phys_e8_80(obj);
                h.ee_sub_1fac30(obj, neg(g));
            }
            1 => {}
            _ => return,
        }
        let a = horiz(sub(neg(self.b.m.v3(0x190)), h.vcall_10c(obj)));
        h.ee_sub_1fac30(obj, a.map(|x| x * F_2C09A4));
    }

    /// FUN_001b84e0(self)
    pub fn ee_sub_1b84e0(&self) -> bool {
        let m = &self.b.m;
        if m.u(0x190c) == 0 && m.i(0x1810) == -1 {
            return true;
        }
        m.i(0x1820) != 0
    }
    /// ally side for FUN_001b8508 / FUN_001b85d0: `front` if the lift anim (+0x190c +0x14) or the ally's
    /// object-space y says so
    fn lift_side(&self, obj: usize, front: i32, back: i32, h: &mut dyn ClassHost) -> i32 {
        let m = &self.b.m;
        let sl = m.u(0x190c);
        if sl != 0 {
            let a = h.obj_i32(sl, 0x14);
            return if a != m.i(0x17fc) && a != m.i(0x1804) { back } else { front };
        }
        let ally = m.u(0x10dc);
        if ally == 0 {
            return front;
        }
        let d = sub(h.ee_sub_1a62e8(ally), h.ee_sub_1a62e8(handle(obj)));
        if 0.0 < self.b.ee_sub_1a6fc0_in(d)[1] {
            back
        } else {
            front
        }
    }
    /// FUN_001b8508(self): SetState 0xb, lift sub-state 1 / 0
    pub fn ee_sub_1b8508(&mut self, obj: usize, cx: &mut Cx) {
        self.ee_sub_1b6078(obj, 0xb, cx);
        let s = self.lift_side(obj, 1, 0, cx.h);
        self.ee_sub_1b86b0(obj, s, cx);
    }
    /// FUN_001b85d0(self): with both +0x1804 / +0x1808 anims: SetState 0xb, lift sub-state 3 / 2
    pub fn ee_sub_1b85d0(&mut self, obj: usize, cx: &mut Cx) {
        if self.b.m.u(0x1804) == 0 || self.b.m.u(0x1808) == 0 {
            return;
        }
        self.ee_sub_1b6078(obj, 0xb, cx);
        let s = self.lift_side(obj, 3, 2, cx.h);
        self.ee_sub_1b86b0(obj, s, cx);
    }

    /// FUN_001b6ba8(self): state-3 update (anim over: SetState 0; sub-state 1: the landing check
    /// FUN_0019a098 / FUN_0019f590 hooks, a contact record -> sub-state 2); air steering +0x190
    pub fn ee_sub_1b6ba8(&mut self, obj: usize, cx: &mut Cx) {
        let mut sl = self.b.m.u(0x1030);
        let cur = loop {
            let prev = self.b.m.i(0x13d8);
            if sl == 0 || cx.h.ee_sub_131968(sl) {
                self.ee_sub_1b6078(obj, 0, cx);
            } else if self.b.m.i(0x13d8) == 1 {
                if self.b.m.u(0x13e4) == 0 {
                    let t0 = self.b.m.u(0x13dc);
                    if cx.h.po_call(0x19a098, obj, 0x131c, &[t0]) != 0 {
                        cx.h.po_call(0x19f590, obj, 0, &[0]);
                    }
                } else {
                    self.ee_sub_1b6da8(obj, 2, cx);
                }
            }
            let cur = self.b.m.i(0x13d8);
            if cur == prev || cur == -1 {
                break cur;
            }
            sl = self.b.m.u(0x1030);
        };
        if (0..2).contains(&cur) {
            let j = &self.b.joy;
            let v = FUN_0011e1a0(j.angle + std::f32::consts::PI, F_2C09A0 * F_2BF730 * j.mag, 0.0);
            self.b.m.set_v3(0x190, v);
            self.b.m.set_f(0x19c, 0.0);
        }
    }

    /// FUN_001b92f8(self): state-4 update by carry sub-state (1: the held object left the world -> SetState
    /// 0; 2: throw anim over -> SetState 5)
    pub fn ee_sub_1b92f8(&mut self, obj: usize, cx: &mut Cx) {
        let mut cur = self.b.m.i(0x1450);
        loop {
            match cur {
                1 => {
                    if self.b.m.i(0x1448) != 0 {
                        if self.b.m.u(0x13f0) == 0 {
                            self.ee_sub_1b6078(obj, 0, cx);
                        } else if self.b.held_180(cx.h).is_none() {
                            self.ee_sub_1b6078(obj, 0, cx);
                            self.b.m.set_u(0x13f0, 0);
                        }
                    }
                }
                2 => {
                    if cx.h.ee_sub_131968(self.b.m.u(0x1030)) {
                        self.ee_sub_1b6078(obj, 5, cx);
                    }
                }
                _ => {}
            }
            let n = self.b.m.i(0x1450);
            if n == cur || n == -1 {
                return;
            }
            cur = n;
        }
    }

    /// FUN_001b8b70(self): state-0xb update: +0x190 = -root velocity, blended (+0x1818) toward the stick
    /// between the anim attributes 0x240 / 0x242; ally lifted (+0x10e4 4): FUN_002046f0
    pub fn ee_sub_1b8b70(&mut self, obj: usize, cx: &mut Cx) {
        let sl = self.b.m.u(0x1030);
        if sl != 0 {
            let r = cx.h.root_vel_204e38(sl);
            let mut v = neg(r);
            self.b.m.set_v3(0x190, v);
            if self.b.m.i(0x1818) != 0 {
                let a = cx.h.obj_i32(sl, 0x14) as u32;
                let a0 = cx.h.anim_attr(a, 0x240).map_or(0.0, f32::from_bits);
                let a1 = cx.h.anim_attr(a, 0x242).map_or(0.0, f32::from_bits);
                let f = (cx.h.anim_slot_f(sl, 0x1c) - a0) / (a1 - a0);
                let p = POLY_2C0A20[0] + f * (POLY_2C0A20[1] + f * POLY_2C0A20[2]);
                let k = if CLAMP_2BBB60[0] <= p && p <= CLAMP_2BBB60[1] { p } else if CLAMP_2BBB60[0] <= p { CLAMP_2BBB60[1] } else { CLAMP_2BBB60[0] };
                let l = self.b.m.v3(0x1700);
                let n = (l[0] * l[0] + l[1] * l[1] + l[2] * l[2]).sqrt();
                let j = &self.b.joy;
                let w = FUN_0011e1a0(j.angle, j.mag * n, 0.0);
                let loc = self.b.ee_sub_1a6fc0_in(w);
                v = [0, 1, 2].map(|i| v[i] * (1.0 - k) - loc[i] * k);
                self.b.m.set_v3(0x190, v);
            }
        }
        if self.b.m.u(0x10dc) != 0 && self.b.m.i(0x10e4) == 4 {
            cx.h.ee_sub_2046f0(obj);
        }
    }

    /// FUN_001b90b8(self): state-0xc update, sub-states 1 / 3 follow the root velocity
    pub fn ee_sub_1b90b8(&mut self, cx: &mut Cx) {
        let s = self.b.m.i(0x1840);
        if s == 1 || s == 3 {
            let r = cx.h.root_vel_204e38(self.b.m.u(0x1030));
            self.b.m.set_v3(0x190, neg(r));
            self.b.m.set_f(0x180, 1.0);
        }
    }

    /// FUN_001b82a0(self): ground / component tail of the update
    pub fn ee_sub_1b82a0(&self, obj: usize, cx: &mut Cx) {
        let b = &self.b;
        match b.state {
            0 | 1 | 4 | 5 | 7 | 8 | 9 | 10 | 0x12 | 0x15 => {
                if b.f_208 != 0 && b.m.i(0x108c) == 0 {
                    if let Some((x, y)) = cx.h.ground_208(b.f_208 as u32) {
                        cx.h.po_call(0x19feb0, obj, 0x9e0, &[x, y]);
                    }
                }
            }
            0xd => {
                cx.h.po_call(0x1942f0, obj, 0x4c0, &[]);
            }
            0x14 => {
                cx.h.po_call(0x19ea60, obj, 0xf50, &[]);
            }
            _ => {}
        }
    }

    /// FUN_001b5038(self): binocucom (+0xf00) input while it is up
    pub fn ee_sub_1b5038(&mut self, obj: usize, cx: &mut Cx) {
        if !self.b.binoc.active() {
            return;
        }
        let t = cx.g.t;
        if self.b.m.i(0xf04) == 1 && self.b.m.i(0xf20) == 0 && self.b.joy.FUN_001c46c0(t, 0.25, 0x4f0) != 0 {
            cx.h.po_call(0x19cce8, obj, 0xf00, &[]);
        }
        cx.h.po_call(0x19d330, obj, 0xf00, &[]);
    }

    /// FUN_001b4aa0(self): state transitions of the update (stick / buttons, anims over), until stable
    pub fn ee_sub_1b4aa0(&mut self, obj: usize, cx: &mut Cx) {
        let o = handle(obj);
        let mut top = self.b.state;
        loop {
            let st = self.b.state;
            let t = cx.g.t;
            match top {
                2 | 3 | 4 | 6 | 7 | 8 | 0xe..=0x12 | 0x14 => {}
                _ => {
                    if 0.1 < t - self.b.m.f(0x20c) {
                        let k = match st {
                            0xc => (self.b.m.i(0x1840) == 3).then_some(0xb),
                            5 => Some(0x10),
                            0xd => match self.b.att_4c8 {
                                1 | 0xc => None,
                                _ => Some(0),
                            },
                            _ => Some(0),
                        };
                        if let Some(k) = k {
                            cx.h.po_call(0x195a00, obj, 0x2c0, &[k, 0, 0]);
                        }
                    }
                }
            }
            match self.b.state {
                0 | 1 | 10 => 'c: {
                    if self.b.joy.held & 8 == 0 && cx.h.po_call(0x140af8, obj, 0, &[0]) != 0 {
                        self.ee_sub_1b6078(obj, 9, cx);
                        break 'c;
                    }
                    if self.b.joy.FUN_001c46c0(t, 0.1, 0x40) != 0 {
                        cx.h.po_call(0x195a00, obj, 0x2c0, &[1, 0, 0]);
                        break 'c;
                    }
                    if self.b.joy.FUN_001c4660(t, 0.15, 0x400) != 0 && cx.h.vcall_ret_arg(o, 0x31c, 3) != 0 {
                        self.b.joy.FUN_001c4738(0x400);
                        cx.h.po_call(0x19cc50, obj, 0xf00, &[0, 0, 0]);
                        break 'c;
                    }
                    if self.ee_sub_1b84e0() && self.b.joy.FUN_001c46c0(t, 0.15, 0x10) != 0 {
                        self.ee_sub_1b85d0(obj, cx);
                        break 'c;
                    }
                    if self.ee_sub_1b70a0(obj, cx) {
                        break 'c;
                    }
                    if self.b.joy.FUN_001c46c0(t, 0.2, 0x20) != 0 {
                        if cx.h.po_call(0x1938c0, obj, 0x4c0, &[0xf8fa, 0]) == 0 && self.b.m.u(0x13ec) != 0 {
                            self.b.m.set_i(0x144c, 0);
                            self.ee_sub_1b6078(obj, 4, cx);
                        }
                        break 'c;
                    }
                    if self.ee_sub_1b84e0() && self.b.joy.FUN_001c46c0(t, 0.15, 0x80) != 0 {
                        self.ee_sub_1b8508(obj, cx);
                        break 'c;
                    }
                    let thr = if self.b.state == 0 { 0.1 } else { 0.0 };
                    if self.b.joy.mag <= thr {
                        self.ee_sub_1b6078(obj, 0, cx);
                    } else if self.b.joy.held & 8 == 0 {
                        self.ee_sub_1b6078(obj, 10, cx);
                    } else {
                        self.ee_sub_1b6078(obj, 1, cx);
                    }
                }
                2 => {
                    cx.h.po_call(0x197500, obj, 0x2c0, &[]);
                    let k = self.b.m.i(0x2c4);
                    let idle = k == 0xb || k == 0 || (0xf..0x11).contains(&k);
                    if !idle
                        && self.b.joy.FUN_001c4660(t, 0.15, 0x90) != 0
                        && cx.h.vcall_ret_arg(o, 0x31c, 2) != 0
                        && (1..6).contains(&self.b.f_2dc)
                        && cx.h.po_call(0x143d20, obj, 0, &[0.1f32.to_bits()]) == 0
                    {
                        self.b.joy.FUN_001c4738(0x90);
                        self.b.m.set_i(0x13e8, 0);
                        self.ee_sub_1b6078(obj, 3, cx);
                    }
                }
                3 => self.ee_sub_1b6ba8(obj, cx),
                4 => self.ee_sub_1b92f8(obj, cx),
                5 => self.ee_sub_1b5380(obj, cx),
                6 => self.ee_sub_1b5820(cx),
                7 | 8 => {
                    if cx.h.ee_sub_131968(self.b.m.u(0x1030)) {
                        self.ee_sub_1b6078(obj, 0, cx);
                    }
                }
                9 => self.ee_sub_1b5990(obj, cx),
                0xb => self.ee_sub_1b8988(obj, cx),
                0xc => self.ee_sub_1b8ec0(obj, cx),
                0xd => {
                    cx.h.po_call(0x1911a8, obj, 0x4c0, &[]);
                }
                0xe => {
                    cx.h.po_call(0x1a12f0, obj, 0xdf0, &[]);
                }
                0xf => {
                    cx.h.po_call(0x21f418, obj, 0x1910, &[]);
                }
                0x11 => {
                    let sl = self.b.m.u(0x1030);
                    let sm = self.b.m.u(0x1034);
                    if (sl != 0 && cx.h.ee_sub_131968(sl)) || (sm != 0 && cx.h.ee_sub_1f0638(sm)) {
                        self.ee_sub_1b6078(obj, 0, cx);
                    }
                }
                0x14 => {
                    cx.h.po_call(0x19e1e0, obj, 0xf50, &[]);
                }
                0x15 => {
                    if self.b.joy.FUN_001c46c0(t, 0.0, 0x40) != 0 {
                        cx.h.po_call(0x195a00, obj, 0x2c0, &[0xd, 0, 0]);
                    } else if self.b.joy.held & self.b.ee_sub_141ea8(0x10) == 0 {
                        self.ee_sub_1b6078(obj, 0, cx);
                    }
                }
                _ => {}
            }
            if self.b.state == top {
                return;
            }
            top = self.b.state;
        }
    }

    /// slot 15 FUN_001b50b0(self): Murray update
    pub fn ee_sub_1b50b0(&mut self, obj: usize, cx: &mut Cx) {
        let o = handle(obj);
        // FUN_001403d0: class-52 base update (K20, classes/k20_54.rs)
        self.b.ee_sub_1403d0(obj, cx);
        if !cx.h.lo_28_bit60(o) {
            return;
        }
        let sl = self.b.m.u(0x190c);
        if sl != 0 && cx.h.ee_sub_131968(sl) {
            // FUN_001306a8(&+0x190c)
            cx.h.ee_sub_1306a8(sl);
            self.b.m.set_u(0x190c, 0);
        }
        cx.h.po_call(0x194a40, obj, 0, &[0xf8fa]);
        self.ee_sub_1b5038(obj, cx);
        self.ee_sub_1b4aa0(obj, cx);
        let (ang, mag) = (self.b.joy.angle, self.b.joy.mag);
        match self.b.state {
            0 => {
                let mask = self.b.ee_sub_1430c8(obj, cx.h);
                cx.h.ee_sub_1f1350(self.b.m.u(0x1034), mask);
            }
            1 | 10 => {
                let (blk, w) = if self.b.state == 1 { (0x1320, 1.0f32) } else { (0x1540, mag) };
                self.b.ee_sub_204660(ang);
                cx.h.po_call(0x20bb58, obj, blk, &[w.to_bits()]);
                cx.h.po_call(0x20bc08, obj, blk, &[]);
            }
            2 => {
                cx.h.po_call(0x197f98, obj, 0x2c0, &[]);
            }
            0xb => self.ee_sub_1b8b70(obj, cx),
            0xc => self.ee_sub_1b90b8(cx),
            0xd => {
                cx.h.po_call(0x191a10, obj, 0x4c0, &[]);
            }
            0x11 | 0x12 => {
                let sl = self.b.m.u(0x1030);
                let r = cx.h.root_vel_204e38(sl);
                self.b.m.set_v3(0x190, neg(r));
                if self.b.state == 0x12 {
                    cx.h.ee_sub_131a00(mag, sl);
                    self.b.ee_sub_204660(ang);
                }
            }
            _ => {}
        }
        self.ee_sub_1b82a0(obj, cx);
        cx.h.po_call(0x1a1358, obj, 0xdf0, &[]);
        let t = cx.g.t;
        let g = self.b.m.u(0x1430);
        if g != 0 && self.b.m.f(0x1434) + 0.1 < t {
            if cx.h.obj_i32(g, 0xec) == 0 || cx.h.po_call(0x133d08, obj, 0, &[self.b.m.u(0xec)]) == 0 {
                self.b.m.set_u(0x1430, 0);
            }
            if self.b.m.u(0x1430) == 0 {
                cx.h.ee_sub_134320(obj);
            }
        }
        let x = self.b.m.u(0x1acc);
        if x != 0 {
            let s = if t - self.b.m.f(0x1ac8) < T_2C08E4 { S_2C08F0 } else { S_2A2DE0 };
            cx.h.po_call(0x12d108, obj, 0, &[x, s[0].to_bits(), s[1].to_bits(), s[2].to_bits()]);
        }
    }

    /// slot 14 FUN_001b49e0(self): load-post (anim blocks, +0x16f0 table, +0x1850 / +0x1910 acts, the 0xc
    /// highlight xf +0x1acc), SetState 0
    pub fn ee_sub_1b49e0(&mut self, obj: usize, cx: &mut Cx) {
        let o = handle(obj);
        cx.h.base(0x1400b8, obj);
        for blk in [0x1320, 0x1460, 0x1630, 0x1540] {
            cx.h.ee_sub_20b868(obj, blk);
        }
        cx.h.ee_sub_1a0300(obj, 0x16f0, 0x17f0);
        cx.h.po_call(0x20bd88, obj, 0x1850, &[]);
        cx.h.po_call(0x21f388, obj, 0x1910, &[]);
        self.b.ee_sub_1426e8(obj, 0x16, cx.h);
        let oid = cx.h.obj_i32(o, 0x18);
        let c = cx.h.ee_sub_1f4c58(o, oid, 0xc);
        if c != 0 {
            let x = cx.h.ee_sub_12dc18(o, c, S_2A2DE0);
            self.b.m.set_u(0x1acc, x);
        }
        self.ee_sub_1b6078(obj, 0, cx);
    }

    /// slot 4 FUN_001b68b8(self, msg, args): messages (9 anim event of +0x1030 / +0x190c; 10 state machine
    /// +0x1034 label in state 6)
    pub fn ee_sub_1b68b8(&mut self, obj: usize, msg: i32, args: &[u32], cx: &mut Cx) {
        let o = handle(obj);
        cx.h.base(0x140f90, obj);
        let a = |i: usize| args.get(i).copied().unwrap_or(0);
        if msg == 9 {
            if a(0) != self.b.m.u(0x1030) && a(0) != self.b.m.u(0x190c) {
                return;
            }
            match a(2) {
                0x1c8 => {
                    if self.b.m.i(0x1450) == 0 {
                        let p = cx.h.ee_sub_1a62e8(o);
                        cx.h.ee_sub_156b98(obj, p);
                        self.ee_sub_1b93e8(obj, 1, cx);
                    }
                }
                0x1a5 => {
                    let p = cx.h.ee_sub_1a62e8(o);
                    cx.h.ee_sub_156b98(obj, p);
                }
                0x1a4 => self.ee_sub_1b6da8(obj, 1, cx),
                0x1c6 => self.b.m.set_i(0x1448, 1),
                0x1c7 => {
                    if self.b.m.i(0x1450) == 1 {
                        self.ee_sub_1b93e8(obj, 2, cx);
                    }
                }
                0x240 => self.b.m.set_i(0x1818, 1),
                0x241 => self.b.m.set_i(0x181c, 1),
                0x242 => self.b.m.set_i(0x1820, 1),
                0x1eb => match self.b.state {
                    8 => {
                        self.ee_sub_1b7ae8(obj, cx.g.t, cx.h);
                    }
                    7 => {
                        self.ee_sub_1b7970(obj, cx.g.t, cx.h);
                    }
                    0xf => {
                        cx.h.po_call(0x2205a8, obj, 0x1910, &[]);
                    }
                    _ => {}
                },
                _ => {}
            }
        } else if msg == 10 && a(0) == self.b.m.u(0x1034) && self.b.state == 6 {
            let l = a(2);
            if l == a(1) {
                return;
            }
            match l {
                0x1cb..=0x1cf => {
                    let f = F_2C0990.get(self.b.m.i(0x1534) as usize).copied().unwrap_or(F_2C0990[0]);
                    self.b.m.set_f(0x1528, f);
                    self.b.m.set_f(0x1524, f);
                    self.b.m.set_i(0x152c, (F_2C099C * f) as i32);
                }
                0x1ca | 0x1d0 => self.ee_sub_1b6078(obj, 0, cx),
                _ => {}
            }
        }
    }

    /// slot 98 FUN_001b98a0(self, kind): gadget `kind` usable now
    pub fn ee_sub_1b98a0(&self, obj: usize, kind: i32, h: &mut dyn ClassHost) -> bool {
        let b = &self.b;
        if b.binoc.active() {
            return false;
        }
        let ok = match b.state {
            0 | 1 | 9 | 10 => true,
            2 => h.po_call(0x19c8d0, obj, 0x2c0, &[kind as u32]) != 0,
            5 => {
                return match kind {
                    3 => true,
                    0x13 => {
                        let x = h.obj_i32(b.m.u(0x13f0), 0x180) as u32;
                        h.ee_sub_132b10(x, 0x38) != 0
                    }
                    _ => false,
                };
            }
            0xd => h.po_call(0x194558, obj, 0x4c0, &[kind as u32]) != 0,
            0x14 => h.po_call(0x19eef0, obj, 0xf50, &[kind as u32]) != 0,
            _ => false,
        };
        if !ok {
            return false;
        }
        match kind {
            1 | 2 | 0xf | 0x10 | 0x14 | 0x15 => true,
            3 => b.m.i(0x12f0) != 0 && h.g_2dfbc0() & 2 != 0 && !h.ee_sub_14fd50(),
            0x16 => {
                let k = b.m.i(0x2c4);
                k != 0 && k != 0xb && k != -1
            }
            _ => false,
        }
    }

    /// slot 99 FUN_001b9a08(self, kind): use gadget `kind`
    pub fn ee_sub_1b9a08(&mut self, obj: usize, kind: i32, cx: &mut Cx) {
        cx.h.ee_sub_1619a0(0x3c, handle(obj));
        match kind {
            0xf => {
                self.b.m.set_i(0x9bc, 0xf);
                self.b.m.set_u(0x130c, self.b.m.u(0x1ac4));
                cx.h.po_call(0x1928f8, obj, 0x4c0, &[0xb, 0, 0]);
            }
            0x10 => self.ee_sub_1b6078(obj, 0x15, cx),
            0x13 => {
                self.b.ee_sub_141cd0(kind, cx.g);
                self.b.m.set_i(0x143c, 1);
                self.ee_sub_1b6078(obj, 8, cx);
            }
            0x14 => {
                self.b.ee_sub_141cd0(kind, cx.g);
                self.ee_sub_1b6078(obj, 0xc, cx);
                self.ee_sub_1b8d30(obj, 0, cx);
                self.ee_sub_1b8d30(obj, 1, cx);
            }
            0x15 => {
                self.b.m.set_i(0x9bc, 0x15);
                let a = self.b.m.u(0x10dc);
                cx.h.po_call(0x1928f8, obj, 0x4c0, &[0xb, a, 0]);
            }
            0x16 => {
                self.b.ee_sub_141cd0(kind, cx.g);
                self.b.m.set_i(0x13e8, 1);
                self.ee_sub_1b6078(obj, 3, cx);
            }
            _ => self.b.ee_sub_141d38(obj, kind, cx),
        }
    }
}

impl ClassSet {
    /// K19: run a Murray function with its frame environment (None: no Murray at `obj`)
    pub fn k19_murray_with<R>(&mut self, obj: usize, h: &mut dyn ClassHost, f: impl FnOnce(&mut C2d44c8, &mut Cx) -> R) -> Option<R> {
        let c = self.k18.iter_mut().find(|x| x.0 == obj).and_then(|x| if let Inst::K18Murray(c) = &mut x.1 { Some(&mut **c) } else { None })?;
        let mut cx = Cx { g: &self.g, rng: &mut self.rng, h };
        Some(f(c, &mut cx))
    }
}
