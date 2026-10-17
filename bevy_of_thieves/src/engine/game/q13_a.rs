//! WP-Q13 (part a): run_00139950 .. run_00195608 free functions. All against `QMem`; callees by address through
//! `QMem::call`. Virtual calls use `vc` (this-adjust s16 at vt+adj_off, function at vt+fn_off, as in the decompile).
use super::q_00114680::*;
use super::q_0014cc58::ee_sub_14cdb0;
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem, V4};

/// `(**(vt + fn_off))(obj + *(s16*)(vt + adj_off), args..)`
pub fn vc<M: QMem + ?Sized>(m: &mut M, obj: u32, adj_off: u32, fn_off: u32, a: &[u32], v: &[V4]) -> (u32, V4) {
    let vt = m.r32(obj + 0xc);
    let adj = m.r16(vt + adj_off) as i16 as i32 as u32;
    let f = m.r32(vt + fn_off);
    let mut args = vec![obj.wrapping_add(adj)];
    args.extend_from_slice(a);
    m.call(f, &args, v)
}

// FUN_00139950 SKIPPED: batch transform whose per-element work is the VU0 micro subroutine 0x10 (_vcallms(0x10));
// the microcode is not decoded (inputs/outputs are only reachable through VU0 registers).

/// FUN_00139ab8: walk the chain of 0x840-byte double-buffered batches at o+0x448, hand each to FUN_00139950
/// (count consumed -> subtract from +0x1084 / o+0x444), then place the effect at the centre of the bounds A/B
/// (+0xf0) with radius |mid - B| + o[+0x120] (+0x100), or call vtable +0x4c when nothing is left.
/// FlushCache(0) dropped; FUN_00139950 (skipped, VU0) is called by address. [HARD2: completed] The decompile
/// passes exactly 8 args ending in &uStack_140 (= A at sp+0x00); B is reached by the callee at A+0x10, so no
/// separate &B argument exists. Bounds B is read back from `b` (= a+0x10) after the call.
pub fn ee_sub_139ab8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let sp = stack_push(m, 0xe0);
    let (a, b, mm, s) = (sp, sp + 0x10, sp + 0x20, sp + 0x60);
    let f4 = m.r32(0x002a_4af4);
    let f8 = m.r32(0x002a_4af8);
    for k in 0..3 {
        m.w32(a + 4 * k, f4);
        m.w32(b + 4 * k, f8);
    }
    m.w32(a + 12, 0);
    m.w32(b + 12, 0);
    m.call(0x0015_bfd8, &[s, o + 0x170], &[]);
    for k in 0..16 {
        let w = m.r32(s + 4 * k);
        m.w32(mm + 4 * k, w);
    }
    let d8 = m.r32(o + 0x450);
    m.w32(s + 8, d8);
    let old = m.r32(0x002d_d2fc);
    m.w32(s + 12, old);
    let mut i1 = m.r32(o + 0x448);
    m.w32(s + 4, i1.wrapping_add(d8));
    let i6 = m.r32(i1 + 0x1080).wrapping_mul(0x840).wrapping_add(i1);
    m.w32(0x002d_d2fc, s);
    m.w32(s, o + 0x448);
    let i3 = m.r32(i1 + 0x1084) << 4;
    m.call(0x0020_1620, &[i3, i6, i3, i6 + 0x2b0, i3, i6 + 0x560, 0, 0], &[]);
    let (u11, u2) = (m.r32(s + 56), m.r32(s + 60));
    while i1 != 0 {
        m.call(0x0020_1998, &[], &[]);
        let p = m.r32(s + 4);
        let i3 = m.r32(p);
        let d8 = m.r32(s + 8);
        m.w32(s + 4, i3.wrapping_add(d8));
        if i3 != 0 {
            let i6 = m.r32(i3 + 0x1084) << 4;
            let i4 = m.r32(i3 + 0x1080).wrapping_mul(0x840).wrapping_add(i3);
            m.call(0x0020_1620, &[i6, i4, i6, i4 + 0x2b0, i6, i4 + 0x560, 0, 0], &[]);
        }
        let i6 = m.r32(0x002c_3c5c).wrapping_mul(0x50);
        let bits = m.r32(0x002d_ded4);
        m.w32(s + 16, bits);
        m.w32(s + 20, ((bits as i32) >> 31) as u32);
        m.w32(s + 24, u11);
        m.w32(s + 28, u2);
        let x1 = m.r32(0x003d_d2c8 + i6 + 4);
        let x2 = m.r32(0x003d_d2d0 + i6 + 4);
        let x3 = m.r32(0x003d_d2d8 + i6 + 4);
        let cnt = m.r32(i1 + 0x1084);
        let k7 = m.r32(o + 0x43c);
        let used = m.call(0x0013_9950, &[mm, bits, x1, x2, x3, cnt, k7, a], &[]).0;
        let u7 = m.r32(i1 + 0x1080).wrapping_add(1) & 1;
        let i4 = u7.wrapping_mul(0x840).wrapping_add(i1);
        m.call(0x0020_17a0, &[i4, i4 + 0x2b0, i4 + 0x560, 0], &[]);
        let c = (m.r32(i1 + 0x1084) as i32).wrapping_sub(used as i32);
        m.w32(i1 + 0x1084, c as u32);
        let t = (m.r32(o + 0x444) as i32).wrapping_sub(used as i32);
        m.w32(o + 0x444, t as u32);
        if c < 1 {
            m.call(0x0013_9098, &[i1], &[]);
        } else {
            m.w32(i1 + 0x1080, u7);
        }
        i1 = i3;
    }
    let dc = m.r32(0x002d_e59c);
    m.call(0x0024_17f0, &[dc, 0, 0], &[]);
    let (va, vb) = (m.rv(a), m.rv(b));
    if (m.r32(o + 0x444) as i32) < 1 {
        vc(m, o, 0x48, 0x4c, &[], &[]);
    } else {
        let mid = [(va[0] + vb[0]) * 0.5, (va[1] + vb[1]) * 0.5, (va[2] + vb[2]) * 0.5, (va[3] + vb[3]) * 0.5];
        let d = [mid[0] - vb[0], mid[1] - vb[1], mid[2] - vb[2]];
        let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        m.wv(o + 0xf0, mid);
        let r = m.rf(o + 0x120);
        m.wf(o + 0x100, len + r);
    }
    let old = m.r32(s + 12);
    m.w32(0x002d_d2fc, old);
    stack_pop(m, 0xe0);
}

