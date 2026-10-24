//! WIRE-S3: 1:1 QMem ports of live-frontier natives (shard 5/6/7).
#![allow(unused)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

// SKIP 0x177f78: VU0 macro heavy (vmul/vsqrt/vdiv/ACC chains over many 128-bit temporaries) plus a
// 5-entry jump table at 0x2a5530; not portable faithfully against QMem without VU0 emulation.

/// FUN_001fda48: lazy-init +0x6c (FUN_002330d0), then FUN_00156be0(s2, s4, player, &+0x74)
/// with s2 = world+0x4034 target updated from quadratic in +0x70 when > 0.5.
pub fn ee_sub_1fda48<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let s0 = a0;
    if m.r32(s0 + 0x10) == 0 {
        m.w32(s0 + 0x70, 0);
    } else if m.r32(s0 + 0x6c) == 0 {
        m.w32(s0 + 0x6c, 1);
        // 9th arg (sd s0, 0(sp)) passed on the stack.
        m.call(0x0023_30d0, &[0x524b_5553, 0xb, 0, 0, 0, 0, 0, 0x001f_cf70, s0], &[]);
    }
    let mut s2: u32 = 0;
    let mut s4: [f32; 4] = [0.0; 4]; // qword (por a1, s4); uninitialised callee-saved in asm on some paths
    let s3: u32 = 0x003d_d490;
    let s1 = m.call(0x001c_4840, &[s3], &[]).0;
    if s1 != 0 && m.call(0x0013_2b10, &[s1, 0x34], &[]).0 != 0 {
        if 0.5f32 < m.rf(s0 + 0x70) {
            s2 = m.r32(s3 + 0x4034);
            s4 = m.call(0x001a_62e8, &[s1], &[]).1;
            let t = m.rf(s0 + 0x70);
            let x = m.rf(s0 + 0x80) + t * (m.rf(s0 + 0x84) + t * m.rf(s0 + 0x88));
            m.wf(s2 + 0x54, x);
            let t = m.rf(s0 + 0x70);
            let y = m.rf(s0 + 0x90) + t * (m.rf(s0 + 0x94) + t * m.rf(s0 + 0x98));
            m.wf(s2 + 0x5c, y);
            m.wf(s2 + 0x58, -y);
        }
    }
    m.call(0x0015_6be0, &[s2, s1, s0 + 0x74], &[s4]);
}

/// FUN_0012b258: (a0 != 0, a1 == 0xffff) -> ref-count six pools at 0x2dd2e0.., first ref inits via FUN_0011cca8.
pub fn ee_sub_12b258<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if a1 != 0xffff || a0 == 0 {
        return;
    }
    const P: [(u32, u32, u32, u32); 6] = [
        (0x002d_d2e0, 0x0045_20a8, 0x14, 0x2000),
        (0x002d_d2e4, 0x0045_20d0, 0x18, 0x800),
        (0x002d_d2e8, 0x0045_20f8, 0x28, 0x200),
        (0x002d_d2ec, 0x0045_2120, 0x1c, 0x800),
        (0x002d_d2f0, 0x0045_2148, 0xc, 0xc8),
        (0x002d_d2f4, 0x0045_2170, 0x14, 0x100),
    ];
    for &(cnt, pool, sz, n) in P.iter() {
        let v = m.r32(cnt).wrapping_add(1);
        m.w32(cnt, v);
        if v == 1 {
            m.call(0x0011_cca8, &[pool, sz, n, 0x10], &[]);
        }
    }
}

