//! WP-Q9: C2cd0a0 slot 15.
use crate::engine::classes::q::q2mem::QMem;

/// FUN_0014faf8 (v15): parent C2c8940_v15 (FUN_00207350); if +0xf80 == -1 and +0x208 != 0: FUN_00205910(this, 0x2bd940, 3, 0x2bd910)
pub fn ee_sub_14faf8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0020_7350, &[o], &[]);
    if m.r32(o + 0xf80) == u32::MAX && m.r32(o + 0x208) != 0 {
        m.call(0x0020_5910, &[o, 0x002b_d940, 3, 0x002b_d910], &[]);
    }
}
