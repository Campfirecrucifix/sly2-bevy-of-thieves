//! WP-Q4: run 0x00156390..0x00157cf0 - the stimulus records (class C2ce2b8, 0x60 bytes from pool
//! 0x3e1498, list 0x3e148c: +0x14 source, +0x20 / +0x30 position, +0x40 arg, +0x48 serial, +0x4c start
//! time, +0x54 source +0x64; source kinds +0x50 flag, +0x54 radius, +0x58 z range, +0x60 duration), the
//! world's seven stimulus kinds (world +0x4020..+0x4038), the DMA chain builder (packet {+0 limit, +4 base,
//! +8 open tag, +0xc cursor, +0x10 align flag, +0x14 / +0x18 flush hook, +0x20}) and the world-to-VU1
//! upload of the camera (FUN_00157cf0).
//! The DMA builder only writes DMAtag qwords into EE memory (cnt 0x1, ref 0x4, call 0x5, ret 0x6, end 0x7):
//! game logic that is ported 1:1 here (the Bevy renderer reads the scene directly; these keep the
//! callers' packet bookkeeping identical).
//! 1:1 against `QMem` (classes::q::q2mem); callees outside the run go through `QMem::call` by address.
//! Quadword (u128 GPR) vector arguments travel in `v`.
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};
use crate::engine::game::q_001e9090::ee_sub_1ea410;

/// DAT_002dded0: game clock
pub const NOW: u32 = 0x002d_ded0;
/// stimulus record list (FUN_0011a220 / 11a2e8 / 11a380)
pub const STIM_LIST: u32 = 0x003e_148c;
/// stimulus record pool (ActPool FUN_0011ccc0)
pub const STIM_POOL: u32 = 0x003e_1498;
/// DAT_003e11f8: stimulus serial
pub const STIM_SERIAL: u32 = 0x003e_11f8;
/// DAT_002bdce0: duration cap of the fade curve (FUN_00156898)
pub const FADE_CAP: u32 = 0x002b_dce0;
/// DAT_002bdcf0 / f4 / f8: fade polynomial c0 + t (c1 + t c2)
pub const FADE_C: u32 = 0x002b_dcf0;

/// FUN_00156390 / 1570a8 / 1577a0 / 1578f8 (identical static pool inits) = FUN_001ea410
pub fn ee_sub_156390<M: QMem + ?Sized>(m: &mut M, init: u32, prio: u32) {
    ee_sub_1ea410(m, init, prio);
}
/// FUN_001564c8 / 1571e0 / 1578d8 / 157a30: (1, 0xffff)
pub fn ee_sub_1564c8<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_1ea410(m, 1, 0xffff);
}

/// FUN_00156548(src, pos, arg, out): new stimulus record (pool 0x3e1498, zeroed 0x60): vtable 0x2ce2b8,
/// +0x14 src, +0x20 = +0x30 = pos, +0x40 arg, +0x48 = ++serial, +0x4c now, +0x54 src +0x64; appended to
/// 0x3e148c, slot 0 twice, FUN_00256ee0(rec, *src), *out = rec, slot 2(2, 0, 0, 0, 0), slot 1
pub fn ee_sub_156548<M: QMem + ?Sized>(m: &mut M, src: u32, pos: V4, arg: u32, out: u32) -> u32 {
    let o = m.call(0x0011_ccc0, &[STIM_POOL], &[]).0;
    m.call(0x0023_33b4, &[o, 0, 0x60], &[]);
    m.wv(o + 0x30, pos);
    m.w32(o + 0x40, arg);
    m.w32(o + 0xc, 0x002c_e2b8);
    m.w32(o, 0);
    m.w32(o + 4, 0);
    m.w32(o + 0x14, src);
    m.wv(o + 0x20, pos);
    let s = m.r32(STIM_SERIAL).wrapping_add(1);
    m.w32(STIM_SERIAL, s);
    m.w32(o + 0x48, s);
    let t = m.r32(NOW);
    m.w32(o + 0x4c, t);
    let k = m.r32(src + 0x64);
    m.w32(o + 0x54, k);
    m.call(0x0011_a220, &[STIM_LIST, o], &[]);
    vslot(m, o, 0, &[], &[]);
    vslot(m, o, 0, &[], &[]);
    let k = m.r32(src);
    m.call(0x0025_6ee0, &[o, k], &[]);
    if out != 0 {
        m.w32(out, o);
    }
    vslot(m, o, 2, &[2, 0, 0, 0, 0], &[]);
    vslot(m, o, 1, &[], &[]);
    o
}

