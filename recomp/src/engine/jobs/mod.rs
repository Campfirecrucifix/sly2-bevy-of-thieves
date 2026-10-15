//! WP-P8: jobs (missions) core: job objects (class 132 C2cf378, vtable 0x2cf378, load 0x1624a8), their
//! tasks (class 133 C2cf1e8, load 0x1635f8) and the job manager event path 0x1619a0 used by Sly's death.
//!
//! World state: DAT_003e0f9c = world+0x3b0c current job (setter 0x165dc0, also DAT_002deb40 = job+0x18),
//! DAT_003e0fa0 = +0x3b10 (0x165de8), DAT_003e0fa4 = +0x3b14 restart task override (0x165e10), DAT_003e0f90
//! task list (+0x88 next), DAT_003e1110 current level row (0x210a88), DAT_003e1c40 level table (0x40 bytes
//! per row, 0x210990), FUN_0020e040(world) "world busy".
//! Job fields: +0x50 LO flag (0x162670 / 0x1637e8 require 0; writer not traced), +0x54 / +0x58 fail-event
//! list, +0x5c / +0x60 complete-event list, +0x64 start task (prop 1243), +0x68 end task (prop 1244), +0x7c
//! start counter, +0xf0 state (2 / 3 = suspended). Task fields: +0x54 state (0x164030: >= 2 = done), +0x7c
//! job (prop 1264), +0xb8 spawn oid (prop 1274, -1 -> 0x189), +0xbc level row (prop 1275, s16), +0xc0
//! (prop 1276; != -1 -> 0x1d8b20 message 0x15), +0xf4 state.
//!
//! Death: 0x1a2388 -> 0x1619a0(0x2deb30, 0x10, jt) -> 0x161a58 (no job: reload the level, P7) -> 0x154a90
//! (difficulty, unported) -> 0x165170 for every task of the current job (0x1637e8 -> msg 0x11 {ev, arg})
//! -> 0x162bb8(job, 0x10): job running (0x162670) -> msg 0x11 to the job's scripts, event in the fail list
//! -> 0x162dd0 (event 0x1d), in the complete list -> 0x162e38; job not running, +0x50 set and the world
//! idle -> 0x162df8 -> 0x165538(start task or the +0x3b14 override) = restart: switch the current job,
//! 0x164df0 / 0x163c28 task resets, then FUN_0022b490(world, row(+0xbc), spawn(+0xb8 | 0x189), -1, 4 /
//! 0x24, 0x30a) = reload the task's level at its spawn.
//! The same restart is reached from the pause menu (0x211d38, DAT_002dd318 retry counter) and the splice
//! builtin 0x10ea98.
//!
//! 1:1: 0x1619a0 (job half), 0x162670, 0x1637e8, 0x165170, 0x162bb8, 0x162dd0, 0x162df8, 0x162e38,
//! 0x165538, 0x1628b8, 0x162908, 0x1633e8, 0x165dc0, 0x210990, 0x164030, 0x1649d0.
//! 0x1629a0 start, 0x162a60 end, 0x162ea0 (WP-P9: their 0x163460 / 0x1620b0 / 0x1623d0 / 0x162940 / 0x164df0 /
//! 0x163ce0 callees are ported in `p9`; script messages vt+0x1c 0x13 / 0x14 still recorded).
//! REPLACED: script messages (vt+0x1c 0x11 / 0x13 / 0x14 go to the splice listeners) -> `JobMgr::calls`;
//! FUN_00210b70 (hub level of 0x162e38 / 0x162a60, = FUN_00210b98(FUN_00210ae0())) -> `JobMgr::hub_row`;
//! save-variable states (+0xd8 / 0x2581c0) -> `Task::state` only; the fail / complete event lists (+0x58 /
//! +0x60, filled by props not mapped yet) -> `Job::fail_ev` / `Job::done_ev` (empty from the level).
//! UNPORTED: 0x161a58 cases other than 0x10, 0x1d8b20, 0x154a90. WP-J14 (`p14`): 0x162f10 job complete,
//! job update slot 15 0x163078 / post-load slot 14 0x162780, task slot 15 0x163b48. Who sets job +0x50: not found (WP-P9: no class-132
//! field binding maps +0x50; 0x162670 / 0x1637e8 / 0x162bb8 only read it).
#![allow(non_snake_case)]

