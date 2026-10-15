//! WP-B5: template instantiation (FUN_001ca370, called by the class-6 handler FUN_001c9b08)
//! and the LO list/activation lifecycle used right after a level loads:
//!   FUN_001a5270 (LO init, vtable +0x3c: link into the parent's object list),
//!   FUN_001a5630 (LO post-load, vtable +0x4c: activate), FUN_001a5478 (LO remove, +0x44),
//!   FUN_001a5900 (LO set parent, +0x54), FUN_001ca118 (template key chain, +0xcc),
//!   +0xe4 "attached objects" (LO 0x001a5be8 = none, class 15 0x00221120, class 31 0x0015c310).
//! Every class the loader creates (2..133) uses these LO base virtuals for slots +0x3c..+0x54
//! and +0xcc (checked against db class_slots; see ENGINE_PORT.md Findings B5).
//!
//! Lists are the engine's DLIST (FUN_0011a220 append, FUN_0011a280 insert before, FUN_0011a2e8
//! remove) plus the "first active" pointer: parent+0x70 for parent+0x64, DAT_003dd4ac for the
//! root list DAT_003dd4a0 (FUN_001a5448). Inactive objects sit before that pointer, active ones
//! from it to the end.

use super::class::{self, ClassId};
use super::loads_b4::B4Load;
use super::objects::{LoadData, Payload};
use super::World;
use std::collections::HashMap;

/// DLIST + first-active pointer.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ObjList {
    pub items: Vec<usize>,
    pub head: Option<usize>,
}

impl ObjList {
    fn pos(&self, x: usize) -> Option<usize> {
        self.items.iter().position(|&v| v == x)
    }
    /// FUN_0011a2e8. Removing an unlinked node would clear the list's first/last in the
    /// original; every object is linked by FUN_001a5270 at creation, so that never happens.
    fn remove(&mut self, x: usize) {
        if let Some(p) = self.pos(x) {
            self.items.remove(p);
        }
    }
    /// FUN_0011a220
    fn append(&mut self, x: usize) {
        self.items.push(x);
    }
    /// FUN_0011a280: None = append, else insert before `before`.
    fn insert_before(&mut self, before: Option<usize>, x: usize) {
        match before.and_then(|b| self.pos(b)) {
            Some(p) => self.items.insert(p, x),
            None => self.append(x),
        }
    }
    /// node +0x24 (next in this list)
    fn next(&self, x: usize) -> Option<usize> {
        self.pos(x).and_then(|p| self.items.get(p + 1).copied())
    }
    /// Objects from the first-active pointer to the end.
    pub fn active(&self) -> &[usize] {
        match self.head.and_then(|h| self.pos(h)) {
            Some(p) => &self.items[p..],
            None => &[],
        }
    }
}

/// Per-object lifecycle state (fields of the LO struct the loader touches after creation).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Lifecycle {
    /// +0x28 bit 56: in the world (set by FUN_001a5630, cleared by FUN_001a5478)
    pub active: Vec<bool>,
    /// +0x28 bit 60: created with no parent (FUN_001a5270), copied from the parent otherwise
    pub root_flag: Vec<bool>,
    /// +0x28 bit 57: attached to a class-15/31 object (set by FUN_00220fd0 / FUN_0015c1e8)
    pub attached_flag: Vec<bool>,
    /// +0x3c: template key chain (FUN_001ca118), newest key first; inherited at init
    pub keys: Vec<Vec<u32>>,
    /// +0x3c as the alias node it points at (index into `alias.nodes`; 0 = root DAT_003e139c)
    pub alias_leaf: Vec<usize>,
    /// +0x38: init flags (set to 0x501502f9 by FUN_001a5270)
    pub lo_flags: Vec<u32>,
    /// +0x40: LO mode byte (set to 0x21 by FUN_001a5270)
    pub lo_mode: Vec<u32>,
    /// FUN_001c99d8 key tree + the world+0x1838 alias buckets
    pub alias: AliasTable,
    /// parent+0x64 list (+0x70 first active) by parent index
    pub lists: HashMap<usize, ObjList>,
    /// DAT_003dd4a0 / DAT_003dd4ac (no parent)
    pub root_list: ObjList,
    /// +0xe4 attached objects of class 15 (+0x10c) / class 31 (+0xe0/+0xe4)
    pub attached: HashMap<usize, Vec<usize>>,
    /// +0x12c applied transforms (template placement matrix, FUN_00117f18 layout)
    pub placed: HashMap<usize, [u8; 48]>,
}

