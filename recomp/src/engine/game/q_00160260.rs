//! WP-Q5: run_00160260 queue package: the double-buffered render-frame hand-off between the game thread
//! and the GS submit thread (frame records 0x2dea50 + 0x50 * i, i < 2; semaphores DAT_002deb18 /
//! DAT_002deb1c; DMA chains 0x2dea00 / 0x2dea28 built through FUN_00157330.. / FUN_00166bd8..), the
//! deferred-callback lists of a frame (+0x34 / +0x40), the bone-rig frame update FUN_001604e8, the
//! interlace field-copy chain FUN_00160f98 and a few save / flag helpers. 1:1 against `QMem`; the
//! kernel calls (WaitSema 0x234a60, SignalSema 0x234a40, AddIntcHandler 0x234700, EnableIntc 0x235178,
//! CreateSema 0x21efd0, 0x2355e0) and DMA kicks stay routed by address. 64-bit GS register values are
//! passed as (lo, hi) argument pairs.
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem};

const WAIT: u32 = 0x0023_4a60;
const SIGNAL: u32 = 0x0023_4a40;
/// DAT_002dea4c current game-side frame record, DAT_002deb14 current submit-side frame record
const CUR: u32 = 0x002d_ea4c;
const SUB: u32 = 0x002d_eb14;
const FRAMES: u32 = 0x002d_ea50;

fn gs<M: QMem + ?Sized>(m: &mut M, pkt: u32, reg: u32, v: u64) {
    m.call(0x0016_6e90, &[pkt, reg, v as u32, (v >> 32) as u32], &[]);
}

/// FUN_001604e8: rig frame update for `p` = {src, dst, pos flag, rot flag}: with +8 the source position
/// (+0x70) goes through the inverse bind pose of dst+0xa0 (+0x40) into dst+0x30 (w = 1); with +0xc the
/// source rotation rows (+0x40..) go through FUN_0011b340 of it and are orthonormalised (FUN_0011b3d0)
/// into dst rows 0..2 (w = 0); stamps dst+0xa4 with ++DAT_003e11e8
pub fn ee_sub_1604e8<M: QMem + ?Sized>(m: &mut M, _a0: u32, p: u32) {
    let sp = stack_push(m, 0x100);
    if m.r32(p + 8) != 0 {
        let d = m.r32(p + 4);
        let k = m.r32(d + 0xa0);
        m.call(0x0011_c318, &[sp + 0x30, k + 0x40], &[]);
        let src = m.r32(p);
        let v = m.rv(src + 0x70);
        let r = [m.rv(sp + 0x30), m.rv(sp + 0x40), m.rv(sp + 0x50), m.rv(sp + 0x60)];
        let mut o = [0.0f32; 4];
        for c in 0..3 {
            o[c] = r[0][c] * v[0] + r[1][c] * v[1] + r[2][c] * v[2] + r[3][c];
        }
        let d = m.r32(p + 4);
        m.call(0x0025_6d18, &[d + 0x30], &[o]);
        m.wf(d + 0x3c, 1.0);
    }
    if m.r32(p + 0xc) != 0 {
        let d = m.r32(p + 4);
        let k = m.r32(d + 0xa0);
        m.call(0x0011_b340, &[sp + 0x70, k + 0x40], &[]);
        let src = m.r32(p);
        let r = [m.rv(sp + 0x70), m.rv(sp + 0x80), m.rv(sp + 0x90)];
        for i in 0..3u32 {
            let v = m.rv(src + 0x40 + 0x10 * i);
            let mut o = v;
            for c in 0..3 {
                o[c] = r[0][c] * v[0] + r[1][c] * v[1] + r[2][c] * v[2];
            }
            m.wv(sp + 0x10 * i, o);
            m.wv(sp + 0x30 + 0x10 * i, o);
            m.wv(sp + 0xa0 + 0x10 * i, o);
        }
        m.call(0x0011_b3d0, &[sp], &[]);
        let d = m.r32(p + 4);
        for i in 0..3u32 {
            let v = m.rv(sp + 0x10 * i);
            m.wv(d + 0x10 * i, v);
            m.w32(d + 0x10 * i + 0xc, 0);
        }
    }
    let n = m.r32(0x003e_11e8).wrapping_add(1);
    m.w32(0x003e_11e8, n);
    let d = m.r32(p + 4);
    m.w32(d + 0xa4, n);
    stack_pop(m, 0x100);
}

