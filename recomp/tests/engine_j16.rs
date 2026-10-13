//! WP-J16 tests (headless): world-update callees, FMV (PSS decode), save/continue wiring.

use sly2_recomp::engine::disc::descriptor::static_table;
use sly2_recomp::engine::disc::iso::{IsoImage, SectorSource};
use sly2_recomp::engine::game::fmv_pss::*;
use std::path::PathBuf;
use std::sync::Arc;

fn root() -> PathBuf {
    let r = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    if r.join("Sly 2 - Band of Thieves (USA).iso").exists() { r } else { PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/..")) }
}

fn iso() -> Option<Arc<IsoImage>> {
    IsoImage::open(root().join("Sly 2 - Band of Thieves (USA).iso")).ok().map(Arc::new)
}

/// Movie rows of the ELF static table (first bytes 00 00 01 BA).
fn movies(iso: &IsoImage) -> Vec<(usize, sly2_recomp::engine::disc::descriptor::Descriptor)> {
    let elf = iso.read_file("SCUS_973.16").unwrap();
    let t = static_table(&elf).unwrap();
    t.iter()
        .enumerate()
        .filter(|(_, d)| d.size() != 0 && iso.read_bytes(d.lsn(), 4).is_ok_and(|b| b == [0, 0, 1, 0xba]))
        .map(|(i, d)| (i, *d))
        .collect()
}

#[test]
fn fmv_movie_rect_32_9() {
    assert_eq!(movie_rect(640.0, 480.0), (640.0, 480.0));
    // 32:9: pillarbox, 4:3 picture of full height
    assert_eq!(movie_rect(5120.0, 1440.0), (1920.0, 1440.0));
    // 16:9 / 21:9
    assert_eq!(movie_rect(1920.0, 1080.0), (1440.0, 1080.0));
    // portrait: letterbox
    assert_eq!(movie_rect(600.0, 1000.0), (600.0, 450.0));
}

/// Every movie: PS stream demuxes to the end, audio header is SPU ADPCM 44100 Hz stereo interleave 0x20 and
/// the whole SSbd body is decoded; the first pictures decode to 640x480 with real content.
#[test]
fn fmv_all_movies_demux_and_decode() {
    let Some(iso) = iso() else { return };
    let ms = movies(&iso);
    assert_eq!(ms.iter().map(|m| m.0).collect::<Vec<_>>(), [(0..22).collect::<Vec<_>>(), vec![24, 28, 29]].concat());
    for (row, d) in ms {
        let mut dec = PssDecoder::default();
        let mut pcm = Vec::new();
        let mut frames = 0u32;
        let mut first: Option<VideoFrame> = None;
        let size = d.size() as usize;
        let mut off = 0;
        // decode the video of the first 1 MiB, demux + audio for the rest (video packets skipped)
        while off < size {
            let n = READ_CHUNK.min(size - off);
            let buf = iso.read_bytes(d.lsn() + (off / 2048) as u32, n).unwrap();
            if off < 0x100000 {
                dec.feed(&buf, first.is_none(), &mut pcm, &mut |f| {
                    frames += 1;
                    if first.is_none() && f.rgba.iter().step_by(4).any(|&r| r > 32) {
                        first = Some(f);
                    }
                });
            } else {
                dec.demux.push(&buf);
                while let Some(p) = dec.demux.next_packet() {
                    if p.stream == 0xbd {
                        dec.audio.push(&p.data, &mut pcm);
                    }
                }
            }
            off += n;
            pcm.clear();
        }
        let h = dec.audio.head.unwrap();
        println!("row {row}: {:?} {:?} body {} consumed {} frames(1MiB) {frames} fps {:?} errors {} restarts {}", h, first.as_ref().map(|f| (f.width, f.height)), dec.audio.body_size, dec.audio.consumed, dec.fps, dec.errors, dec.restarts);
        assert!(h == SShd { kind: 0x10, rate: h.rate, channels: 2, interleave: 0x20 } && (h.rate == 44100 || h.rate == 48000), "row {row}");
        assert!(dec.audio.body_size as u64 - dec.audio.consumed < 0x40, "row {row}");
        assert!(frames > 10, "row {row}");
        let f = first.unwrap_or_else(|| panic!("row {row}: no non-black frame"));
        assert!((f.width == 640 || f.width == 720) && f.height == 480, "row {row}: {}x{}", f.width, f.height);
    }
}

/// Smallest movie (row 24, the intro) fully decoded: video length matches the audio length.
#[test]
fn fmv_intro_full_decode_av_length() {
    let Some(iso) = iso() else { return };
    let elf = iso.read_file("SCUS_973.16").unwrap();
    let d = static_table(&elf).unwrap()[24];
    let data = iso.read_bytes(d.lsn(), d.size() as usize).unwrap();
    let mut dec = PssDecoder::default();
    let mut pcm = Vec::new();
    let mut frames = 0u32;
    for c in data.chunks(READ_CHUNK) {
        dec.feed(c, false, &mut pcm, &mut |_| frames += 1);
    }
    dec.finish(false, &mut |_| frames += 1);
    let fps = dec.fps.unwrap();
    let vt = frames as f32 / fps;
    let at = pcm.len() as f32 / 2.0 / 44100.0;
    println!("intro: {frames} frames @ {fps} = {vt} s, audio {at} s");
    assert!((vt - at).abs() < 0.5, "video {vt} audio {at}");
    assert_eq!(dec.errors, 0);
}

// ---------------------------------------------------------------- item 1: world update callees

use sly2_recomp::engine::audio::music_layers::*;
use sly2_recomp::engine::game::world_update::*;

#[test]
fn j16_music_layers_select_and_crossfade() {
    let mut m = MusicLayers::default();
    let steady = Some(InstState { starting: false });
    // instance starting: active layer 1 forced to 1, others 0
    m.ee_sub_1fdc08(None, Some(InstState { starting: true }), 1.0 / 60.0);
    assert_eq!((m.layer, m.w), (1, [0.0, 1.0, 0.0]));
    assert_eq!(m.vol[..3], [0.0, 1.0, 0.0]);
    assert_eq!(m.dirty, 2);
    // player focus mode 3 -> layer 0; speed 0 (ELF BSS): weight held, layer 1 keeps playing
    let p = PlayerMusic { f_10ec: 3, f_10c8: -1, f_4c8: -1 };
    m.ee_sub_1fdc08(Some(p), steady, 1.0 / 60.0);
    assert_eq!((m.layer, m.w), (0, [0.0, 1.0, 0.0]));
    // with a crossfade speed the weights move with equal power
    m.speed[0] = [1.0, 0.0, 10.0];
    for _ in 0..10 {
        m.ee_sub_1fdc08(Some(p), steady, 1.0 / 60.0);
        let e: f32 = m.w.iter().map(|w| w * w).sum();
        assert!((e - 1.0).abs() < 1e-4, "{:?}", m.w);
    }
    assert!(m.w[0] > 0.0 && m.w[1] < 1.0);
    // rules: +0x5c -> 2, +0x60 -> 0, focus 4 -> 2, attach 0..2 / 0xd -> 0, +0x10c8 set -> 0
    let lay = |f5c, f60, p: Option<PlayerMusic>| {
        let mut m = MusicLayers { f_5c: f5c, f_60: f60, ..Default::default() };
        m.ee_sub_1fdc08(p, None, 0.0);
        m.layer
    };
    let pm = |a, b, c| Some(PlayerMusic { f_10ec: a, f_10c8: b, f_4c8: c });
    assert_eq!(lay(1, 1, pm(3, -1, -1)), 2);
    assert_eq!(lay(0, 1, None), 0);
    assert_eq!(lay(0, 0, pm(4, -1, -1)), 2);
    assert_eq!(lay(0, 0, pm(-1, -1, 2)), 0);
    assert_eq!(lay(0, 0, pm(-1, -1, 0xd)), 0);
    assert_eq!(lay(0, 0, pm(-1, -1, 5)), 1);
    assert_eq!(lay(0, 0, pm(-1, 7, -1)), 0);
    assert_eq!(lay(0, 0, None), 1);
    // FUN_001fc2e0 1/1024 quantisation + clamp
    let mut m = MusicLayers::default();
    assert!(m.ee_sub_1fc2e0(2.0, 0));
    assert_eq!(m.vol[0], 1.0);
    assert!(!m.ee_sub_1fc2e0(1.0005, 0));
    assert!(!m.ee_sub_1fc2e0(0.5, 6));
    assert_eq!(layer_channels(2, 5), vec![4]);
    assert_eq!(layer_count(5), 3);
}

#[test]
fn j16_joy_rec() {
    let mut j = JoyRec { mode: 1, ..Default::default() };
    let pad = PadSnap { left: [0.0, 1.0], right: [0.0, 0.0], held: 0x50, masked: 0x10 };
    j.ee_sub_1c4468(&pad, 0x100, [1.0, 0.0, 0.0], 5.0);
    assert_eq!(j.held, 0x140);
    assert_eq!(j.press_t[6], 5.0);
    assert_eq!(j.press_t[8], 5.0);
    assert_eq!(j.press_t[4], sly2_recomp::engine::player::PRESS_NONE);
    // stick up with camera yaw 0 -> world angle 0
    assert!((j.l_mag - 1.0).abs() < 1e-6 && j.l_angle.abs() < 1e-5, "{} {}", j.l_mag, j.l_angle);
    assert_eq!(j.l_t, [0.0, 5.0]);
    assert_eq!(j.r_t, [5.0, 0.0]);
    // released: angle kept, magnitude decays 0.2 per update
    j.ee_sub_1c4468(&PadSnap::default(), 0, [1.0, 0.0, 0.0], 6.0);
    assert!((j.l_mag - 0.8).abs() < 1e-6 && j.l_angle.abs() < 1e-5);
    assert!((j.l[1] - 0.8).abs() < 1e-5, "{:?}", j.l);
    // held buttons: no new press stamp
    assert_eq!(j.press_t[6], 5.0);
    // mode 0: cleared + presses consumed
    j.mode = 0;
    j.ee_sub_1c4468(&pad, 0, [1.0, 0.0, 0.0], 7.0);
    // the +0x70 hold is not cleared by FUN_001c42e8: the decay continues (0.8 - 0.2)
    assert_eq!(j.held, 0);
    assert!((j.l_mag - 0.6).abs() < 1e-6);
    assert!(j.press_t.iter().all(|&t| t == sly2_recomp::engine::player::PRESS_NONE));
}

#[test]
fn j16_queues_and_card() {
    let mut q: std::collections::VecDeque<u32> = [3, 1, 2].into();
    let mut seen = vec![];
    ee_sub_1be1e8(&mut q, |o| seen.push(o));
    assert_eq!((seen, q.len()), (vec![3, 1, 2], 0));
    let mut l = vec![StampNode { obj: 1, stop: 0, stamp: 1.0 }, StampNode { obj: 2, stop: 0, stamp: 2.0 }, StampNode { obj: 3, stop: 1, stamp: 0.0 }];
    let mut seen = vec![];
    ee_sub_1ec968(&mut l, 2.0, |o| seen.push(o));
    assert_eq!(seen, vec![1]);
    assert_eq!(l[0].stamp, 2.0);

    #[derive(Default)]
    struct H {
        calls: Vec<&'static str>,
        shown: Vec<i32>,
        ok: bool,
    }
    impl CardHost for H {
        fn poll(&mut self, _: &mut CardMgr) {
            self.calls.push("poll");
        }
        fn finish(&mut self, _: &mut CardMgr) {
            self.calls.push("finish");
        }
        fn step(&mut self, _: &mut CardMgr) {
            self.calls.push("step");
        }
        fn dialog_up(&self) -> bool {
            false
        }
        fn card_ok(&self) -> bool {
            self.ok
        }
        fn show(&mut self, m: i32) {
            self.shown.push(m);
        }
    }
    let mut c = CardMgr { state: 2, target: 2, t_10: 0.0, t_14: 1.0, thread: -1 };
    let mut h = H::default();
    ee_sub_1ab1a0(&mut c, &mut h, false, 1, 2.0);
    assert_eq!((h.calls.clone(), h.shown.clone()), (vec!["poll"], vec![0x12]));
    let mut h = H { ok: true, ..Default::default() };
    c.state = 6;
    c.target = 5;
    ee_sub_1ab1a0(&mut c, &mut h, false, 1, 0.5);
    assert_eq!((h.calls.clone(), h.shown.clone()), (vec!["step"], vec![]));
    let mut h = H::default();
    ee_sub_1ab1a0(&mut c, &mut h, true, 1, 2.0);
    assert!(h.calls.is_empty());
}

/// The four slot objects exist in the levels (FUN_0020f710 lookup).
#[test]
fn j16_slot_oids() {
    let mut slots = [Slot::default(); 4];
    ee_sub_20f710(&mut slots, |oid| Some(oid as u32));
    assert_eq!(slots.map(|s| s.obj), [0x371, 0x36d, 0x36e, 0x36f]);
}

/// Item 3: continue from a save slot: FUN_00165e38 in full (episode level / hideout), episode completion
/// values (DAT_003e100c) and the game completion FUN_001d82b8.
#[test]
fn j16_continue_hideout_completion() {
    use sly2_recomp::engine::disc::mods::ModOverlay;
    use sly2_recomp::engine::disc::LevelLoader;
    use sly2_recomp::engine::game::flow::{self, Dest, Start, StartPaths, EPISODE_WEIGHTS, HIDEOUT_DESC};
    use sly2_recomp::engine::game::save::{GameData, MemoryCard, SaveValue, SaveVars};
    let Some(iso) = iso() else { return };
    let elf = iso.read_file("SCUS_973.16").unwrap();
    let st = static_table(&elf).unwrap();
    flow::set_static_table(st.clone());
    let loader = LevelLoader::new(iso, ModOverlay::default());
    let lv = loader.load_blocking(st[25]).unwrap();
    let table = lv.header.level_table.clone();
    let vars = SaveVars::from_block(&lv.header.block_3a18);
    let paths = StartPaths::new(&vars);
    assert!(paths.dt.is_some() && paths.episode_current.is_some());
    assert!(paths.percent.iter().all(|p| p.is_some()), "{:?}", paths.percent);
    let mut d = GameData::default();
    vars.ee_sub_1d7c80(&mut d);
    let set = |p: &[u16], v: f32, d: &mut GameData| {
        let nv = match vars.ee_sub_1d7ed8(p, d) {
            SaveValue::Float(_) => SaveValue::Float(v),
            _ => SaveValue::Int(v as i32),
        };
        vars.ee_sub_1d80c8(p, nv, d);
    };
    // fresh blob: completion 0, title hides
    assert_eq!(flow::FUN_001d82b8(&vars, &paths, &d), 0);
    flow::take_posted();
    assert_eq!(flow::FUN_00165e38_full(true, &vars, &paths, &d, Some(&table), 1), Start::HideTitle);
    assert!(flow::take_posted().is_empty());
    // a game in progress: episode 3 (temple), 40 % -> level slot 3
    set(paths.dt.as_deref().unwrap(), 300.0, &mut d);
    set(paths.episode_current.as_deref().unwrap(), 3.0, &mut d);
    set(paths.percent[3].as_deref().unwrap(), 40.0, &mut d);
    assert_eq!(flow::episode_completion(&vars, &paths, &d, 3), 40);
    let total = EPISODE_WEIGHTS.iter().sum::<i32>() * 100 + 0xd0;
    eprintln!("continue: total {total}, completion {}", flow::FUN_001d82b8(&vars, &paths, &d));
    assert_eq!(flow::FUN_001d82b8(&vars, &paths, &d), (40 * 147 * 100 / total).clamp(1, 99));
    // through the slot file (0x1ac420 write / 0x1ad320 read)
    let dir = std::env::temp_dir().join(format!("sly2_j16_{}", std::process::id()));
    let card = MemoryCard::new(&dir);
    sly2_recomp::engine::game::flow::save_slot(&card, 0, Some(&vars), &mut d).unwrap();
    let back = flow::FUN_001ad320(&card, 0, &vars, paths.dt.as_deref()).unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(flow::episode_completion(&vars, &paths, &back, 3), 40);
    let s = flow::FUN_00165e38_full(true, &vars, &paths, &back, Some(&table), 1);
    let Start::Posted(p) = s else { panic!("{s:?}") };
    assert_eq!((p.dest, p.spawn, p.flags), (Dest::Slot(3), 0x189, 0x40));
    assert_eq!(p.slots.unwrap().restart, -1);
    let r = flow::resolve(&p, Some(&table), None).unwrap();
    assert_eq!(r.desc.0, table.slots[3].0);
    // episode complete (> 99) -> the hideout 0x2a3828 = static row 23
    let mut b = back.clone();
    set(paths.percent[3].as_deref().unwrap(), 100.0, &mut b);
    let Start::Posted(p) = flow::FUN_00165e38_full(true, &vars, &paths, &b, Some(&table), 1) else { panic!() };
    assert_eq!(p.dest, Dest::Static(HIDEOUT_DESC));
    let r = flow::resolve(&p, Some(&table), None).unwrap();
    assert_eq!(r.desc.0, st[23].0);
    // every episode at 100 % (no vault): 99 (vaults missing)
    for i in 0..9 {
        set(paths.percent[i].as_deref().unwrap(), 100.0, &mut b);
    }
    assert_eq!(flow::FUN_001d82b8(&vars, &paths, &b), 99);
    flow::take_posted();
}
