//! WP-K22: C2dbd80 (world class 69, size 0x1e0, parent C2d5898 class 50 = the usable-object base): a
//! player-operated aimable turret / scope. While used (+0x150 tracker set) the left stick pans yaw +0x18c /
//! pitch +0x190 with a limited-acceleration speed (+0x194 / +0x198, scaled by the current field of view
//! +0x1bc), the right stick zooms (+0x1b4 0..1, fov = lerp(+0x1b0, +0x1ac)); with a target object (+0x180)
//! it can instead track it (+0x184) and auto-zoom to frame it (+0x188). A button press (mask 0xff) spawns
//! the template +0x1c0 (FUN_001cb250 -> +0x1c8) and launches it through the launcher +0x1c4, then posts
//! message 5 to itself. The yaw / pitch drive a rotation constraint (+0x1d8, FUN_0012dc18 / 12d108) while
//! +0x1cc is set; using it switches camera policy 9 (FUN_0014e328 / 0014e4c8, desc 0x2de530).
//! Slots: 5 FUN_00113e60, 6 FUN_002259d8, 14 FUN_00225aa8, 15 FUN_00225b60, 40 FUN_00225a68,
//! 60 FUN_00226310, 77 FUN_00225ae8; helpers FUN_00226370 (constraint on/off), FUN_00225ba0 (aim),
//! FUN_00225e98 (zoom), FUN_00226250 (fire), FUN_001c4700, FUN_001c3df8.
//! REPLACED: the auto-zoom target pose FUN_002309e8(dt, target +0x60) (= FUN_00230a98 prediction, unported)
//! uses the target's current position.
use super::ClassHost;
use crate::engine::anim::motion::ee_sub_11cfe0;
use crate::engine::camera::math::FUN_0011bc88;
use crate::engine::physics::dynamics::FUN_0011d018;

/// DAT_002c4408: the speed decay table {5.0, 0.0, 0.5}
const T_2C4408: [f32; 3] = [5.0, 0.0, 0.5];
/// DAT_002c4418 / 2c441c: auto-zoom rate blend; DAT_002a97b0..bc: rate table {0.2, 0.0, 0.5, 1.0}
const D_C4418: f32 = 1.0;
const D_C441C: f32 = 4.0;
const T_2A97B0: [f32; 4] = [0.2, 0.0, 0.5, 1.0];
/// DAT_002c4420 / 24 / 28: zoom speed polynomial
const D_C4420: f32 = 2.0;
const D_C4424: f32 = 0.0;
const D_C4428: f32 = -1.0;
/// DAT_002bbb60: [0, 1]
const R_2BBB60: [f32; 2] = [0.0, 1.0];

/// slot 5: FUN_00113e60
pub fn ee_sub_113e60() -> u32 {
    0x2bb04c
}

/// FUN_0011d948(x, range): clamp into [r0, r1]
fn ee_sub_11d948(x: f32, r: [f32; 2]) -> f32 {
    if x < r[0] {
        r[0]
    } else if r[1] < x {
        r[1]
    } else {
        x
    }
}

/// slot 60 FUN_00226310: FUN_001f4fc8 result `base`, raised to at least 1 when the other object has the
/// template's class (+0x3c)
pub fn ee_sub_226310(base: i32, tmpl_cls: Option<i32>, other_cls: i32) -> i32 {
    match tmpl_cls {
        Some(c) if c == other_cls => base.max(1),
        _ => base,
    }
}

#[derive(Clone, Debug, Default)]
pub struct C2dbd80 {
    /// +0x180 target, +0x184 track it, +0x188 auto-zoom on it
    pub f_180: u32,
    pub f_184: i32,
    pub f_188: i32,
    /// +0x18c yaw, +0x190 pitch, +0x194 / +0x198 their speeds
    pub f_18c: f32,
    pub f_190: f32,
    pub f_194: f32,
    pub f_198: f32,
    /// +0x19c pitch range, +0x1a4 yaw range
    pub f_19c: [f32; 2],
    pub f_1a4: [f32; 2],
    /// +0x1ac / +0x1b0 fov range, +0x1b4 zoom, +0x1b8 zoom speed, +0x1bc fov
    pub f_1ac: f32,
    pub f_1b0: f32,
    pub f_1b4: f32,
    pub f_1b8: f32,
    pub f_1bc: f32,
    /// +0x1c0 spawn template, +0x1c4 launcher, +0x1c8 last spawned
    pub f_1c0: u32,
    pub f_1c4: u32,
    pub f_1c8: u32,
    /// +0x1cc constraint on, +0x1d0 user (self when free), +0x1d4, +0x1d8 constraint handle
    pub f_1cc: i32,
    pub f_1d0: u32,
    pub f_1d4: i32,
    pub f_1d8: u32,
    /// messages posted (vtable +0x1c (self, 5)), counted for tests
    pub fired: u32,
}

