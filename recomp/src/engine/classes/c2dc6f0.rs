//! C2dc6f0 (world class 26): level warp volume (parent C2d1b60 volume). Load FUN_002282b0 is ported in
//! world (`LoadData::L2282b0`: +0x144 first destination block, +0x148/+0x14c list).
//! Object handles in hooks are index + 1 (0 = null).
use super::{ClassGlobals, ClassHost};
use crate::engine::world::props::{Prop, PropValue};

/// WP-EP1d: class-131 FUN_0015cf30 mode 10 intro start of a never-played episode:
/// `o = FUN_0015e938(0x3dd490, 5, 0x2be2a8[ep], 0)`, FUN_00228358(o, 3), FUN_002283d0(o). The UI side
/// (`ui::episode::episode_event`) posts the oid here; the class set runs it at its next frame
/// (`ClassSet::frame_more`), the warp then plays its movies and posts its own descriptor (state 4).
static WARP_STARTS: std::sync::Mutex<Vec<i32>> = std::sync::Mutex::new(Vec::new());

/// WP-EP1d: queue the intro-warp start of class-26 object `oid` (see `WARP_STARTS`).
pub fn post_warp_start(oid: i32) {
    WARP_STARTS.lock().unwrap_or_else(|e| e.into_inner()).push(oid);
}

/// WP-EP1d: queued intro-warp starts (drained).
pub fn take_warp_starts() -> Vec<i32> {
    std::mem::take(&mut *WARP_STARTS.lock().unwrap_or_else(|e| e.into_inner()))
}

#[derive(Clone, Debug, Default)]
pub struct C2dc6f0 {
    /// +0x130 / +0x134 / +0x138 (fields 782-784)
    pub f_130: i32,
    pub f_134: i32,
    pub f_138: i32,
    /// +0x13c state (field 787 via FUN_00228358), +0x140 state time
    pub f_13c: i32,
    pub f_140: f32,
    /// +0x144 destination block present (FUN_00228250 non-null)
    pub f_144: bool,
    /// +0x148 / +0x14c: destination block list length
    pub f_148: i32,
    /// +0x150 / +0x154 (fields 779 / 780)
    pub f_150: i32,
    pub f_154: i32,
    /// +0x158 warp time, +0x15c (781) / +0x160 (785)
    pub f_158: f32,
    /// +0x164 count / +0x168 inline array: task pairs {oid, state} set by FUN_002284e0
    /// (Splice method thunk 0x10b850). Max 4 entries.
    pub task_entries: Vec<[i32; 2]>,
    pub f_15c: f32,
    pub f_160: f32,
    /// +0x40 bit 1
    pub f_40: u32,
}

/// slot 5: FUN_00113bb0
pub fn ee_sub_113bb0() -> u32 {
    0x2bacf0
}

fn word(b: &[u8]) -> u32 {
    match b.len() {
        0 => 0,
        1..=3 => b[0] as u32,
        _ => u32::from_le_bytes([b[0], b[1], b[2], b[3]]),
    }
}

impl C2dc6f0 {
    /// Load: destination blocks + field table 0x2ac880 entries 779-785, 787.
    pub fn feed(&mut self, first: bool, list: usize, props: &[Prop], g: &mut ClassGlobals, h: &mut dyn ClassHost, obj: usize) {
        self.f_144 = first;
        self.f_148 = list as i32;
        for p in props {
            // Field 788 (kind 0x12, flags 0x400, thunk 0x10b850): FUN_0013e818's array path calls the
            // method once per load record -> FUN_002284e0(obj, oid, states) -> FUN_00161f10 (max 4 pairs
            // at +0x164/+0x168). Kinds {0x6 oid, 0x1050 state}: FUN_00201360 hands both over as words.
            if let (788, PropValue::Array(a)) = (p.field, &p.value) {
                if self.task_entries.len() < 4 {
                    use crate::engine::game::job_flow::item_i32;
                    self.task_entries.push([item_i32(a.first()), item_i32(a.get(1))]);
                }
                continue;
            }
            let PropValue::Scalar(b) = &p.value else { continue };
            let v = word(b);
            match p.field {
                779 => self.f_150 = v as i32,
                780 => self.f_154 = v as i32,
                781 => self.f_15c = f32::from_bits(v),
                782 => self.f_130 = v as i32,
                783 => self.f_134 = v as i32,
                784 => self.f_138 = v as i32,
                785 => self.f_160 = f32::from_bits(v),
                787 => self.ee_sub_228358(obj, v as i32, g, h),
                _ => {}
            }
        }
    }

