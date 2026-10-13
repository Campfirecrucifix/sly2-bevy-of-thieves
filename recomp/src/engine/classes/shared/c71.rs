//! WP-K14: class 71 (vtable 0x2d1878, parent 50; children 72-76) parts shared by the K14 children 76 / 73:
//! init FUN_0017f090, slot 11 FUN_0017f110, load-post part FUN_0017f138, slot 25 FUN_0017f170, slot 54 base
//! FUN_0017f1a0, deferred-touch request FUN_0017f5b0 / cancel FUN_0017f6b0; field rows 1363-1370.
use super::super::ClassHost;
use crate::engine::world::props::{Prop, PropValue};

/// class 71 field rows -> member offsets (FIELD_MEMBER)
pub const ROWS_71: &[(i16, u16)] = &[(1363, 0x180), (1364, 0x184), (1365, 0x188), (1366, 0x190), (1369, 0x1a0), (1370, 0x1b0)];

/// DAT_002a5630: PMF {adj 0, vindex 0x53 -> vtbl + 0x53 * 8 - 8 = +0x290 / +0x294 = slot 81, vptr at +0xc}
pub const PMF_2A5630_SLOT: u32 = 81;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct C71 {
    /// ALO +0x40 flag word (inline)
    pub f_40: u32,
    /// +0x180 (10) / +0x184 (copy of +0x180 on reset / slot 25), +0x188 touch damage, +0x18c touch pending,
    /// +0x190 hit-fx emitter (object handle), +0x1a0 {300, 50, 0, 0}, +0x1b0 {0, 800}
    pub f_180: i32,
    pub f_184: i32,
    pub f_188: i32,
    pub f_18c: i32,
    pub f_190: u32,
    pub f_1a0: [f32; 4],
    pub f_1b0: [f32; 2],
}

fn word(b: &[u8], k: usize) -> u32 {
    b.get(k * 4..k * 4 + 4).map(|x| u32::from_le_bytes([x[0], x[1], x[2], x[3]])).unwrap_or(0)
}

impl C71 {
    /// FUN_0017f090: base FUN_001f2108; +0x180 = 10; +0x1a0 = DAT_002be7d0 {300, 50, 0, 0}; +0x1b0 =
    /// DAT_002be7e0 {0, 800}; +0x40 |= 2
    pub fn ee_sub_17f090(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1f2108, obj);
        self.f_180 = 10;
        self.f_1a0 = [300.0, 50.0, 0.0, 0.0];
        self.f_1b0 = [0.0, 800.0];
        self.f_40 |= 2;
    }
    /// slot 11 FUN_0017f110: base FUN_001f2d48; +0x18c = 0
    pub fn ee_sub_17f110(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1f2d48, obj);
        self.f_18c = 0;
    }
    /// FUN_0017f138: base FUN_001f36a8 (the caller then runs vtable +0x29c = slot 82)
    pub fn ee_sub_17f138(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1f36a8, obj);
    }
    /// slot 25 FUN_0017f170: base FUN_001cb518; +0x184 = +0x180
    pub fn ee_sub_17f170(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1cb518, obj);
        self.f_184 = self.f_180;
    }
    /// FUN_0017f1a0(self, contact): base FUN_001f4fb0 (empty); contact +0x98 = +0x94 = 0 (the contact record
    /// is the solver's: not modelled)
    pub fn ee_sub_17f1a0(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1f4fb0, obj);
    }
    /// FUN_0017f5b0(self, other): once (+0x18c 0 -> 1) queue the deferred call FUN_00149a50(0x3dd490, self,
    /// DAT_002a5630 = slot 81, other). Returns whether a call was queued.
    pub fn ee_sub_17f5b0(&mut self) -> bool {
        if self.f_18c == 0 {
            self.f_18c = 1;
            return true;
        }
        false
    }
    /// FUN_0017f6b0: +0x184 = +0x180; a pending touch (+0x18c) is cancelled with FUN_00149b58(0x3dd490, 1, self,
    /// DAT_002a5638, 0) (mask 1 = every node of this object). Returns whether to cancel.
    pub fn ee_sub_17f6b0(&mut self) -> bool {
        self.f_184 = self.f_180;
        self.f_18c != 0
    }
    /// member write by offset (raw prop words)
    pub fn set(&mut self, off: u16, b: &[u8]) {
        match off {
            0x180 => self.f_180 = word(b, 0) as i32,
            0x184 => self.f_184 = word(b, 0) as i32,
            0x188 => self.f_188 = word(b, 0) as i32,
            0x190 => self.f_190 = word(b, 0),
            0x1a0 => self.f_1a0 = [0, 1, 2, 3].map(|k| f32::from_bits(word(b, k))),
            0x1b0 => self.f_1b0 = [0, 1].map(|k| f32::from_bits(word(b, k))),
            _ => {}
        }
    }
    /// rows 1363-1370 (1366 = object reference: resolved by the caller, raw oid kept here)
    pub fn feed(&mut self, props: &[Prop]) {
        for p in props {
            let PropValue::Scalar(b) = &p.value else { continue };
            if let Some(&(_, off)) = ROWS_71.iter().find(|r| r.0 == p.field) {
                self.set(off, b);
            }
        }
    }
}

