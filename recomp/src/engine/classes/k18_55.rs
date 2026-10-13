//! WP-K18: class 55 C2cb230 = Bentley, rest of cluster 55h (tools/engineport/class_port.md "K16 class-55 split"),
//! continued from classes/k16_55.rs / classes/k17_55.rs.
//! 1:1: own slots 4 FUN_00135ad8 (messages), 13 FUN_00134770 (load tail), 14 FUN_001347a8 (load-post), 15
//! FUN_00134da8 (update: FUN_00134d30 binoc, FUN_00134850 state transitions, FUN_00135eb0 ground part,
//! FUN_00136118 state 6, FUN_001367d8 state 8, FUN_00136338 gadget aim +0x16bc), 97 FUN_00135e40, 98
//! FUN_00136870 (gadget usable), 99 FUN_00136a90 (gadget use), 104 FUN_00136c50; class-52 base slots 11
//! FUN_00141928, 71 FUN_001a28b0 (control flow; the landing probe / push are hooks), 78 FUN_001406e8 (+
//! FUN_001c3ed8 control switch); FUN_00136530 (state-8 sub-state, the Murray mount part is a hook),
//! FUN_00136298, FUN_0018d698, FUN_00136a28, FUN_00141ea8, FUN_00141cd0, FUN_00141d38, FUN_001362f8, the
//! +0x1710 aim block FUN_00136d28 / FUN_00136dc0 / FUN_00136f48, FUN_00131a00, FUN_001eecd0 / FUN_001ef0d8.
//! REPLACED: the joystick pointer +0x150 -> `C2cb230::joy` (player::Joy by value, fed by the player layer
//! through `ClassSet::k18_feed_joy`), its +0x80 control mode -> `C2cb230::joy_80` (-1 = no joystick yet,
//! FUN_001c4080), DAT_003e138c (world +0x3efc focus) -> `ClassSet::k18_active`; record pointers +0x16b4 hold the
//! record's byte offset inside the object (FUN_00136a28 returns `obj + off`).
//! Hooks (unported 55g hub / JT-component code that takes a `Jt` in player/): `ClassHost::po_call` for every
//! embedded JT-layout component call (FUN_00195a00, 197500, 197f98, 191a10, 1911a8, 1a12f0, 1a1358, 1942f0,
//! 19ea60, 19feb0, 19e1e0, 21f418, 19cc50, 1938c0, 19cce8, 19d330, 194558, 19c8d0, 19eef0, 1928f8, 2206c8,
//! 1a0450, 140af8, 194a40, base update FUN_001403d0), and the K18 section of `ClassHost`.
#![allow(non_snake_case)]
use super::k16_55::C2cb230;
use super::*;
use crate::engine::physics::driven::GameRng;

fn handle(obj: usize) -> u32 {
    obj as u32 + 1
}

/// DAT_002bc930: state 0xe duration
pub const T_2BC930: f32 = 2.0;
/// DAT_002bc934: FUN_00204660 argument of state 0 with +0x181c
pub const F_2BC934: f32 = 0.5;
/// DAT_002bc940 / 944 / 948, DAT_002bc950 / 954 / 958: state-6 speed polynomials (c0 + m (c1 + m c2))
pub const POLY_2BC940: [f32; 3] = [-1.0, 2.0, 0.0];
pub const POLY_2BC950: [f32; 3] = [1.0, 0.0, 0.0];
/// DAT_002bbb60 / DAT_002bbb64 clamp
pub const CLAMP_2BBB60: [f32; 2] = [0.0, 1.0];
/// DAT_002bc960 turn rate
pub const F_2BC960: f32 = 5.0;
/// DAT_002bc994 gadget aim hold time
pub const T_2BC994: f32 = 2.0;
/// DAT_002c3258: FUN_00141ea8 masks of the three +0x1180 gadget slots
pub const MASK_2C3258: [u32; 3] = [4, 1, 2];
/// DAT_002bc920: FUN_00136d28 8-byte argument
pub const V_2BC920: [f32; 2] = [175.0, 0.0];
/// DAT_002bcd0c (stride 0x20): gadget cost by kind
pub const COST_2BCD0C: [i32; 24] = [0, 0, 0, 0, 0, 0, 0, 0, 5, 5, 15, 20, 1, 20, 25, 10, 0, 0, 0, 20, 10, 15, 25, 30];

/// One frame's environment of the Bentley functions
pub struct Cx<'a> {
    pub g: &'a ClassGlobals,
    pub rng: &'a mut GameRng,
    pub h: &'a mut dyn ClassHost,
}

