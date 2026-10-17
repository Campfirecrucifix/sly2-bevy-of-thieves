//! WP-K28: the emitter -> class-28 spawn path. The emitters (C2ceb78 slot 15 FUN_00159c80, C2ce4d0 slot 51
//! FUN_0015c690) call FUN_0015bd20(count, template, ctx); the host queues the request (`EmitReq`, with the
//! FUN_0015ad48 source frame) and the class set spawns at the start of the class-28 tick:
//! FUN_0015b7e8 (positions / velocities / birth / death per particle) -> FUN_0015b428 (system of the ctx
//! slot, else a new one FUN_001390d8(0x3dd490) + FUN_001394c8 init + slot 50 parent; 0x2b particles per
//! group, FUN_00138ff8 new groups; spin words).
//! 1:1: FUN_0015ae38 (emission grid), FUN_0015a808 shape kind 0 / direction kinds 0, 1, default + the +0x24
//! offset, FUN_0015af08 / FUN_0011e208 / FUN_0011e2d8 (spherical direction in the dir basis; the vector
//! returns lost in the decompile are rebuilt from the callees), FUN_0015b0c8 (birth / death / advance),
//! FUN_0015ad48 position / velocity through the source frame (FUN_001a6318), FUN_0015b428 group fill, the
//! FUN_001394c8 words +0x43c / +0x440 / +0x110 / +0x120 / +0x130 / +0x154 / +0x160 (FUN_0011d5b0 curves).
//! REPLACED: pos.w carries the VU1 size (the +0x110 curve at the particle's age, FUN_0013a688 constant)
//! instead of the birth time for the A13 renderer (birth = vel.w - life, `C2cb5d8::size_curve`).
//! K30: shape kind 1 (FUN_0011d448 per axis in the emitter box the template +0x30 points at), shape kind 2
//! (FUN_001e6f10 on the class-10 path of block +4 at a random arc length in particle i's 1/n share, direction
//! kinds 2 tangent / 3 tangent x +0x40), direction kind 4 (+0x40 through the camera rows DAT_002ddfa0..c0,
//! into the source frame), the FUN_0015ad48 source velocity add (FUN_0022d348, +0x18 0 / 1).
//! UNPORTED: shape kind 3 (mesh record never filled by a ported loader) / direction kind 5; FUN_0015b1b0
//! (template +4 == 1: C2d77d8 world-effect system + FxQueue_push, gfx/fx.rs owns that pool);
//! FUN_00139370 texture words (+0x3b0 = 1, +0x1a0 = 0); the +0xdc palette; the DAT_002ddfb0 camera-side spin
//! sign; FUN_0015b9f0 mode-2 records (FUN_002309e8 animated source sampling); the FUN_0015bf40 /
//! FUN_0015bfd8 motion-mode-0 frame (the K21 step keeps `obj_mat`).
use super::k21_28::{C2cb5d8, Group, Particle};
use super::*;
use crate::engine::player::p5_attach::FUN_0011cf60;

/// One FUN_0015bd20 call: emitter object, count, template words, template block, ctx present (`link`), and
/// the FUN_0015ad48 frame of the block's source object (row vectors, None = no source).
#[derive(Clone, Debug)]
pub struct EmitReq {
    pub obj: usize,
    pub count: i32,
    pub t: [u32; 64],
    pub blk: [u32; 5],
    pub link: bool,
    pub frame: Option<[[f32; 3]; 4]>,
}

/// particles per group (+0x1084 < 0x2b)
pub const GROUP_CAP: usize = 0x2b;
/// K29: FUN_001390d8 takes the run-time systems from the 0x3dd490 + 0x4070 pool (FUN_0011cca8(+0x101c, 0x460,
/// 0x28, 0x10) in FUN_0020d9d0, first chunk made by FUN_0020e740): once its first chunk is allocated (+0x4084)
/// and the free list (+0x407c) is empty it returns 0 (no new chunk), so at most 0x28 systems live at once.
/// The groups (+0x4088, 0x80 per chunk) and mode-2 records (+0x40a0) go through FUN_0011ccc0 directly, which
/// adds a chunk when empty: unbounded. Systems / groups return to the pools through the render-side lists
/// 0x2deaf0 (slot 11 FUN_00139250) / 0x2deb00 (FUN_00139098), handed over by FUN_00160a98.
pub const SYS_CAP: usize = 0x28;
const TAU: f32 = 6.283_185_5;
const HALF_PI: f32 = 1.570_796_4;

