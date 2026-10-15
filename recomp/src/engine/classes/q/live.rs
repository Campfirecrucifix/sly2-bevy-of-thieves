//! WP-WIRE1: the live `QMem` host. Runs the queue ports (classes/q/*, game/q_*.rs) inside the game over a
//! guest-address image of the loaded world (design: ENGINE_PORT.md 'WIRE1').
//! - static image = SCUS_973.16 PT_LOAD bytes (vtables, descriptors, globals at their real addresses),
//!   copy-on-write in 4 KB pages; objects of `World::objects` at synthetic heap addresses with their header
//!   (+0xc vtable, +0x10 parent, +0x18 oid, +0x24 / +0x64 child list, +0x28 bit 60) and member properties;
//! - `call` = generated `live_dispatch::TABLE` (port_tracking -> Rust queue port), then the q trivial-override
//!   tables; every other address is recorded (`LiveStats::native` / `unported`), never silently dropped.
use super::live_dispatch::{NATIVE, TABLE};
use super::live_ownership::{copy_hand_to_guest, snapshot_queue, FieldKey};
use super::q2mem::{slot_off, QMem, V4};
use crate::engine::world::World;
use std::collections::{BTreeMap, HashMap};

const PAGE: u32 = 0x1000;
/// object heap (above the EE executable + bss, below the q2mem scratch stack)
pub const HEAP_BASE: u32 = 0x0100_0000;
pub const HEAP_END: u32 = 0x03e0_0000;
/// &DAT_003dd490: the world (parent of entry-level objects)
pub const WORLD: u32 = 0x003d_d490;
/// clock globals (game/clock.rs): DAT_002dded0 t, DAT_002dded4 dt, DAT_002ddee0 dt_real
pub const G_T: u32 = 0x002d_ded0;
pub const G_DT: u32 = 0x002d_ded4;
pub const G_DT_REAL: u32 = 0x002d_dee0;
/// vt+0x84 = FUN_00226550 ObjectUpdate slot, vt+0x7c = post-load slot
pub const SLOT_UPDATE: u32 = 15;
pub const SLOT_POSTLOAD: u32 = 14;
/// vt+0x3c = init (obj, parent), run by the loader per created object (FUN_001a5270 base)
pub const SLOT_INIT: u32 = 6;
/// vt+0x1c = message handler (FUN_0022b490 dispatches here); nargs 2 (event, arg_handle)
pub const SLOT_MESSAGE: u32 = 2;
const MAX_DEPTH: u32 = 96;
const BUDGET: u64 = 4_000_000;

/// Counters of the bridge (what ran, what had no queue port).
#[derive(Clone, Debug, Default)]
pub struct LiveStats {
    /// calls dispatched into queue ports (any depth)
    pub q_calls: u64,
    /// top-level entries (`LiveMem::enter`) that reached a queue port
    pub top_q: u64,
    /// per queue-port address
    pub per_fn: BTreeMap<u32, u64>,
    /// q trivial overrides (CONST_RET / NOPS tables)
    pub trivial: u64,
    /// callees ported as state-struct code (not QMem-callable): address -> count
    pub native: BTreeMap<u32, u64>,
    /// callees with no Rust port at all (or address 0 / garbage): address -> count
    pub unported: BTreeMap<u32, u64>,
    pub panics: u64,
    pub budget_hits: u64,
    pub depth_hits: u64,
}

impl LiveStats {
    /// WP-ARCH1 (3): write every dispatched address (`per_fn` keys) to `path` as a TSV of
    /// `0xADDRESS<TAB>HITCOUNT` lines, sorted ascending. Consumed by `tools/wired.py` and
    /// `tools/progress.py` to count functions actually reached by the live store (vt+0x1c messages,
    /// slot-6/14/15 dispatch, and `JobMgr::replay_calls_through`) in addition to the static call
    /// graph. Idempotent (overwrites). Hit count is informational only.
    pub fn write_reached(&self, path: &std::path::Path) -> std::io::Result<usize> {
        if let Some(p) = path.parent() {
            std::fs::create_dir_all(p)?;
        }
        let mut v: Vec<(u32, u64)> = self.per_fn.iter().map(|(a, n)| (*a, *n)).collect();
        v.sort_by_key(|(a, _)| *a);
        let mut s = String::with_capacity(v.len() * 24);
        for (a, n) in &v {
            s.push_str(&format!("0x{:06x}\t{}\n", a, n));
        }
        std::fs::write(path, s)?;
        Ok(v.len())
    }

