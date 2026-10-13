//! C2ce2f8 (world class 117, parent C2c94e0 LO): overrides only slots 5 and 6 (defaults of +0x54..+0x60).
use super::k4::{f32_of, word};
use super::ClassHost;
use crate::engine::world::props::{Prop, PropValue};

#[derive(Clone, Debug, Default)]
pub struct C2ce2f8 {
    /// +0x40 flags (slot 6 |= 0x40)
    pub f_40: u32,
    /// +0x50 (field 1090), +0x54 (1091), +0x58/+0x5c (1092), +0x60 (1093), +0x64 (1094)
    pub f_50: i32,
    pub f_54: f32,
    pub f_58: [f32; 2],
    pub f_60: f32,
    pub f_64: i32,
}

/// slot 5: FUN_00114160
pub fn ee_sub_114160() -> u32 {
    0x2bb40c
}

impl C2ce2f8 {
    /// Load fields 1090-1094.
    pub fn feed(&mut self, props: &[Prop]) {
        for p in props {
            let PropValue::Scalar(b) = &p.value else { continue };
            match p.field {
                1090 => self.f_50 = word(b) as i32,
                1091 => self.f_54 = f32_of(b),
                1092 => self.f_58 = [f32_of(b), b.get(4..8).map(f32_of).unwrap_or(0.0)],
                1093 => self.f_60 = f32_of(b),
                1094 => self.f_64 = word(b) as i32,
                _ => {}
            }
        }
    }

    /// slot 6: FUN_001564e8 (DAT_002bdcd0 = 1500, DAT_002bdcd8 = (-200, 500))
    pub fn ee_sub_1564e8(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1a5270, obj);
        self.f_54 = 1500.0;
        self.f_58 = [-200.0, 500.0];
        self.f_60 = 0.25;
        self.f_40 |= 0x40;
    }
}
