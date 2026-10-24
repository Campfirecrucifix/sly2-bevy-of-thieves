//! LOOK: static shadow volumes (mesh type 3 closed strips, `MeshRole::ShadowVolume`) -> decal pass.
//!
//! PS2 (FUN_0016da48 / VU1 overlay 188054619 MSCAL 0x1aa0 / FUN_0016dd58), confirmed against the
//! user's single-frame PCSX2 GS dump of the cellar (shots/LOOK/user/cellar_floor_band.gs.zst,
//! tools/gs/gsdump.py: draws 523..719):
//! - clear: sprite strips, FRAME FBMSK 0xffff00ff (green byte only), green = 0x3e (0x7e for pass 7);
//! - volume faces (kind-2 shader, FUN_001eb960 case 2): ALPHA A2B0C2D1 FIX 128 = Cd - Cs (green
//!   only, colour wrap), TEST 0x75001 (Z GEQUAL, no Z write), face green 2 or 254 (VU 0x09f0
//!   MFIR 2 / 254 by screen winding) -> the green byte counts the volumes enclosing each visible
//!   pixel; an inside-out mesh counts the other way (the cellar's window light shafts cancel the
//!   room-wide darkness volume: that is the "light band", the grate bars are where it is not cancelled);
//! - pass 8: FBMSK 0xfffffeff + ALPHA A1B2C2D2 FIX 2 (Cd * 2 / 128) copies green bit 6 into bit 0;
//! - resolve: the frame's green channel read as an 8-bit texture through the FUN_001ea9f0 CLUT (world
//!   +0x3e70/+0x3e80/+0x3e90 HSV colours = fields 917/919/921, alphas +0x3e64/68/6c = 918/920/922,
//!   defaults colour 0 / alphas 128, 96, 160), ALPHA A0B1C0D1 = lerp(Cd, CLUT.rgb, CLUT.a). The dump's
//!   VRAM CLUT at CBP 0x1800 is black with alpha bytes <= 0x30 (= colour 1 black, a1 96) in the cellar.
//!
//! Port: the per-pixel count is evaluated on the static receivers (adaptive split of each surface
//! triangle until the signed count is uniform, ray-parity inside test) and the visible CLUT entry is
//! drawn as an unlit blended decal (As per vertex). Not ported: type-2 edge volumes (extruded from
//! the light per frame by the VU), dynamic receivers (Sly / NPCs walking through a volume) and
//! volumes on moving objects. Kill switch SLY2_LOOK_NOSHVOL=1; SLY2_LOOK_SHVOL_FLIP=1 flips the sign.

use super::scene::{mul, Affine, LevelScene};
use crate::engine::world::World;

/// world +0x3e68 default (world init, `game::q_wires1i2`): CLUT colour 1 alpha
pub const DEFAULT_ALPHA1: f32 = 96.0;

/// (colour 1 rgb 0..1 display space, a1 = colour 1 alpha 0..255) of the FUN_001ea9f0 CLUT.
pub fn resolve_colour(w: &World) -> ([f32; 3], f32) {
    use crate::engine::world::props::PropValue;
    let (mut hsv, mut a1) = ([0.0f32; 3], DEFAULT_ALPHA1);
    for p in &w.props_3dd490 {
        let PropValue::Scalar(b) = &p.value else { continue };
        let f = |i: usize| b.get(4 * i..4 * i + 4).map(|x| f32::from_le_bytes(x.try_into().unwrap()));
        match p.field {
            919 => {
                if let (Some(h), Some(s), Some(v)) = (f(0), f(1), f(2)) {
                    hsv = [h, s, v];
                }
            }
            920 => a1 = f(0).unwrap_or(a1),
            _ => {}
        }
    }
    let rgb = super::light::ee_sub_11dfa0(hsv).map(|c| (c as i32).clamp(0, 255) as f32 / 255.0);
    (rgb, a1)
}

