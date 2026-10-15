//! WP-Q11 part B: class packages C2d0830, C2d09b0, C2d2e58, C2d3750, C2d37d0, C2d4af8, C2d7558, C2d75a8, C2d7648 ...
//! 1:1 from ghidra_functions.body on `QMem` (conventions as q11_a).
#![allow(unused_variables, unused_mut, dead_code)]
use crate::engine::classes::q::q11_a::dist3;
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};
use crate::engine::classes::q::q6::{ro32, rof, rov, s, vcall};
use crate::engine::classes::q::q7util::{add, f, fr, mul3, scale, sub, CLOCK};

const NEG1: u32 = u32::MAX;
const WORLD: u32 = 0x003d_d490;

/// Q11 overrides whose whole body is `return <const>;`: (fn, value)
pub const CONST_RET: &[(u32, u32)] = &[(0x114410, 0x2bb768)];
/// Q11 overrides that are `jr ra` only
pub const NOPS: &[u32] = &[0x1af388];

fn pos<M: QMem + ?Sized>(m: &mut M, obj: u32) -> V4 {
    m.call(0x001a_62e8, &[obj], &[]).1
}

// ---- C2d0830 ----------------------------------------------------------------------------------------

/// FUN_001795e8 (v9)
pub fn c2d0830_v9<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0017_4b68, &[o], &[]);
    let ow = m.r32(o + 0x10);
    let p = m.r32(ow + 0x35c);
    let q = pos(m, ow);
    m.wv(p + 0x20, q);
    let t = m.rf(CLOCK) + 0.0001;
    if m.rf(o + 0xb0) < t {
        m.w32(o + 0xb0, ro32(0x002a_5588));
        m.call(0x0014_f750, &[0x002d_df40, 0], &[]);
    }
    if m.rf(o + 0xb4) < t {
        m.w32(o + 0xb4, ro32(0x002a_5588));
        let ow = m.r32(o + 0x10);
        m.call(0x001f_7a68, &[ow, 2], &[]);
    }
}
/// FUN_00179780 (v7)
pub fn c2d0830_v7<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let a = m.r32(o + 0x5c);
    let i = m.call(0x001f_78a0, &[a], &[]).0;
    let ow = m.r32(o + 0x10);
    let p = ow.wrapping_add(i.wrapping_mul(0x10)).wrapping_add(0x8d0);
    m.w32(o + 0x40, p);
    if m.r32(p) == NEG1 {
        m.w32(o + 0x40, 0);
    }
    m.call(0x0017_9328, &[o], &[]);
    let ow = m.r32(o + 0x10);
    m.call(0x0016_19a0, &[0x002d_eb30, 0x15, ow], &[]);
    let ow = m.r32(o + 0x10);
    let (a, b) = (m.r32(ow + 0xf64), m.r32(o + 0x50));
    m.call(0x0015_4038, &[a, b, 2], &[]);
    let ow = m.r32(o + 0x10);
    let c = m.r32(ow + 0xfec);
    if c != 0 {
        vslot(m, c, 7, &[], &[]);
    }
}

// ---- C2d09b0 / C2d2e58 / C2d3750 ---------------------------------------------------------------------

/// FUN_00174920 (C2d09b0_v6)
pub fn c2d09b0_v6<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef68, &[o], &[]);
    m.w32(o + 0x3c, 0);
    m.w32(o + 0x30, NEG1);
}
/// FUN_00186610 (C2d2e58_v5)
/// VFIX: a1..a3 pass through untouched to FUN_0012cf88(o, src, a2, a3)
pub fn c2d2e58_v5<M: QMem + ?Sized>(m: &mut M, o: u32, src: u32, a2: u32, a3: u32) {
    m.call(0x0012_cf88, &[o, src, a2, a3], &[]);
    let p = m.r32(o + 8);
    let v = m.r32(p + 0xa8);
    m.w32(o + 0x1c, v);
}
/// FUN_00186640 (C2d2e58_v0)
pub fn c2d2e58_v0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0012_ce98, &[o], &[]);
    let p = m.r32(o + 8);
    let v = m.r32(o + 0x1c);
    m.w32(p + 0xa8, v);
}
/// FUN_001acae8 (C2d3750_v11)
pub fn c2d3750_v11<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001d_a770, &[o], &[]);
    let c = m.r32(o + 0x24);
    if c != 0 {
        m.w32(c + 0xb0, 1);
        if m.r32(o + 0x40) != NEG1 {
            m.call(0x001a_cb58, &[o], &[]);
        }
    }
}
/// FUN_001acb38 (C2d3750_v12)
pub fn c2d3750_v12<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001d_a7b0, &[o], &[]);
}

