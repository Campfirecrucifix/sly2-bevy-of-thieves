//! WP-Q14 part A: class C2c94e0 (LO base) queue package. 1:1 from ghidra_functions(_v2).body on `QMem`
//! (callees outside the package by address through `QMem::call`; ints in `a`, floats / quadwords in `v`).
//! Layout: +0x4 handle, +0xc vtable, +0x10 parent / sibling, +0x14 id, +0x28 u64 flags (bit 0x3c = refresh),
//! +0x2d byte, +0x38 float, +0x40 tagged flag word (bit 0 clear = pointer to the flags), +0x50 body copy.
#![allow(unused_variables, unused_mut, dead_code)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem, V4};
use crate::engine::classes::q::q6::{rov, vcall, WORLD};

/// `if p != 0 && --*p == 0`
fn dec_zero<M: QMem + ?Sized>(m: &mut M, p: u32) -> bool {
    if p != 0 {
        let c = m.r32(p).wrapping_sub(1);
        m.w32(p, c);
        return c == 0;
    }
    false
}
/// release of a pool-0x4520d0 node (FUN_00257250 / Release / FUN_00255438 / FUN_0011cd98)
fn drop_b<M: QMem + ?Sized>(m: &mut M, p: u32) {
    m.call(0x0025_7250, &[p + 0x14, 2], &[]);
    m.call(0x0012_9ec8, &[p + 0xc], &[]);
    m.call(0x0025_5438, &[p], &[]);
    m.call(0x0011_cd98, &[0x0045_20d0, p], &[]);
}
/// FUN_00257e68(out, p): `if p != 0 { ++*p }; *out = p`
pub fn ee_sub_257e68<M: QMem + ?Sized>(m: &mut M, out: u32, p: u32) {
    if p != 0 {
        let c = m.r32(p).wrapping_add(1);
        m.w32(p, c);
    }
    m.w32(out, p);
}
/// FUN_00210268(world, o): when +0x40 holds a pointer (not inline flags) the pointee is moved back into +0x40,
/// its +4 / +0 cleared and world+0x3f18 incremented
pub fn ee_sub_210268<M: QMem + ?Sized>(m: &mut M, w: u32, o: u32) {
    let p = m.r32(o + 0x40);
    if (p ^ 1) & 1 != 0 {
        let v = m.r32(p);
        m.w32(o + 0x40, v);
        m.w32(p + 4, 0);
        m.w32(p, 0);
        let c = m.r32(w + 0x3f18).wrapping_add(1);
        m.w32(w + 0x3f18, c);
    }
}

