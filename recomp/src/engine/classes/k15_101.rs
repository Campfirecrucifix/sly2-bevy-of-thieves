//! WP-K15: class 101 C2c9810 (parent class 100 C2d89a0): the thrown bomb (one template object, oid 3931, in
//! every level; rows 1102 sound object +0x164, 1108 damage +0x178 = -20). +0x15c (row 1104) is the thrower kind
//! (0x1a / 0x29 keep the bomb upright and, for 0x29, aimed at the nearest class-0x38 object +0x160). States
//! (+0x150, slot 75 FUN_00224ec8): 0 resting (pinned by two constraints), 1 flying, 2 stuck (kind 0xb), 3 fuse
//! (by a hit, slot 74 / message 4), 4 gone; each new state plays sound 0x6d + state on +0x168. Contacts in
//! flight (slot 57) queue +0x158 = 2 (stick to the contact object +0x1a0, normal +0x190), slot 15 applies it
//! (FUN_00224258: kinds 0x1a / 0x29 go to 3 instead) and sends slot 74 when the linked object +0x16c (row
//! 1111) is further than +0x170 (row 1112, 3000). Method 1109 = FUN_00224258, method 1110 = FUN_00224e70
//! (explode at the bomb: radius +0x174 (row 1107), damage +0x178).
//! Own slots 4 FUN_002241d8, 5 FUN_00114060, 6 FUN_00223f70, 10 FUN_00224028, 11 FUN_00224068, 13
//! FUN_00223fe0, 15 FUN_002240b0, 54 FUN_00224350, 57 FUN_00224380, 60 FUN_002242e0, 69 FUN_00224698, 74
//! FUN_00224190, 75 FUN_00224ec8 1:1 with FUN_00224258 / 224508 / 224638 / 224688 / 224e70, FUN_00134310
//! (max), FUN_001f5c30 (orientation servo), FUN_0011b938 (upright rows).
//! REPLACED: the SO contact list walk of slot 57 (+0xec +0x64 pairs) -> `Contact57` records from
//! `ClassHost::so_contacts_57`; the FUN_0015e9c8(0x105, 0x38) neighbour query -> `ClassHost::query_15e9c8`;
//! the explosion FUN_00224bf8 (11 rings of FUN_0011e208 probe rays + FUN_0020fc60 neighbour query through
//! FUN_00224820 line tests, 0x60-byte hit records to FUN_001f7598, FUN_00156698 fx) -> one
//! `ClassHost::explode_224bf8(self, pos, radius, damage)` call; the stuck-state (2, kind 0xb) angular
//! constraint orientation (FUN_0022fcc8 arguments lost in the decompile) -> the current orientation.
#![allow(non_snake_case)]
use super::*;
use crate::engine::anim::motion::ee_sub_11bff0;
use crate::engine::camera::looks::FUN_0011b7e0;
use crate::engine::physics::driver::{AloPhys, FUN_0011b3d0, FUN_001f9b20, FUN_0022de88, M3};
use crate::engine::player::p5_states::{FUN_0011ba08, FUN_0011bc00, FUN_0011bcf8};

/// slot 5 FUN_00114060 (class descriptor getter)
pub fn slot5(class: ClassId) -> Option<u32> {
    match class {
        101 => Some(0x2bb2cc),
        102 => Some(super::k23_102::ee_sub_114070()),
        _ => None,
    }
}

/// class 101 field rows -> member offsets (FIELD_MEMBER; 1105 / 1106 no member, 1109 / 1110 methods)
pub const ROWS_101: &[(i16, u16)] = &[(1102, 0x164), (1103, 0x168), (1104, 0x15c), (1107, 0x174), (1108, 0x178), (1111, 0x16c), (1112, 0x170)];

