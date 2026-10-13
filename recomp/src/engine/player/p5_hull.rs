//! WP-P5 hull: Sly's collision hull (GEOM id 199, jt+0x264) and the JT side of the SO contact list
//! (jt+0xec -> +0x64 pair list), replacing P4's radius-30+25 sphere at the JT origin (`Jt::contacts`).
//!
//! 1:1: FUN_001f6030 (geom lookup by id +0xc), FUN_00203af0 hull setup (0x1f70c0 radius +8 = sqrt(max |p|^2)
//! over the source points, extra = 3 iff not is-a 0x34 and +0x1c == point count, jt+0x210 = min local z over
//! [0, +0x1c) from FLT_MAX, P0 = jt+0x240, P1 = P0 + up * jt+0x214 inserted at +0x1c, +0x1c += 2, jt+0x218 /
//! jt+0x21c = world P0 / P1 via 0x1664a0), FUN_001a7c18 record build (side 0 point P, side 1 point P - gap n,
//! n +0x80, gap +0x90, 0.5 / 0.5 at +0x94 / +0x98 = DAT_002bfd30 / DAT_002bfd34, kind / feats, notify slot
//! +0x1bc per side, 0x1a7fc0, prepend the chain to the list), FUN_001a7fc0 (per side +0x30 velocity, slots
//! +0x1c4 / +0x1cc, +0xb0 = v0 - v1), JT slot +0x1bc FUN_00188cc8 (cases 5 / 7 / 0x18; 7 only acts on kind
//! 0xe), PO FUN_00204250 non-0xe part (+0x94 = +0x98 = 0, kind 3 / side-1 kinds 1,4,5,6 with
//! DAT_002c3c60[side] * n.z > 0 -> secondary kind 10 record JT-side-0 along the horizontal normal, inserted
//! after the record), JT slot +0x1cc FUN_00188e78 (side velocity minus w x (p - pos)).
//! REPLACED: the contact source. The original builds the JT/level pair chain in 0x1aa1d8 with mesh/mesh
//! 0x1a8b50 / 0x1a87f0 (kinds 1/2/3; needs 0x1f6c08 adjacency + 0x1a80d8 / 0x1a81f8 / 0x1a8410 / 0x1a86b8 /
//! 0x11ea60, not ported). Here (spec_hull.md section 4.3): FUN_001a93c8 sphere of radius = hull +8 at
//! centre = *(jt+0x218) (world P0) against every collider, level = side 0 (geomA), JT = side 1; SBI kind map
//! 4 vertex -> 1, 5 edge -> 3, 6 face -> 0xd (face-ish; skipped by 0x1f90d0 and the ledge search), others
//! (0xc deep) dropped; P = level feature point c - (n_s.c - w) n_s, n = -n_s (side 1 -> side 0), gap =
//! n_s.c - w - r, featA = level vertex / edge / surface index, featB = 0. Level bodies are static: their
//! side velocity is 0 and their frame is identity (CollisionWorld geometry is in world space); their slot
//! +0x1bc / +0x1c4 / +0x1cc are the SO defaults 0x1f4fb0 / 0x1f4fb8 / 0x1f4fc0 (empty). The hull's frame
//! ALO jt+0x60 = the JT body frame (rot / pos), 0x1f70c0's source-to-owner re-expression = identity (the part
//! is Sly's own). 0x22d348 point velocity = v + w x (p - pos) (not traced; 0x188e78 removes the w term again).
//! UNPORTED (recorded in `calls`): 0x188cc8 case 4 FUN_001a2100, case 6 FUN_001374e0, case 8 FUN_0019e9c8;
//! the kind-0xe branches of 0x204250 / 0x2044b0 (producer 0x204ac8 not ported); 0x1f6c08 adjacency tables;
//! 0x1191e8 BSP transform (BSP used as loaded, identity frame); record pool 0x11ccc0 / island index +0xa8.
#![allow(non_snake_case, dead_code)]

use super::p4::{FUN_00132b10, JtContact};
use crate::engine::physics::driver::M3;
use crate::engine::physics::geom::{build_bsp, build_geom, dot, sub, BspC, CGeom, V3};
use crate::engine::physics::sphere::FUN_001a93c8;
use crate::engine::physics::CollisionWorld;
use crate::engine::world::alo::AloPart;

