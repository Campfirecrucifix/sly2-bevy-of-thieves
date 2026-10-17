//! WP-K7: guard vision occlusion. Vision blockers (world class 23, vtable 0x2dc210, the DAT_003e1480 list
//! fed by its slots 22 / 21), the eye -> target line-of-sight raycast FUN_001545f0 and the player
//! concealment test FUN_00154770, plus the root-motion sample FUN_00204e38 the walk task uses.
use crate::engine::anim::aseg::Aseg;
use crate::engine::anim::keys::{KeyVB, VecCurve};
use crate::engine::gfx::attach::BoneWorlds;
use crate::engine::gfx::scene::{invert, Affine};
use crate::engine::physics::bsp::FUN_00118668;
use crate::engine::physics::geom::{build_bsp, build_geom, BspC, CGeom, V3};
use crate::engine::physics::query::CollisionWorld;
use crate::engine::world::objects::{LoadData, Payload};
use crate::engine::world::World;

/// world class of the vision blockers (0x2dc210)
pub const CLASS_BLOCKER: u16 = 23;

/// One DAT_003e1480 entry: the blocker's +0xec BSP with its world -> local transform.
#[derive(Clone, Debug)]
pub struct Blocker {
    pub obj: usize,
    pub inv: Affine,
    pub geom: CGeom,
    pub bsp: BspC,
}

fn xf(m: &Affine, p: V3) -> V3 {
    [0, 1, 2].map(|j| p[0] * m[0][j] + p[1] * m[1][j] + p[2] * m[2][j] + m[3][j])
}

/// DAT_003e1480 DLIST (link +0x130). Slot 22 FUN_00227e20 (add to world: FUN_001a6178 then
/// FUN_0011a220(0x3e1480, obj)) / slot 21 FUN_00227e58 (remove: FUN_0011a2a8). REPLACED: the list is
/// built once from every placed class-23 object (they are all added to the world at load).
#[derive(Clone, Debug, Default)]
pub struct Blockers(pub Vec<Blocker>);

impl Blockers {
    pub fn build(w: &World) -> Blockers {
        let mut bw = BoneWorlds::default();
        let mut v = Blockers::default();
        for (i, o) in w.objects.iter().enumerate() {
            if o.class == CLASS_BLOCKER {
                v.ee_sub_227e20(w, &mut bw, i);
            }
        }
        v
    }
    /// slot 22 FUN_00227e20: link into DAT_003e1480
    pub fn ee_sub_227e20(&mut self, w: &World, bw: &mut BoneWorlds, i: usize) {
        if self.0.iter().any(|b| b.obj == i) {
            return;
        }
        let solid = match &w.objects[i].payload {
            Payload::Loaded(LoadData::Solid(s)) => s,
            Payload::Loaded(LoadData::L2282b0 { solid, .. }) => solid,
            _ => return,
        };
        let Some(m) = bw.xfm_world(w, i) else { return };
        self.0.push(Blocker { obj: i, inv: invert(&m), geom: build_geom(&solid.geom), bsp: build_bsp(&solid.bsp) });
    }
    /// slot 21 FUN_00227e58: unlink
    pub fn ee_sub_227e58(&mut self, i: usize) {
        self.0.retain(|b| b.obj != i);
    }
    /// FUN_001541d8 blocker loop: FUN_00118668(blk +0xec, 0x7fffffff, local(eye), local(tp), 4, buf) per
    /// blocker (local = FUN_001a6f00(0, blk, p)); any record -> blocked.
    pub fn blocked(&self, eye: V3, tp: V3) -> bool {
        self.0.iter().any(|b| {
            let mut out = Vec::new();
            !b.bsp.nodes.is_empty() && FUN_00118668(&b.geom, &b.bsp, 0, usize::MAX, xf(&b.inv, eye), xf(&b.inv, tp), 4, &mut out) > 0
        })
    }
}

