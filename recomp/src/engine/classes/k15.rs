//! WP-K15: class 73 C2d1080 (parent class 75 C2d0dd8 -> class-71 code in shared/c71.rs): a chaser that keeps a
//! target object (+0x1d4, row 1379) in view. States 0..4 (+0x1c0): 0 idle, 1 / 2 turn to face the target, 3
//! chase it over the nav mesh (+0x1d0) along a scratch path (+0x1d8), 4 hit / dying. Each state has an anim
//! record (+0x220, 0x10: anim oid, its SM (class 0x6d) and sound (class 0x6f) objects, flags; method row 1375
//! FUN_00181308) and a distance / angle range record (+0x270, 0x28: enter ranges +0x8 / +0x18, stay ranges
//! +0x10 / +0x20; method rows 1380 / 1381 FUN_00181330 / FUN_00181378). Per frame (slot 15) the state is kept
//! while the target stays inside the current state's stay ranges, else the highest state 4..1 whose enter
//! ranges hold is chosen (none -> 0).
//! Own slots 4 FUN_00181060, 5 FUN_00113ea0, 6 FUN_00180500, 11 FUN_00180598, 14 FUN_001805c8, 15
//! FUN_00180668, 25 FUN_00180638, 54 FUN_00180ff0, 69 FUN_00180ee8, 74 FUN_001811b8, 82 FUN_00181238 1:1 with
//! FUN_00181268 (state), FUN_001813c0 / 1813f0, FUN_00181308 / 181330 / 181378, FUN_0017f3e0 / 17f408 /
//! 17f4c0 / 17f550, FUN_0011d910; class-71 slot 74 part FUN_0017f1d0 and inherited slot 81 FUN_0017f5f8 in
//! shared/c71.rs.
//! REPLACED: the nav-mesh path search FUN_001c02d0(+0x1d0, ..) -> a straight 2-point path self -> target into
//! the scratch path (FUN_001e94b0 class-10 object -> `Path`); FUN_001e7380 (path walk from +0x1dc) ->
//! FUN_001e6ed8; the FUN_0015e9c8(0x105, class 0x49) neighbour query -> the ported class-73 instances; the
//! FUN_00154c90(0x2de578) factor (BIF 103) -> `ClassHost::ee_sub_154c90`; the third argument of the speed
//! heading FUN_001a6fc0(0, self, ?) is lost in the decompile -> the chase direction (assumption).
//! Slot 4 (anim event 9) takes the FUN_00204e38 root-channel velocity from the caller.
#![allow(non_snake_case)]
use super::shared::c71::{self, C71};
use super::*;
use crate::engine::camera::math::FUN_0011cf60;
use crate::engine::physics::driver::{AloPhys, FUN_0011b500, FUN_0022de88, FUN_00230380, M3};
use crate::engine::player::p5_attach::{Path, FUN_001e6648, FUN_001e6ed8};
use crate::engine::player::p5_states::FUN_001f59a0;
use crate::engine::player::p7_damage::FUN_0011e9f8;

/// K15 world class id -> vtable (kept in `ClassSet::k15`).
pub const PORTED_K15: &[(ClassId, u32)] = &[(73, 0x2d1080), (101, 0x2c9810)];

/// slot 5 FUN_00113ea0 (class descriptor getter)
pub fn slot5(class: ClassId) -> Option<u32> {
    match class {
        73 => Some(0x2bb09c),
        _ => None,
    }
}

/// class 73 field rows -> member offsets (FIELD_MEMBER; 1375 / 1380 / 1381 are method thunks, 1376 / 1377 no
/// member)
pub const ROWS_73: &[(i16, u16)] = &[(1378, 0x1d0), (1379, 0x1d4), (1382, 0x338), (1383, 0x340), (1384, 0x350), (1385, 0x360), (1386, 0x370)];