pub mod p14;
pub mod p9;
pub use p9::{AnimRec, ToggleRec, VarOwner};

use crate::engine::disc::descriptor::Descriptor;
use crate::engine::game::transition::LevelRequest;
use crate::engine::world::loads_b4::B4Load;
use crate::engine::world::objects::{LoadData, Payload};
use crate::engine::world::props::{Prop, PropValue};
use crate::engine::world::World;

pub const CID_JOB: u16 = 132;
pub const CID_TASK: u16 = 133;
/// default spawn of 0x165538 / 0x162e38 / 0x162a60
pub const SPAWN_DEFAULT: i32 = 0x189;
/// job manager events (0x1619a0 param_2)
pub const EV_DIED: i32 = 0x10;
pub const EV_FAILED: i32 = 0x1d;
pub const EV_ENDED: i32 = 0x1f;
pub const EV_STARTED: i32 = 0x21;
pub const EV_STOPPED: i32 = 0x22;

/// class 132 object.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Job {
    pub obj: usize,
    pub oid: i32,
    /// WP-J15: save-variable handles +0xc8 (unused) / +0xd0 / +0xd8 / +0xe0 / +0xe8 (props 1262 / 1263 /
    /// 1260 / 1261, kind 0x16 symbol paths; empty = null handle)
    pub save: [Vec<u16>; 5],
    /// +0x50
    pub f_50: i32,
    /// +0x58 (count +0x54)
    pub fail_ev: Vec<i32>,
    /// +0x60 (count +0x5c)
    pub done_ev: Vec<i32>,
    /// +0x64 start task (index into `JobMgr::tasks`)
    pub start: Option<usize>,
    /// +0x68 end task
    pub end: Option<usize>,
    /// +0x7c start counter
    pub n_7c: i32,
    /// +0xf0
    pub state: i32,
    /// +0x80 task list (task +0x84 next): the tasks whose +0x7c is this job (WP-P9)
    pub tasks: Vec<usize>,
    /// +0x78
    pub f_78: i32,
    /// +0xa4 job timer (save variable +0xe0; FUN_00161478 adds the game dt; WP-J14 f32)
    pub a4: f32,
    /// WP-J14: +0xa8 (save variable +0xe8 via 0x1d7e80)
    pub a8: i32,
    /// +0x9c / +0xa0 difficulty ramp start / end (ctor 480 / 720)
    pub f_9c: f32,
    pub f_a0: f32,
    /// +0xac complete aseg oid, +0xb0 / +0xb4 its start / outro labels (method 1258 0x162660, ctor -1)
    pub ac: i32,
    pub b0: i32,
    pub b4: i32,
    /// +0xb8 complete aseg (object oid), +0xbc aseg 0x379 (post-load 0x162780)
    pub aseg_b8: Option<i32>,
    pub aseg_bc: Option<i32>,
    /// +0xc0 next task of the complete sequence, +0xc4 its state, +0xc8 state time
    pub c0: Option<usize>,
    pub c4: i32,
    pub t_c8: f32,
    /// +0xcc object (prop 1259) shown when there is no next task
    pub cc: Option<i32>,
    /// +0xf4 (0x164640 save / restore)
    pub f4: i32,
    /// LO +0x40 flags 0x102 of 0x163170
    pub flags_40: u32,
    /// +0x8c / +0x90 FUN_001620b0 records
    pub toggles: Vec<ToggleRec>,
    /// +0x94 / +0x98 FUN_001623d0 records
    pub anims: Vec<AnimRec>,
}