/// FUN_00156780(rec): still listed -> slot 0, slot 3(0xf, rec); still listed -> unlink (FUN_0011a2e8),
/// slot 2(3, 0, 0, 0, 0), slot 1 twice; else slot 1 once
pub fn ee_sub_156780<M: QMem + ?Sized>(m: &mut M, r: u32) {
    if ee_sub_156870(m, r) {
        return;
    }
    vslot(m, r, 0, &[], &[]);
    vslot(m, r, 3, &[0xf, r], &[]);
    if !ee_sub_156870(m, r) {
        m.call(0x0011_a2e8, &[STIM_LIST, r], &[]);
        vslot(m, r, 2, &[3, 0, 0, 0, 0], &[]);
        vslot(m, r, 1, &[], &[]);
    }
    vslot(m, r, 1, &[], &[]);
}

/// FUN_00156870(rec): FUN_0011a380(list, rec) == 0 (not listed)
pub fn ee_sub_156870<M: QMem + ?Sized>(m: &mut M, r: u32) -> bool {
    m.call(0x0011_a380, &[STIM_LIST, r], &[]).0 == 0
}

/// FUN_00156898(rec): radius of the record: kind +0x54, faded over duration +0x60 (capped DAT_002bdce0)
/// by c0 + t (c1 + t c2), t = min((now - +0x4c) / duration, 1)
pub fn ee_sub_156898<M: QMem + ?Sized>(m: &mut M, r: u32) -> f32 {
    let k = m.r32(r + 0x14);
    let d = m.rf(k + 0x60);
    if 0.0 < d {
        let cap = m.rf(FADE_CAP);
        let t = (m.rf(NOW) - m.rf(r + 0x4c)) / if d < cap { d } else { cap };
        let t = if 1.0 < t { 1.0 } else { t };
        let (c0, c1, c2) = (m.rf(FADE_C), m.rf(FADE_C + 4), m.rf(FADE_C + 8));
        return m.rf(k + 0x54) * (c0 + t * (c1 + t * c2));
    }
    m.rf(k + 0x54)
}

/// FUN_00156918(rec, pos): d = pos - rec +0x20; z in the kind's range (FUN_0011d910(d.z, kind +0x58)) and
/// hypot(d.x, d.y) <= FUN_00156898(rec)
pub fn ee_sub_156918<M: QMem + ?Sized>(m: &mut M, r: u32, pos: V4) -> bool {
    let c = m.rv(r + 0x20);
    let d = [pos[0] - c[0], pos[1] - c[1], pos[2] - c[2], pos[3] - c[3]];
    let k = m.r32(r + 0x14);
    if m.call(0x0011_d910, &[k + 0x58], &[[d[2], 0.0, 0.0, 0.0]]).0 == 0 {
        return false;
    }
    let l = (d[0] * d[0] + d[1] * d[1]).sqrt();
    let rad = ee_sub_156898(m, r);
    !(rad < l)
}

