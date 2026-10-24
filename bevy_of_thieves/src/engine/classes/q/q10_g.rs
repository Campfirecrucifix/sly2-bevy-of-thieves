//! WP-Q10 file packages (run_001d9280, run_00211a80 tail, ...): world/pool teardown helpers ported 1:1.
//! Hidden register args checked in the disassembly (e.g. FUN_00212158 and FUN_0011a380 get a0).
use crate::engine::classes::q::q2mem::{vslot, QMem};

/// FUN_00211c58(o) -> o
pub fn fun_00211c58<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    m.call(0x0021_2158, &[o], &[]);
    m.w32(o + 0x120, u32::MAX);
    m.w32(o + 0xc, 0x002c_9670);
    for off in [0x110, 0x114, 0x118, 0x11c] {
        m.w32(o + off, u32::MAX);
    }
    o
}
/// FUN_001d93d8(o) -> o
pub fn fun_001d93d8<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    m.call(0x0011_a1f8, &[o, 0x14], &[]);
    o
}
/// FUN_001d9408(list): walk the intrusive list (head list[0], link offset list[2]) calling virtual slot 10 on each
/// node (next pointer read after the call), then tear down the world containers
pub fn fun_001d9408<M: QMem + ?Sized>(m: &mut M, list: u32) {
    let off = m.r32(list + 8);
    let mut node = m.r32(list);
    let mut next_at = node.wrapping_add(off);
    while node != 0 {
        vslot(m, node, 10, &[], &[]);
        node = m.r32(next_at);
        next_at = node.wrapping_add(off);
    }
    m.w32(0x002d_d318, 0);
    for (f, a) in [
        (0x001d_9b30, 0x3d_9e80),
        (0x001a_e970, 0x3d_9ed0),
        (0x001a_3070, 0x2e_1e60),
        (0x001d_aa48, 0x3d_a0f0),
        (0x001d_b7f0, 0x3d_a150),
        (0x001d_a320, 0x3d_a0c0),
        (0x001d_ccd8, 0x3d_a3c0),
        (0x001a_cb38, 0x2e_6550),
        (0x001d_a7b0, 0x2e_65a0),
        (0x001d_a7b0, 0x3d_a4f0),
        (0x001d_a7b0, 0x3d_a530),
    ] {
        m.call(f, &[a], &[]);
    }
}
/// FUN_001d9520()
pub fn fun_001d9520<M: QMem + ?Sized>(m: &mut M) {
    for (f, a) in [
        (0x001d_9968, 0x3d_9e80),
        (0x001a_e928, 0x3d_9ed0),
        (0x001a_3020, 0x2e_1e60),
        (0x001d_a770, 0x3d_a0f0),
        (0x001d_a770, 0x3d_a150),
        (0x001d_a2e0, 0x3d_a0c0),
        (0x001d_cca0, 0x3d_a3c0),
        (0x001a_cae8, 0x2e_6550),
        (0x001d_a770, 0x2e_65a0),
        (0x001d_d690, 0x3d_a4f0),
        (0x001d_a770, 0x3d_a530),
    ] {
        m.call(f, &[a], &[]);
    }
}
/// FUN_001d9820()
pub fn fun_001d9820<M: QMem + ?Sized>(m: &mut M) {
    for a in [0x3d_9e80, 0x3d_a0f0, 0x3d_a150] {
        m.call(0x001d_9878, &[a], &[]);
    }
    m.call(0x0015_74a0, &[0x2d_ea00], &[]);
    m.call(0x0015_74a0, &[0x2d_ea28], &[]);
}
/// FUN_001d9880(a0)
pub fn fun_001d9880<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0011_a380, &[0x3d_9e70, a0], &[]);
}
/// FUN_001d9b30(o)
pub fn fun_001d9b30<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001d_9b60, &[o, 0], &[]);
    m.w32(o + 0x38, 0);
}

use crate::engine::classes::q::q2mem::{stack_pop, stack_push};

const WORLD_UI: u32 = 0x002d_eb30;

