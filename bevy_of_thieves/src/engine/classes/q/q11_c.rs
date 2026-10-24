//! WP-Q11 part C: class packages C2d76e8, C2d7738, C2d77d8, C2d7a40, C2d83e8, C2d8468, C2d84e8 ...
//! 1:1 from ghidra_functions.body on `QMem` (conventions as q11_a). Switch targets of FUN_001dd558 were read from the ELF
//! jump table 0x002a7a50 (disassembly).
#![allow(unused_variables, unused_mut, dead_code)]
use crate::engine::classes::q::q11_a::dist3;
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};
use crate::engine::classes::q::q6::vcall;
use crate::engine::classes::q::q6::{ro32, rof, rov};
use crate::engine::classes::q::q7util::{f, safe_walk, ITER_CHAIN, CLOCK};
use crate::engine::classes::q::q8util::{frame_alloc, frame_begin, frame_end};

const NEG1: u32 = u32::MAX;
const WORLD: u32 = 0x003d_d490;

/// open an iterator record on DAT_002dd2fc that stays registered until `walk_close` (the originals restore
/// the chain only at the end of the function): returns (record, first element)
pub fn walk_open<M: QMem + ?Sized>(m: &mut M, list: u32) -> (u32, u32) {
    let it = stack_push(m, 0x10);
    let off = m.r32(list + 8);
    let c = m.r32(list);
    m.w32(it, list);
    m.w32(it + 4, c.wrapping_add(off));
    m.w32(it + 8, off);
    let prev = m.r32(ITER_CHAIN);
    m.w32(it + 0xc, prev);
    m.w32(ITER_CHAIN, it);
    (it, c)
}
/// next element: `c = *next; next = c + off`
pub fn walk_next<M: QMem + ?Sized>(m: &mut M, it: u32) -> u32 {
    let nx = m.r32(it + 4);
    let c = m.r32(nx);
    let off = m.r32(it + 8);
    m.w32(it + 4, c.wrapping_add(off));
    c
}
pub fn walk_close<M: QMem + ?Sized>(m: &mut M, it: u32) {
    let prev = m.r32(it + 0xc);
    m.w32(ITER_CHAIN, prev);
    stack_pop(m, 0x10);
}
fn pos<M: QMem + ?Sized>(m: &mut M, obj: u32) -> V4 {
    m.call(0x001a_62e8, &[obj], &[]).1
}
fn copy_words<M: QMem + ?Sized>(m: &mut M, dst: u32, src: u32, n: u32) {
    for i in 0..n {
        let w = m.r32(src + 4 * i);
        m.w32(dst + 4 * i, w);
    }
}

// ---- C2d76e8 ----------------------------------------------------------------------------------------

/// FUN_001cfa20 (v4)
pub fn c2d76e8_v4<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001c_f268, &[o], &[]);
}
/// FUN_001cfa40 (v5)
pub fn c2d76e8_v5<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let sp = stack_push(m, 0x70);
    let (b60, b90) = (sp, sp + 0x40);
    let a = m.r32(o + 0xf0);
    let p70 = m.rv(o + 0x70);
    let q = m.call(0x0022_fc48, &[a, 0], &[p70]).1;
    let a = m.r32(o + 0xf0);
    m.call(0x0022_fd38, &[b60, a, 0, o + 0x40], &[]);
    copy_words(m, b90, b60, 12);
    m.call(0x001c_f338, &[o, b90], &[q]);
    stack_pop(m, 0x70);
}

// ---- C2d7738 ----------------------------------------------------------------------------------------

/// FUN_001cf618 (v5)
pub fn c2d7738_v5<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let sp = stack_push(m, 0x70);
    let (b50, b90) = (sp, sp + 0x40);
    let ax = if m.r32(o + 0xf4) == 0 {
        rov(0x002a_2dc0)
    } else {
        let v = m.rv(0x002d_dfa0);
        v.map(|x| x * -1.0)
    };
    let d = m.rv(o + 0x90);
    m.call(0x0011_b830, &[b50], &[d, ax]);
    copy_words(m, b90, b50, 12);
    let p70 = m.rv(o + 0x70);
    m.call(0x001c_f338, &[o, b90], &[p70]);
    stack_pop(m, 0x70);
}
/// FUN_001cf968 (v7): (o, ptr, quad a2, quad a3)
pub fn c2d7738_v7<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32, q3: V4, q4: V4) -> u32 {
    let a = m.r32(p2 + 4);
    if m.call(0x0013_2b10, &[a, ro32(0x002c_1f70)], &[]).0 == 0 {
        let a = m.r32(p2 + 4);
        if m.call(0x0013_2b10, &[a, 0x59], &[]).0 == 0 {
            let (b, c) = (m.r32(o + 0xf0), m.r32(p2));
            m.call(0x001c_f6b0, &[o, b, c], &[q3, q4]);
        }
        m.call(0x001c_e980, &[o], &[]);
        1
    } else {
        0
    }
}

