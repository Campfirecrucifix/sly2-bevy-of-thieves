//! WP-C: ALO mesh records (WP-B3 `world::alo`) -> triangle lists, as the VU1 renderer draws
//! them. The loaders (FUN_0016a000 / FUN_0016cf78) VIF-unpack V3-32 positions, V2-32 UVs (ST,
//! already normalised floats), V3-32 normals and V4-8 colours, so no fixed-point scaling is
//! applied to positions/UVs. Strips: one index per strip vertex; the per-index bit
//! (`index_bits`, packed 32 per word) is the GS ADC flag = "do not draw the triangle ending at
//! this vertex" (every strip starts with >= 2 set bits). Winding alternates with the vertex
//! position in the material run, see `strip_triangles`.
//! Vertex colours (V4-8, 0x80 = 1.0 on the GS) are carried as a separate material input
//! (`colors`, linear 0..2 range), not as baked lighting.

use crate::engine::world::alo::{Mesh0, Mesh3, MeshData, MeshStrips};

/// WP-C4: what the record's VU1 program does with the data (mesh type -> program, decoded from
/// vu_code; MSCAL addresses from the loaders' FUN_00226ce0 calls).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum MeshRole {
    /// type 0 (FUN_0016a000, overlay 376110651): textured lit strips
    #[default]
    Surface,
    /// type 1 (FUN_0016e438, MSCAL 0x738/0x7f0, overlay 188054331 / pass 1 = FUN_0016ea20): ink
    /// outline edge list; the V4-8 array = one edge quad (a, b, c, d) per element, edge = (b, c),
    /// faces (a, b, c) and (b, c, d); drawn as a screen-space quad when `ee_vu1_edge_silhouette`
    /// holds in projected space (0x0500 loop).
    InkEdges,
    /// types 2/3 (FUN_0016caa0 / FUN_0016cf78, MSCAL 0x1910/0x1a68, overlay 188054619 / passes 6,7
    /// = FUN_0016da48, FRAME mask writes only the green byte): shadow volumes, every material is a
    /// shader of kind 2 (SHDK_Shadow). Type 2 = edge quads (silhouette in light space, extruded via
    /// the stride-2 vertex slot copy), type 3 = indexed strips (caps).
    ShadowVolume,
}

/// WP-C4: the VU1 face/edge rejection rule of a primitive.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Vu1Cull {
    /// type 0, flags & 0x10 clear: triangles with non-negative screen winding get ADC
    Back,
    /// type 0 with flags & 0x10 (header +8 |= 0x80), type 3 strips: no winding test
    #[default]
    None,
    /// types 1/2: edge drawn only if its two faces face opposite ways (`ee_vu1_edge_silhouette`)
    Silhouette,
}

/// VU1 silhouette edge test shared by the type-1 (0x05a8..0x0718, overlay 188054331, projected
/// positions) and type-2 (0x0740..0x07b8, overlay 188054619, light space) programs:
/// `z1 = ((c-b) x (b-a)).z`, `z2 = ((c-b) x (d-c)).z` (OPMULA/OPMSUB), `MUL.z z1*z2`, FSAND sign
/// -> emit iff the product is negative (exactly 0 = not drawn). Boundary edges are stored with
/// a == d, which makes z2 = -z1: always a silhouette. The sign of z1 picks the side the quad is
/// extruded to (type 1: perpendicular offset +-0.00024 / |c-b| in projected units).
pub fn ee_vu1_edge_silhouette(a: [f32; 3], b: [f32; 3], c: [f32; 3], d: [f32; 3]) -> bool {
    let e = sub(c, b);
    let z1 = cross(e, sub(b, a))[2];
    let z2 = cross(e, sub(d, c))[2];
    z1 * z2 < 0.0
}

