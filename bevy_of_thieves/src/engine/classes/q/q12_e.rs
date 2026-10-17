//! WP-Q12 part E: file package run_00207a20 (AsegaHost / actor "anim slot" tables: 26-slot groups of
//! {kind, object, host, extra} records at +0x8d0 / +0xa70 / +0xc10, entries at +0x4d0, mask helpers).
//! 1:1 from ghidra_functions(_v2).body on `QMem` (conventions as q12_a). Mask values are u64 (the original
//! `1 << n` on 64-bit registers); calls inside the package run the Rust fns directly, all others by address.
#![allow(unused_variables, unused_mut, dead_code)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem, V4};
use crate::engine::classes::q::q6::{rof, vcall};
use crate::engine::classes::q::q7util::{f, CLOCK};

const NEG1: u32 = u32::MAX;
const WORLD: u32 = 0x003d_d490;

fn w64<M: QMem + ?Sized>(m: &mut M, a: u32, v: u64) {
    m.w32(a, v as u32);
    m.w32(a + 4, (v >> 32) as u32);
}
fn release<M: QMem + ?Sized>(m: &mut M, a: u32) {
    m.call(0x0012_9ec8, &[a], &[]);
}

/// FUN_00207a20: `1 << n`, 0 for -1
pub fn ee_sub_207a20(p: i64) -> u64 {
    if p != -1 {
        1u64.wrapping_shl(p as u32 & 63)
    } else {
        0
    }
}
/// FUN_00207a40(n, ids): or of the bits of n ids
pub fn ee_sub_207a40<M: QMem + ?Sized>(m: &mut M, n: i32, p: u32) -> u64 {
    let mut r = 0;
    let mut n = n;
    let mut p = p;
    if 0 < n {
        loop {
            let id = m.r32(p) as i32 as i64;
            n -= 1;
            p += 4;
            r |= ee_sub_207a20(id);
            if n == 0 {
                break;
            }
        }
    }
    r
}
/// FUN_00207aa0(mask, out): ids 0x2e..=0 whose bit is set, descending; count
pub fn ee_sub_207aa0<M: QMem + ?Sized>(m: &mut M, mask: u64, out: u32) -> i32 {
    let mut i = 0x2e;
    let mut n = 0;
    let mut out = out;
    loop {
        if mask & ee_sub_207a20(i) != 0 {
            m.w32(out, i as u32);
            n += 1;
            out += 4;
        }
        i -= 1;
        if !(-1 < i) {
            break;
        }
    }
    n
}
/// FUN_00207b10(p): mask of the ids of the Splice list p (FUN_002013d8 type 0x1010, 0x10 ids max)
pub fn ee_sub_207b10<M: QMem + ?Sized>(m: &mut M, p: u32) -> u64 {
    let b = stack_push(m, 0x90);
    let (a90, a50, a40) = (b, b + 0x40, b + 0x50);
    m.call(0x0020_13d8, &[a50, p, 0x1010, 0x10, a40, a90], &[]);
    let n = m.r32(a40) as i32;
    let r = ee_sub_207a40(m, n, a90);
    release(m, a50);
    stack_pop(m, 0x90);
    r
}
/// FUN_00207b70(o, list, v4, v12): record of the mask: +0xc = v12, +4 = v4
pub fn ee_sub_207b70<M: QMem + ?Sized>(m: &mut M, o: u32, l: u32, p3: u32, p4: u32) {
    let u = ee_sub_207b10(m, l);
    let r = ee_sub_209c18(m, o, u);
    m.w32(r + 0xc, p4);
    m.w32(r + 4, p3);
}
/// RECORDED_207bc8: find record, set it (FUN_00207b70), FUN_00209130(record, o)
pub fn ee_sub_207bc8<M: QMem + ?Sized>(m: &mut M, o: u32, l: u32, p3: u32, p4: u32) {
    let u = ee_sub_207b10(m, l);
    let r = ee_sub_209c18(m, o, u);
    ee_sub_207b70(m, o, l, p3, p4);
    ee_sub_209130(m, r, o);
}
/// FUN_00207c48: record kind 3 with a C2da9d0 object (0xd0 bytes): +0, +0x30, +0x60, +0x90 = p3..p6; [3] = 8
pub fn ee_sub_207c48<M: QMem + ?Sized>(m: &mut M, o: u32, l: u32, p3: u32, p4: u32, p5: u32, p6: u32) {
    let u = ee_sub_207b10(m, l);
    let r = ee_sub_209c18(m, o, u);
    m.w32(r, 3);
    let obj = m.call(0x0020_ca00, &[0xd0], &[]).0;
    m.w32(r + 4, obj);
    m.call(0x0020_c7d8, &[obj], &[]);
    m.w32(obj, p3);
    m.w32(obj + 0x30, p4);
    m.w32(obj + 0x60, p5);
    m.w32(obj + 0x90, p6);
    m.w32(r + 0xc, 8);
}
pub fn ee_sub_207cf8<M: QMem + ?Sized>(m: &mut M, o: u32, l: u32, p3: u32, p4: u32, p5: u32, p6: u32) {
    let u = ee_sub_207b10(m, l);
    let r = ee_sub_209c18(m, o, u);
    ee_sub_207c48(m, o, l, p3, p4, p5, p6);
    ee_sub_209130(m, r, o);
}
/// FUN_00207d98: record kind 2 with a 0x40-byte object ([0] = p3); [3] = 0
pub fn ee_sub_207d98<M: QMem + ?Sized>(m: &mut M, o: u32, l: u32, p3: u32) {
    let u = ee_sub_207b10(m, l);
    let r = ee_sub_209c18(m, o, u);
    m.w32(r, 2);
    let obj = m.call(0x0020_c010, &[0x40], &[]).0;
    m.w32(r + 4, obj);
    m.call(0x0020_bd78, &[obj], &[]);
    m.w32(obj, p3);
    m.w32(r + 0xc, 0);
}
/// FUN_00207e08(p1..p7): record of (p5, mask(p6)); FUN_00207d98 gets the unchanged first args; FUN_00209130(rec, p5)
pub fn ee_sub_207e08<M: QMem + ?Sized>(m: &mut M, p1: u32, p2: u32, p3: u32, p4: u32, p5: u32, p6: u32, p7: u32) {
    let u = ee_sub_207b10(m, p6);
    let r = ee_sub_209c18(m, p5, u);
    ee_sub_207d98(m, p1, p2, p3);
    ee_sub_209130(m, r, p5);
}
/// FUN_00207eb8(o, mask, v4, v12, slots): reset the selected of 26 slots (stride 0x10) and set +4 / +0xc
pub fn ee_sub_207eb8<M: QMem + ?Sized>(m: &mut M, o: u32, mask: u32, p3: u32, p4: u32, slots: u32) {
    let mut i = 0x19;
    let mut b = 1u32;
    let mut s = slots;
    loop {
        let hit = mask & b;
        i -= 1;
        b <<= 1;
        if hit != 0 {
            ee_sub_2090d0(m, s);
            m.w32(s + 4, p3);
            m.w32(s + 0xc, p4);
        }
        s += 0x10;
        if !(-1 < i) {
            break;
        }
    }
}
/// FUN_00207f50: as FUN_00207eb8 plus FUN_00209130(slot, o)
pub fn ee_sub_207f50<M: QMem + ?Sized>(m: &mut M, o: u32, mask: u32, p3: u32, p4: u32, slots: u32) {
    let mut i = 0x19;
    let mut b = 1u32;
    let mut s = slots;
    loop {
        let hit = mask & b;
        i -= 1;
        b <<= 1;
        if hit != 0 {
            ee_sub_2090d0(m, s);
            m.w32(s + 4, p3);
            m.w32(s + 0xc, p4);
            ee_sub_209130(m, s, o);
        }
        s += 0x10;
        if !(-1 < i) {
            break;
        }
    }
}
/// FUN_00207ff8 / 208038 / 208078: FUN_00207eb8 on the slot group +0xc10 / +0xa70 / +0x8d0
pub fn ee_sub_207ff8<M: QMem + ?Sized>(m: &mut M, o: u32, a: u32, b: u32, c: u32) {
    ee_sub_207eb8(m, o, a, b, c, o + 0xc10);
}
pub fn ee_sub_208038<M: QMem + ?Sized>(m: &mut M, o: u32, a: u32, b: u32, c: u32) {
    ee_sub_207eb8(m, o, a, b, c, o + 0xa70);
}
pub fn ee_sub_208078<M: QMem + ?Sized>(m: &mut M, o: u32, a: u32, b: u32, c: u32) {
    ee_sub_207eb8(m, o, a, b, c, o + 0x8d0);
}
/// FUN_00208018 / 208058: FUN_00207f50 on +0xc10 / +0xa70
pub fn ee_sub_208018<M: QMem + ?Sized>(m: &mut M, o: u32, a: u32, b: u32, c: u32) {
    ee_sub_207f50(m, o, a, b, c, o + 0xc10);
}
pub fn ee_sub_208058<M: QMem + ?Sized>(m: &mut M, o: u32, a: u32, b: u32, c: u32) {
    ee_sub_207f50(m, o, a, b, c, o + 0xa70);
}
/// FUN_002080a0(o, slots): FUN_00209130 on 26 slots
pub fn ee_sub_2080a0<M: QMem + ?Sized>(m: &mut M, o: u32, slots: u32) {
    let mut s = slots;
    for _ in 0..26 {
        ee_sub_209130(m, s, o);
        s += 0x10;
    }
}
/// FUN_002080f0: FUN_00209130 on the +0x4c0 entries (+0x4d0, stride 0x20) and the 3 slot groups
pub fn ee_sub_2080f0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let mut i = 0;
    if 0 < m.r32(o + 0x4c0) as i32 {
        let mut e = o + 0x4d0;
        loop {
            ee_sub_209130(m, e, o);
            i += 1;
            e += 0x20;
            if !(i < m.r32(o + 0x4c0) as i32) {
                break;
            }
        }
    }
    ee_sub_2080a0(m, o, o + 0xc10);
    ee_sub_2080a0(m, o, o + 0x8d0);
    ee_sub_2080a0(m, o, o + 0xa70);
}
/// FUN_00208178(o, slots): FUN_002090d0 on 26 slots
pub fn ee_sub_208178<M: QMem + ?Sized>(m: &mut M, o: u32, slots: u32) {
    let mut s = slots;
    for _ in 0..26 {
        ee_sub_2090d0(m, s);
        s += 0x10;
    }
}
/// FUN_002081c8: FUN_00208178 on +0xc10, +0x8d0, +0xa70
pub fn ee_sub_2081c8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    ee_sub_208178(m, o, o + 0xc10);
    ee_sub_208178(m, o, o + 0x8d0);
    ee_sub_208178(m, o, o + 0xa70);
}
/// FUN_00208208(o, slots): FUN_00209590 on 26 slots
pub fn ee_sub_208208<M: QMem + ?Sized>(m: &mut M, o: u32, slots: u32) {
    let mut s = slots;
    for _ in 0..26 {
        ee_sub_209590(m, s);
        s += 0x10;
    }
}
/// FUN_00208250: FUN_00209590 on the entries and the 3 slot groups
pub fn ee_sub_208250<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let mut i = 0;
    if 0 < m.r32(o + 0x4c0) as i32 {
        let mut e = o + 0x4d0;
        loop {
            i += 1;
            ee_sub_209590(m, e);
            e += 0x20;
            if !(i < m.r32(o + 0x4c0) as i32) {
                break;
            }
        }
    }
    ee_sub_208208(m, o, o + 0xc10);
    ee_sub_208208(m, o, o + 0x8d0);
    ee_sub_208208(m, o, o + 0xa70);
}
/// FUN_00208540(o, p): +0x38c = p; the old one's +0x3cc job FUN_0020a790(job, o), the new FUN_001f8278(p) ->
/// +0x3cc, FUN_0020a730(job, o)
pub fn ee_sub_208540<M: QMem + ?Sized>(m: &mut M, o: u32, p: u32) {
    if p != m.r32(o + 0x38c) {
        if m.r32(o + 0x38c) != 0 {
            let j = m.r32(o + 0x3cc);
            m.call(0x0020_a790, &[j, o], &[]);
            m.w32(o + 0x3cc, 0);
        }
        m.w32(o + 0x38c, p);
        if p != 0 {
            let u = m.call(0x001f_8278, &[p], &[]).0;
            m.w32(o + 0x3cc, u);
            m.call(0x0020_a730, &[u, o], &[]);
        }
    }
}
pub fn ee_sub_2085b0<M: QMem + ?Sized>(m: &mut M, o: u32, p: u32) {
    ee_sub_2085d0(m, o, p, 0);
}
/// FUN_002085d0(o, p, front): insert p into the 8-entry (0x10 each) list at (+0xf64)+0x100 (count +0xfc,
/// +0x3a4 = reserved head entries); 1 when inserted
pub fn ee_sub_2085d0<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32, p3: u32) -> u32 {
    if p2 == 0 {
        return 0;
    }
    let mut r = 0;
    if p2 != o {
        let mut l6 = 0;
        if p3 == 0 {
            let l = m.r32(o + 0xf64);
            l6 = m.call(0x0015_4180, &[l, p2], &[]).0;
        }
        r = 0;
        if l6 == 0 {
            let i1 = m.r32(o + 0xf64);
            let u2 = m.r32(i1 + 0xfc);
            let p5;
            if u2 < 8 {
                if p3 == 0 {
                    let k = m.r32(o + 0x3a4);
                    p5 = i1.wrapping_add(u2.wrapping_sub(k).wrapping_mul(0x10)).wrapping_add(0x100);
                    m.call(0x0023_32b8, &[p5 + 0x10, p5, k << 4], &[]);
                } else {
                    p5 = i1 + u2 * 0x10 + 0x100;
                }
                m.w32(p5, p2);
            } else {
                if p3 != 0 {
                    return 0;
                }
                let i3 = m.r32(o + 0x3a4) as i32;
                if i3 < 1 {
                    return 0;
                }
                m.w32(o + 0x3a4, (i3 - 1) as u32);
                p5 = i1.wrapping_add(((8 - i3) as u32).wrapping_mul(0x10)).wrapping_add(0x100);
                m.call(0x0023_32b8, &[p5 + 0x10, p5, ((i3 - 1) as u32) << 4], &[]);
                m.w32(p5, p2);
            }
            m.w32(p5 + 4, 0);
            r = 1;
            let c = m.r32(CLOCK);
            m.w32(p5 + 8, c);
            let b = m.r32(o + 0xf64);
            let n = m.r32(b + 0xfc);
            m.w32(b + 0xfc, n + 1);
        }
    }
    r
}
/// FUN_002087e8 / 0x2089f0 / 0x208b38: p2 is (FUN_00132b10 isa) one of the class ids in the list at
/// +0x394 (+0x398) / +0x3a8 (+0x3ac) / +0x3c0 (+0x3c4)
fn isa_list<M: QMem + ?Sized>(m: &mut M, o: u32, cnt: u32, p2: u32) -> u32 {
    let mut i = 0;
    if 0 < m.r32(o + cnt) as i32 {
        let mut p = o + cnt + 4;
        loop {
            let c = m.r32(p);
            if m.call(0x0013_2b10, &[p2, c], &[]).0 != 0 {
                return 1;
            }
            i += 1;
            p += 4;
            if !(i < m.r32(o + cnt) as i32) {
                break;
            }
        }
    }
    0
}
pub fn ee_sub_2087e8<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32) -> u32 {
    isa_list(m, o, 0x394, p2)
}
pub fn ee_sub_2089f0<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32) -> u32 {
    isa_list(m, o, 0x3a8, p2)
}
pub fn ee_sub_208b38<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32) -> u32 {
    isa_list(m, o, 0x3c0, p2)
}
/// FUN_00208860(o): remove the (+0xf64) list entry FUN_00154180 finds (when past the reserved heads)
pub fn ee_sub_208860<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let i1 = m.r32(o + 0xf64);
    let u2 = m.call(0x0015_4180, &[i1], &[]).0;
    let i1 = m.r32(o + 0xf64);
    let lim = i1
        .wrapping_add(m.r32(i1 + 0xfc).wrapping_sub(m.r32(o + 0x3a4)).wrapping_mul(0x10))
        .wrapping_add(0x100);
    if u2 != 0 && u2 < lim {
        let n = m.r32(i1 + 0xfc);
        m.w32(i1 + 0xfc, n.wrapping_sub(1));
        let b = m.r32(o + 0xf64);
        let n = m.r32(b + 0xfc) as i32;
        let k = ((u2.wrapping_sub(0x100).wrapping_sub(i1)) as i32) >> 4;
        m.call(0x001a_e5a0, &[u2, u2 + 0x10, (n.wrapping_sub(k) as u32).wrapping_mul(0x10)], &[]);
    }
}
/// FUN_00208bb0: FUN_0011d448(5.0, 7.0)
pub fn ee_sub_208bb0<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x0011_d448, &[], &[f(f32::from_bits(0x40a0_0000)), f(f32::from_bits(0x40e0_0000))]);
}
/// FUN_00208e70(o, other, out): attack / approach classification of `other`: distance pair from FUN_001428d8
/// (float at [0], second word at [4]); *out = distance; result 4 (blocked), 3 (hit by FUN_00227c90), 2, 1, 0
pub fn ee_sub_208e70<M: QMem + ?Sized>(m: &mut M, p1: u32, p2: u32, p3: u32) -> u32 {
    let loc = stack_push(m, 0x50);
    m.call(0x0014_28d8, &[p1, p2, p3, loc], &[]);
    if p3 != 0 {
        let d = m.r32(loc);
        m.w32(p3, d);
    }
    let r = e70_body(m, p1, p2, p3, loc);
    stack_pop(m, 0x50);
    r
}
fn e70_body<M: QMem + ?Sized>(m: &mut M, p1: u32, p2: u32, p3: u32, loc: u32) -> u32 {
    let bit1 = ee_sub_207a20(1);
    let to_ff0 = (m.r64(p1 + 0x2c0) & bit1) != 0;
    if !to_ff0 {
        let mut x: u32 = 0;
        if m.r32(p1 + 0x364) != 0 && m.r32(p2 + 0x10f0) != 0 {
            if m.call(0x0020_9a68, &[p1, 0x1d], &[]).0 == 0 {
                if m.call(0x0020_9a68, &[p1, 5], &[]).0 == 0 {
                    if m.call(0x0020_9a68, &[p1, 6], &[]).0 == 0 {
                        if m.call(0x0020_9a68, &[p1, 0x17], &[]).0 == 0 {
                            x = 0;
                        } else {
                            let g = m.call(0x0020_5458, &[p1, 0xb1], &[]).0;
                            x = m.r32(g + 0x5c);
                            if x == p2 {
                                let v = m.rf(p3) + rof(0x002c_3d90);
                                m.wf(p3, v);
                            }
                        }
                    } else {
                        let g = m.call(0x0020_5458, &[p1, 0xa2], &[]).0;
                        x = m.r32(g + 0x50);
                    }
                } else {
                    let g = m.call(0x0020_5458, &[p1, 0xa1], &[]).0;
                    x = m.r32(g + 0x50);
                }
            } else {
                x = m.r32(p1 + 0x38c);
            }
            // LAB_00208f74
            if x != 0 {
                if x == p2 {
                    return 4;
                }
                if m.call(0x0013_2b10, &[x, 0x34], &[]).0 != 0 {
                    return 4;
                }
            }
        }
        // LAB_00208f98
        let iv2 = m.r32(p2 + 0x10dc);
        let l5 = if p1 == iv2 {
            if m.r32(p2 + 0x10e4) == 3 {
                m.r32(p1 + 0x3f8)
            } else {
                m.r32(p1 + 0x3f4)
            }
        } else {
            m.r32(p1 + 0x3f4)
        };
        if l5 != 0 && (m.r64(l5 + 0x28) >> 0x3c) & 1 != 0 {
            let q = m.call(0x001a_62e8, &[p2], &[]).1;
            if m.call(0x0022_7c90, &[l5], &[q]).0 & 0xff != 0 {
                return 3;
            }
        }
    }
    // LAB_00208ff0
    let mut iv2 = p1 + 0x370;
    if m.call(0x0020_9a68, &[p1, 0x17], &[]).0 != 0 {
        iv2 = p1 + 0x368;
    }
    let f50 = m.rf(loc);
    let u4c = m.rf(loc + 4);
    let mut r3 = 2;
    let go = m.rf(p1 + 0x378) * rof(0x002c_3d94) < f50
        || m.call(0x0011_d910, &[iv2], &[f(u4c)]).0 == 0
        || m.r32(p1 + 0x364) == 0;
    if go {
        if f50 <= m.rf(p1 + 0x388) {
            if m.call(0x0011_d910, &[p1 + 0x380], &[f(u4c)]).0 == 0 {
                return 0;
            }
            if m.r32(p1 + 0x37c) != 0 {
                return 1;
            }
        }
        r3 = 0;
    }
    r3
}
/// FUN_002090d0(slot): release the kind-2 / kind-3 object, [0] = [1] = -1
pub fn ee_sub_2090d0<M: QMem + ?Sized>(m: &mut M, s: u32) {
    let k = m.r32(s);
    if k == 2 {
        let o = m.r32(s + 4);
        m.call(0x0020_c030, &[o], &[]);
        m.w32(s + 4, 0);
    } else if k == 3 {
        let o = m.r32(s + 4);
        m.call(0x0020_ca20, &[o], &[]);
        m.w32(s + 4, 0);
    }
    m.w32(s, NEG1);
    m.w32(s + 4, NEG1);
}
/// FUN_00209130(slot, o): (re)build the slot's object: kind 2 -> FUN_0020bd88(obj, o), 3 -> FUN_0020c7e0(obj),
/// else FUN_0015e938(world, 4, id, o) object: class 0x6d (FUN_00132ea0) -> kind 0 / 2 (when FUN_0012f938(obj,
/// 0x22)), class 0x6f -> kind 1, else -1
pub fn ee_sub_209130<M: QMem + ?Sized>(m: &mut M, s: u32, o: u32) {
    let k = m.r32(s);
    if k == 2 {
        let a = m.r32(s + 4);
        m.call(0x0020_bd88, &[a, o], &[]);
    } else if k == 3 {
        let a = m.r32(s + 4);
        m.call(0x0020_c7e0, &[a], &[]);
    } else {
        let id = m.r32(s + 4);
        let l1 = m.call(0x0015_e938, &[WORLD, 4, id, o], &[]).0;
        if l1 == 0 {
            m.w32(s, NEG1);
        } else {
            let l2 = m.call(0x0013_2ea0, &[l1], &[]).0;
            if l2 == 0x6d {
                if m.call(0x0012_f938, &[l1, 0x22], &[]).0 == 0 {
                    m.w32(s + 4, l1);
                    m.w32(s, 0);
                } else {
                    m.w32(s, 2);
                    let u = m.call(0x0020_c010, &[0x40], &[]).0;
                    m.w32(s + 4, u);
                    m.call(0x0020_bd78, &[u], &[]);
                    m.w32(u + 4, l1);
                    m.call(0x0020_bd88, &[u, o], &[]);
                    if m.r32(s + 0xc) == NEG1 {
                        m.w32(s + 0xc, 0);
                    }
                }
            } else if l2 == 0x6f {
                m.w32(s, 1);
                m.w32(s + 4, l1);
                if m.r32(s + 0xc) == NEG1 {
                    m.w32(s + 0xc, 0);
                }
            } else {
                m.w32(s, NEG1);
            }
        }
    }
}
/// FUN_00209278(this, slot, o, p4): start the slot's object by kind (0 JtAnimator_request, 1 FUN_001f0198,
/// 2 FUN_0020bf80 in o+0xdb0, 3 FUN_0020c948 in o+0xe20) and call its virtual +0xc
pub fn ee_sub_209278<M: QMem + ?Sized>(m: &mut M, p1: u32, s: u32, p3: u32, p4: u32) {
    let a = m.r32(s + 4);
    let i1;
    match m.r32(s) {
        0 => {
            m.call(0x0012_f9b8, &[p1, 0x3f80_0000, a, p3, p4, s + 8], &[]);
            i1 = m.r32(s + 8);
        }
        1 => {
            m.call(0x001f_0198, &[a, p3, NEG1, p4, s + 8], &[]);
            i1 = m.r32(s + 8);
        }
        2 => {
            let t = p3.wrapping_add(0xdb0);
            m.w32(s + 8, t);
            m.call(0x0020_bf80, &[a, p3, p4, t], &[]);
            i1 = m.r32(s + 8);
        }
        3 => {
            let t = p3.wrapping_add(0xe20);
            m.w32(s + 8, t);
            m.call(0x0020_c948, &[a, p3, p4, t], &[]);
            let obj = m.r32(s + 8);
            vcall(m, obj, 0xc, 0xc, &[], &[]);
            return;
        }
        _ => return,
    }
    vcall(m, i1, 0xc, 0xc, &[], &[]);
}
/// FUN_00209360(slot): stop the slot's object (kind 0 FUN_00130b18, 1 FUN_001f0598, 2 FUN_0020c080,
/// 3 FUN_0020ca48), virtual +0x14 on it, [2] = 0
pub fn ee_sub_209360<M: QMem + ?Sized>(m: &mut M, s: u32) {
    let f = match m.r32(s) {
        0 => 0x0013_0b18,
        1 => 0x001f_0598,
        2 => 0x0020_c080,
        3 => 0x0020_ca48,
        _ => return,
    };
    let o = m.r32(s + 8);
    if o == 0 {
        return;
    }
    m.call(f, &[o], &[]);
    let o = m.r32(s + 8);
    vcall(m, o, 0xc, 0x14, &[], &[]);
    m.w32(s + 8, 0);
}
/// FUN_00209430(slot): the slot's object is done (kind 0 FUN_00131968, 1 FUN_001f0638 / +0x30 check,
/// 2 FUN_0020c0d8, 3 FUN_0020ca98); 1 when none
pub fn ee_sub_209430<M: QMem + ?Sized>(m: &mut M, s: u32) -> u32 {
    let k = m.r32(s) as i32;
    let f = match k {
        0 => 0x0013_1968,
        1 => 0x001f_0638,
        2 => 0x0020_c0d8,
        3 => 0x0020_ca98,
        _ => return 1,
    };
    let o = m.r32(s + 8);
    if o == 0 {
        return 1;
    }
    let l = m.call(f, &[o], &[]).0;
    if k == 1 {
        if l == 0 {
            let o = m.r32(s + 8);
            if m.r32(o + 0x30) != 0 {
                return 0;
            }
            return 1;
        }
        return 1;
    }
    (l != 0) as u32
}
/// FUN_00209518(slot): the object's key: kind 0 [2], 1 (+0x30), 2 (+0x38), 3 (+0x24)
pub fn ee_sub_209518<M: QMem + ?Sized>(m: &mut M, s: u32) -> u32 {
    let o = m.r32(s + 8);
    match m.r32(s) as i32 {
        1 => m.r32(o + 0x30),
        0 => o,
        2 => m.r32(o + 0x38),
        3 => m.r32(o + 0x24),
        _ => 0,
    }
}
/// FUN_00209590(slot): kind 2 / 3: clone the object (0x40 / 0xd0 bytes, FUN_001ae5a0 copy) into a new allocation
pub fn ee_sub_209590<M: QMem + ?Sized>(m: &mut M, s: u32) {
    let (n, f) = match m.r32(s) {
        2 => (0x40, 0x0020_c010),
        3 => (0xd0, 0x0020_ca00),
        _ => return,
    };
    let u = m.call(f, &[n], &[]).0;
    let src = m.r32(s + 4);
    m.call(0x001a_e5a0, &[u, src, n], &[]);
    m.w32(s + 4, u);
}
/// FUN_00209768 / 0x209770: +0xc = v
pub fn ee_sub_209768<M: QMem + ?Sized>(m: &mut M, o: u32, v: u32) {
    m.w32(o + 0xc, v);
}
pub fn ee_sub_209770<M: QMem + ?Sized>(m: &mut M, o: u32, v: u32) {
    m.w32(o + 0xc, v);
}
/// FUN_00209778: [0] = [1] = [2] = 0
pub fn ee_sub_209778<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.w32(o + 8, 0);
    m.w32(o, 0);
    m.w32(o + 4, 0);
}
/// FUN_00209788: [0] = v
pub fn ee_sub_209788<M: QMem + ?Sized>(m: &mut M, o: u32, v: u32) {
    m.w32(o, v);
}
/// FUN_00209790: [0] = [1] = 0, host(+0xc) +0x194 / +0x198 / +0x190 = 0, FUN_001f5078(host)
pub fn ee_sub_209790<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.w32(o, 0);
    let h = m.r32(o + 0xc);
    m.w32(o + 4, 0);
    m.w32(h + 0x194, 0);
    m.w32(h + 0x198, 0);
    m.w32(h + 400, 0);
    let h = m.r32(o + 0xc);
    m.call(0x001f_5078, &[h], &[]);
}
/// FUN_002097e0: virtual +0x5c of the object at [0] when set
pub fn ee_sub_2097e0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let i = m.r32(o);
    if i != 0 {
        vcall(m, i, 0xc, 0x5c, &[], &[]);
    }
}
/// FUN_00209870(o, mask): +0x2c0 = mask; FUN_001f1900(o)
pub fn ee_sub_209870<M: QMem + ?Sized>(m: &mut M, o: u32, mask: u64) {
    w64(m, o + 0x2c0, mask);
    m.call(0x001f_1900, &[o], &[]);
}
/// FUN_00209890(o, id): set the state bit, notify virtual +0x1c (7, 0, 2, {&id, &-1}, 0), and for the 2 table
/// entries (DAT_002c3d98: {id, activity}) matching FUN_002099f0 start the world activity
pub fn ee_sub_209890<M: QMem + ?Sized>(m: &mut M, o: u32, id: u32) {
    let b = stack_push(m, 0x80);
    let (p80, p7c, u70, a6c) = (b, b + 4, b + 0x10, b + 0x14);
    m.w32(u70, id);
    let mask = ee_sub_207a20(id as i32 as i64);
    let cur = m.r64(o + 0x2c0);
    ee_sub_209870(m, o, cur | mask);
    m.w32(p7c, a6c);
    m.w32(p80, u70);
    m.w32(a6c, NEG1);
    vcall(m, o, 0xc, 0x1c, &[7, 0, 2, p80, 0], &[]);
    let mut p = 0x002c_3d98;
    for _ in 0..2 {
        let e = m.r32(p);
        if ee_sub_2099f0(mask, e as i32 as i64) {
            let act = m.r32(p + 4);
            m.call(0x0016_19a0, &[0x002d_eb30, act, o], &[]);
        }
        p += 8;
    }
    stack_pop(m, 0x80);
}
/// FUN_00209970(o, id): clear the state bit, notify virtual +0x1c
pub fn ee_sub_209970<M: QMem + ?Sized>(m: &mut M, o: u32, id: u32) {
    let b = stack_push(m, 0x40);
    let (p40, p3c, u30, a2c) = (b, b + 4, b + 0x10, b + 0x14);
    m.w32(a2c, id);
    let mask = ee_sub_207a20(id as i32 as i64);
    let cur = m.r64(o + 0x2c0);
    ee_sub_209870(m, o, cur & !mask);
    m.w32(p40, u30);
    m.w32(p3c, a2c);
    m.w32(u30, NEG1);
    vcall(m, o, 0xc, 0x1c, &[7, 0, 2, p40, 0], &[]);
    stack_pop(m, 0x40);
}
/// FUN_002099f0(mask, id)
pub fn ee_sub_2099f0(mask: u64, id: i64) -> bool {
    mask & ee_sub_207a20(id) != 0
}
/// FUN_00209a28(a, b): popcount of a when a is a subset of b, else 0
pub fn ee_sub_209a28(a: u64, b: u64) -> i32 {
    let mut b = a & b;
    if b != a {
        return 0;
    }
    let mut n = 0;
    while b != 0 {
        b &= b - 1;
        n += 1;
    }
    n
}
/// FUN_00209a88(out, o): FUN_00200f58(out, ids(mask +0x2c0), 0x1010, buf)
pub fn ee_sub_209a88<M: QMem + ?Sized>(m: &mut M, out: u32, o: u32) -> u32 {
    let buf = stack_push(m, 0xe0);
    let mask = m.r64(o + 0x2c0);
    let n = ee_sub_207aa0(m, mask, buf);
    m.call(0x0020_0f58, &[out, n as u32, 0x1010, buf], &[]);
    stack_pop(m, 0xe0);
    out
}
/// FUN_00209be0(o, key): the +0x4d0 entry (stride 0x20) with 64-bit key (+0x10) == key, or 0
pub fn ee_sub_209be0<M: QMem + ?Sized>(m: &mut M, o: u32, key: u64) -> u32 {
    let mut i = 0;
    if 0 < m.r32(o + 0x4c0) as i32 {
        let mut e = o + 0x4d0;
        loop {
            i += 1;
            if m.r64(e + 0x10) == key {
                return e;
            }
            e += 0x20;
            if !(i < m.r32(o + 0x4c0) as i32) {
                break;
            }
        }
    }
    0
}
/// FUN_00209c18(o, key): find or append the entry of key, FUN_002090d0 on it
pub fn ee_sub_209c18<M: QMem + ?Sized>(m: &mut M, o: u32, key: u64) -> u32 {
    let mut e = ee_sub_209be0(m, o, key);
    if e == 0 {
        let n = m.r32(o + 0x4c0);
        e = o.wrapping_add(n.wrapping_mul(0x20)).wrapping_add(0x4d0);
        m.w32(o + 0x4c0, n.wrapping_add(1));
        w64(m, e + 0x10, key);
    }
    ee_sub_2090d0(m, e);
    e
}
/// FUN_00209e88: states 0x16 / 0x12 -> FUN_0017a8c8(Guard_k24_comp_b0(o, 0xb0), 1)
pub fn ee_sub_209e88<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.call(0x0020_9a68, &[o, 0x16], &[]).0 != 0 || m.call(0x0020_9a68, &[o, 0x12], &[]).0 != 0 {
        let g = m.call(0x0020_5458, &[o, 0xb0], &[]).0;
        m.call(0x0017_a8c8, &[g, 1], &[]);
    }
}
/// FUN_00209ee0: states 0x12 / 0x13 -> FUN_0017a8c8(comp 0xb0, 2); else state 0x17 -> comp 0xb1: [0x50] 3 or 6
/// keep, else FUN_0017b1a8(comp, 2); then virtual +0x10c -> FUN_0011e9f8(DAT_002c3da8, v) -> virtual +0x11c
pub fn ee_sub_209ee0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.call(0x0020_9a68, &[o, 0x12], &[]).0 == 0 && m.call(0x0020_9a68, &[o, 0x13], &[]).0 == 0 {
        if m.call(0x0020_9a68, &[o, 0x17], &[]).0 == 0 {
            return;
        }
        let g = m.call(0x0020_5458, &[o, 0xb1], &[]).0;
        let k = m.r32(g + 0x50);
        if k != 3 && k != 6 {
            m.call(0x0017_b1a8, &[g, 2], &[]);
        }
    } else {
        let g = m.call(0x0020_5458, &[o, 0xb0], &[]).0;
        m.call(0x0017_a8c8, &[g, 2], &[]);
    }
    let v = vcall(m, o, 0xc, 0x10c, &[], &[]).0;
    let r = m.call(0x0011_e9f8, &[v], &[f(rof(0x002c_3da8))]).0;
    vcall(m, o, 0xc, 0x11c, &[r], &[]);
}
/// FUN_00209fb8: states 0x16 / 0x13 -> FUN_0017a8c8(comp 0xb0, 3)
pub fn ee_sub_209fb8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.call(0x0020_9a68, &[o, 0x16], &[]).0 != 0 || m.call(0x0020_9a68, &[o, 0x13], &[]).0 != 0 {
        let g = m.call(0x0020_5458, &[o, 0xb0], &[]).0;
        m.call(0x0017_a8c8, &[g, 3], &[]);
    }
}
/// FUN_0020a010: state 0x17 -> comp 0xb1 +0x58, else 0
pub fn ee_sub_20a010<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    if m.call(0x0020_9a68, &[o, 0x17], &[]).0 == 0 {
        return 0;
    }
    let g = m.call(0x0020_5458, &[o, 0xb1], &[]).0;
    m.r32(g + 0x58)
}
/// FUN_0020a058(o, a, b): two Splice list parses (type 0x2002, 4 ids: o+0xf94 / o+0xf98 and o+0xfa8 / o+0xfac)
pub fn ee_sub_20a058<M: QMem + ?Sized>(m: &mut M, o: u32, a: u32, b: u32) {
    let base = stack_push(m, 0x70);
    let (b60, b50) = (base, base + 0x10);
    m.w32(b60, NEG1);
    m.call(0x0020_13d8, &[b50, a, 0x2002, 4, o + 0xf94, o + 0xf98], &[]);
    m.call(0x0012_9658, &[b60, b50], &[]);
    release(m, b50);
    m.call(0x0020_13d8, &[b50, b, 0x2002, 4, o + 0xfa8, o + 0xfac], &[]);
    m.call(0x0012_9658, &[b60, b50], &[]);
    release(m, b50);
    release(m, b60);
    stack_pop(m, 0x70);
}
/// FUN_0020a100(o, id, v): +0xf68 = id, +0xf70 = v; the +0x26c comp 0x9c (kind 0x10) job gets id / v too
pub fn ee_sub_20a100<M: QMem + ?Sized>(m: &mut M, o: u32, id: u32, v: V4) {
    m.w32(o + 0xf68, id);
    for i in 0..4 {
        m.wf(o + 0xf70 + 4 * i, v[i as usize]);
    }
    let l = m.call(0x001e_eee0, &[o + 0x26c, 0x9c], &[]).0;
    if l != 0 && m.r32(o + 0xf68) != 0 && m.r32(l + 0x30) == 0x10 {
        let i1 = m.r32(l + 0x18);
        let id = m.r32(o + 0xf68);
        m.w32(i1 + 0x30, id);
        for k in 0..4 {
            let w = m.r32(o + 0xf70 + 4 * k);
            m.w32(i1 + 0x40 + 4 * k, w);
        }
    }
}
