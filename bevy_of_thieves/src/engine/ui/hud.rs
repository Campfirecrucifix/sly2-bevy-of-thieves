//! WP-I3: HUD counters, pause/options menu and title text as quads in FUN_0015f480 text space
//! (640 x 492.8, y down); `ui::plugin` maps them through `HudLayout::map_text_rect` into the 16:9 rect.
//!
//! The strings are the original's: counters come from the rich-text bindings (`ui::bind`: 0x2cf coins
//! "%d", 0x2d0 "%d/%d", 0x2d1 timer), menu rows from FUN_00211298 (rows 0..6, the selection on row 3),
//! title prompt = level string 5 ("Press START button for New Game").
//! REPLACED: the original draws all of these through the class-126/130 screen objects of the level
//! (animated sprite timelines, FUN_00212298 data) with the rich text renderer FUN_00215ba8/FUN_00215f40/
//! FUN_002166b0. The port uses the I2 DrawText (FUN_0015ffd0 = `Text::draw`) and fixed positions; the
//! colour 0x7f7f664c / 0x7f00667f values are the screen-object text-item colours (K3 records) of the menu
//! rows / highlighted row as found in the level data.
use super::bind::{self, BindCtx, Binding};
use super::caption::{box_quad, BoxQuad};
use super::font::{Fonts, GlyphQuad};
use super::strings::LevelStrings;
use super::text::{Text, TextRect};

/// WP-UI4: meter fill colours in the `box_quad` 0xAABBGGRR order (little-endian RGBA, same as `ROW_COLOR` /
/// the K3 records). The health fill is the blue/teal bar (R 0x20, G 0x40, B 0xc0) and the gadget-power fill is
/// orange (R 0xc0, G 0x60, B 0x20). FIX: these were written as 0xff2040c0 / 0xffc06020 in 0xAARRGGBB order, so
/// `box_quad().to_le_bytes()` read R from the low byte and swapped them -- the health bar came out orange (the
/// bar at the top of every EP2a / EP1a screenshot) and the power bar blue; the bytes are now in GS order.
pub const HEALTH_FILL: u32 = 0xffc0_4020;
pub const POWER_FILL: u32 = 0xff20_60c0;

/// Player values the HUD shows (P7 `Dmg` on the JT).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HudPlayer {
    pub hp: i32,
    pub hp_max: i32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiFrame {
    pub boxes: Vec<BoxQuad>,
    pub glyphs: Vec<GlyphQuad>,
}

/// Menu row colours from the level's pause screen text items (K3 b = RGBA bytes 0x7f7f664c rows,
/// 0x7f00667f highlight; same 0xAABBGGRR layout as `TextRect::color`), alpha forced opaque.
pub const ROW_COLOR: u32 = 0xff7f_664c;
pub const SEL_COLOR: u32 = 0xff00_667f;

/// Draw `s` in the default font at `scale`, aligned in (x, y, w).
pub fn text(fonts: &mut Fonts, s: &[u8], x: f32, y: f32, w: f32, scale: f32, color: u32, halign: i32, out: &mut UiFrame) -> TextRect {
    let f = fonts.default;
    fonts.fonts[f].push_scale(scale, scale);
    let mut t = Text::new(s, f);
    let h = t.height(fonts, w);
    let r = TextRect { x, y, w, h, color, halign, valign: 0 };
    out.glyphs.extend(t.draw(fonts, &r));
    fonts.fonts[f].pop_scale();
    r
}

/// HUD counters: health bar (hp of hp_max) top-left, clue bottles "%d/%d" under them when a total is
/// set, coins (binding 0x2cf) top-right.
pub fn hud(fonts: &mut Fonts, table: &[Binding], strings: &LevelStrings, ctx: &BindCtx, p: Option<HudPlayer>, out: &mut UiFrame) {
    if let Some(p) = p.filter(|p| p.hp_max > 0) {
        // health meter: hp of hp_max (P7 Dmg, max 0x28) as a bar
        let w = 200.0 * (p.hp.clamp(0, p.hp_max) as f32 / p.hp_max as f32);
        out.boxes.push(box_quad(38.0, 26.0, 242.0, 42.0, 0x8000_0000));
        out.boxes.push(box_quad(40.0, 28.0, 40.0 + w, 40.0, HEALTH_FILL));
    }
    if ctx.count.1 > 0 {
        let s = bind::FUN_00216bc8_text(table, strings, ctx, 1, 0x2d0);
        text(fonts, &s, 40.0, 62.0, 200.0, 0.8, 0xff80_8080, 0, out);
    }
    if ctx.coins.is_some() {
        let s = bind::FUN_00216bc8_text(table, strings, ctx, 1, 0x2cf);
        let r = text(fonts, &s, 400.0, 24.0, 200.0, 1.0, 0xff80_8080, 2, out);
        out.boxes.insert(0, box_quad(r.x + r.w - 110.0, r.y - 4.0, r.x + r.w + 10.0, r.y + r.h + 4.0, 0x8000_0000));
    }
}

