//! WP-K24: `K24Host` - a `ClassHost` wrapper that routes the guard-object hooks of the class code to the
//! ported guards (`State::k6`, classes/guard/) and forwards everything else to the wrapped host
//! (classes/k24_fwd.rs, GENERATED). The class-102 bomb (classes/k23_102.rs) grabs / lifts / knocks out a
//! guard through these hooks; before K24 every one was a no-op default on `EngineHost`.
//! Guard handle = object index + 1 (`guard::mod` `ee_sub_1f7598`).
use super::guard::c56::Guard;
use super::{ClassHost, ClassSet, Inst};
use crate::engine::physics::driven::GameRng;

pub struct K24Host<'a> {
    pub h: &'a mut dyn ClassHost,
    pub k6: &'a mut Vec<(usize, Inst)>,
    pub rng: &'a mut GameRng,
    pub now: f32,
    pub dt: f32,
    /// POOL1 r40: the class-48 C2d9978 objects (handles) for FUN_00224bf8's probe / FUN_001f7598 delivery
    pub alo48: Vec<u32>,
    /// POOL1 r42: the class-98 C2d4b78 objects (handles; e.g. the disco pillars' hit receiver 123913 under 124088)
    pub alo98: Vec<u32>,
    /// POOL1 r42: FUN_001f7598 hits to class-98 targets, delivered by `with_k24` through ClassSet::ee_sub_1f7598
    /// (the class-98 objects live in `ClassSet::k5`, outside this host)
    pub deferred: Vec<super::guard::slots::GuardHit>,
}

