//! WP-K21: C2d4260 (world class 49, parent C2d9978 / SO; parent of C2caf88 / C2d4b78): the plain SO object.
//! Own overrides: slots 5, 6, 14, 18, 54, 60; 15 / 16 are the ALO core (actor/alo.rs).
//! 1:1: slot 6 FUN_001b4438, slot 14 FUN_001f36a8 (= `h.base`, the ALO pose), slot 18 FUN_001b4478, slot 54
//! FUN_001f4fb0 (empty), slot 60 FUN_001f4fc8 (c2caf88::ee_sub_1f4fc8).
//! REPLACED: slot 18's +0xc8 branch (FUN_001d57a8 / FUN_001d5808 draw-list build) = `ee_call(0x1d5808, [obj])`.
use super::super::ClassHost;
use super::alo::Alo;

#[derive(Clone, Debug, Default)]
pub struct C2d4260 {
    pub alo: Alo,
    /// +0xc8 (200): the alternative draw path (never set by a loader field)
    pub f_c8: u32,
}

/// slot 5: FUN_00113d20
pub fn ee_sub_113d20() -> u32 {
    0x2baebc
}

impl C2d4260 {
    /// slot 6: FUN_001b4438 = FUN_001f2108, then +0x148 bit 35 cleared
    pub fn ee_sub_1b4438(&mut self, obj: usize, owner_bit: Option<bool>, g_2c3b90: u32, h: &mut dyn ClassHost) {
        self.alo.so.ee_sub_1f2108(obj, owner_bit, g_2c3b90, h);
        self.alo.so.f_148 &= 0xffff_fff7_ffff_ffff;
    }
    /// slot 18: FUN_001b4478: FUN_00168638(1.0, 1.0, +0x80, 0, 0) (no +0x2d mask test), or the +0xc8 path
    pub fn ee_sub_1b4478(&mut self, obj: usize, h: &mut dyn ClassHost) {
        if self.f_c8 == 0 {
            h.ee_sub_168638_2(obj, 1.0, 1.0, 0);
        } else {
            h.ee_call(0x1d5808, &[obj as u32 + 1]);
        }
    }
    /// slot 54: FUN_001f4fb0
    pub fn ee_sub_1f4fb0(&mut self) {}
}

/// K21: class 49 in the ClassSet (kept in `ClassSet::k21c`; the K5 list holds 86 / 98 only)
impl super::super::ClassSet {
    pub(in super::super) fn build_k21_49(&mut self, w: &crate::engine::world::World, h: &mut dyn ClassHost) {
        for (i, o) in w.objects.iter().enumerate() {
            if o.class != 49 {
                continue;
            }
            let owner_bit = o.owner.map(|ow| self.bit35_of(ow).unwrap_or_else(|| h.owner_bit35(ow)));
            let mut c = Box::<C2d4260>::default();
            c.alo.so.owner = o.owner;
            c.ee_sub_1b4438(i, owner_bit, self.g.g_2c3b90, h);
            self.k21c.push((i, super::super::Inst::C2d4260(c)));
        }
    }
    /// slot 14 FUN_001f36a8 then core slot 22 FUN_001f3ea8 (as K5)
    pub(in super::super) fn post_load_k21_49(&mut self, h: &mut dyn ClassHost) {
        for (i, x) in &self.k21c {
            if let super::super::Inst::C2d4260(_) = x {
                h.base(0x1f36a8, *i);
                h.base(0x1f3ea8, *i);
            }
        }
    }
    /// core slots 15 FUN_001f36e0 / 16 FUN_001f3980, own slot 18
    pub(in super::super) fn frame_k21_49(&mut self, h: &mut dyn ClassHost) {
        let dt = self.g.dt;
        for (i, x) in &mut self.k21c {
            if let super::super::Inst::C2d4260(c) = x {
                c.alo.ee_sub_1f36e0(*i, dt, h);
                h.base(0x1f3980, *i);
                c.ee_sub_1b4478(*i, h);
            }
        }
    }
}
