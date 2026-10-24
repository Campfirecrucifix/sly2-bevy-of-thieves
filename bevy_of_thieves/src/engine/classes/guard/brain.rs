//! Guard brain goal (class 0x9c, vtable 0x2d04a0, size 0xb0): substate machine FUN_00177618 / setstate
//! FUN_00176750 / re-plan FUN_00177ba0, and the guard's slot 87 target pick FUN_002082d0.
use super::patrol::{Cmd, Cx, Goto, Routine, Script};
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
    /// sub 4: 0xa5 chase / attack (bit 0x1d, vt 0x2d05a0): WIRE-D `Chase` (approach states 0 / 3)
    Chase(Box<Chase>),
    /// sub 0x11: 0x9e wait
    Wait,
    /// K24: sub 6: 0xb0 lifted goal (`Brain::b0`, guard/lift.rs; bit 0x16)
    Lifted,
    /// BOSS: sub 0xd: class-59 goal 0xb4 perch fight (guard/perch.rs)
    Perch(Box<super::perch::Perch>),
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

/// `Sub::obj` of a sub 2 about the guard's own noise record (`Brain::own_stim`)
pub const OWN_STIM: u32 = u32::MAX;

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
    /// WIRE-D: the record of the guard's own FUN_00156b98 noise (substate +0x10 of sub 2 `OWN_STIM`)
    pub own_stim: Option<super::slots::Stimulus>,
    /// BOSS: the pooled 0xb4 goal's +0x3c record (kept across pushes)
    pub perch_rec: Option<usize>,
    /// POOL1 R71: FUN_00176750's bVar3 (substates 1 / 5 / 0xb / 0x10): FUN_00208540(g, 0) pending for the
    /// guard layer (target +0x38c / record +0x3cc cleared)
    pub clear_target: bool,
}

/// What the brain needs from the guard.
/// WIRE-D: the 0xa5 chase goal (vt 0x2d05a0) - FUN_00176098 (slot 9 update) states 0 (in range: stand,
/// face, attack poll) and 3 (approach: child 0xba goto at the target, FUN_0017dd38 retarget when the target
/// moved more than DAT_002be614 = 500 from +0x50). FUN_00176458 SetState for 0 / 3.
/// BOSS R12: the FUN_001c0d70 region compare runs on the level region tree (guard/region.rs, Body::region_diff);
/// REPLACED: the guard's region = FUN_001c0d70(guard pos, 1); the +0xfc4 speed scale for modes 8 / 0xd is 1; states 1 / 2 (attack 0xa4 / 0xad sub-goals) are
/// the K11 attack driver (attack.rs, FUN_00175ff0 poll from the guard update), 4 / 5 (FUN_00209098 knock /
/// 0x9e wait) are not entered.
#[derive(Clone, Debug)]
pub struct Chase {
    /// +0x34 state, +0x38 state time, +0x50 last goto target
    pub st: i32,
    pub t38: f32,
    pub f_50: [f32; 3],
    /// +0x44 target region at the last retarget
    pub f_44: i32,
    /// +0x18 child 0xba (state 3)
    pub goto: Option<Goto>,
    /// WIRE-G: state 4 (0x9d anim behaviour 0xd) length; `drop_target`: FUN_00154038(sensor, t, 0) +
    /// FUN_00208540(g, 0) pending for the guard layer (the brain only sees the sensor immutably)
    pub l4: f32,
    pub drop_target: bool,
    /// BOSS: state 0's child goal 0xb3 (vt 0x2d0120, FUN_0017bf08 enter / FUN_0017c3e8 update / FUN_0017c550
    /// setstate): 0 = 0x9e face the target, 1 = 0xa7 walk to the FUN_0017bff8 point (REPLACED: straight goto)
    pub b3: Option<Goto>,
}

/// knowledge record (+0x3cc) +0x14 / +0x18 of a boss guard (rec6: 1.0 / 2.0 rad)
const B3_ANG: f32 = 2.0;

/// DAT_002c3db4: FUN_0020a7e0 give-up time (target record +0x3cc not refreshed for 5 s)
pub const DAT_002C3DB4: f32 = 5.0;

/// DAT_002be614
const DAT_002BE614: f32 = 500.0;