/// A converted mesh primitive: one material run (or strip) of a mesh record.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MeshPrim {
    /// index into the owning entry's shader table (FUN_001ec8f8); None = untextured
    pub shader: Option<u32>,
    /// Material0 flags (type 0)
    pub flags: u32,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    /// RGBA, PS2 scale (0x80 = 1.0) converted to float: c / 128
    pub colors: Vec<[f32; 4]>,
    pub tangents: Vec<[f32; 4]>,
    pub indices: Vec<u32>,
    /// skinning (kept for WP-D): bone slot indices of the sub-mesh and per-vertex raw weights
    pub weight_slots: Vec<u8>,
    pub weights: Vec<Vec<u8>>,
    /// VU1 back-face culling (`material::vu1_cull_back`); false = drawn double-sided
    pub cull_back: bool,
    /// WP-C4: VU1 program role / rejection rule
    pub role: MeshRole,
    pub cull: Vu1Cull,
    /// types 1/2: edge quads (a, b, c, d) indexing `positions` (V4-8 array, its own u8 count;
    /// type 2 slot numbers already halved)
    pub edges: Vec<[u32; 4]>,
}

fn bit(bits: &[u32], k: usize) -> bool {
    bits.get(k / 32).is_some_and(|w| (w >> (k % 32)) & 1 != 0)
}

/// Strip -> triangles for indices [start, start+count) of a batch. ADC bit set = skip.
/// Winding: triangle ending at run position k is (k-2, k-1, k) for even k, (k-1, k-2, k) for
/// odd k, k counted from the start of the material run (one GIF/VU batch); the exporter
/// duplicates a vertex to keep strips in phase. Measured: 99.3% of faces agree with the stored
/// normals (batch-global parity 79%, per-strip parity 78%).
pub fn strip_triangles(idx: &[u8], bits: &[u32], start: usize, count: usize, out: &mut Vec<[u32; 3]>) {
    for k in start + 2..start + count {
        if k >= idx.len() || bit(bits, k) {
            continue;
        }
        let (a, b, c) = (idx[k - 2] as u32, idx[k - 1] as u32, idx[k] as u32);
        if a == b || b == c || a == c {
            continue; // degenerate (GS draws nothing)
        }
        out.push(if (k - start) % 2 == 0 { [a, b, c] } else { [b, a, c] });
    }
}

