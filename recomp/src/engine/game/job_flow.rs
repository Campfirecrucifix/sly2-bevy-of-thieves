//! WP-J13: job manager level-load path and the new-game entry.
//!
//! 0x00161a58 case 5 (from FUN_0020e890 after the level load): FUN_00165c98 orders the class-0x85 tasks
//! (prerequisite roots first, FUN_00164100 recursion over the dependents), FUN_00164270 gives each task its
//! state from the save data (under +0xf4 = 1 / +0xf8 = 0), the current job / task / restart task come
//! from the job manager slots 0x2deb40 / 0x2deb44 / 0x2deb48 (written by 0x165dc0 / 0x165de8 / 0x165e10),
//! 0x1629a0 starts the job, 0x163d18 enters the task, request flag 4 (DAT_003e1084, set by 0x165280 /
//! 0x165538) sends event 0x1e through 0x1657a8, then FUN_001643b0 publishes every task (+0xf4 = 0).
//! Class-133 post-load FUN_001638f8: the prerequisite (+0x90 oids -> +0x94 / +0x98) and dependent (+0x9c
//! -> +0xa0 / +0xa4) task lists, oids FUN_0015e938(world, 0x105, oid, 0) resolves dropped.
//! New game FUN_00165e38: restart task 0x62f, no job / task, level slot 0 (FUN_00210b98(0) =
//! DAT_003e1c50[0], the level table's slot descriptors), spawn 0x189, flags 0x48.
//!
//! 1:1: 0x165c98, 0x164100, 0x164270, 0x161a58 case 5, 0x1657a8, 0x1638f8 (task lists), 0x210b98,
//! 0x165e38 (new-game branch), 0x1651d8, 0x165dc0 / 0x165de8 / 0x165e10 slot writes.
//! REPLACED: save-variable reads FUN_002581c0 of the task handles +0xe0 / +0xe8 and of DAT_003e0fac /
//! DAT_003e0fb8 -> `SavedTask` / `NewGameVars` (the handles are bound by the save-variable system, not
//! ported: a new game reads 0); FUN_0015e938(0x105) -> `JobMgr::tasks` by oid; DAT_003e1084 -> the
//! `flags` of the request that loaded the level; DAT_002dd... allocations FUN_001ae378 -> Vec.
//! WP-J16: the FUN_00165e38 continue branch (FUN_00210b98(episodeCurrent), DAT_003e100c[episode] > 99 ->
//! hideout) is `flow::FUN_00165e38_full`. UNPORTED: FUN_001d9880 / FUN_001d54b8 / FUN_001dd888 (memory-card
//! "game started" check and its menu), 0x1638f8 rest (+0xc8 / +0xd0 counters, LO flag 2).
#![allow(non_snake_case)]

use crate::engine::disc::descriptor::{Descriptor, LevelTable};
use crate::engine::game::transition::LevelRequest;
use crate::engine::game::fields::ObjectFields;
use crate::engine::jobs::p9::{FUN_00161fc8, FUN_00161ff0};
use crate::engine::game::save::{GameData, SaveValue, SaveVars};
use crate::engine::jobs::{AnimRec, JobMgr, ToggleRec, VarOwner, SPAWN_DEFAULT};
use crate::engine::world::props::{ArrayItem, Prop, PropValue};
use crate::engine::world::loads_b4::B4Load;
use crate::engine::world::objects::{LoadData, Payload};
use crate::engine::world::World;

/// 0x2deb30 job manager +0x10 / +0x14 / +0x18 / +0x1c (DAT_002deb40 / 44 / 48 / 4c): oids of the
/// current job, current task, restart task (-1 = none) and the save slot of FUN_00165e38. They survive
/// level loads (the per-level `JobMgr` does not).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JobSlots {
    pub job: i32,
    pub task: i32,
    pub restart: i32,
    pub slot: i32,
}

impl Default for JobSlots {
    fn default() -> Self {
        JobSlots { job: -1, task: -1, restart: -1, slot: -1 }
    }
}

/// New-game restart task oid (FUN_00165e38 / 0x62f).
pub const NEW_GAME_TASK: i32 = 0x62f;

/// FUN_002581c0 results of a task's +0xe0 (state, code 0x15) and +0xe8 (code 0x17) save variables.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SavedTask {
    pub state: i32,
    pub e8: i32,
}

