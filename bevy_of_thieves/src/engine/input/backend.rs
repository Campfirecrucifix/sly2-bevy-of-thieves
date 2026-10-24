//! Bevy backend replacing libpad/padman: samples gilrs gamepads + keyboard/mouse through `Bindings`,
//! feeds the 1:1 pad code once per frame and turns actuator commands into gilrs rumble.

use super::bindings::{Action, Bindings, VirtualPad, action_by_name};
use super::pad::{self, CheatDetector, Clock, InputLock, PadState, ReadCtx};
use super::rumble::RumbleCtx;
use bevy::input::gamepad::{Gamepad, GamepadAxis, GamepadButton, GamepadInput, GamepadRumbleIntensity, GamepadRumbleRequest};
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use std::time::Duration;

/// Engine-facing input state (the original globals around 0x2dfc00).
#[derive(Resource)]
pub struct PadInput {
    /// DAT_002dfc00 (+0x1000..)
    pub pad: PadState,
    pub lock: InputLock,
    pub cheats: CheatDetector,
    pub rumble_ctx: RumbleCtx,
    /// see `ReadCtx::dpad_override_suppressed`
    pub dpad_override_suppressed: bool,
    /// DAT_002deb30
    pub t_activity_msg: f32,
    /// set when FUN_00187998 posted the activity message this frame
    pub activity_msg: bool,
    pub bindings: Bindings,
    /// last actuator command [small, big] sent to the device
    pub actuator: [u8; 2],
    /// WIRE-B2: port-only action bits held this frame (`bindings::EXTRA_*`)
    pub extra: u32,
}

impl PadInput {
    /// Pad state after FUN_00187030 opened `pad` (vibration mode 2, nothing locked).
    pub fn new(pad: PadState, bindings: Bindings) -> Self {
        PadInput {
            pad,
            lock: InputLock::default(),
            cheats: CheatDetector::default(),
            rumble_ctx: RumbleCtx { vibration_mode: 2, no_vibration: false },
            dpad_override_suppressed: false,
            t_activity_msg: 0.0,
            activity_msg: false,
            bindings,
            actuator: [0; 2],
            extra: 0,
        }
    }
}

pub struct PadPlugin;

impl Plugin for PadPlugin {
    fn build(&self, app: &mut App) {
        // controls.toml next to the exe (defaults written if missing); a mod may ship mods/<name>/controls.toml
        // which the mod loader can apply by replacing `PadInput::bindings`.
        let path = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("controls.toml")))
            .unwrap_or_else(|| "controls.toml".into());
        let bindings = Bindings::load_or_create(&path).unwrap_or_else(|e| {
            warn!("{e}; using default controls");
            Bindings::default()
        });
        let mut pad = PadState::new();
        // FUN_00187030: open port 0 slot 0
        pad::FUN_00187150(&mut pad, 0, 0, Clock { t: 0.0, dt: 1.0 / 60.0 }, &mut |_| {});
        app.insert_resource(PadInput::new(pad, bindings))
        .add_systems(PreUpdate, (device_script, pad_update).chain().after(bevy::input::InputSystems));
        if let Some(g) = std::env::var("SLY2_PAD_SCRIPT").ok().map(|s| GlobalPadScript::parse(&s)).filter(|g| !g.steps.is_empty()) {
            app.insert_resource(g);
        }
        if let Some(d) = std::env::var("SLY2_PAD_SCRIPT").ok().map(|s| DeviceScript::parse(&s)).filter(|d| !d.steps.is_empty()) {
            app.insert_resource(d);
        }
    }
}

/// WP-EP1a (port addition, verification): items `gA-B:x,y[:buttons]` of env `SLY2_PAD_SCRIPT` hold a virtual pad over
/// APP frames [A, B) (counted from the first PreUpdate, menus / title / FMV included), merged with the real devices
/// before FUN_001871c0. Buttons: hex bits or names joined by `+` (START SELECT X O TRI SQ L1 L2 R1 R2 L3 R3 UP DOWN
/// LEFT RIGHT). Items without the `g` prefix stay player-frame items (`player::plugin::PadScript`).
#[derive(Resource, Default, Clone, Debug)]
pub struct GlobalPadScript {
    pub steps: Vec<(u32, u32, [f32; 2], u32)>,
    pub frame: u32,
}

