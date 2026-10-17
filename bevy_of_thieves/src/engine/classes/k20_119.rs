//! WP-K20: class 119 C2cfa90 (vtable 0x2cfa90, parent LO 0x1a5270): the guard spawner. Spawn points (+0x50 /
//! +0x54, 0x1c records {kind, oid, obj, kind-1 helper, its +0x54, busy +0x14, ready time +0x18}, field rows 1324..1326),
//! spawn slots (+0x5c count, +0x60, 0x30 records {guard, last active +4, free time +8, point +0xc, state +0x10,
//! pos +0x20}), live count +0x58.
//! 1:1: slot 6 FUN_00173158, slot 11 FUN_001746b8, slot 14 FUN_001733f8 (the kind-1 helper object
//! FUN_0013e3c8(0x71, 0, 0x2cb) + FUN_00170c50 = `ee_call`), slot 15 FUN_00173a68, FUN_00173a18 (player
//! position), FUN_00173d18 (slot upkeep), FUN_00173e78 (slot choice), FUN_00173928 (free a slot).
//! HOOKS: FUN_00173f18 (spawn-point choice by distance / visibility, 1400 B of VU) = `po_call(0x173f18)` ->
//! point index + 1 (0 = none); FUN_00174490 (clone + place the guard) = `po_call(0x174490, .., [point])` ->
//! guard handle (0 = none); FUN_001f8240 (the player's alert record), FUN_001f7d10 / 1f7c68 / 154038 = `ee_call`.
//! The default hosts return 0 for the two spawn hooks, so no guard is spawned outside tests.
#![allow(non_snake_case)]
use super::k17_55::Mem;
use super::*;

fn handle(obj: usize) -> u32 {
    obj as u32 + 1
}

