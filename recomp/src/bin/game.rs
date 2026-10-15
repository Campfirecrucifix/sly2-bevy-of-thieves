//! Sly 2 playable app (feature `game`). Usage:
//!   game [path/to/Sly 2.iso] [--level <static-table row> | --table <level-table row>] [--width W --height H]
//!        [--screenshot <out.png> --frames N] [--mods <mods dir>]
//! (ISO else settings.toml `iso`, else the ISO next to the exe). Boots to the first boot level of the ELF static table even when the
//! renderer has nothing to draw yet (empty scene).
//!
//! Frame order mirrors main() FUN_001a7518 (engine::game::main_loop::MainStep): level transition ->
//! pad (PadPlugin, PreUpdate) -> clock -> world -> camera -> render (Bevy) -> frame end.

use bevy::prelude::*;
use bevy::window::{PresentMode, WindowMode, WindowResolution};
use sly2_recomp::engine::camera::bevy_sync::{sync_cm_camera, CmCamera};
use sly2_recomp::engine::camera::widescreen;
use sly2_recomp::engine::camera::{Cm, CmClock};
use sly2_recomp::engine::disc::descriptor::{self, Descriptor};
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::{poll_level_loads, LevelLoader};
use sly2_recomp::engine::audio::emitter::EmitterSystem;
use sly2_recomp::engine::audio::AudioSystem;
use sly2_recomp::engine::game::app::{self as japp, GameAudio, LevelGame, RumbleQueue, ScreenshotRequest};
use sly2_recomp::engine::game::clock::{self, Clock};
#[cfg(feature = "rt")]
use sly2_recomp::engine::game::settings::Lighting;
use sly2_recomp::engine::game::startup::{self, InitFn, StartupHost};
use sly2_recomp::engine::gfx::game::{capture_level_data, spawn_level_geometry};
use sly2_recomp::engine::game::boot::{BootHost, BootQueue};
use sly2_recomp::engine::game::fmv::MovieQueue;
use sly2_recomp::engine::game::fmv_app::{no_movie, FmvPlayback, FmvPlugin};
use sly2_recomp::engine::game::main_loop::MainState;
use sly2_recomp::engine::game::save::MemoryCard;
use sly2_recomp::engine::game::settings::{exe_dir, Settings};
use sly2_recomp::engine::game::transition::{ee_str_oidPoCurrent_20e890, LevelHost, LevelRequest, TransitionState};
use sly2_recomp::engine::input::backend::{PadInput, PadPlugin};
use sly2_recomp::engine::world::{build_world, World, WorldLoadError, WorldOptions};
use std::sync::Arc;
use std::time::Instant;

/// First static-table row that is a level (rows 0-21 are FMVs).
const FIRST_LEVEL_ROW: usize = 22;

/// Engine globals driven by the main loop.
#[derive(Resource)]
struct GameLoop {
    clock: Clock,
    transition: TransitionState,
    main: MainState,
    movies: MovieQueue,
    start: Instant,
    #[allow(dead_code)]
    card: MemoryCard,
    world_reported: bool,
    /// WP-J15: boot transition queue 0x2e1e88
    boot: BootQueue,
    /// DAT_002e6594 (card manager 0x2e6550 +0x44 state; REPLACED: only the boot queue writes it)
    card_state: i32,
}

/// WP-J15: FUN_001a4e80 host: movies go to the 0x2f66b0 queue, requests to the FUN_0022b490 mailbox.
struct BinBootHost<'a> {
    movies: &'a mut MovieQueue,
    card_state: &'a mut i32,
}

impl BootHost for BinBootHost<'_> {
    fn ee_sub_1b3a48(&mut self, desc: u32) {
        // FUN_001b3a48: max 4 movies, descriptor with size != 0
        if let Some(d) = sly2_recomp::engine::game::flow::static_desc(desc).filter(|d| d.size() != 0) {
            if self.movies.movies.len() < 4 {
                self.movies.movies.push(d);
            }
        }
    }
    fn card_state(&self) -> i32 {
        *self.card_state
    }
    fn ee_sub_1acc38(&mut self, s: i32) {
        *self.card_state = s;
    }
}

