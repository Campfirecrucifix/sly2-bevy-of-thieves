//! WP-A6: screen effects of the world effect pool (FUN_001ce888(world, type, 0): 0x3dd490 +0x4054 pool,
//! FUN_001ce6d0 picks the per-type vtable) and the effect instances of FUN_00156698, drawn as Bevy
//! billboards.
//!
//! 1:1 parts (pure data, tested headless):
//!  - type 4 "sparkle" (vtable 0x2d75f8): init FUN_001d0108 (FUN_001ce920 base: +0x14 start = now, +0x18
//!    life 1.0; then life 0.5, +0x30 alpha curve DAT_002c1f90 {0, 4, -4}, +0x20 size curve DAT_002c1fa0
//!    {0.25, 3, -3}); the K8 caller (FUN_001c6308) then sets +0xf0 target, +0x1c drawn object (the
//!    pickup's sparkle child), +0x18 life = 1.5 / max(|vz|, 1). Update FUN_001d0158: position +0x70 =
//!    target position (FUN_001a62e8). FUN_001cf268: freed once `now - start > life`. Draw FUN_001d0190:
//!    d = |camera - pos| * 0.001, u = (now - start) / life,
//!    size = (0.25 + u (3 - 3u)) * clamp(0 + d (0.75 + 0 d), 1, 5),
//!    alpha = clamp(0 + u (4 - 4u), 0, 1) * clamp(0.5 + d (0 + 0.5 d), 0.5, 1),
//!    roll about the view axis = 0 + u (1 + 0 u) rad (DAT_002c1ff0; FUN_0011b500), then FUN_001a5ee8 draws
//!    the +0x1c object with that billboard matrix and alpha.
//!  - type 9 "collect" (vtable 0x2d75a8, init FUN_001d04e8): life DAT_002c2000 = 0.5 s; the descriptor
//!    (+0xf0: coin DAT_002c0da0, health DAT_002c0e10) starts with the HUD target in screen units ((0.83,
//!    0.75) coin counter, (-0.7, 0.72) health) and ends with the arrival callback (FUN_001c67a0 /
//!    FUN_001c6ab0). The screen-space path FUN_00117120 is not ported: REPLACED by a straight flight in
//!    view space from the pickup to the HUD target over the life (same start, end and duration).
//!  - FUN_00156698(desc, pos, obj) (hit / impact effect instances, vtable 0x2ce2b8, pool 0x3e1498): the
//!    descriptors are runtime level assets (DAT_003e14b0 / b8 / bc) and the particle VU code is not
//!    ported: REPLACED by a short additive star burst ([`HIT_LIFE`]).
//!
//! Drawing (feature `game`, [`FxPlugin`]): one unlit, additive, shadowless camera-facing quad per effect
//! with a procedural star texture (the sparkle child mesh is drawn by its own entity elsewhere; the
//! effect only needs the flare). Solari-safe: no `RaytracingMesh3d` (stays out of the BLAS), unlit
//! (no G-buffer lighting input), `AlphaMode::Add` draws in the transparent pass after the RT lighting.

/// Sparkle life default (FUN_001d0108: +0x18 = 0.5).
pub const SPARKLE_LIFE: f32 = 0.5;
/// DAT_002c1fa0: size curve {c0, c1, c2} in u.
pub const SPARKLE_SIZE: [f32; 3] = [0.25, 3.0, -3.0];
/// DAT_002c1f90: alpha curve in u (clamped by DAT_002bbb60 / 64 = [0, 1]).
pub const SPARKLE_ALPHA: [f32; 3] = [0.0, 4.0, -4.0];
/// DAT_002c1fb0 / fc0: distance size factor curve and clamp.
pub const DIST_SIZE: [f32; 3] = [0.0, 0.75, 0.0];
pub const DIST_SIZE_CLAMP: [f32; 2] = [1.0, 5.0];
/// DAT_002c1fd0 / fe0: distance alpha factor curve and clamp.
pub const DIST_ALPHA: [f32; 3] = [0.5, 0.0, 0.5];
pub const DIST_ALPHA_CLAMP: [f32; 2] = [0.5, 1.0];
/// DAT_002c1ff0: roll curve (radians) in u.
pub const SPARKLE_ROLL: [f32; 3] = [0.0, 1.0, 0.0];
/// DAT_002c2000: collect effect life.
pub const COLLECT_LIFE: f32 = 0.5;
/// REPLACED hit burst life (seconds).
pub const HIT_LIFE: f32 = 0.35;
/// A9: hit star pulled this far (level units) from the struck body towards the camera.
pub const HIT_PULL: f32 = 120.0;
/// Descriptors of the K8 collect effects (FUN_001c65f8 coin / FUN_001c68f0 health): HUD target (screen
/// units, x right, y up, [-1, 1]).
pub const DESC_COIN: u32 = 0x002c_0da0;
pub const DESC_HEALTH: u32 = 0x002c_0e10;
pub fn desc_hud_target(desc: u32) -> Option<[f32; 2]> {
    match desc {
        DESC_COIN => Some([0.83, 0.75]),
        DESC_HEALTH => Some([-0.7, 0.72]),
        _ => None,
    }
}

