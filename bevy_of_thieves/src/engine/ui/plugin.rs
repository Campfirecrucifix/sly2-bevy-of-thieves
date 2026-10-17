//! WP-I2: Bevy side of the UI. Replaces the GS sprite output of FUN_0015f480 / FUN_001cd5f0 with 2D sprites
//! on an overlay `Camera2d` (order 10, no clear) using the disc's own font textures. Every quad goes through
//! `HudLayout::map_text_rect`, so the PS2 layout sits in the centred 16:9 rect at any aspect.
//!
//! Port addition (debug / verification): env `SLY2_UI_CAPTION="text"` sets `UiCaption` at startup, which
//! shows the FUN_001b3700 text panel and overrides the string-table text.
//!
//! WP-I3: `UiGame` holds the level strings / string table 0x3e1c20 (captured from the level header before
//! `build_world`), the binding table 0x2a8b90 and the menu manager 0x3d9ed0. `ui_update` is FUN_001d9648's
//! pause / menu part (START -> FUN_001b0290, then FUN_001aeee0) plus the title screen 0x3da530
//! (FUN_001dd920 START / SELECT). `draw_ui` draws caption + HUD counters + menu / title (`ui::hud`).
//! Debug / verification hooks (port additions): `SLY2_UI_CAPTION_ID=<id>` shows the panel for a string id
//! (FUN_00210ca8), `SLY2_UI_PAUSE=1` opens the pause menu once a level is loaded, `SLY2_UI_TITLE=1` starts
//! with the title screen active, `SLY2_UI_HUD=0` hides the counters.
//!
//! WP-I4: option effects (FUN_001afb88 / FUN_001d8ff8 on `LevelGame::data`, mixer levels -> `AudioSystem`),
//! quit -> title screen, coin counter from the FUN_0020e3c0 path ["loot" "ccoin"], and the port's "Graphics"
//! page backed by settings.toml `[video]` (ink outlines live; global illumination / Hor+ saved for the next
//! start). `SLY2_UI_MENU=<n>` (verification) opens the pause menu and selects row n, `SLY2_UI_GFX=1` opens the
//! options menu on its Graphics row, `=2` also the Graphics page.
//!
//! WP-A11: title input = `ui::title::FUN_001dd920` (START -> FUN_001ddb08 new game -> FUN_00165e38),
//! port option pages Display / Sound Levels / Controls (Keys) / Mods (`ui::options`, applied by
//! `port_options`: window resolution / mode / vsync live, settings.toml + mods/load_order.txt written),
//! Hor+ applied live (bin/game.rs reads `menu.gfx.hor_plus`), mouse in menus (`port_mouse`: left click =
//! cross, right click = triangle, wheel = up / down). Verification: `SLY2_UI_GFX=3/4/5/6` opens the
//! Display / Sound Levels / Controls / Mods page, `=7` the Graphics page with GI flipped (note row shown,
//! nothing saved).
//!
//! WP-A12: the Graphics page GI row switches GI LIVE (`sync_gi` -> `gfx::rt::RtState::requested`, applied by
//! `gfx::rt::rt_switch` in rt builds); without `rt` / a ray-tracing GPU a greyed info row says why.
//! Verification: `SLY2_GI_AT=<frame>:<on|off>,...` (frames since start) flips the GI choice like the menu.
use crate::engine::gfx::rt::{gi_schedule, gi_unavailable_reason, RtState};
use super::bind::{self, BindCtx, Binding};
use super::caption::{caption, Caption};
use super::font::Fonts;
use super::hud::{self, HudPlayer, UiFrame};
use super::menu::{MenuCtx, MenuEvent, MenuMgr};
use super::menu::{PORT_AUD_OPEN, PORT_CTL_OPEN, PORT_GFX_ID, PORT_MOD_OPEN, PORT_VID_OPEN};
use super::options::{controls_rows, AudioOpts, ModsOpts, VideoOpts};
use super::settings::{self as opt, GraphicsOpts, SavePaths, SavedOpts};
use super::title::{self, TitleInput};
use crate::engine::audio::AudioSystem;
use crate::engine::game::app::LevelGame;
use crate::engine::game::settings::{Lighting, Settings};
use crate::engine::gfx::game::GfxSettings;
use super::strings::{LevelStrings, StringTable};
use crate::engine::camera::hud::{Anchor, HudLayout};
use crate::engine::disc::{LevelLoadResult, LevelLoader};
use crate::engine::gfx::ultrawide::HudSafeRect;
use crate::engine::input::backend::PadInput;
use crate::engine::input::pad::{self, Clock};
use crate::engine::player::plugin::Player;
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use std::sync::Arc;

/// The ported fonts and their Bevy textures (index = `Font::tex`).
#[derive(Resource)]
pub struct UiFonts {
    pub fonts: Fonts,
    pub images: Vec<Handle<Image>>,
}

/// Text of the FUN_001b3700 panel (None = hidden). Overrides `UiCaptionId`.
#[derive(Resource, Default, Clone, Debug, PartialEq)]
pub struct UiCaption(pub Option<String>);

/// The panel owner's +0x104 string id (None = hidden); text = FUN_00210ca8(0x3e1c20, id).
#[derive(Resource, Default, Clone, Copy, Debug, PartialEq)]
pub struct UiCaptionId(pub Option<i32>);

/// WP-I3 UI state (see module doc).
#[derive(Resource)]
pub struct UiGame {
    pub strings: LevelStrings,
    pub table: StringTable,
    pub bindings: Vec<Binding>,
    pub menu: MenuMgr,
    /// A12: HUD meter object 0x3d9e80 (FUN_001d9968 again after each level load)
    pub meter: super::meter::HudMeter,
    pub meter_init: bool,
    /// A12: binocucom HUD sub-object (+0x24) and the last state the JT side requested (binocucom +0x28)
    pub binoc: super::binoc_hud::BinocHud,
    pub binoc_req: i32,
    /// 0x3da530 +0x24 (title / episode screen active)
    pub title: bool,
    pub show_hud: bool,
    /// SLY2_UI_PAUSE pending
    pub open_pause: bool,
    /// WP-I4: FUN_0020e3c0 save-variable paths of the current level
    pub paths: SavePaths,
    /// settings.toml (None = not found / not parsable: graphics changes are not written)
    pub settings: Option<(std::path::PathBuf, Settings)>,
    /// SLY2_UI_MENU row / SLY2_UI_GFX (verification)
    pub open_row: Option<i32>,
    pub open_gfx: Option<i32>,
    /// A11: Display / Sound Levels / Mods changes for `port_options`
    pub port_events: Vec<MenuEvent>,
    /// A13: the level's class-131 episode select (DAT_003e1c04), built for world `episode_world`
    pub episode: Option<super::episode::Ep131>,
    pub episode_world: usize,
    /// A13: sequence action 0x44 count already turned into `title` (classes::plugin `Classes::host`)
    pub title_shows_seen: u32,
    /// A14: the four world+0x3ba0 HUD slots and their labels; last seen classes-host FUN_00161868 /
    /// FUN_001618b0 counts
    pub slots: super::hud_slots::HudSlots,
    pub slot_feed: (u32, u32),
    /// WP-C9: FUN_0020e890 tail `FUN_001618f8(world, (int)(+0x476c - t))`: countdown seconds carried over a
    /// level load, started once the new level's slots exist (after FUN_0020f710)
    pub countdown_carry: Option<f32>,
    /// WP-K33: last seen `EngineHost::hud_timer` request count
    pub timer_feed: u32,
    /// WP-UI4: vault / safe HUD object 0x2e1e60 (FUN_001a3078 / FUN_001a30d0) shown flag + last seen
    /// `EngineHost::hud_vault` request count (class-78 vault +0x3d80 edges)
    pub hud_vault: bool,
    pub vault_feed: u32,
    /// A14: DAT_002f6334 (card slot chosen on the load menu, 1-based; 0 = none -> START = New Game)
    pub card_slot: i32,
    /// A14: port load menu over the title (REPLACES the card manager's slot choice)
    pub load: super::title::LoadMenu,
    /// A14: the boot blob, restored when the load menu picks New Game after a slot was loaded
    pub fresh_blob: Option<crate::engine::game::save::GameData>,
    /// A14: FUN_001ddbf0 title text of this frame
    pub title_text: Vec<u8>,
    /// A14: 0x2f66b0 movie queue of the UI / class requests (FUN_001b3a48), handed to J16's FmvPlayback
    pub movies: crate::engine::game::fmv::MovieQueue,
}

