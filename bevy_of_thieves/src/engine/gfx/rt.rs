//! WP-R: global ray-traced illumination (port addition, no PS2 equivalent). `[video]
//! global_illumination = "off" | "raytraced"` (settings.toml) is the choice at startup; A12: the Graphics
//! menu switches it LIVE (`RtState::requested` -> `rt_switch`, per CM camera):
//! - "off" = pure raster (forward StandardMaterial, always valid; the default).
//! - "raytraced" = Bevy Solari (feature `rt`, plugins always registered): `SolariLighting` on the CM camera +
//!   a `RaytracingMesh3d` for every static opaque level surface. When the GPU lacks the ray-query
//!   feature set (`SolariPlugins::required_wgpu_features`) the option is greyed and the app stays raster.
//!
//! Solari BLAS rule (bevy_solari 0.20 `scene/blas.rs is_mesh_raytracing_compatible`): TriangleList,
//! U32 indices, `enable_raytracing`, and EXACTLY the attributes {POSITION, NORMAL, UV_0, TANGENT}.
//! gfx::game::to_bevy_mesh also carries COLOR (the baked PS2 vertex colours, the VU1 lighting input),
//! so a level mesh is never accepted as-is. The raster/G-buffer mesh keeps COLOR (vertex colours
//! modulate the base colour of every directly visible surface, raster and RT alike); the BLAS gets a
//! stripped copy (`raytracing_mesh`), i.e. the vertex colours are a separate input of the raster
//! material path and NOT the only lighting — traced bounces see the plain texture albedo.
//! Skinned meshes (no BLAS skinning in Solari) and blended materials are left out of the BLAS.

use super::mesh::{MeshPrim, MeshRole};

