//! WP-Q13 second part: file:run_001b26c8 / run_001b4140 (IOP sound-stream ring buffers), further runs below.
//! Hardware-only functions (WaitSema thread loop, sceSifSetDma) are skipped; their DB rows say why.
use super::q_00114680::*;
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem, V4};

/// FUN_001b2be0: push up to `want` bytes of the stream (+0x24 avail) in 0x400-multiples through FUN_001b2db8
/// (the SIF DMA); mode (+4) 0x10 patches the ADPCM loop flags every `+0xc`-th block (bytes 1/0x11 = 6/2,
/// byte 0x3f1 = 3). trap(7) on a zero divisor ends the call. 1b2db8's dest argument (a2) is `dst + done`.
pub fn ee_sub_1b2be0<M: QMem + ?Sized>(m: &mut M, this: u32, want: i32, dst: u32) -> i32 {
    let mut done: i32 = 0;
    let avail = m.r32(this + 0x24) as i32;
    let mode;
    let mut n: u32;
    if want < 1 || avail < want {
        n = avail as u32;
        mode = m.r32(this + 4);
    } else {
        mode = m.r32(this + 4);
        n = want as u32;
    }
    if mode == 0x10 {
        n &= (m.r32(this + 0xc) as i32).wrapping_mul(-0x400) as u32;
        m.call(0x001b_2aa0, &[this, n, dst], &[]);
    } else {
        n &= 0xffff_fc00;
    }
    if n == 0 {
        return done;
    }
    let mut cap = m.r32(this + 0x18) as i32;
    loop {
        let room = cap.wrapping_sub(m.r32(this + 0x1c) as i32);
        let mut k = n;
        if room <= n as i32 {
            k = room as u32;
        }
        let d = dst.wrapping_add(done as u32);
        if m.r32(this + 4) == 0x10 {
            let i4 = m.r32(this + 0x2c) as i32;
            k = 0x400;
            let i3 = if i4 >= 0 { i4 } else { i4.wrapping_add(0x3ff) };
            let div = m.r32(this + 0xc) as i32;
            if div == 0 {
                return done; // trap(7)
            }
            let q = (i3 >> 10) / div;
            let q3 = if q >= 0 { q } else { q + 3 };
            let r = q + (q3 >> 2) * -4;
            if r == 0 {
                m.w8(d + 1, 6);
                m.w8(d + 0x11, 2);
            } else if r == 3 {
                m.w8(d + 0x3f1, 3);
            }
        }
        let got = m.call(0x001b_2db8, &[this, k, d], &[]).0 as i32;
        n = n.wrapping_sub(got as u32);
        if got == 0 {
            break;
        }
        done = done.wrapping_add(got);
        let pos = (m.r32(this + 0x1c) as i32).wrapping_add(got);
        m.w32(this + 0x1c, pos as u32);
        let sz = m.r32(this + 0x18) as i32;
        if sz == 0 {
            return done; // trap(7)
        }
        for (o, s) in [(0x28u32, got), (0x24, -got), (0x2c, got)] {
            let v = (m.r32(this + o) as i32).wrapping_add(s);
            m.w32(this + o, v as u32);
        }
        m.w32(this + 0x1c, pos.wrapping_rem(sz) as u32);
        if want < 1 || n == 0 {
            return done;
        }
        cap = m.r32(this + 0x18) as i32;
    }
    done
}