#[derive(Component)]
pub struct UiSprite;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        let cap = std::env::var("SLY2_UI_CAPTION").ok().filter(|s| !s.is_empty());
        let cap_id = std::env::var("SLY2_UI_CAPTION_ID").ok().and_then(|s| parse_int(&s));
        let flag = |k: &str| std::env::var(k).ok().is_some_and(|v| !v.is_empty() && v != "0");
        app.insert_resource(UiCaption(cap))
            .insert_resource(UiCaptionId(cap_id))
            .insert_resource(UiGame {
                strings: LevelStrings::default(),
                table: StringTable::FUN_00210cf0(),
                bindings: Vec::new(),
                menu: MenuMgr::new(None),
                meter: Default::default(),
                meter_init: false,
                binoc: Default::default(),
                binoc_req: -1,
                title: flag("SLY2_UI_TITLE"),
                show_hud: std::env::var("SLY2_UI_HUD").map_or(true, |v| v != "0"),
                open_pause: flag("SLY2_UI_PAUSE") || std::env::var("SLY2_UI_MENU").is_ok() || flag("SLY2_UI_GFX"),
                paths: SavePaths::default(),
                settings: load_settings(),
                open_row: std::env::var("SLY2_UI_MENU").ok().and_then(|s| parse_int(&s)),
                open_gfx: std::env::var("SLY2_UI_GFX").ok().and_then(|s| parse_int(&s)).filter(|&v| v != 0),
                port_events: Vec::new(),
                episode: None,
                episode_world: 0,
                title_shows_seen: 0,
                slots: Default::default(),
                slot_feed: (0, 0),
                countdown_carry: None,
                timer_feed: 0,
                hud_vault: false,
                vault_feed: 0,
                card_slot: 0,
                load: Default::default(),
                fresh_blob: None,
                title_text: Vec::new(),
                movies: Default::default(),
            })
            // A14: title START / load menu before J16's `title_continue` (which would always continue a slot)
            .add_systems(
                RunFixedMainLoop,
                title_input
                    .in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop)
                    .before(crate::engine::game::flow_app::title_continue),
            )
            .add_systems(Update, class_movies.after(crate::engine::classes::plugin::tick_classes).before(crate::engine::game::app::lend_classes).before(episode_update))
            .add_systems(Startup, (load_fonts, init_graphics_opts))
            .add_systems(Update, (capture_strings.after(crate::engine::disc::poll_level_loads).before(crate::engine::world::build_world), (port_mouse, ui_update, title_from_sequence.after(crate::engine::classes::plugin::tick_classes).before(crate::engine::game::app::lend_classes), episode_update, port_options, hud_meter, draw_ui).chain()))
            // A11: draw_ui in Update (was PostUpdate): Bevy's sprite_mesh `add_mesh` (PostUpdate, unordered)
            // queued `insert(Mesh2d)` on sprites draw_ui despawned in the same frame -> panic with any open menu
            .add_systems(Update, sync_overlay_camera)
            .init_resource::<RtState>()
            .add_systems(Update, sync_gi.after(port_options));
    }
}

/// The HUD overlay camera.
#[derive(Component)]
pub struct UiCamera;

/// A7: the overlay camera takes the 3D camera's Hdr / Msaa / main texture usages so both render into the same
/// main texture (Bevy shares it only when all three match). With raytraced GI the CM camera is Hdr +
/// Msaa::Off + a STORAGE_BINDING main texture (Solari); the overlay then had its own texture and its
/// translucent HUD boxes came out opaque black.
#[allow(clippy::type_complexity)]
pub fn sync_overlay_camera(
    mut commands: Commands,
    cm: Query<(Has<bevy::camera::Hdr>, Option<&Msaa>, Option<&bevy::camera::CameraMainTextureUsages>), (With<crate::engine::camera::bevy_sync::CmCamera>, Without<UiCamera>)>,
    ui: Query<(Entity, Has<bevy::camera::Hdr>, Option<&Msaa>, Option<&bevy::camera::CameraMainTextureUsages>), With<UiCamera>>,
) {
    let Some((hdr, msaa, usages)) = cm.iter().next() else { return };
    let msaa = msaa.copied().unwrap_or_default();
    let usages = usages.copied().unwrap_or_default();
    for (e, uhdr, umsaa, uusages) in &ui {
        let mut c = commands.entity(e);
        if uhdr != hdr {
            if hdr {
                c.insert(bevy::camera::Hdr);
            } else {
                c.remove::<bevy::camera::Hdr>();
            }
        }
        if umsaa.copied().unwrap_or_default() != msaa {
            c.insert(msaa);
        }
        if uusages.copied().unwrap_or_default().0 != usages.0 {
            c.insert(usages);
        }
    }
}

/// FUN_0015ef00 at boot: fonts from SCUS_973.16, textures uploaded once. WP-I3: FUN_0021aa60 binding table
/// and the menu item tables come from the same image.
fn load_fonts(mut commands: Commands, loader: Option<Res<LevelLoader>>, mut images: ResMut<Assets<Image>>, mut game: ResMut<UiGame>) {
    commands.spawn((Camera2d, Camera { order: 10, clear_color: ClearColorConfig::None, ..default() }, UiCamera));
    let Some(loader) = loader else { return };
    let elf = loader.iso.read_file("SCUS_973.16").ok().map(Arc::new);
    if let Some(elf) = &elf {
        game.bindings = bind::FUN_0021aa60(|va, n| bind::elf_read(elf, va, n)).unwrap_or_default();
        game.menu.elf = Some(elf.clone());
    }
    let fonts = match elf.as_deref().and_then(|elf| Fonts::from_elf(elf)) {
        Some(f) => f,
        None => {
            warn!("ui: fonts not found in SCUS_973.16; text disabled");
            return;
        }
    };
    let handles = fonts
        .images
        .iter()
        .map(|i| {
            images.add(Image::new(
                Extent3d { width: i.w, height: i.h, depth_or_array_layers: 1 },
                TextureDimension::D2,
                i.rgba.clone(),
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::RENDER_WORLD,
            ))
        })
        .collect();
    commands.insert_resource(UiFonts { fonts, images: handles });
}

/// GS colour (0x80 = 1.0) -> Bevy colour. RGB may exceed 1.0 (GS modulate tex * c >> 7, e.g. 0xc0 = 1.5x),
/// alpha saturates at 0x80.
fn gs_color(c: [u8; 4]) -> Color {
    let f = |v: u8| v as f32 / 128.0;
    Color::srgba(f(c[0]), f(c[1]), f(c[2]), f(c[3]).min(1.0))
}

