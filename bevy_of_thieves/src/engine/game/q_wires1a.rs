//! WIRE-S1: 1:1 QMem queue ports of live-frontier natives (shard1/2).

use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem};

/// FUN_00113ad0: returns const 0x2babd8 (class descriptor).
pub fn ee_sub_113ad0<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_abd8
}

/// FUN_00113c10: returns const 0x2bad68 (class descriptor).
pub fn ee_sub_113c10<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_ad68
}

/// FUN_00113c50: returns const 0x2badb8 (class descriptor).
pub fn ee_sub_113c50<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_adb8
}

/// FUN_00113d00: returns const 0x2bae94 (class descriptor).
pub fn ee_sub_113d00<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_ae94
}

/// FUN_00114030: returns const 0x2bb290 (class descriptor).
pub fn ee_sub_114030<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_b290
}

/// FUN_001141a0: returns const 0x2bb45c (class descriptor).
pub fn ee_sub_1141a0<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_b45c
}

/// FUN_00114290: returns const 0x2bb588 (class descriptor).
pub fn ee_sub_114290<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_b588
}

/// FUN_00114340: returns const 0x2bb664 (class descriptor).
pub fn ee_sub_114340<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_b664
}

/// FUN_00114400: returns const 0x2bb754 (class descriptor).
pub fn ee_sub_114400<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_b754
}

/// FUN_00114450: returns const 0x2bb7b8 (class descriptor).
pub fn ee_sub_114450<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_b7b8
}

/// FUN_00114550: returns const 0x2bb8f8 (class descriptor).
pub fn ee_sub_114550<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_b8f8
}

/// FUN_00114610: returns const 0x2bb9e8 (class descriptor).
pub fn ee_sub_114610<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_b9e8
}

/// FUN_00117f70(s, out): BinaryInputStream read 16 bits -> FUN_00117968(s, 0x10, out).
pub fn ee_sub_117f70<M: QMem + ?Sized>(m: &mut M, s: u32, out: u32) -> u32 {
    m.call(0x0011_7968, &[s, 0x10, out], &[]).0
}

/// FUN_00232250(x): 989snd command 0xb with a 4-byte {x} payload via FUN_00232838.
pub fn ee_sub_232250<M: QMem + ?Sized>(m: &mut M, x: u32) -> u32 {
    let sp = stack_push(m, 0x10);
    m.w32(sp, x);
    let r = m.call(0x0023_2838, &[0xb, 4, sp, 0, 0], &[]).0;
    stack_pop(m, 0x10);
    r
}

/// FUN_00218748(mv, i): movie entry i (0x18 stride at +8) flag +0x10 = 1, then FUN_001fb018(+0x14) if set.
pub fn ee_sub_218748<M: QMem + ?Sized>(m: &mut M, mv: u32, i: u32) {
    let off = i.wrapping_mul(0x18);
    let t = m.r32(mv + 8);
    m.w32(off.wrapping_add(t) + 0x10, 1);
    let t = m.r32(mv + 8);
    let h = m.r32(off.wrapping_add(t) + 0x14);
    if h != 0 {
        m.call(0x001f_b018, &[h], &[]);
    }
}

/// FUN_002166b0(p, rc, a2): RichTextDraw: copy rect rc, save +0x7c/+0x80, scale by +0x84/+0x88,
/// layout via FUN_002162d8(p, buf, rect) then draw FUN_00216750(p, buf, a2).
pub fn ee_sub_2166b0<M: QMem + ?Sized>(m: &mut M, p: u32, rc: u32, a2: u32) {
    let sp = stack_push(m, 0x60);
    let buf = sp + 0x20;
    for k in (0..0x20).step_by(4) {
        let w = m.r32(rc + k);
        m.w32(sp + k, w);
    }
    let f1 = m.rf(sp + 0xc);
    let f3 = m.rf(p + 0x84) * f1;
    let f2 = m.rf(sp);
    let f4 = m.rf(p + 0x88);
    let f0 = m.rf(sp + 0x14);
    m.wf(p + 0x80, f1);
    let f5 = f2 * f4;
    let f0 = f0 + f3;
    m.wf(p + 0x7c, f2);
    let f1 = f1 * f4;
    m.wf(sp, f5);
    m.wf(sp + 0x14, f0);
    m.wf(sp + 0xc, f1);
    m.call(0x0021_62d8, &[p, buf, sp], &[]);
    m.call(0x0021_6750, &[p, buf, a2], &[]);
    stack_pop(m, 0x60);
}

/// FUN_001eb098(s): read world entry sections: count (FUN_00117c20) -> 0x3dcff0, alloc n*0x28 ->
/// 0x3dcff4, read each (FUN_001eaf50), 0x3dcf98 = max(count of kind 1, n - that).
pub fn ee_sub_1eb098<M: QMem + ?Sized>(m: &mut M, s: u32) {
    const N: u32 = 0x003d_cff0;
    const T: u32 = 0x003d_cff4;
    const C: u32 = 0x003d_cf98;
    let n = m.call(0x0011_7c20, &[s], &[]).0;
    let sp = stack_push(m, 0x10);
    m.w32(sp + 4, 0x10);
    m.w32(N, n);
    m.w32(sp, n.wrapping_mul(0x28));
    let t = m.call(0x001a_e378, &[sp], &[]).0;
    stack_pop(m, 0x10);
    m.w32(T, t);
    m.w32(C, 0);
    if (m.r32(N) as i32) > 0 {
        let mut i: i32 = 0;
        let mut off: u32 = 0;
        loop {
            let a = m.r32(T).wrapping_add(off);
            m.call(0x001e_af50, &[s, a], &[]);
            let e = m.r32(T).wrapping_add(off);
            if m.r8(e + 4) == 1 {
                let c = m.r32(C).wrapping_add(1);
                m.w32(C, c);
            }
            i += 1;
            off = off.wrapping_add(0x28);
            if i >= m.r32(N) as i32 {
                break;
            }
        }
    }
    let c = m.r32(C);
    let r = m.r32(N).wrapping_sub(c);
    if (c as i32) < (r as i32) {
        m.w32(C, r);
    }
}
