//! C2cf378 (world class 132, parent C2c94e0): level-entry hub of a class-133 group. Load FUN_001624a8 is
//! ported in world (`B4Load::L1624a8`). State 1/2/3 (+0xc4) step through the +0xb8 / +0xbc sequencers
//! (FUN_002121e8 / FUN_00213b00) and end in FUN_00162fd0 (warp hand-over). Object handles in hooks are index + 1.
use super::k4::{f32_of, word};
use super::{ClassGlobals, ClassHost};
use crate::engine::world::props::{Prop, PropValue};

/// The class-133 (C2cf1e8) fields FUN_00163170 / FUN_00164030 read through +0x68 / +0xc0.
#[derive(Clone, Debug, Default)]
pub struct C133View {
    /// +0x54 state
    pub f_54: i32,
    /// +0xc4 object reference
    pub f_c4: u32,
    /// +0x94/+0x98 and +0xa0/+0xa4 resolved object lists
    pub f_98: Vec<u32>,
    pub f_a4: Vec<u32>,
    /// +0xd8 save-variable path (empty = null)
    pub f_d8: Vec<u16>,
}

#[derive(Clone, Debug)]
pub struct C2cf378 {
    /// +0x40 bits (0x102 while active)
    pub f_40: u32,
    /// +0x50 parent (handle)
    pub f_50: u32,
    /// +0x64 / +0x68 object references (fields 1243 / 1244; +0x68 = the class-133 group object)
    pub f_64: u32,
    pub f_68: u32,
    /// +0x6c / +0x70 (fields 1245 / 1246)
    pub f_6c: i32,
    pub f_70: i32,
    /// +0x74 / +0x78 (fields 1241 / 1242)
    pub f_74: u8,
    pub f_78: u8,
    /// +0x7c / +0xa4: save words resolved from the +0xd0 / +0xe0 paths, +0xa8 = FUN_001d7e80 of +0xe8
    pub f_7c: u32,
    pub f_a4: u32,
    pub f_a8: u32,
    /// +0x8c/+0x90, +0x94/+0x98: (count, table) pairs passed to FUN_001620b0 / FUN_001623d0 (method props)
    pub f_8c: i32,
    pub f_90: i32,
    pub f_94: i32,
    pub f_98: i32,
    /// +0x9c / +0xa0 (field 1257)
    pub f_9c: f32,
    pub f_a0: f32,
    /// +0xac oid of +0xb8, +0xb0 / +0xb4 sequence ids
    pub f_ac: i32,
    pub f_b0: i32,
    pub f_b4: i32,
    /// +0xb8 / +0xbc sequencer objects (FUN_0015e938 kind 0x405; +0xbc = oid 0x379)
    pub f_b8: u32,
    pub f_bc: u32,
    /// +0xc0 chosen class-133 object, +0xc4 state, +0xc8 state time
    pub f_c0: u32,
    pub f_c4: i32,
    pub f_c8: f32,
    /// +0xcc warp object (field 1259)
    pub f_cc: u32,
    /// +0xd0 / +0xd8 / +0xe0 / +0xe8 save-variable paths (fields 1262 / 1263 / 1260 / 1261)
    pub f_d0: Vec<u16>,
    pub f_d8: Vec<u16>,
    pub f_e0: Vec<u16>,
    pub f_e8: Vec<u16>,
    /// +0xf0 (field 1253), +0xf4 (field 1254)
    pub f_f0: i32,
    pub f_f4: i32,
}

impl Default for C2cf378 {
    fn default() -> Self {
        C2cf378 {
            f_40: 0,
            f_50: 0,
            f_64: 0,
            f_68: 0,
            f_6c: 0,
            f_70: 0,
            f_74: 0,
            f_78: 0,
            f_7c: 0,
            f_a4: 0,
            f_a8: 0,
            f_8c: 0,
            f_90: 0,
            f_94: 0,
            f_98: 0,
            f_9c: 0.0,
            f_a0: 0.0,
            f_ac: 0,
            f_b0: 0,
            f_b4: 0,
            f_b8: 0,
            f_bc: 0,
            f_c0: 0,
            f_c4: 0,
            f_c8: 0.0,
            f_cc: 0,
            f_d0: Vec::new(),
            f_d8: Vec::new(),
            f_e0: Vec::new(),
            f_e8: Vec::new(),
            f_f0: 0,
            f_f4: 0,
        }
    }
}

/// slot 5: FUN_00114250
pub fn ee_sub_114250() -> u32 {
    0x2bb538
}

/// kind 0x16 save-variable path prop -> u16 list
pub fn path_of(v: &[u8]) -> Vec<u16> {
    v.get(2..).unwrap_or(&[]).chunks_exact(2).map(|x| u16::from_le_bytes([x[0], x[1]])).collect()
}

