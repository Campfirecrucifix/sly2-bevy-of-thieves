//! WP-C Bevy side (feature `game`): LevelScene -> Mesh / Image / StandardMaterial entities.
//! Everything stays Solari-compatible (feature `rt`): POSITION, NORMAL, UV_0, TANGENT, COLOR,
//! U32 indices, lit StandardMaterial, no custom shaders. PS2 vertex colours go in as the COLOR
//! attribute (multiplied into base colour by the PBR shader), not as baked-only lighting.
//! PS2 is Z-up; the level root entity rotates it into Bevy's Y-up.

use super::material::{material_from, Alpha};
use super::mesh::MeshPrim;
use super::scene::{Affine, LevelScene};
use super::texture::{texture_override_path, TexImage};
use crate::engine::disc::{LevelLoadResult, LevelLoader};
use crate::engine::world::World;
use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor, ImageType, CompressedImageFormats};
use bevy::mesh::skinning::{SkinnedMesh, SkinnedMeshInverseBindposes};
use bevy::mesh::{Indices, PrimitiveTopology};
use crate::engine::world::objects::{LoadData, Payload};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use std::collections::HashMap;
use std::sync::Arc;

/// The decompressed level bytes, captured before `world::build_world` drops the load result
/// (blob-resident textures live in them).
#[derive(Resource, Clone)]
pub struct GfxLevelData {
    pub lsn: u32,
    pub data: Arc<Vec<u8>>,
}

/// Root entity of the spawned level geometry.
#[derive(Component)]
pub struct LevelGeometryRoot(pub u32);

/// WIRE-C3: the unlit additive colour pass (kinds 1/7 CLUT set 0) of a level prim; child of the lit prim.
#[derive(Component)]
pub struct ColourPass;

/// One spawned ALO object (world object index).
#[derive(Component)]
pub struct AloObject(pub usize);

/// Schedule `.before(world::build_world)`.
pub fn capture_level_data(mut commands: Commands, res: Option<Res<LevelLoadResult>>) {
    if let Some(Ok(l)) = res.as_deref().map(|r| &r.0) {
        commands.insert_resource(GfxLevelData { lsn: l.desc.lsn(), data: l.data.clone() });
    }
}

pub fn affine_to_transform(m: &Affine) -> Transform {
    // row-vector rows = images of the basis vectors = Bevy columns
    let c = |r: [f32; 3], w: f32| Vec4::new(r[0], r[1], r[2], w);
    Transform::from_matrix(Mat4::from_cols(c(m[0], 0.0), c(m[1], 0.0), c(m[2], 0.0), c(m[3], 1.0)))
}

pub fn to_bevy_mesh(p: &MeshPrim) -> Mesh {
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, p.positions.clone())
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, p.normals.clone())
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, p.uvs.clone())
        .with_inserted_attribute(Mesh::ATTRIBUTE_TANGENT, p.tangents.clone())
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, p.colors.clone())
        .with_inserted_indices(Indices::U32(p.indices.clone()))
}

pub fn to_bevy_image(t: &TexImage, clamp_u: bool, clamp_v: bool) -> Image {
    let mut img = Image::new(
        Extent3d { width: t.width.max(1), height: t.height.max(1), depth_or_array_layers: 1 },
        TextureDimension::D2,
        t.rgba.clone(),
        super::light::ps2_texture_format(),
        RenderAssetUsages::default(),
    );
    img.sampler = sampler(clamp_u, clamp_v);
    img
}

fn sampler(cu: bool, cv: bool) -> ImageSampler {
    let m = |c: bool| if c { ImageAddressMode::ClampToEdge } else { ImageAddressMode::Repeat };
    ImageSampler::Descriptor(ImageSamplerDescriptor { address_mode_u: m(cu), address_mode_v: m(cv), ..ImageSamplerDescriptor::linear() })
}

fn alpha_mode(a: Alpha) -> AlphaMode {
    match a {
        Alpha::Opaque => AlphaMode::Opaque,
        Alpha::Mask(c) => AlphaMode::Mask(c),
        Alpha::Blend => AlphaMode::Blend,
        Alpha::Add => AlphaMode::Add,
    }
}