/// FUN_001607a0: install the GS interrupt handler 0x167f78 (cause 5), create the two frame semaphores,
/// init the two deferred lists and start the submit thread
pub fn ee_sub_1607a0<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x0023_4700, &[5, 0x0016_7f78, 0], &[]);
    m.call(0x0023_5178, &[5], &[]);
    let a = m.call(0x0021_efd0, &[0, 2], &[]).0;
    m.w32(0x002d_eb18, a);
    let b = m.call(0x0021_efd0, &[2, 2], &[]).0;
    m.w32(0x002d_eb1c, b);
    m.call(0x0011_a1f8, &[0x002d_eaf0, 0x454], &[]);
    m.call(0x0011_a1f8, &[0x002d_eb00, 0x1090], &[]);
    let t = m.r32(0x0044_25d4);
    m.call(0x0023_55e0, &[t, 0], &[]);
}

/// next index of a 2-entry ring counter at `at`; returns the frame record for the old index
fn ring<M: QMem + ?Sized>(m: &mut M, at: u32) -> u32 {
    let i = m.r32(at);
    let n = i.wrapping_add(1);
    m.w32(at, if n < 2 { n } else { 0 });
    FRAMES.wrapping_add(i.wrapping_mul(0x50))
}

/// FUN_00160828: begin a game-side frame: (unless DAT_003d3980) FUN_001c2a78(3), take a free record
/// (WaitSema DAT_002deb1c, ring DAT_002deb10), flush its deferred lists, stamp the clock, open the two
/// DMA chains on its buffers
pub fn ee_sub_160828<M: QMem + ?Sized>(m: &mut M) {
    if m.r32(0x003d_3980) == 0 {
        m.call(0x001c_2a78, &[3], &[]);
    }
    let s = m.r32(0x002d_eb1c);
    m.call(WAIT, &[s], &[]);
    let f = ring(m, 0x002d_eb10);
    m.w32(CUR, f);
    ee_sub_160ce0(m, f);
    let f = m.r32(CUR);
    let clk = m.r32(0x002d_df54);
    let (a2, a1) = (m.r32(f + 8), m.r32(f + 0xc));
    m.w32(f + 0x4c, clk);
    m.w32(f + 0x30, 1);
    m.w32(f + 0x2c, 0);
    m.w32(f + 0x24, 0);
    m.call(0x0015_7330, &[0x002d_ea00, a1, a2], &[]);
    let f = m.r32(CUR);
    let (a2, a1) = (m.r32(f + 0x18), m.r32(f + 0x1c));
    m.call(0x0015_7330, &[0x002d_ea28, a1, a2], &[]);
    m.call(0x0015_7500, &[0x002d_ea00, 0xc, 0x002d_f740], &[]);
    m.call(0x0015_74b8, &[0x002d_ea00], &[]);
}

/// FUN_00160918: FUN_00157500(chain, qwords of [a, b), a)
pub fn ee_sub_160918<M: QMem + ?Sized>(m: &mut M, c: u32, a: u32, b: u32) {
    let n = b.wrapping_sub(a).wrapping_add(0xf) >> 4;
    m.call(0x0015_7500, &[c, n, a], &[]);
}

