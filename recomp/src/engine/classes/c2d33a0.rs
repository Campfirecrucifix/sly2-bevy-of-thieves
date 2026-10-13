//! C2d33a0 (world class 4): LIGHT. Init (slot 6 FUN_001a3360), the setters and the FUN_001a3968 math
//! are ported in `gfx::light`; this file ports the class virtuals that re-run FUN_001a3968 and the
//! static light table registration (g_lsm 0x3dd490 +0x4154 table, FUN_001a3d00 / FUN_001a3e88).
//! Object handles in hooks are index + 1 (0 = null).
use super::{ClassGlobals, ClassHost};
use crate::engine::gfx::light::{self, LightDerived, LightParams};
use crate::engine::world::props::Prop;

/// g_lsm (0x3dd490) light table members.
#[derive(Clone, Debug)]
pub struct LightTable {
    /// +0x4154: slot -> light object handle
    pub slots: [u32; 256],
    /// +0x4570: global (directional) light bit set
    pub global: [u32; 8],
    /// +0x4590: free slot bits. Assumed all free at level start (the table constructor is not in the
    /// decompiled set; FUN_001a3d00 can only allocate from set bits).
    pub free: [u32; 8],
    /// +0x4554: table dirty (set by removal; registration skips the region/global insert while set)
    pub dirty: i32,
    /// +0x455c: BSP region tree (non-zero once built)
    pub bsp: u32,
}

impl Default for LightTable {
    fn default() -> Self {
        LightTable { slots: [0; 256], global: [0; 8], free: [u32::MAX; 8], dirty: 0, bsp: 0 }
    }
}

#[derive(Clone, Debug)]
pub struct C2d33a0 {
    pub p: LightParams,
    pub d: LightDerived,
    /// +0xe4: last seen *(+0x50)+0xa4 (slot 16)
    pub f_e4: i32,
    /// +0x118: light table slot (-1 = not registered)
    pub f_118: i32,
    /// +0x120: kind copy (+0xe0) taken by FUN_001a3968
    pub f_120: u32,
}

impl Default for C2d33a0 {
    fn default() -> Self {
        // FUN_001a3360 (slot 6, ported in gfx::light): +0xe4 / +0x118 = -1
        C2d33a0 {
            p: light::ee_sub_1a3360(),
            d: LightDerived { du: 0.0, ru: 0.0, color: [0.0; 3], k13c: 0.0, g: 0.0, vec: [0.0; 3], du_fall: 0.0, ru_fall: 0.0 },
            f_e4: -1,
            f_118: -1,
            f_120: 0,
        }
    }
}

/// slot 5: FUN_00113a50
pub fn ee_sub_113a50() -> u32 {
    0x2bab38
}

impl C2d33a0 {
    /// Load: the field setters 151..158 in stream order (each re-runs FUN_001a3968; only the last result
    /// survives, and slot 14 re-runs it anyway).
    pub fn feed(&mut self, props: &[Prop]) {
        light::apply_props(&mut self.p, props);
    }

    /// FUN_001a3968 (math in `gfx::light::ee_sub_1a3968`) + the +0x120 copy and FUN_001a3b38.
    pub fn ee_sub_1a3968(&mut self, obj: usize, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        let m = h.obj_mat(obj as u32 + 1, 0x60);
        let world = [[m[0][0], m[0][1], m[0][2]], [m[1][0], m[1][1], m[1][2]], [m[2][0], m[2][1], m[2][2]], [m[3][0], m[3][1], m[3][2]]];
        let mut d = light::ee_sub_1a3968(&self.p, &world, g.g_3e12c8 != 0);
        if self.p.kind > 1 {
            // other kinds keep +0x140 and re-scale the previous +0x110 / +0x114
            d.vec = self.d.vec;
            d.du_fall = self.d.du_fall * d.g;
            d.ru_fall = self.d.ru_fall * d.g;
        }
        self.d = d;
        self.f_120 = self.p.kind;
        self.ee_sub_1a3b38(g);
    }

    /// FUN_001a3b38
    pub fn ee_sub_1a3b38(&self, g: &mut ClassGlobals) {
        if !self.p.dynamic {
            g.g_3e11ec += 1;
        }
        if !self.p.excluded && self.p.kind != 0 {
            g.g_3e19e4 = 1;
        }
    }

