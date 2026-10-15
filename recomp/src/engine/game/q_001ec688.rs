//! WP-Q5: run_001ec688 queue package: the texture / CLUT VRAM residency tables (DAT_003dcff8 CLUTs
//! of 0x20 bytes at DAT_003dcffc, DAT_003dcff0 textures of 0x28 bytes at DAT_003dcff4; open-addressed
//! 0x3ffd-slot hashes of the resident ones at DAT_003dd018 / DAT_003dd010), their GS register
//! builders (TEX0 / CLUT fields), the per-bank upload (FUN_001ecc28 into a caller packet,
//! FUN_001ece58 as a standalone image-transfer chain), the texture-animation record table at
//! 0x3da798 (stride 0x14) and small lookups. Game code that formats GS registers / GIF packets is
//! ported 1:1 against `QMem` (the packet / DMA primitives FUN_00157xxx / FUN_00166xxx / FUN_00226xxx
//! / FUN_00227xxx are routed through `QMem::call`).
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem};

/// CLUT table count / base, texture table count / base
const NCLUT: u32 = 0x003d_cff8;
const CLUTS: u32 = 0x003d_cffc;
const NTEX: u32 = 0x003d_cff0;
const TEXS: u32 = 0x003d_cff4;
/// resident hashes: textures / CLUTs
const HTEX: u32 = 0x003d_d010;
const HCLUT: u32 = 0x003d_d018;
const HSLOTS: i32 = 0x3ffd;

fn w64<M: QMem + ?Sized>(m: &mut M, a: u32, v: u64) {
    m.w32(a, v as u32);
    m.w32(a + 4, (v >> 32) as u32);
}

/// FUN_001ec688: FUN_00226f28(d, 0xc, e[0], e[1], k) for entry `i` of +0x20; returns e[0]
pub fn ee_sub_1ec688<M: QMem + ?Sized>(m: &mut M, o: u32, i: u32, k: u32, d: u32) -> u32 {
    let e = m.r32(o + 0x20).wrapping_add(i << 3);
    let (a, b) = (m.r32(e), m.r32(e + 4));
    m.call(0x0022_6f28, &[d, 0xc, a, b, k], &[]);
    m.r32(e)
}

/// FUN_001ec6d0: FUN_00226f28(d, 0xc, 1, &copy of e[1]+0x30 quad, k); returns 1
pub fn ee_sub_1ec6d0<M: QMem + ?Sized>(m: &mut M, o: u32, i: u32, k: u32, d: u32) -> u32 {
    let e = m.r32(o + 0x20).wrapping_add(i << 3);
    let p = m.r32(e + 4);
    let q = m.rv(p + 0x30);
    let sp = stack_push(m, 0x10);
    m.wv(sp, q);
    m.call(0x0022_6f28, &[d, 0xc, 1, sp, k], &[]);
    stack_pop(m, 0x10);
    1
}

/// FUN_001ec718: FUN_001ae5a0(p[2], p[1], p[0])
pub fn ee_sub_1ec718<M: QMem + ?Sized>(m: &mut M, p: u32) {
    let (a, b, c) = (m.r32(p + 8), m.r32(p + 4), m.r32(p));
    m.call(0x001a_e5a0, &[a, b, c], &[]);
}

/// FUN_001ec740: records [lo, hi) of 0x3da798 whose +8 lies in [base, base + n) are rebased to `to`
pub fn ee_sub_1ec740<M: QMem + ?Sized>(m: &mut M, lo: i32, hi: i32, n: u32, base: u32, to: u32) {
    for i in lo..hi.max(lo) {
        let e = (i as u32).wrapping_mul(0x14).wrapping_add(0x003d_a798);
        let v = m.r32(e + 8);
        if base <= v && v < base.wrapping_add(n) {
            m.w32(e + 8, to.wrapping_add(v.wrapping_sub(base)));
        }
    }
}

