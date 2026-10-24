//! WIRE-S2: 1:1 QMem ports of live-frontier natives (shard 3/4).
#![allow(unused)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

/// FUN_0016e438: builds a DMA/VIF packet chain (stack builders at sp+0x10 / sp+0x60), per-entry loop, then FUN_001699e8.
pub fn ee_sub_16e438<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, t0: u32, t1: u32, t2: u32) {
    let sp = stack_push(m, 0x170);
    let b = sp + 0x10;
    let s7 = a1;
    let s5 = t2;
    let mut s3: u32 = 0;
    m.call(0x0022_6c10, &[b], &[]);
    m.call(0x001a_e478, &[sp + 0x40], &[]);
    m.call(0x0015_72e0, &[b, 0x2000], &[]);
    m.call(0x0015_74b8, &[b], &[]);
    m.call(0x0022_6e88, &[b, 0xc, 1, 9, 0, sp + 0x94], &[]);
    let f12 = m.rf(a3);
    let k = m.r32(sp + 0x94);
    m.call(0x0014_e270, &[0x002d_df40, k], &[f4(f12)]);
    let v = m.call(0x0011_7bd0, &[s5], &[]).0;
    m.w8(s7 + 1, v as u8);
    let v1 = ((v as u8 as i8 as i32).wrapping_mul(0x50)) as u32;
    m.w32(sp + 0x50, v1);
    m.w32(sp + 0x54, 0x10);
    let r = m.call(0x001a_e2a0, &[sp + 0x50, sp + 0x50, 0x10], &[]).0;
    let c4 = sp + 0x9c;
    m.w32(a2 + 0x18, r);
    let s1 = sp + 0x60;
    if (m.r8(s7 + 1) as i8 as i32) > 0 {
        let mut s2: u32 = 0;
        loop {
            let s0 = m.r32(a2 + 0x18).wrapping_add(s2);
            s3 = s3.wrapping_add(1);
            let v = m.call(0x0011_7db8, &[s5], &[]).0;
            s2 = s2.wrapping_add(0x50);
            m.w32(s0, v);
            let v = m.call(0x0011_7d50, &[s5], &[]).0;
            m.w32(s0 + 8, v);
            if !((s3 as i32) < (m.r8(s7 + 1) as i8 as i32)) {
                break;
            }
        }
    }
    let mut b0: u32 = 0x22;
    m.call(0x0016_6920, &[s1], &[]);
    m.call(0x0015_72e0, &[s1, 8], &[]);
    m.call(0x0016_6bd8, &[s1, 0, 2, 0x1e], &[]);
    m.call(0x0016_6e90, &[s1, 0x42, 0x44], &[]);
    let k = m.r32(0x002d_eb50);
    m.call(0x0016_6c70, &[s1, k], &[]);
    m.call(0x0016_69e0, &[s1, 1], &[]);
    let v = m.r32(sp + 0x64);
    let n = m.r32(sp + 0x6c).wrapping_sub(v) >> 4;
    m.call(0x0022_6f28, &[b, 0xc, n, v, 0xd], &[]);
    m.call(0x0015_72d0, &[s1], &[]);
    m.call(0x0016_6bd8, &[s1, 0xc1, 3, 0x44e], &[]);
    m.call(0x0016_6e90, &[s1, 0x47, 0x0007_3001], &[]);
    m.call(0x0016_6e08, &[s1, 0, 0, 0, 0], &[]);
    m.call(0x0016_6e08, &[s1, 0, 0, 0, 0], &[]);
    m.call(0x0016_69e0, &[s1, 0], &[]);
    m.call(0x0016_6bd8, &[s1, 1, 3, 0x44e], &[]);
    m.call(0x0016_6e90, &[s1, 0x47, 0x0007_2001], &[]);
    m.call(0x0016_6e08, &[s1, 0, 0, 0, 0], &[]);
    m.call(0x0016_6e08, &[s1, 0, 0, 0, 0], &[]);
    m.call(0x0016_69e0, &[s1, 1], &[]);
    let v = m.r32(sp + 0x64);
    m.call(0x0022_6f28, &[b, 0xc, 2, v, 0x12], &[]);
    let v = m.r32(sp + 0x64);
    m.call(0x0022_6f28, &[b, 0xc, 2, v.wrapping_add(0x40), 0x16], &[]);
    let v = m.r32(sp + 0x64);
    m.call(0x0022_6f28, &[b, 0xc, 2, v, 0x1a], &[]);
    let v = m.r32(sp + 0x64);
    m.call(0x0022_6f28, &[b, 0xc, 2, v.wrapping_add(0x40), 0x1e], &[]);
    let v = m.r8(s7 + 1) as i8 as i32 as u32;
    if v != 0 {
        m.w32(s7 + 0xc, b0);
        b0 = (v << 2).wrapping_add(0x22);
    }
    let b4 = m.call(0x0011_7d50, &[s5], &[]).0;
    let mut s2: u32 = 0;
    let b8 = s7 + 0x10;
    let bc = a2 + 0x10;
    if (b4 as i32) > 0 {
        let c0 = sp + 0x90;
        loop {
            m.call(0x001a_e478, &[sp + 0x90], &[]);
            m.call(0x0022_6e88, &[b, 0xd, 3, 0xa, 0, sp + 0x98], &[]);
            let k = m.r32(sp + 0x98);
            m.call(0x0023_33b4, &[k, 0, 0x18], &[]);
            let p = m.r32(sp + 0x98);
            if s2 == 0 {
                let h = m.r16(p) | 1;
                m.w16(p, h);
            } else if s2 == 1 {
                let h = m.r16(p) | 2;
                m.w16(p, h);
            }
            let mut s1 = b0;
            let fp = s2.wrapping_add(1);
            let p = m.r32(sp + 0x98);
            m.w16(p + 2, s1 as u16);
            let s4 = m.call(0x0011_7bd0, &[s5], &[]).0;
            m.call(0x0022_6fd0, &[b, 8, s4, s5, s1], &[]);
            let p = m.r32(sp + 0x98);
            m.w16(p + 8, s1 as u16);
            s1 = s1.wrapping_add(s4);
            let p = m.r32(sp + 0x98);
            m.w16(p + 0xa, s1 as u16);
            if m.r8(s7 + 1) != 0 {
                let mut s0: u32 = 0;
                let n = m.call(0x0011_7bd0, &[s5], &[]).0;
                let s6 = s1.wrapping_add(s4);
                if (n as i32) > 0 {
                    loop {
                        let v = m.call(0x0011_7bd0, &[s5], &[]).0;
                        let p = m.r32(sp + 0x98);
                        let h = (m.r16(s7 + 0xc) as u32).wrapping_add(v << 2);
                        let a = p.wrapping_add(s0 << 1);
                        s0 = s0.wrapping_add(1);
                        m.w16(a + 0x10, h as u16);
                        if !((s0 as i32) < (n as i32)) {
                            break;
                        }
                    }
                }
                let s3 = n << 2;
                while (s0 as i32) < 4 {
                    let p = m.r32(sp + 0x98);
                    let h = m.r16(p + 0x10);
                    m.w16(p.wrapping_add(s0 << 1) + 0x10, h);
                    s0 = s0.wrapping_add(1);
                }
                let flag = (n as i32) < 4;
                if flag {
                    m.call(0x0022_6d68, &[b, 0x002d_eb60], &[]);
                    let k = m.r32(s3.wrapping_add(0x002b_e540));
                    m.call(0x0022_6dd0, &[b, k], &[]);
                }
                let k = m.r32(s3.wrapping_add(0x002b_e558));
                m.call(0x0022_6fd0, &[b, k, s4, s5, s1], &[]);
                let p = m.r32(sp + 0x98);
                m.w16(p + 4, s1 as u16);
                s1 = s6;
                if flag {
                    m.call(0x0022_6dd0, &[b, 0], &[]);
                }
            }
            s2 = fp;
            let s0 = m.call(0x0011_7bd0, &[s5], &[]).0;
            m.call(0x0022_6fd0, &[b, 0xe, s0, s5, s1], &[]);
            let p = m.r32(sp + 0x98);
            m.w16(p + 0xc, s1 as u16);
            s1 = s1.wrapping_add(s0);
            let p = m.r32(sp + 0x98);
            m.w16(p + 0xe, s1 as u16);
            m.call(0x0022_6ce0, &[b, 0], &[]);
            m.call(0x0022_6ce0, &[b, 0x738], &[]);
            m.call(0x0022_6ce0, &[b, 0x7f0], &[]);
            m.call(0x001a_e4a0, &[c0, 2], &[]);
            if !((s2 as i32) < (b4 as i32)) {
                break;
            }
        }
    }
    m.call(0x0015_76f0, &[b], &[]);
    m.call(0x0015_7620, &[b], &[]);
    m.call(0x0015_73c0, &[b, b8, c4, bc], &[]);
    m.call(0x0016_99e8, &[a0, s7, a2, a3, t0, t1, 0, 0, s5], &[]);
    m.call(0x001a_e4a0, &[sp + 0x40, 2], &[]);
    stack_pop(m, 0x170);
}

