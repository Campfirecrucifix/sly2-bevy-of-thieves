//! WP-D2 (feature `game`): apply a pose's skinning palette to Bevy `SkinnedMesh` joints.
//! Replaced, not 1:1: the PS2 uploads the palette to VU1 per draw (FUN_0022faa0 matrices);
//! Bevy computes `joint_global * inverse_bindpose`, so each joint entity gets the local
//! Transform `palette[i] * bind[i]` (= the bone's object-space matrix). Joint entities must be
//! children of the object's root entity (PS2 object space; the level root does Z-up -> Y-up).
//! gfx/ spawns rigid prims today; the coordinator registers [`AnimSkinPlugin`] and attaches
//! [`AnimSkin`] to skinned entities when they exist.

use super::pose::{ee_sub_11c168, to_cols_array, Mat4};
use bevy::mesh::skinning::SkinnedMesh;
use bevy::prelude::*;

/// Current pose of a skinned entity: palette = world * inverse(bind) (pose::skin_palette).
#[derive(Component, Clone, Debug, Default)]
pub struct AnimSkin {
    pub palette: Vec<Mat4>,
    pub bind: Vec<Mat4>,
}

/// Joint local transforms for a palette (plain fn, testable without an App).
pub fn joint_transforms(skin: &AnimSkin) -> Vec<Transform> {
    skin.palette
        .iter()
        .zip(&skin.bind)
        .map(|(p, b)| Transform::from_matrix(bevy::math::Mat4::from_cols_array(&to_cols_array(&ee_sub_11c168(p, b)))))
        .collect()
}

/// System: write every `AnimSkin` palette into its `SkinnedMesh` joints.
pub fn apply_skin_palette(skins: Query<(&AnimSkin, &SkinnedMesh), Changed<AnimSkin>>, mut joints: Query<&mut Transform>) {
    for (skin, mesh) in &skins {
        for (e, t) in mesh.joints.iter().zip(joint_transforms(skin)) {
            if let Ok(mut tr) = joints.get_mut(*e) {
                *tr = t;
            }
        }
    }
}

/// Registers [`apply_skin_palette`] and the ASEGA driver ([`super::play::drive_asegas`]).
pub struct AnimSkinPlugin;
impl Plugin for AnimSkinPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<super::play::AnimClock>()
            .add_systems(PreUpdate, update_anim_clock)
            .add_systems(Update, (super::play::drive_asegas, apply_skin_palette).chain());
    }
}

/// DAT_002dded4 / DAT_002ddf10 from Bevy's clocks (game time and real time).
pub fn update_anim_clock(mut clock: ResMut<super::play::AnimClock>, time: Res<Time>, real: Res<Time<Real>>) {
    clock.dt = time.delta_secs();
    clock.real_dt = real.delta_secs();
}
