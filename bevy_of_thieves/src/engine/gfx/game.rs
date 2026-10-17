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
        TextureFormat::Rgba8UnormSrgb,
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
    }
    let scene = LevelScene::build_from(data.lsn, &data.data, &world);
    let mut mat_cache: HashMap<(usize, Option<u32>, bool, bool, bool), (Handle<StandardMaterial>, Option<super::material::TwoLayerMeans>)> = HashMap::new();
    let root = commands
        .spawn((LevelGeometryRoot(data.lsn), Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)), Visibility::default()))
        .id();
    let mut ents = Vec::with_capacity(scene.objects.len());
    let mut bw = super::attach::BoneWorlds::default();
    let mut ink_mat: Option<Handle<StandardMaterial>> = None;
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
        for p in &so.prims {
            // WP-C4: only VU1 surface programs draw faces; shadow volumes (types 2/3) stay in
            // LevelScene (Bevy light shadows / Solari stand in for the green-channel shadow
            // pass, see C5 Findings). WP-C5: type-1 ink edges -> per-frame silhouette lines.
            if p.prim.role == super::mesh::MeshRole::InkEdges {
                let mat = ink_mat
                    .get_or_insert_with(|| materials.add(StandardMaterial { base_color: Color::BLACK, unlit: true, /* wgpu: depth bias is invalid for LineList */ ..default() }))
                    .clone();
                let m = meshes.add(line_mesh(&[]));
                commands.spawn((InkOutline(Arc::new(p.prim.clone())), Mesh3d(m), MeshMaterial3d(mat), affine_to_transform(&p.local), ChildOf(e)));
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
            let valpha = super::material::vertex_alpha(&p.prim.colors);
            let key = (so.entry, p.prim.shader, p.prim.cull_back, fog, valpha);
            let (mat, two_layer) = mat_cache
                .entry(key)
                .or_insert_with(|| {
                    let (mut m, tl) = make_material(&scene, so.entry, p.prim.shader, p.prim.cull_back, valpha, loader.as_deref(), &mut images);
                    m.fog_enabled = fog;
                    // WP-C7: env-only shaders -> light volume (gfx::envvol), level material drawn invisible
                    let sh = p.prim.shader.and_then(|s| scene.textures.get(so.entry)?.shaders.get(s as usize));
                    if let (true, Some(env_mats), Some(env_vols)) = (sh.is_some_and(|sh| super::material::env_glow(sh).is_some()) && !super::envvol::c7_before(), env_mats.as_mut(), env_vols.as_mut()) {
                        let v = env_mats.add(super::envvol::volume_material(&m));
                        super::envvol::hide_material(&mut m);
                        let h = materials.add(m);
                        env_vols.0.insert(h.id(), v);
                        return (h, tl);
                    }
                    (materials.add(m), tl)
                })
                .clone();
            // WP-A10: folded two-layer prims: the vertex colour hue only tints the colour layer
            // WP-C9: then the display-space PS2 colour factors -> linear (`linear_vertex_colors`)
            let fix_colors = |mesh: Mesh| linear_vertex_colors(two_layer_colors(two_layer.as_ref(), mesh));
            // WP-C5: prims with weights -> skinned mesh, joints = entities for the record's
            // weight_refs bones (children of the object entity), driven by anim::skin::AnimSkin.
            if let Some(skinned) = crate::engine::anim::skin_mesh::bevy_skinned_mesh(&p.prim) {
                let Payload::Loaded(LoadData::Alo { alo, .. }) = &world.objects[so.object].payload else { unreachable!() };
                let refs = super::skin::record_weight_refs(&alo.mesh.records[p.record].data);
                let sj = super::skin::skin_joints(&world, &mut bw, so.object, refs);
                let locals: Vec<Affine> = sj.rest_world.iter().map(|w| super::skin::joint_local(w.as_ref(), &obj_world, &p.local)).collect();
                let joints: Vec<Entity> = locals.iter().map(|l| commands.spawn((affine_to_transform(l), ChildOf(e))).id()).collect();
                let ibp: Vec<Mat4> = locals.iter().map(|l| affine_to_transform(&super::skin::inverse_bindpose(&p.local, l)).to_matrix()).collect();
                let bind = locals.iter().map(crate::engine::anim::pose::from_4x3).collect::<Vec<_>>();
                let skin = crate::engine::anim::skin::AnimSkin { palette: vec![crate::engine::anim::pose::IDENTITY; bind.len()], bind };
                // WP-A10: Bevy culls a skinned mesh by the Aabb of its raw positions in the entity frame,
                // which is the prim-local rest frame (p.local is only in the joints / inverse bindposes) and
                // ignores the pose -> posed models were culled up to ~200 units off (pop-in at the screen
                // edges, worst at 32:9). Per-joint bounds + DynamicSkinnedMeshBounds refresh it every frame.
                let mut skinned = fix_colors(strip_env_colors(&scene, so.entry, p.prim.shader, skinned));
                let _ = skinned.generate_skinned_mesh_bounds();
                commands.spawn((
                    Mesh3d(meshes.add(skinned)),
                    MeshMaterial3d(mat),
                    SkinnedMesh { inverse_bindposes: inverse_bindposes.add(SkinnedMeshInverseBindposes::from(ibp)), joints },
                    bevy::camera::visibility::DynamicSkinnedMeshBounds,
                    skin,
                    Transform::IDENTITY,
                    ChildOf(e),
                ));
                continue;
            }
            let mut mesh = fix_colors(strip_env_colors(&scene, so.entry, p.prim.shader, to_bevy_mesh(&p.prim)));
            let mut tf = affine_to_transform(&p.local);
            // WP-C8: translucent prims sort by their record centre (gfx::screen::sort_centre)
            if materials.get(&mat).is_some_and(|m| matches!(m.alpha_mode, AlphaMode::Blend | AlphaMode::Add | AlphaMode::Multiply)) {
                if let Some(bevy::mesh::VertexAttributeValues::Float32x3(v)) = mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION) {
                    let c = super::screen::sort_centre(v);
                    v.iter_mut().for_each(|p| (0..3).for_each(|k| p[k] -= c[k]));
                    tf = tf.mul_transform(Transform::from_translation(Vec3::from_array(c)));
                }
            }
            let m = meshes.add(mesh);
            let tag = super::pick::PrimTag { object, oid: world.objects[object].oid, entry: so.entry, shader: p.prim.shader, record: p.record };
            commands.spawn((Mesh3d(m), MeshMaterial3d(mat), tf, ChildOf(e), tag));
        }
    }
    for (i, so) in scene.objects.iter().enumerate() {
        let parent = so.owner.map(|o| ents[o]).unwrap_or(root);
        commands.entity(ents[i]).insert(ChildOf(parent));
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
    e.remove::<(DirectionalLight, PointLight)>();
    match l {
        Some(BevyLight::Directional(d, t)) => {
            e.insert((d, t));
        }
        Some(BevyLight::Point(p, t)) => {
            e.insert((p, t));
        }
        None => {}
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
    commands.insert_resource(ambient_resource(super::light::level_ambient(&set)));
    for l in lights {
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
    mut inks: Query<(&InkOutline, &Mesh3d, &GlobalTransform, &mut Visibility)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let eye = cams.iter().find(|(c, _)| c.is_active).map(|(_, t)| t.translation());
    for (ink, m, gt, mut vis) in inks.iter_mut() {
        let want = if settings.ink_outlines { Visibility::Inherited } else { Visibility::Hidden };
        if *vis != want {
            *vis = want;
        }
        let Some(eye) = eye.filter(|_| settings.ink_outlines) else { continue };
        let local = gt.affine().inverse().transform_point3(eye);
        let mut seg = Vec::new();
        super::ink::ink_segments(&ink.0, local.to_array(), &mut seg);
        if let Some(mut mesh) = meshes.get_mut(&m.0) {
            *mesh = line_mesh(&seg);
        }
    }
}

/// WP-C5: the one graphics plugin for the game app: live lights (GfxLightPlugin), skinning +
/// ASEG driver (anim::skin::AnimSkinPlugin), ink outlines (GfxSettings + update_ink_outlines).
pub struct GfxPlugin;
impl Plugin for GfxPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((GfxLightPlugin, crate::engine::anim::skin::AnimSkinPlugin, super::particles::ParticlePlugin, super::envvol::EnvVolumePlugin, super::screen::ScreenPlugin))
            .init_resource::<GfxSettings>()
            .add_systems(PostUpdate, update_ink_outlines.after(bevy::transform::TransformSystems::Propagate))
            .add_systems(Update, (update_level_fog, sync_level_fog).chain());
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
        commands.insert_resource(f);
    }
}

