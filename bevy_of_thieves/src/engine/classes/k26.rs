//! WP-K26 'world alive': level sounds (class 43 C2cabd8 emitters reach the game audio), anim event dispatch /
//! SMA state-change messages for every class-run ASEGA / SMA, and world bodies (class-owned POs / props move).
use super::k25::SoundRt;
use super::plugin::EngineHost;
use super::{ClassHost, ClassSet, Inst};
use crate::engine::anim::events::EffectCall;
use crate::engine::anim::sm::SmaMsg;
use crate::engine::audio::emitter::EmitterId;
use crate::engine::game::splice_host::RumbleRequest;
use std::collections::HashSet;

impl ClassSet {
    /// K26: the AddLo hook (slot 22, FUN_00210140 world add) of every class-43 emitter list in the world at load:
    /// the world adds the loaded tree after the post-load pass (slot 14 allocates +0xec first).
    pub(super) fn post_load_k26(&mut self, h: &mut dyn ClassHost) {
        for (i, x) in &mut self.insts {
            if let Inst::C2cabd8(c) = x {
                if c.added {
                    c.ee_sub_12ed20(*i, h);
                }
            }
        }
    }

    /// K26: FUN_001f13a8 / FUN_001f0fd8 delivery: msg 10 {old, new} to the SMA owner's slot 4 (vtable +0x2c).
    /// Ported slot-4 receivers: the K8 pickups (FUN_001c5098). K27: owners whose slot 4 is the base handler
    /// FUN_00133590 (an empty function, `jr ra`) consume it (1:1, counted as handled); the other overrides
    /// (k27::slot4_is_base false) stay UNPORTED for message 10.
    pub(super) fn frame_k26(&mut self, h: &mut dyn ClassHost) {
        let msgs = h.k26_take_msgs();
        if msgs.is_empty() {
            return;
        }
        let ClassSet { k8, g, rng, .. } = self;
        let mut handled = 0;
        for (owner, msg, m) in msgs {
            let Some(oi) = (owner as usize).checked_sub(1) else { continue };
            if let Some((_, Inst::K8(p))) = k8.iter_mut().find(|x| x.0 == oi) {
                p.ee_sub_1c5098(oi, msg, m.new, &mut super::k8::Cx { g: &mut *g, rng: &mut *rng }, h);
                handled += 1;
            } else if h.k27_slot4_base(oi) {
                handled += 1;
            }
        }
        h.k26_handled(handled);
    }
}

/// K26 run-time statistics / queues of the host (EngineHost::k26).
#[derive(Clone, Debug, Default)]
pub struct K26Rt {
    /// FUN_00131290 effect calls made for class-run ASEGAs
    pub dispatched: usize,
    /// effect sounds (FUN_00210740) and ASEGs started by case 4
    pub sounds: usize,
    pub spawned: usize,
    /// calls whose engine side is not ported (anim::events kinds 1, 5, 7-0x14 except 0xb / 0xc; case 2 shake)
    pub pending: usize,
    /// FUN_001f13a8 messages delivered to owners / handled by a ported slot 4; ASEG event owner messages
    /// (kind 0xb msg 9, kind 0xc vtable +0x1c) reaching the owner
    pub delivered: usize,
    pub handled: usize,
    pub owner_msgs: usize,
    /// objects posed by class ASEGA channels (sum over frames)
    pub posed: usize,
    /// case 3 FUN_00187e28 rumble requests (drained by the game layer)
    pub rumble: Vec<RumbleRequest>,
}

