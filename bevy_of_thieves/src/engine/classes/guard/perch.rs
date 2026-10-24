//! BOSS: class 59 (Dimitri, vt 0x2cde40) brain substate 0xd = goal 0xb4 (vt 0x2cddc0) with its child goal
//! 0xb5 (vt 0x2cdd40): fight from perches. Records +0x1014[+0x1010] (0x24 bytes, FUN_00155530 / method
//! field 1340): {perch A oid, A obj, vol A oid, vol A obj, perch B oid, B obj, vol B oid, vol B obj, used}.
//!
//! 0xb4 (+0x30 state, +0x38 target, +0x3c record): enter FUN_001555e0, update FUN_00155660, exit FUN_00155598,
//! setstate FUN_00155788 (0: child 0xb5, guard +0x1018 = +0x33c; 1: child 0xa5 chase with +0x40 = 1), record
//! pick FUN_00155868. 0xb5 (+0x30 state, +0x40 perch obj, +0x48 landing time, +0x4c target, +0x50 attack count,
//! +0x54 attack): enter FUN_00155a38, update FUN_00155d90, setstate FUN_00155ae0, perch side FUN_00155a68.
//!
//! REPLACED: state 0's landing (vtable +0xec FUN_002041c0 by `perch - guard +0x240`, then +0x11c / +0x124
//! velocity resets) = the guard is placed on the perch point with no velocity; state 2's 0xa7 nav walk
//! (+0x294 path {guard, perch}, FUN_00178d90, +0xc0 = 8) = the 0xba straight-line goto (same 2-point path);
//! the 0xa4 attack (state 3) runs on the K11 attack driver (attack.rs), started / polled by the guard layer
//! (`Perch::want_attack`, `Perch::attack_done`).
use super::brain::Chase;
use super::patrol::{Cx, Goto};
use super::sensor::{ee_sub_227c90, Vol};

type V3 = [f32; 3];

/// one perch record, oids resolved (FUN_00155450)
#[derive(Clone, Debug)]
pub struct Rec {
    pub a: Option<V3>,
    pub va: Option<Vol>,
    pub b: Option<V3>,
    pub vb: Option<Vol>,
    /// +0x20
    pub used: bool,
}

/// DAT_002bdc50 (1 in the recorded game: used records stay eligible)
const DAT_002BDC50: i32 = 1;

fn dist(a: V3, b: V3) -> f32 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

/// FUN_00155868: the record whose nearer perch is farthest from the target
pub fn ee_sub_155868(recs: &[Rec], t: V3) -> Option<usize> {
    let mut best = f32::from_bits(0xff7f_ffff); // DAT_002a5130
    let mut r = None;
    for (k, x) in recs.iter().enumerate() {
        if DAT_002BDC50 == 0 && x.used {
            continue;
        }
        let (Some(a), Some(b)) = (x.a, x.b) else { continue };
        let d = dist(a, t).min(dist(b, t));
        if d > best {
            best = d;
            r = Some(k);
        }
    }
    r
}

/// 0xb5 child (vt 0x2cdd40)
#[derive(Clone, Debug)]
pub struct B5 {
    pub rec: usize,
    /// +0x30 state, +0x34 state time, +0x40 perch point, +0x44, +0x48 landing time, +0x50 attacks started
    pub st: i32,
    pub t34: f32,
    pub f_40: V3,
    pub f_44: f32,
    pub f_48: f32,
    pub f_50: i32,
    /// state 0: 0x9d anim 0xb start / length
    pub anim: (f32, f32),
    /// state 2: the walk
    pub walk: Option<Goto>,
}

/// 0xb4 goal (vt 0x2cddc0)
#[derive(Clone, Debug)]
pub struct Perch {
    /// +0x30 state (-1 / 0 perch fight / 1 chase), +0x34 state time, +0x3c record
    pub st: i32,
    pub t34: f32,
    pub rec: Option<usize>,
    pub b5: Option<B5>,
    pub chase: Option<Box<Chase>>,
    /// guard-layer writes: +0x1018 = +0x33c, record +0x20 = 1
    pub set_1018: bool,
    pub mark_used: Option<usize>,
}

/// what the 0xb4 goal reads of the guard
pub struct PerchIn<'a> {
    pub recs: &'a [Rec],
    pub hp: i32,
    pub f_1018: i32,
    pub tp: Option<V3>,
}