#[derive(Resource)]
struct AppSettings(Settings);

/// FUN_0020e890 host for the Bevy app: the disc read/parse is started as an async task (WP-A) and the
/// world is built by WP-B's `build_world` when it completes (instead of blocking inside the frame).
struct AsyncHost<'a, 'w, 's> {
    commands: &'a mut Commands<'w, 's>,
    loader: &'a LevelLoader,
    movies: &'a mut MovieQueue,
    /// WP-J16: PSS playback (replaces SkipFmvPlayer)
    fmv: &'a mut FmvPlayback,
}

impl LevelHost for AsyncHost<'_, '_, '_> {
    fn load_level(&mut self, desc: &Descriptor) -> Result<(), String> {
        info!("level transition: loading lsn {} ({} bytes)", desc.lsn(), desc.size());
        self.commands.spawn(self.loader.start(*desc));
        Ok(())
    }
    fn clear_world(&mut self) {
        self.commands.remove_resource::<World>();
        self.commands.remove_resource::<WorldLoadError>();
    }
    fn play_queued_movies(&mut self) -> bool {
        self.movies.ee_sub_1b3b08(u32::MAX, self.fmv)
    }
}

use sly2_recomp::engine::game::vm_state::SpliceState;

fn main() {
    let settings_path = Settings::default_path();
    let mut settings = Settings::load_or_create(&settings_path).unwrap_or_else(|e| {
        eprintln!("{}: {e}; using defaults", settings_path.display());
        Settings::default()
    });
    let args = Args::parse(&mut settings);
    let iso_path = settings.iso_path();
    let iso = match IsoImage::open(&iso_path) {
        Ok(i) => Arc::new(i),
        Err(e) => {
            eprintln!("cannot open ISO {}: {e}\nusage: game <path to Sly 2 - Band of Thieves (USA).iso>", iso_path.display());
            std::process::exit(1);
        }
    };
    // WP-PUB2: the engine's static tables are read from this ISO's SCUS_973.16 on first use
    sly2_recomp::engine::disc::exe::set_iso_path(&iso_path);
    // WIRE-C2: `--mods <dir>` replaces the mods folder next to the exe (render tests / mod authors)
    let mods_dir = args.mods.clone().unwrap_or_else(|| exe_dir().join("mods"));
    let mods = settings.mod_overlay(&mods_dir).unwrap_or_else(|e| {
        eprintln!("mods: {e}; running without mods");
        Default::default()
    });
    let table = iso.read_file("SCUS_973.16").ok().and_then(|elf| descriptor::static_table(&elf)).unwrap_or_default();
    let mut movies = MovieQueue { special: [table.get(28).copied(), table.get(29).copied()], ..Default::default() };
    // WP-J15: static descriptors by address (boot queue rows, hideout 0x2a3828)
    sly2_recomp::engine::game::flow::set_static_table(table.clone());
    // WP-J15: without --level / --table the 1:1 boot FUN_001613a8 -> FUN_001a4e08 (intro movie, card check, level 0)
    let use_boot_queue = args.level.is_none() && args.table.is_none();
    let boot = match args.level {
        Some(r) => match table.get(r) {
            // WP-J3: refuse non-level rows (movies like row 24, data, empty rows)
            Some(&d) => match sly2_recomp::engine::game::level_check::check_level_row(&LevelLoader::new(iso.clone(), mods.clone()), d) {
                Ok(()) => Some(d),
                Err(e) => {
                    eprintln!("--level {r}: static-table row {r} rejected: {e}");
                    std::process::exit(2);
                }
            },
            None => {
                eprintln!("--level {r}: static table has only {} rows", table.len());
                std::process::exit(2);
            }
        },
        None => table.iter().skip(FIRST_LEVEL_ROW).find(|d| d.size() != 0).copied(),
    };
    let loader = LevelLoader::new(iso, mods);
    // --table: row of the FUN_0020f318 level table embedded in the boot level (startup only, so blocking is fine)
    let boot = match (args.table, boot) {
        (Some(r), Some(b)) => match loader.load_blocking(b) {
            Ok(l) => match l.header.level_table.entries.get(r).map(|e| e.desc) {
                // WP-J3: refuse non-level rows instead of silently falling back
                Some(d) => match sly2_recomp::engine::game::level_check::check_level_row(&loader, d) {
                    Ok(()) => Some(d),
                    Err(e) => {
                        eprintln!("--table {r}: level-table row {r} rejected: {e}");
                        std::process::exit(2);
                    }
                },
                None => {
                    eprintln!("--table {r}: level table has no row {r} ({} rows)", l.header.level_table.entries.len());
                    std::process::exit(2);
                }
            },
            Err(e) => {
                eprintln!("boot level read: {e}");
                Some(b)
            }
        },
        (_, b) => b,
    };

    let mut transition = TransitionState::default();
    let mut boot_queue = BootQueue::default();
    let mut card_state = 0;
    if use_boot_queue && !table.is_empty() {
        boot_queue.FUN_001a4e08(&mut BinBootHost { movies: &mut movies, card_state: &mut card_state });
    } else {
        match boot {
            Some(d) => transition.request(LevelRequest::new(d)),
            None => eprintln!("no boot level found in SCUS_973.16; starting with an empty scene"),
        }
    }

    let present_mode = if settings.vsync { PresentMode::AutoVsync } else { PresentMode::AutoNoVsync };
    let mode = if settings.fullscreen { WindowMode::BorderlessFullscreen(MonitorSelection::Primary) } else { WindowMode::Windowed };
    let window = Window {
        title: "Sly 2: Band of Thieves (recomp)".into(),
        resolution: WindowResolution::new(settings.width, settings.height),
        present_mode,
        mode,
        ..default()
    };

    // WP-R: global illumination at startup (raster without feature `rt`); A12: the Graphics menu switches it live
    #[cfg(feature = "rt")]
    let raytraced = settings.effective_lighting() == Lighting::Raytraced;
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin { primary_window: Some(window), ..default() }))
        .add_plugins(PadPlugin)
        .add_plugins(sly2_recomp::engine::gfx::ultrawide::UltrawidePlugin)
        .add_plugins(sly2_recomp::engine::gfx::game::GfxPlugin)
        .add_plugins(sly2_recomp::engine::player::plugin::PlayerPlugin)
        .add_plugins(sly2_recomp::engine::playerdraw::plugin::PlayerDrawPlugin)
        .add_plugins(sly2_recomp::engine::classes::plugin::ClassesPlugin)
        .add_plugins(sly2_recomp::engine::npcdraw::plugin::NpcDrawPlugin)
        .add_plugins(sly2_recomp::engine::ui::plugin::UiPlugin)
        .add_plugins(japp::ScriptClassesPlugin)
        .add_plugins(sly2_recomp::engine::game::arch1_bridge::Arch1BridgePlugin)
        .add_plugins(sly2_recomp::engine::game::flow_app::GameFlowPlugin)
        .add_plugins(FmvPlugin)
        .insert_resource(sly2_recomp::engine::gfx::game::GfxSettings { ink_outlines: settings.ink_outlines })
        .insert_resource(loader)
        .insert_resource(WorldOptions::default())
        .insert_resource(Cm::new())
        .insert_resource(AudioSystem::new())
        .insert_resource(EmitterSystem::default())
        // WP-EP1e: replaced 989snd client state + the verification monitor (env SLY2_AUDIO_LOG=1)
        .insert_resource(sly2_recomp::engine::audio::emitter_tick::Snd989::default())
        .insert_resource(sly2_recomp::engine::audio::bevy_out::AudioMonitor { on: std::env::var("SLY2_AUDIO_LOG").is_ok_and(|v| v == "1"), ..default() })
        .insert_resource(LevelGame { spawn_request: (-1, -1), ..default() })
        .insert_resource(GameAudio::new(settings.master_volume * settings.music_volume, settings.master_volume * settings.sfx_volume))
        .insert_resource(RumbleQueue::default())
        .insert_resource(GameLoop {
            clock: Clock::default(),
            transition,
            main: MainState::default(),
            movies,
            start: Instant::now(),
            card: MemoryCard::next_to_exe(),
            world_reported: false,
            boot: boot_queue,
            card_state,
        })
        .insert_resource(AppSettings(settings))
        .insert_non_send(SpliceState::default())
        .add_systems(Startup, (startup_sequence, setup).chain())
        .add_systems(
            Update,
            (
                (level_transition, poll_level_loads, capture_level_data, japp::capture_level_game, build_world, japp::activate_world, feed_player_level, feed_classes_level, level_scripts).chain(),
                (clock_step, tick_scripts.after(japp::lend_classes)).chain(),
                (report_world, spawn_level_geometry, japp::frame_camera, hor_plus, sync_cm_camera).chain(),
                (japp::start_level_audio, japp::poll_level_audio, japp::update_music_layers, sly2_recomp::engine::disc::mods_sound::apply_sound_mods_system, japp::tick_sound_emitters, sly2_recomp::engine::audio::bevy_out::drain_snd_cmds, japp::play_sound_events, japp::apply_rumble, japp::audio_monitor).chain(),
                frame_end,
            )
                .chain()
                // WP-J16: FUN_001b3b08 blocks the main loop while movies play
                .run_if(no_movie),
        );
    #[cfg(feature = "rt")]
    app.add_plugins(sly2_recomp::engine::gfx::rt::RtPlugin { raytraced });
    // WP-EP1a (port addition, verification): env `SLY2_SHOTS="F:path;F:path..."` = screenshots at APP frames F
    // (same frame count as the `g` items of SLY2_PAD_SCRIPT); the app exits once the last one is saved.
    if let Some(sh) = std::env::var("SLY2_SHOTS").ok().map(|s| StepShots::parse(&s)).filter(|s| !s.0.is_empty()) {
        app.insert_resource(sh).add_systems(Last, step_shots);
    }
    if let Some((path, frames)) = args.screenshot {
        app.insert_resource(ScreenshotRequest::new(path, frames)).add_systems(Last, japp::screenshot_driver);
    }
    app.run();
}

