//! WP-I2 tests: follow camera C2db858 (0x00221a18/0x00221dd8), policy push/revoke between follow and
//! blend, and the PS2 sin/cos polynomial 0x0011d608. Expected values are derived from the decompiled
//! code (derivations in comments), not from running the game.
use sly2_recomp::engine::camera::cm::*;
use sly2_recomp::engine::camera::follow::*;
use sly2_recomp::engine::camera::math::*;
use sly2_recomp::engine::camera::policy::*;
use sly2_recomp::engine::camera::CmClock;

const DT: f32 = 1.0 / 60.0;

fn approx(a: f32, b: f32, eps: f32) -> bool {
    (a - b).abs() <= eps * (1.0 + a.abs().max(b.abs()))
}

/// Scripted focus: CPDEFI pos_base moves along a path, v_base is its velocity.
struct PathWorld {
    pos: V3,
    vel: V3,
}
impl CmWorld for PathWorld {
    fn get_cpdefi(&mut self, _pso: u32) -> Cpdefi {
        Cpdefi { pos_base: v4(self.pos, 1.0), v_base: v4(self.vel, 0.0), ..Default::default() }
    }
    fn jt_state(&mut self, _pso: u32) -> Option<i32> {
        None
    }
}

fn setup(world: &mut PathWorld, t: f32) -> (Cm, usize) {
    let mut cm = Cm::new();
    let id = add_policy(&mut cm, Box::new(CplcyFollow::new(Box::new(NoFollowWorld))));
    let c = CmClock { t, dt: DT, dt_real: DT };
    // Code truth (I3): Cm::new runs InitCm 0x0014d6d8, whose FUN_0014d8c0 leaves cut = 1; it is consumed
    // (cut path, then cut = 0) by the next UpdateCm 0x0014d478 whatever policy is active. Run that frame on
    // the base policy first (mid-game state), so the follow's first update takes the spring path.
    assert_eq!(cm.cut, 1);
    FUN_0014d478(&mut cm, &mut NoWorld, &CplcyCtx::default(), CmClock { t: t - DT, dt: DT, dt_real: DT });
    assert_eq!(cm.cut, 0);
    // focus cached first (as UpdateCm does before the policy runs)
    cm.cpdefi = world.get_cpdefi(1);
    cm.pso_cpdefi = 1;
    assert!(FUN_0014e328(&mut cm, 1, id, 1, 0, c));
    (cm, id)
}

#[test]
fn sincos_polynomial_0x11d608() {
    // x = 0: f = 0.5 -> n = 0; s = 0; c = sqrt(1) = 1.
    assert_eq!(FUN_0011d608(0.0), (0.0, 1.0));
    // x = 1: n = 0, s = 1 - 0.16666657 + 0.0083330255 - 0.00019807414 + 2.601887e-06 = 0.84147096
    let (s, c) = FUN_0011d608(1.0);
    assert!(approx(s, 0.84147096, 1e-6));
    assert!(approx(c, (1.0f32 - s * s).sqrt(), 0.0));
    // x = 4: f = 1.773 -> n = 1 (odd): x' = -(4 - pi), cos negated -> (sin 4, cos 4)
    let (s, c) = FUN_0011d608(4.0);
    assert!(approx(s, 4.0f32.sin(), 1e-5) && approx(c, 4.0f32.cos(), 1e-5));
    // x = -2: f = -0.137 -> trunc 0, f < 0 -> n = -1 (odd)
    let (s, c) = FUN_0011d608(-2.0);
    assert!(approx(s, (-2.0f32).sin(), 1e-5) && approx(c, (-2.0f32).cos(), 1e-5));
    // the sine is clamped to [-1, 1] so the cosine is never NaN
    for i in -100..100 {
        let (s, c) = FUN_0011d608(i as f32 * 0.137);
        assert!(s.abs() <= 1.0 && !c.is_nan());
    }
}

