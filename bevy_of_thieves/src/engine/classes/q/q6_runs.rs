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
    m.call(0x0012_f9b8, &[a0, 0, z, 0], &[super::q5util::f4(f32::from_bits(x)), super::q5util::f4(f32::from_bits(y))]);
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
        return callf(m, 0x001e_66c8, &[o], &[s(t)]); // asm: a0 = o and $f12 = t passed through untouched
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
/// FUN_00178d90 (PART-R5: a1 = pos qword slot, was missing): start following track `tr`: +0x38 = dir, +0x30 = tr; from the owner (+0x10) position:
/// with +0x48 >= 0 track_1e7380(tr, pos, 0x3dd020, 0, 0, 0, this + 0x34) and +0x48 = -1, else
/// track_1e6f60(tr, pos, 0x3dd020, 0, 0, 0, this + 0x34); an open track (+0xe4 == 0) sets +0x3c = 1, end
/// +0x44 = dir ? FUN_001e6620 : FUN_001e6648, FUN_001e6f10(end); closed: +0x3c = 0
pub fn ee_sub_178d90<M: QMem + ?Sized>(m: &mut M, o: u32, tr: u32, dir: u32) {
    m.w32(o + 0x38, dir);
    m.w32(o + 0x30, tr);
    let ow = m.r32(o + 0x10);
    let p = m.call(0x001a_62e8, &[ow], &[]).1;
    if 0.0 <= m.rf(o + 0x48) {
        m.call(0x001e_7380, &[tr, 0, 0x003d_d020, 0, 0, 0, o + 0x34], &[p]);
        m.wf(o + 0x48, -1.0);
    } else {
        m.call(0x001e_6f60, &[tr, 0, 0x003d_d020, 0, 0, 0, o + 0x34], &[p]);
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
pub fn ee_sub_1f07e8<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let (a, b) = (m.r32(o + 0x24), m.r32(o + 0x34));
    m.call(0x001f_04f8, &[a, b], &[]).0
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
    { let f40 = m.rf(c + 0x40); m.call(0x0013_0d30, &[c, k], &[super::q5util::f4(0.0), super::q5util::f4(f40)]); }
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

// ---------------------------------------------------------------- run_00170b40 (path command lists)
// Command node builders over FUN_00170e00(list, kind, arg) (node: +0 kind, +4 arg, +8.. payload, +0x18 next)
fn cmd<M: QMem + ?Sized>(m: &mut M, l: u32, k: u32, a: u32) -> u32 {
    m.call(0x0017_0e00, &[l, k, a], &[]).0
}
fn copy64<M: QMem + ?Sized>(m: &mut M, dst: u32, src: u32) {
    let (a, b) = (m.r32(src), m.r32(src + 4));
    m.w32(dst, a);
    m.w32(dst + 4, b);
}
/// FUN_00170b40: kind 0 node (arg)
pub fn ee_sub_170b40<M: QMem + ?Sized>(m: &mut M, l: u32, a: u32) {
    cmd(m, l, 0, a);
}
/// FUN_00170b60: kind 1 node (arg -1), +8 = the 8 bytes at `p`
pub fn ee_sub_170b60<M: QMem + ?Sized>(m: &mut M, l: u32, p: u32) {
    let n = cmd(m, l, 1, u32::MAX);
    copy64(m, n + 8, p);
}
/// FUN_00170ba0: kind 2 node (arg -1), +8 = the 8 bytes at `p`
pub fn ee_sub_170ba0<M: QMem + ?Sized>(m: &mut M, l: u32, p: u32) {
    let n = cmd(m, l, 2, u32::MAX);
    copy64(m, n + 8, p);
}
/// FUN_00170be0: kind 3 node (arg)
pub fn ee_sub_170be0<M: QMem + ?Sized>(m: &mut M, l: u32, a: u32) {
    cmd(m, l, 3, a);
}
/// FUN_00170c00: kind 4 node (arg)
pub fn ee_sub_170c00<M: QMem + ?Sized>(m: &mut M, l: u32, a: u32) {
    cmd(m, l, 4, a);
}
/// FUN_00170c20: kind 5 node on list `l` (a0), +8 = v ($f12) - VER5: ABI per asm
pub fn ee_sub_170c20<M: QMem + ?Sized>(m: &mut M, l: u32, v: f32) {
    let n = cmd(m, l, 5, u32::MAX);
    m.wf(n + 8, v);
}
/// FUN_00170c50: kind 6 node (arg), +8 = b, +0xc = c
pub fn ee_sub_170c50<M: QMem + ?Sized>(m: &mut M, l: u32, a: u32, b: u32, c: u32) {
    let n = cmd(m, l, 6, a);
    m.w32(n + 0xc, c);
    m.w32(n + 8, b);
}
/// FUN_00170c90: kind 7 node on list `l` (a0, arg a1), +0x10 = c (a2), +0x14 = v ($f12) - VER5: ABI per asm
pub fn ee_sub_170c90<M: QMem + ?Sized>(m: &mut M, l: u32, a: u32, c: u32, v: f32) {
    let n = cmd(m, l, 7, a);
    m.w32(n + 0x10, c);
    m.wf(n + 0x14, v);
}
/// FUN_00170cd0: kind 8 node (arg)
pub fn ee_sub_170cd0<M: QMem + ?Sized>(m: &mut M, l: u32, a: u32) {
    cmd(m, l, 8, a);
}
/// FUN_00170cf0: kind 9 node (arg)
pub fn ee_sub_170cf0<M: QMem + ?Sized>(m: &mut M, l: u32, a: u32) {
    cmd(m, l, 9, a);
}
/// FUN_00170d10: kind 10 node (arg)
pub fn ee_sub_170d10<M: QMem + ?Sized>(m: &mut M, l: u32, a: u32) {
    cmd(m, l, 10, a);
}
/// FUN_00170d30: kind 0xb node (arg -1): "skip on" marker
pub fn ee_sub_170d30<M: QMem + ?Sized>(m: &mut M, l: u32) {
    cmd(m, l, 0xb, u32::MAX);
}
/// FUN_00170d50: kind 0xc node (arg -1): "skip off" marker
pub fn ee_sub_170d50<M: QMem + ?Sized>(m: &mut M, l: u32) {
    cmd(m, l, 0xc, u32::MAX);
}
/// FUN_00170d70: kind 0xd node (arg), +8 = v
pub fn ee_sub_170d70<M: QMem + ?Sized>(m: &mut M, l: u32, a: u32, v: u32) {
    let n = cmd(m, l, 0xd, a);
    m.w32(n + 8, v);
}
/// FUN_00170da0: kind 0xe node (arg), +8 = v
pub fn ee_sub_170da0<M: QMem + ?Sized>(m: &mut M, l: u32, a: u32, v: u32) {
    let n = cmd(m, l, 0xe, a);
    m.w32(n + 8, v);
}
/// FUN_00170dd0: kind 0xf node (arg), +8 = v
pub fn ee_sub_170dd0<M: QMem + ?Sized>(m: &mut M, l: u32, a: u32, v: u32) {
    let n = cmd(m, l, 0xf, a);
    m.w32(n + 8, v);
}
/// FUN_00170e68: when `b` != 0: FUN_00171cf8(b, a)
pub fn ee_sub_170e68<M: QMem + ?Sized>(m: &mut M, a: u32, b: u32) {
    if b != 0 {
        m.call(0x0017_1cf8, &[b, a], &[]);
    }
}
fn dist3(a: V4, b: V4) -> f32 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}
/// FUN_00170e90: nearest command node to `pos` over the list (+0x50, next +0x18). Kind 0: current object =
/// FUN_0015e938(world, cur ? 4 : 5, arg, cur); 0xb / 0xc: skip on / off; 3 / 7 (not skipping): point =
/// node +8 quadword when arg == -1 (pointer may be 0: previous point kept), else FUN_001710f8 on the object
/// `arg` below cur; 6 (not skipping): object `arg`, point from FUN_001e7380 (seeded with out +8) when `out`
/// already holds this node, else FUN_001e6f60 — also records the track parameter. Best distance starts at
/// FLT_MAX (0x2a52b0). out (if set) = {node, cur, param}; the record starts uninitialised in the original
/// (0 here) when no node wins.
pub fn ee_sub_170e90<M: QMem + ?Sized>(m: &mut M, o: u32, pos: V4, out: u32) {
    let t = stack_push(m, 0x30);
    for i in 0..0xc {
        m.w32(t + 4 * i, 0);
    }
    // t+0 {node, cur, param}, t+0x10 point, t+0x20 track param
    let mut cur = o;
    let mut skip = false;
    let mut best = f32::MAX;
    let mut n = m.r32(o + 0x50);
    while n != 0 {
        let k = m.r32(n);
        match k {
            0 => {
                let a = m.r32(n + 4);
                cur = m.call(0x0015_e938, &[WORLD, if cur != 0 { 4 } else { 5 }, a, cur], &[]).0;
            }
            0xb => skip = true,
            0xc => skip = false,
            3 | 7 if !skip => {
                let a = m.r32(n + 4);
                if a == u32::MAX {
                    let p = m.r32(n + 8);
                    if p != 0 {
                        let v = m.rv(p);
                        m.wv(t + 0x10, v);
                    }
                } else {
                    let x = m.call(0x0015_e938, &[WORLD, 4, a, cur], &[]).0;
                    ee_sub_1710f8(m, o, pos, x, t + 0x10, 0);
                }
                let d = dist3(pos, m.rv(t + 0x10));
                if d < best {
                    best = d;
                    m.w32(t, n);
                    m.w32(t + 4, cur);
                }
            }
            6 if !skip => {
                let a = m.r32(n + 4);
                let x = m.call(0x0015_e938, &[WORLD, 4, a, cur], &[]).0;
                if out != 0 && m.r32(out) == n {
                    let f = m.rf(out + 8);
                    m.call(0x001e_7380, &[x, 0, t + 0x10, 0, 0, t + 0x20], &[pos, s(f)]);
                } else {
                    m.call(0x001e_6f60, &[x, 0, t + 0x10, 0, 0, t + 0x20], &[pos]);
                }
                let d = dist3(pos, m.rv(t + 0x10));
                if d < best {
                    best = d;
                    m.w32(t, n);
                    let f = m.r32(t + 0x20);
                    m.w32(t + 8, f);
                    m.w32(t + 4, cur);
                }
            }
            _ => {}
        }
        n = m.r32(n + 0x18);
    }
    if out != 0 {
        for i in 0..3 {
            let v = m.r32(t + 4 * i);
            m.w32(out + 4 * i, v);
        }
    }
    stack_pop(m, 0x30);
}
/// FUN_001710f8: point / heading of target `x` seen from `pos`: x == 0 -> pos, heading 0; class 10 (track,
/// FUN_00132b10) -> FUN_001e6f60 nearest point + tangent, heading atan2(t.y, t.x); else the object's
/// position (FUN_001a62e8) and heading atan2 of its direction (vtable +0xfc, read twice as in the original)
pub fn ee_sub_1710f8<M: QMem + ?Sized>(m: &mut M, _o: u32, pos: V4, x: u32, out_p: u32, out_a: u32) {
    let (p, a);
    if x == 0 {
        p = pos;
        a = 0.0;
    } else if m.call(0x0013_2b10, &[x, 10], &[]).0 == 0 {
        p = m.call(0x001a_62e8, &[x], &[]).1;
        let d1 = vcall(m, x, 0xc, 0xfc, &[], &[]).0;
        let d0 = vcall(m, x, 0xc, 0xfc, &[], &[]).0;
        let (y, xx) = (m.rf(d1 + 4), m.rf(d0));
        a = callf(m, 0x0024_58f0, &[], &[s(y), s(xx)]);
    } else {
        let t = stack_push(m, 0x20);
        m.call(0x001e_6f60, &[x, 0, t, t + 0x10, 0, 0], &[pos]);
        p = m.rv(t);
        let (y, xx) = (m.rf(t + 0x14), m.rf(t + 0x10));
        stack_pop(m, 0x20);
        a = callf(m, 0x0024_58f0, &[], &[s(y), s(xx)]);
    }
    if out_p != 0 {
        m.wv(out_p, p);
    }
    if out_a != 0 {
        m.wf(out_a, a);
    }
}
/// FUN_001711f0: first kind-9 node of the list (+0x50) whose arg is `id`
pub fn ee_sub_1711f0<M: QMem + ?Sized>(m: &mut M, o: u32, id: u32) -> u32 {
    let mut n = m.r32(o + 0x50);
    while n != 0 {
        if m.r32(n) == 9 && m.r32(n + 4) == id {
            return n;
        }
        n = m.r32(n + 0x18);
    }
    0
}
/// FUN_001715b8: FUN_0015e938(world, +0x34 ? 4 : 5, id, +0x34)
pub fn ee_sub_1715b8<M: QMem + ?Sized>(m: &mut M, o: u32, id: u32) -> u32 {
    let c = m.r32(o + 0x34);
    m.call(0x0015_e938, &[WORLD, if c == 0 { 5 } else { 4 }, id, c], &[]).0
}

// ---------------------------------------------------------------- run_00217ba0 (screen-space sprite tracks)
// Object: +0 u16 entry count, +4 entries (0x18 each; key at +0x14, duration +8), +8 slots (0x18 each:
// +8 i16 entry, +0xa i16, +0xc f32, +0x10), +0x10 u64 stamp, +0x18 f32, +0x1c / +0x20 pending entry,
// +0x24 owner, +0x28 enabled, +0x2c optional 2x3 transform.
/// word read that also reaches .rodata (constant blocks like 0x2a3a10 are passed by address)
fn rw<M: QMem + ?Sized>(m: &mut M, a: u32) -> u32 {
    if (0x0026_2380..0x002a_c744).contains(&a) {
        crate::engine::classes::q::q6::ro32(a)
    } else {
        m.r32(a)
    }
}
fn slot_entry<M: QMem + ?Sized>(m: &mut M, o: u32, slot: u32) -> i32 {
    let b = m.r32(o + 8);
    m.r16(b + slot * 0x18 + 8) as i16 as i32
}
/// index of the entry whose key (+0x14) is `key`, None when absent
fn find_key<M: QMem + ?Sized>(m: &mut M, o: u32, key: u32) -> Option<u32> {
    let n = m.r16(o) as u32;
    let b = m.r32(o + 4);
    (0..n).find(|&i| m.r32(b + i * 0x18 + 0x14) == key)
}
/// 2x3 affine transform of 6 floats at `src` by the matrix at `mt` into `dst` (words 0..5)
fn xform23<M: QMem + ?Sized>(m: &mut M, src: u32, mt: u32, dst: u32) {
    let x: Vec<f32> = (0..6).map(|i| f32::from_bits(rw(m, src + 4 * i))).collect();
    let p: Vec<f32> = (0..6).map(|i| m.rf(mt + 4 * i)).collect();
    let r = [
        x[0] * p[0] + x[1] * p[2],
        x[0] * p[1] + x[1] * p[3],
        x[2] * p[0] + x[3] * p[2],
        x[2] * p[1] + x[3] * p[3],
        x[4] * p[0] + x[5] * p[2] + p[4],
        x[4] * p[1] + x[5] * p[3] + p[5],
    ];
    for (i, v) in r.into_iter().enumerate() {
        m.wf(dst + 4 * i as u32, v);
    }
}
/// FUN_00217ba0: when enabled: play the entry with key `key` (0x314 or not found -> entry 0) via FUN_00217c20
pub fn ee_sub_217ba0<M: QMem + ?Sized>(m: &mut M, o: u32, key: u32) {
    if m.r32(o + 0x28) == 0 {
        return;
    }
    let i = if key == 0x314 { 0 } else { find_key(m, o, key).unwrap_or(0) };
    ee_sub_217c20(m, o, i);
}
/// FUN_00217c20: when enabled: slot 0 busy -> FUN_00218638; start entry `i` in slot 0 with the default rect
/// 0x2a3a10, t 0, colours 0x7f7f7f7f / 0xffffffff
pub fn ee_sub_217c20<M: QMem + ?Sized>(m: &mut M, o: u32, i: u32) {
    if m.r32(o + 0x28) == 0 {
        return;
    }
    if slot_entry(m, o, 0) >= 0 {
        ee_sub_218638(m, o);
    }
    let t = stack_push(m, 0x20);
    m.w32(t, 0x7f7f_7f7f);
    m.w32(t + 0x10, 0xffff_ffff);
    ee_sub_217d98(m, o, i, 0, 0x002a_3a10, 0.0, t, t + 0x10);
    stack_pop(m, 0x20);
}
/// FUN_00217ca8: key 0x314: pending entry +0x20 (taken and reset to -1) or, with none, +0x1c / t +0x18 when
/// the stamp +0x10 equals DAT_002ddf18 (else both cleared); other keys: the matching entry (absent -> return).
/// Starts it in slot `slot` with rect `r`, colours *c0 / *c1 (FUN_00217d98)
pub fn ee_sub_217ca8<M: QMem + ?Sized>(m: &mut M, o: u32, key: u32, slot: u32, r: u32, c0: u32, c1: u32) {
    let mut t = 0.0f32;
    let i;
    if key == 0x314 {
        let p = m.r32(o + 0x20);
        if p == u32::MAX {
            if m.r64(o + 0x10) == m.r64(0x002d_df18) {
                t = m.rf(o + 0x18);
                i = m.r32(o + 0x1c);
            } else {
                m.wf(o + 0x18, 0.0);
                i = 0;
                m.w32(o + 0x1c, 0);
            }
        } else {
            m.w32(o + 0x20, u32::MAX);
            i = p;
        }
    } else {
        match find_key(m, o, key) {
            Some(x) => i = x,
            None => return,
        }
    }
    let s = stack_push(m, 0x20);
    let (a, b) = (m.r32(c0), m.r32(c1));
    m.w32(s, a);
    m.w32(s + 0x10, b);
    ee_sub_217d98(m, o, i, slot, r, t, s, s + 0x10);
    stack_pop(m, 0x20);
}
/// FUN_00217d98: slot `slot` = {entry i, 0, t, 0}; rect `r` (6 floats) through the +0x2c transform when set;
/// FUN_00219fe0(entry i, slot, rect, &c0, &c1) (colour words copied into quadword locals whose other words
/// hold the transformed rect as in the original; 0 when untransformed)
#[allow(clippy::too_many_arguments)]
pub fn ee_sub_217d98<M: QMem + ?Sized>(m: &mut M, o: u32, i: u32, slot: u32, r: u32, t: f32, c0: u32, c1: u32) {
    let e = m.r32(o + 8) + slot * 0x18;
    m.w16(e + 8, i as u16);
    m.w32(e + 0x10, 0);
    m.wf(e + 0xc, t);
    m.w16(e + 0xa, 0);
    let s = stack_push(m, 0x40);
    for k in 0..0x10 {
        m.w32(s + 4 * k, 0);
    }
    let mt = m.r32(o + 0x2c);
    let mut rect = r;
    if mt != 0 {
        xform23(m, r, mt, s);
        for k in 0..8 {
            let v = m.r32(s + 4 * k);
            m.w32(s + 0x20 + 4 * k, v);
        }
        rect = s;
    }
    let (a, b) = (m.r32(c0), m.r32(c1));
    m.w32(s + 0x20, a);
    m.w32(s + 0x30, b);
    let idx = m.r16(e + 8) as i16 as i32;
    let ent = m.r32(o + 4).wrapping_add((idx * 0x18) as u32);
    m.call(0x0021_9fe0, &[ent, slot, rect, s + 0x20, s + 0x30], &[[t; 4]]);
    stack_pop(m, 0x40);
}
/// FUN_00217fb8: when enabled: FUN_00217fe0(this, 0, 0, t) (t = the caller's f12, passed through)
pub fn ee_sub_217fb8<M: QMem + ?Sized>(m: &mut M, o: u32, t: f32) {
    if m.r32(o + 0x28) != 0 {
        ee_sub_217fe0(m, o, 0, 0, t);
    }
}
/// FUN_00217fe0: slot busy -> FUN_00218060(this, slot, rec, clamp(t, 0, 1) * entry duration)
pub fn ee_sub_217fe0<M: QMem + ?Sized>(m: &mut M, o: u32, slot: u32, rec: u32, t: f32) {
    let t = t.max(0.0).min(1.0);
    let e = slot_entry(m, o, slot);
    if e >= 0 {
        let ea = m.r32(o + 4).wrapping_add((e * 0x18) as u32) + 8;
        let d = m.rf(ea);
        ee_sub_218060(m, o, slot, rec, d * t);
    }
}
/// FUN_00218038: when enabled: FUN_00218060(this, 0, 0, t) (t passed through)
pub fn ee_sub_218038<M: QMem + ?Sized>(m: &mut M, o: u32, t: f32) {
    if m.r32(o + 0x28) != 0 {
        ee_sub_218060(m, o, 0, 0, t);
    }
}
/// FUN_00218060: seek slot `slot` to time max(t, 0) (capped at the entry duration): geometry from `rec`
/// (words 4..11 rect, 0 / 1 colours) or, with rec 0, FUN_00213e20(owner +0x24, this index in the owner's
/// 0x30-byte array +0x84, ..) (0 -> return); FUN_00217f38 resets, slot {+0xc 0, +0xa 0, +0x10 0},
/// FUN_002183e8 at t 0, then at the capped time, +0x10 = 1
pub fn ee_sub_218060<M: QMem + ?Sized>(m: &mut M, o: u32, slot: u32, rec: u32, t: f32) {
    let e = slot_entry(m, o, slot);
    if e < 0 {
        return;
    }
    let t = t.max(0.0);
    let ea = m.r32(o + 4).wrapping_add((e * 0x18) as u32) + 8;
    let d = m.rf(ea);
    let s = stack_push(m, 0x60);
    // s+0 rect (8 words), s+0x20 colour 0, s+0x30 colour 1, s+0x40 / s+0x50 copies passed on
    if rec == 0 {
        let ow = m.r32(o + 0x24);
        let base = m.r32(ow + 0x84);
        let idx = ((o.wrapping_sub(base)).wrapping_mul(0xaaaa_aaab) as i32) >> 4;
        if m.call(0x0021_3e20, &[ow, idx as u32, s, s + 0x20, s + 0x30, 0, 0], &[]).0 == 0 {
            stack_pop(m, 0x60);
            return;
        }
    } else {
        for k in 0..8 {
            let v = m.r32(rec + 0x10 + 4 * k);
            m.w32(s + 4 * k, v);
        }
        let (a, b) = (m.r32(rec), m.r32(rec + 4));
        m.w32(s + 0x20, a);
        m.w32(s + 0x30, b);
    }
    m.call(0x0021_7f38, &[o, slot], &[]);
    let sl = m.r32(o + 8) + slot * 0x18;
    m.w32(sl + 0xc, 0);
    m.w16(sl + 0xa, 0);
    m.w32(sl + 0x10, 0);
    let (a, b) = (m.r32(s + 0x20), m.r32(s + 0x30));
    m.w32(s + 0x40, a);
    m.w32(s + 0x50, b);
    m.call(0x0021_83e8, &[o, slot, s, s + 0x40, s + 0x50], &[[0.0; 4]]);
    let sl = m.r32(o + 8) + slot * 0x18;
    m.w32(sl + 0x10, 0);
    m.w32(s + 0x40, a);
    m.w32(s + 0x50, b);
    let tt = if d < t { d } else { t };
    m.call(0x0021_83e8, &[o, slot, s, s + 0x40, s + 0x50], &[[tt; 4]]);
    let sl = m.r32(o + 8) + slot * 0x18;
    m.w32(sl + 0x10, 1);
    stack_pop(m, 0x60);
}
/// FUN_00218220: when enabled: rect = 0x2a3a10 block; owner (+0x24) with a target (+0xf4): position moved to
/// the target's screen projection (FUN_0014f308(0x2ddf40, pos): x * 320 + 320, y * -246.4 + 246.4); owner
/// transform +0xf8 applied; FUN_002183e8(this, 0, rect, 0x7f7f7f7f, 0xffffffff) at t
pub fn ee_sub_218220<M: QMem + ?Sized>(m: &mut M, o: u32, t: f32) {
    if m.r32(o + 0x28) == 0 {
        return;
    }
    let s = stack_push(m, 0x40);
    for k in 0..8 {
        let v = rw(m, 0x002a_3a10 + 4 * k);
        m.w32(s + 4 * k, v);
    }
    let ow = m.r32(o + 0x24);
    let tg = m.r32(ow + 0xf4);
    if tg != 0 {
        let p = m.call(0x001a_62e8, &[tg], &[]).1;
        let q = m.call(0x0014_f308, &[0x002d_df40], &[p]).1;
        m.wf(s + 0x10, q[0] * 320.0 + 320.0);
        m.wf(s + 0x14, q[1] * -246.400_01 + 246.400_01);
    }
    let mt = m.r32(ow + 0xf8);
    if mt != 0 {
        xform23(m, s, mt, s);
    }
    m.w32(s + 0x20, 0x7f7f_7f7f);
    m.w32(s + 0x30, 0xffff_ffff);
    m.call(0x0021_83e8, &[o, 0, s, s + 0x20, s + 0x30], &[[t; 4]]);
    stack_pop(m, 0x40);
}
/// FUN_002185f8: slot busy -> FUN_0021a998(its entry)
pub fn ee_sub_2185f8<M: QMem + ?Sized>(m: &mut M, o: u32, slot: u32) {
    let e = slot_entry(m, o, slot);
    if e >= 0 {
        let a = m.r32(o + 4).wrapping_add((e * 0x18) as u32);
        m.call(0x0021_a998, &[a], &[]);
    }
}
/// FUN_00218638: when enabled: FUN_00218660(this, 0)
pub fn ee_sub_218638<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.r32(o + 0x28) != 0 {
        m.call(0x0021_8660, &[o, 0], &[]);
    }
}
/// FUN_00218710: slot 0 holds an entry (i16 != 0) whose +0x10 is still 0
pub fn ee_sub_218710<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let b = m.r32(o + 8);
    (m.r16(b + 8) != 0 && m.r32(b + 0x10) == 0) as u32
}
/// FUN_00218738: slot 0 +0x10
pub fn ee_sub_218738<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let b = m.r32(o + 8);
    m.r32(b.wrapping_add(0x10))
}

// ---------------------------------------------------------------- run_00258898 / run_00259330 (variant field
// accessors of classes 29 / 32-33). Record = FUN_00258658 / FUN_002590f0(this); its +4 selects the variant
// (0 / 1; -1 = unset), +0xc0 the variant block (first word = sub-kind, payload at +0x30), else the shared
// empty block DAT_002de720 / payload DAT_002de750.
/// variant 0 / 1 block: with `force` 1 and the record unset, select variant `v` and copy its 0x40-byte
/// template (0x2bde10 / 0x2bde70) into +0xc0 (FUN_00258898 / FUN_00258920 / FUN_00259330 / FUN_002593b8)
fn variant<M: QMem + ?Sized>(m: &mut M, rec_fn: u32, o: u32, force: u32, v: u32) -> u32 {
    let r = m.call(rec_fn, &[o, force], &[]).0; // a1 (force) is still live into FUN_00258658(o, n)
    if force == 1 && m.r32(r + 4) == u32::MAX {
        m.w32(r + 4, v);
        let tpl = if v == 0 { 0x002b_de10 } else { 0x002b_de70 };
        for k in 0..0x10 {
            let w = m.r32(tpl + 4 * k);
            m.w32(r + 0xc0 + 4 * k, w);
        }
    }
    if m.r32(r + 4) == v {
        r + 0xc0
    } else {
        0x002d_e720
    }
}
/// variant 1 sub-kind set (FUN_00258b70 / FUN_00259608): unset (-1) block takes kind `k` with its payload
/// (3: word 0x2bde58; 2: u64 0x2bde50; 8: u64 0x2bde60 + word 0x2bde68)
fn set_kind1<M: QMem + ?Sized>(m: &mut M, rec_fn: u32, o: u32, k: u32) {
    let p = variant(m, rec_fn, o, 1, 1);
    let cur = m.r32(p);
    if cur != k && cur == u32::MAX {
        m.w32(p, k);
        let w68 = m.r32(0x002b_de68);
        match k {
            3 => {
                let v = m.r32(0x002b_de58);
                m.w32(p + 0x30, v);
            }
            2 => copy64(m, p + 0x30, 0x002b_de50),
            8 => {
                copy64(m, p + 0x30, 0x002b_de60);
                m.w32(p + 0x38, w68);
            }
            _ => {}
        }
    }
}
/// variant 0 sub-kind set (FUN_00258d98 / FUN_00259830): when (+0xe4)->+0xc0 is 0 (kind 0): kind `k`
/// (1: word 0x2bde00; 2: u64 0x2bddf0 + u64 0x2bddf8)
fn set_kind0<M: QMem + ?Sized>(m: &mut M, rec_fn: u32, o: u32, k: u32) {
    let p = variant(m, rec_fn, o, 1, 0);
    let e = m.r32(o + 0xe4);
    if m.r32(p) != k && m.r32(e + 0xc0) == 0 {
        m.w32(p, k);
        match k {
            1 => {
                let v = m.r32(0x002b_de00);
                m.w32(p + 0x30, v);
            }
            2 => {
                copy64(m, p + 0x30, 0x002b_ddf0);
                copy64(m, p + 0x38, 0x002b_ddf8);
            }
            _ => {}
        }
    }
}
/// payload getter: `force` 1 -> set sub-kind `k` and use (+0xe4)+0xc0; else the variant block; payload
/// (+0x30) when its kind is `k`, else DAT_002de750
fn payload<M: QMem + ?Sized>(m: &mut M, rec_fn: u32, o: u32, force: u32, k: u32, var: u32) -> u32 {
    let p = if force == 1 {
        if var == 1 {
            set_kind1(m, rec_fn, o, k);
        } else {
            set_kind0(m, rec_fn, o, k);
        }
        m.r32(o + 0xe4) + 0xc0
    } else {
        variant(m, rec_fn, o, force, var)
    };
    if m.r32(p) == k {
        p + 0x30
    } else {
        0x002d_e750
    }
}
const R29: u32 = 0x0025_8658;
const R32: u32 = 0x0025_90f0;
/// FUN_00258898: variant-0 block (class 29)
pub fn ee_sub_258898<M: QMem + ?Sized>(m: &mut M, o: u32, force: u32) -> u32 {
    variant(m, R29, o, force, 0)
}
/// FUN_00258920: variant-1 block (class 29)
pub fn ee_sub_258920<M: QMem + ?Sized>(m: &mut M, o: u32, force: u32) -> u32 {
    variant(m, R29, o, force, 1)
}
/// FUN_002589a8: select variant `v` (0 / 1)
pub fn ee_sub_2589a8<M: QMem + ?Sized>(m: &mut M, o: u32, v: u32) {
    match v {
        0 => drop(variant(m, R29, o, 1, 0)),
        1 => drop(variant(m, R29, o, 1, 1)),
        _ => {}
    }
}
fn kind_get<M: QMem + ?Sized>(m: &mut M, kf: u32, o: u32, force: u32, k: u32, off: u32) -> u32 {
    if force == 1 {
        m.call(kf, &[o, k], &[]);
    }
    let e = m.r32(o + 0xe4);
    if off == 0x30 {
        m.r32(e + 0x30)
    } else {
        e + off
    }
}
/// FUN_00258ac8: force 1 -> kind_after_props(this, 0) (FUN_00258a00); returns (+0xe4) + 0x10
pub fn ee_sub_258ac8<M: QMem + ?Sized>(m: &mut M, o: u32, force: u32) -> u32 {
    kind_get(m, 0x0025_8a00, o, force, 0, 0x10)
}
/// FUN_00258b00: force 1 -> kind_after_props(this, 1); returns (+0xe4)->+0x30
pub fn ee_sub_258b00<M: QMem + ?Sized>(m: &mut M, o: u32, force: u32) -> u32 {
    kind_get(m, 0x0025_8a00, o, force, 1, 0x30)
}
/// FUN_00258b38: force 1 -> kind_after_props(this, 2); returns (+0xe4) + 0x10
pub fn ee_sub_258b38<M: QMem + ?Sized>(m: &mut M, o: u32, force: u32) -> u32 {
    kind_get(m, 0x0025_8a00, o, force, 2, 0x10)
}
/// FUN_00258b70: variant-1 sub-kind set
pub fn ee_sub_258b70<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32) {
    set_kind1(m, R29, o, k);
}
/// FUN_00258c40: variant-1 sub-kind (no force)
pub fn ee_sub_258c40<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let p = variant(m, R29, o, 0, 1);
    m.r32(p)
}
/// FUN_00258c60: variant-1 kind-2 payload
pub fn ee_sub_258c60<M: QMem + ?Sized>(m: &mut M, o: u32, force: u32) -> u32 {
    payload(m, R29, o, force, 2, 1)
}
/// FUN_00258cc8: variant-1 kind-3 payload
pub fn ee_sub_258cc8<M: QMem + ?Sized>(m: &mut M, o: u32, force: u32) -> u32 {
    payload(m, R29, o, force, 3, 1)
}
/// FUN_00258d30: variant-1 kind-8 payload
pub fn ee_sub_258d30<M: QMem + ?Sized>(m: &mut M, o: u32, force: u32) -> u32 {
    payload(m, R29, o, force, 8, 1)
}
/// FUN_00258d98: variant-0 sub-kind set
pub fn ee_sub_258d98<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32) {
    set_kind0(m, R29, o, k);
}
/// FUN_00258e40: variant-0 sub-kind (no force)
pub fn ee_sub_258e40<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let p = variant(m, R29, o, 0, 0);
    m.r32(p)
}
/// FUN_00258e60: variant-0 kind-2 payload
pub fn ee_sub_258e60<M: QMem + ?Sized>(m: &mut M, o: u32, force: u32) -> u32 {
    payload(m, R29, o, force, 2, 0)
}
/// FUN_00258ec8: variant-0 kind-1 payload
pub fn ee_sub_258ec8<M: QMem + ?Sized>(m: &mut M, o: u32, force: u32) -> u32 {
    payload(m, R29, o, force, 1, 0)
}
/// FUN_00259330: variant-0 block (classes 32 / 33)
pub fn ee_sub_259330<M: QMem + ?Sized>(m: &mut M, o: u32, force: u32) -> u32 {
    variant(m, R32, o, force, 0)
}
/// FUN_002593b8: variant-1 block (classes 32 / 33)
pub fn ee_sub_2593b8<M: QMem + ?Sized>(m: &mut M, o: u32, force: u32) -> u32 {
    variant(m, R32, o, force, 1)
}
/// FUN_00259440: select variant `v`
pub fn ee_sub_259440<M: QMem + ?Sized>(m: &mut M, o: u32, v: u32) {
    match v {
        0 => drop(variant(m, R32, o, 1, 0)),
        1 => drop(variant(m, R32, o, 1, 1)),
        _ => {}
    }
}
/// FUN_00259560: force 1 -> kind_after_props(this, 0) (FUN_00259498); returns (+0xe4) + 0x10
pub fn ee_sub_259560<M: QMem + ?Sized>(m: &mut M, o: u32, force: u32) -> u32 {
    kind_get(m, 0x0025_9498, o, force, 0, 0x10)
}
/// FUN_00259598: force 1 -> kind_after_props(this, 1); returns (+0xe4)->+0x30
pub fn ee_sub_259598<M: QMem + ?Sized>(m: &mut M, o: u32, force: u32) -> u32 {
    kind_get(m, 0x0025_9498, o, force, 1, 0x30)
}
/// FUN_002595d0: force 1 -> kind_after_props(this, 2); returns (+0xe4) + 0x10
pub fn ee_sub_2595d0<M: QMem + ?Sized>(m: &mut M, o: u32, force: u32) -> u32 {
    kind_get(m, 0x0025_9498, o, force, 2, 0x10)
}
/// FUN_00259608: variant-1 sub-kind set
pub fn ee_sub_259608<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32) {
    set_kind1(m, R32, o, k);
}
/// FUN_002596d8: variant-1 sub-kind (no force)
pub fn ee_sub_2596d8<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let p = variant(m, R32, o, 0, 1);
    m.r32(p)
}
/// FUN_002596f8: variant-1 kind-2 payload
pub fn ee_sub_2596f8<M: QMem + ?Sized>(m: &mut M, o: u32, force: u32) -> u32 {
    payload(m, R32, o, force, 2, 1)
}
/// FUN_00259760: variant-1 kind-3 payload
pub fn ee_sub_259760<M: QMem + ?Sized>(m: &mut M, o: u32, force: u32) -> u32 {
    payload(m, R32, o, force, 3, 1)
}
/// FUN_002597c8: variant-1 kind-8 payload
pub fn ee_sub_2597c8<M: QMem + ?Sized>(m: &mut M, o: u32, force: u32) -> u32 {
    payload(m, R32, o, force, 8, 1)
}
/// FUN_00259830: variant-0 sub-kind set
pub fn ee_sub_259830<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32) {
    set_kind0(m, R32, o, k);
}
/// FUN_002598d8: variant-0 sub-kind (no force)
pub fn ee_sub_2598d8<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let p = variant(m, R32, o, 0, 0);
    m.r32(p)
}
/// FUN_002598f8: variant-0 kind-2 payload
pub fn ee_sub_2598f8<M: QMem + ?Sized>(m: &mut M, o: u32, force: u32) -> u32 {
    payload(m, R32, o, force, 2, 0)
}
/// FUN_00259960: variant-0 kind-1 payload
pub fn ee_sub_259960<M: QMem + ?Sized>(m: &mut M, o: u32, force: u32) -> u32 {
    payload(m, R32, o, force, 1, 0)
}
/// FUN_002599d8: word 0
pub fn ee_sub_2599d8<M: QMem + ?Sized>(m: &mut M, p: u32) -> u32 {
    m.r32(p)
}
/// FUN_002599e0: word 1
pub fn ee_sub_2599e0<M: QMem + ?Sized>(m: &mut M, p: u32) -> u32 {
    m.r32(p + 4)
}
/// FUN_002599e8: word 2
pub fn ee_sub_2599e8<M: QMem + ?Sized>(m: &mut M, p: u32) -> u32 {
    m.r32(p + 8)
}
/// FUN_00259a88: FUN_00255770(this), FUN_00255588(this, 0x2dd2ac), +4 |= 0x8000
pub fn ee_sub_259a88<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0025_5770, &[o], &[]);
    m.call(0x0025_5588, &[o, 0x002d_d2ac], &[]);
    let v = m.r32(o + 4);
    m.w32(o + 4, v | 0x8000);
}
/// FUN_00259b48: ref-counted pointer assign: new->AddRef (vtable +0xc), old->Release (vtable +0x14), *p = new
pub fn ee_sub_259b48<M: QMem + ?Sized>(m: &mut M, p: u32, x: u32) {
    if x != 0 {
        vcall(m, x, 0xc, 0xc, &[], &[]);
    }
    let old = m.r32(p);
    if old != 0 {
        vcall(m, old, 0xc, 0x14, &[], &[]);
    }
    m.w32(p, x);
}
/// FUN_00259bb0: new->AddRef (vtable +0xc), *p = new (no release)
pub fn ee_sub_259bb0<M: QMem + ?Sized>(m: &mut M, p: u32, x: u32) {
    if x != 0 {
        vcall(m, x, 0xc, 0xc, &[], &[]);
    }
    m.w32(p, x);
}
/// FUN_00259c18: pooled node (pool 0x452170): the first time (DAT_0045217c == 0) a node per call is kept in
/// DAT_00452160[DAT_002dd2d4++] (all but the first freed back; FUN_001286e0(0x2dd598) when DAT_003e0ea4 is 0
/// or more than 3 were kept); then a fresh node initialised by FUN_002560c8 is returned
pub fn ee_sub_259c18<M: QMem + ?Sized>(m: &mut M) -> u32 {
    if m.r32(0x0045_217c) == 0 {
        if m.r32(0x003e_0ea4) == 0 || 3 < m.r32(0x002d_d2d4) as i32 {
            m.call(0x0012_86e0, &[0x002d_d598], &[]);
        }
        if m.r32(0x0045_217c) == 0 {
            let x = m.call(0x0011_ccc0, &[0x0045_2170], &[]).0;
            let n = m.r32(0x002d_d2d4);
            if n != 0 {
                m.call(0x0011_cd98, &[0x0045_2170, x], &[]);
            }
            m.w32(0x002d_d2d4, n + 1);
            m.w32(0x0045_2160 + n * 4, x);
        }
    }
    let x = m.call(0x0011_ccc0, &[0x0045_2170], &[]).0;
    m.call(0x0025_60c8, &[x], &[]);
    x
}
/// FUN_00259cf8: *p = x with x->+0 (refcount) incremented when x != 0
pub fn ee_sub_259cf8<M: QMem + ?Sized>(m: &mut M, p: u32, x: u32) {
    if x != 0 {
        let v = m.r32(x);
        m.w32(x, v + 1);
    }
    m.w32(p, x);
}
/// FUN_00259d18: tail to FUN_00204660 (heading set; registers passed through)
pub fn ee_sub_259d18<M: QMem + ?Sized>(m: &mut M, o: u32, a1: u32, f: f32) {
    m.call(0x0020_4660, &[o, a1], &[s(f)]);
}

