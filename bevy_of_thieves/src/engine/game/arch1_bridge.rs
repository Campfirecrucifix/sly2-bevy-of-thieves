//! WP-ARCH1 bridge: wires the ARCH1 primitives into the running game as a Bevy system that runs
//! each frame after `tick_live_q`, so recorded script side-effects and messages become real calls
//! on the guest-memory store (not just facts in `recomp/ARCHITECTURE.md`).
//!
//! What this does per frame:
//!   (4) `JobMgr::replay_calls_through(mem)` — every recorded `(addr, arg)` whose addr has a queue
//!       port is dispatched 1:1 through `QMem::call`.
//!   (2) `LiveQ::deliver_message` on every new `JobMgr::task_msgs` and `ev_msgs` entry so vt+0x1c
//!       messages that the hand splice already delivers also run on the store.
//!   (3) `pj_adapt::sync_in_live` / `sync_out_live` on the active `Player::jt` at `0x002d2e98`
//!       (the real DAT global the ELF image owns inside `LiveMem`'s static pages). The sync is
//!       round-trip around a `live_q.frame`-driven pass so any queue port that wrote a mapped Jt
//!       offset reaches `Player::jt`.

use bevy_ecs::prelude::*;

use crate::engine::classes::q::live::LiveQ;
use crate::engine::player::pj_adapt;
use crate::engine::player::plugin::{Player, PlayerJobs};
use crate::engine::world::World;

/// `DAT_002d2e98`: real guest address of the player `Jt` global. The `LiveMem` static-page cache
/// backs this address from the SCUS_973.16 PT_LOAD on first touch; sync_in / sync_out operate on
/// the mapped Jt offsets described by `pj_adapt::FIELDS`.
pub const JT_BASE: u32 = 0x002d_2e98;

/// Running totals of what the bridge did this session.
#[derive(Resource, Default, Debug, Clone)]
pub struct Arch1Bridge {
    pub task_msg_seen: usize,
    pub ev_msg_seen: usize,
    pub calls_dispatched: usize,
    pub messages_delivered: usize,
    pub jt_syncs: usize,
    /// CRAWL R113: World the frontiers belong to (a level load replaces `JobMgr`: its message lists restart)
    pub world_key: usize,
}

/// WP-ARCH1 (a + b): run after `tick_live_q` and `tick_scripts`; drive the store with everything
/// the hand systems recorded. No-op when there is no world or no built store (`live_q.mem` None).
pub fn arch1_bridge_tick(
    mut live_q: ResMut<LiveQ>,
    mut jobs: ResMut<PlayerJobs>,
    world: Option<Res<World>>,
    mut players: Query<&mut Player>,
    mut bridge: ResMut<Arch1Bridge>,
) {
    let Some(w) = world else { return };
    if live_q.mem.is_none() {
        return;
    }

    // (4) Replay recorded script side-effect calls through the store (`QMem::call`).
    if let Some(m) = live_q.mem.as_mut() {
        bridge.calls_dispatched += jobs.0.replay_calls_through(m);
    }

    // (3) pj_adapt view sync for the active `Player::jt` at `DAT_002d2e98`.
    for mut p in players.iter_mut() {
        if let Some(m) = live_q.mem.as_mut() {
            pj_adapt::sync_in_live(m, JT_BASE, &p.jt);
            pj_adapt::sync_out_live(m, JT_BASE, &mut p.jt);
            bridge.jt_syncs += 1;
        }
    }

    // (2) Deliver vt+0x1c messages through the store too (hand splice path still runs).
    // Snapshot the frontier BEFORE calling `deliver_message` so re-entrant pushes stay buffered.
    // CRAWL R113 (FEEL R93 live --table 2 crash "slice index 213 out of range for length 198" after a reload):
    // a level load builds a new `JobMgr` (JobMgr::load), so its `task_msgs` / `ev_msgs` restart while the
    // frontiers kept the old world's counts. Reset them on a new World or a shrunk list, as
    // vm_state::job_messages does for the hand splice path.
    let wk = &*w as *const World as usize;
    if bridge.world_key != wk {
        bridge.world_key = wk;
        bridge.task_msg_seen = 0;
        bridge.ev_msg_seen = 0;
    }
    if jobs.0.task_msgs.len() < bridge.task_msg_seen {
        bridge.task_msg_seen = 0;
    }
    if jobs.0.ev_msgs.len() < bridge.ev_msg_seen {
        bridge.ev_msg_seen = 0;
    }
    let new_task = jobs.0.task_msgs.len();
    let task_oids: Vec<(i32, u32)> = jobs.0.task_msgs[bridge.task_msg_seen..new_task]
        .iter()
        .filter_map(|&(ti, msg)| jobs.0.tasks.get(ti).map(|t| (t.oid, msg)))
        .collect();
    for (oid, msg) in task_oids {
        if live_q.deliver_message(oid, msg as i32, 0) {
            bridge.messages_delivered += 1;
        }
    }
    bridge.task_msg_seen = new_task;

    let new_ev = jobs.0.ev_msgs.len();
    let ev_items: Vec<(i32, i32, u32)> = jobs.0.ev_msgs[bridge.ev_msg_seen..new_ev]
        .iter()
        .filter_map(|&(oi, ev, arg)| w.objects.get(oi).map(|o| (o.oid, ev, arg)))
        .collect();
    for (oid, ev, arg) in ev_items {
        if live_q.deliver_message(oid, ev, arg) {
            bridge.messages_delivered += 1;
        }
    }
    bridge.ev_msg_seen = new_ev;
}

pub struct Arch1BridgePlugin;
impl bevy_app::Plugin for Arch1BridgePlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.init_resource::<Arch1Bridge>()
            .add_systems(bevy_app::Update, arch1_bridge_tick.after(crate::engine::classes::q::live::tick_live_q));
    }
}
