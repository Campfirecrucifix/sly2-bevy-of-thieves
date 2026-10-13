//! WP-PUB2: property descriptor table 0x002ac880 (1449 x 0x28: +0 kind, +4 flags, +8 member offset /
//! thunk, +0x10 method fn, +0x18 count / nargs), read from SCUS_973.16 on first use.

use super::props::FieldDesc;
use crate::engine::disc::exe::exe;
use std::sync::LazyLock;

pub const FIELD_TABLE: u32 = 0x002a_c880;
pub const FIELD_STRIDE: u32 = 0x28;
pub const FIELD_COUNT: usize = 1449;

/// Word `k` (byte offset 4k) of field row `i`.
pub fn field_word(i: usize, k: u32) -> u32 {
    exe().w(FIELD_TABLE + i as u32 * FIELD_STRIDE + 4 * k)
}

/// Field index of a field descriptor address (None outside the table / misaligned).
pub fn field_index(addr: u32) -> Option<usize> {
    let o = addr.checked_sub(FIELD_TABLE)?;
    (o % FIELD_STRIDE == 0 && ((o / FIELD_STRIDE) as usize) < FIELD_COUNT).then_some((o / FIELD_STRIDE) as usize)
}

pub static FIELDS: LazyLock<Vec<FieldDesc>> =
    LazyLock::new(|| (0..FIELD_COUNT).map(|i| FieldDesc { kind: field_word(i, 0), flags: field_word(i, 1), count: field_word(i, 6) as i32 }).collect());
