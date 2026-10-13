//! Bevy integration of WP-J2 (feature `game`): level audio (bank + KPV music + sound events through WP-G/G2),
//! the per-level game state captured before `world::build_world` drops the load result (save-variable
//! table, spawn), camera placement at the spawn, rumble requests -> input backend, and the
//! `--screenshot <path> --frames N` verification mode.

use super::save::{GameData, SaveVars};
use super::spawn::{self, SpawnAliases};
use super::splice_host::RumbleRequest;
use crate::engine::audio::emitter::{EmitterId, EmitterSystem};
use crate::engine::audio::music_layers::{self, InstState, MusicLayers, PlayerMusic};
use crate::engine::audio::{self, bank::Bank, music, vpk, AudioSystem, Pcm, SoundFileKind};
use crate::engine::camera::{cm, Cm, CmClock};
use crate::engine::disc::descriptor::Descriptor;
use crate::engine::disc::{LevelLoadResult, LevelLoader};
use crate::engine::gfx::game::LevelGeometryRoot;
use crate::engine::input::backend::PadInput;
use crate::engine::world::World;
use bevy::audio::Volume;
use bevy::camera::primitives::Aabb;
use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
use bevy::tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

// ---------------------------------------------------------------- level state

/// Per-level game state that lives in the world struct in the original.
#[derive(Resource, Default)]
pub struct LevelGame {
    /// world+0x3a18 (Block3a18) as a save-variable table
    pub vars: Option<SaveVars>,
    /// 0x3d4a60 save-variable blob
    pub data: GameData,
    /// +0x16c alias lists of the class-0x2a objects
    pub aliases: SpawnAliases,
    /// requested spawn (world+0x3c38/+0x3c3c)
    pub spawn_request: (i32, i32),
    /// spawn object found by FUN_00227fe8 (or the first spawn object as a fallback for the camera)
    pub spawn: Option<usize>,
    /// WP-J17: world+0x3c88/+0x3c8c (DAT_003e1118/111c): the request's spawn / parent when FUN_00227fe8
    /// found it, (0, 0) otherwise (the world memset at load); the death reload (FUN_00161a58 case 0x10)
    /// posts it
    pub last_spawn: (i32, i32),
    /// objects live after the load (`spawn::active_objects`)
    pub active: Vec<usize>,
    pub framed: bool,
    /// sound files of the level (block 0x3dd040)
    pub bank: Option<Descriptor>,
    pub music: Option<Descriptor>,
    /// WP-J3: level bytes kept until the script blocks ran (FUN_001ffa70), then dropped
    pub level_data: Option<Arc<Vec<u8>>>,
    /// WP-J3: result of the level's script run
    pub scripts: Option<super::scripts::ScriptRunReport>,
    /// WP-J4: FUN_0020f318 level table of the loaded level (DAT_003e1c44/48, BIFs 98/99)
    pub table: Option<crate::engine::disc::descriptor::LevelTable>,
    /// WP-J4: number of level loads (identifies the world the persistent script state belongs to)
    pub loads: u32,
    /// WP-J5: live ASEGAs of the level (driven in `tick_scripts`)
    pub anims: super::anim_play::LevelAnims,
}

/// Schedule `.before(world::build_world)`: keeps what WP-J needs from the level header.
pub fn capture_level_game(mut g: ResMut<LevelGame>, res: Option<Res<LevelLoadResult>>) {
    let Some(Ok(l)) = res.as_deref().map(|r| &r.0) else { return };
    let b = &l.header.block_3dd040;
    let files = audio::level_sound_files(b);
    g.bank = files.iter().find(|(k, _)| *k == SoundFileKind::Bank).map(|(_, d)| *d);
    g.music = b.desc2.filter(|d| d.size() > 0).or_else(|| files.iter().find(|(k, _)| *k == SoundFileKind::MusicStream).map(|(_, d)| *d));
    let vars = SaveVars::from_block(&l.header.block_3a18);
    g.vars = Some(vars);
    g.level_data = Some(l.data.clone());
    g.scripts = None;
    g.table = Some(l.header.level_table.clone());
    g.loads += 1;
    g.aliases = SpawnAliases::default();
    g.spawn = None;
    g.last_spawn = (0, 0);
    g.active.clear();
    g.framed = false;
}

