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
    /// ANIM-ALL R20: world ASEGA passes (FUN_00226550's world+0x3d88 loop) aborted by a panic / the budget
    pub asega_aborts: u64,
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
        // WIRE-Q2: merge with the rows already in the file (other live tests' reached sets survive)
        let mut v: std::collections::BTreeMap<u32, u64> = self.per_fn.clone().into_iter().collect();
        if let Ok(old) = std::fs::read_to_string(path) {
            for l in old.lines() {
                let mut it = l.split('\t');
                if let (Some(a), Some(n)) = (it.next(), it.next()) {
                    if let (Ok(a), Ok(n)) = (u32::from_str_radix(a.trim_start_matches("0x"), 16), n.trim().parse::<u64>()) {
                        *v.entry(a).or_default() += n;
                    }
                }
            }
        }
        let mut s = String::with_capacity(v.len() * 24);
        for (a, n) in &v {
            s.push_str(&format!("0x{:06x}\t{}\n", a, n));
        }
        std::fs::write(path, s)?;
        Ok(self.per_fn.len())
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
    pages: HashMap<u32, Box<[u8; PAGE as usize]>, PageHash>,
    /// guest address of `World::objects[i]`
    pub obj_addr: Vec<u32>,
    by_addr: BTreeMap<u32, usize>,
    heap: u32,
    /// WIRE-A: 0x100-byte blocks returned through FUN_0011cd98 (pool free), reused by FUN_0011ccc0
    pool_free: Vec<u32>,
    depth: u32,
    /// guest call stack (addr, a0, a1); left as-is by an unwind (ANIMALL_DBG failed-start diagnostics)
    pub cstack: Vec<(u32, u32, u32)>,
    ops: u64,
    pub stats: LiveStats,
    /// log the first call of every unported address (env SLY2_WIRE_LOG)
    pub verbose: bool,
    /// PAD1: recorded FUN_001619a0(addr, event, arg) calls from QMem (drained by the host)
    pub recorded_1619a0: Vec<(u32, i32, i32)>,
    /// BOSS R25: FUN_001a61e0 (LO vt+0x1c) script messages (object address, msg, argument words) of the
    /// live image; drained by the host (classes::q::barrel::script_msgs)
    pub recorded_1a61e0: Vec<(u32, u32, Vec<u32>)>,
    /// CRAWL R103: SMA (C2d9768) script messages (vt +0x1c = FUN_001331f8) of the live image: (SMA address,
    /// msg, argument words); drained by the host (classes::q::barrel::script_msgs) into the level script
    pub recorded_sma_msgs: Vec<(u32, u32, Vec<u32>, u32, u32)>,
    /// PAD1: recorded FUN_00164640(handle, state) calls from QMem (native calls routed to host)
    pub recorded_164640: Vec<(u32, i32)>,
    /// PAD1: recorded gadget-purchase save writes from QMem (entry_id, purchased)
    pub recorded_gadget_purchase: Vec<(u32, bool)>,
    /// WIRE-A: FUN_001dfd58(seq, player) calls from QMem (object index of the sequence, player word); the
    /// sequences run in the class runtime, so the host drains these into `EngineHost::seq_starts`
    pub recorded_1dfd58: Vec<(usize, u32)>,
    /// WIRE-A: world handle for the FUN_0015e938 relative search (set by `LiveQ::build` / `set_world`)
    pub world: Option<std::sync::Arc<World>>,
    /// ANIM-ALL (load_alos): guest stream end of the current ALO's nested object list; FUN_0013fb40 on that
    /// stream skips there (the nested objects already exist in the image, built from `World::objects`)
    pub nested_end: Option<(u32, u32)>,
    /// ANIM-ALL: level-load class loads (byte-wise stream copies of whole meshes) get 25x the per-call budget
    pub load_pass: bool,
    /// WIRE-UI: host SM (`anim::sm::Sm`) of every guest SM object (+0x50/+0x54 states), for FUN_001f0660
    pub sms: HashMap<u32, crate::engine::anim::sm::Sm>,
    /// ANIM-ALL: guest ALOs whose skeleton FUN_0022d7a0 loaded (`load_alos`); the only objects `pose_pass` touches
    /// (+0x50 / +0x54 mean other things on non-ALO classes, e.g. the SM state table)
    pub alos: Vec<u32>,
    /// ANIM-ALL: env SLY2_GUEST_POSE=1 (or set by tests) runs `pose_pass` each frame
    pub guest_pose: bool,
    /// ANIM-ALL R15: guest ASEGA ref words (FUN_001305b8 / FUN_00130658 slots) of host-driven actors, by key
    pub anim_slots: HashMap<u32, u32>,
    /// WIRE-AN: host side of the native save-value store 0x3d4a60 (FUN_001d8a38 / FUN_001d8b20 path values,
    /// FUN_002581c0 / FUN_001d80c8 symbol-keyed values): path -> Splice value {type, payload}
    pub save: BTreeMap<String, [u32; 2]>,
    /// WIRE-JOB2: FUN_001d87c0 name-path objects {len, buf} -> their path (so the symbol-keyed
    /// FUN_002581c0 / FUN_001d80c8 values share keys with the path calls)
    pub save_handles: BTreeMap<u32, String>,
    /// CRAWL R27: FUN_001d87c0 out -> (buf, capacity)
    pub path_bufs: BTreeMap<u32, (u32, u32)>,
    /// SAVE2: vector / clq store values (Splice 0x1a / 0x1c): path -> 16-byte payload (the store word is a
    /// live-heap block holding the same words)
    pub save_vec: crate::engine::game::save::SaveVecs,
    /// SAVE2: card file location for the menu's slot reads FUN_001ac4c8 (set by `LiveQ::build`)
    pub card: Option<crate::engine::game::save::MemoryCard>,
    /// SAVE2: FUN_001d84b0 slot copies of the store (slot -> store, vectors) and the card writes / clears /
    /// loads issued by the card menu (drained by `LiveQ::drain_card_ops`)
    pub card_snaps: HashMap<usize, (BTreeMap<String, [u32; 2]>, crate::engine::game::save::SaveVecs)>,
    pub card_ops: Vec<CardOp>,
    /// WIRE-JOB: guest address of the host-owned Sly JT (class 53 C2d2e98, `player::Jt`; its init FUN_00188488
    /// writes DAT_002e1e40 = self), 0 = none
    pub host_jt: u32,
    /// WIRE-JOB: FUN_00141fc8(jt, slot, gadget) calls on `host_jt` (applied to the host JT by `live_jt_bridge`)
    pub recorded_141fc8: Vec<(usize, i32)>,
}

impl LiveMem {
    fn cstr(&mut self, mut p: u32) -> String {
        let mut v = Vec::new();
        while p != 0 && v.len() < 256 {
            let b = self.r8(p);
            if b == 0 {
                break;
            }
            v.push(b);
            p += 1;
        }
        String::from_utf8_lossy(&v).into_owned()
    }
    /// path of the 0-terminated component strings `c` (FUN_001d87c0)
    fn save_path(&mut self, c: &[u32]) -> String {
        let mut k = String::new();
        for &p in c.iter().take_while(|&&p| p != 0) {
            if !k.is_empty() {
                k.push('/');
            }
            k += &self.cstr(p);
        }
        k
    }
    /// WIRE-AN: the native save-value calls (Splice values are {type, payload} pairs; missing = {-1, 0})
    fn save_native(&mut self, addr: u32, a: &[u32]) -> Option<u32> {
        let arg = |i: usize| a.get(i).copied().unwrap_or(0);
        let get = |m: &mut Self, out: u32, k: String| {
            let v = m.save.get(&k).copied().unwrap_or([u32::MAX, 0]);
            m.w32(out, v[0]);
            m.w32(out + 4, v[1]);
            out
        };
        let put = |m: &mut Self, k: String, val: u32| {
            let mut v = [m.r32(val), m.r32(val + 4)];
            // SAVE2: vector / clq payloads are copied into the store (own block + side table)
            if v[0] == crate::engine::game::save::TAG_VECTOR || v[0] == crate::engine::game::save::TAG_CLQ {
                let q = [m.r32(v[1]), m.r32(v[1] + 4), m.r32(v[1] + 8), m.r32(v[1] + 12)];
                let b = m.alloc(16);
                for (i, w) in q.iter().enumerate() {
                    m.w32(b + 4 * i as u32, *w);
                }
                v[1] = b;
                m.save_vec.insert(k.clone(), q);
            } else {
                m.save_vec.remove(&k);
            }
            m.save.insert(k, v);
        };
        Some(match addr {
            // WIRE-JOB: FUN_001da858(o) (screen stick selection, NATIVE in the generated table): pad o+0x10; with no
            // direction held (+0x28 == -1) a stick magnitude (+0x1044) > 0.8 picks one of 8 directions
            // (atan2(+0x1040, +0x103c) * 4/pi + 8.5, mod 8) -> vt +0x74 (slot 13)(o, dir); while one is held, a
            // magnitude < 0.5 releases it (slot 13(o, -1)).
            0x001d_a858 => {
                let o = arg(0);
                let p = if o != 0 && o < 0x0200_0000 { self.r32(o + 0x10) } else { 0 };
                if p != 0 && p < 0x0200_0000 {
                    let mag = f32::from_bits(self.r32(p + 0x1044));
                    if self.r32(o + 0x28) == u32::MAX {
                        if 0.8 < mag {
                            let ang = f32::from_bits(self.r32(p + 0x1040)).atan2(f32::from_bits(self.r32(p + 0x103c)));
                            let i = (ang * 1.273_239_5 + 8.5) as i32;
                            let i1 = if i < 0 { i + 7 } else { i };
                            let f = self.slot_target(o, 13);
                            self.call(f, &[o, (i - (i1 >> 3) * 8) as u32], &[]);
                        }
                    } else if mag < 0.5 {
                        let f = self.slot_target(o, 13);
                        self.call(f, &[o, u32::MAX], &[]);
                    }
                }
                0
            }
            // WIRE-JOB: FUN_0021ecd8(buf, fmt, ..) = libc sprintf (NATIVE in the generated table; the save paths
            // "mppukfAcquired[%d]" of FUN_001daf38 / FUN_001daea8 are built with it): %d %i %u %x %s %c and %%
            // with integer register arguments
            0x0021_ecd8 => {
                let (buf, fmt) = (arg(0), arg(1));
                let f = self.cstr(fmt);
                let mut out = String::new();
                let mut n = 2;
                let mut it = f.chars().peekable();
                while let Some(c) = it.next() {
                    if c != '%' {
                        out.push(c);
                        continue;
                    }
                    while it.peek().is_some_and(|c| c.is_ascii_digit() || *c == '-' || *c == 'l') {
                        it.next();
                    }
                    let v = arg(n);
                    match it.next() {
                        Some('d') | Some('i') => { out += &(v as i32).to_string(); n += 1 }
                        Some('u') => { out += &v.to_string(); n += 1 }
                        Some('x') => { out += &format!("{v:x}"); n += 1 }
                        Some('s') => { out += &self.cstr(v); n += 1 }
                        Some('c') => { out.push(v as u8 as char); n += 1 }
                        Some(c) => out.push(c),
                        None => {}
                    }
                }
                for (i, b) in out.bytes().chain([0]).enumerate() {
                    self.w8(buf + i as u32, b);
                }
                out.len() as u32
            }
            // WIRE-JOB: FUN_001dc4b0(o, d) (C2d84e8_v13, ThiefNet stick direction, NATIVE in the generated table;
            // `ui::thiefnet::Shop::FUN_001dc4b0`): unless d == +0x28: +0x28 = d, +0x2c = clock DAT_002ddf0c;
            // jump table 0x2a79f0: 0 -> FUN_001dc260(o, 1), 4 -> FUN_001dc260(o, 0), 1..3 -> up, 5..7 -> down
            // (FUN_001dbbe0(o, sel -/+ 1) on +0x24c within +0x48 rows, sound 0x35; at the end sound 0x36)
            0x001d_c4b0 => {
                let (o, d) = (arg(0), arg(1) as i32);
                if d != self.r32(o + 0x28) as i32 {
                    self.w32(o + 0x28, d as u32);
                    let t = self.r32(0x002d_df0c);
                    self.w32(o + 0x2c, t);
                    let (sel, n) = (self.r32(o + 0x24c) as i32, self.r32(o + 0x48) as i32);
                    let snd = |m: &mut Self, k: u32| {
                        let s = m.call(0x001f_bb90, &[k], &[]).0;
                        m.call(0x001f_b2f8, &[s, 0], &[]);
                    };
                    match d {
                        0 => { self.call(0x001d_c260, &[o, 1], &[]); }
                        4 => { self.call(0x001d_c260, &[o, 0], &[]); }
                        1..=3 if 0 < sel => { self.call(0x001d_bbe0, &[o, (sel - 1) as u32], &[]); snd(self, 0x35) }
                        5..=7 if sel < n - 1 => { self.call(0x001d_bbe0, &[o, (sel + 1) as u32], &[]); snd(self, 0x35) }
                        1..=3 | 5..=7 => snd(self, 0x36),
                        _ => {}
                    }
                }
                0
            }
            // FUN_001291a8 / FUN_00129658 (dst, src): value copy / assign; FUN_00129ec8 release
            0x0012_91a8 | 0x0012_9658 => {
                let (d, s) = (arg(0), arg(1));
                let v = [self.r32(s), self.r32(s + 4)];
                self.w32(d, v[0]);
                self.w32(d + 4, v[1]);
                d
            }
            // FUN_001d8a38(out, path..., 0)
            0x001d_8a38 => {
                let k = self.save_path(&a[1.min(a.len())..]);
                get(self, arg(0), k)
            }
            // FUN_001d8b20(out, val, path..., 0)
            0x001d_8b20 => {
                let k = self.save_path(&a[2.min(a.len())..]);
                put(self, k.clone(), arg(1));
                get(self, arg(0), k)
            }
            // WIRE-JOB2: FUN_001d87c0(name, va names..0, out): out = {len, buf} of the name path (one u16 per
            // name; REPLACED: the symbol table 0x3e0eb0 is not modelled, the path string keys the store)
            0x001d_87c0 => {
                let blk = arg(1);
                let mut names = vec![arg(0)];
                for i in 0..7 {
                    let p = if names.last() == Some(&0) { 0 } else { self.r32(blk + 4 * i) };
                    names.push(p);
                    if p == 0 {
                        break;
                    }
                }
                let k = self.save_path(&names);
                let n = names.iter().take_while(|&&p| p != 0).count() as u32;
                let out = arg(2);
                // CRAWL R27: one heap block per out (the bump heap never frees; scripts re-run this every frame)
                let need = (2 * n).max(4);
                let buf = match self.path_bufs.get(&out) {
                    Some(&(b, cap)) if cap >= need => b,
                    _ => {
                        let b = self.alloc(need.max(0x20));
                        self.path_bufs.insert(out, (b, need.max(0x20)));
                        b
                    }
                };
                self.w32(out, n);
                self.w32(out + 4, buf);
                self.save_handles.insert(out, k);
                out
            }
            // FUN_002581c0(out, key, store)
            0x0025_81c0 => {
                let k = self.save_handles.get(&arg(1)).cloned().unwrap_or_else(|| format!("#{:x}", arg(1)));
                get(self, arg(0), k)
            }
            // FUN_001d80c8(out, root, key, val, store)
            0x001d_80c8 => {
                let k = self.save_handles.get(&arg(2)).cloned().unwrap_or_else(|| format!("#{:x}", arg(2)));
                put(self, k.clone(), arg(3));
                get(self, arg(0), k)
            }
            // FUN_0020efc0(world, f) / FUN_0020f030(world, f): world pause count +0x3d78; first pause
            // FUN_00187078(2), last resume FUN_001870a0(2) + FUN_00187e08(0x2dfc00, 8). The f & 1 / f & 2
            // callees (FUN_0020f128/f198, FUN_0020f160/f1a8) are not ported (ThiefNet passes 0).
            0x0020_efc0 | 0x0020_f030 => {
                let w = arg(0);
                let up = addr == 0x0020_efc0;
                let n = self.r32(w + 0x3d78).wrapping_add(if up { 1 } else { u32::MAX });
                self.w32(w + 0x3d78, n);
                if up && n == 1 {
                    self.save_native(0x0018_7078, &[2]);
                } else if !up && n == 0 {
                    self.save_native(0x0018_70a0, &[2]);
                    let mut p = read_pad(self, 0x002d_fc00);
                    crate::engine::input::pad::FUN_00187e08(&mut p, 8);
                    write_pad(self, 0x002d_fc00, &p);
                }
                if arg(1) & 3 != 0 {
                    log::warn!("[wire-an] FUN_{addr:08x} flags {:x} callees not ported", arg(1));
                }
                0
            }
            // FUN_00187078(b) / FUN_001870a0(b): DAT_002dfbc4 |= b / &= !b, then FUN_001870d0
            0x0018_7078 | 0x0018_70a0 => {
                let f = self.r32(0x002d_fbc4);
                let f = if addr == 0x0018_7078 { f | arg(0) } else { f & !arg(0) };
                self.w32(0x002d_fbc4, f);
                self.save_native(0x0018_70d0, &[]);
                0
            }
            // FUN_001870d0: pad routing DAT_002dfbc0 from the DAT_002dfbc4 bits; bit 2 clear -> DAT_002e0cbc = 0xffff
            0x0018_70d0 => {
                let f = self.r32(0x002d_fbc4);
                let v = if f & 0x10 != 0 {
                    5
                } else if f & 0xc != 0 {
                    0
                } else if f & 1 != 0 {
                    5
                } else if f & 2 != 0 {
                    4
                } else {
                    7
                };
                self.w32(0x002d_fbc0, v);
                if v & 2 == 0 {
                    self.w32(0x002e_0cbc, 0xffff);
                }
                0
            }
            _ => return None,
        })
    }
}

