//! WP-P2: Bevy wiring of the JT (Sly). Per frame (Update, chained):
//! spawn_player (Jt at `PlayerLevel::spawn`) -> [feature game: pad_to_player copies `input::backend::PadInput`]
//! -> tick_player (0x18bce0 order: input 0x18b7e8 with the camera-relative stick from the CM matrix, JT forces
//! slot69, then the physics driver 0x1f8dd8 for this body, all inside `Jt::update`) -> publish_focus.
//! Anim: every ASEG request of 0x18c250 lands in `JtAnimEvents` (field, jts, jtbs) for the anim/ driver.
//! Camera: `PlayerFocus` + `PlayerFollowWorld` (CmWorld/FollowWorld giving the follow policy the JT state and
//! position, replacing FUN_00132b10(pso, 0x34) / FUN_001a62e8 / vtable +0x194 for the player).
//! P3: step = `PlayerClock` dt (DAT_002dded4) when the game provides it, else 1/60 s; collision world shared via
//! `PlayerLevel` instead of the world's object lists.

use super::{input_from_pad_cm, Jt, JtAnim, JtInput, JT_RADIUS};
use crate::engine::camera::cm::{Cm, CmWorld, Cpdefi};
use crate::engine::camera::follow::{FollowJt, FollowWorld};
use crate::engine::input::pad::PadState;
use crate::engine::physics::geom::V3;
use crate::engine::physics::CollisionWorld;
use bevy_app::{App, Plugin, Update};
use bevy_ecs::prelude::*;
use std::sync::Arc;

pub const PLAYER_DT: f32 = 1.0 / 60.0;

/// WP-P3: the game clock g_clock 0x2dded0 as the player sees it (DAT_002dded0 t, DAT_002dded4 dt). Set by
/// the game layer from `game::clock::Clock` each frame. When present, `tick_player` steps with `dt` and skips
/// the frame when dt <= 0 (paused: the world update 0x226550 is not run). Absent = fixed PLAYER_DT.
#[derive(Resource, Default, Clone, Copy, Debug)]
pub struct PlayerClock {
    pub t: f32,
    pub dt: f32,
}

/// Level data the player needs (set by the level loader / game layer).
#[derive(Resource, Default, Clone)]
pub struct PlayerLevel {
    pub cw: Arc<CollisionWorld>,
    /// spawn point (game::spawn::spawn_position); the sphere is placed JT_RADIUS + 5 above it
    pub spawn: Option<V3>,
}

/// WP-P5: Sly's collision hull (p5_hull::jt_hull_from_parts over his model's collision parts, GEOM 199). Set
/// by the game layer when Sly's model is loaded; absent / None = the P4 sphere contacts (`Jt::contacts`).
/// The current game layer loads no JT model, so this stays None at runtime (documented fallback).
/// WP-P6: set from the level's World by `load_player_world` (p6::player_hull).
#[derive(Resource, Default, Clone)]
pub struct PlayerHull(pub Option<super::p5_hull::JtHull>);

/// WP-P6: the level's player data (p6::P6Level: JT blend tuning 0x1a0300, hull, path targets), rebuilt when
/// the `World` resource changes and applied to the JT at spawn and whenever it changes.
#[derive(Resource, Default, Clone)]
pub struct PlayerWorldData {
    pub lvl: super::p6::P6Level,
    pub generation: u32,
    /// WP-P8: the level's hazard volumes (p8_hazard::Hazards::load)
    pub hz: super::p8_hazard::Hazards,
    /// WP-P9: the SO collision set Sly collides with (`CollisionWorld::build_so`: ALO parts in world space,
    /// no class-5 / 22-27 volumes, no water). Preferred over `PlayerLevel::cw` (which keeps every geom).
    pub cw: Option<Arc<CollisionWorld>>,
}

/// WP-P9 (port addition, verification): env `SLY2_PAD_SCRIPT="a-b:x,y[:buttons];..."` replaces the pad with a
/// script over player frames [a, b) (stick x right / y forward, camera-relative like the pad; buttons = hex
/// pad bits or `X` = BTN_CROSS); env `SLY2_PAD_LOG=N` prints Sly's position every N frames.
#[derive(Resource, Default, Clone, Debug)]
pub struct PadScript {
    pub steps: Vec<(u32, u32, [f32; 2], u16)>,
    pub frame: u32,
    pub prev: u16,
    pub log: u32,
}

