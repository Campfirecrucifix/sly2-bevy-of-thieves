//! WP-P9: the rest of the job / task core: task state machine 0x164df0 (0x164ce8 prerequisite closure,
//! 0x165a60 / 0x1657d0 / 0x165b40 dependents), current-task switch 0x163c28 (0x163d18 enter / 0x163ce0 ->
//! 0x163f48 leave), script-variable publishes 0x163460 / 0x1643b0, job reset 0x162940, object toggles 0x1620b0
//! (0x161fc8 / 0x161ff0) and anim records 0x1623d0 / 0x162250, 0x162cd0 / 0x162d50 job hooks of a task,
//! 0x161660 completion percentage.
//!
//! 1:1: 0x161fc8, 0x161ff0, 0x1620b0 (DAT_002a5220 = {4, 5}, DAT_002a5228 = {0, 1, 2, 3} read from the ELF),
//! 0x1623d0, 0x162250 (gates), 0x163400, 0x163460, 0x1643b0, 0x162940, 0x164018, 0x163ce0, 0x163f48,
//! 0x163d18, 0x163c28, 0x164ce8, 0x164df0, 0x165a60, 0x1657d0, 0x165b40, 0x162cd0, 0x162d50, 0x165de8,
//! 0x165e10, 0x161660.
//! REPLACED: save-variable writes 0x258200(+0xc8..+0xe8 handle, {code, value}) -> `JobMgr::vars`; script
//! messages vt+0x1c / vt+0x24 and the music cues 0x1da578 / 0x1da4b8 -> `JobMgr::calls`; object lookups
//! 0x15e938 / 0x15e650 by oid + their vtable +0x44 / +0x4c calls -> `JobMgr::toggles`; 0x1f0948 / 0x1f0660
//! (anim play / stop of an is-a 0x30 object) -> `JobMgr::anims`; 0x11a2e8 / 0x11a220 active-task list
//! (DAT_003e0f90) -> `Task::state == 1`; +0x58 = DAT_002dded0 (frame time) not kept.
//! The record lists (+0x8c / +0xa8 toggles, +0x94 / +0xb0 anims), the prerequisite / dependent task lists
//! (+0x94 / +0xa0 of a task) and the save-variable handles come from props not mapped yet: empty from the
//! level, so those loops are no-ops on real data.
//! UNPORTED (recorded in `calls`): 0x162f10 job complete (0x162f60 / 0x162fd0 / 0x163170, the slot-15
//! 0x163078 state machine), 0x1abd98(0x2e65e0) (autosave request), 0x143380, 0x2283d0.
#![allow(non_snake_case)]

use super::JobMgr;

/// FUN_001620b0 record (0x10 bytes): object `oid` under `parent` (-1 = any) shown in the states of `mask`,
/// only on level row `row` (-1 = any).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ToggleRec {
    pub oid: i32,
    pub parent: i32,
    pub mask: u32,
    pub row: i32,
}

/// FUN_00162250 record (0x18 bytes).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AnimRec {
    /// [0] state (-1 = any; then the anim is 0x612 + state)
    pub state: i32,
    /// [1] object oid
    pub oid: i32,
    /// [2] FUN_001f67e0 argument
    pub sel: i32,
    /// [3] anim id
    pub anim: i32,
    /// [4] only when param_4 != 0
    pub gated: i32,
    /// [5] level row (-1 = any)
    pub row: i32,
}

/// Save-variable owners of `JobMgr::vars`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VarOwner {
    Job(usize),
    Task(usize),
}

/// 0x00161fc8
pub fn FUN_00161fc8(s: i32) -> u32 {
    if s != -1 {
        1u32.wrapping_shl(s as u32)
    } else {
        0
    }
}

/// 0x00161ff0
pub fn FUN_00161ff0(states: &[i32]) -> u32 {
    states.iter().fold(0, |m, &s| m | FUN_00161fc8(s))
}

/// DAT_002a5220 / DAT_002a5228 (ELF .data)
pub const DAT_002A5220: [i32; 2] = [4, 5];
pub const DAT_002A5228: [i32; 4] = [0, 1, 2, 3];

impl JobMgr {
    /// 0x00165de8: world+0x3b10 current task (DAT_002deb44 = its oid)
    pub fn FUN_00165de8(&mut self, t: Option<usize>) {
        self.cur_task = t;
    }

    /// 0x00165e10: world+0x3b14 restart override
    pub fn FUN_00165e10(&mut self, t: Option<usize>) {
        self.restart_override = t;
    }

    /// 0x00164018
    pub fn FUN_00164018(&self, t: usize) -> bool {
        self.cur_task == Some(t)
    }

