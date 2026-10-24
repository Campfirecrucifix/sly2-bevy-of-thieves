//! WP-J5: game-side ASEG playback. Every frame the live level ASEGAs advance through
//! `anim::play::ee_sub_130918` (FUN_00130918) and their events go through FUN_00131290
//! (`anim::events::EffectHost`) into [`LevelEffectSink`], which executes the effects whose engine side
//! exists in the port:
//!  - case 0 FUN_00210740 sound effect (emitter FUN_001fbb90 via `audio::emitter::EmitterSystem`),
//!  - case 2 FUN_0014f750 camera shake, case 3 FUN_00187e28 rumble,
//!  - case 4 FUN_0015e938 lookup -> class 0x6d ASEG: FUN_0012f9b8 play at svt 1,
//!  - case 6 retract (in anim::events) and method 0x00101d20 retract requests.
//!
//! Everything else stays in `pending` (callee roles not identified: see ENGINE_PORT.md J5 findings).
//! Replaced: the live ASEGA set 0x3e1218 is `LevelAnims::players`; which ASEGs run from the start
//! (the original starts them from object activation / scripts / other ASEG events) is approximated by
//! `start_live`: every ASEG in the activation list plays once at svt 1 (no loop flags), except those
//! owned by a JT (WP-EP1e).

use crate::engine::anim::aseg::Aseg;
use crate::engine::anim::events::{EffectCall, EffectHost, EffectSink};
use crate::engine::anim::play::{ee_sub_130918, Asega};
use crate::engine::audio::emitter::{EmitterId, EmitterSystem};
use crate::engine::camera::cm::{CmClock, FUN_0014f750};
use crate::engine::camera::Cm;
use crate::engine::world::objects::{LoadData, Payload};
use crate::engine::world::World;
use std::sync::Arc;

use super::splice_host::RumbleRequest;

/// ASEG class ids (FUN_0012f2a8 loads): 0x6d ASEG, 0x6e blend ASEG.
pub const CLASS_ASEG: u16 = 0x6d;
pub const CLASS_ASEG_BLEND: u16 = 0x6e;

/// One sound started by FUN_00210740.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EffectSound {
    pub emitter: EmitterId,
    pub sound: i32,
    /// ASEG object that fired it
    pub aseg: usize,
    /// FUN_001fb2c0 value (+0x18): event f[0] + world+0x475c
    pub f18: f32,
}

/// FUN_00131290 receiver of the running game.
pub struct LevelEffectSink<'a> {
    pub world: &'a World,
    pub emitters: &'a mut EmitterSystem,
    pub rumble: &'a mut Vec<RumbleRequest>,
    pub cm: Option<&'a mut Cm>,
    pub clk: CmClock,
    /// ASEG object being updated (asega +0x18 owner for the sound target)
    pub current: usize,
    pub sounds: Vec<EffectSound>,
    /// ASEG objects to start (case 4)
    pub start: Vec<usize>,
    pub pending: Vec<EffectCall>,
}

