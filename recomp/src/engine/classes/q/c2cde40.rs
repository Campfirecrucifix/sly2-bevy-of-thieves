//! WP-Q9: C2cde40 (record table: count +0x1010, array +0x1014 of 0x24-byte records, +0x1018 limit) slot overrides.
use crate::engine::classes::q::q2mem::QMem;
use crate::engine::classes::q::q9util::*;

/// FUN_001553f0 (v13)(this, p2): +0x1014 = alloc(0x480, 0x10); C2cbe68_v13 (FUN_002039c0)(this, p2); +0x1018 = 0x7fffffff
pub fn ee_sub_1553f0<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32) {
    let p = alloc(m, 0x480, 0x10);
    m.w32(o + 0x1014, p);
    m.call(0x0020_39c0, &[o, p2], &[]);
    m.w32(o + 0x1018, 0x7fff_ffff);
}
/// FUN_00155450 (v14): parent C2c8940_v14 (FUN_002061d8); per record (0x24 bytes): records[1, 3, 5, 7] =
/// FUN_0015e938(world 0x3dd490, 4, records[0, 2, 4, 6], this)
pub fn ee_sub_155450<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0020_61d8, &[o], &[]);
    let mut i = 0i32;
    let mut off = 0u32;
    if 0 < m.r32(o + 0x1010) as i32 {
        loop {
            let rec = m.r32(o + 0x1014).wrapping_add(off);
            i += 1;
            off += 0x24;
            for k in 0..4 {
                let key = m.r32(rec + 8 * k);
                let r = m.call(0x0015_e938, &[WORLD, 4, key, o], &[]).0;
                m.w32(rec + 8 * k + 4, r);
            }
            if m.r32(o + 0x1010) as i32 <= i {
                break;
            }
        }
    }
}
