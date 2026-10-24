//! WP-S1: script method rows ported by S1 (census in tests/engine_s1.rs). Each row runs the engine
//! function on the lent class runtime; an object without a ported instance is recorded (`Recorded`),
//! like the class callees.
//!  - 0x0010fc80 (field 1368, inline `(vtable +0x29c)(obj)` = slot 82 reset): class 73 FUN_00181238
//!    (`k15::C2d1080::ee_sub_181238`), class 76 FUN_001819e0 (`k14::C2d0b30::ee_sub_1819e0`); the
//!    FUN_0017f6b0 cancel = FUN_00149b58(0x3dd490, 1, obj, ..) drops the object's queued slot-81 call.
//!    Classes 72 (FUN_0017fcb0), 74 (FUN_0017f6b0 alone) and 75 (FUN_00181598) have no ported instance.
//!  - 0x0010fe60 (field 1375) -> FUN_00181308(state, oid, flags) on class 73.
//!  - 0x0010c6a8 (field 979) -> FUN_0015e1d8 on class 116 (`C2cf058`).
//!  - 0x0010d4e0 / 0x0010d558 (fields 1066 / 1067) -> FUN_001becf8 / FUN_001bed18 on class 45
//!    (`ClassSet::k7_highlight`).
//!  - 0x001044b0 (field 364) -> FUN_0020af88(a, b) on class 120 (`C2da840`).
//!  - 0x00110a28 (field 1444) -> FUN_001c8af0(player) on class 78 (`k9::Vault`).
//!  - 0x00105200 (field 428) -> FUN_001490c0 = FUN_001487a0(obj + 0x150) on class 83 (`k15_83::Toggle`).

use super::{handle_of, ClassLink, Recorded};
use crate::engine::classes::{ClassHost, ClassSet, Inst};
use crate::engine::game::methods::MArg;
use crate::engine::splice::value::Ref;

/// (thunk, engine callee; 0 = inline thunk) rows executed here.
pub const PORTED: &[(u32, u32)] = &[
    (0x0010_fc80, 0),
    (0x0010_fe60, 0x0018_1308),
    (0x0010_c6a8, 0x0015_e1d8),
    (0x0010_d4e0, 0x001b_ecf8),
    (0x0010_d558, 0x001b_ed18),
    (0x0010_44b0, 0x0020_af88),
    (0x0011_0a28, 0x001c_8af0),
    (0x0010_5200, 0x0014_90c0),
    (0x0011_0478, 0x001d_4e30),
];

/// Row (thunk, engine callee) executed by S1 code.
pub fn ported(thunk: u32, callee: u32) -> bool {
    PORTED.contains(&(thunk, callee))
}

fn int(a: &[MArg], i: usize) -> i32 {
    match a.get(i) {
        Some(MArg::I32(n)) => *n,
        Some(MArg::Word(w)) => *w as i32,
        Some(MArg::F32(f)) => *f as i32,
        _ => 0,
    }
}

impl ClassLink {
    /// Run an S1 row. False when the row is not S1's or the class runtime is not lent.
    pub(super) fn s1_call(&mut self, thunk: u32, callee: u32, obj: u32, raw: &[Ref], args: &[MArg]) -> bool {
        if !ported(thunk, callee) {
            return false;
        }
        let Some(set) = self.set.as_mut() else { return false };
        if !s1_run(set, &mut self.host, thunk, obj as usize, raw, args) {
            self.recorded.push(Recorded { thunk, callee, obj, args: args.to_vec() });
        }
        true
    }
}