/// Why a primitive's data would be invalid as a Solari BLAS / lit StandardMaterial input.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum RtIssue {
    /// ink edges / shadow volumes are not drawn as surfaces
    NotSurface,
    /// skinned (Solari does not skin BLAS geometry)
    Skinned,
    Empty,
    /// attribute count differs from the position count
    AttributeLength(&'static str),
    NonFinite(&'static str),
    /// normal not unit length (|n| outside 0.9..1.1)
    BadNormal,
    /// tangent xyz not unit or w not +-1
    BadTangent,
    IndexOutOfRange,
    NotTriangleList,
}

/// Check one gfx primitive against the Solari requirements (pure data, no GPU).
pub fn prim_rt_issues(p: &MeshPrim) -> Vec<RtIssue> {
    let mut v = vec![];
    if p.role != MeshRole::Surface {
        v.push(RtIssue::NotSurface);
    }
    if !p.weights.is_empty() {
        v.push(RtIssue::Skinned);
    }
    let n = p.positions.len();
    if n == 0 || p.indices.is_empty() {
        v.push(RtIssue::Empty);
    }
    for (name, len) in [("NORMAL", p.normals.len()), ("UV_0", p.uvs.len()), ("TANGENT", p.tangents.len()), ("COLOR", p.colors.len())] {
        if len != n {
            v.push(RtIssue::AttributeLength(name));
        }
    }
    let fin = |s: &[f32]| s.iter().all(|x| x.is_finite());
    if !p.positions.iter().all(|x| fin(x)) {
        v.push(RtIssue::NonFinite("POSITION"));
    }
    if !p.uvs.iter().all(|x| fin(x)) {
        v.push(RtIssue::NonFinite("UV_0"));
    }
    if !p.normals.iter().all(|x| fin(x)) {
        v.push(RtIssue::NonFinite("NORMAL"));
    } else if p.normals.iter().any(|x| !(0.9..1.1).contains(&len3(*x))) {
        v.push(RtIssue::BadNormal);
    }
    if !p.tangents.iter().all(|x| fin(x)) {
        v.push(RtIssue::NonFinite("TANGENT"));
    } else if p.tangents.iter().any(|t| !(0.9..1.1).contains(&len3([t[0], t[1], t[2]])) || t[3].abs() != 1.0) {
        v.push(RtIssue::BadTangent);
    }
    if p.indices.len() % 3 != 0 {
        v.push(RtIssue::NotTriangleList);
    }
    if p.indices.iter().any(|&i| i as usize >= n) {
        v.push(RtIssue::IndexOutOfRange);
    }
    v
}

fn len3(a: [f32; 3]) -> f32 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

/// WP-R2 default `RtTuning::fill`: share of the raster ambient applied as the baked-light fill.
pub const RT_FILL: f32 = 1.6;
/// WP-R2 default `RtTuning::sky`: share of the raster ambient used as the traced sky.
pub const RT_SKY: f32 = 0.5;
/// Option B: share of a light kept where its traced shadow ray is blocked (see `shader_patches`).
pub const RT_SHADOW_FILL: f32 = 0.85;

/// Solari WESL lines patched at shader load: (original line, replacement).
/// - RAY_T_MIN / RECONNECTION_RELAX_DISTANCE: metres -> Sly units (WP-R). RAY_T_MIN is both the ray
///   start and the normal offset; 0.001 m is far below the f32 precision of level coordinates
///   (~1e4 units) and every shadow ray would hit its own surface.
/// - WP-R2 baked-light fill: the final shade (restir.wesl `spatial_and_shade`, no_restir.wesl) adds
///   `G-buffer base colour * fill` (nits, before exposure). The G-buffer base colour is the texture
///   times the PS2 vertex colours (the raster mesh keeps COLOR), so this is the original's
///   `vertex colour * ambient` term, like raster's GlobalAmbientLight but only on primary hits.
/// - LOOK: unlit materials (Bevy deferred writes base 0 + emissive = base colour, e.g. the raster-baked
///   PS2 per-vertex lit level prims, gfx::game::vu1_lit_grey) are shown at their raster value
///   (emissive / exposure), so RT level surfaces get exactly the PS2 light model (linear falloff).
/// - LOOK Option B (user decision): a level LIGHT never casts shadows on the PS2 (VU1 per-vertex
///   lighting, no occlusion), so a traced shadow ray keeps `RT_SHADOW_FILL` of the light: shadows
///   stay visible but shadowed areas sit near the PS2/raster brightness instead of going black.
///   Lit areas are unchanged (visibility 1).
pub fn shader_patches(fill_nits: [f32; 3]) -> Vec<(String, String)> {
    let [r, g, b] = fill_nits;
    vec![
        (
            "return f32(ray_hit.kind == RAY_QUERY_INTERSECTION_NONE);".into(),
            format!("return max(f32(ray_hit.kind == RAY_QUERY_INTERSECTION_NONE), {RT_SHADOW_FILL:?});"),
        ),
        ("const RAY_T_MIN = 0.001f;".into(), "const RAY_T_MIN = 0.5f;".into()),
        ("const RECONNECTION_RELAX_DISTANCE = 1.0;".into(), "const RECONNECTION_RELAX_DISTANCE = 100.0;".into()),
        (
            "pixel_color += surface.material.emissive;".into(),
            format!("pixel_color += surface.material.emissive + surface.material.base_color * vec3<f32>({r:?}, {g:?}, {b:?});
    pixel_color = select(pixel_color, surface.material.emissive / view.exposure, all(surface.material.base_color == vec3<f32>(0.0)));"),
        ),
        // LOOK: unlit pixels (raster-baked PS2 vertex light) show only their emissive: no traced
        // direct / indirect radiance (the PS2 light model is already in the bake)
        (
            "pixel_color += textureLoad(view_output, global_id.xy).rgb;".into(),
            "pixel_color += textureLoad(view_output, global_id.xy).rgb * f32(any(surface.material.base_color != vec3<f32>(0.0)));
    pixel_color = select(pixel_color, surface.material.emissive / view.exposure, all(surface.material.base_color == vec3<f32>(0.0)));".into(),
        ),
        // LOOK R13 (perf, same image): those unlit pixels skip the ReSTIR work whose result the two
        // patches above discard: no initial path / DI candidates / temporal merge (initial_and_temporal)
        // and no spatial merge (spatial_and_shade); they write an empty reservoir and their emissive.
        // At 5120x1440 RT High most of the frame is baked level geometry.
        (
            "    let initial = generate_initial_path(".into(),
            "    if all(surface.material.base_color == vec3<f32>(0.0)) {
        reservoirs_b[pixel_index] = empty_reservoir();
        textureStore(view_output, global_id.xy, vec4(0.0));
        return;
    }
    let initial = generate_initial_path(".into(),
        ),
        // same early-out in no_restir.wesl `initial_and_shade` (Medium / Low): identical output (the fill
        // patch already replaces those pixels by their emissive), without the initial path.
        (
            "    let path = generate_initial_path(".into(),
            "    if all(surface.material.base_color == vec3<f32>(0.0)) {
        textureStore(view_output, global_id.xy, vec4(surface.material.emissive * view.exposure, 1.0));
        return;
    }
    let path = generate_initial_path(".into(),
        ),
        (
            "    let input_reservoir = reservoirs_b[pixel_index];".into(),
            "    if all(surface.material.base_color == vec3<f32>(0.0)) {
        reservoirs_a[pixel_index] = empty_reservoir();
        textureStore(view_output, global_id.xy, vec4(surface.material.emissive, 1.0));
        return;
    }
    let input_reservoir = reservoirs_b[pixel_index];".into(),
        ),
    ]
}

/// Apply every patch whose original line occurs in `src`.
pub fn apply_patches(src: &str, patches: &[(String, String)]) -> String {
    patches.iter().fold(src.to_string(), |s, (a, b)| s.replace(a.as_str(), b))
}

/// A8: HDR blend clamp. PS2 vertex colours keep alpha 0xFF (`ps2_color` -> 1.99), so a Blend
/// surface's shader alpha is texture alpha * ~2. The raster camera renders to an LDR (unorm) target,
/// where the fragment output is clamped to 1 before blending: alpha = min(t * v, 1). The Solari camera
/// is Hdr (Rgba16Float): no clamp, the destination factor 1 - alpha goes negative and leaves (t0 plant)
/// broke up into over-saturated / black / see-through triangles (ENGINE_PORT.md A7/A8). The RT path
/// splits k = max(1, max vertex alpha) between mesh and material: COLOR.a / k and texture alpha
/// min(a * k, 1), giving min(t * k, 1) * v / k = the LDR result wherever v = k (uniform 0xFF alpha).
pub fn ldr_alpha_k(alphas: impl Iterator<Item = f32>) -> f32 {
    alphas.filter(|a| a.is_finite()).fold(1.0f32, f32::max)
}

/// Texture half of the A8 split: alpha byte (linear in every RGBA8 format) -> min(a * k, 255).
pub fn scale_alpha_rgba8(rgba: &mut [u8], k: f32) {
    for a in rgba.iter_mut().skip(3).step_by(4) {
        *a = (*a as f32 * k).round().min(255.0) as u8;
    }
}

#[cfg(feature = "rt")]
pub use bevy_rt::*;

/// A12 live GI state (every build; `ui::plugin` writes `requested` from the Graphics menu, rt builds'
/// `rt_switch` applies it). Port addition, no PS2 equivalent.
#[cfg_attr(feature = "game", derive(bevy::prelude::Resource))]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RtState {
    /// GI the user asked for (settings.toml at start, then the Graphics menu)
    pub requested: bool,
    /// Solari lighting currently on the CM camera
    pub active: bool,
    /// the device has `SolariPlugins::required_wgpu_features()` (rt builds only)
    pub available: bool,
    /// why ray tracing is not offered (shown on the greyed Graphics row); None when available
    pub unavailable: Option<String>,
}

/// What `rt_switch` does this frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GiSwitch {
    Keep,
    Enable,
    Disable,
}

/// Requested vs running GI -> switch (never enables without the GPU features).
pub fn gi_switch(st: &RtState) -> GiSwitch {
    match (st.requested && st.available, st.active) {
        (true, false) => GiSwitch::Enable,
        (false, true) => GiSwitch::Disable,
        _ => GiSwitch::Keep,
    }
}