    /// 0x00163400: the job's end task is done.
    pub fn FUN_00163400(&self, j: usize) -> bool {
        self.jobs[j].end.is_some() && self.FUN_00164030(self.jobs[j].end)
    }

    /// 0x00163460: publish the job's save variables (+0xd0 count, +0xd8 complete, +0xe0 +0xa4).
    pub fn FUN_00163460(&mut self, j: usize) {
        let done = self.FUN_00163400(j);
        let o = VarOwner::Job(j);
        self.vars.push((o, 0xd0, 0x15, self.jobs[j].n_7c));
        self.vars.push((o, 0xd8, 0x17, done as i32));
        self.vars.push((o, 0xe0, 0x16, self.jobs[j].a4.to_bits() as i32)); // WP-J14: tag 0x16 float bits
    }

    /// 0x001643b0: publish the task's save variables, then its job's.
    pub fn FUN_001643b0(&mut self, t: usize) {
        let tk = &self.tasks[t];
        if tk.f4 == 1 || tk.state == -1 {
            return;
        }
        let (n64, n68, st, job) = (tk.n_64, tk.n_68, tk.state, tk.job);
        let done = self.FUN_00164030(Some(t));
        let o = VarOwner::Task(t);
        self.vars.push((o, 0xc8, 0x15, n64));
        self.vars.push((o, 0xd0, 0x15, n68));
        self.vars.push((o, 0xd8, 0x17, done as i32));
        self.vars.push((o, 0xe0, 0x15, st));
        self.vars.push((o, 0xe8, 0x17, 0));
        if let Some(j) = job {
            self.FUN_00163460(j);
        }
    }

    /// 0x00162940: reset the job's tasks' +0x64 counters and its +0x7c start counter.
    pub fn FUN_00162940(&mut self, j: usize) {
        for t in self.jobs[j].tasks.clone() {
            self.tasks[t].n_64 = 0;
            self.FUN_001643b0(t);
        }
        self.jobs[j].n_7c = 0;
        self.FUN_00163460(j);
    }

    /// 0x001620b0: show / hide the records' objects for `state` (vtable +0x44 show / +0x4c hide).
    pub fn FUN_001620b0(&mut self, state: i32, recs: &[ToggleRec]) {
        let bit = FUN_00161fc8(state);
        let group = if (state as u32).wrapping_sub(4) < 2 { FUN_00161ff0(&DAT_002A5220) } else { FUN_00161ff0(&DAT_002A5228) };
        for r in recs {
            if r.mask & group == 0 || (r.row != -1 && r.row != self.cur_row) {
                continue;
            }
            // 0x15e938(world, 5, parent) then (4 under it | 5 anywhere, oid): object lookup REPLACED
            self.toggles.push((r.oid, r.parent, r.mask & bit != 0));
        }
    }

    /// 0x001623d0 (0x00162250 per record)
    pub fn FUN_001623d0(&mut self, state: i32, recs: &[AnimRec], p5: i32) {
        for r in recs {
            if r.state != -1 && r.state != state {
                continue;
            }
            if r.gated != 0 && p5 == 0 {
                continue;
            }
            if r.row != -1 && r.row != self.cur_row {
                continue;
            }
            let mut anim = r.anim;
            if r.state == -1 {
                anim = match state {
                    0 => 0x612,
                    1 => 0x613,
                    2 => 0x614,
                    3 => 0x615,
                    _ => anim,
                };
            }
            // 0x15e650 lookup + is-a 0x30 + 0x1f67e0(sel) REPLACED: the game side resolves the object
            if anim != -1 {
                self.anims.push((r.oid, r.sel, anim, p5 == 1 || p5 == 4));
            }
        }
    }

    /// 0x00163f48: leave the current task.
    pub fn FUN_00163f48(&mut self, t: usize) {
        self.FUN_00165de8(None);
        if self.FUN_001637e8(t) {
            self.FUN_001619a0(0x24, t as i32);
            let (tg, an, f4) = (self.tasks[t].toggles.clone(), self.tasks[t].anims.clone(), self.tasks[t].f4);
            self.FUN_001620b0(5, &tg);
            self.FUN_001623d0(5, &an, f4);
            if self.tasks[t].f_5c != -1 && self.tasks[t].f_60 != 0 {
                self.calls.push((0x001da578, t as i32));
            }
            self.calls.push((0x00163f48, 0x14)); // vt+0x1c msg 0x14
        }
    }

    /// 0x00163ce0
    pub fn FUN_00163ce0(&mut self, t: usize) {
        if self.FUN_00164018(t) {
            self.FUN_00163f48(t);
        }
    }