/// Spawns the level's ALO hierarchy once `World` and `GfxLevelData` exist.
#[allow(clippy::too_many_arguments)]
pub fn spawn_level_geometry(
    mut commands: Commands,
    world: Option<Res<World>>,
    data: Option<Res<GfxLevelData>>,
    loader: Option<Res<LevelLoader>>,
    existing: Query<(Entity, &LevelGeometryRoot)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut inverse_bindposes: ResMut<Assets<SkinnedMeshInverseBindposes>>,
    (mut env_mats, mut env_vols): (Option<ResMut<Assets<super::envvol::EnvVolumeMaterial>>>, Option<ResMut<super::envvol::EnvVolumes>>),
    (mut tl_mats, mut tl_map): (Option<ResMut<Assets<super::twolayer::TwoLayerMaterial>>>, Option<ResMut<super::twolayer::TwoLayerMap>>),
    rt: Option<Res<super::rt::RtState>>,
) {
    let (Some(world), Some(data)) = (world, data) else { return };
    if !world.is_changed() && !data.is_changed() {
        return;
    }
    if existing.iter().any(|(_, r)| r.0 == data.lsn) && !world.is_changed() {
        return;
    }
    for (e, _) in &existing {
        commands.entity(e).despawn();
    }
    if let Some(v) = env_vols.as_mut() {
        v.0.clear();
        v.1.clear();
    }
    // PERF2: merged two-layer draws (gfx::twolayer) of this level
    let tl_on = !super::twolayer::disabled();
    if let Some(m) = tl_map.as_mut() {
        **m = Default::default();
    }
    let scene = LevelScene::build_from(data.lsn, &data.data, &world);
    let mut mat_cache: HashMap<(usize, Option<u32>, bool, bool, bool), (Handle<StandardMaterial>, Option<Handle<StandardMaterial>>)> = HashMap::new();
    // WIRE-C3: brightness of the unlit kinds-1/7 colour pass (`light::two_layer_k`)
    let colour_k = super::light::two_layer_k(&world);
    // LOOK: per-vertex VU1 light terms for the static colour passes (`vu1_lit_overlay`)
    let vu1_lights = super::light::vu1_light_set(&world);
    let vu1_fill = super::light::level_shadow_fill(&world);
    let vu1_fill_w = super::light::level_shadow_fill_w(&world);
    // ANIM R74: skinned prims lit per frame (DynLit)
    let dyn_lights = DynLitLights { lights: vu1_lights.clone(), fill: vu1_fill, fill_w: vu1_fill_w };
    if std::env::var_os("SLY2_R75DBG").is_some() {
        info!("R75 fill {vu1_fill:?} w {vu1_fill_w}");
        for l in super::light::level_lights(&world, &mut super::attach::BoneWorlds::default()) {
            let o = world.objects.get(l.object);
            info!("R75 light oid {} owner {:?} kind {} excl {} g {} col {:?} hsv {:?} fall {:?} {} {}", l.oid, o.and_then(|o| o.owner), l.params.kind, l.params.excluded, l.derived.g, l.derived.color, l.params.hsv, l.params.falloff, l.derived.du_fall, l.derived.ru_fall);
        }
    }
    let mut dyn_late: Vec<(Entity, usize, DynLit)> = Vec::new();
    // LOOK: raster (no GI requested at spawn): the lit grey pass of static two-layer prims takes the VU1
    // per-vertex light too (`vu1_lit_grey`, unlit material); with GI the Bevy lights / Solari stay
    // LOOK: baked under RT too (gfx::rt shader_patches shows unlit G-buffer pixels at their raster value):
    // the PS2 per-vertex light (linear falloff) instead of the inverse-square Solari light proxies
    let _ = &rt;
    let bake_grey = std::env::var("SLY2_LOOK_NOBAKE").is_err();
    let mut unlit_of: HashMap<AssetId<StandardMaterial>, Handle<StandardMaterial>> = HashMap::new();
    let root = commands
        .spawn((LevelGeometryRoot(data.lsn), Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)), Visibility::default()))
        .id();
    let mut ents = Vec::with_capacity(scene.objects.len());
    let mut bw = super::attach::BoneWorlds::default();
    let mut ink_mat: Option<Handle<StandardMaterial>> = None;
    // RENDER: mesh override mods, parsed once per slot (template copies share a slot)
    let mut mesh_mods: HashMap<String, Option<Arc<super::mesh_mod::ObjMesh>>> = HashMap::new();
    // WP-C6: scene objects, then the placed template copies (prims of their template's scene object)
    let items: Vec<(usize, usize, Affine)> = scene
        .objects
        .iter()
        .enumerate()
        .map(|(si, so)| (si, so.object, so.local))
        .chain(scene.copies.iter().map(|c| (c.source, c.object, c.world)))
        .collect();
    for (n, &(si, object, local)) in items.iter().enumerate() {
        let so = &scene.objects[si];
        let e = commands.spawn((AloObject(object), affine_to_transform(&local), Visibility::default())).id();
        if n < scene.objects.len() {
            ents.push(e);
        } else {
            commands.entity(e).insert(ChildOf(root));
        }
        let obj_world = scene.world_of(si);
        // LOOK: bone ref -> (joint entity, rest local) of this object's skinned prims (caster skinning)
        let mut joint_of: HashMap<(i32, i16), (Entity, Affine)> = HashMap::new();
        // ANIM R72: ink edge prims of this object (skinned ones get an InkSkin once joint_of is complete)
        let mut late_ink: Vec<(Entity, &super::scene::ScenePrim)> = Vec::new();
        // LOOK R31 diag: SLY2_LOOK_HIDEOID=oid,oid,.. skips those objects' prims (bisect what draws a pixel)
        if std::env::var("SLY2_LOOK_HIDEOID").is_ok_and(|v| v.split(',').any(|x| x.trim().parse::<i64>().ok() == Some(world.objects[object].oid as i64))) {
            continue;
        }
        for p in &so.prims {
            // LOOK R32 diag: SLY2_LOOK_HIDESH=id,.. skips prims whose shader id (PRIMDBG `shader` tuple .0) matches
            if let Ok(v) = std::env::var("SLY2_LOOK_HIDESH") {
                let id = p.prim.shader.and_then(|s| scene.textures.get(so.entry)?.shaders.get(s as usize)).map(|sh| sh.id as i64);
                if v.split(',').any(|x| x.trim().parse::<i64>().ok() == id && id.is_some()) {
                    continue;
                }
            }
            // WP-C4: only VU1 surface programs draw faces; shadow volumes (types 2/3) stay in
            // LevelScene (Bevy light shadows / Solari stand in for the green-channel shadow
            // pass, see C5 Findings). WP-C5: type-1 ink edges -> per-frame silhouette lines.
            if p.prim.role == super::mesh::MeshRole::InkEdges {
                let mat = ink_mat
                    .get_or_insert_with(|| materials.add(StandardMaterial { base_color: Color::BLACK, unlit: true, /* wgpu: depth bias is invalid for LineList */ ..default() }))
                    .clone();
                let m = meshes.add(line_mesh(&[]));
                let ie = commands.spawn((InkOutline(Arc::new(p.prim.clone())), Mesh3d(m), MeshMaterial3d(mat), affine_to_transform(&p.local), ChildOf(e))).id();
                late_ink.push((ie, p));
                continue;
            }
            if p.prim.role != super::mesh::MeshRole::Surface {
                continue;
            }
            // WP-A9 point-light stand-in of class-48 env volumes REPLACED by the WP-C7 volume pass
            // (gfx::envvol, every env-only prim incl. class 48; material::env_volume_light kept for tests).
            // WP-C7: per-record fog scale (FUN_00167320 f32 -> FUN_0014e270), fog::record_fog_scale
            let fog = match &world.objects[so.object].payload {
                Payload::Loaded(LoadData::Alo { alo, .. }) => alo.mesh.records.get(p.record).map_or(1.0, |r| super::fog::record_fog_scale(r.f1)),
                _ => 1.0,
            };
            let fog = super::fog::fog_enabled_for(fog);
            // WP-C9: GS As = At * Av (TFX modulate): vertex alpha below 0x80 makes a blended prim
            // translucent even with an opaque texture (Paris moonlight shafts, material::vertex_alpha)
            // LOOK: FUN_0016a000 GS vertex colour = (vcol * shader colour +8 + 0xff) >> 9 (0xff x 0xff -> 0x7f)
            let matcol = p.prim.shader.and_then(|s| scene.textures.get(so.entry)?.shaders.get(s as usize)).map(|sh| sh.colors[0]);
            // FEEL R22: the FUN_0016a000 shader alpha, only where ALPHA_1 blends with As (`gs_alpha_sa`)
            let sa = p.prim.shader.and_then(|s| scene.textures.get(so.entry)?.shaders.get(s as usize)).and_then(super::material::gs_alpha_sa);
            let valpha = super::material::vertex_alpha(&p.prim.colors, sa);
            let key = (so.entry, p.prim.shader, p.prim.cull_back, fog, valpha);
            let (mat, overlay) = mat_cache
                .entry(key)
                .or_insert_with(|| {
                    let (mut m, ov) = make_material(&scene, so.entry, p.prim.shader, p.prim.cull_back, valpha, loader.as_deref(), &mut images);
                    m.fog_enabled &= fog;
                    // LOOK R32 diag: SLY2_LOOK_MODENOFOG=add|blend drops fog on that alpha mode's level materials
                    if let Ok(v) = std::env::var("SLY2_LOOK_MODENOFOG") {
                        if (v == "add" && m.alpha_mode == AlphaMode::Add) || (v == "blend" && m.alpha_mode == AlphaMode::Blend) {
                            m.fog_enabled = false;
                        }
                    }
                    // WP-C7: env-only shaders -> light volume (gfx::envvol), level material drawn invisible
                    let sh = p.prim.shader.and_then(|s| scene.textures.get(so.entry)?.shaders.get(s as usize));
                    if let (true, Some(env_mats), Some(env_vols)) = (sh.is_some_and(|sh| super::material::env_glow(sh).is_some()) && !super::envvol::c7_before(), env_mats.as_mut(), env_vols.as_mut()) {
                        let v = env_mats.add(super::envvol::volume_material(&m));
                        let pair = super::envvol::pair_materials(&m).map(|(f, b)| (env_mats.add(f), env_mats.add(b)));
                        if std::env::var_os("SLY2_LOOK_ENVLOG").is_some() {
                            log::info!("ENVVOL oid {} cls {:?} rec {} alpha {:?} tint {:?} pair {}", world.objects[object].oid, world.objects[object].class, p.record, m.alpha_mode, m.base_color.to_srgba(), pair.is_some());
                        }
                        // LOOK R32 diag: SLY2_LOOK_ENVSKIP=oid,.. drops those objects' volumes (incl. npcdraw copies)
                        if std::env::var("SLY2_LOOK_ENVSKIP").is_ok_and(|v| v.split(',').any(|x| x.trim() == "all" || x.trim().parse::<i64>().ok() == Some(world.objects[object].oid as i64))) {
                            super::envvol::hide_material(&mut m);
                            return (materials.add(m), None);
                        }
                        super::envvol::hide_material(&mut m);
                        let h = materials.add(m);
                        env_vols.0.insert(h.id(), v);
                        if let Some(pr) = pair {
                            env_vols.1.insert(h.id(), pr);
                        }
                        return (h, None);
                    }
                    let ov = ov.map(|mut o| {
                        o.fog_enabled = fog;
                        o
                    });
                    let merged = ov.as_ref().filter(|_| tl_on).and_then(|o| super::twolayer::merged_material(&m, o));
                    // FEEL R21: Z-writing twin of a Blend prim whose TEST_1 writes Z for As >= AREF
                    let zref = sh.and_then(|sh| super::material::material_from(sh, true).z_aref).filter(|_| m.alpha_mode == AlphaMode::Blend);
                    let ztwin = zref.map(|r| super::twolayer::z_twin_material(&m, r));
                    let (h, ov) = (materials.add(m), ov.map(|o| materials.add(o)));
                    if let (Some(t), Some(map)) = (ztwin, tl_map.as_mut()) {
                        map.zwrite.insert(h.id(), materials.add(t));
                    }
                    if let (Some(mm), Some(o), Some(tm), Some(map)) = (merged, ov.as_ref(), tl_mats.as_mut(), tl_map.as_mut()) {
                        map.mats.insert(h.id(), (tm.add(mm), o.id()));
                        map.overlays.insert(o.id());
                    }
                    (h, ov)
                })
                .clone();
            // WIRE-C3: two-layer prims: the lit grey pass gets only the vertex colour brightness (n.w),
            // the unlit colour pass the full vertex colour * k (`two_layer_split_colors`)
            // WP-C9: then the display-space PS2 colour factors -> linear (`linear_vertex_colors`)
            let two = overlay.is_some();
            let fix_colors = |mesh: Mesh| linear_vertex_colors(if two { two_layer_split_colors(gs_vertex_alpha(mesh, sa), None) } else { gs_vertex_colors(gs_vertex_alpha(mesh, sa), matcol) });
            let fix_overlay = |mesh: Mesh| linear_vertex_colors(two_layer_split_colors(gs_vertex_colors(gs_vertex_alpha(mesh, sa), matcol), Some(colour_k)));
            // WP-C5: prims with weights -> skinned mesh, joints = entities for the record's
            // weight_refs bones (children of the object entity), driven by anim::skin::AnimSkin.
            if let Some(skinned) = crate::engine::anim::skin_mesh::bevy_skinned_mesh(&p.prim) {
                let Payload::Loaded(LoadData::Alo { alo, .. }) = &world.objects[so.object].payload else { unreachable!() };
                let refs = super::skin::record_weight_refs(&alo.mesh.records[p.record].data);
                let sj = super::skin::skin_joints(&world, &mut bw, so.object, refs);
                let locals: Vec<Affine> = sj.rest_world.iter().map(|w| super::skin::joint_local(w.as_ref(), &obj_world, &p.local)).collect();
                let joints: Vec<Entity> = locals.iter().map(|l| commands.spawn((affine_to_transform(l), ChildOf(e))).id()).collect();
                for (k, r) in refs.iter().enumerate() {
                    joint_of.entry(*r).or_insert((joints[k], locals[k]));
                }
                let ibp: Vec<Mat4> = locals.iter().map(|l| affine_to_transform(&super::skin::inverse_bindpose(&p.local, l)).to_matrix()).collect();
                let bind = locals.iter().map(crate::engine::anim::pose::from_4x3).collect::<Vec<_>>();
                let skin = crate::engine::anim::skin::AnimSkin { palette: vec![crate::engine::anim::pose::IDENTITY; bind.len()], bind };
                // WP-A10: Bevy culls a skinned mesh by the Aabb of its raw positions in the entity frame,
                // which is the prim-local rest frame (p.local is only in the joints / inverse bindposes) and
                // ignores the pose -> posed models were culled up to ~200 units off (pop-in at the screen
                // edges, worst at 32:9). Per-joint bounds + DynamicSkinnedMeshBounds refresh it every frame.
                let raw = strip_env_colors(&scene, so.entry, p.prim.shader, skinned);
                // ANIM R74: per-frame VU1 light (DynLit) for the baked kinds (same rule as the static prims)
                let dyn_lit = {
                    let m = materials.get(&mat);
                    let opaque = m.is_some_and(|m| matches!(m.alpha_mode, AlphaMode::Opaque | AlphaMode::Mask(_)));
                    let blend = m.is_some_and(|m| m.alpha_mode == AlphaMode::Blend);
                    let sh0 = p.prim.shader.and_then(|s| scene.textures.get(so.entry)?.shaders.get(s as usize)).is_some_and(|sh| sh.kind == 0);
                    (bake_grey && std::env::var_os("SLY2_R74LITOFF").is_none() && (two || sh0) && (opaque || blend)).then_some(blend && !opaque)
                };
                let mut mat = mat;
                let mut dyn_comp = None;
                if let Some(keep_alpha) = dyn_lit {
                    use bevy::mesh::VertexAttributeValues as V;
                    let cols = |m: Mesh| match m.attribute(Mesh::ATTRIBUTE_COLOR) {
                        Some(V::Float32x4(c)) => c.clone(),
                        _ => Vec::new(),
                    };
                    let grey_src = cols(if two { raw.clone() } else { gs_vertex_colors(raw.clone(), matcol) });
                    let ov_src = two.then(|| cols(gs_vertex_colors(gs_vertex_alpha(raw.clone(), sa), matcol)));
                    if std::env::var_os("SLY2_R75DBG").is_some() {
                        info!("R75 dynlit two {two} n {} grey {:?} ov {:?}", grey_src.len(), grey_src.iter().take(3).collect::<Vec<_>>(), ov_src.as_ref().map(|o| o.iter().take(3).cloned().collect::<Vec<_>>()));
                    }
                    let alpha = keep_alpha.then(|| match fix_colors(raw.clone()).attribute(Mesh::ATTRIBUTE_COLOR) {
                        Some(V::Float32x4(c)) => c.iter().map(|c| c[3]).collect(),
                        _ => Vec::new(),
                    });
                    let v3 = |a: Option<&V>| match a {
                        Some(V::Float32x3(v)) => v.iter().map(|x| Vec3::from(*x)).collect(),
                        _ => Vec::new(),
                    };
                    let (ji, jw) = match (raw.attribute(Mesh::ATTRIBUTE_JOINT_INDEX), raw.attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT)) {
                        (Some(V::Uint16x4(a)), Some(V::Float32x4(b))) => (a.clone(), b.clone()),
                        _ => (Vec::new(), Vec::new()),
                    };
                    let skin = super::shadowvol_pass::CasterSkin { joints: joints.clone(), ibp: ibp.clone(), ji, jw };
                    dyn_comp = Some(DynLit { skin, pos: v3(raw.attribute(Mesh::ATTRIBUTE_POSITION)), nrm: v3(raw.attribute(Mesh::ATTRIBUTE_NORMAL)), grey_src, ov_src, alpha, root: e });
                    mat = unlit_twin(&mat, &mut materials, &mut unlit_of, tl_mats.as_deref_mut(), tl_map.as_deref_mut(), tl_on);
                }
                let ibp = inverse_bindposes.add(SkinnedMeshInverseBindposes::from(ibp));
                let mut skinned = fix_colors(raw.clone());
                // ANIM R74: rest-pose colours until the first update_dyn_lit
                let dyn_init = dyn_comp.as_ref().map(|d| {
                    let pw = super::scene::mul(&p.local, &obj_world);
                    let at = obj_world[3];
                    let pos: Vec<[f32; 3]> = d.pos.iter().map(|v| [0, 1, 2].map(|j| v.x * pw[0][j] + v.y * pw[1][j] + v.z * pw[2][j] + pw[3][j])).collect();
                    let nrm: Vec<[f32; 3]> = d.nrm.iter().map(|v| Vec3::from([0, 1, 2].map(|j| v.x * pw[0][j] + v.y * pw[1][j] + v.z * pw[2][j])).normalize_or(Vec3::Z).to_array()).collect();
                    dyn_lit_colors(d, &dyn_lights, &pos, &nrm, at)
                });
                if let Some((g, _)) = &dyn_init {
                    skinned.insert_attribute(Mesh::ATTRIBUTE_COLOR, g.clone());
                }
                let _ = skinned.generate_skinned_mesh_bounds();
                let dyn_ov = dyn_init.as_ref().and_then(|(_, o)| o.as_ref()).map(|(oc, av)| {
                    fix_overlay(raw.clone()).with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, oc.clone()).with_inserted_attribute(Mesh::ATTRIBUTE_TANGENT, av.iter().map(|&a| [1.0, 0.0, 0.0, a]).collect::<Vec<[f32; 4]>>())
                });
                let merged = match &dyn_ov {
                    Some(o) => tl_on.then(|| super::twolayer::merged_mesh_baked(&skinned, o, dyn_comp.as_ref().is_some_and(|d| d.alpha.is_some()))).flatten(),
                    _ => (two && tl_on).then(|| super::twolayer::merged_mesh(&skinned, &fix_overlay(raw.clone()))).flatten(),
                };
                let base_mesh = meshes.add(skinned);
                if let (Some(mm), Some(map)) = (merged, tl_map.as_mut()) {
                    map.meshes.insert(base_mesh.id(), meshes.add(mm));
                }
                let main = commands
                    .spawn((
                        Mesh3d(base_mesh),
                        MeshMaterial3d(mat),
                        SkinnedMesh { inverse_bindposes: ibp.clone(), joints: joints.clone() },
                        bevy::camera::visibility::DynamicSkinnedMeshBounds,
                        skin,
                        Transform::IDENTITY,
                        ChildOf(e),
                    ))
                    .id();
                if let Some(d) = dyn_comp {
                    if n < scene.objects.len() {
                        dyn_late.push((main, si, d));
                    } else {
                        commands.entity(main).insert(d);
                    }
                }
                // WIRE-C3: the colour pass is a CHILD of the lit prim (the object's own children stay one
                // Mesh3d per prim, which playerdraw/npcdraw zip with so.prims); same joints = same pose
                if let Some(ov) = overlay {
                    let mut o = dyn_ov.unwrap_or_else(|| fix_overlay(raw));
                    let _ = o.generate_skinned_mesh_bounds();
                    commands.spawn((
                        ColourPass,
                        Mesh3d(meshes.add(o)),
                        MeshMaterial3d(ov),
                        SkinnedMesh { inverse_bindposes: ibp, joints },
                        bevy::camera::visibility::DynamicSkinnedMeshBounds,
                        Transform::IDENTITY,
                        ChildOf(main),
                    ));
                }
                continue;
            }
            // RENDER: mods/<mod>/meshes/<lsn>/<object>_<record>.obj replaces this prim's geometry
            // (env SLY2_LIST_MESHES=1 logs every overridable path with its vertex count)
            if std::env::var_os("SLY2_LIST_MESHES").is_some() {
                let (lo, hi) = p.prim.positions.iter().fold(([f32::MAX; 3], [f32::MIN; 3]), |(lo, hi), v| (std::array::from_fn(|k| lo[k].min(v[k])), std::array::from_fn(|k| hi[k].max(v[k]))));
                log::info!("mesh slot {} verts {} bounds {lo:?}..{hi:?}", super::mesh_mod::mesh_override_path(scene.lsn, object, p.record), p.prim.positions.len());
            }
            let slot = super::mesh_mod::mesh_override_path(scene.lsn, object, p.record);
            let over = mesh_mods.entry(slot).or_insert_with_key(|slot| {
                loader.as_deref().and_then(|l| l.mods.resolve(slot)).and_then(|f| {
                    let r = std::fs::read_to_string(&f).map_err(|e| e.to_string()).and_then(|t| super::mesh_mod::parse_obj(&t));
                    match &r {
                        Ok(o) => log::info!("mesh mod {}: {} triangles", f.display(), o.positions.len() / 3),
                        Err(e) => log::warn!("mesh mod {}: {e}", f.display()),
                    }
                    r.ok().map(Arc::new)
                })
            }).clone();
            let raw = match &over {
                Some(o) => super::mesh_mod::to_bevy_mesh(o.as_ref()),
                None => strip_env_colors(&scene, so.entry, p.prim.shader, to_bevy_mesh(&p.prim)),
            };
            let tag = super::pick::PrimTag { object, oid: world.objects[object].oid, entry: so.entry, shader: p.prim.shader, record: p.record };
            let mut mesh = fix_colors(raw.clone());
            let mut tf = affine_to_transform(&p.local);
            // WP-C8: translucent prims sort by their record centre (gfx::screen::sort_centre)
            if materials.get(&mat).is_some_and(|m| matches!(m.alpha_mode, AlphaMode::Blend | AlphaMode::Add | AlphaMode::Multiply)) {
                if let Some(bevy::mesh::VertexAttributeValues::Float32x3(v)) = mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION) {
                    let c = super::screen::sort_centre(v);
                    v.iter_mut().for_each(|p| (0..3).for_each(|k| p[k] -= c[k]));
                    tf = tf.mul_transform(Transform::from_translation(Vec3::from_array(c)));
                }
            }
            // LOOK: static prims: the colour pass k per vertex from the object's 4 VU1 lights (FUN_001a3f78)
            let prim_world = super::scene::mul(&p.local, &if n < scene.objects.len() { obj_world } else { local });
            // LOOK: light set position = the owner-chain root's world translation (FUN_001f2108)
            let obj_at = if n < scene.objects.len() {
                let mut r = si;
                while let Some(o) = scene.objects[r].owner.filter(|&o| o < scene.objects.len() && o != r) {
                    r = o;
                }
                scene.world_of(r)[3]
            } else {
                local[3]
            };
            // LOOK R17: unlit prelit prims (Mesh0 flags bit0 clear, Material0 & 0x20): the VU1 draws the
            // FUN_0016c8e0 colours as-is, no light term (`prelit_colors`)
            let no_pre: Vec<[[f32; 4]; 2]> = Vec::new();
            let pre = if std::env::var_os("SLY2_LOOK_NOPRELIT").is_some() { &no_pre } else { &p.prim.prelit };
            let fix_overlay = |mesh: Mesh| if pre.is_empty() { vu1_lit_overlay(gs_vertex_colors(gs_vertex_alpha(mesh, sa), matcol), &vu1_lights, vu1_fill_w, &prim_world, obj_at) } else { prelit_colors(gs_vertex_alpha(mesh, sa), pre, 1) };
            // LOOK: opaque two-layer base under raster: grey = n.w * min(L, 2) per vertex (VU1 0x1050..0x1320,
            // vf30), unlit, two draws (opaque grey then the additive colour pass: the PS2 order)
            let opaque = materials.get(&mat).is_some_and(|m| matches!(m.alpha_mode, AlphaMode::Opaque | AlphaMode::Mask(_)));
            // LOOK R26: translucent (Blend) two-layer prims too: the VU1 draws the same unlit grey pass,
            // only the GS blend differs (ALPHA 0x44, As = Ta * Av with Av the material-scaled vertex alpha);
            // the Bevy-lit path added PBR specular (hub 020 awning glass: bright cyan sheet vs faint on PS2)
            let blend = materials.get(&mat).is_some_and(|m| m.alpha_mode == AlphaMode::Blend) && std::env::var_os("SLY2_LOOK_NOBAKEBLEND").is_none();
            let baked = bake_grey && two && over.is_none() && (opaque || blend);
            // LOOK: single-layer (kind 0) opaque prims: the same VU1 lit pass (grey = vertex colour * min(L,2))
            let sh0 = p.prim.shader.and_then(|s| scene.textures.get(so.entry)?.shaders.get(s as usize)).is_some_and(|sh| sh.kind == 0);
            // LOOK R38: translucent kind-0 prims too: the VU1 light term follows the mesh (Mesh0 flags bit0 /
            // prelit, R17), the GS ALPHA only blends the result (R26 for two-layer); the Bevy-lit path drew
            // them ~2x too dim (EP4 job-marker glow quad 30177). Kill switch SLY2_LOOK_NOBAKE0BLEND.
            let baked0 = bake_grey && !two && sh0 && over.is_none() && (opaque || (blend && std::env::var_os("SLY2_LOOK_NOBAKE0BLEND").is_none())) && std::env::var_os("SLY2_LOOK_NOBAKE0").is_none();
            if let Some(q) = std::env::var("SLY2_LOOK_PRIMDBG").ok().map(|v| v.split(',').filter_map(|x| x.trim().parse::<f32>().ok()).collect::<Vec<_>>()).filter(|q| q.len() == 4) {
                let ps = &p.prim.positions;
                let c = ps.iter().fold([0.0f32; 3], |a, v| [a[0] + v[0], a[1] + v[1], a[2] + v[2]]).map(|x| x / ps.len().max(1) as f32);
                let w = [0, 1, 2].map(|j| c[0] * prim_world[0][j] + c[1] * prim_world[1][j] + c[2] * prim_world[2][j] + prim_world[3][j]);
                let d = ((w[0] - q[0]).powi(2) + (w[1] - q[1]).powi(2) + (w[2] - q[2]).powi(2)).sqrt();
                if d < q[3] {
                    let shd = p.prim.shader.and_then(|s| scene.textures.get(so.entry)?.shaders.get(s as usize)).map(|sh| (sh.id, sh.kind, sh.flags, sh.colors, sh.frames.len()));
                    log::info!("PRIMDBG oid {} obj {object} cls {:?} rec {} inst {:?} at {w:?} nv {} flags {:#x} pre {} two {two} baked {baked} baked0 {baked0} sh0 {sh0} at_root {obj_at:?} entry {} shader {:?} {shd:?} alpha {:?} sa {sa:?} matcol {matcol:?} overlay {} pre3 {:?}", world.objects[object].oid, world.objects[object].class, p.record, p.instance, ps.len(), p.prim.flags, pre.len(), so.entry, p.prim.shader, materials.get(&mat).map(|m| m.alpha_mode), overlay.is_some(), &pre[..pre.len().min(60)]);
                    if std::env::var_os("SLY2_LOOK_PRIMDBGV").is_some() { log::info!("PRIMDBGV {:?} uv {:?} world {:?}", p.prim.positions, p.prim.uvs, prim_world); }
                }
            }
            let mut mat = mat;
            if baked || baked0 {
                let grey = if !pre.is_empty() { prelit_colors(raw.clone(), pre, 0) } else if baked { vu1_lit_grey(raw.clone(), &vu1_lights, vu1_fill, vu1_fill_w, &prim_world, obj_at) } else { vu1_lit_vcol(gs_vertex_colors(raw.clone(), matcol), &vu1_lights, vu1_fill, vu1_fill_w, &prim_world, obj_at) };
                if let (Some(bevy::mesh::VertexAttributeValues::Float32x4(g)), Some(bevy::mesh::VertexAttributeValues::Float32x4(c))) = (grey.attribute(Mesh::ATTRIBUTE_COLOR), mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR)) {
                    // LOOK R26: a blended grey pass keeps its GS vertex alpha (gs_vertex_alpha), the blend As
                    for (c, g) in c.iter_mut().zip(g) {
                        *c = [g[0], g[1], g[2], if blend && !opaque { c[3] } else { g[3] }];
                    }
                }
                let lit_id = mat.id();
                mat = unlit_of
                    .entry(lit_id)
                    .or_insert_with(|| {
                        let mut u = materials.get(&mat).cloned().unwrap_or_default();
                        u.unlit = true;
                        // LOOK (perf): keep the one-draw TwoLayer merge (baked grey mode)
                        let mo = tl_map.as_ref().and_then(|m| m.mats.get(&lit_id)).map(|(_, o)| *o);
                        let merged = mo.and_then(|o| materials.get(o).cloned()).filter(|_| tl_on).and_then(|o| super::twolayer::merged_material(&u, &o));
                        let zt = tl_map.as_ref().and_then(|m| m.zwrite.get(&lit_id)).cloned();
                        let h = materials.add(u);
                        if let (Some(t), Some(map)) = (zt, tl_map.as_mut()) {
                            map.zwrite.insert(h.id(), t);
                        }
                        if let (Some(mm), Some(o), Some(tm), Some(map)) = (merged, mo, tl_mats.as_mut(), tl_map.as_mut()) {
                            map.mats.insert(h.id(), (tm.add(mm), o));
                        }
                        h
                    })
                    .clone();
            }
            let merged = (two && tl_on).then(|| if baked { super::twolayer::merged_mesh_baked(&mesh, &fix_overlay(raw.clone()), blend && !opaque) } else { super::twolayer::merged_mesh(&mesh, &fix_overlay(raw.clone())) }).flatten();
            let m = meshes.add(mesh);
            if let (Some(mm), Some(map)) = (merged, tl_map.as_mut()) {
                map.meshes.insert(m.id(), meshes.add(mm));
            }
            let main = commands.spawn((Mesh3d(m), MeshMaterial3d(mat), tf, ChildOf(e), tag)).id();
            if let Some(ov) = overlay {
                // WIRE-C3: colour pass as a child of the lit prim (undo the sort-centre shift of `tf`)
                let rel = Transform::from_matrix(tf.to_matrix().inverse() * affine_to_transform(&p.local).to_matrix());
                commands.spawn((ColourPass, Mesh3d(meshes.add(fix_overlay(raw))), MeshMaterial3d(ov), rel, ChildOf(main)));
            }
        }
        // LOOK: type-2 shadow casters (FUN_0016caa0 edge lists) -> shadowvol_pass::ShadowCaster
        if std::env::var("SLY2_LOOK_NOSHVOL").is_err() && std::env::var("SLY2_LOOK_NOCASTERS").is_err() {
            // ANIM R71: the strips of one record (same record and placement) weld into one volume
            let mut welds: Vec<(usize, Mat4, Vec<Vec3>)> = Vec::new();
            for p in so.prims.iter().filter(|p| p.prim.role == super::mesh::MeshRole::ShadowVolume && !p.prim.edges.is_empty()) {
                let lm = affine_to_transform(&p.local).to_matrix();
                let radius = p.prim.positions.iter().map(|v| lm.transform_point3(Vec3::from(*v)).length()).fold(0.0f32, f32::max);
                let gi = welds.iter().position(|g| g.0 == p.record && g.1 == lm).unwrap_or_else(|| {
                    welds.push((p.record, lm, Vec::new()));
                    welds.len() - 1
                });
                let map: Vec<u32> = p
                    .prim
                    .positions
                    .iter()
                    .map(|v| {
                        let (v, g) = (Vec3::from(*v), &mut welds[gi].2);
                        g.iter().position(|x| x.distance_squared(v) < 1e-6).unwrap_or_else(|| {
                            g.push(v);
                            g.len() - 1
                        }) as u32
                    })
                    .collect();
                let mut caster = super::shadowvol_pass::ShadowCaster { pos: p.prim.positions.iter().map(|v| Vec3::from(*v)).collect(), edges: p.prim.edges.clone(), skin: None, root: e, radius, weld: Some((gi, map)) };
                if let Some((sk, pos)) = prim_skin(&mut commands, &world, &mut bw, so.object, p, &joint_of, &obj_world, e) {
                    caster.pos = pos;
                    caster.skin = Some(sk);
                }
                let tf = if caster.skin.is_some() { Transform::IDENTITY } else { affine_to_transform(&p.local) };
                // ANIM R75: record id so playerdraw hides the casters of undrawn mesh records with their surfaces
                let rec = super::shadowvol_pass::CasterRecord { object, record: p.record, instance: p.instance };
                commands.spawn((caster, rec, tf, Visibility::default(), ChildOf(e)));
            }
        }
        // ANIM R72: skinned ink edges posed like the casters (FUN_0016e438 type 1 records are VU-skinned
        // like the surfaces); SLY2_R72INKOFF: not drawn (old behaviour, playerdraw hides them)
        if std::env::var_os("SLY2_R72INKOFF").is_none() {
            for (ie, p) in late_ink {
                if let Some((sk, pos)) = prim_skin(&mut commands, &world, &mut bw, so.object, p, &joint_of, &obj_world, e) {
                    commands.entity(ie).insert(InkSkin { skin: sk, pos });
                }
            }
        }
    }
    for (i, so) in scene.objects.iter().enumerate() {
        let parent = so.owner.map(|o| ents[o]).unwrap_or(root);
        commands.entity(ents[i]).insert(ChildOf(parent));
    }
    // ANIM R74: DynLit light set position = the owner-chain root (FUN_001f2108), as the static prims
    for (main, si, mut d) in dyn_late {
        let mut r = si;
        while let Some(o) = scene.objects[r].owner.filter(|&o| o < scene.objects.len() && o != r) {
            r = o;
        }
        // ANIM R75: skinned actors light at their own object (like the casters, FUN_001a4478 root =
        // the drawn ALO's position): the owner-chain top stays at its level spawn / origin while the
        // player rig moves (Sly lit with the lights of the level origin -> dark grey, PCSX2 cap_feel_hub
        // 004: vivid blue). SLY2_R75TOPROOT = old owner-chain root.
        if std::env::var_os("SLY2_R75TOPROOT").is_some() {
            d.root = ents[r];
        }
        commands.entity(main).insert(d);
    }
    commands.insert_resource(dyn_lights);
    // LOOK: shadow volumes (types 2/3) -> the FUN_0016dd58 CLUT resolve as decals on static surfaces
    if std::env::var("SLY2_LOOK_NOSHVOL").is_err() && std::env::var("SLY2_LOOK_SHVOL_BAKE").is_err() {
        // LOOK: the real per-frame count pass (shadowvol_pass)
        static GEN: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);
        let (rgb, a1) = super::shadowvol::resolve_colour(&world);
        let rgb = if super::light::display_space() { rgb } else { rgb.map(|c| Color::srgb(c, c, c).to_linear().red) };
        let vols = super::shadowvol::static_volumes(&scene);
        commands.insert_resource(super::shadowvol_pass::CastLights::from_world(&world));
        let mut svs = super::shadowvol_pass::ShadowVolumeScene::new(vols, rgb, a1, GEN.fetch_add(1, std::sync::atomic::Ordering::Relaxed));
        // LOOK R24: colours 0 / 2 of the resolve CLUT (frame-alpha indexed, `shadowvol::clut_index`)
        for k in [0, 2] {
            let c = super::shadowvol::resolve_clut(&world)[k];
            let rgb = if super::light::display_space() { [c[0], c[1], c[2]] } else { [c[0], c[1], c[2]].map(|x| Color::srgb(x, x, x).to_linear().red) };
            svs.clut[k] = [rgb[0], rgb[1], rgb[2], c[3]];
        }
        commands.insert_resource(svs);
    } else if std::env::var("SLY2_LOOK_NOSHVOL").is_err() {
        let (rgb, a1) = super::shadowvol::resolve_colour(&world);
        let (vp, vn, va, ix) = super::shadowvol::static_decals(&scene, 0.5, a1, 4.0);
        if !ix.is_empty() {
            let vc: Vec<[f32; 4]> = va.iter().map(|&a| [1.0, 1.0, 1.0, a]).collect();
            let m = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
                .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vp)
                .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vn)
                .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, vc)
                .with_inserted_indices(Indices::U32(ix));
            let mat = materials.add(StandardMaterial {
                base_color: if super::light::display_space() { Color::linear_rgb(rgb[0], rgb[1], rgb[2]) } else { Color::srgb(rgb[0], rgb[1], rgb[2]) },
                fog_enabled: false,
                unlit: true,
                alpha_mode: AlphaMode::Blend,
                depth_bias: 64.0,
                cull_mode: None,
                ..default()
            });
            commands.spawn((Mesh3d(meshes.add(m)), MeshMaterial3d(mat), Transform::IDENTITY, bevy::light::NotShadowCaster, ChildOf(root)));
        }
    }
    spawn_level_lights(&mut commands, &world, root);
}

