//! WIRE-S1: 1:1 QMem ports of WIRE-ROOTS live-frontier roots (round 3).

use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem};

/// FUN_00164100(node, pcount, list, depth, width): recursive tree flatten — tracks max depth/width
/// (+0x100/+0x104), appends node to list once its visit count (+0xfc) reaches +0x94, recurses children.
pub fn ee_sub_164100<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: i32, t0: i32) {
    let s0 = a0;
    let mut s2 = t0;
    let mut s3 = a3;
    let s4 = a1;
    let s5 = a2;
    let mut d = m.r32(s0 + 0x100) as i32;
    let mut w = m.r32(s0 + 0x104) as i32;
    let v0 = m.r32(s0 + 0xfc).wrapping_add(1);
    if !(s3 < d) {
        d = s3;
    }
    if !(s2 < w) {
        w = s2;
    }
    let lim = m.r32(s0 + 0x94) as i32;
    m.w32(s0 + 0x100, d as u32);
    m.w32(s0 + 0x104, w as u32);
    m.w32(s0 + 0xfc, v0);
    if (v0 as i32) < lim {
        return;
    }
    let n = m.r32(s4);
    m.w32(n.wrapping_shl(2).wrapping_add(s5), s0);
    m.w32(s4, n.wrapping_add(1));
    let p = m.r32(s0 + 0x7c);
    s3 += 1;
    if p != 0 {
        let q = m.r32(p + 0x68);
        s2 += 1;
        if q ^ s0 != 0 {
            s2 = t0;
        }
    }
    if m.r32(s0 + 0xa0) as i32 <= 0 {
        return;
    }
    let sp = stack_push(m, 0x10);
    let mut s1: i32 = 0;
    loop {
        let arr = m.r32(s0 + 0xa4);
        let c = m.r32((s1 as u32).wrapping_shl(2).wrapping_add(arr));
        let x5 = m.r32(s0 + 0xf4);
        let x6 = m.r32(s0 + 0xf8);
        m.w32(sp, 0);
        m.w32(sp + 4, c);
        if c != 0 {
            let v = m.r32(c + 0xf4);
            m.w32(sp + 8, v);
            m.w32(c + 0xf4, x5);
            let c2 = m.r32(sp + 4);
            let v = m.r32(c2 + 0xf8);
            m.w32(sp + 0xc, v);
            m.w32(c2 + 0xf8, x6);
        }
        m.call(0x0016_4100, &[c, s4, s5, s3 as u32, s2 as u32], &[]);
        let v1 = m.r32(sp);
        if v1 != 0 {
            let v = m.r32(sp + 8);
            m.w32(v1 + 0xf0, v);
            let v1 = m.r32(sp);
            let v = m.r32(sp + 0xc);
            m.w32(v1 + 0xf4, v);
        }
        let v1 = m.r32(sp + 4);
        if v1 != 0 {
            let v = m.r32(sp + 8);
            m.w32(v1 + 0xf4, v);
            let v1 = m.r32(sp + 4);
            let v = m.r32(sp + 0xc);
            m.w32(v1 + 0xf8, v);
        }
        s1 += 1;
        if s1 >= m.r32(s0 + 0xa0) as i32 {
            break;
        }
    }
    stack_pop(m, 0x10);
}

/// FUN_00132140(obj, arg): poll sub-lists (+0x150 via 0x1a6290, +0x164 via 0x132b10); returns
/// 1 none/idle, 2 busy w/o +0x178, 3 busy with +0x178, else FUN_001f4fc8 result.
pub fn ee_sub_132140<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let s3 = a0;
    let s4 = a1;
    let mut s2 = 0u32;
    if m.r32(s3 + 0x150) as i32 > 0 {
        let mut s1: i32 = 0;
        let mut s0 = s3 + 0x154;
        loop {
            let e = m.r32(s0);
            s1 += 1;
            s0 += 4;
            let (v0, _) = m.call(0x001a_6290, &[s4, e], &[]);
            let c = s1 < m.r32(s3 + 0x150) as i32;
            if v0 != 0 {
                s2 = 1;
            }
            if !c {
                break;
            }
        }
    }
    if m.r32(s3 + 0x164) as i32 > 0 {
        let mut s1: i32 = 0;
        let mut s0 = s3 + 0x168;
        loop {
            let e = m.r32(s0);
            s1 += 1;
            s0 += 4;
            let (v0, _) = m.call(0x0013_2b10, &[s4, e], &[]);
            let c = s1 < m.r32(s3 + 0x164) as i32;
            if v0 != 0 {
                s2 = 1;
            }
            if !c {
                break;
            }
        }
    }
    if m.r32(s3 + 0x178) != 0 {
        if s2 != 0 {
            return 3;
        }
        m.call(0x001f_4fc8, &[s3, s4], &[]).0
    } else if s2 != 0 {
        2
    } else {
        1
    }
}

