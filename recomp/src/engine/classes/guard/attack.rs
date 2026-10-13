//! K11: the guard's class-120 attack records (C2da840, world class 120) and the attack driver of class 56:
//! guard +0xeb8[8] attack oids (count +0xed8, methods 232 FUN_0020aa40 / 233 FUN_0020aa78 / 234 FUN_0020ab58),
//! +0xedc[] resolved records (count +0xefc, FUN_0020aaa8 from slot 14), +0xf00 running attack, +0xf04 next
//! allowed attack time, +0xf08 / +0xf0c hit part.
//! Selection FUN_001758a0 / FUN_00175ff0, start / stop FUN_0020ab68 (+ FUN_0020ac28), end of the attack by the
//! guard update FUN_00207350 (FUN_00209430 binding done).
//! REPLACED:
//! - records without a prop-233 oid list (the level script adds them with method 232 from Splice) = the
//!   guard's class-120 subtree objects;
//! - the +0xb0 animation binding (FUN_00209278 start / 209360 stop / 209430 done) = the record's ASEG
//!   (method 364 oid, FUN_0015e938 below the guard) clocked from the start time, done at its length;
//! - the ASEG geom events 0xf (enable) / 0x10 (disable) of the guard's ALO hit parts (w0 = geom id,
//!   w1 = hit kind, s0 = amount) -> SO contact: an overlap test of the player's centre against a sphere of the
//!   part's radius (+ the player radius `PLAYER_R`) around the part's bone origin (rest pose in guard space,
//!   turned by the guard yaw). One contact per target while a geom stays on (0x3e15c0 dedupe);
//! - FUN_001758a0's +0x70 line of sight raycast and the script msg 0x18 query pass; the approach goal
//!   FUN_00176458 for records without +0x78 is not ported: the selected attack starts at once;
//! - the attack runs while the brain's chase substate (sub 4, goal 0xa5) targets an object.
use super::super::c2da840::C2da840;
use super::super::ClassHost;
use super::slots::Contact;
use crate::engine::anim::aseg::EventData;
use crate::engine::gfx::attach::{alo_of, BoneWorlds};
use crate::engine::physics::driven::GameRng;
use crate::engine::playerdraw::{aseg_of, ee_sub_15e938_below, subtree};
use crate::engine::world::objects::{LoadData, Payload};
use crate::engine::world::World;

/// DAT_002be604 (FUN_001758a0 angle scale per unit of distance)
const DAT_002BE604: f32 = 0.005;
/// DAT_002be610 (FUN_00175ff0 retry delay)
const DAT_002BE610: f32 = 0.25;
/// DAT_002a88c4
const DAT_002A88C4: f32 = f32::MAX;
/// REPLACED contact: the player's collision radius (JT_RADIUS)
pub const PLAYER_R: f32 = crate::engine::player::JT_RADIUS;

struct Nop;
impl ClassHost for Nop {}

/// Hit part of the guard's ALO (geom id, hit kind, amount, bone origin in guard space, radius).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HitGeom {
    pub geom: i32,
    pub kind: i32,
    pub amount: i16,
    pub local: [f32; 3],
    pub radius: f32,
}

/// One resolved attack record (+0xedc entry).
#[derive(Clone, Debug, Default)]
pub struct AtkRec {
    /// class-120 object index
    pub obj: usize,
    pub rec: C2da840,
    /// ASEG length and geom windows (start, end, geom id)
    pub len: f32,
    pub windows: Vec<(f32, f32, i32)>,
}

#[derive(Clone, Debug, Default)]
pub struct Attacks {
    pub recs: Vec<AtkRec>,
    pub geoms: Vec<HitGeom>,
    /// +0xf00 running attack, +0xf04 next allowed time, +0xf08 last started
    pub f_f00: Option<usize>,
    pub f_f04: f32,
    pub f_f08: Option<usize>,
    /// binding start time (REPLACED +0xb0 clock)
    pub t0: f32,
    /// targets already hit per (geom) while it is on
    pub hit_done: Vec<(i32, u32)>,
    /// attacks started (count, for tests / HUD)
    pub started: u32,
    /// contacts of this frame for slot 73
    pub pending: Vec<Contact>,
}