/// jt+0x264: hull geom id set by JT init 0x188488.
pub const JT_GEOM_ID: i32 = 199;
/// jt+0x240 after JT load 0x1885b8: (0, 0, -75) (0xc2960000).
pub const JT_240: V3 = [0.0, 0.0, -75.0];
/// jt+0x214 (PO init 0x203948): -50.0 (0xc2480000).
pub const JT_214: f32 = -50.0;
/// DAT_002a2dc0: up.
pub const UP: V3 = [0.0, 0.0, 1.0];
/// DAT_002a2da0: (1, 0, 0).
pub const DAT_002A2DA0: V3 = [1.0, 0.0, 0.0];
/// DAT_002bfd30 / DAT_002bfd34: record +0x94 / +0x98 defaults.
pub const DAT_002BFD30: f32 = 0.5;
pub const DAT_002BFD34: f32 = 0.5;
/// DAT_002c3c60: side sign table {+1, -1}.
pub const DAT_002C3C60: [f32; 2] = [1.0, -1.0];
/// DAT_002bf298 / DAT_002bf29c: 0x188cc8 case 5 window / +0x94 value.
pub const DAT_002BF298: f32 = 0.25;
pub const DAT_002BF29C: f32 = 3.0;
/// Contacts kept per collider from FUN_001a93c8 (REPLACED source; 0x14eda0's 0x40 budget).
pub const SBI_MAX: usize = 0x40;

pub const CALL_1A2100: u32 = 0x001a2100;
pub const CALL_1374E0: u32 = 0x001374e0;
pub const CALL_19E9C8: u32 = 0x0019e9c8;

fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn scale(a: V3, s: f32) -> V3 {
    [a[0] * s, a[1] * s, a[2] * s]
}
fn cross(a: V3, b: V3) -> V3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
/// row-vector frame: world = x r0 + y r1 + z r2 + pos
fn to_world(rot: &M3, pos: V3, p: V3) -> V3 {
    add(add(add(scale(rot[0], p[0]), scale(rot[1], p[1])), scale(rot[2], p[2])), pos)
}
fn to_local(rot: &M3, pos: V3, p: V3) -> V3 {
    let d = sub(p, pos);
    [dot(rot[0], d), dot(rot[1], d), dot(rot[2], d)]
}

// ---------------------------------------------------------------- hull setup

/// The JT's hull geom (jt+600, 0xb0 B record).
#[derive(Clone, Debug, PartialEq)]
pub struct JtHull {
    /// +0x78/+0x7c local points (P0 / P1 inserted at the original +0x1c) + surfaces / edges
    pub geom: CGeom,
    /// +0x68..+0x74
    pub bsp: BspC,
    /// +0xc
    pub id: i32,
    /// +8 bounding radius
    pub radius: f32,
    /// +0x1c after `+= 2`
    pub n_hull: usize,
    /// index of P0 (jt+0x218 -> geom+0x94 + i*16); P1 = i + 1
    pub i_p0: usize,
    /// jt+0x210: min local z of the original hull points
    pub z_min: f32,
}

/// 0x001f6030(alo, id): first collision part whose +0xc id matches.
pub fn FUN_001f6030(parts: &[AloPart], id: i32) -> Option<&AloPart> {
    parts.iter().find(|p| p.w0 == id)
}

/// 0x00203af0(po, src) with 0x1f70c0 (1:1 data part; frames identity, see module doc). `cls` = class id of
/// the PO (JT = 53, is-a 0x34 -> extra 2); `p240` / `f214` = jt+0x240 / jt+0x214.
pub fn FUN_00203af0(src: &AloPart, cls: i32, p240: V3, f214: f32) -> JtHull {
    let mut geom = build_geom(&src.geom);
    let npts = geom.verts.len();
    // 0x1f70c0: +8 = sqrt(max |p|^2) over the source points (start 0)
    let m = geom.verts.iter().fold(0.0f32, |m, p| if m < dot(*p, *p) { dot(*p, *p) } else { m });
    let radius = m.sqrt();
    let n = (src.s1.max(0) as usize).min(npts); // src +0x1c
    let extra = if !FUN_00132b10(cls, 0x34) && src.s1 as usize == npts { 3 } else { 2 };
    // jt+0x210: min z over [0, +0x1c) from DAT_002a85f0 = FLT_MAX
    let z_min = geom.verts[..n].iter().fold(f32::MAX, |z, p| if p[2] < z { p[2] } else { z });
    // slots inserted at +0x1c (surfaces / edges are not re-indexed: they use [0, +0x1c))
    let mut ins = vec![p240, add(p240, scale(UP, f214))];
    if extra == 3 {
        ins.push([0.0; 3]); // DAT_002a2dd0
    }
    geom.verts.splice(n..n, ins);
    JtHull { geom, bsp: build_bsp(&src.bsp), id: src.w0, radius, n_hull: n + 2, i_p0: n, z_min }
}

