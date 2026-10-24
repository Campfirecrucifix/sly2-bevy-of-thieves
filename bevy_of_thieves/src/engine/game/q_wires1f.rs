//! WIRE-S1: 1:1 QMem queue ports of unported callers (round 2).
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem};

/// FUN_001b93e8: SetState(obj, state) - state transition (0 idle / 1 grab+attach / 2 release links), syncs +0x1088 flag.
pub fn ee_sub_1b93e8<M: QMem + ?Sized>(m: &mut M, obj: u32, state: i32) {
    let cur = m.r32(obj + 0x1450) as i32;
    if cur == state {
        return;
    }
    if cur == 1 {
        m.call(0x0012_e460, &[obj + 0x13fc], &[]);
        m.call(0x0012_e460, &[obj + 0x1400], &[]);
    }
    m.w32(obj + 0x1450, state as u32);
    let c = m.rf(0x002d_ded0);
    m.wf(obj + 0x1454, c);
    let sp = stack_push(m, 0x90);
    'tail: {
        if state == 0 {
            m.w32(obj + 0x1448, 0);
            break 'tail;
        }
        if state != 1 && state != 2 {
            break 'tail;
        }
        let a0 = m.r32(obj + 0x13f0);
        let mut s3 = 0;
        let ok = a0 != 0 && {
            s3 = m.r32(a0 + 0x180);
            (m.r64(s3 + 0x28) >> 60) & 1 != 0
        };
        if !ok {
            m.call(0x001b_7d20, &[obj], &[]);
            m.w32(obj + 0x13f0, 0);
            m.call(0x001b_6078, &[obj, 0], &[]);
            break 'tail;
        }
        if state == 1 {
            m.call(0x001f_5720, &[s3, 0], &[]);
            let p = m.r32(s3 + 0x58);
            let h = m.r16(p + 0x98) as i16 as i32 as u32;
            m.w32(obj + 0x1404, h);
            let p = m.r32(s3 + 0x5c);
            let h = m.r16(p + 0x98) as i16 as i32 as u32;
            m.w32(obj + 0x1408, h);
            m.call(0x001a_58d0, &[s3, 0, 0xf], &[]);
            m.call(0x001f_6860, &[s3, 1], &[]);
            let p = m.r32(obj + 0x1030);
            let a = m.r32(p + 0x14);
            let s1 = m.call(0x0012_f938, &[a, 0x1c7], &[]).0;
            let a1 = m.r32(obj + 0x60);
            let v = m.r32(obj + 0x1030);
            let f = (m.rf(s1) - m.rf(v + 0x1c)) / m.rf(v + 0x40);
            let p = m.r32(obj + 0x13ec);
            let a = m.r32(p + 0x60);
            m.wf(sp + 0x80, f);
            m.call(0x0023_09e8, &[a, a1, sp, 0], &[[f, 0.0, 0.0, 0.0]]);
            m.call(0x001a_71b0, &[sp + 0x40, obj, 0, sp], &[]);
            for k in (0..0x40).step_by(4) {
                let w = m.r32(sp + 0x40 + k);
                m.w32(sp + k, w);
            }
            m.call(0x001b_72b0, &[obj, 0, sp, sp + 0x40, sp + 0x50], &[]);
            if m.call(0x0013_2b10, &[s3, 0x38], &[]).0 != 0 {
                m.call(0x0020_9e28, &[s3, obj], &[]);
            }
            let q50 = m.rv(sp + 0x50);
            m.call(0x001b_7700, &[obj, s1, sp + 0x80, sp + 0x60, sp + 0x70], &[q50]);
            let f12 = m.rf(0x002d_ded0) + m.rf(sp + 0x80);
            let q50 = m.rv(sp + 0x50);
            let q70 = m.rv(sp + 0x70);
            let a1 = m.r32(s3 + 0x58);
            let r = m.call(0x0012_de58, &[s3, a1, 5], &[q50, q70, [f12, 0.0, 0.0, 0.0]]).0;
            m.w32(obj + 0x13fc, r);
            let q60 = m.rv(sp + 0x60);
            let p = m.r32(s3 + 0x58);
            let a = m.r32(p + 0xa0);
            let r = m.call(0x0022_fcc8, &[a, 0], &[q60]).0;
            vslot(m, s3, 34, &[r], &[]);
            let q40 = m.rv(sp + 0x40);
            let a1 = m.r32(s3 + 0x5c);
            let r = m.call(0x0012_dc18, &[s3, a1, 5, 7, 2], &[q40]).0;
            m.w32(obj + 0x1400, r);
            break 'tail;
        }
        // state 2
        let s0 = m.r32(obj + 0x13ec);
        m.call(0x001a_58d0, &[s3, s0, 0xf], &[]);
        m.call(0x001f_6860, &[s3, 1], &[]);
        let a1 = m.r32(s0 + 0x60);
        m.call(0x001b_72b0, &[obj, a1, 0x002a_2d30, sp + 0x10, sp], &[]);
        let q = m.rv(sp + 0x10);
        let a1 = m.r32(s3 + 0x5c);
        let r = m.call(0x0012_dc18, &[s3, a1, 5, 7, 2], &[q]).0;
        m.w32(obj + 0x13f8, r);
        let q = m.rv(sp);
        let a1 = m.r32(s3 + 0x58);
        let r = m.call(0x0012_dc18, &[s3, a1, 5, 7, 2], &[q]).0;
        m.w32(obj + 0x13f4, r);
        for (lnk, fsrc, o1, o2, src, dst) in [
            (0x190u32, 0x002c_0a3cu32, 0x1610u32, 0x1618u32, 0x15f4u32, 0x1604u32),
            (0x198, 0x002c_0a40, 0x1614, 0x1620, 0x15fc, 0x160c),
        ] {
            let v0 = m.r32(obj + 0x13f0);
            let a2 = m.r32(v0 + lnk);
            if a2 == 0 {
                continue;
            }
            let s0 = m.r32(obj + o1);
            let r = vslot(m, a2, 31, &[sp + 0x20], &[]).0;
            m.w32(s0, r);
            let f = m.rf(fsrc);
            let q = m.rv(sp + 0x50);
            m.wf(s0 + 0xcc, f);
            m.wv(s0 + 0xa0, q);
            let a1 = m.r32(s0 + 4);
            let r = m.call(0x0012_dd28, &[obj, a1, 5, s0], &[]).0;
            m.w32(obj + o2, r);
            let a1 = m.r32(s0 + 8);
            let r = m.call(0x0012_dd28, &[obj, a1, 5, s0], &[]).0;
            m.w32(obj + o2 + 4, r);
            let a1 = m.r32(obj + src);
            let t0 = m.r32(s0);
            let r = m.call(0x0012_ddb0, &[obj, a1, 5, 5, t0, sp + 0x20], &[]).0;
            m.w32(obj + dst, r);
        }
        if m.call(0x0013_2b10, &[s3, 0x38], &[]).0 != 0 {
            m.call(0x0020_9e88, &[s3], &[]);
        }
        m.w32(obj + 0x194, 0);
        m.w32(obj + 0x198, 0);
        m.w32(obj + 0x190, 0);
    }
    let v1 = m.r32(obj + 0x1450) as i32;
    let p = m.r32(obj + 0x1088);
    m.w32(p + 0x1c, (0..2).contains(&v1) as u32);
    stack_pop(m, 0x90);
}
