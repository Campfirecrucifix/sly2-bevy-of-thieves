//! WP-Q9: C2ca6e0 (timed segment: data +0xa8 end time, +0xc0 / +0xd0 end points) slot overrides.
use crate::engine::classes::q::q2mem::QMem;
use crate::engine::classes::q::q9util::*;

/// FUN_0012e010 (v3): data(+8)+0xa8 = *(this+0x1c); data+0xc0 = quad +0x20; data+0xd0 = quad +0x30;
/// data+0x9a (s16) = 2; FUN_0022c818(*(this+4), data)
pub fn ee_sub_12e010<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let d = m.r32(o + 8);
    let t = m.r32(o + 0x1c);
    m.w32(d + 0xa8, t);
    cpyw(m, d + 0xc0, o + 0x20, 4);
    let d = m.r32(o + 8);
    cpyw(m, d + 0xd0, o + 0x30, 4);
    let d = m.r32(o + 8);
    m.w16(d + 0x9a, 2);
    let (a, d) = (m.r32(o + 4), m.r32(o + 8));
    m.call(0x0022_c818, &[a, d], &[]);
}
/// FUN_0012e060 (v4)(dt, p3, p4): data+0xa8 <= clock + dt -> p3 = quad +0x20, p4 = quad +0x30; else
/// FUN_00117120(data+0xa8 - clock, dt, 1.0, data+0x30, data+0xb0, +0x20, +0x30, p3, p4, 0) (Hermite sample)
pub fn ee_sub_12e060<M: QMem + ?Sized>(m: &mut M, dt: f32, o: u32, p3: u32, p4: u32) {
    let d = m.r32(o + 8);
    let end = m.rf(d + 0xa8);
    let clock = m.rf(CLOCK);
    if end <= clock + dt {
        if p3 != 0 {
            cpyw(m, p3, o + 0x20, 4);
        }
        if p4 != 0 {
            cpyw(m, p4, o + 0x30, 4);
        }
    } else {
        let (a0, a1, a2, a3) = (m.r32(d + 0x30), m.r32(d + 0xb0), m.r32(o + 0x20), m.r32(o + 0x30));
        m.call(0x0011_7120, &[a0, a1, a2, a3, p3, p4, 0], &[s(end - clock), s(dt), s(1.0)]);
    }
}