/// FUN_001b2eb8: recompute the ring pointers (+0x20 write, +0x24 free, +0x28 filled) from the IOP position
/// (mode 0x10: FUN_00233018() * block size, else FUN_00232fc0() & 0xffffff)
pub fn ee_sub_1b2eb8<M: QMem + ?Sized>(m: &mut M, p: u32) {
    if m.r32(p + 4) == 0x10 {
        let pos = m.call(0x0023_3018, &[], &[]).0 as i32;
        let mut i2 = pos.wrapping_mul(m.r32(p + 0xc) as i32);
        if i2 < 0 {
            i2 = m.r32(p + 0x18) as i32;
        } else {
            let i1 = m.r32(p + 0x18) as i32;
            if i1 < i2 {
                i2 = i1;
            } else {
                if i1 == 0 {
                    return; // trap(7)
                }
                i2 = i2.wrapping_rem(i1);
                m.w32(p + 0x20, i2 as u32);
                if i2 == m.r32(p + 0x1c) as i32 {
                    m.w32(p + 0x24, i1 as u32);
                } else {
                    let v = i2.wrapping_sub(m.r32(p + 0x1c) as i32).wrapping_add(i1);
                    m.w32(p + 0x24, v as u32);
                    m.w32(p + 0x24, v.wrapping_rem(i1) as u32);
                }
                i2 = m.r32(p + 0x18) as i32;
            }
        }
        let f = i2.wrapping_sub(m.r32(p + 0x24) as i32);
        m.w32(p + 0x28, f as u32);
    } else {
        let u = m.call(0x0023_2fc0, &[], &[]).0;
        let i1 = m.r32(p + 0x18) as i32;
        let i4 = ((u & 0xff_ffff).wrapping_sub(m.r32(p + 0x14)) as i32)
            .wrapping_sub(m.r32(p + 0x1c) as i32)
            .wrapping_sub(0x400)
            .wrapping_add(i1);
        m.w32(p + 0x24, i4 as u32);
        if i1 == 0 {
            return; // trap(7)
        }
        let i2 = i4.wrapping_rem(i1);
        let i4 = i1.wrapping_sub(i2);
        m.w32(p + 0x24, i2 as u32);
        m.w32(p + 0x28, i4 as u32);
        let i2 = (m.r32(p + 0x1c) as i32).wrapping_sub(i4).wrapping_add(i1);
        m.w32(p + 0x20, i2 as u32);
        m.w32(p + 0x20, i2.wrapping_rem(i1) as u32);
    }
}

