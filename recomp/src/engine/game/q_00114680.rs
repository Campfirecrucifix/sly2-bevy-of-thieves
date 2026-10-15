//! WP-Q13: file:run_00114680 .. run_00148300 (small free functions skipped by earlier workers).
//! Shared pieces: the header static-initialiser `static_init_stores` (38 identical copies in the ELF, one per
//! translation unit: refcount DAT_002dd2e0.. and first-use pool creation FUN_0011cca8) and the refcounted
//! node release used by the splice-list builders. Cross-package callees go through `QMem::call`.
//! SKIPPED: FUN_00139950 (batch transform through the VU0 microprogram, `vcallms 0x10`; microcode not decoded).
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem, V4};

const POOL_CREATE: u32 = 0x0011_cca8;
const POOL_FREE: u32 = 0x0011_cd98;

/// (counter address, pool, element size, count, align) of the six common stores
const STORES6: [(u32, u32, u32, u32); 6] = [
    (0x002d_d2e0, 0x4520a8, 0x14, 0x2000),
    (0x002d_d2e4, 0x4520d0, 0x18, 0x800),
    (0x002d_d2e8, 0x4520f8, 0x28, 0x200),
    (0x002d_d2ec, 0x452120, 0x1c, 0x800),
    (0x002d_d2f0, 0x452148, 0xc, 200),
    (0x002d_d2f4, 0x452170, 0x14, 0x100),
];

/// the 38 identical static-initialiser bodies (FUN_00114680, FUN_00118510, ...): for param_2 == 0xffff and
/// param_1 != 0 bump each use counter and create the pool on the first use (align 0x10)
pub fn static_init_stores<M: QMem + ?Sized>(m: &mut M, p1: u64, p2: u64) {
    if p2 == 0xffff && p1 != 0 {
        for (ctr, pool, sz, cnt) in STORES6 {
            let v = m.r32(ctr).wrapping_add(1);
            m.w32(ctr, v);
            if v == 1 {
                m.call(POOL_CREATE, &[pool, sz, cnt, 0x10], &[]);
            }
        }
    }
}

/// FUN_0012aeb0: the 11-store variant (the six common ones + 0x2dd300..0x2dd310)
pub fn ee_sub_12aeb0<M: QMem + ?Sized>(m: &mut M, p1: u64, p2: u64) {
    static_init_stores(m, p1, p2);
    if p2 == 0xffff && p1 != 0 {
        for (ctr, pool, sz, cnt) in [
            (0x002d_d300u32, 0x452188u32, 0x20u32, 0x100u32),
            (0x002d_d304, 0x4521a0, 0x50, 0x100),
            (0x002d_d308, 0x4521b8, 0x20, 0x40),
            (0x002d_d30c, 0x4521d0, 0xc, 0x40),
            (0x002d_d310, 0x4521e8, 0x10, 0x20),
        ] {
            let v = m.r32(ctr).wrapping_add(1);
            m.w32(ctr, v);
            if v == 1 {
                m.call(POOL_CREATE, &[pool, sz, cnt, 0x10], &[]);
            }
        }
    }
}

/// `static_init_stores` entry points (same body, different address) and their `(1, 0xffff)` wrappers
macro_rules! init_fns {
    ($($name:ident),*) => { $(
        pub fn $name<M: QMem + ?Sized>(m: &mut M, p1: u64, p2: u64) { static_init_stores(m, p1, p2) }
    )* };
}
init_fns!(
    ee_sub_114680, ee_sub_118510, ee_sub_1279c0, ee_sub_127fe0, ee_sub_12a828, ee_sub_12acc0, ee_sub_12b258,
    ee_sub_134398, ee_sub_1344f0, ee_sub_138db0, ee_sub_13d400, ee_sub_13d8c0, ee_sub_13e210, ee_sub_1484c8,
    ee_sub_14ce88, ee_sub_14d280, ee_sub_14fb50, ee_sub_171e00, ee_sub_190f38, ee_sub_1957f8, ee_sub_19caa0,
    ee_sub_19dcb8, ee_sub_1a2a78, ee_sub_1a2ec8, ee_sub_1a3108, ee_sub_1b26c8, ee_sub_1b45d0, ee_sub_1cd498,
    ee_sub_1cdb78, ee_sub_1fae18, ee_sub_1ff538, ee_sub_1ff690, ee_sub_1ffba0, ee_sub_223e18, ee_sub_225880,
    ee_sub_2263f8, ee_sub_2289f0, ee_sub_22aee8
);