fn me(obj: usize) -> u32 {
    obj as u32 + 1
}

impl C2dbd80 {
    /// slot 6 FUN_002259d8
    pub fn ee_sub_2259d8(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1c3d70, obj);
        // DAT_002c4400 = {0.1, 1.0}
        self.f_1ac = 0.1;
        self.f_1b0 = 1.0;
        self.f_1bc = self.f_1b0;
        self.f_1a4 = [-1.5, 1.5];
        self.f_19c = [-0.7, 0.7];
        self.f_1d4 = 1;
        self.f_1d0 = me(obj);
        self.f_1cc = 1;
    }

    /// loader fields 997..1010 (FIELD_MEMBER offsets +0x180..+0x1d4)
    pub fn feed(&mut self, props: &[crate::engine::world::props::Prop], obj: usize, h: &mut dyn ClassHost) {
        use crate::engine::world::props::PropValue;
        let w = |b: &[u8], k: usize| b.get(k * 4..k * 4 + 4).map(|x| u32::from_le_bytes([x[0], x[1], x[2], x[3]]));
        let f = |b: &[u8], k: usize| f32::from_bits(w(b, k).unwrap_or(0));
        for p in props {
            let PropValue::Scalar(b) = &p.value else { continue };
            let i = || w(b, 0).unwrap_or(0) as i32;
            match p.field {
                997 => self.f_180 = h.ee_sub_15e938(1, i(), me(obj)),
                998 => self.f_188 = i(),
                999 => self.f_184 = i(),
                1000 => self.f_19c = [f(b, 0), f(b, 1)],
                1001 => self.f_1a4 = [f(b, 0), f(b, 1)],
                1002 => self.f_1c0 = h.ee_sub_15e938(1, i(), me(obj)),
                1003 => self.f_1c4 = h.ee_sub_15e938(1, i(), me(obj)),
                1005 => self.f_18c = f(b, 0),
                1006 => self.f_190 = f(b, 0),
                1007 => self.f_1cc = i(),
                1009 => self.f_1d4 = i(),
                1010 => {
                    self.f_1ac = f(b, 0);
                    if b.len() >= 8 {
                        self.f_1b0 = f(b, 1);
                    }
                }
                _ => {}
            }
        }
    }

    /// the constraint vector (pextlw / pcpyld packing): {0, -pitch, yaw}
    fn rot(&self) -> [f32; 3] {
        [0.0, -self.f_190, self.f_18c]
    }

    /// FUN_00226370(on)
    pub fn ee_sub_226370(&mut self, obj: usize, on: i32, h: &mut dyn ClassHost) {
        if self.f_1cc == on {
            return;
        }
        self.f_1cc = on;
        if on == 0 {
            h.ee_sub_12e460(&mut self.f_1d8);
        } else {
            self.f_1d8 = h.k22_pin_12dc18(me(obj), self.rot());
        }
    }

    /// slot 14 FUN_00225aa8
    pub fn ee_sub_225aa8(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1f36a8, obj);
        if self.f_1cc != 0 {
            self.f_1cc = 0;
            self.ee_sub_226370(obj, 1, h);
        }
    }

    /// slot 40 FUN_00225a68 (base FUN_001c3df8)
    pub fn ee_sub_225a68(&mut self, obj: usize, other: u32, h: &mut dyn ClassHost) {
        h.base(0x1c3df8, obj);
        if self.f_1d0 == other {
            self.f_1d0 = me(obj);
        }
    }

    /// slot 77 FUN_00225ae8 (FUN_001c3ff0 first): camera policy 9 on / off
    pub fn ee_sub_225ae8(&mut self, obj: usize, on: bool, h: &mut dyn ClassHost) {
        h.base(0x1c3ff0, obj);
        if on {
            h.ee_sub_14e328(9, 0x2de530, me(obj), 0);
        } else {
            h.ee_sub_14e4c8_k(0xf, 9, 0x2de530, me(obj) as i32, 0);
        }
    }

    /// K30: the state the 0x2de530 attach policy (C2cd3c0, FUN_00151950) reads from its pso: the +0x1d0
    /// object's vtable +0xfc world matrix, fov +0x1bc, smooth flag +0x1d4 (classes/k30_cam.rs)
    pub fn k30_publish(&self, obj: usize, h: &mut dyn ClassHost) {
        let mat = h.vcall_fc(self.f_1d0);
        let o = crate::engine::camera::policy::CmObject { mat, fov: self.f_1bc, smooth: self.f_1d4 != 0 };
        h.k30_cam_state(me(obj), super::k30_cam::CamState::Attach(o));
    }
    /// slot 15 FUN_00225b60
    pub fn ee_sub_225b60(&mut self, obj: usize, dt: f32, h: &mut dyn ClassHost) {
        h.base(0x1f36e0, obj);
        self.ee_sub_225ba0(obj, dt, h);
        self.ee_sub_225e98(obj, dt, h);
        self.ee_sub_226250(obj, h);
    }

    /// FUN_00225ba0: aim
    pub fn ee_sub_225ba0(&mut self, obj: usize, dt: f32, h: &mut dyn ClassHost) {
        if self.f_180 != 0 && self.f_184 != 0 {
            let t = h.ee_sub_1a62e8(self.f_180);
            let p = h.ee_sub_1a62e8(me(obj));
            // FUN_0011b7e0 look-along + FUN_0011bc88: the yaw / pitch of the direction
            let (yaw, pitch) = FUN_0011bc88([t[0] - p[0], t[1] - p[1], t[2] - p[2]]);
            let (y0, p0) = (self.f_18c, self.f_190);
            self.f_18c = yaw;
            self.f_190 = pitch;
            self.f_194 = yaw - y0;
            self.f_198 = pitch - p0;
        } else if let Some(s) = h.k22_stick(me(obj)) {
            let mut f7 = ee_sub_11cfe0(s[0] + s[2], 1.0);
            let f6 = ee_sub_11cfe0(s[1], 1.0);
            let mut f8 = self.f_1bc;
            let k = ((1.0 - f8) * 0.5 + f8) * 10.0;
            f7 = -f7 * f7 * f7 * k;
            f8 *= 1.3;
            if f7 * self.f_194 <= 0.0 {
                self.f_194 = 0.0;
            }
            let mut f10 = f6 * f6 * f6 * k;
            if h.k22_setting_3e0fe0() == 0 {
                f10 = -f10;
            }
            if f10 * self.f_198 <= 0.0 {
                self.f_198 = 0.0;
            }
            self.f_194 = FUN_0011d018(self.f_194, 0.0, dt, T_2C4408, None);
            self.f_198 = FUN_0011d018(self.f_198, 0.0, dt, T_2C4408, None);
            self.f_194 = ee_sub_11cfe0(self.f_194 + f7 * dt, f8 * 0.9);
            self.f_198 = ee_sub_11cfe0(self.f_198 + f10 * dt, f8 * 0.8);
            self.f_18c += self.f_194 * dt;
            self.f_190 += self.f_198 * dt;
        }
        self.f_18c = ee_sub_11d948(self.f_18c, self.f_1a4);
        self.f_190 = ee_sub_11d948(self.f_190, self.f_19c);
        if self.f_1cc != 0 {
            h.k22_pin_12d108(self.f_1d8, self.rot());
        }
    }

    /// FUN_00225e98: zoom
    pub fn ee_sub_225e98(&mut self, obj: usize, dt: f32, h: &mut dyn ClassHost) {
        if self.f_180 != 0 && self.f_188 != 0 {
            let r = h.k22_radius_ec30(self.f_180);
            let p = h.ee_sub_1a62e8(me(obj));
            // REPLACED: FUN_002309e8(dt, target +0x60) predicted pose -> the current position
            let t = h.ee_sub_1a62e8(self.f_180);
            let d = [t[0] - p[0], t[1] - p[1], t[2] - p[2]];
            let dist = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            let mut f8 = (r / (dist * 0.5)).atan();
            f8 += f8;
            if f8 < self.f_1ac {
                f8 = self.f_1ac;
            }
            let r2 = r + dist;
            let m = h.ee_sub_1a6318(me(obj));
            let l: [f32; 3] = std::array::from_fn(|k| m[k][0] * d[0] + m[k][1] * d[1] + m[k][2] * d[2]);
            let mut f7 = if 0.0001 < l[0] {
                let a = 2.0 * ((l[1].abs() + r2) / (l[0] * h.k22_aspect_2ddf7c())).atan();
                let b = 2.0 * ((l[2].abs() + r2) / l[0]).atan();
                let a = if a < b { b } else { a };
                if self.f_1b0 < a { self.f_1b0 } else { a }
            } else {
                self.f_1b0
            };
            let lo = self.f_1ac;
            f7 = if lo == self.f_1b0 { 0.0 } else { ((if f7 < f8 { f8 } else { f7 }) - lo) / (self.f_1b0 - lo) };
            let tgt = ee_sub_11d948(1.0 - f7, R_2BBB60);
            let e = (tgt - self.f_1b4).abs();
            let k = (1.0 - e) * D_C4418 + e * D_C441C;
            let tab = [T_2A97B0[0] * k, T_2A97B0[1] * k, T_2A97B0[2], T_2A97B0[3] * k];
            let mut v = self.f_1b8;
            self.f_1b4 = crate::engine::classes::k14::FUN_0011d1c0(self.f_1b4, self.f_1b8, tgt, dt, tab, &mut v);
            self.f_1b8 = v;
        } else if let Some(s) = h.k22_stick(me(obj)) {
            let z = self.f_1b4;
            let v = (D_C4420 + z * (D_C4424 + z * D_C4428)) * s[3] * s[3].abs();
            self.f_1b8 = v;
            self.f_1b4 = ee_sub_11d948(z + v * dt, R_2BBB60);
        }
        let f7 = 1.0 - self.f_1b4;
        self.f_1bc = (1.0 - f7) * self.f_1ac + f7 * self.f_1b0;
    }

    /// FUN_00226250: fire (FUN_001c4700 = FUN_001c46c0(0, +0x150, 0xff) != 0)
    pub fn ee_sub_226250(&mut self, obj: usize, h: &mut dyn ClassHost) {
        if h.k22_stick(me(obj)).is_none() || h.k9_pressed(me(obj), 0xff) == 0 {
            return;
        }
        if self.f_1c0 == 0 {
            h.vcall_1c(obj, 5, 0);
            self.fired += 1;
        } else {
            let s = h.ee_sub_1cb250(self.f_1c0, 0);
            if s != 0 {
                self.f_1c8 = s;
                if self.f_1c4 != 0 {
                    h.k22_launch_1d6b00(self.f_1c4, s);
                }
                h.vcall_1c(obj, 5, 0);
                self.fired += 1;
            }
        }
    }
}

