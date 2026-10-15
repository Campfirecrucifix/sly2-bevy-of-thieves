//! WP-Q12 part G: file package run_002567d0 (refcounted hash-map / pool glue of the Splice / asset tables:
//! 6 typed pools {0x4520a8 (0x14), 0x4520d0 (0x18), 0x4520f8 (0x28), 0x452120 (0x1c), 0x452148 (0xc),
//! 0x452170 (0x14)} with handle words `idx16 | tag` and a mark / sweep per pool). 1:1 from ghidra_functions(_v2).body
//! on `QMem` (conventions as q12_a); other packages' functions are called by address.
#![allow(unused_variables, unused_mut, dead_code)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem, V4};
use crate::engine::classes::q::q6::vcall;

const GEN: u32 = 0x002d_d598;

/// `if p != 0 { *p += 1 }`
fn addref<M: QMem + ?Sized>(m: &mut M, p: u32) {
    if p != 0 {
        let c = m.r32(p).wrapping_add(1);
        m.w32(p, c);
    }
}
/// `if p != 0 && --*p == 0`
fn dec_zero<M: QMem + ?Sized>(m: &mut M, p: u32) -> bool {
    if p != 0 {
        let c = m.r32(p).wrapping_sub(1);
        m.w32(p, c);
        return c == 0;
    }
    false
}
/// release of a pool-0x4520a8 node (FUN_00256d40 / 129ec8 / 2550f8 / 11cd98)
fn drop_a<M: QMem + ?Sized>(m: &mut M, p: u32) {
    if dec_zero(m, p) {
        ee_sub_256d40(m, p + 0x10, 2);
        m.call(0x0012_9ec8, &[p + 8], &[]);
        m.call(0x0025_50f8, &[p], &[]);
        m.call(0x0011_cd98, &[0x0045_20a8, p], &[]);
    }
}
/// release of a pool-0x4520d0 node (FUN_00257250 / 129ec8 / 255438 / 11cd98)
fn drop_b<M: QMem + ?Sized>(m: &mut M, p: u32) {
    if dec_zero(m, p) {
        ee_sub_257250(m, p + 0x14, 2);
        m.call(0x0012_9ec8, &[p + 0xc], &[]);
        m.call(0x0025_5438, &[p], &[]);
        m.call(0x0011_cd98, &[0x0045_20d0, p], &[]);
    }
}
/// FUN_002567d0(h): free every chain node (allocator [h+0xc] virtual +0x1c), clear and free the bucket array
pub fn ee_sub_2567d0<M: QMem + ?Sized>(m: &mut M, h: u32) {
    let mut iv4 = 0i32;
    if 0 < m.r32(h) as i32 {
        let mut iv3 = m.r32(h + 4);
        loop {
            iv3 = m.r32((iv4 as u32).wrapping_mul(4).wrapping_add(iv3));
            iv4 += 1;
            if iv3 != 0 {
                let mut p1 = m.r32(h + 0xc);
                loop {
                    let iv2 = m.r32(iv3 + 0xc);
                    vcall(m, p1, 0, 0x1c, &[iv3], &[]);
                    if iv2 == 0 {
                        break;
                    }
                    p1 = m.r32(h + 0xc);
                    iv3 = iv2;
                }
            }
            if m.r32(h) as i32 <= iv4 {
                break;
            }
            iv3 = m.r32(h + 4);
        }
    }
    let (b, n) = (m.r32(h + 4), m.r32(h) << 2);
    m.call(0x0023_33b4, &[b, 0, n], &[]);
    m.w32(h + 8, 0);
    let a = m.r32(h + 0xc);
    let b = m.r32(h + 4);
    vcall(m, a, 0, 0x1c, &[b], &[]);
    m.w32(h + 4, 0);
    m.w32(h, 0);
}
pub fn ee_sub_2568a0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    ee_sub_2567d0(m, o + 0x14);
}
/// FUN_002568c0(h, n): rehash into n buckets (allocator virtual +0x14: realloc(old, n << 2, 3))
pub fn ee_sub_2568c0<M: QMem + ?Sized>(m: &mut M, h: u32, n: u32) {
    let mut p6 = 0u32;
    let mut iv4 = 0i32;
    if 0 < m.r32(h) as i32 {
        let mut iv2 = 0u32;
        loop {
            let hb = m.r32(h + 4);
            let mut iv5 = m.r32(iv2.wrapping_add(hb));
            iv4 += 1;
            if iv5 != 0 {
                if m.r32(iv5 + 0xc) == 0 {
                    m.w32(iv5 + 0xc, p6);
                } else {
                    iv5 = m.r32(iv5 + 0xc);
                    while m.r32(iv5 + 0xc) != 0 {
                        iv5 = m.r32(iv5 + 0xc);
                    }
                    m.w32(iv5 + 0xc, p6);
                }
                let hb = m.r32(h + 4);
                p6 = m.r32(iv2.wrapping_add(hb));
            }
            iv2 = (iv4 as u32).wrapping_mul(4);
            if !(iv4 < m.r32(h) as i32) {
                break;
            }
        }
    }
    m.w32(h, n);
    let a = m.r32(h + 0xc);
    let old = m.r32(h + 4);
    let u = vcall(m, a, 0, 0x14, &[old, n << 2, 3], &[]).0;
    m.w32(h + 4, u);
    let cnt = m.r32(h) << 2;
    m.call(0x0023_33b4, &[u, 0, cnt], &[]);
    if p6 != 0 {
        let mut iv4 = m.r32(h) as i32;
        loop {
            // trap(7) on division by zero
            assert!(iv4 != 0, "FUN_002568c0: divide by zero");
            let p1 = m.r32(p6 + 0xc);
            let idx = ((m.r32(p6) as i32).wrapping_rem(iv4) as u32).wrapping_mul(4);
            let b = m.r32(h + 4);
            let head = m.r32(idx.wrapping_add(b));
            m.w32(p6 + 0xc, head);
            let b = m.r32(h + 4);
            m.w32(idx.wrapping_add(b), p6);
            if p1 == 0 {
                break;
            }
            iv4 = m.r32(h) as i32;
            p6 = p1;
        }
    }
}
/// FUN_002569f8(h, key, val): insert {hash(key), key, val, next} (node 0x10 through allocator virtual +0xc), count +8
/// += 1, an empty table is first sized 0x35; returns the value slot (node + 8)
pub fn ee_sub_2569f8<M: QMem + ?Sized>(m: &mut M, h: u32, key: u32, val: u32) -> u32 {
    let c = m.r32(h + 8).wrapping_add(1);
    m.w32(h + 8, c);
    if m.r32(h) == 0 {
        ee_sub_2568c0(m, h, 0x35);
    }
    let k = m.r32(key);
    let iv2 = m.call(0x0011_afe0, &[k], &[]).0;
    let iv5 = m.r32(h) as i32;
    assert!(iv5 != 0, "FUN_002569f8: divide by zero");
    let a = m.r32(h + 0xc);
    let node = vcall(m, a, 0, 0xc, &[0x10, 3], &[]).0;
    m.call(0x0023_33b4, &[node, 0, 0x10], &[]);
    m.w32(node, iv2);
    let idx = ((iv2 as i32).wrapping_rem(iv5) as u32).wrapping_mul(4);
    let k = m.r32(key);
    m.w32(node + 4, k);
    let v = m.r32(val);
    m.w32(node + 8, v);
    let b = m.r32(h + 4);
    let head = m.r32(idx.wrapping_add(b));
    m.w32(node + 0xc, head);
    let b = m.r32(h + 4);
    m.w32(idx.wrapping_add(b), node);
    node + 8
}
/// FUN_00256af0(o, p): key = (*(o+0x10))(p), val = (p - [[o+0xc]+0x10]) >> 2; insert into the map at o + 0x14
pub fn ee_sub_256af0<M: QMem + ?Sized>(m: &mut M, o: u32, p: u32) {
    let buf = stack_push(m, 0x10);
    let oc = m.r32(o + 0xc);
    let base = m.r32(oc + 0x10);
    let v = ((p.wrapping_sub(base)) as i32) >> 2;
    m.w32(buf + 4, v as u32);
    let fnp = m.r32(o + 0x10);
    let u = m.call(fnp, &[p], &[]).0;
    m.w32(buf, u);
    ee_sub_2569f8(m, o + 0x14, buf, buf | 4);
    stack_pop(m, 0x10);
}
/// FUN_00256d08(d, s): copy 16 bytes
pub fn ee_sub_256d08<M: QMem + ?Sized>(m: &mut M, d: u32, s: u32) -> u32 {
    // VFIX: lq/sq ignore the low 4 address bits
    let v = m.rv(s & !0xf);
    m.wv(d & !0xf, v);
    d
}
/// FUN_00256d18(d, v): store the quadword
pub fn ee_sub_256d18<M: QMem + ?Sized>(m: &mut M, d: u32, v: V4) {
    m.wv(d & !0xf, v); // VFIX: sq aligns down
}
/// FUN_00256d28(a, b): float a * b (VU qmtc2 / vmulbc / qmfc2 x lane), as bits
pub fn ee_sub_256d28(a: u32, b: u32) -> u32 {
    (f32::from_bits(b) * f32::from_bits(a)).to_bits()
}
/// FUN_00256d40(h, flags): release the pool-0x4520a8 node at [h]; bit 0 -> FUN_001ae520(h)
pub fn ee_sub_256d40<M: QMem + ?Sized>(m: &mut M, h: u32, flags: u32) {
    let p = m.r32(h);
    drop_a(m, p);
    if flags & 1 != 0 {
        m.call(0x001a_e520, &[h], &[]);
    }
}
/// allocation from a typed pool: lazily (re)registers a pool chunk (flag word, chunk counter, chunk array)
fn pool_new<M: QMem + ?Sized>(m: &mut M, flag: u32, cnt: u32, arr: u32, pool: u32) -> u32 {
    if m.r32(flag) == 0 {
        if m.r32(0x003e_0ea4) == 0 || 3 < m.r32(cnt) as i32 {
            m.call(0x0012_86e0, &[GEN], &[]);
        }
        if m.r32(flag) == 0 {
            let u = m.call(0x0011_ccc0, &[pool], &[]).0;
            if m.r32(cnt) != 0 {
                m.call(0x0011_cd98, &[pool, u], &[]);
            }
            let c = m.r32(cnt);
            m.w32(c.wrapping_mul(4).wrapping_add(arr), u);
            m.w32(cnt, c + 1);
        }
    }
    m.call(0x0011_ccc0, &[pool], &[]).0
}
/// FUN_00256dd0: new 0x28-byte object (init FUN_00255718, +0xc..+0x1c = 0)
pub fn ee_sub_256dd0<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let u = pool_new(m, 0x0045_2104, 0x002d_d2a4, 0x0045_20e8, 0x0045_20f8);
    m.call(0x0025_5718, &[u], &[]);
    let mut p = u + 0xc;
    for _ in 0..4 {
        m.w32(p, 0);
        p += 4;
    }
    m.w32(u + 0x1c, 0);
    u
}
/// FUN_00256ee0(h, p): *h = p (refcounted; the old pool-0x4520f8 object is destroyed at 0 through FUN_00257078)
pub fn ee_sub_256ee0<M: QMem + ?Sized>(m: &mut M, h: u32, p: u32) {
    addref(m, p);
    let old = m.r32(h);
    if old == 0 {
        m.w32(h, p);
    } else {
        if dec_zero(m, old) {
            ee_sub_257078(m, old, 3);
        }
        m.w32(h, p);
    }
}
/// FUN_00256f48: new 0x1c-byte object (init FUN_00255a58, +8 / +0x14 / +0x18 = 0)
pub fn ee_sub_256f48<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let u = pool_new(m, 0x0045_212c, 0x002d_d2b4, 0x0045_2110, 0x0045_2120);
    m.call(0x0025_5a58, &[u], &[]);
    m.w32(u + 8, 0);
    m.w32(u + 0x14, 0);
    m.w32(u + 0x18, 0);
    u
}
/// FUN_00257038 / FUN_00257058 / FUN_00257430: `*h = p` with a reference added
pub fn ee_sub_257038<M: QMem + ?Sized>(m: &mut M, h: u32, p: u32) {
    addref(m, p);
    m.w32(h, p);
}
pub fn ee_sub_257058<M: QMem + ?Sized>(m: &mut M, h: u32, p: u32) {
    addref(m, p);
    m.w32(h, p);
}
pub fn ee_sub_257430<M: QMem + ?Sized>(m: &mut M, h: u32, p: u32) {
    addref(m, p);
    m.w32(h, p);
}
/// FUN_00257078(o, flags): destructor of the 0x28 object: release [o+0x1c], the 4 slots +0x18..+0xc, FUN_00255770,
/// free to pool 0x4520f8 on bit 0
pub fn ee_sub_257078<M: QMem + ?Sized>(m: &mut M, o: u32, flags: u32) {
    let p1 = m.r32(o + 0x1c);
    drop_b(m, p1);
    if o as i32 != -0xc {
        let mut iv2 = o.wrapping_add(0x18);
        let mut iv4 = o.wrapping_add(0x1c);
        loop {
            let iv3 = iv2;
            if o.wrapping_add(0xc) == iv4 {
                break;
            }
            ee_sub_2572e0(m, iv3, 0);
            iv2 = iv3.wrapping_sub(4);
            iv4 = iv3;
        }
    }
    m.call(0x0025_5770, &[o], &[]);
    if flags & 1 != 0 {
        m.call(0x0011_cd98, &[0x0045_20f8, o], &[]);
    }
}
/// FUN_00257148(o, flags): destructor of the 0x1c object: release [+0x18] (pool a), [+0x14] (pool a), [+8]
/// (FUN_00257078), FUN_00255ab0, free to pool 0x452120 on bit 0
pub fn ee_sub_257148<M: QMem + ?Sized>(m: &mut M, o: u32, flags: u32) {
    let p = m.r32(o + 0x18);
    drop_a(m, p);
    let p = m.r32(o + 0x14);
    if p != 0 {
        drop_a(m, p);
    }
    let p = m.r32(o + 8);
    if dec_zero(m, p) {
        ee_sub_257078(m, p, 3);
    }
    m.call(0x0025_5ab0, &[o], &[]);
    if flags & 1 != 0 {
        m.call(0x0011_cd98, &[0x0045_2120, o], &[]);
    }
}
/// FUN_00257250(h, flags): release the pool-0x4520d0 node at [h]; bit 0 -> FUN_001ae520(h)
pub fn ee_sub_257250<M: QMem + ?Sized>(m: &mut M, h: u32, flags: u32) {
    let p = m.r32(h);
    drop_b(m, p);
    if flags & 1 != 0 {
        m.call(0x001a_e520, &[h], &[]);
    }
}
/// FUN_002572e0(h, flags): release the 0x28 object at [h] (FUN_00257078); bit 0 -> FUN_001ae520(h)
pub fn ee_sub_2572e0<M: QMem + ?Sized>(m: &mut M, h: u32, flags: u32) {
    let p = m.r32(h);
    if dec_zero(m, p) {
        ee_sub_257078(m, p, 3);
    }
    if flags & 1 != 0 {
        m.call(0x001a_e520, &[h], &[]);
    }
}
/// FUN_00257340: new 0x18-byte object (init FUN_002553e0, +0xc = -1, +0x14 = 0)
pub fn ee_sub_257340<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let u = pool_new(m, 0x0045_20dc, 0x002d_d294, 0x0045_20c0, 0x0045_20d0);
    m.call(0x0025_53e0, &[u], &[]);
    m.w32(u + 0xc, u32::MAX);
    m.w32(u + 0x14, 0);
    u
}
/// FUN_00257450(h, p): *h = p, releasing the old pool-0x4520d0 node
pub fn ee_sub_257450<M: QMem + ?Sized>(m: &mut M, h: u32, p: u32) {
    addref(m, p);
    let old = m.r32(h);
    if old == 0 {
        m.w32(h, p);
    } else {
        drop_b(m, old);
        m.w32(h, p);
    }
}
/// FUN_002574e0 .. FUN_00257508: the originals are a bare `return` (6 empty functions)
pub fn ee_sub_2574e0() {}
pub fn ee_sub_2574e8() {}
pub fn ee_sub_2574f0() {}
pub fn ee_sub_2574f8() {}
pub fn ee_sub_257500() {}
pub fn ee_sub_257508() {}

