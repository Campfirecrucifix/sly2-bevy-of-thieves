//! C2ceb78 (world class 29): particle emitter object (template at +0xe4 shared with C2ce4d0's layout).
//! Object handles passed to/returned from hooks here are `index + 1` (0 = null).
use super::c2ce4d0::Template;
use super::{ClassGlobals, ClassHost};
use crate::engine::physics::driven::GameRng;
use crate::engine::world::props::{Prop, PropValue};

/// DAT_002a5148 / DAT_002a514c
const FLT_MAX: f32 = f32::MAX;

#[derive(Clone, Debug)]
pub struct C2ceb78 {
    /// +0xe0: template dirty (set by the template setters FUN_00258658(.., 1) / FUN_0015a240)
    pub f_e0: i32,
    /// *(+0xe4): emitter template (w[0] = reference count, w[4] = +0x10 shape kind, block = *(+0x8))
    pub tpl: Template,
    /// *(template + 0x30) for shape kind 1: box {min xyzw, max xyzw}
    pub bx: [f32; 8],
    /// +0xf0 mode (-1 = pick in slot 14)
    pub f_f0: i32,
    /// +0xf4 remaining particle budget (-1 = unlimited)
    pub f_f4: i32,
    /// +0xf8 / +0xfc rate range
    pub f_f8: f32,
    pub f_fc: f32,
    pub f_100: i32,
    pub f_104: f32,
    pub f_108: f32,
    pub f_10c: f32,
    pub f_110: f32,
    pub f_114: i32,
    /// +0x118 group oid (-1 = none)
    pub f_118: i32,
    /// +0x11c list (link +0x128): group members (object handles)
    pub f_11c: Vec<u32>,
    pub f_130: f32,
    pub f_134: f32,
    pub f_138: f32,
    pub f_13c: f32,
    pub f_140: f32,
    pub f_144: f32,
    pub f_148: f32,
    /// +0x14c / +0x150 references released by the slot-4 message
    pub f_14c: i32,
    pub f_150: i32,
    /// +0x40 bit 1 (both the tagged-pointer and the pointed word variants)
    pub f_40: u32,
}

impl Default for C2ceb78 {
    fn default() -> Self {
        C2ceb78 {
            f_e0: 0,
            tpl: Template { w: ee_sub_159118(), block: [0; 5] },
            bx: [0.0; 8],
            f_f0: 0,
            f_f4: 0,
            f_f8: 0.0,
            f_fc: 0.0,
            f_100: 0,
            f_104: 0.0,
            f_108: 0.0,
            f_10c: 0.0,
            f_110: 0.0,
            f_114: 0,
            f_118: 0,
            f_11c: Vec::new(),
            f_130: 0.0,
            f_134: 0.0,
            f_138: 0.0,
            f_13c: 0.0,
            f_140: 0.0,
            f_144: 0.0,
            f_148: 0.0,
            f_14c: 0,
            f_150: 0,
            f_40: 0,
        }
    }
}

/// slot 5: FUN_00113be0
pub fn ee_sub_113be0() -> u32 {
    0x2bad2c
}

/// FUN_00159118: fresh template = {1, -1, 0, 0} + the 0xb0-byte default DAT_002de5b0 (built by FUN_0015cad8).
pub fn ee_sub_159118() -> [u32; 64] {
    let mut w = [0u32; 64];
    w[0] = 1;
    w[1] = u32::MAX;
    w[8] = u32::MAX; // +0x20 source oid
    w[16] = 1.0f32.to_bits(); // +0x40 = DAT_002a2da0 (1, 0, 0, 0)
    w[20] = 750.0f32.to_bits();
    w[21] = 750.0f32.to_bits();
    w[22] = (-1.0f32).to_bits();
    w[23] = 1.0f32.to_bits();
    w[26] = (-1500.0f32).to_bits();
    w[30] = 1.0f32.to_bits(); // +0x70.. = DAT_002a2dc0 (0, 0, 1, 0)
    w[33] = 200;
    w[34] = 1.0f32.to_bits();
    w[35] = 1.0f32.to_bits();
    w[44] = 1.0f32.to_bits();
    w[46] = (-1.0f32).to_bits();
    w
}

/// What slot 14 asks the owner set to do for a +0x118 group (FUN_0015e938 kind 5 / FUN_0013e3c8 class 0x1d).
#[derive(Clone, Debug, PartialEq)]
pub enum Group {
    None,
    /// join group `oid` (create it when no object has that oid)
    Join(i32),
}