/// DAT_002be820 replan distance, DAT_002be824 replan period, DAT_002be828 look-ahead
pub const D_2BE820: f32 = 100.0;
pub const D_2BE824: f32 = 5.0;
pub const D_2BE828: f32 = 200.0;
/// 0x2be830 BIF-103 speed polynomial, 0x2be840 repulsion distance polynomial (clamped to 0x2be850),
/// 0x2be860 repulsion heading polynomial
pub const P_2BE830: [f32; 3] = [1.0, -0.5, 0.0];
pub const P_2BE840: [f32; 3] = [1500.0, -5.0, 0.0];
pub const C_2BE850: [f32; 2] = [0.0, 1000.0];
pub const P_2BE860: [f32; 3] = [0.0, -1.0, 0.0];
/// 0x2be870..0x2be87c: angular servo {25, 0, 0.25} and linear gain 5 (scaled by FUN_001813c0 / 1813f0)
pub const K_2BE870: [f32; 4] = [25.0, 0.0, 0.25, 5.0];
/// DAT_002bbb60 / DAT_002bbb64: [0, 1] clamp
pub const C_2BBB60: [f32; 2] = [0.0, 1.0];

/// state anim record (+0x220 + 0x10 i)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AnimRec {
    /// +0 oid (-1 none), +4 its SM object (class 0x6d), +8 its sound object (class 0x6f), +0xc flags
    pub oid: i32,
    pub sm: u32,
    pub snd: u32,
    pub flags: u32,
}

/// state range record (+0x270 + 0x28 i)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RangeRec {
    /// +0 distance test on, +4 angle test on
    pub has_d: bool,
    pub has_a: bool,
    /// +0x8 / +0x10 distance enter / stay ranges, +0x18 / +0x20 angle enter / stay ranges (open intervals)
    pub d_in: [f32; 2],
    pub d_stay: [f32; 2],
    pub a_in: [f32; 2],
    pub a_stay: [f32; 2],
}

#[derive(Clone, Debug, Default)]
pub struct C2d1080 {
    pub b: C71,
    /// xfLin / xfAng and the +0xe8 physics block
    pub phys: AloPhys,
    /// +0xec (ALO row 50)
    pub f_ec: u32,
    /// +0x194 anim handle, +0x198 sound handle
    pub f_194: u32,
    pub f_198: u32,
    /// +0x1c0 state, +0x1c4 state start time, +0x1c8 turn freeze, +0x1cc state hold (anim events)
    pub f_1c0: i32,
    pub f_1c4: f32,
    pub f_1c8: u32,
    pub f_1cc: u32,
    /// +0x1d0 nav mesh (row 1378, raw oid), +0x1d4 target (row 1379, object handle after load)
    pub f_1d0: u32,
    pub f_1d4: u32,
    /// +0x1d8 scratch path (FUN_001e94b0(0x2c9, 16)), +0x1dc arc length on it
    pub path: Option<Path>,
    pub f_1dc: f32,
    /// +0x1e0 own / +0x1f0 target position at the last replan, +0x200 its time
    pub f_1e0: [f32; 3],
    pub f_1f0: [f32; 3],
    pub f_200: f32,
    /// +0x204 yaw goal
    pub f_204: f32,
    /// +0x210 chase velocity goal
    pub f_210: [f32; 3],
    pub anims: [AnimRec; 5],
    pub ranges: [RangeRec; 5],
    /// +0x338 speed, +0x340 / +0x350 / +0x360 polynomials (remaining path, heading, facing), +0x370 yaw clamp
    pub f_338: f32,
    pub f_340: [f32; 3],
    pub f_350: [f32; 3],
    pub f_360: [f32; 3],
    pub f_370: [f32; 2],
    /// +0x378 angular servo params, +0x384 linear gain
    pub f_378: [f32; 3],
    pub f_384: f32,
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn len(a: [f32; 3]) -> f32 {
    dot(a, a).sqrt()
}
fn handle(obj: usize) -> u32 {
    obj as u32 + 1
}
/// c0 + x (c1 + x c2)
fn poly(c: [f32; 3], x: f32) -> f32 {
    c[0] + x * (c[1] + x * c[2])
}
/// clamp to [lo, hi] as the decompile writes it (lo when below, hi when above)
fn clamp(x: f32, lo: f32, hi: f32) -> f32 {
    if lo <= x {
        if x <= hi { x } else { hi }
    } else {
        lo
    }
}

/// FUN_0011d910(x, range): lo < x < hi
pub fn FUN_0011d910(x: f32, r: [f32; 2]) -> bool {
    r[0] < x && x < r[1]
}

/// FUN_001a6fc0(0, obj, v) for an unparented object: v in the object's frame (rows of `rot`)
fn to_local(rot: &M3, v: [f32; 3]) -> [f32; 3] {
    [dot(rot[0], v), dot(rot[1], v), dot(rot[2], v)]
}

/// per-frame context (DAT_002dded0 time, DAT_002dded4 dt)
pub struct Cx<'a> {
    pub t: f32,
    pub dt: f32,
    pub h: &'a mut dyn ClassHost,
}

