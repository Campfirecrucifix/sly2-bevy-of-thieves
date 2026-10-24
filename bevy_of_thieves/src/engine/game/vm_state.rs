//! WP-J4: persistent Splice VM state across frames + the per-frame script tick.
//!
//! In the original the script state lives in the world (SW 0x3dd490): the world frame (level script
//! blocks, FUN_001ffa70), the scheduled-callback list world+0x39f0 (pool world+0x39fc, nodes 0x14 B
//! {+0 time, +4 object, +8 call list, +0xc id, +0x10 next}, sorted by time) and the id counter
//! DAT_003e11fc. It survives every frame and is rebuilt only when a level is loaded (new world).
//! `SpliceState` keeps exactly that; `tick_world_update` runs FUN_00149c80 at the positions
//! FUN_00226550 (main-loop step `WorldUpdate`) calls it.
//!
//! Replaced: FUN_00128148 (end of FUN_00226550) only copies six CRef/pair/frame pool counters into
//! their "previous frame" slots (DAT_002dd280.. -> +8) for the allocator stats; the port has no pools.

use super::methods::{self, get_o_method, MArg};
use super::splice_host::{LevelSpliceHost, BASIC_137_GLOBAL, BASIC_EMITTER, BASIC_PLAYER_GLOBAL, BASIC_WORLD_GLOBAL};
use crate::engine::camera::cm::{CmClock, FUN_0014d8c0, FUN_0014f750};
use crate::engine::camera::sound::{CmEmitter, FUN_001fb018, FUN_001fb060, FUN_001fb098};
use crate::engine::camera::Cm;
use crate::engine::classes::ClassHost;
use crate::engine::disc::descriptor::LevelTable;
use crate::engine::splice::eval::Vm;
use crate::engine::splice::frame::FrameRef;
use crate::engine::splice::value::{list_from, list_items, Method, Ref, TAGK_QUOTE};
use crate::engine::splice::{SpliceError, SpliceHost};
use crate::engine::world::World;

/// Class ids of the engine globals scripts get as Basic refs (class info +4).
/// WIRE-EP6: SW field 941 (symid 859), getter FUN_0020f0b0 (world +0x3d78 > 0)
pub const FIELD_SW_3D78: u16 = 941;
/// WIRE-EP6: JT field 759 (symid 742), getter FUN_00144068
pub const FIELD_JT_BUSY: u16 = 759;
/// WIRE-EP7: job (class 132) field 1252 = FUN_001633e8 (is the current job)
pub const FIELD_JOB_CURRENT: u16 = 1252;
/// POOL1 R73: symid of task field 1268 (FUN_00164030, task done)
pub const FIELD_TASK_DONE_SYM: u32 = 1080;
pub const CLASS_SW: u16 = 1; // 0x3dd490 (size 0x4790)
pub const CLASS_CM: u16 = 135; // g_cm 0x2ddf40 (binds 0x1026e0 / 0x102768)
/// POOL1 R52: SMA run-time records (k25 handles)
pub const CLASS_SMA: u16 = 136;
pub const CLASS_EMITTER: u16 = 151; // FUN_001fbb90 emitters (binds 0x101e18 / 0x101e90 / 0x101f08)
/// WP-J8: BIF 118 global 0x3dd040 (song stack +0x200 / +0x204 / +0x208, the level block of FUN_001fd0e8)
pub const CLASS_152: u16 = 152;
/// WP-J8: BIF 137 global 0x3da0c0
pub const CLASS_154: u16 = 154;

/// One world+0x39f0 node.
#[derive(Clone, Debug)]
pub struct Callback {
    /// +0 clock time when due
    pub t: f32,
    /// +4 object (0 = world: the list is evaluated through a fresh frame)
    pub object: u32,
    /// +8 the call list `(proc 'arg ...)`
    pub call: Ref,
    /// +0xc id (-1 for object callbacks and once dispatch started)
    pub id: i32,
}

/// The scheduled-callback list (world+0x39f0) + DAT_003e11fc.
#[derive(Clone, Debug, Default)]
pub struct Schedule {
    pub list: Vec<Callback>,
    /// DAT_003e11fc
    pub last_id: i32,
    /// WP-J5: world-owned object state reached by get-o/set-o, BIF 107 and the world+0x3ca8 list
    pub objs: super::fields::ObjectFields,
    /// WP-J7: the class runtime lent to the script host (method rows / deferred nodes on class objects)
    pub cls: methods::cls::ClassLink,
    /// WP-J9: read-only view of the job manager (`jobs::JobMgr`, copied by `app::lend_classes`)
    pub jobs: JobView,
    /// WP-J9: get-o calls since the last world update (`SCRIPT_WATCHDOG`)
    pub get_o_calls: u32,
    /// WIRE-UI: load-time setter calls into the live store (`LIVE_DEFERRED` callees: the hub list FUN_001d7780)
    /// made while no built LiveQ is lent; replayed by the first `PersistentHost` that has one
    /// ANIM-ALL R23: + the script arguments the MArgs came from (object words -> guest address at replay)
    pub live_deferred: Vec<(u32, u32, Vec<super::methods::MArg>, Vec<Ref>)>,
    /// WP-J10: script messages (object, message) waiting for the script tick (`job_messages`)
    pub msgs: Vec<(u32, u32)>,
    /// WP-J10: `jobs::JobMgr::calls` already turned into messages + the job / task current then
    pub job_seen: usize,
    /// CRAWL R114: `JobMgr::load_gen` of the manager `job_seen` counts (0 = none yet)
    pub job_gen: u64,
    /// WP-J15: `JobMgr::task_msgs` already delivered
    pub task_msg_seen: usize,
    /// WP-EP1f: `JobMgr::ev_msgs` already queued
    pub ev_msg_seen: usize,
    /// WIRE-EP6: `JobMgr::msg17` already queued
    pub msg17_seen: usize,
    /// WP-EP1f: script messages with arguments (object, message, args) waiting for the script tick
    pub arg_msgs: Vec<(u32, u32, Vec<Ref>)>,
    /// WP-EP1f: the post-load script message 0 (FUN_001a5c48) was delivered for the current world
    pub post_load_sent: bool,
    pub job_prev: (Option<u32>, Option<u32>),
}

/// WP-J9: get-o calls one world update's scripts may make before get-o fails (`SpliceError::Malformed`).
/// REPLACED: the original has no guard; a handler run outside its context (e.g. a pick loop over
/// objects whose state it never sees change) would spin forever and freeze the frame.
pub const SCRIPT_WATCHDOG: u32 = 2_000_000;

/// WP-J9: world+0x3b0c / +0x3b10 / +0x3b14 (DAT_003e0f9c / DAT_003e0fa0 / DAT_003e0fa4) as object
/// payloads (world object index), taken from `jobs::JobMgr::{cur, cur_task, restart_override}`.
/// WP-J9: class-133 symid of field row 1264 (task +0x7c job).
const SYMID_TASK_JOB: u32 = 1093;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct JobView {
    pub job: Option<u32>,
    pub task: Option<u32>,
    pub restart: Option<u32>,
    /// (task object, job object): class-133 field row 1264 (symid 1093, kind 0x2084, offset 0x7c =
    /// `jobs::Task::job`)
    pub task_job: Vec<(u32, u32)>,
    /// WP-J11: (task object, state): class-133 field row 1286 (symid 342, kind 0x104f, flags 2, offset
    /// 0x54 = `jobs::Task::state`)
    pub task_state: Vec<(u32, i32)>,
    /// ANIM-ALL R39: (task object, enter count): class-133 field row 1267 (symid 1078, kind 5, flags 2, offset
    /// 0x64 = `jobs::Task::n_64`, FUN_00163d18 +0x64 += 1 before the msg 0x13)
    pub task_n64: Vec<(u32, i32)>,
    /// POOL1 R73: (job object, end task object): job +0x68 (prop 1244) for the field 1251 getter FUN_00163400
    pub job_end: Vec<(u32, u32)>,
}

/// POOL1: test probe of class-sent script messages (k27_script_msgs): (object, message, own frame)
pub static K27_MSG_LOG: std::sync::Mutex<Vec<(u32, u32, bool)>> = std::sync::Mutex::new(Vec::new());

impl JobView {
    pub fn of(m: &crate::engine::jobs::JobMgr) -> Self {
        let t = |i: Option<usize>| i.and_then(|i| m.tasks.get(i)).map(|t| t.obj as u32);
        let task_job = m.tasks.iter().filter_map(|t| Some((t.obj as u32, m.jobs.get(t.job?)?.obj as u32))).collect();
        let task_state = m.tasks.iter().map(|t| (t.obj as u32, t.state)).collect();
        let task_n64 = m.tasks.iter().map(|t| (t.obj as u32, t.n_64)).collect();
        let job_end = m.jobs.iter().filter_map(|j| Some((j.obj as u32, t(j.end)?))).collect();
        JobView { job: m.cur.and_then(|i| m.jobs.get(i)).map(|j| j.obj as u32), task: t(m.cur_task), restart: t(m.restart_override), task_job, task_state, task_n64, job_end }
    }
}

/// WP-J15: the job manager method rows of the last script ticks (`ClassLink::job_calls`): method 1258
/// FUN_00162660(job, aseg, b0, b4) and method 1284 FUN_00165118(task, arg) on the job / task of the object;
/// WP-EP1b: method 1239 FUN_00162df8(job) (restart the job at its start task or the +0x3b14 override).
/// WP-EP1f: method 1069 FUN_00162e38(job) (job complete -> hub).
pub fn apply_job_calls(sched: &mut Schedule, m: &mut crate::engine::jobs::JobMgr) {
    // WIRE-EP8: DAT_003e1114 (warp in progress, FUN_00228358 state 4) gates FUN_00162e38's hub request
    if let Some(set) = sched.cls.set.as_ref() {
        m.f_1114 = set.g.g_3e1114 != 0;
    }
    for (callee, obj, a) in std::mem::take(&mut sched.cls.job_calls) {
        let arg = |i: usize| a.get(i).copied().unwrap_or(0);
        match callee {
            0x0016_2660 => {
                if let Some(j) = m.jobs.iter().position(|j| j.obj == obj as usize) {
                    m.FUN_00162660(j, arg(0), arg(1), arg(2));
                }
            }
            0x0016_5118 => {
                if let Some(t) = m.tasks.iter().position(|t| t.obj == obj as usize) {
                    m.FUN_00165118(t, arg(0));
                }
            }
            // WP-EP1b: method 1239 (thunk 0x10ea98) FUN_00162df8 job restart (was recorded only)
            0x0016_2df8 => {
                if let Some(j) = m.jobs.iter().position(|j| j.obj == obj as usize) {
                    m.FUN_00162df8(j);
                }
            }
            // WP-EP1f: method 1069 (method row 333, thunk 0x10eb10) FUN_00162e38 job complete (the job
            // script's event handler on event 0x1a with a warp volume leaving the mission)
            0x0016_2e38 => {
                if let Some(j) = m.jobs.iter().position(|j| j.obj == obj as usize) {
                    m.FUN_00162e38(j);
                }
            }
            _ => {}
        }
    }
    // WP-K28: the class code's FUN_00164640 (task state) / FUN_00164df0 / FUN_001619a0 (event) calls
    for (callee, h, k, ctx) in std::mem::take(&mut sched.cls.host.k28_task_calls) {
        let task = m.tasks.iter().position(|t| t.obj + 1 == h as usize);
        match callee {
            0x0016_4640 => m.FUN_00164640(task, k, ctx as i32),
            0x0016_4df0 => {
                if let Some(t) = task {
                    m.FUN_00164df0(t, k);
                }
            }
            _ => m.FUN_001619a0(k, h as i32),
        }
    }
    // CRAWL R26: task +0x54 states for the class code (FUN_001f1cf8 key gate)
    sched.cls.host.task_states = m.tasks.iter().map(|t| (t.obj as u32 + 1, t.state)).collect();
}

