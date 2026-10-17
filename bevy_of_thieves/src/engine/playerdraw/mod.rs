//! WP-Q: Sly (the JT object) as a posed rig: his ALO bones, the ASEGs 0x18c250 plays, and the
//! channel -> bone binding. Bevy-independent (tests drive it headless); `plugin` (feature `game`)
//! moves the level's JT entities with the player, poses their skin joints and runs the follow camera.
//!
//! 1:1 parts:
//!  - JT post-load FUN_00188608 -> FUN_001a64e8(jt, 0xad, 0x2be990): table of {flags, id, member}; the
//!    default branch stores FUN_0015e938(0x3dd490, 1, id, jt) (object with oid `id` at or below the JT)
//!    into the member. The animation members are [`JT_ANIM_TABLE`] (table rows 1, 2, 83, 84, 85).
//!  - 0x18c250 plays the blend ASEG of a JT field: +0x1324 built by FUN_0020b868(jt + 0x1320) from the
//!    two entries at +0x1334/+0x1374, +0x1994 built by FUN_0018f908 -> FUN_00137a30(jt, 0xc, 3, jt + 0x1970)
//!    from +0x1974/+0x1980/+0x198c ([`field_members`]).
//!  - Channel binding FUN_0012fe58: target = FUN_0015e938(mode 1, oid & 0xfffff, jt), bone = target
//!    vtable +0x19c (slot 50, FUN_0022d778 = FUN_0022d6b0(kind, target +0x10 owner, target)), i.e.
//!    `gfx::attach::ee_sub_22d6b0(w, kind, owner, Some(target), BoneRef::None)`.
//!  - ASEG time advance: anim::play::drive (FUN_00130918); pose: anim::actor::ActorPose (FUN_0022faa0 walk).
//!  - WP-Q2: blend fields are built 1:1 ([`jtblend`]: FUN_00137a30 / FUN_0020b868 / FUN_0018f908 /
//!    FUN_001a0300) and weighted by FUN_0020bb58 (run) / FUN_00190520 (jump); drawn parts follow
//!    [`vis`] (FUN_001a64e8 removal / record hide, FUN_0018e540).
//! Replaced (see ENGINE_PORT.md Q / Q2 findings): the change of ASEG cross-fades the bone locals over
//! [`BLEND_TIME`] (FUN_0012f9b8 blend-in). WP-A3: the JT +0x1028 SM (class 111) runs 1:1 through
//! [`crate::engine::anim::sm`] (FUN_001f01b8 SMA, FUN_001f0d88 walk, FUN_001f1078 on the ASEG end).

pub mod jtblend;
pub mod ik;
#[cfg(feature = "game")]
pub mod plugin;
pub mod vis;
pub mod glider;
pub mod po;

pub use jtblend::{JtBlend, JumpWeights};

use crate::engine::anim::actor::ActorPose;
use crate::engine::anim::aseg::Aseg;
use crate::engine::anim::play::{bind_segments, drive, AnimClock, AsegaPlayer};
use crate::engine::anim::pose::{from_4x3, Mat4, IDENTITY};
use crate::engine::anim::sm::{Sm, Sma, SmaMsg};
use crate::engine::physics::driven::GameRng;
use crate::engine::gfx::attach::{alo_bone_refs, alo_of, ee_sub_22d6b0, BoneRef, BoneWorlds};
use crate::engine::world::objects::{LoadData, Payload};
use crate::engine::world::World;
use std::collections::HashMap;
use std::sync::Arc;

/// JT class (C2d2e98, classes_gen id 53).
pub const CLASS_JT: u16 = 53;
/// ASEG classes (FUN_0012f2a8 loads).
pub const CLASS_ASEG: u16 = 109;
pub const CLASS_ASEG_BLEND: u16 = 110;

/// 0x2be990 (FUN_001a64e8(jt, 0xad, ..)): {flags, oid, JT member} per row.
pub static JT_TABLE: std::sync::LazyLock<Vec<(u32, i32, u32)>> = std::sync::LazyLock::new(|| crate::engine::disc::exe::rows3(0x002b_e990, 173));

/// Rows whose default branch (flags 0x200 / 0x300) stores the object below the JT into the member;
/// the JT fields that hold ASEGs are a subset ([`jt_anim_members`] keeps class 109/110 objects).
pub fn jt_table_rows() -> impl Iterator<Item = (i32, u32)> {
    JT_TABLE.iter().filter(|r| r.0 == 0x200 || r.0 == 0x300).map(|r| (r.1, r.2))
}

/// JT fields 0x18c250 plays (player::JtAnim::play_aseg).
/// The JT members WP-Q resolved (table rows 1, 2, 83, 84, 85); [`JT_TABLE`] has every row.
pub const JT_ANIM_TABLE: [(i32, u32); 5] = [(0x7d, 0x1334), (0x7e, 0x1374), (0xa5, 0x1974), (0xa6, 0x1980), (0xa7, 0x198c)];

pub const FIELD_RUN: u32 = 0x1324;
pub const FIELD_JUMP: u32 = 0x1994;
/// WP-P19: JMP +0x178 (jt+0x438) paraglider open ASEG of 0x195fb8 case 0x13.
pub const FIELD_GLIDE_OPEN: u32 = 0x438;
/// ASEG members whose end pose is held (no state ASEG follows them).
pub const HOLD_AT_END: [u32; 1] = [FIELD_GLIDE_OPEN];
/// JT +0x1028: the SM (class 111, oid 0x7c) 0x18c250 drives for jts 0 (FUN_001f03a8).
pub const SM_JT_1028: i32 = 0x7c;

