//! WP-J7: script method rows whose engine side belongs to the class runtime (`classes::ClassSet`).
//!
//! `ClassLink` lends the level's `ClassSet` (+ its `EngineHost`) to the script host for the duration of
//! the script tick (`app::ScriptClassesPlugin` moves it in / out around `classes::plugin::tick_classes`).
//! Method thunks (and world+0x3ca8 deferred nodes with a direct fn) are dispatched by their engine
//! callee into the ported instance of the object (`Inst`, object index = Basic payload, handles = index + 1).
//!
//! 1:1 here: FUN_001dfd58 (class 114 start) with FUN_001e02c0 / FUN_001e5ae0 (sequence node creation),
//! FUN_001dfe80, FUN_0015c150 (zeroed request -> slot 51 FUN_0015c690), FUN_0015a150, FUN_00162dd0.
//! Dispatched to existing class ports: FUN_001dfeb8 (c2d8c48), FUN_002283d0 (c2dc6f0, + FUN_002280d0
//! hand-over chain as `frame_more`), FUN_00164640 (ClassHost hook), FUN_00171cf8 (guard routine, WP-J9).
//! Recorded (owning subsystem not ported / world is read-only here, see `RECORDED`): the call is kept in
//! `ClassLink::recorded` in order, like J6's `dispatched`.

pub mod s1;

use super::MArg;
use crate::engine::classes::plugin::EngineHost;
use crate::engine::classes::{c2d8c48::C2d8c48, ClassHost, ClassSet, Inst};
use crate::engine::splice::value::Ref;

/// One FUN_001e5ae0 sequence node (0xf0 B, pool 0x3e1560, vtable 0x2d8c08).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SeqNode {
    /// +0x14 owner (class-114 object handle)
    pub owner: u32,
    /// +0x18 context object (param_3 == 0: the owner, else param_3 +0x18)
    pub ctx: u32,
    /// +0x1c FUN_0015e938(world, 5, record +0x10, 0) (0 when the record word is -1)
    pub target: u32,
    /// +0x28 first record / +0x2c matching end record (indices into C2d8c48::f_54)
    pub first: usize,
    pub last: usize,
}

/// Handles of `SeqNode`s in `C2d8c48::f_5c` (index into `ClassLink::nodes` | this bit).
pub const NODE_HANDLE: u32 = 0x4000_0000;

/// A method / deferred call kept for its (unported) owner.
#[derive(Clone, Debug)]
pub struct Recorded {
    pub thunk: u32,
    pub callee: u32,
    /// object index (Basic payload)
    pub obj: u32,
    pub args: Vec<MArg>,
}

/// The class runtime as seen by the script host.
#[derive(Clone, Debug, Default)]
pub struct ClassLink {
    /// the level's class instances (None = not lent: class rows raise HostUnimplemented as before J7)
    pub set: Option<ClassSet>,
    pub host: EngineHost,
    /// FUN_001e5ae0 nodes
    pub nodes: Vec<SeqNode>,
    pub recorded: Vec<Recorded>,
    /// WP-J11: guard +0xeb8 oid list / +0xedc resolved objects (`GuardOids`), keyed by object index
    pub guard_oids: std::collections::HashMap<u32, GuardOids>,
    /// WP-J12: SetJtJts(jt, jts, -1) requests of method 0x0010b028 on class-53 (JT) objects: (object index,
    /// jts). Not drained yet: the player layer owns FUN_0018c250 (`player::Jt::set_jts`).
    pub jt_states: Vec<(u32, i32)>,
    /// WP-J15: job manager method rows (callee, object index, int args): 0x162660 (method 1258, class 132)
    /// / 0x165118 (method 1284, class 133); applied to `jobs::JobMgr` by `vm_state::apply_job_calls`.
    pub job_calls: Vec<(u32, u32, Vec<i32>)>,
    /// WP-EP1d: FUN_002283d0 targets of the job code (job +0xcc in FUN_00162fd0, task +0xc4 in the
    /// FUN_00163d18 enter): oids from `JobMgr::calls`, run on the class-26 warp instances by `drain_seq_starts`
    pub warp_oids: Vec<i32>,
}

/// WP-J11: guard (class 57 family) +0xeb8 oids (count +0xed8, cap 8) and +0xedc objects (count +0xefc)
/// of FUN_0020aa40 / FUN_0020aaa8 / FUN_0020ab58. REPLACED: kept beside the guard (`classes::guard::c56`
/// has no such fields; their guard-side readers are not ported); objects = Basic payloads (index).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GuardOids {
    pub eb8: Vec<i32>,
    pub edc: Vec<u32>,
    /// WP-J12: +0x3ac classes (count +0x3a8, cap 5) of FUN_00208920 / FUN_00208948 / FUN_002089e8
    pub f_3ac: Vec<i32>,
    /// WP-J12: +0x3c4 classes (count +0x3c0, cap 2) of FUN_00208a68 / FUN_00208a90 / FUN_00208b30
    pub f_3c4: Vec<i32>,
}

