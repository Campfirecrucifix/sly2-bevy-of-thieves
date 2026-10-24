//! WIRE-Q2: queue ports (1:1 against `QMem`) of the native-state slot functions the live store still
//! fell into: C2cb230 (k20) update FUN_001403d0 and the guard (C56) update / post-load FUN_00207350 /
//! FUN_002061d8. Callees go through `m.call` (the live dispatch routes the ported ones).
use crate::engine::classes::q::q2mem::{vslot, QMem};
use crate::engine::classes::q::q6::ro32;

const Z: [f32; 4] = [0.0; 4];

/// FUN_001403d0 (C2cb230 update): base update FUN_00203fc0 + FUN_00141a50; FUN_00140338 on a
/// FUN_001c46c0(+0x150, 0x200, 0.0) hit; with no target (+0xdf4 == -1) the +0xfd0 brain steps
/// (FUN_00144640) and, when FUN_001449c8 is idle, the top of the +0x270 stack runs slot 13 (+0x150)
/// or +0x1084 runs FUN_001e46a0; then FUN_00140558 / 001a2460 / 00140750 / 00142af8, the +0xefc
/// handle drop (FUN_00131968 / 001306a8), the pad-0x100 FUN_001db490(0x3da0f0) shortcut when
/// FUN_0014fd50(0x2de2e0) is clear and +0x150 +0x80 == 1, and FUN_001eed10(+0x26c) when idle
pub fn ee_sub_1403d0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0020_3fc0, &[o], &[]);
    m.call(0x0014_1a50, &[o], &[]);
    let p = m.r32(o + 0x150);
    if m.call(0x001c_46c0, &[p, 0x200], &[Z]).0 != 0 {
        m.call(0x0014_0338, &[o], &[]);
    }
    let s1 = o + 0xfd0;
    if m.r32(o + 0xdf4) == u32::MAX {
        m.call(0x0014_4640, &[s1], &[]);
        if m.call(0x0014_49c8, &[s1], &[]).0 == 0 {
            let n = m.r32(o + 0x270) as i32;
            if n > 0 {
                let a1 = m.r32(o + 0x150);
                let t = m.r32(o + 0x274 + 4 * (n - 1) as u32);
                vslot(m, t, 13, &[a1], &[]);
            } else {
                let h = m.r32(o + 0x1084);
                if h != 0 {
                    let a1 = m.r32(o + 0x150);
                    m.call(0x001e_46a0, &[h, a1], &[]);
                }
            }
        }
    }
    m.call(0x0014_0558, &[o], &[]);
    m.call(0x001a_2460, &[o], &[]);
    m.call(0x0014_0750, &[o], &[]);
    m.call(0x0014_2af8, &[o], &[]);
    let h = m.r32(o + 0xefc);
    if h != 0 && m.call(0x0013_1968, &[h], &[]).0 != 0 {
        m.call(0x0013_06a8, &[o + 0xefc], &[]);
    }
    if m.call(0x0014_fd50, &[0x002d_e2e0], &[]).0 == 0 {
        let p = m.r32(o + 0x150);
        if m.r32(p + 0x80) == 1 && m.call(0x001c_4660, &[p, 0x100], &[Z]).0 != 0 {
            if m.r32(0x002d_fc00 + 0x10b4) & 0x108 == 0x100 {
                m.call(0x0018_7de0, &[0x002d_fc00, 0x100], &[]);
                m.call(0x001d_b490, &[0x003d_a0f0], &[]);
            }
        }
    }
    if m.call(0x0014_49c8, &[s1], &[]).0 == 0 {
        m.call(0x001e_ed10, &[o + 0x26c], &[]);
    }
}

