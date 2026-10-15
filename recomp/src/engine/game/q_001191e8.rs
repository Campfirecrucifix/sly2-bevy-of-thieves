//! WP-Q4: run 0x001191e8..0x0011a6b8 - the engine's intrusive lists ({head, tail, link offset}; doubly
//! linked nodes {next, prev} at obj + offset, singly linked {next}; removals fix the safe-iteration frames
//! DAT_002dd2f8 / DAT_002dd2fc {.., +4 current, .., +0xc outer}), the dense LU factor / solve of the
//! constraint solver (n x n f32 row-major), the plane transform and the solver workspace setup.
//! 1:1 against `QMem` (classes::q::q2mem); callees outside the run go through `QMem::call` by address.
//! The Rust object lists / PGS solver (physics::motor) replace these for the ported callers; the ports
//! below keep the EE-memory behaviour for the QMem callers.
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem, V4};
use crate::engine::game::q_001e9090::ee_sub_1ea410;

/// DAT_002dd2f8: singly linked list iteration frames
pub const SL_FRAMES: u32 = 0x002d_d2f8;
/// DAT_002dd2fc: doubly linked list iteration frames
pub const DL_FRAMES: u32 = 0x002d_d2fc;
/// DAT_002bbaf0: shared sentinel kept as is by FUN_001191e8
pub const SENTINEL: u32 = 0x002b_baf0;

/// FUN_001191e8(src, sb, dst, db): copy a 12-byte node table {next, prev, data}: dst {+0 n, +4, +0xc
/// nodes (FUN_001ae1d0 n * 12)}; links rebased into the new table, data moved by db +0xc - sb +0xc
/// (except the DAT_002bbaf0 sentinel)
pub fn ee_sub_1191e8<M: QMem + ?Sized>(m: &mut M, src: u32, sb: u32, dst: u32, db: u32) {
    let n = m.r32(src);
    m.w32(dst, n);
    let v = m.r32(src + 4);
    m.w32(dst + 4, v);
    let s = stack_push(m, 0x10);
    m.w32(s, n.wrapping_mul(0xc));
    m.w32(s + 4, 0x10);
    let t = m.call(0x001a_e1d0, &[s], &[]).0;
    stack_pop(m, 0x10);
    m.w32(dst + 0xc, t);
    let mut i = 0;
    while i < m.r32(dst) as i32 {
        let off = i as u32 * 0xc;
        let (st, dt) = (m.r32(src + 0xc), m.r32(dst + 0xc));
        let (p3, p4) = (st + off, dt + off);
        let d = m.r32(p3 + 8);
        let nd = if d == SENTINEL { SENTINEL } else { d.wrapping_add(m.r32(db + 0xc).wrapping_sub(m.r32(sb + 0xc))) };
        m.w32(p4 + 8, nd);
        for k in [0, 4] {
            let l = m.r32(p3 + k);
            let nl = if l == 0 { 0 } else { dt.wrapping_add(l.wrapping_sub(st)) };
            m.w32(p4 + k, nl);
        }
        i += 1;
    }
}

/// FUN_001193c0(w, out, v): out = v with w
pub fn ee_sub_1193c0<M: QMem + ?Sized>(m: &mut M, w: f32, out: u32, v: V4) {
    m.wv(out, [v[0], v[1], v[2], w]);
}

/// FUN_001193d8(mat, plane): plane (n, d) through the matrix: n' = rows 0..2 * n, point = row 3 + n' d,
/// result (n', point . n')
pub fn ee_sub_1193d8<M: QMem + ?Sized>(m: &mut M, mat: u32, plane: V4) -> V4 {
    let r = m.rm(mat);
    let n: [f32; 3] = std::array::from_fn(|k| r[0][k] * plane[0] + r[1][k] * plane[1] + r[2][k] * plane[2]);
    let d = plane[3];
    let p: [f32; 3] = std::array::from_fn(|k| r[3][k] + n[k] * d);
    let w = p[0] * n[0] + p[1] * n[1] + p[2] * n[2];
    [n[0], n[1], n[2], w]
}

/// FUN_00119488 / 119a68 / 119c48 / 119da0 / 119ef8 / 11a3f8 (identical static pool inits) = FUN_001ea410
pub fn ee_sub_119488<M: QMem + ?Sized>(m: &mut M, init: u32, prio: u32) {
    ee_sub_1ea410(m, init, prio);
}
/// FUN_001195c0 / 119ba0 / 119d80 / 119ed8 / 11a030 / 11a530: (1, 0xffff)
pub fn ee_sub_1195c0<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_1ea410(m, 1, 0xffff);
}

