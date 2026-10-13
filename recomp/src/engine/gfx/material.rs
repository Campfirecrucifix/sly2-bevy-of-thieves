//! WP-C: shader GS state -> material description. FUN_001eb960 builds per-shader GS register
//! packets (TEST_1 = 0x47, ALPHA_1 = 0x42, FBA = 0x3d, CLAMP_1 = 8, TEX1_1 = 0x14). Only the
//! state of the lit base pass is ported 1:1 (`ee_sub_1eb960_base`); the extra passes
//! (additive CLUT-0 pass of kind 1, kind 3 environment passes, kind 7 saturation pass) are
//! multi-pass GS tricks replaced by the Bevy PBR lighting (Solari-compatible, no custom shader).

use super::texture::Shader;

/// TEST_1 / ALPHA_1 register values of the base pass (exact values written by FUN_001eb960).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GsBaseState {
    pub test: u64,
    pub alpha: u64,
    /// pass is textured (FUN_001ea750 called)
    pub textured: bool,
}

/// FUN_001eb960, first pass of each shader kind.
pub fn ee_sub_1eb960_base(kind: u8, flags: u8) -> Option<GsBaseState> {
    // 0x001eb960
    let blend = |f: u8| ((if f & 1 != 0 { 2u64 } else { 1 }) << 2) | 0x40;
    Some(match kind {
        0 | 4 | 5 | 6 => {
            let z = if kind != 0 { 0x20000 } else { 0x60000 };
            let afail = if flags & 4 != 0 { 0x13000 } else { 0x11000 };
            let aref = if kind == 0 { 0x7eb } else { 0x7e1 };
            if flags & 0x20 != 0 {
                GsBaseState { test: (if flags & 4 != 0 { 0x13001 } else { 0x11001 }) | z, alpha: 0, textured: true }
            } else if flags & 0x10 != 0 {
                GsBaseState { test: aref | afail | z, alpha: (((flags & 1) as u64) << 3) | 1, textured: true }
            } else if flags & 0x80 == 0 {
                GsBaseState { test: aref | afail | z, alpha: blend(flags), textured: true }
            } else {
                GsBaseState { test: 0x77000, alpha: 0x54, textured: true }
            }
        }
        1 => {
            if flags & 0x80 == 0 {
                GsBaseState { test: if flags & 4 == 0 { 0x717eb } else { 0x737eb }, alpha: blend(flags), textured: true }
            } else {
                GsBaseState { test: 0x77000, alpha: 0x54, textured: true }
            }
        }
        2 => GsBaseState { test: if flags & 8 == 0 { 0x75001 } else { 0x71001 }, alpha: 0x8000000062, textured: false },
        3 => {
            if flags & 2 == 0 {
                GsBaseState { test: 0x71001, alpha: 0x6a, textured: false }
            } else {
                GsBaseState { test: 0x71001, alpha: blend(flags), textured: true }
            }
        }
        7 => {
            if flags & 0x20 == 0 {
                GsBaseState { test: 0x5000d, alpha: 0x40, textured: true }
            } else {
                GsBaseState { test: 0x53001, alpha: blend(flags), textured: true }
            }
        }
        _ => return None,
    })
}

/// Bevy-independent alpha mode (mapped to `bevy::prelude::AlphaMode` in gfx::game).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Alpha {
    Opaque,
    /// GS alpha test ATST GEQUAL/GREATER AREF -> cutoff in 0..1 (PS2 alpha scale)
    Mask(f32),
    Blend,
    Add,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MaterialDesc {
    pub alpha: Alpha,
    /// the GS never culls; back-face culling is decided per mesh primitive by the VU1 code
    /// (`vu1_cull_back`, `MeshPrim::cull_back`), so the shader state itself is double-sided.
    pub double_sided: bool,
    /// CLAMP_1 WMS / WMT clamp (texture frame flags bits 0/1)
    pub clamp_u: bool,
    pub clamp_v: bool,
    pub textured: bool,
    /// ZTE/ZTST from TEST_1: false = depth test ALWAYS (no depth ordering)
    pub depth_test: bool,
}