/// After `build_world`: FUN_00227fe8 spawn lookup + activation list.
pub fn activate_world(mut g: ResMut<LevelGame>, w: Option<Res<World>>, opt: Option<Res<crate::engine::world::WorldOptions>>) {
    let Some(w) = w else { return };
    if !w.is_changed() {
        return;
    }
    let (oid, parent) = g.spawn_request;
    let mut found = spawn::ee_sub_227fe8(&w, &g.aliases, oid, parent);
    // FUN_0020e890: world+0x3c88/+0x3c8c = request when found
    g.last_spawn = if found.is_some() { (oid, parent) } else { (0, 0) };
    // WP-J17: debug entry (`--table` / `--level`: the first load, default request) into a level without
    // spawn 0x189 -> the task spawn the game's own request would carry (`spawn::debug_entry_spawn`, REPLACED)
    if found.is_none() && g.loads == 1 && (oid, parent) == (crate::engine::game::transition::SPAWN_DEFAULT, -1) {
        let row = opt.map_or(-1, |o| o.0.mission as i32);
        if let Some((i, s)) = spawn::debug_entry_spawn(&w, &g.aliases, row) {
            found = Some(i);
            g.last_spawn = (s, -1);
        }
    }
    // WP-J12: FUN_0020e890 places the player at the spawn object only when FUN_00227fe8 found one
    // (vtable +0x134 via FUN_00228750 / the +0x3ebc loop); otherwise the JT stays where the level load
    // put it. Was: the first class-0x2a object with a transform (table 30: the cable-car retry spawn).
    g.spawn = found.or_else(|| spawn::player_object(&w));
    g.active = spawn::active_objects(&w);
    let LevelGame { anims, active, .. } = &mut *g;
    anims.start_live(&w, active);
    info!("level activated: {} live objects, spawn {:?} (requested oid {oid}, found {found:?})", g.active.len(), g.spawn);
}

/// Places the CM once the geometry exists: behind the spawn object at the follow-camera default distance
/// (DAT_002c4120: 450 back, pitch atan2(125,450)); without a spawn, above the level bounds.
pub fn frame_camera(
    mut g: ResMut<LevelGame>,
    w: Option<Res<World>>,
    mut cmr: ResMut<Cm>,
    roots: Query<(), With<LevelGeometryRoot>>,
    meshes: Query<(&Aabb, &GlobalTransform), With<Mesh3d>>,
) {
    if g.framed || roots.is_empty() {
        return;
    }
    let Some(w) = w else { return };
    // bounds in Sly space (Bevy (x,y,z) -> Sly (x,-z,y))
    let mut lo = Vec3::splat(f32::MAX);
    let mut hi = Vec3::splat(f32::MIN);
    for (a, t) in &meshes {
        let c = t.transform_point(Vec3::from(a.center));
        lo = lo.min(c);
        hi = hi.max(c);
    }
    if lo.x > hi.x {
        return; // meshes not prepared yet
    }
    g.framed = true;
    let sly = |v: Vec3| [v.x, -v.z, v.y];
    let clk = CmClock::default();
    cmr.s_far_clip = cmr.s_far_clip.max((hi - lo).length() * 2.0);
    if let Some(p) = g.spawn.and_then(|i| spawn::spawn_position(&w, i)) {
        let target = [p[0], p[1], p[2] + 50.0];
        let pos = [p[0], p[1] - 450.0, p[2] + 50.0 + 125.0];
        cm::FUN_0014e1a8(&mut cmr, 0.0, 0.0, pos, target, clk);
        info!("camera at spawn {p:?}");
    } else {
        let c = sly((lo + hi) * 0.5);
        let ext = (hi - lo).length() * 0.5;
        let pos = [c[0], c[1] - ext, c[2] + ext * 0.6];
        cm::FUN_0014e1a8(&mut cmr, 0.0, 0.0, pos, c, clk);
        info!("camera over level bounds centre {c:?} r {ext}");
    }
}

