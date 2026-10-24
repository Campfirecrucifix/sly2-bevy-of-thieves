//! WIRE-S1: 1:1 QMem ports of WIRE-ROOTS live-frontier roots (round 3).

use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem};

fn cp16<M: QMem + ?Sized>(m: &mut M, dst: u32, src: u32) {
    for k in (0..16).step_by(4) {
        let w = m.r32(src + k);
        m.w32(dst + k, w);
    }
}

fn cp4<M: QMem + ?Sized>(m: &mut M, dst: u32, src: u32) {
    let w = m.r32(src);
    m.w32(dst, w);
}

/// FUN_001fb8e0(p): state-flag update: slot 0, +0x50 flag dispatch (slots 15/5/4/6/13/12/10/11)
/// with FUN_001fb658 flag clears, then slot 1.
pub fn ee_sub_1fb8e0<M: QMem + ?Sized>(m: &mut M, p: u32) {
    vslot(m, p, 0, &[], &[]);
    if m.r16(p + 0x50) & 0x200 != 0 {
        vslot(m, p, 15, &[], &[]);
    } else {
        let f = m.rf(p + 0x10);
        m.wf(p + 0x44, f);
    }
    let fl = m.r16(p + 0x50) as u32;
    if fl & 1 != 0 {
        vslot(m, p, 5, &[], &[]);
        let r = vslot(m, p, 4, &[], &[]).0;
        if r == 1 {
            m.call(0x001f_b658, &[p, 0x400, 0x400], &[]);
        }
    } else if m.r32(p + 0x40) != 0 {
        if fl & 8 != 0 {
            vslot(m, p, 6, &[], &[]);
            m.w32(p + 0x40, 0);
            m.call(0x001f_b658, &[p, 0x1dff, 0], &[]);
        } else {
            if fl & 0x1c00 != 0 {
                vslot(m, p, 13, &[], &[]);
            }
            if m.r16(p + 0x52) == 0 {
                if m.r16(p + 0x50) & 0x1f0 != 0 {
                    vslot(m, p, 12, &[], &[]);
                    m.call(0x001f_b658, &[p, 0x1f0, 0], &[]);
                }
                let fl = m.r16(p + 0x50) as u32;
                if fl & 2 != 0 {
                    vslot(m, p, 10, &[], &[]);
                    m.call(0x001f_b658, &[p, 2, 0], &[]);
                } else if fl & 4 != 0 {
                    vslot(m, p, 11, &[], &[]);
                    m.call(0x001f_b658, &[p, 4, 0], &[]);
                }
            }
        }
    } else if m.r16(p + 0x52) == 0 && fl & 0x200 == 0 {
        m.call(0x001f_b658, &[p, 0x1fff, 0], &[]);
    }
    vslot(m, p, 1, &[], &[]);
}

fn dist3(a: [f32; 4]) -> f32 {
    let x = a[0] * a[0];
    let y = a[1] * a[1];
    let z = a[2] * a[2];
    (x + y + 1.0 * z).sqrt()
}