impl C2d1080 {
    /// slot 6 FUN_00180500: FUN_0017f090; +0x340 / +0x350 / +0x364 = 1, +0x338 = 250, +0x370 = {-FLT_MAX, FLT_MAX}
    /// (DAT_002a5658); FUN_001813c0(1) / FUN_001813f0(1); anim records off (FUN_0017f3e0); state -1
    pub fn ee_sub_180500(&mut self, obj: usize, h: &mut dyn ClassHost) {
        self.b.ee_sub_17f090(obj, h);
        self.f_340[0] = 1.0;
        self.f_350[0] = 1.0;
        self.f_360[1] = 1.0;
        self.f_338 = 250.0;
        self.f_370 = [-f32::MAX, f32::MAX];
        self.ee_sub_1813c0(1.0);
        self.ee_sub_1813f0(1.0);
        for a in &mut self.anims {
            a.oid = -1;
        }
        self.f_1c0 = -1;
    }

    /// FUN_001813c0(k): angular servo +0x378 = {25 k, 0 k, 0.25}
    pub fn ee_sub_1813c0(&mut self, k: f32) {
        self.f_378 = [K_2BE870[0] * k, K_2BE870[1] * k, K_2BE870[2]];
    }

    /// FUN_001813f0(k): +0x384 = 5 k
    pub fn ee_sub_1813f0(&mut self, k: f32) {
        self.f_384 = k * K_2BE870[3];
    }

    /// method 1375 FUN_00181308(state, anim oid, flags) (state < 5)
    pub fn ee_sub_181308(&mut self, state: u32, oid: i32, flags: u32) {
        if let Some(a) = self.anims.get_mut(state as usize) {
            a.oid = oid;
            a.flags = flags;
        }
    }

    /// method 1380 FUN_00181330(state, enter, stay): distance ranges on
    pub fn ee_sub_181330(&mut self, state: u32, enter: [f32; 2], stay: [f32; 2]) {
        if let Some(r) = self.ranges.get_mut(state as usize) {
            r.has_d = true;
            r.d_in = enter;
            r.d_stay = stay;
        }
    }

    /// method 1381 FUN_00181378(state, enter, stay): angle ranges on
    pub fn ee_sub_181378(&mut self, state: u32, enter: [f32; 2], stay: [f32; 2]) {
        if let Some(r) = self.ranges.get_mut(state as usize) {
            r.has_a = true;
            r.a_in = enter;
            r.a_stay = stay;
        }
    }

    /// slot 11 FUN_00180598: FUN_0017f110; state -1
    pub fn ee_sub_180598(&mut self, obj: usize, cx: &mut Cx) {
        self.b.ee_sub_17f110(obj, cx.h);
        self.ee_sub_181268(obj, -1, cx);
    }

    /// slot 25 FUN_00180638: FUN_0017f170; state 0
    pub fn ee_sub_180638(&mut self, obj: usize, cx: &mut Cx) {
        self.b.ee_sub_17f170(obj, cx.h);
        self.ee_sub_181268(obj, 0, cx);
    }

    /// slot 82 FUN_00181238 (reset): FUN_0017f6b0; state 0. Returns whether the pending touch is cancelled.
    pub fn ee_sub_181238(&mut self, obj: usize, cx: &mut Cx) -> bool {
        let cancel = self.b.ee_sub_17f6b0();
        self.ee_sub_181268(obj, 0, cx);
        cancel
    }