/// FUN_001147b8 / 00118648 / 001279... : `init(1, 0xffff)` wrappers
macro_rules! init_wrappers {
    ($($name:ident => $target:ident),*) => { $(
        pub fn $name<M: QMem + ?Sized>(m: &mut M) { $target(m, 1, 0xffff) }
    )* };
}
init_wrappers!(
    ee_sub_1147b8 => ee_sub_114680, ee_sub_118648 => ee_sub_118510, ee_sub_127af8 => ee_sub_1279c0,
    ee_sub_128118 => ee_sub_127fe0, ee_sub_12a960 => ee_sub_12a828, ee_sub_12adf8 => ee_sub_12acc0,
    ee_sub_12b0c8 => ee_sub_12aeb0, ee_sub_1344d0 => ee_sub_134398, ee_sub_134628 => ee_sub_1344f0,
    ee_sub_138ee8 => ee_sub_138db0, ee_sub_13d538 => ee_sub_13d400, ee_sub_13d9f8 => ee_sub_13d8c0,
    ee_sub_13e348 => ee_sub_13e210, ee_sub_148600 => ee_sub_1484c8
);

/// FUN_00114ac0: `font_static_init_stores_1147d8(1, 0xffff)`
pub fn ee_sub_114ac0<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x0011_47d8, &[1, 0xffff], &[]);
}

/// the inlined refcounted-node release (count at +0; on zero: FUN_00257250(+0x14, 2), Release(+0xc),
/// FUN_00255438(node), pool free FUN_0011cd98(0x4520d0, node))
fn node_release<M: QMem + ?Sized>(m: &mut M, p: u32) {
    if p != 0 {
        let c = m.r32(p).wrapping_sub(1);
        m.w32(p, c);
        if c == 0 {
            m.call(0x0025_7250, &[p + 0x14, 2], &[]);
            m.call(0x0012_9ec8, &[p + 0xc], &[]);
            m.call(0x0025_5438, &[p], &[]);
            m.call(POOL_FREE, &[0x4520d0, p], &[]);
        }
    }
}

// ---- run_00118480: BinaryInputStream-derived object (vtable DAT_002ca5a0 at +0x30)
/// FUN_00118480: ctor
pub fn ee_sub_118480<M: QMem + ?Sized>(m: &mut M, this: u32) -> u32 {
    m.call(0x0011_77a8, &[this], &[]);
    m.w32(this + 0x30, 0x002c_a5a0);
    this
}
/// FUN_001184b8: dtor (`flags & 1` -> operator delete FUN_001ae520)
pub fn ee_sub_1184b8<M: QMem + ?Sized>(m: &mut M, this: u32, flags: u64) {
    m.w32(this + 0x30, 0x002c_a5a0);
    if flags & 1 != 0 {
        m.call(0x001a_e520, &[this], &[]);
    }
}
/// FUN_00118500
pub fn ee_sub_118500<M: QMem + ?Sized>(m: &mut M, p: u32, a: u32, b: u32, c: u32) {
    m.w32(p, c);
    m.w32(p + 0x34, a);
    m.w32(p + 0x38, b);
}

