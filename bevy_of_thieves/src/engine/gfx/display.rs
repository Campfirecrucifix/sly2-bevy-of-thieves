//! LOOK: PS2 display-space rendering. The GS modulates, adds, alpha-blends and fogs the raw 8-bit
//! framebuffer values (texel * Cv / 0x80, Cs*As + Cd, ...) and the CRTC shows those bytes directly:
//! all PS2 colour math is in display (gamma) space. The port therefore keeps every 3D colour as raw
//! display numbers (textures `Rgba8Unorm`, vertex colours / light factors / fog colour unconverted,
//! `light::display_space`) so Bevy's blending does the same arithmetic as the GS, and this pass turns
//! the finished 3D frame (display values) into linear for the sRGB target (exact sRGB decode, after
//! the PS2 COLCLAMP clamp to 0..1). Before: sRGB textures x gamma-2.2 vertex colours and the additive
//! colour layer summed in linear -> dark frames short on the colour layer (hub 20/24/38 vs PCSX2 25/37/52).
//! Kill switch: `SLY2_LOOK_LINEAR=1` (old linear path, no pass).
use bevy::core_pipeline::fullscreen_material::{FullscreenMaterial, FullscreenMaterialPlugin};
use bevy::prelude::*;
use bevy::render::{extract_component::ExtractComponent, render_resource::ShaderType, RenderApp};
use bevy::shader::ShaderRef;

#[derive(Component, ExtractComponent, Clone, Copy, ShaderType, Default)]
#[extract_app(RenderApp)]
pub struct Ps2DisplayDecode {
    pub on: f32,
}

impl FullscreenMaterial for Ps2DisplayDecode {
    fn fragment_shader() -> ShaderRef {
        "embedded://sly2_recomp/engine/gfx/display.wesl".into()
    }
}

fn attach(mut commands: Commands, cams: Query<Entity, (With<Camera3d>, Without<Ps2DisplayDecode>)>) {
    for e in &cams {
        commands.entity(e).insert(Ps2DisplayDecode { on: if std::env::var_os("SLY2_LOOK_DISPTEST").is_some() { 2.0 } else { 1.0 } });
    }
}

pub struct Ps2DisplayPlugin;
impl Plugin for Ps2DisplayPlugin {
    fn build(&self, app: &mut App) {
        if !super::light::display_space() {
            return;
        }
        bevy::asset::embedded_asset!(app, "display.wesl");
        app.add_plugins(FullscreenMaterialPlugin::<Ps2DisplayDecode>::default()).add_systems(PostUpdate, attach);
    }
}
