//! WP-K15: the projectile family C2cb780 (class 79) -> C2d3fa0 (class 80) -> {C2d3d20 (81, K13), C2d3aa0 (82)}.
//! (classes_gen.rs: 80 is the parent of 81 and 82; the DB vtable tree hangs 2d3d20 under 2d3aa0, which is
//! wrong. The K13 notes' "class 82 layer FUN_001b1db8" is the class-80 slot 6.)
//! Class 79 own slots 4 FUN_0013aae8, 5 FUN_00113f00, 14 FUN_0013aaa0, 15 FUN_0013ab58, 25 FUN_0013bb80, 57
//! FUN_0013aeb0, 59 FUN_0013ade8, 60 FUN_0013bbf8, 74 FUN_0013b2f0, 75 FUN_0013aa70, 76 FUN_0013b250 (slot 6
//! FUN_0013a9e8: K13) with FUN_0013b860 (explode, method 390), FUN_0013b5c8 (fuse, method 389), FUN_0013b428
//! (record the explosion point / orientation), FUN_0013bc60, FUN_0013b1c8, FUN_0013b7b8, FUN_00134310 1:1.
//! Class 80 own slots 5 FUN_00113f10, 11 FUN_001b1e38, 13 FUN_001b1e00, 60 FUN_001b1f40, 76 FUN_001b1f10 and
//! load methods 403 / 404 (FUN_001b2020 / FUN_001b2048) 1:1 (6 FUN_001b1db8 / 77 FUN_001b1e88: K13).
//! Class 82 own slots 5 FUN_00113f30, 6 FUN_001b21f0, 69 FUN_001b2468 (homing: cubic Hermite FUN_00117120 to
//! the launcher target by +0x278), 77 FUN_001b2228 with FUN_001b2398 1:1.
//! Class 81 (K13 `C2d3d20`) runs the inherited 79 / 80 slots through `Proj` (frame_k13).
//! REPLACED: the SO contact walk of slot 57 (+0xec +0x64 chains) -> `ClassHost::proj_contacts_13aeb0`; the
//! FUN_0013b5e8 victim query (FUN_0020fc60 sphere query + FUN_001a93c8 line tests) -> `proj_victims_13b5e8`
//! (sorted / deduplicated here: FUN_0013b7b8); FUN_0013ac18 (SO list DAT_003e13b4 segment tests + FUN_00133a38
//! broadphase pair toggles) -> `proj_los_13ac18`; slot 59's FUN_001a7c18 on such a record -> `ee_sub_1a7c18_los`;
//! FUN_00156698(DAT_003e14bc) explosion fx -> `ee_sub_156698_fx`; vtable slot 9 (+0x4c) after the explosion ->
//! `ClassHost::vcall(.., 0x4c)`; slot 3 (+0x1c) events 5 / 0x1b -> `vcall_1c`; FUN_001b2398's moving-frame
//! prediction FUN_002309e8 -> the static frame (as launch.rs); the class-82 slot-77 arrival estimate
//! FUN_0014c748(+0x274, ..) (arguments lost in the decompile) -> the exit parameter of the segment 0 -> d
//! through the sphere of radius +0x274 at 0. Objects only update in the world (+0x28 bit 60) or once
//! launched (`Proj::in_play`, also the in-world test of FUN_0013b860). Positions are the own xfLin
//! (top-level objects).
#![allow(non_snake_case)]
use super::shared::{self, launch, proj::Proj};
use super::*;
use crate::engine::camera::looks::FUN_0011b7e0;
use crate::engine::physics::driver::{FUN_001f9b20, FUN_0022de88, FUN_00230380};
use crate::engine::player::p5_states::FUN_0011ba08;
use crate::engine::player::p5_target::FUN_00117120;
use crate::engine::player::p7_damage::FUN_0011e9f8;

/// K15 projectile world class id -> vtable (kept in `ClassSet::k15`)
pub const PORTED_K15_79: &[(ClassId, u32)] = &[(79, 0x2cb780), (80, 0x2d3fa0), (82, 0x2d3aa0)];

/// slot 5 FUN_00113f00 / FUN_00113f10 / FUN_00113f30 (class descriptor getters)
pub fn slot5(class: ClassId) -> Option<u32> {
    match class {
        79 => Some(0x2bb114),
        80 => Some(0x2bb128),
        82 => Some(0x2bb150),
        _ => None,
    }
}

