//! Guard brain goal (class 0x9c, vtable 0x2d04a0, size 0xb0): substate machine FUN_00177618 / setstate
//! FUN_00176750 / re-plan FUN_00177ba0, and the guard's slot 87 target pick FUN_002082d0.
use super::patrol::{Cmd, Cx, Routine, Script};
use super::search::{ee_sub_20a408, SGuard, Search};
use super::sensor::Sensor;
use super::slots::Stimulus;
use super::lift::C2d0420;

/// FUN_00176750 child goal of the current substate (brain +0x18).
#[derive(Clone, Debug, Default)]
pub enum Child {
    #[default]
    None,
    /// sub 1: 0xaa routine (behaviour bit 0x1e)
    Routine(Box<Routine>),
    /// sub 5: 0xa9 script with +0x4c = 1 (bit 0x21)
    Script(Box<Script>),
    /// sub 2: 0xab search (K22, behaviour bit 0x1f)
    Search(Box<Search>),
    /// sub 3: 0x9d reaction anim (+0x3c = 0xb, +0x38 = target; bit 0xb): start time, anim length
    React(f32, f32),
    /// sub 4: 0xa5 chase / attack (bit 0x1d) - UNPORTED goal (the guard stands, facing the target)
    Chase,
    /// sub 0x11: 0x9e wait
    Wait,
    /// K24: sub 6: 0xb0 lifted goal (`Brain::b0`, guard/lift.rs; bit 0x16)
    Lifted,
    /// K32: subs 7 / 8 / 9 / 10 / 0xb / 0x13: hit-reaction goals (0xb1 / 0xaf / 0xa1 / 0xa2 / 0xa3 / 0xbf)
    Hit(HitGoal),
}

/// K32: hit-reaction child (anim goal base C2d09b0: state +0x30 is 1 once the anim of behaviour bit `bit`
/// (FUN_00209bc0: the anim of the same id in the +0x4c0 table) played `len`; no anim -> done at once).
/// 9 = 0xa1 stagger (bit 5), 10 = 0xa2 heavy stagger (bit 6), 0xb = 0xa3 knocked out (bit 7);
/// 7 (0xb1), 8 (0xaf, FUN_0017a5b0 timer +0x58) and 0x13 (0xbf) are UNPORTED goals: no anim, done at once
/// (8 after the +0xe7c minimum 16 s), 0xb1 ends with +0x50 = 3.
#[derive(Clone, Debug, Default)]
pub struct HitGoal {
    pub code: i32,
    /// behaviour bit (+0x3c), -1 = none
    pub bit: i32,
    pub t0: f32,
    pub len: f32,
    /// +0x50.. copy of the hit (+0x5c kind gates later hits by the +0x350 / +0x358 masks)
    pub hit: super::slots::GuardHit,
}

impl HitGoal {
    /// FUN_00174fb8: state == 1
    pub fn done(&self, now: f32) -> bool {
        now - self.t0 >= self.len
    }
}

/// Substate record (brain +0x30, 0x70 bytes): code, target / stimulus (+0x34), routine (+0x10 of sub 5)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Sub {
    pub code: i32,
    pub obj: u32,
    pub routine: Option<usize>,
}

#[derive(Clone, Debug, Default)]
pub struct Brain {
    /// +0x30 substate
    pub sub: Sub,
    /// +0xa0 substate start, +0xa4 wait-until (sub 0x11)
    pub a0: f32,
    pub a4: f32,
    pub child: Child,
    /// K24: +0x40 (substate +0x10: the sub-6 message argument, FUN_00209e28)
    pub f_40: i32,
    /// K24: the 0xb0 goal of the guard's +0x26c stack (pushed by sub 6; popped = `active` false)
    pub b0: Option<C2d0420>,
    /// K32: +0x40.. hit copy of the substate record (FUN_00176be8 subs 9 / 10 / 0xb)
    pub hit: super::slots::GuardHit,
}

