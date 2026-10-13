//! Object records: FUN_0013fb40 (object list), FUN_0013e3c8 (instantiate), class `load`
//! virtuals (vtable +0x74), FUN_0013e510 (clone).
//!
//! Record: s8 n; [n > 0: n x s16 mission list]; s16 class; s32 oid; s16 parent; s32 size;
//! then `size` payload bytes consumed by the class load (or skipped by FUN_001179b8 when the
//! record is rejected). n == -1 / -2: create only if no object with `oid` exists yet
//! (FUN_0015e958 0x805 / FUN_0015e938 0x405 lookups), else skip.

use super::class::{self, ClassId, CID_6, LOAD_231170, LOAD_LO};
use super::entry::ee_sub_13fad0;
use super::loads::{Bsp, Geom, Solid, Xform};
use super::props::{ee_sub_13f810, Prop};
use crate::engine::disc::lz::BinaryInputStream;

/// How an object's payload was handled.
#[derive(Clone, Debug, PartialEq)]
pub enum Payload {
    /// class load ported 1:1 and consumed exactly `size` bytes
    Loaded(LoadData),
    /// class load not ported yet: payload skipped by the recorded size (offset kept for later)
    Unported,
    /// class-6 subtree: handed to FUN_001c9b08 (not ported; skipped by size)
    Handler1c9b08,
    /// copy made by FUN_0013e510 (vtable +0x144 copies state from the source)
    Clone,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LoadData {
    /// FUN_001a5c28 (LO): property list
    Lo(Vec<Prop>),
    /// FUN_00231170: FUN_00231768 (s16 -> +0x50 via vtable +0x18c, 4x3 matrix +0x60, u32
    /// FUN_001a6d28) + property list
    L231170 { s: i16, xform: [u8; 48], word: u32, props: Vec<Prop> },
    /// FUN_00227380
    L227380 { xform: Xform, word: u32, geom: Geom, bsp: Bsp, props: Vec<Prop> },
    /// FUN_00228140 / FUN_00228720: FUN_00231170 + FUN_00118420 blob
    L231170Blob { xform: Xform, props: Vec<Prop>, blob: Vec<u8> },
    /// FUN_00227bb8 / FUN_00149358
    Solid(Solid),
    /// FUN_002282b0
    L2282b0 { solid: Solid, first: Option<[u8; 64]>, list: Vec<Option<[u8; 64]>> },
    /// FUN_001f31e0 (and wrappers FUN_00223fe0, FUN_001c3dc8/FUN_002039c0/FUN_001553f0 with
    /// the FUN_00118420 blob in `tail`); children are in `Lo::children`.
    Alo { alo: Box<super::alo::Alo>, tail: Option<Vec<u8>> },
    /// FUN_0012f2a8 (classes 109/110, WP-D `anim::aseg`)
    Aseg(Box<crate::engine::anim::aseg::Aseg>),
    /// WP-B4 loads (world/loads_b4.rs), incl. the class-6 handler FUN_001c9b08
    B4(Box<super::loads_b4::B4Load>),
}

/// One LO instance (object +0x10 owner, +0x18 oid, +0x50 disabled flag).
#[derive(Clone, Debug, PartialEq)]
pub struct Lo {
    pub class: ClassId,
    pub oid: i32,
    /// index into the entry's FUN_001ff858 anchor list (FUN_001ffa40), -1 = none
    pub parent_slot: i16,
    /// owning object (param_1 of FUN_0013fb40; None at entry level)
    pub owner: Option<usize>,
    pub children: Vec<usize>,
    /// directory entry index
    pub entry: usize,
    /// mission list (s8 n > 0) or the -1/-2 singleton marker
    pub filter: Vec<i16>,
    pub singleton: Option<i8>,
    /// +0x50 = 1: created although the mission filter failed (classes 0x84/0x85 only)
    pub disabled: bool,
    pub payload_offset: u32,
    pub payload_size: i32,
    pub payload: Payload,
    pub clone_of: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorldError {
    /// stream ended inside a record
    Eof { what: &'static str, offset: u32 },
    /// a ported class load did not consume exactly the recorded size
    SizeMismatch { class: ClassId, offset: u32, size: i32, consumed: u32 },
    /// class id outside the 199-entry descriptor table / abstract class
    BadClass { class: i32, offset: u32 },
    /// property field id outside the descriptor table
    BadField { field: i16, offset: u32 },
    /// a value outside what the original's tables cover
    BadData { what: &'static str, offset: u32 },
}

impl std::fmt::Display for WorldError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for WorldError {}

/// Object store used while loading (the 0x3dd490 object manager's oid lookup).
pub trait ObjectSink {
    fn objects(&self) -> &[Lo];
    fn push(&mut self, lo: Lo) -> usize;
    fn exists(&self, oid: i32) -> bool;
    fn get_mut(&mut self, i: usize) -> &mut Lo;
    /// WP-B4: directory entry b (+0x60) of the entry whose c (+0x10) is `c` (FUN_001cb770)
    fn template_b(&self, c: i32) -> Option<i32> {
        let _ = c;
        None
    }
    /// WP-B5: FUN_001cb770 (index of the entry with +0x10 == c)
    fn template_index(&self, c: i32) -> Option<usize> {
        let _ = c;
        None
    }
    /// WP-B5: FUN_001ca370 for class-6 record `record`; returns the copy's objects
    fn instantiate_template(&mut self, record: usize, entry: Option<usize>, owner: Option<usize>, bone: i16, oid: i32, xform: [u8; 48]) -> Vec<usize> {
        let _ = (record, entry, owner, bone, oid, xform);
        Vec::new()
    }
    /// WP-B5: FUN_001c9b08 key/xform pass over the record's nested objects
    fn template_nested(&mut self, nested: &[usize], xform: [u8; 48]) {
        let _ = (nested, xform);
    }
    /// WP-B5: vtable +0x4c (FUN_001a5630)
    fn post_load(&mut self, i: usize) {
        let _ = i;
    }
    /// WP-B5: hook after a B4 load finished (class 15/31 attached objects)
    fn after_load(&mut self, i: usize) {
        let _ = i;
    }
}

/// Ported `load` virtuals, by vtable +0x74 address. None = not ported.
fn load_virtual(load: u32, s: &mut BinaryInputStream, off: u32) -> Option<Result<LoadData, WorldError>> {
    let props = |s: &mut BinaryInputStream| ee_sub_13f810(s).map_err(|e| WorldError::BadField { field: e.field, offset: e.offset });
    match load {
        // 0x001a5c28
        LOAD_LO => Some(props(s).map(LoadData::Lo)),
        // 0x00231170 -> 0x00231768
        LOAD_231170 => Some((|| {
            let sv = s.read_s16();
            let mut xform = [0u8; 48];
            s.read(48, Some(&mut xform)); // FUN_00117f18: 3 rows + translation, 12 bytes each
            let word = s.read_u32();
            if s.at_eof() {
                return Err(WorldError::Eof { what: "FUN_00231768", offset: off });
            }
            Ok(LoadData::L231170 { s: sv, xform, word, props: props(s)? })
        })()),
        // 0x0012f2a8 (WP-D)
        crate::engine::anim::aseg::LOAD_ASEG => Some(crate::engine::anim::aseg::ee_sub_12f2a8(s).map(|a| LoadData::Aseg(Box::new(a)))),
        _ => super::loads::load(load, s).map(|r| {
            let r = r?;
            if s.at_eof() {
                return Err(WorldError::Eof { what: "class load", offset: off });
            }
            Ok(r)
        }),
    }
}

/// FUN_0013e3c8 + vtable +0x74: instantiate and load one object; payload handled per class.
fn create_and_load(
    w: &mut dyn ObjectSink,
    s: &mut BinaryInputStream,
    mut lo: Lo,
    mission: i16,
) -> Result<usize, WorldError> {
    // 0x0013e3c8
    let start = s.tell();
    let ci = class::class(lo.class).filter(|c| c.ctor != 0).ok_or(WorldError::BadClass { class: lo.class as i32, offset: start })?;
    // ALO chain: plain/B3 wrappers, then the WP-D actor wrappers (anim::aseg::alo_wrapper_tail;
    // same stream layout as anim::load_alo_wrapper)
    // --- WP-B4 hook: classes with load 0 in classes_gen (vtable set by an init fn) + loads_b4.rs ---
    let load = super::loads_b4::effective_load(lo.class, ci.load);
    if super::loads_b4::handles(load) {
        return super::loads_b4::create_and_load(w, s, lo, load, mission);
    }
    // --- end WP-B4 hook ---
    if let Some(tail) = super::alo::alo_load_kind(load).or_else(|| crate::engine::anim::aseg::alo_wrapper_tail(load)) {
        // FUN_0013e3c8 creates the object before its load: push first so children see it
        let (entry, size, cls) = (lo.entry, lo.payload_size, lo.class);
        let idx = w.push(lo);
        let alo = super::alo::ee_sub_1f31e0(s, mission)?;
        // FUN_0013fb40(this, stream, 0): nested objects owned by this one
        ee_sub_13fb40(w, Some(idx), entry, s, mission)?;
        // FUN_001a6d28: no stream data
        let tail = super::alo::read_tail(tail, s);
        if s.at_eof() {
            return Err(WorldError::Eof { what: "ALO load", offset: start });
        }
        let consumed = s.tell() - start;
        if consumed != size as u32 {
            return Err(WorldError::SizeMismatch { class: cls, offset: start, size, consumed });
        }
        w.get_mut(idx).payload = Payload::Loaded(LoadData::Alo { alo: Box::new(alo), tail });
        return Ok(idx);
    }
    match load_virtual(load, s, start) {
        Some(r) => {
            lo.payload = Payload::Loaded(r?);
            let consumed = s.tell() - start;
            if consumed != lo.payload_size as u32 {
                return Err(WorldError::SizeMismatch { class: lo.class, offset: start, size: lo.payload_size, consumed });
            }
        }
        None => {
            // load virtual not ported: skip by the recorded size (same as FUN_001179b8)
            lo.payload = Payload::Unported;
            s.read(lo.payload_size.max(0) as usize, None);
        }
    }
    Ok(w.push(lo))
}

/// FUN_0013fb40: object list. `owner` = param_1 (0 at entry level), `mission` = DAT_003e1110.
/// Returns the created objects in collector order (FUN_0013f888 list, +0xc append).
pub fn ee_sub_13fb40(
    w: &mut dyn ObjectSink,
    owner: Option<usize>,
    entry: usize,
    s: &mut BinaryInputStream,
    mission: i16,
) -> Result<Vec<usize>, WorldError> {
    // 0x0013fb40
    let mut created = Vec::new();
    let count = s.read_s16();
    for _ in 0..count.max(0) {
        let rec = s.tell();
        let n = s.read_u8() as i8;
        // Quirk: n < -2 also "reads" n*2 bytes; a negative size reads nothing in FUN_00117898.
        let filter: Vec<i16> = (0..n.max(0)).map(|_| s.read_s16()).collect();
        let cls = s.read_s16();
        let oid = s.read_s32();
        let parent = s.read_s16();
        let size = s.read_s32();
        if s.at_eof() {
            return Err(WorldError::Eof { what: "FUN_0013fb40 record", offset: rec });
        }
        if cls < 0 || cls as usize >= class::CLASS_COUNT {
            return Err(WorldError::BadClass { class: cls as i32, offset: rec });
        }
        let lo = Lo {
            class: cls as ClassId,
            oid,
            parent_slot: parent,
            owner,
            children: Vec::new(),
            entry,
            filter: filter.clone(),
            singleton: (n == -1 || n == -2).then_some(n),
            disabled: false,
            payload_offset: s.tell(),
            payload_size: size,
            payload: Payload::Unported,
            clone_of: None,
        };
        let skip = |s: &mut BinaryInputStream| {
            s.read(size.max(0) as usize, None);
        };
        // decision tree of FUN_0013fb40
        let go = if n == -1 || n == -2 {
            // FUN_0015e958(0x3dd490, 0x805, oid) / FUN_0015e938(0x3dd490, 0x405, oid)
            !w.exists(oid)
        } else if n > 0 || n < -2 {
            if ee_sub_13fad0(mission, &filter) {
                true
            } else if (cls as u32).wrapping_sub(0x84) <= 1 {
                // created disabled (+0x50 = 1) and loaded
                let mut lo = lo;
                lo.disabled = true;
                created.push(create_and_load(w, s, lo, mission)?);
                continue;
            } else {
                false
            }
        } else {
            true
        };
        if !go {
            skip(s); // FUN_001179b8
            continue;
        }
        // LAB_0013fd38
        if class::ee_sub_15e988(cls as ClassId, CID_6) {
            // FUN_001c9b08(0x3dd490, owner, class, oid, parent, stream, collector): WP-B4
            created.extend(super::loads_b4::ee_sub_1c9b08(w, s, lo, mission)?);
            continue;
        }
        created.push(create_and_load(w, s, lo, mission)?);
    }
    Ok(created)
}

/// FUN_0013e510: clone `src` (same class via vtable +0x34, same oid +0x18, no parent slot);
/// vtable +0x144 copies the state.
pub fn ee_sub_13e510(w: &mut dyn ObjectSink, src: usize) -> usize {
    // 0x0013e510
    let o = &w.objects()[src];
    let lo = Lo {
        class: o.class,
        oid: o.oid,
        parent_slot: -1,
        owner: None,
        children: Vec::new(),
        entry: o.entry,
        filter: Vec::new(),
        singleton: None,
        disabled: o.disabled,
        payload_offset: o.payload_offset,
        payload_size: o.payload_size,
        payload: Payload::Clone,
        clone_of: Some(src),
    };
    w.push(lo)
}