/// Window pixel rect (y down) -> 2D camera transform (origin centre, y up).
fn place(l: &HudLayout, r: crate::engine::camera::hud::Rect, z: f32) -> Transform {
    Transform::from_xyz(r.x + r.w * 0.5 - l.window_w * 0.5, l.window_h * 0.5 - (r.y + r.h * 0.5), z)
}

pub fn spawn_caption(commands: &mut Commands, ui: &UiFonts, l: &HudLayout, c: &Caption) {
    spawn_frame(commands, ui, l, &UiFrame { boxes: vec![c.back], glyphs: c.glyphs.clone() });
}

/// Spawn boxes (z 0..) and glyphs (z 1) of a UI frame.
pub fn spawn_frame(commands: &mut Commands, ui: &UiFonts, l: &HudLayout, f: &UiFrame) {
    for (i, b) in f.boxes.iter().enumerate() {
        let r = l.map_text_rect(b.x0, b.y0, b.x1, b.y1, Anchor::Center);
        commands.spawn((Sprite { color: gs_color(b.rgba), custom_size: Some(Vec2::new(r.w, r.h)), ..default() }, place(l, r, i as f32 * 0.001), UiSprite));
    }
    for g in &f.glyphs {
        let r = l.map_text_rect(g.x0, g.y0, g.x1, g.y1, Anchor::Center);
        commands.spawn((
            Sprite {
                image: ui.images[g.tex].clone(),
                rect: Some(bevy::math::Rect::new(g.u0, g.v0, g.u1, g.v1)),
                custom_size: Some(Vec2::new(r.w, r.h)),
                color: gs_color(g.rgba),
                ..default()
            },
            place(l, r, 1.0),
            UiSprite,
        ));
    }
}

fn load_settings() -> Option<(std::path::PathBuf, Settings)> {
    let p = Settings::default_path();
    let t = std::fs::read_to_string(&p).ok()?;
    Settings::from_toml(&t).ok().map(|s| (p, s))
}

/// Graphics page values from settings.toml; "Ray traced" only offered with feature `rt` and a GPU that has
/// Solari's wgpu features.
fn init_graphics_opts(mut game: ResMut<UiGame>, pad: Option<Res<PadInput>>, #[cfg(feature = "rt")] dev: Option<Res<bevy::render::renderer::RenderDevice>>) {
    #[cfg(feature = "rt")]
    let rt_available = dev.is_some_and(|d| d.features().contains(bevy::solari::SolariPlugins::required_wgpu_features()));
    #[cfg(not(feature = "rt"))]
    let rt_available = false;
    let s = game.settings.as_ref().map(|(_, s)| s.clone()).unwrap_or_default();
    game.menu.gfx = GraphicsOpts { raytraced: rt_available && s.lighting == Lighting::Raytraced, rt_available, hor_plus: s.hor_plus, ink_outlines: s.ink_outlines };
    // A11: the GI the app runs with = what bin/game.rs gave gfx::rt::RtPlugin at App build
    let rt_running = game.menu.gfx.raytraced;
    game.menu.video = VideoOpts { width: s.width, height: s.height, fullscreen: s.fullscreen, vsync: s.vsync, win_w: s.width as f32, win_h: s.height as f32, rt_running };
    game.menu.audio = AudioOpts { master: s.master_volume, music: s.music_volume, sfx: s.sfx_volume };
    game.menu.mods = ModsOpts::scan(&crate::engine::game::settings::exe_dir().join("mods"), &s.mods);
    game.menu.controls = pad.map(|p| controls_rows(&p.bindings)).unwrap_or_default();
}

/// A12: Graphics-page GI choice -> `RtState::requested` (live switch in rt builds), greyed-row reason, and the
/// `SLY2_GI_AT` verification schedule.
fn sync_gi(mut game: ResMut<UiGame>, mut st: ResMut<RtState>, mut frame: Local<u32>, mut sched: Local<Option<String>>) {
    let sched = sched.get_or_insert_with(|| std::env::var("SLY2_GI_AT").unwrap_or_default());
    *frame += 1;
    if let Some(on) = gi_schedule(sched, *frame) {
        info!("ui: SLY2_GI_AT frame {} -> GI {}", *frame, if on { "on" } else { "off" });
        game.menu.gfx.raytraced = on && game.menu.gfx.rt_available;
    }
    let live = cfg!(feature = "rt");
    let reason = gi_unavailable_reason(live, game.menu.gfx.rt_available);
    if game.menu.gi_live != live || game.menu.gi_reason != reason {
        game.menu.gi_live = live;
        game.menu.gi_reason = reason;
    }
    if game.menu.video.rt_running != st.active {
        game.menu.video.rt_running = st.active;
    }
    if game.menu.gfx.rt_available && st.requested != game.menu.gfx.raytraced {
        st.requested = game.menu.gfx.raytraced;
    }
}

/// A11 port addition: mouse in menus (left click = cross, right click = triangle, wheel = up / down).
fn port_mouse(mut game: ResMut<UiGame>, buttons: Option<Res<ButtonInput<MouseButton>>>, scroll: Option<Res<bevy::input::mouse::AccumulatedMouseScroll>>) {
    if game.menu.depth < 1 {
        return;
    }
    if let Some(b) = buttons {
        if b.just_pressed(MouseButton::Left) {
            game.menu.port_btn |= 0x40;
        }
        if b.just_pressed(MouseButton::Right) {
            game.menu.port_btn |= 0x10;
        }
    }
    if let Some(s) = scroll {
        if s.delta.y > 0.0 {
            game.menu.port_dir = -1;
        } else if s.delta.y < 0.0 {
            game.menu.port_dir = 1;
        }
    }
}

