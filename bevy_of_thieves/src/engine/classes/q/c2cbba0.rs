//! WP-Q8: C2cbba0 slot overrides (state +0x184, blend start +0x188, duration +0x180, colour +0x170).
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem};
use crate::engine::classes::q::q8util::*;

/// FUN_0013bdf0 (slot 6): parent init FUN_001f2108, +0x184 = -1, +0x170 = DAT_002a2de0 (1,1,1,0), flags |= 2
pub fn ee_sub_13bdf0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001f_2108, &[o], &[]);
    let v = rov(0x002a_2de0);
    m.w32(o + 0x184, u32::MAX);
    m.wv(o + 0x170, v);
    set_tagged(m, o + 0x40, 2);
}
/// FUN_0013be58 (slot 13, load): frame; +0x154 = FUN_001ae378(&{0x580, 0x10}); parent load FUN_001f31e0;
/// FUN_0013c440; +0x154 = FUN_001ae238(&{+0x150 * 0x2c, 0x10}, +0x154); frame close (2)
pub fn ee_sub_13be58<M: QMem + ?Sized>(m: &mut M, o: u32, stream: u32) {
    let fr = frame_begin(m);
    let req = stack_push(m, 8);
    m.w32(req, 0x580);
    m.w32(req + 4, 0x10);
    let v = m.call(0x001a_e378, &[req], &[]).0;
    m.w32(o + 0x154, v);
    m.call(0x001f_31e0, &[o, stream], &[]);
    m.call(0x0013_c440, &[o], &[]);
    let n = m.r32(o + 0x150).wrapping_mul(0x2c);
    m.w32(req, n);
    m.w32(req + 4, 0x10);
    let old = m.r32(o + 0x154);
    let v = m.call(0x001a_e238, &[req, old], &[]).0;
    m.w32(o + 0x154, v);
    stack_pop(m, 8);
    frame_end(m, fr, 2);
}
/// FUN_0013bf00: parent FUN_001f36a8, then FUN_0013c560(this, 0)
pub fn ee_sub_13bf00<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001f_36a8, &[o], &[]);
    m.call(0x0013_c560, &[o, 0], &[]);
}
/// FUN_0013bf30: FUN_0013c560(this, 0)
pub fn ee_sub_13bf30<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0013_c560, &[o, 0], &[]);
}
/// FUN_0013c000 (update): state 1 / 5 -> FUN_001f3c70; state 4: blend factor t = (CLOCK - start) / dur through
/// the cubic DAT_002a4b30.. clamped to DAT_002bbb60..64; colour e = (1 - t) + t * colour (xyz) or the plain
/// colour once t >= dur; basis FUN_001a6318 scaled by e (FUN_0011c210) at FUN_001a62e8 -> FUN_001a5ee8(1.0, 1.0, this, m)
/// PARTIAL: 4th lane of e is uninitialised stack in the original on the blending path (0 here).
pub fn ee_sub_13c000<M: QMem + ?Sized>(m: &mut M, o: u32) {
    match m.r32(o + 0x184) {
        1 | 5 => {
            m.call(0x001f_3c70, &[o], &[]);
        }
        4 => {
            let e = stack_push(m, 0x10);
            let mat = stack_push(m, 0x40);
            let mut t = m.rf(CLOCK) - m.rf(o + 0x188);
            let dur = m.rf(o + 0x180);
            if dur <= t {
                let c = m.rv(o + 0x170);
                m.wv(e, c);
            } else {
                t /= dur;
                let p = rof(0x002a_4b30) + t * (rof(0x002a_4b34) + t * rof(0x002a_4b38));
                let mut f = rof(0x002b_bb60);
                if f <= p {
                    f = rof(0x002b_bb64);
                    if p <= f {
                        f = p;
                    }
                }
                m.wv(e, [0.0; 4]);
                for i in 0..3u32 {
                    let c = m.rf(o + 0x170 + 4 * i);
                    m.wf(e + 4 * i, (1.0 - f) + f * c);
                }
            }
            let pos = m.call(0x001a_62e8, &[o], &[]).1;
            let basis = m.call(0x001a_6318, &[o], &[]).0;
            m.call(0x0011_c210, &[mat, basis, e], &[pos]);
            m.call(0x001a_5ee8, &[o, mat], &[super::q5util::f4(1.0), super::q5util::f4(1.0)]);
            stack_pop(m, 0x40);
            stack_pop(m, 0x10);
        }
        _ => {}
    }
}
/// FUN_0013c278: state 5 and FUN_00132b10(**(p2 + 0x14), 0x34) -> 3, else parent FUN_001f4fc8
pub fn ee_sub_13c278<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32) -> u32 {
    if m.r32(o + 0x184) == 5 {
        let q = m.r32(p2 + 0x14);
        let a = m.r32(q);
        if m.call(0x0013_2b10, &[a, 0x34], &[]).0 != 0 {
            return 3;
        }
    }
    m.call(0x001f_4fc8, &[o, p2], &[]).0
}
/// FUN_0013c2d8: parent FUN_001f4fb0; in state 5 with FUN_00132b10(p2[(1 - p3) * 0x40 + 8], 0x34) == 0:
/// FUN_00149ab8(world, this, DAT_002a4b58 (u64), 0)
pub fn ee_sub_13c2d8<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32, p3: u32) {
    m.call(0x001f_4fb0, &[o, p2, p3], &[]);
    if m.r32(o + 0x184) == 5 {
        let pa = p2.wrapping_add((1u32.wrapping_sub(p3)).wrapping_mul(0x40)).wrapping_add(8);
        let a = m.r32(pa);
        if m.call(0x0013_2b10, &[a, 0x34], &[]).0 == 0 {
            m.call(0x0014_9ab8, &[WORLD, o, ro32(0x002a_4b58), 0], &[]);
        }
    }
}
/// FUN_0013c368: FUN_001f7588 != 0 and state 5 -> FUN_0013c560(this, 0), 1; else 0
pub fn ee_sub_13c368<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let r = m.call(0x001f_7588, &[o], &[]).0;
    if r == 0 || m.r32(o + 0x184) != 5 {
        0
    } else {
        m.call(0x0013_c560, &[o, 0], &[]);
        1
    }
}