/// FUN_00154568 (default LOS filter): skip the sensor's own root (+0x14) and anything under an is-a 0x33
/// object (characters). The entry +0x10 bit 48 / owner +0xec->+0x70 bit 2 skips have no world-model source
/// (never set by level data that is ported).
pub fn ee_sub_154568(w: &World, guard: usize, obj: usize) -> bool {
    let root = |mut k: usize| {
        for _ in 0..64 {
            match w.objects[k].owner {
                Some(o) => k = o,
                None => break,
            }
        }
        k
    };
    if root(obj) == root(guard) {
        return false;
    }
    let mut k = Some(obj);
    for _ in 0..64 {
        let Some(i) = k else { break };
        if crate::engine::player::p4::FUN_00132b10(w.objects[i].class as i32, 0x33) {
            return false;
        }
        k = w.objects[i].owner;
    }
    true
}

/// FUN_001545f0(sensor, t): eye -> target LOS. FUN_0020faa0 collects the collision entries whose bbox
/// overlaps the segment bbox (filter FUN_00154568, or FUN_00154528 = class exactly 0x31 when sensor +0xf8
/// & 4), FUN_0014cc58 clips the segment against each (FUN_0014c6d8 -> FUN_00118668, max 2) -> any record
/// = blocked. `pt` = FUN_001a6f00(t, 0, 0) (target origin). The collision entries are the SO parts
/// (`CollisionWorld::build_so`).
pub fn ee_sub_1545f0(w: &World, cw: &CollisionWorld, guard: usize, eye: V3, pt: V3, f8: u32) -> bool {
    let lo = [0, 1, 2].map(|i| eye[i].min(pt[i]));
    let hi = [0, 1, 2].map(|i| eye[i].max(pt[i]));
    let mut n = 0;
    for c in &cw.colliders {
        if c.bsp.nodes.is_empty() || c.geom.verts.is_empty() {
            continue;
        }
        // FUN_0020faa0 bbox test
        let (mut a, mut b) = ([f32::MAX; 3], [f32::MIN; 3]);
        for v in &c.geom.verts {
            for i in 0..3 {
                a[i] = a[i].min(v[i]);
                b[i] = b[i].max(v[i]);
            }
        }
        if (0..3).any(|i| b[i] < lo[i] || a[i] > hi[i]) {
            continue;
        }
        let ok = if f8 & 4 != 0 { w.objects[c.object].class == 0x31 } else { ee_sub_154568(w, guard, c.object) };
        if !ok {
            continue;
        }
        n += 1;
        if n > 0x100 {
            break;
        }
        let mut out = Vec::new();
        if FUN_00118668(&c.geom, &c.bsp, 0, c.bsp.nodes.len(), eye, pt, 2, &mut out) > 0 {
            return true;
        }
    }
    false
}

/// Player attach / cover state FUN_00154770 reads (jt +0x4c8 / +0x4cc / +0x4d4 / +0x1040).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Cover {
    /// +0x4c8 attach state, +0x4cc attach sub-state (-1 none)
    pub st: i32,
    pub sub: i32,
    /// +0x4d4 attach object: is-a 0xc / is-a 0x22, its +0x194 (0xc) or +0xe4 field
    pub att_c: bool,
    pub att_22: bool,
    pub att_v: u32,
    /// +0x1040
    pub f_1040: i32,
    /// FUN_001a6318(t) facing row
    pub dir: V3,
}

/// FUN_00154770(sensor, t): the player (is-a 0x34) is concealed from the sensor at `spos`.
pub fn ee_sub_154770(c: &Cover, tpos: V3, spos: V3) -> bool {
    if c.sub == -1 {
        return false;
    }
    match c.st {
        0 | 0xd => {
            if !(c.att_c || c.att_22) {
                return c.att_v == 0;
            }
            if c.att_v != 0 {
                return false;
            }
            let d = (0..3).map(|i| (tpos[i] - spos[i]) * c.dir[i]).sum::<f32>();
            let b = c.st == 0 && c.f_1040 == 0;
            (d <= 0.0) != b
        }
        1 => c.sub == 0,
        2 => true,
        8 => (c.sub.wrapping_sub(9) as u32) < 2,
        _ => false,
    }
}