// ---- C2d4af8 ----------------------------------------------------------------------------------------

/// FUN_001ba608 (v9)
pub fn c2d4af8_v9<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef98, &[o], &[]);
    let st = loop {
        if m.call(0x001e_f0a8, &[o], &[]).0 != 0 {
            break m.r32(o + 0x30);
        }
        let s0 = m.r32(o + 0x30) as i32;
        if s0 == 1 {
            let ow = m.r32(o + 0x10);
            let iv = m.r32(ow + 0xf64);
            let tg = m.r32(0x002e_1e40);
            let p = pos(m, tg);
            let a = m.r32(iv + 0xe8);
            if m.call(0x0022_7c90, &[a], &[p]).0 == 0 {
                let ow = m.r32(o + 0x10);
                if m.rf(ow + 0x1088) < m.rf(CLOCK) - m.rf(o + 0x34) {
                    m.call(0x001b_a828, &[o], &[]);
                }
            } else {
                m.call(0x001b_a828, &[o], &[]);
            }
        } else if s0 < 2 && s0 == 0 {
            let ow = m.r32(o + 0x10);
            let iv = m.r32(ow + 0xf64);
            let tg = m.r32(0x002e_1e40);
            let p = pos(m, tg);
            let a = m.r32(iv + 0xec);
            if m.call(0x0022_7c90, &[a], &[p]).0 == 0 {
                m.call(0x001b_a828, &[o], &[]);
            }
        }
        if m.r32(o + 0x30) as i32 == s0 {
            break s0 as u32;
        }
    };
    let st = m.r32(o + 0x30);
    if st == 0 {
        let ow = m.r32(o + 0x10);
        if m.call(0x0020_9a68, &[ow, 9], &[]).0 != 0 {
            let a = pos(m, ow);
            let tg = m.r32(0x002e_1e40);
            let b = pos(m, tg);
            let d = dist3(a, b);
            let ow = m.r32(o + 0x10);
            let f7 = m.rf(ow + 0x1070) + d * (m.rf(ow + 0x1074) + d * m.rf(ow + 0x1078));
            let mut f8 = m.rf(ow + 0x1080);
            if m.rf(ow + 0x1080) <= f7 {
                f8 = m.rf(ow + 0x1084);
                if f7 <= m.rf(ow + 0x1084) {
                    f8 = f7;
                }
            }
            let ow = m.r32(o + 0x10);
            let l6 = m.call(0x0020_5458, &[ow, 0xa7], &[]).0;
            let l4 = m.call(0x0020_5458, &[ow, 0xb8], &[]).0;
            if l6 == 0 {
                if l4 != 0 {
                    m.wf(l4 + 0x38, f8);
                }
            } else {
                m.wf(l6 + 0xc4, f8);
            }
        }
    }
}

// ---- C2d7558 / C2d75a8 / C2d7648 ----------------------------------------------------------------------

