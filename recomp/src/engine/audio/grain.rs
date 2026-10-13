//! WP-G2: 989snd SFX grain sequencer (the IOP "block sound handler") as a Rust equivalent.
//! Sources (989snd.irx): FUN_000043fc/FUN_00004000 (start), FUN_000047a4 (240 Hz tick), FUN_0000626c (run one
//! grain), grain handler table 0x1e838 (45 entries, index = grain op >> 24), FUN_00004a44 (tone key-on).
//! Handler layout used below: +0x0c sound vol, +0x10 handler vol, +0x12 pan, +0x14 pitch mod, +0x34 note (60),
//! +0x35 fine, +0x36 pitch bend, +0x38 vol modifier, +0x3a master vol (0x400 = unity), +0x3c/+0x3e pans,
//! +0x48 wait ticks, +0x4a grain index (-1 = done), +0x4c 4 local registers, +0x50/+0x52/+0x54 play-cycle state.
//! Registers: index >= 0 = local (handler +0x4c), < 0 = global table 0x1e818[~r] (the 989snd "music"/global
//! registers, also what RefOpSetMusicRegister would have written).
//! Replaced: voice allocation/SPU programming (FUN_00019750/FUN_00019a58) become `GrainEvent::KeyOn` for the
//! host mixer; child sounds / LFO / plugin messages are reported as events and not further interpreted.

use super::bank::{Bank, ToneFields};
use super::spu::{self, Adsr, PlaybackMode, R250};

/// 989snd tick rate (FUN_000078d0 returns 0xf0).
pub const TICK_HZ: u32 = 240;
pub const GLOBAL_REGS: usize = 32;

/// Grain types (index into table 0x1e838; names = 989snd grain kinds, handlers cited).
pub mod kind {
    pub const NULL: u8 = 0; // 0x4a3c
    pub const TONE: u8 = 1; // 0x4a44
    pub const XREF_ID: u8 = 2; // 0x4f3c no-op
    pub const XREF_NUM: u8 = 3; // 0x4f44 no-op
    pub const LFO_SETTINGS: u8 = 4; // 0x4f4c
    pub const START_CHILD: u8 = 5; // 0x5e08
    pub const STOP_CHILD: u8 = 6; // 0x6118
    pub const PLUGIN_MESSAGE: u8 = 7; // 0x5bbc
    pub const BRANCH: u8 = 8; // 0x5c14
    pub const TONE2: u8 = 9; // 0x4a44
    // 10..=19: 0x623c error 0x5f, stops the sound
    pub const CONTROL_NULL: u8 = 20; // 0x5074
    pub const LOOP_START: u8 = 21; // 0x507c
    pub const LOOP_END: u8 = 22; // 0x5084
    pub const LOOP_CONTINUE: u8 = 23; // 0x5134
    pub const STOP: u8 = 24; // 0x51d0
    pub const RAND_PLAY: u8 = 25; // 0x521c
    pub const RAND_DELAY: u8 = 26; // 0x532c
    pub const RAND_PB: u8 = 27; // 0x5378
    pub const PB: u8 = 28; // 0x5440
    pub const ADD_PB: u8 = 29; // 0x54b8
    pub const SET_REGISTER: u8 = 30; // 0x553c
    pub const SET_REGISTER_RAND: u8 = 31; // 0x5578
    pub const INC_REGISTER: u8 = 32; // 0x560c
    pub const DEC_REGISTER: u8 = 33; // 0x567c
    pub const TEST_REGISTER: u8 = 34; // 0x56ec
    pub const MARKER: u8 = 35; // 0x57b0
    pub const GOTO_MARKER: u8 = 36; // 0x57b8
    pub const GOTO_RANDOM_MARKER: u8 = 37; // 0x5860
    pub const WAIT_FOR_ALL_VOICES: u8 = 38; // 0x595c
    pub const PLAY_CYCLE: u8 = 39; // 0x5998
    pub const ADD_REGISTER: u8 = 40; // 0x5a64
    pub const KEY_OFF_VOICES: u8 = 41; // 0x5ac8
    pub const KILL_VOICES: u8 = 42; // 0x5b00
    pub const ON_STOP_MARKER: u8 = 43; // 0x5b38
    pub const COPY_REGISTER: u8 = 44; // 0x5b58
}

