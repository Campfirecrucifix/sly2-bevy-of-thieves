//! WP-K20: class 54 C2d44c8 = Murray, the K19 leftovers (per-state updates and the held-object helpers),
//! plus the class-52 base update FUN_001403d0 shared by Bentley (class 55) and Murray.
//! 1:1: FUN_001b5380 (state 5, +0x1458), FUN_001b5820 (state 6, throw-count sm labels 0x1ca..0x1cf),
//! FUN_001b5990 (state 9, +0x17f4), FUN_001b8988 (state 0xb, +0x1810) + FUN_001b8938, FUN_001b8ec0
//! (state 0xc, +0x1840), FUN_001b70a0 (ally pickup / lift try), FUN_001b7d20 (release the carry acts),
//! FUN_001b7970 / FUN_001b7ae8 (anim event 0x1eb: throw / toss the held object), slot 86 FUN_001b81b8,
//! FUN_001403d0 (class-52 base update: control flow 1:1, every callee = hook).
//! Hooks: `ClassHost::ee_call` (unported engine functions by address, raw words), `ClassHost::po_call`
//! (JT-layout component calls, as K18 / K19), `ClassHost::ee_sub_1f5860` (PO gravity of another object).
//! REPLACED: FUN_001c4840(0x3dd490) == self in FUN_001b5820 = true (ClassSet::frame only updates the
//! controlled character, `k18_active`, which is what world +0x3efc names).
//! Still PARTIAL: FUN_001b93e8 attach (1) / throw (2) setup of the held object (stack matrices through
//! FUN_002309e8 / 1a71b0 / 1b72b0 / 12de58 / 12dc18 / 12dd28 / 12ddb0) = `po_call(0x1b93e8)`.
#![allow(non_snake_case)]
use super::k16_55::C2cb230;
use super::k18_54::C2d44c8;
use super::k18_55::Cx;
use super::*;
use crate::engine::player::FUN_0011e1a0;

fn handle(obj: usize) -> u32 {
    obj as u32 + 1
}

/// DAT_002c08dc: state-9 lift / push press window
pub const W_2C08DC: f32 = 0.15;
/// DAT_002c0900: state-6 re-throw label gap
pub const F_2C0900: f32 = 1.0;
/// DAT_002c0908 / DAT_002c0948: state-6 next label by (label - 0x1cb) + 5 * +0x1534 (last throw / more left)
pub const T_2C0908: [i32; 15] = [463, -1, -1, -1, 464, 461, -1, 463, -1, 464, 460, 461, 462, 463, 464];
pub const T_2C0948: [i32; 15] = [458, -1, -1, -1, 459, 458, -1, 459, -1, 461, 458, 459, 459, 460, 461];
/// DAT_002c09ac / DAT_002c09b0: ally pickup reach and z window
pub const R_2C09AC: f32 = 300.0;
pub const Z_2C09B0: [f32; 2] = [100.0, 500.0];
/// DAT_002c09b8 / DAT_002c09c0: ally lift reach and z window
pub const R_2C09B8: f32 = 400.0;
pub const Z_2C09C0: [f32; 2] = [0.0, 600.0];
/// DAT_002c09d0: state-8 toss velocity (object space)
pub const V_2C09D0: [f32; 3] = [300.0, 0.0, -1000.0];
/// DAT_002c0a30 / DAT_002c0a34: state-0xc timeouts
pub const T_2C0A30: f32 = 0.66;
pub const T_2C0A34: f32 = 0.15;
/// DAT_002c09f8: FUN_00131b60 argument of FUN_001b8938
pub const A_2C09F8: u32 = 0x2c09f8;

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn len_xy(a: [f32; 3]) -> f32 {
    (a[0] * a[0] + a[1] * a[1]).sqrt()
}
/// FUN_0011d910(x, w): w[0] < x < w[1]
fn in_open(x: f32, w: [f32; 2]) -> bool {
    w[0] < x && x < w[1]
}

/// FUN_001d69f0(t, from, to, g): launch velocity reaching `to` from `from` in t under gravity g.z
pub fn FUN_001d69f0(t: f32, from: [f32; 3], to: [f32; 3], g: [f32; 3]) -> [f32; 3] {
    let d = sub(to, from);
    let vz = (d[2] - t * t * 0.5 * g[2]) / t;
    let r = 1.0 / t;
    [d[0] * r, d[1] * r, vz]
}

