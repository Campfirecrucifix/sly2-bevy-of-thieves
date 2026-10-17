//! WP-Q6: small class-override packages (one section per class, original vtable in the heading):
//! init (slot 6) / load (13) / post-load (14) / update (15) / message (4) overrides that call their
//! parent slot (by address through `QMem::call`) and set a few fields. 1:1 against `QMem`.
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem, V4};
use crate::engine::classes::q::q6::{callf, ro32, rov, s, set_tagged, vcall, CLOCK, WORLD};

// ---------------------------------------------------------------- C2d8368 (parent C2d36d0)
/// FUN_001ddbd0 (slot 13): state `b` at +0x28, its start time +0x2c = DAT_002ddf0c (only on change)
pub fn ee_sub_1ddbd0<M: QMem + ?Sized>(m: &mut M, o: u32, b: u32) {
    if b != m.r32(o + 0x28) {
        m.w32(o + 0x28, b);
        let t = m.r32(0x002d_df0c);
        m.w32(o + 0x2c, t);
    }
}

// ---------------------------------------------------------------- C2d06a0 (parent C2d0728)
/// FUN_00178e98 (slot 6): parent init FUN_00177e80, +0xc0 = 9, +0xe0 = DAT_002be694, +0xc4 = 1.0
pub fn ee_sub_178e98<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0017_7e80, &[o], &[]);
    let v = m.r32(0x002b_e694);
    m.w32(o + 0xc0, 9);
    m.w32(o + 0xe0, v);
    m.wf(o + 0xc4, 1.0);
}

// ---------------------------------------------------------------- C2ca660 (parent C2ca5e0)
/// FUN_002564e0 (slot 0): +0 = src, FUN_0011a070(src + 4, this), +0x24 = *src
pub fn ee_sub_2564e0<M: QMem + ?Sized>(m: &mut M, o: u32, src: u32) {
    m.w32(o, src);
    m.call(0x0011_a070, &[src + 4, o], &[]);
    let v = m.r32(src);
    m.w32(o + 0x24, v);
}
/// FUN_002566b0 (slot 3): BinaryInputStream read of 4 bytes from `stream` into `dst`
pub fn ee_sub_2566b0<M: QMem + ?Sized>(m: &mut M, _o: u32, dst: u32, stream: u32) {
    m.call(0x0011_7968, &[stream, 4, dst], &[]);
}

// ---------------------------------------------------------------- C2da9d0 (parent C2c94a0)
/// FUN_0020caa8 (slot 4, message): msg 9 with event id (+8) 0x17d / 0x17e sets +0x34 = 1 / 0,
/// 0x17f / 0x180 sets +0x38 = 1 / 0; then the parent handler FUN_00133590
pub fn ee_sub_20caa8<M: QMem + ?Sized>(m: &mut M, o: u32, msg: u32, arg: u32, rest: &[u32]) {
    if msg == 9 {
        match m.r32(arg + 8) {
            0x17d => m.w32(o + 0x34, 1),
            0x17e => m.w32(o + 0x34, 0),
            0x17f => m.w32(o + 0x38, 1),
            0x180 => m.w32(o + 0x38, 0),
            _ => {}
        }
    }
    let mut a = vec![o, msg, arg];
    a.extend_from_slice(rest);
    m.call(0x0013_3590, &a, &[]);
}

// ---------------------------------------------------------------- C2cdba0 (parent C2c9a80)
/// FUN_00153d58 (slot 6): parent init FUN_00231100, FUN_001a6ca8(this, 1), +0x40 flags |= 2
pub fn ee_sub_153d58<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0023_1100, &[o], &[]);
    m.call(0x001a_6ca8, &[o, 1], &[]);
    set_tagged(m, o + 0x40, 2);
}
/// FUN_00153db0 (slot 15): parent update FUN_001a5e28, then FUN_00153de0(this)
pub fn ee_sub_153db0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001a_5e28, &[o], &[]);
    m.call(0x0015_3de0, &[o], &[]);
}

