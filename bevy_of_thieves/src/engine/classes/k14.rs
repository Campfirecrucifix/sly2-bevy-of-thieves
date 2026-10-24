//! WP-K14: class 76 C2d0b30 (parent 71 C2d1878): a swarm of riders on up to 4 class-10 rails (groups at
//! +0x1cc, 0x30 bytes, count +0x1c8, added by the Splice method 1398 FUN_00182ab8). Every group keeps its
//! riders (0x70-byte records, FUN_001ae2a0 {0x70, 0x10}) spread in bands around the player's position on the
//! rail and swinging sideways; hit (slot 74) a rider loses hp (+0x4c), at 0 it dies (state 2: off the group
//! list onto +0x28c, the bands are re-laid, the last one signals 0x3a); touching (slot 54 -> deferred slot 81)
//! hurts by +0x188.
//! Own slots 5 FUN_00113ed0, 6 FUN_001817e8, 11 FUN_0017f110, 14 FUN_00181840, 15 FUN_00181f60, 18
//! FUN_001821c0, 25 FUN_0017f170, 48 FUN_00182780, 54 FUN_00182700, 59 FUN_001823c8, 62 FUN_00182778, 74
//! FUN_001828a8, 81 FUN_001829d8, 82 FUN_001819e0 1:1 with FUN_00181a90 / 181b60 / 181cb0 / 181e10 / 181eb0 /
//! 182810 / 182ab8, FUN_0011d1c0, FUN_0014c748, FUN_001e7bb8 (rail world bounds); class-71 parts in
//! shared/c71.rs.
//! REPLACED: rider records / lists (FUN_001ae2a0 pool, FUN_0011a220 / 11a2e8 / 11a200 intrusive lists at
//! +0x60) -> `riders` Vec + per-group index lists (same order: append at the tail); the FUN_00149a50 deferred
//! slot-81 call -> `ClassSet::k14_deferred`, drained at the end of `frame_k14` (FUN_00149c80 phase).
//! Slot 59 (body vs riders contact query of the SO solver): the query record is the caller's; ported over
//! `Query59` (static branch 1:1; swept branch FUN_0014c748 1:1, contact depth REPLACED, see `ee_sub_1823c8`).
//! Slot 18 draw: matrix per rider 1:1, the model draw FUN_001a5ee8 is the `ee_sub_1a5ee8` renderer hook.
//! Quirk kept (checked in the ELF: two `swc1 0x50`): the roll spring of FUN_00181f60 integrates into +0x50,
//! not +0x54.
#![allow(non_snake_case)]
use super::shared::c71::C71;
use super::*;
use crate::engine::camera::looks::FUN_0011b7e0;
use crate::engine::camera::math::FUN_0011b500;
use crate::engine::physics::dynamics::FUN_0011d018;
use crate::engine::player::p5_attach::{Path, FUN_001e6648, FUN_001e6ed8, FUN_001e6f10, FUN_001e6f60};

/// K14 world class id -> vtable (kept in `ClassSet::k14`).
pub const PORTED_K14: &[(ClassId, u32)] = &[(76, 0x2d0b30)];

/// slot 5 FUN_00113ed0 (class descriptor getter)
pub fn slot5(class: ClassId) -> Option<u32> {
    match class {
        76 => Some(0x2bb0d8),
        _ => None,
    }
}

/// DAT_002be884: hit (state 1) duration
pub const D_2BE884: f32 = 0.15;
/// 0x2be888: FUN_0011d1c0 speed table {vmax 1500, v0 0, accel 0.5, max accel change 4500}
pub const T_2BE888: [f32; 4] = [1500.0, 0.0, 0.5, 4500.0];
/// 0x2be898 / 0x2be89c (sideways spring, damping), 0x2be8a0 / 0x2be8a4 (roll spring, damping)
pub const K_2BE898: [f32; 4] = [-100.0, -10.0, -15.0, -2.5];
/// DAT_002be8a8: hit sideways kick
pub const D_2BE8A8: f32 = 0.25;
/// DAT_002be8ac: group rider hp (+0x1c)
pub const D_2BE8AC: i32 = 30;

/// rider record (0x70)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Rider {
    /// +0x0 world position, +0x10 rail tangent
    pub pos: [f32; 3],
    pub tan: [f32; 3],
    /// +0x20 group index
    pub group: usize,
    /// +0x24 state (-1 new, 0 follow, 1 hit, 2 dead), +0x28 state time
    pub state: i32,
    pub t_28: f32,
    /// +0x2c / +0x30 band, +0x34 / +0x38 outer band
    pub band: [f32; 4],
    /// +0x3c rail arc length, +0x40 its speed, +0x44 target, +0x48 -|band centre - player|
    pub l_3c: f32,
    pub v_40: f32,
    pub tgt_44: f32,
    pub f_48: f32,
    /// +0x4c hp
    pub hp: i32,
    /// +0x50 sideways offset, +0x54 roll, +0x58 / +0x5c their speeds
    pub p_50: f32,
    pub p_54: f32,
    pub v_58: f32,
    pub v_5c: f32,
}

