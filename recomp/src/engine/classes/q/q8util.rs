//! WP-Q8: shared helpers of the Q8 queue ports (classes/q/c*.rs, q8_runs*.rs) on `QMem`.
//! Conventions as q2mem / q6 / q7util (ints in `a`, floats / quadwords in `v`, float result `.1[0]`).
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem, V4};
pub use crate::engine::classes::q::q6::{callf, ro32, rof, rov, s, set_tagged, vcall, CLOCK, WORLD};

/// FUN_001ae478(frame): open an allocator frame (the frame record itself is emulated stack)
pub fn frame_begin<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let f = stack_push(m, 0x10);
    m.call(0x001a_e478, &[f], &[]);
    f
}
/// FUN_001ae4a0(frame, flags): close the frame
pub fn frame_end<M: QMem + ?Sized>(m: &mut M, f: u32, flags: u32) {
    m.call(0x001a_e4a0, &[f, flags], &[]);
    stack_pop(m, 0x10);
}
/// FUN_001ae318(&{size, align}) through an emulated-stack request block
pub fn frame_alloc<M: QMem + ?Sized>(m: &mut M, size: u32, align: u32) -> u32 {
    let req = stack_push(m, 8);
    m.w32(req, size);
    m.w32(req + 4, align);
    let p = m.call(0x001a_e318, &[req], &[]).0;
    stack_pop(m, 8);
    p
}
/// 64-bit load / store as two words
pub fn r64<M: QMem + ?Sized>(m: &mut M, a: u32) -> u64 {
    m.r64(a)
}
pub fn w64<M: QMem + ?Sized>(m: &mut M, a: u32, v: u64) {
    m.w32(a, v as u32);
    m.w32(a + 4, (v >> 32) as u32);
}
/// `*(v4 *)dst = *(v4 *)src`
pub fn cpy4<M: QMem + ?Sized>(m: &mut M, dst: u32, src: u32) {
    let v = m.rv(src);
    m.wv(dst, v);
}
/// Q8 overrides whose whole body is `return <const>;`: (fn, value)
pub const CONST_RET: &[(u32, u32)] = &[
    (0x113d00, 0x2bae94),
    (0x114080, 0x2bb2f4),
    (0x114670, 0x2bba60),
    (0x114490, 0x2bb808),
    (0x114090, 0x2bb308),
    (0x1141c0, 0x2bb484),
    (0x1ef0b8, 0),
    (0x113f90, 0x2bb1c8),
    (0x113c30, 0x2bad90),
    (0x113d30, 0x2baed0),
    (0x1c3ed0, 0),
];
/// Q8 overrides whose body is `jr ra` only
pub const NOPS: &[u32] = &[0x1eefb8];