    /// slot 14 FUN_001805c8: FUN_0017f138 (+ slot 82); scratch path FUN_001e94b0(0x2c9, 16); xfLin / xfAng kind 2
    /// (FUN_002303a0); anim records resolved (FUN_0017f408)
    pub fn ee_sub_1805c8(&mut self, obj: usize, cx: &mut Cx) {
        self.b.ee_sub_17f138(obj, cx.h);
        self.ee_sub_181238(obj, cx);
        self.path = Some(Path::polyline(vec![[0.0; 3]; 2], false));
        cx.h.ee_sub_2303a0_off(handle(obj), 0x58, 2);
        cx.h.ee_sub_2303a0_off(handle(obj), 0x5c, 2);
        self.phys.kind = 2;
        self.ee_sub_17f408(obj, cx.h);
    }

    /// FUN_0017f408(self, 5, +0x220): every record with an oid: FUN_0015e938(4, oid) -> class 0x6d object to +4,
    /// else class 0x6f to +8
    pub fn ee_sub_17f408(&mut self, obj: usize, h: &mut dyn ClassHost) {
        for a in &mut self.anims {
            if a.oid == -1 {
                continue;
            }
            let o = h.ee_sub_15e938(4, a.oid, handle(obj));
            if o == 0 {
                continue;
            }
            if h.ee_sub_132b10(o, 0x6d) != 0 {
                a.sm = o;
            } else if h.ee_sub_132b10(o, 0x6f) != 0 {
                a.snd = o;
            }
        }
    }

    /// FUN_00181268(state): on a change (leaving -1 refills hp +0x184 = +0x180): state, time; states 0..2 / 4
    /// clear the chase velocity; FUN_0017f4c0 with the state's anim record (none for -1)
    pub fn ee_sub_181268(&mut self, obj: usize, s: i32, cx: &mut Cx) {
        if self.f_1c0 == s {
            return;
        }
        if self.f_1c0 == -1 {
            self.b.f_184 = self.b.f_180;
        }
        self.f_1c0 = s;
        self.f_1c4 = cx.t;
        if (0..=2).contains(&s) || s == 4 {
            self.f_210 = [0.0; 3];
        }
        let rec = if s == -1 { None } else { self.anims.get(s as usize).copied() };
        self.ee_sub_17f4c0(obj, rec, cx.h);
    }

    /// FUN_0017f4c0(self, rec): stop the sound (FUN_001f15b8 +0x198) and the anim (FUN_00130658 +0x194); with a
    /// record: FUN_001305b8(0, 1, rec+4, self, rec+0xc, +0x194), FUN_001f1448(rec+8, self, -1, rec+0xc, +0x198)
    pub fn ee_sub_17f4c0(&mut self, obj: usize, rec: Option<AnimRec>, h: &mut dyn ClassHost) {
        h.ee_sub_1f15b8(&mut self.f_198);
        shared::anim::ee_sub_130658(&mut self.f_194, h);
        if let Some(r) = rec {
            shared::anim::ee_sub_1305b8(0, 1.0, r.sm, handle(obj), r.flags, &mut self.f_194, h);
            self.f_198 = h.ee_sub_1f1448(r.snd, handle(obj), r.flags);
        }
    }

    /// FUN_0017f550: the state's anim is done (FUN_00131968), or without one its sound (FUN_001f07e8() ==
    /// FUN_001f07c8(+0x198)); neither -> 1
    pub fn ee_sub_17f550(&self, h: &mut dyn ClassHost) -> bool {
        if self.f_194 != 0 {
            h.ee_sub_131968(self.f_194)
        } else if self.f_198 != 0 {
            h.snd_done_1f07c8(self.f_198)
        } else {
            true
        }
    }

    /// target set and in the world (+0x28 bit 60)
    fn target_live(&self, h: &mut dyn ClassHost) -> bool {
        self.f_1d4 != 0 && h.in_world_28(self.f_1d4)
    }