impl GlobalPadScript {
    pub fn parse(src: &str) -> Self {
        let mut steps = Vec::new();
        for item in src.split(';').map(str::trim).filter_map(|s| s.strip_prefix('g')) {
            let mut it = item.split(':');
            let (Some(range), Some(st)) = (it.next(), it.next()) else { continue };
            let Some((a, b)) = range.split_once('-') else { continue };
            let (Ok(a), Ok(b)) = (a.trim().parse(), b.trim().parse()) else { continue };
            let xy: Vec<f32> = st.split(',').filter_map(|v| v.trim().parse().ok()).collect();
            let btn = it.next().map_or(0, |h| h.split('+').map(|n| button_bits(n.trim())).fold(0, |a, b| a | b));
            steps.push((a, b, [*xy.first().unwrap_or(&0.0), *xy.get(1).unwrap_or(&0.0)], btn));
        }
        GlobalPadScript { steps, frame: 0 }
    }
    /// Virtual pad contribution of the next app frame.
    pub fn next(&mut self) -> ([f32; 2], u32) {
        let f = self.frame;
        self.frame += 1;
        let (mut st, mut btn) = ([0.0f32; 2], 0u32);
        for &(_, _, s, k) in self.steps.iter().filter(|x| f >= x.0 && f < x.1) {
            st = s;
            btn |= k;
        }
        (st, btn)
    }
}

/// WP-EP1e (port addition, verification of R13): items `kA-B:Key[+Key]` / `pA-B:Pad[+Pad]` of env `SLY2_PAD_SCRIPT`
/// hold keyboard keys (Bevy `KeyCode` / `Mouse:*` names) or buttons / axes (`GamepadButton` names, `LeftStickY+`)
/// of a synthetic gamepad over APP frames [A, B), so the real `sample` path (controls.toml) is exercised.
#[derive(Resource, Default, Clone, Debug)]
pub struct DeviceScript {
    /// (from, to, keyboard?, names)
    pub steps: Vec<(u32, u32, bool, Vec<String>)>,
    pub frame: u32,
    pub pad: Option<Entity>,
    /// FEEL R82 (verification): mouse motion of this frame from `kA-B:MouseDX=n+MouseDY=n` items (pixels/frame)
    pub motion: Vec2,
}

impl DeviceScript {
    pub fn parse(src: &str) -> Self {
        let mut steps = Vec::new();
        for item in src.split(';').map(str::trim) {
            let (kb, rest) = match item.split_at_checked(1) {
                Some(("k", r)) => (true, r),
                Some(("p", r)) => (false, r),
                _ => continue,
            };
            let Some((range, names)) = rest.split_once(':') else { continue };
            let Some((a, b)) = range.split_once('-') else { continue };
            let (Ok(a), Ok(b)) = (a.trim().parse(), b.trim().parse()) else { continue };
            // names joined by `+`; an axis name keeps its own trailing sign (`LeftStickY+`)
            let mut list: Vec<String> = Vec::new();
            for n in names.split('+').map(str::trim) {
                match list.last_mut() {
                    Some(l) if n.is_empty() => l.push('+'),
                    _ if n.is_empty() => {}
                    _ => list.push(n.to_string()),
                }
            }
            steps.push((a, b, kb, list));
        }
        DeviceScript { steps, ..Default::default() }
    }
}