/// WP-J15: FUN_002581c0(out, handle, 0x3d4a60) payload word (CRef +4) of a save-variable handle; a null
/// handle (empty path) or an unresolved path reads 0.
pub fn FUN_002581c0(path: &[u16], vars: &SaveVars, data: &GameData) -> i32 {
    if path.is_empty() {
        return 0;
    }
    match vars.ee_sub_1d7ed8(path, data) {
        SaveValue::Int(n) => n,
        SaveValue::Float(f) => f.to_bits() as i32,
        SaveValue::Bool(b) => b as i32,
        SaveValue::Quad(_, q) => q[0] as i32,
        SaveValue::Pair(q) => q[0] as i32,
        SaveValue::Triple(q) => q[0] as i32,
        SaveValue::Nil => 0,
    }
}

/// WP-J15: the task save variables FUN_00164270 reads (+0xe0 state, +0xe8).
pub fn saved_task(m: &JobMgr, t: usize, vars: &SaveVars, data: &GameData) -> SavedTask {
    let s = &m.tasks[t].save;
    SavedTask { state: FUN_002581c0(&s[3], vars, data), e8: FUN_002581c0(&s[4], vars, data) }
}

/// WP-J15: the job save variables FUN_00162780 reads (+0xd0 -> +0x7c, +0xe0 -> +0xa4 float). REPLACED:
/// +0xa8 = FUN_001d7e80 (address of the +0xe8 variable in the blob) -> 0.
pub fn saved_job(m: &JobMgr, j: usize, vars: &SaveVars, data: &GameData) -> crate::engine::jobs::p14::JobSaved {
    let s = &m.jobs[j].save;
    crate::engine::jobs::p14::JobSaved { n_7c: FUN_002581c0(&s[1], vars, data), a4: f32::from_bits(FUN_002581c0(&s[3], vars, data) as u32), a8: 0 }
}

/// WP-J15: FUN_00258200(out, handle, value, 0x3d4a60) for the writes 0x163460 / 0x1643b0 recorded in
/// `JobMgr::vars` ({code 0x15 int / 0x16 float / 0x17 bool, value}); drains them.
pub fn flush_vars(m: &mut JobMgr, vars: &SaveVars, data: &mut GameData) {
    for (o, off, code, v) in std::mem::take(&mut m.vars) {
        let save = match o {
            VarOwner::Job(j) => m.jobs.get(j).map(|j| &j.save),
            VarOwner::Task(t) => m.tasks.get(t).map(|t| &t.save),
        };
        let Some(path) = save.and_then(|s| s.get((off as usize).wrapping_sub(0xc8) / 8)).filter(|p| !p.is_empty()) else { continue };
        let val = match code {
            0x16 => SaveValue::Float(f32::from_bits(v as u32)),
            0x17 => SaveValue::Bool(v != 0),
            _ => SaveValue::Int(v),
        };
        vars.ee_sub_1d80c8(path, val, data);
    }
}

/// WP-J15: `JobMgr::playing` (REPLACED FUN_002121e8: the label lists +0x80 / +0x84 of the class-0x405
/// object and FUN_00218710 are not ported) fed from the level's ASEGA players: every label the job code asks
/// for (+0xb0 / +0xb4 on the +0xb8 object, 0x379 on +0xbc) counts as playing while a live ASEGA animates that
/// object's oid. `live_oids` = oids of the objects with a live ASEGA.
pub fn feed_playing(m: &mut JobMgr, live_oids: &[i32]) {
    m.playing.clear();
    for j in 0..m.jobs.len() {
        let job = &m.jobs[j];
        for (o, l) in [(job.aseg_b8, job.b0), (job.aseg_b8, job.b4), (job.aseg_bc, crate::engine::jobs::p14::LABEL_379)] {
            if let Some(o) = o.filter(|o| live_oids.contains(o)) {
                if l != -1 && !m.playing.contains(&(o, l)) {
                    m.playing.push((o, l));
                }
            }
        }
    }
}

/// 0x001638f8 (task list part): +0x98 prerequisites / +0xa4 dependents from the load's oid lists
/// (`B4Load::L1635f8` a / b), unresolved oids dropped.
pub fn FUN_001638f8(m: &mut JobMgr, w: &World) {
    for t in 0..m.tasks.len() {
        let Some(Payload::Loaded(LoadData::B4(b))) = w.objects.get(m.tasks[t].obj).map(|o| &o.payload) else { continue };
        let B4Load::L1635f8 { a, b, .. } = &**b else { continue };
        let res = |oids: &[i32]| oids.iter().filter_map(|&o| m.tasks.iter().position(|t| t.oid == o)).collect::<Vec<_>>();
        let (pre, dep) = (res(a), res(b));
        m.tasks[t].prereq = pre;
        m.tasks[t].deps = dep;
    }
}

