//! WP-PUB2: IOP sound tables, read on first use from the IRX modules embedded in SCUS_973.16.

use crate::engine::disc::exe::{libsd, snd989, Image};
use std::sync::LazyLock;

fn h16(m: &Image, a: u32, n: u32) -> Vec<u16> {
    (0..n).map(|i| m.h(a + 2 * i)).collect()
}

/// 989snd pan table (PTR_DAT_0001ebe4 -> 0x1ebe8): 180 x {left, right}, 0..90 deg in 0.5 deg steps, 0x3fff = full.
pub static PAN_TABLE: LazyLock<Vec<[i16; 2]>> =
    LazyLock::new(|| h16(snd989(), 0x1ebe8, 360).chunks(2).map(|c| [c[0] as i16, c[1] as i16]).collect());

/// libsd sceSdNote2Pitch semitone table (libsd.irx 0x51a0).
pub static NOTE_TABLE: LazyLock<Vec<u16>> = LazyLock::new(|| h16(libsd(), 0x51a0, 12));

/// libsd sceSdNote2Pitch fine table (libsd.irx 0x51b8): 128 steps of 1/128 semitone.
pub static FINE_TABLE: LazyLock<Vec<u16>> = LazyLock::new(|| h16(libsd(), 0x51b8, 128));
