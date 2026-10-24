//! WIRE-W1H10: 1:1 queue ports (against `QMem`) of a batch of live-frontier natives, ported from the EE asm;
//! callees go through `m.call` / `vslot`. QMem convention: integer / pointer args in `a`, quadword and float
//! args in `v`.
#![allow(unused_variables, unused_mut, unused_assignments, clippy::all)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

const GS: u32 = 0x003d_d490; // game state / world

/// `p = *(o+0x40)`: bit0 clear -> pointer, `*p |= bits`; else `*(o+0x40) = p | bits`.
fn or40<M: QMem + ?Sized>(m: &mut M, o: u32, bits: u32) {
    let v = m.r32(o + 0x40);
    if v & 1 == 0 {
        let x = m.r32(v);
        m.w32(v, x | bits);
    } else {
        m.w32(o + 0x40, v | bits);
    }
}

/// FUN_001d9870: empty.
pub fn ee_sub_1d9870<M: QMem + ?Sized>(m: &mut M) {}

/// FUN_001c9940: (a << 16) | (b & 0xffff).
pub fn ee_sub_1c9940<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    (a0 << 16) | (a1 & 0xffff)
}

/// FUN_001fbf00(o): FUN_002323d8(*(o+0x40)).
pub fn ee_sub_1fbf00<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let x = m.r32(a0 + 0x40);
    m.call(0x0023_23d8, &[x], &[]);
}

/// FUN_0014f610(o, f): *(o+0x4c) = f; FUN_0014db70(o).
pub fn ee_sub_14f610<M: QMem + ?Sized>(m: &mut M, a0: u32, f12: f32) {
    m.wf(a0 + 0x4c, f12);
    m.call(0x0014_db70, &[a0], &[]);
}

/// FUN_0015e3c0(base, key): base + 0x1838 + ((key * 0x95675) & 0x1ff) * 12.
pub fn ee_sub_15e3c0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let h = a1.wrapping_mul(0x0009_5675) & 0x1ff;
    a0.wrapping_add(h.wrapping_mul(12).wrapping_add(0x1838))
}

/// FUN_0016f148(o): FUN_00205eb8(o); +0xe78 = 1.
pub fn ee_sub_16f148<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0020_5eb8, &[a0], &[]);
    m.w32(a0 + 0xe78, 1);
}

/// FUN_001b0d60(o): FUN_001cb518(o); FUN_001b0d90(o).
pub fn ee_sub_1b0d60<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001c_b518, &[a0], &[]);
    m.call(0x001b_0d90, &[a0], &[]);
}

/// FUN_00227350(o): FUN_00231100(o); +0x110 = -1.
pub fn ee_sub_227350<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0023_1100, &[a0], &[]);
    m.w32(a0 + 0x110, 0xffff_ffff);
}

/// FUN_0015e958(w, flags, key, anchor, chain): FUN_0015e678(.., flags | 0x200, .., 1, &out) ? out : 0.
pub fn ee_sub_15e958<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, t0: u32) -> u32 {
    let sp = stack_push(m, 0x10);
    let r = m.call(0x0015_e678, &[a0, a1 | 0x200, a2, a3, t0, 1, sp], &[]).0;
    let out = m.r32(sp);
    stack_pop(m, 0x10);
    if r != 0 {
        out
    } else {
        0
    }
}

/// FUN_001c7e38(o): FUN_001c4e50(o); +0x1dc = 50.0.
pub fn ee_sub_1c7e38<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001c_4e50, &[a0], &[]);
    m.wf(a0 + 0x1dc, 50.0);
}

/// FUN_001fcfa0(o, id): first node in list *(o+0xec) (next +0x44) with +0x40 == id, else 0.
pub fn ee_sub_1fcfa0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let mut p = m.r32(a0 + 0xec);
    while p != 0 {
        if m.r32(p + 0x40) == a1 {
            return p;
        }
        p = m.r32(p + 0x44);
    }
    0
}

/// FUN_0019ca20(o): FUN_001a5848(o); FUN_0011a2e8(0x3e1420, o).
pub fn ee_sub_19ca20<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5848, &[a0], &[]);
    m.call(0x0011_a2e8, &[0x003e_1420, a0], &[]);
}

/// FUN_00183fb8(o): FUN_001a6178(o); FUN_0011a220(0x3e1414, o).
pub fn ee_sub_183fb8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_6178, &[a0], &[]);
    m.call(0x0011_a220, &[0x003e_1414, a0], &[]);
}

/// FUN_001cb9b0(o): FUN_001a6178(o); FUN_0011a220(0x3e13cc, o).
pub fn ee_sub_1cb9b0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_6178, &[a0], &[]);
    m.call(0x0011_a220, &[0x003e_13cc, a0], &[]);
}

/// FUN_00150de8(o): FUN_00258040(o+0xc4, 0); FUN_00258040(o+0xc0, 0).
pub fn ee_sub_150de8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0025_8040, &[a0 + 0xc4, 0], &[]);
    m.call(0x0025_8040, &[a0 + 0xc0, 0], &[]);
}

/// FUN_001f0fa0(o): +0x30 = 0; t = +0x38; old = +0x34; +0x38 = -1; +0x34 = t; FUN_001f13a8(o, old, t, t).
pub fn ee_sub_1f0fa0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.w32(a0 + 0x30, 0);
    let a3 = m.r32(a0 + 0x38);
    let a1 = m.r32(a0 + 0x34);
    m.w32(a0 + 0x38, 0xffff_ffff);
    m.w32(a0 + 0x34, a3);
    m.call(0x001f_13a8, &[a0, a1, a3, a3], &[]);
}

/// FUN_0019c870(o, out): selects a word by +0x1c (state); *out = 0.
pub fn ee_sub_19c870<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let s = m.r32(a0 + 0x1c) as i32;
    let v = if s < 0xb {
        m.r32(a0 + 0xc0)
    } else if s < 0xe {
        let p = m.r32(a0);
        m.r32(p + 0x520)
    } else if s == 0x11 {
        m.r32(a0 + 0xc4)
    } else {
        m.r32(a0 + 0xc0)
    };
    m.w32(a1, 0);
    v
}

/// FUN_0017dcb8(o): +0x30 == 1 && FUN_00178d00(+0x18) != 0.
pub fn ee_sub_17dcb8<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let s = m.r32(a0 + 0x30);
    if s != 1 {
        return 0;
    }
    let p = m.r32(a0 + 0x18);
    if m.call(0x0017_8d00, &[p], &[]).0 != 0 {
        s
    } else {
        0
    }
}

/// FUN_001de568(o): FUN_001f36a8; FUN_001de5d8; FUN_001dedb0; FUN_001df260(o, 0).
pub fn ee_sub_1de568<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001f_36a8, &[a0], &[]);
    m.call(0x001d_e5d8, &[a0], &[]);
    m.call(0x001d_edb0, &[a0], &[]);
    m.call(0x001d_f260, &[a0, 0], &[]);
}

/// FUN_00184f50(o): FUN_0021d278(o); FUN_00182c70(o+0x100); +0xe8 = 1, +0x130 = 9, +0x12c = 8.
pub fn ee_sub_184f50<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0021_d278, &[a0], &[]);
    m.call(0x0018_2c70, &[a0 + 0x100], &[]);
    m.w32(a0 + 0xe8, 1);
    m.w32(a0 + 0x130, 9);
    m.w32(a0 + 0x12c, 8);
}

/// FUN_0013cfd8(o): FUN_00183fb8(o); FUN_00149ab8(world, o, *(u64 *)0x2a4bb0, 0) (low word of the PMF).
pub fn ee_sub_13cfd8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0018_3fb8, &[a0], &[]);
    let k = m.r32(0x002a_4bb0);
    m.call(0x0014_9ab8, &[GS, a0, k, 0], &[]);
}

/// FUN_0021bb88(o): FUN_001f2108(o); +0x170 = 1.0, +0x174 = +0x178 = 0; +0x184 = 100.0.
pub fn ee_sub_21bb88<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001f_2108, &[a0], &[]);
    m.wf(a0 + 0x170, 1.0);
    m.w32(a0 + 0x178, 0);
    m.w32(a0 + 0x174, 0);
    m.wf(a0 + 0x184, 100.0);
}

/// FUN_0013aaa0(o): FUN_001f36a8(o); if +0x1f4: +0x1fc = FUN_001ae1d0(&{0xc0, 0x10}).
pub fn ee_sub_13aaa0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001f_36a8, &[a0], &[]);
    if m.r32(a0 + 0x1f4) != 0 {
        let sp = stack_push(m, 0x10);
        m.w32(sp, 0xc0);
        m.w32(sp + 4, 0x10);
        let r = m.call(0x001a_e1d0, &[sp], &[]).0;
        stack_pop(m, 0x10);
        m.w32(a0 + 0x1fc, r);
    }
}