impl K24Host<'_> {
    fn guard(&mut self, obj: u32) -> Option<(usize, &mut Guard)> {
        let i = (obj as usize).checked_sub(1)?;
        self.k6.iter_mut().find(|x| x.0 == i).and_then(|x| if let Inst::Guard(g) = &mut x.1 { Some((i, &mut **g)) } else { None })
    }
    /// vtable +off on a guard: +0x4c = LO Remove (FUN_001a5630: +0x28 bit 60 cleared, out of the world)
    pub(super) fn r_vcall(&mut self, obj: u32, off: u32) {
        match self.guard(obj) {
            Some((_, g)) if off == 0x4c => g.removed = true,
            Some(_) => {}
            None => self.h.vcall(obj, off),
        }
    }
    /// FUN_001a62e8: a guard's world position is its body's
    pub(super) fn r_ee_sub_1a62e8(&mut self, obj: u32) -> [f32; 3] {
        match self.guard(obj) {
            Some((_, g)) => g.body.pos,
            None => self.h.ee_sub_1a62e8(obj),
        }
    }
    /// +0x28: a guard's bit 60 follows `removed`
    pub(super) fn r_obj_flags_28(&mut self, obj: u32) -> u64 {
        let f = self.h.obj_flags_28(obj);
        match self.guard(obj) {
            Some((_, g)) if g.removed => f & !(1 << 60),
            Some(_) => f | 1 << 60,
            None => f,
        }
    }
    pub(super) fn r_in_world_28(&mut self, obj: u32) -> bool {
        match self.guard(obj) {
            Some((_, g)) => !g.removed,
            None => self.h.in_world_28(obj),
        }
    }
    /// FUN_00229738(obj, rec): unlink the record from the guard's +0x138
    pub(super) fn r_ee_sub_229738(&mut self, obj: u32, rec: u32) {
        match self.guard(obj) {
            Some((_, g)) => {
                if let Some(k) = g.f_138.iter().position(|&o| o == rec) {
                    g.f_138.remove(k);
                }
            }
            None => self.h.ee_sub_229738(obj, rec),
        }
    }
    /// FUN_00229710(obj, rec): link at the head of +0x138
    pub(super) fn r_k23_link_229710(&mut self, obj: u32, owner: u32) {
        match self.guard(obj) {
            Some((_, g)) => g.f_138.insert(0, owner),
            None => self.h.k23_link_229710(obj, owner),
        }
    }
    /// FUN_0015e9c8(class): the host's objects plus the in-world guards that are-a `class` (FUN_00132b10)
    pub(super) fn r_query_15e9c8(&mut self, class: i32) -> Vec<u32> {
        use crate::engine::player::p4::FUN_00132b10;
        let mut v = self.h.query_15e9c8(class);
        let gone: Vec<u32> = self.k6.iter().filter(|x| matches!(&x.1, Inst::Guard(g) if g.removed)).map(|x| x.0 as u32 + 1).collect();
        v.retain(|o| !gone.contains(o));
        for (i, x) in self.k6.iter() {
            if let Inst::Guard(g) = x {
                let o = *i as u32 + 1;
                if !g.removed && FUN_00132b10(g.class as i32, class) && !v.contains(&o) {
                    v.push(o);
                }
            }
        }
        v
    }
    pub(super) fn r_ee_sub_1fac30(&mut self, obj: usize, a: [f32; 3]) {
        match self.guard(obj as u32 + 1) {
            Some((_, g)) => g.k24_accel(a),
            None => self.h.ee_sub_1fac30(obj, a),
        }
    }
    pub(super) fn r_phys_e8_80(&mut self, obj: usize) -> [f32; 3] {
        match self.guard(obj as u32 + 1) {
            Some(_) => super::guard::lift::GRAVITY,
            None => self.h.phys_e8_80(obj),
        }
    }
    pub(super) fn r_vcall_11c_v(&mut self, obj: u32, v: [f32; 3]) {
        match self.guard(obj) {
            Some((_, g)) => {
                g.k24_set_vel(v);
                // CRAWL R95: thrown out of Murray's hands: the host stops carrying it (the SO-object path does
                // this in EngineHost::vcall_11c_v), so FUN_001a62e8 reads the flying body (opt-out SLY2_NO_R95_CARRY)
                if std::env::var_os("SLY2_NO_R95_CARRY").is_none() {
                    self.h.so_carry(obj, 0);
                }
            }
            None => self.h.vcall_11c_v(obj, v),
        }
    }
    pub(super) fn r_k22_speed_10c(&mut self, obj: u32) -> [f32; 3] {
        match self.guard(obj) {
            Some((_, g)) => g.k24_vel(),
            None => self.h.k22_speed_10c(obj),
        }
    }
    pub(super) fn r_k23_comp_b0(&mut self, guard: u32) -> Option<i32> {
        match self.guard(guard) {
            Some((_, g)) => g.k24_comp_b0(),
            None => self.h.k23_comp_b0(guard),
        }
    }
    pub(super) fn r_k23_comp_b0_set(&mut self, guard: u32, s: i32) {
        let now = self.now;
        match self.guard(guard) {
            Some((_, g)) => g.k24_comp_b0_set(s, now),
            None => self.h.k23_comp_b0_set(guard, s),
        }
    }
    pub(super) fn r_k23_guard_209840(&mut self, guard: u32, rows: [[f32; 4]; 3]) {
        match self.guard(guard) {
            Some((_, g)) => g.ee_sub_209840(rows),
            None => self.h.k23_guard_209840(guard, rows),
        }
    }
    pub(super) fn r_k23_guard_206eb0(&mut self, obj: u32, k: i32) -> bool {
        match self.guard(obj) {
            Some((_, g)) => !g.removed && g.ee_sub_206eb0(k),
            None => self.h.k23_guard_206eb0(obj, k),
        }
    }
    pub(super) fn r_k23_brain_209e28(&mut self, guard: u32, a: i32) {
        let (now, dt) = (self.now, self.dt);
        let Some(i) = (guard as usize).checked_sub(1) else { return };
        let Some(x) = self.k6.iter_mut().find(|x| x.0 == i) else {
            return self.h.k23_brain_209e28(guard, a);
        };
        if let Inst::Guard(g) = &mut x.1 {
            g.ee_sub_209e28(i, a, now, dt, self.rng, &mut *self.h);
        }
    }
}

