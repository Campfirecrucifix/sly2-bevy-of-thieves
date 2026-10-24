//! WP-G3: the sound-emitter class 0x2d9c70 (base C2d9d00) per-frame machine, 1:1. The music manager
//! FUN_001fd830 walks the active list 0x3dd08c and calls FUN_001fb8e0 on every emitter; the emitter's
//! flag word +0x50 (FUN_001fb658) holds the pending requests:
//!   1 start, 2 pause, 4 resume, 8 stop, 0x10 volume, 0x20 pan, 0x40 pitch (0x1f0 = any parameter),
//!   0x200 positional (3D update FUN_001fb528 every frame), 0x400 poll the voice (kind-1 sounds).
//! vtable 0x2d9c70 (entry k at +8k = {s16 this-adjust, fn}): +0xc add-ref FUN_001fc090, +0x14 release
//! FUN_001fc0a0, +0x3c FUN_001fb0c0, +0x44 kind FUN_001fbc20, +0x4c start FUN_001fbc58, +0x54 stop
//! FUN_001fbe80, +0x5c pause FUN_001fbee0, +0x64 resume FUN_001fbf00, +0x6c params FUN_001fbf20,
//! +0x74 poll FUN_001fbfb0, +0x7c FUN_001fc018, +0x84 3D FUN_001fb528. (C2d9d00 has no-op slots 8..15.)
//!
//! REPLACED: the 989snd EE client (snd_SendIOPCommandNoWait 0x232838 and its wrappers) -> [`SndCmd`]s in
//! [`Snd989`], drained by the host (bevy_audio, `audio::bevy_out`). The IOP's completion callbacks
//! (FUN_001fbaf8 voice handle, FUN_001fbb30 poll result) run in [`Snd989::flush_231c48`] like
//! snd_FlushSoundCommands 0x231c48 delivers them at the end of FUN_001fd830. Voice handles are allocated
//! here; the host reports finished voices in `Snd989::ended`.
#![allow(non_snake_case)]

use super::emitter::{EmitterId, EmitterSystem};
use bevy_ecs::prelude::Resource;
use std::collections::HashSet;

/// 989snd commands the game sends (SndCmd = replaced snd_SendIOPCommandNoWait call).
#[derive(Clone, Debug, PartialEq)]
pub enum SndCmd {
    /// FUN_002322f0 (sync, cmd 0x5c) / FUN_00232320 (async with callback): play bank sound `bank_sound`
    /// (FUN_001fd070 = index of the 0x3dd040 sound record). `mask` 3 | 4 vol | 8 pan | 0x10 pitch.
    /// `voice` = the handle the async callback receives (None = fire and forget).
    Play { emitter: EmitterId, sound: i32, bank_sound: i32, mask: u32, vol: i32, pan: i32, pitch: i16, voice: Option<u32> },
    /// FUN_00232350 (cmd 0x15)
    Stop { voice: u32 },
    /// FUN_00232568(voice, -4, frames, 2) (cmd 0x22): fade out over `frames` / 240 s
    StopFade { voice: u32, frames: i32 },
    /// FUN_002323a8 (cmd 0x13)
    Pause { voice: u32 },
    /// FUN_002323d8 (cmd 0x14)
    Resume { voice: u32 },
    /// FUN_00232500(voice, mask, vol, pan, pitch, 0): mask 1 vol(1024 = 1.0) | 4 pan | 8 pitch
    SetParams { voice: u32, mask: u32, vol: i32, pan: i32, pitch: i32 },
    /// FUN_002321e8 (cmd 9): bus master volume (0x3dd040 +0x1c0[bus])
    BusVolume { bus: usize, vol: f32 },
    /// FUN_00232250 (FUN_001fe9c8): output mode of the options menu (0x3dd040 +0x104); WP-X2
    OutputMode { mode: i32 },
    /// thunk 0x232408 (FUN_001fd700): stop every voice; WP-X2
    StopAll,
    /// FUN_002330d0('SUKR', cmd, ...) IOP query (cmd 0xb / 0xc of FUN_001fda48 / FUN_001fdba8)
    Sukr { cmd: u32 },
}

