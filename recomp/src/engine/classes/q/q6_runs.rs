//! WP-Q6: the small file (run_*) packages of the Q6 queue (one section per run, owner noted in the
//! heading). 1:1 against `QMem`; callees outside the run by address through `QMem::call`.
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem, V4};
use crate::engine::classes::q::q6::{callf, rov, s, vcall, CLOCK, WORLD};

// ---------------------------------------------------------------- run_0017de40 / run_0017e570 (guard AI)
/// FUN_0017de40: +0x30 == -1
pub fn ee_sub_17de40<M: QMem + ?Sized>(m: &mut M, o: u32) -> bool {
    m.r32(o + 0x30) == u32::MAX
}
/// FUN_0017e570: +0x38 == -1
pub fn ee_sub_17e570<M: QMem + ?Sized>(m: &mut M, o: u32) -> bool {
    m.r32(o + 0x38) == u32::MAX
}

// ---------------------------------------------------------------- run_001a15b0 / run_001a2358
/// FUN_001a15b0: 64-bit store +0x1c = v
pub fn ee_sub_1a15b0<M: QMem + ?Sized>(m: &mut M, o: u32, v: u64) {
    m.w32(o + 0x1c, v as u32);
    m.w32(o + 0x20, (v >> 32) as u32);
}
/// FUN_001a2358: virtual slot at vtable +0x2b4 of the object at *o
pub fn ee_sub_1a2358<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let x = m.r32(o);
    vcall(m, x, 0xc, 0x2b4, &[], &[]);
}

// ---------------------------------------------------------------- run_00131270
/// FUN_00131270: +0x18 if set else +0x14
pub fn ee_sub_131270<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    match m.r32(o + 0x18) {
        0 => m.r32(o + 0x14),
        x => x,
    }
}

// ---------------------------------------------------------------- run_0012f8c0 (JT animator)
/// FUN_0012f8c0: thunk JtAnimator_request(p[0], p[1], a0, 0, p[2], 0)
pub fn ee_sub_12f8c0<M: QMem + ?Sized>(m: &mut M, a0: u32, p: u32) {
    let (x, y, z) = (m.r32(p), m.r32(p + 4), m.r32(p + 8));
    m.call(0x0012_f9b8, &[x, y, a0, 0, z, 0], &[]);
}

// ---------------------------------------------------------------- run_00163430
/// FUN_00163430: n = (+0x64)->+0x54; if n > 1 and (+0x68)->+0x54 < 2 the result is 1
pub fn ee_sub_163430<M: QMem + ?Sized>(m: &mut M, o: u32) -> i32 {
    let a = m.r32(o + 0x64);
    let n = m.r32(a + 0x54) as i32;
    if n > 1 {
        let b = m.r32(o + 0x68);
        let k = m.r32(b + 0x54) as i32;
        if k < 2 {
            return 1;
        }
        return k;
    }
    n
}

// ---------------------------------------------------------------- run_001792e8 (guard AI)
/// FUN_001792e8: 1 when 0 <= +0x4c < 2 and +0xe0 < 0
pub fn ee_sub_1792e8<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let k = m.r32(o + 0x4c) as i32;
    (k >= 0 && k < 2 && !(0.0 <= m.rf(o + 0xe0))) as u32
}

// ---------------------------------------------------------------- run_0013c448
/// FUN_0013c448: append `v` to the 3-entry list of the +0x158 record (count +0x1c, entries +0x20)
pub fn ee_sub_13c448<M: QMem + ?Sized>(m: &mut M, o: u32, v: u32) {
    let r = m.r32(o + 0x158);
    let n = m.r32(r + 0x1c);
    if n < 3 {
        m.w32(r + n * 4 + 0x20, v);
        m.w32(r + 0x1c, n + 1);
    }
}

// ---------------------------------------------------------------- run_0015c650
/// FUN_0015c650: lazily create +0xf0 with FUN_001faf70()
pub fn ee_sub_15c650<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    if m.r32(o + 0xf0) == 0 {
        let v = m.call(0x001f_af70, &[], &[]).0;
        m.w32(o + 0xf0, v);
    }
    m.r32(o + 0xf0)
}

// ---------------------------------------------------------------- run_001974c8
/// FUN_001974c8: first non-zero head of the singly-linked list +0x64 ({value, next})
pub fn ee_sub_1974c8<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let mut p = m.r32(o + 0x64);
    if p == 0 {
        return 0;
    }
    loop {
        let v = m.r32(p);
        if v != 0 {
            return v;
        }
        p = m.r32(p + 4);
        if p == 0 {
            return 0;
        }
    }
}

// ---------------------------------------------------------------- run_001490c0
/// FUN_001490c0: FUN_001487a0(this + 0x150)
pub fn ee_sub_1490c0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0014_87a0, &[o + 0x150], &[]);
}
/// FUN_001490e0: FUN_00148860(this + 0x150)
pub fn ee_sub_1490e0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0014_8860, &[o + 0x150], &[]);
}

// ---------------------------------------------------------------- run_001f5ff8
/// FUN_001f5ff8: entry (stride 0x28, array +0xc0, count +0xb8) whose first word is `id`, else 0
pub fn ee_sub_1f5ff8<M: QMem + ?Sized>(m: &mut M, o: u32, id: u32) -> u32 {
    let n = m.r32(o + 0xb8) as i32;
    let mut p = m.r32(o + 0xc0);
    for _ in 0..n.max(0) {
        if m.r32(p) == id {
            return p;
        }
        p += 0x28;
    }
    0
}

// ---------------------------------------------------------------- run_0018d6f8 (JT)
/// FUN_0018d6f8: per-kind record of the JT: 0x1a -> +0x1fb0, 0x25 -> +0x1fbc, 0x26 -> +0x1fc8,
/// 0x29 -> +0x1fd4, else 0
pub fn ee_sub_18d6f8(o: u32, k: u32) -> u32 {
    match k {
        0x1a => o + 0x1fb0,
        0x25 => o + 0x1fbc,
        0x26 => o + 0x1fc8,
        0x29 => o + 0x1fd4,
        _ => 0,
    }
}