/// WP-C7: GS fog (FUN_0014e270 coefficients baked per mesh record, FOGCOL = FUN_001ed278 colour)
/// -> Bevy DistanceFog on the CM camera; records with fog scale < 0.5 have `fog_enabled` off.
pub fn sync_level_fog(mut commands: Commands, fog: Option<Res<LevelFogRes>>, cams: Query<(Entity, Has<DistanceFog>), With<crate::engine::camera::bevy_sync::CmCamera>>) {
    let Some(fog) = fog else { return };
    if super::envvol::c7_before() {
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

/// The level material of a prim, plus the layer means when the kinds-1/7 colour layer was folded
/// into the albedo (WP-A10, `two_layer_colors`).
fn make_material(scene: &LevelScene, entry: usize, shader: Option<u32>, cull_back: bool, vertex_alpha: bool, loader: Option<&LevelLoader>, images: &mut Assets<Image>) -> (StandardMaterial, Option<super::material::TwoLayerMeans>) {
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
        Image::from_buffer(&b, ImageType::Extension("png"), CompressedImageFormats::NONE, true, ImageSampler::Default, RenderAssetUsages::default()).ok()
    });
    let decoded = tex.decode_shader(si as usize, 0).and_then(|r| r.ok());
    let ly = super::material::layers(sh);
    // WP-A9: the additive colour layer is folded into the albedo (material::two_layer_albedo)
    let mut folded = false;
    let mut means = None;
    let decoded = decoded.map(|mut t| {
        if let (None, Some(set)) = (&over, ly.emissive_set) {
            if let Some(Ok(c)) = tex.decode_shader_pass(si as usize, 0, set) {
                if (c.width, c.height) == (t.width, t.height) {
                    means = Some(super::material::two_layer_means(&t.rgba, &c.rgba));
                    super::material::two_layer_albedo(&mut t.rgba, &c.rgba);
                    folded = true;
                }
            }
        }
        t
    });
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
        m.base_color = Color::srgba(t[0], t[1], t[2], t[3]);
        m.alpha_mode = if additive { AlphaMode::Add } else { AlphaMode::Blend };
    }
    (m, means)
}

/// WP-A5: the kind-3 env pass colour is the constant RGBAQ tint, not the per-vertex colour.
fn strip_env_colors(scene: &LevelScene, entry: usize, shader: Option<u32>, mut mesh: Mesh) -> Mesh {
    let sh = shader.and_then(|s| scene.textures.get(entry)?.shaders.get(s as usize));
    if sh.is_some_and(|sh| super::material::env_glow(sh).is_some()) {
        mesh.remove_attribute(Mesh::ATTRIBUTE_COLOR);
    }
    mesh
}