/// FUN_00207350 (C56 guard update): base FUN_00203fc0; in-world (+0x28 bit 60) and not state 0xe
/// (+0xfbc): FUN_00207288 / 00206900, the +0xf00 target sighting (FUN_00209430 -> FUN_0020ab68 with
/// DAT_002a2dd0), +0xf0c FUN_0020acc0, +0x48c FUN_001f1cf8, the +0x498 / +0x49c helpers
/// (FUN_0020ce70 / 0020d1f8), FUN_001eed10(+0x26c), FUN_002097e0(+0xfd0), the +0x34c handle drop,
/// and +0x4b0 = *(+0x218) while +0x4a4 and +0x208 are set
pub fn ee_sub_207350<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0020_3fc0, &[o], &[]);
    if (m.r64(o + 0x28) >> 60) & 1 == 0 || m.r32(o + 0xfbc) == 0xe {
        return;
    }
    m.call(0x0020_7288, &[o], &[]);
    m.call(0x0020_6900, &[o], &[]);
    let t = m.r32(o + 0xf00);
    if t != 0 {
        let a = m.r32(t + 0xc0);
        if m.call(0x0020_9430, &[a], &[]).0 != 0 {
            let q = m.rv(0x002a_2dd0);
            m.call(0x0020_ab68, &[o, 0, 0], &[q]);
        }
    }
    if m.r32(o + 0xf0c) != 0 {
        m.call(0x0020_acc0, &[o], &[]);
    }
    if m.r32(o + 0x48c) != 0 {
        m.call(0x001f_1cf8, &[o], &[]);
    }
    let a = m.r32(o + 0x498);
    m.call(0x0020_ce70, &[a], &[]);
    let a = m.r32(o + 0x49c);
    m.call(0x0020_d1f8, &[a], &[]);
    m.call(0x001e_ed10, &[o + 0x26c], &[]);
    m.call(0x0020_97e0, &[o + 0xfd0], &[]);
    let h = m.r32(o + 0x34c);
    if h != 0 && m.call(0x0013_1968, &[h], &[]).0 != 0 {
        m.call(0x0013_06a8, &[o + 0x34c], &[]);
    }
    if m.r32(o + 0x4a4) != 0 && m.r32(o + 0x208) != 0 {
        let s = m.r32(o + 0x218);
        for k in (0..16).step_by(4) {
            let w = m.r32(s + k);
            m.w32(o + 0x4b0 + k, w);
        }
    }
}

/// FUN_002061d8 (C56 guard post-load): base FUN_00203a38, FUN_001a64e8(o, 1, 0x2c3cf8),
/// FUN_00204da8(o, +0x260), FUN_002303a0 on +0x58 / +0x5c, FUN_002080f0 / 0020aaa8, the +0x498 /
/// +0x49c helpers (FUN_001ae4f8 0x10 / 0x14 -> FUN_0020ce28 / 0020d1b0), FUN_00149a50 registration
/// when +0x3c +0x24 is clear, the 0x71 / 0x2cb sound (FUN_0013e3c8 -> FUN_00170c90 at 40.0) at
/// +0x2ec / +0x2f0, slot 8 on it and on +0x3f4 / +0x3f8, and the +0xfc4 effect (FUN_001f4c58 kind 0xc
/// -> FUN_0012dc18 with DAT_002a2de0)
pub fn ee_sub_2061d8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0020_3a38, &[o], &[]);
    m.call(0x001a_64e8, &[o, 1, 0x002c_3cf8], &[]);
    let f = m.rf(o + 0x260);
    m.call(0x0020_4da8, &[o], &[[f, 0.0, 0.0, 0.0]]);
    let a = m.r32(o + 0x58);
    m.call(0x0023_03a0, &[o, a, 2], &[]);
    let a = m.r32(o + 0x5c);
    m.call(0x0023_03a0, &[o, a, 2], &[]);
    m.call(0x0020_80f0, &[o], &[]);
    m.call(0x0020_aaa8, &[o], &[]);
    let n = m.call(0x001a_e4f8, &[0x10], &[]).0;
    m.w32(o + 0x498, n);
    m.call(0x0020_ce28, &[n, o], &[]);
    let n = m.call(0x001a_e4f8, &[0x14], &[]).0;
    m.w32(o + 0x49c, n);
    m.call(0x0020_d1b0, &[n, o], &[]);
    let c = m.r32(o + 0x3c);
    if m.r32(c + 0x24) == 0 {
        m.call(0x0014_9a50, &[0x003d_d490, o, ro32(0x002a_8608), 0], &[]);
    }
    let s = m.call(0x0013_e3c8, &[0x71, 0, 0x2cb], &[]).0;
    m.w32(o + 0x2ec, s);
    m.call(0x0017_0c90, &[s, u32::MAX, u32::MAX], &[[40.0, 0.0, 0.0, 0.0]]);
    let s = m.r32(o + 0x2ec);
    let w = m.r32(s + 0x50);
    m.w32(o + 0x2f0, w);
    vslot(m, s, 8, &[], &[]);
    for off in [0x3f4, 0x3f8] {
        let h = m.r32(o + off);
        if h != 0 {
            vslot(m, h, 8, &[], &[]);
        }
    }
    let k = m.r32(o + 0x18);
    let e = m.call(0x001f_4c58, &[o, k, 0xc], &[]).0;
    if e != 0 {
        let q = m.rv(0x002a_2de0);
        let r = m.call(0x0012_dc18, &[o, e, 0, 7, 2], &[q]).0;
        m.w32(o + 0xfc4, r);
    }
}