/// FUN_001ec7a0: clear the 8 words at 0x3dcfe8 (the bank tables)
pub fn ee_sub_1ec7a0<M: QMem + ?Sized>(m: &mut M) {
    for k in 0..8 {
        m.w32(0x003d_cfe8 + 4 * k, 0);
    }
}

/// FUN_001ec7e8: DAT_003dcfa0 = 0, record count DAT_003da790 = 0, zero 0x40 bytes at 0x3dcfa8
pub fn ee_sub_1ec7e8<M: QMem + ?Sized>(m: &mut M) {
    m.w32(0x003d_cfa0, 0);
    m.w32(0x003d_a790, 0);
    m.call(0x0023_33b4, &[0x003d_cfa8, 0, 0x40], &[]);
}

/// FUN_001ec820: entry (0x2c bytes, DAT_003dcfec / count DAT_003dcfe8) with id `k` (-1 -> 0)
pub fn ee_sub_1ec820<M: QMem + ?Sized>(m: &mut M, k: u32) -> u32 {
    if k == 0xffff_ffff {
        return 0;
    }
    let n = m.r32(0x003d_cfe8) as i32;
    let b = m.r32(0x003d_cfec);
    for i in 0..n.max(0) as u32 {
        let e = b + i * 0x2c;
        if m.r32(e) == k {
            return e;
        }
    }
    0
}

/// FUN_001ec880: node with id `k` in the list DAT_003e1590 (next +0x18)
pub fn ee_sub_1ec880<M: QMem + ?Sized>(m: &mut M, k: u32) -> u32 {
    if k == 0xffff_ffff {
        return 0;
    }
    let mut p = m.r32(0x003e_1590);
    while p != 0 {
        if m.r32(p) == k {
            return p;
        }
        p = m.r32(p + 0x18);
    }
    0
}

/// FUN_001ec8d0: ee_sub_1ec880(k); if 0, return list head DAT_003e1590
pub fn ee_sub_1ec8d0<M: QMem + ?Sized>(m: &mut M, k: u32) -> u32 {
    let r = ee_sub_1ec880(m, k);
    if r == 0 {
        m.r32(0x003e_1590)
    } else {
        r
    }
}

/// FUN_001ec8f8: DAT_003dcfec + i * 0x2c
pub fn ee_sub_1ec8f8<M: QMem + ?Sized>(m: &mut M, i: u32) -> u32 {
    m.r32(0x003d_cfec).wrapping_add(i.wrapping_mul(0x2c))
}

/// FUN_001ec918: node with +0x14 == k in the list DAT_003e159c (next +0x40); a node with +0x30 set
/// stops the search (0)
pub fn ee_sub_1ec918<M: QMem + ?Sized>(m: &mut M, k: u32) -> u32 {
    if k == 0xffff_ffff {
        return 0;
    }
    let mut p = m.r32(0x003e_159c);
    while p != 0 {
        if m.r32(p + 0x30) != 0 {
            return 0;
        }
        if m.r32(p + 0x14) == k {
            return p;
        }
        p = m.r32(p + 0x40);
    }
    0
}

/// CLUT register (+0x18, TEX0 CLUT half): CBP (bits 37..50) = `cbp`, PSM (20..25) = 0x13 when the
/// CLUT has > 16 entries (+8) else 0x14, CPSM / CSM / CSA 0, CLD (61..63) = 1
pub fn clut_reg(old: u64, cbp: u64, n: u16) -> u64 {
    let psm: u64 = if 0x10 < n { 0x13 } else { 0x14 };
    (old & 0x0000_001f_fc0f_ffff) | (cbp & 0x3fff) << 37 | psm << 20 | 1 << 61
}

/// FUN_001ec9d0: CLUT register of `c` for VRAM block `cbp`
pub fn ee_sub_1ec9d0<M: QMem + ?Sized>(m: &mut M, c: u32, cbp: u32) {
    let old = m.r64(c + 0x18);
    let n = m.r16(c + 8);
    w64(m, c + 0x18, clut_reg(old, cbp as u64, n));
}