/// group record (0x30 at +0x1cc)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Group {
    /// +0x0 rail oid, +0x4 riders, +0x8 riders per band, +0xc alive
    pub oid: i32,
    pub n: i32,
    pub k_8: i32,
    pub alive: i32,
    /// +0x10 alive rider list (indices into `C2d0b30::riders`, head first)
    pub list: Vec<usize>,
    /// +0x1c rider hp, +0x20 rider radius / half spacing
    pub hp_1c: i32,
    pub r_20: f32,
    /// +0x24 rail (object handle) and its path
    pub rail: u32,
    pub path: Option<Path>,
    /// K16: the full class-10 path when it is a spline (+0xe0 == 1): the queries take the spline branches
    pub spline: Option<crate::engine::player::p5_target::Path>,
    /// +0x28 player arc length on the rail, +0x2c next full closest-point search time
    pub l_28: f32,
    pub t_2c: f32,
}

/// body sphere query of slot 59 (the solver's record): +0 body (+0x70 position), +8 radius, +0x10 flags,
/// +0x1c first point, +0x20..+0x50 point transform rows, +0x7c / +0x94 segment starts / ends, +0x90 count
#[derive(Clone, Debug, Default)]
pub struct Query59 {
    pub pos: [f32; 3],
    pub radius: f32,
    pub flags: u64,
    pub start: usize,
    pub m: [[f32; 4]; 4],
    pub pts: Vec<[f32; 3]>,
    pub ends: Vec<[f32; 3]>,
}

/// one FUN_001a7c18 contact of slot 59: rider, depth (negative = penetration), normal
#[derive(Clone, Debug, PartialEq)]
pub struct Contact59 {
    pub rider: usize,
    pub depth: f32,
    pub normal: [f32; 3],
}

#[derive(Clone, Debug, Default)]
pub struct C2d0b30 {
    pub b: C71,
    /// +0x1c0 active (row 1397), +0x1c4 rider model (row 1399, object handle)
    pub f_1c0: u32,
    pub f_1c4: u32,
    pub groups: Vec<Group>,
    pub riders: Vec<Rider>,
    /// +0x28c dead riders
    pub dead: Vec<usize>,
    /// +0xe0 / +0xe4 collision part records added by slot 14 ({+0x60 world, self, -1, flags | 0x3 << 48})
    pub parts_e0: i32,
    /// slot 48 bounds (min, max) after the base FUN_001f50e0 (riders' rails expanded by +0x20)
    pub bounds: Option<([f32; 3], [f32; 3])>,
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
/// _vopmula(Z, v) / _vopmsub(v, Z) = Z x v (DAT_002a2dc0 = (0, 0, 1))
fn z_cross(v: [f32; 3]) -> [f32; 3] {
    [-v[1], v[0], 0.0]
}
fn handle(obj: usize) -> u32 {
    obj as u32 + 1
}

/// FUN_0011d1c0(x, v, target, dt, table, &v'): FUN_0011d018 with the speed change limited to table[3] * dt
/// (x re-integrated at that limit)
pub fn FUN_0011d1c0(x: f32, v: f32, target: f32, dt: f32, t: [f32; 4], vout: &mut f32) -> f32 {
    let mut v2 = 0.0;
    let mut r = FUN_0011d018(x, target, dt, [t[0], t[1], t[2]], Some(&mut v2));
    let a = t[3];
    if 0.0 < a {
        let hi = v + a * dt;
        let lo = v - a * dt;
        if hi < v2 {
            r = x + v * dt + a * dt * 0.5 * dt;
            v2 = hi;
        } else if v2 < lo {
            r = (x + v * dt) - a * dt * 0.5 * dt;
            v2 = lo;
        }
    }
    *vout = v2;
    r
}

/// FUN_0014c748(r, c, a, b, 2, out): segment a -> b against the sphere (c, r): entry / exit parameters
/// clamped to [0, 1], their points and outward normals; None when it misses (disc < 1e-4, both behind, or
/// both past b)
pub fn FUN_0014c748(r: f32, c: [f32; 3], a: [f32; 3], b: [f32; 3]) -> Option<[([f32; 3], [f32; 3], f32); 2]> {
    let d0 = sub(a, c);
    let cc = dot(d0, d0);
    let e = sub(b, a);
    let bq = dot(e, d0) + dot(e, d0);
    let aq = dot(e, e);
    let disc = bq * bq - aq * (cc - r * r) * 4.0;
    if disc < 0.0001 {
        return None;
    }
    let s = disc.sqrt();
    let t0 = (-bq - s) / (aq + aq);
    let t1 = (-bq + s) / (aq + aq);
    if t0 < 0.0 && t1 < 0.0 {
        return None;
    }
    if !(t0 <= 1.0 || t1 <= 1.0) {
        return None;
    }
    let ts = [if 0.0 <= t0 { t0 } else { 0.0 }, if 1.0 < t1 { 1.0 } else { t1 }];
    Some(ts.map(|t| {
        let p = [b[0] * t + a[0] * (1.0 - t), b[1] * t + a[1] * (1.0 - t), b[2] * t + a[2] * (1.0 - t)];
        let d = sub(p, c);
        let l = dot(d, d).sqrt();
        let n = if l < 0.0001 { [1.0, 0.0, 0.0] } else { [d[0] / l, d[1] / l, d[2] / l] };
        (p, n, t)
    }))
}

/// FUN_001e7bb8(rail, &min, &max): world bounds of the rail points
pub fn FUN_001e7bb8(p: &Path) -> ([f32; 3], [f32; 3]) {
    let mut lo = [f32::MAX; 3];
    let mut hi = [-f32::MAX; 3];
    for q in &p.pts {
        let r = &p.rot;
        let w = [0, 1, 2].map(|k| r[0][k] * q[0] + r[1][k] * q[1] + r[2][k] * q[2] + p.pos[k]);
        for k in 0..3 {
            lo[k] = lo[k].min(w[k]);
            hi[k] = hi[k].max(w[k]);
        }
    }
    (lo, hi)
}

/// per-frame context of the slot code (DAT_002dded0 time, DAT_002dded4 dt, rand 0x2342a0)
pub struct Cx<'a> {
    pub t: f32,
    pub dt: f32,
    pub rng: &'a mut GameRng,
    pub h: &'a mut dyn ClassHost,
}

