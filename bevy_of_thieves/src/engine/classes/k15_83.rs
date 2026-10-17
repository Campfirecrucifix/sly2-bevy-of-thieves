//! WP-K15: class 83 C2cc950 (parent class 48 ALO; the DB vtable tree's 0x2cc548 parent is wrong, its slots 4 /
//! 6 / 14 are all overridden): a switch. The toggle block at +0x150 (`Toggle`, FUN_00148xxx; also embedded at
//! +0x130 by another class, FUN_00149718) holds the state (+0x1e0): 0 off, 1 on, 2 on and holding, 3 latched,
//! 4 releasing; off -> on plays the half-0 anim list (+0x154), the half-0 state-machine list (+0x204, method row
//! 424), the sequence (+0x1fc) and the signal list (+0x288, row 425); on -> off the half-1 lists. +0x2a8 (row
//! 413) is the trigger mode: 0 touched by an accepted object (oid list +0x2b0 or class list +0x2d4, FUN_00149100;
//! the deferred FUN_00149010 skips the player while it is landing / in kind 0x11), 1 / 2 hit (slot 74), 3 the
//! player's power-up id (DAT_002e1e40 +0x18c4 while +0x18c8 == 3) equals +0x2f4 (row 414). State 2 times out
//! after +0x1e8 (row 419) to 4, state 4 returns to 0 (mode 1) or 1.
//! Own slots 5 FUN_00113f40, 6 FUN_00148bb0, 11 FUN_001492b0, 13 FUN_00148c08, 14 FUN_00148c38, 15
//! FUN_00148c98, 40 FUN_00148c70, 74 FUN_00148f20 1:1 with FUN_00148718 / 148738 / 148740 / 148668 / 148a78 /
//! 1487a0 / 148860 / 1488d8 / 148620 / 1490a0 / 149100 / 149010, FUN_0020fec0, FUN_00161f10 and the row methods
//! FUN_001491e0 / 149200 / 149270 / 149290 (thunks 0x104f40 / 104fe8 / 105090 / 105158).
//! REPLACED: the SO contact walk of slot 15 (+0xec +0x64, kind != 10) -> `ClassHost::so_contacts_57`; the
//! deferred FUN_00149ab8(PMF 0x2a4e80) -> `C2cc950::pending` run after the frame; FUN_00130598 (anim play
//! through FUN_001304e0, argument order lost in the decompile) -> `ClassHost::ee_sub_130598`; the anim's owner
//! oid (**(anim +0x14) +0x18) -> `ClassHost::anim_oid_14`; FUN_001fbb90(2) + FUN_001fb2f8(.., owner) ->
//! `ClassHost::ee_sub_1fbb90(obj, 2)`; DAT_002e1e40 +0x18c8 / +0x18c4 -> `ClassHost::sly_18c4`.
#![allow(non_snake_case)]
use super::k15_101::Contact57;
use super::k15_79::method_args;
use super::*;

/// slot 5 FUN_00113f40 (class descriptor getter)
pub fn slot5(class: ClassId) -> Option<u32> {
    match class {
        83 => Some(0x2bb164),
        _ => None,
    }
}

/// class 83 field rows -> member offsets (FIELD_MEMBER; 415 / 416 / 424 / 425 methods, 417 / 418 lists)
pub const ROWS_83: &[(i16, u16)] =
    &[(413, 0x2a8), (414, 0x2f4), (419, 0x1e8), (420, 0x1dc), (421, 0x1ec), (422, 0x1f0), (423, 0x1f4), (426, 0x1e0), (427, 0x2f8)];

/// DAT_002a4e80: PMF {vindex -1, FUN_00149010} (deferred toucher check)
pub const PMF_2A4E80: u32 = 0x149010;
/// FUN_00149010: a landing player (+0x2c4 == 3) counts only after 0.15 s
pub const D_149010: f32 = 0.15;