/// FUN_00186820(p): if +0x148 bit 36, for each of +0x50 cells (+0x54, 0x100 stride) walk the +0x90 list
/// and for type-7 nodes store distances of the transformed point to its two endpoints (+0xc4 / +0xc0).
pub fn ee_sub_186820<M: QMem + ?Sized>(m: &mut M, p: u32) {
    if m.r64(p + 0x148) & (0x8000u64 << 21) == 0 {
        return;
    }
    let mut i: i32 = 0;
    while i < m.r32(p + 0x50) as i32 {
        let base = m.r32(p + 0x54);
        let mut n = m.r32(base.wrapping_add((i as u32) << 8) + 0x90);
        while n != 0 {
            if m.r32(n) == 7 {
                let o = m.r32(n + 0x1c);
                let x = m.r32(o + 0xc);
                let a2 = m.r32(o + 8);
                let mut vf2 = m.rv(x + 0x40);
                let vf4 = m.rv(x + 0x60);
                let vf6 = m.rv(x + 0x70);
                let vf5 = m.rv(x + 0x50);
                let vf3 = m.rv(o + 0xb0);
                let mut vf1 = m.rv(a2 + 0x70);
                for k in 0..3 {
                    vf2[k] = vf2[k] * vf3[0] + vf5[k] * vf3[1] + vf4[k] * vf3[2] + vf6[k] * 1.0;
                }
                let a1 = m.r32(o + 4);
                for k in 0..4 {
                    vf2[k] -= vf1[k];
                }
                let d = dist3(vf2);
                let vf3 = m.rv(a1 + 0x70);
                for k in 0..4 {
                    vf1[k] -= vf3[k];
                }
                m.wf(o + 0xc4, d);
                let d = dist3(vf1);
                m.wf(o + 0xc0, d);
            }
            n = m.r32(n + 0x10);
        }
        i += 1;
    }
}

/// byte lerp of FUN_002188b0 (rounding toward the larger end by 0.99999)
fn lerp_byte(a: u8, b: u8, t: f32) -> u8 {
    const E: f32 = f32::from_bits(0x3f7f_be77);
    let (a, b) = (a as i32, b as i32);
    let v = if a == b {
        b
    } else if a < b {
        a + (t * ((b as f32 + E) - a as f32)) as i32
    } else {
        b + ((1.0 - t) * ((a as f32 + E) - b as f32)) as i32
    };
    v as u8
}

fn lerp_rgba<M: QMem + ?Sized>(m: &mut M, sp: u32, t: f32) {
    for k in 0..4 {
        let a = m.r8(sp + 0x70 + k);
        let b = m.r8(sp + 0x80 + k);
        m.w8(sp + 0x90 + k, lerp_byte(a, b, t));
    }
}