impl C2d0b30 {
    /// slot 6 FUN_001817e8: FUN_0017f090; +0x40 |= 8; +0x28c list (link +0x60) empty
    pub fn ee_sub_1817e8(&mut self, obj: usize, h: &mut dyn ClassHost) {
        self.b.ee_sub_17f090(obj, h);
        self.b.f_40 |= 8;
        self.dead.clear();
    }

    /// method 1398 FUN_00182ab8(r, self, oid, n, k): add a group (max 4): +0x1c = DAT_002be8ac, +0x2c = -1
    pub fn ee_sub_182ab8(&mut self, r: f32, oid: i32, n: i32, k: i32) {
        if self.groups.len() < 4 {
            self.groups.push(Group { oid, n, k_8: k, hp_1c: D_2BE8AC, r_20: r, t_2c: -1.0, ..Default::default() });
        }
    }

    /// slot 14 FUN_00181840: resolve every group's rail (FUN_0015e938 kind 4); a group whose rail is missing
    /// shrinks the count and the same slot is retried (no compaction: every group from it on is dropped);
    /// else n riders (+0x20 group, +0x24 -1) go on its list. Then FUN_0017f138 + slot 82 (reset), a collision
    /// part record, and FUN_001f2b20 while in the world (+0x28 bit 60).
    pub fn ee_sub_181840(&mut self, obj: usize, in_world: bool, cx: &mut Cx) {
        let mut i = 0;
        let mut count = self.groups.len();
        while i < count {
            let g = &mut self.groups[i];
            let rail = cx.h.ee_sub_15e938(4, g.oid, handle(obj));
            g.rail = rail;
            if rail == 0 {
                count -= 1;
                continue;
            }
            g.list.clear();
            for _ in 0..g.n.max(0) {
                self.riders.push(Rider { group: i, state: -1, ..Default::default() });
                self.groups[i].list.push(self.riders.len() - 1);
            }
            i += 1;
        }
        self.groups.truncate(count);
        self.b.ee_sub_17f138(obj, cx.h);
        self.ee_sub_1819e0(obj, cx);
        self.parts_e0 += 1;
        if in_world {
            cx.h.base(0x1f2b20, obj);
        }
    }

    /// slot 82 FUN_001819e0 (reset): FUN_0017f6b0; dead riders back on their group lists (tail); every
    /// group: alive = n, FUN_00181a90. Returns whether the pending touch must be cancelled (FUN_00149b58).
    pub fn ee_sub_1819e0(&mut self, obj: usize, cx: &mut Cx) -> bool {
        let cancel = self.b.ee_sub_17f6b0();
        for r in std::mem::take(&mut self.dead) {
            let g = self.riders[r].group;
            self.groups[g].list.push(r);
        }
        for g in 0..self.groups.len() {
            self.groups[g].alive = self.groups[g].n;
            self.ee_sub_181a90(obj, g, cx);
        }
        cancel
    }

    /// FUN_00181a90(group): FUN_00181b60; riders evenly over the rail (L (i + 0.5) / alive), speed 0, state 0,
    /// hp +0x1c
    pub fn ee_sub_181a90(&mut self, obj: usize, g: usize, cx: &mut Cx) {
        self.ee_sub_181b60(g);
        let l = self.groups[g].path.as_ref().map(FUN_001e6648).unwrap_or(0.0);
        let list = self.groups[g].list.clone();
        for (i, &r) in list.iter().enumerate() {
            self.riders[r].state = -1;
            self.ee_sub_181eb0(obj, r, 0, cx);
            let gr = &self.groups[g];
            let n = gr.alive;
            let rd = &mut self.riders[r];
            rd.v_40 = 0.0;
            rd.l_3c = (l * (i as f32 + 0.5)) / n as f32;
            rd.hp = gr.hp_1c;
        }
    }