impl ClassSet {
    /// run `f` with the guard-routing host (`self.k6` / `self.rng` are lent to it for the call; `f` must not
    /// use them through `self`)
    pub fn with_k24(&mut self, h: &mut dyn ClassHost, f: impl FnOnce(&mut Self, &mut dyn ClassHost)) {
        let mut k6 = std::mem::take(&mut self.k6);
        let mut rng = std::mem::replace(&mut self.rng, GameRng(1));
        {
            let alo98 = self.k5.iter().filter(|x| matches!(x.1, Inst::C2d4b78(_))).map(|x| x.0 as u32 + 1).collect();
            let mut kh = K24Host { h, k6: &mut k6, rng: &mut rng, now: self.g.t, dt: self.g.dt, alo48: self.insts.iter().filter(|x| matches!(x.1, Inst::C2d9978(_))).map(|x| x.0 as u32 + 1).collect(), alo98, deferred: Vec::new() };
            f(self, &mut kh);
            let d = std::mem::take(&mut kh.deferred);
            for hit in d {
                // FUN_001f7598: +0x44 = FUN_001f78d8(target, kind); slot 74 (class 98 FUN_001badf0) accepts ->
                // on (+0x44 & 0xf8) the target's vt+0x1c(target, 0xb, 1, 2, &hit) = FUN_001a61e0 script message 0xb
                let f44 = crate::engine::player::p8_hazard::FUN_001f78d8(&kh.h.alo_110(hit.target), hit.kind as u32);
                if self.ee_sub_1f7598(&hit, kh.h) && f44 & 0xf8 != 0 {
                    kh.h.k27_script_msg(hit.target, 0xb, &[hit.src]);
                }
            }
        }
        self.rng = rng;
        k6.append(&mut self.k6);
        self.k6 = k6;
    }
}

