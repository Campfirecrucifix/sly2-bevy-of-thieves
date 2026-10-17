//! WP-K16: class-10 path data (C2d9040 +0xe0..+0x178) of a world object in its world frame.
//! - `ee_sub_1e5df8`: the load (FUN_001e5df8, stream part in world::loads_b4 `B4Load::L1e5df8`): +0xe0 kind
//!   (0 polyline / 1 Hermite spline), +0xe4 looped, +0xe8 n, +0xec knot params (`words`, n f32), +0xf4 knots,
//!   spline only +0x140 incoming / +0x144 outgoing tangents; then FUN_001e6180.
//! - `ee_sub_1e6180`: FUN_001e6330 (+0x178 = 0, +0x148 = -1: the segment-table cache, recomputed per call by
//!   `p5_target::Path::FUN_001e5fc8`), copy of the vtable +0xfc world matrix to +0x100..+0x13c, then the
//!   cumulative arc lengths +0xf0: polyline = world knot distances, spline = per segment the FUN_001e5fc8
//!   11-entry table's last entry (+0x174). The +0x17c tail (FUN_001e9828) is 0 at load and not modelled.
//! - The point / length / closest-point queries are `player::p5_target::Path` (FUN_001e6648, 1e64c8, 1e67b8,
//!   1e6f10, 1e6f60, 1e63c0; spline branches 1:1).
//! - `world_xf`: the vtable +0xfc matrix of the object = its owner-chain world frame (player::p15
//!   `instance_frames`, P15): copy 0 (the streamed subtree) unless `copy` names a placed clone of the root.
//! - `rail`: the same data as the `player::p5_attach::Path` the K14 rail walker takes (+ the full spline path:
//!   p5_attach walks splines as polylines, UNPORTED there).
#![allow(non_snake_case)]
use crate::engine::gfx::attach::BoneWorlds;
use crate::engine::gfx::scene::Affine;
use crate::engine::player::p5_target::Path;
use crate::engine::world::loads_b4::B4Load;
use crate::engine::world::objects::{LoadData, Payload};
use crate::engine::world::World;

/// the vtable +0xfc world matrix of object `i`; `copy` = the placed copy root (P15 copy id) to take, None = copy 0
/// An object without an own stream frame (e.g. a class-112 launcher record) sits at its nearest framed
/// owner's origin (its vtable +0xfc returns the owner's matrix). A class-10 path has its own load xform (B4
/// L1e5df8 +0x40 block, not in gfx::attach::xfm_of): local x the FUN_0022d6b0 owner bone (rest pose), then the
/// P15 placement of its owner-chain root (same composition as player::p15::instance_frames).
pub fn world_xf(w: &World, i: usize, copy: Option<usize>) -> Option<Affine> {
    let mut bw = BoneWorlds::default();
    if let Some(own) = path_own(w, &mut bw, i) {
        return Some(placed(w, &mut bw, i, own, copy));
    }
    let mut j = i;
    loop {
        let f = crate::engine::player::p15::instance_frames(w, &mut bw, j);
        if let Some(m) = copy.and_then(|c| f.iter().find(|x| x.0 == c)).or_else(|| f.iter().find(|x| x.0 == j)).or(f.first()) {
            return Some(m.1);
        }
        j = w.objects.get(j)?.owner?;
    }
}

/// own (unplaced) world frame of a class-10 path object. The FUN_002317e0 load-context bone is not resolved
/// here (gfx::attach::context_of is private; path owners 76 / 85 / 48 do not push one).
fn path_own(w: &World, bw: &mut BoneWorlds, i: usize) -> Option<Affine> {
    use crate::engine::gfx::attach::{affine_from_bytes, ee_sub_22d6b0, BoneRef};
    let Payload::Loaded(LoadData::B4(bx)) = &w.objects.get(i)?.payload else { return None };
    let B4Load::L1e5df8 { xform, .. } = &**bx else { return None };
    let local = affine_from_bytes(&xform.xform);
    Some(match ee_sub_22d6b0(w, xform.s, w.objects[i].owner, None, BoneRef::None) {
        BoneRef::Bone(o, k) => crate::engine::gfx::scene::mul(&local, &bw.get(w, o, k)),
        _ => local,
    })
}

/// P15 placement of an own frame: relative to the placed owner-chain root, under the chosen copy's +0x12c
/// matrix and the copy's owner frame
fn placed(w: &World, bw: &mut BoneWorlds, i: usize, own: Affine, copy: Option<usize>) -> Affine {
    use crate::engine::gfx::attach::affine_from_bytes;
    use crate::engine::gfx::scene::{invert, mul};
    let mut root = i;
    while let Some(o) = w.objects[root].owner {
        if w.life.placed.contains_key(&root) {
            break;
        }
        root = o;
    }
    let c = match copy {
        Some(c) if w.life.placed.contains_key(&c) && (c == root || w.objects[c].clone_of == Some(root)) => c,
        _ if w.life.placed.contains_key(&root) => root,
        _ => return own,
    };
    let rel = match bw.xfm_world(w, root) {
        Some(r) if root != i => mul(&own, &invert(&r)),
        _ => own,
    };
    let mut place = affine_from_bytes(&w.life.placed[&c]);
    if let Some(o) = w.objects[c].owner.and_then(|o| bw.xfm_world(w, o)) {
        place = mul(&place, &o);
    }
    mul(&rel, &place)
}

