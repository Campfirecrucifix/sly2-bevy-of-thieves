//! WP-K21: class 68 C2cd748 (vtable 0x2cd748; one instance, level 11): a button-driven dancer. Its own
//! slots are the 0x152xxx ones; the 0x203xxx..0x205xxx slots it defines are the PO base the player-like
//! classes (Bentley 55, Murray 54, guards) inherit (they reach them as `h.base`).
//! Move records (+0x2c4 count, 0x28-byte records at +0x2c8, field 1343 = FUN_001524d8): +0 button mask, then
//! three (oid, anim, flags) triples at +4 / +0x10 / +0x1c; +0x2c0 (field 1342) the idle anim; +0x408 the
//! playing anim, +0x40c "move running" (messages 0x27 / 0x28 of the anim), +0x190 the root-motion velocity.
//! 1:1: slot 4 FUN_00152250 (message 9), slot 11 FUN_001520c8, slot 14 FUN_00152100 + FUN_00152578, slot 15
//! FUN_00152148, slot 49 FUN_00152350, FUN_001524d8 (field 1343), FUN_00152520, FUN_00152390 (move by
//! button / mask), FUN_001524a0 (play into +0x408 = shared::anim::ee_sub_1305b8); PO base slots 13
//! FUN_002039c0, 18 FUN_00203ad0, 28 FUN_002041c0, 29 FUN_002041f8, 40 FUN_00203a08, 53 FUN_00204c60, 62
//! FUN_00204248, 84 FUN_002040f8, 85 FUN_00205d08, 86 FUN_00205d10.
//! REPLACED: FUN_001a6f00(parent, self, v) / FUN_001a6f00(self, 0, v) frame changes = `ee_call(0x1a6f00, ..)`
//! returning nothing (the +0x240 / +0x230 vectors take the world position); DAT_002de570 (the rhythm game
//! manager) = the ClassSet's class-121 instance (K22, classes/k22_121.rs; was `h.k21_global(0x2de570)`);
//! FUN_001c4700 (tracker buttons held) = `ee_call(0x1c4700, [tracker, mask])`.
//! UNPORTED: PO base slots 37 FUN_002052e0 (attach; its kind test argument is lost), 56 FUN_002044b0 /
//! 57 FUN_00204768 / 59 FUN_00204ac8 / 81 FUN_00205008 (the PO contact lists +0xec are not modelled).
#![allow(non_snake_case)]
use super::k17_55::Mem;
use super::*;

/// one +0x2c8 move record
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Move {
    pub mask: u32,
    /// (oid, anim, flags) x 3
    pub ent: [(i32, u32, u32); 3],
}

#[derive(Clone, Debug, Default)]
pub struct C2cd748 {
    pub m: Mem,
    pub moves: Vec<Move>,
}

