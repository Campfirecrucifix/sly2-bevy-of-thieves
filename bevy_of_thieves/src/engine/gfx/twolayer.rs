//! PERF2: the WIRE-C3 kinds-1/7 two-layer prims in ONE draw (raster GI).
//!
//! WIRE-C3 draws the unlit additive colour layer (CLUT set 0) as its own `ColourPass` child prim
//! (~4100 extra unbatched transparent draws on EP1 t2 = most of the 13.5 ms render-thread encode,
//! logs/PERF1.md). Here the lit grey pass and the colour pass are one `TwoLayerMaterial`
//! (`ExtendedMaterial<StandardMaterial, TwoLayerExt>`, twolayer.wesl): the lit pass is the normal
//! StandardMaterial path with the grey vertex colour (kept in UV_1 = (grey, alpha) of the merged
//! mesh), the colour pass = colour texture * COLOR (vertex colour * k), each through the same
//! post-lighting (fog / in-shader tonemap) as before, then summed premultiplied:
//! Opaque/Mask: base + ov; Blend: base*a + ov over (1-a) (Premultiplied); Add: base*a + ov, a 0.
//!
//! The spawned entities stay the WIRE-C3 ones (StandardMaterial lit prim + ColourPass child), so
//! npcdraw / playerdraw / Solari see the same hierarchy. `two_layer_swap` (raster only) swaps a
//! registered base (mesh, material) to (merged mesh, TwoLayerMaterial), stashing the originals in
//! `TwoLayerSwapped`, and hides the ColourPass child; with ray traced GI it reverts every swap (the
//! RT look / BLAS code works on StandardMaterial). NPC copies sharing the mesh/material handles are
//! swapped too; copies with their own mesh asset (ink paths) keep the two draws.

use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{ExtendedMaterial, MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline, MaterialPlugin};
use bevy::material::OpaqueRendererMethod;
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, BlendComponent, BlendFactor, BlendOperation, BlendState, RenderPipelineDescriptor, SpecializedMeshPipelineError};
use bevy::shader::ShaderRef;
use std::collections::{HashMap, HashSet};

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct TwoLayerExt {
    /// x: 0 opaque/mask, 1 blend, 2 add (the base pass alpha mode before the merge)
    #[uniform(100)]
    pub mode: Vec4,
    #[texture(101)]
    #[sampler(102)]
    pub colour: Option<Handle<Image>>,
}

impl MaterialExtension for TwoLayerExt {
    fn fragment_shader() -> ShaderRef {
        "embedded://sly2_recomp/engine/gfx/twolayer.wesl".into()
    }
}

pub type TwoLayerMaterial = ExtendedMaterial<StandardMaterial, TwoLayerExt>;

/// PERF2 (ray traced GI): the colour pass of a merged (opaque / additive base) prim as an OPAQUE-phase
/// forward draw with additive blending (addpass.wesl). Solari needs the lit base as a deferred
/// StandardMaterial, so the merge is raster only; this keeps the ~4100 colour passes out of the sorted
/// transparent phase (one unbatched draw each, ~20 ms of command encoding at 32:9). Additive blending is
/// order independent, and these overlays sit on opaque bases, so the opaque phase order is safe.
#[derive(Asset, AsBindGroup, Reflect, Debug, Clone, Default)]
pub struct AddPassExt {}

impl MaterialExtension for AddPassExt {
    fn fragment_shader() -> ShaderRef {
        "embedded://sly2_recomp/engine/gfx/addpass.wesl".into()
    }
    fn enable_prepass() -> bool {
        false
    }
    fn enable_shadows() -> bool {
        false
    }
    fn specialize(_: &MaterialExtensionPipeline, d: &mut RenderPipelineDescriptor, _: &MeshVertexBufferLayoutRef, _: MaterialExtensionKey<Self>) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(t) = d.fragment.as_mut().and_then(|f| f.targets.first_mut()).and_then(|t| t.as_mut()) {
            t.blend = Some(BlendState {
                color: BlendComponent { src_factor: BlendFactor::One, dst_factor: BlendFactor::One, operation: BlendOperation::Add },
                alpha: BlendComponent { src_factor: BlendFactor::Zero, dst_factor: BlendFactor::One, operation: BlendOperation::Add },
            });
        }
        if let Some(ds) = d.depth_stencil.as_mut() {
            ds.depth_write_enabled = false.into();
        }
        Ok(())
    }
}

