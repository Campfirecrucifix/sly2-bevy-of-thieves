//! WP-K9: the vault (class 78, vtable 0x2d5b30, size 0x230; parent C2d5898 class 50). Identified by behaviour:
//! Splice method field 1444 (thunk 0x110a28 -> FUN_001c8af0, arg = the player) only opens it when the clue
//! bottle counter DAT_003e1bf4 has reached the world's total DAT_003e1bf8 (= FUN_001c7d00 = 30, set by
//! FUN_0020f710 at world+0x4768); then Sly turns a 9-position dial (left stick, 3x3 grid DAT_002c0ff0) and the
//! last three digits ((code * 10) % 1000 + digit) must equal the combination +0x1f0 (default 123, property
//! 1439). Opened state persists in the world save variable "fVault" (FUN_001c8110 / FUN_001c8188).
//!
//! Overrides: slot 5 FUN_00113ef0 (descriptor 0x2bb100), 6 FUN_001c8280, 14 FUN_001c82f0, 15 FUN_001c8350;
//! non-virtual SetState FUN_001c8730, dial update FUN_001c84d8, dial sub-state FUN_001c8918.
use super::*;

/// K9 world class id -> vtable (kept in `ClassSet::k9`).
pub const PORTED_K9: &[(ClassId, u32)] = &[(78, 0x2d5b30)];

/// slot 5 FUN_00113ef0: class descriptor 0x2bb100 {parent 0x2baed0, id 0x4e, id_end 0x4f, size 0x230, ctor 0x111de8}
pub const DESC_2BB100: u32 = 0x2bb100;

/// DAT_002c0ff0: the 9 dial stick targets (x, y), digit = index + 1
pub const DIAL_2C0FF0: [[f32; 2]; 9] =
    [[-0.7, 0.7], [0.0, 0.8], [0.7, 0.7], [-0.8, 0.0], [0.0, 0.0], [0.8, 0.0], [-0.7, -0.7], [0.0, -0.8], [0.7, -0.7]];
/// DAT_002c0fe8: distance factor of the current digit (hysteresis)
pub const KEEP_2C0FE8: f32 = 0.7;
/// DAT_002a7798 (FLT_MAX): initial best distance
pub const FAR_2A7798: f32 = f32::MAX;
/// combination default (FUN_001c8280 +0x1f0 = 0x7b)
pub const COMBO_DEFAULT: i32 = 0x7b;
/// FUN_001c7d00: clue bottles per world (world +0x4768 -> DAT_003e1bf8)
pub const BOTTLES_1C7D00: i32 = 0x1e;
/// sequence event FUN_001e0020(seq, 0x655) sent on cancel / success
pub const EV_655: i32 = 0x655;
/// FUN_001c46c0 button masks (+0x150 tracker bits): cancel 0x30, press digit 0xc0
pub const MASK_CANCEL: u32 = 0x30;
pub const MASK_PRESS: u32 = 0xc0;
/// FUN_001f4c58(self, 0x645, 1): the dial child
pub const OID_DIAL: i32 = 0x645;

/// FUN_001a64e8 binding table 0x2c0ec8 (24 x {flags, oid, offset}); flags 0x200 = activate (vt+0x4c) after lookup.
/// Lookup kind = 1 for all rows (flags bits 0/1/0x10/0x20/0x40/0x400 clear).
pub const BIND_2C0EC8: [(u32, i32, u32); 24] = [
    (0x200, 0x63c, 0x1a8), (0x200, 0x63d, 0x1ac), (0x200, 0x63e, 0x1b0), (0x200, 0x63f, 0x1b4), (0x200, 0x640, 0x1b8),
    (0x200, 0x641, 0x1bc), (0x200, 0x642, 0x1c0), (0x200, 0x643, 0x1c4), (0x200, 0x644, 0x1c8), (0x200, 0x646, 0x208),
    (0x200, 0x647, 0x1cc), (0x200, 0x648, 0x1d0), (0x200, 0x649, 0x1d4), (0x200, 0x64a, 0x1d8), (0x200, 0x64b, 0x1dc),
    (0x200, 0x64c, 0x1e0), (0x200, 0x64d, 0x1e4), (0x200, 0x64e, 0x1e8), (0x200, 0x64f, 0x1ec), (0, 0x650, 0x210),
    (0, 0x651, 0x214), (0, 0x652, 0x218), (0, 0x653, 0x21c), (0, 0x654, 0x220),
];