/// FUN_001b3130 (word-indexed fields: +0x30/+0x50 ring buffers, +0x70 sub-state)
pub fn ee_sub_1b3130<M: QMem + ?Sized>(m: &mut M, p: u32) {
    if m.r32(p) != 0 {
        m.call(0x0023_2f28, &[], &[]);
        m.w32(p, 1);
        m.w32(p + 4, 0xffff_ffff);
        ee_sub_13d580(m, p + 0x30);
        ee_sub_13d580(m, p + 0x50);
        m.call(0x001b_2a80, &[p + 0x70], &[]);
    }
}
/// FUN_001b3190
pub fn ee_sub_1b3190<M: QMem + ?Sized>(m: &mut M, p: u32) {
    m.w32(p, 0);
    m.call(0x0023_2f60, &[], &[]);
    let a = m.r32(p + 0xa0);
    m.call(0x0023_21e8, &[2, a], &[]);
    let b = m.r32(p + 0xa4);
    m.call(0x0023_21e8, &[0x10, b], &[]);
    m.call(0x0023_2468, &[0xffff_ffff, 0xffff_ffff], &[]);
}
/// FUN_001b31e0: feed a 4-byte-headed packet (header byte 3 = stream id) into the 0x30 / 0x50 ring
pub fn ee_sub_1b31e0<M: QMem + ?Sized>(m: &mut M, p: u32, len: i32, data: u32) -> u32 {
    if m.r32(p) == 0 {
        return 0;
    }
    let id = m.r8(data + 3) as u32;
    if m.r32(p + 4) == 0xffff_ffff {
        m.w32(p + 4, id);
    }
    if m.r32(p + 4) != id {
        return 1;
    }
    let mut len = len - 4;
    let mut data = data + 4;
    let st = m.r32(p);
    if st == 1 {
        let n = ee_sub_13d7d0(m, p + 0x30, len as u64, data);
        data = data.wrapping_add(n as u32);
        len -= n;
        if (m.r32(p + 0x40) as i32) < 1 {
            let r = m.call(0x001b_29d0, &[p + 0x70, p + 8], &[]).0;
            m.w32(p, if r == 0 { 0 } else { 2 });
        } else {
            return 1;
        }
    }
    if (m.r32(p + 0x60) as i32) < len {
        return 0;
    }
    ee_sub_13d7d0(m, p + 0x50, len as u64, data);
    1
}
/// FUN_001b3388
pub fn ee_sub_1b3388<M: QMem + ?Sized>(m: &mut M, _a: u32, msg: u32, o: u32) -> bool {
    let n = m.r32(msg + 0xc);
    let ok = n <= m.r32(o + 0x170);
    if ok {
        let src = m.r32(msg + 8);
        ee_sub_13d7d0(m, o + 0x160, n as u64, src);
    }
    ok
}
/// FUN_001b33c8
pub fn ee_sub_1b33c8<M: QMem + ?Sized>(m: &mut M, _a: u32, msg: u32, o: u32) {
    let (l, d) = (m.r32(msg + 0xc), m.r32(msg + 8));
    ee_sub_1b31e0(m, o + 0x180, l as i32, d);
}
/// FUN_001b33f0
pub fn ee_sub_1b33f0<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    loop {
        loop {
            let fill = m.r32(o + 0x174);
            let rb = o + 0x160;
            if 0xfff < fill as i32 {
                break;
            }
            let r = ee_sub_1b41d0(m, o, 1);
            if r == 2 {
                let sp = stack_push(m, 0x80);
                m.call(0x0023_33b4, &[sp, 0, 0x40], &[]);
                let f = m.r32(o + 0x174) as i32;
                let i3 = if f >= 0 { f } else { f + 0x1f };
                ee_sub_13d7d0(m, rb, (0x40 - (f + (i3 >> 5) * -0x20)) as u64, sp);
                tail(m, rb, sp);
                stack_pop(m, 0x80);
                return 1;
            }
            if r == 1 {
                m.call(0x001b_32b8, &[o + 0x180], &[]);
                break;
            }
        }
        // `goto LAB_001b3420` (after 1b32b8) or the full-buffer exit
        if m.r32(o + 0x174) as i32 > 0xfff {
            let sp = stack_push(m, 0x80);
            tail(m, o + 0x160, sp);
            stack_pop(m, 0x80);
            return 1;
        }
    }
}
fn tail<M: QMem + ?Sized>(m: &mut M, rb: u32, sp: u32) {
    m.w32(sp, 0x002d_4220);
    ee_sub_13d818(m, rb, 0);
    ee_sub_13d6a8(m, rb, 0, sp);
}
/// FUN_001b34d0
pub fn ee_sub_1b34d0<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    ee_sub_1b41d0(m, o, 0);
    m.call(0x001b_32b8, &[o + 0x180], &[]);
    1
}
/// FUN_001b3510: GIF upload of the +0x118 x +0x11c texture from the 0x285a80 + (+0x110) IOP page in 16x16
/// blocks (FUN_00166920/968 packet begin/end, 166bd8 tag, 166e90 A+D regs, 167150, 157500 DMA).
/// PARTIAL: the 64-bit register values of FUN_00166e90 are passed as a (lo, hi) pair (call ABI is 32-bit).
pub fn ee_sub_1b3510<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let (w, h) = (m.r32(o + 0x118) as i32, m.r32(o + 0x11c) as i32);
    let w16 = (if w >= 0 { w } else { w + 0xf }) >> 4;
    let h16 = (if h >= 0 { h } else { h + 0xf }) >> 4;
    let off = m.r32(o + 0x110);
    let mut src = 0x0028_5a80u32.wrapping_add(off);
    let sp = stack_push(m, 0xb0);
    m.call(0x0016_6920, &[sp], &[]);
    m.call(0x0015_7330, &[sp, 0x22d6, off.wrapping_add(0x3f_8e80)], &[]);
    m.call(0x0015_74b8, &[sp], &[]);
    m.call(0x0016_6bd8, &[sp, 0, 1, 0xe], &[]);
    let two = |m: &mut M, r: u32, v: u64| m.call(0x0016_6e90, &[sp, r, v as u32, (v >> 32) as u32], &[]);
    two(m, 0x50, 0xc_1800_0000_0000);
    two(m, 0x52, 0x10_0000_0010);
    let mut by = 0i64;
    while by < h16 as i64 {
        let mut bx = 0i32;
        while bx < w16 {
            m.call(0x0016_6bd8, &[sp, 0, 1, 0xe], &[]);
            two(m, 0x51, (by << 0x24 | ((bx << 4) as i64) << 0x30) as u64);
            bx += 1;
            two(m, 0x53, 0);
            m.call(0x0016_7150, &[sp, 0x40], &[]);
            let a = src & 0x0fff_ffff;
            src = src.wrapping_add(0x400);
            m.call(0x0015_7500, &[sp, 0x40, a], &[]);
        }
        by += 1;
    }
    m.call(0x0016_6bd8, &[sp, 0, 1, 0xe], &[]);
    two(m, 0x3f, 0);
    two(m, 0x61, 0);
    m.call(0x0016_6968, &[sp], &[]);
    m.call(0x0015_76b0, &[sp], &[]);
    m.call(0x0015_7348, &[sp, 0, 0], &[]);
    stack_pop(m, 0xb0);
}
// FUN_001b2828 SKIPPED: IOP-sync thread entry (WaitSema loop on DAT_002f68ec; PS2 RTOS semaphore).
// FUN_001b2db8 SKIPPED: sceSifSetDma / sceSifDmaStat busy-wait to the IOP (PS2 SDK).

