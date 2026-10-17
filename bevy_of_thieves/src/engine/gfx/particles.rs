//! WP-A13: particle rendering of class 28 (C2cb5d8, classes/k21_28.rs): slot 18 FUN_0013a1e8 -> Bevy.
//!
//! 1:1 (pure, tested headless): FUN_0013a1e8's instance selection and cull. With no parent (+0xe0 == 0, or
//! the parent's +0x11c member list empty, FUN_0011a3b0) the system is drawn once with the identity frame
//! DAT_002a2d30 when its bounding sphere (+0xf0 centre, +0x100 radius) passes the view test FUN_0014f850;
//! otherwise once per parent member (FUN_001a61a0 in the world) whose transformed centre passes
//! FUN_0014f850 / FUN_0014f898, with the member's frame (FUN_001a6318 / DAT_002a2d00 identity, w column
//! cleared, FUN_00256d18 position). Per instance and group the VU1 packet carries the live buffer's
//! positions (+0, w = size), spin (+0x560, w = angle) and, spin mode 1 only, velocities (+0x2b0); the
//! micro program is chosen by +0x43c: 0 -> 0x0 (upright), 1 -> 0x390 (along the velocity), 2 -> 0x870
//! (rotated by the spin angle).
//! REPLACED: the VU1 micro programs (VIF packets FUN_00226f28 / FUN_00227010, FUN_0013a688 GS / VU
//! constants +0x110 / +0x130 / +0x140 / +0x150 / +0x1b0 / +0x3b4) = one camera-facing quad per particle of
//! side `pos.w`, upright / velocity-aligned / rolled by `spin.w` per the program above, white additive
//! with the A6 star texture (the GS texture / colour constants are not decoded). The member frames of a
//! parented system are not in the world model: the identity frame is used (positions are level space).
//! Drawing (feature `game`, [`ParticlePlugin`] added by GfxPlugin): one dynamic mesh per frame of every live
//! class-28 instance in `Classes::set.k21c`, unlit, AlphaMode::Add, no shadows, no `RaytracingMesh3d`
//! (out of the BLAS; the mesh still has the Solari attribute set POSITION / NORMAL / UV_0 / TANGENT + U32).
//! Debug: `SLY2_PARTICLE_DEMO=1` feeds a demo system (real C2cb5d8 slot-16 stepping) 300 units in front of
//! the spawn so the path can be checked on a level without emitters.
use crate::engine::classes::k21_28::C2cb5d8;

/// DAT_002a2d30 / DAT_002a2d00: identity frame (rows)
pub const IDENT: [[f32; 4]; 4] = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];

/// One particle quad (level space).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quad {
    pub pos: [f32; 3],
    /// side (pos.w)
    pub size: f32,
    /// spin mode 2: roll angle (spin.w)
    pub angle: f32,
    /// spin mode 1: velocity (the 0x390 streak direction)
    pub vel: Option<[f32; 3]>,
    /// WP-C6: vertex colour (linear 0..1, alpha 1 = GS 0x80) from the +0x3b4 colour keys
    pub color: [f32; 4],
}

fn xform(m: &[[f32; 4]; 4], p: [f32; 3]) -> [f32; 3] {
    let mut r = [0f32; 3];
    for (c, v) in r.iter_mut().enumerate() {
        *v = m[0][c] * p[0] + m[1][c] * p[1] + m[2][c] * p[2] + m[3][c];
    }
    r
}

/// FUN_0013a1e8 instance list: `members` = the parent's in-world member frames (None = no parent / empty
/// list -> the identity frame); `cull(centre, radius)` = FUN_0014f850 (true = visible).
pub fn ee_sub_13a1e8_instances(sys: &C2cb5d8, members: Option<&[[[f32; 4]; 4]]>, cull: impl Fn([f32; 3], f32) -> bool) -> Vec<[[f32; 4]; 4]> {
    let (c, r) = (sys.m.v3(0xf0), sys.m.f(0x100));
    match members {
        None | Some([]) => {
            if cull(c, r) {
                vec![IDENT]
            } else {
                vec![]
            }
        }
        Some(ms) => ms
            .iter()
            .filter(|m| cull(xform(m, c), r))
            .map(|m| {
                let mut f = *m;
                // puVar3[3] / [7] / [0xb] = 0, [0xf] = 1.0
                for row in f.iter_mut().take(3) {
                    row[3] = 0.0;
                }
                f[3][3] = 1.0;
                f
            })
            .collect(),
    }
}

