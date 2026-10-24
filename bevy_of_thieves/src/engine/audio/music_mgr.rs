//! WP-G3: the music / voice manager 0x3dd040 step FUN_001fd830 (world update) and its callees, 1:1:
//! FUN_001fda48 (SUKR IOP level poll -> DAT_003e14c4 curve), FUN_001fdba8 (SUKR poll 0xc), the pause gate
//! +0x58 (DAT_002bd4c0 != 3 pauses the playing instances), FUN_001fdc08 (`music_layers`), FUN_001fdec0 (empty),
//! FUN_001fdec8 (dialogue cue scheduler) + FUN_001fe0e8 (cue dispatch), the pause fade on bus slot 2,
//! FUN_001fea60 (bus volume matrix), FUN_001fb8e0 on every active emitter, FUN_00231c48 flush.
//! Instances: +0x200 the playing dialogue line (a 0x3dd040 +0xec record = `Rec5c`: +0x50 duration (f32),
//! +0x54/+0x58 cues {s16 delta frames, s16 layer slot, s32 arg}; instance +0x68 = play time in EE bus cycles,
//! x 3.390842e-09 = 1 / 294.912 MHz), +0x204/+0x208.. the music instance stack, +0x228 the level ambience
//! (DAT_003dd268). They share the emitter flag / pause-count layout and live in `EmitterSystem` here.
//! Volume matrix +0x13c [8 slots][4 buses] (all 1.0 after FUN_001fcea0), bus product +0x1c0 -> 989snd bus
//! volume (REPLACED by `SndCmd::BusVolume`). Slot 1 = the options sliders (bus 1 music, buses 0 / 3 effects:
//! ui/settings.rs), slot 2 = the pause fade of FUN_001fd830.
#![allow(non_snake_case)]

use super::emitter::{EmitterId, EmitterSystem};
use super::emitter_tick::{clamp01_11d948, EmitterWorld, Snd989, SndCmd};
use super::music_layers::{InstState, MusicLayers, PlayerMusic};
use crate::engine::physics::dynamics::FUN_0011d018;
use bevy_ecs::prelude::Resource;

/// DAT_002a8288: FUN_0011d018 profile of the pause fade {vmax 2, v0 0, accel 0.2}.
pub const PAUSE_FADE: [f32; 3] = [2.0, 0.0, 0.2];
/// 0x31690453: seconds per EE bus cycle.
pub const EE_CYCLE: f32 = 3.390842e-09;

/// A playing dialogue line (instance +0x60 record fields FUN_001fdec8 reads, + the instance clock +0x68).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DialogLine {
    /// record +0x50
    pub duration: f32,
    /// record +0x58: (delta frames, layer slot, arg)
    pub cues: Vec<(i16, i16, i32)>,
    /// instance +0x68 (EE cycles played)
    pub cycles: u64,
}

impl DialogLine {
    pub fn from_rec(r: &crate::engine::disc::level_file::Rec5c) -> Self {
        let cues = r.data.iter().map(|c| (i16::from_le_bytes([c[0], c[1]]), i16::from_le_bytes([c[2], c[3]]), i32::from_le_bytes([c[4], c[5], c[6], c[7]]))).collect();
        Self { duration: f32::from_bits(r.u32v), cues, cycles: 0 }
    }
}

/// Calls of the manager into other subsystems (no-op defaults).
pub trait MusicHost {
    /// FUN_001c4840(0x3dd490) is class 0x34 (FUN_00132b10): its position (FUN_001a62e8) and FUN_001fdc08 fields.
    fn player(&self) -> Option<([f32; 3], PlayerMusic)> {
        None
    }
    /// FUN_001fda48: DAT_003e14c4 +0x54 / +0x58 / +0x5c written (only when the SUKR level > 0.5).
    fn set_3e14c4(&mut self, _f54: f32, _f58: f32, _f5c: f32) {}
    /// FUN_00156be0(DAT_003e14c4 or 0, player position, player, 0x3dd040 + 0x74)
    fn ee_sub_156be0(&mut self, _active: bool, _pos: [f32; 3]) {}
    /// FUN_001fe0e8 slot 3: FUN_001f80d0(arg, +0x238) (class-0x8e children of the slot-3 object)
    fn ee_sub_1f80d0(&mut self, _arg: i32, _obj: u32) {}
    /// FUN_001fe0e8 other slots: FUN_001f0660 (SMA state) on the slot object with `arg`
    fn ee_sub_1f0660(&mut self, _obj: u32, _arg: i32) {}
    /// FUN_001fe738: release (vtable +0x14) of a +0x22c layer object
    fn release_obj(&mut self, _obj: u32) {}
}
pub struct NoMusicHost;

