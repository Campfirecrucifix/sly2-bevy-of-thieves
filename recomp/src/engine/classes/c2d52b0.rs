//! C2d52b0 (world class 17, parent C2c9a80 ALO). Load FUN_001c03c0 is ported in world; the class overrides
//! only slots 5 and 6.
use super::k4::word;
use super::ClassHost;
use crate::engine::world::props::{Prop, PropValue};

#[derive(Clone, Debug)]
pub struct C2d52b0 {
    /// +0x110 (field 1312)
    pub f_110: i32,
    /// +0x118 list (FUN_0011a1f8(+0x118, 0x60): empty, link offset 0x60)
    pub f_118: Vec<u32>,
    /// +0x124
    pub f_124: i32,
}

impl Default for C2d52b0 {
    fn default() -> Self {
        C2d52b0 { f_110: 0, f_118: Vec::new(), f_124: 0 }
    }
}

/// slot 5: FUN_00113b20
pub fn ee_sub_113b20() -> u32 {
    0x2bac3c
}

impl C2d52b0 {
    /// Load field 1312 (+0x110).
    pub fn feed(&mut self, props: &[Prop]) {
        for p in props {
            if let (1312, PropValue::Scalar(b)) = (p.field, &p.value) {
                self.f_110 = word(b) as i32;
            }
        }
    }

    /// slot 6: FUN_001c0378
    pub fn ee_sub_1c0378(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x231100, obj);
        self.f_110 = -1;
        self.f_118.clear();
        self.f_124 = -1;
    }
}
