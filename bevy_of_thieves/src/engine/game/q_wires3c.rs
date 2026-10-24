//! WIRE-S3: 1:1 QMem ports of live-frontier natives (shard 5/6/7).
#![allow(unused)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

/// FUN_00248f38: __kernel_rem_pio2f(x, y, e0, nx, prec, ipio2) (float, 8-bit chunks); returns n & 7.
/// init_jk = DAT_002ac080, PIo2 = DAT_002ac090; scalbnf = FUN_0024a0e8, floorf = FUN_00244f18.
pub fn ee_sub_248f38<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, t0: u32, t1: u32) -> u32 {
    const INIT_JK: u32 = 0x002a_c080;
    const PIO2: u32 = 0x002a_c090;
    let x = a0;
    let y = a1;
    let e0 = a2 as i32;
    let nx = a3 as i32;
    let prec = t0 as i32;
    let ipio2 = t1;
    let scalbnf = |m: &mut M, z: f32, n: i32| m.call(0x0024_a0e8, &[n as u32], &[f4(z)]).1[0];
    let mut iq = [0i32; 20];
    let mut f = [0f32; 20];
    let mut fq = [0f32; 20];
    let mut q = [0f32; 20];
    let jk = m.r32(INIT_JK.wrapping_add((prec as u32) << 2)) as i32;
    let jp = jk;
    let jx = nx - 1;
    let mut jv = (e0 - 3) / 8;
    if jv < 0 {
        jv = 0;
    }
    let mut q0 = e0 - 8 * (jv + 1);
    let mut j = jv - jx;
    let mm = jx + jk;
    let mut i = 0;
    while i <= mm {
        f[i as usize] = if j < 0 { 0.0 } else { m.r32(ipio2.wrapping_add((j as u32) << 2)) as i32 as f32 };
        i += 1;
        j += 1;
    }
    for i in 0..jk + 1 {
        let mut fw = 0.0f32;
        let mut j = 0;
        while j <= jx {
            fw += m.rf(x.wrapping_add((j as u32) << 2)) * f[(jx + i - j) as usize];
            j += 1;
        }
        q[i as usize] = fw;
    }
    let mut jz = jk;
    let mut z;
    let mut n;
    let mut ih;
    loop {
        // recompute
        z = q[jz as usize];
        let mut i = 0usize;
        let mut j = jz;
        while j > 0 {
            let fw = ((z * 0.00390625) as i32) as f32;
            iq[i] = (z - 256.0 * fw) as i32;
            z = q[(j - 1) as usize] + fw;
            i += 1;
            j -= 1;
        }
        z = scalbnf(m, z, q0);
        let fl = m.call(0x0024_4f18, &[], &[f4(z * 0.125)]).1[0];
        z -= fl * 8.0;
        n = z as i32;
        z -= n as f32;
        ih = 0;
        if q0 > 0 {
            let k = (jz - 1) as usize;
            let i = iq[k] >> (8 - q0);
            n += i;
            iq[k] -= i << (8 - q0);
            ih = iq[k] >> (7 - q0);
        } else if q0 == 0 {
            ih = iq[(jz - 1) as usize] >> 8;
        } else if z >= 0.5 {
            ih = 2;
        }
        if ih > 0 {
            n += 1;
            let mut carry = 0;
            for i in 0..jz.max(0) as usize {
                let j = iq[i];
                if carry == 0 {
                    if j != 0 {
                        carry = 1;
                        iq[i] = 0x100 - j;
                    }
                } else {
                    iq[i] = 0xff - j;
                }
            }
            if q0 > 0 {
                let k = (jz - 1) as usize;
                match q0 {
                    1 => iq[k] &= 0x7f,
                    2 => iq[k] &= 0x3f,
                    _ => {}
                }
            }
            if ih == 2 {
                z = 1.0 - z;
                if carry != 0 {
                    z -= scalbnf(m, 1.0, q0);
                }
            }
        }
        if z == 0.0 {
            let mut j = 0;
            let mut i = jz - 1;
            while i >= jk {
                j |= iq[i as usize];
                i -= 1;
            }
            if j == 0 {
                let mut k = 1;
                while iq[(jk - k) as usize] == 0 {
                    k += 1;
                }
                let mut i = jz + 1;
                while i <= jz + k {
                    f[(jx + i) as usize] = m.r32(ipio2.wrapping_add(((jv + i) as u32) << 2)) as i32 as f32;
                    let mut fw = 0.0f32;
                    let mut j = 0;
                    while j <= jx {
                        fw += m.rf(x.wrapping_add((j as u32) << 2)) * f[(jx + i - j) as usize];
                        j += 1;
                    }
                    q[i as usize] = fw;
                    i += 1;
                }
                jz += k;
                continue;
            }
        }
        break;
    }
    if z == 0.0 {
        jz -= 1;
        q0 -= 8;
        while iq[jz as usize] == 0 {
            jz -= 1;
            q0 -= 8;
        }
    } else {
        z = scalbnf(m, z, -q0);
        if z >= 256.0 {
            let fw = ((0.00390625 * z) as i32) as f32;
            iq[jz as usize] = (z - 256.0 * fw) as i32;
            jz += 1;
            q0 += 8;
            iq[jz as usize] = fw as i32;
        } else {
            iq[jz as usize] = z as i32;
        }
    }
    let mut fw = scalbnf(m, 1.0, q0);
    let mut i = jz;
    while i >= 0 {
        q[i as usize] = fw * iq[i as usize] as f32;
        fw *= 0.00390625;
        i -= 1;
    }
    let mut i = jz;
    while i >= 0 {
        let mut fw = 0.0f32;
        let mut k = 0;
        while k <= jp && k <= jz - i {
            fw += m.rf(PIO2 + 4 * k as u32) * q[(i + k) as usize];
            k += 1;
        }
        fq[(jz - i) as usize] = fw;
        i -= 1;
    }
    match prec {
        0 => {
            let mut fw = 0.0f32;
            let mut i = jz;
            while i >= 0 {
                fw += fq[i as usize];
                i -= 1;
            }
            m.wf(y, if ih == 0 { fw } else { -fw });
        }
        1 | 2 => {
            let mut fw = 0.0f32;
            let mut i = jz;
            while i >= 0 {
                fw += fq[i as usize];
                i -= 1;
            }
            m.wf(y, if ih == 0 { fw } else { -fw });
            fw = fq[0] - fw;
            for i in 1..=jz.max(0) {
                fw += fq[i as usize];
            }
            m.wf(y + 4, if ih == 0 { fw } else { -fw });
        }
        3 => {
            let mut i = jz;
            while i > 0 {
                let a = i as usize;
                let fw = fq[a - 1] + fq[a];
                fq[a] += fq[a - 1] - fw;
                fq[a - 1] = fw;
                i -= 1;
            }
            let mut i = jz;
            while i > 1 {
                let a = i as usize;
                let fw = fq[a - 1] + fq[a];
                fq[a] += fq[a - 1] - fw;
                fq[a - 1] = fw;
                i -= 1;
            }
            let mut fw = 0.0f32;
            let mut i = jz;
            while i >= 2 {
                fw += fq[i as usize];
                i -= 1;
            }
            if ih == 0 {
                m.wf(y, fq[0]);
                m.wf(y + 4, fq[1]);
                m.wf(y + 8, fw);
            } else {
                m.wf(y, -fq[0]);
                m.wf(y + 4, -fq[1]);
                m.wf(y + 8, -fw);
            }
        }
        _ => {}
    }
    (n & 7) as u32
}

