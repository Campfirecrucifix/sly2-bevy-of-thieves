//! WP-PUB2: the game executable SCUS_973.16, read from the user's ISO at run time.
//!
//! The engine's static descriptor tables (property fields, Splice methods, classes, BIFs, IOP sound
//! tables, ...) are parsed from it on first use instead of being embedded in the source. The ISO is the
//! one passed to [`set_iso_path`] (the game binary does this with the configured path), else the first
//! of: `$SLY2_ISO`, `<exe dir>/<ISO_NAME>`, `<cwd or an ancestor>/<ISO_NAME>`,
//! `<crate dir or an ancestor>/<ISO_NAME>`.

use super::descriptor::elf_bytes;
use super::iso::IsoImage;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub const ISO_NAME: &str = "Sly 2 - Band of Thieves (USA).iso";

/// An ELF image (the EE executable or an IOP module embedded in it), addressed by virtual address.
pub struct Image {
    data: Vec<u8>,
}

impl Image {
    pub fn new(data: Vec<u8>) -> Self {
        Image { data }
    }
    pub fn bytes(&self, vaddr: u32, len: usize) -> Option<&[u8]> {
        elf_bytes(&self.data, vaddr, len)
    }
    /// u32 at `vaddr`; 0 outside the loaded segments (like the original generator scripts).
    pub fn w(&self, vaddr: u32) -> u32 {
        self.bytes(vaddr, 4).map_or(0, |b| u32::from_le_bytes(b.try_into().unwrap()))
    }
    pub fn h(&self, vaddr: u32) -> u16 {
        self.bytes(vaddr, 2).map_or(0, |b| u16::from_le_bytes(b.try_into().unwrap()))
    }
    /// The IOP module (IRX, an ELF) embedded at `vaddr` (`size` bytes).
    pub fn module(&self, vaddr: u32, size: usize) -> Option<Image> {
        let b = self.bytes(vaddr, size)?;
        (b.get(0..4)? == b"\x7fELF").then(|| Image::new(b.to_vec()))
    }
}

static ISO_PATH: OnceLock<PathBuf> = OnceLock::new();
static EXE: OnceLock<Image> = OnceLock::new();

/// Use this ISO for every table read from the disc (call before the first table access).
pub fn set_iso_path(p: impl Into<PathBuf>) {
    let _ = ISO_PATH.set(p.into());
}

fn search_up(start: &Path) -> Option<PathBuf> {
    start.ancestors().map(|d| d.join(ISO_NAME)).find(|p| p.is_file())
}

/// The ISO the tables are read from (see the module doc).
pub fn iso_path() -> Option<PathBuf> {
    if let Some(p) = ISO_PATH.get() {
        return Some(p.clone());
    }
    if let Some(p) = std::env::var_os("SLY2_ISO").map(PathBuf::from).filter(|p| p.is_file()) {
        return Some(p);
    }
    let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(Path::to_path_buf));
    exe_dir
        .and_then(|d| Some(d.join(ISO_NAME)).filter(|p| p.is_file()))
        .or_else(|| std::env::current_dir().ok().and_then(|d| search_up(&d)))
        .or_else(|| search_up(Path::new(env!("CARGO_MANIFEST_DIR"))))
}

/// SCUS_973.16 from the ISO. Panics when no ISO is found: the engine cannot run without the game data.
pub fn exe() -> &'static Image {
    EXE.get_or_init(|| {
        let p = iso_path().unwrap_or_else(|| panic!("game ISO not found: set SLY2_ISO or put '{ISO_NAME}' next to the executable"));
        let elf = IsoImage::open(&p).and_then(|iso| iso.read_file("SCUS_973.16"));
        Image::new(elf.unwrap_or_else(|e| panic!("{}: cannot read SCUS_973.16: {e}", p.display())))
    })
}

/// libsd.irx embedded in SCUS_973.16 (loaded from memory by the original's module loader).
pub const LIBSD_IRX: (u32, usize) = (0x0049_8040, 28736);
/// 989snd.irx embedded in SCUS_973.16.
pub const SND989_IRX: (u32, usize) = (0x004b_3c00, 178240);

pub fn libsd() -> &'static Image {
    static M: OnceLock<Image> = OnceLock::new();
    M.get_or_init(|| exe().module(LIBSD_IRX.0, LIBSD_IRX.1).expect("libsd.irx in SCUS_973.16"))
}

pub fn snd989() -> &'static Image {
    static M: OnceLock<Image> = OnceLock::new();
    M.get_or_init(|| exe().module(SND989_IRX.0, SND989_IRX.1).expect("989snd.irx in SCUS_973.16"))
}

/// `n` rows of {u32, s32, u32} at `vaddr` (FUN_001a64e8 binding tables: flags, oid, member).
pub fn rows3(vaddr: u32, n: usize) -> Vec<(u32, i32, u32)> {
    let x = exe();
    (0..n as u32).map(|i| (x.w(vaddr + i * 12), x.w(vaddr + i * 12 + 4) as i32, x.w(vaddr + i * 12 + 8))).collect()
}