/// WP-J10: the script messages of the job code (vt+0x1c FUN_001a61e0 -> FUN_001331f8 on the job / task
/// object) from the `JobMgr::calls` entries added since the last call: 0x1629a0 msg 0x13 (job start) /
/// 0x163d18 msg 0x13 (task enter) go to the current job / task, 0x162a60 msg 0x14 (job stop) /
/// 0x163f48 msg 0x14 (task leave) to the job / task current at the previous call when it changed, else
/// the current one. REPLACED: the original sends them synchronously inside the job functions to the
/// object they run on; `JobMgr::calls` keeps only (function, message), so the object is recovered from
/// the current job / task (two enters of one kind between two calls address the last one). Other
/// recorded messages (0x165170 / 0x162bb8 msg 0x11 events) carry no object: not sent. WP-J15: 0x1657d0's
/// vt+0x1c messages 0x16 / 0x15 go to the task object itself (`JobMgr::task_msgs`; e.g. the level-0 task
/// whose message-0x15 handler starts the title sequence oid 1672 once its state is 1).
pub fn job_messages(sched: &mut Schedule, m: &crate::engine::jobs::JobMgr) {
    // CRAWL R114 (merged FEEL R98): `job_seen` runs over calls_drained + calls.len(): `arch1_bridge` drains `calls`
    // (replay_calls_through) after they were seen, and that shrink was taken for a new world -> the kept calls
    // (e.g. 0x2283d0 warp oids) replayed after every drain. New world = a new `load_gen`. Opt-out
    // SLY2_NO_R114_JOBSEEN (old length test).
    let old = std::env::var_os("SLY2_NO_R114_JOBSEEN").is_some();
    let drained = if old { 0 } else { m.calls_drained };
    let total = drained + m.calls.len();
    if total < sched.job_seen || (!old && m.load_gen != sched.job_gen) {
        sched.job_gen = m.load_gen;
        sched.job_seen = drained; // new world: JobMgr::load
        sched.job_prev = (None, None);
        // FEEL R97: `calls` also shrinks every frame the ARCH1 bridge drains it (JobMgr::replay_calls_through);
        // resetting the task / event / msg-0x17 cursors there re-sent every task message (task 1 obj 896 msg 0x13
        // -> its handler restarts the safehouse-exit sequence 897 ~10x). The original sends each once; those lists
        // are never drained and keep their own shrink checks below (a new world also resets them in enter_level).
        // Opt-out SLY2_FEEL_NO_R97TASKMSG (old reset).
        if std::env::var_os("SLY2_FEEL_NO_R97TASKMSG").is_some() {
            sched.task_msg_seen = 0;
            sched.ev_msg_seen = 0;
            sched.msg17_seen = 0;
        }
    }
    if m.task_msgs.len() < sched.task_msg_seen {
        sched.task_msg_seen = 0;
    }
    if m.ev_msgs.len() < sched.ev_msg_seen {
        sched.ev_msg_seen = 0;
    }
    let v = JobView::of(m);
    let (pj, pt) = sched.job_prev;
    for &(f, msg) in &m.calls[sched.job_seen.saturating_sub(drained).min(m.calls.len())..] {
        let o = match (f, msg) {
            (0x001629a0, 0x13) => v.job,
            (0x00163d18, 0x13) => v.task,
            (0x00162a60, 0x14) => if pj != v.job { pj } else { v.job },
            (0x00163f48, 0x14) => if pt != v.task { pt } else { v.task },
            // WP-EP1d: FUN_002283d0(job +0xcc / task +0xc4 warp volume), run when the classes are lent
            (0x002283d0, oid) => {
                sched.cls.warp_oids.push(oid);
                None
            }
            _ => None,
        };
        if let Some(o) = o {
            sched.msgs.push((o, msg as u32));
        }
    }
    sched.job_seen = total;
    for &(t, msg) in &m.task_msgs[sched.task_msg_seen.min(m.task_msgs.len())..] {
        if let Some(tk) = m.tasks.get(t) {
            sched.msgs.push((tk.obj as u32, msg));
        }
    }
    sched.task_msg_seen = m.task_msgs.len();
    // WP-EP1f: message 0x11 {event, object} of FUN_00162bb8 / FUN_00165170 (handler symbol 17, e.g. the
    // job's event 0x1a -> method 1069 = FUN_00162e38 when the object is a warp volume)
    for &(o, ev, h) in &m.ev_msgs[sched.ev_msg_seen.min(m.ev_msgs.len())..] {
        let arg = h.checked_sub(1).map_or(Ref::Nil, Ref::Basic);
        sched.arg_msgs.push((o as u32, 0x11, vec![Ref::S32(ev), arg]));
    }
    sched.ev_msg_seen = m.ev_msgs.len();
    // WIRE-EP6: message 0x17 {arg} of FUN_00165118 (task +0x78 time limit) to the task object
    if m.msg17.len() < sched.msg17_seen {
        sched.msg17_seen = 0;
    }
    for &(o, a) in &m.msg17[sched.msg17_seen..] {
        sched.arg_msgs.push((o as u32, 0x17, vec![Ref::S32(a)]));
    }
    sched.msg17_seen = m.msg17.len();
    sched.job_prev = (v.job, v.task);
}

impl Schedule {
    /// FUN_00133040(t, world, object, call): insert after every node with time <= t (stable), id =
    /// ++DAT_003e11fc for world callbacks, -1 for object callbacks. Returns the id.
    pub fn ee_sub_133040(&mut self, t: f32, object: u32, call: Ref) -> i32 {
        // 0x00133040
        let id = if object == 0 {
            self.last_id += 1;
            self.last_id
        } else {
            -1 // (float)-NAN stored = 0xffffffff
        };
        // Quirk kept: the walk starts only if the head is <= t, then stops at the first node > the new time.
        let pos = if self.list.first().is_some_and(|h| h.t <= t) { self.list.iter().take_while(|c| c.t <= t).count() } else { 0 };
        self.list.insert(pos, Callback { t, object, call, id });
        id
    }

    /// FUN_00133130(world, id): remove the first node with that id (-1 never matches).
    pub fn ee_sub_133130(&mut self, id: i32) {
        // 0x00133130
        if id == -1 {
            return;
        }
        if let Some(i) = self.list.iter().position(|c| c.id == id) {
            self.list.remove(i);
        }
    }
}

/// Persistent script state of the running game (one per loaded world). Not Send (Rc-based VM):
/// the Bevy app keeps it as a NonSend resource.
pub struct SpliceState {
    pub vm: Vm,
    pub sched: Schedule,
    /// level the world frame belongs to (descriptor id, FUN_0020e890 load)
    pub level: Option<u32>,
    /// script ticks run in this world (frames with FUN_00226550)
    pub ticks: u64,
    /// errors raised by dispatched callbacks (the VM yields Nil and continues in the original)
    pub errors: Vec<SpliceError>,
    /// WP-J10: FUN_001ff858 anchor lists: per directory entry, the frame of each script slot
    /// (`scripts::run_level_scripts_framed`); object script frame = FUN_001ffa40(parent slot)
    pub frames: Vec<Vec<FrameRef>>,
}

impl Default for SpliceState {
    fn default() -> Self {
        Self { vm: Vm::new(), sched: Schedule::default(), level: None, ticks: 0, errors: Vec::new(), frames: Vec::new() }
    }
}

impl SpliceState {
    /// Level load (FUN_0020e890 -> new world): the world frame and callback list start empty. The id
    /// counter DAT_003e11fc is a global and is NOT reset. Returns true when the state was rebuilt.
    pub fn enter_level(&mut self, level: u32) -> bool {
        if self.level == Some(level) {
            return false;
        }
        self.vm.reset();
        self.sched.list.clear();
        self.level = Some(level);
        self.ticks = 0;
        self.errors.clear();
        // WP-J7: object memory belongs to the world (script Remove / members / deferred nodes of the
        // previous level must not leak into the new one); the lent class set is swapped by its owner
        self.sched.objs = Default::default();
        self.sched.cls.nodes.clear();
        self.sched.cls.recorded.clear();
        // WP-J10: slot frames of the old world (break proc <-> frame cycles), pending messages
        for f in self.frames.drain(..).flatten() {
            f.borrow_mut().bindings.clear();
        }
        self.sched.msgs.clear();
        self.sched.arg_msgs.clear();
        // WIRE-JOB: job-manager message cursors restart with the new world's JobMgr
        self.sched.job_seen = 0;
        self.sched.job_prev = (None, None);
        self.sched.task_msg_seen = 0;
        self.sched.ev_msg_seen = 0;
        self.sched.msg17_seen = 0;
        self.sched.post_load_sent = false;
        true
    }


}

/// FUN_00149c80 (callback part): run every callback of world+0x39f0 due at clock `t` (head time <= t),
/// in order; a callback scheduled while dispatching runs in this call if it is already due.
/// WP-J5: the world+0x3ca8 deferred member-call list is drained first (FUN_00149c80 head loop).
pub fn ee_sub_149c80(vm: &mut Vm, errors: &mut Vec<SpliceError>, host: &mut PersistentHost, t: f32) {
    // 0x00149c80
    // WP-J7: direct-fn nodes of class callees run in the lent class runtime; the rest stay recorded
    let Schedule { objs, cls, .. } = &mut *host.sched;
    objs.ee_sub_149c80_deferred(|o, c| {
        if !cls.call_deferred(c.object, c.vindex, c.fnw) {
            o.dispatched.push(c)
        }
    });
    while host.sched.list.first().is_some_and(|c| c.t <= t) {
        let mut c = host.sched.list.remove(0);
        c.id = -1;
        // FUN_00133000: eval the call list in the world frame
        let g = vm.global.clone();
        if let Err(e) = vm.eval(host, &c.call, &g) {
            errors.push(e);
        }
    }
}

/// FUN_001331f8(object, msg, schedule = 0, nargs, args, out): the script side of an object message
/// (LO vt+0x1c FUN_001a61e0). When the object's frame binds the handler symbol `msg` (FUN_00127db8,
/// through the parents) the call list `(msg self args..)` (FUN_00132ed8) is evaluated in the object frame
/// (FUN_00133000). Returns the handler's value (None: no frame / no handler).
/// WP-SCR6 d2 diagnostic hook: appends `(obj, msg)` for every `ee_sub_1331f8` call where the
/// handler lookup resolved to `Some(Ref::Nil)` (= the `NotCallable(-1)` case). Zero cost unless a
/// Nil dispatch actually fires (populated only on an existing error path).
pub static DISPATCH_NIL_LOG: std::sync::Mutex<Vec<(u32, u32)>> = std::sync::Mutex::new(Vec::new());

/// FUN_001331f8(object, msg, schedule = 0, nargs, args, out): the script side of an object message
/// (LO vt+0x1c FUN_001a61e0). When the object's frame binds the handler symbol `msg` (FUN_00127db8,
/// through the parents) the call list `(msg self args..)` (FUN_00132ed8) is evaluated in the object frame
/// (FUN_00133000). Returns the handler's value (None: no frame / no handler).
pub fn ee_sub_1331f8(vm: &mut Vm, host: &mut PersistentHost, frame: &FrameRef, obj: u32, msg: u32, args: &[Ref]) -> Result<Option<Ref>, SpliceError> {
    // 0x001331f8
    let found = crate::engine::splice::frame::lookup(frame, msg, true);
    if found.is_none() {
        return Ok(None);
    }
    // WP-SCR6 d2 diagnostic: record Nil dispatches (handler would be NotCallable(-1)).
    if matches!(found, Some(Ref::Nil)) {
        if let Ok(mut g) = DISPATCH_NIL_LOG.lock() {
            g.push((obj, msg));
        }
    }
    let mut call = vec![Ref::Symid(msg), Ref::Basic(obj)];
    call.extend_from_slice(args);
    let call = list_from(call);
    vm.eval(host, &call, frame).map(Some)
}

/// FUN_001ffa40(world, slot) for object `obj`: the frame of its entry's script slot (+0 of the object,
/// set by FUN_0013e3c8). None for slot -1 (null frame). REPLACED: template copies use the slot list of
/// the entry they belong to (`Lo::entry`), not the block current while they were instantiated.
pub fn object_frame(frames: &[Vec<FrameRef>], w: &World, obj: u32) -> Option<FrameRef> {
    // 0x001ffa40
    let mut lo = w.objects.get(obj as usize)?;
    // CRAWL R116: FUN_00256ee0 (refcounted `*obj = frame`) replaces the frame of ANY override-row target, also a
    // placed object with its own slot (r8 bell switch 142627 = entry 106 slot 0, bound to level slot 258.46 whose
    // msg 5 sets task 1819 state 4). FUN_001c9b08: `if (*target) FUN_00127b40(slot, *target)` first appends the
    // target's old frame to the slot frame's parent list (its own handlers stay reachable, e.g. the 258.46
    // `(super msg5)`). Done for every same-slot target at the first lookup (REPLACED: at load). SLY2_NO_R116_SLOTBIND=1
    // = old (only parent-slot -1 copies).
    if lo.parent_slot >= 0 && std::env::var_os("SLY2_NO_R116_SLOTBIND").is_none() {
        if let Some(&(e, s)) = w.script_slots.get(&(obj as usize)) {
            let slot = frames.get(e)?.get(usize::try_from(s).ok()?).cloned()?;
            let mut same: Vec<usize> = w.script_slots.iter().filter(|(_, v)| **v == (e, s)).map(|(&t, _)| t).collect();
            same.sort_unstable();
            for t in same {
                let Some(o) = w.objects.get(t).filter(|o| o.parent_slot >= 0) else { continue };
                if let Some(own) = frames.get(o.entry).and_then(|v| v.get(o.parent_slot as usize)) {
                    crate::engine::splice::frame::add_parent(&slot, own);
                }
            }
            return Some(slot);
        }
    }
    // WIRE-EP8: a template copy (FUN_001c9b08, parent slot -1) runs its source's script block (e.g. the
    // hub mission markers 1715..1719 = copies of 1707: msg 5 mission_start_jt with self = the copy)
    if lo.parent_slot < 0 {
        // WIRE-EP8: a slot bound by a template override row (FUN_00256ee0)
        if let Some(&(e, s)) = w.script_slots.get(&(obj as usize)) {
            return frames.get(e)?.get(usize::try_from(s).ok()?).cloned();
        }
        let mut src = lo.clone_of?;
        lo = w.objects.get(src)?;
        // HEIST: a copy of another placement of the entry runs in that placement's own frames
        if let Some(&k) = w.copy_inst.get(&(obj as usize)) {
            return frames.get(w.entries.len() + k)?.get(usize::try_from(lo.parent_slot).ok()?).cloned();
        }
        // HEIST r26: a copy of a copy (FUN_001c9b08 copies the source's +0 frame pointer, itself copied from
        // its source): follow the clone chain to the placement (row 15 brawl counter copy 3237 -> its tens
        // digit 9295 = copy of 1664 = copy of 1663, entry 103; without this 9295 had no frame and no 1234)
        for _ in 0..64 {
            if lo.parent_slot >= 0 {
                break;
            }
            if let Some(&(e, s)) = w.script_slots.get(&src) {
                return frames.get(e)?.get(usize::try_from(s).ok()?).cloned();
            }
            src = lo.clone_of?;
            lo = w.objects.get(src)?;
        }
    }
    let slot = usize::try_from(lo.parent_slot).ok()?;
    frames.get(lo.entry)?.get(slot).cloned()
}

