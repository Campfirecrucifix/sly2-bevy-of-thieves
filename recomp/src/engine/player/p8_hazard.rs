//! WP-P8: hazard volumes that hurt Sly (water, class 89 C2dca30) and the hit delivery path.
//!
//! Original flow: the water object (class 89, vtable 0x2dca30, init 0x228b48: +0x184 = -10 amount, +0x188 =
//! 0x2000 kind) walks its physics contacts each frame (0x228e50, submerged test 0x229468) and sends msg 4
//! {water, obj} on enter / 5 on leave; the JT handler 0x140f90 stores the water in jt+0x108c (case 4) and
//! clears it (case 5). The JT update 0x1403d0 then runs 0x1a2460: point = position + (0, 0, jt+0x10a0 = 50,
//! minus 10 in damage states 3 / 10); when the water's slot 72 (0x229d78 = 0x1f72f8 point-in-volume) holds
//! it and water+0x184 != 0 and (v.z < 0 or (jtbs == -1 and damage state != 7)) a 0x60-byte hit {src water,
//! target jt, kind +0x188, amount +0x184, pos, velocity} goes through 0x1f7598.
//! 0x1f7598: obj = 0x1a5bf0(target) (target when is-a ALO, else owner), walking up the owner chain: drop
//! duplicates of the world hit list 0x3e15c0 (same src, +4, kind, obj), xp+0x44 = 0x1f78d8(obj, kind),
//! slot 74 TakeDamage (JT: 0x140f50 -> p7 0x1a19a0); accepted -> copy into the list with t = DAT_002dded0
//! (expired after 0.25 s by 0x1f77c0 in the world update 0x226550) and, when xp+0x44 & 0xf8, msgs 0xb.
//!
//! 1:1: 0x1a2460 (damage half), 0x1f7598, 0x1f78d8, 0x1f77c0, 0x1f72f8, 0x1a5bf0 (JT case), water init
//! defaults 0x228b48 and props 876 (+0x184) / 874 (+0x170) / 875 (+0x174), ALO part flags of 0x1f31e0.
//! REPLACED: the enter / leave contact walk 0x228e50 + 0x229468 (hull vs BSP) by the same point test on the
//! JT position or the 0x1a2460 sample point (`water_contacts`); part geometry is transformed once at load
//! with the rest-pose bone world (water does not move); ALO +0x110 kind table (0x1f78d8 input) = zeros
//! (loader not traced), so the msg 0xb / vt+0x1c notifications are never sent; the HitSrc of a water hit
//! uses bone 0's world position as FUN_001a62e8(water).
//! UNPORTED (recorded in `Hazards::calls`): the ripple / splash half of 0x1a2460 (0x1a2788 surface ray,
//! ripple emitters jt+0x1090 / +0x1094 every 0.05 s, vt+0x44), the splash sound 0x1fbb90(0x13) of
//! 0x228e50, effect 0x156698, other hazard classes (lasers / spikes reach slot 74 through their own
//! updates, not traced here).
#![allow(non_snake_case)]
use super::{Jt, JtAnim};
use crate::engine::gfx::attach::{ee_sub_22d6b0, BoneRef, BoneWorlds};
use crate::engine::gfx::scene::{Affine, IDENTITY};
use crate::engine::physics::geom::{build_bsp, build_geom, BspC, CGeom, FUN_00166270, V3};
use crate::engine::physics::point::FUN_00118fb0;
use crate::engine::world::class::ClassId;
use crate::engine::world::objects::{LoadData, Payload};
use crate::engine::world::props::PropValue;
use crate::engine::world::World;

/// class 89 (C2dca30, vtable 0x2dca30)
pub const CID_WATER: ClassId = 89;
/// 0x228b48 water+0x184 default amount
pub const WATER_AMOUNT: i32 = -10;
/// 0x228b48 water+0x188 default kind
pub const WATER_KIND: i32 = 0x2000;
/// 0x188488 jt+0x10a0 (0x42480000)
pub const JT_10A0: f32 = 50.0;
/// 0x1f77c0 hit record lifetime
pub const HIT_TTL: f32 = 0.25;
/// ALO part +0x10 flag bits (0x1f31e0)
pub const PART_SPHERE: u64 = 0x2000000000000;
pub const PART_SKIP: u64 = 0x4000000000000;

/// One 0xb0-byte ALO collision part as 0x1f72f8 reads it.
#[derive(Clone, Debug)]
pub struct VolPart {
    /// +0x10
    pub flags: u64,
    /// *(part+0) + 0x70: bone world position
    pub centre: V3,
    /// part[2]
    pub radius: f32,
    /// geometry + BSP (part[0x1d] absp; None when the part has no BSP)
    pub bsp: Option<(CGeom, BspC)>,
}

