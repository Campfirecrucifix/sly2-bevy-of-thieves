//! WP-G3 (feature `game`): REPLACED 989snd voice playback. Drains `Snd989::cmds` into bevy_audio:
//! Play -> one `AudioPlayer` of the bank sound's PCM (`AudioSystem::sound_pcm`, bank index = the 0x3dd040
//! record index FUN_001fd070), Stop / StopFade -> despawn (fade not rendered), Pause / Resume / SetParams
//! (volume) -> `AudioSink`. Voices whose player entity is gone are reported in `Snd989::ended` (the
//! FUN_001fbfb0 poll). Gain = vol / 1024 x effects bus (0x3dd040 +0x1c0[0] when `MusicMgr` exists, else
//! `AudioSystem::sfx_volume`). WP-EP1e: the start pan (Play mask 8, degrees 0 ahead / 90 right) is baked
//! into a stereo copy of the PCM (`pan_gains`; later SetParams pans and pitch are not rendered).
//! Registration (`Snd989` + this system after `game::app::tick_sound_emitters`) is the game app's.

use super::emitter_tick::{Snd989, SndCmd};
use super::music_mgr::MusicMgr;
use super::{AudioSystem, Pcm};
use bevy::audio::{AudioPlayer, AudioSink, AudioSinkPlayback, AudioSource, PlaybackSettings, Volume};
use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

/// Marks a 989snd voice player.
#[derive(Component, Clone, Copy, Debug)]
pub struct SndVoice(pub u32);

fn source(p: &Pcm) -> AudioSource {
    AudioSource { bytes: Arc::from(p.to_wav()) }
}

/// WP-EP1e (REPLACED SPU2 voice pan): 989snd pan degrees (0 ahead, 90 right, 180 behind, 270 left) ->
/// (left, right) gains, unity in the centre, the far side falls linearly to 0 at +-90 degrees.
pub fn pan_gains(deg: i32) -> (f32, f32) {
    let x = (deg as f32).to_radians().sin();
    ((1.0 - x).min(1.0), (1.0 + x).min(1.0))
}

/// Mono PCM -> stereo with the pan gains (other channel counts unchanged).
pub fn pan_pcm(p: &Pcm, deg: i32) -> Pcm {
    if p.channels != 1 {
        return p.clone();
    }
    let (l, r) = pan_gains(deg);
    let samples = p.samples.iter().flat_map(|&s| [(s as f32 * l) as i16, (s as f32 * r) as i16]).collect();
    Pcm { sample_rate: p.sample_rate, channels: 2, samples }
}

/// WP-EP1e verification (env `SLY2_AUDIO_LOG=1`): every sound started by the replaced voice playback
/// (bank sound, gain, pcm, start time s) for the per-second software-mix RMS of `game::app::audio_monitor`.
#[derive(Resource, Default)]
pub struct AudioMonitor {
    pub on: bool,
    pub started: Vec<(i32, f32, Arc<Pcm>, f64)>,
    /// sounds started since the last log line
    pub plays: Vec<i32>,
}

impl AudioMonitor {
    pub fn record(&mut self, sound: i32, gain: f32, p: &Pcm, t: f64) {
        if self.on {
            self.plays.push(sound);
            self.started.push((sound, gain, Arc::new(p.clone()), t));
        }
    }
}

/// voice -> (entity, seen in a previous frame)
#[derive(Default)]
pub struct Voices(HashMap<u32, (Entity, bool)>);

pub fn drain_snd_cmds(
    mut snd: ResMut<Snd989>,
    sys: Res<AudioSystem>,
    mgr: Option<Res<MusicMgr>>,
    mut sources: ResMut<Assets<AudioSource>>,
    live: Query<(), With<SndVoice>>,
    mut sinks: Query<&mut AudioSink, With<SndVoice>>,
    mut voices: Local<Voices>,
    mut commands: Commands,
    (mut mon, time): (Option<ResMut<AudioMonitor>>, Res<Time>),
) {
    let bus = mgr.map_or(sys.sfx_volume, |m| m.bus[0]);
    let gain = |mask: u32, vol: i32| if mask & 4 != 0 { vol as f32 / 1024.0 } else { 1.0 } * bus;
    let ended: Vec<u32> = voices.0.iter().filter(|(_, (e, seen))| *seen && live.get(*e).is_err()).map(|(v, _)| *v).collect();
    for v in ended {
        voices.0.remove(&v);
        snd.ended.insert(v);
    }
    voices.0.values_mut().for_each(|x| x.1 = true);
    for c in std::mem::take(&mut snd.cmds) {
        match c {
            SndCmd::Play { bank_sound, mask, vol, pan, voice, .. } => {
                let Some(p) = sys.sound_pcm(bank_sound) else { continue };
                snd.played += 1;
                let p = if mask & 8 != 0 { pan_pcm(&p, pan) } else { p };
                if let Some(m) = mon.as_deref_mut() {
                    m.record(bank_sound, gain(mask, vol), &p, time.elapsed_secs_f64());
                }
                let h = sources.add(source(&p));
                let e = commands.spawn((AudioPlayer(h), PlaybackSettings { volume: Volume::Linear(gain(mask, vol)), ..PlaybackSettings::DESPAWN }, SndVoice(voice.unwrap_or(0)))).id();
                if let Some(v) = voice {
                    voices.0.insert(v, (e, false));
                }
            }
            SndCmd::Stop { voice } | SndCmd::StopFade { voice, .. } => {
                if let Some((e, _)) = voices.0.remove(&voice) {
                    commands.entity(e).try_despawn();
                }
                snd.ended.insert(voice);
            }
            SndCmd::Pause { voice } | SndCmd::Resume { voice } => {
                if let Some(s) = voices.0.get(&voice).and_then(|(e, _)| sinks.get_mut(*e).ok()) {
                    if matches!(c, SndCmd::Pause { .. }) {
                        s.pause();
                    } else {
                        s.play();
                    }
                }
            }
            SndCmd::SetParams { voice, mask, vol, .. } => {
                if mask & 1 != 0 {
                    if let Some(mut s) = voices.0.get(&voice).and_then(|(e, _)| sinks.get_mut(*e).ok()) {
                        s.set_volume(Volume::Linear(vol as f32 / 1024.0 * bus));
                    }
                }
            }
            SndCmd::StopAll => {
                for (_, (e, _)) in voices.0.drain() {
                    commands.entity(e).try_despawn();
                }
            }
            SndCmd::BusVolume { .. } | SndCmd::Sukr { .. } | SndCmd::OutputMode { .. } => {}
        }
    }
}