impl PadScript {
    pub fn parse(src: &str) -> Self {
        let mut steps = Vec::new();
        for item in src.split(';').map(str::trim).filter(|s| !s.is_empty()) {
            let mut it = item.split(':');
            let (Some(range), Some(st)) = (it.next(), it.next()) else { continue };
            let Some((a, b)) = range.split_once('-') else { continue };
            let (Ok(a), Ok(b)) = (a.trim().parse(), b.trim().parse()) else { continue };
            let xy: Vec<f32> = st.split(',').filter_map(|v| v.trim().parse().ok()).collect();
            let btn = match it.next().map(str::trim) {
                Some("X") | Some("x") => crate::engine::input::pad::BTN_CROSS as u16,
                Some(h) => u16::from_str_radix(h.trim_start_matches("0x"), 16).unwrap_or(0),
                None => 0,
            };
            steps.push((a, b, [*xy.first().unwrap_or(&0.0), *xy.get(1).unwrap_or(&0.0)], btn));
        }
        PadScript { steps, ..Default::default() }
    }
    pub fn from_env() -> Option<Self> {
        let mut s = Self::parse(&std::env::var("SLY2_PAD_SCRIPT").ok()?);
        s.log = std::env::var("SLY2_PAD_LOG").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
        Some(s)
    }
    /// The scripted pad of the next frame (buttons, pressed = edge over the previous frame).
    pub fn next(&mut self) -> PadState {
        let f = self.frame;
        self.frame += 1;
        let (mut st, mut btn) = ([0.0f32; 2], 0u16);
        for &(a, b, s, k) in &self.steps {
            if f >= a && f < b {
                st = s;
                btn |= k;
            }
        }
        let mut p = PadState::default();
        let m = (st[0] * st[0] + st[1] * st[1]).sqrt();
        if m > 1e-4 {
            p.left.x = st[0] / m;
            p.left.y = st[1] / m;
            p.left.mag = m.min(1.0);
        }
        p.buttons = btn as _;
        p.pressed = (btn & !self.prev) as _;
        self.prev = btn;
        p
    }
}

/// WP-P8: the level's jobs (engine::jobs::JobMgr::load); `reloads` are FUN_0022b490 requests of a job
/// restart / completion for the game layer (`JobReload::level_request`).
#[derive(Resource, Default, Clone)]
pub struct PlayerJobs(pub crate::engine::jobs::JobMgr);

/// Pad state the player reads (DAT_002dfc00 mirror; filled from `PadInput` under feature `game`).
#[derive(Resource, Default)]
pub struct PlayerPad(pub Option<PadState>);

#[derive(Component)]
pub struct Player {
    pub jt: Jt,
}

/// ASEG requests of 0x18c250 this frame: (JT field, jts, jtbs).
#[derive(Component, Default)]
pub struct JtAnimEvents(pub Vec<(u32, i32, i32)>);

impl JtAnim for JtAnimEvents {
    fn play_aseg(&mut self, field: u32, jts: i32, jtbs: i32) {
        self.0.push((field, jts, jtbs));
    }
}

/// Sly's state for the camera (follow policy focus).
#[derive(Resource, Default, Clone, Copy, Debug)]
pub struct PlayerFocus {
    pub valid: bool,
    pub pos: V3,
    pub v: V3,
    pub jts: i32,
    /// WP-P24: jt+0x1024 glider camera track set (Jt::cam_glide)
    pub cam_glide: bool,
    /// WP-P24: binocucom camera policy pushed by 0x19cd38 state 1 (jt+0xf00 +0x20 mode)
    pub binoc_cam: Option<i32>,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerLevel>().init_resource::<PlayerPad>().init_resource::<PlayerFocus>();
        app.init_resource::<PlayerWorldData>().init_resource::<PlayerRespawn>().init_resource::<PlayerJobs>();
        app.init_resource::<super::p20::PoCurrent>();
        if let Some(s) = PadScript::from_env() {
            app.insert_resource(s);
        }
        #[cfg(feature = "game")]
        app.add_systems(Update, (load_player_world, spawn_player, apply_player_world, pad_to_player, tick_player, cane_hits, guard_focus, classes_link, publish_focus, publish_respawn).chain());
        #[cfg(not(feature = "game"))]
        app.add_systems(Update, (load_player_world, spawn_player, apply_player_world, tick_player, cane_hits, guard_focus, classes_link, publish_focus, publish_respawn).chain());
        app.init_resource::<PlayerHud>();
        // WP-P20: between the classes' return from the scripts (J7) and their frame
        #[cfg(feature = "game")]
        app.add_systems(Update, po_link.after(tick_player).after(crate::engine::game::app::return_classes).before(crate::engine::classes::plugin::tick_classes));
        #[cfg(not(feature = "game"))]
        app.add_systems(Update, po_link.after(tick_player).before(crate::engine::classes::plugin::tick_classes));
        // WP-P19: script SetJtJts requests (J12 `ClassLink::jt_states`), applied at the end of the frame
        // (outside Update: its NonSend param reorders the Update executor)
        app.add_systems(bevy_app::Last, script_jt_states);
        // WP-P25: FUN_00228750 spawn activation once the class set of the new world is built
        app.init_resource::<PlayerSpawnLists>();
        #[cfg(feature = "game")]
        app.add_systems(Update, spawn_activate.after(crate::engine::classes::plugin::tick_classes).before(crate::engine::game::app::lend_classes));
        // WP-EP1a: P26 pickpocket loot (holders / pool / flights / drops) per world, after the class frame
        app.init_resource::<LootWorld>();
        #[cfg(feature = "game")]
        app.add_systems(Update, loot_link.after(crate::engine::classes::plugin::tick_classes).before(crate::engine::game::app::lend_classes));
    }
}

