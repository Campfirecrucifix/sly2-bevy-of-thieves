//! WP-Q8: C2ca5e0 (dynamic array {data, count, capacity, elem size, _, allocator} + child list) slot overrides.
use crate::engine::classes::q::q2mem::QMem;
use crate::engine::classes::q::q8util::*;

/// FUN_00116d28: free the data through the allocator (vslot 0x1c), zero data / count / capacity
pub fn ee_sub_116d28<M: QMem + ?Sized>(m: &mut M, p: u32) {
    let al = m.r32(p + 0x14);
    let d = m.r32(p);
    vcall(m, al, 0, 0x1c, &[d], &[]);
    m.w32(p + 4, 0);
    m.w32(p, 0);
    m.w32(p + 8, 0);
}
/// FUN_00116d70: grow capacity to >= n (signed): < 0x200 doubles from max(cap, 8), else rounds up to 1024
pub fn ee_sub_116d70<M: QMem + ?Sized>(m: &mut M, p: u32, n: u32) {
    let cap = m.r32(p + 8);
    if (n as i32) <= cap as i32 {
        return;
    }
    let al = m.r32(p + 0x14);
    let esz = m.r32(p + 0xc);
    let mut u6: u32;
    if (n as i32) < 0x200 {
        u6 = if cap != 0 { cap } else { 8 };
        while (u6 as i32) < n as i32 {
            u6 <<= 1;
        }
    } else {
        u6 = n.wrapping_add(0x3ff) & 0xffff_fc00;
    }
    let d = m.r32(p);
    let r = vcall(m, al, 0, 0x14, &[d, u6.wrapping_mul(esz), 4], &[]).0;
    m.w32(p + 8, u6);
    m.w32(p, r);
}
/// FUN_00116e30: append n elements copied from src (FUN_0023320c memcpy)
pub fn ee_sub_116e30<M: QMem + ?Sized>(m: &mut M, p: u32, src: u32, n: u32) {
    let cnt = m.r32(p + 4);
    ee_sub_116d70(m, p, cnt.wrapping_add(n));
    let d = m.r32(p);
    let esz = m.r32(p + 0xc);
    let cnt = m.r32(p + 4);
    m.call(0x0023_320c, &[d.wrapping_add(esz.wrapping_mul(cnt)), src, esz.wrapping_mul(n)], &[]);
    let cnt = m.r32(p + 4);
    m.w32(p + 4, cnt.wrapping_add(n));
}
/// FUN_00119bd0 (slot 0): FUN_002564e0(this, src) then FUN_00256528(this + 0x34, this, 0x119bc8)
pub fn ee_sub_119bd0<M: QMem + ?Sized>(m: &mut M, o: u32, src: u32) {
    m.call(0x0025_64e0, &[o, src], &[]);
    ee_sub_256528(m, o + 0x34, o, 0x0011_9bc8);
}
/// FUN_00119c08: FUN_00118380(p3, p2, **p1, 7)
pub fn ee_sub_119c08<M: QMem + ?Sized>(m: &mut M, p1: u32, p2: u32, p3: u32) {
    let a = m.r32(p1);
    let v = m.r32(a);
    m.call(0x0011_8380, &[p3, p2, v, 7], &[]);
}
/// FUN_00256528: link record: [4] = fn, [0] = *owner, [0xc] = owner, FUN_0011a070(owner + 0x28, rec), [0x20] = **rec
pub fn ee_sub_256528<M: QMem + ?Sized>(m: &mut M, r: u32, owner: u32, f: u32) {
    let u1 = m.r32(owner);
    m.w32(r + 0x10, f);
    m.w32(r, u1);
    m.w32(r + 0xc, owner);
    m.call(0x0011_a070, &[owner + 0x28, r], &[]);
    let a = m.r32(r);
    let v = m.r32(a);
    m.w32(r + 0x20, v);
}
/// FUN_00256588: for each node of the list at +0x28 (next at +4, vtable at +8): vslot 0x14(node, a)
pub fn ee_sub_256588<M: QMem + ?Sized>(m: &mut M, o: u32, a: u32) {
    let mut n = m.r32(o + 0x28);
    while n != 0 {
        vcall(m, n, 8, 0x14, &[a], &[]);
        n = m.r32(n + 4);
    }
}
/// FUN_002565e0: append `src` (1 element) at +0x10, vslot 0x34(this, &last); returns &last
pub fn ee_sub_2565e0<M: QMem + ?Sized>(m: &mut M, o: u32, src: u32) -> u32 {
    ee_sub_116e30(m, o + 0x10, src, 1);
    let last = |m: &mut M| m.r32(o + 0x10).wrapping_add(m.r32(o + 0x14).wrapping_mul(4)).wrapping_sub(4);
    let l = last(m);
    vcall(m, o, 0xc, 0x34, &[l], &[]);
    last(m)
}
/// FUN_00256658: FUN_00116d28(this + 0x10), then vslot 0xc of every node of the list at +0x28
pub fn ee_sub_256658<M: QMem + ?Sized>(m: &mut M, o: u32) {
    ee_sub_116d28(m, o + 0x10);
    let mut n = m.r32(o + 0x28);
    while n != 0 {
        vcall(m, n, 8, 0xc, &[], &[]);
        n = m.r32(n + 4);
    }
}
/// FUN_002566d8: load: vslot 0x14, frame, temp (4,16), count = FUN_00117db8(stream); resize +0x10; per element:
/// memset(tmp, 0, 4), vslot 0x24(tmp, stream), vslot 0x2c(tmp)
pub fn ee_sub_2566d8<M: QMem + ?Sized>(m: &mut M, o: u32, stream: u32) {
    vcall(m, o, 0xc, 0x14, &[], &[]);
    let fr = frame_begin(m);
    let tmp = frame_alloc(m, 4, 0x10);
    let mut n = m.call(0x0011_7db8, &[stream], &[]).0 as i32;
    ee_sub_116d70(m, o + 0x10, n as u32);
    if 0 < n {
        loop {
            n -= 1;
            m.call(0x0023_33b4, &[tmp, 0, 4], &[]);
            vcall(m, o, 0xc, 0x24, &[tmp, stream], &[]);
            vcall(m, o, 0xc, 0x2c, &[tmp], &[]);
            if n == 0 {
                break;
            }
        }
    }
    frame_end(m, fr, 2);
}
