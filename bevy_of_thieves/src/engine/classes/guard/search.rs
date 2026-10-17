//! K22: class 171 = guard SEARCH goal 0xab (C2cfa10, vtable 0x2cfa10, size 0xe0, parent 155 goal base):
//! investigate a stimulus. Slots 6 FUN_00171f58 (init), 7 FUN_00171fa0 (enter), 8 FUN_00171fd0 (exit),
//! 9 FUN_00172018 (update), setstate FUN_00172320, stimulus FUN_001725e0.
//! States (+0x30): -1 done, 0 surprise anim (0x9d, anim 0xc), 1 go to the stimulus (0xba goto), 2 look at it
//! (0x9e wait, dwell +0x38), 3 wander the 4 search points (0xba goto), 4 give-up anim (0x9d, anim 0xe),
//! 5 alarm wait (0x9e, +0x3c = 1).
//! The child goals are got from the guard's goal pool (FUN_00205438(guard, class)); here the goal owns its
//! child (+0x18) like the rest of the guard tasks.
use super::patrol::{Cx, Goto};
use super::slots::Stimulus;
use crate::engine::physics::driven::GameRng;
use crate::engine::player::FUN_0011e1a0;

type V3 = [f32; 3];

/// DAT_002a5398 / DAT_002a53b8 / DAT_002a53d4 (FLT_MAX), DAT_002a53d0 (-FLT_MAX)
const D_5398: f32 = f32::MAX;
const D_53B8: f32 = f32::MAX;
const D_53D4: f32 = f32::MAX;
const D_53D0: f32 = f32::MIN;
/// DAT_002be5a0, DAT_002be5a8 / ac (search-point angle step), DAT_002be5b0 / b4 (search-point radius)
const D_E5A0: f32 = 2.0;
const D_E5A8: f32 = -0.7;
const D_E5AC: f32 = -0.5;
const D_E5B0: f32 = 300.0;
const D_E5B4: f32 = 400.0;

/// +0x18 child goal
#[derive(Clone, Debug, Default)]
pub enum SChild {
    #[default]
    None,
    /// 0x9d anim task (+0x3c anim): start time, length; state 1 (FUN_00174fb8) = played
    Anim(i32, f32, f32),
    /// 0xba goto (+0xc0 = 1)
    Goto(Box<Goto>),
    /// 0x9e wait (+0x3c); its state 1 is never reached here (the wait goal's own update is unported)
    Wait(i32),
}

impl SChild {
    /// FUN_00174fb8(child): state == 1
    fn ee_sub_174fb8(&self, now: f32) -> bool {
        matches!(self, SChild::Anim(_, t0, l) if now - t0 >= *l)
    }
}

/// 0xab search goal
#[derive(Clone, Debug)]
pub struct Search {
    /// +0x30 state, +0x34 state start, +0x38 dwell end, +0x3c point timer, +0x40
    pub st: i32,
    pub f_34: f32,
    pub f_38: f32,
    pub f_3c: f32,
    pub f_40: f32,
    /// +0x44 stimulus, +0x48 stimulus-changed flag
    pub stim: Option<Stimulus>,
    pub f_48: bool,
    /// +0x50 four search points {goto point, look point}, +0xd0 next index
    pub pts: [(V3, V3); 4],
    pub f_d0: u32,
    /// +0x18 child
    pub child: SChild,
    /// REPLACED: the game's global rand() stream (FUN_0011d448) - per-goal stream
    pub rng: GameRng,
}

/// What the search goal reads from its guard (+0x10).
pub struct SGuard {
    /// FUN_00209098: +0xfc8 != 0 || FUN_0020f1b8(0x3dd490) (alarm)
    pub alarm: bool,
    /// +0xe6c / +0xe70 dwell range, +0xe78 mode
    pub f_e6c: f32,
    pub f_e70: f32,
    pub f_e78: i32,
    /// +0xe74 leash object position (None = 0)
    pub leash: Option<V3>,
}

