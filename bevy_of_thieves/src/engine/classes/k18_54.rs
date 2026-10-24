//! WP-K18: class 54 C2d44c8 = Murray (parent 52, size 0x1b00, vtable 0x2d44c8; tools/engineport/class_port.md
//! "Murray closure"). The class-52 base layout below +0x1320 is the Bentley one: the instance keeps a
//! `k16_55::C2cb230` as its base (`b`, words widened to 0x1b00) and reuses the class-52 base slots ported in
//! K16/K17/K18 as is (`b.state` is +0x13d0).
//! 1:1 own slots: 5 FUN_00113d70, 6 FUN_001b4728 (init, FUN_0013ff70 reused), 49 FUN_001b6728, 54 FUN_001b9130,
//! 60 FUN_001b8450, 69 FUN_001b67f8, 75 FUN_001b83b8, 84 FUN_001b8368, 87 FUN_001b7e40 (jump table 0x2a73f0),
//! 88 FUN_001b7f90, 89 FUN_001b8008, 90 FUN_001b80f0, 91 FUN_001b80f8, 93 FUN_001b8180, 95 FUN_001b9b70,
//! 97 FUN_001b8100, 102 FUN_001b9b60, 104 FUN_00141e70 (empty base); FUN_0020bd78.
//! K19 (classes/k19_54.rs) ported SetState FUN_001b6078, FUN_001b7ba8, FUN_001b6cc0 and slots 4 / 14 / 15 /
//! 98 / 99; `ee_sub_1b7e40` below keeps the K18 hook form (the 1:1 one is `ee_sub_1b7e40_cx`).
//! K25: slot 54 1:1 with FUN_0011f198 / FUN_001aa020 / FUN_001a9fe0 (the +0x13e4 list = `saved_13e4`).
//! Hooks: slot 86 FUN_001b81b8 (vtable calls on the held object), and the shared `ClassHost::po_call` JT-layout component calls.
#![allow(non_snake_case)]
use super::k16_55::{ee_sub_194640, C2cb230, Lead};
use super::k17_55::CamRec;
use super::*;
use crate::engine::player::p5_hull::ContactRec;

fn handle(obj: usize) -> u32 {
    obj as u32 + 1
}

/// class-54 object size
pub const SIZE: usize = 0x1b00;
/// slot 102 FUN_001b9b60
pub const NAME: &str = "murray";
/// DAT_002c0a38: slot-54 state-0xc velocity . normal threshold
pub const F_2C0A38: f32 = -0.3;
/// DAT_002c3c60: side sign
pub const SIGN_2C3C60: [f32; 2] = [1.0, -1.0];
/// slot 87 jump table 0x2a73f0: index -> state passed to FUN_001b6078 (index 0: 5 with +0x13f0 held, else 0)
pub const STATE_OF_INDEX: [i32; 9] = [0, 2, 0xd, 0xe, 0x10, 0x11, 0x12, 0x13, 0x14];

/// Murray instance state
#[derive(Clone, Debug)]
pub struct C2d44c8 {
    /// class-52 base layout (+0x13d0 = `b.state`)
    pub b: C2cb230,
    /// K25: the +0x13e4 contact-record list (head first; see `ee_sub_1b9130`)
    pub saved_13e4: Vec<ContactRec>,
    /// FEEL R69: SetVelocity (vt +0x11c) of the anim event 0x15e (FUN_00140f90) outside the PO host; the player
    /// layer applies it to the controlled body (`player::plugin`)
    pub set_vel_11c: Option<[f32; 3]>,
}

impl Default for C2d44c8 {
    fn default() -> Self {
        let mut b = C2cb230::default();
        b.m.0 = vec![0; SIZE / 4];
        C2d44c8 { b, saved_13e4: Vec::new(), set_vel_11c: None }
    }
}

/// K25: +0x13e4 value of a non-empty saved contact list
pub const REC_TAG: u32 = 0x4600_0000;

/// FUN_0011f198(a, b): cosine of the angle between `a` and `b` (0 when either is shorter than 0.0001)
pub fn ee_sub_11f198(a: [f32; 3], b: [f32; 3]) -> f32 {
    let la = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
    if la < 0.0001 {
        return 0.0;
    }
    let lb = (b[0] * b[0] + b[1] * b[1] + b[2] * b[2]).sqrt();
    if lb < 0.0001 {
        return 0.0;
    }
    (a[0] * b[0] + a[1] * b[1] + a[2] * b[2]) / (la * lb)
}

