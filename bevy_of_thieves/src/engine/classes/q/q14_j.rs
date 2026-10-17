//! WP-Q14 part J: FUN_001ca500 (per-group instance setup) and FUN_001caa60 (remove a node from the group's
//! list and bind it to the group's next free object). Scratch pool FUN_001ae2a0 is called through QMem.
#![allow(unused_variables, unused_mut, dead_code)]
use crate::engine::classes::q::q14_h::ee_sub_1ca930;
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem, V4};
use crate::engine::classes::q::q6::vcall;

fn pool_alloc<M: QMem + ?Sized>(m: &mut M, size: u32) -> u32 {
    let s = stack_push(m, 0x10);
    m.w32(s, size);
    m.w32(s + 4, 0x10);
    let r = m.call(0x001a_e2a0, &[s], &[]).0;
    stack_pop(m, 0x10);
    r
}

/// FUN_001ca500(p)
pub fn ee_sub_1ca500<M: QMem + ?Sized>(m: &mut M, p: u32) {
    let mut gi: i32 = 0;
    if 0 < m.r32(p + 0x3f04) as i32 {
        loop {
            let next = gi + 1;
            let e = m.r32(p + 0x3f08).wrapping_add((gi as u32).wrapping_mul(0x6c));
            let mut j = m.r32(e + 0xc) as i32;
            if j < m.r32(e + 8) as i32 {
                loop {
                    let u = m.call(0x001c_9940, &[gi as u32, j as u32], &[]).0;
                    let t = m.r32(e + 0x18);
                    let v = m.r32(e + 0x14);
                    m.w32((j as u32).wrapping_mul(4).wrapping_add(t), v);
                    let mut cnt = m.r32(e) as i32;
                    let jn = j + 1;
                    let mut k = 0i32;
                    if 0 < cnt {
                        loop {
                            let idx = j.wrapping_mul(cnt).wrapping_add(k);
                            k += 1;
                            let tb = m.r32(e + 4);
                            let o = m.r32((idx as u32).wrapping_mul(4).wrapping_add(tb));
                            vcall(m, o, 0xc, 0xcc, &[u, 1], &[]);
                            let q = m.r32(o + 0x3c);
                            m.w32(q + 0x24, 1);
                            cnt = m.r32(e) as i32;
                            if cnt <= k {
                                break;
                            }
                        }
                    }
                    j = jn;
                    if m.r32(e + 8) as i32 <= j {
                        break;
                    }
                }
                let cnt0 = m.r32(e);
                let buf = pool_alloc(m, cnt0 << 6);
                let mut cnt = m.r32(e) as i32;
                m.w32(e + 0x64, buf);
                let mut k = 0i32;
                if 0 < cnt {
                    let mut j0 = m.r32(e + 0xc) as i32;
                    loop {
                        let dst = m.r32(e + 0x64).wrapping_add((k as u32).wrapping_mul(0x40));
                        let idx = j0.wrapping_mul(cnt).wrapping_add(k);
                        k += 1;
                        let tb = m.r32(e + 4);
                        let o = m.r32((idx as u32).wrapping_mul(4).wrapping_add(tb));
                        let pv = m.call(0x001a_62e8, &[o], &[]).1;
                        m.wv(dst, pv);
                        let mp = m.call(0x001a_6318, &[o], &[]).0;
                        for q in 0..3 {
                            let w = m.rv(mp + 0x10 * q);
                            m.wv(dst + 0x10 + 0x10 * q, w);
                        }
                        m.call(0x0011_b3d0, &[dst + 0x10], &[]);
                        cnt = m.r32(e) as i32;
                        if cnt <= k {
                            break;
                        }
                        j0 = m.r32(e + 0xc) as i32;
                    }
                }
                let n2 = m.r32(e + 8).wrapping_sub(m.r32(e + 0xc));
                let buf2 = pool_alloc(m, n2.wrapping_mul(0x50));
                let mut i2 = m.r32(e + 0xc) as i32;
                m.w32(e + 0x68, buf2);
                if i2 < m.r32(e + 8) as i32 {
                    let mut i1 = m.r32(e + 0xc) as i32;
                    loop {
                        let rec = ((i2 - i1) as u32).wrapping_mul(0x50).wrapping_add(m.r32(e + 0x68));
                        i2 += 1;
                        m.call(0x0023_33b4, &[rec + 0x10, 0, 0x40], &[]);
                        m.w32(rec + 0x10, 0x3f80_0000);
                        m.w32(rec + 0x4c, 0x3f80_0000);
                        m.w32(rec + 0x38, 0x3f80_0000);
                        m.w32(rec + 0x24, 0x3f80_0000);
                        if m.r32(e + 8) as i32 <= i2 {
                            break;
                        }
                        i1 = m.r32(e + 0xc) as i32;
                    }
                }
            }
            gi = next;
            if m.r32(p + 0x3f04) as i32 <= next {
                break;
            }
        }
    }
}