// ---- run_001279c0: splice list nodes
/// FUN_00127b40: append slot (count at +8, items at +0xc), FUN_00256ee0(&item) default-constructs it
pub fn ee_sub_127b40<M: QMem + ?Sized>(m: &mut M, l: u32) {
    let n = m.r32(l + 8);
    m.w32(l + 8, n.wrapping_add(1));
    m.call(0x0025_6ee0, &[n.wrapping_mul(4).wrapping_add(l).wrapping_add(0xc)], &[]);
}
/// FUN_00127b70: remove the first item equal to `v`, shifting the tail down
pub fn ee_sub_127b70<M: QMem + ?Sized>(m: &mut M, l: u32, v: u32) {
    let mut i: i32 = 0;
    if 0 < m.r32(l + 8) as i32 {
        let mut p = l + 0xc;
        loop {
            if m.r32(p) == v {
                m.call(0x0025_6ee0, &[p, 0], &[]);
                let n = (m.r32(l + 8) as i32).wrapping_sub(1);
                m.w32(l + 8, n as u32);
                let (a, b) = (m.r32(p), m.r32(p + 4));
                m.call(0x001a_e5a0, &[a, b, ((n - i) as u32).wrapping_mul(4)], &[]);
                return;
            }
            i += 1;
            p += 4;
            if !(i < m.r32(l + 8) as i32) {
                break;
            }
        }
    }
}
/// FUN_00127d78: FindBinding(); on success optionally FUN_00129658(dst, binding)
pub fn ee_sub_127d78<M: QMem + ?Sized>(m: &mut M, dst: u32) -> u32 {
    let b = m.call(0x0012_7db8, &[], &[]).0;
    if b == 0 {
        0
    } else {
        if dst != 0 {
            m.call(0x0012_9658, &[dst, b], &[]);
        }
        1
    }
}
/// FUN_00127e90: copy a list node (+8 count, +0xc.. items, +0x1c next-chain converted by FUN_0012b0e8)
pub fn ee_sub_127e90<M: QMem + ?Sized>(m: &mut M, dst: u32, src: u32, p3: u32) -> u32 {
    let sp = stack_push(m, 0x20);
    let (s80, s70) = (sp, sp + 0x10);
    let u = m.call(0x0025_6dd0, &[], &[]).0;
    m.call(0x0025_7058, &[s80, u], &[]);
    let p0 = m.r32(s80);
    let cnt = m.r32(src + 8);
    m.w32(p0 + 8, cnt);
    let tail;
    if (m.r32(src + 8) as i32) < 1 {
        tail = m.r32(src + 0x1c);
    } else {
        let mut i: u32 = 0;
        loop {
            let v = m.r32(src + 0xc + 4 * i);
            let nx = i + 1;
            let base = m.r32(s80);
            m.call(0x0025_6ee0, &[base.wrapping_add((i + 3) * 4), v], &[]);
            i = nx;
            if !((i as i32) < m.r32(src + 8) as i32) {
                break;
            }
        }
        tail = m.r32(src + 0x1c);
    }
    let pi = m.r32(s80);
    if tail != 0 {
        ee_sub_12b0e8(m, s70, tail, pi, p3);
        let t = m.r32(s70);
        m.call(0x0025_7450, &[pi + 0x1c, t], &[]);
        let t = m.r32(s70);
        node_release(m, t);
    }
    let cur = m.r32(s80);
    m.call(0x0025_7058, &[dst, cur], &[]);
    let cur = m.r32(s80);
    if cur != 0 {
        let c = m.r32(cur).wrapping_sub(1);
        m.w32(cur, c);
        if c == 0 {
            m.call(0x0025_7078, &[cur, 3], &[]);
        }
    }
    stack_pop(m, 0x20);
    dst
}

