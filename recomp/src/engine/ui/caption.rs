//! WP-I2: text panel of FUN_001b3700 (drawn when its owner's +0x104 string id is set): the string
//! (FUN_00210ca8(0x3e1c20, id), the runtime string table, not ported -> passed in) in the default font at
//! 0.8 scale, wrapped to 320 units, centred on x = 320 with its bottom at y = 476.8 (text space
//! 640 x 492.8), colour 0xffc0c0c0, over a 50% black box (FUN_001cd5f0, colour 0x80000000).
//! The render-to-texture part of FUN_001b3700 (GS 0x41bc00/0x41dc00 buffers) is replaced by the Bevy
//! overlay (ui::plugin).
use super::font::{Fonts, GlyphQuad};
use super::text::{Text, TextRect};

/// FUN_001cd5f0 untextured sprite (x0,y0)-(x1,y1) in text space; `rgba` as written to the GS (alpha halved,
/// blended unless the source alpha was 0xff).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoxQuad {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
    pub rgba: [u8; 4],
    pub blend: bool,
}

/// FUN_001cd5f0 colour/alpha handling.
pub fn box_quad(x0: f32, y0: f32, x1: f32, y1: f32, color: u32) -> BoxQuad {
    let a = (color >> 24) as u8;
    let mut rgba = color.to_le_bytes();
    rgba[3] = ((a as i32 + 1) >> 1) as u8;
    BoxQuad { x0, y0, x1, y1, rgba, blend: a != 0xff }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Caption {
    pub rect: TextRect,
    pub back: BoxQuad,
    pub glyphs: Vec<GlyphQuad>,
}

/// FUN_001b3700 text part.
pub fn caption(fonts: &mut Fonts, s: &[u8]) -> Caption {
    let f = fonts.default;
    fonts.fonts[f].push_scale(0.8, 0.8);
    let mut t = Text::new(s, f);
    let h = t.height(fonts, 320.0);
    let w = t.widest(fonts);
    let rect = TextRect { x: 320.0 - w * 0.5, y: 476.80002 - h, w, h, color: 0xffc0_c0c0, halign: 0, valign: 0 };
    let back = box_quad(rect.x, rect.y, rect.x + rect.w, rect.y + rect.h, 0x8000_0000);
    let glyphs = t.draw(fonts, &rect);
    fonts.fonts[f].pop_scale();
    Caption { rect, back, glyphs }
}
