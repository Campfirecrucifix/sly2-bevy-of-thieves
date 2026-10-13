//! WP-P23: the controlled Bentley / Murray posed from its own anims (before P23 its ALO tree was drawn in rest
//! pose at the JT body).
//!
//! 1:1 source of the pose: the class code's ASEGAs on the K25 run time (`classes::k25::AnimRt`: FUN_0012f9b8
//! requests of its SetState FUN_00135248 / FUN_001b6078 into +0x1030, the SMA +0x1034 links started by
//! FUN_001f03a8) - every live ASEGA whose target (+0x18) is the po is bound onto the po's ALO bones
//! (FUN_0012fe58 channel binding `bind_channels`) at its own time (+0x1c, advanced by FUN_00130918 in the
//! class frame), oldest first (the live set order). The run blends (+0x1324 / +0x1604 / Murray +0x1464 ...,
//! FUN_0020b868 of the {+0x14, +0x54} entry ASEGs) are `jtblend::ee_sub_20b868` over the po rig, weighted by the
//! class's FUN_0020bb58(u, blk) of the frame (`EngineHost::po_bb58`).
//! REPLACED: the blend ASEGs are not run-time handles (the FUN_0020b868 / FUN_001a0300 hooks build nothing in
//! the class layer, so their SetState requests start no ASEGA): while the class weights a run block this frame
//! and no other ASEGA of the po is live, the drawn pose is that block's blend at a local time advanced here at
//! rate 1 (the ASEGA +0x40 of a flags-9 rate-1 request).
//! WP-P24 REPLACED (as the JT, playerdraw::BLEND_TIME): a change of the driving anim (run block, or the newest
//! live ASEGA) cross-fades the bone locals from the last pose over BLEND_TIME (FUN_0012f9b8 blend-in).
//! UNPORTED: the FUN_001a0300 plain blends (+0x14e0 / +0x15f0 / Murray +0x17f0) and their weights.
use super::jtblend::{ee_sub_20b868, JtBlend};
use super::{aseg_of, bind_channels, blend_local, ee_sub_15e938_below, JtRig, BLEND_TIME};
use crate::engine::anim::pose::Mat4;
use crate::engine::anim::play::bind_segments;
use crate::engine::anim::pose::IDENTITY;
use crate::engine::world::World;
use std::collections::HashMap;

/// What drove the pose this frame.
#[derive(Clone, Debug, PartialEq)]
pub enum PoSource {
    /// nothing live: the last pose is held (ALO xforms keep their last written value)
    Hold,
    /// live ASEGAs of the po: ASEG world objects
    Asega(Vec<usize>),
    /// run block (its FUN_0020bb58 weight u)
    Blend(u32, f32),
}

pub struct PoAnimator {
    pub po: usize,
    pub rig: JtRig,
    /// run block -> blend (FUN_0020b868 of the block's entry ASEGs below the po)
    pub blends: HashMap<u32, JtBlend>,
    /// ASEG object -> channel bones (FUN_0012fe58)
    pub bindings: HashMap<usize, Vec<Option<usize>>>,
    pub blk: Option<u32>,
    pub blend_t: f32,
    pub source: PoSource,
    /// WP-P23: last frame's time per live ASEGA (ASEG object) and the timed events reached this frame (ASEG
    /// object, event index, reverse) for the FUN_00131290 dispatch
    pub last_t: HashMap<usize, f32>,
    pub fired: Vec<(usize, usize, bool)>,
    /// WP-P24: the driving anim (run block / newest ASEGA object), the bone locals at its change, fade time left
    pub key: Option<(u8, usize)>,
    pub from: Vec<Mat4>,
    pub fade: f32,
}

impl PoAnimator {
    pub fn new(w: &World, po: usize) -> PoAnimator {
        let rig = JtRig::build(w, po);
        let rows: &[(u32, i32, i32)] = match w.objects[po].class {
            crate::engine::player::p20::CLASS_BENTLEY => &crate::engine::player::p22::LOCO_55,
            crate::engine::player::p20::CLASS_MURRAY => &crate::engine::player::p22::LOCO_54,
            _ => &[],
        };
        let mut blends = HashMap::new();
        for &(blk, o0, o1) in rows {
            let a = [ee_sub_15e938_below(w, po, o0), ee_sub_15e938_below(w, po, o1)].map(|o| o.filter(|&o| aseg_of(w, o).is_some()));
            if let Some(b) = ee_sub_20b868(w, &rig, a) {
                blends.insert(blk, b);
            }
        }
        PoAnimator { po, rig, blends, bindings: HashMap::new(), blk: None, blend_t: 0.0, source: PoSource::Hold, last_t: HashMap::new(), fired: vec![], key: None, from: vec![], fade: 0.0 }
    }

