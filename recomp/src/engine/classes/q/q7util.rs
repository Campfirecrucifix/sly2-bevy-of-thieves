//! WP-Q7: small helpers shared by the Q7 queue ports (classes/q/c*.rs, engine/*/q_*.rs), all on `QMem`.
//! Calling convention used with `QMem::call` (same as Q2): integer / pointer args in `a`; float args
//! ($f12..) and 128-bit quadword args (VU0 vectors passed in GPRs) in `v`, floats first then quadwords,
//! each in source order; a float result is `.1[0]`, a quadword result `.1`.
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem, V4};

/// DAT_002dd2fc: head of the safe-iteration chain (DList iterators registered while the body may unlink)
pub const ITER_CHAIN: u32 = 0x002d_d2fc;
/// DAT_002dded0: the world clock
pub const CLOCK: u32 = 0x002d_ded0;
/// &DAT_003dd490: the world
pub const WORLD: u32 = 0x003d_d490;

/// float argument as a V4 lane-x
pub fn f(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}
/// float result of a call
pub fn fr(r: (u32, V4)) -> f32 {
    r.1[0]
}
pub fn sub(a: V4, b: V4) -> V4 {
    std::array::from_fn(|i| a[i] - b[i])
}
pub fn add(a: V4, b: V4) -> V4 {
    std::array::from_fn(|i| a[i] + b[i])
}
pub fn scale(a: V4, s: f32) -> V4 {
    a.map(|x| x * s)
}
/// `vmulax/vmadday/vmaddz` with rows r0..r2: r0*v.x + r1*v.y + r2*v.z (lanes xyz; w kept from `w`)
pub fn mul3(r: &[V4; 3], v: V4, w: f32) -> V4 {
    let mut o: V4 = std::array::from_fn(|i| r[0][i] * v[0] + r[1][i] * v[1] + r[2][i] * v[2]);
    o[3] = w;
    o
}
/// the 3 basis rows of the matrix at `a`
pub fn rows3<M: QMem + ?Sized>(m: &mut M, a: u32) -> [V4; 3] {
    [m.rv(a), m.rv(a + 0x10), m.rv(a + 0x20)]
}
/// copy `n` bytes (word-wise) from `src` to `dst`
pub fn copy<M: QMem + ?Sized>(m: &mut M, dst: u32, src: u32, n: u32) {
    for i in (0..n).step_by(4) {
        let w = m.r32(src + i);
        m.w32(dst + i, w);
    }
}

/// Safe DList walk: iterator record {list, &next, offset, prev chain} registered on DAT_002dd2fc while
/// `each(elem)` runs (the body may unlink the element; the unlinker patches the iterator). The list header
/// is {head, tail, link offset}.
pub fn safe_walk<M: QMem + ?Sized>(m: &mut M, list: u32, mut each: impl FnMut(&mut M, u32)) {
    let it = stack_push(m, 0x10);
    let off = m.r32(list + 8);
    let mut c = m.r32(list);
    m.w32(it, list);
    m.w32(it + 4, c.wrapping_add(off));
    m.w32(it + 8, off);
    let prev = m.r32(ITER_CHAIN);
    m.w32(it + 0xc, prev);
    m.w32(ITER_CHAIN, it);
    while c != 0 {
        each(m, c);
        let nx = m.r32(it + 4);
        c = m.r32(nx);
        let off = m.r32(it + 8);
        m.w32(it + 4, c.wrapping_add(off));
    }
    let prev = m.r32(it + 0xc);
    m.w32(ITER_CHAIN, prev);
    stack_pop(m, 0x10);
}

/// plain DList walk (no iterator registration): elements from `*list`, next = *(e + *(list + 8))
pub fn dlist<M: QMem + ?Sized>(m: &mut M, list: u32) -> Vec<u32> {
    let off = m.r32(list + 8);
    let mut v = vec![];
    let mut c = m.r32(list);
    while c != 0 {
        v.push(c);
        c = m.r32(c + off);
    }
    v
}