pub(crate) fn item_i32(it: Option<&ArrayItem>) -> i32 {
    match it {
        Some(ArrayItem::One(_, b)) => match b.len() {
            0 => 0,
            1 => b[0] as i8 as i32,
            2 | 3 => i16::from_le_bytes([b[0], b[1]]) as i32,
            _ => i32::from_le_bytes([b[0], b[1], b[2], b[3]]),
        },
        Some(ArrayItem::List(_, l)) => item_i32(l.first().map(|b| ArrayItem::One(0, b.clone())).as_ref()),
        None => -1,
    }
}

/// FUN_00162050 (0x9050 list of states -> FUN_00161ff0 mask).
fn item_states(it: Option<&ArrayItem>) -> u32 {
    match it {
        Some(ArrayItem::List(_, l)) => FUN_00161ff0(&l.iter().map(|b| item_i32(Some(&ArrayItem::One(0, b.clone())))).collect::<Vec<_>>()),
        other => FUN_00161fc8(item_i32(other)),
    }
}

/// Record props of FUN_001a5c28 (array path, flags 0x400 method rows of methods_gen): job 1236 / 1237
/// (0x162600 / 0x162630 fail / complete events), 1255 (0x1626c0 toggle), 1256 (0x162728 anim), task 1287
/// (0x163838 toggle, state list via 0x162050), 1288 (0x1638a0 anim). The adders require +0x50 == 0 and
/// keep at most 0x20 records.
pub fn FUN_001a5c28_records(m: &mut JobMgr, w: &World) {
    let props = |obj: usize| -> Vec<Prop> {
        match w.objects.get(obj).map(|o| &o.payload) {
            Some(Payload::Loaded(LoadData::B4(b))) => match &**b {
                B4Load::L1624a8 { props, .. } | B4Load::L1635f8 { props, .. } => props.clone(),
                _ => Vec::new(),
            },
            _ => Vec::new(),
        }
    };
    for j in 0..m.jobs.len() {
        for p in props(m.jobs[j].obj) {
            let PropValue::Array(a) = &p.value else { continue };
            let e = |i: usize| item_i32(a.get(i));
            let job = &mut m.jobs[j];
            match p.field {
                // 0x162600 / 0x162630: no +0x50 check
                1236 if job.fail_ev.len() < 0x20 => job.fail_ev.push(e(0)),
                1237 if job.done_ev.len() < 0x20 => job.done_ev.push(e(0)),
                1255 if job.f_50 == 0 && job.toggles.len() < 0x20 => job.toggles.push(ToggleRec { oid: e(0), parent: e(2), mask: FUN_00161fc8(e(1)), row: e(3) }),
                1256 if job.f_50 == 0 && job.anims.len() < 0x20 => job.anims.push(AnimRec { state: e(0), oid: e(1), sel: e(2), anim: e(3), gated: 0, row: e(4) }),
                _ => {}
            }
        }
    }
    for t in 0..m.tasks.len() {
        for p in props(m.tasks[t].obj) {
            let PropValue::Array(a) = &p.value else { continue };
            let e = |i: usize| item_i32(a.get(i));
            let tk = &mut m.tasks[t];
            match p.field {
                1287 if tk.f_50 == 0 && tk.toggles.len() < 0x20 => tk.toggles.push(ToggleRec { oid: e(0), parent: e(2), mask: item_states(a.get(1)), row: e(3) }),
                1288 if tk.f_50 == 0 && tk.anims.len() < 0x20 => tk.anims.push(AnimRec { state: e(0), oid: e(1), sel: e(2), anim: e(3), gated: e(4), row: e(5) }),
                _ => {}
            }
        }
    }
}

/// The object of a 0x1620b0 toggle: 0x15e938(world, 5, parent) then (4 under it | 5 anywhere, oid).
/// REPLACED: an object with `oid` below (playerdraw::ee_sub_15e938_below) an object with oid `parent` (the
/// first with `oid` when `parent` is -1).
pub fn toggle_object(w: &World, oid: i32, parent: i32) -> Option<usize> {
    if parent == -1 {
        return w.find_oid(oid).first().copied();
    }
    w.find_oid(parent).iter().find_map(|&p| crate::engine::playerdraw::ee_sub_15e938_below(w, p, oid))
}