/// AUD3: the live game's host: the player (class 0x34) position + FUN_001fdc08 fields; the other calls
/// keep their no-op defaults (DAT_003e14c4 / slot objects are not live objects yet).
pub struct LiveMusicHost(pub Option<([f32; 3], PlayerMusic)>);
impl MusicHost for LiveMusicHost {
    fn player(&self) -> Option<([f32; 3], PlayerMusic)> {
        self.0
    }
}
impl MusicHost for NoMusicHost {}

/// Clock values FUN_001fd830 reads.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MusicClock {
    /// FUN_0014d240(0x2dded0)
    pub ticks: u64,
    /// DAT_002dded4 (scaled dt; 0 = paused)
    pub dt: f32,
    /// DAT_002ddf10 (dt x scale[3])
    pub dt_unscaled: f32,
    /// DAT_002bd4c0 (3 = in game)
    pub game_state: i32,
}

/// 0x3dd040 (the parts FUN_001fd830 uses).
#[derive(Resource, Clone, Debug, PartialEq)]
pub struct MusicMgr {
    /// +0x10 SUKR level poll enabled
    pub f10: i32,
    /// +0x58 instances paused by the game-state gate
    pub f58: i32,
    /// +0x68 (3 = pause fade follows the clock)
    pub f68: i32,
    /// +0x6c request pending, +0x70 result (callback 0x1fcf70)
    pub f6c: i32,
    pub f70: f32,
    /// +0x80..+0x88 / +0x90..+0x98 quadratic curves of +0x70
    pub c80: [f32; 3],
    pub c90: [f32; 3],
    /// +0xa0 request pending, +0xa4 result (callback 0x1fcf88)
    pub fa0: i32,
    pub fa4: u32,
    /// +0x13c volume matrix [slot][bus], +0x1c0 bus products
    pub vol: [[f32; 4]; 8],
    pub bus: [f32; 4],
    /// +0x1f0 / +0x1f8 clock ticks (this / previous frame)
    pub t1f0: u64,
    pub t1f8: u64,
    /// +0x200 dialogue instance (+ its record), +0x204 stack, +0x228 ambience
    pub cur: Option<EmitterId>,
    pub line: Option<DialogLine>,
    pub stack: Vec<EmitterId>,
    pub amb: Option<EmitterId>,
    /// +0x22c..+0x240 layer objects (0 = none)
    pub objs: [u32; 6],
    /// +0x244 next cue, +0x248 cue time, +0x24c elapsed
    pub cue: i32,
    pub cue_t: f32,
    pub elapsed: f32,
    pub layers: MusicLayers,
    /// +0x104 output mode (FUN_001fe9c8); WP-X2
    pub f104: i32,
    /// +0x1bc master volume (FUN_001fe9f8; 1.0 after FUN_001fcea0); WP-X2
    pub master: f32,
    /// FUN_001fe738 requests from FUN_001fdec8 (line over; the stop itself is game::vm_state / classes::k26)
    pub line_end: u32,
}

impl Default for MusicMgr {
    fn default() -> Self {
        // FUN_001fcea0: matrix and products 1.0
        Self {
            f10: 0,
            f58: 0,
            f68: 0,
            f6c: 0,
            f70: 0.0,
            c80: [0.0; 3],
            c90: [0.0; 3],
            fa0: 0,
            fa4: 0,
            vol: [[1.0; 4]; 8],
            bus: [1.0; 4],
            t1f0: 0,
            t1f8: 0,
            cur: None,
            line: None,
            stack: vec![],
            amb: None,
            objs: [0; 6],
            cue: 0,
            cue_t: 0.0,
            elapsed: 0.0,
            layers: MusicLayers::default(),
            f104: 0,
            master: 1.0,
            line_end: 0,
        }
    }
}

