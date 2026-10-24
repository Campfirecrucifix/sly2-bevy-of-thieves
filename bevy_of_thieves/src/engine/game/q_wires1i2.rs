//! WIRE-S1: 1:1 QMem ports of WIRE-ROOTS live-frontier roots (round 3).
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

/// FUN_0020d9d0(p): big manager ctor: init pools (FUN_0011cca8), lists (FUN_0011a1f8/FUN_0011a050),
/// slot arrays and float defaults; returns p.
pub fn ee_sub_20d9d0<M: QMem + ?Sized>(m: &mut M, p: u32) -> u32 {
    m.w32(p, 0);
    m.w32(p + 4, 0);
    m.w32(p + 0xc, 0x002d_c018);
    let pools1: [(u32, u32, u32, u32); 11] = [
        (0x398c, 0x14, 0x100, 0x10),
        (0x39fc, 0x14, 0x100, 0x10),
        (0x3c90, 0x14, 0x100, 0x10),
        (0x3cb4, 0xc, 0x200, 0x10),
        (0x3ce0, 0x80, 0x300, 0x10),
        (0x3cf8, 0x14, 0x400, 0x10),
        (0x3d40, 0xc0, 0x100, 0x10),
        (0x3d94, 0x60, 0x100, 0x10),
        (0x3dac, 0x80, 0x400, 0x10),
        (0x3dc4, 0x14, 0x8, 0x10),
        (0x3de8, 0x50, 0x80, 0x10),
    ];
    for &(o, a1, a2, a3) in pools1.iter() {
        m.call(0x0011_cca8, &[p + o, a1, a2, a3], &[]);
    }
    m.w32(p + 0x3e00, 0x8000_0000);
    let pools2: [(u32, u32, u32, u32); 3] =
        [(0x3e44, 0x30, 0x40, 0x10), (0x4008, 0x60, 0x40, 0x10), (0x403c, 0x1b0, 0x10, 0x10)];
    for &(o, a1, a2, a3) in pools2.iter() {
        m.call(0x0011_cca8, &[p + o, a1, a2, a3], &[]);
    }
    let v = m.call(0x001c_f5c8, &[], &[]).0;
    let a1 = if v != 0 { v } else { 0xf0 };
    m.call(0x0011_cca8, &[p + 0x4054, a1, 0x80, 0x10], &[]);
    let pools3: [(u32, u32, u32, u32); 10] = [
        (0x4070, 0x460, 0x28, 0x10),
        (0x4088, 0x10c0, 0x80, 0x40),
        (0x40a0, 0x560, 0x8, 0x10),
        (0x40b8, 0x1c, 0x200, 0x10),
        (0x40d0, 0xf0, 0x20, 0x10),
        (0x40e8, 0x10, 0x10, 0x10),
        (0x4118, 0x60, 0x20, 0x10),
        (0x413c, 0x30, 0x10, 0x10),
        (0x4648, 0x40, 0x10, 0x10),
        (0x4660, 0xd0, 0x10, 0x10),
    ];
    for &(o, a1, a2, a3) in pools3.iter() {
        m.call(0x0011_cca8, &[p + o, a1, a2, a3], &[]);
    }
    // (fn, offset, arg) in asm order; true = FUN_0011a1f8, false = FUN_0011a050
    let lists: [(bool, u32, u32); 39] = [
        (true, 0x10, 0x24),
        (true, 0x20, 0x4),
        (true, 0x2c, 0xc),
        (true, 0x3d10, 0x48),
        (true, 0x3d1c, 0x40),
        (true, 0x3d28, 0x50),
        (true, 0x3d34, 0x58),
        (true, 0x3d88, 0x24),
        (false, 0x39f0, 0x10),
        (false, 0x3ca8, 0x10),
        (true, 0x3f24, 0xe0),
        (true, 0x3f30, 0x1b4),
        (true, 0x3f3c, 0x1b4),
        (true, 0x3f48, 0x104),
        (true, 0x3f54, 0x174),
        (true, 0x3f60, 0x1b4),
        (true, 0x3f6c, 0x104),
        (true, 0x3f78, 0x104),
        (true, 0x3f84, 0x104),
        (true, 0x3f90, 0xe0),
        (true, 0x3fa8, 0x154),
        (true, 0x3f9c, 0x100),
        (true, 0x3ddc, 0x1c),
        (true, 0x3ffc, 0x58),
        (true, 0x3fb4, 0xfe0),
        (true, 0x3fc0, 0x174),
        (false, 0x3eac, 0x84),
        (false, 0x4100, 0x18),
        (false, 0x410c, 0x40),
        (true, 0x4130, 0x50),
        (true, 0x3fcc, 0x20),
        (true, 0x3fd8, 0x104),
        (true, 0x45b0, 0x1d0),
        (true, 0x45bc, 0x1e0),
        (true, 0x3b00, 0x88),
        (true, 0x3fe4, 0xe0),
        (true, 0x3ff0, 0x130),
        (true, 0x3bd0, 0xb4),
        (false, 0x3bdc, 0x4c),
    ];
    for &(f, o, a) in lists.iter() {
        let fa = if f { 0x0011_a1f8 } else { 0x0011_a050 };
        m.call(fa, &[p + o, a], &[]);
    }
    let mut s0 = p + 0x38;
    for _ in 0..0x200 {
        m.call(0x0011_a050, &[s0, 0x1c], &[]);
        s0 += 0xc;
    }
    let mut s0 = p + 0x3038;
    let mut s2: i32 = 0xc6;
    while s2 >= 0 {
        m.call(0x0011_a050, &[s0, 0x20], &[]);
        s2 -= 1;
        s0 += 0xc;
    }
    let mut s0 = p + 0x1838;
    for _ in 0..0x200 {
        m.call(0x0011_a050, &[s0, 0xc], &[]);
        s0 += 0xc;
    }
    let f8 = m.rf(0x002c_3e6c);
    let f7 = m.rf(0x002c_3e68);
    let f0 = m.rf(0x002c_3e64);
    let f1 = m.rf(0x002c_3e60);
    let f5 = f8 - f7;
    let f3 = f0 * f7;
    m.wf(p + 0x3e10, 180.0);
    let f4 = f1 * f8;
    let f0 = f0 - f1;
    m.wf(p + 0x3e18, 255.0);
    let f3 = f3 - f4;
    m.wf(p + 0x3e14, 255.0);
    let f5 = f5 / f0;
    m.wf(p + 0x3e20, 90.0);
    m.wf(p + 0x3e64, 128.0);
    let f3 = f3 / f0;
    m.wf(p + 0x3e68, 96.0);
    m.wf(p + 0x3e6c, 160.0);
    m.w32(p + 0x3c88, 0x189);
    m.w32(p + 0x3c80, u32::MAX);
    m.w32(p + 0x3c8c, u32::MAX);
    m.w32(p + 0x3ccc, 1);
    m.wf(p + 0x45e0, f8);
    m.w32(p + 0x3f10, (-2i32) as u32);
    m.w32(p + 0x3e24, 0);
    m.wf(p + 0x45e4, f7);
    m.wf(p + 0x45d0, f3);
    m.wf(p + 0x45d4, f5);
    m.w32(p + 0x45d8, 0);
    m.w32(p + 0x4740, u32::MAX);
    m.wf(p + 0x4754, -60.0);
    m.w32(p + 0x4554, 1);
    m.wf(p + 0x4750, -60.0);
    m.call(0x0023_33b4, &[p + 0x4590, u32::MAX, 0x20], &[]);
    p
}