/// One spawned LIGHT (world object index).
#[derive(Component)]
pub struct LevelLightObject(pub usize);

/// WP-C4: live LIGHT state (the original object's members), re-derived on every setter.
#[derive(Component, Clone, Debug)]
pub struct LightLive {
    pub params: super::light::LightParams,
    pub world: Affine,
}

/// One runtime LIGHT property write (field table 0x2ac880 ids 151..158, raw value bytes as the
/// property stream stores them) or a new world matrix (`world`, the post-load / transform
/// virtuals slots 10/14/16/28/29 that also re-run FUN_001a3968).
#[derive(Clone, Debug)]
pub struct LightPropSet {
    pub object: usize,
    pub field: i16,
    pub value: Vec<u8>,
    pub world: Option<Affine>,
}

/// Queue filled by the runtime property path (Splice / game logic); drained by
/// `apply_light_props` every frame.
#[derive(Resource, Default)]
pub struct LightPropQueue(pub Vec<LightPropSet>);

/// WP-C4: PS2 light unit in lux, derived from the VU1 lighting scale and Bevy's default camera
/// exposure (`light::ps2_light_unit_lux`, replaces the C3 guess 10000): 3135.6 lux.
pub fn ps2_light_lux() -> f32 {
    super::light::ps2_light_unit_lux(super::light::BEVY_DEFAULT_EV100)
}

