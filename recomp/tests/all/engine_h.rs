//! WP-H tests: expected values derived by hand from the decompiled pad code (FUN_00187998, FUN_00187768,
//! FUN_00187e88, FUN_001d51b8 ...).
use sly2_recomp::engine::input::bindings::{Bindings, VirtualPad};
use sly2_recomp::engine::input::pad::*;
use sly2_recomp::engine::input::rumble::{RUMBLE_PATTERNS, RumbleCtx};

const DT: f32 = 1.0 / 60.0;

fn clk(t: f32) -> Clock {
    Clock { t, dt: DT }
}

fn ready_pad() -> PadState {
    let mut p = PadState::new();
    FUN_00187150(&mut p, 0, 0, clk(0.0), &mut |_| {});
    FUN_001875b8(&mut p, 3, 4, clk(0.0), &mut |_| {});
    p
}

fn read(p: &mut PadState, raw: &RawPad, t: f32) -> ReadResult {
    let mut tm = 0.0;
    FUN_00187998(p, Some(raw), clk(t), ReadCtx { dpad_override_suppressed: false, cheats: None, t_activity_msg: &mut tm })
}

fn raw_buttons(b: u32) -> RawPad {
    RawPad { buttons_wire: (b as u16) ^ 0xffff, ..RawPad::default() }
}

#[test]
fn connect_resets_and_latches() {
    let p = ready_pad();
    assert_eq!((p.state, p.kind, p.latch, p.rumble_enabled, p.dpad_mag), (3, 4, 0xffff, 1, 1.0));
    assert_eq!(p.left.range, [[-0.75, 0.75]; 4]);
    assert_eq!(p.rumble.state, 1); // FUN_001d5150 on gaining actuators
}

#[test]
fn edge_detection() {
    let mut p = ready_pad();
    // held at connect: latched, no press until released
    read(&mut p, &raw_buttons(BTN_CROSS), 0.1);
    assert_eq!((p.buttons, p.pressed), (BTN_CROSS, 0));
    read(&mut p, &raw_buttons(0), 0.2);
    assert_eq!((p.buttons, p.pressed, p.latch), (0, 0, 0));
    read(&mut p, &raw_buttons(BTN_CROSS | BTN_SQUARE), 0.3);
    assert_eq!(p.pressed, BTN_CROSS | BTN_SQUARE);
    // still held next frame: not pressed again, but unconsumed presses persist (pressed is recomputed)
    read(&mut p, &raw_buttons(BTN_CROSS | BTN_SQUARE), 0.4);
    assert_eq!(p.pressed, 0);
    read(&mut p, &raw_buttons(0), 0.5);
    read(&mut p, &raw_buttons(BTN_CROSS), 0.6);
    assert_eq!(FUN_00187d80(&mut p, BTN_CROSS), BTN_CROSS);
    assert_eq!(FUN_00187dd0(&p, BTN_CROSS), 0);
    assert_eq!(p.latch, BTN_CROSS);
    FUN_00187e08(&mut p, BTN_CROSS);
    assert_eq!((p.buttons, p.latch), (0, 0));
}

#[test]
fn read_failure_and_swap() {
    let mut p = ready_pad();
    read(&mut p, &RawPad::default(), 1.0);
    let mut tm = 0.0;
    fn ctx(tm: &mut f32) -> ReadCtx<'_> { ReadCtx { dpad_override_suppressed: false, cheats: None, t_activity_msg: tm } }
    assert_eq!(FUN_00187998(&mut p, None, clk(2.9), ctx(&mut tm)).ok, 1);
    assert_eq!(FUN_00187998(&mut p, None, clk(3.0), ctx(&mut tm)).ok, 0);
    assert_eq!(read(&mut p, &RawPad { mode_id: 0x73, ..RawPad::default() }, 3.1).ok, 0);
}