fn cp64<M: QMem + ?Sized>(m: &mut M, dst: u32, src: u32) {
    for k in (0..64).step_by(4) {
        let w = m.r32(src + k);
        m.w32(dst + k, w);
    }
}

/// FUN_00186930(p): IK-ish update: snapshot two matrices, transform local point, store two
/// segment lengths (+0xc0/+0xc4), angle (+0xc8, FUN_0011e4b0), blend +0x90 via FUN_0022fcc8.
pub fn ee_sub_186930<M: QMem + ?Sized>(m: &mut M, p: u32) {
    let a1 = m.r32(p + 4);
    let a0 = m.r32(p + 8);
    let t0 = m.r32(p + 0xc);
    cp64(m, p + 0x10, a1);
    let vf6 = m.rv(p + 0xb0);
    cp64(m, p + 0x50, a0);
    let vf5 = m.rv(t0 + 0x70);
    let vf4 = m.rv(t0 + 0x50);
    let vf3 = m.rv(t0 + 0x60);
    let mut vf2 = m.rv(t0 + 0x40);
    let mut vf1 = m.rv(a0 + 0x70);
    for i in 0..3 {
        let acc = vf2[i] * vf6[0] + vf4[i] * vf6[1] + vf3[i] * vf6[2];
        vf2[i] = acc + vf5[i] * 1.0;
    }
    for i in 0..4 {
        vf2[i] -= vf1[i];
    }
    let vf4 = m.rv(a1 + 0x70);
    let l2 = (vf2[0] * vf2[0] + vf2[1] * vf2[1] + vf2[2] * vf2[2]).sqrt();
    for i in 0..4 {
        vf1[i] -= vf4[i];
    }
    let l1 = (vf1[0] * vf1[0] + vf1[1] * vf1[1] + vf1[2] * vf1[2]).sqrt();
    m.wf(p + 0xc4, l2);
    m.wf(p + 0xc0, l1);
    let f0 = m.call(0x0011_e4b0, &[0], &[vf1, vf2]).1[0];
    m.wf(p + 0xc8, f0);
    let o = m.r32(p + 8);
    let a1 = m.r32(o + 0xa0);
    let a2: V4 = m.rv(p + 0x90);
    let r = m.call(0x0022_fcc8, &[0, a1], &[a2]).1;
    m.wv(p + 0x90, r);
}