/// the FUN_00148xxx toggle block (+0x150 in class 83)
#[derive(Clone, Debug, PartialEq)]
pub struct Toggle {
    /// +0 owner handle
    pub owner: u32,
    /// +4 / +0x48 anim lists (half 0 on, half 1 off): {anim oid -> object, word} (8 max)
    pub anims: [Vec<[u32; 2]>; 2],
    /// +0x8c rate, +0x90 state, +0x94 its start time, +0x98 hold time
    pub rate: f32,
    pub state: i32,
    pub t0: f32,
    pub hold: f32,
    /// +0x9c no activation sound, +0xa0 anim mode, +0xa4 stay in 1
    pub no_snd: u32,
    pub anim_mode: u32,
    pub stay1: u32,
    /// +0xa8 sequence user, +0xac sequence
    pub seq_user: u32,
    pub seq: u32,
    /// +0xb0 / +0xb4 state-machine list {oid, id, on state, off state} (8 max, FUN_0020fec0)
    pub sms: Vec<[u32; 4]>,
    /// +0x134 / +0x138 signal list {oid, word} (4 max, FUN_00161f10)
    pub sigs: Vec<[u32; 2]>,
}

impl Default for Toggle {
    fn default() -> Self {
        Toggle { owner: 0, anims: [Vec::new(), Vec::new()], rate: 0.0, state: 0, t0: 0.0, hold: 0.0, no_snd: 0, anim_mode: 0, stay1: 0, seq_user: 0, seq: 0, sms: Vec::new(), sigs: Vec::new() }
    }
}

impl Toggle {
    /// FUN_00148718: state -1, hold 1, rate 1
    pub fn ee_sub_148718(&mut self) {
        self.state = -1;
        self.hold = 1.0;
        self.rate = 1.0;
    }

    /// FUN_00148620(list, a, b): append (8 max)
    pub fn ee_sub_148620(list: &mut Vec<[u32; 2]>, a: u32, b: u32) -> bool {
        if list.len() < 8 {
            list.push([a, b]);
            true
        } else {
            false
        }
    }

    /// FUN_0020fec0(.., list, 8, count, up, oid, id, state): an entry with that oid and id (-1 = any) gets the on
    /// (`up` != 0, +8) or off (+0xc) state; else a new entry with the other state -1 (`count + 1 < 8`)
    pub fn ee_sub_20fec0(&mut self, up: u32, oid: u32, id: u32, state: u32) -> bool {
        for e in &mut self.sms {
            if e[0] == oid && (e[1] as i32 == -1 || e[1] == id) {
                e[if up == 0 { 3 } else { 2 }] = state;
                return true;
            }
        }
        if self.sms.len() + 1 < 8 {
            self.sms.push(if up == 0 { [oid, id, u32::MAX, state] } else { [oid, id, state, u32::MAX] });
            return true;
        }
        false
    }

    /// FUN_00161f10(.., list, 4, count, a, b)
    pub fn ee_sub_161f10(&mut self, a: u32, b: u32) -> bool {
        if self.sigs.len() < 4 {
            self.sigs.push([a, b]);
            true
        } else {
            false
        }
    }

    /// FUN_00148740 / FUN_00148668: both anim lists' oids resolved (FUN_0015e938(4, oid, owner); 0 when missing)
    /// and loaded (vtable +0x4c)
    pub fn ee_sub_148740(&mut self, h: &mut dyn ClassHost) {
        let owner = self.owner;
        for l in &mut self.anims {
            for e in l.iter_mut() {
                let o = h.ee_sub_15e938(4, e[0] as i32, owner);
                e[0] = o;
                if o != 0 {
                    h.vcall(o, 0x4c);
                }
            }
        }
    }

    /// FUN_001488d8(half): the half's anims (target = the owner, or the anim owner's oid object, none for -1;
    /// FUN_00130598 by the anim mode), the state machines (FUN_0020ff70, on states for half 0); half 0 also
    /// starts the sequence (FUN_001dfd58) and the signals (FUN_00161f48)
    pub fn ee_sub_1488d8(&self, half: usize, h: &mut dyn ClassHost) {
        let owner = self.owner;
        for e in &self.anims[half] {
            let anim = e[0];
            if anim == 0 {
                continue;
            }
            let oid = h.anim_oid_14(anim, owner);
            let target = if h.obj_i32(owner, 0x18) == oid {
                owner
            } else if oid == -1 {
                0
            } else {
                h.ee_sub_15e938(4, oid, owner)
            };
            if self.anim_mode == 0 {
                h.ee_sub_130598(anim, target, self.rate, 1, e[1]);
            } else {
                h.ee_sub_130598(anim, target, self.rate, if self.rate < 0.0 { 2 } else { 0 }, e[1]);
            }
        }
        let obj = owner.wrapping_sub(1) as usize;
        let sm: Vec<[u32; 3]> = self.sms.iter().map(|e| [e[0], e[1], if half == 0 { e[2] } else { e[3] }]).collect();
        h.ee_sub_20ff70(obj, &sm);
        if half == 0 {
            if self.seq != 0 {
                h.ee_sub_1dfd58(self.seq, self.seq_user);
            }
            h.ee_sub_161f48_list(obj, &self.sigs);
        }
    }

