//! WP-P15: follow-camera collision input (consumed by playerdraw::plugin::SharedFollow, the FollowWorld of the
//! boxed CplcyFollow): segment queries (0x14ea08 / 0x2236e8 occlusion) and the 0x14f110 sphere sweep over the
//! player's SO collision set.
#![allow(non_snake_case)]

use crate::engine::camera::collide::CmSegmentQuery;
use crate::engine::physics::CollisionWorld;
use crate::engine::gfx::attach::{affine_from_bytes, BoneWorlds};
use crate::engine::gfx::scene::{mul, Affine};
use crate::engine::world::World;
use std::sync::Arc;

/// WP-P15: what the follow camera collides with: the player's SO collision set (PlayerWorldData::cw,
/// p10::player_collision_world = the objects the 0x20faa0 gather of 0x14f110 / 0x2236e8 walks; REPLACED: the
/// gather callback 0x14f030 filter), Cm +0x48 (s_near_radius, the 0x14f040 callback radius) and DAT_002ddee0.
#[derive(Clone, Default)]
pub struct CamCollide {
    pub cw: Option<Arc<CollisionWorld>>,
    pub near_radius: f32,
    pub dt_real: f32,
}

/// CmSegmentQuery over the shared collision world (0x14ea08 -> CollisionWorld::clip_segment).
pub struct CwQuery(pub Arc<CollisionWorld>);
impl CmSegmentQuery for CwQuery {
    fn solid_intervals(&mut self, start: [f32; 3], end: [f32; 3]) -> Vec<(f32, f32)> {
        self.0.clip_segment(start, end).iter().map(|l| (l.t[0], l.t[1])).collect()
    }
}

/// 0x0014f110 sweep: FUN_0020faa0(box = from + DAT_002bd8f0 .. from + DAT_002bd900, +-COLLIDE_BOX) gathers the
/// objects near the camera, FUN_0014eda0(radius, max_len, ..., callback 0x14f040). REPLACED gather: collider
/// vertex AABB vs the box.
pub fn cam_sweep_14f110(c: &CamCollide, radius: f32, max_len: f32, start: [f32; 3], vel: [f32; 3]) -> [f32; 3] {
    use crate::engine::camera::follow::COLLIDE_BOX;
    use crate::engine::physics::dynamics::FUN_0014f040;
    use crate::engine::physics::sphere::FUN_0014eda0;
    let Some(cw) = &c.cw else { return vel };
    let near = CollisionWorld {
        colliders: cw
            .colliders
            .iter()
            .filter(|col| {
                if col.geom.verts.is_empty() {
                    return false;
                }
                let (mut bl, mut bh) = (col.geom.verts[0], col.geom.verts[0]);
                for v in &col.geom.verts {
                    for k in 0..3 {
                        bl[k] = bl[k].min(v[k]);
                        bh[k] = bh[k].max(v[k]);
                    }
                }
                (0..3).all(|k| bh[k] >= start[k] - COLLIDE_BOX && bl[k] <= start[k] + COLLIDE_BOX)
            })
            .cloned()
            .collect(),
    };
    if near.colliders.is_empty() {
        return vel;
    }
    let (nr, dt) = (c.near_radius, c.dt_real);
    FUN_0014eda0(&near, radius, max_len, start, vel, &mut |cc, v, s| FUN_0014f040(nr, dt, cc, v, s))
}


/// WP-P17: 0x00223a70's push-out: 0x0014eda0(Cm +0x48, Cm +0x48, .., callback 0x00223a10, centre, centre) over the
/// 0x0020faa0 gather of the box `lo`..`hi` (REPLACED gather: collider vertex AABB vs the box grown by `radius`).
/// 0x00223a10: b = n.centre - plane.w (signed distance of the centre from the contact plane).
pub fn cam_pushout_223a70(c: &CamCollide, radius: f32, lo: [f32; 3], hi: [f32; 3], centre: [f32; 3]) -> [f32; 3] {
    use crate::engine::physics::sphere::FUN_0014eda0;
    let Some(cw) = &c.cw else { return centre };
    let near = CollisionWorld { colliders: cw.colliders.iter().filter(|col| aabb_hits(&col.geom.verts, lo, hi, radius)).cloned().collect() };
    if near.colliders.is_empty() {
        return centre;
    }
    FUN_0014eda0(&near, radius, radius, centre, centre, &mut |cc, _v, s| FUN_00223a10(cc, s))
}