/// Decode GS state into a material. ALPHA_1 = (A,B,C,D) 2-bit selectors at bits 0,2,4,6:
/// 0x44 (Cs-Cd)*As+Cd = blend; 0x48 Cs*As+Cd = additive; 0 = Cs (opaque).
pub fn material_from(sh: &Shader, texture_has_alpha: bool) -> MaterialDesc {
    let st = ee_sub_1eb960_base(sh.kind, sh.flags).unwrap_or(GsBaseState { test: 0, alpha: 0, textured: true });
    let a = st.alpha & 0xff;
    let ate = st.test & 1 != 0;
    let atst = (st.test >> 1) & 7;
    let aref = ((st.test >> 4) & 0xff) as f32 / 128.0;
    let afail = (st.test >> 12) & 3;
    let alpha = match a {
        0x48 => Alpha::Add,
        0x00 => Alpha::Opaque,
        _ if !texture_has_alpha => Alpha::Opaque,
        // alpha test that discards (AFAIL KEEP) -> cut-out; FB_ONLY/RGB_ONLY keep drawing the
        // failing texels without Z = blended edges
        _ if ate && (atst == 5 || atst == 6) && afail == 0 => Alpha::Mask(aref.min(1.0)),
        _ => Alpha::Blend,
    };
    let frame = sh.frames.first();
    MaterialDesc {
        alpha,
        double_sided: true,
        clamp_u: frame.is_some_and(|f| f.flags & 1 != 0),
        clamp_v: frame.is_some_and(|f| f.flags & 2 != 0),
        textured: st.textured,
        depth_test: (st.test >> 17) & 3 != 1,
    }
}

/// FUN_0016a000 (0x0016a000): mesh type-0 flag byte bit 0x10 sets bit 0x80 in the VU1 batch
/// header word +8 (`*(u16*)(hdr+8) |= 0x80`). The type-0 VU1 program computes the screen-space
/// winding sign per strip triangle and adds 0x4000 (ADC = no draw) to the vertex when the sign
/// is not negative, unless that header bit is set. => flags & 0x10 == 0: back faces culled;
/// flags & 0x10 != 0: drawn double-sided. Types 1/2/3 run other VU1 programs (not culled here).
pub fn vu1_cull_back(mesh0_flags: u8) -> bool {
    mesh0_flags & 0x10 == 0
}

// ---------------------------------------------------------------- WP-C3: all passes

/// One GS pass of FUN_001eb960 (0x001eb960), in draw order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GsPass {
    /// TEST_1 (0x47)
    pub test: u64,
    /// ALPHA_1 (0x42)
    pub alpha: u64,
    /// FUN_001ea750 CLUT set (None = untextured pass)
    pub clut_set: Option<usize>,
    /// first FUN_00166b10 register / value pair (VU1 program setup: 0x7c/0x4f12 = lit,
    /// 0x7c/0x41f2 = unlit, 0x4c/0x4fff = untextured, 0x6c/0x4fff = env texgen, 0x44/0x41 = kind 2)
    pub vu_reg: u8,
    pub vu_val: u16,
    /// RGBAQ (reg 1) written by the pass, if any
    pub rgbaq: Option<u64>,
    /// FBA_1 (0x3d) written (value = DAT_002de18c, or 0)
    pub fba: Option<bool>,
}

fn blend(flags: u8) -> u64 {
    ((if flags & 1 != 0 { 2u64 } else { 1 }) << 2) | 0x40
}