/// A water object (class 89).
#[derive(Clone, Debug)]
pub struct Water {
    pub obj: usize,
    pub oid: i32,
    pub isa: Vec<i32>,
    /// FUN_001a62e8(water)
    pub pos: V3,
    pub parts: Vec<VolPart>,
    /// +0x184
    pub amount: i32,
    /// +0x188
    pub kind: i32,
    /// +0x170 splash emitter oid (prop 874)
    pub e170: i32,
    /// +0x174 ripple emitter oid (prop 875)
    pub e174: i32,
}

impl Water {
    /// 0x001f72f8 (slot 72): point in any non-skipped part (sphere or BSP solid).
    pub fn FUN_001f72f8(&self, p: V3) -> bool {
        for pt in &self.parts {
            if pt.flags & PART_SKIP != 0 {
                continue;
            }
            if pt.flags & PART_SPHERE != 0 {
                let d = [p[0] - pt.centre[0], p[1] - pt.centre[1], p[2] - pt.centre[2]];
                if d[0] * d[0] + d[1] * d[1] + d[2] * d[2] < pt.radius * pt.radius {
                    return true;
                }
            }
            if let Some((g, b)) = &pt.bsp {
                if !b.nodes.is_empty() && FUN_00118fb0(g, b, 0, 0x7fffffff, p).is_some() {
                    return true;
                }
            }
        }
        false
    }
}

/// 0x3e15c0 record (copy of the 0x60-byte hit, fields the dedupe reads).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HitRec {
    /// +0 source (water index)
    pub src: usize,
    pub w4: i32,
    /// +0xc
    pub kind: i32,
    /// +0x48 object that took it (0 = the JT)
    pub obj: usize,
    /// +0x4c DAT_002dded0
    pub t: f32,
}

/// The hit hazards of a level + the world hit list.
#[derive(Clone, Debug, Default)]
pub struct Hazards {
    pub waters: Vec<Water>,
    /// 0x3e15c0
    pub hits: Vec<HitRec>,
    /// unported calls (address, arg)
    pub calls: Vec<(u32, i32)>,
}

fn class_isa(c: ClassId) -> Vec<i32> {
    let mut v = Vec::new();
    let mut ci = crate::engine::world::class::class(c);
    while let Some(i) = ci {
        v.push(i.id as i32);
        ci = if i.parent < 0 { None } else { crate::engine::world::class::class(i.parent as u16) };
    }
    v
}

fn prop_i32(props: &[crate::engine::world::props::Prop], field: i16) -> Option<i32> {
    props.iter().rev().find(|p| p.field == field).and_then(|p| match &p.value {
        PropValue::Scalar(b) if b.len() >= 4 => Some(i32::from_le_bytes([b[0], b[1], b[2], b[3]])),
        _ => None,
    })
}

impl Hazards {
    /// Water objects of a loaded world (0x228b48 defaults + props, parts of 0x1f31e0).
    pub fn load(w: &World) -> Self {
        let mut bw = BoneWorlds::default();
        let mut waters = Vec::new();
        for (i, o) in w.objects.iter().enumerate() {
            if o.class != CID_WATER {
                continue;
            }
            let Payload::Loaded(LoadData::Alo { alo, .. }) = &o.payload else { continue };
            let mut parts = Vec::new();
            for p in &alo.parts {
                let m: Affine = match ee_sub_22d6b0(w, p.r#ref, o.owner, Some(i), BoneRef::None) {
                    BoneRef::Bone(ob, k) => bw.get(w, ob, k),
                    _ => IDENTITY,
                };
                let flags = ((p.b0 as u64 & 1) << 48) | ((p.b1 as u64 & 1) << 49) | ((p.b2 as u64 & 1) << 50);
                let bsp = (!p.bsp.nodes.is_empty()).then(|| (FUN_00166270(&build_geom(&p.geom), Some(&m)), build_bsp(&p.bsp)));
                parts.push(VolPart { flags, centre: m[3], radius: p.f, bsp });
            }
            let pos = if alo.base.bones.is_empty() { [0.0; 3] } else { bw.get(w, i, 0)[3] };
            waters.push(Water {
                obj: i,
                oid: o.oid as i32,
                isa: class_isa(o.class),
                pos,
                parts,
                amount: prop_i32(&alo.props, 876).unwrap_or(WATER_AMOUNT),
                kind: WATER_KIND,
                e170: prop_i32(&alo.props, 874).unwrap_or(0),
                e174: prop_i32(&alo.props, 875).unwrap_or(0),
            });
        }
        Self { waters, hits: Vec::new(), calls: Vec::new() }
    }

    /// 0x001f77c0: drop records older than 0.25 s.
    pub fn FUN_001f77c0(&mut self, now: f32) {
        self.hits.retain(|r| !(r.t + HIT_TTL <= now));
    }
}

/// WP-P8 JT state of the hazard code.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct JtP8 {
    /// jt+0x108c water (index into `Hazards::waters`)
    pub water: Option<usize>,
    /// jt+0x10a0
    pub off_10a0: f32,
    /// ALO +0x110 kind bytes (0x1f78d8 input; REPLACED: zeros)
    pub zap_110: [u8; 26],
}

