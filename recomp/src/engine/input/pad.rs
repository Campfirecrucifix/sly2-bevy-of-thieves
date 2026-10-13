//! The game-side pad object (1:1 port of the EE pad code around 0x00187000).
//!
//! The original object lives at 0x002dfc00 (`DAT_002dfc00`, the only pad). Its first 0x1000 bytes are the
//! libpad DMA buffer (replaced, not kept); [`PadState`] mirrors the game fields at +0x1000..+0x10f0 field by
//! field (`PAD_BASE` + offset_of == original offset, asserted below).
//! Globals passed in explicitly instead of read from memory: `DAT_002ddf0c` (game time, `Clock::t`),
//! `DAT_002ddf10` (frame delta, `Clock::dt`).

use super::rumble::{self, Rumble, RumbleCtx};

/// Offset of [`PadState`] inside the original pad object (fields before it are the libpad DMA buffer).
pub const PAD_BASE: usize = 0x1000;

// Button bits as the game sees them: `(byte2 << 8 | byte3) ^ 0xffff` of the libpad report (0x00187998).
pub const BTN_L2: u32 = 0x0001;
pub const BTN_R2: u32 = 0x0002;
pub const BTN_L1: u32 = 0x0004;
pub const BTN_R1: u32 = 0x0008;
pub const BTN_TRIANGLE: u32 = 0x0010;
pub const BTN_CIRCLE: u32 = 0x0020;
pub const BTN_CROSS: u32 = 0x0040;
pub const BTN_SQUARE: u32 = 0x0080;
pub const BTN_SELECT: u32 = 0x0100;
pub const BTN_L3: u32 = 0x0200;
pub const BTN_R3: u32 = 0x0400;
pub const BTN_START: u32 = 0x0800;
pub const BTN_UP: u32 = 0x1000;
pub const BTN_RIGHT: u32 = 0x2000;
pub const BTN_DOWN: u32 = 0x4000;
pub const BTN_LEFT: u32 = 0x8000;

/// Game time globals (`DAT_002ddf0c` = t, `DAT_002ddf10` = dt).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Clock {
    pub t: f32,
    pub dt: f32,
}

/// One analog stick block (left at +0x103c, right at +0x1070; 0x34 bytes each).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stick {
    /// +0x00 x in [-1,1] (right positive)
    pub x: f32,
    /// +0x04 y in [-1,1] (up positive)
    pub y: f32,
    /// +0x08 magnitude after the 0.35 dead zone, [0,1]
    pub mag: f32,
    /// +0x0c/+0x0d last raw bytes
    pub raw_prev: [u8; 2],
    pub _pad: [u8; 2],
    /// +0x10 stick has been "activated" (moved or outside the centre box)
    pub active: i32,
    /// +0x14 auto-calibrated {min,max} of the projection on each of the 4 axes of `DAT_002be950`
    pub range: [[f32; 2]; 4],
}

impl Default for Stick {
    fn default() -> Self {
        Stick { x: 0.0, y: 0.0, mag: 0.0, raw_prev: [0; 2], _pad: [0; 2], active: 0, range: [[0.0; 2]; 4] }
    }
}