/// 0x00203a38 lookup + 0x203af0: Sly's hull from his model parts (`None` if geom 199 is missing; the caller
/// keeps P4's sphere then).
pub fn jt_hull_from_parts(parts: &[AloPart], cls: i32) -> Option<JtHull> {
    FUN_001f6030(parts, JT_GEOM_ID).map(|g| FUN_00203af0(g, cls, JT_240, JT_214))
}

impl JtHull {
    /// 0x001664a0(ALO+0x40, +0x78, +0x90): world points.
    pub fn FUN_001664a0(&self, rot: &M3, pos: V3) -> Vec<V3> {
        self.geom.verts.iter().map(|&p| to_world(rot, pos, p)).collect()
    }
    /// *(jt+0x218): world P0 (the contact sphere centre).
    pub fn centre(&self, rot: &M3, pos: V3) -> V3 {
        to_world(rot, pos, self.geom.verts[self.i_p0])
    }
    /// WP-P9: FUN_00118668(jt+600 BSP, a, b, 2) > 0 of 0x1999d0 (the grab volume is the hull part's BSP; the port
    /// keeps the hull local, so the world edge is taken to hull space).
    pub fn clip_edge(&self, rot: &M3, pos: V3, a: V3, b: V3) -> bool {
        if self.bsp.nodes.is_empty() {
            return false;
        }
        let mut out = Vec::new();
        crate::engine::physics::bsp::FUN_00118668(&self.geom, &self.bsp, 0, self.bsp.nodes.len(), to_local(rot, pos, a), to_local(rot, pos, b), 2, &mut out) > 0
    }
    /// *(jt+0x21c): world P1.
    pub fn p1(&self, rot: &M3, pos: V3) -> V3 {
        to_world(rot, pos, self.geom.verts[self.i_p0 + 1])
    }
}

// ---------------------------------------------------------------- contact record

/// side +0x00 geom (the owner SO +0x0c / root SO +0x08 follow from it).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GeomRef {
    /// jt+600 (owner / root SO = the JT)
    Jt,
    /// CollisionWorld collider index (static level SO)
    Collider(usize),
}

/// One record side (0x40 B at rec + s*0x40).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CtSide {
    pub geom: GeomRef,
    /// +0x10 world point (+0x18 = z)
    pub point: V3,
    /// +0x20 point in the frame of +0x04 (0x22fc48)
    pub local: V3,
    /// +0x30 point velocity (0x1a7fc0)
    pub vel: V3,
}

/// 0xc0 B contact record built by 0x1a7c18.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ContactRec {
    pub side: [CtSide; 2],
    /// +0x80 normal, from side 1 towards side 0
    pub n: V3,
    /// +0x90 signed gap
    pub gap: f32,
    /// +0x94 / +0x98
    pub f94: f32,
    pub f98: f32,
    /// +0x9c
    pub kind: i32,
    /// +0xa0 / +0xa4
    pub feat_a: usize,
    pub feat_b: usize,
    /// +0xb0 v0 - v1 (+0xb8 = z)
    pub relvel: V3,
}

/// JT state read by the JT slots 0x188cc8 / 0x188e78 and the frames of 0x1a7c18 / 0x1a7fc0.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct JtCtx {
    /// frame ALO jt+0x60 (world rows) and 0x1a62e8 position
    pub rot: M3,
    pub pos: V3,
    /// linear velocity / slot +0x114 angular velocity
    pub v: V3,
    pub w: V3,
    /// +0x18c8 jts, +0x18d0 jtbs, +0x18d4 time, DAT_002dded0 clock
    pub jts: i32,
    pub jtbs: i32,
    pub t_18d4: f32,
    pub t: f32,
}

