//! WP-K22: C2dcf00 (world class 115, parent C2c94e0 / LO, Sly 1 `WR`): the warp object that bends / waves
//! the vertices of the globs registered on it. Up to 8 warp records (+0x60, 0xd0 each, count +0x54, current
//! +0x6e0): kind 0 "circle" (FUN_0022c3b0, two per matrix: rows 0-1 / 2-3) and kind 1 "bend" (FUN_0022c430,
//! noise-driven rotation). Per frame (slot 15 FUN_0022c028) each record writes its matrix (+0x6f0 + idx * 0x40)
//! and circle offsets (+0x7f0 + idx * 0x40); a dirty warp (+0x8fc) first re-bakes the per-vertex weights /
//! phases of its globs (FUN_0022b7d8).
//! Sly 1 names: InitWr 22b720, ApplyWrGlob 22b7a0, UpdateWrStateVectors 22b7d8, UpdateWrMatrixes 22c028,
//! AddWrCircleWarp 22c3b0, AddWrBendWarp 22c430, PwreGetWrCur 22c4a0 (+ Circle 22c4d8 / Bend 22c4f8),
//! SetWrWavelength 22c518, GetWrWavelength 22c570, AddWrBendNoise 22c5f0 / 22c658 (FUN_0022c5b8).
//! 1:1: slots 5 FUN_00114140, 6 FUN_0022b720, 15 FUN_0022c028, the loader fields 945..964 and the noise
//! FUN_0022bf70 / 22bf40 / 22be60 / 22be08. The records are kept as their raw 0xd0 bytes (52 words).
//! UNPORTED: FUN_00167b68 (the model-side glob binding that calls ApplyWrGlob) - no glob is ever registered in
//! the running game, and nothing in the renderer consumes the warp matrices; FUN_0022b7d8 is ported over the
//! `WrGlob` model below (tested directly). Its `+8 != 0` branches (FUN_0011c168 with an argument lost in the
//! decompile) use the matrix unmultiplied.
//! Note: a bend record with no noise term divides 0 / 0 (NaN here; the PS2 FPU gives no NaN).
use super::ClassHost;
use crate::engine::anim::motion::ee_sub_11d608;
use crate::engine::anim::pose::{ee_sub_11c318, Mat4, IDENTITY};
use crate::engine::physics::driver::FUN_0011b500;
use crate::engine::player::p5_attach::FUN_0011cf60;

/// one warp record: the raw 0xd0 bytes
pub type Rec = [f32; 52];

/// DAT_002a2da0 / 2db0 / 2dc0: the x / y / z axes; DAT_002a2dd0: zero
const AX: [[f32; 4]; 3] = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0]];
const TAU: f32 = 6.2831855;

/// slot 5: FUN_00114140
pub fn ee_sub_114140() -> u32 {
    0x2bb3e4
}

fn ri(r: &Rec, k: usize) -> i32 {
    r[k].to_bits() as i32
}
fn set_i(r: &mut Rec, k: usize, v: i32) {
    r[k] = f32::from_bits(v as u32);
}

/// FUN_0022be08: integer hash -> [0, 1)
pub fn ee_sub_22be08(x: u32) -> f32 {
    let x = (x << 13) ^ x;
    (x.wrapping_mul(x.wrapping_mul(x).wrapping_mul(0x3d73).wrapping_add(0xc0ae5)).wrapping_add(0x5208dd0d) & 0x7fff_ffff)
        as f32
        * 4.656613e-10
}

/// FUN_0022be60(freq, phase, amp): smoothstep value noise at 2 * freq * now + phase; even lattice points are
/// pulled to 1 - amp * (1 - h), odd ones to amp * h
pub fn ee_sub_22be60(freq: f32, phase: f32, amp: f32, now: f32) -> f32 {
    let t = (freq + freq) * now + phase;
    let u = t as u32;
    let mut a = ee_sub_22be08(u);
    let mut b = ee_sub_22be08(u.wrapping_add(1));
    if u & 1 == 0 {
        b *= amp;
        a = amp * (a - 1.0) + 1.0;
    } else {
        a *= amp;
        b = amp * (b - 1.0) + 1.0;
    }
    let f = t - (u as i32) as f32;
    a + f * f * (3.0 - (f + f)) * (b - a)
}