impl EngineHost {
    /// K26: FUN_00131290 for the events the class-run ASEGAs reached this frame (`AnimRt::fired`), into the
    /// J5 receiver `game::anim_play::LevelEffectSink` on the class emitter pool: case 0 FUN_00210740 one-shot
    /// emitters (started by `SoundOut::due`), 3 rumble, 4 FUN_0015e938 -> class 0x6d FUN_0012f9b8 at svt 1.
    /// The camera shake (case 2) has no g_cm in the class host (pending).
    pub(super) fn k26_dispatch(&mut self, t: f32, dt: f32) {
        let Some(w) = self.world.clone() else { return };
        let fired = std::mem::take(&mut self.k25.fired);
        if fired.is_empty() {
            self.k25.gone.clear();
            return;
        }
        let clk = crate::engine::camera::cm::CmClock { t, dt, dt_real: dt };
        self.k30_t = t;
        let mut rumble = std::mem::take(&mut self.k26.rumble);
        let mut sink = crate::engine::game::anim_play::LevelEffectSink { world: &w, emitters: &mut self.k25_snd.es, rumble: &mut rumble, cm: None, clk, current: 0, sounds: vec![], start: vec![], pending: vec![] };
        let mut n = 0;
        let mut seeks = vec![];
        let mut k28 = vec![];
        let mut msg9 = vec![];
        for (ai, ev, rev, h) in fired {
            if std::env::var_os("SLY2_K22DBG").is_some() && (ai == 147 || ai == 148) {
                eprintln!("[R50] k26 fired aseg {ai} ev {ev} rev {rev} h {h:#x} live {} gone {:?}", self.k25.asegas.contains_key(&h), self.k25.gone.get(&h));
            }
            let Some(a) = crate::engine::playerdraw::aseg_of(&w, ai) else { continue };
            sink.current = ai;
            n += crate::engine::player::p23::dispatch_events(a, &[(ev, rev)], &mut sink);
            // K27: case 5 FUN_00130d30(+8, +0xc, asega, 0) on this event's ASEGA
            sink.pending.retain(|c| match *c {
                EffectCall::Seek { t, f } => {
                    seeks.push((h, t, f));
                    false
                }
                // WIRE-BM2: case 0xb vt+0x24(asega, 9, {asega, t, v}) -> the target ALO's slot 4
                EffectCall::Msg9 { t, v } => {
                    msg9.push((h, t, v));
                    false
                }
                // HEIST: case 0xc (asega + vt +0x18)->vt+0x1c(4, 0, 1, &payload, 0): ASEGA script message 4
                EffectCall::Owner1c(v) => {
                    // HEIST r16: a retract (+0xa8) event's ASEGA may already be out of the live map (`gone`)
                    if let Some(owner) = self.k25.asegas.get(&h).map(|x| x.obj).or_else(|| self.k25.gone.get(&h).map(|g| g.0)) {
                        if self.heist_aseg_msg4.len() < 256 {
                            self.heist_aseg_msg4.push((ai, h, owner, v));
                        }
                    }
                    false
                }
                // K28: kinds 7..0x10 on this event's ASEGA (classes/k28.rs)
                // K30: kinds 0x11..0x13 (classes/k28.rs)
                EffectCall::E7 { .. } | EffectCall::E9 { .. } | EffectCall::Ed { .. } | EffectCall::Ef { .. } | EffectCall::E11(..) | EffectCall::E12 { .. } | EffectCall::E13 => {
                    k28.push((h, c.clone()));
                    false
                }
                _ => true,
            });
        }
        let crate::engine::game::anim_play::LevelEffectSink { sounds, start, pending, .. } = sink;
        self.k26.rumble = rumble;
        self.k26.dispatched += n;
        for s in &sounds {
            self.k25_snd.plays.push((s.aseg, s.sound));
            self.k26_out.oneshot.push(s.emitter);
        }
        self.k26.sounds += sounds.len();
        for (h, t, f) in seeks {
            self.k25.ee_sub_130d30(&w, t, f, h, 0);
            self.k27_seeks += 1;
        }
        self.k26.owner_msgs += msg9.len();
        for (h, t, v) in msg9 {
            if let Some(a) = self.k25.asegas.get(&h) {
                self.k26_msg9.push((a.obj, [h, t.to_bits(), v as u32]));
                self.k26_msg9_live.push((a.obj, [h, t.to_bits(), v as u32]));
            }
        }
        for (h, c) in k28 {
            self.k28_event(h, c);
        }
        for i in start {
            if self.k25.ee_sub_12f9b8(&w, 0.0, 1.0, i as u32 + 1, 0, 0) != 0 {
                self.k26.spawned += 1;
            }
        }
        for c in pending {
            match c {
                EffectCall::Msg9 { .. } | EffectCall::Owner1c(_) => self.k26.owner_msgs += 1,
                // K27: case 2 FUN_0014f750(g_cm, s16) -> applied to g_cm by tick_classes
                EffectCall::E2(k) => self.k27_shake = Some(k as i32),
                _ => self.k26.pending += 1,
            }
        }
    }
    /// K26 / K29: ASEG bindings of the live class-run ASEGAs through anim::bind (D6): FUN_0012f9b8 acts per
    /// channel (FUN_0012fe58 channel -> (object, bone REFERENCE), priority lists FUN_00230018 / 00230100,
    /// FUN_00130b18 retract when the ASEGA dies), each top act's curve at the ASEGA time (+0x1c, rate +0x40)
    /// set on its bone's local (bone kind), bone world = local * parent (FUN_0022faa0), object world = bone 0.
    /// The object worlds feed the host world matrices (FUN_001a62e8 / vtable +0xfc) the class code reads.
    pub(super) fn k26_bind(&mut self) {
        let Some(w) = self.world.clone() else { return };
        self.k26_posed.clear();
        let live: Vec<(u64, usize, Option<usize>)> = self.k25.asegas.iter().filter(|x| x.1.st.alive).map(|(k, x)| (*k as u64, x.aseg, (x.obj as usize).checked_sub(1))).collect();
        let times: std::collections::HashMap<u64, (f32, f32)> = self.k25.asegas.iter().map(|(k, x)| (*k as u64, (x.st.t, x.st.svt))).collect();
        let pose = {
            let mut b = self.k29_binder.lock().unwrap();
            b.sync(&w, &live);
            b.pose(&w, &|k| times.get(&k).copied())
        };
        self.k26.posed += pose.objects.len();
        let (mut objects, mut bones) = (pose.objects, pose.bones);
        // CRAWL R123: FUN_00130b18 (ASEGA retract) only unlinks the ASEGA: the bone locals / object xf keep what its
        // last FUN_0012f9b8 evaluation wrote (no rest-pose reset). The port re-derived the pose from the live ASEGAs
        // only, so a finished transition snapped back to the rest pose: row 3 slab 136544 (SM 17434 -> state 7 via
        // ASEG 137306 on bone 2) popped back up to z 320 instead of staying lowered. Held per world; a new ASEGA
        // on the same bone overrides. Opt-out SLY2_NO_R123_POSEHOLD=1.
        if std::env::var_os("SLY2_NO_R123_POSEHOLD").is_none() {
            let id = std::sync::Arc::as_ptr(&w) as usize;
            if self.k29_held.0 != id {
                self.k29_held = (id, Default::default(), Default::default());
            }
            let posed_objs: std::collections::HashSet<usize> = bones.keys().map(|k| k.0).chain(objects.keys().copied()).collect();
            for (k, v) in &self.k29_held.1 {
                if !posed_objs.contains(&k.0) {
                    bones.entry(*k).or_insert(*v);
                }
            }
            for (k, v) in &self.k29_held.2 {
                if !posed_objs.contains(k) {
                    objects.entry(*k).or_insert(*v);
                }
            }
            self.k29_held.1 = bones.clone();
            self.k29_held.2 = objects.clone();
        }
        self.k26_posed = objects;
        self.k29_bones = bones;
    }
    /// K26: the SMA messages of this frame (ClassSet::frame_k26 delivers them)
    pub(super) fn k26_msgs(&mut self) -> Vec<(u32, i32, SmaMsg)> {
        let m = std::mem::take(&mut self.k25.msgs);
        // POOL1: script message 8 {old, new} of each SMA state change to its SM object's frame
        let selfs = std::mem::take(&mut self.k25.script8_sma);
        for (k, (sm, old, new)) in std::mem::take(&mut self.k25.script8).into_iter().enumerate() {
            if self.k27_script_msgs.len() < 256 {
                self.k27_script_msgs.push((sm as u32 + 1, 8, vec![old as u32, new as u32]));
                // POOL1 r36: FUN_001331f8(sma, 8, ..): self is the SMA (handle), not the SM object
                if let Some(&x) = selfs.get(k) {
                    self.sma_script_self_q.insert((sm as u32 + 1, old as u32, new as u32), x);
                }
            }
        }
        // POOL1 r30: ASEGA script messages 2 / 3 (FUN_0012f9b8 / FUN_00130b18), capped like k27_script_msgs
        let a = std::mem::take(&mut self.k25.script_aseg);
        let room = 256usize.saturating_sub(self.aseg_script_msgs.len());
        self.aseg_script_msgs.extend(a.into_iter().take(room));
        self.k26.delivered += m.len();
        m
    }
}

