//! Actuator (rumble) state machine at pad+0x10cc (1:1, FUN_001d5150..FUN_001d55b8). scePadSetActDirect
//! (FUN_00243870) is replaced by the `out([small, big])` callback (backend -> gilrs force feedback).

use super::pad::Clock;

/// One step of a rumble pattern: actuator bytes {small motor on/off, big motor 0..255} and its duration.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RumbleStep {
    pub act: [u8; 2],
    pub time: f32,
}

/// Rumble patterns, table 0x2c2260 (0x104-byte records {s32 n; n x {u8 act[2]; u8 pad[2]; f32 time}}),
/// 11 entries (index 11 at 0x2c2d0c has n = 0). Values read from SCUS_973.16.
pub const RUMBLE_PATTERNS: [&[RumbleStep]; 11] = {
    const fn s(small: u8, big: u8, time: f32) -> RumbleStep {
        RumbleStep { act: [small, big], time }
    }
    [
        &[s(1, 0, 0.05)],
        &[s(0, 150, 0.05), s(0, 0, 0.05)],
        &[s(0, 210, 0.05), s(0, 0, 0.1)],
        &[s(0, 255, 0.1), s(0, 0, 0.1)],
        &[s(0, 210, 0.1), s(0, 190, 0.01), s(0, 170, 0.01), s(0, 150, 0.01), s(0, 130, 0.01), s(0, 110, 0.01), s(0, 0, 0.25)],
        &[s(1, 255, 0.1), s(1, 0, 0.05)],
        &[s(0, 170, 0.05), s(0, 0, 0.1)],
        &[s(0, 170, 0.05), s(0, 0, 0.1)],
        &[s(0, 170, 0.05), s(0, 0, 0.1)],
        &[s(0, 160, 0.15), s(0, 0, 0.1), s(0, 90, 0.05)],
        &[s(0, 255, 0.2), s(0, 0, 0.08), s(0, 200, 0.13), s(0, 0, 0.06), s(0, 150, 0.11)],
    ]
};

/// pad+0x10cc, 0x1c bytes.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Rumble {
    /// +0x00
    pub port: i32,
    /// +0x04
    pub slot: i32,
    /// +0x08 0 = no actuators, 1 = idle, 2 = playing, 3 = stopping (-> 1), 4 = stopping (-> 0); -1 transient
    pub state: i32,
    /// +0x0c pattern (original: pointer into 0x2c2260; here the index, -1 = none)
    pub pattern: i32,
    /// +0x10 current step
    pub step: i32,
    /// +0x14 remaining total duration
    pub duration: f32,
    /// +0x18 step / stop timer
    pub timer: f32,
}

/// Globals read by the rumble code.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RumbleCtx {
    /// DAT_002c2d8c vibration option: 0 off, 2 on, 3 off, 1/other = ask the script variable
    pub vibration_mode: i32,
    /// value of the Splice variable read for mode 1 (`fNoVibration`, symbol DAT_003e0fd8)
    pub no_vibration: bool,
}

fn pattern(r: &Rumble) -> &'static [RumbleStep] {
    RUMBLE_PATTERNS.get(r.pattern as usize).copied().unwrap_or(&[])
}

/// FUN_001d5150: init on actuator detection.
pub fn FUN_001d5150(r: &mut Rumble, port: i32, slot: i32, clock: Clock, out: &mut dyn FnMut([u8; 2])) {
    if r.state == 0 {
        FUN_001d53c0(r, 1, clock, out);
    } else {
        FUN_001d53c0(r, 3, clock, out);
    }
    r.slot = slot;
    r.port = port;
}

/// FUN_001d5468: motors off.
pub fn FUN_001d5468(out: &mut dyn FnMut([u8; 2])) {
    out([0, 0]);
}

/// FUN_001d53c0: change state.
pub fn FUN_001d53c0(r: &mut Rumble, state: i32, clock: Clock, out: &mut dyn FnMut([u8; 2])) {
    if state == r.state {
        return;
    }
    match state {
        2 => {
            if let Some(s) = pattern(r).first() {
                out(s.act);
            }
        }
        3 | 4 => {
            FUN_001d5468(out);
            r.timer = clock.dt * 4.0; // keep sending "off" for 4 frames
        }
        _ => {}
    }
    r.state = state;
}

/// FUN_001d55b8: vibration allowed by the options.
pub fn FUN_001d55b8(ctx: &RumbleCtx) -> bool {
    match ctx.vibration_mode {
        0 | 3 => false,
        2 => true,
        _ => !ctx.no_vibration,
    }
}

/// FUN_001d5340: start a pattern (`duration` arrives in $f12 in the original).
pub fn FUN_001d5340(r: &mut Rumble, duration: f32, id: usize, ctx: &RumbleCtx, out: &mut dyn FnMut([u8; 2])) {
    let n = RUMBLE_PATTERNS.get(id).map_or(0, |p| p.len());
    if r.state != 0 && 0 < n && FUN_001d55b8(ctx) {
        r.duration = duration;
        r.pattern = id as i32;
        r.step = 0;
        r.state = -1; // forces the transition to 2 to resend step 0
        r.timer = RUMBLE_PATTERNS[id][0].time;
        // FUN_001d53c0(.., 2) only uses dt in states 3/4
        FUN_001d53c0(r, 2, Clock { t: 0.0, dt: 0.0 }, out);
    }
}

/// FUN_001d5310: pattern by index.
pub fn FUN_001d5310(r: &mut Rumble, id: usize, duration: f32, ctx: &RumbleCtx, out: &mut dyn FnMut([u8; 2])) {
    FUN_001d5340(r, duration, id, ctx, out);
}

/// FUN_001d51b8: per-frame update.
pub fn FUN_001d51b8(r: &mut Rumble, clock: Clock, _ctx: &RumbleCtx, out: &mut dyn FnMut([u8; 2])) {
    let dt = clock.dt;
    if r.state == 2 {
        if r.duration <= 0.0 {
            FUN_001d53c0(r, 3, clock, out);
        } else {
            r.timer -= dt;
            if r.timer <= 0.0 {
                let p = pattern(r);
                r.step += 1;
                if p.len() as i32 <= r.step {
                    r.step = 0; // loops until the duration runs out
                }
                out(p[r.step as usize].act);
                r.timer = p[r.step as usize].time;
            }
            r.duration -= dt;
        }
    } else if 1 < r.state && r.state < 5 {
        if 0.0 < r.timer {
            FUN_001d5468(out);
            r.timer -= dt;
        } else {
            let s = (r.state == 3) as i32;
            FUN_001d53c0(r, s, clock, out);
        }
    }
}
