//! Camera collision push-in: 0x0014ea08 (free-gap search along a segment) and 0x0014eb70 (sphere of
//! probe rays around the focus, keeps the closest blocked point).
//! The segment-vs-geometry clipper (FUN_00118668 per collision geom + FUN_0014c9f8 interval merge) is
//! world/physics data (WP-E); it is replaced by the `CmSegmentQuery` trait returning the merged solid
//! intervals in segment-fraction space.
#![allow(non_snake_case)]

use super::math::*;

pub trait CmSegmentQuery {
    /// Sorted, merged (enter, exit) fractions in [0,1] where the segment start->end is inside geometry.
    fn solid_intervals(&mut self, start: V3, end: V3) -> Vec<(f32, f32)>;
}

/// 0x0014ea08: first free gap longer than `min_gap` between solid intervals (0 and 1 are the bounds).
/// Returns (gap start, gap end) or None.
pub fn FUN_0014ea08(min_gap: f32, q: &mut dyn CmSegmentQuery, start: V3, end: V3) -> Option<(f32, f32)> {
    let iv = q.solid_intervals(start, end);
    let n = iv.len();
    for i in 0..=n {
        let lo = if i > 0 { iv[i - 1].1 } else { 0.0 };
        let hi = if i < n { iv[i].0 } else { 1.0 };
        if min_gap < hi - lo {
            return Some((lo, hi));
        }
    }
    None
}

/// 0x002bd8b0: rays per elevation ring (9 rings, elevation (i-4)*pi/8).
pub const RING_COUNTS: [i32; 9] = [1, 5, 10, 12, 15, 12, 10, 5, 1];

/// 0x0014eb70: probe `radius` around `center`; return the nearest point a gap starts (pulled 1 unit back
/// along the ray), or `default` if nothing is closer than FLT_MAX (0x2a4ffc).
/// Quirk kept: azimuth steps are pi/n (half circle), and `fVar11` is always 0 so any hit fraction > 0 is
/// pushed out by 5/radius.
pub fn FUN_0014eb70(radius: f32, q: &mut dyn CmSegmentQuery, center: V3, default: V3) -> V3 {
    let pad = 5.0 / radius;
    let step = 0.3926991;
    let mut best = f32::MAX;
    let mut result = default;
    for (ring, &n) in RING_COUNTS.iter().enumerate() {
        let el = (ring as i32 - 4) as f32 * step;
        for j in 0..n {
            let dir = FUN_0011e208((j as f32 * PI) / n as f32, el, 1.0);
            let end = add(center, scale(dir, radius));
            if let Some((mut f, _)) = FUN_0014ea08(pad + pad, q, center, end) {
                if 0.0 < f {
                    f += pad;
                }
                if f < best {
                    result = sub(add(scale(center, 1.0 - f), scale(end, f)), dir);
                    best = f;
                }
            }
        }
    }
    result
}
