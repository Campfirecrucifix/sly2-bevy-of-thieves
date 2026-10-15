//! WP-Q9: C2cc2c0 (follower state machine +0x30 = -1 / 0 / 1 / 2, since-time +0x34, owner +0x10, target +0x38) overrides.
//! C2cc340 (c2cc340.rs) is the same code with other constants / offsets; both share `follow_*` below.
use crate::engine::classes::q::q2mem::QMem;
use crate::engine::classes::q::q2mem::V4;
use crate::engine::classes::q::q9util::*;

/// constants and offsets that differ between C2cc2c0 and C2cc340
pub struct Follow {
    /// state 1: stay while DAT >= len (len limit to go back to 0)
    pub lim1: u32,
    /// state 0: len < DAT -> test facing
    pub lim0: u32,
    /// state 0: len > DAT -> state 1
    pub lim2: u32,
    /// owner field read in state 2 (-1 = ready to leave)
    pub busy: u32,
    pub set_state: u32,
}
pub const FOLLOW_2C0: Follow = Follow { lim1: 0x002b_d358, lim0: 0x002b_d350, lim2: 0x002b_d35c, busy: 0x1714, set_state: 0x146650 };
pub const FOLLOW_340: Follow = Follow { lim1: 0x002b_d348, lim0: 0x002b_d344, lim2: 0x002b_d34c, busy: 0x1810, set_state: 0x146158 };

