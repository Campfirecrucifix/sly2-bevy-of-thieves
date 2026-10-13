//! WP-K32: guard hit reactions. The guard's slot 74 FUN_00207468 sends FUN_001eed78(+0x26c, 0xc, hit): the
//! goal stack is walked leaf -> root through slot 12; the anim goals' slot 12 (0x174bd0) answers only msg 9
//! and C2cc1c0_v12 returns 0, so the brain (0x9c, vtable 0x2d04a0) slot 12 FUN_00176be8 takes every hit:
//! lethal (slot 85) -> knocked out (sub 0xb); else the +0x350 / +0x354 / +0x358 kind masks, +0x33c health
//! and +0x340 / +0x344 armour decide flinch (FUN_001305b8 overlay), stagger 9 / heavy stagger 10 (recover
//! through FUN_00177ba0: target -> chase 4, else routine 1) or knocked out 0xb (permanent: FUN_00177618 has
//! no case 0xb). Substate children are `brain::HitGoal`.
use super::brain::{Child, Mind, Sub};
use super::c56::Guard;
use super::patrol::Cx;
use super::slots::GuardHit;
use super::super::ClassHost;

/// FUN_001f78a0(kind): index of the lowest set bit (< 26), the row of the +0x8d0 / +0xa70 / +0xc10 hit-anim
/// tables (26 records of 0x10)
pub fn ee_sub_1f78a0(kind: i32) -> usize {
    (0..26).find(|i| kind as u32 & (1 << i) != 0).unwrap_or(0)
}

impl Guard {
    /// FUN_0020a298(g, src, mask): +0x3d0 -> 0; src is-a 0x36 and g not is-a 0x3a and behaviour bit 6 (heavy
    /// stagger) clear -> 1; else the sensor entry of src: 0 only when it is tracked (state != 0) for at least
    /// DAT_002c3dac and no `mask` behaviour bit is set
    pub fn ee_sub_20a298(&self, obj: usize, src: u32, mask: u64, now: f32, h: &mut dyn ClassHost) -> bool {
        if self.f_3d0 != 0 {
            return false;
        }
        if h.ee_sub_132b10(src, 0x36) != 0 && h.ee_sub_132b10(obj as u32 + 1, 0x3a) == 0 && self.f_2c0 & (1 << 6) == 0 {
            return true;
        }
        match self.sensor.ee_sub_154180(src).map(|k| self.sensor.entries[k]) {
            Some(e) if e.state != 0 && now - e.t >= DAT_002C3DAC => self.f_2c0 & mask != 0,
            _ => true,
        }
    }

    /// FUN_0020a378(g, src, mask): src not is-a 0x36, g is-a 0x3a / 0x3d or g == DAT_002dd648 -> FUN_0020a298, else 1
    pub fn ee_sub_20a378(&self, obj: usize, src: u32, mask: u64, now: f32, h: &mut dyn ClassHost) -> bool {
        let me = obj as u32 + 1;
        if h.ee_sub_132b10(src, 0x36) == 0 || h.ee_sub_132b10(me, 0x3a) != 0 || h.ee_sub_132b10(me, 0x3d) != 0 || self.dat_2dd648 {
            self.ee_sub_20a298(obj, src, mask, now, h)
        } else {
            true
        }
    }

    /// FUN_00176750 for a hit substate (the brain record +0x40 = `hit`), `reset`: brain +0x30 = -1 first
    fn k32_set(&mut self, obj: usize, code: i32, hit: &GuardHit, reset: bool, now: f32, dt: f32, h: &mut dyn ClassHost) {
        let pos_of = |_: u32| None;
        let no = |_: u32| false;
        let heard = Vec::new();
        let routines = Vec::new();
        let m = Mind {
            sensor: &self.sensor,
            pos: self.body.pos,
            pos_of: &pos_of,
            is_guard: &no,
            f_38c: self.f_38c,
            f_390: self.f_390,
            f_fbc: self.f_fbc,
            fa8_f94: false,
            c3b: false,
            routine: None,
            cursor: None,
            t30: f32::MAX,
            routines: &routines,
            heard: &heard,
            sg: self.ee_sub_search_g(),
        };
        if reset {
            self.brain.sub.code = -1;
        }
        self.brain.hit = hit.clone();
        let mut cx = Cx { guard: obj, now, dt, body: &mut self.body, h };
        let saved = self.brain.ee_sub_176750(Sub { code, obj: hit.src, routine: None }, &m, &mut cx);
        drop(cx);
        if saved.is_some() {
            self.f_2dc = saved;
        }
        self.k32_enter(obj, now, h);
    }

