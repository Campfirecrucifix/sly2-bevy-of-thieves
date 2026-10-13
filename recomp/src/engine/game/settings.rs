//! `settings.toml` next to the exe (port addition; no PS2 equivalent except the vibration/sound options).
//! Hand-written flat TOML subset (no toml crate in Cargo.toml): `[section]`, `key = value` with strings,
//! numbers, booleans and string arrays; `#` comments.

use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lighting {
    Raster,
    Raytraced,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    /// path of the disc image; empty = "Sly 2 - Band of Thieves (USA).iso" next to the exe
    pub iso: String,
    pub width: u32,
    pub height: u32,
    pub fullscreen: bool,
    pub vsync: bool,
    /// Hor+ for any aspect (incl. 32:9); false = exact PS2 framing
    pub hor_plus: bool,
    pub lighting: Lighting,
    /// draw the original's ink (cartoon outline) edges
    pub ink_outlines: bool,
    pub master_volume: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
    /// explicit mod load order (names under mods/); empty = mods/load_order.txt + alphabetical
    pub mods: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            iso: String::new(),
            width: 1920,
            height: 1080,
            fullscreen: false,
            vsync: true,
            hor_plus: true,
            lighting: Lighting::Raster,
            ink_outlines: true,
            master_volume: 1.0,
            music_volume: 1.0,
            sfx_volume: 1.0,
            mods: vec![],
        }
    }
}

