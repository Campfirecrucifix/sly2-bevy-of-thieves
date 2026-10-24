//! WP-K20: class 77 C2d7ca8 (vtable 0x2d7ca8, parent class-66 SO 0x2d6460 through FUN_001c3d70 / 1c3df8 /
//! 1c3ff0): a breakable driven vehicle with wheels (+0x190, 0x60 records), treads (+0x4a4, 0x14 records), damage
//! parts (+0x4f8, 0xb0 C2c94a0 records), hit points (+0x55c max, +0x560 left) and a 4-state machine at +0x180:
//! 0 idle, 1 driven by the AI path (FUN_001d2610), 2 driven by the joystick at +0x150 (FUN_001d2150; slot 77
//! with k != 0 enters it), 3 destroyed (hit points 0: message 5).
//! 1:1: slot 6 FUN_001d1370, slot 77 FUN_001d1940, FUN_001d14c0 (load-post reset), FUN_001d4940 (SetState; its
//! anim / sound callees are hooks), FUN_001d4c88 (damage blend), FUN_001d4e08 (part broken),
//! FUN_001d2150 (state 2), slot 74 FUN_001d4568 (hit: hit points and the kick impulse),
//! the scalar head of slot 15 FUN_001d19a8 (heading +0x540, state dispatch, steer +0x548 / slip +0x558 / yaw
//! rate +0x554 polynomials).
//! HOOKS: slot 15's VU tail (tread / wheel bones, part sm labels, engine sound) = `po_call(0x1d19a8, obj, 0x4a4)`;
//! state 1 FUN_001d2610 = `po_call(0x1d2610)`; FUN_001d4b28 (path follow on / off), the sound
//! (FUN_001fbb90 / 1fb2f8 / 258040 / 1fb4f0 / 1fb018 / 1fb098), FUN_00213b00 / 212e40 (the +0x568 damage
//! blend), FUN_001f0948 / 1f0730 (the +0x570 sm) = `ee_call`.
//! UNPORTED: FUN_001d2dd0 (deepest contact), FUN_001d4d30 / 1d4de0 (part defaults / copy), slots 14 FUN_001d1518 (list resolve), 18 FUN_001d41f8 (wheel matrices), 25 FUN_001d1910, 40
//! FUN_001d1410 (part records), 48 FUN_001d4178, 53 FUN_001d3f30, 54 FUN_001d4050, 59 FUN_001d3e48 +
//! FUN_001d3450 (wheel contacts), 69 FUN_001d2e18 (drive forces) and the field-row lists 1419..1421 (the
//! +0x190 / +0x4a4 / +0x4f8 records stay empty, so the per-part loops run over nothing outside tests).
#![allow(non_snake_case)]
use super::k17_55::Mem;
use super::*;
use crate::engine::physics::driven::GameRng;
use crate::engine::player::{Joy, FUN_0011e1a0};

fn handle(obj: usize) -> u32 {
    obj as u32 + 1
}

/// slot 5 FUN_00113ee0 (class descriptor getter)
pub const SLOT5: u32 = 0x2bb0ec;
/// DAT_002c2208: +0x570 sm label per state
pub const L_2C2208: [u32; 4] = [0x73, 0x74, 0x75, 0x76];
/// DAT_002c2030: speed clamp of the yaw rate
pub const F_2C2030: f32 = 500.0;
/// DAT_002c2040..48 / 50..54: steer polynomial + clamp
pub const P_2C2040: [f32; 3] = [0.0, 0.8, 0.0];
pub const C_2C2050: [f32; 2] = [-0.5, 0.5];
/// DAT_002c2060..68 / 70..74: slip polynomial + clamp
pub const P_2C2060: [f32; 3] = [1.5, -0.5, 0.0];
pub const C_2C2070: [f32; 2] = [0.3, 1.0];
/// DAT_002c2080..88 / 90..94: yaw-rate polynomial + clamp
pub const P_2C2080: [f32; 3] = [-0.5, 1.0, 0.0];
pub const C_2C2090: [f32; 2] = [1.0, 2.5];
/// DAT_002c21d0..21e4: kick 0x100 (horizontal, up, spin) ranges
pub const K_2C21D0: [[f32; 2]; 3] = [[50.0, 100.0], [300.0, 400.0], [2.0, 5.0]];
/// DAT_002c21e8..2204: kick 0x1000 (away, up, spin, spin z) ranges
pub const K_2C21E8: [[f32; 2]; 4] = [[50.0, 100.0], [50.0, 75.0], [2.5, 3.0], [-0.5, 0.5]];
const PI: f32 = std::f32::consts::PI;