/// The Bevy light for a LIGHT state (None = the original never lights with it: excluded,
/// kind other than 0/1, zero colour, or zero direction).
pub enum BevyLight {
    Directional(DirectionalLight, Transform),
    Point(PointLight, Transform),
}

/// kind 0 -> DirectionalLight shining along world row 0 (+0x140); kind 1 -> PointLight at the
/// world position, range = falloff far, intensity chosen so the illuminance at `near` matches the
/// PS2 full-strength value.
pub fn bevy_light(p: &super::light::LightParams, world: &Affine) -> Option<BevyLight> {
    if p.excluded {
        return None;
    }
    let d = super::light::ee_sub_1a3968(p, world, false);
    // WP-C9: VU1 terms (x2 scale, ambient part du split off for directional lights) in display
    // units -> linear (`light::ps2_linear`); the facing total stays exact: ambient^2.2 (global
    // ambient, `level_ambient`) + (facing^2.2 - ambient^2.2) (Lambert at full facing).
    let (amb, face) = super::light::vu1_light_terms(&d);
    let c: [f32; 3] = match p.kind {
        0 => [0, 1, 2].map(|k| (super::light::ps2_linear(face[k]) - super::light::ps2_linear(amb[k])).max(0.0)),
        _ => face.map(super::light::ps2_linear),
    };
    let strength = c.iter().cloned().fold(0.0f32, f32::max);
    if strength <= 0.0 {
        return None;
    }
    let color = Color::linear_rgb(c[0] / strength, c[1] / strength, c[2] / strength);
    let v = Vec3::from(d.vec);
    let lux = ps2_light_lux();
    match p.kind {
        0 if v.length_squared() > 1e-12 => Some(BevyLight::Directional(
            DirectionalLight { color, illuminance: lux * strength, shadow_maps_enabled: false, ..default() },
            Transform::IDENTITY.looking_to(v.normalize(), Vec3::Z),
        )),
        1 => {
            let near = p.falloff[0].max(1.0);
            Some(BevyLight::Point(
                PointLight {
                    color,
                    intensity: lux * strength * 4.0 * std::f32::consts::PI * near * near,
                    range: p.falloff[1].max(near),
                    shadow_maps_enabled: false,
                    ..default()
                },
                Transform::from_translation(v),
            ))
        }
        _ => None,
    }
}

/// `level_ambient` -> Bevy resource.
pub fn ambient_resource(a: [f32; 3]) -> GlobalAmbientLight {
    let m = a[0].max(a[1]).max(a[2]);
    let color = if m > 0.0 { Color::linear_rgb(a[0] / m, a[1] / m, a[2] / m) } else { Color::WHITE };
    GlobalAmbientLight { color, brightness: m * ps2_light_lux() / std::f32::consts::PI, ..default() }
}

fn insert_bevy_light(e: &mut EntityCommands, l: Option<BevyLight>) {
    // RENDER: only remove the OTHER light kind; a remove + re-insert of the same kind made the RT
    // point-light proxy (gfx::rt sync_point_light_proxies) despawn/respawn on every light property write
    match l {
        Some(BevyLight::Directional(d, t)) => {
            e.remove::<PointLight>().insert((d, t));
        }
        Some(BevyLight::Point(p, t)) => {
            e.remove::<DirectionalLight>().insert((p, t));
        }
        None => {
            e.remove::<(DirectionalLight, PointLight)>();
        }
    }
}