// ---- C2d77d8 ----------------------------------------------------------------------------------------

/// FUN_001cddc8 (v6)
pub fn c2d77d8_v6<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001f_2108, &[o], &[]);
    m.call(0x0011_a1f8, &[o + 0x15c, 0xc], &[]);
    m.call(0x001a_6ca8, &[o, 1], &[]);
    let p = m.r32(o + 0x40);
    if (p ^ 1) & 1 == 0 {
        m.w32(o + 0x40, (p & 1) | 0xe);
    } else {
        let v = m.r32(p);
        m.w32(p, (v & 1) | 0xe);
    }
    let v = m.r64(o + 0x148) & 0xffff_fff7_ffff_ffff;
    m.w32(o + 0x148, v as u32);
    m.w32(o + 0x14c, (v >> 32) as u32);
    if m.r32(o + 0x150) == 1 {
        m.call(0x001f_27f0, &[o], &[]);
        m.call(0x001f_6860, &[o, 1], &[]);
    }
}
/// FUN_001cde80 (v11)
pub fn c2d77d8_v11<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let (it, mut c) = walk_open(m, o + 0x15c);
    while c != 0 {
        m.call(0x001c_e980, &[c], &[]);
        c = walk_next(m, it);
    }
    if o == m.r32(0x003e_14fc) {
        m.w32(0x003e_14fc, 0);
    }
    m.call(0x001f_2d48, &[o], &[]);
    if m.r32(o + 0xec) != 0 {
        m.call(0x0013_37d0, &[WORLD], &[]);
        m.w32(o + 0xec, 0);
    }
    m.call(0x001a_57a8, &[o], &[]);
    let u = m.call(0x0015_e3a8, &[WORLD, 0x57], &[]).0;
    m.call(0x0011_a118, &[u, o], &[]);
    if m.r32(o + 0x10) == 0 {
        m.call(0x001a_53e0, &[o], &[]);
    }
    m.call(0x0011_cd98, &[0x003e_14cc, o], &[]);
    walk_close(m, it);
}
/// FUN_001cdf88 (v16): bounding box of the children (+0x70 / +0x80 corners) into +0x170 / +0x180, centre +0x190, radius +0x1a0
pub fn c2d77d8_v16<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let mut mn: V4 = [rof(0x002a_77d0); 4];
    let mut mx: V4 = [rof(0x002a_77d4); 4];
    mn[3] = 0.0;
    mx[3] = 0.0;
    let (it, mut c) = walk_open(m, o + 0x15c);
    while c != 0 {
        if m.rf(c + 0x14) <= m.rf(CLOCK) {
            vslot(m, c, 3, &[], &[]);
        }
        let a = m.rv(c + 0x70);
        let b = m.rv(c + 0x80);
        for i in 0..3 {
            mn[i] = mn[i].min(a[i].min(b[i]));
            mx[i] = mx[i].max(a[i].max(b[i]));
        }
        c = walk_next(m, it);
    }
    let fv = m.rf(o + 0x154);
    if fv != 0.0 {
        for i in 0..3 {
            mn[i] -= fv;
            mx[i] -= fv;
        }
    }
    if m.r32(o + 0xec) != 0 {
        let ctr: V4 = std::array::from_fn(|i| mn[i] * 0.5 + mx[i] * 0.5);
        let rad = dist3(ctr, mx);
        m.wv(o + 0x170, mn);
        m.wv(o + 0x180, mx);
        m.wf(o + 0x1a0, rad);
        m.wv(o + 400, ctr);
        m.call(0x001f_4b58, &[o], &[]);
    }
    walk_close(m, it);
}
/// FUN_001ce170 (v48)
pub fn c2d77d8_v48<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let p = m.r32(o + 0xec);
    let q = m.rv(o + 0x170);
    m.wv(p, q);
    let p = m.r32(o + 0xec);
    let q = m.rv(o + 0x180);
    m.wv(p + 0x10, q);
    let p = m.r32(o + 0xec);
    let q = m.rv(o + 400);
    m.wv(p + 0x20, q);
    let p = m.r32(o + 0xec);
    let v = m.r32(o + 0x1a0);
    m.w32(p + 0x30, v);
    m.w32(0x003e_115c, 1);
    m.call(0x001f_4ba8, &[o], &[]);
}
/// FUN_001ce1c8 (v15)
pub fn c2d77d8_v15<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let (it, mut c) = walk_open(m, o + 0x15c);
    while c != 0 {
        if m.rf(c + 0x14) < m.rf(CLOCK) {
            m.call(0x001c_e9c0, &[c], &[]);
            vslot(m, c, 4, &[], &[]);
            m.call(0x001c_e9d0, &[c], &[]);
        }
        c = walk_next(m, it);
    }
    if m.call(0x001a_61a0, &[o], &[]).0 != 0 {
        if m.call(0x0011_a3b0, &[o + 0x15c], &[]).0 == 0 {
            ee_sub_1ce2e8(m, o);
            walk_close(m, it);
            return;
        }
    }
    vslot(m, o, 8, &[], &[]);
    walk_close(m, it);
}
/// FUN_001ce2e8
pub fn ee_sub_1ce2e8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.r32(o + 0x150) == 1 {
        let hdr = o + 0x15c;
        let ob = m.r32(o + 0xec);
        let mut n8 = m.r32(ob + 100);
        if n8 != 0 {
            let mut n5 = m.r32(n8 + 0xc);
            loop {
                let e = m.r32(n5 + 0xec);
                if m.r64(e + 0x70) & 4 == 0 {
                    let g = m.r32(e + 0x60);
                    let b8 = m.call(0x0013_2b10, &[g, 0x59], &[]).0;
                    let t = m.r32(n8 + 0xc);
                    let t = m.r32(t + 0xec);
                    safe_walk(m, t + 0x34, |m, e5| {
                        if (m.r64(e5 + 0x10) & 0x0001_0000_0000_0000) == 0 || b8 != 0 {
                            let fr = frame_begin(m);
                            let mut bc = 0u32;
                            let mut skip = false;
                            if m.r64(e5 + 0x10) & 0x0002_0000_0000_0000 == 0 {
                                let mut c0 = 0x1555u32;
                                if (m.r32(e5 + 0x6c) as i32) < 0x1555 {
                                    c0 = m.r32(e5 + 0x6c);
                                }
                                bc = frame_alloc(m, c0.wrapping_mul(0xc), 0x10);
                                let pc = stack_push(m, 8);
                                m.w32(pc, c0);
                                m.w32(pc + 4, bc);
                                let ob = m.r32(o + 0xec);
                                let (a, b, d) = (m.r32(ob + 0x30), m.r32(e5 + 0x74), m.rv(ob + 0x20));
                                let l4 = m.call(0x0011_8a88, &[a, b, 0x7fff_ffff, pc, pc + 4], &[d]).0;
                                c0 = m.r32(pc);
                                bc = m.r32(pc + 4);
                                stack_pop(m, 8);
                                if l4 != 2 {
                                    frame_end(m, fr, 2);
                                    skip = true;
                                } else {
                                    bc = bc.wrapping_sub(0xc);
                                }
                            }
                            if !skip {
                                let buf = stack_push(m, 0x300);
                                safe_walk(m, hdr, |m, c1| {
                                    if m.rf(c1 + 0x14) < m.rf(CLOCK) {
                                        let (a, b) = (m.rv(c1 + 0x80), m.rv(c1 + 0x70));
                                        let n = m.call(0x0014_c6d8, &[e5, bc, 4, buf], &[a, b]).0 as i32;
                                        let mut p6 = buf;
                                        let mut i7 = 0;
                                        if 0 < n {
                                            loop {
                                                if m.rf(p6 + 0x20) != 0.0 {
                                                    let (x, y) = (m.r32(p6), m.r32(p6 + 0x10));
                                                    if vcall(m, c1, 0xec, 0x44, &[e5, x, y], &[]).0 != 0 {
                                                        break;
                                                    }
                                                }
                                                i7 += 1;
                                                p6 += 0x30;
                                                if !(i7 < n) {
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                });
                                stack_pop(m, 0x300);
                                frame_end(m, fr, 2);
                            }
                        }
                    });
                }
                n8 = m.r32(n8 + 4);
                if n8 == 0 {
                    break;
                }
                n5 = m.r32(n8 + 0xc);
            }
        }
        if m.call(0x0011_a3b0, &[hdr], &[]).0 != 0 {
            vslot(m, o, 8, &[], &[]);
        }
    }
}
/// FUN_001ce608 (v18)
pub fn c2d77d8_v18<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let (it, mut c) = walk_open(m, o + 0x15c);
    while c != 0 {
        if m.rf(c + 0x14) < m.rf(CLOCK) {
            vcall(m, c, 0xec, 0x34, &[], &[]);
        }
        c = walk_next(m, it);
    }
    walk_close(m, it);
}

// ---- C2d7a40 / C2d83e8 / C2d8468 / C2d84e8 ---------------------------------------------------------------

/// FUN_001d4e68 (C2d7a40_v6)
pub fn c2d7a40_v6<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001f_2108, &[o], &[]);
    let p = m.r32(o + 0x40);
    m.w32(o + 0x15c, 0x1e);
    m.w32(o + 0x150, NEG1);
    crate::engine::classes::q::q6::set_tagged(m, o + 0x40, 2);
}
/// FUN_001d4f50 (C2d7a40_v15)
pub fn c2d7a40_v15<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001f_36e0, &[o], &[]);
    if m.r32(o + 0x150) == 0 {
        let p = m.r32(o + 0x58);
        if m.r16(p + 0x98) as i16 == 3 {
            m.call(0x001d_4fa0, &[o, 1], &[]);
        }
    }
}
/// FUN_001dd690 (C2d83e8_v11)
pub fn c2d83e8_v11<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001d_a770, &[o], &[]);
    let c = m.r32(o + 0x24);
    if c != 0 {
        m.w32(c + 0xb0, 1);
    }
}
/// FUN_001dd7b8 (C2d83e8_v13)
pub fn c2d83e8_v13<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32) {
    if p2 != m.r32(o + 0x28) {
        m.w32(o + 0x28, p2);
        let v = m.r32(0x002d_df0c);
        m.w32(o + 0x2c, v);
    }
}
/// FUN_001dcd30 (C2d8468_v6)
pub fn c2d8468_v6<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001d_a808, &[o], &[]);
    if m.r32(o + 0x40) == NEG1 || m.r32(o + 0x10) == 0 {
        return;
    }
    let c = m.r32(o + 0x24);
    if m.r32(c + 0x58) == 0 {
        if m.call(0x0021_27c8, &[c], &[]).0 == 0 {
            return;
        }
        let c = m.r32(o + 0x24);
        m.call(0x0021_2a70, &[c, 1], &[]);
        let c = m.r32(o + 0x24);
        m.call(0x0021_2a50, &[c, 2], &[]);
        let c = m.r32(o + 0x24);
        vslot(m, c, 15, &[], &[]);
    }
    let mut ow = m.r32(o + 0x10);
    let got = m.call(0x0018_7d80, &[ow, 0x800], &[]).0 != 0;
    if got {
        m.call(0x001d_d300, &[o, NEG1], &[]);
        return;
    }
    let st = m.r32(o + 0x40);
    if st == 0 {
        let ow = m.r32(o + 0x10);
        if m.call(0x0018_7d80, &[ow, 0x10], &[]).0 != 0 {
            m.call(0x001d_d300, &[o, NEG1], &[]);
            return;
        }
        if m.r32(o + 0x48) != 0 {
            let ow = m.r32(o + 0x10);
            if m.call(0x0018_7d80, &[ow, 0xf0], &[]).0 != 0 {
                m.call(0x001d_d300, &[o, 1], &[]);
                return;
            }
        }
    } else if st == 1 {
        let mut ow = m.r32(o + 0x10);
        if m.r32(o + 0x114) == 0 {
            if m.call(0x0018_7d80, &[ow, 0x10], &[]).0 != 0 {
                m.call(0x001d_d300, &[o, NEG1], &[]);
                return;
            }
            ow = m.r32(o + 0x10);
        }
        if m.call(0x0018_7d80, &[ow, 0xf0], &[]).0 != 0 {
            m.call(0x001d_d300, &[o, 0], &[]);
            return;
        }
    }
    m.call(0x001d_a858, &[o], &[]);
    ee_sub_1dd1f8(m, o);
}
/// FUN_001dd1f8
pub fn ee_sub_1dd1f8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let c = m.r32(o + 0x24);
    if m.r32(c + 0x58) == 2 && 0 < m.r32(o + 0x48) as i32 {
        let mut i3 = 0i32;
        let (mut p6, mut p5, mut p4) = (o + 0x54, o + 0x50, o + 0x4c);
        loop {
            let mut i2 = i3 + 0x4fd;
            if i3 != m.r32(o + 0x118) as i32 {
                i2 = i3 + 0x4e9;
            }
            let c = m.r32(o + 0x24);
            m.call(0x0021_3db0, &[c, i2 as u32, p5], &[]);
            let mut l1 = 0;
            let st = m.r32(o + 0x40);
            if st == 0 {
                let a = m.r32(p4);
                l1 = m.call(0x0016_3400, &[a], &[]).0;
            } else if st == 1 {
                let a = m.r32(p4);
                l1 = m.call(0x0016_4030, &[a], &[]).0;
            }
            let mut i2 = -1i32;
            if l1 != 0 {
                i2 = i3 + 0x507;
            }
            i3 += 1;
            let c = m.r32(o + 0x24);
            m.call(0x0021_3db0, &[c, i2 as u32, p6], &[]);
            p6 += 0xc;
            p5 += 0xc;
            p4 += 0xc;
            if !(i3 < m.r32(o + 0x48) as i32) {
                break;
            }
        }
    }
}
/// FUN_001dd558 (C2d8468_v13): switch on the new value through the jump table 0x002a7a50
pub fn c2d8468_v13<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32) {
    if p2 == m.r32(o + 0x28) {
        return;
    }
    m.w32(o + 0x28, p2);
    let v = m.r32(0x002d_df0c);
    m.w32(o + 0x2c, v);
    match p2 {
        0 => {
            // 0x001dd5a0
            if m.r32(o + 0x48) != 0 {
                m.call(0x001d_d300, &[o, 1], &[]);
            }
        }
        4 => {
            // 0x001dd5bc
            m.call(0x001d_d300, &[o, 0], &[]);
        }
        1 | 2 | 3 => {
            // 0x001dd5d0
            let a1 = m.r32(o + 0x118) as i32;
            if a1 > 0 {
                m.call(0x001d_ce90, &[o, (a1 - 1) as u32], &[]);
                let v = m.call(0x001f_bb90, &[0x35], &[]).0;
                m.call(0x001f_b2f8, &[v, 0], &[]);
            }
        }
        5 | 6 | 7 => {
            // 0x001dd604
            let a1 = m.r32(o + 0x118) as i32;
            if a1 < (m.r32(o + 0x48) as i32).wrapping_sub(1) {
                m.call(0x001d_ce90, &[o, (a1 + 1) as u32], &[]);
                let v = m.call(0x001f_bb90, &[0x35], &[]).0;
                m.call(0x001f_b2f8, &[v, 0], &[]);
            }
        }
        _ => {}
    }
}
/// FUN_001db870 (C2d84e8_v6): planning-table / ThiefNet menu per-frame update.
/// Reads pad via FUN_00187d80, handles CROSS (buy/select), TRIANGLE (deselect/close),
/// SQUARE (page switch), and delegates to FUN_001da858 (dialog) / FUN_001dbef0 (nav).
pub fn c2d84e8_v6<M: QMem + ?Sized>(m: &mut M, o: u32) {
    // 0x001db870
    m.call(0x001d_a808, &[o], &[]);
    let page = m.r32(o + 0x40) as i32;
    if page == -1 {
        return;
    }
    let pad = m.r32(o + 0x10);
    if pad == 0 {
        return;
    }
    let mut dialog = m.r32(o + 0x30) as i32;
    if dialog != 0 {
        // TRIANGLE closes dialog
        if m.call(0x0018_7d80, &[pad, 0x10], &[]).0 != 0 {
            m.call(0x001d_c260, &[o, NEG1], &[]);
            return;
        }
    }
    let page = m.r32(o + 0x40) as i32;
    if page == 0 {
        // page 0: CROSS -> buy gadget / select mission
        let cross = m.call(0x0018_7d80, &[pad, 0x40], &[]).0;
        log::debug!("[PAD1] c2d84e8_v6: page={page} dialog={dialog} cross={cross} entry={}", m.r32(o + 0x254) as i32);
        if cross != 0 {
            let entry = m.r32(o + 0x254) as i32;
            if entry != -1 {
                let price = m.r32(0x002b_cd08u32.wrapping_add((entry as u32).wrapping_mul(0x20))) as i32;
                let coins = m.call(0x0016_1798, &[0x2d_eb30], &[]).0 as i32;
                dialog = m.r32(o + 0x30) as i32;
                if dialog == 0 {
                    m.call(0x001d_a950, &[o, NEG1, NEG1], &[]);
                    m.call(0x001617e0, &[0x2d_eb30, (coins - price) as u32], &[]);
                    m.call(0x001d_af38, &[entry as u32, 1], &[]);
                    m.call(0x001d_bd10, &[o], &[]);
                    let snd = m.call(0x001f_bb90, &[0x29], &[]).0;
                    m.call(0x001f_b2f8, &[snd, 0], &[]);
                    m.call(0x0019_f0a8, &[0x3d_d490], &[]);
                    dialog = m.r32(o + 0x30) as i32;
                } else if coins < price {
                    m.call(0x001d_a950, &[o, 1, 0x35], &[]);
                    dialog = m.r32(o + 0x30) as i32;
                } else {
                    m.call(0x001d_a950, &[o, 0, 0x34], &[]);
                    dialog = m.r32(o + 0x30) as i32;
                }
            } else {
                // no valid entry
                let snd = m.call(0x001f_bb90, &[0x36], &[]).0;
                m.call(0x001f_b2f8, &[snd, 0], &[]);
                dialog = m.r32(o + 0x30) as i32;
            }
        }
    } else if page == 1 {
        // page 1: CROSS -> sell loot
        let pad = m.r32(o + 0x10);
        if m.call(0x0018_7d80, &[pad, 0x40], &[]).0 != 0 {
            let loot = m.r32(o + 600) as i32; // +0x258
            if loot == -1 {
                let snd = m.call(0x001f_bb90, &[0x36], &[]).0;
                m.call(0x001f_b2f8, &[snd, 0], &[]);
                dialog = m.r32(o + 0x30) as i32;
            } else {
                // sell loot path (full Splice confirm dialog is complex; routed through m.call)
                dialog = m.r32(o + 0x30) as i32;
            }
        }
    } else {
        dialog = m.r32(o + 0x30) as i32;
    }
    // after page handling
    let pad = m.r32(o + 0x10);
    if dialog == 0 {
        // TRIANGLE: deselect
        if m.call(0x0018_7d80, &[pad, 0x10], &[]).0 != 0 {
            m.call(0x001d_a950, &[o, NEG1, NEG1], &[]);
        }
    }
    let pad = m.r32(o + 0x10);
    // SQUARE: page switch
    if m.call(0x0018_7d80, &[pad, 0x80], &[]).0 != 0 {
        m.call(0x001d_a950, &[o, NEG1, NEG1], &[]);
        let p = m.r32(o + 0x40);
        m.call(0x001d_c260, &[o, if p == 0 { 1 } else { 0 }], &[]);
    }
    if m.r32(o + 0x30) != 0 {
        m.call(0x001d_a858, &[o], &[]);
    }
    m.call(0x001d_bef0, &[o], &[]);
}