    pub fn summary(&self) -> String {
        let n: u64 = self.native.values().sum();
        let u: u64 = self.unported.values().sum();
        format!(
            "q_calls {} (top {}, {} fns), trivial {}, native {} ({} fns), unported {} ({} fns), panics {}, budget {}, depth {}",
            self.q_calls,
            self.top_q,
            self.per_fn.len(),
            self.trivial,
            n,
            self.native.len(),
            u,
            self.unported.len(),
            self.panics,
            self.budget_hits,
            self.depth_hits
        )
    }
}

/// queue port for `addr` (generated table)
pub fn lookup(addr: u32) -> Option<super::live_dispatch::QFn> {
    TABLE.binary_search_by_key(&addr, |e| e.0).ok().map(|i| TABLE[i].1)
}

/// q trivial overrides: Some(return value) for CONST_RET / NOPS bodies
pub fn trivial(addr: u32) -> Option<u32> {
    use super::{q10_a, q11_a, q11_b, q11_d, q6, q8util, q9util};
    if q6::is_trivial(addr) {
        return Some(q6::const_ret(addr).unwrap_or(0));
    }
    for t in [q8util::CONST_RET, q9util::CONST_RET, q10_a::CONST_RET, q11_a::CONST_RET, q11_b::CONST_RET] {
        if let Some(x) = t.iter().find(|x| x.0 == addr) {
            return Some(x.1);
        }
    }
    for t in [q8util::NOPS, q9util::NOPS, q10_a::NOPS, q11_b::NOPS, q11_d::NOPS] {
        if t.contains(&addr) {
            return Some(0);
        }
    }
    None
}

/// EE kernel syscalls (libkernel, port_tracking 'skipped': no PC role - the PC build uses the Bevy schedule +
/// std threads). In the shadow host they are faithful no-ops: `GetThreadId` returns a fixed main-thread id so
/// the hard4 recursive locks (ee_sub_21f060/21f0c0) have a stable owner, and `CreateSema`/`WaitSema`/`SignalSema`
/// succeed. Reached transitively from queue code once the lock/unlock wrappers are dispatched.
pub fn kernel_syscall(addr: u32) -> Option<u32> {
    match addr {
        0x0023_4910 => Some(1), // GetThreadId -> main thread
        0x0023_4a20 => Some(1), // CreateSema -> non-zero handle
        0x0023_4a40 | 0x0023_4a60 => Some(0), // SignalSema / WaitSema -> success
        _ => None,
    }
}

/// true when a call to `addr` reaches queue code (table or trivial override)
pub fn is_queue(addr: u32) -> bool {
    lookup(addr).is_some() || trivial(addr).is_some()
}

#[derive(Debug)]
struct BudgetExceeded;

/// The live host.
pub struct LiveMem {
    pages: HashMap<u32, Box<[u8; PAGE as usize]>>,
    /// guest address of `World::objects[i]`
    pub obj_addr: Vec<u32>,
    by_addr: BTreeMap<u32, usize>,
    heap: u32,
    depth: u32,
    ops: u64,
    pub stats: LiveStats,
    /// log the first call of every unported address (env SLY2_WIRE_LOG)
    pub verbose: bool,
    /// PAD1: recorded FUN_001619a0(addr, event, arg) calls from QMem (drained by the host)
    pub recorded_1619a0: Vec<(u32, i32, i32)>,
    /// PAD1: recorded FUN_00164640(handle, state) calls from QMem (native calls routed to host)
    pub recorded_164640: Vec<(u32, i32)>,
    /// PAD1: recorded gadget-purchase save writes from QMem (entry_id, purchased)
    pub recorded_gadget_purchase: Vec<(u32, bool)>,
    /// WIRE-A: FUN_001dfd58(seq, player) calls from QMem (object index of the sequence, player word); the
    /// sequences run in the class runtime, so the host drains these into `EngineHost::seq_starts`
    pub recorded_1dfd58: Vec<(usize, u32)>,
    /// WIRE-A: world handle for the FUN_0015e938 relative search (set by `LiveQ::build` / `set_world`)
    pub world: Option<std::sync::Arc<World>>,
}

