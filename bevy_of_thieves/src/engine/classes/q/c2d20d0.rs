//! WP-Q8: C2d20d0 (state +0x17c, enable +0x18c, effect object +0x190) slot overrides.
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem};
use crate::engine::classes::q::q8util::*;

/// FUN_00183900 (slot 6): parent init FUN_001f2108; FUN_00182c70(this + 0x150); +0x17c = 0, +0x158 = 300.0,
/// +0x18c = 1; FUN_001f7918(this, 0x3ffffff, 8); FUN_001f7918(this, 0x3188001, 0)
pub fn ee_sub_183900<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001f_2108, &[o], &[]);
    m.call(0x0018_2c70, &[o + 0x150], &[]);
    m.w32(o + 0x17c, 0);
    m.w32(o + 0x158, 0x4396_0000);
    m.w32(o + 0x18c, 1);
    m.call(0x001f_7918, &[o, 0x03ff_ffff, 8], &[]);
    m.call(0x001f_7918, &[o, 0x0318_8001, 0], &[]);
}
/// FUN_00183970: parent FUN_001cb518; +0x17c = 0
pub fn ee_sub_183970<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001c_b518, &[o], &[]);
    m.w32(o + 0x17c, 0);
}
/// FUN_00183998: parent FUN_001f3ea8; FUN_0011a220(0x3e13e4, this)
pub fn ee_sub_183998<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001f_3ea8, &[o], &[]);
    m.call(0x0011_a220, &[0x003e_13e4, o], &[]);
}
/// FUN_001839d0: parent FUN_001f3e00; FUN_0011a2e8(0x3e13e4, this)
pub fn ee_sub_1839d0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001f_3e00, &[o], &[]);
    m.call(0x0011_a2e8, &[0x003e_13e4, o], &[]);
}
/// FUN_00183a08: parent FUN_001f6068 preserving the u64 at +0x174
pub fn ee_sub_183a08<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let v = r64(m, o + 0x174);
    m.call(0x001f_6068, &[o], &[]);
    w64(m, o + 0x174, v);
}
/// FUN_00183a50: r = parent FUN_001f4fc8(this, p2); for a p2 of class 0x34 (FUN_00132b10): FUN_00134310(r, 1) when
/// (+0x4c8 == 1 and this == p2[+0x4d4]) and again when (p2[+0x2dc] - 6 < 2 (unsigned) and this == p2[+0x2cc])
pub fn ee_sub_183a50<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32) -> u32 {
    let mut r = m.call(0x001f_4fc8, &[o, p2], &[]).0;
    if m.call(0x0013_2b10, &[p2, 0x34], &[]).0 != 0 {
        if m.r32(p2 + 0x4c8) == 1 && o == m.r32(p2 + 0x4d4) {
            r = m.call(0x0013_4310, &[r, 1], &[]).0;
        }
        let st = m.r32(p2 + 0x2dc);
        if st.wrapping_sub(6) < 2 && o == m.r32(p2 + 0x2cc) {
            r = m.call(0x0013_4310, &[r, 1], &[]).0;
        }
    }
    r
}
/// FUN_00183b00: parent FUN_001f9b20; with +0x18c set and +0x17c == 0: FUN_0011b938(FUN_001a6318(this), 0, buf);
/// FUN_001f5c30(this, buf, 0x2be8d0, DAT_002a2dd0, 0x2be8e0)
pub fn ee_sub_183b00<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001f_9b20, &[o], &[]);
    if m.r32(o + 0x18c) != 0 && m.r32(o + 0x17c) == 0 {
        let mt = m.call(0x001a_6318, &[o], &[]).0;
        let buf = stack_push(m, 0x30);
        m.call(0x0011_b938, &[mt, 0, buf], &[]);
        m.call(0x001f_5c30, &[o, buf, 0x002b_e8d0, ro32(0x002a_2dd0), 0x002b_e8e0], &[]);
        stack_pop(m, 0x30);
    }
}
/// FUN_00183e08 (message-ish, p2 = event): FUN_001f7588 == 0 -> 0; else 1; unless state 3, an event with flag 8
/// (+0x44) spawns FUN_00156b98(world, 0, this @ pos), stops the +0x190 object (FUN_0019cd38(+0xf00, -1), vslot
/// 0x2c4(0)), enters state 3 (FUN_00183b80) and posts vslot 0x1c(0xb, 0, 2, &{p2, p2 + 0xc}, 0)
pub fn ee_sub_183e08<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32) -> u32 {
    if m.call(0x001f_7588, &[o, p2], &[]).0 == 0 {
        return 0;
    }
    if m.r32(o + 0x17c) != 3 && m.r32(p2 + 0x44) & 8 != 0 {
        let pos = m.call(0x001a_62e8, &[o], &[]).1;
        m.call(0x0015_6b98, &[WORLD, 0, o], &[pos]);
        let fx = m.r32(o + 400);
        if fx != 0 {
            m.call(0x0019_cd38, &[fx + 0xf00, u32::MAX], &[]);
            vcall(m, fx, 0xc, 0x2c4, &[0], &[]);
        }
        m.call(0x0018_3b80, &[o, 3], &[]);
        let msg = stack_push(m, 8);
        m.w32(msg, p2);
        m.w32(msg + 4, p2 + 0xc);
        vcall(m, o, 0xc, 0x1c, &[0xb, 0, 2, msg, 0], &[]);
        stack_pop(m, 8);
    }
    1
}
