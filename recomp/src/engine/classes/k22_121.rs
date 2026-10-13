//! WP-K22: C2cd5b8 (world class 121, size 0x10b0, parent C2c94e0 / LO; one instance, level 11): the rhythm
//! (dance) game manager `DAT_002de570`. Driven by the music position (FUN_001fe3f8(0x3dd040)) it holds:
//! the beat track (+0x60 count, 0x30-byte records at +0x70: time, kind, HUD position, hit flag; set by
//! FUN_001531a8 from script action 0x49), up to 3 pending cues (+0x1c0 count, 0x24-byte records at +0x1c4:
//! dancer, button mask, move oid, time, posted time, demo-played, prompt-posted, hit, over; FUN_001532d8 from
//! script action 0x4a), the score history (+0x23c count of the last 16 scores at +0x240, clamped into
//! [+0x234, +0x238]; +0x284 the mean of the last +0x280, +0x288 its smoothed value) and up to 128 crowd
//! records (+0x2a8 count, 0x1c-byte records at +0x2ac, field 1346: crowd object oid, cheer anim 0x626..0x628,
//! state-machine link, current level, 3 score thresholds) whose cheer level follows the smoothed score.
//! States (+0x50): 0 off (the dancer plays by buttons), 1 demo (each cue plays its move when due), 2 player
//! (prompt the cue, judge the press in FUN_001533b0, a missed cue scores +0x2a4).
//! 1:1: slots 5 FUN_001141a0, 6 FUN_001526d8, 14 FUN_001527c8, 15 FUN_00152970; FUN_00152b58,
//! FUN_00152810, FUN_00153128, FUN_001531a8, FUN_001532d8, FUN_001533b0, FUN_001535a0, FUN_00153638,
//! FUN_00153640, FUN_00153760, FUN_00153810, FUN_00152628, FUN_00152680; slot 18 FUN_00152c58's beat-cursor
//! motion (+0x190 / +0x194 / +0x1a0 / +0x1b0).
//! REPLACED: slot 14's deferred member call (FUN_00149a50 with the PMF DAT_002a5010 = FUN_00152810) runs at
//! the end of post-load; the global `DAT_002de570` is the ClassSet's (last) class-121 instance, handed to the
//! class-68 dancer (classes/k21_68.rs) instead of `ClassHost::k21_global`.
//! UNPORTED: slot 18's HUD text drawing (FUN_00215ac0 / 215ed8 / 2163b0, the beat icons table DAT_002bdb70).
use super::k21_68::C2cd748;
use super::{ClassHost, Inst};
use crate::engine::anim::keys::ee_sub_117120;
use crate::engine::physics::driven::GameRng;
use crate::engine::physics::dynamics::FUN_0011d018;

/// DAT_002bdbb0: the beat cursor's lead time
const D_BDBB0: f32 = 0.4;
/// DAT_002bdbc0 / _002bdbd0: the cursor launch / end velocities
const V_BDBC0: [f32; 3] = [100.0, -1000.0, 0.0];
const V_BDBD0: [f32; 3] = [100.0, 1000.0, 0.0];
/// DAT_002bdbe0..e8: the miss-penalty scale polynomial in the BIF-103 skill value
const D_BDBE0: f32 = 1.2;
const D_BDBE4: f32 = -0.6;
const D_BDBE8: f32 = 0.0;
/// DAT_002bbb60 / 64
const R_2BBB60: [f32; 2] = [0.0, 1.0];

/// slot 5: FUN_001141a0
pub fn ee_sub_1141a0() -> u32 {
    0x2bb3ec
}

/// FUN_0011d948(x, range)
fn clamp(x: f32, r: [f32; 2]) -> f32 {
    if x < r[0] {
        r[0]
    } else if r[1] < x {
        r[1]
    } else {
        x
    }
}

/// FUN_00152628: cheer anim -> level
pub fn ee_sub_152628(a: u32) -> i32 {
    match a {
        0x626 => 0,
        0x627 => 1,
        0x628 => 2,
        _ => -1,
    }
}

/// FUN_00152680: level -> cheer anim
pub fn ee_sub_152680(k: i32) -> u32 {
    match k {
        0 => 0x626,
        1 => 0x627,
        2 => 0x628,
        _ => u32::MAX,
    }
}

