//! WP-K8: the pickup family (vtables 0x2d5f68 and descendants). Identified by behaviour:
//! - 95 C2d5f68: clue bottle. Its collect (slot 75 state 1, FUN_001c7ee8) sets the "afClue" bit of its class-7
//!   host's serial (FUN_001c7c30) and recounts DAT_003e1bf4 (the "%d/%d" bottle counter, FUN_001c7d08).
//! - 92 C2d6448: generic drop (collect -> FUN_00164640(+0x230, 2, self)).
//! - 91 C2d6928: coin drop (is-a 0x5b; any player collects; expires after rand(+0x230, +0x234) s; collect ->
//!   coin fx, FUN_00161868 HUD, event 0x12 on 0x2deb30).
//! - 93 C2d66b8: health drop (player slot 79 test, slot 80 (+0x23c hp = 10, +0x240 mp = 20)).
//! - 94 C2d61d8: loot (save var "loot" / "mplootkc" %d ++).
//!
//! Shared code (class 95 slots inherited by the others): slot 4 FUN_001c5098, 6 FUN_001c4e50 core, 14
//! FUN_001c4f60, 15 FUN_001c5e48, 16 FUN_001c5148, 74 FUN_001c5df0, base SetState FUN_001c60b8, pickup test
//! FUN_001c5d18 / FUN_001c5b20 / FUN_001c4bd8.
use super::*;
use crate::engine::physics::driven::GameRng;
use crate::engine::gfx::attach::{affine_from_bytes, BoneWorlds};
use crate::engine::gfx::scene::mul;

/// K8 world class id -> vtable (kept in `ClassSet::k8`).
pub const PORTED_K8: &[(ClassId, u32)] = &[(91, 0x2d6928), (92, 0x2d6448), (93, 0x2d66b8), (94, 0x2d61d8), (95, 0x2d5f68)];

/// slot 5 (class descriptor getter) of the K8 classes
pub fn slot5(class: ClassId) -> Option<u32> {
    // FUN_00114000-range getters are plain `return &desc` stubs; the descriptor addresses are not needed by the
    // port (no K8 caller), so only the class membership is answered.
    PORTED_K8.iter().find(|x| x.0 == class).map(|x| x.1)
}

/// DAT_002c0d4c: velocity damping per second in states 0 / 5
pub const DAMP_2C0D4C: f32 = -5.0;
/// DAT_002c0d50: state-4 settle window (s)
pub const SETTLE_2C0D50: f32 = 5.0;
/// DAT_002c0d7c: pickup reach cull distance
pub const REACH_2C0D7C: f32 = 300.0;
/// DAT_002c0d60: state-3/4 acceleration (gravity)
pub const GRAV_2C0D60: [f32; 4] = [0.0, 0.0, -2250.0, 0.0];
/// anims (FUN_001f0380 ids): 0x184 idle, 0x183 fall, 0x186 vanish
pub const ANIM_IDLE: i32 = 0x184;
pub const ANIM_FALL: i32 = 0x183;
pub const ANIM_VANISH: i32 = 0x186;
/// sound 0x29 (FUN_001fbb90) = collect
pub const SND_COLLECT: i32 = 0x29;

/// Player pickup record (player +0x15c, FUN_001c4bd8 / FUN_001c5b20): +0 alive, +8 radius, +0xc ALO.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PickRec {
    pub alive: i32,
    pub radius: f32,
    pub obj: u32,
}

/// One pickup (C2d5f68 layout; the subclasses add +0x230..+0x244).
#[derive(Clone, Debug)]
pub struct Pickup {
    pub class: ClassId,
    /// +0x150 state (-1 none, 0 idle, 1 collect request, 2 collected / gone, 3 thrown up, 4 falling, 5 vanish,
    /// 6 parked), +0x154 state start time, +0x158 initial state, +0x15c idle anim override
    pub f_150: i32,
    pub f_154: f32,
    pub f_158: i32,
    pub f_15c: i32,
    /// +0x160 anim blend-in (>0 -> FUN_001f10f0), +0x164 anim owner, +0x168 playing anim (None = no handle)
    pub f_160: f32,
    pub f_164: u32,
    pub f_168: Option<i32>,
    /// ANIM-ALL R56: the +0x168 SMA handle (FUN_001f0380 result; 0 = none)
    pub h_168: u32,
    /// +0x16c sparkle child (FUN_0015ed78(1, 0x12))
    pub f_16c: u32,
    /// +0x170 position, +0x180 velocity, +0x190 acceleration
    pub f_170: [f32; 4],
    pub f_180: [f32; 4],
    pub f_190: [f32; 4],
    pub f_1a0: f32,
    /// +0x1a4/+0x1ac/+0x1b4/+0x204/+0x20c/+0x214 oids -> +4 resolved objects (+0x210 = collect listener,
    /// +0x218 = vanish listener)
    pub oids: [i32; 6],
    pub objs: [u32; 6],
    /// +0x1c4 bounced flag, +0x1c8 collectable, +0x1cc auto-collect time
    /// +0x1bc sparkle side (dot sign vs the camera), +0x1c0 last sparkle time (FUN_001c6308)
    pub f_1bc: u32,
    pub f_1c0: f32,
    pub f_1c4: i32,
    pub f_1c8: i32,
    pub f_1cc: f32,
    /// +0x1d0 / +0x1d4 bounce speeds, +0x1d8 collision radius, +0x1dc pickup radius, +0x1e0 / +0x1e4 restitution
    pub f_1d0: f32,
    pub f_1d4: f32,
    pub f_1d8: f32,
    pub f_1dc: f32,
    pub f_1e0: f32,
    pub f_1e4: f32,
    pub f_21c: f32,
    pub f_224: i32,
    /// subclasses: +0x230 (92: target object; 91/93/94: timer min), +0x234 timer max, +0x238 timer,
    /// +0x23c / +0x240 (93: hp / mp gift), +0x244 + 4 (94: loot index)
    pub f_230: f32,
    pub f_230_obj: u32,
    pub f_234: f32,
    pub f_238: f32,
    pub f_23c: i32,
    pub f_240: i32,
    /// +0x244 (94: FUN_001c7a08 clears it on load)
    pub f_244: i32,
    pub f_244_4: i32,
    /// +0x58 transform position (+0x30) / velocity (+0xb0), modelled inline (FUN_001a62e8 of the pickup)
    pub xf_pos: [f32; 4],
    pub xf_vel: [f32; 4],
    /// +0x28 bit 60 (in the world: vt+0x44 add / vt+0x4c remove)
    pub in_world: bool,
    /// REPLACED FUN_001cb658: the class-7 host the pickup is attached to (+0x3c record through the
    /// DAT_003e1398 table), handle; set by `ClassSet::k8_attach`
    pub owner: u32,
    /// the host's +0x100 serial when the host is a ported class-7 instance (`ClassSet::k4`)
    pub owner_100: Option<i32>,
    /// FEEL R39: the spawner copy record +0 (set once FUN_001caec8 has claimed the copy): FUN_001ca978 scores a
    /// never-claimed free copy 4, a returned one <= 3
    pub claimed: bool,
}

impl Default for Pickup {
    fn default() -> Self {
        Pickup {
            class: 95,
            f_150: -1,
            f_154: 0.0,
            f_158: 0,
            f_15c: -1,
            f_160: 0.0,
            f_164: 0,
            f_168: None,
            h_168: 0,
            f_16c: 0,
            f_170: [0.0; 4],
            f_180: [0.0; 4],
            f_190: [0.0; 4],
            f_1a0: 0.0,
            oids: [-1; 6],
            objs: [0; 6],
            f_1bc: 0,
            f_1c0: 0.0,
            f_1c4: 0,
            f_1c8: 0,
            f_1cc: 0.0,
            f_1d0: 0.0,
            f_1d4: 0.0,
            f_1d8: 0.0,
            f_1dc: 0.0,
            f_1e0: 0.0,
            f_1e4: 0.0,
            f_21c: 0.0,
            f_224: 0,
            f_230: 0.0,
            f_230_obj: 0,
            f_234: 0.0,
            f_238: 0.0,
            f_23c: 0,
            f_240: 0,
            f_244: 0,
            f_244_4: 0,
            xf_pos: [0.0, 0.0, 0.0, 1.0],
            xf_vel: [0.0; 4],
            in_world: true,
            owner: 0,
            owner_100: None,
            claimed: false,
        }
    }
}