/// WP-EP1a: the live P26 loot system of the current world (`key` = world identity, `t` = game clock since the load, `PlayerClock` dt).
#[derive(Resource, Default)]
pub struct LootWorld {
    pub key: usize,
    pub sys: super::loot::LootSys,
    pub t: f32,
}

/// WP-EP1a: builds `LootWorld` once per class set (`LootSys::from_class_set`: holders get their loot through
/// FUN_0013cf38) and runs C2cbba0_v15 + the flight timers each frame (`LootSys::update`). Positions /
/// liveness = FUN_001a62e8 / FUN_001a61a0 of the class host. The pickpocket itself (C2cb9f8 slot 50 =
/// `LootSys::ee_sub_13d020`) needs the guard pickpocket-zone volume test (+0x364.., unported, see p13).
#[cfg(feature = "game")]
pub fn loot_link(w: Option<Res<crate::engine::world::World>>, clk: Option<Res<PlayerClock>>, mut lw: ResMut<LootWorld>, classes: Option<ResMut<crate::engine::classes::plugin::Classes>>) {
    use crate::engine::classes::ClassHost;
    let (Some(w), Some(mut c)) = (w, classes) else { return };
    let c = &mut *c;
    if c.set.k31.is_empty() && !w.objects.iter().any(|o| o.class == super::loot::CLASS_LOOT) {
        return;
    }
    let host = std::cell::RefCell::new(&mut c.host);
    let pos = |h: u32| host.borrow_mut().ee_sub_1a62e8(h);
    let alive = |h: u32| host.borrow_mut().ee_sub_1a61a0(h);
    let key = &*w as *const crate::engine::world::World as usize;
    let lw = &mut *lw;
    if lw.key != key {
        lw.sys = super::loot::LootSys::from_class_set(w.objects.iter().map(|o| o.class as u16), &c.set.k31, &[], &pos);
        lw.key = key;
        lw.t = 0.0;
        log::info!("EP1a loot: {} holders, {} loot, {} on holders", lw.sys.guards.len(), lw.sys.loots.len(), lw.sys.guards.iter().filter(|g| g.loot.is_some()).count());
    }
    lw.t += clk.map_or(PLAYER_DT, |c| c.dt.max(0.0));
    lw.sys.update(lw.t, &pos, &alive);
}

pub fn spawn_player(mut cmd: Commands, lvl: Res<PlayerLevel>, hull: Option<Res<PlayerHull>>, q: Query<(), With<Player>>) {
    // WP-P13 (port addition, verification): env `SLY2_SPAWN="x,y,z"` overrides the spawn point
    let env = std::env::var("SLY2_SPAWN").ok().and_then(|s| {
        let v: Vec<f32> = s.split(',').filter_map(|x| x.trim().parse().ok()).collect();
        (v.len() == 3).then(|| [v[0], v[1], v[2]])
    });
    if let (Some(p), true) = (env.or(lvl.spawn), q.is_empty()) {
        let mut jt = Jt::new([p[0], p[1], p[2] + JT_RADIUS + 5.0]);
        // WP-P18 (port addition, verification): env `SLY2_GADGETS="1b,1d"` (hex ids) = acquired gadgets; the ones
        // with a slot mode (GADGETS +0) are equipped on L1 / L2 / R2 in order (stands in for the save data)
        if let Ok(s) = std::env::var("SLY2_GADGETS") {
            let mut slot = 0;
            for g in s.split(',').filter_map(|x| i32::from_str_radix(x.trim(), 16).ok()).filter(|g| (0..0x2b).contains(g)) {
                jt.gad.acquired |= 1 << g;
                if super::p18_gadget::GADGETS[g as usize].mode != -1 && slot < 3 {
                    jt.FUN_00141fc8(slot, g, &mut ());
                    slot += 1;
                }
            }
        }
        jt.hull = hull.and_then(|h| h.0.clone());
        cmd.spawn((Player { jt }, JtAnimEvents::default()));
    }
}