/// a +0x1c4 cue
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Cue {
    /// +0 dancer (handle), +4 button mask, +8 move oid, +0xc time, +0x10 posted at
    pub dancer: u32,
    pub mask: i32,
    pub oid: i32,
    pub t: f32,
    pub t_post: f32,
    /// +0x14 demo played, +0x18 prompt posted, +0x1c hit, +0x20 over
    pub f_14: i32,
    pub f_18: i32,
    pub f_1c: i32,
    pub f_20: i32,
}

/// a +0x2ac crowd record
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Crowd {
    /// +0 oid, +4 cheer anim, +8 state-machine link, +0xc level, +0x10..+0x18 thresholds
    pub oid: i32,
    pub anim: u32,
    pub link: u32,
    pub level: i32,
    pub thr: [f32; 3],
}

#[derive(Clone, Debug)]
pub struct C2cd5b8 {
    /// +0x40 LO flags (inline form)
    pub f_40: u32,
    /// +0x50 state, +0x54 state time, +0x58 early / +0x5c late window
    pub st: i32,
    pub f_54: f32,
    pub f_58: f32,
    pub f_5c: f32,
    /// +0x60 / +0x70 beats (12 words each: time, kind, .., +0x10 position, +0x20 hit)
    pub beats: Vec<[f32; 12]>,
    /// +0x190 cursor state, +0x194 cursor beat, +0x1a0 cursor position, +0x1b0 cursor velocity
    pub f_190: i32,
    pub f_194: i32,
    pub f_1a0: [f32; 3],
    pub f_1b0: [f32; 3],
    /// +0x1c0 / +0x1c4
    pub cues: Vec<Cue>,
    /// +0x230 the last pressed / prompted mask
    pub f_230: i32,
    /// +0x234 score range, +0x23c / +0x240 history, +0x280 mean window, +0x284 mean, +0x288 smoothed,
    /// +0x28c smoothing table
    pub f_234: [f32; 2],
    pub hist: Vec<f32>,
    pub f_280: i32,
    pub f_284: f32,
    pub f_288: f32,
    pub f_28c: [f32; 3],
    /// +0x298 too early, +0x29c wrong button, +0x2a0 pressed twice, +0x2a4 missed
    pub f_298: f32,
    pub f_29c: f32,
    pub f_2a0: f32,
    pub f_2a4: f32,
    /// +0x2a8 / +0x2ac
    pub crowd: Vec<Crowd>,
    /// REPLACED: the global rand() stream (FUN_0011d448)
    pub rng: GameRng,
}

impl Default for C2cd5b8 {
    fn default() -> Self {
        C2cd5b8 {
            f_40: 0,
            st: 0,
            f_54: 0.0,
            f_58: 0.0,
            f_5c: 0.0,
            beats: Vec::new(),
            f_190: 0,
            f_194: 0,
            f_1a0: [0.0; 3],
            f_1b0: [0.0; 3],
            cues: Vec::new(),
            f_230: 0,
            f_234: [0.0; 2],
            hist: Vec::new(),
            f_280: 0,
            f_284: 0.0,
            f_288: 0.0,
            f_28c: [0.0; 3],
            f_298: 0.0,
            f_29c: 0.0,
            f_2a0: 0.0,
            f_2a4: 0.0,
            crowd: Vec::new(),
            rng: GameRng::default(),
        }
    }
}

/// the dancer `d` (handle) among the class-68 instances
fn dancer(ds: &mut [(usize, Inst)], d: u32) -> Option<(usize, &mut C2cd748)> {
    ds.iter_mut().find_map(|(i, x)| match x {
        Inst::K21C68(c) if *i as u32 + 1 == d => Some((*i, &mut **c)),
        _ => None,
    })
}