/// K26: class-pool emitters -> host audio. REPLACED: the 989snd voice allocation (SPU) decides audibility in the
/// original; here an emitter starts once when it is in the active list (no stop request, flags & 8 clear) and its
/// source is within `far` of the listener, at volume +0x10 * (1 inside `near`, linear to 0 at `far`). Unpositioned
/// emitters at the origin are 2D (full volume). A pool slot that leaves the active list may start again.
#[derive(Clone, Debug, Default)]
pub struct SoundOut {
    pub started: HashSet<EmitterId>,
    /// FUN_00210740 one-shot emitters (anim events): started once, then freed from the pool
    pub oneshot: Vec<EmitterId>,
    /// emitters started so far (statistics)
    pub played: usize,
}

impl SoundOut {
    /// `pos_of(attach)` = world position of the +0x1c attach object (handle obj + 1).
    pub fn due(&mut self, snd: &mut SoundRt, listener: [f32; 3], pos_of: &mut dyn FnMut(i32) -> [f32; 3]) -> Vec<(i32, f32)> {
        self.started.retain(|id| snd.es.active.contains(id));
        let oneshot = std::mem::take(&mut self.oneshot);
        let mut out = Vec::new();
        for (k, &id) in snd.es.active.iter().chain(oneshot.iter()).enumerate() {
            let one = k >= snd.es.active.len();
            if !one && self.started.contains(&id) {
                continue;
            }
            let Some(e) = snd.es.get(id) else { continue };
            if !one && e.flags & 8 != 0 {
                continue;
            }
            let p = if e.unpositioned != 0 && e.at_origin() { Some(pos_of(e.unpositioned)) } else if e.at_origin() { None } else { Some(e.pos) };
            let k = match p {
                None => 1.0,
                Some(p) => {
                    let d = (0..3).map(|a| (p[a] - listener[a]) * (p[a] - listener[a])).sum::<f32>().sqrt();
                    if d >= e.far {
                        continue;
                    }
                    if d <= e.near { 1.0 } else { 1.0 - (d - e.near) / (e.far - e.near).max(1.0) }
                }
            };
            if !one {
                self.started.insert(id);
            }
            self.played += 1;
            out.push((e.sound, e.f10 * k));
        }
        for id in oneshot {
            if let Some(s) = snd.es.slots.get_mut(id) {
                *s = None;
            }
        }
        out
    }
}