impl Chase {
    pub fn new(now: f32) -> Chase {
        Chase { st: 0, t38: now, f_50: [0.0; 3], f_44: 0, goto: None, l4: 0.0, drop_target: false, b3: None }
    }
    /// BOSS: FUN_0017bff8 point: the target plus the guard's direction from it at a distance clamped to +0x31c
    /// (REPLACED: no FUN_001c0a40 / FUN_001c0b50 region-mesh candidates; the guard's current bearing is kept)
    fn ee_sub_17bff8(tp: [f32; 3], cx: &Cx) -> [f32; 3] {
        let d = [cx.body.pos[0] - tp[0], cx.body.pos[1] - tp[1]];
        let n = (d[0] * d[0] + d[1] * d[1]).sqrt().max(1e-3);
        let r = n.clamp(cx.body.f_31c[0], cx.body.f_31c[1]);
        [tp[0] + d[0] / n * r, tp[1] + d[1] / n * r, tp[2]]
    }
    /// BOSS: FUN_0017c3e8 (0xb3 update): keep still facing the target while it is within +0x324 and the facing
    /// error <= knowledge +0x18, else (re)walk to the FUN_0017bff8 point; FUN_0017c550(0) = 0x9e face when the
    /// guard has no walk anim (+0xfe8 bit 4 clear)
    fn ee_sub_17c3e8(&mut self, tp: [f32; 3], cx: &mut Cx) {
        let d = [tp[0] - cx.body.pos[0], tp[1] - cx.body.pos[1]];
        let face = d[1].atan2(d[0]);
        if cx.body.f_fe8 & 4 == 0 {
            self.b3 = None;
            super::patrol::ee_sub_209818(cx.body, face);
            cx.body.f_190 = [0.0; 3];
            return;
        }
        let walking = self.b3.as_ref().is_some_and(|g| !g.ee_sub_17dcb8(cx.body.pos));
        if !walking {
            self.b3 = None;
            let dist = (d[0] * d[0] + d[1] * d[1]).sqrt();
            let mut err = face - cx.body.yaw;
            while err > std::f32::consts::PI { err -= std::f32::consts::TAU; }
            while err < -std::f32::consts::PI { err += std::f32::consts::TAU; }
            let inside = cx.body.f_324[0] < dist && dist < cx.body.f_324[1];
            if inside && err.abs() <= B3_ANG {
                super::patrol::ee_sub_209818(cx.body, face);
                cx.body.f_190 = [0.0; 3];
                return;
            }
            if inside {
                super::patrol::ee_sub_209818(cx.body, face);
                cx.body.f_190 = [0.0; 3];
                return;
            }
            let mut g = Goto::new(true);
            g.ee_sub_17dd38(cx, Self::ee_sub_17bff8(tp, cx));
            self.b3 = Some(g);
        }
        if let Some(g) = self.b3.as_mut() {
            g.tick(cx);
        }
    }
    /// FUN_00176458 (states 0 / 3)
    pub fn ee_sub_176458(&mut self, st: i32, tp: [f32; 3], cx: &mut Cx) {
        if st == self.st {
            return;
        }
        self.goto = None;
        cx.body.f_190 = [0.0; 3];
        self.st = st;
        self.t38 = cx.now;
        if st == 4 {
            // FUN_00205438(g, 0x9d) +0x3c = 0xd; FUN_00154038(sensor, t, 0); FUN_00208540(g, 0)
            self.l4 = cx.h.anim_len(cx.guard, 0xd);
            self.drop_target = true;
        }
        if st == 3 {
            // 0xba +0xc0 = 1, +0x38 = 0x1c, FUN_0017dd38(goal, target), +0x50 = target
            let mut g = Goto::new(true);
            g.ee_sub_17dd38(cx, tp);
            self.goto = Some(g);
            self.f_50 = tp;
        }
    }
    /// FUN_00176098 (states 0 / 3; `r` = guard +0x314 / +0x318)
    /// `give_up` = FUN_00176060: chase +0x40 == 0, guard +0x3d4 == 0 and FUN_0020a7e0(+0x3cc) (stale record)
    pub fn tick(&mut self, tp: Option<[f32; 3]>, r: [f32; 2], atk_v: Option<[f32; 3]>, give_up: bool, cx: &mut Cx) {
        if matches!(self.st, 0 | 3) && give_up {
            self.ee_sub_176458(4, tp.unwrap_or(cx.body.pos), cx);
        }
        if self.st == 4 {
            // FUN_00174fb8 (anim reached state 1) -> -1
            if cx.now - self.t38 >= self.l4 {
                self.st = -1;
            }
            cx.body.f_190 = [0.0; 3];
            return;
        }
        if self.st == -1 {
            cx.body.f_190 = [0.0; 3];
            return;
        }
        if let Some(tp) = tp {
            let d = [tp[0] - cx.body.pos[0], tp[1] - cx.body.pos[1], tp[2] - cx.body.pos[2]];
            let dist = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            match self.st {
                // FUN_00176098: stay (0) only within +0x318 on the same region; approach (3) ends within +0x314
                // on the same region; the goto retargets when the target region (+0x44) changed or it moved
                // more than DAT_002be614
                0 if dist > r[1] || cx.body.region_diff => self.ee_sub_176458(3, tp, cx),
                3 if dist < r[0] && !cx.body.region_diff => self.ee_sub_176458(0, tp, cx),
                3 => {
                    let m = [tp[0] - self.f_50[0], tp[1] - self.f_50[1], tp[2] - self.f_50[2]];
                    if self.f_44 != cx.body.tgt_region || (m[0] * m[0] + m[1] * m[1] + m[2] * m[2]).sqrt() > DAT_002BE614 {
                        self.f_44 = cx.body.tgt_region;
                        self.f_50 = tp;
                        if let Some(g) = self.goto.as_mut() {
                            g.ee_sub_17dd38(cx, tp);
                        }
                    }
                }
                _ => {}
            }
        }
        match self.goto.as_mut() {
            Some(g) => g.tick(cx),
            // WIRE-D: no approach goto: a running attack moves the guard by its anim root motion
            None if atk_v.is_some() => cx.body.f_190 = atk_v.unwrap_or([0.0; 3]),
            // BOSS: state 0 runs its 0xb3 child (FUN_00176458 case 0)
            None if self.st == 0 && tp.is_some() => self.ee_sub_17c3e8(tp.unwrap_or_default(), cx),
            None => cx.body.f_190 = [0.0; 3],
        }
    }
}

