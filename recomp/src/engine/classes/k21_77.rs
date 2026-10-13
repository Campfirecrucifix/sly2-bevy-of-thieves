//! WP-K21: class 77 C2d7ca8 leftovers of K20 (classes/k20_77.rs keeps the state machine).
//! 1:1: the script methods FUN_001d4b48 (field 1419 add wheel, +0x190 / 0x60 records), FUN_001d4b90 (1420 add
//! part, +0x4f8 / 0xb0 C2c94a0 records), FUN_001d4c48 (1421 add tread, +0x4a4 / 0x14 records), FUN_001d4e30
//! (1418 part damage), FUN_001d3fa8 (getter 1422 part by id); FUN_001d4d30 (part defaults), FUN_001d4de0 (part
//! copy), slot 40 FUN_001d1410, slot 14 FUN_001d1518 (list resolve, wheel box, +0x4a0), slot 25 FUN_001d1910,
//! slot 18 FUN_001d41f8 (wheel spin: rate / angle / steer -> wheel matrix), FUN_001d2dd0 (deepest contact),
//! slots 48 FUN_001d4178 / 53 FUN_001d3f30 / 54 FUN_001d4050 / 59 FUN_001d3e48 (their SO bases and
//! per-wheel callees are hooks), and the VU tail of slot 15 FUN_001d19a8 (tread yaw, wheel steer, the part
//! ballistic aim +0x74 / +0x78 / +0xa0, engine sound volume / pitch) on the CPU.
//! REPLACED: bones (FUN_001f4c58(self, oid, kind)) are the object's ALO bone list (`Bone.word` = +0x9c,
//! `Bone.kind` = +0x94); a bone's +0x70 position in the object frame = its rest-pose world matrix
//! (`BoneWorlds`) taken into the object's root frame; the wheel bone matrix write (FUN_00256d18 + the
//! +0xa4 stamp) is kept on the record (`Wheel::mat`); the tread / part constraint targets FUN_0012d108 are
//! `ee_call(0x12d108, [xf, x, y, z])` plus the record (`Tread::ang`, part +0x74); the tread bone frame is the
//! object's world rotation; wheel contact lists (+8, filled by the unported FUN_001d3450) stay empty.
//! UNPORTED: slot 59's FUN_001d3450 (wheel contact generation: BSP clip), slot 69 FUN_001d2e18 (drive forces),
//! state 1 FUN_001d2610 (AI path follow; stays the `po_call(0x1d2610)` hook), Murray FUN_001b93e8.
#![allow(non_snake_case)]
use super::k20_77::C2d7ca8;
use super::*;
use crate::engine::camera::math::{FUN_0011d018, FUN_0011d2d0, FUN_0011d948, Smp};
use crate::engine::player::p5_attach::FUN_0011cf60;
use crate::engine::world::props::{ArrayItem, PropValue};
use crate::engine::world::World;

/// DAT_002c2098: wheel steer approach
pub const S_2C2098: Smp = [10.0, 0.0, 0.25, 400.0];
/// DAT_002c20a4: engine sound speed scale; DAT_002c20a8: its approach
pub const F_2C20A4: f32 = 400.0;
pub const S_2C20A8: Smp = [1.0, 0.0, 0.25, 0.0];
/// DAT_002c21a0: suspension approach; DAT_002c21b0 / 21c0: spin rate up / down; DAT_002c21cc lag factor
pub const S_2C21A0: Smp = [1500.0, 500.0, 0.1, 0.0];
pub const S_2C21B0: Smp = [20.0, 5.0, 0.2, 0.0];
pub const S_2C21C0: Smp = [5.0, 0.0, 0.2, 0.25];
pub const F_2C21CC: f32 = 0.25;
/// DAT_002a7834: -FLT_MAX
pub const F_2A7834: f32 = -f32::MAX;
/// FUN_0011d4d0 tolerance 0x3c23d70a
pub const EPS_AIM: f32 = 0.01;