    /// 0x00163d18: enter task `t` (make it current).
    pub fn FUN_00163d18(&mut self, t: usize) {
        self.FUN_00165de8(Some(t));
        if self.tasks[t].spawn != -1 {
            self.FUN_00165e10(Some(t));
        }
        if let Some(j) = self.tasks[t].job {
            if !self.FUN_001633e8(j) {
                self.FUN_00162d50(j);
            }
        }
        if self.FUN_001637e8(t) {
            self.tasks[t].n_64 += 1;
            self.FUN_001643b0(t);
            self.FUN_001619a0(0x23, t as i32);
            let (tg, an, f4) = (self.tasks[t].toggles.clone(), self.tasks[t].anims.clone(), self.tasks[t].f4);
            self.FUN_001620b0(4, &tg);
            self.FUN_001623d0(4, &an, f4);
            if self.tasks[t].f_5c != -1 && self.tasks[t].f_60 != 0 {
                self.calls.push((0x001da4b8, t as i32));
            }
            self.calls.push((0x00163d18, 0x13)); // vt+0x1c msg 0x13
            if let Some(o) = self.tasks[t].c4 {
                self.calls.push((0x002283d0, o)); // +0x40 flag 0x100 then 0x2283d0
            }
        }
    }

    /// 0x00163c28: switch the current task to `t` (the old one leaves under `t`'s +0xf4 / +0xf8).
    pub fn FUN_00163c28(&mut self, t: usize) {
        if self.FUN_00164018(t) {
            return;
        }
        if let Some(c) = self.cur_task {
            let (s4, s8) = (self.tasks[c].f4, self.tasks[c].f8);
            self.tasks[c].f4 = self.tasks[t].f4;
            self.tasks[c].f8 = self.tasks[t].f8;
            self.FUN_00163ce0(c);
            self.tasks[c].f4 = s4;
            self.tasks[c].f8 = s8;
        }
        self.FUN_00163d18(t);
    }

    /// 0x00162d50: a task of a job that is not current started: make the job current.
    pub fn FUN_00162d50(&mut self, j: usize) {
        if !self.FUN_001633e8(j) {
            self.FUN_001619a0(0x1c, j as i32);
            self.calls.push((0x00143380, j as i32)); // 0x1c4840 player is-a 0x34 -> 0x143380
            self.FUN_001628b8(j);
        }
    }

    /// 0x00162cd0: job hooks of task `t` changing state.
    pub fn FUN_00162cd0(&mut self, j: usize, t: usize) {
        if !self.FUN_00162670(j) {
            return;
        }
        if self.jobs[j].start == Some(t) {
            self.FUN_00162d50(j);
        }
        if self.jobs[j].end == Some(t) {
            self.FUN_00162f10(j); // WP-J14
        }
        if self.tasks[t].spawn != -1 {
            self.FUN_00165e10(Some(t));
        }
    }

    /// 0x00164ce8: collect `t` and its prerequisites below state `s` (prerequisites of another, inactive
    /// job become 3).
    pub fn FUN_00164ce8(&mut self, t: usize, s: i32, out: &mut Vec<usize>) {
        for p in self.tasks[t].prereq.clone() {
            let mut sp = s;
            if s == 2 {
                match self.tasks[p].job {
                    None => sp = 3,
                    Some(pj) if Some(pj) != self.tasks[t].job && self.jobs[pj].f_78 == 0 => sp = 3,
                    _ => {}
                }
            }
            self.FUN_00164ce8(p, sp, out);
        }
        if self.tasks[t].f0 == -1 && self.tasks[t].state < s {
            out.push(t);
            self.tasks[t].f0 = s;
        }
    }

    /// 0x00164df0: set task `t` to state `s` (DAT_002deb28 = re-entry guard).
    pub fn FUN_00164df0(&mut self, t: usize, s: i32) {
        let old = self.tasks[t].state;
        if s == old || self.guard_deb28 {
            return;
        }
        self.guard_deb28 = true;
        let close = if s < 3 { if s > 0 { 2 } else { -1 } } else if s == 3 { 3 } else { -1 };
        if close != -1 {
            let mut list = Vec::new();
            self.tasks[t].f0 = s;
            self.FUN_00164ce8(t, close, &mut list);
            for &p in &list {
                let (s4, s8) = (self.tasks[p].f4, self.tasks[p].f8);
                self.tasks[p].f4 = self.tasks[t].f4;
                self.tasks[p].f8 = self.tasks[t].f8;
                let ps = self.tasks[p].f0;
                self.FUN_00165a60(p, ps);
                self.tasks[p].f4 = s4;
                self.tasks[p].f8 = s8;
            }
            for &p in &list {
                self.tasks[p].f0 = -1;
            }
            self.tasks[t].f0 = -1;
        }
        self.FUN_00165a60(t, s);
        self.guard_deb28 = false;
        // DAT_003e100c[0x210ae0()] save variable <- 0x161660 percentage (REPLACED: recorded)
        let pct = self.FUN_00161660();
        self.calls.push((0x00161660, pct));
        if self.FUN_001637e8(t) && old != -1 && old < 2 && self.tasks[t].state > 1 {
            if let Some(j) = self.tasks[t].job {
                self.FUN_00162cd0(j, t);
                if self.tasks[t].job.is_some() {
                    return;
                }
            }
            if !self.tasks[t].prereq.is_empty() && self.tasks[t].deps.iter().any(|&d| self.tasks[d].job.is_none()) {
                return;
            }
            self.calls.push((0x001abd98, t as i32));
        }
    }

