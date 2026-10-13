//! C2db0d0 (world class 105): motion-trail SO (parent C2d4260). Keeps a ring of the object's past world
//! matrices (+0x15c, 0x50-byte entries) and re-skins its mesh (+0xdc groups) through that history each draw.
//! Object handles in hooks are index + 1 (0 = null).
use super::k4::{f32_of, vec3_of, word};
use super::{ClassGlobals, ClassHost};
use crate::engine::anim::pose::Mat4;
use crate::engine::world::props::{Prop, PropValue};

/// One sub-mesh of a +0xdc group (0x20-byte record): +0 n, +0xc vertex buffer (vec4 stride 0x10),
/// +0x10 u16 stream (stride 6), +0x14 colours (u32), +0x18 u32 stream, +0x1c per-vertex record (0x20 bytes).
#[derive(Clone, Debug, Default)]
pub struct Sub105 {
    pub n: i32,
    pub verts: Vec<[f32; 4]>,
    pub f_10: Option<Vec<[u8; 6]>>,
    pub f_14: Option<Vec<u32>>,
    pub f_18: Option<Vec<u32>>,
    /// +0x1c records: {rest position vec4, phase f32, +0x18 word, +0x14 word, pad} as raw words
    pub recs: Vec<[u32; 8]>,
}

/// One +0xdc list node: +0 entry of the +0xbc table (index = (p - +0xbc) / 0x14), +4 count, +8 subs, +0xc next.
#[derive(Clone, Debug, Default)]
pub struct Group105 {
    pub idx: usize,
    pub subs: Vec<Sub105>,
}

#[derive(Clone, Debug)]
pub struct C2db0d0 {
    /// +0x12c alpha (field 57), +0x130 alpha rate (field 58)
    pub f_12c: f32,
    pub f_130: f32,
    /// +0x150 trail duration (1304), +0x154 frozen (1306), +0x158 ring length (1305)
    pub f_150: f32,
    pub f_154: bool,
    pub f_158: i32,
    /// +0x15c ring (empty = null): {t, alpha, distance, -, world matrix 4x4}
    pub f_15c: Vec<[f32; 20]>,
    /// +0x160 ring head, +0x164 frames with alpha <= 0
    pub f_160: i32,
    pub f_164: i32,
    /// +0x168 stretch (1308), +0x170 axis (1307)
    pub f_168: f32,
    pub f_170: [f32; 4],
    /// +0x180 distance target (1309, handle), +0x184 distance scale (1310), +0x188 distance, +0x190 last target position
    pub f_180: u32,
    pub f_184: f32,
    pub f_188: f32,
    pub f_190: [f32; 4],
    /// +0x40 bit 1
    pub f_40: u32,
    /// +0xdc mesh groups (geometry from the host)
    pub groups: Vec<Group105>,
    /// +0xc4 table (0x18-byte entries, xyz written by FUN_0021c5f8), indexed like +0xbc
    pub f_c4: Vec<[f32; 3]>,
}

impl Default for C2db0d0 {
    fn default() -> Self {
        C2db0d0 {
            f_12c: 0.0,
            f_130: 0.0,
            f_150: 0.0,
            f_154: false,
            f_158: 0,
            f_15c: Vec::new(),
            f_160: 0,
            f_164: 0,
            f_168: 0.0,
            f_170: [0.0; 4],
            f_180: 0,
            f_184: 0.0,
            f_188: 0.0,
            f_190: [0.0; 4],
            f_40: 0,
            groups: Vec::new(),
            f_c4: Vec::new(),
        }
    }
}

/// slot 5: FUN_001140a0
pub fn ee_sub_1140a0() -> u32 {
    0x2bb31c
}