impl C2cb230 {
    fn ss(&mut self, obj: usize, s: i32, cx: &mut Cx) {
        self.ee_sub_135248(obj, s, cx.g, cx.rng, cx.h);
    }
    fn po(&self, cx: &mut Cx, addr: u32, obj: usize, off: u32, args: &[u32]) -> i64 {
        cx.h.po_call(addr, obj, off, args)
    }
    /// FUN_00136a28(self, kind): carry record of a gadget kind (byte offset, 0 none)
    pub fn ee_sub_136a28(kind: i32) -> u32 {
        match kind {
            4 => 0x1860,
            5 => 0x1824,
            7 => 0x1830,
            9 => 0x183c,
            0xb => 0x1848,
            0xd => 0x1854,
            _ => 0,
        }
    }
    /// FUN_00141ea8(self, kind): mask of the +0x1180 slots (stride 0xc) holding `kind`
    pub fn ee_sub_141ea8(&self, kind: i32) -> u32 {
        let mut r = 0;
        for i in 0..3 {
            if kind == self.m.i(0x1180 + i * 0xc) {
                r |= MASK_2C3258[i as usize];
            }
        }
        r
    }
    /// FUN_00141cd0(self, kind): pay the gadget cost (counter FUN_001441a0 / FUN_001441c0), +0x1304 = t
    pub fn ee_sub_141cd0(&mut self, kind: i32, g: &ClassGlobals) {
        if kind != -1 {
            let c = self.ee_sub_1441a0();
            let cost = COST_2BCD0C.get(kind as usize).copied().unwrap_or(0);
            self.ee_sub_1441c0(c - cost);
        }
        self.m.set_f(0x1304, g.t);
    }
    /// FUN_00141d38(self, kind): non-carry gadgets (0xc JMP 0x11; 0xe / 0x15 / 0x17 +0x4c0 kind 0xb)
    pub fn ee_sub_141d38(&mut self, obj: usize, kind: i32, cx: &mut Cx) {
        match kind {
            0xc => {
                self.ee_sub_141cd0(0xc, cx.g);
                self.po(cx, 0x195a00, obj, 0x2c0, &[0x11, 0, 0]);
            }
            0xe | 0x15 | 0x17 => {
                self.m.set_i(0x9bc, kind);
                self.po(cx, 0x1928f8, obj, 0x4c0, &[0xb, 0, 0]);
            }
            _ => {}
        }
    }
    /// FUN_0018d698(self, rec, new): carry record `rec` +8 drops its object (FUN_00133448) and takes `new`
    /// (FUN_001333c0)
    pub fn ee_sub_18d698(&mut self, obj: usize, rec: u32, new: u32, h: &mut dyn ClassHost) {
        let o = handle(obj);
        let old = self.m.u(rec + 8);
        if old != 0 {
            h.ee_sub_133448_carry(old, o);
            self.m.set_u(rec + 8, 0);
        }
        if new != 0 {
            self.m.set_u(rec + 8, new);
            h.ee_sub_1333c0_carry(new, o);
        }
    }
    /// FUN_00136298(self): drop the carried object +0x186c of record +0x16b4
    pub fn ee_sub_136298(&mut self, obj: usize, h: &mut dyn ClassHost) {
        let c = self.m.u(0x186c);
        let rec = self.m.u(0x16b4);
        if h.ee_sub_224610(c) {
            h.obj_set_u32(c, 0x16c, handle(obj));
            self.ee_sub_18d698(obj, rec, c, h);
        }
        h.ee_sub_1cb4b8(self.m.u(rec + 4), c);
        self.m.set_u(0x16b4, 0);
        self.m.set_u(0x186c, 0);
    }
    /// FUN_00136530(self, s): state-8 sub-state +0x1800 (0 mounted on Murray DAT_002f7900, 1 free, -1 release)
    pub fn ee_sub_136530(&mut self, obj: usize, s: i32, g: &ClassGlobals, h: &mut dyn ClassHost) {
        let o = handle(obj);
        let cur = self.m.i(0x1800);
        if s == cur {
            return;
        }
        if cur == 0 {
            let v = self.m.u(0x9e4);
            h.ee_sub_1a58d0(obj, 0, 0xf);
            self.m.set_u(0x9e4, v);
            let mut a = self.m.u(0x1810);
            h.ee_sub_12e460(&mut a);
            self.m.set_u(0x1810, a);
            let mut b = self.m.u(0x1814);
            h.ee_sub_12e460(&mut b);
            self.m.set_u(0x1814, b);
            h.ee_sub_2303a0_off(o, 0x58, 2);
            h.ee_sub_2303a0_off(o, 0x5c, 2);
        } else if cur == 1 {
            let yaw = h.ee_sub_204740(obj);
            self.ee_sub_204660(yaw);
            self.m.set_u(0x1a4, 0);
            h.ee_sub_204ca8_po(obj);
        }
        self.m.set_i(0x1800, s);
        self.m.set_f(0x1804, g.t);
        // quirk kept: s == -1 indexes +0x1804 (the time just written)
        let anim = self.m.u((0x1808 + s * 4) as u32);
        if s == 0 {
            // Murray +0x191c mount: vcall +0x104, FUN_0022ff40, vcall +0x54 (attach, +0x9e4 kept), +0x11c / +0x124
            // zero, FUN_0012dc18 x2 -> +0x1810 / +0x1814
            let v = self.m.u(0x9e4);
            let (a, b) = h.ee_sub_136530_mount(obj);
            self.m.set_u(0x9e4, v);
            self.m.set_u(0x1810, a);
            self.m.set_u(0x1814, b);
        } else if s == -1 {
            // Murray state 0xf and +0x1914 != 7: FUN_002206c8(Murray +0x1910, 0)
            h.ee_sub_136530_release();
            return;
        } else if s == 1 {
            self.m.set_i(0x1a4, -1);
        }
        let mut sl = self.m.u(0x1034);
        h.ee_sub_1f15b8(&mut sl);
        self.m.set_u(0x1034, sl);
        let mut a = self.m.u(0x1030);
        super::shared::anim::ee_sub_1305b8(0, 1.0, anim, o, 9, &mut a, h);
        self.m.set_u(0x1030, a);
        h.vcall(o, 0x2d4);
    }
}

// ---- +0x1710 aim block (FUN_00136d28 / FUN_00136dc0 / FUN_00136f48): words [0] owner, [1] +0x1714 state,
// [2] +0x1718 side, [3] +0x171c, [4] +0x1720, [5] +0x1724, [6] +0x1728 t, [7] +0x172c blend, [8..0xb] +0x1730,
// [0xc] / [0xd] +0x1740 / +0x1744 height range, 6 records of 7 words at +0x1748, [0x38] +0x17f0 anim slot,
// [0x39] +0x17f4.
const AIM: u32 = 0x1710;

