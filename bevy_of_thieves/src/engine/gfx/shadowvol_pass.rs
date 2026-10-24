//! LOOK: the PS2 shadow-volume count pass (FUN_0016da48 / VU1 overlay 188054619 / FUN_0016dd58) as a
//! real per-frame GPU pass (raster and RT, any receiver incl. Sly / guards), replacing the load-time
//! decal bake (`shadowvol::static_decals`, kept for the tests / `SLY2_LOOK_SHVOL_BAKE=1`).
//!
//! PS2 (see `shadowvol` + the L3 GS dump, draws 523..719): the green byte is cleared to 0x3e, every
//! volume face is drawn with ZTST GEQUAL (z-pass: faces in front of the visible surface) adding +-2 by
//! winding, kind-2 pass 2 (ZTST ALWAYS) adds the volumes that contain the eye, then the frame's green
//! byte indexes the FUN_001ea9f0 CLUT (lerp to colour 1 by the entry alpha).
//! Port: count texture (R16Float, additive blend, +-1 by `front_facing` x the volume's winding sign)
//! with the camera depth buffer, compare GreaterEqual (reverse-Z = PS2 GEQUAL); the ALWAYS pass of the
//! eye-containing volumes is a constant per frame (z-pass count = surface count - eye count for
//! closed meshes) and is added in the resolve as `eye` (CPU ray parity, `shadowvol::count`). Resolve:
//! fullscreen, before the display decode, `mix(Cd, clut1.rgb, count_alpha(count))`.
//! Kill switch `SLY2_LOOK_NOSHVOL=1`. Not ported: type-2 edge volumes (VU-extruded per frame from the
//! light: character / prop casters) and volumes on moving objects (the level volumes are static).

use super::shadowvol::Volume;
use bevy::core_pipeline::{schedule::Core3d, Core3dSystems, FullscreenShader};
use bevy::prelude::*;
use bevy::render::{
    extract_resource::{ExtractResource, ExtractResourcePlugin},
    render_resource::{
        binding_types::{texture_2d, texture_2d_multisampled, uniform_buffer_sized},
        *,
    },
    renderer::{RenderContext, RenderQueue, ViewQuery},
    view::{ExtractedView, ViewDepthStencilTexture, ViewTarget},
    RenderApp, RenderStartup,
};
use std::collections::HashMap;
use std::sync::Arc;

/// Level shadow volumes (main world, set at level spawn; extracted every frame, Arc-cheap).
#[derive(Resource, Clone, Default, ExtractResource)]
#[extract_app(RenderApp)]
pub struct ShadowVolumeScene {
    /// level-space (PS2 Z-up) volumes, for the eye count
    pub vols: Arc<Vec<Volume>>,
    /// Bevy-space triangle list (xyz, w = winding sign)
    pub verts: Arc<Vec<[f32; 4]>>,
    /// FUN_001ea9f0 colour 1 (display space 0..1) and its alpha a1 (0..255)
    pub rgb: [f32; 3],
    pub a1: f32,
    /// LOOK R24: FUN_001ea9f0 CLUT colours 0/1/2 (rgb display space, alpha 0..255; `shadowvol::resolve_clut`)
    pub clut: [[f32; 4]; 3],
    /// bumped when `verts` changes (vertex buffer upload)
    pub generation: u32,
}

impl ShadowVolumeScene {
    pub fn new(vols: Vec<Volume>, rgb: [f32; 3], a1: f32, generation: u32) -> Self {
        let mut verts = Vec::new();
        for v in &vols {
            for t in &v.tris {
                for p in t {
                    // level root = rotation_x(-90deg): (x, y, z) -> (x, z, -y)
                    verts.push([p[0], p[2], -p[1], v.sign as f32]);
                }
            }
        }
        Self { vols: Arc::new(vols), verts: Arc::new(verts), rgb, a1, clut: [[0.0, 0.0, 0.0, 128.0], [rgb[0], rgb[1], rgb[2], a1], [0.0, 0.0, 0.0, 160.0]], generation }
    }
}

