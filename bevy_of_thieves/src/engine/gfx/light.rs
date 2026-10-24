//! WP-C3: LIGHT objects (class 4, vtable 0x2d33a0, object size 0x170) -> light parameters.
//!
//! Load = FUN_00231170 (class-3 XFM: s16 bone ref, 4x3 matrix, u32, property list). The
//! property setters (field table 0x2ac880 entries 151..158) write the members below and every
//! one re-runs FUN_001a3968, so the final state is "init defaults, props applied in stream
//! order, then FUN_001a3968 on the final world matrix" (it is re-run again by the post-load /
//! transform virtuals slots 10/14/16/28/29 = FUN_001a3400/1a3430/1a3460/1a3550/1a3580).
//!
//! Original pipeline (raster): FUN_001a3d00 registers the light in the static light table
//! (g_lsm 0x3dd490 +0x4154, max 256); directional lights (kind 0) go into the global bit set
//! (+0x4570) that applies everywhere, point lights (kind 1, not dynamic) are bucketed into BSP
//! regions within radius = falloff far (FUN_001a3b80). Per object FUN_001a3f78 picks the strongest
//! lights by `strength = du_fall + ru_fall * distance` and the VU1 code shades vertices with the
//! angular term `du + ru * (c + c^3)` (c = cos of half the normal/light angle, FUN_001a38a8) times
//! `colour`, on top of the baked PS2 vertex colours.
//!
//! Bevy mapping (`gfx::game`): kind 0 -> DirectionalLight, kind 1 -> PointLight (range = far,
//! linear falloff approximated by Bevy's inverse-square + range cutoff), others not spawned.
//! Raster: the vertex colours stay the base colour (COLOR attribute) and the lights add on top
//! through StandardMaterial; RT (Solari): the same Bevy light entities are the GI sources.

use super::attach::BoneWorlds;
use super::fog::{ee_sub_11dd90, ee_sub_1ed2d8};
use super::scene::Affine;
use crate::engine::world::objects::{LoadData, Payload};
use crate::engine::world::props::{Prop, PropValue};
use crate::engine::world::World;

pub const CID_LIGHT: u16 = 4;

/// Members of the LIGHT object written by the loader (offsets in the original object).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LightParams {
    /// +0xe0 (field 151, setter FUN_001a35b0): 0 directional, 1 point, other = not lit
    pub kind: u32,
    /// +0xf0 (field 152, FUN_001a35f0): HSV, h degrees, s and v 0..255
    pub hsv: [f32; 3],
    /// +0x150 (field 153, FUN_001a3610)
    pub g150: f32,
    /// +0x154 (field 154, FUN_001a3630): angle (degrees) where the angular term reaches 0
    pub angle: f32,
    /// +0x158 (field 156, direct u8)
    pub b158: u8,
    /// +0x15c (field 157, FUN_001a3680): excluded (never in the active light sets)
    pub excluded: bool,
    /// +0x160 (field 158, FUN_001a36d8): dynamic (not baked into BSP regions)
    pub dynamic: bool,
    /// +0x164/+0x168 (field 155, FUN_001a3650): falloff near / far
    pub falloff: [f32; 2],
}

/// FUN_001a3360 (0x001a3360): LIGHT init defaults.
pub fn ee_sub_1a3360() -> LightParams {
    LightParams { kind: 0, hsv: [0.0; 3], g150: 0.0, angle: 240.0, b158: 1, excluded: false, dynamic: false, falloff: [200.0, 2000.0] }
}

fn f32_at(b: &[u8], k: usize) -> f32 {
    b.get(k * 4..k * 4 + 4).map(|x| f32::from_le_bytes(x.try_into().unwrap())).unwrap_or(0.0)
}

