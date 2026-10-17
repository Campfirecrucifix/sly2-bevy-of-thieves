//! WP-Q12 part D: file package run_001da2e0 (C2d85e8 / C2d8568 / C2d84e8 selector family, "pick a card" style
//! menu objects). 1:1 from ghidra_functions(_v2).body on `QMem` (conventions as q12_a); the dropped call
//! arguments of FUN_002132d0 (+0x28), FUN_00213928 (+0x24, old id) and FUN_00213b00 (+0x24, id) come from the
//! disassembly (a0 / a1 untouched registers).
#![allow(unused_variables, unused_mut, dead_code)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem};
use crate::engine::classes::q::q6::rof;

const NEG1: u32 = u32::MAX;
const WORLD: u32 = 0x003d_d490;
const CLOCK2: u32 = 0x002d_df0c;

/// FUN_001da2e0: +0x28 = FUN_0015e938(world, 0x405, 0x371, 0)
pub fn ee_sub_1da2e0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let u = m.call(0x0015_e938, &[WORLD, 0x405, 0x371, 0], &[]).0;
    m.w32(o + 0x28, u);
}
/// FUN_001da320: +0x20 = +0x18 = -1, +0x28 = 0, FUN_001da5c8(o, 0)
pub fn ee_sub_1da320<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.w32(o + 0x20, NEG1);
    m.w32(o + 0x18, NEG1);
    m.w32(o + 0x28, 0);
    m.call(0x001d_a5c8, &[o, 0], &[]);
}
/// FUN_001da350: per-frame state machine on +0x10 (1: wait +0x28 state 2 -> 2; 2: +0x20 set -> 4, timer +0x1c == 0
/// -> 3, > 0 and FUN_002132d0(+0x28) == 0 -> +0x14 += dt; 3: +0x28 state < 2 -> 0; 4: job 0x312 done -> 0 / 2)
pub fn ee_sub_1da350<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.r32(o + 0x28) == 0 {
        return;
    }
    match m.r32(o + 0x10) {
        1 => {
            let j = m.r32(o + 0x28);
            if m.r32(j + 0x58) != 2 {
                return;
            }
            m.call(0x001d_a5c8, &[o, 2], &[]);
        }
        2 => {
            if m.r32(o + 0x20) != NEG1 {
                m.call(0x001d_a5c8, &[o, 4], &[]);
                return;
            }
            let t = m.rf(o + 0x1c);
            if t == 0.0 {
                m.call(0x001d_a5c8, &[o, 3], &[]);
                return;
            }
            if t <= 0.0 {
                return;
            }
            let j = m.r32(o + 0x28);
            if j == 0 {
                return;
            }
            if m.call(0x0021_32d0, &[j], &[]).0 & 0xff != 0 {
                return;
            }
            let v = m.rf(o + 0x14) + rof(0x002d_ded4);
            m.wf(o + 0x14, v);
        }
        3 => {
            let j = m.r32(o + 0x28);
            if (m.r32(j + 0x58) as i32) < 2 {
                m.call(0x001d_a5c8, &[o, 0], &[]);
            }
        }
        4 => {
            let j = m.r32(o + 0x28);
            if m.call(0x0021_21e8, &[j, 0x312], &[]).0 != 0 {
                return;
            }
            if m.r32(o + 0x20) == NEG1 {
                m.call(0x001d_a5c8, &[o, 0], &[]);
            } else {
                m.call(0x001d_a5c8, &[o, 2], &[]);
            }
        }
        _ => {}
    }
}
/// FUN_001da5b0: +0x20, or +0x18 when +0x20 == -1
pub fn ee_sub_1da5b0<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let v = m.r32(o + 0x20);
    if v == NEG1 {
        return m.r32(o + 0x18);
    }
    v
}
/// FUN_001da728 (C2d85e8_ctor): [0] = [1] = 0, vtable +0xc = 0x2d85e8, [8] = -1, FUN_001da7d0(this)
pub fn ee_sub_1da728<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    m.w32(o, 0);
    m.w32(o + 4, 0);
    m.w32(o + 0xc, 0x002d_85e8);
    m.w32(o + 0x20, NEG1);
    ee_sub_1da7d0(m, o);
    o
}
/// FUN_001da7d0: +0x28 = -1, +0x2c = DAT_002ddf0c, FUN_001da950(o, -1, -1)
pub fn ee_sub_1da7d0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.w32(o + 0x28, NEG1);
    let c = m.r32(CLOCK2);
    m.w32(o + 0x2c, c);
    m.call(0x001d_a950, &[o, NEG1, NEG1], &[]);
}
/// FUN_001da808 (C2d83e8_v6): C2d86d0_v6(this); +0x34 <= DAT_002ddf0c -> FUN_001da950(o, -1, -1)
pub fn ee_sub_1da808<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001d_9870, &[o], &[]);
    if m.rf(o + 0x34) <= m.rf(CLOCK2) {
        m.call(0x001d_a950, &[o, NEG1, NEG1], &[]);
    }
}
/// FUN_001daa08 (C2d8568_ctor)
pub fn ee_sub_1daa08<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    ee_sub_1da728(m, o);
    m.w32(o + 0xc, 0x002d_8568);
    ee_sub_1daa78(m, o);
    o
}
/// FUN_001daa78: +0x40 = -1, +0x44 = DAT_002ddf0c
pub fn ee_sub_1daa78<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.w32(o + 0x40, NEG1);
    let c = m.r32(CLOCK2);
    m.w32(o + 0x44, c);
}
/// FUN_001dae20(o, id): entry (0x20 bytes, [0] == id) of the +0x50 table (+0x4c entries), or 0
pub fn ee_sub_1dae20<M: QMem + ?Sized>(m: &mut M, o: u32, id: u32) -> u32 {
    let n = m.r32(o + 0x4c) as i32;
    let mut i = 0;
    if 0 < n {
        let mut p = m.r32(o + 0x50);
        loop {
            i += 1;
            if m.r32(p) == id {
                return p;
            }
            p += 0x20;
            if !(i < m.r32(o + 0x4c) as i32) {
                break;
            }
        }
    }
    0
}
/// FUN_001daf38(a, b): Splice call FUN_001d8b20 with a formatted string (sprintf(0x2a79a0, a)), {0x17, b != 0},
/// FUN_00129658 result store; when b != 0: DAT_002deb3c = a, FollowWorld_activity(0x2deb30, 0x3b, 0)
pub fn ee_sub_1daf38<M: QMem + ?Sized>(m: &mut M, a: u32, b: u32) {
    let base = stack_push(m, 0xb0);
    let top = base + 0xb0;
    let l = |x: u32| top - x;
    m.w32(l(0xa0), NEG1);
    m.call(0x0021_ecd8, &[l(0x90), 0x002a_79a0, a], &[]);
    m.w32(l(0x60), 0x17);
    m.w32(l(0x5c), (b != 0) as u32);
    m.call(0x001d_8b20, &[l(0x70), l(0x60), 0x002a_79a8, 0x002a_79b8, l(0x90), 0], &[]);
    m.call(0x0012_9658, &[l(0xa0), l(0x70)], &[]);
    m.call(0x0012_9ec8, &[l(0x70)], &[]);
    m.call(0x0012_9ec8, &[l(0x60)], &[]);
    if b != 0 {
        m.w32(0x002d_eb3c, a);
        m.call(0x0016_19a0, &[0x002d_eb30, 0x3b, 0], &[]);
        // PAD1: record the gadget-purchase save-variable write for host drain.
        // The original FUN_001daf38 writes ("powerUps", "mppukfAcquired", a) = Bool(true) via
        // FUN_001d8b20, but the save-variable chain (0x1d8b20 → 0x1d80c8) is NATIVE in QMem.
        // Recording the purchase lets the host write the save var on the next drain.
        m.record_gadget_purchase(a, true);
    }
    m.call(0x0012_9ec8, &[l(0xa0)], &[]);
    stack_pop(m, 0xb0);
}
/// FUN_001db010: per entry flags +0x1c: bit 1 = FUN_001dae58(id) & 1, bit 2 = bif 138(id) & 1
pub fn ee_sub_1db010<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let mut i = 0;
    if 0 < m.r32(o + 0x4c) as i32 {
        let mut base = m.r32(o + 0x50);
        loop {
            let e = base + i * 0x20;
            i += 1;
            let id = m.r32(e);
            let u = m.call(0x001d_ae58, &[id], &[]).0;
            let w = m.r32(e + 0x1c) & 0xffff_fffd | (u & 1) << 1;
            m.w32(e + 0x1c, w);
            let id = m.r32(e);
            let u = m.call(0x001d_aea8, &[id], &[]).0;
            let w = m.r32(e + 0x1c) & 0xffff_fffb | (u & 1) << 2;
            m.w32(e + 0x1c, w);
            if (m.r32(o + 0x4c) as i32) <= i as i32 {
                break;
            }
            base = m.r32(o + 0x50);
        }
    }
}
/// FUN_001db390(o, sel): +0x48 = sel; per entry the wanted value (sel match -> [3]; flag 4 -> [2]; flag 2 -> [1];
/// else -1) replaces [4] (the old non-(-1) one is released through FUN_00213928(+0x24, old), the new
/// registered with FUN_00213b00(+0x24, v)) when +0x24 is set
pub fn ee_sub_1db390<M: QMem + ?Sized>(m: &mut M, o: u32, sel: u32) {
    m.w32(o + 0x48, sel);
    let mut i: i32 = 0;
    if 0 < m.r32(o + 0x4c) as i32 {
        let mut base = m.r32(o + 0x50);
        loop {
            let e = base.wrapping_add((i as u32) * 0x20);
            let mut v;
            if m.r32(e) == sel {
                v = m.r32(e + 0xc);
            } else if m.r32(e + 0x1c) & 4 == 0 {
                v = NEG1;
                if m.r32(e + 0x1c) & 2 != 0 {
                    v = m.r32(e + 4);
                }
            } else {
                v = m.r32(e + 8);
            }
            let lim;
            if v == m.r32(e + 0x10) {
                lim = m.r32(o + 0x4c) as i32;
            } else {
                let c = m.r32(o + 0x24);
                if c == 0 {
                    m.w32(e + 0x10, v);
                } else if m.r32(e + 0x10) == NEG1 {
                    m.w32(e + 0x10, v);
                } else {
                    let old = m.r32(e + 0x10);
                    m.call(0x0021_3928, &[c, old], &[]);
                    m.w32(e + 0x10, v);
                }
                let c = m.r32(o + 0x24);
                if c != 0 && v != NEG1 {
                    m.call(0x0021_3b00, &[c, v], &[]);
                }
                lim = m.r32(o + 0x4c) as i32;
            }
            i += 1;
            if lim <= i {
                break;
            }
            base = m.r32(o + 0x50);
        }
    }
}
/// FUN_001db490: FUN_001c4840(world) == DAT_002e1e40 / DAT_002dd5bc / DAT_002f7900 -> FUN_001db520(o, 0 / 1 / 2)
pub fn ee_sub_1db490<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let w = m.call(0x001c_4840, &[WORLD], &[]).0;
    if w != 0 {
        if w == m.r32(0x002e_1e40) {
            ee_sub_1db520(m, o, 0);
        } else if w == m.r32(0x002d_d5bc) {
            ee_sub_1db520(m, o, 1);
        } else if w == m.r32(0x002f_7900) {
            ee_sub_1db520(m, o, 2);
        }
    }
}
/// FUN_001db520(o, k): menu page change (+0x40): leaving -1 -> FUN_0014d1d0(0, 0x2dded0, 0), FUN_0020efc0(world, 0),
/// slot 9; pages 0 / 1 / 2 pick id 0x375 / 0x376 / 0x377 and the entry tables 0x2c2de8 (0x12) / 0x2c3028 (9) /
/// 0x2c3148 (8); page -1 tears down (slot 49 on +0x24, entries' [4] = -1, FUN_0014d1d0(1.0), FUN_0020f030,
/// slot 10); then for a page != -1: FUN_001db010, +0x24 = FUN_0015e938(world, 5, +0x20, 0) (0 -> page -1),
/// slot 48(0x318), the entry with flag 4 and the lowest [5] + [6] (< DAT_002a79cc) selected via FUN_001db390
pub fn ee_sub_1db520<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32) {
    let ki = k as i32;
    if k == m.r32(o + 0x40) {
        return;
    }
    if m.r32(o + 0x40) == NEG1 {
        m.call(0x0014_d1d0, &[0, 0x002d_ded0, 0], &[]);
        m.call(0x0020_efc0, &[WORLD, 0], &[]);
        vslot(m, o, 9, &[], &[]);
    }
    ee_sub_1db390(m, o, NEG1);
    m.w32(o + 0x40, k);
    let c = m.r32(CLOCK2);
    m.w32(o + 0x44, c);
    let set = |m: &mut M, id: u32, tab: u32, n: u32| {
        m.w32(o + 0x20, id);
        m.w32(o + 0x4c, n);
        m.w32(o + 0x50, tab);
    };
    let iv7;
    if k == 0 {
        set(m, 0x375, 0x002c_2de8, 0x12);
        iv7 = m.r32(o + 0x40);
    } else if ki > 0 {
        if k == 1 {
            set(m, 0x376, 0x002c_3028, 9);
        } else if k == 2 {
            set(m, 0x377, 0x002c_3148, 8);
        }
        iv7 = m.r32(o + 0x40);
    } else if k != NEG1 {
        iv7 = m.r32(o + 0x40);
    } else {
        let c = m.r32(o + 0x24);
        let mut n = m.r32(o + 0x4c);
        if c != 0 {
            vslot(m, c, 49, &[], &[]);
            n = m.r32(o + 0x4c);
        }
        let mut i = 0;
        if 0 < n as i32 {
            let mut base = m.r32(o + 0x50);
            loop {
                let e = base + i * 0x20;
                i += 1;
                m.w32(e + 0x10, NEG1);
                if (m.r32(o + 0x4c) as i32) <= i as i32 {
                    break;
                }
                base = m.r32(o + 0x50);
            }
        }
        m.w32(o + 0x4c, 0);
        m.w32(o + 0x50, 0);
        m.call(0x0014_d1d0, &[0x3f80_0000, 0x002d_ded0, 0], &[]);
        m.call(0x0020_f030, &[WORLD, 0], &[]);
        vslot(m, o, 10, &[], &[]);
        m.call(0x001d_a950, &[o, NEG1, NEG1], &[]);
        m.w32(o + 0x24, 0);
        m.w32(o + 0x20, NEG1);
        iv7 = m.r32(o + 0x40);
    }
    if iv7 != NEG1 {
        ee_sub_1db010(m, o);
        let id = m.r32(o + 0x20);
        let l = m.call(0x0015_e938, &[WORLD, 5, id, 0], &[]).0;
        m.w32(o + 0x24, l);
        if l == 0 {
            ee_sub_1db520(m, o, NEG1);
        } else {
            vslot(m, l, 48, &[0x318], &[]);
            let mut n = m.r32(o + 0x4c) as i32;
            let mut u4 = NEG1;
            if 0 < n {
                let mut e = m.r32(o + 0x50);
                let mut best = m.rf(0x002a_79cc);
                loop {
                    n -= 1;
                    if m.r32(e + 0x1c) & 4 != 0 && m.rf(e + 0x14) + m.rf(e + 0x18) < best {
                        u4 = m.r32(e);
                        best = m.rf(e + 0x14) + m.rf(e + 0x18);
                    }
                    e += 0x20;
                    if n == 0 {
                        break;
                    }
                }
            }
            ee_sub_1db390(m, o, u4);
        }
    }
}
/// FUN_001db7a8 (C2d84e8_ctor)
pub fn ee_sub_1db7a8<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    ee_sub_1da728(m, o);
    m.w32(o + 0x20, 0x378);
    m.w32(o + 0xc, 0x002d_84e8);
    ee_sub_1db820(m, o);
    o
}
/// FUN_001db820: +0x40 = -1, +0x44 = clock, +0x264 / +0x25c / +0x260 = -1, 0x40 id pairs at +0x4c / +0x50 (stride 8) = -1
pub fn ee_sub_1db820<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.w32(o + 0x40, NEG1);
    let c = m.r32(CLOCK2);
    m.w32(o + 0x264, NEG1);
    m.w32(o + 0x44, c);
    m.w32(o + 0x25c, NEG1);
    m.w32(o + 0x260, NEG1);
    for i in 0..0x40 {
        m.w32(o + 0x4c + 8 * i, NEG1);
        m.w32(o + 0x50 + 8 * i, NEG1);
    }
}