/// Engine functions recorded only (callee address): 0x00142540 / 0x001b9ca0 (FUN_00205438 sub-object of
/// classes 52-55), 0x001a58d0 (vtable +0x54 SetParent, mutates the hierarchy), 0x00206820 / 0x0020ab58 (guard vtable calls / +0xefc +0xed8), 0x00212e40 (class 126-131
/// sequence lookup), 0x001f6b48 / 0x001f6bc8 (SO +0xe4 0xb0-byte records), 0x0013b860 (classes 79-82),
/// and the inline thunks 0x00100a00 (vtable +0x134) / 0x0010daf0 (vtable +0x18c, -1).
/// WP-J10 adds: 0x001cb488 / 0x001cb4b8 (class 6 +0xe0 attachment record +8 = 1 / 0, FUN_001cb410; the
/// ClassHost hook is not modelled), 0x0013b5c8 (classes 79-82 +0x154 = min(+0x154, now + arg)),
/// 0x00207bc8 (guards 57 / 63 / 64: FUN_00207b10 / FUN_00209c18 / FUN_00207b70 / FUN_00209130),
/// 0x001f7bf8 (class 48 +0x130 / +0x12c rate with
/// vtable +0x44 / +0x4c; `C2d9978` has no +0x130), and the inline thunks 0x0010fc80 (classes 72 / 73
/// vtable +0x29c) / 0x00104e80 (class 82 vtable +0x274 with a vector out-arg).
/// WP-EP1f adds 0x00195608 (player method 861: drop the +0x4680 0x30-byte interaction record of the object
/// from the player's +0x4744 prompt list; the prompt list is not modelled).
/// WP-TR3b: removed 11 callees now routed through live_bridge (TABLE) / s1_call / CLASS_CALLEES:
/// 0x142540, 0x213b00, 0x1b9ca0, 0x212e40, 0x1cb488, 0x1cb4b8, 0x207bc8, 0x1d7780, 0x195608 (TABLE),
/// 0x1c8af0 (s1_call), 0x13b860 + 0x1a58d0 (moved to CLASS_CALLEES with run() arms).
pub const RECORDED: &[u32] = &[];
pub const RECORDED_INLINE: &[u32] = &[0x0010_0a00, 0x0010_daf0, 0x0010_db78, 0x0010_fc80, 0x0010_4e80];

/// Callees executed against the class runtime.
/// WP-J10 adds FUN_00133340 (message 5), FUN_001baed8 / FUN_001baf18 (class 98, `C2d4b78`),
/// FUN_001dffe0 / FUN_001e0760 (class 114, `C2d8c48`).
/// WP-EP1b adds FUN_0013b5c8 (method 389, classes 79 / 80 / 82 fuse, `k15_79::Proj::ee_sub_13b5c8`).
/// WP-EP1f adds FUN_00228170 (thunk 0x10b760, field 778: the warp's "go", `ClassHost::ee_sub_228170`).
pub const CLASS_CALLEES: &[u32] =
    &[0x0022_8170, 0x001d_fd58, 0x001d_fe80, 0x001d_feb8, 0x0022_83d0, 0x0015_c150, 0x0015_a150, 0x0016_2dd0, 0x0016_4640, 0x0017_1cf8, 0x0017_0e68, 0x0013_3340, 0x001b_aed8, 0x001b_af18, 0x001d_ffe0, 0x001e_0760, 0x001d_6b00, 0x0020_87c8, 0x0020_88e0, 0x0020_8700, 0x0020_8728, 0x0013_b5c8, 0x0013_3380, 0x001f_6ac8, 0x001f_6b48, 0x001f_6bc8, 0x001f_7c68, 0x0017_1d58, 0x0017_1da8, 0x0015_41d8,
    // WP-TR1 batch
    0x001e_0120, 0x001f_0598, 0x001f_0730, 0x001f_03a8, 0x001f_82b8, 0x001b_2020, 0x001b_2048, 0x0012_91a8, 0x0012_f990, 0x001f_78d8, 0x001f_b4f0, 0x001f_e3f8, 0x001f_e9f8, 0x001f_ea60, 0x0022_7c90, 0x001e_1970, 0x001e_1ad0, 0x001e_1b10,
    // WP-TR3b: explode projectile + SetParent
    0x0013_b860, 0x001a_58d0];

/// WP-J11: guard oid-list callees run on `ClassLink::guard_oids` (no class instance needed):
/// FUN_0020aa78 (method 0x00102988), FUN_0020ab58.
/// WP-J12 adds the guard class lists +0x3a8 (FUN_00208920 / FUN_00208948 / FUN_002089e8, methods
/// 0x001037a8 / 0x00103830 / 0x001038b8) and +0x3c0 (FUN_00208a68 / FUN_00208a90 / FUN_00208b30, methods
/// 0x00103930 / 0x001039b8 / 0x00103a40), kept on `GuardOids` like the oid list (REPLACED storage:
/// `guard::c56::Guard` has no +0x3a8 / +0x3c0 and their guard-side readers are not ported).
pub const GUARD_OID_CALLEES: &[u32] = &[0x0020_aa78, 0x0020_ab58, 0x0020_8920, 0x0020_8948, 0x0020_89e8, 0x0020_8a68, 0x0020_8a90, 0x0020_8b30];

/// WP-J11: recorded only (no model): FUN_0014b468 (method 0x0010fac0, guard brain substate {1} / {0xe, arg}
/// through FUN_00176750; `classes::guard` exposes no substate entry besides `k6_run_routine`),
/// FUN_002089e8 (method 0x001038b8, guard +0x3a8 = 0) / FUN_00208b30 (method 0x00103a40, guard +0x3c0 = 0):
/// +0x3a8 / +0x3c0 are not modelled by `guard::c56::Guard`.
/// FUN_00208920 / FUN_00208948 (methods 0x001037a8 / 0x00103830, add / remove a class on the +0x3a8 list, cap 5)
/// and FUN_00208a68 / FUN_00208a90 (methods 0x00103930 / 0x001039b8, +0x3c0 list, cap 2): same reason.
/// WP-J12: the +0x3a8 / +0x3c0 rows moved to `GUARD_OID_CALLEES`; FUN_0014b468 was recorded.
/// WP-TR3b: removed 0x14b468 — callee is in live_dispatch TABLE, routed through live_bridge.
pub const RECORDED_J11: &[u32] = &[];

/// Script object arg (kind 0x2002 Basic payload = object index) -> class runtime handle (index + 1, 0 = null).
pub fn handle_of(r: Option<&Ref>) -> u32 {
    match r {
        Some(Ref::Basic(i)) => i + 1,
        _ => 0,
    }
}

