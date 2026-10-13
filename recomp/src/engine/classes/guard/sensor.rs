//! Guard sensor (class 0x10 object at guard +0xf64, vtable 0x2cdba0): entry list + vision test
//! (FUN_00153de0 / FUN_001541d8 / FUN_00154080 / FUN_00227c90). Engine calls go through `ClassHost` hooks.
use super::super::ClassHost;
use crate::engine::gfx::scene::Affine;
use crate::engine::physics::geom::{BspC, CGeom};
use crate::engine::physics::point::FUN_00118fb0;

/// A level volume (class-22 VOL, FUN_00227bb8) compiled for FUN_00227c90.
#[derive(Clone, Debug)]
pub struct Vol {
    /// object index
    pub obj: usize,
    /// world -> volume space (FUN_001a6f00(0, v, p)) at the guard's rest pose
    pub inv: Affine,
    pub geom: CGeom,
    pub bsp: BspC,
    /// +0x110 bounding-sphere centre, +0x120 radius^2 (computed from the hull vertices)
    pub c: [f32; 3],
    pub r2: f32,
}

fn xf(m: &Affine, p: [f32; 3]) -> [f32; 3] {
    [0, 1, 2].map(|j| p[0] * m[0][j] + p[1] * m[1][j] + p[2] * m[2][j] + m[3][j])
}
fn d2(a: [f32; 3], b: [f32; 3]) -> f32 {
    (0..3).map(|i| (a[i] - b[i]) * (a[i] - b[i])).sum()
}

impl Vol {
    pub fn new(obj: usize, inv: Affine, geom: CGeom, bsp: BspC) -> Vol {
        let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
        for v in &geom.verts {
            for i in 0..3 {
                lo[i] = lo[i].min(v[i]);
                hi[i] = hi[i].max(v[i]);
            }
        }
        let c = if geom.verts.is_empty() { [0.0; 3] } else { [0, 1, 2].map(|i| (lo[i] + hi[i]) * 0.5) };
        let r2 = geom.verts.iter().map(|v| d2(*v, c)).fold(0.0, f32::max);
        Vol { obj, inv, geom, bsp, c, r2 }
    }
}

/// FUN_00227c90(v, p): point-in-volume. `p` is already in the volume owner's rest frame (the guard's motion
/// since load undone by the caller, = the bone chain of FUN_001a6f00).
pub fn ee_sub_227c90(v: &Vol, p: [f32; 3]) -> bool {
    let l = xf(&v.inv, p);
    if d2(l, v.c) > v.r2 {
        return false;
    }
    FUN_00118fb0(&v.geom, &v.bsp, 0, v.bsp.nodes.len(), l).is_some()
}

/// One +0x100 entry {obj, state, tStateChanged, tFirstNoticed}.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Entry {
    pub obj: u32,
    /// 0 unseen, 1 seen, 2 tracked (outer volume only)
    pub state: i32,
    pub t: f32,
    pub t_first: f32,
}

/// Target facts the vision test needs (resolved by the guard layer).
#[derive(Clone, Copy, Debug, Default)]
pub struct Tgt {
    pub pos: [f32; 3],
    /// is-a 0x32 and vt slot 76 (+0x26c) == 2
    pub c32_hidden: bool,
    /// is-a 0x33 / 0x34
    pub c33: bool,
    pub c34: bool,
    /// is-a 0x34 && +0x11ac (player undetectable)
    pub undetectable: bool,
    /// FUN_001a61a0(t): +0x28 bit 60 and in the 0x3dd4bc set (resolved by the guard layer)
    pub active: bool,
    /// is-a 0x34: attach / cover state for FUN_00154770 (None = +0x4cc == -1, not attached)
    pub cover: Option<super::los::Cover>,
}

#[derive(Clone, Debug, Default)]
pub struct Sensor {
    /// +0xe0 eye owner (guard handle)
    pub f_e0: u32,
    /// +0xe8 vision volume / +0xec outer volume
    pub f_e8: Option<Vol>,
    pub f_ec: Option<Vol>,
    /// +0xf0 always-detect, +0xf4 disabled, +0xf8 flags
    pub f_f0: i32,
    pub f_f4: i32,
    pub f_f8: u32,
    /// +0xfc count / +0x100 entries (cap 8)
    pub entries: Vec<Entry>,
}

