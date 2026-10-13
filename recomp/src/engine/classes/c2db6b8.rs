//! C2db6b8 (world class 15): player-proximity volume (parent C2c9a80 ALO). Load FUN_00220fd0 is ported in
//! world (`B4Load::L220fd0`). Each frame (slot 15) it defers FUN_00221298, which tests the FUN_001c4a48 object
//! against the +0x108 volume and switches the +0x134 state (FUN_00221178: camera 0x2ddf40 + vtable +0x1c).
//! Object handles in hooks are index + 1 (0 = null).
use super::k4::word;
use super::{ClassGlobals, ClassHost};
use crate::engine::world::props::{Prop, PropValue};

#[derive(Clone, Debug, Default)]
pub struct C2db6b8 {
    /// +0xe0 current FUN_001c4a48 object (handle)
    pub f_e0: u32,
    /// +0x10c sub-object (slot 27 / slot 40)
    pub f_10c: u32,
    /// +0x124 (field 201), +0x128 (field 200)
    pub f_124: bool,
    pub f_128: i32,
    /// +0x134 state, +0x138 state time, +0x13c forced state (field 214, -1 = from the volume test)
    pub f_134: i32,
    pub f_138: f32,
    pub f_13c: i32,
    /// +0x164 (field 197), +0x170..+0x17c (fields 203-206), +0x180 (field 202)
    pub f_164: i32,
    pub f_170: [f32; 4],
    pub f_180: i32,
    /// +0x1a0 (field 211) / +0x1b0 (field 212)
    pub f_1a0: [u32; 4],
    pub f_1b0: [u32; 2],
    /// +0x40 bit 1
    pub f_40: u32,
}

/// slot 5: FUN_00113b00
pub fn ee_sub_113b00() -> u32 {
    0x2bac14
}

/// FUN_00149a50 target of slot 15 (DAT_002a96f0 = {-1, 0x221298})
pub const PMF_221298: u32 = 0x221298;

impl C2db6b8 {
    /// Load fields 200, 201, 214 (+0x128, +0x124, +0x13c), written after slot 6.
    pub fn feed(&mut self, props: &[Prop]) {
        for p in props {
            let PropValue::Scalar(b) = &p.value else { continue };
            match p.field {
                200 => self.f_128 = word(b) as i32,
                201 => self.f_124 = word(b) != 0,
                214 => self.f_13c = word(b) as i32,
                _ => {}
            }
        }
    }

    /// slot 6: FUN_00220ed0
    pub fn ee_sub_220ed0(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x231100, obj);
        self.f_170 = [2.0, 1.0, 2.0, 1.0];
        self.f_180 = -1;
        // DAT_002c41c0 / DAT_002c41d0
        self.f_1a0 = [0, 0x3f000000, 0, 0];
        self.f_1b0 = [0, 0x7f7fffff];
        self.f_13c = -1;
        self.f_164 = 7;
        h.ee_sub_1a6ca8(obj, 1);
        self.f_40 |= 2;
    }

    /// slot 11: FUN_00220f88 (FUN_00149b58(0x3dd490, 1, obj, DAT_002a96e8, 0): drop the object's deferred calls)
    pub fn ee_sub_220f88(&mut self, obj: usize, deferred: &mut Vec<(usize, u32)>, h: &mut dyn ClassHost) {
        h.base(0x1a5848, obj);
        deferred.retain(|d| d.0 != obj);
    }

    /// slot 15: FUN_00221388
    pub fn ee_sub_221388(&mut self, obj: usize, deferred: &mut Vec<(usize, u32)>, h: &mut dyn ClassHost) {
        h.ee_sub_1a5e28(obj);
        if self.f_13c != 0 {
            // FUN_00149a50(0x3dd490, obj, DAT_002a96f0, 0)
            deferred.push((obj, PMF_221298));
        }
    }

    /// slot 21: FUN_002213d8
    pub fn ee_sub_2213d8(&mut self, obj: usize, g: &ClassGlobals, h: &mut dyn ClassHost) {
        h.base(0x1a6150, obj);
        self.ee_sub_221178(obj, 0, g, h);
    }

    /// slot 27: FUN_00221120 -> the +0x10c reference when set
    pub fn ee_sub_221120(&self) -> Option<u32> {
        (self.f_10c != 0).then_some(self.f_10c)
    }

    /// slot 40: FUN_00221140 (clone: +0x10c remapped into the clone's owner +0x10)
    pub fn ee_sub_221140(&mut self, obj: usize, owner: u32, h: &mut dyn ClassHost) {
        h.base(0x2311b0, obj);
        self.f_10c = h.ee_sub_13e510(self.f_10c, owner);
    }

    /// FUN_00221178(obj, s): state change
    pub fn ee_sub_221178(&mut self, obj: usize, s: i32, g: &ClassGlobals, h: &mut dyn ClassHost) {
        if s == self.f_134 {
            return;
        }
        if s == 0 {
            h.ee_sub_14e4c8_k(8, -1, 0, 0, obj as u32 + 1);
            h.vcall_1c(obj, 6, 1);
            self.f_134 = 0;
        } else if s == 1 {
            h.ee_sub_14e328(self.f_128 + 2, 0x2de2f0, self.f_e0, obj as u32 + 1);
            h.vcall_1c(obj, 5, 1);
            self.f_134 = 1;
        } else {
            self.f_134 = s;
        }
        self.f_138 = g.t;
    }

    /// FUN_00221298 (deferred from slot 15)
    pub fn ee_sub_221298(&mut self, obj: usize, g: &ClassGlobals, h: &mut dyn ClassHost) {
        let p = h.ee_sub_1c4a48();
        if p == 0 {
            return;
        }
        if p != self.f_e0 {
            self.ee_sub_221178(obj, 0, g, h);
            self.f_e0 = p;
        }
        let mut s = self.f_13c as u32;
        if s == 0xffffffff {
            let w = h.ee_sub_1a62e8(p);
            let l = h.ee_sub_1a6f00(obj, w);
            s = h.ee_sub_118fb0_at(obj, 0x108, l) as u32;
        }
        if self.f_124 || h.ee_sub_132b10(p, 0x34) == 0 || h.obj_i32(p, 0x2dc) == -1 || g.g_2de240 != 0 || self.f_13c != -1 {
            self.ee_sub_221178(obj, s as i32, g, h);
        }
    }
}
