//! WP-B: level object stream -> world (FUN_0013da18 from the first FUN_0013f810 onward).
//! See ENGINE_PORT.md. Pure Rust core; with feature `game` the [`World`] is a Bevy Resource
//! built from a [`LoadedLevel`] (no rendering here, that is WP-C).
//!
//! Stream after `LevelHeader::objects_offset`:
//!   FUN_0013f810(0x3dd490) and FUN_0013f810(0x2ddf40): global property lists; align 16
//!   per directory entry (in directory order): entry.rs sections, FUN_0013fb40 objects, align 16
//! The last entry ends 1-4 KB before the end of the decompressed level; those bytes are never
//! read (FUN_00184cf0 closes the stream right after FUN_0013da18).

pub mod alo;
pub mod class;
pub mod class_table;
pub mod entry;
pub mod field_table;
pub mod loads;
pub mod loads_b4;
pub mod loads_b5;
pub mod objects;
pub mod props;

use crate::engine::disc::level_file::{memory_stream, DirEntry};
use crate::engine::disc::lz::BinaryInputStream;
use crate::engine::disc::LoadedLevel;
use bevy_ecs::prelude::Resource;
use entry::EntrySections;
use objects::{ee_sub_13e510, ee_sub_13fb40, Lo, ObjectSink, Payload, WorldError};
use props::{ee_sub_13f810, Prop};
use std::collections::{BTreeMap, HashMap};

/// Load-time options (game globals read by the loader).
#[derive(Clone, Copy, Debug)]
pub struct LoadOptions {
    /// DAT_003e1110 (FUN_00210a88): current mission; -1 = accept every mission list.
    pub mission: i16,
}
impl Default for LoadOptions {
    fn default() -> Self {
        Self { mission: -1 }
    }
}

/// One directory entry after loading (the 0x6c-byte entry at world+0x3f08).
#[derive(Clone, Debug)]
pub struct EntryInfo {
    pub dir: DirEntry,
    pub sections: EntrySections,
    /// entry+0x00: objects created by FUN_0013fb40 (m)
    pub objects: Vec<usize>,
    /// entry+0x04: m * n instances (first m = `objects`, then FUN_0013e510 clones)
    pub instances: Vec<usize>,
    /// entry+0x0c: load count (template entries, a != 0)
    pub load_count: i32,
    /// entry+0x18: oid per copy (n entries, -1 = unused), written by FUN_001ca370 (WP-B5)
    pub inst_oids: Vec<i32>,
    pub start_offset: u32,
    pub end_offset: u32,
}

/// ANIM-ALL: shared level bytes (Debug prints the length only).
#[derive(Clone, Default)]
pub struct LevelBytes(pub std::sync::Arc<Vec<u8>>);
impl std::fmt::Debug for LevelBytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LevelBytes({})", self.0.len())
    }
}