/// globals + FUN_0011d448 state (`ClassSet::rng`) the pickup code reads
pub struct Cx<'a> {
    pub g: &'a mut ClassGlobals,
    pub rng: &'a mut GameRng,
}
impl std::ops::Deref for Cx<'_> {
    type Target = ClassGlobals;
    fn deref(&self) -> &ClassGlobals {
        self.g
    }
}
impl std::ops::DerefMut for Cx<'_> {
    fn deref_mut(&mut self) -> &mut ClassGlobals {
        self.g
    }
}

fn dot3(a: [f32; 4], b: [f32; 4]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

impl Pickup {
    fn me(obj: usize) -> u32 {
        obj as u32 + 1
    }
    /// +0x210 / +0x218 listener
    pub fn f_210(&self) -> u32 {
        self.objs[4]
    }
    pub fn f_218(&self) -> u32 {
        self.objs[5]
    }

    /// slot-6 core FUN_001c4e50: SO init FUN_001f2108, FUN_001f7918(0x17e, 8), defaults, +0x40 |= 2
    pub fn ee_sub_1c4e50(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1f2108, obj);
        h.base(0x1f7918, obj);
        self.f_1c8 = 1;
        self.f_21c = 30.0;
        // DAT_002a7710 = FLT_MAX
        self.f_1cc = f32::MAX;
        self.f_224 = -1;
        self.f_150 = -1;
        self.f_158 = 0;
        self.f_15c = -1;
        self.oids = [-1; 6];
        h.obj_or_40(Self::me(obj), 2);
    }

    /// slot 6 of each class: FUN_001c7e38 (95), FUN_001c6ab8 (92), FUN_001c6508 (91), FUN_001c67d8 (93),
    /// FUN_001c7648 (94)
    pub fn init(&mut self, class: ClassId, obj: usize, h: &mut dyn ClassHost) {
        self.class = class;
        self.ee_sub_1c4e50(obj, h);
        match class {
            95 => self.f_1dc = 50.0,
            92 => {
                self.f_1dc = 35.0;
                self.f_1a0 = 0.75;
                self.f_1d0 = 500.0;
                self.f_1d4 = 250.0;
                self.f_1e0 = 0.6;
                self.f_1d8 = 35.0;
                self.f_1e4 = 0.6;
            }
            91 => {
                self.f_234 = 5.0;
                self.f_1d8 = 45.0;
                self.f_1dc = 65.0;
                self.f_1d0 = 250.0;
                self.f_1d4 = 150.0;
                self.f_1e0 = 0.5;
                self.f_1a0 = 0.25;
                self.f_1e4 = 0.25;
                self.f_230 = 5.0;
            }
            93 => {
                self.f_1d8 = 45.0;
                self.f_234 = 5.0;
                self.f_1dc = 75.0;
                self.f_1d0 = 250.0;
                self.f_1d4 = 50.0;
                self.f_1e4 = 0.2;
                self.f_1e0 = 0.65;
                self.f_1a0 = 0.25;
                self.f_23c = 10;
                self.f_240 = 20;
                self.f_230 = 4.0;
            }
            94 => {
                self.f_1d8 = 35.0;
                self.f_234 = 10.0;
                self.f_1dc = 40.0;
                self.f_1d0 = 250.0;
                self.f_1d4 = 50.0;
                self.f_1e4 = 0.6;
                self.f_1e0 = 0.75;
                self.f_230 = 8.0;
            }
            _ => {}
        }
    }

    /// slot 14 FUN_001c4f60: base FUN_001f36a8, sparkle child FUN_0015ed78(1, 0x12) removed (vt+0x4c), resolve
    /// the six oids (FUN_0015e938 kind 4), SetState(+0x158)
    pub fn ee_sub_1c4f60(&mut self, obj: usize, g: &mut Cx, h: &mut dyn ClassHost) {
        h.base(0x1f36a8, obj);
        self.f_16c = h.ee_sub_15ed78(1, 0x12, Self::me(obj));
        if self.f_16c != 0 {
            h.vcall(self.f_16c, 0x4c);
        }
        for k in 0..6 {
            if self.oids[k] != -1 {
                self.objs[k] = h.ee_sub_15e938(4, self.oids[k], Self::me(obj));
            }
        }
        self.set_state(obj, self.f_158, g, h);
    }

    /// slot 75 (SetState) of each class: FUN_001c7ee8 (95), FUN_001c6b28 (92), FUN_001c65f8 (91),
    /// FUN_001c68f0 (93), FUN_001c77e8 (94); all end in the base FUN_001c60b8
    pub fn set_state(&mut self, obj: usize, s: i32, g: &mut Cx, h: &mut dyn ClassHost) {
        match self.class {
            95 => self.ee_sub_1c7ee8(obj, s, g, h),
            92 => self.ee_sub_1c6b28(obj, s, g, h),
            91 => self.ee_sub_1c65f8(obj, s, g, h),
            93 => self.ee_sub_1c68f0(obj, s, g, h),
            94 => self.ee_sub_1c77e8(obj, s, g, h),
            _ => self.ee_sub_1c60b8(obj, s, g, h),
        }
    }

    /// FUN_001c7ee8 (95, clue bottle): state 1 -> listener +0x210 msg {1, self} or sound 0x29, FUN_00156b98
    /// at the position, vt+0x1c(5), class-7 host: FUN_001c7c30(host +0x100, 1) + host vt+0x4c + FUN_001618b0
    /// (bottle HUD); then state 2
    pub fn ee_sub_1c7ee8(&mut self, obj: usize, mut s: i32, g: &mut Cx, h: &mut dyn ClassHost) {
        if s == self.f_150 {
            return;
        }
        let me = Self::me(obj);
        if s == 1 {
            if self.f_210() == 0 {
                h.ee_sub_1fbb90(obj, SND_COLLECT);
            } else {
                h.vcall_19c_msg(self.f_210(), 1, me);
            }
            s = 2;
            let p = self.xf_pos;
            h.ee_sub_156b98(obj, [p[0], p[1], p[2]]);
            h.vcall_1c(obj, 5, 0);
            let host = self.owner;
            if host != 0 && h.ee_sub_132b10(host, 7) != 0 {
                let serial = self.owner_100.unwrap_or_else(|| h.obj_i32(host, 0x100));
                ee_sub_1c7c30(g, serial, true);
                h.vcall(host, 0x4c);
                h.ee_sub_1618b0();
            }
        }
        self.ee_sub_1c60b8(obj, s, g, h);
    }

    /// FUN_001c6b28 (92): state 1 -> sound 0, FUN_00164640(+0x230, 2, self), vt+0x1c(5), state 2
    pub fn ee_sub_1c6b28(&mut self, obj: usize, mut s: i32, g: &mut Cx, h: &mut dyn ClassHost) {
        if s == self.f_150 {
            return;
        }
        if s == 1 {
            h.ee_sub_1fbb90(obj, 0);
            if self.f_230_obj != 0 {
                h.ee_sub_164640_src(self.f_230_obj, 2, Self::me(obj));
            }
            s = 2;
            h.vcall_1c(obj, 5, 0);
        }
        self.ee_sub_1c60b8(obj, s, g, h);
    }

    /// FUN_001c65f8 (91, coin): state 0 -> timer +0x238 = rand(+0x230, +0x234); state 1 -> fx FUN_001ce888(9)
    /// (DAT_002c0da0), sound 0x29, listener +0x210, vt+0x1c(5), FUN_00161868 (coin HUD), event 0x12 on
    /// 0x2deb30 (FUN_001619a0), state 2
    pub fn ee_sub_1c65f8(&mut self, obj: usize, mut s: i32, g: &mut Cx, h: &mut dyn ClassHost) {
        if s == self.f_150 {
            return;
        }
        let me = Self::me(obj);
        if s == 0 {
            self.f_238 = g.rng.FUN_0011d448(self.f_230, self.f_234);
        } else if s == 1 {
            let p = self.xf_pos;
            h.ee_sub_1ce888_fx(obj, 0x2c0da0, [p[0], p[1], p[2]]);
            h.ee_sub_1fbb90(obj, SND_COLLECT);
            if self.f_210() != 0 {
                h.vcall_19c_msg(self.f_210(), 1, me);
            }
            s = 2;
            h.vcall_1c(obj, 5, 0);
            h.ee_sub_161868();
            h.ee_sub_1619a0(0x12, me);
        }
        self.ee_sub_1c60b8(obj, s, g, h);
    }

    /// FUN_001c68f0 (93, health): state 0 -> timer; state 1 -> player slot 79 (vt+0x284) refuses -> keep the
    /// state; else fx (DAT_002c0e10), player slot 80 (vt+0x28c)(+0x23c, +0x240), listener +0x210, vt+0x1c(5),
    /// state 2
    pub fn ee_sub_1c68f0(&mut self, obj: usize, mut s: i32, g: &mut Cx, h: &mut dyn ClassHost) {
        if s == self.f_150 {
            return;
        }
        let me = Self::me(obj);
        if s == 0 {
            self.f_238 = g.rng.FUN_0011d448(self.f_230, self.f_234);
        } else if s == 1 {
            let pl = h.ee_sub_1c4840();
            if h.vcall_ret(pl, 0x284) == 0 {
                s = self.f_150;
            } else {
                let p = self.xf_pos;
                h.ee_sub_1ce888_fx(obj, 0x2c0e10, [p[0], p[1], p[2]]);
                h.player_vcall_28c(pl, self.f_23c, self.f_240);
                if self.f_210() != 0 {
                    h.vcall_19c_msg(self.f_210(), 1, me);
                }
                s = 2;
                h.vcall_1c(obj, 5, 0);
            }
        }
        self.ee_sub_1c60b8(obj, s, g, h);
    }

    /// FUN_001c77e8 (94, loot): state 0 -> timer; state 1 -> save var "loot"/"mplootkc"/"%d"(+0x244 + 4) ++
    /// (FUN_001d8a38 read, FUN_001d8b20 write type 0x15), listener +0x210, vt+0x1c(5), state 2
    pub fn ee_sub_1c77e8(&mut self, obj: usize, mut s: i32, g: &mut Cx, h: &mut dyn ClassHost) {
        if s == self.f_150 {
            return;
        }
        let me = Self::me(obj);
        if s == 0 {
            self.f_238 = g.rng.FUN_0011d448(self.f_230, self.f_234);
        } else if s == 1 {
            let n = g.loot.entry(self.f_244_4).or_insert(0);
            *n += 1;
            if self.f_210() != 0 {
                h.vcall_19c_msg(self.f_210(), 1, me);
            }
            s = 2;
            h.vcall_1c(obj, 5, 0);
        }
        self.ee_sub_1c60b8(obj, s, g, h);
    }

    /// base SetState FUN_001c60b8
    pub fn ee_sub_1c60b8(&mut self, obj: usize, s: i32, g: &mut Cx, h: &mut dyn ClassHost) {
        if self.f_150 == s {
            return;
        }
        let mut anim = -1;
        if s != 0 && s != 5 {
            h.ee_sub_1a58d0(obj, 0, 0);
        }
        if self.f_150 == -1 || self.f_150 == 2 {
            self.f_170 = self.xf_pos;
            self.f_180 = self.xf_vel;
        }
        match s {
            0 => {
                anim = if self.f_15c != -1 { self.f_15c } else { ANIM_IDLE };
                self.f_190 = [0.0; 4];
            }
            2 => {
                // DAT_002a773c = FLT_MAX
                self.f_1cc = f32::MAX;
                self.in_world = false;
                h.vcall(Self::me(obj), 0x4c);
                if self.owner != 0 {
                    h.ee_sub_1cb4b8(self.owner, Self::me(obj));
                }
            }
            3 | 4 => {
                anim = ANIM_FALL;
                self.f_190 = GRAV_2C0D60;
            }
            5 => {
                anim = ANIM_VANISH;
                self.f_190 = [0.0; 4];
            }
            _ => {}
        }
        // sparkle child: shown in states 0, 4, 5; hidden otherwise
        let show = s == 0 || s == 4 || s == 5;
        if self.f_16c != 0 {
            h.vcall(self.f_16c, if show { 0x44 } else { 0x4c });
        }
        self.f_150 = s;
        self.f_154 = g.t;
        if anim == -1 {
            if self.f_168.is_some() {
                h.ee_sub_1f0598(obj);
                // ANIM-ALL R56: FUN_001f0598(+0x168), vt +0x14 release
                if self.h_168 != 0 {
                    h.ee_sub_1f0598_link(self.h_168);
                    h.vcall(self.h_168, 0x14);
                    self.h_168 = 0;
                }
                self.f_168 = None;
            }
        } else if self.f_168.is_none() {
            // ANIM-ALL R56: FUN_001f0380(sm = +0x164, obj = this, anim, 0) (the port passed them swapped and kept no
            // handle, so no SMA was ever created for a pickup); vt +0xc add reference
            if self.f_164 != 0 && std::env::var_os("SLY2_R56OFF").is_none() {
                self.h_168 = h.ee_sub_1f0380_link(self.f_164, Self::me(obj), anim, 0);
                if self.h_168 != 0 {
                    h.vcall(self.h_168, 0xc);
                }
            } else {
                h.ee_sub_1f0380(obj, self.f_164, anim as u32, 0);
            }
            self.f_168 = Some(anim);
            if self.f_160 > 0.0 {
                h.ee_sub_1f10f0(obj);
            }
        } else {
            h.ee_sub_1f0660(if self.h_168 != 0 { self.h_168 as usize } else { obj }, anim);
            self.f_168 = Some(anim);
        }
    }

    /// slot 4 FUN_001c5098 (message): base FUN_00133590; msg 10 (anim end) of the playing anim 0x186 ->
    /// listener +0x218 {1, self}, SetState(2)
    pub fn ee_sub_1c5098(&mut self, obj: usize, msg: i32, anim: i32, g: &mut Cx, h: &mut dyn ClassHost) {
        h.base(0x133590, obj);
        if msg == 10 && self.f_168.is_some() && anim == ANIM_VANISH {
            if self.f_218() != 0 {
                h.vcall_19c_msg(self.f_218(), 1, Self::me(obj));
            }
            self.set_state(obj, 2, g, h);
        }
    }

    /// slot 74 FUN_001c5df0 (hit filter): collectable and hit +0x44 bit 3 -> SetState(1), 1
    pub fn ee_sub_1c5df0(&mut self, obj: usize, hit_44: u32, g: &mut Cx, h: &mut dyn ClassHost) -> bool {
        if self.f_1c8 == 0 || hit_44 & 8 == 0 {
            return false;
        }
        self.set_state(obj, 1, g, h);
        true
    }

    /// slot 15: FUN_001c5e48 (95 / 92), FUN_001c6588 (91), FUN_001c6880 (93), FUN_001c7778 (94) = FUN_001c5e48
    /// + state-0 timer +0x238 -= dt -> SetState(5)
    pub fn slot15(&mut self, obj: usize, g: &mut Cx, h: &mut dyn ClassHost) {
        self.ee_sub_1c5e48(obj, g, h);
        if matches!(self.class, 91 | 93 | 94) && self.f_150 == 0 {
            self.f_238 -= g.dt;
            if self.f_238 <= 0.0 {
                self.set_state(obj, 5, g, h);
            }
        }
    }

    /// FUN_001c5e48: ALO update FUN_001f36e0, sparkle FUN_001c6308, per state: 0 fast -> bounce FUN_001c55e0,
    /// 0 / 5 pickup test; 3 falling (vz < 0) -> 4; 4 landed / bounced -> 2 after DAT_002c0d50 s or back to 0
    pub fn ee_sub_1c5e48(&mut self, obj: usize, g: &mut Cx, h: &mut dyn ClassHost) {
        h.base(0x1f36e0, obj);
        self.ee_sub_1c6308(obj, g, h);
        match self.f_150 {
            0 | 5 => {
                if self.f_150 == 0 && dot3(self.f_180, self.f_180) > 10000.0 {
                    self.ee_sub_1c55e0(obj, h);
                }
                self.ee_sub_1c5d18(obj, g, h);
            }
            3 => {
                if self.f_180[2] < 0.0 {
                    self.set_state(obj, 4, g, h);
                }
            }
            4 => {
                if self.f_1c4 == 0 || 25.0 <= self.f_180[2] {
                    if g.t - self.f_154 <= SETTLE_2C0D50 {
                        self.ee_sub_1c55e0(obj, h);
                        self.ee_sub_1c5d18(obj, g, h);
                    } else {
                        self.set_state(obj, 2, g, h);
                    }
                } else {
                    self.f_1c4 = 0;
                    self.set_state(obj, 0, g, h);
                }
            }
            _ => {}
        }
    }

    /// FUN_001c5d18 (pickup test): collectable -> past +0x1cc: SetState(1); else the player records
    /// (FUN_001c4bd8, max 0x10; records after the first only for is-a 0x5b) through FUN_001c5b20
    pub fn ee_sub_1c5d18(&mut self, obj: usize, g: &mut Cx, h: &mut dyn ClassHost) {
        if self.f_1c8 == 0 {
            return;
        }
        if self.f_1cc < g.t {
            self.set_state(obj, 1, g, h);
            return;
        }
        let recs = ee_sub_1c4bd8(0x10, &*g, h);
        for (k, r) in recs.iter().enumerate() {
            if k >= 1 && h.ee_sub_132b10(Self::me(obj), 0x5b) == 0 {
                break;
            }
            if self.ee_sub_1c5b20(obj, r, g, h) {
                return;
            }
        }
    }

    /// FUN_001c5b20(rec): alive record with an ALO; |p - rec ALO| > DAT_002c0d7c -> no; inside the record
    /// radius -> SetState(1); else per ALO part (+0xe4 stride 0xb0; bit 50 skip, bit 49 sphere +0x70 radius
    /// part[2] + +0x1dc, else FUN_001a93c8 radius +0x1dc) -> SetState(1)
    pub fn ee_sub_1c5b20(&mut self, obj: usize, r: &PickRec, g: &mut Cx, h: &mut dyn ClassHost) -> bool {
        if r.alive == 0 || r.obj == 0 {
            return false;
        }
        let p = self.xf_pos;
        let q = h.ee_sub_1a62e8(r.obj);
        let d = [p[0] - q[0], p[1] - q[1], p[2] - q[2]];
        let d2 = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
        if REACH_2C0D7C * REACH_2C0D7C < d2 {
            return false;
        }
        if d2 < r.radius * r.radius || self.parts_1c5b20(r.obj, p, h) {
            self.set_state(obj, 1, g, h);
            return true;
        }
        false
    }

    /// slot 16 FUN_001c5148: state 6 -> base only; v = v (1 + DAT_002c0d4c dt in states 0 / 5) + a dt,
    /// p += v dt, written to the transform (+0x30 / +0xb0, DAT_003e11e8 stamp); the anim's root offset of the
    /// two parts +0x58 / +0x5c (hook); base FUN_001f3980
    pub fn ee_sub_1c5148(&mut self, obj: usize, g: &mut Cx, h: &mut dyn ClassHost) {
        if self.f_150 != 6 {
            let k = if self.f_150 == 0 || self.f_150 == 5 { DAMP_2C0D4C * g.dt + 1.0 } else { 1.0 };
            for i in 0..4 {
                self.f_180[i] = self.f_180[i] * k + self.f_190[i] * g.dt;
                self.f_170[i] += self.f_180[i] * g.dt;
            }
            self.xf_pos = [self.f_170[0], self.f_170[1], self.f_170[2], 1.0];
            self.xf_vel = self.f_180;
            g.g_3e11e8 += 1;
            if self.f_168.is_some() {
                h.ee_sub_1c5148_anim(obj);
            }
        }
        h.base(0x1f3980, obj);
    }

    /// slot 25 FUN_001c7e68 (95) = FUN_001cb518 (add to world: FUN_001f7a68, +0xcc = -1, parts vt+0xd4) +
    /// SetState(0); 92 / 91 / 93 / 94 use FUN_001cb518 alone
    pub fn slot25(&mut self, obj: usize, g: &mut Cx, h: &mut dyn ClassHost) {
        h.base(0x1cb518, obj);
        self.in_world = true;
        if self.class == 95 {
            self.set_state(obj, 0, g, h);
        }
    }
    /// slot 26 FUN_001c7ea8 (95) = FUN_001cb5d0 (remove: parts vt+0xdc) + SetState(-1)
    pub fn slot26(&mut self, obj: usize, g: &mut Cx, h: &mut dyn ClassHost) {
        h.base(0x1cb5d0, obj);
        if self.class == 95 {
            self.set_state(obj, -1, g, h);
        }
    }
}

