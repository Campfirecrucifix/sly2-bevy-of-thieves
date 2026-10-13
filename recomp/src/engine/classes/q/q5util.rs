//! WP-Q5: small shared helpers for the Q5 queue ports (VU0 macro idioms used by many bodies).
use crate::engine::classes::q::q2mem::{QMem, V4};

/// `vsub; vmul.xyz; vaddw.x one; vadday.x; vmaddz.x; vsqrt` : |a - b| over xyz
pub fn dist3(a: V4, b: V4) -> f32 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

/// `vsub; vmul.xy; vaddy.x; vsqrt` : |a - b| over xy (ground plane, z up)
pub fn dist_xy(a: V4, b: V4) -> f32 {
    let d = [a[0] - b[0], a[1] - b[1]];
    (d[0] * d[0] + d[1] * d[1]).sqrt()
}

/// `c0 + x * (c1 + x * c2)` (curve triples stored as {c0, c1, c2} floats)
pub fn poly<M: QMem + ?Sized>(m: &mut M, at: u32, x: f32) -> f32 {
    let (c0, c1, c2) = (m.rf(at), m.rf(at + 4), m.rf(at + 8));
    c0 + x * (c1 + x * c2)
}

/// `obj +0xe8 -> +0xdc -> +0x10` with null checks (the "current surface / standing-on" object)
pub fn e8_dc_10<M: QMem + ?Sized>(m: &mut M, obj: u32) -> u32 {
    let a = m.r32(obj + 0xe8);
    let b = if a != 0 { m.r32(a + 0xdc) } else { 0 };
    if b != 0 {
        m.r32(b + 0x10)
    } else {
        0
    }
}

/// FUN_001a62e8: position quadword (`vslot` 30 -> +0x30)
pub fn pos<M: QMem + ?Sized>(m: &mut M, obj: u32) -> V4 {
    m.call(0x001a_62e8, &[obj], &[]).1
}

/// FUN_00207a20: `k == -1 ? 0 : 1 << k` (u64)
pub fn bit64(k: i64) -> u64 {
    if k != -1 {
        1u64 << k
    } else {
        0
    }
}

pub fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}
