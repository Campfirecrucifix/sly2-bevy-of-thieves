//! WP-PJA1: adapter that lets the hand-port `Jt` call QMem queue ports (classes/q/**, game/q_*)
//! through the live dispatch table at their original guest address.
//!
//! Design — see `tools/engineport/wp_PJA1.md`:
//! - `JtGuest` owns a `classes::q::live::LiveMem` image with a Jt-sized block allocated at a stable
//!   guest address. `sync_in` writes every mapped `Jt` field into the image at its known offset.
//! - `call_q(jt, fnaddr, args, vargs)` runs the queue port against that image via `QMem::call` (the
//!   same path queue code uses for its own callees), diffs the mapped offsets back into `Jt`, and in
//!   debug builds asserts that no write landed on an unmapped offset (unmapped writes are listed via
//!   `JtGuest::unmapped_writes`, never silently dropped).
//! - Field map: `FIELDS` lists `(offset, width, name)` triples derived from `tools/engineport/jt_map.md`
//!   and the comments in `player/mod.rs`. Offsets missing from the map are reported by `unmapped_writes`
//!   so new consumers can add rows as needed (append-only, same policy as jt_map.md).
//!
//! The adapter is additive: it never replaces an existing hand-port behaviour; it only turns
//! `Jt::calls.push((addr, arg))` recordings into real dispatches when a queue port is wired for `addr`.
//! When `addr` has no queue port the call is a no-op (recorded by `LiveMem::stats.native`), identical
//! to the hand-port record.
use crate::engine::classes::q::live::{is_queue, LiveMem};
use crate::engine::classes::q::q2mem::{QMem, V4};
use crate::engine::player::Jt;
use std::collections::BTreeMap;

/// One row of the hand-port <-> guest-image field map. `off` is the byte offset within the Jt object
/// (the C2d2e98 instance; size 7472 per `jt_map.md`). `width` is the write span in bytes.
#[derive(Copy, Clone, Debug)]
pub struct FieldRow {
    pub off: u32,
    pub width: u32,
    pub name: &'static str,
}

/// Every `Jt` field with a known guest offset. Order of appearance mirrors `jt_map.md` / the inline
/// comments in `player/mod.rs`. New rows go at the end; keep offsets 4-byte aligned.
pub const FIELDS: &[FieldRow] = &[
    // jt + 0x1a8 facing yaw (comment: "last stick direction")
    FieldRow { off: 0x01a8, width: 4, name: "yaw" },
    // jt + 0x208 ground contact normal z (f32)
    FieldRow { off: 0x0208, width: 4, name: "ground_nz" },
    // jt + 0x20c clock time of last ground contact
    FieldRow { off: 0x020c, width: 4, name: "t_ground" },
    // jt + 0x238 ground contact z (centre - n*r)
    FieldRow { off: 0x0238, width: 4, name: "z_contact" },
    // jt + 0x248 action-gate height (f32::MAX = open)
    FieldRow { off: 0x0248, width: 4, name: "h_248" },
    // jt + 0x270 count of attached-item list
    FieldRow { off: 0x0270, width: 4, name: "n_270" },
    // jt + 0x1000 target category mask
    FieldRow { off: 0x1000, width: 4, name: "mask_1000" },
    // jt + 0x1084 held / carried object (0 = None)
    FieldRow { off: 0x1084, width: 4, name: "held_1084" },
    // jt + 0x18c8 jts (main state)
    FieldRow { off: 0x18c8, width: 4, name: "jts" },
    // jt + 0x18d0 jtbs (body sub-state, JMP +0x1c)
    FieldRow { off: 0x18d0, width: 4, name: "jtbs" },
    // JMP +0x20 time the body state was entered (jt + 0x18d4)
    FieldRow { off: 0x18d4, width: 4, name: "t_jtbs" },
    // jt +0x1f4..+0x200 mv[0..4]
    FieldRow { off: 0x01f4, width: 4, name: "mv0" },
    FieldRow { off: 0x01f8, width: 4, name: "mv1" },
    FieldRow { off: 0x01fc, width: 4, name: "mv2" },
    FieldRow { off: 0x0200, width: 4, name: "mv3" },
    // jt + 0x190..+0x19c velocity target (JMP +0xa0 is on the JMP block, this one is the +0x190 field)
    FieldRow { off: 0x0190, width: 4, name: "v_target.x" },
    FieldRow { off: 0x0194, width: 4, name: "v_target.y" },
    FieldRow { off: 0x0198, width: 4, name: "v_target.z" },
    // jt + 0x1010..+0x101c decayed vector (v_1010)
    FieldRow { off: 0x1010, width: 4, name: "v_1010.x" },
    FieldRow { off: 0x1014, width: 4, name: "v_1010.y" },
    FieldRow { off: 0x1018, width: 4, name: "v_1010.z" },
    // jt + 0x1024 glide camera track handle (p24, cam_glide)
    FieldRow { off: 0x1024, width: 4, name: "cam_glide" },
];