    /// slot 15 FUN_00180668: base FUN_001f36e0; states 2 / 4 end with their anim; 1..3 need a live target; state
    /// selection by the range records (unless +0x1cc holds it); then 1 / 2: yaw goal = clamp(poly +0x360 of the
    /// target bearing, +0x370) (not while +0x1c8); 3: chase. `others` = (handle, +0xec, position) of the other
    /// class-73 objects (FUN_0015e9c8(0x105, 0x49), REPLACED by the ported instances).
    pub fn ee_sub_180668(&mut self, obj: usize, others: &[(u32, u32, [f32; 3])], cx: &mut Cx) {
        cx.h.base(0x1f36e0, obj);
        let mut s = self.f_1c0;
        if (s == 2 || s == 4) && self.ee_sub_17f550(cx.h) {
            self.ee_sub_181268(obj, 0, cx);
            s = self.f_1c0;
        }
        if s != 0 && s != 4 && !self.target_live(cx.h) {
            self.ee_sub_181268(obj, 0, cx);
        }
        if self.f_1cc == 0 && self.f_1d4 != 0 && cx.h.in_world_28(self.f_1d4) {
            let tp = cx.h.ee_sub_1a62e8(self.f_1d4);
            // FUN_001a62e8(self): the xfLin position
            let sp = self.phys.pos;
            let d = sub(tp, sp);
            let dist = len(d);
            let l = to_local(&self.phys.rot, d);
            let ang = FUN_0011cf60(l[1].atan2(l[0]));
            let keep = |r: &RangeRec, d_r: [f32; 2], a_r: [f32; 2]| {
                let ok = if r.has_d { FUN_0011d910(dist, d_r) } else { r.has_a };
                ok && (!r.has_a || FUN_0011d910(ang, a_r))
            };
            let cur = self.ranges.get(self.f_1c0 as usize).copied().unwrap_or_default();
            if !keep(&cur, cur.d_stay, cur.a_stay) {
                let mut i = 4;
                while 0 < i {
                    let r = self.ranges[i as usize];
                    if keep(&r, r.d_in, r.a_in) {
                        break;
                    }
                    i -= 1;
                }
                self.ee_sub_181268(obj, i, cx);
            }
        }
        match self.f_1c0 {
            1 | 2 => {
                if self.f_1c8 != 0 {
                    return;
                }
                // FUN_0022fcc8(0, xfAng parent = none, d): the world offset
                let d = sub(cx.h.ee_sub_1a62e8(self.f_1d4), self.phys.pos);
                let a = FUN_0011cf60(d[1].atan2(d[0]));
                let y = poly(self.f_360, a);
                let mut v = self.f_370[0];
                if v <= y {
                    v = y;
                    if self.f_370[1] < y {
                        v = self.f_370[1];
                    }
                }
                self.f_204 = v;
            }
            3 => self.chase(obj, others, cx),
            _ => {}
        }
    }

    /// slot 15 state 3 (inside FUN_00180668): replan when the target or self moved more than 100 since the last
    /// plan or 5 s passed; walk the path from +0x1dc, aim 200 ahead; speed = +0x338 * clamp01(poly +0x340 of the
    /// remaining length) * clamp01(poly +0x350 of the local heading) * clamp01(poly 0x2be830 of BIF 103); other
    /// class-73 objects with another +0xec push away; +0x210 = that velocity clamped to the speed, +0x204 = its
    /// heading
    fn chase(&mut self, obj: usize, others: &[(u32, u32, [f32; 3])], cx: &mut Cx) {
        let me = handle(obj);
        let tp = cx.h.ee_sub_1a62e8(self.f_1d4);
        let sp = self.phys.pos;
        // vtable +0x10c (velocity) result unused
        let replan = D_2BE820 < len(sub(tp, self.f_1f0)) || D_2BE820 < len(sub(sp, self.f_1e0)) || D_2BE824 < cx.t - self.f_200;
        if replan {
            // REPLACED: FUN_001c02d0 nav-mesh search -> straight path self -> target
            self.path = Some(Path::polyline(vec![sp, tp], false));
            self.f_1dc = 0.0;
            self.f_1f0 = tp;
            self.f_1e0 = sp;
            self.f_200 = cx.t;
        }
        let Some(p) = self.path.clone() else { return };
        let hit = FUN_001e6ed8(self.f_1dc, &p, sp);
        self.f_1dc = hit.l;
        let aim = [0, 1, 2].map(|k| hit.cp[k] + hit.dir[k] * D_2BE828);
        let dv = sub(aim, sp);
        let dl = len(dv);
        let dir = if dl < 0.0001 { [0.0; 3] } else { dv.map(|x| x / dl) };
        let rem = FUN_001e6648(&p) - self.f_1dc;
        let l = to_local(&self.phys.rot, dir);
        let head = l[1].atan2(l[0]);
        let bif = cx.h.ee_sub_154c90();
        let f1 = clamp(poly(self.f_340, rem), C_2BBB60[0], C_2BBB60[1]);
        let f2 = clamp(poly(self.f_350, head), C_2BBB60[0], C_2BBB60[1]);
        let f3 = clamp(poly(P_2BE830, bif), C_2BBB60[0], C_2BBB60[1]);
        let speed = f1 * f2 * f3 * self.f_338;
        let mut vel = dir.map(|x| x * speed);
        for &(o, ec, op) in others {
            if o == me || ec == self.f_ec {
                continue;
            }
            let d = sub(sp, op);
            let l = len(d);
            let n = if l < 0.0001 { [1.0, 0.0, 0.0] } else { d.map(|x| x / l) };
            let g1 = clamp(poly(P_2BE840, l), C_2BE850[0], C_2BE850[1]);
            let g2 = clamp(poly(P_2BE860, dot(n, dir)), C_2BBB60[0], C_2BBB60[1]);
            let g = g1 * g2;
            vel = [0, 1, 2].map(|k| vel[k] + n[k] * g);
        }
        self.f_210 = FUN_0011e9f8(speed, vel);
        self.f_204 = dir[1].atan2(dir[0]);
    }