/// FUN_001ecab0: CLUT image upload FUN_00170650(cbp, 1, 0, w, rows, +0x10, pk): w = 16 when > 16
/// entries else 8, rows = round_up(n, w) / w
pub fn ee_sub_1ecab0<M: QMem + ?Sized>(m: &mut M, c: u32, cbp: u32, pk: u32) {
    let n = m.r16(c + 8) as i32;
    let w = if n < 0x11 { 8 } else { 0x10 };
    let rows = ((n - 1 + w) & -w) / w;
    let src = m.r32(c + 0x10);
    m.call(0x0017_0650, &[cbp, 1, 0, w as u32, rows as u32, src, pk], &[]);
}

/// texture register (+0x20, TEX0 low half): TBP0 = `tbp`, TBW = +0xd, PSM = +0xc, TW / TH =
/// log2 sizes (`tw`, `th` from FUN_001703c0 of +8 / +0xa), TCC (34) = 1
pub fn tex_reg(old: u64, tbp: u64, tbw: u8, psm: u8, tw: u64, th: u64) -> u64 {
    let v = (old & 0xffff_ffff_ffff_c000) | (tbp & 0x3fff);
    let v = (v & 0xffff_ffff_fff0_3fff) | ((tbw & 0x3f) as u64) << 14;
    let v = (v & 0xffff_ffff_fc0f_ffff) | ((psm & 0x3f) as u64) << 20;
    let v = (v & 0xffff_ffff_c3ff_ffff) | (tw & 0xf) << 26;
    (v & 0xffff_fffc_3fff_ffff) | (th & 0xf) << 30 | 1 << 34
}

/// FUN_001ecb18: texture register of `t` for VRAM block `tbp`
pub fn ee_sub_1ecb18<M: QMem + ?Sized>(m: &mut M, t: u32, tbp: u32) {
    let old = m.r64(t + 0x20);
    let (tbw, psm) = (m.r8(t + 0xd), m.r8(t + 0xc));
    let w = m.r16(t + 8) as u32;
    let tw = m.call(0x0017_03c0, &[w], &[]).0 as u64;
    let h = m.r16(t + 10) as u32;
    let th = m.call(0x0017_03c0, &[h], &[]).0 as u64;
    w64(m, t + 0x20, tex_reg(old, tbp as u64, tbw, psm, tw, th));
}

/// FUN_001ecbf0: texture image upload FUN_00170650(tbp, +0xd, +0xc, +8, +0xa, +0x18, pk)
pub fn ee_sub_1ecbf0<M: QMem + ?Sized>(m: &mut M, t: u32, tbp: u32, pk: u32) {
    let (a, b) = (m.r8(t + 0xd) as u32, m.r8(t + 0xc) as u32);
    let (w, h) = (m.r16(t + 8) as u32, m.r16(t + 10) as u32);
    let src = m.r32(t + 0x18);
    m.call(0x0017_0650, &[tbp, a, b, w, h, src, pk], &[]);
}

