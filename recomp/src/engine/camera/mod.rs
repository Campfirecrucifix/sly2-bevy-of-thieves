//! WP-I: camera system (CM at 0x002ddf40, 0x0014d3d8..0x0014f898 + policies 0x0014fca8..0x00151c38),
//! Hor+ widescreen, HUD safe-rect layout, Splice camera BIF hooks and the Bevy Camera3d sync (feature `game`).
#![allow(non_snake_case)]

pub mod cm;
pub mod follow;
pub mod follow_track;
pub mod collide;
pub mod hud;
pub mod looks;
pub mod math;
pub mod policy;
pub mod sound;
pub mod track;
pub mod widescreen;

#[cfg(feature = "game")]
pub mod bevy_sync;

pub use cm::{Cm, CmClock};

/// Splice BIF hooks as plain functions (ABI glue lives in WP-F).
pub mod bifs {
    use super::cm::*;
    use super::math::V3;

    /// The object id the game uses for the camera object (`g_cm` 0x002ddf40).
    pub const CM_OBJECT_ADDR: u32 = 0x002ddf40;

    /// 0x001246e0 BIF 119 `RefOpFindCameraObject`: returns a reference to the camera object.
    pub fn RefOpFindCameraObject() -> u32 {
        CM_OBJECT_ADDR
    }

    /// 0x00125168 BIF 128 `RefOpNearClipCenter`: world position of the near-clip plane centre.
    pub fn RefOpNearClipCenter(cm: &Cm) -> V3 {
        near_clip_center(cm)
    }
}
