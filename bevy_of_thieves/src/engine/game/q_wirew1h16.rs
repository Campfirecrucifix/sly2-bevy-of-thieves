//! WIRE-W1H16: 1:1 queue ports (against `QMem`) of a batch of live-frontier natives, ported from the EE asm;
//! callees go through `m.call` / `vslot`. QMem convention: integer / pointer args in `a`, quadword and float
//! args in `v`.
#![allow(unused_variables, unused_mut, unused_assignments, clippy::all)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

/// FUN_001dffd8: return p[0x74].
pub fn ee_sub_1dffd8<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    m.r32(a0 + 0x74)
}

/// FUN_001a5ee0: empty.
pub fn ee_sub_1a5ee0<M: QMem + ?Sized>(m: &mut M) {}

/// FUN_002599c8: return 0x2bad68.
pub fn ee_sub_2599c8<M: QMem + ?Sized>(m: &mut M) -> u32 {
    0x002b_ad68
}

/// FUN_001b9b60: return 0x2a75e0.
pub fn ee_sub_1b9b60<M: QMem + ?Sized>(m: &mut M) -> u32 {
    0x002a_75e0
}

/// FUN_001a16a8(p): FUN_0019f590(*p, 0).
pub fn ee_sub_1a16a8<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let x = m.r32(a0);
    m.call(0x0019_f590, &[x, 0], &[]).0
}

/// FUN_001f03a8: FUN_001f01b8(a0..t0, &local); return local.
pub fn ee_sub_1f03a8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, t0: u32) -> u32 {
    let s = stack_push(m, 0x10);
    m.call(0x001f_01b8, &[a0, a1, a2, a3, t0, s], &[]);
    let r = m.r32(s);
    stack_pop(m, 0x10);
    r
}

/// FUN_00208920(p, x): append x to p[0x3ac..] if count p[0x3a8] < 5.
pub fn ee_sub_208920<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let n = m.r32(a0 + 0x3a8);
    if n < 5 {
        m.w32(a0.wrapping_add(n << 2) + 0x3ac, a1);
        m.w32(a0 + 0x3a8, n.wrapping_add(1));
    }
}

/// FUN_0023df70(a0, a1, a2, a3): FUN_0023d338(&{a0, a1, a2, (u64)a3}).
pub fn ee_sub_23df70<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32) -> u32 {
    let s = stack_push(m, 0x20);
    m.w32(s, a0);
    m.w32(s + 4, a1);
    m.w32(s + 8, a2);
    m.w32(s + 0x10, a3);
    m.w32(s + 0x14, ((a3 as i32) >> 31) as u32);
    let r = m.call(0x0023_d338, &[s], &[]).0;
    stack_pop(m, 0x20);
    r
}

/// FUN_001fe9c8(p, x): if p[0x104] != x { p[0x104] = x; FUN_00232250(x) }.
pub fn ee_sub_1fe9c8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if m.r32(a0 + 0x104) != a1 {
        m.w32(a0 + 0x104, a1);
        m.call(0x0023_2250, &[a1], &[]);
    }
}

/// FUN_001b8180(p): state p[0x13d0] in [0,2) or [9,11).
pub fn ee_sub_1b8180<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let v = m.r32(a0 + 0x13d0) as i32;
    if v < 0 {
        return 0;
    }
    if v < 2 {
        return 1;
    }
    if v >= 11 || v < 9 {
        return 0;
    }
    1
}

/// FUN_001ade70(p): if p[0x24] then vslot 49 of it.
pub fn ee_sub_1ade70<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let o = m.r32(a0 + 0x24);
    if o != 0 {
        vslot(m, o, 49, &[], &[]);
    }
}

/// FUN_001fe990(p, x): index of x in p[0x1d4..][..p[0x1d0]] or -1.
pub fn ee_sub_1fe990<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> i32 {
    let n = m.r32(a0 + 0x1d0) as i32;
    let mut i = 0i32;
    let mut p = a0 + 0x1d4;
    if n <= 0 {
        return -1;
    }
    loop {
        if m.r32(p) == a1 {
            return i;
        }
        i += 1;
        p = p.wrapping_add(4);
        if i >= n {
            return -1;
        }
    }
}

/// FUN_001fb2c0(p, f): if p[0x18] != f { p[0x18] = f; FUN_001fb658(p, 0x40, 0x40) }.
pub fn ee_sub_1fb2c0<M: QMem + ?Sized>(m: &mut M, a0: u32, f12: f32) {
    if f12 != m.rf(a0 + 0x18) {
        m.wf(a0 + 0x18, f12);
        m.call(0x001f_b658, &[a0, 0x40, 0x40], &[f4(f12)]);
    }
}

/// FUN_001fb4f0(p, near, far): far = near + 1 if far <= near; p[0x30] = near; p[0x34] = far.
pub fn ee_sub_1fb4f0<M: QMem + ?Sized>(m: &mut M, a0: u32, f12: f32, mut f13: f32) {
    if f13 <= f12 {
        f13 = f12 + 1.0;
    }
    m.wf(a0 + 0x34, f13);
    m.wf(a0 + 0x30, f12);
}

/// FUN_0022b050(p): p[0x54] = 0; p[0x58] = p[0] = -1; FUN_0022b090(p, 0); return p.
pub fn ee_sub_22b050<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    m.w32(a0 + 0x54, 0);
    m.w32(a0 + 0x58, 0xffff_ffff);
    m.w32(a0, 0xffff_ffff);
    m.call(0x0022_b090, &[a0, 0], &[]);
    a0
}

/// FUN_001a2bd0(x): DAT_002e1e54 = x, or (x == -1) *out1 of FUN_001a2c48(&l0, &l1).
pub fn ee_sub_1a2bd0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    if a0 == 0xffff_ffff {
        let s = stack_push(m, 0x10);
        m.call(0x001a_2c48, &[s, s | 4], &[]);
        let v = m.r32(s + 4);
        let x = m.r32(v);
        m.w32(0x002e_1e54, x);
        stack_pop(m, 0x10);
    } else {
        m.w32(0x002e_1e54, a0);
    }
}

/// FUN_00228bb0(p): FUN_001f3590(p); FUN_001f68a8(p, 1); p[0xe8]->{0xd4, 0xd8} = 1.0.
pub fn ee_sub_228bb0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001f_3590, &[a0], &[]);
    m.call(0x001f_68a8, &[a0, 1], &[]);
    let e = m.r32(a0 + 0xe8);
    m.wf(e + 0xd4, 1.0);
    m.wf(e + 0xd8, 1.0);
}

/// FUN_001f82b8(a0, a1, src): v = FUN_001f8278(a0, a1, src); v[0x10] = a1; copy 8 bytes src -> v+0x14.
pub fn ee_sub_1f82b8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) -> u32 {
    let v = m.call(0x001f_8278, &[a0, a1, a2], &[]).0;
    m.w32(v + 0x10, a1);
    if a2 != 0 {
        for k in 0..8 {
            let b = m.r8(a2 + k);
            m.w8(v + 0x14 + k, b);
        }
    }
    v
}

/// FUN_002330d0: FUN_00232838(0x4c, 0x1c, &{a0, a1, a2, a3, t0, t1, t2}, t3, stack0).
pub fn ee_sub_2330d0<M: QMem + ?Sized>(
    m: &mut M,
    a0: u32,
    a1: u32,
    a2: u32,
    a3: u32,
    t0: u32,
    t1: u32,
    t2: u32,
    t3: u32,
    st0: u32,
) -> u32 {
    let s = stack_push(m, 0x20);
    m.w32(s, a0);
    m.w32(s + 4, a1);
    m.w32(s + 8, a2);
    m.w32(s + 0xc, a3);
    m.w32(s + 0x10, t0);
    m.w32(s + 0x14, t1);
    m.w32(s + 0x18, t2);
    let r = m.call(0x0023_2838, &[0x4c, 0x1c, s, t3, st0], &[]).0;
    stack_pop(m, 0x20);
    r
}

/// FUN_001be1e8: drain the list at 0x3e1a4c: while !FUN_0011a3b0(l) { e = *l; FUN_0011a2e8(l, e); FUN_001bd040(e) }.
pub fn ee_sub_1be1e8<M: QMem + ?Sized>(m: &mut M) {
    let l = 0x003e_1a4c;
    while m.call(0x0011_a3b0, &[l], &[]).0 == 0 {
        let e = m.r32(l);
        m.call(0x0011_a2e8, &[l, e], &[]);
        m.call(0x001b_d040, &[e], &[]);
    }
}

/// FUN_002553e0(p): p[0] = 0; FUN_00255248(p, 0x2dd290); p[4] bit31 = DAT_002dd598; return p.
pub fn ee_sub_2553e0<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    m.w32(a0, 0);
    m.call(0x0025_5248, &[a0, 0x002d_d290], &[]);
    let x = m.r32(a0 + 4);
    let b = m.r32(0x002d_d598);
    m.w32(a0 + 4, (x & 0x7fff_ffff) | (b << 31));
    a0
}

/// FUN_00209e28(a0, x): o = FUN_00205458(a0, 0x9c); msg{6, .., [4]=x}; FUN_00176750(o, &msg).
pub fn ee_sub_209e28<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let o = m.call(0x0020_5458, &[a0, 0x9c], &[]).0;
    let s = stack_push(m, 0x70);
    m.call(0x0023_33b4, &[s, 0, 0x70], &[]);
    m.w32(s + 0x10, a1);
    m.w32(s, 6);
    m.call(0x0017_6750, &[o, s], &[]);
    stack_pop(m, 0x70);
}