/// WP-K28 / POOL1: one class-sent script message with argument words (`k27_script_msgs` entry), see send_messages
#[allow(clippy::too_many_arguments)]
fn deliver_k27_msg(vm: &mut Vm, errors: &mut Vec<SpliceError>, frames: &[Vec<FrameRef>], host: &mut PersistentHost, w: &World, h: u32, msg: u32, mut words: Vec<u32>) {
        // HEIST: a class-16 sensor's entry (message 0x1c) may be the player JT handle -> its object
        if msg == 0x1c && words.first() == Some(&crate::engine::classes::guard::PLAYER_HANDLE) {
            words[0] = crate::engine::game::spawn::player_object(w).map_or(0, |p| p as u32 + 1);
        }
        let (Some(obj), Some(args)) = (h.checked_sub(1), crate::engine::classes::k28::ee_sub_200da0(msg, &words)) else { return };
        // POOL1: SMA state messages (msg 8) of an SM object without its own script slot: FUN_001f01b8's
        // FUN_00256ee0(sma, *sm) binds the SM's frame = a script slot of its entry. REPLACED: the slot whose frame
        // binds the handler (e.g. the bomb template's s8 in entry 170)
        // POOL1 verification probe: every SMA/class script message (obj, msg, frame found) reaching the VM
        if let Ok(mut v) = K27_MSG_LOG.lock() { if v.len() < 4096 { v.push((obj, msg, object_frame(frames, w, obj).is_some())); } }
        let fr = object_frame(frames, w, obj).or_else(|| {
            if msg != 8 { return None; }
            let lo = w.objects.get(obj as usize)?;
            // the SM's owner (bomb copy -> its template via clone_of) carries the bound script block
            object_frame(frames, w, lo.owner? as u32).filter(|f| crate::engine::splice::frame::lookup(f, msg, true).is_some())
                // ANIM-ALL R41: only the slot of a same-oid placement (the copy's source, FUN_001c9b08 copies its +0
                // frame); a slot-less SM (e.g. row-14 SM 28246 under 21871) has no frame (FUN_001ffa40(-1) = 0) and
                // must not run another SM's handler (it ran SM 72795's s5 -> NIL (get owner 1652))
                .or_else(|| frames.get(lo.entry)?.iter().enumerate().find(|(si, f)| crate::engine::splice::frame::lookup(f, msg, true).is_some()
                    && w.objects.iter().any(|o| o.entry == lo.entry && o.oid == lo.oid && usize::try_from(o.parent_slot).ok() == Some(*si))).map(|(_, f)| f.clone()))
        });
        if let Some(fr) = fr {
            // POOL1 r36: FUN_001f13a8 -> FUN_001331f8(sma, 8, ..): self = the SMA (field 251 = its +0x28 owner)
            let sma = if msg == 8 && words.len() == 2 { host.sched.cls.host.sma_script_self_q.remove(&(h, words[0], words[1])) } else { None };
            if let Some(x) = sma { host.sched.cls.host.aseg_script_self = Some(x); }
            let r = ee_sub_1331f8(vm, host, &fr, sma.map_or(obj, |x| x.0), msg, &args);
            if sma.is_some() { host.sched.cls.host.aseg_script_self = None; }
            if let Err(e) = r {
                errors.push(e);
            }
        }
    }

/// POOL1 r30: one ASEGA / SMA script message without arguments (`aseg_script_msgs` entry), see send_messages
#[allow(clippy::too_many_arguments)]
fn deliver_aseg_msg(vm: &mut Vm, errors: &mut Vec<SpliceError>, frames: &[Vec<FrameRef>], host: &mut PersistentHost, w: &World, ai: usize, h: u32, owner: u32, msg: u32) {
        let Some(fr) = object_frame(frames, w, ai as u32) else { return };
        host.sched.cls.host.aseg_script_self = Some((h, owner));
        let r = ee_sub_1331f8(vm, host, &fr, h, msg, &[]);
        host.sched.cls.host.aseg_script_self = None;
        if let Err(e) = r {
            errors.push(e);
        }
    }

/// WP-J10: deliver the pending script messages (`Schedule::msgs`) of the job code.
pub fn send_messages(vm: &mut Vm, errors: &mut Vec<SpliceError>, frames: &[Vec<FrameRef>], host: &mut PersistentHost) {
    let mut msgs = std::mem::take(&mut host.sched.msgs);
    let Some(w) = host.world else { return };
    // HEIST r27: FUN_001a5e28 (LO update, vt slot 15 / +0x7c of the C2c94e0 classes): +0x30 (BIF 107 next update
    // time) <= DAT_002dded0 -> (vt+0x1c)(obj, 1, 1, ..) = script message 1, every update while due (the handler
    // re-arms it with BIF 107, e.g. the Big House Brawl spawner 17343 s1). REPLACED: only objects whose +0x30 a
    // script set (the load-time value is never due), sent here in object order.
    let now = host.level.base.t;
    let mut due: Vec<u32> = host.sched.objs.next_update.iter().filter(|&(_, &t)| t <= now).map(|(&o, _)| o).collect();
    due.sort_unstable();
    msgs.extend(due.into_iter().map(|o| (o, 1)));
    // WP-EP1f: FUN_001a5c48 (LO post-load, C2c94e0 slot 14): (vt+0x1c)(obj, 0, schedule = 1) = script
    // message 0 of every object (e.g. task 1757 binds its method 1326 there). REPLACED: the scheduled call
    // (FUN_00133040) is evaluated at the first script tick after the level's frames exist, in object order.
    if !host.sched.post_load_sent && !frames.is_empty() {
        host.sched.post_load_sent = true;
        // WIRE-EP7: the load-time field-8 ensure-o rows (FUN_0013f810 -> FUN_00132d88) precede message 0
        for obj in 0..w.objects.len() {
            for (sym, v) in super::fields::load_user_props_all(w, obj) {
                host.sched.objs.ee_sub_200d10(obj as u32, sym, v);
            }
        }
        for obj in 0..w.objects.len() as u32 {
            if let Some(fr) = object_frame(frames, w, obj) {
                if let Err(e) = ee_sub_1331f8(vm, host, &fr, obj, 0, &[]) {
                    errors.push(e);
                }
            }
        }
    }
    for (obj, msg) in msgs {
        if let Some(fr) = object_frame(frames, w, obj) {
            if let Err(e) = ee_sub_1331f8(vm, host, &fr, obj, msg, &[]) {
                errors.push(e);
            }
        }
    }
    for (obj, msg, args) in std::mem::take(&mut host.sched.arg_msgs) {
        if let Some(fr) = object_frame(frames, w, obj) {
            match ee_sub_1331f8(vm, host, &fr, obj, msg, &args) {
                Ok(r) => {
                    if obj == 917 && msg == 0x11 {
                        eprintln!("[PAD1] obj 917 msg 0x11 result: {r:?}");
                    }
                }
                Err(e) => {
                    if obj == 917 {
                        eprintln!("[PAD1] obj 917 script error: {e:?}");
                    }
                    errors.push(e);
                }
            }
        }
    }
    // WP-K28: level-script messages of the class code with arguments (sequence action 9, handle = index + 1)
    // ANIM-ALL R84: live-image SMA messages in their send order (FUN_001f01b8: message 2 (vt+0x1c(2)) BEFORE the
    // FUN_001f13a8 message 8 of the initial label): the safehouse table script 249.1's s2 (seat + SetState(4), which
    // creates the gang's SM-419 SMAs) must run before its s8 (#sym1748: goal 5526 / 5525 / 17058 on those SMAs).
    // The split queues below delivered every message 8 first -> nil SMA, gang T-pose. Opt-out SLY2_NO_R84_SMAORDER=1.
    for (si, sma, oh, msg, words) in std::mem::take(&mut host.sched.cls.host.live_sma_seq) {
        if msg == 8 && words.len() == 2 {
            host.sched.cls.host.sma_script_self_q.insert((si as u32 + 1, words[0], words[1]), (sma, oh));
            deliver_k27_msg(vm, errors, frames, host, w, si as u32 + 1, 8, words);
        } else {
            deliver_aseg_msg(vm, errors, frames, host, w, si, sma, oh, msg);
        }
    }
    for (h, msg, words) in std::mem::take(&mut host.sched.cls.host.k27_script_msgs) {
        deliver_k27_msg(vm, errors, frames, host, w, h, msg, words);
    }
    // BOSS R30: FUN_001758a0's vt +0x1c message 0x18 {record oid} per candidate attack record (Dimitri's s24 counts
    // the polls / picks); the reply {0x17, 1} stays unless the handler answers #f -> first allowed record
    for (g, cands) in std::mem::take(&mut host.sched.cls.host.atk_msg18_q) {
        let fr = object_frame(frames, w, g as u32).filter(|f| crate::engine::splice::frame::lookup(f, 0x18, true).is_some());
        if fr.is_some() {
            host.sched.cls.host.atk_msg18_hdl.insert(g);
        }
        let mut pick = None;
        for &(k, oid) in &cands {
            let ok = match &fr {
                None => true,
                Some(fr) => match ee_sub_1331f8(vm, host, fr, g as u32, 0x18, &[Ref::S32(oid)]) {
                    Ok(r) => !matches!(r, Some(Ref::Bool(false)) | Some(Ref::Nil)),
                    Err(e) => { errors.push(e); true }
                },
            };
            if ok { pick = Some(k); break; }
        }
        host.sched.cls.host.atk_msg18_ans.insert(g, (cands, pick));
    }
    // POOL1 r30: ASEGA script messages 2 (FUN_0012f9b8 start) / 3 (FUN_00130b18 retract) in the frame the
    // ASEG's oid binds (FUN_00256ee0(asega, *aseg) = the ASEG object's script slot), self = the ASEGA (e.g. the
    // bentley_mine fuse ASEG's s2: ((get (get self 251) 1241)) = explode). REPLACED: the ASEGA is not a
    // script object; get-o field 251 (+0x1c owner) of it is answered from `aseg_script_self`.
    for (ai, h, owner, msg) in std::mem::take(&mut host.sched.cls.host.aseg_script_msgs) {
        deliver_aseg_msg(vm, errors, frames, host, w, ai, h, owner, msg);
    }
    // HEIST: ASEG event 0xc -> ASEGA script message 4 {payload} (FUN_00131290 case 0xc), same frame / self as above
    for (ai, h, owner, v) in std::mem::take(&mut host.sched.cls.host.heist_aseg_msg4) {
        let Some(fr) = object_frame(frames, w, ai as u32) else { continue };
        host.sched.cls.host.aseg_script_self = Some((h, owner));
        host.sched.cls.host.heist_aseg_obj = ai as u32;
        let r = ee_sub_1331f8(vm, host, &fr, h, 4, &[Ref::S32(v)]);
        host.sched.cls.host.aseg_script_self = None;
        if let Err(e) = r {
            errors.push(e);
        }
    }
}

/// The script work of one FUN_00226550 world update at its original positions: FUN_00149c80 at the
/// start (always), then, only when dt > 0, after the object updates and once more at the end.
/// WIRE-UI: script methods whose live-store callee is a load-time setter (state the live store must see
/// even when the level-load scripts ran before it was built): FUN_001d7780 (hub C2d80d0 character list)
/// WIRE-EP8: + FUN_00142540 (JT scripted behaviour 0xc2 of the cellar intro seq; 1:1 in q_00141ef8)
pub const LIVE_DEFERRED: &[u32] = &[0x001d_7780, 0x0014_2540];

impl PersistentHost<'_> {
    /// POOL1 r30: get-o / set-o of a class-101 (bentley_mine / thrown bomb) member row on the live instance
    fn k15_101_member(&mut self, obj: u32, class: Option<u16>, sym: u32, v: Option<&Ref>) -> Option<Ref> {
        if class != Some(101) {
            return None;
        }
        let row = super::fields::ee_sub_1ffcf8_field(101, u16::try_from(sym).ok()?)?;
        // POOL1 R52: flags-4 getter rows (FUN_001ffd98 kind 0: FUN_00129598(getter(self) != 0)): 1105 (sym 961)
        // FUN_00224610 remote kind (+0x15c 7 / 0xb), 1106 FUN_00224638. The bomb SM's s8 at label 111 sets goal 106
        // (the 1.87 s fuse blink -> 112 -> explode) iff (not (get bomb 961)): a placed kind-5 bomb fuses by itself.
        if (row == 1105 || row == 1106) && v.is_none() {
            let c = self.sched.cls.set.as_mut()?.k15_101_get(obj as usize)?;
            return Some(Ref::Bool(if row == 1105 { c.ee_sub_224610() } else { c.ee_sub_224638() }));
        }
        let float = row == 1112;
        let w = v.map(|r| if float { r.as_f32().to_bits() } else { match r { Ref::F32(f) => *f as i32 as u32, o => o.payload() } });
        let set = self.sched.cls.set.as_mut()?;
        let r = set.k15_101_member(obj as usize, row, w)?;
        Some(if float { Ref::F32(f32::from_bits(r)) } else { Ref::S32(r as i32) })
    }
    fn live_ready(&self) -> bool {
        self.live_q.as_ref().is_some_and(|q| q.mem.is_some())
    }
    /// WIRE-UI: replay `Schedule::live_deferred` once a built live store is lent
    pub fn flush_live_deferred(&mut self) {
        // BOSS R28: the class code's vtable +0x44 / +0x4c and ASEG kind 0xd..0x10 calls on the objects whose class
        // runs in the live image (class-88 TNT barrel copies, as R27's field 27): the original's call on that object
        // (e.g. the fuse SM 27600's 30757 -> 30758 ASEG 98049: kind 0x10 drops the barrel's solid shape 11725, so
        // it falls; its retract's kind 8 removes the barrel 95845 = rec11 / cap_boss_tnt_fuse +1.99 s / +2.97 s)
        let calls = std::mem::take(&mut self.sched.cls.host.live_calls);
        if self.live_ready() {
            for (f, h, arg) in calls {
                let Some(obj) = h.checked_sub(1) else { continue };
                // HEIST r23: + class 72 (C2d15d0, the Train Hack ship: slot 78 from sequence action 7)
                // CRAWL R101: + class 70 (hub C2d80d0 field 806 setter, ref_op_set_o)
                if !matches!(self.class_of(obj), Some(88) | Some(72) | Some(70)) {
                    continue;
                }
                let mem = self.live_q.as_mut().unwrap().mem.as_mut().unwrap();
                let addr = super::live_bridge::basic_addr(mem, obj);
                if addr == 0 {
                    continue;
                }
                use crate::engine::classes::q::q2mem::QMem;
                let f = if f < 0x1000 { let vt = mem.r32(addr + 0xc); mem.r32(vt + f) } else { f };
                mem.call(f, &[addr, arg], &[]);
            }
        }
        if !self.live_ready() || self.sched.live_deferred.is_empty() {
            return;
        }
        let lq = self.live_q.as_mut().unwrap();
        for (callee, obj, a, raw) in std::mem::take(&mut self.sched.live_deferred) {
            // ANIM-ALL R23: FUN_00142540's target (int-kind word = the object pointer) as at a live call
            let a = super::live_bridge::basic_words(lq, &raw, &a);
            let _ = super::live_bridge::try_method(lq, callee, obj, &a);
        }
    }
}