impl Default for LiveMem {
    fn default() -> Self {
        LiveMem {
            pages: HashMap::new(),
            obj_addr: Vec::new(),
            world: None,
            by_addr: BTreeMap::new(),
            heap: HEAP_BASE,
            depth: 0,
            ops: 0,
            stats: LiveStats::default(),
            verbose: std::env::var_os("SLY2_WIRE_LOG").is_some(),
            recorded_1619a0: Vec::new(),
            recorded_164640: Vec::new(),
            recorded_gadget_purchase: Vec::new(),
            recorded_1dfd58: Vec::new(),
        }
    }
}

impl LiveMem {
    fn page(&mut self, a: u32) -> &mut [u8; PAGE as usize] {
        let base = a & !(PAGE - 1);
        self.pages.entry(base).or_insert_with(|| {
            let mut p = Box::new([0u8; PAGE as usize]);
            if base < HEAP_BASE {
                let x = crate::engine::disc::exe::exe();
                if let Some(b) = x.bytes(base, PAGE as usize) {
                    p.copy_from_slice(b);
                } else {
                    for (i, d) in p.iter_mut().enumerate() {
                        *d = x.bytes(base + i as u32, 1).map_or(0, |b| b[0]);
                    }
                }
            }
            p
        })
    }

    fn tick(&mut self) {
        self.ops += 1;
        if self.ops > BUDGET && self.depth > 0 {
            std::panic::panic_any(BudgetExceeded);
        }
    }

    /// object index at guest address `a` (object start)
    pub fn obj_at(&self, a: u32) -> Option<usize> {
        self.by_addr.get(&a).copied()
    }

    /// reserve `n` bytes of the object heap (16-aligned, zeroed)
    pub fn alloc(&mut self, n: u32) -> u32 {
        let a = self.heap;
        self.heap = (self.heap + n.max(0x10) + 15) & !15;
        assert!(self.heap < HEAP_END, "LiveMem heap exhausted");
        a
    }

    /// Build the image of `w`: one heap object per `World::objects` entry with its header + member properties.
    pub fn build(w: &World) -> LiveMem {
        use crate::engine::world::class_table::CLASSES;
        let mut m = LiveMem::default();
        for o in &w.objects {
            let size = CLASSES.get(o.class as usize).map_or(0x50, |c| c.size).clamp(0x50, 0x10000);
            let a = m.alloc(size);
            m.by_addr.insert(a, m.obj_addr.len());
            m.obj_addr.push(a);
        }
        for (i, o) in w.objects.iter().enumerate() {
            let a = m.obj_addr[i];
            let vt = CLASSES.get(o.class as usize).map_or(0, |c| c.vtable);
            m.w32(a + 0xc, vt);
            m.w32(a + 0x10, o.owner.map_or(WORLD, |p| m.obj_addr[p]));
            m.w32(a + 0x18, o.oid as u32);
            // children: +0x64 first, +0x24 next sibling (null-terminated)
            let kids: Vec<u32> = o.children.iter().map(|&c| m.obj_addr[c]).collect();
            m.w32(a + 0x64, kids.first().copied().unwrap_or(0));
            for k in kids.windows(2) {
                m.w32(k[0] + 0x24, k[1]);
            }
            if !crate::engine::game::fields::out_of_world_at_load(w, i) {
                let f = m.r64(a + 0x28) | 1 << 60;
                m.w32(a + 0x28, f as u32);
                m.w32(a + 0x2c, (f >> 32) as u32);
            }
            if let Some(props) = crate::engine::game::fields::object_props(&w.objects[o.clone_of.unwrap_or(i)]) {
                m.write_props(a, props, size_of_class(o.class));
                m.method_props(a, props);
            }
            // WP-ARCH1 step 1: write the FUN_00231768 xform (object +0x60) of the three
            // payload types that carry one, so the guest store has positions matching the
            // hand-port World on levels made of those payloads.
            if let Some(bytes) = xform_bytes(&w.objects[o.clone_of.unwrap_or(i)]) {
                for (k, &b) in bytes.iter().enumerate() {
                    m.w8(a + 0x60 + k as u32, b);
                }
            }
        }
        m.apply_class6_overrides(w);
        m.build_sms(w);
        m
    }