/// FUN_001569b8(world): the seven stimulus kinds (class 0x75, ids 0x2c2..0x2c8) at world +0x4020..+0x4038
/// with their flag +0x50 / radius +0x54 / duration +0x60 / +0x64 presets, slot 8 each
pub fn ee_sub_1569b8<M: QMem + ?Sized>(m: &mut M, w: u32) {
    // (id, slot offset, writes): DAT_002bdd00.. hold the preset floats
    let mk = |m: &mut M, id: u32| m.call(0x0013_e3c8, &[0x75, 0, id, u32::MAX], &[]).0;
    let o = mk(m, 0x2c2);
    m.w32(o + 0x50, 0);
    m.w32(w + 0x4020, o);
    vslot(m, o, 8, &[], &[]);
    let o = mk(m, 0x2c3);
    let v = m.r32(0x002b_dd00);
    m.w32(o + 0x50, 1);
    m.w32(o + 0x60, v);
    m.w32(w + 0x4024, o);
    vslot(m, o, 8, &[], &[]);
    let o = mk(m, 0x2c4);
    let v = m.r32(0x002b_dd04);
    m.w32(o + 0x50, 0);
    m.w32(o + 0x54, v);
    m.w32(w + 0x4028, o);
    vslot(m, o, 8, &[], &[]);
    let o = mk(m, 0x2c5);
    let (a, b) = (m.r32(0x002b_dd08), m.r32(0x002b_dd0c));
    m.w32(o + 0x54, a);
    m.w32(o + 0x60, b);
    m.w32(o + 0x50, 0);
    m.w32(w + 0x402c, o);
    vslot(m, o, 8, &[], &[]);
    let o = mk(m, 0x2c7);
    m.w32(o + 0x64, 3);
    m.w32(o + 0x60, 0xbf80_0000);
    m.w32(o + 0x50, 0);
    m.w32(w + 0x4034, o);
    vslot(m, o, 8, &[], &[]);
    let o = mk(m, 0x2c6);
    let (a, b) = (m.r32(0x002b_dd10), m.r32(0x002b_dd14));
    m.w32(o + 0x54, a);
    m.w32(o + 0x60, b);
    m.w32(o + 0x50, 0);
    m.w32(w + 0x4030, o);
    vslot(m, o, 8, &[], &[]);
    let o = mk(m, 0x2c8);
    let v = m.r32(0x002b_dd18);
    m.w32(o + 0x50, 0);
    m.w32(o + 0x54, v);
    m.w32(w + 0x4038, o);
    vslot(m, o, 8, &[], &[]);
}

fn kind_of<M: QMem + ?Sized>(m: &mut M, w: u32, k: u32, other: u32) -> u32 {
    match k {
        0 => m.r32(w + 0x4020),
        1 => m.r32(w + 0x4024),
        _ => other,
    }
}

/// FUN_00156b98(world, k, pos, arg, kind): FUN_00156698(kind 0 / 1 = world +0x4020 / +0x4024 else the
/// passed kind, pos, arg)
pub fn ee_sub_156b98<M: QMem + ?Sized>(m: &mut M, w: u32, k: u32, pos: V4, arg: u32, kind: u32) -> u32 {
    let s = kind_of(m, w, k, kind);
    m.call(0x0015_6698, &[s, arg], &[pos]).0
}

/// FUN_00156be0(src, pos, arg, slot): replace the record in *slot: the old one is ended (FUN_00156780) and
/// released (slot 1), then with a source a new record (FUN_00156548) is retained (slot 0)
pub fn ee_sub_156be0<M: QMem + ?Sized>(m: &mut M, src: u32, pos: V4, arg: u32, slot: u32) {
    let r = m.r32(slot);
    if r != 0 {
        ee_sub_156780(m, r);
        let r = m.r32(slot);
        vslot(m, r, 1, &[], &[]);
        m.w32(slot, 0);
    }
    if src != 0 {
        ee_sub_156548(m, src, pos, arg, slot);
        let r = m.r32(slot);
        vslot(m, r, 0, &[], &[]);
    }
}

/// FUN_00156c80(world, k, pos, arg, slot): FUN_00156be0 with kind 0 / 1 (else no source)
pub fn ee_sub_156c80<M: QMem + ?Sized>(m: &mut M, w: u32, k: u32, pos: V4, arg: u32, slot: u32) {
    let s = kind_of(m, w, k, 0);
    ee_sub_156be0(m, s, pos, arg, slot);
}

/// FUN_00156fe8(obj, rec): obj +0x28 bit 60 set, kind flag +0x50 clear, z in range and hypot(xy) < radius
pub fn ee_sub_156fe8<M: QMem + ?Sized>(m: &mut M, o: u32, r: u32) -> bool {
    if (m.r64(o + 0x28) >> 0x3c) & 1 == 0 {
        return false;
    }
    let k = m.r32(r + 0x14);
    if m.r32(k + 0x50) != 0 {
        return false;
    }
    let p = m.call(0x001a_62e8, &[o], &[]).1;
    let c = m.rv(r + 0x20);
    let d = [p[0] - c[0], p[1] - c[1], p[2] - c[2], p[3] - c[3]];
    if m.call(0x0011_d910, &[k + 0x58], &[[d[2], 0.0, 0.0, 0.0]]).0 == 0 {
        return false;
    }
    (d[0] * d[0] + d[1] * d[1]).sqrt() < m.rf(k + 0x54)
}

