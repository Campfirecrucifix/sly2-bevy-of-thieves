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
    /// FUN_00227bb8 +0x110 / +0x120 (Solid); SO parts (`build_so_masked`): the part's bounding sphere
    pub sphere: Option<(V3, f32)>,
    /// SO parts (`build_so_masked`): the owning SO's world bounds node (min, max), FUN_001f50e0 (SO vtable +0x18c):
    /// union of all its parts' world vertices. The broadphase gather FUN_0020faa0 tests this box.
    pub bbox: Option<(V3, V3)>,
}

impl Collider {
    /// Broadphase: false when the query box c +- r misses this collider's SO bounds node (FUN_0020faa0 node test:
    /// node.min <= q.max && q.min <= node.max per axis; CRAWL R61, 1:1 for SO parts). Colliders without a node
    /// (Solid) keep the R59 sphere reject (REPLACED); without either they always pass.
    pub fn near(&self, c: V3, r: f32) -> bool {
        if let Some((lo, hi)) = self.bbox {
            return (0..3).all(|k| lo[k] <= c[k] + r && c[k] - r <= hi[k]);
        }
        match self.sphere {
            Some((sc, sr)) if sr >= 0.0 => {
                let d = [sc[0] - c[0], sc[1] - c[1], sc[2] - c[2]];
                d[0] * d[0] + d[1] * d[1] + d[2] * d[2] <= (sr + r) * (sr + r)
            }
            _ => true,
        }
    }
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
                colliders.push(Collider { object: i, source, geom, bsp: build_bsp(b), sphere, bbox: None });
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
    /// WP-EP1d: FUN_001f2b20 links only the parts without +0x10 bit 50 (stream flag byte b2) into the SO
    /// collision set; FUN_001f6ac8 / 001f6b48 clear / set the bit at run time (`build_so_masked`).
    pub fn build_so(w: &World) -> Self {
        Self::build_so_masked(w, &std::collections::HashMap::new())
    }