// ---------------------------------------------------------------- run_00153800 (guard sensor helpers)
/// FUN_00153800: +0x288, +0x23c, +0x284 cleared
pub fn ee_sub_153800<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.w32(o + 0x288, 0);
    m.w32(o + 0x23c, 0);
    m.w32(o + 0x284, 0);
}
/// FUN_00153850: pool refcount bump (same body as FUN_001ba9f8)
pub fn ee_sub_153850<M: QMem + ?Sized>(m: &mut M, on: u32, k: u32) {
    ee_sub_1ba9f8(m, on, k);
}
/// FUN_00153988: FUN_00153850(1, 0xffff)
pub fn ee_sub_153988<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_1ba9f8(m, 1, 0xffff);
}
/// FUN_001539b0: pool refcount bump (same body as FUN_001ba9f8)
pub fn ee_sub_1539b0<M: QMem + ?Sized>(m: &mut M, on: u32, k: u32) {
    ee_sub_1ba9f8(m, on, k);
}
/// FUN_00153ae8: FUN_001539b0(1, 0xffff)
pub fn ee_sub_153ae8<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_1ba9f8(m, 1, 0xffff);
}
/// FUN_00153c00: pool refcount bump (same body as FUN_001ba9f8)
pub fn ee_sub_153c00<M: QMem + ?Sized>(m: &mut M, on: u32, k: u32) {
    ee_sub_1ba9f8(m, on, k);
}
/// FUN_00153d38: FUN_00153c00(1, 0xffff)
pub fn ee_sub_153d38<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_1ba9f8(m, 1, 0xffff);
}
/// FUN_00153ef8: b != c -> FUN_00153f18(a, c)
pub fn ee_sub_153ef8<M: QMem + ?Sized>(m: &mut M, a: u32, b: u32, c: u32) {
    if b != c {
        m.call(0x0015_3f18, &[a, c], &[]);
    }
}
/// FUN_00153f80: remove the sensor entry for `key` (FUN_00154180): FUN_00154080(this, e, 0), count +0xfc - 1,
/// the following 16-byte entries (array at +0x100) moved down one
pub fn ee_sub_153f80<M: QMem + ?Sized>(m: &mut M, o: u32, key: u32) {
    let e = m.call(0x0015_4180, &[o, key], &[]).0;
    if e == 0 {
        return;
    }
    m.call(0x0015_4080, &[o, e, 0], &[]);
    let n = m.r32(o + 0xfc).wrapping_sub(1);
    m.w32(o + 0xfc, n);
    let end = o.wrapping_add(n.wrapping_mul(0x10)).wrapping_add(0x100);
    let len = ((end.wrapping_sub(e) as i32) >> 4) << 4;
    let mut i = 0;
    while i < len {
        let v = m.r8(e + 0x10 + i as u32);
        m.w8(e + i as u32, v);
        i += 1;
    }
}
/// FUN_00153ff8: every sensor entry (count +0xfc, 16 bytes from +0x100): +4 = 0, +8 = DAT_002dded0
pub fn ee_sub_153ff8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let mut i = 0;
    while (i as i32) < m.r32(o + 0xfc) as i32 {
        let p = o + 0x108 + i * 0x10;
        m.w32(p - 4, 0);
        let v = m.r32(0x002d_ded0);
        m.w32(p, v);
        i += 1;
    }
}
/// FUN_00154038: entry for `key` (FUN_00154180) -> FUN_00154080(this, e, a2)
pub fn ee_sub_154038<M: QMem + ?Sized>(m: &mut M, o: u32, key: u32, a2: u32) {
    let e = m.call(0x0015_4180, &[o, key], &[]).0;
    if e != 0 {
        m.call(0x0015_4080, &[o, e, a2], &[]);
    }
}
/// FUN_00154158: entry for `key` -> its +4, else -1
pub fn ee_sub_154158<M: QMem + ?Sized>(m: &mut M, o: u32, key: u32) -> u32 {
    let e = m.call(0x0015_4180, &[o, key], &[]).0;
    if e == 0 { u32::MAX } else { m.r32(e + 4) }
}

// ---------------------------------------------------------------- run_00164858 (scoped +0xf4/+0xf8 overrides)
/// FUN_00164858: with +0xf8 = a2 (and the linked +0x7c object's +0xf0/+0xf4 = {this +0xf4, a2}) for the call,
/// restored after: mode `k` (-1 -> 4) when +0x54 != -1: 4 -> (+0x54 == 0 and FUN_001645d0 -> FUN_00164df0(1)),
/// FUN_00163c28; 0..3 -> FUN_00164df0(k); 5 -> FUN_00163ce0
pub fn ee_sub_164858<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32, a2: u32) {
    let (s4, s8) = (m.r32(o + 0xf4), m.r32(o + 0xf8));
    if o != 0 {
        m.w32(o + 0xf8, a2);
    }
    let l = m.r32(o + 0x7c);
    let (l0, l4) = if l != 0 { (m.r32(l + 0xf0), m.r32(l + 0xf4)) } else { (0, 0) };
    if l != 0 {
        let v = m.r32(o + 0xf4);
        m.w32(l + 0xf0, v);
        m.w32(l + 0xf4, a2);
    }
    let k = if k == u32::MAX { 4 } else { k as i32 as u32 };
    if m.r32(o + 0x54) != u32::MAX {
        let ki = k as i32;
        if ki == 4 {
            if m.r32(o + 0x54) == 0 && m.call(0x0016_45d0, &[o], &[]).0 != 0 {
                m.call(0x0016_4df0, &[o, 1], &[]);
            }
            m.call(0x0016_3c28, &[o], &[]);
        } else if ki < 5 {
            if ki >= 0 {
                m.call(0x0016_4df0, &[o, k], &[]);
            }
        } else if ki == 5 {
            m.call(0x0016_3ce0, &[o], &[]);
        }
    }
    if l != 0 {
        m.w32(l + 0xf0, l0);
        m.w32(l + 0xf4, l4);
    }
    if o != 0 {
        m.w32(o + 0xf4, s4);
        m.w32(o + 0xf8, s8);
    }
}
fn scoped_164858<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32) {
    let (s4, s8) = (m.r32(o + 0xf4), m.r32(o + 0xf8));
    if o != 0 {
        m.w32(o + 0xf4, 4);
        m.w32(o + 0xf8, 0);
    }
    ee_sub_164858(m, o, k, 0);
    m.call(0x0016_49d0, &[], &[]);
    if o != 0 {
        m.w32(o + 0xf4, s4);
        m.w32(o + 0xf8, s8);
    }
}
/// FUN_00164aa8: with +0xf4 = 4, +0xf8 = 0: FUN_00164858(this, 0, 0), FUN_001649d0()
pub fn ee_sub_164aa8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    scoped_164858(m, o, 0);
}
/// FUN_00164b38: as FUN_00164aa8 with mode 1
pub fn ee_sub_164b38<M: QMem + ?Sized>(m: &mut M, o: u32) {
    scoped_164858(m, o, 1);
}
/// FUN_00164bc8: as FUN_00164aa8 with mode 2
pub fn ee_sub_164bc8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    scoped_164858(m, o, 2);
}
/// FUN_00164c58: as FUN_00164aa8 with mode 3
pub fn ee_sub_164c58<M: QMem + ?Sized>(m: &mut M, o: u32) {
    scoped_164858(m, o, 3);
}

// ---------------------------------------------------------------- class C2ca720 (anim channel, base C2ca6e0)
/// FUN_0012def0 (C2ca720_v3): local = (host (+8) ->+0xa0 set ? inverse(+0xa0 + 0x40) (FUN_0011c318) *
/// (+0x1c)+0x40 : (+0x1c)+0x40) * (this + 0x20) (FUN_0011c168); host +0xc0 = FUN_0022ca18(host, local),
/// host +0xd0 = quadword 0x2a2dd0, host +0x9a = 3
pub fn ee_sub_12def0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let sp = stack_push(m, 0x100);
    let (s0, s1, s2) = (sp + 0xc0, sp + 0x80, sp + 0x40);
    let h = m.r32(o + 8);
    let b = m.r32(h + 0xa0);
    let a = m.r32(o + 0x1c) + 0x40;
    let src = if b != 0 {
        m.call(0x0011_c318, &[s0, b + 0x40], &[]);
        m.call(0x0011_c168, &[s1, s0, a], &[]);
        s1
    } else {
        a
    };
    m.call(0x0011_c168, &[s2, src, o + 0x20], &[]);
    let mt = m.rm(s2);
    m.wm(sp, mt);
    let h = m.r32(o + 8);
    let q = m.call(0x0022_ca18, &[h, sp], &[]).1;
    m.wv(h + 0xc0, q);
    let h = m.r32(o + 8);
    for k in 0..4 {
        let v = crate::engine::classes::q::q6::ro32(0x002a_2dd0 + 4 * k);
        m.w32(h + 0xd0 + 4 * k, v);
    }
    m.w16(h + 0x9a, 3);
    stack_pop(m, 0x100);
}
/// FUN_0012dfe0 (C2ca720_v4): `p` = host (+8) quadword +0xc0; `q` (12 bytes) cleared
pub fn ee_sub_12dfe0<M: QMem + ?Sized>(m: &mut M, o: u32, p: u32, q: u32) {
    if p != 0 {
        let h = m.r32(o + 8);
        for k in 0..4 {
            let v = m.r32(h + 0xc0 + 4 * k);
            m.w32(p + 4 * k, v);
        }
    }
    if q != 0 {
        m.w32(q, 0);
        m.w32(q + 8, 0);
        m.w32(q + 4, 0);
    }
}

// ---------------------------------------------------------------- class C2cb5a8 (blend vector curve, kind 5)
/// FUN_00138970 (C2cb5a8_v2, evaluate(t, rate, this, &pos, &vel)): the VU0 twin of the scalar
/// BlendScalarCurve::ee_sub_138ba0 (anim/blend.rs). u = t / blend length, or with a phase table (+8)
/// local phase time / phase length (FUN_00138370). Accumulators start at quadword 0x2a2dd0; per entry with
/// weight w != 0: child curve (+0xc)[i] evaluated through its vtable (+0x1c) at sub time u * sub length
/// (rate * sub length / blend length) or phase offset + u * phase rate (wrapped once by the sub length;
/// rate * phase rate / phase length) into stack temps; NULL child: quirk kept - the accumulators are RESET
/// to {default (+0x10)} / {0, 0, 0, w kept} and the stale temps are still added. acc = acc + vf0 + temp * w
/// (vadda with vf0: w += 1 per entry). Results stored when the out pointers are set
pub fn ee_sub_138970<M: QMem + ?Sized>(m: &mut M, o: u32, pos: u32, vel: u32, t: f32, rate: f32) {
    let sp = stack_push(m, 0x60);
    let pt = m.r32(o + 8);
    let (u, idx, ph) = if pt == 0 {
        let bl = { let t = m.r32(o + 4); m.rf(t + 0x74) };
        (t / bl, 0, 0)
    } else {
        let bl = { let t = m.r32(o + 4); m.rf(t + 0x74) };
        let (n, arr) = (m.r32(pt + 4), m.r32(pt + 8));
        let i = m.call(0x0013_8370, &[n, arr, sp + 0x50], &[s(t), s(bl)]).0;
        let ph = m.r32(pt + 8) + i * 8;
        (m.rf(sp + 0x50) / m.rf(ph + 4), i, ph)
    };
    let init = crate::engine::classes::q::q6::rov(0x002a_2dd0);
    m.wv(sp + 0x10, init);
    m.wv(sp, init);
    let mut i = 0u32;
    loop {
        let b = m.r32(o + 4);
        if (i as i32) >= m.r32(b + 0xc4) as i32 {
            break;
        }
        let e = m.r32(b + 0xc8) + i * m.r32(b + 0xc0);
        let w = m.rf(e);
        if w != 0.0 {
            let ch = { let t = m.r32(o + 0xc); m.r32(t + i * 4) };
            if ch == 0 {
                let d = m.rv(o + 0x10);
                m.wv(sp, d);
                m.wf(sp + 0x10, 0.0);
                m.wf(sp + 0x18, 0.0);
                m.wf(sp + 0x14, 0.0);
            } else {
                let sub = { let t = m.r32(e + 4); m.rf(t + 0x74) };
                let (st, r) = if pt == 0 {
                    (u * sub, rate * sub / m.rf(b + 0x74))
                } else {
                    let p = e + m.r32(pt) + idx * 8;
                    let (off, pr) = (m.rf(p), m.rf(p + 4));
                    let mut st = off + u * pr;
                    let r = rate * pr / m.rf(ph + 4);
                    if sub < st {
                        st -= sub;
                    }
                    (st, r)
                };
                vcall(m, ch, 0, 0x1c, &[sp + 0x20, sp + 0x30], &[s(st), s(r)]);
            }
            let w = m.rf(e);
            for (acc, tmp) in [(sp, sp + 0x20), (sp + 0x10, sp + 0x30)] {
                let (a, x) = (m.rv(acc), m.rv(tmp));
                m.wv(acc, [a[0] + x[0] * w, a[1] + x[1] * w, a[2] + x[2] * w, a[3] + 1.0 + x[3] * w]);
            }
            m.wv(sp + 0x40, [w, 0.0, 0.0, 0.0]);
        }
        i += 1;
    }
    if pos != 0 {
        let v = m.rv(sp);
        m.wv(pos, v);
    }
    if vel != 0 {
        let v = m.rv(sp + 0x10);
        m.wv(vel, v);
    }
    stack_pop(m, 0x60);
}
/// FUN_002581b8 (C2cb5a8_v3): curve kind 5
pub fn ee_sub_2581b8<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    5
}

// ---------------------------------------------------------------- class C2ccbb8 (guard type, base C2c8940)
/// FUN_0014b1d8 (C2ccbb8_v14): base C2c8940_v14 (FUN_002061d8), +0x1010 = objects_create_and_load(3, 0,
/// 0x62b, -1) (FUN_0013e3c8)
pub fn ee_sub_14b1d8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0020_61d8, &[o], &[]);
    let x = m.call(0x0013_e3c8, &[3, 0, 0x62b, u32::MAX], &[]).0;
    m.w32(o + 0x1010, x);
}
/// FUN_0014b218 (C2ccbb8_v4, message): base C2c8940_v4 (FUN_002075d0); not state 0x24 (FUN_00209a68) and
/// message 0xb sent by this object -> FUN_00154038(sensor +0xf64, msg[1], 1)
pub fn ee_sub_14b218<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32, msg: u32) {
    m.call(0x0020_75d0, &[o, k, msg], &[]);
    if m.call(0x0020_9a68, &[o, 0x24], &[]).0 == 0 && k == 0xb && m.r32(msg) == o {
        let sn = { let t = m.r32(msg); m.r32(t + 0xf64) };
        let key = m.r32(msg + 4);
        ee_sub_154038(m, sn, key, 1);
    }
}
/// FUN_0014b290 (C2ccbb8_v87, pick target): in state 0x24 the base (FUN_002082d0); else +0x390 when set with
/// +0x28 bit 60 set; else the sensed object (sensor +0xf64 entries, 16 bytes from +0x100, live when +4 != 0)
/// with the lowest class rank (0x34 -> 0, 0x3a -> 1, else 2), ties by distance (the current target +0x38c
/// scored DAT_002bd4b0 + d * (DAT_002bd4b4 + d * DAT_002bd4b8)); start distance 0x2a4ee8, 0 when none
pub fn ee_sub_14b290<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    if m.call(0x0020_9a68, &[o, 0x24], &[]).0 != 0 {
        return m.call(0x0020_82d0, &[o], &[]).0;
    }
    let r = m.r32(o + 0x390);
    if r != 0 && (m.r64(r + 0x28) >> 60) & 1 != 0 {
        return r;
    }
    let mut best_d = crate::engine::classes::q::q6::rof(0x002a_4ee8);
    let mut rank = 999u32;
    let mut best = 0u32;
    let me = m.call(0x001a_62e8, &[o], &[]).1;
    let mut i = 0u32;
    while (i as i32) < { let t = m.r32(o + 0xf64); m.r32(t + 0xfc) } as i32 {
        let p = m.r32(o + 0xf64) + 0x100 + i * 0x10;
        if m.r32(p + 4) != 0 {
            let x = m.r32(p);
            let k = if m.call(0x0013_2b10, &[x, 0x34], &[]).0 != 0 {
                0
            } else if m.call(0x0013_2b10, &[x, 0x3a], &[]).0 != 0 {
                1
            } else {
                2
            };
            let q = m.call(0x001a_62e8, &[x], &[]).1;
            let (dx, dy, dz) = (me[0] - q[0], me[1] - q[1], me[2] - q[2]);
            let mut d = (dx * dx + dy * dy + dz * dz).sqrt();
            if m.r32(p) == m.r32(o + 0x38c) {
                d = m.rf(0x002b_d4b0) + d * (m.rf(0x002b_d4b4) + d * m.rf(0x002b_d4b8));
            }
            if k < rank || (k == rank && d < best_d) {
                rank = k;
                best_d = d;
                best = m.r32(p);
            }
        }
        i += 1;
    }
    best
}

// ---------------------------------------------------------------- class C2d0dd8 (base C2d5898)
/// FUN_00181408 (C2d0dd8_v6): base C2d1878_v6 (FUN_0017f090), FUN_0017f3e0(this, 3, this + 0x1d0)
pub fn ee_sub_181408<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0017_f090, &[o], &[]);
    m.call(0x0017_f3e0, &[o, 3, o + 0x1d0], &[]);
}
/// FUN_00181440 (C2d0dd8_v14): base C2d1328_v14 (FUN_0017f138), FUN_0017f408(this, 3, this + 0x1d0)
pub fn ee_sub_181440<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0017_f138, &[o], &[]);
    m.call(0x0017_f408, &[o, 3, o + 0x1d0], &[]);
}
/// FUN_00181478 (C2d0dd8_v15, update): base C2caf88_v15 (FUN_001f36e0); state machine +0x1c0 re-run while
/// it changes: 0 -> player DAT_002dfb80 within DAT_002be880 of the anchor +0x1cc: +0x1c8 = player, state 1;
/// 1 -> FUN_0017f550 -> state 2
pub fn ee_sub_181478<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001f_36e0, &[o], &[]);
    let mut st = m.r32(o + 0x1c0);
    loop {
        if st == 0 {
            let p = m.r32(0x002d_fb80);
            if p != 0 {
                let a = m.call(0x001a_62e8, &[p], &[]).1;
                let an = m.r32(o + 0x1cc);
                let b = m.call(0x001a_62e8, &[an], &[]).1;
                let (dx, dy, dz) = (a[0] - b[0], a[1] - b[1], a[2] - b[2]);
                let d = (dx * dx + dy * dy + dz * dz).sqrt();
                if d < m.rf(0x002b_e880) {
                    m.w32(o + 0x1c8, p);
                    ee_sub_1815e8(m, o, 1);
                }
            }
        } else if st == 1 && m.call(0x0017_f550, &[o], &[]).0 != 0 {
            ee_sub_1815e8(m, o, 2);
        }
        let n = m.r32(o + 0x1c0);
        if n == st {
            return;
        }
        st = n;
    }
}
/// FUN_00181598 (C2d0dd8_v82): base C2d1328_v82 (FUN_0017f6b0), state 0 (FUN_001815e8),
/// FUN_0020ff70(0x3dd490, +0x200, this + 0x204, this, 0)
pub fn ee_sub_181598<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0017_f6b0, &[o], &[]);
    ee_sub_1815e8(m, o, 0);
    let a = m.r32(o + 0x200);
    m.call(0x0020_ff70, &[0x003d_d490, a, o + 0x204, o, 0], &[]);
}
/// FUN_001815e8: state change to `k`: leaving 1 with a player (+0x1c8) -> FUN_00181728(this); +0x1c4 =
/// DAT_002dded0; 1 -> player +0x294 = anchor +0x1cc, FUN_0017fd08(player, 3); 2 -> FollowWorld_activity
/// (0x2deb30, 0x39, this); FUN_0017f4c0(this, k == -1 ? 0 : this + 0x1d0 + k * 0x10)
pub fn ee_sub_1815e8<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32) {
    let cur = m.r32(o + 0x1c0);
    if cur == k {
        return;
    }
    if cur == 1 && m.r32(o + 0x1c8) != 0 {
        m.call(0x0018_1728, &[o], &[]);
    }
    m.w32(o + 0x1c0, k);
    let c = m.r32(0x002d_ded0);
    m.w32(o + 0x1c4, c);
    if k == 1 {
        let p = m.r32(o + 0x1c8);
        let an = m.r32(o + 0x1cc);
        m.w32(p + 0x294, an);
        m.call(0x0017_fd08, &[p, 3], &[]);
    } else if k == 2 {
        m.call(0x0016_19a0, &[0x002d_eb30, 0x39, o], &[]);
    }
    let st = if k == u32::MAX { 0 } else { o + k.wrapping_mul(0x10) + 0x1d0 };
    m.call(0x0017_f4c0, &[o, st], &[]);
}

// ---------------------------------------------------------------- class C2d1328 (base C2d0dd8)
/// FUN_00180188 (C2d1328_v6): base C2d1878_v6 (FUN_0017f090), +0x188 = 10
pub fn ee_sub_180188<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0017_f090, &[o], &[]);
    m.w32(o + 0x188, 10);
}
/// xyz = r0 * v.x + r1 * v.y + r2 * v.z, w = `w` (VU0 vmulax / vmadday / vmaddz.xyz)
fn mul3(r: &[V4; 3], v: V4, w: f32) -> V4 {
    let mut o = [0.0; 4];
    for k in 0..3 {
        o[k] = r[0][k] * v[0] + r[1][k] * v[1] + r[2][k] * v[2];
    }
    o[3] = w;
    o
}
/// FUN_001801b8 (C2d1328_v15, update): base C2caf88_v15 (FUN_001f36e0); +0x18c == 0 and +0x1c0 <= clock ->
/// vslot +0x294(this, 0). With a target +0x1d0 and turn speed +0x1d4 != 0: turn = FUN_0011cfe0(
/// FUN_0011cf60(heading(target - this) - heading(forward)), +0x1d4 * DAT_002dded4) (headings atan2(y, x),
/// forward = vslot +0x10c); R = MatrixFromAxisAngle(0x2a2dc0, turn) (FUN_0011b500); vslot +0x11c(this,
/// R * forward); vslot +0xf4(this, R * M) where M = the 3 quadwords at local +0x30 that the original never
/// writes (stale stack, kept; row 2 w = R row 0 w by the vmaddz.xyz destination)
pub fn ee_sub_1801b8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let sp = stack_push(m, 0xd0);
    m.call(0x001f_36e0, &[o], &[]);
    if m.r32(o + 0x18c) == 0 && m.rf(o + 0x1c0) <= m.rf(CLOCK) {
        vcall(m, o, 0xc, 0x294, &[0], &[]);
    }
    let tg = m.r32(o + 0x1d0);
    if tg != 0 && m.rf(o + 0x1d4) != 0.0 {
        let a = m.call(0x001a_62e8, &[tg], &[]).1;
        let b = m.call(0x001a_62e8, &[o], &[]).1;
        let h0 = callf(m, 0x0024_58f0, &[], &[s(a[1] - b[1]), s(a[0] - b[0])]);
        let fw = vcall(m, o, 0xc, 0x10c, &[], &[]).1;
        let h1 = callf(m, 0x0024_58f0, &[], &[s(fw[1]), s(fw[0])]);
        let d = callf(m, 0x0011_cf60, &[], &[s(h0 - h1)]);
        let k = m.rf(o + 0x1d4) * m.rf(0x002d_ded4);
        let ang = callf(m, 0x0011_cfe0, &[], &[s(d), s(k)]);
        let axis = crate::engine::classes::q::q6::rov(0x002a_2dc0);
        m.call(0x0011_b500, &[sp + 0x60], &[axis, s(ang)]);
        let r = [m.rv(sp + 0x60), m.rv(sp + 0x70), m.rv(sp + 0x80)];
        for (i, x) in r.iter().enumerate() {
            m.wv(sp + 0x10 * i as u32, *x);
        }
        let v = mul3(&r, fw, fw[3]);
        vcall(m, o, 0xc, 0x11c, &[], &[v]);
        let (m0, m1, m2) = (m.rv(sp + 0x30), m.rv(sp + 0x40), m.rv(sp + 0x50));
        let rows = [mul3(&r, m0, m0[3]), mul3(&r, m1, m1[3]), mul3(&r, m2, r[0][3])];
        for (i, x) in rows.iter().enumerate() {
            for base in [0x30, 0x60, 0x90] {
                m.wv(sp + base + 0x10 * i as u32, *x);
            }
        }
        vcall(m, o, 0xc, 0xf4, &[sp + 0x30], &[]);
    }
    stack_pop(m, 0xd0);
}
/// FUN_001803a8 (C2d1328_v60): 3 when a player (+0x1c8) is set and `x` shares its +0xec, else base
/// C2cc548_v60 (FUN_001f4fc8)
pub fn ee_sub_1803a8<M: QMem + ?Sized>(m: &mut M, o: u32, x: u32) -> u32 {
    let p = m.r32(o + 0x1c8);
    if p != 0 && m.r32(x + 0xec) == m.r32(p + 0xec) {
        return 3;
    }
    m.call(0x001f_4fc8, &[o, x], &[]).0
}
/// FUN_001803e0 (C2d1328_v54, contact): base C2d0dd8_v54 (FUN_0017f1a0); other = c[(1 - side) * 0x10 + 3];
/// (not class 0x47 and anchor +0x1cc set) or (FUN_0017f5b0(this, other), then +0x1cc set) -> c +0x98 = 1.0
pub fn ee_sub_1803e0<M: QMem + ?Sized>(m: &mut M, o: u32, c: u32, side: u32) {
    m.call(0x0017_f1a0, &[o, c, side], &[]);
    let x = m.r32(c.wrapping_add(1u32.wrapping_sub(side).wrapping_mul(0x40)) + 0xc);
    let hit = (m.call(0x0013_2b10, &[x, 0x47], &[]).0 == 0 && m.r32(o + 0x1cc) != 0) || {
        m.call(0x0017_f5b0, &[o, x], &[]);
        m.r32(o + 0x1cc) != 0
    };
    if hit {
        m.wf(c + 0x98, 1.0);
    }
}

// ---------------------------------------------------------------- class C2d8dd8 (track-attached point sets, base C2d5450)
/// xyz = r0 * v.x + r1 * v.y + r2 * v.z (+ t), w = `w`
fn lin3(r: [V4; 3], v: V4, t: [f32; 3], w: f32) -> V4 {
    let mut o = [0.0; 4];
    for k in 0..3 {
        o[k] = r[0][k] * v[0] + r[1][k] * v[1] + r[2][k] * v[2] + t[k];
    }
    o[3] = w;
    o
}
/// FUN_001e9858 (C2d8dd8_v14, post-load): base C2d20d0_v14 (FUN_001f36a8); with a track (+0x150) and point
/// sets (+0xdc list: +4 item count, +8 items of 0x20 {+0 n, +0xc local points, +0x1c track-space points},
/// +0xc next): the node's owner record (+0xc0 + ((*node - +0xbc) * 0xcccccccd >> 2) * 0x28) gets +4 =
/// 0x2c44f0 (old object kept as the frame source), FUN_0016ef00(this + 0x80); every local point
/// (copied to a 0x800-byte local first) goes to world space through the old object's matrix (+0x40 rows,
/// +0x70 translation; w = row 0 w), is projected onto the track (FUN_001e6f60 -> base point, matrix
/// input, param at local +0x8b0), rotated into the track frame (FUN_0011b830 with the orientation
/// 1a6318()+0x20, transposed) and stored with +0xc = the track param in a new array (FUN_001ae1d0
/// {n * 16, 0x10})
pub fn ee_sub_1e9858<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001f_36a8, &[o], &[]);
    if m.r32(o + 0x150) == 0 || m.r32(o + 0xdc) == 0 {
        return;
    }
    let sp = stack_push(m, 0x900);
    let x = m.call(0x001a_6318, &[o], &[]).0;
    let rot = m.rv(x + 0x20);
    m.wv(sp + 0x8c0, rot);
    let mut node = m.r32(o + 0xdc);
    while node != 0 {
        let d = m.r32(node).wrapping_sub(m.r32(o + 0xbc));
        let idx = (d.wrapping_mul(0xcccc_cccd) as i32) >> 2;
        let rec = m.r32(o + 0xc0).wrapping_add((idx as u32).wrapping_mul(0x28));
        let fp = m.r32(rec + 4);
        m.w32(rec + 4, 0x002c_44f0);
        m.call(0x0016_ef00, &[o + 0x80], &[]);
        let mut j = 0u32;
        while (j as i32) < m.r32(node + 4) as i32 {
            let it = m.r32(node + 8) + j * 0x20;
            j += 1;
            let n = m.r32(it);
            let src = m.r32(it + 0xc);
            for b in 0..n.wrapping_mul(16) {
                let v = m.r8(src + b);
                m.w8(sp + b, v);
            }
            m.w32(sp + 0x800, n << 4);
            m.w32(sp + 0x804, 0x10);
            let mut dst = m.call(0x001a_e1d0, &[sp + 0x800], &[]).0;
            m.w32(it + 0x1c, dst);
            let mut k = 0u32;
            while (k as i32) < m.r32(it) as i32 {
                let p = m.rv(sp + k * 0x10);
                let r = [m.rv(fp + 0x40), m.rv(fp + 0x50), m.rv(fp + 0x60)];
                let t = m.rv(fp + 0x70);
                let w = lin3(r, p, [t[0], t[1], t[2]], r[0][3]);
                m.wv(sp + 0x8e0, w);
                let tr = m.r32(o + 0x150);
                m.call(0x001e_6f60, &[tr, 0, sp + 0x800, sp + 0x810, sp + 0x8b0, 0], &[w]);
                let q1 = m.rv(sp + 0x810);
                let q2 = m.rv(sp + 0x8c0);
                m.call(0x0011_b830, &[sp + 0x850], &[q1, q2]);
                let rr = [m.rv(sp + 0x850), m.rv(sp + 0x860), m.rv(sp + 0x870)];
                for (i, x) in rr.iter().enumerate() {
                    m.wv(sp + 0x820 + 0x10 * i as u32, *x);
                }
                // transpose with w = 0 (pextlw / pextuw / pcpyld / pcpyud)
                let tt = [
                    [rr[0][0], rr[1][0], rr[2][0], 0.0],
                    [rr[0][1], rr[1][1], rr[2][1], 0.0],
                    [rr[0][2], rr[1][2], rr[2][2], 0.0],
                ];
                for (i, x) in tt.iter().enumerate() {
                    m.wv(sp + 0x850 + 0x10 * i as u32, *x);
                    m.wv(sp + 0x880 + 0x10 * i as u32, *x);
                }
                let b = m.rv(sp + 0x800);
                let dv = [w[0] - b[0], w[1] - b[1], w[2] - b[2], w[3] - b[3]];
                let l = lin3(tt, dv, [0.0; 3], dv[3]);
                m.wv(dst, l);
                let pr = m.r32(sp + 0x8b0);
                m.w32(dst + 0xc, pr);
                dst += 0x10;
                k += 1;
            }
        }
        node = m.r32(node + 0xc);
    }
    stack_pop(m, 0x900);
}
/// FUN_001e9ac0 (C2d8dd8_v18, update): base C2caf88_v18 (FUN_001f3c70) before and after; track time
/// t = FUN_001e6630(+0x150); every track-space point (item +0x1c) at min(t, point w): FUN_001e67b8(track,
/// &pos, &q, time), frame FUN_0011b830(q, orientation 1a6318()+0x20); local points (item +0xc) xyz =
/// frame * point + pos (w untouched)
pub fn ee_sub_1e9ac0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let sp = stack_push(m, 0x80);
    m.call(0x001f_3c70, &[o], &[]);
    let tr = m.r32(o + 0x150);
    let t = callf(m, 0x001e_6630, &[tr], &[]);
    let x = m.call(0x001a_6318, &[o], &[]).0;
    let rot = m.rv(x + 0x20);
    let mut node = m.r32(o + 0xdc);
    while node != 0 {
        let mut j = 0u32;
        while (j as i32) < m.r32(node + 4) as i32 {
            let it = m.r32(node + 8) + j * 0x20;
            j += 1;
            let mut s1 = m.r32(it + 0x1c);
            let mut s0 = m.r32(it + 0xc);
            let mut k = 0u32;
            while (k as i32) < m.r32(it) as i32 {
                let tw = m.rf(s1 + 0xc);
                let tm = if t < tw { t } else { tw };
                let tr = m.r32(o + 0x150);
                m.call(0x001e_67b8, &[tr, sp, sp + 0x10], &[s(tm)]);
                let q = m.rv(sp + 0x10);
                m.call(0x0011_b830, &[sp + 0x50], &[q, rot]);
                let r = [m.rv(sp + 0x50), m.rv(sp + 0x60), m.rv(sp + 0x70)];
                for (i, x) in r.iter().enumerate() {
                    m.wv(sp + 0x20 + 0x10 * i as u32, *x);
                }
                let p = m.rv(sp);
                let v = lin3(r, m.rv(s1), [p[0], p[1], p[2]], 0.0);
                m.wf(s0, v[0]);
                m.wf(s0 + 8, v[2]);
                m.wf(s0 + 4, v[1]);
                s1 += 0x10;
                s0 += 0x10;
                k += 1;
            }
        }
        node = m.r32(node + 0xc);
    }
    m.call(0x001f_3c70, &[o], &[]);
    stack_pop(m, 0x80);
}