/// class 133 object.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Task {
    pub obj: usize,
    pub oid: i32,
    /// +0x50
    pub f_50: i32,
    /// +0x54
    pub state: i32,
    /// +0x7c job (index into `JobMgr::jobs`)
    pub job: Option<usize>,
    /// +0xb8
    pub spawn: i32,
    /// +0xbc
    pub level: i32,
    /// +0xc0
    pub c0: i32,
    /// +0xf4
    pub f4: i32,
    /// +0x64 enter counter (WP-P9)
    pub n_64: i32,
    /// +0x68 done counter
    pub n_68: i32,
    /// +0xf0 pending state of 0x164df0 (-1 = none)
    pub f0: i32,
    /// +0xf8
    pub f8: i32,
    /// +0x5c / +0x60 music cue of 0x163d18 / 0x163f48
    pub f_5c: i32,
    pub f_60: i32,
    /// +0x70 enter when active, +0x74 DAT_002deb44 when active
    pub f_70: i32,
    pub f_74: i32,
    /// +0xc4 object flagged 0x100 on enter
    pub c4: Option<i32>,
    /// +0x94 / +0x98 prerequisite tasks
    pub prereq: Vec<usize>,
    /// +0xa0 / +0xa4 dependent tasks
    pub deps: Vec<usize>,
    /// +0xa8 / +0xac FUN_001620b0 records
    pub toggles: Vec<ToggleRec>,
    /// +0xb0 / +0xb4 FUN_001623d0 records
    pub anims: Vec<AnimRec>,
    /// WP-J14: +0x6c (prop 1271; FUN_001645d0 counts an active prerequisite with it as done)
    pub f_6c: i32,
    /// +0x58 state time (0x1657d0), +0x78 time limit (prop 1285)
    pub t_58: f32,
    pub t_78: f32,
    /// +0x80 object (prop 1279) shown while the task is active and no job runs; last shown state
    pub obj_80: Option<i32>,
    pub shown_80: Option<bool>,
    /// WP-J15: save-variable handles +0xc8 / +0xd0 / +0xd8 / +0xe0 / +0xe8 (props 1295..1299, kind 0x16
    /// symbol paths; empty = null handle)
    pub save: [Vec<u16>; 5],
}

/// FUN_0022b490 request produced by the job code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JobReload {
    /// level-table row (FUN_00210990); None = FUN_00210b70 hub level when `JobMgr::hub_row` is unknown
    pub row: Option<i32>,
    pub spawn: i32,
    pub spawn_parent: i32,
    pub flags: u32,
    /// 0x30a = FUN_0022b220 transition, -1 = FUN_0020df80
    pub a: i32,
}

impl JobReload {
    /// FUN_0022b490: request record; flags | 2 when the row is the current level.
    pub fn level_request(&self, table: &[Descriptor], current_row: i32) -> Option<LevelRequest> {
        let row = self.row?;
        let desc = *table.get(usize::try_from(row).ok()?)?;
        let mut r = LevelRequest::new(desc);
        r.flags = self.flags | if row == current_row { 2 } else { 0 };
        r.spawn = self.spawn;
        r.spawn_parent = self.spawn_parent;
        Some(r)
    }
}

/// Job manager 0x2deb30 + the world's job slots.
#[derive(Clone, Debug, Default)]
pub struct JobMgr {
    pub jobs: Vec<Job>,
    pub tasks: Vec<Task>,
    /// world+0x3b0c (DAT_003e0f9c)
    pub cur: Option<usize>,
    /// world+0x3b14 (DAT_003e0fa4): restart task override
    pub restart_override: Option<usize>,
    /// DAT_003e0fa8 (0x162e38)
    pub completed: Option<usize>,
    /// FUN_0020e040(world)
    pub world_busy: bool,
    /// DAT_003e1110
    pub cur_row: i32,
    /// FUN_00210b70 row
    pub hub_row: Option<i32>,
    /// DAT_003e12c8
    pub flag_12c8: bool,
    /// DAT_003e1114
    pub f_1114: bool,
    /// DAT_002dd318 (0x211d38 retries)
    pub retries: i32,
    /// requests for the game (FUN_0022b490)
    pub reloads: Vec<JobReload>,
    /// unported / script side effects (address, arg)
    pub calls: Vec<(u32, i32)>,
    /// WP-J15: messages 0x1657d0 sends to the task itself through its vtable +0x1c (0x16 before /
    /// 0x15 after a state change): (task index, message), delivered by `vm_state::job_messages`
    pub task_msgs: Vec<(usize, u32)>,
    /// WP-EP1f: message 0x11 {event, argument} that 0x162bb8 / 0x165170 send to the job / task object
    /// (vt+0x1c, nargs 2): (world object index, event, argument object handle = index + 1, 0 = none),
    /// delivered by `vm_state::job_messages` (the job script's event handler, e.g. method 1069 = 0x162e38)
    pub ev_msgs: Vec<(usize, i32, u32)>,
    /// world+0x3b10 (DAT_003e0fa0) current task (WP-P9)
    pub cur_task: Option<usize>,
    /// DAT_002deb28 0x164df0 re-entry guard
    pub guard_deb28: bool,
    /// REPLACED save-variable writes 0x258200: (owner, handle field, code, value)
    pub vars: Vec<(VarOwner, u16, i32, i32)>,
    /// REPLACED 0x1620b0 object calls: (oid, parent oid, show = vtable +0x44 else +0x4c)
    pub toggles: Vec<(i32, i32, bool)>,
    /// REPLACED 0x162250 anim calls: (oid, 0x1f67e0 arg, anim, play = 0x1f0948 else 0x1f0660)
    pub anims: Vec<(i32, i32, i32, bool)>,
    /// WP-J14: DAT_002dded0 / DAT_002dded4 of the last `update`
    pub now: f32,
    pub dt: f32,
    /// REPLACED FUN_002121e8: (aseg oid, label) pairs still playing
    pub playing: Vec<(i32, i32)>,
    /// FUN_001a4e70(0x2e1e88) pending count
    pub queue_1a4e70: i32,
    /// FUN_001abd98(0x2e65e0, 0) autosave requests
    pub autosave: u32,
    /// DAT_002deb38 (0x163e88)
    pub deb38: i32,
}