fn idle(e: &super::emitter::SoundEmitter) -> bool {
    // the "not started / starting" test shared by FUN_001fb0c0 / FUN_001fdec8
    e.obj40 == 0 && e.s52 == 0 && e.flags & 1 == 0
}

impl MusicMgr {
    /// FUN_001fea60(vol, 0x3dd040, bus, slot): matrix entry = clamp01(vol); slot 2 changes to / from 0 pause
    /// (FUN_001fb018) / resume (FUN_001fb060) the bus's instances (bus 0 +0x228, 1 the stack, 2 +0x200); the
    /// bus product goes to 989snd when it changes at 1/1024 resolution.
    pub fn ee_sub_1fea60(&mut self, es: &mut EmitterSystem, snd: &mut Snd989, vol: f32, bus: i64, slot: u64) {
        // 0x001fea60
        if !(slot < 8 && (0..4).contains(&bus)) {
            return;
        }
        let (b, s) = (bus as usize, slot as usize);
        let v = clamp01_11d948(vol);
        if self.vol[s][b] == v {
            return;
        }
        if s == 2 {
            // member-function pointers DAT_002a8298 = FUN_001fb018, DAT_002a82a0 = FUN_001fb060
            let f: Option<fn(&mut EmitterSystem, EmitterId)> = if v == 0.0 {
                Some(EmitterSystem::ee_sub_1fb018)
            } else if self.vol[2][b] == 0.0 {
                Some(EmitterSystem::ee_sub_1fb060)
            } else {
                None
            };
            if let Some(f) = f {
                match b {
                    1 => self.stack.clone().into_iter().for_each(|i| f(es, i)),
                    0 => self.amb.into_iter().for_each(|i| f(es, i)),
                    2 => self.cur.into_iter().for_each(|i| f(es, i)),
                    _ => {}
                }
            }
        }
        self.vol[s][b] = v;
        let p: f32 = (0..8).map(|k| self.vol[k][b]).product();
        if (self.bus[b] * 1024.0) as i32 != (p * 1024.0) as i32 {
            self.bus[b] = p;
            snd.cmds.push(SndCmd::BusVolume { bus: b, vol: p }); // FUN_002321e8
        }
    }

    /// Options menu volumes (slot 1; ui/settings.rs mapping): bus 1 music, buses 0 and 3 effects.
    pub fn set_option_volumes(&mut self, es: &mut EmitterSystem, snd: &mut Snd989, music: f32, effects: f32) {
        self.ee_sub_1fea60(es, snd, effects, 0, 1);
        self.ee_sub_1fea60(es, snd, music, 1, 1);
        self.ee_sub_1fea60(es, snd, effects, 3, 1);
    }

    /// FUN_001fda48: SUKR level poll (REPLACED IOP request; the result arrives in +0x70 via
    /// `sukr_result_0b`), DAT_003e14c4 curves when the level > 0.5, FUN_00156be0.
    pub fn ee_sub_1fda48(&mut self, snd: &mut Snd989, h: &mut dyn MusicHost) {
        // 0x001fda48
        if self.f10 == 0 {
            self.f70 = 0.0;
        } else if self.f6c == 0 {
            self.f6c = 1;
            snd.ee_sub_2330d0(0xb);
        }
        let mut pos = [0.0; 3];
        let mut active = false;
        if let Some((p, _)) = h.player() {
            if self.f70 > 0.5 {
                pos = p;
                let x = self.f70;
                let f54 = self.c80[0] + x * (self.c80[1] + x * self.c80[2]);
                let f5c = self.c90[0] + x * (self.c90[1] + x * self.c90[2]);
                h.set_3e14c4(f54, -f5c, f5c);
                active = true;
            }
        }
        h.ee_sub_156be0(active, pos);
    }
    /// callback 0x1fcf70: +0x70 = result bits, +0x6c = 0
    pub fn sukr_result_0b(&mut self, bits: u32) {
        self.f70 = f32::from_bits(bits);
        self.f6c = 0;
    }
    /// FUN_001fdba8: SUKR poll 0xc (callback 0x1fcf88: +0xa4 = result, +0xa0 = 0)
    pub fn ee_sub_1fdba8(&mut self, snd: &mut Snd989) {
        // 0x001fdba8
        if self.fa0 == 0 {
            self.fa0 = 1;
            snd.ee_sub_2330d0(0xc);
        }
    }
    pub fn sukr_result_0c(&mut self, v: u32) {
        self.fa4 = v;
        self.fa0 = 0;
    }