/// FUN_002188b0(o, idx, xf, c0, c1, t): update entry idx's 0x70-stride items: keyed 2x3 transform/color
/// lerp at time +0xc, apply xf, FUN_00218790 combine, FUN_00215850 apply when bound, store back.
pub fn ee_sub_2188b0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, t0: u32, f12: f32) {
    let sp = stack_push(m, 0xb0);
    let tbl = m.r32(a0 + 0x10);
    let e = a1.wrapping_mul(0x18).wrapping_add(m.r32(tbl + 8));
    let cnt = m.r32(e) as i32;
    let time = m.rf(e + 0xc);
    let items = m.r32(e + 4);
    m.w32(sp + 0xa0, t0);
    let mut i: i32 = 0;
    while i < cnt {
        let s1 = (i as u32).wrapping_mul(0x70).wrapping_add(items);
        i += 1;
        if m.r32(s1 + 0x30) == 0 {
            m.wf(s1 + 0x5c, time);
            m.call(0x0011_a058, &[s1 + 0x68], &[]);
            continue;
        }
        let k1 = m.r32(s1 + 0x54);
        if m.r8(s1 + 0x60) != 0 && k1 != 0 {
            let f1 = m.rf(s1 + 0x38);
            let f0 = m.rf(s1 + 0x48);
            let t = (time - f1) / (f0 - f1);
            let u = 1.0 - t;
            let k0 = m.r32(s1 + 0x44);
            for k in 0..6 {
                let x = m.rf(k0 + 4 * k);
                let y = m.rf(k1 + 4 * k);
                m.wf(sp + 0x80 + 4 * k, u * x + t * y);
            }
            cp16(m, sp, sp + 0x80);
            cp16(m, sp + 0x70, sp + 0x90);
            cp16(m, sp + 0x60, sp + 0x80);
            cp16(m, sp + 0x10, sp + 0x90);
            cp4(m, sp + 0x70, s1 + 0x3c);
            cp4(m, sp + 0x80, s1 + 0x4c);
            lerp_rgba(m, sp, t);
            cp4(m, sp + 0x60, sp + 0x90);
            cp4(m, sp + 0x50, sp + 0x60);
            cp4(m, sp + 0x70, s1 + 0x40);
            cp4(m, sp + 0x80, s1 + 0x50);
            lerp_rgba(m, sp, t);
            cp4(m, sp + 0x60, sp + 0x90);
            cp4(m, sp + 0x40, sp + 0x60);
        } else {
            let k0 = m.r32(s1 + 0x44);
            cp16(m, sp, k0);
            cp16(m, sp + 0x10, k0 + 0x10);
            cp4(m, sp + 0x50, s1 + 0x3c);
            cp4(m, sp + 0x40, s1 + 0x40);
        }
        let x0 = m.rf(a2);
        let x1 = m.rf(a2 + 4);
        let x2 = m.rf(a2 + 8);
        let x3 = m.rf(a2 + 0xc);
        let x4 = m.rf(a2 + 0x10);
        let x5 = m.rf(a2 + 0x14);
        let s0 = m.rf(sp);
        let s1f = m.rf(sp + 4);
        let s2f = m.rf(sp + 8);
        let s3 = m.rf(sp + 0xc);
        let s4 = m.rf(sp + 0x10);
        let s5 = m.rf(sp + 0x14);
        m.wf(sp + 0x80, s0 * x0 + s1f * x2);
        m.wf(sp + 0x84, s0 * x1 + s1f * x3);
        m.wf(sp + 0x88, s2f * x0 + s3 * x2);
        m.wf(sp + 0x8c, s2f * x1 + s3 * x3);
        m.wf(sp + 0x90, s4 * x0 + s5 * x2 + x4);
        m.wf(sp + 0x94, s4 * x1 + s5 * x3 + x5);
        cp16(m, sp, sp + 0x80);
        cp16(m, sp + 0x60, sp + 0x80);
        cp16(m, sp + 0x10, sp + 0x90);
        cp16(m, sp + 0x70, sp + 0x90);
        let c = m.r32(sp + 0xa0);
        cp4(m, sp + 0x60, c);
        cp4(m, sp + 0x70, a3);
        cp4(m, sp + 0x80, sp + 0x40);
        cp4(m, sp + 0x90, sp + 0x50);
        m.call(0x0021_8790, &[sp + 0x60, sp + 0x70, sp + 0x80, sp + 0x90, sp + 0x30, sp + 0x20], &[]);
        let b = m.r32(s1 + 0x30);
        if b != 0 {
            cp4(m, sp + 0x60, sp + 0x20);
            cp4(m, sp + 0x70, sp + 0x30);
            m.call(0x0021_5850, &[b, s1, sp, sp + 0x60, sp + 0x70], &[[f12, 0.0, 0.0, 0.0]]);
        }
        m.wf(s1 + 0x5c, time);
        cp16(m, s1 + 0x10, sp);
        cp16(m, s1 + 0x20, sp + 0x10);
        cp4(m, s1, sp + 0x20);
        cp4(m, s1 + 4, sp + 0x30);
        m.call(0x0011_a058, &[s1 + 0x68], &[]);
    }
    stack_pop(m, 0xb0);
}