impl Search {
    /// slot 6 FUN_00171f58: +0x30 = -1, +0x40 = +0x38 = DAT_002a5370 (the FUN_00209890(g, 0x1f) behaviour bit
    /// is the brain's `child_bits`)
    pub fn ee_sub_171f58() -> Search {
        Search {
            st: -1,
            f_34: 0.0,
            f_38: 0.0,
            f_3c: 0.0,
            f_40: 0.0,
            stim: None,
            f_48: false,
            pts: [([0.0; 3], [0.0; 3]); 4],
            f_d0: 0,
            child: SChild::None,
            rng: GameRng::default(),
        }
    }

    /// FUN_001725e0(goal, stimulus): retarget, +0x48 = 1 (the pool refcounts are not modelled)
    pub fn ee_sub_1725e0(&mut self, s: Option<Stimulus>) {
        if s != self.stim {
            self.stim = s;
            self.f_48 = true;
        }
    }

    /// slot 7 FUN_00171fa0: setstate 0
    pub fn ee_sub_171fa0(&mut self, g: &SGuard, cx: &mut Cx) {
        self.ee_sub_172320(0, g, cx);
    }

    /// slot 8 FUN_00171fd0: kill child, setstate -1, drop the stimulus
    pub fn ee_sub_171fd0(&mut self, g: &SGuard, cx: &mut Cx) {
        self.child = SChild::None;
        self.ee_sub_172320(-1, g, cx);
        self.ee_sub_1725e0(None);
    }

    /// FUN_00172550: done
    pub fn ee_sub_172550(&self) -> bool {
        self.st == -1
    }

    /// FUN_00172760: stimulus position (+0x20 for a level-4 stimulus about this guard, else +0x30)
    pub fn ee_sub_172760(&self, guard: usize) -> V3 {
        let Some(s) = &self.stim else { return [0.0; 3] };
        if s.level == 4 && s.subject == guard as u32 + 1 {
            s.pos
        } else {
            s.pos30
        }
    }

    /// FUN_00172658: stay put (cannot walk, mode 2, or the stimulus is beyond the leash object + 50)
    pub fn ee_sub_172658(&self, g: &SGuard, cx: &Cx) -> bool {
        if cx.body.f_fe8 & 2 == 0 || g.f_e78 == 2 {
            return true;
        }
        let Some(l) = g.leash else { return false };
        let d = |a: V3, b: V3| ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
        let dl = d(l, cx.body.pos);
        let ds = d(self.ee_sub_172760(cx.guard), cx.body.pos);
        dl + 50.0 > ds
    }

    /// FUN_00172560: dwell end = now + rand(+0xe6c, +0xe70); point timer = FLT_MAX (now + 2 in mode 1)
    fn ee_sub_172560(&mut self, g: &SGuard, now: f32) {
        self.f_38 = now + self.rng.FUN_0011d448(g.f_e6c, g.f_e70);
        self.f_3c = if g.f_e78 == 1 { now + D_E5A0 } else { D_53B8 };
    }

    /// face the stimulus: FUN_00209818(atan2(stim - guard), guard + 0xfd0)
    fn face(&self, cx: &mut Cx) {
        let p = self.ee_sub_172760(cx.guard);
        let d = [p[0] - cx.body.pos[0], p[1] - cx.body.pos[1]];
        super::patrol::ee_sub_209818(cx.body, d[1].atan2(d[0]));
    }

    /// goto child retarget (FUN_0017dd38 with point in the +0x10 half of the record)
    fn goto_to(&mut self, cx: &mut Cx, p: V3) {
        if let SChild::Goto(gt) = &mut self.child {
            gt.ee_sub_17dd38(cx, p);
        }
    }

