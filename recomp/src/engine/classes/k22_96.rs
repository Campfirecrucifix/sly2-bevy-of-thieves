//! WP-K22: C2d3838 (world class 96, parent C2d9978 / SO, size 0x250): a charging creature. Idles (state 0),
//! wanders along a class-10 path +0x16c (1), turns round at the path end (2), spots the nearest target of the
//! world +0x3ebc list (FUN_001c4af8) inside +0x154 (800) - beyond +0x150 (400) only while the target moves
//! (vtable +0x10c speed >= 150) - and turns to it (3), charges along +0x170 or straight at it (4), and on a
//! contact with flag 8 bursts (5: sound 0x14, FUN_00154e10, vtable +0x4c) and messages +0x1c8 (kind 6).
//! Per-state anim table +0x17c (6 x {oid, anim, flags}, loader field 965; flags DAT_002c0230).
//! 1:1: slots 4 FUN_001b10d0, 5 FUN_00114010, 6 FUN_001b0b78, 11 FUN_001b0c68, 13 FUN_001b0ca8, 14
//! FUN_001b0d08, 15 FUN_001b0e28, 16 FUN_001b1030, 18 FUN_001b1140, 25 FUN_001b0d60, 69 FUN_001b10b0, 74
//! FUN_001b0f88 and FUN_001b1328 (setstate) / 1b1900 / 1b1868 / 1b1160 / 1b1bf0 / 1b1b98 / 1b1a38 / 1b0d90 /
//! 1b1818 / 1b17b8, apart from:
//! REPLACED: FUN_0011d448 global rand -> per-instance `GameRng`; FUN_001e6ed8 (warm-started closest path
//! parameter) -> FUN_001e6f60 (global closest); paths = `ClassHost::k22_path` (class-10 `p5_target::Path`);
//! the target list = `ClassHost::k22_targets_3eb8`, its speed (vtable +0x10c) = `k22_speed_10c`.
//! Recorded through `ClassHost::ee_call` (args lost in the decompile / no model): FUN_002303a0 + FUN_0022c840
//! (rotation act stop on a state change), FUN_0022c818 (state 0 act reset), FUN_00154e10 (burst noise),
//! slot 69's FUN_001f59a0(self, +0x1e0 rotation, +0x210 rate) (called by the SO physics, not the frame).
//! UNPORTED: the +0x58 / +0x5c part velocity resets (+0xf0..+0xf8) of slots 13 / 16 (no part physics), +0x1d4
//! (short +0x5c +0x98, unread here); vtable +0x17c(self, 4) / +0x11c state-0 velocity args are guesses
//! (`k22_vcall_i` / zero velocity).
use super::actor::alo::{ee_sub_1f7588, Alo, Touch};
use super::shared::anim::ee_sub_1305b8;
use super::ClassHost;
use crate::engine::physics::driven::GameRng;
use crate::engine::physics::driver::FUN_0011b500;
use crate::engine::player::p5_attach::FUN_0011cf60;

type V3 = [f32; 3];

/// DAT_002c0230: +0x17c entry +8 defaults
pub const FLAGS_2C0230: [u32; 6] = [8, 8, 0xffff_ffff, 0, 8, 0xffff_ffff];
/// DAT_002c0248 / DAT_002c0250: +0x210 rate vector (x, y, z / sqrt(k))
pub const RATE_2C0248: V3 = [25.0, 0.0, 0.25];
/// DAT_002a70e0 (FLT_MAX)
pub const NEVER: f32 = f32::MAX;

/// slot 5: FUN_00114010
pub fn ee_sub_114010() -> u32 {
    0x2bb268
}

