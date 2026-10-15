//! WP-J: main loop, clock, level transitions, save game (mcman -> files), FMV interface, settings.
//! See ENGINE_PORT.md. Core is pure Rust (tests run without Bevy); `src/bin/game.rs` wires it into Bevy.
//!
//! Game progress note: Sly 2 has no fixed C progress struct. Episodes/jobs/keys/coins/upgrades are Splice
//! "save variables" stored in the 0x5408-byte blob at 0x3d4a60, addressed through the level's variable table
//! (world+0x3a18 = 0x3e0ea8, `Block3a18`). `save::SaveVars` + `save::GameData` port that 1:1.

pub mod anim_play;
pub mod arch1_bridge;
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
pub mod q_131990;

#[cfg(feature = "game")]
pub mod app;
#[cfg(feature = "game")]
pub mod flow_app;
#[cfg(feature = "game")]
pub mod fmv_app;
pub mod q_0021c6c8;
pub mod q_001bd040;
pub mod q_001c0320;
pub mod q_001f6838;
pub mod q_001e9090;
pub mod q_001cb2c0;
pub mod q_00156390;
pub mod q_00226960;
pub mod q_001d4b28;
pub mod q_001191e8;
pub mod q_001f10f0;
pub mod q_00200598;
pub mod q_c2c9670;
pub mod q_00182b18;
pub mod q_00141ef8;
pub mod q_0014b468;
pub mod q_00160260;
pub mod q_00166008;
pub mod q_0019eef0;
pub mod q_00189618;
pub mod q_001a4258;
pub mod q_001ad740;
pub mod q_001c8b68;
pub mod q_001dc598;
pub mod q_001f01b8;
pub mod q_001e46a0;
pub mod q_001ec688;
pub mod q_001ee100;
pub mod q_0020f948;
pub mod q_00114680;
pub mod q_0014cc58;
pub mod q_002155c0;
pub mod q_0022fa20;
pub mod q13_a;
pub mod q13_b;
pub mod q13_c;
pub mod q13_d;
pub mod live_bridge;
