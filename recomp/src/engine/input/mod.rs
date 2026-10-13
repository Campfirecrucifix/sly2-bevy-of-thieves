//! WP-H: pad input. `pad`/`rumble` are the 1:1 game-side pad code (0x00187xxx, 0x001d5xxx); libpad/padman
//! are replaced by `backend` (feature `game`: gilrs gamepads + keyboard/mouse via Bevy) driven by `bindings`
//! (`controls.toml`, rebindable).
#![allow(non_snake_case)]

pub mod bindings;
pub mod pad;
pub mod rumble;

#[cfg(feature = "game")]
pub mod backend;

pub use bindings::{Bindings, VirtualPad};
pub use pad::{Clock, PadState, RawPad};
