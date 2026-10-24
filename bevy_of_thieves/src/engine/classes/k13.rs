//! WP-K13: class 81 C2d3d20 (projectile, parent C2d3aa0 class 82 -> C2cb780 class 79): own slots 5
//! FUN_00113f20, 6 FUN_001b2070, 69 FUN_001b20b8 (speed hold), 77 FUN_001b1e88 (launch) 1:1 with their
//! callees in classes/shared/ (FUN_0013a9e8 / FUN_001b1db8 init cores, FUN_001305b8 anim, FUN_0012dca0 act).
//! Fed from rows 383-399 (class 79) + 400-409 (class 81). K15: the inherited class-80 (the real parent, not
//! C2d3aa0 = class 82) slots 11 / 13 / 60 / 76 and class-79 slots 4 / 14 / 15 / 25 / 57 / 59 / 74 / 75 are
//! ported on `Proj` in classes/k15_79.rs; load-post runs 13 / 14, frame_k13 runs 57 / 15 (in the world) and
//! the deferred slot 76.
//! Slot 69 replaces ALO slot 69 FUN_001f9b20 (gravity) inside the E5 physics part FUN_001f3980, run per frame
//! here on the instance's `AloPhys`. REPLACED: slot 33 velocity FUN_001f4558 (vtable +0x10c) = xfLin v
//! (top-level bodies, identity parent frame as in physics::driver).
//! Class 89 C2dca30 (water volume, ALO child): own slots 4 FUN_00229370 (msg 1 = contact gone -> drop its
//! record, msg 5 {water, obj} to it), 5 FUN_00113fa0, 6 FUN_00228b48, 11 FUN_00229308 (remove from world:
//! drop all records), 14 FUN_00228bf8 (= base FUN_001f36a8), 48 FUN_00229680 (= base FUN_001f50e0 ALO
//! bounds over the collision parts: SO core hook), 63 FUN_00228bb0 (physics block: FUN_001f3590 + parts bit 48
//! FUN_001f68a8 + e8 +0xd4 / +0xd8 = 1), 70 FUN_002292d8 (record obj vtable +0x244), 72 FUN_00229d78 (= the
//! P8-ported point-in-volume FUN_001f72f8 on `p8_hazard::Water`) 1:1. Slot 15 FUN_00228e50 (contact walk,
//! adds the +0x150 records + msg 4) stays the player-side REPLACED `p8_hazard::water_contacts`.
//! REPLACED: +0x150 record chain {+4 obj, +8 next in obj +0x138, +0xc next} -> `contacts` (obj handles);
//! record pool free FUN_002296c0 / FUN_0011cd98 (0x3dd490 +0x40e8) -> Vec removal; the obj-side unlink
//! FUN_00229738 (obj +0x138 list + FUN_001f5078 wake) is `ClassHost::ee_sub_229738` (state of the other object).
use super::shared::{self, act, proj::Proj};
use super::*;
use crate::engine::physics::driver::{FUN_001fa810, FUN_0022de88, FUN_00230380};
use crate::engine::physics::dynamics::FUN_0011d018;

/// K13 world class id -> vtable (kept in `ClassSet::k13`).
pub const PORTED_K13: &[(ClassId, u32)] = &[(81, 0x2d3d20), (89, 0x2dca30)];

/// slot 5 FUN_00113f20 (81) / FUN_00113fa0 (89) (class descriptor getters)
pub fn slot5(class: ClassId) -> Option<u32> {
    match class {
        81 => Some(0x2bb13c),
        89 => Some(0x2bb1dc),
        _ => None,
    }
}

/// class 89 field rows 874-881 -> member offsets (FIELD_MEMBER)
pub const ROWS_89: &[(i16, u16)] = &[(874, 0x170), (875, 0x174), (876, 0x184), (877, 0x188), (878, 0x160), (879, 0x180), (880, 0x178), (881, 0x17c)];