fn device_script(
    s: Option<ResMut<DeviceScript>>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut pads: Query<&mut Gamepad>,
    mut commands: Commands,
) {
    let Some(mut s) = s else { return };
    let s = &mut *s;
    let f = s.frame;
    s.frame += 1;
    let Some(e) = s.pad else {
        s.pad = Some(commands.spawn(Gamepad::default()).id());
        return;
    };
    let mut pad = pads.get_mut(e).ok();
    let mut motion = Vec2::ZERO;
    for (a, b, kb, names) in &s.steps {
        let on = f >= *a && f < *b;
        if !on && f != *b {
            continue;
        }
        for n in names {
            if let Some(d) = n.strip_prefix("MouseDX=").and_then(|v| v.parse::<f32>().ok()) {
                motion.x += if on { d } else { 0.0 };
            } else if let Some(d) = n.strip_prefix("MouseDY=").and_then(|v| v.parse::<f32>().ok()) {
                motion.y += if on { d } else { 0.0 };
            } else if *kb {
                match (key_code(n), mouse_button(n)) {
                    (Some(k), _) if on => keys.press(k),
                    (Some(k), _) => keys.release(k),
                    (_, Some(m)) if on => mouse.press(m),
                    (_, Some(m)) => mouse.release(m),
                    _ => {}
                }
            } else if let Some(g) = pad.as_deref_mut() {
                if let Some(btn) = pad_button(n) {
                    if on {
                        g.digital_mut().press(btn);
                    } else {
                        g.digital_mut().release(btn);
                    }
                    g.analog_mut().set(btn, if on { 1.0 } else { 0.0 });
                } else if let Some((axis, sign)) = pad_axis(n) {
                    g.analog_mut().set(axis, if on { sign } else { 0.0 });
                }
            }
        }
    }
    s.motion = motion;
}

fn button_bits(n: &str) -> u32 {
    match n.to_ascii_uppercase().as_str() {
        "START" => pad::BTN_START,
        "SELECT" => pad::BTN_SELECT,
        "X" | "CROSS" => pad::BTN_CROSS,
        "O" | "CIRCLE" => pad::BTN_CIRCLE,
        "TRI" | "TRIANGLE" => pad::BTN_TRIANGLE,
        "SQ" | "SQUARE" => pad::BTN_SQUARE,
        "L1" => pad::BTN_L1,
        "L2" => pad::BTN_L2,
        "R1" => pad::BTN_R1,
        "R2" => pad::BTN_R2,
        "L3" => pad::BTN_L3,
        "R3" => pad::BTN_R3,
        "UP" => pad::BTN_UP,
        "DOWN" => pad::BTN_DOWN,
        "LEFT" => pad::BTN_LEFT,
        "RIGHT" => pad::BTN_RIGHT,
        h => u32::from_str_radix(h.trim_start_matches("0X"), 16).unwrap_or(0),
    }
}

fn key_code(name: &str) -> Option<KeyCode> {
    macro_rules! keys { ($($k:ident)*) => { match name { $(stringify!($k) => Some(KeyCode::$k),)* _ => None } } }
    keys!(Backquote Backslash BracketLeft BracketRight Comma Digit0 Digit1 Digit2 Digit3 Digit4 Digit5 Digit6
        Digit7 Digit8 Digit9 Equal KeyA KeyB KeyC KeyD KeyE KeyF KeyG KeyH KeyI KeyJ KeyK KeyL KeyM KeyN KeyO KeyP
        KeyQ KeyR KeyS KeyT KeyU KeyV KeyW KeyX KeyY KeyZ Minus Period Quote Semicolon Slash AltLeft AltRight
        Backspace CapsLock ControlLeft ControlRight Enter ShiftLeft ShiftRight Space Tab Delete End Home Insert
        PageDown PageUp ArrowDown ArrowLeft ArrowRight ArrowUp Numpad0 Numpad1 Numpad2 Numpad3 Numpad4 Numpad5
        Numpad6 Numpad7 Numpad8 Numpad9 NumpadAdd NumpadDecimal NumpadDivide NumpadEnter NumpadMultiply
        NumpadSubtract Escape F1 F2 F3 F4 F5 F6 F7 F8 F9 F10 F11 F12)
}

fn mouse_button(name: &str) -> Option<MouseButton> {
    Some(match name.strip_prefix("Mouse:")? {
        "Left" => MouseButton::Left,
        "Right" => MouseButton::Right,
        "Middle" => MouseButton::Middle,
        "Back" => MouseButton::Back,
        "Forward" => MouseButton::Forward,
        _ => return None,
    })
}