/// A12 verification schedule `SLY2_GI_AT="<frame>:<on|off>,..."`: the GI choice set at `frame` (None = no entry).
pub fn gi_schedule(spec: &str, frame: u32) -> Option<bool> {
    spec.split(',').filter_map(|e| e.trim().split_once(':')).find(|(f, _)| f.trim().parse() == Ok(frame)).map(|(_, v)| matches!(v.trim(), "on" | "1" | "raytraced"))
}

/// Menu reason for a greyed GI row (None = ray tracing can be switched on).
pub fn gi_unavailable_reason(rt_build: bool, gpu_ok: bool) -> Option<String> {
    if !rt_build {
        Some("build without rt".into())
    } else if !gpu_ok {
        Some("no ray tracing GPU".into())
    } else {
        None
    }
}

#[cfg(feature = "rt")]
mod bevy_rt {
    use super::{apply_patches, gi_switch, gi_unavailable_reason, ldr_alpha_k, scale_alpha_rgba8, shader_patches, GiSwitch, RtState, RT_FILL, RT_SKY};
    use crate::engine::camera::bevy_sync::CmCamera;
    use bevy::anti_alias::taa::TemporalAntiAliasing;
    use bevy::camera::CameraMainTextureUsages;
    use bevy::core_pipeline::prepass::{DeferredPrepass, DeferredPrepassDoubleBuffer, DepthPrepass, DepthPrepassDoubleBuffer, MotionVectorPrepass};
    use bevy::mesh::skinning::SkinnedMesh;
    use bevy::mesh::{Indices, PrimitiveTopology};
    use bevy::pbr::DefaultOpaqueRendererMethod;
    use bevy::prelude::*;
    use bevy::render::render_resource::TextureUsages;
    use bevy::render::renderer::RenderDevice;
    use bevy::shader::{Shader, Source};
    use bevy::solari::prelude::{RaytracingMesh3d, SolariLighting, SolariPlugins};

    /// The exact Solari BLAS test (bevy_solari 0.20 scene/blas.rs, private there).
    pub fn is_solari_compatible(m: &Mesh) -> bool {
        let attrs = m.attributes().map(|(a, _)| (a.id, a.format)).eq([
            (Mesh::ATTRIBUTE_POSITION.id, Mesh::ATTRIBUTE_POSITION.format),
            (Mesh::ATTRIBUTE_NORMAL.id, Mesh::ATTRIBUTE_NORMAL.format),
            (Mesh::ATTRIBUTE_UV_0.id, Mesh::ATTRIBUTE_UV_0.format),
            (Mesh::ATTRIBUTE_TANGENT.id, Mesh::ATTRIBUTE_TANGENT.format),
        ]);
        m.enable_raytracing && m.primitive_topology() == PrimitiveTopology::TriangleList && attrs && matches!(m.indices(), Some(Indices::U32(_)))
    }

    /// BLAS copy of a level mesh: same POSITION/NORMAL/UV_0/TANGENT + U32 indices, every other
    /// attribute (COLOR, joints) dropped. None when the mesh lacks one of the four or isn't triangles.
    pub fn raytracing_mesh(m: &Mesh) -> Option<Mesh> {
        if m.primitive_topology() != PrimitiveTopology::TriangleList {
            return None;
        }
        let mut out = m.clone();
        let keep = [Mesh::ATTRIBUTE_POSITION.id, Mesh::ATTRIBUTE_NORMAL.id, Mesh::ATTRIBUTE_UV_0.id, Mesh::ATTRIBUTE_TANGENT.id];
        let drop: Vec<_> = m.attributes().map(|(a, _)| a.id).filter(|id| !keep.contains(id)).collect();
        for id in drop {
            out.remove_attribute(id);
        }
        if let Some(Indices::U16(i)) = out.indices() {
            let v: Vec<u32> = i.iter().map(|&x| x as u32).collect();
            out.insert_indices(Indices::U32(v));
        }
        out.enable_raytracing = true;
        is_solari_compatible(&out).then_some(out)
    }

    /// Sly world units per metre (PS2 units are ~1 cm: follow-camera distance 450 = a few metres).
    /// Solari's world-space tuning (ray epsilons, world-cache cells, GI ray length) assumes metres.
    pub const UNITS_PER_METER: f32 = 100.0;

    /// WIRE-C3: direct-light samples multiplier over Solari's defaults (speckle reduction).
    pub const RT_DI_SAMPLE_MUL: u32 = 4;

    /// `SolariLighting` with its metre-based distances converted to Sly units.
    pub fn solari_lighting() -> SolariLighting {
        solari_lighting_q(RT_DI_SAMPLE_MUL)
    }

    /// WIRE-OPT: `solari_lighting` with the settings.toml `[graphics] rt_quality` sample multiplier.
    pub fn solari_lighting_q(mul: u32) -> SolariLighting {
        let d = SolariLighting::default();
        SolariLighting {
            world_cache_max_gi_ray_distance: d.world_cache_max_gi_ray_distance * UNITS_PER_METER,
            world_cache_position_base_cell_size: d.world_cache_position_base_cell_size * UNITS_PER_METER,
            world_cache_position_lod_scale: d.world_cache_position_lod_scale * UNITS_PER_METER,
            // no denoiser in this build (DLSS RR not enabled): ReSTIR spatio-temporal reuse keeps the
            // many small light sources (point-light proxies) from rendering as sparse speckles
            restir: true,
            // WIRE-C3 denoise: no DLSS-RR (needs the NVIDIA SDK `dlss` feature); more light samples per
            // pixel + longer ReSTIR / world-cache history, then TAA accumulation (`RtTuning::taa`)
            primary_di_samples: d.primary_di_samples * mul,
            secondary_di_samples: d.secondary_di_samples * mul,
            confidence_weight_cap: d.confidence_weight_cap * 2.0,
            world_cache_max_temporal_samples: d.world_cache_max_temporal_samples * 2.0,
            ..d
        }
    }

