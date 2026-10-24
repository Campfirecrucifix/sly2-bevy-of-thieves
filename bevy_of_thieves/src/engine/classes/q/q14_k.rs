//! WP-Q14 part K: C2cbe68 slots (v4 event handler FUN_00140f90, v5 / v97 / v98) and the static-init wrapper
//! FUN_00229d98 / the helper FUN_0021f388. 1:1 from ghidra_functions(_v2).body on `QMem`.
//! The 128-bit qmtc2 / qmfc2 moves are full quadword moves on the R5900, so the VU0 blocks are lane-exact.
#![allow(unused_variables, unused_mut, dead_code)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem, V4};
use crate::engine::classes::q::q6::vcall;

pub fn c2cbe68_v5() -> u32 {
    0x002b_aef8
}
pub fn c2cbe68_v97<M: QMem + ?Sized>(m: &mut M) -> u32 {
    if m.call(0x0020_f0b0, &[0x003d_d490], &[]).0 != 0 {
        0
    } else {
        0xffff_ffff
    }
}
pub fn c2cbe68_v98() -> u32 {
    0
}
/// FUN_0021f388(p)
pub fn ee_sub_21f388<M: QMem + ?Sized>(m: &mut M, p: u32) {
    let h = m.r32(p);
    m.call(0x0013_7a30, &[h, 0xc, 2, p + 0x40, 0, 0, 0, p + 0x58], &[]);
    m.call(0x0013_7a30, &[h, 0xc, 4, p + 0x68, 0, 0, 0, p + 0x98], &[]);
    m.call(0x0013_7a30, &[h, 0xc, 2, p + 0xa4, 0, 0, 0, p + 0xbc], &[]);
}
/// FUN_00229d98(a, b): six pool inits (counter == 1 on first use)
pub fn ee_sub_229d98<M: QMem + ?Sized>(m: &mut M, a: u32, b: u32) {
    if b == 0xffff && a != 0 {
        for (ctr, pool, esz, n) in [
            (0x002d_d2e0u32, 0x0045_20a8u32, 0x14u32, 0x2000u32),
            (0x002d_d2e4, 0x0045_20d0, 0x18, 0x800),
            (0x002d_d2e8, 0x0045_20f8, 0x28, 0x200),
            (0x002d_d2ec, 0x0045_2120, 0x1c, 0x800),
            (0x002d_d2f0, 0x0045_2148, 0xc, 200),
            (0x002d_d2f4, 0x0045_2170, 0x14, 0x100),
        ] {
            let c = m.r32(ctr).wrapping_add(1);
            m.w32(ctr, c);
            if c == 1 {
                m.call(0x0011_cca8, &[pool, esz, n, 0x10], &[]);
            }
        }
    }
}

fn mul3(r: [V4; 3], v: V4) -> V4 {
    let mut o = [0.0f32; 4];
    for k in 0..4 {
        o[k] = r[0][k] * v[0] + r[1][k] * v[1] + r[2][k] * v[2];
    }
    o
}

