//! WP-K21: the class-119 spawner hooks of K20 ported (classes/k20_119.rs keeps the slot code):
//! 1:1: FUN_00173f18 (spawn point choice: distance window, probe visibility FUN_00173768, rail samples),
//! FUN_00173768 (probe test vs the camera), FUN_00173588 (on-screen box), FUN_00174490 (place + clone the
//! guard), FUN_001cb250 / FUN_001caec8 / FUN_001cade0 / FUN_001ca978 / FUN_001ca930 / FUN_001ca958 (template
//! pool: pick + claim a copy), FUN_001ea3b0 (rail interval list), FUN_001f8240 / FUN_001f8278 / FUN_0020a5f8 /
//! FUN_001f82b8 (alert record list DAT_003e145c).
//! REPLACED: the hooks run inside `Host119`, a `ClassHost` wrapper the ClassSet hands to the K20 slot code
//! (`po_call(0x173f18 / 0x174490)`, `ee_call(0x1f8240)`, guard / object flag reads); a claimed pool copy is
//! built as a K6 guard (`guard::k6_make_guard`, placed by the spawner matrix) and runs slots 14 / 25 / 22 (the
//! vtable +0x44 add-to-world); vtable +0x4c (guard gone) drops it from the K6 set. The copy records
//! (entry +0x68, 0x50 B) are a map copy -> owner spawner; the spawner +0xe8 list keeps only its head.
//! K22: the FUN_0011a380(0x3dd4bc) set lives in `EngineHost::set_3dd4bc` (forwarded; FUN_001ca978's score 1
//! still unused). UNPORTED: rail +0x180 interval lists (none
//! loaded), the kind-1 helper object (FUN_0013e3c8(0x71) +8 flag / +0x54 routine -> guard +0x2e8), the 0xe
//! messages and vtable +0x154 / +0xd4 of FUN_001caec8, the copy-record transform copy.
#![allow(non_snake_case)]
use super::guard::{self, c56::Guard};
use super::k20_119::{C2cfa90, Point};
use super::*;
use crate::engine::gfx::attach::BoneWorlds;
use crate::engine::gfx::scene::Affine;
use crate::engine::world::loads_b4::B4Load;
use crate::engine::world::World;
use std::collections::HashMap;
use std::sync::Arc;

/// DAT_002be5b8: probe distance factor of the radius
pub const F_2BE5B8: f32 = 0.8;
/// DAT_002be5bc: minimum probe depth
pub const F_2BE5BC: f32 = 500.0;
/// DAT_002be5c0 / DAT_002be5c4: wide / tight screen boxes
pub const F_2BE5C0: f32 = 1.7;
pub const F_2BE5C4: f32 = 1.25;
/// DAT_002be5e4: minimum player distance of a spawn place
pub const F_2BE5E4: f32 = 1500.0;
/// DAT_002be5e8: rail sample spacing
pub const F_2BE5E8: f32 = 200.0;
/// DAT_002be5f4 / DAT_002be5f0: ground drop segment (z + 100 -> z - 350)
pub const F_2BE5F4: f32 = 100.0;
pub const F_2BE5F0: f32 = -350.0;
/// DAT_002a53e0
pub const FLT_MAX: f32 = f32::MAX;
/// LO +0x38 draw radius default (field 13 absent: 1e10, see c2d6fe8)
pub const R_DEFAULT: f32 = 1e10;
/// handles of alert records (not world objects)
pub const ALERT_BASE: u32 = 0x7000_0000;

/// g_cm (0x2ddf40) fields FUN_0014f308 reads: the camera matrix and the x / y projection scales
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CamView {
    pub mat: crate::engine::camera::math::M4,
    pub x_scale: f32,
    pub y_scale: f32,
}
impl CamView {
    /// FUN_0014f308(0x2ddf40, p) (= camera::cm::FUN_0014f308 on this copy): (x, y) screen, depth
    pub fn ee_sub_14f308(&self, p: [f32; 3]) -> [f32; 3] {
        let inv = crate::engine::camera::math::FUN_0011c318(&self.mat);
        let l = crate::engine::camera::math::xform_point(&inv, p);
        let mut y = l[2] / self.y_scale;
        let mut x = -l[1] / self.x_scale;
        if 0.0001 < l[0] {
            y /= l[0];
            x /= l[0];
        }
        [x, y, l[0]]
    }
}