/// One LIGHT property setter (field table 0x2ac880 entries 151..158), as the loader and the
/// runtime property path call it. Returns false for fields that are not LIGHT fields. Every LIGHT
/// setter ends by re-running FUN_001a3968; callers must do the same (`ee_sub_1a3968`).
pub fn set_field(p: &mut LightParams, field: i16, b: &[u8]) -> bool {
    match field {
        151 => p.kind = b.get(..4).map(|x| u32::from_le_bytes(x.try_into().unwrap())).unwrap_or(0), // 0x001a35b0
        152 => ee_sub_1a35f0(p, b),
        153 => ee_sub_1a3610(p, b),
        154 => ee_sub_1a3630(p, b),
        155 => ee_sub_1a3650(p, b),
        156 => p.b158 = b.first().copied().unwrap_or(0), // direct u8 +0x158
        157 => ee_sub_1a3680(p, b.first().copied().unwrap_or(0) != 0),
        158 => ee_sub_1a36d8(p, b.first().copied().unwrap_or(0) != 0),
        _ => return false,
    }
    true
}

/// FUN_001a35f0 SetLightColor (HSV, field 152).
pub fn ee_sub_1a35f0(p: &mut LightParams, b: &[u8]) {
    p.hsv = [f32_at(b, 0), f32_at(b, 1), f32_at(b, 2)];
}
/// FUN_001a3610 (field 153, +0x150).
pub fn ee_sub_1a3610(p: &mut LightParams, b: &[u8]) {
    p.g150 = f32_at(b, 0);
}
/// FUN_001a3630 (field 154, angle +0x154).
pub fn ee_sub_1a3630(p: &mut LightParams, b: &[u8]) {
    p.angle = f32_at(b, 0);
}
/// FUN_001a3650 SetLightFallOff (field 155).
pub fn ee_sub_1a3650(p: &mut LightParams, b: &[u8]) {
    p.falloff = [f32_at(b, 0), f32_at(b, 1)];
}
/// FUN_001a3680 (field 157, +0x15c excluded): setting it clears dynamic.
pub fn ee_sub_1a3680(p: &mut LightParams, v: bool) {
    p.excluded = v;
    if v {
        p.dynamic = false;
    }
}
/// FUN_001a36d8 SetLightDynamic (field 158, +0x160): setting it clears excluded.
pub fn ee_sub_1a36d8(p: &mut LightParams, v: bool) {
    p.dynamic = v;
    if v {
        p.excluded = false;
    }
}

/// Apply the light property setters in stream order (fields 151..158; others are generic LO fields).
pub fn apply_props(p: &mut LightParams, props: &[Prop]) {
    for pr in props {
        if let PropValue::Scalar(b) = &pr.value {
            set_field(p, pr.field, b);
        }
    }
}

/// FUN_0011d4d0 FFloatsNear (Sly 2 relative form): |a-b| / max(1,|a|) < eps.
fn floats_near(a: f32, b: f32, eps: f32) -> bool {
    (a - b).abs() / a.abs().max(1.0) < eps
}

/// FUN_001a3730 (0x001a3730) FitLinearFunction -> (du, ru).
pub fn ee_sub_1a3730(x0: f32, y0: f32, x1: f32, y1: f32) -> (f32, f32) {
    if floats_near(x0, x1, 1e-4) {
        return (y0, 0.0);
    }
    let ru = (y1 - y0) / (x1 - x0);
    (y0 - ru * x0, ru)
}

/// FUN_001a38a8 (0x001a38a8) ConvertAngleStrength.
pub fn ee_sub_1a38a8(deg0: f32, g0: f32, deg1: f32, g1: f32) -> (f32, f32) {
    let a = (deg0 * 0.008726647).cos();
    let b = (deg1 * 0.008726647).cos();
    ee_sub_1a3730(a + a * a * a, g0 * 0.003921569, b + b * b * b, g1 * 0.003921569)
}

/// FUN_0011dfa0: HSV with s,v in 0..255 -> RGB 0..255.
pub fn ee_sub_11dfa0(hsv: [f32; 3]) -> [f32; 3] {
    ee_sub_11dd90([hsv[0], hsv[1] * 0.003921569, hsv[2] * 0.003921569]).map(|c| c * 255.0)
}

