//! WP-Q12 part B: file packages run_00132298 (stream reader + world object lists), run_001444e0.
//! 1:1 from ghidra_functions(_v2).body on `QMem` (conventions as q12_a).
#![allow(unused_variables, unused_mut, dead_code)]
use crate::engine::classes::q::q11_a::dist3;
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};
use crate::engine::classes::q::q6::rof;
use crate::engine::classes::q::q6_runs::ee_sub_1ba9f8 as pools_init;
use crate::engine::classes::q::q7util::{f, CLOCK};

const NEG1: u32 = u32::MAX;

// ---------------------------------------------------------------- run_00132298
/// FUN_00132298(this): collect up to 16 distinct "other endpoints" (word +8 of the link, or +0x48 when +8 is
/// this) of every link chained (+0xac) from the items of the list at (*(this+0xec) + 100) (nodes {item, next}),
/// then for each: this+0x17c = endpoint; slot 2 (+0x1c)(5, 0, 0, 0, 0)
pub fn ee_sub_132298<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let mut a: Vec<u32> = vec![];
    let h = m.r32(o + 0xec);
    let mut node = m.r32(h + 100);
    'done: {
        if node == 0 {
            break 'done;
        }
        let mut item = m.r32(node);
        loop {
            if item != 0 {
                loop {
                    let l8 = m.r32(item + 8);
                    let ep = m.r32(item + if l8 == o { 0x40 } else { 0 } + 8);
                    // already collected?
                    if !a.contains(&ep) {
                        a.push(ep);
                        if a.len() > 0xf {
                            break 'done;
                        }
                    }
                    item = m.r32(item + 0xac);
                    if item == 0 {
                        break;
                    }
                }
            }
            node = m.r32(node + 4);
            if node == 0 {
                break 'done;
            }
            item = m.r32(node);
        }
    }
    for &e in &a {
        m.w32(o + 0x17c, e);
        vslot(m, o, 2, &[5, 0, 0, 0, 0], &[]);
    }
}
/// FUN_00132570(this, p2, p3): stream buffer init: +0x14 = p2, +0x18 = (p3 + 0x3f) & ~0x3f, +4 = -1, rest 0
pub fn ee_sub_132570<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32, p3: u32) -> u32 {
    m.w32(o + 0x14, p2);
    m.w32(o + 0x18, p3.wrapping_add(0x3f) & 0xffff_ffc0);
    m.w32(o + 4, NEG1);
    for off in [8, 0x10, 0x20, 0x2c, 0x28, 0xc, 0x24, 0x1c] {
        m.w32(o + off, 0);
    }
    o
}
/// FUN_001325b8(this, flags): FUN_00132688(this); flags & 1 -> FUN_001ae520(this)
pub fn ee_sub_1325b8<M: QMem + ?Sized>(m: &mut M, o: u32, flags: u64) {
    ee_sub_132688(m, o);
    if flags & 1 != 0 {
        m.call(0x001a_e520, &[o], &[]);
    }
}
/// FUN_00132600(this, req): only for req[2] == 2: FUN_00132640(this, req[0], req[1]), else 0
pub fn ee_sub_132600<M: QMem + ?Sized>(m: &mut M, o: u32, req: u32) -> u32 {
    if m.r32(req + 8) == 2 {
        let (a, b) = (m.r32(req), m.r32(req + 4));
        return ee_sub_132640(m, o, a, b);
    }
    0
}
/// FUN_00132640(this, a, b): [2] = a, [10] = b, [0] = 1, [8] = [6], [11] = b, [7] = [9] = 0; FUN_00132710
pub fn ee_sub_132640<M: QMem + ?Sized>(m: &mut M, o: u32, a: u32, b: u32) -> u32 {
    m.w32(o + 8, a);
    m.w32(o + 40, b);
    m.w32(o, 1);
    let v = m.r32(o + 24);
    m.w32(o + 32, v);
    m.w32(o + 44, b);
    m.w32(o + 28, 0);
    m.w32(o + 36, 0);
    ee_sub_132710(m, o);
    1
}
/// FUN_00132688(this): kind 0: close handle [1] (FUN_00238138) when >= 0, [1] = -1; kind 1: when busy ([3] != 0)
/// FUN_00241e88(0) + FUN_001328a8, [2] = 0; [0] = -1
pub fn ee_sub_132688<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let k = m.r32(o);
    if k == 0 {
        if (m.r32(o + 4) as i32) >= 0 {
            m.call(0x0023_8138, &[], &[]);
        }
        m.w32(o + 4, NEG1);
    } else if k == 1 {
        if !ee_sub_1327d0(m, o) {
            m.w32(o + 8, 0);
        } else {
            m.call(0x0024_1e88, &[0], &[]);
            ee_sub_1328a8(m, o);
            m.w32(o + 8, 0);
        }
    }
    m.w32(o, NEG1);
}
/// FUN_00132710(this): when not busy and [10] != 0: chunk = min([10], [5] / 2); dest [4] = [6] (+ [5] / 2 when
/// [8] == [6]); [3] = chunk; kind 0: FUN_002382b8([1], dest, chunk); kind 1: FUN_0014bc78([2], (chunk + 0x7ff) >> 11, dest, 1)
pub fn ee_sub_132710<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let half = (m.r32(o + 20) as i32) / 2;
    if !ee_sub_1327d0(m, o) {
        let n = m.r32(o + 40) as i32;
        if n != 0 {
            let mut dst = m.r32(o + 24) as i32;
            let mut ch = half;
            if n < half {
                ch = n;
            }
            if m.r32(o + 32) as i32 == dst {
                dst = dst.wrapping_add(half);
            }
            m.w32(o + 12, ch as u32);
            m.w32(o + 16, dst as u32);
            let k = m.r32(o);
            if k == 0 {
                let h = m.r32(o + 4);
                m.call(0x0023_82b8, &[h, dst as u32, ch as u32], &[]);
            } else if k == 1 {
                let h = m.r32(o + 8);
                m.call(0x0014_bc78, &[h, (ch as u32).wrapping_add(0x7ff) >> 11, dst as u32, 1], &[]);
            }
        }
    }
}
/// FUN_001327d0: [3] != 0
pub fn ee_sub_1327d0<M: QMem + ?Sized>(m: &mut M, o: u32) -> bool {
    m.r32(o + 0xc) != 0
}
/// FUN_001327e0(this): true when the pending read ([7] == [9], busy) finished (kind 0: FUN_002387e8([1], 1, &st)
/// with st == 0; kind 1: FUN_00241e88(1) == 0, error FUN_00242ae0()); an error clears [3] / [4] and gives false
pub fn ee_sub_1327e0<M: QMem + ?Sized>(m: &mut M, o: u32) -> bool {
    let mut b = false;
    let mut err = 0;
    if m.r32(o + 28) == m.r32(o + 36) && ee_sub_1327d0(m, o) {
        let k = m.r32(o);
        if k == 0 {
            let st = stack_push(m, 0x10);
            let h = m.r32(o + 4);
            err = m.call(0x0023_87e8, &[h, 1, st], &[]).0;
            b = m.r32(st) == 0;
            stack_pop(m, 0x10);
        } else if k == 1 {
            let r = m.call(0x0024_1e88, &[1], &[]).0;
            b = r == 0;
            if b {
                err = m.call(0x0024_2ae0, &[], &[]).0;
            }
        }
    }
    if err != 0 {
        m.w32(o + 16, 0);
        m.w32(o + 12, 0);
        b = false;
    }
    b
}
/// FUN_001328a8(this): when FUN_001327e0: kind 1 advances [2] by [3] >> 11 (sector, rounding toward zero);
/// [7] = [3]; [10] -= [3]; [8] = [4]; [9] = [3] = [4] = 0
pub fn ee_sub_1328a8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if ee_sub_1327e0(m, o) {
        let t;
        if m.r32(o) == 1 {
            let n = m.r32(o + 12) as i32;
            let adj = if -1 < n { n } else { n.wrapping_add(0x7ff) };
            let v = m.r32(o + 8).wrapping_add((adj >> 11) as u32);
            m.w32(o + 8, v);
            t = m.r32(o + 12);
        } else {
            t = m.r32(o + 12);
        }
        m.w32(o + 28, t);
        let v = m.r32(o + 40).wrapping_sub(t);
        m.w32(o + 40, v);
        let v = m.r32(o + 16);
        m.w32(o + 32, v);
        m.w32(o + 36, 0);
        m.w32(o + 12, 0);
        m.w32(o + 16, 0);
    }
}
/// FUN_00132930(this): kind 0: 132710 then 1328a8; kind 1: 1328a8 then 132710
pub fn ee_sub_132930<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let k = m.r32(o);
    if k == 0 {
        ee_sub_132710(m, o);
        ee_sub_1328a8(m, o);
    } else if k == 1 {
        ee_sub_1328a8(m, o);
        ee_sub_132710(m, o);
    }
}
/// FUN_00132998: +0x24 += n
pub fn ee_sub_132998<M: QMem + ?Sized>(m: &mut M, o: u32, n: u32) {
    let v = m.r32(o + 0x24).wrapping_add(n);
    m.w32(o + 0x24, v);
}
/// FUN_00132b78: lazily +4 = FUN_002580a8(&+4, FUN_00257d80()), FUN_00258178(+4)
pub fn ee_sub_132b78<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.r32(o + 4) == 0 {
        let u = m.call(0x0025_7d80, &[], &[]).0;
        m.call(0x0025_80a8, &[o + 4, u], &[]);
        let a = m.r32(o + 4);
        m.call(0x0025_8178, &[a], &[]);
    }
}
/// FUN_00132ea0: slot 5 (+0x34) result r; *(r + 4)
pub fn ee_sub_132ea0<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let r = vslot(m, o, 5, &[], &[]).0;
    m.r32(r + 4)
}
/// FUN_00132ed8(out, a, b, c): build a symbol / list record in `out` (refcounted temp loc: FUN_00256c38 /
/// 256c58 / 129970 / 1295d8 / 129a10; refcount word at *loc dropped, freed through 256d40 / Release / 2550f8 /
/// 11cd98(0x4520a8)); `c` is an optional {flag, value} pair
pub fn ee_sub_132ed8<M: QMem + ?Sized>(m: &mut M, out: u32, a: u32, b: u32, c: u32) -> u32 {
    let loc = stack_push(m, 0x10);
    let u = m.call(0x0025_6b48, &[], &[]).0;
    m.call(0x0025_6c38, &[loc, u], &[]);
    let l0 = m.r32(loc);
    let u = m.call(0x0025_6b48, &[], &[]).0;
    m.call(0x0012_9970, &[l0 + 8, u], &[]);
    let l0 = m.r32(loc);
    let mut p = m.r32(l0 + 0xc);
    m.call(0x0012_95d8, &[p + 8, b], &[]);
    let u = m.call(0x0025_6b48, &[], &[]).0;
    m.call(0x0025_6c58, &[p + 0x10, u], &[]);
    p = m.r32(p + 0x10);
    m.call(0x0012_9a10, &[p + 8, a], &[]);
    if c != 0 && m.r32(c) != 0 {
        let v = m.r32(c + 4);
        m.call(0x0025_6c58, &[p + 0x10, v], &[]);
    }
    let l0 = m.r32(loc);
    m.call(0x0025_6c38, &[out, l0], &[]);
    let l0 = m.r32(loc);
    if l0 != 0 {
        let n = m.r32(l0);
        m.w32(l0, n.wrapping_sub(1));
        if n.wrapping_sub(1) == 0 {
            m.call(0x0025_6d40, &[l0 + 0x10, 2], &[]);
            m.call(0x0012_9ec8, &[l0 + 8], &[]);
            m.call(0x0025_50f8, &[l0], &[]);
            m.call(0x0011_cd98, &[0x0045_20a8, l0], &[]);
        }
    }
    stack_pop(m, 0x10);
    out
}
/// FUN_00133000(a3): Eval(tmp) (FUN_00127838); a3 != 0 -> FUN_00129658(a3, tmp); Release(tmp) (FUN_00129ec8)
pub fn ee_sub_133000<M: QMem + ?Sized>(m: &mut M, a3: u32) {
    let t = stack_push(m, 0x30);
    m.call(0x0012_7838, &[t], &[]);
    if a3 != 0 {
        m.call(0x0012_9658, &[a3, t], &[]);
    }
    m.call(0x0012_9ec8, &[t], &[]);
    stack_pop(m, 0x30);
}
/// FUN_001333c0(this, k): refcounted key list at +8 (nodes {key, count, next}): +1 on a match else push a new
/// node from the pool 0x3e1144 (ActPool_11ccc0) with count 1
pub fn ee_sub_1333c0<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32) {
    let mut p = m.r32(o + 8);
    loop {
        if p == 0 {
            let n = m.call(0x0011_ccc0, &[0x003e_1144], &[]).0;
            m.w32(n, k);
            m.w32(n + 4, 1);
            let h = m.r32(o + 8);
            m.w32(n + 8, h);
            m.w32(o + 8, n);
            return;
        }
        if m.r32(p) == k {
            let c = m.r32(p + 4).wrapping_add(1);
            m.w32(p + 4, c);
            return;
        }
        p = m.r32(p + 8);
    }
}
/// FUN_001334a8: free every +8-chained node into the pool 0x3e1144 (FUN_0011cd98), +8 = 0
pub fn ee_sub_1334a8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let mut n = m.r32(o + 8);
    while n != 0 {
        let nx = m.r32(n + 8);
        m.call(0x0011_cd98, &[0x003e_1144, n], &[]);
        n = nx;
    }
    m.w32(o + 8, 0);
}
/// FUN_001337d0(w, b): remove object b from the world lists: its link records (+100 chain {obj, next, ?, link}),
/// the lists at w+0x3d10 / 0x3d1c / 0x3d28 / 0x3d34 and the 3 sorted 12-byte-entry axis arrays at *(w+0x3cd4+4i)
/// (begin entry [0] == 0, end entry [0] != 0, both [1] == b), whose count is w+0x3cd0
pub fn ee_sub_1337d0<M: QMem + ?Sized>(m: &mut M, w: u32, b: u32) {
    let mut p5 = m.r32(b + 100);
    if p5 != 0 {
        loop {
            let owner = m.r32(b + 0x60);
            let lk = m.r32(p5 + 12);
            vslot(m, owner, 53, &[lk], &[]);
            let lk = m.r32(p5 + 12);
            let bo = m.r32(b + 0x60);
            vslot(m, lk, 53, &[bo], &[]);
            let lk = m.r32(p5 + 12);
            let e = m.r32(lk + 0xec);
            let u3 = ee_sub_133cb8(m, w, e, b);
            if m.r32(p5) != 0 {
                let lk = m.r32(p5 + 12);
                let l = vslot(m, lk, 61, &[], &[]).0;
                if l == 0 {
                    let lk = m.r32(p5 + 12);
                    m.call(0x001f_5078, &[lk], &[]);
                }
                let i2 = m.r32(p5);
                m.call(0x001a_7e10, &[w, i2], &[]);
            }
            let nx = m.r32(p5 + 4);
            m.call(0x0011_cd98, &[w + 0x3cf8, p5], &[]);
            m.call(0x0011_cd98, &[w + 0x3cf8, u3], &[]);
            if nx == 0 {
                break;
            }
            p5 = nx;
        }
    }
    let v = m.r32(b + 0x6c);
    m.call(0x001a_7e10, &[w, v], &[]);
    m.call(0x0011_a2e8, &[w + 0x3d10, b], &[]);
    if m.call(0x0011_a380, &[w + 0x3d1c, b], &[]).0 != 0 {
        m.call(0x0011_a2e8, &[w + 0x3d1c, b], &[]);
    }
    if m.r32(b + 0x70) & 1 != 0 {
        m.call(0x0011_a2e8, &[w + 0x3d28, b], &[]);
    }
    if m.r32(b + 0x70) & 2 != 0 {
        m.call(0x0011_a2e8, &[w + 0x3d34, b], &[]);
    }
    for i in 0..3u32 {
        let base = m.r32(w + 0x3cd4 + 4 * i);
        let mut p1 = base;
        let mut begin = 0;
        let end;
        loop {
            let mut p6 = p1;
            while m.r32(p6 + 4) != b {
                p6 += 12;
            }
            if m.r32(p6) != 0 {
                end = p6;
                break;
            }
            p1 = p6 + 12;
            begin = p6;
        }
        let n1 = (((end.wrapping_sub(begin)) as i32).wrapping_mul(-0x5555_5555) >> 2).wrapping_mul(0xc).wrapping_sub(0xc);
        m.call(0x0023_32b8, &[begin, begin + 12, n1 as u32], &[]);
        let cnt = m.r32(w + 0x3cd0) as i32;
        let idx = ((end.wrapping_sub(base)) as i32).wrapping_mul(-0x5555_5555) >> 2;
        let n2 = cnt.wrapping_sub(idx).wrapping_mul(0xc).wrapping_sub(0xc);
        m.call(0x0023_32b8, &[end - 12, end + 12, n2 as u32], &[]);
    }
    let c = m.r32(w + 0x3cd0).wrapping_sub(2);
    m.w32(w + 0x3cd0, c);
    m.call(0x0011_cd98, &[w + 0x3ce0, b], &[]);
}
/// FUN_00133c68(w, a, b): push a pool (w+0x3cf8) node {+0xc = b->+0x60, +4 = old head} on a->+100
pub fn ee_sub_133c68<M: QMem + ?Sized>(m: &mut M, w: u32, a: u32, b: u32) {
    let n = m.call(0x0011_ccc0, &[w + 0x3cf8], &[]).0;
    let v = m.r32(b + 0x60);
    m.w32(n + 0xc, v);
    let h = m.r32(a + 100);
    m.w32(n + 4, h);
    m.w32(a + 100, n);
}
/// FUN_00133cb8(w, a, b): unlink and return the node of a->+100 whose +0xc == b->+0x60 (0 when none)
pub fn ee_sub_133cb8<M: QMem + ?Sized>(m: &mut M, w: u32, a: u32, b: u32) -> u32 {
    let mut n = m.r32(a + 100);
    if n != 0 {
        let key = m.r32(b + 0x60);
        if m.r32(n + 0xc) == key {
            let nx = m.r32(n + 4);
            m.w32(a + 100, nx);
        } else {
            let mut prev;
            loop {
                prev = n + 4;
                n = m.r32(n + 4);
                if n == 0 {
                    return 0;
                }
                if m.r32(n + 0xc) == key {
                    break;
                }
            }
            let nx = m.r32(n + 4);
            m.w32(prev, nx);
        }
    }
    n
}
/// FUN_00133d08(w, a, b): the boxes (min xyz at +0, max xyz at +0x10) overlap -> r = FUN_00134310(a->+0x60
/// slot 60 (b->+0x60), b->+0x60 slot 60 (a->+0x60)); result 0 for r == 1 / 3, else 1; no overlap -> 0
pub fn ee_sub_133d08<M: QMem + ?Sized>(m: &mut M, w: u32, a: u32, b: u32) -> u32 {
    let mut r = 0;
    if m.rf(a) < m.rf(b + 0x10)
        && m.rf(a + 4) < m.rf(b + 0x14)
        && m.rf(a + 8) < m.rf(b + 0x18)
        && m.rf(b) < m.rf(a + 0x10)
        && m.rf(b + 4) < m.rf(a + 0x14)
        && m.rf(b + 8) < m.rf(a + 0x18)
    {
        let (ao, bo) = (m.r32(a + 0x60), m.r32(b + 0x60));
        let u2 = vslot(m, ao, 60, &[bo], &[]).0;
        let u3 = vslot(m, bo, 60, &[ao], &[]).0;
        let l = m.call(0x0013_4310, &[u2, u3], &[]).0 as i32 as i64;
        r = 1;
        if l == 1 || l == 3 {
            r = 0;
        }
    }
    r
}
/// FUN_00133e30(w): while w+0x3d28 is non-empty: slot 48 (+0x18c) on the head's +0x60 object
pub fn ee_sub_133e30<M: QMem + ?Sized>(m: &mut M, w: u32) {
    while m.r32(w + 0x3d28) != 0 {
        let h = m.r32(w + 0x3d28);
        let ob = m.r32(h + 0x60);
        vslot(m, ob, 48, &[], &[]);
    }
}