/// FUN_00164030 (class 133): its +0x54 state >= 2, or the +0xd8 save word while +0x54 == -1
pub fn ee_sub_164030(c: &C133View, h: &mut dyn ClassHost) -> bool {
    let mut s = c.f_54;
    if s == -1 {
        if !c.f_d8.is_empty() {
            return h.ee_sub_2581c0(&c.f_d8) != 0;
        }
        s = c.f_54;
    }
    !(s < 2)
}

impl C2cf378 {
    /// Load fields 1241-1263 (written after slot 6). Object references resolved by FUN_0015e938.
    pub fn feed(&mut self, obj: usize, props: &[Prop], h: &mut dyn ClassHost) {
        let me = obj as u32 + 1;
        for p in props {
            let PropValue::Scalar(b) = &p.value else { continue };
            match p.field {
                1241 => self.f_74 = word(b) as u8,
                1242 => self.f_78 = word(b) as u8,
                1243 => self.f_64 = h.ee_sub_15e938(0, word(b) as i32, me),
                1244 => self.f_68 = h.ee_sub_15e938(0, word(b) as i32, me),
                // LOOK R39: 2-byte scalars are s16 (k4::word keeps only b[0]: job name ids 458..484 came out
                // 202..228; PS2 EP4 028 C2cf378 +0x6c = 484 'Operation: Trojan Tank' etc.)
                1245 => self.f_6c = if b.len() == 2 { i16::from_le_bytes([b[0], b[1]]) as i32 } else { word(b) as i32 },
                1246 => self.f_70 = word(b) as i32,
                1253 => self.f_f0 = word(b) as i32,
                1254 => self.f_f4 = word(b) as i32,
                1257 => {
                    self.f_9c = f32_of(b);
                    self.f_a0 = b.get(4..8).map(f32_of).unwrap_or(0.0);
                }
                1259 => self.f_cc = h.ee_sub_15e938(0, word(b) as i32, me),
                1260 => self.f_e0 = path_of(b),
                1261 => self.f_e8 = path_of(b),
                1262 => self.f_d0 = path_of(b),
                1263 => self.f_d8 = path_of(b),
                _ => {}
            }
        }
    }