/// FUN_00146650 / FUN_00146158(this, new): on change: leaving state 2 -> FUN_001ef038(this); +0x34 = clock;
/// entering state 2 -> FUN_001eefc0(this)
pub fn follow_set_state<M: QMem + ?Sized>(m: &mut M, o: u32, new: u32) {
    let cur = m.r32(o + 0x30);
    if new != cur {
        if cur == 2 {
            m.call(0x001e_f038, &[o], &[]);
        }
        m.w32(o + 0x30, new);
        let c = m.r32(CLOCK);
        m.w32(o + 0x34, c);
        if new == 2 {
            m.call(0x001e_efc0, &[o], &[]);
        }
    }
}
/// slot 6: parent FUN_001eef68, +0x30 = -1
pub fn follow_v6<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef68, &[o], &[]);
    m.w32(o + 0x30, u32::MAX);
}
/// slot 7: parent FUN_001eef70, set_state(0), owner virtual +0x2fc (owner, 4, target, 4, target), owner +0x10e0 |= 1
pub fn follow_v7<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef70, &[o], &[]);
    follow_set_state(m, o, 0);
    let ow = m.r32(o + 0x10);
    let t = m.r32(o + 0x38);
    vcall(m, ow, 0xc, 0x2fc, &[4, t, 4, t], &[]);
    let ow = m.r32(o + 0x10);
    let f = m.r32(ow + 0x10e0);
    m.w32(ow + 0x10e0, f | 1);
}
/// slot 8: parent FUN_001eef78, owner +0x10e0 &= ~1
pub fn follow_v8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef78, &[o], &[]);
    let ow = m.r32(o + 0x10);
    let f = m.r32(ow + 0x10e0);
    m.w32(ow + 0x10e0, f & 0xffff_fffe);
}
/// slot 9 (update): parent FUN_001eef98; if the state is not -1 and the owner is free (owner +0xdf4 == -1), the target
/// FUN_001a61a0 != 0 and target +0x330 == 0: e = normalize_xy(pos(target) - pos(owner)) (z cleared, < 1e-4 ->
/// quad DAT_002a2da0), len = |xy|; then iterate the transitions until the state stops changing:
/// 1: len < lim1 -> 0; 0: len < lim0 -> (dot(owner facing FUN_001a6318, e) > 0.8 -> 2), else len > lim2 -> 1;
/// 2: owner +busy == -1 and clock - since > 0.1 -> -1. Otherwise (condition false) state -1.
pub fn follow_v9<M: QMem + ?Sized>(m: &mut M, o: u32, k: &Follow) {
    m.call(0x001e_ef98, &[o], &[]);
    if m.r32(o + 0x30) == u32::MAX {
        return;
    }
    let ow = m.r32(o + 0x10);
    let tg = m.r32(o + 0x38);
    let mut ok = m.r32(ow + 0xdf4) == u32::MAX;
    ok = ok && m.call(0x001a_61a0, &[tg], &[]).0 != 0;
    ok = ok && m.r32(tg + 0x330) == 0;
    if !ok {
        follow_set_state(m, o, u32::MAX);
        return;
    }
    let a = m.call(0x001a_62e8, &[tg], &[]).1;
    let b = m.call(0x001a_62e8, &[ow], &[]).1;
    let mut d: V4 = std::array::from_fn(|i| a[i] - b[i]);
    d[2] = 0.0;
    let (e, len) = vu_normalize(d, rov(0x002a_2da0));
    let lim0 = rof(k.lim0);
    let (lim1, lim2) = (rof(k.lim1), rof(k.lim2));
    let mut st = m.r32(o + 0x30) as i32;
    loop {
        match st {
            1 => {
                if !(lim1 <= len) {
                    follow_set_state(m, o, 0);
                }
            }
            0 => {
                if len < lim0 {
                    let p = m.call(0x001a_6318, &[ow], &[]).0;
                    let f = m.rv(p);
                    if f[0] * e[0] + f[1] * e[1] + f[2] * e[2] > 0.8 {
                        follow_set_state(m, o, 2);
                    }
                } else if lim2 < len {
                    follow_set_state(m, o, 1);
                }
            }
            2 => {
                if m.r32(ow + k.busy) == u32::MAX {
                    let since = m.rf(CLOCK) - m.rf(o + 0x34);
                    if 0.1f32 < since {
                        follow_set_state(m, o, u32::MAX);
                    }
                }
            }
            _ => {}
        }
        let now = m.r32(o + 0x30) as i32;
        if now == st {
            return;
        }
        st = now;
    }
}
/// slot 13: parent FUN_001ef0c0(this, p2); state in 0..=2: d = pos(target) - pos(owner); p2+0x1c = 1.0 (0.1 in state 0);
/// p2+0x18 = atan2(d.y, d.x) (FUN_002458f0); state 2 and clock - since < 0.1: p2+0x30 |= 0x80, FUN_001c45a8(p2)
pub fn follow_v13<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32) {
    m.call(0x001e_f0c0, &[o, p2], &[]);
    let st = m.r32(o + 0x30) as i32;
    if st < 3 && -1 < st {
        let tg = m.r32(o + 0x38);
        let ow = m.r32(o + 0x10);
        let a = m.call(0x001a_62e8, &[tg], &[]).1;
        let b = m.call(0x001a_62e8, &[ow], &[]).1;
        let d: V4 = std::array::from_fn(|i| a[i] - b[i]);
        let mut w = 1.0f32;
        if m.r32(o + 0x30) == 0 {
            w = f32::from_bits(0x3dcc_cccd);
        }
        m.wf(p2 + 0x1c, w);
        // PROT3W: lane x of the rotated quad = d.y
        let r = m.call(0x0024_58f0, &[], &[s(d[1]), s(d[0])]).1[0];
        m.wf(p2 + 0x18, r);
        if m.r32(o + 0x30) == 2 && m.rf(CLOCK) - m.rf(o + 0x34) < 0.1 {
            let f = m.r32(p2 + 0x30);
            m.w32(p2 + 0x30, f | 0x80);
            m.call(0x001c_45a8, &[p2], &[]);
        }
    }
}
pub fn ee_sub_1461c8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    follow_v6(m, o)
}
pub fn ee_sub_1461f8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    follow_v7(m, o)
}
pub fn ee_sub_146260<M: QMem + ?Sized>(m: &mut M, o: u32) {
    follow_v8(m, o)
}
pub fn ee_sub_1462a8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    follow_v9(m, o, &FOLLOW_2C0)
}
pub fn ee_sub_146560<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32) {
    follow_v13(m, o, p2)
}
pub fn ee_sub_146650<M: QMem + ?Sized>(m: &mut M, o: u32, new: u32) {
    follow_set_state(m, o, new)
}
