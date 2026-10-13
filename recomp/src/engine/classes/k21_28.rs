//! WP-K21: class 28 C2cb5d8 (vtable 0x2cb5d8): a particle system. Its particles live in groups (+0x448 list,
//! link +0x450; 0x1090-byte groups out of the DAT_002deb00 pool) double buffered at group + buf * 0x840:
//! +0 positions (w = size), +0x2b0 velocities (w = death time), +0x560 spin (z = rate, w = angle); +0x1080 the
//! current buffer, +0x1084 the count, +0x1088 the mode-2 records (0x20 bytes), +0x108c the owner. +0x444 is
//! the total count, +0x43c the spin mode (2 = spin), +0x160 the motion mode (0 = moved by the +0x170 frame,
//! 1 = free flight, 2 = curve), +0xe0 the parent (kind 0x1d), +0xf0 / +0x100 the bounding sphere.
//! 1:1: slot 1 FUN_00139318, slot 6 FUN_001391c8, slot 11 FUN_00139250, slot 15 FUN_001398e0, slot 50
//! FUN_00139900, slot 16 FUN_00139dc0 (CPU branch; bounds, death, compaction, spin), FUN_00139098 (group free).
//! REPLACED: groups are `Group` values (the pool / list words are not modelled; `add_group` stands for the
//! emitters FUN_001400b8 / 140f90 / 159a20 / ...); DAT_002bc9c0 == 0 sends motion mode 0 to FUN_00139ab8, whose
//! VU0 micro program 0x10 (FUN_00139950) is taken as the CPU branch of slot 16 (M * v + T, p + v * dt).
//! UNPORTED: motion mode 2 position (FUN_00117120 Hermite curve, its vector arguments are lost in the
//! decompile: the position is kept), slot 18 FUN_0013a1e8 (VIF / VU1 render packets).
#![allow(non_snake_case)]
use super::k17_55::Mem;
use super::*;

/// DAT_002a4afc / DAT_002a4b00
pub const F_MAX: f32 = f32::MAX;
pub const F_MIN: f32 = -f32::MAX;

/// one particle (the three 0x10 entries of a buffer, + the mode-2 record)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Particle {
    pub pos: [f32; 4],
    pub vel: [f32; 4],
    pub spin: [f32; 4],
    pub rec: [u32; 8],
}

/// one 0x1090 group (the live buffer only: the swap is the compaction)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Group {
    pub parts: Vec<Particle>,
    /// +0x1080
    pub cur: u32,
}

#[derive(Clone, Debug, Default)]
pub struct C2cb5d8 {
    pub m: Mem,
    pub groups: Vec<Group>,
    /// set by vtable +0x4c (the object removes itself once empty)
    pub removed: bool,
    /// K28: the +0x110 size curve (c0, c1, c2 per second, life) of an emitter-spawned system: pos.w carries
    /// the size at the particle's age (birth = vel.w - life), refreshed by slot 16 (classes/k28_emit.rs)
    pub size_curve: Option<[f32; 4]>,
}