/// One positional source of a music layer: the 0x10-byte record at instance +0xb8 + layer * 0x80 + k * 0x10.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MusicSrc {
    /// +0 object handle (obj + 1), +4 / +8 the class-43 +0xf4 entry words 1 / 2 (near / far), +0xc word 3
    pub obj: u32,
    pub near: f32,
    pub far: f32,
    pub w3: i32,
}

/// K26: the music manager 0x3dd040 parts the class-43 slot 14 reaches: +0x1d0 / +0x1d4 the level's music layer
/// keys (FUN_001fd0e8 `Block3dd040::small`) and the current instance DAT_003dd268 (+0x228, set by FUN_001fd0e8
/// when the level has a music stream) with its +0xa0 per-layer counts / +0xb8 per-layer source lists and the
/// +0x50 flag word (FUN_001fb658). REPLACED: the instance is this struct (`inst` = 1 when it exists); the
/// FUN_001fb6a8 / FUN_001fb710 activation behind FUN_001fb658 is the flag word only. UNPORTED: the layer volume
/// update from these sources (FUN_001fd830 family) - the game mixes the KPV layers once at gain 1
/// (audio::music::mix_stereo).
#[derive(Clone, Debug, Default)]
pub struct MusicRt {
    pub keys: Vec<i32>,
    pub inst: i32,
    pub layers: [Vec<MusicSrc>; 6],
    pub f_50: u16,
    /// +0xec definition list ids (+0x40 of each record, file order; FUN_001fd0e8 links them in reverse)
    pub recs: Vec<i32>,
    /// +0x200 playing music instance (0 = none; REPLACED handle `MUSIC_TAG | record index + 1`)
    pub cur: u32,
    /// FUN_001fe5f0 plays / FUN_001fe738 stops (statistics)
    pub plays: usize,
    pub stops: usize,
}