fn word(b: &[u8], i: usize) -> Option<u32> {
    b.get(i * 4..i * 4 + 4).map(|x| u32::from_le_bytes([x[0], x[1], x[2], x[3]]))
}

impl C2ceb78 {
    /// Load-time field writes (field table 0x2ac880, class 29): 469 +0xf4 (setter 0x15a240), 470 +0x114,
    /// 471 +0xf0, 472 +0xf8 (2 floats), 473 +0x110, 474 +0x104, 475 +0x108 (2 floats), 476 +0x100, 477 +0x118;
    /// template (setter 0x258658): 488 +0x20, 490 +0x14, 491-508 at their member offsets; 484 / 487 template +0x30
    /// (setters 0x258ac8 / 0x258b38), 485 / 486 box +0 / +0x10 (setter 0x258b00). `kind` = template +0x10 (load).
    pub fn feed(&mut self, props: &[Prop], kind: i32) {
        self.tpl.w[4] = kind as u32;
        for p in props {
            let PropValue::Scalar(b) = &p.value else { continue };
            let n = b.len() / 4;
            let ws = |o: usize, w: &mut [u32; 64]| {
                for k in 0..n {
                    if o / 4 + k < 64 {
                        w[o / 4 + k] = word(b, k).unwrap();
                    }
                }
            };
            let fl = |k: usize| f32::from_bits(word(b, k).unwrap_or(0));
            match p.field {
                469 => {
                    // FUN_0015a240
                    let v = word(b, 0).unwrap_or(0) as i32;
                    if self.f_f0 == -1 {
                        self.f_f4 = v;
                    } else if self.f_f0 == 0 {
                        if v == 0 {
                            self.f_148 = FLT_MAX; // FUN_0015a150
                        } else if v != -1 {
                            self.f_f4 = v;
                        }
                    }
                    self.f_e0 = 1;
                }
                470 => self.f_114 = b.first().copied().unwrap_or(0) as i32,
                471 => self.f_f0 = word(b, 0).unwrap_or(0) as i32,
                472 => {
                    self.f_f8 = fl(0);
                    self.f_fc = fl(1);
                }
                473 => self.f_110 = fl(0),
                474 => self.f_104 = fl(0),
                475 => {
                    self.f_108 = fl(0);
                    self.f_10c = fl(1);
                }
                476 => self.f_100 = b.first().copied().unwrap_or(0) as i32,
                477 => self.f_118 = word(b, 0).unwrap_or(0) as i32,
                484 | 487 => ws(0x30, &mut self.tpl.w),
                485 => (0..3).for_each(|k| self.bx[k] = fl(k)),
                486 => (0..3).for_each(|k| self.bx[4 + k] = fl(k)),
                488 => ws(0x20, &mut self.tpl.w),
                490 => ws(0x14, &mut self.tpl.w),
                491..=508 => {
                    let o = [0x40, 0x40, 0x40, 0x24, 0x18, 0x50, 0x5c, 0x60, 0x58, 0x80, 0x70, 0x90, 0x84, 0x88, 0x8c, 0x98, 0xa0, 0xb0][(p.field - 491) as usize];
                    ws(o, &mut self.tpl.w);
                    self.f_e0 = 1;
                }
                _ => {}
            }
        }
    }

    /// slot 4: FUN_001599a8
    pub fn ee_sub_1599a8(&mut self, obj: usize, msg: i32, p: i32, h: &mut dyn ClassHost) {
        if msg == 1 {
            if p == self.f_14c {
                h.ee_sub_133448(p, obj);
                self.f_14c = 0;
            } else if p == self.f_150 {
                h.ee_sub_133448(p, obj);
                self.f_150 = 0;
            }
        }
    }

