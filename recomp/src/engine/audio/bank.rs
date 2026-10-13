//! 989snd sound bank ("SBlk" v3, the level sound bank at FUN_001fd0e8 +0xa8 descriptor).
//! File: u32 3, u32 2 (chunk count), {u32 off, u32 size} x 2: chunk0 = SBlk header/sounds/tones, chunk1 = SPU ADPCM.
//! Chunk0: "SBlk", u32 version(3), ... +0x16 u16 sounds, u16 grains, u16 vags; +0x1c u32 sound_off, grain_off,
//! vag_off, vag_data_size, ..., +0x34 grain_data_off.
//! Sound (12 bytes): i8 vol, i8 voice group, i16 pan, u8 grain count, u8 ?, u16 flags, u32 first grain (bytes).
//! Grain (8 bytes): u32 op = type<<24 | offset into grain data, i32 delay. Type 1 = tone (0x18 bytes,
//! +0x10 u32 = SPU byte offset into chunk1). Parsing/decoding replaces the IOP bank loader (snd_BankLoad*).

use super::adpcm::{self, Decoded};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sound {
    pub vol: i8,
    pub voice_group: i8,
    pub pan: i16,
    pub flags: u16,
    pub grains: Vec<Grain>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Grain {
    pub kind: u8,
    pub data_off: u32,
    pub delay: i32,
    pub tone: Option<Tone>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tone {
    pub raw: [u8; 0x18],
    pub vag_off: u32,
}

/// Tone record (0x18 bytes) as read by 989snd FUN_00004a44 / FUN_00019a58 / FUN_00006d2c (WP-G2, from code).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToneFields {
    /// +0 voice priority (FUN_00006d2c)
    pub priority: i8,
    /// +1 volume 0..127; -1..-4 local register, -5 random, < -5 global register
    pub vol: i8,
    /// +2 centre note (>= 0: sample authored at 44100 Hz; < 0: |note| at 48000 Hz, FUN_0001acc8); noise clock
    /// when flags & 8
    pub center_note: i8,
    /// +3 centre fine tune (1/128 semitone)
    pub center_fine: i8,
    /// +4 pan in degrees (same register/random encoding as vol)
    pub pan: i16,
    /// +6 not read by the key-on path
    pub unk6: u16,
    /// +8 / +9 pitch-bend range down / up (semitones, FUN_0001ad68)
    pub bend_down: i8,
    pub bend_up: i8,
    /// +0xa / +0xc SPU ADSR1 / ADSR2 (voice regs 0x300 / 0x400)
    pub adsr1: u16,
    pub adsr2: u16,
    /// +0xe: 1 = voice bit in DAT_0001f548, 2|4 = distance priority mode, 8 = noise voice, 0x10 = dry (no
    /// effect send), 0x20 = voice-alloc flag
    pub flags: u16,
    /// +0x10 SPU address (offset into chunk1 here; relocated by the bank loader on the IOP)
    pub spu_addr: u32,
    /// +0x14 not read by the key-on path
    pub unk14: u32,
}

impl Tone {
    pub fn fields(&self) -> ToneFields {
        let r = &self.raw;
        let h = |o: usize| u16::from_le_bytes([r[o], r[o + 1]]);
        let w = |o: usize| u32::from_le_bytes(r[o..o + 4].try_into().unwrap());
        ToneFields {
            priority: r[0] as i8,
            vol: r[1] as i8,
            center_note: r[2] as i8,
            center_fine: r[3] as i8,
            pan: h(4) as i16,
            unk6: h(6),
            bend_down: r[8] as i8,
            bend_up: r[9] as i8,
            adsr1: h(0xa),
            adsr2: h(0xc),
            flags: h(0xe),
            spu_addr: w(0x10),
            unk14: w(0x14),
        }
    }

    /// Playback rate of this tone at `note` (no bend): FUN_0001acc8 pitch -> Hz.
    pub fn rate_at(&self, note: u16, fine: i16) -> f32 {
        let f = self.fields();
        super::spu::pitch_to_rate(super::spu::tone_pitch(f.center_note, f.center_fine, note, fine))
    }
}

#[derive(Clone, Debug)]
pub struct Bank {
    pub sounds: Vec<Sound>,
    pub n_grains: u16,
    pub n_vags: u16,
    pub spu: Vec<u8>,
}

fn u32le(d: &[u8], o: usize) -> Option<u32> {
    Some(u32::from_le_bytes(d.get(o..o + 4)?.try_into().ok()?))
}
fn u16le(d: &[u8], o: usize) -> Option<u16> {
    Some(u16::from_le_bytes(d.get(o..o + 2)?.try_into().ok()?))
}

pub fn parse(d: &[u8]) -> Option<Bank> {
    if u32le(d, 0)? != 3 || u32le(d, 4)? != 2 {
        return None;
    }
    let (o0, s0) = (u32le(d, 8)? as usize, u32le(d, 12)? as usize);
    let (o1, s1) = (u32le(d, 16)? as usize, u32le(d, 20)? as usize);
    let c = d.get(o0..o0 + s0)?;
    if &c[..4] != b"SBlk" {
        return None;
    }
    let (ns, n_grains, n_vags) = (u16le(c, 0x16)?, u16le(c, 0x18)?, u16le(c, 0x1a)?);
    let (fs, fg, gd) = (u32le(c, 0x1c)? as usize, u32le(c, 0x20)? as usize, u32le(c, 0x34)? as usize);
    let mut sounds = Vec::with_capacity(ns as usize);
    for s in 0..ns as usize {
        let e = fs + 12 * s;
        let ng = *c.get(e + 4)? as usize;
        let first = u32le(c, e + 8)? as usize;
        let mut grains = Vec::with_capacity(ng);
        for g in 0..ng {
            let ge = fg + first + 8 * g;
            let op = u32le(c, ge)?;
            let kind = (op >> 24) as u8;
            let off = op & 0xff_ffff;
            let tone = if kind == 1 {
                let o = gd + off as usize;
                let raw: [u8; 0x18] = c.get(o..o + 0x18)?.try_into().ok()?;
                let vag_off = u32::from_le_bytes(raw[0x10..0x14].try_into().unwrap());
                Some(Tone { raw, vag_off })
            } else {
                None
            };
            grains.push(Grain { kind, data_off: off, delay: u32le(c, ge + 4)? as i32, tone });
        }
        sounds.push(Sound { vol: c[e] as i8, voice_group: c[e + 1] as i8, pan: u16le(c, e + 2)? as i16, flags: u16le(c, e + 6)?, grains });
    }
    Some(Bank { sounds, n_grains, n_vags, spu: d.get(o1..o1 + s1)?.to_vec() })
}

impl Bank {
    /// Decode the sample a tone points at.
    pub fn decode_tone(&self, t: &Tone) -> Option<Decoded> {
        Some(adpcm::decode(self.spu.get(t.vag_off as usize..)?))
    }
}
