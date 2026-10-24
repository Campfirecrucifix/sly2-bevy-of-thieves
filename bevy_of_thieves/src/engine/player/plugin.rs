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
    /// WP-EP1d: the class host's `so_gen` `cw` was built for (FUN_001f6ac8 / 001f6b48 part links)
    pub so_gen: u32,
    /// FEEL R76: SO parts with +0x10 bit 48 set in the live image (FUN_001f68a8: worn TNT barrel, mounted turret);
    /// the collision gathers FUN_0020f948 / FUN_0020fa10 skip them
    pub b48: Vec<(usize, usize)>,
    /// FEEL R87: last posed bone worlds (classes host k29_bones) re-applied to the SO part colliders after a rebuild
    /// (an ASEGA that ended leaves its bone at the last pose: the safehouse door stays open)
    pub pose_held: std::collections::HashMap<(usize, usize), crate::engine::gfx::scene::Affine>,
}

/// FEEL R76: drop the bit-48 parts (`PlayerWorldData::b48`) from a player collision set.
pub fn drop_b48(cw: &mut crate::engine::physics::CollisionWorld, b48: &[(usize, usize)]) {
    if !b48.is_empty() {
        cw.colliders.retain(|c| !matches!(c.source, crate::engine::physics::query::Source::AloPart(k) if b48.contains(&(c.object, k)) || b48.contains(&(c.object, usize::MAX))));
    }
}

/// FEEL R76: parts of the live image's class-88 (TNT barrel) / class-72 (turret) copies with +0x10 bit 48 set
/// (FUN_00183b80 worn -> FUN_001f68a8(barrel, 1); FUN_0017fd08 mode 3 -> FUN_001f68a8(turret, 1)). The other setter
/// (class-89 water) is already outside the player set. `SLY2_FEEL_B48=0` = off (empty).
pub fn live_b48(m: &mut crate::engine::classes::q::live::LiveMem, w: &crate::engine::world::World) -> Vec<(usize, usize)> {
    use crate::engine::classes::q::q2mem::QMem;
    let mut out = Vec::new();
    if std::env::var("SLY2_FEEL_B48").is_ok_and(|v| v == "0") {
        return out;
    }
    for (i, a) in m.obj_addr.clone().into_iter().enumerate() {
        if a == 0 || !w.objects.get(i).is_some_and(|o| matches!(o.class, 72 | 88)) {
            continue;
        }
        let (n, p) = (QMem::r32(m, a + 0xe0), QMem::r32(m, a + 0xe4));
        if p == 0 || n > 256 {
            // REPLACED: the live image loads only the ALO skeletons (LiveMem::load_alos), no part records (+0xe0 0),
            // so FUN_001f68a8 has nothing to mark: a class-88 barrel worn with +0x18c set (FUN_00183b80 st 2 ->
            // FUN_001f68a8(1), leaving st 2 -> 0) marks every part (usize::MAX). Class 72 (FUN_0017fd08) needs the
            // part records (not mirrored here).
            if w.objects[i].class == 88 && QMem::r32(m, a + 0x17c) == 2 && QMem::r32(m, a + 0x18c) != 0 {
                out.push((i, usize::MAX));
            }
            continue;
        }
        for k in 0..n {
            if QMem::r32(m, p + 0xb0 * k + 0x14) & 0x1_0000 != 0 {
                out.push((i, k as usize));
            }
        }
    }
    out
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
    /// WIRE-B3: guard-sensor facts of Sly (Jt::guard_tgt)
    pub tgt: crate::engine::classes::guard::sensor::Tgt,
    /// FEEL R25: jt +0x1a8 facing yaw (CPDEFI rad_home source)
    pub yaw: f32,
    /// FEEL R25: jt +0x238 ground contact z / +0x208 contact present (FUN_00141588)
    pub z_contact: f32,
    pub ground: bool,
    /// FEEL R25: CPDEFI pos_base z = jt +0x1050 (FUN_00141640, computed by follow_camera) and the clock
    /// (FUN_001f58f0 t_move_last = DAT_002dded0)
    pub z_cpdefi: f32,
    pub t: f32,
    /// FEEL R25: jt +0x4c8 / +0x4cc (Attach kind / sub) for the follow camera
    pub attach: (i32, i32),
    /// FEEL R37: jt +0x2dc (JMP +0x1c body state, Jt::jtbs) for the class-15 track volume gate
    pub jtbs: i32,
    /// FEEL R112: CPDEFI +0x00 bit 0 (follow off) as 0x18a428 / 0x141640 write it (Jt::cpdefi_follow_off)
    pub follow_off: bool,
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
        // FEEL R87: load_player_world reads the class host (so_gen / so_parts / k29_bones): only valid between
        // return_classes and lend_classes; unordered it saw the lent stand-in on ~all frames and applied the
        // run-time part links (safehouse door FUN_001f6ac8) late or never
        app.add_systems(Update, (load_player_world.after(crate::engine::game::app::return_classes).before(crate::engine::game::app::lend_classes), spawn_player, apply_player_world, pad_to_player, tick_player, cane_hits, guard_focus, classes_link, publish_focus, publish_respawn).chain());
        #[cfg(not(feature = "game"))]
        app.add_systems(Update, (load_player_world, spawn_player, apply_player_world, tick_player, cane_hits, guard_focus, classes_link, publish_focus, publish_respawn).chain());
        app.init_resource::<PlayerHud>();
        // WP-P20: between the classes' return from the scripts (J7) and their frame
        #[cfg(feature = "game")]
        app.add_systems(Update, po_link.after(tick_player).after(crate::engine::game::app::return_classes).before(crate::engine::classes::plugin::tick_classes));
        // WIRE-B2: player-requested character switch (port addition) right before po_link follows it
        #[cfg(feature = "game")]
        app.add_systems(Update, switch_character.after(tick_player).before(po_link));
        #[cfg(not(feature = "game"))]
        app.add_systems(Update, po_link.after(tick_player).before(crate::engine::classes::plugin::tick_classes));
        // WP-P19: script SetJtJts requests (J12 `ClassLink::jt_states`), applied at the end of the frame
        // (outside Update: its NonSend param reorders the Update executor)
        app.add_systems(bevy_app::Last, script_jt_states);
        app.add_systems(bevy_app::Update, (feed_alo_110, script_bomb_explode));
        // WP-P25: FUN_00228750 spawn activation once the class set of the new world is built
        app.init_resource::<PlayerSpawnLists>();
        #[cfg(feature = "game")]
        app.add_systems(Update, spawn_activate.after(crate::engine::classes::plugin::tick_classes).before(crate::engine::game::app::lend_classes));
        // WP-EP1a: P26 pickpocket loot (holders / pool / flights / drops) per world, after the class frame
        app.init_resource::<LootWorld>();
        #[cfg(feature = "game")]
        app.add_systems(Update, loot_link.after(crate::engine::classes::plugin::tick_classes).before(crate::engine::game::app::lend_classes));
        #[cfg(feature = "game")]
        app.add_systems(Update, po_set_vel.after(crate::engine::classes::plugin::tick_classes).before(crate::engine::game::app::lend_classes));
    }
}

/// WP-EP1a: the live P26 loot system of the current world (`key` = world identity, `t` = game clock since the load, `PlayerClock` dt).
#[derive(Resource, Default)]
pub struct LootWorld {
    pub key: usize,
    pub sys: super::loot::LootSys,
    pub t: f32,
    /// WP-K33: the DAT_003e1414 category-0x80 list for the JT's FUN_00194a40 (None = no world yet)
    pub cands: Option<Vec<super::TargetCand>>,
}

/// WP-K33: the DAT_003e1414 entry of a class 35 / 36 / 37 use point (C2d1ee8 base) for 0x194688: TARGET member
/// +0xe0 = `Sub182c70` (member vt 0x2d1ea8: +0x14 = 0x182cc8 sub 0, +0x1c = 0x184208 position = FUN_001a62e8(obj)),
/// +0x10 prio, +0x18 group, +0x1c mask; obj +0x10 (owner) != 0 -> radius test with obj +0x38.
pub fn use_point_cand(obj: usize, class: i32, b: &crate::engine::classes::k10::C2d1ee8, h: &mut dyn crate::engine::classes::ClassHost) -> Option<super::TargetCand> {
    let o = obj as u32 + 1;
    let pos = h.ee_sub_1a62e8(o);
    let radius = (h.parent_10(o) != 0).then_some((pos, b.f_38));
    Some(super::TargetCand {
        obj: o as usize,
        class,
        f_10c: b.f_10c as i32,
        radius,
        member: 0xe0,
        prio: b.sub.f_10 as i32,
        group: b.sub.f_18 as i32,
        mask: b.sub.f_1c as u32,
        sub: 0,
        pos,
        category: 0x80,
        f_e8: false,
    })
}

