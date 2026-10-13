//! WP-A: disc + level loading. See ENGINE_PORT.md.
//! Core is pure Rust (tests run without Bevy); `LevelLoader` is a Bevy Resource, and with the
//! `game` feature levels load on the AsyncComputeTaskPool (no blocking I/O inside systems).

pub mod descriptor;
// WP-PUB2: SCUS_973.16 from the ISO = source of the static descriptor tables
pub mod exe;
pub mod exe_scan;
pub mod iso;
pub mod level_file;
pub mod lz;
pub mod mods;
// WP-R: per-sound WAV overrides (mods/<mod>/sounds/<bank lsn>/<i>.wav)
pub mod mods_sound;

use bevy_ecs::prelude::Resource;
use descriptor::Descriptor;
use iso::IsoImage;
use level_file::LevelHeader;
use mods::ModOverlay;
use std::io;
use std::sync::Arc;

/// Result of a level load: decompressed bytes + everything parsed before object instantiation.
#[derive(Clone, Debug)]
pub struct LoadedLevel {
    pub desc: Descriptor,
    pub data: Arc<Vec<u8>>,
    pub header: LevelHeader,
    /// true when a mod replaced the level
    pub modded: bool,
}

#[derive(Resource, Clone)]
pub struct LevelLoader {
    pub iso: Arc<IsoImage>,
    pub mods: ModOverlay,
    /// DAT_002e1e54
    pub language: u8,
}

impl LevelLoader {
    pub fn new(iso: Arc<IsoImage>, mods: ModOverlay) -> Self {
        Self { iso, mods, language: 0 }
    }

    /// Blocking load (FUN_0020e890 stream setup + full decompression + FUN_0013da18 header).
    /// Call from an async task / tool, never from a system.
    pub fn load_blocking(&self, desc: Descriptor) -> io::Result<LoadedLevel> {
        let loc = desc.location().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "descriptor size 0"))?;
        let (data, modded) = match self.mods.level_override(loc.lsn)? {
            Some(d) => (d, true),
            None => {
                let mut s = level_file::open_level_stream(self.iso.clone(), &desc).unwrap();
                (s.read_to_end(), false)
            }
        };
        let header = level_file::ee_str_spaprops_dbp_13da18(&mut level_file::memory_stream(data.clone()), self.language);
        Ok(LoadedLevel { desc, data: Arc::new(data), header, modded })
    }
}

#[cfg(feature = "game")]
pub use game::*;

#[cfg(feature = "game")]
mod game {
    use super::*;
    use bevy::prelude::*;
    use bevy::tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task};

    /// In-flight load; spawn with [`LevelLoader::start`] and add [`poll_level_loads`].
    #[derive(Component)]
    pub struct LevelLoadTask(pub Task<io::Result<LoadedLevel>>);

    /// Emitted (as a Resource) when a load finishes.
    #[derive(Resource)]
    pub struct LevelLoadResult(pub io::Result<LoadedLevel>);

    impl LevelLoader {
        pub fn start(&self, desc: Descriptor) -> LevelLoadTask {
            let me = self.clone();
            LevelLoadTask(AsyncComputeTaskPool::get().spawn(async move { me.load_blocking(desc) }))
        }
    }

    pub fn poll_level_loads(mut commands: Commands, mut q: Query<(Entity, &mut LevelLoadTask)>) {
        for (e, mut t) in &mut q {
            if let Some(r) = block_on(future::poll_once(&mut t.0)) {
                commands.insert_resource(LevelLoadResult(r));
                commands.entity(e).despawn();
            }
        }
    }
}
