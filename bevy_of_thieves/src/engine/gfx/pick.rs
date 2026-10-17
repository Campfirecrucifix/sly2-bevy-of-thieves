//! WP-A10 debug aid (port addition, no PS2 counterpart): identify the level prim under a screen
//! pixel. `SLY_A10_PICK="x,y;x,y..."` (window pixels) casts a CPU ray from the active 3D camera at
//! frame `SLY_A10_PICK_FRAME` (default 50) against every tagged level mesh and logs the nearest
//! hit: world object / oid / class, directory entry, shader (kind, flags) and the material state.

use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;

/// Which level prim a spawned mesh entity draws (gfx::game::spawn_level_geometry).
#[derive(Component, Clone, Copy, Debug)]
pub struct PrimTag {
    pub object: usize,
    pub oid: i32,
    pub entry: usize,
    pub shader: Option<u32>,
    pub record: usize,
}

/// Ray / triangle (Moeller-Trumbore), distance along the ray.
pub fn ray_tri(o: Vec3, d: Vec3, a: Vec3, b: Vec3, c: Vec3) -> Option<f32> {
    let (e1, e2) = (b - a, c - a);
    let p = d.cross(e2);
    let det = e1.dot(p);
    if det.abs() < 1e-9 {
        return None;
    }
    let inv = 1.0 / det;
    let t0 = o - a;
    let u = t0.dot(p) * inv;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = t0.cross(e1);
    let v = d.dot(q) * inv;
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = e2.dot(q) * inv;
    (t > 0.0).then_some(t)
}

pub(crate) fn add_pick_debug(app: &mut App) {
    if std::env::var("SLY_A10_PICK").is_ok() {
        app.add_systems(PostUpdate, pick_debug.after(bevy::transform::TransformSystems::Propagate));
    }
}

#[allow(clippy::too_many_arguments)]
fn pick_debug(
    mut frame: Local<u32>,
    cams: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    prims: Query<(&PrimTag, &Mesh3d, &GlobalTransform, &MeshMaterial3d<StandardMaterial>, &InheritedVisibility)>,
    meshes: Res<Assets<Mesh>>,
    mats: Res<Assets<StandardMaterial>>,
    images: Res<Assets<Image>>,
    world: Option<Res<crate::engine::world::World>>,
    data: Option<Res<super::game::GfxLevelData>>,
) {
    *frame += 1;
    let at: u32 = std::env::var("SLY_A10_PICK_FRAME").ok().and_then(|v| v.parse().ok()).unwrap_or(50);
    if *frame != at {
        return;
    }
    let Some((cam, cgt)) = cams.iter().find(|(c, _)| c.is_active) else { return };
    let spec = std::env::var("SLY_A10_PICK").unwrap_or_default();
    let scene = match (&world, &data) {
        (Some(w), Some(d)) => Some(super::scene::LevelScene::build_from(d.lsn, &d.data, w)),
        _ => None,
    };
    for pt in spec.split(';') {
        let Some((x, y)) = pt.split_once(',') else { continue };
        let (Ok(x), Ok(y)) = (x.trim().parse::<f32>(), y.trim().parse::<f32>()) else { continue };
        let Ok(ray) = cam.viewport_to_world(cgt, Vec2::new(x, y)) else { continue };
        let (o, d) = (ray.origin, *ray.direction);
        let mut best: Option<(f32, PrimTag, AssetId<StandardMaterial>)> = None;
        for (tag, m, gt, mat, vis) in prims.iter() {
            if !vis.get() {
                continue;
            }
            let Some(mesh) = meshes.get(&m.0) else { continue };
            let Some(VertexAttributeValues::Float32x3(pos)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) else { continue };
            let aff = gt.affine();
            let wp: Vec<Vec3> = pos.iter().map(|p| aff.transform_point3(Vec3::from(*p))).collect();
            let Some(ix) = mesh.indices() else { continue };
            let ix: Vec<usize> = ix.iter().collect();
            for t in ix.chunks_exact(3) {
                if let Some(dist) = ray_tri(o, d, wp[t[0]], wp[t[1]], wp[t[2]]) {
                    if best.as_ref().is_none_or(|b| dist < b.0) {
                        best = Some((dist, *tag, mat.0.id()));
                    }
                }
            }
        }
        let Some((dist, tag, mid)) = best else {
            info!("A10 pick ({x},{y}): no level prim");
            continue;
        };
        let class = world.as_ref().map(|w| w.objects[tag.object].class);
        let sh = scene.as_ref().and_then(|s| s.textures.get(tag.entry)?.shaders.get(tag.shader? as usize).cloned());
        let shs = sh.as_ref().map(|s| format!("kind {} flags {:#x} colors {:?} frames {}", s.kind, s.flags, s.colors, s.frames.len()));
        let m = mats.get(mid);
        let avg = m.and_then(|m| m.base_color_texture.as_ref()).and_then(|h| images.get(h)).and_then(|i| i.data.as_ref()).map(|px| {
            let n = (px.len() / 4).max(1) as u64;
            let mut s = [0u64; 4];
            for c in px.chunks_exact(4) {
                for k in 0..4 {
                    s[k] += c[k] as u64;
                }
            }
            s.map(|v| v / n)
        });
        info!(
            "A10 pick ({x},{y}): dist {dist:.0} object {} oid {} class {class:?} entry {} record {} shader {:?} [{}] alpha {:?} base {:?} emissive_tex {} unlit {} tex avg rgba {avg:?}",
            tag.object,
            tag.oid,
            tag.entry,
            tag.record,
            tag.shader,
            shs.unwrap_or_default(),
            m.map(|m| m.alpha_mode),
            m.map(|m| m.base_color),
            m.is_some_and(|m| m.emissive_texture.is_some()),
            m.is_some_and(|m| m.unlit),
        );
    }
}