fn tf(t: &[u32; 64], off: usize) -> f32 {
    f32::from_bits(t[off / 4])
}
fn v3(t: &[u32; 64], off: usize) -> [f32; 3] {
    [tf(t, off), tf(t, off + 4), tf(t, off + 8)]
}
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
fn len(a: [f32; 3]) -> f32 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}
fn scale(a: [f32; 3], k: f32) -> [f32; 3] {
    [a[0] * k, a[1] * k, a[2] * k]
}
fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

/// FUN_0015ae38(per, range, n): the emission grid {columns, row step, column step, first row, first column}
pub fn ee_sub_15ae38(per: i32, range: [f32; 2], n: i32, rng: &mut GameRng) -> (i32, [f32; 4]) {
    // 0x0015ae38
    let cols = if per < 1 { n } else { per.min(n) };
    let rows = ((n - 1 + cols) / cols.max(1)).max(1);
    let step = FUN_0011cf60(range[1] - range[0]) / rows as f32;
    (cols, [step, TAU / cols as f32, range[0] + step * 0.5, rng.FUN_0011d448(0.0, TAU)])
}

/// K30: what FUN_0015a808 / FUN_0015ad48 read behind the template: the emitter's box (template +0x30 points
/// at it for shape kind 1), the class-10 path of block +4 (shape kind 2), the camera rows DAT_002ddfa0 /
/// b0 / c0 (direction kind 4), the source frame rows of block +0 (FUN_001a6318) and the source velocity
/// FUN_0022d348(source, point) (None = no source / hook).
#[derive(Default)]
pub struct ShapeSrc<'a> {
    pub bx: [f32; 8],
    pub path: Option<std::sync::Arc<crate::engine::player::p5_target::Path>>,
    pub cam: Option<[[f32; 4]; 3]>,
    pub vel: Option<&'a mut dyn FnMut([f32; 3]) -> [f32; 3]>,
}

/// FUN_0015a808(template, i, n): local position and normalised direction (shape kind +0x10, direction kind
/// +0x14), then the position moved along the direction by FUN_0011d448(+0x24, +0x28). `dir` is the caller's
/// buffer: shape kind 2 without a path object returns with it untouched (position 0).
/// UNPORTED: shape kind 3 (the 0x30-byte mesh record FUN_001ae2a0 allocates for kind 3 is never filled by a
/// ported loader; position 0) and its direction kind 5 (vertex normals).
pub fn ee_sub_15a808_src(t: &[u32; 64], i: i32, n: i32, src: &ShapeSrc, frame: Option<&[[f32; 3]; 4]>, dir: &mut [f32; 3], rng: &mut GameRng) -> [f32; 3] {
    // 0x0015a808
    let mut pos = [0.0; 3];
    let mut set_dir = None;
    match t[4] {
        0 => pos = v3(t, 0x30),
        1 => {
            // template +0x30 -> the emitter box {min +0, max +0x10}
            let b = &src.bx;
            pos = [rng.FUN_0011d448(b[0], b[4]), rng.FUN_0011d448(b[1], b[5]), rng.FUN_0011d448(b[2], b[6])];
        }
        2 => {
            let Some(p) = src.path.as_ref() else { return [0.0; 3] };
            let f = p.l.last().copied().unwrap_or(0.0) / n as f32;
            let l = i as f32 * f;
            let u = rng.FUN_0011d448(l, l + f);
            let (q, tan) = p.FUN_001e6f10(u);
            pos = q;
            match t[5] {
                2 => set_dir = Some(tan),
                3 => set_dir = Some(cross(tan, v3(t, 0x40))),
                _ => {}
            }
        }
        _ => {}
    }
    let d = match set_dir {
        Some(d) => d,
        None => match t[5] {
            0 => v3(t, 0x40),
            1 => {
                let o = v3(t, 0x40);
                [pos[0] - o[0], pos[1] - o[1], pos[2] - o[2]]
            }
            4 => {
                // camera-relative: +0x40 through the camera rows, into the source frame when there is one
                let c = src.cam.unwrap_or([[0.0; 4]; 3]);
                let o = v3(t, 0x40);
                let w = [0, 1, 2].map(|k| c[0][k] * o[0] + c[1][k] * o[1] + c[2][k] * o[2]);
                match frame {
                    Some(m) => [0, 1, 2].map(|r| m[r][0] * w[0] + m[r][1] * w[1] + m[r][2] * w[2]),
                    None => w,
                }
            }
            _ => [1.0, 0.0, 0.0],
        },
    };
    let l = len(d);
    *dir = if l < 0.0001 { [1.0, 0.0, 0.0] } else { scale(d, 1.0 / l) };
    let k = rng.FUN_0011d448(tf(t, 0x24), tf(t, 0x28));
    add(pos, scale(*dir, k))
}