/// The loaded level (SW in Sly 1 terms; the world struct at param_1 of FUN_0013da18).
#[derive(Resource, Clone, Debug, Default)]
pub struct World {
    /// ANIM-ALL: the decompressed level bytes the world was loaded from (the live image re-runs the original
    /// class loads, e.g. FUN_0012f2a8, over the same stream)
    pub level_data: LevelBytes,
    /// FUN_0013f810(0x3dd490, ..): object-manager properties
    pub props_3dd490: Vec<Prop>,
    /// FUN_0013f810(0x2ddf40, ..)
    pub props_2ddf40: Vec<Prop>,
    pub entries: Vec<EntryInfo>,
    pub objects: Vec<Lo>,
    /// the oid hash buckets world+0x38 (FUN_0015e380): every created object by +0x18, in link
    /// order (class-6 template records excluded: the original creates no object for them)
    by_oid: HashMap<i32, Vec<usize>>,
    /// WP-B5: FUN_0015e678 candidates for oids with alias objects (oid bucket ++ alias bucket)
    find: HashMap<i32, Vec<usize>>,
    /// stream offset after the last entry
    pub end_offset: u32,
    /// decompressed bytes never read
    pub tail_len: u32,
    /// WP-B4: the directory (world+0x3f08 entries; FUN_001c9b08 looks templates up by c)
    pub dir: Vec<DirEntry>,
    /// WP-B5: object list / activation state (loads_b5.rs)
    pub life: loads_b5::Lifecycle,
    /// WP-B5: FUN_001ca370 calls in load order
    pub template_instances: Vec<loads_b5::TemplateInstance>,
    /// WP-B5: entry+0x0c increments for entries not loaded yet (added when they load)
    pending_counts: HashMap<usize, i32>,
    /// WIRE-EP7: FUN_001c9b08 template-branch override props (FUN_0013f810 on the copy), per object, in
    /// stream order (later rows win over the copy's own props)
    pub override_props: HashMap<usize, Vec<Prop>>,
    /// POOL1 R95: objects the players' load bind tables Removed (0x200 rows / the oid-0x2ab sweep, vt +0x4c
    /// FUN_001a5630 -> LO +0x28 bit 56): out of the world at load (`game::fields::out_of_world_at_load`)
    pub bind_removed: std::collections::HashSet<usize>,
    /// ANIM-ALL R62: deferred template override rows (copy roots, oid, class filter, props)
    pending_ovr: Vec<(Vec<usize>, i32, i16, Vec<Prop>)>,
    /// WIRE-EP8: FUN_00256ee0 script slots bound by template override rows: object -> (entry, slot)
    pub script_slots: HashMap<usize, (usize, i16)>,
    /// HEIST: copy instance of a template copy (object -> k) and the entry each instance re-runs (k -> entry):
    /// each placement of an entry (FUN_001ca370) loads its own anchor list, so its slots get their own frames
    pub copy_inst: HashMap<usize, usize>,
    pub copy_inst_entry: Vec<usize>,
}