/// False when the object has no ported instance of the row's class.
fn s1_run(set: &mut ClassSet, h: &mut dyn ClassHost, thunk: u32, i: usize, raw: &[Ref], a: &[MArg]) -> bool {
    let (t, dt) = (set.g.t, set.g.dt);
    match thunk {
        // 0x0010fc80: (vtable +0x29c)(obj + (vtable +0x298))
        0x0010_fc80 => {
            // FUN_0017f6b0's cancel: FUN_00149b58(0x3dd490, 1, obj, DAT_002a5638, 0)
            if let Some(c) = set.k15_get(i) {
                // class 73 FUN_00181238
                if c.ee_sub_181238(i, &mut crate::engine::classes::k15::Cx { t, dt, h }) {
                    set.k15_deferred.retain(|x| x.0 != i);
                }
                return true;
            }
            let ClassSet { k14, k14_deferred, rng, .. } = set;
            let Some(c) = k14.iter_mut().find(|x| x.0 == i).and_then(|x| if let Inst::K14(c) = &mut x.1 { Some(&mut **c) } else { None }) else { return false };
            // class 76 FUN_001819e0
            if c.ee_sub_1819e0(i, &mut crate::engine::classes::k14::Cx { t, dt, rng, h }) {
                k14_deferred.retain(|x| x.0 != i);
            }
        }
        // 0x0010fe60: FUN_00181308(obj, state (kind 0x105f), oid (kind 6), flags (kind 0x100b))
        0x0010_fe60 => {
            let Some(c) = set.k15_get(i) else { return false };
            c.ee_sub_181308(int(a, 0) as u32, int(a, 1), int(a, 2) as u32);
        }
        // 0x0010c6a8: FUN_0015e1d8(obj)
        0x0010_c6a8 => {
            let ClassSet { insts, more, g, .. } = set;
            let Some(c) = insts.iter_mut().chain(more.iter_mut()).find(|x| x.0 == i).and_then(|x| if let Inst::C2cf058(c) = &mut x.1 { Some(c) } else { None }) else { return false };
            c.ee_sub_15e1d8(i, g, h);
        }
        // 0x0010d4e0 / 0x0010d558: FUN_001becf8(obj) / FUN_001bed18(obj)
        0x0010_d4e0 | 0x0010_d558 => {
            let on = thunk == 0x0010_d4e0;
            if !set.k7.iter().any(|x| x.0 == i) {
                return false;
            }
            set.k7_highlight(i, on, h);
        }
        // 0x001044b0: FUN_0020af88(obj, a (kind 6), b (kind 0x100b))
        0x0010_44b0 => {
            let ClassSet { insts, more, .. } = set;
            let Some(c) = insts.iter_mut().chain(more.iter_mut()).find(|x| x.0 == i).and_then(|x| if let Inst::C2da840(c) = &mut x.1 { Some(c) } else { None }) else { return false };
            c.ee_sub_20af88(i, int(a, 0) as u32, int(a, 1) as u32, h);
        }
        // 0x00110a28: FUN_001c8af0(obj, player (kind 0x2034))
        0x0011_0a28 => {
            let ClassSet { k9, g, .. } = set;
            let Some(c) = k9.iter_mut().find(|x| x.0 == i).and_then(|x| if let Inst::K9(c) = &mut x.1 { Some(&mut **c) } else { None }) else { return false };
            c.ee_sub_1c8af0(i, handle_of(raw.first()), g, h);
        }
        // 0x00105200: FUN_001490c0(obj) = FUN_001487a0(obj + 0x150)
        0x0010_5200 => {
            let Some(c) = set.k15_83_get(i) else { return false };
            c.tg.ee_sub_1487a0(t, h);
        }
        // WIRE-E: 0x00110478: FUN_001d4e30(part) on the class-77 truck `i`'s part with key a0 (vm_state
        // resolves the class-153 part handle to (truck, key)); FUN_001d3fa8 = the part by key
        0x0011_0478 => {
            let Some(c) = set.k20_77(i) else { return false };
            let Some(k) = c.ee_sub_1d3fa8(int(a, 0)).filter(|&k| k < c.parts.len()) else { return false };
            c.ee_sub_1d4e30(k);
        }
        _ => return false,
    }
    true
}