// ---------------------------------------------------------------- run_0012bb70 / run_0012c0e0 / run_0012c708
/// shared shape of the three key-search helpers: first key index i in [1, n-1) with key[i] > t (keys from
/// `first` with stride `st`), clamped to n - 1
fn key_seg<M: QMem + ?Sized>(m: &mut M, t: f32, c: u32, first: u32, st: u32) -> i32 {
    let mut i = 1;
    let n = m.r32(c + 4) as i32 - 1;
    let mut p = m.r32(c + 8) + first;
    if 1 < n && m.rf(p) <= t {
        i = 2;
        loop {
            p += st;
            if !(i < n && m.rf(p) <= t) {
                break;
            }
            i += 1;
        }
    }
    i
}
/// FUN_0012bb70: segment of `t` in a curve with 0x20-byte keys (time at +0xc)
pub fn ee_sub_12bb70<M: QMem + ?Sized>(m: &mut M, t: f32, c: u32) -> i32 {
    key_seg(m, t, c, 0x2c, 0x20)
}
/// FUN_0012c0e0: segment of `t` in a float key array
pub fn ee_sub_12c0e0<M: QMem + ?Sized>(m: &mut M, t: f32, c: u32) -> i32 {
    key_seg(m, t, c, 4, 4)
}
/// FUN_0012c708: same as FUN_0012c0e0 (second curve class)
pub fn ee_sub_12c708<M: QMem + ?Sized>(m: &mut M, t: f32, c: u32) -> i32 {
    key_seg(m, t, c, 4, 4)
}

// ---------------------------------------------------------------- run_00224590 (player state)
/// FUN_00224590: 0 < +0x150 < 3
pub fn ee_sub_224590<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let k = m.r32(o + 0x150) as i32;
    (k < 3 && k >= 1) as u32
}
/// FUN_00224610: +0x15c is 7 or 0xb
pub fn ee_sub_224610<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let k = m.r32(o + 0x15c);
    (k == 7 || k == 0xb) as u32
}

// ---------------------------------------------------------------- run_00161f48
/// FUN_00161f48: for `n` pairs {oid, arg}: object FUN_0015e938(world, 0x105, oid, ctx) -> FUN_00164640(obj, arg, ctx)
pub fn ee_sub_161f48<M: QMem + ?Sized>(m: &mut M, _a0: u32, n: i32, mut p: u32, ctx: u32) {
    let mut n = n;
    if n > 0 {
        while n != 0 {
            let id = m.r32(p);
            let x = m.call(0x0015_e938, &[WORLD, 0x105, id, ctx], &[]).0;
            if x != 0 {
                let a = m.r32(p + 4);
                m.call(0x0016_4640, &[x, a, ctx], &[]);
            }
            n -= 1;
            p += 8;
        }
    }
}

// ---------------------------------------------------------------- run_002026f8
/// FUN_002026f8: level string +0xc0 (FUN_0013e368(world, id)) or 0 when -1
pub fn ee_sub_2026f8<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let id = m.r32(o + 0xc0);
    if id == u32::MAX {
        return 0;
    }
    m.call(0x0013_e368, &[WORLD, id], &[]).0
}
/// FUN_00202730: level string +0xbc (FUN_0013e368(world, id)) or 0 when -1
pub fn ee_sub_202730<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let id = m.r32(o + 0xbc);
    if id == u32::MAX {
        return 0;
    }
    m.call(0x0013_e368, &[WORLD, id], &[]).0
}

// ---------------------------------------------------------------- run_001224a8 (Splice math BIFs)
/// FUN_001224a8: unary float BIF: the argument (kind 0x16 = float, else int converted; raw disassembly:
/// cvt.s.w only on the int branch) through the math table DAT_002bbb70[k], boxed with F32 (FUN_00129558)
/// into a temp, cloned into `out` (FUN_001291a8), temp released (FUN_00129ec8)
pub fn ee_sub_1224a8<M: QMem + ?Sized>(m: &mut M, out: u32, arg: u32, k: u32) -> u32 {
    let x = if m.r32(arg) == 0x16 { m.rf(arg + 4) } else { m.r32(arg + 4) as i32 as f32 };
    let f = m.r32(0x002b_bb70 + k * 4);
    let r = callf(m, f, &[], &[s(x)]);
    let t = stack_push(m, 0x10);
    m.w32(t, 0xffff_ffff);
    m.call(0x0012_9558, &[t], &[s(r)]);
    m.call(0x0012_91a8, &[out, t], &[]);
    m.call(0x0012_9ec8, &[t], &[]);
    stack_pop(m, 0x10);
    out
}

// ---------------------------------------------------------------- run_00206820
/// FUN_00206820: vt +0xdc (0); message vt +0x1c (0xf, 0, 1, &{&zero}, 0); vt +0xd4 (0);
/// message (0xe, 0, 1, &{&zero}, 0)
pub fn ee_sub_206820<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let t = stack_push(m, 0x10);
    m.w32(t, 0);
    m.w32(t + 4, t);
    vcall(m, o, 0xc, 0xdc, &[0], &[]);
    vcall(m, o, 0xc, 0x1c, &[0xf, 0, 1, t + 4, 0], &[]);
    vcall(m, o, 0xc, 0xd4, &[0], &[]);
    vcall(m, o, 0xc, 0x1c, &[0xe, 0, 1, t + 4, 0], &[]);
    stack_pop(m, 0x10);
}

// ---------------------------------------------------------------- run_001e6330
/// FUN_001e6330: +0x178 = 0, +0x148 = -1
pub fn ee_sub_1e6330<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.w32(o + 0x178, 0);
    m.w32(o + 0x148, 0xffff_ffff);
}
/// FUN_001e6728: `t` clamped to [FUN_001e6620(o), FUN_001e6648(o)] (lower bound returned when below, upper when
/// above); with +0xe4 set FUN_001e66c8()
pub fn ee_sub_1e6728<M: QMem + ?Sized>(m: &mut M, t: f32, o: u32) -> f32 {
    if m.r32(o + 0xe4) != 0 {
        return callf(m, 0x001e_66c8, &[], &[]);
    }
    let lo = callf(m, 0x001e_6620, &[o], &[]);
    let hi = callf(m, 0x001e_6648, &[o], &[]);
    if lo <= t {
        if t <= hi {
            return t;
        }
        return hi;
    }
    lo
}

