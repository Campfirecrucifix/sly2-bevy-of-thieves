//! WP-C5: ink outlines (mesh type 1, VU1 overlay 188054331, pass 1 = FUN_0016ea20).
//! The VU1 program evaluates `mesh::ee_vu1_edge_silhouette` per edge quad (a, b, c, d) in
//! projected space and draws edge (b, c) as a thin screen-space quad (+-0.00024/|c-b|).
//! Port: the same rule evaluated in the primitive's local space against the eye position (the
//! projected-z sign of a face cross product equals the sign of its dot with (eye - point) under
//! a perspective projection, same factor for both faces), emitted as a Bevy LineList each frame
//! (`game::update_ink_outlines`). Replaced, not 1:1: the clip-flag skip and the quad width
//! (1-pixel lines instead), the ink colour (black).

use super::mesh::MeshPrim;

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// `ee_vu1_edge_silhouette` with the projected z replaced by the view-space facing:
/// z1 = ((c-b) x (b-a)).(eye-b), z2 = ((c-b) x (d-c)).(eye-b), emit iff z1*z2 < 0.
pub fn edge_silhouette_eye(a: [f32; 3], b: [f32; 3], c: [f32; 3], d: [f32; 3], eye: [f32; 3]) -> bool {
    let e = sub(c, b);
    let v = sub(eye, b);
    let z1 = dot(cross(e, sub(b, a)), v);
    let z2 = dot(cross(e, sub(d, c)), v);
    z1 * z2 < 0.0
}

/// Line segments (pairs of points, prim-local space) of an ink prim seen from `eye`.
pub fn ink_segments(p: &MeshPrim, eye: [f32; 3], out: &mut Vec<[f32; 3]>) {
    let n = p.positions.len();
    for q in &p.edges {
        if q.iter().any(|&i| i as usize >= n) {
            continue;
        }
        let [a, b, c, d] = q.map(|i| p.positions[i as usize]);
        if edge_silhouette_eye(a, b, c, d, eye) {
            out.push(b);
            out.push(c);
        }
    }
}