// ---------------------------------------------------------------- C2ca578 (allocator front, no parent)
/// FUN_00116a30 (slot 0): ActPool alloc FUN_001ae1d0(&{size, 0x10})
pub fn ee_sub_116a30<M: QMem + ?Sized>(m: &mut M, _o: u32, size: u32) -> u32 {
    let t = stack_push(m, 0x10);
    m.w32(t, size);
    m.w32(t + 4, 0x10);
    let r = m.call(0x001a_e1d0, &[t], &[]).0;
    stack_pop(m, 0x10);
    r
}
/// FUN_00116a58 (slot 1): forward to the object's slot at vtable(+0) +0xc with (a2, align) where
/// align = +4 unless it is -1 (then the caller's a3)
pub fn ee_sub_116a58<M: QMem + ?Sized>(m: &mut M, o: u32, _a1: u32, a2: u32, a3: u32) -> u32 {
    let al = match m.r32(o + 4) {
        0xffff_ffff => a3,
        x => x,
    };
    vcall(m, o, 0, 0xc, &[a2, al], &[]).0
}

// ---------------------------------------------------------------- C2ca550 (linear arena, no parent)
/// FUN_00116ab8 (slot 0): bump allocate `n` (rounded to 16) from base +8, used +0xc, capacity +4; 0 when full
pub fn ee_sub_116ab8<M: QMem + ?Sized>(m: &mut M, o: u32, n: u32) -> u32 {
    let used = m.r32(o + 0xc);
    let end = used.wrapping_add((n + 0xf) & 0xffff_fff0);
    if end as i32 <= m.r32(o + 4) as i32 {
        m.w32(o + 0xc, end);
        return m.r32(o + 8).wrapping_add(used);
    }
    0
}
/// FUN_00116af8 (slot 1): forward (a2, a3) to vtable(+0) +0xc
pub fn ee_sub_116af8<M: QMem + ?Sized>(m: &mut M, o: u32, _a1: u32, a2: u32, a3: u32) -> u32 {
    vcall(m, o, 0, 0xc, &[a2, a3], &[]).0
}

// ---------------------------------------------------------------- C2c9a80 (parent C2c94e0)
/// FUN_002314d0 (slot 33): quadword of the +0x50 link's frame (FUN_0022d148 into a 0x40 local):
/// {m[0x18], m[0x20], m[4], sign word of m[4]} (pextlw / pcpyld, raw disassembly; Ghidra drops the
/// return); without a link the zero vector DAT_002a2dd0.
pub fn ee_sub_2314d0<M: QMem + ?Sized>(m: &mut M, o: u32) -> V4 {
    let l = m.r32(o + 0x50);
    if l == 0 {
        return rov(0x002a_2dd0);
    }
    let t = stack_push(m, 0x40);
    m.call(0x0022_d148, &[t, l, 0], &[]);
    let (a, b, c) = (m.r32(t + 0x18), m.r32(t + 0x20), m.r32(t + 4));
    stack_pop(m, 0x40);
    let sign = if (c as i32) < 0 { u32::MAX } else { 0 };
    [f32::from_bits(a), f32::from_bits(b), f32::from_bits(c), f32::from_bits(sign)]
}

// ---------------------------------------------------------------- C2ca300 (parent C2c94e0)
/// FUN_0012f248 (slot 6): parent init FUN_001a5270, zero 0x2c bytes at +0x50 (FUN_0011a1f8), +0x78 / +0x80 /
/// +0x94 = 1.0, +0x84 / +0x88 / +0x90 = 0, +0x40 |= 0x40
pub fn ee_sub_12f248<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001a_5270, &[o], &[]);
    m.call(0x0011_a1f8, &[o + 0x50, 0x2c], &[]);
    m.wf(o + 0x78, 1.0);
    m.wf(o + 0x80, 1.0);
    m.w32(o + 0x88, 0);
    m.w32(o + 0x84, 0);
    m.wf(o + 0x94, 1.0);
    m.w32(o + 0x90, 0);
    let f = m.r32(o + 0x40);
    m.w32(o + 0x40, f | 0x40);
}
/// FUN_0012f8f0 (slot 14): parent post-load FUN_001a5c48; with +0xb0 set FUN_00149a50(world, this, DAT_002a46c8)
pub fn ee_sub_12f8f0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001a_5c48, &[o], &[]);
    if m.r32(o + 0xb0) != 0 {
        m.call(0x0014_9a50, &[WORLD, o, ro32(0x002a_46c8)], &[]);
    }
}