/// FUN_0015a808 with only the template words (shape kinds 0 / direction kinds 0, 1, default)
pub fn ee_sub_15a808(t: &[u32; 64], rng: &mut GameRng) -> ([f32; 3], [f32; 3]) {
    let mut dir = [1.0, 0.0, 0.0];
    let pos = ee_sub_15a808_src(t, 0, 1, &ShapeSrc::default(), None, &mut dir, rng);
    (pos, dir)
}

/// FUN_0011e2d8(dir, DAT_002a2da0, DAT_002a2db0): the two unit axes across `dir`
pub fn ee_sub_11e2d8(dir: [f32; 3]) -> [[f32; 3]; 2] {
    let mut a = cross([1.0, 0.0, 0.0], dir);
    if a[0] * a[0] + a[1] * a[1] + a[2] * a[2] < 0.0001 {
        a = cross([0.0, 1.0, 0.0], dir);
    }
    let b = cross(dir, a);
    [scale(a, 1.0 / len(a)), scale(b, 1.0 / len(b))]
}

/// FUN_0015af08(jitter, zscale, grid, speed, dir, i): the velocity of particle `i`
pub fn ee_sub_15af08(jit: f32, zs: f32, grid: (i32, [f32; 4]), speed: [f32; 2], dir: [f32; 3], i: i32, rng: &mut GameRng) -> [f32; 3] {
    // 0x0015af08
    let (cols, g) = grid;
    let mut el = g[2] + (i / cols) as f32 * g[0];
    let mut az = g[3] + (i % cols) as f32 * g[1];
    if jit != 0.0 {
        el += jit * rng.FUN_0011d448(-0.5, 0.5) * g[0];
        az += jit * rng.FUN_0011d448(-0.5, 0.5) * g[1];
    }
    let b = FUN_0011cf60(el + HALF_PI);
    let a = FUN_0011cf60(az);
    let r = rng.FUN_0011d448(speed[0], speed[1]);
    // FUN_0011e208(a, b, r)
    let s = [b.cos() * a.cos() * r, b.cos() * a.sin() * r, b.sin() * r * zs];
    let ax = ee_sub_11e2d8(dir);
    add(add(scale(ax[0], s[0]), scale(ax[1], s[1])), scale(dir, s[2]))
}

/// FUN_0015b7e8(n, template, out): per particle (position, velocity, birth, death); FUN_0015b0c8 for the
/// times / advance and FUN_0015ad48 through the source `frame`
pub fn ee_sub_15b7e8(n: i32, t: &[u32; 64], now: f32, frame: Option<&[[f32; 3]; 4]>, rng: &mut GameRng) -> Vec<([f32; 3], [f32; 3], f32, f32)> {
    ee_sub_15b7e8_src(n, t, now, frame, &mut ShapeSrc::default(), rng)
}

/// FUN_0015b7e8 with the K30 shape / source context
pub fn ee_sub_15b7e8_src(n: i32, t: &[u32; 64], now: f32, frame: Option<&[[f32; 3]; 4]>, src: &mut ShapeSrc, rng: &mut GameRng) -> Vec<([f32; 3], [f32; 3], f32, f32)> {
    // 0x0015b7e8
    let grid = ee_sub_15ae38(t[0x84 / 4] as i32, [tf(t, 0x90), tf(t, 0x94)], n, rng);
    let mut out = Vec::with_capacity(n.max(0) as usize);
    let mut dir = [1.0, 0.0, 0.0];
    for i in 0..n {
        let mut pos = ee_sub_15a808_src(t, i, n, src, frame, &mut dir, rng);
        // FUN_0015b0c8(template + 0x50, ..)
        let vel = ee_sub_15af08(tf(t, 0x88), tf(t, 0x5c), grid, [tf(t, 0x50), tf(t, 0x54)], dir, i, rng);
        let birth = now - rng.FUN_0011d448(tf(t, 0xa0), tf(t, 0xa4));
        let death = birth + tf(t, 0x8c);
        let adv = rng.FUN_0011d448(tf(t, 0x98), tf(t, 0x9c));
        if adv != 0.0 {
            pos = add(pos, scale(vel, adv));
        }
        // FUN_0015ad48: FUN_001a72d0 / FUN_001a7348 through the source frame (+0x18 == 3: velocity kept);
        // K30: +0x18 != 2: plus the source velocity FUN_0022d348 at the particle (+0x18 == 1) or the
        // source origin (+0x70)
        let (pos, vel) = match frame {
            Some(m) => {
                let p = [0, 1, 2].map(|c| pos[0] * m[0][c] + pos[1] * m[1][c] + pos[2] * m[2][c] + m[3][c]);
                let mut v = vel;
                if t[6] != 3 {
                    v = [0, 1, 2].map(|c| vel[0] * m[0][c] + vel[1] * m[1][c] + vel[2] * m[2][c]);
                    if t[6] != 2 {
                        if let Some(f) = src.vel.as_mut() {
                            v = add(v, f(if t[6] == 1 { p } else { m[3] }));
                        }
                    }
                }
                (p, v)
            }
            None => (pos, vel),
        };
        out.push((pos, vel, birth, death));
    }
    out
}