    /// slot 6: FUN_00162448
    pub fn ee_sub_162448(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1a5270, obj);
        self.f_9c = 480.0;
        self.f_a0 = 720.0;
        self.f_b4 = -1;
        self.f_6c = -1;
        self.f_70 = -1;
        self.f_ac = -1;
        self.f_b0 = -1;
        // FUN_0011a050(obj + 0x80, 0x84): empty list (no state kept)
    }

    /// FUN_001633e8: DAT_003e0f9c == obj
    pub fn ee_sub_1633e8(obj: usize, g: &ClassGlobals) -> bool {
        g.g_3e0f9c == obj as i32 + 1
    }

    /// FUN_00162670
    pub fn ee_sub_162670(&self, h: &mut dyn ClassHost) -> bool {
        if self.f_50 != 0 {
            return false;
        }
        if self.f_f0 == 2 || self.f_f0 == 3 {
            false
        } else {
            h.ee_sub_20e040() == 0
        }
    }

    /// slot 14: FUN_00162780
    pub fn ee_sub_162780(&mut self, obj: usize, g: &ClassGlobals, h: &mut dyn ClassHost) {
        let me = obj as u32 + 1;
        h.base(0x1a5c48, obj);
        if !Self::ee_sub_1633e8(obj, g) && self.ee_sub_162670(h) {
            h.ee_sub_1620b0(me, 5, self.f_8c, self.f_90);
            h.ee_sub_1623d0(me, 5, self.f_94, self.f_98, 1);
        }
        if !self.f_d0.is_empty() {
            self.f_7c = h.ee_sub_2581c0(&self.f_d0);
        }
        if !self.f_e0.is_empty() {
            self.f_a4 = h.ee_sub_2581c0(&self.f_e0);
        }
        self.f_a8 = h.ee_sub_1d7e80(&self.f_e8);
        if self.f_ac != -1 {
            self.f_b8 = h.ee_sub_15e938(0x405, self.f_ac, 0);
        }
        self.f_bc = h.ee_sub_15e938(0x405, 0x379, 0);
    }

    /// slot 15: FUN_00163078
    pub fn ee_sub_163078(&mut self, obj: usize, g: &mut ClassGlobals, c133: &dyn Fn(u32) -> Option<C133View>, h: &mut dyn ClassHost) {
        h.ee_sub_1a5e28(obj);
        match self.f_c4 {
            2 => {
                if !h.ee_sub_2121e8(self.f_b8, self.f_b4) {
                    self.ee_sub_162fd0(h);
                    self.ee_sub_163170(obj, 0, g, c133, h);
                }
            }
            1 => {
                if !h.ee_sub_2121e8(self.f_b8, self.f_b0) {
                    self.ee_sub_162f60(obj, g, h);
                    if self.f_c0 == 0 && self.f_cc == 0 {
                        self.ee_sub_163170(obj, 2, g, c133, h);
                    } else {
                        self.ee_sub_163170(obj, 3, g, c133, h);
                    }
                }
            }
            3 => {
                if !h.ee_sub_2121e8(self.f_bc, 0x30f) {
                    self.ee_sub_162fd0(h);
                }
            }
            _ => {}
        }
    }

    /// FUN_00163170(obj, s): state change
    pub fn ee_sub_163170(&mut self, obj: usize, s: i32, g: &ClassGlobals, c133: &dyn Fn(u32) -> Option<C133View>, h: &mut dyn ClassHost) {
        if self.f_c4 == s {
            return;
        }
        if self.f_c4 == 0 {
            // first class-133 object of the +0x68 group's +0xa4 list with +0xc4 set
            self.f_c0 = 0;
            if let Some(grp) = c133(self.f_68) {
                for &e in &grp.f_a4 {
                    if c133(e).map(|x| x.f_c4 != 0).unwrap_or(false) {
                        self.f_c0 = e;
                        break;
                    }
                }
            }
            // every other object of its +0x98 list must pass FUN_00164030
            if let Some(sel) = (self.f_c0 != 0).then(|| c133(self.f_c0)).flatten() {
                for &e in &sel.f_98 {
                    if e != self.f_68 {
                        let ok = c133(e).map(|x| ee_sub_164030(&x, h)).unwrap_or(false);
                        if !ok {
                            self.f_c0 = 0;
                            break;
                        }
                    }
                }
            }
            h.ee_sub_1a6ca8(obj, 1);
            self.f_40 |= 0x102;
            h.ee_sub_20efc0(2);
        }
        self.f_c4 = s;
        self.f_c8 = g.t;
        match s {
            1 => {
                h.vcall_arg(self.f_b8, 0x18c, -1);
                h.ee_sub_213b00(self.f_b8, self.f_b0);
            }
            0 => {
                if self.f_bc != 0 {
                    h.vcall(self.f_bc, 0x194);
                }
                if self.f_b8 != 0 {
                    h.vcall(self.f_b8, 0x194);
                }
                h.ee_sub_1a6ca8(obj, 2);
                self.f_40 &= 0xfffffefd;
                h.ee_sub_20f030(2);
            }
            2 => h.ee_sub_213b00(self.f_b8, self.f_b4),
            3 => h.vcall_arg(self.f_bc, 0x18c, -1),
            _ => {}
        }
    }

    /// FUN_00162fd0: pending transition queue (0x2e1e88) first, else hand over to +0xc0 / the +0xcc warp
    pub fn ee_sub_162fd0(&mut self, h: &mut dyn ClassHost) {
        if !h.ee_sub_1a4e70() {
            if self.f_c0 == 0 {
                if self.f_cc != 0 {
                    h.obj_or_40(self.f_cc, 0x100);
                    h.ee_sub_2283d0(self.f_cc);
                }
            } else {
                h.ee_sub_164640(self.f_c0, 4);
            }
        } else {
            h.ee_sub_1a4e80();
        }
    }

    /// FUN_00162f60
    pub fn ee_sub_162f60(&mut self, obj: usize, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        if self.f_78 == 0 {
            h.ee_sub_164df0(self.f_68, 3);
        }
        h.ee_sub_1619a0(0x20, obj as u32 + 1);
        // FUN_00162908
        if Self::ee_sub_1633e8(obj, g) {
            self.ee_sub_162a60(obj, g, h);
        }
        if self.f_78 == 0 {
            h.ee_sub_1abd98(0);
        }
    }

    /// FUN_00162a60
    pub fn ee_sub_162a60(&mut self, obj: usize, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        let me = obj as u32 + 1;
        h.ee_sub_165dc0(0);
        let p = h.g_3e0fa4_7c();
        if p != 0 && p == me {
            h.ee_sub_165e10(0);
        }
        if h.ee_sub_210b70_same() {
            g.g_3e1118 = 0x189;
            g.g_3e111c = -1;
        }
        if self.ee_sub_162670(h) {
            h.ee_sub_1619a0(0x22, me);
            h.ee_sub_1620b0(me, 5, self.f_8c, self.f_90);
            h.ee_sub_1623d0(me, 5, self.f_94, self.f_98, self.f_f0);
            h.vcall_1c(obj, 0x14, 0);
        }
    }
}