// ---------------------------------------------------------------- C2d5450 (parent C2d4260)
/// FUN_001c2d20 (slot 14): parent post-load FUN_001f36a8, bind table 0x2c0cb0 (4 rows, FUN_001a64e8),
/// FUN_001f1448(+0x168, this, -1, 0, this + 0x16c)
pub fn ee_sub_1c2d20<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001f_36a8, &[o], &[]);
    m.call(0x001a_64e8, &[o, 4, 0x002c_0cb0], &[]);
    let a = m.r32(o + 0x168);
    m.call(0x001f_1448, &[a, o, 0xffff_ffff, 0, o + 0x16c], &[]);
}
/// FUN_001c2d70 (slot 18, draw): parent FUN_001f3c70; with +0x158 set objects_drive_obj_draw
/// FUN_001a5ee8(1.0, 1.0, +0x158, *(+0x150) + 0x40)
pub fn ee_sub_1c2d70<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001f_3c70, &[o], &[]);
    let d = m.r32(o + 0x158);
    if d != 0 {
        let x = m.r32(o + 0x150) + 0x40;
        m.call(0x001a_5ee8, &[d, x], &[s(1.0), s(1.0)]);
    }
}

// ---------------------------------------------------------------- C2cda10 (parent C2c94e0)
/// FUN_00153b08 (slot 13, load): parent load FUN_001a5c28, u8 count -> +0x50, +0x54 = FUN_001ae2a0(count *
/// 0x50, align 0x10); per entry (stride 0x50): string (FUN_00118380, 0x2dd590, 7), u8 (FUN_00117d00) +4,
/// s32 +8, s32 +0xc, 64-byte block (FUN_00118458) +0x10
pub fn ee_sub_153b08<M: QMem + ?Sized>(m: &mut M, o: u32, st: u32) {
    m.call(0x001a_5c28, &[o, st], &[]);
    let n = m.call(0x0011_7bd0, &[st], &[]).0;
    let t = stack_push(m, 0x10);
    m.w32(t, n.wrapping_mul(0x50));
    m.w32(t + 4, 0x10);
    m.w32(o + 0x50, n);
    let p = m.call(0x001a_e2a0, &[t], &[]).0;
    stack_pop(m, 0x10);
    m.w32(o + 0x54, p);
    let mut i = 0;
    while (i as i32) < m.r32(o + 0x50) as i32 {
        let e = m.r32(o + 0x54) + i * 0x50;
        m.call(0x0011_8380, &[st, e, 0x002d_d590, 7], &[]);
        let v = m.call(0x0011_7d00, &[st], &[]).0;
        m.w32(e + 4, v);
        let v = m.call(0x0011_7db8, &[st], &[]).0;
        m.w32(e + 8, v);
        let v = m.call(0x0011_7db8, &[st], &[]).0;
        m.w32(e + 0xc, v);
        m.call(0x0011_8458, &[st, e + 0x10], &[]);
        i += 1;
    }
}

// ---------------------------------------------------------------- C2d9f70 (parent C2c9a80)
/// FUN_00203568 (slot 6): parent init FUN_00231100, +0xf8 = -2, +0xfc / +0xf4 / +0x110 = -1
pub fn ee_sub_203568<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0023_1100, &[o], &[]);
    m.w32(o + 0xf8, 0xffff_fffe);
    m.w32(o + 0xfc, 0xffff_ffff);
    m.w32(o + 0xf4, 0xffff_ffff);
    m.w32(o + 0x110, 0xffff_ffff);
}
/// FUN_00203638 (slot 14): parent post-load FUN_001a5c48, then FUN_001333c0(+0xec, this) when set
pub fn ee_sub_203638<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001a_5c48, &[o], &[]);
    let a = m.r32(o + 0xec);
    if a != 0 {
        m.call(0x0013_33c0, &[a, o], &[]);
    }
}
/// FUN_00203670 (slot 4, message): parent FUN_00133590; msg 0x11 with a player (FUN_001c4840(world)) that is a
/// class 0x34 (FUN_00132b10) and has +0xf38: FUN_00202030(), FUN_002021f8(+0xf38)
pub fn ee_sub_203670<M: QMem + ?Sized>(m: &mut M, o: u32, msg: u32, rest: &[u32]) {
    let mut a = vec![o, msg];
    a.extend_from_slice(rest);
    m.call(0x0013_3590, &a, &[]);
    if msg != 0x11 {
        return;
    }
    let p = m.call(0x001c_4840, &[WORLD], &[]).0;
    if p == 0 || m.call(0x0013_2b10, &[p, 0x34], &[]).0 == 0 {
        return;
    }
    let x = m.r32(p + 0xf38);
    if x != 0 {
        m.call(0x0020_2030, &[], &[]);
        m.call(0x0020_21f8, &[x], &[]);
    }
}