fn pad_button(name: &str) -> Option<GamepadButton> {
    use GamepadButton::*;
    Some(match name {
        "South" => South, "East" => East, "North" => North, "West" => West, "C" => C, "Z" => Z,
        "LeftTrigger" => LeftTrigger, "LeftTrigger2" => LeftTrigger2, "RightTrigger" => RightTrigger,
        "RightTrigger2" => RightTrigger2, "Select" => Select, "Start" => Start, "Mode" => Mode,
        "LeftThumb" => LeftThumb, "RightThumb" => RightThumb, "DPadUp" => DPadUp, "DPadDown" => DPadDown,
        "DPadLeft" => DPadLeft, "DPadRight" => DPadRight,
        _ => return None,
    })
}

fn pad_axis(name: &str) -> Option<(GamepadAxis, f32)> {
    let (a, s) = if let Some(a) = name.strip_suffix('+') { (a, 1.0) } else { (name.strip_suffix('-')?, -1.0) };
    Some((
        match a {
            "LeftStickX" => GamepadAxis::LeftStickX,
            "LeftStickY" => GamepadAxis::LeftStickY,
            "RightStickX" => GamepadAxis::RightStickX,
            "RightStickY" => GamepadAxis::RightStickY,
            "LeftZ" => GamepadAxis::LeftZ,
            "RightZ" => GamepadAxis::RightZ,
            _ => return None,
        },
        s,
    ))
}

fn apply(v: &mut VirtualPad, action: Action, value: f32) {
    if value <= 0.0 {
        return;
    }
    match action {
        Action::Button(bit) => {
            if value >= 0.5 {
                v.buttons |= bit;
            }
            let i = bit.trailing_zeros() as usize;
            v.analog[i] = v.analog[i].max(value.min(1.0));
        }
        Action::Stick(s, a, sign) => v.sticks[s as usize][a as usize] += value * sign as f32,
        Action::Extra(bit) => {
            if value >= 0.5 {
                v.extra |= bit;
            }
        }
    }
}

/// FEEL R80 (port addition): radial stick deadzone of a physical gamepad (a resting / worn stick reads a few
/// percent off centre; summed onto the keyboard stick it made Sly creep). Rescaled so full tilt stays 1.0.
pub const PAD_STICK_DEADZONE: f32 = 0.2;

fn deadzone(s: [f32; 2]) -> [f32; 2] {
    let m = (s[0] * s[0] + s[1] * s[1]).sqrt();
    if m <= PAD_STICK_DEADZONE {
        return [0.0, 0.0];
    }
    let k = ((m - PAD_STICK_DEADZONE) / (1.0 - PAD_STICK_DEADZONE)).min(1.0) / m;
    [s[0] * k, s[1] * k]
}

/// FEEL R82: smallest mouse-driven right stick = just past FUN_00187768's dead zone (0.35 of a range of up to 1.0).
pub const MOUSE_STICK_EDGE: f32 = 0.36;

/// Sample everything into one virtual DualShock2. Buttons of keyboard/mouse and the active gamepad are merged;
/// FEEL R80: a stick held on the keyboard (or moved by the mouse) replaces that pad stick instead of being summed
/// with it, and the pad sticks get a radial deadzone (`PAD_STICK_DEADZONE`).
pub fn sample(
    b: &Bindings,
    keys: &ButtonInput<KeyCode>,
    mouse: &ButtonInput<MouseButton>,
    motion: Vec2,
    gamepad: Option<&Gamepad>,
) -> VirtualPad {
    let mut v = sample_kb(b, keys, mouse, motion);
    if let Some(g) = gamepad {
        let p = sample_pad(b, g);
        v.buttons |= p.buttons;
        v.extra |= p.extra;
        for i in 0..16 {
            v.analog[i] = v.analog[i].max(p.analog[i]);
        }
        let old = std::env::var("SLY2_FEEL_R80PAD").is_ok_and(|s| s == "0");
        for i in 0..2 {
            if old {
                v.sticks[i] = [(v.sticks[i][0] + p.sticks[i][0]).clamp(-1.0, 1.0), (v.sticks[i][1] + p.sticks[i][1]).clamp(-1.0, 1.0)];
            } else if v.sticks[i] == [0.0, 0.0] {
                v.sticks[i] = deadzone(p.sticks[i]);
            }
        }
    }
    v
}

