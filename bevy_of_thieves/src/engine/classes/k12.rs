//! WP-K12: class 107 C2dcc98 (SO, parent C2d4260): player-proximity glow. Slot 6 FUN_00229ef0 init, slot 14
//! FUN_00229f88 load-post (clears +0x40 bit 3 of the +0x16c / +0x170 / +0x174 objects), slot 15 FUN_0022a060
//! per-frame proximity state +0x164 (vtable +0x1c 5 / 6), 1:1. Props: 1313 -> +0x154 (is-a class), 1316 ->
//! +0x150 mode, 1317 -> +0x168, 1318-1320 (oid refs) -> +0x16c / +0x170 / +0x174. Handles = object index + 1.
//! REPLACED: slot 18 FUN_0022a908 (VU glow / flare draw of the +0x16c..+0x174 objects: FUN_0022a220,
//! FUN_0011ee08 ray, FUN_001a5ee8 / FUN_001a5fb8 matrix draws, FUN_001cd780 sprite) is renderer-side, a
//! `ClassHost::base(0x22a908)` hook; the oid refs are resolved at load with FUN_0015e938 (first object by oid).
use super::k4::word;
use super::*;
use crate::engine::gfx::attach::alo_of;
use crate::engine::world::props::{Prop, PropValue};

/// K12 world class id -> vtable (kept in `ClassSet::k12`).
pub const PORTED_K12: &[(ClassId, u32)] = &[(107, 0x2dcc98)];

/// slot 5 FUN_001140c0 (class descriptor getter)
pub fn slot5(class: ClassId) -> Option<u32> {
    (class == 107).then_some(0x2bb344)
}

fn handle(obj: usize) -> u32 {
    obj as u32 + 1
}

#[derive(Clone, Debug, PartialEq)]
pub struct C2dcc98 {
    /// +0x40 flags (slot 6 |= 10), +0x148 bit 35 (slot 6 clears)
    pub f_40: u32,
    pub bit35: bool,
    /// +0x150 mode (field 1316; 0 / 2 = off while DAT_003e0f9c), +0x154 is-a class of the player (field 1313)
    pub f_150: i32,
    pub f_154: i32,
    /// +0x158 / +0x15c on / off radius
    pub f_158: f32,
    pub f_15c: f32,
    /// +0x164 proximity state
    pub f_164: i32,
    /// +0x168 (field 1317)
    pub f_168: u32,
    /// +0x16c / +0x170 / +0x174 glow objects (fields 1318-1320, handles)
    pub f_16c: [u32; 3],
    /// +0x178 / +0x184
    pub f_178: i32,
    pub f_184: i32,
}

impl Default for C2dcc98 {
    fn default() -> Self {
        C2dcc98 { f_40: 0, bit35: false, f_150: 0, f_154: 0, f_158: 0.0, f_15c: 0.0, f_164: 0, f_168: 0, f_16c: [0; 3], f_178: 0, f_184: 0 }
    }
}