impl C2cb5d8 {
    /// slot 1 FUN_00139318: into the DAT_003dd490 kind-0x1c list and the DAT_003e1500 list
    pub fn ee_sub_139318(&mut self, obj: usize, h: &mut dyn ClassHost) {
        let l = h.ee_call(0x15e3a8, &[0x3dd490, 0x1c]) as u32;
        h.ee_call(0x11a118, &[l, obj as u32 + 1]);
        h.ee_call(0x11cd98, &[0x3e1500, obj as u32 + 1]);
    }
    /// slot 6 FUN_001391c8 (after FUN_00231100 = `h.base`)
    pub fn ee_sub_1391c8(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x231100, obj);
        self.m.set_u(0x450, 0x1090);
        self.m.set_i(0x18, -1);
        self.m.set_u(0x38, 0x501502f9);
        h.ee_sub_1a6ca8(obj, 1);
        // +0x40 draw flags: low bit kept, | 0xe
        self.m.set_u(0x40, self.m.u(0x40) & 1 | 0xe);
    }
    /// FUN_00139098(group): off the owner list, back to the DAT_002deb00 pool
    pub fn ee_sub_139098(&mut self, k: usize) {
        self.groups.remove(k);
    }
    /// slot 11 FUN_00139250: free every group, FUN_001a5848 / 1a57a8 (/ 1a53e0 unparented), to DAT_002deaf0
    pub fn ee_sub_139250(&mut self, obj: usize, h: &mut dyn ClassHost) {
        while !self.groups.is_empty() {
            self.ee_sub_139098(0);
        }
        h.ee_call(0x1a5848, &[obj as u32 + 1]);
        h.ee_call(0x1a57a8, &[obj as u32 + 1]);
        if self.m.u(0x10) == 0 {
            h.ee_call(0x1a53e0, &[obj as u32 + 1]);
        }
        h.ee_call(0x11a220, &[0x2deaf0, obj as u32 + 1]);
    }
    /// slot 15 FUN_001398e0 = FUN_001a5e28
    pub fn ee_sub_1398e0(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1a5e28, obj);
    }
    /// slot 50 FUN_00139900(self, other): FUN_001333c0, then a kind-0x1d `other` becomes the parent (+0xe0,
    /// +0x38 = its +0x38)
    pub fn ee_sub_139900(&mut self, obj: usize, other: u32, h: &mut dyn ClassHost) {
        h.base(0x1333c0, obj);
        if h.ee_call(0x132b10, &[other, 0x1d]) != 0 {
            self.m.set_u(0xe0, other);
            self.m.set_i(0x38, h.obj_i32(other, 0x38));
        }
    }
    /// K28: the +0x110 size at `age` (the VU1 program's size; pos.w kept when no curve)
    pub fn size_at(&self, age: f32) -> f32 {
        self.size_curve.map_or(0.0, |c| c[0] + age * (c[1] + age * c[2]))
    }
    /// add a group (the emitters' FUN_0011a* list insert): +0x444 += n
    pub fn add_group(&mut self, parts: Vec<Particle>) {
        self.m.set_i(0x444, self.m.i(0x444) + parts.len() as i32);
        self.groups.push(Group { parts, cur: 0 });
    }

    /// slot 16 FUN_00139dc0: step every live particle (`now` DAT_002dded0, `dt` DAT_002dded4; `mtx` the
    /// FUN_0015bfd8 frame of +0x170 for motion mode 0), drop the dead ones, bounding sphere, remove when empty
    pub fn ee_sub_139dc0(&mut self, now: f32, dt: f32, mtx: &[[f32; 4]; 4]) {
        let mode = self.m.i(0x160);
        let spin = self.m.i(0x43c);
        let mut lo = [F_MAX; 3];
        let mut hi = [F_MIN; 3];
        let mut k = 0;
        while k < self.groups.len() {
            let g = &mut self.groups[k];
            let mut out = Vec::with_capacity(g.parts.len());
            let mut dead = 0;
            for s in &g.parts {
                if s.vel[3] <= now {
                    dead += 1;
                    continue;
                }
                let mut d = *s;
                match mode {
                    0 | 1 => {
                        let v = if mode == 0 {
                            let v = s.vel;
                            let mut r = [0f32; 4];
                            for c in 0..4 {
                                r[c] = mtx[0][c] * v[0] + mtx[1][c] * v[1] + mtx[2][c] * v[2] + mtx[3][c];
                            }
                            r
                        } else {
                            s.vel
                        };
                        d.vel = v;
                        for c in 0..3 {
                            d.pos[c] = s.pos[c] + v[c] * dt;
                        }
                    }
                    // mode 2: FUN_00117120 curve (UNPORTED, position kept); the record is carried with the
                    // particle (the compaction copy)
                    _ => {}
                }
                d.pos[3] = match self.size_curve {
                    Some(c) => c[0] + (now - (s.vel[3] - c[3])) * (c[1] + (now - (s.vel[3] - c[3])) * c[2]),
                    None => s.pos[3],
                };
                d.vel[3] = s.vel[3];
                d.spin = s.spin;
                if spin == 2 {
                    d.spin[3] = crate::engine::player::p5_attach::FUN_0011cf60(s.spin[3] + s.spin[2] * dt);
                }
                for c in 0..3 {
                    lo[c] = lo[c].min(d.pos[c]);
                    hi[c] = hi[c].max(d.pos[c]);
                }
                out.push(d);
            }
            g.parts = out;
            self.m.set_i(0x444, self.m.i(0x444) - dead);
            if g.parts.is_empty() {
                self.ee_sub_139098(k);
                continue;
            }
            g.cur ^= 1;
            k += 1;
        }
        if self.m.i(0x444) < 1 {
            // vtable +0x4c: remove the object
            self.removed = true;
            return;
        }
        let c = [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5, (lo[2] + hi[2]) * 0.5];
        self.m.set_v3(0xf0, c);
        let d = [c[0] - hi[0], c[1] - hi[1], c[2] - hi[2]];
        self.m.set_f(0x100, (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() + self.m.f(0x120));
    }
}

/// K21 class 28 in the ClassSet (no level loads one; the emitters spawn them at run time, K28 k28_emit.rs)
impl ClassSet {
    pub(super) fn build_k21_28(&mut self, w: &crate::engine::world::World, h: &mut dyn ClassHost) {
        for (i, o) in w.objects.iter().enumerate() {
            if o.class != 28 {
                continue;
            }
            let mut c = C2cb5d8::default();
            c.ee_sub_1391c8(i, h);
            c.ee_sub_139318(i, h);
            self.k21c.push((i, Inst::K21C28(Box::new(c))));
        }
    }
    pub(super) fn frame_k21_28(&mut self, h: &mut dyn ClassHost) {
        // K28: the emitters' FUN_0015bd20 requests of this frame (classes/k28_emit.rs)
        self.k28_spawn(h);
        let (t, dt) = (self.g.t, self.g.dt);
        for (i, x) in &mut self.k21c {
            if let Inst::K21C28(c) = x {
                if c.removed {
                    continue;
                }
                c.ee_sub_1398e0(*i, h);
                let m = h.obj_mat(*i as u32 + 1, 0x170);
                c.ee_sub_139dc0(t, dt, &m);
            }
        }
    }
}