/// FUN_001871c0(p): memory-card/save status poll state machine; ends in FUN_001875b8(p, state, sub).
pub fn ee_sub_1871c0<M: QMem + ?Sized>(m: &mut M, p: u32) {
    if m.r32(p + 0x100c) == 0 {
        return;
    }
    let mut s1: i32 = 0;
    let h0 = m.r32(p + 0x1000);
    let h1 = m.r32(p + 0x1004);
    let mut a0 = m.call(0x0024_3388, &[h0, h1], &[]).0 as i32;
    if a0 == 6 || a0 == 2 {
        s1 = 1;
    }
    let v1 = m.r32(p + 0x100c) as i32;
    let a1 = m.r32(p + 0x1010);
    if v1 != 3 && a1 != 0 {
        let f2 = m.rf(p + 0x1014);
        let f0 = m.rf(0x002d_df0c) - f2;
        if 2.0 <= f0 {
            a0 = 0;
        }
    }
    if a1 != 0 && a0 == 0 {
        m.call(0x0018_75b8, &[p, 1, 0], &[]);
    }
    if s1 == 0 {
        return;
    }
    s1 = 2;
    let mut s2 = m.r32(p + 0x100c) as i32;
    let mut s3 = m.r32(p + 0x1010) as i32;
    let h0 = m.r32(p + 0x1000);
    let h1 = m.r32(p + 0x1004);
    if s2 == 2 {
        let v1 = m.call(0x0024_3500, &[h0, h1], &[]).0 as i32;
        if v1 == 1 {
            s2 = 1;
        } else if v1 < 2 {
            s3 = 0;
            if v1 == 0 {
                let v1 = m.r32(p + 0x1010) as i32;
                if v1 == 2 {
                    s3 = 3;
                    s2 = 1;
                } else if v1 < 3 {
                    if v1 == 1 {
                        s3 = 2;
                        s2 = 1;
                    }
                } else if v1 == 3 {
                    s3 = 4;
                    s2 = 3;
                }
            }
        } else if v1 != s2 {
            s3 = 0;
        }
    } else if s2 == 1 {
        if s3 == 1 {
            s2 = s1;
            let v = m.call(0x0024_37b8, &[h0, h1, 1, 3], &[]).0;
            if v ^ 1 != 0 {
                s2 = s3;
            }
        } else if s3 == 0 {
            s1 = m.call(0x0024_3680, &[h0, h1, 1, 0], &[]).0 as i32;
            if s1 != 0 {
                let h0 = m.r32(p + 0x1000);
                let h1 = m.r32(p + 0x1004);
                let a0 = m.call(0x0024_3680, &[h0, h1, 2, 0], &[]).0 as i32;
                if a0 >= 1 {
                    s1 = a0;
                }
                if s1 == 4 || s1 == 7 {
                    s3 = 1;
                    s2 = if a0 != 0 { s3 } else { 3 };
                }
            }
        } else if s3 == 2 {
            s1 = m.call(0x0024_3558, &[h0, h1, u32::MAX, 0], &[]).0 as i32;
            let checks: [(u32, u32, i32); 6] = [(0, 1, 1), (0, 2, 2), (0, 3, 0), (1, 1, 1), (1, 2, 1), (1, 3, 1)];
            let mut ok = s1 == s3;
            for &(a2, a3, want) in checks.iter() {
                if !ok {
                    break;
                }
                let h0 = m.r32(p + 0x1000);
                let h1 = m.r32(p + 0x1004);
                ok = m.call(0x0024_3558, &[h0, h1, a2, a3], &[]).0 as i32 == want;
            }
            if !ok {
                s2 = 3;
            } else {
                let sp = stack_push(m, 0x10);
                m.call(0x0023_33b4, &[sp, 0xff, 6], &[]);
                let mut v1: i32 = 0;
                loop {
                    m.w8(sp + v1 as u32, v1 as u8);
                    v1 += 1;
                    if v1 >= s1 {
                        break;
                    }
                }
                let h0 = m.r32(p + 0x1000);
                let h1 = m.r32(p + 0x1004);
                let v = m.call(0x0024_3930, &[h0, h1, sp], &[]).0;
                stack_pop(m, 0x10);
                if v ^ 1 == 0 {
                    s2 = 2;
                }
            }
        } else if s3 == 3 {
            let v = m.call(0x0024_3b70, &[h0, h1], &[]).0 as i32;
            if v == s2 {
                let h0 = m.r32(p + 0x1000);
                let h1 = m.r32(p + 0x1004);
                let v = m.call(0x0024_3bd0, &[h0, h1], &[]).0;
                if v ^ 1 == 0 {
                    s2 = s1;
                }
            } else {
                s2 = 3;
            }
        }
    } else if s2 == 3 {
        let v = m.call(0x0018_7998, &[p], &[]).0;
        if v == 0 {
            s2 = 1;
            s3 = 0;
        } else if m.r32(p + 0x10e8) != 0 && m.r32(p + 0x1010) as i32 >= 3 {
            m.call(0x001d_51b8, &[p + 0x10cc, s2 as u32], &[]);
        }
    }
    m.call(0x0018_75b8, &[p, s2 as u32, s3 as u32], &[]);
}