/// slot 5 FUN_00114180
pub const SLOT5: u32 = 0x2bb434;
/// DAT_002be5cc: alert age limit in mode 1
pub const F_2BE5CC: f32 = 10.0;
/// DAT_002be5d0: spawn retry period
pub const F_2BE5D0: f32 = 1.0;
/// DAT_002be5d4: player must move this far when every slot is live
pub const F_2BE5D4: f32 = 300.0;
/// DAT_002be5d8: period after a spawn
pub const F_2BE5D8: f32 = 2.5;
/// DAT_002be5dc: a slot freed by "lost" re-arms when the player is farther than this
pub const F_2BE5DC: f32 = 3500.0;
/// DAT_002be5e0: a live guard idle this long may be recycled
pub const F_2BE5E0: f32 = 4.0;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Point {
    pub kind: i32,
    pub oid: i32,
    pub obj: u32,
    pub helper: u32,
    pub helper_54: u32,
    pub busy: bool,
    pub ready: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Slot {
    pub guard: u32,
    pub active: f32,
    pub free: f32,
    pub point: Option<usize>,
    pub state: i32,
    pub pos: [f32; 3],
}

#[derive(Clone, Debug, Default)]
pub struct C2cfa90 {
    pub m: Mem,
    pub points: Vec<Point>,
    pub slots: Vec<Slot>,
    /// +0x94 (field 1328, <= 8) template oids / +0x9c (field 1333, <= 16) probe offsets (vec3), read by
    /// FUN_00173f18 / FUN_00174490 (k21_119.rs)
    pub l94: Vec<u32>,
    pub l9c: Vec<[f32; 3]>,
    /// K21: the +0x94 objects (the class-6 spawner record of each oid, handle; 0 = none)
    pub l94_obj: Vec<u32>,
}

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

impl C2cfa90 {
    /// slot 6 FUN_00173158 (after FUN_001a5270 = `h.base`)
    pub fn ee_sub_173158(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1a5270, obj);
        self.m.set_f(0x80, 5500.0);
        self.m.set_f(0xa4, 50.0);
        self.m.set_i(0x84, 1);
        self.m.set_f(0xa0, 30.0);
        self.m.set_i(0x88, 1);
        self.m.set_i(0x8c, 1);
    }

    /// field rows 1324 / 1325 / 1326 (FUN_001732e0 / 173318 / 173358): add a kind 0 / 1 / 2 point (<= 0x80)
    pub fn add_point(&mut self, kind: i32, oid: i32) {
        if self.points.len() < 0x80 {
            self.points.push(Point { kind, oid, ..Default::default() });
        }
    }

    /// slot 14 FUN_001733f8 (after FUN_001a5c48 = `h.base`): resolve the points (kind 1 also makes its
    /// helper), allocate the +0x5c slots, FUN_001a6ca8(self, 1), +0x40 flag 2
    pub fn ee_sub_1733f8(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1a5c48, obj);
        for p in &mut self.points {
            match p.kind {
                0 | 2 => p.obj = h.ee_sub_15e938(5, p.oid, 0),
                1 => {
                    p.obj = h.ee_sub_15e938(5, p.oid, 0);
                    let x = h.ee_call(0x13e3c8, &[0x71, 0, 0x2cb, u32::MAX]) as u32;
                    h.ee_call(0x170c50, &[x, p.oid as u32, 1, u32::MAX]);
                    p.helper = x;
                    p.helper_54 = if x != 0 { h.obj_i32(x, 0x54) as u32 } else { 0 };
                }
                _ => {}
            }
        }
        self.slots = vec![Slot::default(); self.m.i(0x5c).max(0) as usize];
        h.ee_sub_1a6ca8(obj, 1);
        // ponytail: the inline +0x40 flag-2 set lives in the LO base block, not modelled here
    }

    /// slot 11 FUN_001746b8 (after FUN_001a5848 = `h.base`): free every live slot quietly
    pub fn ee_sub_1746b8(&mut self, obj: usize, t: f32, h: &mut dyn ClassHost) {
        h.base(0x1a5848, obj);
        for k in 0..self.slots.len() {
            if self.slots[k].guard != 0 {
                self.ee_sub_173928(obj, k, 0, false, t, h);
            }
        }
    }

    /// FUN_00173a18: the controlled character's position (DAT_002ddfd0 = 0 without one)
    pub fn ee_sub_173a18(h: &mut dyn ClassHost) -> [f32; 3] {
        let p = h.ee_call(0x1c4840, &[0x3dd490]) as u32;
        if p == 0 {
            [0.0; 3]
        } else {
            h.ee_sub_1a62e8(p)
        }
    }

    /// FUN_00173928(self, slot, k, quiet): k 0 = guard gone (its vt+0x4c, point ready now), 1 = lost (slot
    /// waits +0xa0, remembers the player position, point ready after +0xa4); frees the slot and its point
    pub fn ee_sub_173928(&mut self, _obj: usize, k: usize, s: i32, quiet: bool, t: f32, h: &mut dyn ClassHost) {
        let a0 = self.m.f(0xa0);
        let a4 = self.m.f(0xa4);
        self.slots[k].state = s;
        let pt = self.slots[k].point;
        match s {
            0 => {
                self.slots[k].free = t;
                h.vcall(self.slots[k].guard, 0x4c);
                if !quiet {
                    if let Some(p) = pt {
                        self.points[p].ready = t;
                    }
                }
            }
            1 => {
                self.slots[k].free = t + a0;
                self.slots[k].pos = Self::ee_sub_173a18(h);
                if !quiet {
                    if let Some(p) = pt {
                        self.points[p].ready = t + a4;
                    }
                }
            }
            _ => {}
        }
        let sl = &mut self.slots[k];
        sl.guard = 0;
        if let Some(p) = pt {
            self.points[p].busy = false;
        }
        sl.point = None;
        self.m.set_i(0x58, self.m.i(0x58) - 1);
    }

    /// FUN_00173d18(self, player): re-arm lost slots far from the player; drop guards that are dead (+0x330)
    /// or left the world, else track their last activity (+0x84)
    pub fn ee_sub_173d18(&mut self, obj: usize, p: [f32; 3], t: f32, h: &mut dyn ClassHost) {
        for k in 0..self.slots.len() {
            let g = self.slots[k].guard;
            if g == 0 {
                if self.slots[k].state == 1 && F_2BE5DC < dist(self.slots[k].pos, p) {
                    self.slots[k].state = 0;
                    self.slots[k].free = t;
                }
                continue;
            }
            let s = if h.obj_i32(g, 0x330) != 0 {
                1
            } else if h.obj_flags_28(g) >> 60 & 1 != 0 {
                let po = self.slots[k].point.map(|i| self.points[i].obj).unwrap_or(0);
                if h.obj_flags_28(po) >> 60 & 1 != 0 {
                    let a = f32::from_bits(h.obj_i32(g, 0x84) as u32);
                    let sl = &mut self.slots[k];
                    sl.active = if sl.active < a { a } else { sl.active };
                    continue;
                }
                0
            } else {
                0
            };
            self.ee_sub_173928(obj, k, s, false, t, h);
        }
    }

    /// FUN_00173e78(self, keep): a free slot (earliest free time before now), else the live guard idle
    /// longest beyond DAT_002be5e0 that is not `keep`
    pub fn ee_sub_173e78(&self, keep: u32, t: f32) -> Option<usize> {
        let mut best = None;
        let mut idle = t - F_2BE5E0;
        let mut free = t;
        for (k, s) in self.slots.iter().enumerate() {
            if s.guard == 0 {
                if s.free < free {
                    free = s.free;
                    best = Some(k);
                }
            } else if s.guard != keep {
                if best.is_some_and(|b: usize| self.slots[b].guard == 0) {
                    continue;
                }
                if s.active < idle {
                    idle = s.active;
                    best = Some(k);
                }
            }
        }
        best
    }

    /// slot 15 FUN_00173a68 (after FUN_001a5e28 = `h.base`)
    pub fn ee_sub_173a68(&mut self, obj: usize, t: f32, h: &mut dyn ClassHost) {
        h.base(0x1a5e28, obj);
        let p = Self::ee_sub_173a18(h);
        self.ee_sub_173d18(obj, p, t, h);
        if t < self.m.f(0xa8) {
            return;
        }
        self.m.set_f(0xa8, t + F_2BE5D0);
        if self.m.i(0x58) == self.m.i(0x5c) && dist(p, self.m.v3(0x70)) < F_2BE5D4 {
            return;
        }
        let mut alert = 0u32;
        let pl = h.ee_call(0x1c4840, &[0x3dd490]) as u32;
        if pl != 0 {
            alert = h.ee_call(0x1f8240, &[pl]) as u32;
            if alert != 0 {
                if f32::from_bits(h.obj_i32(alert, 0x28) as u32) == 0.0 {
                    alert = 0;
                } else {
                    match self.m.i(0xac) {
                        1 => {
                            if F_2BE5CC < t - f32::from_bits(h.obj_i32(alert, 0) as u32) {
                                return;
                            }
                        }
                        0 => return,
                        _ => {}
                    }
                }
            }
        }
        let keep = if alert != 0 && h.obj_i32(alert, 0x28) == 1 { h.obj_i32(alert, 0x2c) as u32 } else { 0 };
        let Some(k) = self.ee_sub_173e78(keep, t) else {
            return;
        };
        let pi = h.po_call(0x173f18, obj, 0, &[p[0].to_bits(), p[1].to_bits(), p[2].to_bits()]);
        if pi <= 0 || pi as usize > self.points.len() {
            return;
        }
        let pi = pi as usize - 1;
        self.m.set_v3(0x70, p);
        if self.slots[k].guard != 0 {
            self.ee_sub_173928(obj, k, 0, true, t, h);
        }
        let g = h.po_call(0x174490, obj, 0, &[pi as u32]) as u32;
        if g == 0 {
            return;
        }
        h.ee_call(0x1f7d10, &[0.02f32.to_bits()]);
        h.ee_call(0x1f7c68, &[0.5f32.to_bits(), g]);
        if alert != 0 {
            let f64_ = h.obj_i32(g, 0xf64) as u32;
            let c = h.obj_i32(alert, 0xc) as u32;
            h.ee_call(0x154038, &[f64_, c, 2]);
        }
        let s = &mut self.slots[k];
        s.guard = g;
        s.point = Some(pi);
        s.active = t;
        self.m.set_i(0x58, self.m.i(0x58) + 1);
        self.points[pi].busy = true;
        self.m.set_f(0xa8, t + F_2BE5D8);
        // (vt+0x1c)(self, 0x19, 0, 1, &guard, 0): the "spawned" message
        h.vcall_arg(handle(obj), 0x1c, 0x19);
    }
}