impl Default for LiveMem {
    fn default() -> Self {
        LiveMem {
            pages: HashMap::default(),
            obj_addr: Vec::new(),
            world: None,
            nested_end: None,
            load_pass: false,
            by_addr: BTreeMap::new(),
            heap: HEAP_BASE,
            pool_free: Vec::new(),
            depth: 0,
            cstack: Vec::new(),
            ops: 0,
            stats: LiveStats::default(),
            verbose: std::env::var_os("SLY2_WIRE_LOG").is_some(),
            recorded_1619a0: Vec::new(),
            recorded_1a61e0: Vec::new(),
            recorded_164640: Vec::new(),
            recorded_gadget_purchase: Vec::new(),
            recorded_1dfd58: Vec::new(),
            recorded_sma_msgs: Vec::new(),
            sms: HashMap::new(),
            alos: Vec::new(),
            guest_pose: std::env::var("SLY2_GUEST_POSE").map_or(true, |v| v != "0"),
            anim_slots: HashMap::new(),
            save: BTreeMap::new(),
            save_handles: BTreeMap::new(),
            path_bufs: BTreeMap::new(),
            save_vec: Default::default(),
            card: None,
            card_snaps: HashMap::new(),
            card_ops: Vec::new(),
            host_jt: 0,
            recorded_141fc8: Vec::new(),
        }
    }
}