// ---------------------------------------------------------------- class C2d7ca8 (vehicle, base C2d5898)
/// FUN_001fab78(this, a, b): r1 = FUN_001faa00(this, b), r2 = FUN_001faa48(this, a, b);
/// (+0x58)+0xf0 += FUN_0022fcc8(0, (+0x58)+0xa0, r1), (+0x5c)+0xf0 += FUN_0022fcc8(0, (+0x5c)+0xa0, r2)
/// (disassembly: the second add uses FUN_001faa48's result; the decompile shows r1 twice)
pub fn ee_sub_1fab78<M: QMem + ?Sized>(m: &mut M, o: u32, a: V4, b: V4) {
    let r1 = m.call(0x001f_aa00, &[o], &[b]).1;
    let r2 = m.call(0x001f_aa48, &[o], &[a, b]).1;
    for (off, r) in [(0x58, r1), (0x5c, r2)] {
        let p = m.r32(o + off);
        let v = m.rv(p + 0xf0);
        let k = m.r32(p + 0xa0);
        let q = m.call(0x0022_fcc8, &[0, k], &[r]).1;
        m.wv(p + 0xf0, [v[0] + q[0], v[1] + q[1], v[2] + q[2], v[3] + q[3]]);
    }
}
fn dot3(a: V4, b: V4) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn clamp01(x: f32) -> f32 {
    if x < 0.0 {
        0.0
    } else if 1.0 < x {
        1.0
    } else {
        x
    }
}
/// FUN_001d2e18 (C2d7ca8_v69, vehicle forces): ApplyAloGravity (FUN_001f9b20); more than 3 s since +0x5c0 ->
/// FUN_0011b938(1a6318(), 0, m), FUN_001f5c30(this, m, 0x2c2100, 0x2a2dd0, 0). Per wheel i < +0x190 (0x60-byte
/// records at +0x1a0) with contact c = FUN_001d2dd0: compression u = clamp01(c+0x90 / (w+0x28 - w+0x24)),
/// spring = (dot(p(c+0x44) - p(+0x60), up) * polyA(u) * DAT_2c2110 + c+0x90 * polyB(u) * DAT_2c2114)
/// (polys 0x2c2120 / 0x2c2130, p = FUN_0022d348(.., 0, contact point c+0x10), up = 1a6318()+0x20); when
/// >= 0: force = up * spring * (+0xe8)+0xd0 applied at the contact (FUN_001fab78), load += dot(force, up).
/// Then drive / steering: r = FUN_0022fcc8((+0x58)+0xa0, +0x5c, pos + vel * dt (w + 1)); local velocity
/// lv = rows(+0x5c) . (+0x5c)+0xb0; grip = min(DAT_2c2140, +0x53c * load / mass); k = DAT_2c2144 * mass *
/// +0x4a0 / (+0xe8)+0x28; yaw = FUN_0011cfe0((+0x554 - lv.z) / dt, k * grip); torque (FUN_001fac98) =
/// (+0x5c) 0x40 rows * (0, 0, yaw); grip -= |DAT_2c2148 * yaw / k|; R = MatrixFromAxisAngle(0x2a2dc0,
/// (lv.z + yaw * dt / 2) * dt); lateral = (R * (+0x558 * +0x550, 0, 0) - r) / dt (z = 0), x clamped to the
/// throttle window (+0x550 >= 0: [-(+0x538), clamp(poly +0x520(clamp01(r.x / +0x51c)), +0x530, +0x534)],
/// else mirrored); AddAloAccel(this, FUN_0022fcc8(+0x5c, 0, FUN_0011e9f8(lateral, grip))). Stale stack w
/// lanes (locals +0x3c, +0x8c) are read as in the original
pub fn ee_sub_1d2e18<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let sp = stack_push(m, 0x110);
    m.call(0x001f_9b20, &[o], &[]);
    let clock = m.rf(CLOCK);
    if 3.0 < clock - m.rf(o + 0x5c0) {
        let x = m.call(0x001a_6318, &[o], &[]).0;
        m.call(0x0011_b938, &[x, 0, sp], &[]);
        let q = crate::engine::classes::q::q6::rov(0x002a_2dd0);
        m.call(0x001f_5c30, &[o, sp, 0x002c_2100, 0], &[q]);
    }
    let mut load = 0.0f32;
    let mut i = 0u32;
    while (i as i32) < m.r32(o + 0x190) as i32 {
        let w = o + 0x1a0 + i * 0x60;
        i += 1;
        let c = m.call(0x001d_2dd0, &[o, w], &[]).0;
        if c == 0 {
            continue;
        }
        let pt = m.rv(c + 0x10);
        let x = m.call(0x001a_6318, &[o], &[]).0;
        let up = m.rv(x + 0x20);
        m.wv(sp + 0xd0, up);
        let a0 = m.r32(o + 0x60);
        let a = m.call(0x0022_d348, &[a0, 0], &[pt]).1;
        m.wv(sp + 0xe0, a);
        let b0 = m.r32(c + 0x44);
        let b = m.call(0x0022_d348, &[b0, 0], &[pt]).1;
        let comp = m.rf(c + 0x90);
        let u = clamp01(comp / (m.rf(w + 0x28) - m.rf(w + 0x24)));
        let pa = m.rf(0x002c_2120) + u * (m.rf(0x002c_2124) + u * m.rf(0x002c_2128));
        let pb = m.rf(0x002c_2130) + u * (m.rf(0x002c_2134) + u * m.rf(0x002c_2138));
        let d = dot3([b[0] - a[0], b[1] - a[1], b[2] - a[2], 0.0], up);
        let f = d * pa * m.rf(0x002c_2110) + m.rf(c + 0x90) * pb * m.rf(0x002c_2114);
        if f < 0.0 {
            continue;
        }
        let s0 = { let t = m.r32(o + 0xe8); m.rf(t + 0xd0) };
        let k = f * s0;
        let force = [up[0] * k, up[1] * k, up[2] * k, up[3] * k];
        ee_sub_1fab78(m, o, pt, force);
        load += dot3(force, up);
    }
    let dt = m.rf(0x002d_ded4);
    let mass = { let t = m.r32(o + 0xe8); m.rf(t + 0xd0) };
    let g = m.rf(o + 0x53c) * load / mass;
    let b58 = m.r32(o + 0x58);
    let (p, v) = (m.rv(b58 + 0xb0), m.rv(b58 + 0xf0));
    let np = [p[0] + v[0] * dt, p[1] + v[1] * dt, p[2] + v[2] * dt, p[3] + 1.0 + v[3] * dt];
    let lim = m.rf(0x002c_2140);
    let mut grip = if lim < g { lim } else { g };
    let k0 = m.r32(b58 + 0xa0);
    let b5c = m.r32(o + 0x5c);
    let r = m.call(0x0022_fcc8, &[k0, b5c], &[np]).1;
    m.wv(sp + 0x10, r);
    let b5c = m.r32(o + 0x5c);
    let pv = m.rv(b5c + 0xb0);
    let rows = [m.rv(b5c), m.rv(b5c + 0x10), m.rv(b5c + 0x20)];
    let lv = [dot3(rows[0], pv), dot3(rows[1], pv), dot3(rows[2], pv), rows[0][3]];
    m.wv(sp + 0x20, lv);
    let e8 = m.r32(o + 0xe8);
    let k = m.rf(0x002c_2144) * m.rf(e8 + 0xd0) * m.rf(o + 0x4a0) / m.rf(e8 + 0x28);
    m.wf(sp + 0x34, 0.0);
    m.wf(sp + 0x30, 0.0);
    let want = (m.rf(o + 0x554) - lv[2]) / dt;
    let yaw = callf(m, 0x0011_cfe0, &[], &[s(want), s(k * grip)]);
    m.wf(sp + 0x38, yaw);
    let damp = m.rf(0x002c_2148) * yaw / k;
    let b5c = m.r32(o + 0x5c);
    let r4 = [m.rv(b5c + 0x40), m.rv(b5c + 0x50), m.rv(b5c + 0x60)];
    let tv = m.rv(sp + 0x30);
    let torque = mul3(&r4, tv, tv[3]);
    m.wv(sp + 0x40, torque);
    grip -= damp.abs();
    m.call(0x001f_ac98, &[o], &[torque]);
    let dt = m.rf(0x002d_ded4);
    let ang = (lv[2] + yaw * (dt * 0.5)) * dt;
    let axis = crate::engine::classes::q::q6::rov(0x002a_2dc0);
    m.call(0x0011_b500, &[sp + 0x90], &[axis, s(ang)]);
    let thr = m.rf(o + 0x550);
    m.wf(sp + 0x84, 0.0);
    m.wf(sp + 0x88, 0.0);
    let sx = m.rf(o + 0x558) * thr;
    m.wf(sp + 0x80, sx);
    let r2 = [m.rv(sp + 0x90), m.rv(sp + 0xa0), m.rv(sp + 0xb0)];
    let sv = m.rv(sp + 0x80);
    let rv = mul3(&r2, sv, sv[3]);
    m.wv(sp + 0x80, rv);
    for (j, x) in r2.iter().enumerate() {
        m.wv(sp + 0x50 + 0x10 * j as u32, *x);
    }
    let rx = m.rf(sp + 0x10) / m.rf(o + 0x51c);
    let poly = |m: &mut M, x: f32| {
        let u = clamp01(x);
        let pl = m.rf(o + 0x520) + u * (m.rf(o + 0x524) + u * m.rf(o + 0x528));
        let (lo, hi) = (m.rf(o + 0x530), m.rf(o + 0x534));
        if pl < lo {
            lo
        } else if hi < pl {
            hi
        } else {
            pl
        }
    };
    let (lo, hi) = if 0.0 <= thr {
        (-m.rf(o + 0x538), poly(m, rx))
    } else {
        (-poly(m, -rx), m.rf(o + 0x538))
    };
    let r0 = m.rv(sp + 0x10);
    m.wf(sp + 0xc0, dt);
    let q = 1.0 / dt;
    let mut lat = [(rv[0] - r0[0]) * q, (rv[1] - r0[1]) * q, 0.0, (rv[3] - r0[3]) * q];
    lat[0] = if lat[0] < lo {
        lo
    } else if hi < lat[0] {
        hi
    } else {
        lat[0]
    };
    m.wv(sp + 0x90, lat);
    let a = m.call(0x0011_e9f8, &[], &[lat, s(grip)]).1;
    m.wv(sp + 0xb0, a);
    let b5c = m.r32(o + 0x5c);
    let acc = m.call(0x0022_fcc8, &[b5c, 0], &[a]).1;
    m.wv(sp + 0xa0, acc);
    m.call(0x001f_ac30, &[o], &[acc]);
    stack_pop(m, 0x110);
}

// ---------------------------------------------------------------- classes C2ca928 / C2ca900 (per-axis vector curves)
/// slot 2 body: out pos (a1) = default +0x20 quad, out vel (a2) xyz = 0; with keys (+4): segment k
/// (FUN_0012c0e0 / FUN_0012c708) and per present axis i (+0xc + 4i, key stride `st`): FUN_00117030(t_k -
/// t_k-1, t - t_k-1, rate, prev[0], prev[`tv`], next[0], next[1]) -> pos[i] / vel[i]
fn axis_eval<M: QMem + ?Sized>(m: &mut M, o: u32, pos: u32, vel: u32, t: f32, rate: f32, seg: u32, st: u32, tv: u32) {
    if pos != 0 {
        let q = m.rv(o + 0x20);
        m.wv(pos, q);
    }
    if vel != 0 {
        m.w32(vel + 8, 0);
        m.w32(vel + 4, 0);
        m.w32(vel, 0);
    }
    if m.r32(o + 4) == 0 {
        return;
    }
    let k = m.call(seg, &[o], &[s(t)]).0;
    for i in 0..3u32 {
        let a = m.r32(o + 0xc + 4 * i);
        if a == 0 {
            continue;
        }
        let nx = a.wrapping_add(k.wrapping_mul(st));
        let pv = nx.wrapping_sub(st);
        let tp = m.r32(o + 8).wrapping_add(k.wrapping_mul(4));
        let t0 = m.rf(tp.wrapping_sub(4));
        let t1 = m.rf(tp);
        let (p0, v0, p1, v1) = (m.rf(pv), m.rf(pv + tv), m.rf(nx), m.rf(nx + 4));
        let (p, v) = crate::engine::anim::keys::ee_sub_117030(t1 - t0, t - t0, rate, p0, v0, p1, v1);
        if pos != 0 {
            m.wf(pos + 4 * i, p);
        }
        if vel != 0 {
            m.wf(vel + 4 * i, v);
        }
    }
}
/// FUN_0012c440 (C2ca928_v2): per-axis {value, in, out} keys (stride 12; prev out tangent = word 2)
pub fn ee_sub_12c440<M: QMem + ?Sized>(m: &mut M, o: u32, pos: u32, vel: u32, t: f32, rate: f32) {
    axis_eval(m, o, pos, vel, t, rate, 0x0012_c0e0, 0xc, 8);
}
/// FUN_0012ca68 (C2ca900_v2): per-axis {value, tangent} keys (stride 8)
pub fn ee_sub_12ca68<M: QMem + ?Sized>(m: &mut M, o: u32, pos: u32, vel: u32, t: f32, rate: f32) {
    axis_eval(m, o, pos, vel, t, rate, 0x0012_c708, 8, 4);
}

// ---------------------------------------------------------------- class C2ca7a0 (look-at binding)
/// R(rows +0x40/+0x50/+0x60 of `x`) * v (xyz; w kept)
fn rot_by<M: QMem + ?Sized>(m: &mut M, x: u32, v: V4) -> V4 {
    let r = [m.rv(x + 0x40), m.rv(x + 0x50), m.rv(x + 0x60)];
    mul3(&r, v, v[3])
}
/// FUN_0012d508 (C2ca7a0_v3): no target (+0x24): first ancestor of node (+8) via +0xa0 with +0x94 == 1
/// becomes the target; none -> node rotation +0xc0..+0xef = node +0..+0x2f. With a target: its binder
/// (+0x90) -> AnimBinder_locals(+4, target); direction v by target kind (u16 +0x98 - 1): 0,1,6,7,8,10,16,
/// 17,19,20 -> +0xb0; 4,5 -> sub-kind +0x9a: 1 -> Hermite velocity (FUN_00117120 over +0xc0..+0xf0),
/// 3,4 -> +0xd0, else DAT_002a2dd0; other kinds -> 0; v rotated by the target parent (+0xa0) rows.
/// |v| < 100 -> FUN_0011b830(node +0x40, 0x2a2dc0); else up = DAT_002bc380 (+0x1c / +0x20 > 0 and a
/// binder: up += (0,0,0,1) + (binder slot +0x2c position (parent-rotated) - v) * +0x1c / +0x20) and
/// FUN_0011b830(v, up). The 3 rows go through FUN_0022fd38(.., 0, node +0xa0, ..) into node +0xc0; +0x9a = 5
pub fn ee_sub_12d508<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let sp = stack_push(m, 0x90);
    if m.r32(o + 0x24) == 0 {
        let nd = m.r32(o + 8);
        let mut a = m.r32(nd + 0xa0);
        while a != 0 && m.r32(a + 0x94) != 1 {
            a = m.r32(a + 0xa0);
        }
        if a == 0 {
            for k in 0..3 {
                let q = m.rv(nd + 0x10 * k);
                m.wv(nd + 0xc0 + 0x10 * k, q);
            }
            let nd = m.r32(o + 8);
            m.w16(nd + 0x9a, 5);
            stack_pop(m, 0x90);
            return;
        }
        m.w32(o + 0x24, a);
    }
    let tg = m.r32(o + 0x24);
    let b = m.r32(tg + 0x90);
    if b != 0 {
        let x = m.r32(b + 4);
        m.call(0x0022_dd00, &[x, tg], &[]);
    }
    let t2 = m.r32(o + 0x24);
    let kind = (m.r16(t2 + 0x98) as i32 - 1) << 16 >> 16;
    let mut v = match kind {
        0 | 1 | 6 | 7 | 8 | 10 | 16 | 17 | 19 | 20 => m.rv(tg + 0xb0),
        4 | 5 => match m.r16(tg + 0x9a) as i16 {
            1 => {
                let t0 = m.rf(tg + 0x80);
                let dt = m.rf(tg + 0x84) - t0;
                let p = m.r32(tg + 0x88);
                let u = m.rf(p) - t0;
                let r = m.r32(tg + 0x8c);
                let rate = m.rf(r);
                let q = [m.rv(tg + 0xc0), m.rv(tg + 0xd0), m.rv(tg + 0xe0), m.rv(tg + 0xf0)];
                m.call(0x0011_7120, &[0, sp, 0], &[s(dt), s(u), s(rate), q[0], q[1], q[2], q[3]]);
                m.rv(sp)
            }
            3 | 4 => m.rv(tg + 0xd0),
            _ => crate::engine::classes::q::q6::rov(0x002a_2dd0),
        },
        _ => [0.0; 4],
    };
    let pa = m.r32(tg + 0xa0);
    if pa != 0 {
        v = rot_by(m, pa, v);
    }
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 100.0 {
        let nd = m.r32(o + 8);
        let a = m.rv(nd + 0x40);
        let up = crate::engine::classes::q::q6::rov(0x002a_2dc0);
        m.call(0x0011_b830, &[sp + 0x40], &[a, up]);
    } else {
        let mut up = crate::engine::classes::q::q6::rov(0x002b_c380);
        if 0.0 < m.rf(o + 0x20) && 0.0 < m.rf(o + 0x1c) && b != 0 {
            let vt = m.r32(b + 0x18);
            let h = m.r16(vt + 0x28) as i16 as i32;
            let f = m.r32(vt + 0x2c);
            m.call(f, &[b.wrapping_add(h as u32), 0, sp], &[]);
            let pa = m.r32(tg + 0xa0);
            if pa != 0 {
                let w = m.rv(sp);
                let w = rot_by(m, pa, w);
                m.wv(sp, w);
            }
            let k = m.rf(o + 0x1c) / m.rf(o + 0x20);
            let w = m.rv(sp);
            up = [
                up[0] + (w[0] - v[0]) * k,
                up[1] + (w[1] - v[1]) * k,
                up[2] + (w[2] - v[2]) * k,
                up[3] + 1.0 + (w[3] - v[3]) * k,
            ];
        }
        m.call(0x0011_b830, &[sp + 0x40], &[v, up]);
    }
    for k in 0..3 {
        let q = m.rv(sp + 0x40 + 0x10 * k);
        m.wv(sp + 0x10 + 0x10 * k, q);
    }
    let nd = m.r32(o + 8);
    let a = m.r32(nd + 0xa0);
    m.call(0x0022_fd38, &[sp + 0x40, 0, a, sp + 0x10], &[]);
    for k in 0..3 {
        let q = m.rv(sp + 0x40 + 0x10 * k);
        m.wv(nd + 0xc0 + 0x10 * k, q);
    }
    let nd = m.r32(o + 8);
    m.w16(nd + 0x9a, 5);
    stack_pop(m, 0x90);
}

// ---------------------------------------------------------------- run_001cf570 (fx helpers)
/// FUN_001cf578: node from pool 0x3e1144 {+0 `x`, +8 old list head +0xe8} pushed onto this +0xe8
pub fn ee_sub_1cf578<M: QMem + ?Sized>(m: &mut M, o: u32, x: u32) {
    let n = m.call(0x0011_ccc0, &[0x003e_1144], &[]).0;
    m.w32(n, x);
    let h = m.r32(o + 0xe8);
    m.w32(n + 8, h);
    m.w32(o + 0xe8, n);
}
/// FUN_001cfd30: snapshot the source (+0x104): +0x70 = position, +0x40..+0x6f = orientation;
/// FUN_002309e8((+0x1c)+0x60, 0, out, 0, t = +0x18); +0x90 = (out pos - +0x70) / t + +0xa0 * (-t / 2);
/// FUN_0011bc00(rel, out, +0x40) -> FUN_0011bcf8(rel, &+0x100, +0xf0); +0x100 pushed one turn (2 pi) further
/// in its own sign; source vtable(+0xc) +0x4c; +0x104 = 0
pub fn ee_sub_1cfd30<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let sp = stack_push(m, 0xb0);
    let x = m.r32(o + 0x104);
    let p = m.call(0x001a_62e8, &[x], &[]).1;
    m.wv(o + 0x70, p);
    let x = m.r32(o + 0x104);
    let r = m.call(0x001a_6318, &[x], &[]).0;
    for k in 0..3 {
        let q = m.rv(r + 0x10 * k);
        m.wv(o + 0x40 + 0x10 * k, q);
    }
    let c = m.r32(o + 0x1c);
    let a = m.r32(c + 0x60);
    let t = m.rf(o + 0x18);
    m.call(0x0023_09e8, &[a, 0, sp, 0], &[s(t)]);
    let t = m.rf(o + 0x18);
    let (k1, k2) = (crate::engine::math::ps2f::ps2_div(1.0, t), crate::engine::math::ps2f::ps2_mul(t, -0.5));
    let (q, b, v) = (m.rv(sp + 0x30), m.rv(o + 0x70), m.rv(o + 0xa0));
    // VU0: vsub / vmulabc / vmaddbc, each result saturated (no inf / NaN on the VU)
    use crate::engine::math::vu0::fix;
    let r: V4 = std::array::from_fn(|i| fix(fix(fix(q[i] - b[i]) * k1) + fix(v[i] * k2)));
    m.wv(o + 0x90, r);
    m.call(0x0011_bc00, &[sp + 0x80, sp, o + 0x40], &[]);
    for k in 0..3 {
        let q = m.rv(sp + 0x80 + 0x10 * k);
        m.wv(sp + 0x40 + 0x10 * k, q);
    }
    m.call(0x0011_bcf8, &[sp + 0x40, o + 0x100, o + 0xf0], &[]);
    let f = m.rf(o + 0x100);
    m.wf(o + 0x100, if 0.0 < f { f + 6.283_185_5 } else { f - 6.283_185_5 });
    let x = m.r32(o + 0x104);
    vcall(m, x, 0xc, 0x4c, &[], &[]);
    m.w32(o + 0x104, 0);
    stack_pop(m, 0xb0);
}

// ---------------------------------------------------------------- class C2d80d0 (shop / pause-object switch, base C2d5898)
/// game-state block: DAT_002dfbc0 bit 1 -> 0x2dfc00, else 0x2e0d00
fn gstate<M: QMem + ?Sized>(m: &mut M) -> u32 {
    if m.r32(0x002d_fbc0) & 2 == 0 {
        0x002e_0d00
    } else {
        0x002d_fc00
    }
}
/// FUN_001d7238 (C2d80d0_v6): base C2d5898_v6 (FUN_001c3d70), DAT_003d4a54 = this
pub fn ee_sub_1d7238<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001c_3d70, &[o], &[]);
    m.w32(0x003d_4a54, o);
}
/// FUN_001d72a8 (C2d80d0_v15, update): base C2caf88_v15; state 1 with a list (+0x188) and no Revoke
/// (0x2de2e0): x = FUN_00187e88(game state); x != 0 -> FUN_001d77d0(this, x); else the first of +0x18c
/// 12-byte entries at +0x190 {key == FUN_001f07e8(+0x188), FUN_00187d80(state, +4) != 0, target != -1}:
/// 0x378 -> ThiefNet shop FUN_001dc260(0x3da150, 0); else object FUN_0015e938(world, 4, target, this) ->
/// FUN_001dfd58(obj, 0)
pub fn ee_sub_1d72a8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001f_36e0, &[o], &[]);
    if m.r32(o + 0x180) != 1 || m.r32(o + 0x188) == 0 || m.call(0x0014_fd50, &[0x002d_e2e0], &[]).0 != 0 {
        return;
    }
    let g = gstate(m);
    let x = m.call(0x0018_7e88, &[g], &[]).0;
    if x != 0 {
        m.call(0x001d_77d0, &[o, x], &[]);
        return;
    }
    let mut p = o + 0x190;
    let mut i = 0;
    while i < m.r32(o + 0x18c) as i32 {
        let l = m.r32(o + 0x188);
        let k = m.call(0x001f_07e8, &[l], &[]).0;
        if m.r32(p) == k {
            let a = m.r32(p + 4);
            if m.call(0x0018_7d80, &[g, a], &[]).0 != 0 {
                let t = m.r32(p + 8);
                if t != u32::MAX {
                    if t == 0x378 {
                        m.call(0x001d_c260, &[0x003d_a150, 0], &[]);
                        return;
                    }
                    let ob = m.call(0x0015_e938, &[WORLD, 4, t, o], &[]).0;
                    if ob != 0 {
                        m.call(0x001d_fd58, &[ob, 0], &[]);
                        return;
                    }
                }
            }
        }
        p += 0xc;
        i += 1;
    }
}
/// FUN_001d7418 (C2d80d0_v78): base C2d5898_v78 (FUN_001c3ed8); message 0 / 1 -> FUN_001d7480(this, msg)
pub fn ee_sub_1d7418<M: QMem + ?Sized>(m: &mut M, o: u32, msg: u32) {
    m.call(0x001c_3ed8, &[o, msg], &[]);
    if msg < 2 {
        ee_sub_1d7480(m, o, msg);
    }
}
/// FUN_001d7480: state change (+0x180) to `k`: leaving 1 -> FUN_001f1448(0, this, -1, 0, +0x188), vslot
/// +0x1c(6, 0, 0, 0, 0). Entering 1: DAT_003e0f9c set with +0x74 == 0 -> FUN_00162e38; class-0x34 objects
/// (max 3) -> FUN_001433c8 + FUN_001440d0(x, 1); objects 100 (max 32) -> vslot +0x264(4); objects 0x4f
/// (max 32) -> vslot +0x4c; list (+0x184): entry = FUN_001f0400(list, id of FUN_001d8a38(ref, 0x2a7878, 0))
/// or, when -1, FUN_001f04f8(list, 0); none -> -1; FUN_001f1448(list, this, entry, 0, +0x188),
/// FUN_001d77d0(this, 0), FUN_00187e78(game state), vslot +0x1c(5, 0, 0, 0, 0)
pub fn ee_sub_1d7480<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32) {
    if m.r32(o + 0x180) == k {
        return;
    }
    if m.r32(o + 0x180) == 1 {
        m.call(0x001f_1448, &[0, o, u32::MAX, 0, o + 0x188], &[]);
        vcall(m, o, 0xc, 0x1c, &[6, 0, 0, 0, 0], &[]);
    }
    m.w32(o + 0x180, k);
    if k != 1 {
        return;
    }
    let sp = stack_push(m, 0x120);
    let d = m.r32(0x003e_0f9c);
    if d != 0 && m.r32(d + 0x74) == 0 {
        m.call(0x0016_2e38, &[], &[]);
    }
    let n = m.call(0x0015_e9c8, &[WORLD, 5, 0x34, 0, 3, sp], &[]).0 as i32;
    for i in 0..n.max(0) as u32 {
        let x = m.r32(sp + 4 * i);
        m.call(0x0014_33c8, &[x], &[]);
        m.call(0x0014_40d0, &[x, 1], &[]);
    }
    let n = m.call(0x0015_e9c8, &[WORLD, 0x105, 100, 0, 0x20, sp + 0x10], &[]).0 as i32;
    for i in 0..n.max(0) as u32 {
        let x = m.r32(sp + 0x10 + 4 * i);
        vcall(m, x, 0xc, 0x264, &[4], &[]);
    }
    let n = m.call(0x0015_e9c8, &[WORLD, 0x105, 0x4f, 0, 0x20, sp + 0x90], &[]).0 as i32;
    for i in 0..n.max(0) as u32 {
        let x = m.r32(sp + 0x90 + 4 * i);
        vcall(m, x, 0xc, 0x4c, &[], &[]);
    }
    let mut e = u32::MAX;
    if m.r32(o + 0x184) != 0 {
        let r = sp + 0x110;
        m.call(0x001d_8a38, &[r, 0x002a_7878, 0], &[]);
        let id = m.r32(r + 4);
        m.call(0x0012_9ec8, &[r], &[]);
        let l = m.r32(o + 0x184);
        e = if m.call(0x001f_0400, &[l, id], &[]).0 == u32::MAX {
            let l = m.r32(o + 0x184);
            m.call(0x001f_04f8, &[l, 0], &[]).0
        } else {
            id
        };
    }
    let l = m.r32(o + 0x184);
    m.call(0x001f_1448, &[l, o, e, 0, o + 0x188], &[]);
    m.call(0x001d_77d0, &[o, 0], &[]);
    let g = gstate(m);
    m.call(0x0018_7e78, &[g], &[]);
    vcall(m, o, 0xc, 0x1c, &[5, 0, 0, 0, 0], &[]);
    stack_pop(m, 0x120);
}

// ---------------------------------------------------------------- class C2d4820 (base C2c8940)
/// FUN_001ba0b8 (C2d4820_v25): base C2c8940_v25 (FUN_00206518); attached object (+0x1050) -> vslot +0x4c
pub fn ee_sub_1ba0b8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0020_6518, &[o], &[]);
    let x = m.r32(o + 0x1050);
    if x != 0 {
        vcall(m, x, 0xc, 0x4c, &[], &[]);
    }
}

// ---------------------------------------------------------------- class C2c9c20 (trigger volume, base C2c94e0)
/// FUN_001492f0 (C2c9c20_v6): base C2c9a80_v6 (FUN_00231100); FUN_00148718(+0x130), +0x1cc = 1,
/// FUN_001a6ca8(this, 0); flags word (+0x40: tagged pointer, bit 0 clear -> through the pointer) |= 2
pub fn ee_sub_1492f0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0023_1100, &[o], &[]);
    m.call(0x0014_8718, &[o + 0x130], &[]);
    m.w32(o + 0x1cc, 1);
    m.call(0x001a_6ca8, &[o, 0], &[]);
    let p = m.r32(o + 0x40);
    if (p ^ 1) & 1 == 0 {
        m.w32(o + 0x40, p | 2);
    } else {
        let v = m.r32(p);
        m.w32(p, v | 2);
    }
}
/// FUN_00149398 (C2c9c20_v14): base C2c94e0_v14 (FUN_001a5c48); FUN_00148740(+0x130); FUN_00149718(this, 0);
/// tracking list (+0x2c8) -> +0x2c4 = 0x100-byte block (FUN_001ae1d0 {0x100, 0x10})
pub fn ee_sub_149398<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001a_5c48, &[o], &[]);
    m.call(0x0014_8740, &[o + 0x130], &[]);
    ee_sub_149718(m, o, 0);
    if m.r32(o + 0x2c8) != 0 {
        let sp = stack_push(m, 0x10);
        m.w32(sp, 0x100);
        m.w32(sp + 4, 0x10);
        let b = m.call(0x001a_e1d0, &[sp], &[]).0;
        m.w32(o + 0x2c4, b);
        stack_pop(m, 0x10);
    }
}
/// FUN_001493f8 (C2c9c20_v40): base C2c9a80_v40 (FUN_002311b0); +0x130 = this
pub fn ee_sub_1493f8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0023_11b0, &[o], &[]);
    m.w32(o + 0x130, o);
}
/// FUN_00149440 (C2c9c20_v15, update): base C2c94e0_v15 (FUN_001a5e28). State 2: delay +0x1c8 >= 0 elapsed
/// since +0x1c4 -> FUN_00149718(this, 1). States 0 / 1: candidates = explicit objects (+0x288 at +0x28c)
/// + objects of each class in +0x2ac (+0x2b0, FUN_0015e9c8(world, 0x505, cls, 0, 64 - n, ..)) or, with
/// neither, the player list DAT_003e1348 / 0x3e134c; +0x1d8 = first live (FUN_001a61a0) candidate inside
/// the volume (FUN_00227c90(this, pos)); with tracking (+0x2c8) the inside set is collected, the candidate
/// list (not the inside set, as in the original) qsorted with FUN_00149420, and a changed set (count +0x2c0 /
/// block +0x2c4) is stored and announced (vslot +0x1c(9, 0, 0, 0, 0)); without tracking the scan stops at
/// the first hit. FUN_00149718(this, any inside); scratch scope FUN_001ae478 / FUN_001ae4a0(.., 2)
pub fn ee_sub_149440<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001a_5e28, &[o], &[]);
    let st = m.r32(o + 0x1c0) as i32;
    if st == 2 {
        let d = m.rf(o + 0x1c8);
        if 0.0 <= d && d < m.rf(CLOCK) - m.rf(o + 0x1c4) {
            ee_sub_149718(m, o, 1);
        }
        return;
    }
    if !(0..3).contains(&st) {
        return;
    }
    let sp = stack_push(m, 0x210);
    let (l0, l1, sc) = (sp, sp + 0x100, sp + 0x200);
    m.call(0x001a_e478, &[sc], &[]);
    let n0 = m.r32(o + 0x288);
    m.call(0x001a_e5a0, &[l0, o + 0x28c, n0 << 2], &[]);
    let mut n = m.r32(o + 0x288) as i32;
    let mut j = 0;
    while j < m.r32(o + 0x2ac) as i32 {
        let c = m.r32(o + 0x2b0 + 4 * j as u32);
        n += m.call(0x0015_e9c8, &[WORLD, 0x505, c, 0, (0x40 - n) as u32, l0 + 4 * n as u32], &[]).0 as i32;
        j += 1;
    }
    if m.r32(o + 0x288) == 0 && m.r32(o + 0x2ac) == 0 {
        let k = m.r32(0x003e_1348);
        m.call(0x001a_e5a0, &[l0 + 4 * n as u32, 0x003e_134c, k << 2], &[]);
        n += m.r32(0x003e_1348) as i32;
    }
    m.w32(o + 0x1d8, 0);
    let mut hit = 0u32;
    let mut cnt = 0u32;
    let mut stop = false;
    for i in 0..n.max(0) as u32 {
        let x = m.r32(l0 + 4 * i);
        if x == 0 || m.call(0x001a_61a0, &[x], &[]).0 == 0 {
            continue;
        }
        let p = m.call(0x001a_62e8, &[x], &[]).1;
        if m.call(0x0022_7c90, &[o], &[p]).0 & 0xff == 0 {
            continue;
        }
        hit = 1;
        if m.r32(o + 0x1d8) == 0 {
            m.w32(o + 0x1d8, x);
        }
        cnt += 1;
        if m.r32(o + 0x2c8) == 0 {
            stop = true;
            break;
        }
        m.w32(l1 + 4 * (cnt - 1), x);
    }
    if !stop && m.r32(o + 0x2c8) != 0 {
        m.call(0x0023_38c0, &[l0, n as u32, 4, 0x0014_9420], &[]);
        let blk = m.r32(o + 0x2c4);
        let changed = m.r32(o + 0x2c0) != cnt || (0..cnt).any(|k| m.r32(l1 + 4 * k) != m.r32(blk + 4 * k));
        if changed {
            m.w32(o + 0x2c0, cnt);
            m.call(0x001a_e5a0, &[blk, l1, cnt << 2], &[]);
            vcall(m, o, 0xc, 0x1c, &[9, 0, 0, 0, 0], &[]);
        }
    }
    ee_sub_149718(m, o, hit);
    m.call(0x001a_e4a0, &[sc, 2], &[]);
    stack_pop(m, 0x210);
}
/// FUN_00149718: FUN_00148a78(this + 0x130, flag)
pub fn ee_sub_149718<M: QMem + ?Sized>(m: &mut M, o: u32, f: u32) {
    m.call(0x0014_8a78, &[o + 0x130, f], &[]);
}
/// FUN_00149880 (C2c9c20_v11): base C2c94e0_v11 (FUN_001a5848); state 1 / 2 -> FUN_00149718(this, 0) and
/// with tracking (+0x2c8): count +0x2c0 = 0, vslot +0x1c(9, 0, 0, 0, 0)
pub fn ee_sub_149880<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001a_5848, &[o], &[]);
    if m.r32(o + 0x1c0).wrapping_sub(1) < 2 {
        ee_sub_149718(m, o, 0);
        if m.r32(o + 0x2c8) != 0 {
            m.w32(o + 0x2c0, 0);
            vcall(m, o, 0xc, 0x1c, &[9, 0, 0, 0, 0], &[]);
        }
    }
}

// ---------------------------------------------------------------- per-file static pool init copies (same body as FUN_001ba9f8)
/// FUN_0011cb50: pool refcount bump (same body as FUN_001ba9f8)
pub fn ee_sub_11cb50<M: QMem + ?Sized>(m: &mut M, on: u32, k: u32) {
    ee_sub_1ba9f8(m, on, k);
}
/// FUN_0011cc88: FUN_0011cb50(1, 0xffff)
pub fn ee_sub_11cc88<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_1ba9f8(m, 1, 0xffff);
}
/// FUN_0011ce08: pool refcount bump (same body as FUN_001ba9f8)
pub fn ee_sub_11ce08<M: QMem + ?Sized>(m: &mut M, on: u32, k: u32) {
    ee_sub_1ba9f8(m, on, k);
}
/// FUN_0011cf40: FUN_0011ce08(1, 0xffff)
pub fn ee_sub_11cf40<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_1ba9f8(m, 1, 0xffff);
}
/// FUN_00130750: pool refcount bump (same body as FUN_001ba9f8)
pub fn ee_sub_130750<M: QMem + ?Sized>(m: &mut M, on: u32, k: u32) {
    ee_sub_1ba9f8(m, on, k);
}
/// FUN_00130888: FUN_00130750(1, 0xffff)
pub fn ee_sub_130888<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_1ba9f8(m, 1, 0xffff);
}
/// FUN_00154908: pool refcount bump (same body as FUN_001ba9f8)
pub fn ee_sub_154908<M: QMem + ?Sized>(m: &mut M, on: u32, k: u32) {
    ee_sub_1ba9f8(m, on, k);
}
/// FUN_00154a40: FUN_00154908(1, 0xffff)
pub fn ee_sub_154a40<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_1ba9f8(m, 1, 0xffff);
}
/// FUN_00155298: pool refcount bump (same body as FUN_001ba9f8)
pub fn ee_sub_155298<M: QMem + ?Sized>(m: &mut M, on: u32, k: u32) {
    ee_sub_1ba9f8(m, on, k);
}
/// FUN_001553d0: FUN_00155298(1, 0xffff)
pub fn ee_sub_1553d0<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_1ba9f8(m, 1, 0xffff);
}
/// FUN_001a73c0: pool refcount bump (same body as FUN_001ba9f8)
pub fn ee_sub_1a73c0<M: QMem + ?Sized>(m: &mut M, on: u32, k: u32) {
    ee_sub_1ba9f8(m, on, k);
}
/// FUN_001a74f8: FUN_001a73c0(1, 0xffff)
pub fn ee_sub_1a74f8<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_1ba9f8(m, 1, 0xffff);
}
/// FUN_001a7ac0: pool refcount bump (same body as FUN_001ba9f8)
pub fn ee_sub_1a7ac0<M: QMem + ?Sized>(m: &mut M, on: u32, k: u32) {
    ee_sub_1ba9f8(m, on, k);
}
/// FUN_001a7bf8: FUN_001a7ac0(1, 0xffff)
pub fn ee_sub_1a7bf8<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_1ba9f8(m, 1, 0xffff);
}
/// FUN_001bbef8: pool refcount bump (same body as FUN_001ba9f8)
pub fn ee_sub_1bbef8<M: QMem + ?Sized>(m: &mut M, on: u32, k: u32) {
    ee_sub_1ba9f8(m, on, k);
}
/// FUN_001bc030: FUN_001bbef8(1, 0xffff)
pub fn ee_sub_1bc030<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_1ba9f8(m, 1, 0xffff);
}
/// FUN_001c3c18: pool refcount bump (same body as FUN_001ba9f8)
pub fn ee_sub_1c3c18<M: QMem + ?Sized>(m: &mut M, on: u32, k: u32) {
    ee_sub_1ba9f8(m, on, k);
}
/// FUN_001c3d50: FUN_001c3c18(1, 0xffff)
pub fn ee_sub_1c3d50<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_1ba9f8(m, 1, 0xffff);
}
/// FUN_001c4cf8: pool refcount bump (same body as FUN_001ba9f8)
pub fn ee_sub_1c4cf8<M: QMem + ?Sized>(m: &mut M, on: u32, k: u32) {
    ee_sub_1ba9f8(m, on, k);
}
/// FUN_001c4e30: FUN_001c4cf8(1, 0xffff)
pub fn ee_sub_1c4e30<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_1ba9f8(m, 1, 0xffff);
}