/// DAT_002c42e0: contacts are ignored this long after entering state 0
pub const D_2C42E0: f32 = 0.25;
/// DAT_002c42e8: classes that cannot be stuck to (FUN_00224508, the object or any parent +0x10)
pub const C_2C42E8: [i32; 4] = [0x33, 0x64, 0x4f, 0x67];
/// DAT_002c3c60: contact normal sign per half
pub const S_2C3C60: [f32; 2] = [1.0, -1.0];
/// 0x2c4398: sound per state
pub const T_2C4398: [i32; 5] = [0x6d, 0x6e, 0x6f, 0x70, 0x71];
/// 0x2c4300 orientation servo {0, 45, 0}, 0x2c4310 angular damping {0, 7.5, 0}
pub const P_2C4300: [f32; 3] = [0.0, 45.0, 0.0];
pub const P_2C4310: [f32; 3] = [0.0, 7.5, 0.0];

/// one SO contact of slot 57 seen from this object: the other half's object (+0xc), the contact kind (+0x9c),
/// normal (+0x80) and whether this object is the second half (bVar2)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Contact57 {
    pub other: u32,
    pub kind: i32,
    pub normal: [f32; 3],
    pub second: bool,
}

#[derive(Clone, Debug, Default)]
pub struct C2c9810 {
    /// xfLin / xfAng and the +0xe8 block
    pub phys: AloPhys,
    /// +0x40 flags, +0x148 SO flags
    pub f_40: u32,
    pub f_148: u64,
    /// +0xe8 +0xd4 / +0xd8 (set to 2 by slot 13)
    pub e8_d4: f32,
    pub e8_d8: f32,
    /// +0x150 state (-1 none), +0x154 its start time, +0x158 pending state (-1 none)
    pub f_150: i32,
    pub f_154: f32,
    pub f_158: i32,
    /// +0x15c thrower kind (row 1104), +0x160 aim object
    pub f_15c: i32,
    pub f_160: u32,
    /// +0x164 sound object (row 1102), +0x168 its handle
    pub f_164: u32,
    pub f_168: u32,
    /// +0x16c linked object (row 1111), +0x170 leash distance (row 1112)
    pub f_16c: u32,
    pub f_170: f32,
    /// +0x174 explosion radius (row 1107), +0x178 damage (row 1108)
    pub f_174: f32,
    pub f_178: i32,
    /// +0x17c / +0x180 angular / linear constraints (FUN_0012dc18 handles)
    pub f_17c: u32,
    pub f_180: u32,
    /// +0x190 contact normal, +0x1a0 contact object
    pub f_190: [f32; 3],
    pub f_1a0: u32,
    /// K23: the class-102 C2db8a0 members (None = class 101); its slots 15 / 75 override (classes/k23_102.rs)
    pub k23: Option<super::k23_102::C2db8a0>,
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn len(a: [f32; 3]) -> f32 {
    dot(a, a).sqrt()
}
fn handle(obj: usize) -> u32 {
    obj as u32 + 1
}

/// FUN_0011b938(rows, 0, &out): the rows turned by the shortest arc taking row 2 to +Z (upright)
pub fn FUN_0011b938(r: &M3) -> M3 {
    let m = FUN_0011ba08(r[2], [0.0, 0.0, 1.0]);
    r.map(|row| [0, 1, 2].map(|k| row[0] * m[0][k] + row[1] * m[1][k] + row[2] * m[2][k]))
}

/// FUN_001f5c30(obj, target, p, 0, d) for an unparented body: alpha += axis * poly(p, angle to the target rows);
/// then the angular velocity is damped toward 0: alpha += -w * min(d2 |w| + d1 + d0 / |w|, 1 / dt)
pub fn FUN_001f5c30(b: &mut AloPhys, target: &M3, p: [f32; 3], d: [f32; 3], dt: f32) {
    let mut t = *target;
    FUN_0011b3d0(&mut t);
    let r = FUN_0011bc00(&b.rot, &t);
    let (ang, axis) = FUN_0011bcf8(&r);
    if 0.0001 < ang {
        let k = p[2] * ang * ang + p[1] * ang + p[0];
        b.alpha = [0, 1, 2].map(|i| b.alpha[i] + axis[i] * k);
    }
    let e = b.w.map(|x| -x);
    let l = len(e);
    if 0.0001 < l {
        let mut k = d[2] * l + d[1] + d[0] / l;
        if 1.0 / dt < k {
            k = 1.0 / dt;
        }
        b.alpha = [0, 1, 2].map(|i| b.alpha[i] + e[i] * k);
    }
}

/// per-frame context (DAT_002dded0 time, DAT_002dded4 dt)
pub struct Cx<'a> {
    pub t: f32,
    pub dt: f32,
    pub h: &'a mut dyn ClassHost,
}

