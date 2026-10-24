//! WIRE-S1: 1:1 QMem queue ports of unported callers (round 2).
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

/// GPR -> quad of a 32-bit word (lw/mfc1 sign-extend into the low 64 bits); the stale upper 64 bits of the
/// GPR are modelled as 0 (they only reach dead scratch stack slots)
fn wq(bits: u32) -> V4 {
    let hi = if (bits as i32) < 0 { 0xffff_ffffu32 } else { 0 };
    [f32::from_bits(bits), f32::from_bits(hi), 0.0, 0.0]
}
fn fq(f: f32) -> V4 {
    wq(f.to_bits())
}
fn vsub(a: V4, b: V4) -> V4 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2], a[3] - b[3]]
}
fn vadd(a: V4, b: V4) -> V4 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2], a[3] + b[3]]
}
fn vmuls(a: V4, s: f32) -> V4 {
    [a[0] * s, a[1] * s, a[2] * s, a[3] * s]
}
/// vadda ACC, a, vf00 ; vmaddx d, b, s  (xyzw; vf00.w = 1 adds 1 to w)
fn vmadd(a: V4, b: V4, s: f32) -> V4 {
    [a[0] + b[0] * s, a[1] + b[1] * s, a[2] + b[2] * s, (a[3] + 1.0) + b[3] * s]
}
/// vopmula.xyz ACC, a, b ; vopmsub.xyz d, b, a  -> d.xyz = a x b (d.w kept)
fn vcross(d: V4, a: V4, b: V4) -> V4 {
    [a[1] * b[2] - b[1] * a[2], a[2] * b[0] - b[2] * a[0], a[0] * b[1] - b[0] * a[1], d[3]]
}
/// vmul.xyz t, a, b ; vaddw.x one ; vadday.x ACC, t, ty ; vmaddz.x t, one, tz  -> (x*x+y*y)+z*z
fn vdot(a: V4, b: V4) -> f32 {
    let t = [a[0] * b[0], a[1] * b[1], a[2] * b[2]];
    (t[0] + t[1]) + 1.0 * t[2]
}
/// vsqrt Q, x (|x|)
fn vsqrt(x: f32) -> f32 {
    x.abs().sqrt()
}

