//! Hor+ widescreen (port addition). The original FOV (cm+0x20) is VERTICAL; the PS2 aspect is fixed at
//! 640/492.8 = 1.2987 (cm+0x3c) — the 640x448 frame shown on a 4:3 TV. To keep the vertical FOV constant
//! while matching the original *horizontal* FOV at 4:3, the vertical half-extent is scaled once by
//! R_ASPECT_PS2 / (4/3) (= 0.974) and the horizontal extent is vertical * real window aspect.
//! Culling (FUN_0014da08/FUN_0014f850) and projection both derive from cm+0x40/+0x44, so they widen together.
#![allow(non_snake_case)]

use super::cm::*;

pub const ASPECT_4_3: f32 = 4.0 / 3.0;

/// Vertical-extent factor that maps the PS2 framing onto a true 4:3 display.
pub const HOR_PLUS_Y_SCALE: f32 = R_ASPECT_PS2 / ASPECT_4_3;

/// Apply Hor+ for a window of `w` x `h` pixels (any aspect: 4:3 .. 32:9).
pub fn apply_hor_plus(cm: &mut Cm, w: f32, h: f32, clk: CmClock) {
    let a = if h > 0.0 { w / h } else { ASPECT_4_3 };
    cm.r_aspect = a;
    cm.hor_plus_y_scale = HOR_PLUS_Y_SCALE;
    FUN_0014db70(cm, clk);
}

/// Restore the exact PS2 projection.
pub fn apply_original(cm: &mut Cm, clk: CmClock) {
    cm.r_aspect = R_ASPECT_PS2;
    cm.hor_plus_y_scale = 1.0;
    FUN_0014db70(cm, clk);
}

/// Full vertical FOV (radians) the renderer should use for the current state.
pub fn vertical_fov(cm: &Cm) -> f32 {
    2.0 * cm.y_scale.atan()
}

/// Full horizontal FOV (radians).
pub fn horizontal_fov(cm: &Cm) -> f32 {
    2.0 * cm.x_scale.atan()
}
