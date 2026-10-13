//! WP-J16: Bevy side of the FMV replacement. `FmvPlayback` is the `fmv::FmvPlayer` the binary hands to
//! `MovieQueue::ee_sub_1b3b08` (FUN_001b3b08): instead of blocking the frame like FUN_001b3bd0, each movie
//! is queued and played here over the following frames (`movie_playing` gates the main loop meanwhile, the
//! equivalent of the original blocking call).
//!
//! Per movie a thread reads the PSS from the disc (`fmv_pss`): pass 1 demuxes the ADPCM audio into one
//! WAV (bevy_audio), pass 2 decodes the MPEG-2 pictures into a bounded channel. The picture is shown by a
//! 2D camera above everything (black clear) as a 4:3 sprite fitted by `fmv_pss::movie_rect` (pillarbox on
//! 16:9 / 21:9 / 32:9). Video time follows the wall clock from the first picture (`fps` of the sequence
//! header). Skip: FUN_00187d80(pad, 0x91f) when the queue entry was skippable (rule of FUN_001b3bd0, see
//! `fmv::MovieQueue::ee_sub_1b3bd0`).

use super::fmv::{FmvPlayer, SKIP_BUTTONS};
use super::fmv_pss::{movie_rect, PssDecoder, VideoFrame, READ_CHUNK};
use crate::engine::audio::Pcm;
use crate::engine::disc::descriptor::Descriptor;
use crate::engine::disc::iso::{IsoImage, SectorSource};
use crate::engine::disc::LevelLoader;
use crate::engine::input::backend::PadInput;
use crate::engine::input::pad::FUN_00187d80;
use bevy::asset::RenderAssetUsages;
use bevy::audio::{AudioPlayer, AudioSource, PlaybackSettings};
use bevy::camera::ClearColorConfig;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use std::collections::VecDeque;
use std::sync::mpsc::{sync_channel, Receiver, TryRecvError};
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Decoded pictures buffered ahead of display.
const FRAME_AHEAD: usize = 6;

enum Msg {
    Audio(Pcm),
    Fps(f32),
    Frame(VideoFrame),
}

struct Playing {
    desc: Descriptor,
    skippable: bool,
    rx: Mutex<Receiver<Msg>>,
    fps: f32,
    start: Option<Instant>,
    shown: u32,
    image: Option<Handle<Image>>,
    entities: Vec<Entity>,
    /// thread finished (channel closed)
    eof: bool,
}

/// Movie queue playback (replacement of FUN_001b3bd0's decode loop).
#[derive(Resource, Default)]
pub struct FmvPlayback {
    pending: VecDeque<(Descriptor, bool)>,
    cur: Option<Playing>,
    /// movies finished (played to the end or skipped), in order
    pub played: Vec<(Descriptor, bool)>,
}

impl FmvPlayer for FmvPlayback {
    fn play(&mut self, desc: &Descriptor, skippable: bool) -> bool {
        self.pending.push_back((*desc, skippable));
        true
    }
}

impl FmvPlayback {
    pub fn busy(&self) -> bool {
        self.cur.is_some() || !self.pending.is_empty()
    }
}

/// run condition: no movie playing (the original main loop is blocked inside FUN_001b3b08).
pub fn movie_playing(p: Option<Res<FmvPlayback>>) -> bool {
    p.is_some_and(|p| p.busy())
}

pub fn no_movie(p: Option<Res<FmvPlayback>>) -> bool {
    !movie_playing(p)
}

fn decode_thread(iso: Arc<IsoImage>, d: Descriptor, tx: std::sync::mpsc::SyncSender<Msg>) {
    let size = d.size() as usize;
    let read = |off: usize| iso.read_bytes(d.lsn() + (off / 2048) as u32, READ_CHUNK.min(size - off));
    // pass 1: audio
    let mut dec = PssDecoder::default();
    let mut pcm = Vec::new();
    let mut off = 0;
    while off < size {
        let Ok(b) = read(off) else { break };
        dec.demux.push(&b);
        while let Some(p) = dec.demux.next_packet() {
            if p.stream == 0xbd {
                dec.audio.push(&p.data, &mut pcm);
            }
        }
        off += b.len();
    }
    if let Some(h) = dec.audio.head.filter(|h| h.channels > 0 && !pcm.is_empty()) {
        if tx.send(Msg::Audio(Pcm { sample_rate: h.rate, channels: h.channels as u16, samples: pcm })).is_err() {
            return;
        }
    }
    // pass 2: video
    let mut dec = PssDecoder::default();
    let mut sink = Vec::new();
    let mut off = 0;
    let mut fps_sent = false;
    let mut closed = false;
    while off < size && !closed {
        let Ok(b) = read(off) else { break };
        off += b.len();
        let mut frames = Vec::new();
        dec.feed(&b, true, &mut sink, &mut |f| frames.push(f));
        sink.clear();
        if !fps_sent {
            if let Some(f) = dec.fps {
                fps_sent = true;
                closed |= tx.send(Msg::Fps(f)).is_err();
            }
        }
        for f in frames {
            if closed || tx.send(Msg::Frame(f)).is_err() {
                closed = true;
                break;
            }
        }
    }
    if !closed {
        dec.finish(true, &mut |f| {
            let _ = tx.send(Msg::Frame(f));
        });
    }
}