/// FUN_0011d5b0(k, 0, src): the curve src[0] + src[1] u + src[2] u^2 in time (u = k * age)
pub fn ee_sub_11d5b0(k: f32, s: [f32; 3]) -> [f32; 3] {
    [s[0], s[1] * k, s[2] * k * k]
}

/// FUN_001394c8(system, template): the template words of a class-28 system
pub fn ee_sub_1394c8(c: &mut C2cb5d8, t: &[u32; 64]) {
    // 0x001394c8
    let spin = t[0xc0 / 4] as i32;
    c.m.set_i(0x43c, spin);
    c.m.set_f(0x440, if spin == 1 { tf(t, 0xf0) } else { 1.0 });
    // FUN_00139370 (texture): one frame
    if c.m.i(0x3b0) == 0 {
        c.m.set_i(0x3b0, 1);
    }
    let life = tf(t, 0x8c);
    let col = ee_sub_11d5b0(1.0 / life, v3(t, 0xb0));
    let size = ee_sub_11d5b0(1.0 / life, v3(t, 0xe0));
    for k in 0..3 {
        c.m.set_f(0x130 + 4 * k as u32, col[k]);
        c.m.set_f(0x110 + 4 * k as u32, size[k]);
    }
    // +0x120: the largest |size| over [0, life]
    let at = |a: f32| size[0] + a * (size[1] + a * size[2]);
    let mut m = at(0.0).abs().max(at(life).abs());
    if size[2] != 0.0 {
        let a = -size[1] / (size[2] + size[2]);
        if 0.0 < a && a < life {
            m = m.max(at(a).abs());
        }
    }
    c.m.set_f(0x120, m);
    c.m.set_f(0x150, 0.0);
    c.m.set_f(0x158, 0.0);
    c.m.set_f(0x154, 1.0 / life);
    let mut mode = t[0xc4 / 4] as i32;
    // mode 0 without gravity / wind / drag -> 1
    if mode == 0 && v3(t, 0x60) == [0.0; 3] && tf(t, 0x58) == 0.0 && tf(t, 0x80) == 0.0 {
        mode = 1;
    }
    c.m.set_i(0x160, mode);
    c.size_curve = Some([size[0], size[1], size[2], life]);
}

impl ClassSet {
    /// FUN_0015bd20 requests of the frame -> FUN_0015b428 per request
    pub fn k28_spawn(&mut self, h: &mut dyn ClassHost) {
        // K30: anim event 0x11 requests -> the target's vtable +0x19c (class 33 FUN_0015c690, class 31
        // FUN_0015c3c0); before the requests of the frame are taken
        for (o, r) in h.k30_take_reqs() {
            let i = o as usize - 1;
            let rng = &mut self.rng;
            match self.insts.iter_mut().chain(self.more.iter_mut()).find(|x| x.0 == i).map(|x| &mut x.1) {
                Some(Inst::C2ce4d0(c)) => c.ee_sub_15c690(i, &r, rng, h),
                Some(Inst::C2ce828(c)) => c.ee_sub_15c3c0(i, &r, &[[0.0; 4]; 6], h),
                _ => {}
            }
        }
        for r in h.k28_take_emits() {
            // template +4 == 1: FUN_0015b1b0 (the +0x14c system, UNPORTED) instead of FUN_0015b428
            if r.count < 1 || r.t[1] == 1 {
                continue;
            }
            // K30: the emitter box (shape kind 1), block +4 path (kind 2), camera rows (direction kind 4),
            // source velocity FUN_0022d348 of block +0
            let bx = self.more.iter().chain(&self.insts).chain(&self.k23c).find(|x| x.0 == r.obj).map_or([0.0; 8], |x| match &x.1 {
                Inst::C2ceb78(e) => e.bx,
                Inst::C2ce4d0(e) => e.bx,
                Inst::K23C32(e) => e.bx,
                _ => [0.0; 8],
            });
            let path = if r.t[4] == 2 && r.blk[1] != 0 { h.k22_path(r.blk[1]) } else { None };
            let cam = if r.t[5] == 4 { h.k30_cam_rows() } else { None };
            let src_obj = r.blk[0];
            let mut vel = |p: [f32; 3]| h.ee_sub_22d348(src_obj, p);
            let mut src = ShapeSrc { bx, path, cam, vel: None };
            if r.frame.is_some() && src_obj != 0 {
                src.vel = Some(&mut vel);
            }
            let parts = ee_sub_15b7e8_src(r.count, &r.t, self.g.t, r.frame.as_ref(), &mut src, &mut self.rng);
            drop(src);
            self.ee_sub_15b428(&r, parts, h);
        }
    }