/// FUN_001b1e38(o): FUN_001f2d48(o); FUN_001305b8(0, o, 0, o+0x234, 0.0, 1.0); FUN_0012e460(o+0x268).
pub fn ee_sub_1b1e38<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001f_2d48, &[a0], &[]);
    m.call(0x0013_05b8, &[0, a0, 0, a0 + 0x234], &[f4(0.0), f4(1.0)]);
    m.call(0x0012_e460, &[a0 + 0x268], &[]);
}

/// FUN_00148bb0(o): FUN_001f2108(o); FUN_00148718(o+0x150); flags |= 2.
pub fn ee_sub_148bb0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001f_2108, &[a0], &[]);
    m.call(0x0014_8718, &[a0 + 0x150], &[]);
    or40(m, a0, 2);
}

/// FUN_001817e8(o): FUN_0017f090(o); flags |= 8; FUN_0011a1f8(o+0x28c, 0x60).
pub fn ee_sub_1817e8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0017_f090, &[a0], &[]);
    or40(m, a0, 8);
    m.call(0x0011_a1f8, &[a0 + 0x28c, 0x60], &[]);
}

/// FUN_0012e648(o): FUN_001a5c48(o); +0x124 = 1; if bit 60 of *(u64*)(o+0x28): FUN_0014e328(0x2ddf40, 10, 0x2de3e8, 0, o).
pub fn ee_sub_12e648<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5c48, &[a0], &[]);
    m.w32(a0 + 0x124, 1);
    if (m.r64(a0 + 0x28) >> 60) & 1 != 0 {
        m.call(0x0014_e328, &[0x002d_df40, 10, 0x002d_e3e8, 0, a0], &[]);
    }
}

/// FUN_00231300(o, a1): FUN_0022fd38(&tmp, 0, +0x50, a1); +0x60..+0x90 = tmp with words 3 of each qw zeroed;
/// FUN_002313f0(o).
pub fn ee_sub_231300<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let sp = stack_push(m, 0x30);
    let a2 = m.r32(a0 + 0x50);
    m.call(0x0022_fd38, &[sp, 0, a2, a1], &[]);
    let q0 = m.rv(sp);
    let q1 = m.rv(sp + 0x10);
    m.wv(a0 + 0x60, q0);
    m.wv(a0 + 0x70, q1);
    let q2 = m.rv(sp + 0x20);
    m.w32(a0 + 0x6c, 0);
    m.wv(a0 + 0x80, q2);
    m.w32(a0 + 0x8c, 0);
    m.w32(a0 + 0x7c, 0);
    stack_pop(m, 0x30);
    m.call(0x0023_13f0, &[a0], &[]);
}

/// FUN_001c2170(o): FUN_001a5c48(o); +0x130 = q = FUN_001c1f28(o+0x60); +0x124 = FUN_001c0d70(q, 0);
/// +0x140 = q = FUN_001c1f28(o + 0x40 + (+0x50 << 5)); +0x128 = FUN_001c0d70(q, 0).
pub fn ee_sub_1c2170<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5c48, &[a0], &[]);
    let q = m.call(0x001c_1f28, &[a0 + 0x60], &[]).1;
    m.wv(a0 + 0x130, q);
    let r = m.call(0x001c_0d70, &[0], &[q]).0;
    m.w32(a0 + 0x124, r);
    let n = m.r32(a0 + 0x50);
    let p = (n << 5).wrapping_add(a0).wrapping_add(0x40);
    let q = m.call(0x001c_1f28, &[p], &[]).1;
    m.wv(a0 + 0x140, q);
    let r = m.call(0x001c_0d70, &[0], &[q]).0;
    m.w32(a0 + 0x128, r);
}

/// FUN_001ef8c8(o): FUN_001f36e0(o); v = *(0x2ddfd0); if !+0x150: v.z = FUN_001a62e8(o).z; vslot 28 (o, v).
pub fn ee_sub_1ef8c8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001f_36e0, &[a0], &[]);
    let mut v = m.rv(0x002d_dfd0);
    if m.r32(a0 + 0x150) == 0 {
        let q = m.call(0x001a_62e8, &[a0], &[]).1;
        v[2] = q[2];
    }
    vslot(m, a0, 28, &[], &[v]);
}

/// FUN_001b1bf0(o): for 6 slots at o+0x17c (stride 12): if s[0] != -1 { s[1] = FUN_001b1b98(o, s[0]); s[0] = -1 }.
pub fn ee_sub_1b1bf0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let mut s0 = a0 + 0x17c;
    let mut n: i32 = 5;
    loop {
        let a1 = m.r32(s0);
        if a1 != 0xffff_ffff {
            let r = m.call(0x001b_1b98, &[a0, a1], &[]).0;
            m.w32(s0 + 4, r);
            m.w32(s0, 0xffff_ffff);
        }
        n -= 1;
        s0 = s0.wrapping_add(0xc);
        if n < 0 {
            break;
        }
    }
}

/// FUN_0013f810(a, it): while (i = FUN_00117d50(it)) >= 0: e = 0x2ac880 + i*0x28; FUN_0013e818(a, e, e[1] & 0x2580, it).
pub fn ee_sub_13f810<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    loop {
        let i = m.call(0x0011_7d50, &[a1], &[]).0;
        if (i as i32) < 0 {
            break;
        }
        let e = i.wrapping_mul(0x28).wrapping_add(0x002a_c880);
        let f = m.r32(e + 4) & 0x2580;
        m.call(0x0013_e818, &[a0, e, f, a1], &[]);
    }
}

/// FUN_0022b720(o): FUN_001a5270(o); +0x6e0 = +0x5c = -1; +0x50 = -1.0; FUN_0011a050(o+0x8f0, 0xa0);
/// FUN_001a6ca8(o, 1); flags |= 2.
pub fn ee_sub_22b720<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5270, &[a0], &[]);
    m.w32(a0 + 0x6e0, 0xffff_ffff);
    m.w32(a0 + 0x5c, 0xffff_ffff);
    m.wf(a0 + 0x50, -1.0);
    m.call(0x0011_a050, &[a0 + 0x8f0, 0xa0], &[]);
    m.call(0x001a_6ca8, &[a0, 1], &[]);
    or40(m, a0, 2);
}

/// FUN_001842b0(o): FUN_00183f50(o); +0x188/+0x190/+0x198 = FUN_0015e938(world, 4, +0x184/+0x18c/+0x194, o).
pub fn ee_sub_1842b0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0018_3f50, &[a0], &[]);
    for k in 0..3u32 {
        let a2 = m.r32(a0 + 0x184 + 8 * k);
        let r = m.call(0x0015_e938, &[GS, 4, a2, a0], &[]).0;
        m.w32(a0 + 0x188 + 8 * k, r);
    }
}

/// FUN_0013a9e8(o): FUN_001f2108(o); field init; flags |= 2; +0x200 = o.
pub fn ee_sub_13a9e8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001f_2108, &[a0], &[]);
    let a2 = m.r32(a0 + 0x40);
    let f1 = m.rf(0x002b_c9c8);
    let f0 = m.rf(0x002a_4b08);
    m.wf(a0 + 0x160, f1);
    m.wf(a0 + 0x154, f0);
    m.w32(a0 + 0x1e0, 1);
    m.w32(a0 + 0x1ec, 0x100);
    m.w32(a0 + 0x1f0, (-10i32) as u32);
    m.wf(a0 + 0x15c, f1);
    m.w32(a0 + 0x150, 0);
    if a2 & 1 == 0 {
        let x = m.r32(a2);
        m.w32(a2, x | 2);
    } else {
        m.w32(a0 + 0x40, a2 | 2);
    }
    m.w32(a0 + 0x200, a0);
}

/// FUN_00256c58(slot, p): refcounted assign: ++*p; old = *slot; if --*old == 0 destroy old; *slot = p.
pub fn ee_sub_256c58<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if a1 != 0 {
        let x = m.r32(a1);
        m.w32(a1, x.wrapping_add(1));
    }
    let s0 = m.r32(a0);
    if s0 != 0 {
        let x = m.r32(s0).wrapping_sub(1);
        m.w32(s0, x);
        if x == 0 {
            m.call(0x0025_6d40, &[s0 + 0x10, 2], &[]);
            m.call(0x0012_9ec8, &[s0 + 8], &[]);
            m.call(0x0025_50f8, &[s0], &[]);
            m.call(0x0011_cd98, &[0x0045_20a8, s0], &[]);
        }
    }
    m.w32(a0, a1);
}

/// FUN_00159118(o): memset(o, 0, 0x100); +4 = -1; +0 = 1; +0x10..+0x50 = 0x2de5b0[0..0x40];
/// +0x50..+0xc0 = 0x2de5f0[0..0x70].
pub fn ee_sub_159118<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0023_33b4, &[a0, 0, 0x100], &[]);
    m.w32(a0 + 4, 0xffff_ffff);
    m.w32(a0, 1);
    for k in (0..0x40u32).step_by(16) {
        let q = m.rv(0x002d_e5b0 + k);
        m.wv(a0 + 0x10 + k, q);
    }
    for k in (0..0x70u32).step_by(16) {
        let q = m.rv(0x002d_e5f0 + k);
        m.wv(a0 + 0x50 + k, q);
    }
}

