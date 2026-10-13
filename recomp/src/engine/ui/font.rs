//! WP-I2: the disc's bitmap fonts (FUN_0015ef00 .. FUN_0015f8a8), loaded straight from SCUS_973.16.
//!
//! Font table 0x2be320 = 2 descriptor pointers {header, glyphs, bitmap, clut}. The bitmap/CLUT objects are
//! built by C++ static initialisers (FUN_001147d8 font 0, FUN_00114ae0 font 1) that store the CLUT bytes
//! with immediate `sb`s; `static_init_stores` replays those stores (lui/addiu/ori/sb/sh/sw only).
//! Texels are linear (PSMT4 low nibble first / PSMT8), the image is stored bottom-up (the glyph v of
//! FUN_0015f480 counts from the bottom), so `FontImage` is flipped once at load and glyph rects are then
//! plain top-down (u, v, w, line_h).
use crate::engine::disc::descriptor::elf_bytes;
use std::collections::HashMap;

/// 0x2be320: font descriptor pointer table (2 fonts).
pub const FONT_TABLE: u32 = 0x2be320;
/// 0x2be328: `&X` font selector characters, one per table entry.
pub const FONT_SELECT: u32 = 0x2be328;
/// 0x2be338: u16[256] byte -> glyph code map used by FindGlyph.
pub const CHAR_MAP: u32 = 0x2be338;
/// 0x2aa6a1: newlib `_ctype_ + 1` (bit 0x10 = punctuation; the wrapper never breaks before it).
pub const CTYPE: u32 = 0x2aa6a1;
/// Static initialisers holding the CLUT stores of font 0 / font 1 (scanned up to `jr ra`).
pub const FONT_STATIC_INIT: [u32; 2] = [0x1147d8, 0x114ae0];
/// FUN_0015f480 drawable area (x <= 640, y <= 492.8: the 512x224 GS field in 0.8 / 0.4545 units).
pub const TEXT_W: f32 = 640.0;
pub const TEXT_H: f32 = 492.80002;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Glyph {
    pub ch: u16,
    pub u: u16,
    /// counted from the bottom of the stored bitmap == from the top of the flipped `FontImage`
    pub v: u16,
    pub w: u16,
}

/// RGBA8 texture of one font (already flipped top-down, alpha expanded 0x80 -> 0xff).
#[derive(Clone, Debug)]
pub struct FontImage {
    pub w: u32,
    pub h: u32,
    pub rgba: Vec<u8>,
}

/// Port of the 0x140-byte font object (FUN_0015f710).
#[derive(Clone, Debug)]
pub struct Font {
    /// +0x0 registry id (header byte 7)
    pub id: u32,
    /// +0x4 header byte 4
    pub b4: i32,
    /// +0x8 advance of a missing glyph (header byte 5)
    pub adv: i32,
    /// +0xc line height in texels (header byte 6)
    pub line_h: i32,
    /// +0x10 / +0x14 scale (header float +8)
    pub xs: f32,
    pub ys: f32,
    /// +0x18 count, +0x1c scale stack (room for 6 entries before +0x50)
    pub n: usize,
    pub stack: [(f32, f32); 6],
    /// +0x138 / +0x13c
    pub glyphs: Vec<Glyph>,
    /// index of the texture in `Fonts::images` (stands for tex0 +0x50)
    pub tex: usize,
}

impl Font {
    /// FUN_0015f078 push scale. Quirks kept: both saved words are the x scale, and the new scale is
    /// relative to stack entry 0 (the outermost saved scale), not to the current one.
    pub fn push_scale(&mut self, sx: f32, sy: f32) {
        let n = self.n.min(5);
        self.stack[n] = (self.xs, self.xs);
        self.n += 1;
        self.xs = self.stack[0].0 * sx;
        self.ys = self.stack[0].1 * sy;
    }

    /// FUN_0015f0c0 pop scale.
    pub fn pop_scale(&mut self) {
        self.n = self.n.saturating_sub(1);
        let (x, y) = self.stack[self.n.min(5)];
        self.xs = x;
        self.ys = y;
    }

    /// FUN_0015f8a8 FindGlyph: binary search of the char-mapped code (midpoint quirk kept).
    pub fn find_glyph(&self, char_map: &[u16; 256], c: u8) -> Option<&Glyph> {
        let key = char_map[c as usize] as i32;
        let mut lo = 0i32;
        let mut hi = self.glyphs.len() as i32 - 1;
        if hi < 0 {
            return None;
        }
        let mut mid = hi;
        loop {
            mid /= 2;
            let g = self.glyphs.get(mid as usize)?;
            if key == g.ch as i32 {
                return Some(g);
            }
            if key - (g.ch as i32) < 1 {
                hi = mid - 1;
            } else {
                lo = mid + 1;
            }
            mid = hi + lo;
            if lo > hi {
                return None;
            }
        }
    }