    /// FUN_001728b0 (REPLACED): the candidate scan over the world lists (DAT_003e13fc / 13f0 / 1408 / 13e4 /
    /// 13c0 / 13d8 / 13cc / 13b4, FUN_0015e9c8 classes 0x23 / 0x1b, FUN_00172798 angle window, sort
    /// FUN_00172888, >= 150 apart, 200 toward the guard) is unported: no candidates, so the fallback fills
    /// the four points: base = guard + polar(yaw + pi, -350), then point i = base + polar(yaw + sum rand(-0.7,
    /// -0.5), rand(300, 400)).
    fn ee_sub_1728b0(&mut self, cx: &mut Cx) {
        let pos = cx.body.pos;
        let yaw = cx.body.yaw; // FUN_00204740
        let o = FUN_0011e1a0(yaw + std::f32::consts::PI, -350.0, 0.0);
        let base = [pos[0] + o[0], pos[1] + o[1], pos[2] + o[2]];
        let mut f14 = 0.0f32;
        for i in 0..4 {
            f14 += self.rng.FUN_0011d448(D_E5A8, D_E5AC);
            let a = f14 + yaw;
            let r = self.rng.FUN_0011d448(D_E5B0, D_E5B4);
            let v = FUN_0011e1a0(a, r, 0.0);
            let p = [base[0] + v[0], base[1] + v[1], base[2] + v[2]];
            self.pts[i] = (p, p);
        }
        self.f_d0 = 0;
        self.f_3c = D_53D0;
    }

    /// FUN_00172f10: next search point once the goto arrived + 1 s
    fn ee_sub_172f10(&mut self, cx: &mut Cx) {
        if self.f_d0 >= 4 {
            return;
        }
        let arrived = matches!(&self.child, SChild::Goto(gt) if gt.ee_sub_17dcb8(cx.body.pos));
        if self.f_3c == D_53D4 && arrived {
            self.f_3c = cx.now + 1.0;
        }
        if self.f_3c <= cx.now {
            let p = self.pts[self.f_d0 as usize].0;
            self.f_d0 += 1;
            if self.f_d0 > 3 {
                self.f_d0 = 0;
            }
            self.goto_to(cx, p);
            self.f_3c = D_53D4;
        }
    }

    /// FUN_00172320 setstate
    pub fn ee_sub_172320(&mut self, s: i32, g: &SGuard, cx: &mut Cx) {
        if s == self.st {
            return;
        }
        // FUN_001ef0d8(+0x18): kill the child
        self.child = SChild::None;
        cx.body.f_190 = [0.0; 3];
        self.st = s;
        self.f_34 = cx.now;
        match s {
            0 => self.child = SChild::Anim(0xc, cx.now, cx.h.anim_len(cx.guard, 0xc)),
            1 => {
                let mut gt = Goto::new(false);
                let p = self.ee_sub_172760(cx.guard);
                gt.ee_sub_17dd38(cx, p);
                self.child = SChild::Goto(Box::new(gt));
                self.f_38 = D_5398;
            }
            2 => {
                self.face(cx);
                self.child = SChild::Wait(0);
                self.ee_sub_172560(g, cx.now);
            }
            3 => {
                self.child = SChild::Goto(Box::new(Goto::new(false)));
                self.ee_sub_1728b0(cx);
                self.ee_sub_172f10(cx);
            }
            4 => {
                self.child = SChild::Anim(0xe, cx.now, cx.h.anim_len(cx.guard, 0xe));
                self.ee_sub_1725e0(None);
                self.f_48 = false;
            }
            5 => self.child = SChild::Wait(1),
            _ => {}
        }
    }

    /// child update first (FUN_001eed10 leaf first), then slot 9
    pub fn tick(&mut self, g: &SGuard, cx: &mut Cx) {
        match &mut self.child {
            SChild::Goto(gt) => gt.tick(cx),
            _ => cx.body.f_190 = [0.0; 3],
        }
        self.ee_sub_172018(g, cx);
    }