impl ObjectSink for World {
    fn objects(&self) -> &[Lo] {
        &self.objects
    }
    fn push(&mut self, lo: Lo) -> usize {
        let i = self.objects.len();
        self.by_oid.entry(lo.oid).or_default().push(i);
        let oid = lo.oid;
        let owner = lo.owner;
        if let Some(o) = owner {
            self.objects[o].children.push(i);
        }
        self.objects.push(lo);
        self.ee_sub_1a5270(i, owner); // WP-B5: vtable +0x3c list link
        if self.find.contains_key(&oid) {
            self.refresh_find(oid);
        }
        i
    }
    fn get_mut(&mut self, i: usize) -> &mut Lo {
        &mut self.objects[i]
    }
    // FUN_0015e958(0x805) / FUN_0015e938(0x405) mode 5: oid and alias buckets (the +0x3c scope
    // pre-check of FUN_0015e3e8 is not modelled)
    fn exists(&self, oid: i32) -> bool {
        !self.find_oid(oid).is_empty()
    }
    // WP-B4: FUN_001cb770 (entry with +0x10 == c) -> +0x60 (b)
    fn template_b(&self, c: i32) -> Option<i32> {
        self.dir.iter().find(|d| d.c == c).map(|d| d.b)
    }
    /// HEIST r27: FUN_001a6730 +0x3c: during the load (world+0x3f10 = entry index, not -2) the copy shares the
    /// source's alias node (`+0x3c = src+0x3c`); the copy owner's later vt+0xcc key push (FUN_001ca118 recursing to
    /// its attached objects, e.g. the brawl KO counter copies' b2000k) then gives it [k, src's own keys..], whose leaf
    /// names the copy under the copy owner's chain (bif 112 4112 from a counter copy finds its own tens digit).
    /// REPLACED: the kid clones are made after the owners' keys were pushed (see the kid_copies loop), so the
    /// result is built here: the copy's inherited (owner) chain + the keys `src` has beyond its own owner's chain,
    /// interned/claimed by FUN_001ca118. The runtime branch (-2: per-node duplicates naming the copy) is not modelled.
    fn clone_chain(&mut self, src: usize, dst: usize) {
        // 0x001a6730
        let own = &self.life.keys[src];
        let base = self.objects[src].owner.map(|o| self.life.keys[o].as_slice()).unwrap_or(&[]);
        let extra: Vec<u32> = if own.starts_with(base) { own[base.len()..].to_vec() } else { own.clone() };
        let mut path = self.life.keys[dst].clone();
        path.extend(extra);
        if path.is_empty() || path == self.life.keys[dst] {
            return;
        }
        let first = path.remove(0);
        self.life.keys[dst] = path;
        self.ee_sub_1ca118(dst, first, false);
    }
    // WP-B5
    fn template_index(&self, c: i32) -> Option<usize> {
        self.dir.iter().position(|d| d.c == c)
    }
    fn instantiate_template(&mut self, record: usize, entry: Option<usize>, owner: Option<usize>, bone: i16, oid: i32, xform: [u8; 48]) -> Vec<usize> {
        // The record is only bookkeeping here: FUN_001c9b08's template branch never calls
        // FUN_0013e3c8, so nothing with its oid enters the oid bucket (scripts reach the copy
        // through the alias bucket instead).
        let roid = self.objects[record].oid;
        if let Some(v) = self.by_oid.get_mut(&roid) {
            v.retain(|&k| k != record);
            if v.is_empty() {
                self.by_oid.remove(&roid);
            }
        }
        if self.find.contains_key(&roid) {
            self.refresh_find(roid);
        }
        let Some(e) = entry else {
            // FUN_001cb770 == -1: the original indexes entry -1 (heap garbage); nothing placed
            self.template_instances.push(loads_b5::TemplateInstance { record, entry, copy: -1, key: 0, owner, bone, oid, xform, objects: Vec::new(), nested: Vec::new(), overflow: false });
            return Vec::new();
        };
        let (copy, key, objects, overflow) = self.ee_sub_1ca370(e, owner, oid, xform);
        self.template_instances.push(loads_b5::TemplateInstance { record, entry, copy, key, owner, bone, oid, xform, objects: objects.clone(), nested: Vec::new(), overflow });
        objects
    }
    fn apply_override(&mut self, target: usize, props: &[Prop]) {
        self.override_props.entry(target).or_default().extend_from_slice(props);
    }
    fn defer_override(&mut self, roots: &[usize], oid: i32, class: i16, props: &[Prop]) -> bool {
        self.pending_ovr.push((roots.to_vec(), oid, class, props.to_vec()));
        true
    }
    fn bind_script_slot(&mut self, target: usize, entry: usize, slot: i16) {
        self.script_slots.insert(target, (entry, slot));
    }
    fn singleton_loaded(&mut self, i: usize, n: i8, entry: usize) {
        // FUN_0013fa30 (callback DAT_002cbe38): FUN_001a58d0(o, 0, 0) = +0x54 to the root, FUN_0013f990(o, kind),
        // then +0xcc(FUN_001c9940(DAT_003e13a0, -1), 1). DAT_003e13a0 (world +0x3f10) = the entry index when its
        // +0x5c (a) is 0, else -1 (FUN_0013da18); key 0xffffffff leaves the root path (PCSX2 emu_play_ep4/025:
        // 6791 par 0, kind 1, +0x3c [0x13ffff, root]; entry-189 88895 kind 1, +0x3c root).
        self.ee_sub_1a5900(i, None);
        self.ee_sub_13f990(i, if n == -2 { 1 } else { 0 });
        let now = self.dir.get(entry).is_some_and(|d| d.instantiate_now());
        if now {
            self.vt_cc(i, loads_b5::ee_sub_1c9940(entry as i32, -1), true);
        }
        // ANIM-ALL R49: a singleton is not in the entry collector, so FUN_0013da18's a == 0 +0x4c loop never
        // reaches it; its bit 56 comes from the ALO post-load (FUN_001f36a8 -> FUN_001a5d98 -> FUN_001a5d08 ->
        // vt +0x4c FUN_001a5630) running the deferred Remove of property field 26, for any entry (PCSX2 025_job:
        // 6791 / 88895 (a 1) carry field 26 -> bit 56 set; 2385 has none -> clear)
        if self.has_field_26(i) {
            self.ee_sub_1a5630(i);
        }
    }
    fn template_nested(&mut self, nested: &[usize], xform: [u8; 48]) {
        // FUN_001c9b08 after the nested list: +0xcc(key, 1), +0x12c(xform) per nested object
        let Some(t) = self.template_instances.last_mut() else { return };
        t.nested = nested.to_vec();
        let key = t.key;
        for &n in nested {
            self.vt_cc(n, key, true);
            self.apply_xform(n, xform);
        }
    }
    fn post_load(&mut self, i: usize) {
        self.ee_sub_1a5630(i);
    }
    fn after_load(&mut self, i: usize) {
        self.register_attached(i);
    }
}