    /// slot 6: FUN_001591b0
    pub fn ee_sub_1591b0(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x231100, obj);
        self.f_118 = -1;
        self.f_f4 = -1;
        self.f_f0 = -1;
        self.f_f8 = 10.0;
        self.f_148 = -1.0;
        self.f_fc = 10.0;
        self.f_11c.clear(); // FUN_0011a1f8(+0x11c, 0x128)
        h.ee_sub_1a6ca8(obj, 0);
        self.f_40 |= 2;
    }

    /// slot 40: FUN_00258620
    pub fn ee_sub_258620(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x2311b0, obj);
        self.tpl.w[0] = self.tpl.w[0].wrapping_add(1);
    }

    /// FUN_00258658(obj, 1): copy-on-write the shared template
    pub fn ee_sub_258658(&mut self) {
        self.f_e0 = 1;
        if (self.tpl.w[0] as i32) < 2 {
            return;
        }
        // the old copy keeps refcount - 1; this object now owns a private copy with refcount 1
        self.tpl.w[0] = 1;
    }

    /// FUN_002586d8 (slot-14 base): resolve the template block object references.
    pub fn ee_sub_2586d8(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1a5c48, obj);
        let t = &self.tpl.w;
        let a = t[8] as i32; // +0x20
        let b = if t[4] == 2 { t[12] as i32 } else { -1 }; // +0x30
        let mut blk = [0u32; 5];
        if a != -1 {
            blk[0] = h.ee_sub_15e938(4, a, obj as u32 + 1);
        }
        if b != -1 {
            blk[1] = h.ee_sub_15e938(4, b, obj as u32 + 1);
            blk[0] = 0;
        }
        if t[1] == 0 {
            if t[53] as i32 != -1 {
                blk[2] = h.ee_sub_15e938(4, t[53] as i32, obj as u32 + 1);
            }
        } else if t[1] == 1 {
            let (c, d, e) = (t[50] as i32, t[51] as i32, t[52] as i32); // +0xc8 / +0xcc / +0xd0
            if c != -1 {
                blk[2] = h.ee_sub_15e938(4, c, obj as u32 + 1);
            }
            if d != -1 {
                blk[3] = h.ee_sub_15e938(4, d, obj as u32 + 1);
            }
            if e != -1 {
                blk[4] = h.ee_sub_15e938(4, e, obj as u32 + 1);
            }
        }
        // FUN_00233178 compare with the current block (DAT_002ce4a0 zeros when none)
        if blk != self.tpl.block {
            self.ee_sub_258658();
            self.tpl.block = blk;
        }
    }

    /// slot 14: FUN_00159500 (load post). Returns the group request for +0x118.
    pub fn ee_sub_159500(&mut self, obj: usize, rng: &mut GameRng, g: &ClassGlobals, h: &mut dyn ClassHost) -> Group {
        self.ee_sub_2586d8(obj, h);
        if self.tpl.block[0] != 0 {
            h.ee_sub_149a50(obj, 0x159968, 0); // FUN_00149a50(0x3dd490, obj, DAT_002a5140, 0)
        }
        let mut m = self.f_f0;
        if m == -1 {
            self.f_f0 = if self.f_104 == 0.0 && self.f_114 == 0 { (self.f_110 != 0.0) as i32 } else { 3 };
            m = self.f_f0;
        }
        let tail = if m == 2 {
            self.f_138 = FLT_MAX;
            self.f_130 = rng.FUN_0011d448(self.f_f8, self.f_fc);
            true
        } else if m < 3 {
            if m >= 0 {
                self.f_134 = 1.0;
                self.f_138 = 1.0;
                self.f_130 = rng.FUN_0011d448(self.f_f8, self.f_fc);
            }
            false
        } else if m == 3 {
            self.f_138 = FLT_MAX;
            self.f_130 = rng.FUN_0011d448(self.f_f8, self.f_fc) * 60.0;
            true
        } else {
            false
        };
        if tail {
            if self.f_114 == 0 {
                if rng.FUN_0011d448(0.0, 1.0) <= self.f_104 {
                    let v = rng.FUN_0011d448(self.f_108, self.f_10c);
                    self.f_148 = g.t + v; // FUN_0015a138
                }
            } else {
                self.f_148 = FLT_MAX; // FUN_0015a150
            }
        }
        // LAB_001596b0
        self.f_13c = 1.0;
        if self.tpl.w[4] == 1 {
            let p = self.bx;
            if self.tpl.block[0] != 0 {
                let mut a = (p[4] - p[0]).abs();
                let b = (p[5] - p[1]).abs();
                let c = (p[6] - p[2]).abs();
                if (a <= b || a <= c) && {
                    a = b;
                    b <= c
                } {
                    a = c;
                }
                self.f_140 = a * 0.5;
            }
            if self.f_100 != 0 {
                let (k, eps) = if self.f_110 == 0.0 { (p[5] - p[1], 0.0001) } else { ((p[4] - p[0]) * (p[5] - p[1]), 1e-6) };
                self.f_13c = k * (p[6] - p[2]) * eps;
            }
        }
        if self.f_118 != -1 {
            self.f_40 &= !2;
            return Group::Join(self.f_118);
        }
        Group::None
    }

    /// The new group object FUN_00159500 creates (FUN_0013e3c8(0x1d, 0, oid, -1)) when no object has the oid:
    /// fresh refcounted template copy (block reset when the source has one), copied +0xf0..+0x114.
    pub fn new_group(&self, h: &mut dyn ClassHost, obj: usize, newobj: usize) -> C2ceb78 {
        let mut n = C2ceb78::default();
        n.ee_sub_1591b0(newobj, h);
        n.tpl = self.tpl.clone();
        n.tpl.w[0] = 1;
        n.bx = self.bx;
        if self.tpl.block[0] != 0 {
            n.tpl.block = [0; 5];
            n.tpl.w[8] = u32::MAX; // +0x20 = -1
        }
        n.f_f0 = self.f_f0;
        n.f_f4 = self.f_f4;
        n.f_f8 = self.f_f8;
        n.f_fc = self.f_fc;
        n.f_100 = self.f_100;
        n.f_104 = self.f_104;
        n.f_108 = self.f_108;
        n.f_10c = self.f_10c;
        n.f_110 = self.f_110;
        n.f_114 = self.f_114;
        h.ee_sub_1a6ca8(newobj, 1);
        // (vtable +0xf4)(new + adj, FUN_001a6318(obj)): this member's frame (ANIM-ALL R92: was the block source)
        h.vcall_f4_1a6318(newobj, obj as u32 + 1);
        n
    }

    /// FUN_0015a1c0: template changed -> push it to the referenced particle systems, re-pick the rate.
    pub fn ee_sub_15a1c0(&mut self, rng: &mut GameRng, h: &mut dyn ClassHost) {
        self.f_e0 = 0;
        if self.f_150 != 0 {
            h.ee_sub_1394c8(self.f_150, &self.tpl);
        }
        if self.f_14c != 0 {
            h.ee_sub_1cde70(self.f_14c, &self.tpl);
        }
        let v = rng.FUN_0011d448(self.f_f8, self.f_fc);
        self.f_130 = if self.f_f0 == 3 { v * 60.0 } else { v };
    }

    /// FUN_0015a170: 1 = do not emit this frame
    pub fn ee_sub_15a170(&self, g: &ClassGlobals) -> bool {
        if self.f_148 <= g.t {
            return (0..2).contains(&self.f_f0) && self.f_f4 == 0;
        }
        true
    }

    fn rearm(&mut self, rng: &mut GameRng, g: &ClassGlobals) {
        if self.f_114 == 0 {
            if rng.FUN_0011d448(0.0, 1.0) <= self.f_104 {
                let v = rng.FUN_0011d448(self.f_108, self.f_10c);
                self.f_148 = g.t + v;
            }
        } else {
            self.f_148 = FLT_MAX;
        }
    }

    fn take(&mut self, n: i32) -> i32 {
        let b = self.f_f4;
        let mut n = n;
        if b > -1 {
            if b <= n {
                n = b;
            }
            self.f_f4 = b - n;
        }
        n
    }

    /// slot 15: FUN_00159c80 (per-frame update)
    pub fn ee_sub_159c80(&mut self, obj: usize, rng: &mut GameRng, g: &ClassGlobals, h: &mut dyn ClassHost) {
        if self.f_e0 != 0 {
            self.ee_sub_15a1c0(rng, h);
        }
        h.ee_sub_1a5e28(obj);
        if self.f_118 != -1 {
            return;
        }
        let me = obj as u32 + 1;
        if !self.f_11c.is_empty() {
            let mut go = false;
            for &m in &self.f_11c {
                if !h.ee_sub_1a61a0(m) {
                    continue;
                }
                if h.obj_i32(m, 0x10) == 0 {
                    go = true;
                    break;
                }
                let r = h.obj_f32(m, 0x38) * g.g_2ddf5c;
                let p = h.ee_sub_1a62e8(m);
                let d = [p[0] - g.g_2ddfd0[0], p[1] - g.g_2ddfd0[1], p[2] - g.g_2ddfd0[2]];
                if d[0] * d[0] + d[1] * d[1] + d[2] * d[2] < r * r {
                    go = true;
                    break;
                }
            }
            if !go {
                return;
            }
        } else if h.obj_i32(me, 0x10) != 0 {
            let r = h.obj_f32(me, 0x38) * g.g_2ddf5c;
            let p = h.ee_sub_1a62e8(me);
            let d = [p[0] - g.g_2ddfd0[0], p[1] - g.g_2ddfd0[1], p[2] - g.g_2ddfd0[2]];
            if r * r < d[0] * d[0] + d[1] * d[1] + d[2] * d[2] {
                return;
            }
        }
        if self.ee_sub_15a170(g) {
            return;
        }
        let mut n = 0i32;
        let mut emit = true;
        match self.f_f0 {
            1 => {
                let live = if self.tpl.w[1] == 0 {
                    if self.f_150 != 0 { h.obj_i32(self.f_150 as u32, 0x444) } else { 0 }
                } else if self.f_14c != 0 {
                    h.list_len(self.f_14c as u32, 0x15c) // FUN_0011a3c0(+0x14c + 0x15c)
                } else {
                    0
                };
                n = ((self.f_110 * self.f_13c) as i32 - live).max(0);
            }
            0 => {
                let v = self.f_144 + self.f_130 * self.f_13c * g.dt;
                n = v as i32;
                self.f_144 = v - n as f32;
                if n == 0 {
                    emit = false;
                } else {
                    n = self.take(n);
                }
            }
            2 => {
                n = self.take(self.f_130 as i32);
                self.rearm(rng, g);
                self.f_130 = rng.FUN_0011d448(self.f_f8, self.f_fc);
            }
            3 => {
                let v = self.f_144 + self.f_130 * self.f_13c * g.dt;
                n = v as i32;
                self.f_144 = v - n as f32;
                if 1.0 < n as f32 {
                    n = self.take(n);
                    // quirk: the f13 argument of this FUN_0011d448(0, ..) call is not reloaded; (0, 1) assumed
                    let skip = self.f_114 == 0 && self.f_104 < rng.FUN_0011d448(0.0, 1.0);
                    if !skip {
                        if self.f_114 == 0 {
                            let v = rng.FUN_0011d448(self.f_108, self.f_10c);
                            self.f_148 = g.t + v;
                        } else {
                            self.f_148 = FLT_MAX;
                        }
                        self.f_130 = rng.FUN_0011d448(self.f_f8, self.f_fc) * 60.0;
                    }
                }
            }
            _ => {}
        }
        if emit && n != 0 {
            // ctx {&+0x150, &+0x14c, obj}; the block's source is this object while it has none
            let keep = self.tpl.block[0];
            if keep == 0 && self.tpl.block[1] == 0 {
                self.tpl.block[0] = me;
            }
            h.ee_sub_15bd20(obj, n, &self.tpl.w, &self.tpl.block, true);
            self.tpl.block[0] = keep;
        }
        // LAB_0015a0d4
        if self.f_138 <= g.t {
            self.f_130 = rng.FUN_0011d448(self.f_f8, self.f_fc);
            self.f_138 = g.t + self.f_134;
        }
        self.ee_sub_159a20(me, h);
    }

    /// FUN_00159a20: wrap the +0x150 system's particles into the cube of half-size +0x140 around the source
    /// (or the origin when the block has no source). The particle walk is `ClassHost::wrap_particles`.
    pub fn ee_sub_159a20(&mut self, me: u32, h: &mut dyn ClassHost) {
        let r = self.f_140;
        if r <= 0.0001 {
            return;
        }
        let (lo, hi) = if self.tpl.block[0] == 0 {
            ([-r, -r, -r], [r, r, r])
        } else {
            // quirk: lo = pos(obj) - r (FUN_001a62e8 with this object), hi = pos(source) + r
            let a = h.ee_sub_1a62e8(me);
            let b = h.ee_sub_1a62e8(self.tpl.block[0]);
            ([a[0] - r, a[1] - r, a[2] - r], [b[0] + r, b[1] + r, b[2] + r])
        };
        if self.tpl.w[1] == 0 && self.f_150 != 0 {
            h.wrap_particles(self.f_150, lo, hi, r + r); // FUN_0011d898 = fmod-style wrap per axis
        }
    }
}