/// FUN_001c99d8 node (0x28 bytes from FUN_001ae2a0): +0 key, +4 parent, +8 oid
/// (FUN_001c9968(key)), +0xc next in the world+0x1838 bucket, +0x10 object, +0x14 next sibling,
/// +0x18 children (DLIST, link at +0x14).
#[derive(Clone, Debug, PartialEq)]
pub struct AliasNode {
    pub key: u32,
    /// +4 (None = the root DAT_003e139c itself)
    pub parent: Option<usize>,
    pub oid: i32,
    /// +0x10: the object this key path names (set by FUN_001ca118)
    pub object: Option<usize>,
    /// +0x18 child list in link order
    pub children: Vec<usize>,
}

/// The key tree under DAT_003e139c (node 0, key -1) and the alias buckets world+0x1838
/// (FUN_0015e3c0: oid * 0x95675 & 0x1ff; replaced by a map oid -> nodes in link order).
#[derive(Clone, Debug, PartialEq)]
pub struct AliasTable {
    pub nodes: Vec<AliasNode>,
    pub bucket: HashMap<i32, Vec<usize>>,
}

impl Default for AliasTable {
    fn default() -> Self {
        Self { nodes: vec![AliasNode { key: u32::MAX, parent: None, oid: -1, object: None, children: Vec::new() }], bucket: HashMap::new() }
    }
}

/// FUN_001c9950 / FUN_001c9958: entry index / copy of a key.
pub fn ee_sub_1c9950(key: u32) -> i32 {
    // 0x001c9950
    (key as i32) >> 16
}
pub fn ee_sub_1c9958(key: u32) -> i16 {
    // 0x001c9958
    key as u16 as i16
}

/// One FUN_001ca370 call (template branch of FUN_001c9b08).
#[derive(Clone, Debug, PartialEq)]
pub struct TemplateInstance {
    /// the class-6 record (kept as a bookkeeping `Lo`; the original creates no object for it)
    pub record: usize,
    /// directory entry index found by FUN_001cb770 (None = -1, template not in the directory)
    pub entry: Option<usize>,
    /// entry+0x0c before the call = which of the n copies is used
    pub copy: i32,
    /// FUN_001c9940(entry, copy) = entry << 16 | copy & 0xffff
    pub key: u32,
    pub owner: Option<usize>,
    pub bone: i16,
    pub oid: i32,
    pub xform: [u8; 48],
    /// the copy's m objects (pushed to the collector)
    pub objects: Vec<usize>,
    /// objects of the record's nested FUN_0013fb40 list
    pub nested: Vec<usize>,
    /// copy * m + m > entry instances (the original reads past the entry's array)
    pub overflow: bool,
}

/// FUN_001c9940
pub fn ee_sub_1c9940(entry: i32, copy: i32) -> u32 {
    // 0x001c9940
    ((entry as u32) << 16) | (copy as u32 & 0xffff)
}

/// Per-class +0xe4 slot (attached objects).
fn attached_of(w: &World, i: usize) -> Vec<usize> {
    w.life.attached.get(&i).cloned().unwrap_or_default()
}

impl World {
    fn list_mut(&mut self, parent: Option<usize>) -> &mut ObjList {
        match parent {
            Some(p) => self.life.lists.entry(p).or_default(),
            None => &mut self.life.root_list,
        }
    }

    /// FUN_001c9968: the oid of template copy `key` = *(entry+0x18)[copy] (-1 for copy -1).
    /// An entry not loaded yet has no +0x18 array in the original (the read is garbage); -1 here.
    pub fn ee_sub_1c9968(&self, key: u32) -> i32 {
        // 0x001c9968
        let copy = ee_sub_1c9958(key);
        if copy == -1 {
            return -1;
        }
        let e = ee_sub_1c9950(key);
        self.entries.get(e as usize).and_then(|e| e.inst_oids.get(copy as usize)).copied().unwrap_or(-1)
    }