// ---------------------------------------------------------------- run_001c3958 (track helpers, world focus objects)
/// FUN_001c3958: project `x`'s position (FUN_001a62e8) onto track `o` (FUN_001e6f60 -> param *out); with no
/// loop flag (+0xe4 == 0): *out = *out < 100 ? 100 : min(*out, FUN_001e6648(o) - 35)
pub fn ee_sub_1c3958<M: QMem + ?Sized>(m: &mut M, o: u32, x: u32, out: u32) {
    let p = m.call(0x001a_62e8, &[x], &[]).1;
    m.call(0x001e_6f60, &[o, 0, 0, 0, 0, 0, out], &[p]);
    if m.r32(o + 0xe4) == 0 {
        let hi = callf(m, 0x001e_6648, &[o], &[]) - 35.0;
        let v = m.rf(out);
        m.wf(out, if v < 100.0 { 100.0 } else if hi < v { hi } else { v });
    }
}
/// FUN_001c3a08: frame F = FUN_001c3500(o, 1) and position P = FUN_001e6f10(o, .., 0) at `t`; out (when set)
/// = P + (0,0,0,1) + F row 0 * DAT_002c0d04
pub fn ee_sub_1c3a08<M: QMem + ?Sized>(m: &mut M, o: u32, out: u32, t: f32) {
    let sp = stack_push(m, 0x40);
    m.call(0x001c_3500, &[o, 1, sp], &[s(t)]);
    m.call(0x001e_6f10, &[o, sp + 0x30, 0], &[s(t)]);
    if out != 0 {
        let (p, f) = (m.rv(sp + 0x30), m.rv(sp));
        let k = m.rf(0x002c_0d04);
        m.wv(out, [p[0] + f[0] * k, p[1] + f[1] * k, p[2] + f[2] * k, p[3] + 1.0 + f[3] * k]);
    }
    stack_pop(m, 0x40);
}
/// FUN_001c3a88: random spawn point near the track: t = FUN_001e8008(o, t, DAT_002c0d10); frame / position
/// at t; r = rand(-1, 1); d = frame rows * FUN_0011e1a0(r * |r| * DAT_002c0d08, -1, 0) (xyz; w kept);
/// *pos = P + (0,0,0,1) + d * |r| * rand(0, DAT_002c0d0c); *dir = d
pub fn ee_sub_1c3a88<M: QMem + ?Sized>(m: &mut M, o: u32, pos: u32, dir: u32, t: f32) {
    let sp = stack_push(m, 0x50);
    let k = m.rf(0x002c_0d10);
    let t = callf(m, 0x001e_8008, &[o], &[s(t), s(k)]);
    m.call(0x001c_3500, &[o, 1, sp], &[s(t)]);
    m.call(0x001e_6f10, &[o, sp + 0x40, 0], &[s(t)]);
    let r = callf(m, 0x0011_d448, &[], &[s(-1.0), s(1.0)]);
    let a = r.abs();
    let k = m.rf(0x002c_0d08);
    let v = m.call(0x0011_e1a0, &[], &[s(r * a * k), s(-1.0), s(0.0)]).1;
    let rows = [m.rv(sp), m.rv(sp + 0x10), m.rv(sp + 0x20)];
    let d = mul3(&rows, v, v[3]);
    let p = m.rv(sp + 0x40);
    let k = m.rf(0x002c_0d0c);
    let f = a * callf(m, 0x0011_d448, &[], &[s(0.0), s(k)]);
    m.wv(pos, [p[0] + d[0] * f, p[1] + d[1] * f, p[2] + d[2] * f, p[3] + 1.0 + d[3] * f]);
    m.wv(dir, d);
    stack_pop(m, 0x50);
}
/// FUN_001c3bb0: looping track (+0xe4): out = {FUN_001e6620(o), FUN_001e6648(o)}; returns the flag
pub fn ee_sub_1c3bb0<M: QMem + ?Sized>(m: &mut M, o: u32, out: u32) -> u32 {
    let b = m.r32(o + 0xe4) != 0;
    if b {
        let a = m.call(0x001e_6620, &[o], &[]).1[0].to_bits();
        m.w32(out, a);
        let a = m.call(0x001e_6648, &[o], &[]).0;
        m.w32(out + 4, a);
    }
    b as u32
}
/// FUN_001c3e20 (C2d5898_v4): base C2c94a0_v4 (FUN_00133590); state 5 with a sound (+0x174) -> stop it
/// (FUN_001fb098(snd, 0.0)) and release the reference (FUN_00258040(+0x174, 0))
pub fn ee_sub_1c3e20<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32) {
    m.call(0x0013_3590, &[o, k], &[]);
    if k == 5 {
        let e = m.r32(o + 0x174);
        if e != 0 {
            m.call(0x001f_b098, &[e], &[s(0.0)]);
            m.call(0x0025_8040, &[o + 0x174, 0], &[]);
        }
    }
}
/// FUN_001c4080: (+0x150) ? (+0x150)->+0x80 : -1
pub fn ee_sub_1c4080<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let p = m.r32(o + 0x150);
    if p != 0 {
        m.r32(p + 0x80)
    } else {
        u32::MAX
    }
}
/// FUN_001c4098: FUN_001c4080(o) != -1
pub fn ee_sub_1c4098<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    (ee_sub_1c4080(m, o) != u32::MAX) as u32
}
/// FUN_001c40c0: x == FUN_001c4840(0x3dd490)
pub fn ee_sub_1c40c0<M: QMem + ?Sized>(m: &mut M, x: u32) -> u32 {
    (m.call(0x001c_4840, &[0x003d_d490], &[]).0 == x) as u32
}
/// FUN_001c4130: attach object `x` at +0x16c: old one -> FUN_00221268(old, 0); new: +0x128 = -1,
/// FUN_001a58d0(x, 0, 0), and when `o` is the world focus (FUN_001c4a48) FUN_00221268(x, 1)
pub fn ee_sub_1c4130<M: QMem + ?Sized>(m: &mut M, o: u32, x: u32) {
    let old = m.r32(o + 0x16c);
    if x == old {
        return;
    }
    if old != 0 {
        m.call(0x0022_1268, &[old, 0], &[]);
    }
    m.w32(o + 0x16c, x);
    if x != 0 {
        m.w32(x + 0x128, u32::MAX);
        m.call(0x001a_58d0, &[x, 0, 0], &[]);
        if ee_sub_1c4a48(m, 0x003d_d490) == o {
            let y = m.r32(o + 0x16c);
            m.call(0x0022_1268, &[y, 1], &[]);
        }
    }
}
/// FUN_001c4280 (C2ca240 ctor): {0, 0, _, vtable 0x2ca240}, FUN_001c42e8(this), +0x7c = k, +0x80 = -1,
/// list add FUN_0011a070(0x3e133c, this)
pub fn ee_sub_1c4280<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32) -> u32 {
    m.w32(o, 0);
    m.w32(o + 4, 0);
    m.w32(o + 0xc, 0x002c_a240);
    m.call(0x001c_42e8, &[o, k], &[]);
    m.w32(o + 0x7c, k);
    m.w32(o + 0x80, u32::MAX);
    m.call(0x0011_a070, &[0x003e_133c, o], &[]);
    o
}
/// FUN_001c4640: +0x80 = k; k == 1 -> +0x78 = 0xffff
pub fn ee_sub_1c4640<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32) {
    m.w32(o + 0x80, k);
    if k == 1 {
        m.w32(o + 0x78, 0xffff);
    }
}
/// FUN_001c4728: +0x30 & mask != 0
pub fn ee_sub_1c4728<M: QMem + ?Sized>(m: &mut M, o: u32, mask: u32) -> u32 {
    (m.r32(o + 0x30) & mask != 0) as u32
}
/// FUN_001c4848: up to 4 objects in the list +0x477c (count +0x4778); its variant record (FUN_00258658(x, 1))
/// gets a {0x14, 0x10} pool block at +8 when empty (FUN_001ae2a0)
pub fn ee_sub_1c4848<M: QMem + ?Sized>(m: &mut M, w: u32, x: u32) {
    let n = m.r32(w + 0x4778);
    if n < 4 {
        m.w32(w + 0x477c + n * 4, x);
        m.w32(w + 0x4778, n + 1);
        let r = m.call(0x0025_8658, &[x, 1], &[]).0;
        if m.r32(r + 8) == 0 {
            let sp = stack_push(m, 0x10);
            m.w32(sp, 0x14);
            m.w32(sp + 4, 0x10);
            let b = m.call(0x001a_e2a0, &[sp], &[]).0;
            m.w32(r + 8, b);
            stack_pop(m, 0x10);
        }
    }
}
/// FUN_001c48c0: every listed object (+0x477c): its +0xe4 record's +8 block word 0 = focus +0x3efc and
/// FUN_001a58d0(x, focus, 0)
pub fn ee_sub_1c48c0<M: QMem + ?Sized>(m: &mut M, w: u32) {
    let mut i = 0u32;
    while (i as i32) < m.r32(w + 0x4778) as i32 {
        let x = m.r32(w + 0x477c + 4 * i);
        i += 1;
        let f = m.r32(w + 0x3efc);
        let r = m.r32(x + 0xe4);
        let b = m.r32(r + 8);
        m.w32(b, f);
        let f = m.r32(w + 0x3efc);
        m.call(0x001a_58d0, &[x, f, 0], &[]);
    }
}
/// focus change seen by FUN_001c4a48 from `old` to `new`: vslot +0x274 (0 to the old, then the store, then 1
/// to the new); unchanged -> store only
fn focus_swap<M: QMem + ?Sized>(m: &mut M, old: u32, new: u32, at: u32, x: u32) {
    if old == new {
        m.w32(at, x);
        return;
    }
    if old != 0 {
        vcall(m, old, 0xc, 0x274, &[0], &[]);
    }
    m.w32(at, x);
    if new != 0 {
        vcall(m, new, 0xc, 0x274, &[1], &[]);
    }
}
/// FUN_001c4940: set the base focus +0x3efc = x (notifying when the effective focus changes); x of class 0x34
/// -> event FUN_001d8b20({0x15, x+0x18}, 0x2a76f8) with both temporaries released; FUN_00194f70(w, 0, 0, 0);
/// FUN_001c48c0(w)
pub fn ee_sub_1c4940<M: QMem + ?Sized>(m: &mut M, w: u32, x: u32) {
    let old = ee_sub_1c4a48(m, w);
    let ov = m.r32(w + 0x3f00);
    let new = if ov == 0 { x } else { ov };
    focus_swap(m, old, new, w + 0x3efc, x);
    if x != 0 && m.call(0x0013_2b10, &[x, 0x34], &[]).0 != 0 {
        let sp = stack_push(m, 0x20);
        let id = m.r32(x + 0x18);
        m.w32(sp + 0x10, 0x15);
        m.w32(sp + 0x14, id);
        m.call(0x001d_8b20, &[sp, sp + 0x10, 0x002a_76f8, 0], &[]);
        m.call(0x0012_9ec8, &[sp], &[]);
        m.call(0x0012_9ec8, &[sp + 0x10], &[]);
        stack_pop(m, 0x20);
    }
    m.call(0x0019_4f70, &[w, 0, 0, 0], &[]);
    ee_sub_1c48c0(m, w);
}
/// FUN_001c4a48: effective focus = override +0x3f00, else base +0x3efc
pub fn ee_sub_1c4a48<M: QMem + ?Sized>(m: &mut M, w: u32) -> u32 {
    let v = m.r32(w + 0x3f00);
    if v != 0 {
        v
    } else {
        m.r32(w + 0x3efc)
    }
}
/// FUN_001c4a60: set the focus override +0x3f00 = x (0 = back to the base), notifying on change
pub fn ee_sub_1c4a60<M: QMem + ?Sized>(m: &mut M, w: u32, x: u32) {
    let old = ee_sub_1c4a48(m, w);
    let new = if x != 0 { x } else { m.r32(w + 0x3efc) };
    focus_swap(m, old, new, w + 0x3f00, x);
}

// ---------------------------------------------------------------- run_001a6af8 (object tree / transforms)
/// FUN_001a6af8: `a` is `b` or has it among its +0x10 ancestors (a != 0)
pub fn ee_sub_1a6af8<M: QMem + ?Sized>(m: &mut M, a: u32, b: u32) -> u32 {
    let mut a = a;
    while a != 0 && a != b {
        a = m.r32(a + 0x10);
    }
    (a != 0) as u32
}
/// FUN_001a6be8: vslot +0xec(o, quad p+0x30) then vslot +0xf4(o, p) (position + matrix from a transform)
pub fn ee_sub_1a6be8<M: QMem + ?Sized>(m: &mut M, o: u32, p: u32) {
    let v = m.rv(p + 0x30);
    vcall(m, o, 0xc, 0xec, &[], &[v]);
    vcall(m, o, 0xc, 0xf4, &[p], &[]);
}
/// FUN_001a6d28: bit 60 of the class word u64 (*(+0x14))+0x28 -> FUN_00147e88(0x3dd490); vslot +0x174(o, k);
/// every +0x10 ancestor's byte +0x2d |= k
pub fn ee_sub_1a6d28<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32) {
    let c = m.r32(o + 0x14);
    let c = m.r32(c);
    if (m.r64(c + 0x28) >> 60) & 1 != 0 {
        m.call(0x0014_7e88, &[0x003d_d490], &[]);
    }
    vcall(m, o, 0xc, 0x174, &[k], &[]);
    let mut p = m.r32(o + 0x10);
    while p != 0 {
        let b = m.r8(p + 0x2d);
        m.w8(p + 0x2d, b | k as u8);
        p = m.r32(p + 0x10);
    }
}
/// FUN_001a71b0: out = inverse(world(b)) * (a ? world(a) * mt : mt) (world = vslot +0xfc matrix;
/// FUN_0011c318 inverse, FUN_0011c168 multiply); returns out
pub fn ee_sub_1a71b0<M: QMem + ?Sized>(m: &mut M, out: u32, a: u32, b: u32, mt: u32) -> u32 {
    let sp = stack_push(m, 0xc0);
    let (acc, tmp, inv) = (sp, sp + 0x40, sp + 0x80);
    let copy = |m: &mut M, d: u32, s: u32| {
        for k in 0..4 {
            let q = m.rv(s + 0x10 * k);
            m.wv(d + 0x10 * k, q);
        }
    };
    if a == 0 {
        copy(m, acc, mt);
    } else {
        let w = vcall(m, a, 0xc, 0xfc, &[], &[]).0;
        m.call(0x0011_c168, &[tmp, w, mt], &[]);
        copy(m, acc, tmp);
    }
    if b != 0 {
        let w = vcall(m, b, 0xc, 0xfc, &[], &[]).0;
        m.call(0x0011_c318, &[inv, w], &[]);
        m.call(0x0011_c168, &[tmp, inv, acc], &[]);
        copy(m, acc, tmp);
    }
    copy(m, out, acc);
    stack_pop(m, 0xc0);
    out
}
/// rows 0..2 * v (+ row 3 when `aff`), xyz only (w of v kept)
fn xf<M: QMem + ?Sized>(m: &mut M, mt: u32, v: V4, aff: bool) -> V4 {
    let r = [m.rv(mt), m.rv(mt + 0x10), m.rv(mt + 0x20)];
    let q = m.rv(mt + 0x30);
    let t = if aff { [q[0], q[1], q[2]] } else { [0.0; 3] };
    lin3(r, v, t, v[3])
}
/// FUN_001a72d0: point `v` through matrix `a` (when set) then through inverse(b) (FUN_0011c318, when set)
pub fn ee_sub_1a72d0<M: QMem + ?Sized>(m: &mut M, a: u32, b: u32, v: V4) -> V4 {
    let mut v = v;
    if a != 0 {
        v = xf(m, a, v, true);
    }
    if b != 0 {
        let sp = stack_push(m, 0x40);
        m.call(0x0011_c318, &[sp, b], &[]);
        v = xf(m, sp, v, true);
        stack_pop(m, 0x40);
    }
    v
}
/// FUN_001a7348: direction `v` through rotation `a` (when set) then through FUN_0011b340(b) (when set)
pub fn ee_sub_1a7348<M: QMem + ?Sized>(m: &mut M, a: u32, b: u32, v: V4) -> V4 {
    let mut v = v;
    if a != 0 {
        v = xf(m, a, v, false);
    }
    if b != 0 {
        let sp = stack_push(m, 0x40);
        m.call(0x0011_b340, &[sp, b], &[]);
        v = xf(m, sp, v, false);
        stack_pop(m, 0x40);
    }
    v
}
/// VU1 microprogram upload: *ch |= 0x40, FlushCache(0), DMA send FUN_00241728(ch, prog), wait
/// FUN_002417f0(ch, 0, 0), *ch &= !0x40
fn vu_upload<M: QMem + ?Sized>(m: &mut M, chp: u32, prog: u32) {
    let ch = m.r32(chp);
    let v = m.r32(ch);
    m.w32(ch, v | 0x40);
    m.call(0x0023_4ca0, &[0], &[]);
    let ch = m.r32(chp);
    m.call(0x0024_1728, &[ch, prog], &[]);
    let ch = m.r32(chp);
    m.call(0x0024_17f0, &[ch, 0, 0], &[]);
    let ch = m.r32(chp);
    let v = m.r32(ch);
    m.w32(ch, v & !0x40);
}
/// FUN_001a7890: upload VU microprogram 0x259df0 through DAT_002de590
pub fn ee_sub_1a7890<M: QMem + ?Sized>(m: &mut M) {
    vu_upload(m, 0x002d_e590, 0x0025_9df0);
}
/// FUN_001a7900: upload VU microprogram 0x25e050 through DAT_002de594
pub fn ee_sub_1a7900<M: QMem + ?Sized>(m: &mut M) {
    vu_upload(m, 0x002d_e594, 0x0025_e050);
}
/// FUN_001a7e10: free the +0xac-linked chain from `p` into the world contact pool (FUN_0011cd98(w + 0x3d40, p))
pub fn ee_sub_1a7e10<M: QMem + ?Sized>(m: &mut M, w: u32, p: u32) {
    let mut p = p;
    while p != 0 {
        let n = m.r32(p + 0xac);
        m.call(0x0011_cd98, &[w + 0x3d40, p], &[]);
        p = n;
    }
}
/// FUN_001a7e60: per contact pair c = list[i] (i < n), sides j = 0 / 1 (0x40-byte records at c + j*0x40:
/// +0 shape, +4 body, +0x10 out point, +0x20 local point): with a body: (j 0 and 4 <= kind c+0x9c < 7)
/// -> point = body+0x70 + (0,0,0,1) + c+0x80 * -(shape+8); (j 1 and kind 7 / 9) -> same with +(shape+8);
/// else point = FUN_0022fc48(body, 0, local). Then c+0x90 = dot3(c+0x80, c+0x10 - c+0x50)
pub fn ee_sub_1a7e60<M: QMem + ?Sized>(m: &mut M, _a0: u32, n: u32, list: u32) {
    let mut i = 0u32;
    while (i as i32) < n as i32 {
        let c = m.r32(list + 4 * i);
        i += 1;
        for j in 0..2u32 {
            let e = c + j * 0x40;
            let b = m.r32(e + 4);
            if b == 0 {
                continue;
            }
            let kind = m.r32(c + 0x9c) as i32;
            let sh = m.r32(e);
            let f = if j == 0 {
                (4..7).contains(&kind).then(|| -m.rf(sh + 8))
            } else {
                (kind == 7 || kind == 9).then(|| m.rf(sh + 8))
            };
            let v = match f {
                Some(f) => {
                    let p = m.rv(b + 0x70);
                    let d = m.rv(c + 0x80);
                    [p[0] + d[0] * f, p[1] + d[1] * f, p[2] + d[2] * f, p[3] + 1.0 + d[3] * f]
                }
                None => {
                    let l = m.rv(e + 0x20);
                    m.call(0x0022_fc48, &[b, 0], &[l]).1
                }
            };
            m.wv(e + 0x10, v);
        }
        let (a, b, d) = (m.rv(c + 0x10), m.rv(c + 0x50), m.rv(c + 0x80));
        let v = d[0] * (a[0] - b[0]) + d[1] * (a[1] - b[1]) + d[2] * (a[2] - b[2]);
        m.wf(c + 0x90, v);
    }
}
/// clip edge (v0, v1) (u16 vertex indices into `verts`, 0x10 each) against BSP `bsp`
/// (ClsgClipEdgeToBsp(bsp, 0x7fffffff, p0, p1, max - n, out + n * 0x60)); new segments get +0x54 = edge
fn clip_edge<M: QMem + ?Sized>(m: &mut M, bsp: u32, verts: u32, edge: u32, max: u32, out: u32, n: &mut u32) {
    let (i0, i1) = (m.r16(edge) as u32, m.r16(edge + 2) as u32);
    let p0 = m.rv(verts + i0 * 0x10);
    let p1 = m.rv(verts + i1 * 0x10);
    let k = m.call(0x0011_8668, &[bsp, 0x7fff_ffff, max.wrapping_sub(*n), out + *n * 0x60], &[p0, p1]).0;
    let nn = n.wrapping_add(k);
    let mut x = *n;
    while (x as i32) < nn as i32 {
        m.w32(out + x * 0x60 + 0x54, edge);
        x += 1;
    }
    *n = nn;
}
/// FUN_001a80d8: every edge of geometry `g` (+0xa4 edges of 0x10, count +0xa0, last first; vertices +0x94)
/// clipped against `bsp` into `out` (max entries); returns the segment count
pub fn ee_sub_1a80d8<M: QMem + ?Sized>(m: &mut M, _a0: u32, _a1: u32, g: u32, bsp: u32, max: u32, out: u32) -> u32 {
    let mut n = 0u32;
    let mut i = m.r32(g + 0xa0) as i32;
    let (verts, edges) = (m.r32(g + 0x94), m.r32(g + 0xa4));
    loop {
        i -= 1;
        if i < 0 {
            return n;
        }
        clip_edge(m, bsp, verts, edges + i as u32 * 0x10, max, out, &mut n);
    }
}
/// FUN_001a81f8: the `cnt` faces of `list` (0xc stride, face at +8) are marked (+0x18 |= 1); each marked,
/// not yet done (bit 2) face's edge chain (+0x10; edge {u16 v0, v1, _, other face}, next at +8) is clipped
/// against (o+0x74) when the other face (g+0x9c + idx * 0x20) is marked too; marks cleared (& ~3) at the
/// end; returns the segment count
#[allow(clippy::too_many_arguments)]
pub fn ee_sub_1a81f8<M: QMem + ?Sized>(m: &mut M, _a0: u32, o: u32, g: u32, cnt: u32, list: u32, max: u32, out: u32) -> u32 {
    let (faces, verts) = (m.r32(g + 0x9c), m.r32(g + 0x94));
    let mut n = 0u32;
    let face = |m: &mut M, i: i32| m.r32(list.wrapping_add((i * 0xc) as u32) + 8);
    for i in (0..cnt as i32).rev() {
        let f = face(m, i);
        let v = m.r32(f + 0x18);
        m.w32(f + 0x18, v | 1);
    }
    for i in (0..cnt as i32).rev() {
        let f = face(m, i);
        let fl = m.r32(f + 0x18);
        if fl & 1 == 0 || fl & 2 != 0 {
            continue;
        }
        m.w32(f + 0x18, fl | 2);
        let mut e = m.r32(f + 0x10);
        while e != 0 {
            let other = faces + m.r16(e + 6) as u32 * 0x20;
            if m.r32(other + 0x18) & 1 != 0 {
                let bsp = m.r32(o + 0x74);
                clip_edge(m, bsp, verts, e, max, out, &mut n);
            }
            e = m.r32(e + 8);
        }
    }
    for i in (0..cnt as i32).rev() {
        let f = face(m, i);
        let v = m.r32(f + 0x18);
        m.w32(f + 0x18, v & !3);
    }
    n
}
/// FUN_001a86b8: breadth-first search over faces (`faces` + idx * 0x20, visited bit 4 in +0x18) from face
/// `from` to face `to` across the segment list `segs` (linked by +0x5c; edge +0x54 with face indices at +4
/// / +6); queue of {parent entry, segment, face} in a 0x300-byte local. Found: out[] = the segment reaching
/// `to` then the segments back up the parent chain, returns their count; else 0. Visited bits cleared
pub fn ee_sub_1a86b8<M: QMem + ?Sized>(m: &mut M, faces: u32, from: u32, to: u32, segs: u32, out: u32) -> u32 {
    let q = stack_push(m, 0x300);
    m.w32(q, 0);
    m.w32(q + 4, 0);
    m.w32(q + 8, from);
    let mut cnt = 1u32;
    let mut i = 0u32;
    let mut found = 0u32;
    'outer: while segs != 0 {
        let ent = q + i * 0xc;
        let mut sg = segs;
        while sg != 0 {
            let cur = m.r32(ent + 8);
            let ed = m.r32(sg + 0x54);
            let (f0, f1) = (m.r16(ed + 4) as u32, m.r16(ed + 6) as u32);
            let other = if f0 == cur {
                f1
            } else if f1 == cur {
                f0
            } else {
                sg = m.r32(sg + 0x5c);
                continue;
            };
            if other == to {
                m.w32(out, sg);
                found = 1;
                let mut o = out;
                let mut e = ent;
                let mut s1 = m.r32(e + 4);
                while s1 != 0 {
                    o += 4;
                    found += 1;
                    m.w32(o, s1);
                    e = m.r32(e);
                    s1 = m.r32(e + 4);
                }
                break 'outer;
            }
            let fr = faces + other * 0x20;
            let fl = m.r32(fr + 0x18);
            if fl & 4 == 0 {
                m.w32(fr + 0x18, fl | 4);
                let ne = q + cnt * 0xc;
                m.w32(ne + 8, other);
                m.w32(ne, ent);
                m.w32(ne + 4, sg);
                cnt += 1;
            }
            sg = m.r32(sg + 0x5c);
        }
        i += 1;
        if cnt <= i {
            break;
        }
    }
    for k in 0..cnt {
        let fr = faces + m.r32(q + k * 0xc + 8) * 0x20;
        let v = m.r32(fr + 0x18);
        m.w32(fr + 0x18, v & !4);
    }
    stack_pop(m, 0x300);
    found
}

// ---------------------------------------------------------------- class C2d2e98 (boss, base C2cbe68 / C2cb230; state +0x18c8)
/// FUN_00188488 (C2d2e98_v6): base C2cbe68_v6 (FUN_0013ff70); +0x18d0 / +0x18c8 / +0x18d8 = -1, +0x19ac = 15;
/// FUN_0020b840(+0x1320 / +0x1c30, this, 0x6f); move limits +0x1098.. = {-100, 100, 50, -10, 250, 0, 1,
/// 4500}; FUN_00204d48(this, 1.6); +0x1fc8 0x26, +0x1fd4 0x29, +0x1fb0 0x1a, +0x1fbc 0x25;
/// FUN_001371b8(+0x1ff0, this); +0x1000 1, +0x1020 200.0, +0x264 199; FUN_001cc158(+0x17d0);
/// DAT_002e1e40 = this
pub fn ee_sub_188488<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0013_ff70, &[o], &[]);
    m.w32(o + 0x18d0, u32::MAX);
    m.w32(o + 0x19ac, 0x4170_0000);
    m.w32(o + 0x18c8, u32::MAX);
    m.w32(o + 0x18d8, u32::MAX);
    m.call(0x0020_b840, &[o + 0x1320, o, 0x6f], &[]);
    m.call(0x0020_b840, &[o + 0x1c30, o, 0x6f], &[]);
    for (off, v) in [
        (0x1098, 0xc2c8_0000u32),
        (0x109c, 0x42c8_0000),
        (0x10a0, 0x4248_0000),
        (0x10a4, 0xc120_0000),
        (0x10a8, 0x437a_0000),
        (0x10b0, 0x3f80_0000),
        (0x10b4, 0x458c_a000),
        (0x10ac, 0),
    ] {
        m.w32(o + off, v);
    }
    m.call(0x0020_4d48, &[o], &[s(f32::from_bits(0x3fcc_cccd))]);
    m.w32(o + 0x1fc8, 0x26);
    m.w32(o + 0x1fd4, 0x29);
    m.w32(o + 0x1fb0, 0x1a);
    m.w32(o + 0x1fbc, 0x25);
    m.call(0x0013_71b8, &[o + 0x1ff0, o], &[]);
    m.w32(o + 0x1000, 1);
    m.w32(o + 0x1020, 0x4348_0000);
    m.w32(o + 0x264, 199);
    m.call(0x001c_c158, &[o + 0x17d0], &[]);
    m.w32(0x002e_1e40, o);
}
/// FUN_00189488 (C2d2e98_v97): base C2cbe68_v97 (FUN_001409b8); -1 -> 1 in states 1, 5, 6, 9, else -1
pub fn ee_sub_189488<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let r = m.call(0x0014_09b8, &[o], &[]).0;
    if r != u32::MAX {
        return r;
    }
    match m.r32(o + 0x18c8) as i32 {
        1 | 5 | 6 | 9 => 1,
        _ => u32::MAX,
    }
}
/// FUN_0018a238 (C2d2e98_v18): state 3 -> FUN_00190d10; base C2cb230_v18 (FUN_001425b0)
pub fn ee_sub_18a238<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.r32(o + 0x18c8) == 3 {
        ee_sub_190d10(m, o);
    }
    m.call(0x0014_25b0, &[o], &[]);
}
/// FUN_0018a278 (C2d2e98_v16): +0x18a0 = target (+0x19a4) position, +0x1880 = 0; UpdateAlo (FUN_001f3980);
/// +0x18d8 == 0 -> +0x1890 = own position, +0x18b0 = target position, +0x1880 = 3; else +0x1880 = 0
pub fn ee_sub_18a278<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let t = m.r32(o + 0x19a4);
    let p = m.call(0x001a_62e8, &[t], &[]).1;
    m.w32(o + 0x1880, 0);
    m.wv(o + 0x18a0, p);
    m.call(0x001f_3980, &[o], &[]);
    if m.r32(o + 0x18d8) == 0 {
        let p = m.call(0x001a_62e8, &[o], &[]).1;
        m.wv(o + 0x1890, p);
        let t = m.r32(o + 0x19a4);
        let p = m.call(0x001a_62e8, &[t], &[]).1;
        m.wv(o + 0x18b0, p);
        m.w32(o + 0x1880, 3);
    } else {
        m.w32(o + 0x1880, 0);
    }
}
/// FUN_0018a2e0 (C2d2e98_v48): base C2caf88_v48 (FUN_001f50e0); FUN_001f5480(this, point, +0x19ac) for the
/// +0x1880 points at +0x1890
pub fn ee_sub_18a2e0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001f_50e0, &[o], &[]);
    let mut i = 0u32;
    while (i as i32) < m.r32(o + 0x1880) as i32 {
        let p = m.rv(o + 0x1890 + i * 0x10);
        let f = m.rf(o + 0x19ac);
        m.call(0x001f_5480, &[o], &[p, s(f)]);
        i += 1;
    }
}
/// FUN_0018a350 (C2d2e98_v96): state 0x15 -> *out = own position (no base); 3 / 0x16 -> base C2cb230_v96
/// (FUN_00141588) then out.z = (+0x18c4) position z - 175 / - 100; else the base
pub fn ee_sub_18a350<M: QMem + ?Sized>(m: &mut M, o: u32, out: u32) {
    let st = m.r32(o + 0x18c8);
    if st == 0x15 {
        let p = m.call(0x001a_62e8, &[o], &[]).1;
        m.wv(out, p);
        return;
    }
    m.call(0x0014_1588, &[o, out], &[]);
    let d = match st {
        3 => -175.0,
        0x16 => -100.0,
        _ => return,
    };
    let x = m.r32(o + 0x18c4);
    let p = m.call(0x001a_62e8, &[x], &[]).1;
    m.wf(out + 8, p[2] + d);
}
/// FUN_0018a428 (C2d2e98_v49): state not 0 / 0x12 with +0x4cc set and +0xf54 != 1 -> +0x1aa4 = clock; base
/// C2cbe68_v49 (FUN_00141640); then states 3, 5, 6 -> *flags |= 1; 8 -> FUN_0019ea00(+0xf50, flags)
pub fn ee_sub_18a428<M: QMem + ?Sized>(m: &mut M, o: u32, fl: u32) {
    let st = m.r32(o + 0x18c8);
    if st != 0 && st != 0x12 && m.r32(o + 0x4cc) != 0 && m.r32(o + 0xf54) != 1 {
        let c = m.r32(CLOCK);
        m.w32(o + 0x1aa4, c);
    }
    m.call(0x0014_1640, &[o, fl], &[]);
    match m.r32(o + 0x18c8) as i32 {
        3 | 5 | 6 => {
            let v = m.r32(fl);
            m.w32(fl, v | 1);
        }
        8 => drop(m.call(0x0019_ea00, &[o + 0xf50, fl], &[])),
        _ => {}
    }
}
/// FUN_0018c060 (C2d2e98_v89): animation by state (2: FUN_0019c870(+0x2c0, &k); 4: FUN_001a23f8(+0xdf0,
/// &k); 7: FUN_001944c8(+0x4c0, &k); 0x15 / 0x16: +0x520; else table +0x1720[state]); 0 -> +0x1720;
/// FUN_00203d00(this, anim, k)
pub fn ee_sub_18c060<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let sp = stack_push(m, 0x10);
    m.w32(sp, 0);
    let st = m.r32(o + 0x18c8);
    let mut a = match st {
        2 => m.call(0x0019_c870, &[o + 0x2c0, sp], &[]).0,
        4 => m.call(0x001a_23f8, &[o + 0xdf0, sp], &[]).0,
        7 => m.call(0x0019_44c8, &[o + 0x4c0, sp], &[]).0,
        0x15 | 0x16 => m.r32(o + 0x520),
        _ => m.r32(o.wrapping_add(st.wrapping_mul(4)) + 0x1720),
    };
    if a == 0 {
        a = m.r32(o + 0x1720);
    }
    let k = m.r32(sp);
    m.call(0x0020_3d00, &[o, a, k], &[]);
    stack_pop(m, 0x10);
}
/// FUN_0018d500 (C2d2e98_v105): +0xf04 != 1 and Revoke(0x2de2e0) == 0
pub fn ee_sub_18d500<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    if m.r32(o + 0xf04) == 1 {
        return 0;
    }
    (m.call(0x0014_fd50, &[0x002d_e2e0], &[]).0 == 0) as u32
}
/// FUN_0018d548 (C2d2e98_v90): FUN_0018e540(this, -1)
pub fn ee_sub_18d548<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0018_e540, &[o, u32::MAX], &[]);
}
/// FUN_0018d620 (C2d2e98_v88): state -> index (0 0, 2 1, 4 3, 7 2, 8 8, 0x12 4, 0x13 5, 0x14 6, 0x17 7, else -1)
pub fn ee_sub_18d620<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    match m.r32(o + 0x18c8) {
        0 => 0,
        2 => 1,
        4 => 3,
        7 => 2,
        8 => 8,
        0x12 => 4,
        0x13 => 5,
        0x14 => 6,
        0x17 => 7,
        _ => u32::MAX,
    }
}
/// FUN_0018de80 (C2d2e98_v95): base C2cb230_v95 (FUN_00143220); states 0, 1, 5, 10, 12 -> *out = 1
pub fn ee_sub_18de80<M: QMem + ?Sized>(m: &mut M, o: u32, out: u32, f12: f32) {
    // PART r16 ABI: $f12 is passed through untouched to FUN_00143220 (asm 0x18de94)
    m.call(0x0014_3220, &[o, out], &[s(f12)]);
    if matches!(m.r32(o + 0x18c8), 0 | 1 | 5 | 10 | 0xc) {
        m.w32(out, 1);
    }
}
/// FUN_0018dee0 (C2d2e98_v84): +0x204 set or state 3 / 8 / 0x15 / 0x16 -> 0; 0xd / 0xe -> 1; 0x18 -> 15;
/// else base C2cbe68_v84 (FUN_00143c78)
pub fn ee_sub_18dee0<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    if m.r32(o + 0x204) != 0 {
        return 0;
    }
    match m.r32(o + 0x18c8) {
        3 | 8 | 0x15 | 0x16 => 0,
        0xd | 0xe => 1,
        0x18 => 0xf,
        _ => m.call(0x0014_3c78, &[o], &[]).0,
    }
}
/// FUN_0018e3a0 (C2d2e98_v77): base C2cb230_v77 (FUN_00144480); FUN_00149ab8(world, this, DAT_002a5f98, 0)
pub fn ee_sub_18e3a0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0014_4480, &[o], &[]);
    let k = crate::engine::classes::q::q6::ro32(0x002a_5f98);
    m.call(0x0014_9ab8, &[WORLD, o, k, 0], &[]);
}
/// FUN_001904e0 (C2d2e98_v106): body (+0x1900)->+4: AddAloAccel(+0x1930) (FUN_001fac30), FUN_001fac98(+0x1940)
pub fn ee_sub_1904e0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let b = m.r32(o + 0x1900);
    let b = m.r32(b + 4);
    let a = m.rv(o + 0x1930);
    m.call(0x001f_ac30, &[b], &[a]);
    let b = m.r32(o + 0x1900);
    let b = m.r32(b + 4);
    let a = m.rv(o + 0x1940);
    m.call(0x001f_ac98, &[b], &[a]);
}
/// FUN_00190d10: carried body (+0x19c4): d = FUN_0022fcc8(0, body+0xa0, FUN_00190c58(this) + +0x1960 -
/// target (+0x19a4) position); FUN_00256d18(body + 0x30, body+0x30 + d), body+0x3c = 1.0; body+0xa4 =
/// ++world+0x3d58; body+0x40..0x7f = FUN_0011c168(body+0xa0 + 0x40, body); FUN_00186a50(this,
/// (+0x19cc)->+0xa8); vslot +0x94(this)
pub fn ee_sub_190d10<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let sp = stack_push(m, 0x40);
    let a = m.call(0x0019_0c58, &[o], &[]).1;
    let b = m.rv(o + 0x1960);
    let t = m.r32(o + 0x19a4);
    let p = m.call(0x001a_62e8, &[t], &[]).1;
    let v = [a[0] + b[0] - p[0], a[1] + b[1] - p[1], a[2] + b[2] - p[2], a[3] + b[3] - p[3]];
    let bd = m.r32(o + 0x19c4);
    let k = m.r32(bd + 0xa0);
    let d = m.call(0x0022_fcc8, &[0, k], &[v]).1;
    let bd = m.r32(o + 0x19c4);
    let c = m.rv(bd + 0x30);
    m.call(0x0025_6d18, &[bd + 0x30], &[[c[0] + d[0], c[1] + d[1], c[2] + d[2], c[3] + d[3]]]);
    m.w32(bd + 0x3c, 0x3f80_0000);
    let n = m.r32(WORLD + 0x3d58).wrapping_add(1);
    m.w32(WORLD + 0x3d58, n);
    let bd = m.r32(o + 0x19c4);
    m.w32(bd + 0xa4, n);
    let bd = m.r32(o + 0x19c4);
    let pa = m.r32(bd + 0xa0);
    m.call(0x0011_c168, &[sp, pa + 0x40, bd], &[]);
    for k in 0..4 {
        let q = m.rv(sp + 0x10 * k);
        m.wv(bd + 0x40 + 0x10 * k, q);
    }
    let x = m.r32(o + 0x19cc);
    let x = m.r32(x + 0xa8);
    m.call(0x0018_6a50, &[o, x], &[]);
    vcall(m, o, 0xc, 0x94, &[], &[]);
    stack_pop(m, 0x40);
}