    /// FUN_001fe0e8(0x3dd040, cue): slot 3 -> FUN_001f80d0(arg, +0x238); other slots with an object and
    /// arg != -1 -> FUN_001f0660.
    pub fn ee_sub_1fe0e8(&self, h: &mut dyn MusicHost, cue: (i16, i16, i32)) {
        // 0x001fe0e8
        let (_, slot, arg) = cue;
        if slot == -1 {
            return;
        }
        // quirk kept: the slot is not range-checked (+0x22c + 4 * slot); out of range = no object here
        let Some(&obj) = usize::try_from(slot).ok().and_then(|s| self.objs.get(s)) else { return };
        if obj == 0 {
            return;
        }
        if slot == 3 {
            h.ee_sub_1f80d0(arg, self.objs[3]);
        } else if arg != -1 {
            h.ee_sub_1f0660(obj, arg);
        }
    }

    /// FUN_001fdec8: dialogue cues. While the line plays, fire every cue whose cumulative time has been
    /// reached by the instance clock; once the instance is idle (finished) and either its clock is > 0 or
    /// `elapsed` reached the record duration, the line ends (FUN_001fe738, counted in `line_end`).
    pub fn ee_sub_1fdec8(&mut self, es: &EmitterSystem, h: &mut dyn MusicHost, dt_unscaled: f32) {
        // 0x001fdec8
        let Some(e) = self.cur.and_then(|i| es.get(i)) else { return };
        if e.s52 != 0 || e.flags & 1 != 0 || e.s4e >= 1 {
            return;
        }
        let Some(line) = self.line.clone() else { return };
        self.elapsed += dt_unscaled;
        if idle(e) {
            if 0.0 < line.cycles as f32 * EE_CYCLE || line.duration <= self.elapsed {
                self.line_end += 1;
            }
            return;
        }
        while (self.cue as usize) < line.cues.len() {
            let c = line.cues[self.cue as usize];
            let ft = c.0 as i32 as f32 * 0.016666668;
            if (line.cycles as f32) * EE_CYCLE < self.cue_t + ft {
                break;
            }
            self.ee_sub_1fe0e8(h, c);
            self.cue += 1;
            self.cue_t += ft;
        }
    }

    /// FUN_001fe738(0x3dd040): end the dialogue line +0x200 - fire the remaining cues, SMA state 0x611 on the
    /// slot-5 object +0x240, stop the instance (FUN_001fb098(0)) and drop it (FUN_00259b48(+0x200, 0)),
    /// reset +0x24c / +0x244 / +0x248, release the six layer objects +0x22c and clear them (memset 0x18).
    pub fn ee_sub_1fe738(&mut self, es: &mut EmitterSystem, h: &mut dyn MusicHost) {
        // 0x001fe738
        let Some(cur) = self.cur else { return };
        if let Some(line) = self.line.clone() {
            while (self.cue as usize) < line.cues.len() {
                self.ee_sub_1fe0e8(h, line.cues[self.cue as usize]);
                self.cue += 1;
            }
        }
        if self.objs[5] != 0 {
            h.ee_sub_1f0660(self.objs[5], 0x611);
        }
        es.ee_sub_1fb098(0.0, cur);
        es.release(cur); // FUN_00259b48(+0x200, 0): the smart pointer drops its reference
        self.cur = None;
        self.line = None;
        self.elapsed = 0.0;
        self.cue = 0;
        self.cue_t = 0.0;
        for o in self.objs {
            if o != 0 {
                h.release_obj(o);
            }
        }
        self.objs = [0; 6];
    }