/// DAT_002bc9d0: classes a sticking (kind 1) projectile ignores (FUN_0013b1c8, the object or any parent)
pub const C_2BC9D0: [i32; 4] = [0x33, 0x64, 0x4f, 0x67];
/// DAT_002bc9e0: explosion push
pub const D_2BC9E0: f32 = 500.0;
/// DAT_002a4b10: PMF {vindex 0x4e} -> slot 76 (deferred by slot 57)
pub const PMF_2A4B10_SLOT: u32 = 76;
/// class-82 row 412 default / DAT_002a2dd0
const ZERO: [f32; 3] = [0.0; 3];

/// one line-of-sight record (+0x1fc, 0x30 bytes: +0 point, +0x10 normal, +0x20 plane distance, +0x24 SO root)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Los {
    pub pos: [f32; 3],
    pub n: [f32; 3],
    pub d: f32,
    pub so: u32,
}

/// one SO contact of slot 57 seen from the projectile: the other half's object (+0xc / +0x4c), its SO (+0x8 /
/// +0x48) and the contact point (+0x10)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ProjContact {
    pub obj: u32,
    pub so: u32,
    pub point: [f32; 3],
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn len(a: [f32; 3]) -> f32 {
    shared::len3(a)
}
fn unit_or(v: [f32; 3], d: [f32; 3]) -> [f32; 3] {
    let l = len(v);
    if l < 0.0001 { d } else { shared::scale3(v, 1.0 / l) }
}
fn handle(obj: usize) -> u32 {
    obj as u32 + 1
}

/// argument words of a method row (Array items / Scalar), item by item
pub fn method_args(v: &crate::engine::world::props::PropValue) -> Vec<u32> {
    use super::k4::word;
    use crate::engine::world::props::{ArrayItem, PropValue};
    match v {
        PropValue::Array(items) => items
            .iter()
            .flat_map(|it| match it {
                ArrayItem::One(_, b) => b.chunks(4).map(word).collect::<Vec<_>>(),
                ArrayItem::List(_, v) => v.iter().flat_map(|b| b.chunks(4).map(word).collect::<Vec<_>>()).collect(),
            })
            .collect(),
        PropValue::Scalar(b) => b.chunks(4).map(word).collect(),
    }
}

/// per-frame context (DAT_002dded0 time, DAT_002dded4 dt)
pub struct Cx<'a> {
    pub t: f32,
    pub dt: f32,
    pub h: &'a mut dyn ClassHost,
}

impl Proj {
    /// FUN_0013b5c8(d, self) (method 389): fuse +0x154 = min(+0x154, t + d)
    pub fn ee_sub_13b5c8(&mut self, d: f32, t: f32) {
        let x = t + d;
        if x < self.f_154 {
            self.f_154 = x;
        }
    }

    /// FUN_0013bc60(self, owner): +0x200 = owner (FUN_00134320 on a change)
    pub fn ee_sub_13bc60(&mut self, obj: usize, owner: u32, h: &mut dyn ClassHost) {
        if owner != self.f_200 {
            self.f_200 = owner;
            h.ee_sub_134320(obj);
        }
    }

    /// FUN_0013b1c8(self, other): neither `other` nor a parent (+0x10) is one of `C_2BC9D0`
    pub fn ee_sub_13b1c8(other: u32, h: &mut dyn ClassHost) -> bool {
        let mut o = other;
        let mut n = 0;
        while o != 0 && n < 64 {
            if C_2BC9D0.iter().any(|&c| h.ee_sub_132b10(o, c) != 0) {
                return false;
            }
            o = h.parent_10(o);
            n += 1;
        }
        true
    }

    /// FUN_0013b428(self, pos, dir): once (+0x174): +0x180 = pos, +0x1c0 / +0x1d0 = current velocities (slots 33
    /// / 34), +0x190 orientation by +0x170: 1 look along the velocity (FUN_0011b830), 0 / 2 turn row 2 / row 0 to
    /// `dir` (FUN_0011ba08), 3 the current orientation (FUN_001a6318)
    pub fn ee_sub_13b428(&mut self, pos: [f32; 3], dir: [f32; 3]) {
        if self.f_174 != 0 {
            return;
        }
        self.f_180 = pos;
        self.f_174 = 1;
        self.f_1c0 = self.phys.v;
        self.f_1d0 = self.phys.w;
        let r = self.phys.rot;
        match self.f_170 {
            1 => self.f_190 = FUN_0011b7e0(self.f_1c0),
            0 | 2 => {
                let a = if self.f_170 == 0 { r[2] } else { r[0] };
                let m = FUN_0011ba08(a, dir);
                self.f_190 = r.map(|row| [0, 1, 2].map(|k| row[0] * m[0][k] + row[1] * m[1][k] + row[2] * m[2][k]));
            }
            3 => self.f_190 = r,
            _ => {}
        }
    }

