//! WP-PUB2 (was the WP-J5 generated table): every (class, symid) -> field row of the per-class tables
//! 0x00279ad0 (199 x {count, ptr}; entries {symid, field*}) as FUN_001ffcf8 resolves them, + field +8
//! (member offset, flags & 2). Read from SCUS_973.16 on first use.

use crate::engine::disc::exe::exe;
use crate::engine::world::field_table::{field_index, field_word, FIELD_COUNT};
use std::sync::LazyLock;

pub const CLASS_SYMID_TABLES: u32 = 0x0027_9ad0;
pub const FIELD_BINDING_COUNT: usize = 12001;

/// (class, symid, field descriptor address) for every distinct symid of every class table, in (class,
/// symid) order, resolved by replaying FUN_001ffcf8's binary search (duplicated symids keep the row the
/// original finds). 0 = not found.
pub fn class_symid_rows() -> Vec<(u16, u32, u32)> {
    let x = exe();
    let mut out = Vec::new();
    for k in 0..199u32 {
        let cnt = x.w(CLASS_SYMID_TABLES + k * 8) as i64;
        let p = x.w(CLASS_SYMID_TABLES + k * 8 + 4);
        let full: Vec<(u32, u32)> = (0..cnt as u32).map(|j| (x.w(p + j * 8), x.w(p + j * 8 + 4))).collect();
        // 0x001ffcf8
        let search = |sym: u32| -> u32 {
            let (mut lo, mut hi) = (0i64, cnt - 1);
            let mut mid = hi;
            while lo <= hi {
                mid >>= 1;
                let e = full[mid as usize];
                if e.0 == sym {
                    return e.1;
                }
                if sym < e.0 { hi = mid - 1 } else { lo = mid + 1 }
                mid = lo + hi;
            }
            0
        };
        let mut syms: Vec<u32> = full.iter().map(|e| e.0).collect();
        syms.sort_unstable();
        syms.dedup();
        out.extend(syms.into_iter().map(|s| (k as u16, s, search(s))));
    }
    out
}

/// (class id, symid, field index into world::field_table::FIELDS), sorted by (class, symid).
pub static FIELD_BINDINGS: LazyLock<Vec<(u16, u16, u16)>> = LazyLock::new(|| {
    let v: Vec<_> = class_symid_rows().into_iter().filter_map(|(k, s, f)| Some((k, s as u16, field_index(f)? as u16))).collect();
    assert_eq!(v.len(), FIELD_BINDING_COUNT, "SCUS_973.16 field bindings");
    v
});

/// field +8 per field index (member offset for flags & 2 fields).
pub static FIELD_MEMBER: LazyLock<Vec<u32>> = LazyLock::new(|| (0..FIELD_COUNT).map(|i| field_word(i, 2)).collect());
