//! C2d4b78 (world class 98, parent C2d4260 / SO): staged switch. Load-time methods build a filter list
//! (+0x150/+0x154, class + enable pairs) and up to 32 stages (+0x160/+0x164, record -1 = pre-stage), each with
//! up to 64 actions; slot 25 / FUN_001baf18 steps the current stage (+0x15c) running the actions forward
//! (or their inverses backward). Contacts / touches of filtered classes latch the toucher (+0x16c/+0x170).
use super::super::ClassHost;
use super::alo::{Alo, Touch};
use super::c2caf88::prop_args;
use crate::engine::world::props::Prop;

/// FUN_001baed8 member-function word (DAT_002a7630)
pub const PMF_2A7630: u32 = 0x2a7630;
/// FUN_001baf18 targets: first stage / past the last stage
pub const STAGE_FIRST: i32 = 0x609;
pub const STAGE_LAST: i32 = 0x60a;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Act {
    /// +0 kind (0 vcall +0x44, 1 vcall +0x4c, 2..5 part ops, 6..0xb oid / word args, 0xc pair)
    pub kind: u32,
    /// +4 argument (oid / id); kind 0xc: the 8-byte pair it points at
    pub arg: u32,
    pub pair: [u32; 2],
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Stage {
    pub id: i32,
    pub f_4: i32,
    pub acts: Vec<Act>,
}

#[derive(Clone, Debug, Default)]
pub struct C2d4b78 {
    pub alo: Alo,
    /// +0x18 own oid (field 1042 default target)
    pub oid: i32,
    /// +0x150 / +0x154: (class, enable) pairs, max 8
    pub f_154: Vec<(i32, u32)>,
    /// +0x158
    pub f_158: i32,
    /// +0x15c current stage (-1 = pre-stage)
    pub f_15c: i32,
    /// record -1 (pre-stage)
    pub pre: Stage,
    /// +0x160 / +0x164 stages, max 32
    pub stages: Vec<Stage>,
    /// +0x168 (field 1046), +0x16c / +0x170 latched toucher and its word (fields 1028 / 1029)
    pub f_168: u32,
    pub f_16c: u32,
    pub f_170: u32,
}

/// slot 5: FUN_00114030
pub fn ee_sub_114030() -> u32 {
    0x2bb290
}

impl C2d4b78 {
    fn rec(&mut self, i: i32) -> Option<&mut Stage> {
        if i == -1 {
            Some(&mut self.pre)
        } else {
            self.stages.get_mut(i as usize)
        }
    }
    /// slot 6: FUN_001bab50 = FUN_001f2108 + FUN_001f7918(0x3ffffff, 8) + FUN_001f7918(0x80001, 0)
    pub fn ee_sub_1bab50(&mut self, obj: usize, owner_bit: Option<bool>, g_2c3b90: u32, h: &mut dyn ClassHost) {
        self.alo.so.ee_sub_1f2108(obj, owner_bit, g_2c3b90, h);
        self.alo.ee_sub_1f7918(0x3ff_ffff, 8);
        self.alo.ee_sub_1f7918(0x8_0001, 0);
    }
    /// slot 13: FUN_001baba0 (load). REPLACED: the arena allocations / pointer fix-ups are Vec storage; the
    /// FUN_001f31e0 property pass is `feed`; then +0x15c = 0 (and no stages -> +0x164 = 0).
    pub fn ee_sub_1baba0(&mut self, obj: usize, props: &[Prop], h: &mut dyn ClassHost) {
        self.feed(obj, props, h);
        self.f_15c = 0;
    }
    /// FUN_001f31e0 property pass: the class-98 fields (1025-1048).
    pub fn feed(&mut self, obj: usize, props: &[Prop], h: &mut dyn ClassHost) {
        for p in props {
            let a = prop_args(p);
            let a0 = a.first().copied().unwrap_or(0);
            let a1 = a.get(1).copied().unwrap_or(0);
            match p.field {
                1025 => self.ee_sub_1bb918(a0 as i32, a1), // thunk 0x10c820
                1028 => self.f_16c = a0,
                1029 => self.f_170 = a0,
                1030 => self.ee_sub_1bb950(a0 as i32), // 0x10c8c8
                1031 => self.ee_sub_1bb9b8(),          // 0x10c950
                1032 => self.ee_sub_1bba18(a0 as i32), // 0x10c9c8
                1033 => self.push(6, a0),              // setter FUN_001bba60
                1034 => self.push(7, a0),              // FUN_001bba90
                1035 => self.push(8, a0),              // 0x10ca50 -> FUN_001bbac0
                1036 => self.push(9, a0),              // 0x10cad8 -> FUN_001bbaf0
                1037 => self.push(10, a0),             // 0x10cb60 -> FUN_001bbb20
                1038 => self.push(0xb, a0),            // FUN_001bbb50
                1039 => self.push(0, a0),              // FUN_001bbb80
                1040 => self.ee_sub_1bbbb0(obj, a0, h), // 0x10cbe8
                1042 => {
                    // 0x10ce08 -> FUN_001bbc60: -1 = own oid
                    let v = if a0 as i32 == -1 { self.oid as u32 } else { a0 };
                    self.push(1, v);
                }
                1043 => self.ee_sub_1bbc98(obj, a0, h), // 0x10ce90
                1045 => {
                    // 0x10d0b0 -> FUN_001bbd48
                    if let Some(x) = self.ee_sub_1bbdb0(0xc) {
                        x.pair = [a0, a1];
                    }
                }
                1046 => self.f_168 = a0,
                1047 => self.ee_sub_1baed8(obj, 0, 0, h), // 0x10d158
                _ => {}
            }
        }
    }
    fn push(&mut self, kind: u32, arg: u32) {
        if let Some(x) = self.ee_sub_1bbdb0(kind) {
            x.arg = arg;
        }
    }
    /// FUN_001bb918: append a filter pair (max 8)
    pub fn ee_sub_1bb918(&mut self, cls: i32, en: u32) {
        if self.f_154.len() < 8 {
            self.f_154.push((cls, en));
        }
    }
    /// FUN_001bb950: add a stage and make it current (max 32)
    pub fn ee_sub_1bb950(&mut self, id: i32) {
        if self.stages.len() < 0x20 {
            self.f_15c = self.stages.len() as i32;
            self.stages.push(Stage { id, f_4: 1, acts: Vec::new() });
        }
    }
    /// FUN_001bb9b8: make the pre-stage (record -1) current
    pub fn ee_sub_1bb9b8(&mut self) {
        self.f_15c = -1;
        self.pre.id = -1;
        self.pre.f_4 = 1;
    }
    /// FUN_001bba18: current stage +4 = v (v != 0, not the pre-stage)
    pub fn ee_sub_1bba18(&mut self, v: i32) {
        if self.f_15c != -1 && v != 0 {
            let c = self.f_15c;
            if self.ee_sub_1bbe18() {
                if let Some(r) = self.rec(c) {
                    r.f_4 = v;
                }
            }
        }
    }
    /// FUN_001bbe18: current stage index valid (in range, or the pre-stage)
    pub fn ee_sub_1bbe18(&self) -> bool {
        let i = self.f_15c;
        (0 <= i && i < self.stages.len() as i32) || i == -1
    }
    /// FUN_001bbdb0: append an action of `kind` to the current stage (FUN_001bbe68; max 64)
    pub fn ee_sub_1bbdb0(&mut self, kind: u32) -> Option<&mut Act> {
        if !self.ee_sub_1bbe18() {
            return None;
        }
        let c = self.f_15c;
        let r = self.rec(c)?;
        if r.acts.len() >= 0x40 {
            return None;
        }
        r.acts.push(Act { kind, ..Default::default() });
        r.acts.last_mut()
    }
    /// FUN_001bbbb0 / FUN_001bbc98: no current stage -> run now (FUN_001f6978 / FUN_001f69f8), else action 2 / 3
    pub fn ee_sub_1bbbb0(&mut self, obj: usize, v: u32, h: &mut dyn ClassHost) {
        if !self.ee_sub_1bbe18() {
            h.ee_sub_1f6978(obj, v as i32);
        } else {
            self.push(2, v);
        }
    }
    pub fn ee_sub_1bbc98(&mut self, obj: usize, v: u32, h: &mut dyn ClassHost) {
        if !self.ee_sub_1bbe18() {
            h.ee_sub_1f69f8(obj, v as i32);
        } else {
            self.push(3, v);
        }
    }
    /// FUN_001baed8: latch (other, w) and queue DAT_002a7630
    pub fn ee_sub_1baed8(&mut self, obj: usize, other: u32, w: u32, h: &mut dyn ClassHost) {
        self.f_170 = w;
        self.f_16c = other;
        h.ee_sub_149ab8(obj, PMF_2A7630, 0);
    }
    /// FUN_001baf18(obj, target): step the current stage to `target` (0x609 first, 0x60a past last, else id)
    pub fn ee_sub_1baf18(&mut self, obj: usize, target: i32, h: &mut dyn ClassHost) {
        let n = self.stages.len() as i32;
        let mut cur = self.f_15c;
        let goal = match target {
            STAGE_FIRST => 0,
            STAGE_LAST => n,
            _ if target != -1 && n > 0 => self.stages.iter().position(|s| s.id == target).map(|k| k as i32).unwrap_or(cur),
            _ => cur,
        };
        if cur == goal {
            return;
        }
        let me = Alo::h(obj);
        let run = |a: &Act, back: bool, h: &mut dyn ClassHost| {
            // kinds 0, 1, 6, 7 resolve the oid first (FUN_0015e938 kind 1) and are skipped when it is unknown
            let mut tgt = 0;
            if (0xc3u32 >> (a.kind & 0x1f)) & 1 != 0 {
                tgt = h.ee_sub_15e938(1, a.arg as i32, me);
                if tgt == 0 {
                    return;
                }
            }
            let k = if back && a.kind <= 5 { a.kind ^ 1 } else { a.kind };
            match k {
                0 => h.vcall(tgt, 0x44),
                1 => h.vcall(tgt, 0x4c),
                2 => h.ee_sub_1f6978(obj, a.arg as i32),
                3 => h.ee_sub_1f69f8(obj, a.arg as i32),
                4 => h.ee_sub_1f6ac8(obj, a.arg as i32),
                5 => h.ee_sub_1f6b48(obj, a.arg as i32),
                _ => {}
            }
        };
        if cur < goal {
            while cur < goal {
                self.f_15c = cur + 1;
                let acts = self.rec(cur).map(|r| r.acts.clone()).unwrap_or_default();
                for a in &acts {
                    run(a, false, h);
                }
                cur = self.f_15c;
            }
        } else {
            while goal < cur {
                self.f_15c = cur - 1;
                let acts = self.rec(cur - 1).map(|r| r.acts.clone()).unwrap_or_default();
                for a in acts.iter().rev() {
                    run(a, true, h);
                }
                cur = self.f_15c;
            }
        }
        self.f_158 = 0;
    }
    /// slot 21: FUN_001bad80 = core slot 21 (FUN_001f3e00) + remove from DAT_003e1450
    pub fn ee_sub_1bad80(&mut self, obj: usize, l_3e1450: &mut Vec<u32>, h: &mut dyn ClassHost) {
        h.base(0x1f3e00, obj);
        l_3e1450.retain(|&x| x != Alo::h(obj));
    }
    /// slot 22: FUN_001badb8 = core slot 22 (FUN_001f3ea8) + append to DAT_003e1450
    pub fn ee_sub_1badb8(&mut self, obj: usize, l_3e1450: &mut Vec<u32>, h: &mut dyn ClassHost) {
        h.base(0x1f3ea8, obj);
        l_3e1450.push(Alo::h(obj));
    }
    /// slot 25: FUN_001bbea8 = FUN_001cb518 + reset to the first stage + FUN_001f5720/1f57c0(obj, 1) with physics
    pub fn ee_sub_1bbea8(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1cb518, obj);
        self.ee_sub_1baf18(obj, STAGE_FIRST, h);
        if h.phys_e8(obj).is_some() {
            h.ee_sub_1f5720(obj, 1);
            h.ee_sub_1f57c0(obj, 1);
        }
    }
    /// slot 57: FUN_001bb250 = core FUN_001f4f10 + contact latch. `peer` = filter list of another class-98
    /// object (handle); returns the FUN_001baed8(peer, obj, 0) calls to run on those peers.
    pub fn ee_sub_1bb250(&mut self, obj: usize, peer: &dyn Fn(u32) -> Option<Vec<(i32, u32)>>, h: &mut dyn ClassHost) -> Vec<u32> {
        let mut peers = Vec::new();
        self.alo.ee_sub_1f4f10(obj, h);
        if !self.ee_sub_1bbe18() {
            return peers;
        }
        let me = Alo::h(obj);
        let list = h.contacts_ec(obj).list_64;
        for (k, c) in list.iter().enumerate() {
            if !c.flag {
                continue;
            }
            let Some(_) = self.f_154.clone().iter().find(|e| e.1 != 0 && h.ee_sub_132b10(c.other, e.0) != 0) else { continue };
            if self.alo.so.f_148 & 0x300_0000 == 0 {
                self.ee_sub_1baed8(obj, c.other, 0, h);
            }
            if h.ee_sub_132b10(c.other, 0x62) != 0 {
                // the other object is a class-98 switch too: latch us there on its first enabled match
                if let Some(pl) = peer(c.other) {
                    if pl.iter().any(|e| e.1 != 0 && h.ee_sub_132b10(me, e.0) != 0) {
                        peers.push(c.other);
                    }
                }
            }
            h.vcall_1b4(me, c.other);
            h.vcall_1b4(c.other, me);
            h.ee_sub_1a7e10(c.rec);
            h.contact_clear(obj, k);
        }
        peers
    }
    /// slot 74: FUN_001badf0(obj, touch): 1 when the touch is taken (latching a filtered toucher)
    pub fn ee_sub_1badf0(&mut self, obj: usize, t: &Touch, h: &mut dyn ClassHost) -> i32 {
        if !self.ee_sub_1bbe18() || t.w_44 == 0 {
            return 0;
        }
        if t.flags_40 & 0xc_0000_0000 != 0x8_0000_0000 {
            return 1;
        }
        let me = Alo::h(obj);
        if !self.f_154.is_empty() && t.other != me {
            // note: the enable word is not checked here (unlike slot 57)
            if !self.f_154.clone().iter().any(|e| h.ee_sub_132b10(t.other, e.0) != 0) {
                return 0;
            }
        }
        self.ee_sub_1baed8(obj, t.other, t.w_c, h);
        1
    }
}
