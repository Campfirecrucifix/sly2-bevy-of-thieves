//! C2da840 (world class 120): LO bound to its nearest class-0x38 ancestor (+0xac), with an animation
//! binding sub-object at +0xb0 (FUN_002090d0 / 209130 / 209590, unported -> hooks).
//! Object handles in hooks are index + 1 (0 = null).
use super::ClassHost;
use crate::engine::world::props::{ArrayItem, Prop, PropValue};

/// DAT_002a88c8
const DAT_002A88C8: u32 = 0x3f800000;

#[derive(Clone, Debug)]
pub struct C2da840 {
    /// +0x50..+0x6c: fields 353 (+0x50, 2 floats), 354 (+0x58), 355 (+0x60), 357 (+0x68), 360 (+0x6c)
    pub f_50: [f32; 2],
    pub f_58: [f32; 2],
    pub f_60: [f32; 2],
    pub f_68: f32,
    pub f_6c: u32,
    /// +0x70 / +0x74 / +0x78 (fields 359 / 361 / 358)
    pub f_70: u32,
    pub f_74: u32,
    pub f_78: u32,
    /// +0x7c / +0x80 / +0x84 (method 362, FUN_0020af28)
    pub f_7c: i32,
    pub f_80: i32,
    pub f_84: i32,
    /// +0x90..+0x9c: FUN_001300b8 result (row 3 of its matrix)
    pub f_90: [u32; 4],
    /// +0xa0 / +0xa4: FUN_001f4c58 children of +0xac
    pub f_a0: u32,
    pub f_a4: u32,
    /// +0xa8 (field 363)
    pub f_a8: u32,
    /// +0xac: class-0x38 ancestor (handle, 0 = none)
    pub f_ac: u32,
    /// +0xb4 / +0xbc (method 364, FUN_0020af88)
    pub f_b4: u32,
    pub f_bc: u32,
    /// +0xc0 = obj + 0xb0 unless the oid is 0x2cc
    pub f_c0_self: bool,
    /// inputs of slot 14 resolved from the world at build time: +0x18 oid, class-0x38 owner-chain ancestor
    pub oid: i32,
    pub anc: u32,
}

impl Default for C2da840 {
    fn default() -> Self {
        C2da840 {
            f_50: [0.0, 300.0],
            f_58: [-5000.0, 5000.0],
            f_60: [0.2, 0.5],
            f_68: f32::from_bits(0x3e32b8c4),
            f_6c: DAT_002A88C8,
            f_70: 0,
            f_74: 0,
            f_78: 0,
            f_7c: -1,
            f_80: -1,
            f_84: -1,
            f_90: [0; 4],
            f_a0: 0,
            f_a4: 0,
            f_a8: 1,
            f_ac: 0,
            f_b4: 0,
            f_bc: 0,
            f_c0_self: false,
            oid: 0,
            anc: 0,
        }
    }
}

/// slot 5: FUN_00114190
pub fn ee_sub_114190() -> u32 {
    0x2bb448
}

fn w(b: &[u8], k: usize) -> u32 {
    b.get(k * 4..k * 4 + 4).map(|x| u32::from_le_bytes([x[0], x[1], x[2], x[3]])).unwrap_or_else(|| b.first().copied().unwrap_or(0) as u32)
}
fn f(b: &[u8], k: usize) -> f32 {
    f32::from_bits(w(b, k))
}
fn args(v: &PropValue) -> Vec<u32> {
    match v {
        PropValue::Array(items) => items
            .iter()
            .map(|it| match it {
                ArrayItem::One(_, b) => w(b, 0),
                ArrayItem::List(_, v) => v.first().map(|b| w(b, 0)).unwrap_or(0),
            })
            .collect(),
        PropValue::Scalar(b) => b.chunks(4).map(|c| w(c, 0)).collect(),
    }
}

impl C2da840 {
    /// slot 6: FUN_0020ad38 (defaults in `Default`)
    pub fn ee_sub_20ad38(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1a5270, obj);
        *self = C2da840 { oid: self.oid, anc: self.anc, ..C2da840::default() };
        h.ee_sub_2090d0(obj, self.f_ac);
    }

    /// Load (FUN_001a5c28 props): fields 353-361, 363 and methods 362 / 364 / 365.
    pub fn feed(&mut self, obj: usize, props: &[Prop], h: &mut dyn ClassHost) {
        for p in props {
            if let PropValue::Scalar(b) = &p.value {
                match p.field {
                    353 => self.f_50 = [f(b, 0), f(b, 1)],
                    354 => self.f_58 = [f(b, 0), f(b, 1)],
                    355 => self.f_60 = [f(b, 0), f(b, 1)],
                    // K11: field 356 setter FUN_0020af08 (degrees)
                    356 => self.f_68 = f(b, 0) * 0.017453292,
                    357 => self.f_68 = f(b, 0),
                    358 => self.f_78 = w(b, 0),
                    359 => self.f_70 = w(b, 0),
                    360 => self.f_6c = w(b, 0),
                    361 => self.f_74 = w(b, 0),
                    363 => self.f_a8 = w(b, 0),
                    _ => {}
                }
            }
            let a = args(&p.value);
            let a = |k: usize| a.get(k).copied().unwrap_or(0);
            match p.field {
                362 => self.ee_sub_20af28(a(0) as i32, a(1) as i32, a(2) as i32),
                364 => self.ee_sub_20af88(obj, a(0), a(1), h),
                365 => {
                    h.ee_sub_20aff0(obj, [a(0), a(1), a(2), a(3), a(4)]);
                    h.ee_sub_209130(obj, self.f_ac); // FUN_0020b060
                }
                _ => {}
            }
        }
    }

    /// FUN_0020af28 (method 362)
    pub fn ee_sub_20af28(&mut self, a: i32, b: i32, c: i32) {
        self.f_84 = c;
        self.f_7c = a;
        self.f_80 = b;
    }

    /// FUN_0020af88 (method 364)
    pub fn ee_sub_20af88(&mut self, obj: usize, a: u32, b: u32, h: &mut dyn ClassHost) {
        h.ee_sub_2090d0(obj, self.f_ac);
        self.f_b4 = a;
        self.f_bc = b;
        h.ee_sub_209130(obj, self.f_ac);
    }

    /// slot 14: FUN_0020ae18. `anc` = first owner-chain object of class 0x38 (FUN_00132b10 walk), `oid` = +0x18.
    pub fn ee_sub_20ae18(&mut self, obj: usize, anc: u32, oid: i32, h: &mut dyn ClassHost) {
        h.base(0x1a5c48, obj);
        self.f_ac = anc;
        h.ee_sub_209130(obj, anc);
        if oid != 0x2cc {
            self.f_c0_self = true;
        }
        if self.f_7c != -1 {
            self.f_a0 = h.ee_sub_1f4c58(anc, self.f_7c, 6);
            self.f_a4 = h.ee_sub_1f4c58(anc, self.f_80, 6);
            // FUN_0012f938(+0xb4, +0x84) + FUN_001300b8(.., 1.0, +0xb4, +0xac, +0xa4, *(+0xa0 + 0xa0), out, 0)
            self.f_90 = h.ee_sub_1300b8(self.f_b4, self.f_84, anc, self.f_a4, self.f_a0);
        }
    }

    /// slot 40: FUN_0020ade8
    pub fn ee_sub_20ade8(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1a6730, obj);
        h.ee_sub_209590(obj);
    }
}
