//! WP-D6: plain blends (FUN_001a0300) and the cross-fade between animations, for any ALO actor
//! (P24 Bentley / Murray, NPCs). Output = bone locals for [`super::bind::AnimBinder::compose`].
//!
//! 1:1 parts:
//! - FUN_0012ff10(aseg, alo, kind, flagged): first channel (only bit-30 channels when `flagged`)
//!   with a vector curve whose oid is the ALO's oid (+0x18) and whose bone ref, resolved by the
//!   ALO's +0x19c (FUN_0022d6b0), names a bone of kind (+0x94) `kind`.
//! - FUN_001a0300(alo, entries[8] stride 0x20, &out): per entry with an ASEG (+4), the root channel
//!   FUN_0012ff10(aseg, alo, 1, 1) gives the entry velocity +0x10 = (curve(0) - curve(len)) / len
//!   (curve slot +0x1c evaluated at 0 and at ASEG +0x74 with rate 1; VU0 vsub d0 - c0 times 1/len,
//!   as decompiled), then FUN_00137a30 builds the blend over the 8 entries.
//! - The blend itself is the kind-5 curve of anim::blend (FUN_00138660: p = sum(w * pos), weights as
//!   given, entries with weight 0 skipped), bound per channel like actor::bind_blend_vec.
//!
//! REPLACED: FUN_00137a30's blend ASEG record (+0x74 length) is modelled as the weight-averaged
//! sub-length; the cross-fade between two animations is [`CrossFade`]: bone locals at the switch
//! slerped into the new pose over `time` (playerdraw::blend_local) — the original starts the new
//! ASEGA's acts on top of the old ones (FUN_00230018) and FUN_00230cf0 hands the bone state over
//! (its VU0 velocity carry-over is not decompiled).

use super::actor::set_channel;
use super::aseg::Aseg;
use super::bind::{alo_data, channel_bone_ref, channel_oid, ee_sub_12fe58, ee_sub_22d778, BoneKey};
use super::blend::{BlendAseg, BlendEntry};
use super::keys::{eval_segment, V3};
use crate::engine::gfx::scene::{Affine, IDENTITY};
use crate::engine::playerdraw::{aseg_of, blend_local};
use crate::engine::world::World;
use std::collections::HashMap;

/// FUN_0012ff10 (0x0012ff10): index of the channel of `aseg` driving a bone of kind `kind` of `alo`.
pub fn ee_sub_12ff10(w: &World, aseg: &Aseg, alo: usize, kind: i32, flagged: bool) -> Option<usize> {
    let oid = w.objects.get(alo)?.oid;
    alo_data(w, alo)?;
    aseg.channels.iter().position(|c| {
        (!flagged || c.flag)
            && channel_oid(c) == oid
            && c.vec.is_some()
            && ee_sub_22d778(w, alo, channel_bone_ref(c)).and_then(|(o, b)| alo_data(w, o).and_then(|a| a.base.bones.get(b))).is_some_and(|b| b.kind as i32 == kind)
    })
}

/// One plain-blend entry (stride 0x20): +0 weight, +4 ASEG, +0x10 velocity.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PlainEntry {
    pub weight: f32,
    /// ASEG object
    pub aseg: Option<usize>,
    pub vel: V3,
}

/// FUN_001a0300 (0x001a0300) velocity part: entry velocities from the root channels.
pub fn ee_sub_1a0300(w: &World, alo: usize, entries: &mut [PlainEntry]) {
    for e in entries.iter_mut().take(8) {
        let Some(a) = e.aseg.and_then(|o| aseg_of(w, o)) else { continue };
        let Some(c) = ee_sub_12ff10(w, a, alo, 1, true) else { continue };
        let Some(curve) = a.channels[c].vec.as_ref() else { continue };
        let (Some(d0), Some(c0)) = (curve.eval(0.0, 1.0), curve.eval(a.length, 1.0)) else { continue };
        let q = if a.length != 0.0 { 1.0 / a.length } else { 0.0 };
        e.vel = [0, 1, 2].map(|i| (d0[i] - c0[i]) * q);
    }
}

