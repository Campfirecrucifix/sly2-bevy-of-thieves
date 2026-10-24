//! PART: QMem ports of vtable slots of the EP1 live classes (C2cb230 / C2d44c8 / C2d0b30 / ...) so the live
//! queue dispatch (classes/q/live_dispatch.rs, generated) reaches them through vslot. 1:1 to the decomp.
use crate::engine::classes::q::q2mem::{vslot, QMem, V4};

/// const getters (`return &DAT_..`): class descriptor tables
pub fn ee_sub_113a80<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_ab74
}
pub fn ee_sub_113c80<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_adf4
}
pub fn ee_sub_113d70<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_af20
}
pub fn ee_sub_113f20<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_b13c
}
pub fn ee_sub_113fa0<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_b1dc
}
pub fn ee_sub_114010<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_b268
}
pub fn ee_sub_114120<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_b3bc
}
pub fn ee_sub_114200<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_b4d4
}

/// C2caf88_v54 FUN_00132238: C2cc548_v54 (0x1f4fb0, same args); if *o and FindBinding(*o, 5, 1) (0x127db8):
/// FUN_00149ab8(0x3dd490, o, DAT_002a4790, 0)
pub fn ee_sub_132238<M: QMem + ?Sized>(m: &mut M, o: u32, a1: u32, a2: u32) {
    m.call(0x001f_4fb0, &[o, a1, a2], &[]);
    let p = m.r32(o);
    if p != 0 && m.call(0x0012_7db8, &[p, 5, 1], &[]).0 != 0 {
        let d = m.r32(0x002a_4790);
        m.call(0x0014_9ab8, &[0x003d_d490, o, d, 0], &[]);
    }
}

/// C2cb230_v54 FUN_00135a48: unless the hit record (*(a1 + k*0x40)) +0x10 has bit 48: state +0x13d0 4 ->
/// FUN_001a2100(o+0xdf0, a1), 0xd -> FUN_0019e9c8(o+0xf50, a1), else C2cbe68_v54 (0x204250)
pub fn ee_sub_135a48<M: QMem + ?Sized>(m: &mut M, o: u32, a1: u32, k: u32) {
    let rec = m.r32(a1.wrapping_add(k.wrapping_mul(0x40)));
    if m.r64(rec.wrapping_add(0x10)) & 0x1_0000_0000_0000 != 0 {
        return;
    }
    match m.r32(o + 0x13d0) {
        4 => {
            m.call(0x001a_2100, &[o + 0xdf0, a1], &[]);
        }
        0xd => {
            m.call(0x0019_e9c8, &[o + 0xf50, a1], &[]);
        }
        _ => {
            m.call(0x0020_4250, &[o, a1, k], &[]);
        }
    }
}

/// FUN_00135d08: state +0x13d0 -> anim index
pub fn ee_sub_135d08<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    match m.r32(o + 0x13d0) {
        0 => 0,
        2 => 1,
        3 => 2,
        4 => 3,
        9 => 4,
        10 => 5,
        0xb => 6,
        0xc => 7,
        0xd => 8,
        _ => 0xffff_ffff,
    }
}

pub fn ee_sub_135e38<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0
}

/// FUN_00135ff0: state +0x13d0 in {0, 1, 5, 7}
pub fn ee_sub_135ff0<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let s = m.r32(o + 0x13d0) as i32;
    (s == 5 || s == 7 || (0..2).contains(&s)) as u32
}

/// FUN_00136cc0: +0x204 set -> 0; state 0xd -> 0, 8 -> 5, 0xe -> 0xf, else C2cbe68_v84 (0x143c78)
pub fn ee_sub_136cc0<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    if m.r32(o + 0x204) != 0 {
        return 0;
    }
    match m.r32(o + 0x13d0) {
        0xd => 0,
        8 => 5,
        0xe => 0xf,
        _ => ee_sub_143c78(m, o),
    }
}

/// C2cb230_v92 FUN_00140eb0: FollowWorld_activity(0x2deb30, 0x10, o)
pub fn ee_sub_140eb0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0016_19a0, &[0x002d_eb30, 0x10, o], &[]);
}

/// C2cb230_v103 FUN_00141e30: message slot 2 (vt +0x1c) (0x1a, 0, 0, 0, 0)
pub fn ee_sub_141e30<M: QMem + ?Sized>(m: &mut M, o: u32) {
    vslot(m, o, 2, &[0x1a, 0, 0, 0, 0], &[]);
}

/// C2cbe68_v104 FUN_00141e70: empty
pub fn ee_sub_141e70<M: QMem + ?Sized>(_m: &mut M) {}

