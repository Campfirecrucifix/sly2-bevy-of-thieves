//! WP-K7: the remaining class-56 slots (4, 11, 18, 21, 22, 40, 58, 60, 73 inherited, 74-76, 84-86, 88) and the
//! guard -> world hit path (slot 73 FUN_001f7410 contact damage, FUN_001f7598 delivery).
use super::c56::Guard;
use super::super::ClassHost;

/// 0x60-byte hit record built by FUN_001f7410 / FUN_00176b80 / FUN_0020d3c0 and delivered by FUN_001f7598.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GuardHit {
    /// +0 source, +8 target (handles: object index + 1, `PLAYER_HANDLE` = Sly)
    pub src: u32,
    pub target: u32,
    /// +0xc kind, +0x10 amount
    pub kind: i32,
    pub amount: i32,
    /// +0x20 position, +0x30 vector
    pub pos: [f32; 4],
    pub vec: [f32; 4],
    /// +0x44 (FUN_001f7588: non-zero -> the guard reacts to the hit)
    pub f_44: i32,
}

impl GuardHit {
    /// The same record as player::p7_damage::Hit (src = the guard: is-a 0x38 / 0x33, +0xfbc mode).
    pub fn to_p7(&self, src_pos: [f32; 3], st_fbc: i32) -> crate::engine::player::p7_damage::Hit {
        use crate::engine::player::p7_damage::{Hit, HitSrc};
        Hit {
            src: (self.src != 0).then(|| HitSrc { isa: vec![0x33, 0x38], pos: src_pos, st_fbc, ..Default::default() }),
            kind: self.kind,
            amount: self.amount,
            vec: [self.vec[0], self.vec[1], self.vec[2]],
        }
    }
}

/// Stimulus record (FUN_00156698 / FUN_00156548 pool entry) as slot 88 reads it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Stimulus {
    /// +0x40 source, +0x44 subject, +0x54 level
    pub src: u32,
    pub subject: u32,
    pub level: i32,
    /// +0x20 position
    pub pos: [f32; 3],
    /// K22: +0x30 second position (FUN_00172760 for stimuli not about the guard), +0x4c order key (FUN_0020a168)
    pub pos30: [f32; 3],
    pub f_4c: f32,
    /// +0x14 -> +0x50 kind (0 sound, 1 sight)
    pub mode: i32,
    /// +0x14 the stimulus kind (class 117 record) for the FUN_00156918 hearing test; None = always heard
    /// (records fed by callers without a kind)
    pub kind: Option<super::hearing::StimKind>,
}

/// One contact pair half as FUN_001f7410 reads it (guard +0xec -> +0x6c list, stride via +0xac).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Contact {
    /// the guard-side collision part: +0x18 hit kind (0 = not an attack part), +0x14 amount (short)
    pub kind: i32,
    pub amount: i16,
    /// the other half's object (+0xc)
    pub other: u32,
    /// other half +0x10 contact point, guard half +0x30 vector
    pub pos: [f32; 4],
    pub vec: [f32; 4],
}

impl Guard {
    /// slot 4 FUN_002075d0(g, msg, p): incoming-damage tracker weapon (guard +0x49c +0xc): msg 4 with
    /// p[1] == g and weapon +0x184 != 0 -> FUN_0020d460 sets it; msg 5 with p[1] == g clears it; then
    /// FUN_001eed78(+0x26c, msg, p) (goal-stack message, hook).
    pub fn ee_sub_2075d0(&mut self, obj: usize, msg: i32, weapon: u32, target: u32, weapon_184: i32, h: &mut dyn ClassHost) {
        let me = obj as u32 + 1;
        if msg == 4 && target == me && weapon_184 != 0 {
            self.f_4a8 = weapon;
        } else if msg == 5 && target == me && weapon == self.f_4a8 {
            self.f_4a8 = 0;
        }
        h.base(0x1eed78, obj);
    }