/// FUN_00226310(p, q): v = FUN_001f4fc8(p, q); if p[0x1c0] && q[0x3c] == p[0x1c0][0x3c] then FUN_00134310(v, 1).
pub fn ee_sub_226310<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let v = m.call(0x001f_4fc8, &[a0, a1], &[]).0;
    let e = m.r32(a0 + 0x1c0);
    if e != 0 && m.r32(a1 + 0x3c) == m.r32(e + 0x3c) {
        return m.call(0x0013_4310, &[v, 1], &[]).0;
    }
    v
}

/// FUN_001940b0(p, st): set state p[0x2c]; reset timer; p[0x34] derived.
pub fn ee_sub_1940b0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if a1 == m.r32(a0 + 0x2c) {
        return;
    }
    m.w32(a0 + 0x2c, a1);
    let f = m.rf(0x002d_ded0);
    m.w32(a0 + 0x50, 0);
    m.wf(a0 + 0x30, f);
    let s = a1 as i32;
    if s <= 0 {
        return;
    }
    if s < 3 {
        let g = m.r32(a0);
        let x = m.r32(a0 + 0x68);
        if m.r32(g + 0x1024) != x {
            m.w32(a0 + 0x34, 0);
        } else {
            m.w32(a0 + 0x34, 2);
        }
        return;
    }
    if s == 3 {
        m.w32(a0 + 0x34, 1);
    }
}

/// FUN_002242e0(p, q): v = FUN_001f4fc8(p, q); if p[0x150] in {0, 4} && FUN_00132b10(q, 0x34) then FUN_00134310(v, 1).
pub fn ee_sub_2242e0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let mut v = m.call(0x001f_4fc8, &[a0, a1], &[]).0;
    let t = m.r32(a0 + 0x150);
    if t != 0 && t != 4 {
        return v;
    }
    if m.call(0x0013_2b10, &[a1, 0x34], &[]).0 != 0 {
        v = m.call(0x0013_4310, &[v, 1], &[]).0;
    }
    v
}

/// FUN_00162f60(p): pickup/collect sequence.
pub fn ee_sub_162f60<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    if m.r32(a0 + 0x78) == 0 {
        let h = m.r32(a0 + 0x68);
        m.call(0x0016_4df0, &[h, 3], &[]);
    }
    m.call(0x0016_19a0, &[0x002d_eb30, 0x20, a0], &[]);
    m.call(0x0016_2908, &[a0], &[]);
    if m.r32(a0 + 0x78) == 0 {
        m.call(0x001a_bd98, &[0x002e_65e0, 0], &[]);
    }
}

/// FUN_001c8110: scoped lookup through FUN_001c8090 / FUN_002581c0; returns the found word or 0.
pub fn ee_sub_1c8110<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let s = stack_push(m, 0x20);
    m.call(0x001a_e478, &[s], &[]);
    let v = m.call(0x001c_8090, &[], &[]).0;
    let r;
    if v != 0 {
        m.call(0x0025_81c0, &[s + 0x10, v, 0x003d_4a60], &[]);
        r = m.r32(s + 0x14);
        m.call(0x0012_9ec8, &[s + 0x10], &[]);
        m.call(0x001a_e4a0, &[s, 2], &[]);
    } else {
        m.call(0x001a_e4a0, &[s, 2], &[]);
        r = 0;
    }
    stack_pop(m, 0x20);
    r
}

/// FUN_001f1f38(p, on): FUN_00205d10(p, on); if p[0x494] then vslot 20(on != 0).
pub fn ee_sub_1f1f38<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.call(0x0020_5d10, &[a0, a1], &[]);
    let o = m.r32(a0 + 0x494);
    if o != 0 {
        vslot(m, o, 20, &[(a1 != 0) as u32], &[]);
    }
}

/// FUN_001441e8(p, dt): accumulate p[0x1308] += dt * k; whole units go to FUN_001441c0(p, max(FUN_001441a0(p) - n, 0)).
pub fn ee_sub_1441e8<M: QMem + ?Sized>(m: &mut M, a0: u32, f12: f32) {
    let f12 = f12 * m.rf(0x002d_ded4);
    let f1 = m.rf(a0 + 0x1308) + f12;
    let n = f1 as i32;
    m.wf(a0 + 0x1308, f1);
    if n != 0 {
        m.wf(a0 + 0x1308, f1 - n as f32);
        let v = m.call(0x0014_41a0, &[a0], &[f4(f12)]).0;
        let d = v.wrapping_sub(n as u32) as i32;
        let a1 = if -1 < d { d as u32 } else { 0 };
        m.call(0x0014_41c0, &[a0, a1], &[]);
    }
}

/// FUN_00226370(p, on): toggles the p[0x1d8] effect handle via FUN_0012dc18 / FUN_0012e460.
pub fn ee_sub_226370<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if m.r32(a0 + 0x1cc) == a1 {
        return;
    }
    m.w32(a0 + 0x1cc, a1);
    if a1 != 0 {
        let f1 = -m.rf(a0 + 0x190);
        let f2 = m.rf(a0 + 0x18c);
        let hi = ((f2.to_bits() as i32) >> 31) as u32;
        let q: V4 = [0.0, f1, f2, f32::from_bits(hi)];
        let s = stack_push(m, 0x10);
        m.wv(s, q);
        let x = m.r32(a0 + 0x5c);
        let v = m.call(0x0012_dc18, &[a0, x, 1, 5, 3], &[q]).0;
        stack_pop(m, 0x10);
        m.w32(a0 + 0x1d8, v);
    } else {
        m.call(0x0012_e460, &[a0 + 0x1d8], &[]);
    }
}

/// FUN_0020a378(p, q, x): 1 if q has 0x36 and p lacks 0x3a/0x3d and p != DAT_002dd648; else FUN_0020a298(p, q, x).
pub fn ee_sub_20a378<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) -> u32 {
    if m.call(0x0013_2b10, &[a1, 0x36], &[]).0 != 0
        && m.call(0x0013_2b10, &[a0, 0x3a], &[]).0 == 0
        && m.call(0x0013_2b10, &[a0, 0x3d], &[]).0 == 0
        && a0 != m.r32(0x002d_d648)
    {
        return 1;
    }
    m.call(0x0020_a298, &[a0, a1, a2], &[]).0
}

/// FUN_002158c0(p, q): per-kind release of p (4: FUN_00218660; 6: FUN_001bed18 on p[0x14][0x20]); FUN_002156b8(q).
pub fn ee_sub_2158c0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let t = m.r32(a0.wrapping_add(4));
    if t == 4 {
        let i = m.r32(a0 + 0x10);
        let a2 = m.r32(a0 + 8);
        let x = m.r32(a1 + 0x58);
        let b = m.r32(a2 + 0x84);
        m.call(0x0021_8660, &[b.wrapping_add(i.wrapping_mul(0x30)), x], &[]);
    } else if t == 6 {
        let p = m.r32(a0 + 0x14);
        if p != 0 {
            let q = m.r32(p + 0x20);
            if q != 0 {
                m.call(0x001b_ed18, &[q], &[]);
            }
            m.w32(a0 + 0x14, 0);
        }
    }
    m.call(0x0021_56b8, &[a1], &[]);
}

/// FUN_00201f90(p): p[0xb8] = any FUN_00227c90(e, p.q[0x20]) over FUN_0015e9c8(GS, 0x505, 0x19, 0, 16, buf).
pub fn ee_sub_201f90<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.w32(a0 + 0xb8, 0);
    let s = stack_push(m, 0x40);
    let n = m.call(0x0015_e9c8, &[0x003d_d490, 0x505, 0x19, 0, 0x10, s], &[]).0 as i32;
    let mut i = 0i32;
    let mut p = s;
    while i < n {
        let e = m.r32(p);
        let q = m.rv(a0 + 0x20);
        if m.call(0x0022_7c90, &[e], &[q]).0 != 0 {
            m.w32(a0 + 0xb8, 1);
            break;
        }
        p = p.wrapping_add(4);
        i += 1;
    }
    stack_pop(m, 0x40);
}

/// FUN_0019f170(p, on): p[0x4748] = on; p[0x474c] = DAT_002dded0; FUN_00207030 over the 0x105/0x38 set.
pub fn ee_sub_19f170<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.w32(a0 + 0x4748, a1);
    let f = m.rf(0x002d_ded0);
    m.wf(a0 + 0x474c, f);
    let s = stack_push(m, 0x80);
    let mut n = m.call(0x0015_e9c8, &[a0, 0x105, 0x38, 0, 0x20, s], &[]).0;
    if (n as i32) > 0 {
        let mut p = s;
        loop {
            let e = m.r32(p);
            if a1 != 0 {
                m.call(0x0020_7030, &[e, 0xe], &[]);
            } else if m.r32(e + 0xfbc) == 0xe {
                m.call(0x0020_7030, &[e, 0xffff_ffff], &[]);
            }
            p = p.wrapping_add(4);
            n = n.wrapping_sub(1);
            if n == 0 {
                break;
            }
        }
    }
    stack_pop(m, 0x80);
}

/// FUN_00242b78: IOP rpc 0x16 on 0x2d82c0 into 0x2c7940; returns its first word (0 on failure).
pub fn ee_sub_242b78<M: QMem + ?Sized>(m: &mut M) -> u32 {
    if m.call(0x0024_1f98, &[0x1e], &[]).0 == 0 {
        return 0;
    }
    let s1 = 0x002c_7940u32;
    m.w32(0x002c_67a0, 8);
    let r = m.call(0x0023_7288, &[0x002d_82c0, 0x16, 0, 0, 0, s1, 4, 0, 0], &[]).0 as i32;
    if r < 0 {
        let h = m.r32(0x002c_676c);
        m.call(0x0023_4a40, &[h], &[]);
        m.w32(0x002c_67a0, 0);
        return 0;
    }
    m.w32(0x002c_67a0, 0);
    let h = m.r32(0x002c_676c);
    // asm reads through the uncached mirror (s1 | 0x20000000): same physical word
    let v = m.r32(s1);
    m.call(0x0023_4a40, &[h], &[]);
    v
}

