//! WP-D3: ASEGA event effects = FUN_00131290 dispatch, 1:1 by event kind. Each case calls an
//! engine function owned by another subsystem; here the call is produced as an [`EffectCall`]
//! (callee address kept in the variant doc) and handed to an [`EffectSink`], which the game
//! layer implements (audio::emitter, spawn, camera, splice...). Cases that only touch the
//! ASEGA itself are executed here (6 retract FUN_00130b18). `on` = the kind's on/off parity
//! XOR the reverse flag (pairs 7/8, 9/10, 0xd/0xe, 0xf/0x10), exactly as the original.

use super::aseg::{Event, EventData};
use super::play::{Asega, AsegaHost};

#[derive(Clone, Debug, PartialEq)]
pub enum EffectCall {
    /// 0: FUN_00210740(world, block) — emit using the K0 block (s16, 3 x s32, 3 x f32)
    K0 { s: i16, w: [i32; 3], f: [f32; 3] },
    /// 1: FUN_002108e8(world, s16, aseg, asega+0x5c)
    E1(i16),
    /// 2: FUN_0014f750(0x2ddf40, s16)
    E2(i16),
    /// 3: FUN_00187e28(+0xc time, 0x2dfc00, s16)
    E3 { a: i16, t: Option<f32> },
    /// 4: FUN_0015e938(world, 4, oid): class 0x6d (ASEG) -> FUN_0012f9b8 play it at svt 1;
    /// class 0x72 -> FUN_001dfd58(obj, 0); else nothing
    Spawn(i32),
    /// 5: FUN_00130d30(+8, +0xc, asega, 0)
    Seek { t: f32, f: f32 },
    /// 7/8: FUN_00210338(+0xc, world, on, +8, ...)
    E7 { on: bool, a: i32, f: f32 },
    /// 9/10: FUN_00131990(asega, on)
    E9 { on: bool },
    /// 0xb: owner message 9 {asega, time, payload}
    Msg9 { t: f32, v: i32 },
    /// 0xc: owner vtable +0x1c(4, 0, 1, &payload, 0)
    Owner1c(i32),
    /// 0xd/0xe: FUN_00210408(world, on, +8, +0xc, ...)
    Ed { on: bool, a: i32, b: i32 },
    /// 0xf/0x10: FUN_002104a0(world, on, +8, +0xc, ...)
    Ef { on: bool, a: i32, b: i32 },
    /// 0x11: FUN_00210538(block)
    E11([i32; 3], f32),
    /// 0x12: FUN_001618f8(+0x1c time, world)
    E12 { t: f32 },
    /// 0x13: FUN_00161958(world)
    E13,
    /// 0x14: FUN_00210678(block)
    E14(i32, i32, f32),
    /// 0x15: FUN_00230cf0(asega, s16)
    E15(i16),
    /// 0x16 / 0x17: FUN_001f81e0(target, s32 / -1)
    E16(i32),
    /// 0x18: FUN_00143090(target, 3 x 8-byte blocks) when the target is class 0x34
    E18([[u8; 8]; 3]),
}

/// Receiver of the effect calls (game layer).
pub trait EffectSink {
    fn call(&mut self, a: &mut Asega, c: EffectCall);
}

/// FUN_00131290: map one event to its effect. Kind 6 retracts directly (returns None).
pub fn ee_sub_131290(a: &mut Asega, ev: &Event, reverse: bool) -> Option<EffectCall> {
    // 0x00131290 (+0x1c = event time is set by the caller, play::fire)
    let k = ev.kind;
    let on = |first: u8| (k == first) ^ reverse;
    use EventData as D;
    Some(match (k, &ev.data) {
        (0, D::K0 { s, w, f }) => EffectCall::K0 { s: *s, w: *w, f: *f },
        (1, D::S16 { a, .. }) => EffectCall::E1(*a),
        (2, D::S16 { a, .. }) => EffectCall::E2(*a),
        (3, D::S16 { a, time2 }) => EffectCall::E3 { a: *a, t: *time2 },
        (4, D::S32(o)) => EffectCall::Spawn(*o),
        (5, D::TimeF { t, f }) => EffectCall::Seek { t: *t, f: *f },
        (6, _) => {
            a.alive = false; // FUN_00130b18
            return None;
        }
        (7 | 8, D::S32F(x, f)) => EffectCall::E7 { on: on(7), a: *x, f: *f },
        (9 | 10, _) => EffectCall::E9 { on: on(9) },
        (0xb, D::S32(v)) => EffectCall::Msg9 { t: a.t, v: *v },
        (0xc, D::S32(v)) => EffectCall::Owner1c(*v),
        (0xd | 0xe, D::S32x2(x, y)) => EffectCall::Ed { on: on(0xd), a: *x, b: *y },
        (0xf | 0x10, D::S32x2(x, y)) => EffectCall::Ef { on: on(0xf), a: *x, b: *y },
        (0x11, D::K17(w, f)) => EffectCall::E11(*w, *f),
        (0x12, _) => EffectCall::E12 { t: a.t },
        (0x13, _) => EffectCall::E13,
        (0x14, D::K20(x, y, f)) => EffectCall::E14(*x, *y, *f),
        (0x15, D::S16 { a: v, .. }) => EffectCall::E15(*v),
        (0x16, D::S32(v)) => EffectCall::E16(*v),
        (0x17, _) => EffectCall::E16(-1),
        (0x18, D::K24(b)) => EffectCall::E18(*b),
        _ => return None, // unknown kinds: no case in the switch
    })
}

/// AsegaHost that dispatches through FUN_00131290 into an [`EffectSink`].
pub struct EffectHost<'a>(pub &'a mut dyn EffectSink);
impl AsegaHost for EffectHost<'_> {
    fn event(&mut self, a: &mut Asega, _i: usize, ev: &Event, reverse: bool) {
        if let Some(c) = ee_sub_131290(a, ev, reverse) {
            self.0.call(a, c);
        }
    }
}