/// FUN_001195e0(n, a, lu): Crout-style LU of the n x n matrix a into lu (row 0 of U copied, column 0 of
/// L divided by u00; pivots under 1e-8 zero their column; last pivot zeroed under 1e-8)
pub fn ee_sub_1195e0<M: QMem + ?Sized>(m: &mut M, n: i32, a: u32, lu: u32) {
    if n == 0 {
        return;
    }
    let at = |i: i32| (i as u32).wrapping_mul(4);
    let n1 = n - 1;
    for k in 0..n.max(0) {
        let v = m.r32(a + at(k));
        m.w32(lu + at(k), v);
    }
    if m.rf(lu).abs() < 1e-8 {
        for k in 0..n.max(0) {
            m.wf(lu + at(k * n), 0.0);
        }
    } else if 1 < n {
        for k in 1..n {
            let v = m.rf(a + at(k * n)) / m.rf(lu);
            m.wf(lu + at(k * n), v);
        }
    }
    let mut i = 1;
    if 1 < n1 {
        loop {
            let i8 = i + 1;
            let mut j = i;
            while j < n {
                let mut s = 0.0f32;
                for k in 0..i {
                    s += m.rf(lu + at(k * n + j)) * m.rf(lu + at(i * n + k));
                }
                let idx = i * n + j;
                let v = m.rf(a + at(idx)) - s;
                m.wf(lu + at(idx), v);
                j += 1;
            }
            let piv = m.rf(lu + at(i * n + i));
            if piv.abs() < 1e-8 {
                for k in 0..(n - i).max(0) {
                    m.wf(lu + at((i + k) * n + i), 0.0);
                }
            } else {
                let mut j = i8;
                while j < n {
                    let mut s = 0.0f32;
                    for k in 0..i {
                        s += m.rf(lu + at(k * n + i)) * m.rf(lu + at(j * n + k));
                    }
                    let idx = j * n + i;
                    let v = (m.rf(a + at(idx)) - s) / m.rf(lu + at(i * n + i));
                    m.wf(lu + at(idx), v);
                    j += 1;
                }
            }
            i = i8;
            if i8 >= n1 {
                break;
            }
        }
    }
    let mut s = 0.0f32;
    for k in 0..n1.max(0) {
        s += m.rf(lu + at((k + 1) * n - 1)) * m.rf(lu + at(n1 * n + k));
    }
    let last = n1 * n + n - 1;
    let f = m.rf(a + at(last));
    m.wf(lu + at(last), f - s);
    if (f - s).abs() < 1e-8 {
        m.wf(lu + at(last), 0.0);
    }
}

/// FUN_001198f8(n, lu, b, x): forward solve x[i] = (b[i] - sum_k<i lu[k n + i] x[k]) / lu[i n + i] (0 for a
/// zero pivot), then back x[i] -= sum_k>i lu[k n + i] x[k] for non-zero pivots
pub fn ee_sub_1198f8<M: QMem + ?Sized>(m: &mut M, n: i32, lu: u32, b: u32, x: u32) {
    let at = |i: i32| (i as u32).wrapping_mul(4);
    for i in 0..n.max(0) {
        if m.rf(lu + at(i * n + i)) == 0.0 {
            m.wf(x + at(i), 0.0);
        } else {
            let mut s = 0.0f32;
            for k in 0..i {
                s += m.rf(lu + at(k * n + i)) * m.rf(x + at(k));
            }
            let v = (m.rf(b + at(i)) - s) / m.rf(lu + at(i * n + i));
            m.wf(x + at(i), v);
        }
    }
    let mut i = n - 2;
    while -1 < i {
        if m.rf(lu + at(i * n + i)) != 0.0 {
            let mut s = 0.0f32;
            for k in i + 1..n {
                s += m.rf(lu + at(k * n + i)) * m.rf(x + at(k));
            }
            let v = m.rf(x + at(i)) - s;
            m.wf(x + at(i), v);
        }
        i -= 1;
    }
}

/// FUN_0011a050(list, off): link offset +8 = off
pub fn ee_sub_11a050<M: QMem + ?Sized>(m: &mut M, l: u32, off: u32) {
    m.w32(l + 8, off);
}

/// FUN_0011a060 / 11a200 / 11a210(list): head = tail = 0
pub fn ee_sub_11a060<M: QMem + ?Sized>(m: &mut M, l: u32) {
    m.w32(l, 0);
    m.w32(l + 4, 0);
}
/// FUN_0011a200: = FUN_0011a060
pub fn ee_sub_11a200<M: QMem + ?Sized>(m: &mut M, l: u32) {
    ee_sub_11a060(m, l);
}
/// FUN_0011a210: = FUN_0011a060
pub fn ee_sub_11a210<M: QMem + ?Sized>(m: &mut M, l: u32) {
    ee_sub_11a060(m, l);
}

