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
use crate::engine::disc::descriptor::LevelTable;
use crate::engine::splice::eval::Vm;
use crate::engine::splice::frame::FrameRef;
use crate::engine::splice::value::{list_from, list_items, Method, Ref, TAGK_QUOTE};
use crate::engine::splice::{SpliceError, SpliceHost};
use crate::engine::world::World;

/// Class ids of the engine globals scripts get as Basic refs (class info +4).
pub const CLASS_SW: u16 = 1; // 0x3dd490 (size 0x4790)
pub const CLASS_CM: u16 = 135; // g_cm 0x2ddf40 (binds 0x1026e0 / 0x102768)
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
    /// WP-J10: script messages (object, message) waiting for the script tick (`job_messages`)
    pub msgs: Vec<(u32, u32)>,
    /// WP-J10: `jobs::JobMgr::calls` already turned into messages + the job / task current then
    pub job_seen: usize,
    /// WP-J15: `JobMgr::task_msgs` already delivered
    pub task_msg_seen: usize,
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
}

impl JobView {
    pub fn of(m: &crate::engine::jobs::JobMgr) -> Self {
        let t = |i: Option<usize>| i.and_then(|i| m.tasks.get(i)).map(|t| t.obj as u32);
        let task_job = m.tasks.iter().filter_map(|t| Some((t.obj as u32, m.jobs.get(t.job?)?.obj as u32))).collect();
        let task_state = m.tasks.iter().map(|t| (t.obj as u32, t.state)).collect();
        JobView { job: m.cur.and_then(|i| m.jobs.get(i)).map(|j| j.obj as u32), task: t(m.cur_task), restart: t(m.restart_override), task_job, task_state }
    }
}