    /// WIRE-A: FUN_001efac0 (class 111 SM load) state table in the store: +0x50 count, +0x54 {label, next}
    /// pairs (anim::sm::Sm::ee_sub_1efac0 maps next 0x20 -> -2). The C2d80d0 hub +0x184 SM reference is
    /// stored as the SM object's address (the store keeps the raw oid of that field; REPLACED: resolved here
    /// for the class-70 hubs, as the snapshots show a pointer there after load).
    pub fn build_sms(&mut self, w: &World) {
        use crate::engine::world::loads_b4::B4Load;
        use crate::engine::world::objects::{LoadData, Payload};
        for (i, o) in w.objects.iter().enumerate() {
            let Payload::Loaded(LoadData::B4(b)) = &o.payload else { continue };
            let B4Load::L1efac0 { pairs, links, link_f, nested, .. } = &**b else { continue };
            let nested: Vec<(usize, i32)> = nested.iter().map(|&k| (k, w.objects[k].oid)).collect();
            let sm = crate::engine::anim::sm::Sm::ee_sub_1efac0(pairs, links, link_f, &nested);
            let a = self.obj_addr[i];
            let p = self.alloc(8 * sm.states.len() as u32);
            for (k, st) in sm.states.iter().enumerate() {
                self.w32(p + 8 * k as u32, st.0 as u32);
                self.w32(p + 8 * k as u32 + 4, st.1 as u32);
            }
            self.w32(a + 0x50, sm.states.len() as u32);
            self.w32(a + 0x54, p);
        }
        for (i, o) in w.objects.iter().enumerate() {
            if o.class != 70 {
                continue;
            }
            let a = self.obj_addr[i];
            let oid = self.r32(a + 0x184) as i32;
            let target = w.find_oid(oid).iter().copied().find(|&k| w.objects[k].class == 111);
            if let Some(k) = target {
                let t = self.obj_addr[k];
                self.w32(a + 0x184, t);
            }
        }
    }

    /// WIRE-A: FUN_001c9b08 kind-5 overrides (property lists of the class-6 record, FUN_0013f810 on the
    /// object created in place) applied to the world object with the record's oid. Call after `build`.
    pub fn apply_class6_overrides(&mut self, w: &World) {
        use crate::engine::world::loads_b4::B4Load;
        use crate::engine::world::objects::{LoadData, Payload};
        for o in &w.objects {
            let Payload::Loaded(LoadData::B4(b)) = &o.payload else { continue };
            let B4Load::Class6 { overrides, .. } = &**b else { continue };
            let Some(&t) = w.find_oid(o.oid).iter().find(|&&k| w.objects[k].class != 6) else { continue };
            let a = self.obj_addr[t];
            for ov in overrides.iter().filter(|ov| ov.key == 5) {
                if let Some(p) = &ov.props {
                    self.method_props(a, p);
                }
            }
        }
    }

    /// WIRE-A: FUN_0013e818 setter path for method properties (field row +8 = splice thunk, args = the
    /// array items): only the load methods in `LOAD_METHODS` whose callee has a QMem port.
    fn method_props(&mut self, a: u32, props: &[crate::engine::world::props::Prop]) {
        use crate::engine::world::field_table::field_word;
        use crate::engine::world::props::{ArrayItem, PropValue};
        for p in props {
            let PropValue::Array(items) = &p.value else { continue };
            let thunk = field_word(p.field as usize, 2);
            let Some(&(_, callee)) = super::q2_thunks::THUNKS.iter().find(|t| t.0 == thunk) else { continue };
            if !LOAD_METHODS.contains(&callee) {
                continue;
            }
            let mut args = vec![a];
            for it in items {
                let ArrayItem::One(_, b) = it else { continue };
                if b.len() >= 4 {
                    args.push(u32::from_le_bytes([b[0], b[1], b[2], b[3]]));
                }
            }
            self.call(callee, &args, &[]);
        }
    }

    /// FUN_0013e818 / FUN_00200598 direct path: scalar member properties (flags & 0x10 without the 0x2000
    /// sub-object indirection; member offset = field +0x10) into the object
    fn write_props(&mut self, a: u32, props: &[crate::engine::world::props::Prop], size: u32) {
        use crate::engine::world::field_table::field_word;
        use crate::engine::world::props::{field, PropValue};
        for p in props {
            let Some(d) = field(p.field) else { continue };
            if d.flags & 0x10 == 0 || d.flags & 0x2000 != 0 || d.kind == 4 {
                continue;
            }
            let off = field_word(p.field as usize, 4);
            let PropValue::Scalar(b) = &p.value else { continue };
            if off == 0 || off + b.len() as u32 > size {
                continue;
            }
            for (k, &x) in b.iter().enumerate() {
                self.w8(a + off + k as u32, x);
            }
        }
    }