/// Sound records of the level (0x3dd040 +0x18 count / +0x100 id map, +0xf8 / +0xfc kinds; FUN_001fd0e8).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SoundTable {
    /// +0xfc: one u8 kind per record (stored as s32): 1 = polled (FUN_001fbfb0), 2 = async / streamed
    pub kinds: Vec<i32>,
    /// +0x100: sound id -> record index (-1 = none)
    pub map: Vec<i32>,
    /// DAT_003dd128: the bank handle (0 = no bank loaded)
    pub bank: u32,
}

impl SoundTable {
    pub fn from_block(b: &crate::engine::disc::level_file::Block3dd040) -> Self {
        Self { kinds: b.bytes.iter().map(|&k| k as i32).collect(), map: b.map.clone(), bank: 1 }
    }
    /// FUN_001fd028(0x3dd040, id): record index (the record pointer +0xfc + 4 * index), None = 0.
    pub fn record_1fd028(&self, id: i32) -> Option<usize> {
        // 0x001fd028
        if id < 0 || id as usize >= self.map.len() {
            return None;
        }
        let i = self.map[id as usize];
        (i >= 0).then_some(i as usize)
    }
    /// FUN_001fbc20 (vtable +0x44): kind of the emitter's sound, -1 without a record.
    /// Quirk kept: the record pointer is not bounds-checked against +0xf8 in the original; out of range = -1 here.
    pub fn kind_1fbc20(&self, id: i32) -> i32 {
        self.record_1fd028(id).and_then(|i| self.kinds.get(i).copied()).unwrap_or(-1)
    }
}

/// World reads of FUN_001fb528 / FUN_001fb778 / FUN_001fbc58.
pub trait EmitterWorld {
    /// FUN_001c4a48(0x3dd490) + FUN_001a62e8 (player position), else the camera position _DAT_002ddfd0.
    fn listener(&self) -> [f32; 3];
    /// FUN_001a7348(0, 0x2ddfa0, v): `v` in the camera frame (x forward, y left, z up).
    fn to_camera(&self, v: [f32; 3]) -> [f32; 3];
    /// attach object +0x28 bit 60 (object live in the world).
    fn attach_live(&self, obj: i32) -> bool;
    /// FUN_001a6f00(obj, 0, pos): world position of `pos` local to `obj` (obj 0 = world space).
    fn attach_pos(&self, obj: i32, pos: [f32; 3]) -> [f32; 3];
    /// FUN_001fbc58's music-duck test: the global for sound 0x6f (DAT_002e1e40), 0x15 (DAT_002f7900),
    /// 0x73 (DAT_002dd5bc), else 0, compared with the player FUN_001c4840(0x3dd490).
    fn duck_test(&self, sound: i32) -> bool {
        let _ = sound;
        false
    }
}

/// World at the origin with no player: listener (0,0,0), identity camera, no live attach objects.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoWorld;
impl EmitterWorld for NoWorld {
    fn listener(&self) -> [f32; 3] {
        [0.0; 3]
    }
    fn to_camera(&self, v: [f32; 3]) -> [f32; 3] {
        v
    }
    fn attach_live(&self, _obj: i32) -> bool {
        false
    }
    fn attach_pos(&self, _obj: i32, pos: [f32; 3]) -> [f32; 3] {
        pos
    }
}

/// DAT_002a82a8: FUN_001feed0 pitch-step deltas, 7 rows x 6 (row r has r cells of +1 at the end).
pub const STEP_2A82A8: [i32; 42] = [
    -5, -5, -5, -5, -5, -5, -5, -5, -5, -5, -5, 1, -5, -5, -5, -5, 1, 1, -5, -5, -5, 1, 1, 1, -5, -5, 1, 1, 1, 1, -5, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1,
];
/// DAT_002a8350: row of STEP_2A82A8 for step + 12 (step -12..12).
pub const STEP_ROW_2A8350: [i32; 25] = [6, 6, 6, 6, 6, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 4, 3, 2, 1, 0];