/// A14: HUD slots 1..3 (coins 0x2cf, bottles "%d/%d" 0x2d0, countdown seconds) while their class-126
/// label is shown (`ui::hud_slots`), stacked top-left under the meter at the FUN_0020f7e0 offsets.
/// REPLACED: the labels' screen art (text over a box at 26 + offset).
#[allow(clippy::too_many_arguments)]
pub fn slots(fonts: &mut Fonts, table: &[Binding], strings: &LevelStrings, ctx: &BindCtx, s: &super::hud_slots::HudSlots, now: f32, out: &mut UiFrame) {
    for i in 1..4 {
        let Some(off) = s.shown(i, false) else { continue };
        let txt = match i {
            1 if ctx.coins.is_some() => bind::FUN_00216bc8_text(table, strings, ctx, 1, 0x2cf),
            2 if ctx.count.1 > 0 => bind::FUN_00216bc8_text(table, strings, ctx, 1, 0x2d0),
            3 => format!("{}", (s.end - now).max(0.0) as i32).into_bytes(),
            _ => continue,
        };
        let y = 26.0 + off;
        let r = text(fonts, &txt, 48.0, y + 4.0, 200.0, 1.0, 0xff80_8080, 0, out);
        out.boxes.insert(0, box_quad(38.0, y, r.x + 130.0, y + r.h + 8.0, 0x8000_0000));
    }
}

/// WP-UI4: the vault / safe HUD object 0x2e1e60 (FUN_001a3078 show / FUN_001a30d0 hide, screen element
/// 0x373) shown while a class-78 vault is being cracked (world +0x3d80 > 0). REPLACED: the level screen
/// element 0x373 art; the port draws a small indicator box at the top-centre (like the binocucom marker).
pub fn vault(shown: bool, out: &mut UiFrame) {
    if !shown {
        return;
    }
    out.boxes.push(box_quad(296.0, 18.0, 344.0, 34.0, 0x8000_0000));
    out.boxes.push(box_quad(300.0, 22.0, 340.0, 30.0, SEL_COLOR));
}

/// A12: HUD meter (`ui::meter::HudMeter`, FUN_001d9df8) as two bars top-left: health (fill 0x34f) and,
/// when shown, gadget power (fill 0x350) under it. REPLACED: the screen object 0x36b art / timelines.
pub fn meter(m: &super::meter::HudMeter, out: &mut UiFrame) {
    // CRAWL R106 (USER: blue bar top left): the original's idle meter (state 1, FUN_001d9b60 anims[0] = 0x33d
    // for Sly) is only the character icon (ref_video/01 gameplay: hat badge, no bar); the health pips show in
    // the damage / run-down states 2 / 3, the power fill in 4 / 6 / 7. The icon art is not drawn (no sprite).
    let r106 = std::env::var_os("SLY2_NO_R106_METERIDLE").is_none();
    if m.health_shown && (!r106 || matches!(m.state, 2 | 3)) {
        out.boxes.push(box_quad(38.0, 26.0, 242.0, 42.0, 0x8000_0000));
        out.boxes.push(box_quad(40.0, 28.0, 40.0 + 200.0 * m.health.clamp(0.0, 1.0), 40.0, HEALTH_FILL));
    }
    if m.power_shown && (!r106 || matches!(m.state, 4 | 6 | 7)) {
        out.boxes.push(box_quad(38.0, 44.0, 242.0, 56.0, 0x8000_0000));
        out.boxes.push(box_quad(40.0, 46.0, 40.0 + 200.0 * m.power.clamp(0.0, 1.0), 54.0, POWER_FILL));
    }
}