/// FUN_001b3a48(list, e): append 64-byte e to list (max 4) unless e[4] == e[0x3c].
pub fn ee_sub_1b3a48<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if a1 == 0 || m.r32(a1 + 4) == m.r32(a1 + 0x3c) {
        return;
    }
    let n = m.r32(a0);
    if n >= 4 {
        return;
    }
    let d = (n << 6).wrapping_add(a0) + 4;
    for k in 0..0x40 {
        let b = m.r8(a1 + k);
        m.w8(d + k, b);
    }
    m.w32(a0, n + 1);
}

/// FUN_00206380(p, q): v = FUN_001f4fc8(p, q), optionally FUN_00134310(v, 1) by flag 7 / 0x17 checks.
pub fn ee_sub_206380<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let v = m.call(0x001f_4fc8, &[a0, a1], &[]).0;
    if m.call(0x0020_9a68, &[a0, 7], &[]).0 != 0 {
        if vslot(m, a1, 61, &[], &[]).0 != 0 {
            return v;
        }
        return m.call(0x0013_4310, &[v, 1], &[]).0;
    }
    if m.call(0x0020_9a68, &[a0, 0x17], &[]).0 == 0 {
        return v;
    }
    let a2 = m.call(0x001e_eee0, &[a0 + 0x26c, 0xb1], &[]).0;
    let o = m.r32(a2 + 0x5c);
    if m.r32(a1 + 0xec) != m.r32(o + 0xec) {
        return v;
    }
    let st = m.r32(a2 + 0x50) as i32;
    if st != 0 {
        if st < 0 || st >= 6 || st < 4 {
            return v;
        }
    }
    m.call(0x0013_4310, &[v, 1], &[]).0
}

/// FUN_001de2d8(p, q): k = 1, halved-ish via FUN_00134310(k, 2) per p[0x154..] hit (FUN_001a6290) and per
/// p[0x174..] flag (FUN_00132b10).
pub fn ee_sub_1de2d8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let mut k = 1u32;
    if (m.r32(a0 + 0x150) as i32) > 0 {
        let mut i = 0i32;
        let mut p = a0 + 0x154;
        loop {
            let e = m.r32(p);
            if m.call(0x001a_6290, &[a1, e], &[]).0 != 0 {
                k = m.call(0x0013_4310, &[k, 2], &[]).0;
            }
            i += 1;
            p = p.wrapping_add(4);
            if i >= m.r32(a0 + 0x150) as i32 {
                break;
            }
        }
    }
    if (m.r32(a0 + 0x170) as i32) > 0 {
        let mut i = 0i32;
        let mut p = a0 + 0x174;
        loop {
            let e = m.r32(p);
            if m.call(0x0013_2b10, &[a1, e], &[]).0 != 0 {
                k = m.call(0x0013_4310, &[k, 2], &[]).0;
            }
            i += 1;
            p = p.wrapping_add(4);
            if i >= m.r32(a0 + 0x170) as i32 {
                break;
            }
        }
    }
    k
}

/// FUN_0011efb0(v (128-bit a0), out_yaw, out_pitch, out_len): yaw = FUN_0011cf60(FUN_002458f0(v.y, v.x)),
/// pitch = FUN_0011cf60(FUN_002458f0(v.z, |v.xy|)), len = |v.xyz| (each optional).
pub fn ee_sub_11efb0<M: QMem + ?Sized>(m: &mut M, a1: u32, a2: u32, a3: u32, v: V4) {
    if a1 != 0 {
        let f0 = m.call(0x0024_58f0, &[], &[f4(v[1]), f4(v[0])]).1[0];
        let f0 = m.call(0x0011_cf60, &[], &[f4(f0)]).1[0];
        m.wf(a1, f0);
    }
    if a2 != 0 {
        let l = (v[0] * v[0] + v[1] * v[1]).abs().sqrt();
        let f0 = m.call(0x0024_58f0, &[], &[f4(v[2]), f4(l)]).1[0];
        let f0 = m.call(0x0011_cf60, &[], &[f4(f0)]).1[0];
        m.wf(a2, f0);
    }
    if a3 != 0 {
        let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).abs().sqrt();
        m.wf(a3, l);
    }
}

/// FUN_00149100(p, q): (p[0x2ac] == 0 or q in p[0x2b0..]) and (p[0x2d0] == 0 or any FUN_00132b10(q, p[0x2d4..])).
pub fn ee_sub_149100<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let n = m.r32(a0 + 0x2ac) as i32;
    if n > 0 {
        let mut i = 0i32;
        let mut p = a0 + 0x2b0;
        let mut found = false;
        loop {
            if m.r32(p) == a1 {
                found = true;
                break;
            }
            i += 1;
            p = p.wrapping_add(4);
            if i >= n {
                break;
            }
        }
        if !found || i >= n {
            return 0;
        }
    }
    if (m.r32(a0 + 0x2d0) as i32) <= 0 {
        return 1;
    }
    let mut j = 0i32;
    let mut p = a0 + 0x2d4;
    loop {
        if j >= m.r32(a0 + 0x2d0) as i32 {
            return 0;
        }
        let e = m.r32(p);
        p = p.wrapping_add(4);
        if m.call(0x0013_2b10, &[a1, e], &[]).0 != 0 {
            return (j < m.r32(a0 + 0x2d0) as i32) as u32;
        }
        j += 1;
    }
}

/// FUN_001b81b8(p, on): FUN_00205d10(p, on); attach / detach the held object p[0xf1c] (vslot 20, vslot 86).
pub fn ee_sub_1b81b8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.call(0x0020_5d10, &[a0, a1], &[]);
    if a1 == 0 {
        let o = m.r32(a0 + 0xf1c);
        if o == 0 {
            return;
        }
        vslot(m, o, 20, &[1], &[]);
        let o = m.r32(a0 + 0xf1c);
        if m.call(0x0013_2b10, &[o, 0x33], &[]).0 != 0 {
            let o = m.r32(a0 + 0xf1c);
            vslot(m, o, 86, &[0], &[]);
        }
        m.w32(a0 + 0xf1c, 0);
    } else {
        let p = m.r32(a0 + 0x13f0);
        if p == 0 {
            return;
        }
        let o = m.r32(p + 0x180);
        if o == 0 {
            return;
        }
        m.w32(a0 + 0xf1c, o);
        vslot(m, o, 20, &[0], &[]);
        let o = m.r32(a0 + 0xf1c);
        if m.call(0x0013_2b10, &[o, 0x33], &[]).0 != 0 {
            let o = m.r32(a0 + 0xf1c);
            vslot(m, o, 86, &[a1], &[]);
        }
    }
}

/// FUN_00221298(p): track the player (FUN_001c4a48(GS)) in p[0xe0]; FUN_00221178(p, visible).
pub fn ee_sub_221298<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let s1 = m.call(0x001c_4a48, &[0x003d_d490], &[]).0;
    if s1 == 0 {
        return;
    }
    if s1 != m.r32(a0 + 0xe0) {
        m.call(0x0022_1178, &[a0, 0], &[]);
        m.w32(a0 + 0xe0, s1);
    }
    let a1 = m.r32(a0 + 0x13c);
    let mut s2 = a1;
    if a1 == 0xffff_ffff {
        let v = m.call(0x001a_62e8, &[s1], &[]).1;
        let v = m.call(0x001a_6f00, &[0, a0], &[v]).1;
        let x = m.r32(a0 + 0x108);
        s2 = (m.call(0x0011_8fb0, &[x, 0x7fff_ffff], &[v]).0 != 0) as u32;
    }
    if m.r32(a0 + 0x124) == 0
        && m.call(0x0013_2b10, &[s1, 0x34], &[]).0 != 0
        && m.r32(s1 + 0x2dc) != 0xffff_ffff
        && m.r32(0x002d_e240) == 0
        && m.r32(a0 + 0x13c) == 0xffff_ffff
    {
        return;
    }
    m.call(0x0022_1178, &[a0, s2], &[]);
}

/// FUN_001def98(p, src[2] (stride 0xa0), out[2] quads, out_dist): resolve two endpoints (kind 0: FUN_0022fc48,
/// kind 1: FUN_001e6f10); out_dist = |out[0] - out[1]|.
pub fn ee_sub_1def98<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32) {
    let mut s0 = a1;
    let mut s1 = a2;
    for _ in 0..2 {
        let k = m.r32(s0 + 4);
        if k == 0 {
            let x = m.r32(a0 + 0x60);
            let q = m.rv(s0 + 0x10);
            let v = m.call(0x0022_fc48, &[x, 0], &[q]).1;
            m.wv(s1, v);
        } else if k == 1 {
            let x = m.r32(s0 + 0x10);
            let f = m.rf(s0 + 0x14);
            m.call(0x001e_6f10, &[x, s1, 0], &[f4(f)]);
        }
        s1 = s1.wrapping_add(0x10);
        s0 = s0.wrapping_add(0xa0);
    }
    if a3 != 0 {
        let p = m.rv(a2);
        let q = m.rv(a2 + 0x10);
        let d = [p[0] - q[0], p[1] - q[1], p[2] - q[2]];
        let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).abs().sqrt();
        m.wf(a3, l);
    }
}