impl C2cb230 {
    /// FUN_001403d0(self): class-52 base update (Bentley / Murray). Control flow 1:1; callees = hooks
    /// (FUN_00203fc0 PO update, FUN_00141a50 gadgets, FUN_00140338, FUN_00144640 / 1449c8 on +0xfd0, the
    /// +0x274 stack vt+0x74, FUN_001e46a0, FUN_00140558, FUN_001a2460, FUN_00140750, FUN_00142af8,
    /// FUN_00187de0 / FUN_001db490 pause, FUN_001eed10).
    pub fn ee_sub_1403d0(&mut self, obj: usize, cx: &mut Cx) {
        let o = handle(obj);
        let t = cx.g.t;
        cx.h.po_call(0x203fc0, obj, 0, &[]);
        cx.h.po_call(0x141a50, obj, 0, &[]);
        if self.joy.FUN_001c46c0(t, 0.0, 0x200) != 0 {
            cx.h.po_call(0x140338, obj, 0, &[]);
        }
        if self.dmg.state == -1 {
            cx.h.po_call(0x144640, obj, 0xfd0, &[]);
            if cx.h.po_call(0x1449c8, obj, 0xfd0, &[]) == 0 {
                let n = self.m.i(0x270);
                if n >= 1 {
                    let top = self.m.u(0x274 + (n as u32 - 1) * 4);
                    cx.h.vcall(top, 0x74);
                } else if self.m.u(0x1084) != 0 {
                    cx.h.ee_call(0x1e46a0, &[self.m.u(0x1084), o]);
                }
            }
        }
        cx.h.po_call(0x140558, obj, 0, &[]);
        cx.h.po_call(0x1a2460, obj, 0, &[]);
        cx.h.po_call(0x140750, obj, 0, &[]);
        cx.h.po_call(0x142af8, obj, 0, &[]);
        let a = self.m.u(0xefc);
        if a != 0 && cx.h.ee_sub_131968(a) {
            cx.h.ee_sub_1306a8(a);
            self.m.set_u(0xefc, 0);
        }
        if !cx.h.ee_sub_14fd50()
            && self.joy_80 == Some(1)
            && self.joy.FUN_001c4660(t, 0.0, 0x100) != 0
            && cx.h.g_2e0cb4() & 0x108 == 0x100
        {
            cx.h.ee_call(0x187de0, &[0x2dfc00, 0x100]);
            cx.h.ee_call(0x1db490, &[0x3da0f0]);
        }
        if cx.h.po_call(0x1449c8, obj, 0xfd0, &[]) == 0 {
            cx.h.po_call(0x1eed10, obj, 0x26c, &[]);
        }
    }
}

impl C2d44c8 {
    /// FUN_001b7d20(self): drop the twelve carry acts (+0x13f4.. / +0x1600.. / +0x1618..); with an object
    /// held: message 6 to it, restore its two limb bones (+0x1404 / +0x1408), FUN_001f6860(x, 0),
    /// FUN_001a58d0(x, 0, 0xf), +0x1430 = x, +0x1434 = t, FUN_00134320(world, self)
    pub fn ee_sub_1b7d20(&mut self, obj: usize, t: f32, h: &mut dyn ClassHost) {
        for off in [0x13f4, 0x13f8, 0x13fc, 0x1400, 0x1600, 0x1604, 0x1608, 0x160c, 0x1618, 0x161c, 0x1620, 0x1624] {
            h.po_call(0x12e460, obj, off, &[]);
        }
        let c = self.b.m.u(0x13f0);
        if c != 0 {
            h.vcall_arg(c, 0x1c, 6);
            let x = h.obj_i32(c, 0x180) as u32;
            let (b58, b5c) = (h.obj_i32(x, 0x58) as u32, h.obj_i32(x, 0x5c) as u32);
            h.ee_call(0x2303a0, &[x, b58, self.b.m.u(0x1404)]);
            h.ee_call(0x2303a0, &[x, b5c, self.b.m.u(0x1408)]);
            h.ee_call(0x1f6860, &[x, 0]);
            h.ee_call(0x1a58d0, &[x, 0, 0xf]);
            self.b.m.set_u(0x1430, x);
            self.b.m.set_f(0x1434, t);
            h.ee_call(0x134320, &[0x3dd490, handle(obj)]);
        }
    }

