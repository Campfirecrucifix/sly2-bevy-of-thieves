//! WP-I2: HUD / menu text. The disc fonts and the text layout of the original (FUN_0015ef00..FUN_0015ffd0)
//! are ported 1:1 and drawn by Bevy sprites inside the 16:9 safe rect (camera::hud::HudLayout).
pub mod binoc_hud;
pub mod binoc_recon;
pub mod bind;
pub mod caption;
pub mod card_app;
pub mod card_dialog;
pub mod episode;
pub mod font;
pub mod hud;
pub mod hud_slots;
pub mod language;
pub mod menu;
pub mod movie;
pub mod meter;
pub mod options;
pub mod settings;
pub mod strings;
pub mod text;
pub mod thiefnet;
pub mod title;
#[cfg(feature = "game")]
pub mod plugin;