/// FUN_0011d1c0(curve, out, f12, f13 (v), f14, f15 (dt)): FUN_0011d018 then clamp the value by +-c*dt.
pub fn ee_sub_11d1c0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, f12: f32, f13: f32, f14: f32, f15: f32) -> f32 {
    let s = stack_push(m, 0x10);
    let mut f1 = m.call(0x0011_d018, &[a0, s], &[f4(f12), f4(f14), f4(f15)]).1[0];
    let f4v = m.rf(a0 + 0xc);
    let f3 = m.rf(s);
    if 0.0 < f4v {
        let f0 = f4v * f15;
        let f2 = f13 + f0;
        let f5 = f13 - f0;
        if f2 < f3 {
            m.wf(s, f2);
            f1 = (f12 + f13 * f15) + f4v * (f15 * 0.5) * f15;
        } else if f3 < f5 {
            m.wf(s, f5);
            f1 = (f12 + f13 * f15) - f4v * (f15 * 0.5) * f15;
        }
    }
    if a1 != 0 {
        let f = m.rf(s);
        m.wf(a1, f);
    }
    stack_pop(m, 0x10);
    f1
}

/// FUN_0019de58(p): for both p[0x50]/p[0x60] slots, sample the curve (vt @ +0 slot 0x1c) at 0 and len,
/// slot[0xc] = (y0 - y1) / len; then FUN_00137a30(p[0], 0x10, 2, p+0x50, 0, 0, 0, p+0x70).
pub fn ee_sub_19de58<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let s = stack_push(m, 0x20);
    let mut s2 = a0 + 0x50;
    let mut s4 = 1i32;
    loop {
        let s1 = m.r32(s2 + 4);
        if s1 != 0 {
            let x = m.r32(a0);
            let s0 = m.call(0x0012_ff10, &[s1, x, 1, 1], &[]).0;
            if s0 != 0 {
                for (out, t) in [(s, None), (s + 0x10, Some(()))] {
                    let a3 = m.r32(s0 + 4);
                    let vt = m.r32(a3);
                    let adj = m.r16(vt + 0x18) as i16 as i32;
                    let f = m.r32(vt + 0x1c);
                    let f12 = if t.is_some() { m.rf(s1 + 0x74) } else { 0.0 };
                    m.call(f, &[a3.wrapping_add(adj as u32), out, 0], &[f4(f12), f4(1.0)]);
                }
                let r = (m.rf(s + 8) - m.rf(s + 0x18)) / m.rf(s1 + 0x74);
                m.wf(s2 + 0xc, r);
            }
        }
        s4 -= 1;
        s2 = s2.wrapping_add(0x10);
        if s4 < 0 {
            break;
        }
    }
    stack_pop(m, 0x20);
    let x = m.r32(a0);
    m.call(0x0013_7a30, &[x, 0x10, 2, a0 + 0x50, 0, 0, 0, a0 + 0x70], &[]);
}

/// FUN_002020c8(p): for each p[0x34..][..p[0x30]] with a target, show / hide by xy-distance to the camera.
pub fn ee_sub_2020c8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let mut i = 0i32;
    if (m.r32(a0 + 0x30) as i32) <= 0 {
        return;
    }
    let mut s1 = a0 + 0x34;
    loop {
        let s0 = m.r32(s1);
        if m.r32(s0 + 0x100) != 0 {
            let v = m.call(0x001a_62e8, &[s0], &[]).1;
            let f0 = m.rf(s0 + 0x104);
            let c = m.rv(0x002d_df40 + 0x90);
            let (dx, dy) = (v[0] - c[0], v[1] - c[1]);
            let f1 = (dx * dx + dy * dy).abs().sqrt();
            let t = m.r32(s0 + 0x100);
            let bit = (m.r64(t + 0x28) >> 60) & 1;
            if f0 < f1 && bit == 0 {
                m.call(0x001f_7c68, &[t], &[f4(0.25)]);
            } else if f1 <= f0 && bit != 0 {
                m.call(0x001f_7bf8, &[t], &[f4(0.25)]);
            }
        }
        i += 1;
        s1 = s1.wrapping_add(4);
        if i >= m.r32(a0 + 0x30) as i32 {
            break;
        }
    }
}

/// shared body of the identical static-initialiser thunks (`__static_initialization_and_destruction_0`):
/// when (init, 0xffff), bump each pool refcount at 0x2dd2e0..0x2dd2f4 and create the pool on 0 -> 1.
fn static_pools<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if a1 != 0xffff || a0 == 0 {
        return;
    }
    const POOLS: [(u32, u32, u32, u32); 6] = [
        (0x002d_d2e0, 0x0045_20a8, 0x14, 0x2000),
        (0x002d_d2e4, 0x0045_20d0, 0x18, 0x800),
        (0x002d_d2e8, 0x0045_20f8, 0x28, 0x200),
        (0x002d_d2ec, 0x0045_2120, 0x1c, 0x800),
        (0x002d_d2f0, 0x0045_2148, 0xc, 0xc8),
        (0x002d_d2f4, 0x0045_2170, 0x14, 0x100),
    ];
    for (cnt, pool, sz, n) in POOLS {
        let v = m.r32(cnt).wrapping_add(1);
        m.w32(cnt, v);
        if v == 1 {
            m.call(0x0011_cca8, &[pool, sz, n, 0x10], &[]);
        }
    }
}

/// FUN_0019dcb8: static pool init (see `static_pools`).
pub fn ee_sub_19dcb8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    static_pools(m, a0, a1)
}

/// FUN_001b26c8: static pool init (see `static_pools`).
pub fn ee_sub_1b26c8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    static_pools(m, a0, a1)
}

/// FUN_001ff538: static pool init (see `static_pools`).
pub fn ee_sub_1ff538<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    static_pools(m, a0, a1)
}

/// FUN_002263f8: static pool init (see `static_pools`).
pub fn ee_sub_2263f8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    static_pools(m, a0, a1)
}

/// FUN_0022aee8: static pool init (see `static_pools`).
pub fn ee_sub_22aee8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    static_pools(m, a0, a1)
}

fn dot3(a: V4, b: V4) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn scale4(a: V4, k: f32) -> V4 {
    [a[0] * k, a[1] * k, a[2] * k, a[3] * k]
}

/// FUN_001fa810(c, obj, v (128-bit a2)) -> quadword: constrain v by c (kind c[0x10]: 1 = remove the component
/// along the (obj-transformed) normal, 2 = keep only it, 3 = zero, else v).
pub fn ee_sub_1fa810<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, v: V4) -> V4 {
    let t = m.r32(a0 + 0x10) as i32;
    if t == 3 {
        return [0.0; 4];
    }
    if t != 1 && t != 2 {
        return v;
    }
    let a = m.rv(a0);
    let o = m.r32(a1 + 0xa0);
    let n = if o != 0 {
        let c0 = m.rv(o + 0x40);
        let c1 = m.rv(o + 0x50);
        let c2 = m.rv(o + 0x60);
        let mut n = a;
        for k in 0..3 {
            n[k] = c0[k] * a[0] + c1[k] * a[1] + c2[k] * a[2];
        }
        let l = dot3(n, n).abs().sqrt();
        scale4(n, 1.0 / l)
    } else {
        a
    };
    let p = scale4(n, dot3(v, n));
    if t == 2 {
        p
    } else {
        [v[0] - p[0], v[1] - p[1], v[2] - p[2], v[3] - p[3]]
    }
}

/// FUN_0020f7e0(p): ease the four p[0x3ba0..] (stride 0xc) slot widgets toward their stacked offsets;
/// slot 3 also refreshes the countdown digit icon (p[0x4770]).
pub fn ee_sub_20f7e0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let mut f20 = 0.0f32;
    let mut s1 = a0 + 0x3ba0;
    let mut s3 = a0 + 0x3ba4;
    for s4 in 0..4 {
        let s0 = m.r32(s1);
        if s0 == 0 || m.call(0x0021_32d0, &[s0], &[]).0 == 0 {
            m.w32(s1 + 8, 1);
        } else {
            if m.r32(s1 + 8) != 0 {
                let e = m.r32(s0 + 0xf8);
                m.wf(e + 0x14, f20);
                m.w32(s1 + 4, 0);
                m.w32(s1 + 8, 0);
            } else {
                let e = m.r32(s0 + 0xf8);
                let f12 = m.rf(e + 0x14);
                let f13 = m.rf(s1 + 4);
                let f15 = m.rf(0x002d_ded0 + 4);
                let f0 = m.call(0x0011_d1c0, &[0x002c_3e88, s3], &[f4(f12), f4(f13), f4(f20), f4(f15)]).1[0];
                let e = m.r32(s0 + 0xf8);
                m.wf(e + 0x14, f0);
            }
            f20 += m.rf(0x002c_3e80);
            if s4 == 3 {
                let f0 = m.rf(0x002d_ded0);
                let f1 = (m.rf(a0 + 0x476c) - f0).max(0.0);
                let n = f1 as i32 as u32;
                if n != m.r32(a0 + 0x4770) {
                    let id = if n & 1 != 0 { 0x30 } else { 0x31 };
                    let v = m.call(0x001f_bb90, &[id], &[]).0;
                    m.call(0x001f_b2f8, &[v, 0], &[]);
                    m.w32(a0 + 0x4770, n);
                }
            }
        }
        s3 = s3.wrapping_add(0xc);
        s1 = s1.wrapping_add(0xc);
    }
}