// SKIP 0x223a70: VU0 macro-heavy (vmini/vmax/vmula/vmadd on 128-bit GPR quadword args a1/a2, qword-returning callees in a loop)

/// FUN_0021e4b8: formats a field (prefix FUN_0021ed28 + body FUN_0021e1b0) and writes it to a1 via FUN_0020d660 with width padding.
pub fn ee_sub_21e4b8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    let sp = stack_push(m, 0x270);
    let mut s7: u32 = 0;
    let mut s3 = a2;
    if m.r32(a0 + 0x1c) == 0 {
        if m.r32(a0 + 0x14) != 0 {
            if m.r8(a0 + 0x2c) == 0x67 {
                m.w32(a0 + 0x1c, 1);
            }
        } else {
            m.w32(a0 + 0x1c, 6);
        }
    } else if m.r32(a0 + 0x14) != 0 && m.r8(a0 + 0x2c) == 0x67 {
        m.w32(a0 + 0x1c, 1);
    }
    let v = m.call(0x0023_dd30, &[s3, 0], &[]).0;
    let neg = (v as i32) < 0;
    if neg {
        s3 = m.call(0x0023_d7a0, &[0, s3], &[]).0;
    }
    m.w8(sp + 0x1c0, 0);
    if neg {
        let h = m.r16(0x002a_9460);
        m.w16(sp + 0x1c0, h);
    } else if m.r32(a0 + 4) != 0 {
        let h = m.r16(0x002a_9468);
        m.w16(sp + 0x1c0, h);
    } else if m.r32(a0 + 8) != 0 {
        let h = m.r16(0x002a_9470);
        m.w16(sp + 0x1c0, h);
    }
    let s6 = sp + 0x1c0;
    let s4 = m.call(0x0021_ed28, &[s6], &[]).0;
    let s1 = m.call(0x0021_e1b0, &[sp, 0x1b5, a0, s3], &[]).0;
    m.w8(sp.wrapping_add(s1), 0);
    if m.r32(a0 + 0x30) != 0 {
        m.call(0x0021_ee30, &[sp], &[]);
    }
    let w = m.r32(a0 + 0x18);
    let v1 = s4.wrapping_add(s1);
    let mut pad0: u32 = 0;
    let mut s5: u32 = 0;
    if (v1 as i32) < (w as i32) {
        let d = w.wrapping_sub(v1);
        if m.r32(a0) != 0 {
            s5 = d;
        } else if m.r32(a0 + 0x10) == 0 || s1 == 0 {
            pad0 = d;
        } else {
            s7 = d;
        }
    }
    let pad = |m: &mut M, n: u32, c: u8| {
        m.w8(sp + 0x1d0, c);
        let mut s0 = n;
        loop {
            let v = s0;
            s0 = s0.wrapping_sub(1);
            if (v as i32) <= 0 {
                break;
            }
            if m.call(0x0020_d660, &[a1, sp + 0x1d0, 1], &[]).0 != 1 {
                break;
            }
        }
    };
    pad(m, pad0, 0x20);
    m.call(0x0020_d660, &[a1, s6, s4], &[]);
    pad(m, s7, 0x30);
    m.call(0x0020_d660, &[a1, sp, s1], &[]);
    pad(m, s5, 0x20);
    stack_pop(m, 0x270);
}

