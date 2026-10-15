//! WP-Q5: run_002155c0 queue package: the rich-text (HUD font) state machine. Text object
//! {+0 string, +4 cursor, +8 font, +0xc / +0x10 fonts for "&." / "&x", +0x14..+0x1f colours
//! (current / base / alt), +0x20..+0x3c resolved RGBA + alpha, +0x40..+0x5c colour bias quads,
//! +0x60 colour-changed flag, +0x64 / +0x68 screen offset, +0x6c..+0x78 box, +0x7c / +0x80 scale,
//! +0x84 line height, +0x88 advance scale, +0x8c / +0x90 v / h alignment, +0x94 layer}; font
//! {+0x14 glyphs (0x40 each), +0x1c advances, +0x20 count, +0x24 sorted char codes}. The draw path
//! (FUN_002163b0 / FUN_00216750) builds the original DMA chain at DAT_002dea34 1:1 (the Bevy HUD
//! renders text itself through ui::hud::text, WP-I3); glyph emission is FUN_00217058 (not here).
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem};

/// DAT_002dea34: DMA packet cursor
const DMA: u32 = 0x002d_ea34;
const SX: f32 = 0.8;
const SY: f32 = 0.454_545_47;
/// GS fixed-point origin offset added to screen x / y
const ORG: f32 = 786_432.0;

fn advance<M: QMem + ?Sized>(m: &mut M, o: u32, c: u32) -> f32 {
    let f = m.r32(o + 8);
    ee_sub_216b80(m, f, c)
}

/// FUN_002155c0: entry `e` of kind 7 (+4): clear +0x1c / +0x20 and, when its texture (+0x10,
/// FUN_001ec880) exists, upload it through a 0x10-quad GS packet (FUN_00166920 / FUN_001572e0 /
/// FUN_00166bd8(0, 1, 0xe) / regs 6 = TEX0 (64-bit, passed lo / hi), 0x14 = 0x60, 8 = 5 /
/// FUN_00166968 / FUN_001573f8(o+0xc4 -> e+0x1c, e+0x20)) inside a scratch frame
pub fn ee_sub_2155c0<M: QMem + ?Sized>(m: &mut M, o: u32, e: u32) {
    let fr = stack_push(m, 0x40);
    let buf = fr + 0x10;
    m.call(0x001a_e478, &[fr], &[]);
    if m.r32(e + 4) == 7 {
        m.w32(e + 0x1c, 0);
        m.w32(e + 0x20, 0);
        let t = m.r32(e + 0x10);
        let t = m.call(0x001e_c880, &[t], &[]).0;
        if t != 0 {
            m.call(0x0016_6920, &[buf], &[]);
            m.call(0x0015_72e0, &[buf, 0x10], &[]);
            m.call(0x0016_6bd8, &[buf, 0, 1, 0xe], &[]);
            let p = m.r32(t + 0x14);
            let (lo, hi) = (m.r32(p), m.r32(p + 4));
            m.call(0x0016_6e90, &[buf, 6, lo, hi], &[]);
            m.call(0x0016_6e90, &[buf, 0x14, 0x60], &[]);
            m.call(0x0016_6e90, &[buf, 8, 5], &[]);
            m.call(0x0016_6968, &[buf], &[]);
            let a = m.r32(o + 0xc4);
            m.call(0x0015_73f8, &[buf, a, e + 0x1c, 0, e + 0x20], &[]);
        }
    }
    m.call(0x001a_e4a0, &[fr, 2], &[]);
    stack_pop(m, 0x40);
}