/// Replaced 989snd client state (see module doc).
#[derive(Resource, Clone, Debug, Default)]
pub struct Snd989 {
    pub table: SoundTable,
    /// AUD3: Play commands the host has started (bevy_out::drain_snd_cmds), for the live-audio proof
    pub played: u64,
    pub cmds: Vec<SndCmd>,
    pub next_voice: u32,
    /// voices the host has finished (FUN_001fbfb0 poll answers 0 for them)
    pub ended: HashSet<u32>,
    /// callbacks pending until FUN_00231c48: (emitter, Some(voice) = FUN_001fbaf8 / None = FUN_001fbb30 poll)
    pub pending: Vec<(EmitterId, Option<u32>)>,
    /// FUN_001feed0(0x3dd040) calls from FUN_001fbc58 (count; the sound itself is played since WP-X2)
    pub duck_1feed0: u32,
    /// 0x3dd040 +0x64: FUN_001feed0 plays nothing while nonzero
    pub f64: i32,
    /// 0x3dd040 +0x138: FUN_001feed0's pitch step (semitone random walk, -12..12)
    pub step_138: i32,
    /// REPLACED: FUN_0011d3f8 draws from the game's shared rand() (0x2342a0); FUN_001feed0 has its own stream here.
    pub rng: crate::engine::physics::driven::GameRng,
}

impl Snd989 {
    /// Sukr 0x002330d0: packs the 7 args and sends IOP command 0x4c (snd_SendIOPCommandNoWait); the IOP
    /// request is REPLACED by queueing SndCmd::Sukr (its reply comes back through the MusicMgr callbacks).
    pub fn ee_sub_2330d0(&mut self, cmd: u32) {
        self.cmds.push(SndCmd::Sukr { cmd });
    }
    fn voice(&mut self) -> u32 {
        self.next_voice = self.next_voice.wrapping_add(1).max(1);
        self.next_voice
    }

    /// FUN_001feed0(0x3dd040): the pickup jingle. Unless +0x64 is set, a random walk of the pitch step
    /// +0x138 (DAT_002a8350[step + 12] picks a row of DAT_002a82a8, FUN_0011d3f8(0, 5) a column; the cell is
    /// -5 or +1) and sound 0x28 is played with pitch = step x 127 (FUN_002322f0, mask 0x13). The ELF's
    /// "unreachable" blocks are the unaligned-copy variants of the two table copies (obs: 0x001fef08-0x001ff018).
    pub fn ee_sub_1feed0(&mut self, emitter: EmitterId) {
        // 0x001feed0
        if self.f64 != 0 {
            return;
        }
        let r = self.rng.FUN_0011d3f8(0, 5);
        // quirk kept: the step is not range-checked before indexing DAT_002a8350 (it stays in -12..12)
        let row = STEP_ROW_2A8350[(self.step_138 + 12) as usize];
        self.step_138 += STEP_2A82A8[(row * 6 + r) as usize];
        let bank_sound = self.table.record_1fd028(0x28).map_or(0, |i| i as i32); // FUN_001fd070(FUN_001fd028(0x28))
        let pitch = ((self.step_138 << 7) - self.step_138) as i16;
        self.cmds.push(SndCmd::Play { emitter, sound: 0x28, bank_sound, mask: 0x13, vol: 0, pan: 0, pitch, voice: None });
    }

    /// snd_FlushSoundCommands 0x231c48 (REPLACED): the queued commands stay for the host; the completion
    /// callbacks run now, in order.
    pub fn flush_231c48(&mut self, es: &mut EmitterSystem) {
        for (id, v) in std::mem::take(&mut self.pending) {
            match v {
                Some(voice) => es.ee_sub_1fbaf8(voice, id),
                None => {
                    let st = es.get(id).map_or(0, |e| !self.ended.contains(&e.obj40) as i32);
                    es.ee_sub_1fbb30(st, id);
                }
            }
        }
    }
}

/// FUN_0011d948(x, 0x2bbb60 = {0, 1}): x if 0 <= x <= 1, else the violated bound (NaN -> 0).
pub fn clamp01_11d948(x: f32) -> f32 {
    if 0.0 <= x {
        if x <= 1.0 { x } else { 1.0 }
    } else {
        0.0
    }
}

/// FUN_001fb778(this, pos, &{near, far}, &vol, &pan): distance falloff (1 inside `near`, linear to 0 at
/// `far`) and pan angle in degrees (0 ahead, clockwise; +180 when far < near), 0 when silent.
pub fn ee_sub_1fb778(w: &dyn EmitterWorld, pos: [f32; 3], near: f32, far: f32) -> (f32, i32) {
    // 0x001fb778
    let l = w.listener();
    let d = [pos[0] - l[0], pos[1] - l[1], pos[2] - l[2]];
    let dist = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    let f = clamp01_11d948((far - dist) / (far - near));
    let pan = if 0.0 < f {
        let c = w.to_camera(d);
        // PROT3W picks y; FUN_002458f0 = atan2f(-y, x)
        let mut a = (-c[1]).atan2(c[0]);
        if far < near {
            a += 3.1415927;
        }
        (a * 57.295776) as i32
    } else {
        0
    };
    (f, pan)
}