// ---------------------------------------------------------------- run_0017a5b0 (guard AI)
/// FUN_0017a5b0: state +0x50 = k (+0x54 = (float)k); k 0: anim record (+0x40, kind 1) gets FUN_001f0660(+8,
/// 0x44) and +0x58 = clock + rand(owner +0xe7c, +0xe80); k 1 with kind 1: FUN_001f0660(+8, 0x43)
pub fn ee_sub_17a5b0<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32) {
    if k == m.r32(o + 0x50) {
        return;
    }
    m.w32(o + 0x50, k);
    m.wf(o + 0x54, k as i32 as f32);
    let r = m.r32(o + 0x40);
    if k == 0 {
        if m.r32(r) == 1 {
            let x = m.r32(r + 8);
            m.call(0x001f_0660, &[x, 0x44], &[]);
        }
        let ow = m.r32(o + 0x10);
        let c = m.rf(CLOCK);
        let (a, b) = (m.rf(ow + 0xe7c), m.rf(ow + 0xe80));
        let f = callf(m, 0x0011_d448, &[], &[s(a), s(b)]);
        m.wf(o + 0x58, c + f);
    } else if k == 1 && m.r32(r) == 1 {
        let x = m.r32(r + 8);
        m.call(0x001f_0660, &[x, 0x43], &[]);
    }
}

// ---------------------------------------------------------------- run_00175748 (guard AI)
/// FUN_00175748: +0x34 == -1
pub fn ee_sub_175748<M: QMem + ?Sized>(m: &mut M, o: u32) -> bool {
    m.r32(o + 0x34) == u32::MAX
}
/// FUN_00175810: +0x34 == -1 (second class)
pub fn ee_sub_175810<M: QMem + ?Sized>(m: &mut M, o: u32) -> bool {
    m.r32(o + 0x34) == u32::MAX
}
/// FUN_00175820: target filter: contact `c` (flags +0x10 bit 48 clear, its object (+4) ->+0xec ->+0x70 bit 2
/// clear); with +0x74 set the object must not be class 0x62; true when it is not class 0x33
pub fn ee_sub_175820<M: QMem + ?Sized>(m: &mut M, o: u32, c: u32) -> bool {
    if m.r64(c + 0x10) & 0x1_0000_0000_0000 != 0 {
        return false;
    }
    let x = m.r32(c + 4);
    let e = m.r32(x + 0xec);
    if m.r64(e + 0x70) & 4 != 0 {
        return false;
    }
    if m.r32(o + 0x74) != 0 && m.call(0x0013_2b10, &[x, 0x62], &[]).0 != 0 {
        return false;
    }
    let x = m.r32(c + 4);
    m.call(0x0013_2b10, &[x, 0x33], &[]).0 == 0
}

// ---------------------------------------------------------------- run_00124078 (Splice list builder)
/// FUN_00124078: Splice list of `n` Basic values from `p` (u32 each): n == 0 -> SetTag(out, 0); else nodes
/// from FUN_00256b48(), first wrapped with Pair (FUN_00129970) into a temp, the rest linked through
/// FUN_00256c58(prev + 0x10, node), Basic(node + 8, *p) (FUN_00129a10); temp cloned into `out`, released
pub fn ee_sub_124078<M: QMem + ?Sized>(m: &mut M, out: u32, n: i32, mut p: u32) -> u32 {
    if n == 0 {
        m.call(0x0012_a0b0, &[out, 0], &[]);
        return out;
    }
    let t = stack_push(m, 0x10);
    m.w32(t, 0xffff_ffff);
    let mut prev = 0u32;
    let mut n = n;
    if n > 0 {
        while n != 0 {
            let node = m.call(0x0025_6b48, &[], &[]).0;
            if prev == 0 {
                m.call(0x0012_9970, &[t, node], &[]);
            } else {
                m.call(0x0025_6c58, &[prev + 0x10, node], &[]);
            }
            let v = m.r32(p);
            p += 4;
            m.call(0x0012_9a10, &[node + 8, v], &[]);
            n -= 1;
            prev = node;
        }
    }
    m.call(0x0012_91a8, &[out, t], &[]);
    m.call(0x0012_9ec8, &[t], &[]);
    stack_pop(m, 0x10);
    out
}

// ---------------------------------------------------------------- run_002046b0 (heading helpers)
/// FUN_002046b0: heading of the direction quadword `d` (prot3w: atan2(d.y, d.x), f32_atan2 0x2458f0) applied
/// through FUN_00204660(this, angle)
pub fn ee_sub_2046b0<M: QMem + ?Sized>(m: &mut M, o: u32, d: V4) -> V4 {
    let a = callf(m, 0x0024_58f0, &[], &[s(d[1]), s(d[0])]);
    m.call(0x0020_4660, &[o], &[s(a)]).1
}
/// FUN_002046f0: FUN_002046b0(a, pos(b) - pos(a)) (positions FUN_001a62e8, vsub)
pub fn ee_sub_2046f0<M: QMem + ?Sized>(m: &mut M, a: u32, b: u32) {
    let pb = m.call(0x001a_62e8, &[b], &[]).1;
    let pa = m.call(0x001a_62e8, &[a], &[]).1;
    ee_sub_2046b0(m, a, [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2], pb[3] - pa[3]]);
}
/// FUN_00204740: atan2((+0x60)->+0x44, (+0x60)->+0x40)
pub fn ee_sub_204740<M: QMem + ?Sized>(m: &mut M, o: u32) -> f32 {
    let r = m.r32(o + 0x60);
    let (y, x) = (m.rf(r + 0x44), m.rf(r + 0x40));
    callf(m, 0x0024_58f0, &[], &[s(y), s(x)])
}