    /// FUN_00181b60(group): band layout of the alive riders: s = min(L / 2n, +0x20), k = min(+8, n); bands of
    /// width w = (L - 2 n s) / (n - k + 1), step w + 2 s
    pub fn ee_sub_181b60(&mut self, g: usize) {
        let gr = &self.groups[g];
        let l = gr.path.as_ref().map(FUN_001e6648).unwrap_or(0.0);
        let n = gr.alive;
        let fn_ = n as f32;
        let k = if n <= gr.k_8 { n } else { gr.k_8 };
        let s = {
            let a = (l * 0.5) / fn_;
            if a < gr.r_20 { a } else { gr.r_20 }
        };
        let fk = k as f32;
        let c = ((fk + fk) - 1.0) * s;
        let wd = (l - (fn_ + fn_) * s) / ((n - k) + 1) as f32;
        let step = wd + s + s;
        let o = step * (k - 1) as f32;
        let mut b = [c - o, (c + fk * wd) - o, 0.0, 0.0];
        b[2] = b[0] - fk * s;
        b[3] = b[1] + fk * s;
        for &r in &gr.list {
            self.riders[r].band = b;
            for x in &mut b {
                *x += step;
            }
        }
    }

    /// FUN_00181cb0(group): each rider's target = its band lerped by the player's place in the outer band,
    /// kept (2i + 1) s from the start and (2 (n - i - 1) + 1) s from the end
    pub fn ee_sub_181cb0(&mut self, g: usize) {
        let gr = &self.groups[g];
        let l = gr.path.as_ref().map(FUN_001e6648).unwrap_or(0.0);
        let s = {
            let a = (l * 0.5) / gr.alive as f32;
            if a < gr.r_20 { a } else { gr.r_20 }
        };
        for (i, &r) in gr.list.iter().enumerate() {
            let rd = &mut self.riders[r];
            let f = ((gr.l_28 - rd.band[2]) / (rd.band[3] - rd.band[2])).clamp(0.0, 1.0);
            let t = (1.0 - f) * rd.band[0] + f * rd.band[1];
            let rest = ((gr.alive - i as i32) - 1) as f32;
            let lo = (i as f32 + i as f32 + 1.0) * s;
            let hi = l - (rest + rest + 1.0) * s;
            let mut v = lo;
            if lo <= t {
                v = t;
                if hi < t {
                    v = hi;
                }
            }
            rd.tgt_44 = v;
            rd.f_48 = -((rd.band[0] + rd.band[1]) * 0.5 - gr.l_28).abs();
        }
    }

    /// FUN_00181eb0(rider, state): on a change: state, time; state 2 = the rider leaves its group (alive - 1,
    /// onto +0x28c; the last one signals FUN_001619a0(0x2deb30, 0x3a, self)) and the bands are re-laid
    pub fn ee_sub_181eb0(&mut self, obj: usize, r: usize, state: i32, cx: &mut Cx) {
        if state == self.riders[r].state {
            return;
        }
        self.riders[r].state = state;
        self.riders[r].t_28 = cx.t;
        if state == 2 {
            let g = self.riders[r].group;
            let gr = &mut self.groups[g];
            gr.alive -= 1;
            gr.list.retain(|&x| x != r);
            self.dead.push(r);
            if self.groups[g].alive == 0 {
                cx.h.ee_sub_1619a0(0x3a, handle(obj));
            }
            self.ee_sub_181b60(g);
        }
    }

    /// FUN_00181e10(rider): state 1 ends after DAT_002be884
    pub fn ee_sub_181e10(&mut self, obj: usize, r: usize, cx: &mut Cx) {
        let mut s = self.riders[r].state;
        loop {
            if s == 1 && D_2BE884 < cx.t - self.riders[r].t_28 {
                self.ee_sub_181eb0(obj, r, 0, cx);
            }
            if self.riders[r].state == s {
                break;
            }
            s = self.riders[r].state;
        }
    }