pub struct Mind<'a> {
    /// WIRE-D: guard +0x314 / +0x318 (chase near / far range)
    pub chase_r: [f32; 2],
    /// WIRE-D: root velocity of the running K11 attack (0xa4 attack goal anim), None = no attack
    pub atk_v: Option<[f32; 3]>,
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
    /// +0x2e8 / +0x2ec routines, +0x2dc cursor (ANIM R27: the +0x2cc / +0x2d4 ranges are body fields)
    pub routine: Option<Vec<Cmd>>,
    pub cursor: Option<usize>,
    /// level routines (sub 5 `Sub::routine` index)
    pub routines: &'a [Vec<Cmd>],
    /// K22: perceived stimuli (DAT_003e148c list through gate FUN_00208d70 + slot 88; sub 2 `Sub::obj` = index + 1)
    pub heard: &'a [Stimulus],
    /// K22: guard fields the 0xab search goal reads
    pub sg: SGuard,
    /// WIRE-G: +0x3cc target record time (FUN_0020a9a8 stamp), None = no record
    pub f_3cc_t: Option<f32>,
    /// BOSS: class 59 perch records (+0x1010 / +0x1014), +0x33c health, +0x1018
    pub perches: &'a [super::perch::Rec],
    pub hp: i32,
    pub f_1018: i32,
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
        // FUN_00176750 switchD_001767fc_default: bVar3 -> FUN_00208540(g, 0)
        self.clear_target |= matches!(s.code, 1 | 5 | 0xb | 0x10);
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
            1 => Child::Routine(Box::new(Routine::ee_sub_179aa0(m.routine.clone(), m.cursor, cx))),
            // FUN_00205438(g, 0xab) + FUN_001725e0(goal, stimulus), pushed (slot 7 FUN_00171fa0)
            2 => {
                let mut se = Search::ee_sub_171f58();
                let st = if s.obj == OWN_STIM { self.own_stim.take() } else { (s.obj as usize).checked_sub(1).and_then(|k| m.heard.get(k)).cloned() };
                se.ee_sub_1725e0(st);
                se.ee_sub_171fa0(&m.sg, cx);
                Child::Search(Box::new(se))
            }
            3 => Child::React(cx.now, cx.h.anim_len(cx.guard, 0xb)),
            4 => Child::Chase(Box::new(Chase::new(cx.now))),
            // BOSS: 0xd -> FUN_00205438(g, 0xb4) +0x38 = target, pushed (slot 7 FUN_001555e0; +0x3c kept in the
            // guard's pooled goal = `Brain::perch_rec`)
            0xd => {
                let i = super::perch::PerchIn { recs: m.perches, hp: m.hp, f_1018: m.f_1018, tp: (m.pos_of)(s.obj) };
                let p = super::perch::Perch::ee_sub_1555e0(self.perch_rec, &i, cx);
                self.perch_rec = p.rec;
                Child::Perch(Box::new(p))
            }
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
        // BOSS: is-a 0x3b with perch records -> {0xd, FUN_001c4840(0x3dd490) = the player}
        let s = if m.c3b && !m.perches.is_empty() {
            Sub { code: 0xd, obj: super::PLAYER_HANDLE, routine: None }
        } else if m.f_fbc == 0x15 {
            Sub { code: 0x12, ..Default::default() }
        } else if m.fa8_f94 {
            Sub { code: 0xc, ..Default::default() }
        } else {
            let t = ee_sub_2082d0(m);
            if t != 0 {
                Sub { code: 4, obj: t, routine: None }
            } else if (6..0xb).contains(&self.sub.code) {
                // 5 < +0x30 < 0xb: FUN_00156b98(0x3dd490, 0, FUN_001a62e8(g), g) and sub 2 about it
                self.own_noise(cx)
            } else {
                Sub { code: 1, ..Default::default() }
            }
        };
        self.ee_sub_176750(s, m, cx)
    }

    /// FUN_00156b98(0x3dd490, 0, guard pos, guard): the noise (other guards hear it through the pool) and
    /// the sub-2 record about it
    fn own_noise(&mut self, cx: &mut Cx) -> Sub {
        let p = cx.body.pos;
        cx.h.ee_sub_156b98(cx.guard, p);
        self.own_stim = Some(super::hearing::StimKind::world_sound().ee_sub_156548(p, cx.guard as u32 + 1, cx.now));
        Sub { code: 2, obj: OWN_STIM, routine: None }
    }

    /// child goal ticks (FUN_001eed10: leaf first), then FUN_00177618. Returns (new target for +0x38c,
    /// cursor to save in +0x2dc).
    pub fn tick(&mut self, m: &Mind, cx: &mut Cx) -> (Option<u32>, Option<usize>) {
        match &mut self.child {
            Child::Routine(r) => r.tick(cx),
            Child::Script(s) => s.tick(cx),
            Child::Search(s) => s.tick(&m.sg, cx),
            // FUN_00179328: HitGoal per-tick update (C2d09b0 slot 9)
            Child::Hit(g) => {
                cx.body.f_190 = [0.0; 3];
                // face against the hit vector (kind 0x40000: face from owner forward)
                let v = g.hit.vec;
                let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
                let heading = if len > 1.0e-4 && g.hit.kind != 0x40000 {
                    (-v[1]).atan2(-v[0])
                } else {
                    // FUN_001a6318 -> atan2 of owner forward (row 0 of +0xfc matrix) = current yaw
                    cx.body.yaw
                };
                super::patrol::ee_sub_209818(cx.body, heading);
            }
            Child::Chase(c) => {
                let give_up = m.f_3cc_t.is_some_and(|t| cx.now - t > DAT_002C3DB4);
                c.tick((m.pos_of)(self.sub.obj), m.chase_r, m.atk_v, give_up, cx)
            }
            Child::Perch(p) => {
                let give_up = m.f_3cc_t.is_some_and(|t| cx.now - t > DAT_002C3DB4);
                let i = super::perch::PerchIn { recs: m.perches, hp: m.hp, f_1018: m.f_1018, tp: (m.pos_of)(self.sub.obj) };
                p.tick(&i, m.chase_r, m.atk_v, give_up, cx);
            }
            Child::React(..) | Child::Wait | Child::None => cx.body.f_190 = [0.0; 3],
            // K24: 0xb0 slot 9 (ground = +0x208: not flying the REPLACED lift integrator)
            Child::Lifted => {
                cx.body.f_190 = [0.0; 3];
                if let Some(b) = self.b0.as_mut() {
                    b.ee_sub_17a7f8(!cx.body.k24_air, cx.now, cx.guard, cx.h);
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
                // FUN_00175810: chase state -1 -> 1 (LAB_00177a14); target compare against guard +0x38c
                4 if matches!(&self.child, Child::Chase(c) if c.st == -1) => Some(Sub { code: 1, ..Default::default() }),
                4 => {
                    let t = ee_sub_2082d0(m);
                    if t == m.f_38c {
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
                    _ => Some(self.own_noise(cx)),
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
        if matches!(&self.child, Child::React(..)) || matches!(&self.child, Child::Chase(c) if c.st != 3) {
            if let Some(p) = (m.pos_of)(self.sub.obj) {
                cx.body.f_1a8 = (p[1] - m.pos[1]).atan2(p[0] - m.pos[0]);
            }
        }
        (target, saved)
    }
}