/// FUN_0022bf40(x, bias): Schlick bias
pub fn ee_sub_22bf40(x: f32, b: f32) -> f32 {
    x / ((1.0 / b - 2.0) * (1.0 - x) + 1.0)
}

/// FUN_0022bf70(noise): `n` = {min, max, bias, count, 4 x {weight, freq, phase, amp}} -> weighted mean of the
/// terms, biased, mapped into [min, max]
pub fn ee_sub_22bf70(n: &[f32], now: f32) -> f32 {
    let c = n[3].to_bits() as i32;
    let (mut s, mut wsum) = (0.0f32, 0.0f32);
    for i in 0..c.max(0) as usize {
        let e = &n[4 + 4 * i..8 + 4 * i];
        let v = ee_sub_22be60(e[1], e[2], e[3], now);
        wsum += e[0];
        s += e[0] * v;
    }
    let x = ee_sub_22bf40(s / wsum, n[2]);
    n[0] + x * (n[1] - n[0])
}

/// FUN_0022c5b8(a, b, c, d, noise): append a term (at most 4)
pub fn ee_sub_22c5b8(t: [f32; 4], n: &mut [f32]) {
    let c = n[3].to_bits();
    if c < 4 {
        n[3] = f32::from_bits(c + 1);
        n[4 + 4 * c as usize..8 + 4 * c as usize].copy_from_slice(&t);
    }
}

/// one chunk of a registered glob's vertices (+0x24 list): source points, 4 per-matrix output streams
#[derive(Clone, Debug, Default)]
pub struct WrChunk {
    pub verts: Vec<[f32; 4]>,
    pub out: [Vec<[f32; 4]>; 4],
}

/// a glob entry (FUN_00167b68 record, linked at +0x8f0 by ApplyWrGlob)
#[derive(Clone, Debug, Default)]
pub struct WrGlob {
    /// +0 owning ALO (handle for vtable +0xfc)
    pub alo: u32,
    /// +4 -> +4 -> +0x40: glob matrix; +4 -> +8 flag
    pub xf: Mat4,
    pub xf_flag: bool,
    /// +0x30 weight kind (-1 = none), +0x34..+0x40 {x0, y0, x1, y1} weight ramp, +0x50 warp-space matrix
    pub kind: i32,
    pub ramp: [f32; 4],
    pub mat: Mat4,
    pub chunks: Vec<WrChunk>,
}

fn xf(m: &Mat4, v: [f32; 4]) -> [f32; 4] {
    std::array::from_fn(|k| m[0][k] * v[0] + m[1][k] * v[1] + m[2][k] * v[2] + m[3][k])
}
fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}
fn norm3(v: [f32; 3]) -> [f32; 3] {
    let l = len3(v);
    [v[0] / l, v[1] / l, v[2] / l]
}

#[derive(Clone, Debug)]
pub struct C2dcf00 {
    /// +0x40 LO flags (inline form)
    pub f_40: u32,
    /// +0x50 last update time, +0x54 records, +0x58 matrices used, +0x5c open circle matrix, +0x6e0 current
    pub f_50: f32,
    pub n: i32,
    pub f_58: i32,
    pub f_5c: i32,
    pub cur: i32,
    /// +0x60
    pub recs: [Rec; 8],
    /// +0x6f0 warp matrices, +0x7f0 circle offsets (rows: 2 floats used)
    pub mats: [Mat4; 4],
    pub offs: [Mat4; 4],
    /// +0x8f0 registered globs
    pub globs: Vec<WrGlob>,
    /// +0x8fc dirty
    pub dirty: bool,
}

