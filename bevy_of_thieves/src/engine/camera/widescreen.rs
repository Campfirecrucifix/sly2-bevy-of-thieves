//! Hor+ widescreen (port addition). The original FOV (cm+0x20) is VERTICAL: cm+0x44 = tan(fov/2),
//! cm+0x40 = cm+0x44 * aspect (cm+0x3c = 640/492.8 = 1.2987 on PS2). Hor+ keeps the vertical extent and
//! widens x with the window (x_scale = y_scale * window aspect); culling (FUN_0014da08/FUN_0014f850) and
//! projection both derive from cm+0x40/+0x44, so they widen together.
//!
//! LOOK (PS2 screen mapping, 1:1): FUN_0014db70 builds proj sx = 512/(x_scale*4096), sy = 224/(y_scale*4096)
//! (frustum edge -> 0.125 / 0.0547 clip units). The VU1 screen transform DAT_002c1f00 (uploaded as VU1 qw 0,
//! 0x4bfd70a4 = 2^25 * 0.99 for x/y, 2^32 * 0.99 for z) and DAT_002c1f10 (qw 1, 0x50008000 = 2^33 + 2^25:
//! float-mantissa offset 0x8000 = 2048.0 in GS 12.4) put the frustum edge at 0.99 * 256 px / 0.99 * 112 px,
//! while the frame is 512x224 per field (GS SCISSOR 0..511 x 0..223, XYOFFSET (1792, 1936) -> centre
//! (256, 112); savestate 020_082903 GS.bin). So the visible half-extent is tan / 0.99 on both axes: the PS2
//! shows 1/0.99 = 1.0101x more than its cull frustum. The full 224 field lines fill the TV height.
//! PCSX2 widescreen patches only change cm+0x3c (1.7316 = 1.2987 * 4/3 in the emu_rec5 savestates), i.e.
//! they are Hor+ with this same vertical extent.
#![allow(non_snake_case)]

use super::cm::*;

pub const ASPECT_4_3: f32 = 4.0 / 3.0;

/// Vertical-extent factor for Hor+: 1.0 = the PS2's own vertical extent at every aspect (the PS2 4:3
/// picture is 640/492.8 wide in tan units on a 4:3 TV, i.e. 2.6% horizontally squeezed; Hor+ shows it
/// unsqueezed at the original height, as the PCSX2 widescreen patch does). Was R_ASPECT_PS2 / (4/3) = 0.974
/// (kept the 4:3 horizontal extent instead -> port 1.03x more zoomed vertically than PS2).
pub const HOR_PLUS_Y_SCALE: f32 = 1.0;

/// VU1 screen scale (DAT_002c1f00 = 0.99 * 2^25): the cm frustum edge lands at 0.99 of the half screen.
pub const GS_SCREEN_SCALE: f32 = 0.99;

/// Full on-screen FOV (radians) for a cm tangent (cm+0x44 or cm+0x40): 2 * atan(tan / 0.99).
pub fn screen_fov(tan_half: f32) -> f32 {
    2.0 * (tan_half / GS_SCREEN_SCALE).atan()
}

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
    screen_fov(cm.y_scale)
}

/// Full horizontal FOV (radians).
pub fn horizontal_fov(cm: &Cm) -> f32 {
    screen_fov(cm.x_scale)
}