fn rec_of(w: &World, k: usize) -> C2da840 {
    let mut c = C2da840::default();
    let src = w.objects[k].clone_of.unwrap_or(k);
    if let Some(Payload::Loaded(LoadData::Lo(props))) = w.objects.get(src).map(|x| &x.payload) {
        c.feed(k, props, &mut Nop);
    }
    c
}

impl Attacks {
    /// slot 14 part FUN_0020aaa8: resolve the attack oids (`oids`, prop 233; REPLACED: none -> the class-120
    /// subtree objects) below guard `src`, plus the ALO hit parts.
    pub fn ee_sub_20aaa8(w: &World, src: usize, oids: &[i32]) -> Attacks {
        let sub = subtree(w, src);
        let objs: Vec<usize> = if oids.is_empty() {
            sub.iter().copied().filter(|&k| w.objects[k].class == 120).collect()
        } else {
            oids.iter().filter_map(|&o| ee_sub_15e938_below(w, src, o)).filter(|&k| w.objects[k].class == 120).collect()
        };
        let mut a = Attacks::default();
        for k in objs.into_iter().take(8) {
            let rec = rec_of(w, k);
            let mut r = AtkRec { obj: k, len: 0.0, windows: vec![], rec };
            if let Some(x) = ee_sub_15e938_below(w, src, r.rec.f_b4 as i32).and_then(|x| aseg_of(w, x)) {
                r.len = x.length;
                let mut open: Vec<(i32, f32)> = vec![];
                let mut ev: Vec<_> = x.events1.iter().filter_map(|e| match (&e.data, e.kind) {
                    (EventData::S32x2(_, g), 0xf | 0x10) => Some((e.time.unwrap_or(0.0), e.kind, *g)),
                    _ => None,
                }).collect();
                ev.sort_by(|a, b| a.0.total_cmp(&b.0));
                for (t, kd, g) in ev {
                    if kd == 0xf {
                        open.push((g, t));
                    } else if let Some(p) = open.iter().position(|o| o.0 == g) {
                        let (_, t0) = open.remove(p);
                        r.windows.push((t0, t, g));
                    }
                }
                for (g, t0) in open {
                    r.windows.push((t0, x.length, g));
                }
            }
            a.recs.push(r);
        }
        let mut bw = BoneWorlds::default();
        let root = bw.get(w, src, 0);
        for &k in &sub {
            let Some(alo) = alo_of(w, k) else { continue };
            for p in alo.parts.iter().filter(|p| p.w1 != 0) {
                let m = bw.get(w, k, p.r#ref.max(0) as usize);
                let d = [0, 1, 2].map(|j| m[3][j] - root[3][j]);
                let local = [0, 1, 2].map(|i| {
                    let r = root[i];
                    let n = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).max(1e-12);
                    (d[0] * r[0] + d[1] * r[1] + d[2] * r[2]) / n
                });
                a.geoms.push(HitGeom { geom: p.w0, kind: p.w1 as i32, amount: p.s0, local, radius: p.f });
            }
        }
        a
    }

    /// FUN_001758a0(g, target, knowledge): the first record whose ranges hold the target. Records with +0x78
    /// start at once (returns None), others are returned for the approach goal.
    pub fn ee_sub_1758a0(&mut self, g: [f32; 3], yaw: f32, t: [f32; 3], now: f32, rng: &mut GameRng) -> Option<usize> {
        if self.f_f00.is_some() || now < self.f_f04 {
            return None;
        }
        let (dx, dy, dz) = (t[0] - g[0], t[1] - g[1], t[2] - g[2]);
        let dist = (dx * dx + dy * dy).sqrt();
        let (s, c) = yaw.sin_cos();
        let ang = (-s * dx + c * dy).atan2(c * dx + s * dy);
        for (k, r) in self.recs.iter().enumerate() {
            let a = &r.rec;
            // FUN_0011d910: open interval
            let inside = |v: f32, b: [f32; 2]| b[0] < v && v < b[1];
            if a.f_a8 == 0 || !inside(dist, a.f_50) || !inside(dz, a.f_58) {
                continue;
            }
            if (DAT_002BE604 * dist).clamp(0.0, 1.0) * ang.abs() > a.f_68 {
                continue;
            }
            if a.f_78 != 0 {
                self.ee_sub_20ab68(Some(k), now, rng);
                return None;
            }
            return Some(k);
        }
        None
    }