/// FUN_001b4140
pub fn ee_sub_1b4140<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.w32(o + 0x114, 0);
    m.w32(o + 0x10c, 0);
    ee_sub_1b3130(m, o + 0x180);
    ee_sub_1b3190(m, o + 0x180);
    m.call(0x0024_b590, &[o + 0x118], &[]);
    m.call(0x0024_b530, &[o + 0x118], &[]);
    while m.r32(o + 0x240) != 0 {}
    m.w32(0x002d_edbc, 3);
}
/// FUN_001b41d0: 0 = got data, 1 = need more, 2 = end. PS2 FlushCache(0) dropped.
pub fn ee_sub_1b41d0<M: QMem + ?Sized>(m: &mut M, o: u32, block: u64) -> u32 {
    let s = m.r32(o + 0x114);
    let mut i3 = (m.r32(s + 0x1c) as i32).wrapping_sub(m.r32(s + 0x24) as i32);
    if i3 == 0 {
        loop {
            let s = m.r32(o + 0x114);
            if m.r32(s + 0x28) == 0 {
                return 2;
            }
            m.call(0x0013_2930, &[], &[]);
            let s = m.r32(o + 0x114);
            i3 = (m.r32(s + 0x1c) as i32).wrapping_sub(m.r32(s + 0x24) as i32);
            if i3 != 0 {
                break;
            }
            if block == 0 {
                return 0;
            }
        }
    }
    let s = m.r32(o + 0x114);
    let src = m.r32(s + 0x20).wrapping_add(m.r32(s + 0x24));
    let l = m.call(0x0024_f770, &[o + 0x118, src, i3 as u32], &[]).0;
    let s = m.r32(o + 0x114);
    m.call(0x0013_2998, &[s, l], &[]);
    if l != 0 {
        return 0;
    }
    if i3 <= m.r32(o + 0x170) as i32 {
        if (m.r32(o + 0x1e0) as i32) < i3 {
            return 1;
        }
        return 2;
    }
    1
}
/// FUN_001b42c0
pub fn ee_sub_1b42c0<M: QMem + ?Sized>(m: &mut M, o: u32) -> bool {
    m.r32(o + 0x114) != 0
}
/// FUN_001b42d0: the six-store initialiser plus the vtable store DAT_002f68a0 = 0x2d4240
pub fn ee_sub_1b42d0<M: QMem + ?Sized>(m: &mut M, p1: u64, p2: u64) {
    if p2 == 0xffff && p1 != 0 {
        m.w32(0x002f_68a0, 0x002d_4240);
        static_init_stores(m, p1, p2);
    }
}
/// FUN_001b4418
pub fn ee_sub_1b4418<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_1b42d0(m, 1, 0xffff)
}
/// FUN_001b44d8: gather the live (flag bit48 clear) 0xb0-byte entries of every list node (+0x20 chain from
/// DAT_003e0714, count +0xe0, array +0xe4) into a fresh array (FUN_001ae318 {size,align}); outs = (count, ptr)
pub fn ee_sub_1b44d8<M: QMem + ?Sized>(m: &mut M, out_n: u32, out_p: u32) {
    let sp = stack_push(m, 0x10);
    let head = m.r32(0x003e_0714);
    let mut tot = 0u32;
    let mut n = head;
    while n != 0 {
        tot = tot.wrapping_add(m.r32(n + 0xe0));
        n = m.r32(n + 0x20);
    }
    m.w32(sp, tot << 2);
    m.w32(sp + 4, 0x10);
    let base = m.call(0x001a_e318, &[sp], &[]).0;
    let mut cur = base;
    let mut node = head;
    while node != 0 {
        let cnt = m.r32(node + 0xe0) as i32;
        for i in 0..cnt.max(0) {
            let e = m.r32(node + 0xe4).wrapping_add(i as u32 * 0xb0);
            if m.r64(e + 0x10) & 0x1_0000_0000_0000 == 0 {
                m.w32(cur, e);
                cur += 4;
            }
        }
        node = m.r32(node + 0x20);
    }
    m.w32(out_n, (cur.wrapping_sub(base) as i32 >> 2) as u32);
    m.w32(out_p, base);
    stack_pop(m, 0x10);
}
/// FUN_001b4708
pub fn ee_sub_1b4708<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x001b_45d0, &[1, 0xffff], &[]);
}

