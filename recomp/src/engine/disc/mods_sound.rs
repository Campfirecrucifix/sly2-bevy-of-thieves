//! WP-R: sound mods (port addition). A mod replaces one bank sound with a WAV file:
//!   mods/<mod>/sounds/<bank lsn:07>/<sound index>.wav   (16-bit or 8-bit PCM, mono or stereo, any rate)
//! The bank lsn is the level's 989snd bank descriptor (FUN_001fd0e8 block +0xa8, `LevelGame::bank`);
//! the sound index is the bank sound number Splice/emitters use. The WAV is mixed to mono,
//! resampled to the tone's playback rate at the default note (so it plays at its own pitch through the
//! normal tone path), encoded to SPU2 ADPCM, appended to the bank's SPU data and every tone grain of
//! that sound is pointed at it (tone +0x10 SPU offset). Everything downstream (AudioSystem::sound_pcm,
//! emitters, BIF voices) then uses the mod sound; volume/pan/priority stay the original tone's.
//! Load order: last mod in `ModOverlay` wins (same rule as files and levels).

use super::mods::ModOverlay;
use crate::engine::audio::adpcm::{AdpcmState, COEF, FLAG_END};
use crate::engine::audio::bank::Bank;
use crate::engine::audio::{Pcm, DEFAULT_NOTE};

pub fn sound_override_path(bank_lsn: u32, sound: usize) -> String {
    format!("sounds/{bank_lsn:07}/{sound}.wav")
}

/// Minimal RIFF/WAVE reader: PCM format 1, 8 or 16 bit.
pub fn parse_wav(b: &[u8]) -> Option<Pcm> {
    if b.len() < 12 || &b[0..4] != b"RIFF" || &b[8..12] != b"WAVE" {
        return None;
    }
    let (mut fmt, mut data) = (None, None);
    let mut o = 12;
    while o + 8 <= b.len() {
        let id = &b[o..o + 4];
        let n = u32::from_le_bytes(b[o + 4..o + 8].try_into().ok()?) as usize;
        let body = b.get(o + 8..(o + 8 + n).min(b.len()))?;
        match id {
            b"fmt " if n >= 16 => fmt = Some(body),
            b"data" => data = Some(body),
            _ => {}
        }
        o += 8 + n + (n & 1);
    }
    let (f, d) = (fmt?, data?);
    let h = |i: usize| u16::from_le_bytes([f[i], f[i + 1]]);
    let (format, channels, rate, bits) = (h(0), h(2), u32::from_le_bytes(f[4..8].try_into().ok()?), h(14));
    if format != 1 || channels == 0 {
        return None;
    }
    let samples = match bits {
        16 => d.chunks_exact(2).map(|c| i16::from_le_bytes([c[0], c[1]])).collect(),
        8 => d.iter().map(|&x| ((x as i16) - 128) << 8).collect(),
        _ => return None,
    };
    Some(Pcm { sample_rate: rate, channels, samples })
}

/// Mix to mono and linearly resample to `rate`.
pub fn to_mono_rate(p: &Pcm, rate: u32) -> Vec<i16> {
    let ch = p.channels.max(1) as usize;
    let mono: Vec<f32> = p.samples.chunks_exact(ch).map(|c| c.iter().map(|&s| s as f32).sum::<f32>() / ch as f32).collect();
    if mono.is_empty() || p.sample_rate == 0 || rate == 0 {
        return vec![];
    }
    let n = ((mono.len() as u64 * rate as u64) / p.sample_rate as u64).max(1) as usize;
    let step = p.sample_rate as f64 / rate as f64;
    (0..n)
        .map(|i| {
            let x = i as f64 * step;
            let k = x as usize;
            let f = (x - k as f64) as f32;
            let a = mono[k.min(mono.len() - 1)];
            let b = mono[(k + 1).min(mono.len() - 1)];
            (a + (b - a) * f).round().clamp(-32768.0, 32767.0) as i16
        })
        .collect()
}