// ---------------------------------------------------------------- class C2d15d0 (player-mounted turret, base C2d1328; state +0x1c0)
/// FUN_0017f6f8 (C2d15d0_v6): base C2d1878_v6 (FUN_0017f090); FUN_0017f3e0(this, 5, this + 0x1c8);
/// +0x1c0 = -1, DAT_002dfb80 = this, +0x2a8 = -1
pub fn ee_sub_17f6f8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0017_f090, &[o], &[]);
    m.call(0x0017_f3e0, &[o, 5, o + 0x1c8], &[]);
    m.w32(o + 0x1c0, u32::MAX);
    m.w32(0x002d_fb80, o);
    m.w32(o + 0x2a8, u32::MAX);
}
/// FUN_0017f740 (C2d15d0_v11): base C2d0b30_v11 (FUN_0017f110); state -1 (FUN_0017fd08)
pub fn ee_sub_17f740<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0017_f110, &[o], &[]);
    m.call(0x0017_fd08, &[o, u32::MAX], &[]);
}
/// FUN_0017f770: for the +0x230 records (0x18 each at +0x234 {+0 name, +4 id, +8 part, +0xc constraint,
/// +0x10 object, +0x14 handle}): part = FUN_001f4c58(this, name, 6); object = FUN_0015e938(world, 4, id,
/// this); part -> constraint FUN_0012dc18(this, part, 0, 7, 2, DAT_002a2dd0); object -> handle
/// FUN_001f0380(object, this, -1, 0)
pub fn ee_sub_17f770<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let mut i = 0u32;
    while (i as i32) < m.r32(o + 0x230) as i32 {
        let p = o + 0x234 + i * 0x18;
        let nm = m.r32(p);
        let x = m.call(0x001f_4c58, &[o, nm, 6], &[]).0;
        m.w32(p + 8, x);
        let id = m.r32(p + 4);
        let x = m.call(0x0015_e938, &[WORLD, 4, id, o], &[]).0;
        m.w32(p + 0x10, x);
        let pt = m.r32(p + 8);
        if pt != 0 {
            let q = rov(0x002a_2dd0);
            let x = m.call(0x0012_dc18, &[o, pt, 0, 7, 2], &[q]).0;
            m.w32(p + 0xc, x);
        }
        let ob = m.r32(p + 0x10);
        if ob != 0 {
            let x = m.call(0x001f_0380, &[ob, o, u32::MAX, 0], &[]).0;
            m.w32(p + 0x14, x);
        }
        i += 1;
    }
}
/// FUN_0017f850 (C2d15d0_v14): base C2d1328_v14 (FUN_0017f138); vslot +0x27c(this, 0); FUN_002303a0(this,
/// +0x58 / +0x5c, 2); FUN_0017f770; FUN_0017f408(this, 5, this + 0x1c8)
pub fn ee_sub_17f850<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0017_f138, &[o], &[]);
    vcall(m, o, 0xc, 0x27c, &[0], &[]);
    let a = m.r32(o + 0x58);
    m.call(0x0023_03a0, &[o, a, 2], &[]);
    let a = m.r32(o + 0x5c);
    m.call(0x0023_03a0, &[o, a, 2], &[]);
    ee_sub_17f770(m, o);
    m.call(0x0017_f408, &[o, 5, o + 0x1c8], &[]);
}
/// FUN_0017f8c8 (C2d15d0_v77): base C2d5898_v77 (FUN_001c3ff0); k != 0 -> state 1; else class-word bit 60
/// (+0x28) -> state 0
pub fn ee_sub_17f8c8<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32) {
    m.call(0x001c_3ff0, &[o, k], &[]);
    if k != 0 {
        m.call(0x0017_fd08, &[o, 1], &[]);
    } else if (m.r64(o + 0x28) >> 60) & 1 != 0 {
        m.call(0x0017_fd08, &[o, 0], &[]);
    }
}
/// FUN_0017f930 (C2d15d0_v15, update): base C2caf88_v15 (FUN_001f36e0); state 2 and FUN_0017f550 -> state 1
/// (re-run while the state changes); states 0, 2-4 -> heading vector +0x220 xyz = 0; 1 -> +0x218 = track
/// (+0x150) angle +0x18, +0x220 = FUN_0011e1a0(angle, +0x1c * DAT_002be7e8, 0); each record's constraint
/// gets FUN_0012d108(c, {0, 0, track+0x28}) and its handle FUN_001f0660(h, 0 < track+0x2c ? 0x68 : 0x66).
/// Sound +0x2a4: +0x2ac = PosApproachTrapezoid(+0x2ac, state 1 ? track+0x1c : 0, dt, 0x2be7f0, 0), volume
/// (FUN_001fb1f8); playing count +0x4e: > 0 with a level -> FUN_001fb060, <= 0 at 0 -> FUN_001fb018
pub fn ee_sub_17f930<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001f_36e0, &[o], &[]);
    let mut st = m.r32(o + 0x1c0);
    loop {
        if st == 2 && m.call(0x0017_f550, &[o], &[]).0 != 0 {
            m.call(0x0017_fd08, &[o, 1], &[]);
        }
        let n = m.r32(o + 0x1c0);
        if n == st {
            break;
        }
        st = n;
    }
    match st {
        0 | 2 | 3 | 4 => {
            m.w32(o + 0x224, 0);
            m.w32(o + 0x228, 0);
            m.w32(o + 0x220, 0);
        }
        1 => {
            let k = m.rf(0x002b_e7e8);
            let tr = m.r32(o + 0x150);
            let a = m.r32(tr + 0x18);
            m.w32(o + 0x218, a);
            let sp = m.rf(tr + 0x1c);
            let v = m.call(0x0011_e1a0, &[], &[s(f32::from_bits(a)), s(sp * k), s(0.0)]).1;
            m.wv(o + 0x220, v);
            let tr = m.r32(o + 0x150);
            let z = m.r32(tr + 0x28);
            let hi = ((z as i32) >> 31) as u32;
            let mut i = 0u32;
            loop {
                if (m.r32(o + 0x230) as i32) <= i as i32 {
                    break;
                }
                let p = o + 0x234 + i * 0x18;
                let c = m.r32(p + 0xc);
                if c != 0 {
                    m.call(0x0012_d108, &[c], &[[0.0, 0.0, f32::from_bits(z), f32::from_bits(hi)]]);
                }
                let h = m.r32(p + 0x14);
                if h != 0 {
                    let tr = m.r32(o + 0x150);
                    let k = if 0.0 < m.rf(tr + 0x2c) { 0x68 } else { 0x66 };
                    m.call(0x001f_0660, &[h, k], &[]);
                }
                i += 1;
            }
        }
        _ => {}
    }
    let snd = m.r32(o + 0x2a4);
    if snd != 0 {
        let tg = if m.r32(o + 0x1c0) == 1 {
            let tr = m.r32(o + 0x150);
            m.rf(tr + 0x1c)
        } else {
            0.0
        };
        let (cur, dt) = (m.rf(o + 0x2ac), m.rf(0x002d_ded4));
        let v = callf(m, 0x0011_d018, &[0x002b_e7f0, 0], &[s(cur), s(tg), s(dt)]);
        m.wf(o + 0x2ac, v);
        let snd = m.r32(o + 0x2a4);
        m.call(0x001f_b1f8, &[snd], &[s(v)]);
        let snd = m.r32(o + 0x2a4);
        let n = m.r16(snd + 0x4e) as i16;
        if 0.0 < m.rf(o + 0x2ac) {
            if 0 < n {
                m.call(0x001f_b060, &[snd], &[]);
            }
        } else if n < 1 {
            m.call(0x001f_b018, &[snd], &[]);
        }
    }
}
/// FUN_0017fb50 (C2d15d0_v69): state != 3 -> PoMoveToward(this, MatrixFromAxisAngle(0x2a2dc0, +0x218),
/// 0x2be800); FUN_001f5e10(this, DAT_002a2dd0, 0, heading +0x220, 0x2be810)
pub fn ee_sub_17fb50<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.r32(o + 0x1c0) == 3 {
        return;
    }
    let sp = stack_push(m, 0x80);
    let a = m.rf(o + 0x218);
    m.call(0x0011_b500, &[sp + 0x30], &[rov(0x002a_2dc0), s(a)]);
    for k in 0..3 {
        let q = m.rv(sp + 0x30 + 0x10 * k);
        m.wv(sp + 0x10 * k, q);
    }
    m.call(0x001f_59a0, &[o, sp, 0x002b_e800], &[]);
    let h = m.rv(o + 0x220);
    m.call(0x001f_5e10, &[o, 0, 0x002b_e810], &[rov(0x002a_2dd0), h]);
    stack_pop(m, 0x80);
}
/// FUN_0017fbe8 (C2d15d0_v74, use request): states 2-4 -> 0; FUN_0020f0b0(world) or DAT_002e1e00 bit 0 -> 0;
/// base C2d0dd8_v74 (FUN_0017f1d0) accepted -> FUN_00180100, then +0x184 < 1 -> state 4, else
/// FollowWorld_activity(0x2deb30, 0xf, *req) and state 2; returns 1
pub fn ee_sub_17fbe8<M: QMem + ?Sized>(m: &mut M, o: u32, rq: u32) -> u32 {
    let st = m.r32(o + 0x1c0) as i32;
    if (2..=4).contains(&st) {
        return 0;
    }
    if m.call(0x0020_f0b0, &[WORLD], &[]).0 != 0 || m.r32(0x002e_1e00) & 1 != 0 {
        return 0;
    }
    if m.call(0x0017_f1d0, &[o, rq], &[]).0 == 0 {
        return 0;
    }
    m.call(0x0018_0100, &[o], &[]);
    if (m.r32(o + 0x184) as i32) < 1 {
        m.call(0x0017_fd08, &[o, 4], &[]);
    } else {
        let a = m.r32(rq);
        m.call(0x0016_19a0, &[0x002d_eb30, 0xf, a], &[]);
        m.call(0x0017_fd08, &[o, 2], &[]);
    }
    1
}
/// FUN_0017fcb0 (C2d15d0_v82): base C2d1328_v82 (FUN_0017f6b0); state 1 when this is the world focus
/// (FUN_001c40f8) else 0; FUN_00180100
pub fn ee_sub_17fcb0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0017_f6b0, &[o], &[]);
    let k = ee_sub_1c40f8(m, o);
    m.call(0x0017_fd08, &[o, k], &[]);
    m.call(0x0018_0100, &[o], &[]);
}
/// FUN_001c40f8: x == world focus (FUN_001c4a48)
pub fn ee_sub_1c40f8<M: QMem + ?Sized>(m: &mut M, x: u32) -> u32 {
    (ee_sub_1c4a48(m, WORLD) == x) as u32
}

// ---------------------------------------------------------------- classes C2d9c70 / C2d9be0 (sound emitters, IOP RPC 'SUKR')
const SUKR: u32 = 0x524b_5553;
/// +0x1c == 0 and position +0x20 xyz within 1e-4 of DAT_002a2dd0 (FFloatsNear per axis)
fn at_origin<M: QMem + ?Sized>(m: &mut M, o: u32) -> bool {
    if m.r32(o + 0x1c) != 0 {
        return false;
    }
    let z = rov(0x002a_2dd0);
    let p = m.rv(o + 0x20);
    let eps = f32::from_bits(0x38d1_b717);
    for k in 0..3 {
        if m.call(0x0011_d4d0, &[], &[s(z[k]), s(p[k]), s(eps)]).0 == 0 {
            return false;
        }
    }
    true
}
/// FUN_001fb0c0 (C2d9c70_v6): k = 0x1f1 (0x1f3 while playing, +0x4e > 0), | 0x200 unless at the origin;
/// FUN_001fb658(this, k, k)
pub fn ee_sub_1fb0c0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if !idle(m, o) {
        return; // VER4: asm 0x1fb0e0..0x1fb110 gate
    }
    let mut k = if 0 < m.r16(o + 0x4e) as i16 { 0x1f3 } else { 0x1f1 };
    if !at_origin(m, o) {
        k |= 0x200;
    }
    m.call(0x001f_b658, &[o, k, k], &[]);
}
/// +0x40 == 0 and +0x52 == 0 and !(+0x50 & 1)
fn idle<M: QMem + ?Sized>(m: &mut M, o: u32) -> bool {
    m.r32(o + 0x40) == 0 && m.r16(o + 0x52) == 0 && m.r16(o + 0x50) & 1 == 0
}
/// FUN_001fc270 (C2d9be0_v6): idle -> C2d9c70_v6, u64 +0x68 = 0, +0x3bc = 15
pub fn ee_sub_1fc270<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if idle(m, o) {
        ee_sub_1fb0c0(m, o);
        m.w32(o + 0x68, 0);
        m.w32(o + 0x6c, 0);
        m.w32(o + 0x3bc, 0xf);
    }
}
/// FUN_001fc4f8 (C2d9be0_v7): +0x3b8 & 4 ? 2 : 1
pub fn ee_sub_1fc4f8<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    if m.r32(o + 0x3b8) & 4 != 0 {
        2
    } else {
        1
    }
}
/// FUN_001fc510 (C2d9be0_v8, start): FileLocation(&loc) (kind 2); mode = FUN_001fd080(0x3dd040, bank +0x60);
/// bank set with (bank[1] ^ bank[0xf]) != 0 -> loc = {(bank[0] ^ bank[0xd]) + 0xca0, that, 2}; kind 2 ->
/// play++ (FUN_001fb018), +0x52 = 1, vslot +0xc, SUKR 1 with {loc, (int)(+0x44 * 1024), +0x14, mode,
/// +0x3b8 | 1, DAT_003dd050 && mode 2 ? bank+0x4c == 0 : 0} and callback 0x1fcba8, FUN_001fb658(this,
/// 0x3f, 0); else FUN_001fb658(this, 0x1fff, 0)
pub fn ee_sub_1fc510<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let sp = stack_push(m, 0x130);
    m.w32(sp + 8, 2);
    m.call(0x0025_8248, &[sp], &[]);
    let b = m.r32(o + 0x60);
    let mode = m.call(0x001f_d080, &[0x003d_d040, b], &[]).0;
    let b = m.r32(o + 0x60);
    let ok = if b == 0 {
        false
    } else {
        let a = (m.r32(b) ^ m.r32(b + 0x34)).wrapping_add(0xca0);
        let c = m.r32(b + 4) ^ m.r32(b + 0x3c);
        m.w32(sp + 8, 2);
        m.w32(sp, a);
        m.w32(sp + 4, c);
        c != 0
    };
    let k = if !ok || m.r32(sp + 8) != 2 {
        0x1fff
    } else {
        let a = m.r32(sp);
        m.w32(sp + 0x110, a);
        m.call(0x001f_b018, &[o], &[]);
        let v = (m.rf(o + 0x44) * 1024.0) as i32 as u32;
        let fl = m.r32(o + 0x3b8) | 1;
        let id = m.r32(o + 0x14);
        m.w32(sp + 0x120, fl);
        m.w32(sp + 0x118, id);
        m.w32(sp + 0x114, v);
        m.w32(sp + 0x11c, mode);
        let mut x = 0;
        if m.r32(0x003d_d050) != 0 && mode == 2 {
            let b = m.r32(o + 0x60);
            x = (m.r32(b + 0x4c) == 0) as u32;
        }
        m.w32(sp + 0x124, x);
        m.w16(o + 0x52, 1);
        vcall(m, o, 0xc, 0xc, &[], &[]);
        m.call(0x0023_3130, &[SUKR, 1, 0x18, sp + 0x110, 0x001f_cba8, o], &[]);
        0x3f
    };
    m.call(0x001f_b658, &[o, k, 0], &[]);
    stack_pop(m, 0x130);
}
/// FUN_001fc698 / 6d0 / 708 (C2d9be0_v9..v11): SUKR command 2 / 3 / 4 for handle +0x40
pub fn ee_sub_1fc698<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let h = m.r32(o + 0x40);
    m.call(0x0023_3088, &[SUKR, 2, h, 0, 0, 0, 0], &[]);
}
/// FUN_001fc6d0: SUKR 3
pub fn ee_sub_1fc6d0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let h = m.r32(o + 0x40);
    m.call(0x0023_3088, &[SUKR, 3, h, 0, 0, 0, 0], &[]);
}
/// FUN_001fc708: SUKR 4
pub fn ee_sub_1fc708<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let h = m.r32(o + 0x40);
    m.call(0x0023_3088, &[SUKR, 4, h, 0, 0, 0, 0], &[]);
}
/// FUN_001fc740 (C2d9be0_v12, flush changes): +0x50 & 0x70 -> SUKR 5(h, flags, vol, +0x14, pitch) with
/// 0x10: flag 1, vol = (int)(+0x44 * 1024); 0x20: flag 4; 0x40: flag 8, pitch = (int)(+0x18 * 12) (`t0` /
/// `t2` = the caller's stale registers when unset); +0x50 & 0x180 -> SUKR 6(h, i, (int)(+0x70[i] * 1024),
/// +0x88[i], 0) for every bit i < 6 of +0x3bc, then +0x3bc = 0
pub fn ee_sub_1fc740<M: QMem + ?Sized>(m: &mut M, o: u32, t0: u32, t2: u32) {
    let f = m.r16(o + 0x50);
    if f & 0x70 != 0 {
        let (mut fl, mut t0, mut t2) = (0u32, t0, t2);
        if f & 0x10 != 0 {
            fl = 1;
            t0 = (m.rf(o + 0x44) * 1024.0) as i32 as u32;
        }
        if f & 0x20 != 0 {
            fl |= 4;
        }
        if f & 0x40 != 0 {
            fl |= 8;
            t2 = (m.rf(o + 0x18) * 12.0) as i32 as u32;
        }
        let (h, id) = (m.r32(o + 0x40), m.r32(o + 0x14));
        m.call(0x0023_3088, &[SUKR, 5, h, fl, t0, id, t2], &[]);
    }
    if m.r16(o + 0x50) & 0x180 != 0 {
        for i in 0..6u32 {
            if m.r32(o + 0x3bc) & (1 << i) != 0 {
                let h = m.r32(o + 0x40);
                let v = (m.rf(o + 0x70 + 4 * i) * 1024.0) as i32 as u32;
                let p = m.r32(o + 0x88 + 4 * i);
                m.call(0x0023_3088, &[SUKR, 6, h, i, v, p, 0], &[]);
            }
        }
        m.w32(o + 0x3bc, 0);
    }
}
/// FUN_001fc878 (C2d9be0_v13): +0x50 & 0x1000 -> vslot +0xc, Sukr 7 (callback 0x1fcc30),
/// FUN_001fb658(this, 0x1000, 0); then not started (+0x52 == 0): 0x400 -> vslot +0xc, Sukr 8 (callback
/// 0x1fbb30), FUN_001fb658(this, 0x400, 0); 0x800 and not playing (+0x4e < 1) -> u64 +0x68 +=
/// DAT_003dd230 - DAT_003dd238
pub fn ee_sub_1fc878<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.r16(o + 0x50) & 0x1000 != 0 {
        vcall(m, o, 0xc, 0xc, &[], &[]);
        let h = m.r32(o + 0x40);
        m.call(0x0023_30d0, &[SUKR, 7, h, 0, 0, 0, 0, 0x001f_cc30], &[]);
        m.call(0x001f_b658, &[o, 0x1000, 0], &[]);
    }
    if m.r16(o + 0x52) != 0 {
        return;
    }
    if m.r16(o + 0x50) & 0x400 != 0 {
        vcall(m, o, 0xc, 0xc, &[], &[]);
        let h = m.r32(o + 0x40);
        m.call(0x0023_30d0, &[SUKR, 8, h, 0, 0, 0, 0, 0x001f_bb30], &[]);
        m.call(0x001f_b658, &[o, 0x400, 0], &[]);
    }
    if m.r16(o + 0x50) & 0x800 != 0 && (m.r16(o + 0x4e) as i16) < 1 {
        let d = m.r64(0x003d_d230).wrapping_sub(m.r64(0x003d_d238));
        let v = m.r64(o + 0x68).wrapping_add(d);
        m.w32(o + 0x68, v as u32);
        m.w32(o + 0x6c, (v >> 32) as u32);
    }
}
/// FUN_001fc9b0 (C2d9be0_v15, update): C2d9c70_v15 (FUN_001fb528) unless at the origin; per channel i < 6
/// with listeners (+0xa0[i], records of 0x10 at +0xb8 + i * 0x80: {object, out.., scale +0xc}): for every
/// listener object with class-word bit 60: FUN_001fb778(this, its position, rec + 4, &gain, first ? &pan :
/// 0); level = max(0, gain * scale); FUN_001fc380(this, i, exactly one ? pan : 0); FUN_001fc2e0(this, i,
/// level)
pub fn ee_sub_1fc9b0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if !at_origin(m, o) {
        m.call(0x001f_b528, &[o], &[]);
    }
    let sp = stack_push(m, 0x10);
    for i in 0..6u32 {
        let n = m.r32(o + 0xa0 + 4 * i);
        if n == 0 {
            continue;
        }
        let mut lvl = 0.0f32;
        m.w32(sp, 0);
        let mut cnt = 0u32;
        let mut pan = sp;
        let mut left = n;
        let mut r = o + i * 0x80 + 0xb8;
        if 0 < n as i32 {
            loop {
                let x = m.r32(r);
                left = left.wrapping_sub(1);
                if x != 0 && (m.r64(x + 0x28) >> 60) & 1 != 0 {
                    cnt += 1;
                    let p = m.call(0x001a_62e8, &[x], &[]).1;
                    m.call(0x001f_b778, &[o, r + 4, sp + 4, pan], &[p]);
                    pan = 0;
                    let g = m.rf(sp + 4) * m.rf(r + 0xc);
                    lvl = if lvl < g { g } else { lvl };
                }
                r += 0x10;
                if left == 0 {
                    break;
                }
            }
        }
        let pv = if cnt == 1 { m.r32(sp) } else { 0 };
        m.call(0x001f_c380, &[o, i, pv], &[]);
        m.call(0x001f_c2e0, &[o, i], &[s(lvl)]);
    }
    stack_pop(m, 0x10);
}
/// FUN_001fcc90 (C2d9be0_v0, AddRef): i16 +0x4c += 1
pub fn ee_sub_1fcc90<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let v = m.r16(o + 0x4c);
    m.w16(o + 0x4c, v.wrapping_add(1));
}
/// FUN_001fcca0 (C2d9be0_v1, Release): i16 +0x4c -= 1; reaching 0 -> FUN_00258348(this, 0), FUN_001fcd10
pub fn ee_sub_1fcca0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let v = m.r16(o + 0x4c);
    m.w16(o + 0x4c, v.wrapping_sub(1));
    if v == 1 && o != 0 {
        m.call(0x0025_8348, &[o, 0], &[]);
        ee_sub_1fcd10(m, o);
    }
}
/// FUN_001fcd10: free into the emitter pool (FUN_0011cd98(0x3dd074, this))
pub fn ee_sub_1fcd10<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0011_cd98, &[0x003d_d074, o], &[]);
}

// ---------------------------------------------------------------- class C2cd748
/// FUN_00204768 (C2cd748_v57, contact filter): base C2caf88_v57 (FUN_001f4f10); over the contact chains of
/// the world-lists (+0xec: list nodes from +0x64 {head, next, &store}, then the chain at +0x68; chains linked
/// by +0xac): best = the kind-0xe (+0x9c) contact with the smallest +0x90 (below DAT_002a85f4); +0x208 =
/// best, with one: +0x20c = clock, +0x230 = its +0x50 quad. Then every other kind-0xe contact whose side
/// (+0xc, or +0x4c when +8 is not this) is this is unlinked into a free chain (pushed through +0xac); list
/// heads are written back to their store; the chain is freed (FUN_001a7e10(world, chain))
pub fn ee_sub_204768<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001f_4f10, &[o], &[]);
    let mut best = 0u32;
    let mut lo = crate::engine::classes::q::q6::rof(0x002a_85f4);
    let ec = m.r32(o + 0xec);
    let mut n = m.r32(ec + 0x64);
    loop {
        let mut c = if n == 0 {
            let ec = m.r32(o + 0xec);
            m.r32(ec + 0x68)
        } else {
            m.r32(n)
        };
        while c != 0 {
            if m.r32(c + 0x9c) == 0xe {
                let d = m.rf(c + 0x90);
                if d < lo {
                    lo = d;
                    best = c;
                }
            }
            c = m.r32(c + 0xac);
        }
        if n == 0 {
            break;
        }
        n = m.r32(n + 4);
    }
    m.w32(o + 0x208, best);
    if best != 0 {
        let t = m.r32(CLOCK);
        m.w32(o + 0x20c, t);
        let q = m.rv(best + 0x50);
        m.wv(o + 0x230, q);
    }
    let mut freed = 0u32;
    let ec = m.r32(o + 0xec);
    let mut n = m.r32(ec + 0x64);
    loop {
        let mut link = if n == 0 {
            let ec = m.r32(o + 0xec);
            ec + 0x68
        } else {
            n
        };
        let mut c = m.r32(link);
        while c != 0 {
            let side = if o != m.r32(c + 8) { 0x40 } else { 0 };
            if m.r32(c + side + 0xc) == o && m.r32(c + 0x9c) == 0xe && c != best {
                let nx = m.r32(c + 0xac);
                m.w32(link, nx);
                m.w32(c + 0xac, freed);
                freed = c;
            } else {
                link = c + 0xac;
            }
            c = m.r32(link);
        }
        if n == 0 {
            break;
        }
        let (h, st) = (m.r32(n), m.r32(n + 8));
        m.w32(st, h);
        n = m.r32(n + 4);
    }
    ee_sub_1a7e10(m, WORLD, freed);
}

// ---------------------------------------------------------------- run_0011bfb8 (VU0 matrix helpers)
fn sc(v: V4, k: f32) -> V4 {
    [v[0] * k, v[1] * k, v[2] * k, v[3] * k]
}
/// FUN_0011bfb8: FUN_0011bcf8(p, &k, &m) then m row 0 * k
pub fn ee_sub_11bfb8<M: QMem + ?Sized>(m: &mut M, p: u32) -> V4 {
    let sp = stack_push(m, 0x40);
    m.call(0x0011_bcf8, &[p, sp + 0x20, sp], &[]);
    let k = m.rf(sp + 0x20);
    let r = sc(m.rv(sp), k);
    m.wv(sp + 0x10, s(k));
    stack_pop(m, 0x40);
    r
}
/// FUN_0011c128: out rows = src rows * x (x = the low word of a2); returns out
pub fn ee_sub_11c128<M: QMem + ?Sized>(m: &mut M, out: u32, src: u32, x: u32) -> u32 {
    let k = f32::from_bits(x);
    for i in 0..4 {
        let r = sc(m.rv(src + 0x10 * i), k);
        m.wv(out + 0x10 * i, r);
    }
    out
}
/// FUN_0011c1a8: rows 0..2 * v.xyz + row 3 * v.w (all lanes)
pub fn ee_sub_11c1a8<M: QMem + ?Sized>(m: &mut M, mt: u32, v: V4) -> V4 {
    let r: Vec<V4> = (0..4).map(|i| m.rv(mt + 0x10 * i)).collect();
    let mut o = [0.0; 4];
    for k in 0..4 {
        o[k] = r[0][k] * v[0] + r[1][k] * v[1] + r[2][k] * v[2] + r[3][k] * v[3];
    }
    o
}
/// FUN_0011c1d8: out = transpose(src) (FUN_0011c8e8(4, 4, src, out)); returns out
pub fn ee_sub_11c1d8<M: QMem + ?Sized>(m: &mut M, out: u32, src: u32) -> u32 {
    ee_sub_11c8e8(m, 4, 4, src, out);
    out
}
/// rows 0..2 of `mt` scaled by k[i], row 3 = t; w lanes 0, 0, 0, 1
fn scale_rows<M: QMem + ?Sized>(m: &mut M, out: u32, t: V4, mt: u32, k: [f32; 3]) {
    for i in 0..3u32 {
        let r = sc(m.rv(mt + 0x10 * i), k[i as usize]);
        m.wv(out + 0x10 * i, r);
    }
    m.wv(out + 0x30, t);
    for i in 0..3 {
        m.wf(out + 0xc + 0x10 * i, 0.0);
    }
    m.wf(out + 0x3c, 1.0);
}
/// FUN_0011c210: out rows i = mt rows i * k[i] (3 floats at `k`), row 3 = t (w 1); returns out
pub fn ee_sub_11c210<M: QMem + ?Sized>(m: &mut M, out: u32, t: V4, mt: u32, k: u32) -> u32 {
    let k = [m.rf(k), m.rf(k + 4), m.rf(k + 8)];
    scale_rows(m, out, t, mt, k);
    out
}
/// FUN_0011c288: out rows = mt rows * f, row 3 = t (w 1); returns out
pub fn ee_sub_11c288<M: QMem + ?Sized>(m: &mut M, out: u32, t: V4, mt: u32, f: f32) -> u32 {
    scale_rows(m, out, t, mt, [f; 3]);
    out
}
/// FUN_0011c2f0: FUN_0011c500(4, 4, a, b) (4x4 inverse)
pub fn ee_sub_11c2f0<M: QMem + ?Sized>(m: &mut M, a: u32, b: u32) {
    m.call(0x0011_c500, &[4, 4, a, b], &[]);
}
/// FUN_0011c3d0: out = b * inverse(a) (FUN_0011c318 then FUN_0011c168(tmp, b, inv))
pub fn ee_sub_11c3d0<M: QMem + ?Sized>(m: &mut M, a: u32, b: u32, out: u32) {
    let sp = stack_push(m, 0x80);
    m.call(0x0011_c318, &[sp + 0x40, a], &[]);
    m.call(0x0011_c168, &[sp, b, sp + 0x40], &[]);
    for i in 0..4 {
        let q = m.rv(sp + 0x10 * i);
        m.wv(out + 0x10 * i, q);
    }
    stack_pop(m, 0x80);
}
/// FUN_0011c448: projection matrix: zeroed (memset 0x40), [0] = sx, [5] = sy, [8] = cx, [9] = cy, [11] = 1,
/// f = (n + fr) / (n - fr): [10] = f, [14] = n * (1 - f)
pub fn ee_sub_11c448<M: QMem + ?Sized>(m: &mut M, out: u32, sx: f32, sy: f32, cx: f32, cy: f32, n: f32, fr: f32) {
    m.call(0x0023_33b4, &[out, 0, 0x40], &[]);
    let f = crate::engine::math::ps2f::ps2_div(n + fr, n - fr); // PS2 div.s
    m.wf(out, sx);
    m.wf(out + 0x2c, 1.0);
    m.wf(out + 0x20, cx);
    m.wf(out + 0x14, sy);
    m.wf(out + 0x24, cy);
    m.wf(out + 0x28, f);
    m.wf(out + 0x38, n * (1.0 - f));
}
/// FUN_0011c8e8: dst[i + j * n] = src[i * n + j] for i, j < c (words)
pub fn ee_sub_11c8e8<M: QMem + ?Sized>(m: &mut M, n: u32, c: u32, src: u32, dst: u32) {
    for i in 0..c as i32 {
        for j in 0..c as i32 {
            let v = m.r32(src.wrapping_add(((i * n as i32 + j) * 4) as u32));
            m.w32(dst.wrapping_add(((i + j * n as i32) * 4) as u32), v);
        }
    }
}
/// FUN_0011cab0: largest row length of the 3x3 part (sqrt of the max squared xyz length)
pub fn ee_sub_11cab0<M: QMem + ?Sized>(m: &mut M, mt: u32) -> f32 {
    let l = |v: V4| v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
    let a = l(m.rv(mt));
    let b = l(m.rv(mt + 0x10));
    let c = l(m.rv(mt + 0x20));
    let r = if b < a && c < a {
        a
    } else if c < b {
        b
    } else {
        c
    };
    r.sqrt()
}
/// FUN_0011cca8: *p = {x, y, z, 0}; returns p
pub fn ee_sub_11cca8<M: QMem + ?Sized>(m: &mut M, p: u32, x: u32, y: u32, z: u32) -> u32 {
    m.w32(p, x);
    m.w32(p + 4, y);
    m.w32(p + 8, z);
    m.w32(p + 0xc, 0);
    p
}
/// FUN_0011cdb8: pool reset (+0xc, +0x14, +0x10 = 0)
pub fn ee_sub_11cdb8<M: QMem + ?Sized>(m: &mut M, p: u32) {
    m.w32(p + 0xc, 0);
    m.w32(p + 0x14, 0);
    m.w32(p + 0x10, 0);
}
/// FUN_0011cdd8: take a node (FUN_0011ccc0) and free it straight back (FUN_0011cd98(p, node))
pub fn ee_sub_11cdd8<M: QMem + ?Sized>(m: &mut M, p: u32) {
    let x = m.call(0x0011_ccc0, &[p], &[]).0;
    m.call(0x0011_cd98, &[p, x], &[]);
}

