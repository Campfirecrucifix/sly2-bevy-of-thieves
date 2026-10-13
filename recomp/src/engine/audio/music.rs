//! WP-G2: music stream mixing (replaces the 989snd VAG/KPV stream voices).
//! 989snd FUN_0000b920 programs a stream: 1 channel -> one voice with L and R volume; 2 channels -> voice 0
//! left-only, voice 1 right-only; pitch = rate << 12 / 48000; ADSR 0x000f / 0x1fc0 (instant attack, hold).
//! Channels beyond 2 (Sly 2 music has up to 5) are treated as further layers: pairs = stereo, a trailing odd
//! channel = mono centre. Layer volumes are host-controlled (`gains`, 0..1); the original layer selection
//! logic (SetMusicRegister BIF 163) is compiled out in Sly 2, so registers only store values.

use super::vpk::VpkHeader;
use super::Pcm;

/// FUN_0000b920 stream voice ADSR
pub const STREAM_ADSR: (u16, u16) = (0x000f, 0x1fc0);

/// Stereo mix of decoded KPV channels with per-channel gain (missing gains = 1.0). Output at the stream rate.
pub fn mix_stereo(h: &VpkHeader, chans: &[Vec<i16>], gains: &[f32]) -> Pcm {
    let n = chans.iter().map(|c| c.len()).max().unwrap_or(0);
    let mut acc = vec![0f32; n * 2];
    let g = |c: usize| gains.get(c).copied().unwrap_or(1.0);
    let mut c = 0;
    while c < chans.len() {
        let pair = c + 1 < chans.len();
        for (i, &s) in chans[c].iter().enumerate() {
            let v = s as f32 * g(c);
            acc[2 * i] += v;
            if !pair {
                acc[2 * i + 1] += v;
            }
        }
        if pair {
            for (i, &s) in chans[c + 1].iter().enumerate() {
                acc[2 * i + 1] += s as f32 * g(c + 1);
            }
        }
        c += 2;
    }
    Pcm { sample_rate: h.sample_rate, channels: 2, samples: acc.into_iter().map(|v| v.clamp(-32768.0, 32767.0) as i16).collect() }
}
