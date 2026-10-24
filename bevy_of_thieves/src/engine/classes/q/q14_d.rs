//! WP-Q14 part D: class C2dc018 (world / level object, slot 7 = per-frame update) package: FUN_00157ae0
//! (draw-record bucket sort), FUN_001a3ee8 (light rebuild), FUN_001cd9e0, slot 7 FUN_001cda80, FUN_001dac58
//! (debug label overlay). Slot 8 FUN_00157fa0 (GS/DMA packet-chain flush of the sorted records) is not ported:
//! the PS2 display-list submission is replaced by the Bevy renderer (see port_tracking). 1:1 from
//! ghidra_functions(_v2).body on `QMem`.
#![allow(unused_variables, unused_mut, dead_code)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem};
use crate::engine::classes::q::q6::{ro32, vcall, WORLD};

/// FUN_00157ae0(n, recs, out, start[26], count[26]) -> count[25]: counting sort of `n` 16-byte records by the
/// bucket id in their first word into the pointer array `out`; then qsort (FUN_002338c0) of buckets
/// 5, 6, 10, 1, 2, 0x19 (cmp 0x157a50) and 0x15 (cmp 0x157a98)
pub fn ee_sub_157ae0<M: QMem + ?Sized>(m: &mut M, n: i32, recs: u32, out: u32, start: u32, count: u32) -> u32 {
    for i in 0..=0x19u32 {
        m.w32(count + 4 * i, 0);
    }
    let mut p = recs;
    for _ in 0..n.max(0) {
        let k = m.r32(p);
        let c = m.r32(count + 4 * k).wrapping_add(1);
        m.w32(count + 4 * k, c);
        p += 16;
    }
    m.w32(start, 0);
    let tmp = stack_push(m, 0x70);
    m.w32(tmp, 0);
    for k in 0..0x19u32 {
        let c = m.r32(count + 4 * k);
        m.w32(start + 4 * (k + 1), c);
        let nx = m.r32(count + 4 * (k + 1)).wrapping_add(c);
        m.w32(count + 4 * (k + 1), nx);
        m.w32(tmp + 4 * (k + 1), c);
    }
    let mut p = recs;
    for _ in 0..n.max(0) {
        let k = m.r32(p);
        let i = m.r32(tmp + 4 * k);
        m.w32(out.wrapping_add(i.wrapping_mul(4)), p);
        m.w32(tmp + 4 * k, i + 1);
        p += 16;
    }
    for (b, cmp) in [(5u32, 0x157a50u32), (6, 0x157a50), (10, 0x157a50), (1, 0x157a50), (2, 0x157a50), (0x19, 0x157a50), (0x15, 0x157a98)] {
        let s = m.r32(start + 4 * b);
        let c = m.r32(count + 4 * b);
        m.call(0x0023_38c0, &[out.wrapping_add(s.wrapping_mul(4)), c.wrapping_sub(s), 4, cmp], &[]);
    }
    stack_pop(m, 0x70);
    m.r32(count + 4 * 0x19)
}
/// FUN_001a3ee8(world): when +0x4554 is set: clear it, clear +0x4570 (0x20) and the light table
/// (+0x4564, +0x4560 << 5 bytes) and re-create every light of the 0x100 slots at +0x4154 (FUN_001a3d00)
pub fn ee_sub_1a3ee8<M: QMem + ?Sized>(m: &mut M, w: u32) {
    if m.r32(w + 0x4554) == 0 {
        return;
    }
    m.w32(w + 0x4554, 0);
    m.call(0x0023_33b4, &[w + 0x4570, 0, 0x20], &[]);
    let (t, n) = (m.r32(w + 0x4564), m.r32(w + 0x4560) << 5);
    m.call(0x0023_33b4, &[t, 0, n], &[]);
    for i in 0..0x100u32 {
        let e = m.r32(w + 0x4154 + 4 * i);
        if e != 0 {
            m.call(0x001a_3d00, &[w, e], &[]);
        }
    }
}
/// FUN_001cd9e0(world): every object FUN_0015e9c8(w, 0x105, 0x30, 0, 0x400, buf) finds with flag 8 whose +0xcc
/// differs from world+0x3d5c -> FUN_001681f8(o + 0x80)
pub fn ee_sub_1cd9e0<M: QMem + ?Sized>(m: &mut M, w: u32) {
    let buf = stack_push(m, 0x1000);
    let n = m.call(0x0015_e9c8, &[w, 0x105, 0x30, 0, 0x400, buf], &[]).0 as i32;
    for i in 0..n.max(0) as u32 {
        let o = m.r32(buf + 4 * i);
        let mut f = m.r32(o + 0x40);
        if (f ^ 1) & 1 != 0 {
            f = m.r32(f);
        }
        let cur = m.r32(w + 0x3d5c);
        if f & 8 != 0 && m.r32(o + 0xcc) != cur {
            m.call(0x0016_81f8, &[o + 0x80], &[]);
        }
    }
    stack_pop(m, 0x1000);
}
/// FUN_001cda80 (slot 7): per-frame world update
pub fn c2dc018_v7<M: QMem + ?Sized>(m: &mut M, w: u32) {
    ee_sub_1a3ee8(m, w);
    let c = m.r32(w + 0x3d60).wrapping_add(1);
    m.w32(w + 0x3d60, c);
    m.w32(0x003d_4a50, 0x003b_3980);
    if m.r32(w + 0x3d5c) != m.r32(w + 0x3d64) {
        let v = m.r32(w + 0x3d5c);
        m.w32(w + 0x3d64, v);
        ee_sub_1cd9e0(m, w);
    }
    let it = stack_push(m, 0x40);
    m.w32(w + 0x3f1c, it);
    m.w32(it, 0);
    while (m.r32(it) as i32) < m.r32(w + 0x3f14) as i32 {
        let i = m.r32(it);
        let p = m.r32(w + 0x3f20).wrapping_add(i.wrapping_mul(8));
        if m.r32(p) & 0x88 == 8 {
            let u = m.r32(p + 4);
            vcall(m, u, 0xc, 0x9c, &[], &[]);
        }
        let i = m.r32(it).wrapping_add(1);
        m.w32(it, i);
    }
    m.w32(w + 0x3f1c, 0);
    stack_pop(m, 0x40);
    ee_sub_1dac58(m, 0x003d_a0f0);
    m.call(0x0015_74a0, &[0x002d_ea00], &[]);
    m.call(0x0015_74a0, &[0x002d_ea28], &[]);
}
/// FUN_001dac58(p): debug label ("2L" strings at 0x2c3268) for each entry (stride 0x20 at +0x50, count +0x4c)
/// whose id matches one of the 3 records of the player object (FUN_001c4840) +0x1180
pub fn ee_sub_1dac58<M: QMem + ?Sized>(m: &mut M, p: u32) {
    let a24 = m.r32(p + 0x24);
    if a24 == 0 {
        return;
    }
    // asm 0x1dac8c..98: a0 = *(p + 0x24) stays live into FUN_002132d0
    if m.call(0x0021_32d0, &[a24], &[]).0 == 0 {
        return;
    }
    let pl = m.call(0x001c_4840, &[WORLD], &[]).0;
    if pl == 0 || m.call(0x0013_2b10, &[pl, 0x34], &[]).0 == 0 || (m.r32(p + 0x4c) as i32) < 1 {
        return;
    }
    let sp = stack_push(m, 0x100);
    let (s180, a160, a150) = (sp, sp + 0x20, sp + 0x30);
    let mut i9 = 0u32;
    loop {
        let base = m.r32(p + 0x50);
        let e = base.wrapping_add(i9 * 0x20);
        i9 += 1;
        for k in 0..3u32 {
            if m.r32(e) == m.r32(pl + 0x1180 + 12 * k) {
                // VFIX: FUN_00258438/48 take (p = a0, $f12, $f13)
                m.call(0x0025_8438, &[s180], &[[0.0; 4], [0.0; 4]]);
                m.call(0x0025_8448, &[s180], &[[f32::from_bits(0x4420_0000), 0.0, 0.0, 0.0], [f32::from_bits(0x43f6_6667), 0.0, 0.0, 0.0]]);
                m.w32(a160, 0xff80_8080);
                m.call(0x0025_8458, &[s180, a160], &[]);
                m.call(0x0025_8470, &[s180, 0], &[]);
                m.call(0x0025_8478, &[s180, 0], &[]);
                m.call(0x0021_5ac0, &[a150, ro32(0x002c_3268 + 4 * k)], &[super::q5util::f4(f32::from_bits(ro32(0x002c_3264)))]);
                let (x, y) = (m.r32(e + 0x18), m.r32(e + 0x14));
                m.w32(s180 + 4, x);
                m.w32(s180, y);
                m.w32(s180 + 8, 0x4200_0000);
                m.w32(s180 + 12, 0x4200_0000);
                m.call(0x0021_63b0, &[a150, s180, 0x13], &[]);
                break;
            }
        }
        if (m.r32(p + 0x4c) as i32) <= i9 as i32 {
            break;
        }
    }
    stack_pop(m, 0x100);
}