/// FUN_0016ea48
pub fn ee_sub_16ea48<M: QMem + ?Sized>(m: &mut M, p: u32) {
    m.call(0x0015_74b8, &[p], &[]);
    m.call(0x0022_6d28, &[p], &[]);
    m.call(0x0015_76f0, &[p], &[]);
}

/// FUN_00171af8
pub fn ee_sub_171af8<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32, p3: u32, p4: u32) {
    let t = m.r32(o + 0x10);
    let q = m.call(0x001a_62e8, &[t], &[]).1;
    let a = m.r32(o + 0x30);
    m.call(0x0017_10f8, &[a, p2, p3, p4], &[q]);
}
/// FUN_00171b60
pub fn ee_sub_171b60<M: QMem + ?Sized>(m: &mut M, p1: u32, p2: u32, q: V4) {
    let o = m.r32(p2 + 0x10);
    let lo = q[0].to_bits();
    m.call(0x0020_b210, &[o, lo], &[q]);
    vc(m, o, 0xe8, 0xec, &[lo], &[q]);
    m.call(0x0020_4660, &[p1, o], &[]);
    vc(m, o, 0xf0, 0xf4, &[o + 0x1b0], &[]);
    vc(m, o, 0x118, 0x11c, &[], &[]);
    vc(m, o, 0x120, 0x124, &[], &[]);
}
/// FUN_00171c18
pub fn ee_sub_171c18<M: QMem + ?Sized>(m: &mut M, o: u32, out: u32) {
    let t = m.r32(o + 0x10);
    let k = m.r32(t + 0xf80) as i32;
    if k == -1 {
        let p = m.r32(o + 0x38);
        m.w32(out, p);
        let w = m.r32(o + 0x34);
        m.w32(out + 4, w);
        if p != 0 && m.r32(p) == 6 {
            let a = m.r32(o + 0x18);
            let w = m.r32(a + 0x38);
            m.w32(out + 8, w);
        }
    } else if -2 < k && k < 2 {
        m.call(0x0023_33b4, &[out, 0, 0xc], &[]);
    }
}
/// FUN_00171c98
pub fn ee_sub_171c98<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32) {
    let mut u = m.r32(o + 0x30);
    if u != 0 {
        if m.r32(o + 0x40) != 0 {
            m.call(0x001e_f0d8, &[o + 0x18], &[]);
            m.w32(o + 0x40, 0);
            u = m.r32(o + 0x30);
        }
        let r = m.call(0x0017_11f0, &[u, p2], &[]).0;
        m.w32(o + 0x38, r);
    }
}
/// FUN_00171f38
pub fn ee_sub_171f38<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_171e00(m, 1, 0xffff)
}
/// FUN_00176b80: build a 0x60-byte request (kind p3, owner +0x10, flags 0x40000, arg p2) for FUN_001f7598
pub fn ee_sub_176b80<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32, p3: u32) {
    let sp = stack_push(m, 0x60);
    m.call(0x0023_33b4, &[sp, 0, 0x60], &[]);
    let ow = m.r32(o + 0x10);
    m.w32(sp + 8, ow);
    m.w32(sp + 0xc, 0x40000);
    m.w32(sp, p3);
    m.w32(sp + 0x10, p2);
    m.call(0x001f_7598, &[sp], &[]);
    stack_pop(m, 0x60);
}
/// FUN_0017c928: state change of the sidekick: 0 / 1 spawn behaviour 0xba (position record, flag +0xc0),
/// 2 spawn 0x9e; each (re)starts through vslot +0x40/+0x44
pub fn ee_sub_17c928<M: QMem + ?Sized>(m: &mut M, o: u32, st: u64) {
    if st as u32 == m.r32(o + 0x50) {
        return;
    }
    m.call(0x001e_f0d8, &[o + 0x18], &[]);
    m.w32(o + 0x50, st as u32);
    let t = m.r32(0x002d_ded0);
    m.w32(o + 0x54, t);
    let owner = m.r32(o + 0x10);
    let tgt = m.r32(o + 0x30);
    if st == 1 {
        let b = m.call(0x0020_5438, &[owner, 0xba], &[]).0;
        let q = m.call(0x001a_62e8, &[tgt], &[]).1;
        m.wv(o + 0x60, q);
        m.w32(b + 0xc0, 1);
        m.w32(b + 0xac, 1);
        m.w32(b + 0xa8, tgt);
        let sp = stack_push(m, 0x20);
        m.call(0x0025_62a8, &[sp], &[]);
        m.wv(sp + 0x10, q);
        m.call(0x0017_dd38, &[b, sp], &[]);
        stack_pop(m, 0x20);
        m.w32(b + 0x38, 0x1c);
        vc(m, b, 0x40, 0x44, &[], &[]);
    } else if st == 0 {
        let b = m.call(0x0020_5438, &[owner, 0xba], &[]).0;
        let q = m.call(0x001a_62e8, &[tgt], &[]).1;
        m.wv(o + 0x60, q);
        let q2 = m.rv(o + 0x40);
        let r = m.call(0x001a_6f00, &[tgt, 0], &[q2]).1;
        m.w32(b + 0xc0, 1);
        let sp = stack_push(m, 0x20);
        m.call(0x0025_62a8, &[sp], &[]);
        m.wv(sp + 0x10, r);
        m.call(0x0017_dd38, &[b, sp], &[]);
        stack_pop(m, 0x20);
        m.w32(b + 0xa8, tgt);
        vc(m, b, 0x40, 0x44, &[], &[]);
    } else if st == 2 {
        let b = m.call(0x0020_5438, &[owner, 0x9e], &[]).0;
        m.w32(b + 0x3c, 1);
        m.w32(b + 0x38, tgt);
        vc(m, b, 0x40, 0x44, &[], &[]);
    }
}
/// FUN_001816c0
pub fn ee_sub_1816c0<M: QMem + ?Sized>(m: &mut M, o: u32, i: u64, a: u32, b: u32) {
    if i < 3 {
        let base = o.wrapping_add((i as u32).wrapping_mul(0x10));
        m.w32(base + 0x1d0, a);
        m.w32(base + 0x1dc, b);
    }
}
/// FUN_001816e8
pub fn ee_sub_1816e8<M: QMem + ?Sized>(m: &mut M, o: u32, a: u32, b: u32, c: u32, d: u32) {
    m.call(0x0020_fec0, &[0x003d_d490, o + 0x204, 4, o + 0x200, a, b, c, d], &[]);
}
/// FUN_00181728
pub fn ee_sub_181728<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let a = m.r32(o + 0x1c8);
    m.call(0x0017_fd08, &[a, 1], &[]);
    m.w32(o + 0x1c8, 0);
    vc(m, o, 0x18, 0x1c, &[6, 0, 0, 0, 0], &[]);
}
/// FUN_00181780
pub fn ee_sub_181780<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let a = m.r32(o + 0x200);
    m.call(0x0020_ff70, &[0x003d_d490, a, o + 0x204, o, 1], &[]);
    vc(m, o, 0x18, 0x1c, &[5, 0, 0, 0, 0], &[]);
}
/// FUN_0018c120: per bit of `mask` enable (0x12cfe0) / disable (0x12d020) the seven child objects at +0x19e4..
pub fn ee_sub_18c120<M: QMem + ?Sized>(m: &mut M, o: u32, mask: u64) {
    if m.r32(o + 0x19e4) == 0 {
        return;
    }
    for (bit, off) in [(1u64, 0x19e4u32), (2, 0x19e8), (4, 0x19ec), (8, 0x19f0), (0x10, 0x19f4), (0x20, 0x19fc), (0x40, 0x19f8)] {
        let c = m.r32(o + off);
        m.call(if mask & bit == 0 { 0x0012_d020 } else { 0x0012_cfe0 }, &[c], &[]);
    }
}
/// FUN_0018ed70
pub fn ee_sub_18ed70<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.r32(o + 0x18d8) == 0 {
        m.call(0x001c_c2e8, &[o + 0x17d0], &[]);
    }
}
/// FUN_00191070
pub fn ee_sub_191070<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_190f38(m, 1, 0xffff)
}
// FUN_00190c58 SKIPPED: pure VU0 macro-mode vector code (vsub/vsqrt/vdiv/vmaddbc) returning a quadword half through
// _qmfc2; the returned register pair cannot be recovered from the decompile.

