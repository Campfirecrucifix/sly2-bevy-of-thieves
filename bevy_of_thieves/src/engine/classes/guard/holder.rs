//! CRAWL R26: the guard's pick-pocketable item holder (+0x494, class 99 C2d5450, size 0x170) and the guard
//! queue functions that arm / throw it: FUN_001f1cf8, FUN_001f1e20, FUN_001c2dc8, FUN_001c2ef0.
//! Holder fields: +0x150 throw origin (FUN_001f4c58(holder, kind, 0xd)), +0x154 spawner, +0x158 armed object.
//! REPLACED: +0x150 = the guard's own position (FUN_001f4c58's ALO lookup is not modelled); a spawner
//! (FUN_00132b10 6, the 0x2b3 coin pool) yields free class-91 K8 pickups (FUN_001cb250); FUN_001f1900 /
//! FUN_001f17f8 (the loot shine state +0x3fc) are not modelled.
use super::*;
use crate::engine::classes::k8::Cx as K8Cx;
use crate::engine::player::p4::FUN_00132b10 as is_a;

/// class 99 C2d5450 state (+0x150..+0x158)
#[derive(Clone, Debug, Default)]
pub struct Holder99 {
    pub f_150: bool,
    /// +0x154 spawner: Some(class) of the free K8 pool it yields (REPLACED fallback)
    pub f_154: Option<ClassId>,
    /// +0x154 spawner object handle (FUN_001cb250's argument), 0 = none
    pub f_154_sp: u32,
    /// +0x158 armed object (K8 pickup world index + 1), 0 = none
    pub f_158: u32,
}

impl ClassSet {
    pub fn k8_class(&self, hd: u32) -> Option<ClassId> {
        self.k8.iter().find(|x| x.0 as u32 + 1 == hd).and_then(|x| if let Inst::K8(c) = &x.1 { Some(c.class) } else { None })
    }

    fn k8_set_state(&mut self, hd: u32, s: i32, h: &mut dyn ClassHost) {
        let g = &mut K8Cx { g: &mut self.g, rng: &mut self.rng };
        if let Some((i, Inst::K8(c))) = self.k8.iter_mut().find(|x| x.0 as u32 + 1 == hd) {
            c.set_state(*i, s, g, h);
        }
    }

    /// FUN_001cb250(spawner, 0): the first free pooled object of the spawner (REPLACED: free K8 of `class`)
    fn ee_sub_1cb250(&mut self, class: ClassId) -> u32 {
        self.k8.iter().find_map(|(i, x)| match x {
            Inst::K8(c) if c.class == class && !c.in_world && (c.f_150 == 2 || c.f_150 == -1) => Some(*i as u32 + 1),
            _ => None,
        }).unwrap_or(0)
    }

    /// FUN_001c2dc8(holder, src, a2, kind): arm the holder with `src`; 0 = already armed, 1 = spawner empty,
    /// 2 = not armable, 3 = armed
    fn ee_sub_1c2dc8(&mut self, gi: usize, src: u32, h: &mut dyn ClassHost) -> u32 {
        let Some(Inst::Guard(g)) = self.k6.get(gi).map(|x| &x.1) else { return 2 };
        if g.holder.f_158 != 0 {
            return 0;
        }
        let spawner = (h.ee_sub_132b10(src, 6) != 0).then(|| 91);
        let obj = match spawner {
            Some(c) => {
                // FEEL R37: FUN_001cb250(src, 0) claims the spawner's next copy and Adds it (FUN_001caec8 ->
                // vtable +0x44): the armed coin is in the world. Hosts without the entry pool: free K8 of the class.
                let x = match h.ee_sub_1cb250(src, 0) {
                    0 => self.ee_sub_1cb250(c),
                    x => x,
                };
                if x == 0 {
                    return 1;
                }
                x
            }
            None => {
                if h.ee_sub_132b10(src, 0x30) == 0 && self.k8_class(src).map_or(true, |c| !is_a(c as i32, 0x30)) {
                    return 2;
                }
                src
            }
        };
        if let Some((_, Inst::Guard(g))) = self.k6.get_mut(gi) {
            g.holder.f_154 = spawner;
            g.holder.f_154_sp = if spawner.is_some() { src } else { 0 };
            g.holder.f_158 = obj;
            g.holder.f_150 = true;
        }
        // class 0x5e FUN_001c7a58: no K8 class is-a 0x5e. Class 0x5a: slot 75 SetState(2), else slot 8
        if self.k8_class(obj).is_some_and(|c| is_a(c as i32, 0x5a)) {
            self.k8_set_state(obj, 2, h);
        } else {
            h.vcall(obj, 0x4c);
        }
        3
    }