    /// FUN_001c99d8(node, n, keys): walks/extends the key tree from `node` along `keys`; new
    /// nodes get oid FUN_001c9968(key) and are appended to that oid's alias bucket (FUN_0011a070)
    /// and to their parent's child list. Returns the last node (0 when `keys` is empty: the
    /// original returns NULL there, no caller passes n == 0).
    pub fn ee_sub_1c99d8(&mut self, node: usize, keys: &[u32]) -> usize {
        // 0x001c99d8
        let mut cur = node;
        let mut rest = keys;
        while let Some((&k, tail)) = rest.split_first() {
            let t = &self.life.alias;
            match t.nodes[cur].children.iter().copied().find(|&c| t.nodes[c].key == k) {
                Some(c) => {
                    cur = c;
                    rest = tail;
                }
                None => break,
            }
        }
        for &k in rest {
            let oid = self.ee_sub_1c9968(k);
            let t = &mut self.life.alias;
            let n = t.nodes.len();
            t.nodes.push(AliasNode { key: k, parent: Some(cur), oid, object: None, children: Vec::new() });
            t.bucket.entry(oid).or_default().push(n);
            t.nodes[cur].children.push(n);
            cur = n;
        }
        cur
    }

    /// Alias-bucket objects for `oid` (FUN_0015e678 second loop): node +0x10 of every node with
    /// +8 == oid, in bucket order; nodes without an object are skipped.
    pub fn alias_objects(&self, oid: i32) -> Vec<usize> {
        let t = &self.life.alias;
        t.bucket.get(&oid).map(|v| v.iter().filter_map(|&n| t.nodes[n].object).collect()).unwrap_or_default()
    }

    /// The parent+0x64 list (or the root list) as built by init/post-load.
    pub fn obj_list(&self, parent: Option<usize>) -> Option<&ObjList> {
        match parent {
            Some(p) => self.life.lists.get(&p),
            None => Some(&self.life.root_list),
        }
    }

    pub fn is_active(&self, i: usize) -> bool {
        self.life.active.get(i).copied().unwrap_or(false)
    }

    /// FUN_001a5270 (vtable +0x3c) part that the world model keeps: flags, key chain, and the
    /// link into the parent's list before its first-active pointer. Called by `push`.
    pub(super) fn ee_sub_1a5270(&mut self, i: usize, owner: Option<usize>) {
        // 0x001a5270
        let l = &mut self.life;
        l.active.resize(i + 1, false);
        l.attached_flag.resize(i + 1, false);
        let (root, keys, leaf) = match owner {
            None => (true, Vec::new(), 0), // FUN_001a53a0; +0x3c = DAT_003e139c (empty chain)
            Some(o) => (l.root_flag[o], l.keys[o].clone(), l.alias_leaf[o]),
        };
        l.alias_leaf.resize(i + 1, 0);
        l.alias_leaf[i] = leaf;
        l.lo_flags.resize(i + 1, 0);
        l.lo_flags[i] = 0x5015_02f9;
        l.lo_mode.resize(i + 1, 0);
        l.lo_mode[i] = 0x21;
        l.root_flag.resize(i + 1, false);
        l.root_flag[i] = root;
        l.keys.resize(i + 1, Vec::new());
        l.keys[i] = keys;
        let list = self.list_mut(owner);
        let head = list.head;
        list.insert_before(head, i);
    }

    /// FUN_001a5630 (vtable +0x4c): activate after load. FUN_001a61a0 root-set test ported
    /// as owner==None; vtable +0xb4/+0x64 and FUN_00147e20 runtime hooks not ported.
    pub fn ee_sub_1a5630(&mut self, i: usize) {
        // 0x001a5630
        if self.is_active(i) {
            return;
        }
        // FUN_001a61a0 root-set test: root objects with no parent go through FUN_00147e20
        // (manager pending list drop, not modelled)
        let owner = self.objects[i].owner;
        let list = self.list_mut(owner);
        list.remove(i);
        list.append(i);
        if list.head.is_none() {
            list.head = Some(i);
        }
        self.life.active[i] = true;
        for a in attached_of(self, i) {
            self.ee_sub_1a5630(a);
        }
        // root_flag -> vtable +0x64: runtime activation hook (not ported)
        if self.life.root_flag[i] {
            // vtable +0x64 would trigger here
        }
    }

    /// FUN_001a5478 (vtable +0x44): remove from the world (stays linked, moved before the
    /// first-active pointer). vtable +0x5c deactivation hook and FUN_001a61a0 root-set
    /// test ported as owner==None; FUN_00148300 / vtable +0xbc runtime hooks not ported.
    pub fn ee_sub_1a5478(&mut self, i: usize) {
        // 0x001a5478
        if !self.is_active(i) {
            return;
        }
        self.life.active[i] = false;
        let owner = self.objects[i].owner;
        let list = self.list_mut(owner);
        if list.head == Some(i) {
            list.head = list.next(i);
        } else {
            list.remove(i);
            let head = list.head;
            list.insert_before(head, i);
        }
        // vtable +0x5c deactivation: for root objects (no parent) that are NOT in the
        // root set (FUN_001a61a0 == 0), drop from manager pending list (FUN_00147e88)
        if owner.is_none() {
            // FUN_00147e88(0x3dd490, +0x14): not modelled (manager pending list)
        }
        for a in attached_of(self, i) {
            self.ee_sub_1a5478(a);
        }
        // FUN_001a61a0 != 0 -> vtable +0xbc: runtime hook (not ported)
    }

