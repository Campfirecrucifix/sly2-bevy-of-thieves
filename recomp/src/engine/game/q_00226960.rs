//! WP-Q4: run 0x00226960..0x00228518 - the VIF code builder over the DMA packet of FUN_00157278 (packet
//! +0xc cursor, +0x1c current STMASK, +0x20; VIF codes: BASE 0x03, OFFSET 0x02, CYCLE 0x01, MSCAL 0x14,
//! FLUSH 0x11, FLUSHA 0x13, STROW 0x30, STMASK 0x20, DIRECT 0x50, UNPACK 0x60..0x7f), the BSP point / prune
//! tests of the C2... volume (+0xe4 face count, +0xec / +0xf4 BSP, +0x110 inside flag, +0x114 stamp),
//! the HUD key-colour records ({id, colour, 0}) and small forwarders.
//! The builders only write VIF words into EE memory: game logic, ported 1:1 (the Bevy renderer reads the
//! scene directly; these keep the callers' packet layout identical).
//! 1:1 against `QMem` (classes::q::q2mem); callees outside the run go through `QMem::call` by address.
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};
use crate::engine::game::q_00156390::{ee_sub_157278, ee_sub_157500, ee_sub_157660};
use crate::engine::game::q_001e9090::ee_sub_1ea410;

/// DAT_002c4430: bytes per element of each UNPACK format (vn / vl)
pub const UNPACK_SIZE: u32 = 0x002c_4430;
/// DAT_002dded0: game clock
pub const NOW: u32 = 0x002d_ded0;

/// FUN_00226960 / 226ab8 / 2271f8 / 227460 / 227a60 / 227e90 (identical static pool inits) = FUN_001ea410
pub fn ee_sub_226960<M: QMem + ?Sized>(m: &mut M, init: u32, prio: u32) {
    ee_sub_1ea410(m, init, prio);
}
/// FUN_00226a98 / 226bf0 / 227330 / 227598 / 227b98 / 227fc8: (1, 0xffff)
pub fn ee_sub_226a98<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_1ea410(m, 1, 0xffff);
}

fn put<M: QMem + ?Sized>(m: &mut M, p: u32, v: u32) {
    let c = m.r32(p + 0xc);
    m.w32(p + 0xc, c.wrapping_add(4));
    m.w32(c, v);
}

/// FUN_00226c10(pkt): FUN_00157278, +0x10 = 1 (pad on close), +0x18 = &DAT_002ce488, +0x1c = +0x20 = 0
pub fn ee_sub_226c10<M: QMem + ?Sized>(m: &mut M, p: u32) -> u32 {
    ee_sub_157278(m, p);
    m.w32(p + 0x10, 1);
    m.w32(p + 0x18, 0x002c_e488);
    m.w32(p + 0x1c, 0);
    m.w32(p + 0x20, 0);
    p
}

/// FUN_00226c58(pkt, k): zero words until the cursor's word index in its qword ((c & 0xc) >> 2) is k
pub fn ee_sub_226c58<M: QMem + ?Sized>(m: &mut M, p: u32, k: u32) {
    let mut c = m.r32(p + 0xc);
    let mut i = (c & 0xc) >> 2;
    while i != k {
        m.w32(c, 0);
        c = c.wrapping_add(4);
        i = (i + 1) & 3;
    }
    m.w32(p + 0xc, c);
}

/// FUN_00226ca8(pkt, base, offset): BASE base, OFFSET offset
pub fn ee_sub_226ca8<M: QMem + ?Sized>(m: &mut M, p: u32, base: u32, off: u32) {
    put(m, p, base | 0x0300_0000);
    put(m, p, off | 0x0200_0000);
}

/// FUN_00226ce0(pkt, addr): MSCAL addr / 8 (signed)
pub fn ee_sub_226ce0<M: QMem + ?Sized>(m: &mut M, p: u32, addr: i32) {
    put(m, p, (addr / 8) as u32 | 0x1400_0000);
}

/// FUN_00226d10(pkt): FLUSH
pub fn ee_sub_226d10<M: QMem + ?Sized>(m: &mut M, p: u32) {
    put(m, p, 0x1100_0000);
}

/// FUN_00226d28(pkt): FLUSHA
pub fn ee_sub_226d28<M: QMem + ?Sized>(m: &mut M, p: u32) {
    put(m, p, 0x1300_0000);
}

/// FUN_00226d40(pkt, wl, cl): STCYCL cl | wl << 8
pub fn ee_sub_226d40<M: QMem + ?Sized>(m: &mut M, p: u32, wl: u32, cl: u32) {
    put(m, p, cl | wl << 8 | 0x0100_0000);
}