impl LiveMem {
    /// CRAWL R103: read-only word of an already-materialised page (0 when the page was never touched)
    pub fn peek32(&self, a: u32) -> u32 {
        let o = (a & (PAGE - 1)) as usize;
        match self.pages.get(&(a & !(PAGE - 1))) {
            Some(p) if o <= (PAGE - 4) as usize => u32::from_le_bytes([p[o], p[o + 1], p[o + 2], p[o + 3]]),
            _ => 0,
        }
    }
    /// CRAWL R103: the world focus (0x3dd490 +0x3efc) is the safehouse hub C2d80d0 in its select state 1
    /// (FUN_001d7418 -> FUN_001c3ed8(hub, 1) took it): the JT's FUN_001c3ed8(jt, 0) -> FUN_001c4640(jt+0x150, 0)
    pub fn hub_has_focus(&self) -> bool {
        let f = self.peek32(WORLD + 0x3efc);
        f != 0 && self.peek32(f + 0xc) == 0x002d_80d0 && self.peek32(f + 0x180) == 1 && std::env::var_os("SLY2_NO_R103_HUBFOCUS").is_none()
    }
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
        if self.ops > if self.load_pass { BUDGET * 25 } else { BUDGET } && self.depth > 0 {
            if std::env::var_os("SLY2_BUDGET_DBG").is_some() {
                eprintln!("[budget] ops {} load {} stack {:x?}", self.ops, self.load_pass, self.cstack);
            }
            std::panic::panic_any(BudgetExceeded);
        }
    }

    /// object index at guest address `a` (object start)
    pub fn obj_at(&self, a: u32) -> Option<usize> {
        self.by_addr.get(&a).copied()
    }

    /// ANIM-ALL: the original ASEG load FUN_0012f2a8(obj, stream) for every loaded class-109/110 object, on a guest
    /// copy of the level bytes (`data` = the level stream the world was loaded from): a stream {+4 cursor, +8 bytes
    /// left} at each object's payload, so channels (+0x5c/+0x60, FUN_0012b3b0 / FUN_0012cb98 curves), scalar
    /// channels (+0x64/+0x68), events (+0x98..+0xac, FUN_0012f590) and length (+0x74) are in the guest layout.
    /// Returns (loaded, failed).
    pub fn load_asegs(&mut self, w: &World, data: &[u8]) -> (usize, usize) {
        if std::env::var("ANIMALL_NO_ASEGS").is_ok() {
            return (0, 0);
        }
        use crate::engine::world::objects::{LoadData, Payload};
        let idx: Vec<usize> = (0..w.objects.len().min(self.obj_addr.len()))
            .filter(|&i| matches!(w.objects[i].payload, Payload::Loaded(LoadData::Aseg(_))))
            .collect();
        if idx.is_empty() {
            return (0, 0);
        }
        // the bottom-up level scratch arena FUN_001ae1d0 allocates from (DAT_002f6380 base / +0x84 bottom /
        // +0x88 top); unset in the image -> grew from address 0 into the ELF. A host-heap block stands in.
        if QMem::r32(self, 0x002f_6380) == 0 {
            let n = 0x80_0000;
            let a = self.alloc(n);
            QMem::w32(self, 0x002f_6380, a);
            QMem::w32(self, 0x002f_6384, a);
            QMem::w32(self, 0x002f_6388, a + n);
        }
        // one reusable guest buffer holding the current object's payload (stream +4 cursor, +8 bytes left)
        let max = idx.iter().map(|&i| w.objects[i].payload_size.max(0) as u32).max().unwrap_or(0);
        let base = self.alloc(max + 0x10);
        let st = self.alloc(0x40);
        let (mut ok, mut bad) = (0, 0);
        for i in idx {
            let o = &w.objects[i];
            let (off, len) = (o.payload_offset as usize, o.payload_size.max(0) as usize);
            let Some(src) = data.get(off..off + len) else {
                bad += 1;
                continue;
            };
            for (k, &x) in src.iter().enumerate() {
                QMem::w8(self, base + k as u32, x);
            }
            QMem::w32(self, st + 4, base);
            QMem::w32(self, st + 8, len as u32);
            let a = self.obj_addr[i];
            let sp = QMem::r32(self, super::q2mem::SCRATCH_SP);
            let r = self.call_top(0x0012_f2a8, &[a, st]);
            QMem::w32(self, super::q2mem::SCRATCH_SP, sp);
            let used = base + len as u32 == QMem::r32(self, st + 4);
            if r.is_some() && used {
                ok += 1;
            } else {
                bad += 1;
            }
        }
        (ok, bad)
    }

    /// ANIM-ALL: the original ALO class loads (vtable slot 13 = vt+0x74, FUN_001f31e0 and its wrappers) on every
    /// loaded ALO of the image: bones (FUN_0022d7a0: +0x50 count, +0x54 0x100-byte records, oid -> bone via
    /// vt+0x19c), parts (+0xe0 / +0xe4 0xb0-byte records), properties. The nested object list (FUN_0013fb40) is
    /// skipped: those objects are already in the image (by oid, under this ALO). Returns (loaded, failed).
    pub fn load_alos(&mut self, w: &World, data: &[u8]) -> (usize, usize) {
        if std::env::var("ANIMALL_NO_ALOS").is_ok() {
            return (0, 0);
        }
        use crate::engine::world::objects::{LoadData, Payload};
        let idx: Vec<usize> = (0..w.objects.len().min(self.obj_addr.len()))
            .filter(|&i| matches!(w.objects[clone_root(w, i)].payload, Payload::Loaded(LoadData::Alo { .. })))
            .collect();
        let max = idx.iter().map(|&i| w.objects[clone_root(w, i)].payload_size.max(0) as u32).max().unwrap_or(0);
        let base = self.alloc(max + 0x10);
        let st = self.alloc(0x40);
        let (mut ok, mut bad) = (0, 0);
        for i in idx {
            // BOSS R24: a FUN_0013e510 copy (Payload::Clone) of a loaded ALO: vt+0x144 (FUN_001f66f0) copies the
            // source's loaded state. REPLACED: the copy runs the same skeleton load from the source's stream
            // (same bytes = same state; its nested objects are the copy's own children, found by oid).
            let o = &w.objects[clone_root(w, i)];
            let (off, len) = (o.payload_offset as usize, o.payload_size.max(0) as usize);
            let Some(src) = data.get(off..off + len) else {
                bad += 1;
                continue;
            };
            // the nested list ends with the last direct child's record (its payload end); empty list = s16 count
            let end = o.children.iter().map(|&c| &w.objects[c]).filter(|c| c.payload_offset as usize >= off).map(|c| c.payload_offset + c.payload_size.max(0) as u32).max();
            for (k, &x) in src.iter().enumerate() {
                QMem::w8(self, base + k as u32, x);
            }
            QMem::w32(self, st + 4, base);
            QMem::w32(self, st + 8, len as u32);
            let a = self.obj_addr[i];
            // no children: 0 = skip only the s16 count (cursor + 2 when FUN_0013fb40 runs)
            self.nested_end = Some((st, end.map_or(0, |e| base + (e - off as u32))));
            let sp = QMem::r32(self, super::q2mem::SCRATCH_SP);
            self.ops = 0;
            let pre = if std::env::var("ANIMALL_DBG").is_ok() && bad == 0 { Some(self.stats.per_fn.clone()) } else { None };
            self.load_pass = true;
            // the bone part of the ALO load (FUN_001f31e0 -> FUN_0022d7a0, the stream head): the mesh set
            // FUN_00167320 that follows still desyncs in the guest (ANIM-ALL log), so only the skeleton loader runs
            let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| QMem::call(self, 0x0022_d7a0, &[a, st], &[])));
            self.load_pass = false;
            if let (Some(pre), true) = (pre, r.is_err()) {
                let mut d: Vec<(u64, u32)> = self.stats.per_fn.iter().map(|(&k, &v)| (v - pre.get(&k).copied().unwrap_or(0), k)).filter(|x| x.0 > 0).collect();
                d.sort_unstable_by(|a, b| b.cmp(a));
                eprintln!("load_alos first failure top calls: {:x?}", &d[..d.len().min(12)]);
            }
            self.depth = 0;
            self.cstack.clear();
            self.nested_end = None;
            QMem::w32(self, super::q2mem::SCRATCH_SP, sp);
            // FUN_0022d7a0's stream extent = the native skeleton parse (world::alo::ee_sub_22d7a0)
            let mut ns = crate::engine::disc::level_file::memory_stream(src.to_vec());
            let _ = crate::engine::world::alo::ee_sub_22d7a0(&mut ns);
            let used = base + ns.tell() == QMem::r32(self, st + 4);
            if std::env::var("ANIMALL_DBG").is_ok() {
                let msg = r.as_ref().err().map(|e| e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_else(|| if e.downcast_ref::<BudgetExceeded>().is_some() { "budget".into() } else { "?".into() }));
                eprintln!("load_alos {i} class {} len {len} used {used} consumed {} err {msg:?}", o.class, QMem::r32(self, st + 4).wrapping_sub(base));
            }
            if r.is_ok() && used {
                ok += 1;
                self.alos.push(a);
            } else {
                bad += 1;
            }
        }
        (ok, bad)
    }

    /// reserve `n` bytes of the object heap (16-aligned, zeroed)
    pub fn alloc(&mut self, n: u32) -> u32 {
        let a = self.heap;
        self.heap = (self.heap + n.max(0x10) + 15) & !15;
        if (a >> 20) != (self.heap >> 20) && std::env::var_os("SLY2_HEAP_DBG").is_some() {
            eprintln!("HEAP {:#x} n {n:#x}
{}", self.heap, std::backtrace::Backtrace::force_capture());
        }
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
        // WIRE-Q2: the world's +0x64 child list (entry-level objects, parent WORLD), same layout
        let roots: Vec<u32> = w.objects.iter().enumerate().filter(|(_, o)| o.owner.is_none()).map(|(i, _)| m.obj_addr[i]).collect();
        m.w32(WORLD + 0x64, roots.first().copied().unwrap_or(0));
        m.w32(WORLD + 0x68, roots.last().copied().unwrap_or(0));
        m.w32(WORLD + 0x6c, 0x24);
        // BOSS R21: the world's proximity list +0x20 (link offset +0x28 = 4, FUN_00147f40 / FUN_00147e20) and its
        // inside set +0x2c (link offset +0x34 = 0xc) as in the PCSX2 snapshots of 0x3dd490 (rec11); without the
        // offsets FUN_00147f40 walks a list linked at +0 and never meets its cursor
        m.w32(WORLD + 0x28, 4);
        m.w32(WORLD + 0x34, 0xc);
        for k in roots.windows(2) {
            m.w32(k[0] + 0x24, k[1]);
            m.w32(k[1] + 0x28, k[0]);
        }
        for (i, o) in w.objects.iter().enumerate() {
            let a = m.obj_addr[i];
            let vt = CLASSES.get(o.class as usize).map_or(0, |c| c.vtable);
            m.w32(a + 0xc, vt);
            m.w32(a + 0x10, o.owner.map_or(WORLD, |p| m.obj_addr[p]));
            m.w32(a + 0x18, o.oid as u32);
            // children: +0x64 first, +0x24 next sibling (null-terminated)
            // WIRE-Q2: the +0x64 child list is a FUN_0011a220 list {head, tail +0x68, link offset +0x6c = 0x24
            // (FUN_0011a1f8 in the LO constructor)}: next +0x24, prev +0x28 (the low word under the flags)
            let kids: Vec<u32> = o.children.iter().map(|&c| m.obj_addr[c]).collect();
            m.w32(a + 0x64, kids.first().copied().unwrap_or(0));
            m.w32(a + 0x68, kids.last().copied().unwrap_or(0));
            m.w32(a + 0x6c, 0x24);
            for k in kids.windows(2) {
                m.w32(k[0] + 0x24, k[1]);
                m.w32(k[1] + 0x28, k[0]);
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
        m.build_dir(w);
        m.build_sms(w);
        m
    }

    /// WIRE-JOB2: the world directory w+0x3f04 / +0x3f08 (FUN_0013da18: 0x6c-byte entries, +0x1c name,
    /// +0x5c a, +0x60 b, +0x10 c, +0x14 d, +8 copies n, +0x18 oid per copy; FUN_0013fb40 +0 m objects,
    /// +4 m * n instances; FUN_001ca370 +0xc copies placed) and FUN_001c9b08's in-place spawners
    /// (+0xe0 = entry, +0x38 = last instance's +0x38).
    pub fn build_dir(&mut self, w: &World) {
        let n = w.entries.len() as u32;
        let t = self.alloc((n * 0x6c).max(4));
        self.w32(WORLD + 0x3f04, n);
        self.w32(WORLD + 0x3f08, t);
        // CRAWL R27: the deferred-call list w+0x3ca8 {head, tail, link offset +8}: its nodes link at +0x10
        // (FUN_00149c80 walks node+0x10; FUN_0011a070 appends through header+8). REPLACED: the world
        // constructor's list init is not run on the image, so the offset is set here.
        if self.r32(WORLD + 0x3cb0) == 0 && std::env::var_os("LIVE_NO_DEFER").is_none() {
            self.w32(WORLD + 0x3cb0, 0x10);
        }
        for (i, e) in w.entries.iter().enumerate() {
            let r = t + i as u32 * 0x6c;
            for (k, &b) in e.dir.fid.iter().enumerate() {
                self.w8(r + 0x1c + k as u32, b);
            }
            for (off, v) in [(0x5c, e.dir.a), (0x60, e.dir.b), (0x10, e.dir.c), (0x14, e.dir.d), (8, e.dir.n), (0xc, e.load_count), (0, e.objects.len() as i32)] {
                self.w32(r + off, v as u32);
            }
            let ids = self.alloc((4 * e.instances.len() as u32).max(4));
            for (k, &o) in e.instances.iter().enumerate() {
                let a = self.obj_addr[o];
                self.w32(ids + 4 * k as u32, a);
            }
            self.w32(r + 4, ids);
            let oids = self.alloc((4 * e.dir.n.max(0) as u32).max(4));
            for k in 0..e.dir.n.max(0) as usize {
                self.w32(oids + 4 * k as u32, e.inst_oids.get(k).copied().unwrap_or(-1) as u32);
            }
            self.w32(r + 0x18, oids);
        }
        use crate::engine::world::loads_b4::B4Load;
        use crate::engine::world::objects::{LoadData, Payload};
        for i in 0..w.objects.len() {
            // BOSS R23: a FUN_0013e510 copy of an in-place spawner (Payload::Clone: vtable +0x144 copies the
            // source's state, +0xe0 / +0x38 included), e.g. the EP3 row-13 TNT dispenser copy 2829's spawner
            let mut k = i;
            while let (Payload::Clone, Some(c)) = (&w.objects[k].payload, w.objects[k].clone_of) {
                k = c;
            }
            let Payload::Loaded(LoadData::B4(d)) = &w.objects[k].payload else { continue };
            let B4Load::Class6 { in_place: true, template, .. } = **d else { continue };
            let Some(g) = w.dir.iter().position(|d| d.c == template) else { continue };
            let (a, r) = (self.obj_addr[i], t + g as u32 * 0x6c);
            self.w32(a + 0xe0, r);
            let m0 = self.r32(r) as i32;
            if 0 < m0 {
                let ids = self.r32(r + 4);
                let nc = self.r32(r + 8) as i32;
                let last = self.r32(ids + 4 * (m0 * nc - 1) as u32);
                let v = self.r32(last + 0x38);
                self.w32(a + 0x38, v);
            }
        }
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
            self.sms.insert(a, sm);
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
    /// BOSS R24: the load-time placement of the image objects (the guest skeletons must be loaded, `load_alos`):
    /// - FUN_001c9b08's in-place class-6 spawner (and its FUN_0013e510 copies, which copy the source's state):
    ///   +0x50 = FUN_0022d6b0(bone, owner, 0), +0x60 = the FUN_00117f18 matrix, FUN_002313f0. FUN_001caec8 hands
    ///   out pool copies at that frame (vt+0x54 owner/+0x50, vt+0x12c spawner +0x60).
    /// - FUN_001ca370: vt+0x12c (slot 36) of every object of a placed template copy with its placement matrix.
    pub fn place_copies(&mut self, w: &World) {
        use crate::engine::world::loads_b4::B4Load;
        use crate::engine::world::objects::{LoadData, Payload};
        let n = w.objects.len().min(self.obj_addr.len());
        let mx = self.alloc(0x40);
        for i in 0..n {
            let Payload::Loaded(LoadData::B4(d)) = &w.objects[clone_root(w, i)].payload else { continue };
            let B4Load::Class6 { in_place: true, bone, xform, .. } = &**d else { continue };
            let a = self.obj_addr[i];
            for (k, v) in ee_sub_117f18(xform).iter().enumerate() {
                self.w32(a + 0x60 + 4 * k as u32, *v);
            }
            let owner = self.r32(a + 0x10);
            if is_queue(0x0022_d6b0) && owner != WORLD {
                if let Some(r) = self.call_top(0x0022_d6b0, &[*bone as i32 as u32, owner, 0]) {
                    self.w32(a + 0x50, r);
                }
            }
            if is_queue(0x0023_13f0) {
                self.call_top(0x0023_13f0, &[a]);
            }
        }
        for t in &w.template_instances {
            for (k, v) in ee_sub_117f18(&t.xform).iter().enumerate() {
                self.w32(mx + 4 * k as u32, *v);
            }
            for &o in t.objects.iter().filter(|&&o| o < n) {
                let a = self.obj_addr[o];
                if self.slot_target(a, 36) != 0 && is_queue(self.slot_target(a, 36)) {
                    self.enter(a, 36, &[mx], &[]);
                }
            }
        }
    }

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
            // CRAWL R102: field 1339 of the safehouse hub C2d80d0 (thunk 0x10f648, value arg type 0x8006 = int list)
            // -> FUN_001d7780(obj, list): FUN_002013d8(kind 6, max 6) into +0x220 count / +0x224 ints, then
            // FUN_001d77d0(obj, 0). EP1 hub 17239 loads (8 10 7 9) (rec_ep1b 060 at load: +0x220 = 4 {8, 10, 7, 9});
            // without it FUN_001d77d0 found no selectable label and unlinked the select SMA (+0x188) as soon as
            // the hub entered state 1, so the seated select scene ignored the pad. Opt-out SLY2_NO_R102_HUBLIST.
            if callee == 0x001d_7780 && std::env::var_os("SLY2_NO_R102_HUBLIST").is_none() {
                let vals: Vec<u32> = items.iter().flat_map(|it| match it {
                    ArrayItem::List(6, v) => v.iter().filter(|b| b.len() >= 4).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]])).collect(),
                    ArrayItem::One(6, b) if b.len() >= 4 => vec![u32::from_le_bytes([b[0], b[1], b[2], b[3]])],
                    _ => Vec::new(),
                }).collect();
                let k = vals.len().min(6);
                self.w32(a + 0x220, k as u32);
                for (i, v) in vals[..k].iter().enumerate() {
                    self.w32(a + 0x224 + 4 * i as u32, *v);
                }
                self.call(0x001d_77d0, &[a, 0], &[]);
                continue;
            }
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
    /// FEEL R26: (offset, len) of the scalar member properties `write_props` writes
    fn prop_ranges(&self, props: &[crate::engine::world::props::Prop], size: u32) -> Vec<(u32, u32)> {
        use crate::engine::world::field_table::field_word;
        use crate::engine::world::props::{field, PropValue};
        let mut v = Vec::new();
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
            v.push((off, b.len() as u32));
        }
        v
    }
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
    /// WIRE-UI: a top-level `call` with the same budget / panic guard as `enter` (static screens such as
    /// FUN_001db870(0x3da150) called from the host frame, not through an object slot).
    /// WIRE-W1: run any host-side queue call `f` under the `enter` budget / panic guard (live_bridge).
    pub fn guarded<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> Option<R> {
        self.ops = 0;
        let dl = QMem::r32(self, 0x002d_d2fc);
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(self)));
        self.depth = 0;
        self.cstack.clear();
        match r {
            Ok(x) => Some(x),
            Err(e) => {
                // BOSS R24: an aborted guest call leaves its FUN_0011a2e8 iteration frames on DAT_002dd2fc (stack
                // frames that are reused later = a cyclic frame chain); unwind them as the returns would have.
                self.ops = 0;
                QMem::w32(self, 0x002d_d2fc, dl);
                if e.downcast_ref::<BudgetExceeded>().is_some() {
                    self.stats.budget_hits += 1;
                } else {
                    self.stats.panics += 1;
                }
                None
            }
        }
    }

    /// ANIM-ALL: guarded FUN_00226550 ASEGA pass (`q_wirew1h11::world_asega_pass` on the world); a budget /
    /// panic stop restores the DAT_002dd2fc iterator chain.
    /// ANIM-ALL: guest poses of the ALOs whose bones carry acts (`q_wirew1h11::alo_pose_pass`), guarded per ALO.
    pub fn pose_pass(&mut self) -> usize {
        let alos = self.alos.clone();
        self.pose_alos(&alos)
    }

    /// The pose step of `pose_pass` for the ALOs `alos` (guarded per ALO).
    pub fn pose_alos(&mut self, alos: &[u32]) -> usize {
        let mut n = 0;
        for &a in alos {
            let (nb, bp) = (QMem::r32(self, a + 0x50) as i32, QMem::r32(self, a + 0x54));
            if nb <= 0 || nb > 512 || bp < HEAP_BASE || bp >= HEAP_END {
                continue;
            }
            if !(0..nb as u32).any(|k| QMem::r32(self, bp + (k << 8) + 0x90) != 0) {
                continue;
            }
            self.ops = 0;
            let sp = QMem::r32(self, super::q2mem::SCRATCH_SP);
            let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| crate::engine::game::q_wirew1h11::alo_pose_pass(self, a)));
            self.depth = 0;
        self.cstack.clear();
            QMem::w32(self, super::q2mem::SCRATCH_SP, sp);
            match r {
                Ok(()) => n += 1,
                Err(_) => self.stats.panics += 1,
            }
        }
        n
    }

    pub fn asega_pass(&mut self) {
        let saved = QMem::r32(self, 0x002d_d2fc);
        self.ops = 0;
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| crate::engine::game::q_wirew1h11::world_asega_pass(self, WORLD)));
        self.depth = 0;
        self.cstack.clear();
        if let Err(e) = r {
            self.stats.asega_aborts += 1;
            if e.downcast_ref::<BudgetExceeded>().is_some() {
                self.stats.budget_hits += 1;
            } else {
                self.stats.panics += 1;
            }
            if std::env::var_os("ANIMALL_DBG").is_some() {
                eprintln!("[anim-all] asega pass aborted (budget {}, ops {})", e.downcast_ref::<BudgetExceeded>().is_some(), self.ops);
            }
            QMem::w32(self, 0x002d_d2fc, saved);
        }
    }

    pub fn call_top(&mut self, addr: u32, a: &[u32]) -> Option<u32> {
        self.ops = 0;
        let dl = QMem::r32(self, 0x002d_d2fc);
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| QMem::call(self, addr, a, &[]).0));
        self.depth = 0;
        self.cstack.clear();
        match r {
            Ok(x) => Some(x),
            Err(e) => {
                self.ops = 0;
                QMem::w32(self, 0x002d_d2fc, dl);
                if e.downcast_ref::<BudgetExceeded>().is_some() {
                    self.stats.budget_hits += 1;
                } else {
                    self.stats.panics += 1;
                }
                None
            }
        }
    }

    pub fn enter(&mut self, obj: u32, slot: u32, a: &[u32], v: &[V4]) -> Option<(u32, V4)> {
        self.ops = 0;
        let before = self.stats.q_calls;
        let dl = QMem::r32(self, 0x002d_d2fc);
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| super::q2mem::vslot(self, obj, slot, a, v)));
        if r.is_err() {
            self.ops = 0;
            QMem::w32(self, 0x002d_d2fc, dl);
        }
        self.depth = 0;
        self.cstack.clear();
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