/// C2cbe68_v84 FUN_00143c78
pub fn ee_sub_143c78<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    if m.r32(o + 0x204) != 0 {
        return 0;
    }
    let s = m.r32(o + 0x4c8) as i32;
    if (3..5).contains(&s) {
        return 4;
    }
    if m.r32(o + 0x2dc) as i32 == -1 {
        match m.r32(o + 0xdf4) as i32 {
            4 | 7 | 8 | 0xc => {}
            -1 | 0 | 6 => return 7,
            _ => return 0,
        }
    }
    1
}

/// FUN_00144090: C2d5898_v79 (0x1c41b8) && FUN_001a23c0(o+0xdf0) != 2
pub fn ee_sub_144090<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    if m.call(0x001c_41b8, &[o], &[]).0 == 0 {
        return 0;
    }
    (m.call(0x001a_23c0, &[o + 0xdf0], &[]).0 != 2) as u32
}

/// C2d0b30_v54 FUN_00182700: C2d0dd8_v54 (0x17f1a0); with +0x188: the other party u = *(a1 + (1-k)*0x40 + 0xc),
/// FUN_00132b10(u, 0x47) -> FUN_0017f5b0(o, u)
pub fn ee_sub_182700<M: QMem + ?Sized>(m: &mut M, o: u32, a1: u32, k: u32) {
    m.call(0x0017_f1a0, &[o, a1, k], &[]);
    if m.r32(o + 0x188) != 0 {
        let u = m.r32(a1.wrapping_add(1u32.wrapping_sub(k).wrapping_mul(0x40)).wrapping_add(0xc));
        if m.call(0x0013_2b10, &[u, 0x47], &[]).0 != 0 {
            m.call(0x0017_f5b0, &[o, u], &[]);
        }
    }
}

pub fn ee_sub_182778<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    1
}

/// C2d3838_v69 FUN_001b10b0: PoMoveToward(o, o+0x1e0, o+0x210)
pub fn ee_sub_1b10b0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001f_59a0, &[o, o + 0x1e0, o + 0x210], &[]);
}

/// FUN_001b7f90: state +0x13d0 -> anim index
pub fn ee_sub_1b7f90<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    match m.r32(o + 0x13d0) {
        0 | 5 => 0,
        2 => 1,
        0xd => 2,
        0xe => 3,
        0x10 => 4,
        0x11 => 5,
        0x12 => 6,
        0x13 => 7,
        0x14 => 8,
        _ => 0xffff_ffff,
    }
}

pub fn ee_sub_1b80f8<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0
}

/// FUN_001b8368: +0x204 set, +0x13d8 == 1 or state 0x14 -> 0, else C2cbe68_v84 (0x143c78)
pub fn ee_sub_1b8368<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    if m.r32(o + 0x204) != 0 || m.r32(o + 0x13d8) == 1 || m.r32(o + 0x13d0) == 0x14 {
        return 0;
    }
    ee_sub_143c78(m, o)
}

/// FUN_0011f198(a, b): cosine of the angle between a and b (0 when either is shorter than 0.0001)
pub fn ee_sub_11f198<M: QMem + ?Sized>(_m: &mut M, a: V4, b: V4) -> f32 {
    crate::engine::classes::k18_54::ee_sub_11f198([a[0], a[1], a[2]], [b[0], b[1], b[2]])
}

/// C2d44c8_v54 FUN_001b9130 (contact): C2cbe68_v54 (0x204250); state 0xc: with +0x1840 == 1, the other party's
/// slot 61 true and contact +0x88 < 0.7: FUN_0011f198(slot 32 vector, contact +0x80) < DAT_002c0a38 -> +0x1908 = 1;
/// state 3 with +0x13d8 == 1 and 0.7 < DAT_002c3c60[k] * +0x88: FUN_001a9fe0(0x3dd490, FUN_001aa020(0x3dd490, a1),
/// o+0x13e4); 0xe -> FUN_001a2100(o+0xdf0, a1, k); 0x14 -> FUN_0019e9c8(o+0xf50, a1, k)
pub fn ee_sub_1b9130<M: QMem + ?Sized>(m: &mut M, o: u32, a1: u32, k: u32) {
    m.call(0x0020_4250, &[o, a1, k], &[]);
    match m.r32(o + 0x13d0) {
        0xc => {
            if m.r32(o + 0x1840) != 1 {
                return;
            }
            let u = m.r32(a1.wrapping_add(1u32.wrapping_sub(k).wrapping_mul(0x40)).wrapping_add(0xc));
            if vslot(m, u, 61, &[], &[]).0 == 0 || !(m.rf(a1 + 0x88) < 0.7) {
                return;
            }
            let v = vslot(m, o, 32, &[], &[]).1;
            let n: V4 = core::array::from_fn(|i| m.rf(a1 + 0x80 + 4 * i as u32));
            if ee_sub_11f198(m, v, n) < m.rf(0x002c_0a38) {
                m.w32(o + 0x1908, 1);
            }
        }
        3 => {
            if m.r32(o + 0x13d8) == 1 && 0.7 < m.rf(0x002c_3c60u32.wrapping_add(k.wrapping_mul(4))) * m.rf(a1 + 0x88) {
                let r = m.call(0x001a_a020, &[0x003d_d490, a1], &[]).0;
                m.call(0x001a_9fe0, &[0x003d_d490, r, o + 0x13e4], &[]);
            }
        }
        0xe => {
            m.call(0x001a_2100, &[o + 0xdf0, a1, k], &[]);
        }
        0x14 => {
            m.call(0x0019_e9c8, &[o + 0xf50, a1, k], &[]);
        }
        _ => {}
    }
}