impl K24Host<'_> {
    /// slot 57's SO contact walk (REPLACED, classes/k15_79.rs): the host's contacts plus every in-world guard
    /// whose body is within the guard contact reach of the projectile (`player::plugin::GUARD_CONTACT_REACH`,
    /// the same REPLACED body solver as Sly's guard contacts); point = the projectile position
    pub(super) fn r_proj_contacts_13aeb0(&mut self, obj: usize, pos: [f32; 3]) -> Vec<super::k15_79::ProjContact> {
        use crate::engine::player::plugin::GUARD_CONTACT_REACH as R;
        let mut v = self.h.proj_contacts_13aeb0(obj, pos);
        for (i, x) in self.k6.iter() {
            if let Inst::Guard(g) = x {
                let d = [g.body.pos[0] - pos[0], g.body.pos[1] - pos[1], pos[2] - g.body.pos[2]];
                if !g.removed && d[0] * d[0] + d[1] * d[1] < R * R && d[2] > -R && d[2] < 2.0 * R {
                    let o = *i as u32 + 1;
                    v.push(super::k15_79::ProjContact { obj: o, so: o, point: pos });
                }
            }
        }
        v
    }
    /// FUN_0013b5e8 victims (REPLACED FUN_0020fc60 sphere query): the host's plus the in-world guards whose body
    /// is within `r` of `pos`
    pub(super) fn r_proj_victims_13b5e8(&mut self, obj: u32, pos: [f32; 3], r: f32, r_obj: f32, r_34: f32) -> Vec<u32> {
        let mut v = self.h.proj_victims_13b5e8(obj, pos, r, r_obj, r_34);
        for (i, x) in self.k6.iter() {
            if let Inst::Guard(g) = x {
                let d = [g.body.pos[0] - pos[0], g.body.pos[1] - pos[1], g.body.pos[2] - pos[2]];
                if !g.removed && d[0] * d[0] + d[1] * d[1] + d[2] * d[2] <= r * r {
                    v.push(*i as u32 + 1);
                }
            }
        }
        // HEIST r17: the in-world class-98 SOs too (FUN_0020fc60: |pos - sphere centre| <= r + sphere radius), e.g.
        // the EP2 chopper's hit receiver 118349 (turret shells, rec10 196..203: +0x170 0x100, +0x158 counting)
        for (o, c, rr) in self.so98_spheres() {
            let d = [c[0] - pos[0], c[1] - pos[1], c[2] - pos[2]];
            if o != obj && d[0] * d[0] + d[1] * d[1] + d[2] * d[2] <= (r + rr) * (r + rr) {
                v.push(o);
            }
        }
        v
    }
    /// HEIST r17: (handle, world centre, radius) of the in-world class-98 SOs. REPLACED: the SO bound sphere is
    /// centred on the object's current origin (the K26-posed chopper moves; the collision parts are static at
    /// load), radius = its collision parts' bound (`so_bsphere`) or `actor::SO_HIT_R` without collision parts.
    pub(super) fn so98_spheres(&mut self) -> Vec<(u32, [f32; 3], f32)> {
        let mut out = Vec::new();
        for o in self.alo98.clone() {
            if self.h.lo_removed(o as usize - 1) {
                continue;
            }
            let r = self.h.so_bsphere(o).map_or(super::actor::SO_HIT_R, |x| x.1.max(super::actor::SO_HIT_R));
            out.push((o, self.h.ee_sub_1a62e8(o), r));
        }
        out
    }
    /// HEIST r17: FUN_0013aeb0's swept contact (REPLACED SO solver, see the host's `proj_sweep_13aeb0`) with the
    /// in-world class-98 SO spheres too: the nearest entry along p0 -> p1
    pub(super) fn r_proj_sweep_13aeb0(&mut self, obj: usize, p0: [f32; 3], p1: [f32; 3]) -> Option<super::k15_79::ProjContact> {
        let base = self.h.proj_sweep_13aeb0(obj, p0, p1);
        let d = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        let dd = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
        let tb = base.map_or(f32::MAX, |b| {
            let q = [b.point[0] - p0[0], b.point[1] - p0[1], b.point[2] - p0[2]];
            if dd > 0.0 { (q[0] * d[0] + q[1] * d[1] + q[2] * d[2]) / dd } else { 0.0 }
        });
        let mut best = (tb, base);
        for (o, c, r) in self.so98_spheres() {
            if o == obj as u32 + 1 || dd <= 0.0 {
                continue;
            }
            // first t in [0, 1] with |p0 + t d - c| = r (or 0 when p0 is inside)
            let m = [p0[0] - c[0], p0[1] - c[1], p0[2] - c[2]];
            let b = m[0] * d[0] + m[1] * d[1] + m[2] * d[2];
            let cc = m[0] * m[0] + m[1] * m[1] + m[2] * m[2] - r * r;
            let disc = b * b - dd * cc;
            if disc < 0.0 {
                continue;
            }
            let t = if cc <= 0.0 { 0.0 } else { (-b - disc.sqrt()) / dd };
            if (0.0..=1.0).contains(&t) && t < best.0 {
                let pt = [p0[0] + t * d[0], p0[1] + t * d[1], p0[2] + t * d[2]];
                best = (t, Some(super::k15_79::ProjContact { obj: o, so: o, point: pt }));
            }
        }
        best.1
    }
    /// FUN_001f7598(hit) to a guard: +0x44 = FUN_001f78d8(target, kind) (the guard's +0x110 bytes,
    /// `ClassHost::alo_110`), then the target's slot 74 FUN_00207468; else the wrapped host
    pub(super) fn r_ee_sub_1f7598(&mut self, hit: &super::guard::slots::GuardHit) -> bool {
        let Some(i) = (hit.target as usize).checked_sub(1) else { return self.h.ee_sub_1f7598_ok(hit) };
        if self.alo48.contains(&hit.target) {
            // POOL1 r40: class 48 slot 74 = FUN_001f7588 (accept iff hit+0x44 != 0); on (+0x44 & 0xf8) the
            // target's vt+0x1c(target, 0xb, 1, 2, &hit) = FUN_001a61e0 level-script message 0xb (posted)
            let t = self.h.alo_110(hit.target);
            let f = crate::engine::player::p8_hazard::FUN_001f78d8(&t, hit.kind as u32) as i32;
            if f == 0 {
                return false;
            }
            if f & 0xf8 != 0 {
                self.h.k27_script_msg(hit.target, 0xb, &[hit.src]);
            }
            return true;
        }
        if self.alo98.contains(&hit.target) {
            self.deferred.push(hit.clone());
            return true;
        }
        if self.guard(hit.target).is_none() {
            return self.h.ee_sub_1f7598_ok(hit);
        }
        let t = self.h.alo_110(hit.target);
        let mut x = hit.clone();
        x.f_44 = crate::engine::player::p8_hazard::FUN_001f78d8(&t, hit.kind as u32) as i32;
        let h = &mut *self.h;
        match self.k6.iter_mut().find(|y| y.0 == i) {
            Some((_, Inst::Guard(g))) => g.ee_sub_207468(i, &x, false, h),
            _ => false,
        }
    }
}