// ---- run_0012a828
/// FUN_0012a980
pub fn ee_sub_12a980<M: QMem + ?Sized>(m: &mut M, p1: u32, p2: u32, p3: u32, p4: u32) -> u32 {
    let sp = stack_push(m, 0x10);
    let u = m.call(0x0025_7340, &[], &[]).0;
    m.call(0x0025_7430, &[sp, u], &[]);
    let n = m.r32(sp);
    m.w32(n + 8, p3);
    m.call(0x0012_9658, &[n + 0xc, p4], &[]);
    let nx = m.r32(p2 + 8);
    m.call(0x0025_7450, &[n + 0x14, nx], &[]);
    let n2 = m.r32(sp);
    m.call(0x0025_7450, &[p2 + 8, n2], &[]);
    m.call(0x0012_a0b0, &[p1, 0x24], &[]); // SetTag
    let n3 = m.r32(sp);
    node_release(m, n3);
    stack_pop(m, 0x10);
    p1
}
/// FUN_0012ae18
pub fn ee_sub_12ae18<M: QMem + ?Sized>(m: &mut M) {
    m.w32(0x002d_d5b8, 1);
}
/// FUN_0012ae28
pub fn ee_sub_12ae28<M: QMem + ?Sized>(m: &mut M) {
    m.w32(0x002d_d5b8, 0);
    for f in [0x0025_7e88u32, 0x0025_7ec8, 0x0025_7f08, 0x0025_7f48, 0x0025_7f88, 0x0025_7fc8] {
        m.call(f, &[], &[]);
    }
    for p in [0x452188u32, 0x4521a0, 0x4521b8, 0x4521d0, 0x4521e8] {
        m.call(0x0011_cdb8, &[p], &[]);
    }
}
/// FUN_0012b0e8: recursive copy of a node chain (+8 id, +0xc payload, +0x14 next)
pub fn ee_sub_12b0e8<M: QMem + ?Sized>(m: &mut M, p1: u32, p2: u32, p3: u32, p4: u32) -> u32 {
    let sp = stack_push(m, 0x20);
    let (s90, s80) = (sp + 0x10, sp);
    let u = m.call(0x0025_7340, &[], &[]).0;
    m.call(0x0025_7430, &[s90, u], &[]);
    let n = m.r32(s90);
    let id = m.r32(p2 + 8);
    m.w32(n + 8, id);
    m.call(0x0012_9bc8, &[s80, p2 + 0xc, p3, p4], &[]);
    m.call(0x0012_9658, &[n + 0xc, s80], &[]);
    m.call(0x0012_9ec8, &[s80], &[]); // Release
    let n = m.r32(s90);
    let nxt = m.r32(p2 + 0x14);
    if nxt != 0 {
        ee_sub_12b0e8(m, s80, nxt, p3, p4);
        let t = m.r32(s80);
        m.call(0x0025_7450, &[n + 0x14, t], &[]);
        let t = m.r32(s80);
        node_release(m, t);
    }
    let n = m.r32(s90);
    m.call(0x0025_7430, &[p1, n], &[]);
    let n = m.r32(s90);
    node_release(m, n);
    stack_pop(m, 0x20);
    p1
}

// ---- run_00134320 .. run_001383f0
/// FUN_00134320: for every body on the list +0x3d10 (next at +0x48) re-run the broadphase update against
/// the object's +0xec target
pub fn ee_sub_134320<M: QMem + ?Sized>(m: &mut M, w: u32, o: u32) {
    if m.r32(o + 0xec) != 0 {
        m.call(0x001f_4bf8, &[o], &[]);
        let mut i2 = m.r32(w + 0x3d10);
        if i2 != 0 {
            let mut i1 = m.r32(o + 0xec);
            loop {
                if i2 != i1 {
                    m.call(0x0013_3a38, &[w, i1, i2, 0], &[]);
                }
                i2 = m.r32(i2 + 0x48);
                if i2 == 0 {
                    break;
                }
                i1 = m.r32(o + 0xec);
            }
        }
    }
}