/// FUN_001d1370(o): FUN_001c3d70(o); field init.
pub fn ee_sub_1d1370<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001c_3d70, &[a0], &[]);
    let f3 = f32::from_bits(0x44bb_8000);
    m.wf(a0 + 0x51c, 500.0);
    m.w32(a0 + 0x180, 0xffff_ffff);
    m.wf(a0 + 0x520, f3);
    m.wf(a0 + 0x524, -1000.0);
    m.w32(a0 + 0x528, 0);
    m.w32(a0 + 0x59c, 0xffff_ffff);
    m.wf(a0 + 0x534, f3);
    m.wf(a0 + 0x538, f32::from_bits(0x451c_4000));
    m.wf(a0 + 0x53c, 1.0);
    m.w32(a0 + 0x55c, 100);
    m.w32(a0 + 0x578, 30);
    m.w32(a0 + 0x530, 0);
    m.w32(a0 + 0x590, 30);
    m.w32(a0 + 0x584, 30);
}

/// FUN_001aaf90(o, v): FUN_001ab040(o); *(u64*)(o+8) = v (64-bit reg: low word given, high = sign of low);
/// +0x10 = 0x21; CreateThread(&{.., 0x1aaf18, 0x2e2080, 0x4000, gp, 3}) -> +4 and DAT_004425d8;
/// FUN_002355e0(+4, o).
pub fn ee_sub_1aaf90<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.call(0x001a_b040, &[a0], &[]);
    m.w32(a0 + 8, a1);
    m.w32(a0 + 0xc, ((a1 as i32) >> 31) as u32);
    let s1 = stack_push(m, 0x30);
    m.call(0x0023_33b4, &[s1, 0, 0x30], &[]);
    m.w32(a0 + 0x10, 0x21);
    m.w32(s1 + 4, 0x001a_af18);
    m.w32(s1 + 0x10, 0x002e_5270);
    m.w32(s1 + 8, 0x002e_2080);
    m.w32(s1 + 0xc, 0x4000);
    m.w32(s1 + 0x14, 3);
    let t = m.call(0x0023_4820, &[s1], &[]).0;
    stack_pop(m, 0x30);
    m.w32(a0 + 4, t);
    m.w32(0x0044_25d8, t);
    let x = m.r32(a0 + 4);
    m.call(0x0023_55e0, &[x, a0], &[]);
}

/// FUN_001440d0(o, force): +0xe00 / +0x12fc = FUN_001d8c10(vslot102(o), 0x2a4e18 / 0x2a4e20, 0);
/// if !*(+0xe00) || force: FUN_00144188(o, +0xdfc); FUN_001441c0(o, +0x1300); +0x1308 = 0.
pub fn ee_sub_1440d0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let p = vslot(m, a0, 102, &[], &[]).0;
    let r = m.call(0x001d_8c10, &[p, 0x002a_4e18, 0], &[]).0;
    m.w32(a0 + 0xe00, r);
    let p = vslot(m, a0, 102, &[], &[]).0;
    let r = m.call(0x001d_8c10, &[p, 0x002a_4e20, 0], &[]).0;
    m.w32(a0 + 0x12fc, r);
    let e = m.r32(a0 + 0xe00);
    if m.r32(e) != 0 && a1 == 0 {
        return;
    }
    let x = m.r32(a0 + 0xdfc);
    m.call(0x0014_4188, &[a0, x], &[]);
    let x = m.r32(a0 + 0x1300);
    m.call(0x0014_41c0, &[a0, x], &[]);
    m.w32(a0 + 0x1308, 0);
}

/// FUN_00139250(o): pushes an iteration frame on the 0x2dd2fc chain, FUN_00139098 over the list at o+0x448
/// (next at +*(o+0x450)), FUN_001a5848, FUN_001a57a8, FUN_001a53e0 if !+0x10, FUN_0011a220(0x2deaf0, o); pops.
pub fn ee_sub_139250<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let sp = stack_push(m, 0x10);
    let l = a0 + 0x448;
    m.w32(sp + 4, l);
    m.w32(sp, l);
    let old = m.r32(0x002d_d2fc);
    let off = m.r32(l + 8);
    m.w32(0x002d_d2fc, sp);
    m.w32(sp + 0xc, old);
    let mut v = m.r32(a0 + 0x448);
    m.w32(sp + 8, off);
    m.w32(sp + 4, v.wrapping_add(off));
    while v != 0 {
        m.call(0x0013_9098, &[v], &[]);
        let p = m.r32(sp + 4);
        let o2 = m.r32(sp + 8);
        v = m.r32(p);
        m.w32(sp + 4, v.wrapping_add(o2));
    }
    m.call(0x001a_5848, &[a0], &[]);
    m.call(0x001a_57a8, &[a0], &[]);
    if m.r32(a0 + 0x10) == 0 {
        m.call(0x001a_53e0, &[a0], &[]);
    }
    m.call(0x0011_a220, &[0x002d_eaf0, a0], &[]);
    let old = m.r32(sp + 0xc);
    m.w32(0x002d_d2fc, old);
    stack_pop(m, 0x10);
}

/// FUN_0019e110(o, out): out[0..2] = constants / path extents +-35 / +-100.
pub fn ee_sub_19e110<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let p = m.r32(a0 + 0xc);
    if m.r32(p + 0xe4) != 0 {
        let f0 = m.rf(0x002a_6670);
        let f1 = m.rf(0x002a_6674);
        m.wf(a1, f0);
        m.wf(a1 + 4, f1);
        return;
    }
    let (add, sub) = if m.r32(a0 + 0x18) != 0 { (35.0f32, 100.0f32) } else { (100.0, 35.0) };
    let f = m.call(0x001e_6620, &[p], &[]).1[0];
    m.wf(a1, f + add);
    let p = m.r32(a0 + 0xc);
    let f = m.call(0x001e_6648, &[p], &[]).1[0];
    m.wf(a1 + 4, f - sub);
}

/// FUN_001eee00(pool, kind): pops a free block, memset(.., FUN_001ef6f0()), FUN_001ef138(kind, blk), links
/// it after the previous top, vslot 6.
pub fn ee_sub_1eee00<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let v1 = m.r32(a0 + 4);
    let base = a0 + 8;
    let s2 = if v1 == 0 { 0 } else { m.r32(base.wrapping_add(v1.wrapping_sub(1) << 2)) };
    let v = m.r32(a0 + 4);
    let s0 = m.r32(base.wrapping_add(v << 2));
    m.w32(a0 + 4, v.wrapping_add(1));
    let n = m.call(0x001e_f6f0, &[base, a1], &[]).0;
    m.call(0x0023_33b4, &[s0, 0, n], &[]);
    let o = m.call(0x001e_f138, &[a1, s0], &[]).0;
    let h = m.r32(a0);
    m.w32(o + 0x14, s2);
    m.w32(o + 0x10, h);
    if s2 != 0 {
        m.w32(s2 + 0x18, o);
    }
    vslot(m, o, 6, &[], &[]);
    o
}

/// FUN_001deea8(o, k): FUN_001f5988(o, k); unlinks every +0xac-chained node keyed k (owner-adjusted) from each
/// of +0x198 lists (stride 0x150, head +0x148) into a list; FUN_001a7e10(world, list); FUN_00149ab8(world, o,
/// *(u64*)0x2a7ac0 low word, 0).
pub fn ee_sub_1deea8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.call(0x001f_5988, &[a0, a1], &[]);
    let mut s2 = 0u32;
    let mut i: i32 = 0;
    if (m.r32(a0 + 0x198) as i32) > 0 {
        loop {
            let b = m.r32(a0 + 0x19c).wrapping_add((i as u32).wrapping_mul(0x150));
            let mut l = b + 0x148;
            let mut n = m.r32(l);
            while n != 0 {
                let own = (m.r32(n + 8) == a0) as u32;
                let v = n.wrapping_add(own << 6);
                if m.r32(v + 8) == a1 {
                    let nx = m.r32(n + 0xac);
                    m.w32(l, nx);
                    m.w32(n + 0xac, s2);
                    s2 = n;
                } else {
                    l = n + 0xac;
                }
                n = m.r32(l);
            }
            i += 1;
            if i >= m.r32(a0 + 0x198) as i32 {
                break;
            }
        }
    }
    m.call(0x001a_7e10, &[GS, s2], &[]);
    let k = m.r32(0x002a_7ac0);
    m.call(0x0014_9ab8, &[GS, a0, k, 0], &[]);
}