// ---------------------------------------------------------------- run_001fd5e8 (level sound bank)
/// FUN_001fd5e8: once (+0xe8 == 0): sound bank at disc location {sector (+0xa8 ^ +0xdc) + 0xca0, size +0xac ^
/// +0xe4} loaded with snd_BankLoadByLoc (0x231fb0) when size != 0, then the IOP flush (0x232188); every call:
/// each sub-bank i < +0xf8 whose status (+0xfc)[i] != 2 is marked 2 once FUN_002322a0(bank, i) reports it
pub fn ee_sub_1fd5e8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.r32(o + 0xe8) == 0 {
        let loc = (m.r32(o + 0xa8) ^ m.r32(o + 0xdc)).wrapping_add(0xca0);
        let size = m.r32(o + 0xac) ^ m.r32(o + 0xe4);
        if size != 0 {
            let b = m.call(0x0023_1fb0, &[loc, 0], &[]).0;
            m.w32(o + 0xe8, b);
        }
        m.call(0x0023_2188, &[], &[]);
    }
    let mut i = 0;
    while (i as i32) < m.r32(o + 0xf8) as i32 {
        let st = m.r32(o + 0xfc) + i * 4;
        if m.r32(st) != 2 {
            let b = m.r32(o + 0xe8);
            if m.call(0x0023_22a0, &[b, i], &[]).0 != 0 {
                let st = m.r32(o + 0xfc) + i * 4;
                m.w32(st, 2);
            }
        }
        i += 1;
    }
}

// ---------------------------------------------------------------- run_00117eb0 (BinaryInputStream readers)
/// FUN_00117eb0: read a 12-byte vector
pub fn ee_sub_117eb0<M: QMem + ?Sized>(m: &mut M, st: u32, dst: u32) {
    m.call(0x0011_7968, &[st, 0xc, dst], &[]);
}
/// FUN_00117ed0: read a 3x3 matrix (three 12-byte rows at stride 0x10)
pub fn ee_sub_117ed0<M: QMem + ?Sized>(m: &mut M, st: u32, dst: u32) {
    ee_sub_117eb0(m, st, dst);
    ee_sub_117eb0(m, st, dst + 0x10);
    ee_sub_117eb0(m, st, dst + 0x20);
}
/// FUN_00117f18: read a 4x4 transform (3x3 + translation row), w column = {0, 0, 0, 1}
pub fn ee_sub_117f18<M: QMem + ?Sized>(m: &mut M, st: u32, dst: u32) {
    ee_sub_117ed0(m, st, dst);
    ee_sub_117eb0(m, st, dst + 0x30);
    m.w32(dst + 0xc, 0);
    m.wf(dst + 0x3c, 1.0);
    m.w32(dst + 0x1c, 0);
    m.w32(dst + 0x2c, 0);
}
/// FUN_00117f90: read 12 bytes
pub fn ee_sub_117f90<M: QMem + ?Sized>(m: &mut M, st: u32, dst: u32) {
    m.call(0x0011_7968, &[st, 0xc, dst], &[]);
}
/// FUN_00117fb0: read 8 bytes
pub fn ee_sub_117fb0<M: QMem + ?Sized>(m: &mut M, st: u32, dst: u32) {
    m.call(0x0011_7968, &[st, 8, dst], &[]);
}

// ---------------------------------------------------------------- run_00172798 (guard search arc)
/// FUN_00172798: sample record `o` (16-byte rows): d = row[0x82] - row[0], FUN_0011efb0(d) -> {yaw, pitch,
/// dist}; when dist (FUN_0011d910 range 0x2a53c0) and pitch (range 0x2a53c8) pass and count (+0x14) < 0x40,
/// row[2 + 2n] = row[0x82], row[3 + 2n].x = wrapped yaw - +0x10 (FUN_0011cf60; negative -> 2pi - x, kept
/// as in the original); returns 1 when appended
pub fn ee_sub_172798<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let a = m.rv(o + 0x820);
    let b = m.rv(o);
    let t = stack_push(m, 0x10);
    m.call(0x0011_efb0, &[t, t | 4, t | 8], &[[a[0] - b[0], a[1] - b[1], a[2] - b[2], a[3] - b[3]]]);
    let (yaw, pitch, dist) = (m.rf(t), m.rf(t + 4), m.rf(t + 8));
    stack_pop(m, 0x10);
    if m.call(0x0011_d910, &[0x002a_53c0], &[s(dist)]).0 == 0 {
        return 0;
    }
    if m.call(0x0011_d910, &[0x002a_53c8], &[s(pitch)]).0 == 0 {
        return 0;
    }
    let n = m.r32(o + 0x14);
    if n >= 0x40 {
        return 0;
    }
    let row = o + (n * 2 + 2) * 0x10;
    m.w32(o + 0x14, n + 1);
    m.wv(row, a);
    let base = m.rf(o + 0x10);
    let f = callf(m, 0x0011_cf60, &[], &[s(yaw - base)]);
    m.wf(row + 0x10, f);
    if f < 0.0 {
        m.wf(row + 0x10, 6.283_185_5 - f);
    }
    1
}

// ---------------------------------------------------------------- run_001ba980 (guard alert / pools)
/// FUN_001ba980: +0x1090 (12 bytes) cleared, +0x108c = v, message {0xf} (0x70 bytes) to component 0x9c
pub fn ee_sub_1ba980<M: QMem + ?Sized>(m: &mut M, o: u32, v: u32) {
    m.call(0x0023_33b4, &[o + 0x1090, 0, 0xc], &[]);
    m.w32(o + 0x108c, v);
    let t = stack_push(m, 0x70);
    m.call(0x0023_33b4, &[t, 0, 0x70], &[]);
    m.w32(t, 0xf);
    let c = m.call(0x0020_5458, &[o, 0x9c], &[]).0;
    if c != 0 {
        m.call(0x0017_6750, &[c, t], &[]);
    }
    stack_pop(m, 0x70);
}
/// FUN_001ba9f8: with (on, 0xffff): bump the six pool refcounts DAT_002dd2e0..f4; on 0 -> 1 create the pool
/// (FUN_0011cca8(pool, elem size, count, align 0x10))
pub fn ee_sub_1ba9f8<M: QMem + ?Sized>(m: &mut M, on: u32, k: u32) {
    if k != 0xffff || on == 0 {
        return;
    }
    const P: [(u32, u32, u32); 6] =
        [(0x4520a8, 0x14, 0x2000), (0x4520d0, 0x18, 0x800), (0x4520f8, 0x28, 0x200), (0x452120, 0x1c, 0x800), (0x452148, 0xc, 200), (0x452170, 0x14, 0x100)];
    for (i, &(p, sz, n)) in P.iter().enumerate() {
        let c = 0x002d_d2e0 + 4 * i as u32;
        let v = m.r32(c).wrapping_add(1);
        m.w32(c, v);
        if v == 1 {
            m.call(0x0011_cca8, &[p, sz, n, 0x10], &[]);
        }
    }
}
/// FUN_001bab30: FUN_001ba9f8(1, 0xffff)
pub fn ee_sub_1bab30<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_1ba9f8(m, 1, 0xffff);
}