/// FUN_0013a1e8 packets -> quads: every group's live particles once per instance frame (white; see
/// [`ee_sub_13a1e8_quads_at`] for the colour keys).
pub fn ee_sub_13a1e8_quads(sys: &C2cb5d8, inst: &[[[f32; 4]; 4]]) -> Vec<Quad> {
    ee_sub_13a1e8_quads_at(sys, inst, None)
}

/// WP-C6: GS ALPHA_1 register sets of FUN_00138fc0 (FUN_00138f08(n, set): ALPHA_1 = n << 2 | 0x40,
/// TEST_1 = 0x7180b): DAT_002dd5c0 n = 1 -> A Cs, B Cd, C As, D Cd = (Cs - Cd) * As + Cd (blend);
/// DAT_002dd600 n = 2 -> B = 0: Cs * As + Cd (additive). TEST_1: ATE, ATST GEQUAL, AREF 0x80, AFAIL
/// FB_ONLY (pixels under alpha 0x80 keep colour but write no depth), ZTE, ZTST GREATER.
pub const GS_SET_BLEND: u32 = 0x2dd5c0;
pub const GS_SET_ADD: u32 = 0x2dd600;

/// ALPHA_1 of FUN_00138f08(n): A = Cs, B = n, C = As, D = Cd.
pub fn ee_sub_138f08_alpha(n: u32) -> u64 {
    (n as u64) << 2 | 0x40
}

/// WP-C6: the look part of FUN_001394c8(system, template) + FUN_00139370(system, shader):
/// - template +0xdc colour keys (count, then RGBA bytes per key) -> +0x3b0 count, +0x3b4 keys in reverse
///   order, every byte (b + 1) >> 1 (GS 0x80 = 1.0); +0x434 = template +0xcc (VU flag bit 0);
/// - no template keys and +0x3b0 still 0: one key = the shader's +0xc RGBA (same halving);
/// - +0x438 = GS register set: shader +8 bit 0 clear -> DAT_002dd5c0 (blend) else DAT_002dd600 (add);
/// - +0x1a0 = texture frame count rounded up to a power of two, at most 16 (the short tail repeats
///   the leading frames: `frames_padded`);
/// - +0x150 / +0x154 / +0x158 = 0, 1 / life (template +0x8c), 0: the VU's remaining-life fraction.
pub fn ee_sub_1394c8_look(sys: &mut C2cb5d8, keys: Option<&[[u8; 4]]>, flag_cc: u32, shader_flags: u32, shader_rgba: [u8; 4], frames: i32, life: f32) {
    let half = |b: [u8; 4]| u32::from_le_bytes(b.map(|x| ((x as u32 + 1) >> 1) as u8));
    if let Some(k) = keys {
        let n = k.len().min(0x30);
        sys.m.set_i(0x3b0, n as i32);
        for (i, key) in k.iter().take(n).enumerate() {
            sys.m.set_u(0x3b4 + 4 * (n - 1 - i) as u32, half(*key));
        }
        sys.m.set_u(0x434, flag_cc);
    }
    // FUN_00139370
    sys.m.set_u(0x438, if shader_flags & 1 == 0 { GS_SET_BLEND } else { GS_SET_ADD });
    if sys.m.i(0x3b0) == 0 {
        sys.m.set_i(0x3b0, 1);
        sys.m.set_u(0x3b4, half(shader_rgba));
    }
    sys.m.set_i(0x1a0, frames_padded(frames));
    sys.m.set_f(0x150, 0.0);
    sys.m.set_f(0x154, 1.0 / life);
    sys.m.set_f(0x158, 0.0);
}

/// FUN_00139370 frame count: next power of two (n itself when n is one), halved while above 31.
pub fn frames_padded(n: i32) -> i32 {
    let mut p = 1i32;
    let mut k = n;
    loop {
        k >>= 1;
        if k == 0 {
            break;
        }
        p <<= 1;
    }
    if p < n {
        p <<= 1;
    }
    while p > 0x1f {
        p >>= 1;
    }
    p
}