#[derive(Clone, Debug)]
pub struct C2d3838 {
    pub alo: Alo,
    /// +0x150 near / +0x154 far sight radius
    pub f_150: f32,
    pub f_154: f32,
    /// +0x158 / +0x15c state-0 time range, +0x160 / +0x164 state-1 time range
    pub f_158: f32,
    pub f_15c: f32,
    pub f_160: f32,
    pub f_164: f32,
    /// +0x168 (FUN_001f7bf8 rate, charge lead time)
    pub f_168: f32,
    /// +0x16c wander path, +0x170 charge path (handles)
    pub f_16c: u32,
    pub f_170: u32,
    /// +0x174 path parameter, +0x178 path direction
    pub f_174: f32,
    pub f_178: f32,
    /// +0x17c: per-state {oid, anim handle, flags}
    pub tab: [(i32, u32, u32); 6],
    /// +0x1c4 anim slot, +0x1c8 listener
    pub f_1c4: u32,
    pub f_1c8: u32,
    /// +0x1cc wander / +0x1d0 charge speed
    pub f_1cc: f32,
    pub f_1d0: f32,
    /// +0x1d4 part count (short from +0x5c vtable +0x98)
    pub f_1d4: i32,
    /// +0x1d8 target yaw, +0x1e0 target rotation (FUN_0011b500 about z)
    pub f_1d8: f32,
    pub f_1e0: [V3; 3],
    /// +0x210 rotation rate, +0x21c its scale
    pub f_210: V3,
    pub f_21c: f32,
    /// +0x220 state, +0x224 state start, +0x228 / +0x22c timers, +0x230 burst sound
    pub st: i32,
    pub f_224: f32,
    pub f_228: f32,
    pub f_22c: f32,
    pub f_230: i32,
    /// REPLACED: FUN_0011d448's global stream
    pub rng: GameRng,
}

impl Default for C2d3838 {
    fn default() -> Self {
        C2d3838 {
            alo: Alo::default(),
            f_150: 0.0,
            f_154: 0.0,
            f_158: 0.0,
            f_15c: 0.0,
            f_160: 0.0,
            f_164: 0.0,
            f_168: 0.0,
            f_16c: 0,
            f_170: 0,
            f_174: 0.0,
            f_178: 0.0,
            tab: [(0, 0, 0); 6],
            f_1c4: 0,
            f_1c8: 0,
            f_1cc: 0.0,
            f_1d0: 0.0,
            f_1d4: 0,
            f_1d8: 0.0,
            f_1e0: [[0.0; 3]; 3],
            f_210: [0.0; 3],
            f_21c: 0.0,
            st: 0,
            f_224: 0.0,
            f_228: 0.0,
            f_22c: 0.0,
            f_230: 0,
            rng: GameRng::default(),
        }
    }
}

fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn len(a: V3) -> f32 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}
/// normalize (|v| < 0.0001 -> (DAT_002a2da0 = 1, 0, 0)) * k
fn dir_k(v: V3, k: f32) -> V3 {
    let l = len(v);
    let d = if 0.0001 <= l { [v[0] / l, v[1] / l, v[2] / l] } else { [1.0, 0.0, 0.0] };
    [d[0] * k, d[1] * k, d[2] * k]
}

impl C2d3838 {
    fn me(obj: usize) -> u32 {
        obj as u32 + 1
    }

    /// slot 6: FUN_001b0b78
    pub fn ee_sub_1b0b78(&mut self, obj: usize, owner_bit: Option<bool>, g_2c3b90: u32, h: &mut dyn ClassHost) {
        self.alo.so.ee_sub_1f2108(obj, owner_bit, g_2c3b90, h);
        self.f_150 = 400.0;
        self.f_154 = 800.0;
        self.f_158 = 2.0;
        self.f_160 = 6.0;
        self.f_164 = 10.0;
        self.f_168 = 1.0;
        self.f_15c = 6.0;
        for (e, f) in self.tab.iter_mut().zip(FLAGS_2C0230) {
            e.0 = -1;
            e.2 = f;
        }
        self.f_1cc = 300.0;
        self.f_1d0 = 1000.0;
        self.f_230 = -1;
        self.st = -1;
        self.alo.f_40 |= 2;
    }