/// REPLACED handle tag of a FUN_001fc138 music instance
pub const MUSIC_TAG: u32 = 0x4600_0000;

impl MusicRt {
    /// FUN_001fd0e8 (the parts kept here): layer keys, instance present when the level has a music stream
    pub fn from_block(b: &crate::engine::disc::level_file::Block3dd040) -> Self {
        MusicRt {
            keys: b.small.iter().map(|&k| k as i32).collect(),
            inst: b.desc2.is_some() as i32,
            recs: b.recs.iter().map(|r| r.h0 as i32).collect(),
            ..Default::default()
        }
    }
    /// FUN_001fcfa0(0x3dd040, id): the +0xec record with +0x40 == id (head first = the last loaded)
    pub fn ee_sub_1fcfa0(&self, id: i32) -> Option<usize> {
        self.recs.iter().rposition(|&r| r == id)
    }
    /// FUN_001fe738(0x3dd040): stop the playing instance (+0x200 = 0, +0x244..+0x24c = 0). UNPORTED: the
    /// remaining +0x58 cue records FUN_001fe0e8, the +0x240 SMA retarget FUN_001f0660(.., 0x611), FUN_001fb098
    /// and the +0x22c layer objects' vtable +0x14 release (no layer objects: see ee_sub_1fe5f0).
    pub fn ee_sub_1fe738(&mut self) {
        if self.cur != 0 {
            self.cur = 0;
            self.stops += 1;
        }
    }
    /// FUN_001fe5f0(0x3dd040, id, k): FUN_001fe738, then the record `id` (none -> 0) becomes the playing
    /// instance (FUN_001fc138 + FUN_00259b48 +0x200) and its handle is returned. UNPORTED: k == 0's
    /// FUN_00227728(0x4435f0, id, +0x48) and the +0x48 / +0x54 layer objects (FUN_0015ed78 / FUN_001f67e0 into
    /// +0x22c..+0x240); the stream itself is the game's (audio::music, one premixed loop).
    pub fn ee_sub_1fe5f0(&mut self, id: i32, _k: i32) -> u32 {
        self.ee_sub_1fe738();
        let Some(r) = self.ee_sub_1fcfa0(id) else { return 0 };
        self.cur = MUSIC_TAG | (r as u32 + 1);
        self.plays += 1;
        self.cur
    }
    /// FUN_001fe990(0x3dd040, key): index of `key` in the +0x1d4 list, -1 if absent
    pub fn ee_sub_1fe990(&self, key: i32) -> i32 {
        self.keys.iter().position(|&k| k == key).map_or(-1, |i| i as i32)
    }
    /// FUN_001fb658(inst, mask, val): +0x50 = +0x50 & !mask | val & mask
    pub fn ee_sub_1fb658(&mut self, mask: u16, val: u16) {
        self.f_50 = self.f_50 & !mask | val & mask;
    }
    /// FUN_001fc408(w3, inst, layer, obj, &{near, far}): append to the layer's list (unsigned layer < 6, < 8
    /// sources); equal near / far -> far + 1
    pub fn ee_sub_1fc408(&mut self, w3: i32, layer: i32, obj: u32, nf: [i32; 2]) {
        let Some(l) = usize::try_from(layer).ok().filter(|&l| l < 6) else { return };
        if self.layers[l].len() >= 8 {
            return;
        }
        self.ee_sub_1fb658(0x200, 0x200);
        let near = f32::from_bits(nf[0] as u32);
        let mut far = f32::from_bits(nf[1] as u32);
        if near == far {
            far += 1.0;
        }
        self.layers[l].push(MusicSrc { obj, near, far, w3 });
    }
}