pub type AddPassMaterial = ExtendedMaterial<StandardMaterial, AddPassExt>;

/// The AddPass twin of a WIRE-C3 colour-pass material (Add -> Opaque forward, blend set in the pipeline).
pub fn add_pass_material(ov: &StandardMaterial) -> AddPassMaterial {
    let mut base = ov.clone();
    base.alpha_mode = AlphaMode::Opaque;
    base.opaque_render_method = OpaqueRendererMethod::Forward;
    ExtendedMaterial { base, extension: AddPassExt {} }
}

/// A colour-pass entity drawn by `AddPassMaterial` while GI is ray traced (original material).
#[derive(Component, Clone)]
pub struct AddPassSwapped(pub MeshMaterial3d<StandardMaterial>);

/// Merged material for the base lit material `m` and the WIRE-C3 colour pass `ov`; None when the
/// base alpha mode cannot be merged (Multiply) or the colour pass has no texture.
pub fn merged_material(m: &StandardMaterial, ov: &StandardMaterial) -> Option<TwoLayerMaterial> {
    let mut base = m.clone();
    let mode = match m.alpha_mode {
        AlphaMode::Opaque | AlphaMode::Mask(_) => 0.0,
        AlphaMode::Blend => 1.0,
        AlphaMode::Add => 2.0,
        _ => return None,
    };
    // LOOK (R2): blend bases merge too: as two draws the ColourPass child (sorted at the prim origin)
    // often drew before its dark lit pass and was covered (black banners / frames). The merged order
    // is the PS2 one (FUN_001eb960: lit pass then colour pass per prim). `SLY2_PERF2_BLEND=0` = old split.
    if mode == 1.0 && std::env::var("SLY2_PERF2_BLEND").is_ok_and(|v| v == "0") {
        return None;
    }
    if mode > 0.0 {
        base.alpha_mode = AlphaMode::Premultiplied;
    }
    // LOOK: y = 1: the grey pass is the baked VU1 light (unlit base, `merged_mesh_baked`)
    let baked = if m.unlit { 1.0 } else { 0.0 };
    Some(ExtendedMaterial { base, extension: TwoLayerExt { mode: Vec4::new(mode, baked, look_tlpass(), 0.0), colour: Some(ov.base_color_texture.clone()?) } })
}

/// Merged mesh: the lit prim's mesh (`base`, grey COLOR) with COLOR = the colour-pass COLOR and
/// UV_1 = (grey, alpha) of the lit pass.
pub fn merged_mesh(base: &Mesh, overlay: &Mesh) -> Option<Mesh> {
    use bevy::mesh::VertexAttributeValues::Float32x4;
    let (Some(Float32x4(g)), Some(Float32x4(c))) = (base.attribute(Mesh::ATTRIBUTE_COLOR), overlay.attribute(Mesh::ATTRIBUTE_COLOR)) else { return None };
    if g.len() != c.len() {
        return None;
    }
    let uv1: Vec<[f32; 2]> = g.iter().map(|v| [v[0], v[3]]).collect();
    let av = colour_pass_av(overlay, c.len());
    let c = c.clone();
    let mut m = base.clone();
    m.insert_attribute(Mesh::ATTRIBUTE_UV_1, uv1);
    m.insert_attribute(Mesh::ATTRIBUTE_COLOR, c);
    set_colour_pass_av(&mut m, &av);
    Some(m)
}