// ---------------------------------------------------------------- audio

struct AudioLoad {
    bank: Option<Bank>,
    /// WP-J16: one stream per music layer (`audio::music_layers::layer_channels`)
    music: Vec<Pcm>,
    errors: Vec<String>,
}

/// bevy_audio host for WP-G/G2: level music (KPV) and bank sounds started by emitters / BIF voices.
#[derive(Resource, Default)]
pub struct GameAudio {
    task: Option<Task<AudioLoad>>,
    /// WP-J16: one looping player per layer (volume = 0x3dd040 layer volume, FUN_001fd830 step)
    music: Vec<Entity>,
    /// WP-J16: 0x3dd040 layer state (FUN_001fdc08 / FUN_001fc2e0)
    pub layers: MusicLayers,
    /// frames since the music instance started (+0x52 / +0x50 & 1 while 0)
    music_frames: u32,
    started_emitters: HashSet<EmitterId>,
    started_voices: HashSet<u32>,
    pub music_volume: f32,
    pub sfx_volume: f32,
    loaded_for: Option<(Option<Descriptor>, Option<Descriptor>)>,
}

impl GameAudio {
    pub fn new(music_volume: f32, sfx_volume: f32) -> Self {
        Self { music_volume, sfx_volume, ..Default::default() }
    }
}

/// Replaces snd_BankLoadByLoc + the 989snd stream start: read + decode on the async pool.
pub fn start_level_audio(mut ga: ResMut<GameAudio>, g: Res<LevelGame>, loader: Option<Res<LevelLoader>>, mut emitters: ResMut<EmitterSystem>, mut sys: ResMut<AudioSystem>, mut commands: Commands) {
    let key = (g.bank, g.music);
    if g.bank.is_none() && g.music.is_none() || ga.loaded_for == Some(key) {
        return;
    }
    let Some(loader) = loader else { return };
    ga.loaded_for = Some(key);
    // level change: FUN_002321b8 stop streams, emitters freed with the world
    sys.stop_all();
    *emitters = EmitterSystem::default();
    ga.started_emitters.clear();
    ga.started_voices.clear();
    for e in std::mem::take(&mut ga.music) {
        commands.entity(e).despawn();
    }
    let iso = loader.iso.clone();
    let (bank_d, music_d) = key;
    ga.task = Some(AsyncComputeTaskPool::get().spawn(async move {
        let mut out = AudioLoad { bank: None, music: vec![], errors: vec![] };
        if let Some(d) = bank_d {
            match audio::read_descriptor(&*iso, &d) {
                Ok(b) => out.bank = audio::bank::parse(&b).or_else(|| {
                    out.errors.push(format!("bank lsn {} does not parse", d.lsn()));
                    None
                }),
                Err(e) => out.errors.push(format!("bank read: {e}")),
            }
        }
        if let Some(d) = music_d {
            match audio::read_descriptor(&*iso, &d) {
                Ok(b) => match vpk::decode(&b) {
                    Some((h, ch)) => {
                        out.music = (0..music_layers::layer_count(ch.len()))
                            .map(|n| {
                                let on = music_layers::layer_channels(n, ch.len());
                                let gains: Vec<f32> = (0..ch.len()).map(|c| if on.contains(&c) { 1.0 } else { 0.0 }).collect();
                                music::mix_stereo(&h, &ch, &gains)
                            })
                            .collect()
                    }
                    None => out.errors.push(format!("music lsn {} is not KPV", d.lsn())),
                },
                Err(e) => out.errors.push(format!("music read: {e}")),
            }
        }
        out
    }));
}

fn source(pcm: &Pcm) -> AudioSource {
    AudioSource { bytes: Arc::from(pcm.to_wav()) }
}