    /// slot 9 FUN_00172018: the state machine (loops while the state changes; FUN_001ef0a8 pending = never)
    pub fn ee_sub_172018(&mut self, g: &SGuard, cx: &mut Cx) {
        for _ in 0..16 {
            let cur = self.st;
            match cur {
                0 => {
                    if self.child.ee_sub_174fb8(cx.now) {
                        self.ee_sub_172320(1, g, cx);
                    }
                }
                1 => {
                    if g.alarm {
                        self.ee_sub_172320(5, g, cx);
                    } else if !self.f_48 {
                        let (done, failed) = match &self.child {
                            SChild::Goto(gt) => (
                                gt.ee_sub_17dcb8(cx.body.pos),
                                // FUN_0017dcf8: goto state 1..2 and the walk is stuck (FUN_00178d80 +0x4c == 3)
                                (1..3).contains(&gt.st) && gt.walk.as_ref().is_some_and(|w| w.st == 3),
                            ),
                            _ => (false, false),
                        };
                        if self.ee_sub_172658(g, cx) || done || failed {
                            self.ee_sub_172320(2, g, cx);
                        }
                    } else if self.ee_sub_172658(g, cx) {
                        self.ee_sub_172320(2, g, cx);
                        self.f_48 = false;
                    } else {
                        let p = self.ee_sub_172760(cx.guard);
                        self.goto_to(cx, p);
                        self.f_48 = false;
                    }
                }
                2 => {
                    if g.alarm {
                        self.ee_sub_172320(5, g, cx);
                    } else if !self.f_48 {
                        if self.f_38 < cx.now || self.child.ee_sub_174fb8(cx.now) {
                            self.ee_sub_172320(4, g, cx);
                        } else if self.f_3c < cx.now {
                            self.ee_sub_172320(3, g, cx);
                        } else {
                            self.face(cx);
                        }
                    } else if !self.ee_sub_172658(g, cx) {
                        self.ee_sub_172320(1, g, cx);
                        self.f_48 = false;
                    } else {
                        self.ee_sub_172560(g, cx.now);
                        self.f_48 = false;
                    }
                }
                3 => {
                    if g.alarm {
                        self.ee_sub_172320(5, g, cx);
                    } else if !self.f_48 {
                        if self.f_38 < cx.now {
                            self.ee_sub_172320(4, g, cx);
                        } else {
                            self.ee_sub_172f10(cx);
                        }
                    } else if self.ee_sub_172658(g, cx) {
                        self.ee_sub_172320(2, g, cx);
                        self.f_48 = false;
                    } else {
                        self.ee_sub_172320(1, g, cx);
                        self.f_48 = false;
                    }
                }
                4 => {
                    if !self.f_48 {
                        if self.child.ee_sub_174fb8(cx.now) {
                            self.ee_sub_172320(-1, g, cx);
                        }
                    } else if self.ee_sub_172658(g, cx) {
                        self.ee_sub_172320(2, g, cx);
                        self.f_48 = false;
                    } else {
                        self.ee_sub_172320(1, g, cx);
                        self.f_48 = false;
                    }
                }
                5 => {
                    if !g.alarm {
                        self.ee_sub_172320(1, g, cx);
                    }
                }
                _ => {}
            }
            if self.st == cur {
                break;
            }
        }
    }
}

/// FUN_0020a168(guard, a, b): stimulus order (< 0: a beats b) - kind (+0x14 -> +0x50), then level (+0x54),
/// then +0x4c, then the nearer one (equal distance: the pool index difference, 0 here)
pub fn ee_sub_20a168(gpos: V3, a: &Stimulus, b: &Stimulus) -> i32 {
    let k = a.mode - b.mode;
    if k != 0 {
        return k;
    }
    let l = b.level - a.level;
    if l != 0 {
        return l;
    }
    if a.f_4c > b.f_4c {
        return -1;
    }
    if a.f_4c < b.f_4c {
        return 1;
    }
    let d2 = |p: V3| (gpos[0] - p[0]).powi(2) + (gpos[1] - p[1]).powi(2) + (gpos[2] - p[2]).powi(2);
    if d2(b.pos) > d2(a.pos) { -1 } else { 0 }
}

/// FUN_0020a408(guard, cur): the best perceived stimulus (`heard` = the DAT_003e148c list filtered by slot 88
/// FUN_00208be0, gate FUN_00208d70 applied by the caller), else `cur`. The level-4 sensor-entry refresh
/// (FUN_00154180 / FUN_00154080(e, 2), +0xf04 timer) is UNPORTED.
pub fn ee_sub_20a408(gpos: V3, heard: &[Stimulus], cur: Option<&Stimulus>) -> Option<Stimulus> {
    let mut best = cur.cloned();
    for s in heard {
        if best.as_ref().is_none_or(|b| ee_sub_20a168(gpos, s, b) < 0) {
            best = Some(s.clone());
        }
    }
    best
}
