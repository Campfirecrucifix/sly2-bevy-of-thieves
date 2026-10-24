//! WP-K15: class 85 C2d8738 (parent class 84, an abstract filter layer of class 48 with the rows 448 / 449):
//! a laser-beam set. Up to 64 beams (0x150 each, +0x198 count / +0x19c array, row 455 FUN_001df908) between two
//! endpoints (0xa0 each): kind 0 a point fixed in the object's frame (+0x60), kind 1 a point moving along a
//! class-10 path (piecewise motion segments: mode 8 loop, 0 jump back, 0x10 eased ping-pong). A beam's +0x148
//! contact list (an object crossing it, slot 59) clips it to the nearest crossing and zaps that object
//! (FUN_001f7598 kind 0x1000, amount -10).
//! Own slots 6 FUN_001de400 (+ FUN_001de2a0), 13 FUN_001de478, 14 FUN_001de568, 15 FUN_001df3a0, 40
//! FUN_001de518, 52 FUN_001de5a8, 53 FUN_001deea8, 59 FUN_001df0a0, 60 FUN_001de2d8 and FUN_001de5d8 / 1def98 /
//! 1df260 / 1df3d0 / 1df908 / 1dedb0 1:1 apart from:
//! REPLACED: FUN_001de5d8's bbox + collision geometry (+0xe4, FUN_001ae2a0 / FUN_00166270) -> none (the beam
//! test is `ClassHost::beam_hit_14c6a0`); the SO moving-list add of FUN_001df3d0 (FUN_0011a220(0x3e11c4)) ->
//! dropped; FUN_001dedb0's endpoint fx anim lookups (FUN_0015e9c8 class 0x6d / FUN_001f6788 / FUN_00130488 /
//! FUN_0012ff10) -> the fx handles only; slot 59's FUN_0014c6a0 / FUN_00205670 segment tests ->
//! `ClassHost::beam_hit_14c6a0`; FUN_001a7c18's contact record -> `Beam::contacts` of the crossed beam; the
//! callers of slots 59 / 53 (SO overlap begin / end) -> `ClassHost::so_overlaps_85` diffed per frame; slot 18
//! FUN_001df5b0 (beam / endpoint fx draw, FUN_001a5ee8 / FUN_001a5fb8) -> `ClassHost::beam_draw_85` with the
//! clipped segments; FUN_00245a18 fmodf by a zero segment end (mode 0's second segment; NaN on the PS2) -> 0;
//! FUN_001a62e8's z (lost in the decompile, read as FLT_MAX) -> the full position; hit record +4 (contact
//! word 0) dropped; FUN_001df260 with no contact nearer than the beam (null deref in the original) -> no hit.
//! Paths: FUN_001e6648 / FUN_001e6f10 -> `ClassHost::path_len_1e6648` / `path_point_1e6f10`.
#![allow(non_snake_case)]
use super::k15_79::method_args;
use super::*;
use crate::engine::anim::pose::{ee_sub_11c318, Mat4};

/// slot 5 FUN_00113f60 (class descriptor getter)
pub fn slot5(class: ClassId) -> Option<u32> {
    match class {
        85 => Some(0x2bb18c),
        _ => None,
    }
}

/// DAT_002c3610: mode-0x10 ease length cap
pub const R_2C3610: f32 = 100.0;
/// beam hit record kind / amount (FUN_001df260)
pub const HIT_KIND: i32 = 0x1000;
pub const HIT_AMOUNT: i32 = -10;
/// FUN_001def98 (+0x60 frame) point transform
fn pt(m: &Mat4, p: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|r| m[0][r] * p[0] + m[1][r] * p[1] + m[2][r] * p[2] + m[3][r])
}
fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// a motion segment (+0x1c + i * 0x14): distance p0 + v t + a t^2 / 2 while t < tend
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Seg {
    pub p0: f32,
    pub v: f32,
    pub a: f32,
    pub tend: f32,
    pub t: f32,
}

