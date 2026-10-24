//! WP-Q11 part A: class packages C2ca820, C2cd528, C2cfd20, C2cfda0, C2cff20, C2d0020, C2d00a0, C2d0220, C2d02a0, C2d03a0.
//! 1:1 from ghidra_functions.body on `QMem`; callees outside the package go through `m.call` by address
//! (ints in `a`, floats / quadwords in `v`, floats first, float result `.1[0]`).
#![allow(unused_variables, unused_mut, dead_code)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};
use crate::engine::classes::q::q6::{ro32, rof, rov, s};
use crate::engine::classes::q::q7util::{f, fr, mul3, rows3, scale, sub, CLOCK};

const NEG1: u32 = u32::MAX;
const WORLD: u32 = 0x003d_d490;

/// Q11 overrides whose whole body is `return <const>;`: (fn, value)
pub const CONST_RET: &[(u32, u32)] = &[(0x114610, 0x2bb9e8)];

/// |a - b| over xyz (VU0 vsub / vmul / vaddbc / vsqrt)
pub fn dist3(a: V4, b: V4) -> f32 {
    let d: [f32; 3] = std::array::from_fn(|i| a[i] - b[i]);
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}
fn pos<M: QMem + ?Sized>(m: &mut M, obj: u32) -> V4 {
    m.call(0x001a_62e8, &[obj], &[]).1
}
fn cpy16<M: QMem + ?Sized>(m: &mut M, dst: u32, src: u32) {
    let v = m.rv(src);
    m.wv(dst, v);
}

// ---- C2ca820 ----------------------------------------------------------------------------------------

/// FUN_0012d088 (C2ca820_v3)
pub fn c2ca820_v3<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let p = m.r32(o + 8);
    cpy16(m, p + 0xc0, o + 0x20);
    let q = rov(0x002a_2dd0);
    let p = m.r32(o + 8);
    m.wv(p + 0xd0, q);
    let p = m.r32(o + 8);
    let h = m.r16(o + 0x1c);
    m.w16(p + 0x9a, h);
    let (a, b) = (m.r32(o + 4), m.r32(o + 8));
    m.call(0x0022_c818, &[a, b], &[]);
}
/// FUN_0012d0d8 (C2ca820_v4)
pub fn c2ca820_v4<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32, p3: u32) {
    if p2 != 0 {
        cpy16(m, p2, o + 0x20);
    }
    if p3 != 0 {
        m.w32(p3, 0);
        m.w32(p3 + 8, 0);
        m.w32(p3 + 4, 0);
    }
}

// ---- C2cd528 ----------------------------------------------------------------------------------------

/// FUN_0014fda0 (v0)
pub fn c2cd528_v0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0014_fcc0, &[o], &[]);
}
/// FUN_0014fdc0 (v5)
pub fn c2cd528_v5<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0014_fd40, &[o], &[]);
}

// ---- C2cfd20 ----------------------------------------------------------------------------------------

