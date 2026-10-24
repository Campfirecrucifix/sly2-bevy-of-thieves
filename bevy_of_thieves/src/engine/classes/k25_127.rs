//! WP-K25: C2dabf0 (world class 127, size 0x120, parent class 126 C2c9670 = the sequencer / UI screen base of
//! FUN_002123c8 / FUN_00212c50, like class 129 C2dad90): a prompt screen that, once its sequence (+0x58 state 2)
//! has run for 0.5 s with neither the 0x2e6550 nor the 0x3da4f0 screen up, waits for a pad press (mask 0xf0),
//! un-pauses (FUN_0020f030), bumps the DAT_002dd318 counter + FUN_00162df8 unless +0x114, and finishes
//! (vtable +0x194). Slot 48 starts the sequence: pauses (FUN_0020efc0) and opens its 0x33a page.
//! Own slots 5 FUN_00114200, 6 FUN_00211d08, 15 FUN_00211d38, 48 FUN_00211ca8 + FUN_002121c8 1:1. Inherited
//! slot 14 FUN_002123c8 (post-load) = `ClassHost::base`.
//! Fields: +0x110 done (field 1148), +0x114 no-count (field 1149) are this class's; +0xb0 and the sequencer
//! state +0x58 / +0x5c / +0x64 / +0x7c belong to the unported base (read through `ClassHost::obj_i32` /
//! `obj_f32`; +0xb0 REPLACED: kept in the instance).
//! Hooks (`ClassHost::ee_call`): the sequencer base FUN_00212c50 (`base`), FUN_002128e8 (start page), FUN_00213b00
//! (page select), the screen set test FUN_001d9880 (FUN_0011a380(0x3d9e70, x)), the pad edge FUN_00187d80
//! (0x2dfc00), the job FUN_00162df8. REPLACED: DAT_002ddf0c (game clock) -> `ClassGlobals::t`.
use super::k4::word;
use super::{ClassGlobals, ClassHost};
use crate::engine::world::props::{Prop, PropValue};

/// field 1148 (+0x110) / 1149 (+0x114)
pub const FIELD_110: i16 = 1148;
pub const FIELD_114: i16 = 1149;
/// FUN_001d9880 arguments (screens that hold the prompt) and the FUN_00187d80 pad mask
pub const SCREEN_2E6550: u32 = 0x2e6550;
pub const SCREEN_3DA4F0: u32 = 0x3da4f0;
pub const PAD_2DFC00: u32 = 0x2dfc00;
pub const PAD_MASK: u32 = 0xf0;
/// slot 48 page id (FUN_00213b00)
pub const PAGE_33A: u32 = 0x33a;
/// DAT_002dd318 (count of answered prompts; +1 each, the job FUN_00162df8 follows when DAT_003e0f9c)
pub const T_HOLD: f32 = 0.5;

#[derive(Clone, Debug, Default)]
pub struct C2dabf0 {
    /// +0xb0 (base field set by slot 6)
    pub f_b0: i32,
    /// +0x110 done
    pub f_110: i32,
    /// +0x114 no-count
    pub f_114: i32,
}

/// slot 5: FUN_00114200
pub fn ee_sub_114200() -> u32 {
    0x2bb4d4
}

/// FUN_002121c8(seq): +0x58 < 2 and +0x64 < 2 (the sequence is not running)
pub fn ee_sub_2121c8(f_58: i32, f_64: i32) -> bool {
    f_58 < 2 && f_64 < 2
}