/// FUN_00160940: close chain `c`: a local 8-qword packet (FUN_00166920 / 0x157330 / 0x166bd8 / reg 0x61 /
/// 0x166968), end, FUN_00226d10 / FUN_002271c8 (packet), FUN_001576f0, the 0x25e680..0x262320 block,
/// FUN_001576b0 and FUN_00157348(c, a, b)
pub fn ee_sub_160940<M: QMem + ?Sized>(m: &mut M, c: u32, a: u32, b: u32) {
    let sp = stack_push(m, 0x100);
    let pk = sp + 0x80;
    m.call(0x0016_6920, &[pk], &[]);
    m.call(0x0015_7330, &[pk, 8, sp], &[]);
    m.call(0x0016_6bd8, &[pk, 0, 1, 0xe], &[]);
    gs(m, pk, 0x61, 0);
    m.call(0x0016_6968, &[pk], &[]);
    m.call(0x0015_74b8, &[c], &[]);
    m.call(0x0022_6d10, &[c], &[]);
    m.call(0x0022_71c8, &[c, pk], &[]);
    m.call(0x0015_76f0, &[c], &[]);
    ee_sub_160918(m, c, 0x0025_e680, 0x0026_2320);
    m.call(0x0015_76b0, &[c], &[]);
    m.call(0x0015_7348, &[c, a, b], &[]);
    stack_pop(m, 0x100);
}

/// FUN_00160a20: close chain `c` with the reg 0x61 packet and FUN_00157348(c, a, b)
pub fn ee_sub_160a20<M: QMem + ?Sized>(m: &mut M, c: u32, a: u32, b: u32) {
    m.call(0x0016_6bd8, &[c, 0, 1, 0xe], &[]);
    gs(m, c, 0x61, 0);
    m.call(0x0016_6968, &[c], &[]);
    m.call(0x0015_76b0, &[c], &[]);
    m.call(0x0015_7348, &[c, a, b], &[]);
}

/// FUN_00160a98: end the game-side frame: close both chains into the record (+0x10/+0x14, +0/+4),
/// +0x20 = qword size of chain 2 minus +0x24, move the two deferred lists into it (+0x34 / +0x40,
/// FUN_0011a200 resets), release it to the submit thread (SignalSema DAT_002deb18)
pub fn ee_sub_160a98<M: QMem + ?Sized>(m: &mut M) {
    let f = m.r32(CUR);
    ee_sub_160940(m, 0x002d_ea28, f + 0x10, f + 0x14);
    let f = m.r32(CUR);
    ee_sub_160a20(m, 0x002d_ea00, f, f + 4);
    let f = m.r32(CUR);
    let n = m.r32(f + 0x14).wrapping_add(m.r32(f + 0x10).wrapping_mul(0x10)).wrapping_sub(m.r32(f + 0x24));
    m.w32(f + 0x20, ((n as i32) >> 4) as u32);
    for (dst, src) in [(0x34u32, 0x002d_eaf0u32), (0x40, 0x002d_eb00)] {
        let f = m.r32(CUR);
        for k in 0..3 {
            let x = m.r32(src + 4 * k);
            m.w32(f + dst + 4 * k, x);
        }
        m.call(0x0011_a200, &[src], &[]);
    }
    m.w32(CUR, 0);
    let s = m.r32(0x002d_eb18);
    m.call(SIGNAL, &[s], &[]);
}

/// FUN_00160b70: GS submit thread. The original loops forever; this is one iteration: wait for a frame
/// (DAT_002deb20 = waiting), take it (ring DAT_002deb0c), FUN_001ee7b0 / FUN_0016f6b8 / FUN_00170048,
/// kick its two chains (FUN_0016f8b8 on DAT_002de594 / DAT_002de598), the field copy when +0x30 bit 0,
/// FUN_002417f0, free the record (SignalSema DAT_002deb1c), FUN_0016f590, DAT_002dedbc = 1
pub fn ee_sub_160b70<M: QMem + ?Sized>(m: &mut M) {
    m.w32(0x002d_eb20, 1);
    let s = m.r32(0x002d_eb18);
    m.call(WAIT, &[s], &[]);
    m.w32(0x002d_eb20, 0);
    let f = ring(m, 0x002d_eb0c);
    m.w32(SUB, f);
    m.call(0x001e_e7b0, &[], &[]);
    m.call(0x0016_f6b8, &[], &[]);
    m.call(0x0017_0048, &[], &[]);
    let (c0, c1) = (m.r32(0x002d_e594), m.r32(0x002d_e598));
    let f = m.r32(SUB);
    let x = m.r32(f + 0x24);
    m.call(0x0016_f8b8, &[c0, x], &[]);
    let f = m.r32(SUB);
    let x = m.r32(f + 4);
    m.call(0x0016_f8b8, &[c1, x], &[]);
    let f = m.r32(SUB);
    if m.r32(f + 0x30) & 1 != 0 {
        ee_sub_160f98(m);
    }
    let c0 = m.r32(0x002d_e594);
    m.call(0x0024_17f0, &[c0, 0, 0], &[]);
    let s = m.r32(0x002d_eb1c);
    m.call(SIGNAL, &[s], &[]);
    m.call(0x0016_f590, &[], &[]);
    m.w32(0x002d_edbc, 1);
}

