#![allow(non_snake_case)]
//! Level transition state machine: FUN_0020e890 (level open + load, run once per frame from main()).
//! The request lives in the world struct: +0x3bf0 pending flag, +0x3bf4 request (0x4c bytes:
//! u32 flags, 64-byte descriptor, u32 spawn oid (+0x3c38), u32 spawn parent oid (+0x3c3c)),
//! +0x3c40 current level descriptor, +0x3c88/+0x3c8c last spawn used, +0x476c fade end time.
//! Disc open/LZ/parse go through WP-A `LevelLoader` and WP-B `World::load` (via `LevelHost`).

use super::clock::{self, Clock};
use crate::engine::disc::descriptor::Descriptor;
use crate::engine::disc::LevelLoader;
use crate::engine::world::{LoadOptions, World};

/// world+0x3bf4 (0x4c bytes).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LevelRequest {
    /// +0x00 flags: &0xcd != 0 = no fade carry-over; &0x40 = force FUN_0014c258
    pub flags: u32,
    /// +0x04 level descriptor (size field 0 = "no level": reset only)
    pub desc: Descriptor,
    /// +0x48 (world+0x3c38): spawn object oid
    pub spawn: i32,
    /// +0x4c (world+0x3c3c): spawn parent oid (-1 = any)
    pub spawn_parent: i32,
}

/// WP-J12: default spawn oid of a level request (0x189 = 393).
pub const SPAWN_DEFAULT: i32 = 0x189;

impl LevelRequest {
    pub fn new(desc: Descriptor) -> Self {
        // WP-J12: every FUN_0022b490 caller that does not carry a checkpoint / door spawn (boot
        // FUN_001a4e80, FUN_0014c140/0014c190, FUN_0015cf30, FUN_00162e38, FUN_001651d8, FUN_00165e38,
        // FUN_001b09a8, FUN_001dd8e0) requests spawn oid 0x189 with parent -1: the level's default
        // class-0x2a spawn object (oid 393, or one that aliased 0x189 through method 0x0010b8f8)
        Self { flags: 0, desc, spawn: SPAWN_DEFAULT, spawn_parent: -1 }
    }
    /// 0x4c-byte little-endian image (same layout as world+0x3bf4).
    pub fn to_bytes(&self) -> [u8; 0x4c] {
        let mut b = [0u8; 0x4c];
        b[0..4].copy_from_slice(&self.flags.to_le_bytes());
        b[4..0x44].copy_from_slice(&self.desc.0);
        b[0x44..0x48].copy_from_slice(&self.spawn.to_le_bytes());
        b[0x48..0x4c].copy_from_slice(&self.spawn_parent.to_le_bytes());
        b
    }
}

/// Transition-related world fields.
#[derive(Clone, Debug, Default)]
pub struct TransitionState {
    /// +0x3bf0
    pub pending: bool,
    /// +0x3bf4
    pub request: Option<LevelRequest>,
    /// +0x3c40: descriptor of the loaded level
    pub current: Option<Descriptor>,
    /// +0x3c88/+0x3c8c: spawn used by the last load (set only when the spawn object was found)
    pub last_spawn: Option<(i32, i32)>,
    /// +0x476c: fade end time (game clock)
    pub t_fade_end: f32,
}

impl TransitionState {
    /// Post a level change (what script/warp code does before the next main-loop iteration).
    pub fn request(&mut self, r: LevelRequest) {
        self.request = Some(r);
        self.pending = true;
    }
}

