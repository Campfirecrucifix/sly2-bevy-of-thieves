//! WP-I2: text object and layout (FUN_0015f928 .. FUN_0015ffd0).
//!
//! Text struct (0x164 B): +0 string, +4 cursor, +8 current font, +0xc base font, +0x10 local font copy
//! (`&X` font switch), +0x150 font-changed flag, +0x154 current colour, +0x158 `~.` colour, +0x15c base
//! colour, +0x160 colour used after an `&X` switch. Escapes (FUN_0015f9d0):
//!   `^^` `&&` `~~` literal; `^.` pop scale; `^dd` push scale d.d; `&.` back to the base font;
//!   `&X` switch to the font whose selector is X (scaled to the base line height); `~.` saved colour;
//!   `~RRGGBB` colour (halved to GS 0..0x80). Malformed `~`/unknown `&X` fall back to the base font/colour
//!   and print the escape character.
use super::font::{draw_glyph, Font, Fonts, GlyphQuad};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Cur {
    Global(usize),
    Local,
}

#[derive(Clone, Debug)]
pub struct Text {
    /// NUL-terminated bytes (the wrapper writes '\n' into it like the original)
    pub buf: Vec<u8>,
    pub cur: usize,
    font: Cur,
    pub base: usize,
    pub local: Option<Font>,
    pub changed: bool,
    pub color: u32,
    pub saved: u32,
    pub base_color: u32,
    pub alt_color: u32,
}

/// Rect argument of FUN_0015ffd0 (float[7]; halign/valign are ints: 0 left/top, 2 right/bottom, else centre).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub color: u32,
    pub halign: i32,
    pub valign: i32,
}

impl Text {
    /// FUN_0015f928 (colours 0xff808080) + FUN_0015f970.
    pub fn new(s: &[u8], font: usize) -> Text {
        let mut buf: Vec<u8> = s.iter().copied().take_while(|&c| c != 0).collect();
        buf.push(0);
        let mut t = Text {
            buf,
            cur: 0,
            font: Cur::Global(font),
            base: font,
            local: None,
            changed: false,
            color: 0xff80_8080,
            saved: 0xff80_8080,
            base_color: 0xff80_8080,
            alt_color: 0xff80_8080,
        };
        t.reset();
        t
    }

    /// FUN_0015f970.
    pub fn reset(&mut self) {
        self.cur = 0;
        self.font = Cur::Global(self.base);
        self.saved = self.base_color;
        self.color = self.saved;
        self.changed = false;
    }

    /// FUN_0015f9a8.
    pub fn set_color(&mut self, c: u32) {
        self.base_color = c;
        self.color = c;
    }

    pub fn font<'a>(&'a self, fonts: &'a Fonts) -> &'a Font {
        match self.font {
            Cur::Global(i) => &fonts.fonts[i],
            Cur::Local => self.local.as_ref().unwrap(),
        }
    }

    fn font_mut<'a>(&'a mut self, fonts: &'a mut Fonts) -> &'a mut Font {
        match self.font {
            Cur::Global(i) => &mut fonts.fonts[i],
            Cur::Local => self.local.as_mut().unwrap(),
        }
    }

    fn at(&self, i: usize) -> u8 {
        self.buf.get(i).copied().unwrap_or(0)
    }

    /// FUN_0015f9d0: next printable byte (0 = end), consuming escapes.
    pub fn next_char(&mut self, fonts: &mut Fonts) -> u8 {
        loop {
            let p = self.cur;
            let c = self.at(p);
            let mut emit = false;
            match c {
                b'^' => match self.at(p + 1) {
                    b'^' => {
                        emit = true;
                        self.cur = p + 1;
                    }
                    b'.' => {
                        self.font_mut(fonts).pop_scale();
                        self.cur = p + 2;
                    }
                    d => {
                        let f = (d as i32 - 0x30) as f32 + (self.at(p + 2) as i32 - 0x30) as f32 * 0.1;
                        let fm = self.font_mut(fonts);
                        let (sx, sy) = (f * fm.xs, f * fm.ys);
                        fm.push_scale(sx, sy);
                        self.cur = p + 3;
                    }
                },
                b'&' => match self.at(p + 1) {
                    b'&' => {
                        emit = true;
                        self.cur = p + 1;
                    }
                    b'.' => {
                        self.font = Cur::Global(self.base);
                        self.color = self.saved;
                        self.changed = true;
                        self.cur = p + 2;
                    }
                    x => match fonts.by_selector(x) {
                        None => {
                            emit = true;
                            self.font = Cur::Global(self.base);
                            self.color = self.base_color;
                            self.changed = true;
                        }
                        Some(k) => {
                            let mut l = fonts.fonts[k].clone();
                            self.cur = p + 2;
                            let b = &fonts.fonts[self.base];
                            let r = (b.line_h as f32 * b.ys) / (l.line_h as f32 * l.ys);
                            l.ys *= r;
                            l.xs *= r;
                            self.local = Some(l);
                            self.font = Cur::Local;
                            self.color = (self.color & 0xff00_0000) | (self.alt_color & 0x00ff_ffff);
                            self.changed = true;
                        }
                    },
                },
                b'~' => {
                    let n = self.at(p + 1);
                    if n == b'~' {
                        emit = true;
                        self.cur = p + 1;
                    } else if n == b'.' {
                        self.saved = self.base_color;
                        self.color = self.saved;
                        self.cur = p + 2;
                    } else {
                        // up to 6 hex digits; exactly 6 required
                        let mut q = p + 1;
                        let mut v = 0u32;
                        while q - p < 7 {
                            let h = match self.at(q) {
                                d @ b'0'..=b'9' => d - b'0',
                                d @ b'A'..=b'F' => d - 0x37,
                                d @ b'a'..=b'f' => d - 0x57,
                                _ => break,
                            };
                            v = v * 16 + h as u32;
                            q += 1;
                        }
                        if q - p < 7 {
                            self.font = Cur::Global(self.base);
                            self.color = self.base_color;
                            emit = true;
                        } else {
                            let half = |b: u32| ((b & 0xff) + 1) >> 1;
                            self.saved = (self.saved & 0xff00_0000) | half(v >> 16) | half(v >> 8) << 8 | half(v) << 16;
                            self.color = self.saved;
                            self.cur = p + 7;
                        }
                    }
                }
                _ => emit = true,
            }
            if emit {
                let r = self.at(self.cur);
                self.cur += 1;
                return r;
            }
        }
    }