/// one +0x1a0 wheel record (0x60 bytes)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Wheel {
    /// +0 bone oid, +4 bone (index + 1, 0 = none)
    pub oid: i32,
    pub bone: u32,
    /// +8 contact list (depths +0x90; filled by FUN_001d3450)
    pub contacts: Vec<f32>,
    /// +0xc / +0x10 position over the wheel box half extents
    pub fx: f32,
    pub fy: f32,
    /// +0x14 steer angle, +0x18 spin rate, +0x1c spin angle, +0x20 radius
    pub steer: f32,
    pub rate: f32,
    pub ang: f32,
    pub r: f32,
    /// +0x24 / +0x28 suspension range (bottom, top)
    pub lim: [f32; 2],
    /// +0x30 position, +0x40 top (+ up * +0x28), +0x50 bottom (+ up * +0x24)
    pub pos: [f32; 3],
    pub top: [f32; 3],
    pub bot: [f32; 3],
    /// REPLACED: suspension height of the FUN_0011d018 approach, the wheel bone rotation (rows)
    pub z: f32,
    pub mat: [[f32; 3]; 3],
}

/// one +0x4a8 tread record (0x14 bytes)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Tread {
    /// +0 bone oid, +4 bone, +8 constraint (FUN_0012dc18)
    pub oid: i32,
    pub bone: u32,
    pub xf: u32,
    /// +0xc yaw range
    pub lim: [f32; 2],
    /// REPLACED: last FUN_0012d108 yaw
    pub ang: f32,
}

/// the C2c94a0 part record words besides the K20 `Part` (+0x14 .. +0xa0)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PartRec {
    /// +0x10 owner, +0x14 id, +0x18 part oid, +0x2c bone oid, +0x30 bone, +0x34 constraint, +0x38 base oid,
    /// +0x3c base object
    pub owner: u32,
    pub id: i32,
    pub oid: i32,
    pub bone_oid: i32,
    pub bone: u32,
    pub xf: u32,
    pub base_oid: i32,
    pub base: u32,
    /// +0x40 pitch range, +0x48 speed range, +0x50 flight-time polynomial, +0x60 its clamp
    pub pitch: [f32; 2],
    pub speed: [f32; 2],
    pub poly: [f32; 4],
    pub tclamp: [f32; 2],
    /// +0x68 gravity, +0x74 aim pitch, +0x78 aim speed, +0x88 / 8c / 90 / 94 / 98 / 9c, +0xa0 on target
    pub g: f32,
    pub aim: f32,
    pub aim_speed: f32,
    pub f_88: f32,
    pub f_8c: f32,
    pub f_90: f32,
    pub f_94: f32,
    pub f_98: f32,
    pub f_9c: f32,
    pub on: u32,
}

/// FUN_001d4d30(part, owner): defaults
pub fn ee_sub_1d4d30(p: &mut PartRec, owner: u32) {
    p.owner = owner;
    p.pitch = [-1.57, 1.57];
    p.speed = [250.0, 2500.0];
    p.poly = [0.5, 0.001, 0.0, 0.0];
    p.tclamp = [0.5, 1.5];
    p.f_9c = 1.5;
    p.g = -1750.0;
    p.f_88 = 200.0;
    p.f_8c = 400.0;
    p.f_94 = 500.0;
    p.f_98 = 1.0;
    p.f_90 = 0.0;
}

/// FUN_001d4de0(dst, src): copy +0x14 .. +0xb0 (owner kept)
pub fn ee_sub_1d4de0(d: &mut PartRec, s: &PartRec) {
    let owner = d.owner;
    *d = s.clone();
    d.owner = owner;
}

/// FUN_001d2dd0(self, wheel): the deepest contact (+0x90 > -FLT_MAX), last of equals kept first
pub fn ee_sub_1d2dd0(c: &[f32]) -> Option<usize> {
    let mut best = F_2A7834;
    let mut r = None;
    for (k, &d) in c.iter().enumerate() {
        if best < d {
            r = Some(k);
            best = d;
        }
    }
    r
}