    /// slot 6: FUN_002281e8
    pub fn ee_sub_2281e8(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x231100, obj);
        self.f_150 = 0x189;
        self.f_154 = -1;
        h.ee_sub_1a6ca8(obj, 0);
        self.f_40 |= 2;
    }

    /// FUN_00228358: set state (4 = warping: FUN_001a6ca8(obj, 1), FUN_0020efc0(3), DAT_003e1114 = obj)
    pub fn ee_sub_228358(&mut self, obj: usize, s: i32, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        if s == self.f_13c {
            return;
        }
        if s == 4 {
            h.ee_sub_1a6ca8(obj, 1);
            h.ee_sub_20efc0(3);
            g.g_3e1114 = obj as u32 + 1;
            self.f_13c = 4;
        } else {
            self.f_13c = s;
        }
        self.f_140 = g.t;
    }

    /// FUN_002283d0: in state 3 either start the warp (+0x134 == 0) or drop to 0 and hand over to the
    /// FUN_002280d0 object (returned; the caller runs FUN_002283d0 on it).
    pub fn ee_sub_2283d0(&mut self, obj: usize, g: &mut ClassGlobals, h: &mut dyn ClassHost) -> Option<u32> {
        if self.f_13c != 3 {
            return None;
        }
        if self.f_134 == 0 {
            h.ee_sub_161f48_warp(obj, &self.task_entries); // FUN_00161f48(0x2deb30, +0x164, +0x168, obj)
            h.ee_sub_14e4c8(0x2de390); // FUN_0014e4c8(0x2ddf40, 2, -1, 0x2de390, 0, 0)
            h.ee_sub_14e4c8(0x2de460);
            self.ee_sub_228358(obj, 4, g, h);
            self.f_158 = g.t + self.f_160;
            h.vcall_1c(obj, 5, 0);
            None
        } else {
            self.ee_sub_228358(obj, 0, g, h);
            match h.ee_sub_2280d0() {
                0 => None,
                o => Some(o),
            }
        }
    }

    /// slot 15: FUN_00228538 (returns a FUN_002283d0 hand-over target, see there)
    pub fn ee_sub_228538(&mut self, obj: usize, g: &mut ClassGlobals, h: &mut dyn ClassHost) -> Option<u32> {
        h.ee_sub_1a5e28(obj);
        match self.f_13c {
            0 | 1 | 3 => {
                let p = h.ee_sub_1c4840();
                if p == 0 {
                    return None;
                }
                let wp = h.ee_sub_1a62e8(p);
                let lp = h.ee_sub_1a6f00(obj, wp);
                let outside = !h.ee_sub_118fb0(obj, lp); // FUN_00118fb0(+0xec, 0x7fffffff, local pos) == 0
                match self.f_13c {
                    1 => {
                        if !outside {
                            self.ee_sub_228358(obj, 0, g, h);
                        } else if self.f_15c < g.t - self.f_140 {
                            self.ee_sub_228358(obj, 3, g, h);
                        }
                    }
                    3 => {
                        if !outside {
                            return self.ee_sub_2283d0(obj, g, h);
                        }
                    }
                    _ => {
                        if outside {
                            self.ee_sub_228358(obj, 1, g, h);
                        }
                    }
                }
            }
            4 => {
                if 0.0 < self.f_158 {
                    if g.t < self.f_158 {
                        return None;
                    }
                    for k in 0..self.f_148 {
                        h.ee_sub_1b3a48(obj, k); // FUN_001b3a48(0x2f66b0, +0x14c[k])
                    }
                    let fl = if self.f_138 == 0 { 0 } else { 0x20 };
                    h.ee_sub_22b490(obj, self.f_150, fl, self.f_154); // FUN_0022b490(0x3dd490, +0x144, +0x150, -1, fl, +0x154)
                    self.f_158 = 0.0;
                }
            }
            _ => {}
        }
        None
    }
}