    /// FUN_001a5900 (vtable +0x54): set parent. Moves the object between lists (an active
    /// object is appended after the new first-active pointer, an inactive one before it),
    /// sets +0x10 and recurses into the attached objects with the same arguments.
    /// FUN_001a61a0 root-set test ported as owner==None; vtable +0x6c/+0x5c/+0x64/+0xbc
    /// runtime hooks not ported. Also keeps `Lo::children` in step with +0x10.
    pub fn ee_sub_1a5900(&mut self, i: usize, parent: Option<usize>) {
        // 0x001a5900
        let old = self.objects[i].owner;
        let was_active = self.is_active(i);
        // Determine new root flag from parent (or true if root)
        let new_root = match parent {
            None => true,
            Some(p) => {
                if self.life.root_flag.len() > p {
                    self.life.root_flag[p]
                } else {
                    false
                }
            }
        };
        let old_root = if self.life.root_flag.len() > i { self.life.root_flag[i] } else { false };
        let ol = self.list_mut(old);
        if !was_active {
            ol.remove(i);
            let nl = self.list_mut(parent);
            let head = nl.head;
            nl.insert_before(head, i);
        } else {
            if ol.head == Some(i) {
                ol.head = ol.next(i);
            }
            ol.remove(i);
            let nl = self.list_mut(parent);
            nl.append(i);
            if nl.head.is_none() {
                nl.head = Some(i);
            }
        }
        // FUN_001a61a0 root-set management (vtable +0xb4 detach from old root)
        if let Some(o) = old {
            self.objects[o].children.retain(|&c| c != i);
        }
        self.objects[i].owner = parent;
        if let Some(p) = parent {
            self.objects[p].children.push(i);
        }
        // vtable +0x6c: re-parent notification (not ported)
        // root-flag visibility toggle (vtable +0x64 / +0x5c)
        if self.life.root_flag.len() > i {
            self.life.root_flag[i] = new_root;
        }
        if new_root != old_root && was_active {
            // would call vtable +0x64 (activate) or +0x5c (deactivate)
        }
        for a in attached_of(self, i) {
            self.ee_sub_1a5900(a, parent);
        }
        // FUN_001a61a0 != 0 -> vtable +0xbc (runtime hook, not ported)
        // FUN_00148300 for root with no parent when newly activated
    }

    /// FUN_001ca118 (vtable +0xcc): push `key` onto the object's template key chain and,
    /// with `recurse`, onto its attached objects'. The new chain [key, old chain root..leaf] is
    /// interned by FUN_001c99d8 (alias nodes); the old chain's nodes naming this object are
    /// released (from the leaf up while +0x10 == obj) and the new chain's nodes take it (from
    /// the leaf up while the key's entry has exactly one object (entry+0) and +0x10 is unset).
    pub fn ee_sub_1ca118(&mut self, i: usize, key: u32, recurse: bool) {
        // 0x001ca118
        self.life.keys[i].insert(0, key);
        self.life.keys[i].truncate(16); // the chain is copied into a 16-entry buffer
        let path = self.life.keys[i].clone();
        let leaf = self.ee_sub_1c99d8(0, &path);
        let t = &mut self.life.alias;
        let mut touched = Vec::new();
        let mut n = self.life.alias_leaf[i];
        while t.nodes[n].key != u32::MAX && t.nodes[n].object == Some(i) {
            t.nodes[n].object = None;
            touched.push(t.nodes[n].oid);
            n = t.nodes[n].parent.unwrap_or(0);
        }
        let mut n = leaf;
        while t.nodes[n].key != u32::MAX && t.nodes[n].object.is_none() {
            let e = ee_sub_1c9950(t.nodes[n].key);
            // *(DAT_003e1398 + entry * 0x6c) == 1 (entry+0 = m)
            if self.entries.get(e as usize).map(|e| e.objects.len()) != Some(1) {
                break;
            }
            t.nodes[n].object = Some(i);
            touched.push(t.nodes[n].oid);
            n = t.nodes[n].parent.unwrap_or(0);
        }
        self.life.alias_leaf[i] = leaf;
        for oid in touched {
            self.refresh_find(oid);
        }
        if recurse {
            for a in attached_of(self, i) {
                self.ee_sub_1ca118(a, key, true);
            }
        }
    }

