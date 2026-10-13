//! HUD layout helper (port addition): a 16:9 safe rect centred in any window, and mapping of the original
//! 640x448 HUD coordinates into it. Elements keep their aspect (uniform scale = rect height / 448);
//! horizontal anchoring decides whether an element hugs the left edge, the centre or the right edge of
//! the safe rect (the 4:3 HUD is spread to the 16:9 corners instead of being stretched).

pub const HUD_W: f32 = 640.0;
pub const HUD_H: f32 = 448.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Anchor {
    Left,
    Center,
    Right,
    /// Stretch the full 640 width over the safe rect width (backgrounds / fades).
    Stretch,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HudLayout {
    pub window_w: f32,
    pub window_h: f32,
    pub safe: Rect,
    /// pixels per HUD unit (uniform)
    pub scale: f32,
}

impl HudLayout {
    pub fn new(window_w: f32, window_h: f32) -> Self {
        let a = 16.0 / 9.0;
        let (w, h) = if window_w / window_h >= a { (window_h * a, window_h) } else { (window_w, window_w / a) };
        let safe = Rect { x: (window_w - w) * 0.5, y: (window_h - h) * 0.5, w, h };
        HudLayout { window_w, window_h, safe, scale: h / HUD_H }
    }

    /// Anchor chosen from the original x (left third / centre / right third).
    pub fn auto_anchor(x: f32) -> Anchor {
        if x < HUD_W / 3.0 {
            Anchor::Left
        } else if x > HUD_W * 2.0 / 3.0 {
            Anchor::Right
        } else {
            Anchor::Center
        }
    }

    /// Map an original HUD point (640x448, y down) to window pixels (y down).
    pub fn map(&self, x: f32, y: f32, anchor: Anchor) -> (f32, f32) {
        let s = &self.safe;
        let py = s.y + y * self.scale;
        let px = match anchor {
            Anchor::Left => s.x + x * self.scale,
            Anchor::Right => s.x + s.w - (HUD_W - x) * self.scale,
            Anchor::Center => s.x + s.w * 0.5 + (x - HUD_W * 0.5) * self.scale,
            Anchor::Stretch => s.x + x / HUD_W * s.w,
        };
        (px, py)
    }

    /// Map an original HUD box (640x448 units, top-left `x`,`y`, size `w`x`h`) to window pixels. The
    /// box keeps its aspect (uniform scale); `anchor` is applied to the box's anchor-side edge so a
    /// left element hugs the safe rect's left edge, a right one its right edge (WP-R2).
    pub fn map_rect(&self, x: f32, y: f32, w: f32, h: f32, anchor: Anchor) -> Rect {
        let (sw, sh) = match anchor {
            Anchor::Stretch => (w / HUD_W * self.safe.w, h * self.scale),
            _ => (w * self.scale, h * self.scale),
        };
        let (px, py) = match anchor {
            Anchor::Right => {
                let (rx, ry) = self.map(x + w, y, anchor);
                (rx - sw, ry)
            }
            Anchor::Center => {
                let (cx, cy) = self.map(x + w * 0.5, y, anchor);
                (cx - sw * 0.5, cy)
            }
            _ => self.map(x, y, anchor),
        };
        Rect { x: px, y: py, w: sw, h: sh }
    }

    /// `r` (window pixels) lies inside the 16:9 safe rect (0.5 px tolerance).
    pub fn in_safe(&self, r: &Rect) -> bool {
        let s = &self.safe;
        r.x >= s.x - 0.5 && r.y >= s.y - 0.5 && r.x + r.w <= s.x + s.w + 0.5 && r.y + r.h <= s.y + s.h + 0.5
    }

    /// WP-I2: map a box of FUN_0015f480 text space (640 x 492.8, the 512x224 GS field) into window pixels
    /// through `map_rect` (text y scaled by 448 / 492.8 so glyphs keep the PS2 aspect).
    pub fn map_text_rect(&self, x0: f32, y0: f32, x1: f32, y1: f32, anchor: Anchor) -> Rect {
        let k = HUD_H / 492.80002;
        self.map_rect(x0, y0 * k, x1 - x0, (y1 - y0) * k, anchor)
    }

    /// Map a 640x492.8 projected point (FUN_001cd780 label space) into window pixels relative to the
    /// whole window (3D-anchored labels follow the world, not the safe rect).
    pub fn map_projected(&self, x: f32, y: f32) -> (f32, f32) {
        // FUN_001cd780 space covers the full view; with Hor+ the view spans the window.
        (x / 640.0 * self.window_w, y / 492.80002 * self.window_h)
    }
}

/// WP-R2: HUD element boxes of the original 640x448 layout used to verify/visualise the safe-rect
/// mapping (corners + centre, x, y, w, h). The PS2 HUD renderer itself is not ported to Bevy yet;
/// whatever draws HUD/menus must go through `HudLayout::map_rect` / `map` (gfx::ultrawide::HudSafeRect).
pub const HUD_PROBE_BOXES: [[f32; 4]; 5] = [
    [24.0, 24.0, 96.0, 48.0],   // top-left (health / gadget)
    [520.0, 24.0, 96.0, 48.0],  // top-right (coins)
    [24.0, 376.0, 96.0, 48.0],  // bottom-left
    [520.0, 376.0, 96.0, 48.0], // bottom-right
    [272.0, 200.0, 96.0, 48.0], // centre (prompts)
];
