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
    /// FEEL R53: the JT's other 0xb0 hull entries (FUN_00203af0 per JT_TABLE part: 199 200 201 202 203 220):
    /// (part id, first +0x1c local points) = the FUN_00203d00 / FUN_00203e08 morph targets
    pub targets: Vec<(i32, Vec<V3>)>,
    /// +0x25c: current morph target part id
    pub target: i32,
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
    JtHull { geom, bsp: build_bsp(&src.bsp), id: src.w0, radius, n_hull: n + 2, i_p0: n, z_min, targets: Vec::new(), target: src.w0 }
}

/// 0x00203a38 lookup + 0x203af0: Sly's hull from his model parts (`None` if geom 199 is missing; the caller
/// keeps P4's sphere then).
pub fn jt_hull_from_parts(parts: &[AloPart], cls: i32) -> Option<JtHull> {
    let mut h = FUN_001f6030(parts, JT_GEOM_ID).map(|g| FUN_00203af0(g, cls, JT_240, JT_214))?;
    // FEEL R53: the morph targets (FUN_001a64e8 0x2be990 JT rows of jt+0x1720 / 0x1724 / 0x380 / 0x1740 / 0x384
    // / 0x520 -> 0x203af0 entries; all on bone 2 like 199). Ids as read from that table (see FUN_0018c060).
    // FEEL R55: + the jts 4 (jt+0xec4.., 207..218) and jts 7 (jt+0x514 204, jt+0x518 205) parts
    for id in (199..=218).chain([220]) {
        if let Some(p) = FUN_001f6030(parts, id) {
            let g = build_geom(&p.geom);
            let n = (p.s1.max(0) as usize).min(g.verts.len());
            h.targets.push((id, g.verts[..n].to_vec()));
        }
    }
    Some(h)
}

// ---------------------------------------------------------------- FEEL R57: Bentley / Murray hulls

/// ALO property field 226 (descriptor 0x2ac880 + 226 * 0x28: kind 6 S32, flags 0x80, word6 = 0x264): the PO hull
/// part id +0x264 (PO init FUN_00203948 sets -1; only the JT init FUN_00188488 writes 199 itself). Row 3:
/// Bentley 7428, Murray 7048.
pub const PROP_HULL_ID: i16 = 226;
/// FUN_00203948: PO +0x214 default (0xc2480000)
pub const PO_214: f32 = -50.0;
/// FEEL R60: the PO frames from the ALO sub-object (bone) table instead of capture constants.
/// Each collision part's +0x0 ref (resolved by vtable +0x19c) is a bone index; a part's points live in that
/// bone's frame (the 0xb0 entry's owner SO, re-expressed into the hull SO by FUN_0011c318 / FUN_0011c168 in
/// FUN_00203e08 / 0x203d00). The hull SO (the PO's own SO, Murray 0xb6abe0 = part 497's owner, Bentley = the
/// 7428 / 644 / 1501 owner) is REPLACED-derived as the lowest common ancestor bone of all collision-part owners
/// (Murray: 3, Bentley: 2). FUN_002039c0 +0x240 = FUN_001a6f00(parent 0, po, 0) = the object origin in the hull
/// SO frame at post-load = the inverse of the bone chain root..self. Checked against the captures
/// (emu_rec5/060, emu_play_murray_speed/000, emu_rec9_murray/000 + 040): Bentley +0x240 (0,0,-41.55) = -bone 2
/// t 41.553; Murray (0,0,-71.76) = -bone 1 t 71.758; Murray owner offsets 7048 (bone 16) 39.574, 494 / 495 /
/// 496 / 644 / 666 / 673 (bones 20 / 19 / 21 / 18 / 17 / 15) 41.770, 497 (bone 3) 0 - all equal to the bone
/// data (R58's +41.77 assumption for the uncaptured parts is replaced by this derivation).
pub fn po_self_bone(alo: &crate::engine::world::alo::Alo) -> Option<usize> {
    let bones = &alo.base.bones;
    let chain = |mut i: usize| {
        let mut v = vec![i];
        while let Some(p) = bones.get(i).map(|b| b.r#ref).filter(|r| *r >= 0) {
            i = p as usize;
            if v.contains(&i) || i >= bones.len() {
                break;
            }
            v.push(i);
        }
        v
    };
    let owners: Vec<usize> = alo.parts.iter().filter(|p| p.s1 > 0 && p.r#ref >= 0 && (p.r#ref as usize) < bones.len()).map(|p| p.r#ref as usize).collect();
    let first = chain(*owners.first()?);
    first.into_iter().find(|c| owners.iter().all(|o| chain(*o).contains(c)))
}

/// Bone `i`'s frame expressed in ancestor bone `top` (row-vector: p_top = p * R + t, composed up the chain).
/// `None` when `top` is not an ancestor of (or equal to) `i`.
pub fn bone_to_ancestor(alo: &crate::engine::world::alo::Alo, i: usize, top: usize) -> Option<([V3; 3], V3)> {
    let bones = &alo.base.bones;
    let (mut r, mut t) = ([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]], [0.0f32; 3]);
    let mut k = i;
    for _ in 0..bones.len() + 1 {
        if k == top {
            return Some((r, t));
        }
        let b = bones.get(k)?;
        let x = b.xform.unwrap_or([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [0.0; 3]]);
        let m = |v: V3| add(add(scale(x[0], v[0]), scale(x[1], v[1])), scale(x[2], v[2]));
        r = [m(r[0]), m(r[1]), m(r[2])];
        t = add(m(t), x[3]);
        if b.r#ref < 0 {
            return None;
        }
        k = b.r#ref as usize;
    }
    None
}

