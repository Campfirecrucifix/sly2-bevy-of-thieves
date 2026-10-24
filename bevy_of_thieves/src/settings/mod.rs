//! WIRE-OPT: the port-side options layer (no PS2 counterpart). `PortSettings` = the existing
//! `game::settings::Settings` (settings.toml `iso`, `[video]`, `[audio]`, `[mods]`; Deref to it, so every old
//! field keeps working) plus the `[graphics]` section: window mode (3-way), ray-tracing quality preset and
//! the HUD layout on wide screens. Same hand-written TOML subset; the `[graphics]` lines are split off before
//! the base parser sees them, and `to_toml` appends them, so both halves round-trip in ONE file.
//! Location: `settings.toml` next to the exe; when that folder is not writable, the user config dir
//! (`%APPDATA%\bevy-of-thieves\settings.toml`, `$XDG_CONFIG_HOME` / `~/.config/bevy-of-thieves` elsewhere;
//! an existing old `sly2-recomp` folder is still used).
//! CLI flags (`--width`, `--gi`, `--mods`, ...) are applied by game.rs after loading and are never saved.
pub mod mods;

use crate::engine::game::settings::Settings;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WindowModeOpt {
    #[default]
    Windowed,
    /// borderless window covering the primary monitor (the old `fullscreen = true`)
    Borderless,
    /// exclusive fullscreen at the configured resolution
    Fullscreen,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RtQuality {
    Low,
    Medium,
    High,
    /// LOOK: High up to `AUTO_HIGH_MAX_PIXELS` render pixels, Medium above (R13: 5120x1440 RT High 14.7 ms,
    /// Medium 12.8 ms on an RTX 4080 SUPER after the baked-pixel skip + skin-refit LOD). Default.
    Auto,
}

/// LOOK: `RtQuality::Auto` switches from High to Medium above this many render pixels (R13: 5120x1440 =
/// 7.37 M holds 60 fps on High; 3840x2160 = 8.29 M and above get Medium).
pub const AUTO_HIGH_MAX_PIXELS: u32 = 5120 * 1440;

impl Default for RtQuality {
    fn default() -> Self {
        RtQuality::Auto
    }
}

impl RtQuality {
    pub const ALL: [RtQuality; 4] = [RtQuality::Low, RtQuality::Medium, RtQuality::High, RtQuality::Auto];
    /// LOOK: the concrete quality for a render size (Auto -> High / Medium by `AUTO_HIGH_MAX_PIXELS`).
    pub fn resolve(self, w: u32, h: u32) -> Self {
        match self {
            RtQuality::Auto if w.saturating_mul(h) > AUTO_HIGH_MAX_PIXELS => RtQuality::Medium,
            RtQuality::Auto => RtQuality::High,
            q => q,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            RtQuality::Low => "low",
            RtQuality::Medium => "medium",
            RtQuality::High => "high",
            RtQuality::Auto => "auto",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|q| q.name() == s)
    }
    /// direct-light sample multiplier over Solari's defaults (High = WIRE-C3's x4 denoise setting)
    /// PERF2: ReSTIR spatio-temporal reuse (Solari `restir`) only on High: at 5120x1440 it is ~11 of the
    /// 19 ms Solari lighting pass; with TAA the EP1 image without it differs by ~2/255.
    pub fn restir(self) -> bool {
        matches!(self, RtQuality::High | RtQuality::Auto)
    }
    pub fn di_sample_mul(self) -> u32 {
        match self {
            RtQuality::Low => 1,
            RtQuality::Medium => 2,
            RtQuality::High | RtQuality::Auto => 4,
        }
    }
    pub fn index(self) -> u8 {
        self as u8
    }
    pub fn from_index(i: u8) -> Self {
        Self::ALL[(i as usize).min(3)]
    }
}

impl WindowModeOpt {
    pub const ALL: [WindowModeOpt; 3] = [WindowModeOpt::Windowed, WindowModeOpt::Borderless, WindowModeOpt::Fullscreen];
    pub fn name(self) -> &'static str {
        match self {
            WindowModeOpt::Windowed => "windowed",
            WindowModeOpt::Borderless => "borderless",
            WindowModeOpt::Fullscreen => "fullscreen",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|q| q.name() == s)
    }
    pub fn from_index(i: u8) -> Self {
        Self::ALL[(i as usize).min(2)]
    }
    pub fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PortSettings {
    pub base: Settings,
    pub window_mode: WindowModeOpt,
    pub rt_quality: RtQuality,
    /// HUD spread over the whole width (true) or kept in the centred 16:9 safe area (false, default)
    pub hud_stretch: bool,
    /// UPSCALE: render scale / GI scale / sharpness / upscaler (gfx::upscale)
    pub upscale: crate::engine::gfx::upscale::UpscaleOpts,
}

impl std::ops::Deref for PortSettings {
    type Target = Settings;
    fn deref(&self) -> &Settings {
        &self.base
    }
}

impl std::ops::DerefMut for PortSettings {
    fn deref_mut(&mut self) -> &mut Settings {
        &mut self.base
    }
}

impl From<Settings> for PortSettings {
    fn from(base: Settings) -> Self {
        let window_mode = if base.fullscreen { WindowModeOpt::Borderless } else { WindowModeOpt::Windowed };
        PortSettings { base, window_mode, ..Default::default() }
    }
}

fn unquote(v: &str) -> Option<&str> {
    v.trim().strip_prefix('"')?.strip_suffix('"')
}

impl PortSettings {
    pub fn to_toml(&self) -> String {
        let mut base = self.base.clone();
        base.fullscreen = self.window_mode != WindowModeOpt::Windowed;
        format!(
            "{}\n[graphics]\n# \"windowed\" | \"borderless\" | \"fullscreen\" (overrides [video] fullscreen)\nwindow_mode = \"{}\"\n# ray tracing quality: \"auto\" (high up to 2560x1440, medium above) | \"low\" | \"medium\" | \"high\" (used when global_illumination = \"raytraced\")\nrt_quality = \"{}\"\n# \"centre\" = HUD in the centred 16:9 area (wide screens), \"stretch\" = HUD to the screen edges\nhud = \"{}\"\n{}",
            base.to_toml(),
            self.window_mode.name(),
            self.rt_quality.name(),
            if self.hud_stretch { "stretch" } else { "centre" },
            self.upscale.toml_lines()
        )
    }

    pub fn from_toml(text: &str) -> Result<Self, String> {
        let mut rest = String::new();
        let mut g: Vec<(String, String)> = vec![];
        let mut in_g = false;
        for l in text.lines() {
            let t = l.trim();
            if t.starts_with('[') && !t.contains('=') {
                in_g = t.trim_end_matches(|c: char| c != ']') == "[graphics]";
            }
            if in_g {
                let t = t.split('#').next().unwrap_or("").trim();
                if let Some((k, v)) = t.split_once('=') {
                    g.push((k.trim().to_string(), v.trim().to_string()));
                }
            } else {
                rest.push_str(l);
                rest.push('\n');
            }
        }
        let mut s = PortSettings::from(Settings::from_toml(&rest)?);
        let bad = |k: &str| format!("settings.toml: bad value for graphics.{k}");
        for (k, v) in &g {
            let v = unquote(v).ok_or(bad(k))?;
            match k.as_str() {
                "window_mode" => s.window_mode = WindowModeOpt::parse(v).ok_or(bad(k))?,
                "rt_quality" => s.rt_quality = RtQuality::parse(v).ok_or(bad(k))?,
                "hud" => {
                    s.hud_stretch = match v {
                        "stretch" => true,
                        "centre" | "center" => false,
                        _ => return Err(bad(k)),
                    }
                }
                _ => match s.upscale.parse_key(k, v) {
                    Some(r) => r.map_err(|_| bad(k))?,
                    None => log::warn!("settings.toml: unknown key graphics.{k}"),
                },
            }
        }
        s.base.fullscreen = s.window_mode != WindowModeOpt::Windowed;
        Ok(s)
    }

    /// Load `path`, or write defaults there when missing.
    pub fn load_or_create(path: &Path) -> io::Result<Self> {
        match std::fs::read_to_string(path) {
            Ok(t) => Self::from_toml(&t).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)),
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                let s = Self::default();
                if let Some(d) = path.parent() {
                    std::fs::create_dir_all(d)?;
                }
                std::fs::write(path, s.to_toml())?;
                Ok(s)
            }
            Err(e) => Err(e),
        }
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        std::fs::write(path, self.to_toml())
    }

    /// settings.toml to use: next to the exe if it exists or that folder is writable, else the user config dir.
    pub fn default_path() -> PathBuf {
        let local = Settings::default_path();
        if local.is_file() || dir_writable(local.parent().unwrap_or(Path::new("."))) {
            return local;
        }
        user_config_dir().map(|d| d.join("settings.toml")).unwrap_or(local)
    }
}