/// One DAT_003e145c alert record (0xac bytes, FUN_0020a5f8 init)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Alert {
    pub f_0: f32,
    pub f_8: i32,
    /// +0xc owner key (the player)
    pub key: u32,
    pub f_10: i32,
    pub f_14: f32,
    pub f_18: f32,
    /// +0x28 state (read as float == 0 and int == 1), +0x2c guard
    pub f_28: u32,
    pub f_2c: u32,
}

/// FUN_001f8240(key): first record of `key` in list order
pub fn ee_sub_1f8240(l: &[Alert], key: u32) -> Option<usize> {
    l.iter().position(|a| a.key == key)
}
/// FUN_0020a5f8(rec, key) + FUN_0011a220(0x3e145c, rec)
pub fn ee_sub_20a5f8(l: &mut Vec<Alert>, key: u32) -> usize {
    l.push(Alert { key, f_14: 1.0, f_18: 2.0, f_8: -1, f_10: -1, ..Default::default() });
    l.len() - 1
}
/// FUN_001f8278(key): the record of `key`, created when missing
pub fn ee_sub_1f8278(l: &mut Vec<Alert>, key: u32) -> usize {
    ee_sub_1f8240(l, key).unwrap_or_else(|| ee_sub_20a5f8(l, key))
}
/// FUN_001f82b8(key, v, p): +0x10 = v, +0x14/+0x18 = p
pub fn ee_sub_1f82b8(l: &mut Vec<Alert>, key: u32, v: i32, p: Option<[f32; 2]>) {
    let k = ee_sub_1f8278(l, key);
    l[k].f_10 = v;
    if let Some(p) = p {
        l[k].f_14 = p[0];
        l[k].f_18 = p[1];
    }
}

/// FUN_00173588(lim, s): s depth > 1e-4 and |x|, |y| < lim
pub fn ee_sub_173588(lim: f32, s: [f32; 3]) -> bool {
    0.0001 < s[2] && -lim < s[0] && s[0] < lim && -lim < s[1] && s[1] < lim
}

/// FUN_001ea3b0(s, list): the [lo, hi] interval holding `s`
pub fn ee_sub_1ea3b0(s: f32, l: &[[f32; 2]]) -> Option<usize> {
    l.iter().position(|r| r[0] <= s && s <= r[1])
}

/// K21 ClassSet state
#[derive(Clone, Debug, Default)]
pub struct K21 {
    /// DAT_003e145c alert records
    pub alerts: Vec<Alert>,
    /// copy record +0 (owner spawner handle) per (entry, copy)
    pub copy_owner: HashMap<(usize, i32), u32>,
    /// spawner +0xe8 list head (entry, copy)
    pub cur: HashMap<u32, (usize, i32)>,
    /// guards handed out by FUN_001cb250 (object indices, spawn order)
    pub spawned: Vec<usize>,
}

/// spawn place out of FUN_00173f18: position, rotation rows, flip flag
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Place {
    pub pos: [f32; 3],
    pub mat: [[f32; 3]; 3],
    pub flag: u32,
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn len(a: [f32; 3]) -> f32 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}
/// pos + o.x r0 + o.y r1 + o.z r2 (vmulabc / vmaddabc / vmaddbc chain)
fn probe(pos: [f32; 3], m: &[[f32; 3]; 3], o: [f32; 3]) -> [f32; 3] {
    let mut q = pos;
    for k in 0..3 {
        q[k] += m[0][k] * o[0] + m[1][k] * o[1] + m[2][k] * o[2];
    }
    q
}

/// The world-side state FUN_00173f18 / FUN_00174490 read.
pub struct Host119<'a> {
    pub h: &'a mut dyn ClassHost,
    pub w: Arc<World>,
    /// the spawner (snapshot taken before slot 15) and its object index
    pub me: C2cfa90,
    pub obj: usize,
    pub t: f32,
    pub dt: f32,
    pub cam: [f32; 3],
    pub rng: &'a mut crate::engine::physics::driven::GameRng,
    pub k21: &'a mut K21,
    pub k6: &'a mut Vec<(usize, Inst)>,
    pub k6l: &'a guard::K6Level,
    pub live: &'a mut Vec<u32>,
    pub players: Vec<u32>,
    pub out: Place,
}