/// FUN_001a5d98: run the +0x34 callback chain (nodes {fn, arg, next +8}) oldest first: fn(o, arg)
pub fn ee_sub_1a5d98<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let mut v = Vec::new();
    let mut n = m.r32(o + 0x34);
    while n != 0 {
        v.push(n);
        n = m.r32(n + 8);
    }
    for &n in v.iter().rev() {
        let f = m.r32(n);
        let a = m.r32(n + 4);
        m.call(f, &[o, a], &[]);
    }
}

/// FUN_001a61a0: in the root set: in-world (+0x28 bit 60) and the +0x14 node listed on 0x3dd4bc
pub fn ee_sub_1a61a0<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    if (m.r64(o + 0x28) >> 60) & 1 == 0 {
        return 0;
    }
    let n = m.r32(o + 0x14);
    (m.call(0x0011_a380, &[0x003d_d4bc, n], &[]).0 != 0) as u32
}

/// FUN_001a6ca8: set the +0x28 kind nibble (bits 52..55) to `k`; a parentless in-world object whose
/// kind changed goes through FUN_00147e88(0x3dd490, +0x14)
pub fn ee_sub_1a6ca8<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32) {
    let f = m.r64(o + 0x28);
    if (k as i32 as i64) == (f << 8) as i64 >> 60 {
        return;
    }
    let f = (f & 0xff0f_ffff_ffff_ffff) | ((k & 0xf) as u64) << 52;
    m.w32(o + 0x28, f as u32);
    m.w32(o + 0x2c, (f >> 32) as u32);
    if m.r32(o + 0x10) == 0 && (f >> 60) & 1 != 0 {
        let n = m.r32(o + 0x14);
        m.call(0x0014_7e88, &[0x003d_d490, n], &[]);
    }
}

/// FUN_001f4c58: the 0x100-byte entry of the +0x54 table (+0x50 count) with +0x9c == a, +0x94 == b
pub fn ee_sub_1f4c58<M: QMem + ?Sized>(m: &mut M, o: u32, a: u32, b: u32) -> u32 {
    let n = m.r32(o + 0x50) as i32;
    let mut p = m.r32(o + 0x54);
    for _ in 0..n.max(0) {
        if m.r32(p + 0x9c) == a && m.r32(p + 0x94) == b {
            return p;
        }
        p = p.wrapping_add(0x100);
    }
    0
}

/// FUN_001c46c0 (JoyTakePressed): FUN_001c4660(j, mask, t) then consume the hit (FUN_001c4738)
pub fn ee_sub_1c46c0<M: QMem + ?Sized>(m: &mut M, j: u32, mask: u32, t: f32) -> u32 {
    let r = m.call(0x001c_4660, &[j, mask], &[[t, 0.0, 0.0, 0.0]]).0;
    m.call(0x001c_4738, &[j, r], &[]);
    r
}

/// FUN_0014fd50: `x` is the active camera request (DAT_002de254)
pub fn ee_sub_14fd50<M: QMem + ?Sized>(m: &mut M, x: u32) -> u32 {
    (m.r32(0x002d_e254) == x) as u32
}

/// FUN_001a58d0: slot 9 (set parent) with (p, 0, x)
pub fn ee_sub_1a58d0<M: QMem + ?Sized>(m: &mut M, o: u32, p: u32, x: u32) {
    vslot(m, o, 9, &[p, 0, x], &[]);
}

/// FUN_00149a50(w, o, k, x): a pool node (FUN_0011ccc0 on w+0x3c90) {o, k (64-bit, +4), x (+0xc)}
/// appended to w+0x3ca8 (FUN_0011a070). WIRE-S2: the 64-bit `k` (a2) travels as `k` (low word) + `k_hi`
/// (a[4], the PMF fn word; 0 when a caller passes only 4 args).
pub fn ee_sub_149a50<M: QMem + ?Sized>(m: &mut M, w: u32, o: u32, k: u32, x: u32, k_hi: u32) {
    let n = m.call(0x0011_ccc0, &[w + 0x3c90], &[]).0;
    m.w32(n, o);
    m.w32(n + 4, k);
    m.w32(n + 8, k_hi);
    m.w32(n + 0xc, x);
    m.call(0x0011_a070, &[w + 0x3ca8, n], &[]);
}

