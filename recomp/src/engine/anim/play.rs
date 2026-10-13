//! WP-D2: segment playback = ASEGA (an ASEG applied to an ALO; created by FUN_0012f9b8).
//! Fields: +0x14 ASEG, +0x18 target ALO, +0x1c local time, +0x20 event cursor time, +0x38
//! channel bindings, +0x3c flags (8 loop, 0x10 ping-pong, 0x38 = any "keep running" mode),
//! +0x40 speed (svt), +0x44 second speed (sign follows ping-pong), +0x50 next timed event.
//!
//! Per frame FUN_00130918 advances time, fires the timed events (ASEG list 1, +0xa0/+0xa4,
//! 0x10-byte records with the time at +0) crossed in the direction of play, wraps / clamps at
//! the segment ends (FUN_001317d8), then sets every binding's channel segment at the new time
//! (curve vtable slot 1). Event dispatch FUN_00131290 (sound, spawn, camera, scripts...) and
//! retract FUN_00130b18 belong to other subsystems: they are reached through [`AsegaHost`].

use super::aseg::{Aseg, Event};
use super::keys::Segment;
use std::sync::Arc;

/// +0x3c bit: wrap with fmod at the ends
pub const SEGA_LOOP: u32 = 0x8;
/// +0x3c bit: ping-pong between the ends
pub const SEGA_PINGPONG: u32 = 0x10;
/// +0x3c: no retract at the end when any of these is set
pub const SEGA_KEEP: u32 = 0x38;

/// ASEGA playback state (the fields FUN_00130918 and helpers touch).
#[derive(Clone, Debug, PartialEq)]
pub struct Asega {
    /// +0x1c
    pub t: f32,
    /// +0x20
    pub t_event: f32,
    /// +0x3c
    pub flags: u32,
    /// +0x40
    pub svt: f32,
    /// +0x44
    pub svt2: f32,
    /// +0x50
    pub next_event: usize,
    /// ASEG +0x40 bit 0x100: advance with DAT_002ddf10 (unscaled clock) instead of DAT_002dded4
    pub real_clock: bool,
    /// FUN_00131968 == 0 (still in the live ASEGA set 0x3e1218)
    pub alive: bool,
}

impl Default for Asega {
    fn default() -> Self {
        Asega { t: 0.0, t_event: 0.0, flags: 0, svt: 1.0, svt2: 1.0, next_event: 0, real_clock: false, alive: true }
    }
}

/// Engine side of the ASEGA callbacks.
pub trait AsegaHost {
    /// FUN_00131290(asega, event): +0x1c is already the event time. `reverse` = the event
    /// is in the timed list and the speed is negative (flips the on/off event pairs 7/8,
    /// 9/10, 0xd/0xe, 0xf/0x10).
    fn event(&mut self, _a: &mut Asega, _index: usize, _ev: &Event, _reverse: bool) {}
    /// vtable +0x24 message 3 on the owner (segment wrapped / reached its end)
    fn ended(&mut self, _a: &mut Asega) {}
    /// FUN_00130b18: retract (finished, no loop mode)
    fn retract(&mut self, a: &mut Asega) {
        a.alive = false;
    }
}
/// Host that ignores every callback except retract.
pub struct NoHost;
impl AsegaHost for NoHost {}

fn ev_time(aseg: &Aseg, i: usize) -> f32 {
    aseg.events1[i].time.unwrap_or(0.0)
}

fn fire(a: &mut Asega, aseg: &Aseg, i: usize, host: &mut dyn AsegaHost) {
    // 0x00131290: +0x1c = event time; reverse flag only for timed events when svt < 0
    a.t = ev_time(aseg, i);
    let reverse = a.svt < 0.0;
    host.event(a, i, &aseg.events1[i], reverse);
}

/// FUN_00130eb0: position the event cursor for the current time and direction.
pub fn ee_sub_130eb0(a: &mut Asega, aseg: &Aseg) {
    // 0x00130eb0
    let n = aseg.events1.len();
    if n == 0 {
        return;
    }
    if 0.0 <= a.svt {
        let mut i = 0;
        if ev_time(aseg, 0) < a.t {
            i = 1;
            while i < n && ev_time(aseg, i) < a.t {
                i += 1;
            }
        }
        a.next_event = i;
        return;
    }
    let mut i = n;
    loop {
        let j = i as isize - 1;
        if j < 0 || !(a.t < ev_time(aseg, j as usize)) {
            break;
        }
        i -= 1;
    }
    a.next_event = i;
}