/// FUN_00215960: set flag byte `v` (+0x61) of item `it` in menu `o`: kinds 1 / 3 / 6 / 7 store it
/// and refresh (FUN_0011a098(o+8 -> +0xfc)); kind 4 forwards to FUN_002185f8(row +0x10 of the
/// o+8 -> +0x84 table (0x30 stride), it+0x58, it+0x61)
pub fn ee_sub_215960<M: QMem + ?Sized>(m: &mut M, o: u32, it: u32, v: u8) {
    match m.r32(o + 4) {
        1 | 3 | 6 | 7 => {
            m.w8(it + 0x61, v);
            let p = m.r32(o + 8) + 0xfc;
            m.call(0x0011_a098, &[p], &[]);
        }
        4 => {
            let t = m.r32(o + 8);
            let t = m.r32(t + 0x84);
            let r = t.wrapping_add(m.r32(o + 0x10).wrapping_mul(0x30));
            let (a, b) = (m.r32(it + 0x58), m.r8(it + 0x61) as u32);
            m.call(0x0021_85f8, &[r, a, b], &[]);
        }
        _ => {}
    }
}

/// FUN_00215a50: 1-based index of char `c` in the font's sorted code table (+0x24, +0x20 entries)
/// by binary search; 0 when absent
pub fn ee_sub_215a50<M: QMem + ?Sized>(m: &mut M, f: u32, c: u32) -> u32 {
    let c = (c & 0xff) as i32;
    let mut hi = m.r32(f + 0x20) as i32 - 1;
    let mut lo = 0;
    if hi < 0 {
        return 0;
    }
    let mut s = hi;
    loop {
        let mid = s / 2;
        let t = m.r32(f + 0x24);
        let k = m.r32(t.wrapping_add(4 * mid as u32)) as i32;
        if c == k {
            return mid as u32 + 1;
        }
        if c - k < 1 {
            hi = mid - 1;
        } else {
            lo = mid + 1;
        }
        s = hi + lo;
        if lo > hi {
            return 0;
        }
    }
}

/// FUN_00215ac0: init text object `o` with font `font` at size `sz` (+0x22, advance +0x21 = sz *
/// 51.2), defaults (scales 1, box 1792 x 1936, layer 6, colours from DAT_003e1078+0x54, white
/// 0x80808080, bias quads 0x2a2e10 / 0x2a2e20), then reset (FUN_00215ca8)
pub fn ee_sub_215ac0<M: QMem + ?Sized>(m: &mut M, sz: f32, o: u32, font: u32) -> u32 {
    m.w32(o, font);
    m.wf(o + 0x80, 1.0);
    m.w32(o + 0x64, 0x44e0_0000);
    m.w32(o + 0x68, 0x44f2_0000);
    m.wf(o + 0x84, sz * 51.2);
    m.wf(o + 0x7c, 1.0);
    m.wf(o + 0x88, sz);
    m.w32(o + 0x8c, 1);
    m.w32(o + 0x94, 6);
    for a in [0x6c, 0x70, 0x74, 0x78] {
        m.w32(o + a, 0);
    }
    m.w32(o + 0x90, 1);
    let g = m.r32(0x003e_1078);
    let c = m.r32(g + 0x54);
    m.w32(o + 8, c);
    m.w32(o + 0xc, c);
    m.w32(o + 0x18, 0x8080_8080);
    m.w32(o + 0x10, c.wrapping_add(0x40));
    let (a, b) = (m.rv(0x002a_2e10), m.rv(0x002a_2e20));
    m.wv(o + 0x40, a);
    m.wv(o + 0x50, b);
    ee_sub_215ca8(m, o, 0);
    o
}

/// FUN_00215ca8: rewind `o` (new string `s` when non-zero): cursor = start, font / colour = base,
/// alt colour 0x80000000 with the base alpha, resolve (FUN_002160d8)
pub fn ee_sub_215ca8<M: QMem + ?Sized>(m: &mut M, o: u32, s: u32) {
    if s != 0 {
        m.w32(o, s);
    }
    let v = m.r32(o + 0xc);
    m.w32(o + 8, v);
    let v = m.r32(o);
    m.w32(o + 4, v);
    let v = m.r32(o + 0x18);
    m.w32(o + 0x14, v);
    m.w32(o + 0x1c, 0x8000_0000);
    let a = m.r8(o + 0x17);
    m.w8(o + 0x1f, a);
    ee_sub_2160d8(m, o);
}

