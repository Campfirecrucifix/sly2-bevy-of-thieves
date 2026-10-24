//! WP-C5: ink outlines (mesh type 1, VU1 overlay 188054331, pass 1 = FUN_0016ea20).
//! The VU1 program evaluates `mesh::ee_vu1_edge_silhouette` per edge quad (a, b, c, d) in
//! projected space and draws edge (b, c) as a GS LINE offset by 0.00024 projected units
//! (`vu1_ink_line`, one GS pixel wide: AA1 colour line + Z-only line).
//! Port: the same rule evaluated in the primitive's local space against the eye position (the
//! projected-z sign of a face cross product equals the sign of its dot with (eye - point) under
//! a perspective projection, same factor for both faces), emitted as a Bevy LineList each frame
//! (`game::update_ink_outlines`; ANIM R73: as screen quads covering the AA1 GS LINE footprint
//! (+-1 field pixel of the 512x224 buffer on the minor axis) around `vu1_ink_line`, SLY2_R73INKLINE = the old 1-px LineList). Replaced, not 1:1: the
//! clip-flag skip, the AA1 coverage (opaque), the ink colour (black).

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

/// ANIM R73: silhouette edges (a, b, c) of an ink prim seen from `eye` (prim-local); `a` = the far
/// vertex of face (a, b, c), it picks the VU1 offset side (`vu1_ink_line`).
pub fn ink_edges(p: &MeshPrim, eye: [f32; 3], out: &mut Vec<[[f32; 3]; 3]>) {
    let n = p.positions.len();
    for q in &p.edges {
        if q.iter().any(|&i| i as usize >= n) {
            continue;
        }
        let [a, b, c, d] = q.map(|i| p.positions[i as usize]);
        if edge_silhouette_eye(a, b, c, d, eye) {
            out.push([a, b, c]);
        }
    }
}

/// ANIM R73: VU1 projected units -> GS pixels (VU qw0 x/y scale 0.99 * 2^25 against the qw1
/// float-mantissa offset = 0.99 * 2048 GS px per unit, logs/LOOK.md FUN_0014db70).
pub const VU1_GS_PX: f32 = 0.99 * 2048.0;
/// ANIM R73: the ink offset `LOI 0x397ba882` (+-0.00024 projected units) in GS pixels (0.4866).
pub const INK_OFFSET_PX: f32 = 0.00024 * VU1_GS_PX;

/// ANIM R73: overlay 188054331 0x0640..0x06c8 on the screen (GS pixel) positions of a silhouette
/// edge (a, b, c): perp = (dy, -dx) of c-b, s = +1 if z1 = perp.(a-b) >= 0 else -1 (FSAND sign of
/// z1 at 0x0648), b' = b + s k perp/|c-b|, c' = c + k (c-b)/|c-b| + s k perp/|c-b|. The GS then
/// draws the LINE b'c' twice (FUN_0016e438 packet: prim 0xc1 LINE+ABE+AA1 with TEST 0x73001 colour
/// only, prim 1 LINE with TEST 0x72001 Z only) = one GS pixel wide.
pub fn vu1_ink_line(a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> Option<([f32; 2], [f32; 2])> {
    let (dx, dy) = (c[0] - b[0], c[1] - b[1]);
    let l2 = dx * dx + dy * dy;
    if !(l2 > 0.0) {
        return None;
    }
    let r = l2.sqrt().recip();
    let z1 = dy * (a[0] - b[0]) - dx * (a[1] - b[1]);
    let s = if z1 < 0.0 { -INK_OFFSET_PX } else { INK_OFFSET_PX };
    let (px, py) = (dy * s * r, -dx * s * r);
    let (ex, ey) = (dx * INK_OFFSET_PX * r, dy * INK_OFFSET_PX * r);
    Some(([b[0] + px, b[1] + py], [c[0] + ex + px, c[1] + ey + py]))
}