/// A12: binocucom HUD (`ui::binoc_hud::BinocHud`, FUN_0019da78) while its screen is shown: compass strip
/// (element 0x352, needle at the scrub time), zoom bar (0x351) and the target marker (0x353).
/// REPLACED: the binocucom screen objects 0x35f..0x36a (level sprite art).
pub fn binoc(h: &super::binoc_hud::BinocHud, out: &mut UiFrame) {
    if !h.shown {
        return;
    }
    let c = 250.0 + 140.0 * h.compass.rem_euclid(1.0);
    out.boxes.push(box_quad(248.0, 18.0, 392.0, 32.0, 0x8000_0000));
    out.boxes.push(box_quad(c - 2.0, 14.0, c + 2.0, 36.0, 0xff80_8080));
    out.boxes.push(box_quad(602.0, 148.0, 614.0, 352.0, 0x8000_0000));
    let z = 200.0 * h.zoom.clamp(0.0, 1.0);
    out.boxes.push(box_quad(604.0, 350.0 - z, 612.0, 350.0, 0xff80_8080));
    if h.marker {
        out.boxes.push(box_quad(312.0, 238.0, 328.0, 254.0, 0xff00_667f));
    }
}

/// Pause screen geometry from the level's class-126 pause screen object (oid 0x5d6): text items 0x2d2..
/// 0x2d9 are K3 records {line h 22, box h 30.2, width 428.7, halign 1}; draw run #1 is an opaque black
/// quad over the whole screen (VU colour 0x80000000, layer 0).
pub const MENU_ROW_W: f32 = 428.7;
pub const MENU_X: f32 = 320.0 - MENU_ROW_W * 0.5;
/// Run #1 extent in text units (it covers the whole display; at 32:9 the window is wider than 640 so the
/// quad reaches far past both edges).
pub const MENU_BACKDROP: (f32, f32, f32, f32) = (-2000.0, -2000.0, 2640.0, 2492.8);

/// `text` on one line: the scale shrinks until the widest line fits `w` (A12: the original rows never
/// wrap; long port rows like "Invert Turret Controls: Off" wrapped into the next row).
pub fn text_fit(fonts: &mut Fonts, s: &[u8], x: f32, y: f32, w: f32, scale: f32, color: u32, halign: i32, out: &mut UiFrame) -> TextRect {
    let f = fonts.default;
    let wide = Text::new(s, f).widest(fonts);
    let fit = if wide > 0.0 { (w / wide * 0.98).min(scale) } else { scale };
    text(fonts, s, x, y, w, fit, color, halign, out)
}

/// Pause / options menu: the 7 rows of bindings 0x2d2..0x2d8 (FUN_00211298 args 0..6), row 3 = selection,
/// over the pause screen's opaque black backdrop (run #1). NOT PORTED: the vector illustration (run #5,
/// VU1 microcode 0x2a2e10 animated by the screen's group matrices).
pub fn menu(fonts: &mut Fonts, table: &[Binding], strings: &LevelStrings, ctx: &BindCtx, out: &mut UiFrame) {
    let Some(m) = &ctx.menu else { return };
    let n = m.ids.len() as i32;
    let (x0, y0, x1, y1) = MENU_BACKDROP;
    out.boxes.push(box_quad(x0, y0, x1, y1, 0xff00_0000));
    if n == 0 {
        return;
    }
    // rows outside the list are only drawn when the list is long enough to wrap into them
    for row in 0..7 {
        let rel = row - 3;
        if n < 7 && (rel < -(n - 1) / 2 || rel > n / 2) {
            continue;
        }
        let s = bind::FUN_00216bc8_text(table, strings, ctx, 1, 0x2d2 + row);
        let color = if row == 3 { SEL_COLOR } else { ROW_COLOR };
        let scale = if row == 3 { 1.0 } else { 0.8 };
        text_fit(fonts, &s, MENU_X, 230.0 + rel as f32 * 34.0, MENU_ROW_W, scale, color, 1, out);
    }
}

/// Title / episode screen (0x3da530) prompt: level string 5 near the bottom, logo text 0x10 above it.
pub fn title(fonts: &mut Fonts, strings: &LevelStrings, out: &mut UiFrame) {
    text(fonts, strings.FUN_0013e368(0x10), 0.0, 150.0, 640.0, 2.0, 0xff80_8080, 1, out);
    text(fonts, strings.FUN_0013e368(5), 0.0, 400.0, 640.0, 1.0, 0xff80_8080, 1, out);
}