    /// slot 10: FUN_001a3430
    pub fn ee_sub_1a3430(&mut self, obj: usize, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        h.base(0x1a5800, obj);
        self.ee_sub_1a3968(obj, g, h);
    }

    /// slot 11: FUN_001a34b0
    pub fn ee_sub_1a34b0(&mut self, obj: usize, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        h.base(0x1a5848, obj);
        self.ee_sub_1a3b38(g);
    }

    /// slot 14: FUN_001a3400
    pub fn ee_sub_1a3400(&mut self, obj: usize, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        h.base(0x1a5c48, obj);
        self.ee_sub_1a3968(obj, g, h);
    }

    /// slot 16: FUN_001a3460
    pub fn ee_sub_1a3460(&mut self, obj: usize, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        h.base(0x1a5e80, obj);
        let v = h.parent_a4(obj as u32 + 1); // +0x50 ? *(+0x50 + 0xa4) : 0
        if self.f_e4 < v {
            self.f_e4 = v;
            self.ee_sub_1a3968(obj, g, h);
        }
    }

    /// slot 21: FUN_001a34e0 -> FUN_001a3e88 (unregister)
    pub fn ee_sub_1a34e0(&mut self, obj: usize, t: &mut LightTable, h: &mut dyn ClassHost) {
        h.base(0x1a6150, obj);
        self.ee_sub_1a3e88(t);
    }

    /// slot 22: FUN_001a3518 -> FUN_001a3d00 (register)
    pub fn ee_sub_1a3518(&mut self, obj: usize, t: &mut LightTable, h: &mut dyn ClassHost) {
        h.base(0x1a6178, obj);
        self.ee_sub_1a3d00(obj, t, h);
    }

    /// slot 28: FUN_001a3550
    pub fn ee_sub_1a3550(&mut self, obj: usize, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        h.base(0x2312a0, obj);
        self.ee_sub_1a3968(obj, g, h);
    }

    /// slot 29: FUN_001a3580
    pub fn ee_sub_1a3580(&mut self, obj: usize, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        h.base(0x231300, obj);
        self.ee_sub_1a3968(obj, g, h);
    }

    /// FUN_001a3e88(g_lsm, obj). quirk kept: no check for +0x118 == -1 (bit 31 of word -1 / slot -1).
    pub fn ee_sub_1a3e88(&mut self, t: &mut LightTable) {
        let s = self.f_118;
        let w = ((if s < 0 { s + 0x1f } else { s }) >> 5) as isize;
        if (0..8).contains(&w) {
            t.free[w as usize] |= 1 << (s as u32 & 0x1f);
        }
        if (0..256).contains(&s) {
            t.slots[s as usize] = 0;
        }
        self.f_118 = -1;
        t.dirty = 1;
    }

    /// FUN_001a3d00(g_lsm, obj): take the lowest free slot when unregistered (nothing when the table is
    /// full), then unless the table is dirty or the light excluded: kind 0 -> global bit set; point lights
    /// (non-dynamic, BSP built) -> FUN_001a3b80 region insert; else global.
    pub fn ee_sub_1a3d00(&mut self, obj: usize, t: &mut LightTable, h: &mut dyn ClassHost) {
        if self.f_118 < 0 {
            let Some(k) = (0..8).find(|&k| t.free[k] != 0) else { return };
            let low = t.free[k] & t.free[k].wrapping_neg();
            t.free[k] &= !low;
            let s = (k as u32 * 32 + low.trailing_zeros()) as i32;
            t.slots[s as usize] = obj as u32 + 1;
            self.f_118 = s;
        }
        if t.dirty != 0 || self.p.excluded {
            return;
        }
        if self.p.kind != 0 && !self.p.dynamic && t.bsp != 0 {
            let pos = h.ee_sub_1a62e8(obj as u32 + 1);
            h.ee_sub_1a3b80(self.p.falloff[1], t.bsp, pos, self.f_118);
            return;
        }
        let s = self.f_118 as u32;
        t.global[(s >> 5) as usize] |= 1 << (s & 0x1f);
    }
}
