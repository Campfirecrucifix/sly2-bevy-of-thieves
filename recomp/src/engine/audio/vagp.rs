//! "VAGp" single-sample files (dialogue / one-shot streams referenced by FUN_001fd0e8 records).
//! Header 0x30 bytes, big-endian: +0x04 version, +0x0c data size, +0x10 sample rate, +0x20 name[16].

use super::adpcm::{self, Decoded};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vagp {
    pub version: u32,
    pub data_size: u32,
    pub sample_rate: u32,
    pub name: String,
}

fn be(d: &[u8], o: usize) -> u32 {
    u32::from_be_bytes(d[o..o + 4].try_into().unwrap())
}

pub fn parse(d: &[u8]) -> Option<Vagp> {
    if d.len() < 0x30 || &d[..4] != b"VAGp" {
        return None;
    }
    let name = d[0x20..0x30].split(|&c| c == 0).next().unwrap_or(&[]);
    Some(Vagp { version: be(d, 4), data_size: be(d, 0xc), sample_rate: be(d, 0x10), name: String::from_utf8_lossy(name).into() })
}

pub fn decode(d: &[u8]) -> Option<(Vagp, Decoded)> {
    let h = parse(d)?;
    let end = (0x30 + h.data_size as usize).min(d.len());
    Some((h.clone(), adpcm::decode(&d[0x30..end])))
}