impl K24Host<'_> {
    /// FUN_00224bf8(arg, bomb, radius, damage, pos) REPLACED (probe rings FUN_0011e208 / FUN_00224820 line
    /// tests -> the in-world guards whose body is within `radius`): one 0x60 hit {bomb, guard, kind `arg`,
    /// damage, pos, unit(guard - pos) * DAT_002c4334} per victim, delivered by FUN_001f7598, then the host's own
    pub(super) fn r_explode_224bf8(&mut self, obj: u32, kind: u32, pos: [f32; 3], radius: f32, damage: i32) {
        let mut v = Vec::new();
        for (i, x) in self.k6.iter() {
            if let Inst::Guard(g) = x {
                let d = [g.body.pos[0] - pos[0], g.body.pos[1] - pos[1], g.body.pos[2] - pos[2]];
                let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                if !g.removed && l <= radius {
                    let u = if l > 1e-4 { [d[0] / l, d[1] / l, d[2] / l] } else { [0.0; 3] };
                    v.push((*i as u32 + 1, u));
                }
            }
        }
        // POOL1 r40: the class-48 colliders in range too (FUN_00224bf8 probes every collider, not only guards)
        // R53: FUN_0020fc60's test |pos - sphere centre| <= radius + sphere radius on the object's world sphere
        // (e.g. Disco receiver 2104 sits in the wall: its origin is > 300 from any reachable spot)
        for o in self.alo48.iter().chain(self.alo98.iter()).copied().collect::<Vec<_>>() {
            let (p, r) = self.h.so_bsphere(o).unwrap_or((self.h.ee_sub_1a62e8(o), 0.0));
            let d = [p[0] - pos[0], p[1] - pos[1], p[2] - pos[2]];
            let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            if o != obj && l <= radius + r {
                v.push((o, if l > 1e-4 { [d[0] / l, d[1] / l, d[2] / l] } else { [0.0; 3] }));
            }
        }
        for (t, u) in v {
            if std::env::var_os("POOL1_DBG").is_some() { eprintln!("POOL1DBG explode {obj} kind {kind} pos {pos:?} r {radius} hit {t} 110 {:?}", self.h.alo_110(t)); }
            let hit = super::guard::slots::GuardHit { src: obj, target: t, kind: kind as i32, amount: damage, pos: [pos[0], pos[1], pos[2], 0.0], vec: [u[0] * 500.0, u[1] * 500.0, u[2] * 500.0, 0.0], f_44: 0 };
            self.r_ee_sub_1f7598(&hit);
        }
        if std::env::var_os("POOL1_DBG").is_some() { eprintln!("POOL1DBG explode {obj} kind {kind} pos {pos:?} r {radius} alo98 {:?}", self.alo98.iter().map(|&o| (o, self.h.ee_sub_1a62e8(o))).collect::<Vec<_>>()); }
        self.h.explode_224bf8(obj, kind, pos, radius, damage);
    }
}

impl K24Host<'_> {
    /// guard hooks of Murray's pick-up FUN_001b70a0: FUN_00209a68(g, bit) (behaviour bit), FUN_0017b1a8(rec, 2)
    /// on the 0xb1 goal (REPLACED: the goal is unported; state 2 = the guard is carried -> brain child done)
    pub(super) fn r_ee_call(&mut self, addr: u32, args: &[u32]) -> i64 {
        match (addr, args) {
            (0x209a68, &[g, bit, ..]) => match self.guard(g) {
                Some((_, gd)) => gd.ee_sub_209a68(bit) as i64,
                None => self.h.ee_call(addr, args),
            },
            _ => self.h.ee_call(addr, args),
        }
    }
    /// FUN_001eeee0(g + 0x26c, kind) of a guard: its brain child goal of `kind` -> (record handle = the guard,
    /// +0x50 state, +0x5c holder). REPLACED: goal 0xb1 (sub 7) is the K32 `HitGoal` stand-in: +0x50 = 0 while
    /// it runs (3 once done), +0x5c = the hit source (the grabber)
    pub(super) fn r_ee_sub_1eeee0_rec(&mut self, obj: u32, k: i32) -> Option<(u32, i32, u32)> {
        let now = self.now;
        match self.guard(obj) {
            Some((_, g)) => {
                if k != 0xb1 || g.brain.sub.code != 7 {
                    return None;
                }
                match &g.brain.child {
                    super::guard::brain::Child::Hit(c) => Some((obj, if c.done(now) { 3 } else { 0 }, c.hit.src)),
                    _ => None,
                }
            }
            None => self.h.ee_sub_1eeee0_rec(obj, k),
        }
    }
    /// guard +0xfec (carry point; REPLACED: the guard itself, its +0x180 = the guard)
    pub(super) fn r_obj_i32(&mut self, obj: u32, off: u32) -> i32 {
        if matches!(off, 0xfec | 0x180) && self.guard(obj).is_some() {
            return obj as i32;
        }
        self.h.obj_i32(obj, off)
    }
}