impl EmitterSystem {
    /// FUN_001fb098(fade, this): request a stop (+0x48 = fade seconds; flags 0x1fff -> 8).
    pub fn ee_sub_1fb098(&mut self, fade: f32, id: EmitterId) {
        // 0x001fb098
        let Some(e) = self.get_mut(id) else { return };
        e.fade = fade;
        self.set_flags_1fb658(id, 0x1fff, 8);
    }
    /// FUN_001fb018: ++pause count; 0 -> 1 requests a pause (flags 6 -> 2).
    pub fn ee_sub_1fb018(&mut self, id: EmitterId) {
        // 0x001fb018
        let Some(e) = self.get_mut(id) else { return };
        e.s4e += 1;
        if e.s4e == 1 {
            self.set_flags_1fb658(id, 6, 2);
        }
    }
    /// FUN_001fb060: --pause count; 1 -> 0 requests a resume (flags 6 -> 4).
    pub fn ee_sub_1fb060(&mut self, id: EmitterId) {
        // 0x001fb060
        let Some(e) = self.get_mut(id) else { return };
        let old = e.s4e;
        e.s4e = old - 1;
        if old == 1 {
            self.set_flags_1fb658(id, 6, 4);
        }
    }
    /// FUN_001fb1f8(vol, this): +0x10 = clamp01(vol) when it changes at 1/1024 resolution (flag 0x10).
    pub fn ee_sub_1fb1f8(&mut self, vol: f32, id: EmitterId) {
        // 0x001fb1f8
        let v = clamp01_11d948(vol);
        let Some(e) = self.get_mut(id) else { return };
        if (v * 1024.0) as i32 != (e.f10 * 1024.0) as i32 {
            e.f10 = v;
            self.set_flags_1fb658(id, 0x10, 0x10);
        }
    }
    /// FUN_001fb2c0(pitch, this): +0x18 (flag 0x40) when it changes.
    pub fn ee_sub_1fb2c0(&mut self, pitch: f32, id: EmitterId) {
        // 0x001fb2c0
        let Some(e) = self.get_mut(id) else { return };
        if pitch != e.pitch {
            e.pitch = pitch;
            self.set_flags_1fb658(id, 0x40, 0x40);
        }
    }
    /// FUN_001fb268(this, deg): +0x14 = deg mod 360 in 0..360 (flag 0x20) when it changes.
    pub fn ee_sub_1fb268(&mut self, id: EmitterId, deg: i32) {
        // 0x001fb268
        let p = deg % 360;
        let p = if p > -1 { p } else { p + 360 };
        let Some(e) = self.get_mut(id) else { return };
        if p != e.pan {
            e.pan = p;
            self.set_flags_1fb658(id, 0x20, 0x20);
        }
    }

    /// FUN_001fb528 (vtable +0x84): 3D update - stop when the attach object left the world, else volume
    /// = +0x10 x falloff (1/1024 resolution; 0 <-> non-0 transitions go through FUN_001fc018) and pan.
    pub fn ee_sub_1fb528(&mut self, snd: &mut Snd989, w: &dyn EmitterWorld, id: EmitterId) {
        // 0x001fb528
        let Some(e) = self.get(id) else { return };
        let attach = e.unpositioned;
        if attach != 0 && !w.attach_live(attach) {
            self.ee_sub_1fb098(0.0, id);
            return;
        }
        let p = w.attach_pos(attach, e.pos);
        let (f, pan) = ee_sub_1fb778(w, p, e.near, e.far);
        let v = e.f10 * f;
        if (v * 1024.0) as i32 != (e.f44 * 1024.0) as i32 {
            if e.f44 == 0.0 {
                self.ee_sub_1fc018(snd, id, 0);
                if let Some(e) = self.get_mut(id) {
                    e.f44 = v;
                }
            } else {
                if v == 0.0 {
                    self.ee_sub_1fc018(snd, id, 1);
                }
                if let Some(e) = self.get_mut(id) {
                    e.f44 = v;
                }
            }
            self.set_flags_1fb658(id, 0x10, 0x10);
        }
        self.ee_sub_1fb268(id, pan);
    }

