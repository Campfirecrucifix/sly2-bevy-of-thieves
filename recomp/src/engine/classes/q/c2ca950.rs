//! WP-Q9: C2ca950 (keyframe table: count +4, records of 0x20 bytes at +8) slot overrides.
use crate::engine::classes::q::q2mem::QMem;
use crate::engine::classes::q::q9util::*;

/// 0.016666668 = 1/60 (frame -> seconds)
const FRAME: f32 = 0.016666668;
/// FUN_0012ba98 (v0)(this, stream): n = read_u16; +4 = n; +8 = alloc(n << 5, 0x10); per record: +0xc = read_u16 / 60,
/// vec3 at +0 and +0x10 (FUN_00117eb0)
pub fn ee_sub_12ba98<M: QMem + ?Sized>(m: &mut M, o: u32, st: u32) {
    let n = read_u16(m, st);
    m.w32(o + 4, n);
    let buf = alloc(m, n << 5, 0x10);
    m.w32(o + 8, buf);
    let mut i = 0u32;
    while (m.r32(o + 4) as i32) > 0 {
        let rec = m.r32(o + 8).wrapping_add(i * 0x20);
        i += 1;
        let f = read_u16(m, st);
        m.wf(rec + 0xc, (f as i32) as f32 * FRAME);
        read_vec3(m, st, rec);
        read_vec3(m, st, rec + 0x10);
        if (m.r32(o + 4) as i32) <= i as i32 {
            break;
        }
    }
}
/// FUN_0012be68 (v2)(t, p2, this, p4, p5): i = FUN_0012bb70(t, this); p = table + i*0x20;
/// FUN_00117120(p[0xc] - p[-0x14], t - p[-0x14], p2, p[-0x20], p[-0x10], p[0], p[0x10], p4, p5, 0)
pub fn ee_sub_12be68<M: QMem + ?Sized>(m: &mut M, t: f32, p2: f32, o: u32, p4: u32, p5: u32) {
    let i = m.call(0x0012_bb70, &[o], &[s(t)]).0;
    let p = i.wrapping_mul(0x20).wrapping_add(m.r32(o + 8));
    let t0 = m.rf(p.wrapping_sub(0x14));
    let (a0, a1, a2, a3) = (m.r32(p.wrapping_sub(0x20)), m.r32(p.wrapping_sub(0x10)), m.r32(p), m.r32(p + 0x10));
    let f1 = m.rf(p + 0xc) - t0;
    m.call(0x0011_7120, &[a0, a1, a2, a3, p4, p5, 0], &[s(f1), s(t - t0), s(p2)]);
}
