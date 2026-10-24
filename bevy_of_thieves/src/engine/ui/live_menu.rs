//! WIRE-UI: the safehouse hub job-select (C2d80d0, FUN_001d72a8) and the ThiefNet screen (C2d84e8 singleton
//! 0x3da150, FUN_001db870 / FUN_001dc260 / FUN_001dbd10) run LIVE in the Bevy app on the queue store
//! (`classes::q::live::LiveQ`), driven by the real pad:
//! - new world: the static UI globals (FUN_001de260 = static_init_stores part, 0x3da150 ctor), the screen's
//!   pad handle +0x10 = 0x2dfc00, DAT_002dfbc0 = 7 (pad 0x2dfc00 present) and the FUN_0020e890 po selection
//!   (`LiveQ::select_po_at_load`, oidPoCurrent from the job manager = `player::p20::oid_po_current`): when the
//!   po is the hub its slot 78(1) puts it in state 1 with its SM cursor (+0x188).
//! - every frame (after `tick_live_q`, which runs the hub's update slot): the `PadInput` pad block (FUN_001871c0
//!   result) is copied into the guest pad 0x2dfc00 + 0x1000, then FUN_001d9648's FUN_001db870(0x3da150).
//! - the guest's recorded engine calls are handed to the class host (as the harness drains them):
//!   FUN_001dfd58 sequence starts (hub CROSS = start the picked job's exit sequence), FUN_00164640 task
//!   states, FUN_001619a0 events.
//! - `LiveMenuView` snapshots the ThiefNet fields for `ui::plugin::draw_ui` (`hud::thiefnet`).
use crate::engine::classes::q::live::{tick_live_q, write_pad, LiveQ};
use crate::engine::classes::q::q2mem::QMem;
use crate::engine::input::backend::PadInput;
use bevy::prelude::*;

/// C2d84e8 singleton (ThiefNet)
pub const TN: u32 = 0x003d_a150;
/// DAT_002dfc00
pub const PAD: u32 = 0x002d_fc00;

pub use super::thiefnet::ThiefNetRows as ThiefNetView;

/// Hub (C2d80d0) + ThiefNet state of the live store, rebuilt every frame.
#[derive(Resource, Clone, Debug, Default)]
pub struct LiveMenuView {
    /// hub object index (the po) and its SM label (FUN_001f07e8(+0x188) = selected character / laptop)
    pub hub: Option<usize>,
    pub hub_state: i32,
    pub hub_label: i32,
    /// ThiefNet open (+0x40 != -1)
    pub thiefnet: Option<ThiefNetView>,
    /// world objects whose sequence the hub started (FUN_001dfd58), oldest first
    pub started: Vec<usize>,
    /// FUN_001619a0 events (event, arg) and FUN_00164640 (task handle, state) seen from the store
    pub events: Vec<(i32, i32)>,
    pub task_calls: Vec<(u32, i32)>,
}

/// (world id, po selected): the po selection runs one frame after the globals so the level-load script calls
/// of the first script tick (the hub list FUN_001d7780) precede it, as at the end of the original load.
#[derive(Default)]
pub struct LiveMenuSeen(usize, bool);

/// New world: static UI globals, then (next frame) the po selection (see module doc).
pub fn live_menu_init(q: Option<ResMut<LiveQ>>, mut seen: Local<LiveMenuSeen>, jobs: Option<Res<crate::engine::player::plugin::PlayerJobs>>, mut view: ResMut<LiveMenuView>) {
    let Some(mut q) = q else { return };
    let Some(w) = q.world().cloned() else { return };
    let id = std::sync::Arc::as_ptr(&w) as usize;
    if q.mem.is_none() || (seen.0 == id && seen.1) {
        return;
    }
    if seen.0 != id {
        *seen = LiveMenuSeen(id, false);
        *view = LiveMenuView::default();
        let m = q.mem.as_mut().unwrap();
        crate::engine::game::q13_c::ee_sub_1de260(m);
        m.w32(TN + 0x10, PAD);
        m.w32(0x002d_fbc0, 7);
        return;
    }
    seen.1 = true;
    // FUN_001d9520 (UI post-load): the screens' +0x24 world objects (C2d36d0_v11: FUN_0015e938(world, 5, +0x20))
    q.mem.as_mut().unwrap().call(0x001d_9520, &[], &[]);
    let oid = jobs.map_or(-1, |j| crate::engine::player::p20::oid_po_current(&j.0));
    let po = q.select_po_at_load(&w, oid);
    view.hub = po.filter(|&i| {
        let m = q.mem.as_mut().unwrap();
        let o = m.obj_addr[i];
        m.r32(o + 0xc) == HUB_VT
    });
    info!("wire-ui: live menu init, po oid {oid} -> {:?}", po.map(|i| w.objects[i].oid));
}

/// CRAWL R103: the ThiefNet screen is up (C2d84e8 singleton +0x40 page != -1 with its +0x24 screen object:
/// FUN_001dc260 open .. close, during which the original's g_clock scale 0 is 0)
pub fn thiefnet_open(m: &crate::engine::classes::q::live::LiveMem) -> bool {
    m.peek32(TN + 0x24) != 0 && m.peek32(TN + 0x40) != u32::MAX
}

/// C2d80d0 vtable (object +0xc)
pub const HUB_VT: u32 = 0x002d_80d0;