/// FUN_00160c80: re-kick frame `f` (chain 2 always, chain 1 when `both`)
pub fn ee_sub_160c80<M: QMem + ?Sized>(m: &mut M, f: u32, both: u32) {
    m.call(0x001e_e7b0, &[], &[]);
    m.call(0x0017_0048, &[], &[]);
    let c0 = m.r32(0x002d_e594);
    let x = m.r32(f + 0x24);
    m.call(0x0016_f8b8, &[c0, x], &[]);
    if both != 0 {
        let c1 = m.r32(0x002d_e598);
        let x = m.r32(f + 4);
        m.call(0x0016_f8b8, &[c1, x], &[]);
    }
}

/// walk list `l` {head, ?, link offset} with the iteration frame pushed on DAT_002dd2fc
fn walk<M: QMem + ?Sized>(m: &mut M, l: u32, mut each: impl FnMut(&mut M, u32)) {
    let sp = stack_push(m, 0x10);
    let off = m.r32(l + 8);
    let prev = m.r32(0x002d_d2fc);
    m.w32(0x002d_d2fc, sp);
    m.w32(sp + 0xc, prev);
    let mut n = m.r32(l);
    m.w32(sp, l);
    m.w32(sp + 4, n.wrapping_add(off));
    m.w32(sp + 8, off);
    while n != 0 {
        each(m, n);
        let np = m.r32(sp + 4);
        n = m.r32(np);
        let o = m.r32(sp + 8);
        m.w32(sp + 4, n.wrapping_add(o));
    }
    m.call(0x0011_a200, &[l], &[]);
    let prev = m.r32(sp + 0xc);
    m.w32(0x002d_d2fc, prev);
    stack_pop(m, 0x10);
}

/// FUN_00160ce0: flush frame `f`'s deferred lists (when non-empty, FUN_0011a3b0): +0x34 nodes are
/// unlinked (FUN_0011a2e8) and their vtable (+0xc) slot +0x14 called on node + slot +0x10 (i16); +0x40
/// nodes are unlinked and released to the 0x3e1518 pool (FUN_0011cd98), their +0x1088 to 0x3e1530
pub fn ee_sub_160ce0<M: QMem + ?Sized>(m: &mut M, f: u32) {
    let l = f + 0x34;
    if m.call(0x0011_a3b0, &[l], &[]).0 == 0 {
        walk(m, l, |m, n| {
            m.call(0x0011_a2e8, &[l, n], &[]);
            let vt = m.r32(n + 0xc);
            let adj = m.r16(vt + 0x10) as i16 as i32 as u32;
            let fp = m.r32(vt + 0x14);
            m.call(fp, &[n.wrapping_add(adj)], &[]);
        });
    }
    let l = f + 0x40;
    if m.call(0x0011_a3b0, &[l], &[]).0 == 0 {
        walk(m, l, |m, n| {
            let x = m.r32(n + 0x1088);
            if x != 0 {
                m.call(0x0011_cd98, &[0x003e_1530, x], &[]);
            }
            m.call(0x0011_a2e8, &[l, n], &[]);
            m.call(0x0011_cd98, &[0x003e_1518, n], &[]);
        });
    }
}

