//! WP-D3: skinned meshes from the level skin data (replaced, not 1:1: the PS2 blends on VU1
//! with the per-batch palette; Bevy skins with JOINT_INDEX/JOINT_WEIGHT + inverse bind poses).
//!
//! Data (world/alo.rs, FUN_00167320 mesh loaders): each mesh record has `weight_refs` = u8 n x
//! {s32 oid, s16 bone ref} (FUN_0022d6b0 ref encoding); each sub-mesh/strip has `weight_slots`
//! (up to 4 indices into `weight_refs`) and k = min(nweights,4) u8 weights per vertex indexing
//! those slots. Measured on all 43 levels: every vertex's k weights sum to exactly 255.
//! Joint index in the Bevy mesh = index into the record's `weight_refs` (= joint list order).
//!
//! gfx/ wiring (not done here, gfx/ belongs to WP-C3): for a `MeshPrim` with non-empty
//! `weights`, call [`skin_attributes`] (or `bevy_skinned_mesh` under feature `game`), resolve
//! `weight_refs` to joint entities, and build `SkinnedMesh { inverse_bindposes:
//! inverse_bindposes(rest_world), joints }`; anim/skin.rs then drives the joints.

use super::pose::{ee_sub_11c318, Mat4};
use crate::engine::gfx::mesh::MeshPrim;

/// Per-vertex Bevy skin attributes (always 4 joints, unused = joint 0 weight 0).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SkinAttrs {
    pub joints: Vec<[u16; 4]>,
    pub weights: Vec<[f32; 4]>,
}

/// JOINT_INDEX / JOINT_WEIGHT for a skinned prim; None if the prim carries no weights.
/// Weight bytes are scaled by 1/sum (sum = 255 in all shipped data) so they sum to 1.
pub fn skin_attributes(p: &MeshPrim) -> Option<SkinAttrs> {
    if p.weights.is_empty() || p.weights.len() != p.positions.len() {
        return None;
    }
    let mut a = SkinAttrs::default();
    for w in &p.weights {
        let mut j = [0u16; 4];
        let mut f = [0.0f32; 4];
        let sum: u32 = w.iter().take(4).map(|&b| b as u32).sum();
        for (s, &b) in w.iter().take(4).enumerate() {
            j[s] = *p.weight_slots.get(s).unwrap_or(&0) as u16;
            f[s] = if sum == 0 { if s == 0 { 1.0 } else { 0.0 } } else { b as f32 / sum as f32 };
        }
        a.joints.push(j);
        a.weights.push(f);
    }
    Some(a)
}

/// Inverse bind poses (FUN_0011c318 affine inverse) of the joints' rest world matrices.
pub fn inverse_bindposes(rest_world: &[Mat4]) -> Vec<Mat4> {
    rest_world.iter().map(ee_sub_11c318).collect()
}

/// Bevy skinned mesh (Solari-compatible attributes from gfx + joints/weights).
#[cfg(feature = "game")]
pub fn bevy_skinned_mesh(p: &MeshPrim) -> Option<bevy::prelude::Mesh> {
    use bevy::asset::RenderAssetUsages;
    use bevy::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
    use bevy::prelude::Mesh;
    let s = skin_attributes(p)?;
    Some(
        Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, p.positions.clone())
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, p.normals.clone())
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, p.uvs.clone())
            .with_inserted_attribute(Mesh::ATTRIBUTE_TANGENT, p.tangents.clone())
            .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, p.colors.clone())
            .with_inserted_attribute(Mesh::ATTRIBUTE_JOINT_INDEX, VertexAttributeValues::Uint16x4(s.joints))
            .with_inserted_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, s.weights)
            .with_inserted_indices(Indices::U32(p.indices.clone())),
    )
}

/// Bevy inverse bind pose matrices.
#[cfg(feature = "game")]
pub fn bevy_inverse_bindposes(rest_world: &[Mat4]) -> Vec<bevy::math::Mat4> {
    inverse_bindposes(rest_world).iter().map(|m| bevy::math::Mat4::from_cols_array(&super::pose::to_cols_array(m))).collect()
}