    /// FUN_00175ff0 (chase goal attack poll): REPLACED approach -> the selected record starts at once.
    pub fn ee_sub_175ff0(&mut self, g: [f32; 3], yaw: f32, t: [f32; 3], now: f32, rng: &mut GameRng) {
        match self.ee_sub_1758a0(g, yaw, t, now, rng) {
            Some(k) => self.ee_sub_20ab68(Some(k), now, rng),
            None if self.f_f00.is_none() && self.f_f04 < now => self.f_f04 = now + DAT_002BE610,
            None => {}
        }
    }

    /// FUN_0020ab68(g, atk): stop the running attack (+0xf04 = now + rand(+0x60)), start `atk`
    pub fn ee_sub_20ab68(&mut self, atk: Option<usize>, now: f32, rng: &mut GameRng) {
        if let Some(old) = self.f_f00.take() {
            let r = self.recs[old].rec.f_60;
            self.f_f04 = now + rng.FUN_0011d448(r[0], r[1]);
        }
        self.hit_done.clear();
        if let Some(k) = atk {
            self.f_f00 = Some(k);
            self.t0 = now;
            self.f_f04 = DAT_002A88C4;
            // FUN_0020ac28
            self.f_f08 = Some(k);
            self.started += 1;
        }
    }

    /// geoms on at the running attack's binding time
    pub fn geoms_on(&self, now: f32) -> Vec<i32> {
        let Some(k) = self.f_f00 else { return vec![] };
        let t = now - self.t0;
        self.recs[k].windows.iter().filter(|w| w.0 <= t && t < w.1).map(|w| w.2).collect()
    }

    /// Per frame (guard update FUN_00207350 part + the REPLACED geom contacts): `chase` = chase target
    /// position. Returns the contacts for slot 73.
    pub fn step(&mut self, pos: [f32; 3], yaw: f32, chase: Option<[f32; 3]>, players: &[(u32, [f32; 3])], now: f32, rng: &mut GameRng) {
        self.pending.clear();
        if let Some(k) = self.f_f00 {
            // FUN_00209430: binding done
            if now - self.t0 >= self.recs[k].len {
                self.ee_sub_20ab68(None, now, rng);
            }
        }
        if let Some(t) = chase {
            self.ee_sub_175ff0(pos, yaw, t, now, rng);
        }
        let on = self.geoms_on(now);
        if on.is_empty() {
            return;
        }
        let (s, c) = yaw.sin_cos();
        for g in self.geoms.iter().filter(|g| on.contains(&g.geom)) {
            let l = g.local;
            let cw = [pos[0] + c * l[0] - s * l[1], pos[1] + s * l[0] + c * l[1], pos[2] + l[2]];
            for &(h, p) in players {
                let d = [p[0] - cw[0], p[1] - cw[1], p[2] - cw[2]];
                let n = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                if n > g.radius + PLAYER_R || self.hit_done.contains(&(g.geom, h)) {
                    continue;
                }
                self.hit_done.push((g.geom, h));
                let v = if n > 1e-4 { [d[0] / n, d[1] / n, d[2] / n, 0.0] } else { [1.0, 0.0, 0.0, 0.0] };
                self.pending.push(Contact { kind: g.kind, amount: g.amount, other: h, pos: [p[0], p[1], p[2], 1.0], vec: v });
            }
        }
    }
}
