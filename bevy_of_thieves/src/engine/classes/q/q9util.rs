//! WP-Q9: shared helpers of the Q9 queue ports (classes/q/c*.rs, q9_runs_*.rs) on `QMem`.
//! Conventions as q2mem / q6 / q7util / q8util (ints in `a`, floats / quadwords in `v`, float result `.1[0]`).
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem, V4};
pub use crate::engine::classes::q::q6::{callf, ro32, rof, rov, s, set_tagged, vcall, CLOCK, WORLD};

/// FUN_001ae2a0(&{size, align}): allocate through an emulated-stack request block
pub fn alloc<M: QMem + ?Sized>(m: &mut M, size: u32, align: u32) -> u32 {
    let req = stack_push(m, 8);
    m.w32(req, size);
    m.w32(req + 4, align);
    let p = m.call(0x001a_e2a0, &[req], &[]).0;
    stack_pop(m, 8);
    p
}
/// `*(*(a) + off)`
pub fn rr<M: QMem + ?Sized>(m: &mut M, a: u32, off: u32) -> u32 {
    let p = m.r32(a);
    m.r32(p.wrapping_add(off))
}
/// FUN_001a62e8(*(a)): position quadword of the object stored at field `a`
pub fn pos_at<M: QMem + ?Sized>(m: &mut M, a: u32) -> V4 {
    let p = m.r32(a);
    m.call(0x001a_62e8, &[p], &[]).1
}
/// BinaryInputStream_read_u16 (FUN_00117c20)
pub fn read_u16<M: QMem + ?Sized>(m: &mut M, st: u32) -> u32 {
    m.call(0x0011_7c20, &[st], &[]).0
}
/// FUN_00117eb0(stream, dst): BinaryInputStream_read_117968(stream, 0xc, dst)
pub fn read_vec3<M: QMem + ?Sized>(m: &mut M, st: u32, dst: u32) {
    m.call(0x0011_7eb0, &[st, dst], &[]);
}
/// `*(v4 *)dst = *(v4 *)a + *(v4 *)b` (VU0 lqc2 / vadd / sqc2)
pub fn vadd_mem<M: QMem + ?Sized>(m: &mut M, dst: u32, a: u32, b: u32) {
    let x = m.rv(a);
    let y = m.rv(b);
    m.wv(dst, std::array::from_fn(|i| x[i] + y[i]));
}
/// copy `n` words
pub fn cpyw<M: QMem + ?Sized>(m: &mut M, dst: u32, src: u32, n: u32) {
    for i in 0..n {
        let w = m.r32(src + 4 * i);
        m.w32(dst + 4 * i, w);
    }
}
/// the VU0 normalize of the quadword at `a` used by the C2ca6a0 / C2cc2c0 family: len = sqrt(x*x + y*y + z*z)
/// (vmul.xyz, x + y + 1*z accumulate, vsqrt); len < 1e-4 -> `fallback` quad, else every lane * (1 / len)
/// (vdiv + vmulq). Returns (normalized, len).
pub fn vu_normalize(v: V4, fallback: V4) -> (V4, f32) {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 0.0001 {
        (fallback, len)
    } else {
        let q = 1.0 / len;
        (v.map(|x| x * q), len)
    }
}
/// Q9 overrides whose whole body is `return <const>;`: (fn, value)
pub const CONST_RET: &[(u32, u32)] = &[
    (0x114650, 0x2bba38),
    (0x114640, 0x2bba24),
    (0x113e10, 0x2bafe8),
    (0x114560, 0x2bb90c),
    (0x113dc0, 0x2baf84),
    (0x114600, 0x2bb9d4),
    (0x1145f0, 0x2bb9c0),
    (0x1145e0, 0x2bb9ac),
    (0x1145c0, 0x2bb984),
    (0x114590, 0x2bb948),
    (0x114580, 0x2bb934),
    (0x1144e0, 0x2bb86c),
    (0x114500, 0x2bb894),
    (0x114520, 0x2bb8bc),
    (0x1143d0, 0x2bb718),
    (0x114430, 0x2bb790),
    (0x1144f0, 0x2bb880),
    (0x113f80, 0x2bb1b4),
    (0x1140d0, 0x2bb358),
    (0x114310, 0x2bb628),
    (0x1142e0, 0x2bb5ec),
    (0x1142c0, 0x2bb5c4),
    (0x1142b0, 0x2bb5b0),
    (0x114020, 0x2bb27c),
    (0x114320, 0x2bb63c),
    (0x114380, 0x2bb6b4),
    (0x1b2820, 0x0),
];
/// Q9 overrides whose body is `jr ra` only (return;)
pub const NOPS: &[u32] = &[
    0x12ced8,
    0x12cf28,
    0x1af388,
    0x1eda50,
    0x12b4f0,
    0x14d470,
    0x1c6ab0,
    0x1fb520,
];
