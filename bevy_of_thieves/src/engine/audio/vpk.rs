//! " KPV" ("VPK " read big-endian) interleaved multi-channel music streams (FUN_001fd0e8 +0xf4 / +0xf0 lists).
//! LE header: +0 magic 0x56504B20, +4 per-channel byte size, +8 data offset (0x800), +0xc interleave (0x2000),
//! +0x10 sample rate, +0x14 channel count. Data = per-channel interleave blocks ch0,ch1,...,ch0,...
//! The IOP stream player (989snd VAG streaming) is replaced by this whole-file decoder.

use super::adpcm::{self, AdpcmState};

pub const MAGIC: u32 = 0x5650_4B20;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VpkHeader {
    pub channel_size: u32,
    pub data_offset: u32,
    pub interleave: u32,
    pub sample_rate: u32,
    pub channels: u32,
}

fn le(d: &[u8], o: usize) -> u32 {
    u32::from_le_bytes(d[o..o + 4].try_into().unwrap())
}

pub fn parse(d: &[u8]) -> Option<VpkHeader> {
    if d.len() < 0x18 || le(d, 0) != MAGIC {
        return None;
    }
    let h = VpkHeader { channel_size: le(d, 4), data_offset: le(d, 8), interleave: le(d, 0xc), sample_rate: le(d, 0x10), channels: le(d, 0x14) };
    (h.channels > 0 && h.channels <= 16 && h.interleave % 16 == 0 && h.interleave > 0).then_some(h)
}

/// Decode to one PCM track per channel (music layers are mixed by the music-register logic, not here).
pub fn decode(d: &[u8]) -> Option<(VpkHeader, Vec<Vec<i16>>)> {
    let h = parse(d)?;
    let n = h.channels as usize;
    let il = h.interleave as usize;
    let mut st = vec![AdpcmState::default(); n];
    let mut out = vec![Vec::new(); n];
    let mut done = vec![0usize; n];
    let mut o = h.data_offset as usize;
    'outer: loop {
        for c in 0..n {
            if o >= d.len() {
                break 'outer;
            }
            let take = il.min(h.channel_size as usize - done[c].min(h.channel_size as usize)).min(d.len() - o);
            adpcm::decode_stream(&d[o..o + take], &mut st[c], &mut out[c]);
            done[c] += take;
            // last (partial) row is packed: each channel advances by its remaining size only
            o += take;
        }
        if done.iter().all(|&x| x >= h.channel_size as usize) {
            break;
        }
    }
    Some((h, out))
}
