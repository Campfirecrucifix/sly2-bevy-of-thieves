//! WP-K17: class 55 C2cb230 = Bentley, continued from classes/k16_55.rs (clusters of tools/engineport/
//! class_port.md "K16 class-55 split").
//! Layout: the JT-layout words that no named `C2cb230` field models live in `C2cb230::m` (`Mem`, the 0x1880-byte
//! object as words, addressed by the original byte offset). Named fields (state +0x13d0, +0x16b0, +0x2dc,
//! +0x4c8/+0x4cc, +0xf54, +0x1048, +0x1300, dmg +0xdf0, binoc +0xf00, the ALO core) are authoritative; `Mem`
//! never holds a copy of them.
//! Cluster 55f (slot 6 init), 1:1: FUN_00134648 -> FUN_0013ff70 (class-52 base) -> FUN_00203948 (PO) ->
//! FUN_001c3d70 -> FUN_001f2108 (ALO, `C2d9978::ee_sub_1f2108`), component inits FUN_00195950 (+0x2c0 JMP),
//! FUN_00191090 (+0x4c0), FUN_0019f3e8 (+0x9e0), FUN_001a0b18 (+0xdf0, `player::p7_damage::Dmg::default`),
//! FUN_0019cbf8 (+0xf00, `player::p15_binoc::Binoc::default`), FUN_0021f2e0 (+0x11b0), FUN_0019de10 (+0xf50),
//! FUN_00144638 (+0xfd0), FUN_0020b840, FUN_001cc158, FUN_001eec60, FUN_00204d48, FUN_00204660 (rows =
//! camera::math::FUN_0011b500), FUN_001f7918 (`Alo::ee_sub_1f7918`). DAT_002dd5bc = ClassGlobals::g_2dd5bc.
//! Cluster 55e (slots 82 / 83), 1:1: FUN_001442d0 / FUN_00144338 (+0x12f4 nesting count), FUN_00205478 /
//! FUN_00205588 (+0x2a0 six per-kind counters -> state-machine link records of the DAT_002c3c40 oids),
//! FUN_00143ee0 (anim / state-machine pose by oid), FUN_00143ff8, FUN_00144038, FUN_001fd0c0, FUN_001f03c8,
//! FUN_001f0198, FUN_001f1448. Hooks: FUN_001f01b8 / FUN_001f0380 / FUN_001f0598 link records (object +0x68
//! list), FUN_0019cd38 SetBinocState of a non-JT (player::p15_binoc runs it on a `Jt`).
#![allow(non_snake_case)]
use super::k16_55::C2cb230;
use super::*;

/// The object as words (byte offset -> word), for the JT-layout fields without a named field.
#[derive(Clone, Debug)]
pub struct Mem(pub Vec<u32>);

/// class-55 object size
pub const SIZE: usize = 0x1880;

impl Default for Mem {
    fn default() -> Self {
        Mem(vec![0; SIZE / 4])
    }
}

impl Mem {
    pub fn u(&self, off: u32) -> u32 {
        self.0[off as usize / 4]
    }
    pub fn i(&self, off: u32) -> i32 {
        self.u(off) as i32
    }
    pub fn f(&self, off: u32) -> f32 {
        f32::from_bits(self.u(off))
    }
    pub fn set_u(&mut self, off: u32, v: u32) {
        self.0[off as usize / 4] = v;
    }
    pub fn set_i(&mut self, off: u32, v: i32) {
        self.set_u(off, v as u32);
    }
    pub fn set_f(&mut self, off: u32, v: f32) {
        self.set_u(off, v.to_bits());
    }
    pub fn v3(&self, off: u32) -> [f32; 3] {
        [self.f(off), self.f(off + 4), self.f(off + 8)]
    }
    pub fn set_v3(&mut self, off: u32, v: [f32; 3]) {
        for (k, x) in v.iter().enumerate() {
            self.set_f(off + 4 * k as u32, *x);
        }
    }
}

fn handle(obj: usize) -> u32 {
    obj as u32 + 1
}

/// DAT_002bf740: JMP +0xf0 defaults (FUN_00195950)
pub const D_2BF740: [f32; 4] = [37.0, 0.0, 25.0, 0.0];
/// DAT_002c3cb8: PO move params +0x1f4 / +0x1f8 / +0x1fc (FUN_00204d48)
pub const D_2C3CB8: [f32; 3] = [25.0, 0.0, 0.25];
/// DAT_002c3c40: state-machine oid per counter kind (FUN_001fd0c0)
pub const D_2C3C40: [i32; 6] = [0x60c, 0x60d, 0x60e, -1, 0x60f, 0x610];