/// BOSS R24: the loaded object a FUN_0013e510 copy (Payload::Clone, possibly a copy of a copy) was made from.
fn clone_root(w: &World, mut i: usize) -> usize {
    use crate::engine::world::objects::Payload;
    while let (Payload::Clone, Some(c)) = (&w.objects[i].payload, w.objects[i].clone_of) {
        i = c;
    }
    i
}

/// BOSS R24: FUN_00117f18 (FUN_00117ed0 3 rows + FUN_00117eb0 translation at +0x30; w = 0, 0, 0, 1): the
/// 0x40-byte matrix of a packed 48-byte stream xform.
fn ee_sub_117f18(x: &[u8; 48]) -> [u32; 16] {
    let f = |k: usize| u32::from_le_bytes(x[4 * k..4 * k + 4].try_into().unwrap());
    let mut m = [0u32; 16];
    for r in 0..4 {
        for c in 0..3 {
            m[4 * r + c] = f(3 * r + c);
        }
    }
    m[15] = 1.0f32.to_bits();
    m
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
    fn call64(&mut self, addr: u32, a: &[u64]) {
        use crate::engine::game::q_00166008 as g;
        let x = |i: usize| a.get(i).copied().unwrap_or(0);
        match addr {
            0x0016_6bd8 => g::ee_sub_166bd8(self, x(0) as u32, x(1), x(2), x(3)),
            0x0016_6e90 => g::ee_sub_166e90(self, x(0) as u32, x(1), x(2)),
            0x0016_6ef0 => g::ee_sub_166ef0(self, x(0) as u32, x(1), x(2)),
            0x0016_6fe8 => g::ee_sub_166fe8(self, x(0) as u32, x(1), x(2) as i32),
            0x0016_7048 => g::ee_sub_167048(self, x(0) as u32, x(1), x(2) as i32, x(3)),
            _ => {
                let lo: Vec<u32> = a.iter().map(|&v| v as u32).collect();
                self.call(addr, &lo, &[]);
            }
        }
    }
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
        u32::from_le_bytes([self.r8(a), self.r8(a.wrapping_add(1)), self.r8(a.wrapping_add(2)), self.r8(a.wrapping_add(3))])
    }
    fn w32(&mut self, a: u32, v: u32) {
        let o = (a & (PAGE - 1)) as usize;
        if o <= (PAGE - 4) as usize {
            self.tick();
            self.page(a)[o..o + 4].copy_from_slice(&v.to_le_bytes());
            return;
        }
        for (i, b) in v.to_le_bytes().into_iter().enumerate() {
            self.w8(a.wrapping_add(i as u32), b);
        }
    }
    fn call(&mut self, addr: u32, a: &[u32], v: &[V4]) -> (u32, V4) {
        // PAD2: intercept FUN_001619a0 (activity event) — record for host drain
        // WIRE-A: FUN_0011ccc0(pool) fixed-size pool allocation (REPLACED: a fresh 0x100-byte block of
        // the live heap; the pools are not modelled, nothing is returned to them)
        if self.load_pass && std::env::var("ANIMALL_TRACE").is_ok() && (std::env::var("ANIMALL_TRACE").as_deref() == Ok("all") || [0x0022_d7a0u32, 0x0018_6670, 0x0016_03b8, 0x0021_c888, 0x0011_7c80, 0x0016_7320, 0x001f_2b20, 0x0011_7ed0, 0x0013_f810, 0x0013_fb40, 0x001f_35f8].contains(&addr)) {
            if let Some((st, _)) = self.nested_end {
                let c = self.r32(st + 4);
                eprintln!("  trace {addr:x} cursor {c:x} left {}", self.r32(st + 8) as i32);
            }
        }
        if addr == 0x0013_fb40 {
            if let Some((st, end)) = self.nested_end {
                if a.get(1) == Some(&st) {
                    let end = if end == 0 { self.r32(st + 4).wrapping_add(2) } else { end };
                    let left = self.r32(st + 8).wrapping_sub(end.wrapping_sub(self.r32(st + 4)));
                    self.w32(st + 4, end);
                    self.w32(st + 8, left);
                    return (0, [0.0; 4]);
                }
            }
        }
        if addr == 0x0011_ccc0 {
            if let Some(p) = self.pool_free.pop() {
                for i in (0..0x100).step_by(4) {
                    self.w32(p + i, 0);
                }
                return (p, [0.0; 4]);
            }
            return (self.alloc(0x100), [0.0; 4]);
        }
        // WIRE-A: FUN_0011cd98(pool, p) pool free: the block goes back on the replaced pool's free list
        // (the guest pool's used count +0x10 is kept as the original does)
        if addr == 0x0011_cd98 && std::env::var_os("PART_NO_POOL_REUSE").is_none() {
            let (pool, p) = (a.first().copied().unwrap_or(0), a.get(1).copied().unwrap_or(0));
            if p != 0 && p >= HEAP_BASE && p < self.heap && !self.pool_free.contains(&p) {
                let n = self.r32(pool + 0x10);
                self.w32(pool + 0x10, n.wrapping_sub(1));
                self.pool_free.push(p);
            }
            return (0, [0.0; 4]);
        }
        // SAVE2: card routines of the card menu
        if let Some(r) = self.card_native(addr, a) {
            return (r, [0.0; 4]);
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
        // WIRE-UI: FUN_00187e88(pad) menu left/right auto-repeat -> the 1:1 port in input::pad on the guest pad
        // block (+0x1000..+0x10f0 = `PadState`, repr(C)); the hub C2d80d0 cursor (FUN_001d72a8) reads it
        // WIRE-UI: FUN_001f0660(sma, label) SMA goal change -> the host port `anim::sm::Sma::ee_sub_1f0660` on
        // the SMA fields (+0x2c flags, +0x34 cur, +0x38 next, +0x3c goal, +0x44 svt, +0x48 mask) with the SM
        // of +0x24. Transition ASEGs (camera / character moves of the hub) are not played on the queue store:
        // length 0 = the transition completes at once (the FUN_001f0d88 instant path).
        if addr == 0x001f_0660 {
            let sma = a.first().copied().unwrap_or(0);
            let k = self.r32(sma + 0x24);
            if let Some(sm) = self.sms.get(&k).cloned() {
                let mut s = crate::engine::anim::sm::Sma {
                    flags: self.r32(sma + 0x2c),
                    asega: None,
                    cur: self.r32(sma + 0x34) as i32,
                    next: self.r32(sma + 0x38) as i32,
                    goal: self.r32(sma + 0x3c) as i32,
                    trans: None,
                    svt: self.rf(sma + 0x44),
                    mask: self.r32(sma + 0x48),
                    changes: vec![],
                    instant: vec![],
                };
                let mut rng = crate::engine::physics::driven::GameRng::default();
                s.ee_sub_1f0660(&sm, &mut rng, a.get(1).copied().unwrap_or(u32::MAX) as i32, &|_| 0.0);
                self.w32(sma + 0x34, s.cur as u32);
                self.w32(sma + 0x38, s.next as u32);
                self.w32(sma + 0x3c, s.goal as u32);
            }
            return (0, [0.0; 4]);
        }
        if addr == 0x0018_7e88 {
            let base = a.first().copied().unwrap_or(0x002d_fc00) + 0x1000;
            let mut p = read_pad(self, base);
            let clk = crate::engine::input::pad::Clock { t: self.rf(G_T), dt: self.rf(G_DT) };
            let r = crate::engine::input::pad::FUN_00187e88(&mut p, clk);
            write_pad(self, base, &p);
            return (r as u32, [0.0; 4]);
        }
        // WIRE-JOB: the host-owned Sly JT: FUN_00132b10(jt, k) is-a (C2d2e98 class chain 0x35 -> 0x34 -> ...:
        // asked with 0x34 by the gadget menu FUN_001daa90) and FUN_00141fc8(jt, slot, g) equip
        if self.host_jt != 0 && a.first() == Some(&self.host_jt) {
            if addr == 0x0013_2b10 && matches!(a.get(1), Some(0x34) | Some(0x35)) {
                return (1, [0.0; 4]);
            }
            if addr == 0x0014_1fc8 {
                self.recorded_141fc8.push((a.get(1).copied().unwrap_or(3) as usize, a.get(2).copied().unwrap_or(u32::MAX) as i32));
                return (0, [0.0; 4]);
            }
        }
        if let Some(r) = self.save_native(addr, a) {
            return (r, [0.0; 4]);
        }
        // BOSS R25: FUN_001a61e0(o, msg, sched, nargs, argv, out) = FUN_001331f8 (the level-script message,
        // argument words = FUN_00200da0's *argv[i]) + messages 5 / 6 -> FUN_001619a0(0x2deb30, 0x1a / 0x1b, o).
        // REPLACED: the script side runs in the Rust VM, so the message is recorded for the host.
        // CRAWL R103: FUN_001331f8(sma, msg, now, nargs, argv, ..) on a live SMA (vt 0x2d9768; its +0x1c IS
        // FUN_001331f8: msg 2 at FUN_001f01b8, 8 {old, new} at FUN_001f13a8 ...) = the level-script message of the SM's
        // script frame with self = the SMA (e.g. the safehouse hub select SMA -> script 249.1 s2 seats the gang, s8
        // per-character view). REPLACED: the script side runs in the Rust VM, so it is recorded for the host.
        // CRAWL R106: DEFAULT (opt-out SLY2_NO_R106_SMAMSG). R103 suspected it stopped the hub X press -> exit seq; the
        // headless repro (crawl_r98::crawl_r106_probe_x_smamsg: hub select, 60 frames of forwarded messages, X via
        // FUN_001d72a8) starts the exit with and without it (identical hub +0x180 / +0x188 / lock / focus).
        if addr == 0x0013_31f8 && a.first().is_some_and(|&o| o >= HEAP_BASE && self.r32(o + 0xc) == 0x002d_9768) && std::env::var_os("SLY2_NO_R106_SMAMSG").is_none() {
            let (o, msg) = (a[0], a.get(1).copied().unwrap_or(0));
            let (n, argv) = (a.get(3).copied().unwrap_or(0).min(8), a.get(4).copied().unwrap_or(0));
            let words = (0..n).map(|i| if argv == 0 { 0 } else { let p = self.r32(argv + 4 * i); if p == 0 { 0 } else { self.r32(p) } }).collect();
            // every SMA of the load sends msg 2 (FUN_001f01b8) before the host drains: bounded generously
            if self.recorded_sma_msgs.len() >= 8192 {
                self.recorded_sma_msgs.remove(0);
            }
            // CRAWL R110: the SM (+0x24) / owner (+0x28) at send time: FUN_001f0598 frees the SMA (vt +0x14) right after msg 3
            let (sm, ow) = (self.r32(o + 0x24), self.r32(o + 0x28));
            self.recorded_sma_msgs.push((o, msg, words, sm, ow));
            return (0, [0.0; 4]);
        }
        if addr == 0x001a_61e0 {
            let (o, msg) = (a.first().copied().unwrap_or(0), a.get(1).copied().unwrap_or(0));
            let (n, argv) = (a.get(3).copied().unwrap_or(0).min(8), a.get(4).copied().unwrap_or(0));
            let words = (0..n).map(|i| if argv == 0 { 0 } else { let p = self.r32(argv + 4 * i); if p == 0 { 0 } else { self.r32(p) } }).collect();
            // bounded while no host drains (load pass, no player): the oldest go
            if self.recorded_1a61e0.len() >= 256 {
                self.recorded_1a61e0.remove(0);
            }
            self.recorded_1a61e0.push((o, msg, words));
            if msg == 5 || msg == 6 {
                self.record_1619a0(0x002d_eb30, if msg == 5 { 0x1a } else { 0x1b }, o as i32);
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
            self.cstack.push((addr, a.first().copied().unwrap_or(0), a.get(1).copied().unwrap_or(0)));
            let r = f(self, a, v);
            self.cstack.pop();
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

/// SAVE2: card menu requests that need the save-variable table (drained after the frame, in order)
#[derive(Clone, Debug)]
pub enum CardOp {
    /// FUN_001ac420(card, slot): write the FUN_001d84b0 copy of the store to the slot file
    Write(usize, BTreeMap<String, [u32; 2]>, crate::engine::game::save::SaveVecs),
    /// FUN_001ac468(card, slot): write an empty slot
    Clear(usize),
    /// FUN_001ad320 -> FUN_00165e38(slot) / FUN_00166008: the slot becomes the store 0x3d4a60
    Load(usize),
}

/// SAVE2: slot records of the card manager 0x2e65e0: +0x24 + n * 0x5464 (blob 0x5408, then the dir entry)
pub const CARD_SLOTS: u32 = 0x002e_6604;
pub const CARD_SLOT_SIZE: u32 = 0x5464;

impl LiveMem {
    fn slot_of(p: u32) -> Option<usize> {
        let n = p.wrapping_sub(CARD_SLOTS) / CARD_SLOT_SIZE;
        (p >= CARD_SLOTS && n < 3).then_some(n as usize)
    }
    /// SAVE2: give every vector / clq store value without a block (fresh from a card) its live-heap block
    pub fn materialize_vecs(&mut self) {
        for (k, q) in self.save_vec.clone() {
            let Some(v) = self.save.get(&k).copied() else { continue };
            if v[1] == 0 {
                let b = self.alloc(16);
                for (i, w) in q.iter().enumerate() {
                    self.w32(b + 4 * i as u32, *w);
                }
                self.save.insert(k, [v[0], b]);
            }
        }
    }
    /// SAVE2: the card routines the card menu (q12_c) calls, on the save store and the card directory
    pub fn card_native(&mut self, addr: u32, a: &[u32]) -> Option<u32> {
        let arg = |i: usize| a.get(i).copied().unwrap_or(0);
        match addr {
            // FUN_001d84b0(vars, slot): copy of the store 0x3d4a60 into the slot (from_memcard set at write)
            0x001d_84b0 => {
                let n = Self::slot_of(arg(1))?;
                self.card_snaps.insert(n, (self.save.clone(), self.save_vec.clone()));
                Some(0)
            }
            // FUN_001ac420(card, slot): write the slot file
            0x001a_c420 => {
                let n = Self::slot_of(arg(1))?;
                let (s, v) = self.card_snaps.get(&n).cloned().unwrap_or_else(|| (self.save.clone(), self.save_vec.clone()));
                self.card_ops.push(CardOp::Write(n, s, v));
                Some(0)
            }
            // FUN_001ac468(card, slot): empty slot
            0x001a_c468 => {
                let n = Self::slot_of(arg(1))?;
                self.card_ops.push(CardOp::Clear(n));
                Some(0)
            }
            // FUN_001ac4c8(card, slot): clear the record, read the file into it (+0x544c size, +0x5450 attrs)
            0x001a_c4c8 => {
                let p = arg(1);
                let n = Self::slot_of(p)?;
                for i in (0..CARD_SLOT_SIZE).step_by(4) {
                    self.w32(p + i, 0);
                }
                let card = self.card.clone().unwrap_or_else(crate::engine::game::save::MemoryCard::user);
                Some(match card.ee_sub_1ac4c8(n) {
                    Ok(d) => {
                        for (i, b) in d.to_bytes().into_iter().enumerate() {
                            self.w8(p + i as u32, b);
                        }
                        self.w32(p + 0x544c, 0x5408);
                        self.w32(p + 0x5450, 4);
                        0
                    }
                    Err(crate::engine::game::save::McError::NoFile) => 0xe,
                    Err(crate::engine::game::save::McError::Attr) => 0x14,
                    Err(crate::engine::game::save::McError::Size(_)) => 0x1c,
                    Err(crate::engine::game::save::McError::Io(_)) => 0x27,
                })
            }
            // FUN_001ad320(ui): load the selected slot (ui+8) -> FUN_00165e38(slot); ui+0x1c |= 2
            0x001a_d320 => {
                let ui = arg(0);
                let n = self.r32(ui + 8) as usize;
                if ui == 0 || n >= 3 {
                    return None;
                }
                self.card_ops.push(CardOp::Load(n));
                let f = self.r32(ui + 0x1c);
                self.w32(ui + 0x1c, f | 2);
                Some(0)
            }
            // FUN_00166008 with DAT_002deb4c = a slot record: the slot becomes the store
            0x0016_6008 => {
                let n = Self::slot_of(self.r32(0x002d_eb4c))?;
                self.w32(0x002d_eb4c, 0);
                self.card_ops.push(CardOp::Load(n));
                Some(0)
            }
            _ => None,
        }
    }
}

/// Bevy wiring: a shadow image of the active world, queue slots run each frame after the class runtime.
/// WIRE-A: load-time setter methods applied by `LiveMem::method_props`: FUN_001d7748 (C2d80d0 hub
/// entry {state, button, target}, splice thunk 0x10f590 / field 1338).
const LOAD_METHODS: &[u32] = &[0x001d_7748];

/// FEEL R89: a per-LiveQ (per App) id; `tick_live_q` publishes it to its thread while the image runs so the
/// process-global FUN_00141968 launch queue (hard1::LAUNCHES_141968) is consumed by the owning App's player only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QId(pub u64);
impl Default for QId {
    fn default() -> Self {
        static N: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        QId(N.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
    }
}
thread_local! {
    /// FEEL R89: id of the LiveQ whose image is running on this thread (0 = none)
    pub static CUR_QID: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

#[derive(bevy_ecs::prelude::Resource, Default)]
pub struct LiveQ {
    /// FEEL R89: see QId
    pub qid: QId,
    /// ANIM-ALL: FUN_0012f9b8 requests replayed into the image (guest_anim_starts)
    pub anim_starts: u64,
    /// ANIM-ALL R20: guest starts (guest_anim_starts / guest_slot_play) that panicked or overran the budget
    pub anim_start_fails: u64,
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
    /// WIRE-JOB2: host level-table state for the level start FUN_0020e3c0: DAT_003e1110 (current row, -1 none)
    /// and DAT_003e1c44 (row -> episode); None = not known (harness builds): FUN_0020e3c0 is not run
    pub level_rows: Option<(i32, Vec<u32>)>,
    /// SAVE1: card slot read into the save store before the first build (`--load-slot`), card slot the store
    /// is written to whenever it changes (`--save-slot`), card location (default: user config dir `saves/`)
    pub load_slot: Option<usize>,
    pub save_slot: Option<usize>,
    pub card: Option<crate::engine::game::save::MemoryCard>,
    /// SAVE1: loaded store waiting for the next `build`; last store written to the card
    pub pending_store: Option<BTreeMap<String, [u32; 2]>>,
    saved_store: Option<BTreeMap<String, [u32; 2]>>,
    /// SAVE2: vector side table of `pending_store`
    /// SAVE2: slot `saved_store` was written to (a loaded store is still written once to another slot)
    saved_slot: Option<usize>,
    pub pending_vec: Option<crate::engine::game::save::SaveVecs>,
}

impl LiveQ {
    fn card(&self) -> crate::engine::game::save::MemoryCard {
        self.card.clone().unwrap_or_else(crate::engine::game::save::MemoryCard::user)
    }
    /// SAVE1 load: FUN_001ac4c8 (card read, 0x5408 bytes) -> FUN_001d7d30 layout check (mismatch = the
    /// FUN_001d7c80 clear: nothing loaded) -> the blob becomes the save store 0x3d4a60 (applied at the next
    /// `build`: episode, coins, gadgets, episodeBest, keys/bottles and progress are all store values).
    pub fn load_card(&mut self, slot: usize, vars: &crate::engine::game::save::SaveVars) -> bool {
        match self.card().ee_sub_1ac4c8(slot) {
            Ok(d) if vars.ee_sub_1d7d30(&d) => {
                let (mut s, mut v) = vars.blob_to_live_v(&d);
                // SAVE2: vector values from the slot's side file
                let (vs, vv) = self.card().read_vecs(slot);
                s.extend(vs);
                v.extend(vv);
                self.pending_vec = Some(v.clone());
                log::info!("[save1] loaded card slot {slot}: {} values", s.len());
                self.saved_store = Some(s.clone());
                if let Some(m) = self.mem.as_mut() {
                    m.save = s.clone();
                    m.save_vec = v;
                    m.materialize_vecs();
                    self.saved_store = Some(m.save.clone());
                }
                self.pending_store = Some(s);
                true
            }
            Ok(_) => {
                log::warn!("[save1] card slot {slot}: layout mismatch, not loaded");
                false
            }
            Err(e) => {
                log::warn!("[save1] card slot {slot}: {e:?}");
                false
            }
        }
    }
    /// SAVE1 save: store -> blob (FUN_00258200/FUN_001d80c8 per value) -> FUN_001d84b0 slot copy
    /// (from_memcard = 1) -> FUN_001ac420 card write. Skipped while the store is unchanged.
    pub fn save_card(&mut self, slot: usize, vars: &crate::engine::game::save::SaveVars) -> bool {
        let Some(m) = self.mem.as_ref() else { return false };
        if self.saved_slot == Some(slot) && self.saved_store.as_ref() == Some(&m.save) {
            return false;
        }
        let d = vars.ee_str_from_memcard_1d84b0(&vars.live_to_blob_v(&m.save, &m.save_vec));
        let s = m.save.clone();
        let card = self.card();
        if let Err(e) = card.write_vecs(slot, &m.save, &m.save_vec) {
            log::warn!("[save2] card slot {slot} vectors: {e}");
        }
        match card.ee_sub_1ac420(slot, &d) {
            Ok(()) => {
                self.saved_store = Some(s);
                self.saved_slot = Some(slot);
                true
            }
            Err(e) => {
                log::warn!("[save1] card slot {slot} write: {e}");
                self.saved_store = Some(s);
                self.saved_slot = Some(slot);
                false
            }
        }
    }
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
        // ANIM-ALL R93: FUN_0020e890 runs the po selection only when FUN_001c4840(world) (+0x3efc focus) and
        // FUN_001c4a48(world) (+0x3f00, else +0x3efc) are 0 after FUN_00228750(spawn): the spawn's message 5 may
        // already have given the focus away (EP2 spawn 393 msg 5 = (set-o 17239 806 1): hub select state, seated
        // gang); the selection's Sly slot 78(1) took it back -> Sly standing / movable at the table. Returns the
        // focus object then (no slot calls). Opt-out SLY2_R93_NO_FOCUSSKIP.
        if std::env::var_os("SLY2_R93_NO_FOCUSSKIP").is_none() {
            if let Some(m) = self.mem.as_ref() {
                let f = m.peek32(WORLD + 0x3efc);
                let f = if f != 0 { f } else { m.peek32(WORLD + 0x3f00) };
                if f != 0 {
                    return m.obj_addr.iter().position(|&x| x == f);
                }
            }
        }
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
    /// SAVE2: run the card menu's deferred card requests (FUN_001ac420 / FUN_001ac468 / slot load)
    pub fn drain_card_ops(&mut self, vars: &crate::engine::game::save::SaveVars) {
        let Some(m) = self.mem.as_mut() else { return };
        let ops = std::mem::take(&mut m.card_ops);
        let card = self.card();
        for op in ops {
            match op {
                CardOp::Write(n, s, v) => {
                    let d = vars.ee_str_from_memcard_1d84b0(&vars.live_to_blob_v(&s, &v));
                    if let Err(e) = card.write_vecs(n, &s, &v) {
                        log::warn!("[save2] card menu slot {n} vectors: {e}");
                    }
                    match card.ee_sub_1ac420(n, &d) {
                        Ok(()) => log::info!("[save2] card menu saved slot {n}: {} values, {} vectors", s.len(), v.len()),
                        Err(e) => log::warn!("[save2] card menu slot {n} write: {e}"),
                    }
                }
                CardOp::Clear(n) => {
                    let _ = std::fs::remove_file(card.vec_path(n));
                    if let Err(e) = card.ee_sub_1ac468(n, vars, &mut crate::engine::game::save::GameData::default()) {
                        log::warn!("[save2] card menu slot {n} clear: {e}");
                    }
                }
                CardOp::Load(n) => {
                    self.load_card(n, vars);
                }
            }
        }
    }
    pub fn build(&mut self, w: &World) {
        let mut m = LiveMem::build(w);
        m.world = self.world.clone();
        m.card = Some(self.card());
        // WIRE-JOB: the save store 0x3d4a60 is a global of the executable, not of the world: it survives the
        // level load (FUN_0020e890 frees / rebuilds only the world)
        if let Some(old) = self.mem.take() {
            m.save = old.save;
            m.save_vec = old.save_vec;
            m.card_snaps = old.card_snaps;
        }
        if let Some(s) = self.pending_store.take() {
            m.save = s;
            if let Some(v) = self.pending_vec.take() {
                m.save_vec = v;
            }
        }
        // SAVE2: vector store blocks live in the image heap: re-allocate them in the new image
        for v in m.save.values_mut() {
            if v[0] == crate::engine::game::save::TAG_VECTOR || v[0] == crate::engine::game::save::TAG_CLQ {
                v[1] = 0;
            }
        }
        m.materialize_vecs();
        // POOL1 R90: DAT_003e1110 (current level-table row) = -1 'none' when the host has no level table (harness
        // builds; FUN_0020f318 never ran, DAT_003e1c4c = 0): with the static 0 the load pass's FUN_001c7b18 / FUN_001c8090
        // read row 0 of a NULL table and FUN_001ae5a0 copied 0x7f800000 bytes, wiping the image (heist_train_hack: every
        // object header 0, vt slot calls to 0x0). SLY2_R90_NO_E1110=1 = the old static 0.
        if std::env::var_os("SLY2_R90_NO_E1110").is_none() && self.level_rows.is_none() { m.w32(0x003e_1110, u32::MAX); }
        // WIRE-Q: FUN_0020e890 runs the world init FUN_0020e180(&world) before the level's objects are created
        // (render list +0x3f20 / +0x3f0c pools that the slot-22 adds and the render-world frame use)
        if is_queue(0x0020_e180) {
            m.call_top(0x0020_e180, &[WORLD]);
        }
        // ANIM-ALL: FUN_0013da18's object loads ran FUN_0012f2a8 (class 109/110 load) on every ASEG: fill the
        // guest ASEGs (channels / curves / events / length) so FUN_0012f9b8 + the FUN_00130918 pass can play them
        if !w.level_data.0.is_empty() {
            let data = w.level_data.0.clone();
            let (ok, bad) = m.load_asegs(w, &data);
            log::info!("[anim-all] FUN_0012f2a8 guest ASEG loads: {ok} ok, {bad} failed");
            let (ok, bad) = m.load_alos(w, &data);
            log::info!("[anim-all] ALO class loads (FUN_001f31e0) in the image: {ok} ok, {bad} failed");
        }
        // FEEL R24/R26: FUN_0013da18 creates each object (slot-6 init, e.g. the class-15 track's +0x164 = 7) BEFORE its
        // load reads the properties: the inits below run over the prop-filled image, so a scalar member property
        // that an init overwrote gets its load-time image bytes back (R26: the load-time image, not the raw
        // descriptor bytes of the clone source: re-writing those clobbered resolved / per-clone members, e.g. the
        // hub labels and the safehouse select). A/B: SLY2_FEEL_NOREPROPS=1
        let reprops = std::env::var_os("SLY2_FEEL_NOREPROPS").is_none();
        let mut snap: Vec<(u32, Vec<u8>)> = Vec::new();
        if reprops {
            for (i, o) in w.objects.iter().enumerate() {
                if let Some(props) = crate::engine::game::fields::object_props(&w.objects[o.clone_of.unwrap_or(i)]) {
                    let a = m.obj_addr[i];
                    for (off, n) in m.prop_ranges(props, size_of_class(o.class)) {
                        snap.push((a + off, (0..n).map(|k| m.r8(a + off + k)).collect()));
                    }
                }
            }
        }
        // slot 6 init(obj, parent) in creation order, then slot 14 post-load (FUN_0013da18 order)
        self.inited = 0;
        // FIXHUB: the base init FUN_001a5270 recomputes +0x28 bit 60 (in-world) from the parent's, but here it
        // runs over the post-load image (the loader's later world add / removal already applied, and WORLD's own
        // flags are not in the image): keep the load-time in-world bit (out_of_world_at_load) after the inits
        let in_world: Vec<bool> = (0..m.obj_addr.len()).map(|i| m.in_world(i)).collect();
        for i in 0..m.obj_addr.len() {
            let o = m.obj_addr[i];
            let f = m.slot_target(o, SLOT_INIT);
            if f != 0 && is_queue(f) {
                let parent = m.r32(o + 0x10);
                // WIRE-Q2: the base init FUN_001a5270 links the object into its parent's list itself
                // (FUN_0011a280): take it out of the prebuilt list first
                if is_queue(0x0011_a2e8) {
                    let p = if parent != 0 { parent } else { WORLD };
                    m.call_top(0x0011_a2e8, &[p + 0x64, o]);
                }
                m.enter(o, SLOT_INIT, &[parent], &[]);
                self.inited += 1;
            }
        }
        for (i, &b) in in_world.iter().enumerate() {
            let a = m.obj_addr[i];
            let f = (m.r64(a + 0x28) & !(1u64 << 60)) | (b as u64) << 60;
            m.w32(a + 0x2c, (f >> 32) as u32);
        }
        for (a, b) in &snap {
            for (k, &x) in b.iter().enumerate() {
                if m.r8(a + k as u32) != x {
                    m.w8(a + k as u32, x);
                }
            }
        }
        // BOSS R24: the class-6 / template-copy placement of the loads (after the slot-6 creation inits, which
        // reset the class-6 +0x60 matrix; the original creates each object before its load reads it)
        if !w.level_data.0.is_empty() {
            m.place_copies(w);
            // BOSS R24: FUN_0020e740's (per-level init) FUN_001569b8(world): the seven stimulus kinds at
            // world +0x4020..+0x4038 (class 0x75), read by FUN_00156b98 / FUN_00156698 / FUN_00156898
            // and FUN_0020d9d0's (world ctor, not run on the live image) FUN_0011a1f8(world+0x3ffc, 0x58) for
            // STIM_LIST: the record link offset (unset, FUN_0011a2e8 unlinked through +0 = the kind ref)
            if QMem::r32(&mut m, WORLD + 0x3ffc + 8) == 0 {
                m.call_top(0x0011_a1f8, &[WORLD + 0x3ffc, 0x58]);
            }
            if QMem::r32(&mut m, WORLD + 0x4020) == 0 {
                m.call_top(0x0015_69b8, &[WORLD]);
            }
        }
        // ANIM-ALL R16: postload runs once per level and is as heavy as the load (the JT's FUN_00188608 builds
        // its curves through FUN_001300b8), so it gets the load budget
        m.load_pass = true;
        self.post_loaded = m.run_slot(SLOT_POSTLOAD, &[], &[], |_| true);
        m.load_pass = false;
        // WIRE-JOB2: FUN_001ca500(world): per-entry copy records +0x64 / +0x68 (spawner FUN_001cade0 scores)
        crate::engine::classes::q::q14_j::ee_sub_1ca500(&mut m, WORLD);
        // WIRE-JOB2: level start FUN_0020e3c0(&world) (flags/lang, mission, episode, episodeBest into the
        // save store 0x3d4a60), with the host's DAT_003e1110 / DAT_003e1c44
        if let Some((row, eps)) = self.level_rows.clone() {
            m.w32(0x003e_1110, row as u32);
            let t = m.alloc(4 * eps.len().max(1) as u32);
            for (i, &e) in eps.iter().enumerate() {
                m.w32(t + 4 * i as u32, e);
            }
            m.w32(0x003e_1c44, t);
            crate::engine::game::q_wirejob2::ee_sub_20e3c0(&mut m, WORLD);
        }
        self.mem = Some(m);
        self.frames = 0;
        self.t = 0.0;
    }

    /// ANIM-ALL: replay FUN_0012f9b8(a, rate, anim, obj, flags) requests of the class code (handles = object
    /// index + 1) in the image: the original call links the new ASEGA into world+0x3d88, which the FUN_00226550
    /// pass of `frame` then advances. Returns the number started.
    pub fn guest_anim_starts(&mut self, reqs: &[(f32, f32, u32, u32, u32)]) -> usize {
        let Some(m) = self.mem.as_mut() else { return 0 };
        let mut n = 0;
        for &(a, rate, anim, obj, flags) in reqs {
            let addr = |h: u32| (h as usize).checked_sub(1).and_then(|i| m.obj_addr.get(i).copied());
            // ANIM-ALL R17: obj handle 0 = no target (param_4 0: bind_channels searches from the ASEG itself)
            let (Some(asd), Some(o)) = (addr(anim), if obj == 0 { Some(0) } else { addr(obj) }) else { continue };
            m.ops = 0;
            let sp = QMem::r32(m, super::q2mem::SCRATCH_SP);
            // as FUN_00130488 / FUN_001305b8: FUN_0012f9b8 into an out slot, then AddRef (vt+0xc, slot 0) the ASEGA;
            // without it the start's own AddRef/Release pair drops it to 0 and retracts its acts at once
            let out = super::q2mem::stack_push(m, 0x10);
            QMem::w32(m, out, 0);
            // ANIM-ALL R19: no-target starts run within the normal per-call budget (their R17 overrun was the
            // FUN_001f80d0 ABI bug: object read from a1 = 0 -> endless walk of address 0's child list)
            let dbg_pre = std::env::var_os("ANIMALL_DBG").map(|_| m.stats.per_fn.clone());
            let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                QMem::call(m, 0x0012_f9b8, &[asd, o, flags, out], &[[a; 4], [rate; 4]]);
                let h = QMem::r32(m, out);
                if h != 0 {
                    super::q2mem::vslot(m, h, 0, &[], &[]);
                }
            }));
            m.depth = 0;
            let cst = std::mem::take(&mut m.cstack);
            QMem::w32(m, super::q2mem::SCRATCH_SP, sp);
            match r {
                Ok(_) => n += 1,
                Err(_) => {
                    m.stats.panics += 1;
                    self.anim_start_fails += 1;
                    if std::env::var_os("ANIMALL_DBG").is_some() {
                        let mut d: Vec<(u64, u32)> = m.stats.per_fn.iter().map(|(&k, &v)| (v - dbg_pre.as_ref().and_then(|p| p.get(&k)).copied().unwrap_or(0), k)).filter(|x| x.0 > 0).collect();
                        d.sort_unstable_by(|a, b| b.cmp(a));
                        eprintln!("[anim-all] guest start aseg {anim} obj {obj} flags {flags} failed (ops {}) top {:x?} stack {:x?}", m.ops, &d[..d.len().min(10)], cst);
                    }
                }
            }
        }
        self.anim_starts += n as u64;
        n
    }

    /// ANIM-ALL R15: FUN_001305b8(t, rate, aseg, obj, flags, slot) on the guest ASEGA ref word of `key` (a JT
    /// ASEGA field): retracts + releases the ASEGA held there (FUN_00130658), then starts the new one through
    /// FUN_0012f9b8 and AddRefs it. `req` None = FUN_00130658 only (the slot's acts leave the bones).
    /// `req` = (t, rate, anim handle, obj handle, flags), handles = object index + 1.
    pub fn guest_slot_play(&mut self, key: u32, req: Option<(f32, f32, u32, u32, u32)>) -> bool {
        let Some(m) = self.mem.as_mut() else { return false };
        let slot = match m.anim_slots.get(&key) {
            Some(&s) => s,
            None => {
                let s = m.alloc(0x10);
                m.anim_slots.insert(key, s);
                s
            }
        };
        let addr = |m: &LiveMem, h: u32| (h as usize).checked_sub(1).and_then(|i| m.obj_addr.get(i).copied()).filter(|&a| a != 0);
        let args = match req {
            Some((t, rate, anim, obj, flags)) => {
                let (Some(asd), Some(o)) = (addr(m, anim), addr(m, obj)) else { return false };
                Some((asd, o, flags, t, rate))
            }
            None => None,
        };
        m.ops = 0;
        let sp = QMem::r32(m, super::q2mem::SCRATCH_SP);
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match args {
            Some((asd, o, flags, t, rate)) => QMem::call(m, 0x0013_05b8, &[asd, o, flags, slot], &[[t; 4], [rate; 4]]),
            None => QMem::call(m, 0x0013_0658, &[slot], &[]),
        }));
        m.depth = 0;
        m.cstack.clear();
        QMem::w32(m, super::q2mem::SCRATCH_SP, sp);
        match r {
            Ok(_) => {
                self.anim_starts += args.is_some() as u64;
                true
            }
            Err(_) => {
                m.stats.panics += 1;
                self.anim_start_fails += 1;
                false
            }
        }
    }

    /// ANIM-ALL R20: the rest of the original frame for ASEGAs the JT update started. FUN_00226550 runs the object
    /// updates (the JT's FUN_001305b8), then FUN_00130918 over world+0x3d88, then the render poses the bones; the
    /// port's JT starts (`guest_slot_play` from drive_sly) come after `frame`, so the started slot ASEGAs get
    /// this frame's FUN_00130918 here and the ALOs `objs` (object indices) are posed. Without it the frame of a
    /// start drew the bones' load locals (bind pose) / the last anim.
    pub fn guest_slot_settle(&mut self, keys: &[u32], objs: &[usize]) {
        let Some(m) = self.mem.as_mut() else { return };
        for k in keys {
            let Some(&s) = m.anim_slots.get(k) else { continue };
            let h = QMem::r32(m, s);
            if h != 0 && m.call_top(0x0013_0918, &[h]).is_none() {
                self.anim_start_fails += 1;
            }
        }
        if !m.guest_pose {
            return;
        }
        let alos: Vec<u32> = objs.iter().filter_map(|&o| m.obj_addr.get(o).copied()).filter(|a| *a != 0 && m.alos.contains(a)).collect();
        m.pose_alos(&alos);
    }

    /// One frame: clock + slot 15 (FUN_00226550 ObjectUpdate) on in-world objects with a queue-port target.
    /// WP-WIRE3: pre-frame hand -> guest copy, then post-frame guest -> `queue_fields` snapshot.
    pub fn frame(&mut self, dt: f32) {
        let Some(m) = self.mem.as_mut() else { return };
        self.t += dt;
        m.set_clock(self.t, dt, dt);
        if let Some(w) = self.world.as_ref() {
            let _p = crate::engine::classes::ProfGuard::new("lq_copy_hand");
            copy_hand_to_guest(m, &m.obj_addr.clone(), w);
        }
        let mut flags = Vec::with_capacity(m.obj_addr.len());
        for i in 0..m.obj_addr.len() {
            flags.push(m.in_world(i));
        }
        let _p = crate::engine::classes::ProfGuard::new("lq_update_slot");
        self.last_update = m.run_slot(SLOT_UPDATE, &[], &[], |i| flags[i]);
        drop(_p);
        // CRAWL R27: the world update's FUN_00149c80(world) drain of the guest deferred member calls
        // (w+0x3ca8, queued by FUN_00149a50 in the slot code): calls each node and frees it to w+0x3c90
        if std::env::var_os("LIVE_NO_DEFER").is_none() {
            m.call_top(0x0014_9c80, &[WORLD]);
        }
        // ANIM-ALL: FUN_00226550 then advances every live ASEGA of the level (world+0x3d88 DList ->
        // FUN_00130918), whichever actor started it (dt <= 0: FUN_002267c0's 0x100-flag pass, not run here)
        if dt > 0.0 {
            if std::env::var("ANIMALL_NO_ASEGA").is_err() {
                let _p = crate::engine::classes::ProfGuard::new("lq_asega");
                m.asega_pass();
            }
            // BOSS R24: FUN_00226550's next DList walk: FUN_00156728 over world+0x3ffc (= STIM_LIST 0x3e148c),
            // expiring the stimulus records whose kind duration ran out (FUN_00156780 -> FUN_0011cd98 pool free)
            m.guarded(|m| {
                let sp = super::q2mem::stack_push(m, 0x10);
                let saved = QMem::r32(m, 0x002d_d2fc);
                crate::engine::game::q_wirew1h11::dlist_walk(m, sp, WORLD + 0x3ffc, 0x0015_6728);
                QMem::w32(m, 0x002d_d2fc, saved);
                super::q2mem::stack_pop(m, 0x10);
            });
            // ANIM-ALL: guest pose step (FUN_00131cd8 / FUN_0022de88 / FUN_0022faa0) only with SLY2_GUEST_POSE=1:
            // the native player / guard / ledge code still reads the unposed guest bone xfs (wired_sly double
            // jump, wired_guards patrol, ledge grab regress with it on)
            if m.guest_pose {
                m.pose_pass();
            }
        }
        if let Some(w) = self.world.as_ref() {
            let obj = m.obj_addr.clone();
            let _p = crate::engine::classes::ProfGuard::new("lq_snapshot");
            snapshot_queue(m, &obj, w, &mut self.queue_fields);
        }
        // WIRE-Q: the rest of the FUN_001a7518 frame (game/main_loop.rs) that is queue code: render world
        // FUN_001cda80(&world) (C2dc018 slot 7: light rebuild, render-list slot 18 per entry, FUN_001dac58) and
        // UI draw FUN_001d9820(0x3d9e70). Guest-side only (Bevy draws natively); after the snapshot so the
        // hand-owned fields above are unaffected.
        for (f, a) in [(0x001c_da80u32, WORLD), (0x001d_9820, 0x003d_9e70)] {
            if is_queue(f) {
                m.call_top(f, &[a]);
            }
        }
        self.frames += 1;
    }

    /// PAD1: call the static UI screen updates from FUN_001d9648 (UiUpdate main-loop step).
    /// Specifically FUN_001db870(0x3da150) — the planning-table / ThiefNet menu.
    pub fn ui_update(&mut self) {
        let Some(m) = self.mem.as_mut() else { return };
        // FUN_001d9648 calls FUN_001db870(0x3da150) among others.
        // 0x3da150 is the C2d84e8 singleton (ThiefNet / planning-table menu).
        // WIRE-UI: guarded (budget / panic) like the object slots
        m.call_top(0x001d_b870, &[0x3d_a150]);
        // WIRE-JOB: then FUN_001daa90(0x3da0f0) (gadget menu: L1 / L2 / R2 equip the selected gadget)
        m.call_top(0x001d_aa90, &[0x3d_a0f0]);
    }
}

pub fn tick_live_q(level: bevy_ecs::prelude::Res<crate::engine::classes::plugin::ClassesLevel>, mut q: bevy_ecs::prelude::ResMut<LiveQ>, #[cfg(feature = "game")] opt: Option<bevy_ecs::prelude::Res<crate::engine::world::WorldOptions>>, #[cfg(feature = "game")] lg: Option<bevy_ecs::prelude::Res<crate::engine::game::app::LevelGame>>, classes: Option<bevy_ecs::prelude::ResMut<crate::engine::classes::plugin::Classes>>, clock: Option<bevy_ecs::prelude::Res<crate::engine::player::plugin::PlayerClock>>) {
    let Some(w) = &level.0 else { return };
    // FEEL R89: tag the image's FUN_00141968 launches with this App's LiveQ (reset when the system returns)
    struct QidGuard;
    impl Drop for QidGuard {
        fn drop(&mut self) {
            CUR_QID.with(|c| c.set(0));
        }
    }
    CUR_QID.with(|c| c.set(q.qid.0));
    let _qid_guard = QidGuard;
    let id = std::sync::Arc::as_ptr(w) as usize;
    if q.built_for != id {
        // WIRE-JOB2: DAT_003e1110 / DAT_003e1c44 for the level start FUN_0020e3c0
        #[cfg(feature = "game")]
        if let (Some(o), Some(t)) = (opt.as_ref(), lg.as_ref().and_then(|g| g.table.as_ref())) {
            q.level_rows = Some((o.0.mission as i32, t.entries.iter().map(|e| e.idx as u32).collect()));
        }
        // WIRE-UI: world handle before build (LiveMem::world for the FUN_0015e938 searches at init / post-load),
        // as the engine_ep1a harness does (set_world then build)
        q.world = Some(w.clone());
        #[cfg(feature = "game")]
        if let (Some(slot), Some(vars)) = (q.load_slot.take(), lg.as_ref().and_then(|g| g.vars.as_ref())) {
            q.load_card(slot, vars);
        }
        q.build(w);
        q.built_for = id;
        if let Some(m) = &q.mem {
            log::info!("[wire1] live queue image: {} objects, init {} / post-load {} queue slots; {}", m.obj_addr.len(), q.inited, q.post_loaded, m.stats.summary());
        }
    }
    // ANIM-ALL: the class code's FUN_0012f9b8 requests (anim / sound records of the guard / NPC state changes)
    if let Some(mut c) = classes {
        let reqs = std::mem::take(&mut c.host.k25.guest_reqs);
        q.guest_anim_starts(&reqs);
    }
    q.guest_anim_starts(&crate::engine::classes::k25::AnimRt::take_npc_reqs(crate::engine::game::anim_play::world_key(w)));
    q.guest_anim_starts(&crate::engine::game::anim_play::take_guest_starts(crate::engine::game::anim_play::world_key(w)));
    let _p = crate::engine::classes::ProfGuard::new("liveq_frame");
    // ANIM-ALL R16: DAT_002dded4 is g_clock's frame time (FUN_0014d0b8, real time clamped to [1/60, 1/30]), the
    // same clock the native JT movement / anim step uses (PlayerClock); a fixed 1/60 ran the guest anims at half
    // speed against the native ones when frames take 1/30
    q.frame(clock.as_deref().map_or(1.0 / 60.0, |c| c.dt));
    drop(_p);
    #[cfg(feature = "game")]
    if let Some(vars) = lg.as_ref().and_then(|g| g.vars.as_ref()) {
        q.drain_card_ops(vars);
    }
    #[cfg(feature = "game")]
    if let (Some(slot), Some(vars)) = (q.save_slot, lg.as_ref().and_then(|g| g.vars.as_ref())) {
        q.save_card(slot, vars);
    }
    if q.frames % 600 == 0 {
        if let Some(m) = &q.mem {
            log::info!("[wire1] frame {}: {} objects updated by queue code; {}", q.frames, q.last_update, m.stats.summary());
        }
    }
}

/// BOSS R25: the host-owned JT's transform into its live-image root part (jt +0x60 -> local xf +0x00..+0x3f,
/// world xf +0x40..+0x7f; the JT is a root object, so both are its world rows + position; rec11 238:
/// part +0x70 = Sly's position). The original's JT update writes them itself; without them anything parented
/// to the JT (the worn TNT barrel, FUN_00183b80 vt+0x54(jt, bone, 3)) reads the slot-30 frame as garbage.
pub fn jt_xf_sync(m: &mut LiveMem, jt: u32, rot: [[f32; 3]; 3], pos: [f32; 3]) {
    let x = m.r32(jt + 0x60);
    if x == 0 {
        return;
    }
    for base in [0u32, 0x40] {
        for (i, r) in rot.iter().enumerate() {
            m.wv(x + base + 0x10 * i as u32, [r[0], r[1], r[2], 0.0]);
        }
        m.wv(x + base + 0x30, [pos[0], pos[1], pos[2], 1.0]);
    }
}

/// WIRE-JOB: host-owned Sly JT <-> live store. The JT init FUN_00188488 writes DAT_002e1e40 = jt; the po
/// selection (vt +0x44 of the JT) sets the world focus +0x3efc (FUN_001c4940) while Sly is controlled; the JT
/// update's SELECT shortcut FUN_001db490(0x3da0f0) runs on the store; the gadget menu's FUN_00141fc8 equips on
/// the host JT; the JT's acquired test FUN_001daea8 reads the save ("powerUps" / "mppukfAcquired" / "%d").
pub fn live_jt_bridge(mut q: bevy_ecs::prelude::ResMut<LiveQ>, mut players: bevy_ecs::prelude::Query<&mut crate::engine::player::plugin::Player>, po: Option<bevy_ecs::prelude::Res<crate::engine::player::p20::PoCurrent>>) {
    let Some(w) = q.world.clone() else { return };
    let Some(m) = q.mem.as_mut() else { return };
    let Some(mut pl) = players.iter_mut().next() else { return };
    let sly = w.objects.iter().position(|o| o.class == crate::engine::player::p20::CLASS_SLY).and_then(|i| m.obj_addr.get(i).copied());
    let Some(jt) = sly else { return };
    m.host_jt = jt;
    m.w32(0x002e_1e40, jt);
    // CRAWL R103: not while the safehouse hub holds it (only the JT's own +0x27c(1) takes it back)
    if po.is_none_or(|p| p.obj.is_none()) && !m.hub_has_focus() {
        m.w32(WORLD + 0x3efc, jt);
    }
    let jt_ = &mut pl.jt;
    let mut opens = 0;
    jt_.calls.retain(|c| if c.0 == 0x001d_b490 { opens += 1; false } else { true });
    for _ in 0..opens {
        super::q12_d::ee_sub_1db490(m, 0x003d_a0f0);
    }
    for (slot, g) in std::mem::take(&mut m.recorded_141fc8) {
        jt_.FUN_00141fc8(slot, g, &mut ());
    }
    let mut acq = 0u64;
    for g in 0..64 {
        if m.save.get(&format!("powerUps/mppukfAcquired/{g}")).is_some_and(|v| v[0] != u32::MAX && v[1] != 0) {
            acq |= 1 << g;
        }
    }
    jt_.gad.acquired |= acq;
    // WIRE-JOB2: jt+0x2010 = FUN_001885b8 bind row 158 of DAT_002be990 (flags 0x300, oid 0x21d) via
    // FUN_001a64e8 -> FUN_0015e938(world, 1, oid, jt); can-use 0x17 reads FUN_001cb2c0(jt+0x2010)
    let mut t = m.r32(jt + 0x2010);
    if t == 0 {
        t = m.call(0x0015_e938, &[WORLD, 1, 0x21d, jt], &[]).0;
        m.w32(jt + 0x2010, t);
    }
    jt_.gad.f_2010 = t;
    jt_.gad.cb2c0_2010 = t != 0 && crate::engine::game::q_001cb2c0::ee_sub_1cb2c0(m, t);
}

/// WIRE-W1 (WIRE-BM2 request): anim event owner messages (FUN_00131290 case 0xb: `(*(vt+0x2c))(owner, 9, args)`)
/// delivered to the live queue object's slot 4 (args {asega, t, value} on the emulated stack). Bentley /
/// Murray (slot 4 = FUN_00135ad8 / FUN_001b68b8) get it from the store in `frame_k18_55`, so they are skipped.
pub fn live_k26_msg9(mut q: bevy_ecs::prelude::ResMut<LiveQ>, c: Option<bevy_ecs::prelude::ResMut<crate::engine::classes::plugin::Classes>>) {
    let Some(mut c) = c else { return };
    let msgs = std::mem::take(&mut c.host.k26_msg9_live);
    let Some(m) = q.mem.as_mut() else { return };
    for (owner, args) in msgs {
        let Some(&obj) = (owner as usize).checked_sub(1).and_then(|o| m.obj_addr.get(o)) else { continue };
        let f = m.slot_target(obj, 4);
        if f == 0 || f == 0x0013_5ad8 || f == 0x001b_68b8 {
            continue;
        }
        let sp = super::q2mem::stack_push(m, 16);
        for (k, w) in args.iter().enumerate() {
            m.w32(sp + 4 * k as u32, *w);
        }
        m.enter(obj, 4, &[9, sp], &[]);
        super::q2mem::stack_pop(m, 16);
    }
}

pub struct LiveQPlugin;
impl bevy_app::Plugin for LiveQPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        use bevy_ecs::prelude::*;
        app.init_resource::<LiveQ>().add_systems(bevy_app::Update, tick_live_q.after(crate::engine::classes::plugin::tick_classes))
            .add_systems(bevy_app::Update, live_k26_msg9.after(crate::engine::classes::plugin::tick_classes).before(tick_live_q))
            .add_systems(bevy_app::Update, live_jt_bridge.after(tick_live_q).after(crate::engine::player::plugin::tick_player));
    }
}

