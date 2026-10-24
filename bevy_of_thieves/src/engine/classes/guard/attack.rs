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
//! - FUN_001758a0's script msg 0x18 query (BOSS R30): asked through `ClassHost::atk_msg18`; the game layer runs
//!   the guard's handler one frame later (vm_state flush; no synchronous VM path from the class tick), so the
//!   answer for a candidate list is used on the next poll with the same candidates;
//! - FUN_001758a0's +0x70 line of sight raycast; the approach goal
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
    /// SO shape +0x10 bit 50 (0x4000000000000): set = inactive (FUN_001f2b20 / FUN_001f72f8 skip it).
    /// FUN_001f2a60 clears, FUN_001f2ad0 sets. Start value: FUN_001f31e0 stream bit 50; REPLACED = set (the
    /// port's AloPart b2 reads 0 while EMU 276 / 285 shows bit 50 set on all 9 loaded shapes of Dimitri and no
    /// other writer exists in the decomp: parse discrepancy, logs/BOSS.md R7).
    pub off: bool,
    /// (ALO object, bone) the shape rides on
    pub bone: (usize, usize),
}

/// One clip a record's anim can play: (ASEG object, length, timed shape events (time, kind 0xd/0xe/0xf/0x10,
/// +0xc id), root motion at 1/60 s).
#[derive(Clone, Debug, Default)]
pub struct Clip {
    pub aseg: usize,
    pub len: f32,
    pub events: Vec<(f32, u32, i32)>,
    pub root: Vec<[f32; 3]>,
    /// CRAWL R28: per hit geom (index into `Attacks::geoms`) its bone origin in guard space at 1/60 s, posed
    /// by this clip (FUN_001f72f8 reads the shape bone's live world matrix; the port samples it at load)
    pub posed: Vec<(usize, Vec<[f32; 3]>)>,
}

/// SO +0xc0 entry (0x28 bytes, count +0xb8): +0 oid, +0xc flags (0x800 = off, FUN_001f69f8), +0x24 linked
/// +0xe4 shape (index into `Attacks::geoms`).
#[derive(Clone, Debug, Default)]
pub struct HitEntry {
    pub oid: i32,
    pub flags: u32,
    pub shape: Option<usize>,
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
    /// WIRE-D: the attack ASEG root motion FUN_00204e38(aseg, t, 1) sampled at 1/60 s (the 0xa4 attack
    /// goal plays the record's anim: its root channel carries the guard's lunge)
    pub root: Vec<[f32; 3]>,
    /// clips of the record's anim: one plain ASEG (FUN_00209278 case 0), or the transitions of a class-111 SM
    /// (case 1: FUN_001f0198 -> FUN_001f01b8(sm, guard, -1, flags, 4))
    pub clips: Vec<Clip>,
    pub sm: Option<crate::engine::anim::sm::Sm>,
    /// clip playing (index into `clips`)
    pub cur: usize,
}

#[derive(Clone, Debug, Default)]
pub struct Attacks {
    /// SO +0xc0 / +0xb8 entries: the ALO mesh records (FUN_001f31e0 -> FUN_00167320: +0 word, +0xc bit 11)
    pub entries: Vec<HitEntry>,
    /// binding time already dispatched (ASEGA event cursor)
    pub t_ev: f32,
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
    /// BOSS: an attack started outside `step` (perch 0xb5 FUN_001758a0 path): bind its anim next step
    pub started_pending: bool,
    /// contacts of this frame for slot 73
    pub pending: Vec<Contact>,
}

fn rec_of(w: &World, k: usize) -> C2da840 {
    let mut c = C2da840::default();
    // +0x18 = the record object's oid (asked as message 0x18's argument, BOSS R32)
    c.oid = w.objects[k].oid;
    let src = w.objects[k].clone_of.unwrap_or(k);
    if let Some(Payload::Loaded(LoadData::Lo(props))) = w.objects.get(src).map(|x| &x.payload) {
        c.feed(k, props, &mut Nop);
    }
    c
}

impl AtkRec {
    /// make clip `i` the playing one (len / windows / root mirror it for the readers of the record)
    pub fn select(&mut self, i: usize) {
        let Some(c) = self.clips.get(i) else { return };
        self.cur = i;
        self.len = c.len;
        self.root = c.root.clone();
        let mut open: Vec<(i32, f32)> = vec![];
        self.windows.clear();
        for &(t, kd, g) in &c.events {
            if kd == 0xf {
                open.push((g, t));
            } else if kd == 0x10 {
                if let Some(p) = open.iter().position(|o| o.0 == g) {
                    let (_, t0) = open.remove(p);
                    self.windows.push((t0, t, g));
                }
            }
        }
        for (g, t0) in open {
            self.windows.push((t0, c.len, g));
        }
    }
}