/// FUN_001ecc28: make bank `k`'s CLUTs then textures resident in VRAM allocator `va` {+4 cursor,
/// +8 limit}: an entry already resident (hash lookup by id) copies the resident register; else it is
/// allocated (FUN_00170548; textures first align the cursor to 32 blocks unless +0xc), its register
/// built and, with packet `pk` and the cursor within the limit, its image queued (FUN_001ecab0 /
/// FUN_001ecbf0 + FUN_00157500(pk, src, size)) and it is hashed. Returns whether `pk` grew (+0xc).
pub fn ee_sub_1ecc28<M: QMem + ?Sized>(m: &mut M, k: u32, va: u32, pk: u32) -> bool {
    let before = if pk != 0 { m.r32(pk + 0xc) } else { 0 };
    let mut i = 0i32;
    while i < m.r32(NCLUT) as i32 {
        let c = m.r32(CLUTS).wrapping_add((i as u32) << 5);
        if m.r8(c + 4) as u32 == k {
            let id = m.r32(c);
            let r = ee_sub_1ed5d0(m, id);
            if r != 0 {
                let v = m.r64(r + 0x18);
                w64(m, c + 0x18, v);
            } else {
                let n = m.r16(c + 10) as u32;
                let b = m.call(0x0017_0548, &[va, n], &[]).0;
                ee_sub_1ec9d0(m, c, b);
                if pk != 0 && m.r32(va + 4) as i32 <= m.r32(va + 8) as i32 {
                    ee_sub_1ecab0(m, c, b, pk);
                    let (s, z) = (m.r32(c + 0x10), m.r32(c + 0xc));
                    m.call(0x0015_7500, &[pk, s, z], &[]);
                }
                ee_sub_1ed560(m, c);
            }
        }
        i += 1;
    }
    let mut i = 0i32;
    while i < m.r32(NTEX) as i32 {
        let t = m.r32(TEXS).wrapping_add(i as u32 * 0x28);
        if m.r8(t + 4) as u32 == k {
            let id = m.r32(t);
            let r = ee_sub_1ed4e0(m, id);
            if r != 0 {
                let v = m.r64(r + 0x20);
                w64(m, t + 0x20, v);
            } else {
                if m.r8(t + 0xc) == 0 {
                    let a = m.r32(va + 4) & 0x1f;
                    if a != 0 {
                        m.call(0x0017_0548, &[va, 0x20 - a], &[]);
                    }
                }
                let n = m.r16(t + 0xe) as u32;
                let b = m.call(0x0017_0548, &[va, n], &[]).0;
                ee_sub_1ecb18(m, t, b);
                if pk != 0 && m.r32(va + 4) as i32 <= m.r32(va + 8) as i32 {
                    ee_sub_1ecbf0(m, t, b, pk);
                    let (s, z) = (m.r32(t + 0x18), m.r32(t + 0x14));
                    m.call(0x0015_7500, &[pk, s, z], &[]);
                }
                ee_sub_1ed470(m, t);
            }
        }
        i += 1;
    }
    pk != 0 && m.r32(pk + 0xc) != before
}

/// one image of FUN_001ece58: a 10-qword packet (frame +0x70) built by `build`, its GIF tag's EOP
/// (+0xc - 0x10 |= 0x8000), closed (FUN_00157348) and chained (FUN_002270c8 / FUN_00227148 src, size)
fn ece58_image<M: QMem + ?Sized>(m: &mut M, sp: u32, ch: u32, src: u32, size: u32, build: &dyn Fn(&mut M, u32)) {
    m.call(0x0015_72d0, &[sp], &[]);
    m.call(0x0015_7330, &[sp, 10, sp + 0x70], &[]);
    build(m, sp);
    let t = m.r32(sp + 0xc).wrapping_sub(0x10);
    let v = m.r64(t) | 0x8000;
    w64(m, t, v);
    m.call(0x0015_7348, &[sp, sp + 0x110, sp + 0x114], &[]);
    let (a, b) = (m.r32(sp + 0x110), m.r32(sp + 0x114));
    m.call(0x0022_70c8, &[ch, a, b, 0], &[]);
    m.call(0x0022_7148, &[ch, src, size, 0], &[]);
}