/// FUN_002039c0 +0x240 for a PO: the object origin (bone chain root) in the hull SO (`self_bone`) frame.
pub fn po_240(alo: &crate::engine::world::alo::Alo, self_bone: usize) -> V3 {
    let root = (0..alo.base.bones.len()).find(|&i| alo.base.bones[i].r#ref < 0).unwrap_or(0);
    let Some((r, t)) = bone_to_ancestor(alo, self_bone, root) else { return [0.0; 3] };
    // inverse of p_root = p * R + t (R orthonormal): p = (p_root - t) * R^T
    let d = sub([0.0; 3], t);
    [dot(d, r[0]), dot(d, r[1]), dot(d, r[2])]
}

/// FEEL R58: the FUN_001a64e8 hull rows (flags, part id, field) of each class (read from SCUS_973.16): Bentley
/// FUN_00134770 0x2bc390 (0x76 rows), Murray 0x2c0270 (0x89), both + the base FUN_001400b8 0x2bc9f0 (0x35). A
/// field holds the part's 0xb0 entry only when the ALO has that part (capture: Murray +0x514 644 / +0xed0 666 /
/// +0xeec 673 set, +0x380 605 etc. null; Bentley +0x514 644 / +0x524 1501).
pub fn po_table(cls: i32) -> Vec<(u32, i32, u32)> {
    use crate::engine::disc::exe::rows3;
    let mut t = match cls as u16 {
        super::p20::CLASS_BENTLEY => rows3(0x002b_c390, 0x76),
        super::p20::CLASS_MURRAY => rows3(0x002c_0270, 0x89),
        _ => return vec![],
    };
    t.extend(rows3(0x002b_c9f0, 0x35));
    t
}

/// Prop 226 of an ALO (`None` when absent: no hull, +0x264 stays -1).
pub fn prop_hull_id(props: &[crate::engine::world::props::Prop]) -> Option<i32> {
    use crate::engine::world::props::PropValue;
    props.iter().find(|p| p.field == PROP_HULL_ID).and_then(|p| match &p.value {
        PropValue::Scalar(b) if b.len() >= 4 => Some(i32::from_le_bytes([b[0], b[1], b[2], b[3]])),
        _ => None,
    })
}

/// FEEL R58: a Bentley / Murray hull with its FUN_001a64e8 fields.
#[derive(Clone, Debug, PartialEq)]
pub struct PoHull {
    pub obj: usize,
    pub cls: u16,
    pub hull: JtHull,
    /// hull frame origin (owner bone of the hull part) in the hull SO frame (see po_hull_from_parts)
    pub ofs: V3,
    /// (field, part id) of the table rows whose part exists in the ALO (the non-null entry fields)
    pub fields: Vec<(u32, i32)>,
}

impl PoHull {
    /// the part id in object field `field` (0 = null entry)
    pub fn id_of(&self, field: u32) -> i32 {
        self.fields.iter().find(|f| f.0 == field).map_or(0, |f| f.1)
    }
}

/// FUN_00203a38 (+0x264 != -1) + FUN_00203af0 for a non-JT PO: hull part `id` with +0x240 (`po_240`) / +0x214
/// = -50, plus every FUN_001a64e8 table part of the class that exists in the ALO as a morph target (the hull
/// itself first). The port keeps the hull in its owner bone frame (`ofs` = that frame's origin in the hull SO
/// frame; rotation identity for both characters' hull owners) so the part BSP / planes are used as loaded;
/// target points are taken from their owner bone into the hull SO frame (FUN_0011c168) minus `ofs`.
pub fn po_hull_from_parts(obj: usize, alo: &crate::engine::world::alo::Alo, cls: i32, id: i32) -> Option<PoHull> {
    let parts = &alo.parts;
    let me = po_self_bone(alo)?;
    let p240 = po_240(alo, me);
    let frame = |p: &AloPart| bone_to_ancestor(alo, p.r#ref.max(0) as usize, me).unwrap_or(([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]], [0.0; 3]));
    let src = FUN_001f6030(parts, id)?;
    let ofs = frame(src).1;
    let mut h = FUN_00203af0(src, cls, sub(p240, ofs), PO_214);
    let fields: Vec<(u32, i32)> = po_table(cls).into_iter().filter(|r| FUN_001f6030(parts, r.1).is_some()).map(|r| (r.2, r.1)).collect();
    let mut ids = vec![id];
    for f in &fields {
        if !ids.contains(&f.1) {
            ids.push(f.1);
        }
    }
    for t in ids {
        if let Some(p) = FUN_001f6030(parts, t) {
            let g = build_geom(&p.geom);
            let n = (p.s1.max(0) as usize).min(g.verts.len());
            let (r, tr) = frame(p);
            let pts = g.verts[..n].iter().map(|v| sub(add(add(add(scale(r[0], v[0]), scale(r[1], v[1])), scale(r[2], v[2])), tr), ofs)).collect();
            h.targets.push((t, pts));
        }
    }
    Some(PoHull { obj, cls: cls as u16, hull: h, ofs, fields })
}

/// The inputs of the sub-object lookups: (obj+0x2c0)+0x1c state, (obj+0x4c0)+8 attach kind / +0xc substate,
/// (obj+0xdf0)+4 damage state.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PoSub {
    pub s2dc: i32,
    pub att_kind: i32,
    pub att_sub: i32,
    pub dmg: i32,
}

/// 0x0019c870(obj+0x2c0, &flag): +0x1c (obj+0x2dc) < 0xb -> obj+0x380; < 0xe -> obj+0x520; == 0x11 -> obj+0x384;
/// else obj+0x380. flag = 0.
pub fn FUN_0019c870(id_of: &dyn Fn(u32) -> i32, s: i32) -> (i32, bool) {
    (id_of(if s < 0xb { 0x380 } else if s < 0xe { 0x520 } else if s == 0x11 { 0x384 } else { 0x380 }), false)
}

/// 0x00135d80 (Bentley): +0x13d0 3 -> FUN_001944c8(+0x4c0), 2 -> FUN_0019c870(+0x2c0), 4 -> FUN_001a23f8(+0xdf0);
/// 0 -> +0x268 (his own hull); FUN_00203d00(self, part, flag).
pub fn FUN_00135d80(h: &PoHull, state: i32, s: PoSub) -> (i32, bool) {
    let f = |x: u32| h.id_of(x);
    let (id, snap) = match state {
        3 => FUN_001944c8_t(&f, s.att_kind, s.att_sub),
        2 => FUN_0019c870(&f, s.s2dc),
        4 => (FUN_001a23f8_t(&f, s.dmg), false),
        _ => (0, false),
    };
    (if id == 0 { h.hull.id } else { id }, snap)
}

/// 0x001b8008 (Murray): state 5 -> +0x1ab0[+0x1458]; state 2 -> +0x13f0 == 0 ? FUN_0019c870(+0x2c0) : +0x1abc;
/// 0xd -> FUN_001944c8(+0x4c0); 0xe -> FUN_001a23f8(+0xdf0); else +0x1a58[state]; 0 -> +0x268.
pub fn FUN_001b8008(h: &PoHull, state: i32, sub_1458: i32, f_13f0: i32, s: PoSub) -> (i32, bool) {
    let f = |x: u32| h.id_of(x);
    let (id, snap) = match state {
        5 => (f((0x1ab0 + 4 * sub_1458) as u32), false),
        2 if f_13f0 == 0 => FUN_0019c870(&f, s.s2dc),
        2 => (f(0x1abc), false),
        0xd => FUN_001944c8_t(&f, s.att_kind, s.att_sub),
        0xe => (FUN_001a23f8_t(&f, s.dmg), false),
        st => (f((0x1a58 + 4 * st) as u32), false),
    };
    (if id == 0 { h.hull.id } else { id }, snap)
}

impl JtHull {
    /// 0x001664a0(ALO+0x40, +0x78, +0x90): world points.
    pub fn FUN_001664a0(&self, rot: &M3, pos: V3) -> Vec<V3> {
        self.geom.verts.iter().map(|&p| to_world(rot, pos, p)).collect()
    }
    /// FEEL R51: the hull SO in world space for the 1:1 mesh/mesh chain (so_image::mesh_pair): points via the
    /// frame (0x1664a0), planes n' = n * rot, w' = w + n' . pos (plane . v - w convention); plus the SO body
    /// sphere (+0x70 centre / +8 radius) as the bounds centre of the world points and the max distance to it.
    pub fn world_so(&self, rot: &M3, pos: V3) -> (CGeom, V3, f32) {
        let mut g = self.geom.clone();
        g.verts = self.FUN_001664a0(rot, pos);
        for s in &mut g.surfs {
            let n = to_world(rot, [0.0; 3], [s.plane[0], s.plane[1], s.plane[2]]);
            s.plane = [n[0], n[1], n[2], s.plane[3] + dot(n, pos)];
        }
        let (lo, hi) = g.verts.iter().fold(([f32::MAX; 3], [f32::MIN; 3]), |(l, h), v| (std::array::from_fn(|k| l[k].min(v[k])), std::array::from_fn(|k| h[k].max(v[k]))));
        let c = scale(add(lo, hi), 0.5);
        let r = g.verts.iter().map(|v| { let d = sub(*v, c); dot(d, d).sqrt() }).fold(0.0, f32::max);
        (g, c, r)
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
    /// FUN_0014c6a0(hull part, a, b, 1, out) -> FUN_0014c6d8: part +0x74 BSP set -> FUN_00118668(part geom,
    /// a, b, 1); the first record's entry point (world). The port keeps the hull local: a / b go to hull space.
    pub fn FUN_0014c6a0(&self, rot: &M3, pos: V3, a: V3, b: V3) -> Option<V3> {
        if self.bsp.nodes.is_empty() {
            return None;
        }
        let mut out = Vec::new();
        if crate::engine::physics::bsp::FUN_00118668(&self.geom, &self.bsp, 0, self.bsp.nodes.len(), to_local(rot, pos, a), to_local(rot, pos, b), 1, &mut out) > 0 {
            return Some(to_world(rot, pos, out[0].pos[0]));
        }
        None
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
    /// CRAWL R56 (port-only): airborne off a kind-3 rope (`Jt::rope_jump`)
    pub rope_jump: bool,
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
    // CRAWL R56: the replacement sphere (hull radius 105 around P0 = local (0, 0, -75)) reaches far outside the
    // hull itself (the hull part's convex BSP + the P0 -> P1 leg down to local z -125); the real mesh/mesh pairs
    // of 0x1aa1d8 cannot touch anything there. A level point counts only when it is within 8 (the 0x1f70c0
    // radius margin) of the hull: inside the hull BSP after an 8-unit nudge on any axis, or within 8 of the leg.
    // Applied only on a jump off a kind-3 rope (ctx.rope_jump); elsewhere the port keeps the full sphere.
    // USER CAPTURE C5: the original jumps off rope 2523 over the theater balcony rail (top z 822) and lands on
    // the balcony; the port's ledge search 0x199d48 grabbed the rail ~120 under / 70-80 beside Sly.
    let p0 = hull.geom.verts[hull.i_p0];
    let p1 = hull.geom.verts[hull.i_p0 + 1];
    let touches = |q: V3| {
        let l = to_local(&ctx.rot, ctx.pos, q);
        let e = sub(p1, p0);
        let t = (dot(sub(l, p0), e) / dot(e, e).max(1e-6)).clamp(0.0, 1.0);
        let c = add(p0, scale(e, t));
        if dot(sub(l, c), sub(l, c)) <= 64.0 {
            return true;
        }
        if hull.bsp.nodes.is_empty() {
            return true;
        }
        const N: [V3; 7] = [[0.0; 3], [8.0, 0.0, 0.0], [-8.0, 0.0, 0.0], [0.0, 8.0, 0.0], [0.0, -8.0, 0.0], [0.0, 0.0, 8.0], [0.0, 0.0, -8.0]];
        N.iter().any(|&o| crate::engine::physics::point::FUN_00118fb0(&hull.geom, &hull.bsp, 0, hull.bsp.nodes.len(), add(l, o)).is_some())
    };
    let mut list: Vec<ContactRec> = Vec::new();
    for (k, col) in cw.colliders.iter().enumerate() {
        if Some(col.object) == skip {
            continue;
        }
        // CRAWL R59/R61: broadphase (FUN_0020faa0 SO bounds node, see CollisionWorld::build_so_masked)
        if !col.near(c, r) {
            continue;
        }
        let mut sbi = Vec::new();
        FUN_001a93c8(r, c, &col.geom, &col.bsp, SBI_MAX, &mut sbi);
        for s in sbi {
            let Some(kind) = sbi_kind(s.kind) else { continue };
            let ns = [s.plane[0], s.plane[1], s.plane[2]];
            let d = dot(ns, c) - s.plane[3];
            let p = sub(c, scale(ns, d));
            // only on a jump off a rope (R56: applying it to every jump / on the ground broke the ledge hang,
            // wired_cast_ledge_grab_climb_live, and the row-6 theater pick-back route)
            if ctx.rope_jump && !touches(s.pos) {
                continue;
            }
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

/// FEEL R52: the JT hull morph target. 0x0018c060(jt): the part the hull (jt+600) morphs toward, by JT state
/// jt+0x18c8 (`jts`): 2 -> FUN_0019c870(jt+0x2c0) on the JMP state jt+0x2dc (`jtbs`): < 0xb jt+0x380, 0xb..0xd
/// jt+0x520, 0x11 jt+0x384, else jt+0x380; 4 -> FUN_001a23f8(jt+0xdf0) and 7 -> FUN_001944c8(jt+0x4c0)
/// (FEEL R55: ported, `FUN_001a23f8` / `FUN_001944c8` on `Sub18c060`); 0x15 / 0x16 -> jt+0x520; default
/// jt+0x1720 + jts * 4. A null member -> jt+0x1720. The fields hold FUN_001a64e8 0x2be990 rows (part ids):
/// 0x1720 199, 0x1724 200, 0x1740 202, 0x380 201, 0x384 203, 0x520 220. Returns (part id, copy flag) for
/// FUN_00203d00 (flag 0 except via `sub`).
pub fn FUN_0018c060(jts: i32, jtbs: i32, sub: Sub18c060) -> (i32, bool) {
    let id_of = |field: u32| crate::engine::playerdraw::JT_TABLE.iter().find(|r| r.2 == field).map_or(0, |r| r.1);
    let (id, flag) = match jts {
        2 => (id_of(match jtbs {
            j if j < 0xb => 0x380,
            0xb..=0xd => 0x520,
            0x11 => 0x384,
            _ => 0x380,
        }), false),
        4 => (FUN_001a23f8(sub.dmg_state), false),
        7 => FUN_001944c8(sub.att_kind, sub.att_sub),
        0x15 | 0x16 => (id_of(0x520), false),
        _ => (id_of((0x1720 + jts * 4) as u32), false),
    };
    if id == 0 {
        (id_of(0x1720), flag)
    } else {
        (id, flag)
    }
}

/// FEEL R55: the inputs of FUN_0018c060's state 4 / 7 cases: (jt+0xdf0)+4 damage state, (jt+0x4c0)+8 attach
/// kind and +0xc attach substate.
#[derive(Clone, Copy, Debug, Default)]
pub struct Sub18c060 {
    pub dmg_state: i32,
    pub att_kind: i32,
    pub att_sub: i32,
}

/// 0x001a23f8(dmg, &flag) 1:1: if +4 != -1, the part at dmg+0xd4 + 4*state (jt+0xec4 + 4*state), walking the
/// fallback chain DAT_002bfb28[state] while it is null and the chain is not -1. flag = 0.
pub fn FUN_001a23f8(state: i32) -> i32 {
    FUN_001a23f8_t(&|field: u32| crate::engine::playerdraw::JT_TABLE.iter().find(|r| r.2 == field).map_or(0, |r| r.1), state)
}

/// FEEL R58: 0x001a23f8 over any object's fields (`id_of(field)` = the part id of the entry there, 0 = null).
pub fn FUN_001a23f8_t(id_of: &dyn Fn(u32) -> i32, state: i32) -> i32 {
    if state == -1 {
        return 0;
    }
    let x = crate::engine::disc::exe::exe();
    let mut i = state;
    let mut id = id_of(0xec4 + 4 * i as u32);
    while id == 0 {
        i = x.w(0x002b_fb28 + 4 * i as u32) as i32;
        if i == -1 {
            break;
        }
        id = id_of(0xec4 + 4 * i as u32);
    }
    id
}

/// 0x001944c8(att, &flag) 1:1 on the attach object jt+0x4c0: kind 1 -> +0x58 (jt+0x518); kind 0 -> sub 5 ?
/// +0x60 (jt+0x520) : +0x54 (jt+0x514); kind 8 with 8 < sub < 0xb -> +0x64 (jt+0x524), flag 1; else 0.
pub fn FUN_001944c8(kind: i32, sub: i32) -> (i32, bool) {
    FUN_001944c8_t(&|field: u32| crate::engine::playerdraw::JT_TABLE.iter().find(|r| r.2 == field).map_or(0, |r| r.1), kind, sub)
}

/// FEEL R58: 0x001944c8 over any object's fields (obj+0x4c0 + 0x54.. = obj+0x514..).
pub fn FUN_001944c8_t(id_of: &dyn Fn(u32) -> i32, kind: i32, sub: i32) -> (i32, bool) {
    match kind {
        1 => (id_of(0x518), false),
        0 => (id_of(if sub == 5 { 0x520 } else { 0x514 }), false),
        8 if sub < 0xb && 8 < sub => (id_of(0x524), true),
        _ => (0, false),
    }
}

/// DAT_002c3c68: the hull morph speed profile {vmax, v0, accel} for FUN_0011d018 (read from SCUS_973.16).
pub fn smp_2c3c68() -> [f32; 3] {
    let x = crate::engine::disc::exe::exe();
    [0u32, 4, 8].map(|k| f32::from_bits(x.w(0x002c_3c68 + k)))
}

/// FEEL R52: 0x00203e08(po): hull point velocities toward the target part (+0x25c). For the first
/// target +0x1c points: t = target point in the hull frame (FUN_0011c318 / FUN_0011c168 re-expression; identity
/// for Sly's own parts, all on bone 2), per axis v = (FUN_0011d018(cur, t, dt, smp, 0) - cur) / dt, written to
/// the hull's point-velocity array (*(+600 + 0x10)). `cur` = the hull points (+600 +0x7c), `dt` = DAT_002dded4.
pub fn FUN_00203e08(cur: &[V3], target: &[V3], n: usize, dt: f32, smp: [f32; 3]) -> Vec<V3> {
    (0..n).map(|i| std::array::from_fn(|k| (crate::engine::physics::dynamics::FUN_0011d018(cur[i][k], target[i][k], dt, smp, None) - cur[i][k]) / dt)).collect()
}

impl JtHull {
    /// FEEL R53/R54: one frame of the hull morph: FUN_00203d00(jt, FUN_0018c060 part, 0) (+0x25c = target),
    /// FUN_00203e08 point velocities, then FUN_001f6ef8 (FUN_001f3980 per 0xb0 entry with +0x10 != 0).
    /// P0 / P1 (inserted at +0x1c) get no velocity and never move.
    pub fn morph(&mut self, jts: i32, jtbs: i32, sub: Sub18c060, dt: f32) {
        let (id, snap) = FUN_0018c060(jts, jtbs, sub);
        self.morph_to(id, snap, dt);
    }

    /// FEEL R57: FUN_00203d00(po, part `id`, snap) + FUN_00203e08 + FUN_001f6ef8 for a given target part (the
    /// JT picks it with FUN_0018c060, Bentley with FUN_00135d80, Murray with FUN_001b8008).
    pub fn morph_to(&mut self, id: i32, snap: bool, dt: f32) {
        self.target = id;
        let Some(t) = self.targets.iter().find(|t| t.0 == id).map(|t| t.1.clone()) else { return };
        let n = t.len().min(self.i_p0);
        if dt <= 0.0 || n == 0 {
            return;
        }
        if snap {
            // 0x203d00 flag != 0: copy the target points (hull-frame re-expression is identity for Sly's parts),
            // zero the point velocities (FUN_002333b4 memset), FUN_001f4bf8. FEEL R56: 0x1f4bf8 is no plane rebuild,
            // only the SO dirty mark (+0xec root +0x70 |= 1 -> list 0x3e11b8 via FUN_001f4b58, |= 2 -> list
            // 0x3e11c4); the planes come from the next FUN_001f3980 -> 0x1f6ef8 pass with zero velocities = this
            // refit (run here at once instead of at the next per-SO update)
            self.geom.verts[..n].copy_from_slice(&t[..n]);
            let zero = vec![[0.0; 3]; self.geom.verts.len()];
            self.FUN_001f6ef8(dt, &zero);
            return;
        }
        let mut vel = FUN_00203e08(&self.geom.verts, &t, n, dt, smp_2c3c68());
        vel.resize(self.geom.verts.len(), [0.0; 3]);
        self.FUN_001f6ef8(dt, &vel);
    }

    /// 0x001f6ef8(dt, e): per point (+0x78 count, +0x7c points, velocities **(e+0x10), accelerations
    /// *(e+0x10)[1], zero for the JT hull: allocated by FUN_001f6c08, never written): v' = v + a dt,
    /// p += (v + v') dt / 2; e+8 = sqrt(max |p|^2); then each surface plane (+0x84, 0x20 stride) =
    /// FUN_00119308 over its FUN_001f6c08 triangle (*(e+0x10)[2]). (The +0xa8 cache reset is not modelled.)
    pub fn FUN_001f6ef8(&mut self, dt: f32, vel: &[V3]) {
        let tris = FUN_001f6c08_tris(&self.geom);
        let mut m = 0.0f32;
        for (p, v) in self.geom.verts.iter_mut().zip(vel) {
            *p = add(*p, scale(*v, dt));
            m = m.max(dot(*p, *p));
        }
        self.radius = m.sqrt();
        let g = &mut self.geom;
        for (s, t) in g.surfs.iter_mut().zip(&tris) {
            if let Some(t) = t {
                s.plane = crate::engine::physics::geom::FUN_00119308(g.verts[t[0]], g.verts[t[1]], g.verts[t[2]]);
            }
        }
    }
}

/// 0x001f6c08 (triangle part): per surface, the first 3 distinct vertices met walking the edges in order
/// (+0xa4, 0x10 stride; for each edge its +4 then +6 surface, adding +0 then +2 vertex), then [1] / [2]
/// swapped when cross(p1 - p0, p2 - p0) . plane < 0. `None` for a surface with < 3 vertices.
pub fn FUN_001f6c08_tris(g: &CGeom) -> Vec<Option<[usize; 3]>> {
    let mut l: Vec<Vec<usize>> = vec![Vec::new(); g.surfs.len()];
    for e in &g.edges {
        for s in [e.owner as usize, e.other as usize] {
            if s >= l.len() {
                continue;
            }
            for v in [e.a as usize, e.b as usize] {
                if l[s].len() < 3 && !l[s].contains(&v) {
                    l[s].push(v);
                }
            }
        }
    }
    l.iter()
        .zip(&g.surfs)
        .map(|(t, s)| {
            if t.len() < 3 {
                return None;
            }
            let p = |i: usize| g.verts[t[i]];
            let d = |i: usize| [p(i)[0] - p(0)[0], p(i)[1] - p(0)[1], p(i)[2] - p(0)[2]];
            let n = cross(d(1), d(2));
            Some(if dot(n, [s.plane[0], s.plane[1], s.plane[2]]) < 0.0 { [t[0], t[2], t[1]] } else { [t[0], t[1], t[2]] })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::physics::driver::IDENT;
    use crate::engine::physics::geom::{BspNode, Edge, Surf, FUN_00119308};
    use crate::engine::physics::query::{Collider, Source};
    use crate::engine::world::loads::{Bsp, BspNode as LBspNode, Geom, Surface};

    /// FEEL R58: Bentley FUN_00135d80 / Murray FUN_001b8008 over the capture's non-null fields; prop 226.
    #[test]
    fn bm_morph_targets_and_prop226() {
        let (g, b) = boxc([-1.0; 3], [1.0; 3]);
        let hull = |id| JtHull { geom: g.clone(), bsp: b.clone(), id, radius: 0.0, n_hull: 8, i_p0: 8, z_min: 0.0, targets: vec![], target: id };
        // Murray capture fields (murray_speed/000) + his 0x2c0270 1a58 rows
        let m = PoHull { obj: 0, cls: 54, hull: hull(7048), ofs: [0.0; 3], fields: vec![(0x514, 644), (0xed0, 666), (0xeec, 673), (0x1a64, 497), (0x1a74, 494), (0x1a78, 494), (0x1ab0, 496), (0x1ab4, 496), (0x1ab8, 496), (0x1abc, 495)] };
        let s0 = PoSub { s2dc: -1, att_kind: -1, att_sub: -1, dmg: -1 };
        assert_eq!(FUN_001b8008(&m, 0, 0, 0, s0), (7048, false));
        assert_eq!(FUN_001b8008(&m, 3, 0, 0, s0), (497, false));
        assert_eq!(FUN_001b8008(&m, 8, 0, 0, s0), (494, false));
        assert_eq!(FUN_001b8008(&m, 22, 0, 0, s0), (496, false));
        assert_eq!(FUN_001b8008(&m, 5, 3, 0, s0), (495, false));
        assert_eq!(FUN_001b8008(&m, 2, 0, 1, s0), (495, false));
        assert_eq!(FUN_001b8008(&m, 2, 0, 0, s0), (7048, false)); // +0x380 null
        assert_eq!(FUN_001b8008(&m, 0xd, 0, 0, PoSub { att_kind: 0, att_sub: 0, ..s0 }), (644, false));
        assert_eq!(FUN_001b8008(&m, 0xe, 0, 0, PoSub { dmg: 3, ..s0 }), (666, false));
        assert_eq!(FUN_001b8008(&m, 0xe, 0, 0, s0), (7048, false));
        // Bentley capture fields: 644, 1501 (base row +0x524, kind 8 sub 9 -> snap)
        let bt = PoHull { obj: 0, cls: 55, hull: hull(7428), ofs: [0.0; 3], fields: vec![(0x514, 644), (0x524, 1501)] };
        assert_eq!(FUN_00135d80(&bt, 0, s0), (7428, false));
        assert_eq!(FUN_00135d80(&bt, 3, PoSub { att_kind: 8, att_sub: 9, ..s0 }), (1501, true));
        assert_eq!(FUN_00135d80(&bt, 3, PoSub { att_kind: 0, att_sub: 1, ..s0 }), (644, false));
        assert_eq!(FUN_00135d80(&bt, 2, PoSub { s2dc: 12, ..s0 }), (7428, false));
        assert_eq!(FUN_0019c870(&|f| if f == 0x520 { 647 } else { 0 }, 12), (647, false));
        assert_eq!(FUN_0019c870(&|f| if f == 0x384 { 606 } else { 0 }, 0x11), (606, false));
        use crate::engine::world::props::{Prop, PropValue};
        let props = vec![Prop { field: 12, value: PropValue::Scalar(vec![1, 0, 0, 0]) }, Prop { field: 226, value: PropValue::Scalar(7428i32.to_le_bytes().to_vec()) }];
        assert_eq!(prop_hull_id(&props), Some(7428));
        assert_eq!(prop_hull_id(&props[..1]), None);
    }

    /// FEEL R57: morph_to moves the hull points toward the target part and leaves P0 / P1 alone.
    #[test]
    fn morph_to_moves_toward_target() {
        let (g, b) = boxc([-10.0, -10.0, 0.0], [10.0, 10.0, 20.0]);
        let n = g.verts.len();
        let mut h = JtHull { geom: g.clone(), bsp: b, id: 1, radius: 0.0, n_hull: n, i_p0: n, z_min: 0.0, targets: vec![], target: 1 };
        h.geom.verts.push([0.0, 0.0, -30.0]);
        h.geom.verts.push([0.0, 0.0, -80.0]);
        let up: Vec<V3> = g.verts.iter().map(|p| [p[0], p[1], p[2] + 10.0]).collect();
        h.targets = vec![(1, g.verts.clone()), (2, up)];
        h.morph_to(1, false, 1.0 / 60.0);
        assert_eq!(&h.geom.verts[..n], &g.verts[..]);
        for _ in 0..60 {
            h.morph_to(2, false, 1.0 / 60.0);
        }
        assert_eq!(h.target, 2);
        assert!(h.geom.verts[0][2] > g.verts[0][2] + 1.0, "no morph: {:?}", h.geom.verts[0]);
        assert_eq!(h.geom.verts[n], [0.0, 0.0, -30.0]);
        h.morph_to(1, true, 1.0 / 60.0);
        assert_eq!(&h.geom.verts[..n], &g.verts[..]);
    }

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
        CollisionWorld { colliders: vec![Collider { object: 7, source: Source::Solid, geom, bsp, sphere: None, bbox: None }] }
    }

    /// hull with radius exactly r (one point at distance r) and P0 = JT_240
    fn hull(r: f32) -> JtHull {
        let (geom, bsp) = boxc([-1.0; 3], [1.0; 3]);
        let mut geom = geom;
        geom.verts = vec![[0.0, 0.0, -r], [0.0, 0.0, 0.0]];
        geom.verts.splice(1..1, [JT_240, add(JT_240, scale(UP, JT_214))]);
        JtHull { geom, bsp, id: 199, radius: r, n_hull: 3, i_p0: 1, z_min: -r, targets: Vec::new(), target: 199 }
    }

    fn ctx(pos: V3, v: V3) -> JtCtx {
        JtCtx { rot: IDENT, pos, v, w: [0.0, 0.0, 3.0], jts: 2, jtbs: 4, rope_jump: false, t_18d4: 0.0, t: 1.0 }
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

    #[test]
    fn hull_target_18c060() {
        assert_eq!(FUN_0018c060(0, 0, Default::default()), (199, false));
        assert_eq!(FUN_0018c060(1, 0, Default::default()), (200, false));
        assert_eq!(FUN_0018c060(8, 0, Default::default()), (202, false));
        assert_eq!(FUN_0018c060(3, 0, Default::default()), (199, false)); // jt+0x172c null -> jt+0x1720
    }

    #[test]
    fn hull_target_4_7_ids() {
        if std::env::var_os("SLY2_ISO").is_none() { return; }
        let rows: Vec<_> = crate::engine::playerdraw::JT_TABLE.iter().filter(|r| (0xec4..0xf00).contains(&r.2) || [0x514, 0x518, 0x520, 0x524].contains(&r.2)).cloned().collect();
        eprintln!("rows {rows:x?}");
        eprintln!("dmg {:?}", (0..12).map(FUN_001a23f8).collect::<Vec<_>>());
        eprintln!("att {:?}", [(0, 0), (0, 5), (1, 0), (8, 9), (8, 10), (8, 11)].map(|(k, s)| FUN_001944c8(k, s)));
        assert_eq!(FUN_0018c060(2, 0, Default::default()), (201, false));
        assert_eq!(FUN_0018c060(2, 0xc, Default::default()), (220, false));
        assert_eq!(FUN_0018c060(2, 0x11, Default::default()), (203, false));
        assert_eq!(FUN_0018c060(2, 0x12, Default::default()), (201, false));
        assert_eq!(FUN_0018c060(0x15, 0, Default::default()), (220, false));
        let st = |d: i32, k: i32, u: i32| Sub18c060 { dmg_state: d, att_kind: k, att_sub: u };
        assert_eq!(FUN_0018c060(4, 0, st(-1, 0, 0)), (199, false)); // +4 == -1 -> null -> jt+0x1720
        if std::env::var_os("SLY2_ISO").is_some() {
            assert_eq!(FUN_0018c060(4, 0, st(3, 0, 0)), (210, false)); // jt+0xec4 + 12
            assert_eq!(FUN_0018c060(7, 0, st(-1, 0, 0)), (204, false)); // jt+0x514
            assert_eq!(FUN_0018c060(7, 0, st(-1, 0, 5)), (220, false)); // jt+0x520
            assert_eq!(FUN_0018c060(7, 0, st(-1, 1, 0)), (205, false)); // jt+0x518
            assert_eq!(FUN_0018c060(7, 0, st(-1, 8, 9)), (199, true)); // jt+0x524 null -> jt+0x1720, flag kept
        }
    }

    #[test]
    fn hull_morph_refit_keeps_planes() {
        let (geom, bsp) = boxc([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);
        let mut h = JtHull { geom, bsp, id: 199, radius: 2.0, n_hull: 8, i_p0: 8, z_min: -1.0, targets: Vec::new(), target: 199 };
        let before: Vec<_> = h.geom.surfs.iter().map(|s| s.plane).collect();
        let tris = FUN_001f6c08_tris(&h.geom);
        assert_eq!(tris[0], Some([5, 7, 4])); // edges (5,7) (4,6): 5 7 4, already facing +z
        assert!(tris[1..].iter().all(|t| t.is_none()));
        h.FUN_001f6ef8(1.0 / 60.0, &[[0.0; 3]; 8]);
        assert!((h.radius - 3f32.sqrt()).abs() < 1e-5);
        for (a, b) in before.iter().zip(h.geom.surfs.iter().map(|s| s.plane)) {
            for k in 0..4 {
                assert!((a[k] - b[k]).abs() < 1e-5, "{a:?} {b:?}");
            }
        }
        // a +z velocity of 60 lifts the top plane by 1 in one 1/60 frame
        h.FUN_001f6ef8(1.0 / 60.0, &[[0.0, 0.0, 60.0]; 8]);
        assert!((h.geom.surfs[0].plane[3] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn hull_morph_203e08() {
        let smp = smp_2c3c68();
        let v = FUN_00203e08(&[[0.0; 3], [5.0, 5.0, 5.0]], &[[10.0, 0.0, -10.0], [5.0, 5.0, 5.0]], 2, 1.0 / 60.0, smp);
        assert_eq!(v[1], [0.0; 3]);
        assert!(v[0][0] > 0.0 && v[0][1] == 0.0 && v[0][2] < 0.0 && (v[0][0] + v[0][2]).abs() < 1e-3, "{v:?} smp {smp:?}");
    }
}