    /// FUN_00148a78(state): on a change: state, time; 0 -> 1 FUN_001487a0, (not from -1) -> 0 FUN_00148860
    pub fn ee_sub_148a78(&mut self, s: i32, t: f32, h: &mut dyn ClassHost) {
        let old = self.state;
        if old == s {
            return;
        }
        self.state = s;
        self.t0 = t;
        if old == 0 && s == 1 {
            self.ee_sub_1487a0(t, h);
        }
        if old != -1 && s == 0 {
            self.ee_sub_148860(h);
        }
    }

    /// FUN_001487a0 (on): owner slot 3 event 5; unless +0x9c the activation sound (FUN_001fbb90(2)); half 0;
    /// without +0xa4 straight to state 2; owner slot 5 (vtable +0x24) event 6
    pub fn ee_sub_1487a0(&mut self, t: f32, h: &mut dyn ClassHost) {
        let obj = self.owner.wrapping_sub(1) as usize;
        h.vcall_1c(obj, 5, 0);
        if self.no_snd == 0 {
            h.ee_sub_1fbb90(obj, 2);
        }
        self.ee_sub_1488d8(0, h);
        if self.stay1 == 0 {
            self.ee_sub_148a78(2, t, h);
        }
        h.vcall_24(obj, 6, self.owner);
    }

    /// FUN_00148860 (off): owner slot 3 event 6; half 1; owner vtable +0x24 event 7
    pub fn ee_sub_148860(&self, h: &mut dyn ClassHost) {
        let obj = self.owner.wrapping_sub(1) as usize;
        h.vcall_1c(obj, 6, 0);
        self.ee_sub_1488d8(1, h);
        h.vcall_24(obj, 7, self.owner);
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct C2cc950 {
    /// +0x40 flags
    pub f_40: u32,
    /// +0x150 toggle block
    pub tg: Toggle,
    /// +0x2a8 trigger mode (row 413)
    pub f_2a8: i32,
    /// +0x2ac / +0x2b0 accepted objects (row 417, handles after load), +0x2d0 / +0x2d4 accepted classes (row 418)
    pub objs: Vec<u32>,
    pub oids: Vec<i32>,
    pub classes: Vec<i32>,
    /// +0x2f4 power-up id (row 414), +0x2f8 toucher (row 427)
    pub f_2f4: i32,
    pub f_2f8: u32,
    /// FUN_00149ab8(PMF 0x2a4e80) queued
    pub pending: bool,
}

fn handle(obj: usize) -> u32 {
    obj as u32 + 1
}

impl C2cc950 {
    /// slot 6 FUN_00148bb0: FUN_001f2108; FUN_00148718; +0x40 |= 2
    pub fn ee_sub_148bb0(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1f2108, obj);
        self.tg.ee_sub_148718();
        self.f_40 |= 2;
    }

    /// slot 13 FUN_00148c08: FUN_001f31e0; owner (FUN_00148738)
    pub fn ee_sub_148c08(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1f31e0, obj);
        self.tg.owner = handle(obj);
    }

    /// slot 14 FUN_00148c38: FUN_001f36a8; anim lists resolved (FUN_00148740); state 0
    pub fn ee_sub_148c38(&mut self, obj: usize, t: f32, h: &mut dyn ClassHost) {
        h.base(0x1f36a8, obj);
        self.tg.ee_sub_148740(h);
        self.tg.ee_sub_148a78(0, t, h);
    }

