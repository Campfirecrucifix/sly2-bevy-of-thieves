//! C2d97a8 (world class 111). Load FUN_001efac0 is ported in world (`B4Load::L1efac0`).
//! Object handles in hooks are index + 1 (0 = null).
use super::ClassHost;
use crate::engine::world::props::{ArrayItem, Prop, PropValue};

#[derive(Clone, Debug, Default)]
pub struct C2d97a8 {
    /// +0x74: 12-byte block {oid, a, b} allocated by method 370 (FUN_001f0128), None = null
    pub f_74: Option<[u32; 3]>,
    /// +0x40 bit 6
    pub f_40: u32,
}

/// slot 5: FUN_00114100
pub fn ee_sub_114100() -> u32 {
    0x2bb394
}

fn w0(b: &[u8]) -> u32 {
    b.get(..4).map(|x| u32::from_le_bytes([x[0], x[1], x[2], x[3]])).unwrap_or(0)
}

impl C2d97a8 {
    /// Load-time method calls in the property list: field 370 (thunk 0x1049b0 -> FUN_001f0128, 3 args).
    pub fn feed(&mut self, props: &[Prop]) {
        for p in props {
            if p.field != 370 {
                continue;
            }
            let a: Vec<u32> = match &p.value {
                PropValue::Array(items) => items
                    .iter()
                    .map(|it| match it {
                        ArrayItem::One(_, b) => w0(b),
                        ArrayItem::List(_, v) => v.first().map(|b| w0(b)).unwrap_or(0),
                    })
                    .collect(),
                PropValue::Scalar(b) => b.chunks(4).map(w0).collect(),
            };
            self.ee_sub_1f0128(a.first().copied().unwrap_or(0), a.get(1).copied().unwrap_or(0), a.get(2).copied().unwrap_or(0));
        }
    }

    /// FUN_001f0128(obj, a, b, c): +0x74 = new {a, b, c}
    pub fn ee_sub_1f0128(&mut self, a: u32, b: u32, c: u32) {
        self.f_74 = Some([a, b, c]);
    }

    /// slot 6: FUN_001efa90
    pub fn ee_sub_1efa90(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1a5270, obj);
        self.f_40 |= 0x40;
    }

    /// slot 14: FUN_001f00c8
    pub fn ee_sub_1f00c8(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1a5c48, obj);
        if let Some(p) = self.f_74 {
            let o = h.ee_sub_15e938(4, p[0] as i32, obj as u32 + 1);
            h.ee_sub_1f0380(obj, o, p[1], p[2]);
        }
    }
}