impl B5 {
    /// FUN_00155a68: perch side by the target's volume (A when in vol A or in neither)
    fn ee_sub_155a68(r: &Rec, tp: V3) -> V3 {
        let a = r.a.unwrap_or(tp);
        if r.va.as_ref().is_some_and(|v| ee_sub_227c90(v, tp)) {
            return a;
        }
        if r.vb.as_ref().is_some_and(|v| ee_sub_227c90(v, tp)) {
            return r.b.unwrap_or(a);
        }
        a
    }

    /// FUN_00155ae0 setstate (states 0..3; leaving 1 = FUN_0020ac28(g, 0, 0, 0) and leaving 3 = +0xf04 are the
    /// attack driver's)
    pub fn ee_sub_155ae0(&mut self, st: i32, r: &Rec, tp: V3, cx: &mut Cx) {
        if st == self.st {
            return;
        }
        self.walk = None;
        cx.body.f_190 = [0.0; 3];
        self.st = st;
        self.t34 = cx.now;
        match st {
            0 => {
                // +0x40 = side; 0x9d +0x3c = 0xb; FUN_00209818(atan2(perch - guard) + pi)
                self.f_40 = Self::ee_sub_155a68(r, tp);
                let d = [self.f_40[0] - cx.body.pos[0], self.f_40[1] - cx.body.pos[1]];
                super::patrol::ee_sub_209818(cx.body, d[1].atan2(d[0]) + std::f32::consts::PI);
                self.anim = (cx.now, cx.h.anim_len(cx.guard, 0xb));
            }
            2 => {
                let mut g = Goto::new(true);
                g.ee_sub_17dd38(cx, self.f_40);
                self.walk = Some(g);
            }
            3 => self.f_50 += 1,
            _ => {}
        }
    }

    /// FUN_00155d90 update (state 1 -> 3 and 3 -> 1 / 2 are driven by the guard layer: `want_attack` /
    /// `attack_done`)
    pub fn tick(&mut self, r: &Rec, tp: V3, cx: &mut Cx) {
        match self.st {
            0 if cx.now - self.anim.0 >= self.anim.1 => {
                // REPLACED landing: on the perch, no velocity
                cx.body.pos = self.f_40;
                self.f_48 = cx.now;
                self.ee_sub_155ae0(1, r, tp, cx);
            }
            1 => {
                // 0x9e wait facing the target (+0x38)
                let d = [tp[0] - cx.body.pos[0], tp[1] - cx.body.pos[1]];
                super::patrol::ee_sub_209818(cx.body, d[1].atan2(d[0]));
                cx.body.f_190 = [0.0; 3];
            }
            2 => {
                if let Some(g) = self.walk.as_mut() {
                    g.tick(cx);
                }
                if self.walk.as_ref().is_none_or(|g| g.ee_sub_17dcb8(cx.body.pos)) {
                    self.ee_sub_155ae0(1, r, tp, cx);
                }
            }
            _ => cx.body.f_190 = [0.0; 3],
        }
    }

    /// state 1: 2.5 s after landing and 0.1 s in the state -> FUN_001758a0
    pub fn want_attack(&self, now: f32) -> bool {
        self.st == 1 && now - self.f_48 > 2.5 && now - self.t34 > 0.1
    }

    /// state 3, FUN_00175748 (the attack ended): +0x44 = now; side changed -> 2 (walk over), else 1
    pub fn attack_done(&mut self, r: &Rec, tp: V3, cx: &mut Cx) {
        self.f_44 = cx.now;
        let side = Self::ee_sub_155a68(r, tp);
        let st = if side != self.f_40 {
            self.f_40 = side;
            2
        } else {
            1
        };
        self.ee_sub_155ae0(st, r, tp, cx);
    }
}

impl Perch {
    /// FUN_00155560 (constructor part) + FUN_001555e0 enter
    pub fn ee_sub_1555e0(prev: Option<usize>, i: &PerchIn, cx: &mut Cx) -> Perch {
        let mut p = Perch { st: -1, t34: cx.now, rec: prev, b5: None, chase: None, set_1018: false, mark_used: None };
        if i.f_1018.wrapping_sub(i.hp) >= 0x5a {
            p.rec = i.tp.and_then(|t| ee_sub_155868(i.recs, t));
        }
        let st = if p.rec.is_none() { 1 } else { 0 };
        if std::env::var("TMPBOSS").is_ok() { eprintln!("TMPBOSS 1555e0 prev {prev:?} hp {} f1018 {} tp {:?} recs {:?} -> rec {:?} len0xb {}", i.hp, i.f_1018, i.tp, i.recs.iter().map(|r| (r.a, r.b, r.va.is_some(), r.vb.is_some())).collect::<Vec<_>>(), p.rec, cx.h.anim_len(cx.guard, 0xb)); }
        p.ee_sub_155788(st, i, cx);
        p
    }