/// Derived light state written by FUN_001a3968.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LightDerived {
    /// +0x124 / +0x128 (ru negated): angular term du + ru' * (c + c^3)
    pub du: f32,
    pub ru: f32,
    /// +0x130..+0x138: rgb (0..255) / g
    pub color: [f32; 3],
    /// +0x13c: +0x150 / g
    pub k13c: f32,
    /// g = max(+0x150, V): overall strength (0..255 scale)
    pub g: f32,
    /// +0x140: kind 0 = world matrix row 0 (direction), kind 1 = world position
    pub vec: [f32; 3],
    /// +0x110 / +0x114: distance falloff (strength = du_fall + ru_fall * dist), already * g
    pub du_fall: f32,
    pub ru_fall: f32,
}

/// FUN_001a3968 (0x001a3968). `grey_mode` = DAT_003e12c8 (FUN_001ed2d8 grey filter).
pub fn ee_sub_1a3968(p: &LightParams, world: &Affine, grey_mode: bool) -> LightDerived {
    let max = |a: f32, b: f32| if a < b { b } else { a };
    let g = max(p.g150, p.hsv[2]);
    let (du, ru) = ee_sub_1a38a8(if 30.0 < p.angle { p.angle } else { 30.0 }, 0.0, 0.0, g);
    let mut rgb = ee_sub_11dfa0(p.hsv);
    if grey_mode {
        // quirk kept: the grey filter runs on the 0..255 values (result is 0..1 grey)
        rgb = ee_sub_1ed2d8(rgb);
    }
    let (mut color, mut k13c) = (rgb, p.g150);
    if 0.0 < g {
        k13c = p.g150 / g;
        color = rgb.map(|c| c / g);
    }
    let (vec, mut du_fall, mut ru_fall) = match p.kind {
        0 => (world[0], 1.0, 0.0),
        1 => {
            let (d, r) = ee_sub_1a3730(p.falloff[0], 1.0, p.falloff[1], 0.0);
            (world[3], d, r)
        }
        // other kinds: +0x140/+0x110/+0x114 keep their previous values (zero after init)
        _ => ([0.0; 3], 0.0, 0.0),
    };
    du_fall *= g;
    ru_fall *= g;
    LightDerived { du, ru: -ru, color, k13c, g, vec, du_fall, ru_fall }
}

/// One level light.
#[derive(Clone, Debug, PartialEq)]
pub struct LevelLight {
    pub object: usize,
    pub oid: i32,
    pub params: LightParams,
    pub world: Affine,
    pub derived: LightDerived,
}

impl LevelLight {
    /// Light colour 0..1 as the VU1 sees it at full angular/distance term: rgb/255
    /// (= `color * g / 255`).
    pub fn color01(&self) -> [f32; 3] {
        self.derived.color.map(|c| c * self.derived.g / 255.0)
    }
    /// angular term at angle `theta` (radians) between normal and light direction: the
    /// FitLinear line through (f(angle), 0) and (f(0) = 2, g/255), f = c + c^3, c = cos(theta/2)
    /// (+0x128 holds the slope negated; the VU1 side subtracts it).
    pub fn angular(&self, theta: f32) -> f32 {
        let c = (theta * 0.5).cos();
        self.derived.du - self.derived.ru * (c + c * c * c)
    }
    /// distance term (original strength used for light selection)
    pub fn distance(&self, d: f32) -> f32 {
        self.derived.du_fall + self.derived.ru_fall * d
    }
}