/// Pad state as the engine reads it; same shape/order as the original +0x1000..+0x10f0.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct PadState {
    /// +0x1000 libpad port
    pub port: i32,
    /// +0x1004 libpad slot
    pub slot: i32,
    /// +0x1008 terminal/mode id byte of the last report (byte 1); change = pad swapped
    pub term_id: u32,
    /// +0x100c connection state (1 = searching, 2 = configuring, 3 = ready)
    pub state: i32,
    /// +0x1010 pad kind (0 none, 1 digital, 2 analog, 3 DualShock (actuators), 4 DualShock2 pressure mode)
    pub kind: i32,
    /// +0x1014 time of the last state change
    pub t_state: f32,
    /// +0x1018 time of the last successful read (0 = never)
    pub t_read: f32,
    /// +0x101c time of the last activity
    pub t_active: f32,
    /// +0x1020 whole seconds without input
    pub idle_secs: i32,
    /// +0x1024 / +0x1028 menu auto-repeat direction (x: -1 left / +1 right, y: -1 up / +1 down)
    pub repeat_x: i32,
    pub repeat_y: i32,
    /// +0x102c / +0x1030 next x repeat time / current x repeat interval
    pub t_repeat_x: f32,
    pub dt_repeat_x: f32,
    /// +0x1034 / +0x1038 same for y
    pub t_repeat_y: f32,
    pub dt_repeat_y: f32,
    /// +0x103c left stick
    pub left: Stick,
    /// +0x1070 right stick
    pub right: Stick,
    /// +0x10a4 per-second velocity of lx, ly, rx, ry
    pub vel: [f32; 4],
    /// +0x10b4 buttons held
    pub buttons: u32,
    /// +0x10b8 buttons pressed this read and not yet consumed
    pub pressed: u32,
    /// +0x10bc consumed/locked buttons: no new press until released
    pub latch: u32,
    /// +0x10c0 pressure bytes (kind 4 only): right,left,up,down,tri,circle,cross,square,L1,R1,L2,R2
    pub pressure: [u8; 12],
    /// +0x10cc actuator state machine
    pub rumble: Rumble,
    /// +0x10e8 rumble enabled
    pub rumble_enabled: i32,
    /// +0x10ec stick value produced by the d-pad
    pub dpad_mag: f32,
}

const _: () = {
    use core::mem::offset_of;
    assert!(PAD_BASE + offset_of!(PadState, kind) == 0x1010);
    assert!(PAD_BASE + offset_of!(PadState, repeat_x) == 0x1024);
    assert!(PAD_BASE + offset_of!(PadState, left) == 0x103c);
    assert!(PAD_BASE + offset_of!(PadState, left) + offset_of!(Stick, active) == 0x104c);
    assert!(PAD_BASE + offset_of!(PadState, left) + offset_of!(Stick, range) == 0x1050);
    assert!(PAD_BASE + offset_of!(PadState, right) == 0x1070);
    assert!(PAD_BASE + offset_of!(PadState, vel) == 0x10a4);
    assert!(PAD_BASE + offset_of!(PadState, buttons) == 0x10b4);
    assert!(PAD_BASE + offset_of!(PadState, pressure) == 0x10c0);
    assert!(PAD_BASE + offset_of!(PadState, rumble) == 0x10cc);
    assert!(PAD_BASE + offset_of!(PadState, rumble_enabled) == 0x10e8);
    assert!(core::mem::size_of::<PadState>() == 0xf0);
};

/// The parts of a libpad 0x40-byte report the game reads (scePadRead replaced by the backend).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RawPad {
    /// byte 1: terminal/mode id (0x73 analog, 0x79 DualShock2)
    pub mode_id: u8,
    /// bytes 2..3 as `byte2 << 8 | byte3`, active low (0xffff = nothing held)
    pub buttons_wire: u16,
    /// bytes 4,5,6,7: rx, ry, lx, ly (0x80 centre, y down)
    pub rx: u8,
    pub ry: u8,
    pub lx: u8,
    pub ly: u8,
    /// bytes 8..19
    pub pressure: [u8; 12],
}

impl Default for RawPad {
    fn default() -> Self {
        RawPad { mode_id: 0x79, buttons_wire: 0xffff, rx: 0x80, ry: 0x80, lx: 0x80, ly: 0x80, pressure: [0; 12] }
    }
}

/// `DAT_002be950`: the 4 axes the stick is projected on for calibration.
pub const STICK_AXES: [[f32; 2]; 4] = [[1.0, 0.0], [0.0, 1.0], [0.7, 0.7], [0.7, -0.7]];

impl PadState {
    pub fn new() -> Self {
        PadState::default()
    }
}

/// FUN_00187150: open the pad (scePadPortOpen replaced). Original passes 0x2dfc00, port 0, slot 0.
pub fn FUN_00187150(pad: &mut PadState, port: i32, slot: i32, clock: Clock, out: &mut dyn FnMut([u8; 2])) {
    FUN_001875b8(pad, 1, 0, clock, out);
    pad.port = port;
    pad.slot = slot;
    pad.rumble_enabled = 1;
    pad.dpad_mag = 1.0;
}