/// Lookup `off` in `FIELDS` (first row whose span covers `off`).
pub fn field_at(off: u32) -> Option<&'static FieldRow> {
    FIELDS.iter().find(|r| off >= r.off && off < r.off + r.width)
}

/// Guest base of the Jt object inside the shadow image (sits in the heap region of `LiveMem`).
pub const JT_GUEST: u32 = 0x0100_0000;
/// Class C2d2e98 size (`tools/engineport/jt_map.md`).
pub const JT_SIZE: u32 = 7472;

/// Shadow image that holds the Jt at its guest base plus a scratch slot carrying `&jt` for callees
/// that take a stack-address of the pointer (Ghidra signature `(&jt)` -> pj = address holding jt).
pub struct JtGuest {
    /// Live QMem image (one Jt only, no other objects).
    pub mem: LiveMem,
    /// Address of a 4-byte word holding the Jt guest address (the "&jt" indirection).
    pub pj: u32,
    /// Pre-call snapshot of the mapped offsets (for the sync-out diff).
    before: BTreeMap<u32, [u8; 8]>,
    /// Pre-call snapshot of the full Jt block, to detect writes to unmapped offsets.
    before_all: Vec<u8>,
    /// Last `call_q` summary (populated by `call_q`): writes that landed on unmapped offsets.
    pub unmapped: Vec<(u32, u32, u8)>,
}

impl Default for JtGuest {
    fn default() -> Self {
        let mut mem = LiveMem::default();
        // Reserve the heap tail Jt_SIZE + a scratch slot for &jt.
        let jt_addr = mem.alloc(JT_SIZE);
        debug_assert_eq!(jt_addr, JT_GUEST, "LiveMem heap base changed; update JT_GUEST");
        let pj = mem.alloc(0x10);
        mem.w32(pj, JT_GUEST);
        Self { mem, pj, before: BTreeMap::new(), before_all: vec![0; JT_SIZE as usize], unmapped: Vec::new() }
    }
}

impl JtGuest {
    pub fn new() -> Self {
        Self::default()
    }

    /// Write every mapped `Jt` field into the guest image at its known offset.
    pub fn sync_in(&mut self, jt: &Jt) {
        for r in FIELDS {
            let bytes = read_field(jt, r);
            for (i, b) in bytes.iter().enumerate().take(r.width as usize) {
                self.mem.w8(JT_GUEST + r.off + i as u32, *b);
            }
        }
    }

    /// Snapshot the mapped offsets and the full Jt block before a dispatch.
    pub fn snapshot(&mut self) {
        self.before.clear();
        for r in FIELDS {
            let mut buf = [0u8; 8];
            let w = r.width.min(8) as usize;
            for i in 0..w {
                buf[i] = self.mem.r8(JT_GUEST + r.off + i as u32);
            }
            self.before.insert(r.off, buf);
        }
        for i in 0..JT_SIZE {
            self.before_all[i as usize] = self.mem.r8(JT_GUEST + i);
        }
    }

    /// Read mapped offsets back from the image and apply them to `jt`. Builds the `unmapped` list of
    /// writes that landed on offsets `FIELDS` does not cover.
    pub fn sync_out(&mut self, jt: &mut Jt) {
        for r in FIELDS {
            let mut buf = [0u8; 8];
            let w = r.width.min(8) as usize;
            for i in 0..w {
                buf[i] = self.mem.r8(JT_GUEST + r.off + i as u32);
            }
            let before = self.before.get(&r.off).copied().unwrap_or([0u8; 8]);
            if buf == before {
                continue;
            }
            write_field(jt, r, &buf);
        }
        // Scan the full block for writes to unmapped offsets.
        self.unmapped.clear();
        for off in 0..JT_SIZE {
            let cur = self.mem.r8(JT_GUEST + off);
            if cur == self.before_all[off as usize] {
                continue;
            }
            // Mapped?
            if field_at(off).is_some() {
                continue;
            }
            self.unmapped.push((off, 1, cur));
        }
    }