pub fn poll_level_audio(mut ga: ResMut<GameAudio>, mut sys: ResMut<AudioSystem>, mut sources: ResMut<Assets<AudioSource>>, mut commands: Commands) {
    let Some(t) = ga.task.as_mut() else { return };
    let Some(r) = block_on(future::poll_once(t)) else { return };
    ga.task = None;
    for e in &r.errors {
        warn!("audio: {e}");
    }
    sys.bank = r.bank;
    info!("audio: bank {} sounds", sys.bank.as_ref().map_or(0, |b| b.sounds.len()));
    if let Some(m) = r.music.first() {
        info!("audio: music {} Hz, {:.1} s, {} layers", m.sample_rate, m.samples.len() as f32 / 2.0 / m.sample_rate.max(1) as f32, r.music.len());
    }
    // all layers start together (silent until the first FUN_001fdc08 pass sets their volumes)
    for m in &r.music {
        let h = sources.add(source(m));
        let e = commands.spawn((AudioPlayer(h), PlaybackSettings { volume: Volume::Linear(0.0), ..PlaybackSettings::LOOP })).id();
        ga.music.push(e);
    }
    ga.music_frames = 0;
    ga.layers.vol = [0.0; 6];
}

/// WP-J16: the FUN_001fd830 layer step (FUN_001fdc08 / FUN_001fc2e0) on the running game: player = the
/// first `Player` (jt+0x10ec focus mode, +0x4c8 attach kind; +0x10c8 is not modelled by player/ -> -1);
/// the instance is the level music while its layer players exist.
pub fn update_music_layers(mut ga: ResMut<GameAudio>, players: Query<&crate::engine::player::plugin::Player>, mut sinks: Query<&mut AudioSink>, time: Res<Time>) {
    let player = players.iter().next().map(|p| PlayerMusic { f_10ec: p.jt.p13.f_10ec, f_10c8: -1, f_4c8: p.jt.attach.kind });
    let inst = (!ga.music.is_empty()).then(|| InstState { starting: ga.music_frames == 0 });
    let dt = time.delta_secs();
    ga.layers.ee_sub_1fdc08(player, inst, dt);
    if inst.is_some() {
        ga.music_frames += 1;
    }
    if ga.layers.dirty == 0 {
        return;
    }
    let (vol, mv) = (ga.layers.vol, ga.music_volume);
    let mut applied = true;
    for (i, e) in ga.music.iter().enumerate() {
        match sinks.get_mut(*e) {
            Ok(mut s) => s.set_volume(Volume::Linear(mv * vol.get(i).copied().unwrap_or(0.0))),
            Err(_) => applied = false,
        }
    }
    // sinks appear a frame after spawn: keep the dirty bits until applied
    if applied {
        ga.layers.dirty = 0;
    }
}

/// Starts PCM for new emitters (BIF 129 / FUN_001fbb90) and AudioSystem voices.
pub fn play_sound_events(mut ga: ResMut<GameAudio>, sys: Res<AudioSystem>, emitters: Res<EmitterSystem>, mut sources: ResMut<Assets<AudioSource>>, mut commands: Commands) {
    let vol = ga.sfx_volume;
    let mut play = |sound: i32, v: f32, commands: &mut Commands| {
        if let Some(p) = sys.sound_pcm(sound) {
            let h = sources.add(source(&p));
            commands.spawn((AudioPlayer(h), PlaybackSettings { volume: Volume::Linear(v * vol), ..PlaybackSettings::DESPAWN }));
        }
    };
    for &id in &emitters.active {
        if ga.started_emitters.insert(id) {
            if let Some(e) = emitters.get(id) {
                play(e.sound, e.f10, &mut commands);
            }
        }
    }
    for v in &sys.voices {
        if ga.started_voices.insert(v.handle) {
            play(v.sound, v.volume, &mut commands);
        }
    }
}

// ---------------------------------------------------------------- scripts (WP-J3)