/// FUN_001eb960 (0x001eb960): every pass of a shader (base pass identical to `ee_sub_1eb960_base`).
/// `colors` = shader head +8/+0xc/+0x10 (kind 3 tint + alpha).
pub fn ee_sub_1eb960_passes(kind: u8, flags: u8, colors: &[[u8; 4]; 3]) -> Vec<GsPass> {
    let Some(b) = ee_sub_1eb960_base(kind, flags) else { return vec![] };
    let lit = |test, alpha, set| GsPass { test, alpha, clut_set: set, vu_reg: 0x7c, vu_val: 0x4f12, rgbaq: None, fba: None };
    match kind {
        0 | 4 | 5 | 6 => vec![lit(b.test, b.alpha, Some(0))],
        1 => vec![
            GsPass { fba: Some(true), ..lit(b.test, b.alpha, Some(1)) },
            // additive unlit CLUT-0 layer
            if flags & 0x80 == 0 {
                GsPass { vu_val: 0x41f2, ..lit(0x51001, 0x48, Some(0)) }
            } else {
                GsPass { vu_val: 0x41f2, ..lit(0x57000, 0x58, Some(0)) }
            },
        ],
        2 => vec![
            GsPass { test: b.test, alpha: b.alpha, clut_set: None, vu_reg: 0x44, vu_val: 0x41, rgbaq: None, fba: None },
            GsPass { test: if flags & 8 == 0 { 0x35001 } else { 0x31001 }, alpha: b.alpha, clut_set: None, vu_reg: 0x44, vu_val: 0x41, rgbaq: None, fba: None },
        ],
        3 => {
            let tint = |c: [u8; 4]| c.map(|x| ((x as u32 + 1) >> 1) as u64);
            let t = tint(colors[1]);
            let c2 = u32::from_le_bytes(colors[2]) as u64;
            let a = if flags & 2 != 0 { c2 } else { c2 & 0xffffff };
            let first = if flags & 2 == 0 {
                GsPass { test: 0x71001, alpha: 0x6a, clut_set: None, vu_reg: 0x4c, vu_val: 0x4fff, rgbaq: Some(0x100000020), fba: None }
            } else {
                GsPass { test: 0x71001, alpha: blend(flags), clut_set: Some(0), vu_reg: 0x7c, vu_val: 0x4f12, rgbaq: None, fba: Some(true) }
            };
            vec![
                first,
                GsPass { test: 0x71001, alpha: 0x6a, clut_set: None, vu_reg: 0x4c, vu_val: 0x4fff, rgbaq: Some(0x180002000), fba: None },
                // environment pass: VU1 texgen (0x6c), tinted by (head+0xc + 1) >> 1
                GsPass { test: 0x75001, alpha: blend(flags), clut_set: None, vu_reg: 0x6c, vu_val: 0x4fff, rgbaq: Some(t[0] | t[1] << 8 | t[2] << 16 | t[3] << 24 | 0x3f800000 << 32), fba: Some(true) },
                // final pass: RGBAQ = head+0x10 (alpha byte only with flags & 2)
                GsPass { test: 0x75001, alpha: blend(flags), clut_set: None, vu_reg: 0x4d, vu_val: 4, rgbaq: Some(a | 0x100000000), fba: None },
            ]
        }
        7 => {
            let mut v = Vec::new();
            if flags & 0x20 == 0 {
                // Z / alpha pre-pass (colour = Cd), CLUT set 2
                v.push(GsPass { vu_val: 0x41f2, fba: Some(false), ..lit(0x5000d, 0x40, Some(2)) });
            }
            v.push(GsPass { fba: Some(true), ..lit(0x53001, blend(flags), Some(1)) });
            v.push(GsPass { vu_val: 0x41f2, fba: Some(false), ..lit(0x53001, 0x48, Some(0)) });
            v
        }
        _ => vec![],
    }
}

/// StandardMaterial-level description of the extra passes (no custom shaders).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MaterialLayers {
    /// CLUT set of the lit diffuse pass (base_color_texture)
    pub base_set: usize,
    /// additive unlit pass (ALPHA 0x48/0x58, VU 0x41f2) -> emissive_texture with this CLUT set
    pub emissive_set: Option<usize>,
    /// kind 3 environment pass -> reflectance / env map, tint 0..1 ((c+1)>>1 / 128)
    pub env_tint: Option<[f32; 4]>,
    /// kind 2 subtractive (Cd - Cs * FIX) -> AlphaMode::Multiply darkening
    pub subtractive: bool,
}

