//! WP-K11: K10-unported classes. 20 C2c9a80 (local transform node: +0x50 ALO it hangs off, +0x54 world
//! stamp, +0x60 local Mat4, +0xa0 cached world Mat4). Load (slot 13 FUN_00231170) is ported in world
//! (`LoadData::L231170`), slot 48 FUN_0022d750 elsewhere. The guard attack records (class 120) are in
//! guard/attack.rs. 14 C2d4de0: rail outline (winding normalised to kind 3 at load-post, listed in
//! DAT_003e1a4c); its slot 16 is the class-10 FUN_001e6340 (ported) run per frame. Object handles are index + 1 (0 = null).
//! REPLACED: the ALO pointer +0x50 = owner +0x60 (FUN_00231100) is the first owner-chain object with an ALO
//! payload; the ALO world matrix (+0x40) and its stamp (+0xa4) come from `ClassHost::obj_mat` /
//! `ClassHost::obj_i32`; slot 49 FUN_00231220 (clone pointer relocation into the clone's ALO block) is a
//! no-op on index handles.
use super::*;
use crate::engine::anim::pose::{ee_sub_11c168, ee_sub_11c318, from_4x3, Mat4, IDENTITY};
use crate::engine::gfx::attach::{alo_of, xfm_of};

/// K11 world class id -> vtable
pub const PORTED_K11: &[(ClassId, u32)] = &[(20, 0x2c9a80), (14, 0x2d4de0)];

/// slot 5 FUN_00113b50
pub fn slot5(class: ClassId) -> Option<u32> {
    match class {
        20 => Some(0x2bac78),
        14 => Some(SLOT5_14),
        _ => None,
    }
}

/// DAT_002c44f0: the static identity ALO returned by slot 31 when there is none
pub const DAT_002C44F0: u32 = 0x2c44f0;

#[derive(Clone, Debug)]
pub struct C2c9a80 {
    /// +0x50 ALO handle (0 = none), +0x54 world stamp (-1 = dirty)
    pub f_50: u32,
    pub f_54: i32,
    /// +0x60 local matrix, +0xa0 world matrix
    pub f_60: Mat4,
    pub f_a0: Mat4,
    /// +0x10 == 0 && +0x28 bit 0x3c (FUN_002313f0 -> FUN_00147e88(0x3dd490, +0x14))
    pub root_b3c: bool,
}

impl Default for C2c9a80 {
    fn default() -> Self {
        C2c9a80 { f_50: 0, f_54: -1, f_60: IDENTITY, f_a0: IDENTITY, root_b3c: false }
    }
}

fn pt(m: &Mat4, p: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|r| m[0][r] * p[0] + m[1][r] * p[1] + m[2][r] * p[2] + m[3][r])
}

