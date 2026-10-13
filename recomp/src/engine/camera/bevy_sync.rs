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

pub fn sync_cm_camera(cm: Res<Cm>, mut q: Query<(&mut Transform, &mut Projection), With<CmCamera>>) {
    for (mut t, mut p) in &mut q {
        *t = cm_transform(&cm);
        *p = Projection::Perspective(PerspectiveProjection {
            fov: vertical_fov(&cm),
            aspect_ratio: cm.r_aspect,
            near: cm.s_near_clip,
            far: cm.s_far_clip,
            ..default()
        });
    }
}