impl C2cb230 {
    /// FUN_0020b840(p, owner, k)
    pub(super) fn ee_sub_20b840(&mut self, p: u32, owner: u32, k: u32) {
        self.m.set_u(p, owner);
        self.m.set_f(p + 0x94, f32::from_bits(0x3f19999a));
        self.m.set_u(p + 0x90, k);
        self.m.set_f(p + 0x98, 1.0);
    }
    /// FUN_001cc158(p)
    fn ee_sub_1cc158(&mut self, p: u32) {
        self.m.set_i(p + 0x90, -1);
        self.m.set_i(p + 0xa4, -1);
    }
    /// FUN_00191090(self + 0x4c0, owner): attach block
    fn ee_sub_191090(&mut self, owner: u32) {
        let p = 0x4c0;
        self.m.set_u(p, owner);
        self.m.set_i(p + 0x13f * 4, -1);
        self.att_4c8 = -1;
        self.att_4cc = -1;
        self.ee_sub_20b840(p + 0x180, owner, 0x6f);
        self.ee_sub_20b840(p + 0x240, owner, 0x6f);
        self.ee_sub_1cc158(p + 0x440);
    }
    /// FUN_00195950(self + 0x2c0, owner): JMP block
    fn ee_sub_195950(&mut self, owner: u32) {
        let p = 0x2c0;
        self.m.set_u(p, owner);
        self.f_2dc = -1;
        self.m.set_i(p + 4, -1);
        for (k, v) in D_2BF740.iter().enumerate() {
            self.m.set_f(p + 0xf0 + 4 * k as u32, *v);
        }
        self.m.set_f(p + 0x164, 1.0);
        self.m.set_f(p + 0x108, -60.0);
        self.m.set_f(p + 0x10c, -50.0);
        self.m.set_f(p + 0x114, 50.0);
        self.m.set_f(p + 0x11c, 75.0);
        self.m.set_f(p + 0x118, -150.0);
        self.m.set_f(p + 0x110, 75.0);
    }
    /// FUN_0021f2e0(self + 0x11b0, owner)
    fn ee_sub_21f2e0(&mut self, owner: u32) {
        self.ee_sub_21f2e0_at(0x11b0, owner);
    }
    /// FUN_0021f2e0(self + p, owner) (K18: Murray's block is at +0x1910)
    pub(super) fn ee_sub_21f2e0_at(&mut self, p: u32, owner: u32) {
        self.m.set_f(p + 0x1c, 500.0);
        self.m.set_f(p + 0x20, 1500.0);
        self.m.set_f(p + 0x24, 1.5);
        self.m.set_u(p, owner);
        self.m.set_i(p + 4, -1);
        self.m.set_f(p + 0x18, 1.0);
        self.m.set_f(p + 0x14, 1.0);
        self.m.set_f(p + 0x28, 1000.0);
        self.m.set_f(p + 0x30, 0.0);
        self.m.set_f(p + 0x38, 25.0);
        self.m.set_f(p + 0x34, 0.0);
        self.m.set_f(p + 0xc0, 1000.0);
        self.m.set_f(p + 0xc8, 0.0);
        self.m.set_f(p + 0xc4, 0.0);
        self.m.set_f(p + 0xd0, 42.0);
        self.m.set_f(p + 0xd8, -50.0);
        self.m.set_f(p + 0xd4, -73.0);
    }
    /// FUN_0019de10(self + 0xf50, owner): path-follow block
    fn ee_sub_19de10(&mut self, owner: u32) {
        let p = 0xf50;
        self.m.set_u(p, owner);
        self.f_f54 = -1;
        self.m.set_f(p + 0x30, 45.0);
        self.m.set_f(p + 0x38, 40.0);
        self.m.set_f(p + 0x34, 0.0);
        self.m.set_f(p + 0x40, 45.0);
        self.m.set_f(p + 0x48, -20.0);
        self.m.set_f(p + 0x44, 0.0);
    }
    /// FUN_00204d48(k, self): PO move params scaled by k
    pub fn ee_sub_204d48(&mut self, k: f32) {
        self.m.set_f(0x1f4, D_2C3CB8[0] * k);
        self.m.set_f(0x1f8, D_2C3CB8[1] * k);
        self.m.set_f(0x1f0, k);
        self.m.set_f(0x1fc, D_2C3CB8[2] * (1.0 / k.sqrt()));
    }
    /// FUN_00204660(yaw, self): +0x1a8 yaw, +0x1b0 rows = rotation about DAT_002a2dc0
    pub fn ee_sub_204660(&mut self, yaw: f32) {
        self.m.set_f(0x1a8, yaw);
        let r = crate::engine::camera::math::FUN_0011b500(yaw, super::k16_55::UP_2A2DC0);
        for (k, row) in r.iter().take(3).enumerate() {
            for (j, x) in row.iter().enumerate() {
                self.m.set_f(0x1b0 + 0x10 * k as u32 + 4 * j as u32, *x);
            }
        }
    }
    /// FUN_001c3d70(self): ALO init + player-record defaults (+0x15c alive, +0x168 self, +0x40 |= 2)
    fn ee_sub_1c3d70(&mut self, obj: usize, owner_bit: Option<bool>, g_2c3b90: u32, h: &mut dyn ClassHost) {
        self.alo.so.ee_sub_1f2108(obj, owner_bit, g_2c3b90, h);
        self.m.set_i(0x15c, 1);
        self.m.set_u(0x168, handle(obj));
        // +0x40 tagged word, inline form (bit 0 clear)
        self.alo.f_40 |= 2;
    }
    /// FUN_00203948(self): PO init
    fn ee_sub_203948(&mut self, obj: usize, owner_bit: Option<bool>, g_2c3b90: u32, h: &mut dyn ClassHost) {
        self.ee_sub_1c3d70(obj, owner_bit, g_2c3b90, h);
        self.m.set_i(0x264, -1);
        self.m.set_f(0x260, 1.0);
        self.m.set_f(0x200, 1.0);
        self.ee_sub_204d48(1.0);
        self.ee_sub_204660(0.0);
        self.m.set_f(0x220, 30.0);
        self.m.set_f(0x214, -50.0);
        // FUN_001eec60(self + 0x26c, self)
        self.m.set_u(0x26c, handle(obj));
    }
    /// FUN_0013ff70(self): class-52 player-character base init
    pub fn ee_sub_13ff70(&mut self, obj: usize, owner_bit: Option<bool>, g_2c3b90: u32, h: &mut dyn ClassHost) {
        self.ee_sub_203948(obj, owner_bit, g_2c3b90, h);
        let o = handle(obj);
        self.ee_sub_195950(o);
        self.ee_sub_191090(o);
        // FUN_0019f3e8(self + 0x9e0, self)
        self.m.set_u(0x9e0, o);
        // FUN_001a0b18(self + 0xdf0, self)
        self.dmg = crate::engine::player::p7_damage::Dmg::default();
        // FUN_0019cbf8(self + 0xf00, self)
        self.binoc = crate::engine::player::p15_binoc::Binoc::default();
        self.ee_sub_21f2e0(o);
        self.ee_sub_19de10(o);
        // FUN_00144638(self + 0xfd0, self)
        self.m.set_u(0xfd0, o);
        self.m.set_f(0x103c, 20.0);
        self.f_1048 = 125.0;
        self.m.set_f(0x10d4, 1.0);
        self.m.set_f(0x1138, 1.5);
        self.m.set_f(0x113c, 2.5);
        self.m.set_f(0x1140, 3.0);
        self.m.set_f(0x1144, 5.0);
        self.m.set_i(0x12f0, 1);
        self.m.set_f(0x1020, 100.0);
        self.f_1300 = 100;
        // WIRE-SLY: REPLACED +0x12fc "mpCur" pool pointer -> the pool value (as the JT `Gadgets::mp`, full = 100)
        self.ctr_12fc = Some(100);
        self.m.set_f(0x1304, -60.0);
        self.m.set_i(0x10c8, -1);
        self.m.set_i(0x10f0, 1);
        self.m.set_f(0x10d8, 1.0);
        self.m.set_i(0x10ec, -1);
        self.m.set_i(0x10e4, -1);
        self.m.set_i(0x1318, -1);
        self.alo.ee_sub_1f7918(0x3ffffff, 0);
        self.alo.ee_sub_1f7918(0x10ff00, 8);
    }
    /// slot 6 FUN_00134648(self): Bentley init
    pub fn ee_sub_134648(&mut self, obj: usize, owner_bit: Option<bool>, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        self.ee_sub_13ff70(obj, owner_bit, g.g_2c3b90, h);
        self.m.set_i(0x1040, 0);
        self.m.set_i(0x1800, -1);
        self.m.set_f(0x103c, 45.0);
        self.f_1048 = 125.0;
        self.state = -1;
        self.f_16b0 = -1;
        self.m.set_i(0x16bc, -1);
        let o = handle(obj);
        self.ee_sub_20b840(0x1320, o, 0x73);
        self.ee_sub_20b840(0x1600, o, 0x73);
        self.m.set_f(0x3d8, -70.0);
        self.m.set_f(0x1098, -100.0);
        self.m.set_f(0x109c, 100.0);
        self.m.set_f(0x10a0, 50.0);
        self.m.set_f(0x10a4, -10.0);
        self.m.set_f(0x10a8, 250.0);
        self.m.set_f(0x10b4, 4500.0);
        self.m.set_f(0x10b0, 1.0);
        self.m.set_f(0x10ac, 0.0);
        self.m.set_i(0x1000, 2);
        self.m.set_i(0x1830, 7);
        self.m.set_i(0x183c, 9);
        self.m.set_i(0x1848, 0xb);
        self.m.set_i(0x1854, 0xd);
        self.m.set_i(0x1860, 4);
        self.m.set_i(0x1870, 5);
        self.m.set_i(0x1824, 5);
        g.g_2dd5bc = o;
    }
}