fn poly(c: [f32; 3], x: f32) -> f32 {
    c[0] + x * (c[1] + x * c[2])
}

/// Kind of an effect request.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FxKind {
    /// FUN_001ce888(world, 4, 0): follows `target` (object index) when given
    Sparkle { target: Option<usize> },
    /// FUN_001ce888(world, 9, 0) with a descriptor (DESC_COIN / DESC_HEALTH); A7: `obj` = the drawn object
    /// (+0x1c, the pickup itself), `vel` = the FUN_001d04e8 start velocity (+0x90, drawn by the producer
    /// from the game rng)
    Collect { desc: u32, obj: Option<usize>, vel: [f32; 3] },
    /// FUN_00156698 effect instance (REPLACED burst)
    Hit,
}

/// One effect request (level space position, z up).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FxRequest {
    pub kind: FxKind,
    pub pos: [f32; 3],
    /// life in seconds (FUN_001ce920 +0x18; sparkle callers override it)
    pub life: f32,
}

impl FxRequest {
    pub fn sparkle(pos: [f32; 3], target: Option<usize>, life: f32) -> Self {
        Self { kind: FxKind::Sparkle { target }, pos, life }
    }
    pub fn collect(desc: u32, pos: [f32; 3]) -> Self {
        Self { kind: FxKind::Collect { desc, obj: None, vel: [0.0; 3] }, pos, life: COLLECT_LIFE }
    }
    /// A7: collect effect drawing object `obj` with start velocity `vel` (FUN_001c65f8 / FUN_001c68f0).
    pub fn collect_obj(desc: u32, pos: [f32; 3], obj: usize, vel: [f32; 3]) -> Self {
        Self { kind: FxKind::Collect { desc, obj: Some(obj), vel }, pos, life: COLLECT_LIFE }
    }
    pub fn hit(pos: [f32; 3]) -> Self {
        Self { kind: FxKind::Hit, pos, life: HIT_LIFE }
    }
}

/// FUN_001cf268: freed once the age exceeds the life.
pub fn ee_sub_1cf268_expired(age: f32, life: f32) -> bool {
    life < age
}

/// FUN_001d0190 billboard parameters: (size, alpha, roll) at age `age` of `life` seconds and camera
/// distance `dist` (level units).
pub fn ee_sub_1d0190(age: f32, life: f32, dist: f32) -> (f32, f32, f32) {
    let d = dist * 0.001;
    let u = age / life;
    let fs = poly(DIST_SIZE, d).clamp(DIST_SIZE_CLAMP[0], DIST_SIZE_CLAMP[1]);
    let size = poly(SPARKLE_SIZE, u) * fs;
    let a = poly(SPARKLE_ALPHA, u).clamp(0.0, 1.0);
    let fa = poly(DIST_ALPHA, d).clamp(DIST_ALPHA_CLAMP[0], DIST_ALPHA_CLAMP[1]);
    (size, a * fa, poly(SPARKLE_ROLL, u))
}

