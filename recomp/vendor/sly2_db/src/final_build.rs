//! Final-build requirements from `port_requirements`, as engine-facing data types.
//! Pure data + pure functions (no Bevy render dependency) so they compile in tools and tests;
//! with `--features bevy` they are Bevy `Resource`s.
//!
//! - graphics.raytracing: RT GI is a normal option, Off by default, raster fallback always valid.
//! - graphics.aspect: Hor+ from a fixed vertical FOV; 32:9 widens the view, never stretches.
//!   Bevy's `PerspectiveProjection::fov` is already vertical, so keeping `fov` constant and letting
//!   Bevy update `aspect_ratio` gives Hor+; [`hor_plus_hfov`] is for gameplay code that needs it.
//! - modding: mods/ folder, explicit load order, later mods override earlier by asset path.

#[cfg(feature = "bevy")]
use bevy::prelude::Resource;
use std::path::{Path, PathBuf};

/// Global illumination mode. `Off` = raster lighting (PS2-equivalent look, always available).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GlobalIllumination {
    #[default]
    Off,
    RayTracedLow,
    RayTracedHigh,
}

#[cfg_attr(feature = "bevy", derive(Resource))]
#[derive(Clone, Debug, PartialEq)]
pub struct GraphicsSettings {
    pub global_illumination: GlobalIllumination,
    /// Vertical FOV in radians, kept constant across aspect ratios (Hor+).
    pub vertical_fov: f32,
    /// HUD stays inside this aspect (e.g. 16:9) centred on screen; `None` = full width.
    pub hud_max_aspect: Option<f32>,
}

impl Default for GraphicsSettings {
    fn default() -> Self {
        // 16:9 baseline vertical FOV; at 32:9 the horizontal FOV widens (Hor+).
        Self { global_illumination: GlobalIllumination::Off, vertical_fov: 60f32.to_radians(), hud_max_aspect: Some(16.0 / 9.0) }
    }
}

impl GraphicsSettings {
    /// Fallback rule from graphics.raytracing: if the GPU lacks RT support, run raster GI.
    pub fn effective_gi(&self, rt_supported: bool) -> GlobalIllumination {
        if rt_supported { self.global_illumination } else { GlobalIllumination::Off }
    }
}

/// Horizontal FOV for a given vertical FOV and aspect (width / height). 32:9 => aspect 3.555...
pub fn hor_plus_hfov(vertical_fov: f32, aspect: f32) -> f32 {
    2.0 * ((vertical_fov * 0.5).tan() * aspect).atan()
}

/// HUD safe rect (x, y, w, h) in pixels: full height, width clamped to `max_aspect`, centred.
pub fn hud_safe_rect(width: f32, height: f32, max_aspect: Option<f32>) -> (f32, f32, f32, f32) {
    let w = match max_aspect { Some(a) => width.min(height * a), None => width };
    ((width - w) * 0.5, 0.0, w, height)
}

/// Mods in load order. `mods/load_order.txt` lists folder names (one per line, `#` comments);
/// folders not listed are appended alphabetically, except those named by a `# off: name` line (disabled). Later entries override earlier by asset path.
#[cfg_attr(feature = "bevy", derive(Resource))]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ModLoadOrder {
    pub mods: Vec<PathBuf>,
}

impl ModLoadOrder {
    /// Blocking scan: call from a tool or an async task, not from a Bevy system.
    pub fn scan(mods_dir: &Path) -> std::io::Result<Self> {
        if !mods_dir.is_dir() {
            return Ok(Self::default());
        }
        let mut found: Vec<String> = std::fs::read_dir(mods_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        found.sort();
        let text = std::fs::read_to_string(mods_dir.join("load_order.txt")).unwrap_or_default();
        let listed: Vec<String> = text
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(String::from)
            .collect();
        // `# off: name` lines (written by the Mods menu, ui::options::OFF_PREFIX) disable a folder:
        // it is not auto-appended. A plain listed line for the same name wins over the off line.
        let off: Vec<String> = text
            .lines()
            .filter_map(|l| l.trim().strip_prefix("# off:").map(|s| s.trim().to_string()))
            .collect();
        let mut order: Vec<String> = listed.into_iter().filter(|m| found.contains(m)).collect();
        for m in found {
            if !order.contains(&m) && !off.contains(&m) {
                order.push(m);
            }
        }
        Ok(Self { mods: order.into_iter().map(|m| mods_dir.join(m)).collect() })
    }

    /// Resolve an asset path: last mod providing it wins, else the base game path.
    pub fn resolve(&self, base_dir: &Path, rel: &Path) -> PathBuf {
        self.mods.iter().rev().map(|m| m.join(rel)).find(|p| p.exists()).unwrap_or_else(|| base_dir.join(rel))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hor_plus_widens_at_32_9() {
        let v = 60f32.to_radians();
        assert!(hor_plus_hfov(v, 32.0 / 9.0) > hor_plus_hfov(v, 16.0 / 9.0));
        let (x, _, w, _) = hud_safe_rect(5120.0, 1440.0, Some(16.0 / 9.0));
        assert_eq!((x, w), (1280.0, 2560.0));
    }
    #[test]
    fn mod_off_lines_disable() {
        let d = std::env::temp_dir().join(format!("sly2_db_mods_off_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        for m in ["a", "b", "c", "d"] {
            std::fs::create_dir_all(d.join(m)).unwrap();
        }
        // b listed, a disabled, c disabled but also listed (listed wins), d unlisted (appended).
        std::fs::write(d.join("load_order.txt"), "# comment
# off: a
b
# off: c
c
").unwrap();
        let o = ModLoadOrder::scan(&d).unwrap();
        let names: Vec<String> =
            o.mods.iter().map(|p| p.file_name().unwrap().to_string_lossy().into_owned()).collect();
        assert_eq!(names, ["b", "c", "d"]);
        std::fs::write(d.join("load_order.txt"), "# off: a
# off: b
# off: c
# off: d
").unwrap();
        assert!(ModLoadOrder::scan(&d).unwrap().mods.is_empty());
        let _ = std::fs::remove_dir_all(&d);
    }
    #[test]
    fn rt_falls_back() {
        let g = GraphicsSettings { global_illumination: GlobalIllumination::RayTracedHigh, ..Default::default() };
        assert_eq!(g.effective_gi(false), GlobalIllumination::Off);
    }
}