// ---- 55e: slots 82 / 83

/// FUN_001fd0c0(0x3dd040, kind): state-machine oid of counter `kind` (-1 none)
pub fn ee_sub_1fd0c0(kind: u32) -> i32 {
    D_2C3C40.get(kind as usize).copied().unwrap_or(-1)
}

/// FUN_001f1448(sm, obj, a, b, &slot): release the slot's link record, then link `sm` to `obj` into it
pub fn ee_sub_1f1448(sm: u32, obj: u32, a: i32, b: u32, slot: &mut u32, h: &mut dyn ClassHost) {
    if *slot != 0 {
        h.ee_sub_1f0598_link(*slot);
        h.vcall(*slot, 0x14);
        *slot = 0;
    }
    if sm != 0 {
        *slot = h.ee_sub_1f0380_link(sm, obj, a, b);
        h.vcall(*slot, 0xc);
    }
}

/// FUN_00205478 on the +0x2a0..+0x2b4 counters `n` of actor handle `o` (shared by all players; ANIM-ALL R34:
/// split out so Sly, whose JT is not a class instance, keeps his counters in `ClassSet::jt_2a0`)
pub fn ee_sub_205478_cnt(n: &mut [i32; 6], o: u32, mask: u32, h: &mut dyn ClassHost) {
    for u in 0..6u32 {
        if mask & (1 << u) == 0 {
            continue;
        }
        let c = &mut n[u as usize];
        *c += 1;
        if *c < 2 {
            let sm = h.ee_sub_15e938(4, ee_sub_1fd0c0(u), o);
            if sm != 0 && h.ee_sub_1f03c8(sm, o) == 0 {
                // FUN_001f0198 -> FUN_001f01b8(sm, self, -1, 9, &out)
                h.ee_sub_1f01b8(sm, o, -1, 9);
            }
        }
    }
}

/// FUN_00205588 on the counters `n` of actor handle `o`
pub fn ee_sub_205588_cnt(n: &mut [i32; 6], o: u32, mask: u32, h: &mut dyn ClassHost) {
    for u in 0..6u32 {
        let c = &mut n[u as usize];
        if mask & (1 << u) == 0 || *c == 0 {
            continue;
        }
        *c -= 1;
        if *c < 1 {
            let sm = h.ee_sub_15e938(4, ee_sub_1fd0c0(u), o);
            if sm != 0 {
                let rec = h.ee_sub_1f03c8(sm, o);
                if rec != 0 {
                    h.ee_sub_1f0598_link(rec);
                }
            }
        }
    }
}