// ---------------------------------------------------------------- C2d7188 (parent C2c8940)
/// FUN_001cbee0 (slot 88): for a target (`t` +0x14 record) with +0x50 == 0 and a +0x40 record of kind
/// 0x246, temporarily doubles +0x54 and widens the +0x58/+0x5c range {lo, hi} (h = (hi - lo) / 2:
/// lo' = lo + h - 2h, hi' = lo + 3h) around the parent slot FUN_00208be0, then restores the three words.
pub fn ee_sub_1cbee0<M: QMem + ?Sized>(m: &mut M, o: u32, t: u32) {
    let r = m.r32(t + 0x14);
    let (lo_b, hi_b) = (m.r32(r + 0x58), m.r32(r + 0x5c));
    let f = m.rf(r + 0x54);
    let k = m.r32(t + 0x40);
    if m.r32(r + 0x50) == 0 && k != 0 && m.r32(k + 0x18) == 0x246 {
        let (lo, hi) = (f32::from_bits(lo_b), f32::from_bits(hi_b));
        m.wf(r + 0x54, f + f);
        let h = (hi - lo) * 0.5;
        m.wf(r + 0x58, (lo + h) - (h + h));
        m.wf(r + 0x5c, lo + h + h + h);
    }
    m.call(0x0020_8be0, &[o, t], &[]);
    m.w32(r + 0x58, lo_b);
    m.w32(r + 0x5c, hi_b);
    m.wf(r + 0x54, f);
}
/// FUN_001cbf90 (slot 89): +0x1010 = `on`; on change push (FUN_0014e328(0x2ddf40, 6, 0x2de540, DAT_002f7900))
/// or revoke (FUN_0014e4c8(0x2ddf40, 1, 6, 0, 0, 0)) camera policy 6
pub fn ee_sub_1cbf90<M: QMem + ?Sized>(m: &mut M, o: u32, on: u32) {
    if m.r32(o + 0x1010) == on {
        return;
    }
    m.w32(o + 0x1010, on);
    if on == 0 {
        m.call(0x0014_e4c8, &[0x002d_df40, 1, 6, 0, 0, 0], &[]);
    } else {
        let p = m.r32(0x002f_7900);
        m.call(0x0014_e328, &[0x002d_df40, 6, 0x002d_e540, p], &[]);
    }
}

// ---------------------------------------------------------------- C2d28d8 (no parent; owner vtable at +0x20)
/// FUN_00182cc8 (slot 1): *out = 0
pub fn ee_sub_182cc8<M: QMem + ?Sized>(m: &mut M, _o: u32, _a: u32, out: u32) {
    m.w32(out, 0);
}
/// FUN_00182d78 (slot 4): random point around the owner (vtable(+0x20) +0x34): angle FUN_0011d448(-pi, pi),
/// s = rand(-1, 1), r = rand(0, DAT_002be8b0); *out_p = FUN_001a62e8(owner) + FUN_0011e208(angle,
/// |s| * s * pi/2, r) (vadd); *out_v = rodata 0x2a2dc0
pub fn ee_sub_182d78<M: QMem + ?Sized>(m: &mut M, o: u32, out_p: u32, out_v: u32) {
    let ow = vcall(m, o, 0x20, 0x34, &[], &[]).0;
    let ang = callf(m, 0x0011_d448, &[], &[s(f32::from_bits(0xc049_0fdb)), s(f32::from_bits(0x4049_0fdb))]);
    let sv = callf(m, 0x0011_d448, &[], &[s(-1.0), s(1.0)]);
    let rmax = m.rf(0x002b_e8b0);
    let r = callf(m, 0x0011_d448, &[], &[s(0.0), s(rmax)]);
    let p = m.call(0x001a_62e8, &[ow], &[]).1;
    let d = m.call(0x0011_e208, &[], &[s(ang), s(sv.abs() * sv * 1.570_796_4), s(r)]).1;
    m.wv(out_p, [p[0] + d[0], p[1] + d[1], p[2] + d[2], p[3] + d[3]]);
    for i in 0..4 {
        m.w32(out_v + 4 * i, ro32(0x002a_2dc0 + 4 * i));
    }
}