/// FUN_001ece58: upload bank `k` with allocator `va` as its own DMA chain (frame: packet +0, chain
/// +0x30 sized (DAT_003dcf9c * 3 + DAT_003dcf98) * 10 + 4, FUN_001ae478 frame +0x60): every bank
/// CLUT / texture is allocated, its register built and its image chained; when the allocator moved, a
/// TEXFLUSH (0x3f) packet ends the chain, which is closed and kicked (FUN_001573c0) -> returned handle
/// (+0x118, else 0)
pub fn ee_sub_1ece58<M: QMem + ?Sized>(m: &mut M, va: u32, k: u32) -> u32 {
    let sp = stack_push(m, 0x120);
    let ch = sp + 0x30;
    m.call(0x0016_6920, &[sp], &[]);
    m.w32(sp + 0x118, 0);
    m.call(0x0022_6c10, &[ch], &[]);
    let c0 = m.r32(va + 4);
    m.w32(sp + 0x11c, c0);
    m.call(0x001a_e478, &[sp + 0x60], &[]);
    let n = (m.r32(0x003d_cf9c).wrapping_mul(3).wrapping_add(m.r32(0x003d_cf98)))
        .wrapping_mul(10)
        .wrapping_add(4);
    m.call(0x0015_72e0, &[ch, n], &[]);
    m.call(0x0015_74b8, &[ch], &[]);
    m.call(0x0022_6d10, &[ch], &[]);
    let mut i = 0i32;
    while i < m.r32(NCLUT) as i32 {
        let c = m.r32(CLUTS).wrapping_add((i as u32) << 5);
        if m.r8(c + 4) as u32 == k {
            let (src, size) = (m.r32(c + 0x10), m.r32(c + 0xc));
            ece58_image(m, sp, ch, src, size, &|m: &mut M, pk| {
                let n = m.r16(c + 10) as u32;
                let b = m.call(0x0017_0548, &[va, n], &[]).0;
                ee_sub_1ec9d0(m, c, b);
                ee_sub_1ecab0(m, c, b, pk);
            });
        }
        i += 1;
    }
    let mut i = 0i32;
    while i < m.r32(NTEX) as i32 {
        let t = m.r32(TEXS).wrapping_add(i as u32 * 0x28);
        if m.r8(t + 4) as u32 == k {
            let (src, size) = (m.r32(t + 0x18), m.r32(t + 0x14));
            ece58_image(m, sp, ch, src, size, &|m: &mut M, pk| {
                let n = m.r16(t + 0xe) as u32;
                let b = m.call(0x0017_0548, &[va, n], &[]).0;
                ee_sub_1ecb18(m, t, b);
                ee_sub_1ecbf0(m, t, b, pk);
            });
        }
        i += 1;
    }
    if (m.r32(sp + 0x11c) as i32) < m.r32(va + 4) as i32 {
        m.call(0x0015_72d0, &[sp], &[]);
        m.call(0x0015_7330, &[sp, 10, sp + 0x70], &[]);
        m.call(0x0016_6bd8, &[sp, 0, 1, 0xe], &[]);
        m.call(0x0016_6e90, &[sp, 0x3f, 0], &[]);
        m.call(0x0016_6968, &[sp], &[]);
        m.call(0x0015_7348, &[sp, sp + 0x110, sp + 0x114], &[]);
        let (a, b) = (m.r32(sp + 0x110), m.r32(sp + 0x114));
        m.call(0x0022_70c8, &[ch, a, b, 0], &[]);
        m.call(0x0022_6d10, &[ch], &[]);
        m.call(0x0015_76f0, &[ch], &[]);
        m.call(0x0015_7620, &[ch], &[]);
        m.call(0x0015_73c0, &[ch, 0, 0, sp + 0x118], &[]);
    }
    let r = m.r32(sp + 0x118);
    m.call(0x001a_e4a0, &[sp + 0x60, 2], &[]);
    stack_pop(m, 0x120);
    r
}

/// FUN_001ed1a8: with DAT_003e12c8, run the 16-bit RGBA5551 colour at `p` through FUN_001ed278 as
/// RGBA32 {r << 3, g << 3, b << 3, a ? 0xff : 0} and pack it back
pub fn ee_sub_1ed1a8<M: QMem + ?Sized>(m: &mut M, p: u32) {
    if m.r32(0x003e_12c8) == 0 {
        return;
    }
    let v = m.r16(p) as u32;
    let sp = stack_push(m, 0x10);
    m.w8(sp, ((v & 0x1f) << 3) as u8);
    m.w8(sp + 1, ((v >> 2) & 0xf8) as u8);
    m.w8(sp + 2, ((v >> 7) & 0xf8) as u8);
    m.w8(sp + 3, if v & 0x8000 != 0 { 0xff } else { 0 });
    m.call(0x001e_d278, &[sp], &[]);
    let (r, g, b, a) = (m.r8(sp) as u32, m.r8(sp + 1) as u32, m.r8(sp + 2) as u32, m.r8(sp + 3) as u32);
    stack_pop(m, 0x10);
    let o = ((g << 2) & 0x3e0) | r >> 3 | (b >> 3) << 10 | (a >> 7) << 15;
    m.w16(p, o as u16);
}

