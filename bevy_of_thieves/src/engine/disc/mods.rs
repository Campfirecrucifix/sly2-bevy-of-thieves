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

    /// WIRE-C3: text overrides keyed by the ORIGINAL string id (level header strings / table 0x3e1c20):
    /// every `mods/<mod>/text/*.toml` in load order (later mods win), lines `<id> = "text"` (id decimal or
    /// 0x hex; `#` comments and `[section]` headers ignored; escapes backslash-n, backslash-quote,
    /// backslash-backslash). Text is stored
    /// as the game's 8-bit bytes (chars above U+00FF become '?').
    pub fn text_overrides(&self) -> std::collections::BTreeMap<i32, Vec<u8>> {
        let mut out = std::collections::BTreeMap::new();
        for m in &self.order.mods {
            let Ok(rd) = std::fs::read_dir(m.join("text")) else { continue };
            let mut files: Vec<PathBuf> = rd.filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.extension().is_some_and(|x| x == "toml")).collect();
            files.sort();
            for f in files {
                if let Ok(src) = std::fs::read_to_string(&f) {
                    out.extend(parse_text_toml(&src));
                }
            }
        }
        out
    }
}

/// `<id> = "text"` lines of a mod text file (see `ModOverlay::text_overrides`).
pub fn parse_text_toml(src: &str) -> Vec<(i32, Vec<u8>)> {
    let mut v = Vec::new();
    for line in src.lines() {
        let l = line.trim();
        if l.is_empty() || l.starts_with('#') || l.starts_with('[') {
            continue;
        }
        let Some((k, rest)) = l.split_once('=') else { continue };
        let k = k.trim().trim_matches('"');
        let id = match k.strip_prefix("0x") {
            Some(h) => i32::from_str_radix(h, 16).ok(),
            None => k.parse().ok(),
        };
        let Some(id) = id else { continue };
        let Some(body) = rest.trim().strip_prefix('"') else { continue };
        let mut s = Vec::new();
        let mut it = body.chars();
        while let Some(c) = it.next() {
            match c {
                '"' => break,
                '\\' => match it.next() {
                    Some('n') => s.push(b'\n'),
                    Some(c) => s.push(if (c as u32) < 256 { c as u8 } else { b'?' }),
                    None => break,
                },
                c => s.push(if (c as u32) < 256 { c as u8 } else { b'?' }),
            }
        }
        v.push((id, s));
    }
    v
}