impl C2cb230 {
    /// FUN_00136d28(lo, hi, blk, owner, v): init. Assumption: the decompile stores only `hi` at [0xc] (the f12
    /// store was lost); the range is taken as [0xc] = lo, [0xd] = hi.
    pub fn ee_sub_136d28(&mut self, obj: usize, lo: f32, hi: f32, v: [f32; 2], h: &mut dyn ClassHost) {
        self.m.set_u(AIM, handle(obj));
        self.m.set_f(AIM + 0x20, v[0]);
        self.m.set_f(AIM + 0x24, v[1]);
        self.m.set_f(AIM + 0x30, lo);
        self.m.set_f(AIM + 0x34, hi);
        self.m.set_i(AIM + 4, -1);
        for i in 0..6u32 {
            h.ee_sub_137a30(obj, 0xc, 2, AIM + 0x38 + i * 0x1c, AIM + 0x50 + i * 0x1c);
        }
    }
    /// FUN_00136f48(blk, s): aim state
    pub fn ee_sub_136f48(&mut self, obj: usize, s: i32, g: &ClassGlobals, h: &mut dyn ClassHost) {
        let o = handle(obj);
        if s == self.m.i(AIM + 4) {
            return;
        }
        if self.m.i(AIM + 4) == -1 {
            self.m.set_i(AIM + 0xc, 0);
            self.m.set_f(AIM + 0x18, g.t);
            self.m.set_i(AIM + 0x10, 0);
            self.m.set_i(AIM + 0x14, 0);
            let reach = if s == 0 { 3.15 } else { 1.5 };
            let hit = h.ee_sub_18f2b0(obj, self.m.f(0x1a8), 0.15, reach, 2, [self.m.f(AIM + 0x20), self.m.f(AIM + 0x24)]);
            match hit {
                None => self.m.set_f(AIM + 0x1c, 0.5),
                Some(p) => {
                    let (lo, hi) = (self.m.f(AIM + 0x30), self.m.f(AIM + 0x34));
                    self.m.set_f(AIM + 0x1c, ((p[2] - lo) / (hi - lo)).clamp(0.0, 1.0));
                }
            }
            let l = h.ee_sub_1a6f00_of(self.m.u(AIM + 0xe4), o);
            self.m.set_i(AIM + 8, (0.0 < l[1]) as i32);
            if s == 0 {
                if let Some(p) = hit {
                    h.ee_sub_2046b0(obj, p);
                    h.ee_sub_204ca8_po(obj);
                }
            }
        }
        let mut fd = 0.0;
        self.m.set_i(AIM + 4, s);
        // s outside -1..2 leaves the anim register uninitialised in the original; 0 here
        let mut anim = 0u32;
        if (0..3).contains(&s) {
            fd = 0.06666667;
            let rec = AIM + 0x38 + ((s * 2 + self.m.i(AIM + 8)) as u32) * 0x1c;
            let b = self.m.f(AIM + 0x1c);
            self.m.set_f(rec, 1.0 - b);
            anim = self.m.u(rec + 0x18);
            self.m.set_f(rec + 0xc, b);
            h.ee_sub_1384c0_rec(obj, rec);
        }
        let mut sl = self.m.u(AIM + 0xe0);
        super::shared::anim::ee_sub_1305b8((g.t - self.m.f(AIM + 0x18)).to_bits(), 1.0, anim, o, 1, &mut sl, h);
        self.m.set_u(AIM + 0xe0, sl);
        if sl != 0 {
            if self.m.i(AIM + 0x10) != 0 {
                h.ee_sub_131b10(sl, o, self.m.u(0x10b8));
            }
            if fd != 0.0 {
                h.ee_sub_131a90(sl, g.t + fd);
            }
        }
    }
    /// FUN_00136dc0(blk, s): aim per frame
    pub fn ee_sub_136dc0(&mut self, obj: usize, s: i32, g: &ClassGlobals, h: &mut dyn ClassHost) {
        let sl = self.m.u(AIM + 0xe0);
        if sl != 0 && h.ee_sub_131968(sl) {
            self.ee_sub_136f48(obj, -1, g, h);
        }
        let cur = self.m.i(AIM + 4);
        if s != cur && cur != -1 {
            self.ee_sub_136f48(obj, s, g, h);
        }
        if s != -1 && (self.m.i(AIM + 4) == -1 || self.m.i(AIM + 0xc) != 0) {
            if self.joy.FUN_001c4660(g.t, 0.1, 0x80) != 0 && h.vcall_ret_arg(handle(obj), 0x31c, 2) != 0 {
                self.joy.FUN_001c4738(0x80);
                self.ee_sub_136f48(obj, -1, g, h);
                self.ee_sub_136f48(obj, s, g, h);
            }
        }
        if self.joy.held & 0x80 == 0 {
            self.m.set_i(AIM + 0x14, 1);
        }
    }
    /// FUN_001362f8(self, s): gadget aim state +0x16bc (+0x16c0 = t; 0 resets the aim block)
    pub fn ee_sub_1362f8(&mut self, obj: usize, s: i32, g: &ClassGlobals, h: &mut dyn ClassHost) {
        if s != self.m.i(0x16bc) {
            self.m.set_i(0x16bc, s);
            self.m.set_f(0x16c0, g.t);
            if s == 0 {
                self.ee_sub_136f48(obj, -1, g, h);
            }
        }
    }
    /// FUN_00136338(self): gadget aim / fire (+0x16bc: -1 idle, 0 aim, 1 binoc hold)
    pub fn ee_sub_136338(&mut self, obj: usize, cx: &mut Cx) {
        let mut cur = self.m.i(0x16bc);
        loop {
            if cur == -1 {
                if self.binoc.active() && self.binoc.mode != 2 {
                    let s = if cx.g.t - self.m.f(0x1700) <= T_2BC994 { 1 } else { 0 };
                    self.ee_sub_1362f8(obj, s, cx.g, cx.h);
                }
                match self.state {
                    0 => {
                        self.ee_sub_136dc0(obj, 0, cx.g, cx.h);
                        if self.m.i(0x181c) != 0 && self.m.i(0x1714) != -1 && self.m.u(0x1034) != 0 && cx.h.ee_sub_1f0400(self.m.u(0x1028), 0x17b) >= 0 {
                            cx.h.ee_sub_1f09d8(self.m.u(0x1034), 0x17b);
                        }
                    }
                    1 | 2 | 3 | 7 => self.ee_sub_136dc0(obj, 1, cx.g, cx.h),
                    5 => self.ee_sub_136dc0(obj, 2, cx.g, cx.h),
                    _ => self.ee_sub_136dc0(obj, -1, cx.g, cx.h),
                }
            } else if cur == 0 || cur == 1 {
                if !self.binoc.active() {
                    self.ee_sub_1362f8(obj, -1, cx.g, cx.h);
                } else if self.m.i(0x16bc) == 1 {
                    if cx.g.t - self.m.f(0x1700) > T_2BC994 {
                        self.ee_sub_1362f8(obj, 0, cx.g, cx.h);
                    }
                } else if self.joy.FUN_001c46c0(cx.g.t, 0.25, 8) != 0 {
                    cx.h.ee_sub_1357c0(obj, 0x80000);
                }
            }
            let now = self.m.i(0x16bc);
            if cur == now {
                return;
            }
            cur = now;
        }
    }
}