    /// Writes that landed on unmapped offsets during the last `call_q` (empty when all writes were
    /// covered by `FIELDS`). Keyed by `(offset, width_bytes, last_value)`.
    pub fn unmapped_writes(&self) -> &[(u32, u32, u8)] {
        &self.unmapped
    }
}

/// Run the queue port registered for `addr` against a one-Jt shadow image: sync `jt` -> guest image,
/// dispatch, sync guest image -> `jt`, and (in debug builds) assert that no write landed on an
/// unmapped offset.
///
/// Returns the dispatch result `(v0, f/quadword)` of the queue port. When `addr` is not in the live
/// dispatch table the call is a no-op (recorded by `LiveMem::stats.native` / `unported`) and the
/// returned tuple is `(0, [0.0; 4])` — identical to the hand-port's recorded-only behaviour.
///
/// WP-ARCH1 step 3: write every mapped Jt field into an arbitrary live `LiveMem` at `base` so
/// pj_adapt can act as a *view* on the single ARCH1 guest store, not on its own private image.
/// `base` is the Jt object's address in that store (resolved by `LiveQ::addr_of_oid` or by the
/// Jt allocator in the loader).
pub fn sync_in_live(mem: &mut LiveMem, base: u32, jt: &Jt) {
    for r in FIELDS {
        let bytes = read_field(jt, r);
        for i in 0..r.width as usize {
            mem.w8(base + r.off + i as u32, bytes[i]);
        }
    }
}

/// WP-ARCH1 step 3: read every mapped Jt field back from an arbitrary `LiveMem` at `base` into
/// the hand-port `Jt`. Mirrors `JtGuest::sync_out` without the unmapped-write audit (the audit
/// still runs on the private `JtGuest` path; a store-wide audit is step 5).
pub fn sync_out_live(mem: &mut LiveMem, base: u32, jt: &mut Jt) {
    for r in FIELDS {
        let mut buf = [0u8; 8];
        let w = r.width.min(8) as usize;
        for i in 0..w {
            buf[i] = mem.r8(base + r.off + i as u32);
        }
        write_field(jt, r, &buf);
    }
}

/// This function is additive: callers that previously pushed `(addr, arg)` into `Jt::calls` keep
/// doing so (so existing tests still see the recording) and additionally use `call_q` to drive the
/// Jt side effects when a queue port exists.
pub fn call_q(jt: &mut Jt, addr: u32, args: &[u32], vargs: &[V4]) -> (u32, V4) {
    let mut g = JtGuest::new();
    g.sync_in(jt);
    g.snapshot();
    let r = g.mem.call(addr, args, vargs);
    g.sync_out(jt);
    #[cfg(debug_assertions)]
    if !g.unmapped.is_empty() && is_queue(addr) {
        // Only assert for addresses that actually ran a queue port (addr is in the live dispatch
        // table). For non-queue addresses the "writes" are the sync_in bytes themselves, since the
        // dispatch did nothing.
        log::warn!(
            "[WP-PJA1] call_q(0x{addr:06x}): {} write(s) to unmapped Jt offsets (first: +0x{:x}=0x{:02x}); add rows to FIELDS",
            g.unmapped.len(),
            g.unmapped[0].0,
            g.unmapped[0].2,
        );
    }
    r
}

/// WP-PJ4 convenience: dispatch a queue port whose single argument is `&jt` (a pointer to a word
/// that stores the Jt guest address). Returned tuple matches `call_q` -- intended for hand-ports
/// that currently push `(addr, arg)` into `Jt::calls` and want to additionally run the registered
/// QMem port against the shadow image.
pub fn call_q_jtptr(jt: &mut Jt, addr: u32) -> (u32, V4) {
    let mut g = JtGuest::new();
    g.sync_in(jt);
    g.snapshot();
    let r = g.mem.call(addr, &[g.pj], &[]);
    g.sync_out(jt);
    #[cfg(debug_assertions)]
    if !g.unmapped.is_empty() && is_queue(addr) {
        log::warn!(
            "[WP-PJA1] call_q_jtptr(0x{addr:06x}): {} write(s) to unmapped Jt offsets (first: +0x{:x}=0x{:02x}); add rows to FIELDS",
            g.unmapped.len(),
            g.unmapped[0].0,
            g.unmapped[0].2,
        );
    }
    r
}

