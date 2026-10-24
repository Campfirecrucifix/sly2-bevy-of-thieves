//! WP-R: 32:9 ultrawide checks + HUD safe rect for Bevy (port addition).
//! Hor+ itself is camera/widescreen.rs (WP-I): vertical FOV constant (cm+0x20, scaled once by
//! R_ASPECT_PS2/(4/3)), horizontal = vertical * window aspect; the CM cull planes (FUN_0014da08 from
//! cm+0x40/+0x44) and the Bevy projection (bevy_sync: fov = vertical, aspect = cm.r_aspect, Bevy derives
//! its culling frustum from it) widen together. This file reports/verifies that for any window size and
//! gives Bevy HUD/menu code the centred 16:9 safe rect (`HudSafeRect`, camera::hud::HudLayout).

use crate::engine::camera::cm::{Cm, CmClock};
use crate::engine::camera::hud::{HudLayout, Rect};
use crate::engine::camera::widescreen;

/// Projection the game uses for a `w` x `h` window.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewReport {
    pub aspect: f32,
    /// full vertical FOV (radians); constant for every aspect under Hor+
    pub vfov: f32,
    /// full horizontal FOV (radians)
    pub hfov: f32,
    /// CM tangents (cm+0x40 x, cm+0x44 y) used by the cull planes
    pub x_scale: f32,
    pub y_scale: f32,
    /// HUD 16:9 safe rect (window pixels, y down)
    pub safe: Rect,
}

/// Hor+ (or the original framing) for a `w` x `h` window at vertical FOV `rad_fov` (cm+0x20).
pub fn view_report(rad_fov: f32, w: f32, h: f32, hor_plus: bool) -> ViewReport {
    let mut c = Cm::new();
    c.rad_fov = rad_fov;
    let clk = CmClock { t: 0.0, dt: 0.0, dt_real: 0.0 };
    if hor_plus {
        widescreen::apply_hor_plus(&mut c, w, h, clk);
    } else {
        widescreen::apply_original(&mut c, clk);
    }
    ViewReport {
        aspect: c.r_aspect,
        vfov: widescreen::vertical_fov(&c),
        hfov: widescreen::horizontal_fov(&c),
        x_scale: c.x_scale,
        y_scale: c.y_scale,
        safe: HudLayout::new(w, h).safe,
    }
}

/// Camera-space point (CM basis: x forward, y left, z up) inside the CM view pyramid built from
/// cm+0x40/+0x44 (the same tangents FUN_0014da08 turns into the cull planes).
pub fn in_cm_view(r: &ViewReport, p: [f32; 3]) -> bool {
    p[0] > 0.0 && p[1].abs() <= p[0] * r.x_scale && p[2].abs() <= p[0] * r.y_scale
}

/// WIRE-OPT: HUD layout for settings.toml `[graphics] hud`: "centre" = the centred 16:9 safe rect
/// (`HudLayout::new`), "stretch" = same HUD scale but the safe rect spans the full window width, so
/// left / right anchored elements sit at the screen edges.
pub fn hud_layout(w: f32, h: f32, stretch: bool) -> HudLayout {
    let mut l = HudLayout::new(w, h);
    if stretch {
        l.safe.x = 0.0;
        l.safe.w = w;
    }
    l
}

#[cfg(feature = "game")]
pub use bevy_uw::*;

#[cfg(feature = "game")]
mod bevy_uw {
    use super::*;
    use crate::engine::camera::bevy_sync::CmCamera;
    use crate::engine::camera::hud::HUD_PROBE_BOXES;
    use bevy::camera::primitives::Frustum;
    use bevy::prelude::*;

    /// 16:9 safe rect of the primary window (pixels, y down), refreshed every frame. Bevy HUD /
    /// menu roots should be laid out inside it (HudLayout::map for the original 640x448 coordinates).
    #[derive(Resource, Clone, Copy, Debug, PartialEq)]
    pub struct HudSafeRect(pub HudLayout);

    /// Debug outline of the safe rect (env `SLY2_SHOW_SAFE_RECT=1`), drawn with gizmos.
    #[derive(Resource, Clone, Copy, Debug, Default)]
    pub struct ShowSafeRect(pub bool);

    pub struct UltrawidePlugin;