fn quote(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

fn unquote(s: &str) -> Option<String> {
    let s = s.trim();
    let inner = s.strip_prefix('"')?.strip_suffix('"')?;
    let mut out = String::new();
    let mut esc = false;
    for c in inner.chars() {
        if esc {
            out.push(c);
            esc = false;
        } else if c == '\\' {
            esc = true;
        } else {
            out.push(c);
        }
    }
    Some(out)
}

fn strip_comment(l: &str) -> &str {
    let mut in_str = false;
    let mut prev = ' ';
    for (i, c) in l.char_indices() {
        if c == '"' && prev != '\\' {
            in_str = !in_str;
        } else if c == '#' && !in_str {
            return &l[..i];
        }
        prev = c;
    }
    l
}

fn split_array(s: &str) -> Option<Vec<String>> {
    let inner = s.trim().strip_prefix('[')?.strip_suffix(']')?;
    let mut out = vec![];
    let mut cur = String::new();
    let mut in_str = false;
    let mut prev = ' ';
    for c in inner.chars() {
        if c == '"' && prev != '\\' {
            in_str = !in_str;
        }
        if c == ',' && !in_str {
            if !cur.trim().is_empty() {
                out.push(unquote(&cur)?);
            }
            cur.clear();
        } else {
            cur.push(c);
        }
        prev = c;
    }
    if !cur.trim().is_empty() {
        out.push(unquote(&cur)?);
    }
    Some(out)
}

impl Settings {
    pub fn to_toml(&self) -> String {
        // WP-R: written as `global_illumination` ("off" | "raytraced"); the older `lighting` key is still read
        let l = match self.lighting {
            Lighting::Raster => "off",
            Lighting::Raytraced => "raytraced",
        };
        let mods: Vec<String> = self.mods.iter().map(|m| quote(m)).collect();
        let ink = self.ink_outlines;
        format!(
            "# Sly 2 recomp settings\niso = {}\n\n[video]\nwidth = {}\nheight = {}\nfullscreen = {}\nvsync = {}\nhor_plus = {}\n# \"off\" (raster) | \"raytraced\" (Bevy Solari; needs a build with feature `rt` and a ray-tracing GPU)\nglobal_illumination = \"{l}\"\nink_outlines = {ink}\n\n[audio]\nmaster = {}\nmusic = {}\nsfx = {}\n\n[mods]\n# load order (first = lowest priority); empty = mods/load_order.txt\norder = [{}]\n",
            quote(&self.iso),
            self.width,
            self.height,
            self.fullscreen,
            self.vsync,
            self.hor_plus,
            self.master_volume,
            self.music_volume,
            self.sfx_volume,
            mods.join(", ")
        )
    }

    pub fn from_toml(text: &str) -> Result<Self, String> {
        let mut kv = BTreeMap::new();
        let mut section = String::new();
        for (n, raw) in text.lines().enumerate() {
            let l = strip_comment(raw).trim();
            if l.is_empty() {
                continue;
            }
            if let Some(s) = l.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                section = s.trim().to_string();
                continue;
            }
            let (k, v) = l.split_once('=').ok_or(format!("settings.toml:{}: expected key = value", n + 1))?;
            let key = if section.is_empty() { k.trim().to_string() } else { format!("{section}.{}", k.trim()) };
            kv.insert(key, v.trim().to_string());
        }
        let mut s = Settings::default();
        let bad = |k: &str| format!("settings.toml: bad value for {k}");
        for (k, v) in &kv {
            let f = || v.parse::<f32>().map_err(|_| bad(k));
            let u = || v.parse::<u32>().map_err(|_| bad(k));
            let b = || v.parse::<bool>().map_err(|_| bad(k));
            match k.as_str() {
                "iso" => s.iso = unquote(v).ok_or(bad(k))?,
                "video.width" => s.width = u()?,
                "video.height" => s.height = u()?,
                "video.fullscreen" => s.fullscreen = b()?,
                "video.vsync" => s.vsync = b()?,
                "video.hor_plus" => s.hor_plus = b()?,
                "video.ink_outlines" => s.ink_outlines = b()?,
                "video.lighting" => {
                    s.lighting = match unquote(v).ok_or(bad(k))?.as_str() {
                        "raster" => Lighting::Raster,
                        "raytraced" => Lighting::Raytraced,
                        _ => return Err(bad(k)),
                    }
                }
                // WP-R: final-build option name; "off" = pure raster (always valid)
                "video.global_illumination" => {
                    s.lighting = match unquote(v).ok_or(bad(k))?.as_str() {
                        "off" => Lighting::Raster,
                        "raytraced" => Lighting::Raytraced,
                        _ => return Err(bad(k)),
                    }
                }
                "audio.master" => s.master_volume = f()?,
                "audio.music" => s.music_volume = f()?,
                "audio.sfx" => s.sfx_volume = f()?,
                "mods.order" => s.mods = split_array(v).ok_or(bad(k))?,
                _ => log::warn!("settings.toml: unknown key {k}"),
            }
        }
        Ok(s)
    }

    /// Load, or write defaults when missing.
    pub fn load_or_create(path: &Path) -> io::Result<Self> {
        match std::fs::read_to_string(path) {
            Ok(t) => Self::from_toml(&t).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)),
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                let s = Self::default();
                std::fs::write(path, s.to_toml())?;
                Ok(s)
            }
            Err(e) => Err(e),
        }
    }

    pub fn default_path() -> PathBuf {
        exe_dir().join("settings.toml")
    }

    /// Lighting actually used: raytraced only with feature `rt`, otherwise raster + log.
    pub fn effective_lighting(&self) -> Lighting {
        if self.lighting == Lighting::Raytraced && !cfg!(feature = "rt") {
            log::warn!("lighting = \"raytraced\" needs a build with feature `rt`; falling back to raster");
            return Lighting::Raster;
        }
        self.lighting
    }

    /// Mod overlay in the configured order (WP-A `ModOverlay`); unknown names are logged and skipped.
    pub fn mod_overlay(&self, mods_dir: &Path) -> io::Result<crate::engine::disc::mods::ModOverlay> {
        let mut o = crate::engine::disc::mods::ModOverlay::scan(mods_dir)?;
        if !self.mods.is_empty() {
            o.order.mods = self
                .mods
                .iter()
                .filter_map(|m| {
                    let p = mods_dir.join(m);
                    if p.is_dir() {
                        Some(p)
                    } else {
                        log::warn!("settings.toml: mod {m:?} not found in {}", mods_dir.display());
                        None
                    }
                })
                .collect();
        }
        Ok(o)
    }

    pub fn iso_path(&self) -> PathBuf {
        if self.iso.is_empty() { exe_dir().join("Sly 2 - Band of Thieves (USA).iso") } else { PathBuf::from(&self.iso) }
    }
}

pub fn exe_dir() -> PathBuf {
    std::env::current_exe().ok().and_then(|p| p.parent().map(Path::to_path_buf)).unwrap_or_default()
}
