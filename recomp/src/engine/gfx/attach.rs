//! WP-C3: object / bone reference resolution and bone world matrices (attachments).
//!
//! FUN_0022d6b0 (0x0022d6b0) resolves every s16 "bone ref" stored by the loaders:
//! ALO bones (FUN_0022d7a0 parent ref, kinds 8/0xe attach ref, via vtable +0x19c =
//! FUN_0022d778 -> (ref, owner, self)) and class-3 XFM objects (FUN_00231768 via vtable +0x18c =
//! FUN_0022d750 -> (ref, owner, null)). The ref is sign-extended to u32 first:
//!   ref & 0x3000 == 0      -> self bone[ref]           (null when there is no self)
//!   ref & 0x3000 == 0x1000 -> owner bone[ref & 0xffff0fff]
//!   ref & 0x3000 == 0x2000 -> top of the load-context stack DAT_004437a8 (pushed by
//!                             FUN_002317e0 = the enclosing class-3 parent's own bone +0x50)
//!   ref & 0x3000 == 0x3000 -> null (all negative refs, -1 = none)
//! The original never range-checks the index; out-of-range refs are reported as `Dangling`.
//!
//! Bone world matrices (FUN_0022faa0): world = local * parent-bone world, null parent =
//! identity (the local matrix is already world space). Kinds 8/0xe carry no matrix (identity).
//! Class-3 world (FUN_00231368): local (+0x60) if its bone (+0x50) is null, else local * bone world.

use super::scene::{mul, Affine, IDENTITY};
use crate::engine::world::alo::Alo;
use crate::engine::world::objects::{LoadData, Payload};
use crate::engine::world::World;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BoneRef {
    /// null pointer
    None,
    /// (world object index, bone index)
    Bone(usize, usize),
    /// index outside the target's bone array (or target has no bones)
    Dangling,
}

pub fn alo_of(w: &World, i: usize) -> Option<&Alo> {
    match &w.objects.get(i)?.payload {
        Payload::Loaded(LoadData::Alo { alo, .. }) => Some(alo),
        _ => None,
    }
}

/// Class-3 style (FUN_00231768) bone ref + local matrix of a non-ALO object.
pub fn xfm_of(w: &World, i: usize) -> Option<(i16, Affine)> {
    let (s, x) = match &w.objects.get(i)?.payload {
        Payload::Loaded(LoadData::L231170 { s, xform, .. }) => (*s, xform),
        Payload::Loaded(LoadData::L227380 { xform, .. }) | Payload::Loaded(LoadData::L231170Blob { xform, .. }) => (xform.s, &xform.xform),
        Payload::Loaded(LoadData::Solid(sol)) => (sol.xform.s, &sol.xform.xform),
        Payload::Loaded(LoadData::L2282b0 { solid, .. }) => (solid.xform.s, &solid.xform.xform),
        _ => return None,
    };
    Some((s, affine_from_bytes(x)))
}

/// FUN_00117f18: 3 basis rows + translation row (f32 LE).
pub fn affine_from_bytes(b: &[u8; 48]) -> Affine {
    let f = |k: usize| f32::from_le_bytes(b[k * 4..k * 4 + 4].try_into().unwrap());
    [[f(0), f(1), f(2)], [f(3), f(4), f(5)], [f(6), f(7), f(8)], [f(9), f(10), f(11)]]
}

fn bone_at(w: &World, obj: Option<usize>, idx: u32) -> BoneRef {
    match obj.and_then(|o| alo_of(w, o).map(|a| (o, a))) {
        Some((o, a)) if (idx as usize) < a.base.bones.len() => BoneRef::Bone(o, idx as usize),
        _ => BoneRef::Dangling,
    }
}

/// FUN_0022d6b0 (0x0022d6b0). `ctx` = the 0x2000 load-context bone (FUN_002317e0 push).
pub fn ee_sub_22d6b0(w: &World, r: i16, owner: Option<usize>, this: Option<usize>, ctx: BoneRef) -> BoneRef {
    let p = r as i32 as u32;
    match p & 0x3000 {
        0x1000 => bone_at(w, owner, p & 0xffff0fff),
        0 => match this {
            Some(_) => bone_at(w, this, p),
            None => BoneRef::None,
        },
        0x2000 => ctx,
        _ => BoneRef::None,
    }
}

/// Load-context bone for objects created inside a class-3 parent's FUN_002317e0 child list:
/// the parent's own resolved bone (+0x50). Only classes whose load is FUN_00220fd0 /
/// FUN_0015c1e8 push it; anything else -> None (the stack is empty at top level).
fn context_of(w: &World, i: usize) -> BoneRef {
    let Some(o) = w.objects[i].owner else { return BoneRef::None };
    let load = crate::engine::world::class::class(w.objects[o].class).map(|c| c.load).unwrap_or(0);
    if load != 0x00220fd0 && load != 0x0015c1e8 {
        return BoneRef::None;
    }
    match xfm_of(w, o) {
        Some((s, _)) => ee_sub_22d6b0(w, s, w.objects[o].owner, None, context_of(w, o)),
        None => BoneRef::None,
    }
}