/// an endpoint (0xa0)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct End {
    /// +0 oid, +4 kind (-1 / 0 fixed / 1 path)
    pub oid: i32,
    pub kind: i32,
    /// kind 0: +0x10 local point; kind 1: +0x10 path, +0x14 distance
    pub local: [f32; 3],
    pub path: u32,
    pub d: f32,
    /// +0x18 .. +0x24 row-541 words (fraction, speed, duration, mode)
    pub prm: [f32; 3],
    pub mode: i32,
    /// +0x18 count (kind 1), +0x1c segments (6 max), +0x94 current
    pub segs: Vec<Seg>,
    pub cur: usize,
}

/// a beam crossing (the FUN_001a7c18 kind-10 record): +0x4c crossing object, +0x10 point, +0xa0 distance
#[derive(Clone, Debug, PartialEq)]
pub struct BeamContact {
    pub other: u32,
    pub pos: [f32; 3],
    pub d: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Beam {
    pub ends: [End; 2],
    /// +0x140 full length, +0x144 clipped length
    pub len: f32,
    pub clip: f32,
    /// +0x148 contacts
    pub contacts: Vec<BeamContact>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct C2d8738 {
    /// +0x40 flags, +0x148 flags (bit 34 / 35)
    pub f_40: u32,
    pub f_148: u64,
    /// +0x150 / +0x154 oid filter (row 448), +0x170 / +0x174 class filter (row 449)
    pub oids: Vec<i32>,
    pub classes: Vec<i32>,
    /// +0x190 / +0x194 (rows 450 / 451) beam fx, +0x1a0 / +0x1b0 (rows 452 / 453) endpoint fx
    pub f_190: u32,
    pub f_194: u32,
    pub fx: [u32; 2],
    /// +0x1c0 motion rate (row 454)
    pub rate: f32,
    /// +0x198 / +0x19c beams
    pub beams: Vec<Beam>,
    /// SO +0x70 |= 0x20 (slot 52)
    pub so_70: u32,
    /// FUN_00149ab8(PMF 0x2a7ac0 -> FUN_001df260) queued
    pub pending: bool,
    /// objects whose slot 59 ran (overlap begun)
    pub touching: Vec<u32>,
}

fn handle(obj: usize) -> u32 {
    obj as u32 + 1
}
fn f(w: u32) -> f32 {
    f32::from_bits(w)
}

impl C2d8738 {
    /// slot 6 FUN_001de400: FUN_001de2a0 (FUN_001f2108, +0x148 |= 1 << 34); +0x148 &= !(1 << 35); rate 1;
    /// +0x40 |= 8
    pub fn ee_sub_1de400(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1f2108, obj);
        self.f_148 |= 1 << 34;
        self.f_148 &= !(1 << 35);
        self.rate = 1.0;
        self.f_40 |= 8;
    }

    /// row 455 FUN_001df908(a1..a3, a6..a8, this, a0, a4, a5, a9): a beam (64 max)
    pub fn ee_sub_1df908(&mut self, a: &[u32]) {
        let g = |i: usize| a.get(i).copied().unwrap_or(0);
        if self.beams.len() >= 0x40 {
            return;
        }
        let end = |oid: u32, p: [u32; 4]| End { oid: oid as i32, prm: [f(p[0]), f(p[1]), f(p[2])], mode: p[3] as i32, ..Default::default() };
        self.beams.push(Beam { ends: [end(g(0), [g(1), g(2), g(3), g(4)]), end(g(5), [g(6), g(7), g(8), g(9)])], ..Default::default() });
    }

    /// slot 14 FUN_001de568: FUN_001f36a8; FUN_001de5d8; FUN_001dedb0; FUN_001df260
    pub fn ee_sub_1de568(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1f36a8, obj);
        self.ee_sub_1de5d8(obj, h);
        self.ee_sub_1dedb0(obj, h);
        self.ee_sub_1df260(obj, h);
    }