/// WP-A5: a kind-3 shader without flags & 2 (FUN_001eb960 case 3) draws no colour of its own in
/// passes 1/2 (ALPHA 0x6a = Cd, untextured, no Z write: destination-alpha marking only) and pass 4
/// (RGBAQ head+0x10 with the alpha byte cleared -> As = 0 adds nothing); the only visible pass is
/// the VU1 texgen environment pass (0x6c/0x4fff, TEST 0x75001 = no Z write) tinted by the constant
/// RGBAQ (head+0xc + 1) >> 1 and blended with ALPHA `blend(flags)`: flags & 1 -> Cs*As + Cd
/// (additive), else (Cs-Cd)*As + Cd. Guards' vision cones (class 58 family, tints like
/// (250,10,10,40) / (250,220,10,30)) and glass/glow shells use it. Returns (tint 0..1 incl. alpha,
/// additive). Shaders with flags & 2 draw a textured base pass instead (-> None, normal material).
pub fn env_glow(sh: &Shader) -> Option<([f32; 4], bool)> {
    if sh.kind != 3 || sh.flags & 2 != 0 {
        return None;
    }
    let t = layers(sh).env_tint?;
    Some((t.map(|x| x.min(1.0)), sh.flags & 1 != 0))
}

/// WP-A9: the screen result of a kind-3 env-only shader (`env_glow`) on the PS2 is a VOLUME tint,
/// not a translucent shell. FUN_001eb960 case 3: passes 1 and 2 draw the mesh with TEST 0x71001
/// (ATE, ATST NEVER, AFAIL FB_ONLY = no Z write, ZTST GREATER) and ALPHA 0x6a (= Cd): they only
/// write destination alpha (RGBAQ alpha 0x00, then 0x80) from two different VU1 packet ranges
/// (inferred front / back faces: with the same faces the mask would always be empty); the visible env pass has TEST 0x75001 = the same plus DATE with DATM 0, so
/// it adds `Cs*As` (flags & 1) only where the destination alpha MSB is still 0: pixels where the
/// front face is in front of the scene but the back face is hidden, i.e. scene surfaces INSIDE the
/// volume (a red neon-glow box over a building on table 14, glow shells around props). Bevy has
/// no stencil / destination-alpha test, so a static level volume (class 48) is REPLACED by a point
/// light at the prim's bounds centre (range = bounds radius, colour = tint, strength = tint * As)
/// that tints exactly the surfaces near / inside the volume, in raster and RT alike; the shell
/// itself is not drawn. Returns (local centre, radius, linear colour (max 1), strength).
pub fn env_volume_light(sh: &Shader, positions: &[[f32; 3]]) -> Option<([f32; 3], f32, [f32; 3], f32)> {
    let (t, _) = env_glow(sh)?;
    let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
    for p in positions {
        for k in 0..3 {
            lo[k] = lo[k].min(p[k]);
            hi[k] = hi[k].max(p[k]);
        }
    }
    if positions.is_empty() {
        return None;
    }
    let c = [0, 1, 2].map(|k| (lo[k] + hi[k]) * 0.5);
    let r = [0, 1, 2].map(|k| (hi[k] - lo[k]) * 0.5).iter().map(|x| x * x).sum::<f32>().sqrt();
    let m = t[0].max(t[1]).max(t[2]);
    if m <= 0.0 || t[3] <= 0.0 || r <= 0.0 {
        return None;
    }
    Some((c, r, [t[0] / m, t[1] / m, t[2] / m], m * t[3]))
}