/// After `activate_world`: evaluates every entry's FUN_001ff858 script block in the world frame
/// (FUN_001ffa70) with `LevelSpliceHost`; rumble from BIF 130 goes to `RumbleQueue`. `t` = clock t
/// (DAT_002dded0), passed by the caller (the clock lives in the binary's GameLoop).
pub fn run_level_scripts(t: f32, g: &mut LevelGame, st: &mut super::vm_state::SpliceState, w: &World, cm: Option<&Cm>, emitters: &mut EmitterSystem, rumble: &mut RumbleQueue, mission: i16) {
    let Some(data) = g.level_data.take() else { return };
    // WP-J4: the world frame + callback list are rebuilt only for a new world, then persist (NonSend SpliceState)
    st.enter_level(g.loads);
    let LevelGame { vars, data: gd, aliases, table, .. } = g;
    let base = super::splice_host::GameSpliceHost { t, cm, emitters, vars: vars.as_ref(), data: gd };
    let level = super::splice_host::LevelSpliceHost { base, mission: mission as i32, aliases, rumble: &mut rumble.0 };
    let mut host = super::vm_state::PersistentHost { level, cm: None, clk: CmClock::default(), world: Some(w), table: table.as_ref(), sched: &mut st.sched };
    let (rep, frames) = super::scripts::run_level_scripts_framed(&mut st.vm, &data, w, &mut host);
    st.frames = frames;
    info!("level scripts: {} entries, {}/{} expressions ok", rep.entries, rep.ok, rep.exprs);
    for (e, x, err) in rep.errors.iter().take(16) {
        warn!("script entry {e} expr {x}: {err}");
    }
    g.scripts = Some(rep);
}

/// WP-J4: the script part of FUN_00226550 (world update) each frame: due callbacks of world+0x39f0.
/// The camera methods get `&mut Cm`; camera BIFs read it through `PersistentHost` (WP-J5).
#[allow(clippy::too_many_arguments)]
pub fn tick_scripts(t: f32, dt: f32, clk: CmClock, g: &mut LevelGame, st: &mut super::vm_state::SpliceState, w: &World, cm: &mut Cm, emitters: &mut EmitterSystem, rumble: &mut RumbleQueue, mission: i16) {
    if st.level != Some(g.loads) {
        return;
    }
    let LevelGame { vars, data: gd, aliases, table, .. } = g;
    let base = super::splice_host::GameSpliceHost { t, cm: None, emitters, vars: vars.as_ref(), data: gd };
    let level = super::splice_host::LevelSpliceHost { base, mission: mission as i32, aliases, rumble: &mut rumble.0 };
    let super::vm_state::SpliceState { vm, sched, ticks, errors, frames, .. } = st;
    let mut host = super::vm_state::PersistentHost { level, cm: Some(cm), clk, world: Some(w), table: table.as_ref(), sched };
    // WP-J10: messages of the job code (task enter / leave, job start / stop) since the last tick
    super::vm_state::send_messages(vm, errors, frames, &mut host);
    super::vm_state::tick_world_update(vm, errors, &mut host, t, dt);
    *ticks += 1;
    // WP-EP1b: coverage log (no behaviour): every 600 script ticks, the errors raised by callbacks /
    // messages since the level load (HostUnimplemented = UNIMPL) and the recorded (counted-only) class calls
    if *ticks % 600 == 0 {
        script_coverage_log(*ticks, errors, &host.sched.cls.recorded);
    }
    // WP-J5: ASEGA updates of the world update (FUN_00130918 per live ASEGA) with the effect sink
    let mut retract = std::mem::take(&mut sched.objs.retract);
    // WP-J9: FUN_0012f9b8 requests of getter 0x00101910 (ASEG play)
    for (o, svt) in std::mem::take(&mut sched.objs.aseg_play) {
        g.anims.play(w, o as usize, svt);
    }
    g.anims.step(w, dt, clk.dt_real, emitters, &mut rumble.0, Some(cm), clk, &mut retract);
}