/// FUN_0017dde8 (v8)
pub fn c2cfd20_v8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef78, &[o], &[]);
    m.call(0x0017_e278, &[o, NEG1], &[]);
    let ow = m.r32(o + 0x10);
    let a = m.r32(ow + 0xf64);
    m.call(0x0015_3ff8, &[a], &[]);
    let ow = m.r32(o + 0x10);
    m.call(0x0020_8540, &[ow, 0], &[]);
    let ow = m.r32(o + 0x10);
    m.call(0x0020_9970, &[ow, 0x2b], &[]);
}
/// FUN_0017de50 (v9)
pub fn c2cfd20_v9<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef98, &[o], &[]);
    let mut st;
    loop {
        let prev = m.r32(o + 0x30);
        let done = m.call(0x001e_f0a8, &[o], &[]).0 != 0;
        st = m.r32(o + 0x30);
        if done {
            break;
        }
        let ow = m.r32(o + 0x10);
        if st == 1 {
            if m.call(0x0020_9098, &[ow], &[]).0 == 0 {
                let t = m.rf(o + 0x40) - m.rf(CLOCK + 4);
                m.wf(o + 0x40, t);
                if t <= 0.0 {
                    let r = vslot(m, ow, 87, &[], &[]).0;
                    if r != m.r32(o + 0x38) || m.call(0x0020_8b38, &[ow], &[]).0 == 0 {
                        m.call(0x0017_e278, &[o, NEG1], &[]);
                        st = m.r32(o + 0x30);
                        if st != prev {
                            continue;
                        }
                        break;
                    }
                    let v = m.call(0x0020_8bb0, &[ow], &[]).0;
                    m.w32(o + 0x40, v);
                }
                st = m.r32(o + 0x30);
            } else {
                m.call(0x0017_e278, &[o, 2], &[]);
                st = m.r32(o + 0x30);
            }
        } else if st < 2 {
            if st == 0 {
                let a = m.r32(o + 0x18);
                if m.call(0x0017_4fb8, &[a], &[]).0 == 0 {
                    st = m.r32(o + 0x30);
                } else {
                    m.call(0x0017_e278, &[o, 1], &[]);
                    st = m.r32(o + 0x30);
                }
            } else {
                st = m.r32(o + 0x30);
            }
        } else if st == 2 {
            if m.call(0x0020_9098, &[ow], &[]).0 == 0 {
                m.call(0x0017_e278, &[o, 1], &[]);
            }
            st = m.r32(o + 0x30);
        } else {
            st = m.r32(o + 0x30);
        }
        if st == prev {
            break;
        }
    }
    let st = m.r32(o + 0x30);
    if st == 1 {
        let c = m.r32(o + 0x18);
        if rof(0x002b_e760) < m.rf(CLOCK) - m.rf(o + 0x3c) {
            let mut near = false;
            let ow = m.r32(o + 0x10);
            let a = pos(m, ow);
            let e = m.call(0x001c_1ec0, &[c + 0x40], &[a]).1;
            let ow = m.r32(o + 0x10);
            let b = pos(m, ow);
            if rof(0x002b_e764) <= dist3(b, e) {
                let ow = m.r32(o + 0x10);
                let p = pos(m, ow);
                let t = m.r32(o + 0x38);
                let q = pos(m, t);
                if dist3(p, q) < rof(0x002b_e768) {
                    near = true;
                }
            } else {
                near = true;
            }
            if near {
                m.call(0x0017_e0e0, &[o], &[]);
            }
        }
    }
}

// ---- C2cfda0 ----------------------------------------------------------------------------------------

/// FUN_0017eb48 (v6)
pub fn c2cfda0_v6<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef68, &[o], &[]);
    m.w32(o + 0x38, 0xbf80_0000);
    let ow = m.r32(o + 0x10);
    m.call(0x0020_9890, &[ow, 0x2e], &[]);
}
/// FUN_0017ec80 (v8)
pub fn c2cfda0_v8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef78, &[o], &[]);
    let ow = m.r32(o + 0x10);
    m.call(0x0020_9970, &[ow, 0x2e], &[]);
}

// ---- C2cff20 ----------------------------------------------------------------------------------------

/// FUN_0017d1f0 (v7)
pub fn c2cff20_v7<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef70, &[o], &[]);
    let ow = m.r32(o + 0x10);
    let c = m.call(0x0020_5438, &[ow, 0x9d], &[]).0;
    m.w32(c + 0x3c, 2);
    vslot(m, c, 7, &[], &[]);
    let ow = m.r32(o + 0x10);
    let a = m.r32(ow + 0xf84);
    let p = m.call(0x001a_6318, &[a], &[]).0;
    let sp = stack_push(m, 0x30);
    let r0 = m.rv(p);
    let ax = rov(0x002a_2dc0);
    m.call(0x0011_b830, &[sp], &[r0, ax]);
    for i in 0..12 {
        let w = m.r32(sp + 4 * i);
        m.w32(ow + 0x1b0 + 4 * i, w);
    }
    stack_pop(m, 0x30);
}

// ---- C2d0020 ----------------------------------------------------------------------------------------

