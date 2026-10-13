//! WP-C7: kind-3 env-only shaders (FUN_001eb960 case 3 without flags & 2: guard vision /
//! flashlight cones, glow boxes, class-48 neon volumes) drawn as LIGHT VOLUMES like the PS2.
//!
//! Original (A9 / C3 Findings): passes 1/2 draw the mesh with TEST 0x71001 (ZTST GREATER = the
//! normal depth test, AFAIL FB_ONLY = no Z write) and ALPHA 0x6a (= Cd), writing only destination
//! alpha 0x00 (one VU1 packet range, the front faces) and 0x80 (the back faces); every other
//! surface is drawn with FBA (alpha MSB = 1). The visible env pass (VU1 texgen 0x6c, TEST
//! 0x75001 = DATE / DATM 0) then adds the tint `(head+0xc + 1) >> 1` only where the destination
//! alpha MSB is 0: the front face is in front of the scene but the back face is hidden, i.e. on
//! the scene surfaces INSIDE the volume (the lit floor patch of a flashlight cone, not a shell).
//!
//! REPLACED (Bevy has no destination alpha / stencil test in the main pass): one pass over the
//! BACK faces (cull front) with the depth test inverted (`CompareFunction::Less` in Bevy's
//! reversed Z = the back face lies BEHIND the scene surface), unlit, tint colour, AlphaMode::Add
//! (flags & 1) / Blend, no depth write. Every scene surface in front of the volume's back face and
//! inside its screen footprint is tinted; this equals the PS2 mask except for surfaces between the
//! camera and the volume's front face (tinted here, not on the PS2) and a camera inside the volume
//! (tinted here; the PS2 marks nothing because the front faces are behind the camera). Replaces the
//! WP-A5 visible additive shell (the orange / green wedges on t3) and the WP-A9 point-light stand-in
//! of class-48 volumes. The VU1 sphere-map texgen texel is not ported (texel = 1.0).
//!
//! The level prim keeps its StandardMaterial handle (made invisible: black, alpha 0, Add) so the
//! WP-A5 NPC copies (npcdraw, which clone `MeshMaterial3d<StandardMaterial>`) keep working: every
//! entity that gets such a handle receives a child with the same mesh (and skin) and the volume
//! material (`attach_env_volumes`). Not traced by Solari (only StandardMaterial meshes are), so the
//! look is identical with GI Off and On.

use bevy::pbr::{ExtendedMaterial, MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline, MaterialPlugin};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, CompareFunction, Face, RenderPipelineDescriptor, SpecializedMeshPipelineError};
use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::mesh::skinning::SkinnedMesh;
use std::collections::HashMap;

/// Pipeline-only extension: back faces, inverted depth test (see module doc).
#[derive(Asset, AsBindGroup, Reflect, Debug, Clone, Default)]
pub struct EnvVolumeExt {}

impl MaterialExtension for EnvVolumeExt {
    fn enable_prepass() -> bool {
        false
    }
    fn enable_shadows() -> bool {
        false
    }
    fn specialize(_: &MaterialExtensionPipeline, descriptor: &mut RenderPipelineDescriptor, _: &MeshVertexBufferLayoutRef, _: MaterialExtensionKey<Self>) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = Some(Face::Front);
        if let Some(ds) = descriptor.depth_stencil.as_mut() {
            ds.depth_write_enabled = false.into();
            ds.depth_compare = CompareFunction::Less.into();
        }
        Ok(())
    }
}

/// Verification switch `SLY2_C7_BEFORE=1` (port addition): skip the WP-C7 looks (env volumes keep the
/// WP-A5 shell, no level fog) for before / after screenshots.
pub fn c7_before() -> bool {
    std::env::var("SLY2_C7_BEFORE").is_ok_and(|v| v == "1")
}

pub type EnvVolumeMaterial = ExtendedMaterial<StandardMaterial, EnvVolumeExt>;

/// Level StandardMaterial handles of env-only prims -> their volume material.
#[derive(Resource, Default)]
pub struct EnvVolumes(pub HashMap<AssetId<StandardMaterial>, Handle<EnvVolumeMaterial>>);

/// Marks an entity whose volume child exists.
#[derive(Component)]
pub struct EnvVolumeAttached;

/// The volume child of an env-only prim.
#[derive(Component)]
pub struct EnvVolumeChild;

/// Volume material for the A5 env-glow material `m` (tint + alpha mode already set): back faces.
pub fn volume_material(m: &StandardMaterial) -> EnvVolumeMaterial {
    let mut base = m.clone();
    base.unlit = true;
    base.cull_mode = Some(Face::Front);
    base.double_sided = false;
    base.base_color_texture = None;
    base.emissive_texture = None;
    ExtendedMaterial { base, extension: EnvVolumeExt {} }
}

/// The level prim's own material once its volume exists: draws nothing.
pub fn hide_material(m: &mut StandardMaterial) {
    m.base_color = Color::srgba(0.0, 0.0, 0.0, 0.0);
    m.alpha_mode = AlphaMode::Add;
    m.unlit = true;
}

/// Gives every entity carrying an env-only level material its volume child (level prims, NPC
/// copies, re-spawns).
pub fn attach_env_volumes(
    mut commands: Commands,
    vols: Res<EnvVolumes>,
    q: Query<(Entity, &Mesh3d, &MeshMaterial3d<StandardMaterial>, Option<&SkinnedMesh>), (Added<MeshMaterial3d<StandardMaterial>>, Without<EnvVolumeAttached>)>,
) {
    if vols.0.is_empty() {
        return;
    }
    for (e, mesh, mat, skin) in &q {
        let Some(v) = vols.0.get(&mat.0.id()) else { continue };
        let mut c = commands.spawn((EnvVolumeChild, Mesh3d(mesh.0.clone()), MeshMaterial3d(v.clone()), Transform::IDENTITY, ChildOf(e)));
        if let Some(s) = skin {
            c.insert((s.clone(), bevy::camera::visibility::NoFrustumCulling));
        }
        commands.entity(e).insert(EnvVolumeAttached);
    }
}

pub struct EnvVolumePlugin;
impl Plugin for EnvVolumePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<EnvVolumeMaterial>::default()).init_resource::<EnvVolumes>().add_systems(PostUpdate, attach_env_volumes);
    }
}