impl Default for C2dcf00 {
    fn default() -> Self {
        C2dcf00 {
            f_40: 0,
            f_50: 0.0,
            n: 0,
            f_58: 0,
            f_5c: 0,
            cur: 0,
            recs: [[0.0; 52]; 8],
            mats: [[[0.0; 4]; 4]; 4],
            offs: [[[0.0; 4]; 4]; 4],
            globs: Vec::new(),
            dirty: false,
        }
    }
}

impl C2dcf00 {
    /// slot 6: FUN_0022b720 (InitWr)
    pub fn ee_sub_22b720(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1a5270, obj);
        self.cur = -1;
        self.f_5c = -1;
        self.f_50 = -1.0;
        self.globs.clear(); // FUN_0011a050(+0x8f0, 0xa0)
        h.ee_sub_1a6ca8(obj, 1);
        self.f_40 |= 2;
    }

    /// FUN_0022b7a0 (ApplyWrGlob): link a glob
    pub fn ee_sub_22b7a0(&mut self, g: WrGlob) {
        self.globs.push(g);
    }

    /// FUN_0022c3b0 (AddWrCircleWarp)
    pub fn ee_sub_22c3b0(&mut self) {
        let Some(r) = self.recs.get_mut(self.n as usize) else { return };
        self.n += 1;
        self.cur += 1;
        set_i(r, 0, 0);
        r[4..8].copy_from_slice(&AX[0]);
        if self.f_5c < 0 {
            set_i(r, 1, self.f_58);
            self.f_58 += 1;
            self.f_5c = ri(r, 1);
            set_i(r, 0x18, 0);
        } else {
            set_i(r, 1, self.f_5c);
            self.f_5c = -1;
            set_i(r, 0x18, 2);
        }
    }

    /// FUN_0022c430 (AddWrBendWarp)
    pub fn ee_sub_22c430(&mut self) {
        let Some(r) = self.recs.get_mut(self.n as usize) else { return };
        self.n += 1;
        self.cur += 1;
        set_i(r, 0, 1);
        r[4..8].copy_from_slice(&AX[1]);
        r[0x22] = 0.5;
        r[8..12].copy_from_slice(&AX[2]);
        r[0xe] = 0.5;
        self.f_58 += 1;
    }

    /// FUN_0022c4a0 (PwreGetWrCur): the current record (none while empty); a set marks the warp dirty
    pub fn ee_sub_22c4a0(&mut self, set: bool) -> Option<&mut Rec> {
        if self.n == 0 {
            return None;
        }
        if set {
            self.dirty = true;
        }
        self.recs.get_mut(self.cur as usize)
    }

    /// FUN_0022c518 (SetWrWavelength): +0x64 = 1 / w
    pub fn ee_sub_22c518(&mut self, w: f32) {
        if let Some(r) = self.ee_sub_22c4a0(true) {
            r[0x19] = if w == 0.0 { 0.0 } else { 1.0 / w };
        }
    }

    /// FUN_0022c570 (GetWrWavelength)
    pub fn ee_sub_22c570(&mut self) -> f32 {
        match self.ee_sub_22c4a0(true) {
            Some(r) if r[0x19] != 0.0 => 1.0 / r[0x19],
            _ => 0.0,
        }
    }

    /// loader fields: 945 / 946 add a circle / bend record; 958 / 962 add a bend noise term (+0x30 / +0x80,
    /// FUN_0022c5f0 / 22c658); 948 wavelength; the rest are current-record members (word offset = byte / 4).
    pub fn feed(&mut self, props: &[crate::engine::world::props::Prop]) {
        use crate::engine::world::props::{ArrayItem, PropValue};
        let fl = |b: &[u8]| -> Vec<f32> { b.chunks_exact(4).map(|x| f32::from_le_bytes([x[0], x[1], x[2], x[3]])).collect() };
        for p in props {
            match (&p.value, p.field) {
                (_, 945) => self.ee_sub_22c3b0(),
                (_, 946) => self.ee_sub_22c430(),
                (PropValue::Array(es), f @ (958 | 962)) => {
                    let mut t = [0.0f32; 4];
                    for (k, e) in es.iter().take(4).enumerate() {
                        if let ArrayItem::One(_, b) = e {
                            t[k] = fl(b).first().copied().unwrap_or(0.0);
                        }
                    }
                    if let Some(r) = self.ee_sub_22c4a0(true) {
                        ee_sub_22c5b8(t, &mut r[if f == 958 { 0xc } else { 0x20 }..]);
                    }
                }
                (PropValue::Scalar(b), 948) => self.ee_sub_22c518(fl(b).first().copied().unwrap_or(0.0)),
                (PropValue::Scalar(b), f) => {
                    let off = match f {
                        947 | 955 => 0x10,
                        949 => 0x68,
                        950 => 0x6c,
                        951 | 959 => 0x20,
                        952 => 0x40,
                        953 | 956 => 0x30,
                        954 => 0x50,
                        957 => 0x38,
                        960 => 0x80,
                        961 => 0x88,
                        _ => continue,
                    };
                    let v = fl(b);
                    if let Some(r) = self.ee_sub_22c4a0(true) {
                        for (k, x) in v.iter().enumerate() {
                            if let Some(d) = r.get_mut(off / 4 + k) {
                                *d = *x;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// slot 15: FUN_0022c028 (UpdateWrMatrixes), once per game time
    pub fn ee_sub_22c028(&mut self, obj: usize, now: f32, h: &mut dyn ClassHost) {
        h.ee_sub_1a5e28(obj);
        if now == self.f_50 {
            return;
        }
        if self.dirty {
            self.ee_sub_22b7d8(h);
        }
        for i in 0..(self.n.max(0) as usize).min(8) {
            let r = &self.recs[i];
            let Some(idx) = usize::try_from(ri(r, 1)).ok().filter(|&k| k < 4) else { continue };
            match ri(r, 0) {
                0 => {
                    let (s, c) = ee_sub_11d608(FUN_0011cf60((now * r[0x1a] + r[0x1b]) * TAU));
                    let sub = (ri(r, 0x18) as usize).min(2);
                    let v = |k: usize| [r[k], r[k + 1], r[k + 2], r[k + 3]];
                    let (a, b, oa, ob) = (v(8), v(0x10), v(0xc), v(0x14));
                    let m = &mut self.mats[idx];
                    for k in 0..3 {
                        m[sub][k] = a[k] * c + b[k] * s;
                        m[sub + 1][k] = a[k] * -s + b[k] * c;
                    }
                    let o = &mut self.offs[idx];
                    for k in 0..2 {
                        o[sub][k] = oa[k] * c + ob[k] * s;
                        o[sub + 1][k] = oa[k] * -s + ob[k] * c;
                    }
                }
                1 => {
                    let u10 = ee_sub_22bf70(&r[0xc..0x20], now);
                    let u11 = ee_sub_22bf70(&r[0x20..0x34], now);
                    let m1 = FUN_0011b500(norm3([r[8], r[9], r[10]]), u11);
                    let v = norm3([r[4], r[5], r[6]]);
                    let w = norm3(std::array::from_fn(|k| m1[0][k] * v[0] + m1[1][k] * v[1] + m1[2][k] * v[2]));
                    let m2 = FUN_0011b500(w, u10);
                    let m = &mut self.mats[idx];
                    for j in 0..3 {
                        for k in 0..3 {
                            m[j][k] = m2[j][k] - if j == k { 1.0 } else { 0.0 };
                        }
                    }
                }
                _ => {}
            }
        }
        self.f_50 = now;
    }

    /// FUN_0022b7d8 (UpdateWrStateVectors): per registered glob, per vertex: the ramp weight from the vertex in
    /// warp space, then per record either the circle phase (cos, sin) * weight (kind 0) or the weighted offset
    /// from the warp centre (kind 1).
    pub fn ee_sub_22b7d8(&mut self, h: &mut dyn ClassHost) {
        let recs: Vec<Rec> = self.recs[..(self.n.max(0) as usize).min(8)].to_vec();
        for g in &mut self.globs {
            let (m, mut c0, ramp);
            if g.kind == -1 {
                m = IDENTITY;
                c0 = [0.0; 4];
                ramp = (0.0, 0.0, 0.0, 0.0);
            } else {
                m = g.mat;
                let [x0, y0, x1, y1] = g.ramp;
                let lo = if y1 < y0 { y1 } else { y0 };
                let hi = if y0 < y1 { y1 } else { y0 };
                let slope = (y0 - y1) / (x0 - x1);
                ramp = (y0 - slope * x0, slope, lo, hi);
                c0 = match g.kind {
                    4 => AX[0],
                    5 => AX[1],
                    6 => AX[2],
                    _ => [0.0; 4],
                };
                if (4..=6).contains(&g.kind) {
                    let mid = (x0 + x1) * 0.5;
                    c0 = c0.map(|x| x * mid);
                }
                c0 = xf(&ee_sub_11c318(&m), c0);
                // `+8 != 0`: FUN_0011c168(m, +0x50, ?) - argument lost, unmultiplied
            }
            let cp = xf(&h.vcall_fc(g.alo), c0); // FUN_001a6f00(alo, 0, c0)
            for ch in &mut g.chunks {
                let wts: Vec<f32> = ch
                    .verts
                    .iter()
                    .map(|v| {
                        let p = xf(&m, *v);
                        let t = match g.kind {
                            0 => len3([p[0], p[1], p[2]]),
                            1 => len3([p[0], p[1], 0.0]),
                            2 => len3([p[0], 0.0, p[2]]),
                            3 => len3([0.0, p[1], p[2]]),
                            4 => p[0],
                            5 => p[1],
                            6 => p[2],
                            _ => return 1.0,
                        };
                        let f = ramp.0 + t * (ramp.1 + t * 0.0);
                        if ramp.2 <= f {
                            if ramp.3 < f { ramp.3 } else { f }
                        } else {
                            ramp.2
                        }
                    })
                    .collect();
                for r in &recs {
                    let Some(out) = usize::try_from(ri(r, 1)).ok().and_then(|k| ch.out.get_mut(k)) else { continue };
                    out.resize(ch.verts.len(), [0.0; 4]);
                    for (k, v) in ch.verts.iter().enumerate() {
                        let q = xf(&g.xf, *v);
                        match ri(r, 0) {
                            0 => {
                                let d = norm3([r[4], r[5], r[6]]);
                                let x = q[0] * d[0] + q[1] * d[1] + q[2] * d[2];
                                let (s, c) = ee_sub_11d608(FUN_0011cf60(x * r[0x19] * TAU));
                                let sub = (ri(r, 0x18) as usize).min(2);
                                out[k][sub] = c * wts[k];
                                out[k][sub + 1] = s * wts[k];
                            }
                            1 => out[k] = [wts[k] * (q[0] - cp[0]), wts[k] * (q[1] - cp[1]), wts[k] * (q[2] - cp[2]), 1.0],
                            _ => {}
                        }
                    }
                }
            }
        }
        self.dirty = false;
    }
}

impl super::ClassSet {
    pub(super) fn build_k22_115(&mut self, w: &crate::engine::world::World, h: &mut dyn ClassHost) {
        for (i, o) in w.objects.iter().enumerate() {
            if o.class != 115 {
                continue;
            }
            let mut c = Box::<C2dcf00>::default();
            c.ee_sub_22b720(i, h);
            if let Some(p) = w.objects.get(o.clone_of.unwrap_or(i)).and_then(crate::engine::game::fields::object_props) {
                c.feed(p);
            }
            self.k22c.push((i, super::Inst::K22C115(c)));
        }
    }
    pub(super) fn frame_k22_115(&mut self, h: &mut dyn ClassHost) {
        let now = self.g.t;
        for (i, x) in &mut self.k22c {
            if let super::Inst::K22C115(c) = x {
                c.ee_sub_22c028(*i, now, h);
            }
        }
    }
}