/// FUN_00170e00(o, a, b): a pool node (FUN_0011ccc0 on 0x3e1548) {a, b} appended to o+0x50
pub fn ee_sub_170e00<M: QMem + ?Sized>(m: &mut M, o: u32, a: u32, b: u32) -> u32 {
    let n = m.call(0x0011_ccc0, &[0x003e_1548], &[]).0;
    m.w32(n, a);
    m.w32(n + 4, b);
    m.call(0x0011_a070, &[o + 0x50, n], &[]);
    n
}

/// FUN_00148718 (Toggle init): state +0x90 = -1, +0x98 = +0x8c = 1.0
pub fn ee_sub_148718<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.w32(o + 0x90, u32::MAX);
    m.wf(o + 0x98, 1.0);
    m.wf(o + 0x8c, 1.0);
}

/// FUN_00148740 (Toggle): FUN_00148668(side, *o) on both 0x44-byte sides at +4
pub fn ee_sub_148740<M: QMem + ?Sized>(m: &mut M, o: u32) {
    for i in 0..2 {
        let a = m.r32(o);
        m.call(0x0014_8668, &[o + 4 + 0x44 * i, a], &[]);
    }
}

/// FUN_00148a78 (Toggle set state): +0x90 = k, +0x94 = clock; 0 -> 1 runs FUN_001487a0, any
/// non-initial change to 0 runs FUN_00148860
pub fn ee_sub_148a78<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32) {
    let cur = m.r32(o + 0x90);
    if cur == k {
        return;
    }
    let on = cur == 0 && k == 1;
    let off = cur != u32::MAX && k == 0;
    m.w32(o + 0x90, k);
    let t = m.r32(0x002d_ded0);
    m.w32(o + 0x94, t);
    if on {
        m.call(0x0014_87a0, &[o], &[]);
    }
    if off {
        m.call(0x0014_8860, &[o], &[]);
    }
}

/// FUN_001ffa40(o, k): word +4 of entry k (8 bytes) of the +0x39e8 table's +8 array; 0 for k == -1
pub fn ee_sub_1ffa40<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32) -> u32 {
    if k == u32::MAX {
        return 0;
    }
    let t = m.r32(o + 0x39e8);
    let b = m.r32(t + 8);
    m.r32((k << 3).wrapping_add(b) + 4)
}

/// FUN_001128c8: new 0x60-byte object (FUN_001ae4f8) with vtable 0x2cf880, +0 / +4 cleared
pub fn ee_sub_1128c8<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let n = m.call(0x001a_e4f8, &[0x60], &[]).0;
    m.w32(n + 0xc, 0x002c_f880);
    m.w32(n, 0);
    m.w32(n + 4, 0);
    n
}

/// FUN_00135eb0 (C2cb230 Bentley update tail): switch on +0x13d0 (< 15) through the jump table at
/// 0x2a4980: ground case (+0x208 surface, +0x88 > 0.7, owner +0x4c +0x148 bit 29 clear, +0x108c == 0)
/// -> FUN_0019feb0(+0x9e0, +0x208 +0x40, +0x208 +0x60); FUN_001942f0(+0x4c0); FUN_0019ea60(+0xf50)
pub fn ee_sub_135eb0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let s = m.r32(o + 0x13d0);
    if s >= 0xf {
        return;
    }
    match m.r32(0x002a_4980 + s.wrapping_mul(4)) {
        0x0013_5ee4 => {
            let g = m.r32(o + 0x208);
            if g == 0 || !(f32::from_bits(0x3f33_3333) < m.rf(g + 0x88)) {
                return;
            }
            let w = m.r32(g + 0x4c);
            if m.r64(w + 0x148) & 0x2000_0000 != 0 || m.r32(o + 0x108c) != 0 {
                return;
            }
            let q = m.rv(g + 0x60);
            let b = m.r32(g + 0x40);
            m.call(0x0019_feb0, &[o + 0x9e0, b], &[q]);
        }
        0x0013_5f50 => {
            m.call(0x0019_42f0, &[o + 0x4c0], &[]);
        }
        0x0013_5f60 => {
            m.call(0x0019_ea60, &[o + 0xf50], &[]);
        }
        _ => {}
    }
}

