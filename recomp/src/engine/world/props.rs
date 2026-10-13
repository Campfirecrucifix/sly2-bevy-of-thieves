//! Property lists: FUN_0013f810 / FUN_0013e818 / FUN_0013e690.
//!
//! A property list is `{ s16 field; value }*` terminated by a negative s16. `field` indexes the
//! descriptor table 0x002ac880 (1449 x 0x28 bytes: +0 kind, +4 flags, +0x10/+0x18 target/setter
//! data; +0x18 doubles as the element count for array fields). The original writes each value
//! straight into the object (member offset) or calls a setter; here the raw little-endian value
//! bytes are kept per field (the targets are per-class struct layouts, ported with the classes).

use super::field_table::FIELDS;
use crate::engine::disc::lz::BinaryInputStream;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FieldDesc {
    pub kind: u32,
    pub flags: u32,
    pub count: i32,
}

pub fn field(i: i16) -> Option<&'static FieldDesc> {
    FIELDS.get(i as usize)
}

#[derive(Clone, Debug, PartialEq)]
pub enum PropValue {
    /// direct / setter path: raw bytes of `kind`'s value (see [`value_size`])
    Scalar(Vec<u8>),
    /// flags & 0x400 (array path): `count` elements, each a u16 element kind + value
    /// (kind & 0x8000: u8 n then n values of kind & 0x7fff, combined by FUN_00200da0)
    Array(Vec<ArrayItem>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum ArrayItem {
    One(u16, Vec<u8>),
    List(u16, Vec<Vec<u8>>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Prop {
    pub field: i16,
    pub value: PropValue,
}

/// Bytes FUN_0013e818 reads for a scalar of `kind` (identical in the direct (flags & 0x90)
/// and the setter path). 0x16 is variable: u16 n + n u16 (handled by the caller).
fn value_size(kind: u32) -> usize {
    match kind {
        0 => 1,
        1 | 5 | 6 | 0xd => 4,
        2 | 4 | 9 | 0xb => 12, // 4: 3 floats expanded to a matrix by FUN_0011b570
        3 | 0xc => 16,
        7 | 0xe | 0xf | 0x10 | 0x11 | 0x13 | 0x14 | 0x15 | 0x17 => 2,
        10 => 8,
        // default: (kind & ~0x8000) >> 12 == 1 (u32) or 2 (s32 oid, resolved later via
        // FUN_0013e590/FUN_0013e5e0); anything else (8, 0x12, ...) reads nothing.
        _ => match (kind & 0xffff7fff) >> 12 {
            1 | 2 => 4,
            _ => 0,
        },
    }
}

/// FUN_0013e690: one array element of element-kind `k` (u16 from the stream).
pub fn ee_sub_13e690(k: u32, s: &mut BinaryInputStream) -> Vec<u8> {
    // 0x0013e690
    let n = match k {
        0 => 1,
        1 | 5 | 6 => 4,
        2 | 9 => 12,
        10 => 8,
        7 | 0xe | 0xf | 0x10 | 0x11 | 0x13 | 0x14 | 0x15 | 0x17 => 2,
        // default: (k & ~0x8000) >> 12 == 1 -> u32, else nil (0x24) without reading
        _ if (k & 0xffff7fff) >> 12 == 1 => 4,
        _ => 0,
    };
    let mut v = vec![0u8; n];
    s.read(n, Some(&mut v));
    v
}

/// FUN_0013e818: read one field value. `flags` = descriptor flags & 0x2580 (as passed by
/// FUN_0013f810).
pub fn ee_sub_13e818(f: &FieldDesc, flags: u32, s: &mut BinaryInputStream) -> PropValue {
    // 0x0013e818
    if flags & 0x401 != 0 {
        let mut items = Vec::new();
        for _ in 0..f.count.max(0) {
            let k = s.read_u16();
            if k & 0x8000 == 0 {
                items.push(ArrayItem::One(k, ee_sub_13e690(k as u32, s)));
            } else {
                let k2 = (k as u32 & 0xffff7fff) as u16;
                let n = s.read_u8();
                items.push(ArrayItem::List(k2, (0..n).map(|_| ee_sub_13e690(k2 as u32, s)).collect()));
            }
        }
        return PropValue::Array(items);
    }
    if f.kind == 0x16 {
        // u16 count + count u16 (allocated copy)
        let n = s.read_u16() as usize;
        let mut v = vec![0u8; 2 + 2 * n];
        v[..2].copy_from_slice(&(n as u16).to_le_bytes());
        s.read(2 * n, Some(&mut v[2..]));
        return PropValue::Scalar(v);
    }
    let n = value_size(f.kind);
    let mut v = vec![0u8; n];
    s.read(n, Some(&mut v));
    PropValue::Scalar(v)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PropError {
    pub field: i16,
    pub offset: u32,
}

/// FUN_0013f810: property list until a negative field id. An index outside the descriptor
/// table is an error (the original would read garbage descriptors).
pub fn ee_sub_13f810(s: &mut BinaryInputStream) -> Result<Vec<Prop>, PropError> {
    // 0x0013f810
    let mut out = Vec::new();
    loop {
        let off = s.tell();
        let i = s.read_s16();
        if i < 0 {
            return Ok(out);
        }
        let f = field(i).ok_or(PropError { field: i, offset: off })?;
        let value = ee_sub_13e818(f, f.flags & 0x2580, s);
        out.push(Prop { field: i, value });
        if s.at_eof() {
            return Err(PropError { field: i, offset: off });
        }
    }
}
