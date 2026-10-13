//! WP-A4: animate the non-player skinned ALOs (K6/K7 guards, class 56 family, and the K8 pickups) through
//! the same anim/ path as Sly (playerdraw::JtRig bones, ASEG channel binding FUN_0012fe58, ASEGA playback,
//! anim::sm SMAs), driven by the class state:
//!
//! * guards: the anim the ported goal stack would hand its anim task (0x9d / 0x9e) - walk (anim id 9, the
//!   0xa7 walk task +0xc0) while the body moves, at the playback rate of the K7 root motion (FUN_00209618:
//!   body +0x190 = root channel velocity * rate, so rate = |+0x190| / |root velocity at rate 1|); the
//!   reaction anim 0xb of brain substate 3 (Child::React) from its start time; a running K11 attack
//!   (+0xf00, the 0xad attack goal of chase state 2) plays its record's ASEG (C2da840 +0xb4 oid) from the
//!   attack start; the chase goal 0xa5 (brain substate 4, Child::Chase) in its chase state 3
//!   (FUN_00176458(goal, 3): move task 0xba with +0xac = 1 -> FUN_0017d450 walk task 0xa7 +0xc0 = 9)
//!   plays the locomotion anim 9 (REPLACED: the brain does not move the guard while chasing, so the anim
//!   runs at its authored rate 1); otherwise the idle anim (id 1, an SM object: the idle fidget SMA).
//!   A6: the stun FX subtree ([`stun_fx`]) is drawn while the guard is knocked out (K32
//!   `Guard::knocked_out`: brain substate 0xb of FUN_00176be8); stagger / knock-out play anims 5 / 6 / 7.
//! * pickups: the SMA FUN_001c60b8 starts / retargets (FUN_001f0380 / FUN_001f0660) on the pickup's SM
//!   (+0x164: the class-111 object of its template), and its FUN_001f13a8 signals go to the pickup's
//!   slot-4 handler FUN_001c5098 (vanish 0x186 reached -> SetState(2)).
//!
//! The root channel (flagged vector channel on a kind-1 bone, the one FUN_00204e38 reads) is root motion:
//! the class moves the body by it, so its horizontal part is not applied to the bone (REPLACED: the
//! original's root bone is the body itself).
//!
//! Drawing (feature `game`, plugin.rs): template copies (most guards and pickups are `Payload::Clone`
//! objects without meshes of their own) get a copy of the template's spawned mesh entities; the root takes
//! the class transform (placement rotation turned by the body yaw change, body position).

pub mod objdata;
#[cfg(feature = "game")]
pub mod objects;
#[cfg(feature = "game")]
pub mod plugin;
pub mod script;

use crate::engine::anim::aseg::Aseg;
use crate::engine::anim::bind::{AnimBinder, BoneKey};
use crate::engine::anim::play::{bind_segments, drive, AnimClock, AsegaPlayer};
use crate::engine::gfx::scene::Affine;
use crate::engine::anim::pose::{Mat4, IDENTITY};
use crate::engine::anim::sm::{Sm, Sma, SmaMsg, SmaReceiver};
use crate::engine::classes::guard::brain::Child;
use crate::engine::classes::guard::c56::Guard;
use crate::engine::classes::guard::los;
use crate::engine::classes::k8::Cx;
use crate::engine::classes::{ClassHost, ClassSet, Inst};
use crate::engine::physics::driven::GameRng;
use crate::engine::playerdraw::{aseg_of, bind_channels, blend_local, ee_sub_15e938_below, subtree, JtRig, BLEND_TIME};
use crate::engine::world::objects::{LoadData, Payload};
use crate::engine::world::World;
use std::collections::HashMap;
use std::sync::Arc;

