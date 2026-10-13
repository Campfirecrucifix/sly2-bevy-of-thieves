//! WP-A11: the port's option pages (no PS2 counterpart; values live in settings.toml / controls.toml /
//! mods/load_order.txt). Pure data so the menu manager and the tests can use them without Bevy:
//! - Display: resolution, window mode, vsync (applied live to the Bevy window by `ui::plugin`) and the
//!   current window aspect (info row; Hor+ itself is the Graphics page toggle).
//! - Audio: settings.toml `[audio] master / music / sfx` (multiplied with the original's Music / Effects
//!   option sliders 0x67 / 0x68, which stay 1:1 in the original options list).
//! - Controls: read-only list of `controls.toml` (input::bindings) keyboard / gamepad bindings.
//! - Mods: the `mods/` folders in load order with enable / disable and reorder.
use crate::engine::input::bindings::{Bindings, ACTIONS};
use std::path::{Path, PathBuf};

/// Resolutions offered by the Display page (16:9, 16:10, 21:9 and 32:9).
pub const RESOLUTIONS: [(u32, u32); 10] =
    [(1280, 720), (1600, 900), (1920, 1080), (1920, 1200), (2560, 1080), (2560, 1440), (3440, 1440), (3840, 1080), (3840, 2160), (5120, 1440)];

/// Display page values (settings.toml `[video] width / height / fullscreen / vsync`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VideoOpts {
    pub width: u32,
    pub height: u32,
    /// borderless fullscreen on the primary monitor
    pub fullscreen: bool,
    pub vsync: bool,
    /// current window size in physical pixels (the aspect row shows it)
    pub win_w: f32,
    pub win_h: f32,
    /// global illumination the app was STARTED with (Solari registered at App build, see ENGINE_PORT A11)
    pub rt_running: bool,
}

impl Default for VideoOpts {
    fn default() -> Self {
        VideoOpts { width: 1920, height: 1080, fullscreen: false, vsync: true, win_w: 1920.0, win_h: 1080.0, rt_running: false }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VidRow {
    Resolution,
    Window,
    VSync,
    Aspect,
}

pub const VID_ROWS: [VidRow; 4] = [VidRow::Resolution, VidRow::Window, VidRow::VSync, VidRow::Aspect];

/// Name of an aspect ratio ("16:9", "32:9", ...; else "2.10:1").
pub fn aspect_name(w: f32, h: f32) -> String {
    if w <= 0.0 || h <= 0.0 {
        return "?".into();
    }
    let r = w / h;
    const KNOWN: [(f32, &str); 6] = [(4.0 / 3.0, "4:3"), (16.0 / 10.0, "16:10"), (16.0 / 9.0, "16:9"), (21.0 / 9.0, "21:9"), (43.0 / 18.0, "21:9"), (32.0 / 9.0, "32:9")];
    KNOWN.iter().find(|(k, _)| (r / k - 1.0).abs() < 0.02).map_or_else(|| format!("{r:.2}:1"), |(_, n)| n.to_string())
}

impl VideoOpts {
    pub fn label(&self, row: VidRow) -> String {
        let on = |b: bool| if b { "On" } else { "Off" };
        match row {
            VidRow::Resolution => format!("Resolution: {}x{}", self.width, self.height),
            VidRow::Window => format!("Window: {}", if self.fullscreen { "Borderless" } else { "Windowed" }),
            VidRow::VSync => format!("VSync: {}", on(self.vsync)),
            VidRow::Aspect => format!("Screen: {}", aspect_name(self.win_w, self.win_h)),
        }
    }

    /// Cross / right = +1, left = -1. Returns whether a value changed (the aspect row is info only).
    pub fn adjust(&mut self, row: VidRow, d: i32) -> bool {
        match row {
            VidRow::Resolution => {
                let n = RESOLUTIONS.len() as i32;
                let cur = RESOLUTIONS.iter().position(|&r| r == (self.width, self.height)).map_or(if d < 0 { 0 } else { -1 }, |k| k as i32);
                let k = (cur + d).rem_euclid(n) as usize;
                (self.width, self.height) = RESOLUTIONS[k];
            }
            VidRow::Window => self.fullscreen = !self.fullscreen,
            VidRow::VSync => self.vsync = !self.vsync,
            VidRow::Aspect => return false,
        }
        true
    }
}

/// Audio page values (settings.toml `[audio]`, 0..1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AudioOpts {
    pub master: f32,
    pub music: f32,
    pub sfx: f32,
}

impl Default for AudioOpts {
    fn default() -> Self {
        AudioOpts { master: 1.0, music: 1.0, sfx: 1.0 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudRow {
    Master,
    Music,
    Sfx,
}

pub const AUD_ROWS: [AudRow; 3] = [AudRow::Master, AudRow::Music, AudRow::Sfx];

impl AudioOpts {
    fn get(&mut self, row: AudRow) -> &mut f32 {
        match row {
            AudRow::Master => &mut self.master,
            AudRow::Music => &mut self.music,
            AudRow::Sfx => &mut self.sfx,
        }
    }

    pub fn label(&self, row: AudRow) -> String {
        let mut c = *self;
        let v = (*c.get(row) * 100.0).round() as i32;
        let name = match row {
            AudRow::Master => "Master",
            AudRow::Music => "Music",
            AudRow::Sfx => "Effects",
        };
        format!("{name} Level: {v}%")
    }

    /// 10 % steps; cross (d = 0) cycles +10 % and wraps 100 -> 0, left / right clamp.
    pub fn adjust(&mut self, row: AudRow, d: i32) -> bool {
        let v = self.get(row);
        let step = (*v * 10.0).round() as i32;
        let n = if d == 0 { (step + 1) % 11 } else { (step + d).clamp(0, 10) };
        let old = *v;
        *v = n as f32 / 10.0;
        *v != old
    }

    /// Volumes the port's player uses: settings levels times the original's mixer levels (DAT_003dd190 /
    /// DAT_003dd18c, `OptState::music / effects`).
    pub fn effective(&self, music: f32, effects: f32) -> (f32, f32) {
        (self.master * self.music * music, self.master * self.sfx * effects)
    }
}

/// Mods page: every folder under `mods/` in load order, enabled or not.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ModsOpts {
    pub dir: PathBuf,
    /// (folder name, enabled) in load order (first = lowest priority)
    pub entries: Vec<(String, bool)>,
    /// changed since start (the overlay is built at startup -> "restart to apply")
    pub dirty: bool,
}

/// Marker of a disabled mod in mods/load_order.txt (sly2_db's ModLoadOrder::scan skips the folder, A13).
pub const OFF_PREFIX: &str = "# off: ";

impl ModsOpts {
    /// Scan `dir` like `Settings::mod_overlay`: settings.toml `[mods] order` if non-empty (listed = enabled,
    /// the other folders disabled), else sly2_db's `ModLoadOrder::scan` order (enabled) followed by the
    /// folders of load_order.txt's `# off: name` lines (disabled; the scan leaves them out since A13).
    pub fn scan(dir: &Path, settings_order: &[String]) -> Self {
        let order = crate::engine::disc::mods::ModLoadOrder::scan(dir).unwrap_or_default();
        let mut names: Vec<String> = order.mods.iter().filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned())).collect();
        let off: Vec<String> = std::fs::read_to_string(dir.join("load_order.txt"))
            .unwrap_or_default()
            .lines()
            .filter_map(|l| l.trim().strip_prefix(OFF_PREFIX.trim_end()).map(|s| s.trim().to_string()))
            .filter(|n| !names.contains(n) && dir.join(n).is_dir())
            .collect();
        names.extend(off.iter().cloned());
        let entries = if settings_order.is_empty() {
            names.iter().map(|n| (n.clone(), !off.contains(n))).collect()
        } else {
            let mut e: Vec<(String, bool)> = settings_order.iter().filter(|n| names.contains(n)).map(|n| (n.clone(), true)).collect();
            e.extend(names.iter().filter(|n| !settings_order.contains(n)).map(|n| (n.clone(), false)));
            e
        };
        ModsOpts { dir: dir.to_path_buf(), entries, dirty: false }
    }

