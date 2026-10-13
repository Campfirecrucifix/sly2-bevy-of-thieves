//! WP-EP1e tests: the 1:1 emitter path the game now runs (FUN_001fb8e0 via `music_mgr::tick_all`):
//! 3D falloff + pan of FUN_001fb778, the Play command of FUN_001fbc58, the baked stereo pan.
use sly2_recomp::engine::audio::emitter::EmitterSystem;
use sly2_recomp::engine::audio::emitter_tick::{ee_sub_1fb778, EmitterWorld, Snd989, SndCmd, SoundTable};
use sly2_recomp::engine::audio::music_mgr;
#[cfg(feature = "game")]
use sly2_recomp::engine::audio::Pcm;

/// listener at the origin, camera = identity basis (x forward, y left, z up)
struct W;
impl EmitterWorld for W {
    fn listener(&self) -> [f32; 3] {
        [0.0; 3]
    }
    fn to_camera(&self, v: [f32; 3]) -> [f32; 3] {
        v
    }
    fn attach_live(&self, _: i32) -> bool {
        true
    }
    fn attach_pos(&self, _: i32, pos: [f32; 3]) -> [f32; 3] {
        pos
    }
}

#[test]
fn ep1e_falloff_and_pan() {
    // inside near = 1, outside far = 0, linear between
    assert_eq!(ee_sub_1fb778(&W, [100.0, 0.0, 0.0], 300.0, 3000.0).0, 1.0);
    assert_eq!(ee_sub_1fb778(&W, [5000.0, 0.0, 0.0], 300.0, 3000.0), (0.0, 0));
    let (f, pan) = ee_sub_1fb778(&W, [0.0, -1650.0, 0.0], 300.0, 3000.0);
    assert!((f - 0.5).abs() < 1e-6);
    assert_eq!(pan, 90); // to the right (-y = right in the camera frame)
    assert_eq!(ee_sub_1fb778(&W, [0.0, 1650.0, 0.0], 300.0, 3000.0).1, -90);
}

#[test]
fn ep1e_emitter_plays_with_volume_and_pan() {
    let mut es = EmitterSystem::default();
    let mut snd = Snd989 { table: SoundTable { kinds: vec![0], map: vec![0], bank: 1 }, ..Default::default() };
    let id = es.new_emitter_1fbb90(0);
    es.get_mut(id).unwrap().pos = [0.0, -1650.0, 0.0];
    es.set_unpositioned_1fb2f8(id, 1); // attached (FUN_00210740 path)
    for _ in 0..3 {
        music_mgr::tick_all(&mut es, &mut snd, &W);
    }
    let plays: Vec<_> = snd.cmds.iter().filter_map(|c| match c {
        SndCmd::Play { mask, vol, pan, .. } => Some((*mask, *vol, *pan)),
        _ => None,
    }).collect();
    assert_eq!(plays.len(), 1, "{:?}", snd.cmds);
    let (mask, vol, pan) = plays[0];
    assert_eq!(mask & 0xc, 0xc);
    assert_eq!(vol, 512);
    assert_eq!(pan, 90);
}

#[cfg(feature = "game")]
#[test]
fn ep1e_pan_pcm_right() {
    use sly2_recomp::engine::audio::bevy_out::pan_pcm;
    let p = Pcm { sample_rate: 22050, channels: 1, samples: vec![10000; 64] };
    let s = pan_pcm(&p, 90);
    assert_eq!(s.channels, 2);
    let (l, r) = (s.samples[0].unsigned_abs(), s.samples[1].unsigned_abs());
    assert!(r > l, "right louder: l {l} r {r}");
}

// ---------------------------------------------------------------- R13 controls

#[cfg(feature = "game")]
mod controls {
    use bevy::input::gamepad::{Gamepad, GamepadAxis, GamepadButton};
    use bevy::prelude::*;
    use sly2_recomp::engine::input::backend::{sample, DeviceScript};
    use sly2_recomp::engine::input::bindings::Bindings;
    use sly2_recomp::engine::input::pad::{BTN_CROSS, BTN_START};

    #[test]
    fn ep1e_device_script_axis_sign() {
        let s = DeviceScript::parse("k1-3:Enter+KeyW;p4-5:LeftStickY++South;g1-2:0,0:START;7-8:0,1");
        assert_eq!(s.steps.len(), 2);
        assert_eq!(s.steps[0], (1, 3, true, vec!["Enter".to_string(), "KeyW".to_string()]));
        assert_eq!(s.steps[1], (4, 5, false, vec!["LeftStickY+".to_string(), "South".to_string()]));
    }

    #[test]
    fn ep1e_rebind_tap_and_gamepad() {
        let mut b = Bindings::default();
        let toml = b.to_toml().replace("cross = [\"Space\"]", "cross = [\"KeyG\"]");
        b = Bindings::from_toml(&toml).unwrap();
        let mouse = ButtonInput::<MouseButton>::default();
        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::Space);
        assert_eq!(sample(&b, &keys, &mouse, Vec2::ZERO, None).buttons & BTN_CROSS, 0, "Space no longer bound");
        // a tap pressed and released within one frame is not lost
        keys.press(KeyCode::KeyG);
        keys.release(KeyCode::KeyG);
        assert_ne!(sample(&b, &keys, &mouse, Vec2::ZERO, None).buttons & BTN_CROSS, 0);
        let mut g = Gamepad::default();
        g.digital_mut().press(GamepadButton::Start);
        g.analog_mut().set(GamepadAxis::LeftStickY, 0.75);
        let v = sample(&b, &ButtonInput::default(), &mouse, Vec2::ZERO, Some(&g));
        assert_ne!(v.buttons & BTN_START, 0, "digital-only Start");
        assert_eq!(v.sticks[0], [0.0, 0.75]);
    }
}