    /// FUN_0013b860 (method 390): in the world: record at the position (+Z), with +0x164 == 0 hit every victim
    /// (FUN_0013b5e8: within max(+0x15c, +0x160), line radius +0x160 for class 0x34 else +0x15c) with {self,
    /// victim, +0x1ec, +0x1f0, +0x180, unit(victim - +0x180) * 500, 500} (FUN_001f7598); back to the recorded
    /// state (slots 29 / 30 / 35 / 36); message 9 {self, +0x16c, +0x15c} to +0x168; with +0x1e0 the fx at the
    /// position for the owner (+0x200 when it is a class-0x70 object with another root); slot 9; +0x174 = 0,
    /// +0x154 = FLT_MAX; slot 3 event 5
    pub fn ee_sub_13b860(&mut self, obj: usize, h: &mut dyn ClassHost) {
        let me = handle(obj);
        if !self.in_play && !h.in_world_28(me) {
            return;
        }
        self.ee_sub_13b428(self.phys.pos, [0.0, 0.0, 1.0]);
        if self.f_164 == 0 {
            let r = if self.f_15c < self.f_160 { self.f_160 } else { self.f_15c };
            let mut v = h.proj_victims_13b5e8(me, self.f_180, r, self.f_15c, self.f_160);
            // FUN_0013b7b8: sort + unique
            v.sort_unstable();
            v.dedup();
            for t in v {
                let d = unit_or(sub(h.ee_sub_1a62e8(t), self.f_180), [0.0; 3]);
                let p = self.f_180;
                let hit = guard::slots::GuardHit {
                    src: me,
                    target: t,
                    kind: self.f_1ec as i32,
                    amount: self.f_1f0,
                    pos: [p[0], p[1], p[2], 0.0],
                    vec: [d[0] * D_2BC9E0, d[1] * D_2BC9E0, d[2] * D_2BC9E0, 0.0],
                    f_44: 0,
                };
                h.ee_sub_1f7598(&hit);
            }
        }
        self.phys.pos = self.f_180;
        self.phys.rot = self.f_190;
        self.phys.v = self.f_1c0;
        self.phys.w = self.f_1d0;
        if self.f_168 != 0 {
            h.vcall_19c_msg(self.f_168, 9, me);
        }
        if self.f_1e0 != 0 {
            let o = self.f_200;
            let mut owner = me;
            if o != me && h.ee_sub_132b10(o, 0x70) != 0 {
                let r = h.root_14(o);
                if r != 0 && r != o {
                    owner = r;
                }
            }
            h.ee_sub_156698_fx(self.phys.pos, owner);
        }
        h.vcall(me, 0x4c);
        self.f_174 = 0;
        self.f_154 = f32::MAX;
        h.vcall_1c(obj, 5, 0);
    }

    /// slot 4 FUN_0013aae8(self, msg, about): FUN_00133590; message 4 about this kind-0 projectile -> fuse now
    pub fn ee_sub_13aae8(&mut self, obj: usize, msg: i32, about: u32, cx: &mut Cx) {
        cx.h.base(0x133590, obj);
        if msg == 4 && about == handle(obj) && self.f_150 == 0 {
            self.ee_sub_13b5c8(0.0, cx.t);
        }
    }