/// WP-EP1b: `UNIMPL <n> rec <n> err <n>` + the top distinct messages / recorded callees.
fn script_coverage_log(ticks: u64, errors: &[crate::engine::splice::SpliceError], rec: &[super::methods::cls::Recorded]) {
    use crate::engine::splice::SpliceError;
    let mut un: std::collections::BTreeMap<&str, usize> = Default::default();
    let mut oth: std::collections::BTreeMap<String, usize> = Default::default();
    for e in errors {
        match e {
            SpliceError::HostUnimplemented(s) => *un.entry(s.as_str()).or_default() += 1,
            _ => *oth.entry(e.to_string()).or_default() += 1,
        }
    }
    let other: usize = oth.values().sum();
    let mut rc: std::collections::BTreeMap<u32, usize> = Default::default();
    for r in rec {
        *rc.entry(r.callee).or_default() += 1;
    }
    info!("script coverage at tick {ticks}: UNIMPL {} rec {} err {}", un.values().sum::<usize>(), rec.len(), other);
    let mut v: Vec<_> = un.into_iter().collect();
    v.sort_by(|a, b| b.1.cmp(&a.1));
    for (s, n) in v.iter().take(8) {
        info!("  UNIMPL x{n}: {s}");
    }
    let mut v: Vec<_> = oth.into_iter().collect();
    v.sort_by(|a, b| b.1.cmp(&a.1));
    for (s, n) in v.iter().take(8) {
        info!("  err x{n}: {s}");
    }
    let mut v: Vec<_> = rc.into_iter().collect();
    v.sort_by(|a, b| b.1.cmp(&a.1));
    for (c, n) in v.iter().take(8) {
        info!("  rec x{n}: FUN_{c:08x}");
    }
}

// ---------------------------------------------------------------- rumble

#[derive(Resource, Default)]
pub struct RumbleQueue(pub Vec<RumbleRequest>);

/// FUN_00187e28(pad, pattern, duration) for every queued request (actuator output via the input backend).
pub fn apply_rumble(mut q: ResMut<RumbleQueue>, pad: Option<ResMut<PadInput>>) {
    let Some(mut pad) = pad else { return };
    let pad = &mut *pad;
    for r in q.0.drain(..) {
        let mut act = pad.actuator;
        crate::engine::input::pad::FUN_00187e28(&mut pad.pad, r.pattern, r.duration, &pad.rumble_ctx, &mut |a| act = a);
        pad.actuator = act;
    }
}

// ---------------------------------------------------------------- screenshot

/// `--screenshot <path> --frames N`: N frames after the level geometry is up, save PNG, exit.
#[derive(Resource)]
pub struct ScreenshotRequest {
    pub path: PathBuf,
    pub frames: u32,
    /// absolute frame limit (load failure / never-ready level)
    pub max_frames: u32,
    counted: u32,
    total: u32,
    taken: bool,
    wait: u32,
}

impl ScreenshotRequest {
    pub fn new(path: PathBuf, frames: u32) -> Self {
        Self { path, frames, max_frames: frames + 3000, counted: 0, total: 0, taken: false, wait: 0 }
    }
}

pub fn screenshot_driver(mut req: ResMut<ScreenshotRequest>, g: Res<LevelGame>, ga: Res<GameAudio>, mut commands: Commands, mut exit: MessageWriter<AppExit>) {
    req.total += 1;
    if req.taken {
        req.wait += 1;
        if req.path.exists() || req.wait > 600 {
            info!("screenshot: {} ({})", req.path.display(), if req.path.exists() { "saved" } else { "NOT saved" });
            exit.write(AppExit::Success);
        }
        return;
    }
    // ready = camera framed and level audio decoded (or nothing to decode)
    if g.framed && ga.task.is_none() {
        req.counted += 1;
    }
    if req.counted >= req.frames || req.total >= req.max_frames {
        if req.total >= req.max_frames {
            warn!("screenshot: level not ready after {} frames, capturing anyway", req.total);
        }
        if let Some(d) = req.path.parent() {
            let _ = std::fs::create_dir_all(d);
        }
        let _ = std::fs::remove_file(&req.path);
        commands.spawn(Screenshot::primary_window()).observe(save_to_disk(req.path.clone()));
        req.taken = true;
    }
}