/// class 89 C2dca30 water volume
#[derive(Clone, Debug, Default)]
pub struct C2dca30 {
    /// ALO +0x40 flag word (inline)
    pub f_40: u32,
    /// +0x160, +0x170 / +0x174 (splash / ripple emitter oids), +0x178, +0x17c, +0x180 (1), +0x184 (-10 hit
    /// amount), +0x188 (0x2000 hit kind)
    pub f_160: u32,
    pub f_170: u32,
    pub f_174: u32,
    pub f_178: u32,
    pub f_17c: u32,
    pub f_180: u32,
    pub f_184: i32,
    pub f_188: u32,
    /// +0x150 contact records (the record's +4 object handle), head first
    pub contacts: Vec<u32>,
    /// collision parts +0x10 bit 48 (FUN_001f68a8(self, 1) of slot 63), e8 +0xd4 / +0xd8
    pub parts_bit48: bool,
    pub e8_d4: f32,
    pub e8_d8: f32,
    /// slot 72 volume (P8 `Water`: transformed parts)
    pub vol: Option<crate::engine::player::p8_hazard::Water>,
}

impl C2dca30 {
    /// slot 6 FUN_00228b48: base FUN_001f2108; +0x180 = 1, +0x184 = -10, +0x188 = 0x2000, +0x40 |= 2
    pub fn ee_sub_228b48(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1f2108, obj);
        self.f_180 = 1;
        self.f_184 = -10;
        self.f_188 = 0x2000;
        self.f_40 |= 2;
    }
    /// slot 63 FUN_00228bb0: base FUN_001f3590; FUN_001f68a8(self, 1) (bit 48 of every collision part);
    /// e8 +0xd4 = +0xd8 = 1.0
    pub fn ee_sub_228bb0(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1f3590, obj);
        self.parts_bit48 = true;
        if let Some(v) = &mut self.vol {
            for p in &mut v.parts {
                p.flags |= 1 << 48;
            }
        }
        self.e8_d4 = 1.0;
        self.e8_d8 = 1.0;
    }
    /// slot 14 FUN_00228bf8 = FUN_001f36a8 (ALO load-post, SO core hook)
    pub fn ee_sub_228bf8(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1f36a8, obj);
    }
    /// slot 48 FUN_00229680 = FUN_001f50e0 (ALO bounds over the collision parts, SO core hook)
    pub fn ee_sub_229680(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1f50e0, obj);
    }
    /// slot 72 FUN_00229d78 = FUN_001f72f8: world point inside the water
    pub fn ee_sub_229d78(&self, p: [f32; 3]) -> bool {
        self.vol.as_ref().is_some_and(|v| v.FUN_001f72f8(p))
    }
    /// slot 70 FUN_002292d8(self, rec): (rec obj vtable +0x244)(rec obj, self)
    pub fn ee_sub_2292d8(&mut self, obj: usize, rec_obj: u32, h: &mut dyn ClassHost) {
        h.vcall_244(rec_obj, handle(obj));
    }
    /// slot 4 FUN_00229370(self, msg, other): msg 1 from another object -> unlink its record (FUN_00229738,
    /// free), then message 5 {self, other} to other (vtable +0x24); base FUN_00133590 is empty.
    /// Quirk: with no record for `other` the original reads the null record (+4) and calls through it; the
    /// port sends nothing then.
    pub fn ee_sub_229370(&mut self, obj: usize, msg: i32, other: u32, h: &mut dyn ClassHost) {
        if msg == 1 && other != handle(obj) {
            if let Some(k) = self.contacts.iter().position(|&o| o == other) {
                self.contacts.remove(k);
                h.ee_sub_229738(other, handle(obj));
                h.vcall_24(other as usize - 1, 5, handle(obj));
            }
        }
    }
    /// slot 11 FUN_00229308: base FUN_001f2d48; every record unlinked from its object (FUN_00229738), the
    /// chain freed (FUN_002296c0), +0x150 = 0
    pub fn ee_sub_229308(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1f2d48, obj);
        for o in std::mem::take(&mut self.contacts) {
            h.ee_sub_229738(o, handle(obj));
        }
    }
    /// member write by offset (raw prop word)
    pub fn set(&mut self, off: u16, v: u32) {
        match off {
            0x160 => self.f_160 = v,
            0x170 => self.f_170 = v,
            0x174 => self.f_174 = v,
            0x178 => self.f_178 = v,
            0x17c => self.f_17c = v,
            0x180 => self.f_180 = v,
            0x184 => self.f_184 = v as i32,
            0x188 => self.f_188 = v,
            _ => {}
        }
    }
}