#[test]
fn default_pitch_0x220d40() {
    let mut p = FOLLOW_PARAMS_DEFAULT;
    FUN_00220d40(&mut p);
    // atan2(+0x30 = 125, |(-450 + 0, 0)|) = atan2(125, 450), inside [-0.5, 1.555]
    let e = 125.0f32.atan2(450.0);
    assert_eq!(p.pitch_range, [e, e]);
    // already set: untouched
    let mut q = p;
    q.pitch_range = [0.1, 0.2];
    FUN_00220d40(&mut q);
    assert_eq!(q.pitch_range, [0.1, 0.2]);
}

#[test]
fn follow_first_step_keeps_view_and_springs_distance() {
    let mut w = PathWorld { pos: [0.0; 3], vel: [0.0; 3] };
    let (mut cm, _) = setup(&mut w, 1.0);
    // activate: focus = origin (pos_adjust = pos_base, adjust1 = 0); dist = |(0,-2000,500)| - near(100)
    let d0 = (2000.0f32 * 2000.0 + 500.0 * 500.0).sqrt() - 100.0;
    let c = CmClock { t: 1.0 + DT, dt: DT, dt_real: DT };
    FUN_0014d478(&mut cm, &mut w, &CplcyCtx::default(), c);
    // spring 0x00223324: v = (|-450| - d0) * (1*15*dt) (v0 = 0); d1 = d0 + v*dt
    let v = (450.0 - d0) * (15.0 * DT);
    let d1 = d0 + v * DT;
    let p = cm.pos();
    assert!(approx(len(p), d1 + 100.0, 1e-4), "{} vs {}", len(p), d1 + 100.0);
    // yaw/pitch unchanged (no velocity, no stick): still on the original line of sight
    let dir = scale(p, 1.0 / len(p));
    assert!(approx(dir[1], -2000.0 / (d0 + 100.0), 1e-4) && approx(dir[2], 500.0 / (d0 + 100.0), 1e-4));
    // first frame: angle offsets chosen so the view direction is unchanged (forward -> focus)
    let f = v3(cm.mat[0]);
    assert!(approx(dot(f, scale(dir, -1.0)), 1.0, 1e-4), "{f:?} {dir:?}");
}

#[test]
fn follow_auto_yaw_on_scripted_path() {
    let mut w = PathWorld { pos: [0.0; 3], vel: [0.0; 3] };
    let (mut cm, _) = setup(&mut w, 1.0);
    let mut t = 1.0;
    t += DT;
    FUN_0014d478(&mut cm, &mut w, &CplcyCtx::default(), CmClock { t, dt: DT, dt_real: DT });
    // the focus starts running along +x (sideways to the camera, which looks along +y)
    w.vel = [600.0, 0.0, 0.0];
    let mut g = 0.0f32;
    // gain at the first update: FUN_0011d018(0, 1, dt, (4,0,0.25)); second update starts from there
    g = FUN_0011d018(g, 1.0, DT, &[4.0, 0.0, 0.25, 0.0]).0;
    let mut prev_yaw = FUN_0011efb0(sub(cm.pos(), [0.0; 3])).0;
    for _ in 0..30 {
        w.pos = add(w.pos, scale(w.vel, DT));
        t += DT;
        let focus_cam = sub(add(w.pos, [0.0; 3]), cm.pos());
        g = FUN_0011d018(g, 1.0, DT, &[4.0, 0.0, 0.25, 0.0]).0;
        FUN_0014d478(&mut cm, &mut w, &CplcyCtx::default(), CmClock { t, dt: DT, dt_real: DT });
        let focus = add(v3(cm.cpdefi.pos_adjust), cm.pos_adjust1);
        let yaw = FUN_0011efb0(sub(cm.pos(), focus)).0;
        // 0x002225d8: n = up x normalize(focus - cam); w = clamp(v.n / |focus - cam|, 2.5);
        // yaw += w * gain * boost(1) * ramp(1) * dt. Sign check + magnitude bound by 2.5*dt.
        let _ = focus_cam;
        let dyaw = FUN_0011cf60(yaw - prev_yaw);
        assert!(dyaw.abs() <= 2.5 * DT + 1e-3, "{dyaw}");
        prev_yaw = yaw;
    }
    // moving +x with the camera behind (-y): up x (+y) = -x, v.n < 0 -> yaw decreases (camera swings
    // to keep behind the runner, towards -x of the focus i.e. yaw from -pi/2 towards -pi)
    let yaw = FUN_0011efb0(sub(cm.pos(), add(v3(cm.cpdefi.pos_adjust), cm.pos_adjust1))).0;
    assert!(yaw < -std::f32::consts::FRAC_PI_2 - 0.01, "{yaw}");
    assert!(g > 0.0);
}