/// FUN_001c6308's life of the sparkle: 1.5 / max(|vz|, 1).
pub fn sparkle_life(vz: f32) -> f32 {
    let v = vz.abs();
    1.5 / if v < 1.0 { 1.0 } else { v }
}

/// Billboard parameters of any effect: (size in level units, alpha, roll, colour rgb).
pub fn fx_params(kind: FxKind, age: f32, life: f32, dist: f32) -> (f32, f32, f32, [f32; 3]) {
    match kind {
        FxKind::Sparkle { .. } => {
            let (s, a, r) = ee_sub_1d0190(age, life, dist);
            // the sparkle child is drawn at its own size in object units (pickups are ~20-40 units)
            (s * 24.0, a, r, [1.0, 0.95, 0.7])
        }
        FxKind::Collect { desc, .. } => {
            let u = (age / life).clamp(0.0, 1.0);
            let col = if desc == DESC_HEALTH { [1.0, 0.45, 0.45] } else { [1.0, 0.85, 0.35] };
            (40.0 * (1.0 - 0.6 * u), 1.0 - u * u, u * 3.0, col)
        }
        FxKind::Hit => {
            let u = (age / life).clamp(0.0, 1.0);
            (30.0 + 60.0 * u, 1.0 - u, u, [1.0, 1.0, 0.9])
        }
    }
}

/// REPLACED collect flight: view-space point at fraction `u` from the effect's start (view space, -z
/// forward) to the HUD target at the same view depth (`half` = tan(fov_y / 2), `aspect` = w / h).
pub fn collect_view_pos(start: [f32; 3], hud: [f32; 2], u: f32, half: f32, aspect: f32) -> [f32; 3] {
    let depth = (-start[2]).max(1.0);
    let end = [hud[0] * half * aspect * depth, hud[1] * half * depth, start[2]];
    let u = u.clamp(0.0, 1.0);
    std::array::from_fn(|k| start[k] + (end[k] - start[k]) * u)
}

// ---------------------------------------------------------------- A7: type 9 collect, 1:1

/// 3x3 rotation as columns (= the original's memory rows; = `Affine` rows 0..3).
pub type M3 = [[f32; 3]; 3];

/// Collect descriptor (0x70 bytes at DAT_002c0da0 coin / DAT_002c0e10 health; SCUS_973.16 .data).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CollectDesc {
    /// +0x00: Hermite end point in screen units (x right, y up, [-1, 1]) and camera depth
    pub target: [f32; 3],
    /// +0x10: Hermite end velocity (screen units / s)
    pub target_vel: [f32; 3],
    /// +0x20 / +0x30: start velocity range (FUN_0011d448 per component)
    pub vel_min: [f32; 3],
    pub vel_max: [f32; 3],
    /// +0x40: spin axis (FUN_001d0630 second FUN_0011b500)
    pub spin_axis: [f32; 3],
    /// +0x50: size curve, copied to the effect's +0x20 by FUN_001d04e8
    pub size: [f32; 3],
    /// +0x64: arrival callback (run by FUN_001d0800 at expiry)
    pub arrive: u32,
}
pub const COLLECT_COIN: CollectDesc = CollectDesc {
    target: [0.83, 0.75, 100.0],
    target_vel: [1.0, -1.0, 0.0],
    vel_min: [-1.0, 2.0, 0.0],
    vel_max: [1.0, 3.0, 0.0],
    spin_axis: [0.0, 0.0, 1.0],
    size: [1.0, -0.9, 0.0],
    arrive: 0x001c_67a0,
};
pub const COLLECT_HEALTH: CollectDesc = CollectDesc {
    target: [-0.7, 0.72, 100.0],
    target_vel: [1.0, -1.0, 0.0],
    vel_min: [-1.0, 2.0, 0.0],
    vel_max: [1.0, 3.0, 0.0],
    spin_axis: [0.0, 0.0, 1.0],
    size: [1.0, -0.8, 0.0],
    arrive: 0x001c_6ab0,
};
pub fn collect_desc(desc: u32) -> Option<&'static CollectDesc> {
    match desc {
        DESC_COIN => Some(&COLLECT_COIN),
        DESC_HEALTH => Some(&COLLECT_HEALTH),
        _ => None,
    }
}
/// DAT_002bbb50: alpha curve of the collect effect (FUN_001d04e8 +0x30) = constant 1.
pub const COLLECT_ALPHA: [f32; 3] = [1.0, 0.0, 0.0];