/// FUN_001875b8: set connection state / pad kind, resetting what the transition needs.
pub fn FUN_001875b8(pad: &mut PadState, mut state: i32, kind: i32, clock: Clock, out: &mut dyn FnMut([u8; 2])) {
    if state == pad.state && kind == pad.kind {
        return;
    }
    if kind == 0 {
        pad.term_id = 0;
        state = 1;
    }
    if pad.kind < 3 && kind > 2 {
        // gained actuators
        rumble::FUN_001d5150(&mut pad.rumble, pad.port, pad.slot, clock, out);
    }
    if (pad.kind == 4) != (kind == 4) {
        pad.pressure = [0; 12];
    }
    if (pad.state == 3) != (state == 3) {
        pad.t_read = 0.0;
        pad.right.active = 0;
        pad.left.active = 0;
        pad.pressed = 0;
        pad.buttons = 0;
        pad.latch = 0xffff; // everything held at connect must be released first
        pad.pressure = [0; 12];
        pad.repeat_y = 0;
        pad.vel = [0.0; 4];
        pad.left.mag = 0.0;
        pad.left.y = 0.0;
        pad.left.x = 0.0;
        pad.right.mag = 0.0;
        pad.right.y = 0.0;
        pad.right.x = 0.0;
        pad.repeat_x = 0;
        for i in 0..4 {
            pad.right.range[i] = [-0.75, 0.75];
            pad.left.range[i] = [-0.75, 0.75];
        }
    }
    pad.state = state;
    pad.kind = kind;
    pad.t_state = clock.t;
}

/// FUN_00187768: raw stick bytes -> x, y, magnitude (dead zone 0.35, self-calibrating range).
pub fn FUN_00187768(t_read: f32, x_raw: u8, y_raw: u8, stick: &mut Stick) {
    let (mut fx, mut fy) = (0.0f32, 0.0f32);
    let mut convert = true;
    if stick.active == 0 {
        let mut active = 0;
        if t_read != 0.0 {
            // activates when the raw value changed or lies outside the [0x60,0xa0] centre box
            if x_raw != stick.raw_prev[0] || y_raw != stick.raw_prev[1] {
                active = 1;
            } else if x_raw < 0x60 || x_raw > 0xa0 || y_raw < 0x60 || y_raw > 0xa0 {
                active = 1;
            }
        }
        stick.active = active;
        convert = active != 0;
    }
    if convert {
        fx = (x_raw as f32 + x_raw as f32) * 0.003921569 - 1.0;
        fy = 1.0 - (y_raw as f32 + y_raw as f32) * 0.003921569;
    }
    let mut m = 0.0f32;
    for (axis, range) in STICK_AXES.iter().zip(stick.range.iter_mut()) {
        let p = fx * axis[0] + fy * axis[1];
        let bound = if 0.0 <= p {
            if range[1] < p {
                range[1] = p;
            }
            range[1]
        } else {
            if p < range[0] {
                range[0] = p;
            }
            range[0]
        };
        if m < p / bound {
            m = p / bound;
        }
    }
    let d = (m - 0.35) + (m - 0.35);
    let mut mag = 0.0;
    if 0.0 <= d {
        mag = d;
        if 1.0 < d {
            mag = 1.0;
        }
    }
    stick.mag = mag;
    let len = (fx * fx + fy * fy).sqrt();
    if 0.0001 < len {
        stick.x = (mag * fx) / len;
        stick.y = (mag * fy) / len;
    } else {
        stick.y = 0.0;
        stick.x = 0.0;
    }
    stick.raw_prev = [x_raw, y_raw];
}

/// Engine context of a read that the original takes from globals.
pub struct ReadCtx<'a> {
    /// `DAT_002de24c != 0 && DAT_002de254 == &DAT_002de2e0`: d-pad does not drive the left stick.
    pub dpad_override_suppressed: bool,
    /// cheat-code matcher FUN_0014bf48 (0x2ddec0)
    pub cheats: Option<&'a mut CheatDetector>,
    /// `DAT_002deb30`: time of the last "player active" message
    pub t_activity_msg: &'a mut f32,
}

/// What FUN_00187998 returns plus its side effect message.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReadResult {
    /// 0 = pad lost / swapped
    pub ok: u32,
    /// FUN_001619a0(0x2deb30, 0xe, ...) was sent (activity after >15 s)
    pub activity_msg: bool,
}