/// WP-P6: rebuild `PlayerWorldData` / `PlayerHull` from a new or changed `World`.
pub fn load_player_world(w: Option<Res<crate::engine::world::World>>, mut d: ResMut<PlayerWorldData>, mut jobs: ResMut<PlayerJobs>, mut cmd: Commands) {
    if let Some(w) = w.filter(|w| w.is_changed()) {
        d.lvl = super::p6::P6Level::load(&w);
        d.hz = super::p8_hazard::Hazards::load(&w);
        d.cw = Some(Arc::new(super::p10::player_collision_world(&w))); // WP-P10: without sensor parts
        jobs.0 = crate::engine::jobs::JobMgr::load(&w);
        d.generation += 1;
        cmd.insert_resource(PlayerHull(d.lvl.hull.clone()));
    }
}

/// WP-P6: apply the level data to the player (once per `PlayerWorldData` generation).
pub fn apply_player_world(d: Res<PlayerWorldData>, mut q: Query<&mut Player>, mut done: Local<u32>) {
    if d.generation == 0 || *done == d.generation {
        return;
    }
    let mut any = false;
    for mut p in &mut q {
        d.lvl.apply(&mut p.jt);
        any = true;
    }
    if any {
        *done = d.generation;
    }
}

#[cfg(feature = "game")]
pub fn pad_to_player(pad: Option<Res<crate::engine::input::backend::PadInput>>, mut pp: ResMut<PlayerPad>) {
    pp.0 = pad.map(|p| p.pad);
}

pub fn tick_player(
    lvl: Res<PlayerLevel>,
    pad: Res<PlayerPad>,
    cm: Option<Res<Cm>>,
    clk: Option<Res<PlayerClock>>,
    mut wd: ResMut<PlayerWorldData>,
    mut jobs: ResMut<PlayerJobs>,
    mut q: Query<(&mut Player, &mut JtAnimEvents)>,
    mut script: Option<ResMut<PadScript>>,
    mut emitters: Option<ResMut<crate::engine::audio::emitter::EmitterSystem>>,
) {
    // WP-EP1a: a script with only app-frame (`g`) items leaves the player on the real (scripted) pad
    let scripted = script.as_mut().filter(|s| !q.is_empty() && !s.steps.is_empty()).map(|s| s.next());
    let pad_now = scripted.or(pad.0);
    let inp = match &pad_now {
        Some(p) => {
            let fwd = cm.as_ref().map(|c| [c.mat[0][0], c.mat[0][1], c.mat[0][2]]).unwrap_or([1.0, 0.0, 0.0]);
            input_from_pad_cm(p, fwd)
        }
        None => JtInput::default(),
    };
    let clk_t = clk.as_ref().map(|c| c.t);
    let dt = match clk {
        Some(c) if c.dt <= 0.0 => return,
        Some(c) => c.dt,
        None => PLAYER_DT,
    };
    for (mut pl, mut ev) in &mut q {
        ev.0.clear();
        if let Some(p) = &pad_now {
            pl.jt.feed_pad(p);
        }
        // WP-P8: DAT_003e0f9c for 0x161a58 / 0x1a2388, hazards after the JT update, death events -> jobs
        pl.jt.dmg_env.job = jobs.0.cur.is_some();
        let n_ev = pl.jt.dmg.ev.len();
        // WP-P14: raw stick joy+0x10/+0x14 and camera rows joy+0x40.. for FUN_001c4768 (jts 8)
        if let Some(p) = &pad_now {
            pl.jt.p5w.stick = [p.left.x * p.left.mag, p.left.y * p.left.mag];
        }
        pl.jt.p5w.cam_rows = cm.as_ref().map(|c| [0, 1, 2].map(|i| [c.mat[i][0], c.mat[i][1], c.mat[i][2]]));
        // WP-P24: DAT_002de3a0 stand-in (binocucom look yaw) = yaw of the CM forward row
        pl.jt.cam_yaw = cm.as_ref().map(|c| crate::engine::camera::math::FUN_0011bc88([c.mat[0][0], c.mat[0][1], c.mat[0][2]]).0);
        let cw = wd.cw.clone().unwrap_or_else(|| lvl.cw.clone());
        pl.jt.update(&cw, inp, dt, &mut *ev);
        if let Some(s) = script.as_ref().filter(|s| s.log > 0 && s.frame % s.log == 0) {
            eprintln!("SLY2_PAD_LOG f{} pos {:?} v {:?} jts {} jtbs {}", s.frame, pl.jt.phys.pos, pl.jt.phys.v, pl.jt.jts, pl.jt.jtbs);
        }
        let now = clk_t.unwrap_or(pl.jt.t);
        super::p8_hazard::hazard_step(&mut pl.jt, &mut wd.hz, now, &mut *ev);
        super::p8_hazard::job_events(&pl.jt, n_ev, &mut jobs.0);
        // WP-P16: cane swing sounds / jts 0x11 job events
        super::p16::drain(&mut pl.jt, emitters.as_deref_mut(), &mut jobs.0);
        // WP-P18: gadget sounds (FUN_001fbb90 + FUN_001fb2f8(e, 0), unpositioned)
        let sfx = std::mem::take(&mut pl.jt.gad.sfx);
        if let Some(es) = emitters.as_deref_mut() {
            for snd in sfx {
                let id = es.new_emitter_1fbb90(snd as i32);
                es.set_unpositioned_1fb2f8(id, 0);
            }
        }
    }
}

