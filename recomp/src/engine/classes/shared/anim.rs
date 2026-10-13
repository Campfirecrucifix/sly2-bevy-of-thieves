//! FUN_001305b8 / FUN_00130658: (re)play an anim into a handle slot.
use crate::engine::classes::ClassHost;

/// FUN_00130658(&slot): a live handle is retracted (FUN_00130b18), released (vtable +0x14) and cleared
pub fn ee_sub_130658(slot: &mut u32, h: &mut dyn ClassHost) {
    if *slot != 0 {
        h.ee_sub_130b18(*slot);
        h.vcall(*slot, 0x14);
        *slot = 0;
    }
}

/// FUN_001305b8(a, rate, anim, obj, flags, &slot): drop the slot's handle, then (anim != 0) request the anim
/// (FUN_0012f9b8 writes the slot) and add a reference to it (vtable +0xc)
pub fn ee_sub_1305b8(a: u32, rate: f32, anim: u32, obj: u32, flags: u32, slot: &mut u32, h: &mut dyn ClassHost) {
    ee_sub_130658(slot, h);
    if anim != 0 {
        *slot = h.ee_sub_12f9b8(a, rate, anim, obj, flags);
        h.vcall(*slot, 0xc);
    }
}
