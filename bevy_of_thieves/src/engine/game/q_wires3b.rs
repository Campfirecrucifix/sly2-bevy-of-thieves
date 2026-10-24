//! WIRE-S3: 1:1 QMem ports of live-frontier natives (shard 5/6/7).
#![allow(unused)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

const GS: u32 = 0x003d_d490; // game state / world
const CM: u32 = 0x002d_df40; // camera manager
const T: u32 = 0x002d_ded0; // g_clock t

/// byte-wise swap of `n` bytes (same memory effect as the libc long / byte swap loops)
fn swapn<M: QMem + ?Sized>(m: &mut M, a: u32, b: u32, n: u32) {
    for i in 0..n {
        let x = m.r8(a.wrapping_add(i));
        let y = m.r8(b.wrapping_add(i));
        m.w8(a.wrapping_add(i), y);
        m.w8(b.wrapping_add(i), x);
    }
}

fn qcmp<M: QMem + ?Sized>(m: &mut M, cmp: u32, a: u32, b: u32) -> i32 {
    m.call(cmp, &[a, b], &[]).0 as i32
}

fn med3<M: QMem + ?Sized>(m: &mut M, cmp: u32, a: u32, b: u32, c: u32) -> u32 {
    if qcmp(m, cmp, a, b) < 0 {
        if qcmp(m, cmp, b, c) < 0 {
            b
        } else if qcmp(m, cmp, a, c) < 0 {
            c
        } else {
            a
        }
    } else if qcmp(m, cmp, b, c) > 0 {
        b
    } else if qcmp(m, cmp, a, c) < 0 {
        a
    } else {
        c
    }
}

/// FUN_002338c0: qsort(base, n, es, cmp) (BSD/newlib: med3 / ninther pivot, 3-way partition,
/// insertion sort for n < 7 or no swaps, recurse on the left part, loop on the right).
pub fn ee_sub_2338c0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32) {
    let (mut a, mut n, es, cmp) = (a0, a1, a2, a3);
    loop {
        let mut swap_cnt = false;
        if n < 7 {
            let end = a.wrapping_add(n.wrapping_mul(es));
            let mut pm = a.wrapping_add(es);
            while pm < end {
                let mut pl = pm;
                while a < pl && qcmp(m, cmp, pl.wrapping_sub(es), pl) > 0 {
                    swapn(m, pl, pl.wrapping_sub(es), es);
                    pl = pl.wrapping_sub(es);
                }
                pm = pm.wrapping_add(es);
            }
            return;
        }
        let mut pm = a.wrapping_add((n >> 1).wrapping_mul(es));
        if n > 7 {
            let mut pl = a;
            let mut pn = a.wrapping_add(n.wrapping_sub(1).wrapping_mul(es));
            if n > 40 {
                let d = (n >> 3).wrapping_mul(es);
                pl = med3(m, cmp, pl, pl.wrapping_add(d), pl.wrapping_add(d.wrapping_mul(2)));
                pm = med3(m, cmp, pm.wrapping_sub(d), pm, pm.wrapping_add(d));
                pn = med3(m, cmp, pn.wrapping_sub(d.wrapping_mul(2)), pn.wrapping_sub(d), pn);
            }
            pm = med3(m, cmp, pl, pm, pn);
        }
        swapn(m, a, pm, es);
        let mut pa = a.wrapping_add(es);
        let mut pb = pa;
        let mut pc = a.wrapping_add(n.wrapping_sub(1).wrapping_mul(es));
        let mut pd = pc;
        loop {
            while pb <= pc {
                let r = qcmp(m, cmp, pb, a);
                if r > 0 {
                    break;
                }
                if r == 0 {
                    swap_cnt = true;
                    swapn(m, pa, pb, es);
                    pa = pa.wrapping_add(es);
                }
                pb = pb.wrapping_add(es);
            }
            while pb <= pc {
                let r = qcmp(m, cmp, pc, a);
                if r < 0 {
                    break;
                }
                if r == 0 {
                    swap_cnt = true;
                    swapn(m, pc, pd, es);
                    pd = pd.wrapping_sub(es);
                }
                pc = pc.wrapping_sub(es);
            }
            if pb > pc {
                break;
            }
            swapn(m, pb, pc, es);
            swap_cnt = true;
            pb = pb.wrapping_add(es);
            pc = pc.wrapping_sub(es);
        }
        if !swap_cnt {
            let end = a.wrapping_add(n.wrapping_mul(es));
            let mut pm = a.wrapping_add(es);
            while pm < end {
                let mut pl = pm;
                while a < pl && qcmp(m, cmp, pl.wrapping_sub(es), pl) > 0 {
                    swapn(m, pl, pl.wrapping_sub(es), es);
                    pl = pl.wrapping_sub(es);
                }
                pm = pm.wrapping_add(es);
            }
            return;
        }
        let pn = a.wrapping_add(n.wrapping_mul(es));
        let x = pa.wrapping_sub(a) as i32;
        let y = pb.wrapping_sub(pa) as i32;
        let r = if x < y { x } else { y };
        if r > 0 {
            swapn(m, a, pb.wrapping_sub(r as u32), r as u32);
        }
        let x = pd.wrapping_sub(pc);
        let y = pn.wrapping_sub(pd).wrapping_sub(es);
        let r = if x < y { x } else { y };
        if (r as i32) > 0 {
            swapn(m, pb, pn.wrapping_sub(r), r);
        }
        let r = pb.wrapping_sub(pa);
        if r > es {
            m.call(0x0023_38c0, &[a, r / es, es, cmp], &[]);
        }
        let r = pd.wrapping_sub(pc);
        if r <= es {
            return;
        }
        a = pn.wrapping_sub(r);
        n = r / es;
    }
}

