//! WP-K10: bulk class port, next ranked classes of class_port.md: the player-usable interaction points 35 / 36
//! (character mask, use slot 50, sequence start), the class-22 volume 27, the class-18 child 19, the proximity
//! users 38 / 9 / 12 of the class-15 volume (FUN_00221268 / FUN_00221178), and the list members 21 / 40.
//! Classes 35 / 27 / 19 / 38 / 12 embed the FUN_00182c70 sub-block (`Sub182c70`). Handles = object index + 1.
use super::c2d9040::C2d9040;
use super::c2db6b8::C2db6b8;
use super::k4::{f32_of, word};
use super::k7::C2db518;
use super::*;
use crate::engine::game::fields::object_props;
use crate::engine::world::props::{ArrayItem, Prop, PropValue};

/// K10 world class id -> vtable (kept in `ClassSet::k10`).
pub const PORTED_K10: &[(ClassId, u32)] =
    &[(35, 0x2d1ee8), (36, 0x2d1d00), (27, 0x2d1b60), (19, 0x2d2978), (38, 0x2d2378), (21, 0x2db378), (40, 0x2d3200), (9, 0x2d2b18), (12, 0x2d2738)];

/// slot 5 (class descriptor getter): FUN_00113c40 (35), 113c50 (36), 113bc0 (27), 113b40 (19), 113c70 (38),
/// 113b60 (21), 113c90 (40), 113aa0 (9), 113ad0 (12)
pub fn slot5(class: ClassId) -> Option<u32> {
    Some(match class {
        35 => 0x2bada4,
        36 => 0x2badb8,
        27 => 0x2bad04,
        19 => 0x2bac64,
        38 => 0x2bade0,
        21 => 0x2bac8c,
        40 => 0x2bae08,
        9 => 0x2bab9c,
        12 => 0x2babd8,
        _ => return None,
    })
}

/// object lists the K10 slots 21 / 22 (or 10 / 11) remove from / append to (FUN_0011a2e8 / FUN_0011a220)
pub const L_3E1414: u32 = 0x3e1414; // classes 35 / 36 / 37 (slot 22 FUN_00183fb8, slot 21 FUN_00183ff0)
pub const L_3E1438: u32 = 0x3e1438; // class 27
pub const L_3E13B4: u32 = 0x3e13b4; // class 19
pub const L_3E1408: u32 = 0x3e1408; // class 38
pub const L_3E1474: u32 = 0x3e1474; // class 21
pub const L_3E1420: u32 = 0x3e1420; // class 40 (slots 10 / 11)
pub const L_3E142C: u32 = 0x3e142c; // class 9 (slots 10 / 11)
pub const L_3E13F0: u32 = 0x3e13f0; // class 12

/// DAT_002be900 / 2be904 / 2be8c8 / 2be8b4 / 2c40c0: +0x38 radius defaults / pads
const R_2BE900: f32 = 1000.0;
const R_2BE904: f32 = 1000.0;
const R_2BE8C8: f32 = 1250.0;
const R_2BE8B4: f32 = 1250.0;
const R_2C40C0: f32 = 1500.0;
/// +0x38 "not set" marker
const UNSET: f32 = 1e10;

pub(super) fn scalar(p: &Prop) -> Option<&[u8]> {
    match &p.value {
        PropValue::Scalar(b) => Some(b),
        _ => None,
    }
}
fn i16_of(b: &[u8]) -> i32 {
    match b {
        [a, c, ..] => i16::from_le_bytes([*a, *c]) as i32,
        _ => 0,
    }
}
/// argument words of a method-field call (flags 0x401 array)
pub(super) fn args(p: &Prop) -> Vec<u32> {
    match &p.value {
        PropValue::Array(items) => items
            .iter()
            .flat_map(|i| match i {
                ArrayItem::One(_, b) => vec![word(b)],
                ArrayItem::List(_, l) => l.iter().map(|b| word(b)).collect(),
            })
            .collect(),
        PropValue::Scalar(b) => vec![word(b)],
    }
}
pub(super) fn handle(obj: usize) -> u32 {
    obj as u32 + 1
}

/// FUN_00182c70 sub-block (fields base+0 .. base+7: +0 object, +4, +8, +0xc, +0x10, +0x14, +0x18 object
/// (setter), +0x1c). +0 is a class-15 proximity volume when set (slot 14 of 38 / 12: FUN_00221268).
#[derive(Clone, Debug, Default)]
pub struct Sub182c70 {
    pub f_00: u32,
    pub oid_00: Option<i32>,
    pub f_04: u8,
    pub f_08: f32,
    pub f_0c: u8,
    pub f_10: u32,
    pub f_14: i32,
    pub oid_18: Option<i32>,
    pub f_18: u32,
    pub f_1c: i32,
}
impl Sub182c70 {
    /// FUN_00182c70(sub): +0x14 = -1, +8 = FLT_MAX (DAT_002a5680), +0x1c = 7, FUN_001a6ca8(sub vt+0x34(), 3)
    pub fn ee_sub_182c70(&mut self, obj: usize, h: &mut dyn ClassHost) {
        self.f_14 = -1;
        self.f_08 = f32::MAX;
        self.f_1c = 7;
        h.ee_sub_182c70_tail(obj);
    }
    /// field rows `base` .. `base + 7` of the owning class
    pub fn feed(&mut self, p: &Prop, base: i16) {
        let Some(b) = scalar(p) else { return };
        match p.field - base {
            0 => self.oid_00 = Some(word(b) as i32),
            1 => self.f_04 = b.first().copied().unwrap_or(0),
            2 => self.f_08 = f32_of(b),
            3 => self.f_0c = b.first().copied().unwrap_or(0),
            4 => self.f_10 = word(b),
            5 => self.f_14 = i16_of(b),
            6 => self.oid_18 = Some(word(b) as i32),
            7 => self.f_1c = word(b) as i32,
            _ => {}
        }
    }
    /// load-time oid resolution (FUN_0013e590 / FUN_0015e938 kind 4)
    pub fn resolve(&mut self, obj: usize, h: &mut dyn ClassHost) {
        if let Some(o) = self.oid_00.take() {
            self.f_00 = h.ee_sub_15e938(4, o, handle(obj));
        }
        if let Some(o) = self.oid_18.take() {
            self.f_18 = h.ee_sub_15e938(4, o, handle(obj));
        }
    }
}