/// FUN_00134d30 (C2cb230): when FUN_001443d8 holds: with +0xf04 == 1, +0xf20 clear and a JoyTakePressed
/// (+0x150, 0x4f0, 0.25) hit, FUN_0019cce8(+0xf00); then FUN_0019d330(+0xf00)
pub fn ee_sub_134d30<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.call(0x0014_43d8, &[o], &[]).0 == 0 {
        return;
    }
    let s1 = o + 0xf00;
    if m.r32(o + 0xf04) == 1 && m.r32(o + 0xf20) == 0 {
        let j = m.r32(o + 0x150);
        if m.call(0x001c_46c0, &[j, 0x4f0], &[[0.25, 0.0, 0.0, 0.0]]).0 != 0 {
            m.call(0x0019_cce8, &[s1], &[]);
        }
    }
    m.call(0x0019_d330, &[s1], &[]);
}

/// FUN_001c3d70: FUN_001f2108, +0x15c = 1, +0x168 = o, then the +0x40 dirty mark (as FUN_001a5c48):
/// tagged (bit 0) -> +0x40 |= 2, else *(+0x40) |= 2
pub fn ee_sub_1c3d70<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001f_2108, &[o], &[]);
    let p = m.r32(o + 0x40);
    m.w32(o + 0x15c, 1);
    m.w32(o + 0x168, o);
    if p & 1 == 0 {
        let v = m.r32(p);
        m.w32(p, v | 2);
    } else {
        m.w32(o + 0x40, p | 2);
    }
}

/// FUN_001c4660 (JoyPressedWithin): bit i of `mask` set and now (DAT_002dded0) - j+0x88[i] <= t
pub fn ee_sub_1c4660<M: QMem + ?Sized>(m: &mut M, j: u32, mask: u32, t: f32) -> u32 {
    let now = m.rf(0x002d_ded0);
    let (mut r, mut k, mut p, mut b) = (0u32, 0u32, j + 0x88, mask);
    while b != 0 {
        if b & 1 != 0 && now - m.rf(p) <= t {
            r |= 1 << k;
        }
        b >>= 1;
        p += 4;
        k += 1;
    }
    r
}

/// FUN_001c4738 (JoyConsume): j+0x88[i] = DAT_002a76f0 for each set bit of `mask`
pub fn ee_sub_1c4738<M: QMem + ?Sized>(m: &mut M, j: u32, mask: u32) {
    let mut b = mask as i32;
    let mut p = j + 0x88;
    let f = m.rf(0x002a_76f0);
    while b != 0 {
        if b & 1 != 0 {
            m.wf(p, f);
        }
        b >>= 1;
        p += 4;
    }
}

/// FUN_001709f8: base init FUN_001a5270, then the +0x50 list init (FUN_0011a050, link offset 0x18)
pub fn ee_sub_1709f8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001a_5270, &[o], &[]);
    m.call(0x0011_a050, &[o + 0x50, 0x18], &[]);
}

/// FUN_00209a68: FUN_002099f0(+0x2c0)
pub fn ee_sub_209a68<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let a = m.r32(o + 0x2c0);
    m.call(0x0020_99f0, &[a], &[]);
}

/// FUN_0012f938: the 0x10-byte entry of +0xa4 (+0xa0 count) with kind (+4) 0xb and +8 == k, else 0
pub fn ee_sub_12f938<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32) -> u32 {
    let n = m.r32(o + 0xa0) as i32;
    let mut e = m.r32(o + 0xa4);
    for _ in 0..n.max(0) {
        if m.r32(e + 4) == 0xb && m.r32(e + 8) == k {
            return e;
        }
        e += 0x10;
    }
    0
}

/// FUN_001f1350 (anim rt set mask): +0x48 = mask; with +0x30 set and *(+0x40)+0x10 & mask:
/// FUN_001f08d0(o) then FUN_001f0d88(o, 0)
pub fn ee_sub_1f1350<M: QMem + ?Sized>(m: &mut M, o: u32, mask: u32) {
    m.w32(o + 0x48, mask);
    if m.r32(o + 0x30) == 0 {
        return;
    }
    let p = m.r32(o + 0x40);
    if m.r32(p + 0x10) & mask == 0 {
        return;
    }
    m.call(0x001f_08d0, &[o], &[]);
    m.call(0x001f_0d88, &[o, 0], &[]);
}