    /// slot 15 FUN_00181f60: base FUN_001f36e0; while +0x1c0: per group the player's arc length on the rail
    /// (full search FUN_001e6f60 every rand(0.5, 0.75) s, else FUN_001e7380 from the last one), targets
    /// (FUN_00181cb0), then every rider moves (FUN_0011d1c0 + FUN_001e6f10) and swings (springs
    /// 0x2be898..0x2be8a4, trapezoid steps), FUN_00181e10
    pub fn ee_sub_181f60(&mut self, obj: usize, cx: &mut Cx) {
        cx.h.base(0x1f36e0, obj);
        if self.f_1c0 == 0 {
            return;
        }
        let mut g = 0;
        while g < self.groups.len() {
            // DAT_002dfb80: the player (REPLACED by guard::PLAYER_HANDLE)
            let q = cx.h.ee_sub_1a62e8(super::guard::PLAYER_HANDLE);
            if let Some(p) = self.groups[g].path.clone() {
                let spl = self.groups[g].spline.clone();
                let gr = &mut self.groups[g];
                if gr.t_2c < cx.t {
                    gr.l_28 = match &spl {
                        Some(sp) => sp.FUN_001e6f60(q).l,
                        None => FUN_001e6f60(&p, q).l,
                    };
                    gr.t_2c = cx.t + cx.rng.FUN_0011d448(0.5, 0.75);
                } else {
                    // FUN_001e7380(L, rail, q, ..) = incremental walk from L (FUN_001e6ed8)
                    gr.l_28 = match &spl {
                        Some(sp) => super::shared::path10::ee_sub_1e7380(sp, gr.l_28, q).l,
                        None => FUN_001e6ed8(gr.l_28, &p, q).l,
                    };
                }
                self.ee_sub_181cb0(g);
                let list = self.groups[g].list.clone();
                for r in list {
                    let rd = &mut self.riders[r];
                    let mut v = rd.v_40;
                    rd.l_3c = FUN_0011d1c0(rd.l_3c, rd.v_40, rd.tgt_44, cx.dt, T_2BE888, &mut v);
                    rd.v_40 = v;
                    let (pos, tan) = match &spl {
                        Some(sp) => sp.FUN_001e6f10(rd.l_3c),
                        None => FUN_001e6f10(&p, rd.l_3c),
                    };
                    rd.pos = pos;
                    rd.tan = tan;
                    let dt = cx.dt;
                    let (p50, v58, v5c) = (rd.p_50, rd.v_58, rd.v_5c);
                    let n58 = v58 + dt * (K_2BE898[0] * p50 + K_2BE898[1] * v58);
                    rd.v_58 = n58;
                    let n50 = p50 + dt * (v58 + n58) * 0.5;
                    let n5c = v5c + dt * (K_2BE898[2] * rd.p_54 + K_2BE898[3] * v5c);
                    rd.v_5c = n5c;
                    // quirk (ELF 0x182164 swc1 f7, 0x50): the roll step lands on +0x50 too
                    rd.p_50 = n50 + dt * (v5c + n5c) * 0.5;
                    self.ee_sub_181e10(obj, r, cx);
                }
            }
            g += 1;
        }
    }

    /// FUN_00182810(self, p): the alive rider nearest to p (squared distance, start DAT_002a5674 = FLT_MAX)
    pub fn ee_sub_182810(&self, p: [f32; 3]) -> Option<usize> {
        let mut best = f32::MAX;
        let mut out = None;
        for g in &self.groups {
            for &r in &g.list {
                let d = sub(self.riders[r].pos, p);
                let e = dot(d, d);
                if e < best {
                    best = e;
                    out = Some(r);
                }
            }
        }
        out
    }

    /// slot 74 FUN_001828a8(self, hit): only hits with +0x44 bit 3; the nearest rider takes +0x10; at < 1 it
    /// dies (fx +0x190, state 2), else it is kicked sideways ((Z x tangent) . +0x30 * 0.25), rolls rand(-15, 15)
    /// and goes to state 1. Returns whether the hit was taken.
    pub fn ee_sub_1828a8(&mut self, obj: usize, hit: &super::guard::slots::GuardHit, cx: &mut Cx) -> bool {
        if hit.f_44 & 8 == 0 {
            return false;
        }
        let Some(r) = self.ee_sub_182810([hit.pos[0], hit.pos[1], hit.pos[2]]) else { return true };
        self.riders[r].hp += hit.amount;
        if self.riders[r].hp < 1 {
            if self.b.f_190 != 0 {
                cx.h.ee_sub_15c198(self.b.f_190, 4, self.riders[r].pos);
            }
            self.ee_sub_181eb0(obj, r, 2, cx);
        } else {
            let rd = &mut self.riders[r];
            rd.v_58 = dot(z_cross(rd.tan), [hit.vec[0], hit.vec[1], hit.vec[2]]) * D_2BE8A8;
            rd.v_5c = cx.rng.FUN_0011d448(-15.0, 15.0);
            self.ee_sub_181eb0(obj, r, 1, cx);
        }
        true
    }

    /// slot 81 FUN_001829d8(self, other): with a touch damage +0x188 the rider nearest to other hits it
    /// (FUN_001f7598 {self, other, kind 0x100, -+0x188, rider position}); delivered -> fx +0x190, rider state 2.
    /// +0x18c = 0. (The original dereferences a null rider when the swarm is empty; the port skips.)
    pub fn ee_sub_1829d8(&mut self, obj: usize, other: u32, cx: &mut Cx) {
        if other == 0 || self.b.f_188 == 0 {
            return;
        }
        let p = cx.h.ee_sub_1a62e8(other);
        if let Some(r) = self.ee_sub_182810(p) {
            let rp = self.riders[r].pos;
            let hit = super::guard::slots::GuardHit { src: handle(obj), target: other, kind: 0x100, amount: -self.b.f_188, pos: [rp[0], rp[1], rp[2], 0.0], ..Default::default() };
            if cx.h.ee_sub_1f7598_ok(&hit) {
                if self.b.f_190 != 0 {
                    cx.h.ee_sub_15c198(self.b.f_190, 4, rp);
                }
                self.ee_sub_181eb0(obj, r, 2, cx);
            }
        }
        self.b.f_18c = 0;
    }