// ---------------------------------------------------------------- small helpers (runs 0015a138 / 001856b8 / 00154908 / 0012fde8 / 001bb400)
/// FUN_0015a138: +0x148 = clock + t
pub fn ee_sub_15a138<M: QMem + ?Sized>(m: &mut M, o: u32, t: f32) {
    let c = m.rf(CLOCK);
    m.wf(o + 0x148, c + t);
}
/// FUN_0015a160: +0x148 = -1.0
pub fn ee_sub_15a160<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.w32(o + 0x148, 0xbf80_0000);
}
/// FUN_0015a240: +0xf0 == -1 -> +0xf4 = k; +0xf0 == 0 -> k 0: FUN_0015a150(this), k -1: FUN_0015a160,
/// else +0xf4 = k; +0xe0 = 1
pub fn ee_sub_15a240<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32) {
    match m.r32(o + 0xf0) {
        u32::MAX => m.w32(o + 0xf4, k),
        0 => match k {
            0 => drop(m.call(0x0015_a150, &[o], &[])),
            u32::MAX => ee_sub_15a160(m, o),
            _ => m.w32(o + 0xf4, k),
        },
        _ => {}
    }
    m.w32(o + 0xe0, 1);
}
/// FUN_001856b8: *p = 0
pub fn ee_sub_1856b8<M: QMem + ?Sized>(m: &mut M, _a0: u32, p: u32) {
    m.w32(p, 0);
}
/// FUN_00185e30: +0x1f30 = a, +0x1f40 = b, +0x1f50 = c
pub fn ee_sub_185e30<M: QMem + ?Sized>(m: &mut M, o: u32, a: V4, b: V4, c: V4) {
    m.wv(o + 0x1f50, c);
    m.wv(o + 0x1f30, a);
    m.wv(o + 0x1f40, b);
}
/// FUN_00154a60: memset(p, 0, 0x18)
pub fn ee_sub_154a60<M: QMem + ?Sized>(m: &mut M, p: u32) {
    m.call(0x0023_33b4, &[p, 0, 0x18], &[]);
}
/// FUN_00154a80: +0x10, +8, +0xc = 0
pub fn ee_sub_154a80<M: QMem + ?Sized>(m: &mut M, p: u32) {
    m.w32(p + 0x10, 0);
    m.w32(p + 8, 0);
    m.w32(p + 0xc, 0);
}
/// FUN_00154d88: FUN_00154dc8(this, FUN_00154c90(this) + t)
pub fn ee_sub_154d88<M: QMem + ?Sized>(m: &mut M, o: u32, t: f32) {
    let v = callf(m, 0x0015_4c90, &[o], &[]);
    ee_sub_154dc8(m, o, v + t);
}
/// FUN_00154dc8: world +0x3b0c set -> *(it +0xa8) = FUN_0011d948(+0x14, v) (clamp)
pub fn ee_sub_154dc8<M: QMem + ?Sized>(m: &mut M, o: u32, v: f32) {
    if m.r32(WORLD + 0x3b0c) == 0 {
        return;
    }
    let r = m.r32(o + 0x14);
    let x = m.call(0x0011_d948, &[r], &[s(v)]).1[0];
    let w = m.r32(WORLD + 0x3b0c);
    let p = m.r32(w + 0xa8);
    m.wf(p, x);
}
/// FUN_00155530: append {a, _, b, _, c, _, d} (0x24 bytes) to the list +0x1014 (count +0x1010)
pub fn ee_sub_155530<M: QMem + ?Sized>(m: &mut M, o: u32, a: u32, b: u32, c: u32, d: u32) {
    let n = m.r32(o + 0x1010);
    m.w32(o + 0x1010, n + 1);
    let p = m.r32(o + 0x1014).wrapping_add(n.wrapping_mul(0x24));
    m.w32(p + 0x18, d);
    m.w32(p, a);
    m.w32(p + 8, b);
    m.w32(p + 0x10, c);
}
/// FUN_0012fde8: +0xb0 = new {a, b, c} (FUN_001ae1d0 {0xc, 0x10})
/// VER5: ABI per asm: o = a0, c = a1, a = $f12, b = $f13
pub fn ee_sub_12fde8<M: QMem + ?Sized>(m: &mut M, o: u32, c: u32, a: f32, b: f32) {
    let sp = stack_push(m, 0x10);
    m.w32(sp, 0xc);
    m.w32(sp + 4, 0x10);
    let p = m.call(0x001a_e1d0, &[sp], &[]).0;
    m.w32(o + 0xb0, p);
    m.w32(p + 8, c);
    m.wf(p, a);
    m.wf(p + 4, b);
    stack_pop(m, 0x10);
}
/// FUN_00130010: *obj = FUN_0015e938(world, owner ? 1 : 4, rec[1], owner ? owner : ctx); *out = obj ?
/// vslot +0x15c(obj, rec[0]) : 0
pub fn ee_sub_130010<M: QMem + ?Sized>(m: &mut M, ctx: u32, owner: u32, rec: u32, obj: u32, out: u32) {
    let id = m.r32(rec + 4);
    let (k, c) = if owner == 0 { (4, ctx) } else { (1, owner) };
    let x = m.call(0x0015_e938, &[WORLD, k, id, c], &[]).0;
    m.w32(obj, x);
    let x = m.r32(obj);
    if x == 0 {
        m.w32(out, 0);
    } else {
        let a = m.r32(rec);
        let v = vcall(m, x, 0xc, 0x15c, &[a], &[]).0;
        m.w32(out, v);
    }
}
/// FUN_001304a8: the +0x2c-chained child of +0x50 whose +0x18 is `k` (0 when none)
pub fn ee_sub_1304a8<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32) -> u32 {
    let mut c = m.r32(o + 0x50);
    while c != 0 && m.r32(c + 0x18) != k {
        c = m.r32(c + 0x2c);
    }
    c
}
/// FUN_001304e0: out (t0, or a local) = FUN_001304a8(a0, a1), when 0 created by FUN_0012f9b8(a0, a1, a3,
/// out, 0.0, 1.0); FUN_00130d30(*out, a2, f12, f13)
#[allow(clippy::too_many_arguments)]
pub fn ee_sub_1304e0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, t0: u32, f0: f32, f1: f32) {
    let sp = stack_push(m, 0x10);
    let out = if t0 != 0 { t0 } else { sp };
    let x = ee_sub_1304a8(m, a0, a1);
    m.w32(sp, x);
    if x == 0 {
        m.call(0x0012_f9b8, &[a0, a1, a3, out], &[s(0.0), s(1.0)]);
    } else {
        m.w32(out, x);
    }
    let v = m.r32(out);
    m.call(0x0013_0d30, &[v, a2], &[s(f0), s(f1)]);
    stack_pop(m, 0x10);
}
/// FUN_001306a8: *p set -> vslot +0x14 (Release), *p = 0
pub fn ee_sub_1306a8<M: QMem + ?Sized>(m: &mut M, p: u32) {
    let x = m.r32(p);
    if x != 0 {
        vcall(m, x, 0xc, 0x14, &[], &[]);
        m.w32(p, 0);
    }
}
/// FUN_001306f0: FUN_00131a20(child, s) for every +0x2c-chained child of o+0x50; o+0x78 = s
/// (asm: o = a0, s = $f12)
pub fn ee_sub_1306f0<M: QMem + ?Sized>(m: &mut M, o: u32, s: f32) {
    let mut c = m.r32(o + 0x50);
    while c != 0 {
        m.call(0x0013_1a20, &[c], &[[s, 0.0, 0.0, 0.0]]);
        c = m.r32(c + 0x2c);
    }
    m.wf(o + 0x78, s);
}
/// FUN_001bbc08: FUN_001bbe18(o, x) == 0 -> FUN_001f6ac8(o, x); else FUN_001bbdb0(o, 4)+4 = x
pub fn ee_sub_1bbc08<M: QMem + ?Sized>(m: &mut M, o: u32, x: u32) {
    if m.call(0x001b_be18, &[o, x], &[]).0 == 0 {
        m.call(0x001f_6ac8, &[o, x], &[]);
    } else {
        let p = m.call(0x001b_bdb0, &[o, 4], &[]).0;
        m.w32(p + 4, x);
    }
}
/// FUN_001bbcf0: as FUN_001bbc08 with FUN_001f6b48 / kind 5
pub fn ee_sub_1bbcf0<M: QMem + ?Sized>(m: &mut M, o: u32, x: u32) {
    if m.call(0x001b_be18, &[o, x], &[]).0 == 0 {
        m.call(0x001f_6b48, &[o, x], &[]);
    } else {
        let p = m.call(0x001b_bdb0, &[o, 5], &[]).0;
        m.w32(p + 4, x);
    }
}

// ---------------------------------------------------------------- run_0019a4a0 (jump / intercept estimates)
fn add4(a: V4, b: V4) -> V4 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2], a[3] + b[3]]
}
/// FUN_0019a4a0: jump estimate of `o` (*o = the actor) to `to` (a1) over surface `srf` (a2):
/// d = FUN_0022d348(vslot +0x104(srf, 0), 0, from); FUN_0019a400(o, pos, to, from, d, &t) == 0 -> ballistic
/// fall: g = (*o)->+0xe8 +0x88, t = max(0, (d.z - to.z) / g), *t_out = t, *cost = 1010000 - (pos.z -
/// from.z + g * t * t / 2 + (to.z - d.z) * t); else sum = d + v (t0), end = from + d * min(0.15, t) (w +1),
/// *t_out = t, *cost = min(1e6, (|hermite'(0)|.xy + |hermite'(t)|.xy) * t / 2) (two FUN_00117120 over pos,
/// to, end, sum; outputs at t2), *end_out = end, *sum_out = sum; returns end
#[allow(clippy::too_many_arguments)]
pub fn ee_sub_19a4a0<M: QMem + ?Sized>(
    m: &mut M,
    o: u32,
    to: V4,
    srf: u32,
    from: V4,
    v: V4,
    t_out: u32,
    cost: u32,
    end_out: u32,
    sum_out: u32,
) -> V4 {
    let sp = stack_push(m, 0x50);
    let a = m.r32(o);
    let pos = m.call(0x001a_62e8, &[a], &[]).1;
    let i = vcall(m, srf, 0xc, 0x104, &[0], &[]).0;
    let d = m.call(0x0022_d348, &[i, 0], &[from]).1;
    let r = m.call(0x0019_a400, &[o, sp + 0x40], &[pos, to, from, d]).0;
    let t = m.rf(sp + 0x40);
    let mut res = [0.0; 4];
    if r == 0 {
        let f3 = to[2] - d[2];
        let a = m.r32(o);
        let e = m.r32(a + 0xe8);
        let g = m.rf(e + 0x88);
        let t = (-f3 / g).max(0.0);
        let f6 = pos[2] - from[2] + (g * (t * 0.5) * t + f3 * t);
        if t_out != 0 {
            m.wf(t_out, t);
        }
        if cost != 0 {
            m.wf(cost, f32::from_bits(0x4976_9500) - f6);
        }
    } else {
        let k = 0.15f32.min(t);
        let sum = add4(d, v);
        res = [from[0] + d[0] * k, from[1] + d[1] * k, from[2] + d[2] * k, from[3] + 1.0 + d[3] * k];
        if t_out != 0 {
            m.wf(t_out, t);
        }
        if cost != 0 {
            m.call(0x0011_7120, &[0, 0, sp], &[s(t), s(0.0), s(1.0), pos, to, res, sum]);
            let a = m.rv(sp);
            let l0 = (a[0] * a[0] + a[1] * a[1]).sqrt();
            m.call(0x0011_7120, &[0, 0, sp + 0x10], &[s(t), s(t), s(1.0), pos, to, res, sum]);
            let b = m.rv(sp + 0x10);
            let l1 = (b[0] * b[0] + b[1] * b[1]).sqrt();
            m.wf(cost, ((l0 + l1) * (t * 0.5)).min(f32::from_bits(0x4974_2400)));
        }
        if end_out != 0 {
            m.wv(end_out, res);
        }
        if sum_out != 0 {
            m.wv(sum_out, sum);
        }
    }
    stack_pop(m, 0x50);
    res
}
/// FUN_0019aa58 (cost callback): FUN_00193a68(p[0], p[2], &from, &v); FUN_0019a4a0(p[1], p+0x10, p[2],
/// from, v, cost); returns the cost
pub fn ee_sub_19aa58<M: QMem + ?Sized>(m: &mut M, p: u32, f12: f32) -> f32 {
    let sp = stack_push(m, 0x30);
    let (a, c) = (m.r32(p), m.r32(p + 8));
    // PART r16 ABI: the callback's $f12 reaches FUN_00193a68 untouched (asm 0x19aa74)
    m.call(0x0019_3a68, &[a, c, sp, sp + 0x10], &[s(f12)]);
    let (o, to, f, v) = (m.r32(p + 4), m.rv(p + 0x10), m.rv(sp), m.rv(sp + 0x10));
    ee_sub_19a4a0(m, o, to, c, f, v, 0, sp + 0x20, 0, 0);
    let r = m.rf(sp + 0x20);
    stack_pop(m, 0x30);
    r
}
/// shared body of FUN_0019aab8 / FUN_0019ab38: point of the sub-object at `off` (vslot +0x1c(sub, *p[0],
/// &pt, 0)), then FUN_0019a4a0(p[0], p+0x10, p[1], pt, DAT_002a2dd0, cost)
fn sub_cost<M: QMem + ?Sized>(m: &mut M, p: u32, off: u32) -> f32 {
    let sp = stack_push(m, 0x20);
    let x = m.r32(p + 4);
    let a = m.r32(p);
    let a = m.r32(a);
    vcall(m, x + off, 0x20, 0x1c, &[a, sp, 0], &[]);
    let (o, to, pt) = (m.r32(p), m.rv(p + 0x10), m.rv(sp));
    ee_sub_19a4a0(m, o, to, x, pt, rov(0x002a_2dd0), 0, sp + 0x10, 0, 0);
    let r = m.rf(sp + 0x10);
    stack_pop(m, 0x20);
    r
}
/// FUN_0019aab8: sub-object +0xe0
pub fn ee_sub_19aab8<M: QMem + ?Sized>(m: &mut M, p: u32) -> f32 {
    sub_cost(m, p, 0xe0)
}
/// FUN_0019ab38: sub-object +0x190
pub fn ee_sub_19ab38<M: QMem + ?Sized>(m: &mut M, p: u32) -> f32 {
    sub_cost(m, p, 0x190)
}
/// FUN_0019abb8 (search callback): FUN_0019c300(p[0], p[1], p+0x10, &r, 0, 0, f); returns r
pub fn ee_sub_19abb8<M: QMem + ?Sized>(m: &mut M, f: f32, p: u32) -> f32 {
    let sp = stack_push(m, 0x10);
    let (a, b, q) = (m.r32(p), m.r32(p + 4), m.rv(p + 0x10));
    m.call(0x0019_c300, &[a, b, sp, 0, 0], &[s(f), q]);
    let r = m.rf(sp);
    stack_pop(m, 0x10);
    r
}
/// FUN_0019abf0: FUN_001e6f10(p[1], &m, 0); ctx = {p[0], p[1], FUN_001a6f00(0, p[1], m)}; FUN_0011d9e0(
/// FUN_0019abb8, &ctx, 0, &r, 0.5, 0.25, 0.25, 3.0); returns r
pub fn ee_sub_19abf0<M: QMem + ?Sized>(m: &mut M, p: u32, f12: f32) -> f32 {
    let sp = stack_push(m, 0x40);
    let x = m.r32(p + 4);
    // PART r16 ABI: the callback's $f12 reaches FUN_001e6f10 untouched (asm 0x19ac10)
    m.call(0x001e_6f10, &[x, sp, 0], &[s(f12)]);
    let a = m.r32(p);
    let q = m.rv(sp);
    m.w32(sp + 0x14, x);
    m.w32(sp + 0x10, a);
    let v = m.call(0x001a_6f00, &[0, x], &[q]).1;
    m.wv(sp + 0x20, v);
    m.call(0x0011_d9e0, &[0x0019_abb8, sp + 0x10, 0, sp + 0x30], &[s(0.5), s(0.25), s(0.25), s(3.0)]);
    let r = m.rf(sp + 0x30);
    stack_pop(m, 0x40);
    r
}
/// FUN_0019ac88 (search callback): a = *p[0]; FUN_00185d18(p[1], pos(a), p+0x10, a->+0xe8 +0x80, &s0, &s1,
/// &s2, a+0x1f30, t); min(1e6, (|hermite'(0)|.xyz + |hermite'(t)|.xyz) * t / 2) (FUN_00117120 over pos,
/// p+0x10, s1, s2)
pub fn ee_sub_19ac88<M: QMem + ?Sized>(m: &mut M, t: f32, p: u32) -> f32 {
    let sp = stack_push(m, 0x50);
    let x = m.r32(p + 4);
    let a = m.r32(p);
    let a = m.r32(a);
    let pos = m.call(0x001a_62e8, &[a], &[]).1;
    let e = m.r32(a + 0xe8);
    let (q, w, k) = (m.rv(p + 0x10), m.rv(e + 0x80), m.rf(a + 0x1f30));
    m.call(0x0018_5d18, &[x, sp, sp + 0x10, sp + 0x20], &[s(k), s(t), pos, q, w]);
    let (b, c) = (m.rv(sp + 0x10), m.rv(sp + 0x20));
    let l = |v: V4| (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    m.call(0x0011_7120, &[0, 0, sp + 0x30], &[s(t), s(0.0), s(1.0), pos, q, b, c]);
    let l0 = l(m.rv(sp + 0x30));
    m.call(0x0011_7120, &[0, 0, sp + 0x40], &[s(t), s(t), s(1.0), pos, q, b, c]);
    let l1 = l(m.rv(sp + 0x40));
    stack_pop(m, 0x50);
    ((l0 + l1) * (t * 0.5)).min(f32::from_bits(0x4974_2400))
}

// ---------------------------------------------------------------- run_0015b9f0 (rigid-body helpers)
/// FUN_0015be28: request {FUN_00159118 defaults, +4 = 1, +8 = t1, +0x30 = a, +0x40 = b, +0x50..0xbf = the
/// 0x70 bytes at t0, +0xc0..0xff = the matrix at a3}, extra {0, t2, t3} (memset 0xc); FUN_0015bd20(a2,
/// &req, &extra)
#[allow(clippy::too_many_arguments)]
pub fn ee_sub_15be28<M: QMem + ?Sized>(m: &mut M, a: V4, b: V4, tgt: u32, mt: u32, blk: u32, t1: u32, t2: u32, t3: u32) {
    let sp = stack_push(m, 0x110);
    m.call(0x0015_9118, &[sp], &[]);
    m.call(0x0023_33b4, &[sp + 0x100, 0, 0xc], &[]);
    m.wv(sp + 0x30, a);
    m.wv(sp + 0x40, b);
    for i in 0..7 {
        let q = m.rv(blk + 0x10 * i);
        m.wv(sp + 0x50 + 0x10 * i, q);
    }
    let r: Vec<V4> = (0..4).map(|i| m.rv(mt + 0x10 * i)).collect();
    m.w32(sp + 8, t1);
    m.w32(sp + 0x104, t2);
    m.w32(sp + 0x108, t3);
    m.w32(sp + 4, 1);
    for (i, q) in r.into_iter().enumerate() {
        m.wv(sp + 0xc0 + 0x10 * i as u32, q);
    }
    m.call(0x0015_bd20, &[tgt, sp, sp + 0x100], &[]);
    stack_pop(m, 0x110);
}
/// FUN_0015bf40: out = {src+0x60, normalize(src+0x70 xyz) (DAT_002a2dc0 below 1e-4), src+0x80, src+0x58}
pub fn ee_sub_15bf40<M: QMem + ?Sized>(m: &mut M, out: u32, src: u32) {
    let q = m.rv(src + 0x60);
    m.wv(out, q);
    let v = m.rv(src + 0x70);
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    let n = if l < f32::from_bits(0x38d1_b717) { rov(0x002a_2dc0) } else { sc(v, 1.0 / l) };
    m.wv(out + 0x10, n);
    let x = m.r32(src + 0x80);
    m.w32(out + 0x20, x);
    let x = m.r32(src + 0x58);
    m.w32(out + 0x24, x);
}
/// FUN_0015bfd8: step matrix of a spin record p ({pos xyz, axis +0x10, rate +0x20, scale +0x24}) over
/// dt (DAT_002dded4): A = identity with diag xyz = scale, row 3 = pos; B = identity, rotated by
/// MatrixFromAxisAngle(axis, rate * dt) when rate != 0; out = B + (B * A) * dt (FUN_0011c168, FUN_0011c128)
pub fn ee_sub_15bfd8<M: QMem + ?Sized>(m: &mut M, out: u32, p: u32) -> u32 {
    let sp = stack_push(m, 0x170);
    let id = [rov(0x002a_2d30), rov(0x002a_2d40), rov(0x002a_2d50), rov(0x002a_2d60)];
    let rate = m.rf(p + 0x20);
    let k = m.r32(p + 0x24);
    let (x, y, z) = (m.r32(p), m.r32(p + 4), m.r32(p + 8));
    for i in 0..4u32 {
        m.wv(sp + 0x10 * i, id[i as usize]);
        m.wv(sp + 0x40 + 0x10 * i, id[i as usize]);
    }
    m.w32(sp + 0x28, k);
    m.w32(sp + 0x30, x);
    m.w32(sp + 0x34, y);
    m.w32(sp + 0x38, z);
    m.w32(sp, k);
    m.w32(sp + 0x14, k);
    if rate != 0.0 {
        let ax = m.rv(p + 0x10);
        let dt = m.rf(0x002d_ded4);
        m.call(0x0011_b500, &[sp + 0x80], &[ax, s(rate * dt)]);
        for i in 0..3 {
            let q = m.rv(sp + 0x80 + 0x10 * i);
            m.wv(sp + 0x40 + 0x10 * i, q);
        }
    }
    m.call(0x0011_c168, &[sp + 0x130, sp + 0x40, sp], &[]);
    let dt = m.r32(0x002d_ded4);
    ee_sub_11c128(m, sp + 0xf0, sp + 0x130, dt);
    for i in 0..4 {
        let a = m.rv(sp + 0x40 + 0x10 * i);
        let b = m.rv(sp + 0xf0 + 0x10 * i);
        m.wv(out + 0x10 * i, add4(a, b));
    }
    stack_pop(m, 0x170);
    out
}
/// FUN_0015c198: vslot +0x19c(o, &{a1, a2, f12, f13, a3, t0})
pub fn ee_sub_15c198<M: QMem + ?Sized>(m: &mut M, o: u32, a1: u32, a2: u32, f0: f32, f1: f32, q0: V4, q1: V4) {
    let sp = stack_push(m, 0x30);
    m.w32(sp, a1);
    m.w32(sp + 4, a2);
    m.wf(sp + 8, f0);
    m.wf(sp + 0xc, f1);
    m.wv(sp + 0x10, q0);
    m.wv(sp + 0x20, q1);
    vcall(m, o, 0xc, 0x19c, &[sp], &[]);
    stack_pop(m, 0x30);
}

// ---------------------------------------------------------------- run_0011e630 (triangle / quad geometry)
fn sub4(a: V4, b: V4) -> V4 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2], a[3] - b[3]]
}
/// VU OPMULA / OPMSUB pair: a x b (xyz; the w lane of the destination register is not written, 0 here)
fn cross(a: V4, b: V4) -> V4 {
    [a[1] * b[2] - b[1] * a[2], a[2] * b[0] - b[2] * a[0], a[0] * b[1] - b[0] * a[1], 0.0]
}
/// FUN_0011e630: plane of the quad edges (p0, p1, p2, p3): e1 = p1 - p0, e2 = p2 - p0, e3 = p3 - p2,
/// n = e1 x e3; |n|^2 < 1e-4 -> n = e2, st = {0, 0}; else st = {(e2 x e3).n, (e2 x e1).n} / |n|^2 and n
/// flipped to face e2; n normalized (DAT_002a2dc0 below 1e-4); *dist = n.e2, *nrm = n
#[allow(clippy::too_many_arguments)]
pub fn ee_sub_11e630<M: QMem + ?Sized>(m: &mut M, p0: V4, p1: V4, p2: V4, p3: V4, st: u32, dist: u32, nrm: u32) {
    let e2 = sub4(p2, p0);
    let e1 = sub4(p1, p0);
    let e3 = sub4(p3, p2);
    let mut n = cross(e1, e3);
    let n2 = dot3(n, n);
    if n2 < f32::from_bits(0x38d1_b717) {
        n = e2;
        m.wf(st, 0.0);
        m.wf(st + 4, 0.0);
    } else {
        let a = dot3(cross(e2, e3), n) / n2;
        let b = dot3(cross(e2, e1), n) / n2;
        let d = dot3(e2, n);
        m.wf(st, a);
        m.wf(st + 4, b);
        if d < 0.0 {
            n = sc(n, -1.0);
        }
    }
    let l = dot3(n, n).sqrt();
    let n = if l < f32::from_bits(0x38d1_b717) { rov(0x002a_2dc0) } else { sc(n, 1.0 / l) };
    m.wf(dist, dot3(n, e2));
    m.wv(nrm, n);
}
/// FUN_0011e7c8: FUN_0011e630 then st clamped to [0, 1]; v = p0 (s - 1) - p1 s + p2 (1 - t) + p3 t;
/// *len = |v|, *dir = v / |v| (DAT_002a2da0 below 1e-4)
#[allow(clippy::too_many_arguments)]
pub fn ee_sub_11e7c8<M: QMem + ?Sized>(m: &mut M, p0: V4, p1: V4, p2: V4, p3: V4, st: u32, len: u32, dir: u32) {
    ee_sub_11e630(m, p0, p1, p2, p3, st, len, dir);
    let a = m.rf(st).clamp(0.0, 1.0);
    m.wf(st, a);
    let t = m.rf(st + 4);
    let t = if t < 0.0 { 0.0 } else if 1.0 < t { 1.0 } else { t };
    let (k0, k1, k2) = (a - 1.0, -a, 1.0 - t);
    let mut v = [0.0; 4];
    for i in 0..4 {
        v[i] = p0[i] * k0 + p1[i] * k1 + p2[i] * k2 + p3[i] * t;
    }
    let l = dot3(v, v).sqrt();
    m.wf(st + 4, t);
    let d = if l < f32::from_bits(0x38d1_b717) { rov(0x002a_2da0) } else { sc(v, 1.0 / l) };
    m.wv(dir, d);
    m.wf(len, l);
}
/// FUN_0011e958: yaw = FUN_0011cf60(atan2(v.y, v.x)), pitch = FUN_0011cf60(atan2(v.z, |v.xy|))
pub fn ee_sub_11e958<M: QMem + ?Sized>(m: &mut M, v: V4, yaw: u32, pitch: u32) {
    if yaw != 0 {
        let a = callf(m, 0x0024_58f0, &[], &[s(v[1]), s(v[0])]);
        let a = callf(m, 0x0011_cf60, &[], &[s(a)]);
        m.wf(yaw, a);
    }
    if pitch != 0 {
        let h = (v[0] * v[0] + v[1] * v[1]).sqrt();
        let a = callf(m, 0x0024_58f0, &[], &[s(v[2]), s(h)]);
        let a = callf(m, 0x0011_cf60, &[], &[s(a)]);
        m.wf(pitch, a);
    }
}
/// FUN_0011ea60: as FUN_0011e630 without normalization fallbacks: n = e1 x e3, |n|^2 < 1e-6 -> 0; st =
/// {(e2 x e3).n, (e2 x e1).n} / |n|^2; *nrm = n * (sign(e2.n) / |n|); 1
#[allow(clippy::too_many_arguments)]
pub fn ee_sub_11ea60<M: QMem + ?Sized>(m: &mut M, p0: V4, p1: V4, p2: V4, p3: V4, st: u32, nrm: u32) -> u32 {
    let e2 = sub4(p2, p0);
    let e1 = sub4(p1, p0);
    let e3 = sub4(p3, p2);
    let n = cross(e1, e3);
    let n2 = dot3(n, n);
    if n2 < f32::from_bits(0x3586_37bd) {
        return 0;
    }
    if st != 0 {
        let a = dot3(cross(e2, e3), n);
        let b = dot3(cross(e2, e1), n);
        m.wf(st, a / n2);
        m.wf(st + 4, b / n2);
    }
    let k = if dot3(e2, n) < 0.0 { -1.0 } else { 1.0 };
    if nrm != 0 {
        m.wv(nrm, sc(n, k / n2.sqrt()));
    }
    1
}
/// FUN_0011ebb0: ray (dir, org) vs triangle (v0, v1, v2) (Moller-Trumbore, |det| < 1e-6 misses);
/// hit -> *t = distance, uv = {u, v}; 1
#[allow(clippy::too_many_arguments)]
pub fn ee_sub_11ebb0<M: QMem + ?Sized>(m: &mut M, dir: V4, org: V4, v0: V4, v1: V4, v2: V4, t: u32, uv: u32) -> u32 {
    let e2 = sub4(v2, v0);
    let p = cross(dir, e2);
    let e1 = sub4(v1, v0);
    let det = dot3(e1, p);
    let eps = f32::from_bits(0x3586_37bd);
    if -eps < det && det < eps {
        return 0;
    }
    let tv = sub4(org, v0);
    let inv = 1.0 / det;
    let u = dot3(tv, p) * inv;
    if u < 0.0 || 1.0 < u {
        return 0;
    }
    let q = cross(tv, e1);
    let v = dot3(dir, q) * inv;
    if v < 0.0 || 1.0 < u + v {
        return 0;
    }
    let d = dot3(e2, q) * inv;
    if d < 0.0 {
        return 0;
    }
    if t != 0 {
        m.wf(t, d);
    }
    if uv != 0 {
        m.wf(uv + 4, v);
        m.wf(uv, u);
    }
    1
}

// ---------------------------------------------------------------- class C2d4820 (guard dog grab)
/// FUN_001ba018 (C2d4820_v14): base C2c8940_v14 (FUN_002061d8); FUN_001a64e8(this, 3, 0x2c0a50); p =
/// FUN_0012f938(+0x1020, 0x5db); FUN_001300b8(+0x1020, this, +0x1030, (+0x1034)->+0xa0, &out, 0, *p, 1.0);
/// +0x1040 = out+0x30 - (+0x1034)->+0x30; +0x1060 = 0, +0x106c = 1; FUN_001f35c8(this, 0.25)
pub fn ee_sub_1ba018<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let sp = stack_push(m, 0x40);
    m.call(0x0020_61d8, &[o], &[]);
    m.call(0x001a_64e8, &[o, 3, 0x002c_0a50], &[]);
    let a = m.r32(o + 0x1020);
    let p = m.call(0x0012_f938, &[a, 0x5db], &[]).0;
    let b = m.r32(o + 0x1034);
    let f = m.rf(p);
    let c = m.r32(b + 0xa0);
    let a = m.r32(o + 0x1020);
    let d = m.r32(o + 0x1030);
    m.call(0x0013_00b8, &[a, o, d, c, sp, 0], &[s(f), s(1.0)]);
    let b = m.r32(o + 0x1034);
    let v = sub4(m.rv(sp + 0x30), m.rv(b + 0x30));
    m.w32(o + 0x1060, 0);
    m.wv(o + 0x1040, v);
    m.w32(o + 0x106c, 1);
    m.call(0x001f_35c8, &[o], &[s(0.25)]);
    stack_pop(m, 0x40);
}
/// FUN_001ba408: grab state change to `k` (+0x1018): leaving 1 -> AsegaHost_retract(+0x1024) (FUN_00130b18),
/// vslot +0x14 (Release), +0x1024 = 0, FUN_0012e460(&+0x102c), +0x102c / +0x1028 = 0, with skill 2 ->
/// Guard_k24_comp_b0(this, 0x9e)+0x38 = DAT_002e1e40; FUN_00204d48(this, 1.0); +0x101c = clock; entering
/// 1 -> JtAnimator_request(+0x1020, this, 1, &+0x1024, 0.0, 1.0) (FUN_0012f9b8), vslot +0xc (AddRef);
/// +0x102c = FUN_0012dc18(this, +0x1034, 5, 8, 3, DAT_002a2dd0); with skill 2 -> comp+0x38 = +0x1028,
/// FUN_00204d48(this, 4.0)
pub fn ee_sub_1ba408<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32) {
    let cur = m.r32(o + 0x1018);
    if k == cur {
        return;
    }
    if cur == 1 {
        let a = m.r32(o + 0x1024);
        m.call(0x0013_0b18, &[a], &[]);
        let a = m.r32(o + 0x1024);
        vcall(m, a, 0xc, 0x14, &[], &[]);
        m.w32(o + 0x1024, 0);
        m.call(0x0012_e460, &[o + 0x102c], &[]);
        m.w32(o + 0x102c, 0);
        m.w32(o + 0x1028, 0);
        if m.call(0x0020_9a68, &[o, 2], &[]).0 != 0 {
            let c = m.call(0x0020_5458, &[o, 0x9e], &[]).0;
            let x = m.r32(0x002e_1e40);
            m.w32(c + 0x38, x);
        }
        m.call(0x0020_4d48, &[o], &[s(1.0)]);
    }
    m.w32(o + 0x1018, k);
    let t = m.r32(CLOCK);
    m.w32(o + 0x101c, t);
    if k != 1 {
        return;
    }
    let a = m.r32(o + 0x1020);
    m.call(0x0012_f9b8, &[a, o, 1, o + 0x1024], &[s(0.0), s(1.0)]);
    let a = m.r32(o + 0x1024);
    vcall(m, a, 0xc, 0xc, &[], &[]);
    let b = m.r32(o + 0x1034);
    let r = m.call(0x0012_dc18, &[o, b, 5, 8, 3], &[rov(0x002a_2dd0)]).0;
    m.w32(o + 0x102c, r);
    if m.call(0x0020_9a68, &[o, 2], &[]).0 != 0 {
        let c = m.call(0x0020_5458, &[o, 0x9e], &[]).0;
        let x = m.r32(o + 0x1028);
        m.w32(c + 0x38, x);
        m.call(0x0020_4d48, &[o], &[s(4.0)]);
    }
}
/// FUN_001ba128 (C2d4820_v15): base C2c8940_v15 (FUN_00207350); state 0 with skill 0x25: zone = +0x1010
/// (skill 9) / +0x1014 (skill 2) / none; +0x1028 = the nearest (squared distance <= DAT_002a7620) other
/// guard of the list DAT_003e1444 (next +0xfe0) without skill 7 inside the zone (FUN_00227c90); none and
/// no skill 2 -> query the world box (this + DAT_002c0a90 / DAT_002c0aa0 around FUN_001a6f00(this, 0,
/// DAT_002c0a80); FUN_001ae478 / FUN_0020faa0(world, hi, lo, 0x1ba100, 0, &n, &list) / FUN_001ae4a0(.., 2))
/// for the first hit (+4) inside the zone; found -> FUN_001ba408(this, 1). State 1: grab done
/// (FUN_00131968(+0x1024)) -> FUN_001ba408(this, 0); else (+0x102c)->+0x20 = FUN_0011bff0(FUN_0011ba08(
/// +0x1040, FUN_0022fc48(0, (+0x1034)->+0xa0, target position) - (+0x1034)->+0x30))
pub fn ee_sub_1ba128<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0020_7350, &[o], &[]);
    match m.r32(o + 0x1018) {
        0 => {
            if m.call(0x0020_9a68, &[o, 0x25], &[]).0 == 0 {
                return;
            }
            let zone = if m.call(0x0020_9a68, &[o, 9], &[]).0 != 0 {
                m.r32(o + 0x1010)
            } else if m.call(0x0020_9a68, &[o, 2], &[]).0 != 0 {
                m.r32(o + 0x1014)
            } else {
                return;
            };
            m.w32(o + 0x1028, 0);
            let mut best = crate::engine::classes::q::q6::rof(0x002a_7620);
            let mut x = m.r32(0x003e_1444);
            while x != 0 {
                if x != o && m.call(0x0020_9a68, &[x, 7], &[]).0 == 0 {
                    let a = m.call(0x001a_62e8, &[o], &[]).1;
                    let b = m.call(0x001a_62e8, &[x], &[]).1;
                    let d = sub4(a, b);
                    let d2 = dot3(d, d);
                    if d2 <= best {
                        let p = m.call(0x001a_62e8, &[x], &[]).1;
                        if m.call(0x0022_7c90, &[zone], &[p]).0 != 0 {
                            m.w32(o + 0x1028, x);
                            best = d2;
                        }
                    }
                }
                x = m.r32(x + 0xfe0);
            }
            if m.r32(o + 0x1028) == 0 {
                if m.call(0x0020_9a68, &[o, 2], &[]).0 != 0 {
                    return;
                }
                let sp = stack_push(m, 0x80);
                let c = m.call(0x001a_6f00, &[o, 0], &[rov(0x002c_0a80)]).1;
                let hi = add4(c, rov(0x002c_0a90));
                let lo = add4(c, rov(0x002c_0aa0));
                m.wv(sp + 0x70, lo);
                m.wv(sp + 0x50, hi);
                m.call(0x001a_e478, &[sp], &[]);
                m.call(0x0020_faa0, &[WORLD, 0x001b_a100, 0, sp + 0x40, sp + 0x44], &[hi, lo]);
                let mut i = 0u32;
                while (i as i32) < m.r32(sp + 0x40) as i32 {
                    let l = m.r32(sp + 0x44);
                    let e = m.r32(l + 4 * i);
                    let a = m.r32(e + 4);
                    let p = m.call(0x001a_62e8, &[a], &[]).1;
                    i += 1;
                    if m.call(0x0022_7c90, &[zone], &[p]).0 != 0 {
                        let a = m.r32(e + 4);
                        m.w32(o + 0x1028, a);
                        break;
                    }
                }
                m.call(0x001a_e4a0, &[sp, 2], &[]);
                stack_pop(m, 0x80);
                if m.r32(o + 0x1028) == 0 {
                    return;
                }
            }
            ee_sub_1ba408(m, o, 1);
        }
        1 => {
            let a = m.r32(o + 0x1024);
            if m.call(0x0013_1968, &[a], &[]).0 != 0 {
                ee_sub_1ba408(m, o, 0);
                return;
            }
            let sp = stack_push(m, 0x20);
            let b = m.r32(o + 0x1034);
            let t = m.r32(o + 0x1028);
            let p = m.call(0x001a_62e8, &[t], &[]).1;
            let k = m.r32(b + 0xa0);
            let v = m.call(0x0022_fc48, &[0, k], &[p]).1;
            let b = m.r32(o + 0x1034);
            let d = sub4(v, m.rv(b + 0x30));
            let q = m.rv(o + 0x1040);
            m.call(0x0011_ba08, &[sp], &[q, d]);
            let r = m.r32(o + 0x102c);
            let v = m.call(0x0011_bff0, &[sp], &[]).1;
            m.wv(r + 0x20, v);
            stack_pop(m, 0x20);
        }
        _ => {}
    }
}

// ---------------------------------------------------------------- class C2cd748 (slots 81 / 37)
/// FUN_00205008 (C2cd748_v81, axis contacts): chain = the contact list of the first object up from
/// FUN_001a5bf0(a2) (via +0x10) with a world-list (+0xec), searched by its +0x60 in this world-list's
/// nodes (+0x64 chain {.., next +4, store +8, key +0xc}; s4 = (+0x60)->+0xec ->+0x34), else this
/// world-list's chain +0x68; frame = FUN_0011b830(dir t0, DAT_002a2dc0); d = frame rows . (at - from);
/// per axis i (flag bit i forces kind 0xb, flag bit i+3 adds kind 0xa (axis 2: 0xe) when d[i] < 1e-4):
/// FUN_001a7c18(world, +0x258, s4, at, row i, kind, 0, 0, chain, d[i]); *store = *chain
pub fn ee_sub_205008<M: QMem + ?Sized>(m: &mut M, o: u32, fl: u32, a2: u32, from: V4, dir: V4, at: V4) {
    let sp = stack_push(m, 0x30);
    let mut v = m.call(0x001a_5bf0, &[a2], &[]).0;
    let mut wl = 0u32;
    while v != 0 {
        wl = m.r32(v + 0xec);
        if wl != 0 {
            break;
        }
        v = m.r32(v + 0x10);
    }
    let (s4, s2, s7);
    if v == 0 {
        let e = m.r32(o + 0xec);
        s4 = 0;
        s7 = e + 0x68;
        s2 = s7;
    } else {
        let e = m.r32(o + 0xec);
        let key = m.r32(wl + 0x60);
        let mut n = m.r32(e + 0x64);
        while n != 0 && m.r32(n + 0xc) != key {
            n = m.r32(n + 4);
        }
        let w = m.r32(key + 0xec);
        s2 = n;
        s7 = m.r32(n + 8);
        s4 = m.r32(w + 0x34);
    }
    m.call(0x0011_b830, &[sp], &[dir, rov(0x002a_2dc0)]);
    let dl = sub4(at, from);
    let rows = [m.rv(sp), m.rv(sp + 0x10), m.rv(sp + 0x20)];
    let d = [dot3(rows[0], dl), dot3(rows[1], dl), dot3(rows[2], dl)];
    let eps = f32::from_bits(0x38d1_b717);
    let lim = m.r32(o + 0x258);
    for i in 0..3 {
        let force = fl & (1 << i) != 0;
        if force || (fl & (8 << i) != 0 && d[i] < eps) {
            let k = if force { 0xb } else if i == 2 { 0xe } else { 0xa };
            m.call(0x001a_7c18, &[WORLD, lim, s4, k, 0, 0, s2], &[s(d[i]), at, rows[i]]);
        }
    }
    let h = m.r32(s2);
    m.w32(s7, h);
    stack_pop(m, 0x30);
}
/// FUN_002052e0 (C2cd748_v37, attach to x): x without skill 3 (FUN_00132b10) -> base C2caf88_v37
/// (FUN_001f49c8); else M = vslot +0xfc(x): vslot +0xec(this, M * -(+0x240) (xyz, w row 3)), vslot +0xf4(this,
/// M); skill 0x34 -> +0x1060 = M row 3 + w1 + DAT_002a2dc0 * +0x1048; vslot +0x11c / +0x124(this,
/// DAT_002a2dd0); +0x208 = 0
thread_local! {
    /// CRAWL R107: (object, target) guest addresses of this thread's slot-37 MoveToObject calls, for the port's
    /// run-time owners of the PO bodies (`player::p22::apply_po_placements`, drained by `game::app::tick_scripts`)
    pub static PO_MOVES: std::cell::RefCell<Vec<(u32, u32)>> = const { std::cell::RefCell::new(Vec::new()) };
}