/// The game camera as FUN_0014f308 / FUN_0014f268 read it (0x2ddf40): +0x60 orientation columns (x forward,
/// y left, z up), +0x90 position, +0x40 / +0x44 screen scales (tan of the half fov, horizontal / vertical).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CamFrame {
    pub pos: [f32; 3],
    pub rot: M3,
    pub x_scale: f32,
    pub y_scale: f32,
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// FUN_0014f308: level point -> (screen x, screen y, depth) (same as `camera::cm::FUN_0014f308`).
pub fn ee_sub_14f308(cam: &CamFrame, p: [f32; 3]) -> [f32; 3] {
    let d = std::array::from_fn::<f32, 3, _>(|k| p[k] - cam.pos[k]);
    let l = [dot3(cam.rot[0], d), dot3(cam.rot[1], d), dot3(cam.rot[2], d)];
    let (mut x, mut y) = (-l[1] / cam.x_scale, l[2] / cam.y_scale);
    if 0.0001 < l[0] {
        x /= l[0];
        y /= l[0];
    }
    [x, y, l[0]]
}

/// FUN_0014f268: (screen x, screen y, depth) -> level point (same as `camera::cm::FUN_0014f268`).
pub fn ee_sub_14f268(cam: &CamFrame, s: [f32; 3]) -> [f32; 3] {
    let d = s[2];
    let (mut y, mut z) = (-s[0], s[1]);
    if 0.0001 < d {
        y *= d;
        z *= d;
    }
    let l = [d, y * cam.x_scale, z * cam.y_scale];
    std::array::from_fn(|k| cam.pos[k] + cam.rot[0][k] * l[0] + cam.rot[1][k] * l[1] + cam.rot[2][k] * l[2])
}

/// Type 9 state: +0x100 screen point (x, y, depth), +0x90 its velocity.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CollectState {
    pub screen: [f32; 3],
    pub vel: [f32; 3],
}

/// FUN_001d04e8 (type 9 init after FUN_001ce920): +0x100 = FUN_0014f308(camera, pos), +0x90 = per component
/// FUN_0011d448(desc +0x20[k], desc +0x30[k]) (`rand(lo, hi)`; equal bounds draw nothing), life +0x18 =
/// DAT_002c2000, +0x30 = DAT_002bbb50, +0x20 = desc +0x50.
pub fn ee_sub_1d04e8(desc: &CollectDesc, cam: &CamFrame, pos: [f32; 3], mut rand: impl FnMut(f32, f32) -> f32) -> CollectState {
    let screen = ee_sub_14f308(cam, pos);
    let vel = [rand(desc.vel_min[0], desc.vel_max[0]), rand(desc.vel_min[1], desc.vel_max[1]), rand(desc.vel_min[2], desc.vel_max[2])];
    CollectState { screen, vel }
}

/// FUN_001d05a0 (type 9 update): one Hermite step FUN_00117120(max(start + life - now, dt), dt, 1.0, +0x100,
/// +0x90, desc +0x00, desc +0x10) -> new +0x100 / +0x90; the level position +0x70 = FUN_0014f268(camera,
/// +0x100) (previous one to +0x80). Returns the new level position.
pub fn ee_sub_1d05a0(st: &mut CollectState, desc: &CollectDesc, cam: &CamFrame, remaining: f32, dt: f32) -> [f32; 3] {
    let dur = if dt < remaining { remaining } else { dt };
    let (p, v, _) = crate::engine::anim::keys::ee_sub_117120(dur, dt, 1.0, st.screen, st.vel, desc.target, desc.target_vel);
    st.screen = p;
    st.vel = v;
    ee_sub_14f268(cam, st.screen)
}