    /// Gizmo group of the safe-rect overlay: drawn over the scene (depth_bias -1), so geometry closer
    /// than the overlay plane (floors at the bottom of the screen) does not hide it (WP-R2).
    #[derive(Default, Reflect, GizmoConfigGroup)]
    pub struct HudGizmos;

    impl Plugin for UltrawidePlugin {
        fn build(&self, app: &mut App) {
            let show = std::env::var("SLY2_SHOW_SAFE_RECT").is_ok_and(|v| v != "0" && !v.is_empty());
            app.insert_resource(HudSafeRect(HudLayout::new(1920.0, 1080.0)))
                .insert_resource(ShowSafeRect(show))
                .insert_gizmo_config(HudGizmos, GizmoConfig { depth_bias: -1.0, ..default() })
                .add_systems(PreUpdate, update_safe_rect)
                .add_systems(PostUpdate, draw_safe_rect.run_if(|s: Res<ShowSafeRect>| s.0).after(TransformSystems::Propagate));
        }
    }

    pub fn update_safe_rect(windows: Query<&Window>, mut r: ResMut<HudSafeRect>, live: Option<Res<crate::settings::LiveGraphics>>) {
        let Ok(w) = windows.single() else { return };
        let l = hud_layout(w.width(), w.height(), live.is_some_and(|l| l.hud_stretch));
        if r.0 != l {
            r.0 = l;
        }
    }

    /// Outline the safe rect just behind the near plane of the CM camera.
    pub fn draw_safe_rect(mut gizmos: Gizmos<HudGizmos>, r: Res<HudSafeRect>, cams: Query<(&Camera, &GlobalTransform, &Projection), With<CmCamera>>, win: Query<&Window, With<bevy::window::PrimaryWindow>>) {
        let s = r.0.safe;
        for (cam, gt, proj) in &cams {
            let near = match proj {
                Projection::Perspective(p) => p.near,
                _ => 1.0,
            };
            let inset = 2.0;
            let px = [
                Vec2::new(s.x + inset, s.y + inset),
                Vec2::new(s.x + s.w - inset, s.y + inset),
                Vec2::new(s.x + s.w - inset, s.y + s.h - inset),
                Vec2::new(s.x + inset, s.y + s.h - inset),
            ];
            let mut outline = |px: [Vec2; 4], c: Color| {
                let pts: Vec<Vec3> = px.iter().filter_map(|p| cam.viewport_to_world(gt, crate::engine::gfx::upscale::window_to_viewport(cam, win.single().ok(), *p)).ok()).map(|ray| ray.origin + *ray.direction * near * 2.0).collect();
                if pts.len() == 4 {
                    gizmos.linestrip(pts.iter().copied().chain(std::iter::once(pts[0])), c);
                }
            };
            outline(px, Color::srgb(1.0, 0.0, 1.0));
            // WP-R2: the original HUD corner/centre boxes mapped through HudLayout::map_rect
            for [x, y, w, h] in HUD_PROBE_BOXES {
                let b = r.0.map_rect(x, y, w, h, HudLayout::auto_anchor(x + w * 0.5));
                outline([Vec2::new(b.x, b.y), Vec2::new(b.x + b.w, b.y), Vec2::new(b.x + b.w, b.y + b.h), Vec2::new(b.x, b.y + b.h)], Color::srgb(0.0, 1.0, 1.0));
            }
        }
    }

    /// Bevy's culling frustum for the projection bevy_sync builds from `r` (camera at the origin
    /// looking down -Z). Used to prove the Bevy culling widens exactly like the CM planes.
    pub fn bevy_frustum(r: &ViewReport, near: f32, far: f32) -> Frustum {
        use bevy::camera::CameraProjection;
        let p = PerspectiveProjection { fov: r.vfov, aspect_ratio: r.aspect, near, far, ..default() };
        p.compute_frustum(&GlobalTransform::IDENTITY)
    }

    /// Point (Bevy view space) inside Bevy's culling frustum.
    pub fn frustum_contains(f: &Frustum, p: Vec3) -> bool {
        f.intersects_sphere(&bevy::camera::primitives::Sphere { center: p.into(), radius: 0.0 }, true)
    }

    /// CM camera-space point (x forward, y left, z up) -> Bevy view space (-Z forward, -X left, +Y up).
    pub fn cm_to_view(p: [f32; 3]) -> Vec3 {
        Vec3::new(-p[1], p[2], -p[0])
    }
}