/// FUN_001ed3a8: allocate (FUN_001ae318 {0xfff4, 16}) and clear both resident hashes
pub fn ee_sub_1ed3a8<M: QMem + ?Sized>(m: &mut M) {
    let sp = stack_push(m, 0x10);
    for h in [HTEX, HCLUT] {
        m.w32(sp, 0xfff4);
        m.w32(sp + 4, 0x10);
        let p = m.call(0x001a_e318, &[sp], &[]).0;
        m.w32(h, p);
        m.call(0x0023_33b4, &[p, 0, 0xfff4], &[]);
    }
    stack_pop(m, 0x10);
    m.w32(0x003d_d014, 0);
    m.w32(0x003d_d00c, 0);
}

/// FUN_001ed448: forget both hashes
pub fn ee_sub_1ed448<M: QMem + ?Sized>(m: &mut M) {
    for a in [HTEX, HCLUT, 0x003d_d014, 0x003d_d00c] {
        m.w32(a, 0);
    }
}

fn h_insert<M: QMem + ?Sized>(m: &mut M, h: u32, e: u32) {
    let mut i = (m.r32(e) as i32) % HSLOTS;
    loop {
        let s = m.r32(h).wrapping_add((i as u32) << 2);
        if m.r32(s) == 0 {
            m.w32(s, e);
            return;
        }
        i = (i + 1) % HSLOTS;
    }
}

fn h_find<M: QMem + ?Sized>(m: &mut M, h: u32, id: u32) -> u32 {
    if m.r32(h) == 0 {
        return 0;
    }
    let mut i = (id as i32) % HSLOTS;
    loop {
        let s = m.r32(h).wrapping_add((i as u32) << 2);
        let e = m.r32(s);
        if e == 0 || m.r32(e) == id {
            return e;
        }
        i = (i + 1) % HSLOTS;
    }
}

/// FUN_001ed470: hash texture `e` (id +0) into DAT_003dd010 (linear probing)
pub fn ee_sub_1ed470<M: QMem + ?Sized>(m: &mut M, e: u32) {
    h_insert(m, HTEX, e)
}

/// FUN_001ed4e0: resident texture with id `id` (0 when absent / no hash)
pub fn ee_sub_1ed4e0<M: QMem + ?Sized>(m: &mut M, id: u32) -> u32 {
    h_find(m, HTEX, id)
}

/// FUN_001ed560: hash CLUT `e` into DAT_003dd018
pub fn ee_sub_1ed560<M: QMem + ?Sized>(m: &mut M, e: u32) {
    h_insert(m, HCLUT, e)
}

/// FUN_001ed5d0: resident CLUT with id `id`
pub fn ee_sub_1ed5d0<M: QMem + ?Sized>(m: &mut M, id: u32) -> u32 {
    h_find(m, HCLUT, id)
}