    /// FUN_001de5d8: per endpoint (FUN_001f2ba8 first) the oid (FUN_0015e938 kind 1): a class-10 path with
    /// a speed / duration -> kind 1 motion, without -> kind 0 at the path point; a class-3 / 0x30 object ->
    /// kind 0 at its position; none -> kind 0 at the own position. Kind 0 points stored in the own frame.
    pub fn ee_sub_1de5d8(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1f2ba8, obj);
        let me = handle(obj);
        let inv = ee_sub_11c318(&h.obj_mat(me, 0x60));
        for bi in 0..self.beams.len() {
            for ei in 0..2 {
                let mut set_40 = false;
                let e = &mut self.beams[bi].ends[ei];
                e.kind = -1;
                let mut world: Option<[f32; 3]> = None;
                let o = if e.oid == -1 { 0 } else { h.ee_sub_15e938(1, e.oid, me) };
                if o != 0 {
                    if h.ee_sub_132b10(o, 10) != 0 {
                        let t = e.prm[0].clamp(0.0, 1.0);
                        let rev = e.prm[1] < 0.0;
                        let mut s = e.prm[1].abs();
                        let mut dur = e.prm[2].abs();
                        let mut mode = e.mode;
                        if mode != 8 && mode != 0x10 && mode != 0 {
                            mode = 8;
                        }
                        let l = h.path_len_1e6648(o);
                        if 0.0 < dur {
                            s = l / dur;
                        } else if 0.0 < s {
                            dur = l / s;
                        }
                        let d0 = t * l;
                        if s != 0.0 && dur != 0.0 {
                            e.path = o;
                            e.kind = 1;
                            e.d = d0;
                            e.segs.clear();
                            e.cur = 0;
                            let tt = if rev { 1.0 - t } else { t };
                            match mode {
                                8 => {
                                    e.segs.push(Seg { p0: if rev { l } else { 0.0 }, v: if rev { -s } else { s }, a: 0.0, tend: dur, t: tt * dur });
                                    set_40 = true;
                                }
                                0 => {
                                    e.segs.push(Seg { p0: if rev { l } else { 0.0 }, v: if rev { -s } else { s }, a: 0.0, tend: dur, t: tt * dur });
                                    e.segs.push(Seg { p0: if rev { 0.0 } else { l }, ..Default::default() });
                                    set_40 = true;
                                }
                                0x10 => {
                                    let ez = if R_2C3610 * 3.0 <= l { R_2C3610 } else { l * 0.2 };
                                    let e2 = ez + ez;
                                    let ta = (e2 * dur) / (l + e2);
                                    let tc = dur - (ta + ta);
                                    let vc = (l - e2) / tc;
                                    let acc = (vc * vc) / e2;
                                    e.segs = vec![
                                        Seg { p0: 0.0, v: 0.0, a: acc, tend: ta, t: 0.0 },
                                        Seg { p0: ez, v: vc, a: 0.0, tend: tc, t: 0.0 },
                                        Seg { p0: l - ez, v: vc, a: -acc, tend: ta, t: 0.0 },
                                        Seg { p0: l, v: 0.0, a: -acc, tend: ta, t: 0.0 },
                                        Seg { p0: l - ez, v: -vc, a: 0.0, tend: tc, t: 0.0 },
                                        Seg { p0: ez, v: -vc, a: acc, tend: ta, t: 0.0 },
                                    ];
                                    // the start segment: the first of 0..2 whose end (next p0) lies past d0
                                    let mut i = 0;
                                    loop {
                                        let (sg, nx) = (e.segs[i], e.segs[i + 1]);
                                        if d0 < nx.p0 {
                                            let tm = ((d0 - sg.p0) + (d0 - sg.p0)) / (sg.v + nx.v);
                                            if rev {
                                                let j = 5 - i;
                                                e.cur = j;
                                                e.segs[j].t = e.segs[j].tend - tm;
                                            } else {
                                                e.cur = i;
                                                e.segs[i].t = tm;
                                            }
                                            break;
                                        }
                                        e.cur = i + 1;
                                        i += 1;
                                        if i >= 3 {
                                            break;
                                        }
                                    }
                                    set_40 = true;
                                }
                                _ => {}
                            }
                        } else {
                            e.kind = 0;
                            world = Some(h.path_point_1e6f10(o, d0));
                        }
                    } else if h.ee_sub_132b10(o, 3) != 0 || h.ee_sub_132b10(o, 0x30) != 0 {
                        e.kind = 0;
                        world = Some(h.ee_sub_1a62e8(o));
                    }
                }
                if e.kind == -1 {
                    e.kind = 0;
                    world = Some(h.ee_sub_1a62e8(me));
                }
                if e.kind == 0 {
                    if let Some(p) = world {
                        e.local = pt(&inv, p);
                    }
                }
                if set_40 {
                    self.f_40 |= 2;
                }
            }
        }
    }

    /// FUN_001dedb0: the endpoint fx handles (anim lookups REPLACED)
    pub fn ee_sub_1dedb0(&mut self, _obj: usize, _h: &mut dyn ClassHost) {}

    /// FUN_001def98(self, beam, out, len): both endpoints in world space and the distance
    pub fn ee_sub_1def98(&self, obj: usize, b: &Beam, h: &mut dyn ClassHost) -> ([f32; 3], [f32; 3], f32) {
        let m = h.obj_mat(handle(obj), 0x60);
        let mut p = [[0.0; 3]; 2];
        for (i, e) in b.ends.iter().enumerate() {
            if e.kind == 0 {
                p[i] = pt(&m, e.local);
            } else if e.kind == 1 {
                p[i] = h.path_point_1e6f10(e.path, e.d);
            }
        }
        (p[0], p[1], dist(p[0], p[1]))
    }

    /// FUN_001df260: per beam the length; with contacts the nearest one (closer than the beam) gets the hit
    /// record (FUN_001f7598) and clips the beam
    pub fn ee_sub_1df260(&mut self, obj: usize, h: &mut dyn ClassHost) {
        for bi in 0..self.beams.len() {
            let (_, _, len) = self.ee_sub_1def98(obj, &self.beams[bi], h);
            let b = &mut self.beams[bi];
            if b.contacts.is_empty() {
                b.len = len;
                b.clip = len;
                continue;
            }
            let mut best: Option<&BeamContact> = None;
            let mut d = len;
            for c in &b.contacts {
                if c.d < d {
                    d = c.d;
                    best = Some(c);
                }
            }
            if let Some(c) = best {
                let hit = guard::slots::GuardHit { src: handle(obj), target: c.other, kind: HIT_KIND, amount: HIT_AMOUNT, pos: [c.pos[0], c.pos[1], c.pos[2], 1.0], ..Default::default() };
                h.ee_sub_1f7598(&hit);
            }
            b.clip = d;
        }
    }

    /// slot 15 FUN_001df3a0: FUN_001f36e0; FUN_001df3d0 (path endpoints advance by rate * dt; a beam that
    /// moved and has no contacts gets its full length)
    pub fn ee_sub_1df3a0(&mut self, obj: usize, dt: f32, h: &mut dyn ClassHost) {
        h.base(0x1f36e0, obj);
        for bi in 0..self.beams.len() {
            let mut moved = false;
            for e in &mut self.beams[bi].ends {
                if e.kind != 1 || e.segs.is_empty() {
                    continue;
                }
                let n = e.segs.len();
                let s = &mut e.segs[e.cur];
                s.t += self.rate * dt;
                if s.tend < s.t {
                    // FUN_00245a18 fmodf (a zero end: 0, see module doc)
                    let r = if s.tend == 0.0 { 0.0 } else { s.t % s.tend };
                    e.cur = (e.cur + 1) % n;
                    e.segs[e.cur].t = r;
                }
                let s = e.segs[e.cur];
                e.d = s.p0 + s.v * s.t + s.a * s.t * s.t * 0.5;
                moved = true;
            }
            if moved && self.beams[bi].contacts.is_empty() {
                let (_, _, len) = self.ee_sub_1def98(obj, &self.beams[bi], h);
                self.beams[bi].len = len;
                self.beams[bi].clip = len;
            }
        }
    }

    /// slot 52 FUN_001de5a8: SO +0x70 |= 0x20
    pub fn ee_sub_1de5a8(&mut self) {
        self.so_70 |= 0x20;
    }

    /// slot 60 FUN_001de2d8(self, other): 2 (FUN_00134310(r, 2) = max) when `other` carries a listed oid
    /// (FUN_001a6290) or is-a a listed class, else 1
    pub fn ee_sub_1de2d8(&self, other: u32, h: &mut dyn ClassHost) -> u32 {
        let mut r = 1;
        for &o in &self.oids {
            if h.ee_sub_1a6290(other, o) {
                r = r.max(2);
            }
        }
        for &c in &self.classes {
            if h.ee_sub_132b10(other, c) != 0 {
                r = r.max(2);
            }
        }
        r
    }

    /// slot 59 FUN_001df0a0(self, other): FUN_001f5990; each beam the segment test against `other` (REPLACED
    /// hook) -> a contact at the crossing point with the distance from endpoint 0; then FUN_001df260 deferred
    pub fn ee_sub_1df0a0(&mut self, obj: usize, other: u32, h: &mut dyn ClassHost) {
        for bi in 0..self.beams.len() {
            let (p0, p1, _) = self.ee_sub_1def98(obj, &self.beams[bi], h);
            if let Some(p) = h.beam_hit_14c6a0(other, p0, p1) {
                self.beams[bi].contacts.push(BeamContact { other, pos: p, d: dist(p0, p) });
                self.pending = true;
            }
        }
    }

    /// slot 53 FUN_001deea8(self, other): FUN_001f5988; every beam drops its contacts with `other`
    /// (FUN_001a7e10 frees them); FUN_001df260 deferred
    pub fn ee_sub_1deea8(&mut self, other: u32) {
        for b in &mut self.beams {
            b.contacts.retain(|c| c.other != other);
        }
        self.pending = true;
    }

    /// slot 18 FUN_001df5b0 (REPLACED): the clipped beam segments
    pub fn ee_sub_1df5b0(&self, obj: usize, h: &mut dyn ClassHost) -> Vec<([f32; 3], [f32; 3])> {
        let mut v = Vec::new();
        for b in &self.beams {
            let (p0, mut p1, len) = self.ee_sub_1def98(obj, b, h);
            if b.clip < len {
                let k = b.clip / len;
                p1 = std::array::from_fn(|i| p0[i] * (1.0 - k) + p1[i] * k);
            }
            v.push((p0, p1));
        }
        v
    }

    /// rows 454 (+0x1c0), 448 / 449 (filter lists), 450 .. 453 (fx objects) and method 455 (FUN_001df908)
    pub fn feed(&mut self, props: &[crate::engine::world::props::Prop]) {
        use super::k4::word;
        use crate::engine::world::props::PropValue;
        for p in props {
            match p.field {
                455 => self.ee_sub_1df908(&method_args(&p.value)),
                448 => self.oids.extend(method_args(&p.value).into_iter().map(|x| x as i32)),
                449 => self.classes.extend(method_args(&p.value).into_iter().map(|x| x as i32)),
                _ => {
                    let PropValue::Scalar(b) = &p.value else { continue };
                    let w = word(b);
                    match p.field {
                        454 => self.rate = f(w),
                        450 => self.f_190 = w,
                        451 => self.f_194 = w,
                        452 => self.fx[0] = w,
                        453 => self.fx[1] = w,
                        _ => {}
                    }
                }
            }
        }
    }
}