    /// slot 14 FUN_0013aaa0: FUN_001f36a8; with +0x1f4 the 4 line-of-sight records (FUN_001ae1d0(0xc0))
    pub fn ee_sub_13aaa0(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1f36a8, obj);
        if self.f_1f4 != 0 {
            self.los = Vec::with_capacity(4);
        }
    }

    /// slot 15 FUN_0013ab58: FUN_001f36e0; before the fuse and at least +0x158 fast: (+0x1f4: FUN_0013ac18)
    /// keep flying; else explode (FUN_0013b860)
    pub fn ee_sub_13ab58(&mut self, obj: usize, cx: &mut Cx) {
        cx.h.base(0x1f36e0, obj);
        if cx.t < self.f_154 && f32::from_bits(self.f_158) <= len(self.phys.v) {
            if self.f_1f4 != 0 {
                let prev = std::mem::take(&mut self.los);
                self.los = cx.h.proj_los_13ac18(handle(obj), &prev);
                self.los.truncate(4);
            }
            return;
        }
        self.ee_sub_13b860(obj, cx.h);
    }

    /// slot 25 FUN_0013bb80: FUN_001cb518; +0x174 = 0, +0x154 = FLT_MAX, owner self, xfLin / xfAng acts off
    /// (FUN_001f5720 / FUN_001f57c0), FUN_001f6860(0), +0x1e8 = 0, FUN_001a58d0(0, 0xf)
    pub fn ee_sub_13bb80(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1cb518, obj);
        self.f_174 = 0;
        self.f_154 = f32::MAX;
        self.ee_sub_13bc60(obj, handle(obj), h);
        h.ee_sub_1f5720(obj, 0);
        h.ee_sub_1f57c0(obj, 0);
        h.base(0x1f6860, obj);
        self.f_1e8 = 0;
        h.ee_sub_1a58d0(obj, 0, 0xf);
    }

    /// slot 57 FUN_0013aeb0: FUN_001f4f10; kind (+0x150) 1 needs +0x1e4 and no stick (+0x1e8), 0 / 2 no recorded
    /// explosion; the contact nearest the centre that qualifies (1: FUN_0013b1c8 of the object, 0: any, 2: the
    /// SO is a class-0x32 object with FUN_001c4098) -> kind 1 returns it for the deferred slot 76
    /// (FUN_00149a50, PMF 0x2a4b10), 0 / 2 record there along the velocity (+X) and fuse now
    pub fn ee_sub_13aeb0(&mut self, obj: usize, contacts: &[ProjContact], cx: &mut Cx) -> Option<u32> {
        cx.h.base(0x1f4f10, obj);
        let k = self.f_150 as i32;
        match k {
            1 if self.f_1e4 == 0 || self.f_1e8 != 0 => return None,
            0 | 2 if self.f_174 != 0 => return None,
            _ => {}
        }
        let c = self.phys.pos;
        let mut best: Option<&ProjContact> = None;
        let mut bd = f32::MAX;
        for x in contacts {
            let d = len(sub(x.point, c));
            if d < bd {
                let ok = match k {
                    1 => Proj::ee_sub_13b1c8(x.obj, cx.h),
                    0 => true,
                    2 => cx.h.ee_sub_132b10(x.so, 0x32) != 0 && cx.h.ee_sub_1c4098(x.so),
                    _ => false,
                };
                if ok {
                    best = Some(x);
                    bd = d;
                }
            }
        }
        let b = *best?;
        match k {
            1 => Some(b.obj),
            0 | 2 => {
                let n = unit_or(self.phys.v, [1.0, 0.0, 0.0]);
                self.ee_sub_13b428(b.point, n);
                self.ee_sub_13b5c8(0.0, cx.t);
                None
            }
            _ => None,
        }
    }

    /// slot 59 FUN_0013ade8(self, contact): FUN_001f5990 (empty); with +0x1f4 the line-of-sight record of the
    /// contact object's SO root -> FUN_001a7c18 (REPLACED hook)
    pub fn ee_sub_13ade8(&mut self, obj: usize, other: u32, h: &mut dyn ClassHost) {
        if self.f_1f4 == 0 {
            return;
        }
        let root = h.so_root_ec60(other);
        if let Some(r) = self.los.iter().find(|r| r.so == root).copied() {
            h.ee_sub_1a7c18_los(obj, other, &r);
        }
    }

    /// slot 60 FUN_0013bbf8(self, other): FUN_001f4fc8 (both report vtable +0x1f4; the SO flag +0x148 bit 26 is
    /// not modelled), at least 1 when `other` shares the owner's (+0x200) root (+0x14)
    pub fn ee_sub_13bbf8(&self, obj: usize, other: u32, h: &mut dyn ClassHost) -> i64 {
        let mut r = (h.vcall_ret(handle(obj), 500) != 0 && h.vcall_ret(other, 500) != 0) as i64;
        if h.root_14(other) == h.root_14(self.f_200) {
            r = r.max(1);
        }
        r
    }

    /// class 80 slot 60 FUN_001b1f40: FUN_0013bbf8; at least 1 for an ignored oid (+0x23c, the other's +0x18)
    /// or class (+0x250)
    pub fn ee_sub_1b1f40(&self, obj: usize, other: u32, h: &mut dyn ClassHost) -> i64 {
        let mut r = self.ee_sub_13bbf8(obj, other, h);
        for &o in &self.f_23c {
            if o == h.obj_i32(other, 0x18) {
                r = r.max(1);
            }
        }
        for &c in &self.f_250 {
            if h.ee_sub_132b10(other, c) != 0 {
                r = r.max(1);
            }
        }
        r
    }

    /// slot 74 FUN_0013b2f0(self, hit): not kind 1: taken = FUN_001f7588 (+0x44); +0x44 bit 3 records the
    /// explosion here along unit(+0x30) (+Z) and fuses (0.25 s for a class-0x4f source, else now)
    pub fn ee_sub_13b2f0(&mut self, hit: &guard::slots::GuardHit, cx: &mut Cx) -> bool {
        if self.f_150 == 1 {
            return false;
        }
        let taken = hit.f_44 != 0;
        if taken && hit.f_44 & 8 != 0 {
            let n = unit_or([hit.vec[0], hit.vec[1], hit.vec[2]], [0.0, 0.0, 1.0]);
            self.ee_sub_13b428(self.phys.pos, n);
            let d = if cx.h.ee_sub_132b10(hit.src, 0x4f) != 0 { 0.25 } else { 0.0 };
            self.ee_sub_13b5c8(d, cx.t);
        }
        taken
    }

    /// slot 75 FUN_0013aa70: FUN_001f6068; owner self
    pub fn ee_sub_13aa70(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1f6068, obj);
        self.ee_sub_13bc60(obj, handle(obj), h);
    }

    /// FUN_001f4898 / FUN_001f4930(self, 0): xfLin / xfAng dynamic (FUN_002303a0 kind 1 unless FUN_00230310)
    /// at rest (top-level: FUN_0022fcc8 = identity)
    fn rest(&mut self, obj: usize, h: &mut dyn ClassHost) {
        let dynamic = matches!(self.phys.kind, 1 | 2 | 3 | 7 | 8 | 9 | 10 | 0xb | 0xc | 0x11..=0x15);
        if !dynamic {
            h.ee_sub_2303a0_off(handle(obj), 0x58, 1);
            h.ee_sub_2303a0_off(handle(obj), 0x5c, 1);
            self.phys.kind = 1;
        }
        self.phys.v = ZERO;
        self.phys.w = ZERO;
        h.base(0x1f4898, obj);
    }

    /// slot 76 FUN_0013b250(self, other): stick: +0x1e8 = other; FUN_001a58d0(other unless its +0x18 == 1, 3);
    /// at rest (FUN_001f4898 / FUN_001f4930(0)); acts on (FUN_001f5720 / FUN_001f57c0(1)); slot 3 event 0x1b
    pub fn ee_sub_13b250(&mut self, obj: usize, other: u32, h: &mut dyn ClassHost) {
        self.f_1e8 = other;
        let o = if other != 0 && h.obj_i32(other, 0x18) == 1 { 0 } else { other };
        h.ee_sub_1a58d0_to(obj, o, 3);
        self.rest(obj, h);
        h.ee_sub_1f5720(obj, 1);
        h.ee_sub_1f57c0(obj, 1);
        h.vcall_1c(obj, 0x1b, 0);
    }

    /// class 80 slot 76 FUN_001b1f10: FUN_0013b250; act record +0x268 released (FUN_0012e460)
    pub fn ee_sub_1b1f10(&mut self, obj: usize, other: u32, h: &mut dyn ClassHost) {
        self.ee_sub_13b250(obj, other, h);
        h.ee_sub_12e460(&mut self.f_268);
    }

    /// class 80 slot 11 FUN_001b1e38: FUN_001f2d48; anim +0x234 stopped (FUN_001305b8(0, 1.0, 0, ..)); act
    /// record +0x268 released
    pub fn ee_sub_1b1e38(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1f2d48, obj);
        shared::anim::ee_sub_1305b8(0, 1.0, 0, handle(obj), 0, &mut self.f_234, h);
        h.ee_sub_12e460(&mut self.f_268);
    }

    /// class 80 slot 13 FUN_001b1e00: FUN_001f31e0; FUN_001a64e8(self, 1, 0x2c0258)
    pub fn ee_sub_1b1e00(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1f31e0, obj);
        h.ee_sub_1a64e8(obj, 1, 0x2c0258);
    }

    /// class 80 slot 77 FUN_001b1e88(launcher frame, v): +0x210 / +0x220, anim +0x230 (re)played into +0x234,
    /// with +0x264 a kind-4 xfAng act record (FUN_0012dca0) -> +0x268 (K13 port, shared here)
    pub fn ee_sub_1b1e88(&mut self, obj: usize, launcher: u32, v: [f32; 4], pool: &mut shared::act::ActPool, h: &mut dyn ClassHost) {
        self.f_210 = launcher;
        self.f_220 = v;
        shared::anim::ee_sub_1305b8(0, 1.0, self.f_230, handle(obj), 8, &mut self.f_234, h);
        if self.f_264 != 0 {
            self.f_268 = shared::act::ee_sub_12dca0(pool, self.f_26c.to_bits(), 0, handle(obj), &mut self.xf_ang, 0, 5, h);
        }
    }

    /// class 82 slot 6 FUN_001b21f0: FUN_001b1db8; +0x274 = 1500
    pub fn ee_sub_1b21f0(&mut self, obj: usize, h: &mut dyn ClassHost) {
        self.ee_sub_1b1db8(obj, h);
        self.f_274 = 1500.0;
    }

    /// FUN_001b2398(dt, out, self): the launcher target: +0x220 (no launcher) or the frame +0x210 * +0x220 (the
    /// FUN_002309e8 prediction over dt REPLACED by the static frame translation)
    pub fn ee_sub_1b2398(&self, h: &mut dyn ClassHost) -> [f32; 3] {
        let v = [self.f_220[0], self.f_220[1], self.f_220[2]];
        if self.f_210 == 0 {
            v
        } else {
            let o = h.ee_sub_1a62e8(self.f_210);
            [o[0] + v[0], o[1] + v[1], o[2] + v[2]]
        }
    }

    /// class 82 slot 77 FUN_001b2228: FUN_001b1e88; arrival +0x278 = t + +0x270, or without +0x270 the estimate
    /// t + 1 / max(s, 0.25), s = exit parameter of d = (target - p) 1.5 + clamp(-0.5 v, 0.75 +0x274) through the
    /// +0x274 sphere (REPLACED: FUN_0014c748 arguments lost)
    pub fn ee_sub_1b2228(&mut self, obj: usize, launcher: u32, v: [f32; 4], t: f32, pool: &mut shared::act::ActPool, h: &mut dyn ClassHost) {
        self.ee_sub_1b1e88(obj, launcher, v, pool, h);
        if 0.0 < self.f_270 {
            self.f_278 = t + self.f_270;
            return;
        }
        let tg = self.ee_sub_1b2398(h);
        let c = FUN_0011e9f8(self.f_274 * 0.75, shared::scale3(self.phys.v, -0.5));
        let dp = sub(tg, self.phys.pos);
        let d = [0, 1, 2].map(|k| dp[k] * 1.5 + c[k]);
        let s = k14::FUN_0014c748(self.f_274, ZERO, ZERO, d).map(|x| x[1].2).unwrap_or(1.0);
        self.f_278 = t + 1.0 / if s < 0.25 { 0.25 } else { s };
    }

    /// class 82 slot 69 FUN_001b2468 (not stuck): T = max(+0x278 - t, 0) + dt; target (FUN_001b2398); arrival
    /// velocity +0x280, or when it is zero (FFloatsNear 1e-4) (target - p) 1.5 / T - 0.5 v; the Hermite velocity
    /// after dt (FUN_00117120) -> accel (v' - v) / dt (FUN_001fac30). No gravity.
    pub fn ee_sub_1b2468(&mut self, _obj: usize, t: f32, dt: f32, h: &mut dyn ClassHost) {
        if self.f_1e8 != 0 {
            return;
        }
        let mut tt = self.f_278 - t;
        if tt < 0.0 {
            tt = 0.0;
        }
        let tt = tt + dt;
        let tg = self.ee_sub_1b2398(h);
        let p = self.phys.pos;
        let v = self.phys.v;
        let zero = self.f_280.iter().all(|x| x.abs() < 1e-4);
        let v1 = if zero {
            let k = 1.5 / tt;
            let d = sub(tg, p);
            [0, 1, 2].map(|i| d[i] * k + v[i] * -0.5)
        } else {
            self.f_280
        };
        let (_, vel, _) = FUN_00117120(tt, dt, 1.0, p, v, tg, v1);
        let a = [0, 1, 2].map(|i| shared::vu_mul(vel[i] - v[i], shared::vu_rcp(dt)));
        if FUN_00230380(self.phys.kind) {
            self.phys.a = [0, 1, 2].map(|i| self.phys.a[i] + a[i]);
        }
    }
}