/// FUN_00226d68(pkt, row): STROW + the four row words
pub fn ee_sub_226d68<M: QMem + ?Sized>(m: &mut M, p: u32, row: u32) {
    put(m, p, 0x3000_0000);
    for k in 0..4 {
        let v = m.r32(row + 4 * k);
        put(m, p, v);
    }
}

/// FUN_00226dd0(pkt, mask): STMASK mask (only when non-zero), +0x1c = mask
pub fn ee_sub_226dd0<M: QMem + ?Sized>(m: &mut M, p: u32, mask: u32) {
    if mask != 0 {
        put(m, p, 0x2000_0000);
        put(m, p, mask);
    }
    m.w32(p + 0x1c, mask);
}

/// FUN_00226e10(pkt, fmt, n, addr): UNPACK word (fmt < 0x10: FLG | USN 0x6000c000, else fmt & 0xf with
/// FLG 0x60008000) | fmt << 24 | n << 16 | addr, masked (| 0x10000000) while +0x1c is set; returns the
/// byte size DAT_002c4430[fmt] * n
pub fn ee_sub_226e10<M: QMem + ?Sized>(m: &mut M, p: u32, fmt: u32, n: u32, addr: u32) -> i32 {
    let (fmt, base) = if (fmt as i32) < 0x10 { (fmt, 0x6000_c000) } else { (fmt & 0xf, 0x6000_8000) };
    let mut w = base | fmt << 24 | n << 16 | addr;
    if m.r32(p + 0x1c) != 0 {
        w |= 0x1000_0000;
    }
    put(m, p, w);
    (m.r32(UNPACK_SIZE + fmt * 4) as i32).wrapping_mul(n as i32)
}

/// FUN_00226e88(pkt, fmt, n, addr, out_bytes, out_ptr): n == 0 -> both outs 0; else UNPACK
/// (FUN_00226e10), *out_ptr = data start, cursor += bytes rounded up to words, *out_bytes = bytes
pub fn ee_sub_226e88<M: QMem + ?Sized>(m: &mut M, p: u32, fmt: u32, n: u32, addr: u32, ob: u32, op: u32) {
    if n == 0 {
        if op != 0 {
            m.w32(op, 0);
        }
        if ob != 0 {
            m.w32(ob, 0);
        }
        return;
    }
    let b = ee_sub_226e10(m, p, fmt, n, addr);
    if op != 0 {
        let c = m.r32(p + 0xc);
        m.w32(op, c);
    }
    let w = (b.wrapping_add(3) / 4) as u32;
    let c = m.r32(p + 0xc);
    m.w32(p + 0xc, c.wrapping_add(w.wrapping_mul(4)));
    if ob != 0 {
        m.w32(ob, b as u32);
    }
}

/// FUN_00226f28(pkt, fmt, n, src, addr): UNPACK (FUN_00226e88) and copy (bytes + 3) >> 2 words from src
pub fn ee_sub_226f28<M: QMem + ?Sized>(m: &mut M, p: u32, fmt: u32, n: u32, src: u32, addr: u32) {
    let sp = stack_push(m, 0x10);
    ee_sub_226e88(m, p, fmt, n, addr, sp, sp + 4);
    let words = (m.r32(sp) as i32).wrapping_add(3) >> 2;
    let mut d = m.r32(sp + 4);
    stack_pop(m, 0x10);
    // VFIX: asm order (matters when src/dst overlap): words & 3 singles, then 4-word blocks (load 4, store 4)
    let mut s = src;
    let rem = words & 3;
    for _ in 0..rem {
        let v = m.r32(s);
        m.w32(d, v);
        s = s.wrapping_add(4);
        d = d.wrapping_add(4);
    }
    let mut i = 0;
    while i < words - rem {
        let q = [m.r32(s), m.r32(s + 4), m.r32(s + 8), m.r32(s + 12)];
        for (k, v) in q.into_iter().enumerate() {
            m.w32(d + 4 * k as u32, v);
        }
        s = s.wrapping_add(16);
        d = d.wrapping_add(16);
        i += 4;
    }
}

/// FUN_00226fd0(pkt, fmt, n, stream, addr): UNPACK (FUN_00226e88) whose data is read from `stream`
/// (FUN_00117968(stream, bytes, dst))
pub fn ee_sub_226fd0<M: QMem + ?Sized>(m: &mut M, p: u32, fmt: u32, n: u32, stream: u32, addr: u32) {
    let sp = stack_push(m, 0x10);
    ee_sub_226e88(m, p, fmt, n, addr, sp, sp + 4);
    let (b, d) = (m.r32(sp), m.r32(sp + 4));
    stack_pop(m, 0x10);
    m.call(0x0011_7968, &[stream, b, d], &[]);
}