/// FUN_0020df80(o, src): o[+0x3bf0] = 1; copy 0x4c bytes (8 dwords + 12 bytes) src -> o + 0x3bf4 (both decompiled branches are the same copy)
pub fn fun_0020df80<M: QMem + ?Sized>(m: &mut M, o: u32, src: u32) {
    m.w32(o + 0x3bf0, 1);
    for i in 0..0x4c {
        let b = m.r8(src + i);
        m.w8(o + 0x3bf4 + i, b);
    }
}

/// FUN_0020e068(o): level teardown (FUN_00162ea0 gets a0 = o[+0x3b18], seen in the disassembly)
pub fn fun_0020e068<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0016_19a0, &[WORLD_UI, 1, 0], &[]);
    let h = m.r32(o + 0x3b18);
    if h != 0 {
        m.call(0x0016_2ea0, &[h], &[]);
        m.w32(o + 0x3b18, 0);
    }
    m.call(0x001d_9408, &[0x3d_9e70], &[]);
    m.call(0x001f_f830, &[o], &[]);
    m.call(0x0016_1458, &[WORLD_UI], &[]);
    m.call(0x0014_d650, &[0x2d_df40], &[]);
    m.call(0x001f_d700, &[0x3d_d040], &[]);
    m.call(0x001e_c7e8, &[], &[]);
    m.call(0x001d_53c0, &[0x2e_0ccc, 3], &[]);
    for a in [
        0x2e_1e40, 0x2f_7900, 0x2d_d5bc, 0x2d_d648, 0x3d_4a54, 0x2d_e570, 0x2d_fb80, 0x2d_fbc4, 0x3e_1c40, 0x3e_1c44,
        0x3e_1c48, 0x3e_1c4c, 0x3e_1c50, 0x3d_d48c,
    ] {
        m.w32(a, 0);
    }
    m.call(0x001a_e300, &[], &[]);
}

/// FUN_0020e180(o): allocate the three 0x4800 buffers (o[+0x3cd4..]), 0x8000 (o[+0x3f20]), the 0x28-byte
/// record o[+0x3f0c], init two entries at o + 0x45e8 (stride 0x28) from a FUN_001703f8(buf, 0x1400, 0x1800) template
pub fn fun_0020e180<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let sp = stack_push(m, 0x640);
    for i in 0..3 {
        m.w32(sp, 0x4800);
        m.w32(sp + 4, 0x10);
        let r = m.call(0x001a_e1d0, &[sp], &[]).0;
        m.w32(o + 0x3cd4 + 4 * i, r);
    }
    m.w32(sp, 0x8000);
    m.w32(sp + 4, 0x10);
    let r = m.call(0x001a_e2a0, &[sp], &[]).0;
    m.w32(sp + 4, 0x10);
    m.w32(sp, 0x28);
    m.w32(o + 0x3f20, r);
    let p = m.call(0x001a_e2a0, &[sp], &[]).0;
    m.w32(o + 0x3f0c, p);
    m.w32(p, u32::MAX);
    let p = m.r32(o + 0x3f0c);
    m.w32(p + 8, u32::MAX);
    m.call(0x0011_a050, &[p + 0x18, 0x14], &[]);
    let buf = sp + 0x10;
    m.call(0x0017_03f8, &[buf, 0x1400, 0x1800], &[]);
    for i in 0..2 {
        m.call(0x001b_e398, &[o + 0x45e8 + 0x28 * i, buf], &[]);
    }
    stack_pop(m, 0x640);
    if m.r32(o + 0x3bf4) & 0x20 != 0 {
        m.w32(o + 0x3e38, 1);
    }
    m.call(0x0016_19a0, &[WORLD_UI, 2, 0], &[]);
}