fn dir_writable(d: &Path) -> bool {
    let probe = d.join(".sly2_write_probe");
    let ok = std::fs::write(&probe, b"").is_ok();
    let _ = std::fs::remove_file(&probe);
    ok
}

pub fn user_config_dir() -> Option<PathBuf> {
    let base = std::env::var_os("APPDATA")
        .or_else(|| std::env::var_os("XDG_CONFIG_HOME"))
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
    // Renamed from "sly2-recomp" (Sly 2: Bevy of Thieves); keep using an existing old folder.
    let (new, old) = (base.join("bevy-of-thieves"), base.join("sly2-recomp"));
    Some(if !new.exists() && old.exists() { old } else { new })
}

#[cfg(feature = "game")]
impl WindowModeOpt {
    /// Bevy window mode (exclusive fullscreen keeps the desktop video mode; the render size is `[video]`).
    pub fn bevy(self) -> bevy::window::WindowMode {
        use bevy::window::{MonitorSelection, VideoModeSelection, WindowMode};
        match self {
            WindowModeOpt::Windowed => WindowMode::Windowed,
            WindowModeOpt::Borderless => WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
            WindowModeOpt::Fullscreen => WindowMode::Fullscreen(MonitorSelection::Primary, VideoModeSelection::Current),
        }
    }
}

/// Live port options read by render systems (gfx::ultrawide HUD layout, gfx::rt quality).
#[cfg(feature = "game")]
#[derive(bevy::prelude::Resource, Clone, Copy, Debug, Default, PartialEq)]
pub struct LiveGraphics {
    pub rt_quality: RtQuality,
    pub hud_stretch: bool,
    pub upscale: crate::engine::gfx::upscale::UpscaleOpts,
}