    /// FUN_0015b428(template, ctx, n, ..): the system of the ctx slot (emitter +0x150), else a new one
    pub fn ee_sub_15b428(&mut self, r: &EmitReq, parts: Vec<([f32; 3], [f32; 3], f32, f32)>, h: &mut dyn ClassHost) {
        // 0x0015b428
        let live = |s: &Self, g: usize| s.k21c.iter().position(|x| x.0 == g && matches!(&x.1, Inst::K21C28(c) if !c.removed));
        let slot = if r.link { self.k28_sys.iter().find(|x| x.0 == r.obj).map(|x| x.1) } else { None };
        let k = match slot.and_then(|g| live(self, g)) {
            Some(k) => k,
            None => {
                // K29: FUN_001390d8 returns 0 when the 0x4070 pool is exhausted -> no particles
                let n_live = self.k21c.iter().filter(|x| matches!(&x.1, Inst::K21C28(c) if !c.removed)).count();
                if n_live >= SYS_CAP {
                    return;
                }
                // FUN_001390d8(0x3dd490): a new class-28 object (slots 6 / 1), FUN_001394c8, slot 50
                let g = self.next_obj;
                self.next_obj += 1;
                let mut c = C2cb5d8::default();
                c.ee_sub_1391c8(g, h);
                c.ee_sub_139318(g, h);
                ee_sub_1394c8(&mut c, &r.t);
                if r.link {
                    c.ee_sub_139900(g, r.obj as u32 + 1, h);
                    self.k28_sys.retain(|x| x.0 != r.obj);
                    self.k28_sys.push((r.obj, g));
                    if let Some((_, Inst::C2ceb78(e))) = self.more.iter_mut().find(|x| x.0 == r.obj) {
                        e.f_150 = g as i32 + 1;
                    }
                }
                self.k21c.push((g, Inst::K21C28(Box::new(c))));
                self.k21c.len() - 1
            }
        };
        let now = self.g.t;
        let rng = &mut self.rng;
        let Inst::K21C28(c) = &mut self.k21c[k].1 else { return };
        let (spin, tex, f434, f1a0) = (c.m.i(0x43c), c.m.i(0x3b0), c.m.i(0x434), c.m.i(0x1a0));
        let mut gi = 0;
        for (pos, vel, birth, death) in parts {
            while gi < c.groups.len() && c.groups[gi].parts.len() >= GROUP_CAP {
                gi += 1;
            }
            if gi == c.groups.len() {
                // FUN_00138ff8(0x3dd490, system)
                c.groups.push(Group::default());
            }
            let mut s = [0f32; 4];
            let f = rng.FUN_0011d3f8(0, tex - 1) as u32;
            s[0] = f32::from_bits(if f434 == 0 { f } else { f & !1 });
            if r.t[0xc8 / 4] != 0 {
                s[1] = f32::from_bits(rng.FUN_0011d3f8(0, f1a0 - 1) as u32);
            }
            if spin == 2 {
                s[2] = rng.FUN_0011d448(tf(&r.t, 0xf8), tf(&r.t, 0xfc));
                if r.t[0xf4 / 4] != 0 {
                    s[3] = rng.FUN_0011d448(-std::f32::consts::PI, std::f32::consts::PI);
                }
            }
            let size = c.size_at(now - birth);
            c.groups[gi].parts.push(Particle { pos: [pos[0], pos[1], pos[2], size], vel: [vel[0], vel[1], vel[2], death], spin: s, rec: [0; 8] });
            c.m.set_i(0x444, c.m.i(0x444) + 1);
        }
    }
}
