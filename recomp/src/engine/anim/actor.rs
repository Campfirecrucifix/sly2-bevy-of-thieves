//! WP-D3: actor pose composition (ASEG channel segments -> bone locals -> world matrices, with
//! the D2 bone velocity matrices) and run-time binding of class-110 blend curves.
//!
//! 1:1 parts: channel value -> local matrix part per bone kind (FUN_0022cc00 layout, inverse of
//! FUN_0022ca18), Hermite segment evaluation (FUN_00117120), parent chain world walk
//! (FUN_0022faa0: world = local * parent world, null parent = root), velocity (FUN_0022cdc0 /
//! FUN_0022d148). Replaced: the euler -> matrix write for kinds 6/7 is the exact inverse of
//! FUN_0011bff0 (the original writes it through VU0 code that is not decompiled); the basis row
//! lengths (scale/skew) of the rest matrix are kept. Unported: oid -> bone resolution of the
//! channel targets (vtable +0x19c); callers pass the binding.

use super::aseg::Aseg;
use super::blend::{BlendAseg, BlendVecCurve};
use super::keys::{eval_segment, Segment, V3};
use super::motion::{ee_sub_11d608, ee_sub_22cdc0, ee_sub_22d148, BoneState};
use super::pose::{world_matrices, Mat4};

/// Rotation rows whose FUN_0011bff0 decomposition is `e` = (x, y, z):
/// row0 = (cy cz, cy sz, -sy), row1 = (sx sy cz - cx sz, sx sy sz + cx cz, sx cy),
/// row2 = (cx sy cz + sx sz, cx sy sz - sx cz, cx cy). Sin/cos via FUN_0011d608.
pub fn euler_rows(e: V3) -> [V3; 3] {
    let (sx, cx) = ee_sub_11d608(e[0]);
    let (sy, cy) = ee_sub_11d608(e[1]);
    let (sz, cz) = ee_sub_11d608(e[2]);
    [
        [cy * cz, cy * sz, -sy],
        [sx * sy * cz - cx * sz, sx * sy * sz + cx * cz, sx * cy],
        [cx * sy * cz + sx * sz, cx * sy * sz - sx * cz, cx * cy],
    ]
}

/// Write a channel value into the bone's local matrix (inverse of FUN_0022ca18).
pub fn set_channel(kind: i32, local: &mut Mat4, x: V3) {
    match kind {
        1 | 4 | 5 | 9 | 10 => local[3][..3].copy_from_slice(&x),
        6 | 7 => {
            let r = euler_rows(x);
            for i in 0..3 {
                let len = (0..3).map(|j| local[i][j] * local[i][j]).sum::<f32>().sqrt();
                let len = if len > 0.0 { len } else { 1.0 };
                for j in 0..3 {
                    local[i][j] = r[i][j] * len;
                }
            }
        }
        0xb => {
            local[1][0] = x[0];
            local[2][0] = x[1];
            local[2][1] = x[2];
        }
        0xc => {
            local[0][0] = x[0];
            local[1][1] = x[1];
            local[2][2] = x[2];
        }
        _ => {}
    }
}

/// One actor: runtime bones + rest locals.
#[derive(Clone, Debug)]
pub struct ActorPose {
    pub bones: Vec<BoneState>,
    pub rest: Vec<Mat4>,
}

impl ActorPose {
    /// Bones from (kind, rest local, parent); parents must precede children.
    pub fn new(spec: &[(i32, Mat4, Option<usize>)]) -> Self {
        let bones = spec
            .iter()
            .map(|&(k, l, p)| {
                let mut b = BoneState::new(k, l);
                b.parent = p;
                b
            })
            .collect();
        ActorPose { bones, rest: spec.iter().map(|s| s.1).collect() }
    }

    /// Set channel segments for this frame: `binding[c]` = bone driven by channel `c`.
    /// Bones with a segment get motion mode 6 (segment-driven, FUN_0022cdc0) and time `t`.
    pub fn bind(&mut self, segments: &[Option<Segment>], binding: &[Option<usize>], t: f32, rate: f32) {
        for (s, b) in segments.iter().zip(binding) {
            if let (Some(s), Some(b)) = (s, b) {
                let bone = &mut self.bones[*b];
                bone.seg = s.clone();
                bone.motion = 6;
                bone.t = t;
                bone.rate = rate;
            }
        }
    }

    /// Pose: evaluate each driven bone's segment (FUN_00117120 via eval_segment), write it
    /// into the local, then the world walk from `root`.
    pub fn compose(&mut self, root: &Mat4) {
        for (b, rest) in self.bones.iter_mut().zip(&self.rest) {
            b.local = *rest;
            if b.motion == 6 {
                let v = eval_segment(&b.seg, b.t, b.rate);
                set_channel(b.kind, &mut b.local, v);
            }
        }
        let locals: Vec<Mat4> = self.bones.iter().map(|b| b.local).collect();
        let parents: Vec<Option<usize>> = self.bones.iter().map(|b| b.parent).collect();
        for (b, w) in self.bones.iter_mut().zip(world_matrices(root, &locals, &parents)) {
            b.world = w;
        }
    }

    /// World matrices of the current pose.
    pub fn world(&self) -> Vec<Mat4> {
        self.bones.iter().map(|b| b.world).collect()
    }

    /// FUN_0022cdc0 local velocity of bone `i`.
    pub fn local_velocity(&self, i: usize) -> Mat4 {
        ee_sub_22cdc0(&self.bones, i)
    }

    /// FUN_0022d148 velocity of bone `i` in world space.
    pub fn world_velocity(&self, i: usize) -> Mat4 {
        ee_sub_22d148(&self.bones, i, None)
    }
}

/// Bind a kind-5 blend curve at run time (the original stores, per entry, the sub-ASEG's curve
/// for the same channel into +0xc; NULL when the sub-segment has no such channel). Channels
/// match by (target oid, channel kind).
pub fn bind_blend_vec(blend: BlendAseg, subs: &[&Aseg], oid: u32, kind: u16, default: V3) -> BlendVecCurve {
    let children = subs
        .iter()
        .map(|a| a.channels.iter().find(|c| c.oid == oid && c.kind == kind).and_then(|c| c.vec.clone()))
        .collect();
    BlendVecCurve { blend, phases: None, children, default }
}