    /// slot 40 FUN_00148c70: FUN_001f6068; owner
    pub fn ee_sub_148c70(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1f6068, obj);
        self.tg.owner = handle(obj);
    }

    /// slot 11 FUN_001492b0: FUN_001f2d48; on (1 / 2) -> 0
    pub fn ee_sub_1492b0(&mut self, obj: usize, t: f32, h: &mut dyn ClassHost) {
        h.base(0x1f2d48, obj);
        if (self.tg.state as u32).wrapping_sub(1) < 2 {
            self.tg.ee_sub_148a78(0, t, h);
        }
    }

    /// FUN_00149100(self, o): with an object list `o` must be in it (then the class list too, if any); else in
    /// the class list (an empty list accepts all)
    pub fn ee_sub_149100(&self, o: u32, h: &mut dyn ClassHost) -> bool {
        if !self.objs.is_empty() && !self.objs.contains(&o) {
            return false;
        }
        self.classes.is_empty() || self.classes.iter().any(|&c| h.ee_sub_132b10(o, c) != 0)
    }

    /// DAT_002e1e40 +0x18c8 == 3 && +0x18c4 == +0x2f4
    fn powerup(&self, h: &mut dyn ClassHost) -> bool {
        h.sly_18c4() == Some(self.f_2f4)
    }

    /// slot 15 FUN_00148c98: FUN_001f36e0; off / on: mode 0 an accepted contact (kind != 10) -> toucher +0x2f8,
    /// deferred FUN_00149010, state kept; none -> 0. Mode 3 -> the power-up test. State 2: the power-up refreshes
    /// the time; held longer than +0x1e8 (>= 0) -> 4. State 4 -> mode 1: 0, modes 0 / 2 / 3: 1.
    pub fn ee_sub_148c98(&mut self, obj: usize, contacts: &[Contact57], t: f32, h: &mut dyn ClassHost) {
        h.base(0x1f36e0, obj);
        match self.tg.state {
            0 | 1 => {
                let mode = self.f_2a8;
                let mut s = self.tg.state;
                if mode < 3 {
                    if mode == 0 {
                        s = 0;
                        for c in contacts {
                            if c.kind != 10 && self.ee_sub_149100(c.other, h) {
                                self.f_2f8 = c.other;
                                self.pending = true;
                                s = self.tg.state;
                                break;
                            }
                        }
                    }
                } else if mode == 3 {
                    s = self.powerup(h) as i32;
                }
                self.tg.ee_sub_148a78(s, t, h);
            }
            2 => {
                if self.powerup(h) {
                    self.tg.t0 = t;
                }
                let f = self.tg.hold;
                if 0.0 <= f && f < t - self.tg.t0 {
                    self.tg.ee_sub_148a78(4, t, h);
                }
            }
            4 => {
                let s = match self.f_2a8 {
                    1 => 0,
                    0 | 2 | 3 => 1,
                    _ => return,
                };
                self.tg.ee_sub_148a78(s, t, h);
            }
            _ => {}
        }
    }

    /// FUN_00149010 (deferred): a player toucher (class 0x34) in kind 0x11 (+0x2dc) or landing (+0x2c4 == 3 for
    /// less than 0.15 s, +0x2e0) is dropped; else state 1
    pub fn ee_sub_149010(&mut self, t: f32, h: &mut dyn ClassHost) {
        let o = self.f_2f8;
        let skip = h.ee_sub_132b10(o, 0x34) != 0 && (h.obj_i32(o, 0x2dc) == 0x11 || (h.obj_i32(o, 0x2c4) == 3 && t - h.obj_f32(o, 0x2e0) < D_149010));
        if skip {
            self.f_2f8 = 0;
        } else {
            self.tg.ee_sub_148a78(1, t, h);
        }
    }

    /// slot 74 FUN_00148f20(self, hit): taken = FUN_001f7588 (+0x44); unless +0x44 bit 2, an accepted source
    /// switches: mode 1 off -> 1; mode 2 off (held longer than +0x1e8 since the last change) -> 1, on -> 0
    pub fn ee_sub_148f20(&mut self, hit: &guard::slots::GuardHit, t: f32, h: &mut dyn ClassHost) -> bool {
        let taken = hit.f_44 != 0;
        let flags = hit.f_44 as u32;
        if !taken || flags & 4 != 0 || !self.ee_sub_149100(hit.src, h) {
            return taken;
        }
        let s = self.tg.state;
        match self.f_2a8 {
            1 if s == 0 => self.tg.ee_sub_148a78(1, t, h),
            2 if s == 1 => self.tg.ee_sub_148a78(0, t, h),
            2 if s == 0 && self.tg.hold <= t - self.tg.t0 => self.tg.ee_sub_148a78(1, t, h),
            _ => {}
        }
        taken
    }

    /// rows `ROWS_83`, the list rows 417 / 418 and the methods 415 / 416 (FUN_00148620 into the half-0 / half-1
    /// anim list), 424 (FUN_0020fec0), 425 (FUN_00161f10)
    pub fn feed(&mut self, props: &[crate::engine::world::props::Prop]) {
        use super::k4::word;
        use crate::engine::world::props::PropValue;
        for p in props {
            let a = method_args(&p.value);
            let g = |i: usize| a.get(i).copied().unwrap_or(0);
            match p.field {
                415 => {
                    Toggle::ee_sub_148620(&mut self.tg.anims[0], g(0), g(1));
                }
                416 => {
                    Toggle::ee_sub_148620(&mut self.tg.anims[1], g(0), g(1));
                }
                424 => {
                    self.tg.ee_sub_20fec0(g(0), g(1), g(2), g(3));
                }
                425 => {
                    self.tg.ee_sub_161f10(g(0), g(1));
                }
                417 => {
                    for x in a {
                        if self.oids.len() < 8 {
                            self.oids.push(x as i32);
                        }
                    }
                }
                418 => {
                    for x in a {
                        if self.classes.len() < 8 {
                            self.classes.push(x as i32);
                        }
                    }
                }
                _ => {
                    let PropValue::Scalar(b) = &p.value else { continue };
                    let Some(&(_, off)) = ROWS_83.iter().find(|r| r.0 == p.field) else { continue };
                    let w = word(b);
                    match off {
                        0x2a8 => self.f_2a8 = w as i32,
                        0x2f4 => self.f_2f4 = w as i32,
                        0x1e8 => self.tg.hold = f32::from_bits(w),
                        0x1dc => self.tg.rate = f32::from_bits(w),
                        0x1ec => self.tg.no_snd = w,
                        0x1f0 => self.tg.anim_mode = w,
                        0x1f4 => self.tg.stay1 = w,
                        0x1e0 => self.tg.state = w as i32,
                        0x2f8 => self.f_2f8 = w,
                        _ => {}
                    }
                }
            }
        }
    }
}