/// FUN_0011a098(list, obj): singly linked push front
pub fn ee_sub_11a098<M: QMem + ?Sized>(m: &mut M, l: u32, o: u32) {
    let h = m.r32(l);
    if h == 0 {
        m.w32(l, o);
        m.w32(l + 4, o);
        return;
    }
    let off = m.r32(l + 8);
    m.w32(o.wrapping_add(off), h);
    m.w32(l, o);
}

/// FUN_0011a0c0(list, after, obj): singly linked insert after `after` (0: front, the tail: FUN_0011a070
/// append)
pub fn ee_sub_11a0c0<M: QMem + ?Sized>(m: &mut M, l: u32, after: u32, o: u32) {
    if after == 0 {
        ee_sub_11a098(m, l, o);
    } else if after == m.r32(l + 4) {
        ee_sub_11a070(m, l, o);
    } else {
        let off = m.r32(l + 8);
        let p = after.wrapping_add(off);
        let nx = m.r32(p);
        m.w32(o.wrapping_add(off), nx);
        m.w32(p, o);
    }
}

/// FUN_0011a118(list, obj): singly linked remove (walks to the predecessor link; iteration frames at
/// DAT_002dd2f8 pointing at the node move to it; tail fixed)
pub fn ee_sub_11a118<M: QMem + ?Sized>(m: &mut M, l: u32, o: u32) {
    let off = m.r32(l + 8);
    let node = o.wrapping_add(off);
    let mut pred = l;
    if o != m.r32(l) {
        let mut it = m.r32(l);
        loop {
            pred = it.wrapping_add(m.r32(l + 8));
            if o == m.r32(pred) {
                break;
            }
            it = m.r32(pred);
        }
    }
    let mut f = m.r32(SL_FRAMES);
    while f != 0 {
        if m.r32(f + 4) == node {
            m.w32(f + 4, pred);
        }
        f = m.r32(f + 0xc);
    }
    let nx = m.r32(node);
    m.w32(pred, nx);
    if m.r32(l) == 0 {
        m.w32(l + 4, 0);
    } else if nx == 0 {
        let off = m.r32(l + 8);
        m.w32(l + 4, pred.wrapping_sub(off));
    }
    m.w32(node, 0);
}

/// FUN_0011a058(ptr): *ptr = 0  (clear one word)
pub fn ee_sub_11a058<M: QMem + ?Sized>(m: &mut M, p: u32) {
    m.w32(p, 0);
}

/// FUN_0011a070(list, obj): singly linked append (tail link)
pub fn ee_sub_11a070<M: QMem + ?Sized>(m: &mut M, l: u32, o: u32) {
    let t = m.r32(l + 4);
    if t != 0 {
        let off = m.r32(l + 8);
        m.w32(t.wrapping_add(off), o);
        m.w32(l + 4, o);
    } else {
        m.w32(l, o);
        m.w32(l + 4, o);
    }
}

/// FUN_0011a1f8(list, off): set the link-offset field (+8)
pub fn ee_sub_11a1f8<M: QMem + ?Sized>(m: &mut M, l: u32, off: u32) {
    m.w32(l + 8, off);
}

/// FUN_0011a1b8(list, obj): obj linked (next set or it is the tail)
pub fn ee_sub_11a1b8<M: QMem + ?Sized>(m: &mut M, l: u32, o: u32) -> bool {
    let off = m.r32(l + 8);
    m.r32(o.wrapping_add(off)) != 0 || m.r32(l + 4) == o
}

/// FUN_0011a1e8(list): empty
pub fn ee_sub_11a1e8<M: QMem + ?Sized>(m: &mut M, l: u32) -> bool {
    m.r32(l) == 0
}

/// FUN_0011a380(list, obj): membership test (identical to FUN_0011a1b8 at a different address)
pub fn ee_sub_11a380<M: QMem + ?Sized>(m: &mut M, l: u32, o: u32) -> u32 {
    let off = m.r32(l + 8);
    if m.r32(o.wrapping_add(off)) != 0 || m.r32(l + 4) == o { 1 } else { 0 }
}

/// FUN_0011a3b0(list): list empty (head == 0)
pub fn ee_sub_11a3b0<M: QMem + ?Sized>(m: &mut M, l: u32) -> bool {
    m.r32(l) == 0
}

/// FUN_0011a220(list, obj): doubly linked append
pub fn ee_sub_11a220<M: QMem + ?Sized>(m: &mut M, l: u32, o: u32) {
    let off = m.r32(l + 8);
    let t = m.r32(l + 4);
    if t == 0 {
        m.w32(l, o);
        m.w32(l + 4, o);
        return;
    }
    m.w32(o.wrapping_add(off) + 4, t);
    m.w32(t.wrapping_add(off), o);
    m.w32(l + 4, o);
}

