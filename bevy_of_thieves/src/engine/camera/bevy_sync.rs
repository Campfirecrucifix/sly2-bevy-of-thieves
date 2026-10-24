//! Bevy sync (feature `game`): copy the CM pose/fov into the `Camera3d` each frame.
//! Sly is Z-up; Bevy is Y-up: world (x, y, z) -> Bevy (x, z, -y).

use super::cm::Cm;
use super::widescreen::vertical_fov;
use bevy::prelude::*;

/// Marker for the camera entity driven by the CM.
#[derive(Component)]
pub struct CmCamera;

pub fn to_bevy(v: [f32; 3]) -> Vec3 {
    Vec3::new(v[0], v[2], -v[1])
}

/// Camera transform: Bevy looks down -Z; CM column 0 = forward, column 2 = up.
pub fn cm_transform(cm: &Cm) -> Transform {
    let pos = to_bevy(cm.pos());
    let f = to_bevy([cm.mat[0][0], cm.mat[0][1], cm.mat[0][2]]);
    let u = to_bevy([cm.mat[2][0], cm.mat[2][1], cm.mat[2][2]]);
    Transform::from_translation(pos).looking_to(f, u)
}

/// LOOK (port addition, verification): `SLY2_CAM="px,py,pz,fx,fy,fz,ux,uy,uz,tan_half_fov"` (Sly world
/// coords: position, forward = CM column 0, up = CM column 2, g_cm +0x44) pins the 3D camera to a fixed
/// pose for same-spot comparisons with PCSX2 (tools/pcsx2/cam_from_p2s.py prints it from a savestate's
/// g_cm 0x2ddf40). The CM itself keeps running (game logic unchanged).
pub fn cam_override() -> Option<(Transform, f32)> {
    static O: std::sync::OnceLock<Option<(Transform, f32)>> = std::sync::OnceLock::new();
    *O.get_or_init(|| {
        let v: Vec<f32> = std::env::var("SLY2_CAM").ok()?.split(',').filter_map(|x| x.trim().parse().ok()).collect();
        (v.len() >= 9).then(|| {
            let t = Transform::from_translation(to_bevy([v[0], v[1], v[2]])).looking_to(to_bevy([v[3], v[4], v[5]]), to_bevy([v[6], v[7], v[8]]));
            (t, v.get(9).map_or(0.0, |y| super::widescreen::screen_fov(*y)))
        })
    })
}

/// LOOK R14: centred full-height viewport of `aspect` inside a `size` target, or None when the target is
/// already that aspect (Hor+: cm.r_aspect = window aspect) or narrower.
pub fn pillarbox(size: UVec2, aspect: f32) -> Option<(UVec2, UVec2)> {
    let w = (size.y as f32 * aspect).round() as u32;
    (size.y > 0 && w + 1 < size.x).then(|| (UVec2::new((size.x - w) / 2, 0), UVec2::new(w, size.y)))
}

/// LOOK R16: the two black side bars of the Hor+-off pillarbox under ray tracing (drawn by the UI camera).
#[derive(Component)]
pub struct PillarBar(pub f32);

pub fn sync_cm_camera(
    cm: Res<Cm>,
    rt: Option<Res<crate::engine::gfx::rt::RtState>>,
    mut q: Query<(&mut Transform, &mut Projection, &mut Camera), With<CmCamera>>,
    mut bars: Query<(Entity, &PillarBar, &mut Sprite, &mut Transform), Without<CmCamera>>,
    win: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut commands: Commands,
) {
    let ov = cam_override();
    let rt_on = rt.is_some_and(|r| r.active);
    // LOOK R16: Solari lighting assumes viewport == target (main_pass_viewport covers only part of its
    // per-pixel buffers: the RT frame came out as a squeezed column over grey). Under RT the camera keeps
    // the full target and black bars on the UI layer cover the area outside the CM aspect (the 3D view
    // there is wider than the CM cull planes but hidden).
    let bar_w = win.single().ok().map(|w| (w.width(), w.height())).and_then(|(ww, wh)| {
        let cw = wh * cm.r_aspect;
        (rt_on && cw + 1.0 < ww).then_some(((ww - cw) * 0.5, ww, wh))
    });
    match bar_w {
        Some((bw, ww, wh)) => {
            if bars.is_empty() {
                for side in [-1.0f32, 1.0] {
                    commands.spawn((Sprite { color: Color::BLACK, ..default() }, Transform::default(), PillarBar(side)));
                }
            }
            for (_, b, mut s, mut tf) in &mut bars {
                s.custom_size = Some(Vec2::new(bw, wh));
                *tf = Transform::from_xyz(b.0 * (ww - bw) * 0.5, 0.0, -500.0);
            }
        }
        None => bars.iter().for_each(|(e, ..)| commands.entity(e).despawn()),
    }
    for (mut t, mut p, mut cam) in &mut q {
        // LOOK R14: Bevy's camera_system resets aspect_ratio to the viewport size, so with Hor+ off
        // (cm.r_aspect = the PS2 640/492.8) a 32:9 window still showed the wide view while the CM cull
        // planes stayed 4:3 (edge pop-in). Pillarbox: a centred viewport of the CM aspect, full height.
        let vp = if rt_on { None } else { cam.physical_target_size().and_then(|s| pillarbox(s, cm.r_aspect)) };
        if cam.viewport.as_ref().map(|v| (v.physical_position, v.physical_size)) != vp {
            cam.viewport = vp.map(|(physical_position, physical_size)| bevy::camera::Viewport { physical_position, physical_size, ..default() });
        }
        *t = ov.map_or_else(|| cm_transform(&cm), |o| o.0);
        if std::env::var_os("SLY2_R82CAM").is_some() {
            eprintln!("R82CM mat {:?} fov {} near {} far {} top {:?}", cm.mat, vertical_fov(&cm), cm.s_near_clip, cm.s_far_clip, cm.acpr.first().map(|c| (c.cpp, c.pcplcy, c.pso, c.pv)));
        }
        *p = Projection::Perspective(PerspectiveProjection {
            fov: ov.map(|o| o.1).filter(|f| *f > 0.0).unwrap_or_else(|| vertical_fov(&cm)),
            aspect_ratio: cm.r_aspect,
            near: cm.s_near_clip,
            far: cm.s_far_clip,
            ..default()
        });
    }
}