    /// FUN_001b7ba8 with the clock: FUN_001b7d20 ported (the K19 form `ee_sub_1b7ba8(obj, h)` keeps the hook)
    pub fn ee_sub_1b7ba8_t(&mut self, obj: usize, t: f32, h: &mut dyn ClassHost) {
        self.drop_held(obj, Some(t), h);
    }

    /// FUN_001b7970(self): anim event 0x1eb in state 7: FUN_001b7d20, release the held object and throw
    /// it at +0x1420 in |xy distance| / +0x1414 s (FUN_001d69f0 ballistic, the object's gravity)
    pub fn ee_sub_1b7970(&mut self, obj: usize, t: f32, h: &mut dyn ClassHost) {
        self.ee_sub_1b7d20(obj, t, h);
        let c = self.b.m.u(0x13f0);
        if c == 0 {
            return; // ponytail: the original dereferences a null +0x13f0 here
        }
        let x = h.obj_i32(c, 0x180) as u32;
        self.b.m.set_u(0x13f0, 0);
        if h.obj_flags_28(x) >> 60 & 1 == 0 {
            return;
        }
        if h.ee_sub_132b10(x, 0x38) != 0 {
            h.ee_call(0x209fb8, &[x]);
        }
        let p = h.ee_sub_1a62e8(x);
        let to = self.b.m.v3(0x1420);
        let tt = len_xy(sub(p, to)) / self.b.m.f(0x1414);
        let g = h.ee_sub_1f5860(x);
        let v = FUN_001d69f0(tt, p, to, g);
        h.vcall_11c_v(x, v);
        h.po_call(0x1f7a68, obj, 0, &[x, 2]);
    }

    /// FUN_001b7ae8(self): anim event 0x1eb in state 8: FUN_001b7d20, release and toss the held object
    /// (DAT_002c09d0 in Murray's space), vt+0x124, FUN_001f7a68(x, 3)
    pub fn ee_sub_1b7ae8(&mut self, obj: usize, t: f32, h: &mut dyn ClassHost) {
        self.ee_sub_1b7d20(obj, t, h);
        let c = self.b.m.u(0x13f0);
        if c == 0 {
            return;
        }
        let x = h.obj_i32(c, 0x180) as u32;
        self.b.m.set_u(0x13f0, 0);
        if h.obj_flags_28(x) >> 60 & 1 == 0 {
            return;
        }
        if h.ee_sub_132b10(x, 0x38) != 0 {
            h.ee_call(0x209fb8, &[x]);
        }
        let v = [0, 1, 2].map(|i| {
            let (a, b, c) = (self.b.m.v3(0x1b0), self.b.m.v3(0x1c0), self.b.m.v3(0x1d0));
            a[i] * V_2C09D0[0] + b[i] * V_2C09D0[1] + c[i] * V_2C09D0[2]
        });
        h.vcall_11c_v(x, v);
        h.vcall(x, 0x124);
        h.po_call(0x1f7a68, obj, 0, &[x, 3]);
    }

    /// slot 86 FUN_001b81b8(self, k) 1:1: base FUN_00205d10; k == 0: un-hide the remembered +0xf1c
    /// (vt+0xac 1, class 0x33: vt+0x2bc 0); else remember / hide the held object's +0x180 (vt+0xac 0,
    /// class 0x33: vt+0x2bc k)
    pub fn ee_sub_1b81b8_k20(&mut self, obj: usize, k: i32, h: &mut dyn ClassHost) {
        h.base(0x205d10, obj);
        if k == 0 {
            let c = self.b.m.u(0xf1c);
            if c != 0 {
                h.vcall_arg(c, 0xac, 1);
                if h.ee_sub_132b10(c, 0x33) != 0 {
                    h.vcall_arg(c, 0x2bc, 0);
                }
                self.b.m.set_u(0xf1c, 0);
            }
        } else {
            let c = self.b.m.u(0x13f0);
            if c == 0 {
                return;
            }
            let x = h.obj_i32(c, 0x180) as u32;
            if x == 0 {
                return;
            }
            self.b.m.set_u(0xf1c, x);
            h.vcall_arg(x, 0xac, 0);
            if h.ee_sub_132b10(x, 0x33) != 0 {
                h.vcall_arg(x, 0x2bc, k);
            }
        }
    }