/// 0x00223a10(cm, centre, vel, sbi): n.centre - plane.w.
pub fn FUN_00223a10(centre: [f32; 3], s: &crate::engine::physics::sphere::Sbi) -> f32 {
    s.plane[0] * centre[0] + s.plane[1] * centre[1] + s.plane[2] * centre[2] - s.plane[3]
}

fn aabb_hits(verts: &[[f32; 3]], lo: [f32; 3], hi: [f32; 3], pad: f32) -> bool {
    if verts.is_empty() {
        return false;
    }
    let (mut bl, mut bh) = (verts[0], verts[0]);
    for v in verts {
        for k in 0..3 {
            bl[k] = bl[k].min(v[k]);
            bh[k] = bh[k].max(v[k]);
        }
    }
    (0..3).all(|k| bh[k] >= lo[k] - pad && bl[k] <= hi[k] + pad)
}

/// WP-P15 owner chain: world frames of object `i` for every placed copy of its owner-chain root. Directory
/// entries such as row 14 "p_hook" (a = 0, n = 18) are streamed once (copy 0) and cloned n-1 times
/// (FUN_0013e510); every copy, copy 0 included, is then handed to a class-6 record by FUN_001ca370, which sets
/// the owner (vtable +0x54) and the record xform (vtable +0x12c, World::life.placed). The original clones the
/// whole subtree (vtable +0x144), so a point inside the prefab exists once per copy at
/// local(point -> root) x placed(copy) x owner frame. The port's clone copies the root object only: the copy root
/// index stands for the copy (target id; `i` itself for copy 0). A root that was never placed -> one frame,
/// gfx::attach::BoneWorlds::xfm_world (unchanged).
pub fn instance_frames(w: &World, bw: &mut BoneWorlds, i: usize) -> Vec<(usize, Affine)> {
    let mut root = i;
    while let Some(o) = w.objects[root].owner {
        if w.life.placed.contains_key(&root) {
            break;
        }
        root = o;
    }
    let copies: Vec<usize> = std::iter::once(root)
        .chain((0..w.objects.len()).filter(|&c| w.objects[c].clone_of == Some(root)))
        .filter(|c| w.life.placed.contains_key(c))
        .collect();
    let own = bw.xfm_world(w, i);
    if copies.is_empty() {
        return own.map(|m| vec![(i, m)]).unwrap_or_default();
    }
    // point frame relative to the root (the root's own stream frame, if any, is replaced by the placement)
    let rel = if root == i {
        IDENT
    } else {
        let Some(m) = own else { return vec![] };
        match bw.xfm_world(w, root) {
            Some(r) => mul(&m, &inverse(&r)),
            None => m,
        }
    };
    copies
        .into_iter()
        .map(|c| {
            let mut place = affine_from_bytes(&w.life.placed[&c]);
            if let Some(o) = w.objects[c].owner.and_then(|o| bw.xfm_world(w, o)) {
                place = mul(&place, &o);
            }
            (if c == root { i } else { c }, mul(&rel, &place))
        })
        .collect()
}

const IDENT: Affine = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [0.0, 0.0, 0.0]];

/// Affine inverse (row-vector convention of gfx::scene::mul).
fn inverse(a: &Affine) -> Affine {
    let m = [a[0], a[1], a[2]];
    let det = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1]) - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
    if det.abs() < 1e-12 {
        return IDENT;
    }
    let d = 1.0 / det;
    let r = [
        [(m[1][1] * m[2][2] - m[1][2] * m[2][1]) * d, (m[0][2] * m[2][1] - m[0][1] * m[2][2]) * d, (m[0][1] * m[1][2] - m[0][2] * m[1][1]) * d],
        [(m[1][2] * m[2][0] - m[1][0] * m[2][2]) * d, (m[0][0] * m[2][2] - m[0][2] * m[2][0]) * d, (m[0][2] * m[1][0] - m[0][0] * m[1][2]) * d],
        [(m[1][0] * m[2][1] - m[1][1] * m[2][0]) * d, (m[0][1] * m[2][0] - m[0][0] * m[2][1]) * d, (m[0][0] * m[1][1] - m[0][1] * m[1][0]) * d],
    ];
    let t = a[3];
    let rt = [
        -(t[0] * r[0][0] + t[1] * r[1][0] + t[2] * r[2][0]),
        -(t[0] * r[0][1] + t[1] * r[1][1] + t[2] * r[2][1]),
        -(t[0] * r[0][2] + t[1] * r[1][2] + t[2] * r[2][2]),
    ];
    [r[0], r[1], r[2], rt]
}