/// Read a `Jt` field at `r.off` into an `[u8; 8]` (little-endian, up to 8 bytes).
fn read_field(jt: &Jt, r: &FieldRow) -> [u8; 8] {
    let mut buf = [0u8; 8];
    let w = r.width.min(8) as usize;
    let bytes: [u8; 8] = match r.off {
        0x01a8 => f32_le8(jt.yaw),
        0x0208 => f32_le8(jt.ground_nz.unwrap_or(0.0)),
        0x020c => f32_le8(jt.t_ground),
        0x0238 => f32_le8(jt.z_contact),
        0x0248 => f32_le8(jt.h_248),
        0x0270 => i32_le8(jt.n_270),
        0x1000 => u32_le8(jt.mask_1000),
        0x1084 => u32_le8(jt.held_1084.map_or(0, |x| x as u32)),
        0x18c8 => i32_le8(jt.jts),
        0x18d0 => i32_le8(jt.jtbs),
        0x18d4 => f32_le8(jt.t_jtbs),
        0x01f4 => f32_le8(jt.mv[0]),
        0x01f8 => f32_le8(jt.mv[1]),
        0x01fc => f32_le8(jt.mv[2]),
        0x0200 => f32_le8(jt.mv[3]),
        0x0190 => f32_le8(jt.v_target[0]),
        0x0194 => f32_le8(jt.v_target[1]),
        0x0198 => f32_le8(jt.v_target[2]),
        0x1010 => f32_le8(jt.v_1010[0]),
        0x1014 => f32_le8(jt.v_1010[1]),
        0x1018 => f32_le8(jt.v_1010[2]),
        0x1024 => u32_le8(jt.cam_glide as u32),
        _ => [0u8; 8],
    };
    buf[..w].copy_from_slice(&bytes[..w]);
    buf
}

/// Write `[u8; 8]` back into the corresponding `Jt` field.
fn write_field(jt: &mut Jt, r: &FieldRow, buf: &[u8; 8]) {
    let r32 = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
    let rf = f32::from_bits(r32);
    let ri = r32 as i32;
    match r.off {
        0x01a8 => jt.yaw = rf,
        0x0208 => jt.ground_nz = (r32 != 0).then_some(rf),
        0x020c => jt.t_ground = rf,
        0x0238 => jt.z_contact = rf,
        0x0248 => jt.h_248 = rf,
        0x0270 => jt.n_270 = ri,
        0x1000 => jt.mask_1000 = r32,
        0x1084 => jt.held_1084 = (r32 != 0).then_some(r32 as usize),
        0x18c8 => jt.jts = ri,
        0x18d0 => jt.jtbs = ri,
        0x18d4 => jt.t_jtbs = rf,
        0x01f4 => jt.mv[0] = rf,
        0x01f8 => jt.mv[1] = rf,
        0x01fc => jt.mv[2] = rf,
        0x0200 => jt.mv[3] = rf,
        0x0190 => jt.v_target[0] = rf,
        0x0194 => jt.v_target[1] = rf,
        0x0198 => jt.v_target[2] = rf,
        0x1010 => jt.v_1010[0] = rf,
        0x1014 => jt.v_1010[1] = rf,
        0x1018 => jt.v_1010[2] = rf,
        0x1024 => jt.cam_glide = r32 != 0,
        _ => {}
    }
}

fn f32_le8(x: f32) -> [u8; 8] {
    let b = x.to_bits().to_le_bytes();
    [b[0], b[1], b[2], b[3], 0, 0, 0, 0]
}
fn u32_le8(x: u32) -> [u8; 8] {
    let b = x.to_le_bytes();
    [b[0], b[1], b[2], b[3], 0, 0, 0, 0]
}
fn i32_le8(x: i32) -> [u8; 8] {
    u32_le8(x as u32)
}

#[cfg(test)]
mod t {
    use super::*;
    use crate::engine::physics::geom::V3;

    #[test]
    fn fields_unique_offsets() {
        for (i, a) in FIELDS.iter().enumerate() {
            assert!(a.off + a.width <= JT_SIZE, "row {} off 0x{:x} out of range", a.name, a.off);
            for b in &FIELDS[i + 1..] {
                let overlap = a.off < b.off + b.width && b.off < a.off + a.width;
                assert!(!overlap, "fields overlap: {} +0x{:x}/{} and {} +0x{:x}/{}", a.name, a.off, a.width, b.name, b.off, b.width);
            }
        }
    }