/// FUN_0018f0a8: state-driven pose/anim selection (jump table 0x2a6040 on +0x18c8), then FUN_0018e540 transitions.
pub fn ee_sub_18f0a8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let v1 = m.r32(a0 + 0x18d8) as i32;
    if v1 < 0x16 && !(v1 < 0xc) {
        let o = m.r32(a0 + 0x150);
        if m.r32(o + 0x30) & 0x80 == 0 {
            m.w32(a0 + 0x18ec, 0);
        }
    }
    let st = m.r32(a0 + 0x18c8);
    let mut s1: u32 = 0;
    if st < 0x17 {
        match st {
            0 | 1 | 10 | 12 => s1 = 1,
            3 | 21 | 22 => {
                let o = m.r32(a0 + 0x150);
                if m.call(0x001c_46c0, &[o, 0x80], &[f4(0.0)]).0 != 0 {
                    m.call(0x0019_5a00, &[a0 + 0x2c0, 0, 0, 0], &[]);
                }
            }
            5 => {
                let v = m.r32(a0 + 0x18d0) as i32;
                if v < 6 {
                    s1 = 1;
                } else if v < 8 {
                    let o = m.r32(a0 + 0x150);
                    if m.r32(o + 0x30) & 0x80 == 0 {
                        if m.r32(a0 + 0x18e8) != 0 {
                            m.call(0x0018_c250, &[a0, 5, 8], &[]);
                        } else {
                            m.call(0x0018_c250, &[a0, 5, 4], &[]);
                        }
                    }
                } else if v < 0xa {
                    return;
                } else {
                    s1 = 1;
                }
            }
            7 => {
                let v = m.r32(a0 + 0x4c8) as i32;
                if v < 5 {
                    s1 = (!(v < 3)) as u32;
                }
            }
            _ => {}
        }
    }
    if s1 == 0 {
        return;
    }
    let v1 = m.r32(a0 + 0x18d8) as i32;
    if v1 != -1 {
        if v1 < -1 || !(v1 < 0x18) || v1 < 0xc || m.r32(a0 + 0x18e4) == 0 {
            return;
        }
    }
    let f20 = f32::from_bits(0x3eb3_3333);
    let o = m.r32(a0 + 0x150);
    if m.call(0x001c_46c0, &[o, 0x10], &[f4(f20)]).0 != 0 {
        m.call(0x0018_e540, &[a0, 0xffff_ffff], &[]);
        let v = m.call(0x0018_f050, &[a0], &[]).0;
        m.call(0x0018_e540, &[a0, v], &[]);
        return;
    }
    let o = m.r32(a0 + 0x150);
    if m.call(0x001c_46c0, &[o, 0x80], &[f4(f20)]).0 != 0 {
        m.call(0x0018_e540, &[a0, 0xffff_ffff], &[]);
        let v = m.call(0x0018_ee38, &[a0], &[]).0;
        m.call(0x0018_e540, &[a0, v], &[]);
    }
}