/// FUN_0017cb80 (v9)
pub fn c2d0020_v9<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef98, &[o], &[]);
    let sp = stack_push(m, 0x40);
    let (ba0, b80) = (sp, sp + 0x20);
    let mut st = m.r32(o + 0x30);
    loop {
        if st == 0 {
            let ow = m.r32(o + 0x10);
            let f88 = m.rf(ow + 0xf88);
            m.wf(b80, f88);
            let x = m.r32(ow + 0xf84);
            let fl = (f88 < m.rf(o + 0x3c)) as u32;
            m.w32(ow + 0xf8c, fl);
            let ow = m.r32(o + 0x10);
            let p = pos(m, ow);
            let ow = m.r32(o + 0x10);
            let k = rof(0x002b_e74c);
            let fv = if m.r32(ow + 0xf8c) == 0 { f88 - k } else { f88 + k };
            let u = fr(m.call(0x001e_66c8, &[x], &[f(fv)]));
            m.call(0x001e_7380, &[x, 0, 0, ba0, 0, b80], &[f(u), p]);
            let ow = m.r32(o + 0x10);
            let nf = m.r32(b80);
            m.w32(ow + 0xf88, nf);
            if m.call(0x0017_ce30, &[o], &[]).0 == 0 {
                let ow = m.r32(o + 0x10);
                if m.r32(ow + 0xf8c) == 0 {
                    let v = m.rv(ba0);
                    m.wv(ba0, scale(v, -1.0));
                }
                let ow = m.r32(o + 0x10);
                let (y, xx) = (m.rf(ba0 + 4), m.rf(ba0));
                let ang = fr(m.call(0x0024_58f0, &[], &[f(y), f(xx)]));
                m.call(0x0020_9818, &[ow + 0xfd0], &[f(ang)]);
                // PART r16 ABI: asm passes a0 = ((o+0x18)->0x40)->8, $f12 = o+0x38
                let a = m.rf(o + 0x38);
                let b = m.r32(o + 0x18);
                let b = m.r32(b + 0x40);
                let b = m.r32(b + 8);
                m.call(0x0013_1a00, &[b], &[f(a)]);
            } else {
                m.call(0x0017_cd00, &[o, 1], &[]);
            }
        }
        let cur = m.r32(o + 0x30);
        if cur == st {
            break;
        }
        st = m.r32(o + 0x30);
    }
    stack_pop(m, 0x40);
}

// ---- C2d00a0 ----------------------------------------------------------------------------------------

/// FUN_0017c6d0 (v9)
pub fn c2d00a0_v9<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef98, &[o], &[]);
    let sp = stack_push(m, 0x30);
    let a = m.r32(o + 0x30);
    let p_a = pos(m, a);
    let b = m.r32(o + 0x10);
    let p_b = pos(m, b);
    let dist = dist3(p_a, p_b);
    loop {
        if m.call(0x001e_f0a8, &[o], &[]).0 != 0 {
            break;
        }
        let st = m.r32(o + 0x50);
        let ow = m.r32(o + 0x10);
        if st == 1 {
            if m.call(0x0020_9098, &[ow], &[]).0 == 0 {
                if dist < 700.0 {
                    m.call(0x0017_c928, &[o, 0], &[]);
                } else {
                    let q = m.rv(o + 0x60);
                    if dist3(q, p_a) > 250.0 {
                        m.wv(o + 0x60, p_a);
                        m.call(0x0025_62a8, &[sp], &[]);
                        m.wv(sp + 0x10, p_a);
                        let t = m.r32(o + 0x18);
                        m.call(0x0017_dd38, &[t, sp], &[]);
                    }
                }
            } else {
                m.call(0x0017_c928, &[o, 2], &[]);
            }
        } else if st < 2 {
            if st == 0 {
                if m.call(0x0020_9098, &[ow], &[]).0 != 0 {
                    m.call(0x0017_c928, &[o, 2], &[]);
                } else if 1000.0 < dist {
                    m.call(0x0017_c928, &[o, 1], &[]);
                } else {
                    let q = m.rv(o + 0x60);
                    if dist3(q, p_a) > 250.0 {
                        m.wv(o + 0x60, p_a);
                        let (t30, t40) = (m.r32(o + 0x30), m.r32(o + 0x40));
                        let r = m.call(0x001a_6f00, &[t30, 0, t40], &[]).1;
                        m.call(0x0025_62a8, &[sp], &[]);
                        m.wv(sp + 0x10, r);
                        let t = m.r32(o + 0x18);
                        m.call(0x0017_dd38, &[t, sp], &[]);
                    }
                }
            }
        } else if st == 2 {
            if m.call(0x0020_9098, &[ow], &[]).0 == 0 {
                m.call(0x0017_c928, &[o, 0], &[]);
            }
        }
        if st == m.r32(o + 0x50) {
            break;
        }
    }
    stack_pop(m, 0x30);
}