impl C2db6b8 {
    /// FUN_00221268(vol, s): forced state +0x13c = s, re-test (FUN_00221298) when it changes
    pub fn ee_sub_221268(&mut self, obj: usize, s: i32, g: &ClassGlobals, h: &mut dyn ClassHost) {
        if s != self.f_13c {
            self.f_13c = s;
            self.ee_sub_221298(obj, g, h);
        }
    }
}

/// C2d1ee8 (class 35, ALO child): player interaction point. +0xfc = mask of the characters that may use it
/// (bit 0 Sly 0x35, bit 1 Murray 0x37, bit 2 Bentley 0x36); slot 50 (use) records the user, raises its
/// effects / signals and starts the +0x114 sequence.
#[derive(Clone, Debug)]
pub struct C2d1ee8 {
    /// +0xe0 sub-block (fields 1077-1084)
    pub sub: Sub182c70,
    /// +0x38 radius (field 13)
    pub f_38: f32,
    /// +0xf0, +0xfc character mask (setters 1073 FUN_00184028 / 1074 FUN_00184040)
    pub f_f0: i32,
    pub f_fc: u32,
    /// +0x10c (field 1068; slot 50: the user), +0x110 (1069), +0x114 sequence (1070)
    pub f_10c: u32,
    pub oid_10c: Option<i32>,
    pub f_110: f32,
    pub f_114: u32,
    pub oid_114: Option<i32>,
    /// +0x118 count / +0x11c entries (method 1075 FUN_001840b8 -> FUN_0020fec0, max 4)
    pub l_118: Vec<[u32; 3]>,
    /// +0x15c count / +0x160 entries (method 1076 FUN_001840f8 -> FUN_00161f10, max 4)
    pub l_15c: Vec<[u32; 2]>,
    /// field 1073 object oids (setter argument, resolved at load-post)
    pub oids_1073: Vec<i32>,
}
impl Default for C2d1ee8 {
    fn default() -> Self {
        C2d1ee8 {
            sub: Sub182c70::default(),
            f_38: UNSET,
            f_f0: 0,
            f_fc: 0,
            f_10c: 0,
            oid_10c: None,
            f_110: 0.0,
            f_114: 0,
            oid_114: None,
            l_118: Vec::new(),
            l_15c: Vec::new(),
            oids_1073: Vec::new(),
        }
    }
}
impl C2d1ee8 {
    /// slot 6 FUN_00183f08: FUN_00231100, sub init, +0xf0 = 1, +0x110 = 200.0, +0xfc = 0
    pub fn ee_sub_183f08(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x231100, obj);
        self.sub.ee_sub_182c70(obj, h);
        self.f_f0 = 1;
        self.f_110 = 200.0;
        self.f_fc = 0;
    }
    pub fn feed(&mut self, props: &[Prop]) {
        for p in props {
            match p.field {
                13 => self.f_38 = scalar(p).map(f32_of).unwrap_or(self.f_38),
                1068 => self.oid_10c = scalar(p).map(|b| word(b) as i32),
                1069 => self.f_110 = scalar(p).map(f32_of).unwrap_or(self.f_110),
                1070 => self.oid_114 = scalar(p).map(|b| word(b) as i32),
                1073 => self.oids_1073.extend(scalar(p).map(|b| word(b) as i32)),
                1074 => self.ee_sub_184040(scalar(p).map(i16_of).unwrap_or(0)),
                1075 => {
                    let a = args(p);
                    self.ee_sub_1840b8([0, 1, 2].map(|i| a.get(i).copied().unwrap_or(0)));
                }
                1076 => {
                    let a = args(p);
                    self.ee_sub_1840f8([0, 1].map(|i| a.get(i).copied().unwrap_or(0)));
                }
                1077..=1084 => self.sub.feed(p, 1077),
                _ => {}
            }
        }
    }
    /// field 1073 setter FUN_00184028(this, o): +0xfc |= o+0x1000
    pub fn ee_sub_184028(&mut self, o: u32, h: &mut dyn ClassHost) {
        self.f_fc |= h.obj_i32(o, 0x1000) as u32;
    }
    /// field 1074 setter FUN_00184040(this, class): 0x34 -> mask 7, 0x35 |= 1, 0x36 |= 4, 0x37 |= 2
    pub fn ee_sub_184040(&mut self, c: i32) {
        match c {
            0x34 => self.f_fc = 7,
            0x35 => self.f_fc |= 1,
            0x36 => self.f_fc |= 4,
            0x37 => self.f_fc |= 2,
            _ => {}
        }
    }
    /// method 1075 FUN_001840b8(this, a, b, c) -> FUN_0020fec0(0x3dd490, +0x11c, 4, +0x118, a, b, c, 1)
    pub fn ee_sub_1840b8(&mut self, e: [u32; 3]) {
        if self.l_118.len() < 4 {
            self.l_118.push(e);
        }
    }
    /// method 1076 FUN_001840f8(this, a, b) -> FUN_00161f10(0x2deb30, +0x160, 4, +0x15c, a, b)
    pub fn ee_sub_1840f8(&mut self, e: [u32; 2]) {
        if self.l_15c.len() < 4 {
            self.l_15c.push(e);
        }
    }
    /// load-post oid resolution (FUN_0013e590 / setter arguments)
    pub fn resolve(&mut self, obj: usize, h: &mut dyn ClassHost) {
        self.sub.resolve(obj, h);
        if let Some(o) = self.oid_10c.take() {
            self.f_10c = h.ee_sub_15e938(4, o, handle(obj));
        }
        if let Some(o) = self.oid_114.take() {
            self.f_114 = h.ee_sub_15e938(4, o, handle(obj));
        }
        for oid in std::mem::take(&mut self.oids_1073) {
            let o = h.ee_sub_15e938(4, oid, handle(obj));
            if o != 0 {
                self.ee_sub_184028(o, h);
            }
        }
    }
    /// slot 14 FUN_00183f50: FUN_001a5c48; mask 0 -> 7; +0x38 unset -> 1000.0 (DAT_002be900)
    pub fn ee_sub_183f50(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1a5c48, obj);
        if self.f_fc == 0 {
            self.f_fc = 7;
        }
        if self.f_38 == UNSET {
            self.f_38 = R_2BE900;
        }
    }
    /// slot 50 FUN_00184130(this, user): +0x10c = user, vt+0x24(6, this), FUN_0020ff70(0x3dd490, +0x118,
    /// +0x11c, this, 1), FUN_00161f48(0x2deb30, +0x15c, +0x160, this), vt+0x1c(5), sequence +0x114 start
    /// (FUN_001dfd58(seq, user))
    pub fn ee_sub_184130(&mut self, obj: usize, user: u32, h: &mut dyn ClassHost) {
        self.f_10c = user;
        h.vcall_24(obj, 6, handle(obj));
        h.ee_sub_20ff70(obj, &self.l_118);
        h.ee_sub_161f48_list(obj, &self.l_15c);
        h.vcall_1c(obj, 5, 0);
        if self.f_114 != 0 {
            h.ee_sub_1dfd58(self.f_114, user);
        }
    }
    /// slot 22 FUN_00183fb8 (classes 35 / 36): FUN_001a6178, FUN_0011a220(0x3e1414, this)
    pub fn ee_sub_183fb8(&mut self, obj: usize, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        h.base(0x1a6178, obj);
        g.list(L_3E1414).push(handle(obj));
    }
    /// K31: slot 21 FUN_00183ff0 (classes 35 / 36 / 37): FUN_001a6150, FUN_0011a2e8(0x3e1414, this)
    pub fn ee_sub_183ff0(&mut self, obj: usize, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        h.base(0x1a6150, obj);
        g.list(L_3E1414).retain(|&x| x != handle(obj));
    }
    /// may `user` (class id 0x35 / 0x36 / 0x37) use it: the +0xfc mask bit
    pub fn allows(&self, user_class: i32) -> bool {
        let bit = match user_class {
            0x35 => 1,
            0x37 => 2,
            0x36 => 4,
            _ => 0,
        };
        self.f_fc & bit != 0
    }
}