/// All class-4 lights of a world (`grey_mode` false = normal play).
pub fn level_lights(w: &World, bones: &mut BoneWorlds) -> Vec<LevelLight> {
    let mut out = Vec::new();
    for (i, o) in w.objects.iter().enumerate() {
        if o.class != CID_LIGHT {
            continue;
        }
        // LOOK R8: FUN_001ca370 template copies of a LIGHT (`Payload::Clone`) are full lights in the
        // original (world list 0x3dd490 + 0x4154: oid 5613 template + 11 copies in the cellar, each at
        // its own place). Props = the template's then the copy's override rows; world = template world
        // * the +0x12c placement (as gfx::scene::template_copies).
        let src = o.clone_of.unwrap_or(i);
        let Some(Payload::Loaded(LoadData::L231170 { props, .. })) = w.objects.get(src).map(|s| &s.payload) else { continue };
        let mut p = ee_sub_1a3360();
        apply_props(&mut p, props);
        if let Some(ov) = w.override_props.get(&i) {
            apply_props(&mut p, ov);
        }
        let mut world = bones.xfm_world(w, src).unwrap_or(super::scene::IDENTITY);
        if let Some(m) = w.life.placed.get(&i) {
            world = super::scene::mul(&world, &super::attach::affine_from_bytes(m));
        } else if src != i {
            continue;
        }
        out.push(LevelLight { object: i, oid: o.oid, params: p, world, derived: ee_sub_1a3968(&p, &world, false) });
    }
    out
}

/// WP-C4: PS2 light unit -> Bevy illuminance (lux).
///
/// The VU1 lighting block (overlay 376110651, 0x10c0..0x13c0) computes per vertex
/// `L = ambient + sum_i colour_i * min(f_i(n), max_i)` with `f = c + c^3` per light (the
/// FUN_001a38a8 line, whose value at full facing is `g/255` = 1.0 for a full-strength light), then
/// `out = vertex_colour * L` (MULw/MULq on the ITOF'd V4-8 colour, FTOI0; light above 1.0 spills
/// into a white "overbright" term). So one PS2 light unit means "surface shows exactly its albedo
/// (vertex colour x texture)".
/// Bevy PBR (Lambert diffuse) shows `albedo * E / PI * exposure`, exposure = 2^-ev100 / 1.2
/// (bevy_camera `Exposure::exposure`). Setting that to `albedo` gives `E = PI * 1.2 * 2^ev100`:
/// 3135.6 lux at the default `Exposure::BLENDER` (ev100 9.7). The cosine factor Bevy adds is the
/// PBR stand-in for the PS2 `c + c^3` angular curve (both are 1.0 facing the light).
pub fn ps2_light_unit_lux(ev100: f32) -> f32 {
    std::f32::consts::PI * 1.2 * ev100.exp2()
}

/// WP-C9: the VU1 setup block of overlay 376110651 (0x3e0..0x5e0) multiplies EVERY uploaded
/// light term by 2.0 (MULi 0x568..0x590) before storing rows 21..30: a full-strength light facing
/// the surface gives L = 2, and the GS modulate (texel * Cv / 0x80) shows 2x the texel byte. The
/// level textures are authored for that (grey CLUT ramps mostly 0x20..0x9f, J3), so "1 PS2 light
/// unit" = albedo only for half the VU light value.
pub const VU1_LIGHT_SCALE: f32 = 2.0;

/// WP-C9: the PS2 lights and the vertex colours scale bytes in display (gamma) space
/// (`texel * vcol * L`, clamped by the GS), Bevy multiplies linear values. A display factor x is
/// the linear factor x^2.2 (sRGB ~ gamma 2.2): `out_lin = texel_lin * x^2.2` encodes to
/// `texel * x`. Without it a dark term (L = 0.37) showed as 0.64 -> flat, low contrast frames.
pub const PS2_GAMMA: f32 = 2.2;

/// display-space factor -> the factor Bevy multiplies with, per channel, negatives clamped.
/// LOOK: with `display_space` (default) Bevy works in PS2 display numbers (gfx::display), so the
/// factor is unchanged; `SLY2_LOOK_LINEAR=1` = the old linear path (x^`PS2_GAMMA`).
pub fn ps2_linear(x: f32) -> f32 {
    if display_space() { x.max(0.0) } else { x.max(0.0).powf(PS2_GAMMA) }
}

