//! FMV (PSS/MPEG2 via libmpeg + IPU). Original: movie queue at 0x2f66b0 played by FUN_001b3b08 during a
//! level transition (FUN_0020e890); each movie by FUN_001b3bd0 (decoder FUN_001b3ec0/FUN_001b4058,
//! sceMpeg in FUN_0024b540/FUN_0024acd8). The decoder is REPLACED by the `FmvPlayer` interface; the
//! default `SkipFmvPlayer` logs and skips each movie (explicit, never silent); the game uses
//! `fmv_app::FmvPlayback` (WP-J16: PSS demux + MPEG-2 / ADPCM decode in `fmv_pss`).
//!
//! Kept 1:1: queue layout (s32 n; n x 64-byte descriptors at +4; +0x108 replay flag), the free-memory
//! guard (FUN_001ae150(1) < 0x41fc40 drops the queue), the skip rule (buttons 0x91f, only for movies
//! 0x2a3968/0x2a39a8 once DAT_002f68f8 "seen" is set, or always when DAT_002c0268), and that playing
//! 0x2a39a8 sets DAT_002f68f8.

use crate::engine::disc::descriptor::Descriptor;

/// Pad mask that skips a movie (FUN_00187d80(0x2dfc00, 0x91f)).
pub const SKIP_BUTTONS: u16 = 0x91f;
/// FUN_001b3b08 minimum free memory.
pub const MIN_FREE_MEM: u32 = 0x41fc40;

/// Playback backend (replacement for the PSS decoder loop of FUN_001b3bd0).
pub trait FmvPlayer {
    /// Play one movie to the end (or until skipped). Returns false if it could not be played.
    fn play(&mut self, desc: &Descriptor, skippable: bool) -> bool;
}

/// Logs and skips every movie (no decoder yet).
#[derive(Default, Debug)]
pub struct SkipFmvPlayer {
    pub skipped: Vec<Descriptor>,
}

impl FmvPlayer for SkipFmvPlayer {
    fn play(&mut self, desc: &Descriptor, skippable: bool) -> bool {
        log::warn!(
            "FMV not implemented: skipping movie lsn {} size {} (skippable {skippable})",
            desc.lsn(),
            desc.size()
        );
        self.skipped.push(*desc);
        true
    }
}

/// 0x2f66b0 movie queue + the globals FUN_001b3bd0 reads.
#[derive(Clone, Debug, Default)]
pub struct MovieQueue {
    /// +0 n, +4.. descriptors
    pub movies: Vec<Descriptor>,
    /// DAT_002f68f8: the intro movie has been seen
    pub seen: bool,
    /// DAT_002c0268: every movie skippable
    pub always_skippable: bool,
    /// the two special descriptors 0x2a3968 / 0x2a39a8 (from the ELF static table; None if unknown)
    pub special: [Option<Descriptor>; 2],
}

impl MovieQueue {
    /// FUN_001b3b08. Returns true when movies were played (queue emptied either way).
    pub fn ee_sub_1b3b08(&mut self, free_mem: u32, player: &mut impl FmvPlayer) -> bool {
        // 0x001b3b08
        if self.movies.is_empty() {
            return false;
        }
        if free_mem < MIN_FREE_MEM {
            self.movies.clear();
            return false;
        }
        for d in std::mem::take(&mut self.movies) {
            self.ee_sub_1b3bd0(&d, player);
        }
        true
    }

    /// FUN_001b3bd0 (one movie; the replay loop on +0x108 collapses because the player blocks).
    pub fn ee_sub_1b3bd0(&mut self, d: &Descriptor, player: &mut impl FmvPlayer) {
        // 0x001b3bd0
        if d.size() == 0 {
            return;
        }
        let is = |k: usize| self.special[k].is_some_and(|s| s.same_file(d));
        let skippable = ((is(0) || is(1)) && self.seen) || self.always_skippable;
        player.play(d, skippable);
        if is(1) {
            self.seen = true;
        }
    }
}