fn dot3(a: &[f32; 4], b: &[f32; 4]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn flat(m: &Mat4) -> [f32; 16] {
    let mut r = [0.0; 16];
    for i in 0..4 {
        r[i * 4..i * 4 + 4].copy_from_slice(&m[i]);
    }
    r
}

/// row-vector transform of (p.xyz, 1) by the matrix stored at e[4..20]
fn xf(e: &[f32; 20], p: &[f32; 4]) -> [f32; 4] {
    let mut o = [0.0; 4];
    for k in 0..4 {
        o[k] = e[4 + k] * p[0] + e[8 + k] * p[1] + e[12 + k] * p[2] + e[16 + k];
    }
    o
}

impl C2db0d0 {
    /// Load fields 57, 58, 1304-1310 (written after slot 6).
    pub fn feed(&mut self, obj: usize, props: &[Prop], h: &mut dyn ClassHost) {
        for p in props {
            let PropValue::Scalar(b) = &p.value else { continue };
            match p.field {
                57 => self.f_12c = f32_of(b),
                58 => self.f_130 = f32_of(b),
                1304 => self.f_150 = f32_of(b),
                1305 => self.f_158 = word(b) as i32,
                1306 => self.f_154 = word(b) != 0,
                1307 => self.f_170 = vec3_of(b),
                1308 => self.f_168 = f32_of(b),
                1309 => self.f_180 = h.ee_sub_15e938(0, word(b) as i32, obj as u32 + 1),
                1310 => self.f_184 = f32_of(b),
                _ => {}
            }
        }
    }

    /// slot 6: FUN_0021bb88
    pub fn ee_sub_21bb88(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1f2108, obj);
        self.f_170 = [1.0, 0.0, 0.0, self.f_170[3]];
        self.f_184 = 100.0;
    }

    /// slot 10: FUN_0021bbd0
    pub fn ee_sub_21bbd0(&mut self, obj: usize, g: &ClassGlobals, h: &mut dyn ClassHost) {
        h.base(0x1f2c30, obj);
        if !self.f_15c.is_empty() {
            self.ee_sub_21bc08(obj, g, h);
        }
    }

    /// FUN_0021bc08: fill the whole ring with the current world matrix, times t - n/60 .. t - 1/60
    pub fn ee_sub_21bc08(&mut self, obj: usize, g: &ClassGlobals, h: &mut dyn ClassHost) {
        let mut t = g.t;
        let m = flat(&h.vcall_fc(obj as u32 + 1));
        let n = self.f_158;
        assert!(n != 0, "trap(7): FUN_0021bc08 with +0x158 == 0");
        t -= n as f32 * 0.016666668;
        let mut k = n - 1;
        while k >= 0 {
            k -= 1;
            let e = &mut self.f_15c[self.f_160 as usize];
            self.f_160 = (self.f_160 + 1) % self.f_158;
            e[0] = t;
            e[1] = 0.0;
            t += 0.016666668;
            e[4..20].copy_from_slice(&m);
        }
        self.f_164 = 0;
    }

    /// slot 14: FUN_0021bd18
    pub fn ee_sub_21bd18(&mut self, obj: usize, g: &ClassGlobals, h: &mut dyn ClassHost) {
        h.base(0x1f36a8, obj);
        if self.f_158 == 0 && self.f_150 == 0.0 {
            self.f_150 = 0.5;
        }
        let f = self.f_150;
        if f == 0.0 {
            self.f_150 = (self.f_158 - 2) as f32 * 0.016666668;
        } else {
            self.f_158 = (f * 60.0 + 0.5) as i32 + 2;
        }
        // FUN_001ae2a0({n * 0x50, 0x10})
        self.f_15c = vec![[0.0; 20]; self.f_158.max(0) as usize];
        self.groups = h.mesh_dc(obj as u32 + 1);
        let (mut hi, mut lo) = (f32::from_bits(0xff7fffff), f32::from_bits(0x7f7fffff));
        for gr in &self.groups {
            for s in &gr.subs {
                for v in s.verts.iter().take(s.n.max(0) as usize) {
                    let d = dot3(v, &self.f_170);
                    if hi < d {
                        hi = d;
                    }
                    if d < lo {
                        lo = d;
                    }
                }
            }
        }
        for gr in &mut self.groups {
            h.set_mesh_cb(obj as u32 + 1, gr.idx, 0x2c44f0);
            h.ee_sub_16ef00(obj as u32 + 1, gr.idx);
            for s in &mut gr.subs {
                let n = s.n.max(0) as usize;
                s.recs = vec![[0; 8]; n];
                for i in 0..n {
                    let v = s.verts.get(i).copied().unwrap_or([0.0; 4]);
                    let d = dot3(&v, &self.f_170);
                    let k = (self.f_168 - 1.0) * d;
                    let r = &mut s.recs[i];
                    for c in 0..4 {
                        r[c] = (v[c] + self.f_170[c] * k).to_bits();
                    }
                    r[4] = ((d - lo) / (hi - lo)).to_bits();
                    if let Some(x) = &s.f_18 {
                        r[5] = x.get(i).copied().unwrap_or(0);
                    }
                    if let Some(x) = &s.f_14 {
                        r[6] = x.get(i).copied().unwrap_or(0);
                    }
                }
            }
        }
        self.ee_sub_21bc08(obj, g, h);
        if !self.f_154 {
            self.f_40 |= 2;
        }
    }

    /// slot 15: FUN_0021c0d8
    pub fn ee_sub_21c0d8(&mut self, obj: usize, g: &ClassGlobals, h: &mut dyn ClassHost) {
        h.ee_sub_1a5e28(obj);
        if self.f_130 != 0.0 {
            let f = self.f_12c + g.dt * self.f_130;
            self.f_12c = f;
            if f <= 0.0 {
                self.f_130 = 0.0;
                self.f_12c = 0.0;
            } else if f >= 1.0 {
                self.f_12c = 1.0;
                self.f_130 = 0.0;
            }
        }
        if self.f_12c <= 0.0 && self.f_130 <= 0.0 && self.f_158 <= self.f_164 {
            h.vcall(obj as u32 + 1, 0x4c);
        }
    }

    /// slot 16: FUN_0021c1c8
    pub fn ee_sub_21c1c8(&mut self, obj: usize, g: &ClassGlobals, h: &mut dyn ClassHost) {
        h.base(0x1f3980, obj);
        if self.f_154 {
            return;
        }
        assert!(self.f_158 != 0, "trap(7): FUN_0021c1c8 with +0x158 == 0");
        let k = self.f_160 as usize;
        self.f_160 = (self.f_160 + 1) % self.f_158;
        let m = flat(&h.vcall_fc(obj as u32 + 1));
        {
            let e = &mut self.f_15c[k];
            e[0] = g.t;
            e[1] = self.f_12c;
            e[4..20].copy_from_slice(&m);
        }
        if self.f_12c <= 0.0 {
            self.f_164 += 1;
        } else {
            self.f_164 = 0;
        }
        if self.f_180 != 0 {
            if self.f_12c == 0.0 {
                self.f_188 = -4.0;
            }
            let p = h.ee_sub_1a62e8(self.f_180);
            let d = [self.f_190[0] - p[0], self.f_190[1] - p[1], self.f_190[2] - p[2]];
            self.f_190 = [p[0], p[1], p[2], 0.0];
            let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            self.f_188 += l / self.f_184;
            self.f_15c[k][2] = self.f_188;
        }
        let p = h.ee_sub_1a62e8(obj as u32 + 1);
        self.ee_sub_21c5f8(p);
    }

    /// FUN_0021c5f8(obj, pos): +0xc4[group] = pos.xyz for every +0xdc group
    pub fn ee_sub_21c5f8(&mut self, p: [f32; 3]) {
        for gr in &self.groups {
            if self.f_c4.len() <= gr.idx {
                self.f_c4.resize(gr.idx + 1, [0.0; 3]);
            }
            self.f_c4[gr.idx] = p;
        }
    }

    /// slot 18: FUN_0021c340 (re-skin through the ring, then the SO draw FUN_001f3c70 at alpha 1)
    pub fn ee_sub_21c340(&mut self, obj: usize, g: &ClassGlobals, h: &mut dyn ClassHost) {
        let n = self.f_158;
        for gr in &mut self.groups {
            for s in &mut gr.subs {
                let mut c = 0usize;
                let mut u = 0usize;
                for i in 0..s.n.max(0) as usize {
                    assert!(n != 0, "trap(7): FUN_0021c340 with +0x158 == 0");
                    let r = s.recs[i];
                    let ph = f32::from_bits(r[4]) * (n - 2) as f32;
                    let ip = ph as i32;
                    let a = (((self.f_160 + n) - ip) - 1) % n;
                    let fr = ph - ip as f32;
                    let pos = [f32::from_bits(r[0]), f32::from_bits(r[1]), f32::from_bits(r[2]), f32::from_bits(r[3])];
                    let e0 = &self.f_15c[a as usize];
                    let b = ((a + n - 1) % n) as usize;
                    let e1 = &self.f_15c[b];
                    let p0 = xf(e0, &pos);
                    let p1 = xf(e1, &pos);
                    if let Some(v) = s.verts.get_mut(i) {
                        for k in 0..3 {
                            v[k] = p0[k] * (1.0 - fr) + p1[k] * fr;
                        }
                    }
                    if let Some(cols) = &mut s.f_14 {
                        let mut f = 1.0 - (g.t - e0[0]) / self.f_150;
                        if f < 0.0 {
                            f = 0.0;
                        } else if 1.0 < f {
                            f = 1.0;
                        }
                        let al = (e0[1] * f * (r[6] >> 24) as u8 as f32) as i32 as u8;
                        if let Some(x) = cols.get_mut(c) {
                            *x = *x & 0x00ff_ffff | (al as u32) << 24;
                        }
                        c += 1;
                    }
                    if let Some(st) = &mut s.f_10 {
                        if self.f_180 != 0 {
                            let v = (((1.0 - fr) * e0[2] + fr * e1[2]) * 4096.0) as i32;
                            if let Some(x) = st.get_mut(u) {
                                x[0] = v as u8;
                                x[1] = (v as u32 >> 8) as u8;
                            }
                            u += 1;
                        }
                    }
                }
            }
        }
        let a = self.f_12c;
        self.f_12c = 1.0;
        h.so_draw_1f3c70(obj, self.f_12c);
        self.f_12c = a;
    }
}