/// WP-EP1a: builds `LootWorld` once per class set (`LootSys::from_class_set`: holders get their loot through
/// FUN_0013cf38) and runs C2cbba0_v15 + the flight timers each frame (`LootSys::update`). Positions /
/// liveness = FUN_001a62e8 / FUN_001a61a0 of the class host. The pickpocket itself (C2cb9f8 slot 50 =
/// `LootSys::ee_sub_13d020`) needs the guard pickpocket-zone volume test (+0x364.., unported, see p13).
#[cfg(feature = "game")]
pub fn loot_link(w: Option<Res<crate::engine::world::World>>, clk: Option<Res<PlayerClock>>, mut lw: ResMut<LootWorld>, classes: Option<ResMut<crate::engine::classes::plugin::Classes>>, mut players: Query<&mut Player>, lg: Option<Res<crate::engine::game::app::LevelGame>>, opt: Option<Res<crate::engine::world::WorldOptions>>, (pwd, plvl): (Option<Res<PlayerWorldData>>, Option<Res<PlayerLevel>>)) {
    let _crawl_prof = crate::engine::classes::ProfGuard::new("loot_link");
    use crate::engine::classes::ClassHost;
    use crate::engine::classes::guard::PLAYER_HANDLE;
    let (Some(w), Some(mut c)) = (w, classes) else { return };
    let c = &mut *c;
    let host = std::cell::RefCell::new(&mut c.host);
    let pos = |h: u32| host.borrow_mut().ee_sub_1a62e8(h);
    let alive = |h: u32| host.borrow_mut().ee_sub_1a61a0(h);
    let key = &*w as *const crate::engine::world::World as usize;
    let lw = &mut *lw;
    if lw.key != key {
        // FEEL R39: FUN_0013d248's save read: the level row's path (DAT_003e1c4c[DAT_003e1110] = table links) ++
        // booty / aoidAssembly / holder index, from the store 0x3d4a60 (LevelGame::data)
        let saved = |idx: i32| -> Option<i32> {
            use crate::engine::game::save::SaveValue;
            let (lg, row) = (lg.as_ref()?, opt.as_ref()?.0.mission);
            let (vars, links) = (lg.vars.as_ref()?, &lg.table.as_ref()?.entries.get(usize::try_from(row).ok()?)?.links);
            let (b, a) = (vars.ee_sub_1d8720("booty"), vars.ee_sub_1d8720("aoidAssembly"));
            if b < 0 || a < 0 {
                return None;
            }
            let path: Vec<u16> = links.iter().copied().chain([b as u16, a as u16, idx as u16]).collect();
            match vars.ee_sub_1d7ed8(&path, &lg.data) {
                SaveValue::Int(v) => Some(v),
                _ => None,
            }
        };
        lw.sys = super::loot::LootSys::from_world_saved(&w, &c.set.k31, &pos, &saved);
        lw.key = key;
        lw.t = 0.0;
        log::info!("EP1a loot: {} holders, {} loot, {} on holders", lw.sys.guards.len(), lw.sys.loots.len(), lw.sys.guards.iter().filter(|g| g.loot.is_some()).count());
    }
    // FEEL R38: the claims of FUN_001cb250(DAT_003e1ac8 = the loot pool spawner, ..) Add the copy to the world
    for l in std::mem::take(&mut lw.sys.added) {
        let h = &mut **host.borrow_mut();
        let got = crate::engine::classes::plugin::EngineHost::loot_spawner(&w, l).map_or(0, |sp| h.ee_sub_1cb250(sp, 0));
        if got != l {
            log::warn!("FEEL R38 loot pool: claimed {l}, host pool gave {got}");
        }
    }
    lw.t += clk.map_or(PLAYER_DT, |c| c.dt.max(0.0));
    lw.sys.now = lw.t;
    let jt_h = w.objects.iter().position(|o| o.class == 53).map(|o| o as u32 + 1);
    // FEEL R80: FUN_001e1e30 case 0x16 teleport of Sly (host `k22_teleports`, until now drained by the test harness
    // only): FUN_001e4d40 locator position -> SetPosition, then (rec[5] == 0) FUN_00195a00(jt+0x2c0, 0xb, 0, 0)
    // (vt +0x2c4(jt, 1) -> FUN_0018d568 -> SetJtJts(2)). Without it the EP1 hub intro sequence 3609 (rec 1 0x16 ->
    // oid 22869, rec 2 0x24 walk to oid 22865) walked Sly 8000 units across the hub from the spawn, overriding the
    // stick (user playtest: "keeps walking in one direction"); rec_ep1b 062: the original teleports Sly to
    // (6037, -6886, 1244) at clk 9.63. SLY2_FEEL_R80TP=0 = old (not drained).
    if std::env::var("SLY2_FEEL_R80TP").map_or(true, |v| v != "0") {
        let tps = std::mem::take(&mut host.borrow_mut().k22_teleports);
        for (subj, loc, rec5) in tps {
            if subj != PLAYER_HANDLE && Some(subj) != jt_h {
                continue;
            }
            // CRAWL R113: tag 0x100 = slot-37 MoveToObject (FUN_002052e0): locator xf * -(JT +0x240 = (0,0,-75))
            let p = (loc != 0).then(|| if rec5 & 0x100 != 0 { host.borrow_mut().ee_sub_1a6f00_to_world(loc as usize - 1, [0.0, 0.0, 75.0]) } else { pos(loc) });
            // ANIM-ALL R94: FUN_002052e0 also hands the locator's rotation to vt +0xf4 (SetRotation): Sly faces the
            // locator's x axis (cap_ep2_table_seated 001: jt+0x1a8 -2.713 = seat locator; the port kept the walk-in
            // yaw and sat with his back to the table). Opt-out SLY2_R94_NO_SEATROT.
            let rot = (loc != 0 && rec5 & 0x100 != 0 && std::env::var_os("SLY2_R94_NO_SEATROT").is_none())
                .then(|| host.borrow_mut().k24_world_xf(loc))
                .flatten()
                .map(|m| ([m[0], m[1], m[2]], m[0][1].atan2(m[0][0])));
            if std::env::var_os("SLY2_K22DBG").is_some() {
                eprintln!("R94SEAT tp loc {loc} rec5 {rec5:#x} p {p:?} yaw {rot:?}");
            }
            for mut pl in &mut players {
                if let Some(p) = p {
                    pl.jt.phys.pos = p;
                }
                if let Some((m, y)) = rot {
                    pl.jt.phys.rot = m;
                    pl.jt.yaw = y;
                }
                // FEEL R81: the tail vt +0x11c (SetVelocity) / +0x124 (SetAngularVelocity) with no vector = zero:
                // rec_ep1b clk 13.43 Sly stops dead at (2264.84, -6495.90) mid-hop; without it the k22 0x2a hop
                // momentum carried Sly off that roof (R81 live run 3). SLY2_FEEL_R81HOP=0 = old.
                if std::env::var("SLY2_FEEL_R81HOP").map_or(true, |v| v != "0") {
                    pl.jt.set_velocity_pub([0.0; 3]);
                }
                if rec5 == 0 {
                    pl.jt.set_jts(2, -1, &mut ());
                    pl.jt.FUN_00195a00(0xb, 0, &mut ());
                }
            }
        }
    }
    // FEEL R81: FUN_001e1e30 case 0x2a hops of Sly (host `k22_hops`): FUN_0019f590(jt, {t, pos}) -> jtbs 0x15 hop. pos:
    // t == 0 -> the JT position (+0x240 offset taken as 0: the port pos is the JT origin); FUN_00184670(t) == 0 ->
    // locator pos (FUN_001e4d40) dropped onto the ground by FUN_00173610(100, -150); attach-extension targets
    // (is-a 0x58 / 0x1b / 0x22 / 0xc / 0x23 / 0x13 / 0xb / 0xd / 0x27, vt +0x14 point) REPLACED by the locator pos.
    // rec[5] == 1 -> JMP +0x28 (jt+0x2e8) = 1. SLY2_FEEL_R81HOP=0 = old (no hop).
    if std::env::var("SLY2_FEEL_R81HOP").map_or(true, |v| v != "0") {
        host.borrow_mut().hop_drainer = true;
        let hops = std::mem::take(&mut host.borrow_mut().k22_hops);
        let cw = pwd.as_ref().and_then(|d| d.cw.clone()).or_else(|| plvl.as_ref().map(|l| l.cw.clone()));
        for (subj, t, rec5) in hops {
            if subj != PLAYER_HANDLE && Some(subj) != jt_h {
                continue;
            }
            let ext = t != 0 && [0x58, 0x1b, 0x22, 0xc, 0x23, 0x13, 0xb, 0xd, 0x27].iter().any(|&k| host.borrow_mut().ee_sub_132b10(t, k) != 0);
            let mut p = if t != 0 { pos(t) } else { players.iter().next().map_or([0.0; 3], |p| p.jt.phys.pos) };
            if t != 0 && !ext {
                if let Some(cw) = cw.as_ref() {
                    super::Jt::FUN_00173610(cw, 100.0, -150.0, &mut p, false);
                }
            }
            // FEEL R92: a class-0xb pipe target hops onto the pipe (dest {t, ext, vt+0x14}, FUN_0019f590_pipe) instead
            // of the REPLACED locator point (EP1 hub intro: pipe 123964 at (-1731,-3855); rec_ep1b climbs it).
            // SLY2_FEEL_R92PIPE=0 = old.
            let pipe = t != 0 && host.borrow_mut().ee_sub_132b10(t, 0xb) != 0 && std::env::var("SLY2_FEEL_R92PIPE").map_or(true, |v| v != "0");
            if std::env::var_os("SLY2_K22DBG").is_some() {
                eprintln!("HOPLOG k22hop t {t} pipe {pipe} p {p:?} sly {:?}", players.iter().next().map(|p| (p.jt.phys.pos, p.jt.jts, p.jt.jtbs)));
            }
            for mut pl in &mut players {
                pl.jt.with_p5(|jt, st, w| {
                    if !pipe || super::p5_states::FUN_0019f590_pipe(jt, st, w, t as usize - 1, &mut ()).is_none() {
                        super::p5_states::FUN_0019f540(jt, st, w, p, &mut ());
                    }
                });
                if rec5 == 1 {
                    pl.jt.f_28 = true;
                }
                host.borrow_mut().player_jtbs = pl.jt.jtbs;
            }
        }
    }
    // FEEL R86: FUN_001e1e30 case 0x2b launches of Sly (host `k22_launches`): FUN_00141968(jt, pos(target), f12, f13)
    // = the bounce-pad ballistic launch (FUN_001e4d40 locator pos, not dropped). SLY2_FEEL_R86PAD=0 = old (no launch).
    let launches = std::mem::take(&mut host.borrow_mut().k22_launches);
    for (subj, t, f12, f13) in launches {
        if subj != PLAYER_HANDLE && Some(subj) != jt_h {
            continue;
        }
        let p = if t != 0 { pos(t) } else { players.iter().next().map_or([0.0; 3], |p| p.jt.phys.pos) };
        for mut pl in &mut players {
            pl.jt.FUN_00141968(p, f12, f13, &mut ());
        }
    }
    // CRAWL R50: k22 0x24 walk-to goals of this frame (FUN_001e3890 -> FUN_001448f0) and the JT's FUN_00144640
    let walks = std::mem::take(&mut host.borrow_mut().k22_walktos);
    let r1 = std::mem::take(&mut host.borrow_mut().k22_walk_r1);
    for mut pl in &mut players {
        super::walkto::k22_walk_frame(&mut pl.jt, walks.clone(), jt_h);
        pl.jt.walk_fd0.r1 = pl.jt.walk_fd0.stick.is_some() && r1.iter().any(|&s| s == crate::engine::classes::guard::PLAYER_HANDLE || Some(s) == jt_h);
    }
    // FEEL R92: k22 0x1a pipe climbs of this frame (FUN_001e3890 / FUN_001e46a0 case 0x1a)
    let climbs = std::mem::take(&mut host.borrow_mut().k22_climbs);
    for mut pl in &mut players {
        let left = super::walkto::k22_climb_frame(&mut pl.jt, climbs.clone(), jt_h);
        let mut hb = host.borrow_mut();
        hb.k22_climb_left = left;
        hb.player_jts = pl.jt.jts;
    }
    // WP-K33: the JT's 0x1911a8 kind-7 calls target vt+0x19c (slot 50)(target, jt) of this frame
    for mut pl in &mut players {
        for t in std::mem::take(&mut pl.jt.slot50) {
            let i = t.wrapping_sub(1);
            if c.set.k31.iter().any(|x| x.0 == i) {
                // class 37 C2cb9f8_v50 = FUN_0013d020 (JT = class 0x34 subtree)
                let ok = lw.sys.ee_sub_13d020(t as u32, PLAYER_HANDLE, true, &pos);
                log::info!("K33 pickpocket obj {} -> {} sly {:?} holder {:?} guard {:?} vault {} held {:?}", i, ok, pos(PLAYER_HANDLE), pos(t as u32), lw.sys.guards.iter().find(|g| g.obj == t as u32), lw.sys.vault, lw.sys.held_by(PLAYER_HANDLE));
            } else {
                // class 35 / 36 slot 50 FUN_00184130 / FUN_00184330 (Sly = class 0x35)
                let (user, class) = slot50_user(&mut c.set);
                c.set.k10_use(i, user, class, &mut **host.borrow_mut());
            }
        }
        // CRAWL R26: JT event 0x167 -> FUN_001f1e20(focus guard): the guard's holder throws its item
        for g in std::mem::take(&mut pl.jt.cane.pick_167) {
            let r = c.set.ee_sub_1f1e20(g, &mut **host.borrow_mut());
            let cls = c.set.k8_class(r).map_or(-1, |x| x as i32);
            pl.jt.FUN_00188f20_ev167_done(r, cls);
        }
    }
    {
        // FUN_001f36e0 (C2cbba0_v15's parent update) on each loot object through the class host
        let dt = c.set.g.dt;
        let mut parent = |a: &mut crate::engine::classes::actor::alo::Alo, i: usize| a.ee_sub_1f36e0(i, dt, &mut **host.borrow_mut());
        lw.sys.update_with(lw.t, &pos, &alive, &mut parent);
    }
    // WP-K33: timed-loot countdown HUD (FUN_001618f8 / FUN_00161958 of FUN_0013c560)
    for r in std::mem::take(&mut lw.sys.hud) {
        let n = host.borrow().hud_timer.0 + 1;
        host.borrow_mut().hud_timer = (n, r.unwrap_or(-1.0));
    }
    // WP-K33: DAT_003e1414 (classes 35 / 36 / 37) for the next JT frame's FUN_00194a40
    let mut cands = Vec::new();
    {
        use crate::engine::classes::{k10::K10, Inst};
        let h = &mut **host.borrow_mut();
        // members of DAT_003e1414 only (slot 22 enters, slot 21 leaves)
        let list = c.set.g.k10_lists.get(&crate::engine::classes::k10::L_3E1414).cloned().unwrap_or_default();
        for (i, k) in &c.set.k10 {
            if !list.contains(&(*i as u32 + 1)) {
                continue;
            }
            if let Inst::K10(k) = k {
                match &**k {
                    K10::C2d1ee8(b) => cands.extend(use_point_cand(*i, 35, b, h)),
                    K10::C2d1d00(d) => cands.extend(use_point_cand(*i, 36, &d.base, h)),
                    _ => {}
                }
            }
        }
        for (i, x) in c.set.k31.iter().filter(|x| list.contains(&(x.0 as u32 + 1))) {
            cands.extend(use_point_cand(*i, 37, &x.base, h));
        }
    }
    lw.cands = Some(cands);
}