/// FUN_001531a8(o, a1, n, src): +0x60 = n; n -> copy n*0x30 keys to +0x70 and normalise; FUN_00153128(o, a1).
pub fn ee_sub_1531a8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32) {
    let s1 = a0;
    m.w32(s1 + 0x60, a2);
    if a2 != 0 {
        let s2 = s1 + 0x70;
        m.call(0x001a_e5a0, &[s2, a3, a2.wrapping_mul(0x30)], &[]);
        let last = m.r32(s1 + 0x60).wrapping_sub(1);
        let f4v = m.rf(s1 + 0x70);
        let f0 = m.rf(last.wrapping_mul(0x30).wrapping_add(s1).wrapping_add(0x70));
        m.wf(s1 + 0x1a0, 100.0);
        m.w32(s1 + 0x1a4, 0x43ea_6667);
        m.w32(s1 + 0x1a8, 0);
        let f2 = f32::from_bits(0x43dc_0000) / (f0 - f4v);
        let mut i: i32 = 0;
        let mut p = s2;
        while i < m.r32(s1 + 0x60) as i32 {
            let f1 = m.rf(p);
            i += 1;
            let q = m.rv(s1 + 0x1a0);
            m.w32(p + 0x20, 0);
            m.wv(p + 0x10, q);
            let v = m.rf(p + 0x10) + f2 * (f1 - f4v);
            m.wf(p + 0x10, v);
            p = p.wrapping_add(0x30);
        }
        m.wf(s1 + 0x1a0, 10.0);
        m.w32(s1 + 0x1a4, 0x43ea_6667);
        m.w32(s1 + 0x1a8, 0);
        m.w32(s1 + 0x190, 0);
        m.w32(s1 + 0x194, 0);
    }
    m.call(0x0015_3128, &[s1, a1], &[]);
}

/// FUN_0022b090(o, st): state change; 0 clear (+vslot 49 on +0x54), 1/2 vslot 48 msg 0x311/0x30f, 3 FUN_0020df80 + clear.
pub fn ee_sub_22b090<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let s1 = a0;
    if m.r32(s1) == a1 {
        return;
    }
    m.w32(s1, a1);
    match a1 {
        0 => {
            m.w32(s1 + 4, 0);
            m.call(0x0023_33b4, &[s1 + 8, 0, 0x4c], &[]);
            let s0 = m.r32(s1 + 0x54);
            if s0 != 0 {
                vslot(m, s0, 49, &[], &[]);
            }
        }
        1 | 2 => {
            let k = if a1 == 1 { 0x311 } else { 0x30f };
            let o = m.r32(s1 + 0x54);
            vslot(m, o, 48, &[k], &[]);
        }
        3 => {
            let s0 = s1 + 8;
            m.call(0x0020_df80, &[0x003d_d490, s0], &[]);
            m.w32(s1 + 4, 0);
            m.call(0x0023_33b4, &[s0, 0, 0x4c], &[]);
        }
        _ => {}
    }
}

/// FUN_001f3b28(o, on): set/clear flag 8 in +0x40 (direct, or via pointer when even); on -> maybe FUN_001d6378, off -> FUN_001d6310.
pub fn ee_sub_1f3b28<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let s0 = a0;
    let a2 = m.r32(s0 + 0x40);
    let ind = a2 & 1 == 0;
    let v1 = if ind { m.r32(a2) } else { a2 };
    if a1 != 0 {
        if v1 & 8 != 0 {
            return;
        }
        if ind {
            let v = m.r32(a2) | 8;
            m.w32(a2, v);
        } else {
            m.w32(s0 + 0x40, a2 | 8);
        }
        if m.r32(s0 + 0x14c) & 8 == 0 {
            return;
        }
        if m.call(0x001a_61a0, &[s0], &[]).0 == 0 {
            return;
        }
        m.call(0x001d_6378, &[s0], &[]);
    } else {
        if v1 & 8 == 0 {
            return;
        }
        if ind {
            let v = m.r32(a2) & !8;
            m.w32(a2, v);
        } else {
            m.w32(s0 + 0x40, a2 & !8);
        }
        m.call(0x001d_6310, &[s0], &[]);
    }
}