/// FUN_001d0d28 (C2d7558_v6)
pub fn c2d7558_v6<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32) {
    if p2 != 0 && m.r32(o + 0xf4) != 0 {
        let sp = stack_push(m, 0x10);
        let q = m.rv(o + 0x70);
        m.wv(sp, q);
        let mut i3 = m.r32(o + 0xe0);
        let mut i = 0i32;
        if 0 < m.r32(i3 + 0xe0) as i32 {
            let mut off = 0u32;
            loop {
                let base = m.r32(i3 + 0xe4).wrapping_add(off);
                let (a, b) = (m.rv(o + 0x80), m.rv(o + 0x70));
                let l = m.call(0x0014_c6a0, &[base, 1, sp], &[a, b]).0;
                i += 1;
                if l != 0 {
                    break;
                }
                i3 = m.r32(o + 0xe0);
                off += 0xb0;
                if !(i < m.r32(i3 + 0xe0) as i32) {
                    break;
                }
            }
        }
        let l = m.call(0x001c_e888, &[WORLD, 7, 0], &[]).0;
        if l != 0 {
            let xy = [m.rf(sp), m.rf(sp + 4), 0.0, 0.0];
            vcall(m, l, 0xec, 0xc, &[0], &[f(40.0), xy]);
            m.w32(l + 0x18, 0x3f40_0000);
            m.w32(l + 0x20, 0x4220_0000);
            m.w32(l + 0x24, 0x41f0_0000);
            let v = m.r32(o + 0xf4);
            m.w32(l + 0x1c, v);
        }
        stack_pop(m, 0x10);
    }
    m.call(0x001c_ea88, &[o, p2], &[]);
}
/// FUN_001d0800 (C2d75a8_v4): calls a member-function pointer stored at (+0xf0)+0x60 on the object at +0xf4
pub fn c2d75a8_v4<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.rf(o + 0x18) < m.rf(CLOCK) - m.rf(o + 0x14) {
        let src = m.r32(o + 0xf0);
        let u = m.r64(src + 0x60);
        let delta = u as u16 as i16 as i32;
        let idx = (u >> 16) as u16 as i16 as i32;
        let hi = (u >> 32) as u32;
        let obj = m.r32(o + 0xf4);
        let (fnp, adj) = if idx < 0 {
            (hi, delta)
        } else {
            let vt = m.r32(obj.wrapping_add(hi as u16 as i16 as i32 as u32));
            let e = m.r64(vt.wrapping_add((idx * 8) as u32).wrapping_sub(8));
            ((e >> 32) as u32, (e as u16 as i16 as i32) + delta)
        };
        let arg = m.r32(o + 0xf8);
        m.call(fnp, &[obj.wrapping_add(adj as u32), arg], &[]);
        m.call(0x001c_e980, &[o], &[]);
    }
}
/// FUN_001cfcd8 (C2d7648_v0): (t, o)
/// VFIX: a1 (128-bit) and a2 are untouched pass-through args to FUN_001ce920 (asm: jal with only a0 set).
pub fn c2d7648_v0<M: QMem + ?Sized>(m: &mut M, o: u32, a1: V4, a2: u32, t: f32) {
    m.call(0x001c_e920, &[o, a2], &[a1, f(t)]);
    let q = rov(0x002b_bb50);
    m.wf(o + 0x20, t);
    m.w32(o + 0x28, 0);
    m.wv(o + 0x30, q);
    m.wf(o + 0x24, 1.0 - t);
}
/// FUN_001cfe88 (v3)
pub fn c2d7648_v3<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.rf(o + 0x14) <= m.rf(CLOCK) {
        let q = m.rv(o + 0x70);
        m.wv(o + 0x80, q);
        let mut t = (m.rf(o + 0x14) + m.rf(o + 0x18)) - m.rf(CLOCK);
        let dt = m.rf(CLOCK + 4);
        t = if t < dt { dt } else { t };
        let sp = stack_push(m, 0x80);
        let (b120, ba0) = (sp, sp + 0x40);
        let c = m.r32(o + 0x1c);
        let c = m.r32(c + 0x60);
        m.call(0x0023_09e8, &[c, 0, b120, 0], &[s(t)]);
        let target = m.rv(b120 + 0x30);
        let p70 = m.rv(o + 0x70);
        let v90 = m.rv(o + 0x90);
        let a0 = m.rv(o + 0xa0);
        // ACC = (target - pos) * (1.5 / t) + v90 * -0.5 + (a0 * t) * 0.25
        let inv = 1.5 / t;
        let acc: V4 = std::array::from_fn(|i| (target[i] - p70[i]) * inv + v90[i] * -0.5 + (a0[i] * t) * 0.25);
        m.call(0x0011_7120, &[o + 0x70, o + 0x90, 0], &[f(t), f(dt), f(1.0), p70, v90, target, acc]);
        let k = (m.rf(o + 0x100) * t) / m.rf(o + 0x18);
        let ax = m.rv(o + 0xf0);
        m.call(0x0011_b500, &[ba0], &[f(k), ax]);
        let r = [m.rv(ba0), m.rv(ba0 + 0x10), m.rv(ba0 + 0x20)];
        let b0 = m.rv(b120);
        let b1 = m.rv(b120 + 0x10);
        let b2 = m.rv(b120 + 0x20);
        let o0 = mul3(&r, b0, 0.0);
        let o1 = mul3(&r, b1, 0.0);
        let o2 = mul3(&r, b2, 0.0);
        m.wv(o + 0x40, o0);
        m.wv(o + 0x60, o2);
        m.wv(o + 0x50, o1);
        stack_pop(m, 0x80);
    }
}
/// FUN_001d0038 (v4)
pub fn c2d7648_v4<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.rf(o + 0x14) <= m.rf(CLOCK) && m.r32(o + 0x104) != 0 {
        m.call(0x001c_fd30, &[o], &[]);
    }
    if m.rf(o + 0x18) < m.rf(CLOCK) - m.rf(o + 0x14) {
        let c = m.r32(o + 0x1c);
        vslot(m, c, 7, &[], &[]);
        m.call(0x001c_e980, &[o], &[]);
    }
}