pub fn spawn_player(mut cmd: Commands, lvl: Res<PlayerLevel>, hull: Option<Res<PlayerHull>>, q: Query<(), With<Player>>, wd: Res<PlayerWorldData>, mut live: Query<&mut Player>, lq: Option<ResMut<crate::engine::classes::q::live::LiveQ>>) {
    // EMU5 (USER 08:16): the safehouse is the C2d80d0 select screen (hub 17239 +0x180 == 1); Sly is not walkable
    // there (the walk-out seq 27254 places him). OK'd by coordinator; POOL1 notified in REQUESTS.
    // ANIM-ALL R93: no longer at a load into the hub select state (EP2 spawn 393 msg 5): the JT must exist so Sly
    // is drawn seated (SMA 419 on the JT) and the gang idle bodies are drawn (po_link per-JT pass); the hub's
    // focus already keeps Sly from walking. Old skip: SLY2_R93_EMU5SKIP=1.
    if let Some(mut lq) = lq.filter(|_| std::env::var_os("SLY2_R93_EMU5SKIP").is_some()) {
        use crate::engine::classes::q::q2mem::QMem;
        if let Some(a) = lq.addr_of_oid(17239) {
            if q.is_empty() && lq.mem.as_mut().is_some_and(|m| m.r32(a + 0x180) == 1) {
                return;
            }
        }
    }
    // WP-EP1c: a new level / the death reload (FUN_0022b490 -> FUN_0020e890) frees the world with its JT and
    // builds a fresh one at the spawn; hpCur / gadgets survive as save data (0x1440d0 refills hp 0 on JT init)
    // WP-K33 (port addition, verification): SLY2_SPAWN also overrides this respawn (else --table ignores it)
    let env_spawn = || std::env::var("SLY2_SPAWN").ok().and_then(|s| {
        let v: Vec<f32> = s.split(',').filter_map(|x| x.trim().parse().ok()).collect();
        (v.len() == 3).then(|| [v[0], v[1], v[2]])
    });
    if lvl.is_changed() && !lvl.is_added() {
        if let Some(p) = env_spawn().or(lvl.spawn) {
            for mut pl in &mut live {
                respawn_in_place(&mut pl.jt, p, (wd.generation != 0).then_some(&wd.lvl));
            }
        }
    }
    // WP-P13 (port addition, verification): env `SLY2_SPAWN="x,y,z"` overrides the spawn point
    let env = env_spawn();
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
pub fn load_player_world(
    w: Option<Res<crate::engine::world::World>>,
    mut d: ResMut<PlayerWorldData>,
    mut jobs: ResMut<PlayerJobs>,
    classes: Option<Res<crate::engine::classes::plugin::Classes>>,
    st: Option<NonSend<crate::engine::game::vm_state::SpliceState>>,
    mut cmd: Commands,
) {
    // FEEL R88: a water out of the world's update list (LO +0x28 bit 60 clear: rec11 012 EP3 hub water 97690 b60 0
    // b56 1, +0xec bounds node 0, the scripts' object state Removes it at level start) never runs its 0x228e50
    // contact walk -> no enter message, jt+0x108c stays 0. SLY2_FEEL_R88WOOW=0 = old (every loaded water live).
    if let Some(w) = w.as_ref().filter(|w| !w.is_changed()).filter(|_| std::env::var("SLY2_FEEL_R88WOOW").map_or(true, |v| v != "0")) {
        let host = classes.as_ref().map(|c| &c.host).filter(|h| h.world.is_some());
        let objs = st.as_ref().map(|s| &s.sched.objs);
        for wa in d.hz.waters.iter_mut() {
            let oow = host.is_some_and(|h| h.lo_out_of_world(wa.obj)) || !crate::engine::game::fields::lo_in_world(objs, w, wa.obj);
            if oow != wa.oow && std::env::var_os("SLY2_FEEL_WATERDBG").is_some() {
                eprintln!("WATEROOW water obj {} oid {} oow {} -> {oow} (sched {})", wa.obj, wa.oid, wa.oow, objs.is_some());
            }
            wa.oow = oow;
        }
    }
    // WP-EP1d: run-time SO part links (FUN_001f6ac8 / 001f6b48) rebuild Sly's collision set
    if let (Some(w), Some(c)) = (w.as_ref().filter(|w| !w.is_changed()), classes.as_ref()) {
        // FEEL R82: while the class runtime is lent to the script scheduler (game::app lend_classes .. return_classes,
        // not ordered against this system) `Classes::host` is a Default stand-in (no world, so_gen 0, no part masks):
        // the collision set flipped every few frames between the masked set and the unmasked one (CWLOG 223 <-> 179
        // colliders), Sly / the camera saw a different level each frame. SLY2_FEEL_R82LEND=0 = old.
        let lent = c.host.world.is_none() && std::env::var("SLY2_FEEL_R82LEND").map_or(true, |v| v != "0");
        if std::env::var_os("SLY2_FEEL_CWDBG").is_some() && c.host.so_gen != d.so_gen {
            eprintln!("CWDBG gen {} -> {} lent {lent} cw {}", d.so_gen, c.host.so_gen, d.cw.is_some());
        }
        if !lent && c.host.so_gen != d.so_gen && d.cw.is_some() {
            d.so_gen = c.host.so_gen;
            let mut cw = super::p10::player_collision_world_masked(w, &c.host.so_parts);
            drop_b48(&mut cw, &d.b48);
            cw.repose(w, &d.pose_held);
            if std::env::var_os("SLY2_FEEL_CWDBG").is_some() {
                let objs: Vec<usize> = c.host.so_parts.iter().filter(|x| !*x.1).map(|x| x.0 .0).collect();
                eprintln!("CWDBG rebuild gen {} colliders {} off {:?} has3143 {}", d.so_gen, cw.colliders.len(), objs, cw.colliders.iter().any(|c| c.object == 3143));
            }
            d.cw = Some(Arc::new(cw));
        }
        // FEEL R87: SO part colliders follow their posed bones (FUN_001f31e0 part world copy re-transformed)
        if !lent && !c.host.k29_bones.is_empty() {
            let fresh: std::collections::HashMap<(usize, usize), crate::engine::gfx::scene::Affine> = c.host.k29_bones.iter().filter(|(k, v)| d.pose_held.get(k) != Some(v)).map(|(k, v)| (*k, *v)).collect();
            if !fresh.is_empty() {
                d.pose_held.extend(fresh.iter().map(|(k, v)| (*k, *v)));
                if let Some(cw) = d.cw.as_mut() {
                    let ch = cw.repose_calc(w, &fresh);
                    if !ch.is_empty() {
                        Arc::make_mut(cw).repose_apply(ch);
                    }
                }
            }
        }
    }
    if let Some(w) = w.filter(|w| w.is_changed()) {
        if std::env::var_os("SLY2_FEEL_CWDBG").is_some() {
            eprintln!("CWDBG world changed gen {} -> {} lent? {}", d.so_gen, classes.as_ref().map_or(0, |c| c.host.so_gen), classes.as_ref().map_or(true, |c| c.host.world.is_none()));
        }
        d.so_gen = classes.as_ref().map_or(0, |c| c.host.so_gen);
        d.b48.clear();
        d.pose_held.clear();
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
    loot: Option<Res<LootWorld>>,
    mut lq: Option<ResMut<crate::engine::classes::q::live::LiveQ>>,
    wres: Option<Res<crate::engine::world::World>>,
) {
    let _crawl_prof = crate::engine::classes::ProfGuard::new("tick_player");
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
        // ANIM-ALL R85: the 0x18c250 ASEG requests of last frame's script SetJtJts (script_jt_states runs in Last,
        // after drive_sly) are kept for this frame's drive_sly (the clear above dropped them: the table script's
        // m734(4) -> jts 0x12 never requested +0x102c, Sly's seated SM 419 never started). Opt-out SLY2_NO_R85_JTEVCARRY.
        ev.0.extend(std::mem::take(&mut *SCRIPT_JT_EVENTS.lock().unwrap()));
        if let Some(p) = &pad_now {
            pl.jt.feed_pad(p);
        }
        // WP-K33: category lists of FUN_00194a40 (only DAT_003e1414 is fed; the other lists stay empty)
        pl.jt.cands = loot.as_ref().and_then(|l| l.cands.clone());
        // BOSS R25: DAT_003e13e4 read when the JT updates (FUN_00197f98), after the level script's writes
        // (entry 181 message 6: pick range +0x158 = -5000 once the fuse is lit)
        if let Some(m) = lq.as_mut().and_then(|l| l.mem.as_mut()) {
            // R25: the free barrels' fall first (FUN_001f8dd8 on the live image's class-88 copies)
            if let Some(w) = wres.as_deref() {
                let cw = wd.cw.clone().unwrap_or_else(|| lvl.cw.clone());
                crate::engine::classes::q::barrel::phys(m, w, &cw, dt);
            }
            crate::engine::classes::q::barrel::feed(m, &mut pl.jt);
            // FEEL R76: part bit 48 (worn barrel / mounted turret) leaves the player collision set (a worn barrel's
            // level-placed collider stayed a ghost wall); a cleared bit re-adds it through the next so_gen rebuild
            if let Some(w) = wres.as_deref() {
                let b = live_b48(m, w);
                if b != wd.b48 {
                    let cleared = wd.b48.iter().any(|x| !b.contains(x));
                    if let Some(cw) = wd.cw.as_mut() {
                        drop_b48(Arc::make_mut(cw), &b);
                    }
                    if cleared {
                        wd.so_gen = wd.so_gen.wrapping_add(0x8000_0000);
                    }
                    wd.b48 = b;
                }
            }
        }
        // WP-P8: DAT_003e0f9c for 0x161a58 / 0x1a2388, hazards after the JT update, death events -> jobs
        pl.jt.dmg_env.job = jobs.0.cur.is_some();
        // BOSS: FUN_00155228 scale = lerp(DAT_002bdc20, DAT_002bdc24, FUN_00154c90(0x2de578))
        pl.jt.dmg_env.diff = [crate::engine::jobs::SUCK_SCALE_2BDC20[0], crate::engine::jobs::SUCK_SCALE_2BDC20[1], jobs.0.ee_sub_154c90()];
        let n_ev = pl.jt.dmg.ev.len();
        // WP-P14: raw stick joy+0x10/+0x14 and camera rows joy+0x40.. for FUN_001c4768 (jts 8)
        if let Some(p) = &pad_now {
            pl.jt.p5w.stick = [p.left.x * p.left.mag, p.left.y * p.left.mag];
        }
        pl.jt.p5w.cam_rows = cm.as_ref().map(|c| [0, 1, 2].map(|i| [c.mat[i][0], c.mat[i][1], c.mat[i][2]]));
        // WP-P24: DAT_002de3a0 stand-in (binocucom look yaw) = yaw of the CM forward row
        pl.jt.cam_yaw = cm.as_ref().map(|c| crate::engine::camera::math::FUN_0011bc88([c.mat[0][0], c.mat[0][1], c.mat[0][2]]).0);
        pl.jt.cam_pos = cm.as_ref().map(|c| [c.mat[3][0], c.mat[3][1], c.mat[3][2]]);
        // FEEL R86: the live image's FUN_00141968 launches (bounce pad: k22 0x2b in the image's sequences, the EP1
        // hub runs no native C2d8c48) -> the native JT's FUN_00141968 (ballistic velocity to the target, jtbs 8).
        // FEEL R87: only the launches whose actor (a0) is Sly's image copy (the class-53 JT's oid in the live image);
        // an unresolved Sly address keeps every launch (R86). SLY2_FEEL_R86PAD=0 = old (image-only launch).
        let sly_img = lq.as_ref().zip(wres.as_deref()).and_then(|(l, w)| w.objects.iter().find(|o| o.class == 53).and_then(|o| l.addr_of_oid(o.oid)));
        for (jt_a, tgt, f12, f13) in crate::engine::classes::q::hard1::take_launches_141968(lq.as_ref().map_or(0, |l| l.qid.0)) {
            if std::env::var_os("SLY2_K22DBG").is_some() { eprintln!("R86 image launch jt {jt_a:#x} (sly {sly_img:x?}) -> native jt tgt {tgt:?} f12 {f12} f13 {f13} pos {:?}", pl.jt.phys.pos); }
            if sly_img.is_some_and(|s| s != jt_a) {
                continue;
            }
            pl.jt.FUN_00141968(tgt, f12, f13, &mut *ev);
        }
        let cw = wd.cw.clone().unwrap_or_else(|| lvl.cw.clone());
        pl.jt.update(&cw, inp, dt, &mut *ev);
        if let Some(s) = script.as_ref().filter(|s| s.log > 0 && s.frame % s.log == 0) {
            eprintln!("SLY2_PAD_LOG f{} yaw {:.3} body {:.3} pos {:?} v {:?} jts {} jtbs {} hp {} dmg {} pad {:?} scr {}", s.frame, pl.jt.yaw, super::p22::body_yaw(&pl.jt.phys), pl.jt.phys.pos, pl.jt.phys.v, pl.jt.jts, pl.jt.jtbs, pl.jt.dmg.hp, pl.jt.dmg.state, pad_now.as_ref().map(|p| (p.left.x, p.left.y, p.left.mag, p.buttons)), scripted.is_some());
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

/// PLAN1: shared plain fn — update the player JT and drain slot50 into ClassSet::k10_use.
/// Called by the Bevy `loot_link` system and by the headless `Run::step` test harness.
pub fn player_step_drain_slot50(
    jt: &mut super::Jt,
    cw: &crate::engine::physics::CollisionWorld,
    inp: super::JtInput,
    dt: f32,
    cands: Option<Vec<super::TargetCand>>,
    set: &mut crate::engine::classes::ClassSet,
    host: &mut dyn crate::engine::classes::ClassHost,
) {
    jt.cands = cands;
    jt.update(cw, inp, dt, &mut ());
    // Drain slot50 -> k10_use (same logic as loot_link's slot50 loop, class 35/36 path only)
    for t in std::mem::take(&mut jt.slot50) {
        let i = t.wrapping_sub(1);
        // Skip class-37 pickpocket targets (k31); only class 35/36 use points
        if !set.k31.iter().any(|x| x.0 == i) {
            let (user, class) = slot50_user(set);
            set.k10_use(i, user, class, host);
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
    let _crawl_prof = crate::engine::classes::ProfGuard::new("spawn_activate");
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
    let moved = super::p22::apply_po_placements(&w, &r.placed, &mut c.host);
    log::info!("P25 FUN_00228750 spawn {spawn}: warps {} reset {} placed {:?} po moved {moved:?}", r.warps, r.warp_reset.len(), r.placed);
}

/// WP-P20: the controlled character (FUN_0020e890 po selection once per world, then each frame the class set
/// reads the shadow JT's joystick `k18_feed_joy` and `po_call` answers from the shadow JT).
pub fn po_link(
    w: Option<Res<crate::engine::world::World>>,
    jobs: Res<PlayerJobs>,
    mut cur: ResMut<super::p20::PoCurrent>,
    mut q: Query<&mut Player>,
    classes: Option<ResMut<crate::engine::classes::plugin::Classes>>,
    live: Option<Res<crate::engine::classes::q::live::LiveQ>>,
    mut apuk_key: Local<usize>,
) {
    let _crawl_prof = crate::engine::classes::ProfGuard::new("po_link");
    let (Some(w), Some(mut c)) = (w, classes) else { return };
    let c = &mut *c;
    let key = &*w as *const crate::engine::world::World as usize;
    // WP-EP1f: the class set still holds the previous level (job-start load right after the hub load): its
    // k18_active / instances index the old world -> no selection until the classes are rebuilt for this one
    if !c.host.world.as_ref().is_some_and(|hw| hw.objects.len() == w.objects.len() && hw.end_offset == w.end_offset) {
        *cur = super::p20::PoCurrent::default();
        return;
    }
    // WP-EP1c: `is_changed` = a same-level reload (same World address)
    if cur.world_key != key || w.is_changed() {
        *cur = super::p20::PoCurrent::default();
        // ANIM-ALL R93: FUN_0020e890 skips the po selection (vt +0x44 / slot 78(1) / the mode-0 PO Removes) when
        // FUN_001c4840 / FUN_001c4a48 (world +0x3efc / +0x3f00) are set after FUN_00228750(spawn): e.g. EP2 spawn
        // 393's message 5 hands the focus to the safehouse hub 17239 (seated gang). Focus = another object than
        // the selection's -> no selection. Opt-out SLY2_R93_NO_FOCUSSKIP.
        let focus = live.as_ref().and_then(|l| l.mem.as_ref()).map_or(0, |m| {
            let f = m.peek32(0x003d_d490 + 0x3efc);
            if f != 0 { f } else { m.peek32(0x003d_d490 + 0x3f00) }
        });
        let sel_addr = super::p20::ee_sub_20e890_po(&w, super::p20::oid_po_current(&jobs.0)).and_then(|i| live.as_ref()?.mem.as_ref()?.obj_addr.get(i).copied());
        if focus != 0 && Some(focus) != sel_addr && std::env::var_os("SLY2_R93_NO_FOCUSSKIP").is_none() {
            cur.world_key = key;
            log::info!("P20 po: world focus {focus:#x} set after the spawn -> no po selection (FUN_0020e890)");
        } else if super::p20::apply_po(&w, super::p20::oid_po_current(&jobs.0), &mut c.set, &mut c.host, &mut cur) {
            cur.world_key = key;
            log::info!("P20 po: oidPoCurrent {} -> {:?} class {}", super::p20::oid_po_current(&jobs.0), cur.obj, cur.class);
            // POOL1 R55: FUN_0020e890 vtable +0x44 of the selected po / +0x4c of the other mode-0 POs (LO +0x28
            // bit 60, through FEEL R34's run-time Add / Remove)
            if let Some(sel) = super::p20::ee_sub_20e890_po(&w, super::p20::oid_po_current(&jobs.0)) {
                for (o, on) in super::p20::ee_sub_20e890_in_world(&w, sel, &mut c.set) {
                    crate::engine::classes::ClassHost::lo_rt(&mut c.host, o, on);
                }
            }
        }
    }
    // WIRE-BM2: slot 14 FUN_001400b8 -> FUN_00141ef8 of each Bentley / Murray: gadget slots from the save store
    // (once per world, as soon as the live store exists)
    if *apuk_key != key || w.is_changed() {
        if let Some(m) = live.as_ref().and_then(|l| l.mem.as_ref()) {
            *apuk_key = key;
            let save = |p: &str| m.save.get(p).map_or(0, |v| v[1]);
            for (i, o) in w.objects.iter().enumerate() {
                if o.class == super::p20::CLASS_BENTLEY || o.class == super::p20::CLASS_MURRAY {
                    let f = c.set.k18_active == Some(i);
                    c.set.k18_load_apuk_cur(i, f, &save, &mut c.host);
                }
            }
        }
    }
    pool_pass(&mut c.set, &mut c.host);
    // WP-EP1f: the selection is redone above even before the player exists (a job-start level load reached
    // this with no Player yet and kept the previous world's po index -> drive_po out of bounds)
    let Some(mut pl) = q.iter_mut().next() else { return };
    // WP-P22: mid-mission switches (slot 78 through the class set, e.g. K22 sequence action 7)
    if std::mem::take(&mut c.host.po_release) {
        super::p22::set_active_character(&w, &mut c.set, &mut c.host, None);
    }
    // WP-P23: slot-78 callers of the last class frame (FUN_002206c8 / FUN_001c8730 / Bentley / Murray states)
    let q = std::mem::take(&mut c.host.p23_switch);
    let sw = if cur.world_key == key { super::p23::apply_switches(&w, &mut c.set, &mut c.host, q) } else { vec![] };
    // WIRE-MG: the turret focus' +0x150 tracker reads the pad
    c.host.po_focus = c.set.mg_focus.map(|o| o as u32 + 1);
    // WIRE-P: the old focus' +0x27c(0) -> FUN_001c3ed8(jt, 0) -> FUN_001c4640(jt+0x150, 0); +0x27c(1) back -> 1
    // CRAWL R103: + the safehouse hub holding the world focus (select state 1: seated, the pad drives the hub /
    // ThiefNet, not Sly; rec_ep1b 064 CONTROL -> 17239)
    pl.jt.joy_off = c.set.mg_focus.is_some() || live.as_ref().and_then(|l| l.mem.as_ref()).is_some_and(|m| m.hub_has_focus());
    // CRAWL R104: FUN_0014d478 hands the focus object the null pad DAT_002e0d00 instead of DAT_002dfc00 while
    // (DAT_002dfbc0 & 1) == 0, i.e. under any DAT_002dfbc4 lock (FUN_001870d0: 2 world pause FUN_0020efc0 = hub
    // intro / ThiefNet, 4/8 cinematics, without 0x10/1). rec_ep1b 059..062: Sly stands at spawn 393 through
    // the hub intro despite X / stick presses. One DAT_002dfbc4: live image bits + the class code's bits 2/4.
    if std::env::var_os("SLY2_NO_R104_PADMODE").is_none() {
        let f = live.as_ref().and_then(|l| l.mem.as_ref()).map_or(0, |m| m.peek32(0x002d_fbc4)) | (c.host.k27_pad_lock & 6);
        let mode = if f & 0x10 != 0 { 5 } else if f & 0xc != 0 { 0 } else if f & 1 != 0 { 5 } else if f & 2 != 0 { 4 } else { 7 };
        pl.jt.joy_off |= mode & 1 == 0;
    }
    // WIRE-EP6: script getter field 759 (FUN_00144068) of the JT
    c.host.jt_busy = pl.jt.FUN_00144068();
    if !sw.is_empty() {
        log::info!("P23 slot-78 switch -> {:?}", c.set.k18_active);
    }
    if cur.world_key == key && super::p22::follow_active_bodies(&w, c.set.k18_active, &mut cur, &mut pl.jt, &mut c.host) {
        log::info!("P22 po switch -> {:?} class {}", cur.obj, cur.class);
    }
    if cur.world_key == key {
        super::p22::team_link(&w, &mut c.set, &mut c.host, &cur, &mut pl.jt);
        // ANIM-ALL R34: FUN_0019cd38 mode-1 slot 82 / 83 calls (the JT = the po, else Sly)
        let me = cur.obj.or_else(|| w.objects.iter().position(|o| o.class == super::p20::CLASS_SLY));
        // ANIM-ALL R35: sequence action 0x3b / FUN_001e54b0 binocucom calls on the JT (anim hook: none here)
        let ops = std::mem::take(&mut c.set.jt_binoc_ops);
        for op in ops {
            use crate::engine::classes::k22_seq::GuardOp;
            match op {
                GuardOp::Binoc(o, m, a, b) if me.is_some_and(|x| x as u32 + 1 == o) => pl.jt.FUN_0019cc50(m, a, b, &mut ()),
                GuardOp::BinocLeave(o) if me.is_some_and(|x| x as u32 + 1 == o) => pl.jt.FUN_0019cce8(&mut ()),
                _ => {}
            }
        }
        super::p15_binoc::binoc_slots(&mut c.set, &mut c.host, me, &mut pl.jt);
        c.set.jt_binoc_hud = me.map(|o| (o as u32 + 1, pl.jt.binoc.hud));
    }
    match cur.obj {
        Some(o) if c.set.k18_active == Some(o) => {
            // WP-P23: the po's live ASEGAs for playerdraw::po
            cur.plays = super::p23::po_plays(&c.host.k25, o);
            // TAKEN BY HEIST: the JOY press stamps are on the JT clock; the class side tests them against the class
            // clock (one DAT_002dded0 on PS2): rebase them
            let mut joy = pl.jt.po_joy.take().unwrap_or(pl.jt.joy);
            // MURRAY: onto the clock of the next class frame (t += dt runs before it), which first sees them
            joy.rebase_presses(c.set.g.t + c.set.g.dt - pl.jt.t);
            c.set.k18_feed_joy(joy);
            // WIRE-SLY: the press stamps now belong to the class JOY (its FUN_001c4738 consumes them); clear
            // them on the shadow JT so a press is seen (and paid for) once, not re-fed every frame
            pl.jt.joy.press_t = [crate::engine::player::PRESS_NONE; 16];
            // WP-P22: the class PO fields of last frame's slot 15 / 69 drive the body (p22::ee_sub_204108)
            let bb = std::mem::take(&mut c.host.po_bb58);
            cur.bb58 = bb.iter().rev().find(|r| r.0 == o).map(|r| (r.1, r.2));
            cur.plain = super::p23::po_plain(&mut c.set, o, cur.class);
            pl.jt.po_move = super::p22::po_move_of_svt(&mut c.set, o, cur.class, &cur.loco, &bb, &|a| c.host.k25.svt(a));
            // FEEL R69: the class's direct SetVelocity (Murray anim event 0x15e, FUN_00140f90) on the body (late
            // fallback; normally taken by po_set_vel right after the class frame)
            if let Some(v) = c.set.k18_murray(o).and_then(|m| m.set_vel_11c.take()) {
                pl.jt.phys.v = v;
            }
            po_hull_link(&w, &mut c.set, o, cur.class, &mut pl.jt);
            po_drain_slot50(&mut c.set, &mut c.host);
            c.host.po = Some(super::p20::PoHost { obj: o, jt: Box::new(pl.jt.clone()) });
        }
        _ => {
            c.host.po = None;
            c.host.po_bb58.clear();
            cur.bb58 = None;
            cur.plain = None;
            pl.jt.po_move = None;
        }
    }
    // CRAWL R111: uncontrolled in-world Bentley / Murray bodies for playerdraw::drive_idle_po (opt-out SLY2_NO_R111_GANGDRAW)
    cur.idle.clear();
    if cur.world_key == key && std::env::var_os("SLY2_NO_R111_GANGDRAW").is_none() {
        use crate::engine::classes::ClassHost;
        let mut ids: Vec<usize> = c.host.k27_po.keys().copied().filter(|&o| Some(o) != cur.obj && matches!(w.objects.get(o).map(|x| x.class), Some(super::p20::CLASS_BENTLEY | super::p20::CLASS_MURRAY))).collect();
        ids.sort_unstable();
        for o in ids {
            if c.host.lo_removed(o) {
                continue;
            }
            let Some(b) = c.host.k27_po.get(&o).map(|b| (b.phys.pos, b.phys.rot)) else { continue };
            let plays = super::p23::po_plays(&c.host.k25, o);
            let plain = super::p23::po_plain(&mut c.set, o, w.objects[o].class);
            if std::env::var_os("SLY2_R83IDLE").is_some() {
                let pa: Vec<(i32, f32)> = plays.iter().map(|&(a, t)| (w.objects[a].oid, t)).collect();
                let all: Vec<(i32, f32)> = c.host.k25.asegas.values().filter(|x| x.st.alive && x.obj as usize == o + 1).map(|x| (w.objects[x.aseg].oid, x.st.t)).collect();
                let st = match w.objects[o].class { super::p20::CLASS_BENTLEY => c.set.k16_get(o).map(|b| (b.state, b.m.u(0x102c), b.m.u(0x1034), b.m.u(0x1030))), _ => c.set.k18_murray(o).map(|b| (b.b.state, b.b.m.u(0x102c), b.b.m.u(0x1034), b.b.m.u(0x1030))) };
                if c.set.g.t < 0.05 || std::env::var_os("SLY2_R83SMDUMP").is_some() {
                    if let Some(x) = st { if let Some((_, m)) = c.host.k25.sm(&w, x.1) {
                        eprintln!("R83SM states {:?} groups {:?}", m.states, m.groups.iter().map(|g| g.0).collect::<Vec<_>>());
                        for t in &m.trans { eprintln!("R83SM tr {:?} {} -> {} mask {:#x} f10 {} w {}", t.aseg.map(|a| w.objects[a].oid), t.from, t.to, t.mask, t.f10, t.weight); }
                    } }
                }
                let smd = st.and_then(|x| c.host.k25.smas.get(&x.2)).map(|x| (w.objects[x.sm].oid, x.alive, x.sma.cur, x.sma.next, x.sma.goal, x.sma.trans, x.asega, x.sma.asega.is_some(), x.sma.mask, x.sma.flags));
                eprintln!("R83IDLE obj {o} oid {} cls {} st {st:?} sma {smd:?} plays {pa:?} all {all:?} plain {plain:?}", w.objects[o].oid, w.objects[o].class);
            }
            cur.idle.push(super::p20::IdlePo { obj: o, pos: b.0, rot: b.1, plays, plain });
        }
    }
}

/// FEEL R70: FUN_00140f90's SetVelocity (vt +0x11c, Murray anim event 0x15e) lands on the body in the same
/// frame as the event (the anim tick runs after the PO's physics on PS2: capture velocity set on frame N,
/// position moves from N+1). po_link (before the class frame) took it one frame later. SLY2_FEEL_SETVEL_LATE=1 = old.
pub fn po_set_vel(cur: Res<super::p20::PoCurrent>, mut q: Query<&mut Player>, classes: Option<ResMut<crate::engine::classes::plugin::Classes>>) {
    let (Some(o), Some(mut c)) = (cur.obj, classes) else { return };
    if std::env::var_os("SLY2_FEEL_SETVEL_LATE").is_some() || c.set.k18_active != Some(o) {
        return;
    }
    let Some(mut pl) = q.iter_mut().next() else { return };
    let Some(m) = c.set.k18_murray(o) else { return };
    if let Some(v) = m.set_vel_11c.take() {
        pl.jt.phys.v = v;
        // FUN_00140f90 wrote +0x190 = -root vel with it: the next physics step's motor (FUN_00204108) reads the
        // fresh +0x190 (po_move was built before this class frame: stale 1200 -> a 57 ramp after the lunge end)
        if let Some(pm) = pl.jt.po_move.as_mut() {
            pm.v190 = m.b.m.v3(0x190);
        }
    }
}

/// FEEL R57: the controlled Bentley / Murray hull (FUN_00203a38 when prop 226 set +0x264; built once per object)
/// and this frame's morph target (Bentley FUN_00135d80 / Murray FUN_001b8008 from the class state).
pub fn po_hull_link(w: &crate::engine::world::World, set: &mut crate::engine::classes::ClassSet, o: usize, class: u16, jt: &mut super::Jt) {
    use super::p5_hull as h;
    if jt.po_hull.as_ref().map(|x| x.obj) != Some(o) {
        let pc = matches!(class, super::p20::CLASS_BENTLEY | super::p20::CLASS_MURRAY);
        jt.po_hull = crate::engine::gfx::attach::alo_of(w, o).filter(|_| pc).and_then(|alo| h::po_hull_from_parts(o, alo, class as i32, h::prop_hull_id(&alo.props)?));
    }
    let Some(ph) = jt.po_hull.as_ref() else { return };
    // FEEL R58/R59: the sub-object lookup inputs: obj+0x2dc (JMP +0x1c), +0x4c8 / +0x4cc (attach +8 / +0xc), +0xdf4
    // (damage +4). The port keeps them as C2cb230 fields (f_2dc / att_4c8 / att_4cc / dmg.state), -1 from the base init
    // FUN_0013ff70 -> FUN_00195950 / FUN_00191090 / FUN_001a0b18 as the captures show; the raw memory there is unused (0)
    let sub = |b: &crate::engine::classes::k16_55::C2cb230| h::PoSub { s2dc: b.f_2dc, att_kind: b.att_4c8, att_sub: b.att_4cc, dmg: b.dmg.state };
    jt.po_hull_tgt = match class {
        super::p20::CLASS_MURRAY => set.k18_murray(o).map_or((ph.hull.id, false), |b| h::FUN_001b8008(ph, b.b.state, b.b.m.i(0x1458), b.b.m.i(0x13f0), sub(&b.b))),
        _ => set.k16_get(o).map_or((ph.hull.id, false), |b| h::FUN_00135d80(ph, b.state, sub(b))),
    };
}

/// POOL1: the pool-copy pass of po_link (Bentley dart / bomb copies, FUN_001cb250 / FUN_001357c0 / vt+0x124 /
/// vt+0x264 / FUN_001cb488 attachment / REPLACED ground drop), shared with the headless Run harness
pub fn pool_pass(set: &mut crate::engine::classes::ClassSet, host: &mut crate::engine::classes::plugin::EngineHost) {
    // POOL1: pool copies (EngineHost FUN_001cb250) are K15 class 101 / K23 102 instances: the thrower's
    // field writes (+0x15c kind, +0x16c linked object) reach the instance, its +0x150 / +0x158 state is
    // published back for FUN_00224590 / the carry reads (REPLACED: one guest object shared by both)
    // POOL1: FUN_001357c0(o, tgt): +0x16d8 picks the template spawner +0x16dc / +0x16e0, FUN_001cb250 claims a
    // copy, the copy's +0xe4 +0x18 = tgt, launched from the class-112 emitter +0x16e4 (FUN_001d6b00 = k15_fire,
    // +0x60 = 0 / +0x50 = +0x16f0 = k16_fire target), FUN_0013b5c8(copy, 4.0) fuse,
    // FUN_001362f8(o, 1), +0x1700 = now
    for (o, tgt) in std::mem::take(&mut host.pool_fire) {
        let Some(m) = set.k16_get(o).map(|b| [b.m.u(0x16d8), b.m.u(0x16dc), b.m.u(0x16e0), b.m.u(0x16e4)]) else { continue };
        let t = match m[0] { 0 => m[1], 1 => m[2], _ => continue };
        if t == 0 { continue; }
        let s0 = crate::engine::classes::ClassHost::ee_sub_1cb250(host, t, 0);
        if s0 == 0 { continue; }
        // *(copy +0xe4 + 0x18) = tgt: collision part 0's hit kind (0x80000 = sleep)
        host.part0_kind.insert(s0 as usize - 1, tgt as i32);
        if m[3] != 0 {
            host.ride_root.insert(m[3], o as u32 + 1);
            let tp = set.k16_get(o).map(|b| [b.m.f(0x16f0), b.m.f(0x16f4), b.m.f(0x16f8)]);
            let r = set.k16_fire(m[3] as usize - 1, s0 as usize - 1, tp, host);
            log::info!("POOL1 dart fire: copy {s0:#x} from emitter {:#x}: {}", m[3], r.is_some());
        }
        let now = set.g.t;
        // the emitter rides Bentley (owner bone): its loaded offset from Bentley's loaded placement, moved with
        // the live body (REPLACED: bone 4098 anim / yaw, as pool_attach_pos)
        let (ep, bp) = (host.k24_world_xf(m[3]).map(|x| x[3]), host.k24_world_xf(o as u32 + 1).map(|x| x[3]));
        let cp = host.player_pos;
        if let Some(p) = set.k13_get(s0 as usize - 1) {
            if let (Some(e), Some(b)) = (ep, bp) {
                let d = [0, 1, 2].map(|k| cp[k] - b[k]);
                for k in 0..3 { p.p.phys.pos[k] += d[k]; }
                let _ = e;
            }
            p.p.ee_sub_13b5c8(4.0, now);
        }
        // FUN_001362f8(o, 1)
        let g = std::mem::take(&mut set.g);
        if let Some(b) = set.k16_get(o) { b.ee_sub_1362f8(o, 1, &g, host); }
        set.g = g;
        if let Some(b) = set.k16_get(o) { b.m.set_f(0x1700, now); }
    }
    // POOL1: FUN_002205a8 release velocity (vt +0x124) before the state change
    for (h, v) in std::mem::take(&mut host.pool_v) {
        if let Some(b) = set.k15_101_get(h as usize - 1) { b.phys.v = v; }
    }
    // POOL1 r32: through the K24 host (the guards: FUN_00225660 state 5's FUN_0015e9c8 grab query)
    let q264 = std::mem::take(&mut host.pool_264);
        set.with_k24(host, |set, kh| {
        for (h, st) in q264 {
            let o = h as usize - 1;
            set.k15_101_with(o, kh, |b, cx| b.slot75(o, st, cx));
        }
    });
    // R53: a faded-out copy (FUN_001f7bf8 at state 4) leaves the world (vt +0x4c): out of the live set, so
    // FUN_001cade0 can hand the spawner's copy out again (before, Bentley got no 2nd bomb after the 1st blast)
    let dt = set.g.dt;
    let gone: Vec<u32> = host.pool_fade.iter_mut().filter_map(|(&h, t)| { *t -= dt; (*t <= 0.0).then_some(h) }).collect();
    // HEIST r31: the faded-out end is slot 15 C2caf88_v15 (alpha <= 0) -> vt +0x4c FUN_001a5630 (Remove: the
    // subtree's in-world bit, `lo_rt`), e.g. the KO'd brawl enemy's KO clip fading itself out (guard/hit.rs)
    for &h in &gone {
        crate::engine::classes::ClassHost::vcall(host, h, 0x4c);
    }
    let gone: Vec<u32> = { let mut g = gone; for h in std::mem::take(&mut host.pool_off) { if !g.contains(&h) { g.push(h); } } g };
    // R53: FUN_001a5630 (vt +0x4c) on an object in the world (+0x28 bit 60) -> vt +0x64 = slot 11
    // FUN_00224068 (slot 75(-1), the SMA freed); FUN_001a5478 (vt +0x44) -> vt +0x5c = slot 10 FUN_00224028
    // (a new SMA on the copy's SM). REPLACED: post-load already ran slot 10 on every copy, so an add with an
    // SMA still held keeps it.
    let added = std::mem::take(&mut host.pool_add);
    // POOL1 R56: the bit as it was at the vt +0x4c (`pool_off_in`): the in-world sync may have cleared it since
    let was_in = std::mem::take(&mut host.pool_off_in);
    let off: Vec<u32> = gone.iter().copied().filter(|&h| was_in.contains(&h) || crate::engine::classes::ClassHost::obj_i32(host, h, 0x2c) as u32 & 1 << 28 != 0).collect();
    set.with_k24(host, |set, kh| {
        for &h in &off {
            let o = h as usize - 1;
            set.k15_101_with(o, kh, |b, cx| b.ee_sub_224068(o, cx));
        }
        for &h in &added {
            let o = h as usize - 1;
            set.k15_101_with(o, kh, |b, cx| if b.f_168 == 0 { b.ee_sub_224028(o, cx.h) });
        }
    });
    for h in gone {
        host.pool_fade.remove(&h);
        host.pool_live.remove(&h);
        host.pool_attach.remove(&h);
        host.pool_att_prev.remove(&h);
        let hi = crate::engine::classes::ClassHost::obj_i32(host, h, 0x2c) as u32 & !(1 << 28);
        crate::engine::classes::ClassHost::obj_set_u32(host, h, 0x2c, hi);
    }
    let live: Vec<u32> = host.pool_live.iter().copied().collect();
    for h in live {
        let (k, l) = (host.k25_fld.get(h, 0x15c), host.k25_fld.get(h, 0x16c));
        let carrier = set.k18_active.map_or(0, |o| o as u32 + 1);
        let cp = host.player_pos;
        let placing = host.pool_place.contains(&h);
        let att = host.pool_attach_pos(h, carrier, cp);
        let dt = set.g.dt;
        if let Some(b) = set.k15_101_get(h as usize - 1) {
            // POOL1: attached (FUN_001cb488) -> the copy rides the spawner; its velocity is the hand motion, kept
            // when the state-6 exit releases it (vt+0x264(1), FUN_001cb4b8)
            // R51: the velocity is the attach point's own motion since the last frame. Before, it was taken from the
            // copy's integrated position (pos + v*dt), which fed back (v -> -v, |v| ~3.6e5 from the pooled state)
            // and at release dropped the copy ~6000 below the hand -> > +0x170 from Bentley -> slot 74 state 3 ->
            // fuse ASEG msg 2 -> explode at throw time.
            if let Some(p) = att {
                // the FUN_001caec8 placement frame is a teleport, not hand motion
                let prev = host.pool_att_prev.insert(h, p);
                b.phys.v = match prev {
                    Some(q) if dt > 0.0 && !placing => [0, 1, 2].map(|k| (p[k] - q[k]) / dt),
                    _ => [0.0; 3],
                };
                b.phys.pos = p;
            } else {
                host.pool_att_prev.remove(&h);
            }
            if let Some(k) = k { b.f_15c = k as i32; }
            if let Some(l) = l { b.f_16c = l; }
            // POOL1: REPLACED SO world collision of a flying copy: the FUN_00173610 ground drop under it; below
            // the ground -> rest on it (velocity 0) and a slot-57 contact (world, up normal) for its next frame
            if (1..=3).contains(&b.f_150) { // POOL1 r42: state 3 (trigger bomb thrown) flies too
                let p = b.phys.pos;
                let g = crate::engine::classes::ClassHost::k21_ground_173610(host, 100.0, -350.0 + (b.phys.v[2] * dt).min(0.0), p);
                if let Some(g) = g.filter(|g| p[2] <= g[2] + 1.0) {
                    b.phys.pos[2] = g[2];
                    b.phys.v = [0.0; 3];
                    host.pool_contacts.entry(h as usize - 1).or_default().push(crate::engine::classes::k15_101::Contact57 { other: 0, kind: 0, normal: [0.0, 0.0, 1.0], second: false });
                }
            }
            let (s, p) = (b.f_150, b.f_158);
            host.k25_fld.set(h, 0x150, s as u32);
            host.k25_fld.set(h, 0x158, p as u32);
        }
    }
}

/// WIRE-B2: the next character of the switch cycle Sly (None) -> Bentley -> Murray -> Sly over the class-55 / 54
/// instances that exist in `set` (first of each in object order); `active` = `ClassSet::k18_active`.
pub fn next_character(w: &crate::engine::world::World, set: &mut crate::engine::classes::ClassSet, active: Option<usize>) -> Option<usize> {
    use super::p20::{CLASS_BENTLEY, CLASS_MURRAY};
    let mut cycle = vec![None];
    for class in [CLASS_BENTLEY, CLASS_MURRAY] {
        let o = w.objects.iter().enumerate().position(|(i, x)| {
            x.class == class && if class == CLASS_BENTLEY { set.k16_get(i).is_some() } else { set.k18_murray(i).is_some() }
        });
        cycle.extend(o.map(Some));
    }
    let i = cycle.iter().position(|c| *c == active).unwrap_or(0);
    cycle[(i + 1) % cycle.len()]
}

/// WIRE-B2 (port addition: the PS2 game switches only through the safehouse / jobs / scripts): the rebindable
/// `switch_character` action (input::bindings, KeyV / gamepad Mode by default) cycles the controlled character
/// through the ported slot-78 path (`p22::set_active_character` -> `ClassSet::k18_set_active` -> FUN_001406e8 /
/// FUN_001c3ed8 / FUN_001c4940); `po_link` then moves the body (`p22::follow_active`) and feeds the joystick, and
/// the class frame runs the chosen Bentley / Murray (slot 15 / 69, `frame_k18_55` / `frame_k18_54`).
#[cfg(feature = "game")]
pub fn switch_character(
    pad: Option<Res<crate::engine::input::backend::PadInput>>,
    w: Option<Res<crate::engine::world::World>>,
    cur: Res<super::p20::PoCurrent>,
    classes: Option<ResMut<crate::engine::classes::plugin::Classes>>,
    mut prev: Local<bool>,
) {
    let held = pad.is_some_and(|p| p.extra & crate::engine::input::bindings::EXTRA_SWITCH_CHARACTER != 0);
    let edge = held && !*prev;
    *prev = held;
    let (true, Some(w), Some(mut c)) = (edge, w, classes) else { return };
    // only once po_link has made its selection for this world (same class set / world check)
    if cur.world_key != &*w as *const crate::engine::world::World as usize {
        return;
    }
    let c = &mut *c;
    let active = c.set.k18_active;
    let next = next_character(&w, &mut c.set, active);
    super::p22::set_active_character(&w, &mut c.set, &mut c.host, next);
    log::info!("WIRE-B2 switch_character -> {:?}", c.set.k18_active);
}

/// WP-P11: Sly's cane hits on the class set's guards (`p11_cane::cane_hits_to_guards`; no-op without classes).
pub fn cane_hits(mut q: Query<(&mut Player, &mut JtAnimEvents)>, classes: Option<ResMut<crate::engine::classes::plugin::Classes>>) {
    let _crawl_prof = crate::engine::classes::ProfGuard::new("cane_hits");
    let Some(mut c) = classes else { return };
    let c = &mut *c;
    for (mut p, mut ev) in &mut q {
        super::p11_cane::cane_hits_to_guards(&mut p.jt, &mut c.set, &mut c.host, &mut *ev);
    }
}

/// WP-P13: JT focus selection FUN_00140558 over the class set's guards (`p13`; no-op without classes).
pub fn guard_focus(mut q: Query<&mut Player>, classes: Option<Res<crate::engine::classes::plugin::Classes>>) {
    let _crawl_prof = crate::engine::classes::ProfGuard::new("guard_focus");
    let Some(c) = classes else { return };
    let cands = guard_focus_cands(&c.set);
    for mut p in &mut q {
        p.jt.FUN_00140558(&cands);
    }
}

/// WP-P13: the FUN_00140558 focus candidates (the class set's guards). CRAWL R78: also used by the headless
/// test Run for the po's shadow JT (Murray / Bentley base update FUN_001403d0 calls FUN_00140558).
pub fn guard_focus_cands(set: &crate::engine::classes::ClassSet) -> Vec<super::p13::FocusCand> {
    use crate::engine::classes::Inst;
    set.k6
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
        .collect()
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
    w: Option<Res<crate::engine::world::World>>,
    lq: Option<ResMut<crate::engine::classes::q::live::LiveQ>>,
) {
    let _crawl_prof = crate::engine::classes::ProfGuard::new("classes_link");
    let Some((mut p, mut ev)) = q.iter_mut().next() else {
        // ANIM-ALL R93: no JT (spawn_player skips Sly while the safehouse hub holds the select state, e.g. the EP2
        // load: spawn 393 msg 5 -> hub state 1): the live SMA script messages (hub select SMA msg 2 -> the seat
        // script, table camera) are still the level script's; they were never drained. Opt-out SLY2_R93_NO_NOJTMSG.
        if std::env::var_os("SLY2_R93_NO_NOJTMSG").is_none() {
            if let (Some(w), Some(m), Some(c)) = (w, lq.and_then(|l| l.into_inner().mem.as_mut()), classes) {
                crate::engine::classes::q::barrel::script_msgs(m, &w, &mut c.into_inner().host);
            }
        }
        return;
    };
    let mut classes = classes;
    if let Some(c) = classes.as_mut() {
        let c = &mut **c;
        link_classes(&mut p.jt, &mut c.set, &mut c.host, &mut *ev, &mut hud);
    }
    // BOSS R22: the JT -> TNT barrel glue (classes::q::barrel::jt_frame: FUN_00192bd8 kind-1 FUN_00183b80 calls,
    // FUN_001a19a0 jt+0x4c8 == 1 delegation to the barrel's slot 74, DAT_003e13e4 feed) into the live image,
    // after this frame's attach (tick_player) and damage (link_classes) events. (Kept inside classes_link: a
    // separate system reordered the ambiguous Update systems and broke wired_guards_spawn_and_patrol_live.)
    if let (Some(w), Some(m)) = (w, lq.and_then(|l| l.into_inner().mem.as_mut())) {
        crate::engine::classes::q::barrel::jt_frame(m, &w, &mut p.jt, &mut *ev);
        // BOSS R25: the barrels' script messages (explosion 0xb) -> the level script
        if let Some(c) = classes.as_mut() {
            crate::engine::classes::q::barrel::script_msgs(m, &w, &mut c.host);
        }
    }
    hud.hp = p.jt.dmg.hp;
    hud.hp_max = p.jt.dmg.hp_max;
    hud.gadgets = p.jt.gadget_hud();
}

/// POOL1: method 1110 (thunk 0x0010d9e0) requests of the scripts -> FUN_00224e70(bomb, x) on the class-101
/// instance, with the guard-routing host (classes::k24) so FUN_00224bf8's hits reach the guards
pub fn script_bomb_explode(
    st: Option<NonSendMut<crate::engine::game::vm_state::SpliceState>>,
    c: Option<ResMut<crate::engine::classes::plugin::Classes>>,
) {
    let (Some(mut st), Some(mut c)) = (st, c) else { return };
    if st.sched.cls.explode_224e70.is_empty() {
        return;
    }
    // while the class set is lent to the scripts (game::app lend_classes) the bombs are not in `Classes`:
    // keep those requests for the next frame
    let c = &mut *c;
    let q = std::mem::take(&mut st.bypass_change_detection().sched.cls.explode_224e70);
    st.bypass_change_detection().sched.cls.explode_224e70 = bomb_explode_pass(&mut c.set, &mut c.host, q);
}

/// POOL1 r42: script_bomb_explode's body (also run by the test harness): the requests whose bomb is not in the
/// class set are returned (kept for the next frame)
pub fn bomb_explode_pass(set: &mut crate::engine::classes::ClassSet, host: &mut crate::engine::classes::plugin::EngineHost, q: Vec<(u32, u32)>) -> Vec<(u32, u32)> {
    let mut keep = Vec::new();
    for (o, x) in q {
        let i = o as usize;
        if set.k15_101_get(i).is_none() {
            keep.push((o, x));
            continue;
        }
        set.with_k24(host, |s, h| {
            s.k15_101_with(i, h, |b, cx| b.ee_sub_224e70(i, x, cx.h));
        });
        // POOL1 R51: no slot release here. Live capture B3 (original): Bentley's placed-bomb slot +0x186C is
        // written only on place (pc 0x135B4C, FUN_00135ad8 anim event, bomb from FUN_001cb250) and never
        // cleared on explode / death / respawn; the next place overwrites it.
    }
    keep
}

/// POOL1: the script's ALO +0x110 bytes (FUN_001f7918 via method 0x00100fe0) onto the class host, read by
/// FUN_001f78d8 when FUN_001f7598 delivers a hit to a guard (classes/k24.rs `r_ee_sub_1f7598`)
pub fn feed_alo_110(
    st: Option<NonSend<crate::engine::game::vm_state::SpliceState>>,
    c: Option<ResMut<crate::engine::classes::plugin::Classes>>,
) {
    let (Some(st), Some(mut c)) = (st, c) else { return };
    if c.host.alo_110 != st.sched.objs.alo_110 {
        c.host.alo_110 = st.sched.objs.alo_110.clone();
    }
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
    let n0 = ev.0.len();
    drain_jt_states(&mut p.jt, states, jt_obj, &mut *ev);
    if std::env::var_os("SLY2_NO_R85_JTEVCARRY").is_none() {
        let new = ev.0.split_off(n0);
        SCRIPT_JT_EVENTS.lock().unwrap().extend(new);
    }
}

/// ANIM-ALL R85: 0x18c250 ASEG requests of the script SetJtJts calls (script_jt_states), for the next frame's
/// JtAnimEvents (see the player update)
pub static SCRIPT_JT_EVENTS: std::sync::Mutex<Vec<(u32, i32, i32)>> = std::sync::Mutex::new(Vec::new());

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
        jt.FUN_00144268(hp, mp);
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
        let mut h7 = hit.to_p7(pos, fbc);
        // HEIST r15: projectile sources (spotlight bullets): their own is-a / vt+0xfc direction
        if let (Some(s), Some((isa, dir))) = (h7.src.as_mut(), set.proj_hit_src(hit.src)) {
            s.isa = isa;
            s.dir_fc = dir;
        }
        if super::p7_damage::FUN_001a19a0(jt, &h7, &env, anim) {
            hud.hits_taken += 1;
        } else {
            hud.hits_refused += 1;
        }
    }
    // player record +0x15c / position for the pickups and sensors
    let pos = jt.phys.pos;
    host.player_rec = Some(crate::engine::classes::k8::PickRec { alive: (jt.dmg.state == -1) as i32, radius: JT_RADIUS, obj: PLAYER_HANDLE });
    host.player_pos = pos;
    host.jt_live = true;
    host.player_jtbs = jt.jtbs;
    host.player_jts = jt.jts;
    host.player_hull = jt.hull.clone().map(|h| (h, jt.phys.rot));
    // WP-SCR2: FUN_00143ee0 pose requests on Sly, enqueued by `EngineHost::k22_pose_143ee0` (k22 action 0xa
    // class-0x34 branch); FUN_00144038(Sly) back to the host for the k22 wait condition.
    for req in std::mem::take(&mut host.player_pose) {
        jt.ee_sub_143ee0(&req, host);
    }
    host.player_posing = jt.posing_144038();
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
    n.gad.slots.iter_mut().for_each(|s| s.state = 0);
    n.gad.time_scale = 1.0;
    n.dmg.hp = jt.dmg.hp;
    n.FUN_001440d0(0);
    n.dmg_env = jt.dmg_env;
    n.is_player = jt.is_player;
    if let Some(l) = lvl {
        l.apply(&mut n);
    }
    *jt = n;
}

pub fn publish_focus(q: Query<&Player>, mut f: ResMut<PlayerFocus>) {
    let _crawl_prof = crate::engine::classes::ProfGuard::new("publish_focus");
    *f = match q.iter().next() {
        Some(p) => PlayerFocus { valid: true, pos: p.jt.phys.pos, v: p.jt.phys.v, jts: p.jt.jts, cam_glide: p.jt.cam_glide, binoc_cam: (p.jt.binoc.state == 1).then_some(p.jt.binoc.mode), tgt: p.jt.guard_tgt(), yaw: p.jt.yaw, z_contact: p.jt.z_contact, ground: p.jt.ground_nz.is_some(), z_cpdefi: p.jt.phys.pos[2], t: 0.0, attach: (p.jt.attach.kind, p.jt.attach.sub), jtbs: p.jt.jtbs, follow_off: p.jt.cpdefi_follow_off() },
        None => PlayerFocus::default(),
    };
}

/// Follow-camera world view of the player (pso ignored: the player is the only JT).
pub struct PlayerFollowWorld(pub PlayerFocus);

impl CmWorld for PlayerFollowWorld {
    fn get_cpdefi(&mut self, _pso: u32) -> Cpdefi {
        let mut c = Cpdefi::default();
        let (p, v) = (self.0.pos, self.0.v);
        // FEEL R25: 0x0018a428 -> 0x00141640 -> 0x001f58f0 (SO base): pos_base z = jt +0x1050 (sprung camera
        // height), rad_home = atan2(-fwd.y, -fwd.x), s_adjust = 100 (0x42c80000; the 0x00141640 tail scale
        // clamp(1.5 s - 0.5 s^2, 0, 1) of |vt+0x114| is not ported: 1, as the rec5 idle/run states read 100),
        // t_move_last = now. REPLACED fwd = (cos, sin) of jt +0x1a8 (the original reads the ALO matrix row 0).
        c.pos_base = [p[0], p[1], self.0.z_cpdefi, 1.0];
        c.v_base = [v[0], v[1], v[2], 0.0];
        c.rad_home = (-self.0.yaw.sin()).atan2(-self.0.yaw.cos());
        c.s_adjust = 100.0;
        c.t_move_last = self.0.t;
        // FEEL R112: 0x18a428 / 0x141640 set grfcpd bit 0 (CplcyFollow g_follow -> 0 through SMP_GAIN) in the jump with the
        // double jump still available, damage, jts 3 / 5 / 6. Capture cap_feel_cam_lag fol_tg +0x64: 1 -> 0.933 0.8 .. 0.045
        // on take-off / damage, back up the same profile after the double jump / landing. SLY2_FEEL_NO_R112FOLLOW=1 = old.
        if self.0.follow_off && std::env::var_os("SLY2_FEEL_NO_R112FOLLOW").is_none() {
            c.grfcpd[0] |= 1;
        }
        c
    }
    fn jt_state(&mut self, _pso: u32) -> Option<i32> {
        self.0.valid.then_some(self.0.jts)
    }
}

impl FollowWorld for PlayerFollowWorld {
    fn jt(&mut self, _pso: u32) -> Option<FollowJt> {
        self.0.valid.then_some(FollowJt { state: self.0.jts, pos_500: self.0.pos, f_10e4: 0, a_4c8: self.0.attach.0 as u32, a_4cc: self.0.attach.1 as u32, t_528: 0, fwd: [self.0.yaw.cos(), self.0.yaw.sin()] })
    }
    fn so_pos(&mut self, _pso: u32) -> V3 {
        self.0.pos
    }
}

/// MURRAY: the po's FUN_001938c0 kind-7 drops of the last class frame (p20::PoHost::po_call, queued on the
/// shadow JT) -> vt+0x19c (slot 50)(target, po): class 35 / 36 FUN_00184130 / FUN_00184330 (-> FUN_001b7000)
pub fn po_drain_slot50(set: &mut crate::engine::classes::ClassSet, host: &mut crate::engine::classes::plugin::EngineHost) {
    let Some(ts) = host.po.as_mut().map(|p| std::mem::take(&mut p.jt.slot50)) else { return };
    let (user, class) = slot50_user(set);
    for t in ts {
        let i = t.wrapping_sub(1);
        set.k10_use(i, user, class, host);
    }
}

/// MURRAY: the slot-50 user (vt+0x19c(target, po)) is the controlled PO: Sly (PLAYER_HANDLE, class 0x35), or
/// the active Bentley (0x37) / Murray (0x36) the JT body stands in for (REPLACED, p20 / p22::follow_active)
pub fn slot50_user(set: &mut crate::engine::classes::ClassSet) -> (u32, i32) {
    let Some(o) = set.k18_active else { return (crate::engine::classes::guard::PLAYER_HANDLE, 0x35) };
    if set.k18_murray(o).is_some() {
        (o as u32 + 1, super::p20::CLASS_MURRAY as i32)
    } else if set.k16_get(o).is_some() {
        (o as u32 + 1, super::p20::CLASS_BENTLEY as i32)
    } else {
        (crate::engine::classes::guard::PLAYER_HANDLE, 0x35)
    }
}

/// MURRAY: the JOY handed to the class po (k18_feed_joy) with its press stamps (+0x?? press_t, JT clock
/// `jt_t`) moved onto the class clock `g_t`: on PS2 one g_clock stamps the pad (FUN_001c4788) and is read by
/// the po's FUN_001c46c0 / FUN_001c4660 windows (0.1..0.2 s); the port's JT and class clocks start apart.
pub fn po_joy_on_clock(mut joy: crate::engine::player::Joy, jt_t: f32, g_t: f32) -> crate::engine::player::Joy {
    for p in joy.press_t.iter_mut() {
        if *p != crate::engine::player::PRESS_NONE {
            *p += g_t - jt_t;
        }
    }
    joy
}