/// WP-C3/C4: class-4 lights -> entities under the level root (PS2 Z-up space). Every LIGHT gets
/// an entity with `LightLive` (so later property writes can switch it on); only lights the
/// original would put into an active set carry a Bevy light.
pub fn spawn_level_lights(commands: &mut Commands, world: &World, root: Entity) {
    let mut bw = super::attach::BoneWorlds::default();
    let lights = super::light::level_lights(world, &mut bw);
    // WP-C9: the level's directional lights set the ambient (replaces the bin's fixed 600 nits + sun)
    let set: Vec<_> = lights.iter().map(|l| (l.params, l.world)).collect();
    let amb = super::light::level_ambient(&set);
    commands.insert_resource(ambient_resource(amb));
    for l in lights {
        if std::env::var_os("SLY2_LOOK_LDUMP").is_some() { log::info!("LDUMP obj {} oid {} {:?}", l.object, l.oid, l.params); }
        let mut e = commands.spawn((LevelLightObject(l.object), LightLive { params: l.params, world: l.world }, Transform::IDENTITY, ChildOf(root)));
        insert_bevy_light(&mut e, bevy_light(&l.params, &l.world));
    }
}

/// WP-C4: runtime light property re-application. Each write runs the original setter
/// (`light::set_field`) and then FUN_001a3968 again (as every LIGHT setter does), and the Bevy
/// light is rebuilt from the new state (raster and Solari share the entity).
pub fn apply_light_props(mut commands: Commands, queue: Option<ResMut<LightPropQueue>>, mut lights: Query<(Entity, &LevelLightObject, &mut LightLive)>) {
    let Some(mut queue) = queue else { return };
    if queue.0.is_empty() {
        return;
    }
    for w in queue.0.drain(..) {
        for (e, o, mut live) in lights.iter_mut() {
            if o.0 != w.object {
                continue;
            }
            let mut changed = super::light::set_field(&mut live.params, w.field, &w.value);
            if let Some(m) = w.world {
                live.world = m;
                changed = true;
            }
            if changed {
                insert_bevy_light(&mut commands.entity(e), bevy_light(&live.params, &live.world));
            }
        }
    }
}

/// ANIM R72: a skinned type-1 ink edge prim: bind positions (`bevy_skinned_mesh` order = the prim's) and
/// the joints, posed on the CPU each frame in `update_ink_outlines` (as the type-2 casters).
#[derive(Component, Clone)]
pub struct InkSkin {
    pub skin: super::shadowvol_pass::CasterSkin,
    pub pos: Vec<Vec3>,
}

/// ANIM R74: a skinned surface prim lit per frame like the VU1 does for every draw (FUN_001a3f78 picks the
/// object's 4 lights at the owner-chain root, the lit pass = grey (kinds 1/7) or vcol (kind 0) * min(L, 2),
/// the colour pass k per vertex) instead of the Bevy lights + level-wide k (Sly near full-bright in the
/// dark hub, PLAYTEST 10-21 21:45). Bind positions / normals in the prim frame (`bevy_skinned_mesh`), posed on
/// the CPU each frame (`update_dyn_lit`). Opt-out SLY2_R74LITOFF.
#[derive(Component, Clone)]
pub struct DynLit {
    pub skin: super::shadowvol_pass::CasterSkin,
    pub pos: Vec<Vec3>,
    pub nrm: Vec<Vec3>,
    /// lit pass source colours (two-layer: raw colours, grey weight; kind 0: GS colours)
    pub grey_src: Vec<[f32; 4]>,
    /// colour pass source colours (GS colours), two-layer only
    pub ov_src: Option<Vec<[f32; 4]>>,
    /// blended lit pass keeps its GS vertex alpha
    pub alpha: Option<Vec<f32>>,
    pub root: Entity,
}

/// ANIM R74: the level's VU1 light set for `update_dyn_lit`.
#[derive(Resource, Clone)]
pub struct DynLitLights {
    pub lights: Vec<super::light::Vu1Light>,
    pub fill: [f32; 3],
    pub fill_w: f32,
}

/// ANIM R74: (lit pass COLOR, colour pass (COLOR, Av)) of a `DynLit` prim for the posed level-space vertices
/// (`pos` world, `nrm` unit), the per-vertex math of `vu1_lit_grey` / `vu1_lit_vcol` / `vu1_lit_overlay` with
/// the lights picked at `at` (no Mesh clones: runs every frame).
pub fn dyn_lit_colors(d: &DynLit, l: &DynLitLights, pos: &[[f32; 3]], nrm: &[[f32; 3]], at: [f32; 3]) -> (Vec<[f32; 4]>, Option<(Vec<[f32; 4]>, Vec<f32>)>) {
    use super::material as m;
    let picked = super::light::vu1_prep_lights(&super::light::ee_sub_1a4478(&l.lights, at, None), at);
    let lin = |c: [f32; 4]| [super::light::ps2_linear(c[0]), super::light::ps2_linear(c[1]), super::light::ps2_linear(c[2]), c[3]];
    let n = d.grey_src.len();
    let mut grey = Vec::with_capacity(n);
    let mut ov = d.ov_src.as_ref().map(|_| (Vec::with_capacity(n), Vec::with_capacity(n)));
    for i in 0..n {
        let (lt, lw) = match (pos.get(i), nrm.get(i)) {
            (Some(&p), Some(&q)) => super::light::vu1_vertex_terms(&picked, p, q),
            _ => ([0.0; 3], 0.0),
        };
        let lw = lw + l.fill_w;
        let v = d.grey_src[i];
        let g = m::two_layer_vu_fill(lt, lw, l.fill).0;
        let mut c = if d.ov_src.is_some() {
            let w = m::two_layer_grey_weight(v);
            [g[0] * w, g[1] * w, g[2] * w, v[3]]
        } else {
            [v[0] * g[0], v[1] * g[1], v[2] * g[2], v[3]]
        };
        if let Some(a) = d.alpha.as_ref().and_then(|a| a.get(i)) {
            c[3] = *a;
        }
        grey.push(lin(c));
        if let (Some((oc, av)), Some(src)) = (ov.as_mut(), d.ov_src.as_ref()) {
            let k = m::two_layer_vu(lt, lw).1;
            oc.push(lin(m::two_layer_pass_color(src[i], Some(k))));
            av.push(m::two_layer_colour_av(lt, lw));
        }
    }
    (grey, ov)
}

/// ANIM R74: per-frame VU1 light of the `DynLit` skinned prims -> COLOR of the lit pass and its colour
/// pass child, or the merged two-layer mesh (UV_1 / COLOR / TANGENT, `merged_mesh_baked`) when swapped.
pub fn update_dyn_lit(
    q: Query<(&DynLit, &Mesh3d, &ViewVisibility, Option<&super::twolayer::TwoLayerSwapped>, Option<&Children>)>,
    colour: Query<&Mesh3d, With<ColourPass>>,
    xforms: Query<&GlobalTransform>,
    lights: Option<Res<DynLitLights>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let Some(lights) = lights else { return };
    // Bevy world -> level (the level root is rotated -90 deg about x)
    let lv = |v: Vec3| [v.x, -v.z, v.y];
    for (d, m, vis, swapped, kids) in &q {
        if !vis.get() {
            continue;
        }
        let Ok(root) = xforms.get(d.root) else { continue };
        let at = lv(root.translation());
        let s = &d.skin;
        let mats: Vec<Mat4> = s.joints.iter().zip(&s.ibp).map(|(j, ibp)| xforms.get(*j).map(|g| g.to_matrix() * *ibp).unwrap_or(Mat4::IDENTITY)).collect();
        let (mut pos, mut nrm) = (Vec::with_capacity(d.pos.len()), Vec::with_capacity(d.pos.len()));
        for (i, v) in d.pos.iter().enumerate() {
            let (ji, jw) = (s.ji.get(i).copied().unwrap_or_default(), s.jw.get(i).copied().unwrap_or([1.0, 0.0, 0.0, 0.0]));
            let n = d.nrm.get(i).copied().unwrap_or(Vec3::Z);
            let (mut p, mut q) = (Vec3::ZERO, Vec3::ZERO);
            for k in (0..4).filter(|&k| jw[k] > 0.0) {
                let mm = mats.get(ji[k] as usize).unwrap_or(&Mat4::IDENTITY);
                p += jw[k] * mm.transform_point3(*v);
                q += jw[k] * mm.transform_vector3(n);
            }
            pos.push(lv(p));
            nrm.push(lv(q.normalize_or(Vec3::Y)));
        }
        let (grey, ov) = dyn_lit_colors(d, &lights, &pos, &nrm, at);
        let tan = |av: &[f32]| av.iter().map(|&a| [1.0, 0.0, 0.0, a]).collect::<Vec<[f32; 4]>>();
        if swapped.is_some() {
            // merged two-layer mesh (`twolayer::merged_mesh_baked` layout)
            let Some((oc, av)) = &ov else { continue };
            if let Some(mut t) = meshes.get_mut(&m.0) {
                t.insert_attribute(Mesh::ATTRIBUTE_UV_1, grey.iter().map(|g| [g[0], g[1]]).collect::<Vec<[f32; 2]>>());
                t.insert_attribute(Mesh::ATTRIBUTE_COLOR, oc.iter().zip(&grey).map(|(c, g)| [c[0] * c[3], c[1] * c[3], c[2] * c[3], g[2]]).collect::<Vec<[f32; 4]>>());
                let a: Vec<f32> = if d.alpha.is_some() { grey.iter().map(|g| g[3]).collect() } else { av.clone() };
                t.insert_attribute(Mesh::ATTRIBUTE_TANGENT, tan(&a));
            }
            continue;
        }
        if let Some(mut t) = meshes.get_mut(&m.0) {
            t.insert_attribute(Mesh::ATTRIBUTE_COLOR, grey);
        }
        if let Some((oc, av)) = ov {
            for k in kids.into_iter().flat_map(|k| k.iter()) {
                let Ok(cm) = colour.get(k) else { continue };
                if let Some(mut t) = meshes.get_mut(&cm.0) {
                    t.insert_attribute(Mesh::ATTRIBUTE_COLOR, oc.clone());
                    t.insert_attribute(Mesh::ATTRIBUTE_TANGENT, tan(&av));
                }
            }
        }
    }
}

/// ANIM R74: the unlit (VU1-baked) twin of a lit level material, registered for the two-layer merge like
/// the static baked prims.
fn unlit_twin(
    mat: &Handle<StandardMaterial>,
    materials: &mut Assets<StandardMaterial>,
    unlit_of: &mut HashMap<AssetId<StandardMaterial>, Handle<StandardMaterial>>,
    tl_mats: Option<&mut Assets<super::twolayer::TwoLayerMaterial>>,
    tl_map: Option<&mut super::twolayer::TwoLayerMap>,
    tl_on: bool,
) -> Handle<StandardMaterial> {
    let lit_id = mat.id();
    if let Some(h) = unlit_of.get(&lit_id) {
        return h.clone();
    }
    let mut u = materials.get(mat).cloned().unwrap_or_default();
    u.unlit = true;
    let mo = tl_map.as_ref().and_then(|m| m.mats.get(&lit_id)).map(|(_, o)| *o);
    let merged = mo.and_then(|o| materials.get(o).cloned()).filter(|_| tl_on).and_then(|o| super::twolayer::merged_material(&u, &o));
    let zt = tl_map.as_ref().and_then(|m| m.zwrite.get(&lit_id)).cloned();
    let h = materials.add(u);
    if let Some(map) = tl_map {
        if let Some(t) = zt {
            map.zwrite.insert(h.id(), t);
        }
        if let (Some(mm), Some(o), Some(tm)) = (merged, mo, tl_mats) {
            map.mats.insert(h.id(), (tm.add(mm), o));
        }
    }
    unlit_of.insert(lit_id, h.clone());
    h
}

