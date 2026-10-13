//! WP-J14: per-frame job / task update and the job-complete path.
//!
//! Job (class 132) vtable slot 15 FUN_00163078 (update, after the LO base 0x1a5e28) drives the
//! "job complete" sequence of +0xc4 (0 idle, 1 complete label playing, 2 outro label, 3 aseg 0x379)
//! started by FUN_00162f10 (0x162cd0: the end task changed state); FUN_00163170 switches it,
//! FUN_00162f60 publishes the end (event 0x20, 0x162908, autosave 0x1abd98) and FUN_00162fd0 leads to
//! the next task (+0xc0 = first dependent of the end task with a +0xc4 object whose other prerequisites
//! are done -> FUN_00164640(task, 4)). Job slot 14 FUN_00162780 is the post-load (save variables,
//! the job's ASEGs). Task slot 15 FUN_00163b48: the +0x78 time limit (method 1284 0x165118 msg 0x17) and
//! the +0x80 object shown only while no job runs and the task is active.
//! FUN_001645d0 (prerequisites done or active with +0x6c) / FUN_00164640 (task state request) /
//! FUN_00163e88 (refused request) and the job half of the clock hook FUN_00161478.
//!
//! 1:1: 0x163078, 0x163170, 0x162f10, 0x162f60, 0x162fd0, 0x162780 (minus the save reads), 0x163b48,
//! 0x1645d0, 0x164640, 0x163e88, 0x165118, 0x162660, 0x161478 (job timer / difficulty part).
//! REPLACED: FUN_002121e8 (ASEG label playing) -> `JobMgr::playing` (aseg oid, label) pairs the host
//! keeps (none = finished at once); ASEG vt+0x18c / +0x194 / FUN_00213b00, FUN_001a6ca8, object flag
//! 0x100 / 0x102 writes, FUN_002283d0, FUN_0020efc0 / FUN_0020f030 (world busy count 2), FUN_00154d88
//! (difficulty), FUN_001a4e70 / FUN_001a4e80 (queue 0x2e1e88) -> `JobMgr::calls` / `JobMgr::queue_1a4e70`;
//! FUN_001abd98(0x2e65e0, 0) autosave -> `JobMgr::autosave` (the game writes the slot file); the +0x80
//! object's vt+0x44 / +0x4c (every frame in the original, idempotent) -> one `JobMgr::toggles` record per
//! change; save-variable reads of 0x162780 -> `JobSaved`.
//! ASSUMPTION: 0x163b48's time-limit test decompiles as `e < lim && lim + dt <= e` (never true for dt >= 0);
//! ported as the crossing `e - dt < lim && lim <= e` (fires once when the limit is reached).
#![allow(non_snake_case)]

use super::JobMgr;

/// FUN_001a4e70 / 0x1a4e80 queue, FUN_002121e8 label of the job-complete aseg 0x379.
pub const LABEL_379: i32 = 0x30f;
pub const ASEG_JOB_DONE: i32 = 0x379;

/// Save-variable reads of FUN_00162780 (+0xd0 -> +0x7c, +0xe0 -> +0xa4, +0xe8 via 0x1d7e80 -> +0xa8).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct JobSaved {
    pub n_7c: i32,
    pub a4: f32,
    pub a8: i32,
}

impl JobMgr {
    /// FUN_002121e8(aseg, label): label still playing (label -1 / no aseg -> 0).
    pub fn FUN_002121e8(&self, aseg: Option<i32>, label: i32) -> bool {
        match aseg {
            Some(a) if label != -1 => self.playing.contains(&(a, label)),
            _ => false,
        }
    }

    /// 0x00162660 (method 1258 thunk 0x10eda8): complete aseg oid, start / outro labels.
    pub fn FUN_00162660(&mut self, j: usize, aseg: i32, b0: i32, b4: i32) {
        let job = &mut self.jobs[j];
        job.ac = aseg;
        job.b0 = b0;
        job.b4 = b4;
    }

    /// 0x00162780: job post-load. `find` = FUN_0015e938(world, 0x405, oid, 0) (aseg by oid).
    pub fn FUN_00162780(&mut self, j: usize, saved: JobSaved, find: &dyn Fn(i32) -> Option<i32>) {
        if !self.FUN_001633e8(j) && self.FUN_00162670(j) {
            let (tg, an) = (self.jobs[j].toggles.clone(), self.jobs[j].anims.clone());
            self.FUN_001620b0(5, &tg);
            self.FUN_001623d0(5, &an, 1);
        }
        let job = &mut self.jobs[j];
        job.n_7c = saved.n_7c;
        job.a4 = saved.a4;
        job.a8 = saved.a8;
        if job.ac != -1 {
            job.aseg_b8 = find(job.ac);
        }
        job.aseg_bc = find(ASEG_JOB_DONE);
    }