/// FUN_00160e68: drain the pipeline (take and give back both free records, spin until the submit
/// thread waits); with `redraw` re-kick the last frame three times (field copy per +0x30 bit 0,
/// FUN_0016f590; DAT_002dedbc = 3) and flush both records' lists; then FUN_001c2ba8(3), DAT_002deb14 = 0
pub fn ee_sub_160e68<M: QMem + ?Sized>(m: &mut M, redraw: u32) {
    for _ in 0..2 {
        let s = m.r32(0x002d_eb1c);
        m.call(WAIT, &[s], &[]);
    }
    for _ in 0..2 {
        let s = m.r32(0x002d_eb1c);
        m.call(SIGNAL, &[s], &[]);
    }
    while m.r32(0x002d_eb20) == 0 {}
    if redraw != 0 {
        if m.r32(SUB) != 0 {
            for _ in 0..3 {
                let f = m.r32(SUB);
                ee_sub_160c80(m, f, 1);
                let f = m.r32(SUB);
                if m.r32(f + 0x30) & 1 != 0 {
                    ee_sub_160f98(m);
                }
                m.call(0x0016_f590, &[], &[]);
            }
            m.w32(0x002d_edbc, 3);
        }
        let mut f = FRAMES;
        while f < 0x002d_eaf0 {
            ee_sub_160ce0(m, f);
            f += 0x50;
        }
    }
    m.call(0x001c_2ba8, &[3], &[]);
    m.w32(SUB, 0);
}

/// FUN_00160f98: interlace field copy (unless DAT_002deb24 / DAT_003e12c8): a 100-qword chain at sp+0x30
/// that sets FRAME_1 from the displayed buffer's DISPFB (DAT_002dedb0 bit 0 ? 0x2debe0 : 0x2decd0), the
/// copy state (regs 0x14 / 8 / 0x47 / 0x42 / 1 / 0x40) and 32 strips of 0x20 columns (sprite pairs, odd
/// field offset 0x10 unless DAT_002dedb8 bit 0), restores the scissor and kicks it on DAT_002de598
pub fn ee_sub_160f98<M: QMem + ?Sized>(m: &mut M) {
    if m.r32(0x002d_eb24) != 0 || m.r32(0x003e_12c8) != 0 {
        return;
    }
    let sp = stack_push(m, 0x670);
    let (pk, buf) = (sp, sp + 0x30);
    m.call(0x0016_6920, &[pk], &[]);
    m.call(0x0015_7330, &[pk, 100, buf], &[]);
    m.call(0x0015_74b8, &[pk], &[]);
    m.call(0x0016_6bd8, &[pk, 0, 1, 0xe], &[]);
    gs(m, pk, 0x3f, 0);
    let src = if m.r32(0x002d_edb0) & 1 == 0 { 0x002d_ecd0 } else { 0x002d_ebe0 };
    let d = m.r64(src);
    let v = (d & 0x1ff) << 5 | (d >> 0x10 & 0x3f) << 0xe | (d >> 0x18 & 0x3f) << 0x14 | 0x2_a800_0000;
    gs(m, pk, 6, v);
    gs(m, pk, 0x14, 0x61);
    gs(m, pk, 8, 5);
    gs(m, pk, 0x47, 0x33001);
    gs(m, pk, 0x42, 0x30_0000_0064);
    gs(m, pk, 1, 0x3f80_0000_8080_8080);
    gs(m, pk, 0x40, 0x00de_0001_01ff_0000);
    m.call(0x0016_6bd8, &[pk, 0x156, 2, 0x43], &[]);
    let odd = if m.r32(0x002d_edb8) & 1 != 0 { 0 } else { 0x10 };
    let mut x = 0i32;
    loop {
        let x2 = x + 0x20;
        m.call(0x0016_6ce0, &[pk, ((x * 0x10) as u32) | 8, odd], &[]);
        m.call(0x0016_6e08, &[pk, ((x + 0x700) * 0x10) as u32, 0x7900, 0, 0], &[]);
        m.call(0x0016_6ce0, &[pk, ((x2 * 0x10) as u32) | 8, odd + 0xe00], &[]);
        m.call(0x0016_6e08, &[pk, ((x + 0x720) * 0x10) as u32, 0x8700, 0, 0], &[]);
        x = x2;
        if x >= 0x200 {
            break;
        }
    }
    m.call(0x0016_6bd8, &[pk, 0, 1, 0xe], &[]);
    gs(m, pk, 0x40, 0x00df_0000_01ff_0000);
    gs(m, pk, 0x61, 0);
    m.call(0x0016_6968, &[pk], &[]);
    m.call(0x0015_76b0, &[pk], &[]);
    m.call(0x0015_76f0, &[pk], &[]);
    let c1 = m.r32(0x002d_e598);
    m.call(0x0016_f8b8, &[c1, buf], &[]);
    stack_pop(m, 0x670);
}

