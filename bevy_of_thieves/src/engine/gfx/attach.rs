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

fn r80_xfm_off() -> bool {
    static O: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *O.get_or_init(|| std::env::var_os("SLY2_R80XFM").is_none())
}

/// ANIM-ALL R81: emit frames from the live world (classes::plugin ee_sub_15bd20); follows the R80 XFM switch
pub fn r81_frame_on() -> bool {
    !r80_xfm_off() && std::env::var_os("SLY2_R81FRAMEOFF").is_none()
}

/// Class-3 style (FUN_00231768) bone ref + local matrix of a non-ALO object.
pub fn xfm_of(w: &World, i: usize) -> Option<(i16, Affine)> {
    let (s, x) = match &w.objects.get(i)?.payload {
        Payload::Loaded(LoadData::L231170 { s, xform, .. }) => (*s, xform),
        Payload::Loaded(LoadData::L227380 { xform, .. }) | Payload::Loaded(LoadData::L231170Blob { xform, .. }) => (xform.s, &xform.xform),
        Payload::Loaded(LoadData::Solid(sol)) => (sol.xform.s, &sol.xform.xform),
        Payload::Loaded(LoadData::L2282b0 { solid, .. }) => (solid.xform.s, &solid.xform.xform),
        // HEIST r41: an in-place FUN_001c9b08 record (class-6 spawner) keeps the bone ref at +0x50 and the xform at
        // +0x60..+0x9c (text_001c0000.c FUN_001c9b08; EP4 Disguise Bridge 77238 local (6994, -9127, -423))
        Payload::Loaded(LoadData::B4(b)) if let crate::engine::world::loads_b4::B4Load::Class6 { in_place: true, bone, xform, .. } = &**b => (*bone, xform),
        // ANIM-ALL R80: the FUN_00231170-loaded B4 classes (29 emitters FUN_00258598, 31 FUN_0015c1e8, 43 / 45 / 46)
        // are XFM objects too (class 29 vtable +0xfc = FUN_00231440: world = +0x60 local x +0x50 bone); without them
        // every emitter frame was the identity and all class-28 particles spawned at the level origin
        // OPT-IN SLY2_R80XFM=1 (+ SLY2_R80XFMALL=1 for 31/43/45/46): the emitter frames shift the shared GameRng draws and
        // break the RNG-dependent wired_guards_hear_search_and_return_live (guard picks); default = old (identity)
        Payload::Loaded(LoadData::B4(b)) if !r80_xfm_off() => match &**b {
            crate::engine::world::loads_b4::B4Load::L258598 { xform, .. } => (xform.s, &xform.xform),
            crate::engine::world::loads_b4::B4Load::L15c1e8 { xform, .. } | crate::engine::world::loads_b4::B4Load::L231170 { xform, .. }
                if std::env::var_os("SLY2_R80XFMALL").is_some() => (xform.s, &xform.xform),
            _ => return None,
        },
        // ANIM-ALL R81: a FUN_0013e510 copy of an emitter (vt+0x144 copies the +0x50 / +0x60 XFM state of the source):
        // the source's xform, its bone ref resolved against the copy's own owner (SLY2_R81CLONEOFF = old: no xfm)
        Payload::Clone if !r80_xfm_off() && std::env::var_os("SLY2_R81CLONEOFF").is_none() => {
            let mut r = i;
            while let Some(c) = w.objects[r].clone_of.filter(|&c| c != r) {
                r = c;
            }
            match &w.objects[r].payload {
                Payload::Loaded(LoadData::B4(b)) if let crate::engine::world::loads_b4::B4Load::L258598 { xform, .. } = &**b => (xform.s, &xform.xform),
                _ => return None,
            }
        }
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
///
/// FUN_001f21b8 full body (684 bytes):
///   bit 1,2: build cumulative bone matrix via FUN_0011c168 chain (+0x60 linked list +0xa0) or
///            copy +0x40..+0x7c from first bone (bit 0x3c set).
///   bit 4:   read position via vtable +0x10c(+0x108 offset).
///   bit 8:   read scale via vtable +0x114(+0x110 offset).
///   Then re-parent: for each of +0x50 entries in +0x54 array (0x100-byte records),
///     if record +0xa0 is in old parent bone range [iVar8, iVar8+iVar7), set +0xa0 = param_3.
///   Calls FUN_001a5900 (velocity update), FUN_0022faa0 (bone worlds refresh),
///     vtable +0xec/+0xf4/+0x11c/+0x124 setters (position/rotation/scale),
///     FUN_001f2468 + FUN_001f26a8 (attachment refresh), FUN_001f8668 (SO refresh if +0x10 ALO).
///   The ECS port handles static parenting at load time; dynamic runtime re-parenting
///   (velocity hooks, vtable position/rotation/scale setters, attachment + SO refresh)
///   requires raw-pointer ALO layout not available in the Bevy ECS architecture.
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

/// LOOK R27: FUN_001c9b08 in-place branch: the class-6 spawner keeps the record's xform at +0x60, its bone at
/// +0x50 (FUN_0022d6b0(bone, owner)) and its owner at +0x10; FUN_001caec8 hands those to every claimed copy
/// (vt+0x54 / vt+0x12c), so a copy sits where a FUN_001ca370 copy of the same record would (`template_socket`).
/// Returns (bone ref, record xform); the bone ref is None when the owner has no ALO bone for it.
pub fn spawner_socket(w: &World, sp: usize) -> Option<(BoneRef, Affine)> {
    use crate::engine::world::{loads_b4::B4Load, objects::{LoadData, Payload}};
    let mut r = sp;
    while let Some(c) = w.objects[r].clone_of.filter(|&c| c != r) {
        r = c;
    }
    let Payload::Loaded(LoadData::B4(d)) = &w.objects[r].payload else { return None };
    let B4Load::Class6 { in_place: true, bone, xform, .. } = &**d else { return None };
    let pm = affine_from_bytes(xform);
    let Some(o) = w.objects[sp].owner else { return Some((BoneRef::None, pm)) };
    let b = match (alo_of(w, o), ee_sub_22d6b0(w, *bone, Some(o), None, BoneRef::None)) {
        (_, b @ BoneRef::Bone(..)) => b,
        (Some(oa), _) => ee_sub_22d6b0(w, oa.base.refs[2], w.objects[o].owner, Some(o), context_of(w, o)),
        (None, _) => BoneRef::None,
    };
    Some((if matches!(b, BoneRef::Bone(..)) { b } else { BoneRef::None }, pm))
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