/// FUN_0015e3e8(_, obj, key, flags, anchor, chain, max, cnt, [out, best]): oid-match filter; appends obj to out.
#[allow(clippy::too_many_arguments)]
pub fn ee_sub_15e3e8<M: QMem + ?Sized>(m: &mut M, a1: u32, a2: u32, a3: u32, t0: u32, t1: u32, t2: u32, t3: u32, out: u32, best: u32) {
    let (obj, key, flags, anchor, chain, max, cnt) = (a1, a2, a3, t0, t1, t2, t3);
    let bit60 = |m: &mut M| (m.r64(obj + 0x28) >> 60) & 1 != 0;
    if flags & 0x400 == 0 {
        let k = ((m.r64(obj + 0x28) << 12) as i64) >> 60;
        let mut p = chain;
        if k == 0 {
            loop {
                if p == 0 {
                    return;
                }
                let a = m.r32(p);
                let a = m.call(0x001c_9950, &[a], &[]).0;
                let b = m.r32(key);
                if a == m.call(0x001c_9950, &[b], &[]).0 {
                    break;
                }
                if flags & 0x800 != 0 {
                    return;
                }
                p = m.r32(p + 4);
            }
        } else if k == -1 {
            while p != key {
                if p == 0 || flags & 0x800 != 0 {
                    return;
                }
                p = m.r32(p + 4);
            }
        }
    }
    let depth = |m: &mut M, o: u32| m.r8(o + 0x2c) as i8 as i32;
    // jump table DAT_002a51c0
    match (flags & 0xff).wrapping_sub(1) {
        0 => {
            if obj == 0 {
                return;
            }
            let mut o = obj;
            let mut c = depth(m, o);
            while c == 0 {
                o = m.r32(o + 0x10);
                if o == 0 {
                    return;
                }
                c = depth(m, o);
            }
            if c != 1 {
                return;
            }
        }
        1 => {
            if m.r32(obj + 0x10) != anchor {
                return;
            }
        }
        2 => {
            if depth(m, obj) < 2 {
                return;
            }
        }
        3 => {
            let b0 = m.r32(best);
            let mut o = obj;
            while o != 0 {
                let c = depth(m, o);
                if c != 0 && c < b0 as i32 {
                    if flags & 0x100 != 0 && !bit60(m) {
                        return;
                    }
                    m.w32(best, c as u32);
                    m.w32(cnt, 0);
                    store_15e3e8(m, obj, max, cnt, out);
                    return;
                }
                o = m.r32(o + 0x10);
            }
            if b0 != 0x7fff_ffff {
                return;
            }
        }
        4 => {}
        _ => return,
    }
    if flags & 0x100 != 0 && !bit60(m) {
        return;
    }
    store_15e3e8(m, obj, max, cnt, out);
}