/// 0x0022d348(alo, 0, p): point velocity (assumed v + w x (p - pos); level bodies static = 0).
fn FUN_0022d348(ctx: &JtCtx, g: GeomRef, p: V3) -> V3 {
    match g {
        GeomRef::Jt => add(ctx.v, cross(ctx.w, sub(p, ctx.pos))),
        GeomRef::Collider(_) => [0.0; 3],
    }
}

/// 0x0022fc48(0, alo, p): world -> local of the side frame.
fn FUN_0022fc48(ctx: &JtCtx, g: GeomRef, p: V3) -> V3 {
    match g {
        GeomRef::Jt => to_local(&ctx.rot, ctx.pos, p),
        GeomRef::Collider(_) => p,
    }
}

/// 0x001a7c18(gap, world, geomA, geomB, P, n, kind, featA, featB, list): 1:1 (see module doc). Returns the
/// chain (record first, then any records its notifies inserted after it); the caller prepends it to the list.
#[allow(clippy::too_many_arguments)]
pub fn FUN_001a7c18(gap: f32, ga: GeomRef, gb: GeomRef, p: V3, n: V3, kind: i32, feat_a: usize, feat_b: usize, ctx: &JtCtx, calls: &mut Vec<u32>) -> Vec<ContactRec> {
    let p1 = add(p, scale(n, -gap));
    let mk = |g, pt| CtSide { geom: g, point: pt, local: FUN_0022fc48(ctx, g, pt), vel: [0.0; 3] };
    let mut rec = ContactRec {
        side: [mk(ga, p), mk(gb, p1)],
        n,
        gap,
        f94: DAT_002BFD30,
        f98: DAT_002BFD34,
        kind,
        feat_a,
        feat_b,
        relvel: [0.0; 3],
    };
    // slot +0x1bc per side (JT 0x188cc8; level SO default 0x1f4fb0 = empty)
    let mut after = Vec::new();
    for s in 0..2 {
        if rec.side[s].geom == GeomRef::Jt {
            FUN_00188cc8(&mut rec, s, ctx, &mut after, calls);
        }
    }
    FUN_001a7fc0(&mut rec, ctx);
    let mut chain = vec![rec];
    chain.extend(after);
    chain
}

/// 0x001a7fc0(world, 1, &rec): side velocities, slots +0x1c4 (empty) / +0x1cc, relative velocity.
pub fn FUN_001a7fc0(rec: &mut ContactRec, ctx: &JtCtx) {
    for s in 0..2 {
        rec.side[s].vel = FUN_0022d348(ctx, rec.side[s].geom, rec.side[s].point);
    }
    for s in 0..2 {
        if rec.side[s].geom == GeomRef::Jt {
            FUN_00188e78(rec, s, ctx);
        }
    }
    rec.relvel = sub(rec.side[0].vel, rec.side[1].vel);
}

/// 0x00188e78(jt, rec, side): JT slot +0x1cc. 0x2044b0 acts on kind 0xe only (unported); then the JT side
/// velocity loses w x (p - pos).
pub fn FUN_00188e78(rec: &mut ContactRec, side: usize, ctx: &JtCtx) {
    let sd = &mut rec.side[side];
    if sd.geom == GeomRef::Jt {
        sd.vel = sub(sd.vel, cross(ctx.w, sub(sd.point, ctx.pos)));
    }
}

/// 0x00188cc8(jt, rec, side): JT slot +0x1bc = PO 0x204250, then the per-jts +0x94 / +0x98 tweaks.
pub fn FUN_00188cc8(rec: &mut ContactRec, side: usize, ctx: &JtCtx, after: &mut Vec<ContactRec>, calls: &mut Vec<u32>) {
    FUN_00204250(rec, side, ctx, after, calls);
    match ctx.jts {
        4 => calls.push(CALL_1A2100),
        // WP-P25: FUN_001374e0: +0x94 = DAT_002bc9b4 (0)
        6 => rec.f94 = super::p25::DAT_002BC9B4,
        8 => calls.push(CALL_19E9C8),
        5 => {
            if (ctx.jtbs - 8) as u32 <= 1 && ctx.t - ctx.t_18d4 < DAT_002BF298 {
                rec.f94 = DAT_002BF29C;
            }
        }
        // case 7 acts on kind 0xe only (attach target data, producer unported)
        0x18 => {
            rec.f98 = 0.0;
            rec.f94 = 0.0;
        }
        _ => {}
    }
}