// ---- C2d0220 ----------------------------------------------------------------------------------------

/// FUN_0017b620 (v9)
pub fn c2d0220_v9<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef98, &[o], &[]);
    loop {
        if m.call(0x001e_f0a8, &[o], &[]).0 != 0 {
            return;
        }
        let st = m.r32(o + 0x30);
        if st == 0 {
            let a = m.r32(o + 0x18);
            if m.call(0x0017_4fb8, &[a], &[]).0 != 0 {
                m.call(0x0017_b750, &[o, NEG1], &[]);
            }
        }
        if st == m.r32(o + 0x30) {
            return;
        }
    }
}
/// FUN_0017b690 (v12): message 9 from the owner (kinds 0x31 / 0x32 set / clear owner +0xf60)
pub fn c2d0220_v12<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32, p3: u32) -> u32 {
    if p2 == 9 && m.r32(o + 0x30) == 0 {
        let c = m.r32(o + 0x18);
        if m.r32(c + 0x40) != 0 {
            let id = m.call(0x0020_9518, &[], &[]).0;
            if id == m.r32(p3) {
                let k = m.r32(p3 + 8);
                let ow = m.r32(o + 0x10);
                if k == 0x31 {
                    m.w32(ow + 0xf60, 1);
                    return 1;
                }
                if k == 0x32 {
                    m.w32(ow + 0xf60, 0);
                    return 1;
                }
            }
        }
    }
    m.call(0x001e_f0b8, &[o, p2, p3], &[]).0
}

// ---- C2d02a0 ----------------------------------------------------------------------------------------