/// FUN_001356e8: fetch two quads via FUN_0014f268(0x2ddf40, DAT_002bc970/980), ClearWorld, FUN_0014cc58
/// (closest hit); +0x16f0 = hit point or the second quad
pub fn ee_sub_1356e8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let k0 = m.r32(0x002b_c970);
    let k1 = m.r32(0x002b_c980);
    let v6 = m.call(0x0014_f268, &[0x002d_df40, k0], &[]).1;
    let v5 = m.call(0x0014_f268, &[0x002d_df40, k1], &[]).1;
    let sp = stack_push(m, 0xd0);
    m.call(0x001a_e478, &[sp], &[]);
    let (hit, n_a, n_b) = (sp + 0x10, sp + 0x70, sp + 0x74);
    let tgt = m.r32(o + 0xec);
    m.call(0x0020_faa0, &[0x003d_d490, 0x0020_fa10, tgt, n_a, n_b], &[v6, v5]);
    let (cnt, tbl) = (m.r32(n_a), m.r32(n_b));
    let r1 = m.call(0x0014_cc58, &[1, cnt, tbl, hit], &[v6, v5]).0;
    if r1 == 0 {
        m.wv(o + 0x16f0, v5);
    } else {
        for i in 0..4 {
            let w = m.r32(hit + 4 * i);
            m.w32(o + 0x16f0 + 4 * i, w);
        }
    }
    m.call(0x001a_e4a0, &[sp, 2], &[]);
    stack_pop(m, 0xd0);
}

/// FUN_00136ed0: event 9 for our target: 0x16e -> FUN_00131b10 + flag +0x10, 0x242 -> flag +0xc
pub fn ee_sub_136ed0<M: QMem + ?Sized>(m: &mut M, s: u32, ev: i32, msg: u32) {
    if ev == 9 && m.r32(msg) == m.r32(s + 0xe0) {
        let k = m.r32(msg + 8);
        if k == 0x16e {
            let a = m.r32(msg);
            let w = m.r32(s);
            let x = m.r32(w + 0x10b8);
            m.call(0x0013_1b10, &[a, w, x], &[]);
            m.w32(s + 0x10, 1);
        } else if k == 0x242 {
            m.w32(s + 0xc, 1);
        }
    }
}

/// FUN_001383f0: set +0xd4; on change reset the render state (+0x80/+0x84 = DAT_002a4af0, +0x88/+0x8c =
/// 0x2c44e0, +0x9a = 0) of every child with a valid FUN_0012cee0 in the tree under +0x50 (children +0x34 /
/// siblings +0x28, branches +0x2c)
pub fn ee_sub_1383f0<M: QMem + ?Sized>(m: &mut M, o: u32, v: u32) {
    if v == m.r32(o + 0xd4) {
        return;
    }
    let mut i5 = m.r32(o + 0x50);
    m.w32(o + 0xd4, v);
    if i5 == 0 {
        return;
    }
    let mut i4 = m.r32(i5 + 0x34);
    loop {
        let u1 = m.r32(0x002a_4af0);
        if i4 == 0 {
            i5 = m.r32(i5 + 0x2c);
        } else {
            let mut i2 = m.r32(i4 + 0x1c);
            loop {
                if i2 == 0 {
                    i4 = m.r32(i4 + 0x28);
                } else if m.call(0x0012_cee0, &[i4], &[]).0 == 0 {
                    i4 = m.r32(i4 + 0x28);
                } else {
                    let d = m.r32(i4 + 8);
                    m.w16(d + 0x9a, 0);
                    m.w32(d + 0x84, u1);
                    m.w32(d + 0x80, u1);
                    m.w32(d + 0x8c, 0x002c_44e0);
                    m.w32(d + 0x88, 0x002c_44e0);
                    i4 = m.r32(i4 + 0x28);
                }
                if i4 == 0 {
                    break;
                }
                i2 = m.r32(i4 + 0x1c);
            }
            i5 = m.r32(i5 + 0x2c);
        }
        if i5 == 0 {
            break;
        }
        i4 = m.r32(i5 + 0x34);
    }
}

/// FUN_00138ff8: allocate an actor slot from pool +0x4088 (second pool +0x40a0 when owner kind +0x160 == 2)
pub fn ee_sub_138ff8<M: QMem + ?Sized>(m: &mut M, w: u32, owner: u32) -> u32 {
    let a = m.call(0x0011_ccc0, &[w + 0x4088], &[]).0;
    if a == 0 {
        return 0;
    }
    if m.r32(owner + 0x160) == 2 {
        let b = m.call(0x0011_ccc0, &[w + 0x40a0], &[]).0;
        m.w32(a + 0x1088, b);
        if b == 0 {
            m.call(POOL_FREE, &[w + 0x4088, a], &[]);
            return 0;
        }
        m.w32(a + 0x108c, owner);
    } else {
        m.w32(a + 0x108c, owner);
    }
    m.call(0x0011_a220, &[owner + 0x448, a], &[]);
    a
}