    /// FUN_0015f0f0 DxGlyph: advance of `c` = (w + 1) * xs, or the default advance * xs.
    pub fn dx_glyph(&self, char_map: &[u16; 256], c: u8) -> f32 {
        match self.find_glyph(char_map, c) {
            None => self.adv as f32 * self.xs,
            Some(g) => self.xs * (g.w as f32 + 1.0),
        }
    }
}

/// One textured glyph quad in FUN_0015f480 text space (640 x 492.8, y down).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlyphQuad {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
    /// texel rect in the flipped `FontImage` (u0,v0 top-left)
    pub u0: f32,
    pub v0: f32,
    pub u1: f32,
    pub v1: f32,
    pub tex: usize,
    /// GS RGBAQ as written by FUN_0015f480 (alpha already halved: 0xff -> 0x80)
    pub rgba: [u8; 4],
}

/// FUN_0015f480 DrawGlyph: emits the sprite when the glyph overlaps the 640x492.8 area, returns the advance.
pub fn draw_glyph(x: f32, y: f32, font: &Font, char_map: &[u16; 256], c: u8, color: u32, out: &mut Vec<GlyphQuad>) -> f32 {
    let Some(g) = font.find_glyph(char_map, c) else {
        return font.adv as f32 * font.xs;
    };
    let x1 = x + g.w as f32 * font.xs;
    let y1 = y + font.line_h as f32 * font.ys;
    if x1 >= 0.0 && x <= TEXT_W && y1 >= 0.0 && y <= TEXT_H {
        let a = (((color >> 24) + 1) & 0x1fe) << 23;
        let c = color & 0xffffff | a;
        out.push(GlyphQuad {
            x0: x,
            y0: y,
            x1,
            y1,
            u0: g.u as f32,
            v0: g.v as f32,
            u1: (g.u + g.w) as f32,
            v1: g.v as f32 + font.line_h as f32,
            tex: font.tex,
            rgba: c.to_le_bytes(),
        });
    }
    (g.w as f32 + 1.0) * font.xs
}

/// The font registry (DAT_002de9f0[2], default DAT_002de760) plus the tables FindGlyph / wrapping read.
#[derive(Clone, Debug)]
pub struct Fonts {
    pub fonts: Vec<Font>,
    pub images: Vec<FontImage>,
    /// 0x2be328
    pub select: Vec<u8>,
    pub char_map: [u16; 256],
    /// 0x2aa6a1 [c]
    pub ctype: [u8; 256],
    /// DAT_002de760 (FUN_0015ef00: registry[0])
    pub default: usize,
}

impl Fonts {
    /// FUN_0015ef00 / FUN_0015ef30 / FUN_0015f710: build both fonts from the ELF image.
    pub fn from_elf(elf: &[u8]) -> Option<Fonts> {
        let rd = |va: u32, n: usize| elf_bytes(elf, va, n);
        let u32at = |va: u32| rd(va, 4).map(|b| u32::from_le_bytes(b.try_into().unwrap()));
        let mut char_map = [0u16; 256];
        for (i, c) in rd(CHAR_MAP, 512)?.chunks(2).enumerate() {
            char_map[i] = u16::from_le_bytes([c[0], c[1]]);
        }
        let mut ctype = [0u8; 256];
        ctype.copy_from_slice(rd(CTYPE, 256)?);
        let select = rd(FONT_SELECT, 2)?.to_vec();
        let mut fonts = Vec::new();
        let mut images = Vec::new();
        for k in 0..2u32 {
            let desc = u32at(FONT_TABLE + 4 * k)?;
            let hdr = rd(u32at(desc)?, 16)?;
            let nglyph = u16::from_le_bytes([hdr[12], hdr[13]]) as usize;
            let gp = u32at(desc + 4)?;
            let glyphs = rd(gp, nglyph * 8)?
                .chunks(8)
                .map(|g| {
                    let h = |i: usize| u16::from_le_bytes([g[i], g[i + 1]]);
                    Glyph { ch: h(0), u: h(2), v: h(4), w: h(6) }
                })
                .collect();
            let scale = f32::from_le_bytes(hdr[8..12].try_into().unwrap());
            // bitmap / CLUT objects are filled by the static initialiser
            let mem = static_init_stores(elf, FONT_STATIC_INIT[k as usize]);
            let img = decode_bitmap(elf, &mem, k)?;
            fonts.push(Font {
                id: hdr[7] as u32,
                b4: hdr[4] as i32,
                adv: hdr[5] as i32,
                line_h: hdr[6] as i32,
                xs: scale,
                ys: scale,
                n: 0,
                stack: [(0.0, 0.0); 6],
                glyphs,
                tex: images.len(),
            });
            images.push(img);
        }
        // FUN_0015f888: registry[id] = font; FUN_0015ef00: default = registry[0]
        let default = fonts.iter().position(|f| f.id == 0).unwrap_or(0);
        Some(Fonts { fonts, images, select, char_map, ctype, default })
    }