/// FUN_00219190(p, i, j): run entry i's action list (kinds 0..3: object enable/bind, unbind, two
/// FUN_001fe5f0/FUN_001fe738 global ops on 0x3dd040).
pub fn ee_sub_219190<M: QMem + ?Sized>(m: &mut M, p: u32, i: u32, j: u32) {
    let base = m.r32(p + 4);
    let s2 = base.wrapping_add(i << 5);
    if m.r8(s2 + 0x18) == 0 {
        return;
    }
    let fp = j.wrapping_mul(0x18);
    let mut s1: i32 = 0;
    loop {
        let l = m.r32(s2 + 0x1c);
        let a0 = ((s1 as u32) << 3).wrapping_add(l);
        let k = m.r32(a0) as i32;
        match k {
            0 => {
                let o = m.r32(a0 + 4);
                let s0 = m.call(0x001f_bb90, &[o], &[]).0;
                m.call(0x001f_b2f8, &[s0, 0], &[]);
                let v = vslot(m, s0, 7, &[], &[]).0;
                if v == 2 {
                    let t = m.r32(p + 0x10);
                    let b = m.r32(t + 8);
                    m.call(0x0025_8040, &[fp.wrapping_add(b) + 0x14, s0], &[]);
                }
            }
            1 => {
                let t = m.r32(p + 0x10);
                let b = m.r32(t + 8);
                let v0 = fp.wrapping_add(b);
                if m.r32(v0 + 0x14) != 0 {
                    let s0 = v0 + 0x14;
                    if s0 != 0 {
                        let o = m.r32(s0);
                        if o != 0 {
                            m.call(0x001f_b098, &[o], &[[0.0, 0.0, 0.0, 0.0]]);
                            m.call(0x0025_8040, &[s0, 0], &[]);
                        }
                    }
                }
            }
            2 => {
                let o = m.r32(a0 + 4);
                m.call(0x001f_e5f0, &[0x003d_d040, o, 0], &[]);
            }
            3 => {
                m.call(0x001f_e738, &[0x003d_d040], &[]);
            }
            _ => {}
        }
        s1 += 1;
        if s1 >= m.r8(s2 + 0x18) as i32 {
            break;
        }
    }
}