/// class-71 slot 74 part FUN_0017f1d0(self, hit): only hits with +0x44 bit 3; hp +0x184 += +0x10; at < 1 the
/// inherited slot 81 (vtable +0x294, FUN_0017f5f8(self, 0)); else, with a physics block not locked (+0xe8
/// +0xc0 != 3), a horizontal knock-back away from the hit point (+0x20): v += dir * clamp(poly +0x1a0 of the
/// damage, +0x1b0), clamped to +0x1b4 (FUN_0011e9f8), z = 0, projected by the constraint (FUN_001fa810) and set
/// (vtable +0x11c). Returns whether the hit was taken.
pub fn ee_sub_17f1d0(b: &mut C71, phys: &mut crate::engine::physics::driver::AloPhys, obj: usize, hit: &crate::engine::classes::guard::slots::GuardHit, h: &mut dyn ClassHost) -> bool {
    if hit.f_44 & 8 == 0 {
        return false;
    }
    b.f_184 += hit.amount;
    if b.f_184 < 1 {
        ee_sub_17f5f8(b, phys.v, obj, 0, h);
        return true;
    }
    if phys.ckind == 3 {
        return true;
    }
    let p = phys.pos;
    let mut d = [p[0] - hit.pos[0], p[1] - hit.pos[1], p[2] - hit.pos[2]];
    d[2] = 0.0; // minus its DAT_002a2dc0 (z) component
    let l = (d[0] * d[0] + d[1] * d[1]).sqrt();
    let n = if l < 0.0001 { [1.0, 0.0, 0.0] } else { [d[0] / l, d[1] / l, 0.0] };
    let x = -hit.amount as f32;
    let m = b.f_1a0[0] + x * (b.f_1a0[1] + x * b.f_1a0[2]);
    let mut k = b.f_1b0[0];
    if b.f_1b0[0] <= m {
        k = b.f_1b0[1];
        if m <= b.f_1b0[1] {
            k = m;
        }
    }
    let v = [0, 1, 2].map(|i| phys.v[i] + n[i] * k);
    let v = crate::engine::player::p7_damage::FUN_0011e9f8(b.f_1b0[1], v);
    let v = crate::engine::physics::driver::FUN_001fa810(phys.ckind, phys.cnorm, [v[0], v[1], 0.0]);
    phys.v = [v[0], v[1], 0.0];
    true
}

/// inherited slot 81 FUN_0017f5f8(self, other): with `other` and touch damage +0x188, FUN_001f7598 {self, other,
/// kind 0x100, -+0x188, self position, velocity `v`}; the hit fx +0x190 (FUN_0015c150); then vtable +0x4c
/// (slot 8: leave the world). Returns the hit record sent.
pub fn ee_sub_17f5f8(b: &mut C71, v: [f32; 3], obj: usize, other: u32, h: &mut dyn ClassHost) -> Option<crate::engine::classes::guard::slots::GuardHit> {
    let me = obj as u32 + 1;
    let mut out = None;
    if other != 0 && b.f_188 != 0 {
        let p = h.ee_sub_1a62e8(me);
        let hit = crate::engine::classes::guard::slots::GuardHit { src: me, target: other, kind: 0x100, amount: -b.f_188, pos: [p[0], p[1], p[2], 0.0], vec: [v[0], v[1], v[2], 0.0], ..Default::default() };
        h.ee_sub_1f7598(&hit);
        out = Some(hit);
    }
    if b.f_190 != 0 {
        h.ee_sub_15c150(b.f_190, me);
    }
    h.vcall(me, 0x4c);
    out
}