/// FUN_001e5ae0(obj, rec, parent): create a node over the record block starting at `rec` (scan for
/// the matching type-1 record, nesting on type-0 records), append it to +0x5c, init it (node vtable
/// +0xc). Returns the node handle.
pub fn ee_sub_1e5ae0(c: &mut C2d8c48, nodes: &mut Vec<SeqNode>, obj: u32, rec: usize, parent: Option<&SeqNode>, h: &mut dyn ClassHost) -> u32 {
    // 0x001e5ae0
    let ctx = parent.map(|p| p.ctx).unwrap_or(obj);
    let w4 = c.f_54.get(rec).map(|r| r[4] as i32).unwrap_or(-1);
    let target = if w4 != -1 { h.ee_sub_15e938(5, w4, 0) } else { 0 };
    // records after `rec` up to the last one: depth 1, type 0 opens, type 1 closes
    let mut last = rec;
    let mut depth = 1;
    for (k, r) in c.f_54.iter().enumerate().skip(rec + 1) {
        match r[0] {
            0 => depth += 1,
            1 => {
                depth -= 1;
                if depth == 0 {
                    last = k;
                    break;
                }
            }
            _ => {}
        }
    }
    let handle = NODE_HANDLE | nodes.len() as u32;
    nodes.push(SeqNode { owner: obj, ctx, target, first: rec, last });
    c.f_5c.push(handle); // FUN_0011a250(+0x5c, node)
    h.vcall(handle, 0xc); // (node vtable 0x2d8c08 +0xc)(node)
    handle
}

/// FUN_001dfd58(obj, a): class-114 start. Not running: +0x6c = a, running, message 5; then (still
/// running) the pending end record (+0x7c, FUN_001e02c0), the root node over the records + its first
/// update (FUN_001e1c40); no node left -> FUN_001dfeb8, else the +0x40 bit 1 / FUN_001a6ca8(obj, 1) latch.
pub fn ee_sub_1dfd58(c: &mut C2d8c48, nodes: &mut Vec<SeqNode>, i: usize, a: u32, h: &mut dyn ClassHost) {
    // 0x001dfd58
    if c.f_74 != 0 {
        return;
    }
    c.f_6c = a as i32;
    c.f_74 = 1;
    c.f_78 = 0;
    h.vcall_1c(i, 5, 0);
    if c.f_74 == 0 {
        return;
    }
    if c.f_7c != 0 {
        c.ee_sub_1e01f8(1, 1); // FUN_001e02c0
        c.f_7c = 0;
    }
    let n = ee_sub_1e5ae0(c, nodes, i as u32 + 1, 0, None, h);
    h.ee_sub_1e1c40(n);
    if c.f_74 == 0 {
        return;
    }
    if c.f_5c.is_empty() {
        // FUN_0011a3b0(+0x5c) != 0
        c.ee_sub_1dfeb8(i, h);
    } else if c.f_40 & 2 == 0 {
        h.ee_sub_1a6ca8(i, 1);
        c.f_40 |= 2;
        c.f_78 = 1;
    }
}

/// First argument as a word / int (kinds 6 / 0x13 / 0x10).
fn int_arg(args: &[MArg]) -> i32 {
    match args.first() {
        Some(MArg::I32(n)) => *n,
        Some(MArg::Word(w)) => *w as i32,
        _ => 0,
    }
}

fn find<'a>(set: &'a mut ClassSet, i: usize) -> Option<&'a mut Inst> {
    let ClassSet { insts, more, k4, k5, k6, .. } = set;
    insts.iter_mut().chain(more.iter_mut()).chain(k4.iter_mut()).chain(k5.iter_mut()).chain(k6.iter_mut()).find(|x| x.0 == i).map(|x| &mut x.1)
}

impl ClassLink {
    /// WP-J9: guard +0x2c0 behaviour bits (`guard::c56::Guard::f_2c0`); objects without a ported guard
    /// instance keep the ctor value 0.
    pub fn guard_2c0(&self, i: usize) -> u64 {
        let Some(s) = self.set.as_ref() else { return 0 };
        s.k6.iter().find(|x| x.0 == i).and_then(|x| if let Inst::Guard(g) = &x.1 { Some(g.f_2c0) } else { None }).unwrap_or(0)
    }

    /// WP-J10: the FUN_001dfd58(seq, player) calls the class code made while the set was ticked by
    /// classes/ (`EngineHost::seq_starts`, K11: interaction point slot 50 FUN_00184130 / class 0x2d1b60)
    /// run here on the sequence's class-114 instance (seq = handle). REPLACED: deferred to the next lend
    /// instead of the synchronous call; starts a sequence makes are drained too (bounded). A seq without a
    /// ported class-114 instance is recorded. Returns the sequences started.
    pub fn drain_seq_starts(&mut self) -> usize {
        let mut n = 0;
        // WP-EP1d: the job code's `*(obj + 0x40) |= 0x100; FUN_002283d0(obj)` (FUN_00162fd0 / FUN_00163d18):
        // deferred like the sequence starts (REPLACED: the original calls it synchronously in the job code)
        if let Some(set) = self.set.as_mut() {
            for oid in std::mem::take(&mut self.warp_oids) {
                let Some(i) = (self.host.ee_sub_15e938(0, oid, 0) as usize).checked_sub(1) else { continue };
                if let Some((_, Inst::C2dc6f0(c))) = set.more.iter_mut().find(|x| x.0 == i) {
                    c.f_40 |= 0x100;
                }
                if !Self::run(set, &mut self.host, &mut self.nodes, 0x0022_83d0, i as u32, &[], &[]) {
                    self.recorded.push(Recorded { thunk: 0, callee: 0x0022_83d0, obj: i as u32, args: Vec::new() });
                }
            }
        }
        for _ in 0..16 {
            let starts = std::mem::take(&mut self.host.seq_starts);
            if starts.is_empty() {
                break;
            }
            let Some(set) = self.set.as_mut() else { return n };
            for (seq, player) in starts {
                let Some(i) = (seq as usize).checked_sub(1) else { continue };
                match find(set, i) {
                    Some(Inst::C2d8c48(c)) => {
                        ee_sub_1dfd58(c, &mut self.nodes, i, player, &mut self.host);
                        n += 1;
                    }
                    _ => self.recorded.push(Recorded { thunk: 0, callee: 0x001d_fd58, obj: i as u32, args: vec![MArg::Word(player)] }),
                }
            }
        }
        n
    }