    /// 0x00163078: job update (slot 15).
    pub fn FUN_00163078(&mut self, j: usize) {
        let job = &self.jobs[j];
        match job.c4 {
            2 => {
                if !self.FUN_002121e8(job.aseg_b8, job.b4) {
                    self.FUN_00162fd0(j);
                    self.FUN_00163170(j, 0);
                }
            }
            1 => {
                if !self.FUN_002121e8(job.aseg_b8, job.b0) {
                    self.FUN_00162f60(j);
                    let s = if self.jobs[j].c0.is_none() && self.jobs[j].cc.is_none() { 2 } else { 3 };
                    self.FUN_00163170(j, s);
                }
            }
            3 => {
                if !self.FUN_002121e8(job.aseg_bc, LABEL_379) {
                    self.FUN_00162fd0(j);
                }
            }
            _ => {}
        }
    }

    /// 0x00162f10: job complete.
    pub fn FUN_00162f10(&mut self, j: usize) {
        if self.jobs[j].aseg_b8.is_none() {
            self.FUN_00162f60(j);
            self.FUN_00162fd0(j);
        } else {
            self.FUN_00163170(j, 1);
        }
    }

    /// 0x00162f60
    pub fn FUN_00162f60(&mut self, j: usize) {
        if self.jobs[j].f_78 == 0 {
            if let Some(e) = self.jobs[j].end {
                self.FUN_00164df0(e, 3);
            }
        }
        self.FUN_001619a0(0x20, j as i32);
        self.FUN_00162908(j);
        if self.jobs[j].f_78 == 0 {
            self.autosave += 1; // FUN_001abd98(0x2e65e0, 0)
        }
    }

    /// 0x00162fd0
    pub fn FUN_00162fd0(&mut self, j: usize) {
        if self.queue_1a4e70 != 0 {
            self.queue_1a4e70 -= 1; // FUN_001a4e80(0x2e1e88)
            self.calls.push((0x001a4e80, j as i32));
            return;
        }
        match self.jobs[j].c0 {
            None => {
                if let Some(o) = self.jobs[j].cc {
                    self.calls.push((0x002283d0, o)); // flag 0x100 + FUN_002283d0(obj)
                }
            }
            Some(t) => self.FUN_00164640(Some(t), 4, 0),
        }
    }

    /// 0x00163170: job-complete state.
    pub fn FUN_00163170(&mut self, j: usize, s: i32) {
        if self.jobs[j].c4 == s {
            return;
        }
        if self.jobs[j].c4 == 0 {
            let end = self.jobs[j].end;
            let mut c0 = end.and_then(|e| self.tasks[e].deps.iter().copied().find(|&d| self.tasks[d].c4.is_some()));
            if let Some(c) = c0 {
                for p in self.tasks[c].prereq.clone() {
                    if Some(p) != end && !self.FUN_00164030(Some(p)) {
                        c0 = None;
                        break;
                    }
                }
            }
            self.jobs[j].c0 = c0;
            self.calls.push((0x001a6ca8, 1));
            self.jobs[j].flags_40 |= 0x102;
            self.calls.push((0x0020efc0, 2));
        }
        self.jobs[j].c4 = s;
        self.jobs[j].t_c8 = self.now;
        let (b8, bc) = (self.jobs[j].aseg_b8, self.jobs[j].aseg_bc);
        match s {
            1 => {
                self.calls.push((0x0018c000, b8.unwrap_or(0))); // vt+0x18c(-1)
                self.calls.push((0x00213b00, self.jobs[j].b0));
            }
            0 => {
                for a in [bc, b8].into_iter().flatten() {
                    self.calls.push((0x00019400, a)); // vt+0x194
                }
                self.calls.push((0x001a6ca8, 2));
                self.jobs[j].flags_40 &= !0x102;
                self.calls.push((0x0020f030, 2));
            }
            2 => self.calls.push((0x00213b00, self.jobs[j].b4)),
            3 => self.calls.push((0x0018c000, bc.unwrap_or(0))),
            _ => {}
        }
    }

    /// 0x001645d0: prerequisites done (state > 1, or 1 with +0x6c).
    pub fn FUN_001645d0(&self, t: usize) -> bool {
        self.tasks[t].prereq.iter().all(|&p| {
            let s = self.tasks[p].state;
            s > 1 || (s == 1 && self.tasks[p].f_6c != 0)
        })
    }

