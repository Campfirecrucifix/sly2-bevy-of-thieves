//! WP-C: level scene assembly (Bevy-independent): ALO objects -> placed mesh primitives.
//! Transform data (WP-B3): FUN_0022d7a0 joints (+0x54, 0x100 each) hold a FUN_00117f18 4x3
//! matrix (rows = basis, PS2 row-vector convention v' = v * M) relative to the parent joint
//! (s16 ref resolved by FUN_0022d6b0, see gfx::attach; kinds 8/0xe = identity, attach ref resolved there).
//! A mesh record / instance s16 ref is the joint it hangs off; an instance adds its own matrix.
//! A child ALO (FUN_0013fb40 nested in the owner payload) hangs off an owner bone through its
//! root bone's 0x1000 ref (WP-C3; previously approximated as the owner's root joint). Joint animation (FUN_0022faa0 VU0 matrices, ASEG) is WP-D.

use super::attach::BoneWorlds;
use super::mesh::{convert, MeshPrim};
use super::texture::EntryTextures;
use crate::engine::disc::LoadedLevel;
use crate::engine::world::alo::Alo;
use crate::engine::world::objects::{LoadData, Payload};
use crate::engine::world::World;

/// Affine 4x3 (rows 0..2 = basis vectors, row 3 = translation).
pub type Affine = [[f32; 3]; 4];
pub const IDENTITY: Affine = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [0.0, 0.0, 0.0]];

/// a then b (row vectors: v * a * b)
pub fn mul(a: &Affine, b: &Affine) -> Affine {
    let rot = |v: [f32; 3]| [0, 1, 2].map(|j| v[0] * b[0][j] + v[1] * b[1][j] + v[2] * b[2][j]);
    let t = rot(a[3]);
    [rot(a[0]), rot(a[1]), rot(a[2]), [t[0] + b[3][0], t[1] + b[3][1], t[2] + b[3][2]]]
}

/// Rest-pose joint matrices in object space (parent chain, cycles / bad refs -> root).
pub fn joint_matrices(alo: &Alo) -> Vec<Affine> {
    let n = alo.base.bones.len();
    let mut out: Vec<Option<Affine>> = vec![None; n];
    fn solve(alo: &Alo, i: usize, out: &mut Vec<Option<Affine>>, depth: usize) -> Affine {
        if let Some(m) = out[i] {
            return m;
        }
        let b = &alo.base.bones[i];
        let local = b.xform.unwrap_or(IDENTITY);
        let p = b.r#ref;
        let m = if p >= 0 && (p as usize) < alo.base.bones.len() && p as usize != i && depth < 256 {
            mul(&local, &solve(alo, p as usize, out, depth + 1))
        } else {
            local
        };
        out[i] = Some(m);
        m
    }
    (0..n).map(|i| solve(alo, i, &mut out, 0)).collect()
}

/// One placed primitive.
#[derive(Clone, Debug)]
pub struct ScenePrim {
    /// world object index (`World::objects`)
    pub object: usize,
    /// mesh record index in the object's MeshSet, instance (None = the record itself)
    pub record: usize,
    pub instance: Option<usize>,
    /// object-space transform (joint chain [+ instance])
    pub local: Affine,
    pub prim: MeshPrim,
}

#[derive(Clone, Debug)]
pub struct SceneObject {
    pub object: usize,
    pub oid: i32,
    pub entry: usize,
    /// index into `LevelScene::objects` of the owning ALO
    pub owner: Option<usize>,
    /// relative to the owner (or world): the object's root joint
    pub local: Affine,
    pub prims: Vec<ScenePrim>,
}

/// WP-C6: one FUN_001ca370 template copy (`Payload::Clone`, no ALO data of its own): drawn with the
/// prims of its template's scene object at `world`.
#[derive(Clone, Debug)]
pub struct SceneCopy {
    /// world object index of the copy
    pub object: usize,
    /// index into `LevelScene::objects` of the template object whose prims it draws
    pub source: usize,
    /// world matrix of the copy's root (replaces the template root's `world_of`)
    pub world: Affine,
}