/// Command line (beyond the ISO path).
struct Args {
    level: Option<usize>,
    table: Option<usize>,
    screenshot: Option<(std::path::PathBuf, u32)>,
    mods: Option<std::path::PathBuf>,
}

impl Args {
    fn parse(settings: &mut Settings) -> Args {
        let mut a = Args { level: None, table: None, screenshot: None, mods: None };
        let mut shot = None;
        let mut frames = 60;
        let mut it = std::env::args().skip(1);
        while let Some(x) = it.next() {
            let mut num = |what: &str| -> u32 {
                it.next().and_then(|v| v.parse().ok()).unwrap_or_else(|| {
                    eprintln!("{what} needs a number");
                    std::process::exit(2)
                })
            };
            match x.as_str() {
                "--level" => a.level = Some(num("--level") as usize),
                "--table" => a.table = Some(num("--table") as usize),
                "--frames" => frames = num("--frames"),
                "--width" => settings.width = num("--width"),
                "--height" => settings.height = num("--height"),
                "--mods" => a.mods = it.next().map(std::path::PathBuf::from),
                "--screenshot" => shot = it.next().map(std::path::PathBuf::from),
                _ if x.starts_with("--") => {
                    eprintln!("unknown option {x}");
                    std::process::exit(2)
                }
                _ => settings.iso = x,
            }
        }
        if shot.is_some() {
            // deterministic window for verification
            settings.fullscreen = false;
        }
        a.screenshot = shot.map(|p| (p, frames));
        a
    }
}