/// LOOK: merged mesh of a raster-baked prim (gfx::game::vu1_lit_grey, opaque base): the grey pass is a
/// per-channel unlit colour -> UV_1 = (grey r, grey g), COLOR = (colour-pass rgb * its alpha, grey b)
/// (the colour pass is added as rgb * a, so pre-multiplying keeps the sum; the opaque base needs no alpha).
/// LOOK R26: `blend` (Blend base): the tangent w carries the grey pass's GS vertex alpha (the blend As)
/// instead of the colour-pass Av (only the opaque mode writes the frame alpha from Av).
pub fn merged_mesh_baked(base: &Mesh, overlay: &Mesh, blend: bool) -> Option<Mesh> {
    use bevy::mesh::VertexAttributeValues::Float32x4;
    let (Some(Float32x4(g)), Some(Float32x4(c))) = (base.attribute(Mesh::ATTRIBUTE_COLOR), overlay.attribute(Mesh::ATTRIBUTE_COLOR)) else { return None };
    if g.len() != c.len() {
        return None;
    }
    let uv1: Vec<[f32; 2]> = g.iter().map(|v| [v[0], v[1]]).collect();
    let av = if blend { g.iter().map(|v| v[3]).collect() } else { colour_pass_av(overlay, c.len()) };
    let c: Vec<[f32; 4]> = c.iter().zip(g).map(|(c, g)| [c[0] * c[3], c[1] * c[3], c[2] * c[3], g[2]]).collect();
    let mut m = base.clone();
    m.insert_attribute(Mesh::ATTRIBUTE_UV_1, uv1);
    m.insert_attribute(Mesh::ATTRIBUTE_COLOR, c);
    set_colour_pass_av(&mut m, &av);
    Some(m)
}

/// LOOK R24: the PS2 colour-pass GS vertex alpha Av (0x80 = 1.0) per vertex rides in the tangent w
/// (unlit two-layer prims have no normal maps; xyz = +x keeps Bevy's normalize finite). The merged
/// fragment (twolayer.wesl) writes the colour pass's frame alpha As = Ta * Av >> 7 from it, which the
/// shadow-volume resolve reads (`shadowvol::clut_index`).
pub fn set_colour_pass_av(m: &mut Mesh, av: &[f32]) {
    m.insert_attribute(Mesh::ATTRIBUTE_TANGENT, av.iter().map(|&a| [1.0, 0.0, 0.0, a]).collect::<Vec<[f32; 4]>>());
}

/// Av of a colour-pass mesh (`set_colour_pass_av`, written by `game::vu1_lit_overlay`), 1.0 if none.
pub fn colour_pass_av(m: &Mesh, n: usize) -> Vec<f32> {
    match m.attribute(Mesh::ATTRIBUTE_TANGENT) {
        Some(bevy::mesh::VertexAttributeValues::Float32x4(t)) if t.len() == n => t.iter().map(|v| v[3].abs()).collect(),
        _ => vec![1.0; n],
    }
}

/// Registered merges (cleared per level spawn).
#[derive(Resource, Default)]
pub struct TwoLayerMap {
    /// base lit material -> (merged material, colour-pass material)
    pub mats: HashMap<AssetId<StandardMaterial>, (Handle<TwoLayerMaterial>, AssetId<StandardMaterial>)>,
    /// base lit mesh -> merged mesh
    pub meshes: HashMap<AssetId<Mesh>, Handle<Mesh>>,
    /// colour-pass materials of merged prims (their AddPass twins are made lazily)
    pub overlays: HashSet<AssetId<StandardMaterial>>,
    pub add: HashMap<AssetId<StandardMaterial>, Handle<AddPassMaterial>>,
    /// FEEL R21: Blend base material -> its Z-writing Mask(AREF) twin (`z_twin_material`)
    pub zwrite: HashMap<AssetId<StandardMaterial>, Handle<StandardMaterial>>,
}

/// FEEL R21: FUN_001eb960 base TEST_1 of kinds 0/1 (e.g. 0x717eb: ATE, ATST GEQUAL, AREF 0x7e, AFAIL
/// FB_ONLY, ZTE) blends every texel but writes Z only for As >= AREF. Bevy's Blend never writes Z, so
/// those prims (Sly's hat/hands/cane, guards, windows) were overdrawn by later transparent draws
/// (the Paris moonlight shafts: "ghost Sly"). The passing part is drawn a second time as
/// Mask(AREF) in the opaque phase (Z + colour); the Blend draw still covers the whole prim (texels
/// >= AREF blend with As ~1 = the same colour, equal depth passes the transparent depth test).
pub fn z_twin_material(m: &StandardMaterial, aref: f32) -> StandardMaterial {
    let mut t = m.clone();
    t.alpha_mode = AlphaMode::Mask(aref);
    t
}