pub fn ee_sub_2052e0<M: QMem + ?Sized>(m: &mut M, o: u32, x: u32) {
    if x == 0 {
        return;
    }
    PO_MOVES.with(|v| {
        let mut v = v.borrow_mut();
        if v.len() < 64 {
            v.push((o, x));
        }
    });
    if m.call(0x0013_2b10, &[x, 3], &[]).0 == 0 {
        m.call(0x001f_49c8, &[o, x], &[]);
        return;
    }
    let sp = stack_push(m, 0x40);
    let p = vcall(m, x, 0xc, 0xfc, &[], &[]).0;
    let r: Vec<V4> = (0..4).map(|i| m.rv(p + 0x10 * i)).collect();
    for (i, q) in r.iter().enumerate() {
        m.wv(sp + 0x10 * i as u32, *q);
    }
    let q = sc(m.rv(o + 0x240), -1.0);
    let mut v = r[3];
    for k in 0..3 {
        v[k] = r[0][k] * q[0] + r[1][k] * q[1] + r[2][k] * q[2] + r[3][k];
    }
    vcall(m, o, 0xc, 0xec, &[], &[v]);
    vcall(m, o, 0xc, 0xf4, &[sp], &[]);
    if m.call(0x0013_2b10, &[o, 0x34], &[]).0 != 0 {
        let k = m.rf(o + 0x1048);
        let d = rov(0x002a_2dc0);
        let mut w = r[3];
        w[3] += 1.0;
        m.wv(o + 0x1060, [w[0] + d[0] * k, w[1] + d[1] * k, w[2] + d[2] * k, w[3] + d[3] * k]);
    }
    vcall(m, o, 0xc, 0x11c, &[], &[rov(0x002a_2dd0)]);
    vcall(m, o, 0xc, 0x124, &[], &[rov(0x002a_2dd0)]);
    m.w32(o + 0x208, 0);
    stack_pop(m, 0x40);
}

// ---------------------------------------------------------------- run_001d2240 (chaser AI: target pick / path / facing)
fn dxy(a: V4, b: V4) -> f32 {
    let (x, y) = (a[0] - b[0], a[1] - b[1]);
    (x * x + y * y).sqrt()
}
/// FUN_001d2240: first of the +0x4f8 bots (+0x4fc) that is active (FUN_001d4e08) and not in mode 4 (+0x80)
pub fn ee_sub_1d2240<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let mut i = 0u32;
    while (i as i32) < m.r32(o + 0x4f8) as i32 {
        let e = m.r32(o + 0x4fc + 4 * i);
        if m.call(0x001d_4e08, &[e], &[]).0 != 0 && m.r32(e + 0x80) != 4 {
            return e;
        }
        i += 1;
    }
    0
}
/// FUN_001d22d0: nearest (xy) of the up to 16 world objects of class 0x105 / 0x6c (FUN_0015e9c8) with cost
/// d (+ c0 + d * (c4 + d * c8) for the current target +0x5b8, DAT_002c20c0..) + this[0x57c + 0xc * its +0x158],
/// below DAT_002a7810
pub fn ee_sub_1d22d0<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let sp = stack_push(m, 0x40);
    let pos = m.call(0x001a_62e8, &[o], &[]).1;
    let n = m.call(0x0015_e9c8, &[WORLD, 0x105, 0x6c, 0, 0x10, sp], &[]).0;
    let mut lo = crate::engine::classes::q::q6::rof(0x002a_7810);
    let mut best = 0;
    for i in 0..n as i32 {
        let e = m.r32(sp + 4 * i as u32);
        let p = m.call(0x001a_62e8, &[e], &[]).1;
        let mut f = dxy(p, pos);
        if e == m.r32(o + 0x5b8) {
            let (a, b, c) = (m.rf(0x002c_20c0), m.rf(0x002c_20c4), m.rf(0x002c_20c8));
            f = a + f * (b + f * c);
        }
        let k = m.r32(e + 0x158);
        f += m.rf(o.wrapping_add(k.wrapping_mul(0xc)) + 0x57c);
        if f < lo {
            lo = f;
            best = e;
        }
    }
    stack_pop(m, 0x40);
    best
}
/// FUN_001d2420: +0x5b0 = 0; path (+0x5ac) points (+0xf4, count +0xe8) z = own z
pub fn ee_sub_1d2420<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.w32(o + 0x5b0, 0);
    let z = m.call(0x001a_62e8, &[o], &[]).1[2];
    let mut i = 0u32;
    loop {
        let t = m.r32(o + 0x5ac);
        if (i as i32) >= m.r32(t + 0xe8) as i32 {
            break;
        }
        let p = m.r32(t + 0xf4);
        m.wf(p + i * 0x10 + 8, z);
        i += 1;
    }
}
/// FUN_001d2490: closest path point / tangent (track_1e7380(+0x5ac, pos, .., +0x5b0)); look ahead
/// k = clamp(d0 + d * (d4 + d * d8), e0, e4) (DAT_002c20d0..e4, d = xy distance to the point) along the
/// tangent; +0x544 = heading (atan2) to that point (the tangent when closer than 1e-4); +0x550 = +0x51c * 0.75
pub fn ee_sub_1d2490<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let sp = stack_push(m, 0x30);
    let pos = m.call(0x001a_62e8, &[o], &[]).1;
    let (t, f) = (m.r32(o + 0x5ac), m.rf(o + 0x5b0));
    m.call(0x001e_7380, &[t, 0, 0, sp, sp + 0x10, 0, o + 0x5b0], &[pos, s(f)]);
    let (pt, dir) = (m.rv(sp), m.rv(sp + 0x10));
    let d = dxy(pos, pt);
    let mut k = m.rf(0x002c_20d0) + d * (m.rf(0x002c_20d4) + d * m.rf(0x002c_20d8));
    let (lo, hi) = (m.rf(0x002c_20e0), m.rf(0x002c_20e4));
    if k < lo {
        k = lo;
    } else if hi < k {
        k = hi;
    }
    m.wv(sp + 0x20, s(k));
    let v = [pt[0] + dir[0] * k - pos[0], pt[1] + dir[1] * k - pos[1], pt[2] + dir[2] * k - pos[2]];
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    let (x, y) = if l < f32::from_bits(0x38d1_b717) { (dir[0], dir[1]) } else { (v[0] / l, v[1] / l) };
    let a = callf(m, 0x0024_58f0, &[], &[s(y), s(x)]);
    m.wf(o + 0x544, a);
    let r = m.rf(o + 0x51c);
    m.wf(o + 0x550, r * 0.75);
    stack_pop(m, 0x30);
}
/// try to bark at bot `e`: e+0x30 set -> only when its facing (atan2 of FUN_001a6318(e+0x3c) row 0) is
/// within `ka` of +0x54c (FUN_0011cf60) and the z of FUN_0022ca18(e+0x30) within `kz` of (e+0x34)->+0x28;
/// FUN_001f0660(e+0x20, 0x68), +0x5b4 = clock + FUN_0011d448(lo, hi); `strict` = skip on equality
#[allow(clippy::too_many_arguments)]
fn bark<M: QMem + ?Sized>(m: &mut M, o: u32, e: u32, ka: f32, kz: f32, strict: bool, lo: f32, hi: f32) {
    let sp = stack_push(m, 0x30);
    if m.r32(e + 0x30) != 0 {
        let x = m.r32(e + 0x3c);
        let mt = m.call(0x001a_6318, &[x], &[]).0;
        for k in 0..3 {
            let q = m.rv(mt + 0x10 * k);
            m.wv(sp + 0x10 * k, q);
        }
        let (y0, x0) = (m.rf(sp + 4), m.rf(sp));
        let a = callf(m, 0x0024_58f0, &[], &[s(y0), s(x0)]);
        let h = m.r32(e + 0x30);
        let z = m.call(0x0022_ca18, &[h, h], &[]).1[2];
        let c = m.rf(o + 0x54c);
        let da = callf(m, 0x0011_cf60, &[], &[s(a - c)]).abs();
        let p = m.r32(e + 0x34);
        let dz = (z - m.rf(p + 0x28)).abs();
        let skip = if strict { ka <= da || kz <= dz } else { ka < da || kz < dz };
        if skip {
            stack_pop(m, 0x30);
            return;
        }
    }
    let u = m.r32(e + 0x20);
    m.call(0x001f_0660, &[u, 0x68], &[]);
    let c = m.rf(CLOCK);
    let r = callf(m, 0x0011_d448, &[], &[s(lo), s(hi)]);
    m.wf(o + 0x5b4, c + r);
    stack_pop(m, 0x30);
}
/// FUN_001d2610 (chaser update): pick a bot (+0x5bc, FUN_001d2240) or else a target (+0x5b8,
/// FUN_001d22d0); state machine +0x188 (FUN_001d4b28 transitions, re-run until stable: 0 idle -> 1 when the
/// bot is in range +0x88, else 2 with a target / 3; 1 -> 0 when out of range +0x90; 2 -> 0 without target;
/// 3 -> 0 with either); state 1: face the target (+0x54c), bark at the bot; 3: flee; 0: circle the target
/// at the bot's radius +0x7c (acos turn, bot mode +0x80 2 / 3 = +-angle about DAT_002a2dc0), straight path
/// (FUN_001c0960) or chase (C2d1080_chase); 2: chase the target; then barks at any active bot in view
pub fn ee_sub_1d2610<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let sp = stack_push(m, 0x2c0);
    let pos = m.call(0x001a_62e8, &[o], &[]).1;
    let x = m.r32(o + 0x5a4);
    let tg = m.call(0x001a_62e8, &[x], &[]).1;
    let d = [tg[0] - pos[0], tg[1] - pos[1], tg[2] - pos[2], tg[3] - pos[3]];
    let dist = dxy(tg, pos);
    let bot = ee_sub_1d2240(m, o);
    m.w32(o + 0x5bc, bot);
    if bot == 0 {
        let t = ee_sub_1d22d0(m, o);
        m.w32(o + 0x5b8, t);
    } else {
        m.w32(o + 0x5b8, 0);
    }
    let st = loop {
        let st = m.r32(o + 0x188);
        let (b, t) = (m.r32(o + 0x5bc), m.r32(o + 0x5b8));
        let to = match st {
            1 if b != 0 && m.call(0x0011_d910, &[b + 0x90], &[s(dist)]).0 != 0 => None,
            1 => Some(0),
            0 if b == 0 => Some(if t == 0 { 3 } else { 2 }),
            0 if m.call(0x0011_d910, &[b + 0x88], &[s(dist)]).0 != 0 => Some(1),
            2 if t == 0 => Some(0),
            3 if b != 0 || t != 0 => Some(0),
            _ => None,
        };
        if let Some(k) = to {
            m.call(0x001d_4b28, &[o, k], &[]);
        }
        if m.r32(o + 0x188) == st {
            break st;
        }
    };
    let mut path = false;
    let mut chase = None;
    match st {
        1 => {
            m.w32(o + 0x550, 0);
            let h = m.r32(o + 0x540);
            m.w32(o + 0x544, h);
            let a = callf(m, 0x0024_58f0, &[], &[s(d[1]), s(d[0])]);
            m.wf(o + 0x54c, a);
            let c = m.rf(CLOCK);
            if m.rf(o + 0x5b4) < c {
                let e = ee_sub_1d2240(m, o);
                if e != 0 {
                    let (ka, kz) = (m.rf(0x002c_20ec), m.rf(0x002c_20f0));
                    bark(m, o, e, ka, kz, true, 1.0, 1.5);
                }
            }
        }
        3 => {
            let a = callf(m, 0x0024_58f0, &[], &[s(-d[1]), s(-d[0])]);
            m.wf(o + 0x544, a);
            let h = m.r32(o + 0x540);
            m.w32(o + 0x54c, h);
            let r = m.rf(o + 0x51c);
            m.wf(o + 0x550, r * f32::from_bits(0x3f59_999a));
        }
        2 => {
            let t = m.r32(o + 0x5b8);
            let p = m.call(0x001a_62e8, &[t], &[]).1;
            let tr = m.r32(o + 0x5ac);
            let n = m.r32(tr + 0xe8) as i32;
            path = true;
            let pl = m.r32(tr + 0xf4).wrapping_add((n as u32) << 4).wrapping_sub(0x10);
            if n < 2 || 50.0 < dxy(m.rv(pl), p) {
                chase = Some(p);
            }
        }
        0 => {
            path = true;
            let b = m.r32(o + 0x5bc);
            let r = m.rf(b + 0x7c);
            let kr = r * m.rf(0x002c_20e8);
            let ang = callf(m, 0x0024_56f0, &[], &[s(r / dist.max(kr))]);
            let v = [pos[0] - tg[0], pos[1] - tg[1], pos[2] - tg[2], pos[3] - tg[3]];
            let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
            let u = if l < f32::from_bits(0x38d1_b717) {
                rov(0x002a_2da0)
            } else {
                let k = 1.0 / l;
                [v[0] * k, v[1] * k, v[2] * k, v[3] * k]
            };
            let off = [u[0] * r, u[1] * r, u[2] * r, u[3] * r];
            let dest = sp + 0x1f0;
            match m.r32(b + 0x80) {
                0 | 1 => m.wv(dest, tg),
                k @ (2 | 3) => {
                    let mt = if k == 2 { sp + 0x10 } else { sp };
                    let a = if k == 2 { ang } else { -ang };
                    m.call(0x0011_b500, &[mt], &[rov(0x002a_2dc0), s(a)]);
                    let (r0, r1, r2) = (m.rv(mt), m.rv(mt + 0x10), m.rv(mt + 0x20));
                    let mut q = [0.0; 4];
                    for i in 0..3 {
                        q[i] = r0[i] * off[0] + r1[i] * off[1] + r2[i] * off[2] + tg[i];
                    }
                    q[3] = r0[3] + tg[3];
                    m.wv(dest, q);
                }
                _ => {}
            }
            let dv = m.rv(dest);
            let a = m.r32(o + 0x5a8);
            let ok = m.call(0x001c_0960, &[a, 4, sp + 0x40, 2], &[pos, dv]).0 == 1;
            let tr = m.r32(o + 0x5ac);
            if ok && m.rf(sp + 0x80) == 0.0 && m.rf(sp + 0x84) == 1.0 {
                let pts = m.r32(tr + 0xf4);
                m.wv(pts, pos);
                let pts = m.r32(tr + 0xf4);
                m.wv(pts + 0x10, dv);
                m.w32(tr + 0xe8, 2);
                m.call(0x001e_6180, &[tr], &[]);
                ee_sub_1d2420(m, o);
            } else {
                let n = m.r32(tr + 0xe8) as i32;
                let pl = m.r32(tr + 0xf4).wrapping_add((n as u32) << 4).wrapping_sub(0x10);
                if n < 2 || 50.0 < dxy(m.rv(pl), tg) {
                    chase = Some(tg);
                }
            }
        }
        _ => {}
    }
    if let Some(p) = chase {
        let a = m.r32(o + 0x5a8);
        m.call(0x001c_02d0, &[a], &[pos, p]);
        ee_sub_1d2420(m, o);
    }
    if path {
        ee_sub_1d2490(m, o);
        let h = m.r32(o + 0x540);
        m.w32(o + 0x54c, h);
    }
    if m.rf(o + 0x5b4) < m.rf(CLOCK) {
        let mut i = 0u32;
        while (i as i32) < m.r32(o + 0x4f8) as i32 {
            let e = m.r32(o + 0x4fc + 4 * i);
            i += 1;
            if m.call(0x001d_4e08, &[e], &[]).0 == 0 {
                continue;
            }
            let z = m.r32(e + 0x84);
            if z != 0 && m.call(0x0022_7c90, &[z], &[tg]).0 & 0xff == 0 {
                continue;
            }
            if 0.0 < m.rf(e + 0x8c) && m.r32(o + 0x188) != 1 {
                continue;
            }
            let (ka, kz, lo, hi) = (m.rf(0x002c_20f4), m.rf(0x002c_20f8), m.rf(e + 0x98), m.rf(e + 0x9c));
            bark(m, o, e, ka, kz, false, lo, hi);
        }
    }
    stack_pop(m, 0x2c0);
}

// ---------------------------------------------------------------- run_00193c98 (rail / ledge helpers)
/// FUN_00193c98: no hit (r+4 == 0) -> FUN_0012da30(out, DAT_002a2dd0); else rail param t = +0x4e0 +-
/// r+0x10 (negated with +0x4e4), wrapped (FUN_0011d898, closed rail +0xe4) or clamped to [FUN_001e6620,
/// FUN_001e6648] of the rail +0x4d4; point FUN_001e6f10 -> world (FUN_001a6f00); FUN_0014c748(r+0x20,
/// r+0x10, point, 1, &hit, r+8) > 0 -> FUN_0012da30(out, hit.+0x10 - r+0x10)
pub fn ee_sub_193c98<M: QMem + ?Sized>(m: &mut M, o: u32, r: u32, out: u32) {
    if m.r32(r + 4) == 0 {
        m.call(0x0012_da30, &[out], &[rov(0x002a_2dd0)]);
        return;
    }
    let sp = stack_push(m, 0x70);
    let tr = m.r32(o + 0x4d4);
    let mut d = m.rf(r + 0x10);
    if m.r32(o + 0x4e4) != 0 {
        d = -d;
    }
    let lo = callf(m, 0x001e_6620, &[tr], &[]);
    let hi = callf(m, 0x001e_6648, &[tr], &[]);
    let v = m.rf(o + 0x4e0) + d;
    let t = if m.r32(tr + 0xe4) != 0 {
        callf(m, 0x0011_d898, &[], &[s(v), s(hi)])
    } else if v < lo {
        lo
    } else if hi < v {
        hi
    } else {
        v
    };
    m.call(0x001e_6f10, &[tr, sp, 0], &[s(t)]);
    let p = m.rv(sp);
    let w = m.call(0x001a_6f00, &[0, o], &[p]).1;
    m.wv(sp, w);
    let (a, b, f) = (m.rv(r + 0x20), m.rv(r + 0x10), m.rf(r + 8));
    if (m.call(0x0014_c748, &[1, sp + 0x10], &[a, b, w, s(f)]).0 as i32) > 0 {
        let h = m.rv(sp + 0x20);
        m.call(0x0012_da30, &[out], &[[h[0] - b[0], h[1] - b[1], h[2] - b[2], h[3] - b[3]]]);
    }
    stack_pop(m, 0x70);
}
/// FUN_00193de8 (rail mount): kind (p+8) 4 -> vslot +0x294(p[0], 0x23, p[5], pos(p[5]), DAT_002a2db0,
/// *(p[0]+0x218)); kind 3 -> nearest rail point / tangent (FUN_001e6ed8(p[5], *(p[0]+0x218), .., p+0x1c,
/// p+0x20)); open rail (+0x1bc / +0xe4 clear): velocity (vslot +0x10c) along the tangent < -2 or facing
/// (FUN_001a6318 * +0x190) against it < 2 with p+0x20 within 50 of the start -> clamp to start + 50, code
/// 0x2a; then the same towards the end (end - 50, tangent negated); vslot +0x294(p[0], code (0x22),
/// p[5], point, tangent, *(p[0]+0x218))
pub fn ee_sub_193de8<M: QMem + ?Sized>(m: &mut M, p: u32) {
    let k = m.r32(p + 8);
    if k == 4 {
        let x = m.r32(p + 0x14);
        let pos = m.call(0x001a_62e8, &[x], &[]).1;
        let a = m.r32(p);
        let q = m.r32(a + 0x218);
        let q = m.rv(q);
        let x = m.r32(p + 0x14);
        vcall(m, a, 0xc, 0x294, &[0x23, x], &[pos, rov(0x002a_2db0), q]);
        return;
    }
    if k != 3 {
        return;
    }
    let sp = stack_push(m, 0x40);
    let a = m.r32(p);
    let tr = m.r32(p + 0x14);
    let q = m.r32(a + 0x218);
    let q = m.rv(q);
    let f = m.rf(p + 0x1c);
    m.call(0x001e_6ed8, &[tr, 0, sp, sp + 0x10, p + 0x1c, p + 0x20], &[q, s(f)]);
    let mut code = 0x22;
    if m.r32(tr + 0x1bc) == 0 && m.r32(tr + 0xe4) == 0 {
        let a = m.r32(p);
        let v = vcall(m, a, 0xc, 0x10c, &[], &[]).1;
        let tg = m.rv(sp + 0x10);
        let f22 = dot3(v, tg);
        m.wv(sp + 0x20, v);
        let a = m.r32(p);
        let mt = m.call(0x001a_6318, &[a], &[]).0;
        let (r0, r1, r2) = (m.rv(mt), m.rv(mt + 0x10), m.rv(mt + 0x20));
        let a = m.r32(p);
        let x = m.rv(a + 0x190);
        let mut w = [0.0; 4];
        for i in 0..3 {
            w[i] = r0[i] * x[0] + r1[i] * x[1] + r2[i] * x[2];
        }
        let f20 = -dot3(w, tg);
        let mut done = false;
        if f22 < -2.0 || f20 < 2.0 {
            let lo = callf(m, 0x001e_6620, &[tr], &[]);
            if m.rf(p + 0x20) < lo + 50.0 {
                code = 0x2a;
                let lo = callf(m, 0x001e_6620, &[tr], &[]);
                m.call(0x001e_6f10, &[tr, sp, 0], &[s(lo + 50.0)]);
                done = true;
            }
        }
        if !done && (2.0 < f22 || -2.0 < f20) {
            let hi = callf(m, 0x001e_6648, &[tr], &[]);
            if hi - 50.0 < m.rf(p + 0x20) {
                code = 0x2a;
                let hi = callf(m, 0x001e_6648, &[tr], &[]);
                m.call(0x001e_6f10, &[tr, sp, 0], &[s(hi - 50.0)]);
                let t = m.rv(sp + 0x10);
                m.wv(sp + 0x30, s(-1.0));
                m.wv(sp + 0x10, [-t[0], -t[1], -t[2], -t[3]]);
            }
        }
    }
    let a = m.r32(p);
    let q = m.r32(a + 0x218);
    let (pt, tg, q) = (m.rv(sp), m.rv(sp + 0x10), m.rv(q));
    vcall(m, a, 0xc, 0x294, &[code, tr], &[pt, tg, q]);
    stack_pop(m, 0x40);
}
/// FUN_00194118: reset the move state (+0x68): +0x164 5, +0x114 &DAT_002bf670, +0x168 1, +0x128 3,
/// +0x110 1, +0x140 / +0x100 / +0xfc / +0x108 / +0x10c / +0x120 / +0x124 / +0x16c / +0x130 = 0
pub fn ee_sub_194118<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let x = m.r32(o + 0x68);
    for (off, v) in [
        (0x164, 5u32),
        (0x114, 0x002b_f670),
        (0x168, 1),
        (0x128, 3),
        (0x140, 0),
        (0x100, 0),
        (0xfc, 0),
        (0x108, 0),
        (0x10c, 0),
        (0x110, 1),
        (0x120, 0),
        (0x124, 0),
        (0x16c, 0),
        (0x130, 0),
    ] {
        m.w32(x + off, v);
    }
}
/// FUN_001942f0 (attack aim): kind (p+8) 3 -> (p[5]+0x19c clear) FUN_0019fd30(p[0]+0x9e0, p[6], p+0x20);
/// 4 -> (p[5]+0xec clear) FUN_0019fd30(.., 0.0); 0 / 0xd -> by p[5] state (FUN_00132b10): 0xc as kind 3,
/// 0x26 as kind 4, else angle to p[5] (atan2) * (0x22: (+0xec clear) +0x10c, else +0x184); other kinds ->
/// p[0]+0x208 target with +0x88 > 0.7, flag 0x20000000 of (+0x4c)->+0x148 clear and p[0]+0x108c clear:
/// FUN_0019feb0(p[0]+0x9e0, target+0x40, target+0x60)
pub fn ee_sub_1942f0<M: QMem + ?Sized>(m: &mut M, p: u32) {
    let k = m.r32(p + 8);
    let b = m.r32(p + 0x14);
    let f: Option<f32> = match k {
        3 => (m.r32(b + 0x19c) == 0).then(|| m.rf(p + 0x20)),
        4 => (m.r32(b + 0xec) == 0).then_some(0.0),
        0 | 0xd => {
            if m.call(0x0013_2b10, &[b, 0xc], &[]).0 != 0 {
                let b = m.r32(p + 0x14);
                (m.r32(b + 0x19c) == 0).then(|| m.rf(p + 0x20))
            } else if {
                let b = m.r32(p + 0x14);
                m.call(0x0013_2b10, &[b, 0x26], &[]).0 != 0
            } {
                let b = m.r32(p + 0x14);
                (m.r32(b + 0xec) == 0).then_some(0.0)
            } else {
                let b = m.r32(p + 0x14);
                let sc = if m.call(0x0013_2b10, &[b, 0x22], &[]).0 != 0 {
                    let b = m.r32(p + 0x14);
                    (m.r32(b + 0xec) == 0).then(|| m.rf(b + 0x10c))
                } else {
                    let b = m.r32(p + 0x14);
                    Some(m.rf(b + 0x184))
                };
                sc.map(|sc| {
                    let a = m.r32(p);
                    let pa = m.call(0x001a_62e8, &[a], &[]).1;
                    let b = m.r32(p + 0x14);
                    let pb = m.call(0x001a_62e8, &[b], &[]).1;
                    callf(m, 0x0024_58f0, &[], &[s(pa[1] - pb[1]), s(pa[0] - pb[0])]) * sc
                })
            }
        }
        _ => {
            let a = m.r32(p);
            let t = m.r32(a + 0x208);
            if t != 0 && 0.7 < m.rf(t + 0x88) {
                let x = m.r32(t + 0x4c);
                if m.r64(x + 0x148) & 0x2000_0000 == 0 && m.r32(a + 0x108c) == 0 {
                    let (h, q) = (m.r32(t + 0x40), m.rv(t + 0x60));
                    m.call(0x0019_feb0, &[a + 0x9e0, h], &[q]);
                }
            }
            return;
        }
    };
    if let Some(f) = f {
        let a = m.r32(p);
        let h = m.r32(p + 0x18);
        m.call(0x0019_fd30, &[a + 0x9e0, h], &[s(f)]);
    }
}

// ---------------------------------------------------------------- run_0017de40 (escort / guard brains)
/// FUN_0017e0e0: +0x3c = clock; heading from +0x38 to +0x10 (atan2, xy); best = FUN_001c1ec0(goal+0x40,
/// pos(+0x10)) (goal = +0x18); over the 16 offsets DAT_002be770: probe pos + VecFromAngle(heading + off,
/// DAT_002be76c) on the floor (FUN_001c0b50(+0x10 ->+0xe8 ->+0xdc ->+0x10)); farthest (xy) hit from +0x38
/// wins; FUN_002562a8(&msg), msg.+0x10 = best; FUN_0017dd38(goal, &msg)
pub fn ee_sub_17e0e0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let sp = stack_push(m, 0x50);
    let c = m.r32(CLOCK);
    m.w32(o + 0x3c, c);
    let g = m.r32(o + 0x18);
    let x = m.r32(o + 0x38);
    let a = m.call(0x001a_62e8, &[x], &[]).1;
    let x = m.r32(o + 0x10);
    let b = m.call(0x001a_62e8, &[x], &[]).1;
    let x = m.r32(o + 0x10);
    let e = m.r32(x + 0xe8);
    let e = if e != 0 { m.r32(e + 0xdc) } else { 0 };
    let fl = if e != 0 { m.r32(e + 0x10) } else { 0 };
    let h = callf(m, 0x0024_58f0, &[], &[s(b[1] - a[1]), s(b[0] - a[0])]);
    let mut best = m.call(0x001c_1ec0, &[g + 0x40], &[b]).1;
    let mut far = 0.0f32;
    for i in 0..0x10u32 {
        let (off, len) = (m.rf(0x002b_e770 + 4 * i), m.rf(0x002b_e76c));
        let v = m.call(0x0011_e1a0, &[], &[s(h + off), s(len), s(0.0)]).1;
        let q = [b[0] + v[0], b[1] + v[1], b[2] + v[2], b[3] + v[3]];
        if m.call(0x001c_0b50, &[fl, sp], &[q]).0 != 0 {
            let hp = m.rv(sp);
            let d = dxy(hp, a);
            if far < d {
                far = d;
                best = hp;
            }
        }
    }
    m.call(0x0025_62a8, &[sp + 0x10], &[]);
    m.wv(sp + 0x20, best);
    m.call(0x0017_dd38, &[g, sp + 0x10], &[]);
    stack_pop(m, 0x50);
}
/// FUN_0017e278 (set state): changed -> FUN_001ef0d8(+0x18), +0x30 = k, +0x34 = clock; k 1: brain 0xba
/// (Brain_b0_205438(+0x10, ..)) +0xc0 = 1, +0x38 = 0x1c, FUN_0017e0e0, vslot +0x44; 0: brain 0x9d +0x3c =
/// 0xb, +0x38 = this+0x38, vslot +0x44; 2: brain 0x9e +0x3c = 1, vslot +0x44
pub fn ee_sub_17e278<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32) {
    if k == m.r32(o + 0x30) {
        return;
    }
    m.call(0x001e_f0d8, &[o + 0x18], &[]);
    m.w32(o + 0x30, k);
    let c = m.r32(CLOCK);
    m.w32(o + 0x34, c);
    let x = m.r32(o + 0x10);
    match k {
        1 => {
            let b = m.call(0x0020_5438, &[x, 0xba], &[]).0;
            m.w32(b + 0xc0, 1);
            m.w32(b + 0x38, 0x1c);
            ee_sub_17e0e0(m, o);
            vcall(m, b, 0xc, 0x44, &[], &[]);
        }
        0 => {
            let b = m.call(0x0020_5438, &[x, 0x9d], &[]).0;
            m.w32(b + 0x3c, 0xb);
            let v = m.r32(o + 0x38);
            m.w32(b + 0x38, v);
            vcall(m, b, 0xc, 0x44, &[], &[]);
        }
        2 => {
            let b = m.call(0x0020_5438, &[x, 0x9e], &[]).0;
            m.w32(b + 0x3c, 1);
            vcall(m, b, 0xc, 0x44, &[], &[]);
        }
        _ => {}
    }
}
/// FUN_0017e6b0 (set state): changed -> FUN_001ef0d8(+0x18), +0x38 = k, +0x3c = clock; k 0 with route
/// (+0x30) mode +0x120 0: brain 0xa7, path (+0x10 ->+0x294) = {pos, FUN_001c1ec0(route + 0x60 + 0x20 *
/// +0x34, pos)} (+0xe8 = 2, FUN_001e6180), +0x40 = clock + max(xy dist / DAT_002be7b0, DAT_002be7b4),
/// brain +0x40 = 1, +0x64 = 10.0, FUN_00178d90(brain, path, 0), vslot +0x44; mode 1: FUN_0017e628; k 1:
/// brain 0xbf +0x30.. = route waypoint (+0x60 / +0x70 quads), vslot +0x44
pub fn ee_sub_17e6b0<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32) {
    if k == m.r32(o + 0x38) {
        return;
    }
    m.call(0x001e_f0d8, &[o + 0x18], &[]);
    m.w32(o + 0x38, k);
    let c = m.r32(CLOCK);
    m.w32(o + 0x3c, c);
    match k {
        0 => {
            let r = m.r32(o + 0x30);
            match m.r32(r + 0x120) {
                0 => {
                    let x = m.r32(o + 0x10);
                    let b = m.call(0x0020_5438, &[x, 0xa7], &[]).0;
                    let x = m.r32(o + 0x10);
                    let path = m.r32(x + 0x294);
                    let pos = m.call(0x001a_62e8, &[x], &[]).1;
                    let i = m.r32(o + 0x34);
                    let r = m.r32(o + 0x30);
                    let q = m.call(0x001c_1ec0, &[r.wrapping_add(i << 5) + 0x60], &[pos]).1;
                    let d = dxy(pos, q) / m.rf(0x002b_e7b0);
                    let c = m.rf(CLOCK);
                    let k1 = m.rf(0x002b_e7b4);
                    m.wf(o + 0x40, c + d.max(k1));
                    m.w32(path + 0xe8, 2);
                    let pts = m.r32(path + 0xf4);
                    m.wv(pts, pos);
                    let pts = m.r32(path + 0xf4);
                    m.wv(pts + 0x10, q);
                    m.call(0x001e_6180, &[path], &[]);
                    m.w32(b + 0x40, 1);
                    m.w32(b + 0x64, 0x4120_0000);
                    m.call(0x0017_8d90, &[b, path, 0], &[]);
                    vcall(m, b, 0xc, 0x44, &[], &[]);
                }
                1 => drop(m.call(0x0017_e628, &[o], &[])),
                _ => {}
            }
        }
        1 => {
            let x = m.r32(o + 0x10);
            let b = m.call(0x0020_5438, &[x, 0xbf], &[]).0;
            let i = m.r32(o + 0x34);
            let r = m.r32(o + 0x30).wrapping_add(i << 5);
            let (a, c) = (m.rv(r + 0x60), m.rv(r + 0x70));
            m.wv(b + 0x30, a);
            m.wv(b + 0x40, c);
            vcall(m, b, 0xc, 0x44, &[], &[]);
        }
        _ => {}
    }
}

// ---------------------------------------------------------------- run_00169198 (level scene chunks) / run_001856b8
/// FUN_001699e8: LevelScene_build (FUN_00169ac8) for each of the `n` (t0) chunks i: (a0, a1, a2, a3,
/// a1 + 0x14 (i + 1), a2 + 0x28 (i + 1), a3 ? a3 + 0x18 (i + 1) : 0, t1; stack t2, t3, st0) — register
/// args first, then the three stack words
#[allow(clippy::too_many_arguments)]
pub fn ee_sub_1699e8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, n: u32, t1: u32, t2: u32, t3: u32, st0: u32) {
    for i in 1..=n as i32 {
        let i = i as u32;
        let c = if a3 != 0 { a3 + 0x18 * i } else { 0 };
        m.call(0x0016_9ac8, &[a0, a1, a2, a3, a1 + 0x14 * i, a2 + 0x28 * i, c, t1, t2, t3, st0], &[]);
    }
}
/// FUN_00169eb8: copy the `n` quadword DMA chain `src` (FUN_001ae238 {n * 16, 16}); dst+0x10 = copy,
/// dst+0x14 rebased; hdr+0x10 = n; with `fix` (t0): walk the tags (>> 28): 4 -> word 1 += t1 - t2, next
/// quad; 1 -> skip the (u16) quads + 1; 6 -> end; (any other tag spins on the same quad, as the original)
#[allow(clippy::too_many_arguments)]
pub fn ee_sub_169eb8<M: QMem + ?Sized>(m: &mut M, hdr: u32, dst: u32, n: u32, src: u32, fix: u32, t1: u32, t2: u32) {
    let sp = stack_push(m, 0x10);
    m.w32(sp, n << 4);
    m.w32(sp + 4, 0x10);
    m.w32(hdr + 0x10, n);
    let c = m.call(0x001a_e238, &[sp, src], &[]).0;
    let v = m.r32(dst + 0x14);
    m.w32(dst + 0x10, c);
    m.w32(dst + 0x14, v.wrapping_add(c.wrapping_sub(src)));
    stack_pop(m, 0x10);
    if fix == 0 {
        return;
    }
    let mut p = c;
    let d = t1.wrapping_sub(t2);
    let end = |m: &mut M| {
        let (b, k) = (m.r32(dst + 0x10), m.r32(hdr + 0x10));
        b.wrapping_add(k << 4)
    };
    while p < end(m) {
        match m.r32(p) >> 28 {
            4 => {
                let w = m.r32(p + 4);
                m.w32(p + 4, w.wrapping_add(d));
                p += 0x10;
            }
            1 => {
                let k = m.r16(p) as u32;
                p = p.wrapping_add((k << 4) + 0x10);
            }
            6 => p = end(m),
            _ => {}
        }
    }
}
/// FUN_00185a80: project `p` onto the plane set `pl` ({n, ..} then n planes {normal xyz, d}): Gram matrix
/// A[i][j] = n_i . n_j, b[i] = n_i . p + d_i, SolveInequalities(DAT_002a568c, n, A, b, x) (FUN_0011ad30);
/// returns p + sum x_i * plane_i (+1 in w per plane, vadda with vf0)
pub fn ee_sub_185a80<M: QMem + ?Sized>(m: &mut M, _a0: u32, pl: u32, p: V4) -> V4 {
    let n = m.r32(pl) as i32;
    if n == 0 {
        return p;
    }
    let sp = stack_push(m, 0x100);
    let (a, b, x) = (sp, sp + 0x90, sp + 0xb0);
    let mut k = 0;
    for i in 0..n.max(0) as u32 {
        let ri = m.rv(pl + 0x10 + 0x10 * i);
        for j in 0..n as u32 {
            let rj = m.rv(pl + 0x10 + 0x10 * j);
            m.wf(a + 4 * k, dot3(ri, rj));
            k += 1;
        }
    }
    for i in 0..n.max(0) as u32 {
        let r = m.rv(pl + 0x10 + 0x10 * i);
        m.wf(b + 4 * i, r[3] + dot3(p, r));
    }
    let e = crate::engine::classes::q::q6::rof(0x002a_568c);
    m.call(0x0011_ad30, &[n as u32, a, b, x], &[s(e)]);
    let mut v = p;
    let n2 = (m.r32(pl) as i32).max(0) as u32;
    for i in 0..n2 {
        let (r, f) = (m.rv(pl + 0x10 + 0x10 * i), m.rf(x + 4 * i));
        v = [v[0] + r[0] * f, v[1] + r[1] * f, v[2] + r[2] * f, v[3] + 1.0 + r[3] * f];
    }
    stack_pop(m, 0x100);
    v
}