/// FUN_00174bd0 (v12)
pub fn c2d02a0_v12<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32, p3: u32) -> u32 {
    if p2 != 9 {
        return 0;
    }
    if m.r32(o + 0x40) == 0 {
        return 0;
    }
    if m.call(0x0020_9518, &[], &[]).0 != m.r32(p3) {
        return 0;
    }
    let k = m.r32(p3 + 8) as i32;
    let ow = m.r32(o + 0x10);
    if k == 0x2a {
        m.w32(ow + 0x1a4, 0);
        return 1;
    }
    if k < 0x2b {
        if k == 0x27 {
            m.call(0x001e_efc0, &[o], &[]);
            return 1;
        }
        if 0x27 < k {
            if k == 0x28 {
                m.call(0x001e_f038, &[o], &[]);
                return 1;
            }
            if k != 0x29 {
                return 0;
            }
            m.w32(ow + 0x1a4, 1);
            return 1;
        }
        if k == 0x25 {
            m.call(0x0020_97c8, &[ow + 0xfd0, o], &[]);
            return 1;
        }
        if k != 0x26 {
            return 0;
        }
        m.call(0x0020_97d8, &[ow + 0xfd0, o], &[]);
        return 1;
    }
    if k < 0x36 {
        if k < 0x34 {
            if k != 0x2f {
                return 0;
            }
            let r = m.call(0x0020_5458, &[ow, 0xa5], &[]).0;
            if r != 0 {
                let w = m.call(0x001c_4840, &[WORLD], &[]).0;
                if m.r32(r + 0x30) != w {
                    return 0;
                }
                m.call(0x0018_7e28, &[0x002d_fc00, 8], &[f(1.0)]);
                return 0;
            }
        } else {
            let w = m.call(0x001c_4840, &[WORLD], &[]).0;
            let r = m.call(0x0020_5458, &[ow, 0xa0], &[]).0;
            if r == 0 {
                return 0;
            }
            let a = m.r32(r + 0x50);
            if m.call(0x001a_6af8, &[a, w], &[]).0 != 0 {
                if w != m.r32(0x002f_7900) {
                    return 0;
                }
                if m.r32(p3 + 8) != 0x34 {
                    if m.r32(p3 + 8) != 0x35 {
                        return 0;
                    }
                    m.call(0x0014_f750, &[0x002d_df40, 0], &[]);
                    return 0;
                }
                m.call(0x0018_7e28, &[0x002d_fc00, 3], &[f(f32::from_bits(0x3e19_999a))]);
                return 0;
            }
        }
    } else {
        if k != 0x36 {
            if k != 0x15e {
                return 0;
            }
            // vector FUN_00204e38(f = p[1], owner); owner +0x190 = -v; slot 34 with the world-rotated v (rows +0x1b0)
            let sv = m.rf(p3 + 4);
            let v = m.call(0x0020_4e38, &[ow], &[f(sv)]).1;
            m.wv(ow + 0x190, scale(v, -1.0));
            let r = rows3(m, ow + 0x1b0);
            let w = mul3(&r, v, 0.0);
            vslot(m, ow, 34, &[], &[w]);
            return 1;
        }
        m.call(0x001f_7a68, &[ow, 2], &[]);
    }
    0
}
/// FUN_00174ea0 (v11)
pub fn c2d02a0_v11<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let p = m.r32(o + 0x40);
    if m.r32(p) == 2 {
        let q = m.r32(p + 8);
        let x = m.r32(q + 0x10) as i32;
        let mut r = 7;
        if x != 0 {
            if x < 0 || 4 < x {
                r = 1;
            } else {
                r = 1;
                if 2 < x {
                    r = 7;
                }
            }
        }
        r
    } else {
        m.call(0x001e_efa0, &[o], &[]).0
    }
}
/// FUN_00174f08 (v10)
pub fn c2d02a0_v10<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_efb8, &[o], &[]);
    let q = rov(0x002a_2dd0);
    if m.r32(o + 0x30) == 0 {
        let (a, b) = (m.r32(o + 0x40), m.r32(o + 0x10));
        m.call(0x0020_9618, &[a, b], &[]);
    } else {
        let ow = m.r32(o + 0x10);
        m.wv(ow + 400, q);
    }
    let t = m.r32(o + 0x38);
    if t != 0 {
        let a = pos(m, t);
        let ow = m.r32(o + 0x10);
        let b = pos(m, ow);
        let ow = m.r32(o + 0x10);
        let d = sub(a, b);
        let ang = fr(m.call(0x0024_58f0, &[], &[f(d[1]), f(d[0])]));
        m.call(0x0020_9818, &[ow + 0xfd0], &[f(ang)]);
    }
}
/// FUN_0017a4e8 (v7)
pub fn c2d02a0_v7<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0017_4950, &[o], &[]);
    m.call(0x0017_a5b0, &[o, 0], &[]);
    let ow = m.r32(o + 0x10);
    let a = m.r32(ow + 0xf64);
    m.call(0x0015_3ff8, &[a], &[]);
    let ow = m.r32(o + 0x10);
    let p = pos(m, ow);
    let ow = m.r32(o + 0x10);
    m.call(0x0015_6c80, &[WORLD, 1, ow, ow + 0x35c], &[p]);
}
/// FUN_0017a558 (v8)
pub fn c2d02a0_v8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0017_4b28, &[o], &[]);
    m.call(0x0017_a5b0, &[o, NEG1], &[]);
    let ow = m.r32(o + 0x10);
    let q = rov(0x002a_2dd0);
    m.call(0x0015_6c80, &[WORLD, NEG1, 0, ow + 0x35c], &[q]);
}
/// FUN_0017a670 (v9)
pub fn c2d02a0_v9<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0017_4b68, &[o], &[]);
    let mut st = m.r32(o + 0x50);
    loop {
        if st == 0 && m.rf(o + 0x58) < m.rf(CLOCK) {
            m.call(0x0017_a5b0, &[o, 1], &[]);
        }
        if m.r32(o + 0x50) == st {
            break;
        }
        st = m.r32(o + 0x50);
    }
    let ow = m.r32(o + 0x10);
    let p = m.r32(ow + 0x35c);
    let q = pos(m, ow);
    m.wv(p + 0x20, q);
}
/// FUN_002097c8
pub fn ee_sub_2097c8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.w32(o + 4, 1);
}
/// FUN_002097d8
pub fn ee_sub_2097d8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.w32(o + 4, 0);
}

