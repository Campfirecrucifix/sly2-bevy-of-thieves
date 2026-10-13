//! C2d56f8 (world class 11, parent C2d9040 = class 10): sibling of class 13 with a +0x1c0 axis and +0x1d0/+0x1d4
//! ids, listed in DAT_003e13c0. Inherited class-10 overrides run on the embedded `base`. Handles = index + 1.
use super::c2d6fe8::C2d6fe8;
use super::k4::{f32_of, vec3_of, word};
use super::{ClassGlobals, ClassHost};
use crate::engine::world::props::{Prop, PropValue};

#[derive(Clone, Debug)]
pub struct C2d56f8 {
    /// class-10 base + the shared +0x38 / +0x190 sub-block / +0x1b4 layout (FUN_00182c70 is common)
    pub sub: C2d6fe8,
    /// +0x1c0 axis (field 838), +0x1d0 / +0x1d4 (fields 839 / 840)
    pub f_1c0: [f32; 4],
    pub f_1d0: i32,
    pub f_1d4: i32,
}

impl Default for C2d56f8 {
    fn default() -> Self {
        C2d56f8 { sub: C2d6fe8::default(), f_1c0: [0.0; 4], f_1d0: 0, f_1d4: 0 }
    }
}

/// slot 5: FUN_00113ac0
pub fn ee_sub_113ac0() -> u32 {
    0x2babc4
}

/// DAT_002c0d00
const RADIUS_PAD: f32 = 1250.0;

impl C2d56f8 {
    /// Load fields 13 (+0x38), 838-840.
    pub fn feed(&mut self, props: &[Prop]) {
        for p in props {
            let PropValue::Scalar(b) = &p.value else { continue };
            match p.field {
                13 => self.sub.f_38 = f32_of(b),
                838 => self.f_1c0 = vec3_of(b),
                839 => self.f_1d0 = word(b) as i32,
                840 => self.f_1d4 = word(b) as i32,
                _ => {}
            }
        }
    }

    /// slot 6: FUN_001c3390
    pub fn ee_sub_1c3390(&mut self, obj: usize, h: &mut dyn ClassHost) {
        self.sub.base.ee_sub_1e5d80(obj, h);
        self.sub.ee_sub_182c70(obj, h);
        self.sub.f_1ac = 1;
        self.f_1c0 = [1.0, 0.0, 0.0, 0.0]; // DAT_002a2da0
        self.f_1d4 = 9;
        self.f_1d0 = 8;
    }

    /// slot 14: FUN_001c3458
    pub fn ee_sub_1c3458(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1a5c48, obj);
        if self.sub.f_38 == 1e10 {
            self.sub.f_38 = h.ee_sub_1e7d90(obj) + RADIUS_PAD;
        }
    }

    /// slot 21: FUN_001c3420 (FUN_0011a2e8(0x3e13c0, obj))
    pub fn ee_sub_1c3420(&mut self, obj: usize, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        h.base(0x1a6150, obj);
        g.l_3e13c0.retain(|&x| x != obj as u32 + 1);
    }

    /// slot 22: FUN_001c33e8 (FUN_0011a220(0x3e13c0, obj))
    pub fn ee_sub_1c33e8(&mut self, obj: usize, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        h.base(0x1a6178, obj);
        g.l_3e13c0.push(obj as u32 + 1);
    }

    /// slot 40: FUN_001c34b8
    pub fn ee_sub_1c34b8(&mut self, obj: usize, h: &mut dyn ClassHost) {
        let v = self.sub.f_1b4;
        self.sub.base.ee_sub_1e5db8(obj, h);
        self.sub.f_1b4 = v;
    }
}