fn sample_pad(b: &Bindings, g: &Gamepad) -> VirtualPad {
    sample_pad_ax(b, g, &|a| g.get(a).unwrap_or(0.0))
}

fn sample_pad_ax(b: &Bindings, g: &Gamepad, ax: &dyn Fn(GamepadAxis) -> f32) -> VirtualPad {
    let mut v = VirtualPad::default();
    for (name, srcs) in &b.gamepad {
        let Some(action) = action_by_name(name) else { continue };
        for s in srcs {
            let value = if let Some(btn) = pad_button(s) {
                let v = g.get(btn).unwrap_or(0.0);
                if (g.pressed(btn) || g.just_pressed(btn)) && v < 0.5 { 1.0 } else { v }
            } else if let Some((axis, sign)) = pad_axis(s) {
                ax(axis) * sign
            } else {
                0.0
            };
            apply(&mut v, action, value);
        }
    }
    for s in v.sticks.iter_mut().flatten() {
        *s = s.clamp(-1.0, 1.0);
    }
    v
}

/// FEEL R82 (port addition, USER PLAYTEST 6 'controller unusable'): every connected gamepad is read, so the pad
/// that really carries the DualSense / DS4 input drives the game whichever gilrs device it is (gilrs on Windows
/// lists the DualSense as 'PS5 Controller' with an SDL mapping plus a 'No mapping found for UUID 0000..' phantom
/// that reports vendor 0 instead of None, so R80's vendor test let the phantom win). A stuck axis cannot push Sly:
/// an axis's value during the first `PAD_REST_FRAMES` frames a pad is seen is its rest value (a trigger / unmapped
/// axis resting at -1 / +1 under the default mapping) and only the offset from rest counts. Buttons of all pads are
/// OR-ed, each stick is the strongest pad stick after the deadzone.
#[derive(Default, Debug)]
pub struct PadMerge {
    /// (pad, axis) -> rest value
    pub rest: std::collections::HashMap<(Entity, GamepadAxis), f32>,
    /// pad -> frames seen
    pub seen: std::collections::HashMap<Entity, u32>,
}

/// FEEL R82: frames after a pad appears during which its axis readings count as rest.
pub const PAD_REST_FRAMES: u32 = 30;

impl PadMerge {
    pub fn sample(&mut self, b: &Bindings, pads: &[(Entity, &Gamepad)]) -> Option<VirtualPad> {
        self.seen.retain(|e, _| pads.iter().any(|p| p.0 == *e));
        self.rest.retain(|k, _| pads.iter().any(|p| p.0 == k.0));
        let mut out: Option<VirtualPad> = None;
        for &(e, g) in pads {
            let n = self.seen.entry(e).or_insert(0);
            *n += 1;
            let early = *n <= PAD_REST_FRAMES;
            if early {
                for (axis, v) in g.analog().all_axes_and_values() {
                    if let GamepadInput::Axis(a) = axis {
                        self.rest.insert((e, *a), v);
                    }
                }
            }
            let rest = &self.rest;
            let p = sample_pad_ax(b, g, &|a| (g.get(a).unwrap_or(0.0) - rest.get(&(e, a)).copied().unwrap_or(0.0)).clamp(-1.0, 1.0));
            let o = out.get_or_insert_with(VirtualPad::default);
            o.buttons |= p.buttons;
            o.extra |= p.extra;
            for i in 0..16 {
                o.analog[i] = o.analog[i].max(p.analog[i]);
            }
            for i in 0..2 {
                let d = deadzone(p.sticks[i]);
                if d[0].hypot(d[1]) > o.sticks[i][0].hypot(o.sticks[i][1]) {
                    o.sticks[i] = d;
                }
            }
        }
        out
    }
}

/// FEEL R82: keyboard/mouse + the merged pads (`PadMerge`); a held keyboard/mouse stick wins over the pad stick.
pub fn sample_merged(b: &Bindings, keys: &ButtonInput<KeyCode>, mouse: &ButtonInput<MouseButton>, motion: Vec2, pads: Option<VirtualPad>) -> VirtualPad {
    let mut v = sample_kb(b, keys, mouse, motion);
    if let Some(p) = pads {
        v.buttons |= p.buttons;
        v.extra |= p.extra;
        for i in 0..16 {
            v.analog[i] = v.analog[i].max(p.analog[i]);
        }
        for i in 0..2 {
            if v.sticks[i] == [0.0, 0.0] {
                v.sticks[i] = p.sticks[i];
            }
        }
    }
    v
}