/// A11: apply the Display / Sound Levels / Mods pages (window live, settings.toml / load_order.txt written),
/// track the window size for the aspect row, and drive the player volumes.
#[allow(clippy::type_complexity)]
fn port_options(
    mut game: ResMut<UiGame>,
    mut windows: Query<&mut Window, With<bevy::window::PrimaryWindow>>,
    ga: Option<ResMut<crate::engine::game::app::GameAudio>>,
    mut sinks: Query<(&mut AudioSink, &PlaybackSettings)>,
    mut last_vol: Local<Option<(f32, f32)>>,
    mut synced: Local<bool>,
) {
    let game = &mut *game;
    if let Ok(w) = windows.single() {
        let (pw, ph) = (w.resolution.physical_width() as f32, w.resolution.physical_height() as f32);
        if !std::mem::replace(&mut *synced, true) && w.mode == bevy::window::WindowMode::Windowed {
            // first frame: the window may differ from settings.toml (--width / --height)
            game.menu.video.width = w.resolution.width() as u32;
            game.menu.video.height = w.resolution.height() as u32;
        }
        if game.menu.video.win_w != pw || game.menu.video.win_h != ph {
            game.menu.video.win_w = pw;
            game.menu.video.win_h = ph;
        }
    }
    // volumes: settings levels x the original's mixer levels (first frame keeps the startup values)
    let o = game.menu.opts;
    let target = game.menu.audio.effective(o.music, o.effects);
    if let Some(mut ga) = ga {
        if last_vol.is_some_and(|l| l != target) {
            ga.music_volume = target.0;
            ga.sfx_volume = target.1;
            // the level music is the one looping player (GameAudio::music, japp::poll_level_audio)
            for (mut s, ps) in &mut sinks {
                if matches!(ps.mode, bevy::audio::PlaybackMode::Loop) {
                    s.set_volume(bevy::audio::Volume::Linear(target.0));
                }
            }
        }
        *last_vol = Some(target);
    }
    let events: Vec<MenuEvent> = game.port_events.drain(..).collect();
    if events.is_empty() {
        return;
    }
    let v = game.menu.video;
    let a = game.menu.audio;
    for e in &events {
        match e {
            MenuEvent::Video => {
                if let Ok(mut w) = windows.single_mut() {
                    if w.resolution.width() as u32 != v.width || w.resolution.height() as u32 != v.height {
                        w.resolution.set(v.width as f32, v.height as f32);
                    }
                    w.mode = if v.fullscreen { bevy::window::WindowMode::BorderlessFullscreen(bevy::window::MonitorSelection::Primary) } else { bevy::window::WindowMode::Windowed };
                    w.present_mode = if v.vsync { bevy::window::PresentMode::AutoVsync } else { bevy::window::PresentMode::AutoNoVsync };
                }
            }
            MenuEvent::Mods => match game.menu.mods.write() {
                Ok(()) => info!("ui: mods load order written to {} (applies on the next start)", game.menu.mods.dir.join("load_order.txt").display()),
                Err(e) => warn!("ui: cannot write the mods load order: {e}"),
            },
            _ => {}
        }
    }
    let mods = game.menu.mods.enabled();
    let any_mods = events.contains(&MenuEvent::Mods);
    if let Some((path, s)) = game.settings.as_mut() {
        (s.width, s.height, s.fullscreen, s.vsync) = (v.width, v.height, v.fullscreen, v.vsync);
        (s.master_volume, s.music_volume, s.sfx_volume) = (a.master, a.music, a.sfx);
        if any_mods {
            s.mods = mods;
        }
        if let Err(e) = std::fs::write(&*path, s.to_toml()) {
            warn!("ui: cannot write {}: {e}", path.display());
        }
    } else {
        warn!("ui: settings.toml not loaded; option change not saved");
    }
}

fn parse_int(s: &str) -> Option<i32> {
    let s = s.trim();
    s.strip_prefix("0x").map_or_else(|| s.parse().ok(), |h| i32::from_str_radix(h, 16).ok())
}

/// FUN_0013df80 tail: level strings + FUN_00210be0(0x3e1c20) when a level load result arrives.
fn capture_strings(mut game: ResMut<UiGame>, res: Option<Res<LevelLoadResult>>) {
    let Some(res) = res else { return };
    if !res.is_changed() {
        return;
    }
    let Ok(l) = &res.0 else { return };
    game.strings = LevelStrings::from_header(&l.header.strings);
    game.table = StringTable::with_level_ids(&game.strings);
    game.meter_init = false;
}

/// A12: FUN_001d9968 (level init, FUN_001d9520) / FUN_001d9df8 (per frame, FUN_001d9648) on the HUD meter.
/// The port JT is Sly (class 0x35); dt = frame time (DAT_002dded4).
fn hud_meter(mut game: ResMut<UiGame>, players: Query<&Player>, time: Res<Time>, classes: Option<Res<crate::engine::classes::plugin::Classes>>) {
    let jt = players.iter().next().map(|p| super::meter::MeterJt {
        id: 1,
        cls: 0x35,
        hp: p.jt.dmg.hp,
        hp_max: p.jt.dmg.hp_max,
        mp: p.jt.gad.FUN_001441a0(),
        mp_max: p.jt.gad.mp_max,
        f_10ec: p.jt.p13.f_10ec,
        f_10c8: -1,
        acquired: p.jt.gad.acquired,
    });
    let game = &mut *game;
    if !game.meter_init {
        game.meter.FUN_001d9968(jt, true);
        game.meter_init = true;
        // binocucom init 0x19cbf8 -> FUN_0019d540
        game.binoc = Default::default();
        game.binoc.FUN_0019d540(0x35, 0);
        game.binoc_req = -1;
        // A14: FUN_0020f710 (level start, world+0x3ba0 slots)
        game.slots.FUN_0020f710();
        if let Some(t) = game.countdown_carry.take() {
            game.slots.FUN_001618f8(t, time.elapsed_secs());
        }
        game.slot_feed = classes.as_ref().map_or((0, 0), |c| (c.host.hud_coins, c.host.hud_bottles));
        game.timer_feed = classes.as_ref().map_or(0, |c| c.host.hud_timer.0);
        // WP-UI4: FUN_001a3020 (level init) resets the vault HUD object to hidden; sync the request counter
        game.hud_vault = false;
        game.vault_feed = classes.as_ref().map_or(0, |c| c.host.hud_vault.0);
        // debug hook (port): SLY2_HUD_POP=1 pops the coin / bottle labels and a 30 s countdown
        if std::env::var("SLY2_HUD_POP").is_ok_and(|v| v == "1") {
            let now = time.elapsed_secs();
            game.slots.FUN_00161868(now);
            game.slots.FUN_001618b0(now);
            game.slots.FUN_001618f8(30.0, now);
        }
    } else {
        game.meter.FUN_001d9df8(jt, time.delta_secs());
    }
    // A14: FUN_00161868 / FUN_001618b0 requests of the class runtime (K8 coin / bottle pickups, K9 vault)
    let now = time.elapsed_secs();
    if let Some(c) = classes.as_ref() {
        let feed = (c.host.hud_coins, c.host.hud_bottles);
        if feed.0 > game.slot_feed.0 {
            game.slots.FUN_00161868(now);
        }
        if feed.1 > game.slot_feed.1 {
            game.slots.FUN_001618b0(now);
        }
        game.slot_feed = feed;
        // WP-K33: loot countdown (FUN_001618f8 / FUN_00161958 of the loot system)
        if c.host.hud_timer.0 != game.timer_feed {
            game.timer_feed = c.host.hud_timer.0;
            if c.host.hud_timer.1 >= 0.0 {
                game.slots.FUN_001618f8(c.host.hud_timer.1, now);
            } else {
                game.slots.FUN_00161958(now);
            }
        }
        // WP-UI4: vault / safe HUD object 0x2e1e60 (FUN_001a3078 show / FUN_001a30d0 hide) on the
        // class-78 vault +0x3d80 edges
        if c.host.hud_vault.0 != game.vault_feed {
            game.vault_feed = c.host.hud_vault.0;
            game.hud_vault = c.host.hud_vault.1;
        }
    }
    game.slots.meter_shown = game.meter.health_shown || game.meter.power_shown;
    let covered = game.menu.depth > 0 || game.title;
    game.slots.update(now, time.delta_secs(), covered);
    game.slots.sounds.clear();
    // binocucom HUD: state requests from the JT side (0x19d660), update while looking (0x19d330 state 1)
    if let Some(p) = players.iter().next() {
        let bn = &p.jt.binoc;
        let bi = super::binoc_hud::BinocIn { cls: 0x35, mode: bn.mode, yaw: p.jt.yaw, zoom: 0.0, f_10ec: p.jt.p13.f_10ec, f_10e4: p.jt.p13.f_10e4 };
        if bn.hud != game.binoc_req {
            game.binoc_req = bn.hud;
            game.binoc.FUN_0019d660(&bi, bn.hud);
        }
        if bn.state == 1 && (bn.mode == 0 || bn.mode == 1) {
            game.binoc.FUN_0019da78(&bi);
        }
        game.binoc.msgs.clear();
    }
}