/// mark / sweep of one pool: relink the live chain [head .. tail] stamping the current generation bit (bit 31 of
/// +4; handle: idx = +4 >> 16 & 0x1fff, chunk = +4 >> 29 & 3, node = chunk_base[chunk] + idx * stride), then free every
/// node of the old chain whose generation bit differs
fn sweep<M: QMem + ?Sized>(m: &mut M, head: u32, tail: u32, arr: u32, stride: u32, rel: u32, pool: u32) {
    let node = |m: &mut M, w: u32| -> u32 {
        let idx = (w >> 16) & 0x1fff;
        let ch = (w >> 29) & 3;
        if idx != 0 || ch != 0 {
            let b = m.r32(arr + ch * 4);
            b.wrapping_add(idx.wrapping_mul(stride))
        } else {
            0
        }
    };
    let iv1 = m.r32(tail);
    let mut iv2 = m.r32(head);
    let g = m.r32(GEN);
    if iv2 != iv1 {
        let mut u3 = m.r32(iv2 + 4);
        loop {
            if u3 >> 31 != g {
                let w = (u3 & 0x7fff_ffff) | (g << 31);
                m.w32(iv2 + 4, w);
            }
            let w = m.r32(iv2 + 4);
            iv2 = node(m, w);
            if iv2 == iv1 {
                break;
            }
            u3 = m.r32(iv2 + 4);
        }
    }
    loop {
        let i1 = iv2;
        if i1 == 0 {
            break;
        }
        let w = m.r32(i1 + 4);
        iv2 = node(m, w);
        let g = m.r32(GEN);
        if m.r32(i1 + 4) >> 31 != g {
            m.call(rel, &[i1], &[]);
            m.call(0x0011_cd98, &[pool, i1], &[]);
        }
    }
}
pub fn ee_sub_257510<M: QMem + ?Sized>(m: &mut M) {
    sweep(m, 0x002d_d2a0, 0x002d_d2a8, 0x0045_20e8, 0x28, 0x0025_5770, 0x0045_20f8);
}
pub fn ee_sub_257678<M: QMem + ?Sized>(m: &mut M) {
    sweep(m, 0x002d_d290, 0x002d_d298, 0x0045_20c0, 0x18, 0x0025_5438, 0x0045_20d0);
}
pub fn ee_sub_2577e0<M: QMem + ?Sized>(m: &mut M) {
    sweep(m, 0x002d_d280, 0x002d_d288, 0x0045_2098, 0x14, 0x0025_50f8, 0x0045_20a8);
}
pub fn ee_sub_257948<M: QMem + ?Sized>(m: &mut M) {
    sweep(m, 0x002d_d2b0, 0x002d_d2b8, 0x0045_2110, 0x1c, 0x0025_5ab0, 0x0045_2120);
}
pub fn ee_sub_257ab0<M: QMem + ?Sized>(m: &mut M) {
    sweep(m, 0x002d_d2c0, 0x002d_d2c8, 0x0045_2138, 0xc, 0x0025_5de8, 0x0045_2148);
}
pub fn ee_sub_257c18<M: QMem + ?Sized>(m: &mut M) {
    sweep(m, 0x002d_d2d0, 0x002d_d2d8, 0x0045_2160, 0x14, 0x0025_6120, 0x0045_2170);
}
/// FUN_00257d80: new 0xc-byte object (init FUN_00255d90, +8 = 0)
pub fn ee_sub_257d80<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let u = pool_new(m, 0x0045_2154, 0x002d_d2c4, 0x0045_2138, 0x0045_2148);
    m.call(0x0025_5d90, &[u], &[]);
    m.w32(u + 8, 0);
    u
}
/// pool reset: head / tail / count words = 0 (base +0, +8, +0xc, +4), FUN_0011cdb8(pool)
fn reset<M: QMem + ?Sized>(m: &mut M, base: u32, pool: u32) {
    m.w32(base, 0);
    m.w32(base + 8, 0);
    m.w32(base + 0xc, 0);
    m.w32(base + 4, 0);
    m.call(0x0011_cdb8, &[pool], &[]);
}
pub fn ee_sub_257e88<M: QMem + ?Sized>(m: &mut M) {
    reset(m, 0x002d_d280, 0x0045_20a8);
}
pub fn ee_sub_257ec8<M: QMem + ?Sized>(m: &mut M) {
    reset(m, 0x002d_d2a0, 0x0045_20f8);
}
pub fn ee_sub_257f08<M: QMem + ?Sized>(m: &mut M) {
    reset(m, 0x002d_d290, 0x0045_20d0);
}
pub fn ee_sub_257f48<M: QMem + ?Sized>(m: &mut M) {
    reset(m, 0x002d_d2b0, 0x0045_2120);
}
pub fn ee_sub_257f88<M: QMem + ?Sized>(m: &mut M) {
    reset(m, 0x002d_d2c0, 0x0045_2148);
}
pub fn ee_sub_257fc8<M: QMem + ?Sized>(m: &mut M) {
    reset(m, 0x002d_d2d0, 0x0045_2170);
}
/// FUN_002580a8(h, p): *h = p, releasing the old 0xc-byte object (+8 = pool-0x4520d0 node) at 0
pub fn ee_sub_2580a8<M: QMem + ?Sized>(m: &mut M, h: u32, p: u32) {
    addref(m, p);
    let old = m.r32(h);
    if old == 0 {
        m.w32(h, p);
    } else {
        if dec_zero(m, old) {
            let p3 = m.r32(old + 8);
            drop_b(m, p3);
            m.call(0x0025_5de8, &[old], &[]);
            m.call(0x0011_cd98, &[0x0045_2148, old], &[]);
        }
        m.w32(h, p);
    }
}
/// FUN_00258178(o): FUN_00255de8(o); FUN_00255c00(o, 0x2dd2cc); +4 |= 0x8000
pub fn ee_sub_258178<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0025_5de8, &[o], &[]);
    m.call(0x0025_5c00, &[o, 0x002d_d2cc], &[]);
    let w = m.r32(o + 4) | 0x8000;
    m.w32(o + 4, w);
}
/// FUN_00258200(o, a, b, c): FUN_001d80c8(o, 0x3e0ea8, a, b, c); o
pub fn ee_sub_258200<M: QMem + ?Sized>(m: &mut M, o: u32, a: u32, b: u32, c: u32) -> u32 {
    m.call(0x001d_80c8, &[o, 0x003e_0ea8, a, b, c], &[]);
    o
}
/// FUN_00258290(o, flags): release (virtual +0x14 on the +0xc vtable) the objects +0x5e4, +0x5e0, +0x488, +0x484,
/// then FUN_00258348(o, flags)
pub fn ee_sub_258290<M: QMem + ?Sized>(m: &mut M, o: u32, flags: u32) {
    for off in [0x5e4, 0x5e0, 0x488, 0x484] {
        let p = m.r32(o + off);
        if p != 0 {
            vcall(m, p, 0xc, 0x14, &[], &[]);
        }
    }
    ee_sub_258348(m, o, flags);
}
/// FUN_00258348(o, flags): release [o+4] (0xc-byte object, FUN_00255de8 + its +8 node) and [o] (FUN_00257078)
pub fn ee_sub_258348<M: QMem + ?Sized>(m: &mut M, o: u32, flags: u32) {
    let p1 = m.r32(o + 4);
    if dec_zero(m, p1) {
        let p3 = m.r32(p1 + 8);
        drop_b(m, p3);
        m.call(0x0025_5de8, &[p1], &[]);
        m.call(0x0011_cd98, &[0x0045_2148, p1], &[]);
    }
    let p1 = m.r32(o);
    if dec_zero(m, p1) {
        ee_sub_257078(m, p1, 3);
    }
    if flags & 1 != 0 {
        m.call(0x001a_e520, &[o], &[]);
    }
}
/// FUN_00258438(p, a, b): [p+4] = b; [p] = a
/// VFIX: ABI per asm: p = a0, a = $f12, b = $f13 (swc1)
pub fn ee_sub_258438<M: QMem + ?Sized>(m: &mut M, p: u32, a: f32, b: f32) {
    m.wf(p + 4, b);
    m.wf(p, a);
}
/// FUN_00258448(p, a, b): [p+0xc] = b; [p+8] = a
/// VFIX: ABI per asm: p = a0, a = $f12, b = $f13 (swc1)
pub fn ee_sub_258448<M: QMem + ?Sized>(m: &mut M, p: u32, a: f32, b: f32) {
    m.wf(p + 0xc, b);
    m.wf(p + 8, a);
}
/// FUN_00258458(o, p): [o+0x10] = [p]
pub fn ee_sub_258458<M: QMem + ?Sized>(m: &mut M, o: u32, p: u32) {
    let v = m.r32(p);
    m.w32(o + 0x10, v);
}
pub fn ee_sub_258470<M: QMem + ?Sized>(m: &mut M, o: u32, v: u32) {
    m.w32(o + 0x14, v);
}
pub fn ee_sub_258478<M: QMem + ?Sized>(m: &mut M, o: u32, v: u32) {
    m.w32(o + 0x18, v);
}
