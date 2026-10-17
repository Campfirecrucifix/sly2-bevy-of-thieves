//! WP-P17: two-bone IK of the JT rig (Sly's arms and legs). Without it every limb stays in its
//! rest pose (arms spread, legs together) whatever the ASEG plays, because the ASEGs only animate
//! the IK goal bones and the per-chain swivel (scalar channels), never the limb bones themselves.
//!
//! 1:1 parts:
//!  - FUN_00186670 (ALO load: u8 n x {s16 b1, s16 b2, s16 b3, vec3 +0xb0, s16 goal, vec3 +0xa0},
//!    0xe0-byte record; parsed by world::alo::ee_sub_186670) then FUN_0012dd28 per limb bone: a type-7
//!    motion node (vtable 0x2d2e40) with mode 0xd on b1 and 0xe on b2 (bone +0x98), +0x1c = record.
//!  - FUN_00186930 setup (rest): +0x10 / +0x50 = b1 / b2 locals, +0xc0 = |b2 - b1|, +0xc4 =
//!    |b3 * (+0xb0) - b2|, +0xc8 / +0x90 = FUN_0011e4b0(b2 - b1, end - b2) angle / axis, the axis
//!    taken into b2's parent frame (FUN_0022fcc8 with the FUN_0011b340 inverse). FUN_00186820 (ALO
//!    post-load) recomputes +0xc0 / +0xc4 with the same formula (same rest values here).
//!  - FUN_00186a50 solve (FUN_0022de88 bone loop, mode 0xd, in bone order): b1 = rest; law of
//!    cosines bend acos(((d^2 - l1^2 - l2^2) / (2 l1 l2)) clamped) - +0xc8 about +0x90 applied to b2's
//!    rest rows (FUN_0011b500 / FUN_0011b480); then b1 = rest rows * (aim * swivel): aim =
//!    FUN_0011e4b0(end - shoulder, goal - shoulder) about the normalised parent-frame axis, swivel =
//!    +0xcc (scalar curve +0xd0 at time *+0xd4, bound by the ASEG scalar channel FUN_00131c40 on b1)
//!    about the parent-frame shoulder -> goal direction.
//! Replaced: the scalar channel of a blend ASEG (kind-2 curve that FUN_00137a30 binds to b1) is
//! evaluated with anim::blend::BlendScalarCurve over the blend's entries (port's JtBlend keeps vector
//! channels only); FUN_0011e390 (perpendicular fallback when the two vectors are parallel) by the
//! smallest-component swap below; acosf (FUN_002456f0) of FUN_0011e4b0 gets a clamped argument.
//! Unported: FUN_00190d10 (JT +0x19cc record solved by the player code) and FUN_001b4888.

use super::{aseg_of, subtree, JtAnimator, JtPlay, JtRig};
use crate::engine::anim::actor::ActorPose;
use crate::engine::anim::blend::{BlendAseg, BlendEntry, BlendScalarCurve};
use crate::engine::anim::keys::{ScalarCurve, V3};
use crate::engine::anim::motion::ee_sub_11d608;
use crate::engine::anim::pose::{ee_sub_11c318, world_matrices, Mat4, IDENTITY};
use crate::engine::gfx::attach::{alo_of, ee_sub_22d6b0, BoneRef};
use crate::engine::world::World;
use std::collections::HashMap;

/// One FUN_00186670 record resolved to rig bones (+ the FUN_00186930 setup).
#[derive(Clone, Debug)]
pub struct JtIk {
    /// +0x00 goal bone, +0xa0 offset in it
    pub goal: usize,
    pub goal_off: V3,
    /// +0x04 / +0x08 / +0x0c: upper (mode 0xd), lower (mode 0xe), end bone; +0xb0 end offset
    pub b1: usize,
    pub b2: usize,
    pub b3: usize,
    pub end_off: V3,
    /// +0x10 / +0x50
    pub rest1: Mat4,
    pub rest2: Mat4,
    /// +0x90 hinge axis (b2's parent frame), +0xc0 / +0xc4 lengths, +0xc8 rest angle
    pub axis: V3,
    pub l1: f32,
    pub l2: f32,
    pub angle0: f32,
    /// +0xcc swivel (kept when no curve is bound, as the original keeps the field)
    pub swivel: f32,
}