/// a +0x4f8 damage part (C2c94a0 record)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Part {
    /// +0x1c part object, +0x20 its sm
    pub obj: u32,
    pub sm: u32,
    /// +0x24 joystick held mask (break when held), +0x28 press mask
    pub mask: u32,
    pub button: u32,
    /// +0x6c damage stage index, +0x70 stage threshold
    pub stage: i32,
    pub thresh: i32,
}

#[derive(Clone, Debug, Default)]
pub struct C2d7ca8 {
    pub m: Mem,
    /// +0x150 the driving joystick (state 2)
    pub joy: Joy,
    /// +0x4f8 / +0x4fc parts
    pub parts: Vec<Part>,
    /// K21: +0x190 wheels, +0x4a4 treads, the rest of the part records (classes/k21_77.rs)
    pub wheels: Vec<super::k21_77::Wheel>,
    pub treads: Vec<super::k21_77::Tread>,
    pub part_recs: Vec<super::k21_77::PartRec>,
    /// frame time (DAT_002dded4) for the slot-15 tail
    pub dt: f32,
}

/// FUN_0011cf60: wrap to (-pi, pi]
fn wrap(a: f32) -> f32 {
    crate::engine::player::p5_attach::FUN_0011cf60(a)
}

fn poly(p: [f32; 3], x: f32) -> f32 {
    p[0] + x * (p[1] + x * p[2])
}

impl C2d7ca8 {
    pub fn state(&self) -> i32 {
        self.m.i(0x180)
    }