impl World {
    /// ANIM-ALL R50: `i` (or the FUN_0013e510 source it was copied from, whose LO +0x34 deferred list the copy
    /// inherits) carries the property field-26 deferred Remove (FUN_001a5d58 node run by FUN_001a5d98)
    pub fn has_field_26(&self, i: usize) -> bool {
        let mut s = Some(i);
        while let Some(j) = s.filter(|&j| j < self.objects.len()) {
            if crate::engine::game::fields::object_props(&self.objects[j]).is_some_and(|p| p.iter().any(|p| p.field == 26)) {
                return true;
            }
            // ANIM-ALL R54: FUN_001c9b08 placement override rows (FUN_0013f810 on the copy, e.g. the guard parts)
            if std::env::var_os("SLY2_R54OFF").is_none() && self.override_props.get(&j).is_some_and(|p| p.iter().any(|p| p.field == 26)) {
                return true;
            }
            s = self.objects[j].clone_of.filter(|&c| c != j);
        }
        false
    }

    /// The objects FUN_0015e678 offers for `oid`: the oid bucket (+0x18 == oid, creation order,
    /// clones included) followed by the alias bucket (+0x10 of the key-tree nodes whose copy oid
    /// is `oid`, WP-B5), an alias object already offered with +0x18 == oid skipped. Class-6
    /// template records are not objects in the original and are never returned.
    pub fn find_oid(&self, oid: i32) -> &[usize] {
        self.find.get(&oid).or_else(|| self.by_oid.get(&oid)).map(Vec::as_slice).unwrap_or(&[])
    }