/// WP-A9: kinds 1/7 two-layer surfaces (grey CLUT set 1 lit pass + additive CLUT set 0 colour
/// pass, J3). VU1 program 0x1050..0x1320 (overlay 376110651) per vertex, with L = lit rgb, Lw = lit
/// w (light colour w channel), vf31 = 2 (ADD.w vf0 + vf0): h = max(0, 2 - max(L)), k = min(Lw, h),
/// r = h - k; pass 1 (GIF 0x4f12, RGBAQ slot 1 = vf30) = grey * (min(L, 2) + vf11 * r) * n.w,
/// pass 2 (GIF 0x41f2, slot 2 = vf29) = colour * vertex colour * k with As = (128 - 63.5 r) / 128,
/// added (ALPHA 0x48 Cs*As + Cd). Fully lit (Lw = max(L) = 1 -> r = 0) the PS2 shows
/// `grey * L + colour * vcol * L`: the colour is ADDED to the lit grey, it is not the albedo.
/// REPLACED by one albedo texture `grey + colour * a0` (per texel, saturating like the GS
/// COLCLAMP) lit by the Bevy lights, used by raster AND RT (A10: the vertex colour hue applies to
/// the colour layer only, `two_layer_vertex_color`; the vf11 fill is 0 and the colour follows the
/// light linearly, `two_layer_vu`). `base`/`colour` = RGBA8
/// images of the same size (alpha rescaled, 255 = 1.0); the base alpha is kept.
pub fn two_layer_albedo(base: &mut [u8], colour: &[u8]) {
    for (b, c) in base.chunks_exact_mut(4).zip(colour.chunks_exact(4)) {
        for k in 0..3 {
            b[k] = (b[k] as u32 + (c[k] as u32 * c[3] as u32 + 127) / 255).min(255) as u8;
        }
    }
}

pub fn layers(sh: &Shader) -> MaterialLayers {
    let passes = ee_sub_1eb960_passes(sh.kind, sh.flags, &sh.colors);
    let base_set = sh.base_pass();
    let emissive_set = passes.iter().skip(1).find(|p| p.vu_val == 0x41f2 && (p.alpha & 0xff == 0x48 || p.alpha & 0xff == 0x58)).and_then(|p| p.clut_set);
    let env_tint = passes.iter().find(|p| p.vu_reg == 0x6c).and_then(|p| p.rgbaq).map(|q| [0, 8, 16, 24].map(|s| ((q >> s) & 0xff) as f32 / 128.0));
    MaterialLayers { base_set, emissive_set, env_tint, subtractive: sh.kind == 2 }
}

/// WP-A10: VU1 row 10 as the kind-1/7 program reads it (`LQ vf11, 10(vi0)` at 0x10a8, used as the
/// shadow fill `vf30 = min(L, 2) + vf11 * r` at 0x1240). Traced: FUN_0016a000 (mesh load) unpacks
/// the batch header there (FUN_00226e88(.., 0xd = V4-16, 0xb quads, address 10), memset 0 then s16
/// fields: +2 = count, +6 = index, +8.. = flags in row 11), i.e. small INTEGERS; read as floats they
/// are 0 / denormals, which the VU flushes to zero -> the fill is 0 (no grey fill in shadow).
pub const VF11_ROW10_FILL: [f32; 3] = [0.0; 3];