/// Posed bone locals of the blend of `entries` at time `t` (rate `rate`), channels bound with
/// FUN_0012fe58 on `target` (the actor). Bones driven by no entry are absent.
pub fn blend_locals(w: &World, target: usize, entries: &[PlainEntry], t: f32, rate: f32) -> HashMap<BoneKey, Affine> {
    let subs: Vec<(usize, &Aseg)> = entries.iter().filter_map(|e| e.aseg.and_then(|o| aseg_of(w, o).map(|a| (o, a)))).collect();
    let mut out = HashMap::new();
    if subs.is_empty() {
        return out;
    }
    let wsum: f32 = entries.iter().filter(|e| e.aseg.is_some()).map(|e| e.weight).sum();
    let length = if wsum != 0.0 { entries.iter().filter_map(|e| e.aseg.and_then(|o| aseg_of(w, o)).map(|a| e.weight * a.length)).sum::<f32>() / wsum } else { subs[0].1.length };
    let blend = BlendAseg {
        length,
        entries: entries.iter().filter(|e| e.aseg.is_some()).zip(&subs).map(|(e, s)| BlendEntry { weight: e.weight, sub_length: s.1.length, phase: vec![] }).collect(),
        select: -1,
    };
    let refs: Vec<&Aseg> = subs.iter().map(|s| s.1).collect();
    let mut seen = std::collections::HashSet::new();
    for &(ao, a) in &subs {
        for ch in &a.channels {
            if ch.flag || ch.vec.is_none() || !seen.insert((ch.oid, ch.kind)) {
                continue;
            }
            let (_, Some(bone)) = ee_sub_12fe58(w, ao, Some(target), ch) else { continue };
            let Some(ab) = alo_data(w, bone.0).and_then(|x| x.base.bones.get(bone.1)) else { continue };
            let curve = super::actor::bind_blend_vec(blend.clone(), &refs, ch.oid, ch.kind, [0.0; 3]);
            let Some(seg) = curve.ee_sub_138660(t) else { continue };
            let v = eval_segment(&seg, t, rate);
            let mut l4 = to4(&ab.xform.unwrap_or(IDENTITY));
            set_channel(ab.kind as i32, &mut l4, v);
            out.insert(bone, to3(&l4));
        }
    }
    out
}

fn to4(m: &Affine) -> super::pose::Mat4 {
    [0, 1, 2, 3].map(|r| [m[r][0], m[r][1], m[r][2], (r == 3) as u32 as f32])
}

fn to3(m: &super::pose::Mat4) -> Affine {
    [0, 1, 2, 3].map(|r| [m[r][0], m[r][1], m[r][2]])
}

/// Cross-fade between animations of one actor (REPLACED, see module doc).
#[derive(Clone, Debug, Default)]
pub struct CrossFade {
    /// bone locals at the switch
    pub from: HashMap<BoneKey, Affine>,
    /// fade length (seconds) and time left
    pub time: f32,
    pub left: f32,
}

impl CrossFade {
    /// Start a fade from the `current` locals (the pose shown before the switch).
    pub fn start(&mut self, current: &HashMap<BoneKey, Affine>, time: f32) {
        self.from = current.clone();
        self.time = time;
        self.left = time;
    }

    /// Advance by `dt` and blend `locals` (the new animation) from the start pose. Bones only in
    /// the start pose fade back to `rest(bone)`.
    pub fn apply(&mut self, dt: f32, locals: &mut HashMap<BoneKey, Affine>, rest: &dyn Fn(BoneKey) -> Affine) {
        if self.left <= 0.0 || self.time <= 0.0 {
            return;
        }
        self.left = (self.left - dt).max(0.0);
        let u = 1.0 - self.left / self.time;
        for (k, f) in &self.from {
            let to = locals.get(k).copied().unwrap_or_else(|| rest(*k));
            locals.insert(*k, to3(&blend_local(&to4(f), &to4(&to), u)));
        }
        if self.left <= 0.0 {
            self.from.clear();
        }
    }

    pub fn active(&self) -> bool {
        self.left > 0.0
    }
}
