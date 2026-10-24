//! C2d8c48 (world class 114): record table +0x50/+0x54 (0x50-byte records) and an active node list +0x5c.
//! Object handles in hooks are index + 1 (0 = null).
use super::{ClassGlobals, ClassHost};
use crate::engine::world::props::{Prop, PropValue};

#[derive(Clone, Debug, Default)]
pub struct C2d8c48 {
    /// +0x40 word (bit 1 enable, bit 8 selects the DAT_002ddf10 clock)
    pub f_40: u32,
    /// +0x50 count / +0x54 records (0x50 bytes each) / +0x58 capacity (-1 = 0x200)
    pub f_54: Vec<[u32; 20]>,
    pub f_58: i32,
    /// +0x5c list (link +0x64 = 0xe4): node handles
    pub f_5c: Vec<u32>,
    pub f_6c: i32,
    /// +0x70 elapsed, +0x74 running, +0x78, +0x7c, +0x80
    pub f_70: f32,
    pub f_74: i32,
    pub f_78: i32,
    pub f_7c: i32,
    pub f_80: i32,
    /// K22: node states (classes/k22_seq.rs), the interpreter's node counter, the +0x84 signal list
    pub k22_nodes: Vec<super::k22_seq::Node>,
    pub k22_next: u32,
    pub k22_sig: Vec<i32>,
    /// K27: guard routine actions 0x55 / 0x56 (FUN_00171cf8 / FUN_00171d58), applied by `ClassSet` on the
    /// guards after this object's update, and the guard (handle) -> running routine (handle) view the
    /// action-0x55 wait (FUN_00171da8) reads (refreshed by `ClassSet` before the update)
    pub k27_ops: Vec<super::k22_seq::GuardOp>,
    pub k27_running: Vec<(u32, u32)>,
    /// ANIM-ALL R35: object (handle) -> binocucom HUD state (+0xf28) for the action-0x3b / 0x3c waits
    /// (refreshed by `ClassSet` before the update)
    pub binoc_huds: Vec<(u32, i32)>,
    /// K27: action-0x12 world states released by FUN_001e5628 (flags; FUN_0020f030 + FUN_0020f0f0 by the set)
    pub k27_world: Vec<u32>,
}

/// slot 5: FUN_00114130
pub fn ee_sub_114130() -> u32 {
    0x2bb3d0
}

impl C2d8c48 {
    /// slot 6: FUN_001dfad8
    pub fn ee_sub_1dfad8(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1a5270, obj);
        self.f_58 = -1;
        self.f_5c.clear(); // FUN_0011a1f8(+0x5c, 0xe4)
    }

    /// FUN_001e01f8(obj, a, b): append a zeroed record {a, b, ..}; None = table full (DAT_003da740 dummy)
    pub fn ee_sub_1e01f8(&mut self, a: u32, b: u32) -> Option<usize> {
        let cap = if self.f_58 < 0 { 0x200 } else { self.f_58 };
        let n = self.f_54.len() as i32;
        if n < cap {
            let mut r = [0u32; 20];
            r[0] = a;
            r[1] = b;
            self.f_54.push(r);
            Some(n as usize)
        } else {
            None
        }
    }

    /// slot 13 tail (FUN_001dfb10, already parsed by world::loads): FUN_001e0290(obj, -1), +0x7c = 1,
    /// props (field 622 +0x58 via setter 0x1e01c8), then capacity = max(+0x58, +0x50) + 1.
    pub fn feed(&mut self, props: &[Prop]) {
        if let Some(k) = self.ee_sub_1e01f8(0, 1) {
            self.f_54[k][4] = u32::MAX;
        }
        self.f_7c = 1;
        for p in props {
            if let (622, PropValue::Scalar(b)) = (p.field, &p.value) {
                if b.len() >= 4 {
                    self.f_58 = i32::from_le_bytes([b[0], b[1], b[2], b[3]]);
                }
            }
            // K22: the record fields 625..708
            super::k22_seq::add_record(self, p);
        }
        let n = self.f_54.len() as i32;
        self.f_58 = self.f_58.max(n) + 1;
    }

    /// slot 40: FUN_001dfbf8
    pub fn ee_sub_1dfbf8(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1a6730, obj);
        if self.f_80 != 0 {
            // FUN_001ae238: shrink the record block to +0x58 records
            self.f_54.truncate(self.f_58.max(0) as usize);
            self.f_54.shrink_to_fit();
        }
    }

    /// slot 15: FUN_001dfc50
    pub fn ee_sub_1dfc50(&mut self, obj: usize, g: &ClassGlobals, h: &mut dyn ClassHost) {
        h.ee_sub_1a5e28(obj);
        if self.f_74 == 0 {
            return;
        }
        self.f_70 += if self.f_40 & 0x100 == 0 { g.dt } else { g.g_2ddf10 };
        // walk with the DAT_002dd2fc safe-iterator frame: a node may unlink itself
        let nodes = self.f_5c.clone();
        for n in nodes {
            if self.f_5c.contains(&n) {
                super::k22_seq::ee_sub_1e1c40(self, obj, n, g.t, h);
            }
        }
        if self.f_5c.is_empty() {
            self.ee_sub_1dfeb8(obj, h);
        }
    }

    /// FUN_001dfeb8: stop
    pub fn ee_sub_1dfeb8(&mut self, obj: usize, h: &mut dyn ClassHost) {
        if self.f_74 == 0 {
            return;
        }
        self.f_6c = 0;
        self.f_70 = 0.0;
        self.f_74 = 0;
        for &n in &self.f_5c.clone() {
            super::k22_seq::ee_sub_1e5628(self, n);
        }
        // FUN_001e0120: +0x84 = 0
        self.k22_sig.clear();
        h.ee_sub_1e0120(obj as u32 + 1);
        if self.f_78 != 0 {
            h.ee_sub_1a6ca8(obj, 2);
            self.f_40 &= !2;
        }
        h.vcall_1c(obj, 6, 0); // (vtable +0x1c)(obj + adj, 6, 0, 0, 0, 0)
    }
}