/// WP-A10 debug: `SLY_A10_SKINCHECK=N` logs, at frame N and every 30 frames after, how many skinned
/// meshes have joints outside their culling box (Bevy frustum-culls skinned meshes by the `Aabb` of
/// the unposed mesh unless `DynamicSkinnedMeshBounds` refreshes it from the joints).
pub(crate) fn add_skin_check(app: &mut App) {
    if std::env::var("SLY_A10_SKINCHECK").is_ok() {
        app.add_systems(PostUpdate, skin_check.after(bevy::camera::visibility::VisibilitySystems::CheckVisibility));
    }
}

fn skin_check(
    mut frame: Local<u32>,
    skins: Query<(&bevy::mesh::skinning::SkinnedMesh, &Mesh3d, &bevy::camera::primitives::Aabb, &GlobalTransform, &ViewVisibility, Has<bevy::camera::visibility::DynamicSkinnedMeshBounds>)>,
    joints: Query<&GlobalTransform>,
    meshes: Res<Assets<Mesh>>,
    ibps: Res<Assets<bevy::mesh::skinning::SkinnedMeshInverseBindposes>>,
) {
    *frame += 1;
    let at: u32 = std::env::var("SLY_A10_SKINCHECK").ok().and_then(|v| v.parse().ok()).unwrap_or(60);
    if *frame < at || (*frame - at) % 30 != 0 {
        return;
    }
    let (mut n, mut bad, mut dynamic, mut culled_bad, mut worst) = (0, 0, 0, 0, 0f32);
    for (sm, m3, aabb, gt, vis, dyn_b) in &skins {
        n += 1;
        dynamic += dyn_b as u32;
        let (Some(mesh), Some(ibp)) = (meshes.get(&m3.0), ibps.get(&sm.inverse_bindposes)) else { continue };
        let (Some(VertexAttributeValues::Float32x3(pos)), Some(VertexAttributeValues::Uint16x4(ji)), Some(VertexAttributeValues::Float32x4(jw))) =
            (mesh.attribute(Mesh::ATTRIBUTE_POSITION), mesh.attribute(Mesh::ATTRIBUTE_JOINT_INDEX), mesh.attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT))
        else {
            continue;
        };
        let mats: Vec<Mat4> = sm.joints.iter().zip(ibp.iter()).map(|(j, b)| joints.get(*j).map(|g| g.to_matrix() * *b).unwrap_or(Mat4::IDENTITY)).collect();
        let inv = gt.affine().inverse();
        let (c, h) = (Vec3::from(aabb.center), Vec3::from(aabb.half_extents));
        let mut out = 0f32;
        for ((p, i), w) in pos.iter().zip(ji).zip(jw) {
            let mut q = Vec3::ZERO;
            for k in 0..4 {
                if w[k] > 0.0 {
                    q += w[k] * mats.get(i[k] as usize).copied().unwrap_or(Mat4::IDENTITY).transform_point3(Vec3::from(*p));
                }
            }
            let l = inv.transform_point3(q);
            out = out.max(((l - c).abs() - h).max_element());
        }
        if out > 2.0 {
            bad += 1;
            culled_bad += !vis.get() as u32;
            worst = worst.max(out);
        }
    }
    info!("A10 skincheck frame {}: {n} skinned meshes ({dynamic} dynamic bounds), {bad} posed outside their cull box (worst {worst:.0} units, {culled_bad} of them culled)", *frame);
}