/// FUN_00215d00: base and current colour = *c, resolve
pub fn ee_sub_215d00<M: QMem + ?Sized>(m: &mut M, o: u32, c: u32) {
    let v = m.r32(c);
    m.w32(o + 0x18, v);
    m.w32(o + 0x14, v);
    ee_sub_2160d8(m, o);
}

/// FUN_00215d40: next char of `o`, consuming the markup: "&&" -> '&', "&." -> base font / colour,
/// "&x" -> alt font / colour (+0x60 = 1, resolve)
pub fn ee_sub_215d40<M: QMem + ?Sized>(m: &mut M, o: u32) -> u8 {
    m.w32(o + 0x60, 0);
    loop {
        let p = m.r32(o + 4);
        if m.r8(p) != b'&' {
            break;
        }
        let n = m.r8(p + 1);
        if n == b'&' {
            m.w32(o + 4, p + 1);
            break;
        }
        m.w32(o + 4, p + 2);
        let (f, c) = if n == b'.' { (0xc, 0x18) } else { (0x10, 0x1c) };
        let v = m.r32(o + f);
        m.w32(o + 8, v);
        let v = m.r32(o + c);
        m.w32(o + 0x14, v);
        m.w32(o + 0x60, 1);
        ee_sub_2160d8(m, o);
    }
    let p = m.r32(o + 4);
    m.w32(o + 4, p + 1);
    m.r8(p)
}

/// FUN_00215e30: visible char count
pub fn ee_sub_215e30<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    ee_sub_215ca8(m, o, 0);
    let mut n = 0;
    while ee_sub_215d40(m, o) != 0 {
        n += 1;
    }
    n
}

/// FUN_00215e80: truncate the string after `n` + 1 visible chars (0 written at the cursor)
pub fn ee_sub_215e80<M: QMem + ?Sized>(m: &mut M, o: u32, mut n: u32) {
    ee_sub_215ca8(m, o, 0);
    while ee_sub_215d40(m, o) != 0 {
        let more = n != 0;
        n = n.wrapping_sub(1);
        if !more {
            break;
        }
    }
    let p = m.r32(o + 4);
    m.w8(p, 0);
}

/// FUN_00215ed8: width of the whole string (sum of advance * +0x88)
pub fn ee_sub_215ed8<M: QMem + ?Sized>(m: &mut M, o: u32) -> f32 {
    let mut w = 0.0;
    ee_sub_215ca8(m, o, 0);
    loop {
        let c = ee_sub_215d40(m, o);
        if c == 0 {
            return w;
        }
        w += advance(m, o, c as u32) * m.rf(o + 0x88);
    }
}

/// FUN_002160d8: resolve the current colour +0x14 (bytes r g b a) + bias +0x40.. into floats
/// +0x20..+0x28 (capped at 255), copy bias quad 2 (+0x4c..+0x58) to +0x2c.., alpha +0x3c =
/// clamp01(+0x5c * a / 127)
pub fn ee_sub_2160d8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let a = m.rf(o + 0x5c) * 0.007_874_016 * m.r8(o + 0x17) as f32;
    let r = m.rf(o + 0x40) + m.r8(o + 0x14) as f32;
    let b = m.rf(o + 0x48) + m.r8(o + 0x16) as f32;
    let g = m.rf(o + 0x44) + m.r8(o + 0x15) as f32;
    let cap = |x: f32| if x < 255.0 { x } else { 255.0 };
    m.wf(o + 0x28, cap(b));
    m.wf(o + 0x20, cap(r));
    m.wf(o + 0x24, cap(g));
    for k in 0..4 {
        let v = m.r32(o + 0x4c + 4 * k);
        m.w32(o + 0x2c + 4 * k, v);
    }
    let a = if a < 0.0 {
        0.0
    } else if 1.0 < a {
        1.0
    } else {
        a
    };
    m.wf(o + 0x3c, a);
}