/// 0x00204250(po, rec, side): PO slot +0x1bc, non-0xe part (1:1).
pub fn FUN_00204250(rec: &mut ContactRec, side: usize, ctx: &JtCtx, after: &mut Vec<ContactRec>, calls: &mut Vec<u32>) {
    rec.f94 = 0.0;
    rec.f98 = 0.0;
    let k = rec.kind;
    if k == 0xe {
        return; // UNPORTED: +0x94 / +0xa0 / +0xa4 from e8+0xd0 (no kind 0xe producer)
    }
    if !(k == 3 || (side == 1 && matches!(k, 1 | 4 | 5 | 6))) {
        return;
    }
    let sg = DAT_002C3C60[side];
    if !(0.0 < sg * rec.n[2]) {
        return;
    }
    let mut d = scale(rec.n, sg);
    d[2] = 0.0;
    let l = dot(d, d).sqrt();
    let d = if l < 0.0001 { DAT_002A2DA0 } else { scale(d, 1.0 / l) };
    let other = rec.side[1 - side];
    let gap = dot(sub(rec.side[side].point, other.point), d);
    let p = add(other.point, scale(d, gap));
    // 0x1a7c18(gap, world, jt+600, other geom, P, d, 10, 0, 0, &rec->next)
    let mut chain = FUN_001a7c18(gap, GeomRef::Jt, other.geom, p, d, 10, 0, 0, ctx, calls);
    chain.append(after);
    *after = chain;
}

// ---------------------------------------------------------------- contact source (REPLACED)

/// SBI kind -> record kind (REPLACED map, see module doc).
pub fn sbi_kind(k: u32) -> Option<i32> {
    match k {
        4 => Some(1),
        5 => Some(3),
        6 => Some(0xd),
        _ => None,
    }
}

/// REPLACED 0x1aa1d8 JT/level pairs: the JT's pair list (jt+0xec+0x64 chains, list order = prepend order)
/// from FUN_001a93c8 at *(jt+0x218) with the hull radius. `skip` = the JT's own object (its parts are also
/// in `CollisionWorld`).
pub fn jt_hull_records(hull: &JtHull, cw: &CollisionWorld, skip: Option<usize>, ctx: &JtCtx, calls: &mut Vec<u32>) -> Vec<ContactRec> {
    let c = hull.centre(&ctx.rot, ctx.pos);
    let r = hull.radius;
    let mut list: Vec<ContactRec> = Vec::new();
    for (k, col) in cw.colliders.iter().enumerate() {
        if Some(col.object) == skip {
            continue;
        }
        let mut sbi = Vec::new();
        FUN_001a93c8(r, c, &col.geom, &col.bsp, SBI_MAX, &mut sbi);
        for s in sbi {
            let Some(kind) = sbi_kind(s.kind) else { continue };
            let ns = [s.plane[0], s.plane[1], s.plane[2]];
            let d = dot(ns, c) - s.plane[3];
            let p = sub(c, scale(ns, d));
            let chain = FUN_001a7c18(d - r, GeomRef::Collider(k), GeomRef::Jt, p, scale(ns, -1.0), kind, s.feature, 0, ctx, calls);
            list.splice(0..0, chain);
        }
    }
    list
}

/// Ledge-search view of a record: (contact, side) with side = (jt != rec+8), contact = the other side's
/// point / collider, +0x9c, +0xa0, n turned towards the JT, +0xb8. `None` if neither side is a collider.
pub fn to_jt_contact(rec: &ContactRec) -> Option<(JtContact, u32)> {
    let side = (rec.side[0].geom != GeomRef::Jt) as usize;
    let other = rec.side[1 - side];
    let GeomRef::Collider(collider) = other.geom else { return None };
    let n = scale(rec.n, DAT_002C3C60[side]);
    Some((JtContact { collider, kind: rec.kind, feature: rec.feat_a, pos: other.point, n, vn: rec.relvel[2] }, side as u32))
}