/// FUN_00187998: process one report. `raw == None` = scePadRead returned no data.
pub fn FUN_00187998(pad: &mut PadState, raw: Option<&RawPad>, clock: Clock, ctx: ReadCtx) -> ReadResult {
    let mut ok = 1;
    let Some(raw) = raw else {
        // tolerate 2 s of failed reads
        let ok = if clock.t - pad.t_read < 2.0 { 1 } else { 0 };
        return ReadResult { ok, activity_msg: false };
    };
    let prev = pad.buttons;
    let cur = raw.buttons_wire as u32 ^ 0xffff;
    pad.pressed = cur & !prev & !pad.latch;
    pad.latch &= cur;
    pad.buttons = cur;
    if pad.term_id != 0 && pad.term_id != raw.mode_id as u32 {
        ok = 0;
    }
    pad.term_id = raw.mode_id as u32;
    if pad.kind == 4 {
        pad.pressure = raw.pressure;
    }
    let (olx, oly, orx, ory) = (pad.left.x, pad.left.y, pad.right.x, pad.right.y);
    if pad.kind < 2 {
        pad.right.x = 0.0;
        pad.left.mag = 0.0;
        pad.left.y = 0.0;
        pad.left.x = 0.0;
        pad.right.mag = 0.0;
        pad.right.y = 0.0;
    } else {
        let t_read = pad.t_read;
        FUN_00187768(t_read, raw.lx, raw.ly, &mut pad.left);
        FUN_00187768(t_read, raw.rx, raw.ry, &mut pad.right);
    }
    if !ctx.dpad_override_suppressed {
        let b = pad.buttons;
        if b & 0xf000 != 0 {
            pad.left.x = 0.0;
            pad.left.mag = 0.0;
            pad.left.y = 0.0;
            if b & BTN_LEFT != 0 {
                pad.left.x = -pad.dpad_mag;
            } else if b & BTN_RIGHT != 0 {
                pad.left.x = pad.dpad_mag;
            }
            if b & BTN_UP != 0 {
                pad.left.y = pad.dpad_mag;
            } else if b & BTN_DOWN != 0 {
                pad.left.y = -pad.dpad_mag;
            }
            if pad.left.x * pad.left.y != 0.0 {
                pad.left.x *= 0.70710677;
                pad.left.y *= 0.70710677;
            }
            pad.left.mag = 1.0;
        }
    }
    pad.vel[0] = (pad.left.x - olx) / clock.dt;
    pad.vel[1] = (pad.left.y - oly) / clock.dt;
    pad.vel[2] = (pad.right.x - orx) / clock.dt;
    pad.vel[3] = (pad.right.y - ory) / clock.dt;
    if let Some(c) = ctx.cheats {
        FUN_0014bf48(c, true, pad.buttons, prev & 0xffff, clock);
    }
    pad.t_read = clock.t;
    let mut activity_msg = false;
    if pad.buttons == 0 && pad.left.mag == 0.0 && pad.right.mag == 0.0 {
        pad.idle_secs = (clock.t - pad.t_active) as i32;
    } else {
        pad.idle_secs = 0;
        pad.t_active = clock.t;
        if 15.0 < clock.t - *ctx.t_activity_msg {
            activity_msg = true;
            *ctx.t_activity_msg = clock.t;
        }
    }
    ReadResult { ok, activity_msg }
}

/// FUN_001871c0 replacement: per-frame update. The libpad connection negotiation (scePadGetState,
/// scePadInfoMode, scePadSetMainMode, scePadSetActAlign, scePadEnterPressMode) is replaced: a connected
/// backend device is a DualShock2 in pressure mode (state 3, kind 4). The ready-state branch is 1:1.
pub fn FUN_001871c0(
    pad: &mut PadState,
    connected: bool,
    raw: Option<&RawPad>,
    clock: Clock,
    ctx: ReadCtx,
    rctx: &RumbleCtx,
    out: &mut dyn FnMut([u8; 2]),
) -> ReadResult {
    let none = ReadResult { ok: 1, activity_msg: false };
    if pad.state == 0 {
        return none; // never opened
    }
    if !connected {
        if pad.kind != 0 {
            FUN_001875b8(pad, 1, 0, clock, out);
        }
        return none;
    }
    if pad.state != 3 {
        FUN_001875b8(pad, 3, 4, clock, out);
    }
    let r = FUN_00187998(pad, raw, clock, ctx);
    if r.ok == 0 {
        FUN_001875b8(pad, 1, 0, clock, out);
    } else if pad.rumble_enabled != 0 && 2 < pad.kind {
        rumble::FUN_001d51b8(&mut pad.rumble, clock, rctx, out);
    }
    r
}