/// FUN_001f4c58(alo, oid, kind) over the ALO bone list: index + 1 of the first bone with +0x9c == oid and
/// +0x94 == kind
pub fn ee_sub_1f4c58(w: &World, obj: usize, oid: i32, kind: i32) -> u32 {
    let Some(a) = crate::engine::gfx::attach::alo_of(w, obj) else { return 0 };
    a.base.bones.iter().position(|b| b.word == oid && b.kind as i32 == kind).map_or(0, |k| k as u32 + 1)
}

/// FUN_001a6f00(0, self, bone +0x70): the bone position in the object root frame (rest pose)
pub fn bone_pos(w: &World, obj: usize, bone: u32) -> [f32; 3] {
    use crate::engine::gfx::scene::{invert, mul};
    let mut bw = crate::engine::gfx::attach::BoneWorlds::default();
    let b = bw.get(w, obj, bone as usize - 1);
    let root = bw.get(w, obj, 0);
    mul(&b, &invert(&root))[3]
}

fn rows3(m: &crate::engine::camera::math::M4) -> [[f32; 3]; 3] {
    [[m[0][0], m[0][1], m[0][2]], [m[1][0], m[1][1], m[1][2]], [m[2][0], m[2][1], m[2][2]]]
}
fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn one_i32(it: &ArrayItem) -> i32 {
    match it {
        ArrayItem::One(_, b) if b.len() >= 4 => i32::from_le_bytes([b[0], b[1], b[2], b[3]]),
        _ => 0,
    }
}
fn one_f32s(it: &ArrayItem) -> Vec<f32> {
    match it {
        ArrayItem::One(_, b) => b.chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect(),
        _ => vec![],
    }
}

impl C2d7ca8 {
    /// FUN_001d4b48(r, self, oid, &lim): add a wheel (max 8)
    pub fn ee_sub_1d4b48(&mut self, r: f32, oid: i32, lim: [f32; 2]) {
        if self.wheels.len() < 8 {
            self.wheels.push(Wheel { oid, r, lim, ..Default::default() });
        }
    }
    /// FUN_001d4b90(self, id, oid, bone oid, base oid): add a part (max 8)
    pub fn ee_sub_1d4b90(&mut self, obj: usize, id: i32, oid: i32, bone_oid: i32, base_oid: i32) {
        if self.parts.len() < 8 {
            let mut p = PartRec::default();
            ee_sub_1d4d30(&mut p, obj as u32 + 1);
            p.id = id;
            p.oid = oid;
            p.bone_oid = bone_oid;
            p.base_oid = base_oid;
            self.parts.push(Default::default());
            self.part_recs.push(p);
        }
    }
    /// FUN_001d4c48(self, oid, &lim): add a tread (max 4)
    pub fn ee_sub_1d4c48(&mut self, oid: i32, lim: [f32; 2]) {
        if self.treads.len() < 4 {
            self.treads.push(Tread { oid, lim, ..Default::default() });
        }
    }
    /// FUN_001d4e30(part): owner +0x574 + stage * 0xc -= +0x70 (>= 0)
    pub fn ee_sub_1d4e30(&mut self, k: usize) {
        let p = &self.parts[k];
        let off = 0x574 + p.stage as u32 * 0xc;
        self.m.set_i(off, (self.m.i(off) - p.thresh).max(0));
    }
    /// FUN_001d3fa8(self, id): the part with +0x14 == id
    pub fn ee_sub_1d3fa8(&self, id: i32) -> Option<usize> {
        self.part_recs.iter().position(|p| p.id == id)
    }
    fn rec(&mut self, k: usize) -> &mut PartRec {
        if self.part_recs.len() <= k {
            self.part_recs.resize(k + 1, PartRec::default());
        }
        &mut self.part_recs[k]
    }