/// Applies `JobMgr::toggles[*seen..]` (vtable +0x44 Add / +0x4c Remove through the LO rows 0x100a90 /
/// 0x100b88) to the script-visible object state.
pub fn apply_toggles(m: &JobMgr, objs: &mut ObjectFields, w: &World, seen: &mut usize) {
    if m.toggles.len() < *seen {
        *seen = 0; // new world
    }
    for &(oid, parent, show) in &m.toggles[*seen..] {
        if let Some(o) = toggle_object(w, oid, parent) {
            super::methods::lo::call(objs, Some(w), if show { 0x0010_0a90 } else { 0x0010_0b88 }, o as u32, 0);
        }
    }
    *seen = m.toggles.len();
}

/// Per-task scratch of FUN_00164100: +0xfc visits, +0x100 depth, +0x104 job-end depth.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Order {
    pub fc: i32,
    pub d100: i32,
    pub d104: i32,
}

/// 0x00164100: append `t` once all its prerequisites were visited, then visit its dependents (under the
/// task's +0xf4 / +0xf8).
pub fn FUN_00164100(m: &mut JobMgr, t: usize, out: &mut Vec<usize>, sc: &mut [Order], p4: i32, p5: i32) {
    let o = &mut sc[t];
    o.fc += 1;
    o.d100 = o.d100.max(p4);
    o.d104 = o.d104.max(p5);
    if (m.tasks[t].prereq.len() as i32) > o.fc {
        return;
    }
    out.push(t);
    let p5n = match m.tasks[t].job {
        Some(j) if m.jobs[j].end == Some(t) => p5 + 1,
        _ => p5,
    };
    for d in m.tasks[t].deps.clone() {
        let (f4, f8) = (m.tasks[d].f4, m.tasks[d].f8);
        m.tasks[d].f4 = m.tasks[t].f4;
        m.tasks[d].f8 = m.tasks[t].f8;
        FUN_00164100(m, d, out, sc, p4 + 1, p5n);
        m.tasks[d].f4 = f4;
        m.tasks[d].f8 = f8;
    }
}

/// 0x00165c98: the class-0x85 tasks in dependency order (roots = tasks without prerequisites).
pub fn FUN_00165c98(m: &mut JobMgr) -> Vec<usize> {
    let mut out = Vec::new();
    let mut sc = vec![Order::default(); m.tasks.len()];
    for t in 0..m.tasks.len() {
        if m.tasks[t].prereq.is_empty() {
            FUN_00164100(m, t, &mut out, &mut sc, 0, 0);
        }
    }
    out
}

/// 0x00164270: initial state of task `t` from its save variables.
pub fn FUN_00164270(m: &mut JobMgr, t: usize, saved: SavedTask) {
    let mut s = saved.state;
    if saved.e8 != 0 {
        if let Some(j) = m.tasks[t].job {
            if (1..3).contains(&s) {
                m.tasks[t].n_64 = 0;
                s = 1;
                if m.jobs[j].start == Some(t) {
                    m.jobs[j].n_7c = 0;
                }
                if !m.tasks[t].prereq.is_empty() && !m.tasks[t].prereq.iter().all(|&p| m.tasks[p].state > 1) {
                    s = 0;
                }
            }
        }
    }
    if s == 0 {
        s = m.tasks[t].prereq.is_empty() as i32;
    }
    m.FUN_001657d0(t, s);
}

/// 0x001657a8: event 0x1e for the task's job.
pub fn FUN_001657a8(m: &mut JobMgr, t: usize) {
    let j = m.tasks[t].job.map(|j| m.jobs[j].obj as i32 + 1).unwrap_or(0); // the job object (handle)
    m.FUN_001619a0(0x1e, j);
}

fn task_by_oid(m: &JobMgr, oid: i32) -> Option<usize> {
    (oid != -1).then(|| m.tasks.iter().position(|t| t.oid == oid)).flatten()
}

/// Slot writes of 0x165dc0 / 0x165de8 / 0x165e10 (the oid of the object, -1 without).
pub fn sync_slots(m: &JobMgr, s: &mut JobSlots) {
    s.job = m.cur.map(|j| m.jobs[j].oid).unwrap_or(-1);
    s.task = m.cur_task.map(|t| m.tasks[t].oid).unwrap_or(-1);
    s.restart = m.restart_override.map(|t| m.tasks[t].oid).unwrap_or(-1);
}

