//! WP-J3: `--level` / `--table` guard. The static table 0x2a3268 mixes movies (MPEG-PS, e.g. row 24),
//! data files and levels; the original only ever hands level rows to FUN_0020e890 (movies go through
//! the FMV queue 0x2f66b0). Starting the game on a non-level row is refused with a clear error.

use crate::engine::disc::descriptor::Descriptor;
use crate::engine::disc::iso::{IsoImage, SectorSource};
use crate::engine::disc::LevelLoader;
use crate::engine::world::{LoadOptions, World};

/// MPEG-PS pack start code (00 00 01 BA): first bytes of every movie file on the disc.
pub const MPEG_PS_PACK: [u8; 4] = [0, 0, 1, 0xba];

#[derive(Debug, Clone, PartialEq)]
pub enum NotALevel {
    EmptyRow,
    Movie { lsn: u32, size: u32 },
    Unreadable(String),
    NoWorld(String),
}

impl std::fmt::Display for NotALevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotALevel::EmptyRow => write!(f, "empty row (descriptor size 0)"),
            NotALevel::Movie { lsn, size } => write!(f, "a movie (MPEG-PS at lsn {lsn}, {size} bytes), not a level; movies play through the FMV queue"),
            NotALevel::Unreadable(e) => write!(f, "not readable as a level: {e}"),
            NotALevel::NoWorld(e) => write!(f, "not a level (world load failed: {e})"),
        }
    }
}

/// Cheap test on the first sector (no decompression).
pub fn is_movie(iso: &IsoImage, d: &Descriptor) -> bool {
    iso.read_sectors(d.lsn(), 1).is_ok_and(|s| s.starts_with(&MPEG_PS_PACK))
}

/// Full check: not empty, not a movie, decompresses, and builds a world. Blocking (startup only).
pub fn check_level_row(loader: &LevelLoader, d: Descriptor) -> Result<(), NotALevel> {
    if d.size() == 0 {
        return Err(NotALevel::EmptyRow);
    }
    if is_movie(&loader.iso, &d) {
        return Err(NotALevel::Movie { lsn: d.lsn(), size: d.size() });
    }
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let lv = loader.load_blocking(d).map_err(|e| NotALevel::Unreadable(e.to_string()))?;
        World::load(&lv, LoadOptions::default()).map(|_| ()).map_err(|e| NotALevel::NoWorld(e.to_string()))
    }));
    r.unwrap_or_else(|_| Err(NotALevel::Unreadable("decoder panicked".into())))
}