#[test]
fn stick_conversion_and_deadzone() {
    let mut p = ready_pad();
    // first read (t_read == 0): sticks stay inactive -> 0
    read(&mut p, &RawPad { lx: 0xff, ..RawPad::default() }, 0.1);
    assert_eq!((p.left.active, p.left.mag, p.left.x), (0, 0.0, 0.0));
    // unchanged and inside the [0x60,0xa0] box -> still inactive
    read(&mut p, &RawPad { lx: 0x90, ..RawPad::default() }, 0.2);
    assert_eq!(p.left.active, 1); // changed 0xff -> 0x90 activates
    let mut q = ready_pad();
    read(&mut q, &RawPad { lx: 0x90, ..RawPad::default() }, 0.1);
    read(&mut q, &RawPad { lx: 0x90, ..RawPad::default() }, 0.2);
    assert_eq!(q.left.active, 0);
    read(&mut q, &RawPad { lx: 0x90, ly: 0xa1, ..RawPad::default() }, 0.3);
    assert_eq!(q.left.active, 1);

    // full right: fx = 510/255 - 1 = 1, range max grows to 1 -> mag clamps to 1
    read(&mut p, &RawPad { lx: 0xff, ..RawPad::default() }, 0.3);
    assert!((p.left.mag - 1.0).abs() < 1e-6);
    assert!((p.left.x - 1.0).abs() < 1e-4 && p.left.y.abs() < 0.005);
    assert!((p.left.range[0][1] - 1.0).abs() < 1e-5);

    // half right on a fresh pad: fx = 382/255 - 1 = 0.498039, m = fx / 0.75, mag = (m - 0.35) * 2
    let mut p = ready_pad();
    read(&mut p, &RawPad::default(), 0.1);
    read(&mut p, &RawPad { lx: 0xbf, ..RawPad::default() }, 0.2);
    let fx = 382.0f32 * 0.003921569 - 1.0;
    let mag = (fx / 0.75 - 0.35) * 2.0;
    assert!((p.left.mag - mag).abs() < 1e-5, "{} vs {mag}", p.left.mag);
    assert!((mag - 0.628104).abs() < 1e-4);
    // inside the radial dead zone: fx = 0.2 -> m = 0.267 < 0.35
    read(&mut p, &RawPad { lx: 0x99, ..RawPad::default() }, 0.3);
    assert_eq!(p.left.mag, 0.0);
    // y is up-positive: ly = 0 -> fy = 1
    read(&mut p, &RawPad { ly: 0x00, ..RawPad::default() }, 0.4);
    assert!(p.left.y > 0.99 && p.left.x.abs() < 0.005); // 0x80 is not exact centre (fx = 0.0039)
    // velocity = delta / dt
    assert!((p.vel[1] - (p.left.y - 0.0) / DT).abs() < 1e-2);
}

#[test]
fn dpad_drives_left_stick() {
    let mut p = ready_pad();
    read(&mut p, &raw_buttons(0), 0.1);
    read(&mut p, &raw_buttons(BTN_LEFT | BTN_UP), 0.2);
    assert!((p.left.x + 0.70710677).abs() < 1e-6 && (p.left.y - 0.70710677).abs() < 1e-6);
    assert_eq!(p.left.mag, 1.0);
    let mut tm = 0.0;
    FUN_00187998(&mut p, Some(&raw_buttons(BTN_LEFT)), clk(0.3), ReadCtx {
        dpad_override_suppressed: true,
        cheats: None,
        t_activity_msg: &mut tm,
    });
    assert_eq!(p.left.mag, 0.0);
}

#[test]
fn menu_auto_repeat() {
    let mut p = ready_pad();
    read(&mut p, &raw_buttons(0), 0.0);
    read(&mut p, &raw_buttons(BTN_RIGHT), 1.0);
    assert_eq!(FUN_00187e88(&mut p, clk(1.0)), 1);
    assert_eq!(FUN_00187e88(&mut p, clk(1.2)), 0); // first delay 0.3333
    assert_eq!(FUN_00187e88(&mut p, clk(1.34)), 1);
    assert!((p.dt_repeat_x - f32::from_bits(0x3eaaa64c) * 2.0 * 0.33333334).abs() < 1e-6);
    // intervals shrink by 2/3 down to 0.1
    let mut t = 1.34;
    for _ in 0..10 {
        t += p.dt_repeat_x + 1e-4;
        assert_eq!(FUN_00187e88(&mut p, clk(t)), 1);
    }
    assert_eq!(p.dt_repeat_x, 0.1);
    read(&mut p, &raw_buttons(0), t + 0.01);
    assert_eq!(FUN_00187e88(&mut p, clk(t + 0.01)), 0);
    assert_eq!(p.repeat_x, 0);
    // vertical: up = -1 via stick (mag >= 0.8, y > 0)
    read(&mut p, &RawPad { ly: 0x00, ..RawPad::default() }, t + 0.02);
    assert_eq!(FUN_001880d8(&mut p, clk(t + 0.02)), -1);
}