fn store_15e3e8<M: QMem + ?Sized>(m: &mut M, obj: u32, max: u32, cnt: u32, out: u32) {
    let i = m.r32(cnt) as i32;
    if i < max as i32 {
        m.w32(out.wrapping_add((i as u32) << 2), obj);
    }
    let c = m.r32(cnt).wrapping_add(1);
    m.w32(cnt, c);
}

/// FUN_00241f98(lsn): cdvd sync-read wrapper: FUN_00241a30; busy check / FUN_00241f28; FUN_00236a60(0);
/// if DAT_002c6794 < 0, retry FUN_002370a8(0x2c82c0, 0x80000593, 0) until it succeeds with +0x24 != 0.
pub fn ee_sub_241f98<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    m.call(0x0024_1a30, &[a0], &[]);
    let s = m.r32(0x002c_676c);
    let v = m.call(0x0023_4a70, &[s], &[]).0;
    if m.r32(0x002c_676c) != v {
        if m.r32(0x002c_6750) as i32 > 0 {
            let a2 = m.r32(0x002c_6758);
            m.call(0x0023_6960, &[0x002a_b530, a0, a2], &[]);
        }
        return 0;
    }
    m.w32(0x002c_6758, a0);
    if m.call(0x0024_1f28, &[1], &[]).0 != 0 {
        let s = m.r32(0x002c_676c);
        m.call(0x0023_4a40, &[s], &[]);
        return 0;
    }
    m.call(0x0023_6a60, &[0], &[]);
    if m.r32(0x002c_6794) as i32 >= 0 {
        return 1;
    }
    let s0 = 0x002c_82c0;
    loop {
        let r = m.call(0x0023_70a8, &[s0, 0x8000_0593, 0], &[]).0;
        if r as i32 >= 0 {
            if m.r32(s0 + 0x24) != 0 {
                m.w32(0x002c_6794, 0);
                return 1;
            }
        } else if m.r32(0x002c_6750) as i32 > 0 {
            m.call(0x0023_6960, &[0x002a_b558], &[]);
        }
        // busy-wait 0x100000 iterations (no memory effects)
    }
}