/// FUN_0020e2b0(o): of the (up to 16) objects FUN_0015e9c8(o, 5, 0x31, 0, 0x10, buf) returns, pick the one with the
/// largest (strictly greater, > 0) +0xb8; o[+0x3e5c] = pick[+0xc8]; FUN_001a6d28(pick, 0xff)
pub fn fun_0020e2b0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let buf = stack_push(m, 0x40);
    let mut n = m.call(0x0015_e9c8, &[o, 5, 0x31, 0, 0x10, buf], &[]).0 as i32;
    let (mut best, mut best_v) = (0u32, 0i32);
    let mut p = buf;
    while n > 0 {
        let e = m.r32(p);
        let v = m.r32(e + 0xb8) as i32;
        n -= 1;
        if best_v < v {
            best = e;
            best_v = v;
        }
        p += 4;
    }
    stack_pop(m, 0x40);
    if best != 0 {
        let v = m.r32(best + 200);
        m.w32(o + 0x3e5c, v);
        m.call(0x001a_6d28, &[best, 0xff], &[]);
    }
}

/// FUN_0020e338(o): o[+0x3e60] = FUN_0013e3c8(0x74, 0, -1, -1) with +0x60 = 0.1, +0x64 = 0.25, +0x54 = 0x80200000,
/// +0x6c = 1, then virtual slot 8 (0x4c)
pub fn fun_0020e338<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let n = m.call(0x0013_e3c8, &[0x74, 0, u32::MAX, u32::MAX], &[]).0;
    m.w32(o + 0x3e60, n);
    m.w32(n + 0x60, 0x3dcc_cccd);
    m.w32(n + 0x64, 0x3e80_0000);
    m.w32(n + 0x54, 0x8020_0000);
    m.w32(n + 0x6c, 1);
    vslot(m, n, 8, &[], &[]);
}

/// FUN_0020e740(o): per-level init pass, then drain the pending list [+0x39e0, +0x39e4) via virtual slot 14 (0x7c)
pub fn fun_0020e740<M: QMem + ?Sized>(m: &mut M, o: u32) {
    for off in [0x403c, 0x4054, 0x4070, 0x4088] {
        m.call(0x0011_cdd8, &[o + off], &[]);
    }
    for f in [
        0x0020_e338, 0x0020_e2b0, 0x001f_f828, 0x001c_a500, 0x0020_f710, 0x0015_69b8, 0x001a_3260, 0x001c_10c8,
        0x0021_0940, 0x0019_57c0,
    ] {
        match f {
            0x0020_e338 => fun_0020e338(m, o),
            0x0020_e2b0 => fun_0020e2b0(m, o),
            _ => {
                m.call(f, &[o], &[]);
            }
        }
    }
    m.call(0x001d_9520, &[0x3d_9e70], &[]);
    loop {
        m.call(0x0014_9c80, &[o], &[]);
        let b = m.r32(o + 0x39e0);
        let e = m.r32(o + 0x39e4);
        if e == b {
            break;
        }
        if (b as i32) < (e as i32) {
            let mut p = b;
            let mut x = m.r32(p);
            loop {
                p += 4;
                vslot(m, x, 14, &[], &[]);
                if (m.r32(o + 0x39e4) as i32) <= p as i32 {
                    break;
                }
                x = m.r32(p);
            }
        }
        let v = m.r32(o + 0x39e0);
        m.w32(o + 0x39e4, v);
    }
    m.w32(o + 0x39e4, 0);
    m.w32(o + 0x39e0, 0);
    m.w32(o + 0x39d8, 0);
    m.w32(o + 0x39d0, 0);
    let a = m.r32(o + 0x45cc);
    m.call(0x001c_1260, &[a], &[]);
    m.call(0x0014_d470, &[0x2d_df40], &[]);
    m.call(0x0016_19a0, &[WORLD_UI, 4, 0], &[]);
}

/// FUN_0013f888(o) -> o: vtable DAT_002cbe50, three zeroed words
pub fn fun_0013f888<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    m.w32(o + 4, 0);
    m.w32(o, 0x002c_be50);
    m.w32(o + 8, 0);
    m.w32(o + 12, 0);
    o
}