    /// clock globals for the frame
    pub fn set_clock(&mut self, t: f32, dt: f32, dt_real: f32) {
        self.wf(G_T, t);
        self.wf(G_DT, dt);
        self.wf(G_DT_REAL, dt_real);
    }

    /// vtable slot target of the object at `obj`
    pub fn slot_target(&mut self, obj: u32, slot: u32) -> u32 {
        let vt = self.r32(obj + 0xc);
        if vt == 0 {
            return 0;
        }
        self.r32(vt + slot_off(slot))
    }

    /// Top-level entry from the game: `vslot(obj, slot)` with panic / budget containment. Returns None when
    /// the call panicked or ran over budget (counted in `stats`).
    pub fn enter(&mut self, obj: u32, slot: u32, a: &[u32], v: &[V4]) -> Option<(u32, V4)> {
        self.ops = 0;
        let before = self.stats.q_calls;
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| super::q2mem::vslot(self, obj, slot, a, v)));
        self.depth = 0;
        if self.stats.q_calls > before {
            self.stats.top_q += 1;
        }
        match r {
            Ok(x) => Some(x),
            Err(e) => {
                if e.downcast_ref::<BudgetExceeded>().is_some() {
                    self.stats.budget_hits += 1;
                } else {
                    self.stats.panics += 1;
                }
                None
            }
        }
    }

    /// Run `slot` on every object (index filter `pick`) whose target is queue code. Returns the number run.
    pub fn run_slot(&mut self, slot: u32, a_extra: &[u32], v: &[V4], mut pick: impl FnMut(usize) -> bool) -> usize {
        let mut n = 0;
        for i in 0..self.obj_addr.len() {
            if !pick(i) {
                continue;
            }
            let o = self.obj_addr[i];
            let f = self.slot_target(o, slot);
            if f == 0 || !is_queue(f) {
                continue;
            }
            self.enter(o, slot, a_extra, v);
            n += 1;
        }
        n
    }

    /// in-world flag (+0x28 bit 60)
    pub fn in_world(&mut self, i: usize) -> bool {
        let a = self.obj_addr[i];
        (self.r64(a + 0x28) >> 60) & 1 != 0
    }

    /// WP-ARCH1 step 2: deliver through a vtable slot on the object at `obj` with queue dispatch.
    /// Returns `(ok, result, v)`. `ok = false` means the slot has no queue port (recorded in
    /// `stats.native` / `stats.unported` by `call`), so the caller may fall back to the hand port.
    pub fn call_slot(&mut self, obj: u32, slot: u32, a: &[u32], v: &[V4]) -> (bool, u32, V4) {
        let f = self.slot_target(obj, slot);
        if f == 0 || !is_queue(f) {
            return (false, 0, [0.0; 4]);
        }
        let r = self.enter(obj, slot, a, v).unwrap_or((0, [0.0; 4]));
        (true, r.0, r.1)
    }
}

fn size_of_class(c: crate::engine::world::class::ClassId) -> u32 {
    crate::engine::world::class_table::CLASSES.get(c as usize).map_or(0x50, |c| c.size).clamp(0x50, 0x10000)
}

/// WP-ARCH1 step 1: the 48-byte FUN_00231768 xform of `o`, if its payload carries one.
/// Written at object +0x60 (per the FUN_00231768 comment: "4x3 matrix +0x60").
fn xform_bytes(o: &crate::engine::world::objects::Lo) -> Option<[u8; 48]> {
    use crate::engine::world::objects::{LoadData, Payload};
    let Payload::Loaded(d) = &o.payload else { return None };
    match d {
        LoadData::L231170 { xform, .. } => Some(*xform),
        LoadData::L227380 { xform, .. } => Some(xform.xform),
        LoadData::L231170Blob { xform, .. } => Some(xform.xform),
        _ => None,
    }
}