/// Something the host mixer must do (replaces the SPU/voice side effects of the grain handlers).
#[derive(Clone, Debug, PartialEq)]
pub enum GrainEvent {
    /// FUN_00004a44 + FUN_00019a58: start a voice
    KeyOn(KeyOn),
    /// KEY_OFF_VOICES: release all voices of this handler
    KeyOffVoices,
    /// KILL_VOICES: stop all voices immediately
    KillVoices,
    /// pitch bend changed (FUN_000067d4) -> host re-pitches the handler's voices
    PitchBend(i16),
    /// LFO_SETTINGS grain (raw 16+ bytes at the grain data offset)
    Lfo { data_off: u32 },
    StartChild { data_off: u32 },
    StopChild { data_off: u32 },
    PluginMessage { data_off: u32 },
    Branch { data_off: u32 },
}

#[derive(Clone, Debug, PartialEq)]
pub struct KeyOn {
    pub tone: ToneFields,
    pub tone_index: usize,
    /// SPU pitch register (0x1000 = 48 kHz)
    pub pitch: u16,
    /// SPU volume registers after FUN_0001b628 + FUN_0001bd08 (0..0x3fff, sign = phase inversion)
    pub vol: [i16; 2],
    pub adsr: (u16, u16),
    pub noise: bool,
    pub voice_group: i8,
    pub priority: i32,
}

impl KeyOn {
    pub fn envelope(&self) -> Adsr {
        Adsr::new(self.adsr.0, self.adsr.1)
    }
    pub fn rate(&self) -> f32 {
        spu::pitch_to_rate(self.pitch)
    }
}

/// Shared 989snd state the handlers read (globals in the IOP module).
#[derive(Clone, Debug)]
pub struct SndGlobals {
    pub regs: [i8; GLOBAL_REGS],
    pub rng: R250,
    pub mode: PlaybackMode,
    pub group_vol: [i32; 16],
    pub duck: [u32; 16],
}

impl Default for SndGlobals {
    fn default() -> Self {
        Self { regs: [0; GLOBAL_REGS], rng: R250::default(), mode: PlaybackMode::default(), group_vol: [0x400; 16], duck: [0x10000; 16] }
    }
}

/// One playing bank sound (989snd BlockSoundHandler).
#[derive(Clone, Debug)]
pub struct BlockHandler {
    pub sound: usize,
    pub sound_vol: i16,   // +0x0c
    pub handler_vol: i16, // +0x10
    pub pan: i16,         // +0x12
    pub pm: i16,          // +0x14
    pub flags: u8,        // +0x16 (0x20 = no new tones, 0x40 = loop seen)
    pub voice_group: i8,  // +0x17
    pub note: i8,         // +0x34
    pub fine: i8,         // +0x35
    pub bend: i16,        // +0x36
    pub vol_mod: i16,     // +0x38
    pub master_vol: i16,  // +0x3a
    pub pan_a: i16,       // +0x3c
    pub pan_b: i16,       // +0x3e
    pub wait: i16,        // +0x48
    pub index: i16,       // +0x4a
    pub regs: [i8; 4],    // +0x4c
    pub cycle_active: bool,
    pub cycle_count: i16,
    pub cycle_skip: i16,
    /// host sets this to the number of live voices (handler +0x18/+0x1c voice bitmasks)
    pub live_voices: u32,
}

impl BlockHandler {
    /// FUN_00004000 handler init for a sound: vol/pan from the sound record, master 0x400, note 60.
    pub fn new(bank: &Bank, sound: usize, vol: i16, pan: i16) -> Option<Self> {
        let s = bank.sounds.get(sound)?;
        Some(Self {
            sound,
            sound_vol: s.vol as i16,
            handler_vol: s.vol as i16,
            pan: s.pan,
            pm: 0,
            flags: 0,
            voice_group: s.voice_group,
            note: 0x3c,
            fine: 0,
            bend: 0,
            vol_mod: 0,
            master_vol: if vol < 0 { -vol } else { vol },
            pan_a: 0,
            pan_b: if pan == -1 { s.pan } else { pan },
            wait: s.grains.first().map_or(0, |g| g.delay as i16),
            index: if s.grains.is_empty() { -1 } else { 0 },
            regs: [0; 4],
            cycle_active: false,
            cycle_count: 0,
            cycle_skip: 0,
            live_voices: 0,
        })
    }

    pub fn done(&self) -> bool {
        self.index == -1 && self.live_voices == 0
    }