/// WIRE-UI: the guest pad block at `base` (= pad object + 0x1000) as a `PadState` (repr(C), 0x1000..0x10f0).
pub fn read_pad<M: super::q2mem::QMem + ?Sized>(m: &mut M, base: u32) -> crate::engine::input::pad::PadState {
    let mut p = crate::engine::input::pad::PadState::default();
    let n = std::mem::size_of::<crate::engine::input::pad::PadState>();
    // SAFETY: PadState is repr(C) plain old data (ints / floats / byte arrays)
    let b = unsafe { std::slice::from_raw_parts_mut(&mut p as *mut _ as *mut u8, n) };
    for (i, x) in b.iter_mut().enumerate() {
        *x = m.r8(base + i as u32);
    }
    p
}
/// WIRE-UI: store `p` into the guest pad block at `base` (pad object + 0x1000).
pub fn write_pad<M: super::q2mem::QMem + ?Sized>(m: &mut M, base: u32, p: &crate::engine::input::pad::PadState) {
    let n = std::mem::size_of::<crate::engine::input::pad::PadState>();
    // SAFETY: see read_pad
    let b = unsafe { std::slice::from_raw_parts(p as *const _ as *const u8, n) };
    for (i, &x) in b.iter().enumerate() {
        m.w8(base + i as u32, x);
    }
}

