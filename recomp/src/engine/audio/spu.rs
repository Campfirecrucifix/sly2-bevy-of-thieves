//! WP-G2: SPU2 / 989snd voice maths as Rust equivalents (IOP driver code: REPLACED, but the arithmetic is kept
//! bit-exact where it decides what the player hears: pitch, volume/pan, ADSR).
//! Sources: libsd.irx sceSdNote2Pitch (export 13 = 0x38c0, tables 0x51a0/0x51b8), 989snd.irx FUN_0001acc8
//! (tone pitch), FUN_0001ad68 (note + pitch bend), FUN_0001b628 (volume/pan), FUN_0001bd08 (voice-group master
//! volume), FUN_00019a58 (voice key-on: which tone fields go to which SPU register), R250 FUN_0000c440/FUN_0000c32c.

#[path = "spu_tables.rs"]
mod tables;
pub use tables::{FINE_TABLE, NOTE_TABLE, PAN_TABLE};

/// SPU2 output rate; pitch register 0x1000 = one input sample per output sample.
pub const SPU_OUTPUT_RATE: u32 = 48000;

/// libsd sceSdNote2Pitch (libsd.irx 0x38c0, not decompiled by Ghidra: ported from the MIPS listing).
/// Quirk kept: the "negative fine" path can index before the tables in the original (fine < -128); such reads
/// return 0 here instead of reading neighbouring IOP memory.
pub fn sce_sd_note2pitch(center_note: u16, center_fine: u16, note: u16, fine: i16) -> u16 {
    let t0 = center_note as i32;
    let mut a3 = fine as i32 + center_fine as i32;
    let fine2 = (if a3 < 0 { a3 + 0x7f } else { a3 }) >> 7;
    let n = ((note as i32 + fine2 - t0) as i16) as i32;
    let hi = ((n as i64 * 0x2aaa_aaab) >> 32) as i32;
    let oct = (hi >> 1) - (n >> 31); // n / 12 (truncating)
    a3 -= fine2 << 7;
    let off1 = n - oct * 12;
    let mut shift = oct - 2;
    let mut a0 = off1;
    let o16 = off1 as i16;
    if o16 < 0 || (o16 == 0 && a3 < 0) {
        a0 = off1 + 12;
        shift = oct - 3;
    }
    if a3 < 0 {
        a0 = a0 - 1 + fine2;
        a3 += (fine2 + 1) << 7;
    }
    let i1 = (a0 as i16) as i32;
    let t1 = if (0..12).contains(&i1) { NOTE_TABLE[i1 as usize] } else { 0 } as i32;
    let t2 = if (0..128).contains(&a3) { FINE_TABLE[a3 as usize] } else { 0 } as i32;
    let mut r = t1.wrapping_mul(t2) >> 16;
    let s = shift as i16;
    if s < 0 {
        let sh = (-(s as i32)) as u32;
        r = ((r.wrapping_add(1 << ((sh - 1) & 31))) as u32 >> (sh & 31)) as i32;
    }
    r as u16
}

/// FUN_0001acc8: tone pitch register. A tone whose centre note (tone+2) is >= 0 was authored at 44100 Hz:
/// the pitch is scaled by 44100/48000. A negative centre note means |note| at native 48000 Hz.
pub fn tone_pitch(center_note: i8, center_fine: i8, note: u16, fine: i16) -> u16 {
    let c = center_note as i32;
    let p = sce_sd_note2pitch(c.unsigned_abs() as u16 & 0xffff, center_fine as i16 as u16, note, fine) as u32;
    (if c >= 0 { p * 0xac44 / 48000 } else { p }) as u16
}

/// FUN_0001ad68: (note, fine) after pitch bend. `bend` = handler +0x36 pitch-bend (-0x8000..0x7fff),
/// `pm` = extra 1/128-semitone offset (handler +0x14), bend ranges = tone +8 (down) / +9 (up) in semitones.
pub fn note_with_bend(bend_down: i8, bend_up: i8, bend: i32, pm: i32, note: i32, fine: i32) -> (i32, i32) {
    let b = if bend < 0 {
        let v = bend_down as i32 * bend * 0x80;
        (if v < 0 { v + 0x7fff } else { v }) >> 15
    } else {
        bend_up as i32 * bend * 0x80 / 0x7fff
    };
    let t = note * 0x80 + fine + pm + b;
    let q = (if t < 0 { t + 0x7f } else { t }) >> 7;
    (q, t - q * 0x80)
}

/// SPU pitch register -> playback sample rate in Hz.
pub fn pitch_to_rate(pitch: u16) -> f32 {
    SPU_OUTPUT_RATE as f32 * pitch as f32 / 4096.0
}