/// FUN_00248ca8: sqrtf (bitwise fdlibm e_sqrtf).
pub fn ee_sub_248ca8<M: QMem + ?Sized>(m: &mut M, f12: f32) -> f32 {
    let x = f12;
    let mut ix = x.to_bits() as i32;
    if (ix as u32) & 0x7f80_0000 == 0x7f80_0000 {
        return x * x + x;
    }
    let mut mm = ix >> 23;
    if ix <= 0 {
        if ix & 0x7fff_ffff == 0 {
            return x;
        }
        if ix < 0 {
            let d = x - x;
            return crate::engine::math::ps2f::ps2_div(d, d);
        }
    }
    if mm == 0 {
        let mut i = 0;
        while ix & 0x0080_0000 == 0 {
            ix = ix.wrapping_shl(1);
            i += 1;
        }
        mm = 1 - i;
    }
    mm -= 127;
    ix = (ix & 0x007f_ffff) | 0x0080_0000;
    ix = ix.wrapping_shl((mm & 1) as u32);
    mm >>= 1;
    let t0 = mm.wrapping_shl(23);
    ix = ix.wrapping_shl(1);
    let mut q: i32 = 0;
    let mut s: i32 = 0;
    let mut r: i32 = 0x0100_0000;
    while r != 0 {
        let t = s.wrapping_add(r);
        if !(ix < t) {
            ix = ix.wrapping_sub(t);
            s = t.wrapping_add(r);
            q = q.wrapping_add(r);
        }
        r = ((r as u32) >> 1) as i32;
        ix = ix.wrapping_shl(1);
    }
    if ix != 0 {
        q = q.wrapping_add(q & 1);
    }
    let v = (q >> 1).wrapping_add(0x3f00_0000).wrapping_add(t0);
    f32::from_bits(v as u32)
}

/// FUN_0023dff8(fp, bits): pack unpacked float {class, sign, exp, frac} into an f32 (a2 = initial bits).
pub fn ee_sub_23dff8<M: QMem + ?Sized>(m: &mut M, a0: u32, a2: u32) -> f32 {
    let cls = m.r32(a0);
    let sign = m.r32(a0 + 4);
    let mut frac = m.r32(a0 + 0xc);
    let mut e: u32 = 0;
    if cls < 2 {
        e = 0xff;
        frac |= 0x0010_0000;
    } else if cls == 4 {
        e = 0xff;
        frac = 0;
    } else if cls == 2 {
        frac = 0;
    } else if frac != 0 {
        let ex = m.r32(a0 + 8) as i32;
        if ex < -126 {
            let sh = -126 - ex;
            frac = if sh < 26 { frac >> (sh as u32 & 31) } else { 0 };
            frac >>= 7;
        } else if ex >= 128 {
            e = 0xff;
            frac = 0;
        } else {
            e = (ex + 0x7f) as u32;
            if frac & 0x7f != 0x40 {
                frac = frac.wrapping_add(0x3f);
            } else if frac & 0x80 != 0 {
                frac = frac.wrapping_add(0x40);
            }
            if (frac as i32) < 0 {
                frac >>= 1;
                e = e.wrapping_add(1);
            }
            frac >>= 7;
        }
    }
    let mut b = (a2 & 0xff80_0000) | (frac & 0x007f_ffff);
    b &= 0x807f_ffff;
    b |= (e & 0xff) << 23;
    b &= 0x7fff_ffff;
    b |= sign << 31;
    f32::from_bits(b)
}

/// FUN_001d88d0(a0, ...): varargs: frame FUN_001ae478; s0 = FUN_001ae1d0({8, 0x10}); FUN_001d87c0(a0, va, s0);
/// s0[1] = FUN_001ae238({s0[0] * 2, 0x10}, s0[1]); FUN_001ae4a0(frame, 2); returns s0.
#[allow(clippy::too_many_arguments)]
pub fn ee_sub_1d88d0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, t0: u32, t1: u32, t2: u32, t3: u32, f12: f32, f14: f32, f16: f32, f18: f32) -> u32 {
    let sp = stack_push(m, 0x100);
    for (k, r) in [a1, a2, a3, t0, t1, t2, t3].into_iter().enumerate() {
        let o = sp + 0xc8 + 8 * k as u32;
        m.w32(o, r);
        m.w32(o + 4, if (r as i32) < 0 { u32::MAX } else { 0 });
    }
    m.wf(sp + 0xb8, f12);
    m.wf(sp + 0xbc, f14);
    m.wf(sp + 0xc0, f16);
    m.wf(sp + 0xc4, f18);
    m.call(0x001a_e478, &[sp], &[]);
    let s1 = sp + 0x10;
    m.w32(s1, 8);
    m.w32(s1 + 4, 0x10);
    let s0 = m.call(0x001a_e1d0, &[s1], &[]).0;
    m.call(0x001d_87c0, &[a0, sp + 0xc8, s0], &[]);
    let n = m.r32(s0);
    let a = m.r32(s0 + 4);
    m.w32(s1, n << 1);
    m.w32(s1 + 4, 0x10);
    let v = m.call(0x001a_e238, &[s1, a], &[]).0;
    m.w32(s0 + 4, v);
    m.call(0x001a_e4a0, &[sp, 2], &[]);
    stack_pop(m, 0x100);
    s0
}