/// WP-C6: colour of a particle from the +0x3b4 keys: u = remaining life fraction (VU constant +0x150:
/// c0 + c1 * (death - now) + c2 * (death - now)^2), the reversed key table read at u * (n - 1) (desc key
/// 0 at birth, the last at death). REPLACED (VU1 micro program not decoded): linear between keys when
/// +0x434 bit 0 is set, else the lower key; GS 0x80 -> 1.0. No keys (+0x3b0 == 0, the system's look was
/// never set) -> white.
pub fn particle_color(sys: &C2cb5d8, death: f32, now: f32) -> [f32; 4] {
    let n = sys.m.i(0x3b0).clamp(0, 0x30) as usize;
    if n == 0 {
        return [1.0; 4];
    }
    let key = |i: usize| sys.m.u(0x3b4 + 4 * i as u32).to_le_bytes().map(|b| b as f32 / 128.0);
    let r = death - now;
    let u = (sys.m.f(0x150) + r * (sys.m.f(0x154) + r * sys.m.f(0x158))).clamp(0.0, 1.0);
    let x = u * (n - 1) as f32;
    let i = (x.floor() as usize).min(n - 1);
    let (a, f) = (key(i), x - i as f32);
    if sys.m.u(0x434) & 1 == 0 || i + 1 >= n {
        return a;
    }
    let b = key(i + 1);
    [0, 1, 2, 3].map(|c| a[c] + (b[c] - a[c]) * f)
}