    /// Apply `shader_patches` (unit constants + the WP-R2 baked-light fill) to Solari shader sources
    /// as they finish loading (embedded assets go through the asset server, so they appear a few
    /// frames after startup — after game.rs `setup` inserted GlobalAmbientLight — before any pipeline).
    pub fn patch_solari_shaders(
        mut ev: MessageReader<AssetEvent<Shader>>,
        mut shaders: ResMut<Assets<Shader>>,
        tuning: Res<RtTuning>,
        ambient: Option<Res<GlobalAmbientLight>>,
        mut orig: Local<Vec<(AssetId<Shader>, String)>>,
        mut cams: Query<&mut EnvironmentMapLight, With<CmCamera>>,
        mut images: ResMut<Assets<Image>>,
    ) {
        let amb = ambient.as_ref().map(|a| a.color.to_linear().to_vec3() * a.brightness).unwrap_or(Vec3::ZERO);
        let patches = shader_patches((amb * tuning.fill).to_array());
        let ids: Vec<_> = ev
            .read()
            .filter_map(|e| match e {
                AssetEvent::Added { id } => Some(*id),
                _ => None,
            })
            .filter(|id| matches!(shaders.get(*id).map(|sh| &sh.source), Some(Source::Wesl(src)) if patches.iter().any(|(a, _)| src.contains(a))))
            .collect();
        for id in ids {
            let Some(mut sh) = shaders.get_mut(id) else { continue };
            if let Source::Wesl(src) = &sh.source {
                orig.push((id, src.to_string()));
                sh.source = Source::Wesl(apply_patches(src, &patches).into());
                info!("rt: patched Solari shader {} (world units {} per metre, baked-light fill {:?} nits)", sh.path, UNITS_PER_METER, amb * tuning.fill);
            }
        }
        // WP-C9: the level sets GlobalAmbientLight from its LIGHT objects (gfx::light::level_ambient):
        // re-patch the fill from the unpatched sources (the pipelines re-specialise on the shader
        // change) and follow the sky light (rt_switch inserted it from the ambient of that moment).
        if ambient.as_ref().is_some_and(|a| a.is_changed() && !a.is_added()) {
            for (id, src) in orig.iter() {
                if let Some(mut sh) = shaders.get_mut(*id) {
                    sh.source = Source::Wesl(apply_patches(src, &patches).into());
                }
            }
            if let Some(a) = ambient.as_ref().filter(|_| tuning.sky > 0.0) {
                for mut env in cams.iter_mut() {
                    *env = EnvironmentMapLight::solid_color(&mut images, a.color).with_intensity(a.brightness * tuning.sky);
                }
            }
            info!("rt: ambient changed, baked-light fill {:?} nits", amb * tuning.fill);
        }
    }

    /// WP-R2 look tuning (defaults measured on tables 3/10/20, ENGINE_PORT.md R2 findings).
    #[derive(Resource, Clone, Copy, Debug, PartialEq)]
    pub struct RtTuning {
        /// fraction of the raster GlobalAmbientLight added unoccluded to every directly visible
        /// surface as `base_colour * vertex colour * ambient` (the PS2 baked-lighting term; stops
        /// interiors from going black where Solari finds no light path)
        pub fill: f32,
        /// fraction of the raster GlobalAmbientLight used as the traced sky environment (occluded)
        pub sky: f32,
        /// temporal accumulation (Bevy TAA) on the RT camera: no DLSS-RR in this build
        pub taa: bool,
    }

    impl Default for RtTuning {
        fn default() -> Self {
            RtTuning { fill: RT_FILL, sky: RT_SKY, taa: true }
        }
    }

    /// Emissive radiance (nits) below which a material is not treated as a Solari light source.
    pub const NEGLIGIBLE_EMISSIVE: f32 = 1.0;

    /// Point light -> its Solari emissive-sphere proxy (Solari 0.20 traces only directional lights and
    /// emissive meshes; the level's class-4 point LIGHTs would otherwise vanish in raytraced mode).
    #[derive(Component)]
    pub struct RtLightProxy(pub Entity, pub Handle<StandardMaterial>, pub f32);

    /// Proxy sphere radius (world units) when the light has no radius of its own.
    pub const PROXY_RADIUS: f32 = 10.0;

    /// Emissive radiance (nits) of a sphere of radius `r` emitting `lumens` like an isotropic point
    /// light: intensity I = lumens / 4pi (cd), seen as a disc of area pi r^2 -> L = I / (pi r^2).
    pub fn proxy_radiance(lumens: f32, r: f32) -> f32 {
        lumens / (4.0 * std::f32::consts::PI) / (std::f32::consts::PI * r * r)
    }

    /// Keep one ray-traced emissive sphere per PointLight (child entity, no Mesh3d = never rasterized).
    #[allow(clippy::type_complexity)]
    pub fn sync_point_light_proxies(
        mut commands: Commands,
        changed: Query<(Entity, &PointLight, Option<&RtLightProxy>), Changed<PointLight>>,
        mut removed: RemovedComponents<PointLight>,
        proxies: Query<&RtLightProxy, Without<PointLight>>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut mats: ResMut<Assets<StandardMaterial>>,
    ) {
        for e in removed.read() {
            if let Ok(p) = proxies.get(e) {
                commands.entity(p.0).despawn();
                commands.entity(e).remove::<RtLightProxy>();
            }
        }
        for (e, l, old) in &changed {
            let r = if l.radius > 0.0 { l.radius } else { PROXY_RADIUS };
            // RENDER: colour / intensity change -> update the proxy material in place (a new proxy entity
            // = new BLAS instance + Solari light-list change; respawning on every light write made
            // 6-10 frame bursts of 2-3x Solari lighting cost at RT High)
            if let Some(p) = old {
                if p.2 == r {
                    let emissive = LinearRgba::from(l.color) * proxy_radiance(l.intensity, r);
                    if mats.get(&p.1).is_some_and(|m| m.emissive != emissive) {
                        if let Some(mut m) = mats.get_mut(&p.1) {
                            m.emissive = emissive;
                        }
                    }
                    continue;
                }
                commands.entity(p.0).despawn();
            }
            let mut m = Sphere::new(r).mesh().ico(1).expect("ico sphere");
            m.generate_tangents().expect("sphere tangents");
            let Some(m) = raytracing_mesh(&m) else { continue };
            let emissive = LinearRgba::from(l.color) * proxy_radiance(l.intensity, r);
            let mat = mats.add(StandardMaterial { base_color: Color::BLACK, emissive, ..default() });
            let p = commands.spawn((RaytracingMesh3d(meshes.add(m)), MeshMaterial3d(mat.clone()), Transform::default(), ChildOf(e))).id();
            commands.entity(e).insert(RtLightProxy(p, mat, r));
        }
    }