/// FUN_00187dd0: pressed & mask.
pub fn FUN_00187dd0(pad: &PadState, mask: u32) -> u32 {
    pad.pressed & mask
}

/// FUN_00187de0: consume presses (held ones stay latched until released).
pub fn FUN_00187de0(pad: &mut PadState, mask: u32) {
    pad.pressed &= !mask;
    pad.latch |= pad.buttons & mask;
}

/// FUN_00187d80: test-and-consume a press.
pub fn FUN_00187d80(pad: &mut PadState, mask: u32) -> u32 {
    let r = FUN_00187dd0(pad, mask);
    FUN_00187de0(pad, mask);
    r
}

/// FUN_00187e08: forget held buttons.
pub fn FUN_00187e08(pad: &mut PadState, mask: u32) {
    pad.buttons &= !mask;
    pad.latch &= !mask;
}

/// FUN_00187e78: reset menu auto-repeat.
pub fn FUN_00187e78(pad: &mut PadState) {
    pad.repeat_y = 0;
    pad.repeat_x = 0;
}

/// 0x3eaaa64c (0.3333): first repeat delay / max interval.
const REPEAT_MAX: f32 = f32::from_bits(0x3eaaa64c);

/// FUN_00187e88: horizontal menu navigation with auto-repeat (-1 left, +1 right, 0 none).
pub fn FUN_00187e88(pad: &mut PadState, clock: Clock) -> i32 {
    let interval;
    if pad.repeat_x == 0 {
        let p = FUN_00187d80(pad, BTN_LEFT | BTN_RIGHT);
        if p & BTN_LEFT != 0 {
            pad.repeat_x = -1;
        } else if p & BTN_RIGHT != 0 {
            pad.repeat_x = 1;
        } else if 0.8 <= pad.left.mag && pad.left.y.abs() < pad.left.x.abs() {
            pad.repeat_x = if 0.0 <= pad.left.x { 1 } else { -1 };
        }
        if pad.repeat_x == 0 {
            return 0;
        }
        interval = 0.3333;
        pad.dt_repeat_x = REPEAT_MAX;
    } else {
        let held = if pad.repeat_x < 0 {
            pad.buttons & BTN_LEFT != 0
                || (0.25 <= pad.left.mag && pad.left.y.abs() < pad.left.x.abs() && pad.left.x < 0.0)
        } else {
            pad.buttons & BTN_RIGHT != 0
                || (0.25 <= pad.left.mag && pad.left.y.abs() < pad.left.x.abs() && 0.0 < pad.left.x)
        };
        if !held {
            pad.repeat_x = 0;
            return 0;
        }
        if clock.t < pad.t_repeat_x {
            return 0;
        }
        // interval shrinks by 2/3 each repeat, clamped to [0.1, 0.3333]
        let mut f = 0.1;
        let n = (pad.dt_repeat_x + pad.dt_repeat_x) * 0.33333334;
        if 0.1 <= n {
            f = n;
            if 0.3333 < n {
                f = 0.3333;
            }
        }
        pad.dt_repeat_x = f;
        interval = f;
    }
    pad.t_repeat_x = clock.t + interval;
    pad.repeat_x
}