/// FUN_00140f90(p, ev, a): C2cbe68 event handler (ev: 1 target lost, 3 stun/hit end, 4 / 5 attach / detach children,
/// 9 message ids 0x15e / 0x161 / 0x162 / 0x1c7 / 0x1c8 / 0x27b / 0x27c, 0xb)
pub fn c2cbe68_v4<M: QMem + ?Sized>(m: &mut M, p: u32, ev: u32, a: u32) {
    handler(m, p, ev, a);
    m.call(0x001e_ed78, &[p + 0x26c], &[]);
    m.call(0x001c_3e20, &[p], &[]);
}
fn handler<M: QMem + ?Sized>(m: &mut M, p: u32, ev: u32, a: u32) {
    match ev {
        1 => {
            if m.r32(p + 0x4c8) != 0xb || m.r32(p + 0x4d4) != a {
                return;
            }
            m.call(0x0013_3448, &[a, p], &[]);
            m.w32(p + 0x4d4, 0);
        }
        3 => {
            let s;
            if a == m.r32(p + 0x1030) {
                s = m.r32(p + 0xdf4);
            } else {
                let q = m.r32(p + 0x1034);
                if q == 0 || a != m.r32(q + 0x30) {
                    return;
                }
                s = m.r32(p + 0xdf4);
            }
            m.w32(p + 0x1038, 0xffff_ffff);
            if s.wrapping_sub(8) < 5 {
                m.call(0x001a_15d0, &[p + 0xdf0], &[]);
                return;
            }
            let c = m.r32(p + 0x11b4) as i32;
            if 0 < c {
                if c < 3 {
                    m.call(0x0022_06c8, &[p + 0x11b0, 0], &[]);
                    return;
                }
                if c < 7 {
                    if 4 < c {
                        m.call(0x0022_06c8, &[p + 0x11b0, 4], &[]);
                    }
                    return;
                }
            }
        }
        4 => {
            if m.r32(a + 4) == p {
                let w = m.r32(a);
                m.w32(p + 0x108c, w);
                let n = m.r32(p + 0x1094) as i32;
                let mut i = 0i32;
                if 0 < n {
                    loop {
                        let base = m.r32(p + 0x1090);
                        let off = (i as u32) * 8;
                        i += 1;
                        let e = m.r32(base + off);
                        vcall(m, e, 0xc, 0x44, &[], &[]);
                        let base = m.r32(p + 0x1090);
                        m.w32(base + off + 4, 0);
                        if m.r32(p + 0x1094) as i32 <= i {
                            break;
                        }
                    }
                }
            }
        }
        5 => {
            if m.r32(a + 4) == p && m.r32(a) == m.r32(p + 0x108c) {
                m.w32(p + 0x108c, 0);
                let mut i = 0i32;
                if 0 < m.r32(p + 0x1094) as i32 {
                    loop {
                        let base = m.r32(p + 0x1090);
                        let e = m.r32(base + (i as u32) * 8);
                        i += 1;
                        vcall(m, e, 0xc, 0x4c, &[], &[]);
                        if m.r32(p + 0x1094) as i32 <= i {
                            break;
                        }
                    }
                }
            }
        }
        9 => {
            let src = m.r32(a);
            if m.r32(src + 0x18) != p {
                return;
            }
            let id = m.r32(a + 8);
            if id == 0x1c7 {
                if m.r32(p + 0x4c8) == 0xc {
                    let t = m.r32(p + 0x1030);
                    m.call(0x0013_1a00, &[t], &[[1.0, 0.0, 0.0, 0.0]]);
                    let u = m.r32(p + 0x4d4);
                    m.call(0x001c_d098, &[p + 0x900, u], &[]);
                }
                return;
            }
            if id < 0x1c8 {
                if id == 0x161 {
                    m.w32(p + 0x1044, 1);
                } else {
                    if id < 0x162 {
                        if id == 0x15e {
                            // asm 0x141078/0x1410b4: a0 = p, a1 = *(a), $f12 = *(f32*)(a + 4)
                            let (a1, t) = (m.r32(a), m.rf(a + 4));
                            let v = m.call(0x0020_4e38, &[p, a1], &[[t, 0.0, 0.0, 0.0]]).1;
                            let neg = [-v[0], -v[1], -v[2], -v[3]];
                            let r0 = m.rv(p + 0x1b0);
                            let r1 = m.rv(p + 0x1c0);
                            let r2 = m.rv(p + 0x1d0);
                            m.wv(p + 400, neg);
                            let r = mul3([r0, r1, r2], v);
                            vcall(m, p, 0xc, 0x11c, &[], &[r]);
                        }
                        return;
                    }
                    if id != 0x162 {
                        return;
                    }
                    let q = m.rv(p + 0x1060);
                    m.w32(p + 0x1044, 0);
                    let pos = m.call(0x001a_62e8, &[p], &[]).1;
                    let d = [q[0] - pos[0], q[1] - pos[1], q[2] - pos[2], q[3] - pos[3]];
                    m.wv(p + 0x1070, d);
                    m.w32(p + 0x1078, 0);
                }
                return;
            }
            if id == 0x27b {
                if m.r32(p + 0x11b4) != 0xffff_ffff {
                    m.call(0x0022_05a8, &[p + 0x11b0], &[]);
                }
                return;
            }
            if id < 0x27c {
                if id == 0x1c8 && m.r32(p + 0x4c8) == 0xc {
                    let u = m.r32(p + 0x4d4);
                    m.call(0x001c_cc88, &[p + 0x900, u], &[]);
                }
                return;
            }
            if id != 0x27c || m.r32(p + 0x4c8) != 0xb {
                return;
            }
            let u = m.r32(p + 0x4d4);
            if u != 0 {
                if m.call(0x0013_2b10, &[u, 100], &[]).0 != 0 {
                    if m.call(0x0022_4590, &[u], &[]).0 != 0 {
                        m.call(0x0022_45b8, &[u], &[]);
                    }
                }
            }
            let mut clear_c0 = false;
            let k = m.r32(p + 0x130c);
            if k == 0 {
                let sel = m.r32(p + 0x9bc);
                if sel == 0xffff_ffff {
                    m.w32(p + 0x130c, 0);
                } else {
                    m.call(0x0014_1cd0, &[p], &[]);
                    let sel = m.r32(p + 0x9bc) as i32;
                    if sel == 0x15 {
                        m.call(0x0019_f220, &[0x003d_d490, 0x15], &[]);
                        clear_c0 = true;
                    } else if sel < 0x16 {
                        if sel == 0xe {
                            m.call(0x0019_f170, &[0x003d_d490, 1], &[]);
                            clear_c0 = true;
                        } else {
                            m.w32(p + 0x130c, 0);
                        }
                    } else if sel == 0x17 {
                        let w = m.r32(0x002e_1e40);
                        let ww = m.r32(w + 0x2010);
                        let l = m.call(0x001c_b250, &[ww, 0], &[]).0;
                        if l != 0 {
                            m.call(0x001a_58d0, &[l, 0, 3], &[]);
                        }
                        let ctx = stack_push(m, 0x20);
                        let head = m.r32(0x003e_148c);
                        let off = m.r32(0x003e_1494);
                        m.w32(ctx + 4, head.wrapping_add(off));
                        m.w32(ctx, 0x003e_148c);
                        m.w32(ctx + 8, off);
                        let old = m.r32(0x002d_d2fc);
                        m.w32(ctx + 0xc, old);
                        m.w32(0x002d_d2fc, ctx);
                        let mut it = head;
                        while it != 0 {
                            if m.r32(it + 0x54) == 4 && m.r32(it + 0x44) == p {
                                m.call(0x0015_6780, &[it], &[]);
                            }
                            let slot = m.r32(ctx + 4);
                            it = m.r32(slot);
                            let o = m.r32(ctx + 8);
                            m.w32(ctx + 4, it.wrapping_add(o));
                        }
                        let old = m.r32(ctx + 0xc);
                        m.w32(0x002d_d2fc, old);
                        stack_pop(m, 0x20);
                        let t = m.rf(0x002d_ded0) + m.rf(0x002b_cc94);
                        m.wf(0x003e_1be8, t);
                        clear_c0 = true;
                    } else {
                        m.w32(p + 0x130c, 0);
                    }
                }
            } else {
                if m.r32(p + 0x1314) == m.r32(p + 0x9bc) {
                    m.call(0x0013_0658, &[p + 0x1310], &[]);
                    m.w32(p + 0x1314, 0xffff_ffff);
                } else {
                    m.call(0x0014_1cd0, &[p], &[]);
                    let k = m.r32(p + 0x130c);
                    m.call(0x0013_05b8, &[k, p, 8, p + 0x1310], &[[0.0; 4], [1.0, 0.0, 0.0, 0.0]]); // PART r16 ABI: $f12 = 0.0, $f13 = 1.0
                    let s = m.r32(p + 0x9bc);
                    m.w32(p + 0x1314, s);
                }
                clear_c0 = true;
            }
            if clear_c0 {
                m.w32(p + 0x130c, 0);
            }
            m.w32(p + 0x9bc, 0xffff_ffff);
        }
        0xb => {
            if m.r32(a + 4) != p {
                return;
            }
            m.call(0x0013_0658, &[p + 0x1310], &[]);
            m.w32(p + 0x1314, 0xffff_ffff);
        }
        _ => {}
    }
}