// ---------------------------------------------------------------- run_00178cd8 (guard AI)
/// FUN_00178cd8 (C2cc4c0 / C2d0728 slot 14): state +0x4c = k, start time +0x50 = clock (on change)
pub fn ee_sub_178cd8<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32) {
    if k != m.r32(o + 0x4c) {
        m.w32(o + 0x4c, k);
        let t = m.r32(CLOCK);
        m.w32(o + 0x50, t);
    }
}
/// FUN_00178d90: start following track `tr`: +0x38 = dir, +0x30 = tr; from the owner (+0x10) position:
/// with +0x48 >= 0 track_1e7380(tr, pos, 0x3dd020, 0, 0, 0, this + 0x34) and +0x48 = -1, else
/// track_1e6f60(tr, pos, 0x3dd020, 0, 0, 0, this + 0x34); an open track (+0xe4 == 0) sets +0x3c = 1, end
/// +0x44 = dir ? FUN_001e6620 : FUN_001e6648, FUN_001e6f10(end); closed: +0x3c = 0
pub fn ee_sub_178d90<M: QMem + ?Sized>(m: &mut M, o: u32, tr: u32, dir: u32) {
    m.w32(o + 0x38, dir);
    m.w32(o + 0x30, tr);
    let ow = m.r32(o + 0x10);
    let p = m.call(0x001a_62e8, &[ow], &[]).1;
    if 0.0 <= m.rf(o + 0x48) {
        m.call(0x001e_7380, &[tr, 0x003d_d020, 0, 0, 0, o + 0x34], &[p]);
        m.wf(o + 0x48, -1.0);
    } else {
        m.call(0x001e_6f60, &[tr, 0x003d_d020, 0, 0, 0, o + 0x34], &[p]);
    }
    if m.r32(tr + 0xe4) == 0 {
        m.w32(o + 0x3c, 1);
        let e = if m.r32(o + 0x38) == 0 { callf(m, 0x001e_6648, &[tr], &[]) } else { callf(m, 0x001e_6620, &[tr], &[]) };
        m.wf(o + 0x44, e);
        m.call(0x001e_6f10, &[tr], &[s(e)]);
    } else {
        m.w32(o + 0x3c, 0);
    }
}

// ---------------------------------------------------------------- run_0017aa78 (guard AI, VU0)
/// FUN_0017aa78 (raw disassembly; VU0 macro): d = pos(+0x10) - pos(+0x5c) with its component along the axis
/// DAT_002a2dc0 removed (dot over xyz, vmulx over xyzw), n = d / |d| (|d| < 1e-4 -> DAT_002a2da0);
/// *out_a = r[+0x10] + vf0 + n * r[+4] + axis * r[+8], *out_b = r[+0x20] + vf0 + axis * DAT_002be6c4
pub fn ee_sub_17aa78<M: QMem + ?Sized>(m: &mut M, o: u32, r: u32, out_a: u32, out_b: u32) {
    let ow = m.r32(o + 0x10);
    let p = m.call(0x001a_62e8, &[ow], &[]).1;
    let tg = m.r32(o + 0x5c);
    let q = m.call(0x001a_62e8, &[tg], &[]).1;
    let ax = rov(0x002a_2dc0);
    let mut d = [p[0] - q[0], p[1] - q[1], p[2] - q[2], p[3] - q[3]];
    let dot = d[0] * ax[0] + d[1] * ax[1] + d[2] * ax[2];
    for i in 0..4 {
        d[i] -= ax[i] * dot;
    }
    let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    let n = if len < f32::from_bits(0x38d1_b717) {
        rov(0x002a_2da0)
    } else {
        let q = 1.0 / len;
        [d[0] * q, d[1] * q, d[2] * q, d[3] * q]
    };
    const VF0: V4 = [0.0, 0.0, 0.0, 1.0];
    let (a, b) = (m.rf(r + 4), m.rf(r + 8));
    let r1 = m.rv(r + 0x10);
    let mut o1 = [0.0; 4];
    for i in 0..4 {
        o1[i] = r1[i] + VF0[i] + n[i] * a + ax[i] * b;
    }
    m.wv(out_a, o1);
    let c = m.rf(0x002b_e6c4);
    let r2 = m.rv(r + 0x20);
    let mut o2 = [0.0; 4];
    for i in 0..4 {
        o2[i] = r2[i] + VF0[i] + ax[i] * c;
    }
    m.wv(out_b, o2);
}

// ---------------------------------------------------------------- run_00177d20 (guard AI)
/// FUN_00177d20: owner (+0x10) +0xfa8 or +0xf94 set
pub fn ee_sub_177d20<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let ow = m.r32(o + 0x10);
    (m.r32(ow + 0xfa8) != 0 || m.r32(ow + 0xf94) != 0) as u32
}
/// FUN_00177d48: bit 60 of the owner's +0xf68 object flags (+0x28), 0 without one
pub fn ee_sub_177d48<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let ow = m.r32(o + 0x10);
    let x = m.r32(ow + 0xf68);
    if x == 0 {
        return 0;
    }
    ((m.r64(x + 0x28) >> 60) & 1) as u32
}
/// FUN_00177d70: owner is class 0x3c and its +0x108c is set
pub fn ee_sub_177d70<M: QMem + ?Sized>(m: &mut M, o: u32) -> bool {
    let ow = m.r32(o + 0x10);
    if m.call(0x0013_2b10, &[ow, 0x3c], &[]).0 == 0 {
        return false;
    }
    let ow = m.r32(o + 0x10);
    m.r32(ow + 0x108c) != 0
}
/// FUN_00177dc0: two AI goals are equal: same kind and, for kinds 2..6 / 0xd / 0xe / 0x12 the word [4],
/// for 9..0xb the word [7]; other kinds compare equal by kind alone
pub fn ee_sub_177dc0<M: QMem + ?Sized>(m: &mut M, a: u32, b: u32) -> bool {
    let k = m.r32(a);
    if k != m.r32(b) {
        return false;
    }
    let w = match k {
        2..=6 | 0xd | 0xe | 0x12 => 0x10,
        9..=0xb => 0x1c,
        _ => return true,
    };
    m.r32(a + w) == m.r32(b + w)
}

