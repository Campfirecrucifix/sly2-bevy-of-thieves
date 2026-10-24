//! UPSCALE: graphics options (pure data, settings.toml `[graphics] render_scale / gi_scale / sharpness / upscaler`).

/// Render-scale presets (fraction of the window size per axis).
pub const SCALES: [(&str, &str, f32); 4] = [("native", "Native", 1.0), ("quality", "Quality", 0.77), ("balanced", "Balanced", 0.67), ("performance", "Performance", 0.5)];
/// Upscaler backends offered in the menu (index = `UpscaleOpts::method`). DLSS/XeSS slot in here later.
pub const METHODS: [(&str, &str); 2] = [("spatial", "FSR1 (EASU+RCAS)"), ("temporal", "TAA + EASU")];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UpscaleOpts {
    /// index into `SCALES`
    pub scale: u8,
    /// RT GI frames: 0 = same as `scale`, else index+... into `SCALES` (1..=3 -> Quality..Performance) used while ray tracing
    pub gi_scale: u8,
    /// RCAS sharpness in tenths (0..=10)
    pub sharp: u8,
    /// index into `METHODS`
    pub method: u8,
}

impl Default for UpscaleOpts {
    fn default() -> Self {
        UpscaleOpts { scale: 0, gi_scale: 0, sharp: 5, method: 0 }
    }
}

fn find<T>(list: &[T], name: impl Fn(&T) -> &str, v: &str) -> Option<u8> {
    list.iter().position(|e| name(e) == v).map(|i| i as u8)
}

impl UpscaleOpts {
    /// render scale for this frame (ray tracing on -> `gi_scale` when set)
    pub fn factor(&self, raytraced: bool) -> f32 {
        let i = if raytraced && self.gi_scale > 0 { self.gi_scale } else { self.scale };
        SCALES[(i as usize).min(3)].2
    }
    pub fn sharpness(&self) -> f32 {
        self.sharp.min(10) as f32 / 10.0
    }
    pub fn temporal(&self) -> bool {
        self.method == 1
    }
    /// render-target size for a window of `w` x `h` physical pixels
    pub fn render_size(&self, w: u32, h: u32, raytraced: bool) -> (u32, u32) {
        let f = self.factor(raytraced);
        (((w as f32 * f).round() as u32).max(1), ((h as f32 * f).round() as u32).max(1))
    }
    pub fn gi_name(&self) -> &'static str {
        if self.gi_scale == 0 { "same" } else { SCALES[(self.gi_scale as usize).min(3)].0 }
    }
    pub fn toml_lines(&self) -> String {
        format!(
            "# render scale + upscale (UI stays native): \"native\" | \"quality\" (0.77) | \"balanced\" (0.67) | \"performance\" (0.5)\nrender_scale = \"{}\"\n# render scale while ray tracing: \"same\" or a render_scale preset\ngi_scale = \"{}\"\n# RCAS sharpening 0.0 .. 1.0\nsharpness = \"{:.1}\"\n# \"spatial\" (FSR1-style EASU+RCAS) | \"temporal\" (TAA at render res, then EASU+RCAS)\nupscaler = \"{}\"\n",
            SCALES[(self.scale as usize).min(3)].0,
            self.gi_name(),
            self.sharpness(),
            METHODS[(self.method as usize).min(1)].0
        )
    }
    /// one `[graphics]` key; None = not an upscale key, Some(Err) = bad value
    pub fn parse_key(&mut self, k: &str, v: &str) -> Option<Result<(), ()>> {
        let r = match k {
            "render_scale" => find(&SCALES, |s| s.0, v).map(|i| self.scale = i),
            "gi_scale" => if v == "same" { Some(0) } else { find(&SCALES, |s| s.0, v).filter(|&i| i > 0) }.map(|i| self.gi_scale = i),
            "sharpness" => v.parse::<f32>().ok().filter(|s| (0.0..=1.0).contains(s)).map(|s| self.sharp = (s * 10.0).round() as u8),
            "upscaler" => find(&METHODS, |m| m.0, v).map(|i| self.method = i),
            _ => return None,
        };
        Some(r.ok_or(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn upscale_opts_roundtrip_and_sizes() {
        let o = UpscaleOpts { scale: 2, gi_scale: 3, sharp: 8, method: 1 };
        let mut p = UpscaleOpts::default();
        for l in o.toml_lines().lines().filter(|l| !l.starts_with('#')) {
            let (k, v) = l.split_once(" = ").unwrap();
            assert_eq!(p.parse_key(k, v.trim_matches('"')), Some(Ok(())));
        }
        assert_eq!(p, o);
        assert_eq!(o.render_size(5120, 1440, false), (3430, 965));
        assert_eq!(o.render_size(5120, 1440, true), (2560, 720));
        assert_eq!(UpscaleOpts::default().render_size(1920, 1080, true), (1920, 1080));
        assert_eq!(p.parse_key("render_scale", "x"), Some(Err(())));
        assert_eq!(p.parse_key("foo", "x"), None);
    }
}