    /// WP-EP1d: `build_so` with the run-time part +0x10 bit 50 state `on[(object, part)]` (true = linked,
    /// FUN_001f2a60; false = unlinked, FUN_001f2ad0); parts not in `on` keep the streamed bit.
    pub fn build_so_masked(w: &World, on: &std::collections::HashMap<(usize, usize), bool>) -> Self {
        use crate::engine::gfx::attach::{ee_sub_22d6b0, BoneRef, BoneWorlds};
        use super::geom::FUN_00166270;
        let mut bw = BoneWorlds::default();
        let mut colliders = Vec::new();
        for (i, lo) in w.objects.iter().enumerate() {
            // CRAWL R62: a clone (CloneLo: payload Clone, clone_of = the source) carries the source's SO parts at its
            // own placement (Sly 1 CloneSo -> CloneGeom / CloneBspc; row 6 had 174 cloned SOs without collision, e.g.
            // class 98 oid 126405 x13). Clones without a placement or with world-space (ref 0) parts are skipped.
            let src = if matches!(lo.payload, Payload::Clone) { lo.clone_of.filter(|&c| c != i) } else { None };
            let Payload::Loaded(LoadData::Alo { alo, .. }) = &w.objects[src.unwrap_or(i)].payload else { continue };
            if src.is_some() && (w.life.placed.get(&i).is_none() || std::env::var("SLY2_NO_CLONE_SO").is_ok()) {
                continue;
            }
            if lo.class == 89 {
                continue;
            }
            for (k, p) in alo.parts.iter().enumerate() {
                if !on.get(&(i, k)).copied().unwrap_or(p.b2 & 1 == 0) {
                    continue;
                }
                let local = build_geom(&p.geom);
                let bone = if p.r#ref == 0 {
                    None
                } else {
                    match ee_sub_22d6b0(w, p.r#ref, lo.owner, Some(src.unwrap_or(i)), BoneRef::None) {
                        BoneRef::Bone(ob, b) => Some(bw.get(w, ob, b)),
                        _ => None,
                    }
                };
                // WIRE-EP8: placed (P15 life) objects: the bone chain is object-local; the original's bone 0
                // world includes the placement (as `k24_world_xf`: bone x placement), e.g. hub switch 1471
                // (not the 0x34 characters: Sly 53 / Bentley 55 / Murray collide as moving JTs, not as SO parts)
                // EMU5 (PCSX2 emu_rec5 hub, class-48 ALO world pos +0x180): top-level placed objects sit at their placement
                // (oid 141875 z +20000, 139345 (3806,-1277,308)); objects with an owner (children of 141875, e.g.
                // 142138 at (689,777,8)) do NOT get it on top of the owner's bone chain.
                let placed = w.life.placed.get(&i).filter(|_| lo.owner.is_none() && std::env::var("EP8NP").is_err()).filter(|_| !crate::engine::player::p4::FUN_00132b10(lo.class as i32, 0x34)).map(crate::engine::gfx::attach::affine_from_bytes);
                let geom = match (bone, placed) {
                    (Some(m), Some(pm)) => FUN_00166270(&local, Some(&crate::engine::gfx::scene::mul(&m, &pm))),
                    (Some(m), _) => FUN_00166270(&local, Some(&m)),
                    // ref 0: the streamed geom is already world space (DAT_002c44f0 alias)
                    (None, _) if src.is_some() => continue,
                    (None, _) => local,
                };
                // CRAWL R59/R61: the broadphase (FUN_0020faa0) only gathers SOs whose world bounds node overlaps the
                // query box; the node is set below (FUN_001f50e0). Without it a far part counts too: a JT under a
                // parked class-48 ALO (EP4 hub oid 17568 at z 24500) is "inside" its open shell and 0x1a9108 pushes
                // it ~24000 up (swing hook 1838).
                colliders.push(Collider { object: i, source: Source::AloPart(k), geom, bsp: build_bsp(&p.bsp), sphere: None, bbox: None });
            }
        }
        // FUN_001f50e0 (SO vtable +0x18c, via FUN_00133e30): the SO's bounds node = vmini / vmax over the world
        // vertices of all its parts (+0x94 count, +0x94 list; sphere parts centre +- radius: none in the port).
        let mut nodes: std::collections::HashMap<usize, (V3, V3)> = Default::default();
        for c in &colliders {
            for v in &c.geom.verts {
                let e = nodes.entry(c.object).or_insert((*v, *v));
                for k in 0..3 {
                    e.0[k] = e.0[k].min(v[k]);
                    e.1[k] = e.1[k].max(v[k]);
                }
            }
        }
        for c in &mut colliders {
            c.bbox = nodes.get(&c.object).copied();
        }
        CollisionWorld { colliders }
    }

    /// FEEL R87: the SO part world copy (+0x90) is re-transformed when its bone moves: every AloPart collider whose
    /// part bone (FUN_0022d6b0 of part +0x?? ref) has a posed world in `bones` (classes host k29_bones, the live
    /// ASEGA poses, world space incl. placement) gets local geom x bone world. EP1 hub safehouse exit: the door slab
    /// oid 141705 (bone 3) swings open on SMA 141703 -> 17435 and Sly walks 27250 -> 27248 through it.
    /// Returns whether a collider changed. Opt-out SLY2_FEEL_R87POSE=0.
    pub fn repose(&mut self, w: &World, bones: &std::collections::HashMap<(usize, usize), crate::engine::gfx::scene::Affine>) -> bool {
        let ch = self.repose_calc(w, bones);
        self.repose_apply(ch)
    }

    /// FEEL R87: the moved part geoms of [`CollisionWorld::repose`] (collider index, world geom), without mutating.
    pub fn repose_calc(&self, w: &World, bones: &std::collections::HashMap<(usize, usize), crate::engine::gfx::scene::Affine>) -> Vec<(usize, CGeom)> {
        use crate::engine::gfx::attach::{ee_sub_22d6b0, BoneRef};
        let mut out = Vec::new();
        if bones.is_empty() || std::env::var("SLY2_FEEL_R87POSE").is_ok_and(|v| v == "0") {
            return out;
        }
        let objs: std::collections::HashSet<usize> = bones.keys().map(|k| k.0).collect();
        for (ci, c) in self.colliders.iter().enumerate() {
            let Source::AloPart(k) = c.source else { continue };
            let Some(lo) = w.objects.get(c.object) else { continue };
            if !objs.contains(&c.object) && !lo.owner.is_some_and(|o| objs.contains(&o)) {
                continue;
            }
            let src = if matches!(lo.payload, Payload::Clone) { lo.clone_of.filter(|&s| s != c.object) } else { None };
            let Payload::Loaded(LoadData::Alo { alo, .. }) = &w.objects[src.unwrap_or(c.object)].payload else { continue };
            let Some(p) = alo.parts.get(k) else { continue };
            if p.r#ref == 0 {
                continue;
            }
            let BoneRef::Bone(ob, b) = ee_sub_22d6b0(w, p.r#ref, lo.owner, Some(src.unwrap_or(c.object)), BoneRef::None) else { continue };
            let Some(m) = bones.get(&(ob, b)) else { continue };
            let g = super::geom::FUN_00166270(&build_geom(&p.geom), Some(m));
            if g.verts != c.geom.verts {
                out.push((ci, g));
            }
        }
        out
    }

    /// FEEL R87: apply [`CollisionWorld::repose_calc`] (geoms + the FUN_001f50e0 bounds node of the moved SOs).
    pub fn repose_apply(&mut self, ch: Vec<(usize, CGeom)>) -> bool {
        if ch.is_empty() {
            return false;
        }
        let mut changed: std::collections::HashSet<usize> = Default::default();
        for (ci, g) in ch {
            changed.insert(self.colliders[ci].object);
            self.colliders[ci].geom = g;
        }
        let mut nodes: std::collections::HashMap<usize, (V3, V3)> = Default::default();
        for c in self.colliders.iter().filter(|c| changed.contains(&c.object)) {
            for v in &c.geom.verts {
                let e = nodes.entry(c.object).or_insert((*v, *v));
                for k in 0..3 {
                    e.0[k] = e.0[k].min(v[k]);
                    e.1[k] = e.1[k].max(v[k]);
                }
            }
        }
        for c in &mut self.colliders {
            if let Some(n) = nodes.get(&c.object) {
                c.bbox = Some(*n);
            }
        }
        true
    }

    /// 0x0014ea08 inner loop: union of all colliders' clip lists for p1->p2 (max 0x40 records).
    /// WP-P15: the original only clips the objects FUN_0020faa0 gathered for the query box; REPLACED by a
    /// collider vertex AABB vs segment AABB reject (without it a closed "solid outside" mesh such as row 14
    /// obj 3577, class 48 at z 25000, classifies every segment as inside).
    pub fn clip_segment(&self, p1: V3, p2: V3) -> Vec<Lsg> {
        self.clip_segment_lim(p1, p2, false)
    }

    /// R29: the camera's 0x0014ea08 form: parts gathered with filter 0x0014f030 (BSPC third word `b` > 0) and
    /// each part's BSP clipped with node limit `b` (0x14ea78: a1 = part +0x74 + part +0x70 * 0xc).
    pub fn clip_segment_cam(&self, p1: V3, p2: V3) -> Vec<Lsg> {
        self.clip_segment_lim(p1, p2, true)
    }

    fn clip_segment_lim(&self, p1: V3, p2: V3, cam: bool) -> Vec<Lsg> {
        let lo = [p1[0].min(p2[0]), p1[1].min(p2[1]), p1[2].min(p2[2])];
        let hi = [p1[0].max(p2[0]), p1[1].max(p2[1]), p1[2].max(p2[2])];
        let mut acc: Vec<Lsg> = Vec::new();
        for c in &self.colliders {
            if c.bsp.nodes.is_empty() || c.geom.verts.is_empty() || (cam && c.bsp.b as i32 <= 0) {
                continue;
            }
            let limit = if cam { c.bsp.b as usize } else { c.bsp.nodes.len() };
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
            let n = FUN_00118668(&c.geom, &c.bsp, 0, limit, p1, p2, 0x40usize.saturating_sub(acc.len()), &mut tmp);
            if n != 0 {
                acc = FUN_0014c9f8(&acc, &tmp[..n], 0x40, 0);
            }
        }
        acc
    }
}

impl CmSegmentQuery for CollisionWorld {
    fn solid_intervals(&mut self, start: V3, end: V3) -> Vec<(f32, f32)> {
        self.clip_segment_cam(start, end).iter().map(|l| (l.t[0], l.t[1])).collect()
    }
}