// ---------------------------------------------------------------- WP-J7: class runtime <-> scripts, script visibility

/// Gives the class runtime lent to the scripts (`vm_state::Schedule::cls`) back to `Classes` before
/// `classes::plugin::tick_classes` (which rebuilds / ticks it).
pub fn return_classes(mut c: ResMut<crate::engine::classes::plugin::Classes>, st: Option<NonSendMut<super::vm_state::SpliceState>>) {
    let Some(mut st) = st else { return };
    let l = &mut st.sched.cls;
    if let Some(set) = l.set.take() {
        c.set = set;
        c.host = std::mem::take(&mut l.host);
    }
}

/// Lends the class runtime to the scripts after `tick_classes`: the script tick (FUN_00226550 ->
/// FUN_00149c80) runs method rows / deferred nodes on the live class instances.
pub fn lend_classes(mut c: ResMut<crate::engine::classes::plugin::Classes>, st: Option<NonSendMut<super::vm_state::SpliceState>>, mut jobs: Option<ResMut<crate::engine::player::plugin::PlayerJobs>>) {
    let Some(mut st) = st else { return };
    // WP-J15: methods 1258 / 1284 of the last script tick
    if let Some(j) = jobs.as_mut() {
        // WP-K28: job-manager calls of the class tick (the class runtime is in `Classes` until lent below)
        let q = std::mem::take(&mut c.host.k28_task_calls);
        st.sched.cls.host.k28_task_calls.extend(q);
        super::vm_state::apply_job_calls(&mut st.sched, &mut j.0);
    }
    // WP-J9: read-only job manager view for BIF 100 / 101 / 102
    st.sched.jobs = jobs.as_ref().map(|j| super::vm_state::JobView::of(&j.0)).unwrap_or_default();
    // WP-J10: script messages of the job code since the last frame
    if let Some(j) = &jobs {
        super::vm_state::job_messages(&mut st.sched, &j.0);
    }
    let l = &mut st.sched.cls;
    if l.set.is_none() {
        l.set = Some(std::mem::take(&mut c.set));
        l.host = std::mem::take(&mut c.host);
    }
    // WP-J10: sequence starts of the class tick (interaction points) on the class-114 instances
    l.drain_seq_starts();
}

/// Read-only: an ALO entity whose object the scripts removed (LO Remove / vtable +0x4c, J6
/// `ObjectFields::vis`) is hidden; shown again (Inherited) when a script adds it back. Only entities this
/// system hid are touched, so other visibility owners (playerdraw JT subtree) keep theirs.
pub fn apply_script_vis(st: Option<NonSend<super::vm_state::SpliceState>>, mut q: Query<(Entity, &crate::engine::gfx::game::AloObject, &mut Visibility)>, mut hid: Local<HashSet<Entity>>) {
    let Some(st) = st else { return };
    let vis = &st.sched.objs.vis;
    for (e, o, mut v) in &mut q {
        let off = !vis.drawn(o.0);
        if off && hid.insert(e) {
            *v = Visibility::Hidden;
        } else if !off && hid.remove(&e) {
            *v = Visibility::Inherited;
        }
    }
}

/// WP-J8: marker for a spawned mesh-record primitive entity (world object, mesh record, instance; None =
/// the record itself), so script mesh show/hide (FUN_001f6978 / FUN_001f69f8, `JtVis::set_mesh`) can
/// hide single records. The spawner (gfx `spawn_level_geometry`) inserts it on each prim entity.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScriptMeshRecord {
    pub object: usize,
    pub record: usize,
    pub instance: Option<usize>,
}