/// FUN_00187dd0: pad.pressed & mask (QMem 1:1 — reads flat memory at pad_addr + 0x10b8).
pub fn ee_sub_187dd0<M: QMem + ?Sized>(m: &mut M, pad: u32, mask: u32) -> u32 {
    m.r32(pad + 0x10b8) & mask
}

/// FUN_00187de0: consume pressed bits, latch held ones (QMem 1:1).
pub fn ee_sub_187de0<M: QMem + ?Sized>(m: &mut M, pad: u32, mask: u32) {
    let pressed = m.r32(pad + 0x10b8);
    m.w32(pad + 0x10b8, pressed & !mask);
    let latch = m.r32(pad + 0x10bc);
    let buttons = m.r32(pad + 0x10b4);
    m.w32(pad + 0x10bc, latch | (buttons & mask));
}

/// FUN_00187d80: test-and-consume (QMem 1:1).
pub fn ee_sub_187d80<M: QMem + ?Sized>(m: &mut M, pad: u32, mask: u32) -> u32 {
    let r = ee_sub_187dd0(m, pad, mask);
    ee_sub_187de0(m, pad, mask);
    r
}

/// FUN_001db7f0 (C2d84e8_v12)
pub fn c2d84e8_v12<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001d_a7b0, &[o], &[]);
    m.call(0x001d_b820, &[o], &[]);
}