// ---- run_0013d1a8: byte ring buffer (+0 buf, +4 size, +8 write pos, +0xc read pos, +0x10 free, +0x14 used)
/// FUN_0013d1a8: build the 3-word mission event and post it
pub fn ee_sub_13d1a8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let l2 = m.call(0x0021_0a88, &[], &[]).0;
    let l3 = if l2 as i32 != -1 { l2 } else { 0 };
    let sp = stack_push(m, 0x30);
    let a = m.call(0x001d_8720, &[0x003e_0eb0, 0x002a_4bb8], &[]).0;
    m.w16(sp, a as u16);
    let b = m.call(0x001d_8720, &[0x003e_0eb0, 0x002a_4bc0], &[]).0;
    m.w16(sp + 2, b as u16);
    let c = m.r16(o + 0x180);
    m.w16(sp + 4, c);
    m.w32(sp + 0x10, 3);
    m.w32(sp + 0x14, sp);
    let u = m.call(0x0021_09b8, &[l3], &[]).0;
    m.call(0x001d_8cc8, &[u, sp + 0x10], &[]);
    stack_pop(m, 0x30);
}
/// FUN_0013d558
pub fn ee_sub_13d558<M: QMem + ?Sized>(m: &mut M, p: u32, buf: u32, size: u32) {
    m.w32(p + 4, buf);
    m.w32(p, size);
    ee_sub_13d580(m, p);
}
/// FUN_0013d580
pub fn ee_sub_13d580<M: QMem + ?Sized>(m: &mut M, p: u32) {
    let v = m.r32(p + 4);
    m.w32(p + 0x1c, 0);
    m.w32(p + 0x10, v);
    m.w32(p + 0x18, 0);
    m.w32(p + 0x14, 0);
    m.w32(p + 0xc, 0);
    m.w32(p + 8, 0);
}
/// virtual slot at byte +0xc of the stream's vtable: `read(this + adj(+8), n, dst) -> bytes`
fn s_read<M: QMem + ?Sized>(m: &mut M, s: u32, n: u32, dst: u32) -> u32 {
    let vt = m.r32(s);
    let adj = m.r16(vt + 8) as i16 as i32 as u32;
    let f = m.r32(vt + 0xc);
    m.call(f, &[s.wrapping_add(adj), n, dst], &[]).0
}
/// FUN_0013d5a0: pull up to `want` bytes (<= 0 or > avail -> all of +0x10) from stream `s` into the ring.
/// `trap(7)` (divide by a zero ring size) ends the call.
pub fn ee_sub_13d5a0<M: QMem + ?Sized>(m: &mut M, rb: u32, want: i32, s: u32) -> i32 {
    let mut done: i32 = 0;
    let mut left = want;
    if want < 1 || (m.r32(rb + 0x10) as i32) < want {
        left = m.r32(rb + 0x10) as i32;
    }
    loop {
        if left == 0 {
            return done;
        }
        let size = m.r32(rb + 4) as i32;
        let mut n = size.wrapping_sub(m.r32(rb + 8) as i32);
        if left < n {
            n = left;
        }
        let dst = m.r32(rb).wrapping_add(m.r32(rb + 8));
        let got = s_read(m, s, n as u32, dst) as i32;
        if got == 0 {
            break;
        }
        let w = m.r32(rb + 8) as i32;
        done += got;
        left -= got;
        m.w32(rb + 8, w.wrapping_add(got) as u32);
        if size == 0 {
            return done; // trap(7)
        }
        for (o, d) in [(0x14u32, got), (0x10, -got), (0x1c, got)] {
            let v = (m.r32(rb + o) as i32).wrapping_add(d);
            m.w32(rb + o, v as u32);
        }
        m.w32(rb + 8, w.wrapping_add(got).wrapping_rem(size) as u32);
        if want < 1 {
            return done;
        }
    }
    done
}
/// FUN_0013d6a8: drains `+0x14` bytes from the read side (+0xc) into the sink `s`; the sink's vtable slot
/// +0x14 selects the +0x10 / +0x18 counter
pub fn ee_sub_13d6a8<M: QMem + ?Sized>(m: &mut M, rb: u32, want: i32, s: u32) -> i32 {
    let mut done: i32 = 0;
    let mut left = want;
    if want < 1 || (m.r32(rb + 0x14) as i32) < want {
        left = m.r32(rb + 0x14) as i32;
    }
    loop {
        if left == 0 {
            return done;
        }
        let size = m.r32(rb + 4) as i32;
        let mut n = size.wrapping_sub(m.r32(rb + 0xc) as i32);
        if left < n {
            n = left;
        }
        let src = m.r32(rb).wrapping_add(m.r32(rb + 0xc));
        let got = s_read(m, s, n as u32, src) as i32;
        if got == 0 {
            break;
        }
        let rp = m.r32(rb + 0xc) as i32;
        m.w32(rb + 0xc, rp.wrapping_add(got) as u32);
        if size == 0 {
            return done; // trap(7)
        }
        let c = (m.r32(rb + 0x14) as i32).wrapping_sub(got);
        m.w32(rb + 0x14, c as u32);
        m.w32(rb + 0xc, rp.wrapping_add(got).wrapping_rem(size) as u32);
        let vt = m.r32(s);
        let adj = m.r16(vt + 0x10) as i16 as i32 as u32;
        let f = m.r32(vt + 0x14);
        let k = m.call(f, &[s.wrapping_add(adj)], &[]).0;
        let off = if k == 0 { 0x10 } else { 0x18 };
        let v = (m.r32(rb + off) as i32).wrapping_add(got);
        m.w32(rb + off, v as u32);
        done += got;
        left -= got;
        if want < 1 {
            return done;
        }
    }
    done
}
/// FUN_0013d7d0: wrap a memory range (vtable DAT_002cbe20) as the source of 13d5a0
pub fn ee_sub_13d7d0<M: QMem + ?Sized>(m: &mut M, rb: u32, n: u64, p3: u32) -> i32 {
    if n == 0 {
        return 0;
    }
    let sp = stack_push(m, 0x20);
    m.w32(sp, 0x002c_be20);
    m.w32(sp + 8, n as u32);
    m.w32(sp + 4, p3);
    m.w32(sp + 0xc, 0);
    m.w32(sp + 0x10, n as u32);
    let r = ee_sub_13d5a0(m, rb, n as i32, sp);
    stack_pop(m, 0x20);
    r
}
/// FUN_0013d818: advance the consumed count (0 = all pending +0x18)
pub fn ee_sub_13d818<M: QMem + ?Sized>(m: &mut M, p: u32, n: i32) {
    let mut n = n;
    if n == 0 {
        n = m.r32(p + 0x18) as i32;
        if n == 0 {
            return;
        }
    }
    let a = (m.r32(p + 0x10) as i32).wrapping_add(n);
    m.w32(p + 0x10, a as u32);
    let b = (m.r32(p + 0x18) as i32).wrapping_sub(n);
    m.w32(p + 0x18, b as u32);
}
/// FUN_0013d850: copy min(n, +0x10) bytes from buffer (+4) at read pos +0xc to `dst` (memcpy FUN_0023320c)
pub fn ee_sub_13d850<M: QMem + ?Sized>(m: &mut M, p: u32, n: i32, dst: u32) -> i32 {
    let mut k = m.r32(p + 0x10) as i32;
    if n < k {
        k = n;
    }
    let src = m.r32(p + 4).wrapping_add(m.r32(p + 0xc));
    m.call(0x0023_320c, &[dst, src, k as u32], &[]);
    let a = m.r32(p + 0xc).wrapping_add(k as u32);
    m.w32(p + 0xc, a);
    let b = (m.r32(p + 0x10) as i32).wrapping_sub(k);
    m.w32(p + 0x10, b as u32);
    k
}