/// 989snd output mode (DAT_0001f518, set by snd_SetPlaybackMode FUN_0001b23c; 0 at boot).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PlaybackMode {
    /// 0: stereo with Pro-Logic style phase inversion for rear angles (boot value)
    #[default]
    Surround,
    /// 1: mono (left = right = volume)
    Mono,
    /// 2: plain stereo (no inversion)
    Stereo,
}

/// FUN_0001b628 snd_MakeVolume: volumes are 0..127 except `vol_master`; `flags` bits 0..3 square
/// (v*v/127) vol_master/vol_sound/vol_handler/vol_tone. Pans in degrees. `prev` = previous [L,R] register
/// pair (the inversion logic looks at its signs, like the original which reads/writes in place).
/// Returns the new [L,R] (0x3fff scale, before voice-group master FUN_0001bd08).
#[allow(clippy::too_many_arguments)]
pub fn make_volume(
    mode: PlaybackMode,
    mut vol_master: i32,
    mut vol_sound: i32,
    pan_a: i32,
    mut vol_handler: i32,
    pan_b: i32,
    mut vol_tone: i32,
    pan_c: i32,
    prev: [i16; 2],
    flags: u32,
) -> [i16; 2] {
    if flags & 1 != 0 {
        vol_master = vol_master * vol_master / 0x7f;
    }
    if flags & 2 != 0 {
        vol_sound = vol_sound * vol_sound / 0x7f;
    }
    if flags & 4 != 0 {
        vol_handler = vol_handler * vol_handler / 0x7f;
    }
    if flags & 8 != 0 {
        vol_tone = vol_tone * vol_tone / 0x7f;
    }
    let v = (((vol_sound * 0x102 * vol_tone) / 0x7f * vol_handler) / 0x7f * vol_master) / 0x7f;
    if v == 0 {
        return [0, 0];
    }
    if mode == PlaybackMode::Mono {
        return [v as i16, v as i16];
    }
    let mut a = pan_a + pan_c + pan_b;
    while a > 0x167 {
        a -= 0x168;
    }
    while a < 0 {
        a += 0x168;
    }
    let i = if a < 0x10e { a + 0x5a } else { a - 0x10e } as usize;
    let (l0, r0) = (prev[0], prev[1]);
    if i < 0xb4 {
        let l = v * PAN_TABLE[i][0] as i32 / 0x3fff;
        let r = v * PAN_TABLE[i][1] as i32 / 0x3fff;
        if l0 < 0 && r0 < 0 {
            return [-l as i16, -r as i16];
        }
        if (l0 < 0) != (r0 < 0) {
            // exactly one previous channel inverted: sign of the larger previous magnitude (0x0001b8c4 path)
            let (pl, pr) = (l0 as i32, r0 as i32);
            let s = if pr.abs() < pl.abs() { pl / pl.abs() } else { pr / pr.abs() };
            return [(l * s) as i16, (r * s) as i16];
        }
        [l as i16, r as i16]
    } else {
        let mut r = v * PAN_TABLE[i - 0xb4][0] as i32 / 0x3fff;
        let mut l = v * PAN_TABLE[i - 0xb4][1] as i32 / 0x3fff;
        if mode != PlaybackMode::Stereo {
            // rear half: one channel phase-inverted (the smaller one, or keep the previously inverted side)
            let invert_r = if l0 >= 0 && r0 >= 0 { r < l } else if l0 >= 0 { true } else if r0 < 0 { l < r } else { false };
            if invert_r {
                r = -r;
            } else {
                l = -l;
            }
        }
        [l as i16, r as i16]
    }
}

/// FUN_0001bd08: apply voice-group master volume (`group_vol` 0..0x400, DAT_0001f418) and duck
/// (`duck` 0x10000 = none, DAT_0001f498). Groups > 14 are unaffected. The SPU register gets the result >> 1.
pub fn group_volume(vol: i16, group: i32, group_vol: &[i32; 16], duck: &[u32; 16]) -> i32 {
    let mut v = vol as i32;
    if group > 0xe {
        return v;
    }
    if v > 0x7ffe {
        v = 0x7ffe;
    }
    let g = group.max(0) as usize;
    let mut d = duck[g];
    if d.wrapping_sub(1) < 0xffff {
        d = d * d >> 16;
    }
    let mut gv = group_vol[g].wrapping_mul(group_vol[g]);
    if gv < 0 {
        gv += 0x3ff;
    }
    let m = ((gv >> 10) as u32).wrapping_mul(d) >> 16;
    let mut r = v.wrapping_mul(m as i32);
    if r < 0 {
        r += 0x3ff;
    }
    (r << 6) >> 16
}