fn menu_ctx(game: &UiGame, player: Option<&Player>) -> MenuCtx {
    MenuCtx {
        transition: false,
        save_ui: false,
        screens_ready: true,
        // FUN_00132ea0: the port only runs Sly (0x35)
        player_char: player.map(|_| 0x35),
        // FUN_00210ae0 / DAT_003da3e4 / DAT_003e0f9c: level table / job state not available -> hidden rows
        level_flag: false,
        job_help: false,
        job_active: false,
        title: game.title,
        graphics: true,
    }
}

/// FUN_001d9648 (menu part) + FUN_001dd920 (title START / SELECT).
#[allow(clippy::too_many_arguments)]
fn ui_update(
    mut game: ResMut<UiGame>,
    pad: Option<ResMut<PadInput>>,
    time: Res<Time>,
    players: Query<&Player>,
    mut lg: Option<ResMut<LevelGame>>,
    audio: Option<ResMut<AudioSystem>>,
    gfx: Option<ResMut<GfxSettings>>,
    (mut slots, mut card, mut cap, mut prev_depth): (
        Option<ResMut<crate::engine::game::flow_app::SaveSlots>>,
        Local<super::card_app::CardApp>,
        ResMut<UiCaption>,
        Local<i32>,
    ),
) {
    let Some(mut pad) = pad else { return };
    let game = &mut *game;
    // FUN_0020e3c0 (level start): resolve the fixed save-variable paths; FUN_001af9c8 reads the stored values
    if let Some(g) = lg.as_deref() {
        if let Some(v) = g.vars.as_ref() {
            if lg.as_ref().is_some_and(|l| l.is_changed()) || game.paths == SavePaths::default() {
                game.paths = SavePaths::FUN_0020e3c0(v);
            }
            game.menu.saved = SavedOpts::read(v, &g.data, &game.paths);
        }
    }
    let player = players.iter().next();
    let ctx = menu_ctx(game, player);
    let clock = Clock { t: time.elapsed_secs(), dt: time.delta_secs() };
    let p = &mut pad.pad;
    if game.open_pause && !game.strings.0.is_empty() && player.is_some() {
        game.open_pause = !game.menu.FUN_001b0290(&ctx);
        if !game.open_pause {
            if let Some(r) = game.open_row.take() {
                if let Some(f) = game.menu.frames.get_mut(game.menu.depth.saturating_sub(1) as usize) {
                    f.sel = r;
                }
            }
            if let Some(g) = game.open_gfx.take() {
                // options menu with its Graphics row selected (2: the Graphics page too; A11: 3..6 the
                // Display / Sound Levels / Controls / Mods page, 7 the Graphics page with GI flipped)
                game.menu.FUN_001b04e0(&ctx, false);
                if let Some(f) = game.menu.top_mut_pub() {
                    f.sel = f.items.iter().position(|i| i.id == PORT_GFX_ID).unwrap_or(0) as i32;
                }
                match g {
                    2 => game.menu.port_open_graphics(),
                    3..=6 => game.menu.port_open_page([PORT_VID_OPEN, PORT_AUD_OPEN, PORT_CTL_OPEN, PORT_MOD_OPEN][g as usize - 3]),
                    7 => {
                        game.menu.gfx.raytraced = game.menu.gfx.rt_available && !game.menu.gfx.raytraced;
                        game.menu.port_open_graphics();
                    }
                    _ => {}
                }
            }
        }
    }
    if game.title && game.menu.depth == 0 {
        // A14: FUN_001dd920 runs in `title_input` (before J16's title_continue)
    } else if game.menu.depth == 0 && pad::FUN_00187d80(p, 0x800) != 0 && !game.menu.FUN_001b0290(&ctx) {
        pad::FUN_00187e08(p, 0x800);
    }
    // WP-EP1e: FUN_001acc38 card refresh of the slot copies when the pause menu opens (MenuMgr::slots /
    // cur_slot feed FUN_001ad538 and the slot rows)
    if game.menu.depth > 0 && *prev_depth == 0 {
        card_refresh(game, &card, slots.as_deref(), lg.as_deref());
    }
    *prev_depth = game.menu.depth;
    // WP-EP1e: the memory-card dialog 0x2e6550 (FUN_001accc8) runs before the menu; while one of its
    // messages is up the menu gets no input
    let mut out = card_dialog_run(game, &mut card, p, clock.t, lg.as_deref_mut(), slots.as_deref_mut(), |h| h.run(|d, h| d.FUN_001accc8(h)));
    if card.dlg.msg == -1 {
        game.menu.FUN_001aeee0(p, clock, &ctx);
    }
    let events: Vec<MenuEvent> = game.menu.events.drain(..).collect();
    for e in events {
        debug!("ui: menu event {e:?}");
        match e {
            MenuEvent::SaveSlot(n) => {
                out.merge(card_dialog_run(game, &mut card, p, clock.t, lg.as_deref_mut(), slots.as_deref_mut(), |h| {
                    h.run(|d, h| {
                        d.FUN_001ad480(h, n);
                    })
                }));
            }
            MenuEvent::LoadSlot(n) => card_load(game, &mut card, n, lg.as_deref_mut(), slots.as_deref_mut()),
            MenuEvent::DeleteSlot(n) => {
                if let Some(s) = slots.as_deref() {
                    let path = s.card.slot_path(n.clamp(0, 2) as usize);
                    match std::fs::remove_file(&path) {
                        Ok(()) => info!("card: deleted slot {} ({})", n + 1, path.display()),
                        Err(e) => warn!("card: delete slot {}: {e}", n + 1),
                    }
                }
                card_refresh(game, &card, slots.as_deref(), lg.as_deref());
            }
            MenuEvent::Setting(_) | MenuEvent::Revert => {
                let Some(g) = lg.as_deref_mut() else { continue };
                let Some(v) = g.vars.as_ref() else { continue };
                if let MenuEvent::Setting(id) = e {
                    opt::FUN_001afb88(id, &mut game.menu.opts, v, &game.paths, &mut g.data);
                } else {
                    opt::FUN_001d8ff8(&mut game.menu.opts, v, &game.paths, &g.data);
                }
            }
            // FUN_001b0878 "Yes, Quit": with a slot in use save first (dialog purpose 1 quits afterwards),
            // else FUN_001660f8 (back to the title / episode screen)
            MenuEvent::Quit | MenuEvent::ExitToEpisode => {
                let mut saving = false;
                if e == MenuEvent::Quit {
                    out.merge(card_dialog_run(game, &mut card, p, clock.t, lg.as_deref_mut(), slots.as_deref_mut(), |h| saving = h.FUN_001b0878()));
                }
                if !saving {
                    game.menu.depth = 0;
                    game.menu.state = 0;
                    game.title = true;
                }
            }
            MenuEvent::Graphics => apply_graphics(game, gfx.as_deref()),
            MenuEvent::Video | MenuEvent::Audio | MenuEvent::Mods => game.port_events.push(e),
            _ => info!("ui: menu event {e:?} (effect not ported)"),
        }
    }
    {
        let o = out;
        if o.saved.is_some() {
            card_refresh(game, &card, slots.as_deref(), lg.as_deref());
        }
        if o.pop {
            game.menu.FUN_001af608();
        }
        if o.close_all {
            game.menu.FUN_001af660();
        }
        if o.quit {
            // FUN_001660f8
            game.menu.depth = 0;
            game.menu.state = 0;
            game.title = true;
        }
    }
    // the dialog message (REPLACED: screen string 0x52c) as the caption
    if card.dlg.msg != card.logged {
        info!("card: dialog msg {:#x} -> {:#x} (purpose {})", card.logged, card.dlg.msg, card.dlg.purpose);
        card.logged = card.dlg.msg;
    }
    let text = super::card_app::msg_text(card.dlg.msg).map(str::to_string);
    if text.is_some() || card.dlg.msg == -1 && cap.0.as_deref().is_some_and(|c| (0..0x30).any(|m| super::card_app::msg_text(m) == Some(c))) {
        if cap.0 != text {
            cap.0 = text;
        }
    }
    if let Some(mut gfx) = gfx {
        if gfx.ink_outlines != game.menu.gfx.ink_outlines {
            gfx.ink_outlines = game.menu.gfx.ink_outlines;
        }
    }
    // 989snd mixer levels (DAT_003dd190 / DAT_003dd18c) -> the port's audio volumes
    if let Some(mut a) = audio {
        let o = game.menu.opts;
        if a.music_volume != o.music || a.sfx_volume != o.effects {
            a.music_volume = o.music;
            a.sfx_volume = o.effects;
        }
    }
}

