//! C2caf88 (world class 86): SO subclass with an oid / class filter (fields 456-459). Its own overrides are
//! slots 5, 6, 14, 54 and 60; every other slot is the ALO core (actor/alo.rs).
use super::super::k4::word;
use super::super::ClassHost;
use super::alo::Alo;
use crate::engine::world::props::{ArrayItem, Prop, PropValue};

/// FUN_00149ab8 member-function word queued by slot 54 (DAT_002a4790)
pub const PMF_2A4790: u32 = 0x2a4790;

#[derive(Clone, Debug, Default)]
pub struct C2caf88 {
    pub alo: Alo,
    /// +0x150 count / +0x154 oids (max 4)
    pub f_154: Vec<i32>,
    /// +0x164 count / +0x168 class ids (max 4)
    pub f_168: Vec<i32>,
    /// +0x178 (field 458, u8)
    pub f_178: u8,
    /// +0x17c (field 459, oid reference)
    pub f_17c: i32,
}

/// slot 5: FUN_00113f70
pub fn ee_sub_113f70() -> u32 {
    0x2bb1a0
}

/// Raw argument words of a method / scalar prop.
pub fn prop_args(p: &Prop) -> Vec<u32> {
    match &p.value {
        PropValue::Array(items) => items
            .iter()
            .map(|it| match it {
                ArrayItem::One(_, b) => word(b),
                ArrayItem::List(_, v) => v.first().map(|b| word(b)).unwrap_or(0),
            })
            .collect(),
        PropValue::Scalar(b) => vec![word(b)],
    }
}

impl C2caf88 {
    /// Load-time fields: 456 (thunk 0x1057e0 -> FUN_001323c8), 457 (0x105868 -> FUN_001323f0), 458, 459.
    pub fn feed(&mut self, props: &[Prop]) {
        for p in props {
            let a = prop_args(p);
            let a0 = a.first().copied().unwrap_or(0);
            match p.field {
                456 => self.ee_sub_1323c8(a0 as i32),
                457 => self.ee_sub_1323f0(a0 as i32),
                458 => self.f_178 = a0 as u8,
                459 => self.f_17c = a0 as i32,
                _ => {}
            }
        }
    }
    /// FUN_001323c8: append an oid (+0x150 < 4)
    pub fn ee_sub_1323c8(&mut self, oid: i32) {
        if self.f_154.len() < 4 {
            self.f_154.push(oid);
        }
    }
    /// FUN_001323f0: append a class id (+0x164 < 4)
    pub fn ee_sub_1323f0(&mut self, cls: i32) {
        if self.f_168.len() < 4 {
            self.f_168.push(cls);
        }
    }
    /// slot 6: FUN_001320c8 = FUN_001f2108 + (+0x148 |= bit 34)
    pub fn ee_sub_1320c8(&mut self, obj: usize, owner_bit: Option<bool>, g_2c3b90: u32, h: &mut dyn ClassHost) {
        self.alo.so.ee_sub_1f2108(obj, owner_bit, g_2c3b90, h);
        self.alo.so.f_148 |= 0x4_0000_0000;
    }
    /// slot 14: FUN_00132100 = FUN_001f36a8 (+0x80 pose: FUN_001a5c48, FUN_0022faa0, FUN_00167b68) +
    /// FUN_00134320(0x3dd490, obj) when +0xe8 is set
    pub fn ee_sub_132100(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1f36a8, obj);
        if h.phys_e8(obj).is_some() {
            h.ee_sub_134320(obj);
        }
    }
    /// slot 54: FUN_00132238 = FUN_001f4fb0 (empty) + queue DAT_002a4790 when the method table has handler 5
    pub fn ee_sub_132238(&mut self, obj: usize, h: &mut dyn ClassHost) {
        if h.ee_sub_127db8(obj, 5, 1) {
            h.ee_sub_149ab8(obj, PMF_2A4790, 0);
        }
    }
    /// slot 60: FUN_00132140(obj, other): 1 / 2 / 3 or the core FUN_001f4fc8 answer
    pub fn ee_sub_132140(&self, obj: usize, other: u32, h: &mut dyn ClassHost) -> i32 {
        // every entry is tested (no early exit)
        let mut b = false;
        for &oid in &self.f_154 {
            b |= h.ee_sub_1a6290(other, oid);
        }
        for &c in &self.f_168 {
            b |= h.ee_sub_132b10(other, c) != 0;
        }
        if self.f_178 == 0 {
            if b { 2 } else { 1 }
        } else if b {
            3
        } else {
            ee_sub_1f4fc8(&self.alo, obj, other, h)
        }
    }
}

/// FUN_001f4fc8(obj, other): 0 unless obj has +0x148 bit 26 or vcall +500 != 0; then other's vcall +500 != 0
pub fn ee_sub_1f4fc8(alo: &Alo, obj: usize, other: u32, h: &mut dyn ClassHost) -> i32 {
    if alo.so.f_148 & 0x400_0000 == 0 && h.vcall_ret(Alo::h(obj), 500) == 0 {
        return 0;
    }
    (h.vcall_ret(other, 500) != 0) as i32
}
