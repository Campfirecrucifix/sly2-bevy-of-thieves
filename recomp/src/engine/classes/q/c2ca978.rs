//! WP-Q9: C2ca978 (keyframe table with records of 0x30 bytes) load override.
use crate::engine::classes::q::q2mem::QMem;
use crate::engine::classes::q::q9util::*;

/// FUN_0012b5e0 (v0)(this, stream): n = read_u16; +4 = n; +8 = alloc(n * 0x30, 0x10); per record: +0xc = read_u16 / 60,
/// vec3 at +0, +0x10, +0x20
pub fn ee_sub_12b5e0<M: QMem + ?Sized>(m: &mut M, o: u32, st: u32) {
    let n = read_u16(m, st);
    m.w32(o + 4, n);
    let buf = alloc(m, n.wrapping_mul(0x30), 0x10);
    m.w32(o + 8, buf);
    let mut i = 0u32;
    let mut off = 0u32;
    while (m.r32(o + 4) as i32) > 0 {
        i += 1;
        let rec = m.r32(o + 8).wrapping_add(off);
        let f = read_u16(m, st);
        off += 0x30;
        m.wf(rec + 0xc, (f as i32) as f32 * 0.016666668);
        read_vec3(m, st, rec);
        read_vec3(m, st, rec + 0x10);
        read_vec3(m, st, rec + 0x20);
        if (m.r32(o + 4) as i32) <= i as i32 {
            break;
        }
    }
}
