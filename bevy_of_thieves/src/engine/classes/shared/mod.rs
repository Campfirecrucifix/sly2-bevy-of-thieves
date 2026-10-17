//! WP-K13: engine callees shared by the K13+ class ports (1:1, reusable by later classes).
//! - `act`: xf actuator records (FUN_0012e238 kinds, FUN_0012db50 / FUN_0012dca0 constructors, FUN_00230018
//!   priority list insert at xf +0x90, FUN_0022c840 xf reset). REPLACED: the 0x3e123c record pool
//!   (FUN_0012e1f0 -> FUN_0011ccc0 free list over the FUN_001ae1d0 / FUN_001ae1c8 bump allocator DAT_002f6384)
//!   and FUN_002333b4 memset = zero-filled Rust words.
//! - `anim`: FUN_001305b8 / FUN_00130658 anim handle (re)play (FUN_0012f9b8 request and FUN_00130b18 retract
//!   stay ClassHost hooks: their ports are player-/anim-side).
//! - `proj`: the C2cb780 (class 79) / C2d3aa0 (class 82) projectile base: init FUN_0013a9e8 / FUN_001b1db8 and
//!   the field rows 383-409.
//! - `c71`: (K14) the C2d1878 (class 71) base of 72-76: init FUN_0017f090, slots 11 / 25, load-post part,
//!   touch request / cancel, field rows 1363-1370.
//! - `launch`: (K14) the class-112 launcher record (rows 460-467) and FUN_001d6b00 (aim + launch a projectile,
//!   slot 77).
//! - `path10`: (K16) class-10 path data FUN_001e5df8 / FUN_001e6180 in the object's world frame.
pub mod act;
pub mod anim;
pub mod c71;
pub mod launch;
pub mod path10;
pub mod proj;

/// v * s
pub fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}
/// |v|
pub fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}
/// VU0 1/x (vdiv): x == 0 gives the largest float instead of inf (so 0 * (1/0) = 0, not NaN)
pub fn vu_rcp(x: f32) -> f32 {
    if x == 0.0 { f32::MAX } else { 1.0 / x }
}
/// VU0 multiply: FLT_MAX * 0 = 0 (no inf / NaN on the VU)
pub fn vu_mul(a: f32, b: f32) -> f32 {
    if a == 0.0 || b == 0.0 { 0.0 } else { (a * b).clamp(-f32::MAX, f32::MAX) }
}
