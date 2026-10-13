//! WP-G: audio. See ENGINE_PORT.md.
//! The 989snd IOP driver (banks in SPU RAM, voice allocation, VAG streaming, RPC) is REPLACED: files are read
//! from the ISO/mods, decoded to PCM here, and played by the host (bevy_audio under `game`).
//! Disc sound files are reached only through the level block FUN_001fd0e8 (WP-A `Block3dd040`):
//!   +0xa8 desc   -> SBlk sound bank ([`bank`])
//!   +0xf4 desc2  -> " KPV" interleaved music stream ([`vpk`])
//!   +0xec recs   -> "VAGp" dialogue/one-shot files ([`vagp`])
//!   +0xf0 recs48 -> " KPV" streams (size 0 = unused slot pointing at the movie LSN)

pub mod adpcm;
pub mod bank;
pub mod emitter;
pub mod emitter_tick;
#[cfg(feature = "game")]
pub mod bevy_out;
pub mod grain;
pub mod music;
pub mod music_layers;
pub mod music_mgr;
pub mod mods;
pub mod spu;
pub mod vagp;
pub mod vpk;

use crate::engine::disc::descriptor::Descriptor;
use crate::engine::disc::iso::SectorSource;
use crate::engine::disc::level_file::Block3dd040;
use bevy_ecs::prelude::Resource;
use std::io;

/// Kind of a sound file referenced by a level.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SoundFileKind {
    Bank,
    MusicStream,
    Vag,
}

/// Every sound file referenced by FUN_001fd0e8's block (deduplicated by LSN).
pub fn level_sound_files(b: &Block3dd040) -> Vec<(SoundFileKind, Descriptor)> {
    let mut v = vec![(SoundFileKind::Bank, b.desc.clone())];
    if let Some(d) = &b.desc2 {
        v.push((SoundFileKind::MusicStream, d.clone()));
    }
    v.extend(b.recs.iter().map(|r| (SoundFileKind::Vag, r.desc.clone())));
    v.extend(b.recs48.iter().map(|(_, d)| (SoundFileKind::MusicStream, Descriptor(*d))));
    v.retain(|(_, d)| d.size() > 0);
    v
}

pub fn read_descriptor(disc: &dyn SectorSource, d: &Descriptor) -> io::Result<Vec<u8>> {
    disc.read_bytes(d.lsn(), d.size() as usize)
}

/// Mono/multi PCM ready for playback.
#[derive(Clone, Debug, Default)]
pub struct Pcm {
    pub sample_rate: u32,
    pub channels: u16,
    /// interleaved
    pub samples: Vec<i16>,
}

impl Pcm {
    /// RIFF/WAVE bytes (bevy_audio's `AudioSource { bytes }` input).
    pub fn to_wav(&self) -> Vec<u8> {
        let data_len = (self.samples.len() * 2) as u32;
        let ba = self.channels as u32 * 2;
        let mut w = Vec::with_capacity(44 + data_len as usize);
        w.extend_from_slice(b"RIFF");
        w.extend_from_slice(&(36 + data_len).to_le_bytes());
        w.extend_from_slice(b"WAVEfmt ");
        w.extend_from_slice(&16u32.to_le_bytes());
        w.extend_from_slice(&1u16.to_le_bytes());
        w.extend_from_slice(&self.channels.to_le_bytes());
        w.extend_from_slice(&self.sample_rate.to_le_bytes());
        w.extend_from_slice(&(self.sample_rate * ba).to_le_bytes());
        w.extend_from_slice(&(ba as u16).to_le_bytes());
        w.extend_from_slice(&16u16.to_le_bytes());
        w.extend_from_slice(b"data");
        w.extend_from_slice(&data_len.to_le_bytes());
        for s in &self.samples {
            w.extend_from_slice(&s.to_le_bytes());
        }
        w
    }
}

/// SPU2 output rate. WP-G2 correction: a tone does NOT simply play at 48000 Hz at its centre note: 989snd
/// FUN_0001acc8 scales the pitch by 44100/48000 when the tone's centre note (+2) is >= 0 (see `spu::tone_pitch`).
pub const SPU_RATE: u32 = 48000;

/// Default handler note (989snd FUN_000043fc/FUN_00004000 set handler +0x34 = 0x3c).
pub const DEFAULT_NOTE: u16 = 0x3c;