/// FUN_00150f58(o): v = FUN_00150780(o); all of FUN_0011d4d0(o[+0x20].c, v.c, 1e-4) for x, y, z -> 1.
pub fn ee_sub_150f58<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let a = m.rv(a0 + 0x20);
    let b = m.call(0x0015_0780, &[a0], &[]).1;
    let eps = f32::from_bits(0x38d1_b717);
    for c in 0..3 {
        if m.call(0x0011_d4d0, &[], &[f4(a[c]), f4(b[c]), f4(eps)]).0 == 0 {
            return 0;
        }
    }
    1
}

/// FUN_001aa020(pool, src): clone the +0xac-linked list src (0xc0-byte nodes from FUN_0011ccc0(pool + 0x3d40)).
pub fn ee_sub_1aa020<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    if a1 == 0 {
        return 0;
    }
    let mut head = 0u32;
    let mut prev: Option<u32> = None;
    let mut s0 = a1;
    loop {
        let v = m.call(0x0011_ccc0, &[a0 + 0x3d40], &[]).0;
        for k in (0..0xc0).step_by(4) {
            let w = m.r32(s0 + k);
            m.w32(v + k, w);
        }
        match prev {
            None => head = v,
            Some(p) => m.w32(p, v),
        }
        s0 = m.r32(s0 + 0xac);
        prev = Some(v + 0xac);
        if s0 == 0 {
            break;
        }
    }
    head
}

/// FUN_00150728(o): FUN_0014fca8(o); vtable 0x2cd498; +0xb4 = 7.0; +0x18 = -1.0; +0xc0 = +0xc4 = 0.
pub fn ee_sub_150728<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    m.call(0x0014_fca8, &[a0], &[]);
    m.w32(a0 + 0xc, 0x002c_d498);
    m.wf(a0 + 0xb4, 7.0);
    m.wf(a0 + 0x18, -1.0);
    m.w32(a0 + 0xc0, 0);
    m.w32(a0 + 0xc4, 0);
    a0
}

/// FUN_00156728(o): d = (+0x14)[+0x60]; d >= 0 && +0x4c + d < t -> FUN_00156780(o).
pub fn ee_sub_156728<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let p = m.r32(a0 + 0x14);
    let d = m.rf(p + 0x60);
    if 0.0 <= d && m.rf(a0 + 0x4c) + d < m.rf(0x002d_ded0) {
        m.call(0x0015_6780, &[a0], &[]);
    }
}

/// FUN_001813c0(o, s): +0x378 = DAT_002be870 * s; +0x37c = DAT_002be874 * s; +0x380 = DAT_002be878.
pub fn ee_sub_1813c0<M: QMem + ?Sized>(m: &mut M, a0: u32, f12: f32) {
    let v = 0x002b_e870;
    let x = m.rf(v) * f12;
    m.wf(a0 + 0x378, x);
    let y = m.rf(v + 4) * f12;
    m.wf(a0 + 0x37c, y);
    let z = m.rf(v + 8);
    m.wf(a0 + 0x380, z);
}

/// FUN_0019b088(o, k, a, b): a + b (+ DAT_002bf900 when k == +0xbc).
pub fn ee_sub_19b088<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, f12: f32, f13: f32) -> f32 {
    let mut f = f12 + f13;
    if a1 == m.r32(a0 + 0xbc) {
        f += m.rf(0x002b_f900);
    }
    f
}

/// FUN_00113bb0: returns 0x2bacf0.
pub fn ee_sub_113bb0<M: QMem + ?Sized>(m: &mut M) -> u32 {
    0x002b_acf0
}

/// FUN_00113d80: returns 0x2baf34.
pub fn ee_sub_113d80<M: QMem + ?Sized>(m: &mut M) -> u32 {
    0x002b_af34
}

/// FUN_001140e0: returns 0x2bb36c.
pub fn ee_sub_1140e0<M: QMem + ?Sized>(m: &mut M) -> u32 {
    0x002b_b36c
}

/// FUN_00114190: returns 0x2bb448.
pub fn ee_sub_114190<M: QMem + ?Sized>(m: &mut M) -> u32 {
    0x002b_b448
}

/// FUN_00114540: returns 0x2bb8e4.
pub fn ee_sub_114540<M: QMem + ?Sized>(m: &mut M) -> u32 {
    0x002b_b8e4
}