fn m3_to_m4(m: &M3) -> crate::engine::camera::math::M4 {
    [[m[0][0], m[0][1], m[0][2], 0.0], [m[1][0], m[1][1], m[1][2], 0.0], [m[2][0], m[2][1], m[2][2], 0.0], [0.0, 0.0, 0.0, 1.0]]
}
fn m4_to_m3(m: &crate::engine::camera::math::M4) -> M3 {
    std::array::from_fn(|c| [m[c][0], m[c][1], m[c][2]])
}
/// Column product a * b (b applied first).
pub fn m3_mul(a: &M3, b: &M3) -> M3 {
    std::array::from_fn(|j| std::array::from_fn(|i| a[0][i] * b[j][0] + a[1][i] * b[j][1] + a[2][i] * b[j][2]))
}
fn m3_t(a: &M3) -> M3 {
    std::array::from_fn(|j| std::array::from_fn(|i| a[i][j]))
}

/// FUN_001d0630 orientation: rel = FUN_0011bc00(object +0x40, camera +0x60) = cam * obj^T, (angle, axis) =
/// FUN_0011bcf8(rel); A = FUN_0011b500(angle * u, axis), S = FUN_0011b500((u + u) * 3.1415927, desc +0x40);
/// result A * S * obj (u = (now - start) / life): the object's rotation at u = 0, the camera's at u = 1,
/// one full turn about the spin axis on the way.
pub fn ee_sub_1d0630_rot(obj: &M3, cam: &M3, u: f32, spin_axis: [f32; 3]) -> M3 {
    use crate::engine::camera::math::{FUN_0011b500, FUN_0011bcf8};
    let rel = m3_mul(cam, &m3_t(obj));
    let (ang, axis) = FUN_0011bcf8(&m3_to_m4(&rel));
    let a = m4_to_m3(&FUN_0011b500(ang * u, axis));
    let s = m4_to_m3(&FUN_0011b500((u + u) * 3.1415927, spin_axis));
    m3_mul(&m3_mul(&a, &s), obj)
}

/// FUN_001cf338 scale and alpha of the drawn object: size = +0x20 curve (desc +0x50) in u, alpha = clamp(+0x30
/// curve, 0, 1) (the FUN_0014f898 distance fade factor is not applied).
pub fn collect_size_alpha(desc: &CollectDesc, u: f32) -> (f32, f32) {
    (poly(desc.size, u), poly(COLLECT_ALPHA, u).clamp(0.0, 1.0))
}

#[cfg(feature = "game")]
pub use plugin::*;

#[cfg(feature = "game")]
mod plugin {
    use super::*;
    use crate::engine::camera::bevy_sync::CmCamera;
    use crate::engine::gfx::game::{InkOutline, LevelGeometryRoot};
    use bevy::asset::RenderAssetUsages;
    use bevy::mesh::skinning::SkinnedMesh;
    use bevy::light::NotShadowCaster;
    use bevy::prelude::*;
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

    /// Effect requests of this frame (level space). Producers: npcdraw (K8 collect, guard stun hook),
    /// classes / player hosts (FUN_001ce888 / FUN_00156698 hooks) via [`FxQueue::push`].
    #[derive(Resource, Default)]
    pub struct FxQueue {
        pub pending: Vec<FxRequest>,
        /// requests spawned so far (diagnostics / tests)
        pub spawned: usize,
    }
    impl FxQueue {
        pub fn push(&mut self, r: FxRequest) {
            self.pending.push(r);
        }
    }

    /// Positions of objects the sparkles follow (FUN_001d0158 target), refreshed by the producers.
    #[derive(Resource, Default)]
    pub struct FxTargets(pub std::collections::HashMap<usize, [f32; 3]>);

    /// A live effect billboard.
    #[derive(Component)]
    pub struct FxLive {
        pub req: FxRequest,
        pub age: f32,
        /// collect without a drawn object: view-space start (set on the first frame)
        pub view0: Option<[f32; 3]>,
        /// A7 collect: FUN_001d04e8 state (set on the first frame, needs the camera)
        pub collect: Option<CollectState>,
        /// A7 collect: rotation of the drawn object when collected (FUN_001a6318 -> +0x40); Some = this entity
        /// is a copy of the object's subtree (FUN_001cf338 -> FUN_001a5ee8 draws the object itself)
        pub obj_rot: Option<M3>,
    }

