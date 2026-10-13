//! WP-K24: guard goal component 0xb0 (class C2d0420, parent C2cc440 goal base): the "lifted" goal brain
//! substate 6 starts (FUN_00176750 case 6, message sent by FUN_00209e28 - the class-102 bomb that grabs a
//! guard, classes/k23_102.rs). Slots 7 FUN_0017a730 (activate), 8 FUN_0017a7b8 (deactivate), 9 FUN_0017a7f8
//! (update), setstate FUN_0017a8c8, state test FUN_0017a9c0. The component lives in the guard's +0x26c list
//! (FUN_00205438 push + slot 6, FUN_00205458 find, FUN_001ef0d8 pop): `Brain::b0`, found while `active`.
//! States: 0 grabbed (anim 18), 1 struggling (19, noise every 1 s), 2 dropped (20), 3 knocked (21), 4 held
//! close (19, noise); -1 done (the brain re-plans, FUN_00177618 case 6). States 2 / 3 end at ground contact
//! (+0x208 != 0).
//! The 0x9d anim goal FUN_0017a8c8 pushes (+0x3c = DAT_002a55a0[state]) is kept as (`anim`, `t_anim`) - the
//! guard's anim player reads it (as `Child::React`). UNPORTED leaves: FUN_00153ff8 / FUN_00154038 (voice
//! +0xf64 stop / play `f_3c`), FUN_001f4e28 (PO contact list refresh), FUN_00156698(DAT_003e14b0, pos, g)
//! noise records (counted in `noises`; the 0x3e14b0 list has no listener port).
use super::ClassHost;

/// DAT_002a55a0..b0: 0x9d anim id per state 0..4
pub const ANIMS_2A55A0: [i32; 5] = [18, 19, 20, 21, 19];
/// DAT_002be6c0: noise period (s)
pub const D_2BE6C0: f32 = 1.0;
/// behaviour bit of the component (FUN_00209970(g, 0x16) on deactivate)
pub const BIT_16: u64 = 1 << 0x16;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct C2d0420 {
    /// +0x30 state (-1 = done)
    pub f_30: i32,
    /// +0x34 state start (DAT_002dded0)
    pub f_34: f32,
    /// +0x38 last noise
    pub f_38: f32,
    /// +0x3c voice (brain +0x40 = message +0x10)
    pub f_3c: i32,
    /// active (in the goal stack)
    pub active: bool,
    /// the pushed 0x9d anim goal: anim id, start
    pub anim: Option<(i32, f32)>,
    /// guard +0x32c written by setstate (the guard applies it, `Guard::k24_sync`)
    pub w_32c: Option<i32>,
    /// guard +0x2c0 bits to clear (FUN_00209970), applied with `w_32c`
    pub w_clear: u64,
    /// FUN_00156698 noise records raised
    pub noises: u32,
}

