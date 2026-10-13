//! WP-C2: camera fog data/colour path. FUN_0014e270 turns the CM fog fields into GS FOG
//! coefficients (linear in 1/w, GS fog 0 = full fog colour, 0xff (x16 = 4080) = none);
//! FUN_001ed278 converts the u8 fog colour (optional grey "flashback" filter). The GS FOGCOL /
//! per-vertex fog is replaced by Bevy `DistanceFog` (`distance_fog`, feature `game`).

/// FLT_MAX at DAT_002a4ff8 = "no scale given" sentinel.
pub const FLT_MAX_SENTINEL: f32 = f32::MAX;

/// CM fog fields read by FUN_0014e270.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CmFog {
    /// cm+0x240: distance where fog starts (fog value 4080 = none)
    pub start: f32,
    /// cm+0x244: distance where fog reaches `max`
    pub end: f32,
    /// cm+0x248: fog amount at `end` (0..1)
    pub max: f32,
}

/// FUN_001a37d0 (0x001a37d0): solve f(r) = p5 + p6 / r through (a, b) and (c, d) for
/// r = a, c (here r = distance / start, so f is linear in 1/w). Degenerate -> (b, 0).
pub fn ee_sub_1a37d0(a: f32, b: f32, c: f32, d: f32) -> (f32, f32) {
    // 0x001a37d0; FUN_0011d4d0(a, c, 1e-4) = "nearly equal"
    let eps = 0.0001;
    if (a - c).abs() >= eps && a.abs() >= eps && c.abs() >= eps {
        let k = (a * c * (d - b)) / (a - c);
        (b - k / a, k)
    } else {
        (b, 0.0)
    }
}

/// FUN_0014e270 (0x0014e270) output: [base + 8417280 (float->int magic), k, start, base + magic + 8192].
pub fn ee_sub_14e270(scale: f32, cm: &CmFog) -> [f32; 4] {
    let scale = if scale == FLT_MAX_SENTINEL { 1.0 } else { scale };
    let (base, k) = ee_sub_1a37d0(1.0, 4080.0, cm.end / cm.start, (1.0 - cm.max * scale) * 4080.0);
    let b = base + 8.41728e6;
    [b, k, cm.start, b + 8192.0]
}

/// GS fog value (0..4080, 4080 = no fog) at view distance `dist` from FUN_0014e270's output.
pub fn gs_fog_value(coef: &[f32; 4], dist: f32) -> f32 {
    let base = coef[0] - 8.41728e6;
    (base + coef[1] * coef[2] / dist.max(1e-6)).clamp(0.0, 4080.0)
}

/// Fog amount 0..1 (1 = full fog colour) at distance `dist`.
pub fn fog_amount(scale: f32, cm: &CmFog, dist: f32) -> f32 {
    if dist <= cm.start {
        return 0.0;
    }
    1.0 - gs_fog_value(&ee_sub_14e270(scale, cm), dist) / 4080.0
}

/// FUN_0011dcd8: u8 rgb -> 0..1 (x 1/255).
pub fn ee_sub_11dcd8(c: [u8; 4]) -> [f32; 3] {
    [c[0], c[1], c[2]].map(|x| x as f32 * 0.003921569)
}

/// FUN_0011dd30: 0..1 rgb + alpha byte -> u8 (round half up, truncating cast).
pub fn ee_sub_11dd30(c: [f32; 3], a: u8) -> [u8; 4] {
    let f = |x: f32| (x * 255.0 + 0.5) as i32 as u8;
    [f(c[0]), f(c[1]), f(c[2]), a]
}

/// FUN_0011dd90: HSV (h degrees, s, v) -> RGB, clamps h to [0,360] (negative -> -1), s/v to [0,1].
pub fn ee_sub_11dd90(hsv: [f32; 3]) -> [f32; 3] {
    let h = if hsv[0] < 0.0 { -1.0 } else { hsv[0].min(360.0) };
    let s = hsv[1].clamp(0.0, 1.0);
    let v = hsv[2].clamp(0.0, 1.0);
    if s == 0.0 {
        return [v, v, v];
    }
    let h = if h == 360.0 { 0.0 } else { h } * 0.016666668;
    let i = h.floor();
    let f = h - i as i32 as f32;
    let (p, q, t) = (v * (1.0 - s), v * (1.0 - s * f), v * (1.0 - s * (1.0 - f)));
    match i as i32 {
        0 | 6 => [v, t, p],
        1 => [q, v, p],
        2 => [p, v, t],
        3 => [p, q, v],
        4 => [t, p, v],
        5 => [v, p, q],
        _ => [0.0; 3], // original leaves the output untouched (h = -1)
    }
}