/// WP-P23: the JT SM fields 0x18c250 starts through FUN_001f03a8 (iStack_118) other than +0x1028:
/// +0x1950 (oid 0xa8, jtbs 3), +0x1bc4 (oid 0x117, jtbs 4), +0x1f14 (oid 0x132, jtbs 10).
pub fn sm_field_oid(field: u32) -> Option<i32> {
    match field {
        0x1950 | 0x1bc4 | 0x1f14 => JT_TABLE.iter().find(|r| r.2 == field).map(|r| r.1),
        _ => None,
    }
}
/// 0x18c250 jts 0 label (from jts other than 2 / 10 / 0xc).
pub const SM_LABEL_STAND: i32 = 0x175;
pub const FIELD_POLE: u32 = 0x1bc0;
pub const FIELD_RUN2: u32 = 0x1c34;

/// Cross-fade between ASEGs on a field change (replaced FUN_0012f9b8 blend-in, seconds).
pub const BLEND_TIME: f32 = 0.15;

/// How a blend field is built at JT post-load (FUN_00188608 tail).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FieldBuild {
    /// FUN_0020b868(jt + field - 4): 2 entries stride 0x40 at +0x10 (ASEG +4)
    Run,
    /// FUN_00137a30 without phase tables (FUN_0018f908 / FUN_001a0300)
    Plain,
}

/// Blend fields of the JT: (field, builder, entry ASEG members).
pub fn blend_fields() -> Vec<(u32, FieldBuild, Vec<u32>)> {
    vec![
        (FIELD_RUN, FieldBuild::Run, vec![0x1334, 0x1374]),
        (FIELD_RUN2, FieldBuild::Run, vec![0x1c44, 0x1c84]),
        (FIELD_JUMP, FieldBuild::Plain, vec![0x1974, 0x1980, 0x198c]),
        (FIELD_POLE, FieldBuild::Plain, (0..8).map(|k| 0x1ac4 + 0x20 * k).collect()),
        // FUN_00188608 tail: FUN_00137a30(jt, 0xc, 2, entries, 0, 0, 0, out) x4
        (0x1a1c, FieldBuild::Plain, vec![0x1a08, 0x1a14]),
        (0x1a38, FieldBuild::Plain, vec![0x1a24, 0x1a30]),
        (0x1a54, FieldBuild::Plain, vec![0x1a40, 0x1a4c]),
        (0x1a70, FieldBuild::Plain, vec![0x1a5c, 0x1a68]),
        // FUN_001a0300(jt, jt + 0x13d0, jt + 0x14d0) / (jt + 0x1e10, jt + 0x1f10)
        (0x14d0, FieldBuild::Plain, (0..8).map(|k| 0x13d4 + 0x20 * k).collect()),
        (0x1f10, FieldBuild::Plain, (0..8).map(|k| 0x1e14 + 0x20 * k).collect()),
        // FUN_00137a30(jt, 0xc, 4, jt + 0x1f60, ..) / (jt, 0xc, 3, jt + 0x2014, ..)
        (0x1f90, FieldBuild::Plain, (0..4).map(|k| 0x1f64 + 0xc * k).collect()),
        (0x2038, FieldBuild::Plain, (0..3).map(|k| 0x2018 + 0xc * k).collect()),
    ]
}

/// Entry members of a blend field (empty for direct ASEG members).
pub fn field_members(field: u32) -> Vec<u32> {
    blend_fields().into_iter().find(|f| f.0 == field).map(|f| f.2).unwrap_or_default()
}

/// The JT of the level (first class-53 object).
pub fn find_jt(w: &World) -> Option<usize> {
    w.objects.iter().position(|o| o.class == CLASS_JT)
}

/// `root` and its owned objects (FUN_0013fb40 children), depth first.
pub fn subtree(w: &World, root: usize) -> Vec<usize> {
    let mut out = vec![];
    let mut st = vec![root];
    while let Some(i) = st.pop() {
        out.push(i);
        st.extend(w.objects[i].children.iter().rev().copied());
    }
    out
}

/// FUN_0015e938(0x3dd490, 1, oid, root): object with `oid` at or below `root`.
pub fn ee_sub_15e938_below(w: &World, root: usize, oid: i32) -> Option<usize> {
    subtree(w, root).into_iter().find(|&i| w.objects[i].oid == oid)
}

/// FUN_001a64e8 for [`JT_TABLE`]: JT member -> world object (ASEGs only, as the members are).
/// WP-P19: then the post-load table 0x2bc9f0 (FUN_001400b8 -> FUN_001a64e8; glide ASEGs jt+0x438..+0x470).
/// WP-P24: and 0x2bf1b0 (post-load, after the remove-all: run2 blend entries jt+0x1c44 / jt+0x1c84 = oids
/// 0x114 / 0x115, bound on every level).
pub fn jt_anim_members(w: &World, jt: usize) -> HashMap<u32, usize> {
    let post = vis::JT_BIND_2BC9F0.iter().chain(vis::JT_BIND_2BF1B0.iter()).filter(|r| r.0 == 0x200 || r.0 == 0x300).map(|r| (r.1, r.2));
    jt_table_rows()
        .chain(post)
        .filter_map(|(id, m)| {
            let o = ee_sub_15e938_below(w, jt, id)?;
            matches!(w.objects[o].class, CLASS_ASEG | CLASS_ASEG_BLEND).then_some((m, o))
        })
        .collect()
}