/// FUN_001880d8: vertical menu navigation with auto-repeat (-1 up, +1 down, 0 none).
pub fn FUN_001880d8(pad: &mut PadState, clock: Clock) -> i32 {
    let interval;
    if pad.repeat_y == 0 {
        let p = FUN_00187d80(pad, BTN_UP | BTN_DOWN);
        if p & BTN_UP != 0 {
            pad.repeat_y = -1;
        } else if p & BTN_DOWN != 0 {
            pad.repeat_y = 1;
        } else if 0.8 <= pad.left.mag && pad.left.x.abs() < pad.left.y.abs() {
            pad.repeat_y = if pad.left.y <= 0.0 { 1 } else { -1 };
        }
        if pad.repeat_y == 0 {
            return 0;
        }
        interval = 0.3333;
        pad.dt_repeat_y = REPEAT_MAX;
    } else {
        let held = if pad.repeat_y < 0 {
            pad.buttons & BTN_UP != 0
                || (0.25 <= pad.left.mag && pad.left.x.abs() < pad.left.y.abs() && 0.0 < pad.left.y)
        } else {
            pad.buttons & BTN_DOWN != 0
                || (0.25 <= pad.left.mag && pad.left.x.abs() < pad.left.y.abs() && pad.left.y < 0.0)
        };
        if !held {
            pad.repeat_y = 0;
            return 0;
        }
        if clock.t < pad.t_repeat_y {
            return 0;
        }
        let mut f = 0.1;
        let n = (pad.dt_repeat_y + pad.dt_repeat_y) * 0.33333334;
        if 0.1 <= n {
            f = n;
            if 0.3333 < n {
                f = 0.3333;
            }
        }
        pad.dt_repeat_y = f;
        interval = f;
    }
    pad.t_repeat_y = clock.t + interval;
    pad.repeat_y
}

/// FUN_00187e28: start rumble pattern `id` (table 0x2c2260) for `duration` seconds.
pub fn FUN_00187e28(pad: &mut PadState, id: usize, duration: f32, rctx: &RumbleCtx, out: &mut dyn FnMut([u8; 2])) {
    if pad.rumble_enabled != 0 && 2 < pad.kind && pad.state == 3 {
        rumble::FUN_001d5310(&mut pad.rumble, id, duration, rctx, out);
    }
}

/// Input lock (`DAT_002dfbc4` flags -> `DAT_002dfbc0` mode), FUN_00187078/FUN_001870a0/FUN_001870d0.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InputLock {
    /// DAT_002dfbc4
    pub flags: u32,
    /// DAT_002dfbc0
    pub mode: u32,
}

/// FUN_001870d0
pub fn FUN_001870d0(lock: &mut InputLock, pad: &mut PadState) {
    let f = lock.flags;
    lock.mode = if f & 0x10 != 0 {
        5
    } else if f & 0xc != 0 {
        0
    } else if f & 1 != 0 {
        5
    } else if f & 2 != 0 {
        4
    } else {
        7
    };
    if lock.mode & 2 == 0 {
        pad.latch = 0xffff; // DAT_002e0cbc: require release of everything
    }
}

/// FUN_00187078
pub fn FUN_00187078(lock: &mut InputLock, pad: &mut PadState, bits: u32) {
    lock.flags |= bits;
    FUN_001870d0(lock, pad);
}

/// FUN_001870a0
pub fn FUN_001870a0(lock: &mut InputLock, pad: &mut PadState, bits: u32) {
    lock.flags &= !bits;
    FUN_001870d0(lock, pad);
}

/// One button-combo sequence (0x38-byte records at 0x2bd4e0.., list at 0x2ddec0).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CheatSeq {
    pub buttons: Vec<u32>,
    /// +0x10 matched prefix length
    pub idx: usize,
}

/// Cheat matcher state (0x2ddec0).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CheatDetector {
    pub seqs: Vec<CheatSeq>,
    /// DAT_003da0b8: matching enabled
    pub enabled: bool,
    /// +0x0c: (int)(t + 1)
    pub deadline: i32,
}

/// FUN_0014bea8: register a sequence (prefix starts at 1 if its first combo is held right now).
pub fn FUN_0014bea8(det: &mut CheatDetector, buttons: Vec<u32>, cur_buttons: u32) {
    let idx = if buttons.first() == Some(&cur_buttons) { 1 } else { 0 };
    det.seqs.push(CheatSeq { buttons, idx });
}

/// FUN_0014bf48: advance every sequence on a new non-empty button combination (exact mask match).
pub fn FUN_0014bf48(det: &mut CheatDetector, is_main_pad: bool, cur: u32, prev: u32, clock: Clock) {
    if is_main_pad && prev & 0xffff != cur && cur != 0 && det.enabled {
        for s in det.seqs.iter_mut() {
            if s.idx < s.buttons.len() {
                s.idx = if s.buttons[s.idx] == cur { s.idx + 1 } else { 0 };
            }
        }
        det.deadline = (clock.t + 1.0) as i32;
    }
}