fn handle(obj: usize) -> u32 {
    obj as u32 + 1
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct C2d3d20 {
    pub p: Proj,
}

impl C2d3d20 {
    /// slot 6 FUN_001b2070: FUN_001b1db8; +0x270 = 2000, +0x274 = 5000, +0x264 = 0
    pub fn ee_sub_1b2070(&mut self, obj: usize, h: &mut dyn ClassHost) {
        self.p.ee_sub_1b1db8(obj, h);
        self.p.f_270 = 2000.0;
        self.p.f_274 = 5000.0;
        self.p.f_264 = 0;
    }

    /// slot 69 FUN_001b20b8: while +0x1e8 == 0 accelerate along the velocity so the speed approaches +0x270
    /// (FUN_0011d018 trapezoid {+0x274, 0, 1}): a = (dir * s' - v) / dt, projected by the +0xe8 constraint
    /// (FUN_001fa810) and added to xfLin (FUN_001fac30). No gravity (the ALO slot 69 is not chained).
    pub fn ee_sub_1b20b8(&mut self, dt: f32) {
        let p = &mut self.p;
        if p.f_1e8 != 0 {
            return;
        }
        let v = p.phys.v;
        let s = shared::len3(v);
        let q = shared::vu_rcp(s);
        let dir = [shared::vu_mul(v[0], q), shared::vu_mul(v[1], q), shared::vu_mul(v[2], q)];
        let s2 = FUN_0011d018(s, p.f_270, dt, [p.f_274, 0.0, 1.0], None);
        let r = shared::vu_rcp(dt);
        let a = [0, 1, 2].map(|k| shared::vu_mul(dir[k] * s2 - v[k], r));
        let a = FUN_001fa810(p.phys.ckind, p.phys.cnorm, a);
        // FUN_001fac30: xfLin accel += a for dynamic xf kinds
        if FUN_00230380(p.phys.kind) {
            p.phys.a = [p.phys.a[0] + a[0], p.phys.a[1] + a[1], p.phys.a[2] + a[2]];
        }
    }

    /// slot 77 FUN_001b1e88(launcher, v): +0x210 = launcher, +0x220 = v, (re)play anim +0x230 into +0x234
    /// (FUN_001305b8(0, 1.0, .., 8, ..)); with +0x264 a kind-4 act record (FUN_0012dca0(+0x26c, 0, self, +0x5c,
    /// 0, 5)) goes on the xfAng list -> +0x268
    pub fn ee_sub_1b1e88(&mut self, obj: usize, launcher: u32, v: [f32; 4], pool: &mut act::ActPool, h: &mut dyn ClassHost) {
        self.p.ee_sub_1b1e88(obj, launcher, v, pool, h);
    }

    /// FUN_001f3980 physics part with this class's slot 69: previous velocities, accel reset + slot 69 for
    /// dynamic kinds, then FUN_0022de88
    pub fn physics(&mut self, obj: usize, dt: f32, h: &mut dyn ClassHost) {
        h.base(0x1f3980, obj);
        let b = &mut self.p.phys;
        b.prev_v = b.v;
        b.prev_w = b.w;
        if matches!(b.kind, 1 | 2) {
            b.a = [0.0; 3];
            b.alpha = [0.0; 3];
            self.ee_sub_1b20b8(dt);
        }
        FUN_0022de88(&mut self.p.phys, dt);
    }
}

impl ClassSet {
    /// K13: construct + init (slot 6) + feed the `PORTED_K13` classes (clones read their source's props);
    /// xfLin starts at the object's rest world origin, at rest
    pub(super) fn build_k13(&mut self, w: &World, h: &mut dyn ClassHost) {
        let mut bw = crate::engine::gfx::attach::BoneWorlds::default();
        let mut waters = None;
        for (i, o) in w.objects.iter().enumerate() {
            if !PORTED_K13.iter().any(|x| x.0 == o.class) {
                continue;
            }
            let src = super::clone_src(w, i);
            let props = w.objects.get(src).and_then(crate::engine::game::fields::object_props);
            if o.class == 89 {
                let mut c = C2dca30::default();
                c.ee_sub_228b48(i, h);
                for p in props.unwrap_or(&[]) {
                    let crate::engine::world::props::PropValue::Scalar(b) = &p.value else { continue };
                    if let Some(&(_, off)) = ROWS_89.iter().find(|r| r.0 == p.field) {
                        c.set(off, super::k4::word(b));
                    }
                }
                let hz = waters.get_or_insert_with(|| crate::engine::player::p8_hazard::Hazards::load(w));
                c.vol = hz.waters.iter().find(|x| x.obj == i).cloned();
                c.ee_sub_228bb0(i, h);
                self.k13.push((i, Inst::K13Water(Box::new(c))));
                continue;
            }
            let mut c = C2d3d20::default();
            c.ee_sub_1b2070(i, h);
            if let Some(props) = props {
                c.p.feed(props, shared::proj::ROWS_81);
            }
            // POOL1: row 394 +0x168 (message-9 target, e.g. the dart's explode_dart_gas emitter) is an oid
            // resolved like the K15 projectiles (FUN_0015e938 mode 0x2002)
            if c.p.f_168 != 0 {
                c.p.f_168 = h.ee_sub_15e938(0x2002, c.p.f_168 as i32, handle(i));
            }
            c.p.phys.pos = bw.get(w, i, 0)[3];
            c.p.phys.gravity = [0.0; 3];
            c.p.xf_ang = act::XfActs { xf: handle(i) << 8 | 0x5c, kind_98: 2, ..Default::default() };
            self.k13.push((i, Inst::K13(Box::new(c))));
        }
    }
    /// K13 class-89 water at object index `obj`
    pub fn k13_water(&mut self, obj: usize) -> Option<&mut C2dca30> {
        self.k13.iter_mut().find(|x| x.0 == obj).and_then(|x| if let Inst::K13Water(c) = &mut x.1 { Some(&mut **c) } else { None })
    }
    /// K13 load-post: class 89 slot 14
    pub(super) fn post_load_k13(&mut self, h: &mut dyn ClassHost) {
        for (i, x) in &mut self.k13 {
            if let Inst::K13Water(c) = x {
                c.ee_sub_228bf8(*i, h);
            }
            // K15: inherited class-80 slot 13 / class-79 slot 14
            if let Inst::K13(c) = x {
                c.p.ee_sub_1b1e00(*i, h);
                c.p.ee_sub_13aaa0(*i, h);
            }
        }
    }
    /// K13 class 81 state at object index `obj`
    pub fn k13_get(&mut self, obj: usize) -> Option<&mut C2d3d20> {
        self.k13.iter_mut().find(|x| x.0 == obj).and_then(|x| if let Inst::K13(c) = &mut x.1 { Some(&mut **c) } else { None })
    }
    /// K13 slot 77 (launch) of object `obj`
    pub fn k13_launch(&mut self, obj: usize, launcher: u32, v: [f32; 4], h: &mut dyn ClassHost) -> bool {
        let Some(k) = self.k13.iter().position(|x| x.0 == obj) else { return false };
        let Inst::K13(c) = &mut self.k13[k].1 else { return false };
        c.ee_sub_1b1e88(obj, launcher, v, &mut self.g.act_pool, h);
        true
    }
    /// K13 per frame: the FUN_001f3980 physics part (slot 69)
    pub(super) fn frame_k13(&mut self, h: &mut dyn ClassHost) {
        let (t, dt) = (self.g.t, self.g.dt);
        let mut deferred = Vec::new();
        for (i, x) in &mut self.k13 {
            if let Inst::K13(c) = x {
                // K15: the inherited class-79 slots 57 / 15 (classes/k15_79.rs) for objects in the world
                if c.p.in_play || h.in_world_28(handle(*i)) {
                    let mut cx = super::k15_79::Cx { t, dt, h: &mut *h };
                    if let Some(o) = super::k15_79::frame_proj(&mut c.p, *i, &mut cx) {
                        deferred.push((*i, o));
                    }
                }
                c.physics(*i, dt, h);
            }
        }
        // K15: deferred class-80 slot 76 (FUN_001b1f10)
        for (obj, other) in deferred {
            if let Some(c) = self.k13_get(obj) {
                c.p.ee_sub_1b1f10(obj, other, h);
            }
        }
    }
}