/// FUN_001aa020(0x3dd490, rec): copy of the record chain (one record in the port)
pub fn ee_sub_1aa020(rec: &ContactRec) -> Vec<ContactRec> {
    vec![*rec]
}

/// FUN_001a9fe0(0x3dd490, chain, list): `chain` (tail +0xac -> old head) becomes the head of `list`
pub fn ee_sub_1a9fe0(list: &mut Vec<ContactRec>, chain: Vec<ContactRec>) {
    if !chain.is_empty() {
        list.splice(0..0, chain);
    }
}

impl C2d44c8 {
    /// slot 5 FUN_00113d70: class descriptor 0x2baf20
    pub fn ee_sub_113d70(&self) -> u32 {
        0x2baf20
    }
    /// slot 102 FUN_001b9b60
    pub fn ee_sub_1b9b60(&self) -> &'static str {
        NAME
    }
    /// slot 104 FUN_00141e70 (base, empty)
    pub fn ee_sub_141e70(&self) {}
    /// slot 6 FUN_001b4728(self): Murray init (FUN_0013ff70 base; DAT_002f7900 = self is `ClassGlobals` side
    /// state the caller records: returns the handle)
    pub fn ee_sub_1b4728(&mut self, obj: usize, owner_bit: Option<bool>, g: &ClassGlobals, h: &mut dyn ClassHost) -> u32 {
        let b = &mut self.b;
        b.ee_sub_13ff70(obj, owner_bit, g.g_2c3b90, h);
        let o = handle(obj);
        b.m.set_f(0x220, 50.0);
        b.m.set_i(0x17f4, -1);
        b.m.set_f(0x1410, 1500.0);
        b.m.set_f(0x1414, 3500.0);
        b.m.set_f(0x103c, 60.0);
        b.f_1048 = 125.0;
        b.m.set_i(0x13d8, -1);
        b.m.set_i(0x1810, -1);
        b.m.set_i(0x1840, -1);
        b.state = -1;
        b.m.set_i(0x144c, -1);
        b.m.set_i(0x1450, -1);
        b.m.set_i(0x1458, -1);
        for p in [0x1320, 0x1460, 0x1630, 0x1540] {
            b.ee_sub_20b840(p, o, 0x15);
        }
        // FUN_0020bd78(self + 0x1850)
        b.m.set_i(0x1850, -1);
        b.m.set_f(0x1434, -60.0);
        b.m.set_f(0x1098, -100.0);
        b.m.set_f(0x109c, 100.0);
        b.m.set_f(0x10a0, 50.0);
        b.m.set_f(0x10a4, -10.0);
        b.m.set_f(0x10a8, 250.0);
        b.m.set_f(0x1ac8, -60.0);
        b.m.set_f(0x10b4, 4500.0);
        b.m.set_f(0x10b0, 1.0);
        b.m.set_f(0x10ac, 0.0);
        b.ee_sub_21f2e0_at(0x1910, o);
        b.m.set_i(0x1000, 4);
        o
    }
    /// slot 69 FUN_001b67f8: physics forces of the current state
    pub fn ee_sub_1b67f8(&self, obj: usize, h: &mut dyn ClassHost) {
        let s = self.b.state;
        if s == 0x14 {
            h.ee_sub_1f9b20(obj);
        } else {
            h.ee_sub_204108(obj);
        }
        match s {
            2 => h.po_part(0x196df8, obj, 0x2c0),
            // K19: FUN_001b6cc0 in classes/k19_54.rs
            3 => self.ee_sub_1b6cc0(obj, h),
            0xd => h.po_part(0x194170, obj, 0x4c0),
            0xe => h.po_part(0x1a1148, obj, 0xdf0),
            0x14 => h.po_part(0x19df70, obj, 0xf50),
            _ => {}
        }
    }
    /// slot 75 FUN_001b83b8(self, kind): hit of `kind` vetoed
    pub fn ee_sub_1b83b8(&self, obj: usize, kind: i32, h: &mut dyn ClassHost) -> bool {
        match self.b.state {
            0xf => return true,
            0xd if ee_sub_194640(self.b.att_4c8, self.b.att_4cc) => return true,
            0x14 if kind == 0x4000 => return true,
            _ => {}
        }
        h.ee_sub_140ed8(obj, kind)
    }
    /// slot 84 FUN_001b8368
    pub fn ee_sub_1b8368(&self) -> i32 {
        if self.b.f_204 != 0 || self.b.m.i(0x13d8) == 1 || self.b.state == 0x14 {
            return 0;
        }
        self.b.ee_sub_143c78()
    }
    /// slot 87 FUN_001b7e40(self, i): SetState FUN_001b6078 (hook) of index `i`; FUN_001b7ba8 (hook) first for
    /// the indices 3 / 5..8, and for 1 unless +0x2c4 is 0xf / 0x10; i >= 9 does nothing
    pub fn ee_sub_1b7e40(&self, obj: usize, i: u32, h: &mut dyn ClassHost) {
        let Some(&s) = STATE_OF_INDEX.get(i as usize) else {
            return;
        };
        let s = if i == 0 && self.b.m.u(0x13f0) != 0 { 5 } else { s };
        let drop = match i {
            1 => !(0xf..0x11).contains(&self.b.m.i(0x2c4)),
            3 | 5 | 6 | 7 | 8 => true,
            _ => false,
        };
        if drop {
            h.po_call(0x1b7ba8, obj, 0, &[]);
        }
        h.po_call(0x1b6078, obj, 0, &[s as u32]);
    }
    /// slot 88 FUN_001b7f90: HUD index of the state
    pub fn ee_sub_1b7f90(&self) -> i32 {
        match self.b.state {
            0 | 5 => 0,
            2 => 1,
            0xd => 2,
            0xe => 3,
            0x10 => 4,
            0x11 => 5,
            0x12 => 6,
            0x13 => 7,
            0x14 => 8,
            _ => -1,
        }
    }
    /// slot 89 FUN_001b8008: base ASEG by state, default per-state table +0x1a58 / +0x268
    pub fn ee_sub_1b8008(&mut self, obj: usize, h: &mut dyn ClassHost) {
        let b = &mut self.b;
        let (mut a, f) = match b.state {
            5 => (b.m.u((b.m.i(0x1458) * 4 + 0x1ab0) as u32), 0),
            2 => {
                if b.m.u(0x13f0) == 0 {
                    (b.ee_sub_19c870(), 0)
                } else {
                    (b.m.u(0x1abc), 0)
                }
            }
            0xd => b.ee_sub_1944c8(),
            0xe => (b.dmg.FUN_001a23f8().map_or(0, |x| x as u32), 0),
            s => (b.m.u((s * 4 + 0x1a58) as u32), 0),
        };
        if a == 0 {
            a = b.m.u(0x268);
        }
        b.ee_sub_203d00(obj, a, f, h);
    }
    /// slot 90 FUN_001b80f0
    pub fn ee_sub_1b80f0(&self) {}
    /// slot 91 FUN_001b80f8
    pub fn ee_sub_1b80f8(&self) -> i32 {
        0
    }
    /// slot 93 FUN_001b8180: states 0, 1, 9, 10
    pub fn ee_sub_1b8180(&self) -> bool {
        let s = self.b.state;
        (0..2).contains(&s) || (9..0xb).contains(&s)
    }
    /// slot 95 FUN_001b9b70(self, out, t): base lead record, kind 1 in states 0 / 1 / 9 / 10 / 0xb (0xb: +0x10
    /// = +0x1830.., +0x20 = DAT_002a2dd0 zero), +4 = 200
    pub fn ee_sub_1b9b70(&self, obj: usize, t: f32, h: &mut dyn ClassHost) -> Lead {
        let mut l = self.b.ee_sub_143220(obj, t, h);
        match self.b.state {
            0 | 1 | 9 | 10 => l.kind = 1,
            0xb => {
                l.kind = 1;
                l.pos = self.b.m.v3(0x1830);
                l.vel = [0.0; 3];
            }
            _ => {}
        }
        l.r_4 = 200.0;
        l
    }
    /// slot 97 FUN_001b8100
    pub fn ee_sub_1b8100(&self, h: &mut dyn ClassHost) -> i32 {
        // FUN_001409b8
        let v = if 0 < h.world_3d78() { 0 } else { -1 };
        if v != -1 {
            return v;
        }
        match self.b.state {
            9 => {
                if self.b.m.i(0x17f4) == 0 {
                    -1
                } else {
                    1
                }
            }
            1 | 0xc => 1,
            _ => -1,
        }
    }
    /// slot 49 FUN_001b6728(self, rec): camera target
    pub fn ee_sub_1b6728(&mut self, obj: usize, rec: &mut CamRec, g: &ClassGlobals, h: &mut dyn ClassHost) {
        let b = &mut self.b;
        if b.state != 0 && b.state != 0x10 && b.att_4cc != 0 {
            b.m.set_f(0x1080, g.t);
        }
        b.ee_sub_141640(obj, rec, g, h);
        match b.state {
            0xb if b.m.i(0x10e4) != 4 => {}
            0xb | 3 | 9 | 0xc => rec[0] |= 1,
            // FUN_00220388(self + 0x1910, rec)
            0xf => h.po_part_rec(0x220388, obj, 0x1910, rec),
            0x14 => h.po_part_rec(0x19ea00, obj, 0xf50, rec),
            _ => {}
        }
    }
    /// slot 54 FUN_001b9130(self, rec, side, other): contact record part (`other` = the object of side 1 - side)
    pub fn ee_sub_1b9130(&mut self, obj: usize, rec: &mut ContactRec, side: usize, other: u32, h: &mut dyn ClassHost) {
        h.ee_sub_204250(obj, rec, side);
        match self.b.state {
            0xc => {
                // (other vtable +500)() pushable, normal z < 0.7, velocity . normal < -0.3 -> +0x1908 = 1
                if self.b.m.i(0x1840) == 1 && h.vcall_ret(other, 500) != 0 && rec.n[2] < 0.7 {
                    // K25: FUN_0011f198 (cosine of the velocity / normal angle), not the raw dot product
                    let v = h.vcall_10c(obj);
                    if ee_sub_11f198(v, rec.n) < F_2C0A38 {
                        self.b.m.set_i(0x1908, 1);
                    }
                }
            }
            3 => {
                if self.b.m.i(0x13d8) == 1 && 0.7 < SIGN_2C3C60[side.min(1)] * rec.n[2] {
                    // K25: FUN_001aa020(0x3dd490, rec) (copy of the record chain from the 0x3dd490 +0x3d40 pool) +
                    // FUN_001a9fe0(0x3dd490, copy, self + 0x13e4) (prepended to the +0x13e4 list). REPLACED: the
                    // port's contact record is a single value (no +0xac chain): one copy, kept in `saved_13e4`;
                    // +0x13e4 holds `REC_TAG | count` (non-zero = the state-3 landing record of FUN_001b6ba8)
                    ee_sub_1a9fe0(&mut self.saved_13e4, ee_sub_1aa020(rec));
                    self.b.m.set_u(0x13e4, REC_TAG | self.saved_13e4.len() as u32);
                }
            }
            0xe => {
                if let Some([a, b]) = self.b.dmg.FUN_001a2100() {
                    rec.f94 = a;
                    rec.f98 = b;
                }
            }
            0x14 => {
                rec.f98 = 0.0;
                rec.f94 = 0.0;
            }
            _ => {}
        }
    }
    /// slot 60 FUN_001b8450(self, other): base contact filter; +0x1430 grabs -> max(r, 3); state 0xf the
    /// +0x1910 FUN_00220498
    pub fn ee_sub_1b8450(&self, obj: usize, other: u32, g: &ClassGlobals, h: &mut dyn ClassHost) -> i64 {
        let r = self.b.ee_sub_144388(obj, other, g, h);
        if self.b.m.i(0x1430) != 0 && h.po_call(0x1a6af8, obj, 0, &[other]) != 0 {
            return r.max(3);
        }
        if self.b.state == 0xf {
            let k = self.b.m.i(0x1914);
            if other == self.b.m.u(0x1920) && k != -1 && (k > 2 || (other == g.g_2dd5bc && h.obj_i32(other, 0x2dc) == 0x12)) {
                return r.max(3);
            }
        }
        r
    }
    /// slot 86 FUN_001b81b8(self, k): ported 1:1 in K20 (classes/k20_54.rs `ee_sub_1b81b8_k20`)
    pub fn ee_sub_1b81b8(&mut self, obj: usize, k: i32, h: &mut dyn ClassHost) {
        self.ee_sub_1b81b8_k20(obj, k, h);
    }
}

