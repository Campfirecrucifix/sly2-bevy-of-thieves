//! WP-Q9: C2ca9a0 (constant vec3 at +4) slot overrides.
use crate::engine::classes::q::q2mem::QMem;

/// FUN_0012b4f8 (v0)(this, stream): BinaryInputStream_read_117968(stream, 0xc, this + 4)
pub fn ee_sub_12b4f8<M: QMem + ?Sized>(m: &mut M, o: u32, st: u32) {
    m.call(0x0011_7968, &[st, 0xc, o + 4], &[]);
}
/// FUN_0012b590 (v2)(p2, p3): p2 != 0 -> FUN_001ae5a0(p2, this + 4, 0xc); p3 != 0 -> xyz of p3 = 0
pub fn ee_sub_12b590<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32, p3: u32) {
    if p2 != 0 {
        m.call(0x001a_e5a0, &[p2, o + 4, 0xc], &[]);
    }
    if p3 != 0 {
        m.w32(p3, 0);
        m.w32(p3 + 8, 0);
        m.w32(p3 + 4, 0);
    }
}