/// a direct class 79 / 80 / 82 instance
#[derive(Clone, Debug, Default, PartialEq)]
pub struct C2cb780 {
    pub cls: ClassId,
    pub p: Proj,
}

impl C2cb780 {
    /// FUN_001f3980 physics part with the class's slot 69 (79 / 80: ALO gravity FUN_001f9b20, 82: FUN_001b2468)
    pub fn physics(&mut self, obj: usize, t: f32, dt: f32, h: &mut dyn ClassHost) {
        h.base(0x1f3980, obj);
        let b = &mut self.p.phys;
        b.prev_v = b.v;
        b.prev_w = b.w;
        if matches!(b.kind, 1 | 2) {
            b.a = ZERO;
            b.alpha = ZERO;
            if self.cls == 82 {
                self.p.ee_sub_1b2468(obj, t, dt, h);
            } else {
                FUN_001f9b20(&mut self.p.phys);
            }
        }
        FUN_0022de88(&mut self.p.phys, dt);
    }

    /// slot 60 of the class (79: FUN_0013bbf8, 80 / 82: FUN_001b1f40)
    pub fn slot60(&self, obj: usize, other: u32, h: &mut dyn ClassHost) -> i64 {
        if self.cls == 79 { self.p.ee_sub_13bbf8(obj, other, h) } else { self.p.ee_sub_1b1f40(obj, other, h) }
    }