/// refcounted node release (inline in FUN_001272e0): --*p == 0 -> destroy + free to pool 0x4520a8
fn rel_1272e0<M: QMem + ?Sized>(m: &mut M, s0: u32) {
    if s0 == 0 {
        return;
    }
    let v = m.r32(s0).wrapping_sub(1);
    m.w32(s0, v);
    if v == 0 {
        m.call(0x0025_6d40, &[s0 + 0x10, 2], &[]);
        m.call(0x0012_9ec8, &[s0 + 8], &[]);
        m.call(0x0025_50f8, &[s0], &[]);
        m.call(0x0011_cd98, &[0x0045_20a8, s0], &[]);
    }
}

/// FUN_001272e0(ret, expr, env): evaluate a call form — resolve the head (0x19 builtin / 0x20 method
/// / 0x22 closure), evaluate fixed + rest args into a 32-slot arg array, invoke, copy result to `ret`.
pub fn ee_sub_1272e0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) -> u32 {
    let sp = stack_push(m, 0x1a0);
    let fp = a2;
    m.w32(sp + 0x180, a0);
    m.w32(sp, 0xffff_ffff);
    let mut s0 = m.r32(a1 + 0xc);
    m.w32(sp + 0x184, 0);
    m.w32(sp + 0x188, 0);
    m.w32(sp + 0x18c, 0);
    m.call(0x0012_7838, &[sp + 0x10, s0, a2], &[]);
    let v1 = m.r32(sp + 0x10);
    let s4: u32;
    let s5: u32;
    if v1 == 0x19 {
        let i = m.r32(sp + 0x14);
        let p = i.wrapping_mul(0xc).wrapping_add(0x002b_bb90);
        m.w32(sp + 0x184, p);
        s4 = m.r32(p + 4);
        s5 = m.r32(p + 8);
    } else if v1 != 0x22 {
        let p = m.r32(sp + 0x14);
        m.w32(sp + 0x188, p);
        s4 = m.r32(p + 0xc);
        s5 = m.r32(p + 0x10);
    } else {
        let p = m.r32(sp + 0x14);
        s5 = 0;
        m.w32(sp + 0x18c, p);
        s4 = m.r32(p + 0x10);
    }
    m.w32(sp + 0x190, s4.wrapping_add((s5 != 0) as u32));
    let s7 = sp + 0x120;
    for k in 0..32u32 {
        m.w32(sp + 0x20 + k * 8, 0xffff_ffff);
    }
    s0 = m.r32(s0 + 0x10);
    let mut s6 = s4;
    let mut s1: i32 = 0;
    if s6 as i32 > 0 {
        let s3 = s7;
        let mut s2 = sp + 0x20;
        loop {
            m.call(0x0012_7838, &[s3, s0, fp], &[]);
            s1 += 1;
            m.call(0x0012_9658, &[sp, s3], &[]);
            m.call(0x0012_9ec8, &[s3], &[]);
            m.call(0x0012_9658, &[s2, sp], &[]);
            s2 += 8;
            let c = s1 < s4 as i32;
            s0 = m.r32(s0 + 0x10);
            if !c {
                break;
            }
        }
    }
    if s5 != 0 {
        m.call(0x0025_6c38, &[sp + 0x120, s0], &[]);
        let s2 = (s1 as u32) << 3;
        m.w32(sp + 0x130, 0);
        m.w32(sp + 0x140, 0);
        if m.r32(sp + 0x120) != 0 {
            let s1 = sp + 0x160;
            loop {
                let (v0, _) = m.call(0x0025_6b48, &[], &[]);
                m.call(0x0025_6c38, &[sp + 0x150, v0], &[]);
                let n = m.r32(sp + 0x150);
                let e = m.r32(sp + 0x120);
                m.call(0x0012_7838, &[s1, e, fp], &[]);
                m.call(0x0012_9658, &[n + 8, s1], &[]);
                m.call(0x0012_9ec8, &[s1], &[]);
                let t = m.r32(sp + 0x130);
                let n = m.r32(sp + 0x150);
                if t != 0 {
                    m.call(0x0025_6c58, &[t + 0x10, n], &[]);
                    let n = m.r32(sp + 0x150);
                    m.call(0x0025_6c58, &[sp + 0x130, n], &[]);
                } else {
                    m.call(0x0025_6c58, &[sp + 0x130, n], &[]);
                    let t = m.r32(sp + 0x130);
                    m.call(0x0025_6c58, &[sp + 0x140, t], &[]);
                }
                let e = m.r32(sp + 0x120);
                s6 = s6.wrapping_add(1);
                let nx = m.r32(e + 0x10);
                m.call(0x0025_6c58, &[sp + 0x120, nx], &[]);
                let n = m.r32(sp + 0x150);
                rel_1272e0(m, n);
                if m.r32(sp + 0x120) == 0 {
                    break;
                }
            }
        }
        let a1v = m.r32(sp + 0x140);
        if a1v != 0 {
            m.call(0x0012_9970, &[s2 + sp + 0x20, a1v], &[]);
        } else {
            let p = s2 + sp + 0x20;
            m.call(0x0012_9ec8, &[p], &[]);
            m.call(0x0012_a0b0, &[p, 0], &[]);
        }
        let h = m.r32(sp + 0x140);
        rel_1272e0(m, h);
        let h = m.r32(sp + 0x130);
        rel_1272e0(m, h);
        let h = m.r32(sp + 0x120);
        rel_1272e0(m, h);
    }
    let v1 = m.r32(sp + 0x10);
    if v1 == 0x20 {
        let (v0, _) = m.call(0x0025_6dd0, &[], &[]);
        m.call(0x0025_7058, &[sp + 0x120, v0], &[]);
        let c = m.r32(sp + 0x188);
        let o = m.r32(sp + 0x120);
        let x = m.r32(c + 8);
        m.call(0x0012_7b18, &[o, x], &[]);
        let c = m.r32(sp + 0x188);
        let s3 = sp + 0x170;
        let cnt = m.r32(sp + 0x190);
        let mut s0 = m.r32(c + 0x14);
        if cnt as i32 > 0 {
            let mut s1 = cnt;
            let mut s2 = sp + 0x20;
            loop {
                if m.r32(s0 + 8) == 0x14 {
                    s0 = m.r32(s0 + 0x10);
                }
                let a2v = m.r32(s0 + 0xc);
                let o = m.r32(sp + 0x120);
                m.call(0x0012_7c00, &[s3, o, a2v, s2], &[]);
                s2 += 8;
                s1 = s1.wrapping_sub(1);
                s0 = m.r32(s0 + 0x10);
                m.call(0x0012_9ec8, &[s3], &[]);
                if s1 == 0 {
                    break;
                }
            }
        }
        let c = m.r32(sp + 0x188);
        let o = m.r32(sp + 0x120);
        m.call(0x0012_7190, &[s3, c, o], &[]);
        m.call(0x0012_9658, &[sp, s3], &[]);
        m.call(0x0012_9ec8, &[s3], &[]);
        let o = m.r32(sp + 0x120);
        if o != 0 {
            let v = m.r32(o).wrapping_sub(1);
            m.w32(o, v);
            if v == 0 {
                m.call(0x0025_7078, &[o, 3], &[]);
            }
        }
    } else if v1 == 0x19 {
        let p = m.r32(sp + 0x184);
        let f = m.r32(p);
        m.call(f, &[s7, s6, sp + 0x20, fp], &[]);
        m.call(0x0012_9658, &[sp, s7], &[]);
        m.call(0x0012_9ec8, &[s7], &[]);
    } else if v1 == 0x22 {
        let p = m.r32(sp + 0x18c);
        let f = m.r32(p + 0xc);
        let x = m.r32(p + 8);
        m.call(f, &[s7, x, s6, sp + 0x20], &[]);
        m.call(0x0012_9658, &[sp, s7], &[]);
        m.call(0x0012_9ec8, &[s7], &[]);
    }
    let r = m.r32(sp + 0x180);
    m.call(0x0012_91a8, &[r, sp], &[]);
    let mut s0 = s7;
    while s0 != sp + 0x20 {
        s0 -= 8;
        m.call(0x0012_9ec8, &[s0], &[]);
    }
    m.call(0x0012_9ec8, &[sp + 0x10], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    let r = m.r32(sp + 0x180);
    stack_pop(m, 0x1a0);
    r
}

/// FUN_00126ba0(ret, expr, env): evaluate a cond-like form — test each clause (type 0xb = else),
/// on first true clause evaluate its body forms; last value copied to `ret`.
pub fn ee_sub_126ba0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) -> u32 {
    let sp = stack_push(m, 0x50);
    let s6 = a2;
    let s5 = sp + 0x30;
    let fp = sp + 0x10;
    m.w32(sp, 0xffff_ffff);
    m.w32(sp + 0x40, a0);
    m.w32(sp + 0x44, sp + 0x20);
    m.w32(sp + 0x10, 0xffff_ffff);
    m.w32(sp + 0x18, 0xffff_ffff);
    m.w32(sp + 0x20, 0xffff_ffff);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    let h = m.r32(a1 + 0xc);
    let s0 = m.r32(h + 0x10);
    m.call(0x0012_7838, &[s5, s0, s6], &[]);
    m.call(0x0012_9658, &[fp, s5], &[]);
    m.call(0x0012_9ec8, &[s5], &[]);
    let mut s3 = m.r32(s0 + 0x10);
    while s3 != 0 {
        let mut s1 = m.r32(s3 + 0xc);
        let mut s4: u32;
        if m.r32(s1 + 8) == 0xb {
            s4 = 1;
        } else {
            let mut s0 = m.r32(s1 + 0xc);
            s4 = 0;
            while s0 != 0 {
                m.call(0x0012_9658, &[sp + 0x18, s0 + 8], &[]);
                m.call(0x0012_0548, &[s5, 2, fp, s6], &[]);
                let d = m.r32(sp + 0x44);
                m.call(0x0012_9658, &[d, s5], &[]);
                m.call(0x0012_9ec8, &[s5], &[]);
                if m.r32(sp + 0x20) != 0x17 || m.r32(sp + 0x24) == 0 {
                    s0 = m.r32(s0 + 0x10);
                } else {
                    s4 = 1;
                    s0 = 0;
                }
            }
        }
        if s4 == 0 {
            s3 = m.r32(s3 + 0x10);
        } else {
            s1 = m.r32(s1 + 0x10);
            while s1 != 0 {
                m.call(0x0012_7838, &[s5, s1, s6], &[]);
                m.call(0x0012_9658, &[sp, s5], &[]);
                m.call(0x0012_9ec8, &[s5], &[]);
                s1 = m.r32(s1 + 0x10);
            }
            s3 = 0;
        }
    }
    let r = m.r32(sp + 0x40);
    m.call(0x0012_91a8, &[r, sp], &[]);
    let mut s0 = m.r32(sp + 0x44);
    m.call(0x0012_9ec8, &[sp + 0x20], &[]);
    while fp != s0 {
        s0 = s0.wrapping_sub(8);
        m.call(0x0012_9ec8, &[s0], &[]);
    }
    m.call(0x0012_9ec8, &[sp], &[]);
    let r = m.r32(sp + 0x40);
    stack_pop(m, 0x50);
    r
}

/// FUN_00219348(obj, i, j): for each index of group i (+4 table, 0x20 stride) refresh the 0x70-stride
/// entries of set j: FUN_002158c0 if entry +0x30 set, else FUN_002156b8.
pub fn ee_sub_219348<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    let v1 = m.r32(a0 + 0x10);
    let base = m.r32(a0 + 4);
    let t = m.r32(v1 + 8);
    let s1 = base.wrapping_add(a1 << 5);
    let p = a2.wrapping_mul(0x18).wrapping_add(t);
    let s2 = m.r32(p + 4);
    let mut s0: i32 = 0;
    if m.r8(s1 + 6) == 0 {
        return;
    }
    loop {
        let ix = m.r32(s1 + 0x14);
        let k = m.r16(((s0 as u32) << 1).wrapping_add(ix)) as u32;
        let e = k.wrapping_mul(0x70).wrapping_add(s2);
        let q = m.r32(e + 0x30);
        if q != 0 {
            m.call(0x0021_58c0, &[q, e], &[]);
        } else {
            m.call(0x0021_56b8, &[e], &[]);
        }
        s0 += 1;
        if s0 >= m.r8(s1 + 6) as i32 {
            break;
        }
    }
}