// ---- slot 15 update
impl C2cb230 {
    /// FUN_00134d30(self): binocucom part (+0xf00)
    pub fn ee_sub_134d30(&mut self, obj: usize, cx: &mut Cx) {
        if self.binoc.active() {
            if self.binoc.state == 1 && self.binoc.mode == 0 && self.joy.FUN_001c46c0(cx.g.t, 0.25, 0x4f0) != 0 {
                self.po(cx, 0x19cce8, obj, 0xf00, &[]);
            }
            self.po(cx, 0x19d330, obj, 0xf00, &[]);
        }
    }
    /// FUN_00136118(self): state 6 (carry) per frame on +0x16b0
    pub fn ee_sub_136118(&mut self, obj: usize, cx: &mut Cx) {
        let mut cur = self.f_16b0;
        loop {
            'body: {
                if cur == 0 {
                    if cx.h.ee_sub_131968(self.m.u(0x1030)) {
                        self.ee_sub_136040(obj, 1, cx.h);
                    }
                } else if cur > 0 && cur < 3 {
                    let c = self.m.u(0x186c);
                    let k = cx.h.obj_i32(c, 0x158);
                    if k == 2 {
                        cx.h.ee_sub_224258_carry(c);
                    } else {
                        if k != 4 {
                            let kind = self.m.i(self.m.u(0x16b4));
                            let mask = self.ee_sub_141ea8(kind);
                            // FUN_001c4728: held & mask
                            if self.joy.held & (mask | 0x10) != 0 {
                                if cx.g.t - self.m.f(0x20c) <= 0.1 {
                                    self.ee_sub_136040(obj, 1, cx.h);
                                } else {
                                    self.ee_sub_136040(obj, 2, cx.h);
                                }
                                break 'body;
                            }
                        }
                        cx.h.vcall_arg(c, 0x264, 1);
                    }
                    self.ee_sub_136298(obj, cx.h);
                    self.po(cx, 0x195a00, obj, 0x2c0, &[0xb, 0, 0]);
                }
            }
            if self.f_16b0 == cur || self.state != 6 {
                return;
            }
            cur = self.f_16b0;
        }
    }
    /// FUN_001367d8(self): state 8 per frame on +0x1800
    pub fn ee_sub_1367d8(&mut self, obj: usize, cx: &mut Cx) {
        let mut cur = self.m.i(0x1800);
        loop {
            if cur == 1 && self.f_208 != 0 {
                self.ss(obj, 0, cx);
                cx.h.vcall_arg(handle(obj), 0x27c, 1);
            }
            let now = self.m.i(0x1800);
            if now == cur || now == -1 {
                return;
            }
            cur = now;
        }
    }
    /// gadget fire / carry pick-up shared tail (LAB_00134c24)
    fn lab_134c24(&mut self, obj: usize, cx: &mut Cx) {
        self.joy.FUN_001c4738(0x10);
        self.m.set_u(0x16b4, Self::ee_sub_136a28(self.m.i(0x1870)));
        self.ss(obj, 6, cx);
    }
    /// LAB_00134be4: carry button
    fn lab_134be4(&mut self, obj: usize, cx: &mut Cx) {
        if self.joy.FUN_001c4660(cx.g.t, 0.15, 0x10) != 0 && cx.h.vcall_ret_arg(handle(obj), 0x31c, self.m.i(0x1870)) != 0 {
            self.lab_134c24(obj, cx);
        }
    }
    /// FUN_00134850(self): state transitions (re-run while the state changes)
    pub fn ee_sub_134850(&mut self, obj: usize, cx: &mut Cx) {
        let o = handle(obj);
        let mut cur = self.state;
        loop {
            let since = cx.g.t - self.m.f(0x20c);
            match cur {
                3 => {
                    if !(self.att_4c8 == 1 || self.att_4c8 == 0xc || since <= 0.1) {
                        self.po(cx, 0x195a00, obj, 0x2c0, &[0, 0, 0]);
                    }
                }
                2 | 4 | 6 | 8 | 9 | 10 | 0xb | 0xd | 0xe => {}
                _ => {
                    if 0.1 < since {
                        self.po(cx, 0x195a00, obj, 0x2c0, &[0, 0, 0]);
                    }
                }
            }
            let t = cx.g.t;
            match self.state {
                0 | 1 | 7 => 'c: {
                    if self.joy.held & 8 == 0 && self.po(cx, 0x140af8, obj, 0, &[0]) != 0 {
                        self.ss(obj, 5, cx);
                        break 'c;
                    }
                    if self.joy.FUN_001c46c0(t, 0.1, 0x40) != 0 {
                        self.po(cx, 0x195a00, obj, 0x2c0, &[1, 0, 0]);
                        break 'c;
                    }
                    if self.joy.FUN_001c4660(t, 0.15, 0x400) != 0 && cx.h.vcall_ret_arg(o, 0x31c, 3) != 0 {
                        self.joy.FUN_001c4738(0x400);
                        self.po(cx, 0x19cc50, obj, 0xf00, &[0, 0, 0]);
                        break 'c;
                    }
                    if self.joy.FUN_001c4660(t, 0.15, 0x10) != 0 && cx.h.vcall_ret_arg(o, 0x31c, self.m.i(0x1870)) != 0 {
                        self.lab_134c24(obj, cx);
                        break 'c;
                    }
                    if self.joy.FUN_001c46c0(t, 0.2, 0x20) != 0 {
                        self.po(cx, 0x1938c0, obj, 0x4c0, &[0xf8fe, 0]);
                        break 'c;
                    }
                    let thr = if self.state == 0 { 0.1 } else { 0.0 };
                    if self.joy.mag <= thr {
                        self.ss(obj, 0, cx);
                    } else if self.joy.held & 8 == 0 {
                        self.ss(obj, 7, cx);
                    } else {
                        self.ss(obj, 1, cx);
                    }
                }
                2 => {
                    self.po(cx, 0x197500, obj, 0x2c0, &[]);
                    self.lab_134be4(obj, cx);
                }
                3 => {
                    self.po(cx, 0x1911a8, obj, 0x4c0, &[]);
                }
                4 => {
                    self.po(cx, 0x1a12f0, obj, 0xdf0, &[]);
                }
                5 => 'c: {
                    if self.joy.held & 8 == 0 && self.po(cx, 0x140af8, obj, 0, &[1]) != 0 {
                        if self.joy.FUN_001c46c0(t, 0.1, 0x40) != 0 {
                            self.po(cx, 0x195a00, obj, 0x2c0, &[1, 0, 0]);
                            break 'c;
                        }
                        if self.joy.FUN_001c4660(t, 0.15, 0x400) != 0 && cx.h.vcall_ret_arg(o, 0x31c, 3) != 0 {
                            self.joy.FUN_001c4738(0x400);
                            self.po(cx, 0x19cc50, obj, 0xf00, &[0, 0, 0]);
                            break 'c;
                        }
                        if self.joy.FUN_001c46c0(t, 0.2, 0x20) != 0 {
                            self.po(cx, 0x1938c0, obj, 0x4c0, &[0xf8fe, 0]);
                            break 'c;
                        }
                        self.lab_134be4(obj, cx);
                    } else {
                        self.ss(obj, 0, cx);
                    }
                }
                6 => self.ee_sub_136118(obj, cx),
                8 => self.ee_sub_1367d8(obj, cx),
                10 => {
                    let a = self.m.u(0x1030);
                    let done = if a != 0 && cx.h.ee_sub_131968(a) {
                        true
                    } else {
                        let sm = self.m.u(0x1034);
                        sm != 0 && cx.h.ee_sub_1f0638(sm)
                    };
                    if done {
                        self.ss(obj, 0, cx);
                    }
                }
                0xc => {
                    self.po(cx, 0x21f418, obj, 0x11b0, &[]);
                }
                0xd => {
                    self.po(cx, 0x19e1e0, obj, 0xf50, &[]);
                }
                0xe => {
                    if T_2BC930 < t - self.m.f(0x13d4) {
                        self.ss(obj, 0, cx);
                    }
                }
                _ => {}
            }
            self.ee_sub_136338(obj, cx);
            if self.state == cur {
                return;
            }
            cur = self.state;
        }
    }
    /// FUN_00135eb0(self): ground / component tail of the update
    pub fn ee_sub_135eb0(&mut self, obj: usize, cx: &mut Cx) {
        match self.state {
            0 | 1 | 5 | 6 | 7 | 0xb | 0xc | 0xe => {
                if self.f_208 != 0 && self.m.i(0x108c) == 0 {
                    // ground +0x208: +0x88 > 0.7 and owner +0x148 bit 29 clear -> FUN_0019feb0(+0x9e0, +0x40, +0x60)
                    if let Some((a, b)) = cx.h.ground_208(self.f_208 as u32) {
                        self.po(cx, 0x19feb0, obj, 0x9e0, &[a, b]);
                    }
                }
            }
            3 => {
                self.po(cx, 0x1942f0, obj, 0x4c0, &[]);
            }
            0xd => {
                self.po(cx, 0x19ea60, obj, 0xf50, &[]);
            }
            _ => {}
        }
    }
    /// LAB_00135108: +0x190 = rows (+0x1b0, +0x1c0, +0x1d0) . v, +0x1038 = -1, FUN_001a0450
    fn lab_135108(&mut self, obj: usize, v: [f32; 3], cx: &mut Cx) {
        let d = |r: [f32; 3]| r[0] * v[0] + r[1] * v[1] + r[2] * v[2];
        let out = [d(self.m.v3(0x1b0)), d(self.m.v3(0x1c0)), d(self.m.v3(0x1d0))];
        self.m.set_i(0x1038, -1);
        self.m.set_v3(0x190, out);
        self.po(cx, 0x1a0450, obj, 0, &[]);
    }
    /// slot 15 FUN_00134da8(self): Bentley update
    pub fn ee_sub_134da8(&mut self, obj: usize, cx: &mut Cx) {
        let o = handle(obj);
        // FUN_001403d0: class-52 base update (K20, classes/k20_54.rs)
        self.ee_sub_1403d0(obj, cx);
        if !cx.h.lo_28_bit60(o) {
            return;
        }
        self.po(cx, 0x194a40, obj, 0, &[]);
        self.ee_sub_134d30(obj, cx);
        self.ee_sub_134850(obj, cx);
        let (ang, mag) = (self.joy.angle, self.joy.mag);
        match self.state {
            0 => {
                if self.m.i(0x181c) == 0 {
                    let mask = self.ee_sub_1430c8(obj, cx.h);
                    cx.h.ee_sub_1f1350(self.m.u(0x1034), mask);
                } else {
                    self.ee_sub_204660(F_2BC934);
                }
            }
            1 | 7 => {
                let blk = if self.state == 1 { 0x1320 } else { 0x1600 };
                self.ee_sub_204660(ang);
                let w = if self.state == 1 { 1.0 } else { mag };
                self.po(cx, 0x20bb58, obj, blk, &[w.to_bits()]);
                self.po(cx, 0x20bc08, obj, blk, &[]);
            }
            2 => {
                self.po(cx, 0x197f98, obj, 0x2c0, &[]);
            }
            3 => {
                self.po(cx, 0x191a10, obj, 0x4c0, &[]);
            }
            5 => {
                cx.h.ee_sub_2046f0(obj);
                let l = self.m.v3(0x1410);
                let n = (l[0] * l[0] + l[1] * l[1] + l[2] * l[2]).sqrt();
                let rate = cx.h.anim_slot_f(self.m.u(0x1030), 0x40);
                let v = crate::engine::player::FUN_0011e1a0(ang + std::f32::consts::PI, mag * n * rate, 0.0);
                self.m.set_v3(0x190, v);
                self.lab_135108(obj, v, cx);
            }
            6 => {
                if self.f_16b0 == 1 {
                    let p = |c: [f32; 3]| c[0] + mag * (c[1] + mag * c[2]);
                    let clamp = |x: f32| if CLAMP_2BBB60[0] <= x && x <= CLAMP_2BBB60[1] { x } else if CLAMP_2BBB60[0] <= x { CLAMP_2BBB60[1] } else { CLAMP_2BBB60[0] };
                    let f6 = clamp(p(POLY_2BC940));
                    let f4 = clamp(p(POLY_2BC950));
                    if 0.0 < mag {
                        let d = crate::engine::camera::math::FUN_0011cf60(ang - self.m.f(0x1a8));
                        let d = crate::engine::anim::motion::ee_sub_11cfe0(d, f4 * F_2BC960 * cx.g.dt);
                        self.ee_sub_204660(self.m.f(0x1a8) + d);
                    }
                    let l = self.m.v3(0x1520);
                    let n = (l[0] * l[0] + l[1] * l[1] + l[2] * l[2]).sqrt();
                    let rate = cx.h.anim_slot_f(self.m.u(0x1030), 0x40);
                    let v = crate::engine::player::FUN_0011e1a0(ang + std::f32::consts::PI, f6 * n * rate, 0.0);
                    self.m.set_v3(0x190, v);
                    self.lab_135108(obj, v, cx);
                }
            }
            10 | 0xb | 0xe => {
                let r = cx.h.root_vel_204e38(self.m.u(0x1030));
                self.m.set_v3(0x190, [-r[0], -r[1], -r[2]]);
                if self.state == 0xb {
                    // FUN_00131a00(mag, +0x1030): anim +0x44 = mag, +0x40 = mag * +0x48 * +0x4c
                    cx.h.ee_sub_131a00(mag, self.m.u(0x1030));
                }
                if self.state != 10 {
                    self.ee_sub_204660(ang);
                }
            }
            _ => {}
        }
        self.ee_sub_135eb0(obj, cx);
        self.po(cx, 0x1a1358, obj, 0xdf0, &[]);
    }
}

