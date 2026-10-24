//! WIRE-S1: 1:1 QMem ports of WIRE-ROOTS live-frontier roots (round 3).
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem};

/// FUN_001ffa70(out, _, tbl, i): lazily create the handle for entry i of tbl, then build a 0x24 result into out.
pub fn ee_sub_1ffa70<M: QMem + ?Sized>(m: &mut M, a0: u32, _a1: u32, a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x30);
    let s1 = a3 << 3;
    m.w32(sp, 0xffff_ffff);
    let e = s1.wrapping_add(m.r32(a2 + 8));
    if m.r32(e) != 0 && m.r32(e + 4) == 0 {
        let v = m.call(0x0025_6dd0, &[], &[]).0;
        m.call(0x0025_7058, &[sp + 0x10, v], &[]);
        let h = m.r32(sp + 0x10);
        m.call(0x0025_9a88, &[h], &[]);
        let h = m.r32(sp + 0x10);
        m.w32(h + 0x24, 1);
        let s0 = sp + 0x20;
        let b = m.r32(a2 + 8);
        let a1 = m.r32(s1.wrapping_add(b));
        let h = m.r32(sp + 0x10);
        m.call(0x0012_7838, &[s0, a1, h], &[]);
        m.call(0x0012_9658, &[sp, s0], &[]);
        m.call(0x0012_9ec8, &[s0], &[]);
        let h = m.r32(sp + 0x10);
        let c = m.r32(h).wrapping_add(1);
        m.w32(h, c);
        let b = m.r32(a2 + 8);
        let h = m.r32(sp + 0x10);
        m.w32(s1.wrapping_add(b) + 4, h);
        let h = m.r32(sp + 0x10);
        if h != 0 {
            let c = m.r32(h).wrapping_sub(1);
            m.w32(h, c);
            if c == 0 {
                m.call(0x0025_7078, &[h, 3], &[]);
            }
        }
    }
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    m.call(0x0012_91a8, &[a0, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x30);
    a0
}

/// FUN_00163838(q, a, b, c, d): if q not locked (+0x50) and count (+0xa8) < 32, append {a, c, FUN_00162050(b), d}.
pub fn ee_sub_163838<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, t0: u32) {
    if m.r32(a0 + 0x50) != 0 {
        return;
    }
    let n = m.r32(a0 + 0xa8) as i32;
    if n >= 0x20 {
        return;
    }
    let s0 = m.r32(a0 + 0xac).wrapping_add((n as u32) << 4);
    m.w32(a0 + 0xa8, (n + 1) as u32);
    m.w32(s0, a1);
    m.w32(s0 + 4, a3);
    let v = m.call(0x0016_2050, &[a2], &[]).0;
    m.w32(s0 + 0xc, t0);
    m.w32(s0 + 8, v);
}

const MARK_G: u32 = 0x002d_d598;

/// Set the mark bit of obj+4 to the global epoch bit; true if it changed.
fn mark<M: QMem + ?Sized>(m: &mut M, obj: u32) -> bool {
    let g = m.r32(MARK_G);
    let v = m.r32(obj + 4);
    if v >> 31 == g {
        return false;
    }
    m.w32(obj + 4, (v & 0x7fff_ffff) | (g << 31));
    true
}

fn push<M: QMem + ?Sized>(m: &mut M, p: u32, cnt: u32, arr: u32, x: u32) {
    let n = m.r32(p + cnt);
    let a = m.r32(p + arr);
    m.w32((n << 2).wrapping_add(a), x);
    m.w32(p + cnt, n.wrapping_add(1));
}

fn pop<M: QMem + ?Sized>(m: &mut M, p: u32, cnt: u32, arr: u32) -> u32 {
    let n = m.r32(p + cnt) as i32;
    if n > 0 {
        let n = (n - 1) as u32;
        let a = m.r32(p + arr);
        m.w32(p + cnt, n);
        m.r32((n << 2).wrapping_add(a))
    } else {
        0
    }
}