impl C2c9a80 {
    /// slot 6 FUN_00231100(obj): base FUN_001a5270, +0x50 = owner ALO, local = identity, FUN_002313f0
    pub fn ee_sub_231100(&mut self, obj: usize, alo: u32, h: &mut dyn ClassHost) {
        h.base(0x1a5270, obj);
        if alo != 0 {
            self.f_50 = alo;
        }
        self.f_60 = IDENTITY;
        self.ee_sub_2313f0(obj, h);
    }
    /// FUN_002313f0: world dirty
    pub fn ee_sub_2313f0(&mut self, obj: usize, h: &mut dyn ClassHost) {
        self.f_54 = -1;
        if self.root_b3c {
            h.base(0x147e88, obj);
        }
    }
    /// FUN_00231368: refresh +0xa0 from the ALO world (+0x40) when its stamp (+0xa4) moved
    pub fn ee_sub_231368(&mut self, h: &mut dyn ClassHost) {
        if self.f_50 == 0 {
            if self.f_54 >= 0 {
                return;
            }
            self.f_a0 = self.f_60;
            return;
        }
        let stamp = h.obj_i32(self.f_50, 0xa4);
        if stamp <= self.f_54 {
            return;
        }
        self.f_54 = stamp;
        self.f_a0 = ee_sub_11c168(&h.obj_mat(self.f_50, 0x40), &self.f_60);
    }
    /// slot 30 FUN_00231440: world matrix (+0xa0)
    pub fn ee_sub_231440(&mut self, h: &mut dyn ClassHost) -> Mat4 {
        self.ee_sub_231368(h);
        self.f_a0
    }
    /// slot 31 FUN_00231550(out): local matrix + the ALO (DAT_002c44f0 when none)
    pub fn ee_sub_231550(&self) -> (Mat4, u32) {
        (self.f_60, if self.f_50 == 0 { DAT_002C44F0 } else { self.f_50 })
    }
    /// slot 28 FUN_002312a0(pos): world position -> ALO space (FUN_0022fc48(0, alo, pos)), w = 1
    pub fn ee_sub_2312a0(&mut self, obj: usize, pos: [f32; 3], h: &mut dyn ClassHost) {
        let p = if self.f_50 != 0 { pt(&ee_sub_11c318(&h.obj_mat(self.f_50, 0x40)), pos) } else { pos };
        self.f_60[3] = [p[0], p[1], p[2], 1.0];
        self.ee_sub_2313f0(obj, h);
    }
    /// slot 29 FUN_00231300(rot): world rotation rows -> ALO space (FUN_0022fd38(0, alo, rot)), w = 0
    pub fn ee_sub_231300(&mut self, obj: usize, rot: [[f32; 3]; 3], h: &mut dyn ClassHost) {
        let inv = if self.f_50 != 0 { ee_sub_11c318(&h.obj_mat(self.f_50, 0x40)) } else { IDENTITY };
        for r in 0..3 {
            let v = rot[r];
            self.f_60[r] = [
                v[0] * inv[0][0] + v[1] * inv[1][0] + v[2] * inv[2][0],
                v[0] * inv[0][1] + v[1] * inv[1][1] + v[2] * inv[2][1],
                v[0] * inv[0][2] + v[1] * inv[1][2] + v[2] * inv[2][2],
                0.0,
            ];
        }
        self.ee_sub_2313f0(obj, h);
    }
    /// slot 36 FUN_00231598(m): local = m * local (FUN_0011c168(out, m, +0x60))
    pub fn ee_sub_231598(&mut self, obj: usize, m: &Mat4, h: &mut dyn ClassHost) {
        self.f_60 = ee_sub_11c168(m, &self.f_60);
        self.ee_sub_2313f0(obj, h);
    }
    /// slot 32 FUN_00231468: velocity of the ALO at the node's world position (FUN_0022d348, hook);
    /// DAT_002a2dd0 (0) without an ALO
    pub fn ee_sub_231468(&self, h: &mut dyn ClassHost) -> [f32; 3] {
        if self.f_50 == 0 {
            return [0.0; 3];
        }
        let p = pt(&h.obj_mat(self.f_50, 0x40), [self.f_60[3][0], self.f_60[3][1], self.f_60[3][2]]);
        h.ee_sub_22d348(self.f_50, p)
    }
    /// slot 40 FUN_002311b0(src): base FUN_001a6730, ALO link copied (vt+0x194 remap of +0x50), dirty
    pub fn ee_sub_2311b0(&mut self, obj: usize, src: &C2c9a80, h: &mut dyn ClassHost) {
        h.base(0x1a6730, obj);
        self.f_50 = src.f_50;
        self.f_60 = src.f_60;
        self.ee_sub_2313f0(obj, h);
    }
    /// slot 9 FUN_002315f0(owner_alo, alo, flags): re-parent; flags & 3 keep the world transform
    /// (world = chain of the old ALO, re-applied through slots 28 (bit 1) / 29 (bit 2) under the new one)
    pub fn ee_sub_2315f0(&mut self, obj: usize, owner_alo: u32, alo: u32, flags: u32, h: &mut dyn ClassHost) {
        let world = if flags & 3 != 0 {
            if self.root_b3c {
                Some(self.ee_sub_231440(h))
            } else if self.f_50 != 0 {
                Some(ee_sub_11c168(&h.obj_mat(self.f_50, 0x40), &self.f_60))
            } else {
                Some(self.f_60)
            }
        } else {
            None
        };
        self.f_50 = if alo != 0 { alo } else { owner_alo };
        h.base(0x1a5900, obj);
        if let Some(m) = world {
            if flags & 1 != 0 {
                self.ee_sub_2312a0(obj, [m[3][0], m[3][1], m[3][2]], h);
            }
            if flags & 2 != 0 {
                self.ee_sub_231300(obj, [[m[0][0], m[0][1], m[0][2]], [m[1][0], m[1][1], m[1][2]], [m[2][0], m[2][1], m[2][2]]], h);
            }
        }
    }
}

/// slot 5 FUN_00113af0 (class 14)
const SLOT5_14: u32 = 0x2bac00;
/// DAT_002c0ab0: FUN_001bc278 collinearity band
const DAT_002C0AB0: f32 = 1.0;
/// object list DAT_003e1a4c (class 14 slots 10 / 11, FUN_001bd218)
pub const L_3E1A4C: u32 = 0x3e1a4c;

/// FUN_001bc208(a, b): lexicographic (x, then y) compare of two points: -1 / 0 / 1
pub fn ee_sub_1bc208(a: [f32; 3], b: [f32; 3]) -> i32 {
    if a[0] < b[0] {
        return -1;
    }
    if b[0] < a[0] {
        return 1;
    }
    if a[1] < b[1] {
        -1
    } else if b[1] < a[1] {
        1
    } else {
        0
    }
}