/// WP-J15: a kind-0x16 prop (u16 n + n u16 symbols, FUN_0013e818) = save-variable path.
fn path_prop(props: &[Prop], f: i16) -> Vec<u16> {
    props.iter().rev().find(|p| p.field == f).map_or(Vec::new(), |p| match &p.value {
        PropValue::Scalar(b) if b.len() >= 2 => {
            let n = u16::from_le_bytes([b[0], b[1]]) as usize;
            b[2..].chunks_exact(2).take(n).map(|c| u16::from_le_bytes([c[0], c[1]])).collect()
        }
        _ => Vec::new(),
    })
}

fn scalar_i32(props: &[Prop], f: i16) -> Option<i32> {
    props.iter().rev().find(|p| p.field == f).and_then(|p| match &p.value {
        PropValue::Scalar(b) if b.len() >= 4 => Some(i32::from_le_bytes([b[0], b[1], b[2], b[3]])),
        PropValue::Scalar(b) if b.len() >= 2 => Some(i16::from_le_bytes([b[0], b[1]]) as i32),
        _ => None,
    })
}

impl JobMgr {
    /// Jobs and tasks of a loaded world (props of 0x1624a8 / 0x1635f8; ctor defaults -1).
    pub fn load(w: &World) -> Self {
        let mut m = JobMgr { cur_row: -1, ..Default::default() };
        let mut jp = Vec::new();
        let mut tp = Vec::new();
        for (i, o) in w.objects.iter().enumerate() {
            let Payload::Loaded(LoadData::B4(b)) = &o.payload else { continue };
            match &**b {
                B4Load::L1624a8 { props, .. } if o.class == CID_JOB => {
                    // ctor 0x162448: +0x9c 480, +0xa0 720, +0xac / +0xb0 / +0xb4 -1
                    m.jobs.push(Job { obj: i, oid: o.oid as i32, f_9c: 480.0, f_a0: 720.0, ac: -1, b0: -1, b4: -1, ..Default::default() });
                    jp.push(props.clone());
                }
                B4Load::L1635f8 { props, .. } if o.class == CID_TASK => {
                    m.tasks.push(Task { obj: i, oid: o.oid as i32, state: -1, spawn: -1, level: -1, c0: -1, f0: -1, f_5c: -1, ..Default::default() });
                    tp.push(props.clone());
                }
                _ => {}
            }
        }
        let task_of = |m: &JobMgr, oid: Option<i32>| oid.and_then(|o| m.tasks.iter().position(|t| t.oid == o));
        for (k, p) in jp.iter().enumerate() {
            m.jobs[k].start = task_of(&m, scalar_i32(p, 1243));
            m.jobs[k].end = task_of(&m, scalar_i32(p, 1244));
            // WP-P9: field bindings of class 132 (fieldbind_gen FIELD_MEMBER): 1242 -> +0x78
            m.jobs[k].f_78 = scalar_i32(p, 1242).unwrap_or(0);
            // WP-J14: 1259 -> +0xcc (object oid)
            m.jobs[k].cc = scalar_i32(p, 1259).filter(|&o| o != -1);
            // WP-J15: kind-0x16 fields (row +0x18 = member): 1262 +0xd0, 1263 +0xd8, 1260 +0xe0, 1261 +0xe8
            for (f, i) in [(1262, 1), (1263, 2), (1260, 3), (1261, 4)] {
                m.jobs[k].save[i] = path_prop(p, f);
            }
        }
        for (k, p) in tp.iter().enumerate() {
            let j = scalar_i32(p, 1264).and_then(|o| m.jobs.iter().position(|j| j.oid == o));
            let t = &mut m.tasks[k];
            t.job = j;
            t.spawn = scalar_i32(p, 1274).unwrap_or(-1);
            t.level = scalar_i32(p, 1275).unwrap_or(-1);
            t.c0 = scalar_i32(p, 1276).unwrap_or(-1);
            // WP-P9: 1265 -> +0x5c, 1266 -> +0x60, 1272 -> +0x70, 1273 -> +0x74, 1278 -> +0xc4 (oid)
            t.f_5c = scalar_i32(p, 1265).unwrap_or(-1);
            t.f_60 = scalar_i32(p, 1266).unwrap_or(0);
            t.f_70 = scalar_i32(p, 1272).unwrap_or(0);
            t.f_74 = scalar_i32(p, 1273).unwrap_or(0);
            t.c4 = scalar_i32(p, 1278);
            // WP-J14: 1271 -> +0x6c, 1285 -> +0x78 (float), 1279 -> +0x80 (object oid)
            t.f_6c = scalar_i32(p, 1271).unwrap_or(0);
            t.t_78 = scalar_i32(p, 1285).map(|b| f32::from_bits(b as u32)).unwrap_or(0.0);
            t.obj_80 = scalar_i32(p, 1279).filter(|&o| o != -1);
            // WP-J15: kind-0x16 fields 1295..1299 -> +0xc8 .. +0xe8
            for i in 0..5 {
                t.save[i] = path_prop(p, 1295 + i as i16);
            }
            if let Some(j) = j {
                m.jobs[j].tasks.push(k);
            }
        }
        m
    }