pub fn tick_world_update(vm: &mut Vm, errors: &mut Vec<SpliceError>, host: &mut PersistentHost, t: f32, dt: f32) {
    host.flush_live_deferred();
    // 0x00226550
    host.sched.get_o_calls = 0;
    // WP-J15: the FUN_00226550 step order (`world_update`); the other steps are the app's systems
    struct H<'a, 'b> {
        vm: &'a mut Vm,
        errors: &'a mut Vec<SpliceError>,
        host: &'a mut PersistentHost<'b>,
        t: f32,
    }
    impl super::world_update::WorldUpdateHost for H<'_, '_> {
        fn step(&mut self, s: super::world_update::WorldStep) {
            if s == super::world_update::WorldStep::Scripts {
                ee_sub_149c80(self.vm, self.errors, self.host, self.t);
            }
        }
    }
    super::world_update::ee_sub_226550(&mut H { vm, errors, host, t }, dt, 0.0);
}

/// Splice host for the persistent state: the J3 `LevelSpliceHost` (fallback) plus the method table,
/// get-o for methods, BIFs 98/99 (level table), 105/106 (callbacks) and the camera methods.
pub struct PersistentHost<'a> {
    pub level: LevelSpliceHost<'a>,
    pub cm: Option<&'a mut Cm>,
    pub clk: CmClock,
    pub world: Option<&'a World>,
    /// FUN_0020f318 table of the loaded level (DAT_003e1c44 / DAT_003e1c48)
    pub table: Option<&'a LevelTable>,
    pub sched: &'a mut Schedule,
    /// WP-TR1: optional bridge to the QMem store for method callees ported as `q` functions.
    pub live_q: Option<&'a mut crate::engine::classes::q::live::LiveQ>,
}

impl PersistentHost<'_> {
    /// Class of a Basic payload (vtable +0x34 class info +4).
    pub fn class_of(&self, basic: u32) -> Option<u16> {
        match basic {
            BASIC_PLAYER_GLOBAL => Some(CLASS_SW),
            // WP-J8: 0x3dd040 (ctor FUN_001fcd38, vtable 0x2ca510 -> class info 0x2bb6c8) / 0x3da0c0
            // (FUN_001de008, vtable 0x2ca490 at +0xc -> class info 0x2bb6f0)
            BASIC_WORLD_GLOBAL => Some(CLASS_152),
            BASIC_137_GLOBAL => Some(CLASS_154),
            crate::engine::camera::bifs::CM_OBJECT_ADDR => Some(CLASS_CM),
            // POOL1 R52: k25 SMA handles (tag 0x42..) are class 136 (fields 371 owner +0x28, 372 goal); their
            // tag also has the emitter bit set
            b if crate::engine::classes::k25::AnimRt::is_sma(b) => Some(CLASS_SMA),
            b if b & BASIC_EMITTER != 0 => Some(CLASS_EMITTER),
            // WP-J9: getter-created run-time records (ASEGA / link / class 147)
            b if b & super::fields::BASIC_RT != 0 => self.sched.objs.rt_get(b).map(|r| r.class),
            // HEIST r16: an embedded JOY tracker (C2ca240, class 137)
            b if b & 0xf000_0000 == super::fields::BASIC_JOY => Some(137),
            b => self.world.and_then(|w| w.objects.get(b as usize)).map(|o| o.class),
        }
    }

    fn mission_entry(&self) -> Option<&crate::engine::disc::descriptor::LevelTableEntry> {
        let m = self.level.mission;
        if m == -1 {
            return None;
        }
        // Quirk not reproduced: the original indexes the table with no bound (heap read)
        self.table.and_then(|t| t.entries.get(m as usize))
    }

    /// The SO collision parts (`CollisionWorld::build_so`; the J7-lent one when built).
    pub fn so_parts(&self, w: &World) -> std::sync::Arc<crate::engine::physics::query::CollisionWorld> {
        self.sched.cls.host.so.clone().unwrap_or_else(|| std::sync::Arc::new(crate::engine::physics::query::CollisionWorld::build_so(w)))
    }

    /// WP-J11: FUN_001249a8(op, argv): argv[0] an object list (kind 0 -> returned as is); for each object,
    /// each +0xe4 part (0xb0 B, count +0xe0) without +0x10 bit 50: FUN_0014c6a0(part, *argv[1], *argv[2],
    /// 1, &lsg) = FUN_0014c6d8 (part +0x74 BSP -> FUN_00118668; none -> FUN_0014c748 when +0x10 bit 49).
    /// A hit: op 0x7b appends (obj lsg+0x00 lsg+0x10) to the result list, op 0x7c returns (obj lsg+0x10)
    /// at once. No hit: the empty list. REPLACED: the parts are the object's SO colliders in order
    /// (`CollisionWorld::colliders`); parts without a BSP (bit 49 primitive path) and the bit-50 skip are
    /// not in the world model (never hit / never skipped).
    fn ee_sub_1249a8(&mut self, all: bool, a: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x001249a8
        let Some(w) = self.world else {
            return if all { self.level.ref_op_hit_test_objects(a) } else { self.level.ref_op_hit_test_objects_first(a) };
        };
        let Some(l) = a.first() else { return Ok(Ref::Nil) };
        let Ref::Pair(p) = l else { return Ok(if l.tagk() == 0 { l.clone() } else { list_from(Vec::new()) }) };
        let v = |i: usize| match a.get(i) {
            Some(Ref::Vector(v)) => *v,
            _ => [0.0; 3],
        };
        let (p1, p2) = (v(1), v(2));
        let so = self.so_parts(w);
        let mut out = Vec::new();
        for o in list_items(Some(p.clone())) {
            let Ref::Basic(obj) = o else { continue };
            for c in so.colliders.iter().filter(|c| c.object as u32 == obj) {
                if c.bsp.nodes.is_empty() {
                    continue;
                }
                let mut lsg = Vec::new();
                if crate::engine::physics::bsp::FUN_00118668(&c.geom, &c.bsp, 0, c.bsp.nodes.len(), p1, p2, 1, &mut lsg) < 1 {
                    continue;
                }
                let Some(h) = lsg.first() else { continue };
                if !all {
                    return Ok(list_from(vec![Ref::Basic(obj), Ref::Vector(h.pos[1])]));
                }
                out.push(list_from(vec![Ref::Basic(obj), Ref::Vector(h.pos[0]), Ref::Vector(h.pos[1])]));
            }
        }
        Ok(list_from(out))
    }
}

fn emitter_slot(m: &Method) -> CmEmitter {
    CmEmitter { id: Some((m.object & !BASIC_EMITTER) as usize), pitch: 0.0, fade: 0.0 }

}