/// C2d1d00 (class 36, child of 35): Bentley interaction point (+0x180, three object links resolved at
/// load-post into +0x188 / +0x190 / +0x198); listed in DAT_003e1414.
#[derive(Clone, Debug, Default)]
pub struct C2d1d00 {
    pub base: C2d1ee8,
    /// +0x180 (field 1085), +0x184 / +0x18c / +0x194 oids (1086-1088) -> +0x188 / +0x190 / +0x198, +0x19c (1089)
    pub f_180: i32,
    pub oid: [i32; 3],
    pub obj: [u32; 3],
    pub f_19c: i32,
}
impl C2d1d00 {
    /// slot 6 FUN_00184268: FUN_00183f08, +0xfc = 4, +0xf0 = -1, +0x184 / +0x18c / +0x194 / +0x19c = -1
    pub fn ee_sub_184268(&mut self, obj: usize, h: &mut dyn ClassHost) {
        self.base.ee_sub_183f08(obj, h);
        self.base.f_fc = 4;
        self.base.f_f0 = -1;
        self.oid = [-1; 3];
        self.f_19c = -1;
    }
    pub fn feed(&mut self, props: &[Prop]) {
        self.base.feed(props);
        for p in props {
            let Some(b) = scalar(p) else { continue };
            match p.field {
                1085 => self.f_180 = word(b) as i32,
                1086..=1088 => self.oid[(p.field - 1086) as usize] = word(b) as i32,
                1089 => self.f_19c = word(b) as i32,
                _ => {}
            }
        }
    }
    /// slot 14 FUN_001842b0: FUN_00183f50, then FUN_0015e938(0x3dd490, 4, oid, this) for the three links
    pub fn ee_sub_1842b0(&mut self, obj: usize, h: &mut dyn ClassHost) {
        self.base.ee_sub_183f50(obj, h);
        for k in 0..3 {
            self.obj[k] = h.ee_sub_15e938(4, self.oid[k], handle(obj));
        }
    }
    /// slot 22 FUN_00183fb8: FUN_001a6178, FUN_0011a220(0x3e1414, this)
    pub fn ee_sub_183fb8(&mut self, obj: usize, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        self.base.ee_sub_183fb8(obj, g, h);
    }
    /// slot 50 FUN_00184330: FUN_00184130, user is-a 0x36 (Bentley) -> FUN_001b7000(user, this)
    pub fn ee_sub_184330(&mut self, obj: usize, user: u32, h: &mut dyn ClassHost) {
        self.base.ee_sub_184130(obj, user, h);
        if h.ee_sub_132b10(user, 0x36) != 0 {
            h.ee_sub_1b7000(user, handle(obj));
        }
    }
}