pub struct ShadowVolumePassPlugin;
impl Plugin for ShadowVolumePassPlugin {
    fn build(&self, app: &mut App) {
        if std::env::var("SLY2_LOOK_NOSHVOL").is_ok() || std::env::var("SLY2_LOOK_SHVOL_BAKE").is_ok() {
            return;
        }
        app.init_resource::<ShadowVolumeScene>().add_plugins(ExtractResourcePlugin::<ShadowVolumeScene>::default());
        // LOOK: type-2 casters, kill switch SLY2_LOOK_NOCASTERS=1
        if std::env::var("SLY2_LOOK_NOCASTERS").is_err() {
            app.init_resource::<DynamicShadowVolumes>()
                .init_resource::<CastLights>()
                .add_plugins(ExtractResourcePlugin::<DynamicShadowVolumes>::default())
                .add_systems(PostUpdate, build_caster_volumes.after(bevy::transform::TransformSystems::Propagate));
        }
        let sh = {
            let mut shaders = app.world_mut().resource_mut::<Assets<Shader>>();
            ShvShaders(
                shaders.add(wgsl_part("//COUNT_BEGIN", "//COUNT_END", "")),
                [
                    shaders.add(wgsl_part("//RESOLVE_BEGIN", "//RESOLVE_END", "texture_2d<f32>")),
                    shaders.add(wgsl_part("//RESOLVE_BEGIN", "//RESOLVE_END", "texture_multisampled_2d<f32>")),
                ],
            )
        };
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else { return };
        render_app
            .insert_resource(sh)
            .add_systems(RenderStartup, init_pipeline)
            .add_systems(Core3d, shadow_volume_pass.in_set(Core3dSystems::EarlyPostProcess));
    }
}

#[derive(Resource)]
struct ShvPipeline {
    shader: Handle<Shader>,
    resolve_shader: [Handle<Shader>; 2],
    fullscreen: FullscreenShader,
    count_layout: BindGroupLayoutDescriptor,
    resolve_layout: [BindGroupLayoutDescriptor; 2],
}

/// plain WGSL (no preprocessor): the count / resolve halves of shadowvol.wgsl as separate modules
fn wgsl_part(begin: &str, end: &str, cnt: &str) -> Shader {
    let s = include_str!("shadowvol.wgsl");
    let (a, b) = (s.find(begin).unwrap(), s.find(end).unwrap());
    Shader::from_wgsl(s[a..b].replace("CNT_TYPE", cnt), format!("shadowvol.wgsl#{begin}{cnt}"))
}

#[derive(Resource, Clone)]
struct ShvShaders(Handle<Shader>, [Handle<Shader>; 2]);

fn init_pipeline(mut commands: Commands, shaders: Res<ShvShaders>, fullscreen: Res<FullscreenShader>) {
    let count_layout = BindGroupLayoutDescriptor::new(
        "shv_count_layout",
        &BindGroupLayoutEntries::single(ShaderStages::VERTEX, uniform_buffer_sized(false, None)),
    );
    let mk = |ms: bool| {
        let count = if ms { texture_2d_multisampled(TextureSampleType::Float { filterable: false }) } else { texture_2d(TextureSampleType::Float { filterable: false }) };
        BindGroupLayoutDescriptor::new(
            "shv_resolve_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (texture_2d(TextureSampleType::Float { filterable: false }), count, uniform_buffer_sized(false, None)),
            ),
        )
    };
    commands.insert_resource(ShvPipeline {
        shader: shaders.0.clone(),
        resolve_shader: shaders.1.clone(),
        fullscreen: fullscreen.clone(),
        count_layout,
        resolve_layout: [mk(false), mk(true)],
    });
}

/// per-pass GPU state cached by the system
#[derive(Default)]
struct ShvState {
    pipes: HashMap<(u32, TextureFormat), (CachedRenderPipelineId, CachedRenderPipelineId)>,
    vbuf: Option<(u32, Buffer, u32)>,
    count: Option<(UVec2, u32, Texture, TextureView)>,
    ubuf: Option<(Buffer, Buffer)>,
}