/// FUN_001ed650: TU static init (1, 0xffff): six ref-counted pools (FUN_0011cca8 on first use):
/// 0x4520a8 {0x14, 0x2000}, 0x4520d0 {0x18, 0x800}, 0x4520f8 {0x28, 0x200}, 0x452120 {0x1c,
/// 0x800}, 0x452148 {0xc, 200}, 0x452170 {0x14, 0x100}, align 16 (counters 0x2dd2e0..)
pub fn ee_sub_1ed650<M: QMem + ?Sized>(m: &mut M, init: u32, prio: u32) {
    if prio != 0xffff || init == 0 {
        return;
    }
    let pools = [
        (0x004520a8u32, 0x14u32, 0x2000u32),
        (0x004520d0, 0x18, 0x800),
        (0x004520f8, 0x28, 0x200),
        (0x00452120, 0x1c, 0x800),
        (0x00452148, 0xc, 200),
        (0x00452170, 0x14, 0x100),
    ];
    for (i, (p, sz, n)) in pools.iter().enumerate() {
        let c = 0x002d_d2e0 + 4 * i as u32;
        let v = m.r32(c).wrapping_add(1);
        m.w32(c, v);
        if v == 1 {
            m.call(0x0011_cca8, &[*p, *sz, *n, 0x10], &[]);
        }
    }
}

/// FUN_001ed788: FUN_001ed650(1, 0xffff)
pub fn ee_sub_1ed788<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_1ed650(m, 1, 0xffff)
}

/// FUN_001ed960: register texture animation `o` with (a, d): a record of 0x3da798 (count
/// DAT_003da790 < 0x200; list node +0x10 FUN_0011a058, appended to `o`+0x34, +0xc = o) or a stack
/// scratch record; FUN_00226e88(d, 0xd, 1, a, rec, rec + 8), rec+4 = o + 0x24, FUN_001ec718(rec)
pub fn ee_sub_1ed960<M: QMem + ?Sized>(m: &mut M, o: u32, a: u32, d: u32) -> u32 {
    let sp = stack_push(m, 0x20);
    let n = m.r32(0x003d_a790) as i32;
    let r = if n < 0x200 {
        m.w32(0x003d_a790, (n + 1) as u32);
        let r = (n as u32) * 0x14 + 0x003d_a798;
        m.call(0x0011_a058, &[r + 0x10], &[]);
        m.call(0x0011_a098, &[o + 0x34, r], &[]);
        m.w32(r + 0xc, o);
        r
    } else {
        sp
    };
    m.call(0x0022_6e88, &[d, 0xd, 1, a, r, r + 8], &[]);
    m.w32(r + 4, o + 0x24);
    ee_sub_1ec718(m, r);
    stack_pop(m, 0x20);
    1
}

/// C2d92a0 slot 8 (FUN_001eda58): 0.0
pub fn ee_sub_1eda58() -> f32 {
    0.0
}

/// FUN_001eda68: when `o` has no active animations (+0x34 list empty, FUN_0011a1e8), frame +0x28 =
/// clamp(k, 0, +0x1c - 1), dirty +0x2c on change
pub fn ee_sub_1eda68<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32) {
    if m.call(0x0011_a1e8, &[o + 0x34], &[]).0 != 0 {
        return;
    }
    let hi = m.r32(o + 0x1c).wrapping_sub(1);
    let v = if (k as i32) < 0 {
        0
    } else if (k as i32) <= hi as i32 {
        k
    } else {
        hi
    };
    if m.r16(o + 0x28) as u32 != v & 0xffff {
        m.w16(o + 0x28, v as u16);
        m.w32(o + 0x2c, 1);
    }
}

/// FUN_001edad8: likewise the UV scroll +0x24 / +0x26 = 12-bit fixed (x * 4096 + 0.5) & 0xfff
pub fn ee_sub_1edad8<M: QMem + ?Sized>(m: &mut M, u: f32, v: f32, o: u32) {
    if m.call(0x0011_a1e8, &[o + 0x34], &[]).0 != 0 {
        return;
    }
    let a = ((u * 4096.0 + 0.5) as i32 & 0xfff) as i64;
    let b = ((v * 4096.0 + 0.5) as i32 & 0xfff) as i64;
    if m.r16(o + 0x24) as i16 as i64 != a || m.r16(o + 0x26) as i16 as i64 != b {
        m.w16(o + 0x24, a as u16);
        m.w32(o + 0x2c, 1);
        m.w16(o + 0x26, b as u16);
    }
}