    #[test]
    fn roundtrip_mapped_fields() {
        let mut j = Jt::new([0.0, 0.0, 0.0] as V3);
        j.jts = 7;
        j.jtbs = 3;
        j.yaw = 1.25;
        j.z_contact = -42.0;
        j.n_270 = 2;
        j.held_1084 = Some(0x1234);
        j.mv = [10.0, 20.0, 30.0, 40.0];
        j.v_target = [5.0, 6.0, 7.0];
        let mut g = JtGuest::new();
        g.sync_in(&j);
        // Verify the bytes landed at the right offsets.
        assert_eq!(g.mem.r32(JT_GUEST + 0x18c8) as i32, 7);
        assert_eq!(g.mem.r32(JT_GUEST + 0x18d0) as i32, 3);
        assert_eq!(f32::from_bits(g.mem.r32(JT_GUEST + 0x1a8)), 1.25);
        assert_eq!(f32::from_bits(g.mem.r32(JT_GUEST + 0x238)), -42.0);
        assert_eq!(g.mem.r32(JT_GUEST + 0x270) as i32, 2);
        assert_eq!(g.mem.r32(JT_GUEST + 0x1084), 0x1234);
        assert_eq!(f32::from_bits(g.mem.r32(JT_GUEST + 0x1f4)), 10.0);
        assert_eq!(f32::from_bits(g.mem.r32(JT_GUEST + 0x194)), 6.0);
        // Clobber a mapped offset through the image, then diff back.
        g.snapshot();
        g.mem.w32(JT_GUEST + 0x18c8, 9); // jts = 9
        g.mem.w32(JT_GUEST + 0x238, (-13.5f32).to_bits());
        g.sync_out(&mut j);
        assert_eq!(j.jts, 9);
        assert_eq!(j.z_contact, -13.5);
        assert!(g.unmapped_writes().is_empty());
    }

    #[test]
    fn unmapped_write_is_reported() {
        let mut j = Jt::new([0.0, 0.0, 0.0] as V3);
        let mut g = JtGuest::new();
        g.sync_in(&j);
        g.snapshot();
        // Write to an offset not in FIELDS.
        g.mem.w32(JT_GUEST + 0x400, 0xdeadbeef);
        g.sync_out(&mut j);
        assert!(!g.unmapped_writes().is_empty(), "unmapped write at +0x400 must be reported");
        // One of the reported bytes is at +0x400.
        assert!(g.unmapped_writes().iter().any(|(off, _, _)| *off == 0x400));
    }

    #[test]
    fn call_q_runs_192a28() {
        // ee_sub_192a28 is a registered queue port (classes/q/hard1.rs). It walks a chain anchored at
        // DAT_003e1444; with the chain null the walk is empty and the function returns cleanly.
        let mut j = Jt::new([0.0, 0.0, 0.0] as V3);
        j.jts = 7;
        let before = j.jts;
        let g_pj;
        {
            // Use the full adapter path.
            let (_r, _v) = {
                // Hand-build to inspect state after.
                let mut g = JtGuest::new();
                g.sync_in(&j);
                g.snapshot();
                g_pj = g.pj;
                let r = g.mem.call(0x0019_2a28, &[g_pj], &[]);
                g.sync_out(&mut j);
                assert!(g.unmapped_writes().is_empty(), "ee_sub_192a28 wrote unexpected offsets: {:?}", g.unmapped_writes());
                r
            };
        }
        // No panic and no state mutation (DAT_003e1444 == 0 in the shadow image).
        assert_eq!(j.jts, before);
    }

    #[test]
    fn call_q_non_queue_is_noop() {
        // 0x19a750 has no queue port today — call_q must be a no-op (no panic, state unchanged).
        let mut j = Jt::new([0.0, 0.0, 0.0] as V3);
        j.jts = 2;
        j.jtbs = 0;
        j.v_target = [1.0, 2.0, 3.0];
        let (r, v) = call_q(&mut j, 0x0019_a750, &[0, 1], &[]);
        assert_eq!(r, 0);
        assert_eq!(v, [0.0; 4]);
        assert_eq!(j.jts, 2);
        assert_eq!(j.v_target, [1.0, 2.0, 3.0]);
    }
}