impl Pickup {
    /// slot 13 (load): FUN_001c4ef8 (95 / 92 / 91 / 93), FUN_001c76d0 (94)
    pub fn slot13(&mut self, obj: usize, h: &mut dyn ClassHost) {
        if self.class == 94 {
            self.ee_sub_1c76d0(obj, h);
        } else {
            self.ee_sub_1c4ef8(obj, h);
        }
    }

    /// FUN_001c4ef8: SO load FUN_001f31e0, FUN_001f7b98(self) +0xf0 = 1, FUN_002303a0 of the +0x58 / +0x5c parts
    /// (0x15), bindings 0x2c0d40 = {0x200, oid 0x182 -> +0x164 (anim owner), activate}
    pub fn ee_sub_1c4ef8(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1f31e0, obj);
        h.ee_sub_1f7b98_f0(Self::me(obj));
        h.ee_sub_2303a0_off(Self::me(obj), 0x58, 0x15);
        h.ee_sub_2303a0_off(Self::me(obj), 0x5c, 0x15);
        let r = h.ee_sub_15e938(1, BIND_2C0D40.1, Self::me(obj));
        if r != 0 {
            h.vcall(r, 0x4c);
            self.f_164 = r;
        }
    }

    /// FUN_001c76d0 (94): FUN_001c4ef8 + FUN_001c7a08 (+0x244 = 0). REPLACED: the +0x240 pool block
    /// (FUN_001ae318 / FUN_001ae238 resize to +0x23c * 0x2c, FUN_001ae478 / FUN_001ae4a0 frame) is allocator state.
    pub fn ee_sub_1c76d0(&mut self, obj: usize, h: &mut dyn ClassHost) {
        self.ee_sub_1c4ef8(obj, h);
        self.f_244 = 0;
    }

    /// slot 18 FUN_001c5fa0 (draw): not in state 2; with a playing anim whose segment has channel 0xc
    /// (FUN_0012ff10) draw at that channel's matrix (host); else the SO draw FUN_001f3c70
    pub fn ee_sub_1c5fa0(&self, obj: usize, h: &mut dyn ClassHost) {
        if self.f_150 == 2 {
            return;
        }
        if self.f_168.is_some() && h.ee_sub_1c5fa0_anim(obj) {
            return;
        }
        // +0x12c (SO alpha) is not written by the pickup code: SO default
        h.so_draw_1f3c70(obj, 1.0);
    }

    /// FUN_001c6308: sparkle when the pickup's (1,1,0) diagonal (FUN_001a6fc0) crosses the camera plane
    /// (0x2ddfb0), with chance +0x1a0 and at most every DAT_002c0d90 s; the emitter (FUN_001ce888(world, 4, 0))
    /// goes to +0x1b0 / +0x1b8 by the side of 0x2ddfa0
    pub fn ee_sub_1c6308(&mut self, obj: usize, g: &mut Cx, h: &mut dyn ClassHost) {
        if self.objs[0] == 0 || self.f_1a0 <= 0.0 {
            return;
        }
        let m = h.ee_sub_1a6318(Self::me(obj));
        let v: [f32; 4] = std::array::from_fn(|k| m[0][k] * DIAG_2C0D80[0] + m[1][k] * DIAG_2C0D80[1] + m[2][k] * DIAG_2C0D80[2]);
        let [c_a0, c_b0] = h.cam_2ddfa0();
        let side = (dot3(v, c_b0) < 0.0) as u32;
        if side == self.f_1bc {
            return;
        }
        let r = g.rng.FUN_0011d448(0.0, 1.0);
        if r < self.f_1a0 && SPARKLE_2C0D90 < g.t - self.f_1c0 {
            let target = if dot3(v, c_a0) < 0.0 { self.objs[1] } else { self.objs[2] };
            if target != 0 {
                let at = h.ee_sub_1a62e8(target);
                let vz = self.xf_vel[2].abs();
                let scale = 1.5 / if vz < 1.0 { 1.0 } else { vz };
                if h.ee_sub_1ce888_sparkle(at, target, self.objs[0], scale) {
                    self.f_1c0 = g.t;
                }
            }
        }
        self.f_1bc = side;
    }

    /// FUN_001c55e0: bounce. World parts in the +-0x1d8 box (FUN_0020faa0, filter FUN_001c5570): sphere parts
    /// (flag bit 49) by distance, others by FUN_001a93c8 contact planes; the deepest penetration whose normal
    /// faces the velocity wins. Ground (n.z > 0.7) in state 4: horizontal friction +0x1e4 (<= 300) and vertical
    /// bounce -+0x1e0 vz (<= 600), a first bounce below +0x1d0 jumps +0x1d4 (+0x1c4 = 1); else
    /// v = -e v_n + f v_t; p -= n * depth
    pub fn ee_sub_1c55e0(&mut self, _obj: usize, h: &mut dyn ClassHost) {
        let r = self.f_1d8;
        let p = self.xf_pos;
        let lo = [p[0] - r, p[1] - r, p[2] - r];
        let hi = [p[0] + r, p[1] + r, p[2] + r];
        let mut best = 0.0f32;
        let mut n = [0.0f32; 4];
        for part in h.ee_sub_20faa0_1c5570(lo, hi) {
            if part.flags & (1 << 49) == 0 {
                if part.f_74 == 0 {
                    continue;
                }
                for [cp, cn] in h.ee_sub_1a93c8(r, [p[0], p[1], p[2]], part.id, 8) {
                    let d = dot3(cp, cn) - cn[3];
                    if d < best && dot3(self.f_180, cn) < 0.0 {
                        n = cn;
                        best = d;
                    }
                }
            } else {
                let c = part.center;
                let dc = [c[0] - p[0], c[1] - p[1], c[2] - p[2], 0.0];
                let d = dot3(dc, dc).sqrt() - part.radius - r;
                if d < best {
                    let dir: [f32; 4] = std::array::from_fn(|k| self.f_170[k] - c[k]);
                    let len = dot3(dir, dir).sqrt();
                    let dir = if len < 0.0001 { X_2A2DA0 } else { dir.map(|x| x / len) };
                    if dot3(self.f_180, dir) < 0.0 {
                        n = dir;
                        best = d;
                    }
                }
            }
        }
        if best >= 0.0 {
            return;
        }
        let v = self.f_180;
        let nv = if 0.7 < n[2] && self.f_150 == 4 {
            // FUN_0011e290(v, up, 0, &perp)
            let vz = dot3(v, UP_2A2DC0);
            let perp: [f32; 4] = std::array::from_fn(|k| v[k] - UP_2A2DC0[k] * vz);
            let len = dot3(perp, perp).sqrt();
            let dir = if len < 0.0001 { ZERO_2A2DD0 } else { perp.map(|x| x / len) };
            let fh = (self.f_1e4 * len).min(FRICT_2C0D74);
            let fv = (-self.f_1e0 * vz).min(BOUNCE_2C0D78);
            let mut nv: [f32; 4] = std::array::from_fn(|k| dir[k] * fh + UP_2A2DC0[k] * fv);
            if self.f_1c4 == 0 && dot3(nv, n) < self.f_1d0 {
                nv = std::array::from_fn(|k| dir[k] * fh + UP_2A2DC0[k] * self.f_1d4);
                self.f_1c4 = 1;
            }
            nv
        } else {
            // FUN_0011e290(v, n, &par, &perp)
            let d = dot3(v, n);
            std::array::from_fn(|k| (n[k] * d) * -self.f_1e0 + (v[k] - n[k] * d) * self.f_1e4)
        };
        for k in 0..4 {
            self.f_170[k] += n[k] * -best;
        }
        self.f_180 = nv;
    }

    /// FUN_001c5b20 part loop over the record ALO's parts (+0xe0 count, +0xe4 stride 0xb0): bit 50 skip; bit 49
    /// sphere (owner +0x70, radius part +8 + +0x1dc); else FUN_001a93c8(+0x1dc, p, part, .., 1) hit
    fn parts_1c5b20(&self, alo: u32, p: [f32; 4], h: &mut dyn ClassHost) -> bool {
        for part in h.alo_parts_e4(alo) {
            if part.flags & (1 << 50) != 0 {
                continue;
            }
            if part.flags & (1 << 49) == 0 {
                if !h.ee_sub_1a93c8(self.f_1dc, [p[0], p[1], p[2]], part.id, 1).is_empty() {
                    return true;
                }
            } else {
                let rr = self.f_1dc + part.radius;
                let c = part.center;
                let d = [p[0] - c[0], p[1] - c[1], p[2] - c[2], 0.0];
                if dot3(d, d) < rr * rr {
                    return true;
                }
            }
        }
        false
    }
}

