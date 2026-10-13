//! WP-J6: the world oid search behind BIFs 112 / 115 (FUN_0015e958 -> FUN_0015e678 -> FUN_0015e3e8).
//!
//! FUN_0015e678(world, mode, oid, obj, scope, max, out): marks obj and its owner chain (+0x10) with
//! their distance (+0x2c: obj 1, owner 2, ...), walks the oid hash bucket (world+0x38, FUN_0015e380)
//! and the alias bucket (world+0x1838, FUN_0015e3c0) and lets FUN_0015e3e8 accept each candidate
//! by `mode & 0xff`:
//!   1 inside obj's subtree (first marked node up the candidate's chain is obj itself),
//!   2 direct child of obj, 3 a strict ancestor of obj (mark >= 2),
//!   4 nearest relative (smallest mark up the chain wins; unrelated only while nothing related found),
//!   5 any object with the oid.
//! FUN_0015e958 = FUN_0015e678(.., mode | 0x200, .., max 1) and returns the first stored object.
//!
//! Replaced: the oid bucket is `World::find_oid` (creation order). Not modelled: the alias bucket
//! (world+0x1838, no alias objects are created by the port's loader) and the +0x28 bits 48..51 /
//! +0x3c scope-list pre-check of FUN_0015e3e8 (FUN_001c9950 part ids) — every candidate passes it.

use crate::engine::world::World;
use std::collections::HashMap;

/// FUN_0015e958 (via FUN_0015e678 with max 1): the object found, 0 = none.
pub fn ee_sub_15e958(w: &World, mode: u32, oid: i32, obj: u32) -> u32 {
    // 0x0015e958
    let m = mode & 0xff;
    if oid == -1 || (obj == 0 && m != 5) {
        return 0;
    }
    // owner-chain marks (+0x2c)
    let mut mark: HashMap<usize, i32> = HashMap::new();
    if obj != 0 {
        let mut c = Some(obj as usize);
        let mut d = 1;
        while let Some(i) = c {
            mark.insert(i, d);
            d += 1;
            c = w.objects.get(i).and_then(|o| o.owner);
        }
    }
    let owner = |i: usize| w.objects.get(i).and_then(|o| o.owner);
    // first marked node up the chain from i (inclusive)
    let first_mark = |i: usize| {
        let mut c = Some(i);
        while let Some(j) = c {
            if let Some(&d) = mark.get(&j) {
                return Some(d);
            }
            c = owner(j);
        }
        None
    };
    let (mut count, mut best, mut out) = (0i32, i32::MAX, 0u32);
    for &cand in w.find_oid(oid) {
        // FUN_0015e3e8
        let accept = match m {
            1 => first_mark(cand) == Some(1),
            2 => owner(cand).map(|p| p as u32) == Some(obj),
            3 => mark.get(&cand).is_some_and(|&d| d >= 2),
            4 => match first_mark(cand) {
                Some(d) if d < best => {
                    best = d;
                    count = 0;
                    true
                }
                Some(_) => false,
                None => best == i32::MAX,
            },
            5 => true,
            _ => false,
        };
        if accept {
            if count < 1 {
                out = cand as u32;
            }
            count += 1;
        }
    }
    out
}

/// WP-J8: FUN_0015e9c8(world, mode, cid, obj, max, out) for the modes BIF 120 uses (0x401 with an
/// object, 0x405 without): walks the class buckets world+0x3038 (FUN_0015e3a8) of every class id from
/// `cid` up while it is in `cid`'s subtree (FUN_0015e988) and keeps the objects mode 5 (any) or mode 1
/// | 0x400 (FUN_0015e3e8 case 0, scope check skipped: inside obj's subtree, obj included) accepts.
/// Returns the objects stored (count clamped to `max` unless mode & 0x200).
/// Replaced: a class bucket is `World::of_class(id, false)` (creation order). Modes 2/3/4 are not
/// reached from scripts (BIF 120 only) and return nothing.
pub fn ee_sub_15e9c8(w: &World, mode: u32, cid: i32, obj: u32, max: usize) -> Vec<u32> {
    // 0x0015e9c8
    let m = mode & 0xff;
    if cid == -1 || (obj == 0 && m != 5) || cid < 0 {
        return Vec::new();
    }
    if m != 1 && m != 5 {
        return Vec::new();
    }
    // mode 1: the +0x2c marks of obj's owner chain; a candidate passes when the first marked node up
    // its chain is obj itself (mark 1)
    let in_subtree = |i: usize| {
        let mut c = Some(i);
        while let Some(j) = c {
            if j as u32 == obj {
                return true;
            }
            c = w.objects.get(j).and_then(|o| o.owner);
        }
        false
    };
    let mut out = Vec::new();
    let mut id = cid as u16;
    while id < 199 && crate::engine::world::class::ee_sub_15e988(id, cid as u16) {
        for i in w.of_class(id, false) {
            if m == 5 || in_subtree(i) {
                if out.len() < max {
                    out.push(i as u32);
                }
            }
        }
        id += 1;
    }
    out
}