/// FUN_0011a250(list, obj): doubly linked push front
pub fn ee_sub_11a250<M: QMem + ?Sized>(m: &mut M, l: u32, o: u32) {
    let off = m.r32(l + 8);
    let h = m.r32(l);
    if h == 0 {
        m.w32(l, o);
        m.w32(l + 4, o);
        return;
    }
    m.w32(o.wrapping_add(off), h);
    m.w32(h.wrapping_add(off) + 4, o);
    m.w32(l, o);
}

/// FUN_0011a280(list, before, obj): doubly linked insert before `before` (0: append, the head: push front)
pub fn ee_sub_11a280<M: QMem + ?Sized>(m: &mut M, l: u32, before: u32, o: u32) {
    if before == 0 {
        ee_sub_11a220(m, l, o);
    } else if before == m.r32(l) {
        ee_sub_11a250(m, l, o);
    } else {
        let off = m.r32(l + 8);
        let pv = m.r32(before.wrapping_add(off) + 4);
        m.w32(o.wrapping_add(off), before);
        m.w32(o.wrapping_add(off) + 4, pv);
        m.w32(before.wrapping_add(off) + 4, o);
        m.w32(pv.wrapping_add(off), o);
    }
}

/// FUN_0011a2e8(list, obj): doubly linked remove; iteration frames (DAT_002dd2fc) on the node step back
/// to its predecessor link (the list itself for the head)
pub fn ee_sub_11a2e8<M: QMem + ?Sized>(m: &mut M, l: u32, o: u32) {
    let off = m.r32(l + 8);
    let node = o.wrapping_add(off);
    let mut f = m.r32(DL_FRAMES);
    while f != 0 {
        if m.r32(f + 4) == node {
            let pv = m.r32(node + 4);
            let v = if pv == 0 { l } else { pv.wrapping_add(m.r32(l + 8)) };
            m.w32(f + 4, v);
        }
        f = m.r32(f + 0xc);
    }
    let (nx, pv) = (m.r32(node), m.r32(node + 4));
    if pv == 0 {
        m.w32(l, nx);
    } else {
        let off = m.r32(l + 8);
        m.w32(pv.wrapping_add(off), nx);
    }
    let nx = m.r32(node);
    if nx == 0 {
        let pv = m.r32(node + 4);
        m.w32(l + 4, pv);
    } else {
        let off = m.r32(l + 8);
        let pv = m.r32(node + 4);
        m.w32(nx.wrapping_add(off) + 4, pv);
    }
    m.w32(node + 4, 0);
    m.w32(node, 0);
}

/// FUN_0011a550(ws, n): solver workspace: +0 n, +4 = +0x3c = 0, +8 = n 0x28-byte rows (FUN_001ae378),
/// four lists +0xc / +0x18 / +0x24 / +0x30 (memset, FUN_0011a1f8(list, 0x20)), +0x40 / +0x44 n x n floats,
/// +0x48 / +0x4c n floats (FUN_001ae318)
pub fn ee_sub_11a550<M: QMem + ?Sized>(m: &mut M, w: u32, n: u32) -> u32 {
    let s = stack_push(m, 0x10);
    m.w32(s, n.wrapping_mul(0x28));
    m.w32(w, n);
    m.w32(w + 0x3c, 0);
    m.w32(w + 4, 0);
    m.w32(s + 4, 0x10);
    let r = m.call(0x001a_e378, &[s], &[]).0;
    m.w32(w + 8, r);
    for off in [0xc, 0x18, 0x24, 0x30] {
        m.call(0x0023_33b4, &[w + off, 0, 0xc], &[]);
    }
    for off in [0xc, 0x18, 0x24, 0x30] {
        m.call(0x0011_a1f8, &[w + off, 0x20], &[]);
    }
    for (off, size) in [(0x40, n.wrapping_mul(n) * 4), (0x44, n.wrapping_mul(n) * 4), (0x48, n << 2), (0x4c, n << 2)] {
        m.w32(s, size);
        m.w32(s + 4, 0x10);
        let r = m.call(0x001a_e318, &[s], &[]).0;
        m.w32(w + off, r);
    }
    stack_pop(m, 0x10);
    w
}

/// FUN_0011a6b8(ws): reset the workspace: +0x3c = 0, the four lists emptied, every row (+8, n = +4) gets
/// +0xc = +0x10 = 0, its list +0x20 emptied and is appended to the free list +0x30
pub fn ee_sub_11a6b8<M: QMem + ?Sized>(m: &mut M, w: u32) {
    m.w32(w + 0x3c, 0);
    for off in [0xc, 0x18, 0x24, 0x30] {
        ee_sub_11a200(m, w + off);
    }
    let mut i = 0;
    while i < m.r32(w + 4) as i32 {
        let r = m.r32(w + 8) + i as u32 * 0x28;
        m.w32(r + 0x10, 0);
        m.w32(r + 0xc, 0);
        ee_sub_11a210(m, r + 0x20);
        ee_sub_11a220(m, w + 0x30, r);
        i += 1;
    }
}
