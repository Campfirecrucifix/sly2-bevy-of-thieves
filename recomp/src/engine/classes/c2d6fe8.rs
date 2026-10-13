//! C2d6fe8 (world class 13, parent C2d9040 = class 10): class-10 object with a +0x190 sub-block, a draw radius
//! fixed at load and membership in the DAT_003e13cc list. The inherited class-10 overrides (slots 6/16/40 of
//! C2d9040) run on the embedded `base`. Object handles in hooks are index + 1.
use super::c2d9040::C2d9040;
use super::k4::{f32_of, word};
use super::{ClassGlobals, ClassHost};
use crate::engine::world::props::{Prop, PropValue};

#[derive(Clone, Debug)]
pub struct C2d6fe8 {
    pub base: C2d9040,
    /// +0x38 draw radius (field 13; 1e10 = derive from the geometry at load)
    pub f_38: f32,
    /// +0x190 sub-block (FUN_00182c70): +0x198, +0x1a4, +0x1ac
    pub f_198: f32,
    pub f_1a4: i32,
    pub f_1ac: i32,
    /// +0x1b4 (8 bytes, kept across the clone)
    pub f_1b4: u64,
    /// +0x1bc (field 858), +0x1c0 (859), +0x1c4 (860), +0x1c8 (861)
    pub f_1bc: bool,
    pub f_1c0: f32,
    pub f_1c4: f32,
    pub f_1c8: u32,
}

impl Default for C2d6fe8 {
    fn default() -> Self {
        C2d6fe8 {
            base: C2d9040::default(),
            // assumption: LO default +0x38 (base ctor not decompiled)
            f_38: 1e10,
            f_198: 0.0,
            f_1a4: 0,
            f_1ac: 0,
            f_1b4: 0,
            f_1bc: false,
            f_1c0: 0.0,
            f_1c4: 0.0,
            f_1c8: 0,
        }
    }
}

/// slot 5: FUN_00113ae0
pub fn ee_sub_113ae0() -> u32 {
    0x2babec
}

/// DAT_002c1ec0
const RADIUS_PAD: f32 = 1250.0;

impl C2d6fe8 {
    /// Load fields 13 (+0x38), 858-861.
    pub fn feed(&mut self, props: &[Prop]) {
        for p in props {
            let PropValue::Scalar(b) = &p.value else { continue };
            match p.field {
                13 => self.f_38 = f32_of(b),
                858 => self.f_1bc = word(b) != 0,
                859 => self.f_1c0 = f32_of(b),
                860 => self.f_1c4 = f32_of(b),
                861 => self.f_1c8 = word(b),
                _ => {}
            }
        }
    }

    /// slot 6: FUN_001cb908
    pub fn ee_sub_1cb908(&mut self, obj: usize, h: &mut dyn ClassHost) {
        self.base.ee_sub_1e5d80(obj, h);
        self.ee_sub_182c70(obj, h);
        self.f_1c4 = 1.0;
        self.f_1c0 = 1000.0;
    }

    /// FUN_00182c70(obj + 0x190)
    pub fn ee_sub_182c70(&mut self, obj: usize, h: &mut dyn ClassHost) {
        self.f_1a4 = -1;
        self.f_198 = f32::MAX; // DAT_002a5680
        self.f_1ac = 7;
        // FUN_001a6ca8((sub vtable +0x34)(sub), 3)
        h.ee_sub_182c70_tail(obj);
    }

    /// slot 14: FUN_001cb950
    pub fn ee_sub_1cb950(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1a5c48, obj);
        if self.f_38 == 1e10 {
            self.f_38 = h.ee_sub_1e7d90(obj) + RADIUS_PAD;
        }
    }

    /// slot 21: FUN_001cb9e8 (FUN_0011a2e8(0x3e13cc, obj))
    pub fn ee_sub_1cb9e8(&mut self, obj: usize, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        h.base(0x1a6150, obj);
        g.l_3e13cc.retain(|&x| x != obj as u32 + 1);
    }

    /// slot 22: FUN_001cb9b0 (FUN_0011a220(0x3e13cc, obj))
    pub fn ee_sub_1cb9b0(&mut self, obj: usize, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        h.base(0x1a6178, obj);
        g.l_3e13cc.push(obj as u32 + 1);
    }

    /// slot 40: FUN_001cba20
    pub fn ee_sub_1cba20(&mut self, obj: usize, h: &mut dyn ClassHost) {
        let v = self.f_1b4;
        self.base.ee_sub_1e5db8(obj, h);
        self.f_1b4 = v;
    }
}