/// FUN_001acfe0(p): per-kind (p[0x40], jump table @0x2a6e10) completion handling.
pub fn ee_sub_1acfe0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let k = m.r32(a0 + 0x40);
    let s0 = 0x003d_9ed0u32;
    let s2 = 0x003d_a530u32;
    match k {
        17 | 18 | 21 | 24 => {
            if m.call(0x001d_9880, &[s2], &[]).0 != 0 && m.call(0x001b_0160, &[s0], &[]).0 != 0 {
                m.call(0x001a_cbb8, &[a0], &[]);
                m.call(0x001a_cc38, &[a0, 2], &[]);
                m.call(0x001a_f660, &[s0], &[]);
                m.call(0x001d_db08, &[s2], &[]);
                return;
            }
            ee_sub_1acfe0_b(m, a0);
        }
        20 | 23 | 29 | 35 | 36 | 46 => ee_sub_1acfe0_b(m, a0),
        27 | 28 | 32 => {
            m.call(0x001a_cbb8, &[a0], &[]);
        }
        34 => {
            let s0 = m.r32(0x003e_0fe4);
            let s = stack_push(m, 0x30);
            m.w32(s, 0x17);
            m.w32(s + 4, 1);
            m.call(0x0012_91a8, &[s + 0x10, s], &[]);
            m.call(0x0025_8200, &[s + 0x20, s0, s + 0x10, 0x003d_4a60], &[]);
            m.call(0x0012_9ec8, &[s + 0x20], &[]);
            m.call(0x0012_9ec8, &[s + 0x10], &[]);
            m.call(0x0012_9ec8, &[s], &[]);
            stack_pop(m, 0x30);
            let v = m.r32(a0 + 0x4c) as i32;
            if v < 4 && v > 0 {
                m.call(0x001a_cb58, &[a0, 0x1a], &[]);
                m.call(0x001a_bd40, &[0x002e_65e0], &[]);
            }
        }
        _ => {}
    }
}

/// FUN_001acfe0 case block @0x1ad07c.
fn ee_sub_1acfe0_b<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_cbb8, &[a0], &[]);
    m.call(0x001a_cc38, &[a0, 2], &[]);
    let s0 = 0x003d_9ed0;
    if m.call(0x001b_0108, &[s0], &[]).0 != 0 {
        m.call(0x001a_f608, &[s0], &[]);
    }
}

/// FUN_002267c0(p): pre pass (vslot 15 on 0x102 entries), FUN_00130918 over the p[0x3d88] list (guarded
/// iterator linked at DAT_002dd2fc), post pass (vslot 16 on 0x104/0x124 entries); p[0x3f1c] -> live index.
pub fn ee_sub_2267c0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let sp = stack_push(m, 0x20);
    let s3 = sp + 0x10;
    m.w32(a0 + 0x3f1c, s3);
    m.w32(sp + 0x10, 0);
    if (m.r32(a0 + 0x3f14) as i32) > 0 {
        let mut i = 0u32;
        loop {
            let e = m.r32(a0 + 0x3f20).wrapping_add(i << 3);
            if m.r32(e) & 0x102 == 0x102 {
                let o = m.r32(e + 4);
                vslot(m, o, 15, &[], &[]);
            }
            i = m.r32(sp + 0x10).wrapping_add(1);
            m.w32(sp + 0x10, i);
            if (i as i32) >= m.r32(a0 + 0x3f14) as i32 {
                break;
            }
        }
    }
    m.w32(a0 + 0x3f1c, 0);
    let l = a0 + 0x3d88;
    m.w32(sp + 4, l);
    m.w32(sp, l);
    let prev = m.r32(0x002d_d2fc);
    let off = m.r32(l + 8);
    m.w32(0x002d_d2fc, sp);
    m.w32(sp + 0xc, prev);
    let first = m.r32(l);
    m.w32(sp + 8, off);
    m.w32(sp + 4, first.wrapping_add(off));
    let mut a1 = first;
    while a1 != 0 {
        let d = m.r32(a1 + 0x14);
        let mut f = m.r32(d + 0x40);
        if (f ^ 1) & 1 != 0 {
            f = m.r32(f);
        }
        if f & 0x100 != 0 {
            m.call(0x0013_0918, &[a1], &[]);
        }
        let c = m.r32(sp + 4);
        let off = m.r32(sp + 8);
        let nx = m.r32(c);
        m.w32(sp + 4, nx.wrapping_add(off));
        a1 = nx;
    }
    m.w32(a0 + 0x3f1c, s3);
    m.w32(sp + 0x10, 0);
    if (m.r32(a0 + 0x3f14) as i32) > 0 {
        let mut i = 0u32;
        loop {
            let e = m.r32(a0 + 0x3f20).wrapping_add(i << 3);
            if m.r32(e) & 0x124 == 0x104 {
                let o = m.r32(e + 4);
                vslot(m, o, 16, &[], &[]);
            }
            i = m.r32(sp + 0x10).wrapping_add(1);
            m.w32(sp + 0x10, i);
            if (i as i32) >= m.r32(a0 + 0x3f14) as i32 {
                break;
            }
        }
    }
    let prev = m.r32(sp + 0xc);
    m.w32(a0 + 0x3f1c, 0);
    m.w32(0x002d_d2fc, prev);
    stack_pop(m, 0x20);
}

fn xf3(r0: V4, r1: V4, r2: V4, v: V4, w: f32) -> [f32; 3] {
    let mut o = [0.0; 3];
    for k in 0..3 {
        o[k] = r0[k] * v[0] + r1[k] * v[1] + r2[k] * v[2];
    }
    o
}

/// FUN_001e5fc8(p, seg): when seg != p[0x148], sample segment seg's Hermite curve (world space via p[0x100]
/// matrix) at 11 steps and store the cumulative arc lengths in p[0x14c..]; p[0x148] = seg.
pub fn ee_sub_1e5fc8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if a1 == m.r32(a0 + 0x148) {
        return;
    }
    let o = a1 << 4;
    let a2 = o.wrapping_add(m.r32(a0 + 0xf4));
    let v1 = o.wrapping_add(m.r32(a0 + 0x144));
    let v0 = o.wrapping_add(m.r32(a0 + 0x140));
    let r0 = m.rv(a0 + 0x100);
    let r1 = m.rv(a0 + 0x110);
    let r2 = m.rv(a0 + 0x120);
    let t = m.rv(a0 + 0x130);
    let q5 = m.rv(v0 + 0x10);
    let q4 = m.rv(a2);
    let q7 = m.rv(a2 + 0x10);
    let q8 = m.rv(v1);
    m.w32(a0 + 0x14c, 0);
    let ta = m.r32(a0 + 0xec).wrapping_add(a1 << 2);
    let f1 = m.rf(ta);
    let c = xf3(r0, r1, r2, q5, 0.0);
    let s50: V4 = [c[0], c[1], c[2], 0.0];
    let c = xf3(r0, r1, r2, q7, 1.0);
    let s30: V4 = [c[0] + t[0], c[1] + t[1], c[2] + t[2], q5[3]];
    let c = xf3(r0, r1, r2, q4, 1.0);
    let s20: V4 = [c[0] + t[0], c[1] + t[1], c[2] + t[2], 0.0];
    let c = xf3(r0, r1, r2, q8, 0.0);
    let s40: V4 = [c[0], c[1], c[2], q4[3]];
    let f20 = m.rf(ta + 4) - f1;
    let f21 = f32::from_bits(0x3dcc_cccd);
    let sp = stack_push(m, 0x20);
    let mut s0 = a0 + 0x14c;
    for s2 in 0..0xbi32 {
        let f13 = (s2 as f32) * f21 * f20;
        m.call(0x0011_7120, &[sp + 0x10, 0, 0], &[s20, s40, s30, s50, f4(f20), f4(f13), f4(1.0)]);
        let n = m.rv(sp + 0x10);
        if s2 > 0 {
            let p = m.rv(sp);
            let d = [p[0] - n[0], p[1] - n[1], p[2] - n[2], 0.0];
            let l = dot3(d, d).abs().sqrt();
            let f = m.rf(s0 - 4) + l;
            m.wf(s0, f);
        }
        m.wv(sp, n);
        s0 = s0.wrapping_add(4);
    }
    stack_pop(m, 0x20);
    m.w32(a0 + 0x148, a1);
}

fn mx3(c0: V4, c1: V4, c2: V4, v: V4, w: f32) -> V4 {
    let o = xf3(c0, c1, c2, v, 0.0);
    [o[0], o[1], o[2], w]
}

/// FUN_001d0630(p): orbit-camera step: basis from FUN_0011bc00, eased spin (FUN_0011b500 x2), compose with p's
/// local frame, then FUN_001cf338(p, FUN_0014f268(CM, p.q[0x100]), &mat). (unknown VU w lanes taken as 0)
pub fn ee_sub_1d0630<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let cm = 0x002d_df40u32;
    let q = m.rv(a0 + 0x100);
    let s4 = m.call(0x0014_f268, &[cm], &[q]).1;
    let sp = stack_push(m, 0x170);
    let s2 = sp + 0xd0;
    m.call(0x0011_bc00, &[s2, a0 + 0x40, cm + 0x60], &[]);
    let (x, y, z) = (m.rv(sp + 0xe0), m.rv(sp + 0xf0), m.rv(sp + 0xd0));
    m.wv(sp + 0x10, x);
    m.wv(sp + 0x20, y);
    m.wv(sp, z);
    m.call(0x0011_bcf8, &[sp, sp + 0x160, sp + 0xc0], &[]);
    let f0 = m.rf(a0 + 0x14);
    let mut f20 = m.rf(0x002d_ded0) - f0;
    let f1 = m.rf(a0 + 0x18);
    f20 = f20 / f1;
    let f12 = m.rf(sp + 0x160) * f20;
    let ax = m.rv(sp + 0xc0);
    m.call(0x0011_b500, &[s2], &[ax, f4(f12)]);
    let (a5, a4, a3) = (m.rv(sp + 0xf0), m.rv(sp + 0xe0), m.rv(sp + 0xd0));
    f20 = f20 + f20;
    m.wv(sp + 0x40, a4);
    m.wv(sp + 0x50, a5);
    m.wv(sp + 0x30, a3);
    let f12 = f20 * f32::from_bits(0x4049_0fdb);
    let o = m.r32(a0 + 0xf0);
    let ax = m.rv(o + 0x40);
    m.call(0x0011_b500, &[s2], &[ax, f4(f12)]);
    let (c3, c2, c1) = (m.rv(sp + 0x30), m.rv(sp + 0x40), m.rv(sp + 0x50));
    let v9 = m.rv(sp + 0xd0);
    let v7 = m.rv(sp + 0xf0);
    let v8 = m.rv(sp + 0xe0);
    let v5 = mx3(c3, c2, c1, v7, 0.0);
    let v4 = mx3(c3, c2, c1, v9, 0.0);
    let v3 = mx3(c3, c2, c1, v8, c3[3]);
    let l6 = m.rv(a0 + 0x60);
    let l1 = m.rv(a0 + 0x40);
    let l2 = m.rv(a0 + 0x50);
    let v6 = mx3(v4, v3, v5, l6, l6[3]);
    let v1 = mx3(v4, v3, v5, l1, l1[3]);
    let v2 = mx3(v4, v3, v5, l2, l2[3]);
    for (off, v) in [
        (0x90, v1),
        (0xa0, v2),
        (0xb0, v6),
        (0x60, v9),
        (0x70, v8),
        (0x80, v7),
        (0x100, v4),
        (0x110, v3),
        (0x120, v5),
        (0x130, v1),
        (0xd0, v1),
        (0x140, v2),
        (0xe0, v2),
        (0x150, v6),
        (0xf0, v6),
    ] {
        m.wv(sp + off, v);
    }
    m.call(0x001c_f338, &[a0, sp + 0x90], &[s4]);
    stack_pop(m, 0x170);
}