    /// slot 6 FUN_001d1370 (after the SO init FUN_001c3d70 = `h.base`)
    pub fn ee_sub_1d1370(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1c3d70, obj);
        let m = &mut self.m;
        m.set_f(0x51c, 500.0);
        m.set_i(0x180, -1);
        m.set_f(0x520, 1500.0);
        m.set_f(0x524, -1000.0);
        m.set_f(0x528, 0.0);
        m.set_i(0x59c, -1);
        m.set_f(0x534, 1500.0);
        m.set_f(0x538, 2500.0);
        m.set_f(0x53c, 1.0);
        m.set_i(0x55c, 100);
        m.set_i(0x578, 0x1e);
        m.set_f(0x530, 0.0);
        m.set_i(0x590, 0x1e);
        m.set_i(0x584, 0x1e);
    }

    /// FUN_001d4c88: the damage blend FUN_00212e40(hp / max (inverted by +0x564), +0x568, 0x34f)
    pub fn ee_sub_1d4c88(&self, h: &mut dyn ClassHost) {
        let b = self.m.u(0x568);
        if b != 0 {
            let mut f = self.m.i(0x560) as f32 / self.m.i(0x55c) as f32;
            if self.m.u(0x564) != 0 {
                f = 1.0 - f;
            }
            h.ee_call(0x212e40, &[f.to_bits(), b, 0x34f]);
        }
    }

    /// FUN_001d4e08(part): the part is broken (+0x70 <= owner +0x574 + stage * 0xc)
    pub fn ee_sub_1d4e08(&self, p: &Part) -> bool {
        p.thresh <= self.m.i(0x574 + p.stage as u32 * 0xc)
    }

    /// FUN_001d14c0: hit points full, damage blend, the +0x570 sm to its first label, SetState 0
    pub fn ee_sub_1d14c0(&mut self, obj: usize, t: f32, h: &mut dyn ClassHost) {
        self.m.set_u(0x560, self.m.u(0x55c));
        self.ee_sub_1d4c88(h);
        let sm = self.m.u(0x570);
        if sm != 0 {
            h.ee_call(0x1f0948, &[sm, 0]);
        }
        self.ee_sub_1d4940(obj, 0, t, h);
    }

    /// FUN_001d4940(self, s): SetState
    pub fn ee_sub_1d4940(&mut self, obj: usize, s: i32, t: f32, h: &mut dyn ClassHost) {
        let o = handle(obj);
        let old = self.m.i(0x180);
        if s == old {
            return;
        }
        match old {
            0 => {
                let b = self.m.u(0x568);
                if b != 0 {
                    h.vcall_arg(b, 0x18c, -1);
                    h.ee_call(0x213b00, &[b, 0x34f]);
                    self.ee_sub_1d4c88(h);
                }
                if self.m.i(0x59c) != -1 {
                    // FUN_001fbb90 / 1fb2f8 / 258040 / 1fb4f0(250, 1500) / 1fb018: start the engine sound at +0x598
                    let snd = h.ee_call(0x1fbb90, &[self.m.u(0x59c)]) as u32;
                    h.ee_call(0x1fb2f8, &[snd, o]);
                    self.m.set_u(0x598, snd);
                    h.ee_call(0x1fb4f0, &[250f32.to_bits(), 1500f32.to_bits(), snd]);
                    h.ee_call(0x1fb018, &[snd]);
                }
            }
            1 => {
                h.ee_call(0x1d4b28, &[o, u32::MAX]);
            }
            _ => {}
        }
        self.m.set_i(0x180, s);
        self.m.set_f(0x184, t);
        match s {
            1 => {
                h.ee_call(0x1d4b28, &[o, 0]);
            }
            0 => {
                let b = self.m.u(0x568);
                if b != 0 {
                    h.vcall(b, 0x194);
                }
                let snd = self.m.u(0x598);
                if snd != 0 {
                    h.ee_call(0x1fb098, &[snd, 0]);
                    self.m.set_u(0x598, 0);
                }
            }
            3 => {
                h.vcall_arg(o, 0x1c, 5);
            }
            _ => {}
        }
        let sm = self.m.u(0x570);
        let cur = self.m.i(0x180);
        if sm != 0 && cur != -1 {
            h.ee_call(0x1f0730, &[sm, L_2C2208[cur as usize & 3]]);
        }
    }

    /// slot 77 FUN_001d1940(self, k): base FUN_001c3ff0; k != 0 -> state 2 (driven); k == 0 leaves state 2
    pub fn ee_sub_1d1940(&mut self, obj: usize, k: i32, t: f32, h: &mut dyn ClassHost) {
        h.base(0x1c3ff0, obj);
        if k == 0 {
            if self.m.i(0x180) == 2 {
                self.ee_sub_1d4940(obj, 0, t, h);
            }
        } else {
            self.ee_sub_1d4940(obj, 2, t, h);
        }
    }

    /// FUN_001d2150: state 2 (joystick): heading target +0x544 = stick angle; 0x40 held -> speed +0x51c
    /// along +0x540, else stop; each unbroken part: held & +0x24 -> label 0x67, press +0x28 -> 0x68
    pub fn ee_sub_1d2150(&mut self, t: f32, h: &mut dyn ClassHost) {
        let a = self.joy.angle;
        self.m.set_f(0x544, a);
        if self.joy.held & 0x40 == 0 {
            self.m.set_f(0x54c, a);
            self.m.set_f(0x550, 0.0);
        } else {
            self.m.set_f(0x550, self.m.f(0x51c));
            self.m.set_f(0x54c, self.m.f(0x540));
        }
        for k in 0..self.parts.len() {
            let p = self.parts[k].clone();
            if !self.ee_sub_1d4e08(&p) {
                continue;
            }
            if self.joy.held & p.mask != 0 {
                h.ee_call(0x1f0660, &[p.sm, 0x67]);
            } else if self.joy.FUN_001c46c0(t, 0.0, p.button) != 0 {
                h.ee_call(0x1f0660, &[p.sm, 0x68]);
            }
        }
    }

    /// slot 15 FUN_001d19a8 (after the SO update FUN_001f36e0 = `h.base`). `heading` = atan2 of the +0x60
    /// xform's +0x44 / +0x40; `fc28` = (vt+0xfc)(self) +0x28 (> 0.5 stamps +0x5c0).
    pub fn ee_sub_1d19a8(&mut self, obj: usize, heading: f32, fc28: f32, t: f32, h: &mut dyn ClassHost) {
        h.base(0x1f36e0, obj);
        self.m.set_f(0x540, heading);
        for k in 0..self.parts.len() {
            let p = self.parts[k].clone();
            if !self.ee_sub_1d4e08(&p) {
                h.ee_call(0x1f0660, &[p.sm, 0x65]);
            }
        }
        match self.m.i(0x180) {
            1 => {
                h.po_call(0x1d2610, obj, 0, &[]);
            }
            2 => self.ee_sub_1d2150(t, h),
            0 | 3 => {
                self.m.set_f(0x550, 0.0);
                self.m.set_f(0x544, self.m.f(0x540));
                self.m.set_f(0x554, 0.0);
            }
            _ => {}
        }
        let d = wrap(self.m.f(0x544) - self.m.f(0x540));
        self.m.set_f(0x548, poly(P_2C2040, d).clamp(C_2C2050[0], C_2C2050[1]));
        let d = wrap(self.m.f(0x544) - self.m.f(0x540)).abs();
        self.m.set_f(0x558, poly(P_2C2060, d).clamp(C_2C2070[0], C_2C2070[1]));
        let d = wrap(self.m.f(0x544) - self.m.f(0x540)).abs();
        let r = poly(P_2C2080, d).clamp(C_2C2090[0], C_2C2090[1]);
        let v = self.m.f(0x550).min(F_2C2030);
        self.m.set_f(0x554, r * v * self.m.f(0x548) / self.m.f(0x4a0));
        if 0.5 < fc28 {
            self.m.set_f(0x5c0, t);
        }
        // K21: the VU tail (classes/k21_77.rs); +0x5a4 target position / velocity
        let rot = h.obj_mat(handle(obj), 0x60);
        let rot = [[rot[0][0], rot[0][1], rot[0][2]], [rot[1][0], rot[1][1], rot[1][2]], [rot[2][0], rot[2][1], rot[2][2]]];
        let me = h.ee_sub_1a62e8(handle(obj));
        let tg = self.m.u(0x5a4);
        let (tp, tv) = if tg != 0 { (h.ee_sub_1a62e8(tg), h.vcall_10c(tg as usize - 1)) } else { (me, [0.0; 3]) };
        let dt = self.dt;
        self.ee_sub_1d19a8_tail(obj, &rot, me, tp, tv, dt, h);
    }

    /// slot 74 FUN_001d4568(self, hit {src, .., kind +0xc, damage +0x10}): `base_ok` = FUN_001f7588. Hit points
    /// += damage (>= 0), 0 -> state 3; kind 0x100 / 0x1000 kicks velocity and spin. 1 = taken.
    pub fn ee_sub_1d4568(&mut self, obj: usize, base_ok: bool, src: u32, kind: u32, dmg: i32, t: f32, rng: &mut GameRng, h: &mut dyn ClassHost) -> bool {
        if !base_ok {
            return false;
        }
        let o = handle(obj);
        self.m.set_i(0x560, (self.m.i(0x560) + dmg).max(0));
        self.ee_sub_1d4c88(h);
        if self.m.i(0x560) == 0 {
            self.ee_sub_1d4940(obj, 3, t, h);
        }
        let (v, w) = match kind {
            0x100 => {
                let a = rng.FUN_0011d448(-PI, PI);
                let r = rng.FUN_0011d448(K_2C21D0[0][0], K_2C21D0[0][1]);
                let z = rng.FUN_0011d448(K_2C21D0[1][0], K_2C21D0[1][1]);
                let v = FUN_0011e1a0(a, r, z);
                let s = rng.FUN_0011d448(K_2C21D0[2][0], K_2C21D0[2][1]);
                let c = [-v[1], v[0], 0.0];
                let n = (c[0] * c[0] + c[1] * c[1]).sqrt();
                let d = if n < 0.0001 { [1.0, 0.0, 0.0] } else { [c[0] / n, c[1] / n, 0.0] };
                (v, d.map(|x| x * s))
            }
            0x1000 => {
                let p = h.ee_sub_1a62e8(o);
                let q = h.ee_sub_1a62e8(src);
                let d = [p[0] - q[0], p[1] - q[1], 0.0];
                let n = (d[0] * d[0] + d[1] * d[1]).sqrt();
                let d = if n < 0.0001 { [1.0, 0.0, 0.0] } else { [d[0] / n, d[1] / n, 0.0] };
                let r0 = rng.FUN_0011d448(K_2C21E8[0][0], K_2C21E8[0][1]);
                let r1 = rng.FUN_0011d448(K_2C21E8[1][0], K_2C21E8[1][1]);
                let v = [d[0] * r0, d[1] * r0, r1];
                let r2 = rng.FUN_0011d448(K_2C21E8[2][0], K_2C21E8[2][1]);
                let r3 = rng.FUN_0011d448(K_2C21E8[3][0], K_2C21E8[3][1]);
                (v, [-d[1] * r2, d[0] * r2, r3])
            }
            _ => ([0.0; 3], [0.0; 3]),
        };
        let v0 = h.vcall_10c(obj);
        h.vcall_11c_v(o, [v0[0] + v[0], v0[1] + v[1], v0[2] + v[2]]);
        let w0 = h.vcall_114(obj);
        h.vcall_124_v(o, [w0[0] + w[0], w0[1] + w[1], w0[2] + w[2]]);
        true
    }
}