/// WP-P25: the class-0x2a run-time lists FUN_00228750 reads (Splice CMethods 0x10ba70 / 0x10bb38 / 0x10bbe0;
/// filled by the script host when it wires them, empty otherwise). Cleared on a new world.
#[derive(Resource, Default, Clone)]
pub struct PlayerSpawnLists(pub super::p25::SpawnLists);

/// WP-P25: FUN_0020e890 -> FUN_00228750 on the spawn object FUN_00227fe8 found (`LevelGame::last_spawn` set),
/// once per class-set build (keyed by the class host's world). REPLACED order: after the first class frame of
/// the world (the original runs it before the first world update; the class set is built in `tick_classes`).
#[cfg(feature = "game")]
pub fn spawn_activate(
    g: Option<Res<crate::engine::game::app::LevelGame>>,
    classes: Option<ResMut<crate::engine::classes::plugin::Classes>>,
    cm: Option<ResMut<Cm>>,
    mut lists: ResMut<PlayerSpawnLists>,
    mut done: Local<usize>,
) {
    let (Some(g), Some(mut c)) = (g, classes) else { return };
    let c = &mut *c;
    let Some(w) = c.host.world.clone() else { return };
    let key = Arc::as_ptr(&w) as usize;
    if *done == key {
        return;
    }
    *done = key;
    let l = std::mem::take(&mut lists.0);
    let Some(spawn) = g.spawn.filter(|&s| g.last_spawn != (0, 0) && w.objects.get(s).is_some_and(|o| o.class == crate::engine::game::spawn::CLASS_SPAWN)) else { return };
    let r = super::p25::FUN_00228750(&w, spawn, &l, &mut c.set, &mut c.host, cm.map(|m| m.into_inner()));
    log::info!("P25 FUN_00228750 spawn {spawn}: warps {} reset {} placed {:?}", r.warps, r.warp_reset.len(), r.placed);
}

/// WP-P20: the controlled character (FUN_0020e890 po selection once per world, then each frame the class set
/// reads the shadow JT's joystick `k18_feed_joy` and `po_call` answers from the shadow JT).
pub fn po_link(
    w: Option<Res<crate::engine::world::World>>,
    jobs: Res<PlayerJobs>,
    mut cur: ResMut<super::p20::PoCurrent>,
    mut q: Query<&mut Player>,
    classes: Option<ResMut<crate::engine::classes::plugin::Classes>>,
) {
    let (Some(w), Some(mut c)) = (w, classes) else { return };
    let Some(mut pl) = q.iter_mut().next() else { return };
    let c = &mut *c;
    let key = &*w as *const crate::engine::world::World as usize;
    if cur.world_key != key {
        *cur = super::p20::PoCurrent::default();
        if super::p20::apply_po(&w, super::p20::oid_po_current(&jobs.0), &mut c.set, &mut c.host, &mut cur) {
            cur.world_key = key;
            log::info!("P20 po: oidPoCurrent {} -> {:?} class {}", super::p20::oid_po_current(&jobs.0), cur.obj, cur.class);
        }
    }
    // WP-P22: mid-mission switches (slot 78 through the class set, e.g. K22 sequence action 7)
    if std::mem::take(&mut c.host.po_release) {
        super::p22::set_active_character(&w, &mut c.set, &mut c.host, None);
    }
    // WP-P23: slot-78 callers of the last class frame (FUN_002206c8 / FUN_001c8730 / Bentley / Murray states)
    let q = std::mem::take(&mut c.host.p23_switch);
    if cur.world_key == key && !super::p23::apply_switches(&w, &mut c.set, &mut c.host, q).is_empty() {
        log::info!("P23 slot-78 switch -> {:?}", c.set.k18_active);
    }
    if cur.world_key == key && super::p22::follow_active(&w, c.set.k18_active, &mut cur, &mut pl.jt) {
        log::info!("P22 po switch -> {:?} class {}", cur.obj, cur.class);
    }
    match cur.obj {
        Some(o) if c.set.k18_active == Some(o) => {
            // WP-P23: the po's live ASEGAs for playerdraw::po
            cur.plays = super::p23::po_plays(&c.host.k25, o);
            c.set.k18_feed_joy(pl.jt.joy);
            // WP-P22: the class PO fields of last frame's slot 15 / 69 drive the body (p22::ee_sub_204108)
            let bb = std::mem::take(&mut c.host.po_bb58);
            cur.bb58 = bb.iter().rev().find(|r| r.0 == o).map(|r| (r.1, r.2));
            pl.jt.po_move = super::p22::po_move_of(&mut c.set, o, cur.class, &cur.loco, &bb);
            c.host.po = Some(super::p20::PoHost { obj: o, jt: Box::new(pl.jt.clone()) });
        }
        _ => {
            c.host.po = None;
            c.host.po_bb58.clear();
            cur.bb58 = None;
            pl.jt.po_move = None;
        }
    }
}

