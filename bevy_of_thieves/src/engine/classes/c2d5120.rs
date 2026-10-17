//! C2d5120 (world class 124).
use super::{ClassGlobals, ClassHost};

#[derive(Clone, Debug, Default)]
pub struct C2d5120 {
    /// +0x50: index into the 0x20-stride table at +0x40
    pub f_50: i32,
    pub f_120: i32,
    pub f_124: i32,
    pub f_128: i32,
    pub f_130: [u32; 4],
    pub f_140: [u32; 4],
    pub f_150: i32,
}

/// slot 5: FUN_001141d0 (class descriptor)
pub fn ee_sub_1141d0() -> u32 {
    0x2bb498
}

impl C2d5120 {
    /// slot 6: FUN_001c2048
    pub fn ee_sub_1c2048(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1a5270, obj);
        self.f_150 = -1;
        self.f_120 = -1;
    }
    /// slot 10: FUN_001c2078
    pub fn ee_sub_1c2078(&mut self, obj: usize, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        h.base(0x1a5800, obj);
        g.g_3e1204 += 1;
        g.g_3e1a5c_28 = g.g_3e1204;
    }
    /// slot 11: FUN_001c20b0
    pub fn ee_sub_1c20b0(&mut self, obj: usize, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        h.base(0x1a5848, obj);
        g.g_3e1204 += 1;
        g.g_3e1a5c_28 = g.g_3e1204;
    }
    /// slot 14: FUN_001c2170
    pub fn ee_sub_1c2170(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1a5c48, obj);
        self.f_130 = h.ee_sub_1c1f28(obj, 0x60);
        self.f_124 = h.ee_sub_1c0d70(self.f_130, 0);
        self.f_140 = h.ee_sub_1c1f28(obj, (self.f_50 as u32).wrapping_mul(0x20).wrapping_add(0x40));
        self.f_128 = h.ee_sub_1c0d70(self.f_140, 0);
    }
}