/// FUN_001c6308(p): when the facing side (vs camera) flips, maybe (random < p[0x1a0], cooldown) spawn an
/// effect (FUN_001ce888(GS, 4, 0)) at p[0x1b0] / p[0x1b8]; p[0x1bc] = side.
pub fn ee_sub_1c6308<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    if m.r32(a0 + 0x1a8) == 0 {
        return;
    }
    let f20 = 0.0f32;
    if m.rf(a0 + 0x1a0) <= f20 {
        return;
    }
    let cm = 0x002d_df40u32;
    let q = m.rv(0x002c_0d80);
    let v = m.call(0x001a_6fc0, &[a0, 0], &[q]).1;
    let c = m.rv(cm + 0x70);
    let s5 = (dot3(v, c) < f20) as u32;
    if s5 == m.r32(a0 + 0x1bc) {
        return;
    }
    let f0 = m.call(0x0011_d448, &[], &[f4(f20), f4(1.0)]).1[0];
    'out: {
        if !(f0 < m.rf(a0 + 0x1a0)) {
            break 'out;
        }
        let f2 = m.rf(a0 + 0x1c0);
        let now = m.rf(0x002d_ded0);
        if !(m.rf(0x002c_0d90) < now - f2) {
            break 'out;
        }
        let c = m.rv(cm + 0x60);
        let s4 = if dot3(v, c) < f20 { m.r32(a0 + 0x1b0) } else { m.r32(a0 + 0x1b8) };
        if s4 == 0 {
            break 'out;
        }
        let s2 = m.call(0x001c_e888, &[0x003d_d490, 4, 0], &[]).0;
        if s2 == 0 {
            break 'out;
        }
        let now = m.rf(0x002d_ded0);
        m.wf(a0 + 0x1c0, now);
        let vt = m.r32(s2 + 0xec);
        let adj = m.r16(vt + 8) as i16 as i32;
        let pos = m.call(0x001a_62e8, &[s4], &[]).1;
        let f = m.r32(vt + 0xc);
        m.call(f, &[s2.wrapping_add(adj as u32), 0], &[pos, f4(1.0)]);
        let x = m.r32(a0 + 0x1a8);
        m.w32(s2 + 0xf0, s4);
        m.w32(s2 + 0x1c, x);
        let o = m.r32(a0 + 0x5c);
        let z = m.rf(o + 0xb8);
        let f = 1.0f32.max(z.abs());
        m.wf(s2 + 0x18, 1.5 / f);
    }
    m.w32(a0 + 0x1bc, s5);
}

/// FUN_001feed0(p): when idle (p[0x64] == 0), roll the next taunt / reaction from the static tables
/// 0x2a82a8 / 0x2a8350 (FUN_0011d3f8(0, 5)) and post msg 0x13 (FUN_002322f0).
pub fn ee_sub_1feed0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    if m.r32(a0 + 0x64) != 0 {
        return;
    }
    let sp = stack_push(m, 0x150);
    for k in 0..0xa8 {
        let b = m.r8(0x002a_82a8 + k);
        m.w8(sp + k, b);
    }
    let s1 = sp + 0xb0;
    let s2 = sp + 0x120;
    for k in 0..0x64 {
        let b = m.r8(0x002a_8350 + k);
        m.w8(s1 + k, b);
    }
    let r = m.call(0x0011_d3f8, &[0, 5], &[]).0;
    let t0 = m.r32(a0 + 0x138);
    let a3 = s1.wrapping_add(t0.wrapping_add(0xc) << 2);
    let v1 = m.r32(a3);
    let off = (r << 2).wrapping_add(v1.wrapping_mul(0x18));
    let t0 = t0.wrapping_add(m.r32(sp.wrapping_add(off)));
    m.w32(a0 + 0x138, t0);
    m.call(0x0023_33b4, &[s2, 0, 0x30], &[]);
    let x = m.r32(a0 + 0xe8);
    m.w32(sp + 0x148, 0x13);
    m.w32(sp + 0x120, x);
    let v = m.call(0x001f_d028, &[a0, 0x28], &[]).0;
    let v = m.call(0x001f_d070, &[a0, v], &[]).0;
    let t = m.r32(a0 + 0x138);
    m.w16(sp + 0x130, v as u16);
    m.w16(sp + 0x140, (t << 7).wrapping_sub(t) as u16);
    m.call(0x0023_22f0, &[s2], &[]);
    stack_pop(m, 0x150);
}

fn copy_words<M: QMem + ?Sized>(m: &mut M, dst: u32, src: u32, n: u32) {
    for k in 0..n {
        let w = m.r32(src + 4 * k);
        m.w32(dst + 4 * k, w);
    }
}

/// FUN_00217058(o, mat, reg, flags): append a VIF/GIF packet (0x2dea28 builder): matrix (flags & 1: 4 qw,
/// else 1 qw of mat+0x20), optional 3-qw register block, DIRECT tag o[0x1c].
pub fn ee_sub_217058<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32) {
    let t3 = 0x002d_ea28u32;
    let t4 = m.r32(t3 + 0xc);
    m.w32(t4, 0x1000_0000);
    m.w32(t4 + 4, 0);
    let p = m.r32(t3 + 0xc);
    m.w32(t3 + 0xc, p + 0x10);
    if a3 & 1 != 0 {
        m.w32(p + 0x10, 0x6c04_c000);
        let q = m.r32(t3 + 0xc) + 4;
        m.w32(t3 + 0xc, q);
        copy_words(m, q, a1, 16);
        let v = m.r32(t3 + 0xc) + 0x40;
        m.w32(t3 + 0xc, v);
    } else {
        m.w32(p + 0x10, 0x6c01_c002);
        let q = m.r32(t3 + 0xc) + 4;
        m.w32(t3 + 0xc, q);
        copy_words(m, q, a1 + 0x20, 4);
        let v = m.r32(t3 + 0xc) + 0x10;
        m.w32(t3 + 0xc, v);
    }
    let a3 = a3 & 2;
    m.w32(a2 + 0xc, 0x7fff);
    if m.r16(a0 + 0x12) < 2 {
        m.w32(a2 + 4, 0);
    } else {
        m.w32(a2 + 4, 1);
    }
    if a3 != 0 || m.r32(a2 + 4) != 0 {
        let q = m.r32(t3 + 0xc);
        m.w32(q, 0x6c03_c004);
        let q = m.r32(t3 + 0xc) + 4;
        m.w32(t3 + 0xc, q);
        copy_words(m, q, a2, 12);
        let v = m.r32(t3 + 0xc) + 0x30;
        m.w32(t3 + 0xc, v);
    }
    loop {
        let q = m.r32(t3 + 0xc);
        if q & 0xf == 0 {
            break;
        }
        m.w32(q, 0);
        m.w32(t3 + 0xc, q + 4);
    }
    let v = m.r32(t4);
    let n = ((m.r32(t3 + 0xc).wrapping_sub(t4) as i32) >> 4).wrapping_sub(1) as u32;
    m.w32(t4, v | n);
    loop {
        let q = m.r32(t3 + 0xc);
        if q & 0xf == 0 {
            break;
        }
        m.w32(q, 0);
        m.w32(t3 + 0xc, q + 4);
    }
    let x = m.r32(a0 + 0x1c);
    let q = m.r32(t3 + 0xc);
    m.w32(q, 0x5000_0000);
    m.w32(q + 4, x);
    m.w32(t3 + 0xc, q + 0x10);
}

