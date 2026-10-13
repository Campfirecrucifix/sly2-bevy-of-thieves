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
            Some((_, g)) => g.k24_set_vel(v),
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
            let mut kh = K24Host { h, k6: &mut k6, rng: &mut rng, now: self.g.t, dt: self.g.dt };
            f(self, &mut kh);
        }
        self.rng = rng;
        k6.append(&mut self.k6);
        self.k6 = k6;
    }
}