/// FUN_001bc278(a, b, c): turn direction of a -> b -> c in xy (|cross| <= DAT_002c0ab0 -> 0)
pub fn ee_sub_1bc278(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> i32 {
    let f = (a[0] - b[0]) * (b[1] - c[1]) - (a[1] - b[1]) * (b[0] - c[0]);
    if f < -DAT_002C0AB0 {
        -1
    } else if f > DAT_002C0AB0 {
        1
    } else {
        0
    }
}

/// FUN_001bc530(n, i): knot index wrap (i % n; negative -> + n - 1; the closing knot n - 1 -> 0)
pub fn ee_sub_1bc530(n: i32, i: i32) -> i32 {
    let p = i % n;
    if p < 0 {
        p - 1 + n
    } else if p == n - 1 {
        0
    } else {
        p
    }
}

type Path = crate::engine::player::p5_attach::Path;

/// C2d4de0 (world class 14): a class-10 rail used as an outline polygon. +0x190 winding kind
/// (FUN_001bc2f8), +0x1c4 member list (link offset 0x1d0), +0x1d8 owning outline, +0x1dc FUN_001c1968
/// user, +0x1e8, +0x1ec edge buffer ((n - 1) * 0x10 bytes, FUN_001ae2a0: its size here).
#[derive(Clone, Debug, Default)]
pub struct C2d4de0 {
    pub base: super::c2d9040::C2d9040,
    pub path: Option<Path>,
    pub f_190: i32,
    pub f_1c4: Vec<u32>,
    pub f_1d8: u32,
    pub f_1dc: u32,
    pub f_1e8: i32,
    pub f_1ec: usize,
}

impl C2d4de0 {
    /// slot 6 FUN_001bc050: class-10 init FUN_001e5d80, +0x1e8 = -1, +0x1c4 list (FUN_0011a1f8 offset 0x1d0)
    pub fn ee_sub_1bc050(&mut self, obj: usize, h: &mut dyn ClassHost) {
        self.base.ee_sub_1e5d80(obj, h);
        self.f_1e8 = -1;
        self.f_1c4.clear();
    }
    /// slot 13 FUN_001bc088: class-10 load FUN_001e5df8 (world loader: `guard::rail_of`) + the edge buffer
    pub fn ee_sub_1bc088(&mut self, path: Option<Path>) {
        let n = path.as_ref().map_or(0, |p| p.pts.len());
        self.path = path;
        self.f_1ec = (n * 0x10).saturating_sub(0x10);
    }
    /// knots in world space (vt+0xfc matrix)
    pub fn world_pts(&self) -> Vec<[f32; 3]> {
        let Some(p) = &self.path else { return vec![] };
        p.pts.iter().map(|q| [0, 1, 2].map(|j| q[0] * p.rot[0][j] + q[1] * p.rot[1][j] + q[2] * p.rot[2][j] + p.pos[j])).collect()
    }
    /// FUN_001bc2f8: 2 = degenerate (< 2 knots or no turn), 3 / 4 = convex turning +1 / -1 (xy), 0 = turns
    /// both ways; x direction reversing 3+ times -> 1 without a turn, else 0
    pub fn ee_sub_1bc2f8(&self) -> i32 {
        let p = self.world_pts();
        let n = p.len() as i32;
        if n <= 1 {
            return 2;
        }
        let (mut flips, mut turn) = (0, 0);
        let mut dir = ee_sub_1bc208(p[0], p[1]);
        for i in 0..n - 1 {
            let a = ee_sub_1bc530(n, i) as usize;
            let b = ee_sub_1bc530(n, i + 1) as usize;
            let c = ee_sub_1bc530(n, i + 2) as usize;
            let d = ee_sub_1bc208(p[b], p[c]);
            if d != 0 && d == -dir {
                flips += 1;
            }
            let t = ee_sub_1bc278(p[a], p[b], p[c]);
            let opp = -turn;
            if t != 0 {
                turn = t;
                if t == opp {
                    return 0;
                }
            }
            dir = d;
        }
        if flips < 3 {
            match turn {
                -1 => 4,
                1 => 3,
                _ => 2,
            }
        } else {
            (turn == 0) as i32
        }
    }
    /// slot 14 FUN_001bc570: base FUN_001a5c48; kind 4 -> reverse the knots (s = total - s, FUN_001e6180),
    /// kind 3
    pub fn ee_sub_1bc570(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1a5c48, obj);
        self.f_190 = self.ee_sub_1bc2f8();
        if self.f_190 != 4 {
            return;
        }
        if let Some(p) = &mut self.path {
            let total = crate::engine::player::p5_attach::FUN_001e6630(p);
            let n = p.pts.len();
            for i in 0..n / 2 {
                let j = n - i - 1;
                p.pts.swap(i, j);
                p.s.swap(i, j);
                p.s[i] = total - p.s[i];
                p.s[j] = total - p.s[j];
            }
            crate::engine::player::p6::FUN_001e6180(p);
        }
        self.f_190 = 3;
    }
    /// slot 10 FUN_001bc160: FUN_001a5800, append to DAT_003e1a4c
    pub fn ee_sub_1bc160(&mut self, obj: usize, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        h.base(0x1a5800, obj);
        g.list(L_3E1A4C).push(obj as u32 + 1);
    }
    /// slot 11 FUN_001bc198: FUN_001a5848, owner detach FUN_001bd218 (hook), FUN_001bd1e0 (+0x1dc user:
    /// FUN_001c1968 hook), remove from DAT_003e1a4c
    pub fn ee_sub_1bc198(&mut self, obj: usize, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        h.base(0x1a5848, obj);
        if self.f_1d8 != 0 {
            h.base(0x1bd218, obj);
        }
        if self.f_1dc != 0 {
            h.base(0x1c1968, obj);
            self.f_1dc = 0;
        }
        g.list(L_3E1A4C).retain(|&x| x != obj as u32 + 1);
    }
    /// slot 40 FUN_001bc0d0(src): class-10 clone FUN_001e5db8 keeping +0x1d0..+0x1e8, new edge buffer
    pub fn ee_sub_1bc0d0(&mut self, obj: usize, src: &C2d4de0, h: &mut dyn ClassHost) {
        let keep = (self.f_1c4.clone(), self.f_1d8, self.f_1dc, self.f_1e8);
        self.base.ee_sub_1e5db8(obj, h);
        self.path = src.path.clone();
        (self.f_1c4, self.f_1d8, self.f_1dc, self.f_1e8) = keep;
        self.f_1ec = (self.path.as_ref().map_or(0, |p| p.pts.len()) * 0x10).saturating_sub(0x10);
    }
}