// ---- run_0013e210
/// FUN_0013e590: field loader: obj[+off] = FUN_0015e938(0x3dd490, 4, desc[0], obj)
pub fn ee_sub_13e590<M: QMem + ?Sized>(m: &mut M, obj: u32, d: u32) {
    let k = m.r32(d);
    let v = m.call(0x0015_e938, &[0x003d_d490, 4, k, obj], &[]).0;
    let off = m.r32(d + 4);
    m.w32(obj.wrapping_add(off), v);
}
/// FUN_0013e5e0: loads a value, then calls the setter named by the descriptor (direct fn at +8 or a
/// table-indexed (fn, this-adjust) pair when desc.s16(+6) >= 0)
pub fn ee_sub_13e5e0<M: QMem + ?Sized>(m: &mut M, obj: u32, d: u32) {
    let k = m.r32(d);
    let v = m.call(0x0015_e938, &[0x003d_d490, 4, k, obj], &[]).0;
    let idx = m.r16(d + 6) as i16 as i32;
    let (f, adj);
    if idx < 0 {
        f = m.r32(d + 8);
        adj = m.r16(d + 4) as i16 as i32;
    } else {
        let toff = m.r16(d + 8) as i16 as i32 as u32;
        let t = m.r32(obj.wrapping_add(toff));
        let e = m.r64(t.wrapping_add((idx * 8) as u32).wrapping_sub(8));
        f = (e >> 32) as u32;
        adj = (e as u16 as i16 as i32) + (m.r16(d + 4) as i16 as i32);
    }
    m.call(f, &[obj.wrapping_add(adj as u32), v], &[]);
}

