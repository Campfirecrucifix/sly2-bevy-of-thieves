//! C2d9040 (world class 10).
use super::ClassHost;

#[derive(Clone, Debug, Default)]
pub struct C2d9040 {
    pub f_148: i32,
    pub f_178: i32,
    /// +0x17c: active sub-object (0 = none)
    pub f_17c: i32,
}

/// slot 5: FUN_00113ab0
pub fn ee_sub_113ab0() -> u32 {
    0x2babb0
}

impl C2d9040 {
    /// slot 6: FUN_001e5d80
    pub fn ee_sub_1e5d80(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x231100, obj);
        self.f_178 = 1;
        self.f_148 = -1;
    }
    /// slot 16: FUN_001e6340
    pub fn ee_sub_1e6340(&mut self, obj: usize, h: &mut dyn ClassHost) {
        if self.f_17c == 0 {
            h.ee_sub_1a5e80(obj);
            return;
        }
        match h.state_4c(self.f_17c) {
            0 | 3 => h.ee_sub_1e8378(self.f_17c),
            1 | 2 => h.ee_sub_1e9090(),
            _ => {}
        }
    }
    /// slot 40: FUN_001e5db8
    pub fn ee_sub_1e5db8(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x2311b0, obj);
        if self.f_17c != 0 {
            self.f_17c = 0;
            h.ee_sub_1e7840(obj, 0);
        }
    }
}