    /// Execute the engine function `callee` of a method row (thunk) on object index `obj`.
    /// Returns false when the row is not one of this module's or the class runtime is not lent. With
    /// the runtime lent, a class callee on an object without a ported instance is recorded.
    pub fn call(&mut self, thunk: u32, callee: u32, obj: u32, raw: &[Ref], args: Vec<MArg>) -> bool {
        if thunk == 0x0010_b028 {
            self.ee_sub_10b028(obj, &args);
            return true;
        }
        // WP-EP2F: methods of fields 595-612 (FUN_00170b40..FUN_00170dd0): entry appended to the class-113 +0x50
        // list (`C2cf880::apply_field`); the resolved level routine gets the matching world-free command
        // (`K6Level::build`: 5 wait, 9 label, 10 jump, other kinds `Other`). Rail / object entries (fields
        // 601 / 602) need the world and the 8-byte vector kinds (596 / 597) the Value payload: recorded.
        if let Some(field) = crate::engine::classes::c2cf880::C2cf880::field_of_callee(callee).filter(|f| !matches!(f, 596 | 597 | 601 | 602)) {
            let word = |i: usize| match args.get(i) {
                Some(MArg::I32(n)) => *n as u32,
                Some(MArg::Word(w)) => *w,
                Some(MArg::F32(f)) => f.to_bits(),
                _ => 0,
            };
            let kind = self.set.as_mut().and_then(|set| match find(set, obj as usize) {
                Some(Inst::C2cf880(c)) => {
                    c.apply_field(field, word, |_| [0, 0]);
                    c.f_50.last().map(|e| (e.kind, e.id, e.f_8[0]))
                }
                _ => None,
            });
            if let Some((kind, id, f8)) = kind {
                use crate::engine::classes::guard::patrol::Cmd;
                let cmd = match kind {
                    5 => Cmd::Wait(f32::from_bits(f8)),
                    9 => Cmd::Label(id),
                    10 => Cmd::Jump(id),
                    k => Cmd::Other(k),
                };
                if let Some(set) = self.set.as_mut() {
                    if let Some(k) = set.k6l.routines.iter().position(|x| x.0 == obj as usize) {
                        set.k6l.routines[k].1.push(cmd.clone());
                        set.k6l.cmds[k].push(cmd);
                    }
                }
            } else {
                self.recorded.push(Recorded { thunk, callee, obj, args });
            }
            return true;
        }
        if matches!(callee, 0x0016_2660 | 0x0016_5118 | 0x0016_2df8 | 0x0016_2e38) {
            let ints = args.iter().map(|a| match a {
                MArg::I32(n) => *n,
                MArg::Word(w) => *w as i32,
                MArg::F32(f) => *f as i32,
                _ => 0,
            });
            self.job_calls.push((callee, obj, ints.collect()));
            return true;
        }
        if GUARD_OID_CALLEES.contains(&callee) {
            self.guard_oid_call(callee, obj, &args);
            return true;
        }
        // WP-S1 rows (cls/s1.rs)
        if self.s1_call(thunk, callee, obj, raw, &args) {
            return true;
        }
        if RECORDED.contains(&callee) || RECORDED_J11.contains(&callee) || (callee == 0 && RECORDED_INLINE.contains(&thunk)) {
            self.recorded.push(Recorded { thunk, callee, obj, args });
            return true;
        }
        // WP-Q2: the class-114 record adder thunks 0x106a80..0x10a1b8 (fields 625..716 -> FUN_001e0290..)
        if self.k22_add_record(thunk, obj, &args) {
            return true;
        }
        if !CLASS_CALLEES.contains(&callee) {
            return false;
        }
        let Some(set) = self.set.as_mut() else { return false };
        if !Self::run(set, &mut self.host, &mut self.nodes, callee, obj, raw, &args) {
            // the object's class has no ported instance / override: kept for it like the recorded rows
            self.recorded.push(Recorded { thunk, callee, obj, args });
        } else if callee == 0x0020_88e0 {
            // WP-J12: FUN_002088e0 tail: jal 0x002089e8 / jal 0x00208b30 (+0x3a8 = +0x3c0 = 0)
            if let Some(g) = self.guard_oids.get_mut(&obj) {
                g.f_3ac.clear();
                g.f_3c4.clear();
            }
        }
        true
    }