/// SPU2 ADPCM encoder (exhaustive predictor 0..4 x shift 0..12 per 28-sample block, exact decoder
/// model of audio::adpcm::decode_block). Last block carries FLAG_END (one-shot, no loop).
pub fn encode_adpcm(pcm: &[i16]) -> Vec<u8> {
    let mut out = vec![];
    let mut st = AdpcmState::default();
    let blocks = pcm.len().div_ceil(28).max(1);
    for bi in 0..blocks {
        let mut x = [0i32; 28];
        for (i, v) in x.iter_mut().enumerate() {
            *v = pcm.get(bi * 28 + i).copied().unwrap_or(0) as i32;
        }
        let mut best: Option<(i64, u8, [u8; 28], AdpcmState)> = None;
        for (p, &(f0, f1)) in COEF.iter().enumerate() {
            for shift in 0..=12u8 {
                let mut s = st;
                let mut err = 0i64;
                let mut nibs = [0u8; 28];
                for i in 0..28 {
                    let pred = (s.s1 * f0 + s.s2 * f1 + 32) >> 6;
                    let q = 1i32 << (12 - shift as i32);
                    let d = x[i] - pred;
                    let nib = ((d as f32 / q as f32).round() as i32).clamp(-8, 7);
                    let v = ((((nib as i16) << 12) as i32 >> shift) + pred).clamp(-32768, 32767);
                    err += ((x[i] - v) as i64).pow(2);
                    s.s2 = s.s1;
                    s.s1 = v;
                    nibs[i] = (nib & 0xf) as u8;
                }
                if best.as_ref().is_none_or(|b| err < b.0) {
                    best = Some((err, (p as u8) << 4 | shift, nibs, s));
                }
            }
        }
        let (_, hdr, nibs, s) = best.unwrap();
        st = s;
        out.push(hdr);
        out.push(if bi + 1 == blocks { FLAG_END } else { 0 });
        for k in 0..14 {
            out.push(nibs[2 * k] | nibs[2 * k + 1] << 4);
        }
    }
    out
}

/// Replace bank sound `sound` with mono PCM already at the tone rate. Returns false when the sound
/// has no tone grain.
pub fn replace_sound(bank: &mut Bank, sound: usize, pcm: &[i16]) -> bool {
    let Some(s) = bank.sounds.get(sound) else { return false };
    if !s.grains.iter().any(|g| g.tone.is_some()) {
        return false;
    }
    while bank.spu.len() % 16 != 0 {
        bank.spu.push(0);
    }
    let off = bank.spu.len() as u32;
    bank.spu.extend_from_slice(&encode_adpcm(pcm));
    for g in &mut bank.sounds[sound].grains {
        if let Some(t) = g.tone.as_mut() {
            t.vag_off = off;
            t.raw[0x10..0x14].copy_from_slice(&off.to_le_bytes());
        }
    }
    true
}

/// Playback rate `AudioSystem::sound_pcm` uses for the sound's first tone.
pub fn tone_rate(bank: &Bank, sound: usize) -> Option<u32> {
    let t = bank.sounds.get(sound)?.grains.iter().find_map(|g| g.tone.as_ref())?;
    Some(t.rate_at(DEFAULT_NOTE, 0).round() as u32)
}

/// Apply every `sounds/<bank_lsn>/<i>.wav` override of the load order. Blocking file reads: call at
/// bank load (the game system runs once per newly loaded bank). Returns the replaced sound indices.
pub fn apply_sound_mods(bank: &mut Bank, mods: &ModOverlay, bank_lsn: u32) -> Vec<usize> {
    let mut done = vec![];
    if mods.order.mods.is_empty() {
        return done;
    }
    for i in 0..bank.sounds.len() {
        let Some(path) = mods.resolve(&sound_override_path(bank_lsn, i)) else { continue };
        let Some(wav) = std::fs::read(&path).ok().and_then(|b| parse_wav(&b)) else {
            log::warn!("mods: {} is not a PCM WAV; ignored", path.display());
            continue;
        };
        let Some(rate) = tone_rate(bank, i) else { continue };
        if replace_sound(bank, i, &to_mono_rate(&wav, rate)) {
            log::info!("mods: bank {bank_lsn:07} sound {i} <- {}", path.display());
            done.push(i);
        }
    }
    done
}

#[cfg(feature = "game")]
pub use bevy_mods::*;

#[cfg(feature = "game")]
mod bevy_mods {
    use super::*;
    use crate::engine::audio::AudioSystem;
    use crate::engine::disc::LevelLoader;
    use crate::engine::game::app::LevelGame;
    use bevy::prelude::*;

    /// Schedule right after `game::app::poll_level_audio`: patches each newly loaded bank once.
    pub fn apply_sound_mods_system(loader: Option<Res<LevelLoader>>, g: Res<LevelGame>, mut sys: ResMut<AudioSystem>, mut done: Local<Option<(u32, usize)>>) {
        let (Some(loader), Some(d)) = (loader, g.bank) else { return };
        let Some(len) = sys.bank.as_ref().map(|b| b.spu.len()) else { return };
        if *done == Some((d.lsn(), len)) {
            return;
        }
        let bank = sys.bank.as_mut().unwrap();
        apply_sound_mods(bank, &loader.mods, d.lsn());
        *done = Some((d.lsn(), bank.spu.len()));
    }
}