    /// FUN_00155788 setstate
    pub fn ee_sub_155788(&mut self, st: i32, i: &PerchIn, cx: &mut Cx) {
        if st == self.st {
            return;
        }
        if self.st == 0 {
            self.mark_used = self.rec;
        }
        self.b5 = None;
        self.chase = None;
        cx.body.f_190 = [0.0; 3];
        self.st = st;
        self.t34 = cx.now;
        match st {
            0 => {
                let rec = self.rec.unwrap_or(0);
                let mut b = B5 { rec, st: -1, t34: cx.now, f_40: [0.0; 3], f_44: f32::MAX, f_48: 0.0, f_50: 0, anim: (cx.now, 0.0), walk: None };
                if let (Some(r), Some(tp)) = (i.recs.get(rec), i.tp) {
                    b.ee_sub_155ae0(0, r, tp, cx);
                }
                self.b5 = Some(b);
                self.set_1018 = true;
            }
            1 => self.chase = Some(Box::new(Chase::new(cx.now))),
            _ => {}
        }
    }

    /// FUN_00155660 update (children first, FUN_001eed10 order). `chase_r` / `atk_v` / `give_up` as the
    /// brain's 0xa5 chase.
    pub fn tick(&mut self, i: &PerchIn, chase_r: [f32; 2], atk_v: Option<V3>, give_up: bool, cx: &mut Cx) {
        // FUN_00176060 needs chase +0x40 == 0; FUN_00155788 state 1 sets +0x40 = 1: the perch chase never gives up
        let _ = give_up;
        if let Some(c) = self.chase.as_mut() {
            c.tick(i.tp, chase_r, atk_v, false, cx);
        }
        let Some(tp) = i.tp else { return };
        if let Some(b) = self.b5.as_mut() {
            if let Some(r) = i.recs.get(b.rec) {
                b.tick(r, tp, cx);
            }
        }
        if self.st == 0 {
            let d = [cx.body.pos[0] - tp[0], cx.body.pos[1] - tp[1], cx.body.pos[2] - tp[2]];
            let near = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() < 850.0 && d[2].abs() < 400.0;
            if near && self.b5.as_ref().is_some_and(|b| b.st != 0) {
                self.ee_sub_155788(1, i, cx);
            }
        }
    }
}

impl super::super::ClassSet {
    /// BOSS: FUN_00206820(guard) (script method 454 = field 330, thunk FUN_00103fe8): vtable +0xdc slot 26
    /// FUN_00206748 reset; message 0xf; vtable +0xd4 slot 25 FUN_00206518 (home +0x300 / +0x310 = the guard's
    /// current pose, health / armour start values, static sensor entries, brain start; +0x494 -> FUN_001f1a40 +
    /// FUN_001f1cf8); message 0xe. Level row 5 calls it on Dimitri in the 1918 / 1929 enters before the
    /// routine (method 441) warps him, so his home follows the fight (EMU 282: +0x300 = (702, 1066, 801)).
    /// false = `obj` is no ported guard.
    pub fn k6_reset_206820(&mut self, obj: usize, h: &mut dyn super::super::ClassHost) -> bool {
        let Some(gi) = self.k6.iter().position(|x| x.0 == obj && matches!(x.1, super::super::Inst::Guard(_))) else { return false };
        let (now, players) = (self.g.t, self.g.player_globals());
        let super::super::Inst::Guard(g) = &mut self.k6[gi].1 else { return false };
        g.ee_sub_206748(obj, now, h);
        h.vcall_1c(obj, 0xf, 0);
        g.ee_sub_206518(obj, &players, now, h);
        let loot = g.f_494_oid != 0;
        if loot {
            g.ee_sub_1f1a40(obj, &mut self.rng, h);
            self.ee_sub_1f1cf8(gi, h);
        }
        h.vcall_1c(obj, 0xe, 0);
        true
    }
}