/// Marks an entity whose Z twin was spawned (or that is a twin).
#[derive(Component)]
pub struct ZTwin;

/// FEEL R21: spawn the Mask twin as a child of every prim drawn with a registered Blend material
/// (level, NPC and player copies alike; skinned prims share the joints).
#[allow(clippy::type_complexity)]
pub fn z_twin_spawn(
    mut commands: Commands,
    map: Res<TwoLayerMap>,
    q: Query<(Entity, &Mesh3d, &MeshMaterial3d<StandardMaterial>, Option<&bevy::mesh::skinning::SkinnedMesh>), (Added<MeshMaterial3d<StandardMaterial>>, Without<ZTwin>)>,
) {
    if map.zwrite.is_empty() || std::env::var_os("SLY2_FEEL_NOZTWIN").is_some() {
        return;
    }
    for (e, mesh, mat, skin) in &q {
        let Some(t) = map.zwrite.get(&mat.0.id()) else { continue };
        let mut c = commands.spawn((ZTwin, Mesh3d(mesh.0.clone()), MeshMaterial3d(t.clone()), Transform::IDENTITY, bevy::light::NotShadowCaster, ChildOf(e)));
        if let Some(s) = skin {
            c.insert(s.clone());
        }
        commands.entity(e).insert(ZTwin);
    }
}

/// Originals of a swapped base prim (restored when GI goes ray traced).
#[derive(Component, Clone)]
pub struct TwoLayerSwapped {
    pub mesh: Mesh3d,
    pub mat: MeshMaterial3d<StandardMaterial>,
}

/// A colour-pass child hidden because its parent draws merged (previous visibility).
#[derive(Component)]
pub struct TwoLayerHidden(pub Visibility);

/// Verification switch `SLY2_PERF2_ADD_OFF=1`: ray traced GI keeps the AlphaMode::Add colour passes.
pub fn add_off() -> bool {
    std::env::var("SLY2_PERF2_ADD_OFF").is_ok_and(|v| v == "1")
}

/// Verification switch `SLY2_PERF2_OFF=1`: keep the WIRE-C3 two draws (before/after shots).
pub fn disabled() -> bool {
    std::env::var("SLY2_PERF2_OFF").is_ok_and(|v| v == "1")
}