/// FUN_0020ae18(o): FUN_001a5c48(o); +0xac = first ancestor (via +0x10) with FUN_00132b10(p, 0x38);
/// FUN_00209130(o+0xb0, it); +0xc0 = o+0xb0 unless +0x18 == 0x2cc; if +0x7c != -1: +0xa0/+0xa4 =
/// FUN_001f4c58(+0xac, +0x7c/+0x80, 6); FUN_001300b8(+0xb4, +0xac, +0xa4, *(+0xa0+0xa0), &tmp, 0,
/// *FUN_0012f938(+0xb4, +0x84), 1.0); +0x90 = tmp+0x30.
pub fn ee_sub_20ae18<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5c48, &[a0], &[]);
    let mut s0 = m.r32(a0 + 0x10);
    while s0 != 0 {
        if m.call(0x0013_2b10, &[s0, 0x38], &[]).0 != 0 {
            break;
        }
        s0 = m.r32(s0 + 0x10);
    }
    m.w32(a0 + 0xac, s0);
    let s2 = a0 + 0xb0;
    m.call(0x0020_9130, &[s2, s0], &[]);
    if m.r32(a0 + 0x18) != 0x2cc {
        m.w32(a0 + 0xc0, s2);
    }
    let a1 = m.r32(a0 + 0x7c);
    if a1 == 0xffff_ffff {
        return;
    }
    let x = m.r32(a0 + 0xac);
    let r = m.call(0x001f_4c58, &[x, a1, 6], &[]).0;
    let x = m.r32(a0 + 0xac);
    let y = m.r32(a0 + 0x80);
    m.w32(a0 + 0xa0, r);
    let r = m.call(0x001f_4c58, &[x, y, 6], &[]).0;
    let s0 = m.r32(a0 + 0xb4);
    let y = m.r32(a0 + 0x84);
    m.w32(a0 + 0xa4, r);
    let v = m.call(0x0012_f938, &[s0, y], &[]).0;
    let sp = stack_push(m, 0x40);
    let a = m.r32(a0 + 0xa0);
    let f12 = m.rf(v);
    let a3 = m.r32(a + 0xa0);
    let a1 = m.r32(a0 + 0xac);
    let a2 = m.r32(a0 + 0xa4);
    m.call(0x0013_00b8, &[s0, a1, a2, a3, sp, 0], &[f4(f12), f4(1.0)]);
    let q = m.rv(sp + 0x30);
    stack_pop(m, 0x40);
    m.wv(a0 + 0x90, q);
}

/// FUN_001c8350(o): state pump on +0x180 (re-runs while the state changes).
pub fn ee_sub_1c8350<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001f_36e0, &[a0], &[]);
    let mut s1 = m.r32(a0 + 0x180);
    loop {
        let busy = |m: &mut M| -> bool {
            let p = m.r32(a0 + 0x19c);
            p != 0 && m.call(0x001d_ffd8, &[p], &[]).0 != 0
        };
        match s1 as i32 {
            2 => {
                if busy(m) {
                    m.call(0x001c_84d8, &[a0], &[]);
                } else if m.r32(a0 + 0x198) == m.r32(a0 + 0x1f0) {
                    m.call(0x001c_8730, &[a0, 3], &[]);
                } else {
                    m.call(0x001c_8730, &[a0, 4], &[]);
                }
            }
            1 => {
                if !busy(m) {
                    m.call(0x001c_8730, &[a0, 2], &[]);
                }
            }
            3 => {
                if !busy(m) {
                    m.call(0x001c_8188, &[1], &[]);
                    m.call(0x001c_8730, &[a0, 5], &[]);
                }
            }
            4 => {
                if !busy(m) {
                    m.call(0x001c_8730, &[a0, 0], &[]);
                }
            }
            _ => {}
        }
        let v = m.r32(a0 + 0x180);
        if v == s1 {
            break;
        }
        s1 = v;
    }
}

/// FUN_001a0300(o, tab, out): for 8 entries (stride 0x20) with a target: FUN_0012ff10(t, o, 1, 1) -> s;
/// sample s.4's curve (vtable at +0, slot +0x18) at 0 and at t.0x74; entry+0x10 = (p0 - p1) / t.0x74;
/// then FUN_00137a30(o, 0x20, 8, tab, 0, 0, 0, &r); *out = r.
pub fn ee_sub_1a0300<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    let sp = stack_push(m, 0x40);
    let mut s1 = a1;
    let mut s3: i32 = 7;
    loop {
        let t = m.r32(s1 + 4);
        if t != 0 {
            let s0 = m.call(0x0012_ff10, &[t, a0, 1, 1], &[]).0;
            if s0 != 0 {
                let a3 = m.r32(s0 + 4);
                let vt = m.r32(a3);
                let adj = m.r16(vt + 0x18) as i16 as i32 as u32;
                let f = m.r32(vt + 0x1c);
                m.call(f, &[a3.wrapping_add(adj), sp, 0, a3], &[f4(0.0), f4(1.0)]);
                let a3 = m.r32(s0 + 4);
                let t = m.r32(s1 + 4);
                let vt = m.r32(a3);
                let f12 = m.rf(t + 0x74);
                let adj = m.r16(vt + 0x18) as i16 as i32 as u32;
                let f = m.r32(vt + 0x1c);
                m.call(f, &[a3.wrapping_add(adj), sp + 0x10, 0, a3], &[f4(f12), f4(1.0)]);
                let t = m.r32(s1 + 4);
                let p0 = m.rv(sp);
                let d = m.rf(t + 0x74);
                let p1 = m.rv(sp + 0x10);
                let q = 1.0f32 / d;
                let r = [(p0[0] - p1[0]) * q, (p0[1] - p1[1]) * q, (p0[2] - p1[2]) * q, (p0[3] - p1[3]) * q];
                m.wv(s1 + 0x10, r);
            }
        }
        s3 -= 1;
        s1 = s1.wrapping_add(0x20);
        if s3 < 0 {
            break;
        }
    }
    m.call(0x0013_7a30, &[a0, 0x20, 8, a1, 0, 0, 0, sp + 0x30], &[]);
    let r = m.r32(sp + 0x30);
    stack_pop(m, 0x40);
    m.w32(a2, r);
}

/// FUN_0013b2f0(o, hit): FUN_001f7588 gate; on flag 8 pushes along the normalised hit+0x30 (or the default
/// *(0x2a2dc0) if shorter than 1e-4) via FUN_0013b428; FUN_0013b5c8(o, 0.25 or 0).
pub fn ee_sub_13b2f0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    if m.r32(a0 + 0x150) == 1 {
        return 0;
    }
    let s2 = m.call(0x001f_7588, &[a0, a1], &[]).0;
    if s2 == 0 {
        return 0;
    }
    if m.r32(a1 + 0x44) & 8 == 0 {
        return s2;
    }
    let v = m.rv(a1 + 0x30);
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    let d = if len < f32::from_bits(0x38d1_b717) {
        m.rv(0x002a_2dc0)
    } else {
        let q = 1.0f32 / len;
        [v[0] * q, v[1] * q, v[2] * q, v[3] * q]
    };
    let p = m.call(0x001a_62e8, &[a0], &[]).1;
    m.call(0x0013_b428, &[a0], &[p, d]);
    let h = m.r32(a1);
    if m.call(0x0013_2b10, &[h, 0x4f], &[]).0 != 0 {
        m.call(0x0013_b5c8, &[a0], &[f4(0.25)]);
    } else {
        m.call(0x0013_b5c8, &[a0], &[f4(0.0)]);
    }
    s2
}

/// FUN_001791c8(o, st): state change to +0x4c with entry effects (FUN_00205438 0x9d / 0x9e + vslot 7).
pub fn ee_sub_1791c8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let s1 = m.r32(a0 + 0x4c);
    if a1 == s1 {
        return;
    }
    m.call(0x0017_8cd8, &[a0, a1], &[]);
    let v1 = m.r32(a0 + 0x4c) as i32;
    if v1 < 0 {
        return;
    }
    if v1 < 2 {
        if s1 < 2 {
            return;
        }
        m.call(0x001e_f0d8, &[a0 + 0x18], &[]);
        let x = m.r32(a0 + 0x10);
        let e = m.call(0x0020_5438, &[x, 0x9d], &[]).0;
        let c = m.r32(a0 + 0xc0);
        m.w32(e + 0x3c, c);
        if m.r32(a0 + 0xc0) == 8 {
            let c = m.r32(a0 + 0xc8);
            m.w32(e + 0x38, c);
        }
        vslot(m, e, 7, &[], &[]);
        let x = m.r32(a0 + 0x10);
        let q = m.call(0x001a_62e8, &[x], &[]).1;
        m.wv(a0 + 0xd0, q);
        let f = m.rf(0x002b_e694);
        m.wf(a0 + 0xe0, f);
    } else if v1 < 5 {
        if s1 == 2 || s1 == 4 || s1 == 3 {
            return;
        }
        m.call(0x001e_f0d8, &[a0 + 0x18], &[]);
        let x = m.r32(a0 + 0x10);
        let e = m.call(0x0020_5438, &[x, 0x9e], &[]).0;
        let c = m.r32(a0 + 0xc8);
        m.w32(e + 0x38, c);
        vslot(m, e, 7, &[], &[]);
    }
}