    /// slot 54 FUN_00182700(self, contact, side): FUN_0017f1a0; with +0x188 a touching class-71 object
    /// (FUN_00132b10(other, 0x47)) queues the deferred slot 81 (FUN_0017f5b0). Returns the queued `other`.
    pub fn ee_sub_182700(&mut self, obj: usize, other: u32, h: &mut dyn ClassHost) -> Option<u32> {
        self.b.ee_sub_17f1a0(obj, h);
        if self.b.f_188 != 0 && h.ee_sub_132b10(other, 0x47) != 0 && self.b.ee_sub_17f5b0() {
            return Some(other);
        }
        None
    }

    /// slot 59 FUN_001823c8(self, query): with query +0x10 bit 49, every alive rider against the body sphere:
    /// inside radius + +0x20 -> contact (depth = distance - that, normal body -> rider, (1, 0, 0) under 1e-4);
    /// else each query segment (start transformed by the +0x20 rows, end) is tested with FUN_0014c748 and the
    /// first hit gives a contact. REPLACED: the swept depth (VU sequence after FUN_002564b8, unclear) = entry
    /// normal . (end - entry) - body radius.
    pub fn ee_sub_1823c8(&self, obj: usize, q: &Query59, h: &mut dyn ClassHost) -> Vec<Contact59> {
        h.base(0x1f5990, obj);
        let mut out = Vec::new();
        if q.flags & (1 << 49) == 0 {
            return out;
        }
        for g in &self.groups {
            let rr = q.radius + g.r_20;
            for &r in &g.list {
                let c = self.riders[r].pos;
                let d = sub(c, q.pos);
                let l = dot(d, d).sqrt();
                let n = if l < 0.0001 { [1.0, 0.0, 0.0] } else { [d[0] / l, d[1] / l, d[2] / l] };
                if l < rr {
                    out.push(Contact59 { rider: r, depth: l - rr, normal: n });
                    continue;
                }
                for j in q.start..q.pts.len().min(q.ends.len()) {
                    let p = q.pts[j];
                    let m = &q.m;
                    let a = [0, 1, 2].map(|k| m[0][k] * p[0] + m[1][k] * p[1] + m[2][k] * p[2] + m[3][k]);
                    if let Some(x) = FUN_0014c748(rr, c, a, q.ends[j]) {
                        let (pe, ne, _) = x[0];
                        out.push(Contact59 { rider: r, depth: dot(ne, sub(q.ends[j], pe)) - q.radius, normal: ne });
                        break;
                    }
                }
            }
        }
        out
    }

    /// slot 48 FUN_00182780: base FUN_001f50e0; every group's rail bounds (FUN_001e7bb8) widened by +0x20
    /// (FUN_001f5480)
    pub fn ee_sub_182780(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1f50e0, obj);
        for g in &self.groups {
            let Some(p) = &g.path else { continue };
            let (lo, hi) = FUN_001e7bb8(p);
            for c in [lo, hi] {
                let (mut a, mut b) = self.bounds.unwrap_or((c, c));
                for k in 0..3 {
                    a[k] = a[k].min(c[k] - g.r_20);
                    b[k] = b[k].max(c[k] + g.r_20);
                }
                self.bounds = Some((a, b));
            }
        }
    }

    /// slot 62 FUN_00182778
    pub fn ee_sub_182778(&self) -> i32 {
        1
    }

    /// slot 18 FUN_001821c0: base FUN_001f3c70; with a model +0x1c4 every alive rider is drawn at its rail
    /// frame (FUN_0011b830 along the tangent), moved sideways by +0x50 (Z x tangent) when |+0x50| > 1, rolled
    /// by +0x54 (FUN_0011b500; axis = the forward row, the decompile drops the args) when |+0x54| > 0.01:
    /// FUN_001a5ee8(1, 1, model, m). Returns the matrices.
    pub fn ee_sub_1821c0(&self, obj: usize, h: &mut dyn ClassHost) -> Vec<[[f32; 4]; 4]> {
        h.so_draw_1f3c70(obj, 1.0);
        let mut out = Vec::new();
        if self.f_1c4 == 0 {
            return out;
        }
        for g in &self.groups {
            for &r in &g.list {
                let rd = &self.riders[r];
                let mut rows = FUN_0011b7e0(rd.tan);
                let mut p = rd.pos;
                if 1.0 < rd.p_50.abs() {
                    let c = z_cross(rd.tan);
                    p = [p[0] + c[0] * rd.p_50, p[1] + c[1] * rd.p_50, p[2] + c[2] * rd.p_50];
                }
                if 0.01 < rd.p_54.abs() {
                    let rot = FUN_0011b500(rd.p_54, rows[0]);
                    let r0 = rows;
                    rows = [0, 1, 2].map(|j| [0, 1, 2].map(|k| rot[j][0] * r0[0][k] + rot[j][1] * r0[1][k] + rot[j][2] * r0[2][k]));
                }
                let m = [[rows[0][0], rows[0][1], rows[0][2], 0.0], [rows[1][0], rows[1][1], rows[1][2], 0.0], [rows[2][0], rows[2][1], rows[2][2], 0.0], [p[0], p[1], p[2], 1.0]];
                h.ee_sub_1a5ee8(self.f_1c4, &m);
                out.push(m);
            }
        }
        out
    }

    /// rows 1397 (+0x1c0), 1398 (method FUN_001103b0 -> FUN_00182ab8 {oid, n, k, r}), 1399 (+0x1c4 raw oid) and
    /// the class-71 rows
    pub fn feed(&mut self, props: &[crate::engine::world::props::Prop]) {
        use crate::engine::world::props::{ArrayItem, PropValue};
        self.b.feed(props);
        for p in props {
            match (p.field, &p.value) {
                (1397, PropValue::Scalar(b)) => self.f_1c0 = super::k4::word(b),
                (1399, PropValue::Scalar(b)) => self.f_1c4 = super::k4::word(b),
                (1398, v) => {
                    let a: Vec<u32> = match v {
                        PropValue::Array(items) => items
                            .iter()
                            .flat_map(|it| match it {
                                ArrayItem::One(_, b) => vec![super::k4::word(b)],
                                ArrayItem::List(_, v) => v.iter().map(|b| super::k4::word(b)).collect(),
                            })
                            .collect(),
                        PropValue::Scalar(b) => b.chunks(4).map(super::k4::word).collect(),
                    };
                    let arg = |i: usize| a.get(i).copied().unwrap_or(0);
                    self.ee_sub_182ab8(f32::from_bits(arg(3)), arg(0) as i32, arg(1) as i32, arg(2) as i32);
                }
                _ => {}
            }
        }
    }
}