/// FUN_001995d8(o, mat, cnt, out): writes 4 corner points from o+0x108..0x11c, then 9 rotated pairs
/// (FUN_0011b500 rotation by k*pi/12, k = 4..-4, about DAT_002a2dc0) and transforms the first cnt by mat.
pub fn ee_sub_1995d8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32) {
    let p = a3;
    let v = m.r32(a0 + 0x114);
    m.w32(p, v);
    let v = m.r32(a0 + 0x11c);
    m.w32(p + 8, v);
    m.w32(p + 4, 0);
    let v = m.r32(a0 + 0x114);
    m.w32(p + 0x10, v);
    let v = m.r32(a0 + 0x118);
    m.w32(p + 0x18, v);
    m.w32(p + 0x14, 0);
    let v = m.r32(a0 + 0x108);
    m.w32(p + 0x20, v);
    let v = m.r32(a0 + 0x110);
    m.w32(p + 0x28, v);
    m.w32(p + 0x24, 0);
    m.w32(p + 0x34, 0);
    let v = m.r32(a0 + 0x114);
    m.w32(p + 0x30, v);
    let v = m.r32(a0 + 0x11c);
    m.w32(p + 0x38, v);
    let f3 = m.rf(p + 0x34);
    let x = m.rf(a0 + 0x108);
    let z0 = m.r32(a0 + 0x110);
    let z1 = m.r32(a0 + 0x10c);
    // quads built via pextlw/pcpyld: w = sign-extension high half of the z word
    let sx = |z: u32| f32::from_bits(if (z as i32) < 0 { 0xffff_ffff } else { 0 });
    let q70 = [x, f3, f32::from_bits(z0), sx(z0)];
    let q80 = [x, f3, f32::from_bits(z1), sx(z1)];
    let f20 = f32::from_bits(0x3e86_0a92);
    let sp = stack_push(m, 0x40);
    let mut s0 = p + 0x40;
    let mut s1: i32 = 8;
    loop {
        let ax = m.rv(0x002a_2dc0);
        let f12 = (s1 - 4) as f32 * f20;
        s1 -= 1;
        m.call(0x0011_b500, &[sp], &[ax, f4(f12)]);
        let r0 = m.rv(sp);
        let r1 = m.rv(sp + 0x10);
        let r2 = m.rv(sp + 0x20);
        let mut o = [0.0f32; 4];
        for i in 0..3 {
            o[i] = r0[i] * q70[0] + r1[i] * q70[1] + r2[i] * q70[2];
        }
        o[3] = q80[3];
        m.wv(s0, o);
        let mut o2 = r0;
        for i in 0..3 {
            o2[i] = r0[i] * q80[0] + r1[i] * q80[1] + r2[i] * q80[2];
        }
        m.wv(s0 + 0x10, o2);
        s0 += 0x20;
        if s1 < 0 {
            break;
        }
    }
    stack_pop(m, 0x40);
    let mut s1 = (a2 as i32).wrapping_sub(1);
    let mut s0 = p;
    while s1 >= 0 {
        let v = m.rv(s0);
        s1 -= 1;
        let r0 = m.rv(a1);
        let r1 = m.rv(a1 + 0x10);
        let r2 = m.rv(a1 + 0x20);
        let r3 = m.rv(a1 + 0x30);
        let mut o = r0;
        for i in 0..3 {
            o[i] = r0[i] * v[0] + r1[i] * v[1] + r2[i] * v[2] + r3[i];
        }
        m.wv(s0, o);
        s0 += 0x10;
    }
}