pub fn aseg_of(w: &World, o: usize) -> Option<&Aseg> {
    match &w.objects[o].payload {
        Payload::Loaded(LoadData::Aseg(a)) => Some(a),
        _ => None,
    }
}

/// Sly's bones: every ALO bone of the JT subtree, parents first.
pub struct JtRig {
    pub jt: usize,
    /// (object, bone index) per rig bone
    pub keys: Vec<(usize, usize)>,
    pub index: HashMap<(usize, usize), usize>,
    /// rest-pose world (level space) per rig bone
    pub rest_world: Vec<Mat4>,
    pub pose: ActorPose,
    pub members: HashMap<u32, usize>,
    pub bindings: HashMap<usize, Vec<Option<usize>>>,
}

fn affine(m: &[[f32; 3]; 4]) -> Mat4 {
    from_4x3(m)
}

impl JtRig {
    pub fn build(w: &World, jt: usize) -> JtRig {
        let mut bw = BoneWorlds::default();
        // collect bones and parents
        let mut raw: Vec<((usize, usize), i32, Mat4, Option<(usize, usize)>)> = vec![];
        for o in subtree(w, jt) {
            let Some(a) = alo_of(w, o) else { continue };
            let refs = alo_bone_refs(w, o);
            // A6: template copies hang their null-parent bones off the owner's socket bone
            // (FUN_001ca370 -> FUN_001f21b8), placed by the +0x12c matrix
            let socket = crate::engine::gfx::attach::template_socket(w, o);
            for (k, b) in a.base.bones.iter().enumerate() {
                let mut local = b.xform.unwrap_or(crate::engine::gfx::scene::IDENTITY);
                let parent = match (refs.get(k).map(|r| r.0), socket) {
                    (Some(BoneRef::Bone(po, pk)), _) if (po, pk) != (o, k) => Some((po, pk)),
                    (Some(BoneRef::None), Some((BoneRef::Bone(po, pk), pm))) => {
                        local = crate::engine::gfx::scene::mul(&local, &pm);
                        Some((po, pk))
                    }
                    _ => None,
                };
                raw.push(((o, k), b.kind as i32, affine(&local), parent));
            }
        }
        let present: HashMap<(usize, usize), usize> = raw.iter().enumerate().map(|(i, r)| (r.0, i)).collect();
        // topological order (parents first)
        let mut order = vec![];
        let mut state = vec![0u8; raw.len()];
        fn visit(i: usize, raw: &[((usize, usize), i32, Mat4, Option<(usize, usize)>)], present: &HashMap<(usize, usize), usize>, state: &mut [u8], order: &mut Vec<usize>) {
            if state[i] != 0 {
                return;
            }
            state[i] = 1;
            if let Some(p) = raw[i].3.and_then(|p| present.get(&p).copied()) {
                visit(p, raw, present, state, order);
            }
            state[i] = 2;
            order.push(i);
        }
        for i in 0..raw.len() {
            visit(i, &raw, &present, &mut state, &mut order);
        }
        let mut index = HashMap::new();
        let mut keys = vec![];
        let mut spec = vec![];
        for &i in &order {
            let (key, kind, mut local, parent) = raw[i];
            let p = match parent {
                Some(pk) => match index.get(&pk) {
                    Some(&pi) => Some(pi),
                    None => {
                        // parent outside the subtree (or a cycle): fold its rest world in
                        let pw = affine(&bw.get(w, pk.0, pk.1));
                        local = crate::engine::anim::pose::ee_sub_11c168(&pw, &local);
                        None
                    }
                },
                None => None,
            };
            index.insert(key, keys.len());
            keys.push(key);
            spec.push((kind, local, p));
        }
        let mut pose = ActorPose::new(&spec);
        pose.compose(&IDENTITY);
        let rest_world = pose.world();
        let members = jt_anim_members(w, jt);
        let mut bindings = HashMap::new();
        for &o in members.values() {
            if let Some(a) = aseg_of(w, o) {
                bindings.insert(o, bind_channels(w, jt, a, &index));
            }
        }
        JtRig { jt, keys, index, rest_world, pose, members, bindings }
    }

    /// Rig bone of a skin joint ref.
    pub fn bone_of(&self, r: &BoneRef) -> Option<usize> {
        match r {
            BoneRef::Bone(o, k) => self.index.get(&(*o, *k)).copied(),
            _ => None,
        }
    }

    /// Direct ASEG member of a field (None for blend fields and unresolved members).
    /// JT field holding ASEG object `o` (0 if none).
    pub fn field_of(&self, o: usize) -> u32 {
        self.members.iter().find(|m| *m.1 == o).map_or(0, |m| *m.0)
    }

    pub fn aseg_for(&self, field: u32) -> Option<usize> {
        if !field_members(field).is_empty() {
            return None;
        }
        self.members.get(&field).copied()
    }

    /// The JT's blend ASEGs, built as the JT post-load does (FUN_0020b868 / FUN_0018f908 / FUN_001a0300).
    pub fn build_blends(&self, w: &World) -> HashMap<u32, JtBlend> {
        let mut out = HashMap::new();
        for (field, build, ms) in blend_fields() {
            let asegs: Vec<Option<usize>> = ms.iter().map(|m| self.members.get(m).copied()).collect();
            let b = match build {
                FieldBuild::Run => jtblend::ee_sub_20b868(w, self, [asegs[0], asegs[1]]),
                FieldBuild::Plain => jtblend::ee_sub_137a30(w, self, &asegs, &[]),
            };
            if let Some(b) = b {
                out.insert(field, b);
            }
        }
        out
    }
}