impl C2cb230 {
    /// FUN_00205478(self, mask): first enable of each counter kind links its state machine to self
    pub fn ee_sub_205478(&mut self, obj: usize, mask: u32, h: &mut dyn ClassHost) {
        let mut n = self.cnt_2a0();
        ee_sub_205478_cnt(&mut n, handle(obj), mask, h);
        self.set_cnt_2a0(n);
    }
    /// FUN_00205588(self, mask): last disable of each counter kind removes the link record
    pub fn ee_sub_205588(&mut self, obj: usize, mask: u32, h: &mut dyn ClassHost) {
        let mut n = self.cnt_2a0();
        ee_sub_205588_cnt(&mut n, handle(obj), mask, h);
        self.set_cnt_2a0(n);
    }
    fn cnt_2a0(&self) -> [i32; 6] {
        std::array::from_fn(|u| self.m.i(0x2a0 + 4 * u as u32))
    }
    fn set_cnt_2a0(&mut self, n: [i32; 6]) {
        for (u, v) in n.into_iter().enumerate() {
            self.m.set_i(0x2a0 + 4 * u as u32, v);
        }
    }
    /// FUN_00144038: slot 88 index is 5 or 6 (states 10 / 11)
    pub fn ee_sub_144038(&self) -> bool {
        (self.ee_sub_135d08().wrapping_sub(5) as u32) < 2
    }
    /// FUN_00143ff8: back to slot-87 index 0 from states 10 / 11
    pub fn ee_sub_143ff8(&self, obj: usize, h: &mut dyn ClassHost) {
        if self.ee_sub_144038() {
            self.ee_sub_135c20(obj, 0, h);
        }
    }
    /// FUN_00143ee0(self, oid, flags, index): pose by oid: an anim (is-a 0x6d) plays into +0x1030, a state
    /// machine (is-a 0x6f) links into +0x1034; both enter slot-87 index `index`. oid -1 = FUN_00143ff8.
    pub fn ee_sub_143ee0(&mut self, obj: usize, oid: i32, flags: u32, index: u32, h: &mut dyn ClassHost) {
        if oid == -1 {
            self.ee_sub_143ff8(obj, h);
            return;
        }
        let o = h.ee_sub_15e938(1, oid, handle(obj));
        if o == 0 {
            return;
        }
        if h.ee_sub_132b10(o, 0x6d) == 0 {
            if h.ee_sub_132b10(o, 0x6f) != 0 {
                self.ee_sub_135c20(obj, index, h);
                let mut s = self.m.u(0x1030);
                super::shared::anim::ee_sub_130658(&mut s, h);
                self.m.set_u(0x1030, s);
                let mut s = self.m.u(0x1034);
                ee_sub_1f1448(o, handle(obj), -1, flags, &mut s, h);
                self.m.set_u(0x1034, s);
            }
        } else {
            self.ee_sub_135c20(obj, index, h);
            let mut s = self.m.u(0x1034);
            h.ee_sub_1f15b8(&mut s);
            self.m.set_u(0x1034, s);
            let mut s = self.m.u(0x1030);
            super::shared::anim::ee_sub_1305b8(0, 1.0, o, handle(obj), flags, &mut s, h);
            self.m.set_u(0x1030, s);
        }
    }
    /// slot 82 FUN_001442d0(self, mask, nest)
    pub fn ee_sub_1442d0(&mut self, obj: usize, mask: u32, nest: bool, h: &mut dyn ClassHost) {
        self.ee_sub_205478(obj, mask, h);
        if nest {
            let n = self.m.i(0x12f4) + 1;
            self.m.set_i(0x12f4, n);
            if n == 1 {
                self.ee_sub_143ee0(obj, 0x60b, 9, 5, h);
                // FUN_0019cd38(self + 0xf00, -1)
                h.ee_sub_19cd38_po(obj, -1);
            }
        }
    }
    /// slot 83 FUN_00144338(self, mask, nest)
    pub fn ee_sub_144338(&mut self, obj: usize, mask: u32, nest: bool, h: &mut dyn ClassHost) {
        self.ee_sub_205588(obj, mask, h);
        if nest {
            let n = self.m.i(0x12f4) - 1;
            self.m.set_i(0x12f4, n);
            if n == 0 {
                self.ee_sub_143ff8(obj, h);
            }
        }
    }
}

// ---- 55d: slot 18 draw

/// DAT_002bd268 / DAT_002bd26c: draw fade targets while the attach kind (+0x4c8) is 2
pub const R_2BD268: f32 = 0.15;
pub const R_2BD26C: f32 = 0.0;
/// DAT_002bd270: FUN_0011d018 approach profile {vmax, v0, accel} of the draw fades
pub const P_2BD270: [f32; 3] = [12.0, 2.0, 0.1];

impl C2cb230 {
    /// slot 18 FUN_001425b0 (draw): +0x10d4 / +0x10d8 fade towards 0.15 / 0 while attached as kind 2 (else 1),
    /// then the ALO draw FUN_00168638(+0x12c * +0x10d4, +0x12c * +0x10d8, self + 0x80, 0, +0x134).
    /// REPLACED: FUN_00168638 and its DMA / VU1 packet callees (FUN_001681f8, FUN_001684d0, FUN_00167da0,
    /// FUN_001cd998, FUN_001eda20) = the renderer hook `ClassHost::ee_sub_168638_2`. The DAT_002e1e40 branch
    /// (+0x19a0 / +0x19a4 alpha copies) is dead for class 55: those offsets lie beyond the 0x1880-byte object,
    /// DAT_002e1e40 is the JT.
    pub fn ee_sub_1425b0(&mut self, obj: usize, g: &ClassGlobals, h: &mut dyn ClassHost) {
        if self.alo.so.f_2d & g.g_2ddf54 == 0 {
            return;
        }
        h.ee_sub_1f3c10(obj);
        if self.binoc.state == 1 && self.binoc.mode != 2 {
            return;
        }
        let (a, b) = if self.att_4c8 == 2 { (R_2BD268, R_2BD26C) } else { (1.0, 1.0) };
        use crate::engine::physics::dynamics::FUN_0011d018;
        let x = FUN_0011d018(self.m.f(0x10d4), a, g.dt, P_2BD270, None);
        self.m.set_f(0x10d4, x);
        let y = FUN_0011d018(self.m.f(0x10d8), b, g.dt, P_2BD270, None);
        self.m.set_f(0x10d8, y);
        let f = self.alo.so.f_12c;
        h.ee_sub_168638_2(obj, f * x, f * y, self.alo.so.f_134);
    }
}

// ---- 55c: small slots (own 89 / 49, base 57 / 58 / 67 / 81 / 28 / 60)