// ---------------------------------------------------------------- run_002317e0
/// FUN_002317e0: FUN_0013fb40(+0x10, a1, list) inside the +0x50 attach context (FUN_0022d670 / FUN_0022d698),
/// then each entry of `list` (count +8, array +0xc) gets its vt +0x12c slot with (this + 0x60)
pub fn ee_sub_2317e0<M: QMem + ?Sized>(m: &mut M, o: u32, a1: u32, list: u32) {
    let l = m.r32(o + 0x50);
    m.call(0x0022_d670, &[l], &[]);
    let x = m.r32(o + 0x10);
    m.call(0x0013_fb40, &[x, a1, list], &[]);
    let l = m.r32(o + 0x50);
    m.call(0x0022_d698, &[l], &[]);
    let mut i = 0;
    while (i as i32) < m.r32(list + 8) as i32 {
        let arr = m.r32(list + 0xc);
        let e = m.r32(arr + i * 4);
        vcall(m, e, 0xc, 0x12c, &[o + 0x60], &[]);
        i += 1;
    }
}
/// FUN_00231890: same body as FUN_001ba9f8 (pool refcounts, second copy)
pub fn ee_sub_231890<M: QMem + ?Sized>(m: &mut M, on: u32, k: u32) {
    ee_sub_1ba9f8(m, on, k);
}
/// FUN_002319c8: FUN_00231890(1, 0xffff)
pub fn ee_sub_2319c8<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_231890(m, 1, 0xffff);
}

// ---------------------------------------------------------------- run_0011d740 (math)
/// FUN_0011d740: double round-toward-zero of `x` (bits); when every dropped fraction bit is 1 the
/// truncated value is bumped by 1.0 (__adddf3 0x23d748) — the original's quirky ceil-ish rounding
pub fn ee_sub_11d740(x: u64) -> u64 {
    let e = ((x >> 52) as u32 & 0x7ff) as i32 - 0x3ff;
    if e < 0 {
        return 0;
    }
    if e < 0x34 {
        let f = x & 0xf_ffff_ffff_ffff;
        let mask = (1u64 << (0x34 - e)) - 1;
        let t = x & 0xfff0_0000_0000_0000 | f & !mask;
        if f & mask == mask {
            return (f64::from_bits(t) + 1.0).to_bits();
        }
        return t;
    }
    x
}
/// FUN_0011d7f0: float version of FUN_0011d740
pub fn ee_sub_11d7f0(x: f32) -> f32 {
    let b = x.to_bits();
    let e = ((b >> 23) & 0xff) as i32 - 0x7f;
    if e < 0 {
        return 0.0;
    }
    if e < 0x17 {
        let mask = (1u32 << (0x17 - e)) - 1;
        let f = b & 0x7f_ffff;
        let t = f32::from_bits(b & 0xff80_0000 | f & !mask);
        if f & mask == mask {
            return t + 1.0;
        }
        return t;
    }
    x
}
/// FUN_0011d8d8: quadratic through (0, a), (1, b), (n, c): out = {a, (b - a) - k, k} with
/// k = ((c - a) / n - (b - a)) / (n - 1)
pub fn ee_sub_11d8d8<M: QMem + ?Sized>(m: &mut M, a: f32, b: f32, n: f32, c: f32, out: u32) {
    m.wf(out, a);
    let k = ((c - a) / n - (b - a)) / (n - 1.0);
    m.wf(out + 8, k);
    m.wf(out + 4, (b - a) - k);
}
/// FUN_0011d978: angle `t` clamped to the arc {lo, hi} at `r`: mid = (lo + hi) / 2,
/// wrap(mid + FUN_0011cfe0(wrap(t - mid), (hi - lo) / 2)) (wrap = FUN_0011cf60)
pub fn ee_sub_11d978<M: QMem + ?Sized>(m: &mut M, t: f32, r: u32) -> f32 {
    let (lo, hi) = (m.rf(r), m.rf(r + 4));
    let mid = (hi + lo) * 0.5;
    let w = callf(m, 0x0011_cf60, &[], &[s(t - mid)]);
    let c = callf(m, 0x0011_cfe0, &[], &[s(w), s((hi - lo) * 0.5)]);
    callf(m, 0x0011_cf60, &[], &[s(mid + c)])
}

// ---------------------------------------------------------------- run_00149738 (C2cc950 feeds, +0x130)
/// FUN_00149738: FUN_001487a0(this + 0x130)
pub fn ee_sub_149738<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0014_87a0, &[o + 0x130], &[]);
}
/// FUN_00149758: FUN_00148860(this + 0x130)
pub fn ee_sub_149758<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0014_8860, &[o + 0x130], &[]);
}
/// FUN_00149778: C2cc950_feed_148af8(this + 0x130)
pub fn ee_sub_149778<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0014_8af8, &[o + 0x130], &[]);
}
/// FUN_00149798: C2cc950_feed_148b18(this + 0x130)
pub fn ee_sub_149798<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0014_8b18, &[o + 0x130], &[]);
}
/// FUN_00149808: C2cc950_feed_148b38(this + 0x130)
pub fn ee_sub_149808<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0014_8b38, &[o + 0x130], &[]);
}
/// FUN_00149828: C2cc950_feed_148b78(this + 0x130)
pub fn ee_sub_149828<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0014_8b78, &[o + 0x130], &[]);
}
/// FUN_00149848: FUN_00200f58(out, src +0x2c0, 0x2002, src +0x2c4); returns out
pub fn ee_sub_149848<M: QMem + ?Sized>(m: &mut M, out: u32, src: u32) -> u32 {
    let (a, b) = (m.r32(src + 0x2c0), m.r32(src + 0x2c4));
    m.call(0x0020_0f58, &[out, a, 0x2002, b], &[]);
    out
}
/// FUN_001498f8: same body as FUN_001ba9f8 (pool refcounts, third copy)
pub fn ee_sub_1498f8<M: QMem + ?Sized>(m: &mut M, on: u32, k: u32) {
    ee_sub_1ba9f8(m, on, k);
}
/// FUN_00149a30: FUN_001498f8(1, 0xffff)
pub fn ee_sub_149a30<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_1498f8(m, 1, 0xffff);
}

