//! WP-Q8: C2cc4c0 slot overrides (owner at +0x10, target vector +0x80, gain +0xd0).
use crate::engine::classes::q::q2mem::QMem;
use crate::engine::classes::q::q8util::*;

/// FUN_00144b28 (slot 6): parent init FUN_00177e80, +0xd0 = 1.0
pub fn ee_sub_144b28<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0017_7e80, &[o], &[]);
    m.w32(o + 0xd0, 0x3f80_0000);
}
/// FUN_00144b58: parent FUN_00177ef0; FUN_001448f0(owner + 0xfd0, +0x80)
pub fn ee_sub_144b58<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0017_7ef0, &[o], &[]);
    let (p, a) = (m.r32(o + 0x10), m.r32(o + 0x80));
    m.call(0x0014_48f0, &[p + 0xfd0, a], &[]);
}
/// FUN_00144b90: parent FUN_001789f8; FUN_001448f0(owner + 0xfd0, +0x80)
pub fn ee_sub_144b90<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0017_89f8, &[o], &[]);
    let (p, a) = (m.r32(o + 0x10), m.r32(o + 0x80));
    m.call(0x0014_48f0, &[p + 0xfd0, a], &[]);
}
/// FUN_00144bc8 (update into request p2): parent FUN_001ef0c0; for mode +0x4c in 0..=1: offset of the
/// target (+0x80) to the owner position minus its component along DAT_002a2dc0 (0,0,1,0); strength = cubic
/// (DAT_002bd2e0..e8) of the horizontal length clamped to DAT_002bbb60..64 -> p2[0x1c] = strength * gain,
/// p2[0x18] = atan2(y, x); flags p2[0x30] |= 8 (+0xcc set, or the +0xc0 object's owner further than 500.0 on
/// the xy plane); once +0xc8 s have passed since +0x50: merge the +0xc0 object's flags (& +0xc4), FUN_001c45a8(p2).
/// PARTIAL: VU0 lane selection of the dot / length / atan2 (xy plane, z = DAT_002a2dc0) inferred from the data.
pub fn ee_sub_144bc8<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32) {
    m.call(0x001e_f0c0, &[o, p2], &[]);
    let mode = m.r32(o + 0x4c) as i32;
    if !(mode < 2 && -1 < mode) {
        return;
    }
    let tgt = m.rv(o + 0x80);
    let own = m.r32(o + 0x10);
    let pos = m.call(0x001a_62e8, &[own], &[]).1;
    let d: [f32; 4] = std::array::from_fn(|i| tgt[i] - pos[i]);
    let k = rov(0x002a_2dc0);
    let dot = d[0] * k[0] + d[1] * k[1] + d[2] * k[2];
    let e: [f32; 4] = std::array::from_fn(|i| d[i] - k[i] * dot);
    let len = (e[0] * e[0] + e[1] * e[1]).sqrt();
    let c = rof(0x002b_d2e0) + len * (rof(0x002b_d2e4) + len * rof(0x002b_d2e8));
    let mut f = rof(0x002b_bb60);
    if f <= c {
        f = rof(0x002b_bb64);
        if c <= f {
            f = c;
        }
    }
    if 0.0 < f {
        let g = m.rf(o + 0xd0);
        m.wf(p2 + 0x1c, f * g);
        let a = callf(m, 0x0024_58f0, &[], &[s(e[1]), s(e[0])]);
        m.wf(p2 + 0x18, a);
    }
    if m.r32(o + 0xcc) != 0 {
        let fl = m.r32(p2 + 0x30);
        m.w32(p2 + 0x30, fl | 8);
    }
    let obj = m.r32(o + 0xc0);
    if obj != 0 {
        let mut ok = true;
        if m.rf(o + 200) < m.rf(CLOCK) - m.rf(o + 0x50) {
            let other = m.r32(obj + 0x7c);
            let a = m.call(0x001a_62e8, &[own], &[]).1;
            let b = m.call(0x001a_62e8, &[other], &[]).1;
            let dx: [f32; 4] = std::array::from_fn(|i| a[i] - b[i]);
            let dist = (dx[0] * dx[0] + dx[1] * dx[1]).sqrt();
            if rof(0x002b_d2f0) < dist {
                let fl = m.r32(p2 + 0x30);
                m.w32(p2 + 0x30, fl | 8);
            }
            if m.call(0x0013_2b10, &[other, 0x34], &[]).0 != 0 {
                ok = vcall(m, other, 0xc, 0x2cc, &[], &[]).0 != 0;
            }
            if ok {
                let ob = m.r32(o + 0xc0);
                let src = m.r32(ob + 0x30) & m.r32(o + 0xc4);
                let fl = m.r32(p2 + 0x30);
                m.w32(p2 + 0x30, fl | src);
                m.call(0x001c_45a8, &[p2], &[]);
            }
        }
    }
}
