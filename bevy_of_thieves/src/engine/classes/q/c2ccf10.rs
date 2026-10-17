//! WP-Q8: C2ccf10 (count +0x54, records {_, id, handle} * 0xc at +0x58, block +0x5c) slot override.
use crate::engine::classes::q::q2mem::QMem;
use crate::engine::classes::q::q8util::*;

/// FUN_0014a0a8 (slot 13, load): parent FUN_001a5c48; block +0x5c = FUN_001e94b0(0x62c, count) with its +0xe8 =
/// count; each record resolves its id (FUN_0015e938(world, 4, id, this)) into +8 and its position
/// (FUN_001a62e8) goes to block[+0xf4][i]; FUN_001e6180(block)
pub fn ee_sub_14a0a8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001a_5c48, &[o], &[]);
    let n = m.r32(o + 0x54);
    let blk = m.call(0x001e_94b0, &[0x62c, n], &[]).0;
    m.w32(o + 0x5c, blk);
    let n = m.r32(o + 0x54);
    m.w32(blk + 0xe8, n);
    let (mut i, mut off) = (0u32, 0u32);
    while (i as i32) < m.r32(o + 0x54) as i32 {
        let rec = m.r32(o + 0x58).wrapping_add(off);
        let id = m.r32(rec + 4);
        let h = m.call(0x0015_e938, &[WORLD, 4, id, o], &[]).0;
        m.w32(rec + 8, h);
        if h != 0 {
            let b5 = m.r32(o + 0x5c);
            let arr = m.r32(b5 + 0xf4);
            let p = m.call(0x001a_62e8, &[h], &[]).1;
            m.wv(arr + i * 0x10, p);
        }
        i += 1;
        off += 0xc;
    }
    let b = m.r32(o + 0x5c);
    m.call(0x001e_6180, &[b], &[]);
}