/// What the brain needs from the guard.
pub struct Mind<'a> {
    pub sensor: &'a Sensor,
    /// guard position / per-entry positions (None = unknown object)
    pub pos: [f32; 3],
    pub pos_of: &'a dyn Fn(u32) -> Option<[f32; 3]>,
    /// is-a 0x38 (guard) of an object
    pub is_guard: &'a dyn Fn(u32) -> bool,
    /// guard fields: +0x38c target, +0x390 forced target, +0xfbc mode, +0xfa8 / +0xf94, is-a 0x3b
    pub f_38c: u32,
    pub f_390: u32,
    pub f_fbc: i32,
    pub fa8_f94: bool,
    pub c3b: bool,
    /// +0x2e8 / +0x2ec routines, +0x2dc cursor, +0x2cc..+0x2d0 timer
    pub routine: Option<Vec<Cmd>>,
    pub cursor: Option<usize>,
    pub t30: f32,
    /// level routines (sub 5 `Sub::routine` index)
    pub routines: &'a [Vec<Cmd>],
    /// K22: perceived stimuli (DAT_003e148c list through gate FUN_00208d70 + slot 88; sub 2 `Sub::obj` = index + 1)
    pub heard: &'a [Stimulus],
    /// K22: guard fields the 0xab search goal reads
    pub sg: SGuard,
}

/// slot 87 FUN_002082d0: pick the target among the sensor entries (ignore / priority / +0x394 class lists empty)
pub fn ee_sub_2082d0(m: &Mind) -> u32 {
    if m.f_390 != 0 {
        return m.f_390;
    }
    let mut best = 0;
    let mut bc = f32::MAX; // DAT_002a886c
    let mode_guards = m.f_fbc == 0x22 || m.f_fbc == 0x25;
    for e in &m.sensor.entries {
        if e.obj == 0 {
            continue;
        }
        if e.state == 0 && !(mode_guards && (m.is_guard)(e.obj)) {
            continue;
        }
        let Some(p) = (m.pos_of)(e.obj) else { continue };
        let d = [p[0] - m.pos[0], p[1] - m.pos[1], p[2] - m.pos[2]];
        let mut c = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        if e.obj == m.f_38c {
            c = -400.0 + c * (0.8 + c * 0.0); // DAT_002c3d70/74/78
        }
        if mode_guards && (m.is_guard)(e.obj) {
            c = -500.0 + c * (0.8 + c * 0.0); // DAT_002c3d80/84/88
        }
        if c < bc {
            bc = c;
            best = e.obj;
        }
    }
    best
}

