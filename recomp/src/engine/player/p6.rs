//! WP-P6: Sly's level data for the P5 moves (tuning blends from his ASEGs, his collision hull from his ALO,
//! attach / air targets from the level's path objects).
//!
//! 1:1: FUN_001a0300 (JT post-load 0x188608: 8-way blend entry velocities from the entry ASEGs, then the blend
//! ASEG), the FUN_0012ff10(aseg, jt, 1, 1) channel search it uses, the 0x2be990 member rows of the three JT
//! blends (FUN_001a64e8 default branch = FUN_0015e938 mode 1 for flags 0x200 / 0x300), FUN_001e6180's
//! polyline branch (arc length L of the world points).
//! The curve's slot 2 (vtable +0x1c) = `KeyVB::eval` for Bez curves, `VecCurve::eval` otherwise. The blend
//! ASEG FUN_00137a30(jt, 0x20, 8, ..) is not built (runtime class 0x6e, see Q findings): `present` = at least
//! one entry ASEG, which is what the 0x1a0450 gate (param_3 != 0) needs. Path targets: category mask / range
//! (+0x1ac / +0x198) = WP-P11 `FUN_00182c70_defaults` for classes 0xb / 0xc / 0xd (the property-list override,
//! field table index 1010 = member 0x1ac, is not applied), other classes `TARGET_MASK` / `TARGET_RANGE`; path frame = the object's own
//! FUN_00231768 matrix (owner chain not composed); knot params s = L (spline paths and their s table skipped).
//! UNPORTED: the attach-object blends a+0x70 / a+0x330 (jt+0x530 / jt+0x7f0; member rows 34..41 oids
//! 0x87..0x8e): no FUN_001a0300 call or other velocity writer exists for them in the binary (searched).
#![allow(non_snake_case)]
use super::p5_attach::{AttachTarget, Path};
use super::p5_hull::{jt_hull_from_parts, JtHull};
use super::p5_states::Blend8;
use super::P5Target;
use crate::engine::anim::aseg::{Aseg, Channel};
use crate::engine::anim::keys::{KeyVB, VecCurve};
use crate::engine::gfx::attach::{affine_from_bytes, alo_of};
use crate::engine::physics::geom::V3;
use crate::engine::playerdraw::{aseg_of, ee_sub_15e938_below, find_jt, CLASS_JT};
use crate::engine::world::loads_b4::B4Load;
use crate::engine::world::objects::{LoadData, Payload};
use crate::engine::world::World;

/// 0x2be990 rows of the JT blends: blend offset -> 8 entry ASEG oids (entry i member = blend + i*0x20 + 4).
pub const BLEND_13D0: [i32; 8] = [0x7f, 0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86];
pub const BLEND_1AC0: [i32; 8] = [0x10c, 0x10d, 0x10e, 0x10f, 0x110, 0x111, 0x112, 0x113];
/// rows 141..148 (members 0x1ed4, 0x1e54, 0x1e14, 0x1e94, 0x1ef4, 0x1e74, 0x1e34, 0x1eb4) by entry
pub const BLEND_1E10: [i32; 8] = [0x12c, 0x130, 0x12b, 0x12f, 0x12d, 0x131, 0x12a, 0x12e];

/// WP-P11: 0x00182c70 target-attachment init (att+0x1c = obj+0x1ac = 7, att+8 = obj+0x198 = DAT_002a5680 =
/// FLT_MAX) of the classes whose init calls it: 0xc (C2d2738 FUN_00182e80), 0xd (C2d6fe8 FUN_001cb908) and 0xb
/// (C2d56f8 FUN_001c3390, which then sets +0x1ac = 1). None = no traced writer.
pub fn FUN_00182c70_defaults(class: i32) -> Option<(u32, f32)> {
    match class {
        0xb => Some((1, f32::MAX)),
        0xc | 0xd => Some((7, f32::MAX)),
        _ => None,
    }
}