impl SpliceHost for PersistentHost<'_> {
    fn fallback(&mut self) -> Option<&mut dyn SpliceHost> {
        Some(&mut self.level)
    }

    fn call_method(&mut self, m: &Method, args: &[Ref]) -> Result<Ref, SpliceError> {
        // WP-J9: getter rows (kind != 0x12, flags & 1) return the callee's value (methods::getters)
        if let Some(g) = methods::getter_by_thunk(m.method_id) {
            let a = g.m.kinds.map(|k| methods::ee_sub_201110(k, args)).unwrap_or_default();
            return methods::getters::call(self, g, m.object, &a, args);
        }
        let desc = methods::by_thunk(m.method_id);
        let a = desc.and_then(|d| d.kinds).map(|k| methods::ee_sub_201110(k, args)).unwrap_or_default();
        let es = &mut *self.level.base.emitters;
        match m.method_id {
            // 0x00101e18 / 0x00101e90 / 0x00101f08 (class 151 emitter)
            0x0010_1e18 => FUN_001fb018(es, &emitter_slot(m)),
            0x0010_1e90 => FUN_001fb060(es, &emitter_slot(m)),
            0x0010_1f08 => {
                let fade = match a.first() {
                    Some(MArg::F32(f)) => *f,
                    _ => 0.0,
                };
                FUN_001fb098(es, fade, &mut emitter_slot(m));
            }
            // PORT2: 0x00101f90 -> FUN_001fb4f0(near, far, e): +0x34 = far <= near ? near + 1 : far, +0x30 = near
            0x0010_1f90 => {
                let f = |x: Option<&MArg>| match x {
                    Some(MArg::F32(f)) => *f,
                    Some(MArg::I32(n)) => *n as f32,
                    _ => 0.0,
                };
                super::music_link::ee_sub_1fb4f0(es, f(a.first()), f(a.get(1)), emitter_slot(m).id);
            }
            // 0x001026e0 FUN_0014f750(g_cm, arg kind 0x1004 = payload) / 0x00102768 FUN_0014d8c0(g_cm)
            0x0010_26e0 | 0x0010_2768 => {
                let Some(cm) = self.cm.as_deref_mut() else { return Err(SpliceError::HostUnimplemented("cm method without camera".into())) };
                if m.method_id == 0x0010_2768 {
                    FUN_0014d8c0(cm);
                } else {
                    let k = match a.first() {
                        Some(MArg::Word(w)) => *w as i32,
                        _ => 0,
                    };
                    FUN_0014f750(cm, k, self.clk);
                }
            }
            // WP-J10: 0x001020d8 -> FUN_001fe738(0x3dd040): stop the current music instance (replaced like
            // getter 0x00102038: only the +0x200 current-instance key is modelled)
            0x0010_20d8 => self.sched.objs.music_cur = None,
            // PORT2: 0x001025a8 -> FUN_001fe9f8(vol, 0x3dd040) / 0x00102630 -> FUN_001fea60(vol, 0x3dd040, bus, 0)
            0x0010_25a8 | 0x0010_2630 => {
                let f = |x: Option<&MArg>| match x {
                    Some(MArg::F32(f)) => *f,
                    Some(MArg::I32(n)) => *n as f32,
                    _ => 0.0,
                };
                let ml = &mut self.sched.objs.music_link;
                if m.method_id == 0x0010_25a8 {
                    ml.ee_sub_1025a8(f(a.first()));
                } else {
                    let bus = match a.first() {
                        Some(MArg::I32(n)) => *n,
                        Some(MArg::Word(w)) => *w as i32,
                        _ => 0,
                    };
                    ml.ee_sub_102630(bus, f(a.get(1)));
                }
            }
            // WP-J10: 0x00104bf8 -> FUN_001f0598 on a link record (class 136, `fields::RtRecord`)
            0x0010_4bf8 if self.sched.objs.ee_sub_1f0598(m.object) => {}
            // BOSS R27: on a live SMA (getter 0x00104840): FUN_001f0598 (method 237) / FUN_001f0948(sma, label) (method 236)
            0x0010_4bf8 if crate::engine::classes::k25::AnimRt::is_sma(m.object) => self.sched.cls.host.k25.ee_sub_1f0598(m.object),
            0x0010_4b70 if crate::engine::classes::k25::AnimRt::is_sma(m.object) => {
                let label = match a.first() {
                    Some(MArg::I32(n)) => *n,
                    Some(MArg::Word(w)) => *w as i32,
                    Some(MArg::F32(f)) => *f as i32,
                    _ => -1,
                };
                if std::env::var_os("SLY2_R83IDLE").is_some() {
                    eprintln!("R83 104b70 sma {:#x} label {label}", m.object);
                }
                if let Some(w) = self.sched.cls.host.world.clone() {
                    self.sched.cls.host.k25.ee_sub_1f0948(&w, m.object, label);
                }
            }
            // WP-J11: 0x00101c68 (ASEGA, class 134) -> FUN_00130d30(f1, f2, asega, mode (kind 0x100c)): speed
            // +0x44 = f2; time +0x1c = f1 (mode 0) / +0x1c + f1 (mode 1) / ASEG +0x74 + f1 (mode 2); a new
            // time re-evaluates the ASEGA. Replaced like getter 0x00101a88: the seek is a play request at
            // the new time (`aseg_play`), the record's `f` keeps the time; mode 2 (ASEG +0x74 not on the
            // record) and the speed are not modelled.
            // WP-J11: 0x001105b8 (field 1420) -> FUN_001d4b90(obj, key, oid, a, b): +0x4f8 < 8 -> new
            // 0xb0-byte entry {+0x14 key, +0x18 oid, +0x2c a, +0x38 b} at +0x4fc[+0x4f8++] (getter 0x00110738
            // finds it by key). Replaced: the entry is a class-153 `rt` record (alo = oid, arg = key); +0x2c /
            // +0x38 and the post-load bind (FUN_0015e938 / FUN_001f4c58 of the class-77 load) are not kept.
            0x0011_05b8 => {
                let n = |i: usize| match a.get(i) {
                    Some(MArg::I32(n)) => *n,
                    Some(MArg::Word(w)) => *w as i32,
                    _ => 0,
                };
                let objs = &mut self.sched.objs;
                if objs.rt.iter().filter(|r| r.class == 153 && r.owner == m.object).count() < 8 {
                    objs.rt_push(crate::engine::game::fields::RtRecord { class: 153, owner: m.object, alo: n(1) as u32, f: 0.0, arg: n(0) });
                }
            }
            0x0010_1c68 => {
                let f = |i: usize| match a.get(i) {
                    Some(MArg::F32(f)) => *f,
                    _ => 0.0,
                };
                let mode = match a.first() {
                    Some(MArg::I32(n)) => *n,
                    Some(MArg::Word(w)) => *w as i32,
                    _ => -1,
                };
                let objs = &mut self.sched.objs;
                if objs.rt_get(m.object).is_some_and(|r| r.class == 134) {
                    let r = &mut objs.rt[(m.object & 0x0fff_ffff) as usize];
                    let t = match mode {
                        0 => Some(f(1)),
                        1 => Some(r.f + f(1)),
                        _ => None,
                    };
                    if let Some(t) = t.filter(|&t| t != r.f) {
                        r.f = t;
                        let owner = r.owner;
                        objs.aseg_play.push((owner, t));
                    }
                }
            }
            // 0x00101d20 (ASEG): FUN_00130b18 retract
            0x0010_1d20 => self.sched.objs.retract.push(m.object),
            // WP-J6: LO add/remove, mesh show/hide, +0x84 oid list (methods::lo)
            id if methods::lo::THUNKS.contains(&id) => {
                let k = match a.first() {
                    Some(MArg::I32(n)) => *n,
                    Some(MArg::Word(w)) => *w as i32,
                    _ => 0,
                };
                methods::lo::call(&mut self.sched.objs, self.world, id, m.object, k);
                // CRAWL R53: on a class-114 sequence object the +0x84 list is the one its wait 4 (FUN_001e3890)
                // reads: C2d8c48::k22_sig (row 6 seq 1847 waits for the control box script's signal 0x4368)
                if matches!(id, 0x0010_6800 | 0x0010_6888) {
                    if let Some(set) = self.sched.cls.set.as_mut() {
                        if let Some((_, crate::engine::classes::Inst::C2d8c48(c))) = set.more.iter_mut().find(|x| x.0 == m.object as usize) {
                            if id == 0x0010_6800 {
                                crate::engine::classes::k22_seq::ee_sub_1e0020(c, k);
                            } else {
                                crate::engine::classes::k22_seq::ee_sub_1e0070(c, k);
                            }
                        }
                    }
                }
                // CRAWL: LO remove / add reach the class host (collision SO, update lists)
                if matches!(id, 0x0010_0a90 | 0x0010_0b88) {
                    self.sched.cls.host.lo_world_link(m.object as usize, id == 0x0010_0a90);
                }
            }
            // POOL1: 0x0010d9e0 (method 1110) -> FUN_00224e70(o, x) (classes::k15_101, player::plugin drains it)
            0x0010_d9e0 => {
                let x = match a.first() {
                    Some(MArg::Word(w)) => *w,
                    // the radius (FUN_00224e70 float argument)
                    Some(MArg::I32(n)) => (*n as f32).to_bits(),
                    Some(MArg::F32(f)) => f.to_bits(),
                    _ => 0,
                };
                self.sched.cls.explode_224e70.push((m.object, x));
                self.sched.cls.explode_count += 1;
            }
            // WP-J8: 0x00100fe0 -> FUN_001f7918(alo, mask (kind 0x1011), byte (kind 0x1012))
            0x0010_0fe0 => {
                let w = |i: usize| match a.get(i) {
                    Some(MArg::Word(w)) => *w,
                    Some(MArg::I32(n)) => *n as u32,
                    _ => 0,
                };
                self.sched.objs.ee_sub_1f7918(m.object, w(0), w(1) as u8);
                // HEIST r18: the class host's +0x110 table (read by FUN_001f78d8 on hits, keyed by handle) gets the same
                // write over its current bytes (load-time rows / class init), e.g. the EP2 battle start se69
                // ((get self 125) 256 8) on the chopper 118349
                let hh = m.object + 1;
                let mut t = crate::engine::classes::ClassHost::alo_110(&mut self.sched.cls.host, hh);
                for (i, v) in t.iter_mut().enumerate() {
                    if w(0) & (1 << i) != 0 {
                        *v = w(1) as u8;
                    }
                }
                self.sched.cls.host.alo_110.insert(hh, t);
            }
            // WP-EP2F: 0x00101218 -> FUN_001f7fa0(alo, a0 (kind 1), a1 (kind 1)): FUN_001ee678(a0, a1, x) (x+0x60 = a0,
            // x+0x64 = a1) for every attached element (`MeshSet::descs`, the +0xa8 list) of class 0x8f. 1:1 on
            // `ObjectFields::desc_60` (the port has no element instances).
            0x0010_1218 => {
                let f = |i: usize| match a.get(i) {
                    Some(MArg::F32(f)) => f.to_bits(),
                    _ => 0,
                };
                let (a0, a1) = (f(0), f(1));
                let hits: Vec<usize> = self
                    .world
                    .and_then(|w| crate::engine::gfx::attach::alo_of(w, m.object as usize))
                    .map(|al| {
                        al.mesh.descs.iter().enumerate().filter(|(_, d)| d.as_ref().is_some_and(|d| crate::engine::player::p4::FUN_00132b10(d.0 as i32, 0x8f))).map(|(i, _)| i).collect()
                    })
                    .unwrap_or_default();
                for i in hits {
                    self.sched.objs.desc_60.insert((m.object, i), (a0, a1));
                }
            }
            // WP-J8: 0x001024b0 -> FUN_001fe310(0x3dd040, id (kind 0x13)); returns Void
            0x0010_24b0 => {
                let id = match a.first() {
                    Some(MArg::I32(n)) => *n,
                    Some(MArg::Word(w)) => *w as i32,
                    _ => 0,
                };
                self.sched.objs.ee_sub_1fe310(id);
            }
            // WP-J8: 0x3dd040 counters (no arguments)
            0x0010_2250 | 0x0010_22c8 | 0x0010_2340 | 0x0010_23b8 => {
                self.sched.objs.ee_sub_1fee90(desc.map(|d| d.callee).unwrap_or(0));
            }
            // WP-J8: 0x0010f460 -> FUN_001da4b8(value (kind 1), 0x3da0c0, id (kind 0x10)) /
            // 0x0010f508 -> FUN_001da578(0x3da0c0, id (kind 0x10))
            0x0010_f460 | 0x0010_f508 => {
                let int = |i: usize| match a.get(i) {
                    Some(MArg::I32(n)) => *n,
                    Some(MArg::Word(w)) => *w as i32,
                    _ => 0,
                };
                let t = self.level.base.t;
                let g = &mut self.sched.objs.g3da0c0;
                if m.method_id == 0x0010_f460 {
                    let v = match a.get(1) {
                        Some(MArg::F32(f)) => *f,
                        _ => 0.0,
                    };
                    g.ee_sub_1da4b8(v, int(0), t, None);
                } else {
                    g.ee_sub_1da578(int(0));
                }
            }
            // WP-Q3: user-property setters 0x001002a8..0x00100760 -> FUN_00132bc8..FUN_00132e40 (q_131990)
            id if super::q_131990::call(&mut self.sched.objs, id, m.object, &a) => {}
            // PORT2: inline virtual-call thunks (vslot_bridge)
            id if self.live_q.is_some() && super::vslot_bridge::row(id).is_some() => {
                let lq = self.live_q.as_mut().unwrap();
                let a = super::live_bridge::basic_words(lq, args, &a);
                return super::vslot_bridge::try_live(lq, id, m.object, &a)
                    .ok_or_else(|| SpliceError::HostUnimplemented(format!("method {id:#x} (vslot)")));
            }
            // WP-J7: class-runtime rows (methods::cls)
            id if self.sched.cls.call(id, desc.map(|d| d.callee).unwrap_or(0), m.object, args, a.clone()) => {}
            // spawn alias methods and anything else: J3 host
            _ if methods::IMPLEMENTED.contains(&m.method_id) => return self.level.call_method(m, args),
            // WIRE-UI: a load-time setter without a built live store: kept for the first host that has one
            _ if LIVE_DEFERRED.contains(&desc.map(|d| d.callee).unwrap_or(0)) && !self.live_ready() => {
                self.sched.live_deferred.push((desc.unwrap().callee, m.object, a.clone(), args.to_vec()));
            }
            // WP-TR1: bridge to QMem-ported callees via LiveQ store
            id if self.live_q.is_some() && desc.map(|d| d.callee).unwrap_or(0) != 0 => {
                let callee = desc.unwrap().callee;
                let lq = self.live_q.as_mut().unwrap();
                let a = super::live_bridge::basic_words(lq, args, &a);
                match super::live_bridge::try_method(lq, callee, m.object, &a) {
                    Some(Ok(r)) => return Ok(r),
                    _ => return Err(SpliceError::HostUnimplemented(format!("method {id:#x} -> FUN_{callee:08x}"))),
                }
            }
            id => {
                let callee = desc.map(|d| d.callee).unwrap_or(0);
                return Err(SpliceError::HostUnimplemented(format!("method {id:#x} -> FUN_{callee:08x}")));
            }
        }
        Ok(Ref::Void)
    }

    fn ref_op_get_o(&mut self, a: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x00124048 -> FUN_00123eb0(kind 0x6f) -> FUN_00200ba0(obj = argv[0]+4, symid = argv[1]+4)
        self.sched.get_o_calls += 1;
        if self.sched.get_o_calls > SCRIPT_WATCHDOG {
            return Err(SpliceError::Malformed("script watchdog (get-o calls of one world update)"));
        }
        let obj = a.first().map(|r| r.payload()).unwrap_or(0);
        let sym = a.get(1).map(|r| r.payload()).unwrap_or(0);
        // CRAWL R110: class-114 sequence field 620 (desc 0x2b2960: kind 0x2002, +0x6c) = the FUN_001dfd58 subject
        // (C2d8c48::f_6c, a handle), e.g. the safehouse exit seq 27254 started by 141710 (case 0xb, rec[5] = Sly):
        // its script does (set-o (get-o seq 643) 806 1) -> Sly takes the focus -> hub +0x27c(0) (rec_ep1b 065:
        // 27254 +0x6c = Sly, ctl 7, hub +0x180 0). The generic member read returned Nil. Opt-out SLY2_NO_R110_SEQ6C.
        if self.class_of(obj) == Some(114) && super::fields::ee_sub_1ffcf8_field(114, sym as u16) == Some(620) && std::env::var_os("SLY2_NO_R110_SEQ6C").is_none() {
            if let Some(set) = self.sched.cls.set.as_ref() {
                if let Some((_, crate::engine::classes::Inst::C2d8c48(c))) = set.more.iter().find(|x| x.0 == obj as usize) {
                    return Ok((c.f_6c as u32).checked_sub(1).map(Ref::Basic).unwrap_or(Ref::Nil));
                }
            }
        }
        // POOL1 r30: field 251 of the ASEGA running a script message = its +0x1c owner (FUN_0012f9b8 param_1)
        // ANIM-ALL R94: field 481 (SMA field 375, getter FUN_001f07e8 = the current state's label) of a LIVE-image SMA
        // running its script message (self = its guest address): the live FUN_001f07e8, e.g. the safehouse hub select
        // SMA (SM 27339, labels 7 / 9 / 10 / 8) whose s8 picks the leaning pose 5525 for the selected character (label
        // 7 = Sly, PCSX2 cap_ep2_table_seated 001: Sly SM 419 state 3 ASEG 4542). The k25 getter below answered -1
        // (no port SMA) -> everyone 5526. Opt-out SLY2_R94_NO_LIVESMA481.
        if sym == 481 && std::env::var_os("SLY2_R94_NO_LIVESMA481").is_none() {
            if let Some((h, _)) = self.sched.cls.host.aseg_script_self {
                if obj == h && obj >= crate::engine::classes::q::live::HEAP_BASE && self.live_ready() {
                    let mem = self.live_q.as_mut().unwrap().mem.as_mut().unwrap();
                    let l = mem.call_top(0x001f_07e8, &[h]).map_or(-1, |x| x as i32);
                    return Ok(Ref::S32(l));
                }
            }
        }
        if sym == 251 {
            if let Some((h, owner)) = self.sched.cls.host.aseg_script_self {
                if obj == h {
                    return Ok(owner.checked_sub(1).map(Ref::Basic).unwrap_or(Ref::Nil));
                }
            }
        }
        // HEIST: field 250 of that ASEGA = its ASEG object (the valve scripts' (find 142138 (get self 250)))
        if sym == 250 {
            if let Some((h, _)) = self.sched.cls.host.aseg_script_self {
                if obj == h {
                    return Ok(Ref::Basic(self.sched.cls.host.heist_aseg_obj));
                }
            }
        }
        if sym < 0x4d1 {
            if let Some(class) = self.class_of(obj) {
                if let Some(r) = get_o_method(obj, class, sym as u16) {
                    return Ok(r);
                }
            }
        }
        // WP-J5: ordinary fields (FUN_001ffd98 other kinds) and user properties (FUN_0012ab28)
        let class = self.class_of(obj);
        // WP-J9: task +0x7c (the owning job) lives in the job manager, not in the object's props
        if class == Some(133) && sym == SYMID_TASK_JOB && !self.sched.objs.members.contains_key(&(obj, 1264)) {
            if let Some(&(_, j)) = self.sched.jobs.task_job.iter().find(|t| t.0 == obj) {
                return Ok(Ref::Basic(j));
            }
        }
        // ANIM-ALL R39: task +0x64 enter count (field 1267, symid 1078) likewise: the EP4 task 2399 msg-0x13 handler
        // starts seq 2351 (FUN_00106688) only when it is 1 (emu_play_ep4 024 -> 025: +0x64 0 -> 1)
        if class == Some(133) && super::fields::ee_sub_1ffcf8_field(133, sym as u16) == Some(1267) && !self.sched.objs.members.contains_key(&(obj, 1267)) {
            if let Some(&(_, n)) = self.sched.jobs.task_n64.iter().find(|t| t.0 == obj) {
                return Ok(Ref::S32(n));
            }
        }
        // HEIST r28: field 18 (symid 31, flags 4) getter FUN_001cb658: the object's +0x3c alias node with +0x24 set (a
        // pool copy handed out by FUN_001caec8) -> entry +0x68 per-copy record [copy - entry +0xc] +0 = the spawner
        // that claimed it (FUN_001caec8 *slot = spawner); else 0. REPLACED: the record owner is the classes host's
        // `pool_owner` ((entry, copy) -> spawner handle); a copy never handed out reads null
        if class.is_some_and(|c| super::fields::ee_sub_1ffcf8_field(c, sym as u16) == Some(18)) && !self.sched.objs.members.contains_key(&(obj, 18)) {
            if let Some(w) = self.world {
                let e = w.objects.get(obj as usize).map(|o| o.entry);
                let host = &self.sched.cls.host;
                let sp = e.and_then(|e| w.entries.get(e).map(|en| (e, en))).and_then(|(e, en)| en.instances.iter().position(|&o| o == obj as usize).and_then(|k| host.pool_owner.get(&(e, k as i32)).copied()));
                return Ok(sp.map_or(Ref::Nil, |h| Ref::Basic(h - 1)));
            }
        }
        // CRAWL R95: field 447 (class 24 trigger volume, symid 534) = +0x1d8, the first object FUN_00149440 found
        // inside (the pump volume's msg 5 removes the thrown guard with ((get self 534) . 40)); SLY2_NO_R95_VOL1D8=1 = old
        if class == Some(24) && (sym == 534 || super::fields::ee_sub_1ffcf8_field(24, sym as u16) == Some(447)) && !self.sched.objs.members.contains_key(&(obj, 447)) && std::env::var_os("SLY2_NO_R95_VOL1D8").is_none() {
            if let Some(v) = self.sched.cls.set.as_ref().and_then(|s| s.k24v.iter().find(|(i, _)| *i == obj as usize)) {
                return Ok(if v.1.f_1d8 == 0 { Ref::Nil } else { Ref::Basic(v.1.f_1d8 - 1) });
            }
        }
        // WP-J11: task +0x54 state (field 1286) likewise
        if class == Some(133) && super::fields::ee_sub_1ffcf8_field(133, sym as u16) == Some(1286) && !self.sched.objs.members.contains_key(&(obj, 1286)) {
            if let Some(&(_, s)) = self.sched.jobs.task_state.iter().find(|t| t.0 == obj) {
                return Ok(Ref::S32(s));
            }
            // WP-J12: no job-manager task: the class-133 instance's +0x54
            if let Some(v) = self.sched.cls.set.as_ref().and_then(|s| crate::engine::classes::k4::c133_view(&s.more, obj + 1)) {
                return Ok(Ref::S32(v.f_54));
            }
        }
        // WP-J12: field 1268 (symid 0x438, kind 0, flags 4): getter FUN_00164030 (task done: +0x54 state
        // >= 2, or the +0xd8 save word while it is -1); the state as for field 1286 above, else the
        // class-133 instance, else -1 (the FUN_001635a0 init value)
        // POOL1 R73: field 1251 (job class 132, symid 1080, flags 4): getter FUN_00163400 = job +0x68 (end task)
        // != 0 && FUN_00164030(end task) (the field 1268 getter below on that task); e.g. the Paris hub script
        // picks its guard spawner set (123842/123843 vs 123844) by (get-o job1771 1080)
        let (obj, class, sym) = if class == Some(132) && super::fields::ee_sub_1ffcf8_field(132, sym as u16) == Some(1251) && !self.sched.objs.members.contains_key(&(obj, 1251)) {
            match self.sched.jobs.job_end.iter().find(|j| j.0 == obj) {
                Some(&(_, t)) => (t, Some(133u16), FIELD_TASK_DONE_SYM),
                None => return Ok(Ref::Bool(false)),
            }
        } else {
            (obj, class, sym)
        };
        if class == Some(133) && super::fields::ee_sub_1ffcf8_field(133, sym as u16) == Some(1268) && !self.sched.objs.members.contains_key(&(obj, 1268)) {
            let cls = &mut self.sched.cls;
            let view = cls.set.as_ref().and_then(|s| crate::engine::classes::k4::c133_view(&s.more, obj + 1));
            let inst = view.as_ref().map(|v| v.f_54).unwrap_or(-1);
            let mut v = view.unwrap_or_default();
            v.f_54 = self.sched.jobs.task_state.iter().find(|t| t.0 == obj).map(|t| t.1).unwrap_or(inst);
            // 0x00164030
            return Ok(Ref::Bool(crate::engine::classes::c2cf378::ee_sub_164030(&v, &mut cls.host)));
        }
        // WIRE-EP6: field 759 (JT classes 52-55, flags 4) getter FUN_00144068 (JT busy: +0x270 > 0 || +0x1084)
        if class.is_some_and(|c| super::fields::ee_sub_1ffcf8_field(c, sym as u16) == Some(FIELD_JT_BUSY)) && !self.sched.objs.members.contains_key(&(obj, FIELD_JT_BUSY)) {
            return Ok(Ref::Bool(self.sched.cls.host.jt_busy));
        }
        // WIRE-EP7: field 1252 (job class 132, flags 4) getter FUN_001633e8: DAT_003e0f9c == job (current job),
        // e.g. the cellar spawn 397's message 5 enters task 1773 only while job 1771 is current
        if class == Some(132) && super::fields::ee_sub_1ffcf8_field(132, sym as u16) == Some(FIELD_JOB_CURRENT) && !self.sched.objs.members.contains_key(&(obj, FIELD_JOB_CURRENT)) {
            let cur = self.sched.jobs.job.or_else(|| self.sched.cls.set.as_ref().map(|s| s.g.g_3e0f9c).filter(|&v| v > 0).map(|v| v as u32 - 1));
            return Ok(Ref::Bool(cur == Some(obj)));
        }
        // WIRE-EP6: field 941 (SW, flags 4) getter FUN_0020f0b0: 0 < world +0x3d78 (the K27 cine counter);
        // e.g. the mission marker's message 5 (mission_start_jt) only starts the job while it is 0
        if class == Some(CLASS_SW) && super::fields::ee_sub_1ffcf8_field(CLASS_SW, sym as u16) == Some(FIELD_SW_3D78) && !self.sched.objs.members.contains_key(&(obj, FIELD_SW_3D78)) {
            return Ok(Ref::Bool(0 < self.sched.cls.host.k27_cine[0]));
        }
        // HEIST: field 926 (SW, getter FUN_001c4840) = SW +0x3efc, the CURRENT player object, which
        // FUN_001c4940 re-points on every po switch (p20::apply_po / po_link): while another character
        // (Bentley / Murray) is controlled the getter returns it, not the level's JT (e.g. the Thunder Beak
        // marker 3687 tests `(get-o (bif117) 844)`'s oid == 8)
        if class == Some(CLASS_SW) && super::fields::ee_sub_1ffcf8_field(CLASS_SW, sym as u16) == Some(super::fields::FIELD_SW_PLAYER) && !self.sched.objs.members.contains_key(&(obj, super::fields::FIELD_SW_PLAYER)) {
            if let Some(po) = self.sched.cls.host.po.as_ref() {
                return Ok(Ref::Basic(po.obj as u32));
            }
            // CRAWL R98: the safehouse hub C2d80d0 (oid 17239, oidPoCurrent at a hub load) in its select state 1 has
            // taken the focus (slot 78(1) FUN_001c3ed8 -> FUN_001c4940(world, hub)): the getter returns the hub, so the
            // spawner pick 59.7 ((bif117) 844 oid not 7/8/9) leaves the four hub spawners 123842..5 out of the world
            // (rec5 001/002: all out, ctl 17239). Opt-out SLY2_NO_R98_HUBFOCUS.
            if std::env::var_os("SLY2_NO_R98_HUBFOCUS").is_none() {
                if let Some(lq) = self.live_q.as_mut() {
                    if let (Some(a), Some(i)) = (lq.addr_of_oid(17239), self.world.and_then(|w| w.find_oid(17239).first().copied())) {
                        use crate::engine::classes::q::q2mem::QMem;
                        if lq.mem.as_mut().is_some_and(|m| m.r32(a + 0x180) == 1) {
                            return Ok(Ref::Basic(i as u32));
                        }
                    }
                }
            }
        }
        // HEIST: field 21 (ALO pos, getter FUN_001a62e8 = world matrix row 3) of an object with a live body (a
        // K27 character body, e.g. Bentley / Murray as the po): the live position, not the load-time matrix
        if class.is_some_and(|c| super::fields::ee_sub_1ffcf8_field(c, sym as u16) == Some(super::fields::FIELD_ALO_POS)) && !self.sched.objs.members.contains_key(&(obj, super::fields::FIELD_ALO_POS)) {
            let h = &self.sched.cls.host;
            // HEIST r18: a class-69 turret (C2dbd80) under its placed root (unplaced, local xf only): world = local x the
            // owner's world, e.g. the EP2 chopper's se69 (get-o 28548 34) = the missile target point (rec10 198: 28548
            // at (2745.5, 245.1, 2413.2) = root 27891 (2745.5, 245.1, 1717.3) + local z 695.8 = launcher 9070 +0x50)
            if let Some(w) = h.world.clone().filter(|w| w.objects.get(obj as usize).is_some_and(|o| o.class == 69 && !w.life.placed.contains_key(&(obj as usize)))) {
                if let Some(o) = w.objects[obj as usize].owner {
                    let l = super::fields::ee_sub_1a62e8(&w, obj as usize);
                    if let Some(m) = self.sched.cls.host.k24_world_xf_static(o as u32 + 1) {
                        return Ok(Ref::Vector([0, 1, 2].map(|k| l[0] * m[0][k] + l[1] * m[1][k] + l[2] * m[2][k] + m[3][k])));
                    }
                }
            }
            let h = &self.sched.cls.host;
            // CRAWL R83: also a live K6 guard (e.g. a pool copy handed out by method 1019 at its spawner frame): FUN_001a62e8
            // reads its live matrix, not the copy's load-time one ((0,0,0) for an unplaced pool copy). The row-7 pump
            // generator (obj 17346 #sym1685) keeps its current guard until (19960 .108 (guard .34)) is false.
            let k6 = std::env::var_os("SLY2_NO_R83_K6POS").is_none() && self.sched.cls.set.as_ref().is_some_and(|s| s.k6_guard(obj as usize).is_some());
            if k6 || h.po.as_ref().is_some_and(|p| p.obj == obj as usize) || h.k27_po.contains_key(&(obj as usize)) {
                let hh: &mut dyn crate::engine::classes::ClassHost = &mut self.sched.cls.host;
                return Ok(Ref::Vector(hh.ee_sub_1a62e8(obj + 1)));
            }
        }
        // HEIST: field 897 (pickups 91..95 C2d5f68, kind 0x1034, flags 0x22, member +0x150) = the live pickup state
        // (-1 none, 0 idle, 1 collect, 2 collected, ...): e.g. TB task 1900 tests (get-o 123938 819) == 2
        if class.is_some_and(|c| super::fields::ee_sub_1ffcf8_field(c, sym as u16) == Some(897)) && !self.sched.objs.members.contains_key(&(obj, 897)) {
            use crate::engine::classes::Inst;
            if let Some(s) = self.sched.cls.set.as_ref().and_then(|s| s.k8.iter().find(|x| x.0 == obj as usize).and_then(|x| match &x.1 { Inst::K8(p) => Some(p.f_150), _ => None })) {
                return Ok(Ref::S32(s));
            }
        }
        // HEIST: field 1068 (classes 35 / 36, kind 0x2002, flags 2, member +0x10c) = the use point's user, written
        // by slot 50 FUN_00184130 (the press) before vt+0x1c(5): the live K10 instance's +0x10c (handle; 0 = nil)
        if class.is_some_and(|c| super::fields::ee_sub_1ffcf8_field(c, sym as u16) == Some(1068)) && !self.sched.objs.members.contains_key(&(obj, 1068)) {
            use crate::engine::classes::{k10::K10, Inst};
            let u = self.sched.cls.set.as_ref().and_then(|s| s.k10.iter().find(|x| x.0 == obj as usize).and_then(|x| match &x.1 {
                Inst::K10(k) => match &**k { K10::C2d1ee8(c) => Some(c.f_10c), K10::C2d1d00(d) => Some(d.base.f_10c), _ => None },
                _ => None,
            }));
            if let Some(u) = u {
                // FUN_00129a10: object ref; handles are index + 1 (the player JT handle maps to its object)
                let h = if u == crate::engine::classes::guard::PLAYER_HANDLE { self.world.and_then(crate::engine::game::spawn::player_object).map(|p| p as u32 + 1).unwrap_or(0) } else { u };
                return Ok(h.checked_sub(1).map(Ref::Basic).unwrap_or(Ref::Nil));
            }
        }
        // HEIST r16: SMA field 375 (sym 481; kind 6, flags 4): getter FUN_001f07e8(sma) = the current state's label
        // (-1 no live SMA), e.g. the EP2 chopper turret alternates its barrels on (get-o sma 481) == 30822
        if class == Some(CLASS_SMA) && super::fields::ee_sub_1ffcf8_field(CLASS_SMA, sym as u16) == Some(375) {
            return Ok(Ref::S32(self.sched.cls.host.k25.ee_sub_1f07e8(obj)));
        }
        // BOSS R27: class-88 TNT barrels run in the live image (pool copies placed / Added by FUN_001cb250 there, not
        // by the scripts): field 27 (FUN_00259ae8, LO +0x28 bit 60) reads the live object (+0x2c bit 28), as the
        // fuse SM's message-8 handler tests (get-o barrel 41) (the script state has the copy out of the world)
        if class == Some(88) && super::fields::ee_sub_1ffcf8_field(88, sym as u16) == Some(super::fields::FIELD_IN_WORLD) && self.live_ready() {
            let mem = self.live_q.as_mut().unwrap().mem.as_mut().unwrap();
            let addr = super::live_bridge::basic_addr(mem, obj);
            if addr != 0 {
                return Ok(Ref::Bool(crate::engine::classes::q::q2mem::QMem::r32(mem, addr + 0x2c) & 1 << 28 != 0));
            }
        }
        // BOSS R27: SMA field 371 (sym 251) = the +0x28 actor (object ref; handles are index + 1), e.g. the EP3 TNT
        // barrel fuse SM 27600's message-8 handler stops the fuse on 30756 -> 30757 unless (get-o (get-o sma 251) 41)
        if class == Some(CLASS_SMA) && super::fields::ee_sub_1ffcf8_field(CLASS_SMA, sym as u16) == Some(371) {
            if let Some(x) = self.sched.cls.host.k25.smas.get(&obj) {
                return Ok(x.obj.checked_sub(1).map(Ref::Basic).unwrap_or(Ref::Nil));
            }
        }
        // HEIST r16: field 888 (C2d5898 family, kind 0x2089, flags 2, member +0x150) = the object's embedded JOY
        // tracker (class 137): FUN_001ffd98 hands out its address; the port's handle is BASIC_JOY | object (e.g. the
        // EP2 chopper turret's message 5 tests ((get-o self 810) 909 8) = FUN_001c4728(tracker, R1) held)
        if class.is_some_and(|c| super::fields::ee_sub_1ffcf8_field(c, sym as u16) == Some(888)) && !self.sched.objs.members.contains_key(&(obj, 888)) && self.world.is_some_and(|w| (obj as usize) < w.objects.len()) {
            return Ok(Ref::Basic(super::fields::BASIC_JOY | obj));
        }
        // POOL1 r30: class-101 member rows (FUN_001ffd98 member kinds) read the live bomb instance
        if let Some(v) = self.k15_101_member(obj, class, sym, None) {
            return Ok(v);
        }
        // HEIST: sym 34 (FUN_001a62e8 position getter) of the controlled character's object = the live JT
        // position (the EP3 spotlight aims its launcher at (get target 34))
        if sym == 34 && self.world.and_then(crate::engine::game::spawn::player_object) == Some(obj as usize) {
            return Ok(Ref::Vector(ClassHost::ee_sub_1a62e8(&mut self.sched.cls.host, crate::engine::classes::guard::PLAYER_HANDLE)));
        }
        Ok(self.sched.objs.ee_sub_200ba0(self.world, obj, class, sym))
    }

    fn ref_op_set_o(&mut self, a: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x00124018 -> FUN_00123eb0(kind 0x6e) -> FUN_00200c28(obj, symid, &argv[2])
        let obj = a.first().map(|r| r.payload()).unwrap_or(0);
        let sym = a.get(1).map(|r| r.payload()).unwrap_or(0);
        let class = self.class_of(obj);
        if std::env::var_os("SLY2_R83IDLE").is_some() && (sym == 478 || class == Some(CLASS_SMA)) {
            eprintln!("R83 set-o obj {obj:#x} sym {sym} class {class:?} a {a:?}");
        }
        // POOL1 R52: SMA field 372 (sym 478; kind 6, flags 0x24): setter FUN_001f0660(sma, label) (e.g. the bomb
        // template's s8: label 111 -> goal 106 = the placed bomb's fuse)
        if class == Some(CLASS_SMA) && super::fields::ee_sub_1ffcf8_field(CLASS_SMA, sym as u16) == Some(372) {
            let label = match a.get(2) { Some(Ref::F32(f)) => *f as i32, Some(r) => r.payload() as i32, None => 0 };
            if std::env::var_os("SLY2_R83IDLE").is_some() {
                eprintln!("R83 set372 sma {obj:#x} label {label} arg {:?} live {}", a.get(2), self.sched.cls.host.k25.smas.contains_key(&obj));
            }
            if let Some(sm) = crate::engine::playerdraw::jt_sma_of(obj) {
                crate::engine::playerdraw::JT_SMA_GOALS.lock().unwrap().push((sm, label, 0));
                return Ok(Ref::Void);
            }
            ClassHost::ee_sub_1f0660(&mut self.sched.cls.host, obj as usize, label);
            return Ok(Ref::Void);
        }
        if let Some(v) = self.k15_101_member(obj, class, sym, a.get(2)) {
            return Ok(v);
        }
        if sym == 806 && std::env::var_os("R101DBG").is_some() { eprintln!("R101SET a {a:?} obj {obj} class {class:?} field {:?} want {:?} v {:?} live {}", class.and_then(|c| super::fields::ee_sub_1ffcf8_field(c, sym as u16)), crate::engine::world::field_table::field_index(0x002b_52a0), a.get(2), self.live_ready()); }
        // CRAWL R101: safehouse hub C2d80d0 (class 70) field 806 (desc 0x2b52a0: kind 0x1037 raw payload, getter
        // FUN_001c4080, setter = vtable slot +0x278/+0x27c = FUN_001d7418 -> FUN_001d7480(state) + FUN_001c3ed8
        // focus). Hub scripts 258.4 sym3 (end of the 1752 intro sequence) and 258.12 sym5 (hub load with 1752
        // done) set it to 1 = the seated select scene (rec_ep1b 064 / rec_ep1d 047+: CONTROL -> 17239). The
        // generic member write never ran the setter, so the hub never entered state 1. Opt-out SLY2_NO_R101_HUB806.
        if class == Some(70) && super::fields::ee_sub_1ffcf8_field(70, sym as u16) == Some(crate::engine::world::field_table::field_index(0x002b_52a0).unwrap_or(usize::MAX) as u16) && std::env::var_os("SLY2_NO_R101_HUB806").is_none() {
            let v = a.get(2).map_or(0, |r| r.payload());
            if self.live_ready() {
                let mem = self.live_q.as_mut().unwrap().mem.as_mut().unwrap();
                let addr = super::live_bridge::basic_addr(mem, obj);
                if addr != 0 {
                    use crate::engine::classes::q::q2mem::QMem;
                    let vt = mem.r32(addr + 0xc);
                    let f = mem.r32(vt + 0x27c);
                    mem.call(f, &[addr, v], &[]);
                }
            } else {
                self.sched.cls.host.live_calls.push((0x27c, obj + 1, v));
            }
        }
        // CRAWL R110: the same field on Sly (class 53, setter vt +0x27c = slot 78, e.g. the safehouse exit seq
        // 27254's (set-o subject 806 1)): Sly takes the control back -> FUN_001c3ed8(jt, 1) -> hub +0x27c(0) = hub
        // state 0 + select SMA retract (FUN_001f0598 msg 3 -> s3 Remove of the table cameras) (rec_ep1b 065: ctl 7,
        // hub +0x180 0). Same route as the k22 slot-78 switch to Sly (k22_take_po_switches). Opt-out SLY2_NO_R110_SLY806.
        if class == Some(crate::engine::player::p20::CLASS_SLY) && a.get(2).is_some_and(|r| r.payload() == 1) && super::fields::ee_sub_1ffcf8_field(class.unwrap(), sym as u16) == Some(crate::engine::world::field_table::field_index(0x002b_52a0).unwrap_or(usize::MAX) as u16) && std::env::var_os("SLY2_NO_R110_SLY806").is_none() {
            self.sched.cls.host.po_release = true;
            let mut done = false;
            if self.live_ready() {
                let m = self.live_q.as_mut().unwrap().mem.as_mut().unwrap();
                let jt = m.obj_addr.get(obj as usize).copied().unwrap_or(m.host_jt);
                if jt != 0 && m.hub_has_focus() {
                    crate::engine::game::q_wireq3::ee_sub_1c3ed8(m, jt, 1);
                    done = true;
                }
            }
            if !done {
                self.sched.cls.host.sly_focus_req = true;
            }
        }
        // BOSS R25: class-88 TNT barrel field 832 (sym 778, float; FUN_00200598 member store) = live-image +0x158,
        // the FUN_00197f98 pick range (entry 181: 300.0 on spawn (message 14), -5000.0 once the fuse is lit
        // (message 6); rec11 239 / 242). REPLACED: the descriptor's sub-object getter is not run; the offset is
        // the one rec11 shows.
        if class == Some(88) && super::fields::ee_sub_1ffcf8_field(88, sym as u16) == Some(832) && self.live_ready() {
            let mem = self.live_q.as_mut().unwrap().mem.as_mut().unwrap();
            let addr = super::live_bridge::basic_addr(mem, obj);
            if addr != 0 {
                crate::engine::classes::q::q2mem::QMem::w32(mem, addr + 0x158, a.get(2).map_or(0.0, |r| r.as_f32()).to_bits());
            }
        }
        // HEIST: class-112 launcher row 461 (+0x50 target, vector) on the live record (the EP3 spotlight
        // aims its launcher 9070 at the target's position before method 468 FUN_001d6b00)
        if class == Some(112) && super::fields::ee_sub_1ffcf8_field(112, sym as u16) == Some(461) {
            if let (Some(Ref::Vector(v)), Some(set)) = (a.get(2), self.sched.cls.set.as_mut()) {
                for x in set.k7.iter_mut().filter(|x| x.0 == obj as usize) {
                    if let crate::engine::classes::Inst::K7(crate::engine::classes::k7::K7::C2d7f40(c)) = &mut x.1 {
                        c.f_50 = [v[0], v[1], v[2], c.f_50[3]];
                    }
                }
            }
        }
        Ok(self.sched.objs.ee_sub_200c28(obj, class, sym, a.get(2).cloned().unwrap_or(Ref::Nil)))
    }

    fn ref_op_defer_object_update(&mut self, a: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x00123e18
        let obj = a.first().map(|r| r.payload()).unwrap_or(0);
        let d = a.get(1).map(|r| r.as_f32()).unwrap_or(0.0);
        self.sched.objs.ee_sub_123e18(self.level.base.t, obj, d);
        Ok(Ref::Void)
    }

    fn ref_op_near_clip_center(&mut self, a: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x00125168. WP-J5: inside world-update callbacks the camera is borrowed mutably by this host
        // (method calls), so the base host has none: read it from here (same g_cm, same semantics).
        match self.cm.as_deref() {
            Some(cm) => Ok(Ref::Vector(crate::engine::camera::bifs::RefOpNearClipCenter(cm))),
            None => self.level.ref_op_near_clip_center(a),
        }
    }

    fn bif98_splice_bif_98(&mut self, _: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x00123a60: S32 FUN_00210a98() = DAT_003e1c48[mission] (level table u8 #2), -1 without mission
        Ok(Ref::S32(self.mission_entry().map(|e| e.b1 as i32).unwrap_or(-1)))
    }

    fn bif99_splice_bif_99(&mut self, _: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x00123a98: S32 FUN_00210ae0() = DAT_003e1c44[mission] (level table u8 #1 = slot index)
        Ok(Ref::S32(self.mission_entry().map(|e| e.idx as i32).unwrap_or(-1)))
    }

    fn ref_op_schedule_callback(&mut self, a: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x00123bd8: call = (argv[1] (quote r) ...) for each rest element; due at t + (float)argv[0];
        // FUN_00133040(t, world, 0, call) -> S32 id
        let mut items = vec![a.get(1).cloned().unwrap_or(Ref::Nil)];
        if a.len() > 2 {
            for r in list_items(a[2].pair()) {
                items.push(list_from(vec![Ref::Tag(TAGK_QUOTE), r]));
            }
        }
        let t = self.level.base.t + a.first().map(|r| r.as_f32()).unwrap_or(0.0);
        Ok(Ref::S32(self.sched.ee_sub_133040(t, 0, list_from(items))))
    }

    fn bif112_splice_bif_112(&mut self, a: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x00124150: FUN_0015e958(world, obj ? 1 : 5, oid = argv[0]+4, obj = argv[1]+4, scope) -> object
        let Some(w) = self.world else { return self.level.bif112_splice_bif_112(a) };
        let obj = a.get(1).map(|r| r.payload()).unwrap_or(0);
        let oid = a.first().map(|r| r.payload() as i32).unwrap_or(-1);
        Ok(Ref::Basic(methods::find::ee_sub_15e958(w, if obj != 0 { 1 } else { 5 }, oid, obj)))
    }

    fn bif115_splice_bif_115(&mut self, a: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x001244e8: FUN_0015e958(world, 4, oid = argv[0]+4, obj = argv[1]+4, scope) -> object
        let Some(w) = self.world else { return self.level.bif115_splice_bif_115(a) };
        let obj = a.get(1).map(|r| r.payload()).unwrap_or(0);
        let oid = a.first().map(|r| r.payload() as i32).unwrap_or(-1);
        Ok(Ref::Basic(methods::find::ee_sub_15e958(w, 4, oid, obj)))
    }

    fn ref_op_add_o(&mut self, a: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x00123fb8 -> FUN_00123eb0(kind 0x6c) -> FUN_00200ca8(obj = argv[0]+4, symid = argv[1]+4, &argv[2])
        let obj = a.first().map(|r| r.payload()).unwrap_or(0);
        let sym = a.get(1).map(|r| r.payload()).unwrap_or(0);
        Ok(self.sched.objs.ee_sub_200ca8(obj, sym, a.get(2).cloned().unwrap_or(Ref::Nil)))
    }

    fn ref_op_ensure_o(&mut self, a: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x00123fe8 -> FUN_00123eb0(kind 0x6d) -> FUN_00200d10
        let obj = a.first().map(|r| r.payload()).unwrap_or(0);
        let sym = a.get(1).map(|r| r.payload()).unwrap_or(0);
        Ok(self.sched.objs.ee_sub_200d10(obj, sym, a.get(2).cloned().unwrap_or(Ref::Nil)))
    }

    fn bif100_splice_bif_100(&mut self, _: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x00123ad0: FUN_001294c0 = Basic(DAT_003e0f9c) (current job, world+0x3b0c; obj + 1 in ClassGlobals)
        // WP-J9: the job manager's current job (JobMgr::cur) first, the class globals second.
        if let Some(j) = self.sched.jobs.job {
            return Ok(Ref::Basic(j));
        }
        let v = self.sched.cls.set.as_ref().map(|s| s.g.g_3e0f9c).unwrap_or(0);
        Ok(Ref::Basic(if v > 0 { v as u32 - 1 } else { 0 }))
    }

    fn bif103_splice_bif_103(&mut self, _: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x00123b60: FUN_00154c90(0x2de578). A current job -> **(job+0xa8) = the job's +0xe8 save variable
        // (FUN_001d7e80); else the mean of the +8 / +0xc / +0x10 handles FUN_00154a90 case 3 makes
        // (FUN_001d88d0("suck_history", "mission_0..2")), 0.0 while any handle is 0.
        // Replaced: the job variable path (+0xe8) is not modelled by `jobs::JobMgr` -> 0.0 (fresh save).
        if self.sched.jobs.job.is_some() {
            return Ok(Ref::F32(0.0));
        }
        let Some(vars) = self.level.base.vars else { return Ok(Ref::F32(0.0)) };
        let mut s = 0.0;
        for m in ["mission_0", "mission_1", "mission_2"] {
            let p = save_path(vars, &["suck_history", m]);
            if vars.ee_sub_1d7da0(&p).is_none() {
                return Ok(Ref::F32(0.0));
            }
            // FUN_002581c0 -> CRef +4 read as float
            s += match vars.ee_sub_1d7ed8(&p, self.level.base.data) {
                super::save::SaveValue::Float(f) => f,
                super::save::SaveValue::Int(i) => f32::from_bits(i as u32),
                _ => 0.0,
            };
        }
        Ok(Ref::F32(s * 0.333_333_34))
    }

    fn bif114_splice_bif_114(&mut self, a: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x001242b8: argv[0] a list of oids (nil -> returned as is); each -> FUN_0015e958(world, obj ? 1 : 5,
        // oid, obj = argv[1]+4, scope) like BIF 112; the objects as a new list in order.
        let Some(w) = self.world else { return self.level.bif114_splice_bif_114(a) };
        let Some(l) = a.first() else { return Ok(Ref::Nil) };
        let Ref::Pair(p) = l else { return Ok(if matches!(l, Ref::Nil) { l.clone() } else { list_from(Vec::new()) }) };
        let obj = a.get(1).map(|r| r.payload()).unwrap_or(0);
        let v = list_items(Some(p.clone())).iter().map(|o| Ref::Basic(methods::find::ee_sub_15e958(w, if obj != 0 { 1 } else { 5 }, o.payload() as i32, obj))).collect();
        Ok(list_from(v))
    }

    fn bif138_splice_bif_138(&mut self, a: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x001256e0: FUN_001daea8(argv[0]+4): -1 -> false; else save variable
        // ("powerUps", "mppukfAcquired", "%d" n) (FUN_001d8a38 -> FUN_001d87c0 / FUN_001d7ed8), payload != 0.
        let n = a.first().map(|r| r.payload() as i32).unwrap_or(-1);
        let Some(vars) = self.level.base.vars.filter(|_| n != -1) else { return Ok(Ref::Bool(false)) };
        let p = save_path(vars, &["powerUps", "mppukfAcquired", &n.to_string()]);
        let on = match vars.ee_sub_1d7ed8(&p, self.level.base.data) {
            super::save::SaveValue::Int(i) => i != 0,
            super::save::SaveValue::Bool(b) => b,
            super::save::SaveValue::Float(f) => f.to_bits() != 0,
            super::save::SaveValue::Quad(_, q) => q[0] != 0,
            super::save::SaveValue::Pair(q) => q[0] != 0,
            super::save::SaveValue::Triple(q) => q[0] != 0,
            _ => false,
        };
        Ok(Ref::Bool(on))
    }

    fn ref_op_emit_smoke_cloud(&mut self, a: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x00125738 RefOpEmitSmokeCloud (BIF 139): SetTag(out, 0x24); FUN_001daf38(argv[0]+4, argv[1]+4);
        // returns Void. FUN_001daf38(a, b) builds a splice smoke particle (type 0x17) at object `a`
        // (FUN_001d8b20 / FUN_00129658 -> the particle request, NOT modelled: the recomp has no particle
        // subsystem) and, when b != 0, sets DAT_002deb3c = a and sends the camera activity message 0x3b
        // (FollowWorld_activity(0x2deb30, 0x3b, 0)); the obj is the activity subject (= DAT_002deb3c).
        // 1:1 port of the observable contract: the particle visual is unmodelled, the activity 0x3b hint
        // is routed through the class host (ee_sub_1619a0, obj = index+1 handle, matching q12_d::ee_sub_1daf38).
        // PAD1: FUN_001daf38 also writes the save variable ("powerUps", "mppukfAcquired", "%d" a) = {0x17, 1}
        // via FUN_001d8b20; this is what the task system listens on to advance task 1754.
        let obj = a.first().map(|r| r.payload()).unwrap_or(0);
        let b = a.get(1).map(|r| r.payload()).unwrap_or(0);
        if b != 0 {
            // PAD1: write save variable ("powerUps", "mppukfAcquired", obj) = Bool(true)
            // mirrors FUN_001daf38 -> FUN_001d8b20({0x17, 1}, 0x2a79a8, 0x2a79b8, sprintf("%d", a), 0)
            if let Some(vars) = self.level.base.vars {
                let p = save_path(vars, &["powerUps", "mppukfAcquired", &obj.to_string()]);
                vars.ee_sub_1d80c8(&p, super::save::SaveValue::Bool(true), self.level.base.data);
            }
            self.sched.cls.host.ee_sub_1619a0(0x3b, obj);
        }
        Ok(Ref::Void)
    }

    fn ref_op_convert_object_position(&mut self, a: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x00125000: FUN_001a6f00(from = argv[0]+4, to = argv[1]+4, *argv[2]): from != 0 -> through
        // from's world matrix (vtable +0xfc), to != 0 -> through the inverse of to's -> Vector.
        // Replaced: world matrices are the rest-pose root joints (`gfx::attach::BoneWorlds`, joint 0 =
        // object placement as `gfx::scene` uses it) / class-3 XFM matrices; others identity.
        let Some(w) = self.world else { return self.level.ref_op_convert_object_position(a) };
        let Some(Ref::Vector(mut p)) = a.get(2).cloned() else { return Err(SpliceError::Type("vector", a.get(2).map(|r| r.tagk()).unwrap_or(-1))) };
        let mut bw = crate::engine::gfx::attach::BoneWorlds::default();
        // HEIST: the class host's world matrix (vtable +0xfc as K24 sees it: placements, nested placed parents,
        // live K27 bodies) when it has the world; else the rest-pose fallback
        let host_ok = self.sched.cls.host.world.is_some();
        let cls = &mut self.sched.cls.host;
        let mut world_of = |o: u32| -> crate::engine::gfx::scene::Affine {
            let i = o as usize;
            if host_ok {
                if let Some(mut m) = cls.k24_world_xf(o + 1) {
                    let hh: &mut dyn crate::engine::classes::ClassHost = cls;
                    m[3] = hh.ee_sub_1a62e8(o + 1);
                    return m;
                }
            }
            match crate::engine::gfx::attach::alo_of(w, i) {
                Some(al) if !al.base.bones.is_empty() => bw.get(w, i, 0),
                _ => bw.xfm_world(w, i).unwrap_or(crate::engine::gfx::scene::IDENTITY),
            }
        };
        let xf = |m: &crate::engine::gfx::scene::Affine, p: [f32; 3]| [0, 1, 2].map(|j| p[0] * m[0][j] + p[1] * m[1][j] + p[2] * m[2][j] + m[3][j]);
        let (from, to) = (a.first().map(|r| r.payload()).unwrap_or(0), a.get(1).map(|r| r.payload()).unwrap_or(0));
        if from != 0 {
            p = xf(&world_of(from), p);
        }
        if to != 0 {
            p = xf(&crate::engine::gfx::scene::invert(&world_of(to)), p);
        }
        Ok(Ref::Vector(p))
    }

    fn ref_op_convert_object_vector(&mut self, a: &[Ref]) -> Result<Ref, SpliceError> {
        // WP-J11: 0x00125060: FUN_001a6fc0(from = argv[0]+4, to = argv[1]+4, *argv[2]): from != 0 -> the
        // rotation of from's world matrix (FUN_001a6318), to != 0 -> its transpose (FUN_0011b340) for
        // `to`; no translation -> Vector. Replaced world matrices as `ref_op_convert_object_position`.
        let Some(w) = self.world else { return self.level.ref_op_convert_object_vector(a) };
        let Some(Ref::Vector(mut p)) = a.get(2).cloned() else { return Err(SpliceError::Type("vector", a.get(2).map(|r| r.tagk()).unwrap_or(-1))) };
        let mut bw = crate::engine::gfx::attach::BoneWorlds::default();
        // HEIST: the class host's world matrix (vtable +0xfc as K24 sees it: placements, nested placed parents,
        // live K27 bodies) when it has the world; else the rest-pose fallback
        let host_ok = self.sched.cls.host.world.is_some();
        let cls = &mut self.sched.cls.host;
        let mut world_of = |o: u32| -> crate::engine::gfx::scene::Affine {
            let i = o as usize;
            if host_ok {
                if let Some(mut m) = cls.k24_world_xf(o + 1) {
                    let hh: &mut dyn crate::engine::classes::ClassHost = cls;
                    m[3] = hh.ee_sub_1a62e8(o + 1);
                    return m;
                }
            }
            match crate::engine::gfx::attach::alo_of(w, i) {
                Some(al) if !al.base.bones.is_empty() => bw.get(w, i, 0),
                _ => bw.xfm_world(w, i).unwrap_or(crate::engine::gfx::scene::IDENTITY),
            }
        };
        let (from, to) = (a.first().map(|r| r.payload()).unwrap_or(0), a.get(1).map(|r| r.payload()).unwrap_or(0));
        if from != 0 {
            let m = world_of(from);
            p = [0, 1, 2].map(|j| p[0] * m[0][j] + p[1] * m[1][j] + p[2] * m[2][j]);
        }
        if to != 0 {
            let m = world_of(to);
            p = [0, 1, 2].map(|j| p[0] * m[j][0] + p[1] * m[j][1] + p[2] * m[j][2]);
        }
        Ok(Ref::Vector(p))
    }

    fn bif101_splice_bif_101(&mut self, _: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x00123b00: Basic(DAT_003e0fa0) (current task, world+0x3b10; obj + 1 in ClassGlobals)
        if let Some(t) = self.sched.jobs.task {
            return Ok(Ref::Basic(t));
        }
        let v = self.sched.cls.set.as_ref().map(|s| s.g.g_3e0fa0).unwrap_or(0);
        Ok(Ref::Basic(v.saturating_sub(1)))
    }

    fn bif102_splice_bif_102(&mut self, _: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x00123b30: Basic(DAT_003e0fa4) (world+0x3b14 restart task override, JobMgr::restart_override
        // via `JobView`; none -> null object)
        Ok(Ref::Basic(self.sched.jobs.restart.unwrap_or(0)))
    }

    fn bif133_splice_bif_133(&mut self, _: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x00125510: inherited lookup of argv[0] (FUN_00127db8(frame, symid, 1)) from the parent of the
        // nearest caller frame with +0x24 set (an object's script frame); not found -> Bifk 0x86 (BIF 134,
        // the void no-op). Replaced: the port runs every script block in the level frame (no per-object
        // frames, and the host gets no caller frame), so the lookup has no parent frame -> Bifk 0x86.
        Ok(Ref::Bifk(0x86))
    }

    fn ref_op_find_class_objects(&mut self, a: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x00124730: FUN_0015e9c8(world, obj ? 0x401 : 0x405, cid = argv[0]+4, obj = argv[1]+4, 0x180)
        // -> FUN_00124078 (list of objects, count 0 = empty list)
        let Some(w) = self.world else { return self.level.ref_op_find_class_objects(a) };
        let cid = a.first().map(|r| r.payload() as i32).unwrap_or(-1);
        let obj = a.get(1).map(|r| r.payload()).unwrap_or(0);
        let v = methods::find::ee_sub_15e9c8(w, if obj != 0 { 0x401 } else { 0x405 }, cid, obj, 0x180);
        Ok(list_from(v.into_iter().map(Ref::Basic).collect()))
    }

    fn ref_op_find_objects_in_bounding_box(&mut self, a: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x001247d8: FUN_0020faa0(0x3dd490, *argv[0], *argv[1], 0, 0): the collision entries (max 0x100)
        // of the +0x3d1c tree nodes whose box overlaps [min, max] of the two vectors; FUN_0020fdc8: their
        // SOs (entry +4), each once (SO +0x148 bit 30 mark), in entry order; FUN_00124078 -> object list.
        // REPLACED: the tree is the flat SO collider list (`CollisionWorld::build_so`, the J7-lent one when
        // built), its order stands for the tree walk order; an entry's box = its vertices' box.
        let Some(w) = self.world else { return self.level.ref_op_find_objects_in_bounding_box(a) };
        let v = |i: usize| match a.get(i) {
            Some(Ref::Vector(v)) => *v,
            _ => [0.0; 3],
        };
        let (p, q) = (v(0), v(1));
        let lo = [0, 1, 2].map(|i| p[i].min(q[i]));
        let hi = [0, 1, 2].map(|i| p[i].max(q[i]));
        let so = self.so_parts(w);
        let mut out: Vec<u32> = Vec::new();
        let mut n = 0;
        for c in &so.colliders {
            if c.geom.verts.is_empty() {
                continue;
            }
            let (mut mn, mut mx) = ([f32::MAX; 3], [f32::MIN; 3]);
            for vv in &c.geom.verts {
                for i in 0..3 {
                    mn[i] = mn[i].min(vv[i]);
                    mx[i] = mx[i].max(vv[i]);
                }
            }
            if (0..3).any(|i| mn[i] > hi[i] || lo[i] > mx[i]) {
                continue;
            }
            n += 1;
            if n > 0x100 {
                break;
            }
            if !out.contains(&(c.object as u32)) {
                out.push(c.object as u32);
            }
        }
        Ok(list_from(out.into_iter().map(Ref::Basic).collect()))
    }

    fn ref_op_hit_test_objects(&mut self, a: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x00124f00 -> FUN_001249a8(0x7b, argv)
        self.ee_sub_1249a8(true, a)
    }

    fn ref_op_hit_test_objects_first(&mut self, a: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x00124f80 -> FUN_001249a8(0x7c, argv)
        self.ee_sub_1249a8(false, a)
    }

    fn bif106_splice_bif_106(&mut self, a: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x00123dd8: FUN_00133130(0x3dd490, argv[0]+4) -> Void
        self.sched.ee_sub_133130(a.first().map(|r| r.payload() as i32).unwrap_or(-1));
        Ok(Ref::Void)
    }
}

/// Game-side receiver of FUN_00131290 effect calls: case 3 is the original rumble caller
/// `FUN_00187e28(duration = event +0xc, &g_pad, id = event +8)`; every other effect is kept in
/// `pending` for the systems that will own it (sound/camera/script effects not identified yet).
pub struct GameEffectSink<'a> {
    pub rumble: &'a mut Vec<super::splice_host::RumbleRequest>,
    pub pending: Vec<crate::engine::anim::events::EffectCall>,
}

impl crate::engine::anim::events::EffectSink for GameEffectSink<'_> {
    fn call(&mut self, _a: &mut crate::engine::anim::play::Asega, c: crate::engine::anim::events::EffectCall) {
        use crate::engine::anim::events::EffectCall;
        match c {
            // 0x00131290 case 3
            EffectCall::E3 { a, t } => self.rumble.push(super::splice_host::RumbleRequest { pattern: a as usize, duration: t.unwrap_or(0.0) }),
            other => self.pending.push(other),
        }
    }
}

/// WP-J9: FUN_001d87c0 name path: FUN_001d8720(0x3e0eb0, name) per name, -1 -> FUN_00233800 (atoi).
pub fn save_path(vars: &super::save::SaveVars, names: &[&str]) -> Vec<u16> {
    // 0x001d87c0
    names
        .iter()
        .map(|n| match vars.ee_sub_1d8720(n) {
            -1 => {
                let t = n.trim_start();
                let e = t.char_indices().take_while(|&(i, c)| c.is_ascii_digit() || (i == 0 && (c == '-' || c == '+'))).last().map_or(0, |(i, _)| i + 1);
                t[..e].parse::<i32>().unwrap_or(0) as u16
            }
            s => s as u16,
        })
        .collect()
}