    /// Rebuilds the FUN_0015e678 candidate list of `oid` after its alias objects changed.
    pub(super) fn refresh_find(&mut self, oid: i32) {
        if oid == -1 {
            return; // FUN_0015e678 finds nothing for -1
        }
        let mut v = self.by_oid.get(&oid).cloned().unwrap_or_default();
        let n = v.len();
        for a in self.alias_objects(oid) {
            // 0x0015e7f8: skip when +0x18 == oid and already stored
            if self.objects[a].oid == oid && v[..n].contains(&a) {
                continue;
            }
            v.push(a);
        }
        if self.life.alias.bucket.contains_key(&oid) {
            self.find.insert(oid, v);
        } else {
            self.find.remove(&oid);
        }
    }
    /// Objects of exactly this class / of its subtree (FUN_0015e988).
    pub fn of_class(&self, cid: class::ClassId, subtree: bool) -> impl Iterator<Item = usize> + '_ {
        self.objects
            .iter()
            .enumerate()
            .filter(move |(_, o)| if subtree { class::ee_sub_15e988(o.class, cid) } else { o.class == cid })
            .map(|(i, _)| i)
    }
    /// Entry by directory name (the FUN_0014b690 name after "FK$X").
    pub fn entry_by_name(&self, name: &str) -> Option<&EntryInfo> {
        self.entries.iter().find(|e| e.dir.name() == name)
    }
    /// Stream-created objects per class (clones excluded).
    pub fn class_counts(&self) -> BTreeMap<class::ClassId, usize> {
        let mut m = BTreeMap::new();
        for o in self.objects.iter().filter(|o| o.clone_of.is_none()) {
            *m.entry(o.class).or_insert(0) += 1;
        }
        m
    }
    /// Classes whose payload was skipped because their load virtual is not ported.
    pub fn unported_classes(&self) -> BTreeMap<class::ClassId, usize> {
        let mut m = BTreeMap::new();
        for o in self.objects.iter().filter(|o| o.payload == Payload::Unported) {
            *m.entry(o.class).or_insert(0) += 1;
        }
        m
    }

    /// FUN_0013da18 from the first FUN_0013f810 to the end of the entry loop.
    pub fn load(level: &LoadedLevel, opt: LoadOptions) -> Result<World, WorldError> {
        let mut s = memory_stream(level.data.to_vec());
        s.read(level.header.objects_offset as usize, None);
        let mut w = World::default();
        w.level_data = LevelBytes(level.data.clone());
        w.load_stream(&mut s, &level.header.directory, opt)?;
        w.tail_len = (level.data.len() as u32).saturating_sub(w.end_offset);
        Ok(w)
    }

    pub fn load_stream(&mut self, s: &mut BinaryInputStream, dir: &[DirEntry], opt: LoadOptions) -> Result<(), WorldError> {
        // 0x0013da18 (continued)
        self.dir = dir.to_vec(); // WP-B4
        let pe = |e: props::PropError| WorldError::BadField { field: e.field, offset: e.offset };
        self.props_3dd490 = ee_sub_13f810(s).map_err(pe)?;
        self.props_2ddf40 = ee_sub_13f810(s).map_err(pe)?;
        s.align(0x10);
        // FUN_001ff7e8 / FUN_0020e3c0 / FUN_0020f1c8 / heaps: no stream data.
        // HEIST: (template, copy) pairs whose FUN_001f66f0 child clones are made after the entry loop
        let mut kid_copies: Vec<(usize, usize)> = Vec::new();
        for (i, d) in dir.iter().enumerate() {
            let start = s.tell();
            let sections = entry::read_entry_sections(s, opt.mission);
            if s.at_eof() {
                return Err(WorldError::Eof { what: "entry sections", offset: start });
            }
            let objects = ee_sub_13fb40(self, None, i, s, opt.mission)?;
            s.align(0x10);
            // copies 2..n (FUN_0013e510)
            let mut instances = objects.clone();
            for _ in 1..d.n.max(1) {
                for &o in &objects {
                    let c = ee_sub_13e510(self, o);
                    kid_copies.push((o, c));
                    instances.push(c);
                }
            }
            // a == 0: vtable +0x4c post-load per instance (WP-B5: FUN_001a5630);
            // else entry+0x0c load count++ (on top of FUN_001ca370 calls made before this
            // entry loaded, which already advanced +0x0c).
            let pending = self.pending_counts.remove(&i).unwrap_or(0);
            if d.instantiate_now() {
                for &o in &instances {
                    self.ee_sub_1a5630(o);
                }
            }
            let load_count = pending + if d.instantiate_now() { 0 } else { 1 };
            let inst_oids = vec![-1; d.n.max(0) as usize];
            self.entries.push(EntryInfo { dir: d.clone(), sections, objects, instances, load_count, inst_oids, start_offset: start, end_offset: s.tell() });
        }
        // HEIST: FUN_0013e510 vt +0x144 (FUN_001f66f0) clones each copy's children. REPLACED: made here,
        // after every entry loaded, instead of inside the copy loop, so the stream objects keep their
        // indices (pinned by tests / lanes); owners, oids and clone_of are the original's.
        // perf (hub, wtest, 600 f): 5376 -> 9805 objects, set.frame +25..35%, load +2.7 s.
        // CRAWL R35: the three tests that broke with it ON (ep1a_paris_loot_system, wired_cast_wall_sneak_live,
        // wired_guards_spot_sly_live) pass with it ON now.
        // ON by default (EMU5 RAM dump: 4 class-37 holders); SLY2_CLONE_KIDS=0 switches it off.
        let clone_kids = std::env::var("SLY2_CLONE_KIDS").map_or(true, |v| v != "0");
        for (src, c) in kid_copies.into_iter().filter(|_| clone_kids) {
            objects::ee_sub_1f66f0_kids(self, src, c);
        }
        self.ee_sub_1ca500();
        // ANIM-ALL R50: the post-load of every object (FUN_001a5c48 / FUN_001f36a8 -> FUN_001a5d98 -> FUN_001a5d08
        // -> vt +0x4c FUN_001a5630) runs its deferred Remove of property field 26 (a FUN_0013e510 copy inherits
        // its source's +0x34 list). `SLY2_R50OFF` = singletons only (R49).
        if std::env::var_os("SLY2_R50OFF").is_none() {
            for i in 0..self.objects.len() {
                if self.has_field_26(i) {
                    self.ee_sub_1a5630(i);
                }
            }
        }
        // ANIM-ALL R51: the players' load binds (slot 13 / slot 14 FUN_001a64e8 tables, FUN_001400b8's 0x2bc9f0 table
        // and FUN_0015e650 oid-0x2ab sweep): a 0x200 row runs vt +0x4c on the bound object (PCSX2 025_job: ~95% of
        // the JT / Bentley / Murray class-109 ASEGs have bit 56). Same rows as playerdraw::vis::JtVis::load(_po).
        if std::env::var_os("SLY2_R51OFF").is_none() {
            use crate::engine::playerdraw::{self as pd, vis};
            for i in 0..self.objects.len() {
                let tables: Vec<&[(u32, i32, u32)]> = match self.objects[i].class {
                    pd::CLASS_JT => vec![pd::JT_TABLE.as_slice(), vis::JT_BIND_2BC9F0.as_slice(), vis::JT_BIND_2BF1B0.as_slice()],
                    crate::engine::player::p20::CLASS_BENTLEY => vec![vis::PO_BIND_2BC390.as_slice(), vis::JT_BIND_2BC9F0.as_slice()],
                    crate::engine::player::p20::CLASS_MURRAY => vec![vis::PO_BIND_2C0270.as_slice(), vis::JT_BIND_2BC9F0.as_slice()],
                    _ => continue,
                };
                for t in tables {
                    for &(flags, oid, _) in t {
                        if flags & 0x200 != 0 && flags & 0x70 == 0 {
                            if let Some(o) = pd::ee_sub_15e938_below(self, i, oid) {
                                self.ee_sub_1a5630(o);
                                self.bind_removed.insert(o);
                            }
                        }
                    }
                }
                let all: Vec<usize> = pd::subtree(self, i).into_iter().filter(|&o| self.objects[o].oid == vis::JT_REMOVE_ALL_OID).collect();
                for o in all {
                    self.ee_sub_1a5630(o);
                    self.bind_removed.insert(o);
                }
            }
        }
        // ANIM-ALL R55: the other post-loads' FUN_001a64e8 bind tables with 0x200 rows (vt +0x4c on the bound
        // object): class 99 (C2d5450) FUN_001c2d20 -> 0x2c0cb0 (4 rows, oids 0x2b2..0x2b5; PCSX2 000: 690 / 693
        // under 12787 and the top-level 691 / 692 set). FUN_0015e938 mode: flag 2 -> 4, flag 1 -> 5, else 1.
        if std::env::var_os("SLY2_R55OFF").is_none() {
            // + K8 pickups 91 / 92 / 93 / 95 load FUN_001c4ef8 (after FUN_001f31e0) -> 0x2c0d40 (1 row, oid 0x182)
            let r99 = crate::engine::disc::exe::rows3(0x002c_0cb0, 4);
            let rk8 = crate::engine::disc::exe::rows3(0x002c_0d40, 1);
            for i in 0..self.objects.len() {
                let rows = match self.objects[i].class {
                    99 => &r99,
                    91 | 92 | 93 | 95 => &rk8,
                    _ => continue,
                };
                for &(flags, oid, _) in rows {
                    if flags & 0x200 == 0 || flags & 0x70 != 0 {
                        continue;
                    }
                    let mode = if flags & 2 != 0 { 4 } else if flags & 1 != 0 { 5 } else { 1 } | (flags & 0x400);
                    let o = crate::engine::game::methods::find::ee_sub_15e958(self, mode, oid, i as u32) as usize;
                    if o != 0 {
                        self.ee_sub_1a5630(o);
                    }
                }
            }
        }
        // ANIM-ALL R62: deferred template override rows: the oid inside the copy's (now complete) subtree, else
        // the first world object with it (the pre-R62 rule)
        for (roots, oid, class, props) in std::mem::take(&mut self.pending_ovr) {
            let mut st = roots.clone();
            let mut t = None;
            while let Some(i) = st.pop() {
                if self.objects[i].oid == oid {
                    t = Some(i);
                    break;
                }
                st.extend(self.objects[i].children.iter().copied());
            }
            let t = t.or_else(|| self.objects.iter().position(|x| x.oid == oid));
            if let Some(t) = t.filter(|&i| class >= 0 && class::ee_sub_15e988(self.objects[i].class, class as class::ClassId)) {
                self.apply_override(t, &props);
            }
        }
        self.build_copy_inst();
        self.end_offset = s.tell();
        if s.at_eof() {
            return Err(WorldError::Eof { what: "entry loop", offset: self.end_offset });
        }
        Ok(())
    }
}