/// FUN_001ed2d8 (0x001ed2d8): luminance (0.3, 0.6, 0.1), v = clamp(L^0.6, 0..1) (DAT_002c3698,
/// range 0x2bbb60), rgb = hsv(60 (DAT_002c3690), 0 (DAT_002c3694), v) = grey.
pub fn ee_sub_1ed2d8(c: [f32; 3]) -> [f32; 3] {
    let l = c[0] * 0.3 + c[1] * 0.6 + c[2] * 0.1;
    let v = l.powf(0.6);
    let v = if (0.0..=1.0).contains(&v) { v } else if v < 0.0 { 0.0 } else { 1.0 };
    ee_sub_11dd90([60.0, 0.0, v])
}

/// FUN_001ed278 (0x001ed278): applied only while DAT_003e12c8 (grey filter mode) is set;
/// otherwise the colour is returned unchanged.
pub fn ee_sub_1ed278(c: [u8; 4], grey_mode: bool) -> [u8; 4] {
    if !grey_mode {
        return c;
    }
    ee_sub_11dd30(ee_sub_1ed2d8(ee_sub_11dcd8(c)), c[3])
}

/// Bevy fog. Replacement for the GS fog: linear falloff between `start` and `end` with the
/// colour alpha = amount at `end` (the GS curve is hyperbolic in 1/w between the same two
/// points; `fog_amount` gives the exact original curve).
#[cfg(feature = "game")]
pub fn distance_fog(cm: &CmFog, scale: f32, color: [u8; 4], grey_mode: bool) -> bevy::pbr::DistanceFog {
    use bevy::prelude::*;
    let c = ee_sub_1ed278(color, grey_mode);
    let amount = fog_amount(scale, cm, cm.end).clamp(0.0, 1.0);
    bevy::pbr::DistanceFog {
        color: Color::srgba_u8(c[0], c[1], c[2], (amount * 255.0) as u8),
        falloff: bevy::pbr::FogFalloff::Linear { start: cm.start, end: cm.end.max(cm.start + 1e-3) },
        ..default()
    }
}

/// WP-C7: the level's g_cm fog state at mesh-load time. The level stream applies the property
/// list FUN_0013f810(0x2ddf40) (`World::props_2ddf40`) to g_cm before any object loads; the fog
/// setters are field-table 0x2ac880 entries 166 (+0x240 near, FUN_0014f690), 167 (+0x244 far,
/// FUN_0014f6b0), 169 (+0x248 max, FUN_0014f6d0) and 168 (+0x24c rgba, FUN_0014f6f0), each a
/// plain store (+ FUN_0014db70 re-deriving the GS registers). Starting values = InitCm
/// FUN_0014d6d8 (500, 20000, 0.5, 0x80303030). 42 of 43 levels set all four (t1 none).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LevelFog {
    pub fog: CmFog,
    /// cm+0x24c as stored (bytes r, g, b, a = 0x80 in every level)
    pub rgba: [u8; 4],
}

impl Default for LevelFog {
    fn default() -> Self {
        let d = crate::engine::camera::cm::FOG_DEFAULTS;
        LevelFog { fog: CmFog { start: d[0], end: d[1], max: d[2] }, rgba: 0x80303030u32.to_le_bytes() }
    }
}

/// Apply the g_cm fog fields of a level property list in stream order (other fields = camera).
pub fn level_fog(props: &[crate::engine::world::props::Prop]) -> LevelFog {
    use crate::engine::world::props::PropValue;
    let mut f = LevelFog::default();
    for p in props {
        let PropValue::Scalar(b) = &p.value else { continue };
        let Some(w) = b.get(..4).map(|x| <[u8; 4]>::try_from(x).unwrap()) else { continue };
        match p.field {
            166 => f.fog.start = f32::from_le_bytes(w), // 0x0014f690
            167 => f.fog.end = f32::from_le_bytes(w),   // 0x0014f6b0
            169 => f.fog.max = f32::from_le_bytes(w),   // 0x0014f6d0
            168 => f.rgba = w,                          // 0x0014f6f0
            _ => {}
        }
    }
    f
}

/// WP-C7: per-mesh-record fog scale. FUN_00167320 reads a f32 per record (`MeshRecord::f1`) and
/// FUN_0016a000 / FUN_0016e438 pass it to FUN_0014e270(scale, g_cm) when baking the record's GS
/// fog coefficients: FLT_MAX = default (1), 0 = no fog (distant backdrop buildings, class 97/75
/// cards), 0.3..0.7 = partial (fog amount at `end` = max * scale). Returns the effective scale.
pub fn record_fog_scale(f1: f32) -> f32 {
    if f1 == FLT_MAX_SENTINEL { 1.0 } else { f1 }
}

/// Bevy has one fog per camera and only an on/off switch per material (`fog_enabled`), so a
/// record's partial scale is REPLACED by on when the scale is at least one half.
pub fn fog_enabled_for(scale: f32) -> bool {
    scale >= 0.5
}
