//! C2cf1e8 (world class 133). Object handles in hooks are index + 1 (0 = null).
use super::{ClassGlobals, ClassHost};
use crate::engine::world::props::{Prop, PropValue};

#[derive(Clone, Debug, Default)]
pub struct C2cf1e8 {
    /// +0x28 (u64 LO flags)
    pub f_28: u64,
    /// +0x40 bit 1
    pub f_40: u32,
    /// +0x50
    pub f_50: i32,
    /// +0x54 state (setter field 1286), +0x58 state time
    pub f_54: i32,
    pub f_58: f32,
    /// +0x5c (field 1265, s16)
    pub f_5c: i32,
    pub f_60: u8,
    /// +0x64 / +0x68: save-blob words FUN_002581c0 resolved for the +0xc8 / +0xd0 paths
    pub f_64: u32,
    pub f_68: u32,
    pub f_6c: u8,
    pub f_70: u8,
    pub f_74: u8,
    /// +0x78 (f32, field 1285)
    pub f_78: f32,
    /// +0x7c / +0x80 object references (fields 1264 / 1279; oid fixed up at load)
    pub f_7c: u32,
    pub f_80: u32,
    /// +0x90/+0x94 -> +0x98 and +0x9c/+0xa0 -> +0xa4: oid lists (load FUN_001635f8) and their resolved objects
    pub f_90: Vec<i32>,
    pub f_98: Vec<u32>,
    pub f_9c: Vec<i32>,
    pub f_a4: Vec<u32>,
    pub f_a8: i32,
    pub f_ac: i32,
    pub f_b0: i32,
    pub f_b4: i32,
    pub f_b8: i32,
    pub f_bc: i32,
    pub f_c0: i32,
    /// +0xc4 object reference (field 1278)
    pub f_c4: u32,
    /// +0xc8 / +0xd0: save-variable paths (kind 0x16 u16 lists, fields 1295 / 1296)
    pub f_c8: Vec<u16>,
    pub f_d0: Vec<u16>,
    pub f_f0: i32,
    pub f_f4: i32,
}

/// slot 5: FUN_00114260
pub fn ee_sub_114260() -> u32 {
    0x2bb54c
}

fn word(b: &[u8]) -> u32 {
    b.get(..4).map(|x| u32::from_le_bytes([x[0], x[1], x[2], x[3]])).unwrap_or(0)
}

impl C2cf1e8 {
    /// Load-time field writes (class 133 rows of 0x2ac880) + the FUN_001635f8 oid lists. Object-reference
    /// fields keep the oid; `fix` = load-time oid fixup (FUN_0013e590/0013e5e0) to a handle.
    pub fn feed(&mut self, props: &[Prop], a: &[i32], b: &[i32], fix: &mut dyn FnMut(i32) -> u32) {
        self.f_90 = a.to_vec();
        self.f_9c = b.to_vec();
        for p in props {
            let PropValue::Scalar(v) = &p.value else { continue };
            let b0 = v.first().copied().unwrap_or(0);
            match p.field {
                1264 => self.f_7c = fix(word(v) as i32),
                1265 => self.f_5c = v.get(..2).map(|x| i16::from_le_bytes([x[0], x[1]]) as i32).unwrap_or(0),
                1266 => self.f_60 = b0,
                1271 => self.f_6c = b0,
                1272 => self.f_70 = b0,
                1273 => self.f_74 = b0,
                1274 => self.f_b8 = word(v) as i32,
                1275 => self.f_bc = v.get(..2).map(|x| i16::from_le_bytes([x[0], x[1]]) as i32).unwrap_or(0),
                1276 => self.f_c0 = word(v) as i32,
                1278 => self.f_c4 = fix(word(v) as i32),
                1279 => self.f_80 = fix(word(v) as i32),
                1285 => self.f_78 = f32::from_bits(word(v)),
                1295 | 1296 => {
                    let l: Vec<u16> = v.get(2..).unwrap_or(&[]).chunks_exact(2).map(|x| u16::from_le_bytes([x[0], x[1]])).collect();
                    if p.field == 1295 { self.f_c8 = l } else { self.f_d0 = l }
                }
                _ => {}
            }
        }
    }

    /// slot 6: FUN_001635a0
    pub fn ee_sub_1635a0(&mut self, obj: usize, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        h.base(0x1a5270, obj);
        self.f_f4 = 0;
        self.f_c0 = -1;
        self.f_54 = -1;
        self.f_f0 = -1;
        self.f_5c = -1;
        self.f_b8 = -1;
        self.f_bc = -1;
        g.g_3e0f8c += 1;
    }

    /// FUN_001637e8
    pub fn ee_sub_1637e8(&self, h: &mut dyn ClassHost) -> bool {
        self.f_50 == 0 && self.f_f4 != 2 && self.f_f4 != 3 && h.ee_sub_20e040() == 0
    }

    /// slot 14: FUN_001638f8
    pub fn ee_sub_1638f8(&mut self, obj: usize, g: &ClassGlobals, h: &mut dyn ClassHost) {
        let me = obj as u32 + 1;
        h.base(0x1a5c48, obj);
        if self.f_7c != 0 {
            h.ee_sub_11a070(self.f_7c, 0x80, me); // FUN_0011a070(+0x7c + 0x80, obj)
        }
        // keep the oids that resolve (FUN_0015e938 kind 0x105), compacted
        self.f_98 = self.f_90.iter().map(|&o| h.ee_sub_15e938(0x105, o, 0)).filter(|&r| r != 0).collect();
        self.f_a4 = self.f_9c.iter().map(|&o| h.ee_sub_15e938(0x105, o, 0)).filter(|&r| r != 0).collect();
        // FUN_00164018: DAT_003e0fa0 == obj
        if g.g_3e0fa0 != me && self.ee_sub_1637e8(h) {
            h.ee_sub_1620b0(me, 5, self.f_a8, self.f_ac);
            h.ee_sub_1623d0(me, 5, self.f_b0, self.f_b4, 1);
        }
        if !self.f_c8.is_empty() {
            self.f_64 = h.ee_sub_2581c0(&self.f_c8); // FUN_002581c0(.., +200, 0x3d4a60) + FUN_00129ec8
        }
        if !self.f_d0.is_empty() {
            self.f_68 = h.ee_sub_2581c0(&self.f_d0);
        }
        if self.f_78 == 0.0 && self.f_80 == 0 {
            return;
        }
        if (self.f_28 << 8) >> 60 == 2 {
            h.ee_sub_1a6ca8(obj, 0);
        }
        self.f_40 |= 2;
    }

    /// slot 15: FUN_00163b48
    pub fn ee_sub_163b48(&mut self, obj: usize, g: &ClassGlobals, h: &mut dyn ClassHost) {
        h.ee_sub_1a5e28(obj);
        if self.f_54 == 1 {
            let d = self.f_78;
            let e = g.t - self.f_58;
            // quirk: e < d && d + dt <= e can never hold for dt >= 0 (kept)
            if 0.0 < d && e < d && d + g.dt <= e {
                h.ee_sub_165118(obj as u32 + 1, 0);
            }
        }
        if self.f_80 != 0 {
            let off = if g.g_3e0f9c == 0 && self.f_54 == 1 { 0x44 } else { 0x4c };
            h.vcall(self.f_80, off);
        }
    }
}