    /// slot 76 of the class (79: FUN_0013b250, 80 / 82: FUN_001b1f10)
    pub fn slot76(&mut self, obj: usize, other: u32, h: &mut dyn ClassHost) {
        if self.cls == 79 { self.p.ee_sub_13b250(obj, other, h) } else { self.p.ee_sub_1b1f10(obj, other, h) }
    }
}

/// per frame of one projectile `Proj` (also K13 class 81): slot 57 with the host contacts (deferred slot 76
/// returned), slot 15
pub fn frame_proj(p: &mut Proj, obj: usize, cx: &mut Cx) -> Option<u32> {
    let contacts = cx.h.proj_contacts_13aeb0(obj);
    let d = if contacts.is_empty() { None } else { p.ee_sub_13aeb0(obj, &contacts, cx) };
    p.ee_sub_13ab58(obj, cx);
    d
}

impl ClassSet {
    /// K15 classes 79 / 80 / 82: construct + init (slot 6) + feed; object rows +0x168 / +0x16c resolved
    pub(super) fn build_k15_79(&mut self, w: &World, h: &mut dyn ClassHost) {
        let mut bw = crate::engine::gfx::attach::BoneWorlds::default();
        for (i, o) in w.objects.iter().enumerate() {
            if !PORTED_K15_79.iter().any(|x| x.0 == o.class) {
                continue;
            }
            let src = o.clone_of.unwrap_or(i);
            let props = w.objects.get(src).and_then(crate::engine::game::fields::object_props);
            let mut c = C2cb780 { cls: o.class, ..Default::default() };
            match o.class {
                79 => c.p.ee_sub_13a9e8(i, h),
                80 => c.p.ee_sub_1b1db8(i, h),
                _ => c.p.ee_sub_1b21f0(i, h),
            }
            if let Some(p) = props {
                c.p.feed(p, if o.class == 79 { &[] } else { shared::proj::ROWS_82 });
            }
            if c.p.f_168 != 0 {
                c.p.f_168 = h.ee_sub_15e938(0x2002, c.p.f_168 as i32, handle(i));
            }
            c.p.phys.pos = bw.get(w, i, 0)[3];
            if o.class != 79 {
                c.p.xf_ang = shared::act::XfActs { xf: handle(i) << 8 | 0x5c, kind_98: 2, ..Default::default() };
            }
            self.k15.push((i, Inst::K15c(Box::new(c))));
        }
    }
    /// K15 projectile (79 / 80 / 82) at object index `obj`
    pub fn k15_79_get(&mut self, obj: usize) -> Option<&mut C2cb780> {
        self.k15.iter_mut().find(|x| x.0 == obj).and_then(|x| if let Inst::K15c(c) = &mut x.1 { Some(&mut **c) } else { None })
    }
    /// K15 projectiles load-post: slot 13 (80 / 82), slot 14
    pub(super) fn post_load_k15_79(&mut self, h: &mut dyn ClassHost) {
        for (i, x) in &mut self.k15 {
            if let Inst::K15c(c) = x {
                if c.cls != 79 {
                    c.p.ee_sub_1b1e00(*i, h);
                }
                c.p.ee_sub_13aaa0(*i, h);
            }
        }
    }
    /// K15 slot 74 of a projectile (79 / 80 / 82, or the K13 class 81)
    pub fn k15_proj_hit(&mut self, obj: usize, hit: &guard::slots::GuardHit, h: &mut dyn ClassHost) -> Option<bool> {
        let (t, dt) = (self.g.t, self.g.dt);
        let mut cx = Cx { t, dt, h };
        if let Some(c) = self.k15_79_get(obj) {
            return Some(c.p.ee_sub_13b2f0(hit, &mut cx));
        }
        self.k13_get(obj).map(|c| c.p.ee_sub_13b2f0(hit, &mut cx))
    }
    /// K15: launch projectile `proj` (79 / 80 / 82 here, 81 through `k14_fire`) from the class-112 launcher
    /// `launcher` (FUN_001d6b00 + slot 77)
    pub fn k15_fire(&mut self, launcher: usize, proj: usize, h: &mut dyn ClassHost) -> Option<launch::Launch> {
        if self.k13_get(proj).is_some() {
            return self.k14_fire(launcher, proj, h);
        }
        let l = self.k7.iter().find_map(|x| match &x.1 {
            Inst::K7(super::k7::K7::C2d7f40(c)) if x.0 == launcher => Some(c.clone()),
            _ => None,
        })?;
        let frame = if l.f_60 != 0 { h.ee_sub_15e938(0x2002, l.f_60, handle(launcher)) } else { 0 };
        let t = self.g.t;
        let k = self.k15.iter().position(|x| x.0 == proj && matches!(x.1, Inst::K15c(_)))?;
        let Inst::K15c(c) = &mut self.k15[k].1 else { return None };
        let out = launch::ee_sub_1d6b00(&launch::Launcher { l: &l, frame }, c.cls as i32, c.p.phys.pos, c.p.f_260, c.p.phys.gravity, &mut self.rng, h)?;
        c.p.ee_sub_13bc60(proj, handle(launcher), h);
        c.p.in_play = true;
        c.p.phys.v = out.v;
        if let Some(r) = out.rot {
            c.p.phys.rot = r;
        }
        if let Some(w) = out.w {
            c.p.phys.w = w;
        }
        if let Some((f, v)) = out.slot77 {
            if c.cls == 82 {
                c.p.ee_sub_1b2228(proj, f, v, t, &mut self.g.act_pool, h);
            } else {
                c.p.ee_sub_1b1e88(proj, f, v, &mut self.g.act_pool, h);
            }
        }
        Some(out)
    }
    /// K15 projectiles per frame (in the world or launched): slots 57 / 15, the physics part (slot 69), then
    /// the deferred slot-76 calls (FUN_00149c80 phase)
    pub(super) fn frame_k15_79(&mut self, h: &mut dyn ClassHost) {
        let (t, dt) = (self.g.t, self.g.dt);
        let mut deferred = Vec::new();
        for (i, x) in &mut self.k15 {
            if let Inst::K15c(c) = x {
                if !c.p.in_play && !h.in_world_28(handle(*i)) {
                    continue;
                }
                let mut cx = Cx { t, dt, h: &mut *h };
                if let Some(o) = frame_proj(&mut c.p, *i, &mut cx) {
                    deferred.push((*i, o));
                }
                c.physics(*i, t, dt, h);
            }
        }
        for (obj, other) in deferred {
            if let Some(c) = self.k15_79_get(obj) {
                c.slot76(obj, other, h);
            }
        }
    }
}