    /// loader fields: 965 {4150 index, 6 oid, 4107 flags}, 968 +0x1c8 (oid, resolved), 971 +0x158, 972 +0x160,
    /// 13 +0x38
    pub fn feed(&mut self, props: &[crate::engine::world::props::Prop], obj: usize, h: &mut dyn ClassHost) {
        use crate::engine::world::props::PropValue;
        let w = |b: &[u8], k: usize| b.get(k * 4..k * 4 + 4).map(|x| u32::from_le_bytes([x[0], x[1], x[2], x[3]]));
        for p in props {
            match (&p.value, p.field) {
                (PropValue::Array(es), 965) => {
                    let (mut ix, mut oid, mut fl) = (None, None, None);
                    for e in es {
                        if let crate::engine::world::props::ArrayItem::One(f, b) = e {
                            match f {
                                4150 => ix = w(b, 0),
                                6 => oid = w(b, 0),
                                4107 => fl = w(b, 0),
                                _ => {}
                            }
                        }
                    }
                    if let Some(e) = ix.and_then(|i| self.tab.get_mut(i as usize)) {
                        if let Some(o) = oid {
                            e.0 = o as i32;
                        }
                        if let Some(f) = fl {
                            e.2 = f;
                        }
                    }
                }
                (PropValue::Scalar(b), 968) => {
                    if let Some(o) = w(b, 0) {
                        self.f_1c8 = h.ee_sub_15e938(1, o as i32, Self::me(obj));
                    }
                }
                (PropValue::Scalar(b), 971) => {
                    self.f_158 = f32::from_bits(w(b, 0).unwrap_or(0));
                    self.f_15c = f32::from_bits(w(b, 1).unwrap_or(0));
                }
                (PropValue::Scalar(b), 972) => {
                    self.f_160 = f32::from_bits(w(b, 0).unwrap_or(0));
                    self.f_164 = f32::from_bits(w(b, 1).unwrap_or(0));
                }
                (PropValue::Scalar(b), 13) => self.alo.f_38 = f32::from_bits(w(b, 0).unwrap_or(0)),
                _ => {}
            }
        }
    }

