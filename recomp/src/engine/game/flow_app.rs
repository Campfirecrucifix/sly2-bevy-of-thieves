//! WP-J14: Bevy side of `flow`: the binary writes `FrameClock` after its clock step (FUN_0014d0b8) and
//! drains `FlowRequests` into its level transition; this plugin turns the posted FUN_0022b490 requests,
//! the player's death reload (`PlayerRespawn`) and the job manager's reloads into `FlowRequests`, runs the
//! per-frame job / task update (slot 15 of classes 132 / 133), the playtime hook FUN_00161478 and the job
//! autosave (slot file).
use bevy::prelude::*;

use super::app::{GameJobSlots, LevelGame};
use super::flow::{self, Post};
use super::save::MemoryCard;
use crate::engine::player::plugin::{PlayerJobs, PlayerRespawn};

/// Game clock of this frame (g_clock t / dt) and whether FUN_0014d0b8 advanced the game time.
#[derive(Resource, Default, Clone, Copy, Debug)]
pub struct FrameClock {
    pub t: f32,
    pub dt: f32,
    pub advanced: bool,
}

/// FUN_0022b490 requests for the level transition (the binary resolves them with `flow::resolve`).
#[derive(Resource, Default, Debug)]
pub struct FlowRequests(pub Vec<Post>);

/// Save slot files and the slot of the running game (DAT_002deb4c).
#[derive(Resource)]
pub struct SaveSlots {
    pub card: MemoryCard,
    pub slot: usize,
}

impl Default for SaveSlots {
    fn default() -> Self {
        SaveSlots { card: MemoryCard::next_to_exe(), slot: 0 }
    }
}

/// Posted requests, the death reload and job reloads -> `FlowRequests` (the job manager slots go with
/// the request that set them).
pub fn collect_flow(mut fr: ResMut<FlowRequests>, mut js: ResMut<GameJobSlots>, mut resp: ResMut<PlayerRespawn>, mut jobs: ResMut<PlayerJobs>, lg: Option<Res<LevelGame>>) {
    for p in flow::take_posted() {
        if let Some(s) = p.slots {
            js.slots = s;
        }
        fr.0.push(p);
    }
    if let Some(mut r) = resp.0.take() {
        // WP-J17: FUN_00161a58 case 0x10 reloads at DAT_003e1118/111c = world+0x3c88/+0x3c8c (last spawn)
        if let Some(lg) = lg.as_ref() {
            (r.spawn, r.spawn_parent) = lg.last_spawn;
        }
        fr.0.push(flow::respawn_post(&r));
    }
    let reloads = std::mem::take(&mut jobs.0.reloads);
    for r in &reloads {
        let p = flow::job_post(&jobs.0, r, &mut js.slots);
        fr.0.push(p);
    }
}

/// Slot 15 of every job / task, FUN_00161478 when the game time advanced, autosave requests.
pub fn job_frame(mut fc: ResMut<FrameClock>, mut jobs: ResMut<PlayerJobs>, lg: Option<ResMut<LevelGame>>, ui: Option<Res<crate::engine::ui::plugin::UiGame>>, slots: Res<SaveSlots>, w: Option<Res<crate::engine::world::World>>) {
    if !fc.advanced {
        return;
    }
    fc.advanced = false;
    // WP-J15: ASEG "playing" list (FUN_002121e8) from the live ASEGAs of the level
    if let (Some(lg), Some(w)) = (lg.as_ref(), w.as_ref()) {
        let live: Vec<i32> = lg.anims.players.iter().filter(|p| p.state.alive).filter_map(|p| w.objects.get(p.object)).map(|o| o.oid as i32).collect();
        super::job_flow::feed_playing(&mut jobs.0, &live);
    }
    jobs.0.update(fc.t, fc.dt);
    let Some(mut lg) = lg else { return };
    let lg = &mut *lg;
    // WP-J15: the job / task save-variable writes of this frame (FUN_00258200)
    if let Some(vars) = lg.vars.as_ref() {
        super::job_flow::flush_vars(&mut jobs.0, vars, &mut lg.data);
    }
    if let (Some(vars), Some(ui)) = (lg.vars.as_ref(), ui.as_ref()) {
        // DAT_002e0c20 (level kind) REPLACED -> 0
        // WP-J15: episode timer DAT_003e0fe8[FUN_00210ae0()]
        let ep = flow::FUN_00210ae0(jobs.0.cur_row, lg.table.as_ref());
        let paths = flow::episode_paths(vars);
        let ep_dt = usize::try_from(ep).ok().filter(|&e| e < 9).and_then(|e| paths[e][0].clone());
        flow::FUN_00161478_full(fc.dt, vars, ui.paths.dt.as_deref(), ep_dt.as_deref(), &mut lg.data, Some(&mut jobs.0), 0.0);
    }
    if jobs.0.autosave > 0 {
        jobs.0.autosave = 0;
        match flow::save_slot(&slots.card, slots.slot, lg.vars.as_ref(), &mut lg.data) {
            Ok(()) => info!("flow: autosave slot {}", slots.slot),
            Err(e) => warn!("flow: autosave slot {} failed: {e}", slots.slot),
        }
    }
}