/// DAT_002bcca0..DAT_002bccb4: slot-49 camera height window {lo, hi} over the ground z: default / +0x434 == 0 /
/// +0x434 == 0 && JMP +0x2dc == 0x16
pub const CAM_Z_2BCCA0: [[f32; 2]; 3] = [[-125.0, 200.0], [-275.0, 150.0], [-50.0, 85.0]];
/// DAT_002bcc98 / DAT_002bcc9c: clamp of the smoothed camera height (DAT_002de1a0 == self)
pub const CAM_Z_2BCC98: [f32; 2] = [-300.0, 200.0];
/// DAT_002bccb8 / DAT_002bccc8: FUN_0011d018 profiles of the smoothed height (+0x434 == 0 / != 0)
pub const P_2BCCB8: [f32; 3] = [4000.0, 0.0, 4.0];
pub const P_2BCCC8: [f32; 3] = [4000.0, 0.0, 0.3];
/// 0x2bccd8: FUN_0011f088 profile of the +0x1070 offset decay
pub const P_2BCCD8: [f32; 3] = [1500.0, 0.0, 0.5];
/// DAT_002bccf0..f8: speed-scale polynomial, clamped to DAT_002bbb60 / DAT_002bbb64
pub const SPD_2BCCF0: [f32; 3] = [1.5, -0.5, 0.0];
pub const CLAMP_2BBB60: [f32; 2] = [0.0, 1.0];
/// 0x3c23d70a: slot-67 push scale against the player characters
pub const PUSH_SCALE_67: f32 = 0.01;

/// camera target record filled by slot 49 (FUN_001f58f0 layout, 0x40 bytes as words: +0 flags, +0x10 position,
/// +0x20, +0x30 yaw, +0x34 speed (100), +0x38 time)
pub type CamRec = [u32; 16];

/// FUN_00134310(a, b) = max
fn ee_sub_134310(a: i64, b: i64) -> i64 {
    a.max(b)
}