/// K20 class 77 in the ClassSet
impl ClassSet {
    pub(super) fn build_k20_77(&mut self, w: &crate::engine::world::World, h: &mut dyn ClassHost) {
        for (i, o) in w.objects.iter().enumerate() {
            if o.class != 77 {
                continue;
            }
            let mut c = C2d7ca8::default();
            c.ee_sub_1d1370(i, h);
            // field row 1434 -> +0x59c (engine sound id)
            if let Some(p) = w.objects.get(o.clone_of.unwrap_or(i)).and_then(crate::engine::game::fields::object_props) {
                for x in p {
                    if let (1434, crate::engine::world::props::PropValue::Scalar(b)) = (x.field, &x.value) {
                        let mut a = [0u8; 4];
                        a[..b.len().min(4)].copy_from_slice(&b[..b.len().min(4)]);
                        c.m.set_u(0x59c, u32::from_le_bytes(a) & if b.len() == 2 { 0xffff } else { u32::MAX });
                    }
                }
            }
            self.k20.push((i, Inst::K20Truck(Box::new(c))));
        }
    }
    pub fn k20_77(&mut self, obj: usize) -> Option<&mut C2d7ca8> {
        self.k20.iter_mut().find(|x| x.0 == obj).and_then(|x| if let Inst::K20Truck(c) = &mut x.1 { Some(&mut **c) } else { None })
    }
    /// load-post: slot 14 FUN_001d1518 (K21, classes/k21_77.rs; ends in FUN_001d14c0)
    pub(super) fn post_load_k20_77(&mut self, h: &mut dyn ClassHost) {
        self.k21_77_slot14(h);
    }
    /// per frame: slot 15
    pub(super) fn frame_k20_77(&mut self, h: &mut dyn ClassHost) {
        let t = self.g.t;
        let dt = self.g.dt;
        for (i, x) in &mut self.k20 {
            // FEEL R33: slot 15 runs from the world update list only (LO +0x28 bit 60)
            if h.lo_removed(*i) {
                continue;
            }
            if let Inst::K20Truck(c) = x {
                c.dt = dt;
                // K21 slot 18: the forward speed = the velocity (vtable +0x10c) in the object frame
                let m = h.obj_mat(handle(*i), 0x60);
                let v = h.vcall_10c(*i);
                c.ee_sub_1d41f8(*i, m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2], dt, h);
                // REPLACED: heading = atan2 of the world matrix x axis (the +0x60 xform +0x44 / +0x40)
                let m = h.obj_mat(handle(*i), 0x60);
                c.ee_sub_1d19a8(*i, m[0][1].atan2(m[0][0]), 0.0, t, h);
            }
        }
    }
}