    /// slot 11: FUN_001b0c68 = FUN_001f2d48, then the anim slot released
    pub fn ee_sub_1b0c68(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1f2d48, obj);
        ee_sub_1305b8(0, 0.0, 0, 0, 0, &mut self.f_1c4, h);
    }

    /// slot 13: FUN_001b0ca8 = FUN_001f31e0 + FUN_002303a0(self, +0x58, 1), then the anims resolved
    pub fn ee_sub_1b0ca8(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1f31e0, obj);
        h.ee_sub_2303a0_off(Self::me(obj), 0x58, 1);
        // zero +0x58 part velocity (+0xf0..+0xf8)
        let p58 = h.obj_i32(Self::me(obj), 0x58) as u32;
        h.obj_set_u32(p58, 0xf0, 0);
        h.obj_set_u32(p58, 0xf4, 0);
        h.obj_set_u32(p58, 0xf8, 0);
        // +0x1d4 = (short)(+0x5c vtable +0x98) part count
        let p5c = h.obj_i32(Self::me(obj), 0x5c) as u32;
        let vt5c = h.obj_i32(p5c, 0xc) as u32;
        self.f_1d4 = h.obj_i32(vt5c, 0x98) as i16 as i32;
        self.ee_sub_1b1bf0(obj, h);
    }

    /// FUN_001b1bf0: resolve every table oid (FUN_001b1b98 = FUN_0015e938(1, oid, self) + vtable +0x4c)
    pub fn ee_sub_1b1bf0(&mut self, obj: usize, h: &mut dyn ClassHost) {
        for k in 0..6 {
            if self.tab[k].0 != -1 {
                let a = h.ee_sub_15e938(1, self.tab[k].0, Self::me(obj));
                if a != 0 {
                    h.vcall(a, 0x4c);
                }
                self.tab[k].1 = a;
                self.tab[k].0 = -1;
            }
        }
    }

    /// FUN_001b1a38(anim, &speed): root-track x travel / anim length (+0x74) * rate (+0x78)
    pub fn ee_sub_1b1a38(obj: usize, anim: u32, out: &mut f32, h: &mut dyn ClassHost) {
        if anim == 0 {
            return;
        }
        if let Some((p0, p1)) = h.root_track_12ff10(anim, obj) {
            *out = ((p1[0] - p0[0]).abs() / h.anim_74(anim)) * h.k22_anim_f(anim, 0x78);
        }
    }

    /// slot 14: FUN_001b0d08
    pub fn ee_sub_1b0d08(&mut self, obj: usize, now: f32, h: &mut dyn ClassHost) {
        h.base(0x1f36a8, obj);
        self.ee_sub_1b1bf0(obj, h);
        Self::ee_sub_1b1a38(obj, self.tab[1].1, &mut self.f_1cc, h);
        Self::ee_sub_1b1a38(obj, self.tab[4].1, &mut self.f_1d0, h);
        self.ee_sub_1b0d90(obj, now, h);
    }

    /// slot 25: FUN_001b0d60 = FUN_001cb518, then reset
    pub fn ee_sub_1b0d60(&mut self, obj: usize, now: f32, h: &mut dyn ClassHost) {
        h.base(0x1cb518, obj);
        self.ee_sub_1b0d90(obj, now, h);
    }

    /// FUN_001b0d90: reset - idle (no wander path) or wander from the path start
    pub fn ee_sub_1b0d90(&mut self, obj: usize, now: f32, h: &mut dyn ClassHost) {
        self.ee_sub_1b17b8(1.0);
        if self.f_16c == 0 {
            self.ee_sub_1b1328(obj, 0, now, h);
        } else {
            self.ee_sub_1b1328(obj, 1, now, h);
            if let Some(p) = h.k22_path(self.f_16c) {
                self.f_174 = p.FUN_001e6610();
                self.f_178 = p.FUN_001e77e8();
                let q = p.FUN_001e67b8(self.f_174).0;
                h.k22_vcall_ec_pos(Self::me(obj), q);
            }
        }
    }

    /// FUN_001b17b8(k)
    pub fn ee_sub_1b17b8(&mut self, k: f32) {
        self.f_210 = [RATE_2C0248[0] * k, RATE_2C0248[1] * k, RATE_2C0248[2] * (1.0 / k.sqrt())];
        self.f_21c = k;
    }

    /// FUN_001b1818(yaw): +0x1d8, +0x1e0 = rotation about z (DAT_002a2dc0)
    pub fn ee_sub_1b1818(&mut self, yaw: f32) {
        self.f_1d8 = yaw;
        self.f_1e0 = FUN_0011b500([0.0, 0.0, 1.0], yaw);
    }

    /// FUN_001b1868: facing the target yaw within 1 degree (row 0 of vtable +0xfc)
    pub fn ee_sub_1b1868(&self, obj: usize, h: &mut dyn ClassHost) -> bool {
        let m = h.vcall_fc(Self::me(obj));
        let y = m[0][1].atan2(m[0][0]);
        FUN_0011cf60(self.f_1d8 - y).abs() < 0.017453294
    }

    /// FUN_001b1900: spot the nearest target (FUN_001c4af8): -> 3 with the yaw toward it, else the state
    pub fn ee_sub_1b1900(&mut self, obj: usize, h: &mut dyn ClassHost) -> i32 {
        let cur = self.st;
        let pos = h.ee_sub_1a62e8(Self::me(obj));
        let Some(t) = Self::ee_sub_1c4af8(pos, h) else { return cur };
        let d = sub(h.ee_sub_1a62e8(t), pos);
        let l = len(d);
        if l <= self.f_154 {
            if self.f_150 < l && len(h.k22_speed_10c(t)) < 150.0 {
                return cur;
            }
            self.ee_sub_1b1818(d[1].atan2(d[0]));
            return 3;
        }
        cur
    }

    /// FUN_001c4af8(world, p): the nearest world +0x3ebc entry with +0x28 bit 60 (the list = targets)
    pub fn ee_sub_1c4af8(p: V3, h: &mut dyn ClassHost) -> Option<u32> {
        let mut best = None;
        let mut bd = f32::MAX; // DAT_002a7708
        for t in h.k22_targets_3eb8() {
            let d = sub(p, h.ee_sub_1a62e8(t));
            let d2 = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
            if d2 < bd {
                bd = d2;
                best = Some(t);
            }
        }
        best
    }

    /// slot 15: FUN_001b0e28 (after the ALO core FUN_001f36e0)
    pub fn ee_sub_1b0e28(&mut self, obj: usize, now: f32, dt: f32, h: &mut dyn ClassHost) {
        self.alo.ee_sub_1f36e0(obj, dt, h);
        let mut s = self.st;
        match self.st {
            0 => {
                let r = self.ee_sub_1b1900(obj, h);
                s = r;
                if r != 3 && self.f_228 <= now {
                    s = 1;
                    if self.f_16c == 0 {
                        s = r;
                    }
                }
            }
            1 => {
                s = self.ee_sub_1b1900(obj, h);
                if s != 3 && self.f_228 <= now {
                    s = 0;
                }
            }
            2 => {
                s = self.ee_sub_1b1900(obj, h);
                if s != 3 && self.ee_sub_1b1868(obj, h) {
                    s = 1;
                }
            }
            3 => {
                if self.f_1c4 == 0 && self.ee_sub_1b1868(obj, h) {
                    s = 4;
                }
            }
            4 => {
                if self.f_22c <= now {
                    h.ee_sub_1f7bf8(obj, self.f_168);
                    self.f_22c = NEVER;
                }
            }
            _ => {}
        }
        self.ee_sub_1b1328(obj, s, now, h);
        self.ee_sub_1b1160(obj, now, h);
    }

    /// FUN_001b1160: follow the state's path (1 wander +0x16c / 4 charge +0x170); off its end in state 1 -> 2
    pub fn ee_sub_1b1160(&mut self, obj: usize, now: f32, h: &mut dyn ClassHost) {
        let (path, speed) = match self.st {
            1 => (self.f_16c, self.f_1cc),
            4 => (self.f_170, self.f_1d0),
            _ => (0, 0.0),
        };
        if path == 0 {
            return;
        }
        let Some(p) = h.k22_path(path) else { return };
        let pos = h.ee_sub_1a62e8(Self::me(obj));
        self.f_174 = p.FUN_001e6f60(pos).s;
        let x = p.FUN_001e6660(self.f_174 + self.f_178);
        if p.FUN_001e6610() <= x && x <= p.FUN_001e6630() {
            let q = p.FUN_001e67b8(x).0;
            h.vcall_11c_v(Self::me(obj), dir_k(sub(q, pos), speed));
            return;
        }
        if self.st == 1 {
            self.ee_sub_1b1328(obj, 2, now, h);
        }
    }

    /// FUN_001b1328 setstate
    pub fn ee_sub_1b1328(&mut self, obj: usize, s: i32, now: f32, h: &mut dyn ClassHost) {
        let cur = self.st;
        if cur == s {
            return;
        }
        let me = Self::me(obj);
        if -2 < cur && (cur < 1 || (1 < cur && cur < 4)) {
            h.ee_call(0x2303a0, &[me]);
            h.ee_call(0x22c840, &[me]);
        }
        match s {
            0 | 1 => {
                let (lo, hi) = if s == 0 {
                    h.vcall_11c_v(me, [0.0; 3]);
                    h.ee_call(0x22c818, &[me]);
                    (self.f_158, self.f_15c)
                } else {
                    h.k22_vcall_i(me, 0x17c, 4);
                    (self.f_160, self.f_164)
                };
                self.f_228 = now + self.rng.FUN_0011d448(lo, hi);
            }
            2 => {
                self.f_178 = -self.f_178;
                let m = h.vcall_fc(me);
                let v = [-m[0][0], -m[0][1], -m[0][2]];
                // FUN_0011efb0: yaw of the reversed facing
                self.ee_sub_1b1818(FUN_0011cf60(v[1].atan2(v[0])));
                h.vcall_11c_v(me, [0.0; 3]);
                h.ee_call(0x2303a0, &[me]);
                h.ee_call(0x22c840, &[me]);
            }
            3 => {
                h.vcall_11c_v(me, [0.0; 3]);
                self.ee_sub_1b17b8(self.f_21c + self.f_21c);
                h.ee_call(0x2303a0, &[me]);
                h.ee_call(0x22c840, &[me]);
            }
            4 => {
                let pos = h.ee_sub_1a62e8(me);
                let target;
                if self.f_170 == 0 {
                    let t = Self::ee_sub_1c4af8(pos, h);
                    target = t.map_or(pos, |t| h.ee_sub_1a62e8(t));
                    self.f_22c = now + 2.0;
                } else if let Some(p) = h.k22_path(self.f_170) {
                    let c = p.FUN_001e6f60(pos);
                    self.f_174 = c.s;
                    target = c.cp;
                    self.f_178 = p.FUN_001e77e8();
                    let d = len(sub(pos, c.cp));
                    self.f_22c = (now + d / self.f_1d0 + (p.FUN_001e6648() - c.l) / self.f_1d0) - self.f_168;
                } else {
                    target = pos;
                    self.f_22c = now - self.f_168;
                }
                let d = sub(target, pos);
                h.vcall_11c_v(me, dir_k([d[0], d[1], 0.0], self.f_1d0));
                h.k22_vcall_i(me, 0x17c, 4);
            }
            5 => {
                h.ee_sub_1fbb90(obj, 0x14);
                h.ee_call(0x154e10, &[me, self.f_230 as u32, (-100i32) as u32]);
                h.vcall(me, 0x4c);
            }
            _ => {}
        }
        self.st = s;
        self.f_224 = now;
        if s != 2 {
            if let Some(&(_, anim, fl)) = usize::try_from(s).ok().and_then(|k| self.tab.get(k)) {
                ee_sub_1305b8(0, 1.0, anim, me, fl, &mut self.f_1c4, h);
            }
        }
    }

    /// slot 16: FUN_001b1030 (turning states: vtable +0x234), then FUN_001f3980
    pub fn ee_sub_1b1030(&mut self, obj: usize, h: &mut dyn ClassHost) {
        if self.st < 4 && 1 < self.st {
            // zero +0x58 part velocity (+0xf0..+0xf8)
            let p58 = h.obj_i32(Self::me(obj), 0x58) as u32;
            h.obj_set_u32(p58, 0xf0, 0);
            h.obj_set_u32(p58, 0xf4, 0);
            h.obj_set_u32(p58, 0xf8, 0);
            // zero +0x5c part velocity (+0xf0..+0xf8)
            let p5c = h.obj_i32(Self::me(obj), 0x5c) as u32;
            h.obj_set_u32(p5c, 0xf0, 0);
            h.obj_set_u32(p5c, 0xf4, 0);
            h.obj_set_u32(p5c, 0xf8, 0);
            h.vcall(Self::me(obj), 0x234);
        }
        h.base(0x1f3980, obj);
    }

    /// slot 18: FUN_001b1140 = FUN_001f3c70
    pub fn ee_sub_1b1140(&mut self, obj: usize, g_2ddf54: u8, h: &mut dyn ClassHost) {
        self.alo.so.ee_sub_1f3c70(obj, g_2ddf54, h);
    }

    /// slot 4: FUN_001b10d0: the turn anim (entry 3) finished (message 2 on the +0x1c4 slot) -> charge
    pub fn ee_sub_1b10d0(&mut self, obj: usize, msg: i32, arg: u32, now: f32, h: &mut dyn ClassHost) {
        if msg == 2 && arg == self.f_1c4 && arg != 0 && h.obj_i32(arg, 0x14) as u32 == self.tab[3].1 {
            self.ee_sub_1b1328(obj, 4, now, h);
        }
    }

    /// slot 69: FUN_001b10b0 = FUN_001f59a0(self, +0x1e0, +0x210) (recorded)
    pub fn ee_sub_1b10b0(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.ee_call(0x1f59a0, &[Self::me(obj)]);
    }

    /// slot 74: FUN_001b0f88: a flag-8 contact bursts it (state 5) and tells +0x1c8 (message kind 6)
    pub fn ee_sub_1b0f88(&mut self, obj: usize, t: &Touch, now: f32, h: &mut dyn ClassHost) -> bool {
        let r = ee_sub_1f7588(t);
        if r && t.w_44 & 8 != 0 && self.st != 5 {
            self.ee_sub_1b1328(obj, 5, now, h);
            if self.f_1c8 != 0 {
                h.vcall_19c_msg(self.f_1c8, 6, Self::me(obj));
            }
        }
        r
    }
}