    pub fn job_by_oid(&self, oid: i32) -> Option<usize> {
        self.jobs.iter().position(|j| j.oid == oid)
    }

    /// 0x00165dc0
    pub fn FUN_00165dc0(&mut self, j: Option<usize>) {
        self.cur = j;
    }

    /// 0x001633e8
    pub fn FUN_001633e8(&self, j: usize) -> bool {
        self.cur == Some(j)
    }

    /// 0x00162670: job running.
    pub fn FUN_00162670(&self, j: usize) -> bool {
        let job = &self.jobs[j];
        job.f_50 == 0 && job.state != 2 && job.state != 3 && !self.world_busy
    }

    /// 0x001637e8: task running.
    pub fn FUN_001637e8(&self, t: usize) -> bool {
        let tk = &self.tasks[t];
        tk.f_50 == 0 && tk.f4 != 2 && tk.f4 != 3 && !self.world_busy
    }

    /// 0x00164030: task done (state >= 2; -1 = save variable, REPLACED: not done).
    pub fn FUN_00164030(&self, t: Option<usize>) -> bool {
        t.map(|t| self.tasks[t].state >= 2).unwrap_or(false)
    }

    /// 0x001619a0 (job half; 0x161a58 is p7 / the game, 0x154a90 unported).
    /// `arg` = the event object handle (world index + 1, 0 = none).
    pub fn FUN_001619a0(&mut self, ev: i32, arg: i32) {
        self.calls.push((0x00154a90, ev));
        for t in 0..self.tasks.len() {
            if self.tasks[t].job == self.cur {
                self.FUN_00165170(t, ev, arg);
            }
        }
        if let Some(j) = self.cur {
            self.FUN_00162bb8(j, ev, arg);
        }
    }

    /// 0x00165170
    pub fn FUN_00165170(&mut self, t: usize, ev: i32, arg: i32) {
        if self.FUN_001637e8(t) {
            self.calls.push((0x00165170, ev)); // msg 0x11 {ev, arg} to the task
            self.ev_msgs.push((self.tasks[t].obj, ev, arg as u32));
        }
    }