// ---------------------------------------------------------------- run_001f0438 (anim channel host)
/// FUN_001f0438: forwards to FUN_001f0400(a0, a1)
pub fn ee_sub_1f0438<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    m.call(0x001f_0400, &[a0, a1], &[]).0
}
/// FUN_001f0458: first pair {mask, value} (array +0x64, count +0x60) whose mask has a bit of `bits`
pub fn ee_sub_1f0458<M: QMem + ?Sized>(m: &mut M, o: u32, bits: u32) -> u32 {
    let n = m.r32(o + 0x60) as i32;
    let mut p = m.r32(o + 0x64);
    for _ in 0..n.max(0) {
        if m.r32(p) & bits != 0 {
            return p;
        }
        p += 8;
    }
    0
}
/// FUN_001f07e8: FUN_001f04f8(+0x24, +0x34)
pub fn ee_sub_1f07e8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let (a, b) = (m.r32(o + 0x24), m.r32(o + 0x34));
    m.call(0x001f_04f8, &[a, b], &[]);
}
/// FUN_001f0850 (raw disassembly: a0 = this, f12 = value): unless FUN_001f0638(this): lock (vt slot 0),
/// +0x44 = value, with an active channel (+0x30) FUN_00131a00(+0x30, value), unlock (vt slot 1)
pub fn ee_sub_1f0850<M: QMem + ?Sized>(m: &mut M, o: u32, f: f32) {
    if m.call(0x001f_0638, &[o], &[]).0 != 0 {
        return;
    }
    vcall(m, o, 0xc, 0xc, &[], &[]);
    let c = m.r32(o + 0x30);
    m.wf(o + 0x44, f);
    if c != 0 {
        m.call(0x0013_1a00, &[c], &[s(f)]);
    }
    vcall(m, o, 0xc, 0x14, &[], &[]);
}
/// FUN_001f08d0: retract the active channel +0x30 (cleared): FUN_00130d30(ch, +0x40 < 0 ? 0 : 2),
/// AsegaHost_retract (0x130b18), FUN_001f0fa0(this)
pub fn ee_sub_1f08d0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let c = m.r32(o + 0x30);
    if c == 0 {
        return;
    }
    m.w32(o + 0x30, 0);
    let k = if m.rf(c + 0x40) < 0.0 { 0 } else { 2 };
    m.call(0x0013_0d30, &[c, k], &[]);
    m.call(0x0013_0b18, &[c], &[]);
    m.call(0x001f_0fa0, &[o], &[]);
}
/// FUN_001f0948: unless FUN_001f0638(this): lock, +0x3c = FUN_001f0438(+0x24, a1), retract, FUN_001f0d88(this,
/// 1), unlock
pub fn ee_sub_1f0948<M: QMem + ?Sized>(m: &mut M, o: u32, a1: u32) {
    if m.call(0x001f_0638, &[o], &[]).0 != 0 {
        return;
    }
    vcall(m, o, 0xc, 0xc, &[], &[]);
    let h = m.r32(o + 0x24);
    let v = ee_sub_1f0438(m, h, a1);
    m.w32(o + 0x3c, v);
    ee_sub_1f08d0(m, o);
    m.call(0x001f_0d88, &[o, 1], &[]);
    vcall(m, o, 0xc, 0x14, &[], &[]);
}

// ---------------------------------------------------------------- run_0017cd00 (guard track follow)
/// shared body of FUN_0017cd00 / FUN_0017d058: state +0x30 = k (+0x34 = clock), FUN_001ef0d8(this + 0x18);
/// k 0 / 1 set the owner's component 0x9d mode (+0x3c) 9 / 2 and call its vt +0x44 slot; `clr` = word
/// cleared on k 1 (FUN_0017d058: +0x3c)
fn track_state<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32, clr: Option<u32>) {
    if k == m.r32(o + 0x30) {
        return;
    }
    m.call(0x001e_f0d8, &[o + 0x18], &[]);
    m.w32(o + 0x30, k);
    let t = m.r32(CLOCK);
    m.w32(o + 0x34, t);
    let mode = match k {
        0 => 9,
        1 => 2,
        _ => return,
    };
    let ow = m.r32(o + 0x10);
    let c = m.call(0x0020_5438, &[ow, 0x9d], &[]).0;
    m.w32(c + 0x3c, mode);
    vcall(m, c, 0xc, 0x44, &[], &[]);
    if k == 1 {
        if let Some(w) = clr {
            m.w32(o + w, 0);
        }
    }
}
/// FUN_0017cd00: track state (see `track_state`)
pub fn ee_sub_17cd00<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32) {
    track_state(m, o, k, None);
}
/// shared body of FUN_0017cdb8 / FUN_0017d110: target `w` = end of the owner's track (+0xf84; dir 0 ->
/// FUN_001e6620, else FUN_001e6648), owner +0xf8c = (+0xf88 < target)
fn track_target<M: QMem + ?Sized>(m: &mut M, o: u32, dir: u32, w: u32) {
    let ow = m.r32(o + 0x10);
    let tr = m.r32(ow + 0xf84);
    let v = if dir == 0 { callf(m, 0x001e_6620, &[tr], &[]) } else { callf(m, 0x001e_6648, &[tr], &[]) };
    m.wf(o + w, v);
    let ow = m.r32(o + 0x10);
    let lt = m.rf(ow + 0xf88) < m.rf(o + w);
    m.w32(ow + 0xf8c, lt as u32);
}
/// FUN_0017cdb8: track_target(+0x3c), state 0
pub fn ee_sub_17cdb8<M: QMem + ?Sized>(m: &mut M, o: u32, dir: u32) {
    track_target(m, o, dir, 0x3c);
    ee_sub_17cd00(m, o, 0);
}
/// FUN_0017ce30: |owner +0xf88 - +0x3c| < 50
pub fn ee_sub_17ce30<M: QMem + ?Sized>(m: &mut M, o: u32) -> bool {
    let ow = m.r32(o + 0x10);
    (m.rf(ow + 0xf88) - m.rf(o + 0x3c)).abs() < 50.0
}
/// FUN_0017d058: track state, k 1 also clears +0x3c
pub fn ee_sub_17d058<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32) {
    track_state(m, o, k, Some(0x3c));
}
/// FUN_0017d110: track_target(+0x38), state 0
pub fn ee_sub_17d110<M: QMem + ?Sized>(m: &mut M, o: u32, dir: u32) {
    track_target(m, o, dir, 0x38);
    ee_sub_17d058(m, o, 0);
}
/// FUN_0017d188: |owner +0xf88 - +0x38| < 50
pub fn ee_sub_17d188<M: QMem + ?Sized>(m: &mut M, o: u32) -> bool {
    let ow = m.r32(o + 0x10);
    (m.rf(ow + 0xf88) - m.rf(o + 0x38)).abs() < 50.0
}