/// R12: the in-game card path on the real Paris (Episode 1 hub, `--table 2`) save-variable table. A full
/// progress blob (every data byte patterned: mission / job state, coins, loot, gadgets all live in it) is saved
/// through the memory-card dialog ("Save Game" slot -> autosave notice 0x22 -> square -> 0x1a FUN_001abd40)
/// into the slot file and loads back byte-exact through FUN_001ad320 (only the FUN_001d84b0 `from_memcard`
/// bytes differ; they are 1 in the saved copy).
#[test]
fn ep1e_save_load_progress_roundtrip() {
    use sly2_recomp::engine::disc::descriptor::static_table;
    use sly2_recomp::engine::disc::iso::IsoImage;
    use sly2_recomp::engine::disc::mods::ModOverlay;
    use sly2_recomp::engine::disc::LevelLoader;
    use sly2_recomp::engine::game::flow::{self, StartPaths};
    use sly2_recomp::engine::game::save::{GameData, MemoryCard, SaveValue, SaveVars};
    use sly2_recomp::engine::input::pad::PadState;
    use sly2_recomp::engine::ui::card_app::{self, CardApp, Host};
    use sly2_recomp::engine::ui::settings::SavePaths;
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let root = if root.join("Sly 2 - Band of Thieves (USA).iso").exists() { root } else { std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/..")) };
    let Ok(iso) = IsoImage::open(root.join("Sly 2 - Band of Thieves (USA).iso")) else { return };
    let iso = std::sync::Arc::new(iso);
    let st = static_table(&iso.read_file("SCUS_973.16").unwrap()).unwrap();
    // `game --table 2`: level-table row 2 of the boot level (first static-table level row, 22+)
    let loader = LevelLoader::new(iso, ModOverlay::default());
    let boot = loader.load_blocking(*st.iter().skip(22).find(|d| d.size() != 0).unwrap()).unwrap();
    let lv = loader.load_blocking(boot.header.level_table.entries[2].desc).unwrap();
    let vars = SaveVars::from_block(&lv.header.block_3a18);
    let paths = SavePaths::FUN_0020e3c0(&vars);
    // level load: FUN_001d7c80(table, 0x3d4a60) stamps the live blob
    let mut live = GameData::default();
    vars.ee_sub_1d7c80(&mut live);
    assert_ne!(live.version, 0);
    for (i, b) in live.data.iter_mut().enumerate() {
        *b = (i as u32).wrapping_mul(2654435761).rotate_right(13) as u8;
    }
    vars.ee_sub_1d80c8(paths.dt.as_deref().unwrap(), SaveValue::Float(1234.5), &mut live);
    vars.ee_sub_1d80c8(paths.flags[sly2_recomp::engine::ui::settings::F_AUTOSAVE_WARNED].as_deref().unwrap(), SaveValue::Bool(false), &mut live);
    let dir = std::env::temp_dir().join(format!("sly2_ep1e_card_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let card = MemoryCard::new(&dir);
    let mut app = CardApp::default();
    let mut pad = PadState::new();
    let mut slot = 0usize;
    let mut now = 10.0f32;
    let step = |app: &mut CardApp, pad: &mut PadState, slot: &mut usize, live: &mut GameData, now: f32, f: &mut dyn FnMut(&mut Host)| {
        let mut h = Host { app, now, pad, card: &card, slot, vars: &vars, paths: &paths, data: live, vibration: 0, title: false, save_menu: true, purpose: None, out: Default::default() };
        f(&mut h);
        h.out
    };
    // "Save Game..." slot 2 (empty -> FUN_001ad3e0 -> autosave notice)
    step(&mut app, &mut pad, &mut slot, &mut live, now, &mut |h| h.run(|d, h| {
        d.FUN_001ad480(h, 1);
    }));
    assert_eq!((app.dlg.msg, slot), (0x22, 1));
    now += 0.5;
    // square acknowledges (FUN_001accc8 mask 0x80) -> 0x1a + save
    pad.buttons = 0x80;
    pad.pressed = 0x80;
    let out = step(&mut app, &mut pad, &mut slot, &mut live, now, &mut |h| h.run(|d, h| d.FUN_001accc8(h)));
    assert_eq!((app.dlg.msg, out.saved), (0x1a, Some(1)));
    now += 0.5;
    let out = step(&mut app, &mut pad, &mut slot, &mut live, now, &mut |h| h.run(|d, h| d.FUN_001accc8(h)));
    assert_eq!(app.dlg.msg, -1);
    assert!(out.close_all, "purpose 2 closes the menus after the save");
    assert!(card.slot_path(1).is_file());
    assert_eq!(card_app::slot_dt(&card, 1, &vars, &paths), 1234.5);
    // load back (FUN_001ad320) and compare every byte
    let back = flow::FUN_001ad320(&card, 1, &vars, StartPaths::new(&vars).dt.as_deref()).unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    let sym = vars.ee_sub_1d8720("from_memcard");
    let memcard: Vec<usize> = vars.recs.iter().filter(|r| r.sym as i32 == sym).map(|r| r.offset as usize).collect();
    assert!(!memcard.is_empty());
    assert_eq!((back.version, back.checksum), (live.version, live.checksum));
    for i in 0..live.data.len() {
        if memcard.contains(&i) {
            assert_eq!(back.data[i], 1, "from_memcard byte {i:#x}");
        } else {
            assert_eq!(back.data[i], live.data[i], "byte {i:#x}");
        }
    }
    // the autosave notice was acknowledged once (fAutosaveWarned set in the live blob and the file)
    assert_eq!(vars.ee_sub_1d7ed8(paths.flags[sly2_recomp::engine::ui::settings::F_AUTOSAVE_WARNED].as_deref().unwrap(), &back), SaveValue::Bool(true));
}
