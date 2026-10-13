//! WP-J: main loop, clock, level transitions, save game (mcman -> files), FMV interface, settings.
//! See ENGINE_PORT.md. Core is pure Rust (tests run without Bevy); `src/bin/game.rs` wires it into Bevy.
//!
//! Game progress note: Sly 2 has no fixed C progress struct. Episodes/jobs/keys/coins/upgrades are Splice
//! "save variables" stored in the 0x5408-byte blob at 0x3d4a60, addressed through the level's variable table
//! (world+0x3a18 = 0x3e0ea8, `Block3a18`). `save::SaveVars` + `save::GameData` port that 1:1.

pub mod anim_play;
pub mod boot;
pub mod world_update;
pub mod clock;
pub mod fieldbind;
/// WP-PUB2: old path of [`fieldbind`] (the table is read from SCUS_973.16 now).
pub use fieldbind as fieldbind_gen;
pub mod fields;
pub mod flow;
pub mod fmv;
pub mod fmv_pss;
pub mod job_flow;
pub mod level_check;
pub mod main_loop;
pub mod methods;
pub mod method_table;
pub mod save;
pub mod scripts;
pub mod settings;
pub mod spawn;
pub mod splice_host;
pub mod startup;
pub mod transition;
pub mod vm_state;

#[cfg(feature = "game")]
pub mod app;
#[cfg(feature = "game")]
pub mod flow_app;
#[cfg(feature = "game")]
pub mod fmv_app;
