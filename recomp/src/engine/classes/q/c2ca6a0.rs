//! WP-Q9: C2ca6a0 (base of the 0x2ca6a0 path-segment family: parent object +4 / data +8) slot overrides.
use crate::engine::classes::q::q2mem::QMem;
use crate::engine::classes::q::q9util::*;

/// FUN_0012ced8 (v1) / FUN_0012cf28 (v3): empty
pub fn ee_sub_12ced8() {}
pub fn ee_sub_12cf28() {}
/// FUN_0012cf30 (v4): p2 != 0 -> *(v4*)p2 = FUN_0022ca18(*(this+8), *(this+8)); p3 != 0 -> xyz of p3 = 0 (w kept)
pub fn ee_sub_12cf30<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32, p3: u32) {
    if p2 != 0 {
        let d = m.r32(o + 8);
        let r = m.call(0x0022_ca18, &[d, d], &[]).1;
        m.wv(p2, r);
    }
    if p3 != 0 {
        m.w32(p3, 0);
        m.w32(p3 + 8, 0);
        m.w32(p3 + 4, 0);
    }
}
/// FUN_0012e120 (v0): C2ca6e0_v0 (FUN_0012ce98)(this); the quad at +0x20 -> data(+8)+0xb0 and +0xc0, then the
/// +0xc0 quad is normalized (xyz length, < 1e-4 -> the quad at DAT_002a2da0); data+0xd0 = length, data+0xe0 = 0
pub fn ee_sub_12e120<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0012_ce98, &[o], &[]);
    let q = m.rv(o + 0x20);
    let d = m.r32(o + 8);
    m.wv(d + 0xb0, q);
    m.wv(d + 0xc0, q);
    let (n, len) = vu_normalize(q, rov(0x002a_2da0));
    m.wv(d + 0xc0, n);
    m.wf(d + 0xd0, len);
    let d = m.r32(o + 8);
    m.w32(d + 0xe0, 0);
}