    /// FUN_001b70a0(self): ally +0x10dc in reach: pick it up (Bentley in the 0xb1 slot of its +0x26c list,
    /// FUN_0017b1a8 + FUN_001b7050) or lift it (SetState 0xb, sub-state 4). 1 = handled.
    pub fn ee_sub_1b70a0(&mut self, obj: usize, cx: &mut Cx) -> bool {
        let t = cx.g.t;
        let ally = self.b.m.u(0x10dc);
        if ally == 0 {
            return false;
        }
        if cx.h.ee_call(0x209a68, &[ally, 0x17]) == 0 || self.b.joy.FUN_001c4660(t, 0.15, 0xa0) == 0 {
            return false;
        }
        let Some((rec, n50, o5c)) = cx.h.ee_sub_1eeee0_rec(ally, 0xb1) else {
            return false;
        };
        if n50 >= 3 {
            return false;
        }
        if o5c != handle(obj) {
            return false;
        }
        let d = sub(cx.h.ee_sub_1a62e8(ally), cx.h.ee_sub_1a62e8(handle(obj)));
        let n = len_xy(d);
        let mut go = false;
        if n < R_2C09AC && in_open(d[2], Z_2C09B0) && cx.h.obj_i32(ally, 0xfec) != 0 {
            go = self.b.joy.FUN_001c46c0(t, 0.15, 0x20) != 0;
        }
        if go {
            cx.h.ee_call(0x17b1a8, &[rec, 2]);
            let f = cx.h.obj_i32(ally, 0xfec) as u32;
            cx.h.po_call(0x1b7050, obj, 0, &[f]);
            return true;
        }
        let mut go = false;
        if n < R_2C09B8 && in_open(d[2], Z_2C09C0) && self.b.m.u(0x180c) != 0 {
            go = self.b.joy.FUN_001c46c0(t, 0.15, 0x80) != 0;
        }
        if go {
            self.ee_sub_1b6078(obj, 0xb, cx);
            self.ee_sub_1b86b0(obj, 4, cx);
            return true;
        }
        false
    }