/// Guard anim ids (field-243 table): walk (0xa7 +0xc0 default), reaction (brain substate 3), idle SM.
pub const GUARD_ANIM_WALK: i32 = 9;
pub const GUARD_ANIM_REACT: i32 = 0xb;
pub const GUARD_ANIM_IDLE: i32 = 1;
/// SM class (load FUN_001efac0)
pub const CLASS_SM: u16 = 111;
/// SMA mask for the NPC SMAs: FUN_001f0380 (the FUN_001c60b8 / FUN_0017f770 / FUN_001d1518 entry) calls
/// FUN_001f01b8 with `t0` (5th argument, SMA +0x48) = 4 (`addiu t0, zero, 4` at 0x1f0384; Ghidra drops
/// the register argument).
pub const NPC_SMA_MASK: u32 = 4;
/// Locomotion anim of the chase move (0xa7 +0xc0 from FUN_0017d450 with 0xba +0xa8 target, +0xac = 1).
pub const GUARD_ANIM_CHASE: i32 = 9;
/// Classes of the stun FX parts: sound emitter (C2cabd8) and ASEG
const CLASS_SOUND: u16 = 43;
const CLASS_ASEG: u16 = 109;

/// What an NPC plays this frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NpcAnim {
    /// anim id of the owner's table at a playback rate (looped)
    Loop { id: i32, rate: f32 },
    /// anim id from a start time (clamped at the end)
    Once { id: i32, t: f32 },
    /// SMA on an SM object in state `label`
    Sm { sm: usize, label: i32 },
    /// a given ASEG object from a start time (clamped at the end)
    Aseg { obj: usize, t: f32 },
    /// rest pose
    None,
}

/// The guard's anim (see the module doc). `now` = class clock (ClassGlobals::t).
pub fn guard_anim(w: &World, gi: usize, g: &Guard, now: f32, t_walk: f32) -> NpcAnim {
    // K11 running attack: its record's ASEG (the +0xb0 binding) from the attack start
    if let Some(r) = g.atk.f_f00.and_then(|k| g.atk.recs.get(k)) {
        let oid = r.rec.f_b4 as i32;
        let obj = ee_sub_15e938_below(w, template_of(w, gi), oid).or_else(|| w.find_oid(oid).first().copied());
        if let Some(obj) = obj.filter(|&k| aseg_of(w, k).is_some()) {
            return NpcAnim::Aseg { obj, t: now - g.atk.t0 };
        }
    }
    // K32: hit goals (stagger 5 / heavy stagger 6 / knocked out 7: the table anim of the goal's behaviour bit)
    if let Child::Hit(h) = &g.brain.child {
        if h.bit >= 0 && anim_object(w, gi, h.bit).is_some() {
            return NpcAnim::Once { id: h.bit, t: now - h.t0 };
        }
    }
    let v = g.body.f_190;
    let speed = (v[0] * v[0] + v[1] * v[1]).sqrt();
    if speed > 1.0e-3 {
        let r = los::root_velocity(w, gi, GUARD_ANIM_WALK, t_walk, 1.0);
        let r1 = (r[0] * r[0] + r[1] * r[1]).sqrt();
        let rate = if r1 > 1.0e-3 { speed / r1 } else { 1.0 };
        return NpcAnim::Loop { id: GUARD_ANIM_WALK, rate };
    }
    if let Child::React(t0, _) = g.brain.child {
        return NpcAnim::Once { id: GUARD_ANIM_REACT, t: now - t0 };
    }
    if let Child::Chase = g.brain.child {
        if anim_object(w, gi, GUARD_ANIM_CHASE).is_some_and(|k| aseg_of(w, k).is_some()) {
            return NpcAnim::Loop { id: GUARD_ANIM_CHASE, rate: 1.0 };
        }
    }
    match anim_object(w, gi, GUARD_ANIM_IDLE) {
        Some(sm) if w.objects[sm].class == CLASS_SM => NpcAnim::Sm { sm, label: -1 },
        Some(_) => NpcAnim::Loop { id: GUARD_ANIM_IDLE, rate: 1.0 },
        None => NpcAnim::None,
    }
}

/// Template (mesh / skeleton owner) of an object: the clone source.
pub fn template_of(w: &World, o: usize) -> usize {
    w.objects[o].clone_of.unwrap_or(o)
}