// ---------------------------------------------------------------- run_0012cc68 (anim curves / channel pool)
/// FUN_0012cc68 (load): +4 = stream f32 (FUN_00117e38 returns in f0)
pub fn ee_sub_12cc68<M: QMem + ?Sized>(m: &mut M, o: u32, st: u32) {
    let v = callf(m, 0x0011_7e38, &[st], &[]);
    m.wf(o + 4, v);
}
/// FUN_0012ccb8 (load): u16 key count -> +4, keys (16 bytes each, FUN_001ae2a0(&{n * 16, 0x10})) -> +8:
/// {u16 frame / 60 (0.016666668), f32, f32, f32}
pub fn ee_sub_12ccb8<M: QMem + ?Sized>(m: &mut M, o: u32, st: u32) {
    let n = m.call(0x0011_7c20, &[st], &[]).0;
    let t = stack_push(m, 0x10);
    m.w32(t, n << 4);
    m.w32(t + 4, 0x10);
    m.w32(o + 4, n);
    let p = m.call(0x001a_e2a0, &[t], &[]).0;
    stack_pop(m, 0x10);
    m.w32(o + 8, p);
    let mut i = 0;
    while (i as i32) < m.r32(o + 4) as i32 {
        let k = m.r32(o + 8) + i * 0x10;
        i += 1;
        let fr = m.call(0x0011_7c20, &[st], &[]).0;
        m.wf(k, fr as i32 as f32 * 0.016_666_668);
        for j in 1..4 {
            let v = callf(m, 0x0011_7e38, &[st], &[]);
            m.wf(k + 4 * j, v);
        }
    }
}
/// FUN_0012ce50: ActPool_11ccc0(0x3e1254) (channel pool alloc)
pub fn ee_sub_12ce50<M: QMem + ?Sized>(m: &mut M) -> u32 {
    m.call(0x0011_ccc0, &[0x003e_1254], &[]).0
}
/// FUN_0012ce70: return `p` to the channel pool 0x3e1254 (FUN_0011cd98)
pub fn ee_sub_12ce70<M: QMem + ?Sized>(m: &mut M, p: u32) {
    m.call(0x0011_cd98, &[0x003e_1254, p], &[]);
}
/// FUN_0012ce98 (C2ca6e0 / C2ca7a0 / C2ca860 slot 0): FUN_002303a0(+4, +8, +0x14), FUN_0022c840(+4, +8)
pub fn ee_sub_12ce98<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let (a, b, c) = (m.r32(o + 4), m.r32(o + 8), m.r32(o + 0x14));
    m.call(0x0023_03a0, &[a, b, c], &[]);
    m.call(0x0022_c840, &[a, b], &[]);
}
/// FUN_0012cee0: channel `c` is current on its host (+8 -> +0x90): the current channel itself, or a kind-5
/// blend whose +0x1c is `c`
pub fn ee_sub_12cee0<M: QMem + ?Sized>(m: &mut M, c: u32) -> u32 {
    let h = m.r32(c + 8);
    let cur = m.r32(h + 0x90);
    if cur == c {
        return 1;
    }
    if cur == 0 {
        return 0;
    }
    if m.r32(cur) == 5 && m.r32(cur + 0x1c) == c {
        return 1;
    }
    0
}
/// FUN_0012cf88 (C2ca6a0 / C2ca760 / C2caf08 slot 5, clone): copy words 0, 3, 5 of `src`, +4 = a2, +8 = a3,
/// tail from +0x1c copied (FUN_001ae5a0, DAT_003e123c - 0x1c bytes)
pub fn ee_sub_12cf88<M: QMem + ?Sized>(m: &mut M, o: u32, src: u32, a2: u32, a3: u32) {
    let v = m.r32(src);
    m.w32(o, v);
    let v = m.r32(src + 0xc);
    m.w32(o + 0xc, v);
    let v = m.r32(src + 0x14);
    m.w32(o + 4, a2);
    m.w32(o + 0x14, v);
    m.w32(o + 8, a3);
    let n = m.r32(0x003e_123c).wrapping_sub(0x1c);
    m.call(0x001a_e5a0, &[o + 0x1c, src + 0x1c, n], &[]);
}
/// FUN_0012cfe0: unless FUN_0012d058(this): FUN_00230018(+4, +8, this)
pub fn ee_sub_12cfe0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.call(0x0012_d058, &[o], &[]).0 == 0 {
        let (a, b) = (m.r32(o + 4), m.r32(o + 8));
        m.call(0x0023_0018, &[a, b, o], &[]);
    }
}
/// FUN_0012d020: when FUN_0012d058(this): FUN_00230100(+4, +8, this)
pub fn ee_sub_12d020<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.call(0x0012_d058, &[o], &[]).0 != 0 {
        let (a, b) = (m.r32(o + 4), m.r32(o + 8));
        m.call(0x0023_0100, &[a, b, o], &[]);
    }
}