/// WP-J15: the job manager method rows of the last script ticks (`ClassLink::job_calls`): method 1258
/// FUN_00162660(job, aseg, b0, b4) and method 1284 FUN_00165118(task, arg) on the job / task of the object;
/// WP-EP1b: method 1239 FUN_00162df8(job) (restart the job at its start task or the +0x3b14 override).
pub fn apply_job_calls(sched: &mut Schedule, m: &mut crate::engine::jobs::JobMgr) {
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
    if m.calls.len() < sched.job_seen {
        sched.job_seen = 0; // new world: JobMgr::load
        sched.job_prev = (None, None);
        sched.task_msg_seen = 0;
    }
    if m.task_msgs.len() < sched.task_msg_seen {
        sched.task_msg_seen = 0;
    }
    let v = JobView::of(m);
    let (pj, pt) = sched.job_prev;
    for &(f, msg) in &m.calls[sched.job_seen..] {
        let o = match (f, msg) {
            (0x001629a0, 0x13) => v.job,
            (0x00163d18, 0x13) => v.task,
            (0x00162a60, 0x14) => if pj != v.job { pj } else { v.job },
            (0x00163f48, 0x14) => if pt != v.task { pt } else { v.task },
            _ => None,
        };
        if let Some(o) = o {
            sched.msgs.push((o, msg as u32));
        }
    }
    sched.job_seen = m.calls.len();
    for &(t, msg) in &m.task_msgs[sched.task_msg_seen.min(m.task_msgs.len())..] {
        if let Some(tk) = m.tasks.get(t) {
            sched.msgs.push((tk.obj as u32, msg));
        }
    }
    sched.task_msg_seen = m.task_msgs.len();
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
pub fn ee_sub_1331f8(vm: &mut Vm, host: &mut PersistentHost, frame: &FrameRef, obj: u32, msg: u32, args: &[Ref]) -> Result<Option<Ref>, SpliceError> {
    // 0x001331f8
    if crate::engine::splice::frame::lookup(frame, msg, true).is_none() {
        return Ok(None);
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
    let lo = w.objects.get(obj as usize)?;
    let slot = usize::try_from(lo.parent_slot).ok()?;
    frames.get(lo.entry)?.get(slot).cloned()
}

/// WP-J10: deliver the pending script messages (`Schedule::msgs`) of the job code.
pub fn send_messages(vm: &mut Vm, errors: &mut Vec<SpliceError>, frames: &[Vec<FrameRef>], host: &mut PersistentHost) {
    let msgs = std::mem::take(&mut host.sched.msgs);
    let Some(w) = host.world else { return };
    for (obj, msg) in msgs {
        if let Some(fr) = object_frame(frames, w, obj) {
            if let Err(e) = ee_sub_1331f8(vm, host, &fr, obj, msg, &[]) {
                errors.push(e);
            }
        }
    }
    // WP-K28: level-script messages of the class code with arguments (sequence action 9, handle = index + 1)
    for (h, msg, words) in std::mem::take(&mut host.sched.cls.host.k27_script_msgs) {
        let (Some(obj), Some(args)) = (h.checked_sub(1), crate::engine::classes::k28::ee_sub_200da0(msg, &words)) else { continue };
        if let Some(fr) = object_frame(frames, w, obj) {
            if let Err(e) = ee_sub_1331f8(vm, host, &fr, obj, msg, &args) {
                errors.push(e);
            }
        }
    }
}

/// The script work of one FUN_00226550 world update at its original positions: FUN_00149c80 at the
/// start (always), then, only when dt > 0, after the object updates and once more at the end.
pub fn tick_world_update(vm: &mut Vm, errors: &mut Vec<SpliceError>, host: &mut PersistentHost, t: f32, dt: f32) {
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
            b if b & BASIC_EMITTER != 0 => Some(CLASS_EMITTER),
            // WP-J9: getter-created run-time records (ASEGA / link / class 147)
            b if b & super::fields::BASIC_RT != 0 => self.sched.objs.rt_get(b).map(|r| r.class),
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
            return methods::getters::call(self, g, m.object, &a);
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
            // WP-J10: 0x00104bf8 -> FUN_001f0598 on a link record (class 136, `fields::RtRecord`)
            0x0010_4bf8 if self.sched.objs.ee_sub_1f0598(m.object) => {}
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
            }
            // WP-J8: 0x00100fe0 -> FUN_001f7918(alo, mask (kind 0x1011), byte (kind 0x1012))
            0x0010_0fe0 => {
                let w = |i: usize| match a.get(i) {
                    Some(MArg::Word(w)) => *w,
                    Some(MArg::I32(n)) => *n as u32,
                    _ => 0,
                };
                self.sched.objs.ee_sub_1f7918(m.object, w(0), w(1) as u8);
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
                    g.ee_sub_1da4b8(v, int(0), t);
                } else {
                    g.ee_sub_1da578(int(0));
                }
            }
            // WP-J7: class-runtime rows (methods::cls)
            id if self.sched.cls.call(id, desc.map(|d| d.callee).unwrap_or(0), m.object, args, a.clone()) => {}
            // spawn alias methods and anything else: J3 host
            _ if methods::IMPLEMENTED.contains(&m.method_id) => return self.level.call_method(m, args),
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
        if class == Some(133) && super::fields::ee_sub_1ffcf8_field(133, sym as u16) == Some(1268) && !self.sched.objs.members.contains_key(&(obj, 1268)) {
            let cls = &mut self.sched.cls;
            let view = cls.set.as_ref().and_then(|s| crate::engine::classes::k4::c133_view(&s.more, obj + 1));
            let inst = view.as_ref().map(|v| v.f_54).unwrap_or(-1);
            let mut v = view.unwrap_or_default();
            v.f_54 = self.sched.jobs.task_state.iter().find(|t| t.0 == obj).map(|t| t.1).unwrap_or(inst);
            // 0x00164030
            return Ok(Ref::Bool(crate::engine::classes::c2cf378::ee_sub_164030(&v, &mut cls.host)));
        }
        Ok(self.sched.objs.ee_sub_200ba0(self.world, obj, class, sym))
    }

    fn ref_op_set_o(&mut self, a: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x00124018 -> FUN_00123eb0(kind 0x6e) -> FUN_00200c28(obj, symid, &argv[2])
        let obj = a.first().map(|r| r.payload()).unwrap_or(0);
        let sym = a.get(1).map(|r| r.payload()).unwrap_or(0);
        let class = self.class_of(obj);
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

    fn ref_op_convert_object_position(&mut self, a: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x00125000: FUN_001a6f00(from = argv[0]+4, to = argv[1]+4, *argv[2]): from != 0 -> through
        // from's world matrix (vtable +0xfc), to != 0 -> through the inverse of to's -> Vector.
        // Replaced: world matrices are the rest-pose root joints (`gfx::attach::BoneWorlds`, joint 0 =
        // object placement as `gfx::scene` uses it) / class-3 XFM matrices; others identity.
        let Some(w) = self.world else { return self.level.ref_op_convert_object_position(a) };
        let Some(Ref::Vector(mut p)) = a.get(2).cloned() else { return Err(SpliceError::Type("vector", a.get(2).map(|r| r.tagk()).unwrap_or(-1))) };
        let mut bw = crate::engine::gfx::attach::BoneWorlds::default();
        let mut world_of = |o: u32| -> crate::engine::gfx::scene::Affine {
            let i = o as usize;
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
        let mut world_of = |o: u32| -> crate::engine::gfx::scene::Affine {
            let i = o as usize;
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