impl C2c9810 {
    /// slot 6 FUN_00223f70: FUN_001f2108; +0x178 = -10, +0x170 = 3000, state / pending -1, +0x40 |= 6
    pub fn ee_sub_223f70(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1f2108, obj);
        self.f_178 = -10;
        self.f_170 = 3000.0;
        self.f_150 = -1;
        self.f_158 = -1;
        self.f_40 |= 6;
    }

    /// slot 13 FUN_00223fe0: FUN_001f31e0; with a physics block +0xd4 = +0xd8 = 2
    pub fn ee_sub_223fe0(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1f31e0, obj);
        self.e8_d4 = 2.0;
        self.e8_d8 = 2.0;
    }

    /// slot 10 FUN_00224028: FUN_001f2c30; the sound object +0x164 starts on +0x168 (FUN_001f1448)
    pub fn ee_sub_224028(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1f2c30, obj);
        if self.f_164 != 0 {
            self.f_168 = h.ee_sub_1f1448(self.f_164, handle(obj), 0);
        }
    }

    /// slot 11 FUN_00224068: FUN_001f2d48; slot 75 (-1); sound off (FUN_001f15b8)
    pub fn ee_sub_224068(&mut self, obj: usize, cx: &mut Cx) {
        cx.h.base(0x1f2d48, obj);
        self.slot75(obj, -1, cx);
        cx.h.ee_sub_1f15b8(&mut self.f_168);
    }

    /// FUN_00224638: thrower kinds that stick: 4 (only while flying), 5, 7, 9, 0xb, 0xd, 0x1a, 0x25, 0x29
    pub fn ee_sub_224638(&self) -> bool {
        match self.f_15c {
            4 => self.f_150 == 1,
            5 | 7 | 9 | 0xb | 0xd | 0x1a | 0x25 | 0x29 => true,
            _ => false,
        }
    }

    /// FUN_00224688: thrower kind 0xb
    pub fn ee_sub_224688(&self) -> bool {
        self.f_15c == 0xb
    }

    /// FUN_00224508(self, other): neither `other` nor any parent (+0x10) is one of `C_2C42E8`
    pub fn ee_sub_224508(&self, other: u32, h: &mut dyn ClassHost) -> bool {
        let mut o = other;
        let mut guard = 0;
        while o != 0 && guard < 64 {
            if C_2C42E8.iter().any(|&c| h.ee_sub_132b10(o, c) != 0) {
                return false;
            }
            o = h.parent_10(o);
            guard += 1;
        }
        true
    }

    /// FUN_00224258(self, other) (method 1109): thrower kinds 0x1a / 0x29 -> state 3; else stick to `other`
    /// (none when its +0x18 == 1; FUN_001a58d0(self, other, 3)) and state 2
    pub fn ee_sub_224258(&mut self, obj: usize, other: u32, cx: &mut Cx) {
        if self.f_15c == 0x1a || self.f_15c == 0x29 {
            self.slot75(obj, 3, cx);
            return;
        }
        let o = if other != 0 && cx.h.obj_i32(other, 0x18) == 1 { 0 } else { other };
        cx.h.ee_sub_1a58d0_to(obj, o, 3);
        self.slot75(obj, 2, cx);
    }

    /// slot 75 FUN_00224ec8(state): on a change: pending -1, start time, both constraints released
    /// (FUN_0012e460); leaving 0 / 4 -> FUN_00134320. 1: FUN_001a58d0(self, 0, 0xf), FUN_001f5720(0),
    /// FUN_001f57c0(kind 0x29); kind 0x29 aims at the nearest class-0x38 object (z distance doubled). 0:
    /// FUN_001f5720 / 1f57c0 (0), velocities 0, pinned by a linear and an angular constraint (FUN_0012dc18 at the
    /// position / euler angles), FUN_00134320. 2: FUN_001f5720 / 1f57c0 (1), velocities 0; kind 0xb adds the
    /// angular constraint. 4: FUN_00134320, FUN_001f7bf8(0.25). Then sound `T_2C4398[state]`.
    pub fn ee_sub_224ec8(&mut self, obj: usize, s: i32, cx: &mut Cx) {
        let old = self.f_150;
        if s == old {
            return;
        }
        self.f_150 = s;
        self.f_158 = -1;
        self.f_154 = cx.t;
        cx.h.ee_sub_12e460(&mut self.f_180);
        cx.h.ee_sub_12e460(&mut self.f_17c);
        if old == 0 || old == 4 {
            cx.h.ee_sub_134320(obj);
        }
        let me = handle(obj);
        match s {
            1 => {
                cx.h.ee_sub_1a58d0(obj, 0, 0xf);
                cx.h.ee_sub_1f5720(obj, 0);
                if self.f_15c != 0x29 {
                    cx.h.ee_sub_1f57c0(obj, 0);
                } else {
                    cx.h.ee_sub_1f57c0(obj, 1);
                    // FUN_0015e9c8(0x105, 0x38, .., 0x20): REPLACED by the host query
                    let mut best = f32::MAX;
                    let mut out = 0;
                    let sp = self.phys.pos;
                    for o in cx.h.query_15e9c8(0x38).into_iter().take(0x20) {
                        let mut d = sub(cx.h.ee_sub_1a62e8(o), sp);
                        d[2] += d[2];
                        let l = len(d);
                        if l < best {
                            best = l;
                            out = o;
                        }
                    }
                    self.f_160 = out;
                }
            }
            0 => {
                cx.h.ee_sub_1f5720(obj, 0);
                cx.h.ee_sub_1f57c0(obj, 0);
                self.phys.v = [0.0; 3];
                self.phys.w = [0.0; 3];
                let e = ee_sub_11bff0(&m4(&self.phys.rot));
                self.f_180 = cx.h.ee_sub_12dc18(me, 0x58, self.phys.pos);
                self.f_17c = cx.h.ee_sub_12dc18(me, 0x5c, e);
                cx.h.ee_sub_134320(obj);
            }
            2 => {
                cx.h.ee_sub_1f5720(obj, 1);
                cx.h.ee_sub_1f57c0(obj, 1);
                self.phys.v = [0.0; 3];
                self.phys.w = [0.0; 3];
                if self.ee_sub_224688() {
                    // REPLACED: the FUN_0022fcc8 / FUN_0011b830 rows (arguments lost) -> current orientation
                    let e = ee_sub_11bff0(&m4(&self.phys.rot));
                    self.f_17c = cx.h.ee_sub_12dc18(me, 0x5c, e);
                }
            }
            4 => {
                cx.h.ee_sub_134320(obj);
                cx.h.ee_sub_1f7bf8(obj, 0.25);
            }
            _ => {}
        }
        if self.f_168 != 0 && -1 < self.f_150 {
            if let Some(&snd) = T_2C4398.get(self.f_150 as usize) {
                cx.h.ee_sub_1f0730(self.f_168, snd);
            }
        }
    }

    /// the virtual slot 75 ((vtable +0x264)(self, s)): class 102 FUN_00225660, else FUN_00224ec8
    pub fn slot75(&mut self, obj: usize, s: i32, cx: &mut Cx) {
        if self.k23.is_some() {
            super::k23_102::ee_sub_225660(self, obj, s, cx);
        } else {
            self.ee_sub_224ec8(obj, s, cx);
        }
    }

    /// slot 15: class 102 FUN_00225300, else FUN_002240b0
    pub fn slot15(&mut self, obj: usize, cx: &mut Cx) {
        if self.k23.is_some() {
            super::k23_102::ee_sub_225300(self, obj, cx);
        } else {
            self.ee_sub_2240b0(obj, cx);
        }
    }

    /// slot 74 FUN_00224190: flying / stuck -> state 3. Returns 1.
    pub fn ee_sub_224190(&mut self, obj: usize, cx: &mut Cx) -> bool {
        if self.f_150 < 3 && 0 < self.f_150 {
            self.slot75(obj, 3, cx);
        }
        true
    }

    /// slot 4 FUN_002241d8(self, msg, p): FUN_00133590; message 4 about this object while flying / stuck -> 3
    pub fn ee_sub_2241d8(&mut self, obj: usize, msg: i32, about: u32, cx: &mut Cx) {
        cx.h.base(0x133590, obj);
        if msg == 4 && about == handle(obj) && (self.f_150 as u32).wrapping_sub(1) < 2 {
            self.slot75(obj, 3, cx);
        }
    }

    /// slot 15 FUN_002240b0: FUN_001f36e0; flying with a pending state -> FUN_00224258(+0x1a0); the linked object
    /// +0x16c further than +0x170 -> slot 74
    pub fn ee_sub_2240b0(&mut self, obj: usize, cx: &mut Cx) {
        cx.h.base(0x1f36e0, obj);
        if self.f_150 == 1 && self.f_158 != -1 {
            self.ee_sub_224258(obj, self.f_1a0, cx);
        }
        if self.f_16c != 0 {
            let d = len(sub(self.phys.pos, cx.h.ee_sub_1a62e8(self.f_16c)));
            if self.f_170 < d {
                self.ee_sub_224190(obj, cx);
            }
        }
    }

    /// slot 54 FUN_00224350(self, rec): FUN_001f4fb0 (empty); rec +0x98 = 0.25 (returned)
    pub fn ee_sub_224350(&self) -> f32 {
        0.25
    }

    /// slot 57 FUN_00224380: FUN_001f4f10; contacts are ignored the first 0.25 s of state 0. Per contact of this
    /// object: kind 8 or not stickable (FUN_00224638 / FUN_00224508) -> state 0 queues 4; else +0x1a0 = other,
    /// pending 2, +0x190 = normal * `S_2C3C60[second]`
    pub fn ee_sub_224380(&mut self, obj: usize, contacts: &[Contact57], cx: &mut Cx) {
        cx.h.base(0x1f4f10, obj);
        if self.f_150 == 0 && cx.t - self.f_154 < D_2C42E0 {
            return;
        }
        for c in contacts {
            let stick = c.kind != 8 && self.ee_sub_224638() && self.ee_sub_224508(c.other, cx.h);
            if stick {
                self.f_1a0 = c.other;
                self.f_158 = 2;
                let s = S_2C3C60[c.second as usize];
                self.f_190 = c.normal.map(|x| x * s);
            } else if self.f_150 == 0 && self.f_158 == -1 {
                self.f_158 = 4;
            }
        }
    }

    /// slot 60 FUN_002242e0(self, other): FUN_001f4fc8; resting / gone against a class-0x34 object -> max(r, 1)
    pub fn ee_sub_2242e0(&self, obj: usize, other: u32, h: &mut dyn ClassHost) -> i64 {
        // FUN_001f4fc8 (actor::c2caf88::ee_sub_1f4fc8 on this object's +0x148)
        let mut r = if self.f_148 & 0x400_0000 == 0 && h.vcall_ret(handle(obj), 500) == 0 { 0 } else { (h.vcall_ret(other, 500) != 0) as i64 };
        if (self.f_150 == 0 || self.f_150 == 4) && h.ee_sub_132b10(other, 0x34) != 0 {
            r = r.max(1);
        }
        r
    }

    /// slot 69 FUN_00224698: not resting / gone: ALO gravity (FUN_001f9b20); kinds 0x1a / 0x29 servo the
    /// orientation (FUN_001f5c30, 0x2c4300 / 0x2c4310) to upright (FUN_0011b938) or, with an aim object, to face
    /// it horizontally (FUN_0011b830)
    pub fn ee_sub_224698(&mut self, dt: f32, h: &mut dyn ClassHost) {
        if self.f_150 == 0 || self.f_150 == 4 {
            return;
        }
        FUN_001f9b20(&mut self.phys);
        if self.f_15c != 0x1a && self.f_15c != 0x29 {
            return;
        }
        let target = if self.f_160 == 0 {
            FUN_0011b938(&self.phys.rot)
        } else {
            let mut d = sub(h.ee_sub_1a62e8(self.f_160), self.phys.pos);
            d[2] = 0.0;
            let l = len(d);
            let dir = if l < 0.0001 { [1.0, 0.0, 0.0] } else { d.map(|x| x / l) };
            FUN_0011b7e0(dir)
        };
        FUN_001f5c30(&mut self.phys, &target, P_2C4300, P_2C4310, dt);
    }

    /// FUN_001f3980 physics part with this class's slot 69
    pub fn physics(&mut self, obj: usize, dt: f32, h: &mut dyn ClassHost) {
        h.base(0x1f3980, obj);
        let b = &mut self.phys;
        b.prev_v = b.v;
        b.prev_w = b.w;
        if matches!(b.kind, 1 | 2) {
            b.a = [0.0; 3];
            b.alpha = [0.0; 3];
            self.ee_sub_224698(dt, h);
        }
        FUN_0022de88(&mut self.phys, dt);
    }

    /// method 1110 FUN_00224e70(arg, self): FUN_001a58d0(self, 0, 1); FUN_00224bf8(arg, self, +0x174, +0x178,
    /// position) -> REPLACED by `ClassHost::explode_224bf8`
    pub fn ee_sub_224e70(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.ee_sub_1a58d0(obj, 0, 1);
        h.explode_224bf8(handle(obj), self.phys.pos, self.f_174, self.f_178);
    }

    /// rows `ROWS_101` (1109 / 1110 are run-time methods)
    pub fn feed(&mut self, props: &[crate::engine::world::props::Prop]) {
        use crate::engine::world::props::PropValue;
        use super::k4::word;
        for p in props {
            let PropValue::Scalar(b) = &p.value else { continue };
            let Some(&(_, off)) = ROWS_101.iter().find(|r| r.0 == p.field) else { continue };
            let w = word(b);
            match off {
                0x15c => self.f_15c = w as i32,
                0x164 => self.f_164 = w,
                0x168 => self.f_168 = w,
                0x16c => self.f_16c = w,
                0x170 => self.f_170 = f32::from_bits(w),
                0x174 => self.f_174 = f32::from_bits(w),
                0x178 => self.f_178 = w as i32,
                _ => {}
            }
        }
    }
}