/// FUN_001bc570(o): FUN_001a5c48; +0x190 = FUN_001bc2f8(o); on 4: reverse the path (+0xf4 qws, +0xec
/// distances mirrored against FUN_001e6630's length), FUN_001e6180(o), +0x190 = 3.
pub fn ee_sub_1bc570<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5c48, &[a0], &[]);
    let v = m.call(0x001b_c2f8, &[a0], &[]).0;
    m.w32(a0 + 0x190, v);
    if v == 3 || v != 4 {
        return;
    }
    let n = m.r32(a0 + 0xe8) as i32;
    let s1 = n / 2;
    let f2 = m.call(0x001e_6630, &[a0], &[]).1[0];
    let mut i: i32 = 0;
    if s1 > 0 {
        loop {
            let pts = m.r32(a0 + 0xf4);
            let n = m.r32(a0 + 0xe8);
            let a = ((i as u32) << 4).wrapping_add(pts);
            let j = n.wrapping_sub(i as u32).wrapping_sub(1);
            let b = (j << 4).wrapping_add(pts);
            let qa = m.rv(a);
            let qb = m.rv(b);
            m.wv(a, qb);
            m.wv(b, qa);
            let d = m.r32(a0 + 0xec);
            let pb = (j << 2).wrapping_add(d);
            let pa = ((i as u32) << 2).wrapping_add(d);
            let fb = m.rf(pb);
            let fa = m.rf(pa);
            m.wf(pa, fb);
            m.wf(pb, fa);
            let d = m.r32(a0 + 0xec);
            let pa = ((i as u32) << 2).wrapping_add(d);
            let x = m.rf(pa);
            m.wf(pa, f2 - x);
            let d = m.r32(a0 + 0xec);
            let pb = (j << 2).wrapping_add(d);
            let x = m.rf(pb);
            m.wf(pb, f2 - x);
            i += 1;
            if i >= s1 {
                break;
            }
        }
    }
    m.call(0x001e_6180, &[a0], &[]);
    m.w32(a0 + 0x190, 3);
}

fn v4sub(a: V4, b: V4) -> V4 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2], a[3] - b[3]]
}

/// vf sqrt-length normalise (xyz length; |v| < 1e-4 -> *(0x2a2dc0)); scales xyzw by 1/len.
fn vnorm<M: QMem + ?Sized>(m: &mut M, v: V4) -> V4 {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < f32::from_bits(0x38d1_b717) {
        m.rv(0x002a_2dc0)
    } else {
        let q = 1.0f32 / len;
        [v[0] * q, v[1] * q, v[2] * q, v[3] * q]
    }
}

/// FUN_00159240(mesh, stream): reads vertices (+0 count, +4 pos[], +0x10 normals[]), triangles (+8 count,
/// +0xc {i0,i1,i2,area}), accumulates face normals into the vertex normals, +0x14 = total area; bounds at +0x20;
/// normalises the vertex normals.
pub fn ee_sub_159240<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let sp = stack_push(m, 0x10);
    let n = m.call(0x0011_7c20, &[a1], &[]).0;
    m.w32(sp + 4, 0x10);
    m.w32(sp, n << 4);
    m.w32(a0, n);
    let r = m.call(0x001a_e1d0, &[sp], &[]).0;
    let n = m.r32(a0);
    m.w32(a0 + 4, r);
    m.w32(sp + 4, 0x10);
    m.w32(sp, n << 4);
    let r = m.call(0x001a_e2a0, &[sp], &[]).0;
    m.w32(a0 + 0x10, r);
    let mut i: i32 = 0;
    if (m.r32(a0) as i32) > 0 {
        loop {
            let p = m.r32(a0 + 4).wrapping_add((i as u32) << 4);
            i += 1;
            m.call(0x0011_7eb0, &[a1, p], &[]);
            if i >= m.r32(a0) as i32 {
                break;
            }
        }
    }
    let s6 = a0 + 0x20;
    let n = m.call(0x0011_7c20, &[a1], &[]).0;
    m.w32(sp + 4, 0x10);
    m.w32(sp, n << 4);
    m.w32(a0 + 8, n);
    let r = m.call(0x001a_e1d0, &[sp], &[]).0;
    stack_pop(m, 0x10);
    m.w32(a0 + 0xc, r);
    m.w32(a0 + 0x14, 0);
    let mut t: i32 = 0;
    if (m.r32(a0 + 8) as i32) > 0 {
        loop {
            let s2 = m.r32(a0 + 0xc).wrapping_add((t as u32) << 4);
            for k in 0..3u32 {
                let x = m.call(0x0011_7c20, &[a1], &[]).0;
                m.w32(s2 + 4 * k, x);
            }
            let f = m.call(0x0011_7e38, &[a1], &[]).1[0];
            m.wf(s2 + 0xc, f);
            let acc = m.rf(a0 + 0x14);
            let pos = m.r32(a0 + 4);
            m.wf(a0 + 0x14, acc + f);
            let i0 = m.r32(s2);
            let i1 = m.r32(s2 + 4);
            let i2 = m.r32(s2 + 8);
            let p0 = m.rv((i0 << 4).wrapping_add(pos));
            let p1 = m.rv((i1 << 4).wrapping_add(pos));
            let p2 = m.rv((i2 << 4).wrapping_add(pos));
            let a = v4sub(p1, p0);
            let b = v4sub(p2, p0);
            let c = [a[1] * b[2] - b[1] * a[2], a[2] * b[0] - b[2] * a[0], a[0] * b[1] - b[0] * a[1], a[3]];
            for k in 0..3u32 {
                let idx = m.r32(s2 + 4 * k);
                let p = m.r32(a0 + 0x10).wrapping_add(idx << 4);
                let v = m.rv(p);
                m.wv(p, [v[0] + c[0], v[1] + c[1], v[2] + c[2], v[3] + c[3]]);
            }
            t += 1;
            if t >= m.r32(a0 + 8) as i32 {
                break;
            }
        }
    }
    m.call(0x0011_7eb0, &[a1, s6], &[]);
    let mut i: i32 = 0;
    if (m.r32(a0) as i32) > 0 {
        loop {
            let p = ((i as u32) << 4).wrapping_add(m.r32(a0 + 0x10));
            let v = m.rv(p);
            let r = vnorm(m, v);
            m.wv(p, r);
            i += 1;
            if i >= m.r32(a0) as i32 {
                break;
            }
        }
    }
}

/// FUN_00172320(o, st): state switch (jump table 0x2a53a0, 6 cases) with entry effects.
pub fn ee_sub_172320<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if a1 == m.r32(a0 + 0x30) {
        return;
    }
    m.call(0x001e_f0d8, &[a0 + 0x18], &[]);
    m.w32(a0 + 0x30, a1);
    let f = m.rf(0x002d_ded0);
    m.wf(a0 + 0x34, f);
    match a1 {
        0 => {
            let x = m.r32(a0 + 0x10);
            let e = m.call(0x0020_5438, &[x, 0x9d], &[]).0;
            m.w32(e + 0x3c, 0xc);
            vslot(m, e, 7, &[], &[]);
        }
        1 => {
            let x = m.r32(a0 + 0x10);
            let s1 = m.call(0x0020_5438, &[x, 0xba], &[]).0;
            m.w32(s1 + 0xc0, 1);
            let q = m.call(0x0017_2760, &[a0], &[]).1;
            let sp = stack_push(m, 0x20);
            m.call(0x0025_62a8, &[sp], &[]);
            m.wv(sp + 0x10, q);
            m.call(0x0017_dd38, &[s1, sp], &[]);
            stack_pop(m, 0x20);
            vslot(m, s1, 7, &[], &[]);
            let f = m.rf(0x002a_5398);
            m.wf(a0 + 0x38, f);
        }
        2 => {
            let q = m.call(0x0017_2760, &[a0], &[]).1;
            let x = m.r32(a0 + 0x10);
            let p = m.call(0x001a_62e8, &[x], &[]).1;
            let s0 = m.r32(a0 + 0x10).wrapping_add(0xfd0);
            let d = v4sub(q, p);
            let ang = m.call(0x0024_58f0, &[], &[f4(d[1]), f4(d[0])]).1[0];
            m.call(0x0020_9818, &[s0], &[f4(ang)]);
            let x = m.r32(a0 + 0x10);
            let e = m.call(0x0020_5438, &[x, 0x9e], &[]).0;
            vslot(m, e, 7, &[], &[]);
            m.call(0x0017_2560, &[a0], &[]);
        }
        3 => {
            let x = m.r32(a0 + 0x10);
            let s0 = m.call(0x0020_5438, &[x, 0xba], &[]).0;
            m.w32(s0 + 0xc0, 1);
            m.call(0x0017_28b0, &[a0], &[]);
            m.call(0x0017_2f10, &[a0], &[]);
            vslot(m, s0, 7, &[], &[]);
        }
        4 => {
            let x = m.r32(a0 + 0x10);
            let e = m.call(0x0020_5438, &[x, 0x9d], &[]).0;
            m.w32(e + 0x3c, 0xe);
            vslot(m, e, 7, &[], &[]);
            m.call(0x0017_25e0, &[a0, 0], &[]);
            m.w32(a0 + 0x48, 0);
        }
        5 => {
            let x = m.r32(a0 + 0x10);
            let e = m.call(0x0020_5438, &[x, 0x9e], &[]).0;
            m.w32(e + 0x3c, 1);
            vslot(m, e, 7, &[], &[]);
        }
        _ => {}
    }
}