/// FUN_00249990(iy, x, y): `__kernel_tanf` (coefficients at 0x2ac0f8).
pub fn ee_sub_249990<M: QMem + ?Sized>(m: &mut M, a0: u32, f12: f32, f13: f32) -> f32 {
    let hx = f12.to_bits() as i32;
    let ix = (hx & 0x7fff_ffff) as i32;
    let (mut x, mut y) = (f12, f13);
    if !(0x317f_ffff < ix) && (x as i32) == 0 {
        if (ix as u32 | a0.wrapping_add(1)) == 0 {
            let f0 = m.call(0x0024_4ef8, &[], &[f4(x)]).1[0];
            return crate::engine::math::ps2f::ps2_div(1.0, f0);
        }
        if a0 == 1 {
            return x;
        }
        return crate::engine::math::ps2f::ps2_div(-1.0, x);
    }
    let big = 0x3f2c_a13f < ix;
    let z;
    if big {
        if hx < 0 {
            x = -x;
            y = -y;
        }
        let zz = f32::from_bits(0x3f49_0fda) - x;
        let ww = f32::from_bits(0x3322_2168) - y;
        y = 0.0;
        x = zz + ww;
        z = x * x;
    } else {
        z = x * x;
    }
    let tb = 0x002a_c0f8u32;
    let mut t = [0.0f32; 13];
    for k in 0..13 {
        t[k] = m.rf(tb + 4 * k as u32);
    }
    let w = z * z;
    let s = z * x;
    let f2 = w * (t[9] + w * t[11]);
    let f1 = w * (t[5] + w * (t[7] + f2));
    let r0 = t[1] + w * (t[3] + f1);
    let f3 = w * (t[8] + w * (t[10] + w * t[12]));
    let f8 = t[2] + w * (t[4] + w * (t[6] + f3));
    let v = z * f8;
    let mut r = s * (r0 + v);
    r = r + y;
    r = z * r;
    let r = (y + r) + t[0] * s;
    let w = x + r;
    if big {
        let v = a0 as i32 as f32;
        let f0 = w * w;
        let f1 = w + v;
        let sg = (1 - ((hx >> 30) & 2)) as f32;
        let q = f0 / f1 - r;
        return sg * (v - (x - q) * 2.0);
    }
    if a0 == 1 {
        return w;
    }
    let zz = f32::from_bits(w.to_bits() & 0xffff_f000);
    let f1 = zz - x;
    let a = -1.0 / w;
    let vv = r - f1;
    let tt = f32::from_bits(a.to_bits() & 0xffff_f000);
    let ss = tt * zz + 1.0;
    tt + a * (ss + tt * vv)
}

/// FUN_00247b00(y, x): `atan2f` (atanf = FUN_00244b68, fabsf = FUN_00244ef8).
pub fn ee_sub_247b00<M: QMem + ?Sized>(m: &mut M, f12: f32, f13: f32) -> f32 {
    let (y, x) = (f12, f13);
    let hx = x.to_bits() as i32;
    let hy = y.to_bits() as i32;
    let ix = hx & 0x7fff_ffff;
    let iy = hy & 0x7fff_ffff;
    let fb = f32::from_bits;
    if 0x7f80_0000 < ix || 0x7f80_0000 < iy {
        return x + y;
    }
    if hx == 0x3f80_0000 {
        return m.call(0x0024_4b68, &[], &[f4(y)]).1[0];
    }
    let mm = ((hy as u32) >> 31) as i32 | ((hx >> 30) & 2);
    let half_pi = |hy: i32| if hy >= 0 { fb(0x3fc9_0fdb) } else { fb(0xbfc9_0fdb) };
    if iy == 0 {
        match mm {
            0 | 1 => return y,
            2 => return fb(0x4049_0fda),
            3 => return fb(0xc049_0fda),
            _ => {}
        }
    }
    if ix == 0 {
        return half_pi(hy);
    }
    if ix == 0x7f80_0000 {
        if iy == 0x7f80_0000 {
            match mm {
                0 => return fb(0x3f49_0fdb),
                1 => return fb(0xbf49_0fdb),
                2 => return fb(0x4016_cbe4),
                3 => return fb(0xc016_cbe4),
                _ => {}
            }
        } else {
            match mm {
                0 => return 0.0,
                1 => return m.rf(0x002a_bbfc),
                2 => return fb(0x4049_0fda),
                3 => return fb(0xc049_0fda),
                _ => {}
            }
        }
    }
    if iy == 0x7f80_0000 {
        return half_pi(hy);
    }
    let k = iy.wrapping_sub(ix) >> 23;
    let z: u32 = if !(k < 0x3d) {
        0x3fc9_0fdc
    } else if hx < 0 && k < -0x3c {
        0
    } else {
        let f0 = m.call(0x0024_4ef8, &[], &[f4(y / x)]).1[0];
        m.call(0x0024_4b68, &[], &[f4(f0)]).1[0].to_bits()
    };
    match mm {
        0 => fb(z),
        1 => fb(z ^ 0x8000_0000),
        2 => fb(0x4049_0fda) - (fb(z) - fb(0x3422_2168)),
        _ => (fb(z) - fb(0x3422_2168)) - fb(0x4049_0fda),
    }
}

/// all-lanes FUN_0011d4d0(a, b, 1e-4) (approx-equal) over xyz, short-circuit like the asm.
fn approx3<M: QMem + ?Sized>(m: &mut M, a: V4, b: V4) -> bool {
    let e = f4(f32::from_bits(0x38d1_b717));
    for k in 0..3 {
        if m.call(0x0011_d4d0, &[], &[f4(a[k]), f4(b[k]), e]).0 == 0 {
            return false;
        }
    }
    true
}

/// FUN_001e57a0(cam, from, to, dur): start a camera blend from -> to (angles unwrapped via FUN_0011cf60;
/// zero pos / dir default to the current camera), then FUN_0014e4c8 / FUN_0014e328 on CM.
pub fn ee_sub_1e57a0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, f12: f32) {
    let f0 = if m.rf(a1 + 0x40) == 0.0 {
        m.rf(0x002a_8124)
    } else {
        m.w32(a2 + 4, 0);
        m.rf(0x002d_ded0) + f12
    };
    m.wf(a0 + 0x80, f0);
    m.w32(a0 + 0x84, a1 + 0x10);
    m.w32(a0 + 0x88, a2 + 0x10);
    m.wf(a0 + 0x8c, f12);
    m.w32(a0 + 0x90, 1);
    let lst = a0 + 0x84;
    let x = m.r32(a1 + 0x44);
    m.w32(a0 + 0x9c, 1);
    m.w32(a0 + 0x94, x);
    m.w32(a0 + 0x98, a0);
    for k in 0..3u32 {
        let s0 = a2 + 0x20 + 4 * k;
        let s1 = a1 + 0x20 + 4 * k;
        let d = m.rf(s0) - m.rf(s1);
        let f0 = m.call(0x0011_cf60, &[], &[f4(d)]).1[0];
        let f = m.rf(s1) + f0;
        m.wf(s0, f);
    }
    let fp = 0x002a_2dd0u32;
    for s6 in 0..2u32 {
        let s2 = m.r32(lst + 4 * s6);
        let z = m.rv(fp);
        let p = m.rv(s2);
        if approx3(m, z, p) {
            let c = m.rv(0x002d_dfd0);
            m.wv(s2, c);
        }
        let d = m.rv(s2 + 0x10);
        let z = m.rv(fp);
        if approx3(m, z, d) {
            let v = m.call(0x0011_bff0, &[0x002d_dfa0], &[]).1;
            m.wv(s2 + 0x10, v);
        }
        if m.rf(s2 + 0x20) == 0.0 {
            let f = m.rf(0x002d_df60);
            m.wf(s2 + 0x20, f);
        }
    }
    if m.rf(a0 + 0x8c) == 0.0 {
        let f = m.rf(0x002a_8124);
        m.wf(a0 + 0x8c, f);
    }
    let s0 = 0x002d_e440u32;
    if m.call(0x0014_fd50, &[s0], &[]).0 != 0 {
        let t1 = m.r32(0x002d_df40 + 0x510);
        m.call(0x0014_e4c8, &[0x002d_df40, 2, 0xa, s0, 0, t1], &[]);
    }
    m.call(0x0014_e328, &[0x002d_df40, 0xa, s0, 0, lst], &[]);
}