/// FUN_00204e38(t, guard, task): root velocity of the anim task's ASEG = FUN_0012ff10(aseg, guard, 1, 1)
/// channel (flagged vector channel whose target bone kind is 1: the guard skeletons' shared ASEGs target the
/// skeleton-local oid, not the guard's), curve slot 2 (vt+0x1c) at (t, rate) with the velocity output.
/// Bezier curves give the velocity directly (KeyVB::eval); REPLACED for the other curve kinds (no velocity
/// output ported): central difference of the value over 1/240 s.
pub fn ee_sub_204e38(a: &Aseg, t: f32, rate: f32) -> V3 {
    let Some(v) = a.channels.iter().find(|c| c.flag && c.kind == 1).and_then(|c| c.vec.as_ref()) else { return [0.0; 3] };
    match v {
        VecCurve::Bez(k) => KeyVB::eval(k, t, rate).1,
        _ => {
            const H: f32 = 1.0 / 480.0;
            let (Some(p0), Some(p1)) = (v.eval((t - H).max(0.0), rate), v.eval((t + H).min(a.length.max(t + H)), rate)) else { return [0.0; 3] };
            let dt = (t + H).min(a.length.max(t + H)) - (t - H).max(0.0);
            [0, 1, 2].map(|i| (p1[i] - p0[i]) / dt)
        }
    }
}

/// Guard anim table (+0x4c0, field 243 entries {id list, oid (item kind 6), flags}): oid of anim `id`.
pub fn anim_oid(w: &World, guard: usize, id: i32) -> Option<i32> {
    use crate::engine::world::props::{ArrayItem, PropValue};
    let o = &w.objects[guard];
    let props = w.objects.get(o.clone_of.unwrap_or(guard)).and_then(crate::engine::game::fields::object_props)?;
    let word = |b: &Vec<u8>| b.get(..4).map(|x| i32::from_le_bytes([x[0], x[1], x[2], x[3]]));
    for p in props.iter().filter(|p| p.field == 243) {
        let PropValue::Array(items) = &p.value else { continue };
        let ids: Vec<i32> = items.iter().filter_map(|i| if let ArrayItem::List(_, l) = i { Some(l.iter().filter_map(word).collect::<Vec<_>>()) } else { None }).next().unwrap_or_default();
        if ids.as_slice() != [id] {
            continue;
        }
        return items.iter().find_map(|i| if let ArrayItem::One(6, b) = i { word(b) } else { None });
    }
    None
}

/// FUN_00209618 (walk task 0x9d branch) for the guard at object `guard`: anim `id` from the table, its ASEG
/// (FUN_0015e938 oid lookup), looped at the playback time `t` -> FUN_00204e38 velocity v (+0x190 = -v; the
/// port's Body::f_190 keeps v). The +0xfc4 speed record (`*(+8)` scale) is 1.
pub fn root_velocity(w: &World, guard: usize, id: i32, t: f32, rate: f32) -> V3 {
    let Some(oid) = anim_oid(w, guard, id) else { return [0.0; 3] };
    let Some(a) = w.find_oid(oid).iter().find_map(|&k| crate::engine::playerdraw::aseg_of(w, k)) else { return [0.0; 3] };
    let t = if a.length > 0.0 { t.rem_euclid(a.length) } else { 0.0 };
    ee_sub_204e38(a, t, rate)
}

/// K32: length of anim `id` of the guard at object `guard` (the table ASEG as `root_velocity` finds it;
/// 0 = no ASEG: the anim goal reaches state 1 at once)
pub fn anim_length(w: &World, guard: usize, id: i32) -> f32 {
    let Some(oid) = anim_oid(w, guard, id) else { return 0.0 };
    w.find_oid(oid).iter().find_map(|&k| crate::engine::playerdraw::aseg_of(w, k)).map_or(0.0, |a| a.length)
}