// ---- run_001ba828 .. run_001cf518
/// `(**(vt + fn_off))(obj + (s16)(vt + adj_off), args..)`
pub fn vcl<M: QMem + ?Sized>(m: &mut M, obj: u32, adj_off: u32, fn_off: u32, a: &[u32], v: &[V4]) -> (u32, V4) {
    let vt = m.r32(obj + 0xc);
    let adj = m.r16(vt + adj_off) as i16 as i32 as u32;
    let f = m.r32(vt + fn_off);
    let mut args = vec![obj.wrapping_add(adj)];
    args.extend_from_slice(a);
    m.call(f, &args, v)
}
/// FUN_001ba828: switch the HUD state (+0x30): 0 / 1 / 2 spawn the matching overlay object (0xa9 / 0x9e / 0x9d)
pub fn ee_sub_1ba828<M: QMem + ?Sized>(m: &mut M, o: u32, v: u32) {
    if v == m.r32(o + 0x30) {
        return;
    }
    let w = m.r32(o + 0x10);
    if m.r32(o + 0x30) == 0 {
        let a = m.r32(o + 0x18);
        m.call(0x0017_1c18, &[a, w + 0x1090], &[]);
    }
    m.call(0x001e_f0d8, &[o + 0x18], &[]);
    m.w32(o + 0x30, v);
    let t = m.r32(0x002d_ded0);
    m.w32(o + 0x34, t);
    if v == 1 {
        let i = m.call(0x0020_5438, &[w, 0x9e], &[]).0;
        let g = m.r32(0x002e_1e40);
        m.w32(i + 0x38, g);
        vcl(m, i, 0x40, 0x44, &[], &[]);
    } else if (v as i32) < 2 {
        if v == 0 {
            let u = m.call(0x0020_5438, &[w, 0xa9], &[]).0;
            let a = m.r32(o + 0x38);
            m.call(0x0017_1258, &[u, a, 0, w + 0x1090], &[]);
            vcl(m, u, 0x40, 0x44, &[], &[]);
        }
    } else if v == 2 {
        let i = m.call(0x0020_5438, &[w, 0x9d], &[]).0;
        m.w32(i + 0x40, w + 0x1060);
        vcl(m, i, 0x40, 0x44, &[], &[]);
        m.call(0x0016_19a0, &[0x002d_eb30, 0x33, w], &[]);
    }
}
/// FUN_001c64f8
pub fn ee_sub_1c64f8<M: QMem + ?Sized>(m: &mut M, o: u32, a: u32, b: u32, c: u32) {
    m.w32(o + 0x1b4, c);
    m.w32(o + 0x1a4, a);
    m.w32(o + 0x1ac, b);
}
/// FUN_001c67a0: counter DAT_002deb30 += 1 (getter FUN_00161798 / setter FUN_001617e0)
pub fn ee_sub_1c67a0<M: QMem + ?Sized>(m: &mut M) {
    let v = m.call(0x0016_1798, &[0x002d_eb30], &[]).0;
    m.call(0x0016_17e0, &[0x002d_eb30, v.wrapping_add(1)], &[]);
}
/// FUN_001c6ab0: empty virtual (`jr ra`)
pub fn ee_sub_1c6ab0() {}
/// FUN_001c6bd0: init the effect object `o` (a1): activate, set tint/owner; `q` = a2 128-bit register stored
/// at +0x180, `delay` (f12) -> +0x1cc = now + delay unless it equals DAT_002a7758. The a1 128-bit register
/// passed to the vslot +0xec call is `o` itself (high half unknown).
pub fn ee_sub_1c6bd0<M: QMem + ?Sized>(m: &mut M, p1: u32, delay: f32, o: u32, q: V4) {
    vcl(m, o, 0x40, 0x44, &[], &[]);
    m.call(0x001a_58d0, &[o, 0, 0], &[]);
    vcl(m, o, 0xe8, 0xec, &[o], &[]);
    vcl(m, o, 0xf0, 0xf4, &[0x002a_2d00], &[]);
    for off in [0x58u32, 0x5c] {
        let i = m.r32(o + off);
        m.w32(i + 0xb4, 0);
        m.w32(i + 0xb8, 0);
        m.w32(i + 0xb0, 0);
    }
    m.w32(o + 0x160, p1);
    m.w32(o + 0x1c4, 0);
    vcl(m, o, 0x260, 0x264, &[3], &[]);
    let mut f = m.rf(0x002a_7758);
    let ne = delay != f;
    m.wv(o + 0x180, q);
    if ne {
        f = m.rf(0x002d_ded0) + delay;
    }
    m.wf(o + 0x1cc, f);
}
/// FUN_001c6ce8: take up to `max` free pooled objects from the pool of kind 0x5b (0x2b3) / 0x5d (0x2b4)
pub fn ee_sub_1c6ce8<M: QMem + ?Sized>(m: &mut M, w: u32, kind: i32, max: i32, out: u32) -> i32 {
    let mut n = 0;
    if max == 0 {
        return 0;
    }
    let key = if kind == 0x5b {
        0x2b3
    } else if kind == 0x5d {
        0x2b4
    } else {
        return 0;
    };
    let pool = m.call(0x0015_e938, &[w, 5, key, 0], &[]).0;
    if pool == 0 {
        return 0;
    }
    let mut out = out;
    while n < max {
        let o = m.call(0x001c_b250, &[pool, 0], &[]).0;
        if o == 0 {
            break;
        }
        n += 1;
        vcl(m, o, 0x260, 0x264, &[2], &[]);
        m.call(0x001c_b488, &[pool, o], &[]);
        m.w32(out, o);
        out += 4;
    }
    n
}
// FUN_001c6de8 SKIPPED: VU0 macro-mode ray set-up (lqc2/vmaddbc on DAT_002a2dc0) feeding FUN_0014cc58; the
// operand registers are not recoverable from the decompile.
// FUN_001cf6b0 SKIPPED: VU0 macro-mode matrix products (vmulabc/vmaddabc) building the particle basis.