/// FUN_00130f88: fire every timed event between the cursor and +0x1c (forward events at
/// t >= time unless exactly equal while reversing; backward the mirror). +0x1c ends at the
/// target time, or at the last fired event's time if a callback killed the ASEGA.
pub fn ee_sub_130f88(a: &mut Asega, aseg: &Aseg, host: &mut dyn AsegaHost) {
    // 0x00130f88
    let n = aseg.events1.len();
    if n == 0 {
        return;
    }
    a.t_event = a.t;
    while a.alive {
        let i = a.next_event;
        let ft = a.t_event;
        let stay = i >= n || {
            let e = ev_time(aseg, i);
            ft <= e && (ft != e || a.svt <= 0.0)
        };
        if !stay {
            a.next_event = i + 1;
            fire(a, aseg, i, host);
            continue;
        }
        if i < 1 {
            a.t = ft;
            break;
        }
        let p = ev_time(aseg, i - 1);
        if p <= ft {
            if ft != p || 0.0 <= a.svt {
                a.t = ft;
                break;
            }
        }
        a.next_event = i - 1;
        fire(a, aseg, i - 1, host);
    }
}

/// Event kinds FUN_00131100 re-fires when seeking (state-like events).
pub const SEEK_KINDS: [u8; 11] = [7, 8, 0xb, 0xd, 0xe, 0xf, 0x10, 0x15, 0x16, 0x17, 0x18];

/// FUN_00131100: like FUN_00130f88 (forward on `<`, backward only with svt < 0) but fires
/// only [`SEEK_KINDS`] events.
pub fn ee_sub_131100(a: &mut Asega, aseg: &Aseg, host: &mut dyn AsegaHost) {
    // 0x00131100
    let n = aseg.events1.len();
    if n == 0 {
        return;
    }
    a.t_event = a.t;
    while a.alive {
        let i = a.next_event;
        let ft = a.t_event;
        let ev;
        if i >= n || ft < ev_time(aseg, i) || a.svt < 0.0 {
            if 0 < i {
                if ev_time(aseg, i - 1) < ft {
                    a.t = ft;
                    break;
                }
                if a.svt < 0.0 {
                    a.next_event = i - 1;
                    ev = i - 1;
                } else {
                    a.t = ft;
                    break;
                }
            } else {
                a.t = ft;
                break;
            }
        } else {
            a.next_event = i + 1;
            ev = i;
        }
        if SEEK_KINDS.contains(&aseg.events1[ev].kind) {
            fire(a, aseg, ev, host);
        }
    }
}

/// FUN_001317d8(asega, &t, &svt): end handling. Returns true when the segment wrapped /
/// ended in the direction of play. Loop: fmod (negative times wrapped by FUN_0011d898).
/// Ping-pong: fold into [0, 2L), speeds made positive then negated on the way back.
/// Clamp: t = end, svt kept only if it points back inside (else 0).
pub fn ee_sub_1317d8(a: &mut Asega, length: f32) -> bool {
    // 0x001317d8
    let t = a.t;
    let l = length;
    let r;
    if l <= t {
        r = 0.0 < a.svt;
        if a.flags & SEGA_LOOP != 0 {
            a.t = t % l; // FUN_00245a18 = fmodf
            return r;
        }
        if a.flags & SEGA_PINGPONG == 0 {
            a.t = l;
            // movz: svt kept if negative, else 0
            a.svt = if a.svt < 0.0 { a.svt } else { 0.0 };
            return r;
        }
        pingpong(a, t, l);
        return r;
    }
    if 0.0 <= t {
        return false;
    }
    r = a.svt < 0.0;
    if a.flags & SEGA_LOOP != 0 {
        // FUN_0011d898: fmodf, + L when negative
        let mut f = t % l;
        if f < 0.0 {
            f += l;
        }
        a.t = f;
        return r;
    }
    if a.flags & SEGA_PINGPONG != 0 {
        pingpong(a, -t, l);
        return r;
    }
    a.t = 0.0;
    a.svt = if 0.0 < a.svt { a.svt } else { 0.0 };
    r
}