/// FUN_0012fe58 per channel: rig bone driven by each channel of `a` (None: target or bone unresolved).
pub fn bind_channels(w: &World, jt: usize, a: &Aseg, index: &HashMap<(usize, usize), usize>) -> Vec<Option<usize>> {
    a.channels
        .iter()
        .map(|c| {
            let t = ee_sub_15e938_below(w, jt, ((c.oid << 12) as i32) >> 12)?;
            match ee_sub_22d6b0(w, c.kind as i16, w.objects[t].owner, Some(t), BoneRef::None) {
                BoneRef::Bone(o, k) => index.get(&(o, k)).copied(),
                _ => None,
            }
        })
        .collect()
}

/// What the JT's ASEGA plays.
pub enum JtPlay {
    /// a direct ASEG member (object) through the ASEGA driver
    Aseg(usize, AsegaPlayer),
    /// a blend field: local time and speed (ASEGA +0x1c / +0x40)
    Blend { field: u32, t: f32, svt: f32 },
}

/// Running animation of the JT: the current ASEGA, its blends, the cross-fade source and the posed rig.
pub struct JtAnimator {
    pub rig: JtRig,
    pub blends: HashMap<u32, JtBlend>,
    /// blends built (JT post-load, done on the first request)
    pub built: bool,
    pub jump: JumpWeights,
    /// JT +0x18d0 (jtbs) read by the jump blend update (set by the caller each frame)
    pub jtbs: i32,
    pub current: Option<JtPlay>,
    /// bone locals at the last ASEG change and the fade time left
    pub from: Vec<Mat4>,
    pub fade: f32,
    /// WP-Q2: JT +0x1028 SM (oid 0x7c) walk: (SM object, target state of the playing link)
    pub sm: Option<(usize, Sma)>,
    /// post-load SMs by object (FUN_001efac0 tables)
    pub sms: HashMap<usize, Arc<Sm>>,
    /// rand() state for the SM's weighted transition picks (FUN_0011d448)
    pub rng: GameRng,
    /// the SM link ASEG ended (sm_tick enters the target state)
    pub sm_due: bool,
    /// WP-A4: FUN_001f13a8 signals of the last request_sm / sm_tick (the JT's own slot-4 handler for
    /// message 10 is not ported; consumers read them here)
    pub signals: Vec<SmaMsg>,
    /// WP-P17: two-bone IK records (FUN_00186670 / FUN_00186a50)
    pub ik: ik::JtIkSet,
    /// WP-P23: timed ASEG events fired by `step` (ASEG object, event index, reverse) for the FUN_00131290
    /// dispatch (playerdraw::plugin::dispatch_player_events takes them)
    pub fired: Vec<(usize, usize, bool)>,
    /// WP-P23: per blend field, its FUN_00137a30 event source (+0xd8 ASEG object, length, timed events1)
    pub blend_ev: HashMap<u32, (usize, f32, Vec<(usize, f32)>)>,
    /// WP-P24: JT +0x1d28 glider SMA (SM 0x5f6 on the glider model 0x5f5, FUN_00188608)
    pub glider: Option<JtSma2>,
}

/// WP-P24: a second SMA of the JT whose transition ASEGs play over the body pose (only the bones their
/// channels bind are written): JT +0x1d28 = FUN_001f0380(jt+0x1d24 SM, jt+0x1d20 actor, -1, 1) (mask 4).
#[derive(Clone)]
pub struct JtSma2 {
    pub sm: usize,
    pub sma: Sma,
    /// the playing transition ASEGA (ASEG object, player)
    pub play: Option<(usize, AsegaPlayer)>,
    /// the transition ASEGA ended (FUN_001f1078 due)
    pub due: bool,
}

/// WP-P24: JT_BIND_2BF1B0 rows 10 / 11: jt+0x1d20 glider model, jt+0x1d24 glider SM.
pub const OID_GLIDER_MODEL: i32 = 0x5f5;
pub const OID_GLIDER_SM: i32 = 0x5f6;
/// FUN_001f0380 passes t0 = 4 as the FUN_001f01b8 mask.
pub const SMA_MASK_1F0380: u32 = 4;

/// Unit quaternion (w, x, y, z) of an orthonormal 3x3 (rows) and back; any consistent pair works
/// for blending.
fn quat_of(m: &[[f32; 3]; 3]) -> [f32; 4] {
    let tr = m[0][0] + m[1][1] + m[2][2];
    let q = if tr > 0.0 {
        let s = (tr + 1.0).sqrt() * 2.0;
        [0.25 * s, (m[2][1] - m[1][2]) / s, (m[0][2] - m[2][0]) / s, (m[1][0] - m[0][1]) / s]
    } else if m[0][0] > m[1][1] && m[0][0] > m[2][2] {
        let s = (1.0 + m[0][0] - m[1][1] - m[2][2]).sqrt() * 2.0;
        [(m[2][1] - m[1][2]) / s, 0.25 * s, (m[0][1] + m[1][0]) / s, (m[0][2] + m[2][0]) / s]
    } else if m[1][1] > m[2][2] {
        let s = (1.0 + m[1][1] - m[0][0] - m[2][2]).sqrt() * 2.0;
        [(m[0][2] - m[2][0]) / s, (m[0][1] + m[1][0]) / s, 0.25 * s, (m[1][2] + m[2][1]) / s]
    } else {
        let s = (1.0 + m[2][2] - m[0][0] - m[1][1]).sqrt() * 2.0;
        [(m[1][0] - m[0][1]) / s, (m[0][2] + m[2][0]) / s, (m[1][2] + m[2][1]) / s, 0.25 * s]
    };
    let n = q.iter().map(|x| x * x).sum::<f32>().sqrt();
    q.map(|x| x / n)
}