    /// FUN_001fc018 (vtable +0x7c): audible again (0) -> FUN_001fb0c0; silent (1) -> stop request
    /// (flags 0x3ff, or 0x1ff for kind-2 sounds, -> 8).
    pub fn ee_sub_1fc018(&mut self, snd: &Snd989, id: EmitterId, silent: i32) {
        // 0x001fc018
        if silent == 0 {
            self.refresh_1fb0c0(id);
        } else {
            let k = self.get(id).map_or(-1, |e| snd.table.kind_1fbc20(e.sound));
            self.set_flags_1fb658(id, if k == 2 { 0x1ff } else { 0x3ff }, 8);
        }
    }

    /// FUN_001fbc58 (vtable +0x4c): start the sound.
    pub fn ee_sub_1fbc58(&mut self, snd: &mut Snd989, w: &dyn EmitterWorld, id: EmitterId) {
        // 0x001fbc58
        let Some(e) = self.get(id).cloned() else { return };
        let rec = snd.table.record_1fd028(e.sound);
        let Some(rec) = rec.filter(|_| snd.table.bank != 0) else {
            self.set_flags_1fb658(id, 0x1fff, 0);
            return;
        };
        if e.flags & 2 != 0 {
            // FUN_001fb710: leave the active list (paused before it started)
            if self.active.contains(&id) {
                self.active.retain(|&a| a != id);
                self.release(id);
            }
            return;
        }
        let mut mask = 3u32;
        let (mut vol, mut pan, mut pitch) = (0, 0, 0i16);
        if e.flags & 0x10 != 0 {
            mask |= 4;
            vol = (e.f44 * 1024.0) as i32;
        }
        if e.flags & 0x20 != 0 {
            pan = e.pan;
            mask |= 8;
        }
        if e.flags & 0x40 != 0 {
            mask |= 0x10;
            pitch = (e.pitch * 1524.0) as i32 as i16;
        }
        let kind = snd.table.kinds.get(rec).copied().unwrap_or(-1);
        if kind != 2 {
            snd.cmds.push(SndCmd::Play { emitter: id, sound: e.sound, bank_sound: rec as i32, mask, vol, pan, pitch, voice: None });
            self.set_flags_1fb658(id, 0x1fff, 0);
            if w.duck_test(e.sound) {
                snd.duck_1feed0 += 1;
                snd.ee_sub_1feed0(id);
            }
            return;
        }
        if let Some(e) = self.get_mut(id) {
            e.s52 = 1;
        }
        self.add_ref(id);
        let voice = snd.voice();
        snd.cmds.push(SndCmd::Play { emitter: id, sound: e.sound, bank_sound: rec as i32, mask, vol, pan, pitch, voice: Some(voice) });
        snd.pending.push((id, Some(voice)));
        self.set_flags_1fb658(id, 0x7f, 0);
    }

    /// FUN_001fbaf8(voice, this): async start callback - +0x40 = voice, +0x52 = 0, release.
    pub fn ee_sub_1fbaf8(&mut self, voice: u32, id: EmitterId) {
        // 0x001fbaf8
        let Some(e) = self.get_mut(id) else { return };
        e.obj40 = voice;
        e.s52 = 0;
        self.release(id);
    }

    /// FUN_001fbb30(status, this): poll callback - 0 (voice ended) -> FUN_001fb098(0), else poll again.
    pub fn ee_sub_1fbb30(&mut self, status: i32, id: EmitterId) {
        // 0x001fbb30
        if status == 0 {
            self.ee_sub_1fb098(0.0, id);
        } else {
            self.set_flags_1fb658(id, 0x400, 0x400);
        }
        self.release(id);
    }

    /// FUN_001fbe80 (vtable +0x54): stop the voice, fading over +0x48 s (x240 frames) when > 0.
    pub fn ee_sub_1fbe80(&self, snd: &mut Snd989, id: EmitterId) {
        // 0x001fbe80
        let Some(e) = self.get(id) else { return };
        if 0.0 < e.fade {
            snd.cmds.push(SndCmd::StopFade { voice: e.obj40, frames: (e.fade * 240.0) as i32 });
        } else {
            snd.cmds.push(SndCmd::Stop { voice: e.obj40 });
        }
    }