impl ClassSet {
    /// K14: construct + init (slot 6) + feed the `PORTED_K14` classes (clones read their source's props)
    pub(super) fn build_k14(&mut self, w: &World, h: &mut dyn ClassHost) {
        for (i, o) in w.objects.iter().enumerate() {
            if !PORTED_K14.iter().any(|x| x.0 == o.class) {
                continue;
            }
            let src = o.clone_of.unwrap_or(i);
            let props = w.objects.get(src).and_then(crate::engine::game::fields::object_props);
            let mut c = C2d0b30::default();
            c.ee_sub_1817e8(i, h);
            if let Some(p) = props {
                c.feed(p);
            }
            // the rail paths of the group oids (slot 14 resolves the same objects through FUN_0015e938 kind 4 =
            // the swarm's nearest relative), in the rail's world frame (vtable +0xfc, owner chain; K16)
            let copy = o.clone_of.map(|_| i);
            for g in &mut c.groups {
                let r = h.ee_sub_15e938(4, g.oid, handle(i));
                let rail = (r as usize).checked_sub(1).and_then(|j| super::shared::path10::rail(w, j, copy));
                (g.path, g.spline) = rail.map_or((None, None), |(p, s)| (Some(p), s));
            }
            // kind 0x201e / 0x2030 object references (assumption: resolved like the K12 kind-0x2030 rows)
            if c.b.f_190 != 0 {
                c.b.f_190 = h.ee_sub_15e938(0x201e, c.b.f_190 as i32, handle(i));
            }
            if c.f_1c4 != 0 {
                c.f_1c4 = h.ee_sub_15e938(0x2030, c.f_1c4 as i32, handle(i));
            }
            self.k14.push((i, Inst::K14(Box::new(c))));
        }
    }
    /// FUN_001d6b00(launcher, projectile) (Splice method 468 / class 69 / class 55 callers): launcher = a
    /// class-112 object index, projectile = a ported class-81 object index. FUN_0013bc60: +0x200 = launcher;
    /// velocity / orientation / spin on the xfLin body, then slot 77 (FUN_001b1e88). Returns the launch.
    pub fn k14_fire(&mut self, launcher: usize, proj: usize, h: &mut dyn ClassHost) -> Option<super::shared::launch::Launch> {
        self.k16_fire(launcher, proj, None, h)
    }
    /// K16: FUN_001357c0 launch tail (class 55): launcher +0x60 = 0 and +0x50 = `target` (world point, written
    /// into the launcher record) before FUN_001d6b00; `None` = the launcher's own +0x50 / +0x60 (class 69
    /// FUN_00226250, Splice method 468). A projectile not in play is first pulled from its pool (FUN_001cb250:
    /// FUN_001caec8 starts it at the spawner's +0x60 frame). REPLACED: the spawner frame = the launcher's world
    /// origin (FUN_001a62e8, owner chain), spawner and launcher records sit on the same owner.
    pub fn k16_fire(&mut self, launcher: usize, proj: usize, target: Option<[f32; 3]>, h: &mut dyn ClassHost) -> Option<super::shared::launch::Launch> {
        let l = self.k7.iter_mut().find_map(|x| match &mut x.1 {
            Inst::K7(super::k7::K7::C2d7f40(c)) if x.0 == launcher => {
                if let Some(t) = target {
                    c.f_60 = 0;
                    c.f_50 = [t[0], t[1], t[2], c.f_50[3]];
                }
                Some(c.clone())
            }
            _ => None,
        })?;
        if let Some(k) = self.k13.iter().position(|x| x.0 == proj) {
            if let Inst::K13(c) = &mut self.k13[k].1 {
                if !c.p.in_play || h.pool_take_place(handle(proj)) {
                    c.p.phys.pos = h.ee_sub_1a62e8(handle(launcher));
                    // HEIST: a frameless launcher record (the EP3 spotlight's 9070) starts the shot at its owner
                    // (spawner and launcher sit on the same owner). REPLACED: the owner's loaded world position.
                    if l.f_60 == 0 {
                        if let Some(&p) = self.launcher_owner.get(&launcher) {
                            c.p.phys.pos = p;
                        }
                    }
                }
            }
        }
        // +0x60 frame: kind 0x2002 object reference (resolved like the other object rows)
        let frame = if l.f_60 != 0 { h.ee_sub_15e938(0x2002, l.f_60, handle(launcher)) } else { 0 };
        let k = self.k13.iter().position(|x| x.0 == proj && matches!(x.1, Inst::K13(_)))?;
        let Inst::K13(c) = &mut self.k13[k].1 else { return None };
        let out = super::shared::launch::ee_sub_1d6b00(&super::shared::launch::Launcher { l: &l, frame }, 81, c.p.phys.pos, c.p.f_260, c.p.phys.gravity, &mut self.rng, h)?;
        c.p.f_200 = handle(launcher);
        c.p.in_play = true;
        c.p.phys.v = out.v;
        if let Some(r) = out.rot {
            c.p.phys.rot = r;
        }
        if let Some(w) = out.w {
            c.p.phys.w = w;
        }
        if let Some((f, v)) = out.slot77 {
            c.ee_sub_1b1e88(proj, f, v, &mut self.g.act_pool, h);
        }
        Some(out)
    }
    /// K14 class-76 state at object index `obj`
    pub fn k14_get(&mut self, obj: usize) -> Option<&mut C2d0b30> {
        self.k14.iter_mut().find(|x| x.0 == obj).and_then(|x| if let Inst::K14(c) = &mut x.1 { Some(&mut **c) } else { None })
    }
    /// K14 load-post: slot 14 (rails, riders, reset), slot 48 bounds
    pub(super) fn post_load_k14(&mut self, h: &mut dyn ClassHost) {
        let (t, dt) = (self.g.t, self.g.dt);
        for (i, x) in &mut self.k14 {
            if let Inst::K14(c) = x {
                let mut cx = Cx { t, dt, rng: &mut self.rng, h: &mut *h };
                c.ee_sub_181840(*i, false, &mut cx);
                c.ee_sub_182780(*i, &mut *cx.h);
            }
        }
    }
    /// K14 slot 74: a hit record delivered to object `obj`
    pub fn k14_hit(&mut self, obj: usize, hit: &super::guard::slots::GuardHit, h: &mut dyn ClassHost) -> bool {
        let (t, dt) = (self.g.t, self.g.dt);
        let Some(k) = self.k14.iter().position(|x| x.0 == obj) else { return false };
        let Inst::K14(c) = &mut self.k14[k].1 else { return false };
        let mut cx = Cx { t, dt, rng: &mut self.rng, h: &mut *h };
        c.ee_sub_1828a8(obj, hit, &mut cx)
    }
    /// K14 slot 54: `other` touches object `obj` (queues the deferred slot 81)
    pub fn k14_touch(&mut self, obj: usize, other: u32, h: &mut dyn ClassHost) {
        let Some(k) = self.k14.iter().position(|x| x.0 == obj) else { return };
        let Inst::K14(c) = &mut self.k14[k].1 else { return };
        if let Some(o) = c.ee_sub_182700(obj, other, h) {
            self.k14_deferred.push((obj, o));
        }
    }
    /// K14 per frame: slot 15, slot 18, then the deferred slot-81 calls (FUN_00149c80 phase)
    pub(super) fn frame_k14(&mut self, h: &mut dyn ClassHost) {
        let (t, dt) = (self.g.t, self.g.dt);
        for (i, x) in &mut self.k14 {
            // FEEL R33: slot 15 runs from the world update list only (LO +0x28 bit 60)
            if h.lo_removed(*i) {
                continue;
            }
            if let Inst::K14(c) = x {
                let mut cx = Cx { t, dt, rng: &mut self.rng, h: &mut *h };
                c.ee_sub_181f60(*i, &mut cx);
                c.ee_sub_1821c0(*i, &mut *cx.h);
            }
        }
        for (obj, other) in std::mem::take(&mut self.k14_deferred) {
            let Some(k) = self.k14.iter().position(|x| x.0 == obj) else { continue };
            if let Inst::K14(c) = &mut self.k14[k].1 {
                let mut cx = Cx { t, dt, rng: &mut self.rng, h: &mut *h };
                c.ee_sub_1829d8(obj, other, &mut cx);
            }
        }
    }
}