impl C2d0420 {
    /// FUN_00205438(g, 0xb0): FUN_001eee00 pushes a fresh zeroed goal on the guard's +0x26c stack (FUN_001ef138
    /// case 0xb0) and runs slot 6 FUN_0017a6f8: goal base init, FUN_00209890(g, 0x16) (bit 0x16: derived from
    /// `Child::Lifted` by the port, c56.rs), +0x30 = -1
    pub fn new() -> Self {
        C2d0420 { f_30: -1, ..Default::default() }
    }
    /// FUN_0017a9c0
    pub fn ee_sub_17a9c0(&self) -> bool {
        self.f_30 == -1
    }
    /// FUN_0017a8c8(self, s)
    pub fn ee_sub_17a8c8(&mut self, s: i32, now: f32) {
        if self.f_30 == s {
            return;
        }
        // FUN_001ef0d8(self + 0x18): pop the child (the 0x9d anim goal)
        self.anim = None;
        self.f_30 = s;
        self.f_34 = now;
        if s == -1 {
            // FUN_001ef038(self): the goal is finished
            self.w_32c = Some(0);
        } else if (-2 < s) && (s < 5) {
            // FUN_00205438(g, 0x9d) +0x3c = DAT_002a55a0[s], vtable +0x44 (activate)
            self.anim = Some((ANIMS_2A55A0[s as usize], now));
            self.w_32c = Some(((s as u32) < 2) as i32);
        }
    }
    /// slot 7 FUN_0017a730 (activate)
    pub fn ee_sub_17a730(&mut self, guard: usize, now: f32, h: &mut dyn ClassHost) {
        // FUN_001eef70: goal base activate
        self.active = true;
        self.ee_sub_17a8c8(0, now);
        // FUN_0020b210(g) (+0xf80 look mode reset) and FUN_0020ab68(g, 0, 0, 0) (attack stop) need the guard:
        // run by the caller (`Guard::ee_sub_209e28`); FUN_001f4e28 / FUN_00153ff8 / FUN_00154038 UNPORTED
        h.ee_call(0x1f4e28, &[guard as u32 + 1]);
        if self.f_3c != 0 {
            h.ee_call(0x154038, &[guard as u32 + 1, self.f_3c as u32, 2]);
        }
    }
    /// slot 8 FUN_0017a7b8 (deactivate)
    pub fn ee_sub_17a7b8(&mut self, now: f32) {
        // FUN_001eef78
        self.active = false;
        self.ee_sub_17a8c8(-1, now);
        // FUN_00209970(g, 0x16)
        self.w_clear |= BIT_16;
    }
    /// slot 9 FUN_0017a7f8 (update): `ground` = guard +0x208 != 0
    pub fn ee_sub_17a7f8(&mut self, ground: bool, now: f32) {
        // FUN_001eef98: goal base update (child tick: the anim goal plays on its own)
        loop {
            let s = self.f_30;
            if (2..4).contains(&s) && ground {
                self.ee_sub_17a8c8(-1, now);
            }
            if s == self.f_30 {
                if (s == 1 || s == 4) && D_2BE6C0 < now - self.f_38 {
                    self.f_38 = now;
                    // FUN_00156698(DAT_003e14b0, FUN_001a62e8(g), g) +0x54 = 3
                    self.noises += 1;
                }
                return;
            }
        }
    }
}

/// gravity of a guard PO (`*(+0xe8) + 0x80`; physics::driver::GRAVITY_Z, the default the game sets)
pub const GRAVITY: [f32; 3] = [0.0, 0.0, crate::engine::physics::driver::GRAVITY_Z];

/// K24 REPLACED: the guard's PO dynamics (FUN_001f5e10 xfLin integration, ground contact +0x208) while a
/// class-102 bomb lifts it - the port's guards are kinematic patrol bodies. Velocity set by vtable +0x11c,
/// accelerations accumulated by FUN_001fac30, integrated once per frame with the PO gravity; the guard lands
/// (+0x208 set, PO dropped) when no +0x138 record holds it and it falls back to the height it was lifted from.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GuardPo {
    pub vel: [f32; 3],
    pub acc: [f32; 3],
    /// REPLACED ground: the body height when the lift started
    pub ground_z: f32,
}