/// FUN_00171468(o): +0x40 == 0 -> 0; switch on (+0x38)->type (5..14) polling the step's completion
/// (+0x40 = 0 when done); +0x40 == 0 -> FUN_001ef0d8(o + 0x18); returns +0x40.
pub fn ee_sub_171468<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    if m.r32(a0 + 0x40) == 0 {
        return 0;
    }
    let a1 = m.r32(a0 + 0x38);
    let ty = m.r32(a1).wrapping_sub(5);
    // done: Some(v0) -> "if v0 != 0 { +0x40 = 0 }"; None -> plain reload
    let mut res: Option<u32> = None;
    match ty {
        0 => {
            let f2 = m.rf(a0 + 0x3c);
            let f0 = m.rf(T) - f2;
            let f1 = m.rf(a1 + 8);
            if f1 < f0 {
                m.w32(a0 + 0x40, 0);
            }
        }
        1 => {
            let k = m.r32(a0 + 0x10);
            let v = m.r32(k + 0xf80) as i32;
            let s = m.r32(a0 + 0x18);
            if v == 0 {
                res = Some(m.call(0x0017_ce30, &[s], &[]).0);
            } else if v == -1 {
                res = Some(m.call(0x0017_ee78, &[s], &[]).0);
            } else if v == 1 {
                res = Some(m.call(0x0017_d188, &[s], &[]).0);
            }
        }
        2 => {
            let s = m.r32(a0 + 0x18);
            if m.call(0x0017_dcb8, &[s], &[]).0 != 0 {
                let k = m.r32(a0 + 0x10);
                m.w32(a0 + 0x40, 0);
                let f12 = m.rf(a0 + 0x48);
                m.call(0x0020_9818, &[k.wrapping_add(0xfd0)], &[f4(f12)]);
            }
        }
        8 | 9 => {
            let s = m.r32(a0 + 0x18);
            res = Some(m.call(0x0017_4fb8, &[s], &[]).0);
        }
        _ => {}
    }
    if let Some(v) = res {
        if v != 0 {
            m.w32(a0 + 0x40, 0);
        }
    }
    if m.r32(a0 + 0x40) == 0 {
        m.call(0x001e_f0d8, &[a0 + 0x18], &[]);
    }
    m.r32(a0 + 0x40)
}

/// FUN_0023d338(fp): pack an unpacked double {class, sign, exp, frac64} into IEEE-754 bits
/// (NaN / inf / zero / denormal / round-to-nearest-even).
pub fn ee_sub_23d338<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u64 {
    let cls = m.r32(a0);
    let mut exp: u32 = 0;
    let sign = m.r32(a0 + 4);
    let mut fr = m.r64(a0 + 0x10);
    if cls < 2 {
        fr |= 1u64 << 51;
        exp = 0x7ff;
    } else if cls == 4 {
        exp = 0x7ff;
        fr = 0;
    } else if cls == 2 {
        fr = 0;
    } else if fr != 0 {
        let e = m.r32(a0 + 8) as i32;
        if e < -0x3fe {
            let d = (-0x3fe_i32).wrapping_sub(e);
            fr = if d < 0x39 { fr >> (d as u32 & 63) } else { 0 };
            fr >>= 8;
        } else if e >= 0x400 {
            exp = 0x7ff;
            fr = 0;
        } else {
            exp = e.wrapping_add(0x3ff) as u32;
            if fr & 0xff != 0x80 {
                fr = fr.wrapping_add(0x7f);
            } else if fr & 0x100 != 0 {
                fr = fr.wrapping_add(0x80);
            }
            if (u64::MAX >> 3) < fr {
                fr >>= 1;
                exp = exp.wrapping_add(1);
            }
            fr >>= 8;
        }
    }
    (fr & (u64::MAX >> 12)) | (((exp & 0x7ff) as u64) << 52) | ((sign as u64) << 63)
}