impl ClassSet {
    /// K15 class 83: construct + init (slot 6) + feed; the object list resolved to handles
    pub(super) fn build_k15_83(&mut self, w: &World, h: &mut dyn ClassHost) {
        for (i, o) in w.objects.iter().enumerate() {
            if o.class != 83 {
                continue;
            }
            let src = o.clone_of.unwrap_or(i);
            let props = w.objects.get(src).and_then(crate::engine::game::fields::object_props);
            let mut c = C2cc950::default();
            c.ee_sub_148bb0(i, h);
            if let Some(p) = props {
                c.feed(p);
            }
            c.objs = c.oids.iter().map(|&oid| h.ee_sub_15e938(4, oid, handle(i))).collect();
            if c.f_2f8 != 0 {
                c.f_2f8 = h.ee_sub_15e938(0x2002, c.f_2f8 as i32, handle(i));
            }
            self.k15.push((i, Inst::K15d(Box::new(c))));
        }
    }
    /// K15 class-83 state at object index `obj`
    pub fn k15_83_get(&mut self, obj: usize) -> Option<&mut C2cc950> {
        self.k15.iter_mut().find(|x| x.0 == obj).and_then(|x| if let Inst::K15d(c) = &mut x.1 { Some(&mut **c) } else { None })
    }
    /// K15 class 83 load-post: slot 13, slot 14
    pub(super) fn post_load_k15_83(&mut self, h: &mut dyn ClassHost) {
        let t = self.g.t;
        for (i, x) in &mut self.k15 {
            if let Inst::K15d(c) = x {
                c.ee_sub_148c08(*i, h);
                c.ee_sub_148c38(*i, t, h);
            }
        }
    }
    /// K15 class 83 slot 74
    pub fn k15_83_hit(&mut self, obj: usize, hit: &guard::slots::GuardHit, h: &mut dyn ClassHost) -> Option<bool> {
        let t = self.g.t;
        self.k15_83_get(obj).map(|c| c.ee_sub_148f20(hit, t, h))
    }
    /// K15 class 83 per frame: slot 15 (host contacts), then the deferred FUN_00149010 calls
    pub(super) fn frame_k15_83(&mut self, h: &mut dyn ClassHost) {
        let t = self.g.t;
        for (i, x) in &mut self.k15 {
            if let Inst::K15d(c) = x {
                let contacts = if c.f_2a8 == 0 { h.so_contacts_57(*i) } else { Vec::new() };
                c.ee_sub_148c98(*i, &contacts, t, h);
            }
        }
        for (_, x) in &mut self.k15 {
            if let Inst::K15d(c) = x {
                if std::mem::take(&mut c.pending) {
                    c.ee_sub_149010(t, h);
                }
            }
        }
    }
}