    /// WP-J12: method thunk 0x0010b028 (field 750, symid 0x2de, classes 52-55; arg kind 0x1057 = state
    /// index): `(vtable +0x2c4)(obj, i)`, returns Void. Slot 87 of the three player-character vtables:
    ///  - class 53 JT (0x2d2e98) FUN_0018d568: i < 9 -> FUN_0018c250(jt, table 0x2a5d00[i], -1)
    ///    (`player::p7_damage::FUN_0018d568`); queued on `jt_states` for the player layer (SetJtJts needs
    ///    the player's JtAnim, owned by player/);
    ///  - class 55 (0x2cb230) FUN_00135c20 -> K16 `C2cb230::ee_sub_135c20` (FUN_00135248 hook);
    ///  - class 54 (0x2d44c8) FUN_001b7e40 (jump table 0x2a73f0 -> FUN_001b6078 / FUN_001b7ba8): class 54
    ///    is not ported -> recorded. Class 52 is abstract (no instance).
    pub fn ee_sub_10b028(&mut self, obj: u32, args: &[MArg]) {
        // 0x0010b028
        let i = int_arg(args);
        let class = self.host.world.as_deref().and_then(|w| w.objects.get(obj as usize)).map(|o| o.class);
        match class {
            Some(53) => {
                // 0x0018d568: (u32) i < 9, else nothing
                if let Some(jts) = crate::engine::player::p7_damage::FUN_0018d568(i) {
                    self.jt_states.push((obj, jts));
                }
            }
            Some(55) if self.set.is_some() => {
                let (set, host) = (self.set.as_mut().unwrap(), &mut self.host);
                if let Some(c) = set.k16_get(obj as usize) {
                    // 0x00135c20
                    c.ee_sub_135c20(obj as usize, i as u32, host);
                } else {
                    self.recorded.push(Recorded { thunk: 0x0010_b028, callee: 0x0013_5c20, obj, args: args.to_vec() });
                }
            }
            _ => {
                let callee = if class == Some(54) { 0x001b_7e40 } else { 0 };
                self.recorded.push(Recorded { thunk: 0x0010_b028, callee, obj, args: args.to_vec() });
            }
        }
    }

    /// WP-J11: FUN_0020aa78(guard, oid) = FUN_0020aa40 (append oid != -1 to +0xeb8 while +0xed8 < 8) then
    /// FUN_0020aaa8 (+0xefc = 0; each +0xeb8 oid -> FUN_0015e938(world, 4, oid, guard), found ones appended
    /// to +0xedc); FUN_0020ab58(guard): +0xefc = +0xed8 = 0. 1:1 on `guard_oids`.
    /// WP-Q2: thunk of a class-114 record field (`k22_seq::add_record_args`) on a `C2d8c48` object.
    /// Arguments as words: ints / payload words one word, floats their bits, vectors 3 floats.
    fn k22_add_record(&mut self, thunk: u32, obj: u32, args: &[MArg]) -> bool {
        let Some(field) = super::by_thunk(thunk).map(|m| m.field as i16).filter(|f| matches!(f, 625..=716)) else { return false };
        let words = |a: &MArg| match a {
            MArg::I32(n) => vec![*n as u32],
            MArg::Word(w) => vec![*w],
            MArg::F32(f) => vec![f.to_bits()],
            MArg::Ref(Ref::Vector(v)) | MArg::Value(Ref::Vector(v)) => v.iter().map(|x| x.to_bits()).collect(),
            MArg::Ref(r) | MArg::Value(r) => vec![r.payload()],
            MArg::None => vec![],
        };
        let w: Vec<Vec<u32>> = args.iter().map(words).collect();
        let Some(set) = self.set.as_mut() else { return false };
        match find(set, obj as usize) {
            Some(Inst::C2d8c48(c)) => {
                crate::engine::classes::k22_seq::add_record_args(c, field, &w);
                true
            }
            _ => false,
        }
    }

    fn guard_oid_call(&mut self, callee: u32, obj: u32, args: &[MArg]) {
        let g = self.guard_oids.entry(obj).or_default();
        // WP-J12: the class lists (push with cap; remove = memmove down while the index still advances,
        // so an adjacent duplicate survives, as FUN_00208728)
        let remove = |l: &mut Vec<i32>, c: i32| {
            let mut k = 0;
            while k < l.len() {
                if l[k] == c {
                    l.remove(k);
                }
                k += 1;
            }
        };
        match callee {
            // 0x00208920
            0x0020_8920 if g.f_3ac.len() < 5 => return g.f_3ac.push(int_arg(args)),
            // 0x00208948
            0x0020_8948 => return remove(&mut g.f_3ac, int_arg(args)),
            // 0x002089e8
            0x0020_89e8 => return g.f_3ac.clear(),
            // 0x00208a68
            0x0020_8a68 if g.f_3c4.len() < 2 => return g.f_3c4.push(int_arg(args)),
            // 0x00208a90
            0x0020_8a90 => return remove(&mut g.f_3c4, int_arg(args)),
            // 0x00208b30
            0x0020_8b30 => return g.f_3c4.clear(),
            0x0020_8920 | 0x0020_8a68 => return,
            _ => {}
        }
        if callee == 0x0020_ab58 {
            // 0x0020ab58
            g.edc.clear();
            g.eb8.clear();
            return;
        }
        // 0x0020aa40
        let oid = int_arg(args);
        if oid != -1 && g.eb8.len() < 8 {
            g.eb8.push(oid);
        }
        // 0x0020aaa8
        g.edc.clear();
        let Some(w) = self.host.world.as_deref() else { return };
        for &o in &g.eb8 {
            let f = super::find::ee_sub_15e958(w, 4, o, obj);
            if f != 0 {
                g.edc.push(f);
            }
        }
    }