impl C2cd5b8 {
    /// slot 6 FUN_001526d8 (also sets DAT_002de570 = self)
    pub fn ee_sub_1526d8(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1a5270, obj);
        self.f_234 = [-1.5, 1.0];
        self.f_5c = 0.4;
        self.f_280 = 8;
        self.f_58 = 0.4;
        self.f_28c = [0.8333334, 0.0, 5.0];
        self.f_29c = -0.8;
        self.f_2a0 = -0.2;
        self.f_2a4 = -1.5;
        self.f_298 = -1.5;
        h.ee_sub_1a6ca8(obj, 1);
        self.f_40 |= 10;
    }

    /// FUN_00153810 (field 1346 through FUN_0010f918): add a crowd record {oid, cheer anim}
    pub fn ee_sub_153810(&mut self, oid: i32, anim: u32) {
        if self.crowd.len() < 0x80 {
            self.crowd.push(Crowd { oid, anim, level: -1, ..Default::default() });
        }
    }

    /// loader fields 1344 (+0x58), 1345 (+0x5c), 1346 (crowd records)
    pub fn feed(&mut self, props: &[crate::engine::world::props::Prop]) {
        use crate::engine::world::props::{ArrayItem, PropValue};
        let w = |b: &[u8]| b.get(..4).map(|x| u32::from_le_bytes([x[0], x[1], x[2], x[3]])).unwrap_or(0);
        for p in props {
            match (&p.value, p.field) {
                (PropValue::Scalar(b), 1344) => self.f_58 = f32::from_bits(w(b)),
                (PropValue::Scalar(b), 1345) => self.f_5c = f32::from_bits(w(b)),
                (PropValue::Array(a), 1346) if a.len() >= 2 => {
                    let v = |it: &ArrayItem| match it {
                        ArrayItem::One(_, b) => w(b),
                        _ => 0,
                    };
                    self.ee_sub_153810(v(&a[0]) as i32, v(&a[1]));
                }
                _ => {}
            }
        }
    }

    /// slot 14 FUN_001527c8 = FUN_001a5c48 + the deferred FUN_00152810
    pub fn ee_sub_1527c8(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1a5c48, obj);
    }

    /// FUN_00152810 (deferred): bind each crowd object's cheer state machine (FUN_0015e938 kinds 4 / 1 0x625,
    /// FUN_001f1448), its level from the anim and two random thresholds a third of the range apart
    pub fn ee_sub_152810(&mut self, obj: usize, h: &mut dyn ClassHost) {
        let step = (self.f_234[1] - self.f_234[0]).abs() * 0.33333334;
        let half = step * 0.5;
        for k in 0..self.crowd.len() {
            let c = self.crowd[k];
            let o = h.ee_sub_15e938(4, c.oid, obj as u32 + 1);
            if o == 0 {
                continue;
            }
            let sm = h.ee_sub_15e938(1, 0x625, o);
            if sm == 0 {
                continue;
            }
            let mut slot = c.link;
            super::k17_55::ee_sub_1f1448(sm, o, c.anim as i32, 0, &mut slot, h);
            let r = &mut self.crowd[k];
            r.link = slot;
            r.level = ee_sub_152628(c.anim);
            let mut f = self.f_234[1];
            for i in 0..2 {
                f -= step;
                r.thr[i] = self.rng.FUN_0011d448(f - half, f + half);
            }
            r.thr[2] = self.f_234[0];
        }
    }

    /// FUN_00153128: set state (leaving 1 / entering 1: FUN_0020f030 / FUN_0020efc0(0x3dd490, 3)), drop cues
    pub fn ee_sub_153128(&mut self, s: i32, now: f32, h: &mut dyn ClassHost) {
        if self.st == s {
            return;
        }
        if self.st == 1 {
            h.ee_sub_20f030(3);
        }
        self.st = s;
        self.cues.clear();
        self.f_54 = now;
        if s == 1 {
            h.ee_sub_20efc0(3);
        }
    }

    /// FUN_001531a8 (script action 0x49): the beat track, laid out over 440 HUD units from x = 100, then
    /// setstate
    pub fn ee_sub_1531a8(&mut self, s: i32, beats: &[[f32; 12]], now: f32, h: &mut dyn ClassHost) {
        if !beats.is_empty() {
            self.beats = beats.to_vec();
            let t0 = self.beats[0][0];
            let t1 = self.beats[self.beats.len() - 1][0];
            let base = [100.0f32, 468.8, 0.0];
            for b in &mut self.beats {
                b[8] = 0.0;
                b[4] = base[0] + (440.0 / (t1 - t0)) * (b[0] - t0);
                b[5] = base[1];
                b[6] = base[2];
                b[7] = 0.0;
            }
            self.f_1a0 = [10.0, 468.8, 0.0];
            self.f_190 = 0;
            self.f_194 = 0;
        }
        self.ee_sub_153128(s, now, h);
    }

    /// FUN_001532d8 (script action 0x4a): queue a cue at `t` (3 kept: the oldest is shifted out)
    pub fn ee_sub_1532d8(&mut self, t: f32, dancer: u32, mask: i32, oid: i32, now: f32) {
        if self.cues.len() == 3 {
            self.cues.remove(0);
        }
        self.cues.push(Cue { dancer, mask, oid, t, t_post: now, ..Default::default() });
    }

    /// FUN_00153760: mean of the last +0x280 scores, clamped
    pub fn ee_sub_153760(&self) -> f32 {
        let n = self.f_280;
        if n < 1 {
            return 0.0;
        }
        let lo = (self.hist.len() as i32 - n).max(0) as usize;
        let s: f32 = self.hist[lo..].iter().rev().sum();
        clamp(s / n as f32, self.f_234)
    }

    /// FUN_00153640(score): a miss is scaled by clamp(1.2 - 0.6 * skill); push the clamped score (16 kept),
    /// new mean, FUN_001619a0(0x2deb30, 0x36, self)
    pub fn ee_sub_153640(&mut self, obj: usize, mut s: f32, h: &mut dyn ClassHost) {
        if s < 0.0 {
            let k = h.ee_sub_154c90();
            s *= clamp(D_BDBE0 + k * (D_BDBE4 + k * D_BDBE8), R_2BBB60);
        }
        if self.hist.len() == 16 {
            self.hist.remove(0);
        }
        self.hist.push(clamp(s, self.f_234));
        self.f_284 = self.ee_sub_153760();
        h.ee_sub_1619a0(0x36, obj as u32 + 1);
    }

    /// FUN_001535a0(cue): timing accuracy, 1 on the beat .. 0 at the window edge
    pub fn ee_sub_1535a0(&self, c: &Cue, now: f32) -> f32 {
        let (d, w) = if now < c.t { (c.t - now, self.f_58) } else { (now - c.t, self.f_5c) };
        let f = 1.0 - d / w;
        (1.0 - f) * R_2BBB60[0] + f * R_2BBB60[1]
    }

    /// FUN_001533b0(self, dancer): judge the dancer's press against its open cue
    pub fn ee_sub_1533b0(&mut self, obj: usize, dobj: usize, d: &mut C2cd748, now: f32, h: &mut dyn ClassHost) {
        let tracker = d.m.u(0x150);
        let pressed = h.ee_call(0x1c4660, &[0, tracker, 0xf0]) as i32;
        self.f_230 = pressed;
        if pressed == 0 {
            return;
        }
        let dh = dobj as u32 + 1;
        let Some(k) = self.cues.iter().position(|c| c.dancer == dh && c.f_20 == 0) else { return };
        h.ee_sub_1619a0(0x35, obj as u32 + 1);
        let c = self.cues[k];
        if now < c.t - self.f_58 {
            d.ee_sub_152390(dobj, -1, -1, 1, h);
            self.ee_sub_153640(obj, self.f_298, h);
            return;
        }
        if c.f_1c != 0 {
            d.ee_sub_152390(dobj, -1, -1, 1, h);
            self.ee_sub_153640(obj, self.f_2a0, h);
            return;
        }
        self.cues[k].f_1c = 1;
        if h.ee_call(0x1c4700, &[tracker, c.mask as u32]) == 0 {
            d.ee_sub_152390(dobj, -1, -1, 1, h);
            self.ee_sub_153640(obj, self.f_29c, h);
            return;
        }
        d.ee_sub_152390(dobj, c.mask, c.oid, 0, h);
        let s = self.ee_sub_1535a0(&c, now); // FUN_00153638 = identity
        self.ee_sub_153640(obj, s, h);
        if let Some(b) = self.beats.iter_mut().find(|b| (b[0] - c.t).abs() < 0.1) {
            b[8] = 1.0;
        }
    }

    /// slot 15 FUN_00152970 (FUN_001a5e28 first)
    pub fn ee_sub_152970(&mut self, obj: usize, now: f32, dt: f32, ds: &mut [(usize, Inst)], h: &mut dyn ClassHost) {
        h.ee_sub_1a5e28(obj);
        if self.st == 1 {
            if let Some(k) = self.cues.iter().position(|c| c.f_14 == 0 && c.t < now) {
                let c = self.cues[k];
                if let Some((di, d)) = dancer(ds, c.dancer) {
                    d.ee_sub_152390(di, c.mask, c.oid, 0, h);
                }
                self.cues[k].f_14 = 1;
            }
        } else if self.st == 2 {
            for k in 0..self.cues.len() {
                let c = self.cues[k];
                if c.f_20 != 0 {
                    continue;
                }
                if c.f_18 == 0 && c.t < now {
                    self.f_230 = c.mask;
                    h.ee_sub_1619a0(0x34, obj as u32 + 1);
                    self.cues[k].f_18 = 1;
                }
                if c.t + self.f_5c < now {
                    self.cues[k].f_20 = 1;
                    if c.f_1c == 0 {
                        if let Some((di, d)) = dancer(ds, c.dancer) {
                            d.ee_sub_152390(di, c.mask, c.oid, 2, h);
                        }
                        self.ee_sub_153640(obj, self.f_2a4, h);
                    }
                    break;
                }
            }
        }
        self.f_288 = FUN_0011d018(self.f_288, self.f_284, dt, self.f_28c, None);
        self.ee_sub_152b58(h);
    }

    /// FUN_00152b58: each live crowd link plays the first level whose threshold the smoothed score reaches
    /// (FUN_001f0660(link, FUN_00152680(level))); a finished link is released
    pub fn ee_sub_152b58(&mut self, h: &mut dyn ClassHost) {
        for r in &mut self.crowd {
            if r.link == 0 {
                continue;
            }
            if !h.ee_sub_1f0638(r.link) {
                if let Some(i) = (0..3).find(|&i| r.thr[i] <= self.f_288) {
                    h.ee_sub_1f0660(r.link as usize, ee_sub_152680(i as i32) as i32);
                    r.level = i as i32;
                }
            } else {
                super::k17_55::ee_sub_1f1448(0, 0, -1, 0, &mut r.link, h);
                r.level = -1;
            }
        }
    }

    /// slot 18 FUN_00152c58 (in states 1..2 with beats): the beat cursor - 0.4 s before the next beat it
    /// launches (+0x1b0 = (100, -1000)) and flies on a Hermite arc onto the beat's HUD position (end velocity
    /// (100, 1000)); past the beat it moves to the next one. The HUD text drawing is UNPORTED.
    pub fn ee_sub_152c58(&mut self, now: f32, dt: f32) {
        if !(1..=2).contains(&self.st) || self.beats.is_empty() {
            return;
        }
        let Some(b) = self.beats.get(self.f_194.max(0) as usize).copied() else { return };
        if self.f_190 == 0 {
            if b[0] - now < D_BDBB0 {
                self.f_190 = 1;
                self.f_1b0 = V_BDBC0;
            }
        } else if self.f_190 == 1 && b[0] < now {
            self.f_190 = 0;
            self.f_194 += 1;
        }
        if self.f_190 == 1 {
            let t = b[0] - now;
            let t = if dt < t { t } else { dt };
            let (p, v, _) = ee_sub_117120(t, dt, 1.0, self.f_1a0, self.f_1b0, [b[4], b[5], 0.0], V_BDBD0);
            self.f_1a0 = p;
            self.f_1b0 = v;
        }
    }
}