/// FUN_002161c8: widest line ('\n' separated) of the string
pub fn ee_sub_2161c8<M: QMem + ?Sized>(m: &mut M, o: u32) -> f32 {
    let (mut best, mut cur) = (0.0f32, 0.0f32);
    ee_sub_215ca8(m, o, 0);
    loop {
        let c = ee_sub_215d40(m, o);
        if c == 0 {
            break;
        }
        if c == b'\n' {
            best = if best < cur { cur } else { best };
            cur = 0.0;
        } else {
            cur += advance(m, o, c as u32) * m.rf(o + 0x88);
        }
    }
    if best < cur {
        cur
    } else {
        best
    }
}

/// FUN_00216260: text extent: *pw = widest line, *ph = lines (FUN_00215f40) * line height
pub fn ee_sub_216260<M: QMem + ?Sized>(m: &mut M, o: u32, pw: u32, ph: u32) {
    let n = m.call(0x0021_5f40, &[o], &[]).0 as i32;
    let w = ee_sub_2161c8(m, o);
    let lh = m.rf(o + 0x84);
    if pw != 0 {
        m.wf(pw, w);
    }
    if ph != 0 {
        m.wf(ph, n as f32 * lh);
    }
}

/// FUN_002162d8: screen matrix `d` (0x40) from rect `r` {sx, sy, ?, ?, x, y}: diag scaled by
/// 0.8 / 0.4545 (PS2 pixel aspect), translation + screen offset (+0x64 / +0x68) + GS origin
pub fn ee_sub_2162d8<M: QMem + ?Sized>(m: &mut M, o: u32, d: u32, r: u32) {
    m.call(0x0023_33b4, &[d, 0, 0x40], &[]);
    let v = m.rf(r) * SX;
    m.wf(d, v);
    let v = m.rf(r + 4) * SY;
    m.wf(d + 4, v);
    let v = m.rf(r + 8) * SX;
    m.wf(d + 0x10, v);
    let v = m.rf(r + 0xc) * SY;
    m.wf(d + 0x14, v);
    let v = m.rf(r + 0x10) * SX + m.rf(o + 100) + ORG;
    m.wf(d + 0x20, v);
    let y = m.rf(r + 0x14);
    let oy = m.rf(o + 0x68);
    m.wf(d + 0x3c, 1.0);
    m.wf(d + 0x28, 1.0);
    m.wf(d + 0x24, y * SY + oy + ORG);
}

/// pad the DMA cursor with zero words until (cursor & 0xf) == `to`
fn pad<M: QMem + ?Sized>(m: &mut M, to: u32) {
    loop {
        let p = m.r32(DMA);
        if p & 0xf == to {
            return;
        }
        m.w32(p, 0);
        m.w32(DMA, p + 4);
    }
}

/// queue the DMA chain at `start` on layer `layer` with depth bias `z` (FUN_001cd998 entry
/// {0x15, z + (layer << 18), start & 0x0fffffff, 0})
fn submit<M: QMem + ?Sized>(m: &mut M, start: u32, z: f32, layer: u32) {
    let e = m.call(0x001c_d998, &[], &[]).0;
    m.w32(e, 0x15);
    m.w32(e + 8, start & 0x0fff_ffff);
    m.w32(e + 0xc, 0);
    m.wf(e + 4, z + ((layer << 0x12) as i32) as f32);
}