    /// vtable +0x12c (LO 0x001a6450 forwards to the attached objects; class-3 overrides store
    /// the matrix): record the placement matrix per object.
    pub fn apply_xform(&mut self, i: usize, xform: [u8; 48]) {
        self.life.placed.insert(i, xform);
        for a in attached_of(self, i) {
            self.apply_xform(a, xform);
        }
    }

    /// FUN_001ca370(world, entry, owner, bone, oid, xform, collector): use the next prepared
    /// copy of template entry `entry` (entry+0x0c, starts at 1 after a template entry loads, so
    /// copy 0 = the stream objects is never handed out). Returns the m objects of that copy.
    pub fn ee_sub_1ca370(&mut self, entry: usize, owner: Option<usize>, oid: i32, xform: [u8; 48]) -> (i32, u32, Vec<usize>, bool) {
        // 0x001ca370
        // An entry not loaded yet (same or later directory position) still has m == 0 and no
        // instance array: nothing is placed but the counter still advances.
        let (copy, objs, overflow) = match self.entries.get(entry) {
            Some(e) => {
                let copy = e.load_count;
                let m = e.objects.len();
                let lo = copy.max(0) as usize * m;
                let objs: Vec<usize> = e.instances.iter().skip(lo).take(m).copied().collect();
                (copy, objs, lo + m > e.instances.len())
            }
            None => (self.pending_load_count(entry), Vec::new(), false),
        };
        let key = ee_sub_1c9940(entry as i32, copy);
        // *(entry+0x18)[copy] = oid (out of bounds for copy >= n in the original)
        if let Some(e) = self.entries.get_mut(entry) {
            if let Some(slot) = e.inst_oids.get_mut(copy as usize) {
                *slot = oid;
            }
        }
        for &o in &objs {
            self.ee_sub_1ca118(o, key, true); // +0xcc
            self.ee_sub_1a5478(o); // +0x44
            self.ee_sub_1a5900(o, owner); // +0x54(owner, bone, 0)
            self.apply_xform(o, xform); // +0x12c
        }
        match self.entries.get_mut(entry) {
            Some(e) => e.load_count = copy + 1,
            None => self.bump_pending(entry),
        }
        (copy, key, objs, overflow)
    }

    fn pending_load_count(&self, entry: usize) -> i32 {
        self.pending_counts.get(&entry).copied().unwrap_or(0)
    }
    fn bump_pending(&mut self, entry: usize) {
        *self.pending_counts.entry(entry).or_insert(0) += 1;
    }

    /// Class-15/31 attached objects (+0xe4), registered after their nested list loads.
    /// FUN_00220fd0: exactly one nested object -> +0x10c; FUN_0015c1e8: nested objects of
    /// class 30's subtree go to +0xe4, the others are post-loaded at once (+0x4c).
    pub(super) fn register_attached(&mut self, i: usize) {
        let Payload::Loaded(LoadData::B4(b)) = &self.objects[i].payload else { return };
        let list: Vec<usize> = match &**b {
            B4Load::L220fd0 { nested, .. } => {
                if nested.len() == 1 {
                    nested.clone()
                } else {
                    Vec::new()
                }
            }
            B4Load::L15c1e8 { nested, .. } => {
                let nested = nested.clone();
                for &n in &nested {
                    self.life.attached_flag[n] = true;
                }
                let (att, now): (Vec<usize>, Vec<usize>) = nested.into_iter().partition(|&n| class::ee_sub_15e988(self.objects[n].class, 30 as ClassId));
                for n in now {
                    self.ee_sub_1a5630(n);
                }
                att
            }
            _ => return,
        };
        for &n in &list {
            self.life.attached_flag[n] = true;
        }
        if !list.is_empty() {
            self.life.attached.insert(i, list);
        }
    }
}

/// Live objects after load: active (+0x28 bit 56). Class-6 template-branch record Lo's are
/// never activated (they do not exist in the original).
pub fn live_objects(w: &World) -> impl Iterator<Item = usize> + '_ {
    (0..w.objects.len()).filter(move |&i| w.is_active(i))
}