#[allow(clippy::type_complexity)]
pub fn two_layer_swap(
    mut commands: Commands,
    mut map: ResMut<TwoLayerMap>,
    std_mats: Res<Assets<StandardMaterial>>,
    mut add_mats: ResMut<Assets<AddPassMaterial>>,
    add_swapped: Query<(Entity, &AddPassSwapped)>,
    rt: Option<Res<super::rt::RtState>>,
    fresh: Query<(Entity, &Mesh3d, &MeshMaterial3d<StandardMaterial>, Option<&Children>), (Added<MeshMaterial3d<StandardMaterial>>, Without<TwoLayerSwapped>)>,
    all: Query<(Entity, &Mesh3d, &MeshMaterial3d<StandardMaterial>, Option<&Children>), Without<TwoLayerSwapped>>,
    swapped: Query<(Entity, &TwoLayerSwapped, Option<&Children>)>,
    kids: Query<(&MeshMaterial3d<StandardMaterial>, &Visibility), Without<TwoLayerHidden>>,
    hidden: Query<&TwoLayerHidden>,
    mut was_rt: Local<bool>,
    mut rescan: Local<bool>,
    fresh_mats: Query<(Entity, &MeshMaterial3d<StandardMaterial>), (Added<MeshMaterial3d<StandardMaterial>>, Without<AddPassSwapped>)>,
    all_mats: Query<(Entity, &MeshMaterial3d<StandardMaterial>), Without<AddPassSwapped>>,
) {
    let rt_on = rt.is_some_and(|r| r.active || r.requested);
    if rt_on {
        let first = !*was_rt;
        if first {
            // revert: originals back (Added -> the RT look picks them up), colour passes shown
            for (e, sw, ch) in &swapped {
                commands.entity(e).remove::<(TwoLayerSwapped, MeshMaterial3d<TwoLayerMaterial>)>().insert((sw.mesh.clone(), sw.mat.clone()));
                for c in ch.into_iter().flatten() {
                    if let Ok(h) = hidden.get(*c) {
                        commands.entity(*c).remove::<TwoLayerHidden>().insert(h.0);
                    }
                }
            }
        }
        // colour passes -> AddPass (opaque phase); `kids` sees every entity with a StandardMaterial
        if !add_off() {
            let map = &mut *map;
            let todo: Vec<(Entity, &MeshMaterial3d<StandardMaterial>)> = if first { all_mats.iter().collect() } else { fresh_mats.iter().collect() };
            for (e, mat) in todo {
                if !map.overlays.contains(&mat.0.id()) {
                    continue;
                }
                // LOOK: colour passes over a blended lit pass stay in the transparent phase (sorted after it)
                if std_mats.get(&mat.0).is_some_and(|m| m.depth_bias > 0.0) {
                    continue;
                }
                let h = match map.add.get(&mat.0.id()) {
                    Some(h) => h.clone(),
                    None => {
                        let Some(m) = std_mats.get(&mat.0) else { continue };
                        let h = add_mats.add(add_pass_material(m));
                        map.add.insert(mat.0.id(), h.clone());
                        h
                    }
                };
                commands.entity(e).insert((AddPassSwapped(mat.clone()), MeshMaterial3d(h))).remove::<MeshMaterial3d<StandardMaterial>>();
            }
        }
        *was_rt = true;
        return;
    }
    if *was_rt {
        for (e, a) in &add_swapped {
            commands.entity(e).remove::<(AddPassSwapped, MeshMaterial3d<AddPassMaterial>)>().insert(a.0.clone());
        }
        // the colour passes get their StandardMaterial back at the end of this frame: rescan next frame
        *was_rt = false;
        *rescan = true;
        return;
    }
    let back = std::mem::replace(&mut *rescan, false);
    if map.mats.is_empty() {
        return;
    }
    // after RT -> raster every original is back (no Added): rescan all
    let todo: Vec<_> = if back { all.iter().collect() } else { fresh.iter().collect() };
    for (e, mesh, mat, ch) in todo {
        let (Some((tl, ov)), Some(mm)) = (map.mats.get(&mat.0.id()), map.meshes.get(&mesh.0.id())) else { continue };
        // only prims that draw their WIRE-C3 colour-pass child (npcdraw's NPC prim copies have none:
        // merging those would add a colour layer the two-draw path never showed)
        let Some((c, v)) = ch.into_iter().flatten().find_map(|c| kids.get(*c).ok().filter(|(m, _)| m.0.id() == *ov).map(|(_, v)| (*c, *v))) else { continue };
        commands.entity(c).insert((TwoLayerHidden(v), Visibility::Hidden));
        commands
            .entity(e)
            .insert((TwoLayerSwapped { mesh: mesh.clone(), mat: mat.clone() }, Mesh3d(mm.clone()), MeshMaterial3d(tl.clone())))
            .remove::<MeshMaterial3d<StandardMaterial>>();
    }
}

pub struct TwoLayerPlugin;
impl Plugin for TwoLayerPlugin {
    fn build(&self, app: &mut App) {
        bevy::asset::embedded_asset!(app, "twolayer.wesl");
        bevy::asset::embedded_asset!(app, "addpass.wesl");
        app.add_plugins((MaterialPlugin::<TwoLayerMaterial>::default(), MaterialPlugin::<AddPassMaterial>::default())).init_resource::<TwoLayerMap>().add_systems(PostUpdate, (z_twin_spawn, two_layer_swap).chain());
    }
}

/// LOOK diag: `SLY2_LOOK_TLPASS=1` draws only the two-layer base (grey) pass, `=2` only the colour pass.
fn look_tlpass() -> f32 {
    std::env::var("SLY2_LOOK_TLPASS").ok().and_then(|v| v.parse().ok()).unwrap_or(0.0)
}