// ---------------------------------------------------------------- run_00154908 (mission 0 brain / loot roll)
/// FUN_00154a90 (mission 0 message handler, `ctx` = a2): 3 -> +0x14 = &DAT_002bdc00, +8 / +0xc / +0x10 =
/// FUN_001d88d0(0x2a5060, 0x2a5070 / 80 / 90, 0); 0x10, 0x1d -> FUN_00154d88(this, (+0x14)->+8); 0x22 ->
/// link +8 -> +0xc -> +0x10 (FUN_001d7ed8(&r, 0x3e0ea8, next, 0x3d4a60), FUN_00258200(&o, cur, &r,
/// 0x3d4a60), Release both) then FUN_00258200(&o, +0x10, clone({0x16, *(ctx +0xa8)}), 0x3d4a60); 0x25 ->
/// when ctx is the world's +0x7c with +0x54 == 2 and +0x68 == 1: FUN_00154d88(this, (+0x14)->+0xc)
pub fn ee_sub_154a90<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32, ctx: u32) {
    match k {
        3 => {
            m.w32(o + 0x14, 0x002b_dc00);
            for (i, n) in [(8, 0x002a_5070), (0xc, 0x002a_5080), (0x10, 0x002a_5090)] {
                let v = m.call(0x001d_88d0, &[0x002a_5060, n, 0], &[]).0;
                m.w32(o + i, v);
            }
        }
        0x10 | 0x1d => {
            let t = m.r32(o + 0x14);
            let t = m.rf(t + 8);
            ee_sub_154d88(m, o, t);
        }
        0x22 => {
            let sp = stack_push(m, 0x60);
            let (b0, a0, n90, s80, s70, s60) = (sp, sp + 0x10, sp + 0x20, sp + 0x30, sp + 0x40, sp + 0x50);
            for (cur, nx, out) in [(8, 0xc, a0), (0xc, 0x10, n90)] {
                let n = m.r32(o + nx);
                m.call(0x001d_7ed8, &[b0, 0x003e_0ea8, n, 0x003d_4a60], &[]);
                let c = m.r32(o + cur);
                m.call(0x0025_8200, &[out, c, b0, 0x003d_4a60], &[]);
                m.call(0x0012_9ec8, &[out], &[]);
                m.call(0x0012_9ec8, &[b0], &[]);
            }
            let c = m.r32(o + 0x10);
            let x = m.r32(ctx + 0xa8);
            let x = m.r32(x);
            m.w32(s80 + 4, x);
            m.w32(s80, 0x16);
            m.call(0x0012_91a8, &[s70, s80], &[]);
            m.call(0x0025_8200, &[s60, c, s70, 0x003d_4a60], &[]);
            m.call(0x0012_9ec8, &[s60], &[]);
            m.call(0x0012_9ec8, &[s70], &[]);
            m.call(0x0012_9ec8, &[s80], &[]);
            stack_pop(m, 0x60);
        }
        0x25 => {
            if m.r32(WORLD + 0x3b0c) == m.r32(ctx + 0x7c) && m.r32(ctx + 0x54) == 2 && m.r32(ctx + 0x68) == 1 {
                let t = m.r32(o + 0x14);
                let t = m.rf(t + 0xc);
                ee_sub_154d88(m, o, t);
            }
        }
        _ => {}
    }
}
/// random int in [lo, hi] (inclusive by +0.999) from r = FUN_00154c90(0x2de578), reversed when lo > hi
fn roll<M: QMem + ?Sized>(m: &mut M, lo: i32, hi: i32) -> i32 {
    let r = callf(m, 0x0015_4c90, &[0x002d_e578], &[]);
    let e = f32::from_bits(0x3f7f_be77);
    if lo == hi {
        hi
    } else if lo < hi {
        lo.wrapping_add((r * ((hi as f32 + e) - lo as f32)) as i32)
    } else {
        hi.wrapping_add(((1.0 - r) * ((lo as f32 + e) - hi as f32)) as i32)
    }
}
/// FUN_00154e10 (loot drop counters `c` = {coins, items, .., .., .., table}): counters below 1 re-rolled
/// (FUN_0011d3f8 of two rolls over table +0x28/+0x30 & +0x2c/+0x34, +0x38/+0x40 & +0x3c/+0x44); coins -=
/// (actor in state 0x38 ? 2 : 1), items += `di`; n < 0 -> n = FUN_0011d3f8(table +0x10/+0x14 when the coins
/// ran out, else +0x18/+0x1c); n > 0 -> spawn n coins (FUN_001c6fd8(world, 0x5b, pos, actor, n)); items
/// below 1 -> spawn one item (kind 0x5d)
pub fn ee_sub_154e10<M: QMem + ?Sized>(m: &mut M, c: u32, pos: V4, n: i32, di: u32, actor: u32) {
    for (i, a, b) in [(0u32, 0x28u32, 0x2cu32), (4, 0x38, 0x3c)] {
        if (m.r32(c + i) as i32) < 1 {
            let t = m.r32(c + 0x14);
            let (l0, h0) = (m.r32(t + a) as i32, m.r32(t + a + 8) as i32);
            let x = roll(m, l0, h0);
            let t = m.r32(c + 0x14);
            let (l1, h1) = (m.r32(t + b) as i32, m.r32(t + b + 8) as i32);
            let y = roll(m, l1, h1);
            let v = m.call(0x0011_d3f8, &[x as u32, y as u32], &[]).0;
            m.w32(c + i, v);
        }
    }
    let st = m.call(0x0013_2b10, &[actor, 0x38], &[]).0;
    let coins = m.r32(c) as i32;
    let d = if st != 0 { -2 } else { -1 };
    let it = m.r32(c + 4).wrapping_add(di);
    m.w32(c + 4, it);
    let left = coins.wrapping_add(d);
    m.w32(c, left as u32);
    let mut n = n;
    if n < 0 {
        let t = m.r32(c + 0x14);
        let (a, b) = if left < 1 { (0x10, 0x14) } else { (0x18, 0x1c) };
        let (x, y) = (m.r32(t + a), m.r32(t + b));
        n = m.call(0x0011_d3f8, &[x, y], &[]).0 as i32;
    }
    if 0 < n {
        m.call(0x001c_6fd8, &[0x003d_d490, 0x5b, actor, n as u32], &[pos]);
    }
    if (m.r32(c + 4) as i32) < 1 {
        m.call(0x001c_6fd8, &[0x003d_d490, 0x5d, actor, 1], &[pos]);
    }
}

// ---------------------------------------------------------------- run_001bb400 (scripted sequence player)
/// FUN_001bb648 (fire the steps of sequence entry `e` = {_, repeat, n, steps (8 B {kind, arg})}): kinds
/// 0..5 skipped once repeating (+0x158 > 0, or the entry has no repeat count); kinds 0, 1, 6, 7 resolve
/// arg (FUN_0015e938(world, 1, arg, this), none -> skip); 0 / 1 -> its vslot +0x44 / +0x4c; 2 / 3 ->
/// FUN_001f6978 / FUN_001f69f8(this, arg); 4 / 5 -> FUN_001f6ac8 / FUN_001f6b48(this, arg); 6 -> remember it;
/// 7 -> (in state 0x1e) vslot +0x19c(&{1, remembered or it}); 8 (+0x168 & 2) -> FUN_0014f750(0x2ddf40,
/// arg); 9 (& 4) -> RumbleRequest(0x2dfc00, arg, 0.3); 10 (& 8) -> loot (FUN_00154e10(0x2de578, pos of the
/// remembered object or this, arg, -100, this)); 0xb -> own vslot +0x1c(4, 0, 1, &(&arg), 0); 0xc ->
/// FUN_00161f48(0x2deb30, 1, arg, this)
pub fn ee_sub_1bb648<M: QMem + ?Sized>(m: &mut M, o: u32, e: u32) {
    if (m.r32(e + 8) as i32) < 1 {
        return;
    }
    let sp = stack_push(m, 0x50);
    let mut keep = 0u32;
    let mut i = 0u32;
    while (i as i32) < m.r32(e + 8) as i32 {
        let p = m.r32(e + 0xc) + i * 8;
        i += 1;
        let k = m.r32(p);
        let rep = 0 < m.r32(o + 0x158) as i32 || (m.r32(e + 4) as i32) < 1;
        if rep && (0x3fu32 >> (k & 0x1f)) & 1 != 0 {
            continue;
        }
        let mut x = 0;
        if (0xc3u32 >> (k & 0x1f)) & 1 != 0 {
            let id = m.r32(p + 4);
            x = m.call(0x0015_e938, &[WORLD, 1, id, o], &[]).0;
            if x == 0 {
                continue;
            }
        }
        let a = m.r32(p + 4);
        match k {
            0 => drop(vcall(m, x, 0xc, 0x44, &[], &[])),
            1 => drop(vcall(m, x, 0xc, 0x4c, &[], &[])),
            2 => drop(m.call(0x001f_6978, &[o, a], &[])),
            3 => drop(m.call(0x001f_69f8, &[o, a], &[])),
            4 => drop(m.call(0x001f_6ac8, &[o, a], &[])),
            5 => drop(m.call(0x001f_6b48, &[o, a], &[])),
            6 => keep = x,
            7 => {
                if m.call(0x0013_2b10, &[x, 0x1e], &[]).0 != 0 {
                    m.w32(sp, 1);
                    m.w32(sp + 4, if keep != 0 { keep } else { x });
                    vcall(m, x, 0xc, 0x19c, &[sp], &[]);
                }
            }
            8 if m.r32(o + 0x168) & 2 != 0 => drop(m.call(0x0014_f750, &[0x002d_df40, a], &[])),
            9 if m.r32(o + 0x168) & 4 != 0 => drop(m.call(0x0018_7e28, &[0x002d_fc00, a], &[s(0.3)])),
            10 if m.r32(o + 0x168) & 8 != 0 => {
                let pos = m.call(0x001a_62e8, &[if keep != 0 { keep } else { o }], &[]).1;
                ee_sub_154e10(m, 0x002d_e578, pos, a as i32, (-100i32) as u32, o);
            }
            0xb => {
                m.w32(sp + 0x40, p + 4);
                vcall(m, o, 0xc, 0x1c, &[4, 0, 1, sp + 0x40, 0], &[]);
            }
            0xc => drop(ee_sub_161f48(m, 0x002d_eb30, 1, a, o)),
            _ => {}
        }
    }
    stack_pop(m, 0x50);
}
/// FUN_001bb400 (sequence step): FUN_001bbe48(this) set -> nothing; +0x168 = 0xe when +0x16c is the
/// player (FUN_001c4840(world)), 8 when +0x170 is 0x100 / 0x10000 / 0x20000 or +0x16c is in state 0x3b,
/// else 0; own vslot +0x1c(0x10, 0, 0, 0, 0); +0x168 bit 0 clear -> fire the previous (+0x164 - 0x10) and
/// current (+0x164 + 0x10 * +0x15c) entries, notify FUN_00156b98(world, 0, pos, +0x16c) unless +0x16c is in
/// state 0x38 but not 0x3c; repeat count (+0x158) reached -> next entry, past +0x160 -> own vslot +0x24
/// (0x10, this), vslot +0x1c(5, ..), FollowWorld activity 0x11 (FUN_001619a0(0x2deb30, 0x11, this)); then
/// (FUN_001bbe18) the new entry has no repeat -> clear the +0x150 flags (+0x154 [i].+4)
pub fn ee_sub_1bb400<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.call(0x001b_be48, &[o], &[]).0 != 0 {
        return;
    }
    let pl = m.call(0x001c_4840, &[WORLD], &[]).0;
    let t = m.r32(o + 0x16c);
    let st = if t == pl {
        0xe
    } else {
        let k = m.r32(o + 0x170);
        if k == 0x100 || k == 0x10000 || k == 0x20000 || (t != 0 && m.call(0x0013_2b10, &[t, 0x3b], &[]).0 != 0) {
            8
        } else {
            0
        }
    };
    m.w32(o + 0x168, st);
    vcall(m, o, 0xc, 0x1c, &[0x10, 0, 0, 0, 0], &[]);
    if m.r32(o + 0x168) & 1 != 0 {
        return;
    }
    let base = m.r32(o + 0x164);
    let cur = base.wrapping_add(m.r32(o + 0x15c) << 4);
    m.call(0x001b_b648, &[o, base.wrapping_sub(0x10)], &[]);
    m.call(0x001b_b648, &[o, cur], &[]);
    let t = m.r32(o + 0x16c);
    let quiet = t != 0
        && m.call(0x0013_2b10, &[t, 0x38], &[]).0 != 0
        && {
            let t = m.r32(o + 0x16c);
            m.call(0x0013_2b10, &[t, 0x3c], &[]).0 == 0
        };
    if !quiet {
        let pos = m.call(0x001a_62e8, &[o], &[]).1;
        let t = m.r32(o + 0x16c);
        m.call(0x0015_6b98, &[WORLD, 0, t], &[pos]);
    }
    if 0 < m.r32(cur + 4) as i32 {
        let n = m.r32(o + 0x158).wrapping_add(1);
        m.w32(o + 0x158, n);
        if m.r32(cur + 4) as i32 <= n as i32 {
            let j = m.r32(o + 0x15c).wrapping_add(1);
            m.w32(o + 0x158, 0);
            m.w32(o + 0x15c, j);
            if m.r32(o + 0x160) as i32 <= j as i32 {
                vcall(m, o, 0xc, 0x24, &[0x10, o], &[]);
                vcall(m, o, 0xc, 0x1c, &[5, 0, 0, 0, 0], &[]);
                m.call(0x0016_19a0, &[0x002d_eb30, 0x11, o], &[]);
            }
        }
    }
    if m.call(0x001b_be18, &[o], &[]).0 != 0 {
        let e = m.r32(o + 0x164).wrapping_add(m.r32(o + 0x15c) << 4);
        if (m.r32(e + 4) as i32) < 1 {
            let mut i = 0u32;
            while (i as i32) < m.r32(o + 0x150) as i32 {
                let b = m.r32(o + 0x154);
                m.w32(b + i * 8 + 4, 0);
                i += 1;
            }
        }
    }
}

// ---------------------------------------------------------------- run_0015b9f0 (animated emitter source sampling)
/// FUN_0015b9f0: source object so = ((a1+8)->+8); its anim state (+0xe4) frame (->+8) word temporarily
/// defaulted to so+0x10; out+0x10 = 0, FUN_0015b7e8(n, so+0xe4, out), (so+0xe4)+0x1c = 0; base = vslot
/// +0x104(so, &B) (B 4x4); for each of the `n` samples i (time t = times(a2+0xc)[i] - clock): unless t is
/// within 1e-4 of the previous one (initial DAT_002a5150): M = FUN_002309e8(base, t) * B, and for record
/// kind (so+0xe4)+0x18 != 3: R = FUN_002309e8(.., &V) * B (+ V * B); point[i].xyz = M * p (w = M[0].w);
/// kind != 3: vel[i].xyz = R3x3 * v; kind != 2: vel[i] += V * (kind 1 ? point[i] : R[3]) (xyz with
/// translation, w = V[0].w); vel[i] *= -1 (all lanes); finally the frame word is restored
pub fn ee_sub_15b9f0<M: QMem + ?Sized>(m: &mut M, n: u32, a1: u32, a2: u32, out: u32) {
    let sp = stack_push(m, 0x180);
    let x = m.r32(a1 + 8);
    let so = m.r32(x + 8);
    let x = m.r32(so + 0xe4);
    let fw = m.r32(x + 8);
    let saved = m.r32(fw);
    if saved == 0 {
        let v = m.r32(so + 0x10);
        m.w32(fw, v);
    }
    m.w32(out + 0x10, 0);
    let st = m.r32(so + 0xe4);
    m.call(0x0015_b7e8, &[n, st, out], &[]);
    let mut prev = crate::engine::classes::q::q6::rof(0x002a_5150);
    let st = m.r32(so + 0xe4);
    m.w32(st + 0x1c, 0);
    let base = vcall(m, so, 0xc, 0x104, &[sp + 0x100], &[]).0;
    for k in 0..4 {
        let q = m.rv(sp + 0x100 + 0x10 * k);
        m.wv(sp + 0xc0 + 0x10 * k, q);
    }
    let fma = |r: [V4; 4], v: V4, w: f32| -> [f32; 3] {
        let mut o = [0.0; 3];
        for i in 0..3 {
            o[i] = r[0][i] * v[0] + r[1][i] * v[1] + r[2][i] * v[2] + r[3][i] * w;
        }
        o
    };
    let mut off = 0u32;
    for i in 0..(n as i32).max(0) as u32 {
        let tb = m.r32(a2 + 0xc);
        let t = m.rf(tb + 4 * i) - m.rf(CLOCK);
        let st = m.r32(so + 0xe4);
        let kind = m.r32(st + 0x18);
        let eps = f32::from_bits(0x38d1_b717);
        if m.call(0x0011_d4d0, &[], &[s(t), s(prev), s(eps)]).0 == 0 {
            prev = t;
            m.call(0x0023_09e8, &[base, 0, sp, 0], &[s(t)]);
            m.call(0x0011_c168, &[sp + 0x140, sp, sp + 0xc0], &[]);
            for k in 0..4 {
                let q = m.rv(sp + 0x140 + 0x10 * k);
                m.wv(sp + 0x10 * k, q);
            }
            if kind != 3 {
                m.call(0x0023_09e8, &[base, 0, sp + 0x40, sp + 0x80], &[s(t)]);
                m.call(0x0011_c168, &[sp + 0x140, sp + 0x40, sp + 0x100], &[]);
                for k in 0..4 {
                    let q = m.rv(sp + 0x140 + 0x10 * k);
                    m.wv(sp + 0x40 + 0x10 * k, q);
                }
                m.call(0x0011_c168, &[sp + 0x140, sp + 0x80, sp + 0x100], &[]);
                for k in 0..4 {
                    let q = m.rv(sp + 0x140 + 0x10 * k);
                    m.wv(sp + 0x80 + 0x10 * k, q);
                }
            }
        }
        let rows = |m: &mut M, b: u32| [m.rv(b), m.rv(b + 0x10), m.rv(b + 0x20), m.rv(b + 0x30)];
        let pp = m.r32(out).wrapping_add(off);
        let mm = rows(m, sp);
        let p = m.rv(pp);
        let r = fma(mm, p, 1.0);
        m.wv(pp, [r[0], r[1], r[2], mm[0][3]]);
        if kind != 3 {
            let vp = m.r32(out + 4).wrapping_add(off);
            let rr = rows(m, sp + 0x40);
            let v = m.rv(vp);
            let r = fma([rr[0], rr[1], rr[2], [0.0; 4]], v, 0.0);
            m.wv(vp, [r[0], r[1], r[2], v[3]]);
            if kind != 2 {
                let w = if kind == 1 {
                    let q = m.r32(out).wrapping_add(off);
                    m.rv(q)
                } else {
                    rr[3]
                };
                m.wv(sp + 0x140, w);
                let vp = m.r32(out + 4).wrapping_add(off);
                let vv = rows(m, sp + 0x80);
                let a = fma(vv, w, 1.0);
                let v = m.rv(vp);
                m.wv(vp, [v[0] + a[0], v[1] + a[1], v[2] + a[2], v[3] + vv[0][3]]);
            }
        }
        let vp = m.r32(out + 4).wrapping_add(off);
        let v = m.rv(vp);
        m.wv(sp + 0x140, [-1.0, 0.0, 0.0, 0.0]);
        m.wv(vp, [-v[0], -v[1], -v[2], -v[3]]);
        off += 0x10;
    }
    let st = m.r32(so + 0xe4);
    let fw = m.r32(st + 8);
    m.w32(fw, saved);
    stack_pop(m, 0x180);
}

// ---------------------------------------------------------------- run_0015a138 (impact effect + hit sound)
/// FUN_0015a2c0 (impact at `at` (a2) by `src` (a0, record `rec` a1), `chance` f12, `rate` f13, `size`
/// f14, effect pos `fx` (a3)): d = |at - vslot +0x10c(src)|.xyz; lucky = FUN_0011d448(0, 3) < chance;
/// sink = src when in state 0x32. Far (d > 10) or lucky: k = clamp((d - 10) / 790, 0, 1); sink with
/// rec+0x180: volume 0.9k + 0.1 on its loop sound (+0x174, created from one of the 4 cues DAT_002bdeb0 via
/// FUN_001fbb90 / FUN_001fb2f8 / FUN_001fb4f0(100, 1000)). Unless lucky, throttle: p = 1 - k * (1 - rate);
/// sink -> clock - sink+0x170 > p, else (int)(clock / p) != (int)((clock + chance) / p). Effect rec+0x17c ->
/// FxQueue_push(world, 7, 0) item: vslot(+0xec) +0xc(fx, 0, size), +0x20 = size, +0x24 = 0.6 size, +0x1c =
/// rec+0x17c, sink+0x170 = clock; d > 500 with rec+0x178 -> debris burst FUN_0015be28(fx + 30 z,
/// DAT_002a2dc0, FUN_0011d3f8(4, 6), {1, _, +0x178 id, +0x17c id, rec+0x18, _, 8, 12, 1}, {d/2, 3d/4, 0, 1,
/// gravity (src+0xe8)+0x80, DAT_002a2dc0, .., 1, 5, pi/8, pi/3, .., 1, _, -1}, {0, 0, +0x178, +0x17c, rec},
/// 0, 0). Near and unlucky: sink loop sound stopped (FUN_001fb098, cref cleared)
#[allow(clippy::too_many_arguments)]
pub fn ee_sub_15a2c0<M: QMem + ?Sized>(m: &mut M, src: u32, rec: u32, at: V4, fx: V4, chance: f32, rate: f32, size: f32) {
    let sp = stack_push(m, 0x160);
    let p = vcall(m, src, 0xc, 0x10c, &[], &[]).1;
    let v = [at[0] - p[0], at[1] - p[1], at[2] - p[2]];
    let d = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    let r = callf(m, 0x0011_d448, &[], &[s(0.0), s(3.0)]);
    let lucky = r < chance;
    let sink = if m.call(0x0013_2b10, &[src, 0x32], &[]).0 != 0 { src } else { 0 };
    if 10.0 < d || lucky {
        let f = (d - 10.0) * f32::from_bits(0x3aa5_e9f7);
        let k = if f < 0.0 {
            0.0
        } else if 1.0 < f {
            1.0
        } else {
            f
        };
        if sink != 0 && m.r32(rec + 0x180) != 0 {
            let vol = k * f32::from_bits(0x3f66_6666) + f32::from_bits(0x3dcc_cccd);
            let e = m.r32(sink + 0x174);
            if e != 0 {
                m.call(0x001f_b1f8, &[e], &[s(vol)]);
            } else {
                let i = m.call(0x0011_d3f8, &[0, 3], &[]).0;
                let cue = m.r32(0x002b_deb0u32.wrapping_add(i << 2));
                let e = m.call(0x001f_bb90, &[cue], &[]).0;
                m.call(0x001f_b2f8, &[e, sink], &[]);
                m.call(0x001f_b4f0, &[e], &[s(100.0), s(1000.0)]);
                m.call(0x001f_b1f8, &[e], &[s(vol)]);
                m.call(0x0025_8040, &[sink + 0x174, e], &[]);
            }
        }
        let go = lucky || {
            let p = 1.0 - k * (1.0 - rate);
            let c = m.rf(CLOCK);
            if sink != 0 {
                p < c - m.rf(sink + 0x170)
            } else {
                (c / p) as i32 != ((c + chance) / p) as i32
            }
        };
        if go && m.r32(rec + 0x17c) != 0 {
            let it = m.call(0x001c_e888, &[WORLD, 7, 0], &[]).0;
            if it != 0 {
                vcall(m, it, 0xec, 0xc, &[0], &[fx, s(size)]);
                m.wf(it + 0x20, size);
                m.wf(it + 0x24, size * f32::from_bits(0x3f19_999a));
                let x = m.r32(rec + 0x17c);
                m.w32(it + 0x1c, x);
                if sink != 0 {
                    let c = m.r32(CLOCK);
                    m.w32(sink + 0x170, c);
                }
                if 500.0 < d && m.r32(rec + 0x178) != 0 {
                    let (a, b, c) = (sp + 0x40, sp + 0xf0, sp + 0x80);
                    m.call(0x0023_33b4, &[a, 0, 0x40], &[]);
                    m.w32(a, 1);
                    let x = m.r32(rec + 0x178);
                    let x = m.r32(x + 0x18);
                    m.w32(a + 8, x);
                    let x = m.r32(rec + 0x17c);
                    let x = m.r32(x + 0x18);
                    let y = m.r32(rec + 0x18);
                    m.wf(a + 0x18, 8.0);
                    m.wf(a + 0x1c, 12.0);
                    m.w32(a + 0x10, y);
                    m.w32(a + 0xc, x);
                    m.wf(a + 0x20, 1.0);
                    for k in 0..4 {
                        let q = m.rv(a + 0x10 * k);
                        m.wv(sp + 0x10 * k, q);
                    }
                    m.call(0x0023_33b4, &[b, 0, 0x70], &[]);
                    let e = m.r32(src + 0xe8);
                    m.wf(b + 0xc, 1.0);
                    m.wf(b, d * 0.5);
                    m.wf(b + 4, d * 0.75);
                    let (q0, q1) = (rov(0x002a_2dc0), m.rv(e + 0x80));
                    m.wf(b + 0x40, f32::from_bits(0x3ec9_0fdb));
                    m.wv(b + 0x20, q0);
                    m.wv(b + 0x10, q1);
                    m.wf(b + 0x3c, 5.0);
                    m.wf(b + 0x38, 1.0);
                    m.wf(b + 0x44, f32::from_bits(0x3f86_0a92));
                    m.wf(b + 0x60, 1.0);
                    m.wf(b + 0x68, -1.0);
                    for k in 0..7 {
                        let q = m.rv(b + 0x10 * k);
                        m.wv(c + 0x10 * k, q);
                    }
                    m.call(0x0023_33b4, &[a, 0, 0x14], &[]);
                    m.wv(a + 0x20, fx);
                    let z = m.rf(a + 0x28) + 30.0;
                    let (x, y) = (m.r32(rec + 0x178), m.r32(rec + 0x17c));
                    m.w32(a + 8, x);
                    m.wf(a + 0x28, z);
                    m.w32(a + 0xc, y);
                    let q = m.rv(a + 0x20);
                    m.w32(a + 0x10, rec);
                    let n = m.call(0x0011_d3f8, &[4, 6], &[]).0;
                    ee_sub_15be28(m, q, q0, n, sp, c, a, 0, 0);
                }
            }
        }
    } else if sink != 0 {
        let e = m.r32(sink + 0x174);
        if e != 0 {
            m.call(0x001f_b098, &[e], &[s(0.0)]);
            m.call(0x0025_8040, &[sink + 0x174, 0], &[]);
        }
    }
    stack_pop(m, 0x160);
}

// ---------------------------------------------------------------- run_0012fde8 (bone chain accumulate)
/// FUN_001300b8 (accumulate the chain `from` (a2) up to `to` (a3, parents via +0xa0) of anim `an` at
/// time/blend (f12, f13)): frame FUN_001ae478; P = identity, D = 0; bind table (FUN_001ae318 {n * 4, 16},
/// n = an+0x5c): per channel record (an+0x60 [i]) without bit 30 FUN_0012fe58(an, a1, rec, &tmp, &tbl[i]),
/// else 0. Per bone b: the first bound channel (not bit 31) -> its vslot +0x1c(f12, f13, &q, &dq);
/// L = FUN_0022c8c0(b, q); dL = 6 <= b+0x94 < 8 ? FUN_0022cb90(b, &L, q, dq) : FUN_0022cc00(b, dq);
/// unbound -> L = b's rest matrix (b+0..0x30), dL = 0. Then D = L * D + dL * P, P = L * P. out_p (t0) = P,
/// out_d (t1) = D (when set); FUN_001ae4a0(frame, 2)
#[allow(clippy::too_many_arguments)]
pub fn ee_sub_1300b8<M: QMem + ?Sized>(m: &mut M, an: u32, a1: u32, from: u32, to: u32, out_p: u32, out_d: u32, t: f32, w: f32) {
    let sp = stack_push(m, 0x210);
    m.call(0x001a_e478, &[sp + 0x100], &[]);
    let (l, dl, d, p, tb) = (sp, sp + 0x40, sp + 0xc0, sp + 0x80, sp + 0x130);
    m.call(0x0023_33b4, &[d, 0, 0x40], &[]);
    m.call(0x0023_33b4, &[p, 0, 0x40], &[]);
    for k in [0x3c, 0x28, 0x14, 0] {
        m.wf(p + k, 1.0);
    }
    let n = m.r32(an + 0x5c);
    m.w32(tb, n << 2);
    m.w32(tb + 4, 0x10);
    let arr = m.call(0x001a_e318, &[tb], &[]).0;
    let mut i = 0u32;
    while (i as i32) < m.r32(an + 0x5c) as i32 {
        let rec = m.r32(an + 0x60).wrapping_add(i << 3);
        if m.r32(rec) & 0x4000_0000 != 0 {
            m.w32(arr + 4 * i, 0);
        } else {
            m.call(0x0012_fe58, &[an, a1, rec, sp + 0x200, arr + 4 * i], &[]);
        }
        i += 1;
    }
    let copy = |m: &mut M, dst: u32, src: u32| {
        for k in 0..4 {
            let q = m.rv(src + 0x10 * k);
            m.wv(dst + 0x10 * k, q);
        }
    };
    let mut b = from;
    // ANIM-ALL R16: the walk ends at `to` (an ancestor via +0xa0); a chain that runs out (b == 0: the guest
    // image misses a link the original's load sets) stops there instead of spinning on address 0
    while b != to && b != 0 {
        let n = m.r32(an + 0x5c) as i32;
        let mut hit = false;
        for i in 0..n.max(0) as u32 {
            let rec = m.r32(an + 0x60).wrapping_add(i << 3);
            if m.r32(arr + 4 * i) != b || m.r32(rec) & 0x8000_0000 != 0 {
                continue;
            }
            let ch = m.r32(rec + 4);
            vcall(m, ch, 0, 0x1c, &[sp + 0x110, sp + 0x120], &[s(t), s(w)]);
            let q = m.rv(sp + 0x110);
            m.call(0x0022_c8c0, &[sp + 0x140, b], &[q]);
            copy(m, l, sp + 0x140);
            let k = m.r32(b + 0x94) as i32;
            let dq = m.rv(sp + 0x120);
            if (6..8).contains(&k) {
                m.call(0x0022_cb90, &[tb, b, l], &[q, dq]);
            } else {
                m.call(0x0022_cc00, &[tb, b], &[dq]);
            }
            copy(m, dl, tb);
            hit = true;
            break;
        }
        if !hit {
            copy(m, l, b);
            m.call(0x0023_33b4, &[dl, 0, 0x40], &[]);
        }
        m.call(0x0011_c168, &[sp + 0x180, l, d], &[]);
        m.call(0x0011_c168, &[sp + 0x1c0, dl, p], &[]);
        for k in 0..4 {
            let (x, y) = (m.rv(sp + 0x180 + 0x10 * k), m.rv(sp + 0x1c0 + 0x10 * k));
            m.wv(tb + 0x10 * k, [x[0] + y[0], x[1] + y[1], x[2] + y[2], x[3] + y[3]]);
        }
        copy(m, d, tb);
        m.call(0x0011_c168, &[tb, l, p], &[]);
        copy(m, p, tb);
        b = m.r32(b + 0xa0);
    }
    if out_p != 0 {
        copy(m, out_p, p);
    }
    if out_d != 0 {
        copy(m, out_d, d);
    }
    m.call(0x001a_e4a0, &[sp + 0x100, 2], &[]);
    stack_pop(m, 0x210);
}

/// FUN_001856c0 (boundary planes): M = vslot +0xfc(obj) (4x4); h = DAT_002dded4 / 2, reach = cap[2] *
/// (cap[0] + cap[1]) / 2 (cap = t0); for each triangle t < obj+0xe8 (verts obj+0xec, neighbours obj+0xf0,
/// 3 words each) with an open edge (neighbour -1): P[i].xyz = M * vert (w = vert w; P[2] w = M[0].w),
/// N = (P1 - P0) x (P2 - P0); per open edge j (A = P[(j+1)%3], B = P[(j+2)%3]): side = N x (B - A) (w =
/// M[0].w); best = DAT_002a5688; for each of the `n` points q (a3): FUN_0011ee08(q, A, B, &c, 0, &dist),
/// dist < r + reach -> u = side / |side| (all lanes), FUN_0011d018(cap, &s, r, u . (q - c), h); s < best
/// -> best = s, plane = FUN_001193c0(u, s); improved -> append the plane to `out` ({count, quads from
/// +0x10})
pub fn ee_sub_1856c0<M: QMem + ?Sized>(m: &mut M, obj: u32, out: u32, n: u32, pts: u32, cap: u32, r: f32) {
    let sp = stack_push(m, 0x110);
    let mp = vcall(m, obj, 0xc, 0xfc, &[], &[]).0;
    let mt = [m.rv(mp), m.rv(mp + 0x10), m.rv(mp + 0x20), m.rv(mp + 0x30)];
    let h = m.rf(0x002d_ded4) * 0.5;
    let reach = m.rf(cap + 8) * ((m.rf(cap) + m.rf(cap + 4)) * 0.5);
    let xf = |v: V4, w: f32| -> V4 {
        let mut o = [0.0, 0.0, 0.0, w];
        for i in 0..3 {
            o[i] = mt[0][i] * v[0] + mt[1][i] * v[1] + mt[2][i] * v[2] + mt[3][i];
        }
        o
    };
    let cross = |a: V4, b: V4, w: f32| -> V4 {
        [a[1] * b[2] - b[1] * a[2], a[2] * b[0] - b[2] * a[0], a[0] * b[1] - b[0] * a[1], w]
    };
    let sub = |a: V4, b: V4| -> V4 { [a[0] - b[0], a[1] - b[1], a[2] - b[2], a[3] - b[3]] };
    let big = crate::engine::classes::q::q6::rof(0x002a_5688);
    let mut t = 0u32;
    while (t as i32) < m.r32(obj + 0xe8) as i32 {
        let nb = m.r32(obj + 0xf0) + 0xc * t;
        let vi = m.r32(obj + 0xec) + 0xc * t;
        let open = (0..3).any(|k| m.r32(nb + 4 * k) == u32::MAX);
        if open {
            let vb = m.r32(obj + 0xe4);
            let mut raw = [[0.0f32; 4]; 3];
            for k in 0..3u32 {
                let i = m.r32(vi + 4 * k);
                raw[k as usize] = m.rv(vb.wrapping_add(i << 4));
            }
            let p = [xf(raw[0], raw[0][3]), xf(raw[1], raw[1][3]), xf(raw[2], mt[0][3])];
            for (k, q) in p.iter().enumerate() {
                m.wv(sp + 0x40 + 0x10 * k as u32, *q);
            }
            let nrm = cross(sub(p[1], p[0]), sub(p[2], p[0]), raw[2][3]);
            m.wv(sp + 0xa0, nrm);
            for j in 0..3u32 {
                let w = m.r32(obj + 0xf0).wrapping_add((3 * t + j) << 2);
                if m.r32(w) != u32::MAX {
                    continue;
                }
                let (a, b) = (p[((j + 1) % 3) as usize], p[((j + 2) % 3) as usize]);
                m.wv(sp + 0xb0, a);
                m.wv(sp + 0xc0, b);
                let mut best = big;
                let mut plane = [0.0f32; 4];
                let side = cross(nrm, sub(b, a), mt[0][3]);
                for k in 0..(n as i32).max(0) as u32 {
                    let q = m.rv(pts + 0x10 * k);
                    m.wv(sp + 0xf0, side);
                    m.wv(sp + 0x100, q);
                    m.call(0x0011_ee08, &[sp + 0x70, 0, sp + 0x90], &[q, a, b]);
                    if m.rf(sp + 0x90) < r + reach {
                        let l = (side[0] * side[0] + side[1] * side[1] + side[2] * side[2]).sqrt();
                        let c = m.rv(sp + 0x70);
                        let dq = sub(q, c);
                        let iq = 1.0 / l;
                        let u = [side[0] * iq, side[1] * iq, side[2] * iq, side[3] * iq];
                        m.wv(sp + 0xe0, u);
                        let d = u[0] * dq[0] + u[1] * dq[1] + u[2] * dq[2];
                        m.call(0x0011_d018, &[cap, sp + 0x94], &[s(r), s(d), s(h)]);
                        let sv = m.rf(sp + 0x94);
                        if sv < best {
                            best = sv;
                            m.call(0x0011_93c0, &[sp + 0x80], &[u, s(best)]);
                            plane = m.rv(sp + 0x80);
                        }
                    }
                }
                if best != big {
                    let c = m.r32(out);
                    m.w32(out, c.wrapping_add(1));
                    m.wv(out.wrapping_add(c << 4) + 0x10, plane);
                }
            }
        }
        t += 1;
    }
    stack_pop(m, 0x110);
}