fn find_255248<M: QMem + ?Sized>(m: &mut M, x: u32) -> (u32, u32) {
    let n = m.r32(0x002d_d294);
    let mut i: u32 = 0;
    while i < n {
        let b = m.r32(0x0045_20c0 + 4 * i);
        if !(x < b) && x < b.wrapping_add(0xc000) {
            let q = ((x.wrapping_sub(b).wrapping_mul(0xaaaa_aaab)) as i32 >> 3) as u32;
            return (q, i);
        }
        i += 1;
    }
    (0, 0)
}

/// FUN_00255248: link a0 after *a1: encode pool slot/index of a0 into (*a1)+4 and of *a1 into a0+4; *a1 = a0.
pub fn ee_sub_255248<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let (s0, s4) = find_255248(m, a0);
    let a2 = m.r32(a1);
    if a2 != 0 {
        let mut v = m.r32(a2 + 4);
        v = (v & 0xffff_e000) | (s0 & 0x1fff);
        m.w32(a2 + 4, v);
        v = (v & 0xffff_9fff) | ((s4 & 3) << 13);
        m.w32(a2 + 4, v);
    }
    let p = m.r32(a1);
    let (q, i) = find_255248(m, p);
    let mut v = m.r32(a0 + 4);
    v = (v & 0xe000_ffff) | ((q & 0x1fff) << 16);
    v = (v & 0x9fff_ffff) | ((i & 3) << 29);
    m.w32(a0 + 4, v);
    m.w32(a1, a0);
}