    /// Entity checked and kept out of the BLAS (blended, incompatible).
    #[derive(Component)]
    pub struct RtExcluded;

    /// WIRE-C3: skinned / animated mesh in the BLAS. Solari 0.20 has no GPU BLAS skinning; its
    /// `prepare_raytracing_blas` rebuilds the BLAS of every modified mesh asset, so the RT copy `blas` is
    /// re-skinned on the CPU (same linear blend skinning as Bevy's skinning.wgsl) whenever a joint moved.
    #[derive(Component)]
    pub struct RtSkin {
        pub src: Handle<Mesh>,
        pub blas: Handle<Mesh>,
        /// joint matrices of the last refit (skip unchanged poses)
        pub last: Vec<Mat4>,
    }

    /// Linear blend skinning of `pos`/`nrm` into the entity-local space of `inv_entity`.
    pub fn skin_positions(pos: &[[f32; 3]], nrm: &[[f32; 3]], joints: &[[u16; 4]], weights: &[[f32; 4]], mats: &[Mat4], inv_entity: Mat4) -> (Vec<[f32; 3]>, Vec<[f32; 3]>) {
        let mut po = Vec::with_capacity(pos.len());
        let mut no = Vec::with_capacity(pos.len());
        for i in 0..pos.len() {
            let mut m = Mat4::ZERO;
            for k in 0..4 {
                let w = weights.get(i).map_or(0.0, |w| w[k]);
                if w != 0.0 {
                    m += *mats.get(joints[i][k] as usize).unwrap_or(&Mat4::IDENTITY) * w;
                }
            }
            let m = inv_entity * m;
            po.push(m.transform_point3(Vec3::from(pos[i])).into());
            let n = nrm.get(i).map_or(Vec3::Z, |n| m.transform_vector3(Vec3::from(*n)).normalize_or_zero());
            no.push(n.into());
        }
        (po, no)
    }

    /// LOOK R13: distance (Sly units) under which a skinned BLAS refits at the full `SLY2_RT_SKIN_DIV` rate.
    pub const RT_SKIN_NEAR: f32 = 1500.0;