    /// FUN_001f1cf8(g): arm the holder with the first live queue entry from +0x488; a class-0x5c target whose
    /// task (+0x230) is in state -1 / 0 blocks (+0x48c = 1, retried each frame by FUN_00207350); state 1 arms,
    /// other states skip the entry
    pub fn ee_sub_1f1cf8(&mut self, gi: usize, h: &mut dyn ClassHost) {
        let Some((gobj, Inst::Guard(g))) = self.k6.get_mut(gi) else { return };
        let me = *gobj as u32 + 1;
        g.f_48c = 0;
        loop {
            let Some((_, Inst::Guard(g))) = self.k6.get(gi) else { return };
            if g.f_488 >= g.f_404.len() as i32 {
                break;
            }
            let id = g.f_404[g.f_488 as usize][0];
            let t = h.ee_sub_15e938(4, id, me);
            if t != 0 {
                let mut go = true;
                if self.k8_class(t).is_some_and(|c| is_a(c as i32, 0x5c)) || h.ee_sub_132b10(t, 0x5c) != 0 {
                    let task = self.k8.iter().find(|x| x.0 as u32 + 1 == t).and_then(|x| if let Inst::K8(c) = &x.1 { Some(c.f_230_obj) } else { None }).unwrap_or(0);
                    if task != 0 {
                        let st = h.task_state_54(task);
                        if (st as u32).wrapping_add(1) < 2 {
                            if let Some((_, Inst::Guard(g))) = self.k6.get_mut(gi) {
                                g.f_48c = 1;
                            }
                            break;
                        }
                        go = st == 1;
                    }
                }
                if go {
                    if self.ee_sub_1c2dc8(gi, t, h) == 1 {
                        if let Some((_, Inst::Guard(g))) = self.k6.get_mut(gi) {
                            g.f_48c = 1;
                        }
                    }
                    break;
                }
            }
            if let Some((_, Inst::Guard(g))) = self.k6.get_mut(gi) {
                g.f_488 += 1;
            }
        }
    }