/// C2d1b60 (class 27, class-22 volume child): +0x130 sub-block, +0x15c (field 1182), +0x38 radius from the
/// bounding-sphere centre; listed in DAT_003e1438.
#[derive(Clone, Debug)]
pub struct C2d1b60 {
    pub sub: Sub182c70,
    pub f_38: f32,
    pub f_15c: i32,
}
impl Default for C2d1b60 {
    fn default() -> Self {
        C2d1b60 { sub: Sub182c70::default(), f_38: UNSET, f_15c: 0 }
    }
}
impl C2d1b60 {
    /// slot 6 FUN_001843c0: FUN_00231100, FUN_00182c70(+0x130)
    pub fn ee_sub_1843c0(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x231100, obj);
        self.sub.ee_sub_182c70(obj, h);
    }
    pub fn feed(&mut self, props: &[Prop]) {
        for p in props {
            match p.field {
                13 => self.f_38 = scalar(p).map(f32_of).unwrap_or(self.f_38),
                1182 => self.f_15c = scalar(p).map(|b| word(b) as i32).unwrap_or(0),
                1183..=1190 => self.sub.feed(p, 1183),
                _ => {}
            }
        }
    }
    /// slot 14 FUN_001843f0: FUN_001a5c48; +0x38 unset -> |FUN_001a62e8(this) - FUN_001a6f00(this, 0, +0x110)|
    /// + 1000.0 (DAT_002be904)
    pub fn ee_sub_1843f0(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1a5c48, obj);
        if self.f_38 == UNSET {
            let p = h.ee_sub_1a62e8(handle(obj));
            let c = h.vol_centre_110(obj);
            let c = h.ee_sub_1a6f00_to_world(obj, c);
            let d = (0..3).map(|i| (p[i] - c[i]) * (p[i] - c[i])).sum::<f32>().sqrt();
            self.f_38 = d + R_2BE904;
        }
    }
    /// slot 21 FUN_001844d8 / slot 22 FUN_001844a0 (DAT_003e1438)
    pub fn ee_sub_1844d8(&mut self, obj: usize, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        h.base(0x1a6150, obj);
        g.list(L_3E1438).retain(|&x| x != handle(obj));
    }
    pub fn ee_sub_1844a0(&mut self, obj: usize, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        h.base(0x1a6178, obj);
        g.list(L_3E1438).push(handle(obj));
    }
}

/// C2d2978 (class 19, child of the K7 class 18 C2db518): +0x100 sub-block, fields 1011-1016; listed in
/// DAT_003e13b4.
#[derive(Clone, Debug)]
pub struct C2d2978 {
    pub base: C2db518,
    pub sub: Sub182c70,
    pub f_38: f32,
    /// +0xe0 (8 bytes, kept across the clone)
    pub f_e0: u64,
    /// +0x124 (field 1011), +0x128 (1012), +0x12c / +0x130 (1013 / 1014), +0x134 (1015), +0x138 object (1016)
    pub f_124: u32,
    pub f_128: f32,
    pub f_12c: i32,
    pub f_130: i32,
    pub f_134: f32,
    pub f_138: u32,
    pub oid_138: Option<i32>,
}
impl Default for C2d2978 {
    fn default() -> Self {
        C2d2978 {
            base: C2db518::default(),
            sub: Sub182c70::default(),
            f_38: UNSET,
            f_e0: 0,
            f_124: 0,
            f_128: 0.0,
            f_12c: 0,
            f_130: 0,
            f_134: 0.0,
            f_138: 0,
            oid_138: None,
        }
    }
}
impl C2d2978 {
    /// slot 6 FUN_00184f50: FUN_0021d278 (class 18), FUN_00182c70(+0x100), +0xe8 = 1, +0x130 = 9, +0x12c = 8
    pub fn ee_sub_184f50(&mut self, obj: usize, h: &mut dyn ClassHost) {
        self.base.ee_sub_21d278(obj, h);
        self.sub.ee_sub_182c70(obj, h);
        self.base.f_e8 = 1;
        self.f_130 = 9;
        self.f_12c = 8;
    }
    pub fn feed(&mut self, props: &[Prop]) {
        self.base.feed(props);
        for p in props {
            if let 1017..=1024 = p.field {
                self.sub.feed(p, 1017);
                continue;
            }
            let Some(b) = scalar(p) else { continue };
            match p.field {
                13 => self.f_38 = f32_of(b),
                1011 => self.f_124 = word(b),
                1012 => self.f_128 = f32_of(b),
                1013 => self.f_12c = i16_of(b),
                1014 => self.f_130 = i16_of(b),
                1015 => self.f_134 = f32_of(b),
                1016 => self.oid_138 = Some(word(b) as i32),
                _ => {}
            }
        }
    }
    pub fn resolve(&mut self, obj: usize, h: &mut dyn ClassHost) {
        self.sub.resolve(obj, h);
        if let Some(o) = self.oid_138.take() {
            self.f_138 = h.ee_sub_15e938(4, o, handle(obj));
        }
    }
    /// slot 14 FUN_0021d2c8: FUN_001a5c48; +0x38 unset -> +0xec + 1500.0 (DAT_002c40c0)
    pub fn ee_sub_21d2c8(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1a5c48, obj);
        if self.f_38 == UNSET {
            self.f_38 = self.base.f_ec + R_2C40C0;
        }
    }
    /// slot 21 FUN_0021d320 / slot 22 FUN_0021d358 (DAT_003e13b4)
    pub fn ee_sub_21d320(&mut self, obj: usize, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        h.base(0x1a6150, obj);
        g.list(L_3E13B4).retain(|&x| x != handle(obj));
    }
    pub fn ee_sub_21d358(&mut self, obj: usize, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        h.base(0x1a6178, obj);
        g.list(L_3E13B4).push(handle(obj));
        g.l3e13b4_e8.insert(handle(obj), self.base.f_e8 as u32);
        g.l3e13b4_f4.insert(handle(obj), self.base.f_f4);
    }
    /// slot 40 FUN_0021d390: FUN_002311b0 keeping +0xe0
    pub fn ee_sub_21d390(&mut self, obj: usize, h: &mut dyn ClassHost) {
        let v = self.f_e0;
        h.base(0x2311b0, obj);
        self.f_e0 = v;
    }
}