fn pipelines(p: &ShvPipeline, cache: &PipelineCache, samples: u32, format: TextureFormat) -> (CachedRenderPipelineId, CachedRenderPipelineId) {
    let count = cache.queue_render_pipeline(RenderPipelineDescriptor {
        label: Some("shv_count".into()),
        layout: vec![p.count_layout.clone()],
        vertex: VertexState {
            shader: p.shader.clone(),
            entry_point: Some("count_vs".into()),
            buffers: vec![bevy::mesh::VertexBufferLayout::from_vertex_formats(VertexStepMode::Vertex, [VertexFormat::Float32x4])],
            ..default()
        },
        primitive: PrimitiveState { cull_mode: None, ..default() },
        depth_stencil: Some(DepthStencilState {
            format: TextureFormat::Depth32Float,
            depth_write_enabled: Some(false),
            depth_compare: Some(CompareFunction::GreaterEqual),
            stencil: StencilState::default(),
            bias: DepthBiasState::default(),
        }),
        multisample: MultisampleState { count: samples, mask: !0, alpha_to_coverage_enabled: false },
        fragment: Some(FragmentState {
            shader: p.shader.clone(),
            entry_point: Some("count_fs".into()),
            targets: vec![Some(ColorTargetState {
                format: TextureFormat::R16Float,
                blend: Some(BlendState {
                    color: BlendComponent { src_factor: BlendFactor::One, dst_factor: BlendFactor::One, operation: BlendOperation::Add },
                    alpha: BlendComponent { src_factor: BlendFactor::One, dst_factor: BlendFactor::One, operation: BlendOperation::Add },
                }),
                write_mask: ColorWrites::RED,
            })],
            ..default()
        }),
        ..default()
    });
    let ms = samples > 1;
    let resolve = cache.queue_render_pipeline(RenderPipelineDescriptor {
        label: Some("shv_resolve".into()),
        layout: vec![p.resolve_layout[ms as usize].clone()],
        vertex: p.fullscreen.to_vertex_state(),
        fragment: Some(FragmentState {
            shader: p.resolve_shader[ms as usize].clone(),
            entry_point: Some("resolve_fs".into()),
            targets: vec![Some(ColorTargetState { format, blend: None, write_mask: ColorWrites::ALL })],
            ..default()
        }),
        ..default()
    });
    (count, resolve)
}

