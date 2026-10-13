//! WP-K5: the ALO/SO core vtable slots shared by every class-48 descendant (C2d9978 -> C2d4260 -> C2caf88,
//! C2d4b78, ...). The DB rooted the 63 "overrides" of class 86 here; only slots 5/6/14/54/60 are class 86's own
//! (see c2caf88.rs). Object fields are named by offset; the +0x40 tagged word is modelled as the inline form.
use super::super::c2d9978::C2d9978;
use super::super::ClassHost;

/// FUN_00149b58 / FUN_00149ab8 member-function words the core queues (the data word, as K4's PMF ids)
pub const PMF_2A8270: u32 = 0x2a8270;
pub const PMF_2A8278: u32 = 0x2a8278;

/// ALO core state (the SO fields of C2d9978 + the fields the core slots touch).
#[derive(Clone, Debug, Default)]
pub struct Alo {
    /// +0x10 owner, +0x2d, +0x12c, +0x134, +0x13c, +0x140, +0x148 (K: C2d9978)
    pub so: C2d9978,
    /// +0x38 (float, LOD / sort key written by slot 41)
    pub f_38: f32,
    /// +0x40 flags word (inline form)
    pub f_40: u32,
    /// +0x130 (fade rate; +0x12c is the fade value)
    pub f_130: f32,
    /// +0x110..+0x12a: 26 per-kind bytes (FUN_001f7918)
    pub f_110: [u8; 26],
}

