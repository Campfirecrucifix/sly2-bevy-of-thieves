//! Mod overlay: before a disc file or level is read, `mods/<mod>/<same path>` is checked in load
//! order (sly2_db::final_build::ModLoadOrder); later mods win. No mods = byte-identical disc data.
//!
//! Paths: ISO files by their ISO path ("SCUS_973.16", "IRX/FOO.IRX"); levels (which have no ISO
//! name, only a descriptor) as "levels/<lsn:07>.bin" = the decompressed level bytes.

use super::iso::IsoImage;
use std::io;
use std::path::{Path, PathBuf};
pub use sly2_db::final_build::ModLoadOrder;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ModOverlay {
    pub order: ModLoadOrder,
}

impl ModOverlay {
    /// Blocking directory scan: call at startup / in an async task.
    pub fn scan(mods_dir: &Path) -> io::Result<Self> {
        Ok(Self { order: ModLoadOrder::scan(mods_dir)? })
    }

    fn norm(rel: &str) -> PathBuf {
        rel.strip_prefix("cdrom0:").unwrap_or(rel).split(';').next().unwrap_or("").split(['/', '\\']).filter(|s| !s.is_empty()).collect()
    }

    /// Mod file overriding `rel`, last mod in load order first.
    pub fn resolve(&self, rel: &str) -> Option<PathBuf> {
        let r = Self::norm(rel);
        self.order.mods.iter().rev().map(|m| m.join(&r)).find(|p| p.is_file())
    }

    pub fn level_path(lsn: u32) -> String {
        format!("levels/{lsn:07}.bin")
    }

    /// Read an ISO file through the overlay.
    pub fn read_file(&self, iso: &IsoImage, path: &str) -> io::Result<Vec<u8>> {
        match self.resolve(path) {
            Some(p) => std::fs::read(p),
            None => iso.read_file(path),
        }
    }

    /// Decompressed replacement for a level, if a mod provides one.
    pub fn level_override(&self, lsn: u32) -> io::Result<Option<Vec<u8>>> {
        self.resolve(&Self::level_path(lsn)).map(std::fs::read).transpose()
    }
}