/// FUN_001caa60(s, node)
pub fn ee_sub_1caa60<M: QMem + ?Sized>(m: &mut M, s: u32, node: u32) {
    let head = m.r32(s + 0xe8);
    let mut found = true;
    if head != 0 {
        if head == node {
            let nx = m.r32(node + 4);
            m.w32(s + 0xe8, nx);
        } else {
            let mut cur = head;
            loop {
                let link = cur + 4;
                cur = m.r32(link);
                if cur == 0 {
                    found = false;
                    break;
                }
                if cur == node {
                    let nx = m.r32(node + 4);
                    m.w32(link, nx);
                    break;
                }
            }
        }
        if found {
            m.w32(node + 4, 0);
        }
    }
    m.w32(node, 0);
    let sp = stack_push(m, 0x90);
    let (mat, w_child, w_self) = (sp, sp + 0x40, sp + 0x44);
    m.call(0x0011_c318, &[mat, node + 0x10], &[]);
    m.call(0x0023_33b4, &[node + 0x10, 0, 0x40], &[]);
    for off in [0x24u32, 0x4c, 0x38, 0x10] {
        m.w32(node + off, 0x3f80_0000);
    }
    let idx = ee_sub_1ca930(m, s, node as i32) as u32;
    m.w32(w_self, s);
    let g = m.r32(s + 0xe0);
    let mut n = m.r32(g) as i32;
    if 0 < n {
        let mut i = 0i32;
        loop {
            let nxt = i + 1;
            let g = m.r32(s + 0xe0);
            let tb = m.r32(g + 4);
            let child = m.r32(tb.wrapping_add(((idx as i32).wrapping_mul(n).wrapping_add(i) as u32).wrapping_mul(4)));
            m.w32(w_child, child);
            let cnt = m.r32(s + 0xec) as i32;
            let mut k = 0i32;
            let mut off = 0u32;
            while k < cnt {
                let rec = m.r32(s + 0xf0).wrapping_add(off);
                if m.r32(rec) != 5 {
                    let x = m.r32(rec + 8);
                    if x == 0 {
                        m.w32(rec + 0x14, 0);
                    } else if m.r32(s + 0xe4) == 0 {
                        let pp = m.r32(rec + 0x14);
                        if m.r32(x + 8) == 0 {
                            m.call(0x0025_6ee0, &[pp, 0], &[]);
                        } else {
                            let c = m.r32(x + 0xc);
                            m.call(0x0025_6ee0, &[pp, c], &[]);
                            let w = m.r32(pp);
                            m.call(0x0012_7b70, &[x, w], &[]);
                        }
                        m.w32(rec + 0x14, 0);
                    } else {
                        m.w32(rec + 0x14, 0);
                    }
                }
                k += 1;
                off += 0x18;
            }
            vcall(m, child, 0xc, 0xdc, &[s], &[]);
            vcall(m, s, 0xc, 0x1c, &[0xf, 0, 1, w_child, 0], &[]);
            vcall(m, child, 0xc, 0x1c, &[0xf, 0, 1, w_self, 0], &[]);
            vcall(m, child, 0xc, 0x4c, &[], &[]);
            vcall(m, child, 0xc, 0x12c, &[mat], &[]);
            m.call(0x001a_58d0, &[child, 0, 0], &[]);
            if m.call(0x0013_2b10, &[child, 0x30], &[]).0 != 0 {
                let g2 = m.r32(s + 0xe0);
                let p5 = m.r32(g2 + 100).wrapping_add((i as u32).wrapping_mul(0x40));
                let c = m.rv(0x002a_2dd0);
                if m.r32(child + 0x58) != 0 {
                    let a = m.rv(p5);
                    vcall(m, child, 0xc, 0xec, &[], &[a]);
                    vcall(m, child, 0xc, 0x11c, &[], &[c]);
                }
                if m.r32(child + 0x5c) != 0 {
                    let a = m.rv(p5 + 16);
                    vcall(m, child, 0xc, 0xf4, &[], &[a]);
                    vcall(m, child, 0xc, 0x124, &[], &[c]);
                }
            }
            let g = m.r32(s + 0xe0);
            n = m.r32(g) as i32;
            if n <= nxt {
                break;
            }
            i = nxt;
        }
    }
    stack_pop(m, 0x90);
}