    /// 0x00162bb8
    pub fn FUN_00162bb8(&mut self, j: usize, ev: i32, arg: i32) {
        if !self.FUN_00162670(j) {
            if ev == EV_DIED && self.jobs[j].f_50 != 0 && !self.world_busy {
                self.FUN_00162df8(j);
            }
            return;
        }
        self.calls.push((0x00162bb8, ev)); // msg 0x11 {ev, arg} to the job
        self.ev_msgs.push((self.jobs[j].obj, ev, arg as u32));
        if self.jobs[j].fail_ev.contains(&ev) {
            self.FUN_00162dd0(j);
        }
        if self.jobs[j].done_ev.contains(&ev) {
            self.FUN_00162e38(j);
        }
    }

    /// 0x00162dd0: job failed -> event 0x1d.
    pub fn FUN_00162dd0(&mut self, j: usize) {
        self.FUN_001619a0(EV_FAILED, (self.jobs[j].obj + 1) as i32);
    }

    /// 0x00162df8: restart the job (pause menu 0x211d38, builtin 0x10ea98, death).
    pub fn FUN_00162df8(&mut self, j: usize) {
        match self.restart_override {
            Some(t) => self.FUN_00165538(Some(t)),
            None => self.FUN_00165538(self.jobs[j].start),
        }
    }

    /// 0x00162e38: job complete -> back to the hub at 0x189.
    pub fn FUN_00162e38(&mut self, j: usize) {
        self.completed = Some(j);
        if !self.world_busy && !self.f_1114 {
            self.reloads.push(JobReload { row: self.hub_row, spawn: SPAWN_DEFAULT, spawn_parent: -1, flags: 0, a: -1 });
        }
    }

    /// 0x00165538: restart at task `t` (oidPoCurrent).
    pub fn FUN_00165538(&mut self, t: Option<usize>) {
        // +0xf4 / +0xf8 of the task and +0xf0 / +0xf4 of its job are forced to 2 / 0 around the call (no
        // events fire from them) and restored at the end.
        let saved_t = t.map(|t| self.tasks[t].f4);
        let job = t.and_then(|t| self.tasks[t].job);
        let saved_j = job.map(|j| self.jobs[j].state);
        if let Some(t) = t {
            self.tasks[t].f4 = 2;
        }
        if let Some(j) = job {
            self.jobs[j].state = 2;
        }
        if job != self.cur {
            if let Some(c) = self.cur {
                let s = self.jobs[c].state;
                self.jobs[c].state = 2;
                self.FUN_00162ea0(c);
                self.jobs[c].state = s;
            }
            if let Some(j) = job {
                self.FUN_001628b8(j);
            }
        }
        if let Some(t) = t {
            self.calls.push((0x00164df0, t as i32)); // trace of the reset (tests/engine_p8 contract)
            self.FUN_00164df0(t, 1);
            self.FUN_00163c28(t);
        }
        let tk = t.map(|t| self.tasks[t].clone()).unwrap_or_default();
        if t.is_some() && tk.c0 != -1 {
            self.calls.push((0x001d8b20, tk.c0)); // message 0x15 tk.c0
        }
        let spawn = if t.is_some() && tk.spawn != -1 { tk.spawn } else { SPAWN_DEFAULT };
        let level = if t.is_some() { tk.level } else { -1 };
        let mut flags = 4;
        if self.flag_12c8 && (level == -1 || level == self.cur_row) {
            flags = 0x24;
        }
        self.reloads.push(JobReload { row: Self::FUN_00210990(level), spawn, spawn_parent: -1, flags, a: 0x30a });
        if let (Some(j), Some(s)) = (job, saved_j) {
            self.jobs[j].state = s;
        }
        if let (Some(t), Some(s)) = (t, saved_t) {
            self.tasks[t].f4 = s;
        }
    }

    /// 0x00210990: level-table row (-1 -> none = the current level in 0x22b490).
    pub fn FUN_00210990(row: i32) -> Option<i32> {
        (row != -1).then_some(row)
    }

    /// 0x001628b8: make `j` the current job.
    pub fn FUN_001628b8(&mut self, j: usize) {
        if !self.FUN_001633e8(j) {
            if let Some(c) = self.cur {
                self.FUN_00162908(c);
            }
            self.FUN_001629a0(j);
        }
    }

    /// 0x00162908
    pub fn FUN_00162908(&mut self, j: usize) {
        if self.FUN_001633e8(j) {
            self.FUN_00162a60(j);
        }
    }