/// FUN_0013f8a8(v, n, src): append n words to the growable word array {+4 capacity, +8 count, +0xc data}
/// (capacity doubles from a minimum of 0x20; FUN_001ae318 allocator, FUN_001ae5a0 copy)
pub fn fun_0013f8a8<M: QMem + ?Sized>(m: &mut M, v: u32, n: u32, src: u32) {
    let mut cap = m.r32(v + 4) as i32;
    let need = (m.r32(v + 8) as i32).wrapping_add(n as i32);
    if cap < need {
        loop {
            let d = cap.wrapping_shl(1);
            cap = if d > 0x1f { d } else { 0x20 };
            if cap >= need {
                break;
            }
        }
        let sp = stack_push(m, 0x10);
        m.w32(sp, (cap as u32) << 2);
        m.w32(sp + 4, 0x10);
        let new = m.call(0x001a_e318, &[sp], &[]).0;
        stack_pop(m, 0x10);
        let (old, cnt) = (m.r32(v + 0xc), m.r32(v + 8));
        m.call(0x001a_e5a0, &[new, old, cnt << 2], &[]);
        m.w32(v + 4, cap as u32);
        m.w32(v + 0xc, new);
    }
    let dst = m.r32(v + 0xc).wrapping_add(m.r32(v + 8).wrapping_mul(4));
    m.call(0x001a_e5a0, &[dst, src, n << 2], &[]);
    let c = m.r32(v + 8).wrapping_add(n);
    m.w32(v + 8, c);
}

/// FUN_0013f990(o, k): bits 48..51 of the u64 at o+0x28 = k & 0xf; recurse over the child chain o[+0x64] (next +0x24)
/// when FUN_00132b10(o, 0x30) is set
pub fn fun_0013f990<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32) {
    let hi = (m.r32(o + 0x2c) & 0xfff0_ffff) | ((k & 0xf) << 16);
    m.w32(o + 0x2c, hi);
    if m.call(0x0013_2b10, &[o, 0x30], &[]).0 == 0 {
        return;
    }
    let mut c = m.r32(o + 0x64);
    while c != 0 {
        fun_0013f990(m, c, k);
        c = m.r32(c + 0x24);
    }
}

/// FUN_0013fa30(v, n, list): for each of n objects: FUN_001a58d0(o, 0, 0); FUN_0013f990(o, v[+4]); when v[+8] != -1
/// virtual slot 24 (0xcc) with (v[+8], 1)
pub fn fun_0013fa30<M: QMem + ?Sized>(m: &mut M, v: u32, mut n: i32, mut list: u32) {
    while n > 0 {
        let o = m.r32(list);
        m.call(0x001a_58d0, &[o, 0, 0], &[]);
        let k = m.r32(v + 4);
        fun_0013f990(m, o, k);
        let a = m.r32(v + 8);
        if a != u32::MAX {
            vslot(m, o, 24, &[a, 1], &[]);
        }
        n -= 1;
        list += 4;
    }
}

/// FUN_00140338(): when FUN_00210ae0(): FUN_00161868(world); FUN_0015e9c8(world, 0x605, 0x5f, 0, 0, 0) != 0 ->
/// FUN_001618b0(world); FUN_001fb2f8(FUN_001fbb90(0x32), 0); world+0x4754 = 1.0; counter DAT_002de3e4++
pub fn fun_00140338<M: QMem + ?Sized>(m: &mut M) {
    const W: u32 = 0x003d_d490;
    if m.call(0x0021_0ae0, &[], &[]).0 == 0 {
        return;
    }
    m.call(0x0016_1868, &[W], &[]);
    if m.call(0x0015_e9c8, &[W, 0x605, 0x5f, 0, 0, 0], &[]).0 != 0 {
        m.call(0x0016_18b0, &[W], &[]);
    }
    let r = m.call(0x001f_bb90, &[0x32], &[]).0;
    m.call(0x001f_b2f8, &[r, 0], &[]);
    let one = m.r32(0x002d_ded0);
    m.w32(W + 0x4754, one);
    // LOOK R39: mirror DAT_003e1be4 into ClassGlobals.g_3e1be4 (K12 marker fade)
    crate::engine::classes::k12::stamp_job_t();
    let c = m.r32(0x002d_e3e4).wrapping_add(1);
    m.w32(0x002d_e3e4, c);
}