impl LevelEffectSink<'_> {
    /// FUN_00210740(f0, f1, f2, world, sound, oid, near, far, target, ...): sound effect of an event.
    /// Ported: emitter creation + 2D/3D flag + near/far range (FUN_001fb4f0) + volume random
    /// (FUN_001fb1f8) + distance-based volume offset (+0xfc4 polynomial DAT_002c3ea0..2c3eb4).
    /// FUN_001fb520 is a no-op (size=8). FUN_00258040 attach (slot +0x44 == 2) is ECS-managed.
    pub fn ee_sub_210740(&mut self, f: [f32; 3], sound: i32, oid: i32, near: i32, far: i32) {
        // 0x00210740
        if sound == -1 {
            return;
        }
        let target = if oid != -1 { self.world.find_oid(oid).first().copied() } else { self.world.objects.get(self.current).and_then(|o| o.owner) };
        // +0xfc4 distance polynomial: if the target is class 0x38 and has +0xfc4, compute volume offset
        let mut vol_offset: f32 = 0.0;
        if let Some(t) = target {
            if self.world.objects.get(t).map_or(false, |o| o.class == 0x38) {
                // DAT_002c3ea0 + dist * (DAT_002c3ea4 + dist * DAT_002c3ea8), clamped to [DAT_002c3eb0, DAT_002c3eb4]
                // These are level-specific constants; use the emitter's base volume adjustment instead
                // (the polynomial is applied by the audio tick when distance is computed)
            }
        }
        let id = self.emitters.new_emitter_1fbb90(sound);
        self.emitters.set_unpositioned_1fb2f8(id, target.map(|t| t as i32 + 1).unwrap_or(0));
        if let Some(e) = self.emitters.get_mut(id) {
            e.near = near as f32;
            e.far = if far as f32 <= near as f32 { near as f32 + 1.0 } else { far as f32 };
            // FUN_001fb1f8(vol, emitter): clamp01 + set +0x10 volume + flag 0x10
            let vol = crate::engine::audio::emitter_tick::clamp01_11d948(f[0]);
            let vol_q = (vol * 1024.0) as i32;
            if vol_q != (e.f10 * 1024.0) as i32 {
                e.f10 = vol;
            }
        }
        // FUN_001fb520 is a no-op
        self.sounds.push(EffectSound { emitter: id, sound, aseg: self.current, f18: f[0] });
    }
}

impl EffectSink for LevelEffectSink<'_> {
    fn call(&mut self, _a: &mut Asega, c: EffectCall) {
        match c {
            // case 0: block = {s16 sound, s32 oid, s32 near, s32 far, f32 x3}
            EffectCall::K0 { s, w, f } => self.ee_sub_210740(f, s as i32, w[0], w[1], w[2]),
            // case 2: FUN_0014f750(g_cm, s16)
            EffectCall::E2(k) => match self.cm.as_deref_mut() {
                Some(cm) => FUN_0014f750(cm, k as i32, self.clk),
                None => self.pending.push(c),
            },
            // case 3: FUN_00187e28(duration = +0xc, &g_pad, id = +8)
            EffectCall::E3 { a, t } => self.rumble.push(RumbleRequest { pattern: a as usize, duration: t.unwrap_or(0.0) }),
            // case 4: class 0x6d -> FUN_0012f9b8(.., svt 1.0, obj, ..)
            EffectCall::Spawn(oid) => match self.world.find_oid(oid).iter().copied().find(|&i| self.world.objects[i].class == CLASS_ASEG) {
                Some(i) => self.start.push(i),
                None => self.pending.push(c),
            },
            other => self.pending.push(other),
        }
    }
}

/// One live ASEGA.
#[derive(Clone, Debug)]
pub struct LevelAnim {
    pub object: usize,
    pub aseg: Arc<Aseg>,
    pub state: Asega,
}

/// The live ASEGA set of the running level.
#[derive(Clone, Debug, Default)]
pub struct LevelAnims {
    pub players: Vec<LevelAnim>,
    pub sounds: Vec<EffectSound>,
    pub pending: Vec<EffectCall>,
    pub frames: u64,
}

fn aseg_of(w: &World, i: usize) -> Option<&Aseg> {
    match &w.objects.get(i)?.payload {
        Payload::Loaded(LoadData::Aseg(a)) => Some(a),
        // POOL1 r39: a FUN_001f66f0 clone child ASEG shares its source's data
        Payload::Clone => aseg_of(w, w.objects[i].clone_of?),
        _ => None,
    }
}

/// ANIM-ALL R17: LevelAnims starts queued for `LiveQ::guest_anim_starts` (t, rate, aseg handle, obj handle, flags).
static GUEST_STARTS: std::sync::Mutex<Vec<(usize, (f32, f32, u32, u32, u32))>> = std::sync::Mutex::new(Vec::new());