    fn reg(&self, g: &SndGlobals, r: i8) -> i8 {
        if r >= 0 { self.regs.get(r as usize).copied().unwrap_or(0) } else { g.regs[(!r) as usize % GLOBAL_REGS] }
    }
    fn reg_mut<'a>(&'a mut self, g: &'a mut SndGlobals, r: i8) -> &'a mut i8 {
        if r >= 0 { &mut self.regs[r as usize & 3] } else { &mut g.regs[(!r) as usize % GLOBAL_REGS] }
    }

    /// Tone volume / pan source (FUN_00004a44): >= 0 literal, -1..-4 local register, -5 random,
    /// < -5 global register (-6 = global 0).
    fn src(&self, g: &mut SndGlobals, v: i32) -> Result<i32, i32> {
        match v {
            0.. => Ok(v),
            -4..=-1 => Err(self.regs[(!v) as usize] as i32),
            -5 => Ok(-5),
            _ => Err(g.regs[((-v - 6) as usize) % GLOBAL_REGS] as i32),
        }
    }

    /// FUN_00004a44: compute the key-on of tone grain `op_off`.
    fn key_on(&mut self, bank: &Bank, g: &mut SndGlobals, tone_index: usize, t: &ToneFields) -> Option<KeyOn> {
        if self.flags & 0x20 != 0 {
            return None;
        }
        self.handler_vol = (self.vol_mod + 0x7f).clamp(0, 0x7f);
        let mut p = self.pan_b as i32 + self.pan_a as i32;
        while p > 0x167 {
            p -= 0x168;
        }
        while p < 0 {
            p += 0x168;
        }
        self.pan = p as i16;
        let tvol = match self.src(g, t.vol as i32) {
            Ok(-5) => (g.rng.next() & 0x7f) as i32,
            Ok(v) => v,
            Err(r) => r,
        }
        .max(0);
        let mut tpan = match self.src(g, t.pan as i32) {
            Ok(-5) => (g.rng.next() as u32 % 0x168) as i32,
            Ok(v) => v,
            Err(r) => r * 0x168 / 0x7f,
        };
        while tpan < 0 {
            tpan += 0x168;
        }
        while tpan > 0x167 {
            tpan -= 0x168;
        }
        let s = &bank.sounds[self.sound];
        let tpan = if s.flags & 4 != 0 { distance_pan(tpan, self.master_vol as i32) } else { tpan };
        let v = spu::make_volume(
            g.mode,
            self.master_vol as i32 * 0x7f >> 10,
            self.sound_vol as i32,
            0,
            self.handler_vol as i32,
            self.pan as i32,
            tvol,
            tpan,
            [0, 0],
            10,
        );
        let gl = |x: i16| (spu::group_volume(x, self.voice_group as i32, &g.group_vol, &g.duck) as u32 as u16 >> 1) as i16;
        let (n, f) = spu::note_with_bend(t.bend_down, t.bend_up, self.bend as i32, self.pm as i32, self.note as i32, self.fine as i32);
        Some(KeyOn {
            tone: t.clone(),
            tone_index,
            pitch: spu::tone_pitch(t.center_note, t.center_fine, n as u16, f as i16),
            vol: [gl(v[0]), gl(v[1])],
            adsr: (t.adsr1, t.adsr2),
            noise: t.flags & 8 != 0,
            voice_group: self.voice_group,
            priority: priority(self.master_vol as i32, t),
        })
    }

    /// FUN_000047a4: one 240 Hz tick. Returns true when the handler finished (grains done, no voices).
    pub fn tick(&mut self, bank: &mut Bank, g: &mut SndGlobals, out: &mut Vec<GrainEvent>) -> bool {
        if self.index == -1 {
            return self.live_voices == 0;
        }
        self.wait -= 1;
        self.flags &= !0x40;
        if self.wait < 1 {
            loop {
                if self.step(bank, g, out) != 0 {
                    return true;
                }
                if self.index == -1 || self.wait != 0 {
                    break;
                }
            }
        }
        false
    }

    /// FUN_00004000 start: run grains until the first wait.
    pub fn start(&mut self, bank: &mut Bank, g: &mut SndGlobals, out: &mut Vec<GrainEvent>) -> bool {
        self.flags &= !0x40;
        let mut r = 0;
        if self.index != -1 && self.wait == 0 {
            loop {
                r = self.step(bank, g, out);
                if r != 0 || self.index == -1 || self.wait != 0 {
                    break;
                }
            }
        }
        r != 0 || (self.wait == 0 && self.index == -1 && self.live_voices == 0)
    }