/// FUN_00157200: DMA channel handles (sceDmaReset(1) FUN_00241468, FUN_00240338, sceDmaGetChan
/// FUN_00241440 0 / 1 / 2 / 8 / 9 -> DAT_002de590..5a0)
pub fn ee_sub_157200<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x0024_1468, &[1], &[]);
    m.call(0x0024_0338, &[], &[]);
    for (i, ch) in [0u32, 1, 2, 8, 9].into_iter().enumerate() {
        let h = m.call(0x0024_1440, &[ch], &[]).0;
        m.w32(0x002d_e590 + 4 * i as u32, h);
    }
}

/// FUN_00157278(pkt): +0x10 = +0x14 = 0, +0x18 = &DAT_002ce488, FUN_001572b8
pub fn ee_sub_157278<M: QMem + ?Sized>(m: &mut M, p: u32) -> u32 {
    m.w32(p + 0x10, 0);
    m.w32(p + 0x14, 0);
    m.w32(p + 0x18, 0x002c_e488);
    ee_sub_1572b8(m, p);
    p
}

/// FUN_001572b8(pkt): +0..+0xc = 0
pub fn ee_sub_1572b8<M: QMem + ?Sized>(m: &mut M, p: u32) {
    m.w32(p + 0xc, 0);
    m.w32(p, 0);
    m.w32(p + 4, 0);
    m.w32(p + 8, 0);
}

/// FUN_001572d0(pkt): rewind: +8 = 0, cursor = base
pub fn ee_sub_1572d0<M: QMem + ?Sized>(m: &mut M, p: u32) {
    m.w32(p + 8, 0);
    let b = m.r32(p + 4);
    m.w32(p + 0xc, b);
}

/// FUN_001572e0(pkt, n): n qwords from FUN_001ae318({16 n, 16}): base = cursor, limit = base + 16 n
pub fn ee_sub_1572e0<M: QMem + ?Sized>(m: &mut M, p: u32, n: u32) {
    let s = stack_push(m, 0x10);
    m.w32(s, n.wrapping_mul(0x10));
    m.w32(s + 4, 0x10);
    let a = m.call(0x001a_e318, &[s], &[]).0;
    stack_pop(m, 0x10);
    m.w32(p + 4, a);
    m.w32(p, a.wrapping_add(n.wrapping_mul(0x10)));
    m.w32(p + 0xc, a);
}

/// FUN_00157330(pkt, n, buf): base = cursor = buf, limit = buf + 16 n
pub fn ee_sub_157330<M: QMem + ?Sized>(m: &mut M, p: u32, n: u32, buf: u32) {
    m.w32(p + 4, buf);
    m.w32(p + 0xc, buf);
    m.w32(p, buf.wrapping_add(n.wrapping_mul(0x10)));
}

/// FUN_00157348(pkt, out_n, out_base): close (FUN_001576f0), *out_n = qwords, *out_base = base, reset
pub fn ee_sub_157348<M: QMem + ?Sized>(m: &mut M, p: u32, out_n: u32, out_base: u32) {
    ee_sub_1576f0(m, p);
    ee_sub_1574a0(m, p);
    if out_n != 0 {
        let n = m.r32(p + 0xc).wrapping_sub(m.r32(p + 4)) >> 4;
        m.w32(out_n, n);
    }
    if out_base != 0 {
        let b = m.r32(p + 4);
        m.w32(out_base, b);
    }
    ee_sub_1572b8(m, p);
}

/// FUN_001573c0(pkt, out_n, out_base, out_h): FUN_001573f8 with the default cache 0x2dd590
pub fn ee_sub_1573c0<M: QMem + ?Sized>(m: &mut M, p: u32, out_n: u32, out_base: u32, out_h: u32) {
    ee_sub_1573f8(m, p, 0x002d_d590, out_n, out_base, out_h);
}

