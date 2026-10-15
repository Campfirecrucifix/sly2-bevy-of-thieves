//! WP-WIRE3: ownership of per-object fields between the queue port (`live::LiveMem` guest image) and the
//! hand port (`World` + `ClassSet`). One row per `(class, offset, width)`; policy in `recomp/WIRE_OWNERSHIP.md`.
//!
//! The queue update runs in a shadow QMem image. Per-frame sync:
//!   - Before `run_slot(SLOT_UPDATE, ..)`: `copy_hand_to_guest` writes every `Owner::Hand` field from the
//!     hand-port state into the guest image (feed-in).
//!   - After `run_slot(SLOT_UPDATE, ..)`: `snapshot_queue` reads every `Owner::Queue` field out of the guest
//!     into `LiveQ::queue_fields`, the public bus other systems read (feed-out).
//!
//! Items marked `Owner::Hand` with no live hand-port getter are listed only to pin the decision; their
//! copy-in is a no-op until the getter is added. See the "Deferred" list in `WIRE_OWNERSHIP.md`.
use crate::engine::classes::q::q2mem::QMem;
use crate::engine::world::World;
use std::collections::HashMap;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum WireOwner {
    /// QMem guest image is authoritative; consumer reads `LiveQ::queue_fields` after the frame.
    Queue,
    /// Hand-port state is authoritative; the queue update sees it as input only.
    Hand,
    /// Informational row, no cross-port exchange (local to one side).
    Neither,
}

/// `(class_id, offset, width_bytes, owner, kind)`. Class id is `world::class::ClassId` (u16); offset is
/// relative to the object's guest base (header start). Width is 1/4/8/48/... (bytes copied verbatim).
pub struct WireField {
    pub class: u16,
    pub offset: u32,
    pub width: u32,
    pub owner: WireOwner,
    pub kind: &'static str,
}

/// WIRE3 start: ONE blanket row for every class that reaches queue code — the LO base `+0x28` u64 flags are
/// RMW'd by queue update code (`c2caf88`, `q6_small`, `c2cc548` and transitively); never by the hand port
/// after load (see `classes/plugin.rs::tick_classes`). Class = `u16::MAX` means "every class" (class filter
/// is skipped). Per-class rows are added as per-class audits land.
///
/// WIRE3b: added blanket Queue rows for scalar state offsets written by per-frame slot-15 queue ports on
/// Episode 1 levels — none of them are touched by the hand port today (`classes/plugin.rs::tick_classes`
/// never writes LO bytes after load; per-class SM state lives in `ClassSet`). Offsets:
///   - `+0x150` state machine word on C2cc548 (`c2cc548::ee_sub_147278` -> `q14_e::ee_sub_147410`), updated
///     on the first slot-15 tick (state 0 -> 1) and on message events.
///   - `+0x154` state-change timestamp on C2cc548 (`q14_e::ee_sub_147410` writes `g_clock`).
///   - `+0x1c0` SM state on C2c9c20 volume (`q6_runs::ee_sub_149440`); `+0x1d8` first-live-candidate.
///   - `+0x30` scalar state on C2cc1c0 / C2cc2c0 / C2cdd40 slot overrides (sentinels / counters per
///     `c2cc1c0::ee_sub_146f18`, `c2cc2c0`).
/// Keeping them blanket (class = u16::MAX) is sound: offsets past the object's allocated size read as 0
/// (`LiveMem` zero pages) and stay 0, so classes without the field contribute invariant rows to the bus.
pub const TABLE: &[WireField] = &[
    WireField { class: u16::MAX, offset: 0x28, width: 8, owner: WireOwner::Queue, kind: "lo_flags_28" },
    WireField { class: u16::MAX, offset: 0x30, width: 4, owner: WireOwner::Queue, kind: "q_state_30" },
    WireField { class: u16::MAX, offset: 0x150, width: 4, owner: WireOwner::Queue, kind: "q_sm_state_150" },
    WireField { class: u16::MAX, offset: 0x154, width: 4, owner: WireOwner::Queue, kind: "q_sm_t_154" },
    WireField { class: u16::MAX, offset: 0x1c0, width: 4, owner: WireOwner::Queue, kind: "q_sm_state_1c0" },
    WireField { class: u16::MAX, offset: 0x1d8, width: 4, owner: WireOwner::Queue, kind: "q_live_cand_1d8" },
];

/// Keys the `LiveQ::queue_fields` output bus.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FieldKey {
    pub obj: u32,
    pub offset: u32,
    pub width: u32,
}

/// After the queue update, snapshot every `Owner::Queue` row of `TABLE` out of `m` for every live object.
/// The returned map is overwritten in place each frame to avoid reallocation.
pub fn snapshot_queue<M: QMem + ?Sized>(m: &mut M, obj_addr: &[u32], w: &World, out: &mut HashMap<FieldKey, [u8; 8]>) {
    out.clear();
    for (i, &a) in obj_addr.iter().enumerate() {
        let cls = w.objects.get(i).map(|o| o.class).unwrap_or(0);
        for r in TABLE {
            if r.owner != WireOwner::Queue || (r.class != u16::MAX && r.class != cls) {
                continue;
            }
            let mut buf = [0u8; 8];
            let w = r.width.min(8) as usize;
            for k in 0..w {
                buf[k] = m.r8(a + r.offset + k as u32);
            }
            out.insert(FieldKey { obj: a, offset: r.offset, width: r.width }, buf);
        }
    }
}

/// Pre-frame feed-in: every `Owner::Hand` row of `TABLE` would be written here. Current `TABLE` has none
/// (see Deferred in `WIRE_OWNERSHIP.md`), so this is a no-op; wired up front so the writeback path exists
/// the moment a getter lands.
pub fn copy_hand_to_guest<M: QMem + ?Sized>(_m: &mut M, _obj_addr: &[u32], _w: &World) {
    for r in TABLE {
        if r.owner == WireOwner::Hand {
            // placeholder: no hand getter wired yet (WIRE3 deferred, see WIRE_OWNERSHIP.md)
            let _ = r;
        }
    }
}

/// Guard: a `TABLE` row must never appear with both owners (one owner per `(class, offset, width)`).
#[cfg(test)]
mod t {
    use super::*;
    #[test]
    fn no_double_owner() {
        for (i, a) in TABLE.iter().enumerate() {
            for b in &TABLE[i + 1..] {
                if a.class == b.class && a.offset == b.offset && a.width == b.width {
                    assert_eq!(a.owner, b.owner, "double-owner row 0x{:x}+{} class {}", a.offset, a.width, a.class);
                }
            }
        }
    }
}