/// Resolved refs of one ALO's bones: (parent, attach).
pub fn alo_bone_refs(w: &World, i: usize) -> Vec<(BoneRef, Option<BoneRef>)> {
    let Some(a) = alo_of(w, i) else { return vec![] };
    let owner = w.objects[i].owner;
    let ctx = context_of(w, i);
    a.base
        .bones
        .iter()
        .map(|b| (ee_sub_22d6b0(w, b.r#ref, owner, Some(i), ctx), b.attach.map(|r| ee_sub_22d6b0(w, r, owner, Some(i), ctx))))
        .collect()
}

/// A6: socket of a template copy's null-parent bones. FUN_001ca370 calls, per copy object, vtable
/// +0x54(owner, bone, 0) then +0x12c(xform); the ALO +0x54 is FUN_001f21b8(alo, newParent, bone,
/// flags): `bone` defaults to newParent +0x60 (the owner ALO's third base ref) and every bone whose
/// parent (+0xa0) lies in the old parent's bone array - for a top-level template copy (+0x10 null:
/// range [0, 0x100)) exactly the null parents - is re-parented to it. `bone` is FUN_001c9b08's
/// FUN_0022d6b0(ref, owner, null). Returns (socket bone, +0x12c placement matrix) for template copy
/// objects with an ALO owner; None otherwise (null parent = identity, FUN_0022faa0).
/// Used by BoneWorlds (FUN_0022faa0 after the re-parent) and the character rigs; alo_bone_refs keeps
/// the load-time refs (the C3 attachment counts).
pub fn template_socket(w: &World, i: usize) -> Option<(BoneRef, Affine)> {
    let t = w.template_instances.iter().rev().find(|t| t.objects.contains(&i))?;
    let o = t.owner?;
    let oa = alo_of(w, o)?;
    let b = match ee_sub_22d6b0(w, t.bone, Some(o), None, BoneRef::None) {
        b @ BoneRef::Bone(..) => b,
        _ => ee_sub_22d6b0(w, oa.base.refs[2], w.objects[o].owner, Some(o), context_of(w, o)),
    };
    matches!(b, BoneRef::Bone(..)).then(|| (b, affine_from_bytes(&t.xform)))
}

/// Bone ref of a class-3 (XFM) object (FUN_00231768 -> +0x50).
pub fn xfm_bone_ref(w: &World, i: usize) -> Option<BoneRef> {
    let (s, _) = xfm_of(w, i)?;
    Some(ee_sub_22d6b0(w, s, w.objects[i].owner, None, context_of(w, i)))
}

/// Rest-pose world matrices of every ALO bone in the world (FUN_0022faa0 order-independent).
#[derive(Default)]
pub struct BoneWorlds {
    cache: HashMap<(usize, usize), Affine>,
    refs: HashMap<usize, Vec<(BoneRef, Option<BoneRef>)>>,
}

impl BoneWorlds {
    pub fn get(&mut self, w: &World, obj: usize, bone: usize) -> Affine {
        self.solve(w, obj, bone, 0)
    }

    fn solve(&mut self, w: &World, obj: usize, bone: usize, depth: usize) -> Affine {
        if let Some(m) = self.cache.get(&(obj, bone)) {
            return *m;
        }
        let Some(a) = alo_of(w, obj) else { return IDENTITY };
        let Some(b) = a.base.bones.get(bone) else { return IDENTITY };
        let local = b.xform.unwrap_or(IDENTITY);
        let refs = self.refs.entry(obj).or_insert_with(|| alo_bone_refs(w, obj));
        let parent = refs[bone].0;
        let m = match parent {
            BoneRef::Bone(o, k) if (o, k) != (obj, bone) && depth < 256 => mul(&local, &self.solve(w, o, k, depth + 1)),
            // A6: template copy null parents hang off the owner's socket (FUN_001f21b8, +0x12c)
            BoneRef::None if depth < 256 => match template_socket(w, obj) {
                Some((BoneRef::Bone(o, k), pm)) => mul(&mul(&local, &pm), &self.solve(w, o, k, depth + 1)),
                _ => local,
            },
            _ => local,
        };
        self.cache.insert((obj, bone), m);
        m
    }

    /// FUN_00231368: class-3 object world matrix.
    pub fn xfm_world(&mut self, w: &World, i: usize) -> Option<Affine> {
        let (_, local) = xfm_of(w, i)?;
        Some(match xfm_bone_ref(w, i)? {
            BoneRef::Bone(o, k) => mul(&local, &self.get(w, o, k)),
            _ => local,
        })
    }
}