/// WP-P11: Sly's cane hits on the class set's guards (`p11_cane::cane_hits_to_guards`; no-op without classes).
pub fn cane_hits(mut q: Query<(&mut Player, &mut JtAnimEvents)>, classes: Option<ResMut<crate::engine::classes::plugin::Classes>>) {
    let Some(mut c) = classes else { return };
    let c = &mut *c;
    for (mut p, mut ev) in &mut q {
        super::p11_cane::cane_hits_to_guards(&mut p.jt, &mut c.set, &mut c.host, &mut *ev);
    }
}

/// WP-P13: JT focus selection FUN_00140558 over the class set's guards (`p13`; no-op without classes).
pub fn guard_focus(mut q: Query<&mut Player>, classes: Option<Res<crate::engine::classes::plugin::Classes>>) {
    use crate::engine::classes::Inst;
    let Some(c) = classes else { return };
    let cands: Vec<super::p13::FocusCand> = c
        .set
        .k6
        .iter()
        .filter_map(|(i, x)| match x {
            Inst::Guard(g) => Some(super::p13::FocusCand {
                handle: *i as u32 + 1,
                pos: g.body.pos,
                is_3a: super::p4::FUN_00132b10(g.class as i32, 0x3a),
                prio: super::p13::guard_focus_prio(g.f_2c0),
            }),
            _ => None,
        })
        .collect();
    for mut p in &mut q {
        p.jt.FUN_00140558(&cands);
    }
}

/// WP-P12: the HUD counters the player layer publishes (P7 Dmg hp, K8 clue bottles DAT_003e1bf4 of
/// DAT_003e1bf8, K8 loot save-variable counts, coin pickups taken this level). The UI reads them.
#[derive(Resource, Default, Clone, Debug, PartialEq)]
pub struct PlayerHud {
    pub hp: i32,
    pub hp_max: i32,
    pub bottles: i32,
    pub bottles_total: i32,
    pub loot: std::collections::BTreeMap<i32, i32>,
    /// slot 80 gifts consumed (hp, mp) and K7 hits taken / refused, for diagnostics
    pub gifts: usize,
    pub hits_taken: usize,
    pub hits_refused: usize,
    /// WP-P18: gadget power / slots (ui: power bar mp / mp_max flashing after t_1304, slot icons L1 L2 R2)
    pub gadgets: super::p18_gadget::GadgetHud,
}

/// WP-P12 REPLACED contact solver for the K7 slot 73 list: Sly's sphere touching a guard body (horizontal
/// distance < this, |dz| < 2x) yields a contact pair on the guard. The guard-side part's hit kind (+0x18) /
/// amount (+0x14) come from the attack records (class 120, chase goal FUN_001758a0) which are unported, so
/// these contacts carry kind 0 (body contact: slot 73 delivers no hit for them).
pub const GUARD_CONTACT_REACH: f32 = 2.0 * JT_RADIUS;