#[test]
fn follow_manual_right_stick() {
    let mut w = PathWorld { pos: [0.0; 3], vel: [0.0; 3] };
    let (mut cm, _) = setup(&mut w, 1.0);
    let mut t = 1.0 + DT;
    FUN_0014d478(&mut cm, &mut w, &CplcyCtx::default(), CmClock { t, dt: DT, dt_real: DT });
    let y0 = FUN_0011efb0(cm.pos()).0;
    let pad = CplcyCtx { right: [1.0, 0.0, 1.0], ..Default::default() };
    t += DT;
    FUN_0014d478(&mut cm, &mut w, &pad, CmClock { t, dt: DT, dt_real: DT });
    // rate = clamp(1*|1|*2.5, [0 - 15dt, 0 + 15dt]) = 0.25; atan2(0, 1) < 15 deg -> pitch rate 0
    // yaw += 0.25 * dt_real
    let y1 = FUN_0011efb0(cm.pos()).0;
    assert!(approx(FUN_0011cf60(y1 - y0), 0.25 * DT, 1e-3), "{}", y1 - y0);
}

#[test]
fn push_blend_over_follow_and_revoke() {
    let mut w = PathWorld { pos: [0.0; 3], vel: [0.0; 3] };
    let (mut cm, follow) = setup(&mut w, 1.0);
    let mut t = 1.0 + DT;
    FUN_0014d478(&mut cm, &mut w, &CplcyCtx::default(), CmClock { t, dt: DT, dt_real: DT });
    let k = |x: f32| CmKey { pos: [x, 0.0, 0.0], rot: [0.0; 3], fov: 1.0 };
    let b = add_policy(&mut cm, Box::new(CplcyBlend::new(CmBlend { a: k(0.0), b: k(100.0), duration: 1.0, curve: 0, cut_on_end: true })));
    let c = CmClock { t, dt: DT, dt_real: DT };
    assert!(FUN_0014e328(&mut cm, 5, b, 0, 0, c));
    assert_eq!(cm.acpr[0].pcplcy, b);
    assert_eq!(cm.acpr[1].pcplcy, follow);
    t += 0.5;
    FUN_0014d478(&mut cm, &mut w, &CplcyCtx::default(), CmClock { t, dt: DT, dt_real: DT });
    assert!(approx(cm.pos()[0], 50.0, 1e-4)); // linear blend half way
    FUN_0014e4c8(&mut cm, 2, 0, b, 0, 0, CmClock { t, dt: DT, dt_real: DT });
    assert_eq!(cm.acpr[0].pcplcy, follow);
    // follow re-activated; blend cut_on_end -> cut: the follow places the camera directly
    // (cut != 0: springs zeroed, distance = |s_dist| = 450, pitch = params pitch range)
    t += DT;
    FUN_0014d478(&mut cm, &mut w, &CplcyCtx::default(), CmClock { t, dt: DT, dt_real: DT });
    let focus = add(v3(cm.cpdefi.pos_adjust), cm.pos_adjust1);
    let d = sub(cm.pos(), focus);
    assert!(approx(len(d), 450.0 + 100.0, 1e-4), "{}", len(d));
    let e = 125.0f32.atan2(450.0);
    assert!(approx(FUN_0011efb0(d).1, e, 1e-4));
}

/// WP-I2 (UI): disc fonts + text layout FUN_0015ef00..FUN_0015ffd0 and the FUN_001b3700 text panel.
/// Expected values come from the ELF data (font headers / glyph table) and the decompiled arithmetic.
mod ui_text {
    use sly2_recomp::engine::camera::hud::{Anchor, HudLayout};
    use sly2_recomp::engine::disc::iso::IsoImage;
    use sly2_recomp::engine::ui::caption::caption;
    use sly2_recomp::engine::ui::font::Fonts;
    use sly2_recomp::engine::ui::text::{Text, TextRect};
    use std::path::PathBuf;