impl ClassSet {
    /// K18: construct + init (slot 6) the class-54 (Murray) instances
    pub(super) fn build_k18_54(&mut self, w: &World, h: &mut dyn ClassHost) {
        for (i, o) in w.objects.iter().enumerate() {
            if o.class == 54 {
                let mut c: Box<C2d44c8> = Box::default();
                c.b.alo.so.owner = o.owner;
                let owner_bit = o.owner.map(|ow| self.bit35_of(ow).unwrap_or_else(|| h.owner_bit35(ow)));
                c.ee_sub_1b4728(i, owner_bit, &self.g, h);
                self.k18.push((i, Inst::K18Murray(c)));
            }
        }
    }
    /// K18 class-54 state at object index `obj`
    pub fn k18_murray(&mut self, obj: usize) -> Option<&mut C2d44c8> {
        self.k18.iter_mut().find(|x| x.0 == obj).and_then(|x| if let Inst::K18Murray(c) = &mut x.1 { Some(&mut **c) } else { None })
    }
    /// K18 / K19: slot 14 FUN_001b49e0 (classes/k19_54.rs) of every Murray
    pub(super) fn post_k18_54(&mut self, h: &mut dyn ClassHost) {
        let objs: Vec<usize> = self.k18.iter().map(|x| x.0).collect();
        for o in objs {
            self.k19_murray_with(o, h, |c, cx| c.ee_sub_1b49e0(o, cx));
        }
    }
    /// K18: per frame, the controlled Murray: slot 15 (K19: FUN_001b50b0 in classes/k19_54.rs), slot 69
    /// forces, slot 89 ASEG, slot 18 draw. No-op until `k18_set_active` picks a Murray.
    pub(super) fn frame_k18_54(&mut self, h: &mut dyn ClassHost) {
        // ANIM-ALL R43: an uncontrolled Murray's binocucom part of FUN_001b50b0 (FUN_001b5038 -> FUN_0019d330 after
        // the +0x28 bit-60 gate; no input) still runs, as the uncontrolled Bentley's (k18_55 team loop), so a
        // sequence action 0x3b mode-1 open reaches state 1 -> slot 82 / 83 (0x3f) on the partner
        let team: Vec<usize> = self.k18.iter().map(|x| x.0).filter(|&m| Some(m) != self.k18_active).collect();
        for m in team {
            if self.k18_murray(m).is_some_and(|c| c.b.binoc.active()) {
                self.k19_murray_with(m, h, |c, cx| {
                    if cx.h.lo_28_bit60(handle(m)) {
                        c.b.binoc_19d330(m, cx);
                    }
                });
            }
        }
        let Some(o) = self.k18_active else {
            return;
        };
        if self.k18_murray(o).is_some() {
            self.k10_mirror_180(h);
        }
        self.refresh_2dd5bc_13d0();
        if self.k19_murray_with(o, h, |c, cx| c.ee_sub_1b50b0(o, cx)).is_none() {
            return;
        }
        // TAKEN BY HEIST: FUN_00136530 calls of the frame (FUN_002205a8 throw release) on the Bentley
        for (b, s) in h.take_136530() {
            let g = self.g.clone();
            if let Some(c) = self.k16_get(b) {
                c.ee_sub_136530(b, s, &g, h);
            }
        }
        let g = self.g.clone();
        let Some(c) = self.k18_murray(o) else {
            return;
        };
        c.ee_sub_1b67f8(o, h);
        // K27: the body's physics frame after the slot 69 forces (classes/k27.rs)
        h.k27_po_step(o, super::k27::motor_of(|x| c.b.m.f(x)), g.dt);
        c.ee_sub_1b8008(o, h);
        c.b.ee_sub_1425b0(o, &g, h);
    }
    /// MURRAY: class-36 point +0x180 (field 1085, an object ref: the thing Murray lifts there) as the LO word
    /// FUN_001b93e8 / held_180 read (obj_i32(point, 0x180)). REPLACED: the K10 struct keeps the field as the
    /// object's oid; on PS2 it is the object pointer -> the first object of that oid (FUN_0015e938(4, oid)).
    fn k10_mirror_180(&mut self, h: &mut dyn ClassHost) {
        use super::k10::K10;
        let pts: Vec<(usize, i32)> = self.k10.iter().filter_map(|(i, k)| match k {
            Inst::K10(k) => match &**k { K10::C2d1d00(d) if d.f_180 > 0 => Some((*i, d.f_180)), _ => None },
            _ => None,
        }).collect();
        for (i, oid) in pts {
            let x = h.ee_sub_15e938(4, oid, handle(i));
            h.obj_set_u32(handle(i), 0x180, x);
        }
    }
}