/// FUN_001f2108 (C2d9978 init): FUN_001a5270, kind 0 (FUN_001a6ca8), +0x64 list (FUN_0011a1f8, 0x24),
/// +0x148 / +0x149 = -10 / -20, +0x12c = 1.0, +0x13c = DAT_002c3b90, +0x140 = -1 and +0x148 bit 35 =
/// the parent's bit 35 (1 when parentless)
pub fn ee_sub_1f2108<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001a_5270, &[o], &[]);
    m.call(0x001a_6ca8, &[o, 0], &[]);
    m.call(0x0011_a1f8, &[o + 0x64, 0x24], &[]);
    m.w8(o + 0x148, (-10i8) as u8);
    m.w8(o + 0x149, (-20i8) as u8);
    let f = m.rf(0x002c_3b90);
    m.wf(o + 0x12c, 1.0);
    m.wf(o + 0x13c, f);
    let p = m.r32(o + 0x10);
    let b = if p != 0 { (m.r64(p + 0x148) >> 35) & 1 } else { 1 };
    let v = (m.r64(o + 0x148) & 0xffff_fff7_ffff_ffff) | b << 35;
    m.w32(o + 0x140, u32::MAX);
    m.w32(o + 0x148, v as u32);
    m.w32(o + 0x14c, (v >> 32) as u32);
}

/// FUN_00203a38 (guard-base post-load): FUN_001f36a8, +0x294 = FUN_001e94b0(0x2c9, 0x10),
/// FUN_002303a0(o, +0x58 / +0x5c, 1); with +0x264 != -1: +0x268 = FUN_001f6030(o, +0x264) and when
/// found FUN_00203af0(o, it) / FUN_001f2ad0(o, +0x268); then FUN_001eec70(+0x26c)
pub fn ee_sub_203a38<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001f_36a8, &[o], &[]);
    let r = m.call(0x001e_94b0, &[0x2c9, 0x10], &[]).0;
    let a = m.r32(o + 0x58);
    m.w32(o + 0x294, r);
    m.call(0x0023_03a0, &[o, a, 1], &[]);
    let a = m.r32(o + 0x5c);
    m.call(0x0023_03a0, &[o, a, 1], &[]);
    let k = m.r32(o + 0x264);
    if k != u32::MAX {
        let v = m.call(0x001f_6030, &[o, k], &[]).0;
        m.w32(o + 0x268, v);
        if v != 0 {
            m.call(0x0020_3af0, &[o, v], &[]);
            let v = m.r32(o + 0x268);
            m.call(0x001f_2ad0, &[o, v], &[]);
        }
    }
    m.call(0x001e_ec70, &[o + 0x26c], &[]);
}

/// FUN_00231100 (C2c9a80 init): FUN_001a5270, +0x50 = parent +0x60 when parented, the +0x60 4x4 cleared
/// (FUN_002333b4 0x40) to identity, FUN_002313f0
pub fn ee_sub_231100<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001a_5270, &[o], &[]);
    let p = m.r32(o + 0x10);
    if p != 0 {
        let v = m.r32(p + 0x60);
        m.w32(o + 0x50, v);
    }
    let s = o + 0x60;
    m.call(0x0023_33b4, &[s, 0, 0x40], &[]);
    for d in [0x14, 0x3c, 0x28, 0] {
        m.wf(s + d, 1.0);
    }
    m.call(0x0023_13f0, &[o], &[]);
}

/// FUN_002333b4 (memset): `n` bytes of `p` = `c` (the 0x10 / 8-byte block stores of the original are
/// the same bytes); returns `p`
pub fn ee_sub_2333b4<M: QMem + ?Sized>(m: &mut M, p: u32, c: u32, n: u32) -> u32 {
    for i in 0..n {
        m.w8(p + i, c as u8);
    }
    p
}

/// FUN_00207288 (guard state timeout): +0xfbc - 8 (< 0x1e) through the jump table at 0x2a87f0; the
/// three timed states end (FUN_00207030(o, -1)) once now - +0xfc0 > DAT_002c3d40 / 3d44 / 3d48
pub fn ee_sub_207288<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let i = m.r32(o + 0xfbc).wrapping_sub(8);
    if i >= 0x1e {
        return;
    }
    let lim = match m.r32(0x002a_87f0 + i.wrapping_mul(4)) {
        0x0020_72c0 => 0x002c_3d40,
        0x0020_72d8 => 0x002c_3d44,
        0x0020_7310 => 0x002c_3d48,
        _ => return,
    };
    let t = m.rf(o + 0xfc0);
    let now = m.rf(0x002d_ded0);
    if m.rf(lim) < now - t {
        m.call(0x0020_7030, &[o, u32::MAX], &[]);
    }
}