/// LOOK: 3D colours are PS2 display values (raw texel bytes, vertex colours, light factors, fog
/// colour), blended like the GS, decoded to linear once per frame (gfx::display). Off with
/// `SLY2_LOOK_LINEAR=1`.
pub fn display_space() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("SLY2_LOOK_LINEAR").map_or(true, |v| v != "1"))
}

/// LOOK: texture format of PS2 texel bytes (`display_space`: raw, else sRGB-decoded).
#[cfg(feature = "game")]
pub fn ps2_texture_format() -> bevy::render::render_resource::TextureFormat {
    use bevy::render::render_resource::TextureFormat as F;
    if display_space() { F::Rgba8Unorm } else { F::Rgba8UnormSrgb }
}

/// LOOK: a PS2 display-space colour as a Bevy colour (`display_space`: raw numbers, else sRGB).
#[cfg(feature = "game")]
pub fn ps2_color(r: f32, g: f32, b: f32, a: f32) -> bevy::color::Color {
    if display_space() { bevy::color::Color::linear_rgba(r, g, b, a) } else { bevy::color::Color::srgba(r, g, b, a) }
}

/// WP-C9: per-light terms as the VU1 applies them (PS2 display units, `VU1_LIGHT_SCALE`
/// included): (ambient, facing). FUN_001a4478 uploads +0x120..+0x14f (q0 = (+0x120, du, ru,
/// +0x12c), q1 = colour rgb/g + k13c, q2 = vec) to VU rows 0x15 + 3i; the setup sums
/// `colour_i * du_i` into the ambient row (vf10 = row 29) and the angular slope into the light
/// matrix, so a surface perpendicular to the light gets `colour * du` and one facing it
/// `colour * g/255` (the FitLinear end point).
pub fn vu1_light_terms(d: &LightDerived) -> ([f32; 3], [f32; 3]) {
    let amb = d.color.map(|c| VU1_LIGHT_SCALE * c * d.du.max(0.0));
    let face = d.color.map(|c| VU1_LIGHT_SCALE * c * d.g / 255.0);
    (amb, face)
}

/// ev100 of bevy_camera `Exposure::default()` (= `Exposure::BLENDER`).
pub const BEVY_DEFAULT_EV100: f32 = 9.7;

/// WP-C9: the VU1 ambient row (vf10 = row 29) of the global (directional, FUN_001a3d00 bit set
/// +0x4570) lights: sum of `colour * du` x2 over every active directional LIGHT, linearised, as
/// `GlobalAmbientLight` (gfx::game::ambient_resource; colour, brightness in nits: Bevy shows `albedo * brightness * exposure`,
/// so one PS2 unit = `ps2_light_lux() / PI` = 998.6 nits at the default exposure). Point lights'
/// du parts are local (BSP buckets) and not folded in.
pub fn level_ambient<'a>(lights: impl IntoIterator<Item = &'a (LightParams, Affine)>) -> [f32; 3] {
    let mut a = [0.0f32; 3];
    for (p, w) in lights {
        if p.excluded || p.kind != 0 {
            continue;
        }
        let d = ee_sub_1a3968(p, w, false);
        if d.g <= 0.0 {
            continue;
        }
        let (amb, _) = vu1_light_terms(&d);
        for k in 0..3 {
            a[k] += amb[k];
        }
    }
    a.map(ps2_linear)
}