/// Per frame: pad -> guest, FUN_001db870(0x3da150), drain, snapshot.
/// Before the queue tick: the `PadInput` pad block (FUN_001871c0 result) -> guest pad 0x2dfc00 + 0x1000, so
/// the hub update (FUN_001d72a8) and the screen see this frame's presses.
pub fn live_menu_pad(q: Option<ResMut<LiveQ>>, pad: Option<Res<PadInput>>) {
    if let (Some(mut q), Some(p)) = (q, pad) {
        if let Some(m) = q.mem.as_mut() {
            write_pad(m, PAD + 0x1000, &p.pad);
        }
    }
}

pub fn live_menu_tick(q: Option<ResMut<LiveQ>>, mut view: ResMut<LiveMenuView>, gps: Option<Res<crate::engine::input::backend::GlobalPadScript>>, classes: Option<ResMut<crate::engine::classes::plugin::Classes>>) {
    let Some(mut q) = q else { return };
    let Some(m) = q.mem.as_mut() else { return };
    q.ui_update();
    let m = q.mem.as_mut().unwrap();
    let seqs = std::mem::take(&mut m.recorded_1dfd58);
    let tasks = std::mem::take(&mut m.recorded_164640);
    let evs = std::mem::take(&mut m.recorded_1619a0);
    if let Some(mut c) = classes {
        // CRAWL R103: sequence slot 78(1) on Sly (exit seq 27254) -> FUN_001c3ed8(jt, 1) in the live image
        if std::mem::take(&mut c.host.sly_focus_req) && m.host_jt != 0 && m.hub_has_focus() {
            let jt = m.host_jt;
            crate::engine::game::q_wireq3::ee_sub_1c3ed8(m, jt, 1);
            let f = m.r32(0x003d_d490 + 0x3efc);
            info!("wire-ui: Sly takes the focus back (focus {f:#x} hub state {})", m.r32(f + 0x180));
        }
        for &(i, player) in &seqs {
            c.host.seq_starts.push((i as u32 + 1, player));
        }
        for &(h, k) in &tasks {
            c.host.k28_task_calls.push((0x164640, h, k, 0));
        }
    }
    view.started.extend(seqs.iter().map(|s| s.0));
    if !seqs.is_empty() {
        debug!("wire-ui: FUN_001dfd58 starts {seqs:?}");
    }
    view.task_calls.extend(tasks);
    view.events.extend(evs.iter().map(|e| (e.1, e.2)));
    // CRAWL R101: the hub also takes the focus after the load (hub scripts 258.4 sym3 / 258.12 sym5: set-o 17239
    // 806 1 -> slot 78(1), vm_state ref_op_set_o), not only as the load po: track it once it is in state 1
    if view.hub.is_none() && std::env::var_os("SLY2_NO_R101_HUB806").is_none() {
        view.hub = q.addr_of_oid(17239).and_then(|a| {
            let m = q.mem.as_mut().unwrap();
            (m.r32(a + 0xc) == HUB_VT && m.r32(a + 0x180) == 1).then(|| m.obj_addr.iter().position(|&x| x == a)).flatten()
        });
        if view.hub.is_some() {
            info!("wire-ui: hub 17239 took the focus (select state 1)");
        }
    }
    let m = q.mem.as_mut().unwrap();
    if let Some(h) = view.hub {
        let o = m.obj_addr[h];
        let s = m.r32(o + 0x180) as i32;
        if s != view.hub_state {
            let ents: Vec<[u32; 3]> = (0..m.r32(o + 0x18c).min(16)).map(|i| [0, 4, 8].map(|k| m.r32(o + 0x190 + 0xc * i + k))).collect();
            info!("wire-ui: [g{}] hub state {} -> {s} (+0x188 {:#x}), started {:?}, pad entries {:x?}", gps.as_ref().map_or(0, |g| g.frame), view.hub_state, m.r32(o + 0x188), seqs, ents);
        }
        view.hub_state = s;
        let l = m.r32(o + 0x188);
        view.hub_label = if l == 0 { -1 } else { m.call_top(0x001f_07e8, &[l]).map_or(-1, |x| x as i32) };
    }
    let tn = (m.r32(TN + 0x40) as i32, m.r32(TN + 0x24));
    if view.thiefnet.as_ref().map(|t| t.page) != (tn.0 != -1).then_some(tn.0) {
        info!("wire-ui: [g{}] ThiefNet page {} (screen {:#x}, lock {:#x}, scale0 {})", gps.as_ref().map_or(0, |g| g.frame), tn.0, tn.1, m.r32(0x002d_fbc4), f32::from_bits(m.r32(0x002d_ded0 + 0x18)));
    }
    view.thiefnet = (m.r32(TN + 0x40) != u32::MAX).then(|| {
        let n = (m.r32(TN + 0x48) as i32).clamp(0, 0x40) as u32;
        ThiefNetView {
            page: m.r32(TN + 0x40) as i32,
            dialog: m.r32(TN + 0x30) as i32,
            dialog_msg: m.r32(TN + 0x38) as i32,
            ent: (0..n).map(|i| [m.r32(TN + 0x4c + 8 * i) as i32, m.r32(TN + 0x50 + 8 * i) as i32]).collect(),
            sel: m.r32(TN + 0x24c) as i32,
            first: m.r32(TN + 0x250) as i32,
        }
    });
}

pub struct LiveMenuPlugin;
impl Plugin for LiveMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LiveMenuView>().add_systems(
            Update,
            (live_menu_init, live_menu_tick)
                .chain()
                .after(tick_live_q)
                .before(crate::engine::game::app::lend_classes),
        )
        .add_systems(Update, live_menu_pad.before(tick_live_q));
    }
}
