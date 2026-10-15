//! WP-Q14 part M: FUN_0017b1a8 (NPC state change: leave / enter side effects, then per-state entry actions).
#![allow(unused_variables, unused_mut, dead_code)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem, V4};

fn rr<M: QMem + ?Sized>(m: &mut M, a: u32, off: u32) -> u32 {
    let p = m.r32(a);
    m.r32(p + off)
}
/// FUN_0017b1a8(obj, new): state at +0x50, owner +0x10, anim record +0x40, aux +0x5c.
/// VU0 owner-position block ported via scalar decomposition of qmtc2/vsub/vmulabc/vmaddbc.
pub fn ee_sub_17b1a8<M: QMem + ?Sized>(m: &mut M, obj: u32, new: u32) -> u32 {
    let old = m.r32(obj + 0x50);
    if new == old {
        return 0;
    }
    let owner = m.r32(obj + 0x10);
    if old == 0xffff_ffff {
        let f = { let p = m.r32(owner + 0xe8); let a = m.rf(p + 0xd0); let b = m.rf(0x002b_e6fc); a * b };
        m.call(0x001f_35c8, &[], &[[f, 0.0, 0.0, 0.0]]);
    } else if old == 3 && new != 0xffff_ffff {
        let t = rr(m, obj + 0x40, 8);
        m.call(0x0013_0d30, &[0, 0x3f80_0000, t], &[]);
    }
    let cur = m.r32(obj + 0x50);
    if cur < 2 && new >= 2 {
        m.call(0x0012_e460, &[obj + 100], &[]);
        let o = m.r32(obj + 0x10);
        let (a, b) = (m.r32(o + 0x58), m.r32(o + 0x5c));
        m.call(0x0023_03a0, &[o, a, 2], &[]);
        m.call(0x0023_03a0, &[o, b, 2], &[]);
    }
    let cur = m.r32(obj + 0x50);
    let fa = cur.wrapping_sub(4) < 2 || cur == 0;
    let fb = new.wrapping_sub(4) < 2 || new == 0;
    m.w32(obj + 0x50, new);
    if fa != fb {
        let o = m.r32(obj + 0x10);
        m.call(0x0013_4320, &[0x003d_d490, o], &[]);
        m.call(0x001f_4e28, &[o], &[]);
        m.w32(obj + 0x50, new);
    }
    let t = m.rf(0x002d_ded0);
    m.wf(obj + 0x54, t);
    if new.wrapping_add(1) < 7 {
        match new as i32 {
            0 => {
                let sp = stack_push(m, 0x30);
                let o = m.r32(obj + 0x5c);
                let vt = m.r32(o + 0xc);
                let this = o.wrapping_add(m.r16(vt + 0x300) as i16 as i32 as u32);
                let f = m.rf(0x002b_e700);
                let fp = m.r32(vt + 0x304);
                m.call(fp, &[this, sp], &[[f, 0.0, 0.0, 0.0]]);
                m.call(0x0017_aa78, &[obj, sp, sp + 0x10, sp + 0x20], &[]);
                let (e0, qx, ax) = (m.r32(sp), m.rf(sp + 0x10), m.rf(sp + 0x20));
                let owner = m.r32(obj + 0x10);

                // VU0: owner position via FUN_001a62e8, direction = contact_pt - owner_pos,
                // scale by DAT_002be704 / DAT_002be700, offset by DAT_002a2dc0 * DAT_002be708,
                // then call owner vtable +0x11c
                {
                    let be700 = m.rf(0x002b_e700);
                    let scale = m.rf(0x002b_e704) / be700;
                    let bias  = m.rf(0x002b_e708);
                    // contact point from 0x17aa78 output (sp+0x10..sp+0x1c = b0..a4)
                    let cp = [m.rf(sp + 0x10), m.rf(sp + 0x14), m.rf(sp + 0x18), m.rf(sp + 0x1c)];
                    // owner world position via FUN_001a62e8
                    let ow = m.call(0x001a_62e8, &[owner], &[]).1;
                    // dir = contact_point - owner_pos
                    let dx = cp[0] - ow[0];
                    let dy = cp[1] - ow[1];
                    let dz = cp[2] - ow[2];
                    let dw = cp[3] - ow[3];
                    // _DAT_002a2dc0 is the zero vector (VU0 accumulator init)
                    // result = dir * scale + 0 * bias = dir * scale
                    let rx = dx * scale;
                    let ry = dy * scale;
                    let rz = dz * scale;
                    let rw = dw * scale;
                    let ovt = m.r32(owner + 0xc);
                    let ofs = m.r16(ovt + 0x118) as i16 as i32 as u32;
                    let fp  = m.r32(ovt + 0x11c);
                    m.call(fp, &[owner.wrapping_add(ofs)], &[[rx, ry, rz, rw]]);
                }

                let ow58 = m.r32(owner + 0x58);
                if m.r32(obj + 100) == 0 {
                    let fv = m.rf(0x002d_ded0) + m.rf(0x002b_e700);
                    let a0 = m.r32(ow58 + 0xa0);
                    let r1 = m.call(0x0022_fc48, &[0, a0], &[[qx, 0.0, 0.0, 0.0]]).1;
                    let r2 = m.call(0x0022_fcc8, &[0, a0], &[[ax, 0.0, 0.0, 0.0]]).1;
                    let h = m.call(0x0012_de58, &[owner, ow58, 0], &[[fv, 0.0, 0.0, 0.0], r1, r2]).0;
                    m.w32(obj + 100, h);
                }
                let v = if e0 != 0 { (rr(m, obj + 0x5c, 0x10dc) == owner) as u32 } else { 0 };
                m.w32(obj + 0x60, v);
                stack_pop(m, 0x30);
                if m.r32(owner + 0x334) == 0 {
                    return owner;
                }
                let r = m.call(0x001c_4840, &[0x003d_d490], &[]).0;
                if r == 0 {
                    return r;
                }
                let aux = m.r32(obj + 0x5c);
                if aux != r {
                    return aux;
                }
                return m.call(0x0018_7e28, &[0x3e19_999a, 0x002d_fc00, 3], &[]).0;
            }
            3 => {
                let a1 = rr(m, obj + 0x40, 4);
                let r = m.call(0x0012_f938, &[a1, 0x3c], &[]).0;
                if r != 0 {
                    let (a, t) = ({ let x = m.r32(r); x }, rr(m, obj + 0x40, 8));
                    return m.call(0x0013_0d30, &[a, 0x3f80_0000, t, 0], &[]).0;
                }
            }
            4 | 5 => {
                if m.r32(obj + 0x50) != 4 {
                    return 4;
                }
                let a1 = rr(m, obj + 0x40, 4);
                let r = m.call(0x0012_f938, &[a1, 0x173], &[]).0;
                if r != 0 {
                    let (a, t) = ({ let x = m.r32(r); x }, rr(m, obj + 0x40, 8));
                    m.call(0x0013_0d30, &[a, 0x3f80_0000, t, 0], &[]);
                }
            }
            -1 => {
                let f = { let p = m.r32(owner + 0xe8); let a = m.rf(p + 0xd0); let b = m.rf(0x002b_e6fc); a / b };
                m.call(0x001f_35c8, &[], &[[f, 0.0, 0.0, 0.0]]);
                return m.call(0x0017_49c0, &[obj, 1], &[]).0;
            }
            _ => {}
        }
    }
    0
}