impl Attacks {
    /// FUN_001f6978(so, oid): +0xc0 entries of `oid`: flags &= ~0x800, linked shape bit 50 cleared (ASEG 0xd)
    pub fn ee_sub_1f6978(&mut self, oid: i32) {
        for e in self.entries.iter_mut().filter(|e| e.oid == oid) {
            e.flags &= !0x800;
            if let Some(g) = e.shape.and_then(|k| self.geoms.get_mut(k)) {
                g.off = false;
            }
        }
    }
    /// FUN_001f69f8(so, oid): flags |= 0x800, linked shape bit 50 set (ASEG 0xe)
    pub fn ee_sub_1f69f8(&mut self, oid: i32) {
        for e in self.entries.iter_mut().filter(|e| e.oid == oid) {
            e.flags |= 0x800;
            if let Some(g) = e.shape.and_then(|k| self.geoms.get_mut(k)) {
                g.off = true;
            }
        }
    }
    /// FUN_001f6a68(so): every +0xc0 entry off
    pub fn ee_sub_1f6a68(&mut self) {
        for e in &mut self.entries {
            e.flags |= 0x800;
            if let Some(g) = e.shape.and_then(|k| self.geoms.get_mut(k)) {
                g.off = true;
            }
        }
    }
    /// FUN_001f6ac8(so, id) -> FUN_001f2a60: shapes with +0xc id: bit 50 cleared (ASEG 0xf)
    pub fn ee_sub_1f6ac8(&mut self, id: i32) {
        self.geoms.iter_mut().filter(|g| g.geom == id).for_each(|g| g.off = false);
    }
    /// FUN_001f6b48(so, id) -> FUN_001f2ad0: bit 50 set (ASEG 0x10)
    pub fn ee_sub_1f6b48(&mut self, id: i32) {
        self.geoms.iter_mut().filter(|g| g.geom == id).for_each(|g| g.off = true);
    }
    /// FUN_00210408 / FUN_002104a0 (ASEG events 0xd..0x10, +8 = -1: the owner itself) of the running clip
    /// between the event cursor and binding time `t` (FUN_00130f88 forward)
    fn fire(&mut self, t: f32) {
        let Some(k) = self.f_f00 else { return };
        let r = &self.recs[k];
        let Some(c) = r.clips.get(r.cur) else { return };
        let evs: Vec<(u32, i32)> = c.events.iter().filter(|e| e.0 >= self.t_ev && (e.0 < t || (e.0 == 0.0 && self.t_ev == 0.0))).map(|e| (e.1, e.2)).collect();
        self.t_ev = t;
        for (kd, id) in evs {
            match kd {
                0xd => self.ee_sub_1f6978(id),
                0xe => self.ee_sub_1f69f8(id),
                0xf => self.ee_sub_1f6ac8(id),
                _ => self.ee_sub_1f6b48(id),
            }
        }
    }

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
            let mut r = AtkRec { obj: k, len: 0.0, windows: vec![], rec, root: vec![], clips: vec![], sm: None, cur: 0 };
            let anim = ee_sub_15e938_below(w, src, r.rec.f_b4 as i32);
            let clip = |x: usize| -> Option<Clip> {
                let g = aseg_of(w, x)?;
                let mut events: Vec<(f32, u32, i32)> = g.events1.iter().filter_map(|e| match (&e.data, e.kind) {
                    (EventData::S32x2(_, id), 0xd..=0x10) => Some((e.time.unwrap_or(0.0), e.kind as u32, *id)),
                    _ => None,
                }).collect();
                events.sort_by(|a, b| a.0.total_cmp(&b.0));
                let root = (0..=(g.length * 60.0) as usize).map(|i| super::los::ee_sub_204e38(g, i as f32 / 60.0, 1.0)).collect();
                Some(Clip { aseg: x, len: g.length, events, root, posed: vec![] })
            };
            match anim.map(|k| (k, &w.objects[k].payload)) {
                // FUN_00209278 case 1: class-111 SM (load FUN_001efac0)
                Some((_, Payload::Loaded(LoadData::B4(b)))) if matches!(&**b, crate::engine::world::loads_b4::B4Load::L1efac0 { .. }) => {
                    if let crate::engine::world::loads_b4::B4Load::L1efac0 { nested, pairs, links, link_f, .. } = &**b {
                        let nested: Vec<(usize, i32)> = nested.iter().map(|&n| (n, w.objects[n].oid)).collect();
                        let sm = crate::engine::anim::sm::Sm::ee_sub_1efac0(pairs, links, link_f, &nested);
                        r.clips = nested.iter().filter_map(|&(n, _)| clip(n)).collect();
                        r.sm = Some(sm);
                    }
                }
                Some((x, _)) => r.clips = clip(x).into_iter().collect(),
                None => {}
            }
            r.select(0);
            a.recs.push(r);
        }
        // FUN_00167320 (SO +0x80): one 0x28 entry per mesh record; +0x24 is never written (0 in every EMU
        // entry), so 0xd / 0xe only flip the draw bit 0x800
        if let Some(alo) = alo_of(w, src) {
            a.entries = alo.mesh.records.iter().map(|m| HitEntry { oid: m.word, flags: (m.u1 as u32 & 1) << 11, shape: None }).collect();
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
                a.geoms.push(HitGeom { geom: p.w0, kind: p.w1 as i32, amount: p.s0, local, radius: p.f, off: true, bone: (k, p.r#ref.max(0) as usize) });
            }
        }
        // CRAWL R28: pose every shape bone along each clip (AnimBinder: the clip's channels bound on the guard)
        let geoms: Vec<(usize, (usize, usize))> = a.geoms.iter().enumerate().map(|(i, g)| (i, g.bone)).collect();
        let rest: Vec<[f32; 3]> = a.geoms.iter().map(|g| g.local).collect();
        for r in &mut a.recs {
            for c in &mut r.clips {
                let Some(g) = aseg_of(w, c.aseg) else { continue };
                let mut b = crate::engine::anim::bind::AnimBinder::default();
                if b.start(w, 1, c.aseg, g, Some(src), 0, 0) == 0 {
                    continue;
                }
                let n = (c.len * 60.0) as usize + 1;
                // the binder's bone chain may carry a placement the rest BoneWorlds lack: compose the bone at
                // its rest local through the same chain (`rc`) and read the clip's displacement in that frame
                let place: Vec<Option<[[f32; 3]; 4]>> = geoms.iter().map(|(_, key)| {
                    let l = crate::engine::anim::bind::alo_data(w, key.0)?.base.bones.get(key.1)?.xform.unwrap_or(crate::engine::gfx::scene::IDENTITY);
                    b.compose(w, std::collections::HashMap::from([(*key, l)])).bones.get(key).copied()
                }).collect();
                let mut per: Vec<Vec<[f32; 3]>> = vec![Vec::with_capacity(n); geoms.len()];
                for f in 0..n {
                    let t = f as f32 / 60.0;
                    let pose = b.pose(w, &|_| Some((t, 1.0)));
                    for (i, key) in &geoms {
                        let l0 = rest[*i];
                        let (Some(rc), Some(m)) = (place[*i], pose.bones.get(key)) else {
                            per[*i].push(l0);
                            continue;
                        };
                        // R_place = R_rest^T * R_rc (rows); displacement (m - rc) back through R_place^T, then
                        // into guard space through the rest bone world (bw: guard space)
                        let m0 = bw.get(w, key.0, key.1);
                        let d = [0, 1, 2].map(|j| m[3][j] - rc[3][j]);
                        let rp = [0, 1, 2].map(|r| [0, 1, 2].map(|c| (0..3).map(|k| m0[k][r] * rc[k][c]).sum::<f32>()));
                        let dl = [0, 1, 2].map(|r| (0..3).map(|c| d[c] * rp[r][c]).sum::<f32>());
                        per[*i].push([0, 1, 2].map(|j| l0[j] + dl[j]));
                    }
                }
                c.posed = per.into_iter().enumerate().collect();
            }
        }
        a
    }

    /// FUN_001758a0(g, target, knowledge): the first record whose ranges hold the target. Records with +0x78
    /// start at once (returns None), others are returned for the approach goal.
    /// `guard` = the guard's object index: each record passing the range / angle tests is put to the guard's script as
    /// vt +0x1c message 0x18 {record +0x18 oid} (reply {0x17, 1} kept = allowed) through `h.atk_msg18` (module doc).
    pub fn ee_sub_1758a0(&mut self, guard: usize, g: [f32; 3], yaw: f32, t: [f32; 3], now: f32, rng: &mut GameRng, h: &mut dyn ClassHost) -> Option<usize> {
        if self.f_f00.is_some() || now < self.f_f04 {
            return None;
        }
        let (dx, dy, dz) = (t[0] - g[0], t[1] - g[1], t[2] - g[2]);
        let dist = (dx * dx + dy * dy).sqrt();
        let (s, c) = yaw.sin_cos();
        let ang = (-s * dx + c * dy).atan2(c * dx + s * dy);
        let mut cands = Vec::new();
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
            cands.push((k, a.oid));
        }
        if cands.is_empty() {
            return None;
        }
        // vt +0x1c(guard, 0x18, 0, 1, {+0x18}, &{0x17, 1}) per candidate in record order; first allowed one wins
        let k = h.atk_msg18(guard, &cands)?;
        if self.recs[k].rec.f_78 != 0 {
            self.ee_sub_20ab68(Some(k), now, rng);
            return None;
        }
        Some(k)
    }

    /// FUN_00175ff0 (chase goal attack poll): REPLACED approach -> the selected record starts at once.
    pub fn ee_sub_175ff0(&mut self, guard: usize, g: [f32; 3], yaw: f32, t: [f32; 3], now: f32, rng: &mut GameRng, h: &mut dyn ClassHost) {
        match self.ee_sub_1758a0(guard, g, yaw, t, now, rng, h) {
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
            // FUN_00209278: case 1 (class-111 SM) FUN_001f0198 -> FUN_001f01b8(sm, guard, -1, flags, mask 4):
            // the SMA walks state 0 -> its next state by a weighted transition (FUN_001f0b80) and plays its ASEG
            let r = &mut self.recs[k];
            if let Some(sm) = &r.sm {
                let lens: Vec<(usize, f32)> = r.clips.iter().map(|c| (c.aseg, c.len)).collect();
                let len = |o: usize| lens.iter().find(|x| x.0 == o).map_or(0.0, |x| x.1);
                let sma = crate::engine::anim::sm::Sma::ee_sub_1f01b8(sm, rng, -1, 0, 4, &len);
                if let Some(i) = sma.asega.and_then(|a| r.clips.iter().position(|c| c.aseg == a.aseg)) {
                    r.select(i);
                }
            }
            self.t_ev = 0.0;
            self.f_f00 = Some(k);
            self.t0 = now;
            self.f_f04 = DAT_002A88C4;
            // FUN_0020ac28
            self.f_f08 = Some(k);
            self.started += 1;
        }
    }

    /// WIRE-D: root velocity of the running attack at its binding time (None = no attack running)
    pub fn root_v(&self, now: f32) -> Option<[f32; 3]> {
        let r = &self.recs[self.f_f00?];
        let i = ((now - self.t0).max(0.0) * 60.0) as usize;
        Some(r.root.get(i.min(r.root.len().saturating_sub(1))).copied().unwrap_or([0.0; 3]))
    }

    /// geoms on at the running attack's binding time
    pub fn geoms_on(&self, now: f32) -> Vec<i32> {
        let Some(k) = self.f_f00 else { return vec![] };
        let t = now - self.t0;
        self.recs[k].windows.iter().filter(|w| w.0 <= t && t < w.1).map(|w| w.2).collect()
    }

    /// Per frame (guard update FUN_00207350 part + the REPLACED geom contacts): `chase` = chase target
    /// position. Returns the contacts for slot 73.
    pub fn step(&mut self, guard: usize, pos: [f32; 3], yaw: f32, chase: Option<[f32; 3]>, players: &[(u32, [f32; 3])], now: f32, rng: &mut GameRng, h: &mut dyn ClassHost) {
        self.pending.clear();
        if self.f_f00.is_some() {
            self.fire(now - self.t0 + 1e-4);
        }
        if let Some(k) = self.f_f00 {
            // FUN_00209430: binding done
            if now - self.t0 >= self.recs[k].len {
                self.ee_sub_20ab68(None, now, rng);
            }
        }
        if let Some(t) = chase {
            self.ee_sub_175ff0(guard, pos, yaw, t, now, rng, h);
        }
        let (s, c) = yaw.sin_cos();
        // FUN_001f72f8: active shapes only (bit 50 clear)
        let posed = self.f_f00.and_then(|k| self.recs[k].clips.get(self.recs[k].cur)).map(|c| (c, ((now - self.t0) * 60.0) as usize));
        for (gi, g) in self.geoms.iter().enumerate().filter(|g| !g.1.off) {
            let l = posed.and_then(|(c, f)| c.posed.get(gi).and_then(|p| p.1.get(f.min(p.1.len().saturating_sub(1))))).copied().unwrap_or(g.local);
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