/// Vault state (object fields by offset). Handles are object index + 1 (0 = null).
#[derive(Clone, Debug, PartialEq)]
pub struct Vault {
    /// +0x180 state: 0 closed, 1 approach sequence, 2 dialing, 3 opening, 4 wrong code, 5 open
    pub f_180: i32,
    /// +0x184 / +0x18c state / sub-state start time
    pub f_184: f32,
    /// +0x188 dial sub-state: -1 none, 0 choose, 1 turning, 2 pressing, 3 done
    pub f_188: i32,
    pub f_18c: f32,
    /// +0x190 the player that opened it (FUN_001c8af0 argument)
    pub f_190: u32,
    /// +0x194 current dial digit index (-1 none)
    pub f_194: i32,
    /// +0x198 entered code
    pub f_198: i32,
    /// +0x19c active sequence (= +0x20c[state])
    pub f_19c: u32,
    /// +0x1a0 press anim handle (FUN_001305b8), +0x1a4 FUN_001fe5f0 handle
    pub f_1a0: u32,
    pub f_1a4: u32,
    /// +0x1a8[9] player poses per digit, +0x1cc[9] press anims per digit
    pub f_1a8: [u32; 9],
    pub f_1cc: [u32; 9],
    /// +0x1f0 combination (prop 1439), +0x1f4 dialing cue (1440), +0x1f8 locked cue (1441), +0x1fc opened cue
    /// (1443), +0x200 save flag id (1442)
    pub f_1f0: i32,
    pub f_1f4: i32,
    pub f_1f8: i32,
    pub f_1fc: i32,
    pub f_200: i32,
    /// +0x204 dial child (FUN_001f4c58), its +0xa8 turn-end time
    pub f_204: u32,
    pub dial_a8: f32,
    /// +0x208 turning pose
    pub f_208: u32,
    /// +0x20c[6] sequence per state (+0x20c itself is never bound = 0)
    pub f_20c: [u32; 6],
}

impl Default for Vault {
    fn default() -> Self {
        Vault {
            f_180: -1,
            f_184: 0.0,
            f_188: 0,
            f_18c: 0.0,
            f_190: 0,
            f_194: -1,
            f_198: 0,
            f_19c: 0,
            f_1a0: 0,
            f_1a4: 0,
            f_1a8: [0; 9],
            f_1cc: [0; 9],
            f_1f0: COMBO_DEFAULT,
            f_1f4: -1,
            f_1f8: -1,
            f_1fc: -1,
            f_200: -1,
            f_204: 0,
            dial_a8: 0.0,
            f_208: 0,
            f_20c: [0; 6],
        }
    }
}

fn me(obj: usize) -> u32 {
    obj as u32 + 1
}

/// FUN_001c8110: the world's "fVault" save variable (REPLACED: `ClassGlobals::f_vault`; None = no world path)
pub fn ee_sub_1c8110(g: &ClassGlobals) -> bool {
    g.f_vault.unwrap_or(false)
}

/// FUN_001c8188(on): write "fVault"; first opening also calls FUN_001abd98(0x2e65e0, 0)
pub fn ee_sub_1c8188(g: &mut ClassGlobals, on: bool, h: &mut dyn ClassHost) {
    let Some(v) = g.f_vault else { return };
    if on && !v {
        h.ee_sub_1abd98(0);
    }
    g.f_vault = Some(on);
}

/// FUN_0020f128 / FUN_0020f160 (world +0x3d80 counter; 0 <-> 1 edges = FUN_001a3078 / FUN_001a30d0(0x2e1e60))
fn ee_sub_20f128(g: &mut ClassGlobals, h: &mut dyn ClassHost) {
    g.w_3d80 += 1;
    if g.w_3d80 == 1 {
        h.ee_sub_1a3078(true);
    }
}
fn ee_sub_20f160(g: &mut ClassGlobals, h: &mut dyn ClassHost) {
    g.w_3d80 -= 1;
    if g.w_3d80 == 0 {
        h.ee_sub_1a3078(false);
    }
}