fn pingpong(a: &mut Asega, t: f32, l: f32) {
    // LAB_001318d4
    let l2 = l + l;
    a.t = t % l2;
    a.svt = a.svt.abs();
    a.svt2 = a.svt2.abs();
    if a.t < l {
        return;
    }
    a.t = l2 - a.t;
    a.svt = -a.svt;
    a.svt2 = -a.svt2;
}

/// FUN_00130918: per-frame ASEGA update. `dt` = DAT_002dded4 (scaled frame time), `real_dt` =
/// DAT_002ddf10. Returns true when the bindings must be set up at `a.t` (the original then
/// calls each binding curve's slot 1 with (t, svt)); false when |svt| < 1e-4 or the ASEGA died.
pub fn ee_sub_130918(a: &mut Asega, aseg: &Aseg, dt: f32, real_dt: f32, host: &mut dyn AsegaHost) -> bool {
    // 0x00130918
    if a.svt.abs() < 0.0001 {
        return false;
    }
    a.t += a.svt * if a.real_clock { real_dt } else { dt };
    ee_sub_130f88(a, aseg, host);
    if !a.alive {
        return false;
    }
    let wrapped = ee_sub_1317d8(a, aseg.length);
    if wrapped {
        if a.flags & SEGA_LOOP != 0 {
            a.next_event = if a.svt <= 0.0 { aseg.events1.len() } else { 0 };
        }
        host.ended(a);
        if !a.alive {
            return false;
        }
    }
    ee_sub_130f88(a, aseg, host);
    if !a.alive {
        return false;
    }
    if wrapped && a.flags & SEGA_KEEP == 0 {
        host.retract(a);
    }
    // the original walks the binding list (+0x38) even after a retract
    true
}

/// Binding setup (slot 1 of each channel curve) at time `t`: one segment per ASEG channel.
pub fn bind_segments(aseg: &Aseg, t: f32) -> Vec<Option<Segment>> {
    aseg.channels.iter().map(|c| c.vec.as_ref().and_then(|v| v.segment(t))).collect()
}

/// Frame clock: DAT_002dded4 (game dt) and DAT_002ddf10 (unscaled dt).
#[derive(bevy_ecs::prelude::Resource, Clone, Copy, Debug, Default)]
pub struct AnimClock {
    pub dt: f32,
    pub real_dt: f32,
}

/// An ASEGA as an ECS component: playback state + current channel segments.
#[derive(bevy_ecs::prelude::Component, Clone, Debug)]
pub struct AsegaPlayer {
    pub aseg: Arc<Aseg>,
    pub state: Asega,
    pub segments: Vec<Option<Segment>>,
    /// timed events fired during the last update: (index, reverse)
    pub fired: Vec<(usize, bool)>,
    pub ended: bool,
}

impl AsegaPlayer {
    pub fn new(aseg: Arc<Aseg>, flags: u32, svt: f32) -> Self {
        let state = Asega { flags, svt, svt2: svt, ..Default::default() };
        let segments = bind_segments(&aseg, 0.0);
        AsegaPlayer { aseg, state, segments, fired: Vec::new(), ended: false }
    }
}

struct Collect<'a>(&'a mut Vec<(usize, bool)>, bool);
impl AsegaHost for Collect<'_> {
    fn event(&mut self, _a: &mut Asega, i: usize, _ev: &Event, rev: bool) {
        self.0.push((i, rev));
    }
    fn ended(&mut self, _a: &mut Asega) {
        self.1 = true;
    }
}

/// Per-frame driver step for one player (FUN_00130918 + binding setup).
pub fn drive(p: &mut AsegaPlayer, clock: AnimClock) {
    p.fired.clear();
    let mut host = Collect(&mut p.fired, false);
    if ee_sub_130918(&mut p.state, &p.aseg, clock.dt, clock.real_dt, &mut host) {
        p.ended |= host.1;
        p.segments = bind_segments(&p.aseg, p.state.t);
    } else {
        p.ended |= host.1;
    }
}

/// System: advance every live [`AsegaPlayer`].
pub fn drive_asegas(clock: bevy_ecs::prelude::Res<AnimClock>, mut q: bevy_ecs::prelude::Query<&mut AsegaPlayer>) {
    for mut p in &mut q {
        if p.state.alive {
            drive(&mut p, *clock);
        }
    }
}