    /// Drawn entity (level-space root) of each object a collect effect may draw (+0x1c), refreshed by the
    /// producers (npcdraw: the K8 pickups).
    #[derive(Resource, Default)]
    pub struct FxModels(pub std::collections::HashMap<usize, Entity>);

    #[derive(Resource)]
    struct FxAssets {
        quad: Handle<Mesh>,
        tex: Handle<Image>,
        /// A10: billboard materials created at startup and recycled (a fresh StandardMaterial asset is only
        /// drawn 4-8 frames after its spawn: asset prepare + Add pipeline specialisation)
        free: Vec<Handle<StandardMaterial>>,
        warm: Option<Entity>,
    }

    /// A10: billboard materials pre-created at startup (grown on demand).
    pub const FX_MAT_POOL: usize = 48;

    /// A10: the billboard material (additive, unlit, white star texture), fully transparent until driven.
    pub fn fx_billboard_material(tex: Handle<Image>) -> StandardMaterial {
        StandardMaterial {
            base_color: Color::srgba(0.0, 0.0, 0.0, 0.0),
            base_color_texture: Some(tex),
            unlit: true,
            alpha_mode: AlphaMode::Add,
            double_sided: true,
            cull_mode: None,
            fog_enabled: false,
            ..default()
        }
    }

    /// A10: marker of the always-drawn (transparent) warm-up billboard in front of the camera that keeps the
    /// effect pipeline, texture and quad prepared, so a spawned effect draws on its first frame.
    #[derive(Component)]
    pub struct FxWarm;

    pub struct FxPlugin;
    impl Plugin for FxPlugin {
        fn build(&self, app: &mut App) {
            app.init_resource::<FxQueue>().init_resource::<FxTargets>().init_resource::<FxModels>().add_systems(Startup, setup_fx).add_systems(PostUpdate, (warm_fx, spawn_fx, drive_fx).chain().before(TransformSystems::Propagate));
        }
    }

    /// 4-point star flare (procedural, 64x64 RGBA, white with alpha falloff).
    pub fn star_texture() -> Image {
        const N: usize = 64;
        let mut px = vec![0u8; N * N * 4];
        for y in 0..N {
            for x in 0..N {
                let (fx, fy) = ((x as f32 + 0.5) / N as f32 * 2.0 - 1.0, (y as f32 + 0.5) / N as f32 * 2.0 - 1.0);
                let r = (fx * fx + fy * fy).sqrt();
                let core = (1.0 - r * 2.2).max(0.0);
                let rays = ((1.0 - fx.abs()) * (1.0 - fy.abs() * 12.0).max(0.0)).max((1.0 - fy.abs()) * (1.0 - fx.abs() * 12.0).max(0.0));
                let a = (core * core + rays * rays).min(1.0) * (1.0 - r).max(0.0);
                let i = (y * N + x) * 4;
                px[i..i + 4].copy_from_slice(&[255, 255, 255, (a * 255.0) as u8]);
            }
        }
        Image::new(Extent3d { width: N as u32, height: N as u32, depth_or_array_layers: 1 }, TextureDimension::D2, px, TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::RENDER_WORLD)
    }

    fn setup_fx(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut images: ResMut<Assets<Image>>, mut mats: ResMut<Assets<StandardMaterial>>) {
        let quad = meshes.add(Rectangle::new(1.0, 1.0));
        let tex = images.add(star_texture());
        let free = (0..FX_MAT_POOL).map(|_| mats.add(fx_billboard_material(tex.clone()))).collect();
        commands.insert_resource(FxAssets { quad, tex, free, warm: None });
    }

    /// A10: once a game camera exists, hang the transparent warm-up billboard 10 units in front of it
    /// (adds 0: Add with a zero colour). NoFrustumCulling keeps it queued every frame.
    fn warm_fx(mut commands: Commands, assets: Option<ResMut<FxAssets>>, cam: Query<Entity, With<CmCamera>>, mut mats: ResMut<Assets<StandardMaterial>>) {
        let Some(mut a) = assets else { return };
        if a.warm.is_some_and(|e| commands.get_entity(e).is_ok()) {
            return;
        }
        let Some(c) = cam.iter().next() else { return };
        let mat = mats.add(fx_billboard_material(a.tex.clone()));
        let e = commands
            .spawn((Mesh3d(a.quad.clone()), MeshMaterial3d(mat), Transform::from_xyz(0.0, 0.0, -10.0), NotShadowCaster, bevy::camera::visibility::NoFrustumCulling, FxWarm, ChildOf(c)))
            .id();
        a.warm = Some(e);
    }