impl QMem for LiveMem {
    fn r8(&mut self, a: u32) -> u8 {
        self.tick();
        self.page(a)[(a & (PAGE - 1)) as usize]
    }
    fn w8(&mut self, a: u32, v: u8) {
        self.tick();
        self.page(a)[(a & (PAGE - 1)) as usize] = v;
    }
    fn r32(&mut self, a: u32) -> u32 {
        let o = (a & (PAGE - 1)) as usize;
        if o <= (PAGE - 4) as usize {
            self.tick();
            let p = self.page(a);
            return u32::from_le_bytes([p[o], p[o + 1], p[o + 2], p[o + 3]]);
        }
        u32::from_le_bytes([self.r8(a), self.r8(a + 1), self.r8(a + 2), self.r8(a + 3)])
    }
    fn w32(&mut self, a: u32, v: u32) {
        let o = (a & (PAGE - 1)) as usize;
        if o <= (PAGE - 4) as usize {
            self.tick();
            self.page(a)[o..o + 4].copy_from_slice(&v.to_le_bytes());
            return;
        }
        for (i, b) in v.to_le_bytes().into_iter().enumerate() {
            self.w8(a + i as u32, b);
        }
    }
    fn call(&mut self, addr: u32, a: &[u32], v: &[V4]) -> (u32, V4) {
        // PAD2: intercept FUN_001619a0 (activity event) — record for host drain
        // WIRE-A: FUN_0011ccc0(pool) fixed-size pool allocation (REPLACED: a fresh 0x100-byte block of
        // the live heap; the pools are not modelled, nothing is returned to them)
        if addr == 0x0011_ccc0 {
            return (self.alloc(0x100), [0.0; 4]);
        }
        // WIRE-A: FUN_0015e938(world, kind, oid, ctx): modes 1-5 run the 1:1 relative search
        // (game::methods::find::ee_sub_15e958, ctx = the context object); REPLACED otherwise (no world
        // handle, other modes, nothing found): the first store object with that oid at +0x18
        if addr == 0x0015_e938 {
            let oid = a.get(2).copied().unwrap_or(u32::MAX);
            let kind = a.get(1).copied().unwrap_or(0);
            if let Some(w) = self.world.clone() {
                if (1..=5).contains(&(kind & 0xff)) {
                    let ctx = a.get(3).and_then(|&c| self.obj_at(c)).map_or(0, |i| i as u32);
                    let r = crate::engine::game::methods::find::ee_sub_15e958(&w, kind, oid as i32, ctx);
                    if r != 0 {
                        if let Some(&h) = self.obj_addr.get(r as usize) {
                            return (h, [0.0; 4]);
                        }
                    }
                }
            }
            let hit = self.obj_addr.clone().into_iter().find(|&o| self.r32(o + 0x18) == oid).unwrap_or(0);
            return (hit, [0.0; 4]);
        }
        // WIRE-A: FUN_001dfd58(seq, player) -> host sequence start (record for drain)
        if addr == 0x001d_fd58 {
            if let Some(i) = a.first().and_then(|&s| self.obj_at(s)) {
                self.recorded_1dfd58.push((i, a.get(1).copied().unwrap_or(0)));
            }
            return (0, [0.0; 4]);
        }
        if addr == 0x0016_19a0 {
            self.record_1619a0(a[0], a[1] as i32, a[2] as i32);
            return (0, [0.0; 4]);
        }
        if let Some(f) = lookup(addr) {
            if self.depth >= MAX_DEPTH {
                self.stats.depth_hits += 1;
                return (0, [0.0; 4]);
            }
            self.stats.q_calls += 1;
            *self.stats.per_fn.entry(addr).or_default() += 1;
            self.depth += 1;
            let r = f(self, a, v);
            self.depth -= 1;
            return r;
        }
        if let Some(r) = trivial(addr).or_else(|| kernel_syscall(addr)) {
            self.stats.trivial += 1;
            return (r, [0.0; 4]);
        }
        let native = NATIVE.binary_search(&addr).is_ok();
        let map = if native { &mut self.stats.native } else { &mut self.stats.unported };
        let e = map.entry(addr).or_default();
        *e += 1;
        if *e == 1 {
            if self.verbose {
                log::warn!("[wire1] {} callee 0x{addr:06x} (args {a:x?})", if native { "native" } else { "unported" });
            } else {
                log::debug!("[wire1] {} callee 0x{addr:06x}", if native { "native" } else { "unported" });
            }
        }
        (0, [0.0; 4])
    }
    fn record_1619a0(&mut self, addr: u32, ev: i32, arg: i32) {
        self.recorded_1619a0.push((addr, ev, arg));
    }
    fn record_164640(&mut self, handle: u32, state: i32) {
        self.recorded_164640.push((handle, state));
    }
    fn record_gadget_purchase(&mut self, entry_id: u32, purchased: bool) {
        self.recorded_gadget_purchase.push((entry_id, purchased));
    }
}