/// Unscaled PS2 vertex colour -> float (0x80 = 1.0).
pub fn ps2_color(c: [u8; 4]) -> [f32; 4] {
    c.map(|x| x as f32 / 128.0)
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
fn norm(v: [f32; 3]) -> Option<[f32; 3]> {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    (l > 1e-12 && l.is_finite()).then(|| [v[0] / l, v[1] / l, v[2] / l])
}

/// Smooth normals from faces (types without stored normals).
pub fn face_normals(p: &[[f32; 3]], tris: &[[u32; 3]]) -> Vec<[f32; 3]> {
    let mut n = vec![[0f32; 3]; p.len()];
    for t in tris {
        let f = cross(sub(p[t[1] as usize], p[t[0] as usize]), sub(p[t[2] as usize], p[t[0] as usize]));
        for &i in t {
            for j in 0..3 {
                n[i as usize][j] += f[j];
            }
        }
    }
    n.into_iter().map(|v| norm(v).unwrap_or([0.0, 0.0, 1.0])).collect()
}

/// Per-vertex tangents (MikkTSpace-like accumulation, w = handedness).
pub fn tangents(p: &[[f32; 3]], n: &[[f32; 3]], uv: &[[f32; 2]], tris: &[[u32; 3]]) -> Vec<[f32; 4]> {
    let mut t = vec![[0f32; 3]; p.len()];
    let mut b = vec![[0f32; 3]; p.len()];
    for tri in tris {
        let [i0, i1, i2] = tri.map(|i| i as usize);
        let (e1, e2) = (sub(p[i1], p[i0]), sub(p[i2], p[i0]));
        let (du1, dv1) = (uv[i1][0] - uv[i0][0], uv[i1][1] - uv[i0][1]);
        let (du2, dv2) = (uv[i2][0] - uv[i0][0], uv[i2][1] - uv[i0][1]);
        let d = du1 * dv2 - du2 * dv1;
        if d.abs() < 1e-20 || !d.is_finite() {
            continue;
        }
        let r = 1.0 / d;
        let sd = [(e1[0] * dv2 - e2[0] * dv1) * r, (e1[1] * dv2 - e2[1] * dv1) * r, (e1[2] * dv2 - e2[2] * dv1) * r];
        let td = [(e2[0] * du1 - e1[0] * du2) * r, (e2[1] * du1 - e1[1] * du2) * r, (e2[2] * du1 - e1[2] * du2) * r];
        for i in [i0, i1, i2] {
            for j in 0..3 {
                t[i][j] += sd[j];
                b[i][j] += td[j];
            }
        }
    }
    (0..p.len())
        .map(|i| {
            let nn = n[i];
            let d = t[i][0] * nn[0] + t[i][1] * nn[1] + t[i][2] * nn[2];
            let o = [t[i][0] - nn[0] * d, t[i][1] - nn[1] * d, t[i][2] - nn[2] * d];
            let tt = norm(o).unwrap_or_else(|| {
                // any vector perpendicular to n
                let a = if nn[0].abs() < 0.9 { [1.0, 0.0, 0.0] } else { [0.0, 1.0, 0.0] };
                norm(cross(nn, a)).unwrap_or([1.0, 0.0, 0.0])
            });
            let w = if cross(nn, tt).iter().zip(b[i]).map(|(x, y)| x * y).sum::<f32>() < 0.0 { -1.0 } else { 1.0 };
            [tt[0], tt[1], tt[2], w]
        })
        .collect()
}

fn finish(mut p: MeshPrim, tris: Vec<[u32; 3]>) -> Option<MeshPrim> {
    if tris.is_empty() {
        return None;
    }
    if p.normals.len() != p.positions.len() {
        p.normals = face_normals(&p.positions, &tris);
    } else {
        // replace degenerate / NaN stored normals
        let f = face_normals(&p.positions, &tris);
        for (n, g) in p.normals.iter_mut().zip(f) {
            *n = norm(*n).unwrap_or(g);
        }
    }
    if p.uvs.len() != p.positions.len() {
        p.uvs = vec![[0.0, 0.0]; p.positions.len()];
    }
    if p.colors.len() != p.positions.len() {
        p.colors = vec![[1.0; 4]; p.positions.len()];
    }
    p.tangents = tangents(&p.positions, &p.normals, &p.uvs, &tris);
    p.indices = tris.into_iter().flatten().collect();
    Some(p)
}

fn split_weights(w: &[u8], k: usize, nv: usize) -> Vec<Vec<u8>> {
    if k == 0 {
        return vec![];
    }
    (0..nv).map(|i| w.get(i * k..(i + 1) * k).map(|s| s.to_vec()).unwrap_or_default()).collect()
}

/// FUN_0016a000 data: one primitive per (sub-mesh, material run).
pub fn convert_mesh0(m: &Mesh0) -> Vec<MeshPrim> {
    let k = (m.nweights.max(0) as usize).min(4);
    let mut out = Vec::new();
    for s in &m.subs {
        let mut start = 0usize;
        for mat in &s.materials {
            let mut tris = Vec::new();
            strip_triangles(&s.indices, &s.index_bits, start, mat.count as usize, &mut tris);
            start += mat.count as usize;
            let nv = s.positions.len();
            if tris.iter().flatten().any(|&i| i as usize >= nv) {
                continue; // out of range index: not drawable (counted by tests)
            }
            let p = MeshPrim {
                shader: Some(mat.material),
                flags: mat.flags,
                positions: s.positions.clone(),
                normals: s.normals.clone(),
                uvs: s.uvs.clone(),
                colors: s.colors.iter().map(|&c| ps2_color(c)).collect(),
                weight_slots: s.weight_slots.map(|w| w.to_vec()).unwrap_or_default(),
                weights: split_weights(&s.weights, k, nv),
                cull_back: super::material::vu1_cull_back(m.flags),
                cull: if super::material::vu1_cull_back(m.flags) { Vu1Cull::Back } else { Vu1Cull::None },
                ..Default::default()
            };
            out.extend(finish(p, tris));
        }
    }
    out
}

/// FUN_0016cf78 data: indexed position-only strips (no UVs / normals / colours in the stream).
pub fn convert_mesh3(m: &Mesh3) -> Vec<MeshPrim> {
    let mut out = Vec::new();
    for s in &m.strips {
        let mut tris = Vec::new();
        strip_triangles(&s.indices, &s.index_bits, 0, s.indices.len(), &mut tris);
        if tris.iter().flatten().any(|&i| i as usize >= s.positions.len()) {
            continue;
        }
        let k = if s.positions.is_empty() { 0 } else { s.weights.len() / s.positions.len() };
        let p = MeshPrim {
            shader: Some(s.material as u32),
            positions: s.positions.clone(),
            weight_slots: s.weight_slots.map(|w| w.to_vec()).unwrap_or_default(),
            weights: split_weights(&s.weights, k, s.positions.len()),
            role: MeshRole::ShadowVolume,
            ..Default::default()
        };
        out.extend(finish(p, tris));
    }
    out
}

/// Faces carried by edge quads: (a, b, c) and (c, b, d) of every edge, deduplicated by vertex set
/// (first orientation wins).
pub fn edge_faces(edges: &[[u32; 4]]) -> Vec<[u32; 3]> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for &[a, b, c, d] in edges {
        for t in [[a, b, c], [c, b, d]] {
            if t[0] == t[1] || t[1] == t[2] || t[0] == t[2] {
                continue;
            }
            let mut k = t;
            k.sort_unstable();
            if seen.insert(k) {
                out.push(t);
            }
        }
    }
    out
}

