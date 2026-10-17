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
        152 => p.hsv = [f32_at(b, 0), f32_at(b, 1), f32_at(b, 2)],                                      // 0x001a35f0
        153 => p.g150 = f32_at(b, 0),                                                                    // 0x001a3610
        154 => p.angle = f32_at(b, 0),                                                                   // 0x001a3630
        155 => p.falloff = [f32_at(b, 0), f32_at(b, 1)],                                                 // 0x001a3650
        156 => p.b158 = b.first().copied().unwrap_or(0),                                                 // direct u8 +0x158
        // 0x001a3680 / 0x001a36d8: setting one clears the other
        157 => {
            p.excluded = b.first().copied().unwrap_or(0) != 0;
            if p.excluded {
                p.dynamic = false;
            }
        }
        158 => {
            p.dynamic = b.first().copied().unwrap_or(0) != 0;
            if p.dynamic {
                p.excluded = false;
            }
        }
        _ => return false,
    }
    true
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
        let Payload::Loaded(LoadData::L231170 { props, .. }) = &o.payload else { continue };
        let mut p = ee_sub_1a3360();
        apply_props(&mut p, props);
        let world = bones.xfm_world(w, i).unwrap_or(super::scene::IDENTITY);
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

/// display-space factor -> linear factor (`PS2_GAMMA`), per channel, negatives clamped.
pub fn ps2_linear(x: f32) -> f32 {
    x.max(0.0).powf(PS2_GAMMA)
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