fn start(p: &mut FmvPlayback, iso: &Arc<IsoImage>) {
    let Some((desc, skippable)) = p.pending.pop_front() else { return };
    info!("FMV: playing movie lsn {} ({} bytes, skippable {skippable})", desc.lsn(), desc.size());
    let (tx, rx) = sync_channel(FRAME_AHEAD);
    let iso = iso.clone();
    std::thread::spawn(move || decode_thread(iso, desc, tx));
    p.cur = Some(Playing { desc, skippable, rx: Mutex::new(rx), fps: 30000.0 / 1001.0, start: None, shown: 0, image: None, entities: Vec::new(), eof: false });
}

/// Update: start / advance / skip / end the current movie.
pub fn fmv_update(
    mut commands: Commands,
    mut p: ResMut<FmvPlayback>,
    loader: Option<Res<LevelLoader>>,
    mut pad: Option<ResMut<PadInput>>,
    mut images: ResMut<Assets<Image>>,
    mut sources: ResMut<Assets<AudioSource>>,
    windows: Query<&Window>,
    mut sprites: Query<&mut Sprite>,
) {
    if p.cur.is_none() {
        let Some(l) = loader else {
            p.pending.clear();
            return;
        };
        start(&mut p, &l.iso);
        let Some(c) = p.cur.as_mut() else { return };
        let cam = commands
            .spawn((Camera2d, Camera { order: 1000, clear_color: ClearColorConfig::Custom(Color::BLACK), ..default() }))
            .id();
        c.entities.push(cam);
    }
    let c = p.cur.as_mut().unwrap();
    // skip (FUN_00187d80(0x2dfc00, 0x91f))
    let skip = c.skippable && pad.as_mut().is_some_and(|pd| FUN_00187d80(&mut pd.pad, SKIP_BUTTONS as u32) != 0);
    let mut latest = None;
    if !skip {
        let target = c.start.map_or(0, |s| (s.elapsed().as_secs_f32() * c.fps) as u32);
        while c.shown <= target {
            let m = c.rx.get_mut().unwrap().try_recv();
            match m {
                Ok(Msg::Audio(pcm)) => {
                    let h = sources.add(AudioSource { bytes: Arc::from(pcm.to_wav()) });
                    let e = commands.spawn((AudioPlayer(h), PlaybackSettings::DESPAWN)).id();
                    c.entities.push(e);
                }
                Ok(Msg::Fps(f)) => c.fps = f,
                Ok(Msg::Frame(f)) => {
                    c.shown += 1;
                    latest = Some(f);
                    if c.start.is_none() {
                        c.start = Some(Instant::now());
                        break;
                    }
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    c.eof = true;
                    break;
                }
            }
        }
    }
    if let Some(f) = latest {
        let size = Extent3d { width: f.width, height: f.height, depth_or_array_layers: 1 };
        let img = Image::new(size, TextureDimension::D2, f.rgba, TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::RENDER_WORLD);
        match &c.image {
            Some(h) => {
                let _ = images.insert(h.id(), img);
            }
            None => {
                let h = images.add(img);
                c.image = Some(h.clone());
                let e = commands.spawn(Sprite { image: h, ..default() }).id();
                c.entities.push(e);
            }
        }
    }
    // 4:3 picture in the window (letter/pillarbox)
    if let Some(w) = windows.iter().next() {
        let (pw, ph) = movie_rect(w.width(), w.height());
        for e in &c.entities {
            if let Ok(mut s) = sprites.get_mut(*e) {
                s.custom_size = Some(Vec2::new(pw, ph));
            }
        }
    }
    if skip || c.eof {
        let c = p.cur.take().unwrap();
        info!("FMV: movie lsn {} {} after {} frames", c.desc.lsn(), if skip { "skipped" } else { "ended" }, c.shown);
        for e in c.entities {
            commands.entity(e).try_despawn();
        }
        if let Some(h) = c.image {
            images.remove(h.id());
        }
        p.played.push((c.desc, skip));
    }
}

pub struct FmvPlugin;

impl Plugin for FmvPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FmvPlayback>().add_systems(Update, fmv_update.run_if(movie_playing));
    }
}
