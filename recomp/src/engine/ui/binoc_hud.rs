#![allow(non_snake_case)]
//! WP-A12: binocucom HUD, the jt+0xf00 binocucom's +0x24 sub-object (player/p15_binoc.rs records the calls).
//!
//! 1:1: FUN_0019d540 (screens per character: Sly 0x35f / 0x368, Murray 0x360 / 0x369, Bentley 0x361 /
//! 0x36a), FUN_0019dca0 (object screen +0x10 = arg or the default +0xc), FUN_0019d660 (state: 0 open,
//! 2 close, -1 hidden, 1 shown; game messages 7 / 8 (own look) or 0xc / 0xd (mode 1, through an object) to
//! 0x2deb30), FUN_0019da78 (per frame while looking: 0 -> 1 once the screen is fully open, 2 -> -1 once it
//! is closed; element 0x353 = target marker while jt+0x10ec == 4 with DAT_002bf920 = jt+0x10e4; element
//! 0x352 = compass (jt+0x1a8 + pi) / 2pi, element 0x351 = zoom (binocucom +0x3c)).
//! REPLACED: the screens are level screen objects (sprite timelines, unported); their open / close
//! (screen +0x58: 2 = open, < 2 = closed) completes at once; `ui::hud::binoc` draws a compass strip, a zoom
//! bar and the target marker. RECORDED: the 0x2deb30 messages (`msgs`), FUN_00227788(0x4435f0), the
//! FUN_00202768 sound handle volumes, FUN_0019d948 job / task ids (+0xcc / +0xd0). The zoom +0x3c
//! (DAT_002de3b0 / DAT_002de50c binocucom camera) is not kept by the port binocucom: 0.
use crate::engine::player::p4::FUN_00132b10;

/// The jt / binocucom values the sub-object reads.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BinocIn {
    /// JT class (FUN_00132b10 0x35 / 0x37 / 0x36)
    pub cls: i32,
    /// binocucom +0x20 mode (1 = looking through an object)
    pub mode: i32,
    /// jt+0x1a8
    pub yaw: f32,
    /// binocucom +0x3c
    pub zoom: f32,
    /// jt+0x10ec / jt+0x10e4
    pub f_10ec: i32,
    pub f_10e4: i32,
}

/// Sub-object binocucom +0x24.
#[derive(Clone, Debug, PartialEq)]
pub struct BinocHud {
    /// +0x28 (param_1[1]) state
    pub state: i32,
    /// +0x2c / +0x30 / +0x34 screen oids (0 = none)
    pub screen: u32,
    pub screen_default: u32,
    pub screen_obj: u32,
    /// DAT_002bf920
    pub marker_kind: i32,
    /// port view of the screen: elements 0x352 / 0x351 shown + scrub times, 0x353 marker shown
    pub shown: bool,
    pub compass: f32,
    pub zoom: f32,
    pub marker: bool,
    /// RECORDED FUN_001619a0(0x2deb30, msg, 0)
    pub msgs: Vec<i32>,
}

impl Default for BinocHud {
    fn default() -> Self {
        Self { state: -1, screen: 0, screen_default: 0, screen_obj: 0, marker_kind: -1, shown: false, compass: 0.0, zoom: 0.0, marker: false, msgs: Vec::new() }
    }
}

impl BinocHud {
    /// 0x0019d540 (+ 0x19dca0(arg))
    pub fn FUN_0019d540(&mut self, cls: i32, arg: u32) {
        let pick = if FUN_00132b10(cls, 0x35) {
            Some((0x35f, 0x368))
        } else if FUN_00132b10(cls, 0x37) {
            Some((0x360, 0x369))
        } else if FUN_00132b10(cls, 0x36) {
            Some((0x361, 0x36a))
        } else {
            None
        };
        if let Some((a, b)) = pick {
            self.screen = a;
            self.screen_default = b;
        }
        self.FUN_0019dca0(arg);
    }

    /// 0x0019dca0
    pub fn FUN_0019dca0(&mut self, arg: u32) {
        self.screen_obj = if arg == 0 { self.screen_default } else { arg };
    }

    fn scrub(&mut self, b: &BinocIn) {
        self.compass = (b.yaw + std::f32::consts::PI) * 0.15915494;
        self.zoom = b.zoom;
    }

    /// 0x0019d660
    pub fn FUN_0019d660(&mut self, b: &BinocIn, s: i32) {
        if self.state == s {
            return;
        }
        let prev = self.state;
        self.state = s;
        match s {
            0 => {
                self.msgs.push(if b.mode == 1 { 0xc } else { 7 });
                if self.screen != 0 {
                    self.shown = true;
                    self.scrub(b);
                }
            }
            2 => self.msgs.push(if b.mode == 1 { 0xd } else { 8 }),
            -1 => {
                if prev != 2 {
                    self.msgs.push(if b.mode == 1 { 0xd } else { 8 });
                }
                // FUN_00212a70 / FUN_00212a50(screen, 1): screen off
                self.shown = false;
                self.marker = false;
            }
            _ => {}
        }
    }

    /// 0x0019da78
    pub fn FUN_0019da78(&mut self, b: &BinocIn) {
        // REPLACED: screen +0x58 reaches 2 (open) / < 2 (closed) at once
        match self.state {
            0 => {
                self.marker_kind = -1;
                self.FUN_0019d660(b, 1);
            }
            -1 => return,
            2 => {
                self.FUN_0019d660(b, -1);
                return;
            }
            _ => {}
        }
        if self.screen == 0 {
            return;
        }
        if self.state != 0 {
            let mut done = false;
            if b.f_10ec == 4 {
                if self.marker_kind != 4 {
                    self.marker = true; // FUN_00213b00(screen, 0x353)
                }
                self.marker_kind = b.f_10e4;
                done = true;
            }
            if !done {
                if self.marker_kind == 4 {
                    self.marker = false; // FUN_00213928(screen, 0x353)
                }
                self.marker_kind = b.f_10e4;
            }
        }
        self.scrub(b);
    }
}