    /// Slot 7 side effects of the new hit goal: C2d0930_v7 / C2d0830_v7 (FollowWorld 0x15, FUN_00154038 src
    /// tracked), C2d08b0_v7 (first knock-out: +0x330, FollowWorld 0x18 by the focus / 0x17, loot FUN_00154e10
    /// and FUN_00134320 hooks, FUN_00153ff8 sensor reset); behaviour bit (FUN_00209890)
    fn k32_enter(&mut self, obj: usize, now: f32, h: &mut dyn ClassHost) {
        let Child::Hit(g) = &self.brain.child else { return };
        let (code, src) = (g.code, g.hit.src);
        match code {
            9 | 10 => {
                h.ee_sub_1619a0(0x15, obj as u32 + 1);
                if let Some(k) = self.sensor.ee_sub_154180(src) {
                    self.sensor.ee_sub_154080(obj, k, 2, now, h);
                }
                if code == 10 {
                    h.base(0x179780, obj); // +0xfec object vt+0x44
                }
            }
            0xb if self.f_330 == 0 => {
                self.f_330 = 1;
                // FUN_001c4840(0x3dd490): the focus (Sly)
                h.ee_sub_1619a0(if src == super::PLAYER_HANDLE { 0x18 } else { 0x17 }, obj as u32 + 1);
                h.base(0x154e10, obj);
                h.base(0x134320, obj);
                // FUN_00153ff8(+0xf64)
                for e in &mut self.sensor.entries {
                    e.state = 0;
                    e.t = now;
                }
            }
            _ => {}
        }
        let b = g_bit(&self.brain.child);
        self.f_2c0 = (self.f_2c0 & !CHILD_HIT_BITS) | if b >= 0 { 1 << b } else { 0 };
    }