fn mat_of(q: [f32; 4]) -> [[f32; 3]; 3] {
    let [w, x, y, z] = q;
    [
        [1.0 - 2.0 * (y * y + z * z), 2.0 * (x * y - z * w), 2.0 * (x * z + y * w)],
        [2.0 * (x * y + z * w), 1.0 - 2.0 * (x * x + z * z), 2.0 * (y * z - x * w)],
        [2.0 * (x * z - y * w), 2.0 * (y * z + x * w), 1.0 - 2.0 * (x * x + y * y)],
    ]
}

/// Cross-fade of two bone locals (REPLACED FUN_0012f9b8 blend-in): basis split into row scales and
/// rotation; rotation slerped, scales and translation lerped, so a fade never shrinks a limb (a
/// plain matrix lerp between rotations does).
pub fn blend_local(a: &Mat4, b: &Mat4, u: f32) -> Mat4 {
    let split = |m: &Mat4| {
        let s: [f32; 3] = std::array::from_fn(|i| (m[i][0] * m[i][0] + m[i][1] * m[i][1] + m[i][2] * m[i][2]).sqrt());
        let r: [[f32; 3]; 3] = std::array::from_fn(|i| std::array::from_fn(|j| if s[i] > 1e-6 { m[i][j] / s[i] } else { 0.0 }));
        (s, r)
    };
    let ((sa, ra), (sb, rb)) = (split(a), split(b));
    let det = |r: &[[f32; 3]; 3]| r[0][0] * (r[1][1] * r[2][2] - r[1][2] * r[2][1]) - r[0][1] * (r[1][0] * r[2][2] - r[1][2] * r[2][0]) + r[0][2] * (r[1][0] * r[2][1] - r[1][1] * r[2][0]);
    // squashed / mirrored / sheared bases (scale channels through zero): plain lerp
    if sa.iter().chain(&sb).any(|&s| s <= 1e-6) || det(&ra) < 0.99 || det(&rb) < 0.99 {
        return lerp_m(a, b, u);
    }
    let (qa, mut qb) = (quat_of(&ra), quat_of(&rb));
    let mut d: f32 = qa.iter().zip(&qb).map(|(x, y)| x * y).sum();
    if d < 0.0 {
        qb = qb.map(|x| -x);
        d = -d;
    }
    let q = if d > 0.9995 {
        let q: [f32; 4] = std::array::from_fn(|i| qa[i] + (qb[i] - qa[i]) * u);
        let n = q.iter().map(|x| x * x).sum::<f32>().sqrt();
        q.map(|x| x / n)
    } else {
        let th = d.acos();
        let (ka, kb) = (((1.0 - u) * th).sin() / th.sin(), (u * th).sin() / th.sin());
        std::array::from_fn(|i| qa[i] * ka + qb[i] * kb)
    };
    let r = mat_of(q);
    let mut o = lerp_m(a, b, u);
    for i in 0..3 {
        let s = sa[i] + (sb[i] - sa[i]) * u;
        for j in 0..3 {
            o[i][j] = r[i][j] * s;
        }
    }
    o
}

fn lerp_m(a: &Mat4, b: &Mat4, u: f32) -> Mat4 {
    std::array::from_fn(|i| std::array::from_fn(|j| a[i][j] + (b[i][j] - a[i][j]) * u))
}

impl JtAnimator {
    pub fn new(rig: JtRig) -> Self {
        let from = rig.pose.bones.iter().map(|b| b.local).collect();
        JtAnimator { rig, blends: HashMap::new(), built: false, jump: JumpWeights::default(), jtbs: 0, current: None, from, fade: 0.0, sm: None, sms: HashMap::new(), rng: GameRng::default(), sm_due: false, signals: vec![], ik: ik::JtIkSet::default(), fired: vec![], blend_ev: HashMap::new(), glider: None }
    }

    /// Field of the current ASEGA.
    pub fn current_field(&self) -> Option<u32> {
        match &self.current {
            Some(JtPlay::Blend { field, .. }) => Some(*field),
            Some(JtPlay::Aseg(o, _)) => self.rig.members.iter().find(|(_, v)| *v == o).map(|(k, _)| *k),
            None => None,
        }
    }