/// Bevy wiring: a shadow image of the active world, queue slots run each frame after the class runtime.
/// WIRE-A: load-time setter methods applied by `LiveMem::method_props`: FUN_001d7748 (C2d80d0 hub
/// entry {state, button, target}, splice thunk 0x10f590 / field 1338).
const LOAD_METHODS: &[u32] = &[0x001d_7748];

#[derive(bevy_ecs::prelude::Resource, Default)]
pub struct LiveQ {
    pub mem: Option<LiveMem>,
    built_for: usize,
    pub frames: u64,
    pub t: f32,
    /// objects run per frame (last frame) / at post-load
    pub last_update: usize,
    pub post_loaded: usize,
    pub inited: usize,
    /// WP-WIRE3: post-frame snapshot of every `Owner::Queue` row of `live_ownership::TABLE` (output bus
    /// consumed by any hand-port system that reads a queue-owned field; empty on built-but-unframed worlds)
    pub queue_fields: HashMap<FieldKey, [u8; 8]>,
    /// WP-WIRE3: handle to the active `World` captured at `build` time, used for the per-frame sync
    world: Option<std::sync::Arc<World>>,
}

impl LiveQ {
    /// WP-WIRE3: handle to the world this image was built for (used by the per-frame sync).
    pub fn world(&self) -> Option<&std::sync::Arc<World>> {
        self.world.as_ref()
    }

    /// WP-ARCH1 step 2: install the world handle (so `addr_of_oid` / `call_vt_slot_by_oid` can
    /// resolve oids without going through the `tick_live_q` system).
    pub fn set_world(&mut self, w: std::sync::Arc<World>) {
        if let Some(m) = self.mem.as_mut() {
            m.world = Some(w.clone());
        }
        self.world = Some(w);
    }

    /// WP-ARCH1 step 2: guest address of the (first) object with `oid` in the active store.
    /// Returns `None` when no store is built or the oid is not in `World::find_oid`.
    pub fn addr_of_oid(&self, oid: i32) -> Option<u32> {
        let w = self.world.as_ref()?;
        let m = self.mem.as_ref()?;
        let i = *w.find_oid(oid).first()?;
        m.obj_addr.get(i).copied()
    }

    /// WP-ARCH1 step 2: dispatch slot `slot` on the object with `oid` through queue dispatch.
    /// Returns `(ok, result, v)`; `ok = false` when there is no store yet or the vtable slot is
    /// not a queue port (caller falls back to the hand-port behaviour).
    pub fn call_vt_slot_by_oid(&mut self, oid: i32, slot: u32, a: &[u32], v: &[V4]) -> (bool, u32, V4) {
        let Some(addr) = self.addr_of_oid(oid) else { return (false, 0, [0.0; 4]) };
        let Some(m) = self.mem.as_mut() else { return (false, 0, [0.0; 4]) };
        m.call_slot(addr, slot, a, v)
    }

    /// WIRE-A: FUN_0020e890 level-load tail (po selection): `oid_po_current` (the save value
    /// "oidPoCurrent", FUN_001d8a38 0x2a8a88; -1 none) resolved by FUN_0015e938(world, 5, oid, 0), else the
    /// first Sly / Bentley / Murray (DAT_002e1e40 / DAT_002dd5bc / DAT_002f7900); then vtable +0x44 (slot 17)
    /// and slot 78 (+0x27c)(po, 1). Returns the selected object index. The slot calls run in the store when
    /// the target class has a queue port (the hub C2d80d0: FUN_001d7418 -> FUN_001d7480 state 1).
    pub fn select_po_at_load(&mut self, w: &World, oid_po_current: i32) -> Option<usize> {
        let po = crate::engine::player::p20::ee_sub_20e890_po(w, oid_po_current)?;
        let m = self.mem.as_mut()?;
        let a = *m.obj_addr.get(po)?;
        m.call_slot(a, 17, &[], &[]);
        m.call_slot(a, 78, &[1], &[]);
        Some(po)
    }