fn sample_kb(b: &Bindings, keys: &ButtonInput<KeyCode>, mouse: &ButtonInput<MouseButton>, motion: Vec2) -> VirtualPad {
    let mut v = VirtualPad::default();
    for (name, srcs) in &b.keyboard {
        let Some(action) = action_by_name(name) else { continue };
        for s in srcs {
            // a tap pressed and released within one frame still counts (just_pressed)
            let held = key_code(s).is_some_and(|k| keys.pressed(k) || keys.just_pressed(k))
                || mouse_button(s).is_some_and(|m| mouse.pressed(m) || mouse.just_pressed(m));
            apply(&mut v, action, if held { 1.0 } else { 0.0 });
        }
    }
    if b.mouse_right_stick {
        let m = Vec2::new(motion.x, -motion.y) * b.mouse_sensitivity;
        let l = m.length();
        // FEEL R82 (port addition, user playtest 5 'camera does not turn with the mouse'): FUN_00187768 drops a
        // stick below 0.35 of its calibrated range (0.75..1) and squares the rest in the camera, so ordinary mouse
        // speeds (< 13 px/frame at sensitivity 0.02) never reached the camera. Mouse motion starts just past that
        // dead zone instead. SLY2_FEEL_R82MOUSE=0 = old (linear, inside the dead zone).
        let m = if l <= 0.0 || std::env::var("SLY2_FEEL_R82MOUSE").is_ok_and(|s| s == "0") {
            m
        } else {
            m * ((MOUSE_STICK_EDGE + l).min(1.0) / l)
        };
        v.sticks[1][0] += m.x;
        v.sticks[1][1] += m.y;
    }
    for s in v.sticks.iter_mut().flatten() {
        *s = s.clamp(-1.0, 1.0);
    }
    v
}