/// WIRE-C3: colour-pass factor k of the kinds-1/7 VU1 program (`material::two_layer_vu`) for the
/// level's global (directional) lights at full facing: L = sum of the facing terms
/// (`vu1_light_terms`), Lw = sum of the light w terms (q1.w = k13c = +0x150 / g, times the angular
/// end point g/255 and the x2 VU scale = 2 * +0x150 / 255). Local point lights are not folded in
/// (they only raise L/Lw near them). No directional light -> 1.0 (unlit colour shown as is).
pub fn two_layer_k(w: &World) -> f32 {
    let (mut l, mut lw, mut any) = ([0.0f32; 3], 0.0f32, false);
    for lt in level_lights(w, &mut BoneWorlds::default()) {
        if lt.params.excluded || lt.params.kind != 0 || lt.derived.g <= 0.0 {
            continue;
        }
        let (_, face) = vu1_light_terms(&lt.derived);
        (0..3).for_each(|k| l[k] += face[k]);
        lw += VU1_LIGHT_SCALE * lt.derived.k13c * lt.derived.g / 255.0;
        any = true;
    }
    if !any {
        return 1.0;
    }
    super::material::two_layer_vu(l, lw).1
}

/// LOOK: one light as the VU1 lighting block sees it (FUN_001a4478 upload of +0x110..+0x14f).
#[derive(Clone, Copy, Debug)]
pub struct Vu1Light {
    pub kind: u32,
    pub d: LightDerived,
}

/// LOOK: the level's lights that reach the VU1 (kinds 0/1, not excluded, g > 0).
pub fn vu1_light_set(w: &World) -> Vec<Vu1Light> {
    level_lights(w, &mut BoneWorlds::default())
        .into_iter()
        .filter(|l| !l.params.excluded && l.params.kind <= 1 && l.derived.g > 0.0 && in_world_light_list(w, l))
        .map(|l| Vu1Light { kind: l.params.kind, d: l.derived })
        .collect()
}

/// LOOK: `vu1_light_set` with each light's +0x158 (field 156, `b158`): FUN_001a4478 weights only the
/// picked lights with +0x158 != 0 into the shadow-casting light position.
pub fn cast_light_set(w: &World) -> Vec<(Vu1Light, bool)> {
    level_lights(w, &mut BoneWorlds::default())
        .into_iter()
        .filter(|l| !l.params.excluded && l.params.kind <= 1 && l.derived.g > 0.0 && in_world_light_list(w, l))
        .map(|l| (Vu1Light { kind: l.params.kind, d: l.derived }, l.params.b158 != 0))
        .collect()
}

/// LOOK: membership of the world light list (world 0x3dd490 + 0x4154, filled by FUN_001a3d00 via the
/// LIGHT add FUN_001a3518 = vt+0xbc). FUN_001a5478 only calls it when FUN_001a61a0 holds, which an
/// owned LIGHT (LO +0x10 != 0, e.g. the black rim directionals oid 7430 / 3797 / 7031 under Bentley /
/// Sly / Murray, and the oid 5613 lights under C2ce4d0 actors) never satisfies: in the cellar savestate
/// (emu_rec5/000_082432) all 103 list lights have +0x10 = 0 and every owned light has +0x118 = -1.
/// Those lights belong to their actor's own light set (+0x134, FUN_001f3c10), not to level geometry.
pub fn in_world_light_list(w: &World, l: &LevelLight) -> bool {
    w.objects.get(l.object).is_some_and(|o| o.owner.is_none())
}

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// FUN_001a3f78 (0x001a3f78): the (at most `n`, FUN_001a4478 uploads 4) strongest lights at `pos`
/// by `strength = +0x110 + +0x114 * |pos - +0x100|` (directional: +0x110 = g, +0x114 = 0), sorted
/// descending; lights with strength <= 0 are not picked (the list is zero-filled).
pub fn ee_sub_1a3f78(lights: &[Vu1Light], pos: [f32; 3], n: usize) -> Vec<Vu1Light> {
    let mut v: Vec<(f32, Vu1Light)> = lights
        .iter()
        .map(|l| (if l.kind == 0 { l.d.du_fall } else { l.d.du_fall + l.d.ru_fall * dist(l.d.vec, pos) }, *l))
        .filter(|(s, _)| *s > 0.0)
        .collect();
    v.sort_by(|a, b| b.0.total_cmp(&a.0));
    v.into_iter().take(n).map(|(_, l)| l).collect()
}