/// Skin (joints, inverse binds, weights) and bind positions of a skinned prim (LOOK casters, R72 ink).
#[allow(clippy::too_many_arguments)]
fn prim_skin(
    commands: &mut Commands,
    world: &World,
    bw: &mut super::attach::BoneWorlds,
    object: usize,
    p: &super::scene::ScenePrim,
    joint_of: &HashMap<(i32, i16), (Entity, Affine)>,
    obj_world: &Affine,
    e: Entity,
) -> Option<(super::shadowvol_pass::CasterSkin, Vec<Vec3>)> {
    let sk = crate::engine::anim::skin_mesh::bevy_skinned_mesh(&p.prim)?;
    let Payload::Loaded(LoadData::Alo { alo, .. }) = &world.objects[object].payload else { return None };
    use bevy::mesh::VertexAttributeValues as V;
    let refs = super::skin::record_weight_refs(&alo.mesh.records[p.record].data);
    let sj = super::skin::skin_joints(world, bw, object, refs);
    let (mut joints, mut ibp) = (Vec::new(), Vec::new());
    for (k, w) in sj.rest_world.iter().enumerate() {
        let (j, l) = refs.get(k).and_then(|r| joint_of.get(r).copied()).unwrap_or_else(|| {
            let l = super::skin::joint_local(w.as_ref(), obj_world, &p.local);
            (commands.spawn((affine_to_transform(&l), ChildOf(e))).id(), l)
        });
        joints.push(j);
        ibp.push(affine_to_transform(&super::skin::inverse_bindpose(&p.local, &l)).to_matrix());
    }
    let ji = match sk.attribute(Mesh::ATTRIBUTE_JOINT_INDEX) {
        Some(V::Uint16x4(v)) => v.clone(),
        _ => vec![],
    };
    let jw = match sk.attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT) {
        Some(V::Float32x4(v)) => v.clone(),
        _ => vec![],
    };
    let pos = match sk.attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(V::Float32x3(v)) => v.iter().map(|x| Vec3::from(*x)).collect(),
        _ => p.prim.positions.iter().map(|v| Vec3::from(*v)).collect(),
    };
    Some((super::shadowvol_pass::CasterSkin { joints, ibp, ji, jw }, pos))
}

/// WP-C5: a type-1 ink edge prim (prim-local positions + edge quads); its LineList mesh is
/// rebuilt every frame from the camera position (`update_ink_outlines`).
#[derive(Component, Clone)]
pub struct InkOutline(pub Arc<MeshPrim>);

/// WP-C5 graphics options (port addition). `ink_outlines` = settings.toml `[video] ink_outlines`.
#[derive(Resource, Clone, Debug)]
pub struct GfxSettings {
    pub ink_outlines: bool,
}
impl Default for GfxSettings {
    fn default() -> Self {
        GfxSettings { ink_outlines: true }
    }
}

fn line_mesh(seg: &[[f32; 3]]) -> Mesh {
    // never empty (2 coincident points draw nothing)
    let pos = if seg.is_empty() { vec![[0.0; 3]; 2] } else { seg.to_vec() };
    Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::default()).with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, pos)
}

/// WP-C5: per-frame silhouette selection (gfx::ink, the VU1 0x05a8..0x0718 rule against the
/// active camera's eye) -> LineList positions. Hidden when `GfxSettings::ink_outlines` is off.
pub fn update_ink_outlines(
    settings: Res<GfxSettings>,
    cams: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut inks: Query<(&InkOutline, &Mesh3d, &GlobalTransform, &mut Visibility, Option<&InkSkin>)>,
    xforms: Query<&GlobalTransform>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let cam = cams.iter().find(|(c, _)| c.is_active);
    let eye = cam.map(|(_, t)| t.translation());
    // ANIM R73: world -> clip of the active camera (the GS LINE width is in screen pixels)
    let clip_from_world = cam.map(|(c, t)| c.clip_from_view() * t.to_matrix().inverse());
    let lines = std::env::var("SLY2_R73INKLINE").is_ok();
    for (ink, m, gt, mut vis, skin) in inks.iter_mut() {
        let want = if settings.ink_outlines { Visibility::Inherited } else { Visibility::Hidden };
        if *vis != want {
            *vis = want;
        }
        let Some(eye) = eye.filter(|_| settings.ink_outlines) else { continue };
        let inv = gt.affine().inverse();
        let local = inv.transform_point3(eye);
        // ANIM R72: skinned ink edges: the posed vertices (world, as the casters) in the prim's frame
        let posed;
        let prim: &MeshPrim = match skin {
            Some(InkSkin { skin: s, pos }) => {
                let mats: Vec<Mat4> = s.joints.iter().zip(&s.ibp).map(|(j, ibp)| xforms.get(*j).map(|g| g.to_matrix() * *ibp).unwrap_or(Mat4::IDENTITY)).collect();
                let mut prim = MeshPrim { edges: ink.0.edges.clone(), ..Default::default() };
                prim.positions = pos
                    .iter()
                    .enumerate()
                    .map(|(i, v)| {
                        let (ji, jw) = (s.ji.get(i).copied().unwrap_or_default(), s.jw.get(i).copied().unwrap_or([1.0, 0.0, 0.0, 0.0]));
                        let wv = (0..4).filter(|&k| jw[k] > 0.0).fold(Vec3::ZERO, |acc, k| acc + jw[k] * mats.get(ji[k] as usize).unwrap_or(&Mat4::IDENTITY).transform_point3(*v));
                        inv.transform_point3(wv).to_array()
                    })
                    .collect();
                posed = prim;
                &posed
            }
            None => &ink.0,
        };
        let new = match clip_from_world.filter(|_| !lines) {
            Some(cfw) => {
                let mut edges = Vec::new();
                super::ink::ink_edges(prim, local.to_array(), &mut edges);
                ink_quad_mesh(&edges, cfw * gt.to_matrix())
            }
            None => {
                let mut seg = Vec::new();
                super::ink::ink_segments(prim, local.to_array(), &mut seg);
                line_mesh(&seg)
            }
        };
        // PERF2: only rewrite (re-upload) outlines whose silhouette changed; get_mut marks the asset
        // modified, and every outline was re-uploaded every frame (render allocate_and_free_meshes)
        let same = meshes.get(&m.0).is_some_and(|old| {
            old.primitive_topology() == new.primitive_topology()
                && old.attribute(Mesh::ATTRIBUTE_POSITION).map(|a| a.get_bytes()) == new.attribute(Mesh::ATTRIBUTE_POSITION).map(|a| a.get_bytes())
                && old.indices().map(|i| i.iter().collect::<Vec<_>>()) == new.indices().map(|i| i.iter().collect::<Vec<_>>())
        });
        if !same {
            if let Some(mut mesh) = meshes.get_mut(&m.0) {
                *mesh = new;
            }
        }
    }
}

/// ANIM R73: GS field pixels per NDC unit (the 512x224 field buffer spans the viewport: SCISSOR
/// 0..511 x 0..223, logs/LOOK.md), x and y.
pub const INK_GS_PX_PER_NDC: [f32; 2] = [256.0, 112.0];

/// ANIM R73: half extent (GS pixels, minor axis) of an AA1 GS LINE (2-pixel coverage footprint).
pub const INK_AA1_HALF_PX: f32 = 1.0;

/// ANIM R73: silhouette edges (a, b, c) (prim-local) -> TriangleList of screen quads covering the
/// GS AA1 LINE b'c' footprint (`ink::vu1_ink_line`, `INK_AA1_HALF_PX`), placed back in prim-local space at the
/// endpoints' clip depth. `clip_from_local` = camera clip * prim world. Edges with a vertex behind
/// the eye are skipped (stand-in for the VU clip-flag skip).
pub fn ink_quad_mesh(edges: &[[[f32; 3]; 3]], clip_from_local: Mat4) -> Mesh {
    let local_from_clip = clip_from_local.inverse();
    let [sx, sy] = INK_GS_PX_PER_NDC;
    let mut out: Vec<[f32; 3]> = Vec::with_capacity(edges.len() * 6);
    for e in edges {
        let clip = e.map(|p| clip_from_local * Vec3::from(p).extend(1.0));
        if clip.iter().any(|c| c.w <= 1e-6) {
            continue;
        }
        let px = clip.map(|c| [c.x / c.w * sx, c.y / c.w * sy]);
        let Some((b, c)) = super::ink::vu1_ink_line(px[0], px[1], px[2]) else { continue };
        // GS LINE footprint: one pixel per major-axis step; AA1 adds the coverage neighbour on the
        // minor axis -> +-1 GS pixel along the minor axis around b'c' (the half inside the body sits
        // behind the nearer surface and is depth-hidden, as on the GS)
        let (nx, ny) = if (c[0] - b[0]).abs() >= (c[1] - b[1]).abs() { (0.0, INK_AA1_HALF_PX) } else { (INK_AA1_HALF_PX, 0.0) };
        let back = |p: [f32; 2], cl: Vec4| -> [f32; 3] {
            let v = local_from_clip * Vec4::new(p[0] / sx * cl.w, p[1] / sy * cl.w, cl.z, cl.w);
            (v.truncate() / v.w).to_array()
        };
        let q = [
            back([b[0] + nx, b[1] + ny], clip[1]),
            back([b[0] - nx, b[1] - ny], clip[1]),
            back([c[0] - nx, c[1] - ny], clip[2]),
            back([c[0] + nx, c[1] + ny], clip[2]),
        ];
        // both windings (the ink material keeps the default back-face cull; mirrored prims flip it)
        out.extend_from_slice(&[q[0], q[1], q[2], q[0], q[2], q[3], q[0], q[2], q[1], q[0], q[3], q[2]]);
    }
    if out.is_empty() {
        out = vec![[0.0; 3]; 3];
    }
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default()).with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, out)
}

/// WP-C5: the one graphics plugin for the game app: live lights (GfxLightPlugin), skinning +
/// ASEG driver (anim::skin::AnimSkinPlugin), ink outlines (GfxSettings + update_ink_outlines).
pub struct GfxPlugin;
impl Plugin for GfxPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((GfxLightPlugin, crate::engine::anim::skin::AnimSkinPlugin, super::particles::ParticlePlugin, super::envvol::EnvVolumePlugin, super::twolayer::TwoLayerPlugin, super::screen::ScreenPlugin, super::display::Ps2DisplayPlugin, super::shadowvol_pass::ShadowVolumePassPlugin))
            .init_resource::<GfxSettings>()
            .add_systems(PostUpdate, update_ink_outlines.after(bevy::transform::TransformSystems::Propagate))
            // ANIM R75: after CheckVisibility - before it every ViewVisibility reads false (R74 never relit)
            .add_systems(PostUpdate, update_dyn_lit.after(bevy::transform::TransformSystems::Propagate).after(bevy::camera::visibility::VisibilitySystems::CheckVisibility))
            .add_systems(Update, (update_level_fog, sync_level_fog).chain())
            .add_systems(PreUpdate, super::fog::patch_fog_shaders);
        super::pick::add_pick_debug(app);
        super::pick::add_skin_check(app);
    }
}

/// WP-C7: the level's g_cm fog (`fog::level_fog` of `World::props_2ddf40`, set before any mesh
/// loads like FUN_0013f810(0x2ddf40) in the original stream).
#[derive(Resource, Clone, Copy, Debug, PartialEq, Default)]
pub struct LevelFogRes(pub super::fog::LevelFog);

pub fn update_level_fog(mut commands: Commands, world: Option<Res<World>>, fog: Option<Res<LevelFogRes>>) {
    let Some(w) = world else { return };
    if !w.is_changed() && fog.is_some() {
        return;
    }
    let f = LevelFogRes(super::fog::level_fog(&w.props_2ddf40));
    if fog.as_deref() != Some(&f) {
        info!("level fog: {:?} amount at end {:.3}", f.0, super::fog::fog_amount(1.0, &f.0.fog, f.0.fog.end));
        commands.insert_resource(f);
    }
}