    /// One frame: `plays` = live ASEGAs on the po (ASEG object, time) oldest first, `bb58` = the class's
    /// FUN_0020bb58(u, blk) of this frame.
    pub fn step(&mut self, w: &World, plays: &[(usize, f32)], bb58: Option<(u32, f32)>, dt: f32) {
        self.fired.clear();
        for b in &mut self.rig.pose.bones {
            b.motion = 0;
        }
        let bb = bb58.filter(|b| self.blends.contains_key(&b.0));
        let key = if plays.is_empty() { bb.map(|b| (1u8, b.0 as usize)) } else { plays.last().map(|p| (2u8, p.0)) };
        if key.is_some() && key != self.key {
            self.from = self.rig.pose.bones.iter().map(|b| b.local).collect();
            self.fade = BLEND_TIME;
        }
        if key.is_some() {
            self.key = key;
        }
        if plays.is_empty() {
            if let Some((blk, u)) = bb {
                if self.blk != Some(blk) {
                    self.blk = Some(blk);
                    self.blend_t = 0.0;
                }
                let b = self.blends.get_mut(&blk).expect("blend");
                b.ee_sub_20bb58(u, std::slice::from_mut(&mut self.blend_t));
                let t0 = self.blend_t;
                self.blend_t += dt;
                if b.length > 0.0 {
                    self.blend_t = self.blend_t.rem_euclid(b.length);
                }
                if self.blend_t != t0 {
                    self.fired.extend(crate::engine::player::p23::blend_events_between(w, b, t0, self.blend_t).into_iter().map(|(o, i)| (o, i, false)));
                }
                let (segments, binding) = b.segments(self.blend_t);
                self.rig.pose.bind(&segments, &binding, self.blend_t, 1.0);
                self.source = PoSource::Blend(blk, u);
            } else {
                self.blk = None;
                self.source = PoSource::Hold;
            }
        } else {
            self.blk = None;
            for &(o, t) in plays {
                let Some(a) = aseg_of(w, o) else { continue };
                let (po, index) = (self.po, &self.rig.index);
                let binding = self.bindings.entry(o).or_insert_with(|| bind_channels(w, po, a, index));
                let segments = bind_segments(a, t);
                self.rig.pose.bind(&segments, binding, t, 1.0);
            }
            self.source = PoSource::Asega(plays.iter().map(|p| p.0).collect());
        }
        // events: the K25 run time's FUN_00130918 only counts them; the window since last frame is replayed here
        let mut last = HashMap::new();
        for &(o, t) in plays {
            let Some(a) = aseg_of(w, o) else { continue };
            let t0 = self.last_t.get(&o).copied().unwrap_or(-1.0);
            self.fired.extend(crate::engine::player::p23::events_between(a, t0, t).into_iter().map(|i| (o, i, false)));
            last.insert(o, t);
        }
        self.last_t = last;
        self.rig.pose.compose(&IDENTITY);
        if self.fade > 0.0 && self.from.len() == self.rig.pose.bones.len() {
            self.fade = (self.fade - dt).max(0.0);
            let u = 1.0 - self.fade / BLEND_TIME;
            let locals: Vec<Mat4> = self.rig.pose.bones.iter().zip(&self.from).map(|(b, f)| blend_local(f, &b.local, u)).collect();
            let parents: Vec<Option<usize>> = self.rig.pose.bones.iter().map(|b| b.parent).collect();
            let worlds = crate::engine::anim::pose::world_matrices(&IDENTITY, &locals, &parents);
            for ((b, l), wm) in self.rig.pose.bones.iter_mut().zip(locals).zip(worlds) {
                b.local = l;
                b.world = wm;
            }
        }
    }

    /// Bones driven by at least one channel of the current source (diagnostics / tests).
    pub fn bound_bones(&self) -> usize {
        match &self.source {
            PoSource::Hold => 0,
            PoSource::Blend(blk, _) => self.blends[blk].channels.iter().filter(|c| c.bone.is_some()).count(),
            PoSource::Asega(v) => v.iter().filter_map(|o| self.bindings.get(o)).map(|b| b.iter().flatten().count()).sum(),
        }
    }
}

pub use crate::engine::player::p23::po_plays;