    /// WP-ARCH1 step 2: `FUN_0022b490`-style message delivery through vt+0x1c (message slot).
    /// `event` and `arg_handle` match `JobMgr::ev_msgs` 0x11 payloads (vt+0x1c, nargs 2).
    pub fn deliver_message(&mut self, dest_oid: i32, event: i32, arg_handle: u32) -> bool {
        let slot = SLOT_MESSAGE;
        self.call_vt_slot_by_oid(dest_oid, slot, &[event as u32, arg_handle], &[]).0
    }

    /// (Re)build for `w` (post-load slot 14 for queue-port targets).
    pub fn build(&mut self, w: &World) {
        let mut m = LiveMem::build(w);
        m.world = self.world.clone();
        // slot 6 init(obj, parent) in creation order, then slot 14 post-load (FUN_0013da18 order)
        self.inited = 0;
        for i in 0..m.obj_addr.len() {
            let o = m.obj_addr[i];
            let f = m.slot_target(o, SLOT_INIT);
            if f != 0 && is_queue(f) {
                let parent = m.r32(o + 0x10);
                m.enter(o, SLOT_INIT, &[parent], &[]);
                self.inited += 1;
            }
        }
        self.post_loaded = m.run_slot(SLOT_POSTLOAD, &[], &[], |_| true);
        self.mem = Some(m);
        self.frames = 0;
        self.t = 0.0;
    }

    /// One frame: clock + slot 15 (FUN_00226550 ObjectUpdate) on in-world objects with a queue-port target.
    /// WP-WIRE3: pre-frame hand -> guest copy, then post-frame guest -> `queue_fields` snapshot.
    pub fn frame(&mut self, dt: f32) {
        let Some(m) = self.mem.as_mut() else { return };
        self.t += dt;
        m.set_clock(self.t, dt, dt);
        if let Some(w) = self.world.as_ref() {
            copy_hand_to_guest(m, &m.obj_addr.clone(), w);
        }
        let mut flags = Vec::with_capacity(m.obj_addr.len());
        for i in 0..m.obj_addr.len() {
            flags.push(m.in_world(i));
        }
        self.last_update = m.run_slot(SLOT_UPDATE, &[], &[], |i| flags[i]);
        if let Some(w) = self.world.as_ref() {
            let obj = m.obj_addr.clone();
            snapshot_queue(m, &obj, w, &mut self.queue_fields);
        }
        self.frames += 1;
    }

    /// PAD1: call the static UI screen updates from FUN_001d9648 (UiUpdate main-loop step).
    /// Specifically FUN_001db870(0x3da150) — the planning-table / ThiefNet menu.
    pub fn ui_update(&mut self) {
        let Some(m) = self.mem.as_mut() else { return };
        // FUN_001d9648 calls FUN_001db870(0x3da150) among others.
        // 0x3da150 is the C2d84e8 singleton (ThiefNet / planning-table menu).
        m.call(0x001d_b870, &[0x3d_a150], &[]);
    }
}

pub fn tick_live_q(level: bevy_ecs::prelude::Res<crate::engine::classes::plugin::ClassesLevel>, mut q: bevy_ecs::prelude::ResMut<LiveQ>) {
    let Some(w) = &level.0 else { return };
    let id = std::sync::Arc::as_ptr(w) as usize;
    if q.built_for != id {
        q.build(w);
        q.world = Some(w.clone());
        q.built_for = id;
        if let Some(m) = &q.mem {
            log::info!("[wire1] live queue image: {} objects, init {} / post-load {} queue slots; {}", m.obj_addr.len(), q.inited, q.post_loaded, m.stats.summary());
        }
    }
    q.frame(1.0 / 60.0);
    if q.frames % 600 == 0 {
        if let Some(m) = &q.mem {
            log::info!("[wire1] frame {}: {} objects updated by queue code; {}", q.frames, q.last_update, m.stats.summary());
        }
    }
}

pub struct LiveQPlugin;
impl bevy_app::Plugin for LiveQPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        use bevy_ecs::prelude::*;
        app.init_resource::<LiveQ>().add_systems(bevy_app::Update, tick_live_q.after(crate::engine::classes::plugin::tick_classes));
    }
}