/// C2d2378 (class 38, ALO child): +0xe0 sub-block whose +0 class-15 volume is forced to state 0 at load-post;
/// +0x10c (field 816); listed in DAT_003e1408.
#[derive(Clone, Debug)]
pub struct C2d2378 {
    pub sub: Sub182c70,
    pub f_38: f32,
    pub f_10c: u32,
    /// +0x104 (8 bytes, kept across the clone)
    pub f_104: u64,
}
impl Default for C2d2378 {
    fn default() -> Self {
        C2d2378 { sub: Sub182c70::default(), f_38: UNSET, f_10c: 0, f_104: 0 }
    }
}
impl C2d2378 {
    /// slot 6 FUN_001836c8: FUN_00231100, FUN_00182c70(+0xe0)
    pub fn ee_sub_1836c8(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x231100, obj);
        self.sub.ee_sub_182c70(obj, h);
    }
    pub fn feed(&mut self, props: &[Prop]) {
        for p in props {
            match p.field {
                13 => self.f_38 = scalar(p).map(f32_of).unwrap_or(self.f_38),
                816 => self.f_10c = scalar(p).map(word).unwrap_or(0),
                818..=825 => self.sub.feed(p, 818),
                _ => {}
            }
        }
    }
    /// slot 14 FUN_00183768: FUN_001a5c48; +0xe0 volume -> FUN_00221268(vol, 0) (done by the caller, which
    /// owns the class-15 instances); +0x38 unset -> 1250.0 (DAT_002be8c8). Returns the volume handle.
    pub fn ee_sub_183768(&mut self, obj: usize, h: &mut dyn ClassHost) -> u32 {
        h.base(0x1a5c48, obj);
        if self.f_38 == UNSET {
            self.f_38 = R_2BE8C8;
        }
        self.sub.f_00
    }
    /// slot 21 FUN_00183730 / slot 22 FUN_001836f8 (DAT_003e1408)
    pub fn ee_sub_183730(&mut self, obj: usize, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        h.base(0x1a6150, obj);
        g.list(L_3E1408).retain(|&x| x != handle(obj));
    }
    pub fn ee_sub_1836f8(&mut self, obj: usize, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        h.base(0x1a6178, obj);
        g.list(L_3E1408).push(handle(obj));
    }
    /// slot 40 FUN_001837d0: FUN_002311b0 keeping +0x104
    pub fn ee_sub_1837d0(&mut self, obj: usize, h: &mut dyn ClassHost) {
        let v = self.f_104;
        h.base(0x2311b0, obj);
        self.f_104 = v;
    }
}

/// C2db378 (class 21, ALO child): +0xe8 / +0xec..+0xf0 range / +0xf4; listed in DAT_003e1474.
#[derive(Clone, Debug, Default)]
pub struct C2db378 {
    /// +0xe8 (field 220), +0xec / +0xf0 (219), +0xf4 (221)
    pub f_e8: f32,
    pub f_ec: [f32; 2],
    pub f_f4: u32,
    /// +0xe0 (8 bytes, kept across the clone)
    pub f_e0: u64,
}
impl C2db378 {
    /// slot 6 FUN_0021d3d8: FUN_00231100, +0xf4 = 1, +0xe8 = 2000.0, +0xec = -800.0, +0xf0 = 800.0
    pub fn ee_sub_21d3d8(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x231100, obj);
        self.f_f4 = 1;
        self.f_e8 = 2000.0;
        self.f_ec = [-800.0, 800.0];
    }
    pub fn feed(&mut self, props: &[Prop]) {
        for p in props {
            let Some(b) = scalar(p) else { continue };
            match p.field {
                219 => self.f_ec = [f32_of(b), b.get(4..8).map(f32_of).unwrap_or(0.0)],
                220 => self.f_e8 = f32_of(b),
                221 => self.f_f4 = word(b),
                _ => {}
            }
        }
    }
    /// slot 21 FUN_0021d428 / slot 22 FUN_0021d460 (DAT_003e1474)
    pub fn ee_sub_21d428(&mut self, obj: usize, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        h.base(0x1a6150, obj);
        g.list(L_3E1474).retain(|&x| x != handle(obj));
    }
    pub fn ee_sub_21d460(&mut self, obj: usize, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        h.base(0x1a6178, obj);
        g.list(L_3E1474).push(handle(obj));
    }
    /// slot 40 FUN_0021d498: FUN_002311b0 keeping +0xe0
    pub fn ee_sub_21d498(&mut self, obj: usize, h: &mut dyn ClassHost) {
        let v = self.f_e0;
        h.base(0x2311b0, obj);
        self.f_e0 = v;
    }
}