impl C2cd748 {
    /// FUN_001524d8(self, mask, oid0, flags0, oid1, oid2): add a move (max 8)
    pub fn ee_sub_1524d8(&mut self, mask: u32, oid0: i32, flags0: u32, oid1: i32, oid2: i32) {
        if self.moves.len() < 8 {
            self.moves.push(Move { mask, ent: [(oid0, 0, flags0), (oid1, 0, 0), (oid2, 0, 0)] });
            self.m.set_i(0x2c4, self.moves.len() as i32);
        }
    }
    /// FUN_00152520(self, oid): the kind-4 object (anim) of `oid`, vtable +0x4c on it
    pub fn ee_sub_152520(obj: usize, oid: i32, h: &mut dyn ClassHost) -> u32 {
        let a = h.ee_sub_15e938(4, oid, obj as u32 + 1);
        if a != 0 {
            h.vcall(a, 0x4c);
        }
        a
    }
    /// FUN_00152578: resolve every triple with an oid
    pub fn ee_sub_152578(&mut self, obj: usize, h: &mut dyn ClassHost) {
        for k in 0..self.moves.len() {
            for j in 0..3 {
                if self.moves[k].ent[j].0 != -1 {
                    self.moves[k].ent[j].1 = Self::ee_sub_152520(obj, self.moves[k].ent[j].0, h);
                }
            }
        }
    }
    /// FUN_001524a0(self, anim, flags) = FUN_001305b8(0, 1.0, anim, self, flags | 1, &+0x408)
    pub fn ee_sub_1524a0(&mut self, obj: usize, anim: u32, flags: u32, h: &mut dyn ClassHost) {
        let mut slot = self.m.u(0x408);
        super::shared::anim::ee_sub_1305b8(0, 1.0, anim, obj as u32 + 1, flags | 1, &mut slot, h);
        self.m.set_u(0x408, slot);
    }
    /// FUN_00152390(self, mask, oid, k): the first move of `mask` (-1: whose buttons are held) plays its
    /// triple `k` (a given oid replaces triple 0's anim first)
    pub fn ee_sub_152390(&mut self, obj: usize, mask: i32, oid: i32, k: usize, h: &mut dyn ClassHost) {
        let o = obj as u32 + 1;
        for r in 0..self.moves.len() {
            let mv = self.moves[r];
            let hit = if mask == -1 { h.ee_call(0x1c4700, &[self.m.u(0x150), mv.mask]) != 0 } else { mv.mask as i32 == mask };
            if !hit {
                continue;
            }
            if oid != -1 {
                let a = Self::ee_sub_152520(obj, oid, h);
                if a != 0 {
                    self.moves[r].ent[0].1 = a;
                }
            }
            h.ee_sub_2303a0_off(o, 0x58, 2);
            h.ee_sub_2303a0_off(o, 0x5c, 2);
            let (_, a, f) = self.moves[r].ent[k];
            self.ee_sub_1524a0(obj, a, f, h);
            return;
        }
    }
    /// slot 4 FUN_00152250(self, msg, args): message 9 from the own anim: 0x28 move over, 0x27 move running,
    /// 0x15e root motion of anim args[1] -> +0x190 and the velocity (vtable +0x11c)
    pub fn ee_sub_152250(&mut self, obj: usize, msg: i32, from_self: bool, args: [u32; 3], h: &mut dyn ClassHost) {
        h.base(0x1c3e20, obj);
        if msg != 9 || !from_self {
            return;
        }
        match args[2] {
            0x28 => self.m.set_i(0x40c, 0),
            0x27 => self.m.set_i(0x40c, 1),
            0x15e => {
                let r = h.root_vel_204e38(args[1]);
                self.m.set_v3(0x190, [-r[0], -r[1], -r[2]]);
                let mut v = [0f32; 3];
                for c in 0..3 {
                    v[c] = self.m.f(0x1b0 + c as u32 * 4) * r[0] + self.m.f(0x1c0 + c as u32 * 4) * r[1] + self.m.f(0x1d0 + c as u32 * 4) * r[2];
                }
                h.vcall_11c_v(obj as u32 + 1, v);
            }
            _ => {}
        }
    }
    /// slot 11 FUN_001520c8: FUN_001f2d48, anim stopped
    pub fn ee_sub_1520c8(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1f2d48, obj);
        self.ee_sub_1524a0(obj, 0, 0, h);
    }
    /// slot 14 FUN_00152100: PO load-post FUN_00203a38, resolve the moves, vtable +0x27c(self, 0)
    pub fn ee_sub_152100(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x203a38, obj);
        self.ee_sub_152578(obj, h);
        h.vcall_arg(obj as u32 + 1, 0x27c, 0);
    }
    /// slot 15 FUN_00152148: PO update FUN_00203fc0; idle when the move ended; the manager drives the moves
    /// (state 0: by buttons unless running, 2: FUN_001533b0); +0x190 from the anim's root motion
    pub fn ee_sub_152148(&mut self, obj: usize, mgr: Option<(usize, &mut super::k22_121::C2cd5b8)>, now: f32, h: &mut dyn ClassHost) {
        let o = obj as u32 + 1;
        h.base(0x203fc0, obj);
        let a = self.m.u(0x408);
        if a == 0 || h.ee_sub_131968(a) {
            h.ee_sub_2303a0_off(o, 0x58, 1);
            h.ee_sub_2303a0_off(o, 0x5c, 1);
            self.m.set_i(0x40c, 0);
            self.ee_sub_1524a0(obj, self.m.u(0x2c0), 8, h);
        }
        let st = mgr.as_ref().map_or(0, |m| m.1.st);
        match st {
            0 => {
                if self.m.i(0x40c) == 0 {
                    self.ee_sub_152390(obj, -1, -1, 0, h);
                }
            }
            2 => {
                if let Some((mi, m)) = mgr {
                    m.ee_sub_1533b0(mi, obj, self, now, h);
                }
            }
            _ => {}
        }
        let a = self.m.u(0x408);
        if a != 0 {
            let r = h.root_vel_204e38(a);
            self.m.set_v3(0x190, [-r[0], -r[1], -r[2]]);
        }
    }
    /// slot 49 FUN_00152350(self, rec): FUN_001f58f0, rec +0x30 -= 3pi/4
    pub fn ee_sub_152350(&mut self, obj: usize, rec30: f32, h: &mut dyn ClassHost) -> f32 {
        h.base(0x1f58f0, obj);
        rec30 - 2.3561945
    }

    // ---- PO base slots defined by this vtable
    /// slot 13 FUN_002039c0: FUN_001c3dc8, +0x240 = the origin in the parent frame, FUN_001f8300
    pub fn ee_sub_2039c0(&mut self, obj: usize, h: &mut dyn ClassHost) {
        let o = obj as u32 + 1;
        h.base(0x1c3dc8, obj);
        h.ee_call(0x1a6f00, &[self.m.u(0x10), o, 0]);
        self.m.set_v3(0x240, h.ee_sub_1a62e8(o));
        h.ee_call(0x1f8300, &[o]);
    }
    /// slot 18 FUN_00203ad0 = FUN_001f3c70
    pub fn ee_sub_203ad0(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1f3c70, obj);
    }
    /// slot 28 FUN_002041c0: FUN_001f40a0, +0x230 = +0x240 in the world
    pub fn ee_sub_2041c0(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1f40a0, obj);
        let v = self.m.v3(0x240);
        h.ee_call(0x1a6f00, &[obj as u32 + 1, 0, v[0].to_bits(), v[1].to_bits(), v[2].to_bits()]);
        self.m.set_v3(0x230, v);
    }
    /// slot 29 FUN_002041f8: FUN_001f42e0, yaw = FUN_00204740 -> FUN_00204660; tracker +0x18 = +0x1a8
    pub fn ee_sub_2041f8(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1f42e0, obj);
        let yaw = h.ee_sub_204740(obj);
        self.m.set_f(0x1a8, yaw);
        let r = crate::engine::camera::math::FUN_0011b500(yaw, [0.0, 0.0, 1.0]);
        for k in 0..3 {
            self.m.set_v3(0x1b0 + k as u32 * 0x10, [r[k][0], r[k][1], r[k][2]]);
        }
        let t = self.m.u(0x150);
        if t != 0 {
            h.obj_set_u32(t, 0x18, self.m.u(0x1a8));
        }
    }
    /// slot 40 FUN_00203a08: FUN_001c3df8, FUN_001eec68(self + 0x26c, self)
    pub fn ee_sub_203a08(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1c3df8, obj);
        h.ee_call(0x1eec68, &[obj as u32 + 1]);
    }
    /// slot 53 FUN_00204c60(self, x): FUN_001f5988; +0x208 cleared when its +0x48 is `x`
    pub fn ee_sub_204c60(&mut self, obj: usize, x: u32, h: &mut dyn ClassHost) {
        h.base(0x1f5988, obj);
        let c = self.m.u(0x208);
        if c != 0 && h.obj_i32(c, 0x48) as u32 == x {
            self.m.set_u(0x208, 0);
        }
    }
    /// slot 62 FUN_00204248
    pub fn ee_sub_204248(&self) -> i32 {
        1
    }
    /// slot 84 FUN_002040f8: 7 unless +0x204
    pub fn ee_sub_2040f8(&self) -> i32 {
        if self.m.i(0x204) != 0 {
            0
        } else {
            7
        }
    }
    /// slot 85 FUN_00205d08 / slot 86 FUN_00205d10
    pub fn ee_sub_205d08(&self) -> i32 {
        0
    }
    pub fn ee_sub_205d10(&self) {}
}

