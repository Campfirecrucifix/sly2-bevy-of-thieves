//! C2d6e08 (world class 6): overrides only slot 5 and slot 14, which it binds straight to the LO
//! load-post FUN_001a5c48 (skipping its parent C2d5dc8's FUN_001c8030).
use super::ClassHost;

/// LO fields FUN_001a5c48 touches.
#[derive(Clone, Debug, Default)]
pub struct C2d6e08 {
    /// +0x28 (u64 flags; bits 52..55 = update kind)
    pub f_28: u64,
    /// +0x30 next update time (FLT_MAX DAT_002a6a40 = never)
    pub f_30: f32,
    /// +0x40 bit 1 (tagged pointer or pointed word)
    pub f_40: u32,
    /// +0xe4 (field 1178, C2d5dc8)
    pub f_e4: bool,
}

/// slot 5: FUN_00113a70
pub fn ee_sub_113a70() -> u32 {
    0x2bab60
}

impl C2d6e08 {
    /// slot 14: FUN_001a5c48
    pub fn ee_sub_1a5c48(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.ee_sub_1a5d98(obj);
        h.vcall_1c(obj, 0, 1); // (vtable +0x1c)(obj + adj, 0, 1, 0, 0, 0)
        // *obj == 0 || FUN_00127db8(*obj, 1, 1) == 0: no handler 1 -> never scheduled
        if !h.ee_sub_127db8(obj, 1, 1) {
            self.f_30 = f32::MAX;
        } else {
            if (self.f_28 << 8) >> 60 == 2 {
                h.ee_sub_1a6ca8(obj, 0);
            }
            self.f_40 |= 2;
        }
    }

    /// slot 22 inherited from C2d5dc8: FUN_001ca8e0
    pub fn ee_sub_1ca8e0(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1a6178, obj);
        if !self.f_e4 {
            h.ee_sub_149ab8(obj, super::c2d5dc8::PMF_2A77C0, 0);
        }
    }
}