impl C2cb230 {
    /// FUN_00144068: +0x270 > 0 || +0x1084 != 0
    pub fn ee_sub_144068(&self) -> bool {
        self.m.i(0x270) > 0 || self.m.i(0x1084) != 0 || self.k22_link != 0
    }
    /// FUN_0019c870(self + 0x2c0, &flag): JMP state ASEG (flag 0)
    pub fn ee_sub_19c870(&self) -> u32 {
        let s = self.f_2dc;
        if s < 0xb {
            self.m.u(0x380)
        } else if s < 0xe {
            // *(owner + 0x520): the owner of the embedded JMP block is this object
            self.m.u(0x520)
        } else if s == 0x11 {
            self.m.u(0x384)
        } else {
            self.m.u(0x380)
        }
    }
    /// FUN_001944c8(self + 0x4c0, &flag): attach ASEG by kind +0x4c8 / sub +0x4cc (the player/ version
    /// `p5_attach::FUN_001944c8` reads a `Jt`; same control flow over this object's words)
    pub fn ee_sub_1944c8(&self) -> (u32, u32) {
        match self.att_4c8 {
            1 => (self.m.u(0x518), 0),
            0 => (if self.att_4cc == 5 { self.m.u(0x520) } else { self.m.u(0x514) }, 0),
            8 if self.att_4cc < 0xb && self.att_4cc > 8 => (self.m.u(0x524), 1),
            _ => (0, 0),
        }
    }
    /// FUN_00203d00(self, aseg, flag): +0x25c = aseg; flag -> rebase the bone rows of +0x258 onto +0x25c
    /// (REPLACED: bone copy + FUN_002333b4 + FUN_001f4bf8 = `ClassHost::ee_sub_203d00_bones`)
    pub fn ee_sub_203d00(&mut self, obj: usize, aseg: u32, flag: u32, h: &mut dyn ClassHost) {
        self.m.set_u(0x25c, aseg);
        if flag != 0 {
            h.ee_sub_203d00_bones(obj);
        }
    }
    /// slot 89 FUN_00135d80: base ASEG by state (2 JMP, 3 attach, 4 damage), default +0x268
    pub fn ee_sub_135d80(&mut self, obj: usize, h: &mut dyn ClassHost) {
        let (mut a, f) = match self.state {
            3 => self.ee_sub_1944c8(),
            2 => (self.ee_sub_19c870(), 0),
            4 => (self.dmg.FUN_001a23f8().map_or(0, |x| x as u32), 0),
            _ => (0, 0),
        };
        if a == 0 {
            a = self.m.u(0x268);
        }
        self.ee_sub_203d00(obj, a, f, h);
    }
    /// FUN_00141640(self, rec): class-52 camera target (height window over the ground, +0x1070 offset decay,
    /// component flags, speed scale)
    pub fn ee_sub_141640(&mut self, obj: usize, rec: &mut CamRec, g: &ClassGlobals, h: &mut dyn ClassHost) {
        use crate::engine::physics::dynamics::FUN_0011d018;
        h.ee_sub_1f58f0(obj, rec);
        if self.m.i(0x1044) == 0 {
            // FUN_001a62e8 position, upper word = z
            let z = h.ee_sub_1a62e8(handle(obj))[2];
            let p = h.vcall_30c(obj);
            let w = if self.m.i(0x434) != 0 {
                CAM_Z_2BCCA0[0]
            } else if self.f_2dc == 0x16 {
                CAM_Z_2BCCA0[2]
            } else {
                CAM_Z_2BCCA0[1]
            };
            let mut v = z + w[0];
            if v <= p[2] {
                v = p[2];
                if z + w[1] < p[2] {
                    v = z + w[1];
                }
            }
            self.f_104c = v;
            let out = if g.g_2de240 == 0 && h.g_2de1a0() == handle(obj) {
                let prm = if self.m.i(0x434) == 0 { P_2BCCB8 } else { P_2BCCC8 };
                let s = FUN_0011d018(self.m.f(0x1050), self.f_104c, g.dt, prm, None);
                let lo = z + CAM_Z_2BCC98[0];
                if lo <= s {
                    let hi = z + CAM_Z_2BCC98[1];
                    if s <= hi { s } else { hi }
                } else {
                    lo
                }
            } else {
                v
            };
            rec[6] = out.to_bits();
            rec[4] = p[0].to_bits();
            rec[5] = p[1].to_bits();
        } else {
            rec[4] = self.m.u(0x1060);
            rec[5] = self.m.u(0x1064);
            rec[6] = self.m.u(0x1068);
            rec[7] = self.m.u(0x106c);
            self.f_104c = self.m.f(0x1068);
        }
        let mut vel = [0.0; 3];
        let o = crate::engine::player::p5_attach::FUN_0011f088(g.dt, self.m.v3(0x1070), [0.0; 3], P_2BCCD8, &mut vel);
        self.m.set_v3(0x1070, o);
        let p = [f32::from_bits(rec[4]) + o[0], f32::from_bits(rec[5]) + o[1], f32::from_bits(rec[6]) + o[2]];
        for k in 0..3 {
            rec[4 + k] = p[k].to_bits();
        }
        self.m.set_v3(0x1060, p);
        self.m.set_u(0x1050, rec[6]);
        if self.f_2dc != -1 {
            // FUN_0019c8b0: JMP +0x28 != 0 -> flags |= 1
            if self.m.i(0x2e8) != 0 {
                rec[0] |= 1;
            }
        }
        if self.att_4cc != -1 {
            h.po_part_rec(0x18a4e8, obj, 0x4c0, rec);
        }
        if self.dmg.state != -1 {
            let (mut f, mut w) = (rec[0], rec[13]);
            self.dmg.FUN_001a2160(&mut f, &mut w);
            rec[0] = f;
            rec[13] = w;
        }
        let v = h.vcall_114(obj);
        let s = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        let k = SPD_2BCCF0[0] + s * (SPD_2BCCF0[1] + s * SPD_2BCCF0[2]);
        let k = if CLAMP_2BBB60[0] <= k { if CLAMP_2BBB60[1] < k { CLAMP_2BBB60[1] } else { k } } else { CLAMP_2BBB60[0] };
        rec[13] = (f32::from_bits(rec[13]) * k).to_bits();
    }
    /// slot 49 FUN_001358a8(self, rec): camera target
    pub fn ee_sub_1358a8(&mut self, obj: usize, rec: &mut CamRec, g: &ClassGlobals, h: &mut dyn ClassHost) {
        if self.state != 0 && self.state != 9 && self.att_4cc != 0 {
            self.m.set_f(0x1080, g.t);
        }
        self.ee_sub_141640(obj, rec, g, h);
        if self.state == 5 {
            rec[0] |= 1;
        } else if self.state == 0xd {
            // FUN_0019ea00(self + 0xf50, rec): path direction * +0xf64 into +0x20, +0x34 = 0
            h.po_part_rec(0x19ea00, obj, 0xf50, rec);
        }
    }
    /// slot 57 FUN_00140d70: PO FUN_00204768, then the JMP part FUN_00199298 while +0x2dc != -1
    pub fn ee_sub_140d70(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x204768, obj);
        if self.f_2dc != -1 {
            h.po_part(0x199298, obj, 0x2c0);
        }
    }
    /// slot 58 FUN_00140d00: SO FUN_001f4fa8, then JMP FUN_001991a0 / attach FUN_00193de8 / path FUN_0019e950
    pub fn ee_sub_140d00(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1f4fa8, obj);
        if self.f_2dc != -1 {
            h.po_part(0x1991a0, obj, 0x2c0);
        }
        if self.att_4cc != -1 {
            h.po_part(0x193de8, obj, 0x4c0);
        }
        if self.f_f54 != -1 {
            h.po_part(0x19e950, obj, 0xf50);
        }
    }
    /// slot 67 FUN_00140db0(self, a, rec, b, side): pushes against Sly / Bentley / Murray are scaled by 0.01
    /// (unless world +0x150 +0x80 == 1 and !FUN_00144068), then SO FUN_001fad00 on the (restored) record.
    /// `push` = rec +0x40, `other` = the pair object of side 1 - side.
    pub fn ee_sub_140db0(&self, obj: usize, push: [f32; 3], other: u32, side: u32, g: &ClassGlobals, h: &mut dyn ClassHost) {
        let mut v = push;
        let pc = other == g.g_2dd5bc || h.is_player_2e1e40_2f7900(other);
        if pc && (h.world_150_80() != 1 || self.ee_sub_144068()) {
            v = [v[0] * PUSH_SCALE_67, v[1] * PUSH_SCALE_67, v[2] * PUSH_SCALE_67];
        }
        h.ee_sub_1fad00(obj, v, side);
    }
    /// slot 81 FUN_00140c30(self, a, other, yaw): other not in the world (+0x28 bit 60 clear) -> JMP start
    /// FUN_00195a00(+0x2c0, 0, 0, 0); else PO FUN_00205008(self, other, yaw - +0x1010)
    pub fn ee_sub_140c30(&self, obj: usize, other: u32, yaw: f32, h: &mut dyn ClassHost) {
        if !h.lo_28_bit60(other) {
            h.po_part(0x195a00, obj, 0x2c0);
        } else {
            h.ee_sub_205008(obj, other, yaw - self.m.f(0x1010));
        }
    }
    /// slot 28 FUN_00143cf8(self, v): PO translate FUN_002041c0 (REPLACED hook), +0x9e4 = 0
    pub fn ee_sub_143cf8(&mut self, obj: usize, v: [f32; 3], h: &mut dyn ClassHost) {
        h.ee_sub_2041c0(obj, v);
        self.m.set_u(0x9e4, 0);
    }
    /// FUN_001a5bf0(o): o if is-a 0x30, else its parent (+0x10)
    fn ee_sub_1a5bf0(o: u32, h: &mut dyn ClassHost) -> u32 {
        if h.ee_sub_132b10(o, 0x30) == 0 {
            h.obj_i32(o, 0x10) as u32
        } else {
            o
        }
    }
    /// slot 60 FUN_00144388(self, other): contact filter = max(ALO FUN_001f4fc8, +0x11b0 FUN_00220498,
    /// +0x900 FUN_001cc9a8)
    pub fn ee_sub_144388(&self, obj: usize, other: u32, g: &ClassGlobals, h: &mut dyn ClassHost) -> i64 {
        let mut r = crate::engine::classes::actor::c2caf88::ee_sub_1f4fc8(&self.alo, obj, other, h) as i64;
        // FUN_00220498(self + 0x11b0, other, r)
        let k = self.m.i(0x11b4);
        if other == self.m.u(0x11c0) && k != -1 && (k > 2 || (other == g.g_2dd5bc && h.obj_i32(other, 0x2dc) == 0x12)) {
            r = ee_sub_134310(r, 3);
        }
        // FUN_001cc9a8(self + 0x900, other, r)
        if self.m.i(0x9a4) == 1 {
            let p = Self::ee_sub_1a5bf0(self.m.u(0x984), h);
            if p != 0 {
                let e = h.obj_i32(p, 0xec);
                if e != 0 && h.obj_i32(other, 0xec) == e {
                    r = ee_sub_134310(r, 3);
                }
            }
        }
        r
    }
}

