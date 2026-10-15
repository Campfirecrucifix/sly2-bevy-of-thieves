//! WP-Q8: C2cc1c0 (owner object at +0x10 with sub-state +0x1840) slot overrides.
use crate::engine::classes::q::q2mem::QMem;
use crate::engine::classes::q::q8util::*;

/// DAT_002e1e40: the hero (look-at target)
const HERO: u32 = 0x002e_1e40;

/// FUN_00146e98: parent FUN_001eef70; owner +0x1ac0 = 1; FUN_001b6078(owner, 0xc); FUN_001b8d30(owner, 0)
pub fn ee_sub_146e98<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef70, &[o], &[]);
    let p = m.r32(o + 0x10);
    m.w32(p + 0x1ac0, 1);
    m.call(0x001b_6078, &[p, 0xc], &[]);
    m.call(0x001b_8d30, &[p, 0], &[]);
}
/// FUN_00146ee0: parent FUN_001eef78; owner +0x1ac0 = 0; FUN_001b6078(owner, 0)
pub fn ee_sub_146ee0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef78, &[o], &[]);
    let p = m.r32(o + 0x10);
    m.w32(p + 0x1ac0, 0);
    m.call(0x001b_6078, &[p, 0], &[]);
}
/// FUN_00146f18 (update): parent FUN_001eef98; owner sub-state 0: turn the owner towards the hero
/// (FUN_002046b0(owner, hero - owner)) and remember its +0x1a8 in +0x30; sub-state 1: ease the owner's yaw
/// (FUN_00204740) towards atan2 of the offset (FUN_002458f0) by DAT_002bd378 * dt when within DAT_002bd374
pub fn ee_sub_146f18<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef98, &[o], &[]);
    let p = m.r32(o + 0x10);
    let st = m.r32(p + 0x1840);
    let hero = m.r32(HERO);
    if st == 0 {
        let tgt = if hero == 0 {
            p
        } else {
            let a = m.call(0x001a_62e8, &[hero], &[]).1;
            let b = m.call(0x001a_62e8, &[p], &[]).1;
            let d: [f32; 4] = std::array::from_fn(|i| a[i] - b[i]);
            m.call(0x0020_46b0, &[p], &[d]);
            m.r32(o + 0x10)
        };
        let v = m.r32(tgt + 0x1a8);
        m.w32(o + 0x30, v);
    } else if st == 1 && hero != 0 {
        let a = m.call(0x001a_62e8, &[hero], &[]).1;
        let b = m.call(0x001a_62e8, &[p], &[]).1;
        let d: [f32; 4] = std::array::from_fn(|i| a[i] - b[i]);
        let yaw = callf(m, 0x0024_58f0, &[], &[s(d[1]), s(d[0])]);
        let y0 = m.rf(o + 0x30);
        let diff = callf(m, 0x0011_cf60, &[], &[s(yaw - y0)]);
        if diff.abs() < rof(0x002b_d374) {
            let cur = callf(m, 0x0020_4740, &[p], &[]);
            let rest = callf(m, 0x0011_cf60, &[], &[s(yaw - cur)]);
            let dt = m.rf(0x002d_ded4);
            let step = callf(m, 0x0011_cfe0, &[], &[s(rest), s(rof(0x002b_d378) * dt)]);
            m.call(0x0020_4660, &[p], &[s(cur + step)]);
        }
    }
}
/// FUN_001eefa0: 7 unless the owner's +0x204 is set
pub fn ee_sub_1eefa0<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let p = m.r32(o + 0x10);
    if m.r32(p + 0x204) != 0 {
        0
    } else {
        7
    }
}