    /// FUN_0015fd68: break lines at `width` (0 = no limit) by writing '\n' into the string; returns the line
    /// count. Breaks at the last space/tab not followed by punctuation; with none, the character at the
    /// cursor is overwritten by '\n' (original quirk: it is lost).
    pub fn wrap(&mut self, fonts: &mut Fonts, width: f32) -> i32 {
        let mut w = 0.0f32;
        let mut lines = 1;
        self.reset();
        loop {
            loop {
                let c = self.next_char(fonts);
                if c == 0 {
                    return lines;
                }
                if c == b'\n' {
                    w = 0.0;
                    lines += 1;
                } else {
                    w += self.font(fonts).dx_glyph(&fonts.char_map, c);
                }
                if !(width == 0.0 || w <= width) {
                    break;
                }
            }
            let mut p = self.cur as isize - 1;
            if p >= 0 && self.buf[p as usize] != b'\n' {
                loop {
                    let c = self.buf[p as usize];
                    if (c == b' ' || c == b'\t') && fonts.ctype[self.at(p as usize + 1) as usize] & 0x10 == 0 {
                        lines += 1;
                        self.cur = p as usize + 1;
                        w = 0.0;
                        self.buf[p as usize] = b'\n';
                        break;
                    }
                    p -= 1;
                    if p < 0 || self.buf[p as usize] == b'\n' {
                        break;
                    }
                }
            }
            if w != 0.0 && self.at(self.cur) != 0 {
                self.buf[self.cur] = b'\n';
                lines += 1;
                w = 0.0;
            }
        }
    }

    /// FUN_0015fef8: wrapped height = lines * base line height * base y scale.
    pub fn height(&mut self, fonts: &mut Fonts, width: f32) -> f32 {
        let b = &fonts.fonts[self.base];
        let lh = b.line_h as f32 * b.ys;
        lh * self.wrap(fonts, width) as f32
    }

    /// FUN_0015ff40: widest line (int-truncated like the original).
    pub fn widest(&mut self, fonts: &mut Fonts) -> f32 {
        let (mut best, mut w) = (0.0f32, 0.0f32);
        self.reset();
        loop {
            let c = self.next_char(fonts);
            if c == 0 {
                break;
            }
            if c == b'\n' {
                best = if best < w { w as i32 } else { best as i32 } as f32;
                w = 0.0;
            } else {
                w += self.font(fonts).dx_glyph(&fonts.char_map, c);
            }
        }
        (if best < w { w as i32 } else { best as i32 }) as f32
    }

    /// FUN_0015ffd0 DrawText: per-line alignment inside `r`, glyphs through FUN_0015f480.
    pub fn draw(&mut self, fonts: &mut Fonts, r: &TextRect) -> Vec<GlyphQuad> {
        let mut widths = [0.0f32; 64];
        let mut line = 0usize;
        self.reset();
        loop {
            let c = self.next_char(fonts);
            if c == 0 {
                break;
            }
            if c == b'\n' {
                line += 1;
            } else {
                let d = self.font(fonts).dx_glyph(&fonts.char_map, c);
                widths[line.min(63)] += d;
            }
        }
        let b = &fonts.fonts[self.base];
        let lh = b.line_h as f32 * b.ys;
        let mut y = match r.valign {
            0 => r.y,
            2 => r.y + r.h - (line + 1) as f32 * lh,
            _ => r.y + (r.h - (line + 1) as f32 * lh) * 0.5,
        };
        self.set_color(r.color);
        self.reset();
        let mut out = Vec::new();
        let (mut li, mut new_line, mut x) = (0usize, true, 0.0f32);
        loop {
            let c = self.next_char(fonts);
            if c == 0 {
                return out;
            }
            if c == b'\n' {
                new_line = true;
                li += 1;
                let b = &fonts.fonts[self.base];
                y += b.line_h as f32 * b.ys;
                continue;
            }
            if new_line {
                let lw = widths[li.min(63)];
                x = match r.halign {
                    0 => r.x,
                    2 => r.x + r.w - lw,
                    _ => r.x + (r.w - lw) * 0.5,
                };
                new_line = false;
            }
            // FUN_0015f220 / FUN_0015f3e8 only (re)load GS texture state
            self.changed = false;
            let color = self.color;
            x += draw_glyph(x, y, self.font(fonts), &fonts.char_map, c, color, &mut out);
        }
    }
}