impl super::ClassSet {
    pub(super) fn build_k22_121(&mut self, w: &crate::engine::world::World, h: &mut dyn ClassHost) {
        for (i, o) in w.objects.iter().enumerate() {
            if o.class != 121 {
                continue;
            }
            let mut c = Box::<C2cd5b8>::default();
            c.ee_sub_1526d8(i, h);
            if let Some(p) = w.objects.get(o.clone_of.unwrap_or(i)).and_then(crate::engine::game::fields::object_props) {
                c.feed(p);
            }
            self.k22c.push((i, Inst::K22C121(c)));
        }
    }
    pub(super) fn post_load_k22_121(&mut self, h: &mut dyn ClassHost) {
        for (i, x) in &mut self.k22c {
            if let Inst::K22C121(c) = x {
                c.ee_sub_1527c8(*i, h);
            }
        }
        // the deferred member call
        for (i, x) in &mut self.k22c {
            if let Inst::K22C121(c) = x {
                c.ee_sub_152810(*i, h);
            }
        }
    }
    pub(super) fn frame_k22_121(&mut self, h: &mut dyn ClassHost) {
        let (now, dt) = (h.k22_music_1fe3f8(), self.g.dt);
        for (i, x) in &mut self.k22c {
            if let Inst::K22C121(c) = x {
                c.ee_sub_152970(*i, now, dt, &mut self.k21c, h);
                c.ee_sub_152c58(now, dt);
            }
        }
    }
    /// `DAT_002de570`: the (last) rhythm manager
    pub fn k22_rhythm(&mut self) -> Option<(usize, &mut C2cd5b8)> {
        self.k22c.iter_mut().rev().find_map(|(i, x)| match x {
            Inst::K22C121(c) => Some((*i, &mut **c)),
            _ => None,
        })
    }
}