    pub fn label(&self, k: usize) -> String {
        match self.entries.get(k) {
            Some((n, on)) => format!("{}. {}: {}", k + 1, n, if *on { "On" } else { "Off" }),
            None => "No mods found".into(),
        }
    }

    /// Cross toggles (d = 0); left / right move the mod one place earlier / later.
    pub fn adjust(&mut self, k: usize, d: i32) -> bool {
        if k >= self.entries.len() {
            return false;
        }
        if d == 0 {
            self.entries[k].1 = !self.entries[k].1;
        } else {
            let j = k as i32 + d;
            if j < 0 || j >= self.entries.len() as i32 {
                return false;
            }
            self.entries.swap(k, j as usize);
        }
        self.dirty = true;
        true
    }

    /// Enabled mods in load order (settings.toml `[mods] order`).
    pub fn enabled(&self) -> Vec<String> {
        self.entries.iter().filter(|e| e.1).map(|e| e.0.clone()).collect()
    }

    /// mods/load_order.txt text: enabled mods in order, then `# off: name` per disabled mod. Since A13
    /// ModLoadOrder::scan honours the `# off:` lines, so load_order.txt alone disables a mod (the settings.toml
    /// `[mods] order` list written alongside agrees with it).
    pub fn load_order_text(&self) -> String {
        let mut s = String::from("# Sly 2 recomp mod load order (first = lowest priority); written by the Mods menu\n");
        for (n, on) in &self.entries {
            if *on {
                s.push_str(n);
                s.push('\n');
            }
        }
        for (n, on) in &self.entries {
            if !*on {
                s.push_str(OFF_PREFIX);
                s.push_str(n);
                s.push('\n');
            }
        }
        s
    }

    pub fn write(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.dir)?;
        std::fs::write(self.dir.join("load_order.txt"), self.load_order_text())
    }
}

fn pretty(s: &str) -> String {
    for (p, r) in [("Mouse:", "Mouse "), ("Key", ""), ("Arrow", ""), ("Digit", "")] {
        if let Some(t) = s.strip_prefix(p) {
            return format!("{r}{t}");
        }
    }
    s.to_string()
}

/// Controls page rows: "<action>: <keys> / <pad>" for every `input::bindings::ACTIONS` entry.
pub fn controls_rows(b: &Bindings) -> Vec<String> {
    ACTIONS
        .iter()
        .map(|(name, _)| {
            let k = b.keyboard.get(*name).map(|v| v.iter().map(|s| pretty(s)).collect::<Vec<_>>().join(",")).unwrap_or_default();
            let g = b.gamepad.get(*name).and_then(|v| v.first().cloned()).unwrap_or_default();
            let n = name.replace("left_stick_", "LS ").replace("right_stick_", "RS ");
            format!("{n}: {k} / {g}")
        })
        .collect()
}