/// FUN_001573f8(pkt, cache, out_n, out_base, out_h): FUN_00157348 plus *out_h = FUN_00116970(cache,
/// 16 n, base, 4)
pub fn ee_sub_1573f8<M: QMem + ?Sized>(m: &mut M, p: u32, cache: u32, out_n: u32, out_base: u32, out_h: u32) {
    ee_sub_1576f0(m, p);
    ee_sub_1574a0(m, p);
    let n = m.r32(p + 0xc).wrapping_sub(m.r32(p + 4)) >> 4;
    if out_n != 0 {
        m.w32(out_n, n);
    }
    if out_base != 0 {
        let b = m.r32(p + 4);
        m.w32(out_base, b);
    }
    if out_h != 0 {
        let b = m.r32(p + 4);
        let h = m.call(0x0011_6970, &[cache, n << 4, b, 4], &[]).0;
        m.w32(out_h, h);
    }
    ee_sub_1572b8(m, p);
}

/// FUN_001574a0: empty
pub fn ee_sub_1574a0<M: QMem + ?Sized>(_m: &mut M, _p: u32) {}

fn put_tag<M: QMem + ?Sized>(m: &mut M, p: u32, lo: u32, hi: u32) -> u32 {
    let c = m.r32(p + 0xc);
    m.w32(p + 0xc, c + 0x10);
    m.w32(c, lo);
    m.w32(c + 4, hi);
    m.w32(c + 8, 0);
    m.w32(c + 0xc, 0);
    c
}

/// FUN_001574b8(pkt): close, open a cnt tag (0x10000000; qwc patched by the next close) at +8
pub fn ee_sub_1574b8<M: QMem + ?Sized>(m: &mut M, p: u32) {
    ee_sub_1576f0(m, p);
    let c = put_tag(m, p, 0x1000_0000, 0);
    m.w32(p + 8, c);
}

fn ref_like<M: QMem + ?Sized>(m: &mut M, p: u32, lo: u32, hi: u32) {
    let open = m.r32(p + 8) != 0;
    if open {
        ee_sub_1576f0(m, p);
    }
    put_tag(m, p, lo, hi);
    if open {
        let c = put_tag(m, p, 0x1000_0000, 0);
        m.w32(p + 8, c);
    }
}

/// FUN_00157500(pkt, qwc, addr): ref tag (qwc | 0x40000000, addr); an open cnt is closed and reopened after
pub fn ee_sub_157500<M: QMem + ?Sized>(m: &mut M, p: u32, qwc: u32, addr: u32) {
    ref_like(m, p, qwc | 0x4000_0000, addr);
}

/// FUN_00157598(pkt, addr): call tag (0x50000000, addr), cnt reopened like FUN_00157500
pub fn ee_sub_157598<M: QMem + ?Sized>(m: &mut M, p: u32, addr: u32) {
    ref_like(m, p, 0x5000_0000, addr);
}

/// FUN_00157620(pkt): close, ret tag 0x60000000
pub fn ee_sub_157620<M: QMem + ?Sized>(m: &mut M, p: u32) {
    ee_sub_1576f0(m, p);
    put_tag(m, p, 0x6000_0000, 0);
}

/// FUN_00157660(pkt, n, src): FUN_001ae528(cursor, src, n) (qword copy), cursor += 16 n
pub fn ee_sub_157660<M: QMem + ?Sized>(m: &mut M, p: u32, n: u32, src: u32) {
    let c = m.r32(p + 0xc);
    m.call(0x001a_e528, &[c, src, n], &[]);
    let c = m.r32(p + 0xc);
    m.w32(p + 0xc, c.wrapping_add(n.wrapping_mul(0x10)));
}

/// FUN_001576b0(pkt): close, end tag 0x70000000
pub fn ee_sub_1576b0<M: QMem + ?Sized>(m: &mut M, p: u32) {
    ee_sub_1576f0(m, p);
    put_tag(m, p, 0x7000_0000, 0);
}