/// all contact nodes of *(o+0xec)+0x64 (lists chained via +4, nodes via +0xac)
fn contact_nodes<M: QMem + ?Sized>(m: &mut M, o: u32) -> Vec<u32> {
    let mut out = Vec::new();
    let e = m.r32(o + 0xec);
    let mut l = m.r32(e + 0x64);
    while l != 0 {
        let mut n = m.r32(l);
        while n != 0 {
            out.push(n);
            n = m.r32(n + 0xac);
        }
        l = m.r32(l + 4);
    }
    out
}

/// node normal (+0x80), negated unless the node's owner (+8) is o
fn contact_normal<M: QMem + ?Sized>(m: &mut M, n: u32, o: u32) -> V4 {
    let v = m.rv(n + 0x80);
    if m.r32(n + 8) == o {
        v
    } else {
        [-v[0], -v[1], -v[2], -v[3]]
    }
}

fn dot3(a: V4, b: V4) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// FUN_0019a098(o, last, t): rate-limited (1 s) contact-support test: builds the NxN normal Gram matrix and
/// rhs against -*(0x2a2dc0), solves via FUN_0011ad30, returns (resulting z > -0.1).
pub fn ee_sub_19a098<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, f12: f32) -> u32 {
    let f1 = m.rf(0x002d_ded0);
    if f1 - f12 < 1.0 {
        return 0;
    }
    if a1 != 0 {
        let x = m.rf(a1);
        if f1 - x < 1.0 {
            return 0;
        }
        m.wf(a1, f1);
    }
    if m.r32(a0 + 0x208) != 0 {
        return 0;
    }
    let s4 = contact_nodes(m, a0).len() as u32;
    if s4 == 0 {
        return 0;
    }
    let sp = stack_push(m, 0x30);
    m.call(0x001a_e478, &[sp], &[]);
    let d = m.rv(0x002a_2dc0);
    let g = [-d[0], -d[1], -d[2], -d[3]];
    m.wv(sp + 0x20, g);
    let s0p = sp + 0x10;
    m.w32(sp + 0x10, s4.wrapping_mul(s4) << 2);
    m.w32(s0p + 4, 0x10);
    let s6 = m.call(0x001a_e318, &[s0p], &[]).0;
    m.w32(sp + 0x10, s4 << 2);
    m.w32(s0p + 4, 0x10);
    let s5 = m.call(0x001a_e318, &[s0p], &[]).0;
    m.w32(sp + 0x10, s4 << 2);
    m.w32(s0p + 4, 0x10);
    let s0 = m.call(0x001a_e318, &[s0p], &[]).0;
    let mut a3 = s6;
    for a in contact_nodes(m, a0) {
        let va = contact_normal(m, a, a0);
        for b in contact_nodes(m, a0) {
            let vb = contact_normal(m, b, a0);
            m.wf(a3, dot3(va, vb));
            a3 = a3.wrapping_add(4);
        }
    }
    let mut a3 = s5;
    let g = m.rv(sp + 0x20);
    for a in contact_nodes(m, a0) {
        let va = contact_normal(m, a, a0);
        m.wf(a3, dot3(va, g));
        a3 = a3.wrapping_add(4);
    }
    let f = m.rf(0x002a_656c);
    m.call(0x0011_ad30, &[s4, s6, s5, s0], &[f4(f)]);
    let mut acc = m.rv(sp + 0x20);
    let mut a3 = s0;
    for a in contact_nodes(m, a0) {
        let va = contact_normal(m, a, a0);
        let k = m.rf(a3);
        a3 = a3.wrapping_add(4);
        acc = [acc[0] + va[0] * k, acc[1] + va[1] * k, acc[2] + va[2] * k, acc[3] + va[3] * k];
    }
    let r = (f32::from_bits(0xbdcc_cccd) < acc[2]) as u32;
    m.call(0x001a_e4a0, &[sp, 2], &[]);
    stack_pop(m, 0x30);
    r
}

/// FUN_001d1518(o): resolves the +0x1a0 (stride 0x60, by FUN_001f4c58 kind 0xd), +0x4a8 (stride 0x14) and
/// +0x4fc (pointer) tables, compacting out unresolved entries; extents / normalised offsets; FUN_001e94b0,
/// FUN_001f1448, FUN_001d14c0.
pub fn ee_sub_1d1518<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let mut mn: V4 = [0.0; 4];
    let mut mx: V4 = [0.0; 4];
    m.call(0x001f_36a8, &[a0], &[]);
    let mut i: i32 = 0;
    if (m.r32(a0 + 0x190) as i32) > 0 {
        let mut s0 = a0 + 0x1a0;
        loop {
            let k = m.r32(s0);
            let v = m.call(0x001f_4c58, &[a0, k, 0xd], &[]).0;
            m.w32(s0 + 4, v);
            if v == 0 {
                let n = m.r32(a0 + 0x190);
                let len = n.wrapping_sub(i as u32).wrapping_mul(0x60).wrapping_sub(0x60);
                m.call(0x0023_32b8, &[s0, s0 + 0x60, len], &[]);
                s0 = s0.wrapping_sub(0x60);
                i -= 1;
                let n = m.r32(a0 + 0x190).wrapping_sub(1);
                m.w32(a0 + 0x190, n);
            } else {
                let q = m.rv(v + 0x70);
                let p = m.call(0x001a_6f00, &[0, a0], &[q]).1;
                m.wv(s0 + 0x30, p);
                let f3 = m.rf(s0 + 0x28);
                let f4v = m.rf(s0 + 0x24);
                for l in 0..3 {
                    mn[l] = if p[l] < mn[l] { p[l] } else { mn[l] };
                    mx[l] = if p[l] > mx[l] { p[l] } else { mx[l] };
                }
                let d = m.rv(0x002a_2dc0);
                m.wv(s0 + 0x40, [p[0] + d[0] * f3, p[1] + d[1] * f3, p[2] + d[2] * f3, p[3] + d[3] * f3]);
                let d = m.rv(0x002a_2dc0);
                m.wv(s0 + 0x50, [p[0] + d[0] * f4v, p[1] + d[1] * f4v, p[2] + d[2] * f4v, p[3] + d[3] * f4v]);
            }
            i += 1;
            s0 = s0.wrapping_add(0x60);
            if i >= m.r32(a0 + 0x190) as i32 {
                break;
            }
        }
    }
    m.wf(a0 + 0x4a0, mx[0] - mn[0]);
    if (m.r32(a0 + 0x190) as i32) > 0 {
        let f2 = (-mn[0]).max(mx[0]);
        let f3 = (-mn[1]).max(mx[1]);
        let mut e = a0 + 0x1a0;
        let mut i: i32 = 0;
        loop {
            let x = m.rf(e + 0x30);
            i += 1;
            let y = m.rf(e + 0x34);
            m.wf(e + 0xc, x / f2);
            m.wf(e + 0x10, y / f3);
            if i >= m.r32(a0 + 0x190) as i32 {
                break;
            }
            e = e.wrapping_add(0x60);
        }
    }
    let mut i: i32 = 0;
    if (m.r32(a0 + 0x4a4) as i32) > 0 {
        let mut s0 = a0 + 0x4a8;
        loop {
            let k = m.r32(s0);
            let v = m.call(0x001f_4c58, &[a0, k, 6], &[]).0;
            m.w32(s0 + 4, v);
            if v == 0 {
                let n = m.r32(a0 + 0x4a4);
                let len = n.wrapping_sub(i as u32).wrapping_mul(0x14).wrapping_sub(0x14);
                m.call(0x0023_32b8, &[s0, s0 + 0x14, len], &[]);
                s0 = s0.wrapping_sub(0x14);
                i -= 1;
                let n = m.r32(a0 + 0x4a4).wrapping_sub(1);
                m.w32(a0 + 0x4a4, n);
            } else {
                let t1 = m.rv(0x002a_2dd0);
                let r = m.call(0x0012_dc18, &[a0, v, 1, 7, 3], &[t1]).0;
                m.w32(s0 + 8, r);
            }
            i += 1;
            s0 = s0.wrapping_add(0x14);
            if i >= m.r32(a0 + 0x4a4) as i32 {
                break;
            }
        }
    }
    let mut i: i32 = 0;
    if (m.r32(a0 + 0x4f8) as i32) > 0 {
        let mut s2 = a0 + 0x4fc;
        loop {
            let s0 = m.r32(s2);
            let k = m.r32(s0 + 0x18);
            let v = m.call(0x0015_e938, &[GS, 1, k, a0], &[]).0;
            m.w32(s0 + 0x1c, v);
            let mut remove = v == 0;
            if !remove {
                let k = m.r32(s0 + 0x2c);
                let v = m.call(0x001f_4c58, &[a0, k, 6], &[]).0;
                m.w32(s0 + 0x30, v);
                remove = v == 0 && m.r32(s0 + 0x2c) != 0xffff_ffff;
            }
            if !remove {
                let k = m.r32(s0 + 0x38);
                let v = m.call(0x0015_e938, &[GS, 1, k, a0], &[]).0;
                m.w32(s0 + 0x3c, v);
                remove = v == 0 && m.r32(s0 + 0x38) != 0xffff_ffff;
            }
            if remove {
                let n = m.r32(a0 + 0x4f8);
                let len = (n.wrapping_sub(i as u32) << 2).wrapping_sub(4);
                m.call(0x0023_32b8, &[s2, s2 + 4, len], &[]);
                s2 = s2.wrapping_sub(4);
                i -= 1;
                let n = m.r32(a0 + 0x4f8).wrapping_sub(1);
                m.w32(a0 + 0x4f8, n);
            } else {
                let x = m.r32(s0 + 0x1c);
                let r = m.call(0x001f_0380, &[x, a0, 0xffff_ffff, 0], &[]).0;
                let a1 = m.r32(s0 + 0x30);
                m.w32(s0 + 0x20, r);
                if a1 != 0 {
                    let t1 = m.rv(0x002a_2dd0);
                    let r = m.call(0x0012_dc18, &[a0, a1, 1, 7, 3], &[t1]).0;
                    m.w32(s0 + 0x34, r);
                }
            }
            i += 1;
            s2 = s2.wrapping_add(4);
            if i >= m.r32(a0 + 0x4f8) as i32 {
                break;
            }
        }
    }
    let r = m.call(0x001e_94b0, &[0x2c9, 0x10], &[]).0;
    let x = m.r32(a0 + 0x56c);
    m.w32(a0 + 0x5ac, r);
    m.call(0x001f_1448, &[x, a0, 0xffff_ffff, 0, a0 + 0x570], &[]);
    m.call(0x001d_14c0, &[a0], &[]);
}