impl super::c56::Guard {
    /// K24: guard fields the 0xb0 component wrote (FUN_0017a8c8 +0x32c, FUN_00209970 bits)
    pub fn k24_sync(&mut self) {
        if let Some(b) = self.brain.b0.as_mut() {
            if let Some(v) = b.w_32c.take() {
                self.f_32c = v;
            }
            self.f_2c0 &= !b.w_clear;
            b.w_clear = 0;
        }
    }
    /// FUN_00206eb0(g, k): the guard accepts hit kind `k`
    pub fn ee_sub_206eb0(&self, k: i32) -> bool {
        use crate::engine::player::p4::FUN_00132b10;
        if k == -1 {
            return true;
        }
        if self.ee_sub_209a68(7) {
            return false;
        }
        // FUN_00205458(g, 0xa9) +0x4c: the sub-5 script goal (pushed with +0x4c = 1)
        if matches!(self.brain.child, super::brain::Child::Script(_)) {
            return false;
        }
        let is = |c: i32| FUN_00132b10(self.class as i32, c);
        let c = match k {
            8 | 0xd | 0xe => return true,
            0xb => {
                if [0x3b, 0x3d, 0x40, 0x3e, 0x41, 0x3f].into_iter().any(is) {
                    return false;
                }
                0x3c
            }
            0x15 => 0x3e,
            // FUN_00206d48: is-a any of DAT_002c3d10 {0x3b, 0x40, 0x3d, 0x3e, 0x41}
            0x22 | 0x25 => return ![0x3b, 0x40, 0x3d, 0x3e, 0x41].into_iter().any(is),
            _ => return false,
        };
        !is(c) && !is(0x42) && !is(0x43)
    }
    /// FUN_00209e28(g, a): brain message 6 (FUN_00176750(FUN_00205458(g, 0x9c), {6, +0x10 = a}))
    pub fn ee_sub_209e28(&mut self, obj: usize, a: i32, now: f32, dt: f32, rng: &mut crate::engine::physics::driven::GameRng, h: &mut dyn ClassHost) {
        self.brain.f_40 = a;
        let was = matches!(self.brain.child, super::brain::Child::Lifted) && self.brain.sub.code == 6;
        self.k21_goal(obj, 6, now, dt, &[], h);
        if !was {
            // FUN_0017a730 tail: FUN_0020b210(g) (path mode +0xf80 0..3 -> FUN_00204f78 detach) and
            // FUN_0020ab68(g, 0, 0, 0)
            if (0..4).contains(&self.f_f80) {
                h.base(0x204f78, obj);
            }
            self.f_f80 = -1;
            self.atk.ee_sub_20ab68(None, now, rng);
        }
        self.k24_sync();
    }
    /// FUN_00205458(g, 0xb0) +0x30 (None = no component)
    pub fn k24_comp_b0(&self) -> Option<i32> {
        self.brain.b0.as_ref().filter(|b| b.active).map(|b| b.f_30)
    }
    /// FUN_0017a8c8(FUN_00205458(g, 0xb0), s)
    pub fn k24_comp_b0_set(&mut self, s: i32, now: f32) {
        if let Some(b) = self.brain.b0.as_mut().filter(|b| b.active) {
            b.ee_sub_17a8c8(s, now);
        }
        self.k24_sync();
    }
    /// FUN_00209840(g + 0xfd0, rows): target orientation (the body keeps its yaw: +0x1a8 from the x row)
    pub fn ee_sub_209840(&mut self, rows: [[f32; 4]; 3]) {
        self.body.f_1a8 = rows[0][1].atan2(rows[0][0]);
    }
    /// vtable +0x10c: velocity (the PO's, else the root-motion velocity +0x190 in world axes)
    pub fn k24_vel(&self) -> [f32; 3] {
        if let Some(p) = &self.po {
            return p.vel;
        }
        let (s, c) = self.body.yaw.sin_cos();
        let v = self.body.f_190;
        [c * v[0] - s * v[1], s * v[0] + c * v[1], v[2]]
    }
    /// vtable +0x11c: set velocity (starts the REPLACED PO)
    pub fn k24_set_vel(&mut self, v: [f32; 3]) {
        let z = self.body.pos[2];
        self.po.get_or_insert_with(|| GuardPo { ground_z: z, ..Default::default() }).vel = v;
        self.body.k24_air = true;
    }
    /// FUN_001fac30(g, a)
    pub fn k24_accel(&mut self, a: [f32; 3]) {
        if let Some(p) = self.po.as_mut() {
            for k in 0..3 {
                p.acc[k] += a[k];
            }
        }
    }
    /// one frame of the PO (after the guard's slot 15). K27: the E8 xfLin path of FUN_001f3980 on a kind-2
    /// (PO) `AloPhys`: accel reset, slot 69 = ALO gravity FUN_001f9b20 + the accumulated FUN_001fac30
    /// accelerations, FUN_0022de88 integration. The ground landing stays REPLACED (see `GuardPo`).
    pub fn k24_po_step(&mut self, dt: f32) {
        use crate::engine::physics::driver::{AloPhys, FUN_001f9b20, FUN_001fac30, FUN_0022de88};
        let held = !self.f_138.is_empty();
        let Some(p) = self.po.as_mut() else { return };
        let mut b = AloPhys { pos: self.body.pos, v: p.vel, kind: 2, gravity: GRAVITY, ..Default::default() };
        b.a = [0.0; 3];
        FUN_001f9b20(&mut b);
        FUN_001fac30(&mut b, p.acc);
        FUN_0022de88(&mut b, dt);
        p.vel = b.v;
        self.body.pos = b.pos;
        p.acc = [0.0; 3];
        if !held && self.body.pos[2] <= p.ground_z && p.vel[2] <= 0.0 {
            self.body.pos[2] = p.ground_z;
            self.po = None;
            self.body.k24_air = false;
        }
    }
}