    /// the field rows 1419 / 1420 / 1421 (script method calls of the load) in order
    pub fn k21_field_rows(&mut self, obj: usize, ps: &[crate::engine::world::props::Prop]) {
        for x in ps {
            let PropValue::Array(a) = &x.value else { continue };
            match (x.field, a.as_slice()) {
                (1419, [o, r, l]) => {
                    let l = one_f32s(l);
                    self.ee_sub_1d4b48(f32::from_bits(one_i32(r) as u32), one_i32(o), [l[0], l[1]]);
                }
                (1420, [a, b, c, d]) => self.ee_sub_1d4b90(obj, one_i32(a), one_i32(b), one_i32(c), one_i32(d)),
                (1421, [o, l]) => {
                    let l = one_f32s(l);
                    self.ee_sub_1d4c48(one_i32(o), [l[0], l[1]]);
                }
                _ => {}
            }
        }
    }

    /// slot 40 FUN_001d1410 (after FUN_001c3df8 = `h.base`): each part re-made from the template's record
    pub fn ee_sub_1d1410(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1c3df8, obj);
        for k in 0..self.parts.len() {
            let src = self.rec(k).clone();
            let mut p = PartRec::default();
            ee_sub_1d4d30(&mut p, obj as u32 + 1);
            ee_sub_1d4de0(&mut p, &src);
            self.part_recs[k] = p;
        }
    }

    /// slot 14 FUN_001d1518 (after FUN_001f36a8 = `h.base`): resolve the wheel / tread / part lists (unresolved
    /// entries removed), the wheel box (+0x4a0 = x extent, +0xc / +0x10 per wheel), the +0x5ac path, the +0x570
    /// sm, then FUN_001d14c0
    pub fn ee_sub_1d1518(&mut self, obj: usize, w: Option<&World>, t: f32, h: &mut dyn ClassHost) {
        h.base(0x1f36a8, obj);
        // REPLACED: a pool copy (clone) has no bones / children of its own: they are its template's
        let obj0 = obj;
        let obj = w.and_then(|w| w.objects[obj].clone_of).unwrap_or(obj);
        let o = obj as u32 + 1;
        let mut lo = [0f32; 3];
        let mut hi = [0f32; 3];
        let mut k = 0;
        while k < self.wheels.len() {
            let b = w.map_or_else(|| h.ee_sub_1f4c58(o, self.wheels[k].oid, 0xd), |w| ee_sub_1f4c58(w, obj, self.wheels[k].oid, 0xd));
            self.wheels[k].bone = b;
            if b == 0 {
                self.wheels.remove(k);
                continue;
            }
            let p = w.map_or([0.0; 3], |w| bone_pos(w, obj, b));
            let wh = &mut self.wheels[k];
            wh.pos = p;
            for i in 0..3 {
                lo[i] = lo[i].min(p[i]);
                hi[i] = hi[i].max(p[i]);
            }
            wh.top = [p[0], p[1], p[2] + wh.lim[1]];
            wh.bot = [p[0], p[1], p[2] + wh.lim[0]];
            wh.z = wh.bot[2];
            k += 1;
        }
        self.m.set_f(0x4a0, hi[0] - lo[0]);
        for wh in &mut self.wheels {
            wh.fx = wh.pos[0] / hi[0].max(-lo[0]);
            wh.fy = wh.pos[1] / hi[1].max(-lo[1]);
        }
        let mut k = 0;
        while k < self.treads.len() {
            let b = w.map_or_else(|| h.ee_sub_1f4c58(o, self.treads[k].oid, 6), |w| ee_sub_1f4c58(w, obj, self.treads[k].oid, 6));
            self.treads[k].bone = b;
            if b == 0 {
                self.treads.remove(k);
                continue;
            }
            self.treads[k].xf = h.ee_sub_12dc18(o, b, [0.0; 3]);
            k += 1;
        }
        let mut k = 0;
        while k < self.parts.len() {
            let mut p = self.rec(k).clone();
            // +0x1c part object, +0x30 bone, +0x3c base object, +0x20 the part sm, +0x34 constraint
            let po = h.ee_sub_15e938(1, p.oid, o);
            self.parts[k].obj = po;
            let mut ok = po != 0;
            if ok {
                p.bone = w.map_or_else(|| h.ee_sub_1f4c58(o, p.bone_oid, 6), |w| ee_sub_1f4c58(w, obj, p.bone_oid, 6));
                ok = !(p.bone == 0 && p.bone_oid != -1);
            }
            if ok {
                p.base = h.ee_sub_15e938(1, p.base_oid, o);
                ok = !(p.base == 0 && p.base_oid != -1);
            }
            if ok {
                self.parts[k].sm = h.ee_sub_1f0380_link(po, o, -1, 0);
                if p.bone != 0 {
                    p.xf = h.ee_sub_12dc18(o, p.bone, [0.0; 3]);
                }
            }
            if !ok {
                self.parts.remove(k);
                self.part_recs.remove(k);
                continue;
            }
            self.part_recs[k] = p;
            k += 1;
        }
        let o = obj0 as u32 + 1;
        self.m.set_u(0x5ac, h.ee_call(0x1e94b0, &[0x2c9, 0x10]) as u32);
        let sm = h.ee_sub_1f1448(self.m.u(0x56c), o, u32::MAX);
        if sm != 0 {
            self.m.set_u(0x570, sm);
        }
        self.ee_sub_1d14c0(obj0, t, h);
    }

    /// slot 25 FUN_001d1910: FUN_001cb518 (`h.base`) then FUN_001d14c0
    pub fn ee_sub_1d1910(&mut self, obj: usize, t: f32, h: &mut dyn ClassHost) {
        h.base(0x1cb518, obj);
        self.ee_sub_1d14c0(obj, t, h);
    }

    /// slot 18 FUN_001d41f8: per wheel the suspension approach, the spin rate (from the forward speed `vx` with
    /// a contact, else towards +0x550 / radius for the rear wheels), the spin angle and the wheel matrix
    /// rot(steer, z) * rot(angle, y); then FUN_0022faa0 / FUN_001f3c70 (`h.base`)
    pub fn ee_sub_1d41f8(&mut self, obj: usize, vx: f32, dt: f32, h: &mut dyn ClassHost) {
        use crate::engine::camera::math::FUN_0011b500;
        let speed = self.m.f(0x550);
        for (i, wh) in self.wheels.iter_mut().enumerate() {
            let c = ee_sub_1d2dd0(&wh.contacts);
            match c {
                None => wh.z = FUN_0011d018(wh.z, wh.bot[2], dt, &S_2C21A0).0,
                Some(k) => {
                    let tz = wh.bot[2] + wh.contacts[k];
                    if tz <= wh.z {
                        wh.z = FUN_0011d018(wh.z, tz, dt, &S_2C21A0).0;
                    } else {
                        wh.z = tz;
                    }
                }
            }
            let target = if i < 2 { 0.0 } else { speed / wh.r };
            match c {
                None => {
                    let s = if wh.rate.abs() < target.abs() { &S_2C21B0 } else { &S_2C21C0 };
                    wh.rate = FUN_0011d018(wh.rate, target, dt, s).0;
                }
                Some(_) => {
                    let v = vx / wh.r;
                    if (0.0 < target && v < F_2C21CC * target) || (target < 0.0 && F_2C21CC * target < v) {
                        wh.rate = FUN_0011d018(wh.rate, F_2C21CC * target, dt, &S_2C21B0).0;
                    } else {
                        wh.rate = v;
                    }
                }
            }
            wh.ang = FUN_0011cf60(wh.ang + wh.rate * dt);
            let a = rows3(&FUN_0011b500(wh.steer, [0.0, 0.0, 1.0]));
            let b = rows3(&FUN_0011b500(wh.ang, [0.0, 1.0, 0.0]));
            let mut m = [[0f32; 3]; 3];
            for r in 0..3 {
                for k in 0..3 {
                    m[r][k] = b[r][0] * a[0][k] + b[r][1] * a[1][k] + b[r][2] * a[2][k];
                }
            }
            wh.mat = m;
        }
        h.base(0x22faa0, obj);
        h.base(0x1f3c70, obj);
    }

    /// slot 48 FUN_001d4178: FUN_001f50e0 then FUN_001f5480(wheel bone, self, bottom in world) per wheel
    pub fn ee_sub_1d4178(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1f50e0, obj);
        for k in 0..self.wheels.len() {
            let p = h.ee_sub_1a6f00(obj, self.wheels[k].bot);
            let b = self.wheels[k].bone;
            h.ee_call(0x1f5480, &[b, obj as u32 + 1, p[0].to_bits(), p[1].to_bits(), p[2].to_bits()]);
        }
    }
    /// slot 53 FUN_001d3f30(self, x): FUN_001f5988 then FUN_001f4da8(self, x, &wheel +8) per wheel
    pub fn ee_sub_1d3f30(&mut self, obj: usize, x: u32, h: &mut dyn ClassHost) {
        h.base(0x1f5988, obj);
        for k in 0..self.wheels.len() {
            h.ee_call(0x1f4da8, &[obj as u32 + 1, x, k as u32]);
        }
    }
    /// slot 54 FUN_001d4050(self, contact, side): `n` = the contact normal (negated for side 1), `surf` the
    /// other surface; +0x94 friction 0 on material 0x4d or n.z < 0.7, else 0.5; +0x98 = 0.1; material 0x6c
    /// sends FUN_00149ab8. Returns (+0x94, +0x98).
    pub fn ee_sub_1d4050(&mut self, obj: usize, n: [f32; 3], side: i32, surf: u32, h: &mut dyn ClassHost) -> (f32, f32) {
        h.base(0x1f4fb0, obj);
        let n = if side == 0 { n } else { [-n[0], -n[1], -n[2]] };
        let r = if h.ee_call(0x132b10, &[surf, 0x4d]) != 0 || n[2] < 0.7 { (0.0, 0.1) } else { (0.5, 0.1) };
        if h.ee_call(0x132b10, &[surf, 0x6c]) != 0 {
            h.ee_call(0x149ab8, &[0x3dd490, obj as u32 + 1, 0xffff0000, surf]);
        }
        r
    }
    /// slot 59 FUN_001d3e48: FUN_001f5990 then, when the other body is solid (vtable +500), FUN_001d3450 per
    /// wheel (UNPORTED: `po_call(0x1d3450, obj, wheel)`)
    pub fn ee_sub_1d3e48(&mut self, obj: usize, solid: bool, h: &mut dyn ClassHost) {
        h.base(0x1f5990, obj);
        if solid {
            for k in 0..self.wheels.len() {
                h.po_call(0x1d3450, obj, k as u32, &[]);
            }
        }
    }

    /// the VU tail of slot 15 FUN_001d19a8: tread yaw toward +0x54c, wheel steer approach, part aim at the
    /// +0x5a4 target (`me` / `tgt` positions, `tv` its velocity), engine sound, physics wake (FUN_001f5078)
    pub fn ee_sub_1d19a8_tail(&mut self, obj: usize, rot: &[[f32; 3]; 3], me: [f32; 3], tgt: [f32; 3], tv: [f32; 3], dt: f32, h: &mut dyn ClassHost) {
        use crate::engine::physics::driven::FUN_0022fcc8;
        let o = obj as u32 + 1;
        for k in 0..self.treads.len() {
            let d = crate::engine::player::FUN_0011e1a0(self.m.f(0x54c), 1.0, 0.0);
            let l = FUN_0022fcc8(None, Some(rot), d);
            let a = FUN_0011d948(l[1].atan2(l[0]), self.treads[k].lim);
            self.treads[k].ang = a;
            h.ee_call(0x12d108, &[self.treads[k].xf, 0, 0, a.to_bits()]);
        }
        let speed = self.m.f(0x550);
        let steer = self.m.f(0x548);
        for wh in &mut self.wheels {
            let t = if speed == 0.0 { 0.0 } else { wh.fx * steer };
            wh.steer = FUN_0011d2d0(wh.steer, t, dt, &S_2C2098);
        }
        let dist = len3([tgt[0] - me[0], tgt[1] - me[1], tgt[2] - me[2]]);
        for k in 0..self.parts.len() {
            let part = self.parts[k].clone();
            let base_pos = if self.rec(k).base != 0 { h.ee_sub_1a62e8(self.rec(k).base) } else { me };
            let ok = self.ee_sub_1d4e08(&part);
            let p = self.rec(k);
            if p.bone == 0 {
                continue;
            }
            if speed == 0.0 && ok {
                let ft = p.poly[0] + dist * (p.poly[1] + dist * p.poly[2]);
                let ft = FUN_0011d948(ft, p.tclamp);
                let d = [tgt[0] - base_pos[0], tgt[1] - base_pos[1], tgt[2] - base_pos[2]];
                let hv = [tv[0], tv[1], 0.0];
                let q = p.g * ft * -0.5;
                let l = [hv[0] + d[0] / ft, hv[1] + d[1] / ft, d[2] / ft + q];
                let pitch = l[2].atan2((l[0] * l[0] + l[1] * l[1]).sqrt());
                let sp = len3(l);
                p.aim = FUN_0011d948(pitch, p.pitch);
                p.aim_speed = FUN_0011d948(sp, p.speed);
                let on = crate::engine::physics::driver::FUN_0011d4d0(p.aim, pitch, EPS_AIM)
                    && crate::engine::physics::driver::FUN_0011d4d0(p.aim_speed, sp, EPS_AIM);
                p.on = on as u32;
                if !on {
                    p.aim = 0.0;
                    p.aim_speed = p.speed[0];
                }
            } else {
                p.on = 0;
                p.aim_speed = p.speed[0];
                p.aim = 0.0;
            }
            let (xf, a) = (p.xf, -p.aim);
            h.ee_call(0x12d108, &[xf, 0, a.to_bits(), 0]);
        }
        let snd = self.m.u(0x598);
        let mut wake = speed;
        if snd != 0 {
            let v = FUN_0011d018(self.m.f(0x5a0), speed / F_2C20A4, dt, &S_2C20A8).0;
            self.m.set_f(0x5a0, v);
            let v = v.clamp(0.2, 1.0);
            h.ee_call(0x1fb1f8, &[v.to_bits(), snd]);
            h.ee_call(0x1fb2c0, &[(v * 0.2).to_bits(), snd]);
            if (h.obj_i32(snd, 0x4c) >> 16) as i16 >= 1 {
                h.ee_call(0x1fb060, &[snd]);
                wake = self.m.f(0x550);
            }
        }
        if wake != 0.0 || self.m.f(0x554) != 0.0 {
            h.ee_call(0x1f5078, &[o]);
        }
    }
}

/// K21 class 77 in the ClassSet: field rows at build, slot 14 at load-post, slots 18 + the 15 tail per frame
impl ClassSet {
    pub(super) fn build_k21_77(&mut self, w: &World) {
        for (i, x) in &mut self.k20 {
            let Inst::K20Truck(c) = x else { continue };
            let src = w.objects[*i].clone_of.unwrap_or(*i);
            if let Some(ps) = crate::engine::game::fields::object_props(&w.objects[src]) {
                c.k21_field_rows(*i, ps);
            }
        }
    }
    pub fn k21_77_slot14(&mut self, h: &mut dyn ClassHost) {
        let t = self.g.t;
        let w = h.k21_world();
        for (i, x) in &mut self.k20 {
            if let Inst::K20Truck(c) = x {
                c.ee_sub_1d1518(*i, w.as_deref(), t, h);
            }
        }
    }
}
