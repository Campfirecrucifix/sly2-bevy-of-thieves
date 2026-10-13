//! `controls.toml` (next to the exe; mods may ship one): rebinding of the 16 PS2 buttons and the stick
//! directions to keyboard keys, mouse buttons and gamepad buttons/axes. New (no original counterpart).
//!
//! Format (a TOML subset, hand-parsed to avoid a dependency):
//! ```toml
//! [keyboard]
//! cross = ["Space"]
//! [gamepad]
//! cross = ["South"]
//! [options]
//! mouse_right_stick = true
//! ```

use super::pad::*;
use std::collections::BTreeMap;

/// Bindable actions: the 16 buttons (bit = game button bit) and 8 stick half-axes.
pub const ACTIONS: [(&str, Action); 24] = [
    ("l2", Action::Button(BTN_L2)),
    ("r2", Action::Button(BTN_R2)),
    ("l1", Action::Button(BTN_L1)),
    ("r1", Action::Button(BTN_R1)),
    ("triangle", Action::Button(BTN_TRIANGLE)),
    ("circle", Action::Button(BTN_CIRCLE)),
    ("cross", Action::Button(BTN_CROSS)),
    ("square", Action::Button(BTN_SQUARE)),
    ("select", Action::Button(BTN_SELECT)),
    ("l3", Action::Button(BTN_L3)),
    ("r3", Action::Button(BTN_R3)),
    ("start", Action::Button(BTN_START)),
    ("up", Action::Button(BTN_UP)),
    ("right", Action::Button(BTN_RIGHT)),
    ("down", Action::Button(BTN_DOWN)),
    ("left", Action::Button(BTN_LEFT)),
    ("left_stick_up", Action::Stick(0, 1, 1)),
    ("left_stick_down", Action::Stick(0, 1, -1)),
    ("left_stick_left", Action::Stick(0, 0, -1)),
    ("left_stick_right", Action::Stick(0, 0, 1)),
    ("right_stick_up", Action::Stick(1, 1, 1)),
    ("right_stick_down", Action::Stick(1, 1, -1)),
    ("right_stick_left", Action::Stick(1, 0, -1)),
    ("right_stick_right", Action::Stick(1, 0, 1)),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// game button bit
    Button(u32),
    /// (stick 0 = left / 1 = right, axis 0 = x / 1 = y, sign; y up positive)
    Stick(u8, u8, i8),
}

pub fn action_by_name(name: &str) -> Option<Action> {
    ACTIONS.iter().find(|(n, _)| *n == name).map(|(_, a)| *a)
}

/// Binding set. Key/button names are Bevy `KeyCode` / `MouseButton` (as `Mouse:Left`) / `GamepadButton`
/// variant names, gamepad axes as `LeftStickX+` etc.
#[derive(Clone, Debug, PartialEq)]
pub struct Bindings {
    pub keyboard: BTreeMap<String, Vec<String>>,
    pub gamepad: BTreeMap<String, Vec<String>>,
    /// mouse motion drives the right stick
    pub mouse_right_stick: bool,
    /// stick units per pixel of mouse motion
    pub mouse_sensitivity: f32,
}

fn map(pairs: &[(&str, &[&str])]) -> BTreeMap<String, Vec<String>> {
    pairs.iter().map(|(k, v)| (k.to_string(), v.iter().map(|s| s.to_string()).collect())).collect()
}

impl Default for Bindings {
    fn default() -> Self {
        Bindings {
            keyboard: map(&[
                ("cross", &["Space"]),
                ("circle", &["KeyE"]),
                ("square", &["KeyQ", "Mouse:Left"]),
                ("triangle", &["KeyF"]),
                ("l1", &["ShiftLeft"]),
                ("r1", &["Mouse:Right"]),
                ("l2", &["KeyZ"]),
                ("r2", &["KeyC"]),
                ("l3", &["ControlLeft"]),
                ("r3", &["Mouse:Middle"]),
                ("start", &["Escape", "Enter"]),
                ("select", &["Tab"]),
                ("up", &["ArrowUp"]),
                ("down", &["ArrowDown"]),
                ("left", &["ArrowLeft"]),
                ("right", &["ArrowRight"]),
                ("left_stick_up", &["KeyW"]),
                ("left_stick_down", &["KeyS"]),
                ("left_stick_left", &["KeyA"]),
                ("left_stick_right", &["KeyD"]),
                ("right_stick_up", &["KeyI"]),
                ("right_stick_down", &["KeyK"]),
                ("right_stick_left", &["KeyJ"]),
                ("right_stick_right", &["KeyL"]),
            ]),
            gamepad: map(&[
                ("cross", &["South"]),
                ("circle", &["East"]),
                ("square", &["West"]),
                ("triangle", &["North"]),
                ("l1", &["LeftTrigger"]),
                ("r1", &["RightTrigger"]),
                ("l2", &["LeftTrigger2"]),
                ("r2", &["RightTrigger2"]),
                ("l3", &["LeftThumb"]),
                ("r3", &["RightThumb"]),
                ("start", &["Start"]),
                ("select", &["Select"]),
                ("up", &["DPadUp"]),
                ("down", &["DPadDown"]),
                ("left", &["DPadLeft"]),
                ("right", &["DPadRight"]),
                ("left_stick_up", &["LeftStickY+"]),
                ("left_stick_down", &["LeftStickY-"]),
                ("left_stick_left", &["LeftStickX-"]),
                ("left_stick_right", &["LeftStickX+"]),
                ("right_stick_up", &["RightStickY+"]),
                ("right_stick_down", &["RightStickY-"]),
                ("right_stick_left", &["RightStickX-"]),
                ("right_stick_right", &["RightStickX+"]),
            ]),
            mouse_right_stick: true,
            mouse_sensitivity: 0.02,
        }
    }
}