impl Host119<'_> {
    fn idx(&self, obj: u32) -> Option<usize> {
        (obj as usize).checked_sub(1).filter(|&i| i < self.w.objects.len())
    }
    fn guard(&self, obj: u32) -> Option<&Guard> {
        let i = self.idx(obj)?;
        self.k6.iter().find(|x| x.0 == i).and_then(|x| if let Inst::Guard(g) = &x.1 { Some(&**g) } else { None })
    }
    /// +0x28 bit 60: guards = in the K6 set, other objects = loaded (EngineHost::lo_28_bit60 convention)
    fn active(&self, obj: u32) -> bool {
        let Some(i) = self.idx(obj) else { return false };
        match crate::engine::world::class::ee_sub_15e988(self.w.objects[i].class, 0x38) {
            true => self.guard(obj).is_some(),
            // K18 convention (EngineHost::lo_28_bit60): every loaded non-guard object is in the world
            false => true,
        }
    }
    /// LO +0x38 (field 13), default 1e10; a class-6 spawner holds its template copy's (FUN_001c9b08)
    fn f_38(&self, obj: u32) -> f32 {
        let Some(mut i) = self.idx(obj) else { return R_DEFAULT };
        if let Some(e) = Self::entry_of(&self.w, i) {
            match self.w.entries[e].instances.last() {
                Some(&k) => i = k,
                None => return R_DEFAULT,
            }
        }
        let src = self.w.objects[i].clone_of.unwrap_or(i);
        crate::engine::game::fields::object_props(&self.w.objects[src])
            .and_then(|ps| ps.iter().find(|p| p.field == 13))
            .and_then(|p| match &p.value {
                crate::engine::world::props::PropValue::Scalar(b) if b.len() >= 4 => Some(f32::from_le_bytes([b[0], b[1], b[2], b[3]])),
                _ => None,
            })
            .unwrap_or(R_DEFAULT)
    }
    fn pos(&self, obj: u32) -> [f32; 3] {
        self.idx(obj).map_or([0.0; 3], |i| crate::engine::game::fields::ee_sub_1a62e8(&self.w, i))
    }
    fn mat(&self, obj: u32) -> [[f32; 3]; 3] {
        let m = self.idx(obj).map_or(crate::engine::gfx::scene::IDENTITY, |i| crate::engine::game::fields::ee_sub_1a6318(&self.w, i));
        [m[0], m[1], m[2]]
    }

    /// FUN_00173768(r, self, q): 0 = the probe is plainly in view (or too far / behind); 1 = outside the tight
    /// box, 2 = beyond +0x80, 3 = occluded (+0x8c)
    pub fn ee_sub_173768(&mut self, r: f32, q: [f32; 3]) -> i32 {
        let d = sub(q, self.cam);
        if len(d) > r * F_2BE5B8 {
            return 0;
        }
        let Some(s) = self.h.k21_cam_14f308(q) else { return 0 };
        if s[2] < F_2BE5BC || !ee_sub_173588(F_2BE5C0, s) {
            return 0;
        }
        if self.me.m.i(0x84) != 0 && !ee_sub_173588(F_2BE5C4, s) {
            return 1;
        }
        if self.me.m.i(0x88) != 0 && self.me.m.f(0x80) < len(d) {
            return 2;
        }
        if self.me.m.i(0x8c) != 0 && self.h.k21_blocked_14cc58(self.cam, q) {
            return 3;
        }
        0
    }

    fn probes_ok(&mut self, r: f32, pos: [f32; 3], m: &[[f32; 3]; 3]) -> bool {
        let l9c = self.me.l9c.clone();
        l9c.iter().all(|&o| self.ee_sub_173768(r, probe(pos, m, o)) != 0)
    }

    /// FUN_00173f18(self, &pos, &mat, &flag) with the player at `p`: the free, ready point nearest to the
    /// player at >= DAT_002be5e4 whose probes all pass; kind 1 samples the rail every DAT_002be5e8.
    pub fn ee_sub_173f18(&mut self, p: [f32; 3]) -> Option<usize> {
        let mut r_min = FLT_MAX;
        for &o in &self.me.l94_obj.clone() {
            r_min = r_min.min(self.f_38(o));
        }
        let mut best = FLT_MAX;
        let mut res: Option<(usize, Place)> = None;
        let points: Vec<Point> = self.me.points.clone();
        for (k, pt) in points.iter().enumerate() {
            if pt.busy || self.t < pt.ready || pt.obj == 0 || !self.active(pt.obj) {
                continue;
            }
            match pt.kind {
                0 | 2 => {
                    let pos = self.pos(pt.obj);
                    let mat = self.mat(pt.obj);
                    let r = if pt.kind == 0 { self.f_38(pt.obj) } else { r_min };
                    let d = len(sub(p, pos));
                    if F_2BE5E4 <= d && d <= best && self.probes_ok(r, pos, &mat) {
                        best = d;
                        res = Some((k, Place { pos, mat, flag: res.map_or(0, |x| x.1.flag) }));
                    }
                }
                1 => {
                    let Some(path) = self.h_path(pt.obj) else { continue };
                    let n = (path.FUN_001e6648() / F_2BE5E8) as i32;
                    for i in 0..n.max(0) {
                        let s = F_2BE5E8 * (i as f32 + self.rng.FUN_0011d448(0.0, 1.0));
                        // rail +0x180 interval list: none loaded (UNPORTED)
                        if ee_sub_1ea3b0(s, &[]).is_some() {
                            continue;
                        }
                        let (pos, mut tan) = path.FUN_001e6f10(s);
                        let d = len(sub(p, pos));
                        if !(F_2BE5E4 <= d && d <= best) {
                            continue;
                        }
                        let fw = self.h.cam_2ddfa0()[0];
                        let flag = (fw[0] * tan[0] + fw[1] * tan[1] + fw[2] * tan[2] < 0.0) as u32;
                        if flag == 0 {
                            tan = [-tan[0], -tan[1], -tan[2]];
                        }
                        // FUN_002458f0(y, x) = atan2, FUN_0011b500 rotation about z
                        let m4 = crate::engine::camera::math::FUN_0011b500(tan[1].atan2(tan[0]), [0.0, 0.0, 1.0]);
                        let mat = [[m4[0][0], m4[0][1], m4[0][2]], [m4[1][0], m4[1][1], m4[1][2]], [m4[2][0], m4[2][1], m4[2][2]]];
                        if self.probes_ok(r_min, pos, &mat) {
                            best = d;
                            res = Some((k, Place { pos, mat, flag }));
                        }
                    }
                }
                _ => {}
            }
        }
        let (k, pl) = res?;
        self.out = pl;
        Some(k)
    }

    fn h_path(&mut self, obj: u32) -> Option<Arc<crate::engine::player::p5_target::Path>> {
        let i = self.idx(obj)?;
        super::shared::path10::world_path(&self.w, i, None).map(Arc::new)
    }

    /// the class-6 in-place record (object +0xe0 = its template's directory entry) among the oid's objects
    pub fn spawner_of(w: &World, oid: i32) -> Option<usize> {
        w.find_oid(oid).iter().copied().find(|&k| Self::entry_of(w, k).is_some())
    }
    fn entry_of(w: &World, k: usize) -> Option<usize> {
        let crate::engine::world::objects::Payload::Loaded(crate::engine::world::objects::LoadData::B4(b)) = &w.objects[k].payload else { return None };
        let B4Load::Class6 { template, in_place: true, .. } = &**b else { return None };
        w.entries.iter().position(|e| e.dir.c == *template)
    }

    /// FUN_001ca978(spawner, copy): 0 locked, 4 unowned, else min over the copy objects (1 visible, 2 active,
    /// 3 out of the world)
    fn ee_sub_1ca978(&self, e: usize, copy: i32) -> i32 {
        let Some(_) = self.k21.copy_owner.get(&(e, copy)) else { return 4 };
        let en = &self.w.entries[e];
        let m = en.objects.len();
        let mut r = 5;
        for j in 0..m {
            let Some(&o) = en.instances.get(copy as usize * m + j) else { continue };
            let s = if self.active(o as u32 + 1) { 2 } else { 3 };
            r = r.min(s);
        }
        r
    }

    /// FUN_001cade0(spawner, min): the spawner's current copy while it scores >= 3, else the best scoring copy
    /// in [entry +0xc, n) with score >= min (later copies win ties)
    fn k21_1cade0(&self, sp: u32, e: usize, mut min: i32) -> Option<i32> {
        if let Some(&(ce, c)) = self.k21.cur.get(&sp) {
            if self.ee_sub_1ca978(ce, c) < 3 {
                return None;
            }
            return Some(c);
        }
        let en = &self.w.entries[e];
        let mut best = None;
        for c in en.load_count..en.dir.n {
            let s = self.ee_sub_1ca978(e, c);
            if min <= s {
                min = s;
                best = Some(c);
            }
        }
        best
    }

    /// FUN_001cb250(spawner, k) -> copy object handle placed by the spawner matrix `pl` (0 = none)
    pub fn ee_sub_1cb250(&mut self, sp: u32, k: i32, pl: Place) -> u32 {
        let Some(si) = self.idx(sp) else { return 0 };
        let Some(e) = Self::entry_of(&self.w, si) else { return 0 };
        if self.w.entries[e].objects.len() != 1 {
            return 0;
        }
        // FUN_001caec8
        let Some(copy) = self.k21_1cade0(sp, e, if k != 0 { 1 } else { 2 }) else { return 0 };
        if let Some(old) = self.k21.copy_owner.insert((e, copy), sp) {
            if old != sp {
                self.k21.cur.remove(&old); // FUN_001caa60
            }
        }
        self.k21.cur.insert(sp, (e, copy));
        // FUN_001ca930 -> entry +4 [copy]
        let Some(&o) = self.w.entries[e].instances.get(copy as usize) else { return 0 };
        let place: Affine = [pl.mat[0], pl.mat[1], pl.mat[2], pl.pos];
        // vtable +0x12c / +0x44: (re)build the copy at the place and add it to the world
        self.k6.retain(|x| x.0 != o);
        self.live.retain(|&g| g != o as u32 + 1);
        let Some(&(_, vt)) = guard::PORTED_K6.iter().find(|x| x.0 == self.w.objects[o].class) else { return 0 };
        let mut bw = BoneWorlds::default();
        let mut g = guard::k6_make_guard(&self.w, &mut bw, o, vt, Some(place), self.h);
        g.ee_sub_2061d8(o, self.h);
        g.ee_sub_206518(o, &self.players, self.t, self.h);
        g.ee_sub_2079e0(o, self.live, self.h);
        self.k6.push((o, Inst::Guard(g)));
        self.k21.spawned.push(o);
        o as u32 + 1
    }

    /// FUN_00174490(self, point, &pos, &mat, flag)
    pub fn ee_sub_174490(&mut self, pi: usize) -> u32 {
        let pt = self.me.points[pi].clone();
        let mut pl = self.out;
        let mut g = 0;
        match pt.kind {
            0 => g = self.ee_sub_1cb250(pt.obj, 0, pl),
            1 | 2 => {
                if let Some(q) = self.h.k21_ground_173610(F_2BE5F4, F_2BE5F0, pl.pos) {
                    pl.pos = q;
                }
                let n = self.me.l94_obj.len();
                let mut ord: Vec<usize> = (0..n).collect();
                for k in 0..n {
                    let r = self.rng.FUN_0011d3f8(0, n as i32 - 1) as usize;
                    ord.swap(k, r);
                }
                for &k in &ord {
                    // vtable +0xec / +0xf4: the template takes the place; FUN_001cb250 copies its matrix
                    g = self.ee_sub_1cb250(self.me.l94_obj[k], 0, pl);
                    if g != 0 {
                        break;
                    }
                }
                if pt.kind == 1 {
                    if g == 0 {
                        return 0;
                    }
                    // helper +8 = flag, guard +0x2e8 = helper +0x54 (UNPORTED helper object)
                    let routines = &self.k6l.cmds;
                    let (t, dt, o) = (self.t, self.dt, g as usize - 1);
                    if let Some((_, Inst::Guard(gd))) = self.k6.iter_mut().find(|x| x.0 == o) {
                        gd.k21_goal(o, -1, t, dt, routines, self.h);
                        gd.k21_goal(o, 1, t, dt, routines, self.h);
                    }
                }
            }
            _ => {}
        }
        if g != 0 {
            let o = g as usize - 1;
            if let Some((_, Inst::Guard(gd))) = self.k6.iter_mut().find(|x| x.0 == o) {
                gd.f_1000 = 1;
            }
        }
        g
    }
}