    /// slot 69 FUN_00180ee8 (no ALO gravity): except in state 4 the xfAng servo (FUN_001f59a0, +0x378) turns to
    /// the yaw goal (FUN_0011b500 about z; FUN_0022fd38 = identity without a parent); in state 3 the xfLin accel
    /// += (+0x210 - v) * +0x384 (FUN_001fac30)
    pub fn ee_sub_180ee8(&mut self, dt: f32) {
        let b = &mut self.phys;
        if self.f_1c0 != 4 {
            let m = FUN_0011b500([0.0, 0.0, 1.0], self.f_204);
            let a = FUN_001f59a0(&b.rot, b.w, b.alpha, &m, self.f_378, dt);
            b.alpha = [0, 1, 2].map(|k| b.alpha[k] + a[k]);
        }
        if self.f_1c0 == 3 {
            let a = sub(self.f_210, b.v).map(|x| x * self.f_384);
            if FUN_00230380(b.kind) {
                b.a = [0, 1, 2].map(|k| b.a[k] + a[k]);
            }
        }
    }

    /// FUN_001f3980 physics part with this class's slot 69
    pub fn physics(&mut self, obj: usize, dt: f32, h: &mut dyn ClassHost) {
        h.base(0x1f3980, obj);
        let b = &mut self.phys;
        b.prev_v = b.v;
        b.prev_w = b.w;
        if matches!(b.kind, 1 | 2) {
            b.a = [0.0; 3];
            b.alpha = [0.0; 3];
            self.ee_sub_180ee8(dt);
        }
        FUN_0022de88(&mut self.phys, dt);
    }

    /// slot 4 FUN_00181060(self, kind, ev): base FUN_001c3e20; anim events (kind 9) of the state's anim (+0x194)
    /// or sound (+0x198): 0x25 / 0x26 turn freeze +0x1c8 on / off, 0x27 / 0x28 state hold +0x1cc on / off,
    /// 0x15e: +0x210 = the anim's root velocity `v` (FUN_00204e38) turned by the yaw goal, set as the xfLin
    /// velocity (vtable +0x11c). `src` = the event's source handle.
    pub fn ee_sub_181060(&mut self, obj: usize, kind: i32, src: u32, code: i32, v: [f32; 3], h: &mut dyn ClassHost) {
        h.base(0x1c3e20, obj);
        if kind != 9 {
            return;
        }
        if src != self.f_194 && (self.f_198 == 0 || src != self.f_198) {
            return;
        }
        match code {
            0x25 => self.f_1c8 = 1,
            0x26 => self.f_1c8 = 0,
            0x27 => self.f_1cc = 1,
            0x28 => self.f_1cc = 0,
            0x15e => {
                let m = FUN_0011b500([0.0, 0.0, 1.0], self.f_204);
                let w = [0, 1, 2].map(|k| m[0][k] * v[0] + m[1][k] * v[1] + m[2][k] * v[2]);
                self.f_210 = w;
                self.phys.v = w;
            }
            _ => {}
        }
    }