/// Engine services FUN_0020e890 calls (stubbable for tests).
pub trait LevelHost {
    /// FUN_00184d60 + FUN_0013da18 + object load: open, decompress, build the world. Err = open failed.
    fn load_level(&mut self, desc: &Descriptor) -> Result<(), String>;
    /// FUN_0020e068: free the old world (always called when the open succeeded or for a reset).
    fn clear_world(&mut self) {}
    /// FUN_001b3b08(0x2f66b0): play queued FMVs; true if any played.
    fn play_queued_movies(&mut self) -> bool {
        false
    }
    /// FUN_0014c258: cancel the HUD/music cue 0x1f.
    fn cancel_cue(&mut self) {}
    /// FUN_00227fe8: find the spawn object (class 0x2a) `spawn` (under `parent` if != -1).
    fn find_spawn(&mut self, _spawn: i32, _parent: i32) -> bool {
        false
    }
    /// FUN_001618f8(fade): start a fade-in of `fade` seconds.
    fn fade_in(&mut self, _fade: f32) {}
}

#[derive(Clone, Debug, PartialEq)]
pub enum TransitionOutcome {
    /// +0x3bf0 was clear
    Idle,
    /// level descriptor could not be opened; request dropped (pending already cleared)
    OpenFailed(String),
    /// level loaded (or reset when size == 0)
    Loaded { same_level: bool, movies: bool, spawn_found: bool, fade: f32, reset_only: bool },
}

/// FUN_0020e890.
pub fn ee_str_oidPoCurrent_20e890(st: &mut TransitionState, clk: &mut Clock, host: &mut impl LevelHost) -> TransitionOutcome {
    // 0x0020e890
    if !st.pending {
        return TransitionOutcome::Idle;
    }
    let Some(req) = st.request else {
        st.pending = false;
        return TransitionOutcome::Idle;
    };
    st.pending = false;
    // fade carry-over: (int)(t_fade_end - t), only when positive (int truncation quirk kept)
    let fade = if req.flags & 0xcd == 0 {
        let f = st.t_fade_end - clk.t;
        if 0.0 < f { (f as i32) as f32 } else { 0.0 }
    } else {
        0.0
    };
    let size = req.desc.size();
    let reset_only = size == 0;
    // same level: descriptor ident (w2^w14) equals the current one
    let same_level = match &st.current {
        Some(c) => req.desc.same_file(c),
        None => false,
    };
    if !reset_only {
        if let Err(e) = host.load_level(&req.desc) {
            return TransitionOutcome::OpenFailed(e);
        }
    }
    // FUN_0020e068 / clock reset FUN_0014d068 / FUN_00187078(8) / memset world / FUN_0020d9d0
    host.clear_world();
    clock::ee_sub_14d068(clk);
    let movies = host.play_queued_movies();
    st.current = Some(req.desc);
    st.request = Some(req);
    if 0.0 < fade {
        host.fade_in(fade);
    }
    if movies || !same_level || req.flags & 0x40 != 0 {
        host.cancel_cue();
    }
    let spawn_found = host.find_spawn(req.spawn, req.spawn_parent);
    if spawn_found {
        st.last_spawn = Some((req.spawn, req.spawn_parent));
    }
    TransitionOutcome::Loaded { same_level, movies, spawn_found, fade, reset_only }
}

/// `LevelHost` over the real disc + world loaders (blocking: run from an async task or a tool).
pub struct DiscLevelHost<'a> {
    pub loader: &'a LevelLoader,
    pub options: LoadOptions,
    pub world: Option<World>,
}

impl LevelHost for DiscLevelHost<'_> {
    fn load_level(&mut self, desc: &Descriptor) -> Result<(), String> {
        let lv = self.loader.load_blocking(*desc).map_err(|e| e.to_string())?;
        let w = World::load(&lv, self.options).map_err(|e| e.to_string())?;
        self.world = Some(w);
        Ok(())
    }
    fn find_spawn(&mut self, spawn: i32, parent: i32) -> bool {
        // FUN_00227fe8 (WP-J17: the port, parent subtree + last match; the +0x16c alias lists are filled
        // by the level scripts, which this host does not run)
        self.world.as_ref().is_some_and(|w| super::spawn::ee_sub_227fe8(w, &super::spawn::SpawnAliases::default(), spawn, parent).is_some())
    }
}