// ---- run_00148300
/// FUN_00148300: activate / deactivate `o` against the focus point by the range mode in bits 52..55 of its
/// +0x28 word (0 / 3: squared distance to the point at 0x2ddfd0 / the camera quad < radius +0x38 when the
/// mask 0x2ddf54 & byte +0x2d is set; 1: always active; 2: never). The `unaff_s2` registers of the original
/// decompile for other mode values (4..15) are taken as 0 (inf:).
pub fn ee_sub_148300<M: QMem + ?Sized>(m: &mut M, w: u32, o: u32) {
    let mask = m.r32(0x002d_df54);
    let fixed = m.rv(0x002d_dfd0);
    if m.r32(0x003e_0e70) != 0 {
        return;
    }
    let l8 = m.call(0x001c_4840, &[], &[]).0;
    let cam: V4 = if l8 == 0 { fixed } else { m.call(0x001a_62e8, &[l8], &[]).1 };
    let mode = ((m.r64(o + 0x28) as i64) << 8) >> 0x3c;
    let key = m.r32(o + 0x14);
    let mut s2: u32 = 0;
    if mode == 1 {
        s2 = 1;
    } else if (mode == 0 || mode == 3) && mask & m.r8(o + 0x2d) as u32 != 0 {
        let q = if mode == 0 { fixed } else { cam };
        let rad = m.rf(o + 0x38);
        let p = m.call(0x001a_62e8, &[o], &[]).1;
        let d = [p[0] - q[0], p[1] - q[1], p[2] - q[2]];
        s2 = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2] < rad * rad) as u32;
    }
    let set = w + 0x2c;
    let cur = m.call(0x0011_a380, &[set, key], &[]).0;
    if s2 != cur {
        let vt = m.r32(o + 0xc);
        if s2 == 0 {
            let adj = m.r16(vt + 0xb0) as i16 as i32 as u32;
            let f = m.r32(vt + 0xb4);
            m.call(f, &[o.wrapping_add(adj)], &[]);
            m.call(0x0011_a2e8, &[set, key], &[]);
        } else {
            m.call(0x0011_a220, &[set, key], &[]);
            let adj = m.r16(vt + 0xb8) as i16 as i32 as u32;
            let f = m.r32(vt + 0xbc);
            m.call(f, &[o.wrapping_add(adj)], &[]);
        }
    }
}