/// FUN_0018fc60: steer/aim update for the s1 object: builds the target frame from its +0x18c4 partner
/// (positions, look basis, damped turn/pitch rates), integrates the velocity at +0xf0 into the new goal
/// and either hands it to FUN_0018f958 (with +0x1900 target) or stores the goal delta at +0x1910.
pub fn ee_sub_18fc60<M: QMem + ?Sized>(m: &mut M, s1: u32) {
    let sp = stack_push(m, 0x2a0);
    let eps = f32::from_bits(0x38d1_b717);
    let p = m.r32(s1 + 0x18c4);
    let v = m.call(0x001a_62e8, &[p], &[]).1;
    let p = m.r32(s1 + 0x18c4);
    m.wv(sp, v);
    let mp = m.call(0x001a_6318, &[p], &[]).0;
    let q = m.rv(mp);
    m.wv(sp + 0x40, q);
    let q = m.rv(mp + 0x10);
    m.wv(sp + 0x50, q);
    let q = m.rv(mp + 0x20);
    m.wv(sp + 0x60, q);
    let p = m.r32(s1 + 0x18c4);
    let r = vslot(m, p, 31, &[0], &[]).0;
    let q0 = m.rv(sp);
    let r = m.call(0x0022_d348, &[r, 0], &[q0]).1;
    m.wv(sp + 0x20, r);
    let vf2 = m.call(0x001a_62e8, &[s1], &[]).1;
    let vf1 = m.rv(sp);
    let vf3 = vsub(vf2, vf1);
    let d = vdot(vf3, vf3);
    m.wv(sp + 0x30, vf3);
    let f22 = vsqrt(d);
    m.wv(sp + 0x10, vf3);
    if f22 < eps {
        let q = m.rv(0x002a_2dc0);
        m.wv(sp + 0x30, q);
    } else {
        m.wv(sp + 0x30, vmuls(vf3, 1.0 / f22));
    }
    let vf3 = vslot(m, s1, 32, &[], &[]).1;
    let a1 = m.rv(sp + 0x30);
    let vf1 = m.rv(sp + 0x20);
    let vf2 = vsub(vf3, vf1);
    m.wv(sp + 0x70, vf2);
    let f23 = m.call(0x0011_e290, &[sp + 0x90, sp + 0x80], &[vf2, a1]).1[0];
    let vf2 = m.rv(sp + 0x30);
    let vf1 = m.rv(sp + 0x80);
    let vf3 = vcross(vf3, vf2, vf1);
    let l = vsqrt(vdot(vf3, vf3));
    let mut vf4 = m.rv(0x002a_2da0);
    if !(l < eps) {
        vf4 = vmuls(vf3, 1.0 / l);
    }
    let vf1 = m.rv(sp + 0x80);
    let l2 = vsqrt(vdot(vf1, vf1));
    m.wv(sp + 0xa0, vf4);
    let vf3 = m.rv(sp + 0x30);
    let v1 = m.r32(s1 + 0xe8);
    let f4 = l2 / f22;
    let vf1 = fq(f4);
    let vf2 = vmuls(vf4, vf1[0]);
    m.wv(sp + 0xc0, vf1);
    m.wv(sp + 0xb0, vf2);
    let vf2 = m.rv(v1 + 0x80);
    let dd = vdot(vf2, vf3);
    let qv0 = m.rv(0x002a_2db0);
    let vf1 = vmuls(vf3, dd);
    let vf2 = vsub(vf2, vf1);
    let mut vf4 = vf4;
    vf4[0] = 1.0;
    let vf4 = vcross(vf4, vf3, vf2);
    m.wv(sp + 0xc0, vf2);
    let l3 = vsqrt(vdot(vf4, vf4));
    let mut vf3 = qv0;
    if !(l3 < eps) {
        vf3 = vmuls(vf4, 1.0 / l3);
    }
    let vf1 = m.rv(sp + 0xc0);
    let l4 = vsqrt(vdot(vf1, vf1));
    let f1 = f4 * m.rf(0x002b_f4f8);
    let f0 = l4 / f22;
    let f3 = m.rf(0x002b_f4f0);
    let f2 = m.rf(0x002b_f4f4) + f1;
    let f5 = m.rf(0x002b_f500);
    let f1 = m.rf(0x002b_f504);
    let f2 = f4 * f2;
    m.wv(sp + 0xd0, vf3);
    let vf2 = fq(f0);
    let f0 = f3 + f2;
    let vf1 = vmuls(vf3, vf2[0]);
    m.wv(sp + 0xe0, vf1);
    m.wv(sp + 0xf0, vf2);
    let f20 = if f0 < f5 {
        f5
    } else if f1 < f0 {
        f1
    } else {
        f0
    };
    let v1 = m.r32(s1 + 0x18d0);
    let f21 = if v1 == 0 || v1 == 2 { m.rf(0x002b_f508) } else { m.rf(0x002b_f50c) };
    let vf1 = m.rv(sp + 0xe0);
    let vf2 = m.rv(sp + 0xb0);
    let a1 = m.rv(sp + 0x50);
    let vf3 = wq(m.r32(0x002d_ded4));
    let vf2 = vmadd(vf2, vf1, vf3[0]);
    m.wv(sp + 0x120, vf3);
    m.wv(sp + 0xf0, vf2);
    m.call(0x0011_e290, &[sp + 0x100, sp + 0x110], &[vf2, a1]);
    let f3 = m.rf(0x002d_ded4);
    let f1 = f20 * f3 + 1.0;
    let v1 = m.r32(s1 + 0x18d0);
    let f0 = f21 * f3 + 1.0;
    let vf4 = m.rv(sp + 0x100);
    let vf3 = m.rv(sp + 0x110);
    let vf1 = fq(f1);
    let vf2 = fq(f0);
    m.wv(sp + 0x120, vf1);
    let vf1 = [
        vf4[0] * vf1[0] + vf3[0] * vf2[0],
        vf4[1] * vf1[0] + vf3[1] * vf2[0],
        vf4[2] * vf1[0] + vf3[2] * vf2[0],
        vf4[3] * vf1[0] + vf3[3] * vf2[0],
    ];
    m.wv(sp + 0x120, vf2);
    m.wv(sp + 0xf0, vf1);
    if v1 == 0 || v1 == 2 {
        let f0 = m.rf(0x002d_ded0) - m.rf(s1 + 0x18d4);
        if f0 < m.rf(0x002b_f510) {
            let mut f6 = f3 * m.rf(0x002b_f514);
            if v1 == 0 {
                f6 = -f6;
            }
            if m.r32(s1 + 0x18fc) != 0 {
                f6 = -f6;
            }
            let v0 = m.r32(s1 + 0xe8);
            let vf1 = m.rv(sp + 0x80);
            let l5 = vsqrt(vdot(vf1, vf1));
            let f5 = -m.rf(v0 + 0x88);
            let f1 = l5 * 0.5;
            let f0 = f5 * m.rf(sp + 0x18);
            let f4 = m.rf(0x002b_f528);
            let f2 = m.rf(0x002b_f524);
            let f3 = l5 * f1;
            let f7 = m.rf(0x002b_bb60);
            let f1 = m.rf(0x002b_f520);
            let f0 = (f0 + f3) / f5;
            let f4 = f0 * f4;
            let f2 = f2 + f4;
            let f0 = f0 * f2;
            let mut f1 = f1 + f0;
            let f8 = m.rf(0x002b_bb64);
            if f1 < f7 {
                f1 = f7;
            } else if f8 < f1 {
                f1 = f8;
            }
            let f6 = f6 * f1;
            let vf1 = m.rv(sp + 0xf0);
            let vf2 = m.rv(sp + 0x50);
            let vf3 = fq(f6);
            let vf1 = vmadd(vf1, vf2, vf3[0]);
            m.wv(sp + 0x120, vf3);
            m.wv(sp + 0xf0, vf1);
        }
    }
    let v0 = m.r32(s1 + 0x18c4);
    let f14 = m.rf(0x002b_f544) - m.rf(v0 + 0x134);
    let f15 = m.rf(0x002d_ded4);
    let f20 = m.call(0x0011_d1c0, &[0x002b_f530, sp + 0x1d0], &[[f22; 4], [f23; 4], [f14; 4], [f15; 4]]).1[0];
    let f1 = m.rf(0x002b_f544).min(f22);
    let f0 = m.rf(0x002b_f540).max(f20);
    let vf2 = m.rv(sp + 0xf0);
    let v1 = m.r32(s1 + 0x1900);
    let vf1 = fq(f1 / f0);
    let vf2 = vmuls(vf2, vf1[0]);
    m.wv(sp + 0x120, vf1);
    m.wv(sp + 0xf0, vf2);
    if v1 != 0 {
        let a0 = m.r32(v1 + 4);
        let v1q = wq(m.r32(0x002d_ded4));
        m.wv(sp + 0x1f0, v1q);
        let vf1 = m.rv(sp);
        let vf2 = m.rv(0x002a_2dd0);
        let v0q = m.rv(sp + 0x20);
        m.wv(sp + 0x200, vf1);
        m.wv(sp + 0x130, vf2);
        m.wv(sp + 0x140, vf2);
        m.wv(sp + 0x210, vf2);
        m.wv(sp + 0x1e0, v0q);
        m.wv(sp + 0x150, v1q);
        let vf3 = m.call(0x001a_62e8, &[a0], &[]).1;
        let vf1 = m.rv(sp + 0x200);
        let vf2 = m.rv(sp + 0x210);
        let vf1 = vsub(vf1, vf3);
        let vf1 = vcross(vf1, vf2, vf1);
        let vf4 = m.rv(sp + 0x1e0);
        let vf5 = m.rv(sp + 0x1f0);
        let acc = vmadd(vf4, vf2, vf5[0]); // ACC = vf4 + vf00 + vf2*x
        let vf1 = [acc[0] + vf1[0] * vf5[0], acc[1] + vf1[1] * vf5[0], acc[2] + vf1[2] * vf5[0], acc[3] + vf1[3] * vf5[0]];
        m.wv(sp + 0x120, vf1);
    } else {
        let q = m.rv(sp + 0x20);
        m.wv(sp + 0x120, q);
    }
    let f0 = m.rf(0x002d_ded4) * 0.5;
    let vf1 = m.rv(sp + 0xf0);
    let vf2 = vadd(m.rv(sp + 0xb0), vf1);
    let vf3 = fq(f0);
    let vf1 = vmuls(vf2, vf3[0]);
    m.wv(sp + 0x1c0, vf3);
    m.call(0x0011_b570, &[sp + 0x190], &[vf1]);
    let vf10 = m.rv(sp + 0x190);
    let vf9 = m.rv(sp + 0x1a0);
    let vf8 = m.rv(sp + 0x1b0);
    let vf7 = fq(f20);
    let mut vf4 = m.rv(sp + 0x30);
    let (x, y, z) = (vf4[0], vf4[1], vf4[2]);
    for k in 0..3 {
        vf4[k] = (vf10[k] * x + vf9[k] * y) + vf8[k] * z;
    }
    let vf11 = wq(m.r32(sp + 0x1d0));
    let vf6 = vmuls(vf4, vf7[0]);
    let vf2 = m.rv(sp + 0xf0);
    let vf2 = vcross(vf2, vf2, vf6);
    let vf3 = m.rv(sp + 0x120);
    let vf5 = vmuls(vf4, vf11[0]);
    let vf3 = vadd(vf3, vf2);
    let vf1 = m.rv(sp);
    let vf3 = vadd(vf3, vf5);
    let vf1 = vadd(vf1, vf6);
    m.wv(sp + 0x140, vf1);
    m.wv(sp + 0x150, vf3);
    m.wv(sp + 0x160, vf10);
    m.wv(sp + 0x170, vf9);
    m.wv(sp + 0x180, vf8);
    m.wv(sp + 0x130, vf4);
    m.wv(sp + 0x190, vf7);
    m.wv(sp + 0x220, vf3);
    m.wv(sp + 0x190, vf11);
    let vf1 = vslot(m, s1, 32, &[], &[]).1;
    let vf2 = wq(m.r32(0x002d_ded4));
    let v1 = m.r32(s1 + 0x1900);
    m.wv(sp + 0x190, vf2);
    let vf3 = vsub(m.rv(sp + 0x220), vf1);
    let v0 = m.r32(s1 + 0xe8);
    let vf2 = vmuls(vf3, 1.0 / vf2[0]);
    let vf1 = vsub(vf2, m.rv(v0 + 0x80));
    m.wv(sp + 0xc0, vf1);
    if v1 != 0 {
        let a1 = m.r32(v1 + 4);
        m.call(0x0018_f958, &[s1, a1, sp, sp + 0xc0, s1 + 0x1910], &[]);
    } else {
        m.wv(s1 + 0x1910, vf1);
    }
    stack_pop(m, 0x2a0);
}