impl C2dcc98 {
    /// slot 6 FUN_00229ef0: base FUN_001f2108, +0x154 = 0x32, radii 300 / 500, +0x184 = +0x178 = -1,
    /// +0x40 |= 10, +0x148 bit 35 cleared
    pub fn ee_sub_229ef0(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1f2108, obj);
        self.f_154 = 0x32;
        self.f_158 = 300.0;
        self.f_15c = 500.0;
        self.f_184 = -1;
        self.f_178 = -1;
        self.f_40 |= 10;
        self.bit35 = false;
    }

    /// property rows (fields 1313 / 1316 / 1317; 1318-1320 through `oid`)
    pub fn feed(&mut self, props: &[Prop], oid: &mut dyn FnMut(i32) -> u32) {
        for p in props {
            let PropValue::Scalar(b) = &p.value else { continue };
            match p.field {
                1313 => self.f_154 = u16::from_le_bytes([b.first().copied().unwrap_or(0), b.get(1).copied().unwrap_or(0)]) as i32,
                1316 => self.f_150 = word(b) as i32,
                1317 => self.f_168 = word(b),
                1318..=1320 => self.f_16c[(p.field - 1318) as usize] = oid(word(b) as i32),
                _ => {}
            }
        }
    }

    /// slot 14 FUN_00229f88: base FUN_001f36a8, then clear +0x40 bit 3 of each set +0x16c / +0x170 / +0x174
    pub fn ee_sub_229f88(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1f36a8, obj);
        for o in self.f_16c {
            if o != 0 {
                h.obj_and_40(o, !8);
            }
        }
    }

    /// slot 15 FUN_0022a060: base FUN_001f36e0; on = player (FUN_001c4840) is-a +0x154 and closer than
    /// +0x158 (off) / +0x15c (on); modes 0 / 2 are off while DAT_003e0f9c; a JT (is-a 0x34) only turns it on
    /// when free (+0x2dc == -1, vtable +0x26c != 2), else the state is kept; a change sets +0x164 and calls
    /// vtable +0x1c with 5 (on) / 6 (off)
    pub fn ee_sub_22a060(&mut self, obj: usize, g: &ClassGlobals, h: &mut dyn ClassHost) {
        h.base(0x1f36e0, obj);
        let p = h.ee_sub_1c4840();
        let mut on = false;
        if p != 0 && h.ee_sub_132b10(p, self.f_154) != 0 {
            let r = if self.f_164 == 0 { self.f_158 } else { self.f_15c };
            let (a, b) = (h.ee_sub_1a62e8(p), h.ee_sub_1a62e8(handle(obj)));
            let d = ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
            on = d < r;
        }
        if (self.f_150 == 0 || self.f_150 == 2) && g.g_3e0f9c != 0 {
            on = false;
        }
        let mut s = on as i32;
        if on && h.ee_sub_132b10(p, 0x34) != 0 && !h.jt_free_2dc_26c(p) {
            s = self.f_164;
        }
        if s != self.f_164 {
            self.f_164 = s;
            h.vcall_1c(obj, if s == 0 { 6 } else { 5 }, 0);
        }
    }

    /// slot 18 FUN_0022a908 (REPLACED: VU glow / flare draw, renderer hook)
    pub fn ee_sub_22a908(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x22a908, obj);
    }
}

impl ClassSet {
    /// K12: construct + init (slot 6) + feed the `PORTED_K12` classes (clones read their source's props)
    pub(super) fn build_k12(&mut self, w: &World, h: &mut dyn ClassHost) {
        for (i, o) in w.objects.iter().enumerate() {
            if !PORTED_K12.iter().any(|x| x.0 == o.class) {
                continue;
            }
            let src = o.clone_of.unwrap_or(i);
            let mut c = C2dcc98::default();
            c.ee_sub_229ef0(i, h);
            let props = props_of(w, src).map(|p| p.to_vec()).or_else(|| alo_of(w, src).map(|a| a.props.clone())).unwrap_or_default();
            c.feed(&props, &mut |oid| h.ee_sub_15e938(0, oid, handle(i)));
            self.k12.push((i, Inst::C2dcc98(c)));
        }
    }
    /// K12 state at object index `obj`
    pub fn k12_get(&mut self, obj: usize) -> Option<&mut C2dcc98> {
        self.k12.iter_mut().find(|x| x.0 == obj).and_then(|x| if let Inst::C2dcc98(c) = &mut x.1 { Some(c) } else { None })
    }
    /// K12 load-post: slot 14
    pub(super) fn post_load_k12(&mut self, h: &mut dyn ClassHost) {
        for (i, x) in &mut self.k12 {
            if let Inst::C2dcc98(c) = x {
                c.ee_sub_229f88(*i, h);
            }
        }
    }
    /// K12 per frame: slot 15, then slot 18 (draw hook)
    pub(super) fn frame_k12(&mut self, h: &mut dyn ClassHost) {
        for (i, x) in &mut self.k12 {
            if let Inst::C2dcc98(c) = x {
                c.ee_sub_22a060(*i, &self.g, h);
                c.ee_sub_22a908(*i, h);
            }
        }
    }
}