    /// FUN_001fe3f8(0x3dd040): play time in seconds of the top music instance (+0x204 count, +0x208 stack;
    /// 0 when empty). Quirk kept: the original reads `this + 0x204 + 4 * count` = the top entry.
    pub fn ee_sub_1fe3f8(&self, es: &EmitterSystem) -> f32 {
        // 0x001fe3f8
        match self.stack.last().and_then(|&i| es.get(i)) {
            Some(e) => e.cycles as f32 * EE_CYCLE, // FUN_0023c078 = u64 -> f32
            None => 0.0,
        }
    }

    /// FUN_001fe888(0x3dd040): play time of the dialogue line +0x200: the instance clock, or +0x24c
    /// (`elapsed`) while the clock is 0 and the instance is idle (finished / never started); 0 without a line.
    pub fn ee_sub_1fe888(&self, es: &EmitterSystem) -> f32 {
        // 0x001fe888
        let Some(e) = self.cur.and_then(|i| es.get(i)) else { return 0.0 };
        let t = self.line.as_ref().map_or(0, |l| l.cycles) as f32 * EE_CYCLE;
        if t == 0.0 && idle(e) {
            return self.elapsed;
        }
        t
    }

    /// FUN_001fe5b8(0x3dd040, bits, on): set / clear `bits` of +0x68.
    pub fn ee_sub_1fe5b8(&mut self, bits: u32, on: i64) {
        // 0x001fe5b8
        if on != 0 {
            self.f68 = (self.f68 as u32 | bits) as i32;
        } else {
            self.f68 = (self.f68 as u32 & !bits) as i32;
        }
    }

    /// FUN_001fe9c8(0x3dd040, mode): +0x104 = mode, FUN_00232250 (REPLACED: `SndCmd::OutputMode`) when it changes.
    pub fn ee_sub_1fe9c8(&mut self, snd: &mut Snd989, mode: i32) {
        // 0x001fe9c8
        if self.f104 != mode {
            self.f104 = mode;
            snd.cmds.push(SndCmd::OutputMode { mode });
        }
    }

    /// FUN_001fe9f8(vol, 0x3dd040): master volume +0x1bc = clamp01(vol); FUN_002321e8(0x10, ..) (REPLACED:
    /// `SndCmd::BusVolume { bus: 0x10 }`) when it changes at 1/1024 resolution.
    pub fn ee_sub_1fe9f8(&mut self, snd: &mut Snd989, vol: f32) {
        // 0x001fe9f8
        let v = clamp01_11d948(vol);
        if (self.master * 1024.0) as i32 != (v * 1024.0) as i32 {
            self.master = v;
            snd.cmds.push(SndCmd::BusVolume { bus: 0x10, vol: v });
        }
    }

    /// FUN_001fedf0(0x3dd040): master volume 1.0 and every matrix entry 1.0 except slot 1 (the options sliders).
    pub fn ee_sub_1fedf0(&mut self, es: &mut EmitterSystem, snd: &mut Snd989) {
        // 0x001fedf0
        self.ee_sub_1fe9f8(snd, 1.0);
        for slot in 0..8u64 {
            if slot != 1 {
                for bus in 0..4 {
                    self.ee_sub_1fea60(es, snd, 1.0, bus, slot);
                }
            }
        }
    }

    /// FUN_001fd700(0x3dd040): level teardown (from FUN_0020e068). REPLACED: thunk 0x232408 -> `SndCmd::StopAll`,
    /// SUKR cmd 10, the flush loop until FUN_00231c48 returns 0 -> one `flush_231c48`. Then the record tables
    /// +0xec..+0x100 and +0x138 are cleared (the bank handle +0xe8 stays), FUN_00156be0(0, 0, 0, +0x74),
    /// +0x200 / +0x228 are dropped, the stack +0x204/+0x208 and layer objects +0x22c are zeroed WITHOUT
    /// releasing (the pools are reset right after: FUN_0011cdb8(+0x1c / +0x34), FUN_0011a200(+0x4c) =
    /// the emitter pool and active list), +0x68 = 3, FUN_001fedf0. Not modelled: +0x14 = 0, +0x18 = -1,
    /// +0x1d0 / +0x5c / +0x60 = 0, +0x108 (12 bytes) = 0, FUN_00227788(0x4435f0).
    pub fn ee_sub_1fd700(&mut self, es: &mut EmitterSystem, snd: &mut Snd989, h: &mut dyn MusicHost) {
        // 0x001fd700
        snd.cmds.push(SndCmd::StopAll);
        snd.ee_sub_2330d0(10);
        snd.flush_231c48(es);
        snd.step_138 = 0;
        let bank = snd.table.bank;
        snd.table = super::emitter_tick::SoundTable { bank, ..Default::default() };
        h.ee_sub_156be0(false, [0.0; 3]);
        if let Some(c) = self.cur.take() {
            es.release(c);
        }
        self.line = None;
        self.stack.clear();
        if let Some(a) = self.amb.take() {
            es.release(a);
        }
        self.objs = [0; 6];
        self.f68 = 3;
        *es = EmitterSystem::default();
        self.ee_sub_1fedf0(es, snd);
    }