/// REPLACED category mask / max drop of a path target without a traced writer (see module doc).
pub const TARGET_MASK: u32 = 1;
pub const TARGET_RANGE: f32 = 500.0;

/// 0x0012ff10(aseg, jt, 1, flag 1): first flagged vector channel on the JT whose bone (resolved by the ALO's
/// +0x19c) is of kind (+0x94) 1 — the real search `anim::blends::ee_sub_12ff10`.
pub fn FUN_0012ff10<'a>(w: &World, a: &'a Aseg, jt: usize) -> Option<&'a Channel> {
    crate::engine::anim::blends::ee_sub_12ff10(w, a, jt, 1, true).map(|i| &a.channels[i])
}

/// curve vtable +0x1c (slot 2) at (t, rate 1.0).
pub fn slot2(c: &VecCurve, t: f32) -> V3 {
    match c {
        VecCurve::Bez(k) => KeyVB::eval(k, t, 1.0).0,
        _ => c.eval(t, 1.0).unwrap_or([0.0; 3]),
    }
}

/// 0x001a0300(jt, blend, out): `en` = entry member set (0x1a64e8), entry velocity +0x10 =
/// (root(0) - root(T)) / T with T = aseg +0x74 (length). Returns (blend, out != 0).
pub fn FUN_001a0300(w: &World, jt: usize, oids: &[i32; 8]) -> (Blend8, bool) {
    let mut b = Blend8::default();
    for i in 0..8 {
        let Some(a) = ee_sub_15e938_below(w, jt, oids[i]).and_then(|o| aseg_of(w, o)) else { continue };
        b.en[i] = true;
        if let Some(c) = FUN_0012ff10(w, a, jt) {
            let v = c.vec.as_ref().unwrap();
            let (p0, p1) = (slot2(v, 0.0), slot2(v, a.length));
            let q = 1.0 / a.length;
            b.vel[i] = [(p0[0] - p1[0]) * q, (p0[1] - p1[1]) * q, (p0[2] - p1[2]) * q];
        }
    }
    let present = b.en.iter().any(|&e| e);
    (b, present)
}

/// The three JT blends of 0x188608 (order 0x1ac0, 0x1e10, 0x13d0 in the original; independent).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct JtTuning {
    pub blend_13d0: Blend8,
    pub blend_1ac0: Blend8,
    pub blend_1e10: Blend8,
    /// +0x14d0 / +0x1bc0 / +0x1f10 blend ASEG present
    pub present: [bool; 3],
}

impl JtTuning {
    pub fn load(w: &World, jt: usize) -> Self {
        let (b1, p1) = FUN_001a0300(w, jt, &BLEND_1AC0);
        let (b2, p2) = FUN_001a0300(w, jt, &BLEND_1E10);
        let (b0, p0) = FUN_001a0300(w, jt, &BLEND_13D0);
        Self { blend_13d0: b0, blend_1ac0: b1, blend_1e10: b2, present: [p0, p1, p2] }
    }
    /// copy into the JT's P5 state
    pub fn apply(&self, jt: &mut super::Jt) {
        jt.p5.blend_13d0 = self.blend_13d0;
        jt.p5.blend_1ac0 = self.blend_1ac0;
        jt.p5.blend_1e10 = self.blend_1e10;
    }
}

/// Sly's hull: GEOM 199 of the JT's own ALO parts (0x203a38 / 0x203af0 via p5_hull).
pub fn player_hull(w: &World, jt: usize) -> Option<JtHull> {
    jt_hull_from_parts(&alo_of(w, jt)?.parts, CLASS_JT as i32)
}