/// K20 class 119 in the ClassSet
impl ClassSet {
    pub(super) fn build_k20_119(&mut self, w: &crate::engine::world::World, h: &mut dyn ClassHost) {
        use crate::engine::game::fields::object_props;
        use crate::engine::world::props::PropValue;
        for (i, o) in w.objects.iter().enumerate() {
            if o.class != 119 {
                continue;
            }
            let mut c = C2cfa90::default();
            c.ee_sub_173158(i, h);
            if let Some(ps) = w.objects.get(o.clone_of.unwrap_or(i)).and_then(object_props) {
                for x in ps {
                    let PropValue::Scalar(b) = &x.value else { continue };
                    let mut a = [0u8; 4];
                    a[..b.len().min(4)].copy_from_slice(&b[..b.len().min(4)]);
                    let v = u32::from_le_bytes(a);
                    let oid = if b.len() == 2 { (v & 0xffff) as i32 } else { v as i32 };
                    match x.field {
                        1324 => c.add_point(0, oid),
                        1325 => c.add_point(1, oid),
                        1326 => c.add_point(2, oid),
                        1328 if c.l94.len() < 8 => c.l94.push(v),
                        1333 if c.l9c.len() < 16 && b.len() >= 12 => {
                            let f = |k: usize| f32::from_le_bytes([b[k], b[k + 1], b[k + 2], b[k + 3]]);
                            c.l9c.push([f(0), f(4), f(8)])
                        }
                        1327 => c.m.set_u(0x5c, v),
                        1329 => c.m.set_u(0x84, v),
                        1330 => c.m.set_u(0x88, v),
                        1331 => c.m.set_u(0x8c, v),
                        1332 => c.m.set_u(0x80, v),
                        1334 => c.m.set_u(0xa0, v),
                        1335 => c.m.set_u(0xa4, v),
                        1336 => c.m.set_u(0xac, v),
                        _ => {}
                    }
                }
            }
            c.l94_obj = c.l94.iter().map(|&o| super::k21_119::Host119::spawner_of(w, o as i32).map_or(0, |k| k as u32 + 1)).collect();
            self.k20.push((i, Inst::K20Spawner(Box::new(c))));
        }
    }
    pub fn k20_119(&mut self, obj: usize) -> Option<&mut C2cfa90> {
        self.k20.iter_mut().find(|x| x.0 == obj).and_then(|x| if let Inst::K20Spawner(c) = &mut x.1 { Some(&mut **c) } else { None })
    }
    pub(super) fn post_load_k20_119(&mut self, h: &mut dyn ClassHost) {
        for (i, x) in &mut self.k20 {
            if let Inst::K20Spawner(c) = x {
                c.ee_sub_1733f8(*i, h);
            }
        }
    }
    pub(super) fn frame_k20_119(&mut self, h: &mut dyn ClassHost) {
        let t = self.g.t;
        for (i, x) in &mut self.k20 {
            if let Inst::K20Spawner(c) = x {
                c.ee_sub_173a68(*i, t, h);
            }
        }
    }
}