/// FUN_0015c690(o, req): builds a sound/emitter descriptor from template *(o+0xe4) (+ request overrides by
/// flag bits), FUN_0015bd20(count, &desc, ctx), then positions / configures the emitter *(o+0xf0).
pub fn ee_sub_15c690<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let sp = stack_push(m, 0x170);
    let s5 = sp + 0x20;
    let e = m.r32(a0 + 0xe4);
    let src = m.r32(e + 8);
    for k in (0..0x10u32).step_by(4) {
        let w = m.r32(src + k);
        m.w32(sp + k, w);
    }
    let w = m.r32(src + 0x10);
    m.w32(sp + 0x10, w);
    for k in (0..0x100u32).step_by(16) {
        let q = m.rv(e + k);
        m.wv(s5 + k, q);
    }
    m.w32(sp + 0x28, sp);
    let mut s3: i32 = if m.r32(a1) & 0x80 != 0 {
        m.r32(a1 + 0x38) as i32
    } else {
        let lo = m.rf(a0 + 0xf4);
        let hi = m.rf(a0 + 0xf8);
        let f = m.call(0x0011_d448, &[], &[f4(lo), f4(hi)]).1[0];
        f as i32
    };
    if m.r32(a1) & 1 != 0 {
        let x = m.r32(a1 + 4);
        m.w32(sp, x);
    }
    if m.r32(a1) & 2 != 0 {
        let q = m.rv(a1 + 0x10);
        m.wv(sp + 0x60, q);
    }
    let fl = m.r32(a1);
    if fl & 4 != 0 {
        let q = m.rv(a1 + 0x20);
        m.w32(sp + 0x30, 0);
        m.wv(sp + 0x50, q);
    } else if fl & 0x40 != 0 {
        let x = m.r32(a1 + 0x34);
        m.w32(sp + 0x3c, x);
    }
    if m.r32(a1) & 8 != 0 && m.r32(sp + 0x24) == 0 {
        let f12 = m.rf(a1 + 8);
        let mut f20 = f12;
        if f12 == 0.0 {
            f20 = m.rf(a1 + 0xc) * f32::from_bits(0x3c23_d70a);
        }
        let f0 = if f20 < 1.0 { (s3 as f32) * f20 } else { (s3 as f32) / f20 };
        s3 = f0 as i32;
        let v = (m.rf(a0 + 0xf8) as i32) / 2;
        if !(v < s3) {
            s3 = v;
        }
        let k = m.call(0x0011_d948, &[0x002b_dec0], &[f4(f20)]).1[0];
        let x = m.rf(sp + 0x70);
        let y = m.rf(sp + 0x74);
        f20 = if 0.5 > f20 { 0.5 } else { f20 };
        m.wf(sp + 0x70, x * k);
        m.wf(sp + 0x74, y * k);
        m.call(0x0025_6ce8, &[sp + 0x150], &[f4(f20)]);
        let s0 = m.rv(sp + 0x150);
        m.call(0x0025_6d08, &[sp + 0x160, sp + 0x100], &[]);
        let b = m.rv(sp + 0x160);
        let v = m.call(0x0025_6d28, &[], &[s0, b]).1;
        m.call(0x0025_6d18, &[sp + 0x100], &[v]);
    }
    if m.r32(sp + 0x24) == 1 {
        if m.r32(a1) & 0x10 != 0 {
            let f = m.rf(a1 + 8);
            m.wf(sp + 0xfc, f);
            m.wf(sp + 0xf8, f);
        }
        if m.r32(a1) & 0x20 != 0 {
            let p = m.r32(sp + 0x28);
            let x = m.r32(a1 + 0x30);
            m.w32(p + 0x10, x);
        }
    }
    let a2 = if m.r32(a0 + 0xfc) != 0 {
        m.w32(sp + 0x120, a0 + 0x104);
        m.w32(sp + 0x124, a0 + 0x100);
        m.w32(sp + 0x128, a0);
        sp + 0x120
    } else {
        0
    };
    if m.r32(sp) == 0 && m.r32(sp + 0x30) != 2 && m.r32(a1) & 0x48 == 0 {
        m.w32(sp, a0);
    }
    m.call(0x0015_bd20, &[s3 as u32, s5, a2], &[]);
    if m.r32(a0 + 0xf0) == 0 {
        stack_pop(m, 0x170);
        return;
    }
    match m.r32(sp + 0x30) {
        0 => {
            let q = m.rv(sp + 0x50);
            m.wv(sp + 0x130, q);
        }
        1 => {
            let p = m.r32(sp + 0x50);
            let a = m.rv(p);
            let b = m.rv(p + 0x10);
            let mut r = [0.0f32; 4];
            for l in 0..4 {
                r[l] = a[l] * 0.5 + b[l] * 0.5;
            }
            m.wv(sp + 0x130, r);
        }
        2 => {
            let p = m.r32(sp + 0x28);
            let p = m.r32(p + 4);
            let p = m.r32(p + 0xf4);
            let q = m.rv(p);
            m.wv(sp + 0x130, q);
        }
        3 => {
            let p = m.r32(sp + 0x50);
            let q = m.rv(p + 0x20);
            m.wv(sp + 0x130, q);
        }
        _ => {}
    }
    let mt = m.r32(sp + 0x3c);
    if mt != 0 {
        let v = m.rv(sp + 0x130);
        let r0 = m.rv(mt);
        let r1 = m.rv(mt + 0x10);
        let r2 = m.rv(mt + 0x20);
        let r3 = m.rv(mt + 0x30);
        let mut r = v;
        for l in 0..3 {
            r[l] = r1[l] * v[0] + r2[l] * v[1] + r3[l] * v[2];
        }
        for l in 0..4 {
            r[l] += r0[l];
        }
        m.wv(sp + 0x130, r);
    }
    let a = m.r32(sp);
    let q = m.rv(sp + 0x130);
    let p = m.call(0x001a_6f00, &[a, 0], &[q]).1;
    m.wv(sp + 0x130, p);
    stack_pop(m, 0x170);
    let f0p = m.r32(a0 + 0xf0);
    let x = m.r32(f0p);
    let s0 = m.call(0x001f_bb90, &[x], &[]).0;
    m.call(0x001f_b2f8, &[s0, 0], &[]);
    m.call(0x001f_b3f0, &[s0], &[p]);
    let f0p = m.r32(a0 + 0xf0);
    let a = m.rf(f0p + 4);
    let b = m.rf(f0p + 8);
    m.call(0x001f_b4f0, &[s0], &[f4(a), f4(b)]);
    let f0p = m.r32(a0 + 0xf0);
    let a = m.rf(f0p + 0xc);
    m.call(0x001f_b1f8, &[s0], &[f4(a)]);
    let f0p = m.r32(a0 + 0xf0);
    let a = m.rf(f0p + 0x10);
    m.call(0x001f_b2c0, &[s0], &[f4(a)]);
}

