//! C2d9dd0 (world class 39, parent C2c9a80 ALO): ALO with the FUN_00182c70 sub-block at +0xe0 (fields 850-857
//! are relative to it), a fixed 1250 draw radius and membership in the DAT_003e13d8 list. Handles = index + 1.
use super::k4::{f32_of, word};
use super::{ClassGlobals, ClassHost};
use crate::engine::world::props::{Prop, PropValue};

#[derive(Clone, Debug)]
pub struct C2d9dd0 {
    /// +0x38 draw radius (field 13; 1e10 = default)
    pub f_38: f32,
    /// +0xe0 sub-block: +0xe8 (field 852), +0xf4 (855), +0xfc (857)
    pub f_e8: f32,
    pub f_f4: i32,
    pub f_fc: i32,
    /// +0x104 (8 bytes, kept across the clone)
    pub f_104: u64,
}

impl Default for C2d9dd0 {
    fn default() -> Self {
        // assumption: LO default +0x38 = 1e10 (as class 13)
        C2d9dd0 { f_38: 1e10, f_e8: 0.0, f_f4: 0, f_fc: 0, f_104: 0 }
    }
}

/// slot 5: FUN_00113c80
pub fn ee_sub_113c80() -> u32 {
    0x2badf4
}

impl C2d9dd0 {
    /// Load field 13 and the sub-block fields 852 / 855 / 857.
    pub fn feed(&mut self, props: &[Prop]) {
        for p in props {
            let PropValue::Scalar(b) = &p.value else { continue };
            match p.field {
                13 => self.f_38 = f32_of(b),
                852 => self.f_e8 = f32_of(b),
                855 => self.f_f4 = word(b) as i32,
                857 => self.f_fc = word(b) as i32,
                _ => {}
            }
        }
    }

    /// slot 6: FUN_001ff3b8
    pub fn ee_sub_1ff3b8(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x231100, obj);
        // FUN_00182c70(obj + 0xe0)
        self.f_f4 = -1;
        self.f_e8 = f32::MAX;
        self.f_fc = 7;
        h.ee_sub_182c70_tail(obj);
    }

    /// slot 14: FUN_001ff3e8 (DAT_002c3c58 = 1250)
    pub fn ee_sub_1ff3e8(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1a5c48, obj);
        if self.f_38 == 1e10 {
            self.f_38 = 1250.0;
        }
    }

    /// slot 21: FUN_001ff470 (FUN_0011a2e8(0x3e13d8, obj))
    pub fn ee_sub_1ff470(&mut self, obj: usize, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        h.base(0x1a6150, obj);
        g.l_3e13d8.retain(|&x| x != obj as u32 + 1);
    }

    /// slot 22: FUN_001ff438 (FUN_0011a220(0x3e13d8, obj))
    pub fn ee_sub_1ff438(&mut self, obj: usize, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        h.base(0x1a6178, obj);
        g.l_3e13d8.push(obj as u32 + 1);
    }

    /// slot 40: FUN_001ff4a8
    pub fn ee_sub_1ff4a8(&mut self, obj: usize, h: &mut dyn ClassHost) {
        let v = self.f_104;
        h.base(0x2311b0, obj);
        self.f_104 = v;
    }
}