/// SPU2 ADSR envelope (hardware semantics, psx-spx/SPU2 docs). ADSR1 = tone +0xa, ADSR2 = tone +0xc
/// (FUN_00019a58 writes them to voice registers 0x300/0x400).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdsrPhase {
    Attack,
    Decay,
    Sustain,
    Release,
    Off,
}

#[derive(Clone, Copy, Debug)]
pub struct Adsr {
    pub adsr1: u16,
    pub adsr2: u16,
    pub phase: AdsrPhase,
    pub level: i32,
    wait: u32,
}

impl Adsr {
    pub fn new(adsr1: u16, adsr2: u16) -> Self {
        Self { adsr1, adsr2, phase: AdsrPhase::Attack, level: 0, wait: 0 }
    }

    pub fn sustain_level(&self) -> i32 {
        ((self.adsr1 as i32 & 0xf) + 1) * 0x800
    }

    /// (exponential, decreasing, shift, step) of the current phase
    fn rate(&self) -> (bool, bool, u32, i32) {
        let (a1, a2) = (self.adsr1 as u32, self.adsr2 as u32);
        match self.phase {
            AdsrPhase::Attack => (a1 & 0x8000 != 0, false, (a1 >> 10) & 0x1f, 7 - ((a1 >> 8) & 3) as i32),
            AdsrPhase::Decay => (true, true, ((a1 >> 4) & 0xf) << 2, -8),
            AdsrPhase::Sustain => {
                let dec = a2 & 0x4000 != 0;
                let s = ((a2 >> 6) & 3) as i32;
                (a2 & 0x8000 != 0, dec, (a2 >> 8) & 0x1f, if dec { -8 + s } else { 7 - s })
            }
            AdsrPhase::Release => (a2 & 0x20 != 0, true, (a2 & 0x1f) << 2, -8),
            AdsrPhase::Off => (false, true, 0, 0),
        }
    }

    pub fn key_off(&mut self) {
        if self.phase != AdsrPhase::Off {
            self.phase = AdsrPhase::Release;
            self.wait = 0;
        }
    }

    /// Advance one output sample (48 kHz); returns the envelope level 0..0x7fff.
    pub fn tick(&mut self) -> i32 {
        if self.phase == AdsrPhase::Off {
            return 0;
        }
        if self.wait > 0 {
            self.wait -= 1;
        }
        if self.wait == 0 {
            let (exp, dec, shift, step0) = self.rate();
            let mut cycles = 1u32 << shift.saturating_sub(11);
            let mut step = step0 << 11u32.saturating_sub(shift);
            if exp && !dec && self.level > 0x6000 {
                cycles *= 4;
            }
            if exp && dec {
                step = step * self.level >> 15;
            }
            self.level = (self.level + step).clamp(0, 0x7fff);
            self.wait = cycles;
            match self.phase {
                AdsrPhase::Attack if self.level >= 0x7fff => self.phase = AdsrPhase::Decay,
                AdsrPhase::Decay if self.level <= self.sustain_level() => self.phase = AdsrPhase::Sustain,
                AdsrPhase::Release if self.level == 0 => self.phase = AdsrPhase::Off,
                _ => {}
            }
        }
        self.level
    }
}

/// 989snd random generator: R250 (FUN_0000c440), seeded by FUN_0000c32c from the LCG FUN_0000c4a8.
#[derive(Clone, Debug)]
pub struct R250 {
    table: [u16; 250],
    index: usize,
}

impl R250 {
    /// FUN_0000c32c(seed) (FUN_0000c4d4 stores the LCG seed DAT_0001ebe0).
    pub fn new(seed: u32) -> Self {
        let mut lcg = seed;
        let mut next = || {
            lcg = lcg.wrapping_mul(0x15a4e35).wrapping_add(1);
            (lcg >> 16) & 0x7fff
        };
        let mut table = [0u16; 250];
        for t in table.iter_mut() {
            *t = next() as u16;
        }
        for t in table.iter_mut() {
            if next() > 0x4000 {
                *t |= 0x8000;
            }
        }
        let (mut bit, mut mask) = (0x8000u32, 0xffffu32);
        for k in 0..16 {
            let i = 3 + 11 * k;
            table[i] = (bit | (mask & table[i] as u32)) as u16;
            bit >>= 1;
            mask >>= 1;
        }
        Self { table, index: 0 }
    }

    /// FUN_0000c440
    pub fn next(&mut self) -> u16 {
        let i = self.index;
        let j = if i > 0x92 { i - 0x93 } else { i + 0x67 };
        let v = self.table[i] ^ self.table[j];
        self.table[i] = v;
        self.index = if i < 0xf9 { i + 1 } else { 0 };
        v
    }
}

impl Default for R250 {
    fn default() -> Self {
        Self::new(1)
    }
}