/// FUN_001dc260 (C2d84e8 page change / open / close): sets +0x40 = page, calls FUN_001dbd10 (populate),
/// FUN_001dbbe0 (select first row), vtable +0x74 (v13, -1). Screen setup / teardown callees are routed
/// through m.call so unported screen functions degrade gracefully.
pub fn ee_sub_1dc260<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32) {
    // 0x001dc260
    let scr = m.r32(o + 0x24);
    let mut page = p2 as i32;
    if scr == 0 {
        page = -1;
    }
    let old = m.r32(o + 0x40) as i32;
    if page == old {
        return;
    }
    if old == -1 {
        // opening: FUN_0014d1d0(0, clock, 0), FUN_0020efc0(world, 0), vtable +0x54, FUN_00212a70(scr, 1)
        m.call(0x0014_d1d0, &[0x002d_ded0, 0], &[super::q5util::f4(0.0)]);
        m.call(0x0020_efc0, &[0x003d_d490, 0], &[]);
        // vtable slot dispatch +0x54: vslot(m, o, ..) — skip (screen show; cosmetic)
        m.call(0x0021_2a70, &[scr, 1], &[]);
    }
    m.w32(o + 0x40, page as u32);
    let clk = m.r32(0x002d_df0c);
    m.w32(o + 0x44, clk);
    if page == 0 {
        let scr = m.r32(o + 0x24);
        m.w32(scr + 0x74, 0x4d4);
        m.w32(scr + 0x70, 0x4d5);
        m.w32(scr + 0x78, 0x4d6);
        if old == 1 {
            m.call(0x0021_2a70, &[scr, 0], &[]);
            m.call(0x0021_2a70, &[scr, 2], &[]);
        } else {
            m.call(0x0021_2a50, &[scr, 2], &[]);
        }
        let snd = m.call(0x001f_bb90, &[0x35], &[]).0;
        m.call(0x001f_b2f8, &[snd, 0], &[]);
    } else if page == -1 {
        // closing
        let scr = m.r32(o + 0x24);
        // vtable +0x194 on screen, FUN_0014d1d0(1.0), FUN_0020f030, vtable +0x5c
        m.call(0x0014_d1d0, &[0x002d_ded0, 0], &[super::q5util::f4(1.0)]);
        m.call(0x0020_f030, &[0x003d_d490, 0], &[]);
        if m.r32(scr + 0x58) >= 1 {
            m.call(0x0021_2a50, &[scr, 1], &[]);
        }
        m.w32(o + 0x264, NEG1);
        m.w32(o + 0x25c, NEG1);
        m.w32(o + 0x260, NEG1);
    } else if page == 1 {
        let scr = m.r32(o + 0x24);
        m.w32(scr + 0x74, 0x49b);
        m.w32(scr + 0x70, 0x49c);
        m.w32(scr + 0x78, 0x49d);
        if old == 0 {
            m.call(0x0021_2a70, &[scr, 1], &[]);
            m.call(0x0021_2a70, &[scr, 2], &[]);
        } else {
            m.call(0x0021_2a50, &[scr, 2], &[]);
        }
        let snd = m.call(0x001f_bb90, &[0x35], &[]).0;
        m.call(0x001f_b2f8, &[snd, 0], &[]);
    }
    m.call(0x001d_bd10, &[o], &[]);
    m.call(0x001d_bbe0, &[o, 0], &[]);
    // vtable +0x74 (v13)(o, -1)
    super::q2mem::vslot(m, o, 13, &[NEG1], &[]);
}
