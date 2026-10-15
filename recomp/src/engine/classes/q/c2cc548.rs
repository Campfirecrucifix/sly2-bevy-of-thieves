//! WP-Q8: C2cc548 (state machine +0x150, handle +0x160, flags +0x164..+0x16c, sound +0x170) slot overrides.
use crate::engine::classes::q::q2mem::QMem;
use crate::engine::classes::q::q8util::*;

/// FUN_001471c0 (slot 6): parent init FUN_001f2108, +0x150 = -1, FUN_001a6ca8(this, 1), +0x40 flags |= 0x102
pub fn ee_sub_1471c0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001f_2108, &[o], &[]);
    m.w32(o + 0x150, u32::MAX);
    m.call(0x001a_6ca8, &[o, 1], &[]);
    set_tagged(m, o + 0x40, 0x102);
}
/// FUN_00147220: parent FUN_001f36a8; FUN_001a64e8(this, 2, 0x2bd380); +0x150 = -1; vslot 0x4c(this)
pub fn ee_sub_147220<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001f_36a8, &[o], &[]);
    m.call(0x001a_64e8, &[o, 2, 0x002b_d380], &[]);
    m.w32(o + 0x150, u32::MAX);
    vcall(m, o, 0xc, 0x4c, &[], &[]);
}
/// FUN_00147278 (update): state 0 -> 1; state 2: once +0x160 is 0 or FUN_00131968(+0x160) -> release it
/// (FUN_001306a8) and state -1; state 3: FUN_001fe848(0x3dd040, +0x170) == 0 -> state 2 (FUN_00147410)
pub fn ee_sub_147278<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let st = m.r32(o + 0x150) as i32;
    if st == 2 {
        let h = m.r32(o + 0x160);
        if h == 0 || m.call(0x0013_1968, &[h], &[]).0 != 0 {
            m.call(0x0013_06a8, &[o + 0x160], &[]);
            m.call(0x0014_7410, &[o, u32::MAX], &[]);
        }
    } else if st < 3 {
        if st == 0 {
            m.call(0x0014_7410, &[o, 1], &[]);
        }
    } else if st == 3 {
        let a = m.r32(o + 0x170);
        if m.call(0x001f_e848, &[0x003d_d040, a], &[]).0 == 0 {
            m.call(0x0014_7410, &[o, 2], &[]);
        }
    }
}
/// FUN_00147338 (message): parent FUN_00133590; msg 9 from handle +0x160: event (+8) 0x5fb/0x5fc -> +0x164 = 1/0,
/// 0x5fd/0x5fe -> +0x168 = 1/0, 0x5ff/0x600 -> +0x16c = 0/1, 0x601 with +0x170 != -1 -> state 3
pub fn ee_sub_147338<M: QMem + ?Sized>(m: &mut M, o: u32, msg: u32, arg: u32) {
    m.call(0x0013_3590, &[o, msg, arg], &[]);
    if msg == 9 && m.r32(arg) == m.r32(o + 0x160) {
        match m.r32(arg + 8) {
            0x5fb => m.w32(o + 0x164, 1),
            0x5fc => m.w32(o + 0x164, 0),
            0x5fd => m.w32(o + 0x168, 1),
            0x5fe => m.w32(o + 0x168, 0),
            0x5ff => m.w32(o + 0x16c, 0),
            0x600 => m.w32(o + 0x16c, 1),
            0x601 => {
                if m.r32(o + 0x170) != u32::MAX {
                    m.call(0x0014_7410, &[o, 3], &[]);
                }
            }
            _ => {}
        }
    }
}