/// FUN_001a7970 host: pad init (FUN_00187030), sound globals (FUN_001fcea0); the rest is replaced by Bevy.
struct BevyStartup<'a> {
    pad: Option<&'a mut PadInput>,
}

impl StartupHost for BevyStartup<'_> {
    fn init_pad(&mut self) {
        if let Some(p) = self.pad.as_deref_mut() {
            // FUN_001870d0 + FUN_00187150(0x2dfc00, 0, 0)
            sly2_recomp::engine::input::pad::FUN_001870d0(&mut p.lock, &mut p.pad);
        }
    }
    fn replaced(&mut self, f: &InitFn) {
        debug!("startup {:#010x}: replaced ({})", f.addr, f.what);
    }
}

fn startup_sequence(mut g: ResMut<GameLoop>, pad: Option<ResMut<PadInput>>, mut ga: ResMut<GameAudio>) {
    let now = g.start.elapsed().as_secs_f64();
    let mut pad = pad;
    let mut host = BevyStartup { pad: pad.as_deref_mut() };
    let st = startup::ee_str_0813_0032_1a7970(&mut g.clock, now, &mut host);
    // FUN_001fcea0 master volume scales the configured volumes
    ga.music_volume *= st.sound.master;
    ga.sfx_volume *= st.sound.master;
}