    /// One 0x18c250 ASEG request (player::JtAnimEvents entry).
    pub fn request(&mut self, w: &World, field: u32, jtbs: i32) {
        if !self.built {
            self.blends = self.rig.build_blends(w);
            self.built = true;
        }
        ik::ensure_built(self, w);
        self.jtbs = jtbs;
        // WP-P23: 0x18c250 jtbs 3 / 4 / 10 hold SM fields (class 111): FUN_001f03a8(field, -1) (a running SMA
        // of the same SM is kept, as for +0x1028)
        if let Some(oid) = sm_field_oid(field).filter(|_| self.rig.aseg_for(field).is_none()) {
            if ee_sub_15e938_below(w, self.rig.jt, oid).is_some() {
                return self.request_sm_oid(w, oid, -1);
            }
        }
        self.sm = None;
        let play = if let Some(b) = self.blends.get(&field) {
            if let (false, Some(src)) = (self.blend_ev.contains_key(&field), crate::engine::player::p23::blend_event_src(w, b)) {
                let a = aseg_of(w, src).expect("aseg");
                self.blend_ev.insert(field, (src, a.length, a.events1.iter().enumerate().filter_map(|(i, e)| e.time.map(|t| (i, t))).collect()));
            }
            // jtbs 0..2 keep a playing jump blend (iStack_110 = 1); its start: startf 0.5, a = b = 1
            if field == FIELD_JUMP {
                if matches!(self.current, Some(JtPlay::Blend { field: FIELD_JUMP, .. })) {
                    return;
                }
                self.jump = JumpWeights::default();
                JtPlay::Blend { field, t: b.length * 0.5, svt: 0.0 }
            } else {
                // run: start = jt+0x1a0 * length (+0x1a0 not tracked: 0)
                JtPlay::Blend { field, t: 0.0, svt: 1.0 }
            }
        } else {
            let Some(o) = self.rig.aseg_for(field) else { return };
            if matches!(&self.current, Some(JtPlay::Aseg(c, _)) if *c == o) {
                return;
            }
            let Some(a) = aseg_of(w, o) else { return };
            // FUN_0012f9b8 start at svt 1 (flags 0); looping replaced (restart on end)
            JtPlay::Aseg(o, AsegaPlayer::new(Arc::new(a.clone()), 0, 1.0))
        };
        self.from = self.rig.pose.bones.iter().map(|b| b.local).collect();
        self.fade = BLEND_TIME;
        self.current = Some(play);
    }

    /// 0x18c250 jts 0: FUN_001f03a8(JT +0x1028 SM, jt, label, flags 1, FUN_001430c8 mask) with label
    /// 0x175 (0x177 from jts 10, 0x176 from jts 0xc, 0x178/0x179 landing from jts 2; FUN_001f0400 -1
    /// -> none). FUN_001430c8: JT +0x10fc look mode 0 (no look-at target tracked) -> mask 8.
    pub fn request_sm(&mut self, w: &World, label: i32) {
        self.request_sm_oid(w, SM_JT_1028, label);
    }

    /// FUN_001f03a8(SM below the JT with oid `oid`, label): the 0x18c250 iStack_118 SM fields (JT +0x1028
    /// jts 0, WP-P23: +0x1950 jtbs 3, +0x1bc4 jtbs 4, +0x1f14 jtbs 10, all with label -1).
    pub fn request_sm_oid(&mut self, w: &World, oid: i32, label: i32) {
        ik::ensure_built(self, w);
        let Some(sm) = ee_sub_15e938_below(w, self.rig.jt, oid) else { return };
        if matches!(self.sm, Some((o, _)) if o == sm) {
            return;
        }
        let Some(m) = self.sm_of(w, sm) else { return };
        let label = if crate::engine::anim::sm::ee_sub_1f0400(&m.states, label) < 0 { -1 } else { label };
        let len = |o: usize| aseg_of(w, o).map_or(0.0, |a| a.length);
        let mut sma = Sma::ee_sub_1f01b8(&m, &mut self.rng, label, 1, 8, &len);
        self.signals = sma.dispatch(&m, None, &mut []);
        self.sm = Some((sm, sma));
        self.sm_play(w);
    }

    /// WP-P24: FUN_00188608 tail: JT +0x1d28 = FUN_001f0380(SM 0x5f6, model 0x5f5, -1, 1) when +0x1d24 is set.
    pub fn glider_init(&mut self, w: &World) {
        if self.glider.is_some() {
            return;
        }
        let Some(sm) = ee_sub_15e938_below(w, self.rig.jt, OID_GLIDER_SM) else { return };
        let Some(m) = self.sm_of(w, sm) else { return };
        let len = |o: usize| aseg_of(w, o).map_or(0.0, |a| a.length);
        let sma = Sma::ee_sub_1f01b8(&m, &mut self.rng, -1, 1, SMA_MASK_1F0380, &len);
        self.glider = Some(JtSma2 { sm, sma, play: None, due: false });
        self.glider_play(w);
    }

    /// WP-P24: 0x195fb8 FUN_001f0660(JT +0x1d28, label) (0x5f7 glide entry, 0x5f8 glide exit).
    pub fn glider_goal(&mut self, w: &World, label: i32) {
        self.glider_init(w);
        let Some(mut g) = self.glider.take() else { return };
        if let Some(m) = self.sm_of(w, g.sm) {
            let len = |o: usize| aseg_of(w, o).map_or(0.0, |a| a.length);
            let before = g.sma.asega.clone();
            g.sma.ee_sub_1f0660(&m, &mut self.rng, label, &len);
            let changed = g.sma.asega != before;
            self.glider = Some(g);
            if changed {
                self.glider_play(w);
            }
        } else {
            self.glider = Some(g);
        }
    }

    /// WP-P24: the glider transition ASEGA ended -> FUN_001f1078; call before `step`.
    pub fn glider_tick(&mut self, w: &World) {
        let Some(mut g) = self.glider.take() else { return };
        if std::mem::take(&mut g.due) {
            if let Some(m) = self.sm_of(w, g.sm) {
                let len = |o: usize| aseg_of(w, o).map_or(0.0, |a| a.length);
                g.sma.ee_sub_1f1078(&m, &mut self.rng, &len);
            }
            self.glider = Some(g);
            self.glider_play(w);
        } else {
            self.glider = Some(g);
        }
    }