/// WP-EP1e: run `f` with the card dialog host (no-op without a level / save table / slot files).
fn card_dialog_run(
    game: &mut UiGame,
    card: &mut super::card_app::CardApp,
    p: &mut pad::PadState,
    now: f32,
    lg: Option<&mut LevelGame>,
    slots: Option<&mut crate::engine::game::flow_app::SaveSlots>,
    f: impl FnOnce(&mut super::card_app::Host),
) -> super::card_app::CardOut {
    let (Some(g), Some(s)) = (lg, slots) else { return Default::default() };
    let Some(vars) = g.vars.as_ref() else { return Default::default() };
    let top = game.menu.FUN_001b0078();
    let mut h = super::card_app::Host {
        app: card,
        now,
        pad: p,
        card: &s.card,
        slot: &mut s.slot,
        vars,
        paths: &game.paths,
        data: &mut g.data,
        vibration: game.menu.opts.vibration,
        title: game.title,
        save_menu: game.menu.depth > 0 && matches!(top, 0x19 | 0x1e | 0x1f),
        purpose: None,
        out: Default::default(),
    };
    f(&mut h);
    h.out
}

/// WP-EP1e: FUN_001acc38(0x2e6550, 1) card refresh -> MenuMgr::slots (`dt` of each slot file) / cur_slot.
fn card_refresh(game: &mut UiGame, card: &super::card_app::CardApp, slots: Option<&crate::engine::game::flow_app::SaveSlots>, lg: Option<&LevelGame>) {
    let (Some(s), Some(v)) = (slots, lg.and_then(|g| g.vars.as_ref())) else { return };
    game.menu.slots = super::card_app::slots_dt(&s.card, v, &game.paths);
    game.menu.cur_slot = card.in_use.then_some(s.slot as i32);
}

/// WP-EP1e: FUN_001ad320 (in-game "Load Game..." slot n): DAT_002f6330 = the slot; an empty slot (dt == 0)
/// gets FUN_001d8d80(slot, 1); then FUN_00165e38 continues from it (`flow::FUN_00165e38_full`: the episode
/// level / hideout request, or a new game on an empty slot). REPLACED: the slot blob becomes the live blob
/// here (the original's level load copies DAT_002deb4c) and the menus close.
fn card_load(game: &mut UiGame, card: &mut super::card_app::CardApp, n: i32, lg: Option<&mut LevelGame>, slots: Option<&mut crate::engine::game::flow_app::SaveSlots>) {
    use crate::engine::game::flow;
    let (Some(g), Some(s)) = (lg, slots) else { return };
    let Some(vars) = g.vars.as_ref() else { return };
    let Ok(i) = usize::try_from(n) else { return };
    let paths = flow::StartPaths::new(vars);
    let d = match flow::FUN_001ad320(&s.card, i, vars, paths.dt.as_deref()) {
        Ok(d) => d,
        Err(crate::engine::game::save::McError::NoFile) => {
            // an empty slot: a fresh blob keeping the options (FUN_001d8d80(slot, 1))
            let mut d = g.data.clone();
            let kept: Vec<_> = game.paths.flags.iter().flatten().map(|p| (p, vars.ee_sub_1d7ed8(p, &g.data))).collect();
            vars.ee_sub_1d8d80(&mut d, false, &[], None);
            for (p, v) in kept {
                vars.ee_sub_1d80c8(p, v, &mut d);
            }
            d
        }
        Err(e) => {
            warn!("card: load slot {}: {e:?}", i + 1);
            return;
        }
    };
    s.slot = i;
    card.in_use = true;
    g.data = d;
    let st = flow::FUN_00165e38_full(false, vars, &paths, &g.data, g.table.as_ref(), i as i32 + 1);
    info!("card: load slot {}: {st:?} (completion {}%): {}", i + 1, flow::FUN_001d82b8(vars, &paths, &g.data), super::card_app::blob_summary(vars, &game.paths, &g.data));
    game.menu.FUN_001af660();
}

/// Port addition: write the graphics page into settings.toml (ink outlines also apply live).
fn apply_graphics(game: &mut UiGame, _gfx: Option<&GfxSettings>) {
    let o = game.menu.gfx;
    let Some((path, s)) = game.settings.as_mut() else {
        warn!("ui: settings.toml not loaded; graphics change not saved");
        return;
    };
    s.hor_plus = o.hor_plus;
    s.ink_outlines = o.ink_outlines;
    if o.rt_available {
        s.lighting = if o.raytraced { Lighting::Raytraced } else { Lighting::Raster };
    }
    match std::fs::write(&*path, s.to_toml()) {
        Ok(()) => info!("ui: graphics saved to {} (ink / Hor+ / global illumination live)", path.display()),
        Err(e) => warn!("ui: cannot write {}: {e}", path.display()),
    }
}