#[derive(Clone, Debug, Default)]
pub struct LevelScene {
    pub lsn: u32,
    /// per directory entry
    pub textures: Vec<EntryTextures>,
    pub objects: Vec<SceneObject>,
    /// WP-C6: placed template copies (see `template_copies`)
    pub copies: Vec<SceneCopy>,
}

/// WP-C6: classes whose copies other draw paths place themselves (npcdraw: K6 guards, K8 pickups;
/// npcdraw::objects: 81 projectiles, 89 water clones, 76 swarm riders).
pub fn copy_drawn_elsewhere(class: u16) -> bool {
    use crate::engine::classes::{guard::PORTED_K6, k8::PORTED_K8};
    matches!(class, 76 | 81 | 89) || PORTED_K6.iter().chain(PORTED_K8).any(|x| x.0 == class)
}

/// WP-C6: the copies FUN_001c9b08's template branch instantiates (FUN_001ca370: entry +4 copy objects,
/// vtable +0x12c xform) are full objects in the original world and draw like the template (the copy
/// shares the template's MeshSet, FUN_0013e510). The world model keeps them as `Payload::Clone`, so the
/// level scene only had the template objects (at their load place, usually the origin): on t20 the
/// floor / wall pieces (classes 48 / 98 / 99) never showed. Copy world = template object world (rest) *
/// placement, placement = owner socket (+0x12c relative to the owner bone, gfx::attach::template_socket)
/// or the bare +0x12c matrix; copies owned by another copy keep their template-relative place.
fn template_copies(world: &World, objects: &[SceneObject], map: &std::collections::HashMap<usize, usize>, worlds_of: &dyn Fn(usize) -> Affine, bw: &mut BoneWorlds) -> Vec<SceneCopy> {
    let mut copies: Vec<SceneCopy> = Vec::new();
    let mut at: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    for t in &world.template_instances {
        for &c in &t.objects {
            let o = &world.objects[c];
            let (Payload::Clone, Some(src)) = (&o.payload, o.clone_of) else { continue };
            let Some(&si) = map.get(&src) else { continue };
            if copy_drawn_elsewhere(o.class) || objects[si].prims.is_empty() || at.contains_key(&c) {
                continue;
            }
            let ws = worlds_of(si);
            let owner_copy = o.owner.and_then(|p| at.get(&p).copied());
            let world_m = if let Some(pc) = owner_copy {
                let ps = copies[pc].source;
                mul(&mul(&ws, &invert(&worlds_of(ps))), &copies[pc].world)
            } else if let Some((super::attach::BoneRef::Bone(ob, k), pm)) = super::attach::template_socket(world, c) {
                mul(&mul(&ws, &pm), &bw.get(world, ob, k))
            } else if let Some(p) = world.life.placed.get(&c) {
                mul(&ws, &super::attach::affine_from_bytes(p))
            } else {
                continue;
            };
            at.insert(c, copies.len());
            copies.push(SceneCopy { object: c, source: si, world: world_m });
        }
    }
    copies
}

impl LevelScene {
    pub fn build(level: &LoadedLevel, world: &World) -> LevelScene {
        Self::build_from(level.desc.lsn(), &level.data, world)
    }