/// FUN_001cd780(a0, a1, a2, a3, f): world-anchored HUD text: project a0 via FUN_0014f308 (camera 0x2ddf40),
/// cull by depth, scale text, center on screen point and draw with color *a3 via FUN_002163b0.
pub fn ee_sub_1cd780<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32, f: f32) {
    let cm = 0x002d_df40;
    let q = m.call(0x0014_f308, &[cm, a0], &[]).1;
    let z = q[2];
    if z < m.rf(cm + 0x4c) {
        return;
    }
    let sp = stack_push(m, 0xe0);
    let f0 = m.rf(cm + 0x40) * z;
    let s = m.rf(0x002c_1f20) / f0;
    let s = f * s;
    let sc = m.call(0x0011_d948, &[0x002c_1f28], &[[s, 0.0, 0.0, 0.0]]).1[0];
    m.call(0x0021_5ac0, &[sp, a1], &[[sc, 0.0, 0.0, 0.0]]);
    m.call(0x0025_8438, &[sp + 0xa0], &[[0.0, 0.0, 0.0, 0.0], [0.0, 0.0, 0.0, 0.0]]);
    let h = f32::from_bits(0x43f6_6667);
    m.call(0x0025_8448, &[sp + 0xa0], &[[640.0, 0.0, 0.0, 0.0], [h, 0.0, 0.0, 0.0]]);
    m.w32(sp + 0xc0, 0xff80_8080);
    m.call(0x0025_8458, &[sp + 0xa0, sp + 0xc0], &[]);
    m.call(0x0025_8470, &[sp + 0xa0, 0], &[]);
    m.call(0x0025_8478, &[sp + 0xa0, 0], &[]);
    m.call(0x0021_6260, &[sp, sp + 0xd0, sp + 0xd4], &[]);
    let w = m.rf(sp + 0xd0);
    let hh = m.rf(sp + 0xd4);
    let t0 = m.r32(0x002c_1f34) as i32;
    let a3w = m.r32(0x002c_1f30) as i32;
    let f7 = t0.wrapping_shl(1) as f32;
    let f5 = a3w as f32;
    let f3 = a3w.wrapping_shl(1) as f32;
    let f4 = t0 as f32;
    let x = ((q[0] * 0.5 + 0.5) * 640.0 - w * 0.5 - f5) + f5;
    let y = (((0.5 - q[1] * 0.5) * h - hh) - f7) + f4;
    m.wf(sp + 0xa8, w + f3);
    m.wf(sp + 0xac, hh + f7);
    m.wf(sp + 0xa0, x);
    m.wf(sp + 0xa4, y);
    cp4(m, sp + 0xc0, a3);
    cp4(m, sp + 0xb0, sp + 0xc0);
    m.call(0x0021_63b0, &[sp, sp + 0xa0, 6], &[]);
    stack_pop(m, 0xe0);
}

/// FUN_00126a18(out, o, ctx): collect: walk o's (+0xc)->+0x10 list, evaluate each node chain
/// (FUN_00127838) and merge results into a local value (FUN_00129658), store it to out (FUN_001291a8).
pub fn ee_sub_126a18<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) -> u32 {
    let sp = stack_push(m, 0x30);
    m.w32(sp + 0x10, 0xffff_ffff);
    m.w32(sp, 0xffff_ffff);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    let h = m.r32(a1 + 0xc);
    let mut s0 = m.r32(h + 0x10);
    let s4 = sp + 0x10;
    let s3 = sp + 0x20;
    while s0 != 0 {
        let mut s1 = m.r32(s0 + 0xc);
        if m.r32(s1 + 8) != 0xb {
            m.call(0x0012_7838, &[s3, s1, a2], &[]);
            m.call(0x0012_9658, &[s4, s3], &[]);
            m.call(0x0012_9ec8, &[s3], &[]);
            if m.r32(sp + 0x10) == 0x17 && m.r32(sp + 0x14) == 0 {
                s0 = m.r32(s0 + 0x10);
                continue;
            }
            if m.r32(s1 + 0x10) == 0 {
                m.call(0x0012_9658, &[sp, s4], &[]);
            }
        }
        s1 = m.r32(s1 + 0x10);
        while s1 != 0 {
            m.call(0x0012_7838, &[s3, s1, a2], &[]);
            m.call(0x0012_9658, &[sp, s3], &[]);
            m.call(0x0012_9ec8, &[s3], &[]);
            s1 = m.r32(s1 + 0x10);
        }
        s0 = 0;
    }
    m.call(0x0012_91a8, &[a0, sp], &[]);
    m.call(0x0012_9ec8, &[sp + 0x10], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x30);
    a0
}