/// WP-A10: per-vertex terms of the kinds-1/7 VU1 program (overlay 376110651 0x1050..0x1320) for lit
/// `l` (vf23.xyz) and `lw` (vf23.w): returns (grey multiplier vf30.xyz before n.w, colour factor k).
/// h = max(0, 2 - max(l)), k = min(lw, h), r = h - k; vf30 = min(l, 2) + vf11 * r (vf11 = 0, see
/// `VF11_ROW10_FILL`). The colour pass writes Cv = vcol * k * Q with Q = va / (128 - 63.5 r) (DIV at
/// 0x1230: vf27.w / vf28.w) and Av = 128 - 63.5 r, so in Cs * As (ALPHA 0x48) the (128 - 63.5 r)
/// cancels: the colour layer is `colour * vcol * k`, LINEAR in the light (~L in shadow, k = Lw), not
/// ~L^2 as A9 assumed. Both layers therefore follow the light linearly in the normal range (L <= 1),
/// which is what the Bevy lights (raster) and Solari (RT) do with one lit albedo; above 1 the PS2
/// grey saturates at 2 while the colour fades with h = 2 - max(L) (overbright, not reproduced).
pub fn two_layer_vu(l: [f32; 3], lw: f32) -> ([f32; 3], f32) {
    let m = l[0].max(l[1]).max(l[2]);
    let h = (2.0 - m).max(0.0);
    let k = lw.min(h);
    let r = h - k;
    ([0, 1, 2].map(|c| l[c].min(2.0) + VF11_ROW10_FILL[c] * r), k)
}

/// WP-A10: grey-pass vertex weight n.w. FUN_0016a000 packs the vertex normal as V4-8
/// (`n * 127.0`) with w = `(char)(max(r, g, b) * 0.003921569 * 127.0)` of the vertex colour bytes;
/// the VU multiplies the grey pass by it (MULw.xyz vf30, vf30, vf19 at 0x1280) and the GS modulates
/// by /128. So the grey layer only gets the vertex colour's brightness, never its hue; the hue
/// (full rgb vertex colour) is applied to the colour pass only. `vcol` in `mesh::ps2_color` scale
/// (byte / 128). Returns the weight in the same scale (1.0 = GS 0x80).
pub fn two_layer_grey_weight(vcol: [f32; 4]) -> f32 {
    let b = |x: f32| (x * 128.0).round().clamp(0.0, 255.0);
    let m = b(vcol[0]).max(b(vcol[1])).max(b(vcol[2]));
    ((m * 0.003921569 * 127.0) as i32) as f32 / 128.0
}

/// WP-A10: channel means of the two layers of a folded kind-1/7 texture (grey CLUT set 1, colour
/// CLUT set 0 weighted by its alpha like `two_layer_albedo`), 0..255.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TwoLayerMeans {
    pub grey: [f32; 3],
    pub colour: [f32; 3],
}

pub fn two_layer_means(grey: &[u8], colour: &[u8]) -> TwoLayerMeans {
    let (mut g, mut c, mut n) = ([0f64; 3], [0f64; 3], 0f64);
    for (b, s) in grey.chunks_exact(4).zip(colour.chunks_exact(4)) {
        for k in 0..3 {
            g[k] += b[k] as f64;
            c[k] += s[k] as f64 * s[3] as f64 / 255.0;
        }
        n += 1.0;
    }
    let n = n.max(1.0);
    TwoLayerMeans { grey: g.map(|x| (x / n) as f32), colour: c.map(|x| (x / n) as f32) }
}

/// WP-A10: COLOR attribute for a folded two-layer prim. The material albedo is `grey + colour`
/// (`two_layer_albedo`) and Bevy multiplies it by COLOR, which tinted the grey layer with the
/// vertex colour too (table 14: class-89 water oid 39702, grey 36 + colour (33, 18, 53) times
/// vertex colours ~(1.7, 0.23, 1.23) -> saturated magenta). The PS2 result per vertex is
/// `grey * n.w + colour * vcol` (`two_layer_grey_weight`, `two_layer_vu`), so the per-channel
/// factor is `(G * n.w + C * vcol) / (G + C)` over the layer means: exact for flat layers (the
/// water), the mean ratio otherwise. Alpha is kept.
pub fn two_layer_vertex_color(vcol: [f32; 4], m: &TwoLayerMeans) -> [f32; 4] {
    let gw = two_layer_grey_weight(vcol);
    let mut o = vcol;
    for k in 0..3 {
        let d = m.grey[k] + m.colour[k];
        o[k] = if d > 0.0 { (m.grey[k] * gw + m.colour[k] * vcol[k]) / d } else { vcol[k] };
    }
    o
}