impl World {
    /// ANIM-ALL R36: FUN_001ca500 (from FUN_0020e740, the per-level init after the load) key part: every copy not
    /// handed out yet (entry+0xc load count .. n) gets vt+0xcc(FUN_001c9940(entry, copy), 1) on its m objects, so
    /// e.g. an a == 0 entry's stream objects (copy 0) carry key (entry, 0) (emu 025: oid 60283 0x6b4780 +0x3c =
    /// alias 0x1e83a30 key 0x250000). Not modelled: alias +0x24 = 1, the per-copy matrix arrays (entry+0x64/0x68).
    fn ee_sub_1ca500(&mut self) {
        // 0x001ca500
        for e in 0..self.entries.len() {
            let (m, n, lc) = (self.entries[e].objects.len(), self.entries[e].dir.n, self.entries[e].load_count);
            for copy in lc.max(0)..n {
                let key = loads_b5::ee_sub_1c9940(e as i32, copy);
                for j in 0..m {
                    if let Some(&o) = self.entries[e].instances.get(copy as usize * m + j) {
                        self.vt_cc(o, key, true);
                    }
                }
            }
        }
    }

    /// HEIST: template-copy instances (see `copy_inst`). A copy root = a copy (clone_of, parent slot -1, no
    /// FUN_00256ee0 slot row) whose owner is not a copy; it and its copied subtree share instance k, which
    /// re-runs its template's entry block in fresh frames. SLY2_COPY_FRAMES=0: copies share the template's.
    fn build_copy_inst(&mut self) {
        self.copy_inst.clear();
        self.copy_inst_entry.clear();
        if std::env::var("SLY2_COPY_FRAMES").is_ok_and(|v| v == "0") {
            return;
        }
        let is_copy = |w: &World, o: usize| w.objects[o].clone_of.is_some() && w.objects[o].parent_slot < 0 && !w.script_slots.contains_key(&o);
        let mut root_k: HashMap<usize, usize> = HashMap::new();
        for o in 0..self.objects.len() {
            if !is_copy(self, o) {
                continue;
            }
            let mut r = o;
            for _ in 0..64 {
                match self.objects[r].owner {
                    Some(p) if is_copy(self, p) => r = p,
                    _ => break,
                }
            }
            let Some(t) = self.objects[r].clone_of else { continue };
            let (te, ts) = (self.objects[t].entry, self.objects[t].parent_slot);
            // only instances of an entry with script slots; a kid templated from another entry keeps the shared frame
            let Some(to) = self.objects[o].clone_of else { continue };
            if ts < 0 || self.objects[to].entry != te {
                continue;
            }
            let k = *root_k.entry(r).or_insert_with(|| {
                self.copy_inst_entry.push(te);
                self.copy_inst_entry.len() - 1
            });
            self.copy_inst.insert(o, k);
        }
    }
}

#[cfg(feature = "game")]
pub use game::*;

#[cfg(feature = "game")]
mod game {
    use super::{LoadOptions, World};
    use crate::engine::disc::LevelLoadResult;
    use bevy::prelude::{Commands, Res, Resource};

    /// Set when a finished level failed to build a world (explicit, never silently ignored).
    #[derive(Resource, Debug)]
    pub struct WorldLoadError(pub String);

    /// Builds the [`World`] Resource from a finished [`LevelLoadResult`] (WP-A).
    pub fn build_world(mut commands: Commands, res: Option<Res<LevelLoadResult>>, opt: Option<Res<WorldOptions>>) {
        let Some(res) = res else { return };
        let opt = opt.map(|o| o.0).unwrap_or_default();
        match &res.0 {
            Ok(level) => match World::load(level, opt) {
                Ok(w) => commands.insert_resource(w),
                Err(e) => commands.insert_resource(WorldLoadError(e.to_string())),
            },
            Err(e) => commands.insert_resource(WorldLoadError(e.to_string())),
        }
        commands.remove_resource::<LevelLoadResult>();
    }

    #[derive(Resource, Clone, Copy, Default)]
    pub struct WorldOptions(pub LoadOptions);
}