/// Rebuild the UI sprites when the content or the window layout changes.
#[allow(clippy::too_many_arguments)]
fn draw_ui(
    mut commands: Commands,
    ui: Option<ResMut<UiFonts>>,
    game: Res<UiGame>,
    cap: Res<UiCaption>,
    cap_id: Res<UiCaptionId>,
    safe: Res<HudSafeRect>,
    players: Query<&Player>,
    lg: Option<Res<crate::engine::game::app::LevelGame>>,
    phud: Option<Res<crate::engine::player::plugin::PlayerHud>>,
    old: Query<Entity, With<UiSprite>>,
    time: Res<Time>,
    mut last: Local<Option<(UiFrame, HudLayout)>>,
) {
    let Some(mut ui) = ui else { return };
    let mut f = UiFrame::default();
    let text = cap.0.as_ref().map(|s| s.as_bytes().to_vec()).or_else(|| cap_id.0.map(|id| game.table.FUN_00210ca8(id).to_vec()));
    if let Some(s) = text.filter(|s| !s.is_empty()) {
        let c = caption(&mut ui.fonts, &s);
        f.boxes.push(c.back);
        f.glyphs.extend(c.glyphs);
    }
    // DAT_003e0fb0 = FUN_0020e3c0 path ["loot" "ccoin"]
    let coins = lg.as_ref().and_then(|g| {
        let vars = g.vars.as_ref()?;
        game.paths.coins.as_ref()?;
        Some(opt::word(vars, &g.data, &game.paths.coins) as i32)
    });
    // DAT_003e1bf4 / DAT_003e1bf8 (bottle counter) fed from the player/classes link (WP-P12 PlayerHud)
    let count = phud.as_ref().map(|h| (h.bottles, h.bottles_total)).unwrap_or_default();
    let ctx = BindCtx { coins, count, menu: game.menu.view(), ..default() };
    let player = players.iter().next().map(|p| HudPlayer { hp: p.jt.dmg.hp, hp_max: p.jt.dmg.hp_max });
    if game.title {
        hud::title_menu(&mut ui.fonts, &game.strings, &game.title_text, Some(&game.load), &mut f);
    } else if let Some(e) = game.episode.as_ref().filter(|e| e.f_11c >= 0 && game.menu.depth == 0) {
        let inp = episode_in(lg.as_deref(), &[]);
        hud::episode(&mut ui.fonts, e, &inp, &mut f);
    } else if game.show_hud && player.is_some() && game.menu.depth == 0 {
        // A12: the pause screen backdrop is opaque; the counters are not drawn over it
        // A12: health / power bars from the HUD meter (FUN_001d9df8), counters from hud()
        // A14: the counters are the world+0x3ba0 slot labels (shown after a pickup)
        hud::slots(&mut ui.fonts, &game.bindings, &game.strings, &ctx, &game.slots, time.elapsed_secs(), &mut f);
        hud::meter(&game.meter, &mut f);
        hud::binoc(&game.binoc, &mut f);
        // WP-UI4: vault / safe HUD object 0x2e1e60 (FUN_001a3078 / FUN_001a30d0)
        hud::vault(game.hud_vault, &mut f);
    }
    if game.menu.depth > 0 {
        hud::menu(&mut ui.fonts, &game.bindings, &game.strings, &ctx, &mut f);
    }
    let key = (f, safe.0);
    if last.as_ref() == Some(&key) {
        return;
    }
    for e in &old {
        commands.entity(e).despawn();
    }
    spawn_frame(&mut commands, &ui, &safe.0, &key.0);
    *last = Some(key);
}

/// A13: the title screen 0x3da530 follows the level sequence action 0x44 (FUN_001dd820, K22 counter
/// `k22_title_shows`) and its FUN_001d9880 answer (`k22_title_active`) mirrors `UiGame::title`, so the
/// sequence waits while the title is up and goes on after START (FUN_001dd888 hides it). Replaces the
/// SLY2_UI_TITLE-only stand-in (the env flag stays as a verification override).
/// WP-EP1a: ordered after `tick_classes` / before `lend_classes`: while the class host is lent to the scripts
/// `Classes::host` is a default one (0 shows), which reset `title_shows_seen` and re-raised the title every
/// other frame (title back after START) and dropped the `k22_title_active` mirror.
fn title_from_sequence(mut game: ResMut<UiGame>, classes: Option<ResMut<crate::engine::classes::plugin::Classes>>) {
    let Some(mut c) = classes else { return };
    // WP-EP1c: while the class set is lent to the script host `Classes` holds a default host (counter 0):
    // not a new world, so the seen count must not be reset (the title log re-fired ~every 18 frames)
    if c.host.world.is_none() {
        return;
    }
    let shows = c.host.k22_title_shows;
    if shows < game.title_shows_seen {
        game.title_shows_seen = 0;
    }
    if shows > game.title_shows_seen {
        game.title_shows_seen = shows;
        game.title = true;
        info!("ui: title screen shown by sequence action 0x44 (FUN_001dd820, #{shows})");
    }
    if c.host.k22_title_active != game.title {
        c.host.k22_title_active = game.title;
    }
}

/// A13: FUN_002581c0 reads of the episode save variables (FUN_0020e3c0 paths `episode <name> dt` /
/// `percent_complete`, game::flow::episode_paths).
fn episode_in(lg: Option<&LevelGame>, oids: &[i32]) -> super::episode::EpIn {
    use crate::engine::game::save::SaveValue;
    let mut inp = super::episode::EpIn { oids: oids.to_vec(), ..Default::default() };
    if let Some((v, g)) = lg.and_then(|g| g.vars.as_ref().map(|v| (v, g))) {
        let paths = crate::engine::game::flow::episode_paths(v);
        for (i, [dt, pct]) in paths.iter().enumerate() {
            let read = |p: &Option<Vec<u16>>| {
                p.as_ref().map(|p| match v.ee_sub_1d7ed8(p, &g.data) {
                    SaveValue::Float(f) => f,
                    SaveValue::Int(n) => n as f32,
                    _ => 0.0,
                })
            };
            inp.dt[i] = read(dt);
            inp.pct[i] = read(pct);
        }
    }
    inp
}

/// A13: class 131 (`ui::episode`) of the active world: FUN_0015cd68 + FUN_0015cda8 on a new world, then
/// FUN_0015cf30 per frame (blocked by the pause menu / title). The label animations (class-126 base) are
/// not ported: labels end at once (`busy` false); level loads / sounds are logged.
fn episode_update(
    mut game: ResMut<UiGame>,
    level: Option<Res<crate::engine::classes::plugin::ClassesLevel>>,
    pad: Option<ResMut<PadInput>>,
    time: Res<Time>,
    lg: Option<Res<LevelGame>>,
    mut fmv: Option<ResMut<crate::engine::game::fmv_app::FmvPlayback>>,
) {
    let game = &mut *game;
    let w = level.as_ref().and_then(|l| l.0.clone());
    let id = w.as_ref().map_or(0, |w| Arc::as_ptr(w) as usize);
    let oids: Vec<i32> = w.as_ref().map(|w| w.objects.iter().map(|o| o.oid as i32).collect()).unwrap_or_default();
    let t = time.elapsed_secs();
    if id != game.episode_world {
        game.episode_world = id;
        game.episode = w.as_ref().filter(|w| w.objects.iter().any(|o| o.class == 131)).map(|_| {
            let mut e = super::episode::Ep131::FUN_0015cd68();
            e.FUN_0015cda8(&episode_in(lg.as_deref(), &oids), t);
            e
        });
    }
    let (Some(e), Some(mut pad)) = (game.episode.as_mut(), pad) else { return };
    let mut inp = episode_in(lg.as_deref(), &oids);
    inp.blocked = game.menu.depth > 0 || game.title;
    e.FUN_0015cf30(&mut pad.pad, Clock { t, dt: time.delta_secs() }, &inp);
    let ep = e.f_110;
    let evs: Vec<_> = e.events.drain(..).collect();
    for ev in evs {
        super::episode::episode_event(&mut game.movies, ev, ep, &mut pad.pad);
        info!("ui: episode select {ev:?} (episode {ep})");
        if let Some(f) = fmv.as_deref_mut().filter(|_| !game.movies.movies.is_empty()) {
            play_movies(&mut game.movies, f);
        }
    }
}

// ---------------------------------------------------------------- WP-A14: title / load menu, movies

/// A14: free memory passed to FUN_001b3b08 (the port has no EE heap limit).
const FREE_MEM: u32 = u32::MAX;

/// A14: completion of each save slot file (FUN_001ad320 load + FUN_001d82b8), None = no / unreadable file.
fn scan_slots(slots: &crate::engine::game::flow_app::SaveSlots, vars: &crate::engine::game::save::SaveVars) -> [Option<i32>; crate::engine::game::save::SLOT_COUNT] {
    use crate::engine::game::flow;
    let paths = flow::StartPaths::new(vars);
    std::array::from_fn(|i| {
        if !slots.card.slot_path(i).is_file() {
            return None;
        }
        flow::FUN_001ad320(&slots.card, i, vars, paths.dt.as_deref()).ok().map(|d| flow::FUN_001d82b8(vars, &paths, &d))
    })
}