impl Vault {
    /// property feed (fields 1439..1443 of class 78; kind 0x10 = s16 sign-extended by FUN_00117d50)
    pub fn feed(&mut self, props: &[crate::engine::world::props::Prop]) {
        use crate::engine::world::props::PropValue;
        for p in props {
            let PropValue::Scalar(b) = &p.value else { continue };
            let s16 = || (b.len() >= 2).then(|| i16::from_le_bytes([b[0], b[1]]) as i32);
            let s32 = || (b.len() >= 4).then(|| i32::from_le_bytes([b[0], b[1], b[2], b[3]]));
            match p.field {
                1439 => self.f_1f0 = s32().unwrap_or(self.f_1f0),
                1440 => self.f_1f4 = s16().unwrap_or(self.f_1f4),
                1441 => self.f_1f8 = s16().unwrap_or(self.f_1f8),
                1442 => self.f_200 = s32().unwrap_or(self.f_200),
                1443 => self.f_1fc = s16().unwrap_or(self.f_1fc),
                _ => {}
            }
        }
    }

    /// slot 6 FUN_001c8280: parent init FUN_001c3d70, defaults, +0x40 |= 2
    pub fn ee_sub_1c8280(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1c3d70, obj);
        self.f_194 = -1;
        self.f_1f0 = COMBO_DEFAULT;
        self.f_1fc = -1;
        self.f_1f8 = -1;
        self.f_1f4 = -1;
        self.f_200 = -1;
        self.f_180 = -1;
        h.obj_or_40(me(obj), 2);
    }

    /// FUN_001a64e8(self, 0x18, 0x2c0ec8)
    fn bind(&mut self, obj: usize, h: &mut dyn ClassHost) {
        for (flags, oid, off) in BIND_2C0EC8 {
            let r = h.ee_sub_15e938(1, oid, me(obj));
            if r == 0 {
                continue;
            }
            if flags & 0x200 != 0 {
                h.vcall(r, 0x4c);
            }
            match off {
                0x1a8..=0x1c8 => self.f_1a8[((off - 0x1a8) / 4) as usize] = r,
                0x1cc..=0x1ec => self.f_1cc[((off - 0x1cc) / 4) as usize] = r,
                0x208 => self.f_208 = r,
                _ => self.f_20c[((off - 0x20c) / 4) as usize] = r,
            }
        }
    }

    /// slot 14 FUN_001c82f0: base FUN_001f36a8, bindings, dial child, SetState(fVault ? 5 : 0)
    pub fn ee_sub_1c82f0(&mut self, obj: usize, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        h.base(0x1f36a8, obj);
        self.bind(obj, h);
        self.f_204 = h.ee_sub_1f4c58(me(obj), OID_DIAL, 1);
        let s = if ee_sub_1c8110(g) { 5 } else { 0 };
        self.ee_sub_1c8730(obj, s, g, h);
    }

    /// FUN_001c8af0 (Splice method field 1444): open request by `player`
    pub fn ee_sub_1c8af0(&mut self, obj: usize, player: u32, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        if g.g_3e1bf4 < g.g_3e1bf8 {
            h.ee_sub_1618b0();
            if self.f_1f8 != -1 {
                h.ee_sub_1fe5f0(self.f_1f8, 1);
            }
        } else {
            self.f_190 = player;
            self.ee_sub_1c8730(obj, 1, g, h);
        }
    }

    /// slot 15 FUN_001c8350: base FUN_001f36e0, then state transitions until stable
    pub fn ee_sub_1c8350(&mut self, obj: usize, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        h.base(0x1f36e0, obj);
        let mut s = self.f_180;
        loop {
            let busy = |v: &Self, h: &mut dyn ClassHost| v.f_19c != 0 && h.ee_sub_1dffd8(v.f_19c);
            match s {
                1 => {
                    if !busy(self, h) {
                        self.ee_sub_1c8730(obj, 2, g, h);
                    }
                }
                2 => {
                    if busy(self, h) {
                        self.ee_sub_1c84d8(obj, g, h);
                    } else if self.f_198 == self.f_1f0 {
                        self.ee_sub_1c8730(obj, 3, g, h);
                    } else {
                        self.ee_sub_1c8730(obj, 4, g, h);
                    }
                }
                3 => {
                    if !busy(self, h) {
                        ee_sub_1c8188(g, true, h);
                        self.ee_sub_1c8730(obj, 5, g, h);
                    }
                }
                4 => {
                    if !busy(self, h) {
                        self.ee_sub_1c8730(obj, 0, g, h);
                    }
                }
                _ => {}
            }
            if self.f_180 == s {
                return;
            }
            s = self.f_180;
        }
    }

    /// FUN_001c8730 SetState
    pub fn ee_sub_1c8730(&mut self, obj: usize, s: i32, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        if s == self.f_180 {
            return;
        }
        if self.f_1a4 != 0 {
            h.ee_sub_1fe738();
            self.f_1a4 = 0;
        }
        if self.f_19c != 0 {
            h.ee_sub_1dfeb8(self.f_19c);
        }
        // exit actions of the old state
        match self.f_180 {
            2 => {
                self.ee_sub_1c8918(obj, -1, g, h);
                ee_sub_20f160(g, h);
            }
            0 => {
                if self.f_190 != 0 {
                    g.w_3d84 += 1; // FUN_0020f198
                    h.world_3d84_add(1); // CRAWL R97: the one world counter
                }
            }
            3 => {
                if self.f_200 != -1 {
                    h.ee_sub_1daf38(self.f_200, true);
                }
                // quirk kept: +0x1fc defaults to -1, which passes the != 0 test
                if self.f_1fc != 0 {
                    self.f_1a4 = h.ee_sub_1fe5f0(self.f_1fc, 1);
                }
            }
            _ => {}
        }
        self.f_180 = s;
        self.f_184 = g.t;
        match s {
            0 | 5 => {
                if self.f_190 != 0 {
                    h.vcall_arg(self.f_190, 0x27c, 1);
                    self.f_190 = 0;
                    g.w_3d84 -= 1; // FUN_0020f1a8
                    h.world_3d84_add(-1); // CRAWL R97
                }
            }
            1 => h.vcall_arg(me(obj), 0x27c, 1),
            2 => {
                self.f_194 = 5;
                self.ee_sub_1c8918(obj, 1, g, h);
                ee_sub_20f128(g, h);
                if self.f_1f4 != 0 {
                    self.f_1a4 = h.ee_sub_1fe5f0(self.f_1f4, 0);
                }
            }
            -1 => return,
            _ => {}
        }
        let seq = self.f_20c.get(self.f_180 as usize).copied().unwrap_or(0);
        self.f_19c = seq;
        if seq != 0 {
            h.ee_sub_1dfd58(seq, self.f_190);
        }
    }

    /// FUN_001c84d8: dial input while dialing
    pub fn ee_sub_1c84d8(&mut self, obj: usize, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        loop {
            let old = self.f_188;
            if h.k9_pressed(me(obj), MASK_CANCEL) != 0 {
                h.ee_sub_1e0020(self.f_19c, EV_655);
                self.ee_sub_1c8918(obj, 3, g, h);
            }
            match self.f_188 {
                1 => {
                    if self.dial_a8 <= g.t {
                        self.ee_sub_1c8918(obj, 0, g, h);
                    }
                }
                0 => {
                    if h.k9_pressed(me(obj), MASK_PRESS) == 0 {
                        let st = h.k9_stick(me(obj));
                        let (mut best, mut bi) = (FAR_2A7798, -1i32);
                        for (k, p) in DIAL_2C0FF0.iter().enumerate() {
                            let (dx, dy) = (st[0] - p[0], st[1] - p[1]);
                            let mut d = (dx * dx + dy * dy).sqrt();
                            if k as i32 == self.f_194 {
                                d *= KEEP_2C0FE8;
                            }
                            if d < best {
                                best = d;
                                bi = k as i32;
                            }
                        }
                        if bi != self.f_194 {
                            self.f_194 = bi;
                            self.ee_sub_1c8918(obj, 1, g, h);
                        }
                    } else {
                        self.ee_sub_1c8918(obj, 2, g, h);
                    }
                }
                2 => {
                    if self.f_1a0 == 0 || h.ee_sub_131968(self.f_1a0) {
                        self.f_198 = (self.f_198 * 10) % 1000 + 1 + self.f_194;
                        if self.f_198 != self.f_1f0 {
                            self.ee_sub_1c8918(obj, 0, g, h);
                        } else {
                            h.ee_sub_1e0020(self.f_19c, EV_655);
                            self.ee_sub_1c8918(obj, 3, g, h);
                        }
                    }
                }
                _ => {}
            }
            if self.f_188 == old {
                return;
            }
        }
    }

    fn pose(&self, k: i32) -> u32 {
        self.f_1a8.get(k as usize).copied().unwrap_or(0)
    }

    /// FUN_001c8918: dial sub-state
    pub fn ee_sub_1c8918(&mut self, obj: usize, s: i32, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        if s == self.f_188 {
            return;
        }
        // FUN_001306a8(+0x1a0)
        if self.f_1a0 != 0 {
            h.ee_sub_1306a8(self.f_1a0);
            self.f_1a0 = 0;
        }
        self.f_188 = s;
        self.f_18c = g.t;
        match s {
            0 => h.ee_sub_142140(self.f_190, self.pose(self.f_194)),
            -1 => {
                self.f_194 = -1;
                h.ee_sub_142140(self.f_190, 0);
            }
            1 => {
                h.ee_sub_142140(self.f_190, self.f_208);
                self.dial_a8 = g.t + 0.25;
                // REPLACED: dial +0xb0/+0xc0/+0xd0 velocity targets (FUN_0022fcc8 / FUN_0022fc48 of the player and
                // the digit pose) + FUN_002303a0(self, dial, 0x11)
                h.k9_dial_turn(me(obj), self.f_204, self.f_190, self.pose(self.f_194));
            }
            2 => {
                h.ee_sub_142140(self.f_190, self.pose(self.f_194));
                let a = self.f_1cc.get(self.f_194 as usize).copied().unwrap_or(0);
                self.f_1a0 = h.ee_sub_1305b8(a, me(obj));
            }
            _ => {}
        }
    }
}