    /// brain slot 12 FUN_00176be8(brain, 0xc, hit). Returns the message result (1 = handled).
    pub fn ee_sub_176be8(&mut self, obj: usize, hit: &GuardHit, now: f32, dt: f32, h: &mut dyn ClassHost) -> bool {
        let mut hit = hit.clone();
        let sub = self.brain.sub.code;
        let child_kind = match &self.brain.child {
            Child::Hit(g) => g.hit.kind as u32,
            _ => 0,
        };
        // vt+0x2b4 = slot 85 FUN_00207530 lethal -> knocked out with +0x44 = 0x40 (focus kind 0x40000 hooks)
        if self.ee_sub_207530(&hit) {
            if hit.kind == 0x40000 {
                self.hit_fx += 1; // FUN_00156698(DAT_003e14bc)
                h.base(0x156698, obj);
            }
            hit.f_44 = 0x40;
            self.k32_set(obj, 0xb, &hit, false, now, dt, h);
            return true;
        }
        match self.f_fbc {
            8 | 0xd => hit.amount <<= 2,
            0xe => self.ee_sub_207030(obj, -1, now, h),
            _ => {}
        }
        if self.f_29c != 0 {
            hit.amount = 0;
        }
        let kind = hit.kind as u32;
        if self.f_4a4 != 0 && sub != 0xb && sub != 5 && (kind == 0x4000 || kind == 0x2000) {
            h.base(0x1f7a68, obj); // FUN_001f7a68(g, 0)
            let z = GuardHit::default();
            self.k32_set(obj, 0x13, &z, false, now, dt, h);
            return true;
        }
        match sub {
            10 if kind & self.f_354 == 0 => return true,
            0xb if kind & self.f_358 == 0 || child_kind & self.f_358 != 0 => return true,
            9 if crate::engine::player::p8_hazard::FUN_001f78d8(&[0; 26], kind) & 0x40 == 0 && (kind & self.f_350 == 0 || child_kind & self.f_350 != 0) => {
                return true
            }
            _ => {}
        }
        // FUN_0020a9a8(+0x3cc, 5)
        if self.f_38c == hit.src {
            self.f_3cc = Some((now, 5));
        }
        let fl8 = hit.f_44 & 8 != 0;
        if kind == 0x80000 && fl8 {
            if sub == 8 {
                // FUN_0017a5b0(child, 0) + child +0x58 = max(+0x58, now + DAT_002be620)
                if let Child::Hit(g) = &mut self.brain.child {
                    g.len = g.len.max(now + DAT_002BE620 - g.t0);
                }
                return true;
            }
            self.f_e88 += hit.amount;
            if self.f_e88 > 0 {
                h.base(0x1305b8, obj);
                return true;
            }
            self.f_e88 = self.f_e84;
            let z = GuardHit::default();
            self.k32_set(obj, 8, &z, false, now, dt, h);
            return true;
        }
        if kind == 0x200000 && fl8 && self.ee_sub_20a378(obj, hit.src, 1 << 6, now, h) {
            self.f_33c = (self.f_33c + hit.amount).max(5);
            self.hit_fx += 1;
            h.base(0x156698, obj);
            let z = GuardHit { src: hit.src, ..Default::default() };
            self.k32_set(obj, 7, &z, false, now, dt, h);
            return true;
        }
        if (kind == 0x1000000 || kind == 0x2000000) && fl8 {
            let code = if self.ee_sub_20a298(obj, hit.src, 1 << 0x17, now, h) { 10 } else { 9 };
            self.k32_set(obj, code, &hit, true, now, dt, h);
            self.ee_sub_207030(obj, if kind == 0x1000000 { 8 } else { 0x22 }, now, h);
            return true;
        }
        if hit.f_44 & 0xf0 == 0 {
            return true;
        }
        // hit effect FUN_00156698(DAT_003e14b0 / 3e14bc for kind 0x40000) (REPLACED: `hit_fx` counter)
        self.hit_fx += 1;
        h.base(0x156698, obj);
        let mut fl = hit.f_44;
        if (8..=10).contains(&sub) && h.ee_sub_132b10(hit.src, 0x65) != 0 && fl & 0xb0 != 0 {
            fl |= 0x40;
        }
        if self.ee_sub_20a298(obj, hit.src, 1 << 0x17, now, h) {
            fl |= 0x80;
        }
        if fl & 0x40 != 0 {
            fl &= !0xb0;
        }
        if fl & 0xa0 != 0 {
            fl &= !0x10;
        }
        if fl & 0x80 != 0 {
            fl &= !0x20;
            let k = ee_sub_1f78a0(hit.kind);
            if self.hit_anim(0, k) == -1 && self.hit_anim(1, k) != -1 {
                fl = fl & !0x80 | 0x20;
            }
            // FUN_00209ad0(g, 1 << 6): the heavy-stagger anim 6 is in the table
            if self.hit_anim(0, k) == -1 && !self.anims.contains(&6) {
                fl = fl & !0x80 | 0x20;
            }
        }
        if fl & 0x10 != 0 {
            if self.f_340 < 1 {
                fl |= 0x20;
            } else {
                self.f_344 += hit.amount;
                if self.f_344 < 1 {
                    fl |= 0x20;
                } else {
                    h.base(0x1305b8, obj); // FUN_001305b8(+0x348 flinch anim, g, 1, g + 0x34c, 0, 1)
                    self.f_360 = now;
                }
            }
        }
        let mut code = 0;
        if fl & 0xa0 != 0 {
            self.f_344 = self.f_340;
            let d = if self.f_340 >= 1 { -self.f_340 } else { hit.amount };
            self.f_33c = (self.f_33c + d).max(0);
            if self.f_33c < 1 {
                fl |= 0x40;
            } else {
                code = if fl & 0x80 != 0 { 10 } else { 9 };
            }
        }
        if fl & 0x40 != 0 {
            code = 0xb;
        }
        if fl & 0xe0 == 0 {
            return true;
        }
        // src is-a 0x34 (player): hit vector = 500 * normalize(g - (src - 150 * src fwd)) (UNPORTED: the source's
        // matrix is not in the hit record; the delivered vector is kept). Else a guard src in mode 0x22 puts
        // an idle (-1) guard in 0x22 (UNPORTED: other guards' +0xfbc are not visible here).
        self.k32_set(obj, code, &hit, true, now, dt, h);
        true
    }

    /// +0x8d0 / +0xa70 / +0xc10 hit-anim record kind (table `t`, row `k`); -1 = none (the slot 14 FUN_002080f0 /
    /// FUN_00209130 oid resolution is not ported: `hit_anims` stays empty)
    pub fn hit_anim(&self, t: usize, k: usize) -> i32 {
        self.hit_anims.get(t).map_or(-1, |r| r[k])
    }

    /// K32: knocked out (brain substate 0xb, the 0xa3 goal)
    pub fn knocked_out(&self) -> bool {
        self.brain.sub.code == 0xb
    }

    /// K32: in a stagger (substates 9 / 10)
    pub fn staggered(&self) -> bool {
        matches!(self.brain.sub.code, 9 | 10)
    }
}

/// DAT_002c3dac (sensor entry age before FUN_0020a298 can refuse), DAT_002be620 (0xaf extension)
const DAT_002C3DAC: f32 = 0.2;
const DAT_002BE620: f32 = 5.0;

/// behaviour bits of the hit goals (5 / 6 / 7)
pub const CHILD_HIT_BITS: u64 = (1 << 5) | (1 << 6) | (1 << 7);

fn g_bit(c: &Child) -> i32 {
    match c {
        Child::Hit(g) => g.bit,
        _ => -1,
    }
}