// ---- 55h (part): SetState FUN_00135248 and its own small callees

/// DAT_002bd2c0 / DAT_002bd2c4: FUN_001430c8 side-angle bands (|angle| < 1 front, <= 2.5 side, else back)
pub const ANG_2BD2C0: [f32; 2] = [1.0, 2.5];

impl C2cb230 {
    /// FUN_00142868(self, k): +0x1148 = k, +0x114c = FLT_MAX (DAT_002a4d64)
    pub fn ee_sub_142868(&mut self, k: i32) {
        self.m.set_i(0x1148, k);
        self.m.set_f(0x114c, f32::MAX);
    }
    /// FUN_00142880(self): +0x114c = t + rand(+0x1138, +0x113c) (+0x1148 != 0) / rand(+0x1140, +0x1144)
    pub fn ee_sub_142880(&mut self, g: &ClassGlobals, rng: &mut crate::engine::physics::driven::GameRng) {
        let (lo, hi) = if self.m.i(0x1148) == 0 { (self.m.f(0x1140), self.m.f(0x1144)) } else { (self.m.f(0x1138), self.m.f(0x113c)) };
        let r = rng.FUN_0011d448(lo, hi);
        self.m.set_f(0x114c, g.t + r);
    }
    /// FUN_001430c8(self): idle state-machine mask by +0x10fc (1: 0x10 << side, 2: 0x100 << side, 3: 0x1000,
    /// 4: 0x100, else 8); side from the angle of +0x1100's position in this object's frame when +0x10f8 != 0
    /// (the third argument register is the side when +0x10f8 == 0: always 0 from FUN_00135248)
    pub fn ee_sub_1430c8(&self, obj: usize, h: &mut dyn ClassHost) -> i32 {
        let mut side = 0u32;
        if self.m.i(0x10f8) != 0 {
            let p = h.ee_sub_1a62e8(self.m.u(0x10f8));
            let l = h.ee_sub_1a6f00(obj, p);
            // _prot3w: (y, z, x) -> atan2(y, x)
            let a = l[1].atan2(l[0]);
            side = 0;
            if ANG_2BD2C0[0] <= a.abs() {
                side = 3;
                if a.abs() <= ANG_2BD2C0[1] {
                    side = 2;
                    if 0.0 < a {
                        side = 1;
                    }
                }
            }
        }
        match self.m.i(0x10fc) {
            1 => 0x10 << side,
            2 => 0x100 << side,
            3 => 0x1000,
            4 => 0x100,
            _ => 8,
        }
    }
    /// FUN_001f14f8(sm, self, label, flags, mask, &slot): drop the slot's link, link `sm` (FUN_001f03a8)
    fn ee_sub_1f14f8(&mut self, obj: usize, sm: u32, label: i32, flags: u32, mask: i32, h: &mut dyn ClassHost) {
        let mut s = self.m.u(0x1034);
        h.ee_sub_1f15b8(&mut s);
        if sm != 0 {
            s = h.ee_sub_1f03a8_link(sm, handle(obj), label, flags, mask);
            if s != 0 {
                h.vcall(s, 0xc);
            }
        }
        self.m.set_u(0x1034, s);
    }
    /// shared tail of FUN_00135248 / FUN_00136040: anim slot +0x1030 restarted with `anim`, vt +0x2d4
    fn play_1030(&mut self, obj: usize, start: u32, rate: f32, anim: u32, flags: u32, h: &mut dyn ClassHost) {
        let mut s = self.m.u(0x1030);
        super::shared::anim::ee_sub_1305b8(start, rate, anim, handle(obj), flags, &mut s, h);
        self.m.set_u(0x1030, s);
    }
    /// FUN_00136040(self, s): state-6 sub-state +0x16b0 (0 +0x14e4 and +0x190 cleared, 1 +0x15f0,
    /// 2 +0x14e8 or +0x15d4)
    pub fn ee_sub_136040(&mut self, obj: usize, s: i32, h: &mut dyn ClassHost) {
        if s == self.f_16b0 {
            return;
        }
        self.f_16b0 = s;
        let mut flags = 1;
        let mut a = 0;
        match s {
            1 => {
                a = self.m.u(0x15f0);
                flags = 9;
            }
            0 => {
                a = self.m.u(0x14e4);
                self.m.set_v3(0x190, [0.0; 3]);
            }
            2 => {
                a = self.m.u(0x14e8);
                if a == 0 {
                    a = self.m.u(0x15d4);
                }
                flags = 9;
            }
            _ => {}
        }
        let mut sl = self.m.u(0x1034);
        h.ee_sub_1f15b8(&mut sl);
        self.m.set_u(0x1034, sl);
        self.play_1030(obj, 0, 1.0, a, flags, h);
        h.vcall(handle(obj), 0x2d4);
    }
    /// FUN_00135248(self, s): Bentley SetState. Exit of the old state (components +0x2c0 / +0x4c0 / +0xdf0 /
    /// +0x11b0 / +0xf50 told -1, idle timer, carried object +0x186c), +0x13d0 = s, +0x13d4 = t, enter (anim
    /// +0x1030 and state machine +0x1034 by state; 6 / 8 / 0xd hand over, 0xc plays nothing).
    pub fn ee_sub_135248(&mut self, obj: usize, s: i32, g: &ClassGlobals, rng: &mut crate::engine::physics::driven::GameRng, h: &mut dyn ClassHost) {
        let o = handle(obj);
        if s == self.state {
            return;
        }
        match self.state {
            0 => {
                h.ee_sub_2303a0_off(o, 0x58, 2);
                h.ee_sub_2303a0_off(o, 0x5c, 2);
                self.ee_sub_142880(g, rng);
            }
            1 | 7 => self.m.set_f(0x1a0, h.anim_slot_phase(self.m.u(0x1030))),
            2 => {
                if self.m.i(0x108c) == 0 {
                    let mut x = 0;
                    if s >= 0 {
                        if s < 2 {
                            x = self.m.u(0x16d0);
                        } else if s == 3 && self.att_4c8 != 1 {
                            let k = self.att_4c8;
                            x = if k < 1 || k > 4 || k < 3 { self.m.u(0x16d0) } else { self.m.u(0x16d4) };
                        }
                    }
                    if x != 0 {
                        h.vcall(x, 0x44);
                        h.ee_sub_15c150(x, o);
                        h.vcall(x, 0x4c);
                    }
                }
                h.po_part_s(0x195fb8, obj, 0x2c0, -1);
            }
            3 => h.po_part_s(0x192bd8, obj, 0x4c0, -1),
            4 => h.po_part_s(0x1a0b40, obj, 0xdf0, -1),
            6 => {
                if self.m.u(0x186c) != 0 {
                    h.vcall_arg(self.m.u(0x186c), 0x264, 1);
                    // FUN_00136298: drop the carried object (+0x186c / +0x16b4)
                    // K18: in-set FUN_00136298 (clears both fields itself)
                    self.ee_sub_136298(obj, h);
                }
                self.ee_sub_136040(obj, -1, h);
            }
            8 => {
                self.ee_sub_136530(obj, -1, g, h);
                h.vcall_arg(o, 0x27c, 1);
            }
            9 | 10 | 0xb => {
                h.ee_sub_2303a0_off(o, 0x58, 2);
                h.ee_sub_2303a0_off(o, 0x5c, 2);
            }
            0xc => h.po_part_s(0x2206c8, obj, 0x11b0, -1),
            0xd => h.po_part_s(0x19eab0, obj, 0xf50, -1),
            _ => {}
        }
        let old = self.state;
        self.state = s;
        let (mut anim, mut sm, mut flags, mut label, mut mask, mut start, mut rate) = (0u32, 0u32, 1u32, -1i32, 4i32, 0f32, 1f32);
        self.m.set_f(0x13d4, g.t);
        match s {
            0 => {
                h.ee_sub_2303a0_off(o, 0x58, 1);
                h.ee_sub_2303a0_off(o, 0x5c, 1);
                if self.m.i(0x181c) == 0 {
                    sm = self.m.u(0x1028);
                    label = if old == 2 { 0x178 } else if old == 7 { 0x176 } else { 0x175 };
                    if h.ee_sub_1f0400(sm, label) < 0 {
                        label = -1;
                    }
                } else {
                    anim = self.m.u(0x1820);
                    flags = 9;
                }
                self.m.set_v3(0x190, [0.0; 3]);
                self.m.set_f(0x19c, 0.0);
                mask = self.ee_sub_1430c8(obj, h);
                self.ee_sub_142868(1);
            }
            1 | 7 => {
                let blk = if s == 1 { 0x1320 } else { 0x1600 };
                anim = self.m.u(blk + 4);
                flags = 9;
                start = self.m.f(0x1a0) * h.anim_74(anim);
                // FUN_0020bd68(blk)
                self.m.set_u(blk + 0x9c, 0);
                self.m.set_u(blk + 0xa0, 0);
            }
            4 => h.ee_sub_19cd38_po(obj, -1),
            5 => {
                anim = self.m.u(0x14e0);
                flags = 9;
            }
            6 => {
                self.ee_sub_136040(obj, 0, h);
                return;
            }
            8 => {
                self.ee_sub_136530(obj, 0, g, h);
                return;
            }
            9 | 10 | 0xb => {
                if s == 9 {
                    sm = self.m.u(0x102c);
                }
                h.ee_sub_2303a0_off(o, 0x58, 1);
                h.ee_sub_2303a0_off(o, 0x5c, 1);
                self.m.set_v3(0x190, [0.0; 3]);
                self.m.set_f(0x19c, 0.0);
            }
            0xc => return,
            0xd => {
                h.po_part_s(0x19eab0, obj, 0xf50, 0);
                return;
            }
            0xe => {
                anim = self.m.u(0x1878);
                flags = 9;
                if anim == 0 {
                    anim = self.m.u(0x1374);
                    rate = 1.5;
                }
            }
            _ => {}
        }
        let mut sl = self.m.u(0x1030);
        super::shared::anim::ee_sub_130658(&mut sl, h);
        self.m.set_u(0x1030, sl);
        let mut sl = self.m.u(0x1034);
        h.ee_sub_1f15b8(&mut sl);
        self.m.set_u(0x1034, sl);
        self.play_1030(obj, start.to_bits(), rate, anim, flags, h);
        self.ee_sub_1f14f8(obj, sm, label, flags, mask, h);
        h.vcall(o, 0x2d4);
    }
}
