//! LOOK R39: world-anchored text of FUN_001cd780 (v2 name `cm_screen_px_ps2`; QMem port
//! `game::q_wires1i3::ee_sub_1cd780`), used by the K12 job marker FUN_0022a908 for the job name: the point is
//! projected by FUN_0014f308 (g_cm 0x2ddf40), culled behind the near clip (cm +0x4c), the text scale is
//! s = f x 640 (DAT_002c1f20) / (cm +0x40 x depth) clamped by FUN_0011d948 to (0.4, 1.5) (DAT_002c1f28; consts
//! read from the PCSX2 EP4 028 savestate), the text (FUN_00215ac0, default font, FUN_00216260 size) is
//! centred on the point's x with its bottom 8 (DAT_002c1f34) above it and drawn by FUN_002163b0 in a rect
//! padded by 16 / 16 (DAT_002c1f30 / 34 x 2), colour *a3. The 640 x 492.8 projection space maps onto the
//! window through `HudLayout::map_projected` (as the other 3D-anchored labels).
use super::font::{Fonts, GlyphQuad};
use super::plugin::{UiFonts, UiGame};
use super::text::{Text, TextRect};
use crate::engine::camera::cm::{Cm, FUN_0014f308};
use crate::engine::camera::math::FUN_0011d948;
use crate::engine::gfx::ultrawide::HudSafeRect;
use bevy::prelude::*;

/// One FUN_001cd780 call of this frame: level string id (FUN_0013e368(world, id)), world point, scale f
/// and the colour word (*a3).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldLabel {
    pub id: i32,
    pub pos: [f32; 3],
    pub f: f32,
    pub color: u32,
}

/// Labels queued by the gfx side this frame (npcdraw::objects K12 marker draw).
#[derive(Resource, Default, Clone, Debug)]
pub struct WorldLabels(pub Vec<WorldLabel>);

#[derive(Component)]
pub struct WorldLabelSprite;

/// FUN_001cd780 layout in projection space (640 x 492.8); None when the point is behind the near clip.
pub fn ee_sub_1cd780(fonts: &mut Fonts, cm: &Cm, s: &[u8], p: [f32; 3], f: f32, color: u32) -> Option<Vec<GlyphQuad>> {
    let q = FUN_0014f308(cm, p);
    if q[2] < cm.s_near_clip {
        return None;
    }
    let sc = FUN_0011d948(f * (640.0 / (cm.x_scale * q[2])), [0.4, 1.5]);
    let fi = fonts.default;
    fonts.fonts[fi].push_scale(sc, sc);
    let mut t = Text::new(s, fi);
    let hh = t.height(fonts, 640.0);
    let w = t.widest(fonts);
    let x = (q[0] * 0.5 + 0.5) * 640.0 - w * 0.5;
    let y = (0.5 - q[1] * 0.5) * 492.80002 - hh - 8.0;
    let g = t.draw(fonts, &TextRect { x, y, w: w + 16.0, h: hh + 16.0, color, halign: 0, valign: 0 });
    fonts.fonts[fi].pop_scale();
    Some(g)
}

/// Respawn the label sprites every frame (projection space -> window via `map_projected`).
pub fn draw_world_labels(
    mut commands: Commands,
    ui: Option<ResMut<UiFonts>>,
    game: Res<UiGame>,
    cm: Option<Res<Cm>>,
    labels: Option<Res<WorldLabels>>,
    safe: Res<HudSafeRect>,
    old: Query<Entity, With<WorldLabelSprite>>,
) {
    for e in &old {
        commands.entity(e).despawn();
    }
    if std::env::var_os("SLY2_LOOK_K12LOG").is_some() {
        info!("WLABELS ui {} cm {} n {:?} strings {}", ui.is_some(), cm.is_some(), labels.as_ref().map(|l| l.0.len()), game.strings.0.len());
    }
    let (Some(mut ui), Some(cm), Some(labels)) = (ui, cm, labels) else { return };
    let l = &safe.0;
    for wl in &labels.0 {
        let s = game.strings.FUN_0013e368(wl.id).to_vec();
        if s.is_empty() {
            continue;
        }
        let gs = ee_sub_1cd780(&mut ui.fonts, &cm, &s, wl.pos, wl.f, wl.color);
        if std::env::var_os("SLY2_LOOK_K12LOG").is_some() {
            info!("WLABEL id {} '{}' at {:?} q {:?} glyphs {:?} first {:?}", wl.id, super::strings::to_string(&s), wl.pos, FUN_0014f308(&cm, wl.pos), gs.as_ref().map(|g| g.len()), gs.as_ref().and_then(|g| g.first()).map(|g| (g.x0, g.y0, g.x1, g.y1)));
        }
        let Some(gs) = gs else { continue };
        for g in gs {
            let (x0, y0) = l.map_projected(g.x0, g.y0);
            let (x1, y1) = l.map_projected(g.x1, g.y1);
            let c = |v: u8| v as f32 / 128.0;
            commands.spawn((
                Sprite {
                    image: ui.images[g.tex].clone(),
                    rect: Some(bevy::math::Rect::new(g.u0, g.v0, g.u1, g.v1)),
                    custom_size: Some(Vec2::new(x1 - x0, y1 - y0)),
                    color: Color::srgba(c(g.rgba[0]), c(g.rgba[1]), c(g.rgba[2]), c(g.rgba[3]).min(1.0)),
                    ..default()
                },
                Transform::from_xyz((x0 + x1) * 0.5 - l.window_w * 0.5, l.window_h * 0.5 - (y0 + y1) * 0.5, 0.9),
                WorldLabelSprite,
            ));
        }
    }
}