impl super::ClassSet {
    pub(super) fn build_k22_69(&mut self, w: &crate::engine::world::World, h: &mut dyn ClassHost) {
        for (i, o) in w.objects.iter().enumerate() {
            if o.class != 69 {
                continue;
            }
            let mut c = Box::<C2dbd80>::default();
            c.ee_sub_2259d8(i, h);
            if let Some(p) = w.objects.get(o.clone_of.unwrap_or(i)).and_then(crate::engine::game::fields::object_props) {
                c.feed(&super::props_ov(w, i, p), i, h);
            }
            self.k22c.push((i, super::Inst::K22C69(c)));
        }
    }
    pub(super) fn post_load_k22_69(&mut self, h: &mut dyn ClassHost) {
        for (i, x) in &mut self.k22c {
            if let super::Inst::K22C69(c) = x {
                c.ee_sub_225aa8(*i, h);
            }
        }
    }
    pub(super) fn frame_k22_69(&mut self, h: &mut dyn ClassHost) {
        let dt = self.g.dt;
        for (i, x) in &mut self.k22c {
            if let super::Inst::K22C69(c) = x {
                // FEEL R33: slot 15 from the world update list only (LO +0x28 bit 60)
                if !h.lo_removed(*i) {
                    c.ee_sub_225b60(*i, dt, h);
                }
                c.k30_publish(*i, h);
            }
        }
    }
}