/// A collision part (SO +0xe4 entry, stride 0xb0) as FUN_001c55e0 / FUN_001c5b20 read it
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct K8Part {
    /// part handle (FUN_001a93c8 argument)
    pub id: u32,
    /// +0x10 flags (bit 49 sphere, bit 50 skip)
    pub flags: u64,
    /// owner (*part) +0x70 world position, part +8 radius
    pub center: [f32; 4],
    pub radius: f32,
    /// +0x74 (geometry present)
    pub f_74: u32,
}

/// FUN_001a64e8 table 0x2c0d40: {flags 0x200, oid 0x182, offset 0x164}
pub const BIND_2C0D40: (u32, i32, u32) = (0x200, 0x182, 0x164);
/// DAT_002c0d80: sparkle diagonal (object space)
pub const DIAG_2C0D80: [f32; 4] = [1.0, 1.0, 0.0, 0.0];
/// DAT_002c0d90: min seconds between sparkles
pub const SPARKLE_2C0D90: f32 = 1.0;
/// DAT_002c0d74 / DAT_002c0d78: ground bounce caps
pub const FRICT_2C0D74: f32 = 300.0;
pub const BOUNCE_2C0D78: f32 = 600.0;
/// DAT_002a2da0 / DAT_002a2dc0 / DAT_002a2dd0
pub const X_2A2DA0: [f32; 4] = [1.0, 0.0, 0.0, 0.0];
pub const UP_2A2DC0: [f32; 4] = [0.0, 0.0, 1.0, 0.0];
pub const ZERO_2A2DD0: [f32; 4] = [0.0; 4];