/// owner-chain ALO of `i` (REPLACED +0x10 -> +0x60)
fn alo_above(w: &World, i: usize) -> u32 {
    let mut a = w.objects[i].owner;
    for _ in 0..64 {
        let Some(k) = a else { return 0 };
        if alo_of(w, k).is_some() {
            return k as u32 + 1;
        }
        a = w.objects[k].owner;
    }
    0
}

#[derive(Clone, Debug)]
pub enum K11 {
    C2c9a80(C2c9a80),
    C2d4de0(C2d4de0),
}

impl ClassSet {
    /// K11: construct + init (slot 6) + load (local matrix of slot 13) of the `PORTED_K11` classes
    pub(super) fn build_k11(&mut self, w: &World, h: &mut dyn ClassHost) {
        for (i, o) in w.objects.iter().enumerate() {
            if !PORTED_K11.iter().any(|x| x.0 == o.class) {
                continue;
            }
            let src = o.clone_of.unwrap_or(i);
            if o.class == 14 {
                let mut c = C2d4de0::default();
                c.ee_sub_1bc050(i, h);
                c.ee_sub_1bc088(super::guard::rail_of(w, src));
                self.k11.push((i, Inst::K11(Box::new(K11::C2d4de0(c)))));
                continue;
            }
            let mut c = C2c9a80 { root_b3c: false, ..Default::default() };
            c.ee_sub_231100(i, alo_above(w, i), h);
            if let Some((_, m)) = xfm_of(w, src) {
                c.f_60 = from_4x3(&m);
                c.ee_sub_2313f0(i, h);
            }
            self.k11.push((i, Inst::K11(Box::new(K11::C2c9a80(c)))));
        }
    }
    /// K11 state at object index `obj`
    pub fn k11_get(&mut self, obj: usize) -> Option<&mut K11> {
        self.k11.iter_mut().find(|x| x.0 == obj).and_then(|x| if let Inst::K11(k) = &mut x.1 { Some(&mut **k) } else { None })
    }
    /// K11 load-post: slot 14, then slot 10 (world lists)
    pub(super) fn post_load_k11(&mut self, h: &mut dyn ClassHost) {
        for (i, x) in &mut self.k11 {
            if let Inst::K11(k) = x {
                if let K11::C2d4de0(c) = k.as_mut() {
                    c.ee_sub_1bc570(*i, h);
                    c.ee_sub_1bc160(*i, &mut self.g, h);
                }
            }
        }
    }
    /// K11 per frame: world matrices of the transform nodes (slot 30 FUN_00231440 consumers), class-14
    /// slot 16 FUN_001e6340
    pub(super) fn frame_k11(&mut self, h: &mut dyn ClassHost) {
        for (i, x) in &mut self.k11 {
            if let Inst::K11(k) = x {
                match k.as_mut() {
                    K11::C2c9a80(c) => c.ee_sub_231368(h),
                    K11::C2d4de0(c) => c.base.ee_sub_1e6340(*i, h),
                }
            }
        }
    }
}