impl ClassSet {
    /// K15 class 85: construct + init (slot 6) + feed; the object rows resolved to handles
    pub(super) fn build_k15_85(&mut self, w: &World, h: &mut dyn ClassHost) {
        for (i, o) in w.objects.iter().enumerate() {
            if o.class != 85 {
                continue;
            }
            let src = super::clone_src(w, i);
            let props = w.objects.get(src).and_then(crate::engine::game::fields::object_props);
            let mut c = C2d8738::default();
            c.ee_sub_1de400(i, h);
            if let Some(p) = props {
                c.feed(p);
            }
            let [fx0, fx1] = &mut c.fx;
            for x in [&mut c.f_190, &mut c.f_194, fx0, fx1] {
                if *x != 0 {
                    *x = h.ee_sub_15e938(0x2002, *x as i32, handle(i));
                }
            }
            self.k15.push((i, Inst::K15e(Box::new(c))));
        }
    }
    /// K15 class-85 state at object index `obj`
    pub fn k15_85_get(&mut self, obj: usize) -> Option<&mut C2d8738> {
        self.k15.iter_mut().find(|x| x.0 == obj).and_then(|x| if let Inst::K15e(c) = &mut x.1 { Some(&mut **c) } else { None })
    }
    /// K15 class 85 load-post: slot 13 (FUN_001f31e0), slot 14
    pub(super) fn post_load_k15_85(&mut self, h: &mut dyn ClassHost) {
        for (i, x) in &mut self.k15 {
            if let Inst::K15e(c) = x {
                h.base(0x1f31e0, *i);
                c.ee_sub_1de568(*i, h);
            }
        }
    }
    /// K15 class 85 per frame: slot 15, the overlap begin / end (slots 59 / 53), the deferred FUN_001df260,
    /// the beam draw
    pub(super) fn frame_k15_85(&mut self, h: &mut dyn ClassHost) {
        let dt = self.g.dt;
        for (i, x) in &mut self.k15 {
            let Inst::K15e(c) = x else { continue };
            let i = *i;
            // CRAWL: an LO removed by the level script (FUN_001a5630) is out of the update list
            if h.lo_removed(i) {
                continue;
            }
            c.ee_sub_1df3a0(i, dt, h);
            let q = h.ee_sub_1a62e8(i as u32 + 1);
            // SO overlap (REPLACED +0xe4 beam geometry): candidates whose position is within reach of a beam
            let full: Vec<_> = c.beams.iter().map(|b| c.ee_sub_1def98(i, b, h)).collect();
            let mut now = h.so_overlaps_85(i);
            now.retain(|&o| {
                let q = h.ee_sub_1a62e8(o);
                full.iter().any(|(p0, p1, _)| seg_capsule_hit(*p0, *p1, q, 120.0, 30.0).is_some())
            });
            for o in std::mem::take(&mut c.touching) {
                if now.contains(&o) {
                    c.touching.push(o);
                } else {
                    c.ee_sub_1deea8(o);
                }
            }
            for &o in &now {
                if !c.touching.contains(&o) && c.ee_sub_1de2d8(o, h) == 1 {
                    c.touching.push(o);
                    c.ee_sub_1df0a0(i, o, h);
                }
            }
            if std::mem::take(&mut c.pending) {
                c.ee_sub_1df260(i, h);
            }
            let segs = c.ee_sub_1df5b0(i, h);
            h.beam_draw_85(i, &segs);
        }
    }
}