/// Guard FUN_0020b090: flag 0x14 (FUN_00207a20 = 1 << n) in +0x2c0 -> 1; any of 0x12 / 0x13 / 0x16 -> 0; else
/// +0xfd0 sub-object slot 11 (vt +100) or C2cd748_v84 (0x2040f8)
pub fn ee_sub_20b090<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let bit = |m: &mut M, n: u32| m.call(0x0020_7a20, &[n], &[]).0 as u64;
    let f = m.r64(o + 0x2c0);
    if f & bit(m, 0x14) != 0 {
        return 1;
    }
    let mask = bit(m, 0x12) | bit(m, 0x13) | bit(m, 0x16);
    if f & mask != 0 {
        return 0;
    }
    let s = m.r32(o + 0xfd0);
    if s == 0 {
        m.call(0x0020_40f8, &[o], &[]).0
    } else {
        vslot(m, s, 11, &[], &[]).0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::classes::q::q2mem::FlatMem;

    #[test]
    fn part4_state_maps() {
        let mut m = FlatMem::default();
        m.w32(0x1000 + 0x13d0, 9);
        assert_eq!(ee_sub_135d08(&mut m, 0x1000), 4);
        assert_eq!(ee_sub_135ff0(&mut m, 0x1000), 0);
        m.w32(0x1000 + 0x13d0, 7);
        assert_eq!(ee_sub_135ff0(&mut m, 0x1000), 1);
        assert_eq!(ee_sub_135d08(&mut m, 0x1000), 0xffff_ffff);
        m.w32(0x1000 + 0x13d0, 0x14);
        assert_eq!(ee_sub_1b7f90(&mut m, 0x1000), 8);
        assert_eq!(ee_sub_1b8368(&mut m, 0x1000), 0);
    }

    #[test]
    fn part4_143c78() {
        let mut m = FlatMem::default();
        let o = 0x2000;
        m.w32(o + 0x4c8, 3);
        assert_eq!(ee_sub_143c78(&mut m, o), 4);
        m.w32(o + 0x4c8, 0);
        m.w32(o + 0x2dc, 0xffff_ffff);
        m.w32(o + 0xdf4, 6);
        assert_eq!(ee_sub_143c78(&mut m, o), 7);
        m.w32(o + 0xdf4, 4);
        assert_eq!(ee_sub_143c78(&mut m, o), 1);
        m.w32(o + 0xdf4, 2);
        assert_eq!(ee_sub_143c78(&mut m, o), 0);
        m.w32(o + 0x13d0, 8);
        assert_eq!(ee_sub_136cc0(&mut m, o), 5);
        m.w32(o + 0x204, 1);
        assert_eq!(ee_sub_143c78(&mut m, o), 0);
    }

    #[test]
    fn part4_cos_and_1b9130() {
        let mut m = FlatMem::default();
        assert!((ee_sub_11f198(&mut m, [1.0, 0.0, 0.0, 0.0], [2.0, 0.0, 0.0, 0.0]) - 1.0).abs() < 1e-6);
        assert_eq!(ee_sub_11f198(&mut m, [0.0; 4], [1.0, 0.0, 0.0, 0.0]), 0.0);
        m.w32(0x4000 + 0x13d0, 0xe);
        ee_sub_1b9130(&mut m, 0x4000, 0x5000, 0);
        assert_eq!(m.called(), vec![0x204250, 0x1a2100]);
    }

    #[test]
    fn part4_calls() {
        let mut m = FlatMem::default();
        ee_sub_1b10b0(&mut m, 0x3000);
        ee_sub_140eb0(&mut m, 0x3000);
        assert_eq!(m.called(), vec![0x1f59a0, 0x1619a0]);
    }
}
