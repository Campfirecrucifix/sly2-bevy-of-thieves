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
//! (world+0x1838, no alias objects are created by the port's loader). The +0x3c scope pre-check of FUN_0015e3e8
//! is ported for the default +0x28 kind -1 (ANIM-ALL R34).

use crate::engine::world::World;
use std::collections::HashMap;

/// FUN_0015e958 (via FUN_0015e678 with max 1): the object found, 0 = none.
pub fn ee_sub_15e958(w: &World, mode: u32, oid: i32, obj: u32) -> u32 {
    // 0x0015e958
    let m = mode & 0xff;
    // POOL1 r30: REPLACED: a template copy (FUN_001c9b08) owns a cloned subtree in the original; the port's
    // copies have no children of their own, so the search scope is the template (e.g. the bentley_mine copy's
    // explode sound 3995 in script field 1240)
    let obj = match w.objects.get(obj as usize).and_then(|o| o.clone_of) {
        // HEIST: only when the template has a subtree the copy lacks (a childless clone child, e.g. the TB valve
        // copies' use points with SLY2_CLONE_KIDS, keeps itself so the search starts from its own owner chain)
        Some(t) if !w.objects.iter().any(|o| o.owner == Some(obj as usize)) && w.objects.iter().any(|o| o.owner == Some(t)) => t as u32,
        _ => obj,
    };
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
    // ANIM-ALL R34: FUN_0015e3e8 scope pre-check (mode without 0x400): the searcher scope is obj +0x3c (FUN_0015e678's
    // fallback for a 0 scope; DAT_003e139c root without obj); a candidate whose +0x28 bits 48..51 are -1 (every
    // object FUN_0013f990 never touched: 98% in emu_play_ep4/025) passes only when its +0x3c node is on that
    // chain (leaf .. root). Not modelled: the 0 / >0 kinds (FUN_0013f990 via the 0x2cbe44 callback, FUN_0013fa30).
    let chain = scope_chain(w, obj);
    // HEIST r23: then the alias bucket (FUN_0015e3c0, world+0x1838): every alias node naming `oid` with an object
    // (+0x10) is a candidate whose scope node is the alias node's parent (+4), skipped when that object is already
    // stored (e.g. the Train Hack ship 17344: its +0x3c node 0xf0a610 hangs under the root, so sequence 2410's
    // root-scoped action 7 finds it only here; emu_play_ep4b/002)
    let leafs = w.find_oid(oid).iter().map(|&c| (c, w.life.alias_leaf.get(c).copied().unwrap_or(0), false));
    let aliases = w.life.alias.bucket.get(&oid).into_iter().flatten().filter_map(|&n| {
        let nd = w.life.alias.nodes.get(n)?;
        Some((nd.object?, nd.parent.unwrap_or(0), true))
    });
    let cands: Vec<(usize, usize, bool)> = leafs.chain(aliases).collect();
    for (cand, node, alias) in cands {
        if alias && count > 0 && out == cand as u32 {
            continue;
        }
        if mode & 0x400 == 0 && !scope_ok(w, &chain, cand, node, mode) {
            continue;
        }
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

/// ANIM-ALL R36: FUN_0015e3e8 kind -1 over both FUN_0015e678 buckets (own +0x3c, then alias +4): some object with
/// `oid` passes the scope pre-check of `chain` (the R34 guard in classes/plugin.rs ee_sub_15e938)
pub fn any_in_scope(w: &World, chain: &[usize], oid: i32) -> bool {
    w.find_oid(oid).iter().any(|&c| in_scope(w, chain, c))
        || w.life.alias.bucket.get(&oid).into_iter().flatten().any(|&n| {
            w.life.alias.nodes.get(n).is_some_and(|a| a.object.is_some_and(|o| scope_ok(w, chain, o, a.parent.unwrap_or(0), 0)))
        })
}

/// ANIM-ALL R34: the searcher's +0x3c chain (alias node indices, leaf .. root node 0) of FUN_0015e3e8
pub fn scope_chain(w: &World, obj: u32) -> Vec<usize> {
    let mut v = Vec::new();
    let mut n = if obj != 0 { w.life.alias_leaf.get(obj as usize).copied() } else { Some(0) };
    while let Some(k) = n {
        v.push(k);
        n = w.life.alias.nodes.get(k).and_then(|x| x.parent);
    }
    v
}

/// ANIM-ALL R34: FUN_0015e3e8 +0x28 kind -1: the candidate's +0x3c node is on the searcher's chain
pub fn in_scope(w: &World, chain: &[usize], cand: usize) -> bool {
    scope_ok(w, chain, cand, w.life.alias_leaf.get(cand).copied().unwrap_or(0), 0)
}

/// ANIM-ALL R47: FUN_0015e3e8 pre-check by the candidate's +0x28 kind (FUN_0013f990, `World::life.scope_kind`):
/// -1 (default) `node` is on `chain`; 0 some chain node (only the first with 0x800) has `node`'s entry index
/// (FUN_001c9950 of the node keys); > 0 always passes (-2 records of FUN_0013fb40).
pub fn scope_ok(w: &World, chain: &[usize], cand: usize, node: usize, mode: u32) -> bool {
    let key = |n: usize| w.life.alias.nodes.get(n).map_or(u32::MAX, |x| x.key);
    match w.life.scope_kind.get(&cand).copied().unwrap_or(-1) {
        -1 => chain.contains(&node),
        0 => {
            let e = crate::engine::world::loads_b5::ee_sub_1c9950(key(node));
            let n = if mode & 0x800 != 0 { 1 } else { chain.len() };
            chain.iter().take(n).any(|&p| crate::engine::world::loads_b5::ee_sub_1c9950(key(p)) == e)
        }
        _ => true,
    }
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