/// FUN_001cd098: bind the attachment: rebuild the three effect handles (+0x94/98/9c) against object `o2`,
/// re-expressing the parent's world matrix relative to the mount (FUN_001cca40) when present.
/// PARTIAL: the first quad of the 0x40-byte copy fed to the matrix inverse is dropped by the decompile (taken
/// as the copy of the mount matrix); 64-bit register results of 22ca18 are passed as their low word.
pub fn ee_sub_1cd098<M: QMem + ?Sized>(m: &mut M, p: u32, o2: u32) {
    if o2 == m.r32(p + 0xa0) {
        return;
    }
    for off in [0x94u32, 0x98, 0x9c] {
        m.call(0x0012_e460, &[p + off], &[]);
    }
    let sp = stack_push(m, 0x140);
    let (a, b, m160, inv, buf) = (sp, sp + 0x40, sp + 0x80, sp + 0xc0, sp + 0x100);
    let w = m.r32(p + 0x34);
    let u = vcl(m, w, 0x100, 0x104, &[buf], &[]).0;
    let k = m.r32(p);
    vcl(m, o2, 0x50, 0x54, &[k, u, 0], &[]);
    let q = vcl(m, w, 0xf8, 0xfc, &[], &[]).0;
    for i in 0..16 {
        let x = m.r32(q + 4 * i);
        m.w32(m160 + 4 * i, x);
    }
    let l6 = m.call(0x001c_ca40, &[k, o2], &[]).0;
    if l6 != 0 {
        let u = vcl(m, l6, 0xf8, 0xfc, &[], &[]).0;
        m.call(0x001a_71b0, &[a, 0, o2, u], &[]);
        for i in 0..16 {
            let x = m.r32(a + 4 * i);
            m.w32(b + 4 * i, x);
        }
        m.call(0x0011_c318, &[inv, b], &[]);
        m.call(0x0011_c168, &[a, m160, inv], &[]);
        for i in 0..16 {
            let x = m.r32(a + 4 * i);
            m.w32(m160 + 4 * i, x);
        }
    }
    m.call(0x001a_6be8, &[o2, m160], &[]);
    if m.r32(o2 + 0xe8) != 0 {
        m.call(0x001f_5720, &[o2, 1], &[]);
        m.call(0x001f_57c0, &[o2, 1], &[]);
    }
    for (src, dst) in [(0x58u32, 0x98u32), (0x5c, 0x9c)] {
        let u5 = m.r32(o2 + src);
        let u2 = m.call(0x0022_ca18, &[u5, u5], &[]).0;
        let r = m.call(0x0012_dc18, &[o2, u5, 5, 0, 2, u2], &[]).0;
        m.w32(p + dst, r);
    }
    m.call(0x001f_7c68, &[0x3e80_0000, o2], &[]);
    m.w32(p + 0xa0, o2);
    stack_pop(m, 0x140);
}
/// FUN_001cd2c0
pub fn ee_sub_1cd2c0<M: QMem + ?Sized>(m: &mut M, p: u32) {
    if m.r32(p + 0xa0) != 0 {
        for off in [0x94u32, 0x98, 0x9c] {
            m.call(0x0012_e460, &[p + off], &[]);
        }
        m.w32(p + 0xa0, 0);
    }
}
/// FUN_001cd308: detach: release the handles, re-create two (5/0 and 5/7 flavours) at the target's position
/// (FUN_001a62e8) / basis (FUN_001a6318 -> 22fd38). PARTIAL: 64-bit/VU register results passed as low word.
pub fn ee_sub_1cd308<M: QMem + ?Sized>(m: &mut M, t: u32, o2: u32) {
    let at = m.r32(t + 0xa0);
    if at == 0 {
        return;
    }
    m.call(0x001c_cac0, &[t, 0xffff_ffff, 0xffff_ffff], &[]);
    for off in [0x94u32, 0x98, 0x9c] {
        m.call(0x0012_e460, &[t + off], &[]);
    }
    let sp = stack_push(m, 0xc0);
    let (mb, m70) = (sp, sp + 0x40);
    let i5;
    if m.r32(o2 + 0x10) == 0 {
        m.call(0x001a_58d0, &[at, 0, 0xf], &[]);
        i5 = m.r32(t + 0xa0);
    } else {
        let u = vcl(m, o2, 0x100, 0x104, &[mb], &[]).0;
        let k = m.r32(o2 + 0x10);
        vcl(m, at, 0x50, 0x54, &[k, u, 0xf], &[]);
        i5 = m.r32(t + 0xa0);
    }
    let i5 = m.r32(i5 + 0x58);
    let q = m.call(0x001a_62e8, &[o2], &[]).1;
    let tgt = m.r32(i5 + 0xa0);
    let r = m.call(0x0022_fc48, &[0, tgt, q[0].to_bits(), q[1].to_bits()], &[q]);
    let f = m.rf(0x002d_ded0) + m.rf(0x002c_1ef4);
    let c = m.rv(0x002a_2dd0);
    let at = m.r32(t + 0xa0);
    // FUN_0022fc48 returns a 64-bit (two floats); the decompile forwards the full auVar7._0_8_ to FUN_0012de58,
    // so pass both words (lo = r.0, hi = r.1[1]) rather than the low word only. [HARD2]
    let h = m.call(0x0012_de58, &[f.to_bits(), at, i5, 5, r.0, r.1[1].to_bits()], &[c]).0;
    m.w32(t + 0x94, h);
    let a0 = m.r32(t + 0xa0);
    let j = m.r32(a0 + 0x5c);
    let u = m.call(0x001a_6318, &[o2], &[]).0;
    let jt = m.r32(j + 0xa0);
    m.call(0x0022_fd38, &[m70, 0, jt, u], &[]);
    for i in 0..12 {
        let x = m.r32(m70 + 4 * i);
        m.w32(mb + 4 * i, x);
    }
    let u2 = m.call(0x0011_bff0, &[mb], &[]);
    let at = m.r32(t + 0xa0);
    // FUN_0011bff0 returns a 64-bit (two floats); the decompile forwards the full auVar7._0_8_ to FUN_0012dc18,
    // so pass both words (lo = u2.0, hi = u2.1[1]). [HARD2]
    let h = m.call(0x0012_dc18, &[at, j, 5, 7, 2, u2.0, u2.1[1].to_bits()], &[]).0;
    m.w32(t + 0x9c, h);
    
    let at = m.r32(t + 0xa0);
    m.w32(t + 0x84, at);
    m.call(0x001c_cac0, &[t, 1], &[]);
    m.w32(t + 0xa0, 0);
    stack_pop(m, 0xc0);
}
/// FUN_001cdcb0
pub fn ee_sub_1cdcb0<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x001c_db78, &[1, 0xffff], &[]);
}
/// FUN_001cdcd0: get (flag == 0: cached at world+0x406c) or create a pooled (0x1b0-byte) effect object
pub fn ee_sub_1cdcd0<M: QMem + ?Sized>(m: &mut M, w: u32, flag: u32) -> u32 {
    if flag == 0 {
        let c = m.r32(w + 0x406c);
        if c != 0 {
            return c;
        }
    }
    let p = m.call(0x0011_ccc0, &[w + 0x403c], &[]).0;
    if p == 0 {
        return 0;
    }
    m.call(0x0023_33b4, &[p, 0, 0x1b0], &[]);
    m.w32(p, 0);
    m.w32(p + 4, 0);
    m.w32(p + 0xc, 0x002d_9978);
    m.call(0x0016_7310, &[p + 0x80], &[]);
    m.w32(p + 0x150, flag);
    m.w32(p + 0xc, 0x002d_77d8);
    let u = m.call(0x0015_e3a8, &[w, 0x57], &[]).0;
    m.call(0x0011_a070, &[u, p], &[]);
    vcl(m, p, 0x38, 0x3c, &[], &[]);
    vcl(m, p, 0x58, 0x5c, &[], &[]);
    vcl(m, p, 0x78, 0x7c, &[], &[]);
    if flag == 0 {
        m.w32(w + 0x406c, p);
    }
    p
}
/// FUN_001cde70
pub fn ee_sub_1cde70<M: QMem + ?Sized>(m: &mut M, a: u32, b: u32) {
    let v = m.r32(b + 0xd4);
    m.w32(a + 0x158, v);
}
/// FUN_001cf518
pub fn ee_sub_1cf518<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let sp = stack_push(m, 0x30);
    for i in 0..12 {
        let x = m.r32(o + 0x40 + 4 * i);
        m.w32(sp + 4 * i, x);
    }
    let k = m.r32(o + 0x70);
    if m.call(0x001c_f338, &[o, k, sp], &[]).0 == 0 {
        m.call(0x001c_e980, &[o], &[]);
    }
    stack_pop(m, 0x30);
}
/// `*(*(a) + off)`
pub fn rr<M: QMem + ?Sized>(m: &mut M, a: u32, off: u32) -> u32 {
    let p = m.r32(a);
    m.r32(p + off)
}