/// FUN_00126db8(out, x, a2): build combined result (temporaries via FUN_00129658 / FUN_00129ec8)
/// over a node's child lists, then FUN_001291a8(out, tmp); returns out.
pub fn ee_sub_126db8<M: QMem + ?Sized>(m: &mut M, out: u32, x: u32, a2: u32) -> u32 {
    let sp = stack_push(m, 0x50);
    m.w32(sp + 0x30, 0);
    m.w32(sp + 0x20, u32::MAX);
    m.w32(sp, u32::MAX);
    m.w32(sp + 0x10, u32::MAX);
    let t = m.r32(x + 0xc);
    let s5 = m.r32(t + 0x10);
    let mut s0;
    if m.r32(s5 + 8) == 0x1f {
        let v = m.call(0x0025_6dd0, &[], &[]).0;
        m.call(0x0025_6ee0, &[sp + 0x30, v], &[]);
        let mut s2 = m.r32(s5 + 0xc);
        while s2 != 0 {
            let s0 = m.r32(s2 + 0xc);
            let a1 = m.r32(s0 + 0x10);
            m.call(0x0012_7838, &[sp + 0x40, a1, a2], &[]);
            m.call(0x0012_9658, &[sp + 0x10, sp + 0x40], &[]);
            m.call(0x0012_9ec8, &[sp + 0x40], &[]);
            let a1 = m.r32(sp + 0x30);
            let c = m.r32(s0 + 0xc);
            m.call(0x0012_7c00, &[sp + 0x40, a1, c, sp + 0x10], &[]);
            m.call(0x0012_9658, &[sp + 0x20, sp + 0x40], &[]);
            m.call(0x0012_9ec8, &[sp + 0x40], &[]);
            s2 = m.r32(s2 + 0x10);
        }
        let a0 = m.r32(sp + 0x30);
        m.call(0x0012_7b18, &[a0, a2], &[]);
        s0 = m.r32(s5 + 0x10);
    } else {
        m.call(0x0025_6ee0, &[sp + 0x30, a2], &[]);
        s0 = m.r32(s5 + 0x10);
    }
    while s0 != 0 {
        let c = m.r32(sp + 0x30);
        m.call(0x0012_7838, &[sp + 0x40, s0, c], &[]);
        m.call(0x0012_9658, &[sp, sp + 0x40], &[]);
        m.call(0x0012_9ec8, &[sp + 0x40], &[]);
        s0 = m.r32(s0 + 0x10);
    }
    m.call(0x0012_91a8, &[out, sp], &[]);
    let a0 = m.r32(sp + 0x30);
    if a0 != 0 {
        let v = m.r32(a0).wrapping_sub(1);
        m.w32(a0, v);
        if v == 0 {
            m.call(0x0025_7078, &[a0, 3], &[]);
        }
    }
    m.call(0x0012_9ec8, &[sp + 0x20], &[]);
    m.call(0x0012_9ec8, &[sp + 0x10], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x50);
    out
}

/// FUN_00126490(out, x, a2): build result from node's first child via FUN_00127838 / FUN_00127cf0,
/// FUN_001291a8(out, tmp); returns out.
pub fn ee_sub_126490<M: QMem + ?Sized>(m: &mut M, out: u32, x: u32, a2: u32) -> u32 {
    let sp = stack_push(m, 0x20);
    m.w32(sp, u32::MAX);
    let t = m.r32(x + 0xc);
    let s1 = m.r32(t + 0x10);
    let a1 = m.r32(s1 + 0x10);
    m.call(0x0012_7838, &[sp + 0x10, a1, a2], &[]);
    m.call(0x0012_9658, &[sp, sp + 0x10], &[]);
    m.call(0x0012_9ec8, &[sp + 0x10], &[]);
    let c = m.r32(s1 + 0xc);
    m.call(0x0012_7cf0, &[sp + 0x10, a2, c, sp], &[]);
    m.call(0x0012_9658, &[sp, sp + 0x10], &[]);
    m.call(0x0012_9ec8, &[sp + 0x10], &[]);
    m.call(0x0012_91a8, &[out, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x20);
    out
}