/// GS As of the CLUT entry the resolve reads for signed count `c` (green 0x3e + 2c, pass 8 copies
/// bit 6 into bit 0; n & 0x81 == 1 -> colour 1, alpha byte ((n & 0x7f) - 1) / 126 * a1 / 255 * 128;
/// n & 0x81 == 0 -> CLUT entry 0 = transparent). Assumes receiver frame alpha bits 1..6 = 0x40
/// (`clut_index` with fa 0x40); the per-pixel resolve (shadowvol.wgsl) uses the real frame alpha.
pub fn count_alpha(c: i32, a1: f32) -> f32 {
    let n = clut_index(c, 0x40);
    if n & 0x81 != 1 {
        return 0.0;
    }
    clut_alpha(n, a1)
}

/// LOOK R24: the PSMT8H index the FUN_0016dd58 resolve (GS dump draw 719, TEX2 PSM T8H CBP 0x1800)
/// reads for a pixel with signed volume count `c` and receiver frame alpha `fa` (the As byte the
/// world draw left in the frame, = colour-pass Ta * Av >> 7): the green byte g = 0x3e + 2c; pass 8
/// (draw 716, Cd * 2 >> 7 under FBMSK 0xfffffeff) copies g bit 6 into bit 0; draws 717/718 (context
/// 2, CT16 channel shuffle) move g bits 0 and 7 into the alpha byte, whose bits 1..6 stay the
/// receiver's -> n = (fa & 0x7e) | (g' & 0x81). Verified on the L3 cellar dump (gsim23.py: median
/// error vs the PCSX2 snapshot -0.016 = the sim's base offset, MAD 0.024).
pub fn clut_index(c: i32, fa: u32) -> u32 {
    let g = (0x3e + 2 * c) as u32 & 0xff;
    let g = g | ((g >> 6) & 1);
    (fa & 0x7e) | (g & 0x81)
}

/// LOOK R24: FUN_001ea9f0 CLUT entry `n` (CSM1 order): (bit 7, bit 0) = (0, 0) transparent,
/// (0, 1) colour 1, (1, 0) colour 0, (1, 1) colour 2; alpha byte ((n & 0x7f) - 1) / 126 * a / 255 * 128
/// (a = that colour's alpha 0..255; VRAM CBP 0x1800 of the L3 dump matches entry for entry).
/// Returns the colour slot (0..2, None = transparent).
pub fn clut_slot(n: u32) -> Option<usize> {
    match (n >> 7 & 1, n & 1) {
        (0, 0) => None,
        (0, _) => Some(1),
        (_, 0) => Some(0),
        _ => Some(2),
    }
}

/// GS As (0x80 = 1.0) of CLUT entry `n` for colour alpha `a` (0..255).
pub fn clut_alpha(n: u32, a: f32) -> f32 {
    let byte = ((((n & 0x7f) as i32 - 1).max(0) as f32 * 0.007936508) * (a * 0.003921569 * 128.0)) as i32;
    byte as f32 / 128.0
}

/// LOOK R24: the three FUN_001ea9f0 CLUT colours (rgb 0..1 display space, alpha 0..255): world
/// fields 917/918 (colour 0, default black / 128), 919/920 (colour 1, default black / 96),
/// 921/922 (colour 2, default black / 160).
pub fn resolve_clut(w: &World) -> [[f32; 4]; 3] {
    use crate::engine::world::props::PropValue;
    let mut hsv = [[0.0f32; 3]; 3];
    let mut a = [128.0, DEFAULT_ALPHA1, 160.0];
    for p in &w.props_3dd490 {
        let PropValue::Scalar(b) = &p.value else { continue };
        let f = |i: usize| b.get(4 * i..4 * i + 4).map(|x| f32::from_le_bytes(x.try_into().unwrap()));
        if !(917..=922).contains(&p.field) {
            continue;
        }
        let k = ((p.field - 917) / 2) as usize;
        if (p.field - 917) % 2 == 0 {
            if let (Some(h), Some(s), Some(v)) = (f(0), f(1), f(2)) {
                hsv[k] = [h, s, v];
            }
        } else {
            a[k] = f(0).unwrap_or(a[k]);
        }
    }
    [0, 1, 2].map(|k| {
        let rgb = super::light::ee_sub_11dfa0(hsv[k]).map(|c| (c as i32).clamp(0, 255) as f32 / 255.0);
        [rgb[0], rgb[1], rgb[2], a[k]]
    })
}