/// Decoded file, by kind.
#[derive(Clone, Debug)]
pub enum DecodedFile {
    Bank(bank::Bank),
    Vag(vagp::Vagp, adpcm::Decoded),
    Music(vpk::VpkHeader, Vec<Vec<i16>>),
}

pub fn decode_file(kind: SoundFileKind, data: &[u8]) -> Option<DecodedFile> {
    match kind {
        SoundFileKind::Bank => bank::parse(data).map(DecodedFile::Bank),
        SoundFileKind::Vag => vagp::decode(data).map(|(h, d)| DecodedFile::Vag(h, d)),
        SoundFileKind::MusicStream => vpk::decode(data).map(|(h, c)| DecodedFile::Music(h, c)),
    }
}

/// A playing sound request (what the 989snd handle stood for).
#[derive(Clone, Debug, PartialEq)]
pub struct Voice {
    pub handle: u32,
    pub sound: i32,
    pub volume: f32,
    pub pitch: f32,
    pub pan: f32,
}

/// Engine-side audio state (replaces the 989snd EE client state + IOP). Host drains `pending` each frame.
#[derive(Resource, Default, Debug)]
pub struct AudioSystem {
    pub bank: Option<bank::Bank>,
    pub voices: Vec<Voice>,
    pub next_handle: u32,
    /// uMusicVolume (string at 0x2a89c0) / sfx volume, 0..1
    pub music_volume: f32,
    pub sfx_volume: f32,
    /// music registers (BIF 163 SetMusicRegister is compiled out in Sly 2: kept as plain storage)
    pub music_registers: [i32; 16],
}

impl AudioSystem {
    pub fn new() -> Self {
        Self { next_handle: 1, music_volume: 1.0, sfx_volume: 1.0, ..Default::default() }
    }

    /// Replaces snd_BankLoadByLoc (0x00231fb0): synchronous parse instead of IOP RPC + SPU upload.
    pub fn bank_load(&mut self, data: &[u8]) -> bool {
        self.bank = bank::parse(data);
        self.bank.is_some()
    }

    /// PCM of a bank sound's first tone at the default handler note 60 (rate from FUN_0001acc8).
    pub fn sound_pcm(&self, sound: i32) -> Option<Pcm> {
        let b = self.bank.as_ref()?;
        let t = b.sounds.get(sound as usize)?.grains.iter().find_map(|g| g.tone.as_ref())?;
        let d = b.decode_tone(t)?;
        Some(Pcm { sample_rate: t.rate_at(DEFAULT_NOTE, 0).round() as u32, channels: 1, samples: d.pcm })
    }

    // ---- SpliceHost sound BIF hooks (WP-F host calls these) ----

    /// BIF StartSound (RefOpStartSound 0x001251e8): returns a handle, 0 = no sound.
    pub fn bif_start_sound(&mut self, sound: i32, volume: f32) -> u32 {
        if self.bank.as_ref().is_none_or(|b| sound < 0 || sound as usize >= b.sounds.len()) {
            return 0;
        }
        let handle = self.next_handle.max(1);
        self.next_handle = handle.wrapping_add(1);
        self.voices.push(Voice { handle, sound, volume, pitch: 1.0, pan: 0.0 });
        handle
    }

    /// BIF StopSound (RefOpStopSound 0x00125398).
    pub fn bif_stop_sound(&mut self, handle: u32) {
        self.voices.retain(|v| v.handle != handle);
    }

    pub fn set_voice(&mut self, handle: u32, volume: Option<f32>, pitch: Option<f32>, pan: Option<f32>) {
        if let Some(v) = self.voices.iter_mut().find(|v| v.handle == handle) {
            v.volume = volume.unwrap_or(v.volume);
            v.pitch = pitch.unwrap_or(v.pitch);
            v.pan = pan.unwrap_or(v.pan);
        }
    }

    /// BIF SetMusicRegister (RefOpSetMusicRegister 0x00125eb0) — compiled-out no-op in Sly 2; value stored only.
    pub fn bif_set_music_register(&mut self, reg: i32, val: i32) {
        if let Some(r) = self.music_registers.get_mut(reg as usize) {
            *r = val;
        }
    }

    /// Replaces snd_StopAllStreams / FUN_002321b8 stream stop on level change.
    pub fn stop_all(&mut self) {
        self.voices.clear();
    }
}