// ---- C2d03a0 ----------------------------------------------------------------------------------------

/// FUN_0017aa08 (v7)
pub fn c2d03a0_v7<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0017_4950, &[o], &[]);
    let ow = m.r32(o + 0x10);
    let q = rov(0x002a_2dd0);
    m.call(0x0020_ab68, &[ow, 0, 0], &[q]);
    let ow = m.r32(o + 0x10);
    m.call(0x0020_b210, &[ow], &[]);
    m.call(0x0017_b1a8, &[o, 0], &[]);
    m.call(0x001e_efc0, &[o], &[]);
    let ow = m.r32(o + 0x10);
    let a = m.r32(ow + 0xf64);
    let b = m.r32(o + 0x5c);
    m.call(0x0015_4038, &[a, b, 2], &[]);
}
/// FUN_0017abc8
pub fn ee_sub_17abc8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.r32(o + 0x60) == 0 {
        return;
    }
    let sp = stack_push(m, 0x70);
    let (b90, b60, b50) = (sp, sp + 0x30, sp + 0x40);
    let c = m.r32(o + 0x5c);
    let t = m.r32(o + 0x64);
    let dt = m.rf(t + 0x1c) - m.rf(CLOCK);
    vslot(m, c, 95, &[b90], &[f(dt)]);
    if m.r32(b90) == 0 {
        m.w32(o + 0x60, 0);
    } else {
        let ow = m.r32(o + 0x10);
        let iv = m.r32(ow + 0x58);
        m.call(0x0017_aa78, &[o, b90, b60, b50], &[]);
        let a = m.r32(iv + 0xa0);
        let q1 = m.call(0x0022_fc48, &[0, a], &[]).1;
        let a = m.r32(iv + 0xa0);
        let q2 = m.call(0x0022_fcc8, &[0, a], &[]).1;
        let t = m.r32(o + 0x64);
        m.call(0x0012_e0f0, &[t], &[q1, q2]);
    }
    stack_pop(m, 0x70);
}
/// FUN_0017ac90 (v9)
pub fn c2d03a0_v9<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0017_4b68, &[o], &[]);
    let sp = stack_push(m, 0x100);
    let ow = m.r32(o + 0x10);
    let q0 = vslot(m, ow, 32, &[], &[]).1;
    let mut st = m.r32(o + 0x50) as i32;
    loop {
        let ow = m.r32(o + 0x10);
        let mut fall = true;
        match st {
            0 | 1 => {
                fall = false;
                if st == 1 && m.r32(ow + 0x208) != 0 {
                    m.call(0x0017_b1a8, &[o, 3], &[]);
                } else {
                    let a = m.r32(ow + 0x58);
                    let iv = m.r32(a + 0x90);
                    if iv != m.r32(o + 0x64) || m.rf(iv + 0x1c) < m.rf(CLOCK) {
                        m.call(0x0017_b1a8, &[o, 2], &[]);
                    } else if 0.0 <= q0[2] || m.r32(o + 0x50) != 0 {
                        ee_sub_17abc8(m, o);
                    } else {
                        m.call(0x0017_b1a8, &[o, 1], &[]);
                    }
                }
            }
            2 => {
                if m.r32(ow + 0x208) != 0 {
                    m.call(0x0017_b1a8, &[o, 3], &[]);
                    fall = false;
                }
            }
            4 | 5 => {
                fall = false;
                let p = pos(m, ow);
                let ow2 = m.r32(o + 0x10);
                let q2 = vslot(m, ow2, 32, &[], &[]).1;
                let c = m.r32(o + 0x5c);
                let c = m.r32(c + 0x19a4);
                let c = m.r32(c + 0x60);
                let (bf, e) = (sp, sp + 0x30);
                let dt = m.rf(CLOCK + 4);
                m.call(0x0023_09e8, &[c, 0, bf, 0], &[s(dt)]);
                let mut tgt = m.rv(bf + 0x30);
                tgt[2] += rof(0x002b_e6c8);
                let ow3 = m.r32(o + 0x10);
                let p3 = pos(m, ow3);
                let inv = 1.0 / m.rf(CLOCK + 4);
                let mut vel: V4 = std::array::from_fn(|i| (tgt[i] - p3[i]) * inv);
                let cur = m.r32(o + 0x50);
                let mut go = true;
                if cur == 4 {
                    if p[2] < tgt[2] || rof(0x002b_e6cc) < m.rf(CLOCK) - m.rf(o + 0x54) {
                        m.call(0x0017_b1a8, &[o, 5], &[]);
                        go = false;
                    } else {
                        vel[2] = q2[2];
                    }
                }
                if go {
                    let ow4 = m.r32(o + 0x10);
                    vslot(m, ow4, 34, &[], &[vel]);
                }
            }
            6 => {
                if m.r32(ow + 0x208) != 0 {
                    m.call(0x0017_b1a8, &[o, NEG1], &[]);
                }
            }
            _ => {}
        }
        let cur = m.r32(o + 0x50) as i32;
        if cur == st {
            break;
        }
        st = cur;
    }
    stack_pop(m, 0x100);
}
/// FUN_0017af30 (v12)
pub fn c2d03a0_v12<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32, p3: u32) -> u32 {
    if m.call(0x0017_4bd0, &[o, p2, p3], &[]).0 != 0 {
        return 1;
    }
    if p2 == 9 {
        let t = m.r32(o + 0x40);
        if t != 0 && m.call(0x0020_9518, &[], &[]).0 == m.r32(p3) && m.r32(p3 + 8) == 0x3d {
            m.w32(o + 0x58, 0);
        }
        return 0;
    }
    if !(p2 == 0xc && m.r32(p3 + 0xc) == 0x20_0000 && (m.r32(p3 + 0x44) & 8) != 0 && m.r32(o + 0x50) != 0) {
        return 0;
    }
    let t = m.r32(o + 0x40);
    if m.call(0x0020_9430, &[t], &[]).0 != 0 {
        m.call(0x0020_9360, &[t], &[]);
        let (ow, fl) = (m.r32(o + 0x10), m.r32(t + 0xc) | 1);
        m.call(0x0020_9278, &[t, ow, fl], &[f(0.0)]); // PART r16 ABI: $f12 = 0.0
    }
    let n = (m.r32(o + 0x58) as i32).wrapping_add(1);
    m.w32(o + 0x58, n as u32);
    let mut i3 = if n < 4 { n } else { 4 };
    let mut r = 0u32;
    let mut off;
    loop {
        off = (i3 * 4) as u32;
        if i3 < 1 {
            break;
        }
        let (a, b) = (m.r32(t + 4), ro32(0x002b_e6d0 + off));
        let p = m.call(0x0012_f938, &[a, b], &[]).0;
        i3 -= 1;
        if p != 0 {
            r = m.r32(p);
            break;
        }
    }
    let ptr = m.r32(t + 8);
    m.call(0x0013_0d30, &[ptr, 0], &[f(f32::from_bits(r)), f(1.0)]); // PART r16 ABI: $f12 = *p, $f13 = 1.0
    m.call(0x0017_b1a8, &[o, 0], &[]);
    let w = m.call(0x0015_e938, &[WORLD, 5, ro32(0x002b_e6e8 + off), 0], &[]).0;
    if w != 0 {
        let sp = stack_push(m, 0x40);
        m.call(0x0023_33b4, &[sp, 0, 0x40], &[]);
        m.w32(sp, 5);
        let ow = m.r32(o + 0x10);
        let pv = m.call(0x001a_62e8, &[ow], &[]).0;
        m.w32(sp + 0x20, pv);
        vslot(m, w, 50, &[sp], &[]);
        stack_pop(m, 0x40);
    }
    let ow = m.r32(o + 0x10);
    let n3 = (m.r32(ow + 0x33c) as i32).wrapping_add(m.r32(p3 + 0x10) as i32);
    let n3 = if 4 < n3 { n3 } else { 5 };
    m.w32(ow + 0x33c, n3 as u32);
    let p = pos(m, ow);
    let g = m.r32(0x003e_14b8);
    m.call(0x0015_6698, &[g, ow], &[p]);
    1
}