/// WP-J16: DAT_002f6334, the card slot of the title (1-based, 0 = none). REPLACED: the boot card check
/// FUN_001acc38(0x2e6550, 1) / card manager choice -> the slot file of `SaveSlots::slot` exists.
pub fn title_slot(slots: &SaveSlots) -> i32 {
    if slots.card.slot_path(slots.slot).is_file() { slots.slot as i32 + 1 } else { 0 }
}

/// WP-J16: FUN_001dd920 START branch with DAT_002f6334 != 0 (title Continue): FUN_001acc38 + 0x1ad320
/// (slot file -> live blob) + FUN_00165e38 (`flow::FUN_00165e38_full`: hide the title on a fresh blob, else
/// the episode level or the hideout). Runs before the UI's FUN_001dd920 (which handles the slot-0 New Game
/// branch with DAT_002f6334 = 0); the START press is consumed here.
pub fn title_continue(
    mut pad: Option<ResMut<crate::engine::input::backend::PadInput>>,
    ui: Option<ResMut<crate::engine::ui::plugin::UiGame>>,
    lg: Option<ResMut<LevelGame>>,
    slots: Res<SaveSlots>,
) {
    let (Some(pad), Some(mut ui), Some(mut lg)) = (pad.as_mut(), ui, lg) else { return };
    if !ui.title || ui.menu.depth != 0 || pad.pad.pressed & 0x800 == 0 {
        return;
    }
    let slot = title_slot(&slots);
    if slot == 0 {
        return;
    }
    let lg = &mut *lg;
    let Some(vars) = lg.vars.as_ref() else { return };
    let paths = flow::StartPaths::new(vars);
    let d = match flow::FUN_001ad320(&slots.card, slots.slot, vars, paths.dt.as_deref()) {
        Ok(d) => d,
        Err(e) => {
            warn!("flow: continue slot {slot}: {e:?}");
            return;
        }
    };
    crate::engine::input::pad::FUN_00187d80(&mut pad.pad, 0x800);
    lg.data = d;
    let st = flow::FUN_00165e38_full(true, vars, &paths, &lg.data, lg.table.as_ref(), slot);
    info!("flow: title continue slot {slot}: {st:?} (completion {}%)", flow::FUN_001d82b8(vars, &paths, &lg.data));
    if st == flow::Start::HideTitle {
        let vib = ui.menu.opts.vibration;
        crate::engine::ui::title::FUN_001d54b8(vib, vars, &ui.paths, &mut lg.data);
    }
    // FUN_001dd888: the title screen hides
    ui.title = false;
}

pub struct GameFlowPlugin;
impl Plugin for GameFlowPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FrameClock>().init_resource::<FlowRequests>().init_resource::<SaveSlots>().init_resource::<GameJobSlots>();
        app.add_systems(Update, (job_frame, collect_flow).chain().after(super::app::job_level_loaded));
        // WP-J16: after the pad update (PreUpdate), before the UI's FUN_001dd920 (Update)
        app.add_systems(RunFixedMainLoop, title_continue.in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop));
    }
}