fn quote(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

fn unquote(s: &str) -> Result<String, String> {
    let s = s.trim();
    let inner = s.strip_prefix('"').and_then(|s| s.strip_suffix('"')).ok_or_else(|| format!("expected string: {s}"))?;
    Ok(inner.replace("\\\"", "\"").replace("\\\\", "\\"))
}

impl Bindings {
    pub fn to_toml(&self) -> String {
        let mut o = String::from("# Sly 2 controls. Names: Bevy KeyCode / Mouse:<button> / GamepadButton / <Axis>+|-\n");
        for (sec, m) in [("keyboard", &self.keyboard), ("gamepad", &self.gamepad)] {
            o += &format!("\n[{sec}]\n");
            for (k, v) in m {
                let v: Vec<String> = v.iter().map(|s| quote(s)).collect();
                o += &format!("{k} = [{}]\n", v.join(", "));
            }
        }
        o += &format!(
            "\n[options]\nmouse_right_stick = {}\nmouse_sensitivity = {:?}\n",
            self.mouse_right_stick, self.mouse_sensitivity
        );
        o
    }

    /// Parse; keys missing from the file keep their defaults, unknown actions are an error.
    pub fn from_toml(text: &str) -> Result<Bindings, String> {
        let mut b = Bindings::default();
        let mut sec = String::new();
        for (ln, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let err = |e: String| format!("controls.toml:{}: {e}", ln + 1);
            if let Some(s) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                sec = s.trim().to_string();
                continue;
            }
            let (k, v) = line.split_once('=').ok_or_else(|| err("expected key = value".into()))?;
            let (k, v) = (k.trim(), v.trim());
            match sec.as_str() {
                "keyboard" | "gamepad" => {
                    if action_by_name(k).is_none() {
                        return Err(err(format!("unknown action {k}")));
                    }
                    let list = if let Some(inner) = v.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                        inner.split(',').filter(|s| !s.trim().is_empty()).map(unquote).collect::<Result<Vec<_>, _>>()
                    } else {
                        unquote(v).map(|s| vec![s])
                    }
                    .map_err(err)?;
                    let m = if sec == "keyboard" { &mut b.keyboard } else { &mut b.gamepad };
                    m.insert(k.to_string(), list);
                }
                "options" => match k {
                    "mouse_right_stick" => b.mouse_right_stick = v.parse().map_err(|e| err(format!("{e}")))?,
                    "mouse_sensitivity" => b.mouse_sensitivity = v.parse().map_err(|e| err(format!("{e}")))?,
                    _ => return Err(err(format!("unknown option {k}"))),
                },
                _ => return Err(err(format!("unknown section [{sec}]"))),
            }
        }
        Ok(b)
    }

    /// Load `path`; write the defaults there if missing.
    pub fn load_or_create(path: &std::path::Path) -> Result<Bindings, String> {
        match std::fs::read_to_string(path) {
            Ok(t) => Bindings::from_toml(&t),
            Err(_) => {
                let b = Bindings::default();
                let _ = std::fs::write(path, b.to_toml());
                Ok(b)
            }
        }
    }
}

/// Logical input sampled by the backend, before conversion to a libpad report.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct VirtualPad {
    /// game button bits held
    pub buttons: u32,
    /// [left, right] sticks, x right / y up, each in [-1, 1]
    pub sticks: [[f32; 2]; 2],
    /// pressure per button bit index 0..15 (0..1); digital sources give 1.0
    pub analog: [f32; 16],
}

fn stick_byte(v: f32) -> u8 {
    ((v.clamp(-1.0, 1.0) + 1.0) * 127.5).round() as u8
}

impl VirtualPad {
    /// Build the libpad report the game expects (DualShock2, id 0x79, active-low buttons, y down).
    pub fn to_raw(&self) -> RawPad {
        // pressure byte order: right, left, up, down, triangle, circle, cross, square, L1, R1, L2, R2
        const ORDER: [u32; 12] = [
            BTN_RIGHT, BTN_LEFT, BTN_UP, BTN_DOWN, BTN_TRIANGLE, BTN_CIRCLE, BTN_CROSS, BTN_SQUARE, BTN_L1, BTN_R1,
            BTN_L2, BTN_R2,
        ];
        let mut pressure = [0u8; 12];
        for (p, bit) in pressure.iter_mut().zip(ORDER) {
            let a = self.analog[bit.trailing_zeros() as usize];
            let a = if self.buttons & bit != 0 { a.max(1.0 / 255.0) } else { a };
            *p = (a.clamp(0.0, 1.0) * 255.0).round() as u8;
        }
        RawPad {
            mode_id: 0x79,
            buttons_wire: (self.buttons as u16) ^ 0xffff,
            rx: stick_byte(self.sticks[1][0]),
            ry: stick_byte(-self.sticks[1][1]),
            lx: stick_byte(self.sticks[0][0]),
            ly: stick_byte(-self.sticks[0][1]),
            pressure,
        }
    }
}