    /// slot 11 FUN_002064d8 (reset): FUN_001f2d48 (base), FUN_00208540(g, 0) target, FUN_001eecd0(brain)
    /// goal stack reset, FUN_00130658(+0x34c) (hook)
    pub fn ee_sub_2064d8(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1f2d48, obj);
        self.f_38c = 0;
        self.f_3cc = None;
        self.brain = Default::default();
        h.base(0x130658, obj);
    }

    /// slot 18 FUN_00207690 (draw: squash on the +0x360 timer, FUN_00168638) -> render hook
    pub fn ee_sub_207690(&self, obj: usize, h: &mut dyn ClassHost) {
        h.ee_sub_168638(obj, 1.0, 0);
    }

    /// slot 21 FUN_002079a0 (remove from world): FUN_001f3e00, unlink from the guard list 0x3e1444, +0xff8 = now
    pub fn ee_sub_2079a0(&mut self, obj: usize, live: &mut Vec<u32>, now: f32, h: &mut dyn ClassHost) {
        h.base(0x1f3e00, obj);
        live.retain(|&g| g != obj as u32 + 1);
        self.f_ff8 = now;
    }

    /// slot 22 FUN_002079e0 (add to world): FUN_001f3ea8, link into 0x3e1444, +0xff8 = DAT_002a8868
    pub fn ee_sub_2079e0(&mut self, obj: usize, live: &mut Vec<u32>, h: &mut dyn ClassHost) {
        h.base(0x1f3ea8, obj);
        if !live.contains(&(obj as u32 + 1)) {
            live.push(obj as u32 + 1);
        }
        self.f_ff8 = f32::MAX;
    }

    /// slot 40 FUN_00206100 (clone from `src`): the base copy keeps this guard's own +0xf64 sensor object;
    /// the sensor state is copied by its vt+0x14c; FUN_00209770(+0xfd0) / FUN_00208250 re-link (hooks).
    pub fn ee_sub_206100(&mut self, obj: usize, src: &Guard, h: &mut dyn ClassHost) {
        let e0 = self.sensor.f_e0;
        *self = src.clone();
        self.sensor.f_e0 = e0;
        h.base(0x209770, obj);
        h.base(0x208250, obj);
    }

    /// slot 58 FUN_00209c88: FUN_001f4fa8 then, by the +0xf80 path mode 0..3, vt+0x294 with the path point
    /// FUN_001e6f10(+0xf88, +0xf84) (hook: the guard path object is not modelled)
    pub fn ee_sub_209c88(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1f4fa8, obj);
        if (0..4).contains(&self.f_f80) {
            h.base(0x1e6f10, obj);
        }
    }

    /// FUN_00209a68(g, bit): behaviour bit (+0x2c0)
    pub fn ee_sub_209a68(&self, bit: u32) -> bool {
        self.f_2c0 >> bit & 1 != 0
    }

    /// slot 60 FUN_00206380(g, other): FUN_001f4fc8 flags, then flag 1 (FUN_00134310) unless bit 7 and
    /// other vt+0x1f4 != 0, or bit 0x17 with the goal 0xb1 on the same +0xec and its +0x50 in 1..5 (< 4 -> keep).
    /// `base` = FUN_001f4fc8 result, `other_1f4` = other vt+0x1f4, `b1` = (same +0xec, goal 0xb1 +0x50).
    pub fn ee_sub_206380(&self, base: u64, other_1f4: bool, b1: Option<(bool, i32)>) -> u64 {
        let keep = if self.ee_sub_209a68(7) {
            other_1f4
        } else if self.ee_sub_209a68(0x17) {
            let Some((same, st)) = b1 else { return base };
            if !same || st < 0 || st > 5 {
                return base;
            }
            st != 0 && st < 4
        } else {
            return base;
        };
        if keep {
            base
        } else {
            base | 1 // FUN_00134310(flags, 1)
        }
    }

    /// slot 74 FUN_00207468 TakeDamage: lethal (slot 85) -> goal message; else immune (slot 75) -> taken; own
    /// projectile (src is-a 0x4f whose +0x200 owner root == g root) -> taken; !FUN_001f7588 -> refused;
    /// FUN_001eed78(+0x26c, 0xc, hit) (goal-stack damage message, hook). `own_shot` = the 0x4f test.
    pub fn ee_sub_207468(&mut self, obj: usize, hit: &GuardHit, own_shot: bool, h: &mut dyn ClassHost) -> bool {
        if !self.ee_sub_207530(hit) {
            if self.ee_sub_207598(hit.kind) {
                return true;
            }
            if own_shot {
                return true;
            }
            if hit.f_44 == 0 {
                return false; // FUN_001f7588
            }
        }
        self.f_damage.push(hit.clone());
        h.base(0x1eed78, obj);
        true
    }

    /// slot 75 FUN_00207598(g, kind): immune (+0x32c, +0x298 or behaviour bit 0x2a)
    pub fn ee_sub_207598(&self, _kind: i32) -> bool {
        self.f_32c != 0 || self.f_298 != 0 || self.ee_sub_209a68(0x2a)
    }

    /// slot 76 FUN_0020b3d0: damage status (bit 7 -> 2, bits 5 / 6 -> 1, else 0)
    pub fn ee_sub_20b3d0(&self) -> i32 {
        if self.ee_sub_209a68(7) {
            2
        } else if self.ee_sub_209a68(5) || self.ee_sub_209a68(6) {
            1
        } else {
            0
        }
    }

    /// slot 84 FUN_0020b090: bit 0x14 -> 1; bits 0x12 / 0x13 / 0x16 -> 0; else +0xfd0 vt+0x64 or
    /// FUN_002040f8 (`rest`)
    pub fn ee_sub_20b090(&self, rest: impl FnOnce() -> i32) -> i32 {
        if self.ee_sub_209a68(0x14) {
            1
        } else if self.ee_sub_209a68(0x12) || self.ee_sub_209a68(0x13) || self.ee_sub_209a68(0x16) {
            0
        } else {
            rest()
        }
    }

    /// slot 85 FUN_00207530(g, hit): lethal = g+0x338 <= -amount, kinds 0 / 1 / 0x80000 / 0x100000 never
    pub fn ee_sub_207530(&self, hit: &GuardHit) -> bool {
        !matches!(hit.kind, 0 | 1 | 0x80000 | 0x100000) && self.f_338 <= -hit.amount
    }

    /// slot 86 FUN_001f1f38(g, a): FUN_00205d10 (base) then +0x494 object vt+0xac(a == 0) (hook)
    pub fn ee_sub_1f1f38(&mut self, obj: usize, a: i32, h: &mut dyn ClassHost) -> bool {
        h.base(0x205d10, obj);
        if self.f_494 != 0 {
            h.vcall(self.f_494, 0xac);
        }
        a == 0
    }

    /// slot 88 FUN_00208be0(g, stimulus) -> perceived. Gate FUN_00208d70 (`gate`); level +0x54 >= +0xe68; not
    /// from this guard (+0x40); level-4 stimuli about another object (+0x44) only when it is a sensor entry;
    /// kind +0x14 -> +0x50: 0 (sound) -> FUN_00156918 (`hear`, unported), 1 (sight) -> inside the vision
    /// volume (+0xf64 -> +0xe8, `p_rest` = the point in the guard rest frame) and no geometry between the guard
    /// position and the point (FUN_0020faa0 filter 0x20fa10 + FUN_0014cc58, `blocked`).
    pub fn ee_sub_208be0(
        &self,
        obj: usize,
        s: &Stimulus,
        gate: bool,
        p_rest: [f32; 3],
        hear: impl FnOnce() -> bool,
        blocked: impl FnOnce([f32; 3], [f32; 3]) -> bool,
    ) -> bool {
        let me = obj as u32 + 1;
        if !gate || s.level < self.f_e68 || (s.src != 0 && s.src == me) {
            return false;
        }
        if s.level == 4 && s.subject != me && self.sensor.ee_sub_154180(s.subject).is_none() {
            return false;
        }
        match s.mode {
            0 => hear(),
            1 => {
                let Some(v) = &self.sensor.f_e8 else { return false };
                super::sensor::ee_sub_227c90(v, p_rest) && !blocked(self.body.pos, s.pos)
            }
            _ => false,
        }
    }

    /// slot 73 FUN_001f7410 (class 0x2caf88, inherited): every contact whose guard-side part carries a hit kind
    /// (+0x18) and whose other object accepts it (FUN_001f7530 owner walk, `accept`) -> hit {src g, target,
    /// kind, amount (short +0x14), pos, vec} delivered by FUN_001f7598 (`deliver`); the contact list is
    /// then freed (FUN_001a7e10).
    pub fn ee_sub_1f7410(obj: usize, contacts: &[Contact], accept: &mut dyn FnMut(u32, i32) -> u32, deliver: &mut dyn FnMut(GuardHit)) {
        for c in contacts {
            if c.kind == 0 {
                continue;
            }
            let t = accept(c.other, c.kind);
            if t == 0 {
                continue;
            }
            deliver(GuardHit { src: obj as u32 + 1, target: t, kind: c.kind, amount: c.amount as i32, pos: c.pos, vec: c.vec, f_44: 0 });
        }
    }
}