/// FUN_00117030(pos_out, vel_out, dur, t, vscale, p0, v0, p1, v1): cubic Hermite position / velocity at u = t/dur.
pub fn ee_sub_117030<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, f12: f32, f13: f32, f14: f32, f15: f32, f16: f32, f17: f32, f18: f32) {
    let f13 = f13 / f12;
    let f8 = 1.0f32;
    let f7 = f17 - f15;
    let f6 = f8 - f13;
    if a0 != 0 {
        let mut f1 = f13 * f6;
        let f4 = 3.0f32;
        let mut f0 = f6 * f6;
        let mut f2 = f13 * f13;
        f1 = f1 * f6;
        f0 = f0 * f6;
        let mut f3 = f2 * f6;
        let f5 = f1 * f4;
        f1 = f1 * f12;
        let f4 = f3 * f4;
        f0 = f0 + f5;
        f2 = f2 * f13;
        f1 = f1 * f16;
        f0 = f0 * f15;
        f3 = f3 * f12;
        f2 = f2 + f4;
        f0 = f0 + f1;
        f3 = f3 * f18;
        f2 = f2 * f17;
        f0 = f0 - f3;
        f0 = f0 + f2;
        m.wf(a0, f0);
    }
    if a1 != 0 {
        let mut f0 = f6 * 6.0;
        let mut f2 = f13 * 3.0;
        let mut f1 = 4.0f32;
        let f3 = -2.0f32;
        f0 = f13 * f0;
        f1 = f1 - f2;
        f2 = f2 + f3;
        f0 = f0 / f12;
        f1 = f13 * f1;
        f2 = f13 * f2;
        f1 = f8 - f1;
        f2 = f2 * f18;
        f0 = f0 * f7;
        f1 = f1 * f16;
        f0 = f0 + f1;
        f0 = f0 + f2;
        f0 = f0 * f14;
        m.wf(a1, f0);
    }
}

/// FUN_00170a58(o): walk the +0x50 list (next +0x18): type 0 -> o = FUN_0015e938(GS, o ? 4 : 5, +4, o);
/// type 6 -> x = FUN_0015e938(GS, 4, +4, o), FUN_001709a8(x) == -1 -> FUN_001e9c30(x).
pub fn ee_sub_170a58<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let mut s2 = a0;
    let mut s1 = m.r32(a0 + 0x50);
    while s1 != 0 {
        let v = m.r32(s1);
        if v == 0 {
            let a1 = if s2 != 0 { 4 } else { 5 };
            let a2 = m.r32(s1 + 4);
            s2 = m.call(0x0015_e938, &[GS, a1, a2, s2], &[]).0;
        } else if v == 6 {
            let a2 = m.r32(s1 + 4);
            let s0 = m.call(0x0015_e938, &[GS, 4, a2, s2], &[]).0;
            if s0 != 0 && m.call(0x0017_09a8, &[s0], &[]).0 == 0xffff_ffff {
                m.call(0x001e_9c30, &[s0], &[]);
            }
        }
        s1 = m.r32(s1 + 0x18);
    }
}

/// FUN_00143d20(o, f12): h = +0x248 + f12 * vslot32(o).z + 0.5 f12^2 (+0xe8)->+0x88; h > 0 -> 0;
/// else FUN_00173610(&pos (FUN_001a62e8), 0, 0, 0.0, h).
pub fn ee_sub_143d20<M: QMem + ?Sized>(m: &mut M, a0: u32, f12: f32) -> u32 {
    let mut f20 = f12;
    let mut f21 = m.rf(a0 + 0x248);
    let f22 = 0.0f32;
    let q = vslot(m, a0, 32, &[], &[]).1;
    let f1 = f20 * 0.5;
    let f0 = f20 * q[2];
    let e8 = m.r32(a0 + 0xe8);
    let f2 = m.rf(e8 + 0x88);
    f20 = f20 * f1;
    f21 = f21 + f0;
    f20 = f20 * f2;
    f21 = f21 + f20;
    if f22 < f21 {
        return 0;
    }
    let pos = m.call(0x001a_62e8, &[a0], &[]).1;
    let sp = stack_push(m, 0x10);
    m.wv(sp, pos);
    let r = m.call(0x0017_3610, &[sp, 0, 0], &[f4(f22), f4(f21)]).0;
    stack_pop(m, 0x10);
    r
}