/// FUN_001a4478 (0x001a4478) pick position: the light set's position `p` (+4 = root +0x60 + 0x70, the
/// root's local translation) as is when param_2 is NULL (FUN_001f3c10, ra 0x001f3c4c), else transformed
/// by the param_2 matrix (FUN_001f3cc8 passes the draw's parent world matrix, ra 0x001f3d7c):
/// `M.row0 * p.x + M.row1 * p.y + M.row2 * p.z + M.row3`. USER CAPTURE L4 (cellar): p = (0, 0, 13.2717),
/// M row3 = (12710.8, -6310.4, 259.1) -> (12712.2, -6309.1, 265.4).
pub fn ee_sub_1a4478_pos(p: [f32; 3], m: Option<&Affine>) -> [f32; 3] {
    match m {
        None => p,
        Some(m) => [0, 1, 2].map(|j| m[0][j] * p[0] + m[1][j] * p[1] + m[2][j] * p[2] + m[3][j]),
    }
}

/// FUN_001a4478 (0x001a4478) light upload: the (at most 4) strongest lights at the pick position
/// (`ee_sub_1a4478_pos`, FUN_001a3f78), uploaded in strength order to VU rows 0x15 + 3i (+0x120..+0x14f);
/// the list stops at the first empty pick and the remaining slots get the zero light DAT_002bfc60
/// (q0 = (0, 0, 0.5, 0), colour 0: contributes nothing, so it is not returned). Nothing is uploaded
/// when the light set's +0x10 is 0.
pub fn ee_sub_1a4478(lights: &[Vu1Light], p: [f32; 3], m: Option<&Affine>) -> Vec<Vu1Light> {
    ee_sub_1a3f78(lights, ee_sub_1a4478_pos(p, m), 4)
}

/// LOOK: VU1 light prep (overlay 376110651, 0x2a0..0x3c8), once per draw before the vertex loop:
/// vf5 = row 3 of the draw matrix; each light with q0.x != 0 (point): q2 = vf5 - q2 (light -> origin),
/// att = clamp(q0.w + q2.w * |q2|, 0, 1) (= (+0x110 + +0x114 * d) / g), q0.yz (du, ru) *= att; then q2
/// is rotated to local space and normalised. A point light is a directional light per draw (no
/// per-vertex fall-off on the VU).
pub fn vu1_prep_lights(lights: &[Vu1Light], at: [f32; 3]) -> Vec<Vu1Light> {
    lights
        .iter()
        .map(|l| {
            if l.kind == 0 {
                return *l;
            }
            let mut o = *l;
            let att = ((l.d.du_fall + l.d.ru_fall * dist(l.d.vec, at)) / l.d.g).clamp(0.0, 1.0);
            o.kind = 0;
            o.d.vec = [at[0] - l.d.vec[0], at[1] - l.d.vec[1], at[2] - l.d.vec[2]];
            o.d.du *= att;
            o.d.ru *= att;
            o
        })
        .collect()
}