/// A14: title screen 0x3da530 per frame, run before J16's `flow_app::title_continue`: the port load menu
/// (DAT_002f6334), FUN_001ddbf0 text, FUN_001dd920 (START -> FUN_001ddb08 new game / FUN_00165e38 continue
/// through `game::flow::FUN_00165e38_full`, SELECT -> options). START is consumed here, so title_continue
/// never auto-continues slot 0 and New Game stays reachable when a save exists.
#[allow(clippy::too_many_arguments)]
fn title_input(
    mut game: ResMut<UiGame>,
    pad: Option<ResMut<PadInput>>,
    time: Res<Time>,
    players: Query<&Player>,
    mut lg: Option<ResMut<LevelGame>>,
    mut slots: Option<ResMut<crate::engine::game::flow_app::SaveSlots>>,
) {
    let game = &mut *game;
    if !game.title {
        game.load.open = false;
        game.load.offered = false;
        return;
    }
    let Some(mut pad) = pad else { return };
    let p = &mut pad.pad;
    let clock = Clock { t: time.elapsed_secs(), dt: time.delta_secs() };
    let lg = lg.as_deref_mut();
    // offer the load menu once per title show when slot files exist (the card manager's load screen)
    if !game.load.offered {
        if let (Some(s), Some(v)) = (slots.as_deref(), lg.as_ref().and_then(|g| g.vars.as_ref())) {
            game.load.offered = true;
            game.load.slots = scan_slots(s, v);
            game.load.open = game.load.any() && std::env::var("SLY2_UI_LOAD").map_or(true, |v| v != "0");
            game.load.row = game.card_slot;
        }
    }
    if game.load.open && game.menu.depth == 0 {
        let mut pick = game.load.input(p, clock);
        // debug hook (port): SLY2_UI_SLOT=n picks load-menu row n once (0 = New Game, 1..3 = slot)
        if let Some(n) = std::env::var("SLY2_UI_SLOT").ok().and_then(|v| v.parse::<usize>().ok()).filter(|_| game.card_slot == 0 && !game.load.auto) {
            game.load.auto = true;
            game.load.open = false;
            pick = if n == 0 { title::LoadPick::NewGame } else { title::LoadPick::Slot(n - 1) };
        }
        match pick {
            title::LoadPick::NewGame => {
                game.card_slot = 0;
                if let (Some(g), Some(b)) = (lg, game.fresh_blob.as_ref()) {
                    g.data = b.clone();
                }
                // port: a new game saves into the first empty slot
                if let Some(s) = slots.as_deref_mut() {
                    if let Some(i) = game.load.slots.iter().position(Option::is_none) {
                        s.slot = i;
                    }
                }
            }
            title::LoadPick::Slot(i) => {
                let (Some(g), Some(s)) = (lg, slots.as_deref_mut()) else { return };
                let Some(v) = g.vars.as_ref() else { return };
                let dt = crate::engine::game::flow::StartPaths::new(v).dt;
                match crate::engine::game::flow::FUN_001ad320(&s.card, i, v, dt.as_deref()) {
                    Ok(d) => {
                        if game.fresh_blob.is_none() {
                            game.fresh_blob = Some(g.data.clone());
                        }
                        g.data = d;
                        info!("card: title load slot {}: {}", i + 1, super::card_app::blob_summary(v, &game.paths, &g.data));
                        game.card_slot = i as i32 + 1;
                        s.slot = i;
                    }
                    Err(e) => warn!("ui: load slot {}: {e:?}", i + 1),
                }
            }
            title::LoadPick::Close | title::LoadPick::None => {}
        }
        if pick != title::LoadPick::None {
            info!("ui: load menu {pick:?} -> DAT_002f6334 = {}", game.card_slot);
        }
        pad::FUN_00187de0(p, 0x800 | 0x100 | 0x80);
        return;
    }
    // FUN_001ddbf0: DAT_002e0c0c state not modelled (false); X1: language count FUN_001a2c98 (1 on the US image)
    let languages = crate::engine::ui::language::Language { region: game.menu.opts.region, lang: game.menu.opts.lang }.FUN_001a2c98();
    let slot = game.card_slot;
    let dt = lg.as_ref().and_then(|g| g.vars.as_ref().map(|v| f32::from_bits(opt::word(v, &g.data, &game.paths.dt)))).unwrap_or(0.0);
    game.title_text = title::FUN_001ddbf0(false, slot, || dt, languages, &game.strings);
    if game.menu.depth != 0 {
        return;
    }
    let player = players.iter().next();
    let ctx = menu_ctx(game, player);
    match title::FUN_001dd920(true, p, slot, languages) {
        TitleInput::NewGame | TitleInput::Continue(_) => {
            let Some(g) = lg else {
                game.title = false;
                return;
            };
            let Some(v) = g.vars.as_ref() else {
                game.title = false;
                return;
            };
            let vib = game.menu.opts.vibration;
            let st = if slot == 0 {
                title::FUN_001ddb08_app(vib, v, &game.paths, &mut g.data, g.table.as_ref())
            } else {
                // FUN_001acc38(0x2e6550, 1): card refresh REPLACED (the blob was loaded by the load menu)
                title::FUN_00165e38_app(vib, v, &game.paths, &mut g.data, g.table.as_ref(), slot)
            };
            info!("ui: title START slot {slot}: {st:?}");
            // FUN_001dd888: the title screen hides (HideTitle: play on in level 0; else the posted transition)
            game.title = false;
        }
        TitleInput::Options => game.menu.FUN_001b04e0(&ctx, false),
        TitleInput::Language | TitleInput::None => {
            // port: square reopens the load menu
            if game.load.any() && pad::FUN_00187d80(p, 0x80) != 0 {
                game.load.open = true;
            }
        }
    }
}

/// A14: FUN_001b3a48(0x2f66b0, desc) requests of the class runtime (class c2dc6f0 slot 15 warp movies,
/// `EngineHost::movies`) -> the UI movie queue -> FUN_001b3b08 -> J16's `FmvPlayback`.
fn class_movies(
    mut game: ResMut<UiGame>,
    c: Option<ResMut<crate::engine::classes::plugin::Classes>>,
    fmv: Option<ResMut<crate::engine::game::fmv_app::FmvPlayback>>,
) {
    let Some(mut c) = c else { return };
    if c.host.movies.is_empty() {
        return;
    }
    for d in std::mem::take(&mut c.host.movies) {
        super::episode::queue_movie(&mut game.movies, crate::engine::disc::descriptor::Descriptor(d));
    }
    if let Some(mut f) = fmv {
        play_movies(&mut game.movies, &mut f);
    }
}

/// FUN_001b3b08 on the UI queue (REPLACED timing: the original plays it inside the next FUN_0020e890
/// transition; here the movies start at once and the posted transition runs behind them).
fn play_movies(q: &mut crate::engine::game::fmv::MovieQueue, f: &mut crate::engine::game::fmv_app::FmvPlayback) {
    if q.special == [None, None] {
        q.special = [crate::engine::game::flow::static_desc(0x2a3968), crate::engine::game::flow::static_desc(0x2a39a8)];
    }
    q.ee_sub_1b3b08(FREE_MEM, f);
}