/// IK records of the rig and the scalar (swivel) curves of every blend entry per record.
#[derive(Default, Clone, Debug)]
pub struct JtIkSet {
    pub built: bool,
    pub iks: Vec<JtIk>,
    /// blend field -> per record -> per blend entry: the entry ASEG's scalar curve on b1
    pub blend_curves: HashMap<u32, Vec<Vec<Option<ScalarCurve>>>>,
}

fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn dot(a: V3, b: V3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn len(a: V3) -> f32 {
    dot(a, a).sqrt()
}
fn cross(a: V3, b: V3) -> V3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
fn scale(a: V3, s: f32) -> V3 {
    [a[0] * s, a[1] * s, a[2] * s]
}
/// v * rows 0..2 (direction)
fn dir(m: &Mat4, v: V3) -> V3 {
    std::array::from_fn(|j| v[0] * m[0][j] + v[1] * m[1][j] + v[2] * m[2][j])
}
/// v * rows 0..2 + row 3 (point)
fn point(m: &Mat4, v: V3) -> V3 {
    let d = dir(m, v);
    [d[0] + m[3][0], d[1] + m[3][1], d[2] + m[3][2]]
}
fn trans(m: &Mat4) -> V3 {
    [m[3][0], m[3][1], m[3][2]]
}

/// FUN_0011e390: a vector perpendicular to `v` (REPLACED: smallest-component swap).
fn perp(v: V3) -> V3 {
    let (x, y, z) = (v[0].abs(), v[1].abs(), v[2].abs());
    if x <= y && x <= z {
        [0.0, v[2], -v[1]]
    } else if z <= y {
        [v[1], -v[0], 0.0]
    } else {
        [-v[2], 0.0, v[0]]
    }
}

/// FUN_0011e4b0(a, b, &axis): angle between a and b, axis = normalised a x b ((1,0,0) and 0 when
/// either is shorter than 1e-4).
pub fn ee_sub_11e4b0(a: V3, b: V3) -> (f32, V3) {
    let (la, lb) = (len(a), len(b));
    if la < 0.0001 || lb < 0.0001 {
        return (0.0, [1.0, 0.0, 0.0]);
    }
    let c = cross(a, b);
    let lc = len(c);
    let axis = if lc > 1e-8 { scale(c, 1.0 / lc) } else { perp(a) };
    ((dot(a, b) / (la * lb)).clamp(-1.0, 1.0).acos(), axis)
}

/// FUN_0011b500(angle, out, axis) = FUN_0011b480(out, sin, cos, axis): rotation rows for row
/// vectors, M[i][j] = c d_ij + (1 - c) a_i a_j + s e_ijk a_k.
pub fn ee_sub_11b500(angle: f32, a: V3) -> [[f32; 3]; 3] {
    let (s, c) = ee_sub_11d608(angle);
    let k = 1.0 - c;
    [
        [k * a[0] * a[0] + c, k * a[0] * a[1] + s * a[2], k * a[0] * a[2] - s * a[1]],
        [k * a[1] * a[0] - s * a[2], k * a[1] * a[1] + c, k * a[1] * a[2] + s * a[0]],
        [k * a[2] * a[0] + s * a[1], k * a[2] * a[1] - s * a[0], k * a[2] * a[2] + c],
    ]
}

fn mul3(a: &[[f32; 3]; 3], b: &[[f32; 3]; 3]) -> [[f32; 3]; 3] {
    std::array::from_fn(|i| std::array::from_fn(|j| (0..3).map(|k| a[i][k] * b[k][j]).sum()))
}
fn rows_times(m: &Mat4, r: &[[f32; 3]; 3]) -> Mat4 {
    let mut o = *m;
    for i in 0..3 {
        for j in 0..3 {
            o[i][j] = (0..3).map(|k| m[i][k] * r[k][j]).sum();
        }
        o[i][3] = 0.0;
    }
    o[3][3] = 1.0;
    o
}

/// FUN_0022fcc8(0, bone, v): v in the frame of world matrix `m` (FUN_0011b340 inverse rows).
fn into_frame(m: &Mat4, v: V3) -> V3 {
    dir(&ee_sub_11c318(m), v)
}

fn parent_world(pose: &ActorPose, b: usize) -> Mat4 {
    pose.bones[b].parent.map_or(IDENTITY, |p| pose.bones[p].world)
}

/// FUN_00186670 records of every ALO in the JT subtree, resolved to rig bones, with the
/// FUN_00186930 setup over the rest pose; sorted by b1 (FUN_0022de88 bone order).
pub fn build_iks(w: &World, rig: &JtRig) -> Vec<JtIk> {
    let mut out = vec![];
    for o in subtree(w, rig.jt) {
        let Some(a) = alo_of(w, o) else { continue };
        let res = |r: i16| match ee_sub_22d6b0(w, r, w.objects[o].owner, Some(o), BoneRef::None) {
            BoneRef::Bone(bo, bk) => rig.index.get(&(bo, bk)).copied(),
            _ => None,
        };
        for r in &a.base.r186670 {
            let (Some(b1), Some(b2), Some(b3), Some(goal)) = (res(r.refs[0]), res(r.refs[1]), res(r.refs[2]), res(r.r3)) else { continue };
            let rw = &rig.rest_world;
            let end = point(&rw[b3], r.v0);
            let (p1, p2) = (trans(&rw[b1]), trans(&rw[b2]));
            let (angle0, axis) = ee_sub_11e4b0(sub(p2, p1), sub(end, p2));
            let pw2 = rig.pose.bones[b2].parent.map_or(IDENTITY, |p| rw[p]);
            out.push(JtIk {
                goal,
                goal_off: r.v1,
                b1,
                b2,
                b3,
                end_off: r.v0,
                rest1: rig.pose.rest[b1],
                rest2: rig.pose.rest[b2],
                axis: into_frame(&pw2, axis),
                l1: len(sub(p2, p1)),
                l2: len(sub(end, p2)),
                angle0,
                swivel: 0.0,
            });
        }
    }
    out.sort_by_key(|k| k.b1);
    out
}

/// FUN_00186a50 on the posed rig (bone worlds current up to b1's parent and the goal).
pub fn ee_sub_186a50(pose: &mut ActorPose, ik: &JtIk) {
    let pw = parent_world(pose, ik.b1);
    pose.bones[ik.b1].local = ik.rest1;
    let s = point(&pw, trans(&ik.rest1));
    let g = point(&pose.bones[ik.goal].world, ik.goal_off);
    let d = sub(g, s);
    let dl = len(d);
    let c = ((dl * dl - ik.l1 * ik.l1) - ik.l2 * ik.l2) / ((ik.l1 + ik.l1) * ik.l2);
    let bend = c.clamp(-1.0, 1.0).acos();
    let m = ee_sub_11b500(bend - ik.angle0, ik.axis);
    pose.bones[ik.b2].local = rows_times(&ik.rest2, &m);
    // end point through the chain b3 .. up to b1's parent (b1 rest, b2 bent)
    let stop = pose.bones[ik.b1].parent;
    let mut v = ik.end_off;
    let mut b = Some(ik.b3);
    let mut guard = 0;
    while let Some(i) = b {
        if Some(i) == stop || guard > pose.bones.len() {
            break;
        }
        v = point(&pose.bones[i].local, v);
        b = pose.bones[i].parent;
        guard += 1;
    }
    let e = sub(point(&pw, v), s);
    let (aim, axis2) = ee_sub_11e4b0(e, d);
    let norm = |v: V3| {
        let l = len(v);
        if l > 0.0 { scale(v, 1.0 / l) } else { v }
    };
    let m2 = ee_sub_11b500(aim, norm(into_frame(&pw, axis2)));
    let m3 = ee_sub_11b500(ik.swivel, norm(into_frame(&pw, d)));
    pose.bones[ik.b1].local = rows_times(&ik.rest1, &mul3(&m2, &m3));
}

fn refresh_world(pose: &mut ActorPose) {
    let locals: Vec<Mat4> = pose.bones.iter().map(|b| b.local).collect();
    let parents: Vec<Option<usize>> = pose.bones.iter().map(|b| b.parent).collect();
    for (b, wm) in pose.bones.iter_mut().zip(world_matrices(&IDENTITY, &locals, &parents)) {
        b.world = wm;
    }
}

/// Build the records and the blend entries' swivel curves (needs the level; called from the
/// animator's requests).
pub fn ensure_built(an: &mut JtAnimator, w: &World) {
    if an.ik.built && an.ik.blend_curves.len() == an.blends.len() {
        return;
    }
    if !an.ik.built {
        an.ik.built = true;
        an.ik.iks = build_iks(w, &an.rig);
    }
    let mut curves = HashMap::new();
    for (&field, b) in &an.blends {
        let per: Vec<Vec<Option<ScalarCurve>>> = an
            .ik
            .iks
            .iter()
            .map(|ik| b.entries.iter().map(|e| e.aseg.and_then(|o| scalar_on(w, &an.rig, o, ik.b1))).collect())
            .collect();
        curves.insert(field, per);
    }
    an.ik.blend_curves = curves;
}

/// Scalar channel of ASEG object `o` bound to rig bone `b1`.
fn scalar_on(w: &World, rig: &JtRig, o: usize, b1: usize) -> Option<ScalarCurve> {
    let a = aseg_of(w, o)?;
    let binding = match rig.bindings.get(&o) {
        Some(b) => b.clone(),
        None => super::bind_channels(w, rig.jt, a, &rig.index),
    };
    a.channels.iter().zip(&binding).find(|(c, b)| c.scalar.is_some() && **b == Some(b1)).and_then(|(c, _)| c.scalar.clone())
}

/// Swivels of the current play (+0xcc via the bound curve), then FUN_00186a50 per record in bone
/// order, refreshing the worlds after each (FUN_0022de88 walks the bones in order).
pub fn apply(an: &mut JtAnimator) {
    if an.ik.iks.is_empty() {
        return;
    }
    for k in 0..an.ik.iks.len() {
        let b1 = an.ik.iks[k].b1;
        let sw = match &an.current {
            Some(JtPlay::Aseg(o, p)) => an.rig.bindings.get(o).and_then(|binding| {
                p.aseg.channels.iter().zip(binding).find(|(c, b)| c.scalar.is_some() && **b == Some(b1)).and_then(|(c, _)| c.scalar.as_ref()?.eval(p.state.t, 1.0)).map(|v| v.0)
            }),
            Some(JtPlay::Blend { field, t, .. }) => an.blends.get(field).and_then(|b| {
                let children = an.ik.blend_curves.get(field)?.get(k)?.clone();
                if children.iter().all(|c| c.is_none()) {
                    return None;
                }
                let curve = BlendScalarCurve {
                    blend: BlendAseg {
                        length: b.length,
                        entries: b.entries.iter().map(|e| BlendEntry { weight: e.weight, sub_length: e.sub_length, phase: vec![] }).collect(),
                        select: b.select,
                    },
                    phases: None,
                    children,
                    default: 0.0,
                };
                curve.ee_sub_138ba0(*t, 1.0).map(|v| v.0)
            }),
            None => None,
        };
        if let Some(s) = sw {
            an.ik.iks[k].swivel = s;
        }
    }
    for k in 0..an.ik.iks.len() {
        let ik = an.ik.iks[k].clone();
        ee_sub_186a50(&mut an.rig.pose, &ik);
        refresh_world(&mut an.rig.pose);
    }
}