    /// FUN_0012f9b8 for the glider SMA's transition ASEG (None: the last channel values stay on the bones).
    fn glider_play(&mut self, w: &World) {
        let Some(g) = self.glider.as_mut() else { return };
        g.play = None;
        let Some(p) = g.sma.asega.clone() else { return };
        let Some(a) = aseg_of(w, p.aseg) else { return };
        if !self.rig.bindings.contains_key(&p.aseg) {
            let b = bind_channels(w, self.rig.jt, a, &self.rig.index);
            self.rig.bindings.insert(p.aseg, b);
        }
        let mut pl = AsegaPlayer::new(Arc::new(a.clone()), p.flags, p.svt);
        pl.state.t = p.start;
        g.play = Some((p.aseg, pl));
    }

    /// FUN_001efac0 post-load tables of an SM object (cached).
    fn sm_of(&mut self, w: &World, sm: usize) -> Option<Arc<Sm>> {
        use crate::engine::world::loads_b4::B4Load;
        if let Some(m) = self.sms.get(&sm) {
            return Some(m.clone());
        }
        let Payload::Loaded(LoadData::B4(b)) = &w.objects[sm].payload else { return None };
        let B4Load::L1efac0 { nested, pairs, links, link_f, .. } = &**b else { return None };
        let nested: Vec<(usize, i32)> = nested.iter().map(|&o| (o, w.objects[o].oid)).collect();
        let m = Arc::new(Sm::ee_sub_1efac0(pairs, links, link_f, &nested));
        self.sms.insert(sm, m.clone());
        Some(m)
    }

    /// The SMA's transition ASEGA ended (FUN_001f1078); call before `step`.
    pub fn sm_tick(&mut self, w: &World) {
        if !std::mem::take(&mut self.sm_due) {
            return;
        }
        let Some((o, mut sma)) = self.sm.take() else { return };
        if let Some(m) = self.sm_of(w, o) {
            let len = |o: usize| aseg_of(w, o).map_or(0.0, |a| a.length);
            sma.ee_sub_1f1078(&m, &mut self.rng, &len);
            self.signals = sma.dispatch(&m, None, &mut []);
        }
        self.sm = Some((o, sma));
        self.sm_play(w);
    }

    /// FUN_0012f9b8 for the SMA's transition ASEG (start time, signed speed, flags).
    fn sm_play(&mut self, w: &World) {
        let Some((_, sma)) = &self.sm else { return };
        let Some(p) = sma.asega.clone() else { return };
        let Some(a) = aseg_of(w, p.aseg) else { return };
        if !self.rig.bindings.contains_key(&p.aseg) {
            let b = bind_channels(w, self.rig.jt, a, &self.rig.index);
            self.rig.bindings.insert(p.aseg, b);
        }
        let mut pl = AsegaPlayer::new(Arc::new(a.clone()), p.flags, p.svt);
        pl.state.t = p.start;
        self.from = self.rig.pose.bones.iter().map(|b| b.local).collect();
        self.fade = BLEND_TIME;
        self.current = Some(JtPlay::Aseg(p.aseg, pl));
    }

