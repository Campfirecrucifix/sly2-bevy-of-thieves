//! C2cf880 (world class 113): load-time object-path list. Its property list calls the script methods of
//! fields 595-612 (thunks 0x105d58..0x1065e0 -> FUN_00170b40..FUN_00170dd0), each appending a 0x18-byte entry
//! (FUN_00170e00) to the +0x50 list; slot 14 walks that list (FUN_00170a58). Object handles = index + 1.
use super::{ClassHost};
use crate::engine::world::props::{ArrayItem, Prop, PropValue};

/// One +0x50 list entry (FUN_00170e00: +0 kind, +4 id, +8..+0x18 payload).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Entry {
    pub kind: i32,
    pub id: i32,
    /// +8 (8 bytes: kinds 1/2 copy one 8-byte argument; others store words at +8 / +0xc)
    pub f_8: [u32; 2],
    pub f_10: u32,
    pub f_14: u32,
}

#[derive(Clone, Debug, Default)]
pub struct C2cf880 {
    /// +0x50 list (FUN_0011a050(+0x50, 0x18), FUN_0011a070 append)
    pub f_50: Vec<Entry>,
}

/// slot 5: FUN_00114120
pub fn ee_sub_114120() -> u32 {
    0x2bb3bc
}

fn w(b: &[u8], i: usize) -> u32 {
    b.get(i * 4..i * 4 + 4).map(|x| u32::from_le_bytes([x[0], x[1], x[2], x[3]])).unwrap_or(0)
}

/// FUN_001709a8: 0 if the object has a class-0xd ancestor, else 1 for class 0xb, else -1
pub fn ee_sub_1709a8(o: u32, h: &mut dyn ClassHost) -> i32 {
    if h.ee_sub_132b10(o, 0xd) == 0 {
        if h.ee_sub_132b10(o, 0xb) != 0 { 1 } else { -1 }
    } else {
        0
    }
}

impl C2cf880 {
    /// slot 6: FUN_001709f8
    pub fn ee_sub_1709f8(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1a5270, obj);
        self.f_50.clear();
    }

    /// FUN_00170e00(obj, kind, id): append a new entry, returned for the payload writes
    pub fn ee_sub_170e00(&mut self, kind: i32, id: i32) -> &mut Entry {
        self.f_50.push(Entry { kind, id, ..Default::default() });
        self.f_50.last_mut().unwrap()
    }

    /// FUN_00170b40(obj, id): kind 0 node (method thunk 0x00105d58, field 595)
    pub fn ee_sub_170b40(&mut self, id: i32) {
        self.ee_sub_170e00(0, id);
    }

    /// Field of the method callee FUN_00170b40..FUN_00170dd0 (see `apply_field`).
    pub fn field_of_callee(callee: u32) -> Option<i16> {
        Some(match callee {
            0x170b40 => 595, 0x170b60 => 596, 0x170ba0 => 597, 0x170be0 => 598, 0x170c00 => 599, 0x170c20 => 600,
            0x170c50 => 601, 0x170c90 => 602, 0x170cd0 => 603, 0x170cf0 => 604, 0x170d10 => 605, 0x170d70 => 606,
            0x170d30 => 607, 0x170d50 => 608, 0x170da0 => 611, 0x170dd0 => 612,
            _ => return None,
        })
    }

    /// One load-time method call of field 595-612 (FUN_00170b40..FUN_00170dd0): `arg(i)` = word of argument i,
    /// `arg8(i)` = its first 8 bytes (kind-10 vector arguments).
    pub fn apply_field(&mut self, field: i16, arg: impl Fn(usize) -> u32, arg8: impl Fn(usize) -> [u32; 2]) {
        match field {
            595 => self.ee_sub_170b40(arg(0) as i32), // FUN_00170b40
            596 => self.ee_sub_170e00(1, -1).f_8 = arg8(0), // FUN_00170b60
            597 => self.ee_sub_170e00(2, -1).f_8 = arg8(0), // FUN_00170ba0
            598 => { self.ee_sub_170e00(3, arg(0) as i32); } // FUN_00170be0
            599 => { self.ee_sub_170e00(4, arg(0) as i32); } // FUN_00170c00
            600 => self.ee_sub_170e00(5, -1).f_8[0] = arg(0), // FUN_00170c20
            601 => {
                // FUN_00170c50(obj, a, b, c)
                let e = self.ee_sub_170e00(6, arg(0) as i32);
                e.f_8 = [arg(1), arg(2)];
            }
            602 => {
                // FUN_00170c90(b, obj, a, c)
                let (x, y, z) = (arg(0), arg(1), arg(2));
                let e = self.ee_sub_170e00(7, x as i32);
                e.f_10 = z;
                e.f_14 = y;
            }
            603 => { self.ee_sub_170e00(8, arg(0) as i32); } // FUN_00170cd0
            604 => { self.ee_sub_170e00(9, arg(0) as i32); } // FUN_00170cf0
            605 => { self.ee_sub_170e00(10, arg(0) as i32); } // FUN_00170d10
            606 => self.ee_sub_170e00(0xd, arg(0) as i32).f_8[0] = arg(1), // FUN_00170d70
            607 => { self.ee_sub_170e00(0xb, -1); } // FUN_00170d30
            608 => { self.ee_sub_170e00(0xc, -1); } // FUN_00170d50
            611 => self.ee_sub_170e00(0xe, arg(0) as i32).f_8[0] = arg(1), // FUN_00170da0
            612 => self.ee_sub_170e00(0xf, arg(0) as i32).f_8[0] = arg(1), // FUN_00170dd0
            _ => {} // 609 / 610: no handler
        }
    }

    /// Load-time method calls (fields 595-612). Arguments as FUN_00201360 hands them over: one word each
    /// (the first 8 bytes for the kind-10 vector arguments of 596 / 597).
    pub fn feed(&mut self, props: &[Prop]) {
        for p in props {
            if !(595..=612).contains(&p.field) {
                continue;
            }
            let a: Vec<Vec<u8>> = match &p.value {
                PropValue::Array(items) => items
                    .iter()
                    .map(|it| match it {
                        ArrayItem::One(_, b) => b.clone(),
                        ArrayItem::List(_, v) => v.first().cloned().unwrap_or_default(),
                    })
                    .collect(),
                PropValue::Scalar(b) => vec![b.clone()],
            };
            let arg = |i: usize| a.get(i).map(|b| w(b, 0)).unwrap_or(0);
            let arg8 = |i: usize| a.get(i).map(|b| [w(b, 0), w(b, 1)]).unwrap_or([0, 0]);
            self.apply_field(p.field, arg, arg8);
        }
    }

    /// slot 14: FUN_00170a28
    pub fn ee_sub_170a28(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1a5c48, obj);
        self.ee_sub_170a58(obj, h);
    }

    /// FUN_00170a58: kind-0 entries step the search root down a path, kind-6 entries resolve an object under
    /// it and hand ones without a class-0xd / 0xb ancestor to FUN_001e9c30.
    pub fn ee_sub_170a58(&mut self, obj: usize, h: &mut dyn ClassHost) {
        let mut root = obj as u32 + 1;
        for e in &self.f_50 {
            match e.kind {
                0 => root = h.ee_sub_15e938(if root != 0 { 4 } else { 5 }, e.id, root),
                6 => {
                    let x = h.ee_sub_15e938(4, e.id, root);
                    if x != 0 && ee_sub_1709a8(x, h) == -1 {
                        h.ee_sub_1e9c30(x);
                    }
                }
                _ => {}
            }
        }
    }
}