    /// slot 54 FUN_00180ff0(self, contact, side): FUN_0017f1a0; with touch damage +0x188 a touch by the player
    /// (DAT_002dfb80) queues the deferred slot 81 (FUN_0017f5b0). Returns whether it was queued.
    pub fn ee_sub_180ff0(&mut self, obj: usize, other: u32, h: &mut dyn ClassHost) -> bool {
        self.b.ee_sub_17f1a0(obj, h);
        self.b.f_188 != 0 && other == guard::PLAYER_HANDLE && self.b.ee_sub_17f5b0()
    }

    /// slot 74 FUN_001811b8(self, hit): not in state 4: FUN_0017f1d0; taken with hp +0x184 >= 0 -> (hp < 1:
    /// FUN_001619a0(0x18, self)) state 4. Returns whether the hit was taken.
    pub fn ee_sub_1811b8(&mut self, obj: usize, hit: &guard::slots::GuardHit, cx: &mut Cx) -> bool {
        if self.f_1c0 == 4 {
            return false;
        }
        if !c71::ee_sub_17f1d0(&mut self.b, &mut self.phys, obj, hit, cx.h) {
            return false;
        }
        if self.b.f_184 < 0 {
            return true;
        }
        if self.b.f_184 < 1 {
            cx.h.ee_sub_1619a0(0x18, handle(obj));
        }
        self.ee_sub_181268(obj, 4, cx);
        true
    }

    /// rows 1375 / 1380 / 1381 (methods FUN_0010fe60 / 10ff18 / 10ffe8 -> FUN_00181308 / 181330 / 181378), the
    /// member rows (`ROWS_73`), ALO row 50 (+0xec) and the class-71 rows
    pub fn feed(&mut self, props: &[crate::engine::world::props::Prop]) {
        use crate::engine::world::props::{ArrayItem, PropValue};
        use super::k4::word;
        self.b.feed(props);
        for p in props {
            // argument words of a method row, item by item (a vec2 item = 2 words)
            let args = |v: &PropValue| -> Vec<u32> {
                match v {
                    PropValue::Array(items) => items
                        .iter()
                        .flat_map(|it| match it {
                            ArrayItem::One(_, b) => b.chunks(4).map(word).collect::<Vec<_>>(),
                            ArrayItem::List(_, v) => v.iter().flat_map(|b| b.chunks(4).map(word).collect::<Vec<_>>()).collect(),
                        })
                        .collect(),
                    PropValue::Scalar(b) => b.chunks(4).map(word).collect(),
                }
            };
            let f = |a: &[u32], i: usize| f32::from_bits(a.get(i).copied().unwrap_or(0));
            match p.field {
                1375 => {
                    let a = args(&p.value);
                    self.ee_sub_181308(a.first().copied().unwrap_or(u32::MAX), a.get(1).copied().unwrap_or(0) as i32, a.get(2).copied().unwrap_or(0));
                }
                1380 | 1381 => {
                    let a = args(&p.value);
                    let st = a.first().copied().unwrap_or(u32::MAX);
                    let (e, s) = ([f(&a, 1), f(&a, 2)], [f(&a, 3), f(&a, 4)]);
                    if p.field == 1380 {
                        self.ee_sub_181330(st, e, s);
                    } else {
                        self.ee_sub_181378(st, e, s);
                    }
                }
                50 => {
                    if let PropValue::Scalar(b) = &p.value {
                        self.f_ec = word(b);
                    }
                }
                _ => {
                    let PropValue::Scalar(b) = &p.value else { continue };
                    let Some(&(_, off)) = ROWS_73.iter().find(|r| r.0 == p.field) else { continue };
                    let w = |k: usize| f32::from_bits(b.get(k * 4..k * 4 + 4).map(word).unwrap_or(0));
                    match off {
                        0x1d0 => self.f_1d0 = word(b),
                        0x1d4 => self.f_1d4 = word(b),
                        0x338 => self.f_338 = w(0),
                        0x340 => self.f_340 = [w(0), w(1), w(2)],
                        0x350 => self.f_350 = [w(0), w(1), w(2)],
                        0x360 => self.f_360 = [w(0), w(1), w(2)],
                        0x370 => self.f_370 = [w(0), w(1)],
                        _ => {}
                    }
                }
            }
        }
    }
}