impl ClassHost for Host119<'_> {
    fn base(&mut self, addr: u32, obj: usize) {
        self.h.base(addr, obj)
    }
    fn ee_sub_1a61a0(&mut self, obj: u32) -> bool {
        self.h.ee_sub_1a61a0(obj)
    }
    fn ee_sub_11a380(&mut self, obj: u32) -> bool {
        self.h.ee_sub_11a380(obj)
    }
    fn ee_sub_11a2e8(&mut self, obj: u32) {
        self.h.ee_sub_11a2e8(obj)
    }
    fn ee_sub_15e938(&mut self, kind: i32, oid: i32, ctx: u32) -> u32 {
        self.h.ee_sub_15e938(kind, oid, ctx)
    }
    fn ee_sub_1a6ca8(&mut self, obj: usize, v: i32) {
        self.h.ee_sub_1a6ca8(obj, v)
    }
    fn ee_sub_1a62e8(&mut self, obj: u32) -> [f32; 3] {
        self.h.ee_sub_1a62e8(obj)
    }
    fn vcall_arg(&mut self, obj: u32, off: u32, arg: i32) {
        self.h.vcall_arg(obj, off, arg)
    }
    fn ee_call(&mut self, addr: u32, args: &[u32]) -> i64 {
        if addr == 0x1f8240 {
            return ee_sub_1f8240(&self.k21.alerts, args[0]).map_or(0, |k| (ALERT_BASE + k as u32) as i64);
        }
        if addr == 0x1c4840 {
            // FUN_001c4840(0x3dd490): the player handle
            return self.h.ee_sub_1c4840() as i64;
        }
        self.h.ee_call(addr, args)
    }
    fn obj_i32(&mut self, obj: u32, off: u32) -> i32 {
        if obj >= ALERT_BASE {
            let Some(a) = self.k21.alerts.get((obj - ALERT_BASE) as usize) else { return 0 };
            return match off {
                0 => a.f_0.to_bits() as i32,
                0xc => a.key as i32,
                0x28 => a.f_28 as i32,
                0x2c => a.f_2c as i32,
                _ => 0,
            };
        }
        if self.guard(obj).is_some() {
            return match off {
                // +0x330 / +0x84 (dead flag / last-active time): not modelled on the K6 guard
                _ => 0,
            };
        }
        self.h.obj_i32(obj, off)
    }
    fn obj_flags_28(&mut self, obj: u32) -> u64 {
        if self.idx(obj).is_some() {
            return (self.active(obj) as u64) << 60;
        }
        self.h.obj_flags_28(obj)
    }
    fn vcall(&mut self, obj: u32, off: u32) {
        if off == 0x4c && self.guard(obj).is_some() {
            let i = obj as usize - 1;
            self.k6.retain(|x| x.0 != i);
            self.live.retain(|&g| g != obj);
            return;
        }
        self.h.vcall(obj, off)
    }
    fn po_call(&mut self, addr: u32, obj: usize, off: u32, args: &[u32]) -> i64 {
        match addr {
            0x173f18 if obj == self.obj => {
                let p = [f32::from_bits(args[0]), f32::from_bits(args[1]), f32::from_bits(args[2])];
                self.ee_sub_173f18(p).map_or(0, |k| k as i64 + 1)
            }
            0x174490 if obj == self.obj => self.ee_sub_174490(args[0] as usize) as i64,
            _ => self.h.po_call(addr, obj, off, args),
        }
    }
}

/// K21 class 119 per frame (replaces `frame_k20_119`'s direct host)
impl ClassSet {
    pub(super) fn frame_k21_119(&mut self, h: &mut dyn ClassHost) {
        let Some(w) = h.k21_world() else {
            return self.frame_k20_119(h);
        };
        let t = self.g.t;
        let dt = self.g.dt;
        let cam = h.k21_cam_pos().unwrap_or(self.g.g_2ddfd0);
        let players: Vec<u32> = self.g.k6_players.iter().map(|p| p.0).collect();
        let mut k20 = std::mem::take(&mut self.k20);
        for (i, x) in &mut k20 {
            let Inst::K20Spawner(c) = x else { continue };
            let mut hh = Host119 {
                h: &mut *h,
                w: w.clone(),
                me: (**c).clone(),
                obj: *i,
                t,
                dt,
                cam,
                rng: &mut self.rng,
                k21: &mut self.k21,
                k6: &mut self.k6,
                k6l: &self.k6l,
                live: &mut self.g.k7_live,
                players: players.clone(),
                out: Place::default(),
            };
            c.ee_sub_173a68(*i, t, &mut hh);
        }
        k20.append(&mut self.k20);
        self.k20 = k20;
    }
}