/// Object of anim `id` of a guard: the table oid (los::anim_oid) looked up below the guard's template
/// (FUN_0015e938), else anywhere.
pub fn anim_object(w: &World, gi: usize, id: i32) -> Option<usize> {
    let oid = los::anim_oid(w, gi, id)?;
    let ok = |k: usize| aseg_of(w, k).is_some() || w.objects[k].class == CLASS_SM;
    ee_sub_15e938_below(w, template_of(w, gi), oid).filter(|&k| ok(k)).or_else(|| w.find_oid(oid).iter().copied().find(|&k| ok(k)))
}

/// Stun FX of a guard template (REPLACED, see the module doc): the subtrees of the template's direct
/// children that hold a self-animated effect - an ALO with a sound emitter child (class 43, the chirp) and
/// an ASEG child (class 109) whose channels drive that ALO (the flap loop): the dizzy birds.
pub fn stun_fx(w: &World, tpl: usize) -> Vec<usize> {
    let kids = |o: usize| w.objects.iter().enumerate().filter(move |(_, x)| x.owner == Some(o)).map(|(k, _)| k);
    let fx = |o: usize| {
        let oid = w.objects[o].oid;
        kids(o).any(|k| w.objects[k].class == CLASS_SOUND)
            && kids(o).any(|k| w.objects[k].class == CLASS_ASEG && aseg_of(w, k).is_some_and(|a| !a.channels.is_empty() && a.channels.iter().all(|c| c.oid as i64 == oid as i64)))
    };
    let mut out = vec![];
    for c in kids(tpl) {
        let sub = subtree(w, c);
        if sub.iter().any(|&o| fx(o)) {
            out.extend(sub);
        }
    }
    out
}

/// The pickup's SM (+0x164): the class-111 object of its template's subtree.
pub fn pickup_sm(w: &World, pi: usize) -> Option<usize> {
    subtree(w, template_of(w, pi)).into_iter().find(|&o| w.objects[o].class == CLASS_SM)
}

/// FUN_001efac0 tables of an SM object.
pub fn sm_tables(w: &World, sm: usize) -> Option<Sm> {
    use crate::engine::world::loads_b4::B4Load;
    let Payload::Loaded(LoadData::B4(b)) = &w.objects[sm].payload else { return None };
    let B4Load::L1efac0 { nested, pairs, links, link_f, .. } = &**b else { return None };
    let nested: Vec<(usize, i32)> = nested.iter().map(|&o| (o, w.objects[o].oid)).collect();
    Some(Sm::ee_sub_1efac0(pairs, links, link_f, &nested))
}

/// Animated rig of one NPC (bones of its template subtree, rest level space of the template).
pub struct NpcAnimator {
    pub rig: JtRig,
    /// playing ASEG object and its ASEGA
    pub cur: Option<(usize, AsegaPlayer)>,
    /// what was requested last (restart detection)
    pub anim: NpcAnim,
    /// SMA and its SM object
    pub sm: Option<(usize, Sma)>,
    sms: HashMap<usize, Arc<Sm>>,
    pub rng: GameRng,
    /// cross-fade source locals and time left (FUN_0012f9b8 blend-in, as playerdraw)
    from: Vec<Mat4>,
    fade: f32,
    /// rest locals and the rig bones a root-motion channel drives
    rest: Vec<Mat4>,
    root_bones: HashMap<usize, Vec<usize>>,
}

impl NpcAnimator {
    pub fn new(w: &World, obj: usize) -> NpcAnimator {
        let rig = JtRig::build(w, template_of(w, obj));
        let rest: Vec<Mat4> = rig.pose.bones.iter().map(|b| b.local).collect();
        NpcAnimator { rig, cur: None, anim: NpcAnim::None, sm: None, sms: HashMap::new(), rng: GameRng::default(), from: rest.clone(), fade: 0.0, rest, root_bones: HashMap::new() }
    }

    fn bind(&mut self, w: &World, a: usize) -> Option<Arc<Aseg>> {
        let aseg = aseg_of(w, a)?;
        if !self.rig.bindings.contains_key(&a) {
            let b = bind_channels(w, self.rig.jt, aseg, &self.rig.index);
            let roots = aseg.channels.iter().zip(&b).filter(|(c, _)| c.flag && c.kind == 1).filter_map(|(_, b)| *b).collect();
            self.root_bones.insert(a, roots);
            self.rig.bindings.insert(a, b);
        }
        Some(Arc::new(aseg.clone()))
    }

