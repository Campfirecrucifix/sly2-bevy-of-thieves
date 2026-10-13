#![allow(non_snake_case)]
//! WP-A12: live GI switch (gfx::rt::RtState / gi_switch, Graphics page without the restart note, greyed
//! GI row with a reason).
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::gfx::rt::*;
use sly2_recomp::engine::input::pad::{Clock, PadState};
use sly2_recomp::engine::ui::menu::*;
use sly2_recomp::engine::ui::settings::*;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

fn root() -> PathBuf {
    let r = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    if r.join("Sly 2 - Band of Thieves (USA).iso").exists() { r } else { PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/..")) }
}
fn elf() -> Arc<Vec<u8>> {
    static E: OnceLock<Arc<Vec<u8>>> = OnceLock::new();
    E.get_or_init(|| Arc::new(IsoImage::open(root().join("Sly 2 - Band of Thieves (USA).iso")).expect("ISO").read_file("SCUS_973.16").unwrap()))
        .clone()
}
const CLK: Clock = Clock { t: 10.0, dt: 1.0 / 60.0 };
fn ctx() -> MenuCtx {
    MenuCtx { screens_ready: true, player_char: Some(0x35), graphics: true, ..Default::default() }
}
fn tick(m: &mut MenuMgr, c: &MenuCtx, n: usize) {
    for _ in 0..n {
        m.FUN_001aeee0(&mut PadState::new(), CLK, c);
    }
}
fn text(m: &MenuMgr) -> Vec<String> {
    m.view().unwrap().text.iter().map(|t| t.as_ref().map(|b| String::from_utf8_lossy(b).into_owned()).unwrap_or_default()).collect()
}
fn cross_on(m: &mut MenuMgr, c: &MenuCtx, k: i32) {
    let d = m.depth as usize - 1;
    m.frames[d].sel = k;
    let mut p = PadState::new();
    p.buttons |= 0x40;
    p.pressed |= 0x40;
    m.FUN_001aeee0(&mut p, CLK, c);
    tick(m, c, 3);
}

#[test]
fn a12_gi_switch() {
    let st = |requested, active, available| RtState { requested, active, available, unavailable: None };
    assert_eq!(gi_switch(&st(false, false, true)), GiSwitch::Keep);
    assert_eq!(gi_switch(&st(true, false, true)), GiSwitch::Enable);
    assert_eq!(gi_switch(&st(true, true, true)), GiSwitch::Keep);
    assert_eq!(gi_switch(&st(false, true, true)), GiSwitch::Disable);
    // never enabled without the GPU features
    assert_eq!(gi_switch(&st(true, false, false)), GiSwitch::Keep);
    assert_eq!(gi_unavailable_reason(false, true).as_deref(), Some("build without rt"));
    assert_eq!(gi_unavailable_reason(true, false).as_deref(), Some("no ray tracing GPU"));
    assert_eq!(gi_unavailable_reason(true, true), None);
    assert_eq!(gi_schedule("300:on, 600:off", 300), Some(true));
    assert_eq!(gi_schedule("300:on, 600:off", 600), Some(false));
    assert_eq!(gi_schedule("300:on, 600:off", 301), None);
    assert_eq!(gi_schedule("", 1), None);
}

#[test]
fn a12_live_gi_no_restart_note() {
    let c = ctx();
    let mut m = MenuMgr::new(Some(elf()));
    m.gfx = GraphicsOpts { raytraced: false, rt_available: true, hor_plus: true, ink_outlines: true };
    m.gi_live = true;
    m.port_open_graphics();
    tick(&mut m, &c, 1);
    let n = text(&m).len();
    cross_on(&mut m, &c, 0);
    assert!(m.gfx.raytraced);
    assert!(m.events.contains(&MenuEvent::Graphics));
    let t = text(&m);
    assert_eq!(t.len(), n, "{t:?}");
    assert!(!t.iter().any(|s| s.to_lowercase().contains("restart")), "{t:?}");
    assert_eq!(t[0], "GI: Ray traced");
}

#[test]
fn a12_gi_greyed_with_reason() {
    let c = ctx();
    let mut m = MenuMgr::new(Some(elf()));
    m.gfx = GraphicsOpts { raytraced: false, rt_available: false, hor_plus: true, ink_outlines: true };
    m.gi_live = true;
    m.gi_reason = gi_unavailable_reason(true, false);
    m.port_open_graphics();
    tick(&mut m, &c, 1);
    let t = text(&m);
    assert_eq!(t, ["GI: Off (no ray tracing GPU)", "Aspect: Hor+", "Ink Outlines: On"]);
    // the info row does nothing
    let g = m.gfx;
    cross_on(&mut m, &c, 0);
    assert_eq!(m.gfx, g);
    assert_eq!(text(&m).len(), 3);
}

// --- group 2: menu readability ---
use sly2_recomp::engine::ui::bind::{self, BindCtx};
use sly2_recomp::engine::ui::font::Fonts;
use sly2_recomp::engine::ui::hud::{self, UiFrame, MENU_BACKDROP, MENU_ROW_W, MENU_X};
use sly2_recomp::engine::ui::strings::LevelStrings;

#[test]
fn a12_menu_opaque_backdrop() {
    let e = elf();
    let mut fonts = Fonts::from_elf(&e).unwrap();
    let t = bind::FUN_0021aa60(|va, n| bind::elf_read(&e, va, n)).unwrap();
    let mut m = MenuMgr::new(Some(elf()));
    m.FUN_001b0290(&ctx());
    let c = BindCtx { menu: m.view(), ..Default::default() };
    let mut f = UiFrame::default();
    hud::menu(&mut fonts, &t, &LevelStrings::default(), &c, &mut f);
    let b = &f.boxes[0];
    assert_eq!(b.rgba, [0, 0, 0, 0x80]); // GS alpha 0x80 = opaque (pause screen run #1)
    assert!(!b.blend);
    assert_eq!((b.x0, b.y0, b.x1, b.y1), MENU_BACKDROP);
    // covers a 32:9 window: 640-unit 16:9 safe rect centred, window twice as wide
    assert!(b.x0 <= -320.0 && b.x1 >= 960.0 && b.y0 <= 0.0 && b.y1 >= 492.8);
}

#[test]
fn a12_menu_rows_fit_one_line() {
    let e = elf();
    let mut fonts = Fonts::from_elf(&e).unwrap();
    let mut f = UiFrame::default();
    // A11 "06" overlap: this row wrapped "Off" onto the next row
    let r = hud::text_fit(&mut fonts, b"Invert Turret Controls: Off", MENU_X, 100.0, MENU_ROW_W, 0.8, hud::ROW_COLOR, 1, &mut f);
    let lh = fonts.fonts[fonts.default].line_h as f32;
    assert!(r.h <= lh * 0.8 + 0.01, "wrapped: h {}", r.h);
    let y0 = f.glyphs[0].y0;
    assert!(f.glyphs.iter().all(|g| (g.y0 - y0).abs() < lh * 0.5), "glyphs on more than one line");
    assert!(f.glyphs.iter().all(|g| g.x0 >= MENU_X - 0.5 && g.x1 <= MENU_X + MENU_ROW_W + 0.5));
    // long Controls row shrinks instead of wrapping
    let mut f = UiFrame::default();
    let s = b"rl: Mouse Right / RightTrigger / ShiftRight / Mouse Middle";
    let r = hud::text_fit(&mut fonts, s, MENU_X, 100.0, MENU_ROW_W, 0.8, hud::ROW_COLOR, 1, &mut f);
    assert!(r.h < lh * 0.8 && f.glyphs.iter().all(|g| g.x1 <= MENU_X + MENU_ROW_W + 0.5));
    // short rows keep the requested scale
    let r = hud::text_fit(&mut fonts, b"Display", MENU_X, 100.0, MENU_ROW_W, 1.0, hud::SEL_COLOR, 1, &mut UiFrame::default());
    assert!((r.h - lh).abs() < 0.01);
}

// --- group 3: HUD meter (FUN_001d9968 / FUN_001d9df8 / FUN_001d9b60) ---
use sly2_recomp::engine::player::p18_gadget::GADGETS;
use sly2_recomp::engine::ui::meter::*;

fn sly(hp: i32, mp: i32, acquired: u64) -> MeterJt {
    MeterJt { id: 1, cls: 0x35, hp, hp_max: 3, mp, mp_max: 100, f_10ec: -1, f_10c8: -1, acquired }
}

#[test]
fn a12_meter_spawn_and_power_visibility() {
    let mut m = HudMeter::default();
    m.FUN_001d9968(None, true); // level init before the JT exists
    assert_eq!((m.state, m.health, m.power), (0, 0.0, 0.0));
    m.FUN_001d9df8(Some(sly(3, 100, 0)), 1.0 / 60.0);
    // new JT -> state 3 -> values jump -> state 1 (Sly animations 0x33d..)
    assert_eq!(m.state, 1);
    assert_eq!((m.health, m.power), (1.0, 1.0));
    assert_eq!(m.anims, [0x33d, 0x33e, 0x33f, 0x340, 0x341]);
    assert_eq!(m.anim, Some(0x33d));
    assert!(m.health_shown && !m.power_shown, "no power gadget acquired -> fill 0x350 hidden");
    // a gadget of Sly's list that costs power shows the power fill on the next state-1 entry
    let g = DAT_002C2DE8.iter().copied().find(|&g| GADGETS[g as usize].cost > 0).unwrap();
    let mut m2 = HudMeter::default();
    m2.FUN_001d9968(None, true);
    m2.FUN_001d9df8(Some(sly(3, 100, 1 << g)), 1.0 / 60.0);
    assert!(m2.power_shown);
    // Murray / Bentley lists and animation sets
    let mut mm = HudMeter::default();
    mm.FUN_001d9968(Some(MeterJt { cls: 0x37, ..sly(3, 100, 0) }), true);
    assert_eq!(mm.anims[0], 0x347);
    mm.FUN_001d9968(Some(MeterJt { cls: 0x36, ..sly(3, 100, 0) }), true);
    assert_eq!(mm.anims[0], 0x342);
}

#[test]
fn a12_meter_damage_runs_down() {
    let dt = 1.0 / 60.0;
    let mut m = HudMeter::default();
    m.FUN_001d9968(None, true);
    m.FUN_001d9df8(Some(sly(3, 100, 0)), dt);
    m.FUN_001d9df8(Some(sly(2, 100, 0)), dt);
    assert_eq!((m.state, m.anim), (2, Some(0x33e))); // hit animation +0x40
    m.FUN_001d9df8(Some(sly(2, 100, 0)), dt); // animation done (no timelines) -> 3
    assert_eq!(m.state, 3);
    let mut last = m.health;
    let mut n = 0;
    while m.state == 3 && n < 600 {
        m.FUN_001d9df8(Some(sly(2, 100, 0)), dt);
        assert!(m.health <= last + 1e-6 && last - m.health <= 100.0 * dt + 1e-4);
        last = m.health;
        n += 1;
    }
    assert_eq!(m.state, 1);
    assert!(n > 1, "falling health runs down over several frames");
    assert!((m.health - 2.0 / 3.0).abs() < 1e-3);
    // healing jumps up at once
    m.FUN_001d9df8(Some(sly(3, 100, 0)), dt);
    assert_eq!(m.state, 3);
    m.FUN_001d9df8(Some(sly(3, 100, 0)), dt);
    assert_eq!((m.state, m.health), (1, 1.0));
    // power spend runs down in state 4, death -> state 0 (+0x44)
    m.FUN_001d9df8(Some(sly(3, 40, 0)), dt);
    assert_eq!(m.state, 4);
    for _ in 0..600 {
        m.FUN_001d9df8(Some(sly(3, 40, 0)), dt);
    }
    assert_eq!(m.state, 1);
    assert!((m.power - 0.4).abs() < 1e-3);
    m.FUN_001d9df8(Some(sly(0, 40, 0)), dt);
    assert_eq!((m.state, m.anim), (0, Some(0x33f)));
    assert!(FUN_0011d4d0(0.5, 0.50001, METER_EPS) && !FUN_0011d4d0(0.5, 0.6, METER_EPS));
    assert!(FUN_0011d4d0(200.0, 200.01, METER_EPS) && !FUN_0011d4d0(200.0, 200.1, METER_EPS));
}

#[test]
fn a12_meter_bars() {
    let mut m = HudMeter::default();
    m.FUN_001d9968(Some(sly(3, 50, 0)), true);
    let mut f = UiFrame::default();
    hud::meter(&m, &mut f);
    assert_eq!(f.boxes.len(), 4); // init shows both fills
    assert_eq!(f.boxes[1].x1, 240.0);
    assert_eq!(f.boxes[3].x1, 140.0);
    m.power_shown = false;
    let mut f = UiFrame::default();
    hud::meter(&m, &mut f);
    assert_eq!(f.boxes.len(), 2);
}

// --- group 4: binocucom HUD (0x19d540 / 0x19d660 / 0x19da78) ---
use sly2_recomp::engine::ui::binoc_hud::*;

#[test]
fn a12_binoc_hud_states() {
    let mut h = BinocHud::default();
    h.FUN_0019d540(0x35, 0);
    assert_eq!((h.screen, h.screen_default, h.screen_obj), (0x35f, 0x368, 0x368));
    let mut m = BinocHud::default();
    m.FUN_0019d540(0x37, 0x999);
    assert_eq!((m.screen, m.screen_obj), (0x360, 0x999));
    m.FUN_0019d540(0x36, 0);
    assert_eq!(m.screen, 0x361);
    let mut b = BinocIn { cls: 0x35, mode: 0, yaw: 0.0, zoom: 0.25, f_10ec: -1, f_10e4: -1 };
    h.FUN_0019d660(&b, 0); // JT enters binocucom
    assert_eq!((h.state, h.shown, h.msgs.clone()), (0, true, vec![7]));
    assert!((h.compass - 0.5).abs() < 1e-6 && h.zoom == 0.25);
    h.FUN_0019da78(&b); // screen open -> 1
    assert_eq!(h.state, 1);
    b.yaw = std::f32::consts::FRAC_PI_2;
    b.f_10ec = 4;
    b.f_10e4 = 2;
    h.FUN_0019da78(&b);
    assert!(h.marker && h.marker_kind == 2);
    assert!((h.compass - 0.75).abs() < 1e-6);
    b.f_10ec = -1;
    b.f_10e4 = 4;
    h.FUN_0019da78(&b);
    assert!(h.marker, "kept: DAT_002bf920 was not 4");
    h.FUN_0019da78(&b); // DAT_002bf920 == 4 now -> hidden
    assert!(!h.marker);
    h.FUN_0019d660(&b, 2);
    h.FUN_0019da78(&b); // screen closed -> -1
    assert_eq!((h.state, h.shown), (-1, false));
    assert_eq!(h.msgs, vec![7, 8]);
    // mode 1 (through an object) messages 0xc / 0xd; leaving without the close state still sends 0xd
    let mut o = BinocHud::default();
    o.FUN_0019d540(0x35, 0);
    let b1 = BinocIn { mode: 1, ..b };
    o.FUN_0019d660(&b1, 0);
    o.FUN_0019d660(&b1, -1);
    assert_eq!(o.msgs, vec![0xc, 0xd]);
    // drawing: 4 boxes (+ marker)
    let mut f = UiFrame::default();
    let mut v = BinocHud { shown: true, compass: 0.5, zoom: 0.5, marker: true, ..BinocHud::default() };
    hud::binoc(&v, &mut f);
    assert_eq!(f.boxes.len(), 5);
    v.shown = false;
    let mut f = UiFrame::default();
    hud::binoc(&v, &mut f);
    assert!(f.boxes.is_empty());
}