/// LOOK: per-vertex VU1 light terms (L rgb, Lw) of the kinds-1/7 program for a world position and
/// unit world normal: per light `colour * (du + ru * (c + c^3))` with c = cos of half the
/// normal / to-light angle (FUN_001a38a8 fit, clamped at 0), point lights times the linear distance
/// falloff (+0x110 + +0x114 * d) / g clamped to 0..1, all x2 (`VU1_LIGHT_SCALE`). Lw sums the same
/// terms with the w channel (+0x13c). Feeds `material::two_layer_vu` per vertex (replaces the one
/// level-wide `two_layer_k`, which left the additive colour layer unshaded).
pub fn vu1_vertex_terms(lights: &[Vu1Light], pos: [f32; 3], n: [f32; 3]) -> ([f32; 3], f32) {
    let (mut l, mut lw) = ([0.0f32; 3], 0.0f32);
    for lt in lights {
        let d = &lt.d;
        let (to, att) = if lt.kind == 0 {
            ([-d.vec[0], -d.vec[1], -d.vec[2]], 1.0)
        } else {
            let r = dist(d.vec, pos);
            ([d.vec[0] - pos[0], d.vec[1] - pos[1], d.vec[2] - pos[2]], ((d.du_fall + d.ru_fall * r) / d.g).clamp(0.0, 1.0))
        };
        let len = (to[0] * to[0] + to[1] * to[1] + to[2] * to[2]).sqrt();
        if len <= 1e-12 || att <= 0.0 {
            continue;
        }
        let cosf = ((n[0] * to[0] + n[1] * to[1] + n[2] * to[2]) / len).clamp(-1.0, 1.0);
        // VU1 setup 0x3e0..0x5e0: light matrix rows 21..24 = q2 (+0x140) * 1/127 transposed, the normal
        // is V4-8 (n * 127) -> c = n . dir (cos of the full angle; FUN_001a38a8's cos(deg/2) fit means the
        // light's angle field is a full cone angle)
        let c = cosf;
        let term = (d.du - d.ru * (c + c * c * c)).max(0.0) * att;
        for k in 0..3 {
            l[k] += VU1_LIGHT_SCALE * d.color[k] * term;
        }
        lw += VU1_LIGHT_SCALE * d.k13c * term;
    }
    (l, lw)
}

/// LOOK: vf11.w of the level's VU1 row 10 = world +0x3e34 = +0x3e24 / 255 (FUN_0020f1c8; field 915,
/// setter FUN_0020f2d8, default 0). The kinds-1/7 program adds it to Lw (`ADD.w vf10, vf10, vf11` at
/// 0x10e0). Not ported: the +0x3e38 variant (world +0x3bf4 & 0x20, FUN_0011d948(w + 0.3, 0x2bbb60)).
pub fn level_shadow_fill_w(w: &World) -> f32 {
    use crate::engine::world::props::PropValue;
    let mut v = 0.0f32;
    for p in &w.props_3dd490 {
        if let (915, PropValue::Scalar(b)) = (p.field, &p.value) {
            if let Some(x) = b.get(0..4) {
                v = f32::from_le_bytes(x.try_into().unwrap());
            }
        }
    }
    v * 0.003921569
}

/// LOOK: the level's VU1 shadow fill (vf11 = row 10, `material::two_layer_vu_fill`): FUN_0020f1c8
/// (0x0020f1c8) sets world +0x3e28..+0x3e30 = hsv(+0x3e10, +0x3e14/255, +0x3e18/255) * +0x3e20/255
/// (FUN_0011dd90), uploaded to VU row 10 by FUN_00157cf0. World field-table setters: 914 = hsv vector
/// (FUN_0020f2b8, +0x3e10), 916 = intensity (FUN_0020f2f8, +0x3e20); defaults from the world init (game::q_wires1i2)
/// (hsv 180/255/255, intensity 90). Cellar (260, 180, 180) * 110 -> (0.161, 0.090, 0.304) = the
/// vu1Memory row 10 of the PCSX2 savestate.
pub fn level_shadow_fill(w: &World) -> [f32; 3] {
    use crate::engine::world::props::PropValue;
    let (mut hsv, mut k) = ([180.0f32, 255.0, 255.0], 90.0f32);
    for p in &w.props_3dd490 {
        let PropValue::Scalar(b) = &p.value else { continue };
        let f = |i: usize| b.get(4 * i..4 * i + 4).map(|x| f32::from_le_bytes(x.try_into().unwrap()));
        match p.field {
            914 => {
                if let (Some(h), Some(s), Some(v)) = (f(0), f(1), f(2)) {
                    hsv = [h, s, v];
                }
            }
            916 => k = f(0).unwrap_or(k),
            _ => {}
        }
    }
    let c = super::fog::ee_sub_11dd90([hsv[0], hsv[1] * 0.003921569, hsv[2] * 0.003921569]);
    c.map(|x| x * k * 0.003921569)
}