/// FUN_00150780(o): +0x10 -> FUN_001a62e8(+0x10); else FUN_001c4a48(GS) -> its pos with z += 150;
/// else camera position (CM + 0x90).
pub fn ee_sub_150780<M: QMem + ?Sized>(m: &mut M, a0: u32) -> V4 {
    let k = m.r32(a0 + 0x10);
    if k != 0 {
        return m.call(0x001a_62e8, &[k], &[]).1;
    }
    let v = m.call(0x001c_4a48, &[GS], &[]).0;
    if v != 0 {
        let mut q = m.call(0x001a_62e8, &[v], &[]).1;
        q[2] = q[2] + 150.0;
        return q;
    }
    m.rv(CM + 0x90)
}

/// FUN_002376d0(i): FUN_002374e0(i); lock (FUN_00234a60(DAT_002c4a88)); i < 32 -> &DAT_0044bb80[i] (16-byte
/// entries) else 0; unlock (FUN_00234a40).
pub fn ee_sub_2376d0<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    m.call(0x0023_74e0, &[a0], &[]);
    let s = m.r32(0x002c_4a88);
    m.call(0x0023_4a60, &[s], &[]);
    if a0 >= 0x20 {
        let s = m.r32(0x002c_4a88);
        m.call(0x0023_4a40, &[s], &[]);
        return 0;
    }
    let r = (a0 << 4).wrapping_add(0x0044_bb80);
    let s = m.r32(0x002c_4a88);
    m.call(0x0023_4a40, &[s], &[]);
    r
}

/// FUN_00150ea0(o, a (quad), b (quad), f): +0x80 = a, +0x90 = b, +0xa0 = f; first time (+0x40 == 0)
/// FUN_00150ef0(o, a, b, f); +0x40 = 1.
pub fn ee_sub_150ea0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: V4, a2: V4, f12: f32) {
    m.wv(a0 + 0x80, a1);
    m.wv(a0 + 0x90, a2);
    m.wf(a0 + 0xa0, f12);
    if m.r32(a0 + 0x40) == 0 {
        m.call(0x0015_0ef0, &[a0], &[a1, a2, f4(f12)]);
    }
    m.w32(a0 + 0x40, 1);
}

/// FUN_001a9fe0(_, node, &head): node != 0 -> append *head at the tail of node's +0xac chain; *head = node.
pub fn ee_sub_1a9fe0<M: QMem + ?Sized>(m: &mut M, _a0: u32, a1: u32, a2: u32) {
    if a1 == 0 {
        return;
    }
    let mut v1 = a1;
    let a0 = m.r32(a2);
    if m.r32(a1 + 0xac) != 0 {
        v1 = m.r32(v1 + 0xac);
        while m.r32(v1 + 0xac) != 0 {
            v1 = m.r32(v1 + 0xac);
        }
    }
    m.w32(v1 + 0xac, a0);
    m.w32(a2, a1);
}

/// FUN_00113a20: returns 0x002baafc.
pub fn ee_sub_113a20<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_aafc
}

/// FUN_00113a70: returns 0x002bab60.
pub fn ee_sub_113a70<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_ab60
}

/// FUN_00113c00: returns 0x002bad54.
pub fn ee_sub_113c00<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_ad54
}

/// FUN_00113ea0: returns 0x002bb09c.
pub fn ee_sub_113ea0<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_b09c
}

/// FUN_00114130: returns 0x002bb3d0.
pub fn ee_sub_114130<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_b3d0
}

/// FUN_00114260: returns 0x002bb54c.
pub fn ee_sub_114260<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_b54c
}

/// FUN_00237b80: tail FUN_00234a40(DAT_002c4a84).
pub fn ee_sub_237b80<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let s = m.r32(0x002c_4a84);
    m.call(0x0023_4a40, &[s], &[]).0
}
