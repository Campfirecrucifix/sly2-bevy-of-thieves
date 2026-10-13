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

/// The loaded level (SW in Sly 1 terms; the world struct at param_1 of FUN_0013da18).
#[derive(Resource, Clone, Debug, Default)]
pub struct World {
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
    fn template_nested(&mut self, nested: &[usize], xform: [u8; 48]) {
        // FUN_001c9b08 after the nested list: +0xcc(key, 1), +0x12c(xform) per nested object
        let Some(t) = self.template_instances.last_mut() else { return };
        t.nested = nested.to_vec();
        let key = t.key;
        for &n in nested {
            self.ee_sub_1ca118(n, key, true);
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
                    instances.push(ee_sub_13e510(self, o));
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
        self.end_offset = s.tell();
        if s.at_eof() {
            return Err(WorldError::Eof { what: "entry loop", offset: self.end_offset });
        }
        Ok(())
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