/// C2d3200 (class 40, ALO child): +0xe8 (field 870); listed in DAT_003e1420 by slots 10 / 11.
#[derive(Clone, Debug, Default)]
pub struct C2d3200 {
    pub f_e8: f32,
    pub f_e0: u64,
}
impl C2d3200 {
    /// slot 6 FUN_0019c9b8: FUN_00231100, +0xe8 = 100.0
    pub fn ee_sub_19c9b8(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x231100, obj);
        self.f_e8 = 100.0;
    }
    pub fn feed(&mut self, props: &[Prop]) {
        for p in props {
            if let (870, Some(b)) = (p.field, scalar(p)) {
                self.f_e8 = f32_of(b);
            }
        }
    }
    /// slot 10 FUN_0019c9e8 (FUN_001a5800, append DAT_003e1420) / slot 11 FUN_0019ca20 (FUN_001a5848, remove)
    pub fn ee_sub_19c9e8(&mut self, obj: usize, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        h.base(0x1a5800, obj);
        g.list(L_3E1420).push(handle(obj));
    }
    pub fn ee_sub_19ca20(&mut self, obj: usize, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        h.base(0x1a5848, obj);
        g.list(L_3E1420).retain(|&x| x != handle(obj));
    }
    /// slot 40 FUN_0019ca58: FUN_002311b0 keeping +0xe0
    pub fn ee_sub_19ca58(&mut self, obj: usize, h: &mut dyn ClassHost) {
        let v = self.f_e0;
        h.base(0x2311b0, obj);
        self.f_e0 = v;
    }
}

/// C2d2b18 (class 9, child of class 8; load FUN_00185150 already in world): +0x110 class-15 volume put in state
/// 0 at load-post (FUN_00221178); listed in DAT_003e142c by slots 10 / 11.
#[derive(Clone, Debug, Default)]
pub struct C2d2b18 {
    /// +0x108 (field 871), +0x10c (872), +0x110 volume (873)
    pub f_108: f32,
    pub f_10c: u8,
    pub f_110: u32,
    pub oid_110: Option<i32>,
    /// +0x100 (8 bytes, kept across the clone)
    pub f_100: u64,
}
/// class-8 slot 6 FUN_00185130: FUN_00231100
pub fn ee_sub_185130(obj: usize, h: &mut dyn ClassHost) {
    h.base(0x231100, obj);
}
impl C2d2b18 {
    /// slot 6 FUN_00185bf8: FUN_00185130, +0x108 = FLT_MAX (DAT_002a5690)
    pub fn ee_sub_185bf8(&mut self, obj: usize, h: &mut dyn ClassHost) {
        ee_sub_185130(obj, h);
        self.f_108 = f32::MAX;
    }
    pub fn feed(&mut self, props: &[Prop]) {
        for p in props {
            let Some(b) = scalar(p) else { continue };
            match p.field {
                871 => self.f_108 = f32_of(b),
                872 => self.f_10c = b.first().copied().unwrap_or(0),
                873 => self.oid_110 = Some(word(b) as i32),
                _ => {}
            }
        }
    }
    pub fn resolve(&mut self, obj: usize, h: &mut dyn ClassHost) {
        if let Some(o) = self.oid_110.take() {
            self.f_110 = h.ee_sub_15e938(4, o, handle(obj));
        }
    }
    /// slot 10 FUN_00185c28 (FUN_001a5800, append DAT_003e142c) / slot 11 FUN_00185c60 (FUN_001a5848, remove)
    pub fn ee_sub_185c28(&mut self, obj: usize, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        h.base(0x1a5800, obj);
        g.list(L_3E142C).push(handle(obj));
    }
    pub fn ee_sub_185c60(&mut self, obj: usize, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        h.base(0x1a5848, obj);
        g.list(L_3E142C).retain(|&x| x != handle(obj));
    }
    /// slot 14 FUN_00185ce0: FUN_001a5c48; +0x110 volume -> FUN_00221178(vol, 0) (done by the caller). Returns
    /// the volume handle.
    pub fn ee_sub_185ce0(&mut self, obj: usize, h: &mut dyn ClassHost) -> u32 {
        h.base(0x1a5c48, obj);
        self.f_110
    }
    /// slot 40 FUN_00185c98: FUN_002311b0 keeping +0x100
    pub fn ee_sub_185c98(&mut self, obj: usize, h: &mut dyn ClassHost) {
        let v = self.f_100;
        h.base(0x2311b0, obj);
        self.f_100 = v;
    }
}

