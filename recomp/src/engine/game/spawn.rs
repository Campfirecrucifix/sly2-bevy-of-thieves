//! Spawn points after a level transition: FUN_00227fe8 (find the class-0x2a spawn object by oid, also through
//! its alias list +0x16c/+0x170) and FUN_00228988 (append an alias), plus the post-load activation set.
//!
//! The alias list is filled at run time by Splice CMethods 0x0010b8f8 (alias 0x189), 0x0010b970 (alias 0x18a)
//! and 0x0010b9e8 (alias = script argument), all of which call FUN_00228988 on the class-0x2a object; there is
//! no stream data for it. `SpawnAliases` is that per-object list (keyed by world object index).

use crate::engine::world::World;
use std::collections::HashMap;

/// Class id of the spawn-point object (C2dc550, load FUN_00228720).
pub const CLASS_SPAWN: u16 = 0x2a;
/// FUN_00228988: at most 4 aliases (+0x170..+0x17c).
pub const MAX_ALIASES: usize = 4;
/// aiStack_c0[32] in FUN_00227fe8 (FUN_0015e9c8 max 0x20).
pub const MAX_CANDIDATES: usize = 0x20;

/// +0x16c count / +0x170 oids, per spawn object.
#[derive(Clone, Debug, Default)]
pub struct SpawnAliases(pub HashMap<usize, Vec<i32>>);

impl SpawnAliases {
    /// FUN_00228988: append `oid` unless the list already has 4 entries (silently ignored).
    pub fn ee_sub_228988(&mut self, obj: usize, oid: i32) {
        // 0x00228988
        let l = self.0.entry(obj).or_default();
        if l.len() < MAX_ALIASES {
            l.push(oid);
        }
    }
    pub fn get(&self, obj: usize) -> &[i32] {
        self.0.get(&obj).map(Vec::as_slice).unwrap_or(&[])
    }
}

fn is_under(w: &World, mut i: usize, root: usize) -> bool {
    while let Some(o) = w.objects[i].owner {
        if o == root {
            return true;
        }
        i = o;
    }
    false
}

/// FUN_0015e9c8(world, mode, 0x2a, root, 0x20, out) as used by FUN_00227fe8: mode 5 = every object of the
/// class subtree (per class id, creation order), mode 1 = only those below `root`. Returns the TOTAL count
/// and the first 0x20 candidates (the original only stores 0x20 but returns the full count; FUN_00227fe8
/// then reads past its buffer when > 0x20 — not reproduced: the extra entries are not inspected).
pub fn ee_sub_15e9c8(w: &World, root: Option<usize>, class: u16) -> (usize, Vec<usize>) {
    let mut ids: Vec<usize> = w.of_class(class, true).filter(|&i| root.is_none_or(|r| is_under(w, i, r))).collect();
    ids.sort_by_key(|&i| (w.objects[i].class, i));
    let n = ids.len();
    ids.truncate(MAX_CANDIDATES);
    (n, ids)
}

/// FUN_00227fe8(world, oid, parent_oid): spawn object for a transition. Quirk kept: every candidate is
/// tested and the LAST match wins (oid at +0x18 or any alias).
pub fn ee_sub_227fe8(w: &World, aliases: &SpawnAliases, oid: i32, parent: i32) -> Option<usize> {
    // 0x00227fe8
    let mut root = None;
    if parent != -1 {
        // FUN_0015e938(world, 5, parent, 0): object by oid; when found the search mode becomes 1 (below it)
        root = w.find_oid(parent).first().copied();
    }
    let (_, cands) = ee_sub_15e9c8(w, root, CLASS_SPAWN);
    let mut found = None;
    for i in cands {
        if w.objects[i].oid == oid || aliases.get(i).contains(&oid) {
            found = Some(i);
        }
    }
    found
}

/// WP-J17 (REPLACED, no original counterpart): start spawn of a debug entry (`--table` / `--level`
/// without a transition request). The original only enters a level through a request (boot: FUN_001a4e08
/// row 25 spawn 0x189; warps / tasks / respawn: FUN_0022b490 with the spawn of the warp volume +0x150 or
/// the task +0xb8), so a level whose default spawn 0x189 does not exist (e.g. table 10 / 20, entered from
/// the hub through a task) leaves the JT at its load position (the origin). Stand-in for the request the
/// game would have made: the spawn +0xb8 of the first task (`jobs::JobMgr::load`, creation order) whose
/// level +0xbc is `row` (any row when `row` < 0) and that FUN_00227fe8 resolves in this world.
pub fn debug_entry_spawn(w: &World, aliases: &SpawnAliases, row: i32) -> Option<(usize, i32)> {
    let m = crate::engine::jobs::JobMgr::load(w);
    m.tasks.iter().filter(|t| row < 0 || t.level == row).find_map(|t| ee_sub_227fe8(w, aliases, t.spawn, -1).map(|i| (i, t.spawn)))
}

/// Objects that are live after a load: the directory entries with a == 0 are instantiated (vtable +0x4c
/// post-load runs on them), template entries (a != 0) only provide clone sources; objects created with
/// +0x50 = 1 (mission filter failed, classes 0x84/0x85) stay disabled.
pub fn active_objects(w: &World) -> Vec<usize> {
    let mut v = Vec::new();
    for e in w.entries.iter().filter(|e| e.dir.a == 0) {
        for &i in &e.instances {
            collect(w, i, &mut v);
        }
        if e.instances.is_empty() {
            for &i in &e.objects {
                collect(w, i, &mut v);
            }
        }
    }
    v
}

fn collect(w: &World, i: usize, out: &mut Vec<usize>) {
    if w.objects[i].disabled {
        return;
    }
    out.push(i);
    for &c in &w.objects[i].children {
        collect(w, c, out);
    }
}

/// WP-J12: the player character the level loaded (first class-53 JT object), where FUN_0020e890 leaves
/// the player when no spawn object was found.
pub fn player_object(w: &World) -> Option<usize> {
    w.of_class(53, false).next()
}

/// Spawn object transform (FUN_00231170 local matrix at +0x60: 4 rows x 3 f32, row 3 = position).
/// WP-J12: for the JT ([`player_object`]) its load position (FUN_001a62e8 world matrix row 3).
pub fn spawn_position(w: &World, i: usize) -> Option<[f32; 3]> {
    use crate::engine::world::objects::{LoadData, Payload};
    let x: &[u8; 48] = match &w.objects[i].payload {
        Payload::Loaded(LoadData::L231170 { xform, .. }) => xform,
        Payload::Loaded(LoadData::L231170Blob { xform, .. }) => &xform.xform,
        _ if w.objects[i].class == 53 => return Some(crate::engine::game::fields::ee_sub_1a62e8(w, i)),
        _ => return None,
    };
    let f = |k: usize| f32::from_le_bytes(x[k * 4..k * 4 + 4].try_into().unwrap());
    Some([f(9), f(10), f(11)])
}