    /// Advance and pose (bone worlds in rest level space, `rig.pose.world()`). `jtbs` = JT +0x18d0.
    pub fn step(&mut self, dt: f32) {
        self.fired.clear();
        let jtbs = self.jtbs;
        for b in &mut self.rig.pose.bones {
            b.motion = 0;
        }
        match &mut self.current {
            Some(JtPlay::Aseg(o, p)) => {
                drive(p, AnimClock { dt, real_dt: dt });
                self.fired.extend(p.fired.iter().map(|&(i, r)| (*o, i, r)));
                if self.sm.is_some() {
                    // SMA transition: hold the end pose; sm_tick (FUN_001f1078) picks the next one
                    if p.ended {
                        self.sm_due = true;
                    }
                } else if HOLD_AT_END.contains(&self.rig.field_of(*o)) && p.state.t >= p.aseg.length {
                    // WP-P19: 0x195fb8 case 0x14 plays JMP[0x6e] (jt+0x478, unbound for Sly): FUN_001305b8
                    // with no ASEG leaves the glide-open end pose in place
                    p.state.t = p.aseg.length;
                } else if p.ended || p.state.t >= p.aseg.length {
                    let a = p.aseg.clone();
                    *p = AsegaPlayer::new(a, 0, 1.0);
                }
                let t = p.state.t;
                let segments = bind_segments(&p.aseg, t);
                if let Some(binding) = self.rig.bindings.get(o) {
                    self.rig.pose.bind(&segments, binding, t, 1.0);
                }
            }
            Some(JtPlay::Blend { field, t, svt }) => {
                let Some(b) = self.blends.get_mut(field) else { return };
                match *field {
                    // 0x18bce0 state 1: FUN_0020bb58(1.0, jt + 0x1320) every frame
                    FIELD_RUN => b.ee_sub_20bb58(1.0, std::slice::from_mut(t)),
                    FIELD_JUMP if dt > 0.0 => {
                        // lean target (+0x18c4 direction) not tracked: f = 0
                        if let Some(s) = jtblend::ee_sub_190520(b, &mut self.jump, jtbs, dt, 0.0, t) {
                            *svt = s;
                        }
                    }
                    _ => {}
                }
                let t0 = *t;
                *t += dt * *svt;
                let forward = *t >= t0;
                if b.length > 0.0 {
                    if *field == FIELD_JUMP {
                        *t = t.clamp(0.0, b.length);
                    } else {
                        *t = t.rem_euclid(b.length);
                    }
                }
                // WP-P23: the blend's shared event list (FUN_001384c0 times * length / source length); a
                // backward play (jump svt < 0) fires nothing here (UNPORTED: reverse cursor)
                if let (true, Some((src, len, timed))) = (forward && *t != t0, self.blend_ev.get(field)) {
                    let k = if *len > 0.0 { b.length / len } else { 1.0 };
                    let scaled: Vec<(usize, f32)> = timed.iter().map(|&(i, x)| (i, x * k)).collect();
                    self.fired.extend(crate::engine::player::p23::window(&scaled, t0, *t).into_iter().map(|i| (*src, i, false)));
                }
                let (segments, binding) = b.segments(*t);
                self.rig.pose.bind(&segments, &binding, *t, 1.0);
            }
            None => {}
        }
        // WP-P24: the glider SMA's ASEGA writes its bound bones over the body pose (and out of the fade)
        if let Some((o, p)) = self.glider.as_mut().and_then(|g| g.play.as_mut().map(|(o, p)| (*o, p))) {
            drive(p, AnimClock { dt, real_dt: dt });
            self.fired.extend(p.fired.iter().map(|&(i, r)| (o, i, r)));
            let ended = p.ended;
            let t = p.state.t.clamp(0.0, p.aseg.length);
            let segments = bind_segments(&p.aseg, t);
            if let Some(binding) = self.rig.bindings.get(&o) {
                self.rig.pose.bind(&segments, binding, t, 1.0);
                for &b in binding.iter().flatten() {
                    if let (Some(f), Some(x)) = (self.from.get_mut(b), self.rig.pose.bones.get(b)) {
                        *f = x.local;
                    }
                }
            }
            if ended {
                if let Some(g) = self.glider.as_mut() {
                    g.due = true;
                }
            }
        }
        self.rig.pose.compose(&IDENTITY);
        if self.fade > 0.0 {
            self.fade = (self.fade - dt).max(0.0);
            let u = 1.0 - self.fade / BLEND_TIME;
            let locals: Vec<Mat4> = self.rig.pose.bones.iter().zip(&self.from).map(|(b, f)| blend_local(f, &b.local, u)).collect();
            let parents: Vec<Option<usize>> = self.rig.pose.bones.iter().map(|b| b.parent).collect();
            let worlds = crate::engine::anim::pose::world_matrices(&IDENTITY, &locals, &parents);
            for ((b, l), wm) in self.rig.pose.bones.iter_mut().zip(locals).zip(worlds) {
                b.local = l;
                b.world = wm;
            }
        }
        // WP-P17: FUN_0022de88 mode 0xd bones -> FUN_00186a50
        ik::apply(self);
    }
}

/// WP-P22: the two wrist bones of Sly's JT skeleton (parents of the finger chains: 63 -> 64/67/70/120,
/// 73 -> 74/80/83/86/89/92/96/100/104/107/111).
pub const JT_WRIST_BONES: [usize; 2] = [63, 73];

/// WP-P22 (REPLACED, ledge hang draw): jtbs 0x11 holds the JT body at the hang point = contact - JMP+0xf0
/// (player::p4::ledge_hang_point, 1:1), i.e. the ledge contact is at JMP+0xf0 = (37, 0, 25) in the JT frame
/// (x = facing). The hang ASEG JMP[0x55] (jt+0x414) leaves the root / pelvis unanimated (pelvis +75 over the
/// feet origin, wrists ~+150), so drawn at the body origin the torso rises above the ledge. The original's
/// draw-side mechanism is not found; here the drawn JT tree is shifted (model space) so the wrists' midpoint
/// lies on the contact. `worlds` = posed bone worlds from the identity root (`rig.pose.world()`).
pub fn hang_grip_offset(rig: &JtRig, worlds: &[Mat4]) -> Option<[f32; 3]> {
    let mut g = [0.0f32; 3];
    for k in JT_WRIST_BONES {
        let b = *rig.index.get(&(rig.jt, k))?;
        for i in 0..3 {
            g[i] += worlds.get(b)?[3][i] * 0.5;
        }
    }
    let c = crate::engine::player::p4::JMP_F0;
    Some([c[0] - g[0], c[1] - g[1], c[2] - g[2]])
}

/// WP-P24: jmp[0x6f], the JT's glider camera track oid.
pub const OID_GLIDE_TRACK: i32 = 0x5f4;

/// WP-P24: the glider track (class-15 object 0x5f4 below the JT) as (pv = object + 1, C2db858 view).
pub fn glide_track(w: &World) -> Option<(u32, crate::engine::camera::follow::FollowTrack)> {
    use crate::engine::world::loads_b4::B4Load;
    let o = ee_sub_15e938_below(w, find_jt(w)?, OID_GLIDE_TRACK)?;
    let crate::engine::world::objects::Payload::Loaded(crate::engine::world::objects::LoadData::B4(b)) = &w.objects[o].payload else { return None };
    let B4Load::L220fd0 { props, nested, .. } = &**b else { return None };
    // +0x10c path: a nested path object (none on the glider tracks seen)
    Some((o as u32 + 1, crate::engine::camera::follow::follow_track(props, !nested.is_empty())))
}