/// Per-frame entry: Sly's contacts as P4 `JtContact`s with their side (replaces `Jt::contacts`).
pub fn jt_hull_contacts(hull: &JtHull, cw: &CollisionWorld, skip: Option<usize>, ctx: &JtCtx, calls: &mut Vec<u32>) -> Vec<(JtContact, u32)> {
    jt_hull_records(hull, cw, skip, ctx, calls).iter().filter_map(to_jt_contact).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::physics::driver::IDENT;
    use crate::engine::physics::geom::{BspNode, Edge, Surf, FUN_00119308};
    use crate::engine::physics::query::{Collider, Source};
    use crate::engine::world::loads::{Bsp, BspNode as LBspNode, Geom, Surface};

    /// Box [x0,x1]x[y0,y1]x[z0,z1]; surfaces 0 top, 1 +x, 2 -x, 3 +y, 4 -y, 5 bottom; the top owns its 4
    /// edges (edge 0 = the +x edge, other surface 1). Convex BSP chain (null negative child = solid).
    fn boxc(lo: V3, hi: V3) -> (CGeom, BspC) {
        let v = |i: usize| [if i & 1 != 0 { hi[0] } else { lo[0] }, if i & 2 != 0 { hi[1] } else { lo[1] }, if i & 4 != 0 { hi[2] } else { lo[2] }];
        let verts: Vec<V3> = (0..8).map(v).collect();
        let quads: [[usize; 3]; 6] = [[4, 5, 7], [1, 3, 7], [0, 4, 6], [2, 6, 7], [0, 1, 5], [0, 2, 3]];
        let mut surfs: Vec<Surf> = quads
            .iter()
            .map(|q| Surf { plane: FUN_00119308(verts[q[0]], verts[q[1]], verts[q[2]]), own_edges: vec![], other_edges: vec![], v0: q[0] as u16 })
            .collect();
        let edges = vec![
            Edge { a: 5, b: 7, owner: 0, other: 1 },
            Edge { a: 4, b: 6, owner: 0, other: 2 },
            Edge { a: 6, b: 7, owner: 0, other: 3 },
            Edge { a: 4, b: 5, owner: 0, other: 4 },
        ];
        surfs[0].own_edges = vec![0, 1, 2, 3];
        for (k, e) in edges.iter().enumerate() {
            surfs[e.other as usize].other_edges.push(k);
        }
        let nodes = (0..6).map(|i| BspNode { child: [(i < 5).then_some(i + 1), None], surf: Some(i) }).collect();
        (CGeom { verts, surfs, edges }, BspC { a: 0, b: 0, nodes })
    }

    fn world(lo: V3, hi: V3) -> CollisionWorld {
        let (geom, bsp) = boxc(lo, hi);
        CollisionWorld { colliders: vec![Collider { object: 7, source: Source::Solid, geom, bsp, sphere: None }] }
    }

    /// hull with radius exactly r (one point at distance r) and P0 = JT_240
    fn hull(r: f32) -> JtHull {
        let (geom, bsp) = boxc([-1.0; 3], [1.0; 3]);
        let mut geom = geom;
        geom.verts = vec![[0.0, 0.0, -r], [0.0, 0.0, 0.0]];
        geom.verts.splice(1..1, [JT_240, add(JT_240, scale(UP, JT_214))]);
        JtHull { geom, bsp, id: 199, radius: r, n_hull: 3, i_p0: 1, z_min: -r }
    }

    fn ctx(pos: V3, v: V3) -> JtCtx {
        JtCtx { rot: IDENT, pos, v, w: [0.0, 0.0, 3.0], jts: 2, jtbs: 4, t_18d4: 0.0, t: 1.0 }
    }

    fn near(a: V3, b: V3) -> bool {
        (0..3).all(|i| (a[i] - b[i]).abs() < 1e-3)
    }

    #[test]
    fn hull_setup_from_part() {
        let s = 0.0f32;
        let verts = vec![[10.0, 0.0, -20.0], [0.0, 30.0, 40.0], [-5.0, 0.0, -90.0], [1.0, 1.0, 1.0]];
        let geom = Geom { verts, surfaces: vec![Surface { verts: [0, 1, 2], discarded: [s; 4], edges: vec![] }], edge_pool: 0 };
        let bsp = Bsp { a: 0, b: 0, nodes: vec![LBspNode { surface: 0, children: Some([-1, -1]) }] };
        let part = AloPart { r#ref: 0, w0: 199, b0: 0, b1: 0, f: 0.0, geom, bsp, w1: 0, s0: 0, s1: 3, b2: 0 };
        let mut other = part.clone();
        other.w0 = 5;
        let parts = [other, part];
        let h = jt_hull_from_parts(&parts, 53).unwrap();
        // radius over all source points: max(|(-5,0,-90)|) = sqrt(8125)
        assert!((h.radius - 8125.0f32.sqrt()).abs() < 1e-3);
        assert_eq!(h.z_min, -90.0); // [0, 3): ignores the probe point (1,1,1)
        assert_eq!((h.i_p0, h.n_hull), (3, 5));
        assert_eq!(h.geom.verts[3], [0.0, 0.0, -75.0]);
        assert_eq!(h.geom.verts[4], [0.0, 0.0, -125.0]);
        assert_eq!(h.geom.verts[5], [1.0, 1.0, 1.0]); // JT is-a 0x34 -> extra 2
        assert_eq!(h.centre(&IDENT, [100.0, 0.0, 0.0]), [100.0, 0.0, -75.0]);
        assert_eq!(h.p1(&IDENT, [100.0, 0.0, 0.0]), [100.0, 0.0, -125.0]);
        // not is-a 0x34 (class 1) and +0x1c == point count -> 3 slots, zero point third
        let mut p3 = parts[1].clone();
        p3.s1 = 4;
        let h3 = FUN_00203af0(&p3, 1, JT_240, JT_214);
        assert_eq!(h3.geom.verts.len(), 7);
        assert_eq!(h3.geom.verts[6], [0.0; 3]);
        assert!(jt_hull_from_parts(&parts[..1], 53).is_none());
    }

    #[test]
    fn floor_face_is_kind_d_jt_side_1() {
        let cw = world([-1000.0, -1000.0, -1000.0], [1000.0, 1000.0, 0.0]);
        let h = hull(20.0);
        // centre = pos + (0,0,-75) = (0,0,10): 10 above the floor, r 20 -> gap -10
        let c = ctx([0.0, 0.0, 85.0], [100.0, 0.0, -50.0]);
        let mut calls = Vec::new();
        let recs = jt_hull_records(&h, &cw, None, &c, &mut calls);
        assert_eq!(recs.len(), 1);
        let r = recs[0];
        assert_eq!((r.kind, r.feat_a, r.feat_b), (0xd, 0, 0));
        assert_eq!(r.side[0].geom, GeomRef::Collider(0));
        assert_eq!(r.side[1].geom, GeomRef::Jt);
        assert!(near(r.n, [0.0, 0.0, -1.0]));
        assert!((r.gap + 10.0).abs() < 1e-4);
        assert!(near(r.side[0].point, [0.0, 0.0, 0.0]));
        assert!(near(r.side[1].point, [0.0, 0.0, -10.0])); // centre - r up
        assert!(near(r.side[1].local, [0.0, 0.0, -95.0]));
        // JT velocity linear only (0x188e78), level static: relvel = -v
        assert!(near(r.side[1].vel, [100.0, 0.0, -50.0]));
        assert!(near(r.relvel, [-100.0, 0.0, 50.0]));
        // JT notify 0x204250 zeroed +0x94/+0x98
        assert_eq!((r.f94, r.f98), (0.0, 0.0));
        let (jc, side) = to_jt_contact(&r).unwrap();
        assert_eq!((side, jc.kind, jc.collider), (1, 0xd, 0));
        assert!(near(jc.n, [0.0, 0.0, 1.0]));
        assert_eq!(jc.vn, 50.0);
        // own object skipped
        assert!(jt_hull_records(&h, &cw, Some(7), &c, &mut calls).is_empty());
    }

    #[test]
    fn ledge_edge_is_kind_3_plus_kind_10() {
        // ledge edge along y at x = 0, z = 0 (edge 0, top surface 0 / +x surface 1)
        let cw = world([-1000.0, -1000.0, -1000.0], [0.0, 1000.0, 0.0]);
        let h = hull(20.0);
        let c = ctx([10.0, 0.0, 85.0], [100.0, 0.0, -50.0]); // centre (10, 0, 10)
        let mut calls = Vec::new();
        let recs = jt_hull_records(&h, &cw, None, &c, &mut calls);
        assert_eq!(recs.len(), 2);
        let (r3, r10) = (recs[0], recs[1]);
        let s = std::f32::consts::FRAC_1_SQRT_2;
        let d = 200.0f32.sqrt();
        assert_eq!((r3.kind, r3.feat_a), (3, 0));
        assert_eq!((r3.side[0].geom, r3.side[1].geom), (GeomRef::Collider(0), GeomRef::Jt));
        assert!(near(r3.n, [-s, 0.0, -s]));
        assert!((r3.gap - (d - 20.0)).abs() < 1e-3);
        assert!(near(r3.side[0].point, [0.0; 3]));
        let jp = 10.0 - 20.0 * s; // -4.142
        assert!(near(r3.side[1].point, [jp, 0.0, jp]));
        assert_eq!(r3.relvel[2], 50.0);
        // 0x204250: side 1, -n.z > 0 -> kind 10, JT side 0, d = (1,0,0), gap = jp, P = (jp,0,0)
        assert_eq!(r10.kind, 10);
        assert_eq!((r10.side[0].geom, r10.side[1].geom), (GeomRef::Jt, GeomRef::Collider(0)));
        assert!(near(r10.n, [1.0, 0.0, 0.0]));
        assert!((r10.gap - jp).abs() < 1e-3);
        assert!(near(r10.side[0].point, [jp, 0.0, 0.0]));
        assert!(near(r10.side[1].point, [0.0; 3]));
        assert_eq!(r10.relvel[2], -50.0);
        // ledge view: kind 3 side 1, other point = the edge point, not separating ({1,-1}[1] * 50 < 0)
        let v: Vec<_> = recs.iter().filter_map(to_jt_contact).collect();
        assert_eq!((v[0].1, v[0].0.kind, v[0].0.feature), (1, 3, 0));
        assert!(near(v[0].0.pos, [0.0; 3]));
        assert!(DAT_002C3C60[v[0].1 as usize] * v[0].0.vn <= 0.0);
        assert_eq!((v[1].1, v[1].0.kind), (0, 10));
    }

    #[test]
    fn vertex_is_kind_1() {
        // centre beyond the (0,0,0) corner: both top edges ending there clamp to vertex 7
        let cw = world([-100.0, -100.0, -100.0], [0.0, 0.0, 0.0]);
        let h = hull(20.0);
        let c = ctx([5.0, 5.0, 80.0], [0.0; 3]); // centre (5, 5, 5)
        let mut calls = Vec::new();
        let recs = jt_hull_records(&h, &cw, None, &c, &mut calls);
        let r = recs.iter().find(|r| r.kind == 1).expect("vertex contact");
        assert_eq!(r.feat_a, 7); // vertex (0,0,0) = index 7 of the box
        assert!(near(r.side[0].point, [0.0; 3]));
        assert_eq!(to_jt_contact(r).unwrap().1, 1);
    }

    #[test]
    fn jt_state_tweaks() {
        let mut calls = Vec::new();
        let mut c = ctx([0.0; 3], [0.0; 3]);
        c.jts = 5;
        c.jtbs = 9;
        c.t = 1.1;
        c.t_18d4 = 1.0;
        let r = FUN_001a7c18(-1.0, GeomRef::Collider(0), GeomRef::Jt, [0.0; 3], [0.0, 0.0, 1.0], 0xd, 0, 0, &c, &mut calls);
        assert_eq!((r[0].f94, r[0].f98), (3.0, 0.0));
        c.jts = 0x18;
        let r = FUN_001a7c18(-1.0, GeomRef::Collider(0), GeomRef::Jt, [0.0; 3], [0.0, 0.0, 1.0], 0xd, 0, 0, &c, &mut calls);
        assert_eq!((r[0].f94, r[0].f98), (0.0, 0.0));
        c.jts = 4;
        FUN_001a7c18(-1.0, GeomRef::Collider(0), GeomRef::Collider(1), [0.0; 3], [0.0, 0.0, 1.0], 0xd, 0, 0, &c, &mut calls);
        assert!(calls.is_empty()); // no JT side -> no JT notify, defaults kept
        let r = FUN_001a7c18(-1.0, GeomRef::Collider(0), GeomRef::Collider(1), [0.0; 3], [0.0, 0.0, 1.0], 0xd, 0, 0, &c, &mut calls);
        assert_eq!((r[0].f94, r[0].f98), (0.5, 0.5));
        FUN_001a7c18(-1.0, GeomRef::Collider(0), GeomRef::Jt, [0.0; 3], [0.0, 0.0, 1.0], 0xd, 0, 0, &c, &mut calls);
        assert_eq!(calls, vec![CALL_1A2100]);
    }
}