/// WP-J8: read-only, like [`apply_script_vis`] for the tagged record entities (`JtVis::record_drawn`).
pub fn apply_script_record_vis(st: Option<NonSend<super::vm_state::SpliceState>>, mut q: Query<(Entity, &ScriptMeshRecord, &mut Visibility)>, mut hid: Local<HashSet<Entity>>) {
    let Some(st) = st else { return };
    let vis = &st.sched.objs.vis;
    for (e, r, mut v) in &mut q {
        let off = !vis.record_drawn(r.object, r.record, r.instance);
        if off && hid.insert(e) {
            *v = Visibility::Hidden;
        } else if !off && hid.remove(&e) {
            *v = Visibility::Inherited;
        }
    }
}

/// WP-J13: the job manager's slots (0x2deb40 / 44 / 48, survive level loads) and how many
/// `JobMgr::toggles` reached the script-visible object state.
#[derive(Resource, Default)]
pub struct GameJobSlots {
    pub slots: super::job_flow::JobSlots,
    pub toggle_seen: usize,
}

/// WP-J13: 0x00161a58 case 5 on the level's new `PlayerJobs` (FUN_0020e890 after the load): task
/// states, current job / task / restart from the slots, job start, task enter; the job / task object
/// toggles (0x1620b0) go to the scripts' object state. REPLACED: DAT_003e1084 (flags of the request that
/// loaded the level) -> 0 (bit 4 = event 0x1e is only set by the job restart paths 0x165280 / 0x165538).
pub fn job_level_loaded(w: Option<Res<World>>, mut jobs: ResMut<crate::engine::player::plugin::PlayerJobs>, mut js: ResMut<GameJobSlots>, st: Option<NonSendMut<super::vm_state::SpliceState>>, lg: Option<Res<LevelGame>>, opt: Option<Res<crate::engine::world::WorldOptions>>) {
    let Some(w) = w.filter(|w| w.is_changed()) else { return };
    let js = &mut *js;
    // WP-J15: DAT_003e1110 current level row (FUN_00210a88) = the row the world was loaded as
    jobs.0.cur_row = opt.map_or(-1, |o| o.0.mission as i32);
    // WP-J14: job post-load (slot 14 FUN_00162780, asegs by oid); WP-J15: save reads through the
    // kind-0x16 handles (`job_flow::saved_job` / `saved_task`; no level save table -> defaults)
    let find = |oid: i32| w.objects.iter().any(|o| o.oid as i32 == oid).then_some(oid);
    let sv = lg.as_ref().and_then(|lg| Some((lg.vars.as_ref()?, &lg.data)));
    for j in 0..jobs.0.jobs.len() {
        let saved = sv.map(|(v, d)| super::job_flow::saved_job(&jobs.0, j, v, d)).unwrap_or_default();
        jobs.0.FUN_00162780(j, saved, &find);
    }
    let saved: Vec<_> = (0..jobs.0.tasks.len()).map(|t| sv.map(|(v, d)| super::job_flow::saved_task(&jobs.0, t, v, d)).unwrap_or_default()).collect();
    super::job_flow::FUN_00161a58_case5(&mut jobs.0, &w, &mut js.slots, 0, &|t| saved.get(t).copied().unwrap_or_default());
    js.toggle_seen = 0;
    if let Some(mut st) = st {
        super::job_flow::apply_toggles(&jobs.0, &mut st.sched.objs, &w, &mut js.toggle_seen);
    }
}

/// WP-J7 systems. The binary must also order its script tick outside [`return_classes`, `lend_classes`]
/// (e.g. after `lend_classes`) for the scripts to see the instances in the same frame.
pub struct ScriptClassesPlugin;
impl Plugin for ScriptClassesPlugin {
    fn build(&self, app: &mut App) {
        use crate::engine::classes::plugin::tick_classes;
        app.add_systems(Update, (return_classes.before(tick_classes), lend_classes.after(tick_classes), apply_script_vis, apply_script_record_vis));
        // WP-J13: job manager level-load path between the job load and the script lending
        app.init_resource::<GameJobSlots>();
        app.add_systems(Update, job_level_loaded.after(crate::engine::player::plugin::load_player_world).before(lend_classes));
    }
}
