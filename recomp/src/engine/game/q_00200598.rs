//! WP-Q4: run 0x00200598..0x00201dc8 - script member setter (FUN_00200598: the descriptor's setter /
//! sub-object getter by vtable index or direct pointer), CRef list builders (FUN_00200f58 / 2013d8), the
//! double-buffered DMA packet chain at 0x3dd2c0 / 0x3dd310 (two 0x50-byte records {+0 tag base, +4 data
//! base, +8 {aligned size, data addr} x 8, +0x48 count, +0x4c state}, current DAT_002c3c5c, DMA channels
//! DAT_002de59c / DAT_002de5a0), and the binoc HUD setup.
//! 1:1 against `QMem` (classes::q::q2mem); CRef / list / DMA library callees go through `QMem::call` by
//! address (CRef: 0x129ec8 Release, 0x12a0b0 SetTag, 0x129658 assign, 0x1291a8 copy out).
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem, V4};
use crate::engine::game::q_001e9090::ee_sub_1ea410;

/// DAT_002c3c5c: current packet buffer (0 / 1)
pub const CUR: u32 = 0x002c_3c5c;
/// DAT_003dd2c0: the two packet buffer records (0x50 bytes each)
pub const BUFS: u32 = 0x003d_d2c0;
/// DAT_002de59c / DAT_002de5a0: DMA channel register blocks
pub const CH_A: u32 = 0x002d_e59c;
pub const CH_B: u32 = 0x002d_e5a0;
/// world
pub const WORLD: u32 = 0x003d_d490;

const RELEASE: u32 = 0x0012_9ec8;
const SET_TAG: u32 = 0x0012_a0b0;
const ASSIGN: u32 = 0x0012_9658;

fn w64<M: QMem + ?Sized>(m: &mut M, a: u32, v: u64) {
    m.w32(a, v as u32);
    m.w32(a + 4, (v >> 32) as u32);
}

fn s16<M: QMem + ?Sized>(m: &mut M, a: u32) -> i32 {
    m.r16(a) as i16 as i32
}

/// descriptor method triplet at d + b {+0 short this-adjust, +2 short vtable index (< 0: direct), +4 fn
/// pointer or short offset of the vtable pointer}: (fn, this)
fn method<M: QMem + ?Sized>(m: &mut M, obj: u32, d: u32, b: u32) -> (u32, u32) {
    let idx = s16(m, d + b + 2);
    let off = s16(m, d + b);
    if idx < 0 {
        (m.r32(d + b + 4), obj.wrapping_add(off as u32))
    } else {
        let vo = s16(m, d + b + 4);
        let vt = m.r32(obj.wrapping_add(vo as u32));
        let e = vt.wrapping_add((idx as u32).wrapping_mul(8)).wrapping_sub(8);
        let adj = s16(m, e);
        (m.r32(e + 4), obj.wrapping_add(adj.wrapping_add(off) as u32))
    }
}