/// FUN_001e5df8 (data part) + FUN_001e6180 with world matrix `xf`
pub fn ee_sub_1e5df8(w: &World, i: usize, xf: Affine) -> Option<Path> {
    let Payload::Loaded(LoadData::B4(bx)) = &w.objects.get(i)?.payload else { return None };
    let B4Load::L1e5df8 { a, b, words, p0, p1, p2, .. } = &**bx else { return None };
    let n = p0.len();
    if n < 2 {
        // every query indexes [i + 1]: a 0/1-knot path has no segment
        return None;
    }
    let s: Vec<f32> = words.chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect();
    let spline = *a != 0;
    let mut p = Path {
        spline,
        looped: *b != 0,
        s,
        l: vec![0.0; n],
        pts: p0.clone(),
        tan_in: if spline { p1.clone() } else { vec![] },
        tan_out: if spline { p2.clone() } else { vec![] },
        xf,
    };
    ee_sub_1e6180(&mut p, xf);
    Some(p)
}

/// FUN_001e6180: world matrix + cumulative arc lengths (+0xf0)
pub fn ee_sub_1e6180(p: &mut Path, xf: Affine) {
    p.xf = xf;
    let n = p.pts.len();
    p.l = vec![0.0; n];
    if !p.spline {
        let wp = |q: [f32; 3]| [0, 1, 2].map(|j| q[0] * xf[0][j] + q[1] * xf[1][j] + q[2] * xf[2][j] + xf[3][j]);
        let mut a = wp(p.pts[0]);
        let mut acc = 0.0f32;
        for i in 1..n {
            let b = wp(p.pts[i]);
            let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            acc += (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            p.l[i] = acc;
            a = b;
        }
    } else {
        for i in 0..n - 1 {
            let t = p.FUN_001e5fc8(i);
            p.l[i + 1] = p.l[i] + t[10];
        }
    }
}

/// path of object `i` in its world frame (copy as `world_xf`)
pub fn world_path(w: &World, i: usize, copy: Option<usize>) -> Option<Path> {
    let xf = world_xf(w, i, copy)?;
    ee_sub_1e5df8(w, i, xf)
}

/// rail view (`player::p5_attach::Path`: kind, knots, s = the loaded knot params, l = the +0xf0 arc lengths, world
/// matrix) of a path in its world frame + the full path for the spline queries (`Some` for kind 1)
pub fn rail(w: &World, i: usize, copy: Option<usize>) -> Option<(crate::engine::player::p5_attach::Path, Option<Path>)> {
    let p = world_path(w, i, copy)?;
    let mut r = crate::engine::player::p5_attach::Path::polyline(p.pts.clone(), p.looped);
    r.kind = p.spline as i32;
    r.rot = [p.xf[0], p.xf[1], p.xf[2]];
    r.pos = p.xf[3];
    r.s = p.s.clone();
    r.l = p.l.clone();
    Some((r, if p.spline { Some(p) } else { None }))
}

/// FUN_001e7380(L, path, q): closest point searched from the param of arc length L (FUN_001e64c8), FUN_001e6ed8
/// spline branch FUN_001e6db8 (FUN_0011d9e0 over the 0x1e69a0 squared distance, bounds s[0] / s[n-1], unbounded
/// when looped; then FUN_001e6660, FUN_001e67b8, FUN_001e63c0). Type 0 = `p5_attach::FUN_001e6ed8`.
pub fn ee_sub_1e7380(p: &Path, l: f32, q: [f32; 3]) -> crate::engine::player::p5_target::PathClosest {
    let u0 = p.FUN_001e64c8(l);
    let n = p.pts.len();
    let (lo, hi) = if p.looped { (-f32::MAX, f32::MAX) } else { (p.s[0], p.s[n - 1]) };
    let (s, _) = crate::engine::player::p5_target::FUN_0011d9e0(u0, (p.s[1] - p.s[0]) * 0.1, lo, hi, |s| {
        let (c, _) = p.FUN_001e67b8(s);
        let d = [q[0] - c[0], q[1] - c[1], q[2] - c[2]];
        d[0] * d[0] + d[1] * d[1] + d[2] * d[2]
    });
    let s = p.FUN_001e6660(s);
    let (cp, dir) = p.FUN_001e67b8(s);
    crate::engine::player::p5_target::PathClosest { cp, dir, s, l: p.FUN_001e63c0(s) }
}