/// FUN_00195608: find the 0x30-byte slot at +0x4680 (count +0x4678) whose id == p2, build its object with FUN_001956d0
/// from +0x4744 and install it in slot+0x10 (freeing the previous one through its vtable +0x4c). The decompile reads an
/// unset register (`unaff_s0`) when nothing matches; that case is treated as null.
pub fn ee_sub_195608<M: QMem + ?Sized>(m: &mut M, o: u32, id: u32) {
    if id == 0 || m.r32(o + 0x4744) == 0 {
        return;
    }
    let n = m.r32(o + 0x4678) as i32;
    let mut slot = 0u32;
    for i in 0..n.max(0) {
        let s = o + 0x4680 + (i as u32) * 0x30;
        if m.r32(s) == id {
            slot = s;
            break;
        }
    }
    if slot == 0 {
        return;
    }
    let src = m.r32(o + 0x4744);
    let r = ee_sub_1956d0(m, o, src, slot);
    if r != 0 {
        let old = m.r32(slot + 0x10);
        if old != 0 {
            vc(m, old, 0x48, 0x4c, &[], &[]);
        }
        m.w32(slot + 0x10, r);
    }
}

/// FUN_001956d0: instantiate (FUN_001cb250) from template `src`, position it (slot+0x20 quad word) and set the
/// default FX parameters (0x2a2d00, DAT_002a2dd0 pair)
pub fn ee_sub_1956d0<M: QMem + ?Sized>(m: &mut M, o: u32, src: u32, slot: u32) -> u32 {
    let _ = o;
    if src == 0 {
        return 0;
    }
    let l = m.call(0x001c_b250, &[src, 0], &[]).0;
    if l == 0 {
        return 0;
    }
    m.call(0x001a_58d0, &[l, 0, 0], &[]);
    vc(m, l, 0x48, 0x4c, &[], &[]);
    m.call(0x001f_7c68, &[0x3e80_0000, l], &[]);
    let (a, b) = (m.r32(slot + 0x20), m.r32(slot + 0x24));
    vc(m, l, 0xe8, 0xec, &[a, b], &[]);
    vc(m, l, 0xf0, 0xf4, &[0x002a_2d00], &[]);
    let (c, d) = (m.r32(0x002a_2dd0), m.r32(0x002a_2dd4));
    vc(m, l, 0x118, 0x11c, &[c, d], &[]);
    vc(m, l, 0x120, 0x124, &[c, d], &[]);
    l
}
/// FUN_001957c0
pub fn ee_sub_1957c0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let r = m.call(0x0015_e938, &[o, 0x405, 0x624, 0], &[]).0;
    m.w32(o + 0x4744, r);
}
/// FUN_00195930
pub fn ee_sub_195930<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_1957f8(m, 1, 0xffff)
}
/// FUN_001959c0
pub fn ee_sub_1959c0<M: QMem + ?Sized>(m: &mut M, p: u32) {
    let a = m.r32(p);
    m.call(0x0013_7a30, &[a, 0xc, 5, p + 0x5f * 4, 0, 0, 0, p + 0x6e * 4], &[]);
}
/// FUN_0019cbd8
pub fn ee_sub_19cbd8<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_19caa0(m, 1, 0xffff)
}
/// FUN_0019cc30
pub fn ee_sub_19cc30<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0019_d540, &[o + 0x24], &[]);
}
/// FUN_0019d4d0
pub fn ee_sub_19d4d0<M: QMem + ?Sized>(m: &mut M, p: u32, p2: u32) {
    m.w32(p, p2);
    m.w32(p + 4, 0xffff_ffff);
    let w = m.r32(p2);
    let l = m.call(0x0013_2b10, &[w, 0x35], &[]).0;
    if l == 0 {
        m.w32(p + 0x14, 0);
    } else {
        let a = m.r32(p);
        let i = m.r32(a);
        m.w32(p + 0x14, i.wrapping_add(0x1d40));
        m.call(0x0020_1d78, &[i.wrapping_add(0x1d40), p2], &[]);
    }
}
/// FUN_0019d948: record in +8 object (+0xcc / +0xd0) the focus id and the first matching list entry's +0x5c
pub fn ee_sub_19d948<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let t = m.r32(o + 8);
    if t == 0 {
        return;
    }
    m.w32(t + 0xcc, 0xffff_ffff);
    m.w32(t + 0xd0, 0xffff_ffff);
    let f = m.r32(0x003e_0f9c);
    if f != 0 && m.r32(f + 0x6c) != 0xffff_ffff {
        let t = m.r32(o + 8);
        let w = m.r32(f + 0x6c);
        m.w32(t + 0xcc, w);
        let sp = stack_push(m, 0x90);
        m.call(0x001a_e478, &[sp], &[]);
        m.call(0x0016_5c98, &[sp | 4, sp | 8], &[]);
        let (n, arr) = (m.r32(sp + 4) as i32, m.r32(sp + 8));
        m.call(0x0023_38c0, &[arr, n as u32, 4, 0x0016_5c28], &[]);
        for i in 0..n.max(0) {
            let e = m.r32(arr + 4 * i as u32);
            if f == m.r32(e + 0x7c) && m.r32(e + 0x5c) != 0xffff_ffff && m.call(0x0016_4030, &[e], &[]).0 == 0 {
                let t = m.r32(o + 8);
                let w = m.r32(e + 0x5c);
                m.w32(t + 0xd0, w);
                break;
            }
        }
        m.call(0x001a_e4a0, &[sp, 2], &[]);
        stack_pop(m, 0x90);
    }
}
/// FUN_0019ddf0
pub fn ee_sub_19ddf0<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_19dcb8(m, 1, 0xffff)
}
/// FUN_0019e950
pub fn ee_sub_19e950<M: QMem + ?Sized>(m: &mut M, p: u32) {
    let sp = stack_push(m, 0x40);
    let (a, b, q) = (sp, sp + 0x10, sp + 0x20);
    let (p4, p3) = (m.r32(p + 0x10), m.r32(p + 0xc));
    m.call(0x001e_6f10, &[p4, p3, a, b], &[]);
    let o = m.r32(p);
    let qv = m.rv(p + 0x30);
    let r = m.call(0x001a_6f00, &[o, 0], &[qv]).1;
    m.wv(q, r);
    let (a0, b0) = (m.r32(a), m.r32(b));
    vc(m, o, 0x290, 0x294, &[7, p3, a0, b0, r[0].to_bits(), r[1].to_bits()], &[]);
    stack_pop(m, 0x40);
}
/// FUN_0019ea00
pub fn ee_sub_19ea00<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32) {
    let sp = stack_push(m, 0x20);
    let (p4, p3) = (m.r32(o + 0x10), m.r32(o + 0xc));
    m.call(0x001e_6f10, &[p4, p3, 0, sp], &[]);
    let s = m.rf(o + 0x14);
    let v = m.rv(sp);
    m.w32(p2 + 0x34, 0);
    m.wv(p2 + 0x20, [v[0] * s, v[1] * s, v[2] * s, v[3] * s]);
    stack_pop(m, 0x20);
}
/// FUN_0019ea60
pub fn ee_sub_19ea60<M: QMem + ?Sized>(m: &mut M, p: u32) {
    let i1 = m.r32(p + 0xc);
    if m.r32(i1 + 0x19c) == 0 {
        let i2 = if i1 == 0 { 0 } else { i1 + 400 };
        let (a, b) = (m.r32(p + 0x10), m.r32(p));
        m.call(0x0019_fd30, &[a, b + 0x9e0, i2], &[]);
    }
}
/// FUN_0019eaa0
pub fn ee_sub_19eaa0<M: QMem + ?Sized>(m: &mut M, o: u32, v2: V4, v3: V4) {
    m.wv(o + 0x40, v3);
    m.wv(o + 0x30, v2);
}
/// FUN_001a1098
pub fn ee_sub_1a1098<M: QMem + ?Sized>(m: &mut M, p: u32) {
    m.w32(p + 0x14, 7);
    let sp = stack_push(m, 0x10);
    m.w32(sp, 0x54);
    m.w32(sp + 4, 0x10);
    let r = m.call(0x001a_e238, &[sp, 0x002b_fa30], &[]).0;
    m.w32(p + 0x18, r);
    let n = m.r32(p + 0x14) as i32;
    for i in 0..n.max(0) {
        let e = m.r32(p + 0x18).wrapping_add(i as u32 * 0xc);
        let (w, k) = (m.r32(p), m.r32(e));
        let r = m.call(0x001f_4c58, &[w, k, 6], &[]).0;
        m.w32(e + 8, r);
    }
    stack_pop(m, 0x10);
}
/// FUN_001a2788: probe from the object's position (offset by DAT_002a2dd0) down/up by +0x1098 / +0x109c through
/// FUN_001f7e08 + FUN_0014cdb0; on a hit write the first hit record's quad + the offset to `out`.
/// inf: the quad moves (`_qmtc2`) of the decompile are read as the full position vector.
pub fn ee_sub_1a2788<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32, out: u32) -> bool {
    let c = m.rv(0x002a_2dd0);
    let pos = m.call(0x001a_62e8, &[o], &[]).1;
    let mut a = [pos[0] - c[0], pos[1] - c[1], pos[2] - c[2], pos[3] - c[3]];
    let mut b = a;
    a[2] += m.rf(o + 0x1098);
    b[2] += m.rf(o + 0x109c);
    let sp = stack_push(m, 0x6c0);
    let (rec, frame, ai) = (sp, sp + 0x600, sp + 0x610);
    m.call(0x001a_e478, &[frame], &[]);
    let n = m.r32(p2 + 0xe0) << 2;
    m.w32(ai, n);
    m.w32(ai + 4, 0x10);
    let objs = m.call(0x001a_e318, &[ai], &[]).0;
    let cnt = m.call(0x001f_7e08, &[p2, 0x0022_9d60, 0, objs], &[]).0 as i32;
    let l6 = ee_sub_14cdb0(m, b, a, cnt, objs, 0x10, rec);
    let r = m.rv(rec);
    if l6 != 0 {
        m.wv(out, [r[0] + c[0], r[1] + c[1], r[2] + c[2], r[3] + c[3]]);
    }
    m.call(0x001a_e4a0, &[frame, 2], &[]);
    stack_pop(m, 0x6c0);
    l6 != 0
}
/// FUN_001a2bb0 / 001a3000 / 001a3240: `init(1, 0xffff)` wrappers
pub fn ee_sub_1a2bb0<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_1a2a78(m, 1, 0xffff)
}
pub fn ee_sub_1a3000<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_1a2ec8(m, 1, 0xffff)
}
pub fn ee_sub_1a3240<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_1a3108(m, 1, 0xffff)
}
/// FUN_001a3020
pub fn ee_sub_1a3020<M: QMem + ?Sized>(m: &mut M, p: u32) {
    let l = m.call(0x0015_e938, &[0x003d_d490, 0x405, 0x373, 0], &[]).0;
    m.w32(p, l);
    if l != 0 {
        m.call(0x0021_2a50, &[l, 1], &[]);
    }
}
/// FUN_001a3070
pub fn ee_sub_1a3070<M: QMem + ?Sized>(m: &mut M, p: u32) {
    m.w32(p, 0);
}
/// FUN_001a3078
pub fn ee_sub_1a3078<M: QMem + ?Sized>(m: &mut M, p: u32) {
    let o = m.r32(p);
    if o != 0 {
        m.call(0x0022_7788, &[0x0044_35f0], &[]);
        vc(m, o, 0x188, 0x18c, &[0xffff_ffff, 0xffff_ffff], &[]);
    }
}
/// FUN_001a30d0
pub fn ee_sub_1a30d0<M: QMem + ?Sized>(m: &mut M, p: u32) {
    let o = m.r32(p);
    if o != 0 {
        vc(m, o, 400, 0x194, &[], &[]);
    }
}
/// FUN_001a3260: unless an object of kind 0 is already on the DAT_003e04f8 list (next +0x20), create the kind-4
/// light object (0x188) and give it the direction (-0.2, 0.3, -1.0, NaN-bits) matrix, then FUN_001a3610 /
/// FUN_001a3630 with 255.0 / 240.0. PARTIAL: the register layout of FUN_0011b830's arguments (quad / 64-bit) is
/// inferred from the pextlw / pcpyld packing.
pub fn ee_sub_1a3260<M: QMem + ?Sized>(m: &mut M) {
    let mut i = m.r32(0x003e_04f8);
    loop {
        if i == 0 {
            break;
        }
        if m.r32(i + 0x160) == 0 {
            return;
        }
        i = m.r32(i + 0x20);
    }
    let o = m.call(0x0013_e3c8, &[4, 0, 0x188, 0xffff_ffff, 0xffff_ffff], &[]).0;
    let v: V4 = [-0.2, 0.3, -1.0, f32::from_bits(0xffff_ffff)];
    let sp = stack_push(m, 0x80);
    let c = m.rv(0x002a_2dc0);
    m.call(0x0011_b830, &[sp + 0x40], &[v, c]);
    for k in 0..12 {
        let w = m.r32(sp + 0x40 + 4 * k);
        m.w32(sp + 4 * k, w);
    }
    vc(m, o, 0xf0, 0xf4, &[sp], &[]);
    m.call(0x001a_3610, &[0x437f_0000, o], &[]);
    m.call(0x001a_3630, &[0x4370_0000, o], &[]);
    stack_pop(m, 0x80);
}
fn is_tag<M: QMem + ?Sized>(m: &mut M, o: u32, tag: u32) -> bool {
    if m.call(0x001d_9880, &[], &[]).0 == 0 {
        return false;
    }
    let i = m.r32(o + 0x24) as i32;
    if i <= 0 {
        return false;
    }
    let mut p = o.wrapping_add((i as u32).wrapping_mul(0x1c)).wrapping_add(0xc);
    if m.r32(o + 0x1e8) == 1 && i < m.r32(o + 0x1ec) as i32 {
        p = o.wrapping_add(m.r32(o + 0x1ec).wrapping_mul(0x1c)).wrapping_add(0xc);
    }
    m.r32(p + 4) == tag
}
/// FUN_001b0160
pub fn ee_sub_1b0160<M: QMem + ?Sized>(m: &mut M, o: u32) -> bool {
    is_tag(m, o, 0x002b_fda8)
}
/// FUN_001b01e8
pub fn ee_sub_1b01e8<M: QMem + ?Sized>(m: &mut M, o: u32) -> bool {
    is_tag(m, o, 0x002b_fe08)
}
/// FUN_001b2800
pub fn ee_sub_1b2800<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_1b26c8(m, 1, 0xffff)
}
// FUN_001b2828 SKIPPED: endless PS2 decoder thread body (WaitSema(DAT_002f68ec) loop writing DMA buffers through
// FUN_0016f8b8 / FUN_0016f6b8 / FUN_0016f590); threads and semaphores have no engine counterpart.
/// FUN_001b29d0
pub fn ee_sub_1b29d0<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32) -> bool {
    let i2 = m.r32(p2 + 8);
    m.w32(o + 4, i2);
    let w = m.r32(p2 + 0xc);
    m.w32(o + 8, w);
    let i3 = m.r32(p2 + 0x10);
    m.w32(o + 0xc, i3);
    let w = m.r32(p2 + 0x14);
    m.w32(o + 0x10, w);
    let (u4, u5, u6);
    if i2 == 0x10 {
        u4 = 0x400;
        m.w32(o + 0x18, i3 << 12);
        u5 = 0x1000;
        u6 = 3;
    } else {
        u4 = 0x6000;
        m.w32(o + 0x18, 0x6000);
        u5 = 0;
        u6 = 2;
    }
    let r = m.call(0x0023_2ee8, &[u4, u5, 0x400, 0, 2, u6], &[]).0;
    m.w32(o + 0x14, r);
    let ok = (r as i32) >= 0;
    if ok {
        ee_sub_1b2a80(m, o);
    }
    ok
}
/// FUN_001b2a80
pub fn ee_sub_1b2a80<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.w32(o + 0x2c, 0);
    let w = m.r32(o + 0x18);
    m.w32(o + 0x24, w);
    m.w32(o + 0x28, 0);
    m.w32(o + 0x20, 0);
    m.w32(o + 0x1c, 0);
}
/// FUN_001b2aa0: de-interleave the 0x400-byte sectors of `p2` bytes at `p3` through a pool buffer
/// (inf: the memcpy length of the inner copy, absent from the decompile's argument list, is the sector size 0x400)
pub fn ee_sub_1b2aa0<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u64, p3: u32) {
    if m.r32(o + 4) != 0x10 || m.r32(o + 0xc) == 1 {
        return;
    }
    let sp = stack_push(m, 0x40);
    let (frame, ai) = (sp, sp + 0x10);
    m.call(0x001a_e478, &[frame], &[]);
    let i2 = m.r32(o + 0xc) as i32;
    let i8 = i2.wrapping_mul(0x400);
    m.w32(ai + 4, 0x10);
    m.w32(ai, i8 as u32);
    let buf = m.call(0x001a_e318, &[ai], &[]).0;
    let mut p3 = p3;
    if p2 != 0 {
        let mut i10 = (p2 as u32).wrapping_add((i2.wrapping_mul(-0x400)) as u32) as i32;
        loop {
            let (mut i7, mut i9, mut i6) = (0i32, 0i32, 0i32);
            if 0 < i8 {
                loop {
                    let i3 = m.r32(o + 0x10) as i32;
                    let i5 = i7 * 0x400;
                    i7 += 1;
                    let dst = buf.wrapping_add(i5 as u32).wrapping_add(i9.wrapping_mul(i3) as u32);
                    m.call(0x001a_e5a0, &[dst, p3.wrapping_add(i6 as u32), 0x400], &[]);
                    if (m.r32(o + 0xc) as i32) <= i7 {
                        i9 += 1;
                        i7 = 0;
                    }
                    i6 = i6.wrapping_add(m.r32(o + 0x10) as i32);
                    if i8 <= i6 {
                        break;
                    }
                }
            }
            m.call(0x001a_e5a0, &[p3, buf, i8 as u32], &[]);
            let more = i10 != 0;
            i10 = i10.wrapping_add(i2.wrapping_mul(-0x400));
            p3 = p3.wrapping_add(i8 as u32);
            if !more {
                break;
            }
        }
    }
    m.call(0x001a_e4a0, &[frame, 2], &[]);
    stack_pop(m, 0x40);
}