/// A14: title screen with the FUN_001ddbf0 START text (New Game / Continue, `start`; empty = string 5 as
/// before) and the port load menu (`load`, open only) over it (rows centred, the chosen row in SEL_COLOR).
pub fn title_menu(fonts: &mut Fonts, strings: &LevelStrings, start: &[u8], load: Option<&super::title::LoadMenu>, out: &mut UiFrame) {
    text(fonts, strings.FUN_0013e368(0x10), 0.0, 150.0, 640.0, 2.0, 0xff80_8080, 1, out);
    if let Some(m) = load.filter(|m| m.open) {
        for (i, r) in m.rows().iter().enumerate() {
            let color = if i as i32 == m.row { SEL_COLOR } else { ROW_COLOR };
            text(fonts, r.as_bytes(), 0.0, 300.0 + 36.0 * i as f32, 640.0, 1.0, color, 1, out);
        }
        return;
    }
    let s = if start.is_empty() { strings.FUN_0013e368(5) } else { start };
    text(fonts, s, 0.0, 400.0, 640.0, 1.0, 0xff80_8080, 1, out);
}

/// A13: episode select (class 131) values of the selected episode: completion (binding 0x52f,
/// FUN_0015d8f8) and play time (0x530, FUN_0015d9c0) of DAT_003e1c04 +0x110, plus "Episode n". REPLACED:
/// the original shows them inside the class-126 label animation of the episode screen (FUN_00213300,
/// not ported); the port draws them at fixed positions. FUN_001daea8 (+5 bonus) is not ported (false).
pub fn episode(fonts: &mut Fonts, e: &super::episode::Ep131, inp: &super::episode::EpIn, out: &mut UiFrame) {
    let Some(i) = usize::try_from(e.f_110).ok().filter(|&i| i < 9) else { return };
    let ctx = BindCtx {
        completion: inp.pct[i].map(|p| (p as i32, false)),
        play_time: inp.dt[i],
        ..Default::default()
    };
    text(fonts, if i == 0 { "Prologue".to_string() } else { format!("Episode {i}") }.as_bytes(), 0.0, 330.0, 640.0, 1.2, SEL_COLOR, 1, out);
    text(fonts, &bind::FUN_0015d8f8(&ctx), 0.0, 370.0, 640.0, 1.0, ROW_COLOR, 1, out);
    text(fonts, &bind::FUN_0015d9c0(&ctx), 0.0, 400.0, 640.0, 1.0, ROW_COLOR, 1, out);
}

/// WIRE-UI: the ThiefNet screen 0x3da150 (`ui::live_menu::ThiefNetView`) in text space: a backdrop, the
/// page title, the visible rows (DAT_002c35b4 = 6 rows of DAT_002c35b0 = 40.0 from +0x250, the +0x24c row
/// highlighted) as gadget name (GADGETS +0x14 string id) + price (+8), and the +0x30 dialog's +0x38 string.
/// REPLACED: the original draws it through the class-126 screen object at +0x24 (row elements 0x2f5..).
pub fn thiefnet(fonts: &mut Fonts, v: &super::thiefnet::ThiefNetRows, name: &dyn Fn(u32) -> Vec<u8>, out: &mut UiFrame) {
    use super::thiefnet::{ROWS, ROW_H};
    out.boxes.push(box_quad(96.0, 64.0, 544.0, 430.0, 0xc000_0000));
    let title: &[u8] = if v.page == 0 { b"ThiefNet - Gadgets" } else { b"ThiefNet - Loot" };
    text(fonts, title, 96.0, 76.0, 448.0, 1.0, SEL_COLOR, 1, out);
    for r in 0..ROWS {
        let i = v.first + r;
        let Some(&[id, _]) = v.ent.get(i.max(0) as usize).filter(|_| i >= 0) else { break };
        let y = 120.0 + r as f32 * ROW_H;
        let c = if i == v.sel { SEL_COLOR } else { ROW_COLOR };
        let g = crate::engine::player::p18_gadget::GADGETS.get(id.max(0) as usize).filter(|_| v.page == 0);
        let label = g.map(|g| name(g.name)).filter(|s| !s.is_empty()).unwrap_or_else(|| format!("#{id}").into_bytes());
        text(fonts, &label, 116.0, y, 300.0, 0.8, c, 0, out);
        if let Some(g) = g {
            text(fonts, format!("{}", g.price).as_bytes(), 420.0, y, 104.0, 0.8, c, 2, out);
        }
    }
    if v.dialog >= 0 {
        out.boxes.push(box_quad(140.0, 360.0, 500.0, 420.0, 0xe020_2020));
        text(fonts, &name(v.dialog_msg as u32), 150.0, 372.0, 340.0, 0.8, SEL_COLOR, 1, out);
    }
}