// ---------------------------------------------------------------- C2da3d8 (parent C2c94e0)
/// FUN_0020b4a8 (slot 6): parent init FUN_001a5270, +0x40 |= 2, FUN_001a6ca8(this, 1)
pub fn ee_sub_20b4a8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001a_5270, &[o], &[]);
    set_tagged(m, o + 0x40, 2);
    m.call(0x001a_6ca8, &[o, 1], &[]);
}
/// FUN_0020b500 (slot 14): parent post-load FUN_001a5c48, then the list of class-0x38 objects: count =
/// FUN_0015e9c8(world, 0x605, 0x38, 0, 0, 0) -> +0x50, array (FUN_001ae1d0 count * 4) -> +0x54, filled by
/// FUN_0015e9c8(world, 0x405, 0x38, 0, count, array)
pub fn ee_sub_20b500<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001a_5c48, &[o], &[]);
    let n = m.call(0x0015_e9c8, &[WORLD, 0x605, 0x38, 0, 0, 0], &[]).0;
    let t = stack_push(m, 0x10);
    m.w32(t, n << 2);
    m.w32(t + 4, 0x10);
    m.w32(o + 0x50, n);
    let a = m.call(0x001a_e1d0, &[t], &[]).0;
    stack_pop(m, 0x10);
    m.w32(o + 0x54, a);
    let n = m.r32(o + 0x50);
    m.call(0x0015_e9c8, &[WORLD, 0x405, 0x38, 0, n, a], &[]);
}
/// FUN_0020b590 (slot 15): parent update FUN_001a5e28; for each listed guard with flag bit 60 (+0x28) and
/// clock - +0xff8 > DAT_002c3dbc: kind (+0x1000) 1 -> vtable +0x4c slot, 2 -> component 0x9c (FUN_00205458)
/// gets a zeroed 0x70 message {1} (FUN_00176750) and with +0xf64 FUN_00153ff8(); every passing guard
/// gets +0xff8 = DAT_002a88cc
pub fn ee_sub_20b590<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001a_5e28, &[o], &[]);
    let reset = ro32(0x002a_88cc);
    let n = m.r32(o + 0x50) as i32;
    let arr = m.r32(o + 0x54);
    for i in 0..n.max(0) as u32 {
        let g = m.r32(arr + 4 * i);
        if (m.r64(g + 0x28) >> 60) & 1 == 0 {
            continue;
        }
        let lim = m.rf(0x002c_3dbc);
        if !(lim < m.rf(CLOCK) - m.rf(g + 0xff8)) {
            continue;
        }
        match m.r32(g + 0x1000) as i32 {
            1 => {
                vcall(m, g, 0xc, 0x4c, &[], &[]);
            }
            2 => {
                let c = m.call(0x0020_5458, &[g, 0x9c], &[]).0;
                if c != 0 {
                    let t = stack_push(m, 0x70);
                    m.call(0x0023_33b4, &[t, 0, 0x70], &[]);
                    m.w32(t, 1);
                    m.call(0x0017_6750, &[c, t], &[]);
                    stack_pop(m, 0x70);
                }
                if m.r32(g + 0xf64) != 0 {
                    m.call(0x0015_3ff8, &[], &[]);
                }
            }
            _ => {}
        }
        m.w32(g + 0xff8, reset);
    }
}