#[cfg(test)]
mod pool_tests {
    use super::*;

    /// FUN_0011cd98 returns a block to the replaced pool; the next FUN_0011ccc0 reuses it zeroed.
    #[test]
    fn pool_free_block_is_reused_zeroed() {
        let mut m = LiveMem::default();
        let pool = 0x002e_0000;
        m.w32(pool + 0x10, 1);
        let p = m.call(0x0011_ccc0, &[pool], &[]).0;
        m.w32(p + 8, 0xdead_beef);
        m.call(0x0011_cd98, &[pool, p], &[]);
        m.call(0x0011_cd98, &[pool, p], &[]); // double free ignored
        assert_eq!(m.r32(pool + 0x10), 0);
        assert_eq!(m.call(0x0011_ccc0, &[pool], &[]).0, p);
        assert_eq!(m.r32(p + 8), 0);
        assert_ne!(m.call(0x0011_ccc0, &[pool], &[]).0, p);
    }
}

/// CRAWL R36: page-base hasher for `QMem2::pages` (keys are page-aligned u32s; SipHash dominated the
/// per-read cost of the 9805-object hub). Same map contents, cheaper lookups.
#[derive(Default, Clone, Copy)]
pub struct PageHasher(u64);
impl std::hash::Hasher for PageHasher {
    fn finish(&self) -> u64 {
        // page bases have zero low bits: fold the high product bits down (hashbrown buckets on the low bits)
        self.0 ^ (self.0 >> 29)
    }
    fn write(&mut self, b: &[u8]) {
        for &x in b {
            self.0 = (self.0.rotate_left(8) ^ x as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        }
    }
    fn write_u32(&mut self, x: u32) {
        self.0 = (x as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    }
}
pub type PageHash = std::hash::BuildHasherDefault<PageHasher>;
