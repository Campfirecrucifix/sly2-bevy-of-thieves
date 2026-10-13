//! C2d9978 (world class 48).
use super::ClassHost;

#[derive(Clone, Debug, Default)]
pub struct C2d9978 {
    /// +0x10
    pub owner: Option<usize>,
    /// +0x2d
    pub f_2d: u8,
    /// +0x12c (float)
    pub f_12c: f32,
    pub f_134: i32,
    pub f_13c: u32,
    pub f_140: i32,
    /// +0x148 (u64 flags; bytes 0x148/0x149 written separately)
    pub f_148: u64,
}

/// slot 5: FUN_00113d10
pub fn ee_sub_113d10() -> u32 {
    0x2baea8
}

impl C2d9978 {
    pub fn bit35(&self) -> bool {
        self.f_148 >> 35 & 1 != 0
    }
    /// slot 6: FUN_001f2108; `owner_bit` = owner's +0x148 bit 35 (None = no owner -> 1)
    pub fn ee_sub_1f2108(&mut self, obj: usize, owner_bit: Option<bool>, g_2c3b90: u32, h: &mut dyn ClassHost) {
        h.base(0x1a5270, obj);
        h.ee_sub_1a6ca8(obj, 0);
        h.ee_sub_11a1f8(obj, 100, 0x24);
        self.f_148 = (self.f_148 & !0xffff) | 0xecf6;
        self.f_12c = 1.0;
        self.f_13c = g_2c3b90;
        let b = owner_bit.unwrap_or(true) as u64;
        self.f_140 = -1;
        self.f_148 = self.f_148 & 0xffff_fff7_ffff_ffff | b << 35;
    }
    /// slot 18: FUN_001f3c70
    pub fn ee_sub_1f3c70(&mut self, obj: usize, g_2ddf54: u8, h: &mut dyn ClassHost) {
        if self.f_2d & g_2ddf54 != 0 {
            h.ee_sub_1f3c10(obj);
            h.ee_sub_168638(obj, self.f_12c, self.f_134);
        }
    }
}