// ---- small slots
impl C2cb230 {
    /// slot 4 FUN_00135ad8(self, msg, args): messages (1 object gone: carry records drop it; 9 anim event)
    pub fn ee_sub_135ad8(&mut self, obj: usize, msg: i32, args: &[u32], h: &mut dyn ClassHost) {
        h.base(0x140f90, obj);
        let a0 = args.first().copied().unwrap_or(0);
        if msg == 1 {
            for i in 0..6u32 {
                let rec = 0x1824 + i * 0xc;
                if self.m.u(rec + 8) == a0 {
                    self.ee_sub_18d698(obj, rec, 0, h);
                }
            }
        } else if msg == 9 {
            let a2 = args.get(2).copied().unwrap_or(0);
            if a0 == self.m.u(0x1030) {
                if a2 == 0x216 {
                    let rec = self.m.u(0x16b4);
                    let host = self.m.u(rec + 4);
                    let c = h.ee_sub_1cb250(host, 0);
                    self.m.set_u(0x186c, c);
                    h.ee_sub_1cb488(host, c);
                    h.obj_set_u32(c, 0x15c, self.m.u(rec));
                    h.vcall_arg(c, 0x264, 0);
                }
            } else if a0 == self.m.u(0x16cc) {
                if a2 == 0x243 {
                    h.ee_sub_1357c0(obj, 0x80000);
                }
            } else {
                h.ee_sub_136ed0(obj, 9, args);
            }
        }
    }
    /// FUN_001eecd0(self + 0x26c): drop the +0x274 link (FUN_001ef0d8) when +0x270 > 0; +0x270 = 0
    pub fn ee_sub_1eecd0(&mut self, h: &mut dyn ClassHost) {
        if 0 < self.m.i(0x270) {
            let x = self.m.u(0x274);
            if x != 0 {
                h.vcall(x, 0x4c);
                h.ee_sub_1eeed0_of(x);
            }
        }
        self.m.set_i(0x270, 0);
    }
    /// base slot 11 FUN_00141928(self): ALO remove FUN_001f2d48, anim slots +0xefc / +0xf14, FUN_001eecd0
    pub fn ee_sub_141928(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1f2d48, obj);
        for off in [0xefc, 0xf14] {
            let mut s = self.m.u(off);
            super::shared::anim::ee_sub_130658(&mut s, h);
            self.m.set_u(off, s);
        }
        self.ee_sub_1eecd0(h);
    }
    /// slot 13 FUN_00134770(self): PO load FUN_002039c0, FUN_001a64e8(self, 0x76, 0x2bc390)
    pub fn ee_sub_134770(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x2039c0, obj);
        h.ee_sub_1a64e8(obj, 0x76, 0x2bc390);
    }
    /// FUN_001426e8(self, oid): +0x10f4 xf act on child `oid` (+0x1150.. rows from FUN_0011b340 = hook)
    pub fn ee_sub_1426e8(&mut self, obj: usize, oid: i32, h: &mut dyn ClassHost) {
        let o = handle(obj);
        let c = h.ee_sub_1f4c58(o, oid, 6);
        if c != 0 {
            h.ee_sub_1426e8_rows(obj, c);
            let x = h.ee_sub_12dc18(o, c, [0.0; 3]);
            self.m.set_u(0x10f4, x);
            h.ee_sub_12d020(x);
        }
    }
    /// slot 14 FUN_001347a8(self): load-post
    pub fn ee_sub_1347a8(&mut self, obj: usize, cx: &mut Cx) {
        let o = handle(obj);
        cx.h.base(0x1400b8, obj);
        let c = cx.h.ee_sub_1f4c58(o, 0xb, 1);
        self.m.set_u(0x10b8, c);
        cx.h.ee_sub_1a0300(obj, 0x13e0, 0x14e0);
        cx.h.ee_sub_1a0300(obj, 0x14f0, 0x15f0);
        cx.h.ee_sub_20b868(obj, 0x1320);
        cx.h.ee_sub_20b868(obj, 0x1600);
        self.ee_sub_136d28(obj, -50.0, 50.0, V_2BC920, cx.h);
        self.ee_sub_1426e8(obj, 0x16, cx.h);
        self.ss(obj, 0, cx);
    }
    /// slot 97 FUN_00135e40(self)
    pub fn ee_sub_135e40(&self, h: &mut dyn ClassHost) -> i32 {
        // FUN_001409b8: FUN_0020f0b0(world) (+0x3d78 > 0) -> 0 else -1
        let v = if 0 < h.world_3d78() { 0 } else { -1 };
        if v != -1 {
            return v;
        }
        match self.state {
            1 | 5 | 0xe => 1,
            _ => -1,
        }
    }
    /// slot 98 FUN_00136870(self, kind): gadget `kind` usable now
    pub fn ee_sub_136870(&self, obj: usize, kind: i32, h: &mut dyn ClassHost) -> bool {
        let _ = obj;
        if self.m.u(0x1818) != 0 || self.m.u(0x181c) != 0 {
            return false;
        }
        if self.binoc.active() {
            return false;
        }
        let ok = match self.state {
            0 | 1 | 5 | 7 => true,
            2 => h.po_call(0x19c8d0, obj, 0x2c0, &[kind as u32]) != 0,
            3 => h.po_call(0x194558, obj, 0x4c0, &[kind as u32]) != 0,
            0xc => {
                if (self.m.i(0x11b4) as u32).wrapping_sub(4) < 3 && matches!(kind, 4 | 7 | 9 | 0xb | 0xd) {
                    return h.obj_i32(self.m.u(0x11c0), 0x15c) == kind;
                }
                false
            }
            0xd => h.po_call(0x19eef0, obj, 0xf50, &[kind as u32]) != 0,
            _ => false,
        };
        if !ok {
            return false;
        }
        match kind {
            1 | 2 | 8 | 10 | 0xc | 0xe => true,
            3 => self.m.i(0x12f0) != 0 && h.g_2dfbc0() & 2 != 0 && !h.ee_sub_14fd50(),
            4 | 5 | 7 | 9 | 0xb | 0xd => {
                let rec = Self::ee_sub_136a28(kind);
                let c = self.m.u(rec + 8);
                if c == 0 {
                    let host = self.m.u(rec + 4);
                    host != 0 && h.ee_sub_1cade0(host, 2)
                } else {
                    h.ee_sub_224590(c)
                }
            }
            _ => false,
        }
    }
    /// slot 99 FUN_00136a90(self, kind): use gadget `kind`
    pub fn ee_sub_136a90(&mut self, obj: usize, kind: i32, cx: &mut Cx) {
        let o = handle(obj);
        // DAT_002deb3c = kind; FUN_001619a0(0x2deb30, 0x3c, self)
        cx.h.ee_sub_1619a0(0x3c, o);
        match kind {
            4 | 5 | 7 | 9 | 0xb | 0xd => {
                let rec = Self::ee_sub_136a28(kind);
                if (self.m.i(0x11b4) as u32).wrapping_sub(4) < 3 {
                    self.po(cx, 0x2206c8, obj, 0x11b0, &[7]);
                    return;
                }
                if self.m.u(rec + 8) != 0 {
                    let c = self.m.u(rec + 8);
                    self.po(cx, 0x1928f8, obj, 0x4c0, &[0xb, c, 0]);
                    return;
                }
                self.m.set_u(0x16b4, rec);
                if kind != 7 && kind != 0xb {
                    self.ee_sub_141cd0(kind, cx.g);
                    self.ss(obj, 6, cx);
                    return;
                }
                let c = cx.h.ee_sub_1cb250(self.m.u(rec + 4), 0);
                if c == 0 {
                    return;
                }
                cx.h.obj_set_u32(c, 0x15c, self.m.u(rec));
                if cx.h.ee_sub_224610(c) {
                    cx.h.obj_set_u32(c, 0x16c, o);
                    self.ee_sub_18d698(obj, rec, c, cx.h);
                }
                self.m.set_u(0x11c0, c);
                self.ee_sub_141ea8(kind);
                self.ss(obj, 0xc, cx);
                self.po(cx, 0x2206c8, obj, 0x11b0, &[4]);
            }
            8 => {
                self.m.set_i(0x9bc, kind);
                self.m.set_u(0x130c, self.m.u(0x1874));
                self.po(cx, 0x1928f8, obj, 0x4c0, &[0xb, 0, 0]);
            }
            10 => {
                self.ee_sub_141cd0(kind, cx.g);
                self.ss(obj, 0xe, cx);
            }
            _ => self.ee_sub_141d38(obj, kind, cx),
        }
    }
    /// slot 104 FUN_00136c50(self, focus): base FUN_00141e70 (empty); part 0x24d shown while focused and
    /// gadget 0xc is in a slot
    pub fn ee_sub_136c50(&self, obj: usize, focused: bool, h: &mut dyn ClassHost) {
        if !focused || self.ee_sub_141ea8(0xc) == 0 {
            h.ee_sub_1f69f8(obj, 0x24d);
        } else {
            h.ee_sub_1f6978(obj, 0x24d);
        }
    }
    /// base slot 71 FUN_001a28b0(self, other): contact with `other` (landing on it in damage states 3 / 10,
    /// else FUN_002297f8 attachment)
    pub fn ee_sub_1a28b0(&mut self, obj: usize, other: u32, h: &mut dyn ClassHost) {
        let k = self.dmg.state;
        if self.f_2dc != 5 && k != 7 {
            if k != 3 && k != 10 {
                h.ee_sub_2297f8(obj, other);
                return;
            }
            // FUN_001a2788 probe (pool FUN_001ae318 / FUN_001ae478, FUN_001f7e08, FUN_0014cdb0)
            let Some(p) = h.ee_sub_1a2788(obj, other) else {
                return;
            };
            // FUN_00228c18 + FUN_0011d1c0 landing push -> FUN_001fac30 (hook: the decompile of the vector
            // part is not recoverable)
            h.ee_sub_1a28b0_push(obj, other, p);
            return;
        }
        let v = h.vcall_10c(obj);
        if v[2] < 0.0 {
            h.ee_sub_2297f8(obj, other);
        }
    }
    /// FUN_001c3ed8(self, k): joystick control mode (+0x150 +0x80); 1 = take the focus, <= 0 drop it.
    /// Returns the focus change: Some(true) take, Some(false) drop, None unchanged.
    pub fn ee_sub_1c3ed8(&mut self, k: i32, focused: bool) -> Option<bool> {
        if k == self.joy_80.unwrap_or(-1) {
            return None;
        }
        // +0x150 == 0: FUN_001ae4f8(0xe0) + FUN_001c4280 joystick alloc (DAT_003e1348 table, max 16) REPLACED
        let mut r = None;
        if (-1..1).contains(&k) {
            if focused {
                r = Some(false);
            }
        } else if k == 1 && !focused {
            r = Some(true);
        }
        // FUN_001c4640
        self.joy_80 = Some(k);
        if k == 1 {
            self.joy_78 = 0xffff;
        }
        r
    }
}

