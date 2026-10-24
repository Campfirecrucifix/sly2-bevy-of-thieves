//! CRAWL: Sly's crawl (under tables / through vents).
//!
//! Original (EMU4 snapshots 010..019, help text "Press O near the table to crawl under it."): O near a
//! class-0x1b object -> FUN_001938c0 kind 8 -> FUN_001928f8(8) -> jts 7 attach kind 8 (p5_attach subs
//! 7/9/10/8); the crawl body velocity comes from FUN_001a0450 and the walk state under the table is jts 0xc.
//! REPLACED (no class-0x1b target list / attach data reaches the plugin yet, FUN_001a0450 unported): O while
//! grounded with a low ceiling above / ahead of Sly enters crawl; while crawling the body is the low sphere
//! CRAWL_R (centre lowered onto the floor) instead of the JT_RADIUS sphere; crawl ends as soon as the
//! standing sphere fits again (the original leaves the attach at the end of the crawl surface).
use super::{Jt, JT_RADIUS};
use crate::engine::input::pad::BTN_CIRCLE;
use crate::engine::physics::CollisionWorld;

/// Crawl body radius (clearance under the cellar table is 20..55).
pub const CRAWL_R: f32 = 9.0;
/// Look-ahead for the low ceiling that makes O start a crawl (FUN_001938c0 pick range stand-in).
const CRAWL_REACH: f32 = 150.0;

#[derive(Clone, Copy, Debug, Default)]
pub struct Crawl {
    pub on: bool,
}

/// Ceiling over (x, y) between z0 and z0 + h?
fn ceiling(cw: &CollisionWorld, x: f32, y: f32, z0: f32, h: f32) -> bool {
    cw.clip_segment([x, y, z0], [x, y, z0 + h]).iter().any(|l| l.t[0] > 0.0)
}

/// Low ceiling (crawl height) within CRAWL_REACH ahead along `yaw` over the floor at `floor`?
fn low_ahead(cw: &CollisionWorld, p: [f32; 3], floor: f32, yaw: f32) -> bool {
    let (dx, dy) = (yaw.cos(), yaw.sin());
    (0..=6).any(|k| {
        let d = k as f32 * CRAWL_REACH / 6.0;
        ceiling(cw, p[0] + dx * d, p[1] + dy * d, floor + 2.0 * CRAWL_R + 1.0, 2.0 * JT_RADIUS - 2.0 * CRAWL_R)
    })
}

impl Jt {
    /// Body radius for the physics step.
    pub fn body_radius(&self) -> f32 {
        if self.crawl.on { CRAWL_R } else { JT_RADIUS }
    }

    /// Per-frame crawl enter / exit (before the physics step). `pressed` = pad buttons pressed this frame.
    pub fn crawl_update(&mut self, cw: &CollisionWorld, pressed: u32) {
        let p = self.phys.pos;
        if self.crawl.on {
            let floor = p[2] - CRAWL_R;
            let clear = (-2..=2).all(|i| (-2..=2).all(|j| {
                !ceiling(cw, p[0] + i as f32 * 15.0, p[1] + j as f32 * 15.0, floor + 1.0, 2.0 * JT_RADIUS + 10.0)
            }));
            // CRAWL R124: the original stays attached to the class-0x1b crawl surface up to its end, so a crawl
            // started in front of a vent mouth is kept while the low ceiling is still ahead (opt-out SLY2_NO_R124_CRAWLKEEP)
            let ahead = std::env::var_os("SLY2_NO_R124_CRAWLKEEP").is_none() && low_ahead(cw, p, floor, self.yaw);
            if clear && !ahead {
                self.crawl.on = false;
                self.phys.pos[2] = floor + JT_RADIUS;
            }
            return;
        }
        if pressed & BTN_CIRCLE == 0 || !self.grounded || !(matches!(self.jts, 0 | 1) || (self.jts == 0xc && std::env::var_os("SLY2_NO_R124_CRAWLWALK").is_none())) {
            // CRAWL R124: 0x18a8c8 handles stand / run / walk (0xc) in one group, so O picks the crawl target from walk too

            return;
        }
        let floor = p[2] - JT_RADIUS;
        if low_ahead(cw, p, floor, self.yaw) {
            self.crawl.on = true;
            self.phys.pos[2] = floor + CRAWL_R;
        }
    }
}