fn m4(r: &M3) -> [[f32; 4]; 4] {
    [[r[0][0], r[0][1], r[0][2], 0.0], [r[1][0], r[1][1], r[1][2], 0.0], [r[2][0], r[2][1], r[2][2], 0.0], [0.0, 0.0, 0.0, 1.0]]
}

impl ClassSet {
    /// K15 class 101: construct + init (slot 6) + feed; xfLin at the rest world origin
    pub(super) fn build_k15_101(&mut self, w: &World, h: &mut dyn ClassHost) {
        let mut bw = crate::engine::gfx::attach::BoneWorlds::default();
        for (i, o) in w.objects.iter().enumerate() {
            if o.class != 101 && o.class != 102 {
                continue;
            }
            let src = o.clone_of.unwrap_or(i);
            let props = w.objects.get(src).and_then(crate::engine::game::fields::object_props);
            let mut c = C2c9810::default();
            if o.class == 102 {
                // K23: slot 6 FUN_002252e0, row 1113
                c.k23 = Some(Default::default());
                super::k23_102::ee_sub_2252e0(&mut c, i, h);
            } else {
                c.ee_sub_223f70(i, h);
            }
            if let Some(p) = props {
                c.feed(p);
                if let Some(e) = c.k23.as_mut() {
                    e.feed(p, i, h);
                }
            }
            // object references (resolved like the other object rows): +0x164 sound, +0x16c linked object
            if c.f_164 != 0 {
                c.f_164 = h.ee_sub_15e938(0x2002, c.f_164 as i32, handle(i));
            }
            if c.f_16c != 0 {
                c.f_16c = h.ee_sub_15e938(0x2002, c.f_16c as i32, handle(i));
            }
            c.phys.pos = bw.get(w, i, 0)[3];
            if c.k23.is_some() {
                self.k23c.push((i, Inst::K23C102(Box::new(c))));
            } else {
                self.k15.push((i, Inst::K15b(Box::new(c))));
            }
        }
    }
    /// K15 class-101 state at object index `obj`
    pub fn k15_101_get(&mut self, obj: usize) -> Option<&mut C2c9810> {
        self.k15.iter_mut().chain(self.k23c.iter_mut()).find(|x| x.0 == obj).and_then(|x| match &mut x.1 {
            Inst::K15b(c) | Inst::K23C102(c) => Some(&mut **c),
            _ => None,
        })
    }
    /// run `f` on the class-101 instance `obj` with a frame context
    pub fn k15_101_with<R>(&mut self, obj: usize, h: &mut dyn ClassHost, f: impl FnOnce(&mut C2c9810, &mut Cx) -> R) -> Option<R> {
        let (t, dt) = (self.g.t, self.g.dt);
        let c = self.k15_101_get(obj)?;
        Some(f(c, &mut Cx { t, dt, h }))
    }
    /// K15 class 101 (and K23 class 102) load-post: slot 13, slot 10
    pub(super) fn post_load_k15_101(&mut self, h: &mut dyn ClassHost) {
        for (i, x) in self.k15.iter_mut().chain(self.k23c.iter_mut()) {
            if let Inst::K15b(c) | Inst::K23C102(c) = x {
                c.ee_sub_223fe0(*i, h);
                c.ee_sub_224028(*i, h);
            }
        }
    }
    /// K15 class 101 per frame, for objects in the world (+0x28 bit 60; the template is not until cloned and
    /// thrown, or a state is set): slot 57 (host contacts), slot 15, the physics part (slot 69)
    pub(super) fn frame_k15_101(&mut self, h: &mut dyn ClassHost) {
        let (t, dt) = (self.g.t, self.g.dt);
        for (i, x) in self.k15.iter_mut().chain(self.k23c.iter_mut()) {
            if let Inst::K15b(c) | Inst::K23C102(c) = x {
                if c.f_150 == -1 && !h.in_world_28(handle(*i)) {
                    continue;
                }
                let mut cx = Cx { t, dt, h: &mut *h };
                let contacts = cx.h.so_contacts_57(*i);
                if !contacts.is_empty() {
                    c.ee_sub_224380(*i, &contacts, &mut cx);
                }
                c.slot15(*i, &mut cx);
                // K23: class 102 slot 70 for the held guard (REPLACED: the guard's +0x138 record walk)
                if c.k23.as_ref().is_some_and(|e| e.f_1b4 != 0) {
                    super::k23_102::ee_sub_2255c8(c, *i, dt, &mut *cx.h);
                }
                c.physics(*i, dt, &mut *cx.h);
            }
        }
    }
}