impl ClassSet {
    fn k18_bentley(&mut self, obj: usize) -> Option<(&mut C2cb230, &ClassGlobals, &mut GameRng)> {
        let c = self.k16.iter_mut().find(|x| x.0 == obj).and_then(|x| if let Inst::K16(c) = &mut x.1 { Some(&mut **c) } else { None })?;
        Some((c, &self.g, &mut self.rng))
    }
    /// K18: run a Bentley function with its frame environment (None: no Bentley at `obj`)
    pub fn k18_with<R>(&mut self, obj: usize, h: &mut dyn ClassHost, f: impl FnOnce(&mut C2cb230, &mut Cx) -> R) -> Option<R> {
        let (c, g, rng) = self.k18_bentley(obj)?;
        let mut cx = Cx { g, rng, h };
        Some(f(c, &mut cx))
    }
    /// K18: slot 14 load-post of every Bentley (after slot 6 in build_k16)
    pub(super) fn post_k18_55(&mut self, h: &mut dyn ClassHost) {
        let objs: Vec<usize> = self.k16.iter().map(|x| x.0).collect();
        for o in objs {
            self.k18_with(o, h, |c, cx| {
                c.ee_sub_134770(o, cx.h);
                c.ee_sub_1347a8(o, cx);
            });
        }
    }
    /// K18: base slot 78 FUN_001406e8(obj, k) on a Bentley / Murray: k == 1 makes it the controlled character
    /// (DAT_003e138c focus = `k18_active`; the previous focus gets slot 78(0) / vcall +0x27c(0)), k <= 0 drops it.
    pub fn k18_slot78(&mut self, obj: usize, k: i32, h: &mut dyn ClassHost) {
        let focused = self.k18_active == Some(obj);
        let ch = if let Some(c) = self.k16_get(obj) {
            c.ee_sub_1c3ed8(k, focused)
        } else if let Some(m) = self.k18_murray(obj) {
            m.b.ee_sub_1c3ed8(k, focused)
        } else {
            return;
        };
        match ch {
            Some(true) => {
                if let Some(old) = self.k18_active.take() {
                    self.k18_slot78(old, 0, h);
                }
                // FUN_001c4940(world, obj): world +0x3efc focus, camera / HUD part = hook
                self.k18_active = Some(obj);
                h.ee_sub_1c4940(handle(obj));
            }
            Some(false) => {
                self.k18_active = None;
                h.ee_sub_1c4940(0);
            }
            None => {}
        }
        // FUN_001f82b8(obj, k == 1 ? 0 : -1, 0): sound listener record (hook)
        h.ee_sub_1f82b8(obj, if k == 1 { 0 } else { -1 });
        if k == 1 {
            if let Some(c) = self.k16_get(obj) {
                c.ee_sub_1eecd0(h);
            } else if let Some(m) = self.k18_murray(obj) {
                m.b.ee_sub_1eecd0(h);
            }
        }
    }
    /// K18 player API: make the Bentley / Murray at object index `obj` the controlled character (None: release)
    pub fn k18_set_active(&mut self, obj: Option<usize>, h: &mut dyn ClassHost) {
        match obj {
            Some(o) => self.k18_slot78(o, 1, h),
            None => {
                if let Some(o) = self.k18_active {
                    self.k18_slot78(o, 0, h);
                }
                self.k18_active = None;
            }
        }
    }
    /// K18 player API: the controlled character's joystick for this frame (REPLACED jt +0x150 JOY)
    pub fn k18_feed_joy(&mut self, joy: crate::engine::player::Joy) {
        let Some(o) = self.k18_active else {
            return;
        };
        if let Some(c) = self.k16_get(o) {
            c.joy = joy;
        } else if let Some(m) = self.k18_murray(o) {
            m.b.joy = joy;
        }
    }
    /// K18: per frame, the controlled Bentley: slot 15 update, slot 69 forces, slot 18 draw (the order the
    /// player layer runs JtUpdate / forces / draw for Sly). No-op until `k18_set_active`.
    pub(super) fn frame_k18_55(&mut self, h: &mut dyn ClassHost) {
        let Some(o) = self.k18_active else {
            return;
        };
        self.k18_with(o, h, |c, cx| {
            c.ee_sub_134da8(o, cx);
            c.ee_sub_135940(o, cx.h);
            // K27: the body's physics frame after the slot 69 forces (classes/k27.rs)
            cx.h.k27_po_step(o, super::k27::motor_of(|x| c.m.f(x)), cx.g.dt);
            c.ee_sub_1425b0(o, cx.g, cx.h);
        });
    }
}