    /// FUN_0015efa0: font for an `&X` selector character.
    pub fn by_selector(&self, c: u8) -> Option<usize> {
        let i = self.select.iter().position(|&s| s == c)?;
        // registry index i -> font with id i
        self.fonts.iter().position(|f| f.id == i as u32)
    }
}

/// Bitmap header source addresses (FUN_001147d8 / FUN_00114ae0 pass them to FUN_001eb050) and CLUT bases.
const BITMAP_HDR: [u32; 2] = [0x29a790, 0x2a28d0];
const CLUT_BASE: [u32; 2] = [0x29a7c0, 0x2a2900];

/// Replays the immediate stores of a static initialiser (lui/addiu/ori + sb/sh/sw), linear up to `jr ra`.
pub fn static_init_stores(elf: &[u8], func: u32) -> HashMap<u32, u8> {
    let mut r = [0u32; 32];
    let mut mem = HashMap::new();
    let mut pc = func;
    let mut end = None;
    while let Some(b) = elf_bytes(elf, pc, 4) {
        let w = u32::from_le_bytes(b.try_into().unwrap());
        let (op, rs, rt) = (w >> 26, ((w >> 21) & 31) as usize, ((w >> 16) & 31) as usize);
        let im = w & 0xffff;
        let se = im as u16 as i16 as i32 as u32;
        let ea = r[rs].wrapping_add(se);
        match op {
            0x0f => r[rt] = im << 16,
            0x09 => r[rt] = r[rs].wrapping_add(se),
            0x0d => r[rt] = r[rs] | im,
            0x28 => {
                mem.insert(ea, r[rt] as u8);
            }
            0x29 => {
                for i in 0..2 {
                    mem.insert(ea + i, (r[rt] >> (8 * i)) as u8);
                }
            }
            0x2b => {
                for i in 0..4 {
                    mem.insert(ea + i, (r[rt] >> (8 * i)) as u8);
                }
            }
            _ => {}
        }
        r[0] = 0;
        if end == Some(pc) {
            break;
        }
        if w == 0x03e0_0008 {
            end = Some(pc + 4); // delay slot
        }
        pc += 4;
        if pc - func > 0x4000 {
            break;
        }
    }
    mem
}

/// PSMT4 (CSM1 16 colours) / PSMT8 (CSM1 256 colours, entries 8..15 <-> 16..23 swizzled) -> RGBA8,
/// flipped top-down, alpha 0x80 -> 0xff (min(255, a*2)).
fn decode_bitmap(elf: &[u8], mem: &HashMap<u32, u8>, k: u32) -> Option<FontImage> {
    let h = elf_bytes(elf, BITMAP_HDR[k as usize], 0x18)?;
    let w = u16::from_le_bytes([h[8], h[9]]) as u32;
    let hh = u16::from_le_bytes([h[10], h[11]]) as u32;
    let psm = h[12];
    let size = u32::from_le_bytes(h[16..20].try_into().unwrap()) as usize;
    let ptr = u32::from_le_bytes(h[20..24].try_into().unwrap());
    let px = elf_bytes(elf, ptr, size)?;
    let ncol = if psm == 0x14 { 16 } else { 256 };
    let base = CLUT_BASE[k as usize];
    let clut: Vec<[u8; 4]> = (0..ncol)
        .map(|i| {
            let j = if ncol == 256 { (i & !0x18) | ((i & 8) << 1) | ((i & 0x10) >> 1) } else { i };
            let b = |o: u32| mem.get(&(base + 4 * j as u32 + o)).copied().or_else(|| elf_bytes(elf, base + 4 * j as u32 + o, 1).map(|b| b[0])).unwrap_or(0);
            [b(0), b(1), b(2), (b(3) as u32 * 2).min(255) as u8]
        })
        .collect();
    let mut rgba = vec![0u8; (w * hh * 4) as usize];
    for y in 0..hh {
        for x in 0..w {
            let i = (y * w + x) as usize;
            let idx = if psm == 0x14 { (px.get(i / 2).copied().unwrap_or(0) >> ((i & 1) * 4)) & 15 } else { px.get(i).copied().unwrap_or(0) };
            let o = (((hh - 1 - y) * w + x) * 4) as usize;
            rgba[o..o + 4].copy_from_slice(&clut[idx as usize]);
        }
    }
    Some(FontImage { w, h: hh, rgba })
}