    #[allow(clippy::type_complexity, clippy::too_many_arguments)]
    fn spawn_fx(
        mut commands: Commands,
        mut q: ResMut<FxQueue>,
        assets: Option<ResMut<FxAssets>>,
        roots: Query<Entity, With<LevelGeometryRoot>>,
        mut mats: ResMut<Assets<StandardMaterial>>,
        models: Res<FxModels>,
        kids: Query<&Children>,
        parts: Query<(&Transform, Option<&Mesh3d>, Option<&MeshMaterial3d<StandardMaterial>>, Option<&InkOutline>, Option<&SkinnedMesh>)>,
        mut mesh_assets: ResMut<Assets<Mesh>>,
    ) {
        let (Some(mut a), Some(root)) = (assets, roots.iter().next()) else {
            q.pending.clear();
            return;
        };
        for r in std::mem::take(&mut q.pending) {
            // A7: FUN_001cf338 draws the collected object itself (+0x1c) -> a copy of its drawn subtree
            if let FxKind::Collect { obj: Some(o), .. } = r.kind {
                if let Some((&src, Ok((t, ..)))) = models.0.get(&o).map(|e| (e, parts.get(*e))) {
                    let m = t.to_matrix();
                    let (x, y, z) = (m.x_axis.truncate().normalize_or_zero(), m.y_axis.truncate().normalize_or_zero(), m.z_axis.truncate().normalize_or_zero());
                    let rot: M3 = [x.to_array(), y.to_array(), z.to_array()];
                    let aff = [rot[0], rot[1], rot[2], r.pos];
                    let e = crate::engine::npcdraw::objects::copy_subtree(&mut commands, root, &aff, src, &kids, &parts, &mut mesh_assets);
                    commands.entity(e).insert((FxLive { req: r, age: 0.0, view0: None, collect: None, obj_rot: Some(rot) }, Visibility::Inherited));
                    q.spawned += 1;
                    continue;
                }
            }
            // A10: a prepared pool material (fresh asset only when the pool is exhausted)
            let mat = match a.free.pop() {
                Some(m) => m,
                None => mats.add(fx_billboard_material(a.tex.clone())),
            };
            let t = Transform::from_translation(Vec3::from_array(r.pos)).with_scale(Vec3::ZERO);
            commands.spawn((Mesh3d(a.quad.clone()), MeshMaterial3d(mat), t, NotShadowCaster, FxLive { req: r, age: 0.0, view0: None, collect: None, obj_rot: None }, ChildOf(root)));
            q.spawned += 1;
        }
    }