/// K22: class 96 in the ClassSet (`ClassSet::k22c`)
impl super::ClassSet {
    pub(super) fn build_k22_96(&mut self, w: &crate::engine::world::World, h: &mut dyn ClassHost) {
        for (i, o) in w.objects.iter().enumerate() {
            if o.class != 96 {
                continue;
            }
            let owner_bit = o.owner.map(|ow| self.bit35_of(ow).unwrap_or_else(|| h.owner_bit35(ow)));
            let mut c = Box::<C2d3838>::default();
            c.alo.so.owner = o.owner;
            c.ee_sub_1b0b78(i, owner_bit, self.g.g_2c3b90, h);
            if let Some(p) = w.objects.get(o.clone_of.unwrap_or(i)).and_then(crate::engine::game::fields::object_props) {
                c.feed(p, i, h);
            }
            c.ee_sub_1b0ca8(i, h);
            self.k22c.push((i, super::Inst::K22C96(c)));
        }
    }
    pub(super) fn post_load_k22_96(&mut self, h: &mut dyn ClassHost) {
        let now = self.g.t;
        for (i, x) in &mut self.k22c {
            if let super::Inst::K22C96(c) = x {
                c.ee_sub_1b0d08(*i, now, h);
                h.base(0x1f3ea8, *i);
            }
        }
    }
    pub(super) fn frame_k22_96(&mut self, h: &mut dyn ClassHost) {
        let (now, dt, m) = (self.g.t, self.g.dt, self.g.g_2ddf54);
        for (i, x) in &mut self.k22c {
            if let super::Inst::K22C96(c) = x {
                c.ee_sub_1b0e28(*i, now, dt, h);
                c.ee_sub_1b1030(*i, h);
                c.ee_sub_1b1140(*i, m, h);
            }
        }
    }
}