    /// FUN_0000626c: run grain `index`; nonzero = handler finished/stopped.
    pub fn step(&mut self, bank: &mut Bank, g: &mut SndGlobals, out: &mut Vec<GrainEvent>) -> i32 {
        let si = self.sound;
        let n = bank.sounds[si].grains.len() as i16;
        if self.index < 0 || self.index >= n {
            // out-of-range jump (bad data): the original would read past the grain array; stop instead
            self.index = -1;
            return 1;
        }
        let gi = self.index as usize;
        let (k, off) = {
            let gr = &bank.sounds[si].grains[gi];
            (gr.kind, gr.data_off)
        };
        let a = [off as u8 as i8, (off >> 8) as u8 as i8, (off >> 16) as u8 as i8];
        let r: i32 = match k {
            kind::TONE | kind::TONE2 => {
                if let Some(t) = bank.sounds[si].grains[gi].tone.as_ref().map(|t| t.fields()) {
                    if let Some(kon) = self.key_on(bank, g, gi, &t) {
                        out.push(GrainEvent::KeyOn(kon));
                    }
                }
                0
            }
            kind::LFO_SETTINGS => {
                out.push(GrainEvent::Lfo { data_off: off });
                0
            }
            kind::START_CHILD => {
                out.push(GrainEvent::StartChild { data_off: off });
                0
            }
            kind::STOP_CHILD => {
                out.push(GrainEvent::StopChild { data_off: off });
                0
            }
            kind::PLUGIN_MESSAGE => {
                out.push(GrainEvent::PluginMessage { data_off: off });
                0
            }
            kind::BRANCH => {
                out.push(GrainEvent::Branch { data_off: off });
                0
            }
            10..=19 => -1,
            kind::LOOP_END => {
                // search back for LOOP_START; second pass in the same tick returns 1 (one tick wait)
                let mut r = -1;
                let mut i = self.index as i32 - 1;
                while i >= 0 {
                    let j = i;
                    i -= 1;
                    if bank.sounds[si].grains[j as usize].kind == kind::LOOP_START {
                        self.index = i as i16;
                        r = if self.flags & 0x40 == 0 {
                            self.flags |= 0x40;
                            0
                        } else {
                            1
                        };
                        break;
                    }
                }
                r
            }
            kind::LOOP_CONTINUE => {
                let mut r = -1;
                for i in self.index as usize + 1..n as usize {
                    if bank.sounds[si].grains[i].kind == kind::LOOP_END {
                        self.index = i as i16;
                        r = 0;
                        break;
                    }
                }
                r
            }
            kind::STOP => -1,
            kind::RAND_PLAY | kind::PLAY_CYCLE => {
                if self.cycle_active {
                    -1
                } else {
                    let (cnt, step, last) = (a[0] as i32, a[1] as i32, a[2] as i32);
                    let pick = if k == kind::RAND_PLAY {
                        let mut p = g.rng.next() as i32 % cnt.max(1);
                        if p == last {
                            p += 1;
                            if p == cnt {
                                p = 0;
                            }
                        }
                        p
                    } else {
                        last
                    };
                    let next_last = if k == kind::RAND_PLAY { pick } else if last + 1 == cnt { 0 } else { last + 1 };
                    // quirk: the state byte lives in the bank's grain op (shared by all instances of the sound)
                    let gr = &mut bank.sounds[si].grains[gi];
                    gr.data_off = (gr.data_off & 0x00ffff) | ((next_last as u8 as u32) << 16);
                    self.index += (pick * step) as i16;
                    self.cycle_count = (step + 1) as i16;
                    self.cycle_active = true;
                    self.cycle_skip = ((cnt - 1 - pick) * step) as i16;
                    0
                }
            }
            kind::RAND_DELAY => (g.rng.next() as u32 % ((off & 0xff_ffff) + 1)) as i32,
            kind::RAND_PB => {
                let r = g.rng.next() as i32;
                let pb = ((((r % 0x7fff) * 0xffff) / 0x7fff - 0x8000) * a[0] as i32 / 100) as i16;
                self.bend = pb;
                out.push(GrainEvent::PitchBend(pb));
                0
            }
            kind::PB => {
                let v = a[0] as i32;
                let pb = if v < 0 { -((-(v << 15)) >> 7) } else { (v * 0x7fff) / 0x7f } as i16;
                self.bend = pb;
                out.push(GrainEvent::PitchBend(pb));
                0
            }
            kind::ADD_PB => {
                let pb = (self.bend as i32 + a[0] as i32 * 0x7fff / 0x7f).clamp(-0x8000, 0x7fff) as i16;
                self.bend = pb;
                out.push(GrainEvent::PitchBend(pb));
                0
            }
            kind::SET_REGISTER => {
                *self.reg_mut(g, a[0]) = a[1];
                0
            }
            kind::SET_REGISTER_RAND => {
                let span = a[2] as i32 - a[1] as i32 + 1;
                let v = (g.rng.next() as i32 % if span == 0 { 1 } else { span }) as i8 as i32 + a[1] as i32;
                *self.reg_mut(g, a[0]) = v as i8;
                0
            }
            kind::INC_REGISTER => {
                let r = self.reg_mut(g, a[0]);
                *r = (*r as i32 + 1).min(0x7f) as i8;
                0
            }
            kind::DEC_REGISTER => {
                let r = self.reg_mut(g, a[0]);
                *r = (*r as i32 - 1).max(-0x80) as i8;
                0
            }
            kind::ADD_REGISTER => {
                let r = self.reg_mut(g, a[1]);
                *r = (*r as i32 + a[0] as i32).clamp(-0x80, 0x7f) as i8;
                0
            }
            kind::COPY_REGISTER => {
                let v = self.reg(g, a[0]);
                *self.reg_mut(g, a[1]) = v;
                0
            }
            kind::TEST_REGISTER => {
                let v = self.reg(g, a[0]);
                let ok = match a[1] {
                    0 => v < a[2],
                    1 => v == a[2],
                    2 => v > a[2],
                    _ => true,
                };
                if !ok {
                    self.index += 1; // skip the next grain
                }
                0
            }
            kind::GOTO_MARKER | kind::GOTO_RANDOM_MARKER => {
                let m = if k == kind::GOTO_MARKER {
                    a[0] as i32
                } else {
                    let span = a[1] as i32 - a[0] as i32 + 1;
                    g.rng.next() as i32 % if span == 0 { 1 } else { span } + a[0] as i32
                };
                let gs = &bank.sounds[si].grains;
                match gs.iter().position(|x| x.kind == kind::MARKER && (x.data_off as u8 as i8 as i32) == m) {
                    Some(p) => {
                        self.index = p as i16 - 1;
                        0
                    }
                    None => -1,
                }
            }
            kind::WAIT_FOR_ALL_VOICES => {
                if self.live_voices != 0 {
                    self.index -= 1;
                    1
                } else {
                    0
                }
            }
            kind::KEY_OFF_VOICES => {
                out.push(GrainEvent::KeyOffVoices);
                0
            }
            kind::KILL_VOICES => {
                out.push(GrainEvent::KillVoices);
                0
            }
            kind::ON_STOP_MARKER => {
                self.index = n - 1;
                0
            }
            _ => 0, // NULL, XREF_*, CONTROL_NULL, LOOP_START, MARKER
        };
        if r < 0 {
            return 1;
        }
        if self.cycle_active {
            let c = self.cycle_count;
            self.cycle_count -= 1;
            if c == 1 {
                self.cycle_active = false;
                self.index += self.cycle_skip;
            }
        }
        self.index += 1;
        if self.index > n - 1 {
            self.index = -1;
            return (self.live_voices == 0) as i32;
        }
        self.wait = bank.sounds[si].grains[self.index as usize].delay as i16 + r as i16;
        0
    }
}

/// FUN_00006c98: distance-scaled pan for sounds with flag 4 (vol 0xab..0x355 narrows the pan toward centre).
pub fn distance_pan(pan: i32, vol: i32) -> i32 {
    if vol >= 0x356 {
        pan
    } else if vol < 0xab {
        0
    } else if pan < 0xb5 {
        pan * (vol - 0xaa) / 0x2ac
    } else {
        0x168 - ((0x168 - pan) * (vol - 0xaa)) / 0x2ac
    }
}

/// FUN_00006d2c: voice priority (tone +0, lowered with distance when tone flags bits 1-2 are set).
pub fn priority(vol: i32, t: &ToneFields) -> i32 {
    let p = t.priority as i32;
    let mode = (t.flags >> 1) & 3;
    if t.flags & 6 == 0 {
        return p;
    }
    let d = match mode {
        1 => p / 5,
        2 => p >> 1,
        _ => p,
    };
    if d != 0 {
        if vol > 0x3b6 {
            return p;
        }
        return (p - (d - d * vol / 0x3b6)) as i8 as i32;
    }
    p as i8 as i32
}