#[test]
fn rumble_pattern_playback() {
    let mut p = ready_pad();
    let ctx = RumbleCtx { vibration_mode: 2, no_vibration: false };
    let mut log = vec![];
    FUN_00187e28(&mut p, 3, 0.25, &ctx, &mut |a| log.push(a));
    assert_eq!(log, vec![[0, 255]]);
    assert_eq!(p.rumble.state, 2);
    let c = Clock { t: 0.0, dt: 0.05 };
    let mut tm = 0.0;
    let mut frames = vec![];
    for _ in 0..20 {
        let mut out = vec![];
        FUN_001871c0(&mut p, true, Some(&RawPad::default()), c, ReadCtx {
            dpad_override_suppressed: false,
            cheats: None,
            t_activity_msg: &mut tm,
        }, &ctx, &mut |a| out.push(a));
        frames.push((p.rumble.state, out));
    }
    // step 0 [0,255] for 0.1 s, step 1 [0,0] 0.1 s, loops; total 0.25 s then 4 frames of "off", then idle
    assert_eq!(RUMBLE_PATTERNS[3][1].act, [0, 0]);
    assert!(frames[1].1.contains(&[0, 0]));
    assert!(frames.iter().any(|(s, _)| *s == 3));
    assert_eq!(frames.last().unwrap().0, 1);
    // vibration off option
    let off = RumbleCtx { vibration_mode: 0, no_vibration: false };
    let mut n = 0;
    FUN_00187e28(&mut p, 3, 1.0, &off, &mut |_| n += 1);
    assert_eq!((n, p.rumble.state), (0, 1));
    let script = RumbleCtx { vibration_mode: 1, no_vibration: true };
    FUN_00187e28(&mut p, 3, 1.0, &script, &mut |_| n += 1);
    assert_eq!(n, 0);
}

#[test]
fn input_lock() {
    let mut p = ready_pad();
    p.latch = 0;
    let mut l = InputLock::default();
    FUN_00187078(&mut l, &mut p, 2);
    assert_eq!((l.mode, p.latch), (4, 0xffff));
    p.latch = 0;
    FUN_001870a0(&mut l, &mut p, 2);
    assert_eq!((l.mode, p.latch), (7, 0));
    FUN_00187078(&mut l, &mut p, 8);
    assert_eq!(l.mode, 0);
}

#[test]
fn cheat_sequence() {
    let mut d = CheatDetector { enabled: true, ..Default::default() };
    FUN_0014bea8(&mut d, vec![BTN_LEFT, BTN_RIGHT, BTN_LEFT], 0);
    for (cur, prev) in [(BTN_LEFT, 0), (BTN_RIGHT, BTN_LEFT), (BTN_LEFT, BTN_RIGHT)] {
        FUN_0014bf48(&mut d, true, cur, prev, clk(5.0));
    }
    assert_eq!(d.seqs[0].idx, 3);
    assert_eq!(d.deadline, 6);
}

#[test]
fn bindings_round_trip() {
    let mut b = Bindings::default();
    b.keyboard.insert("cross".into(), vec!["KeyJ".into(), "Mouse:Left".into()]);
    b.mouse_sensitivity = 0.5;
    b.mouse_right_stick = false;
    let text = b.to_toml();
    assert_eq!(Bindings::from_toml(&text).unwrap(), b);
    // partial file keeps defaults
    let p = Bindings::from_toml("[gamepad]\ncross = \"East\"\n").unwrap();
    assert_eq!(p.gamepad["cross"], vec!["East".to_string()]);
    assert_eq!(p.keyboard, Bindings::default().keyboard);
    assert!(Bindings::from_toml("[keyboard]\njump = [\"Space\"]\n").is_err());
    // file creation
    let dir = std::env::temp_dir().join("sly2_engine_h");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("controls.toml");
    let _ = std::fs::remove_file(&path);
    assert_eq!(Bindings::load_or_create(&path).unwrap(), Bindings::default());
    assert_eq!(Bindings::load_or_create(&path).unwrap(), Bindings::default());
}

#[test]
fn virtual_pad_report() {
    let mut v = VirtualPad { buttons: BTN_CROSS | BTN_START, ..Default::default() };
    v.analog[6] = 1.0;
    v.sticks = [[1.0, 1.0], [-1.0, 0.0]];
    let r = v.to_raw();
    assert_eq!(r.buttons_wire, !((BTN_CROSS | BTN_START) as u16));
    assert_eq!((r.lx, r.ly, r.rx, r.ry), (255, 0, 0, 128));
    assert_eq!(r.pressure[6], 255);
    // round trip through the game read
    let mut p = ready_pad();
    read(&mut p, &RawPad::default(), 0.1);
    read(&mut p, &r, 0.2);
    assert_eq!(p.pressed, BTN_CROSS | BTN_START);
    assert!(p.left.mag == 1.0 && p.left.x > 0.7 && p.left.y > 0.7);
}