impl ClassSet {
    /// K15: construct + init (slot 6) + feed the `PORTED_K15` classes (clones read their source's props); xfLin
    /// at the object's rest world origin, at rest, no gravity (slot 69 does not chain the ALO one)
    pub(super) fn build_k15(&mut self, w: &World, h: &mut dyn ClassHost) {
        let mut bw = crate::engine::gfx::attach::BoneWorlds::default();
        for (i, o) in w.objects.iter().enumerate() {
            if o.class != 73 {
                continue;
            }
            let src = o.clone_of.unwrap_or(i);
            let props = w.objects.get(src).and_then(crate::engine::game::fields::object_props);
            let mut c = C2d1080::default();
            c.ee_sub_180500(i, h);
            if let Some(p) = props {
                c.feed(p);
            }
            // object references (resolved like the K14 rows): +0x190 hit fx, +0x1d4 target
            if c.b.f_190 != 0 {
                c.b.f_190 = h.ee_sub_15e938(0x201e, c.b.f_190 as i32, handle(i));
            }
            if c.f_1d4 != 0 {
                c.f_1d4 = h.ee_sub_15e938(0x2002, c.f_1d4 as i32, handle(i));
            }
            c.phys.pos = bw.get(w, i, 0)[3];
            c.phys.gravity = [0.0; 3];
            self.k15.push((i, Inst::K15(Box::new(c))));
        }
    }
    /// K15 class-73 state at object index `obj`
    pub fn k15_get(&mut self, obj: usize) -> Option<&mut C2d1080> {
        self.k15.iter_mut().find(|x| x.0 == obj).and_then(|x| if let Inst::K15(c) = &mut x.1 { Some(&mut **c) } else { None })
    }
    /// K15 load-post: slot 14
    pub(super) fn post_load_k15(&mut self, h: &mut dyn ClassHost) {
        let (t, dt) = (self.g.t, self.g.dt);
        for (i, x) in &mut self.k15 {
            if let Inst::K15(c) = x {
                let mut cx = Cx { t, dt, h: &mut *h };
                c.ee_sub_1805c8(*i, &mut cx);
            }
        }
    }
    /// K15 slot 74: a hit record delivered to object `obj`
    pub fn k15_hit(&mut self, obj: usize, hit: &guard::slots::GuardHit, h: &mut dyn ClassHost) -> bool {
        if let Some(r) = self.k15_83_hit(obj, hit, h) {
            return r;
        }
        let (t, dt) = (self.g.t, self.g.dt);
        let Some(c) = self.k15_get(obj) else { return false };
        c.ee_sub_1811b8(obj, hit, &mut Cx { t, dt, h })
    }
    /// K15 slot 54: `other` touches object `obj` (queues the deferred slot 81)
    pub fn k15_touch(&mut self, obj: usize, other: u32, h: &mut dyn ClassHost) {
        let Some(c) = self.k15_get(obj) else { return };
        if c.ee_sub_180ff0(obj, other, h) {
            self.k15_deferred.push((obj, other));
        }
    }
    /// K15 per frame: slot 15 (with the other class-73 positions), the physics part (slot 69), then the
    /// deferred slot-81 calls (FUN_00149c80 phase)
    pub(super) fn frame_k15(&mut self, h: &mut dyn ClassHost) {
        let (t, dt) = (self.g.t, self.g.dt);
        let others: Vec<(u32, u32, [f32; 3])> = self
            .k15
            .iter()
            .filter_map(|(i, x)| if let Inst::K15(c) = x { Some((handle(*i), c.f_ec, c.phys.pos)) } else { None })
            .collect();
        for (i, x) in &mut self.k15 {
            if let Inst::K15(c) = x {
                let mut cx = Cx { t, dt, h: &mut *h };
                c.ee_sub_180668(*i, &others, &mut cx);
                c.physics(*i, dt, &mut *cx.h);
            }
        }
        for (obj, other) in std::mem::take(&mut self.k15_deferred) {
            if let Some(c) = self.k15_get(obj) {
                c71::ee_sub_17f5f8(&mut c.b, c.phys.v, obj, other, h);
            }
        }
    }
}
