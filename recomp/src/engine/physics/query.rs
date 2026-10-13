//! WP-E: level collision set + camera segment query (camera::collide::CmSegmentQuery).
//! The original camera query (0x0014ea08) walks a list of collision objects, each giving a BSPC
//! (+0x70 cbsp, +0x74 absp), clips with 0x00118668 (limit = absp + 0xc*cbsp, max 0x40 - n) and unions
//! the result into the running list with 0x0014c9f8(mode 0, max 0x40).
#![allow(non_snake_case)]

use super::bsp::{FUN_00118668, FUN_0014c9f8, Lsg};
use super::geom::{bounding_sphere_227bb8, build_bsp, build_geom, BspC, CGeom, V3};
use crate::engine::camera::collide::CmSegmentQuery;
use crate::engine::world::objects::{LoadData, Payload};
use crate::engine::world::World;

/// Where a collision geom came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    /// FUN_00227bb8 / FUN_00149358 / FUN_002282b0 (+0xf0 geom, +0xe0 BSP)
    Solid,
    /// FUN_00227380 (+0xf8 geom, +0xe8 BSP)
    L227380,
    /// FUN_001f31e0 collision part k (+0xe4 + 0xb0 k)
    AloPart(usize),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Collider {
    pub object: usize,
    pub source: Source,
    pub geom: CGeom,
    pub bsp: BspC,
    /// FUN_00227bb8 +0x110 / +0x120 (Solid only)
    pub sphere: Option<(V3, f32)>,
}

/// All collision geometry the loaded level carries (geometry in the space it was streamed in; ALO parts
/// with a non-zero ref are transformed by FUN_00166270 with the referenced object's matrix at run time,
/// which needs the (unported) ALO transform hierarchy, so they are kept untransformed).
#[derive(Clone, Debug, Default)]
pub struct CollisionWorld {
    pub colliders: Vec<Collider>,
}

impl CollisionWorld {
    pub fn build(w: &World) -> Self {
        let mut colliders = Vec::new();
        for (i, lo) in w.objects.iter().enumerate() {
            let Payload::Loaded(ld) = &lo.payload else { continue };
            let mut add = |source, g, b, solid: bool| {
                let geom = build_geom(g);
                let sphere = solid.then(|| bounding_sphere_227bb8(&geom));
                colliders.push(Collider { object: i, source, geom, bsp: build_bsp(b), sphere });
            };
            match ld {
                LoadData::Solid(s) | LoadData::L2282b0 { solid: s, .. } => add(Source::Solid, &s.geom, &s.bsp, true),
                LoadData::L227380 { geom, bsp, .. } => add(Source::L227380, geom, bsp, false),
                LoadData::Alo { alo, .. } => {
                    for (k, p) in alo.parts.iter().enumerate() {
                        add(Source::AloPart(k), &p.geom, &p.bsp, false);
                    }
                }
                _ => {}
            }
        }
        CollisionWorld { colliders }
    }

    /// WP-P9: the SO collision set the dynamics see (0x1f8dd8 broadphase over objects' +0xe4 parts): only
    /// FUN_001f31e0 collision parts, in world space. FUN_00227380 (class 5) and FUN_00227bb8 (classes 22-27)
    /// geoms are VOLUMES (point-in-volume tests 0x2273f0 / 0x227c90 on the point taken to object space by
    /// 0x1a6f00), never contact geometry; `build` keeps them for the camera-era tests. Part world copy (+0x90):
    /// ref == 0 -> +0 = DAT_002c44f0 and the copy aliases the streamed geom (already world space); else
    /// +0 = vtable +0x19c(ref) (bone) and FUN_00166270(local, bone world) (rest pose here; parts are not
    /// re-transformed when bones move). Water (class 89) parts are hazard volumes (player::p8_hazard).
    pub fn build_so(w: &World) -> Self {
        use crate::engine::gfx::attach::{ee_sub_22d6b0, BoneRef, BoneWorlds};
        use super::geom::FUN_00166270;
        let mut bw = BoneWorlds::default();
        let mut colliders = Vec::new();
        for (i, lo) in w.objects.iter().enumerate() {
            let Payload::Loaded(LoadData::Alo { alo, .. }) = &lo.payload else { continue };
            if lo.class == 89 {
                continue;
            }
            for (k, p) in alo.parts.iter().enumerate() {
                let local = build_geom(&p.geom);
                let geom = if p.r#ref == 0 {
                    local
                } else {
                    match ee_sub_22d6b0(w, p.r#ref, lo.owner, Some(i), BoneRef::None) {
                        BoneRef::Bone(ob, b) => FUN_00166270(&local, Some(&bw.get(w, ob, b))),
                        _ => local,
                    }
                };
                colliders.push(Collider { object: i, source: Source::AloPart(k), geom, bsp: build_bsp(&p.bsp), sphere: None });
            }
        }
        CollisionWorld { colliders }
    }

    /// 0x0014ea08 inner loop: union of all colliders' clip lists for p1->p2 (max 0x40 records).
    /// WP-P15: the original only clips the objects FUN_0020faa0 gathered for the query box; REPLACED by a
    /// collider vertex AABB vs segment AABB reject (without it a closed "solid outside" mesh such as row 14
    /// obj 3577, class 48 at z 25000, classifies every segment as inside).
    pub fn clip_segment(&self, p1: V3, p2: V3) -> Vec<Lsg> {
        let lo = [p1[0].min(p2[0]), p1[1].min(p2[1]), p1[2].min(p2[2])];
        let hi = [p1[0].max(p2[0]), p1[1].max(p2[1]), p1[2].max(p2[2])];
        let mut acc: Vec<Lsg> = Vec::new();
        for c in &self.colliders {
            if c.bsp.nodes.is_empty() || c.geom.verts.is_empty() {
                continue;
            }
            let (mut bl, mut bh) = (c.geom.verts[0], c.geom.verts[0]);
            for v in &c.geom.verts {
                for k in 0..3 {
                    bl[k] = bl[k].min(v[k]);
                    bh[k] = bh[k].max(v[k]);
                }
            }
            if (0..3).any(|k| bh[k] < lo[k] || bl[k] > hi[k]) {
                continue;
            }
            let mut tmp = Vec::new();
            let n = FUN_00118668(&c.geom, &c.bsp, 0, c.bsp.nodes.len(), p1, p2, 0x40usize.saturating_sub(acc.len()), &mut tmp);
            if n != 0 {
                acc = FUN_0014c9f8(&acc, &tmp[..n], 0x40, 0);
            }
        }
        acc
    }
}

impl CmSegmentQuery for CollisionWorld {
    fn solid_intervals(&mut self, start: V3, end: V3) -> Vec<(f32, f32)> {
        self.clip_segment(start, end).iter().map(|l| (l.t[0], l.t[1])).collect()
    }
}