/// 0x00161a58 case 5 (level loaded): task states, current job / task / restart from `slots`, job start,
/// task enter. `flags` = DAT_003e1084 (the request flags of the load); `saved` = the task save variables.
pub fn FUN_00161a58_case5(m: &mut JobMgr, w: &World, slots: &mut JobSlots, flags: u32, saved: &dyn Fn(usize) -> SavedTask) {
    FUN_001a5c28_records(m, w);
    FUN_001638f8(m, w);
    let order = FUN_00165c98(m);
    for &t in &order {
        let (f4, f8) = (m.tasks[t].f4, m.tasks[t].f8);
        m.tasks[t].f4 = 1;
        m.tasks[t].f8 = 0;
        FUN_00164270(m, t, saved(t));
        m.tasks[t].f4 = f4;
        m.tasks[t].f8 = f8;
    }
    // FUN_0015e938(0x105, oid) of the slots (0x165dc0 etc. rewrite the slots from the objects)
    let s = *slots;
    let j = (s.job != -1).then(|| m.job_by_oid(s.job)).flatten();
    m.FUN_00165dc0(j);
    m.FUN_00165de8(task_by_oid(m, s.task));
    m.FUN_00165e10(task_by_oid(m, s.restart));
    if let Some(j) = m.cur {
        m.FUN_001629a0(j);
    }
    if let Some(t) = m.cur_task {
        m.FUN_00163d18(t);
    }
    if flags & 4 != 0 {
        match m.restart_override {
            None => {
                if let Some(t) = m.cur.and_then(|j| m.jobs[j].start) {
                    FUN_001657a8(m, t);
                }
            }
            Some(t) => FUN_001657a8(m, t),
        }
    }
    for &t in &order {
        let (f4, f8) = (m.tasks[t].f4, m.tasks[t].f8);
        m.tasks[t].f4 = 0;
        m.tasks[t].f8 = 0;
        m.FUN_001643b0(t);
        m.tasks[t].f4 = f4;
        m.tasks[t].f8 = f8;
    }
    sync_slots(m, slots);
}

/// 0x00210b98: slot descriptor `i` of the level table (DAT_003e1c50 + i * 0x40) when valid (+4 != +0x3c).
pub fn FUN_00210b98(t: &LevelTable, i: i32) -> Option<Descriptor> {
    if i == -1 {
        return None;
    }
    let d = *t.slots.get(usize::try_from(i).ok()?)?;
    (d.0[4..8] != d.0[0x3c..0x40]).then_some(d)
}

/// WP-EP1d: 0x00210b70 hub level of the job complete / end paths (0x162e38 / 0x162a60):
/// FUN_00210b98(FUN_00210ae0()) = slot descriptor DAT_003e1c50[DAT_003e1c44[current row]]; REPLACED: the
/// descriptor -> the level-table row loading the same file (`JobReload` carries rows).
pub fn FUN_00210b70(t: &LevelTable, cur_row: i32) -> Option<i32> {
    let d = FUN_00210b98(t, crate::engine::game::flow::FUN_00210ae0(cur_row, Some(t)))?;
    t.entries.iter().position(|e| e.desc.same_file(&d)).map(|k| k as i32)
}

/// FUN_00165e38 save-variable reads (REPLACED, see the module doc).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NewGameVars {
    /// DAT_003e0fac
    pub started: f32,
    /// FUN_001d9880(0x3da530)
    pub listed: bool,
}

/// 0x00165e38: start the game of save slot `slot`; the new-game branch returns the request of level slot
/// 0 (None = continue / menu branch, unported).
pub fn FUN_00165e38(t: &LevelTable, slots: &mut JobSlots, slot: i32, v: NewGameVars) -> Option<LevelRequest> {
    if v.listed || v.started != 0.0 {
        return None;
    }
    *slots = JobSlots { job: -1, task: -1, restart: NEW_GAME_TASK, slot };
    // FUN_0022b490(world, FUN_00210b98(0), 0x189, -1, 0x48, -1); a null descriptor = the current level
    let d = FUN_00210b98(t, 0)?;
    let mut r = LevelRequest::new(d);
    r.flags = 0x48;
    r.spawn = SPAWN_DEFAULT;
    r.spawn_parent = -1;
    Some(r)
}

/// 0x001651d8: restart at task oid `oid` of level `desc` (task not in this level: set the slots and load).
pub fn FUN_001651d8(m: &mut JobMgr, slots: &mut JobSlots, desc: Option<Descriptor>, oid: i32) -> Option<LevelRequest> {
    match task_by_oid(m, oid) {
        None => {
            *slots = JobSlots { job: -1, task: -1, restart: oid, slot: slots.slot };
            let mut r = LevelRequest::new(desc?);
            r.flags = 8;
            Some(r)
        }
        Some(t) => {
            m.FUN_00165e10(Some(t));
            m.FUN_00165538(Some(t));
            sync_slots(m, slots);
            None
        }
    }
}