/// FUN_00200598(ret, obj, desc, val): member write: desc +4 bit 0x2000 first swaps obj for the
/// sub-object getter (desc +0x20, arg 1); bit 0x10: ret = FUN_00201110(1, val, desc, obj + desc +0x10)
/// (field reference); else by kind desc +0: 1 float (val tag 0x16 or FUN_00129da8) in f12, 2 vector
/// *val +4, 5..8 / 0xd..0x11 / 0x13 / 0x14 / 0x17 int (tag 0x15 or FUN_00129d58), 0 / 4 / 9..0xb and
/// kinds with bit 0x1000 / 0x2000 the raw payload; the setter is the desc +0x10 method. ret = the
/// result ref (tag 0x24 without one)
pub fn ee_sub_200598<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, d: u32, val: u32) -> u32 {
    let s = stack_push(m, 0x40);
    let (r, t, t2) = (s, s + 0x10, s + 0x20);
    m.w32(r, u32::MAX);
    m.call(RELEASE, &[r], &[]);
    m.call(SET_TAG, &[r, 0x24], &[]);
    let mut obj = obj;
    let fl = m.r32(d + 4);
    if fl & 0x2000 != 0 {
        let (f, this) = method(m, obj, d, 0x20);
        obj = m.call(f, &[this, 1], &[]).0;
    }
    if fl & 0x10 != 0 {
        let p = obj.wrapping_add(m.r32(d + 0x10));
        m.w32(t2, p);
        m.call(0x0020_1110, &[t, 1, val, d, t2], &[]);
        m.call(ASSIGN, &[r, t], &[]);
        m.call(RELEASE, &[t], &[]);
    } else {
        let kind = m.r32(d);
        let conv = |m: &mut M, tag: u32, f: u32| -> u32 {
            if m.r32(val) == tag {
                m.r32(val + 4)
            } else {
                m.w32(t, u32::MAX);
                m.call(f, &[t2, val], &[]);
                m.call(ASSIGN, &[t, t2], &[]);
                m.call(RELEASE, &[t2], &[]);
                let v = m.r32(t + 4);
                m.call(RELEASE, &[t], &[]);
                v
            }
        };
        match kind {
            1 => {
                let v = conv(m, 0x16, 0x0012_9da8);
                let (f, this) = method(m, obj, d, 0x10);
                m.call(f, &[this], &[[f32::from_bits(v), 0.0, 0.0, 0.0]]);
            }
            2 => {
                let (f, this) = method(m, obj, d, 0x10);
                let p = m.r32(val + 4);
                let v: V4 = m.rv(p);
                m.call(f, &[this], &[v]);
            }
            5..=8 | 0xd..=0x11 | 0x13 | 0x14 | 0x17 => {
                let v = conv(m, 0x15, 0x0012_9d58);
                let (f, this) = method(m, obj, d, 0x10);
                m.call(f, &[this, v], &[]);
            }
            0 | 4 | 9..=0xb => {
                let (f, this) = method(m, obj, d, 0x10);
                let v = m.r32(val + 4);
                m.call(f, &[this, v], &[]);
            }
            k if k & 0x3000 != 0 => {
                let (f, this) = method(m, obj, d, 0x10);
                let v = m.r32(val + 4);
                m.call(f, &[this, v], &[]);
            }
            _ => {}
        }
    }
    m.call(0x0012_91a8, &[ret, r], &[]);
    m.call(RELEASE, &[r], &[]);
    stack_pop(m, 0x40);
    ret
}

fn node_release<M: QMem + ?Sized>(m: &mut M, n: u32) {
    if n == 0 {
        return;
    }
    let c = m.r32(n).wrapping_sub(1);
    m.w32(n, c);
    if c == 0 {
        m.call(0x0025_6d40, &[n + 0x10, 2], &[]);
        m.call(RELEASE, &[n + 8], &[]);
        m.call(0x0025_50f8, &[n], &[]);
        m.call(0x0011_cd98, &[0x0045_20a8, n], &[]);
    }
}

/// FUN_00200f58(ret, n, kind, base): list ref of n values read by FUN_001ffd98 from base with the method
/// arg {kind, 2, i * 4} (nodes FUN_00256b48, {+0 refs, +8 value, +0x10 next}); tag 0 for n = 0
pub fn ee_sub_200f58<M: QMem + ?Sized>(m: &mut M, ret: u32, n: u32, kind: u32, base: u32) -> u32 {
    if n == 0 {
        m.call(SET_TAG, &[ret, 0], &[]);
        return ret;
    }
    let s = stack_push(m, 0x60);
    let (arg, head, nodep, tmp) = (s, s + 0x30, s + 0x40, s + 0x50);
    m.w32(head, 0);
    m.call(0x0023_33b4, &[arg, 0, 0x28], &[]);
    m.w32(arg, kind);
    m.w32(arg + 4, 2);
    m.call(0x0025_6c58, &[head, 0], &[]);
    let mut slot = head;
    let mut i = 0;
    while i < n as i32 {
        m.w32(arg + 8, (i as u32) << 2);
        let nd = m.call(0x0025_6b48, &[], &[]).0;
        m.call(0x0025_6c38, &[nodep, nd], &[]);
        let node = m.r32(nodep);
        m.call(0x001f_fd98, &[tmp, base, arg], &[]);
        m.call(ASSIGN, &[node + 8, tmp], &[]);
        m.call(RELEASE, &[tmp], &[]);
        let node = m.r32(nodep);
        m.call(0x0025_6c58, &[slot, node], &[]);
        slot = node + 0x10;
        node_release(m, node);
        i += 1;
    }
    let h = m.r32(head);
    m.call(0x0012_9480, &[ret, h], &[]);
    node_release(m, h);
    stack_pop(m, 0x60);
    ret
}