    /// AUD3: the level music starts (REPLACED FUN_001fe150 push of the level's KPV instance): a new
    /// instance on top of the +0x204 stack, flagged starting (+0x52) until the host's players run.
    pub fn start_track(&mut self, es: &mut EmitterSystem, sound: i32) -> EmitterId {
        let id = es.alloc_instance(sound);
        if let Some(e) = es.get_mut(id) {
            e.s52 = 1;
        }
        if self.stack.len() == 8 {
            let old = self.stack.remove(0);
            es.release(old);
        }
        self.stack.push(id);
        id
    }

    /// FUN_001fd830(0x3dd040): one manager step (see module doc).
    pub fn ee_sub_1fd830(&mut self, es: &mut EmitterSystem, snd: &mut Snd989, w: &dyn EmitterWorld, h: &mut dyn MusicHost, clk: MusicClock) {
        // 0x001fd830
        self.t1f8 = self.t1f0;
        self.t1f0 = clk.ticks;
        self.ee_sub_1fda48(snd, h);
        self.ee_sub_1fdba8(snd);
        let top = self.stack.last().copied();
        let insts = [top, self.amb, self.cur];
        if clk.game_state == 3 {
            if self.f58 != 0 {
                insts.into_iter().flatten().for_each(|i| es.ee_sub_1fb060(i));
                self.f58 = 0;
            }
        } else if self.f58 == 0 {
            insts.into_iter().flatten().for_each(|i| es.ee_sub_1fb018(i));
            self.f58 = 1;
        }
        let player = h.player().map(|p| p.1);
        let inst = top.and_then(|i| es.get(i)).map(|e| InstState { starting: e.s52 != 0 || e.flags & 1 != 0 });
        self.layers.ee_sub_1fdc08(player, inst, clk.dt_unscaled);
        // FUN_001fdec0: empty
        let ended = self.line_end;
        self.ee_sub_1fdec8(es, h, clk.dt_unscaled);
        if self.line_end != ended {
            // the FUN_001fe738 call at the end of FUN_001fdec8 (WP-X2)
            self.ee_sub_1fe738(es, h);
        }
        let target = if self.f68 == 3 && clk.dt == 0.0 { 0.0 } else { 1.0 }; // FUN_001fe5e0
        let v = FUN_0011d018(self.vol[2][0], target, clk.dt_unscaled, PAUSE_FADE, None);
        self.ee_sub_1fea60(es, snd, v, 0, 2);
        self.ee_sub_1fea60(es, snd, v, 1, 2);
        self.ee_sub_1fea60(es, snd, target, 2, 2);
        tick_all(es, snd, w);
        snd.flush_231c48(es);
    }
}

/// The FUN_001fd830 list walk: FUN_001fb8e0 on each emitter of 0x3dd08c, the next link read before the call
/// (emitters appended during the walk are visited too).
pub fn tick_all(es: &mut EmitterSystem, snd: &mut Snd989, w: &dyn EmitterWorld) {
    let mut cur = es.active.first().copied();
    while let Some(id) = cur {
        let next = es.active.iter().position(|&a| a == id).and_then(|p| es.active.get(p + 1).copied());
        es.ee_sub_1fb8e0(snd, w, id);
        cur = next;
    }
}