/// FUN_0013a1e8 quads with colours at time `now` (None = white).
pub fn ee_sub_13a1e8_quads_at(sys: &C2cb5d8, inst: &[[[f32; 4]; 4]], now: Option<f32>) -> Vec<Quad> {
    let spin = sys.m.i(0x43c);
    let mut out = Vec::new();
    for m in inst {
        for g in &sys.groups {
            for p in &g.parts {
                out.push(Quad {
                    pos: xform(m, [p.pos[0], p.pos[1], p.pos[2]]),
                    size: p.pos[3],
                    angle: if spin == 2 { p.spin[3] } else { 0.0 },
                    vel: (spin == 1).then_some([p.vel[0], p.vel[1], p.vel[2]]),
                    color: now.map_or([1.0; 4], |t| particle_color(sys, p.vel[3], t)),
                });
            }
        }
    }
    out
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn scale(a: [f32; 3], k: f32) -> [f32; 3] {
    [a[0] * k, a[1] * k, a[2] * k]
}
fn norm(a: [f32; 3]) -> Option<[f32; 3]> {
    let l = dot(a, a).sqrt();
    (l > 1e-6).then(|| scale(a, 1.0 / l))
}

/// Quad corners (level space) facing a camera with axes `right` / `up` (unit, level space).
pub fn quad_corners(q: &Quad, right: [f32; 3], up: [f32; 3]) -> [[f32; 3]; 4] {
    let h = q.size * 0.5;
    // the streak: up along the velocity projected on the view plane
    let fwd = [right[1] * up[2] - right[2] * up[1], right[2] * up[0] - right[0] * up[2], right[0] * up[1] - right[1] * up[0]];
    let (r, u) = match q.vel.and_then(|v| norm(sub(v, scale(fwd, dot(v, fwd))))) {
        Some(u2) => {
            let r2 = [u2[1] * fwd[2] - u2[2] * fwd[1], u2[2] * fwd[0] - u2[0] * fwd[2], u2[0] * fwd[1] - u2[1] * fwd[0]];
            (r2, u2)
        }
        None => {
            let (s, c) = q.angle.sin_cos();
            let r2 = [right[0] * c + up[0] * s, right[1] * c + up[1] * s, right[2] * c + up[2] * s];
            let u2 = [up[0] * c - right[0] * s, up[1] * c - right[1] * s, up[2] * c - right[2] * s];
            (r2, u2)
        }
    };
    let p = q.pos;
    let at = |a: f32, b: f32| [p[0] + (r[0] * a + u[0] * b) * h, p[1] + (r[1] * a + u[1] * b) * h, p[2] + (r[2] * a + u[2] * b) * h];
    [at(-1.0, -1.0), at(1.0, -1.0), at(1.0, 1.0), at(-1.0, 1.0)]
}

/// Mesh arrays of the quads: positions, normals, uvs, tangents, u32 indices (the Solari attribute set).
#[allow(clippy::type_complexity)]
pub fn quad_arrays(qs: &[Quad], right: [f32; 3], up: [f32; 3]) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<[f32; 4]>, Vec<u32>) {
    let n = [right[1] * up[2] - right[2] * up[1], right[2] * up[0] - right[0] * up[2], right[0] * up[1] - right[1] * up[0]];
    let (mut p, mut nn, mut uv, mut t, mut ix) = (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for q in qs {
        let b = p.len() as u32;
        p.extend(quad_corners(q, right, up));
        nn.extend([n; 4]);
        uv.extend([[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]]);
        t.extend([[right[0], right[1], right[2], 1.0]; 4]);
        ix.extend([b, b + 1, b + 2, b, b + 2, b + 3]);
    }
    (p, nn, uv, t, ix)
}

/// Demo system (debug `SLY2_PARTICLE_DEMO`): a free-flight (motion mode 1) spinning fountain at `at`.
pub fn demo_system(at: [f32; 3], now: f32) -> C2cb5d8 {
    use crate::engine::classes::k21_28::Particle;
    let mut s = C2cb5d8::default();
    s.m.set_i(0x160, 1);
    s.m.set_i(0x43c, 2);
    let parts = (0..48)
        .map(|k| {
            let a = k as f32 * 0.7;
            let life = 0.6 + (k % 12) as f32 * 0.12;
            Particle {
                pos: [at[0], at[1], at[2], 18.0 + (k % 5) as f32 * 4.0],
                vel: [a.cos() * 120.0, a.sin() * 120.0, 220.0 + (k % 7) as f32 * 20.0, now + life],
                spin: [0.0, 0.0, 3.0, a],
                ..Default::default()
            }
        })
        .collect();
    s.add_group(parts);
    s
}

#[cfg(feature = "game")]
pub use plugin::*;

#[cfg(feature = "game")]
mod plugin {
    use super::*;
    use crate::engine::camera::bevy_sync::CmCamera;
    use crate::engine::classes::plugin::Classes;
    use crate::engine::classes::Inst;
    use crate::engine::gfx::game::LevelGeometryRoot;
    use bevy::asset::RenderAssetUsages;
    use bevy::light::NotShadowCaster;
    use bevy::mesh::{Indices, PrimitiveTopology};
    use bevy::prelude::*;

    /// The particle mesh entity (per level root and GS register set: 0 additive, 1 blend).
    #[derive(Component)]
    pub struct ParticleMesh(pub usize);

    /// Debug demo systems (`SLY2_PARTICLE_DEMO`), stepped with the real slot 16.
    #[derive(Resource, Default)]
    pub struct ParticleDemo {
        pub on: bool,
        pub systems: Vec<C2cb5d8>,
        next: f32,
    }

    /// Quads drawn last frame (diagnostics).
    #[derive(Resource, Default)]
    pub struct ParticleStats {
        pub quads: usize,
        pub logged: bool,
    }

    pub struct ParticlePlugin;
    impl Plugin for ParticlePlugin {
        fn build(&self, app: &mut App) {
            let on = std::env::var("SLY2_PARTICLE_DEMO").is_ok_and(|v| v != "0");
            app.insert_resource(ParticleDemo { on, ..default() }).init_resource::<ParticleStats>().add_systems(PostUpdate, draw_particles.before(bevy::transform::TransformSystems::Propagate));
        }
    }

    fn mesh_of(qs: &[Quad], right: [f32; 3], up: [f32; 3]) -> Mesh {
        let (p, n, uv, t, ix) = quad_arrays(qs, right, up);
        let mut m = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD);
        m.insert_attribute(Mesh::ATTRIBUTE_POSITION, p);
        m.insert_attribute(Mesh::ATTRIBUTE_NORMAL, n);
        m.insert_attribute(Mesh::ATTRIBUTE_UV_0, uv);
        m.insert_attribute(Mesh::ATTRIBUTE_TANGENT, t);
        // WP-C6: colour keys (GS vertex colour, modulates the texture)
        m.insert_attribute(Mesh::ATTRIBUTE_COLOR, qs.iter().flat_map(|q| [q.color; 4]).collect::<Vec<_>>());
        m.insert_indices(Indices::U32(ix));
        // WP-C6: the quad layout is exactly Solari's BLAS format, so bevy_solari (compiled in with
        // feature `rt`, even with GI off) built a BLAS for it and panicked on the empty (0-quad) mesh
        // (blas.rs mesh_vertex_slice().unwrap()). Particles stay out of the BLAS (module doc).
        m.enable_raytracing = false;
        m
    }

    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    fn draw_particles(
        mut commands: Commands,
        classes: Option<Res<Classes>>,
        mut demo: ResMut<ParticleDemo>,
        mut stats: ResMut<ParticleStats>,
        time: Res<Time>,
        cam: Query<&GlobalTransform, With<CmCamera>>,
        roots: Query<(Entity, &GlobalTransform), With<LevelGeometryRoot>>,
        existing: Query<(Entity, &Mesh3d, &ChildOf, &ParticleMesh)>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut mats: ResMut<Assets<StandardMaterial>>,
        mut images: ResMut<Assets<Image>>,
        mut mat: Local<[Option<Handle<StandardMaterial>>; 2]>,
    ) {
        let (Some(cg), Some((root, rg))) = (cam.iter().next(), roots.iter().next()) else { return };
        let to_level = rg.affine().inverse();
        let right = to_level.transform_vector3(cg.right().as_vec3()).normalize_or_zero().to_array();
        let up = to_level.transform_vector3(cg.up().as_vec3()).normalize_or_zero().to_array();
        let eye = to_level.transform_point3(cg.translation());
        let back = to_level.transform_vector3(cg.back().as_vec3()).normalize_or_zero();
        // FUN_0014f850 stand-in: the sphere is not entirely behind the camera
        let cull = |c: [f32; 3], r: f32| (Vec3::from_array(c) - eye).dot(-back) > -r;
        if demo.on {
            let (t, dt) = (time.elapsed_secs(), time.delta_secs().min(1.0 / 15.0));
            if t >= demo.next {
                demo.next = t + 0.25;
                let at = (eye - back * 300.0).to_array();
                demo.systems.push(demo_system(at, t));
            }
            for s in &mut demo.systems {
                // motion mode 1 + gravity-free free flight: slot 16 FUN_00139dc0 (frame unused in mode 1)
                s.ee_sub_139dc0(t, dt, &IDENT);
            }
            demo.systems.retain(|s| !s.removed);
        }
        // WP-C6: one mesh per GS register set (+0x438): [0] additive (DAT_002dd600, or no look set), [1] blend
        let mut qs: [Vec<Quad>; 2] = [Vec::new(), Vec::new()];
        let set = classes.as_ref().map(|c| &c.set.k21c);
        let live = set.into_iter().flatten().filter_map(|(_, x)| match x {
            Inst::K21C28(c) if !c.removed => Some(&**c),
            _ => None,
        });
        let now = time.elapsed_secs();
        for s in live.chain(demo.systems.iter()) {
            // the parent's member frames are not modelled: identity (positions in level space)
            let inst = ee_sub_13a1e8_instances(s, None, cull);
            qs[(s.m.u(0x438) == GS_SET_BLEND) as usize].extend(ee_sub_13a1e8_quads_at(s, &inst, Some(now)));
        }
        stats.quads = qs[0].len() + qs[1].len();
        if stats.quads > 0 && !stats.logged {
            stats.logged = true;
            info!("gfx: class-28 particles drawn ({} quads, FUN_0013a1e8)", stats.quads);
        }
        for (k, q) in qs.iter().enumerate() {
            let mesh = mesh_of(q, right, up);
            if let Some((_, m, _, _)) = existing.iter().find(|(_, _, p, pm)| p.parent() == root && pm.0 == k) {
                if let Some(mut slot) = meshes.get_mut(&m.0) {
                    *slot = mesh;
                }
                continue;
            }
            if q.is_empty() {
                continue; // spawned with the first quad (an empty mesh asset only warns every frame)
            }
            for (e, _, p, pm) in &existing {
                if pm.0 == k || p.parent() != root {
                    commands.entity(e).despawn();
                }
            }
            let mh = mat[k]
                .get_or_insert_with(|| {
                    let tex = images.add(crate::engine::gfx::fx::star_texture());
                    let mut m = crate::engine::gfx::fx::fx_billboard_material(tex);
                    m.base_color = Color::WHITE;
                    if k == 1 {
                        m.alpha_mode = AlphaMode::Blend;
                    }
                    mats.add(m)
                })
                .clone();
            commands.spawn((Mesh3d(meshes.add(mesh)), MeshMaterial3d(mh), Transform::default(), NotShadowCaster, bevy::camera::visibility::NoFrustumCulling, ParticleMesh(k), ChildOf(root)));
        }
    }
}