impl Default for JtP8 {
    fn default() -> Self {
        Self { water: None, off_10a0: JT_10A0, zap_110: [0; 26] }
    }
}

/// 0x001f78d8: OR of the +0x110 bytes of every kind bit < 26.
pub fn FUN_001f78d8(t: &[u8; 26], kind: u32) -> u8 {
    (0..26).filter(|i| kind & (1 << i) != 0).fold(0, |a, i| a | t[i])
}

/// 0x60-byte hit as 0x1a2460 builds it.
#[derive(Clone, Debug, PartialEq)]
pub struct Xp {
    pub src: usize,
    pub w4: i32,
    pub kind: i32,
    pub amount: i32,
    pub pos: V3,
    pub vel: V3,
    /// +0x44
    pub zap: u8,
}

/// 0x001f7598 with the JT as target (0x1a5bf0: JT is-a ALO -> itself; no owner above it).
/// Returns true when the JT took the hit (the original returns the object).
pub fn FUN_001f7598(jt: &mut Jt, hz: &mut Hazards, xp: &mut Xp, now: f32, anim: &mut dyn JtAnim) -> bool {
    if hz.hits.iter().any(|r| r.src == xp.src && r.w4 == xp.w4 && r.kind == xp.kind && r.obj == 0) {
        return false;
    }
    xp.zap = FUN_001f78d8(&jt.p8.zap_110, xp.kind as u32);
    let w = &hz.waters[xp.src];
    let hit = super::p7_damage::Hit {
        src: Some(super::p7_damage::HitSrc { isa: w.isa.clone(), pos: w.pos, ..Default::default() }),
        kind: xp.kind,
        amount: xp.amount,
        vec: xp.vel,
    };
    let env = jt.dmg_env;
    // slot 74 0x140f50 -> 0x1a19a0
    if !super::p7_damage::FUN_001a19a0(jt, &hit, &env, anim) {
        return false;
    }
    hz.hits.push(HitRec { src: xp.src, w4: xp.w4, kind: xp.kind, obj: 0, t: now });
    if xp.zap & 0xf8 != 0 {
        hz.calls.push((0x001f7598, 0xb)); // msgs 0xb / vt+0x1c 10 / 0xb
    }
    true
}

/// 0x1a2460 sample point.
fn sample_point(jt: &Jt) -> V3 {
    let mut p = jt.phys.pos;
    p[2] += jt.p8.off_10a0;
    if jt.dmg.state == 3 || jt.dmg.state == 10 {
        p[2] -= 10.0;
    }
    p
}

/// 0x001a2460 (damage half; the ripple half is recorded).
pub fn FUN_001a2460(jt: &mut Jt, hz: &mut Hazards, now: f32, anim: &mut dyn JtAnim) {
    let Some(wi) = jt.p8.water else { return };
    let p = sample_point(jt);
    if !hz.waters[wi].FUN_001f72f8(p) {
        return;
    }
    if hz.waters[wi].amount != 0 {
        let v = jt.velocity_pub();
        let hit = v[2] < 0.0 || (jt.jtbs == -1 && jt.dmg.state != 7);
        if hit {
            let w = &hz.waters[wi];
            let mut xp = Xp { src: wi, w4: 0, kind: w.kind, amount: w.amount, pos: jt.phys.pos, vel: v, zap: 0 };
            FUN_001f7598(jt, hz, &mut xp, now, anim);
        }
    }
    if hz.waters[wi].e174 != 0 {
        hz.calls.push((0x001a2788, wi as i32)); // surface ray + ripple emitters
    }
}

/// REPLACED 0x228e50 / 0x229468 / 0x140f90 cases 4 and 5: jt+0x108c = the water holding the JT position or
/// the 0x1a2460 sample point.
pub fn water_contacts(jt: &mut Jt, hz: &Hazards) {
    let p = sample_point(jt);
    let c = jt.phys.pos;
    jt.p8.water = hz.waters.iter().position(|w| w.FUN_001f72f8(c) || w.FUN_001f72f8(p));
}

/// Per frame after `Jt::update` (world update 0x226550 order: 0x1f77c0, water updates, JT 0x1a2460).
pub fn hazard_step(jt: &mut Jt, hz: &mut Hazards, now: f32, anim: &mut dyn JtAnim) {
    hz.FUN_001f77c0(now);
    water_contacts(jt, hz);
    FUN_001a2460(jt, hz, now, anim);
}

/// WP-P8: job manager events raised by the JT since `from` (DmgEv::Event: 0xf damaged, 0x10 died) ->
/// 0x1619a0 job half (0x161a58 case 0x10 is p7's `Respawn` when no job is current).
pub fn job_events(jt: &Jt, from: usize, jobs: &mut crate::engine::jobs::JobMgr) {
    for e in jt.dmg.ev.iter().skip(from) {
        if let super::p7_damage::DmgEv::Event(k) = e {
            jobs.FUN_001619a0(*k, 0);
        }
    }
}