impl Alo {
    pub fn h(obj: usize) -> u32 {
        obj as u32 + 1
    }
    /// FUN_001f7918(obj, mask, v): +0x110[i] = v for every set bit i < 26 of `mask`
    pub fn ee_sub_1f7918(&mut self, mask: u32, v: u8) {
        for i in 0..0x1a {
            if (1u32 << i) & mask != 0 {
                self.f_110[i] = v;
            }
        }
    }
    /// slot 10: FUN_001f2c30
    /// `g_3e1a5c_2c` = DAT_003e1a5c != 0 && *(DAT_003e1a5c + 0x2c) != 0
    pub fn ee_sub_1f2c30(&mut self, obj: usize, g_3e1a5c_2c: bool, h: &mut dyn ClassHost) {
        h.base(0x1a5800, obj);
        h.base(0x22faa0, obj);
        h.base(0x1f2b20, obj);
        self.f_40 |= 0x14;
        if h.phys_e8(obj).is_some_and(|p| p.dc) {
            // position the sound source now, else FUN_00149a50(0x3dd490, ...) (args dropped by the decompiler)
            if g_3e1a5c_2c {
                let p = h.ee_sub_1a62e8(Alo::h(obj));
                h.ee_sub_1c0d70([p[0].to_bits(), p[1].to_bits(), p[2].to_bits(), 0], 1);
                h.ee_sub_1f37c8(obj);
            } else {
                h.ee_sub_149a50(obj, 0x1f37c8, 0);
            }
        }
        for c in h.alo_children(Alo::h(obj)) {
            h.vcall(c, 0x5c);
        }
    }
    /// slot 11: FUN_001f2d48
    pub fn ee_sub_1f2d48(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1a5848, obj);
        h.base(0x1f2ba8, obj);
        self.f_130 = 0.0;
        self.so.f_12c = 1.0;
        if h.phys_e8(obj).is_some_and(|p| p.dc) {
            h.ee_sub_149b58(3, obj, PMF_2A8270);
            h.ee_sub_1f37c8(obj);
        }
        for c in h.alo_children(Alo::h(obj)) {
            // children flagged +0x28 bit 56 stop the walk
            if h.obj_flags_28(c) >> 56 & 1 != 0 {
                break;
            }
            h.vcall(c, 0x64);
        }
    }
    /// slot 15: FUN_001f36e0 (update + alpha fade)
    pub fn ee_sub_1f36e0(&mut self, obj: usize, dt: f32, h: &mut dyn ClassHost) {
        h.ee_sub_1a5e28(obj);
        if h.phys_e8(obj).is_some_and(|p| p.dc) {
            let p = h.ee_sub_1a62e8(Alo::h(obj));
            h.ee_sub_1c0d70([p[0].to_bits(), p[1].to_bits(), p[2].to_bits(), 0], 1);
            h.ee_sub_1f37c8(obj);
        }
        if self.f_130 != 0.0 {
            let a = self.so.f_12c + dt * self.f_130;
            self.so.f_12c = a;
            if a <= 0.0 {
                self.f_130 = 0.0;
                self.so.f_12c = 0.0;
                h.vcall(Alo::h(obj), 0x4c);
            } else if 1.0 <= a {
                self.f_130 = 0.0;
                self.so.f_12c = 1.0;
            }
        }
    }
    /// slot 20: FUN_001f3b28 (+0x40 bit 3 on/off)
    pub fn ee_sub_1f3b28(&mut self, obj: usize, on: bool, h: &mut dyn ClassHost) {
        if !on {
            if self.f_40 & 8 != 0 {
                self.f_40 &= !8;
                h.ee_sub_1d6310(obj);
            }
        } else if self.f_40 & 8 == 0 {
            self.f_40 |= 8;
            if self.so.bit35() && h.ee_sub_1a61a0(Alo::h(obj)) {
                h.ee_sub_1d6378(obj);
            }
        }
    }
    /// slot 41: FUN_001f3118(obj, f)
    pub fn ee_sub_1f3118(&mut self, obj: usize, f: f32, h: &mut dyn ClassHost) {
        self.f_38 = f;
        if self.so.f_140 != -1 {
            h.ee_sub_1d6310(obj);
            h.ee_sub_1d6378(obj);
        }
        if self.so.owner.is_none() && h.obj_flags_28(Alo::h(obj)) >> 60 & 1 != 0 {
            h.ee_sub_147e88(obj);
        }
        for c in h.alo_children(Alo::h(obj)) {
            if h.obj_f32(c, 0x38) > f {
                h.vcall_f(c, 0x154, f);
            }
        }
    }
    /// slots 46 / 47: FUN_001f2fe0 / FUN_001f3078 (+0x40 |= v / &= ~v, propagated to the children)
    pub fn ee_sub_1f2fe0(&mut self, obj: usize, v: u32, h: &mut dyn ClassHost) {
        self.f_40 |= v;
        for c in h.alo_children(Alo::h(obj)) {
            h.vcall_arg(c, 0x17c, v as i32);
        }
    }
    pub fn ee_sub_1f3078(&mut self, obj: usize, v: u32, h: &mut dyn ClassHost) {
        self.f_40 &= !v;
        for c in h.alo_children(Alo::h(obj)) {
            h.vcall_arg(c, 0x184, v as i32);
        }
    }
    /// slot 57: FUN_001f4f10 (queue the contact pass when the +0xec contact block has entries)
    pub fn ee_sub_1f4f10(&mut self, obj: usize, h: &mut dyn ClassHost) {
        let cs = h.contacts_ec(obj);
        if cs.head_6c || (self.so.f_148 & 0x300_0000 != 0 && cs.list_64.iter().any(|c| c.rec != 0)) {
            h.ee_sub_149ab8(obj, PMF_2A8278, 0);
        }
    }
    /// slot 61: FUN_001f5040 (no physics, or both +0xe8 states (+0xa0, +0xc0) == 3)
    pub fn ee_sub_1f5040(&self, obj: usize, h: &mut dyn ClassHost) -> bool {
        match h.phys_e8(obj) {
            None => true,
            Some(p) => p.s_c0 == 3 && p.s_a0 == 3,
        }
    }
    /// slots 27 / 42 / 62: FUN returning 0
    pub fn ret0(&self) -> i32 {
        0
    }
}

/// FUN_001f7588(obj, rec): rec +0x44 != 0 (slot 74 of the core)
pub fn ee_sub_1f7588(t: &Touch) -> bool {
    t.w_44 != 0
}

/// +0xe8 physics block fields the core reads (None = +0xe8 null).
#[derive(Clone, Copy, Debug, Default)]
pub struct PhysE8 {
    /// *(e8 + 0xdc) != 0
    pub dc: bool,
    pub s_a0: i32,
    pub s_c0: i32,
}

/// One entry of the (+0xec) + 100 contact list: [0] record, [2] -> flag word, [3] other object.
#[derive(Clone, Copy, Debug, Default)]
pub struct Contact {
    pub rec: u32,
    pub flag: bool,
    pub other: u32,
}

/// +0xec contact block (None fields = empty).
#[derive(Clone, Debug, Default)]
pub struct Contacts {
    /// *(ec + 0x6c) != 0
    pub head_6c: bool,
    pub list_64: Vec<Contact>,
}

/// Touch record passed to slot 74: [0] other object, [3] word, +0x40 u64 flags, [0x11] word.
#[derive(Clone, Copy, Debug, Default)]
pub struct Touch {
    pub other: u32,
    pub w_c: u32,
    pub flags_40: u64,
    pub w_44: u32,
}