impl ClassSet {
    /// K9: construct + feed + init (slot 6) the vaults.
    pub(super) fn build_k9(&mut self, w: &World, h: &mut dyn ClassHost) {
        for (i, o) in w.objects.iter().enumerate() {
            if !PORTED_K9.iter().any(|x| x.0 == o.class) {
                continue;
            }
            let mut c = Vault::default();
            c.ee_sub_1c8280(i, h);
            if let Some(p) = props_of(w, o.clone_of.unwrap_or(i)) {
                c.feed(p);
            }
            self.k9.push((i, Inst::K9(Box::new(c))));
        }
    }
    pub(super) fn post_load_k9(&mut self, h: &mut dyn ClassHost) {
        // FUN_0020f710 (world activation): DAT_003e1bf8 = FUN_001c7d00
        self.g.g_3e1bf8 = BOTTLES_1C7D00;
        // a loaded world always has a save path (FUN_00210a88 != -1)
        self.g.f_vault.get_or_insert(false);
        for (i, x) in &mut self.k9 {
            if let Inst::K9(c) = x {
                c.ee_sub_1c82f0(*i, &mut self.g, h);
            }
        }
    }
    pub(super) fn frame_k9(&mut self, h: &mut dyn ClassHost) {
        for (i, x) in &mut self.k9 {
            // FEEL R33: slot 15 runs from the world update list only (LO +0x28 bit 60)
            if h.lo_removed(*i) {
                continue;
            }
            if let Inst::K9(c) = x {
                c.ee_sub_1c8350(*i, &mut self.g, h);
            }
        }
    }
    /// Vault state of object `obj`
    pub fn k9_vault(&mut self, obj: usize) -> Option<&mut Vault> {
        self.k9.iter_mut().find(|x| x.0 == obj).and_then(|(_, x)| match x {
            Inst::K9(c) => Some(&mut **c),
            _ => None,
        })
    }
    /// Splice method field 1444 (thunk 0x110a28 -> FUN_001c8af0) on vault `obj` by `player`
    pub fn k9_try_open(&mut self, obj: usize, player: u32, h: &mut dyn ClassHost) -> bool {
        let Some(k) = self.k9.iter().position(|x| x.0 == obj) else { return false };
        if let Inst::K9(c) = &mut self.k9[k].1 {
            c.ee_sub_1c8af0(obj, player, &mut self.g, h);
        }
        true
    }
}