/// C2d2738 (class 12, child of class 10 C2d9040): +0x190 sub-block (volume forced to state 0 at load-post),
/// +0x1bc (field 806); listed in DAT_003e13f0. Inherited class-10 slot 16 runs on `base`.
#[derive(Clone, Debug)]
pub struct C2d2738 {
    pub base: C2d9040,
    pub sub: Sub182c70,
    pub f_38: f32,
    pub f_1bc: u32,
    /// +0x1b4 (8 bytes, kept across the clone)
    pub f_1b4: u64,
}
impl Default for C2d2738 {
    fn default() -> Self {
        C2d2738 { base: C2d9040::default(), sub: Sub182c70::default(), f_38: UNSET, f_1bc: 0, f_1b4: 0 }
    }
}
impl C2d2738 {
    /// slot 6 FUN_00182e80: FUN_001e5d80 (class 10), FUN_00182c70(+0x190), +0x1bc = 3
    pub fn ee_sub_182e80(&mut self, obj: usize, h: &mut dyn ClassHost) {
        self.base.ee_sub_1e5d80(obj, h);
        self.sub.ee_sub_182c70(obj, h);
        self.f_1bc = 3;
    }
    pub fn feed(&mut self, props: &[Prop]) {
        for p in props {
            match p.field {
                13 => self.f_38 = scalar(p).map(f32_of).unwrap_or(self.f_38),
                806 => self.f_1bc = scalar(p).map(word).unwrap_or(0),
                808..=815 => self.sub.feed(p, 808),
                _ => {}
            }
        }
    }
    /// slot 14 FUN_00182f28: FUN_001a5c48; +0x190 volume -> FUN_00221268(vol, 0) (done by the caller); +0x38
    /// unset -> FUN_001e7d90(this) + 1250.0 (DAT_002be8b4). Returns the volume handle.
    pub fn ee_sub_182f28(&mut self, obj: usize, h: &mut dyn ClassHost) -> u32 {
        h.base(0x1a5c48, obj);
        if self.f_38 == UNSET {
            self.f_38 = h.ee_sub_1e7d90(obj) + R_2BE8B4;
        }
        self.sub.f_00
    }
    /// slot 21 FUN_00182ef0 / slot 22 FUN_00182eb8 (DAT_003e13f0)
    pub fn ee_sub_182ef0(&mut self, obj: usize, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        h.base(0x1a6150, obj);
        g.list(L_3E13F0).retain(|&x| x != handle(obj));
    }
    pub fn ee_sub_182eb8(&mut self, obj: usize, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        h.base(0x1a6178, obj);
        g.list(L_3E13F0).push(handle(obj));
    }
    /// slot 40 FUN_00182fa0: FUN_001e5db8 (class 10) keeping +0x1b4
    pub fn ee_sub_182fa0(&mut self, obj: usize, h: &mut dyn ClassHost) {
        let v = self.f_1b4;
        self.base.ee_sub_1e5db8(obj, h);
        self.f_1b4 = v;
    }
}

/// One K10 instance.
#[derive(Clone, Debug)]
pub enum K10 {
    C2d1ee8(C2d1ee8),
    C2d1d00(C2d1d00),
    C2d1b60(C2d1b60),
    C2d2978(C2d2978),
    C2d2378(C2d2378),
    C2db378(C2db378),
    C2d3200(C2d3200),
    C2d2b18(C2d2b18),
    C2d2738(C2d2738),
}

impl ClassGlobals {
    /// K10 object list at `addr` (FUN_0011a220 / FUN_0011a2e8 targets)
    pub fn list(&mut self, addr: u32) -> &mut Vec<u32> {
        self.k10_lists.entry(addr).or_default()
    }
}

impl ClassSet {
    /// K10: construct + init (slot 6) + feed the `PORTED_K10` classes; slot 40 for clones.
    pub(super) fn build_k10(&mut self, w: &World, h: &mut dyn ClassHost) {
        for (i, o) in w.objects.iter().enumerate() {
            if !PORTED_K10.iter().any(|x| x.0 == o.class) {
                continue;
            }
            let props = w.objects.get(o.clone_of.unwrap_or(i)).and_then(object_props).unwrap_or(&[]);
            let clone = o.clone_of.is_some();
            let k = match o.class {
                35 => {
                    let mut c = C2d1ee8::default();
                    c.ee_sub_183f08(i, h);
                    c.feed(props);
                    K10::C2d1ee8(c)
                }
                36 => {
                    let mut c = C2d1d00::default();
                    c.ee_sub_184268(i, h);
                    c.feed(props);
                    K10::C2d1d00(c)
                }
                27 => {
                    let mut c = C2d1b60::default();
                    c.ee_sub_1843c0(i, h);
                    c.feed(props);
                    K10::C2d1b60(c)
                }
                19 => {
                    let mut c = C2d2978::default();
                    c.ee_sub_184f50(i, h);
                    c.feed(props);
                    if clone {
                        c.ee_sub_21d390(i, h);
                    }
                    K10::C2d2978(c)
                }
                38 => {
                    let mut c = C2d2378::default();
                    c.ee_sub_1836c8(i, h);
                    c.feed(props);
                    if clone {
                        c.ee_sub_1837d0(i, h);
                    }
                    K10::C2d2378(c)
                }
                21 => {
                    let mut c = C2db378::default();
                    c.ee_sub_21d3d8(i, h);
                    c.feed(props);
                    if clone {
                        c.ee_sub_21d498(i, h);
                    }
                    K10::C2db378(c)
                }
                40 => {
                    let mut c = C2d3200::default();
                    c.ee_sub_19c9b8(i, h);
                    c.feed(props);
                    if clone {
                        c.ee_sub_19ca58(i, h);
                    }
                    K10::C2d3200(c)
                }
                9 => {
                    let mut c = C2d2b18::default();
                    c.ee_sub_185bf8(i, h);
                    c.feed(props);
                    if clone {
                        c.ee_sub_185c98(i, h);
                    }
                    K10::C2d2b18(c)
                }
                _ => {
                    let mut c = C2d2738::default();
                    c.ee_sub_182e80(i, h);
                    c.feed(props);
                    if clone {
                        c.ee_sub_182fa0(i, h);
                    }
                    K10::C2d2738(c)
                }
            };
            self.k10.push((i, Inst::K10(Box::new(k))));
        }
    }