    /// The class-runtime callees; false = no ported instance of the right class for `obj`.
    fn run(set: &mut ClassSet, host: &mut EngineHost, nodes: &mut Vec<SeqNode>, callee: u32, obj: u32, raw: &[Ref], args: &[MArg]) -> bool {
        let h: &mut dyn ClassHost = host;
        let i = obj as usize;
        match callee {
            // FUN_00162dd0(obj): FUN_001619a0(0x2deb30, 0x1d, obj)
            0x0016_2dd0 => h.ee_sub_1619a0(0x1d, obj + 1),
            // WP-J10: FUN_00133340(obj): (vtable +0x1c)(obj + adj, 5, 0, 0, 0, 0)
            0x0013_3340 => h.vcall_1c(i, 5, 0),
            // WP-Q3: FUN_00133380(obj): (vtable +0x1c)(obj + adj, 6, 0, 0, 0, 0)
            0x0013_3380 => h.vcall_1c(i, 6, 0),
            // WP-EP1f: FUN_00228170(warp) (thunk 0x10b760)
            0x0022_8170 => h.ee_sub_228170(obj + 1),
            // FUN_00164640(obj, k, ctx): class 133, ClassHost hook (ctx = the +0xf8 word restored on exit)
            0x0016_4640 => {
                let k = match args.first() {
                    Some(MArg::Word(w)) => *w as i32,
                    Some(MArg::I32(n)) => *n,
                    _ => 0,
                };
                h.ee_sub_164640(obj + 1, k)
            }
            // FUN_0015c150(obj): (vtable +0x19c = slot 51)(obj, zeroed 0x40-byte request)
            0x0015_c150 => {
                let ClassSet { insts, rng, .. } = set;
                let Some((_, Inst::C2ce4d0(c))) = insts.iter_mut().find(|x| x.0 == i) else { return false };
                c.ee_sub_15c690(i, &[0; 15], rng, h);
            }
            // WP-J9: FUN_00171cf8(guard, routine) (method 0x00103db0, field 317, routine = kind 0x2071
            // class-113 object): the guard's brain substate {5, routine} (guard::K6Level::routines)
            0x0017_1cf8 => {
                let Some(Ref::Basic(r)) = raw.first() else { return false };
                let Some(ri) = set.k6l.routines.iter().position(|x| x.0 == *r as usize) else { return false };
                if set.k6_guard(i).is_none() {
                    return false;
                }
                set.k6_run_routine(i, ri, h);
            }
            // WP-K28: FUN_00170e68(routine = obj, guard (kind 0x2038)) (method 0x00105cd0, field 594, class
            // 113): guard != 0 -> FUN_00171cf8(guard, routine)
            0x0017_0e68 => {
                let Some(Ref::Basic(g)) = raw.first() else { return false };
                let Some(ri) = set.k6l.routines.iter().position(|x| x.0 == i) else { return false };
                if set.k6_guard(*g as usize).is_none() {
                    return false;
                }
                set.k6_run_routine(*g as usize, ri, h);
            }
            // WP-EP1b: FUN_0013b5c8(d, proj) (method 389): fuse +0x154 = min(+0x154, DAT_002dded0 + d)
            0x0013_b5c8 => {
                let d = match args.first() {
                    Some(MArg::F32(f)) => *f,
                    Some(MArg::I32(n)) => *n as f32,
                    _ => 0.0,
                };
                let t = set.g.t;
                let Some(c) = set.k15_79_get(i) else { return false };
                c.p.ee_sub_13b5c8(d, t);
            }
            // WP-TR3b: FUN_0013b860 (method 390, classes 79-82): explode projectile
            0x0013_b860 => {
                let Some(c) = set.k15_79_get(i) else { return false };
                c.p.ee_sub_13b860(i, h);
            }
            // WP-TR3b: FUN_001a58d0 (vtable +0x54 SetParent): reparent object
            0x001a_58d0 => {
                let a = int_arg(args);
                let b = match args.get(1) {
                    Some(MArg::I32(n)) => *n,
                    Some(MArg::Word(w)) => *w as i32,
                    _ => 0,
                };
                h.ee_sub_1a58d0(i, a, b);
            }
            // WP-J11: FUN_001d6b00(launcher = obj, projectile (kind 0x2030)) (method 0x001058f0, field 468):
            // the K14 port `ClassSet::k14_fire` (class-112 launcher, class-81 projectile)
            0x001d_6b00 => {
                let Some(Ref::Basic(p)) = raw.first() else { return false };
                if set.k14_fire(i, *p as usize, h).is_none() {
                    return false;
                }
            }
            // WP-J11: FUN_002087c8(guard): sensor (+0xf64) +0xfc -= +0x3a4 (the dynamic entries are the
            // tail), +0x3a4 = 0, +0x394 = 0. FUN_002088e0(guard): sensor +0xfc = 0, +0x394 = +0x3a4 = 0,
            // FUN_002089e8 (+0x3a8 = 0) / FUN_00208b30 (+0x3c0 = 0): WP-J12 `GuardOids::f_3ac` / `f_3c4`.
            // WP-J11: FUN_00208700(guard, class) (method 0x00103520): +0x394 < 3 -> +0x398[+0x394++] = class
            // (the sensor's candidate classes, `Guard::f_394`). 1:1.
            0x0020_8700 => {
                let Some((_, Inst::Guard(g))) = set.k6.iter_mut().find(|x| x.0 == i) else { return false };
                if g.f_394.len() < 3 {
                    g.f_394.push(int_arg(args));
                }
            }
            // WP-J11: FUN_00208728(guard, class) (method 0x001035a8): remove `class` from +0x398[..+0x394]
            // (memmove down; the index still advances, so an adjacent duplicate survives). 1:1.
            0x0020_8728 => {
                let Some((_, Inst::Guard(g))) = set.k6.iter_mut().find(|x| x.0 == i) else { return false };
                let c = int_arg(args);
                let mut k = 0;
                while k < g.f_394.len() {
                    if g.f_394[k] == c {
                        g.f_394.remove(k);
                    }
                    k += 1;
                }
            }
            0x0020_87c8 | 0x0020_88e0 => {
                let Some((_, Inst::Guard(g))) = set.k6.iter_mut().find(|x| x.0 == i) else { return false };
                let n = if callee == 0x0020_88e0 { 0 } else { g.sensor.entries.len().saturating_sub(g.f_3a4) };
                g.sensor.entries.truncate(n);
                g.f_3a4 = 0;
                g.f_394.clear();
            }
            0x0022_83d0 => {
                let mut chain = Vec::new();
                {
                    let ClassSet { more, g, .. } = set;
                    let Some((_, Inst::C2dc6f0(c))) = more.iter_mut().find(|x| x.0 == i) else { return false };
                    chain.extend(c.ee_sub_2283d0(i, g, h));
                }
                // FUN_002280d0 hand-over (recursive call in the original)
                while let Some(o) = chain.pop() {
                    let ClassSet { more, g, .. } = set;
                    if let Some((j, Inst::C2dc6f0(c))) = more.iter_mut().find(|x| x.0 + 1 == o as usize) {
                        chain.extend(c.ee_sub_2283d0(*j, g, h));
                    }
                }
            }
            // WP-TR1: FUN_001f6ac8(obj, id) = so_part_link(obj, id, true)
            0x001f_6ac8 => host.so_part_link(i, int_arg(args) as i32, true),
            // WP-TR1: FUN_001f6b48(obj, id) = so_part_link(obj, id, false)
            0x001f_6b48 => host.so_part_link(i, int_arg(args) as i32, false),
            // WP-TR1: FUN_001f6bc8(obj, id, v) = so part +0x18 setter
            0x001f_6bc8 => {
                let id = int_arg(args);
                let v = if args.len() > 1 { int_arg(&args[1..]) as u32 } else { 0 };
                host.ee_sub_1f6bc8(i, id, v);
            }
            // WP-TR1: FUN_001f7c68 = EngineHost::ee_call(addr, args) — generic callee dispatch
            0x001f_7c68 => {
                let addr = int_arg(args) as u32;
                let rest: Vec<u32> = args[1..].iter().map(|a| int_arg(std::slice::from_ref(a)) as u32).collect();
                host.ee_call(addr, &rest);
            }
            // WP-TR1: FUN_00171d58 = k22_seq::ee_sub_171d58(c, guard_id) — stop guard op
            0x0017_1d58 => match (callee, find(set, i)) {
                (0x0017_1d58, Some(Inst::C2d8c48(c))) => {
                    crate::engine::classes::k22_seq::ee_sub_171d58(c, int_arg(args) as u32);
                }
                _ => return false,
            },
            // WP-TR1: FUN_00171da8 = k22_seq::ee_sub_171da8(c, guard_id, running) — query running
            0x0017_1da8 => match (callee, find(set, i)) {
                (0x0017_1da8, Some(Inst::C2d8c48(c))) => {
                    let _running = crate::engine::classes::k22_seq::ee_sub_171da8(c, int_arg(args) as u32, if args.len() > 1 { int_arg(&args[1..]) as u32 } else { 0 });
                }
                _ => return false,
            },
            // WP-TR1b: Sensor::ee_sub_1541d8 — needs See + ClassHost not in ClassLink::run; partial
            0x0015_41d8 => return false,
            // WP-TR1: FUN_001e0120(obj) — ClassHost trait: k24 sequence start
            0x001e_0120 => h.ee_sub_1e0120(obj),
            // WP-TR1: FUN_001f0598(obj) — ClassHost trait: link off
            0x001f_0598 => h.ee_sub_1f0598(i),
            // WP-TR1: FUN_001f0730(snd, id) — ClassHost trait: sound dispatch
            0x001f_0730 => h.ee_sub_1f0730(obj, int_arg(args) as i32),
            // WP-TR1: FUN_001f03a8(sm, obj, label, flags, mask) — ClassHost trait: link setup
            0x001f_03a8 => { let _ = h.ee_sub_1f03a8_link(obj, int_arg(args) as u32, if args.len() > 1 { int_arg(&args[1..]) } else { 0 } as i32, if args.len() > 2 { int_arg(&args[2..]) as u32 } else { 0 }, if args.len() > 3 { int_arg(&args[3..]) } else { 0 } as i32); }
            // WP-TR1: FUN_001f82b8(obj, v) — ClassHost trait: alert/trigger
            0x001f_82b8 => h.ee_sub_1f82b8(i, int_arg(args)),
            // WP-TR1b: FUN_001b2020(obj, v) — k15_79 Proj::f_23c push (cap 4)
            0x001b_2020 => {
                let v = int_arg(args) as i32;
                if let Some(c) = set.k15_79_get(i) {
                    if c.p.f_23c.len() < 4 { c.p.f_23c.push(v); }
                }
            }
            // WP-TR1b: FUN_001b2048(obj, v) — k15_79 Proj::f_250 push (cap 4)
            0x001b_2048 => {
                let v = int_arg(args) as i32;
                if let Some(c) = set.k15_79_get(i) {
                    if c.p.f_250.len() < 4 { c.p.f_250.push(v); }
                }
            }
            // WP-TR1: FUN_001291a8 = thunk result copy (inline); already handled by method_table
            0x0012_91a8 => { /* noop: result already captured by ee_sub_201110 */ }
            // WP-TR1b: these callees are ported (sensor.rs, jtblend.rs, p8_hazard.rs, k25.rs,
            // music_mgr.rs) but need subsystems (Aseg, MusicMgr, K25, target+0x110) not available
            // in ClassLink::run. Stub → recorded; the subsystem owners execute them.
            0x0012_f990 => return false, // jtblend::ee_sub_12f990 (Aseg)
            0x001f_78d8 => return false, // p8_hazard::FUN_001f78d8 (target +0x110)
            0x001f_b4f0 => return false, // K25::ee_sub_1fb4f0 (fog; K25 SoundRt)
            0x001f_e3f8 => return false, // MusicMgr::ee_sub_1fe3f8 (getter)
            0x001f_e9f8 => return false, // MusicMgr::ee_sub_1fe9f8
            0x001f_ea60 => return false, // MusicMgr::ee_sub_1fea60
            0x0022_7c90 => return false, // sensor::ee_sub_227c90 (Vol + pos)
            // WP-TR1b: FUN_001e1970(arg1, obj, arg0) — C2d8c48 record {0x53}: [5]=arg1, [4]=arg0
            0x001e_1970 => match find(set, i) {
                Some(Inst::C2d8c48(c)) => {
                    let a0 = int_arg(args) as u32;
                    let a1 = if args.len() > 1 { int_arg(&args[1..]) as u32 } else { 0 };
                    if let Some(k) = c.ee_sub_1e01f8(0x53, 0) {
                        c.f_54[k][5] = a1; // +0x14
                        c.f_54[k][4] = a0; // +0x10
                    }
                }
                _ => return false,
            },
            // WP-TR1b: FUN_001e1ad0(obj, arg0, arg1, arg2) — C2d8c48 record {0x59, arg2}: [5]=arg1, [4]=arg0
            0x001e_1ad0 => match find(set, i) {
                Some(Inst::C2d8c48(c)) => {
                    let a0 = int_arg(args) as u32;
                    let a1 = if args.len() > 1 { int_arg(&args[1..]) as u32 } else { 0 };
                    let a2 = if args.len() > 2 { int_arg(&args[2..]) as u32 } else { 0 };
                    if let Some(k) = c.ee_sub_1e01f8(0x59, a2) {
                        c.f_54[k][5] = a1; // +0x14
                        c.f_54[k][4] = a0; // +0x10
                    }
                }
                _ => return false,
            },
            // WP-TR1b: FUN_001e1b10(obj, arg0, arg1, arg2) — C2d8c48 record {0x5a, arg2}: [4]=arg0, [5]=arg1, [6]=0
            0x001e_1b10 => match find(set, i) {
                Some(Inst::C2d8c48(c)) => {
                    let a0 = int_arg(args) as u32;
                    let a1 = if args.len() > 1 { int_arg(&args[1..]) as u32 } else { 0 };
                    let a2 = if args.len() > 2 { int_arg(&args[2..]) as u32 } else { 0 };
                    if let Some(k) = c.ee_sub_1e01f8(0x5a, a2) {
                        c.f_54[k][4] = a0; // +0x10
                        c.f_54[k][5] = a1; // +0x14
                        c.f_54[k][6] = 0;  // +0x18
                    }
                }
                _ => return false,
            },
            _ => match (callee, find(set, i)) {
                // FUN_0015a150: +0x148 = DAT_002a514c (FLT_MAX)
                (0x0015_a150, Some(Inst::C2ceb78(c))) => c.f_148 = f32::MAX,
                (0x001d_fd58, Some(Inst::C2d8c48(c))) => ee_sub_1dfd58(c, nodes, i, handle_of(raw.first()), h),
                // FUN_001dfe80: FUN_001dfd58(obj, FUN_001c4840(SW) = SW +0x3efc)
                (0x001d_fe80, Some(Inst::C2d8c48(c))) => {
                    let p = h.ee_sub_1c4840();
                    ee_sub_1dfd58(c, nodes, i, p, h)
                }
                (0x001d_feb8, Some(Inst::C2d8c48(c))) => c.ee_sub_1dfeb8(i, h),
                // WP-J10: FUN_001dffe0: stop, drop the records (+0x50 = 0), FUN_001e0290(obj, -1), +0x7c = 1
                (0x001d_ffe0, Some(Inst::C2d8c48(c))) => {
                    c.ee_sub_1dfeb8(i, h);
                    c.f_54.clear();
                    if let Some(k) = c.ee_sub_1e01f8(0, 1) {
                        c.f_54[k][4] = u32::MAX;
                    }
                    c.f_7c = 1;
                }
                // WP-J10: FUN_001e0760(obj, a): record {0x14, 1} with +0x10 = a
                (0x001e_0760, Some(Inst::C2d8c48(c))) => {
                    if let Some(k) = c.ee_sub_1e01f8(0x14, 1) {
                        c.f_54[k][4] = int_arg(args) as u32;
                    }
                }
                // WP-TR1: FUN_001e0120(obj) = ClassHost ee_sub_1e0120 — handled as top-level arm below
                // WP-J10: FUN_001baed8(obj, 0, 0) (thunk 0x0010d158) / FUN_001baf18(obj, target)
                (0x001b_aed8, Some(Inst::C2d4b78(c))) => c.ee_sub_1baed8(i, 0, 0, h),
                (0x001b_af18, Some(Inst::C2d4b78(c))) => c.ee_sub_1baf18(i, int_arg(args), h),
                _ => return false,
            },
        }
        true
    }