// ---------------------------------------------------------------- run_001444e0 (guard lure / noise-alert component)
/// FUN_001444e0 / FUN_001449d0: the pool refcount init of FUN_001ba9f8; FUN_00144618 / FUN_00144b08: (1, 0xffff)
pub fn ee_sub_1444e0<M: QMem + ?Sized>(m: &mut M, on: u32, k: u32) {
    pools_init(m, on, k);
}
pub fn ee_sub_1449d0<M: QMem + ?Sized>(m: &mut M, on: u32, k: u32) {
    pools_init(m, on, k);
}
pub fn ee_sub_144618<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_1444e0(m, 1, 0xffff);
}
pub fn ee_sub_144b08<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_1449d0(m, 1, 0xffff);
}
/// FUN_00144640(this): [0] = owner, [4..8] = last position quad, [8] = last distance, [9] = next check time,
/// [10] = busy flag. Idle: when FUN_00144068(owner) and [9] <= clock: [9] = clock + DAT_002bd2d0; the owner's
/// link partners (list *(*(owner+0xec)+100)) passing FUN_00132b10(p, 0x38) alert (FollowWorld_activity /
/// FUN_001f7bf8(0.5, p)) and end the check (when FUN_0020f0b0(world)); then, with +0x1084 == 0 a component
/// (0xa8) state in 0..1 / else +0x1084 -> +0x40 -> 0x24, the distance to the stored position is tracked and a
/// FUN_00173610 / FUN_0017d608 hit sends FUN_0019f540(owner, pos) and [10] = 1. Busy: cleared when owner +0x2dc == -1
pub fn ee_sub_144640<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.r32(o + 0x28) == 0 {
        let ow = m.r32(o);
        let l = m.call(0x0014_4068, &[ow], &[]).0;
        if l != 0 && !(m.rf(o + 0x24) > m.rf(CLOCK)) {
            let v = m.rf(CLOCK) + rof(0x002b_d2d0);
            m.wf(o + 0x24, v);
            let iv5;
            let w = m.call(0x0020_f0b0, &[0x003d_d490], &[]).0 & 0xff;
            if w == 0 {
                iv5 = m.r32(o);
            } else {
                let ow = m.r32(o);
                let h = m.r32(ow + 0xec);
                let mut p6 = m.r32(h + 100);
                let mut b1 = false;
                'walk: {
                    if p6 == 0 {
                        break 'walk;
                    }
                    let mut item = m.r32(p6);
                    loop {
                        if item != 0 {
                            loop {
                                let ow = m.r32(o);
                                let l8 = m.r32(item + 8);
                                let p = m.r32(item + if ow == l8 { 0x40 } else { 0 } + 8);
                                if m.call(0x0013_2b10, &[p, 0x38], &[]).0 & 0xff == 0 {
                                    item = m.r32(item + 0xac);
                                } else {
                                    let c = m.call(0x0020_5458, &[p, 0xa9], &[]).0;
                                    if c == 0 || m.r32(c + 0x4c) == 0 {
                                        let fv = m.rf(p + 0x130);
                                        if 0.0 <= fv {
                                            m.call(0x0016_19a0, &[0x002d_eb30, 0x17, p], &[]);
                                        }
                                        b1 = true;
                                        m.call(0x001f_7bf8, &[p], &[f(0.5)]);
                                    }
                                    item = m.r32(item + 0xac);
                                }
                                if item == 0 {
                                    break;
                                }
                            }
                        }
                        p6 = m.r32(p6 + 4);
                        if p6 == 0 {
                            break 'walk;
                        }
                        item = m.r32(p6);
                    }
                }
                if b1 {
                    return;
                }
                iv5 = m.r32(o);
            }
            let c84 = m.r32(iv5 + 0x1084);
            if c84 == 0 {
                let c = m.call(0x0020_5458, &[iv5, 0xa8], &[]).0;
                if c == 0 {
                    return;
                }
                let k = m.r32(c + 0x4c) as i32;
                if 1 < k || k < 0 {
                    return;
                }
            } else {
                let t = m.r32(c84 + 0x40);
                if m.r32(t) != 0x24 {
                    return;
                }
            }
            let ow = m.r32(o);
            let pos = m.call(0x001a_62e8, &[ow], &[]).1;
            let q = m.rv(o + 0x10);
            let d = dist3(pos, q);
            if m.rf(o + 0x20) - d < rof(0x002b_d2d4) * rof(0x002b_d2d0) {
                let buf = stack_push(m, 0x10);
                m.wv(buf, q);
                let hit = m.call(0x0017_3610, &[0x42c8_0000, 0xc316_0000, buf, 0, 0], &[]).0;
                if hit == 0 && m.call(0x0017_d608, &[buf], &[]).0 & 0xff == 0 {
                    m.wf(o + 0x20, d);
                    stack_pop(m, 0x10);
                    return;
                }
                let ow = m.r32(o);
                let bq = m.rv(buf);
                m.call(0x0019_f540, &[ow], &[bq]);
                m.w32(o + 0x28, 1);
                stack_pop(m, 0x10);
            }
            m.wf(o + 0x20, d);
        }
    } else {
        let ow = m.r32(o);
        if m.r32(ow + 0x2dc) == NEG1 {
            m.w32(o + 0x28, 0);
        }
    }
}
/// FUN_001448f0(this, q): idle ([10] == 0) and |[4..8] - q| >= DAT_002bd2d8: [4..8] = q,
/// [8] = |pos(owner) - q|, [9] = clock + DAT_002bd2d0
pub fn ee_sub_1448f0<M: QMem + ?Sized>(m: &mut M, o: u32, q: V4) {
    if m.r32(o + 0x28) == 0 {
        let c = m.rv(o + 0x10);
        let d = dist3(c, q);
        if rof(0x002b_d2d8) <= d {
            m.wv(o + 0x10, q);
            let ow = m.r32(o);
            let pos = m.call(0x001a_62e8, &[ow], &[]).1;
            let fv = rof(0x002b_d2d0);
            let c = m.rv(o + 0x10);
            let d2 = dist3(pos, c);
            m.wf(o + 0x20, d2);
            let t = m.rf(CLOCK) + fv;
            m.wf(o + 0x24, t);
        }
    }
}
/// FUN_001449c8: +0x28
pub fn ee_sub_1449c8<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    m.r32(o + 0x28)
}
/// FUN_00144ee8: FUN_00178d00(+0x18) != 0 || (+0x18)->+0x4c == 2
pub fn ee_sub_144ee8<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let i = m.r32(o + 0x18);
    let l = m.call(0x0017_8d00, &[i], &[]).0 & 0xff;
    (l != 0 || m.r32(i + 0x4c) == 2) as u32
}
/// FUN_00144f48(this, q): +0x40 = q; with a target (+0x18): owner (+0x10) AI record r = owner+0x294; when no
/// chase component (owner+0xe8 -> +0xdc -> +0x10) or +0x30 != 0: r+0xe8 = 2, *(r+0xf4) = owner pos,
/// *(r+0xf4 + 0x10) = q, FUN_001e6180(r); else C2d1080_chase(comp, pos, q, r); then FUN_00178d90(+0x18, r, 0)
pub fn ee_sub_144f48<M: QMem + ?Sized>(m: &mut M, o: u32, q: V4) {
    m.wv(o + 0x40, q);
    let mut iv7 = 0;
    if m.r32(o + 0x18) != 0 {
        let i1 = m.r32(o + 0x10);
        let i2 = m.r32(i1 + 0x294);
        let e8 = m.r32(i1 + 0xe8);
        if e8 != 0 {
            iv7 = m.r32(e8 + 0xdc);
        }
        let mut iv8 = 0;
        if iv7 != 0 {
            iv8 = m.r32(iv7 + 0x10);
        }
        let pos = m.call(0x001a_62e8, &[i1], &[]).1;
        if iv8 == 0 || m.r32(o + 0x30) != 0 {
            let p3 = m.r32(i2 + 0xf4);
            m.w32(i2 + 0xe8, 2);
            m.wv(p3, pos);
            let p7 = m.r32(i2 + 0xf4);
            let qq = m.rv(o + 0x40);
            m.wv(p7 + 0x10, qq);
            m.call(0x001e_6180, &[i2], &[]);
        } else {
            let qq = m.rv(o + 0x40);
            m.call(0x001c_02d0, &[iv8, i2], &[pos, qq]);
        }
        let u = m.r32(o + 0x18);
        m.call(0x0017_8d90, &[u, i2, 0], &[]);
    }
}