/// FUN_001576f0(pkt): close the open cnt: align flag +0x10 pads the cursor to 16 with zero words, else a
/// flush hook (+0x14) calls (*(+0x18) +0xc)(pkt + adj, 0); then the open tag's qwc |= qwords since it - 1
pub fn ee_sub_1576f0<M: QMem + ?Sized>(m: &mut M, p: u32) {
    if m.r32(p + 0x10) == 0 {
        if m.r32(p + 0x14) != 0 {
            let vt = m.r32(p + 0x18);
            let f = m.r32(vt + 0xc);
            let adj = m.r16(vt + 8) as i16 as i32;
            m.call(f, &[p.wrapping_add(adj as u32), 0], &[]);
        }
    } else {
        let mut c = m.r32(p + 0xc);
        while c & 0xf != 0 {
            m.w32(c, 0);
            c += 4;
        }
        m.w32(p + 0xc, c);
    }
    let t = m.r32(p + 8);
    if t != 0 {
        let q = ((m.r32(p + 0xc).wrapping_sub(t) as i32) >> 4) as u32;
        let v = m.r32(t) | q.wrapping_sub(1);
        m.w32(t, v);
        m.w32(p + 8, 0);
    }
}

/// FUN_00157cf0(pkt, a, b): world pass header: call the VU1 microprogram chain (0x2df800 without
/// DAT_003e12c8, else 0x2df940 / 0x2dfa80 by DAT_002dea4c == &DAT_002dea50), +0x20 = 1, open cnt,
/// FUN_00226d10, FUN_002270c8(pkt, 0x25, 0x2df010, 0), FUN_00226ca8(pkt, 0, 0); unpack (FUN_00226f28(pkt, 0xc,
/// n, src, addr)) the projection {DAT_002c1f00 (12 B), 1 / (near (1 - f)), DAT_002c1f10 (12 B), -f / (near
/// (1 - f))}, f = (near + far) / (near - far) (camera 0x2ddf40 +0x4c / +0x50) to 0, the two camera matrices
/// (0x2de080 / 0x2ddfa0, or FUN_0014f768(cam, 1 / 2, 0, m0, m1)) to 2 / 6 and 0x3e12b8 to 10;
/// FUN_00226ca8(pkt, 0x32, 0x1e7), close, FUN_0016e1a8(pkt, a, b)
pub fn ee_sub_157cf0<M: QMem + ?Sized>(m: &mut M, p: u32, a: u32, b: u32) {
    let split = m.r32(0x003e_12c8) != 0;
    let alt = m.r32(0x002d_ea4c) == 0x002d_ea50;
    let chain = if !split {
        0x002d_f800
    } else if alt {
        0x002d_f940
    } else {
        0x002d_fa80
    };
    ee_sub_157598(m, p, chain);
    m.w32(p + 0x20, 1);
    ee_sub_1574b8(m, p);
    m.call(0x0022_6d10, &[p], &[]);
    m.call(0x0022_70c8, &[p, 0x25, 0x002d_f010, 0], &[]);
    m.call(0x0022_6ca8, &[p, 0, 0], &[]);
    let cam = 0x002d_df40;
    let sp = stack_push(m, 0xa0);
    let (near, far) = (m.rf(cam + 0x4c), m.rf(cam + 0x50));
    let f = (near + far) / (near - far);
    let d = near * (1.0 - f);
    for k in 0..3 {
        let v = m.r32(0x002c_1f00 + 4 * k);
        m.w32(sp + 4 * k, v);
        let v = m.r32(0x002c_1f10 + 4 * k);
        m.w32(sp + 0x10 + 4 * k, v);
    }
    m.wf(sp + 0xc, 1.0 / d);
    m.wf(sp + 0x1c, -f / d);
    m.call(0x0022_6f28, &[p, 0xc, 2, sp, 0], &[]);
    let (m0, m1) = (sp + 0x20, sp + 0x60);
    if !split {
        let x = m.rm(0x002d_e080);
        m.wm(m0, x);
        let x = m.rm(0x002d_dfa0);
        m.wm(m1, x);
    } else {
        m.call(0x0014_f768, &[cam, if alt { 1 } else { 2 }, 0, m0, m1], &[]);
    }
    m.call(0x0022_6f28, &[p, 0xc, 4, m0, 2], &[]);
    m.call(0x0022_6f28, &[p, 0xc, 4, m1, 6], &[]);
    m.call(0x0022_6f28, &[p, 0xc, 1, 0x003e_12b8, 0xa], &[]);
    stack_pop(m, 0xa0);
    m.call(0x0022_6ca8, &[p, 0x32, 0x1e7], &[]);
    ee_sub_1576f0(m, p);
    m.call(0x0016_e1a8, &[p, a, b], &[]);
}