fn xf(m: &Affine, v: [f32; 3]) -> [f32; 3] {
    [0, 1, 2].map(|j| v[0] * m[0][j] + v[1] * m[1][j] + v[2] * m[2][j] + m[3][j])
}
fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// A closed volume mesh (world space) with its count sign (+1 outward-wound).
#[derive(Clone, Debug)]
pub struct Volume {
    pub tris: Vec<[[f32; 3]; 3]>,
    pub sign: i32,
    pub lo: [f32; 3],
    pub hi: [f32; 3],
}

/// Volume from world-space triangles (None if open / flat: signed volume ~ 0).
pub fn volume(pos: &[[f32; 3]], idx: &[[u32; 3]]) -> Option<Volume> {
    if idx.len() < 4 {
        return None;
    }
    let tris: Vec<[[f32; 3]; 3]> = idx.iter().map(|t| t.map(|i| pos[i as usize])).collect();
    let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
    for p in pos {
        for a in 0..3 {
            lo[a] = lo[a].min(p[a]);
            hi[a] = hi[a].max(p[a]);
        }
    }
    let c = [0, 1, 2].map(|a| (lo[a] + hi[a]) * 0.5);
    let v: f32 = tris.iter().map(|t| dot(sub(t[0], c), cross(sub(t[1], c), sub(t[2], c)))).sum::<f32>() / 6.0;
    let bb = (hi[0] - lo[0]) * (hi[1] - lo[1]) * (hi[2] - lo[2]);
    let flip = if std::env::var("SLY2_LOOK_SHVOL_FLIP").is_ok() { -1 } else { 1 };
    (v.abs() > 1e-3 * bb.max(1.0)).then_some(Volume { tris, sign: if v > 0.0 { flip } else { -flip }, lo, hi })
}

impl Volume {
    /// ray parity along a skewed +z ray (avoids shared edges)
    pub fn inside(&self, p: [f32; 3]) -> bool {
        if (0..3).any(|a| p[a] < self.lo[a] || p[a] > self.hi[a]) {
            return false;
        }
        let d = [0.000_131, 0.000_217, 1.0];
        let mut n = 0;
        for t in &self.tris {
            let e1 = sub(t[1], t[0]);
            let e2 = sub(t[2], t[0]);
            let h = cross(d, e2);
            let a = dot(e1, h);
            if a.abs() < 1e-9 {
                continue;
            }
            let f = 1.0 / a;
            let s = sub(p, t[0]);
            let u = f * dot(s, h);
            if !(0.0..=1.0).contains(&u) {
                continue;
            }
            let q = cross(s, e1);
            let v = f * dot(d, q);
            if v < 0.0 || u + v > 1.0 {
                continue;
            }
            if f * dot(e2, q) > 0.0 {
                n += 1;
            }
        }
        n & 1 == 1
    }
}

/// Signed volume count at `p` (the green counter, relative to the 0x3e clear).
pub fn count(vols: &[Volume], p: [f32; 3]) -> i32 {
    vols.iter().filter(|v| v.inside(p)).map(|v| v.sign).sum()
}

fn items(scene: &LevelScene) -> Vec<(usize, Affine)> {
    (0..scene.objects.len()).map(|i| (i, scene.world_of(i))).chain(scene.copies.iter().map(|c| (c.source, c.world))).collect()
}

/// Volumes of the static type-3 shadow strips (no edge list, no skinning).
pub fn static_volumes(scene: &LevelScene) -> Vec<Volume> {
    use super::mesh::MeshRole;
    let mut vols = Vec::new();
    for (si, w) in items(scene) {
        for p in &scene.objects[si].prims {
            if p.prim.role != MeshRole::ShadowVolume || !p.prim.edges.is_empty() || p.prim.weights.iter().any(|x| !x.is_empty()) {
                continue;
            }
            let m = mul(&w, &p.local);
            let pos: Vec<[f32; 3]> = p.prim.positions.iter().map(|&v| xf(&m, v)).collect();
            let tris: Vec<[u32; 3]> = p.prim.indices.chunks_exact(3).map(|t| [t[0], t[1], t[2]]).collect();
            if let Some(v) = volume(&pos, &tris) {
                vols.push(v);
            }
        }
    }
    vols
}