    /// WIRE-C3: per-frame BLAS refit of skinned meshes (runs while RT is on).
    pub fn rt_skin_refit(
        mut commands: Commands,
        mut q: Query<(Entity, &mut RtSkin, &SkinnedMesh, &GlobalTransform)>,
        joints: Query<&GlobalTransform>,
        cams: Query<&GlobalTransform, With<CmCamera>>,
        ibps: Res<Assets<bevy::mesh::skinning::SkinnedMeshInverseBindposes>>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut frame: Local<u32>,
    ) {
        use bevy::mesh::VertexAttributeValues as V;
        let mut refits = 0u32;
        let t0 = std::time::Instant::now();
        // UPSCALE R2: animated skins refit on 1 of every `div` frames, staggered by entity (BLAS = GI/shadow
        // geometry only; the drawn mesh is GPU-skinned every frame). env SLY2_RT_SKIN_DIV (default 2, 1 = every frame)
        let div = std::env::var("SLY2_RT_SKIN_DIV").ok().and_then(|v| v.parse::<u32>().ok()).unwrap_or(2).max(1);
        *frame = frame.wrapping_add(1);
        // LOOK R13 (perf): every refit is a new mesh asset + BLAS build (~266 per frame in t4 at div 2, the
        // largest RT cost after the lighting pass). Skins farther than RT_SKIN_NEAR from the camera refit
        // div x (1 + d / RT_SKIN_NEAR, max 4) frames apart (far shadows / GI lag <= 8 frames).
        let cam = cams.iter().next().map(|c| c.translation());
        for (e, mut s, sk, gt) in &mut q {
            let lod = cam.map_or(1, |c| 1 + ((gt.translation().distance(c) / RT_SKIN_NEAR) as u32).min(3));
            if !s.last.is_empty() && (e.index_u32().wrapping_add(*frame)) % (div * lod) != 0 {
                continue;
            }
            let Some(ibp) = ibps.get(&sk.inverse_bindposes) else { continue };
            // UPSCALE R2: compare the pose in entity-local space (what the BLAS stores), so a mesh that only
            // moves/turns as a whole keeps its BLAS (TLAS instance transform covers that) instead of a rebuild
            let inv = gt.to_matrix().inverse();
            let mats: Vec<Mat4> = sk.joints.iter().zip(ibp.iter()).map(|(j, b)| inv * joints.get(*j).map_or(Mat4::IDENTITY, |g| g.to_matrix()) * *b).collect();
            if mats.len() == s.last.len() && mats.iter().zip(&s.last).all(|(a, b)| a.abs_diff_eq(*b, 1e-4)) {
                continue;
            }
            let Some(src) = meshes.get(&s.src) else { continue };
            let (Some(V::Float32x3(p)), Some(V::Uint16x4(j)), Some(V::Float32x4(w))) =
                (src.attribute(Mesh::ATTRIBUTE_POSITION), src.attribute(Mesh::ATTRIBUTE_JOINT_INDEX), src.attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT))
            else {
                continue;
            };
            let n = match src.attribute(Mesh::ATTRIBUTE_NORMAL) {
                Some(V::Float32x3(n)) => n.clone(),
                _ => Vec::new(),
            };
            let (po, no) = skin_positions(p, &n, j, w, &mats, Mat4::IDENTITY);
            refits += 1;
            // a NEW asset per refit: modifying the same mesh twice before Solari's compaction queue reaches
            // it queues a second compaction of one BLAS ("Compaction is already being prepared"); the
            // dropped old handle goes through Solari's deferred BLAS deletion instead
            let Some(mut b) = meshes.get(&s.blas).cloned() else { continue };
            b.insert_attribute(Mesh::ATTRIBUTE_POSITION, po);
            if !n.is_empty() {
                b.insert_attribute(Mesh::ATTRIBUTE_NORMAL, no);
            }
            s.blas = meshes.add(b);
            commands.entity(e).insert(RaytracingMesh3d(s.blas.clone()));
            s.last = mats;
        }
        if std::env::var_os("SLY2_RT_SKIN_LOG").is_some() {
            eprintln!("rt_skin_refit: {refits} of {} meshes, {:.2} ms", q.iter().count(), t0.elapsed().as_secs_f64() * 1e3);
        }
    }

    /// A12: entity whose material look was switched to the RT variant (`rt_apply_look`); holds the raster
    /// material handle when the A8 blend split swapped it (restored by `rt_revert_look`).
    #[derive(Component)]
    pub struct RtLook(pub Option<Handle<StandardMaterial>>);

    /// A12: what the CM camera had before `rt_switch` turned Solari on (restored when GI goes off).
    #[derive(Component, Clone)]
    pub struct RtCamSaved {
        pub msaa: Option<Msaa>,
        pub usages: Option<CameraMainTextureUsages>,
        pub hdr: bool,
        pub depth: bool,
        pub motion: bool,
        pub deferred: bool,
        pub taa: bool,
        pub env: bool,
    }

    /// A12: raster values of every material / mesh the RT look changed (live switch back to raster).
    #[derive(Resource, Default)]
    pub struct RtLookCache {
        /// materials already examined for the emissive -> albedo swap while RT is on
        pub checked: bevy::platform::collections::HashSet<AssetId<StandardMaterial>>,
        /// swapped materials: raster (emissive, emissive_texture, base_color_texture)
        pub dimmed: bevy::platform::collections::HashMap<AssetId<StandardMaterial>, (LinearRgba, Option<Handle<Image>>, Option<Handle<Image>>)>,
        pub blend: RtBlendCache,
        /// `rt_revert_look` still has to run (set by `rt_switch` when GI goes off)
        pub revert: bool,
    }

    /// Add to the app once in rt builds. Solari is ALWAYS registered (A12): its plugins only enable
    /// themselves when the device has `SolariPlugins::required_wgpu_features()`, and bevy_render requests
    /// every adapter feature at device creation (default `WgpuSettingsPriority::Functionality`).
    /// `raytraced` = the settings.toml choice at start; the Graphics menu changes `RtState::requested`
    /// at runtime and `rt_switch` applies it per camera.
    pub struct RtPlugin {
        pub raytraced: bool,
    }

    impl Plugin for RtPlugin {
        fn build(&self, app: &mut App) {
            app.insert_resource(RtState { requested: self.raytraced, ..default() }).init_resource::<RtTuning>().init_resource::<RtLookCache>();
            app.add_plugins(SolariPlugins);
            // SolariLightingPlugin::build made deferred the global default; raster (the default GI) is forward
            app.insert_resource(DefaultOpaqueRendererMethod::forward());
            app.add_systems(PreUpdate, patch_solari_shaders).add_systems(PostStartup, rt_init).add_systems(
                PostUpdate,
                (rt_switch, rt_revert_look.run_if(|c: Res<RtLookCache>| c.revert), (rt_apply_look, sync_point_light_proxies, rt_skin_refit, rt_apply_quality).run_if(|s: Res<RtState>| s.active)).chain(),
            );
        }
    }

    /// WIRE-OPT: settings.toml `[graphics] rt_quality` (Options > Graphics, live): the DI sample count of
    /// every Solari camera follows `LiveGraphics::rt_quality` (Low x1, Medium x2, High x4 = WIRE-C3 default).
    pub fn rt_apply_quality(live: Option<Res<crate::settings::LiveGraphics>>, mut cams: Query<(&mut SolariLighting, &Camera), With<CmCamera>>) {
        for (mut c, cam) in cams.iter_mut() {
            // LOOK: Auto resolves by the camera's render size
            let sz = cam.physical_viewport_size().unwrap_or_default();
            let q = live.as_ref().map(|l| l.rt_quality.resolve(sz.x, sz.y));
            let mul = q.map_or(RT_DI_SAMPLE_MUL, |q| q.di_sample_mul());
            let restir = q.map_or(true, |q| q.restir());
            let want = solari_lighting_q(mul);
            if c.restir != restir {
                c.restir = restir;
            }
            if c.primary_di_samples != want.primary_di_samples {
                c.primary_di_samples = want.primary_di_samples;
                c.secondary_di_samples = want.secondary_di_samples;
            }
        }
    }

    /// Feature check against the created device (`RtState::available` / `unavailable` reason for the menu).
    pub fn rt_init(dev: Option<Res<RenderDevice>>, mut st: ResMut<RtState>) {
        let need = SolariPlugins::required_wgpu_features();
        let have = dev.as_ref().map(|d| d.features());
        st.available = have.is_some_and(|f| f.contains(need));
        st.unavailable = gi_unavailable_reason(true, st.available);
        if !st.available {
            let missing = have.map(|f| need.difference(f));
            warn!("ray-traced GI unavailable: GPU lacks {missing:?}; raster only (Graphics menu row greyed)");
        }
    }

    /// A12 live GI switch (per CM camera): on = SolariLighting (+ Msaa off, storage-bindable main texture,
    /// traced sky, TAA), deferred opaque default; off = the camera's raster setup restored, forward default,
    /// RT material look reverted (`rt_revert_look`). Every material is touched so Bevy re-prepares it with
    /// the new render method (`DefaultOpaqueRendererMethod` is only read when a material is prepared).
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn rt_switch(
        mut commands: Commands,
        mut st: ResMut<RtState>,
        cams: Query<
            (Entity, Has<bevy::camera::Hdr>, Has<DepthPrepass>, Has<MotionVectorPrepass>, Has<DeferredPrepass>, Has<TemporalAntiAliasing>, Has<EnvironmentMapLight>, Option<&Msaa>, Option<&CameraMainTextureUsages>, Option<&RtCamSaved>),
            With<CmCamera>,
        >,
        mut dl: Query<&mut DirectionalLight>,
        mut pl: Query<&mut PointLight>,
        ambient: Option<Res<GlobalAmbientLight>>,
        mut images: ResMut<Assets<Image>>,
        tuning: Res<RtTuning>,
        mut mats: ResMut<Assets<StandardMaterial>>,
        mut cache: ResMut<RtLookCache>,
    ) {
        let sw = gi_switch(&st);
        if sw == GiSwitch::Keep {
            return;
        }
        let on = sw == GiSwitch::Enable;
        // Solari ignores GlobalAmbientLight; the raster ambient (same colour, nits) becomes a uniform sky
        // environment light instead, so it is occluded/bounced by the level like real sky light.
        let sky = ambient.filter(|_| on && tuning.sky > 0.0).map(|a| EnvironmentMapLight::solid_color(&mut images, a.color).with_intensity(a.brightness * tuning.sky));
        for (e, hdr, depth, motion, deferred, taa, env, msaa, usages, saved) in &cams {
            let mut c = commands.entity(e);
            if on {
                c.insert(RtCamSaved { msaa: msaa.cloned(), usages: usages.cloned(), hdr, depth, motion, deferred, taa, env });
                // Solari: Msaa off + storage-bindable main texture (bevy example 3d/solari.rs)
                c.insert((solari_lighting(), Msaa::Off, CameraMainTextureUsages::default().with(TextureUsages::STORAGE_BINDING)));
                if let Some(sky) = &sky {
                    c.insert(sky.clone());
                }
                if tuning.taa {
                    // WP-R2: TAA history = temporal accumulation of the 1-spp ReSTIR output (no denoiser)
                    c.insert(TemporalAntiAliasing::default());
                }
            } else {
                let s = saved.cloned().unwrap_or(RtCamSaved { msaa: None, usages: None, hdr: false, depth: false, motion: false, deferred: false, taa: false, env: false });
                c.remove::<(SolariLighting, DeferredPrepassDoubleBuffer, DepthPrepassDoubleBuffer, RtCamSaved)>();
                if !s.taa {
                    c.remove::<(TemporalAntiAliasing, bevy::render::camera::TemporalJitter, bevy::render::camera::MipBias)>();
                }
                if !s.env {
                    c.remove::<EnvironmentMapLight>();
                }
                if !s.hdr {
                    c.remove::<bevy::camera::Hdr>();
                }
                if !s.deferred {
                    c.remove::<DeferredPrepass>();
                }
                if !s.motion {
                    c.remove::<MotionVectorPrepass>();
                }
                if !s.depth {
                    c.remove::<DepthPrepass>();
                }
                match s.msaa {
                    Some(m) => c.insert(m),
                    None => c.remove::<Msaa>(),
                };
                match s.usages {
                    Some(u) => c.insert(u),
                    None => c.remove::<CameraMainTextureUsages>(),
                };
            }
        }
        if on {
            // Solari replaces shadow maps (the raster lights have them off already, gfx::game)
            for mut l in &mut dl {
                l.shadow_maps_enabled = false;
            }
            // re-sync every point-light proxy (`sync_point_light_proxies` only runs while RT is on)
            for mut l in &mut pl {
                l.shadow_maps_enabled = false;
                l.set_changed();
            }
        } else {
            cache.revert = true;
        }
        commands.insert_resource(if on { DefaultOpaqueRendererMethod::deferred() } else { DefaultOpaqueRendererMethod::forward() });
        // `Assets::iter_mut` queues AssetEvent::Modified for every material
        for _ in mats.iter_mut() {}
        st.active = on;
        info!("global illumination: {}", if on { "raytraced (Bevy Solari)" } else { "off (raster)" });
    }

    /// A12: undo `rt_apply_look` (raster materials / vertex alphas back) after GI went off. The BLAS copies
    /// and light proxies stay (never rasterized; reused when GI comes back on).
    pub fn rt_revert_look(mut commands: Commands, looks: Query<(Entity, &RtLook)>, mut mats: ResMut<Assets<StandardMaterial>>, mut meshes: ResMut<Assets<Mesh>>, mut cache: ResMut<RtLookCache>) {
        for (e, l) in &looks {
            let mut c = commands.entity(e);
            if let Some(h) = &l.0 {
                c.insert(MeshMaterial3d(h.clone()));
            }
            c.remove::<RtLook>();
        }
        let dimmed: Vec<_> = cache.dimmed.drain().collect();
        for (id, (em, emt, base)) in dimmed {
            if let Some(mut m) = mats.get_mut(id) {
                m.emissive = em;
                m.emissive_texture = emt;
                m.base_color_texture = base;
            }
        }
        let split: Vec<_> = cache.blend.mesh_k.drain().collect();
        for (id, k) in split {
            if k > 1.0 {
                if let Some(mut m) = meshes.get_mut(id) {
                    unsplit_vertex_alpha(&mut m, k);
                }
            }
        }
        cache.checked.clear();
        cache.revert = false;
    }

    /// A8 split undone: COLOR alpha * k.
    pub fn unsplit_vertex_alpha(m: &mut Mesh, k: f32) {
        if let Some(bevy::mesh::VertexAttributeValues::Float32x4(c)) = m.attribute_mut(Mesh::ATTRIBUTE_COLOR) {
            for v in c.iter_mut() {
                v[3] *= k;
            }
        }
    }

    /// A8: per-mesh vertex-alpha split factor and the per-(material, k) LDR-alpha material copies.
    #[derive(Default)]
    pub struct RtBlendCache {
        pub mesh_k: bevy::platform::collections::HashMap<AssetId<Mesh>, f32>,
        pub mats: bevy::platform::collections::HashMap<(AssetId<StandardMaterial>, u32), Handle<StandardMaterial>>,
    }

    /// A8: divide the COLOR alpha of `m` by k = `ldr_alpha_k` of its alphas (no-op when k = 1); returns k.
    pub fn split_vertex_alpha(m: &mut Mesh) -> f32 {
        let Some(bevy::mesh::VertexAttributeValues::Float32x4(c)) = m.attribute_mut(Mesh::ATTRIBUTE_COLOR) else { return 1.0 };
        let k = ldr_alpha_k(c.iter().map(|v| v[3]));
        if k > 1.0 {
            for v in c.iter_mut() {
                v[3] /= k;
            }
        }
        k
    }

    /// A8: the material half of the split: base texture alpha min(a * k, 1) (new image; RGBA8 formats
    /// only) and base_color alpha min(b * k, 1) when untextured.
    pub fn ldr_material_alpha(m: &mut StandardMaterial, k: f32, images: &mut Assets<Image>) {
        let Some(t) = m.base_color_texture.clone() else {
            let a = m.base_color.alpha();
            m.base_color.set_alpha((a * k).min(1.0));
            return;
        };
        let Some(img) = images.get(&t) else { return };
        if img.texture_descriptor.format.block_copy_size(None) != Some(4) || img.texture_descriptor.format.components() != 4 {
            return;
        }
        let mut img = img.clone();
        if let Some(d) = img.data.as_mut() {
            scale_alpha_rgba8(d, k);
            m.base_color_texture = Some(images.add(img));
        }
    }

    /// While GI is ray traced: every mesh entity gets the RT material look once (`RtLook`, undone by
    /// `rt_revert_look`) and every new static opaque level surface a BLAS copy (`raytracing_mesh`).
    #[allow(clippy::type_complexity)]
    pub fn rt_apply_look(
        mut commands: Commands,
        q: Query<(Entity, &Mesh3d, &MeshMaterial3d<StandardMaterial>, Has<SkinnedMesh>, Has<RaytracingMesh3d>, Has<RtExcluded>), Without<RtLook>>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut mats: ResMut<Assets<StandardMaterial>>,
        mut images: ResMut<Assets<Image>>,
        mut cache: ResMut<RtLookCache>,
    ) {
        if q.is_empty() {
            return;
        }
        let cache = &mut *cache;
        let blend = &mut cache.blend;
        // forget assets of unloaded levels (the copies are strong handles)
        blend.mats.retain(|(id, _), _| mats.contains(*id));
        blend.mesh_k.retain(|id, _| meshes.contains(*id));
        cache.dimmed.retain(|id, _| mats.contains(*id));
        let (mut added, mut excl) = (0, 0);
        for (e, m, mat, skinned, has_blas, excluded) in &q {
            if !meshes.contains(&m.0) {
                continue; // not loaded yet: retried next frame (keeps `mesh_k` from caching a guess)
            }
            // gfx::game maps the PS2 additive unlit colour layer (CLUT set 0; kinds 1/7 draw the lit pass
            // with the grey ramp set 1, ENGINE_PORT.md J3) to emissive_texture * WHITE (1 nit,
            // emissive_exposure_weight 0 = full colour in raster). Solari adds G-buffer emissive AFTER
            // exposure scaling (1/1000 -> invisible) and makes every emissive mesh a light source (noise),
            // so in RT the colour layer becomes the albedo instead (WP-R2): base = set-0 colour texture *
            // vertex colours, lit by the traced lights + the baked-light fill; the grey ramp is dropped.
            if cache.checked.insert(mat.0.id()) && mats.get(&mat.0).is_some_and(|m| m.emissive != LinearRgba::BLACK && m.emissive.to_vec3().max_element() <= NEGLIGIBLE_EMISSIVE) {
                if let Some(mut m) = mats.get_mut(&mat.0) {
                    cache.dimmed.insert(mat.0.id(), (m.emissive, m.emissive_texture.clone(), m.base_color_texture.clone()));
                    if let Some(t) = m.emissive_texture.take() {
                        m.base_color_texture = Some(t);
                    }
                    m.emissive = LinearRgba::BLACK;
                }
            }
            // A8: HDR blend clamp (see `ldr_alpha_k`): Blend surfaces get their vertex alpha divided by
            // k and a base texture whose alpha is min(a * k, 1), so the shader alpha never exceeds 1.
            let mut mat_h = mat.0.clone();
            let mut orig = None;
            if mats.get(&mat_h).is_some_and(|x| x.alpha_mode == AlphaMode::Blend) {
                let k = *blend.mesh_k.entry(m.0.id()).or_insert_with(|| meshes.get_mut(&m.0).map(|mut x| split_vertex_alpha(&mut x)).unwrap_or(1.0));
                if k > 1.0 {
                    let key = (mat_h.id(), k.to_bits());
                    let h = match blend.mats.get(&key) {
                        Some(h) => Some(h.clone()),
                        None => mats.get(&mat_h).cloned().map(|mut c| {
                            ldr_material_alpha(&mut c, k, &mut images);
                            let h = mats.add(c);
                            blend.mats.insert(key, h.clone());
                            h
                        }),
                    };
                    if let Some(h) = h {
                        commands.entity(e).insert(MeshMaterial3d(h.clone()));
                        orig = Some(mat_h.clone());
                        mat_h = h;
                    }
                }
            }
            commands.entity(e).insert(RtLook(orig));
            if has_blas || excluded {
                continue; // BLAS decision made during an earlier RT period
            }
            let Some(mesh) = meshes.get(&m.0) else { continue };
            let Some(mat) = mats.get(&mat_h) else { continue };
            let opaque = matches!(mat.alpha_mode, AlphaMode::Opaque | AlphaMode::Mask(_));
            match (opaque).then(|| raytracing_mesh(mesh)).flatten() {
                Some(rt) => {
                    let h = meshes.add(rt);
                    if skinned {
                        // WIRE-C3: skinned BLAS = CPU-skinned copy refitted every frame (`rt_skin_refit`)
                        commands.entity(e).insert(RtSkin { src: m.0.clone(), blas: h.clone(), last: Vec::new() });
                    }
                    commands.entity(e).insert(RaytracingMesh3d(h));
                    added += 1;
                }
                None => {
                    commands.entity(e).insert(RtExcluded);
                    excl += 1;
                }
            }
        }
        if added + excl > 0 {
            info!("rt: {added} BLAS meshes, {excl} excluded (skinned/blended/incompatible)");
        }
    }
}
