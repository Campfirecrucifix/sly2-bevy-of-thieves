//! WIRE-S2: 1:1 QMem ports of live-frontier natives (shard 3/4).
#![allow(unused)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

const T: u32 = 0x002d_ded0; // g_clock

fn sub4(a: V4, b: V4) -> V4 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2], a[3] - b[3]]
}

/// FUN_0019e1e0: rail/wire step state update (smooth +0x1010, input checks, direction pick, speed integrate).
pub fn ee_sub_19e1e0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let sp = stack_push(m, 0xc0);
    body(m, a0, sp);
    stack_pop(m, 0xc0);
}

fn body<M: QMem + ?Sized>(m: &mut M, s1: u32, sp: u32) {
    let s0 = m.r32(s1);
    let f12 = m.rf(T + 4);
    let qa = m.rv(s0 + 0x1010);
    let qb = m.rv(0x002a_2dd0);
    let r = m.call(0x0011_f088, &[0x002b_f928, 0], &[qa, qb, f4(f12)]).1;
    m.wv(s0 + 0x1010, r);
    let v1 = m.r32(s1 + 0xc);
    let a0 = m.r32(s1);
    let bit = (m.r64(v1 + 0x28) >> 60) & 1;
    let s2 = m.r32(a0 + 0x150);
    if bit == 0 {
        m.call(0x0019_5a00, &[a0 + 0x2c0, 0, 0, 0], &[]);
        return;
    }
    if m.r32(s1 + 4) != 0 && m.call(0x001c_46c0, &[s2, 0x40], &[f4(0.25)]).0 != 0 {
        let a = m.r32(s1);
        m.call(0x0019_5a00, &[a + 0x2c0, 2, 0, 0], &[]);
        return;
    }
    if m.call(0x001c_4660, &[s2, 0x400], &[f4(f32::from_bits(0x3e19_999a))]).0 != 0 {
        let a2 = m.r32(s1);
        if vslot(m, a2, 98, &[3], &[]).0 != 0 {
            m.call(0x001c_4738, &[s2, 0x400], &[]);
            let a = m.r32(s1);
            m.call(0x0019_cc50, &[a + 0xf00, 0, 0, 0], &[]);
        }
    }
    let mut s0 = m.r32(s1 + 4);
    let v1 = loop {
        if s0 == 0 || s0 == 4 {
            let o = m.r32(s1);
            let k = m.r32(o + 0x1030);
            if m.call(0x0013_1968, &[k], &[]).0 != 0 {
                m.call(0x0019_eab0, &[s1, 1], &[]);
            }
        }
        let v1 = m.r32(s1 + 4);
        if v1 != s0 {
            s0 = m.r32(s1 + 4);
            continue;
        }
        break v1;
    };
    if v1 == 0 || v1 == 4 {
        return;
    }
    let f20c: f32 = 50.0;
    let o = m.r32(s1 + 0xc);
    let f = m.rf(s1 + 0x10) + f20c;
    let f0 = m.call(0x001e_6728, &[o], &[f4(f)]).1[0];
    let o = m.r32(s1 + 0xc);
    m.call(0x001e_6f10, &[o, sp, 0], &[f4(f0)]);
    let f = m.rf(s1 + 0x10) - f20c;
    let o = m.r32(s1 + 0xc);
    let f0 = m.call(0x001e_6728, &[o], &[f4(f)]).1[0];
    let o = m.r32(s1 + 0xc);
    m.call(0x001e_6f10, &[o, sp + 0x10, 0], &[f4(f0)]);
    let d = sub4(m.rv(sp), m.rv(sp + 0x10));
    let len = ((d[0] * d[0] + d[1] * d[1]) + d[2] * d[2]).sqrt();
    let n: V4 = if len < f32::from_bits(0x38d1_b717) {
        m.rv(0x002a_2dc0)
    } else {
        let q = 1.0 / len;
        [d[0] * q, d[1] * q, d[2] * q, d[3] * q]
    };
    let nz = n[2];
    // labels: 0 = L490/494 path, 1 = L53c (state 4), 2 = L54c
    let go53c;
    if 0.5 < nz.abs() {
        let v0 = m.r32(s1 + 0x18);
        go53c = if 0.0 < nz { v0 != 0 } else { v0 == 0 };
    } else {
        go53c = false;
    }
    let mut to53c = go53c;
    if !to53c {
        let f2 = m.rf(s1 + 0x1c);
        if f32::from_bits(0x3ecc_cccd) < f2 && m.r32(s1 + 0x18) != 0 && f32::from_bits(0xbe99_999a) < nz {
            to53c = true;
        } else if f2 < f32::from_bits(0xbecc_cccd) && m.r32(s1 + 0x18) == 0 && nz < f32::from_bits(0x3e99_999a) {
            to53c = true;
        }
    }
    if to53c {
        m.call(0x0019_eab0, &[s1, 4], &[]);
        return;
    }
    // L54c
    let d = sub4(m.rv(sp), m.rv(sp + 0x10));
    let vf4 = m.call(0x001c_4768, &[s2], &[d]).1;
    let x = m.rf(s2 + 0x10);
    let y = m.rf(s2 + 0x14);
    let f4v = m.rf(s2 + 0x1c);
    m.wv(sp + 0x20, [x, y, 0.0, 0.0]);
    let mut f3 = (vf4[0] * x + vf4[1] * y) + vf4[2] * 0.0;
    let f2z: f32 = 0.0;
    let st = m.r32(s1 + 0x20) as i32;
    if st == 0 {
        if f4v < 0.25 {
            m.w32(s1 + 0x20, (-1i32) as u32);
        } else if f3 < f32::from_bits(0xbf33_3333) {
            m.w32(s1 + 0x20, 1);
        }
    } else if st == 1 {
        if f4v < 0.25 {
            m.w32(s1 + 0x20, (-1i32) as u32);
        } else if f32::from_bits(0x3f33_3333) < f3 {
            m.w32(s1 + 0x20, 0);
        }
    } else if st == -1 {
        if f32::from_bits(0x3eb3_3333) < f4v {
            if f2z <= f3 {
                m.w32(s1 + 0x20, 0);
            } else {
                m.w32(s1 + 0x20, 1);
            }
        }
    }
    let v1 = m.r32(s1 + 0x20) as i32;
    let mut s2r: i32 = -1;
    let v1b;
    if v1 == 0 {
        v1b = m.r32(s1 + 0x18);
        f3 = f4v;
        s2r = if v1b != 0 { 3 } else { 2 };
    } else if v1 == 1 {
        v1b = m.r32(s1 + 0x18);
        f3 = f4v;
        s2r = if v1b != 0 { 2 } else { 3 };
    } else if v1 == -1 {
        s2r = 1;
        f3 = 0.0;
        v1b = m.r32(s1 + 0x18);
    } else {
        v1b = m.r32(s1 + 0x18);
    }
    let mut f20: f32;
    let f1: f32;
    if f32::from_bits(0x3c23_d70a) < f3 {
        f1 = m.rf(s1 + 0x6c);
        let f2 = m.rf(s1 + 0x5c);
        f20 = (1.0 - f3) * f2 + f3 * f1;
        if s2r == 3 {
            f20 = -f20;
        }
    } else {
        f20 = 0.0;
        s2r = 1;
        f1 = m.rf(s1 + 0x6c);
    }
    if v1b != 0 {
        f20 = -f20;
    }
    let f5 = f1.abs();
    let dt = m.rf(T + 4);
    let f4c = m.rf(s1 + 0x1c);
    m.wf(sp + 0x20, f5);
    m.wf(sp + 0x28, f32::from_bits(0x3e4c_cccd));
    m.w32(sp + 0x24, 0);
    let f0 = (-dt * f20) / f5 + (1.0 - dt) * f4c;
    m.wf(s1 + 0x1c, f0);
    let o = m.r32(s1 + 0xc);
    if m.r32(o + 0xe4) == 0 {
        m.call(0x0019_e110, &[s1, sp + 0x30], &[]);
        let a = m.rf(s1 + 0x10);
        let b = m.rf(sp + 0x30);
        let c = m.rf(T + 4);
        m.call(0x0011_d018, &[sp + 0x20, sp + 0x40], &[f4(a), f4(b), f4(c)]);
        let c = m.rf(T + 4);
        let a = m.rf(s1 + 0x10);
        let b = m.rf(sp + 0x34);
        m.call(0x0011_d018, &[sp + 0x20, sp + 0x44], &[f4(a), f4(b), f4(c)]);
        let g1 = -m.rf(sp + 0x44);
        let g0 = -m.rf(sp + 0x40);
        if f20 < g1 {
            f20 = g1;
        } else if g0 < f20 {
            f20 = g0;
        }
    }
    let f2 = -f20;
    m.wf(s1 + 0x14, f2);
    if f20 != 0.0 {
        let p = m.rf(s1 + 0x10) + f2 * m.rf(T + 4);
        let a = m.r32(s1);
        m.wf(s1 + 0x10, p);
        m.call(0x001f_4bf8, &[a], &[]);
    }
    let o = m.r32(s1 + 0xc);
    if m.r32(o + 0xe4) != 0 {
        let g = m.call(0x001e_6648, &[o], &[]).1[0];
        let p = m.rf(s1 + 0x10);
        let r = m.call(0x0011_d898, &[], &[f4(p), f4(g)]).1[0];
        m.wf(s1 + 0x10, r);
    }
    let f2 = f20.abs();
    if f2 < f32::from_bits(0x3c23_d70a) {
        s2r = 1;
    } else {
        let lo = m.rf(s1 + 0x5c).abs();
        let hi = m.rf(s1 + 0x6c).abs();
        let mut u = (f2 - lo) / (hi - lo);
        if u < 0.0 {
            u = 0.0;
        } else if 1.0 < u {
            u = 1.0;
        }
        let a = m.r32(s1 + 0x70);
        m.wf(s1 + 0x60, u);
        m.wf(s1 + 0x50, 1.0 - u);
        m.call(0x0013_84c0, &[a], &[]);
    }
    m.call(0x0019_eab0, &[s1, s2r as u32], &[]);
}