    /// FUN_001c2ef0(holder, n): throw the armed object (then n - 1 more from the spawner) at the player:
    /// aim = player + FUN_0011e1a0(rng(-1.5, 1.5), rng(100, 150), -75) in the horizontal from-holder frame,
    /// v = FUN_001d69f0(rng(0.75, 1.2), origin, aim, (0, 0, -2250)); class 0x5a launch FUN_001c6bd0(0.05 i,
    /// 0x5b / 0x5d ? 0.15 i : FLT_MAX, v) (state 3). Clears the holder (FUN_001c2db8); returns the first.
    fn ee_sub_1c2ef0(&mut self, gi: usize, n: i32, h: &mut dyn ClassHost) -> u32 {
        let Some((_, Inst::Guard(g))) = self.k6.get(gi) else { return 0 };
        let (armed, spawner, sp, tpos) = (g.holder.f_158, g.holder.f_154, g.holder.f_154_sp, g.body.pos);
        let mut first = 0;
        for i in 0..n {
            let obj = if i == 0 {
                armed
            } else {
                let Some(c) = spawner else { break };
                let x = match h.ee_sub_1cb250(sp, 0) {
                    0 => self.ee_sub_1cb250(c),
                    x => x,
                };
                if x == 0 {
                    break;
                }
                self.k8_set_state(x, 2, h);
                x
            };
            if obj == 0 {
                break;
            }
            if first == 0 {
                first = obj;
            }
            let pp = h.ee_sub_1a62e8(PLAYER_HANDLE);
            let ang = self.rng.FUN_0011d448(-1.5, 1.5);
            let dist = self.rng.FUN_0011d448(100.0, 150.0);
            let off = crate::engine::player::FUN_0011e1a0(ang, dist, -75.0);
            let d = [pp[0] - tpos[0], pp[1] - tpos[1], 0.0];
            let l = (d[0] * d[0] + d[1] * d[1]).sqrt();
            let dir = if l < 0.0001 { [1.0, 0.0] } else { [d[0] / l, d[1] / l] };
            // FUN_0011b830(dir): x = dir, y = up x dir, z = up
            let aim = [pp[0] + dir[0] * off[0] - dir[1] * off[1], pp[1] + dir[1] * off[0] + dir[0] * off[1], pp[2] + off[2]];
            let t = self.rng.FUN_0011d448(0.75, 1.2);
            let gz = -2250.0f32;
            let v = [(aim[0] - tpos[0]) / t, (aim[1] - tpos[1]) / t, (aim[2] - tpos[2]) / t - gz * t * 0.5, 0.0];
            let cls = self.k8_class(obj);
            if cls.is_some_and(|c| is_a(c as i32, 0x5a)) {
                let c = cls.unwrap();
                let spread = if is_a(c as i32, 0x5b) || is_a(c as i32, 0x5d) { i as f32 * 0.15 } else { f32::MAX };
                let now = self.g.t;
                let gx = &mut K8Cx { g: &mut self.g, rng: &mut self.rng };
                if let Some((o, Inst::K8(p))) = self.k8.iter_mut().find(|x| x.0 as u32 + 1 == obj) {
                    p.in_world = true;
                    h.vcall(obj, 0x44);
                    p.f_170 = [tpos[0], tpos[1], tpos[2], 1.0];
                    p.xf_pos = p.f_170;
                    p.f_160 = i as f32 * 0.05;
                    p.f_1c4 = 0;
                    p.set_state(*o, 3, gx, h);
                    p.f_180 = v;
                    p.xf_vel = v;
                    p.f_1cc = if spread == f32::MAX { f32::MAX } else { now + spread };
                }
            } else {
                h.vcall(obj, 0x44);
            }
        }
        // FUN_001c2db8
        if let Some((_, Inst::Guard(g))) = self.k6.get_mut(gi) {
            g.holder = Holder99::default();
        }
        first
    }

    /// FUN_001f1e20(g) (JT event 0x167 on its focus guard): event 0x19 on 0x2deb30, throw the holder with the
    /// entry's +0xc count, cursor + 1, FUN_001f1cf8; 0 when the queue is done. `guard` = handle.
    pub fn ee_sub_1f1e20(&mut self, guard: u32, h: &mut dyn ClassHost) -> u32 {
        let Some(gi) = self.k6.iter().position(|x| x.0 as u32 + 1 == guard && matches!(x.1, Inst::Guard(_))) else { return 0 };
        let Some((_, Inst::Guard(g))) = self.k6.get(gi) else { return 0 };
        if g.f_488 >= g.f_404.len() as i32 {
            return 0;
        }
        let n = g.f_404[g.f_488 as usize][3];
        h.ee_sub_1619a0(0x19, guard);
        let r = self.ee_sub_1c2ef0(gi, n, h);
        if let Some((_, Inst::Guard(g))) = self.k6.get_mut(gi) {
            g.f_488 += 1;
        }
        self.ee_sub_1f1cf8(gi, h);
        r
    }
}