/// FUN_002013d8(ret, list, kind, _, count, out): unpack every list node (+4 first, next +0x10, value
/// +8) with FUN_00201360(tmp, 1, value, {kind}, out + count * 4), *count = nodes; ret tag 0x24
pub fn ee_sub_2013d8<M: QMem + ?Sized>(m: &mut M, ret: u32, list: u32, kind: u32, _p4: u32, cnt: u32, out: u32) -> u32 {
    let s = stack_push(m, 0x30);
    m.w32(s + 0x20, kind);
    m.w32(cnt, 0);
    if m.r32(list) != 0 {
        let mut nd = m.r32(list + 4);
        while nd != 0 {
            m.w32(s, u32::MAX);
            let o = out.wrapping_add(m.r32(cnt).wrapping_mul(4));
            m.call(0x0020_1360, &[s + 0x10, 1, nd + 8, s + 0x20, o], &[]);
            m.call(ASSIGN, &[s, s + 0x10], &[]);
            m.call(RELEASE, &[s + 0x10], &[]);
            let c = m.r32(cnt);
            m.w32(cnt, c + 1);
            m.call(RELEASE, &[s], &[]);
            nd = m.r32(nd + 0x10);
        }
    }
    m.call(SET_TAG, &[ret, 0x24], &[]);
    stack_pop(m, 0x30);
    ret
}

/// FUN_002014c8 (static pool init) = FUN_001ea410
pub fn ee_sub_2014c8<M: QMem + ?Sized>(m: &mut M, init: u32, prio: u32) {
    ee_sub_1ea410(m, init, prio);
}
/// FUN_00201600: (1, 0xffff)
pub fn ee_sub_201600<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_1ea410(m, 1, 0xffff);
}

/// FUN_00201620(pairs): fill the back buffer (1 - cur): wait channel A when it is in state 3, state 1;
/// for each (size, addr) pair up to size 0: record {size aligned 16, data offset + 0x10} (offsets
/// accumulate from +4), REF tag (qwc | addr << 32 | 0x30000000 when another addr follows) at tag base +
/// 16 i when addr != 0; count +0x48; channel B CHCR |= 0x40, TADR = data base | 0x80000000,
/// FUN_00241728(B, tag base)
pub fn ee_sub_201620<M: QMem + ?Sized>(m: &mut M, pairs: &[u32]) {
    let cur = m.r32(CUR);
    let b = BUFS + (1u32.wrapping_sub(cur)).wrapping_mul(0x50);
    let mut off = m.r32(b + 4);
    if m.r32(b + 0x4c) == 3 {
        let ch = m.r32(CH_A);
        m.call(0x0024_17f0, &[ch, 0, 0], &[]);
    }
    m.w32(b + 0x4c, 1);
    let at = |k: usize| pairs.get(k).copied().unwrap_or(0);
    let mut i = 0u32;
    let mut k = 0usize;
    let mut size = at(0);
    let mut addr = at(1);
    while size != 0 {
        let (nsize, naddr) = (at(k + 2), at(k + 3));
        let al = size.wrapping_add(0xf) & !0xf;
        let r = b + 8 + i * 8;
        off = off.wrapping_add(0x10);
        m.w32(r + 4, off);
        m.w32(r, al);
        off = off.wrapping_add(al);
        if addr != 0 {
            let mut t = ((al as i32 >> 4) as i64 as u64) | ((addr as u64) << 32);
            if naddr != 0 {
                t |= 0x3000_0000;
            }
            let tb = m.r32(b);
            w64(m, tb.wrapping_add(i * 16), t);
        }
        k += 2;
        size = nsize;
        addr = naddr;
        i += 1;
    }
    m.w32(b + 0x48, i);
    let ch = m.r32(CH_B);
    let c = m.r32(ch);
    m.w32(ch, c | 0x40);
    let v = (m.r32(b + 4) & 0x0fff_ffff) | 0x8000_0000;
    m.w32(ch + 0x80, v);
    let tb = m.r32(b) & 0x0fff_ffff;
    m.call(0x0024_1728, &[ch, tb], &[]);
}

/// FUN_002017a0(addrs): kick the current buffer: state 3, wait channel A; the last n records (n = the
/// non-zero addrs) get CNT tags (qwc | addr << 32 | 0x10000000) at data - 0x10, the final one END
/// (0x70000000); channel A CHCR &= ~0x40, TADR = data base | 0x80000000, FUN_002417a0(A); the back
/// buffer's state 3 -> 0
pub fn ee_sub_2017a0<M: QMem + ?Sized>(m: &mut M, addrs: &[u32]) {
    let ch = m.r32(CH_A);
    let cur = m.r32(CUR);
    let b = BUFS + cur.wrapping_mul(0x50);
    m.w32(b + 0x4c, 3);
    m.call(0x0024_17f0, &[ch, 0, 0], &[]);
    let list: Vec<u32> = addrs.iter().copied().take_while(|&a| a != 0).collect();
    let cnt = m.r32(b + 0x48) as i32;
    let mut i = cnt - list.len() as i32;
    let mut j = 0usize;
    let tag = |m: &mut M, i: i32, id: u64, a: u32| {
        let r = b + 8 + (i as u32).wrapping_mul(8);
        let size = m.r32(r) as i32;
        let off = m.r32(r + 4);
        let t = ((size >> 4) as i64 as u64) | id | ((a as i32 as i64 as u64) << 32);
        w64(m, off.wrapping_sub(0x10), t);
    };
    while i < cnt - 1 {
        tag(m, i, 0x1000_0000, list.get(j).copied().unwrap_or(0));
        i += 1;
        j += 1;
    }
    tag(m, i, 0x7000_0000, list.get(j).copied().unwrap_or(0));
    let ch = m.r32(CH_A);
    let c = m.r32(ch);
    m.w32(ch, c & !0x40);
    let v = (m.r32(b + 4) & 0x0fff_ffff) | 0x8000_0000;
    m.w32(ch + 0x80, v);
    m.call(0x0024_17a0, &[ch], &[]);
    let o = BUFS + (1u32.wrapping_sub(m.r32(CUR))).wrapping_mul(0x50);
    if m.r32(o + 0x4c) == 3 {
        m.w32(o + 0x4c, 0);
    }
}