/// FUN_00152578: for each of +0x2c4 entries (stride 0x28 at +0x2cc), 3 slots: id != -1 -> +4 = FUN_00152520(o, id).
pub fn ee_sub_152578<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let s2 = a0;
    let mut i: i32 = 0;
    while i < m.r32(s2 + 0x2c4) as i32 {
        let mut s0 = (i as u32).wrapping_mul(0x28).wrapping_add(s2).wrapping_add(0x2d0);
        for _ in 0..3 {
            let a1 = m.r32(s0 - 4);
            if a1 != 0xffff_ffff {
                let v = m.call(0x0015_2520, &[s2, a1], &[]).0;
                m.w32(s0, v);
            }
            s0 = s0.wrapping_add(0xc);
        }
        i += 1;
    }
}

/// FUN_00136d28(o, a1, q, f13): init header then 6x FUN_00137a30(+0, 0xc, 2, &+0x38+i*0x1c, 0, 0, 0, &+0x50+i*0x1c).
pub fn ee_sub_136d28<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: V4, f13: f32) {
    let s0 = a0;
    m.w32(s0, a1);
    m.wv(s0 + 0x20, a2);
    m.wf(s0 + 0x30, f13);
    m.w32(s0 + 4, 0xffff_ffff);
    let mut s1 = s0 + 0x38;
    let mut s2 = s0 + 0x50;
    for _ in 0..6 {
        let k = m.r32(s0);
        m.call(0x0013_7a30, &[k, 0xc, 2, s1, 0, 0, 0, s2], &[]);
        s2 = s2.wrapping_add(0x1c);
        s1 = s1.wrapping_add(0x1c);
    }
}

/// FUN_001510d8: +0x18 < 0 -> 1; else FUN_0011d4d0(+0xac, +0x18, 0.01) != 0.
pub fn ee_sub_1510d8<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let f13 = m.rf(a0 + 0x18);
    if f13 < 0.0 {
        return 1;
    }
    let f12 = m.rf(a0 + 0xac);
    let f14 = f32::from_bits(0x3c23_d70a);
    if m.call(0x0011_d4d0, &[], &[f4(f12), f4(f13), f4(f14)]).0 == 0 {
        0
    } else {
        1
    }
}

/// FUN_00171d58: o = FUN_00205458(a0, 0x9c); o -> FUN_00176750(o, &{1, 0...}[0x70]).
pub fn ee_sub_171d58<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let s0 = m.call(0x0020_5458, &[a0, 0x9c], &[]).0;
    if s0 != 0 {
        let sp = stack_push(m, 0x70);
        m.call(0x0023_33b4, &[sp, 0, 0x70], &[]);
        m.w32(sp, 1);
        m.call(0x0017_6750, &[s0, sp], &[]);
        stack_pop(m, 0x70);
    }
}

/// FUN_0011a3c0: length of intrusive list (head +0, next offset +8).
pub fn ee_sub_11a3c0<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let mut v = m.r32(a0);
    let mut n = 0u32;
    if v != 0 {
        let off = m.r32(a0 + 8);
        loop {
            v = m.r32(v.wrapping_add(off));
            n += 1;
            if v == 0 {
                break;
            }
        }
    }
    n
}

/// FUN_001813f0: +0x384 = f12 * DAT_002be87c.
pub fn ee_sub_1813f0<M: QMem + ?Sized>(m: &mut M, a0: u32, f12: f32) {
    let k = m.rf(0x002b_e87c);
    m.wf(a0 + 0x384, f12 * k);
}

/// FUN_00113a50: returns 0x2bab38.
pub fn ee_sub_113a50<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_ab38
}

/// FUN_00113be0: returns 0x2bad2c.
pub fn ee_sub_113be0<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_ad2c
}

/// FUN_00113e60: returns 0x2bb04c.
pub fn ee_sub_113e60<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_b04c
}

/// FUN_00114100: returns 0x2bb394.
pub fn ee_sub_114100<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_b394
}

/// FUN_00114250: returns 0x2bb538.
pub fn ee_sub_114250<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_b538
}

/// FUN_00114660: returns 0x2bba4c.
pub fn ee_sub_114660<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_ba4c
}