/// Per-frame FUN_001871c0 (libpad replaced by `sample`).
fn pad_update(
    mut input: ResMut<PadInput>,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    gamepads: Query<(Entity, &Gamepad)>,
    mut rumble: MessageWriter<GamepadRumbleRequest>,
    script: Option<ResMut<GlobalPadScript>>,
    dev_script: Option<Res<DeviceScript>>,
    mut log: Local<(u32, Option<bool>, VirtualPad, Option<Entity>)>,
    mut merge: Local<PadMerge>,
) {
    let dt = time.delta_secs();
    // DAT_002ddf10 is never 0 in the original; guard the first frame
    let clock = Clock { t: time.elapsed_secs(), dt: if dt > 0.0 { dt } else { 1.0 / 60.0 } };
    // FEEL R80 (port addition, user playtest: a DualSense showed up twice, once as an unmapped gilrs device whose
    // axes pushed Sly in one direction): pads without a USB vendor id (gilrs "No mapping found for UUID 0000..")
    // are ignored while a real one is connected; the active pad is the last one that pressed a button (sticky),
    // else the first real one. SLY2_FEEL_R80PAD=0 = old (first pad, sticks summed with the keyboard).
    let old_pad = std::env::var("SLY2_FEEL_R80PAD").is_ok_and(|s| s == "0");
    // FEEL R82 (verification): env SLY2_PAD_DUMP=N logs every gamepad entity's ids and non-zero inputs every N app frames
    if let Some(n) = std::env::var("SLY2_PAD_DUMP").ok().and_then(|v| v.parse::<u32>().ok()).filter(|n| *n > 0 && log.0 % *n == 0) {
        for (e, g) in gamepads.iter() {
            let ax: Vec<_> = g.analog().all_axes_and_values().filter(|(_, v)| v.abs() > 0.001).map(|(a, v)| format!("{a:?}={v:.3}")).collect();
            let bt: Vec<_> = g.digital().get_pressed().map(|b| format!("{b:?}")).collect();
            info!("PADDUMP f{} {e:?} vendor {:?} product {:?} axes {ax:?} pressed {bt:?}", log.0, g.vendor_id(), g.product_id());
        }
    }
    let any_real = gamepads.iter().any(|(_, g)| g.vendor_id().is_some_and(|v| v != 0));
    let eligible = |g: &Gamepad| !any_real || g.vendor_id().is_some_and(|v| v != 0);
    if let Some((e, _)) = gamepads.iter().find(|(_, g)| eligible(g) && g.get_just_pressed().next().is_some()) {
        log.3 = Some(e);
    }
    if log.3.is_some_and(|e| !gamepads.get(e).is_ok_and(|(_, g)| eligible(g))) {
        log.3 = None;
    }
    let first = if old_pad {
        gamepads.iter().next()
    } else {
        log.3.and_then(|e| gamepads.get(e).ok()).or_else(|| gamepads.iter().find(|(_, g)| eligible(g)))
    };
    // raw mouse motion is reported even while the window is in the background: only a focused window
    // turns it into right-stick input (and never during a scripted verification run)
    let focused = windows.iter().next().is_some_and(|w| w.focused) && script.is_none() && dev_script.is_none();
    let motion = match &dev_script {
        Some(d) if d.motion != Vec2::ZERO => d.motion,
        _ if focused => motion.delta,
        _ => Vec2::ZERO,
    };
    // FEEL R82: all pads merged with rest calibration (SLY2_FEEL_R82PADS=0 = R80 single active pad)
    let mut v = if old_pad || std::env::var("SLY2_FEEL_R82PADS").is_ok_and(|s| s == "0") {
        sample(&input.bindings, &keys, &mouse, motion, first.map(|(_, g)| g))
    } else {
        let pads: Vec<(Entity, &Gamepad)> = gamepads.iter().collect();
        let p = merge.sample(&input.bindings, &pads);
        sample_merged(&input.bindings, &keys, &mouse, motion, p)
    };
    // WP-EP1a: scripted app-frame pad (verification)
    if let Some(mut s) = script {
        let (st, btn) = s.next();
        v.buttons |= btn;
        for i in 0..16 {
            if btn & (1 << i) != 0 {
                v.analog[i] = 1.0;
            }
        }
        if st != [0.0, 0.0] {
            v.sticks[0] = [st[0].clamp(-1.0, 1.0), st[1].clamp(-1.0, 1.0)];
        }
    }
    let raw = v.to_raw();
    // WP-EP1e (verification): env SLY2_INPUT_LOG=1 logs the sampled virtual pad whenever it changes
    let on = *log.1.get_or_insert_with(|| std::env::var("SLY2_INPUT_LOG").is_ok_and(|s| s == "1"));
    log.0 += 1;
    if on && v != log.2 {
        info!("input f{}: buttons {:#06x} sticks {:?} (gamepads {})", log.0, v.buttons, v.sticks, gamepads.iter().count());
        log.2 = v;
    }
    let input = &mut *input;
    input.extra = v.extra;
    let mut sent: Option<[u8; 2]> = None;
    let r = pad::FUN_001871c0(
        &mut input.pad,
        true, // keyboard is always connected
        Some(&raw),
        clock,
        ReadCtx {
            dpad_override_suppressed: input.dpad_override_suppressed,
            cheats: Some(&mut input.cheats),
            t_activity_msg: &mut input.t_activity_msg,
        },
        &input.rumble_ctx,
        &mut |a| sent = Some(a),
    );
    input.activity_msg = r.activity_msg;
    if let (Some(a), Some((e, _))) = (sent, first) {
        if a != input.actuator {
            rumble.write(GamepadRumbleRequest::Stop { gamepad: e });
            if a != [0, 0] {
                rumble.write(GamepadRumbleRequest::Add {
                    gamepad: e,
                    duration: Duration::from_secs(10), // held until the next actuator command
                    intensity: GamepadRumbleIntensity {
                        strong_motor: a[1] as f32 / 255.0,
                        weak_motor: if a[0] != 0 { 1.0 } else { 0.0 },
                    },
                });
            }
        }
    }
    if let Some(a) = sent {
        input.actuator = a;
    }
}

