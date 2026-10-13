//! WP-C5: skinned-mesh hookup (Bevy-independent part). Replaced, not 1:1: the PS2 uploads the
//! per-batch bone palette to VU1 (FUN_0022faa0 matrices) and blends there; Bevy skins with
//! JOINT_INDEX/JOINT_WEIGHT (anim::skin_mesh) + joint entities + inverse bind poses.
//!
//! Joint list = the mesh record's `weight_refs` (u8 n x {s32 oid, s16 bone ref}); the bone ref
//! uses the FUN_0022d6b0 encoding with (owner, self) = (the ALO's owner, the ALO), resolved by
//! gfx::attach against the object whose oid is the s32: 33628 of 34048 refs (all 43 levels) name
//! the mesh's own ALO; the other 420 name an ALO owned (FUN_0013fb40 nested) by it, every one
//! found (2 oids occur twice: the one owned by the mesh's object chain is taken). Measured from the
//! data (the VU1 palette upload that consumes the list is not ported).
//! Joint entities are children of the object entity (PS2 object space, see anim::skin), so a
//! joint's local transform is its rest bone world * inverse(object world). The inverse bind pose
//! also folds in the primitive's own object-space placement (`ScenePrim::local`), so at rest
//! `joint * ibp = prim local` and the skinned mesh draws exactly where the rigid one did.

use super::attach::{ee_sub_22d6b0, BoneRef, BoneWorlds};
use super::mesh::MeshPrim;
use super::scene::{invert, mul, Affine};
use crate::engine::world::alo::MeshData;
use crate::engine::world::World;

/// The record's joint list (FUN_00167320 loaders: types 0/1/2/3 all carry weight_refs).
pub fn record_weight_refs(d: &MeshData) -> &[(i32, i16)] {
    match d {
        MeshData::Type0(m) => &m.weight_refs,
        MeshData::Type1(m) | MeshData::Type2(m) => &m.weight_refs,
        MeshData::Type3(m) => &m.weight_refs,
        _ => &[],
    }
}

/// Resolved joints of one mesh record of world object `obj`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SkinJoints {
    pub refs: Vec<BoneRef>,
    /// rest world matrix per joint (None for null / dangling refs)
    pub rest_world: Vec<Option<Affine>>,
}

pub fn skin_joints(world: &World, bw: &mut BoneWorlds, obj: usize, refs: &[(i32, i16)]) -> SkinJoints {
    let mut j = SkinJoints::default();
    for &(oid, r) in refs {
        let t = if world.objects[obj].oid == oid { obj } else { oid_target(world, obj, oid).unwrap_or(obj) };
        let b = ee_sub_22d6b0(world, r, world.objects[t].owner, Some(t), BoneRef::None);
        j.rest_world.push(match b {
            BoneRef::Bone(o, k) => Some(bw.get(world, o, k)),
            _ => None,
        });
        j.refs.push(b);
    }
    j
}

/// Object with `oid`, preferring one owned (directly or not) by `obj`.
fn oid_target(world: &World, obj: usize, oid: i32) -> Option<usize> {
    let owned_by = |mut i: usize| {
        for _ in 0..64 {
            match world.objects[i].owner {
                Some(o) if o == obj => return true,
                Some(o) => i = o,
                None => return false,
            }
        }
        false
    };
    let c: Vec<usize> = (0..world.objects.len()).filter(|&i| world.objects[i].oid == oid).collect();
    c.iter().copied().find(|&i| owned_by(i)).or(c.first().copied())
}

/// Every joint index of the prim's skin attributes names a resolved joint and every vertex has
/// weight.
pub fn skin_valid(p: &MeshPrim, j: &SkinJoints) -> bool {
    let Some(a) = crate::engine::anim::skin_mesh::skin_attributes(p) else { return false };
    a.joints.iter().zip(&a.weights).all(|(js, ws)| {
        ws.iter().sum::<f32>() > 0.99
            && js.iter().zip(ws).all(|(&ji, &w)| w == 0.0 || j.rest_world.get(ji as usize).is_some_and(|r| r.is_some()))
    })
}

/// Joint entity transform relative to the object entity (`obj_world` = LevelScene::world_of).
/// Unresolved joints sit at `prim_local` (their inverse bind pose is then identity -> rigid).
pub fn joint_local(rest_world: Option<&Affine>, obj_world: &Affine, prim_local: &Affine) -> Affine {
    match rest_world {
        Some(w) => mul(w, &invert(obj_world)),
        None => *prim_local,
    }
}

/// Inverse bind pose (row-vector convention): v * prim_local * inverse(joint_local).
pub fn inverse_bindpose(prim_local: &Affine, joint_local: &Affine) -> Affine {
    mul(prim_local, &invert(joint_local))
}