    /// 0x00163e88: refused state request (event 0x27, msg 0x12 to the task and its running job).
    pub fn FUN_00163e88(&mut self, t: usize, s: i32) {
        if self.FUN_001637e8(t) {
            self.deb38 = s;
            self.FUN_001619a0(0x27, t as i32);
            self.calls.push((0x00163e88, 0x12));
        }
        if let Some(j) = self.tasks[t].job {
            if self.FUN_00162670(j) {
                self.calls.push((0x00163e88, 0x12));
            }
        }
    }

    /// 0x00164640(task, s, f8): task state request under task +0xf4 / +0xf8 and job +0xf0 / +0xf4.
    pub fn FUN_00164640(&mut self, t: Option<usize>, s: i32, f8: i32) {
        let Some(t) = t else { return };
        let (f4, old_f8) = (self.tasks[t].f4, self.tasks[t].f8);
        self.tasks[t].f8 = f8;
        let j = self.tasks[t].job;
        let saved_job = j.map(|j| (self.jobs[j].state, self.jobs[j].f4));
        if let Some(j) = j {
            self.jobs[j].state = f4;
            self.jobs[j].f4 = f8;
        }
        let cur = self.tasks[t].state;
        let s = if s == -1 { 4 } else { s };
        if cur != -1 {
            if s == 4 {
                let mut skip = false;
                if cur == 0 {
                    if !self.FUN_001645d0(t) {
                        self.FUN_00163e88(t, 4);
                        skip = true;
                    } else {
                        self.FUN_00164df0(t, 1);
                    }
                } else if cur == 3 {
                    skip = true;
                }
                if !skip {
                    self.FUN_00163c28(t);
                }
            } else if (0..5).contains(&s) {
                let mut ok = s - cur == 1;
                let mut skip = false;
                if cur == 1 {
                    if s == 3 {
                        ok = true;
                    }
                } else if cur < 2 {
                    if cur == 0 && s == 1 {
                        ok = self.FUN_001645d0(t);
                    }
                } else if cur == 3 {
                    skip = true;
                }
                if !skip {
                    if ok {
                        self.FUN_00164df0(t, s);
                    } else {
                        self.FUN_00163e88(t, s);
                    }
                }
            } else if s == 5 {
                self.FUN_00163ce0(t);
            }
        }
        if let (Some(j), Some((st, f4j))) = (j, saved_job) {
            self.jobs[j].state = st;
            self.jobs[j].f4 = f4j;
        }
        self.tasks[t].f4 = f4;
        self.tasks[t].f8 = old_f8;
    }

    /// 0x00165118 (method 1284): msg 0x17 {arg} to a running task.
    pub fn FUN_00165118(&mut self, t: usize, arg: i32) {
        if self.FUN_001637e8(t) {
            self.calls.push((0x00165118, arg));
        }
    }

    /// 0x00163b48: task update (slot 15).
    pub fn FUN_00163b48(&mut self, t: usize) {
        let tk = &self.tasks[t];
        if tk.state == 1 {
            let lim = tk.t_78;
            let e = self.now - tk.t_58;
            if 0.0 < lim && e - self.dt < lim && lim <= e {
                self.FUN_00165118(t, 0);
            }
        }
        let tk = &self.tasks[t];
        if let Some(o) = tk.obj_80 {
            let show = self.cur.is_none() && tk.state == 1;
            if self.tasks[t].shown_80 != Some(show) {
                self.tasks[t].shown_80 = Some(show);
                self.toggles.push((o, -1, show));
            }
        }
    }

    /// Per-frame update of every job and task (the world's slot-15 pass over classes 132 / 133).
    pub fn update(&mut self, now: f32, dt: f32) {
        self.now = now;
        self.dt = dt;
        for j in 0..self.jobs.len() {
            self.FUN_00163078(j);
        }
        for t in 0..self.tasks.len() {
            self.FUN_00163b48(t);
        }
    }

    /// 0x00161478 (job part): the current job's +0xa4 timer; past +0x9c the difficulty ramps
    /// (FUN_00154d88(dt / (+0xa0 - +0x9c)), REPLACED -> `calls`). `level_kind` = DAT_002e0c20 (as float);
    /// the caller runs it only while the "dt" save variable is bound (DAT_003e0fac != 0).
    pub fn FUN_00161478(&mut self, dt: f32, level_kind: f32) {
        let Some(j) = self.cur else { return };
        if !(level_kind < 10.0) {
            return;
        }
        let job = &mut self.jobs[j];
        job.a4 += dt;
        if job.a4 > job.f_9c {
            let f = dt / (job.f_a0 - job.f_9c);
            self.calls.push((0x00154d88, f.to_bits() as i32));
        }
    }
}