/// FUN_00227010(pkt, fmt, n, src, addr, out): n != 0 -> pad to word 3, UNPACK, ref tag of (bytes + 15) / 16
/// qwords at src (FUN_00157500); *out = the open cnt tag (+8) - 0xc (the UNPACK word)
pub fn ee_sub_227010<M: QMem + ?Sized>(m: &mut M, p: u32, fmt: u32, n: u32, src: u32, addr: u32, out: u32) {
    if n == 0 {
        return;
    }
    ee_sub_226c58(m, p, 3);
    let b = ee_sub_226e10(m, p, fmt, n, addr);
    ee_sub_157500(m, p, (b.wrapping_add(0xf) / 16) as u32, src);
    if out != 0 {
        let t = m.r32(p + 8);
        m.w32(out, t.wrapping_sub(0xc));
    }
}

/// FUN_002270c8(pkt, qwc, src, irq): pad to word 3, DIRECT qwc (| irq << 31), copy qwc qwords (FUN_00157660)
pub fn ee_sub_2270c8<M: QMem + ?Sized>(m: &mut M, p: u32, qwc: u32, src: u32, irq: u32) {
    ee_sub_226c58(m, p, 3);
    put(m, p, qwc | irq << 31 | 0x5000_0000);
    ee_sub_157660(m, p, qwc, src);
}

/// FUN_00227148(pkt, qwc, src, irq): DIRECT like FUN_002270c8 but referencing src (FUN_00157500)
pub fn ee_sub_227148<M: QMem + ?Sized>(m: &mut M, p: u32, qwc: u32, src: u32, irq: u32) {
    ee_sub_226c58(m, p, 3);
    put(m, p, qwc | irq << 31 | 0x5000_0000);
    ee_sub_157500(m, p, qwc, src);
}

/// FUN_002271c8(pkt, other): DIRECT copy of the other packet's contents
pub fn ee_sub_2271c8<M: QMem + ?Sized>(m: &mut M, p: u32, o: u32) {
    let (c, b) = (m.r32(o + 0xc), m.r32(o + 4));
    ee_sub_2270c8(m, p, c.wrapping_sub(b) >> 4, b, 0);
}

/// FUN_002273f0(vol, pos): inside = PointInBsp(vol +0xf4, 0x7fffffff, FUN_001a6f00(0, vol, pos)) != 0
/// -> FUN_00227440
pub fn ee_sub_2273f0<M: QMem + ?Sized>(m: &mut M, o: u32, pos: V4) {
    let l = m.call(0x001a_6f00, &[0, o], &[pos]).1;
    let b = m.r32(o + 0xf4);
    let r = m.call(0x0011_8fb0, &[b, 0x7fff_ffff], &[l]).0;
    ee_sub_227440(m, o, (r != 0) as u32);
}

/// FUN_00227440(vol, v): +0x110 changes -> +0x110 = v, +0x114 = now
pub fn ee_sub_227440<M: QMem + ?Sized>(m: &mut M, o: u32, v: u32) {
    if v != m.r32(o + 0x110) {
        m.w32(o + 0x110, v);
        let t = m.r32(NOW);
        m.w32(o + 0x114, t);
    }
}

/// FUN_002275b8 (colour part): HUD key colour of kind k
pub fn key_colour(k: u32) -> u32 {
    match k {
        0x35 => 0x7f99_6633,
        0x36 => 0x7f66_3399,
        0x37 => 0x7f33_9933,
        0x3c => 0x7f99_3399,
        0x3f => 0x7f33_3399,
        _ => 0x7f66_6666,
    }
}

/// FUN_002275b8(out, k): *out = key_colour(k)
pub fn ee_sub_2275b8<M: QMem + ?Sized>(m: &mut M, out: u32, k: u32) -> u32 {
    m.w32(out, key_colour(k));
    out
}

/// FUN_00227728(rec, id, k): FUN_00227788(rec), rec[0] = id, rec[1] = key_colour(k)
pub fn ee_sub_227728<M: QMem + ?Sized>(m: &mut M, r: u32, id: u32, k: u32) {
    ee_sub_227788(m, r);
    m.w32(r, id);
    m.w32(r + 4, key_colour(k));
}

/// FUN_00227788(rec): {-1, key_colour(-1), 0}
pub fn ee_sub_227788<M: QMem + ?Sized>(m: &mut M, r: u32) {
    m.w32(r, u32::MAX);
    m.w32(r + 4, key_colour(u32::MAX));
    m.w32(r + 8, 0);
}

/// FUN_002277d0(rec, buf): id -1 -> empty string, else FUN_0021ed58(buf, FUN_0013e368(0x3dd490, id))
pub fn ee_sub_2277d0<M: QMem + ?Sized>(m: &mut M, r: u32, buf: u32) {
    let id = m.r32(r);
    if id == u32::MAX {
        m.w8(buf, 0);
    } else {
        let s = m.call(0x0013_e368, &[0x003d_d490, id], &[]).0;
        m.call(0x0021_ed58, &[buf, s], &[]);
    }
}