    /// class-15 volume of handle `vol` (K4 set)
    fn c15(k4: &mut [(usize, Inst)], vol: u32) -> Option<(usize, &mut C2db6b8)> {
        k4.iter_mut().find_map(|(i, x)| match x {
            Inst::C2db6b8(c) if *i as u32 + 1 == vol => Some((*i, c)),
            _ => None,
        })
    }

    /// K10 load-post: oid resolution, slot 14, then entry into the world lists (slot 22 / slot 10, as K4).
    pub(super) fn post_load_k10(&mut self, h: &mut dyn ClassHost) {
        for (i, x) in &mut self.k10 {
            let Inst::K10(k) = x else { continue };
            let i = *i;
            let g = &mut self.g;
            // (volume, forced state 0 via FUN_00221268) / (volume, FUN_00221178(vol, 0))
            let mut vol_268 = 0;
            let mut vol_178 = 0;
            match k.as_mut() {
                K10::C2d1ee8(c) => {
                    c.resolve(i, h);
                    c.ee_sub_183f50(i, h);
                    // K31: class 35 slot 22 is FUN_00183fb8 too (vtable 0x2d1ee8 +0xb8)
                    c.ee_sub_183fb8(i, g, h);
                }
                K10::C2d1d00(c) => {
                    g.pt_19c.insert(i as u32 + 1, c.f_19c);
                    c.base.resolve(i, h);
                    c.ee_sub_1842b0(i, h);
                    c.ee_sub_183fb8(i, g, h);
                }
                K10::C2d1b60(c) => {
                    c.sub.resolve(i, h);
                    c.ee_sub_1843f0(i, h);
                    c.ee_sub_1844a0(i, g, h);
                }
                K10::C2d2978(c) => {
                    c.resolve(i, h);
                    c.ee_sub_21d2c8(i, h);
                    c.ee_sub_21d358(i, g, h);
                }
                K10::C2d2378(c) => {
                    c.sub.resolve(i, h);
                    vol_268 = c.ee_sub_183768(i, h);
                    c.ee_sub_1836f8(i, g, h);
                }
                K10::C2db378(c) => {
                    h.base(0x1a5c48, i);
                    c.ee_sub_21d460(i, g, h);
                }
                K10::C2d3200(c) => {
                    h.base(0x1a5c48, i);
                    c.ee_sub_19c9e8(i, g, h);
                }
                K10::C2d2b18(c) => {
                    c.resolve(i, h);
                    vol_178 = c.ee_sub_185ce0(i, h);
                    c.ee_sub_185c28(i, g, h);
                }
                K10::C2d2738(c) => {
                    c.sub.resolve(i, h);
                    vol_268 = c.ee_sub_182f28(i, h);
                    c.ee_sub_182eb8(i, g, h);
                }
            }
            if vol_268 != 0 {
                if let Some((vi, v)) = Self::c15(&mut self.k4, vol_268) {
                    v.ee_sub_221268(vi, 0, g, h);
                }
            }
            if vol_178 != 0 {
                if let Some((vi, v)) = Self::c15(&mut self.k4, vol_178) {
                    v.ee_sub_221178(vi, 0, g, h);
                }
            }
        }
    }
    /// K10 per frame: class 12 inherited class-10 slot 16 (FUN_001e6340)
    pub(super) fn frame_k10(&mut self, h: &mut dyn ClassHost) {
        for (i, x) in &mut self.k10 {
            // FEEL R33: slot 15 runs from the world update list only (LO +0x28 bit 60)
            if h.lo_removed(*i) {
                continue;
            }
            if let Inst::K10(k) = x {
                if let K10::C2d2738(c) = k.as_mut() {
                    c.base.ee_sub_1e6340(*i, h);
                }
            }
        }
    }
    /// slot 50 (use) of a class-35 / 36 object by `user` (handle) of class `user_class` (0x35 / 0x36 / 0x37):
    /// refused (false) when the +0xfc mask excludes the character.
    pub fn k10_use(&mut self, obj: usize, user: u32, user_class: i32, h: &mut dyn ClassHost) -> bool {
        let Some(Inst::K10(k)) = self.k10.iter_mut().find(|x| x.0 == obj).map(|x| &mut x.1) else { return false };
        match k.as_mut() {
            K10::C2d1ee8(c) if c.allows(user_class) => c.ee_sub_184130(obj, user, h),
            K10::C2d1d00(c) if c.base.allows(user_class) => c.ee_sub_184330(obj, user, h),
            _ => return false,
        }
        true
    }
}