/// WP-C7: GS fog (FUN_0014e270 coefficients baked per mesh record, FOGCOL = FUN_001ed278 colour)
/// -> Bevy DistanceFog on the CM camera; records with fog scale < 0.5 have `fog_enabled` off.
pub fn sync_level_fog(mut commands: Commands, fog: Option<Res<LevelFogRes>>, cams: Query<(Entity, Has<DistanceFog>), With<crate::engine::camera::bevy_sync::CmCamera>>) {
    let Some(fog) = fog else { return };
    if super::envvol::c7_before() || std::env::var_os("SLY2_LOOK_NOFOG").is_some() {
        return;
    }
    for (e, has) in &cams {
        if fog.is_changed() || !has {
            // DAT_003e12c8 grey filter (flashback) is not raised by any ported code -> false
            commands.entity(e).insert(super::fog::distance_fog(&fog.0.fog, 1.0, fog.0.rgba, false));
        }
    }
}

/// Registers the live light path (coordinator adds this plugin to the game app).
pub struct GfxLightPlugin;
impl Plugin for GfxLightPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LightPropQueue>().add_systems(Update, apply_light_props);
    }
}

/// WP-A10: rewrite COLOR of a folded two-layer prim (`material::two_layer_vertex_color`).
pub fn two_layer_colors(m: Option<&super::material::TwoLayerMeans>, mut mesh: Mesh) -> Mesh {
    let Some(m) = m else { return mesh };
    if let Some(bevy::mesh::VertexAttributeValues::Float32x4(c)) = mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR) {
        for v in c.iter_mut() {
            *v = super::material::two_layer_vertex_color(*v, m);
        }
    }
    mesh
}

/// WP-C9: the GS multiplies the texel by the vertex colour in display (gamma) space
/// (`texel * Cv / 0x80`); Bevy multiplies COLOR with the linear albedo, so the rgb factors are
/// raised to `light::PS2_GAMMA` (alpha unchanged: blending is linear in both).
pub fn linear_vertex_colors(mut mesh: Mesh) -> Mesh {
    if let Some(bevy::mesh::VertexAttributeValues::Float32x4(c)) = mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR) {
        for v in c.iter_mut() {
            for k in 0..3 {
                v[k] = super::light::ps2_linear(v[k]);
            }
        }
    }
    mesh
}

/// FEEL R22: COLOR alpha -> the FUN_0016a000 GS vertex alpha (`material::gs_alpha`).
pub fn gs_vertex_alpha(mut mesh: Mesh, sa: Option<u8>) -> Mesh {
    if let Some(bevy::mesh::VertexAttributeValues::Float32x4(c)) = mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR) {
        for v in c.iter_mut() {
            v[3] = super::material::gs_alpha(v[3], sa);
        }
    }
    mesh
}

/// LOOK: FUN_0016a000 (0x0016a000) builds the GS vertex colour array (+0x34) as
/// `(vcol * c + 0xff) >> 9` per rgb byte with c = the shader's colour (record +8, `Shader::colors[0]`),
/// so a 0xff vertex colour under a 0xff shader colour is 0x7f (= 1.0 at the GS 0x80 scale), not 2.0.
/// The port used the raw bytes / 128 (everything up to 2x: washed-out frames, playtest R1). Alpha:
/// (va * sa + 0xff) / 0x1fe (`gs_vertex_alpha`, FEEL R22, applied by the caller). `None` (no shader):
/// unchanged. COLOR in `mesh::ps2_color` scale.
pub fn gs_vertex_colors(mut mesh: Mesh, matcol: Option<[u8; 4]>) -> Mesh {
    let Some(m) = matcol else { return mesh };
    if let Some(bevy::mesh::VertexAttributeValues::Float32x4(c)) = mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR) {
        for v in c.iter_mut() {
            for k in 0..3 {
                let b = (v[k] * 128.0).round().clamp(0.0, 255.0) as u32;
                v[k] = ((b * m[k] as u32 + 0xff) >> 9) as f32 / 128.0;
            }
        }
    }
    mesh
}

/// LOOK R17: COLOR of an unlit prelit prim (`mesh::mesh0_prelit`): layer 0 = the first GS pass (VU qw+1:
/// two-layer grey pass / single-layer, rgb = c0, vertex alpha kept), layer 1 = the additive colour
/// pass (qw+2: Cs * As = c1 * b, alpha 1). Then display -> linear (raster and RT, unlit material).
pub fn prelit_colors(mut mesh: Mesh, pre: &[[[f32; 4]; 2]], layer: usize) -> Mesh {
    // LOOK R24: the prelit colour pass's GS Av (0x80 = 1.0) = its frame alpha source (`twolayer::set_colour_pass_av`)
    let n = mesh.count_vertices();
    if layer == 1 && pre.len() >= n && n > 0 {
        let av: Vec<f32> = pre[..n].iter().map(|q| q[1][3].clamp(0.0, 255.0 / 128.0)).collect();
        super::twolayer::set_colour_pass_av(&mut mesh, &av);
    }
    if let Some(bevy::mesh::VertexAttributeValues::Float32x4(c)) = mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR) {
        for (v, q) in c.iter_mut().zip(pre) {
            let e = q[layer];
            *v = if layer == 0 { [e[0], e[1], e[2], v[3]] } else { [e[0] * e[3], e[1] * e[3], e[2] * e[3], v[3].min(1.0)] };
        }
    }
    linear_vertex_colors(mesh)
}

/// LOOK verification switch (port addition): `SLY2_LOOK_LAYER=grey` drops the kinds-1/7 colour pass
/// (k = 0), `=colour` drops the baked grey pass, to see which layer a pixel comes from.
fn look_layer() -> Option<&'static str> {
    static L: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
    L.get_or_init(|| std::env::var("SLY2_LOOK_LAYER").ok()).as_deref()
}

/// LOOK: the 4 lights a static draw's VU1 program sees: FUN_001a4478 picks them (FUN_001a3f78) at the
/// light set position (`at`: the root object's world translation, FUN_001f2108 set +4 = root +0x60 + 0x70),
/// then VU1 0x2a0..0x3c8 turns each point light into a directional one at the draw matrix row 3
/// (`light::vu1_prep_lights`; header word 12 = 0 for unskinned draws, FUN_0016a000). Diag
/// `SLY2_LOOK_PICK=centre` picks at the prim centroid instead.
fn vu1_object_lights(lights: &[super::light::Vu1Light], centre: [f32; 3], at: [f32; 3], world: &Affine) -> Vec<super::light::Vu1Light> {
    let pick = if std::env::var("SLY2_LOOK_PICK").as_deref() == Ok("centre") { centre } else { at };
    super::light::vu1_prep_lights(&super::light::ee_sub_1a4478(lights, pick, None), world[3])
}

/// LOOK: COLOR of the lit grey pass of a static kinds-1/7 prim with the VU1 per-vertex light (raster):
/// rgb = n.w * min(L, 2) (`material::two_layer_vu_fill` .0 with the level shadow fill vf11, n.w = `two_layer_grey_weight`),
/// L = the object's 4 lights (FUN_001a3f78) with the linear distance fall-off (`light::vu1_vertex_terms`),
/// then display -> linear. Drawn unlit: replaces the Bevy point lights (inverse-square fall-off left the
/// cellar far darker than the PS2, ref_pcsx2 pcsx2-qt_3teg0JtiLF.png).
pub fn vu1_lit_grey(mut mesh: Mesh, lights: &[super::light::Vu1Light], fill: [f32; 3], fill_w: f32, world: &Affine, at: [f32; 3]) -> Mesh {
    use bevy::mesh::VertexAttributeValues as V;
    let xp = |v: [f32; 3]| [0, 1, 2].map(|j| v[0] * world[0][j] + v[1] * world[1][j] + v[2] * world[2][j] + world[3][j]);
    let xn = |v: [f32; 3]| {
        let r = [0, 1, 2].map(|j| v[0] * world[0][j] + v[1] * world[1][j] + v[2] * world[2][j]);
        let l = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt().max(1e-12);
        r.map(|x| x / l)
    };
    let pos: Vec<[f32; 3]> = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) { Some(V::Float32x3(p)) => p.iter().map(|&v| xp(v)).collect(), _ => Vec::new() };
    let nrm: Vec<[f32; 3]> = match mesh.attribute(Mesh::ATTRIBUTE_NORMAL) { Some(V::Float32x3(p)) => p.iter().map(|&v| xn(v)).collect(), _ => Vec::new() };
    let c = pos.iter().fold([0.0f32; 3], |a, p| [a[0] + p[0], a[1] + p[1], a[2] + p[2]]).map(|x| x / pos.len().max(1) as f32);
    let picked = vu1_object_lights(lights, c, at, world);
    if let Some(V::Float32x4(cols)) = mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR) {
        for (i, v) in cols.iter_mut().enumerate() {
            let (l, lw) = match (pos.get(i), nrm.get(i)) {
                (Some(&p), Some(&n)) => super::light::vu1_vertex_terms(&picked, p, n),
                _ => ([0.0; 3], 0.0),
            };
            // VU1 0x10e0: Lw += vf11.w (`light::level_shadow_fill_w`)
            let lw = lw + fill_w;
            let g = if look_layer() == Some("colour") { [0.0; 3] } else { super::material::two_layer_vu_fill(l, lw, fill).0 };
            let w = super::material::two_layer_grey_weight(*v);
            *v = [g[0] * w, g[1] * w, g[2] * w, v[3]];
        }
    }
    if let Some(V::Float32x4(cols)) = mesh.attribute(Mesh::ATTRIBUTE_COLOR) {
        look_vcstat("VC1", &pos, cols, look_uvs(&mesh));
    }
    linear_vertex_colors(mesh)
}

/// LOOK diag: `SLY2_LOOK_VCSTAT="x,y,z,r"` prints the computed vertex colours within r of (x,y,z).
fn look_uvs(mesh: &Mesh) -> Vec<[f32; 2]> {
    match mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
        Some(bevy::mesh::VertexAttributeValues::Float32x2(u)) => u.clone(),
        _ => Vec::new(),
    }
}

fn look_vcstat(tag: &str, pos: &[[f32; 3]], cols: &[[f32; 4]], uv: Vec<[f32; 2]>) {
    static F: std::sync::OnceLock<Option<[f32; 4]>> = std::sync::OnceLock::new();
    let Some(f) = *F.get_or_init(|| {
        let v: Vec<f32> = std::env::var("SLY2_LOOK_VCSTAT").ok()?.split(',').filter_map(|x| x.parse().ok()).collect();
        (v.len() == 4).then(|| [v[0], v[1], v[2], v[3]])
    }) else { return };
    for (i, (p, c)) in pos.iter().zip(cols).enumerate() {
        if (p[0] - f[0]).powi(2) + (p[1] - f[1]).powi(2) + (p[2] - f[2]).powi(2) < f[3] * f[3] {
            let t = uv.get(i).copied().unwrap_or([-1.0; 2]);
            eprintln!("{tag} {} {} {} {} {} {} {} {} {}", c[0], c[1], c[2], c[3], t[0], t[1], p[0], p[1], p[2]);
        }
    }
}