    /// 0x001629a0: start (PARTIAL).
    pub fn FUN_001629a0(&mut self, j: usize) {
        self.FUN_00165dc0(Some(j));
        if self.FUN_00162670(j) {
            self.jobs[j].n_7c += 1;
            self.FUN_00163460(j);
            self.FUN_001619a0(EV_STARTED, (self.jobs[j].obj + 1) as i32);
            let (tg, an, f0) = (self.jobs[j].toggles.clone(), self.jobs[j].anims.clone(), self.jobs[j].state);
            self.FUN_001620b0(4, &tg);
            self.FUN_001623d0(4, &an, f0);
            self.calls.push((0x001629a0, 0x13)); // msg 0x13
        }
    }

    /// 0x00162a60: stop (PARTIAL).
    pub fn FUN_00162a60(&mut self, j: usize) {
        self.FUN_00165dc0(None);
        if let Some(t) = self.restart_override {
            if self.tasks[t].job == Some(j) {
                self.restart_override = None;
            }
        }
        if self.hub_row.is_some() && self.hub_row == Some(self.cur_row) {
            self.calls.push((0x00162a60, SPAWN_DEFAULT)); // DAT_003e1118 = 0x189, DAT_003e111c = -1
        }
        if self.FUN_00162670(j) {
            self.FUN_001619a0(EV_STOPPED, (self.jobs[j].obj + 1) as i32);
            let (tg, an, f0) = (self.jobs[j].toggles.clone(), self.jobs[j].anims.clone(), self.jobs[j].state);
            self.FUN_001620b0(5, &tg);
            self.FUN_001623d0(5, &an, f0);
            self.calls.push((0x00162a60, 0x14)); // msg 0x14
        }
    }

    /// 0x00162ea0: end.
    pub fn FUN_00162ea0(&mut self, j: usize) {
        self.FUN_00162940(j);
        if let Some(t) = self.jobs[j].start {
            self.FUN_00164df0(t, 1);
        }
        if let Some(c) = self.cur_task.filter(|&c| self.tasks[c].job == Some(j)) {
            self.FUN_00163ce0(c);
        }
        self.FUN_001619a0(EV_ENDED, (self.jobs[j].obj + 1) as i32);
        self.FUN_00162908(j);
    }

    /// 0x001649d0: pick the current job: start task done and end task not done.
    pub fn FUN_001649d0(&mut self) {
        let mut pick = None;
        for j in 0..self.jobs.len() {
            if self.FUN_00164030(self.jobs[j].start) && !self.FUN_00164030(self.jobs[j].end) {
                pick = Some(j);
            }
        }
        if pick != self.cur {
            if let Some(c) = self.cur {
                self.FUN_00162908(c);
            }
            if let Some(p) = pick {
                self.FUN_001628b8(p);
            }
        }
    }

    /// 0x00211d38 restart part (pause menu "restart job").
    pub fn restart_current(&mut self) {
        if let Some(c) = self.cur {
            self.retries += 1;
            self.FUN_00162df8(c);
        }
    }

    /// WP-ARCH1 step 4: replay recorded script side-effect calls through the live guest-memory
    /// store. Each `(addr, arg)` whose `addr` has a queue-port target is dispatched 1:1 via
    /// `QMem::call`; others are left in `self.calls` for the next pass (so adding a new queue port
    /// retroactively drains them). Returns the number dispatched this call.
    ///
    /// This is the ARCH1 bridge from "JobMgr records what the script wanted" to "JobMgr performs
    /// what the script wanted": once JobMgr has access to the store, the EP2+ job-handler blockers
    /// B1/B2/B3/B5 (user-property setup via `add-o`/`ensure-o`/`set-o`) can be driven by the real
    /// splice BIFs instead of staying as recorded no-ops.
    pub fn replay_calls_through(&mut self, mem: &mut crate::engine::classes::q::live::LiveMem) -> usize {
        use crate::engine::classes::q::live::{is_queue, trivial};
        use crate::engine::classes::q::q2mem::QMem;
        let mut keep = Vec::with_capacity(self.calls.len());
        let mut n = 0usize;
        for (addr, arg) in std::mem::take(&mut self.calls) {
            if is_queue(addr) || trivial(addr).is_some() {
                let _ = mem.call(addr, &[arg as u32], &[]);
                n += 1;
            } else {
                keep.push((addr, arg));
            }
        }
        self.calls = keep;
        n
    }
}