/// K21 class 68 in the ClassSet
impl ClassSet {
    pub(super) fn build_k21_68(&mut self, w: &crate::engine::world::World, h: &mut dyn ClassHost) {
        use crate::engine::world::props::{ArrayItem, PropValue};
        let word = |it: &ArrayItem| match it {
            ArrayItem::One(_, b) if b.len() >= 4 => u32::from_le_bytes([b[0], b[1], b[2], b[3]]),
            _ => 0,
        };
        for (i, o) in w.objects.iter().enumerate() {
            if o.class != 68 {
                continue;
            }
            let mut c = C2cd748::default();
            // slot 6 FUN_00203948 (PO init, ported K17) = `h.base`
            h.base(0x203948, i);
            if let Some(ps) = w.objects.get(o.clone_of.unwrap_or(i)).and_then(crate::engine::game::fields::object_props) {
                for x in ps {
                    match (x.field, &x.value) {
                        (1342, PropValue::Scalar(b)) if b.len() >= 4 => c.m.set_u(0x2c0, u32::from_le_bytes([b[0], b[1], b[2], b[3]])),
                        (1343, PropValue::Array(a)) if a.len() == 5 => {
                            c.ee_sub_1524d8(word(&a[0]), word(&a[1]) as i32, word(&a[2]), word(&a[3]) as i32, word(&a[4]) as i32)
                        }
                        _ => {}
                    }
                }
            }
            self.k21c.push((i, Inst::K21C68(Box::new(c))));
        }
    }
    pub(super) fn post_load_k21_68(&mut self, h: &mut dyn ClassHost) {
        for (i, x) in &mut self.k21c {
            if let Inst::K21C68(c) = x {
                c.ee_sub_2039c0(*i, h);
                c.ee_sub_152100(*i, h);
            }
        }
    }
    pub(super) fn frame_k21_68(&mut self, h: &mut dyn ClassHost) {
        let now = h.k22_music_1fe3f8();
        for (i, x) in &mut self.k21c {
            // FEEL R33: slot 15 runs from the world update list only (LO +0x28 bit 60)
            if h.lo_removed(*i) {
                continue;
            }
            if let Inst::K21C68(c) = x {
                let mgr = self.k22c.iter_mut().rev().find_map(|(j, y)| match y {
                    Inst::K22C121(m) => Some((*j, &mut **m)),
                    _ => None,
                });
                c.ee_sub_152148(*i, mgr, now, h);
                c.ee_sub_203ad0(*i, h);
            }
        }
    }
}