/// FUN_002163b0: draw `o` in box `bx` {x, y, w, h, colour, valign, halign} on `layer`: take the
/// box (+0x6c..), colour (FUN_00215d00), build the matrix (FUN_002162d8), emit the glyphs
/// (FUN_00216750), then a second chain: FLUSH, DIRECT 3 (0x442210), DIRECT 0x25 (0x2df010), FLUSH,
/// end tag
pub fn ee_sub_2163b0<M: QMem + ?Sized>(m: &mut M, o: u32, bx: u32, layer: u32) {
    if bx == 0 {
        return;
    }
    for k in 0..4 {
        let v = m.r32(bx + 4 * k);
        m.w32(o + 0x6c + 4 * k, v);
    }
    let v = m.r32(bx + 0x14);
    m.w32(o + 0x90, v);
    let v = m.r32(bx + 0x18);
    m.w32(o + 0x94, layer);
    m.w32(o + 0x8c, v);
    let sp = stack_push(m, 0xb0);
    let (col, rect, mat, st) = (sp, sp + 0x10, sp + 0x30, sp + 0x70);
    let c = m.r32(bx + 0x10);
    m.w32(col, c);
    ee_sub_215d00(m, o, col);
    m.call(0x0023_33b4, &[rect, 0, 0x20], &[]);
    let v = m.rf(o + 0x80) * m.rf(o + 0x88);
    m.wf(rect + 0xc, v);
    let v = m.rf(o + 0x7c) * m.rf(o + 0x88);
    m.wf(rect, v);
    let v = m.r32(o + 0x6c);
    m.w32(rect + 0x10, v);
    let v = m.rf(o + 0x70) + m.rf(o + 0x84) * m.rf(o + 0x80);
    m.wf(rect + 0x14, v);
    ee_sub_2162d8(m, o, mat, rect);
    let (a, b) = (m.rv(0x002a_2e20), m.rv(0x002a_2e10));
    m.wv(st + 0x20, a);
    m.wv(st + 0x10, b);
    m.wf(st + 8, 16.0);
    ee_sub_216750(m, o, mat, st);
    stack_pop(m, 0xb0);

    let tag = m.r32(DMA);
    m.w32(tag, 0x1000_0000);
    m.w32(tag + 4, 0);
    m.w32(tag + 0x10, 0x1100_0000);
    m.w32(DMA, tag + 0x14);
    for (src, n) in [(0x0044_2210u32, 3u32), (0x002d_f010, 0x25)] {
        pad(m, 0xc);
        let p = m.r32(DMA);
        m.w32(p, 0x5000_0000 | n);
        m.w32(DMA, p + 4);
        m.call(0x001a_e528, &[p + 4, src, n], &[]);
        m.w32(DMA, p + 4 + n * 0x10);
    }
    let p = m.r32(DMA);
    m.w32(p, 0x1100_0000);
    m.w32(DMA, p + 4);
    pad(m, 0);
    let p = m.r32(DMA);
    let qwc = ((p.wrapping_sub(tag) as i32 >> 4) - 1) as u32;
    let t = m.r32(tag) | qwc;
    m.w32(tag, t);
    m.w32(p, 0x6000_0000);
    m.w32(p + 4, 0);
    m.w32(DMA, p + 0x10);
    let layer = m.r32(o + 0x94);
    submit(m, tag, 0.0, layer);
}

