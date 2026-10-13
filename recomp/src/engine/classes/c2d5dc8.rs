//! C2d5dc8 (world class 7, parent C2c9a80 ALO): attachment host (+0xe8 attached list, +0xec/+0xf0 bound-method
//! table). Slot 6 FUN_001ca788 is also class 6's inherited init. Object handles in hooks are index + 1.
use super::k4::word;
use super::{ClassGlobals, ClassHost};
use crate::engine::world::props::{Prop, PropValue};

#[derive(Clone, Debug, Default)]
pub struct C2d5dc8 {
    /// +0xe4 (field 1178)
    pub f_e4: bool,
    /// +0xe8 attached-item list (singly linked through item +4; front first)
    pub f_e8: Vec<u32>,
    /// +0xec/+0xf0: 0x18-byte entries; word 2 = bound method reference (0 = empty)
    pub f_f0: Vec<[u32; 6]>,
    /// identity of the +0xf0 allocation (clones share it until slot 40 copies it)
    pub f_f0_id: u32,
    /// +0x100 serial (DAT_003e1bf0)
    pub f_100: i32,
}

/// slot 5: FUN_00113a80
pub fn ee_sub_113a80() -> u32 {
    0x2bab74
}

/// FUN_00149ab8 target of slot 22 (DAT_002a77c0)
pub const PMF_2A77C0: u32 = 0x2a77c0;

impl C2d5dc8 {
    /// Load field 1178 (+0xe4).
    pub fn feed(&mut self, props: &[Prop]) {
        for p in props {
            if let (1178, PropValue::Scalar(b)) = (p.field, &p.value) {
                self.f_e4 = word(b) != 0;
            }
        }
    }

    /// slot 6: FUN_001ca788
    pub fn ee_sub_1ca788(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x231100, obj);
        h.ee_sub_1a6ca8(obj, 0);
    }

    /// slot 14: FUN_001c8030
    pub fn ee_sub_1c8030(&mut self, obj: usize, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        h.base(0x1a5c48, obj);
        self.f_100 = g.g_3e1bf0;
        g.g_3e1bf0 += 1;
        if h.ee_sub_1c7bb8(self.f_100) {
            h.vcall(obj as u32 + 1, 0x4c);
        }
    }

    /// slot 22: FUN_001ca8e0
    pub fn ee_sub_1ca8e0(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1a6178, obj);
        if !self.f_e4 {
            h.ee_sub_149ab8(obj, PMF_2A77C0, 0);
        }
    }

    /// slot 23: FUN_001cb6d8
    pub fn ee_sub_1cb6d8(&mut self, obj: usize, h: &mut dyn ClassHost) {
        let me = obj as u32 + 1;
        while let Some(&x) = self.f_e8.first() {
            // FUN_001caa60(obj, x): unlink x from +0xe8 (here the front), then detach it
            self.f_e8.remove(0);
            h.ee_sub_1caa60(obj, x);
        }
        // FUN_0011a380(0x3dd4bc, +0x14): in the set -> (vtable +0xb4)(**(+0x14)), FUN_0011a2e8 remove
        if h.ee_sub_11a380(me) {
            let r = h.ref_14(me);
            h.vcall(r, 0xb4);
            h.ee_sub_11a2e8(me);
        }
    }

    /// slot 40: FUN_001ca7b8 (clone of `src`): rebind every bound method to the clone, copying a shared table
    pub fn ee_sub_1ca7b8(&mut self, obj: usize, src_f0_id: u32, h: &mut dyn ClassHost) {
        h.base(0x2311b0, obj);
        for k in 0..self.f_f0.len() {
            if self.f_f0[k][2] == 0 {
                continue;
            }
            if self.f_f0_id == src_f0_id {
                // FUN_001ae238({n * 0x18, 0x10}, +0xf0): own copy
                self.f_f0_id = 0x8000_0000 | (obj as u32 + 1);
            }
            let old = self.f_f0[k][2];
            let r = h.ee_sub_127e90(old, obj as u32 + 1);
            self.f_f0[k][2] = r;
            if r != 0 {
                h.ref_release(r); // temporary's destructor (FUN_00257078 at 0)
            }
            h.ee_sub_259a88(r);
        }
    }
}