    /// FUN_001b5380(self): state-5 (holding) update by +0x1458: object gone -> SetState 0; Square (0x40,
    /// gadget 0x12) -> JMP 0xf; 0xa0 -> throw (7); 0x10 -> toss (8, +0x1438); 0x400 binocucom; else the
    /// stand / walk sub-state by the stick
    pub fn ee_sub_1b5380(&mut self, obj: usize, cx: &mut Cx) {
        let o = handle(obj);
        let mut old = self.b.m.i(0x1458);
        loop {
            let t = cx.g.t;
            'c: {
                if !(0..3).contains(&old) {
                    break 'c;
                }
                if self.b.held_180_k20(cx.h).is_none() {
                    self.ee_sub_1b7d20(obj, t, cx.h);
                    self.b.m.set_u(0x13f0, 0);
                    self.ee_sub_1b6078(obj, 0, cx);
                    break 'c;
                }
                if self.b.joy.FUN_001c4660(t, 0.1, 0x40) != 0 && cx.h.po_call(0x1daea8, obj, 0, &[0x12]) != 0 {
                    self.b.joy.FUN_001c4738(0x40);
                    cx.h.po_call(0x195a00, obj, 0x2c0, &[0xf, 0, 0]);
                    break 'c;
                }
                if self.b.joy.FUN_001c46c0(t, 0.0, 0xa0) != 0 {
                    self.ee_sub_1b6078(obj, 7, cx);
                    break 'c;
                }
                if self.b.joy.FUN_001c46c0(t, 0.0, 0x10) != 0 && self.b.m.u(0x1438) != 0 {
                    self.b.m.set_u(0x143c, 0);
                    self.ee_sub_1b6078(obj, 8, cx);
                    break 'c;
                }
                if self.b.joy.FUN_001c4660(t, 0.15, 0x400) != 0 && cx.h.vcall_ret_arg(o, 0x31c, 3) != 0 {
                    self.b.joy.FUN_001c4738(0x400);
                    cx.h.po_call(0x19cc50, obj, 0xf00, &[0, 0, 0]);
                    break 'c;
                }
                let thr = if self.b.m.i(0x1458) == 0 { 0.1 } else { 0.0 };
                if self.b.joy.mag <= thr {
                    self.ee_sub_1b5658(obj, 0, cx);
                } else if self.b.joy.held & 8 == 0 || cx.h.po_call(0x1daea8, obj, 0, &[0x12]) == 0 {
                    self.ee_sub_1b5658(obj, 1, cx);
                } else {
                    self.ee_sub_1b5658(obj, 2, cx);
                }
            }
            let cur = self.b.m.i(0x1458);
            if cur == old || self.b.state != 5 {
                match cur {
                    1 => {
                        self.b.ee_sub_204660(self.b.joy.angle);
                        cx.h.po_call(0x20bb58, obj, 0x1460, &[self.b.joy.mag.to_bits()]);
                        cx.h.po_call(0x20bc08, obj, 0x1460, &[]);
                    }
                    2 => {
                        self.b.ee_sub_204660(self.b.joy.angle);
                        let r = cx.h.root_vel_204e38(self.b.m.u(0x1030));
                        self.b.m.set_v3(0x190, [-r[0], -r[1], -r[2]]);
                    }
                    _ => {}
                }
                return;
            }
            old = cur;
        }
    }

    /// FUN_001b5820(self): state-6 (throw volley) update: while the sm +0x1034 rests on a label in
    /// 0x1cb..0x1cf, +0x1524 counts down; 0xf0 counts a throw (+0x152c -1, +0x1528 = +0x1524); at 0 the
    /// next label comes from DAT_002c0908 / DAT_002c0948, a pause > DAT_002c0900 goes to 0x1ca.
    pub fn ee_sub_1b5820(&mut self, cx: &mut Cx) {
        // FUN_001c4840(0x3dd490) == self: REPLACED (only the controlled character is updated)
        if cx.h.world_3d78() > 0 {
            return;
        }
        let sm = self.b.m.u(0x1034);
        let a = cx.h.ee_call(0x1f07e8, &[sm]) as i32;
        let b = cx.h.ee_call(0x1f07c8, &[sm]) as i32;
        if a != b || !(0x1cb..0x1d0).contains(&a) {
            return;
        }
        let mut lab = a;
        let f = self.b.m.f(0x1524) - cx.g.dt;
        self.b.m.set_f(0x1524, f);
        if self.b.joy.FUN_001c46c0(cx.g.t, 0.0, 0xf0) != 0 {
            self.b.m.set_i(0x152c, self.b.m.i(0x152c) - 1);
            self.b.m.set_f(0x1528, f);
        }
        if f <= 0.0 {
            let tab = if self.b.m.i(0x152c) < 1 { &T_2C0908 } else { &T_2C0948 };
            let i = (a - 0x1cb) + self.b.m.i(0x1534) * 5;
            lab = tab.get(i as usize).copied().unwrap_or(-1);
        } else if F_2C0900 < self.b.m.f(0x1528) - f {
            lab = 0x1ca;
        }
        let s2 = self.b.m.u(0x1530);
        if s2 != 0 {
            cx.h.ee_call(0x1f0660, &[s2, lab as u32]);
        }
        cx.h.ee_call(0x1f0660, &[sm, lab as u32]);
    }

    /// FUN_001b5990(self): state-9 (crouch) update by +0x17f4 (Square: JMP 1, pickup try, Triangle: grab
    /// with the +0x4c0 component, lift / push, binocucom, else the sub-state by the stick)
    pub fn ee_sub_1b5990(&mut self, obj: usize, cx: &mut Cx) {
        let o = handle(obj);
        let mut old = self.b.m.i(0x17f4);
        loop {
            let t = cx.g.t;
            'c: {
                if !(0..3).contains(&old) {
                    break 'c;
                }
                if !(self.b.joy.held & 8 == 0 && cx.h.po_call(0x140af8, obj, 0, &[0]) != 0) {
                    self.ee_sub_1b6078(obj, 0, cx);
                    break 'c;
                }
                if self.b.joy.FUN_001c46c0(t, 0.1, 0x40) != 0 {
                    cx.h.po_call(0x195a00, obj, 0x2c0, &[1, 0, 0]);
                    break 'c;
                }
                if self.ee_sub_1b70a0(obj, cx) {
                    break 'c;
                }
                if self.b.joy.FUN_001c46c0(t, 0.2, 0x20) != 0 {
                    if cx.h.po_call(0x1938c0, obj, 0x4c0, &[0x80, 0]) == 0 && self.b.m.u(0x13ec) != 0 {
                        self.b.m.set_i(0x144c, 0);
                        self.ee_sub_1b6078(obj, 4, cx);
                    }
                    break 'c;
                }
                if self.ee_sub_1b84e0() && self.b.joy.FUN_001c46c0(t, W_2C08DC, 0x80) != 0 {
                    self.ee_sub_1b8508(obj, cx);
                    break 'c;
                }
                if self.b.joy.FUN_001c4660(t, 0.15, 0x400) != 0 && cx.h.vcall_ret_arg(o, 0x31c, 3) != 0 {
                    self.b.joy.FUN_001c4738(0x400);
                    cx.h.po_call(0x19cc50, obj, 0xf00, &[0, 0, 0]);
                    break 'c;
                }
                if self.ee_sub_1b84e0() && self.b.joy.FUN_001c46c0(t, W_2C08DC, 0x10) != 0 {
                    self.ee_sub_1b85d0(obj, cx);
                    break 'c;
                }
                // 1:1: the threshold tests +0x13d0 (the state, 9 here), not +0x17f4
                let thr = if self.b.state == 0 { 0.1 } else { 0.0 };
                let s = if thr < self.b.joy.mag { 2 } else { 0 };
                self.ee_sub_1b5d30(obj, s, cx);
            }
            let cur = self.b.m.i(0x17f4);
            if cur == old || self.b.state != 9 {
                match cur {
                    1 => {
                        self.b.ee_sub_204660(self.b.joy.angle);
                        cx.h.po_call(0x20bb58, obj, 0x1630, &[self.b.joy.mag.to_bits()]);
                        cx.h.po_call(0x20bc08, obj, 0x1630, &[]);
                    }
                    0 => cx.h.ee_sub_2046f0(obj),
                    2 => {
                        cx.h.ee_sub_2046f0(obj);
                        let n = len_xy(self.b.m.v3(0x1720));
                        let rate = cx.h.anim_slot_f(self.b.m.u(0x1030), 0x40);
                        let j = &self.b.joy;
                        let v = FUN_0011e1a0(j.angle + std::f32::consts::PI, j.mag * n * rate, 0.0);
                        let d = |r: [f32; 3]| r[0] * v[0] + r[1] * v[1] + r[2] * v[2];
                        let l = [d(self.b.m.v3(0x1b0)), d(self.b.m.v3(0x1c0)), d(self.b.m.v3(0x1d0))];
                        self.b.m.set_i(0x1038, -1);
                        self.b.m.set_v3(0x190, l);
                        self.b.m.set_f(0x19c, 0.0);
                        cx.h.po_call(0x1a0450, obj, 0x16f0, &[self.b.m.u(0x17f0)]);
                    }
                    _ => {}
                }
                return;
            }
            old = cur;
        }
    }

    /// FUN_001b8938(self): end the lift anim (FUN_00131b60(+0x1030, self, 5, DAT_002c09f8), kept running in
    /// +0x190c) and SetState 0
    pub fn ee_sub_1b8938(&mut self, obj: usize, cx: &mut Cx) {
        let sl = self.b.m.u(0x1030);
        cx.h.ee_call(0x131b60, &[sl, handle(obj), 5, A_2C09F8]);
        self.b.m.set_u(0x1030, 0);
        self.b.m.set_u(0x190c, sl);
        self.ee_sub_1b6078(obj, 0, cx);
    }

    /// FUN_001b8988(self): state-0xb (ally lift) update by +0x1810 (anim over -> SetState 0; Triangle /
    /// Circle re-lift, pickup try, +0x181c window + stick -> FUN_001b8938)
    pub fn ee_sub_1b8988(&mut self, obj: usize, cx: &mut Cx) {
        let mut sl = self.b.m.u(0x1030);
        loop {
            let prev = self.b.m.i(0x1810);
            let t = cx.g.t;
            'c: {
                if sl == 0 || cx.h.ee_sub_131968(sl) {
                    self.ee_sub_1b6078(obj, 0, cx);
                    break 'c;
                }
                let s = self.b.m.i(0x1810);
                if !(0..4).contains(&s) {
                    break 'c;
                }
                let (mask, next) = if s < 2 { (0x80, (s == 0) as i32) } else { (0x10, if s != 2 { 2 } else { 3 }) };
                if self.ee_sub_1b84e0() && self.b.joy.FUN_001c46c0(t, 0.25, mask) != 0 {
                    self.ee_sub_1b86b0(obj, next, cx);
                    break 'c;
                }
                if self.ee_sub_1b84e0() && self.ee_sub_1b70a0(obj, cx) {
                    break 'c;
                }
                if self.b.m.i(0x181c) != 0 && 0.0 < self.b.joy.mag {
                    self.ee_sub_1b8938(obj, cx);
                }
            }
            let cur = self.b.m.i(0x1810);
            if cur == prev || cur == -1 {
                return;
            }
            sl = self.b.m.u(0x1030);
        }
    }

    /// FUN_001b8ec0(self): state-0xc (push) update by +0x1840 (0 anim over -> 1; 1: +0x1908 -> 2, timeout
    /// DAT_002c0a30 -> 3; 2: the +0x1900 record (FUN_0020c0d8 / 20c0e8) then +0x1ac0 -> 0 else SetState 0;
    /// 3: after DAT_002c0a34 with the stick -> SetState 0)
    pub fn ee_sub_1b8ec0(&mut self, obj: usize, cx: &mut Cx) {
        let mut cur = self.b.m.i(0x1840);
        loop {
            let t = cx.g.t;
            let over = |s: &Self, h: &mut dyn ClassHost| {
                let sl = s.b.m.u(0x1030);
                sl == 0 || h.ee_sub_131968(sl)
            };
            match cur {
                1 => {
                    if over(self, cx.h) {
                        self.ee_sub_1b6078(obj, 0, cx);
                    } else if self.b.m.u(0x1908) != 0 {
                        self.ee_sub_1b8d30(obj, 2, cx);
                    } else if self.b.m.u(0x1ac0) == 0 && T_2C0A30 < t - self.b.m.f(0x1844) {
                        self.ee_sub_1b8d30(obj, 3, cx);
                    }
                }
                0 => {
                    if over(self, cx.h) {
                        self.ee_sub_1b8d30(obj, 1, cx);
                    }
                }
                2 => 'c: {
                    let rec = self.b.m.u(0x1900);
                    // +0x1900 = the +0x1890 record kept as its byte offset (K19): component call on self
                    if rec != 0 && cx.h.po_call(0x20c0d8, obj, rec, &[]) == 0 {
                        cx.h.po_call(0x20c0e8, obj, rec, &[]);
                        break 'c;
                    }
                    if self.b.m.u(0x1ac0) == 0 {
                        self.ee_sub_1b6078(obj, 0, cx);
                    } else {
                        self.ee_sub_1b8d30(obj, 0, cx);
                    }
                }
                3 => {
                    if over(self, cx.h) {
                        self.ee_sub_1b6078(obj, 0, cx);
                    } else if T_2C0A34 < t - self.b.m.f(0x1844) && self.b.joy_80 == Some(1) && 0.0 < self.b.joy.mag {
                        self.ee_sub_1b6078(obj, 0, cx);
                    }
                }
                _ => {}
            }
            let n = self.b.m.i(0x1840);
            if n == cur || n == -1 {
                return;
            }
            cur = n;
        }
    }
}

impl C2cb230 {
    /// held_180 for K20 (same rule as K19's private helper: +0x13f0 +0x180 with LO flag bit 60)
    pub fn held_180_k20(&self, h: &mut dyn ClassHost) -> Option<u32> {
        let c = self.m.u(0x13f0);
        if c == 0 {
            return None;
        }
        let x = h.obj_i32(c, 0x180) as u32;
        (h.obj_flags_28(x) >> 60 & 1 != 0).then_some(x)
    }
}
