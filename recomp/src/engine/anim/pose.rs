//! WP-D: pose matrices and the skinning palette.
//!
//! Matrix layout as the engine stores it (VU0 quadwords): `m[0..3]` = basis vectors (rows in
//! memory, columns for VU0 `M * v`), `m[3]` = translation with w = 1 (FUN_00117f18 forces the
//! w components to (0,0,0,1)). Composition FUN_0011c168 is VU0 microcode (`vcallms 0x60`, not
//! decompiled); it is replaced by the standard column-vector product `out = a * b`
//! (`out[j] = sum_i b[j][i] * a[i]`), i.e. apply `b` first, then `a`.

use super::keys::V3;

pub type Mat4 = [[f32; 4]; 4];

pub const IDENTITY: Mat4 = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];

/// FUN_00117f18 result (3 rows + translation, 12 bytes each) as a Mat4.
pub fn from_4x3(m: &[[f32; 3]; 4]) -> Mat4 {
    let mut o = IDENTITY;
    for i in 0..4 {
        o[i][..3].copy_from_slice(&m[i]);
    }
    o
}

/// FUN_0011c168 (replaced, see module doc): a * b.
pub fn ee_sub_11c168(a: &Mat4, b: &Mat4) -> Mat4 {
    std::array::from_fn(|j| std::array::from_fn(|r| (0..4).map(|i| b[j][i] * a[i][r]).sum()))
}

fn cross(a: V3, b: V3) -> V3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
fn dot(a: V3, b: V3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn v(m: &[f32; 4]) -> V3 {
    [m[0], m[1], m[2]]
}

/// FUN_0011c318: affine inverse. VU0 code: c0 = r1 x r2, c1 = r2 x r0, c2 = r0 x r1,
/// q = 1 / (r0 . c0); the inverse's 3x3 is the transposed cofactors scaled by q, and its
/// translation = -(inverse 3x3) * t; w row (0,0,0,1).
pub fn ee_sub_11c318(m: &Mat4) -> Mat4 {
    // 0x0011c318
    let (r0, r1, r2, t) = (v(&m[0]), v(&m[1]), v(&m[2]), v(&m[3]));
    let c0 = cross(r1, r2);
    let c1 = cross(r2, r0);
    let c2 = cross(r0, r1);
    let q = 1.0 / dot(r0, c0);
    // inverse columns (stored rows) j: (c0[j], c1[j], c2[j]) * q
    let mut o = IDENTITY;
    for j in 0..3 {
        o[j] = [c0[j] * q, c1[j] * q, c2[j] * q, 0.0];
    }
    for r in 0..3 {
        o[3][r] = -(o[0][r] * t[0] + o[1][r] * t[1] + o[2][r] * t[2]);
    }
    o
}

/// FUN_0022cc00: matrix carrying a channel vector `x` for bone kind +0x94 (1/4/5/9/10:
/// translation row; 0xb: [1][0], [2][0], [2][1]; 0xc: diagonal (scale)); everything else zero.
pub fn ee_sub_22cc00(bone_kind: i32, x: V3) -> Mat4 {
    // 0x0022cc00
    let mut f = [0.0f32; 16];
    match bone_kind {
        1 | 4 | 5 | 9 | 10 => {
            f[12] = x[0];
            f[13] = x[1];
            f[14] = x[2];
        }
        0xb => {
            f[4] = x[0];
            f[8] = x[1];
            f[9] = x[2];
        }
        0xc => {
            f[0] = x[0];
            f[5] = x[1];
            f[10] = x[2];
        }
        _ => {}
    }
    std::array::from_fn(|i| std::array::from_fn(|j| f[i * 4 + j]))
}

/// World matrices from local matrices and parent indices (parents must precede children;
/// `None` = root, `root` applied). Replaces the FUN_0022faa0 VU0 world-matrix walk.
pub fn world_matrices(root: &Mat4, locals: &[Mat4], parents: &[Option<usize>]) -> Vec<Mat4> {
    let mut w: Vec<Mat4> = Vec::with_capacity(locals.len());
    for (i, l) in locals.iter().enumerate() {
        let p = match parents[i] {
            Some(p) if p < i => w[p],
            _ => *root,
        };
        w.push(ee_sub_11c168(&p, l));
    }
    w
}

/// Skinning palette: world[i] * inverse(bind[i]) (Bevy SkinnedMesh joint matrices).
pub fn skin_palette(world: &[Mat4], bind: &[Mat4]) -> Vec<Mat4> {
    world.iter().zip(bind).map(|(w, b)| ee_sub_11c168(w, &ee_sub_11c318(b))).collect()
}

/// Max deviation of the 3x3 basis from orthonormal (|ri.rj - dij|).
pub fn orthonormal_error(m: &Mat4) -> f32 {
    let mut e = 0.0f32;
    for i in 0..3 {
        for j in 0..3 {
            let d = dot(v(&m[i]), v(&m[j])) - if i == j { 1.0 } else { 0.0 };
            e = e.max(d.abs());
        }
    }
    e
}

/// Column-major f32x16 (Bevy `Mat4::from_cols_array`).
pub fn to_cols_array(m: &Mat4) -> [f32; 16] {
    std::array::from_fn(|i| m[i / 4][i % 4])
}