/// Decal triangles (level space, PS2 Z-up), normals and per-vertex As over the static receivers.
pub fn static_decals(scene: &LevelScene, lift: f32, a1: f32, min_edge: f32) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<f32>, Vec<u32>) {
    use super::mesh::MeshRole;
    let vols = static_volumes(scene);
    let (mut vp, mut vn, mut va, mut ix) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    if vols.is_empty() {
        return (vp, vn, va, ix);
    }
    for (si, w) in items(scene) {
        for p in &scene.objects[si].prims {
            if p.prim.role != MeshRole::Surface || p.prim.weights.iter().any(|x| !x.is_empty()) {
                continue;
            }
            let m = mul(&w, &p.local);
            let pos: Vec<[f32; 3]> = p.prim.positions.iter().map(|&v| xf(&m, v)).collect();
            for t in p.prim.indices.chunks_exact(3) {
                let [Some(a), Some(b), Some(c)] = [t[0], t[1], t[2]].map(|i| pos.get(i as usize).copied()) else { continue };
                let near: Vec<Volume> = vols
                    .iter()
                    .filter(|v| !(0..3).any(|k| a[k].max(b[k]).max(c[k]) < v.lo[k] || a[k].min(b[k]).min(c[k]) > v.hi[k]))
                    .cloned()
                    .collect();
                if near.is_empty() {
                    continue;
                }
                let n = cross(sub(b, a), sub(c, a));
                let l = dot(n, n).sqrt();
                if l < 1e-6 {
                    continue;
                }
                let n = n.map(|x| x / l);
                let mut emit = |tri: [[f32; 3]; 3], al: f32| {
                    let base = vp.len() as u32;
                    for q in tri {
                        vp.push([0, 1, 2].map(|k| q[k] + n[k] * lift));
                        vn.push(n);
                        va.push(al);
                    }
                    ix.extend([base, base + 1, base + 2]);
                };
                split(&near, [a, b, c], n, a1, min_edge, 0, &mut emit);
            }
        }
    }
    (vp, vn, va, ix)
}

fn split(vols: &[Volume], t: [[f32; 3]; 3], n: [f32; 3], a1: f32, min_edge: f32, depth: u32, emit: &mut impl FnMut([[f32; 3]; 3], f32)) {
    let mid = |p: [f32; 3], q: [f32; 3]| [0, 1, 2].map(|k| (p[k] + q[k]) * 0.5);
    let (ab, bc, ca) = (mid(t[0], t[1]), mid(t[1], t[2]), mid(t[2], t[0]));
    let cen = [0, 1, 2].map(|k| (t[0][k] + t[1][k] + t[2][k]) / 3.0);
    // samples pulled 2% towards the centre so receivers sharing an edge classify alike, and lifted
    // 1 unit off the surface: the GS Z GREATER test never counts a volume face coplanar with the
    // receiver (crates / planks whose own volume shares their faces)
    let ins = |p: [f32; 3]| [0, 1, 2].map(|k| p[k] + (cen[k] - p[k]) * 0.02);
    let up = |p: [f32; 3]| [0, 1, 2].map(|k| p[k] + n[k]);
    let cs: Vec<i32> = [ins(t[0]), ins(t[1]), ins(t[2]), ab, bc, ca, cen].iter().map(|&p| count(vols, up(p))).collect();
    let len = |p: [f32; 3], q: [f32; 3]| dot(sub(p, q), sub(p, q)).sqrt();
    let longest = len(t[0], t[1]).max(len(t[1], t[2])).max(len(t[2], t[0]));
    let uniform = cs.iter().all(|&c| c == cs[0]);
    if (uniform && longest < 300.0) || longest <= min_edge || depth > 12 {
        let al = count_alpha(cs[6], a1);
        if al > 0.0 {
            emit(t, al);
        }
        return;
    }
    for s in [[t[0], ab, ca], [ab, t[1], bc], [ca, bc, t[2]], [ab, bc, ca]] {
        split(vols, s, n, a1, min_edge, depth + 1, emit);
    }
}