impl Brain {
    /// FUN_00176750(brain, sub): pop the current child, store the substate, push / start the new child
    pub fn ee_sub_176750(&mut self, s: Sub, m: &Mind, cx: &mut Cx) -> Option<usize> {
        if s == self.sub {
            return None;
        }
        // leaving 0xaa saves the script cursor (FUN_00171c18 -> guard +0x2dc)
        let saved = match &self.child {
            Child::Routine(r) => r.cursor(),
            _ => None,
        };
        // K24: popping the 0xb0 goal runs its slot 8 (FUN_0017a7b8)
        if matches!(self.child, Child::Lifted) {
            if let Some(b) = self.b0.as_mut() {
                b.ee_sub_17a7b8(cx.now);
            }
        }
        self.child = Child::None;
        cx.body.f_190 = [0.0; 3];
        self.a0 = cx.now;
        self.child = match s.code {
            1 => Child::Routine(Box::new(Routine::ee_sub_179aa0(m.routine.clone(), m.cursor, m.t30, cx))),
            // FUN_00205438(g, 0xab) + FUN_001725e0(goal, stimulus), pushed (slot 7 FUN_00171fa0)
            2 => {
                let mut se = Search::ee_sub_171f58();
                se.ee_sub_1725e0((s.obj as usize).checked_sub(1).and_then(|k| m.heard.get(k)).cloned());
                se.ee_sub_171fa0(&m.sg, cx);
                Child::Search(Box::new(se))
            }
            3 => Child::React(cx.now, cx.h.anim_len(cx.guard, 0xb)),
            4 => Child::Chase,
            5 => match s.routine.and_then(|k| m.routines.get(k)) {
                Some(c) => Child::Script(Box::new(Script::ee_sub_171258(c.clone(), true, None, cx))),
                None => Child::None,
            },
            // K24: FUN_00205438(g, 0xb0) +0x3c = brain +0x40, pushed (slot 7 FUN_0017a730)
            6 => {
                let b = self.b0.insert(C2d0420::new());
                b.f_3c = self.f_40;
                b.ee_sub_17a730(cx.guard, cx.now, cx.h);
                Child::Lifted
            }
            0x11 => Child::Wait,
            // K32: hit goals (C2d0930 / C2d0830 / C2d08b0 slot 7 -> C2d09b0_v7 FUN_00179328)
            7 | 8 | 9 | 10 | 0xb | 0x13 => {
                let bit = match s.code {
                    9 => 5,
                    10 => 6,
                    0xb => 7,
                    _ => -1,
                };
                let len = match s.code {
                    8 => 16.0,
                    _ if bit >= 0 => cx.h.anim_len(cx.guard, bit),
                    _ => 0.0,
                };
                let hit = self.hit.clone();
                if bit >= 0 {
                    // FUN_00209818: face against the hit vector (kind 0x40000: keep the current heading)
                    let v = hit.vec;
                    if (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt() > 1.0e-4 && hit.kind != 0x40000 {
                        cx.body.f_1a8 = (-v[1]).atan2(-v[0]);
                    }
                }
                Child::Hit(HitGoal { code: s.code, bit, t0: cx.now, len, hit })
            }
            _ => Child::None,
        };
        self.sub = s;
        saved
    }

    /// FUN_00177ba0 re-plan
    fn ee_sub_177ba0(&mut self, m: &Mind, cx: &mut Cx) -> Option<usize> {
        let s = if m.f_fbc == 0x15 {
            Sub { code: 0x12, ..Default::default() }
        } else if m.fa8_f94 {
            Sub { code: 0xc, ..Default::default() }
        } else {
            let t = ee_sub_2082d0(m);
            if t != 0 { Sub { code: 4, obj: t, routine: None } } else { Sub { code: 1, ..Default::default() } }
        };
        self.ee_sub_176750(s, m, cx)
    }

    /// child goal ticks (FUN_001eed10: leaf first), then FUN_00177618. Returns (new target for +0x38c,
    /// cursor to save in +0x2dc).
    pub fn tick(&mut self, m: &Mind, cx: &mut Cx) -> (Option<u32>, Option<usize>) {
        match &mut self.child {
            Child::Routine(r) => r.tick(cx),
            Child::Script(s) => s.tick(cx),
            Child::Search(s) => s.tick(&m.sg, cx),
            Child::React(..) | Child::Chase | Child::Wait | Child::None | Child::Hit(_) => cx.body.f_190 = [0.0; 3],
            // K24: 0xb0 slot 9 (ground = +0x208: not flying the REPLACED lift integrator)
            Child::Lifted => {
                cx.body.f_190 = [0.0; 3];
                if let Some(b) = self.b0.as_mut() {
                    b.ee_sub_17a7f8(!cx.body.k24_air, cx.now);
                }
            }
        }
        let mut saved = None;
        let mut target = None;
        for _ in 0..8 {
            let cur = self.sub.code;
            let next: Option<Sub> = match cur {
                // (0xb knocked out: no case in FUN_00177618, the guard stays down)
                0 => Some(Sub { code: 1, ..Default::default() }),
                1 | 0x10 | 0x11 => {
                    if m.fa8_f94 {
                        Some(Sub { code: 0xc, ..Default::default() })
                    } else {
                        let t = ee_sub_2082d0(m);
                        // K22: FUN_0020a408(g, 0) -> sub 2 {stimulus}
                        let st = ee_sub_20a408(m.pos, m.heard, None);
                        if t != 0 {
                            Some(Sub { code: 3, obj: t, routine: None })
                        } else if let Some(k) = st.and_then(|x| m.heard.iter().position(|y| *y == x)) {
                            Some(Sub { code: 2, obj: k as u32 + 1, routine: None })
                        } else if cur == 0x11 && self.a4 > cx.now {
                            None
                        } else {
                            Some(Sub { code: 1, ..Default::default() })
                        }
                    }
                }
                // K22: sub 2 search - target seen -> 3; better stimulus -> FUN_001725e0; done -> 1
                2 => {
                    if m.fa8_f94 {
                        Some(Sub { code: 0xc, ..Default::default() })
                    } else {
                        let t = ee_sub_2082d0(m);
                        if t != 0 {
                            Some(Sub { code: 3, obj: t, routine: None })
                        } else if let Child::Search(se) = &mut self.child {
                            let n = ee_sub_20a408(m.pos, m.heard, se.stim.as_ref());
                            if n != se.stim {
                                se.ee_sub_1725e0(n);
                                None
                            } else if se.ee_sub_172550() {
                                Some(Sub { code: 1, ..Default::default() })
                            } else {
                                None
                            }
                        } else {
                            Some(Sub { code: 1, ..Default::default() })
                        }
                    }
                }
                3 => match self.child {
                    // FUN_00174fb8: the reaction anim reached state 1
                    Child::React(t0, l) if cx.now - t0 < l => None,
                    _ => Some(Sub { code: if m.c3b { 0xd } else { 4 }, obj: self.sub.obj, routine: None }),
                },
                4 => {
                    let t = ee_sub_2082d0(m);
                    if t == self.sub.obj {
                        None
                    } else if t != 0 {
                        Some(Sub { code: 3, obj: t, routine: None })
                    } else {
                        Some(Sub { code: 1, ..Default::default() })
                    }
                }
                5 => match &self.child {
                    Child::Script(s) if !s.ee_sub_171458() => None,
                    _ => {
                        saved = saved.or(self.ee_sub_177ba0(m, cx));
                        None
                    }
                },
                // K24: sub 6 - FUN_0017a9c0 (0xb0 state -1) -> FUN_00177ba0
                6 => {
                    if self.b0.as_ref().map_or(true, |b| b.ee_sub_17a9c0()) {
                        saved = saved.or(self.ee_sub_177ba0(m, cx));
                    }
                    None
                }
                0xc => (!m.fa8_f94).then(|| Sub { code: 1, ..Default::default() }),
                // K32: 7 - 0xb1 done: +0x50 == 3 -> FUN_00177ba0 (-1: FUN_00176b80 self-hit, unreached)
                // 9 / 10 / 0x13 - hit goal done -> FUN_00177ba0 (recover: target ? 4 chase : 1 routine)
                7 | 9 | 10 | 0x13 => {
                    if !matches!(&self.child, Child::Hit(g) if !g.done(cx.now)) {
                        saved = saved.or(self.ee_sub_177ba0(m, cx));
                        if self.sub.code == 4 {
                            target = Some(self.sub.obj); // FUN_00208540
                        }
                    }
                    None
                }
                // K32: 8 - 0xaf done -> stimulus FUN_00156b98 at the guard (hook), sub 2
                8 => match &self.child {
                    Child::Hit(g) if !g.done(cx.now) => None,
                    _ => Some(Sub { code: 2, ..Default::default() }),
                },
                _ => None,
            };
            match next {
                Some(s) if s != self.sub => {
                    if matches!(s.code, 3 | 4) {
                        target = Some(s.obj); // FUN_00208540
                    }
                    if let Some(c) = self.ee_sub_176750(s, m, cx) {
                        saved = Some(c);
                    }
                }
                _ => break,
            }
            if self.sub.code == cur {
                break;
            }
        }
        // 0x9d / 0xa5 face the target (FUN_00174f08 +0x38)
        if matches!(self.child, Child::React(..) | Child::Chase) {
            if let Some(p) = (m.pos_of)(self.sub.obj) {
                cx.body.f_1a8 = (p[1] - m.pos[1]).atan2(p[0] - m.pos[0]);
            }
        }
        (target, saved)
    }
}