    #[allow(clippy::type_complexity)]
    fn drive_fx(
        mut commands: Commands,
        time: Res<Time>,
        targets: Res<FxTargets>,
        cam: Query<(&GlobalTransform, &Projection), With<CmCamera>>,
        roots: Query<&GlobalTransform, With<LevelGeometryRoot>>,
        mut fx: Query<(Entity, &mut FxLive, &mut Transform, Option<&MeshMaterial3d<StandardMaterial>>)>,
        mut mats: ResMut<Assets<StandardMaterial>>,
        assets: Option<ResMut<FxAssets>>,
    ) {
        let mut assets = assets;
        let dt = time.delta_secs().min(1.0 / 15.0);
        let Some((cg, proj)) = cam.iter().next() else { return };
        let root = roots.iter().next().copied().unwrap_or_default();
        let to_level = root.affine().inverse();
        let (half, aspect) = match proj {
            Projection::Perspective(p) => ((p.fov * 0.5).tan(), p.aspect_ratio),
            _ => (0.5, 16.0 / 9.0),
        };
        let cam_level = to_level.transform_point3(cg.translation());
        // camera axes in level space (billboard facing)
        let right = to_level.transform_vector3(cg.right().as_vec3()).normalize_or_zero();
        let up = to_level.transform_vector3(cg.up().as_vec3()).normalize_or_zero();
        let back = to_level.transform_vector3(cg.back().as_vec3()).normalize_or_zero();
        let view = cg.affine().inverse();
        // the game camera frame in level space (x forward, y left, z up; FUN_0014f308 / FUN_0014f268)
        let camf = CamFrame {
            pos: cam_level.to_array(),
            rot: [(-back).to_array(), (-right).to_array(), up.to_array()],
            x_scale: half * aspect,
            y_scale: half,
        };
        for (e, mut l, mut t, m) in &mut fx {
            l.age += dt;
            if ee_sub_1cf268_expired(l.age, l.req.life) {
                // A10: billboard material back to the pool (transparent until reused)
                if let (Some(m), Some(a), None) = (m, assets.as_deref_mut(), l.obj_rot) {
                    if let Some(mut mat) = mats.get_mut(&m.0) {
                        mat.base_color = Color::srgba(0.0, 0.0, 0.0, 0.0);
                    }
                    a.free.push(m.0.clone());
                }
                commands.entity(e).despawn();
                continue;
            }
            let mut pos = Vec3::from_array(l.req.pos);
            match l.req.kind {
                FxKind::Sparkle { target: Some(o) } => {
                    if let Some(p) = targets.0.get(&o) {
                        pos = Vec3::from_array(*p);
                    }
                }
                FxKind::Collect { desc, obj, vel } if obj.is_some() || collect_desc(desc).is_some() => {
                    // A7 1:1: FUN_001d04e8 init, FUN_001d05a0 screen-space Hermite step, FUN_0014f268 back to level
                    let Some(d) = collect_desc(desc) else { continue };
                    let (remaining, u, obj_rot) = (l.req.life - l.age, l.age / l.req.life, l.obj_rot);
                    let st = l.collect.get_or_insert_with(|| {
                        let mut k = 0;
                        ee_sub_1d04e8(d, &camf, pos.to_array(), |_, _| {
                            k += 1;
                            vel[k - 1]
                        })
                    });
                    pos = Vec3::from_array(ee_sub_1d05a0(st, d, &camf, remaining, dt));
                    if let Some(rot) = obj_rot {
                        let f = ee_sub_1d0630_rot(&rot, &camf.rot, u, d.spin_axis);
                        let (size, _alpha) = collect_size_alpha(d, u);
                        let c = |v: [f32; 3]| Vec3::from_array(v) * size;
                        *t = Transform::from_matrix(Mat4::from_cols(c(f[0]).extend(0.0), c(f[1]).extend(0.0), c(f[2]).extend(0.0), pos.extend(1.0)));
                        continue;
                    }
                }
                FxKind::Collect { desc, .. } => {
                    let world = root.transform_point(pos);
                    let v0 = *l.view0.get_or_insert(view.transform_point3(world).to_array());
                    let hud = desc_hud_target(desc).unwrap_or([0.0, 0.0]);
                    let vp = collect_view_pos(v0, hud, l.age / l.req.life, half, aspect);
                    pos = to_level.transform_point3(cg.affine().transform_point3(Vec3::from_array(vp)));
                }
                // A9: the star is drawn in front of the struck body (the cane contact side), not inside it
                FxKind::Hit => pos += (cam_level - pos).normalize_or_zero() * HIT_PULL,
                _ => {}
            }
            let (size, alpha, roll, col) = fx_params(l.req.kind, l.age, l.req.life, (cam_level - pos).length());
            let (s, c) = roll.sin_cos();
            let (r2, u2) = (right * c + up * s, up * c - right * s);
            t.translation = pos;
            t.rotation = Quat::from_mat3(&Mat3::from_cols(r2, u2, back));
            t.scale = Vec3::splat(size.max(0.0));
            if let Some(mut mat) = m.and_then(|m| mats.get_mut(&m.0)) {
                mat.base_color = Color::srgba(col[0] * alpha, col[1] * alpha, col[2] * alpha, alpha);
            }
        }
    }
}