    /// FUN_001fbf20 (vtable +0x6c): send the changed parameters (flags 0x10 vol, 0x20 pan, 0x40 pitch x12).
    /// Unset values are uninitialised registers in the original (0 here).
    pub fn ee_sub_1fbf20(&self, snd: &mut Snd989, id: EmitterId) {
        // 0x001fbf20
        let Some(e) = self.get(id) else { return };
        let u = e.flags;
        if u & 0x70 == 0 {
            return;
        }
        let (mut m, mut vol, mut pitch) = (0u32, 0, 0);
        if u & 0x10 != 0 {
            m = 1;
            vol = (e.f44 * 1024.0) as i32;
        }
        if u & 0x20 != 0 {
            m |= 4;
        }
        if u & 0x40 != 0 {
            m |= 8;
            pitch = (e.pitch * 12.0) as i32;
        }
        snd.cmds.push(SndCmd::SetParams { voice: e.obj40, mask: m, vol, pan: e.pan, pitch });
    }

    /// FUN_001fbfb0 (vtable +0x74): poll the voice (flag 0x400) -> FUN_001fbb30 at the next flush.
    pub fn ee_sub_1fbfb0(&mut self, snd: &mut Snd989, id: EmitterId) {
        // 0x001fbfb0
        let Some(e) = self.get(id) else { return };
        if e.flags & 0x400 != 0 {
            self.add_ref(id);
            snd.pending.push((id, None));
            self.set_flags_1fb658(id, 0x400, 0);
        }
    }

    /// FUN_001fb8e0: per-frame emitter update (see module doc).
    pub fn ee_sub_1fb8e0(&mut self, snd: &mut Snd989, w: &dyn EmitterWorld, id: EmitterId) {
        // 0x001fb8e0
        if self.get(id).is_none() {
            return;
        }
        self.add_ref(id);
        self.tick_body(snd, w, id);
        self.release(id);
    }

    fn tick_body(&mut self, snd: &mut Snd989, w: &dyn EmitterWorld, id: EmitterId) {
        let Some(e) = self.get_mut(id) else { return };
        if e.flags & 0x200 == 0 {
            e.f44 = e.f10;
        } else {
            self.ee_sub_1fb528(snd, w, id);
        }
        let Some(e) = self.get(id) else { return };
        let u = e.flags;
        if u & 1 != 0 {
            self.ee_sub_1fbc58(snd, w, id);
            let k = self.get(id).map_or(-1, |e| snd.table.kind_1fbc20(e.sound));
            if k == 1 {
                self.set_flags_1fb658(id, 0x400, 0x400);
            }
            return;
        }
        if e.obj40 == 0 {
            if e.s52 == 0 && u & 0x200 == 0 {
                self.set_flags_1fb658(id, 0x1fff, 0);
            }
            return;
        }
        if u & 8 != 0 {
            self.ee_sub_1fbe80(snd, id);
            if let Some(e) = self.get_mut(id) {
                e.obj40 = 0;
            }
            self.set_flags_1fb658(id, 0x1dff, 0);
            return;
        }
        if u & 0x1c00 != 0 {
            self.ee_sub_1fbfb0(snd, id);
        }
        let Some(e) = self.get(id) else { return };
        if e.s52 != 0 {
            return;
        }
        if e.flags & 0x1f0 != 0 {
            self.ee_sub_1fbf20(snd, id);
            self.set_flags_1fb658(id, 0x1f0, 0);
        }
        let Some(e) = self.get(id) else { return };
        let (u, voice) = (e.flags, e.obj40);
        let m = if u & 2 != 0 {
            snd.cmds.push(SndCmd::Pause { voice }); // FUN_001fbee0
            2
        } else if u & 4 != 0 {
            snd.cmds.push(SndCmd::Resume { voice }); // FUN_001fbf00
            4
        } else {
            return;
        };
        self.set_flags_1fb658(id, m, 0);
    }
}

/// FUN_001faf70: allocates a 0x14-byte sound parameter block {sound -1, near 300, far 3000, volume 1}.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SoundParams {
    pub sound: i32,
    pub near: f32,
    pub far: f32,
    pub volume: f32,
}
impl SoundParams {
    pub fn ee_sub_1faf70() -> Self {
        // 0x001faf70
        Self { sound: -1, near: 300.0, far: 3000.0, volume: 1.0 }
    }
}
