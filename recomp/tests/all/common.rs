//! WP-T1: helpers shared by every test module of the single `all` test binary.
//! - [`root`] / [`levels`]: repo root (the ISO lives there) + every disc level.
//! - [`load_blocking`]: drop-in for `LevelLoader::load_blocking` with a level cache: in-process (OnceLock, one
//!   decompression per level per test run) + files in <temp>/sly2_testcache/<iso size>_<iso mtime>/ (decompressed
//!   bytes; the header is re-parsed). Mod-overridden levels bypass the cache.
#![allow(dead_code)]
use sly2_recomp::engine::disc::descriptor::Descriptor;
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::level_file;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::{LevelLoader, LoadedLevel};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

pub const ISO_NAME: &str = "Sly 2 - Band of Thieves (USA).iso";

/// Repo root (holds the ISO).
pub fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

/// Every level of the disc: the level table embedded in the boot level (FUN_0020f318) lists every file; its
/// non-empty rows are the 43 level files (same set as the dev analysis DB), sorted by LSN (ISO only, no analysis DB).
pub fn levels() -> (LevelLoader, Vec<Descriptor>) {
    let iso = Arc::new(IsoImage::open(root().join(ISO_NAME)).expect("ISO"));
    let elf = iso.read_file("SCUS_973.16").unwrap();
    let table = sly2_recomp::engine::disc::descriptor::static_table(&elf).unwrap();
    let boot = *table.iter().skip(22).find(|d| d.size() != 0).unwrap();
    let loader = LevelLoader::new(iso, ModOverlay::default());
    let lt = loader.load_blocking(boot).unwrap().header.level_table;
    let mut descs: Vec<Descriptor> = vec![];
    for e in lt.entries {
        let d = e.desc;
        if d.size() != 0 && !descs.iter().any(|x| x.same_file(&d)) {
            descs.push(d);
        }
    }
    descs.sort_by_key(|d| d.lsn());
    (loader, descs)
}

/// File-cache directory keyed by ISO size + mtime (None = no ISO metadata / cache dir unusable).
fn cache_dir() -> Option<&'static PathBuf> {
    static DIR: OnceLock<Option<PathBuf>> = OnceLock::new();
    DIR.get_or_init(|| {
        let iso = std::env::var_os("SLY2_ISO").map(PathBuf::from).unwrap_or_else(|| root().join(ISO_NAME));
        let m = std::fs::metadata(iso).ok()?;
        let mtime = m.modified().ok()?.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs();
        let d = std::env::temp_dir().join(format!("sly2_testcache/{}_{}", m.len(), mtime));
        std::fs::create_dir_all(&d).ok()?;
        Some(d)
    })
    .as_ref()
}

type Key = (Vec<u8>, u8, u64);

/// Decompressed level bytes, cached in-process and on disk.
fn cached_data(loader: &LevelLoader, desc: Descriptor) -> std::io::Result<Arc<Vec<u8>>> {
    static MEM: OnceLock<Mutex<HashMap<Key, Arc<OnceLock<Arc<Vec<u8>>>>>>> = OnceLock::new();
    let key: Key = (desc.0.to_vec(), loader.language, loader.iso.sectors);
    let cell = MEM.get_or_init(Default::default).lock().unwrap().entry(key).or_default().clone();
    if let Some(d) = cell.get() {
        return Ok(d.clone());
    }
    let loc = desc.location().ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "descriptor size 0"))?;
    let file = cache_dir().map(|d| d.join(format!("{}_{}.bin", loc.lsn, loc.size)));
    let data = match file.as_ref().and_then(|f| std::fs::read(f).ok()) {
        Some(d) => d,
        None => {
            let d = level_file::open_level_stream(loader.iso.clone(), &desc).unwrap().read_to_end();
            if let Some(f) = &file {
                // write-then-rename so a concurrent reader never sees a partial file
                let tmp = f.with_extension(format!("tmp{}", std::process::id()));
                if std::fs::write(&tmp, &d).is_ok() && std::fs::rename(&tmp, f).is_err() {
                    let _ = std::fs::remove_file(&tmp);
                }
            }
            d
        }
    };
    Ok(cell.get_or_init(|| Arc::new(data)).clone())
}

/// Drop-in for `loader.load_blocking(desc)` backed by the level cache (same result: decompressed bytes + header).
pub fn load_blocking(loader: &LevelLoader, desc: Descriptor) -> std::io::Result<LoadedLevel> {
    let Some(loc) = desc.location() else { return loader.load_blocking(desc) };
    if loader.mods.level_override(loc.lsn)?.is_some() {
        return loader.load_blocking(desc);
    }
    let data = cached_data(loader, desc)?;
    let header = level_file::ee_str_spaprops_dbp_13da18(&mut level_file::memory_stream((*data).clone()), loader.language);
    Ok(LoadedLevel { desc, data, header, modded: false })
}

/// Process-wide lock for tests that touch the engine's process globals (flow::POSTED / STATIC_TABLE,
/// boot::RESTART): one test binary = one process since WP-T1, so modules no longer get a private copy.
pub static FLOW: Mutex<()> = Mutex::new(());

pub fn flow_lock() -> std::sync::MutexGuard<'static, ()> {
    FLOW.lock().unwrap_or_else(|e| e.into_inner())
}