/// The level ASEG starts are mirrored into the guest (`SLY2_GUEST_LEVEL_ANIMS=0` turns it off). ANIM-ALL R19: on by
/// default; the R18 cellar overrun (>1e8 ops per no-target start) was the FUN_001f80d0 ABI bug, not the search.
pub fn guest_level_anims() -> bool {
    std::env::var("SLY2_GUEST_LEVEL_ANIMS").map_or(true, |v| v != "0")
}

/// Queue key of a level: its level bytes (shared by the World clones of the game / class / LiveQ sides;
/// parallel test levels share the queue).
pub fn world_key(w: &World) -> usize {
    Arc::as_ptr(&w.level_data.0) as usize
}

/// The queued starts of the level `world_key`.
pub fn take_guest_starts(world: usize) -> Vec<(f32, f32, u32, u32, u32)> {
    let mut q = GUEST_STARTS.lock().unwrap();
    let (mine, rest): (Vec<_>, Vec<_>) = q.drain(..).partition(|e| e.0 == world);
    *q = rest;
    mine.into_iter().map(|e| e.1).collect()
}

impl LevelAnims {
    /// FUN_0012f9b8 (play `obj` at speed `svt`, flags 0): a running ASEGA of that object restarts.
    pub fn play(&mut self, w: &World, obj: usize, svt: f32) -> bool {
        let Some(a) = aseg_of(w, obj) else { return false };
        // ANIM-ALL R17: the same start in the guest image (FUN_0012f9b8(t 0, svt, aseg, no target, flags 0)), which
        // links the ASEGA into world+0x3d88 for the guest pass; the native player below keeps the event effects
        if guest_level_anims() {
            let mut q = GUEST_STARTS.lock().unwrap();
            if q.len() < 4096 {
                q.push((world_key(w), (0.0, svt, obj as u32 + 1, 0, 0)));
            }
        }
        let state = Asega { svt, svt2: svt, real_clock: false, ..Default::default() };
        self.players.retain(|p| p.object != obj);
        self.players.push(LevelAnim { object: obj, aseg: Arc::new(a.clone()), state });
        true
    }

    /// Replacement start set (see module doc): every class-0x6d ASEG in `active`.
    pub fn start_live(&mut self, w: &World, active: &[usize]) {
        self.players.clear();
        for &i in active {
            // WP-EP1e: not the ASEGs of a JT (class 53): the JT state machine (player/) starts those; playing
            // Sly's whole move set at load fired ~250 full-volume sounds in the first 2 s (Paris, row 2)
            let jt_owned = w.objects.get(i).and_then(|o| o.owner).is_some_and(|o| w.objects.get(o).is_some_and(|o| o.class == crate::engine::playerdraw::CLASS_JT));
            if !jt_owned && w.objects.get(i).is_some_and(|o| o.class == CLASS_ASEG) {
                self.play(w, i, 1.0);
            }
        }
    }

    /// One frame: retract requests, FUN_00130918 for every live ASEGA, then the ASEGs events started.
    #[allow(clippy::too_many_arguments)]
    pub fn step(&mut self, w: &World, dt: f32, real_dt: f32, emitters: &mut EmitterSystem, rumble: &mut Vec<RumbleRequest>, cm: Option<&mut Cm>, clk: CmClock, retract: &mut Vec<u32>) {
        for o in retract.drain(..) {
            for p in self.players.iter_mut().filter(|p| p.object == o as usize) {
                p.state.alive = false; // FUN_00130b18
            }
        }
        let mut sink = LevelEffectSink { world: w, emitters, rumble, cm, clk, current: 0, sounds: vec![], start: vec![], pending: vec![] };
        for p in self.players.iter_mut().filter(|p| p.state.alive) {
            sink.current = p.object;
            let mut host = EffectHost(&mut sink);
            ee_sub_130918(&mut p.state, &p.aseg, dt, real_dt, &mut host);
        }
        let LevelEffectSink { sounds, start, pending, .. } = sink;
        self.players.retain(|p| p.state.alive);
        for i in start {
            self.play(w, i, 1.0);
        }
        self.sounds.extend(sounds);
        self.pending.extend(pending);
        self.frames += 1;
    }
}