/// FUN_00227820(rec, text): reveal the key text: f = FUN_001fcfd0(0x3dd040) - 0.5, n = FUN_00215e30(text);
/// k = n, or (int)(FUN_001fe888 / f * n) while f > 0 and FUN_001fe848(0x3dd040, id); k < n ->
/// FUN_00215e80(text, k) and highlight, else blink ((int)(now * 4) & 1); highlight ->
/// FUN_0021eda0(*text, 0x2a97f0)
pub fn ee_sub_227820<M: QMem + ?Sized>(m: &mut M, r: u32, text: u32) {
    let id = m.r32(r);
    if id == u32::MAX {
        return;
    }
    let f = m.call(0x001f_cfd0, &[0x003d_d040], &[]).1[0] - 0.5;
    let n = m.call(0x0021_5e30, &[text], &[]).0 as i32;
    let mut k = n;
    if !(f <= 0.0) && m.call(0x001f_e848, &[0x003d_d040, id], &[]).0 != 0 {
        let g = m.call(0x001f_e888, &[0x003d_d040], &[]).1[0];
        k = (g / f * n as f32) as i32;
    }
    let hl = if k < n {
        m.call(0x0021_5e80, &[text, k as u32], &[]);
        true
    } else {
        (m.rf(NOW) * 4.0) as i32 & 1 != 0
    };
    if hl {
        let s = m.r32(text);
        m.call(0x0021_eda0, &[s, 0x002a_97f0], &[]);
    }
}

/// FUN_00227940(rec, buf): buf = "", then for the 16 {mask, char} pairs at DAT_002a97f8 matching rec +8:
/// FUN_0021ecd8(buf, "%c" 0x2a9878, char)
pub fn ee_sub_227940<M: QMem + ?Sized>(m: &mut M, r: u32, buf: u32) {
    m.w8(buf, 0);
    for i in 0..0x10u32 {
        let e = 0x002a_97f8 + 8 * i;
        if m.r32(r + 8) & m.r32(e) != 0 {
            let c = m.r8(e + 4) as u32;
            m.call(0x0021_ecd8, &[buf, 0x002a_9878, c], &[]);
        }
    }
}

/// FUN_00227d20(vol, pos, radius): radius <= 1 -> FUN_00227c90(vol, pos); else in a frame
/// (FUN_001ae478 / 1ae4a0(.., 2)) a buffer of min(+0xe4, 0x1555) * 12 bytes (FUN_001ae318) and
/// PruneBsp(radius, vol +0xec, 0x7fffffff, local pos, &count, &buf) in {0, 2}
pub fn ee_sub_227d20<M: QMem + ?Sized>(m: &mut M, o: u32, pos: V4, rad: f32) -> bool {
    if rad <= 1.0 {
        return m.call(0x0022_7c90, &[o], &[pos]).0 != 0;
    }
    let sp = stack_push(m, 0x30);
    m.call(0x001a_e478, &[sp], &[]);
    let n = (m.r32(o + 0xe4) as i32).min(0x1555);
    m.w32(sp + 0x20, n as u32);
    m.w32(sp + 0x10, (n * 0xc) as u32);
    m.w32(sp + 0x14, 0x10);
    let b = m.call(0x001a_e318, &[sp + 0x10], &[]).0;
    m.w32(sp + 0x24, b);
    let l = m.call(0x001a_6f00, &[0, o], &[pos]).1;
    let bsp = m.r32(o + 0xec);
    let r = m.call(0x0011_8a88, &[bsp, 0x7fff_ffff, sp + 0x20, sp + 0x24], &[[rad, 0.0, 0.0, 0.0], l]).0;
    m.call(0x001a_e4a0, &[sp, 2], &[]);
    stack_pop(m, 0x30);
    r == 0 || r == 2
}

/// FUN_00228170(o): the world's current object (FUN_001c4840(0x3dd490)) slot 37(o), then o slot 2(5, 0, 0,
/// 0, 0)
pub fn ee_sub_228170<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let w = m.call(0x001c_4840, &[0x003d_d490], &[]).0;
    if w != 0 {
        vslot(m, w, 37, &[o], &[]);
        vslot(m, o, 2, &[5, 0, 0, 0, 0], &[]);
    }
}

/// FUN_002284e0(o, a, b): FUN_00161f10(0x2deb30, o +0x168, 4, o +0x164, a, b)
pub fn ee_sub_2284e0<M: QMem + ?Sized>(m: &mut M, o: u32, a: u32, b: u32) {
    m.call(0x0016_1f10, &[0x002d_eb30, o + 0x168, 4, o + 0x164, a, b], &[]);
}

/// FUN_00228518(o): FUN_002109d0(o +0x144)
pub fn ee_sub_228518<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let v = m.r32(o + 0x144);
    m.call(0x0021_09d0, &[v], &[]);
}
