//! WP-Q8: class C2d8568 (see 1daa48 / 1daa90 / 1db0c8)
use crate::engine::classes::q::q2mem::QMem;

/// FUN_001daa48: o+0x24 = 0 (FUN_001da7b0 inlined: FUN_001da7d0(o)); then FUN_001daa78(o)
pub fn ee_sub_1daa48<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.w32(o + 0x24, 0);
    m.call(0x001d_a7d0, &[o], &[]);
    m.w32(o + 0x40, u32::MAX);
    let c = m.r32(0x002d_df0c);
    m.w32(o + 0x44, c);
}
/// FUN_001daa90: after FUN_001da808(o): when +0x24 != 0 && +0x40 != -1 && +0x10 != 0 either FUN_001da858 + the
/// three-button prompt scan (table DAT_002c3258) or FUN_001db520(o, -1)
pub fn ee_sub_1daa90<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001d_a808, &[o], &[]);
    if m.r32(o + 0x24) == 0 || m.r32(o + 0x40) == u32::MAX || m.r32(o + 0x10) == 0 {
        return;
    }
    let ent = m.r32(o + 0x10);
    let mut l2 = 0;
    let mut ok = false;
    if m.r32(o + 0x30) == 0 || {
        l2 = m.call(0x0018_7d80, &[ent, 0x10], &[]).0;
        l2 == 0
    } {
        l2 = m.call(0x001c_4840, &[0x003d_d490], &[]).0;
        if l2 != 0 && m.call(0x0013_2b10, &[l2, 0x34], &[]).0 != 0 {
            ok = true;
        }
    }
    if !ok {
        m.call(0x001d_b520, &[o, u32::MAX], &[]);
        return;
    }
    m.call(0x001d_a858, &[o], &[]);
    let (p10, p48) = (m.r32(o + 0x10), m.r32(o + 0x48));
    if m.call(0x0018_7d80, &[p10, 0x7e8], &[]).0 != 0 {
        let e = m.call(0x001d_ae20, &[o, p48], &[]).0;
        if e == 0 || m.r32(e + 0x1c) & 1 == 0 {
            m.call(0x001d_a950, &[o, 1, 0xe2], &[]);
        } else {
            m.call(0x001d_a950, &[o, 1, 0x31], &[]);
        }
    }
    for i in 0..3u32 {
        let k = m.r32(0x002c_3258 + 4 * i);
        let (p10, p48) = (m.r32(o + 0x10), m.r32(o + 0x48));
        if m.call(0x0018_7d80, &[p10, k], &[]).0 != 0 {
            let e = m.call(0x001d_ae20, &[o, p48], &[]).0;
            let f = if e == 0 { 0 } else { m.r32(e + 0x1c) };
            if e == 0 || f & 4 == 0 {
                m.call(0x001d_a950, &[o, 1, 0x32], &[]);
            } else if f & 1 == 0 {
                m.call(0x001d_a950, &[o, 1, 0x33], &[]);
            } else {
                let u = m.call(0x001f_bb90, &[0x34], &[]).0;
                m.call(0x001f_b2f8, &[u, 0], &[]);
                let n = m.r32(o + 0x48);
                m.call(0x0014_1fc8, &[l2, i, n], &[]);
            }
        }
    }
}