    fn fonts() -> Fonts {
        let r = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
        let r = if r.join("Sly 2 - Band of Thieves (USA).iso").exists() { r } else { PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/..")) };
        let iso = IsoImage::open(r.join("Sly 2 - Band of Thieves (USA).iso")).expect("ISO");
        Fonts::from_elf(&iso.read_file("SCUS_973.16").unwrap()).expect("fonts")
    }

    #[test]
    fn fonts_from_elf() {
        let f = fonts();
        assert_eq!(f.fonts.len(), 2);
        let f0 = &f.fonts[0];
        // header 0x27a250: ff ff ff ff 19 0a 29 00 | 1.0f | a3 00
        assert_eq!((f0.id, f0.b4, f0.adv, f0.line_h, f0.xs, f0.ys, f0.glyphs.len()), (0, 0x19, 10, 41, 1.0, 1.0, 163));
        assert_eq!(f.fonts[1].glyphs.len(), 22);
        assert_eq!((f.images[0].w, f.images[0].h), (512, 512)); // PSMT4 @0x27a790
        assert_eq!((f.images[1].w, f.images[1].h), (256, 128)); // PSMT8 @0x29a8d0
        // CLUT 0 from FUN_001147d8: entry 1 = 67 67 67 7d -> alpha 0x7d*2 = 0xfa
        assert!(f.images[0].rgba.chunks(4).any(|p| p == [0x67, 0x67, 0x67, 0xfa]));
        assert_eq!(f.select, vec![b'0', b'2']);
        assert_eq!(f.default, 0);
        assert_eq!(f.ctype[b' ' as usize] & 0x10, 0);
        assert_ne!(f.ctype[b'!' as usize] & 0x10, 0);
    }

    #[test]
    fn find_glyph_and_advance() {
        let f = fonts();
        for font in &f.fonts {
            for c in 0..=255u8 {
                let lin = font.glyphs.iter().find(|g| g.ch == f.char_map[c as usize]);
                assert_eq!(font.find_glyph(&f.char_map, c), lin, "char {c}");
            }
        }
        let f0 = &f.fonts[0];
        let a = f0.find_glyph(&f.char_map, b'A').unwrap();
        assert_eq!((a.ch, a.u, a.v, a.w), (65, 18, 43, 16));
        assert_eq!(f0.dx_glyph(&f.char_map, b'A'), 17.0); // (w + 1) * xs
        assert!(f0.find_glyph(&f.char_map, 1).is_none());
        assert_eq!(f0.dx_glyph(&f.char_map, 1), 10.0); // default advance * xs
    }

    #[test]
    fn scale_stack_quirk() {
        let mut f = fonts();
        let f0 = &mut f.fonts[0];
        f0.push_scale(0.8, 0.5);
        assert_eq!((f0.xs, f0.ys), (0.8, 0.5));
        f0.push_scale(0.5, 0.5); // relative to stack[0] (1.0), not to 0.8
        assert_eq!((f0.xs, f0.ys), (0.5, 0.5));
        f0.pop_scale();
        assert_eq!((f0.xs, f0.ys), (0.8, 0.8)); // y restored from the saved x
        f0.pop_scale();
        assert_eq!((f0.xs, f0.ys), (1.0, 1.0));
    }

    fn chars(f: &mut Fonts, s: &str) -> (Vec<u8>, Vec<u32>) {
        let mut t = Text::new(s.as_bytes(), 0);
        let (mut v, mut c) = (vec![], vec![]);
        loop {
            let ch = t.next_char(f);
            if ch == 0 {
                return (v, c);
            }
            v.push(ch);
            c.push(t.color);
        }
    }

    #[test]
    fn escapes() {
        let mut f = fonts();
        assert_eq!(chars(&mut f, "a^^b&&c~~d").0, b"a^b&c~d");
        let (v, c) = chars(&mut f, "a~FF0000b~.c");
        assert_eq!(v, b"abc");
        assert_eq!(c, vec![0xff80_8080, 0xff00_0080, 0xff80_8080]); // (0xff+1)>>1 = 0x80, (0+1)>>1 = 0
        // malformed colour: prints '~' and continues
        assert_eq!(chars(&mut f, "~zz").0, b"~zz");
        // ^dd scale escape: 0.5 * current scale pushed on the font, ^. pops
        let mut t = Text::new(b"^05A^.B", 0);
        assert_eq!(t.next_char(&mut f), b'A');
        assert_eq!(f.fonts[0].xs, 0.5);
        assert_eq!(t.next_char(&mut f), b'B');
        assert_eq!(f.fonts[0].xs, 1.0);
        // &2 switches to font 1 rescaled to the base line height
        let mut t = Text::new(b"&2A", 0);
        t.next_char(&mut f);
        let l = t.local.as_ref().unwrap();
        assert!((l.line_h as f32 * l.ys - 41.0).abs() < 1e-3);
    }

    #[test]
    fn wrap_and_measure() {
        let mut f = fonts();
        let mut t = Text::new(b"AAAA AAAA", 0);
        assert_eq!(t.wrap(&mut f, 85.0), 2); // 5 * 17 fits "AAAA " then the 2nd word overflows
        assert_eq!(t.buf[4], b'\n');
        assert_eq!(t.widest(&mut f), 68.0);
        // no break opportunity: the overflowing char stays, the next one is overwritten by a newline
        // which is then counted twice (original quirks)
        let mut t = Text::new(b"AAAAAA", 0);
        assert_eq!(t.wrap(&mut f, 40.0), 3);
        assert_eq!(&t.buf[..6], b"AAA\nAA");
        // alignment: right-aligned single line ends at x + w
        let mut t = Text::new(b"AA", 0);
        let q = t.draw(&mut f, &TextRect { x: 100.0, y: 50.0, w: 200.0, h: 100.0, color: 0xff80_8080, halign: 2, valign: 1 });
        assert_eq!(q.len(), 2);
        assert_eq!(q[0].x0, 300.0 - 34.0);
        assert_eq!(q[0].y0, 50.0 + (100.0 - 41.0) * 0.5);
        assert_eq!((q[0].u0, q[0].v0, q[0].u1, q[0].v1), (18.0, 43.0, 34.0, 84.0));
        assert_eq!(q[0].rgba, [0x80, 0x80, 0x80, 0x80]);
    }

    #[test]
    fn caption_panel_layout() {
        let mut f = fonts();
        let c = caption(&mut f, b"AA");
        // width trunc(2 * 17 * 0.8) = 27, height 41 * 0.8
        assert_eq!(c.rect.w, 27.0);
        assert!((c.rect.h - 32.8).abs() < 1e-4);
        assert_eq!(c.rect.x, 320.0 - 13.5);
        assert!((c.rect.y - (476.80002 - 32.8)).abs() < 1e-3);
        assert_eq!(c.glyphs.len(), 2);
        assert_eq!(c.glyphs[0].rgba, [0xc0, 0xc0, 0xc0, 0x80]);
        assert!((c.glyphs[0].x1 - c.glyphs[0].x0 - 16.0 * 0.8).abs() < 1e-4);
        assert_eq!(c.back.rgba, [0, 0, 0, 0x40]);
        assert!(c.back.blend);
        assert_eq!(f.fonts[0].xs, 1.0); // scale popped
        // mapped into the 16:9 rect at 21:9 / 32:9
        for (w, h) in [(1920.0, 1080.0), (2560.0, 1080.0), (5120.0, 1440.0)] {
            let l = HudLayout::new(w, h);
            let r = l.map_text_rect(c.back.x0, c.back.y0, c.back.x1, c.back.y1, Anchor::Center);
            assert!(l.in_safe(&r));
            assert!((r.x + r.w * 0.5 - w * 0.5).abs() < 0.01);
            assert!((r.h / r.w - (32.8 * 448.0 / 492.80002) / 27.0).abs() < 1e-3);
        }
    }
}