/// FUN_00216750: emit the glyphs of `o` through FUN_00217058 with matrix `mt` (+0x20 x, +0x24 y)
/// and glyph state `st` (+0 = 0, +8 depth, +0x10 / +0x20 colours): line widths first, vertical
/// alignment (+0x8c: 1 centre / 2 bottom), per line horizontal alignment (+0x90), glyphs inside the
/// screen window DAT_002c40a8..b4 only; the first two glyphs (or two after a colour change) get
/// flags 3 / colour refresh; then end tag + queue entry (depth st+8)
pub fn ee_sub_216750<M: QMem + ?Sized>(m: &mut M, o: u32, mt: u32, st: u32) {
    let start = m.r32(DMA);
    let x0 = m.rf(mt + 0x20);
    m.w32(st, 0);
    let lh = m.rf(o + 0x84) * m.rf(o + 0x80) * SY;
    let mut widths = vec![0.0f32; 32];
    let mut line = 0usize;
    ee_sub_215ca8(m, o, 0);
    loop {
        let c = ee_sub_215d40(m, o);
        if c == 0 {
            break;
        }
        if c == b'\n' {
            line += 1;
            if widths.len() <= line {
                widths.push(0.0);
            }
        } else {
            let a = advance(m, o, c as u32);
            widths[line] += a * m.rf(o + 0x88);
        }
    }
    let va = m.r32(o + 0x8c);
    if va != 0 {
        let d = m.rf(o + 0x78) * m.rf(o + 0x80) * SY - (line as i32 + 1) as f32 * lh;
        let y = m.rf(mt + 0x24);
        m.wf(mt + 0x24, if va == 2 { y + d } else { y + d * 0.5 });
    }
    ee_sub_215ca8(m, o, 0);
    let mut fl = 3u32;
    let mut new_line = true;
    let (mut n, mut until) = (0i32, 2i32);
    let mut li = 0usize;
    loop {
        let c = ee_sub_215d40(m, o);
        if c == 0 {
            break;
        }
        if m.r32(o + 0x60) != 0 {
            until = n + 3;
            fl |= 2;
        }
        if c == b'\n' {
            new_line = true;
            li += 1;
            let y = m.rf(mt + 0x24);
            m.wf(mt + 0x24, y + lh);
            continue;
        }
        if new_line {
            let ha = m.r32(o + 0x90);
            if ha == 0 {
                m.wf(mt + 0x20, x0);
            } else {
                let sc = m.rf(o + 0x7c);
                let w = widths.get(li).copied().unwrap_or(0.0) * sc * SX;
                let bw = m.rf(o + 0x74) * sc * SX;
                let d = if ha == 2 { bw - w } else { (bw - w) * 0.5 };
                m.wf(mt + 0x20, x0 + d);
            }
            new_line = false;
        }
        let f = m.r32(o + 8);
        let g = ee_sub_216b40(m, f, c as u32);
        if g == 0 {
            continue;
        }
        let (x, y) = (m.rf(mt + 0x20), m.rf(mt + 0x24));
        let vis = m.rf(0x002c_40a8) <= x
            && x <= m.rf(0x002c_40ac)
            && m.rf(0x002c_40b0) <= y
            && y <= m.rf(0x002c_40b4);
        if m.r32(g + 4) != 0 && vis {
            if fl & 2 != 0 {
                let (a, b) = (m.rv(o + 0x20), m.rv(o + 0x30));
                m.wv(st + 0x10, a);
                m.wv(st + 0x20, b);
            }
            n += 1;
            m.call(0x0021_7058, &[g, mt, st, fl], &[]);
            if until <= n {
                fl = 0;
            }
        }
        let a = advance(m, o, c as u32);
        let x = m.rf(mt + 0x20) + a * m.rf(o + 0x88) * m.rf(o + 0x7c) * SX;
        m.wf(mt + 0x20, x);
    }
    pad(m, 0);
    let p = m.r32(DMA);
    m.w32(p, 0x6000_0000);
    m.w32(p + 4, 0);
    m.w32(DMA, p + 0x10);
    let z = m.rf(st + 8);
    let layer = m.r32(o + 0x94);
    submit(m, start, z, layer);
}

/// FUN_00216b40: glyph record of char `c` (+0x14 + index * 0x40), 0 when absent
pub fn ee_sub_216b40<M: QMem + ?Sized>(m: &mut M, f: u32, c: u32) -> u32 {
    match ee_sub_215a50(m, f, c) {
        0 => 0,
        i => m.r32(f + 0x14).wrapping_add(i * 0x40),
    }
}

/// FUN_00216b80: advance of char `c` (+0x1c table), 0 when absent
pub fn ee_sub_216b80<M: QMem + ?Sized>(m: &mut M, f: u32, c: u32) -> f32 {
    match ee_sub_215a50(m, f, c) {
        0 => 0.0,
        i => {
            let t = m.r32(f + 0x1c);
            m.rf(t.wrapping_add(i * 4) - 4)
        }
    }
}