fn setup(mut commands: Commands, s: Res<AppSettings>) {
    let lighting = s.0.effective_lighting();
    info!("lighting: {lighting:?}");
    // WP-R / A12: gfx::rt::rt_switch adds / removes SolariLighting on this camera whenever RtState::requested
    // changes (settings.toml at start, then the Graphics menu); stays raster when the GPU lacks ray tracing.
    // WP-C9: no stand-in sun / fixed ambient: the level's LIGHT objects set both (gfx::game::spawn_level_lights,
    // light::level_ambient). Tonemapping::None: the PS2 output is a plain clamp of texel * vcol * L (no filmic curve).
    // The startup 600 nits stay only as the reference for the RT baked-light fill / sky (gfx::rt, R2
    // tuning, read before any level exists); spawn_level_lights replaces it per level.
    commands.insert_resource(GlobalAmbientLight { color: Color::WHITE, brightness: 600.0, ..default() });
    commands.spawn((Camera3d::default(), CmCamera, Transform::default(), bevy::core_pipeline::tonemapping::Tonemapping::None));
}

/// MainStep::LevelTransition (FUN_0020e890).
fn level_transition(mut commands: Commands, loader: Res<LevelLoader>, mut g: ResMut<GameLoop>, mut lg: ResMut<LevelGame>, mut fr: ResMut<sly2_recomp::engine::game::flow_app::FlowRequests>, mut fmv: ResMut<FmvPlayback>, (mut wipe, w, ui): (ResMut<sly2_recomp::engine::gfx::screen::ScreenWipe>, Option<Res<World>>, Option<ResMut<sly2_recomp::engine::ui::plugin::UiGame>>)) {
    let g = &mut *g;
    // WP-J15: FUN_001660f8 (quit to title): the boot queue again
    if sly2_recomp::engine::game::boot::take_restart() {
        g.boot.FUN_001a4e08(&mut BinBootHost { movies: &mut g.movies, card_state: &mut g.card_state });
        fr.0.extend(sly2_recomp::engine::game::flow::take_posted());
    }
    // WP-J14: FUN_0022b490 requests (title, death reload, job restarts)
    // WP-C8: FUN_0022b490 tail: a != -1 goes through the 0x443740 screen wipe (gfx::screen), which
    // releases the request once it has closed
    for p in std::mem::take(&mut fr.0) {
        wipe.request(w.as_deref(), p);
    }
    for p in std::mem::take(&mut wipe.posted) {
        match sly2_recomp::engine::game::flow::resolve(&p, lg.table.as_ref(), g.transition.current) {
            Some(r) => g.transition.request(r),
            None => warn!("level transition: unresolved request {p:?}"),
        }
    }
    let mut host = AsyncHost { commands: &mut commands, loader: &loader, movies: &mut g.movies, fmv: &mut fmv };
    let o = ee_str_oidPoCurrent_20e890(&mut g.transition, &mut g.clock, &mut host);
    if !matches!(o, sly2_recomp::engine::game::transition::TransitionOutcome::Idle) {
        info!("level transition: {o:?}");
        // world+0x3c38/+0x3c3c: FUN_00227fe8 runs once the async world build finished (japp::activate_world)
        if let Some(r) = g.transition.request {
            lg.spawn_request = (r.spawn, r.spawn_parent);
        }
        g.world_reported = false;
        // WP-C8: FUN_0022b420 (wipe re-found in the new level). WP-C9: FUN_001618f8 is the HUD countdown
        // (world+0x3bc4 = slot 3 of world+0x3ba0, oid 0x36f), not a screen fade: the carried-over
        // seconds restart that countdown in the new level
        if let sly2_recomp::engine::game::transition::TransitionOutcome::Loaded { fade, .. } = o {
            if let Some(mut ui) = ui {
                ui.countdown_carry = (fade > 0.0).then_some(fade);
            }
            wipe.level_load_started(w.as_deref());
        }
        if let sly2_recomp::engine::game::transition::TransitionOutcome::OpenFailed(_) = &o {
            wipe.level_loaded(w.as_deref()); // stay on the old level: re-open the wipe
        }
        // WP-J15: FUN_0020e890 tail: a size-0 descriptor pops the next boot queue entry (FUN_001a4e80(0x2e1e88))
        if matches!(o, sly2_recomp::engine::game::transition::TransitionOutcome::Loaded { reset_only: true, .. }) {
            g.boot.FUN_001a4e80(&mut BinBootHost { movies: &mut g.movies, card_state: &mut g.card_state });
        }
    }
}