/// FUN_00199d48(ctx, out_side, out_pos): best (largest [0x18]) reachable attach edge among ctx[0]'s
/// edge lists (FUN_001999d0 test, FUN_00118fb0 clearance); returns the edge or 0.
pub fn ee_sub_199d48<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) -> u32 {
    let s4 = a0;
    let fp = a2;
    let x = m.r32(s4);
    if m.call(0x0014_4068, &[x], &[]).0 != 0 {
        return 0;
    }
    let v1 = m.r32(s4 + 0x1c) as i32;
    if v1 != 1 && !(v1 > 0 && v1 < 6 && v1 >= 3) {
        return 0;
    }
    let r4 = m.r32(s4 + 4) as i32;
    if r4 < 0x11 && !(r4 < 0xf) {
        return 0;
    }
    let sp = stack_push(m, 0x40);
    let mut s6 = 0u32;
    let mut f20 = 0.0f32;
    let o = m.r32(s4);
    let o = m.r32(o + 0xec);
    let mut s5 = m.r32(o + 0x64);
    if s5 == 0 {
        stack_pop(m, 0x40);
        return 0;
    }
    let field = |m: &mut M, p: u32| -> i32 { ((m.r32(p + 0x148) << 12) as i32) >> 28 };
    let mut e = m.r32(s5);
    loop {
        m.w32(sp + 0x30, e);
        if e != 0 {
            loop {
                let e0 = m.r32(sp + 0x30);
                'skip: {
                    let s1 = (m.r32(s4) != m.r32(e0 + 8)) as u32;
                    let so = (1 - s1) << 6;
                    let s2 = m.r32(e0.wrapping_add(so));
                    let mut p = m.r32(s2 + 4);
                    while p != 0 {
                        if field(m, p) == 2 {
                            break 'skip;
                        }
                        p = m.r32(p + 0x10);
                    }
                    let k = m.r32(e0 + 0x9c) as i32;
                    let t0 = match k {
                        2 => {
                            if s1 == 1 {
                                break 'skip;
                            }
                            m.r32(e0 + 0xa4)
                        }
                        1 => {
                            if s1 == 0 {
                                break 'skip;
                            }
                            m.r32(e0 + 0xa0)
                        }
                        3 => {
                            if s1 == 0 {
                                m.r32(e0 + 0xa4)
                            } else {
                                m.r32(e0 + 0xa0)
                            }
                        }
                        _ => break 'skip,
                    };
                    let q = m.rv(e0.wrapping_add(so) + 0x10);
                    let a3 = m.r32(e0 + 0x9c);
                    if m.call(0x0019_99d0, &[s4, s2, a3, t0, sp], &[q]).0 == 0 {
                        break 'skip;
                    }
                    let o = m.r32(s4);
                    let o = m.r32(o + 0xec);
                    let o2 = m.r32(s2 + 4);
                    let o2 = m.r32(o2 + 0xec);
                    if m.r64(o + 0x70) & 2 != 0 || m.r64(o2 + 0x70) & 2 != 0 {
                        m.call(0x001a_7fc0, &[0x003d_d490, 1, sp + 0x30], &[]);
                    }
                    let e1 = m.r32(sp + 0x30);
                    let f0 = m.rf(0x002c_3c60 + (s1 << 2)) * m.rf(e1 + 0xb8);
                    if 0.0 < f0 {
                        break 'skip;
                    }
                    let f0 = -m.rf(0x002b_f8a4);
                    let g = m.rf(0x002b_f8a8);
                    let b = m.rv(e1.wrapping_add(so) + 0x10);
                    let n = m.rv(sp);
                    let u = m.rv(0x002a_2dc0);
                    let one = [0.0, 0.0, 0.0, 1.0];
                    let mut r = [0.0f32; 4];
                    for k in 0..4 {
                        r[k] = b[k] + one[k] + n[k] * f0 + u[k] * g;
                    }
                    m.wv(sp + 0x10, r);
                    m.wv(sp + 0x20, [g, 0.0, 0.0, 0.0]);
                    let h = m.r32(s2 + 0x74);
                    if m.call(0x0011_8fb0, &[h, 0x7fff_ffff], &[r]).0 != 0 {
                        break 'skip;
                    }
                    if s6 != 0 && m.rf(e1.wrapping_add(so) + 0x18) < f20 {
                        break 'skip;
                    }
                    s6 = e1;
                    f20 = m.rf(s6.wrapping_add(so) + 0x18);
                    m.w32(a1, s1);
                    let q = m.rv(sp);
                    m.wv(fp, q);
                }
                let cur = m.r32(sp + 0x30);
                let nx = m.r32(cur + 0xac);
                m.w32(sp + 0x30, nx);
                if nx == 0 {
                    break;
                }
            }
        }
        s5 = m.r32(s5 + 4);
        if s5 == 0 {
            break;
        }
        e = m.r32(s5);
    }
    stack_pop(m, 0x40);
    s6
}

fn sub4(a: V4, b: V4) -> V4 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2], a[3] - b[3]]
}

/// FUN_00225e98(p): camera FOV / zoom driver: target-framing angle (atanf) eased via FUN_0011d1c0, or
/// (no target) the p[0x150] stick input; p[0x1bc] = lerp(p[0x1ac], p[0x1b0], p[0x1b4]).
pub fn ee_sub_225e98<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let s1 = a0;
    let t = m.r32(s1 + 0x180);
    if t != 0 && m.r32(s1 + 0x188) != 0 {
        let sp = stack_push(m, 0xb0);
        let d = m.r32(t + 0xec);
        let f20 = if d != 0 { m.rf(d + 0x30) } else { 0.0 };
        let p = m.call(0x001a_62e8, &[s1], &[]).1;
        m.wv(sp + 0xa0, p);
        let t = m.r32(s1 + 0x180);
        let q = m.call(0x001a_62e8, &[t], &[]).1;
        let p = m.rv(sp + 0xa0);
        let dd = sub4(p, q);
        let dist = dot3(dd, dd).abs().sqrt();
        let f12 = f20 / (dist * 0.5);
        let f0 = m.call(0x0024_4b68, &[], &[f4(f12)]).1[0];
        let mut f21 = f0 + f0;
        let t = m.r32(s1 + 0x180);
        let lo = m.rf(s1 + 0x1ac);
        let dt = m.rf(0x002d_ded0 + 4);
        let h = m.r32(t + 0x60);
        f21 = lo.max(f21);
        m.call(0x0023_09e8, &[h, 0, sp, 0], &[f4(dt)]);
        let c = m.rv(sp + 0x30);
        m.wv(sp + 0x40, c);
        m.wv(sp + 0xa0, c);
        let t = m.r32(s1 + 0x180);
        let q = m.call(0x001a_62e8, &[t], &[]).1;
        let c1 = m.rv(sp + 0xa0);
        let d1 = sub4(c1, q);
        m.wv(sp + 0x50, d1);
        let c = m.rv(sp + 0x40);
        m.wv(sp + 0x90, c);
        let f22 = f20 + dot3(d1, d1).abs().sqrt();
        let q2 = m.call(0x001a_62e8, &[s1], &[]).1;
        let c = m.rv(sp + 0x90);
        let v = sub4(c, q2);
        m.wv(sp + 0x60, v);
        let mp = m.call(0x001a_6318, &[s1], &[]).0;
        let r2 = m.rv(mp + 0x20);
        let r0 = m.rv(mp);
        let r1 = m.rv(mp + 0x10);
        let v = m.rv(sp + 0x60);
        let lv = [dot3(r0, v), dot3(r1, v), dot3(r2, v), r0[3]];
        m.wv(sp + 0x60, lv);
        let ny = -m.rf(sp + 0x64);
        let dx = m.rf(sp + 0x60);
        let dz = m.rf(sp + 0x68);
        m.wf(sp + 0x78, dx);
        m.wf(sp + 0x74, dz);
        m.wf(sp + 0x70, ny);
        let mut f0;
        if f32::from_bits(0x38d1_b717) < dx {
            let k = m.rf(0x002d_df7c);
            let a = m.call(0x0024_4b68, &[], &[f4((ny.abs() + f22) / (dx * k))]).1[0];
            let a = a + a;
            let z = m.rf(sp + 0x74).abs() + f22;
            let x = m.rf(sp + 0x78);
            let b = m.call(0x0024_4b68, &[], &[f4(z / x)]).1[0];
            let b = b + b;
            f0 = m.rf(s1 + 0x1b0).min(b.max(a));
        } else {
            f0 = m.rf(s1 + 0x1b0);
        }
        let lo = m.rf(s1 + 0x1ac);
        let hi = m.rf(s1 + 0x1b0);
        f0 = f21.max(f0);
        let u = if lo == hi { 0.0 } else { (f0 - lo) / (hi - lo) };
        let f0 = m.call(0x0011_d948, &[0x002b_bb60], &[f4(1.0 - u)]).1[0];
        let f12 = m.rf(s1 + 0x1b4);
        let f5 = m.rf(0x002c_4418);
        let f4v = m.rf(0x002c_441c);
        for k in 0..16 {
            let b = m.r8(0x002a_97b0 + k);
            m.w8(sp + 0x80 + k, b);
        }
        let f1 = (f0 - f12).abs();
        let f15 = m.rf(0x002d_ded0 + 4);
        let f13 = m.rf(s1 + 0x1b8);
        let w = (1.0 - f1) * f5 + f1 * f4v;
        for off in [0x84, 0x8c, 0x80] {
            let x = m.rf(sp + off) * w;
            m.wf(sp + off, x);
        }
        let r = m.call(0x0011_d1c0, &[sp + 0x80, s1 + 0x1b8], &[f4(f12), f4(f13), f4(f0), f4(f15)]).1[0];
        m.wf(s1 + 0x1b4, r);
        stack_pop(m, 0xb0);
    } else {
        let a1 = m.r32(s1 + 0x150);
        if a1 != 0 {
            let f3 = m.rf(s1 + 0x1b4);
            let c0 = m.rf(0x002c_4420);
            let c1 = m.rf(0x002c_4424);
            let c2 = m.rf(0x002c_4428);
            let f2 = m.rf(a1 + 0x24);
            let f12 = (c0 + f3 * (c1 + f3 * c2)) * f2 * f2.abs();
            m.wf(s1 + 0x1b8, f12);
            let k = m.rf(0x002d_ded4);
            let f0 = m.call(0x0011_d948, &[0x002b_bb60], &[f4(f3 + f12 * k)]).1[0];
            m.wf(s1 + 0x1b4, f0);
        }
    }
    let f1 = 1.0 - m.rf(s1 + 0x1b4);
    let hi = m.rf(s1 + 0x1b0);
    let lo = m.rf(s1 + 0x1ac);
    let f0 = 1.0 - f1;
    m.wf(s1 + 0x1bc, f0 * lo + f1 * hi);
}

/// FUN_001f0cf0(p, mask, id): collect 0x18-stride entries of p[0x5c] (count p[0x58]) matching mask and
/// id (at +4, or +8 when flag bit 1), into a 64-slot stack array, then FUN_001f0ad0(p, n, arr).
pub fn ee_sub_1f0cf0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) -> u32 {
    let sp = stack_push(m, 0x110);
    let mut cnt = m.r32(a0.wrapping_add(0x58)) as i32;
    let mut n: u32 = 0;
    if cnt > 0 {
        let mut e = m.r32(a0.wrapping_add(0x5c));
        let mut t0 = sp;
        loop {
            let fl = m.r32(e.wrapping_add(0xc));
            if fl & a1 != 0
                && (m.r32(e.wrapping_add(4)) == a2 || (fl & 1 != 0 && m.r32(e.wrapping_add(8)) == a2))
            {
                m.w32(t0, e);
                n = n.wrapping_add(1);
                t0 = t0.wrapping_add(4);
            }
            cnt = cnt.wrapping_sub(1);
            e = e.wrapping_add(0x18);
            if cnt == 0 {
                break;
            }
        }
    }
    let r = if n != 0 { m.call(0x1f0ad0, &[a0, n, sp], &[]).0 } else { 0 };
    stack_pop(m, 0x110);
    r
}