/// FUN_001f8f80: walk list +0x3d1c (next +0x40, obj +0x60), process eligible objs via FUN_001f8e90/FUN_001f9240, then clear flag bits 27/28 of +0x148.
pub fn ee_sub_1f8f80<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let sp = stack_push(m, 0xc10);
    let mut s1 = m.r32(a0 + 0x3d1c);
    while s1 != 0 {
        let s0 = m.r32(s1 + 0x60);
        'body: {
            let v1 = m.r32(s0 + 0xe8);
            if v1 == 0 {
                break 'body;
            }
            if m.r32(s0 + 0x148) & 0x0800_0000 != 0 {
                break 'body;
            }
            let k = m.r32(v1 + 0xc0);
            if k == 3 && m.r32(v1 + 0xa0) == k {
                break 'body;
            }
            if a1 & 1 != 0 {
                let p = m.r32(s0 + 0x58);
                let h = m.r16(p + 0x98) as i16 as i32 as u32;
                if m.call(0x0023_0348, &[h, s0], &[]).0 != 0 {
                    let p = m.r32(s0 + 0x5c);
                    let h = m.r16(p + 0x98) as i16 as i32 as u32;
                    if m.call(0x0023_0348, &[h, s0], &[]).0 != 0 {
                        break 'body;
                    }
                }
            }
            m.w32(sp + 0xc04, 0);
            m.call(0x001f_8e90, &[a0, s0, sp + 0xc04, sp], &[]);
            let c = m.r32(sp + 0xc04);
            m.call(0x001f_9240, &[a0, a1, c, sp], &[]);
        }
        s1 = m.r32(s1 + 0x40);
    }
    let mut s1 = m.r32(a0 + 0x3d1c);
    while s1 != 0 {
        let s0 = m.r32(s1 + 0x60);
        let v = m.r32(s0 + 0x148) & 0xf7ff_ffff & 0xefff_ffff;
        m.w32(s0 + 0x148, v);
        s1 = m.r32(s1 + 0x40);
    }
    stack_pop(m, 0xc10);
}

/// FUN_001ed2d8: if DAT_003e12c8: luma = dot(a0.xyz, (0.3,0.6,0.1)); buf = {DAT_002c3690, DAT_002c3694, FUN_0011d948(0x2bbb60, FUN_00245b58(luma, DAT_002c3698))}; FUN_0011dd90(buf, a0).
pub fn ee_sub_1ed2d8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    if m.r32(0x003e_12c8) == 0 {
        return;
    }
    let sp = stack_push(m, 0x10);
    let c = [f32::from_bits(0x3e99_999a), f32::from_bits(0x3f19_999a), f32::from_bits(0x3dcc_cccd)];
    let v = m.rv(a0);
    let (x, y, z) = (v[0] * c[0], v[1] * c[1], v[2] * c[2]);
    let f12 = x + y + 1.0 * z;
    let f0 = m.rf(0x002c_3690);
    let f1 = m.rf(0x002c_3694);
    let f13 = m.rf(0x002c_3698);
    m.wf(sp, f0);
    m.wf(sp + 4, f1);
    let r = m.call(0x0024_5b58, &[], &[f4(f12), f4(f13)]).1[0];
    let r = m.call(0x0011_d948, &[0x002b_bb60], &[f4(r)]).1[0];
    m.wf(sp + 8, r);
    m.call(0x0011_dd90, &[sp, a0], &[]);
    stack_pop(m, 0x10);
}

/// FUN_001d53c0: set state +8 = a1 (jump table 0x2a7850): 2 -> FUN_00243870(+0, +4, {+0xc[4], +0xc[5]}); 3/4 -> FUN_001d5468, +0x18 = DAT_002ddf10 * 4.
pub fn ee_sub_1d53c0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if a1 == m.r32(a0 + 8) {
        return;
    }
    match a1 {
        2 => {
            let sp = stack_push(m, 0x10);
            let a3 = m.r32(a0 + 0xc);
            let p0 = m.r32(a0);
            let p1 = m.r32(a0 + 4);
            let b = m.r8(a3 + 4);
            m.w8(sp, b);
            let b = m.r8(a3 + 5);
            m.w8(sp + 1, b);
            m.call(0x0024_3870, &[p0, p1, sp], &[]);
            stack_pop(m, 0x10);
        }
        3 | 4 => {
            m.call(0x001d_5468, &[a0], &[]);
            let f = m.rf(0x002d_df10) * 4.0;
            m.wf(a0 + 0x18, f);
        }
        _ => {}
    }
    m.w32(a0 + 8, a1);
}

/// FUN_001a5020: if a1 && a1[+4] != a1[+0x3c]: FUN_001a4fb0(a0, 0)[+4] = a1.
pub fn ee_sub_1a5020<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if a1 != 0 && (m.r32(a1 + 4) ^ m.r32(a1 + 0x3c)) != 0 {
        let r = m.call(0x001a_4fb0, &[a0, 0], &[]).0;
        m.w32(r + 4, a1);
    }
}

/// FUN_00130598: FUN_001304e0(a0, a1, a2, a3, &out); return out.
pub fn ee_sub_130598<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x10);
    m.call(0x0013_04e0, &[a0, a1, a2, a3, sp], &[]);
    let v = m.r32(sp);
    stack_pop(m, 0x10);
    v
}