/// FUN_00161238: TU static init that also constructs the two frame chains (FUN_00166920(0x2dea00),
/// FUN_00226c10(0x2dea28)) before the shared pool registration
pub fn ee_sub_161238<M: QMem + ?Sized>(m: &mut M, init: u32, prio: u32) {
    if prio != 0xffff || init == 0 {
        return;
    }
    m.call(0x0016_6920, &[0x002d_ea00], &[]);
    m.call(0x0022_6c10, &[0x002d_ea28], &[]);
    crate::engine::game::q_0021c6c8::ee_sub_21c6c8(m, 1, 0xffff);
}

/// FUN_00161388: `ee_sub_161238(1, 0xffff)`
pub fn ee_sub_161388<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_161238(m, 1, 0xffff);
}

/// FUN_001613f8: reset save slot record `p` (0x1c bytes zeroed, +0x10..+0x18 = -1), FUN_001d8d80 on
/// 0x3d4a60, FUN_00154a60(0x2de578), DAT_002e1e00 = 0
pub fn ee_sub_1613f8<M: QMem + ?Sized>(m: &mut M, p: u32) {
    m.call(0x0023_33b4, &[p, 0, 0x1c], &[]);
    m.w32(p + 0x18, !0);
    m.w32(p + 0x10, !0);
    m.w32(p + 0x14, !0);
    m.call(0x001d_8d80, &[0x003d_4a60, 0], &[]);
    m.call(0x0015_4a60, &[0x002d_e578], &[]);
    m.w32(0x002e_1e00, 0);
}

/// FUN_00161458: FUN_00154a80(0x2de578)
pub fn ee_sub_161458<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x0015_4a80, &[0x002d_e578], &[]);
}

/// FUN_00161798: value (+4) of the DAT_003e0fb0 key in the 0x3d4a60 table (FUN_002581c0, released)
pub fn ee_sub_161798<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let sp = stack_push(m, 0x10);
    let k = m.r32(0x003e_0fb0);
    m.call(0x0025_81c0, &[sp, k, 0x003d_4a60], &[]);
    let v = m.r32(sp + 4);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x10);
    v
}

/// FUN_001617e0: store {0x15, v} under the DAT_003e0fb0 key of 0x3d4a60 (FUN_001291a8 / FUN_00258200,
/// temporaries released), then FUN_00161868(0x3dd490)
pub fn ee_sub_1617e0<M: QMem + ?Sized>(m: &mut M, _a0: u32, v: u32) {
    let sp = stack_push(m, 0x30);
    m.w32(sp + 4, v);
    m.w32(sp, 0x15);
    let k = m.r32(0x003e_0fb0);
    m.call(0x0012_91a8, &[sp + 0x10, sp], &[]);
    m.call(0x0025_8200, &[sp + 0x20, k, sp + 0x10, 0x003d_4a60], &[]);
    m.call(0x0012_9ec8, &[sp + 0x20], &[]);
    m.call(0x0012_9ec8, &[sp + 0x10], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0016_1868, &[0x003d_d490], &[]);
    stack_pop(m, 0x30);
}