    /// A world+0x3ca8 node (FUN_00149c80 deferred phase): direct-fn PMFs (vindex -1) whose fn is one
    /// of the class callees above run with no args; false = not handled (caller records it).
    pub fn call_deferred(&mut self, obj: u32, vindex: i16, fnw: u32) -> bool {
        if vindex != -1 || !CLASS_CALLEES.contains(&fnw) || matches!(fnw, 0x001d_fd58 | 0x0016_4640 | 0x0017_1cf8 | 0x0017_0e68 | 0x001e_0760 | 0x001b_af18 | 0x001b_aed8 | 0x001d_6b00 | 0x0020_8700 | 0x0020_8728 | 0x001f_6ac8 | 0x001f_6b48 | 0x001f_6bc8 | 0x001f_7c68 | 0x0017_1d58 | 0x0017_1da8 | 0x0015_41d8 | 0x001e_0120 | 0x001f_0598 | 0x001f_0730 | 0x001f_03a8 | 0x001f_82b8 | 0x001b_2020 | 0x001b_2048 | 0x0012_91a8 | 0x0012_f990 | 0x001f_78d8 | 0x001f_b4f0 | 0x001f_e3f8 | 0x001f_e9f8 | 0x001f_ea60 | 0x0022_7c90 | 0x001e_1970 | 0x001e_1ad0 | 0x001e_1b10) {
            return false;
        }
        let Some(set) = self.set.as_mut() else { return false };
        Self::run(set, &mut self.host, &mut self.nodes, fnw, obj, &[], &[])
    }
}