/// WP-P12 (K7/K8 hookup): the player <-> class-set links the original makes through the JT object.
/// - player record +0x15c (FUN_001c4bd8 / FUN_001c5b20): alive +0x15c (init 1 by FUN_001c3d70, 0 while a
///   damage state runs, 1 again on its exit = FUN_001a0b40) -> `EngineHost::player_rec`, position ->
///   `player_pos`. REPLACED: radius +0x164 stands for the ALO part loop of FUN_001c5b20 (Sly's parts are
///   not in the class host): JT_RADIUS.
/// - JT slot 80 0x00144268(jt, hp, mp) per `player_gifts`: FUN_001c4228 script msg 0xc (recorded),
///   hp = min(hpCur + hp, max) (0x00144188); mp only with the +0x12fc pool pointer (none in the port: recorded).
/// - K7 hits (FUN_001f7598 with the player as target) -> P7 TakeDamage FUN_001a19a0 (slot 74 0x140f50) with
///   the source guard's position / +0xfbc mode.
/// - k7_contacts from Sly's sphere vs the guard bodies (`GUARD_CONTACT_REACH`).
/// - `PlayerHud` from the Dmg state and the class globals.
pub fn classes_link(
    mut q: Query<(&mut Player, &mut JtAnimEvents)>,
    classes: Option<ResMut<crate::engine::classes::plugin::Classes>>,
    mut hud: ResMut<PlayerHud>,
) {
    let Some((mut p, mut ev)) = q.iter_mut().next() else { return };
    if let Some(mut c) = classes {
        let c = &mut *c;
        link_classes(&mut p.jt, &mut c.set, &mut c.host, &mut *ev, &mut hud);
    }
    hud.hp = p.jt.dmg.hp;
    hud.hp_max = p.jt.dmg.hp_max;
    hud.gadgets = p.jt.gadget_hud();
}

/// WP-P19 (J12): drains `ClassLink::jt_states` (method 0x0010b028 on the class-53 JT -> FUN_0018d568 ->
/// SetJtJts FUN_0018c250(jt, jts, -1)) each frame onto the player's JT.
pub fn script_jt_states(
    mut q: Query<(&mut Player, &mut JtAnimEvents)>,
    w: Option<Res<crate::engine::world::World>>,
    st: Option<NonSendMut<crate::engine::game::vm_state::SpliceState>>,
) {
    let Some(mut st) = st else { return };
    if st.sched.cls.jt_states.is_empty() {
        return;
    }
    // not a SpliceState change other systems should react to
    let states = std::mem::take(&mut st.bypass_change_detection().sched.cls.jt_states);
    let Some((mut p, mut ev)) = q.iter_mut().next() else { return };
    let jt_obj = w.as_deref().and_then(|w| w.objects.iter().position(|o| o.class == 53)).map(|o| o as u32);
    drain_jt_states(&mut p.jt, states, jt_obj, &mut *ev);
}

/// The work of [`script_jt_states`]: FUN_0018d568's SetJtJts(jt, jts, -1) for each request on the player's
/// JT object (`jt_obj`; requests on other objects have no player to drive and are dropped).
pub fn drain_jt_states(jt: &mut super::Jt, states: Vec<(u32, i32)>, jt_obj: Option<u32>, anim: &mut dyn JtAnim) {
    for (obj, jts) in states {
        if Some(obj) == jt_obj {
            // 0x0018d568 -> FUN_0018c250(jt, jts, -1)
            jt.set_jts(jts, -1, anim);
        }
    }
}

/// The work of [`classes_link`] on plain values (tests drive it without Bevy too).
pub fn link_classes(
    jt: &mut super::Jt,
    set: &mut crate::engine::classes::ClassSet,
    host: &mut crate::engine::classes::plugin::EngineHost,
    anim: &mut dyn JtAnim,
    hud: &mut PlayerHud,
) {
    use crate::engine::classes::guard::{slots::Contact, PLAYER_HANDLE};
    // slot 80 gifts
    for (hp, mp) in std::mem::take(&mut host.player_gifts) {
        jt.calls.push((0x001c4228, 0xc));
        let v = jt.dmg.hp + hp;
        jt.dmg.FUN_00144188(v);
        if mp != 0 {
            jt.calls.push((0x001441c0, mp));
        }
        hud.gifts += 1;
    }
    // K7 hits on Sly -> TakeDamage
    for hit in std::mem::take(&mut host.player_hits) {
        if hit.target != PLAYER_HANDLE {
            continue;
        }
        let src = (hit.src as usize).checked_sub(1).and_then(|i| set.k6_guard(i)).map(|g| (g.body.pos, g.f_fbc));
        let (pos, fbc) = src.unwrap_or(([hit.pos[0], hit.pos[1], hit.pos[2]], 0));
        let env = jt.dmg_env;
        if super::p7_damage::FUN_001a19a0(jt, &hit.to_p7(pos, fbc), &env, anim) {
            hud.hits_taken += 1;
        } else {
            hud.hits_refused += 1;
        }
    }
    // player record +0x15c / position for the pickups and sensors
    let pos = jt.phys.pos;
    host.player_rec = Some(crate::engine::classes::k8::PickRec { alive: (jt.dmg.state == -1) as i32, radius: JT_RADIUS, obj: PLAYER_HANDLE });
    host.player_pos = pos;
    // body contacts with the guards (REPLACED solver)
    for (gi, _) in set.k6.iter() {
        let Some(g) = set.k6_guard(*gi) else { continue };
        let d = [g.body.pos[0] - pos[0], g.body.pos[1] - pos[1], g.body.pos[2] - pos[2]];
        if d[0] * d[0] + d[1] * d[1] < GUARD_CONTACT_REACH * GUARD_CONTACT_REACH && d[2].abs() < 2.0 * GUARD_CONTACT_REACH {
            host.k7_contacts.push((*gi, Contact { kind: 0, other: PLAYER_HANDLE, pos: [pos[0], pos[1], pos[2], 1.0], ..Default::default() }));
        }
    }
    hud.bottles = set.g.g_3e1bf4;
    hud.bottles_total = set.g.g_3e1bf8;
    hud.loot = set.g.loot.clone();
}