/// FUN_00114170 (slot 5): class descriptor
pub fn c2c94e0_v5() -> u32 {
    0x002b_b420
}
/// FUN_0012ab78(out, src, arg): handle copy (pool 0x4520d0 node refs)
pub fn ee_sub_12ab78<M: QMem + ?Sized>(m: &mut M, out: u32, src: u32, arg: u32) -> u32 {
    let s = stack_push(m, 0x40);
    let u = m.call(0x0025_7d80, &[], &[]).0;
    ee_sub_257e68(m, s, u);
    let p2 = m.r32(s);
    if m.r32(src + 8) != 0 {
        let a = m.r32(src + 8);
        m.call(0x0012_b0e8, &[s + 0x10, a, 0, arg], &[]);
        let n = m.r32(s + 0x10);
        m.call(0x0025_7450, &[p2 + 8, n], &[]);
        if dec_zero(m, n) {
            drop_b(m, n);
        }
    }
    let p = m.r32(s);
    ee_sub_257e68(m, out, p);
    let p = m.r32(s);
    if dec_zero(m, p) {
        let q = m.r32(p + 8);
        if dec_zero(m, q) {
            drop_b(m, q);
        }
        m.call(0x0025_5de8, &[p], &[]);
        m.call(0x0011_cd98, &[0x0045_2148, p], &[]);
    }
    stack_pop(m, 0x40);
    out
}
/// FUN_001a5b80 (slot 12)
pub fn c2c94e0_v12<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32, p3: u32) {
    let p = m.r32(o + 0x10);
    if p == 0 {
        if p3 != 0 {
            m.w32(o + 0x14, 0);
            m.call(0x001a_53a0, &[o], &[]);
        }
    } else {
        let u;
        if p2 == o {
            if p3 == 0 {
                m.call(0x001a_53e0, &[], &[]);
                let q = m.r32(o + 0x10);
                u = m.r32(q + 0x14);
            } else {
                u = m.r32(p + 0x14);
            }
        } else {
            u = m.r32(p + 0x14);
        }
        m.w32(o + 0x14, u);
    }
}
/// slots 18, 19, 28, 29, 34, 35, 26, 17: empty
pub fn c2c94e0_nop() {}
/// FUN_001a6120 / FUN_001a6138 (slots 32 / 33): `_DAT_002a2dd0`
pub fn c2c94e0_v32() -> V4 {
    rov(0x002a_2dd0)
}
pub fn c2c94e0_v33() -> V4 {
    rov(0x002a_2dd0)
}
/// FUN_001a6150 (slot 21)
pub fn c2c94e0_v21<M: QMem + ?Sized>(m: &mut M, o: u32) {
    ee_sub_210268(m, WORLD, o);
}
/// FUN_001a6178 (slot 22)
pub fn c2c94e0_v22<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0021_0140, &[WORLD, o], &[]);
}
/// FUN_001a62d8 (slot 30): &DAT_002a2d30
pub fn c2c94e0_v30() -> u32 {
    0x002a_2d30
}
/// FUN_001a6450 (slot 36): slot 0x38 (+0xe4 / adj +0xe0) fills a pointer array, then slot +0x12c of each
pub fn c2c94e0_v36<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32) {
    let buf = stack_push(m, 0x30);
    let n = vcall(m, o, 0xc, 0xe4, &[buf], &[]).0 as i32;
    let mut i = 0i32;
    if 0 < n {
        loop {
            let a = (i as u32).wrapping_mul(4);
            i += 1;
            let b = m.r32(buf);
            let c = m.r32(a.wrapping_add(b));
            vcall(m, c, 0xc, 0x12c, &[p2], &[]);
            if i >= n {
                break;
            }
        }
    }
    stack_pop(m, 0x30);
}
/// FUN_001a66c8 (slot 46): flags |= bits (+0x40 tagged)
pub fn c2c94e0_v46<M: QMem + ?Sized>(m: &mut M, o: u32, bits: u32) {
    let p = m.r32(o + 0x40);
    if (p ^ 1) & 1 != 0 {
        let v = m.r32(p);
        m.w32(p, v | bits);
        return;
    }
    m.w32(o + 0x40, p | bits);
}
/// FUN_001a66f8 (slot 47): flags &= ~bits
pub fn c2c94e0_v47<M: QMem + ?Sized>(m: &mut M, o: u32, bits: u32) {
    let p = m.r32(o + 0x40);
    if (p ^ 1) & 1 != 0 {
        let v = m.r32(p);
        m.w32(p, v & !bits);
        return;
    }
    m.w32(o + 0x40, p & !bits);
}
/// FUN_001a6730 (slot 40): copy-assign `o` <- `src` (handles, flag bitfield of +0x28, +0x40 flags, children)
pub fn c2c94e0_v40<M: QMem + ?Sized>(m: &mut M, o: u32, src: u32) {
    let s = stack_push(m, 0x40);
    m.call(0x0012_8148, &[0x002d_d598], &[]);
    let mut iv4;
    if m.r32(src + 4) == 0 {
        iv4 = m.r32(src);
    } else {
        let a = m.r32(src + 4);
        ee_sub_12ab78(m, s, a, o);
        let h = m.r32(s);
        m.call(0x0025_80a8, &[o + 4, h], &[]);
        if dec_zero(m, h) {
            let p1 = m.r32(h + 8);
            if dec_zero(m, p1) {
                drop_b(m, p1);
            }
            m.call(0x0025_5de8, &[h], &[]);
            m.call(0x0011_cd98, &[0x0045_2148, h], &[]);
        }
        let x = m.r32(o + 4);
        m.call(0x0025_8178, &[x], &[]);
        iv4 = m.r32(src);
    }
    if iv4 != 0 {
        m.call(0x0012_7e90, &[s, iv4, o], &[]);
        let h = m.r32(s);
        m.call(0x0025_6ee0, &[o, h], &[]);
        if dec_zero(m, h) {
            m.call(0x0025_7078, &[h, 3], &[]);
        }
        let x = m.r32(o);
        m.call(0x0025_9a88, &[x], &[]);
    }
    // +0x28: bits 48..55, 57 and 58 are copied from src
    let old = m.r64(o + 0x28);
    let v38 = m.r32(src + 0x38);
    m.w32(o + 0x38, v38);
    let sv = m.r64(src + 0x28);
    let u5 = (((sv << 8) as i64 >> 0x3c) as u64 & 0xf) << 0x34;
    let u6 = ((sv >> 0x39) & 1) << 0x39;
    let u7 = (((sv << 0xc) as i64 >> 0x3c) as u64 & 0xf) << 0x30;
    let nv = old & 0xf900_ffff_ffff_ffff | u5 | u6 | u7 | ((sv >> 0x3a) & 1) << 0x3a;
    m.w32(o + 0x28, nv as u32);
    m.w32(o + 0x2c, (nv >> 32) as u32);
    // +0x40 tagged flag word: low bit of the destination kept, rest copied
    let mut pv = m.r32(src + 0x40);
    if (pv ^ 1) & 1 != 0 {
        pv = m.r32(pv);
    }
    let pd = m.r32(o + 0x40);
    if (pd ^ 1) & 1 == 0 {
        m.w32(o + 0x40, pv & 0xffff_fffe | pd & 1);
    } else {
        let v = m.r32(pd);
        m.w32(pd, pv & 0xffff_fffe | v & 1);
    }
    let v = m.r32(src + 0x34);
    m.w32(o + 0x34, v);
    if m.r32(0x003e_13a0) == 0xffff_fffe {
        let mut iv4 = m.r32(src + 0x3c);
        if iv4 != 0 && m.r32(iv4 + 0x10) == src {
            loop {
                m.w32(s + 0x10, 0x28);
                m.w32(s + 0x14, 0x10);
                let u8_ = m.call(0x001a_e238, &[s + 0x10, iv4], &[]).0;
                m.w32(u8_ + 0x10, o);
                m.call(0x0011_a058, &[u8_ + 0xc], &[]);
                m.call(0x0011_a058, &[u8_ + 0x14], &[]);
                m.call(0x0011_a060, &[u8_ + 0x18], &[]);
                let q = m.r32(u8_ + 4);
                m.call(0x0011_a070, &[q + 0x18, u8_], &[]);
                let k = m.r32(u8_ + 8);
                let u9 = m.call(0x0015_e3c0, &[WORLD, k], &[]).0;
                m.call(0x0011_a070, &[u9, u8_], &[]);
                iv4 = m.r32(iv4 + 4);
                if iv4 == 0 || m.r32(iv4 + 0x10) != src {
                    break;
                }
            }
        }
    } else {
        let v = m.r32(src + 0x3c);
        m.w32(o + 0x3c, v);
    }
    let r = vcall(m, o, 0xc, 0x34, &[], &[]).0;
    let sz = m.r32(r + 0xc).wrapping_sub(0x50);
    m.call(0x001a_e5a0, &[o + 0x50, src + 0x50, sz], &[]);
    stack_pop(m, 0x40);
}
/// FUN_001a6ad0 (slot 39): slot at +0x14c (adj +0x148)
pub fn c2c94e0_v39<M: QMem + ?Sized>(m: &mut M, o: u32) {
    vcall(m, o, 0xc, 0x14c, &[], &[]);
}
/// FUN_001a6b30 (slot 31): identity matrix into `p2` (when non-null); returns the matrix class descriptor
pub fn c2c94e0_v31<M: QMem + ?Sized>(m: &mut M, p2: u32) -> u32 {
    if p2 != 0 {
        m.call(0x0023_33b4, &[p2, 0, 0x40], &[]);
        m.w32(p2, 0x3f80_0000);
        m.w32(p2 + 0x3c, 0x3f80_0000);
        m.w32(p2 + 0x28, 0x3f80_0000);
        m.w32(p2 + 0x14, 0x3f80_0000);
    }
    0x002c_44f0
}
/// FUN_001a6b88 (slot 37): FUN_001a6be8(this, other.slot(+0xfc / adj +0xf8)())
pub fn c2c94e0_v37<M: QMem + ?Sized>(m: &mut M, o: u32, other: u32) {
    let u = vcall(m, other, 0xc, 0xfc, &[], &[]).0;
    m.call(0x001a_6be8, &[o, u], &[]);
}
/// FUN_001a6bc8 (slot 38)
pub fn c2c94e0_v38<M: QMem + ?Sized>(m: &mut M, o: u32, other: u32) {
    m.call(0x001a_6be8, &[o, other + 0x40], &[]);
}
/// FUN_001a6c48 (slot 41): +0x38 = x; for a root (+0x10 == 0) with the refresh bit (+0x28 bit 0x3c) FUN_00147e88(world, +0x14)
pub fn c2c94e0_v41<M: QMem + ?Sized>(m: &mut M, o: u32, x: f32) {
    m.wf(o + 0x38, x);
    if m.r32(o + 0x10) == 0 && (m.r64(o + 0x28) >> 0x3c) & 1 != 0 {
        let id = m.r32(o + 0x14);
        m.call(0x0014_7e88, &[WORLD, id], &[]);
    }
}
/// FUN_001a6dc0 (slot 45): byte +0x2d
pub fn c2c94e0_v45<M: QMem + ?Sized>(m: &mut M, o: u32, b: u8) {
    m.w8(o + 0x2d, b);
}
/// FUN_001a6e80 (slot 20): flag 8 set / cleared
pub fn c2c94e0_v20<M: QMem + ?Sized>(m: &mut M, o: u32, on: u64) {
    if on != 0 {
        c2c94e0_v46(m, o, 8);
    } else {
        c2c94e0_v47(m, o, 8);
    }
}
/// FUN_001cb4e0 (slot 25): unless the 0x0400_0000_0000_0000 bit of +0x28 is set, FUN_001a6d28(o, other.byte(+0x2d))
pub fn c2c94e0_v25<M: QMem + ?Sized>(m: &mut M, o: u32, other: u32) {
    if other != 0 && m.r64(o + 0x28) & 0x0400_0000_0000_0000 == 0 {
        let b = m.r8(other + 0x2d) as u32;
        m.call(0x001a_6d28, &[o, b], &[]);
    }
}