/// WIRE-OBJ: segment p0 -> p1 against the vertical capsule (base `q`, height `hgt`, radius `r`): the beam point
/// closest to the capsule axis when within `r` (closest points of two segments)
pub fn seg_capsule_hit(p0: [f32; 3], p1: [f32; 3], q: [f32; 3], hgt: f32, r: f32) -> Option<[f32; 3]> {
    let d1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
    let d2 = [0.0, 0.0, hgt];
    let rr = [p0[0] - q[0], p0[1] - q[1], p0[2] - q[2]];
    let dot = |a: [f32; 3], b: [f32; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    let (a, e, f) = (dot(d1, d1), dot(d2, d2), dot(d2, rr));
    if a <= 1e-6 {
        return None;
    }
    let (b, c) = (dot(d1, d2), dot(d1, rr));
    let den = a * e - b * b;
    let mut s = if den > 1e-6 { ((b * f - c * e) / den).clamp(0.0, 1.0) } else { 0.0 };
    let mut t = (b * s + f) / e;
    if t < 0.0 {
        t = 0.0;
        s = (-c / a).clamp(0.0, 1.0);
    } else if t > 1.0 {
        t = 1.0;
        s = ((b - c) / a).clamp(0.0, 1.0);
    }
    let c1 = [p0[0] + d1[0] * s, p0[1] + d1[1] * s, p0[2] + d1[2] * s];
    let c2 = [q[0], q[1], q[2] + hgt * t];
    let d = [c1[0] - c2[0], c1[1] - c2[1], c1[2] - c2[2]];
    (dot(d, d) <= r * r).then_some(c1)
}