impl C2dabf0 {
    /// load props of fields 1148 / 1149
    pub fn feed(&mut self, props: &[Prop]) {
        for p in props {
            if let PropValue::Scalar(b) = &p.value {
                match p.field {
                    FIELD_110 => self.f_110 = word(b) as i32,
                    FIELD_114 => self.f_114 = word(b) as i32,
                    _ => {}
                }
            }
        }
    }
    /// slot 6: FUN_00211d08 = FUN_001a5270, +0xb0 = 1
    pub fn ee_sub_211d08(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1a5270, obj);
        self.f_b0 = 1;
    }
    /// slot 15: FUN_00211d38
    pub fn ee_sub_211d38(&mut self, obj: usize, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        let me = obj as u32 + 1;
        if ee_sub_2121c8(h.obj_i32(me, 0x58), h.obj_i32(me, 0x64)) {
            return;
        }
        h.base(0x212c50, obj);
        if h.obj_i32(me, 0x58) == 2
            && T_HOLD < g.t - h.obj_f32(me, 0x5c)
            && h.ee_call(0x1d9880, &[SCREEN_2E6550]) == 0
            && h.ee_call(0x1d9880, &[SCREEN_3DA4F0]) == 0
            && h.ee_call(0x187d80, &[PAD_2DFC00, PAD_MASK]) != 0
        {
            h.ee_sub_20f030(0);
            if self.f_114 == 0 && g.g_3e0f9c != 0 {
                g.g_2dd318 += 1;
                h.ee_call(0x162df8, &[]);
            }
            h.vcall(me, 0x194);
        }
        self.f_110 = 1;
    }
    /// slot 48: FUN_00211ca8(self, a): only while +0x7c == -1: FUN_002128e8(self, a), FUN_0020efc0(0x3dd490, 0),
    /// +0x110 = 1, FUN_00213b00(self, 0x33a)
    pub fn ee_sub_211ca8(&mut self, obj: usize, a: u32, h: &mut dyn ClassHost) {
        let me = obj as u32 + 1;
        if h.obj_i32(me, 0x7c) != -1 {
            return;
        }
        h.ee_call(0x2128e8, &[me, a]);
        h.ee_sub_20efc0(0);
        self.f_110 = 1;
        h.ee_call(0x213b00, &[me, PAGE_33A]);
    }
}

/// K25: class 127 in the ClassSet (`ClassSet::k25c`)
impl super::ClassSet {
    pub(super) fn build_k25(&mut self, w: &crate::engine::world::World, h: &mut dyn ClassHost) {
        for (i, o) in w.objects.iter().enumerate() {
            if o.class != 127 {
                continue;
            }
            let mut c = C2dabf0::default();
            c.ee_sub_211d08(i, h);
            if let Some(p) = w.objects.get(o.clone_of.unwrap_or(i)).and_then(crate::engine::game::fields::object_props) {
                c.feed(p);
            }
            self.k25c.push((i, super::Inst::K25C127(c)));
        }
    }
    /// inherited slot 14 FUN_002123c8
    pub(super) fn post_load_k25(&mut self, h: &mut dyn ClassHost) {
        for (i, x) in &self.k25c {
            if let super::Inst::K25C127(_) = x {
                h.base(0x2123c8, *i);
            }
        }
    }
    pub(super) fn frame_k25(&mut self, h: &mut dyn ClassHost) {
        for (i, x) in &mut self.k25c {
            // FEEL R33: slot 15 runs from the world update list only (LO +0x28 bit 60)
            if h.lo_removed(*i) {
                continue;
            }
            if let super::Inst::K25C127(c) = x {
                c.ee_sub_211d38(*i, &mut self.g, h);
            }
        }
    }
    pub fn k25_127_get(&self, obj: usize) -> Option<&C2dabf0> {
        self.k25c.iter().find(|x| x.0 == obj).and_then(|x| match &x.1 {
            super::Inst::K25C127(c) => Some(c),
            _ => None,
        })
    }
    /// slot 48 of a class-127 object (for the vtable caller)
    pub fn k25_127_start(&mut self, obj: usize, a: u32, h: &mut dyn ClassHost) {
        if let Some((_, super::Inst::K25C127(c))) = self.k25c.iter_mut().find(|x| x.0 == obj) {
            c.ee_sub_211ca8(obj, a, h);
        }
    }
}