    /// `data` = the decompressed level bytes the world was loaded from (blob textures).
    pub fn build_from(lsn: u32, data: &[u8], world: &World) -> LevelScene {
        let textures = world.entries.iter().map(|e| EntryTextures::from_sections(&e.sections, data)).collect();
        let mut objects = Vec::new();
        let mut worlds = Vec::new();
        let mut map = std::collections::HashMap::new();
        // WP-C3: bone world matrices with FUN_0022d6b0 refs (child-ALO roots hang off an owner
        // bone via 0x1000 refs; null parent = world space), see gfx::attach.
        let mut bw = BoneWorlds::default();
        for (oi, o) in world.objects.iter().enumerate() {
            let Payload::Loaded(LoadData::Alo { alo, .. }) = &o.payload else { continue };
            let nb = alo.base.bones.len();
            let bone = |bw: &mut BoneWorlds, r: i16| if r >= 0 && (r as usize) < nb { bw.get(world, oi, r as usize) } else { IDENTITY };
            // root joint = object placement
            let root = if nb > 0 { bw.get(world, oi, 0) } else { IDENTITY };
            let mut prims = Vec::new();
            for (ri, r) in alo.mesh.records.iter().enumerate() {
                let p = convert(&r.data);
                if p.is_empty() {
                    continue;
                }
                let base = bone(&mut bw, r.r#ref);
                for prim in &p {
                    prims.push(ScenePrim { object: oi, record: ri, instance: None, local: base, prim: prim.clone() });
                }
                for (ii, inst) in r.instances.iter().enumerate() {
                    let m = mul(&inst.xform, &bone(&mut bw, inst.r#ref));
                    for prim in &p {
                        prims.push(ScenePrim { object: oi, record: ri, instance: Some(ii), local: m, prim: prim.clone() });
                    }
                }
            }
            map.insert(oi, objects.len());
            worlds.push(root);
            objects.push(SceneObject { object: oi, oid: o.oid, entry: o.entry, owner: None, local: root, prims });
        }
        // owners: walk up Lo::owner to the nearest ALO
        for so in objects.iter_mut() {
            let mut cur = world.objects[so.object].owner;
            while let Some(c) = cur {
                if let Some(&i) = map.get(&c) {
                    so.owner = Some(i);
                    break;
                }
                cur = world.objects[c].owner;
            }
        }
        // prims relative to the object root; object root relative to its owner's root
        for (i, so) in objects.iter_mut().enumerate() {
            let inv = invert(&worlds[i]);
            for p in so.prims.iter_mut() {
                p.local = mul(&p.local, &inv);
            }
            so.local = match so.owner {
                Some(o) => mul(&worlds[i], &invert(&worlds[o])),
                None => worlds[i],
            };
        }
        let mut scene = LevelScene { lsn, textures, objects, copies: Vec::new() };
        let s = &scene;
        let copies = template_copies(world, &s.objects, &map, &|i| s.world_of(i), &mut bw);
        scene.copies = copies;
        scene
    }

    /// World matrix of a scene object (owner chain).
    pub fn world_of(&self, i: usize) -> Affine {
        let mut m = self.objects[i].local;
        let mut cur = self.objects[i].owner;
        let mut guard = 0;
        while let Some(c) = cur {
            if guard > 64 {
                break;
            }
            guard += 1;
            m = mul(&m, &self.objects[c].local);
            cur = self.objects[c].owner;
        }
        m
    }
}

/// General affine inverse (singular -> identity).
pub fn invert(m: &Affine) -> Affine {
    let [a, b, c] = [m[0], m[1], m[2]];
    let det = a[0] * (b[1] * c[2] - b[2] * c[1]) - a[1] * (b[0] * c[2] - b[2] * c[0]) + a[2] * (b[0] * c[1] - b[1] * c[0]);
    if det.abs() < 1e-20 || !det.is_finite() {
        return IDENTITY;
    }
    let d = 1.0 / det;
    let r = [
        [(b[1] * c[2] - b[2] * c[1]) * d, (a[2] * c[1] - a[1] * c[2]) * d, (a[1] * b[2] - a[2] * b[1]) * d],
        [(b[2] * c[0] - b[0] * c[2]) * d, (a[0] * c[2] - a[2] * c[0]) * d, (a[2] * b[0] - a[0] * b[2]) * d],
        [(b[0] * c[1] - b[1] * c[0]) * d, (a[1] * c[0] - a[0] * c[1]) * d, (a[0] * b[1] - a[1] * b[0]) * d],
    ];
    let t = m[3];
    let nt = [0, 1, 2].map(|j| -(t[0] * r[0][j] + t[1] * r[1][j] + t[2] * r[2][j]));
    [r[0], r[1], r[2], nt]
}