/// WP-P7: level reload requested by Sly's death (0x161a58 case 0x10); the game turns it into the world
/// request with `Respawn::level_request(current descriptor)`.
#[derive(Resource, Default, Clone, Copy, Debug)]
pub struct PlayerRespawn(pub Option<super::p7_damage::Respawn>);

///
/// WP-P21: the game layer (game/flow_app.rs `collect_flow`, J15) consumes `PlayerRespawn` and runs the 1:1
/// FUN_0022b490 level reload (flags | 2), which respawns Sly with the world; the player layer only publishes
/// the request (the P18 in-place stand-in `respawn_in_place` is no longer called here).
pub fn publish_respawn(mut q: Query<&mut Player>, mut r: ResMut<PlayerRespawn>) {
    for mut p in &mut q {
        if let Some(x) = p.jt.respawn.take() {
            // WP-P21: death goes only through PlayerRespawn -> game/ flow (FUN_0022b490 1:1 level reload,
            // flags | 2; J15). `respawn_in_place` is no longer called here (kept for tests / tools).
            r.0 = Some(x);
        }
    }
}

/// WP-P18: the JT half of the death reload (see `publish_respawn`).
pub fn respawn_in_place(jt: &mut Jt, spawn: V3, lvl: Option<&super::p6::P6Level>) {
    let mut n = Jt::new([spawn[0], spawn[1], spawn[2] + JT_RADIUS + 5.0]);
    n.hull = jt.hull.take();
    n.gad = std::mem::take(&mut jt.gad);
    n.gad.refill(false, jt.dmg.hp == 0);
    n.gad.slots.iter_mut().for_each(|s| s.state = 0);
    n.gad.time_scale = 1.0;
    n.dmg.hp = jt.dmg.hp;
    n.dmg.FUN_001440d0(true);
    n.dmg_env = jt.dmg_env;
    n.is_player = jt.is_player;
    if let Some(l) = lvl {
        l.apply(&mut n);
    }
    *jt = n;
}

pub fn publish_focus(q: Query<&Player>, mut f: ResMut<PlayerFocus>) {
    *f = match q.iter().next() {
        Some(p) => PlayerFocus { valid: true, pos: p.jt.phys.pos, v: p.jt.phys.v, jts: p.jt.jts, cam_glide: p.jt.cam_glide, binoc_cam: (p.jt.binoc.state == 1).then_some(p.jt.binoc.mode) },
        None => PlayerFocus::default(),
    };
}

/// Follow-camera world view of the player (pso ignored: the player is the only JT).
pub struct PlayerFollowWorld(pub PlayerFocus);

impl CmWorld for PlayerFollowWorld {
    fn get_cpdefi(&mut self, _pso: u32) -> Cpdefi {
        let mut c = Cpdefi::default();
        let (p, v) = (self.0.pos, self.0.v);
        c.pos_base = [p[0], p[1], p[2], 1.0];
        c.v_base = [v[0], v[1], v[2], 0.0];
        c
    }
    fn jt_state(&mut self, _pso: u32) -> Option<i32> {
        self.0.valid.then_some(self.0.jts)
    }
}

impl FollowWorld for PlayerFollowWorld {
    fn jt(&mut self, _pso: u32) -> Option<FollowJt> {
        self.0.valid.then_some(FollowJt { state: self.0.jts, pos_500: self.0.pos, f_10e4: 0 })
    }
    fn so_pos(&mut self, _pso: u32) -> V3 {
        self.0.pos
    }
}