/// 0x001e6180 polyline branch (spline branch + full object: classes/shared/path10.rs ee_sub_1e6180): L[0] = 0, L[i] = L[i-1] + |W p_i - W p_(i-1)|.
pub fn FUN_001e6180(p: &mut Path) {
    let wp = |q: V3| {
        let r = &p.rot;
        [0, 1, 2].map(|j| q[0] * r[0][j] + q[1] * r[1][j] + q[2] * r[2][j] + p.pos[j])
    };
    let mut l = vec![0.0f32; p.pts.len()];
    for i in 1..p.pts.len() {
        let (a, b) = (wp(p.pts[i - 1]), wp(p.pts[i]));
        let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        l[i] = l[i - 1] + (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    }
    p.l = l;
}

/// Path of a FUN_001e5df8 object (is-a 10), polyline only (kind +0xe0 == 0).
pub fn path_of(w: &World, i: usize) -> Option<Path> {
    let Payload::Loaded(LoadData::B4(bx)) = &w.objects[i].payload else { return None };
    let B4Load::L1e5df8 { xform, a, b, p0, .. } = &**bx else { return None };
    if *a != 0 || p0.len() < 2 {
        return None;
    }
    let m = affine_from_bytes(&xform.xform);
    let mut p = Path::polyline(p0.clone(), *b != 0);
    p.rot = [m[0], m[1], m[2]];
    p.pos = m[3];
    FUN_001e6180(&mut p);
    p.s = p.l.clone();
    Some(p)
}

/// Raw scalar property `field` of object `i` (last write wins, like the setter loop FUN_0013e818).
fn prop_bytes(w: &World, i: usize, field: i16) -> Option<&[u8]> {
    use crate::engine::world::props::PropValue;
    crate::engine::game::fields::object_props(&w.objects[i])?.iter().rev().find(|p| p.field == field).and_then(|p| match &p.value {
        PropValue::Scalar(b) => Some(b.as_slice()),
        _ => None,
    })
}
pub fn prop_u32(w: &World, i: usize, field: i16) -> Option<u32> {
    let b = prop_bytes(w, i, field)?;
    let mut v = [0u8; 4];
    let n = b.len().min(4);
    v[..n].copy_from_slice(&b[..n]);
    Some(u32::from_le_bytes(v))
}
pub fn prop_f32(w: &World, i: usize, field: i16) -> Option<f32> {
    let b = prop_bytes(w, i, field)?;
    (b.len() >= 4).then(|| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

/// WP-P14: base property index of the 8-entry TARGET sub-struct (f2092 field rows k200f, k0, k1 range +0x08,
/// k0, k106a member ref +0x10, k10, k2002, k106c mask +0x1c) per target class, and the class's TARGET
/// offset. The struct is shared by classes 0xb (842, +0x190), 0xc (808), 0xd (862), 0x13 (1017, +0x100),
/// 0x22 (798), 0x26 (818, +0xe0); the property setter writes range = prop base+2, mask = prop base+7.
pub fn target_prop_base(class: i32) -> Option<i16> {
    use super::p4::FUN_00132b10 as isa;
    // deepest class first (a subclass with its own init re-registers)
    [(0x26, 818), (0x22, 798), (0x13, 1017), (0xd, 862), (0xc, 808), (0xb, 842)]
        .into_iter()
        .find(|(c, _)| isa(class, *c))
        .map(|(_, b)| b)
}

/// WP-P14: (mask, range) of a target: init defaults (0x182c70 for 0xb..0xd, 0x184f50 7 / FLT_MAX for 0x13,
/// 0x1836c8 for 0x26: mask set by prop 825 only, range FLT_MAX) overridden by the TARGET properties.
pub fn target_mask_range(w: &World, i: usize, class: i32) -> (u32, f32) {
    use super::p4::FUN_00132b10 as isa;
    let (mut mask, mut range) = if isa(class, 0x26) || isa(class, 0x22) {
        (0, f32::MAX)
    } else if isa(class, 0x13) {
        (7, f32::MAX)
    } else {
        FUN_00182c70_defaults(if isa(class, 0xd) { 0xd } else if isa(class, 0xc) { 0xc } else { 0xb }).unwrap_or((TARGET_MASK, TARGET_RANGE))
    };
    if let Some(b) = target_prop_base(class) {
        if let Some(r) = prop_f32(w, i, b + 2) {
            range = r;
        }
        if let Some(m) = prop_u32(w, i, b + 7) {
            mask = m;
        }
    }
    (mask, range)
}

/// WP-P14: the level's attach / air targets, registered per class list like the inits do (FUN_0011a220
/// callers): is-a 0xb (C2d56f8, init 0x1c33e8 -> DAT_003e13c0) = category F path on a body (kind 0xf ->
/// jts 8 pipe climb), 0xc (0x182eb8 -> DAT_003e13f0) = category H rail (kind 0xb, member = point on the path),
/// 0xd (0x1cb9b0 -> DAT_003e13cc) = category B rope (kind 0xa), plus `level_point_targets`. Mask / range =
/// `target_mask_range`. id = world object index. REPLACED: path frame = the object's own matrix (owner chain
/// not composed), spline paths skipped (polyline only).
pub fn level_targets(w: &World) -> Vec<P5Target> {
    use super::p4::FUN_00132b10 as isa;
    use super::p5_target::{MemberGeom, TargetGeom, TargetObj};
    let mut v = vec![];
    for (i, o) in w.objects.iter().enumerate() {
        let c = o.class as i32;
        if !isa(c, 10) || o.disabled {
            continue;
        }
        let Some(path) = path_of(w, i) else { continue };
        let mut tg = AttachTarget::new(c, path.pts[0]);
        tg.pos = [path.pos[0], path.pos[1], path.pos[2]];
        let tp = super::target_path(&path);
        tg.path = Some(path);
        let (mask, range) = target_mask_range(w, i, c);
        // class 0xd +0x1bc (edge / wall-hug flag of the kind-3 attach, moved by 0x194170) / +0x1c0 / +0x1c4:
        // props 858 / 859 / 860 over the FUN_001cb908 constructor defaults (+0x1c0 = 1000, +0x1c4 = 1)
        let geom = if isa(c, 0xd) {
            tg.f_1bc = prop_u32(w, i, 858).unwrap_or(0);
            tg.f_1c0 = prop_f32(w, i, 859).unwrap_or(1000.0);
            tg.f_1c4 = prop_f32(w, i, 860).unwrap_or(1.0);
            TargetGeom::Rope { path: tp }
        } else if isa(c, 0xc) {
            TargetGeom::Rail { path: tp, member: MemberGeom::PathArc }
        } else if isa(c, 0xb) {
            TargetGeom::PathBody { path: tp }
        } else {
            continue;
        };
        v.push(P5Target { obj: TargetObj { id: i, mask, range, vel: [0.0; 3], geom }, attach: Some(tg) });
    }
    v.extend(level_point_targets(w));
    v
}

/// WP-P12 / P14: swing points (is-a 0x13, class 19 C2d2978, init 0x184f50 -> DAT_003e13b4 category C, TARGET at
/// +0x100) and hang points (is-a 0x26, class 38 C2d2378, init 0x1836c8 -> DAT_003e1408 category I kind 0xd,
/// TARGET at +0xe0; prop 816 = +0x10c attach kind for jtbs 0xd) at the object's FUN_00231368 world
/// position; round poles (is-a 0x22, init 0x183340 -> DAT_003e13fc category G, prop 796 radius +0x10c; no
/// instance in any level) on their axis. Mask / range = `target_mask_range` (props 1019 / 1024, 820 / 825).
/// Category I member at u = 0 = FUN_00183818: pos + row0 * (hang radius jt+0x103c when +0x10c == 0) ->
/// the object position here (kind 0xd objects, the only ones in the levels, offset 0).
pub fn level_point_targets(w: &World) -> Vec<P5Target> {
    use super::p4::FUN_00132b10 as isa;
    use super::p5_target::{MemberGeom, TargetGeom, TargetObj};
    let mut bw = crate::engine::gfx::attach::BoneWorlds::default();
    let mut v = vec![];
    for (i, o) in w.objects.iter().enumerate() {
        let c = o.class as i32;
        if !(isa(c, 0x26) || isa(c, 0x22) || isa(c, 0x13)) || o.disabled {
            continue;
        }
        // WP-P15: one target per placed copy of the owner-chain root (p15::instance_frames); a clone of a
        // point object is a copy of its prototype and comes out of the prototype's list
        if o.clone_of.is_some_and(|p| w.objects[p].class == o.class) {
            continue;
        }
        for (id, m) in super::p15::instance_frames(w, &mut bw, i) {
            let mut tg = AttachTarget::new(c, m[3]);
            tg.rot = [m[0], m[1], m[2]];
            let (mask, range) = target_mask_range(w, i, c);
            let geom = if isa(c, 0x26) {
                tg.f_10c = prop_u32(w, i, 816).unwrap_or(0) as i32;
                TargetGeom::Spot0xd { pt: m[3] }
            } else if isa(c, 0x22) {
                tg.r_10c = prop_f32(w, i, 796).unwrap_or(0.0);
                TargetGeom::Pole { pos: m[3], scale: tg.r_10c, member: MemberGeom::Pole { a: m[3], b: m[3], radius: tg.r_10c } }
            } else {
                TargetGeom::Point { pos: m[3], f_e8: true, is_13: true }
            };
            v.push(P5Target { obj: TargetObj { id, mask, range, vel: [0.0; 3], geom }, attach: Some(tg) });
        }
    }
    v
}

/// Everything P6 reads from a level for the player.
#[derive(Clone, Default)]
pub struct P6Level {
    pub jt: Option<usize>,
    pub tuning: Option<JtTuning>,
    pub hull: Option<JtHull>,
    pub targets: Vec<P5Target>,
    /// WP-P11: cane ASEGs / hit geom
    pub cane: super::p11_cane::CaneData,
    /// WP-P14: jts 8 speeds / state ASEG lengths (FUN_0019de58)
    pub p14: super::p14::JtP14,
    /// WP-P15: binocucom enter / exit ASEG lengths (jt+0xf0c / jt+0xf10)
    pub binoc_len: [f32; 2],
    /// WP-P18: DAT_003e13b4 aim targets (class 18 subtree)
    pub aims: Vec<super::p18::AimTarget>,
}

impl P6Level {
    pub fn load(w: &World) -> Self {
        let jt = find_jt(w);
        Self {
            jt,
            tuning: jt.map(|j| JtTuning::load(w, j)),
            hull: jt.and_then(|j| player_hull(w, j)),
            targets: level_targets(w),
            cane: jt.map(|j| super::p11_cane::CaneData::load(w, j)).unwrap_or_default(),
            p14: jt.map(|j| super::p14::JtP14::load(w, j)).unwrap_or_default(),
            binoc_len: jt.map(|j| super::p15_binoc::binoc_lens(w, j)).unwrap_or_default(),
            aims: super::p18::level_aim_targets(w),
        }
    }
    pub fn apply(&self, jt: &mut super::Jt) {
        if let Some(t) = &self.tuning {
            t.apply(jt);
        }
        jt.hull = self.hull.clone();
        jt.p5_targets = self.targets.clone();
        jt.cane.data = self.cane.clone();
        jt.p14 = self.p14.clone();
        jt.binoc.len = self.binoc_len;
        jt.p18.aims = self.aims.clone();
        jt.att.rope_blend = self.p14.rope_blend;
        jt.att.hide_blend = self.p14.hide_blend;
        jt.att.rope_v0 = self.p14.rope_v0;
        jt.att.rope_vmax = self.p14.rope_vmax;
        jt.att.b8_vel_c0 = self.p14.b8_vel_c0;
        jt.att.b8_vel_100 = self.p14.b8_vel_100;
        // jt+0x103c = 20.0 (FUN_0013ff70, base of JT init 0x188488); jt+0x240 = (0, 0, -75) (JT load 0x1885b8)
        jt.att.hang_r = 20.0;
        jt.att.grab = [0.0, 0.0, -75.0];
    }
}