/// FUN_00201998: wait channel B, flip DAT_002c3c5c, the new current buffer's state = 2
pub fn ee_sub_201998<M: QMem + ?Sized>(m: &mut M) {
    let ch = m.r32(CH_B);
    m.call(0x0024_17f0, &[ch, 0, 0], &[]);
    let c = 1u32.wrapping_sub(m.r32(CUR));
    m.w32(CUR, c);
    m.w32(BUFS + c.wrapping_mul(0x50) + 0x4c, 2);
}

/// FUN_002019f0(init, prio): static init: buffer 0 {0x203dd380, 0x70000000, 0..}, buffer 1 {0x203dd400,
/// 0x70002000, 0..}, then the pool counters (FUN_001ea410)
pub fn ee_sub_2019f0<M: QMem + ?Sized>(m: &mut M, init: u32, prio: u32) {
    if prio == 0xffff && init != 0 {
        for (b, t, d) in [(BUFS, 0x203d_d380u32, 0x7000_0000u32), (BUFS + 0x50, 0x203d_d400, 0x7000_2000)] {
            for k in 0..0x14 {
                m.w32(b + 4 * k, 0);
            }
            m.w32(b, t);
            m.w32(b + 4, d);
        }
    }
    ee_sub_1ea410(m, init, prio);
}
/// FUN_00201d58: (1, 0xffff)
pub fn ee_sub_201d58<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_2019f0(m, 1, 0xffff);
}

/// FUN_00201d78(p, v): +0x10 = 320.0, +0x14 = 246.4 (0x43766667), +0x18 = 0, +0xa8 = v, +0 = -1
pub fn ee_sub_201d78<M: QMem + ?Sized>(m: &mut M, p: u32, v: u32) {
    m.w32(p + 0x10, 0);
    m.w32(p + 0xa8, v);
    m.w32(p + 0x14, 0);
    m.w32(p + 0x18, 0);
    let x = m.rf(p + 0x10) * 320.0 + 320.0;
    m.wf(p + 0x10, x);
    let y = m.r32(p + 0x14);
    m.w32(p + 0x18, y);
    m.w32(p + 0x14, 0x4376_6667);
    m.w32(p, u32::MAX);
}

/// FUN_00201dc8(h): binoc HUD links: class-0x35 owner (**(+0xa8)) -> +0xac / +0xb0 = objects 0x362 /
/// 0x365; +0xa4 = 0x602; +0x30 = FUN_0015e9c8(world, 0x405, 0x2e, 0, 0x10, +0x34) (count); FUN_00201e88(h)
pub fn ee_sub_201dc8<M: QMem + ?Sized>(m: &mut M, h: u32) {
    let op = m.r32(h + 0xa8);
    let o = m.r32(op);
    if m.call(0x0013_2b10, &[o, 0x35], &[]).0 != 0 {
        let a = m.call(0x0015_e938, &[WORLD, 0x405, 0x362, 0], &[]).0;
        m.w32(h + 0xac, a);
        let a = m.call(0x0015_e938, &[WORLD, 0x405, 0x365, 0], &[]).0;
        m.w32(h + 0xb0, a);
    }
    let a = m.call(0x0015_e938, &[WORLD, 0x405, 0x602, 0], &[]).0;
    m.w32(h + 0xa4, a);
    let n = m.call(0x0015_e9c8, &[WORLD, 0x405, 0x2e, 0, 0x10, h + 0x34], &[]).0;
    m.w32(h + 0x30, n);
    m.call(0x0020_1e88, &[h], &[]);
}