/// FUN_0022d7a0(o, stream): loads +0x50 nodes (0x100 each, at +0x54) with optional attachments (flag bits
/// 4/8/0x10/0x20 of each node), per-kind transform setup (jump table 0x2a9d00), registers attachments via
/// FUN_00230018, then the tail loaders and the +0x58/+0x5c/+0x60 lookups; clears flag 0x20 if any node.
pub fn ee_sub_22d7a0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let s1 = a1;
    let n = m.call(0x0011_7d50, &[s1], &[]).0;
    let sp = stack_push(m, 0x10);
    m.w32(sp + 4, 0x10);
    m.w32(sp, n << 8);
    m.w32(a0 + 0x50, n);
    let r = m.call(0x001a_e2a0, &[sp], &[]).0;
    stack_pop(m, 0x10);
    m.w32(a0 + 0x54, r);
    let mut fp: i32 = 0;
    if (m.r32(a0 + 0x50) as i32) > 0 {
        loop {
            let s0 = m.r32(a0 + 0x54).wrapping_add((fp as u32) << 8);
            let x = m.call(0x0011_7d50, &[s1], &[]).0;
            let v = vslot(m, a0, 50, &[x], &[]).0;
            m.w32(s0 + 0xa0, v);
            let v = m.call(0x0011_7d50, &[s1], &[]).0;
            m.w32(s0 + 0x94, v);
            let v = m.call(0x0011_7db8, &[s1], &[]).0;
            m.w32(s0 + 0x9c, v);
            let s4 = m.call(0x0011_7bd0, &[s1], &[]).0;
            if s4 & 1 != 0 {
                let f = m.call(0x0011_7e38, &[s1], &[]).1[0];
                m.w16(s0 + 0xae, 1000);
                m.w16(s0 + 0xac, ((f * 1000.0) as i32) as u16);
            } else if s4 & 2 != 0 {
                let f = m.call(0x0011_7e38, &[s1], &[]).1[0];
                m.w16(s0 + 0xac, ((f * 1000.0) as i32) as u16);
                let f = m.call(0x0011_7e38, &[s1], &[]).1[0];
                m.w16(s0 + 0xae, ((f * 1000.0) as i32) as u16);
            } else {
                m.w16(s0 + 0xae, 1000);
                m.w16(s0 + 0xac, 1000);
            }
            let (mut s6, mut s3, mut s5, mut s7) = (0u32, 0u32, 0u32, 0u32);
            let b4 = s4 & 4;
            if b4 != 0 {
                s6 = m.call(0x0012_e1f0, &[0x30], &[]).0;
                m.w32(s6 + 0x18, 0x002c_a820);
                m.w32(s6, 1);
                m.w32(s6 + 0x1c, 2);
                let v = m.call(0x0011_7d00, &[s1], &[]).0;
                m.w32(s6 + 0x14, v);
            }
            let b8 = s4 & 8;
            if b8 != 0 {
                s3 = m.call(0x0012_e1f0, &[0x80], &[]).0;
                m.w32(s3, 2);
                m.w32(s3 + 0x18, 0x002c_a7e0);
                let v = m.call(0x0011_7d00, &[s1], &[]).0;
                m.w32(s3 + 0x14, v);
                let f = m.call(0x0011_7e38, &[s1], &[]).1[0];
                m.wf(s3 + 0x60, f);
                m.call(0x0011_7f90, &[s1, s3 + 0x30], &[]);
                m.call(0x0011_7fb0, &[s1, s3 + 0x50], &[]);
                m.call(0x0011_7f90, &[s1, s3 + 0x40], &[]);
                m.call(0x0011_7fb0, &[s1, s3 + 0x58], &[]);
                let v = m.call(0x0011_7d00, &[s1], &[]).0;
                m.w32(s3 + 0x64, v);
                let v = m.call(0x0011_7d00, &[s1], &[]).0;
                m.w32(s3 + 0x68, v);
                let v = m.call(0x0011_7db8, &[s1], &[]).0;
                m.w32(s3 + 0x6c, v);
            }
            let b10 = s4 & 0x10;
            if b10 != 0 {
                s5 = m.call(0x0012_e1f0, &[0x28], &[]).0;
                m.w32(s5, 4);
                m.w32(s5 + 0x18, 0x002c_a7a0);
                let v = m.call(0x0011_7d00, &[s1], &[]).0;
                m.w32(s5 + 0x14, v);
                let f = m.call(0x0011_7e38, &[s1], &[]).1[0];
                m.wf(s5 + 0x1c, f);
                let f = m.call(0x0011_7e38, &[s1], &[]).1[0];
                m.wf(s5 + 0x20, f);
            }
            let b20 = s4 & 0x20;
            if b20 != 0 {
                s7 = m.call(0x0012_e1f0, &[0x30], &[]).0;
                m.w32(s7, 9);
                m.w32(s7 + 0x18, 0x002c_a6a0);
                m.w32(s7 + 0x14, 0x14);
                m.call(0x0011_7eb0, &[s1, s7 + 0x20], &[]);
            }
            match m.r32(s0 + 0x94) {
                0 | 13 | 15 => {
                    m.call(0x0011_7f18, &[s1, s0], &[]);
                }
                8 | 14 => {
                    m.call(0x0023_33b4, &[s0, 0, 0x40], &[]);
                    m.wf(s0, 1.0);
                    m.wf(s0 + 0x3c, 1.0);
                    m.wf(s0 + 0x28, 1.0);
                    m.wf(s0 + 0x14, 1.0);
                    let x = m.call(0x0011_7d50, &[s1], &[]).0;
                    let v = vslot(m, a0, 50, &[x], &[]).0;
                    m.w32(s0 + 0xa8, v);
                    for k in [0x30u32, 0x34, 0x38] {
                        let f = m.rf(v + k);
                        m.wf(s0 + k, -f);
                    }
                }
                _ => {
                    m.call(0x0011_7f18, &[s1, s0], &[]);
                    let q = m.call(0x0022_ca18, &[s0, s0], &[]).1;
                    m.wv(s0 + 0xc0, q);
                    m.wv(s0 + 0xe0, q);
                }
            }
            m.call(0x0022_c818, &[a0, s0], &[]);
            if b4 != 0 {
                m.w32(s6 + 4, a0);
                m.w32(s6 + 8, s0);
                m.w32(s6 + 0xc, 0xffff_ffff);
                let q = m.call(0x0022_ca18, &[s0, s0], &[]).1;
                m.wv(s6 + 0x20, q);
                m.call(0x0023_0018, &[a0, s0, s6], &[]);
            }
            if b8 != 0 {
                let v = m.r32(s3 + 0x68);
                m.w32(s3 + 4, a0);
                m.w32(s3 + 8, s0);
                m.w32(s3 + 0xc, v);
                let q = m.call(0x0022_ca18, &[s0, s0], &[]).1;
                m.wv(s3 + 0x20, q);
                m.call(0x0023_0018, &[a0, s0, s3], &[]);
            }
            if b10 != 0 {
                m.w32(s5 + 4, a0);
                m.w32(s5 + 8, s0);
                m.w32(s5 + 0xc, 0xffff_ffff);
                m.call(0x0023_0018, &[a0, s0, s5], &[]);
            }
            if b20 != 0 {
                m.w32(s7 + 4, a0);
                m.w32(s7 + 8, s0);
                m.w32(s7 + 0xc, 0xffff_ffff);
                m.call(0x0023_0018, &[a0, s0, s7], &[]);
            }
            fp += 1;
            if fp >= m.r32(a0 + 0x50) as i32 {
                break;
            }
        }
    }
    m.call(0x0022_faa0, &[a0], &[]);
    m.call(0x0018_6670, &[a0, s1], &[]);
    m.call(0x0016_03b8, &[a0, s1], &[]);
    m.call(0x0021_c888, &[a0, s1], &[]);
    for off in [0x58u32, 0x5c, 0x60] {
        let x = m.call(0x0011_7d50, &[s1], &[]).0;
        let v = vslot(m, a0, 50, &[x], &[]).0;
        m.w32(a0 + off, v);
    }
    if (m.r32(a0 + 0x50) as i32) > 0 {
        let p = m.r32(a0 + 0x40);
        if p & 1 == 0 {
            let x = m.r32(p);
            m.w32(p, x & !0x20);
        } else {
            m.w32(a0 + 0x40, p & !0x20);
        }
    }
}