    fn start(&mut self, w: &World, a: usize, t: f32, svt: f32, flags: u32) {
        let Some(aseg) = self.bind(w, a) else { return };
        let mut p = AsegaPlayer::new(aseg, flags, svt);
        p.state.t = t;
        self.from = self.rig.pose.bones.iter().map(|b| b.local).collect();
        self.fade = BLEND_TIME;
        self.cur = Some((a, p));
    }

    fn sm_of(&mut self, w: &World, sm: usize) -> Option<Arc<Sm>> {
        if let Some(m) = self.sms.get(&sm) {
            return Some(m.clone());
        }
        let m = Arc::new(sm_tables(w, sm)?);
        self.sms.insert(sm, m.clone());
        Some(m)
    }

    /// FUN_0012f9b8 for the SMA's transition ASEG.
    fn sm_play(&mut self, w: &World) {
        let Some(p) = self.sm.as_ref().and_then(|s| s.1.asega.clone()) else { return };
        self.start(w, p.aseg, p.start, p.svt, p.flags);
    }

    /// One frame: (re)start what `anim` asks for, advance by `dt`, pose. `owner` receives the SMA's
    /// FUN_001f13a8 messages. Returns the delivered signals.
    pub fn step(&mut self, w: &World, owner: usize, anim: NpcAnim, dt: f32, recv: Option<&mut dyn SmaReceiver>) -> Vec<SmaMsg> {
        let len = |o: usize| aseg_of(w, o).map_or(0.0, |a| a.length);
        let mut due = false;
        match anim {
            NpcAnim::Sm { sm, label } => {
                let cur_sm = self.sm.as_ref().map(|s| s.0);
                if cur_sm != Some(sm) {
                    if let Some(m) = self.sm_of(w, sm) {
                        // FUN_001f0380(sm, owner, label, 0)
                        let sma = Sma::ee_sub_1f01b8(&m, &mut self.rng, label, 0, NPC_SMA_MASK, &len);
                        self.sm = Some((sm, sma));
                        self.cur = None;
                        self.sm_play(w);
                    }
                } else if !matches!(self.anim, NpcAnim::Sm { label: l, .. } if l == label) {
                    // FUN_001f0660(sma, label)
                    if let (Some(m), Some((_, sma))) = (self.sm_of(w, sm), self.sm.as_mut()) {
                        let had = sma.asega.clone();
                        sma.ee_sub_1f0660(&m, &mut self.rng, label, &len);
                        if sma.asega != had {
                            self.sm_play(w);
                        }
                    }
                }
            }
            NpcAnim::Loop { id, .. } | NpcAnim::Once { id, .. } => {
                self.sm = None;
                let a = anim_object(w, owner, id).filter(|&k| aseg_of(w, k).is_some());
                match a {
                    Some(a) if self.cur.as_ref().map(|c| c.0) != Some(a) => {
                        let t = if let NpcAnim::Once { t, .. } = anim { t.max(0.0) } else { 0.0 };
                        self.start(w, a, t, 1.0, 0);
                    }
                    Some(_) => {}
                    None => self.cur = None,
                }
            }
            NpcAnim::Aseg { obj, t } => {
                self.sm = None;
                if self.cur.as_ref().map(|c| c.0) != Some(obj) {
                    self.start(w, obj, t.max(0.0), 1.0, 0);
                }
            }
            NpcAnim::None => {
                self.sm = None;
                self.cur = None;
            }
        }
        self.anim = anim;
        for b in &mut self.rig.pose.bones {
            b.motion = 0;
        }
        if let Some((a, p)) = &mut self.cur {
            match anim {
                NpcAnim::Loop { rate, .. } => {
                    let l = p.aseg.length;
                    p.state.t += dt * rate;
                    if l > 0.0 {
                        p.state.t = p.state.t.rem_euclid(l);
                    }
                }
                NpcAnim::Once { t, .. } | NpcAnim::Aseg { t, .. } => p.state.t = t.clamp(0.0, p.aseg.length),
                _ => {
                    drive(p, AnimClock { dt, real_dt: dt });
                    due = p.ended;
                }
            }
            let t = p.state.t;
            let segments = bind_segments(&p.aseg, t);
            if let Some(binding) = self.rig.bindings.get(a) {
                self.rig.pose.bind(&segments, binding, t, 1.0);
            }
            // root motion: the body carries the horizontal part
            for &b in self.root_bones.get(a).map(|v| v.as_slice()).unwrap_or(&[]) {
                let (l, r) = (&mut self.rig.pose.bones[b].local, &self.rest[b]);
                l[3][0] = r[3][0];
                l[3][1] = r[3][1];
            }
        }
        let mut out = vec![];
        if let Some((smo, mut sma)) = self.sm.take() {
            if let Some(m) = self.sm_of(w, smo) {
                if due {
                    // FUN_001f1078: the transition ASEGA ended
                    sma.ee_sub_1f1078(&m, &mut self.rng, &len);
                }
                out = sma.dispatch(&m, recv, &mut []);
            }
            self.sm = Some((smo, sma));
            if due {
                self.sm_play(w);
            }
        }
        self.rig.pose.compose(&IDENTITY);
        if self.fade > 0.0 {
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
        out
    }

    /// WP-D6: class-run ASEGA acts (anim::bind, FUN_0012f9b8 -> FUN_00230018) on top of the NPC's own anim:
    /// every rig bone whose top act is a class anim takes that act's local (`locals`, AnimBinder::locals);
    /// keys of the NPC object itself map to its template's bones (a clone shares the template skeleton).
    /// Returns the rig bones overridden. REPLACED: the NPC's own anim is not an act in the bone lists, so
    /// a class act always wins over it (the original orders both by ASEG +0x6c priority).
    pub fn overlay(&mut self, w: &World, obj: usize, locals: &HashMap<BoneKey, Affine>) -> usize {
        let tpl = template_of(w, obj);
        let mut n = 0;
        for (&(o, k), l) in locals {
            let key = if o == obj { (tpl, k) } else { (o, k) };
            let Some(&b) = self.rig.index.get(&key) else { continue };
            self.rig.pose.bones[b].local = [0, 1, 2, 3].map(|r| [l[r][0], l[r][1], l[r][2], (r == 3) as u32 as f32]);
            n += 1;
        }
        if n > 0 {
            let locals: Vec<Mat4> = self.rig.pose.bones.iter().map(|b| b.local).collect();
            let parents: Vec<Option<usize>> = self.rig.pose.bones.iter().map(|b| b.parent).collect();
            let worlds = crate::engine::anim::pose::world_matrices(&IDENTITY, &locals, &parents);
            for (b, wm) in self.rig.pose.bones.iter_mut().zip(worlds) {
                b.world = wm;
            }
        }
        n
    }

    /// Playback time of the current ASEG.
    pub fn time(&self) -> f32 {
        self.cur.as_ref().map_or(0.0, |c| c.1.state.t)
    }
    /// Current ASEG object.
    pub fn aseg(&self) -> Option<usize> {
        self.cur.as_ref().map(|c| c.0)
    }
}

/// Animators of every guard (K6/K7 class 56 family) and pickup (K8) of a class set, by object.
#[derive(Default)]
pub struct NpcSet {
    pub npcs: HashMap<usize, NpcAnimator>,
    /// walk-anim time per guard (root velocity sample for the rate)
    t_walk: HashMap<usize, f32>,
    /// FUN_001f13a8 signals delivered last step (object, message)
    pub signals: Vec<(usize, SmaMsg)>,
}

impl NpcSet {
    /// One frame after `ClassSet::frame`: pick each NPC's anim from its class state, advance and pose it;
    /// pickup SMA signals go to the pickup's slot-4 handler FUN_001c5098.
    pub fn step(&mut self, w: &World, set: &mut ClassSet, h: &mut dyn ClassHost, dt: f32) {
        self.signals.clear();
        let now = set.g.t;
        for (gi, x) in &set.k6 {
            let Inst::Guard(g) = x else { continue };
            let a = self.npcs.entry(*gi).or_insert_with(|| NpcAnimator::new(w, *gi));
            let tw = self.t_walk.entry(*gi).or_insert(0.0);
            let anim = guard_anim(w, *gi, g, now, *tw);
            a.step(w, *gi, anim, dt, None);
            *tw = if a.aseg().is_some() && matches!(anim, NpcAnim::Loop { id: GUARD_ANIM_WALK, .. }) { a.time() } else { 0.0 };
        }
        let ClassSet { k8, g, rng, .. } = set;
        for (pi, x) in k8.iter_mut() {
            let Inst::K8(p) = x else { continue };
            let Some(sm) = pickup_sm(w, *pi) else { continue };
            let anim = NpcAnim::Sm { sm, label: p.f_168.unwrap_or(-1) };
            let a = self.npcs.entry(*pi).or_insert_with(|| NpcAnimator::new(w, *pi));
            let pi = *pi;
            let mut recv = |msg: i32, m: &SmaMsg| p.ee_sub_1c5098(pi, msg, m.new, &mut Cx { g: &mut *g, rng: &mut *rng }, &mut *h);
            for m in a.step(w, pi, anim, dt, Some(&mut recv)) {
                self.signals.push((pi, m));
            }
        }
    }
}

impl NpcSet {
    /// WP-D6: pose every NPC's bones driven by class-run ASEGAs (`binder`, synced from the K25 ASEGAs by the
    /// caller; `time(handle)` = ASEGA (t, svt)) over its own anim. Returns the NPCs overridden.
    pub fn overlay_class_anims(&mut self, w: &World, binder: &AnimBinder, time: &dyn Fn(u64) -> Option<(f32, f32)>) -> Vec<usize> {
        let locals = binder.locals(w, time);
        if locals.is_empty() {
            return vec![];
        }
        let mut keys: Vec<usize> = self.npcs.keys().copied().collect();
        keys.sort_unstable();
        keys.into_iter().filter(|o| self.npcs.get_mut(o).is_some_and(|a| a.overlay(w, *o, &locals) > 0)).collect()
    }
}

/// A8: SLY_NPC_PATROL is a debug-build-only capture aid. Since J10 the level scripts start the guard
/// routines (task-enter handlers -> FUN_00171cf8), so release builds ignore the variable.
pub fn patrol_override() -> bool {
    cfg!(debug_assertions) && std::env::var("SLY_NPC_PATROL").is_ok()
}

/// Debug stand-in for the level script's FUN_00171cf8 calls (method field 317; J10 runs the real ones): for every guard able to walk (+0xfe8 & 2) the routine with a rail command nearest to it
/// (within `max_off`), as tests/engine_k6.rs picks it. Returns (guard object, routine).
pub fn patrol_routines(set: &ClassSet, max_off: f32) -> Vec<(usize, usize)> {
    use crate::engine::classes::guard::patrol::Cmd;
    use crate::engine::player::p5_attach::FUN_001e6f60;
    let mut out = vec![];
    for (gi, x) in &set.k6 {
        let Inst::Guard(g) = x else { continue };
        if g.body.f_fe8 & 2 == 0 {
            continue;
        }
        let mut best: Option<(f32, usize)> = None;
        for (ri, c) in set.k6l.cmds.iter().enumerate() {
            for cmd in c {
                if let Cmd::Rail(Some(p), ..) = cmd {
                    let hit = FUN_001e6f60(p, g.body.pos);
                    let e = (0..3).map(|k| (hit.cp[k] - g.body.pos[k]).powi(2)).sum::<f32>().sqrt();
                    if best.is_none_or(|b| e < b.0) {
                        best = Some((e, ri));
                    }
                }
            }
        }
        if let Some((e, ri)) = best.filter(|b| b.0 <= max_off) {
            let _ = e;
            out.push((*gi, ri));
        }
    }
    out
}