    /// 0x00165a60: set the state, then re-evaluate the dependents.
    pub fn FUN_00165a60(&mut self, t: usize, s: i32) {
        if self.tasks[t].state == s {
            return;
        }
        self.FUN_001657d0(t, s);
        let st = self.tasks[t].state;
        for d in self.tasks[t].deps.clone() {
            let (s4, s8) = (self.tasks[d].f4, self.tasks[d].f8);
            self.tasks[d].f4 = self.tasks[t].f4;
            self.tasks[d].f8 = self.tasks[t].f8;
            self.FUN_00165b40(d, st);
            self.tasks[d].f4 = s4;
            self.tasks[d].f8 = s8;
        }
    }

    /// 0x00165b40: dependent `t` of a task now in state `s`.
    pub fn FUN_00165b40(&mut self, t: usize, s: i32) {
        if self.tasks[t].f0 != -1 {
            return;
        }
        let mut n = self.tasks[t].state;
        if s >= 0 {
            if s < 2 {
                n = 0;
            } else if s < 4 {
                if self.tasks[t].prereq.iter().all(|&p| self.tasks[p].state >= 2) {
                    if n == 0 {
                        n = 1;
                    } else if n == 3 {
                        n = s;
                    }
                } else {
                    n = 0;
                }
            }
        }
        self.FUN_00165a60(t, n);
    }

    /// 0x001657d0: the state change itself.
    pub fn FUN_001657d0(&mut self, t: usize, s: i32) {
        if self.tasks[t].state == s {
            return;
        }
        if self.FUN_001637e8(t) {
            self.FUN_001619a0(0x26, t as i32);
            self.calls.push((0x001657d0, 0x16)); // vt+0x1c msg 0x16
            self.task_msgs.push((t, 0x16));
        }
        if self.FUN_001637e8(t) {
            let (tg, an, f4) = (self.tasks[t].toggles.clone(), self.tasks[t].anims.clone(), self.tasks[t].f4);
            self.FUN_001620b0(s, &tg);
            self.FUN_001623d0(s, &an, f4);
        }
        let tk = &mut self.tasks[t];
        match s {
            0 if tk.f4 == 4 => {
                tk.n_64 = 0;
                tk.n_68 = 0;
                if let Some(j) = tk.job {
                    if self.jobs[j].start == Some(t) {
                        self.jobs[j].n_7c = 0;
                    }
                }
            }
            2 if tk.f4 == 0 => tk.n_68 += 1,
            _ => {}
        }
        self.tasks[t].state = s;
        self.tasks[t].t_58 = self.now; // WP-J14: +0x58 = DAT_002dded0
        self.FUN_001643b0(t);
        if self.FUN_001637e8(t) {
            self.FUN_001619a0(0x25, t as i32);
            self.calls.push((0x001657d0, 0x15)); // vt+0x1c msg 0x15
            self.task_msgs.push((t, 0x15));
            self.calls.push((0x001657d0, 0x11)); // vt+0x24 msg 0x11 (self)
        }
        match self.tasks[t].f4 {
            0 | 2 | 4 => match self.tasks[t].state {
                1 => {
                    if self.tasks[t].f_70 != 0 {
                        self.FUN_00163c28(t);
                    }
                }
                0 | 3 => self.FUN_00163ce0(t),
                _ => {}
            },
            1 => {
                if self.tasks[t].state == 1 && self.tasks[t].f_74 != 0 {
                    self.calls.push((0x002deb44, self.tasks[t].oid)); // DAT_002deb44 = oid
                }
            }
            _ => {}
        }
    }

    /// 0x00161660: completion percentage over tasks with dependents (0, 1..99, 100).
    pub fn FUN_00161660(&self) -> i32 {
        let with = self.tasks.iter().filter(|t| !t.deps.is_empty());
        let (n, done) = with.fold((0, 0), |(n, d), t| (n + 1, d + (t.state == 3) as i32));
        if done == 0 {
            0
        } else if done == n {
            100
        } else {
            ((done * 100) / n).clamp(1, 99)
        }
    }
}