/// WP-J3: level script blocks (FUN_001ffa70) once the world is active.
fn level_scripts(g: Res<GameLoop>, mut lg: ResMut<LevelGame>, mut st: NonSendMut<SpliceState>, w: Option<Res<World>>, cm: Res<Cm>, mut em: ResMut<EmitterSystem>, mut rq: ResMut<RumbleQueue>, opt: Res<WorldOptions>, mut spawn_lists: ResMut<sly2_recomp::engine::player::plugin::PlayerSpawnLists>, mut live_q: ResMut<sly2_recomp::engine::classes::q::live::LiveQ>) {
    let Some(w) = w else { return };
    japp::run_level_scripts(g.clock.t, &mut lg, &mut st, &w, Some(&cm), &mut em, &mut rq, opt.0.mission, &mut spawn_lists.0, Some(&mut live_q));
}

/// WP-J4: MainStep::WorldUpdate (FUN_00226550) script work: due Splice callbacks, after the clock step.
#[allow(clippy::too_many_arguments)]
fn tick_scripts(g: Res<GameLoop>, mut lg: ResMut<LevelGame>, mut st: NonSendMut<SpliceState>, w: Option<Res<World>>, mut cm: ResMut<Cm>, mut em: ResMut<EmitterSystem>, mut rq: ResMut<RumbleQueue>, opt: Res<WorldOptions>, mut spawn_lists: ResMut<sly2_recomp::engine::player::plugin::PlayerSpawnLists>, mut live_q: ResMut<sly2_recomp::engine::classes::q::live::LiveQ>) {
    let Some(w) = w else { return };
    let clk = CmClock { t: g.clock.t, dt: g.clock.dt, dt_real: g.clock.dt_real };
    japp::tick_scripts(g.clock.t, g.clock.dt, clk, &mut lg, &mut st, &w, &mut cm, &mut em, &mut rq, opt.0.mission, &mut spawn_lists.0, Some(&mut live_q));
}

/// MainStep::Clock (FUN_0014d0b8(g_clock, 0)).
fn clock_step(mut g: ResMut<GameLoop>, hud: Option<Res<sly2_recomp::engine::player::plugin::PlayerHud>>, mut fc: ResMut<sly2_recomp::engine::game::flow_app::FrameClock>) {
    // WP-J14: time-scale gadgets FUN_0014d1d0(scale, g_clock, 1)
    if let Some(h) = hud {
        sly2_recomp::engine::game::flow::apply_time_scale(&mut g.clock, h.gadgets.time_scale);
    }
    let now = g.start.elapsed().as_secs_f64();
    let advanced = clock::ee_sub_14d0b8(&mut g.clock, now, false);
    *fc = sly2_recomp::engine::game::flow_app::FrameClock { t: g.clock.t, dt: g.clock.dt, advanced };
}

fn report_world(mut g: ResMut<GameLoop>, w: Option<Res<World>>, err: Option<Res<WorldLoadError>>) {
    if g.world_reported {
        return;
    }
    if let Some(w) = w {
        info!("world loaded: {} objects, {} entries ({} unported classes)", w.objects.len(), w.entries.len(), w.unported_classes().len());
        g.world_reported = true;
    } else if let Some(e) = err {
        error!("world load failed: {}", e.0);
        g.world_reported = true;
    }
}

