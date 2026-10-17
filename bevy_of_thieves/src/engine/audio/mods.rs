//! WP-G3: sound mods for the streamed files (port addition, same overlay rules as WP-A / WP-R): a mod
//! replaces a dialogue / one-shot VAGp file or a KPV music stream (level block FUN_001fd0e8 +0xec / +0xf4 /
//! +0xf0 descriptors) by a WAV, keyed by the disc LSN of the original:
//!   mods/<mod>/sounds/vag/<lsn:07>.wav     dialogue line / VAGp
//!   mods/<mod>/sounds/music/<lsn:07>.wav   music stream (played as ONE layer: the mod has no KPV layers)
//! Bank sounds stay `disc::mods_sound` (sounds/<bank lsn>/<index>.wav). Last mod in load order wins.

use super::{adpcm, read_descriptor, vagp, vpk, Pcm};
use crate::engine::disc::descriptor::Descriptor;
use crate::engine::disc::iso::SectorSource;
use crate::engine::disc::mods::ModOverlay;
use crate::engine::disc::mods_sound::parse_wav;

pub fn vag_override_path(lsn: u32) -> String {
    format!("sounds/vag/{lsn:07}.wav")
}
pub fn music_override_path(lsn: u32) -> String {
    format!("sounds/music/{lsn:07}.wav")
}

/// WAV of a mod overriding `rel`, if any (unparsable files are ignored).
pub fn mod_pcm(mods: &ModOverlay, rel: &str) -> Option<Pcm> {
    parse_wav(&std::fs::read(mods.resolve(rel)?).ok()?)
}

/// A dialogue / one-shot VAGp through the overlay (mod WAV, else the disc file decoded at its rate).
pub fn load_vag(disc: &dyn SectorSource, mods: &ModOverlay, d: &Descriptor) -> Option<Pcm> {
    if let Some(p) = mod_pcm(mods, &vag_override_path(d.lsn())) {
        return Some(p);
    }
    let (h, adpcm::Decoded { pcm, .. }) = vagp::decode(&read_descriptor(disc, d).ok()?)?;
    Some(Pcm { sample_rate: h.sample_rate, channels: 1, samples: pcm })
}

/// A music stream through the overlay: Ok(layers) where a mod WAV is the single layer, else the disc KPV
/// mixed per layer like the game (`music_layers::layer_channels`).
pub fn load_music(disc: &dyn SectorSource, mods: &ModOverlay, d: &Descriptor) -> Option<Vec<Pcm>> {
    if let Some(p) = mod_pcm(mods, &music_override_path(d.lsn())) {
        return Some(vec![p]);
    }
    let (h, ch) = vpk::decode(&read_descriptor(disc, d).ok()?)?;
    Some(
        (0..super::music_layers::layer_count(ch.len()))
            .map(|n| {
                let on = super::music_layers::layer_channels(n, ch.len());
                let g: Vec<f32> = (0..ch.len()).map(|c| if on.contains(&c) { 1.0 } else { 0.0 }).collect();
                super::music::mix_stereo(&h, &ch, &g)
            })
            .collect(),
    )
}