fn shadow_volume_pass(
    view: ViewQuery<(&ViewTarget, &ViewDepthStencilTexture, &ExtractedView)>,
    scene: Option<Res<ShadowVolumeScene>>,
    dynv: Option<Res<DynamicShadowVolumes>>,
    pipe: Option<Res<ShvPipeline>>,
    cache: Res<PipelineCache>,
    queue: Res<RenderQueue>,
    mut ctx: RenderContext,
    mut st: Local<ShvState>,
) {
    let (Some(scene), Some(pipe)) = (scene, pipe) else { return };
    let dynv = dynv.map(|d| d.verts.clone()).unwrap_or_default();
    if std::env::var("SLY2_LOOK_CASTDBG").is_ok() {
        info!("LOOK shv pass: static {} dyn {}", scene.verts.len(), dynv.len());
    }
    if scene.verts.is_empty() && dynv.is_empty() {
        return;
    }
    let (target, depth, ev) = view.into_inner();
    // only the main (perspective) game view
    if ev.clip_from_view.w_axis.w == 1.0 {
        return;
    }
    let samples = depth.texture().sample_count();
    let size = depth.texture().size();
    let size = UVec2::new(size.width, size.height);
    let ids = *st.pipes.entry((samples, ev.target_format)).or_insert_with(|| pipelines(&pipe, &cache, samples, ev.target_format));
    let (Some(count_pipe), Some(resolve_pipe)) = (cache.get_render_pipeline(ids.0), cache.get_render_pipeline(ids.1)) else { return };
    let device = ctx.render_device().clone();
    if st.vbuf.as_ref().map(|v| v.0) != Some(scene.generation) {
        let buf = device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("shv_vertices"),
            contents: &f32_bytes(if scene.verts.is_empty() { &[0.0; 4] } else { scene.verts.as_flattened() }),
            usage: BufferUsages::VERTEX,
        });
        st.vbuf = Some((scene.generation, buf, scene.verts.len() as u32));
    }
    if st.count.as_ref().is_none_or(|c| c.0 != size || c.1 != samples) {
        let tex = device.create_texture(&TextureDescriptor {
            label: Some("shv_count"),
            size: Extent3d { width: size.x, height: size.y, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: samples,
            dimension: TextureDimension::D2,
            format: TextureFormat::R16Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let tv = tex.create_view(&TextureViewDescriptor::default());
        st.count = Some((size, samples, tex, tv));
    }
    if st.ubuf.is_none() {
        let mk = |label, n: u64| {
            device.create_buffer(&BufferDescriptor { label: Some(label), size: n, usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST, mapped_at_creation: false })
        };
        st.ubuf = Some((mk("shv_view", 64), mk("shv_params", 64)));
    }
    // camera position in level space (inverse of the root rotation) -> eye count (the ALWAYS pass)
    let cam = ev.world_from_view.translation();
    let eye = super::shadowvol::count(&scene.vols, [cam.x, -cam.z, cam.y]);
    let clip_from_world = ev.clip_from_world.unwrap_or_else(|| ev.clip_from_view * ev.world_from_view.to_matrix().inverse());
    if std::env::var("SLY2_LOOK_CASTDBG").is_ok() {
        let pr = |v: &[f32; 4]| clip_from_world.project_point3(Vec3::new(v[0], v[1], v[2]));
        let inview = dynv.iter().filter(|v| { let c = pr(v); c.x.abs() < 1.0 && c.y.abs() < 1.0 }).count();
        let sinview = scene.verts.iter().filter(|v| { let c = pr(v); c.x.abs() < 1.0 && c.y.abs() < 1.0 }).count();
        info!("LOOK shv clip: dyn in view {inview}/{} static {sinview}/{} first {:?}", dynv.len(), scene.verts.len(), dynv.first().map(pr));
    }
    let (ubuf_view, ubuf_params) = {
        let u = st.ubuf.as_ref().unwrap();
        (u.0.clone(), u.1.clone())
    };
    queue.write_buffer(&ubuf_view, 0, &f32_bytes(&clip_from_world.to_cols_array()));
    let (c0, c2) = (scene.clut[0], scene.clut[2]);
    let params = [scene.rgb[0], scene.rgb[1], scene.rgb[2], scene.a1, eye as f32, std::env::var("SLY2_LOOK_SHV_DEBUG").is_ok() as u32 as f32, 0.0, 0.0, c0[0], c0[1], c0[2], c0[3], c2[0], c2[1], c2[2], c2[3]];
    queue.write_buffer(&ubuf_params, 0, &f32_bytes(&params));
    let dyn_buf = (!dynv.is_empty()).then(|| {
        device.create_buffer_with_data(&BufferInitDescriptor { label: Some("shv_casters"), contents: &f32_bytes(dynv.as_flattened()), usage: BufferUsages::VERTEX })
    });
    let (count_view, (vbuf, nv)) = {
        let c = st.count.as_ref().unwrap();
        let v = st.vbuf.as_ref().unwrap();
        (c.3.clone(), (v.1.clone(), v.2))
    };

    // 1: count pass (z-pass against the camera depth)
    let count_bg = device.create_bind_group("shv_count_bg", &cache.get_bind_group_layout(&pipe.count_layout), &BindGroupEntries::single(ubuf_view.as_entire_binding()));
    {
        let mut pass = ctx.command_encoder().begin_render_pass(&RenderPassDescriptor {
            label: Some("shv_count"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &count_view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations { load: LoadOp::Clear(Default::default()), store: StoreOp::Store },
            })],
            depth_stencil_attachment: Some(depth.get_attachment(StoreOp::Store)),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(count_pipe);
        pass.set_bind_group(0, &count_bg, &[]);
        pass.set_vertex_buffer(0, *vbuf.slice(..));
        if !scene.verts.is_empty() && std::env::var("SLY2_LOOK_SHV_DYNONLY").is_err() {
            pass.draw(0..nv, 0..1);
        }
        // LOOK: the per-frame type-2 caster volumes (`build_caster_volumes`)
        if let Some(b) = &dyn_buf {
            pass.set_vertex_buffer(0, *b.slice(..));
            pass.draw(0..dynv.len() as u32, 0..1);
        }
    }

    // 2: CLUT resolve
    let post = target.post_process_write();
    let ms = samples > 1;
    let resolve_bg = device.create_bind_group(
        "shv_resolve_bg",
        &cache.get_bind_group_layout(&pipe.resolve_layout[ms as usize]),
        &BindGroupEntries::sequential((post.source, &count_view, ubuf_params.as_entire_binding())),
    );
    let mut pass = ctx.command_encoder().begin_render_pass(&RenderPassDescriptor {
        label: Some("shv_resolve"),
        color_attachments: &[Some(RenderPassColorAttachment { view: post.destination, depth_slice: None, resolve_target: None, ops: Operations::default() })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
    pass.set_pipeline(resolve_pipe);
    pass.set_bind_group(0, &resolve_bg, &[]);
    pass.draw(0..3, 0..1);
}

fn f32_bytes(v: &[f32]) -> Vec<u8> {
    v.iter().flat_map(|f| f.to_le_bytes()).collect()
}

// ---------------------------------------------------------------------------------------------
// LOOK: dynamic casters (mesh type 2, FUN_0016caa0 edge lists: Sly / guards / props).
// PS2: per object FUN_001a4478 (called through FUN_001f3cc8 / FUN_001f3c10 with the object's light
// set +0x134 = {owner, &root position, .., +0x10, .., +0x1c cast radius, +0x20 cast length}) picks
// the 4 strongest lights at the root position (FUN_001a3f78), averages the positions of the ones with
// +0x158 != 0 weighted by their strength (directional: root - world row 0 * 1000 (DAT_002bfc90),
// point: the light's world position), pushes that light out to 2.5 (DAT_002bfc94) x the cast radius
// and uploads a light frame (VIF 0x6c05c00e: 2 axes from FUN_0011e2d8(dir), dir * far, the light
// position, near / far) with far = distance + cast length (+0x20 = object +0x13c, FUN_001f2108
// default 400 = DAT_002c3b90). VU1 overlay 188054619 0x0458 projects the edge-list vertices into that
// light frame, 0x05c0 keeps the edges whose two faces (a, b, c) / (c, b, d) face opposite ways in the
// light projection and emits the extruded quads, counted like the static volumes (FUN_0016da48).
// Port: the same light pick / push-out / far distance; silhouette edges against the light position,
// side quads extruded to the far plane, light-facing faces as the near cap and (LOOK R38, VU 0x0800) a
// fan to the far axis point lp + dir * far * 1.5 as the far cap (closed, so the z-pass count stays exact for a camera outside the volume). Cast radius =
// the caster's rest bound around its object origin (FUN_001f2500: sub-object bound + offset).
// Not ported: the eye count of caster volumes (camera inside a character's shadow volume), the field
// override of the cast length, the exact VU extrusion arithmetic (0x0638.. rows 18..22).

/// FUN_001f2108: object +0x13c default (DAT_002c3b90), the shadow cast length.
pub const CAST_LENGTH: f32 = 400.0;
/// DAT_002bfc90: directional light distance; DAT_002bfc94: push-out factor of the cast radius.
pub const DIR_LIGHT_DIST: f32 = 1000.0;
pub const CAST_PUSH: f32 = 2.5;

/// A type-2 caster prim (rigid: positions in the entity frame; skinned: Bevy skinning inputs).
#[derive(Component, Clone)]
pub struct ShadowCaster {
    pub pos: Vec<Vec3>,
    pub edges: Vec<[u32; 4]>,
    pub skin: Option<CasterSkin>,
    /// object entity whose origin is the FUN_001a4478 root position
    pub root: Entity,
    pub radius: f32,
    /// ANIM R71: (mesh record, local vertex -> record vertex) - the VU1 batches (strips) of one type-2
    /// record form ONE closed volume (each silhouette edge's fan is batch independent in the VU, so the
    /// original's count is the whole mesh's); a strip alone is open (stray light / dark quads).
    pub weld: Option<(usize, Vec<u32>)>,
}

/// ANIM R75: (world object, mesh record, instance) of a caster, for the JT hidden-record set
/// (playerdraw `hide_hidden_casters`, vis::JtVis::record_drawn).
#[derive(Component, Clone, Copy, Debug)]
pub struct CasterRecord {
    pub object: usize,
    pub record: usize,
    pub instance: Option<usize>,
}

#[derive(Clone)]
pub struct CasterSkin {
    pub joints: Vec<Entity>,
    pub ibp: Vec<Mat4>,
    pub ji: Vec<[u16; 4]>,
    pub jw: Vec<[f32; 4]>,
}

/// A light of the world list in Bevy space (`light::cast_light_set`).
#[derive(Clone, Copy, Debug)]
pub struct CastLight {
    pub kind: u32,
    /// kind 0: world row 0 (direction), kind 1: position
    pub vec: Vec3,
    pub du_fall: f32,
    pub ru_fall: f32,
    /// +0x158 != 0
    pub casts: bool,
}

#[derive(Resource, Clone, Default)]
pub struct CastLights(pub Arc<Vec<CastLight>>);

impl CastLights {
    pub fn from_world(w: &crate::engine::world::World) -> Self {
        let lv = |v: [f32; 3]| Vec3::new(v[0], v[2], -v[1]);
        Self(Arc::new(
            super::light::cast_light_set(w)
                .into_iter()
                .map(|(l, casts)| CastLight { kind: l.kind, vec: lv(l.d.vec), du_fall: l.d.du_fall, ru_fall: l.d.ru_fall, casts })
                .collect(),
        ))
    }
}

/// Per-frame caster volumes (Bevy space, w = winding sign), extracted to the count pass.
#[derive(Resource, Clone, Default, ExtractResource)]
#[extract_app(RenderApp)]
pub struct DynamicShadowVolumes {
    pub verts: Arc<Vec<[f32; 4]>>,
}

/// FUN_001a4478 shadow part: (light position, unit direction light -> root, far distance) for a
/// caster rooted at `p` with cast radius `radius`; None when no picked light casts.
pub fn ee_sub_1a4478_cast(lights: &[CastLight], p: Vec3, radius: f32, length: f32) -> Option<(Vec3, Vec3, f32)> {
    let mut v: Vec<(f32, CastLight)> =
        lights.iter().map(|l| (if l.kind == 0 { l.du_fall } else { l.du_fall + l.ru_fall * l.vec.distance(p) }, *l)).filter(|(s, _)| *s > 0.0).collect();
    v.sort_by(|a, b| b.0.total_cmp(&a.0));
    let (mut acc, mut sum) = (Vec3::ZERO, 0.0f32);
    for (s, l) in v.into_iter().take(4).filter(|(_, l)| l.casts) {
        acc += s * if l.kind == 0 { p - l.vec * DIR_LIGHT_DIST } else { l.vec };
        sum += s;
    }
    if sum == 0.0 {
        return None;
    }
    let mut lp = acc / sum;
    let diff = p - lp;
    let d = diff.length();
    let r = CAST_PUSH * radius;
    let near = if d < r {
        // d < 1: DAT_002a2dc0 = level +z (Bevy +y)
        let dir = if d < 1.0 { Vec3::Y } else { diff / d };
        lp = p - dir * r;
        r
    } else {
        d
    };
    Some((lp, (p - lp).normalize_or(Vec3::NEG_Y), near + length))
}

/// Closed shadow volume of one caster (world positions) from the light (`lp`, `dir`, `far`):
/// triangles appended to `out` with the volume's winding sign in w. Silhouette quads (VU1 0x05c0)
/// plus the light-facing faces as the near cap and a fan to the 1.5 x far axis point as the far cap (z-pass needs a
/// closed volume: open, the far end lightens what it covers). The near side is pushed `NEAR_PUSH`
/// away from the light so the caster's lit surface does not z-fight the cap under GEQUAL.
pub fn caster_volume(pos: &[Vec3], edges: &[[u32; 4]], lp: Vec3, dir: Vec3, far: f32, out: &mut Vec<[f32; 4]>) {
    const NEAR_PUSH: f32 = 2.0;
    // LOOK R38: VU1 overlay 188054619 0x0800 emits per silhouette edge the near axis point, the two
    // edge verts, their ray extrudes to the far plane (light projection x rows 19..22, depth 1) and
    // the far axis point vf7 = row22 + row21 * 1.5 (LOI 0x3fc00000; FUN_001a4478 auStack_d0 =
    // lp + dir * far * 1.5): the sides end at `far`, the far cap is a fan to that apex.
    const FAR_K: f32 = 1.5;
    let apex = lp + dir * far * FAR_K;
    let ext = |v: Vec3| {
        let t = (v - lp).dot(dir).max(1.0);
        lp + (v - lp) * (far / t).max(1.0)
    };
    let near = |v: Vec3| v + (v - lp).normalize_or_zero() * NEAR_PUSH;
    let get = |i: u32| pos.get(i as usize).copied();
    let lit = |a: Vec3, b: Vec3, c: Vec3| (b - a).cross(c - a).dot(lp - a) > 0.0;
    let mut tris: Vec<[Vec3; 3]> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for e in edges {
        let [Some(a), Some(b), Some(c), Some(d)] = e.map(get) else { continue };
        let (l1, l2) = (lit(a, b, c), lit(c, b, d));
        for (f, l, k) in [([a, b, c], l1, [e[0], e[1], e[2]]), ([c, b, d], l2, [e[2], e[1], e[3]])] {
            let mut key = k;
            key.sort();
            if l && key[0] != key[1] && key[1] != key[2] && seen.insert(key) {
                tris.push(f.map(near));
            }
        }
        if l1 != l2 {
            let (u, w) = if l1 { (b, c) } else { (c, b) };
            tris.push([near(w), near(u), ext(u)]);
            tris.push([near(w), ext(u), ext(w)]);
            tris.push([ext(w), ext(u), apex]);
        }
    }
    if tris.is_empty() {
        return;
    }
    let c = tris.iter().flatten().fold(Vec3::ZERO, |s, v| s + *v) / (tris.len() * 3) as f32;
    let vol: f32 = tris.iter().map(|t| (t[0] - c).dot((t[1] - c).cross(t[2] - c))).sum();
    if vol.abs() < 1e-3 {
        return;
    }
    let sign = vol.signum();
    for t in tris {
        for v in t {
            out.push([v.x, v.y, v.z, sign]);
        }
    }
}

fn build_caster_volumes(
    casters: Query<(&ShadowCaster, &GlobalTransform, &InheritedVisibility)>,
    xforms: Query<&GlobalTransform>,
    lights: Res<CastLights>,
    mut out: ResMut<DynamicShadowVolumes>,
) {
    let mut verts = Vec::new();
    // ANIM R71: strips of one record welded into one caster (SLY2_R71WELDOFF: per strip)
    let weld_on = std::env::var("SLY2_R71WELDOFF").is_err();
    let mut groups: std::collections::BTreeMap<(Entity, usize), (Vec<Vec3>, Vec<[u32; 4]>, f32)> = Default::default();
    let mut posed: Vec<(Entity, f32, Vec<Vec3>, Vec<[u32; 4]>)> = Vec::new();
    for (c, gt, vis) in &casters {
        if !vis.get() {
            continue;
        }
        if std::env::var("SLY2_R75CASTMAXR").ok().and_then(|v| v.parse::<f32>().ok()).is_some_and(|m| c.radius > m) {
            continue;
        }
        let Ok(root) = xforms.get(c.root) else { continue };
        if ee_sub_1a4478_cast(&lights.0, root.translation(), c.radius, CAST_LENGTH).is_none() {
            continue;
        }
        let pos: Vec<Vec3> = match &c.skin {
            Some(s) => {
                let mats: Vec<Mat4> = s.joints.iter().zip(&s.ibp).map(|(j, ibp)| xforms.get(*j).map(|g| g.to_matrix() * *ibp).unwrap_or(Mat4::IDENTITY)).collect();
                c.pos
                    .iter()
                    .enumerate()
                    .map(|(i, v)| {
                        let (ji, jw) = (s.ji.get(i).copied().unwrap_or_default(), s.jw.get(i).copied().unwrap_or([1.0, 0.0, 0.0, 0.0]));
                        (0..4).filter(|&k| jw[k] > 0.0).fold(Vec3::ZERO, |acc, k| acc + jw[k] * mats.get(ji[k] as usize).unwrap_or(&Mat4::IDENTITY).transform_point3(*v))
                    })
                    .collect()
            }
            None => c.pos.iter().map(|v| gt.transform_point(*v)).collect(),
        };
        match c.weld.as_ref().filter(|_| weld_on) {
            Some((rec, map)) => {
                let g = groups.entry((c.root, *rec)).or_default();
                for (k, v) in pos.iter().enumerate() {
                    let i = map.get(k).copied().unwrap_or(0) as usize;
                    if g.0.len() <= i {
                        g.0.resize(i + 1, Vec3::ZERO);
                    }
                    g.0[i] = *v;
                }
                g.1.extend(c.edges.iter().map(|e| e.map(|i| map.get(i as usize).copied().unwrap_or(0))));
                g.2 = g.2.max(c.radius);
            }
            None => posed.push((c.root, c.radius, pos, c.edges.clone())),
        }
    }
    posed.extend(groups.into_iter().map(|((root, _), (pos, edges, radius))| (root, radius, pos, edges)));
    for (root_e, radius, pos, edges) in posed {
        let Ok(root) = xforms.get(root_e) else { continue };
        let Some((lp, dir, far)) = ee_sub_1a4478_cast(&lights.0, root.translation(), radius, CAST_LENGTH) else { continue };
        let n0 = verts.len();
        caster_volume(&pos, &edges, lp, dir, far, &mut verts);
        if std::env::var("SLY2_LOOK_CASTDBG").is_ok() && verts.len() > n0 {
            let (mn, mx) = verts[n0..].iter().fold((Vec3::splat(1e9), Vec3::splat(-1e9)), |(a, b), v| (a.min(Vec3::new(v[0], v[1], v[2])), b.max(Vec3::new(v[0], v[1], v[2]))));
            info!("LOOK caster root {:?} r {:.0} lp {lp:?} far {far:.0} tris {} box {mn:?}..{mx:?} p0 {:?}", root.translation(), radius, (verts.len() - n0) / 3, pos.first());
        }
    }
    if std::env::var("SLY2_LOOK_CASTDBG").is_ok() {
        let n = casters.iter().count();
        let casting = lights.0.iter().filter(|l| l.casts).count();
        info!("LOOK casters: {n} casters, {} lights ({casting} casting), {} verts", lights.0.len(), verts.len());
    }
    if !(verts.is_empty() && out.verts.is_empty()) {
        out.verts = Arc::new(verts);
    }
}

#[cfg(test)]
mod caster_tests {
    use super::*;
    /// edge list (a, b, c, d) of a triangle mesh: faces (a, b, c) and (c, b, d) share b-c
    fn edges_of(t: &[[u32; 3]]) -> Vec<[u32; 4]> {
        let mut out = Vec::new();
        for (i, f) in t.iter().enumerate() {
            for k in 0..3 {
                let (a, b, c) = (f[k], f[(k + 1) % 3], f[(k + 2) % 3]);
                // face (a,b,c) rotated so the edge is b-c: need (x, b, c) -> edge b->c; other face has c->b
                for g in t.iter().skip(i + 1) {
                    for m in 0..3 {
                        if g[m] == c && g[(m + 1) % 3] == b {
                            out.push([a, b, c, g[(m + 2) % 3]]);
                        }
                    }
                }
            }
        }
        out
    }
    #[test]
    fn look_caster_cube_volume_closed() {
        let p: Vec<Vec3> = (0..8).map(|i| Vec3::new((i & 1) as f32, ((i >> 1) & 1) as f32, ((i >> 2) & 1) as f32) * 10.0).collect();
        let t = [[0, 2, 1], [1, 2, 3], [4, 5, 6], [5, 7, 6], [0, 1, 4], [1, 5, 4], [2, 6, 3], [3, 6, 7], [0, 4, 2], [2, 4, 6], [1, 3, 5], [3, 7, 5]];
        let e = edges_of(&t);
        let mut v = Vec::new();
        caster_volume(&p, &e, Vec3::new(5.0, 100.0, 5.0), Vec3::NEG_Y, 200.0, &mut v);
        eprintln!("edges {} verts {}", e.len(), v.len());
        assert_eq!(v.len(), 42, "2 near + 4 silhouette edges x (2 side + 1 apex fan) triangles");
        // z-pass count along eye -> p: +sign entering a front face, -sign a back face
        let zpass = |eye: Vec3, p: Vec3| -> f32 {
            let mut c = 0.0;
            for t in v.chunks_exact(3) {
                let [a, b, cc] = [0, 1, 2].map(|k| Vec3::new(t[k][0], t[k][1], t[k][2]));
                let (e1, e2, d) = (b - a, cc - a, p - eye);
                let h = d.cross(e2);
                let det = e1.dot(h);
                if det.abs() < 1e-9 { continue; }
                let sv = eye - a;
                let u = sv.dot(h) / det;
                let q = sv.cross(e1);
                let w = d.dot(q) / det;
                let tt = e2.dot(q) / det;
                if u >= 0.0 && w >= 0.0 && u + w <= 1.0 && tt > 0.0 && tt < 1.0 {
                    let n = e1.cross(e2);
                    c += if n.dot(d) < 0.0 { t[0][3] } else { -t[0][3] };
                }
            }
            c
        };
        let eye = Vec3::new(-100.0, -20.3, 5.1);
        assert_eq!(zpass(eye, Vec3::new(5.0, -20.0, 5.0)), 1.0, "point under the cube in shadow");
        assert_eq!(zpass(eye, Vec3::new(50.0, -20.0, 5.0)), 0.0);
    }
}