/// FUN_001281b0(p): mark-propagation worklist drain over three stacks (+4/+8, +0x14/+0x18, +0xc/+0x10).
pub fn ee_sub_1281b0<M: QMem + ?Sized>(m: &mut M, p: u32) {
    let mut t3: u32 = 0;
    loop {
        let t2 = pop(m, p, 4, 8);
        if t2 != 0 {
            let mut a2 = m.r32(t2 + 0x1c);
            while a2 != 0 {
                let g = m.r32(MARK_G);
                let a1 = a2 + 0xc;
                let v = m.r32(a2 + 4);
                let k = m.r32(a2 + 0xc) as i32;
                m.w32(a2 + 4, (v & 0x7fff_ffff) | (g << 31));
                if k == 0x20 {
                    t3 = m.r32(a1 + 4);
                    if mark(m, t3) {
                        push(m, p, 0x14, 0x18, t3);
                    }
                } else if k < 0x21 {
                    if k == 0x1f {
                        let x = m.r32(a1 + 4);
                        if mark(m, x) {
                            push(m, p, 0xc, 0x10, x);
                        }
                    }
                } else if k == 0x22 {
                    // asm uses stale $t3 here
                    let g = m.r32(MARK_G);
                    let v = m.r32(t3 + 4);
                    m.w32(t3 + 4, (v & 0x7fff_ffff) | (g << 31));
                }
                a2 = m.r32(a2 + 0x14);
            }
            let mut i: i32 = 0;
            if (m.r32(t2 + 8) as i32) > 0 {
                loop {
                    let a1 = m.r32(t2 + 0xc + ((i as u32) << 2));
                    if mark(m, a1) {
                        push(m, p, 4, 8, a1);
                    }
                    i += 1;
                    if i >= m.r32(t2 + 8) as i32 {
                        break;
                    }
                }
            }
            continue;
        }
        t3 = pop(m, p, 0x14, 0x18);
        if t3 != 0 {
            let mut a1 = m.r32(t3 + 0x14);
            while a1 != 0 {
                mark(m, a1);
                a1 = m.r32(a1 + 0x10);
            }
            let a1 = m.r32(t3 + 0x18);
            if mark(m, a1) {
                push(m, p, 0xc, 0x10, a1);
            }
            let t2 = m.r32(t3 + 8);
            if mark(m, t2) {
                push(m, p, 4, 8, t2);
            }
            continue;
        }
        let a1 = pop(m, p, 0xc, 0x10);
        if a1 == 0 {
            return;
        }
        let k = m.r32(a1 + 8) as i32;
        if k == 0x20 {
            t3 = m.r32(a1 + 0xc);
            if mark(m, t3) {
                push(m, p, 0x14, 0x18, t3);
            }
        } else if k < 0x21 {
            if k == 0x1f {
                let a2 = m.r32(a1 + 0xc);
                if mark(m, a2) {
                    push(m, p, 0xc, 0x10, a2);
                }
            }
        } else if k == 0x22 {
            let a0 = m.r32(a1 + 0xc);
            let g = m.r32(MARK_G);
            let v = m.r32(a0 + 4);
            m.w32(a0 + 4, (v & 0x7fff_ffff) | (g << 31));
        }
        let a1 = m.r32(a1 + 0x10);
        if a1 != 0 && mark(m, a1) {
            push(m, p, 0xc, 0x10, a1);
        }
    }
}

/// FUN_002190f8(o, i, x): for each byte of entry i's type list equal to 7, call FUN_00218748(o+0x10, x).
pub fn ee_sub_2190f8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    let s1 = m.r32(a0 + 4).wrapping_add(a1 << 5);
    if m.r8(s1 + 7) == 0 {
        return;
    }
    let mut s0: i32 = 0;
    loop {
        let b = m.r32(s1 + 8);
        if m.r8(b.wrapping_add(s0 as u32)) == 7 {
            let o = m.r32(a0 + 0x10);
            m.call(0x0021_8748, &[o, a2], &[]);
        }
        s0 += 1;
        if s0 >= m.r8(s1 + 7) as i32 {
            break;
        }
    }
}