/// LOOK: COLOR of a static single-layer (kind 0) opaque prim lit like the kinds-1/7 grey pass but with
/// the full GS vertex colour: rgb = vcol * min(L, 2) (VU1 lit pass vf30, the object's 4 lights,
/// linear falloff, `light::vu1_vertex_terms`), then display -> linear. Drawn unlit (raster and RT).
pub fn vu1_lit_vcol(mut mesh: Mesh, lights: &[super::light::Vu1Light], fill: [f32; 3], fill_w: f32, world: &Affine, at: [f32; 3]) -> Mesh {
    use bevy::mesh::VertexAttributeValues as V;
    let xp = |v: [f32; 3]| [0, 1, 2].map(|j| v[0] * world[0][j] + v[1] * world[1][j] + v[2] * world[2][j] + world[3][j]);
    let xn = |v: [f32; 3]| {
        let r = [0, 1, 2].map(|j| v[0] * world[0][j] + v[1] * world[1][j] + v[2] * world[2][j]);
        let l = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt().max(1e-12);
        r.map(|x| x / l)
    };
    let pos: Vec<[f32; 3]> = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) { Some(V::Float32x3(p)) => p.iter().map(|&v| xp(v)).collect(), _ => Vec::new() };
    let nrm: Vec<[f32; 3]> = match mesh.attribute(Mesh::ATTRIBUTE_NORMAL) { Some(V::Float32x3(p)) => p.iter().map(|&v| xn(v)).collect(), _ => Vec::new() };
    let c = pos.iter().fold([0.0f32; 3], |a, p| [a[0] + p[0], a[1] + p[1], a[2] + p[2]]).map(|x| x / pos.len().max(1) as f32);
    let picked = vu1_object_lights(lights, c, at, world);
    if let Some(V::Float32x4(cols)) = mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR) {
        for (i, v) in cols.iter_mut().enumerate() {
            let (l, lw) = match (pos.get(i), nrm.get(i)) {
                (Some(&p), Some(&n)) => super::light::vu1_vertex_terms(&picked, p, n),
                _ => ([0.0; 3], 0.0),
            };
            // VU1 0x10e0: Lw += vf11.w (`light::level_shadow_fill_w`)
            let lw = lw + fill_w;
            let g = super::material::two_layer_vu_fill(l, lw, fill).0;
            *v = [v[0] * g[0], v[1] * g[1], v[2] * g[2], v[3]];
        }
    }
    linear_vertex_colors(mesh)
}

/// LOOK: COLOR of the additive colour pass of a static kinds-1/7 prim with the VU1 per-vertex light:
/// the object's lights (FUN_001a3f78 at the prim centre, 4 like FUN_001a4478), per vertex L / Lw
/// (`light::vu1_vertex_terms`) -> k = `material::two_layer_vu(L, Lw).1`, rgb = vcol * k (then linear).
/// Replaces the one level-wide k (directional lights only, 1.0 without one): the colour layer was
/// added unshaded at up to 2x everywhere (flat, oversaturated frames, playtest R1).
pub fn vu1_lit_overlay(mut mesh: Mesh, lights: &[super::light::Vu1Light], fill_w: f32, world: &Affine, at: [f32; 3]) -> Mesh {
    use bevy::mesh::VertexAttributeValues as V;
    let xp = |v: [f32; 3]| [0, 1, 2].map(|j| v[0] * world[0][j] + v[1] * world[1][j] + v[2] * world[2][j] + world[3][j]);
    let xn = |v: [f32; 3]| {
        let r = [0, 1, 2].map(|j| v[0] * world[0][j] + v[1] * world[1][j] + v[2] * world[2][j]);
        let l = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt().max(1e-12);
        r.map(|x| x / l)
    };
    let pos: Vec<[f32; 3]> = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) { Some(V::Float32x3(p)) => p.iter().map(|&v| xp(v)).collect(), _ => Vec::new() };
    let nrm: Vec<[f32; 3]> = match mesh.attribute(Mesh::ATTRIBUTE_NORMAL) { Some(V::Float32x3(p)) => p.iter().map(|&v| xn(v)).collect(), _ => Vec::new() };
    let c = pos.iter().fold([0.0f32; 3], |a, p| [a[0] + p[0], a[1] + p[1], a[2] + p[2]]).map(|x| x / pos.len().max(1) as f32);
    let picked = vu1_object_lights(lights, c, at, world);
    let mut av = Vec::with_capacity(pos.len());
    if let Some(V::Float32x4(cols)) = mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR) {
        for (i, v) in cols.iter_mut().enumerate() {
            let (l, lw) = match (pos.get(i), nrm.get(i)) {
                (Some(&p), Some(&n)) => super::light::vu1_vertex_terms(&picked, p, n),
                _ => ([0.0; 3], 0.0),
            };
            // VU1 0x10e0: Lw += vf11.w (`light::level_shadow_fill_w`)
            let lw = lw + fill_w;
            if std::env::var_os("SLY2_LOOK_LWSTAT").is_some() && i == 0 {
                eprintln!("LW at {at:?} w3 {:?} p {:?} l {l:?} lw {lw} picked {:?}", world[3], pos.get(i), picked.iter().map(|q| (q.kind, q.d.du, q.d.ru, q.d.k13c, q.d.color, q.d.vec)).collect::<Vec<_>>());
            }
            let k = if look_layer() == Some("grey") { 0.0 } else { super::material::two_layer_vu(l, lw).1 };
            *v = super::material::two_layer_pass_color(*v, Some(k));
            av.push(super::material::two_layer_colour_av(l, lw));
        }
    }
    if av.len() == pos.len() && !av.is_empty() {
        super::twolayer::set_colour_pass_av(&mut mesh, &av);
    }
    if let Some(V::Float32x4(cols)) = mesh.attribute(Mesh::ATTRIBUTE_COLOR) {
        look_vcstat("VC2", &pos, cols, look_uvs(&mesh));
    }
    linear_vertex_colors(mesh)
}

/// WIRE-C3: COLOR of a two-layer (kinds 1/7) prim. `None` = the lit grey pass: rgb = the vertex
/// colour brightness only (n.w, `material::two_layer_grey_weight`); `Some(k)` = the unlit additive
/// colour pass: rgb = vertex colour * k, alpha 1 (Av = 128 - 63.5 r = 0x80 at r = 0, `two_layer_vu`).
pub fn two_layer_split_colors(mut mesh: Mesh, k: Option<f32>) -> Mesh {
    if let Some(bevy::mesh::VertexAttributeValues::Float32x4(c)) = mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR) {
        for v in c.iter_mut() {
            *v = super::material::two_layer_pass_color(*v, k);
        }
    }
    mesh
}

/// The level material of a prim, plus the unlit additive colour-pass material of kinds 1/7
/// (WIRE-C3: drawn as its own prim, like FUN_001eb960's second pass; replaces the A9 fold).
fn make_material(scene: &LevelScene, entry: usize, shader: Option<u32>, cull_back: bool, vertex_alpha: bool, loader: Option<&LevelLoader>, images: &mut Assets<Image>) -> (StandardMaterial, Option<StandardMaterial>) {
    // VU1 back-face test (material::vu1_cull_back) -> Bevy cull mode
    let cull_mode = cull_back.then_some(bevy::render::render_resource::Face::Back);
    let mut m = StandardMaterial { perceptual_roughness: 0.9, reflectance: 0.2, double_sided: !cull_back, cull_mode, ..default() };
    let Some(tex) = scene.textures.get(entry) else { return (m, None) };
    let Some(si) = shader else { return (m, None) };
    let Some(sh) = tex.shaders.get(si as usize) else { return (m, None) };
    let desc_has_alpha;
    let frame = sh.frames.first();
    // mods/<mod>/textures/<lsn>/<TEX id>.png
    let over = frame.and_then(|f| loader?.mods.resolve(&texture_override_path(scene.lsn, f.id))).and_then(|p| std::fs::read(p).ok()).and_then(|b| {
        Image::from_buffer(&b, ImageType::Extension("png"), CompressedImageFormats::NONE, !super::light::display_space(), ImageSampler::Default, RenderAssetUsages::default()).ok()
    });
    let decoded = tex.decode_shader(si as usize, 0).and_then(|r| r.ok());
    let ly = super::material::layers(sh);
    // WIRE-C3: the additive unlit colour layer (CLUT set 0) is its own pass/prim (the light colour
    // never tints it; the A9 fold into the lit albedo turned EP1 blue under the Paris moonlight)
    let mut colour_pass = None;
    if let (None, Some(set)) = (&over, ly.emissive_set) {
        if let Some(Ok(c)) = tex.decode_shader_pass(si as usize, 0, set) {
            colour_pass = Some(c);
        }
    }
    let folded = colour_pass.is_some();
    desc_has_alpha = decoded.as_ref().is_some_and(|t| t.has_alpha()) || over.is_some() || vertex_alpha;
    let d = material_from(sh, desc_has_alpha);
    let img = match (over, decoded) {
        (Some(mut i), _) => {
            i.sampler = sampler(d.clamp_u, d.clamp_v);
            Some(i)
        }
        (None, Some(t)) => Some(to_bevy_image(&t, d.clamp_u, d.clamp_v)),
        _ => None,
    };
    if d.textured {
        m.base_color_texture = img.map(|i| images.add(i));
    }
    m.alpha_mode = alpha_mode(d.alpha);
    // LOOK (R2): FUN_001eb960 kinds 4/5/6 (sky): background pass, TEST 0x317e1 (ZTST ALWAYS, no Z write),
    // colour = baked vertex colour * shader colour only (no VU1 lighting) -> unlit, no fog
    if matches!(sh.kind, 4..=6) {
        m.unlit = true;
        m.fog_enabled = false;
    }
    m.double_sided = d.double_sided && !cull_back;
    // WP-C3: extra FUN_001eb960 passes -> StandardMaterial layers (no custom shaders)
    if let Some(set) = ly.emissive_set.filter(|_| !folded) {
        // additive unlit CLUT pass (ALPHA Cs*As+Cd) -> emissive texture (only when it could not be
        // folded into the albedo: texture override or a size mismatch)
        if let Some(Ok(t)) = tex.decode_shader_pass(si as usize, 0, set) {
            m.emissive_texture = Some(images.add(to_bevy_image(&t, d.clamp_u, d.clamp_v)));
            m.emissive = LinearRgba::WHITE;
        }
    }
    if let Some(t) = ly.env_tint {
        // kind 3 VU1 texgen environment pass -> specular reflection of the scene env map
        m.reflectance = (t[0].max(t[1]).max(t[2])).clamp(0.0, 1.0);
        m.perceptual_roughness = 0.2;
    }
    if ly.subtractive {
        m.alpha_mode = AlphaMode::Multiply;
    }
    if let Some((t, additive)) = super::material::env_glow(sh) {
        // WP-A5: kind-3 environment-only shader (guard vision cones, glow shells): the only
        // visible FUN_001eb960 pass is the tinted texgen env pass with no Z write. Previously this
        // fell through as an opaque untextured lit surface (large white shapes). The sphere-map
        // texture of the VU1 texgen pass is not ported: texel = 1.0, colour = the RGBAQ tint
        // (vertex colours are not used by that pass, see `strip_env_colors`).
        m.base_color_texture = None;
        m.emissive_texture = None;
        m.unlit = true;
        m.base_color = super::light::ps2_color(t[0], t[1], t[2], t[3]);
        m.alpha_mode = if additive { AlphaMode::Add } else { AlphaMode::Blend };
        colour_pass = None;
    }
    let overlay = colour_pass.map(|c| StandardMaterial {
        base_color_texture: Some(images.add(to_bevy_image(&c, d.clamp_u, d.clamp_v))),
        unlit: true,
        alpha_mode: AlphaMode::Add,
        double_sided: m.double_sided,
        cull_mode: m.cull_mode,
        // LOOK: PS2 order lit pass then colour pass (FUN_001eb960). Over a blended lit pass the separate
        // colour pass must sort after it (transparent phase sort key includes depth_bias), else the lit
        // pass covers it (black/flat cloth, e.g. the Paris safehouse table, under RT where no merge exists)
        depth_bias: if m.alpha_mode == AlphaMode::Blend { 1000.0 } else { 0.0 },
        ..default()
    });
    (m, overlay)
}

/// WP-A5: the kind-3 env pass colour is the constant RGBAQ tint, not the per-vertex colour.
fn strip_env_colors(scene: &LevelScene, entry: usize, shader: Option<u32>, mut mesh: Mesh) -> Mesh {
    let sh = shader.and_then(|s| scene.textures.get(entry)?.shaders.get(s as usize));
    if sh.is_some_and(|sh| super::material::env_glow(sh).is_some()) {
        mesh.remove_attribute(Mesh::ATTRIBUTE_COLOR);
    }
    mesh
}