/// WP-C4: FUN_0016e438 (type 1) / FUN_0016caa0 (type 2) edge lists -> Solari-compatible meshes
/// (the faces the edge quads reference), with the edges kept for the VU1 silhouette rule.
pub fn convert_edges(m: &MeshStrips, role: MeshRole) -> Vec<MeshPrim> {
    let mut out = Vec::new();
    for s in &m.strips {
        let n = s.positions.len();
        // type 2 unpacks positions with VIF skip-write stride 2 (FUN_00226d40(.., 1, 2)): vertex
        // k lives in VU slot 2k, slot 2k+1 receives the program's extruded copy; the edge
        // indices are slot numbers (always even in the data).
        let stride = if role == MeshRole::ShadowVolume { 2 } else { 1 };
        if s.v4_8.iter().flatten().any(|&i| i as u32 % stride != 0) {
            continue;
        }
        let edges: Vec<[u32; 4]> = s.v4_8.iter().map(|q| q.map(|i| i as u32 / stride)).collect();
        if edges.iter().flatten().any(|&i| i as usize >= n) {
            continue;
        }
        let tris = edge_faces(&edges);
        let k = if n == 0 { 0 } else { s.weights.len() / n };
        let p = MeshPrim {
            shader: s.material.map(|m| m as u32),
            positions: s.positions.clone(),
            weight_slots: s.weight_slots.clone(),
            weights: split_weights(&s.weights, k, n),
            role,
            cull: Vu1Cull::Silhouette,
            edges,
            ..Default::default()
        };
        out.extend(finish(p, tris));
    }
    out
}

/// [superseded by `convert_edges` in WP-C4] FUN_0016e438 / FUN_0016caa0 positions read as
/// sequential strips. Kept for comparison only: the VU1 programs never draw these as strips.
pub fn convert_strips(m: &MeshStrips) -> Vec<MeshPrim> {
    let mut out = Vec::new();
    for s in &m.strips {
        let n = s.positions.len();
        let idx: Vec<u8> = (0..n).map(|i| i as u8).collect();
        let bits = vec![0b11u32];
        let mut tris = Vec::new();
        strip_triangles(&idx, &bits, 0, n, &mut tris);
        let k = if n == 0 { 0 } else { s.weights.len() / n };
        let p = MeshPrim {
            shader: s.material.map(|m| m as u32),
            positions: s.positions.clone(),
            weight_slots: s.weight_slots.clone(),
            weights: split_weights(&s.weights, k, n),
            ..Default::default()
        };
        out.extend(finish(p, tris));
    }
    out
}

pub fn convert(d: &MeshData) -> Vec<MeshPrim> {
    match d {
        MeshData::Type0(m) => convert_mesh0(m),
        MeshData::Type1(m) => convert_edges(m, MeshRole::InkEdges),
        MeshData::Type2(m) => convert_edges(m, MeshRole::ShadowVolume),
        MeshData::Type3(m) => convert_mesh3(m),
        MeshData::Other | MeshData::Skipped { .. } => vec![],
    }
}