/// FUN_001c4bd8(n, out): the player's record (FUN_001c4840 +0x15c; none -> DAT_002c0d18 = zero record); a
/// player of class is-a 0x34 adds up to 3 partners of PTR_DAT_002c0d28 (FUN_001eeee0(+0x26c, 0xc2), in the
/// world). Players = ClassGlobals::k6_players (the player-character globals); records from the host.
pub fn ee_sub_1c4bd8(n: usize, g: &ClassGlobals, h: &mut dyn ClassHost) -> Vec<PickRec> {
    let mut out = Vec::new();
    if n < 1 {
        return out;
    }
    let Some(&(pl, _)) = g.k6_players.first() else {
        out.push(PickRec::default());
        return out;
    };
    out.push(h.rec_15c(pl));
    if h.ee_sub_132b10(pl, 0x34) != 0 && n > 1 {
        for &(o, _) in g.k6_players.iter().skip(1).take(3) {
            if o != 0 && o != pl && h.ee_sub_1eeee0(o, 0xc2) && h.obj_flags_28(o) >> 60 & 1 != 0 {
                out.push(h.rec_15c(o));
            }
            if out.len() >= n {
                break;
            }
        }
    }
    out
}

/// FUN_001c7c30(serial, on): set the "afClue" bit `serial` of the current world (FUN_001c7b18 save path,
/// REPLACED by ClassGlobals::af_clue), DAT_003e1bf4 = FUN_001c7d08 recount (popcount)
pub fn ee_sub_1c7c30(g: &mut ClassGlobals, serial: i32, on: bool) {
    let Ok(i) = usize::try_from(serial) else { return };
    if g.af_clue.len() <= i / 32 {
        g.af_clue.resize(i / 32 + 1, 0);
    }
    if on {
        g.af_clue[i / 32] |= 1 << (i % 32);
    } else {
        g.af_clue[i / 32] &= !(1 << (i % 32));
    }
    g.g_3e1bf4 = g.af_clue.iter().map(|w| w.count_ones() as i32).sum();
}