/// Inputs of one sensor pass.
pub struct See<'a> {
    pub now: f32,
    /// FUN_00153e60 eye (guard +80 up, DAT_002a2dc0), sensor position (FUN_001a62e8(s) = the guard's)
    pub eye: [f32; 3],
    pub pos: [f32; 3],
    /// world -> guard rest frame
    pub to_rest: &'a dyn Fn([f32; 3]) -> [f32; 3],
    pub tgt: &'a dyn Fn(u32) -> Option<Tgt>,
    /// is-a 0x46 of the current player (FUN_001c4840)
    pub player_c46: bool,
}

impl Sensor {
    /// FUN_00154180(s, o): entry of `o`
    pub fn ee_sub_154180(&self, o: u32) -> Option<usize> {
        self.entries.iter().position(|e| e.obj == o)
    }
    /// FUN_00153f18 / FUN_00153ef8: add a static entry {state 0, t = now}
    pub fn ee_sub_153f18(&mut self, o: u32, now: f32) {
        if o != 0 && self.ee_sub_154180(o).is_none() && self.entries.len() < 8 {
            self.entries.push(Entry { obj: o, state: 0, t: now, t_first: 0.0 });
        }
    }

    /// FUN_00153de0: evaluate every entry
    pub fn ee_sub_153de0(&mut self, guard: usize, s: &See, h: &mut dyn ClassHost) {
        for k in 0..self.entries.len() {
            let e = self.entries[k];
            let n = self.ee_sub_1541d8(guard, e.obj, e.state, s, h);
            self.ee_sub_154080(guard, k, n, s.now, h);
        }
    }

    /// FUN_001541d8(s, t, prev): vision state of `t` (return values from the disassembly, R1 1541d8.s)
    pub fn ee_sub_1541d8(&self, guard: usize, t: u32, prev: i32, s: &See, h: &mut dyn ClassHost) -> i32 {
        let Some(tg) = (if t != 0 { (s.tgt)(t) } else { None }) else { return 0 };
        if !tg.active {
            return 0;
        }
        if tg.c32_hidden || (tg.c34 && s.player_c46) || h.ee_sub_20f1b8() || self.f_f4 != 0 || tg.undetectable {
            return 0;
        }
        if self.f_f0 != 0 {
            return 1;
        }
        let mut tp = tg.pos;
        if tg.c33 {
            let a = h.aim_240(t);
            tp = [tp[0] + a[0], tp[1] + a[1], tp[2] + a[2]];
        }
        if h.ee_sub_118668_blockers(s.eye, tp) {
            return 0;
        }
        let rp = (s.to_rest)(tp);
        let inn = self.f_e8.as_ref().is_some_and(|v| ee_sub_227c90(v, rp));
        let out = self.f_ec.as_ref().is_none_or(|v| ee_sub_227c90(v, rp));
        let f8 = self.f_f8;
        // FUN_001545f0 aims at the target origin FUN_001a6f00(t, 0, 0), not the +0x240 aim point
        let pt = tg.pos;
        let los = |h: &mut dyn ClassHost| h.ee_sub_1545f0(guard, s.eye, pt, f8);
        match prev {
            0 => {
                // the +0xe4 forced list (guard +0xec contact nodes) is not modelled (empty)
                if !inn || !out {
                    return 0;
                }
                if f8 & 1 == 0 && tg.c34 && tg.cover.is_some_and(|c| super::los::ee_sub_154770(&c, tg.pos, s.pos)) {
                    return 0;
                }
                if f8 & 2 != 0 {
                    return 1;
                }
                (!los(h)) as i32
            }
            1 => {
                if !out {
                    0
                } else if !inn {
                    2
                } else if f8 & 8 != 0 {
                    if los(h) { 0 } else { 1 }
                } else {
                    1
                }
            }
            2 => {
                if !out {
                    0
                } else if !inn {
                    if f8 & 8 != 0 && los(h) { 0 } else { 2 }
                } else if f8 & 2 != 0 {
                    1
                } else if los(h) {
                    2
                } else {
                    1
                }
            }
            p => p,
        }
    }

    /// FUN_00154080(s, e, new): state change + script event 0x1c (+ FUN_001619a0(0x2deb30, 0x13, s+0x10)
    /// when the owner is-a 0x33 and the state becomes 1)
    pub fn ee_sub_154080(&mut self, guard: usize, k: usize, new: i32, now: f32, h: &mut dyn ClassHost) {
        let Some(e) = self.entries.get_mut(k) else { return };
        if new == e.state {
            return;
        }
        if e.state == 0 {
            e.t_first = now;
        }
        let n = h.sensor_event_1c(guard, e.obj, e.state, new);
        e.state = n;
        e.t = now;
        if n == 1 {
            // s+0x10 = sensor owner (the guard, is-a 0x33)
            h.ee_sub_1619a0(0x13, guard as u32 + 1);
        }
    }
}