/// Hor+ for the current window aspect (WP-I widescreen). A11: the options Graphics page switches it live.
fn hor_plus(mut cm: ResMut<Cm>, g: Res<GameLoop>, s: Res<AppSettings>, windows: Query<&Window>, ui: Option<Res<sly2_recomp::engine::ui::plugin::UiGame>>) {
    let Ok(win) = windows.single() else { return };
    let clk = CmClock { t: g.clock.t, dt: g.clock.dt, dt_real: g.clock.dt_real };
    if ui.map_or(s.0.hor_plus, |u| u.menu.gfx.hor_plus) {
        widescreen::apply_hor_plus(&mut cm, win.width(), win.height(), clk);
    } else {
        widescreen::apply_original(&mut cm, clk);
    }
}

/// MainStep::FrameEnd: DAT_002be538++.
fn frame_end(mut g: ResMut<GameLoop>) {
    g.main.frame = g.main.frame.wrapping_add(1);
}

/// WP-P2 wiring: hand the active world's collision + spawn point to the player (once per world).
fn feed_player_level(w: Option<Res<World>>, lg: Res<LevelGame>, mut pl: ResMut<sly2_recomp::engine::player::plugin::PlayerLevel>, mut done: Local<Option<usize>>) {
    let Some(w) = w else { return };
    // WP-EP1c: keyed by the load count, not the World address (a same-level reload reuses the resource slot,
    // so the death reload never re-fed the spawn / collision and Sly kept dying in the old JT)
    let key = lg.loads as usize;
    if *done == Some(key) || lg.spawn.is_none() { return; }
    *done = Some(key);
    pl.cw = std::sync::Arc::new(sly2_recomp::engine::physics::CollisionWorld::build(&w));
    pl.spawn = lg.spawn.and_then(|i| sly2_recomp::engine::game::spawn::spawn_position(&w, i)).map(Into::into);
    info!("player: spawn {:?}", pl.spawn);
}

/// WP-K2 wiring: hand the active world to the class overrides (rebuilt once per world).
fn feed_classes_level(w: Option<Res<World>>, mut cl: ResMut<sly2_recomp::engine::classes::plugin::ClassesLevel>, mut done: Local<usize>) {
    let Some(w) = w else { return };
    let key = &*w as *const World as usize;
    // WP-EP1c: a same-level reload reuses the World address; `is_changed` catches it
    if *done == key && !w.is_changed() { return; }
    *done = key;
    cl.0 = Some(std::sync::Arc::new((*w).clone()));
}

/// WP-EP1a: SLY2_SHOTS queue (app frame, path), frame counter, pending save.
#[derive(Resource, Default)]
struct StepShots(Vec<(u32, std::path::PathBuf)>, u32, Option<(std::path::PathBuf, u32)>);

impl StepShots {
    fn parse(s: &str) -> Self {
        let mut v: Vec<(u32, std::path::PathBuf)> = s
            .split(';')
            .filter_map(|it| it.trim().split_once(':').and_then(|(f, p)| Some((f.trim().parse().ok()?, p.trim().into()))))
            .collect();
        v.sort_by_key(|x| x.0);
        StepShots(v, 0, None)
    }
}

fn step_shots(mut sh: ResMut<StepShots>, mut commands: Commands, mut exit: MessageWriter<AppExit>) {
    use bevy::render::view::screenshot::{save_to_disk, Screenshot};
    let f = sh.1;
    sh.1 += 1;
    if let Some((p, t)) = sh.2.take() {
        if !p.exists() && f < t + 600 {
            sh.2 = Some((p, t));
            return;
        }
        info!("shot f{f}: {} ({})", p.display(), if p.exists() { "saved" } else { "NOT saved" });
        if sh.0.is_empty() {
            exit.write(AppExit::Success);
        }
    }
    if sh.0.first().is_some_and(|x| x.0 <= f) {
        let (_, p) = sh.0.remove(0);
        if let Some(d) = p.parent() {
            let _ = std::fs::create_dir_all(d);
        }
        let _ = std::fs::remove_file(&p);
        commands.spawn(Screenshot::primary_window()).observe(save_to_disk(p.clone()));
        sh.2 = Some((p, f));
    }
}