impl ClassSet {
    /// K8: construct + init (slot 6) the pickup classes; transforms from the placement (unplaced template
    /// copies start outside the world)
    pub(super) fn build_k8(&mut self, w: &World, h: &mut dyn ClassHost) {
        let mut bw = BoneWorlds::default();
        let template: std::collections::HashSet<usize> = w.entries.iter().flat_map(|e| e.instances.iter().copied()).collect();
        for (i, o) in w.objects.iter().enumerate() {
            if !PORTED_K8.iter().any(|x| x.0 == o.class) {
                continue;
            }
            let mut c = Box::<Pickup>::default();
            c.init(o.class, i, h);
            c.slot13(i, h);
            // CRAWL R20: FUN_001c9b08 key-5/6 override rows -> FUN_0013f810 member writes on the copy: field 898
            // (+0x158 int) and 906 (+0x230, kind 0x825 = object by oid, FUN_0015e938 resolve)
            if let Some(ov) = w.override_props.get(&i) {
                use crate::engine::world::props::PropValue;
                for p in ov {
                    let PropValue::Scalar(b) = &p.value else { continue };
                    if b.len() < 4 { continue }
                    let v = i32::from_le_bytes([b[0], b[1], b[2], b[3]]);
                    match p.field {
                        898 => c.f_158 = v,
                        906 => c.f_230_obj = w.find_oid(v).first().map_or(0, |&k| k as u32 + 1),
                        _ => {}
                    }
                }
            }
            let place = w.life.placed.get(&i).map(affine_from_bytes);
            let m = match &place {
                Some(pm) => mul(&bw.get(w, i, 0), pm),
                None => bw.get(w, i, 0),
            };
            c.xf_pos = [m[3][0], m[3][1], m[3][2], 1.0];
            c.in_world = !(template.contains(&i) && place.is_none());
            self.k8.push((i, Inst::K8(c)));
        }
    }
    /// K8 load-post: slot 14 FUN_001c4f60
    pub(super) fn post_load_k8(&mut self, h: &mut dyn ClassHost) {
        let g = &mut Cx { g: &mut self.g, rng: &mut self.rng };
        for (i, x) in &mut self.k8 {
            if let Inst::K8(c) = x {
                c.ee_sub_1c4f60(*i, g, h);
            }
        }
        self.k8_proxy_hosts(h);
    }
    /// WIRE-OBJ: the class-7 proxy hosts' deferred slot-22 call (C2d5dc8_v22 queues FUN_001ca8c0 on the world
    /// +0x3ca8 list via FUN_00149ab8 when +0xe4 == 0; the list is drained after the load) for hosts whose
    /// template (B4 class-6 `template` oid) is a single K8 pickup: FUN_001caec8(host, 2) = FUN_001cade0 picks
    /// the template copy of best FUN_001ca978 priority >= 2 (4 free; owned 3 / 2, 1 = an in-world part visible
    /// (FUN_001a61a0) -> kept; ties -> later copy), FUN_001caa60 detaches it from its old host, the copy joins
    /// the host +0xe8 list and takes the host matrix (+0x60..+0x9c), FUN_001c99d8 rebuilds it in the world.
    pub(super) fn k8_proxy_hosts(&mut self, h: &mut dyn ClassHost) {
        use crate::engine::world::loads_b4::B4Load;
        use crate::engine::world::objects::{LoadData, Payload};
        let Some(w) = h.k21_world() else { return };
        let hosts: Vec<(usize, bool)> = self.k4.iter().filter_map(|(i, x)| match x {
            Inst::C2d5dc8(c) => Some((*i, c.f_e4)),
            _ => None,
        }).collect();
        for (hi, e4) in hosts {
            if e4 {
                continue;
            }
            let Some(o) = w.objects.get(hi) else { continue };
            let Payload::Loaded(LoadData::B4(b)) = &o.payload else { continue };
            let B4Load::Class6 { template, xform, .. } = &**b else { continue };
            let Some(e) = w.entries.iter().position(|e| e.dir.c == *template) else { continue };
            if w.entries[e].objects.len() != 1 {
                continue;
            }
            let copies: Vec<usize> = w.entries[e].instances.iter().copied().filter(|c| self.k8.iter().any(|x| x.0 == *c)).collect();
            // FUN_001cade0(host, 2)
            let mut best = 2;
            let mut pick = None;
            for &c in &copies {
                let Some(p) = self.k8_pickup(c) else { continue };
                let pr = if p.owner == 0 {
                    4
                } else if p.in_world && h.ee_sub_1a61a0(c as u32 + 1) {
                    1
                } else if p.in_world {
                    2
                } else {
                    3
                };
                if pr >= best {
                    best = pr;
                    pick = Some(c);
                }
            }
            let Some(c) = pick else { continue };
            let host = hi as u32 + 1;
            let old = self.k8_pickup(c).map_or(0, |p| p.owner);
            if old != 0 {
                // FUN_001caa60: off the old host's +0xe8 list, out of the world
                let oi = old as usize - 1;
                for (i, x) in &mut self.k4 {
                    if let Inst::C2d5dc8(hc) = x {
                        if *i == oi {
                            hc.f_e8.retain(|&k| k != c as u32 + 1);
                        }
                    }
                }
                let g = &mut Cx { g: &mut self.g, rng: &mut self.rng };
                if let Some((_, Inst::K8(p))) = self.k8.iter_mut().find(|x| x.0 == c) {
                    p.slot26(c, g, h);
                    p.in_world = false;
                }
            }
            for (i, x) in &mut self.k4 {
                if let Inst::C2d5dc8(hc) = x {
                    if *i == hi {
                        hc.f_e8.insert(0, c as u32 + 1);
                    }
                }
            }
            let m = affine_from_bytes(xform);
            if let Some(p) = self.k8_pickup(c) {
                p.f_170 = [m[3][0], m[3][1], m[3][2], 1.0];
                p.f_180 = [0.0; 4];
                p.xf_pos = p.f_170;
            }
            self.k8_attach(c, host, h);
        }
    }
    /// Drop sources per frame. (a) C2d08b0_v7 first knock-out of a guard (+0x330 0 -> 1, guard/hit.rs
    /// `base(0x154e10)` hook): FUN_00154e10(&DAT_002de578, guard +0x3e0 world point, n = `ko_coins()`
    /// (+0x3d8, or (< 0) half the guard's +0x404 coin piles), +0x3f0, actor) -> coin drop FUN_001c6fd8(0x5b,
    /// pos, n) and the DAT_002de578 health countdown (c+4 <= 0 -> FUN_001c6fd8(0x5d, pos, 1)). (b) queued
    /// host requests (`ClassHost::k8_take_drops`, e.g. K23 FUN_001c6fd8(0x5d) health).
    pub(super) fn k8_drops(&mut self, h: &mut dyn ClassHost) {
        let mut reqs = h.k8_take_drops();
        let mut kos = Vec::new();
        for (i, x) in &self.k6 {
            if let Inst::Guard(gd) = x {
                if gd.f_330 != 0 && !self.k8_ko_dropped.contains(i) {
                    kos.push((*i, gd.drop_pos_3e0(), gd.ko_coins(), gd.f_3f0));
                }
            }
        }
        for (o, pos, n, di) in kos {
            self.k8_ko_dropped.push(o);
            self.ee_sub_154e10(pos, n, di, o as u32 + 1, &mut reqs, h);
        }
        for r in &reqs {
            self.k8_drop(r.0, r.1, r.3, h);
        }
    }
    /// FUN_00154e10(&DAT_002de578 c, pos, n, di, actor): counters below 1 re-rolled from table c+0x14 =
    /// DAT_002bdc00 (two FUN_00154c90-weighted rolls over +0x28/+0x30 & +0x2c/+0x34 (coins), +0x38/+0x40 &
    /// +0x3c/+0x44 (items), FUN_0011d3f8 between them); coins -= (FUN_00132b10(actor, 0x38) ? 2 : 1), items +=
    /// di; n < 0 -> FUN_0011d3f8(+0x10/+0x14 when coins ran out, else +0x18/+0x1c); n > 0 -> coins
    /// FUN_001c6fd8(0x5b, pos, n); items < 1 -> one health FUN_001c6fd8(0x5d, pos, 1).
    fn ee_sub_154e10(&mut self, pos: [f32; 3], mut n: i32, di: i32, actor: u32, reqs: &mut Vec<(i32, [f32; 3], u32, i32)>, h: &mut dyn ClassHost) {
        let exe = crate::engine::disc::exe::exe();
        let t = exe.w(0x002d_e578 + 0x14);
        let t = if t != 0 { t } else { 0x002b_dc00 };
        let tw = |k: u32| exe.w(t + k) as i32;
        let roll = |h: &mut dyn ClassHost, lo: i32, hi: i32| -> i32 {
            let r = h.ee_sub_154c90();
            if lo == hi {
                hi
            } else if lo < hi {
                lo + (r * ((hi as f32 + 0.999) - lo as f32)) as i32
            } else {
                hi + ((1.0 - r) * ((lo as f32 + 0.999) - hi as f32)) as i32
            }
        };
        for (k, a, b) in [(0usize, 0x28u32, 0x2cu32), (1, 0x38, 0x3c)] {
            if self.g.g_2de578[k] < 1 {
                let x = roll(h, tw(a), tw(a + 8));
                let y = roll(h, tw(b), tw(b + 8));
                self.g.g_2de578[k] = self.rng.FUN_0011d3f8(x, y);
            }
        }
        let d = if h.ee_sub_132b10(actor, 0x38) != 0 { -2 } else { -1 };
        self.g.g_2de578[1] += di;
        self.g.g_2de578[0] += d;
        if n < 0 {
            let (lo, hi) = if self.g.g_2de578[0] < 1 { (tw(0x10), tw(0x14)) } else { (tw(0x18), tw(0x1c)) };
            n = self.rng.FUN_0011d3f8(lo, hi);
        }
         if 0 < n {
            reqs.push((0x5b, pos, actor, n));
        }
        if self.g.g_2de578[1] < 1 {
            reqs.push((0x5d, pos, actor, 1));
        }
    }
    /// FUN_001c6de8(pos, parts, out, down, up, min): ground / ceiling probe along DAT_002a2dc0 (+z). No parts
    /// -> FUN_0020faa0(pos .. A, filter FUN_001c5570). up > 0: FUN_0014cc58(pos -> pos + z up) hit starting
    /// inside (+0x40 t0 == 0): exit +0x44 == 1 or exit-plane z +0x38 < min -> fail, else the exit point +0x10.
    /// Else down < 0: FUN_0014cc58(pos -> pos + z down) entry (t0 != 0) with entry-plane z +0x28 >= min ->
    /// the entry point; down >= 0 -> pos.
    fn ee_sub_1c6de8(pos: [f32; 3], parts: Option<&[K8Part]>, down: f32, up: f32, min: f32, h: &mut dyn ClassHost) -> Option<[f32; 3]> {
        let a = [pos[0], pos[1], pos[2] + up];
        let b = [pos[0], pos[1], pos[2] + down];
        let own;
        let parts = match parts {
            Some(p) => p,
            None => {
                let lo = std::array::from_fn(|i| pos[i].min(a[i]));
                let hi = std::array::from_fn(|i| pos[i].max(a[i]));
                own = h.ee_sub_20faa0_1c5570(lo, hi);
                &own[..]
            }
        };
        if 0.0 < up {
            if let Some(l) = h.ee_sub_14cc58(parts, pos, a) {
                if l.t[0] == 0.0 {
                    if l.t[1] == 1.0 || l.plane[1][2] < min {
                        return None;
                    }
                    return Some(l.pos[1]);
                }
            }
        }
        if down < 0.0 {
            let l = h.ee_sub_14cc58(parts, pos, b)?;
            (l.t[0] != 0.0 && !(l.plane[0][2] < min)).then_some(l.pos[0])
        } else {
            Some(pos)
        }
    }
    /// FUN_001c6fd8(0x3dd490, kind, pos, actor, n) + FUN_001c6ce8: kind 0x5b -> coin class 91 (template 0x2b3),
    /// 0x5d -> health class 93 (0x2b4). Probe g = FUN_001c6de8(pos, 0, 300, -FLT_MAX); snap onto the nav
    /// region (FUN_001c0940, z kept); |g - p|xy > DAT_002c0e80 -> none; p = FUN_001c6de8(p, -250, 1500); up to
    /// n free template copies (FUN_001cb250, SetState(2)); world parts in p +- 300 (FUN_0020faa0); spiral from
    /// FUN_0011d448(-pi, pi) / radius 100 (angle += 1.8 FUN_0011cf60, radius += 23, > 300 -> -= 275), 50 tries:
    /// target t = spot + outward * clamp(200 - 250 (i 0.02), 0, 150) with clear nav lines both ways and a
    /// ground hit FUN_001c6de8(spot, parts, -300, 300, 0.7) -> landing = hit + z (+0x1d8 - 1), flight time
    /// FUN_00196928(850, -850, 1, 1, landing - g, DAT_002c0d60) + k 0.075, launch FUN_001c6bd0(k * 0.1,
    /// FLT_MAX, g, (landing - g) / t - grav t / 2) (state 3). Unplaced copies: SetState(2). Returns the placed.
    /// Nav region: the host's (FUN_001c4840 focus region / FUN_001c0d70 lookup not distinguished: region 0).
    pub fn k8_drop(&mut self, kind: i32, pos: [f32; 3], n: i32, h: &mut dyn ClassHost) -> Vec<usize> {
        use crate::engine::player::p5_states::FUN_00196928;
        let class = match kind {
            0x5b => 91,
            0x5d => 93,
            _ => return Vec::new(),
        };
        let mut out = Vec::new();
        let Some(g) = Self::ee_sub_1c6de8(pos, None, 0.0, 300.0, -f32::MAX, h) else { return out };
        let mut p = match h.ee_sub_1c0940(0, g) {
            Some(q) => [q[0], q[1], g[2]],
            None => return out,
        };
        if ((g[0] - p[0]).powi(2) + (g[1] - p[1]).powi(2)).sqrt() > 2000.0 {
            return out;
        }
        match Self::ee_sub_1c6de8(p, None, -250.0, 1500.0, -f32::MAX, h) {
            Some(q) => p = q,
            None => return out,
        }
        // FUN_001c6ce8 (n == 0 -> none): n x FUN_001cb250 -> FUN_001cade0 keeps the LAST copy of the best
        // FUN_001ca978 score (`min <= s`): host-owned copies (record +8 != 0) score 0, never-claimed free copies
        // 4 before returned ones (3), from the end (FEEL
        // R39, cap_feel_t3_coins: the t3 coin bursts claim copies 24, 23, 22, then 21..18, 17..14, 13..11)
        let mut cand: Vec<(bool, usize)> = self.k8.iter().filter_map(|(i, x)| match x {
            Inst::K8(c) if c.class == class && !c.in_world && c.owner == 0 => Some((c.claimed, *i)),
            _ => None,
        }).collect();
        cand.sort_by(|a, b| a.0.cmp(&b.0).then(b.1.cmp(&a.1)));
        let free: Vec<usize> = cand.into_iter().take(n.max(0) as usize).map(|c| c.1).collect();
        for (i, x) in &mut self.k8 {
            if let Inst::K8(c) = x {
                if free.contains(i) {
                    c.f_150 = 2;
                    c.claimed = true;
                }
            }
        }
        if free.is_empty() {
             return out;
        }
        let r = 300.0f32;
        let parts = h.ee_sub_20faa0_1c5570([p[0] - r, p[1] - r, p[2] - r], [p[0] + r, p[1] + r, p[2] + r]);
        let mut ang = self.rng.FUN_0011d448(-std::f32::consts::PI, std::f32::consts::PI);
        let mut rad = 100.0f32;
        let grav = [GRAV_2C0D60[0], GRAV_2C0D60[1], GRAV_2C0D60[2]];
        let mut i = 0;
        let mut launch = Vec::new();
        loop {
            let t = i as f32 * 0.02;
            let hh = (200.0 + t * (-250.0 + t * 0.0)).clamp(0.0, 150.0);
            let v5 = [p[0] + ang.cos() * rad, p[1] + ang.sin() * rad, p[2]];
            let v3 = [v5[0] - g[0], v5[1] - g[1], 0.0];
            let len = (v3[0] * v3[0] + v3[1] * v3[1]).sqrt();
            let dir = if len < 1.0e-4 { [1.0, 0.0, 0.0] } else { [v3[0] / len, v3[1] / len, 0.0] };
            let tgt = [v5[0] + dir[0] * hh, v5[1] + dir[1] * hh, v5[2]];
            let clear = !h.ee_sub_1c0980(0, v5, tgt) && !h.ee_sub_1c0980(0, tgt, v5);
            if clear {
                if let Some(hit) = Self::ee_sub_1c6de8(tgt, Some(&parts), -300.0, 300.0, 0.7, h) {
                    let o = free[launch.len()];
                    let lift = match self.k8.iter().find(|x| x.0 == o) {
                        Some((_, Inst::K8(c))) => c.f_1d8 - 1.0,
                        _ => 0.0,
                    };
                    let land = [hit[0], hit[1], hit[2] + lift];
                    let d = [land[0] - g[0], land[1] - g[1], land[2] - g[2]];
                    let tf = FUN_00196928(850.0, -850.0, 1.0, 1.0, d, grav) + launch.len() as f32 * 0.075;
                    let vel = [d[0] / tf - grav[0] * tf * 0.5, d[1] / tf - grav[1] * tf * 0.5, d[2] / tf - grav[2] * tf * 0.5];
                    launch.push((o, vel));
                }
            }
            ang = crate::engine::camera::math::FUN_0011cf60(ang + 1.8);
            rad += 23.0;
            if r < rad {
                rad -= r - 25.0;
            }
            i += 1;
            if i >= 0x32 || launch.len() >= free.len() {
                break;
            }
        }
        
        for (k, &(o, vel)) in launch.iter().enumerate() {
            let gx = &mut Cx { g: &mut self.g, rng: &mut self.rng };
            if let Some((_, Inst::K8(c))) = self.k8.iter_mut().find(|x| x.0 == o) {
                c.in_world = true;
                c.f_170 = [g[0], g[1], g[2], 1.0];
                c.xf_pos = c.f_170;
                c.f_160 = (k + 1) as f32 * 0.1;
                c.f_1c4 = 0;
                c.set_state(o, 3, gx, h);
                c.f_180 = [vel[0], vel[1], vel[2], 0.0];
                c.xf_vel = c.f_180;
                c.f_1cc = f32::MAX;
            }
            out.push(o);
        }
        out
    }
    /// K8 per frame: slots 15 then 16 of every pickup in the world
    pub(super) fn frame_k8(&mut self, h: &mut dyn ClassHost) {
        let g = &mut Cx { g: &mut self.g, rng: &mut self.rng };
        for (i, x) in &mut self.k8 {
            if let Inst::K8(c) = x {
                if c.in_world {
                    c.slot15(*i, g, h);
                }
                if c.in_world {
                    c.ee_sub_1c5148(*i, g, h);
                }
                if c.in_world {
                    c.ee_sub_1c5fa0(*i, h);
                }
            }
        }
    }
    /// pickup state of object `obj`
    pub fn k8_pickup(&mut self, obj: usize) -> Option<&mut Pickup> {
        self.k8.iter_mut().find_map(|(i, x)| match x {
            Inst::K8(c) if *i == obj => Some(&mut **c),
            _ => None,
        })
    }
    /// REPLACED attachment (FUN_001cb658 table DAT_003e1398): attach pickup `obj` to class-7 host `host`
    /// (handle) and put it in the world (slot 25)
    pub fn k8_attach(&mut self, obj: usize, host: u32, h: &mut dyn ClassHost) {
        let serial = (host as usize).checked_sub(1).and_then(|hi| {
            self.k4.iter().find_map(|(i, x)| match x {
                Inst::C2d5dc8(c) if *i == hi => Some(c.f_100),
                _ => None,
            })
        });
        let g = &mut Cx { g: &mut self.g, rng: &mut self.rng };
        let Some(c) = self.k8.iter_mut().find_map(|(i, x)| match x {
            Inst::K8(c) if *i == obj => Some(c),
            _ => None,
        }) else {
            return;
        };
        c.owner = host;
        c.owner_100 = serial;
        c.slot25(obj, g, h);
    }
}
