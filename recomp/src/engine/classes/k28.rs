//! WP-K28: argument blocks of the class code's level-script messages (sequence action 9 -> owner vtable
//! +0x1c (owner, 4, 0, 1, &rec[4], 0) -> FUN_001331f8 -> FUN_00200da0). The K27 queue
//! (`EngineHost::k27_script_msgs`, handle, msg, argument words) is delivered by `game::vm_state::send_messages`
//! with the CRefs built here.
use crate::engine::splice::value::Ref;

/// DAT_0027a1d8: per message id 0..0x1c, the index of its first argument kind in DAT_0027a10c (entry 0x1d =
/// end of the kinds, 51).
pub const MSG_KIND_INDEX: [u16; 30] = [0, 1, 2, 3, 4, 6, 7, 8, 11, 14, 15, 18, 21, 24, 25, 27, 29, 30, 33, 35, 36, 37, 38, 39, 41, 43, 45, 46, 47, 51];

/// DAT_0027a10c: argument kinds (field-kind words, FUN_001ffd98), 51 entries.
pub const MSG_KINDS: [u32; 51] = [
    0x2002, 0x2000, 0x2000, 0x2000, 0x6, 0x2002, 0x2002, 0x2038, 0x1010, 0x1010, 0x2088, 0x6, 0x6, 0x2018, 0x2030, 0x2030, 0x1011, 0x2030, 0x2030, 0x1011,
    0x2002, 0x5, 0x5, 0x2032, 0x2002, 0x2002, 0x2002, 0x2002, 0x2062, 0x2002, 0x104e, 0x2002, 0x2002, 0x1050, 0x2002, 0x2002, 0x2085, 0x2085, 0x2085, 0x0,
    0x2038, 0x6, 0x2077, 0x2038, 0x2034, 0x204f, 0x2010, 0x2002, 0x106b, 0x106b, 0x0,
];

/// FUN_00200da0(out, count, kinds of `msg`, values): one CRef per argument word (FUN_001ffd98 of the word
/// at the pointer with that kind): kind 0 bool, 1 float, 5..8 / 0xe..0x11 / 0x13..0x15 int; object-pointer
/// kinds (0x2000 bit) carry no object in the class code's words -> Nil (no caller passes one).
/// count = `words.len()`; None for a message id outside the table.
pub fn ee_sub_200da0(msg: u32, words: &[u32]) -> Option<Vec<Ref>> {
    // 0x00200da0
    if msg as usize + 1 >= MSG_KIND_INDEX.len() {
        return None;
    }
    let base = MSG_KIND_INDEX[msg as usize] as usize;
    let out = words
        .iter()
        .enumerate()
        .map(|(k, &w)| match MSG_KINDS.get(base + k).copied().unwrap_or(0x2000) {
            0 => Ref::Bool(w != 0),
            1 => Ref::F32(f32::from_bits(w)),
            5..=8 | 0xe..=0x11 | 0x13..=0x15 => Ref::S32(w as i32),
            _ => Ref::Nil,
        })
        .collect();
    Some(out)
}

impl super::plugin::EngineHost {
    /// FUN_00131290 kinds 7..0x10 for the ASEGA `h` (K26 dispatch): ctx = FUN_00131270(asega) (+0x18 target,
    /// else +0x14 the ASEG). 7 / 8: FUN_00210338(+0xc fade, world, on, +8 oid, ctx, owner) (classes/k22_seq.rs);
    /// 9 / 10: FUN_00131990(asega, on): on != +0x58 -> FUN_0020efc0(world) / FUN_0020f030(world, 3) (the pad
    /// lock counters); 0xd..0x10: FUN_00210408 / 002104a0(world, on, +8 oid, +0xc, +0x18, ctx): the ALO (oid
    /// != -1: FUN_0015e938(4, oid, ctx), else +0x18) of class 0x30 -> FUN_001f6978 / 69f8 / 6ac8 / 6b48.
    /// REPLACED: FUN_00131968 (the ASEGA is still in 0x3e1218) is true for every dispatched ASEGA.
    pub(super) fn k28_event(&mut self, h: u32, c: crate::engine::anim::events::EffectCall) {
        // 0x00131290 (cases 7..0x10)
        use crate::engine::anim::events::EffectCall as E;
        use super::ClassHost;
        let Some(a) = self.k25.asegas.get(&h) else { return };
        let (tgt, aseg) = (a.obj, a.aseg as u32 + 1);
        let ctx = if tgt == 0 { aseg } else { tgt };
        self.k28_events += 1;
        match c {
            E::E7 { on, a, f } => {
                super::k22_seq::ee_sub_210338(f, on, a as u32, ctx, self);
            }
            E::E9 { on } => {
                if self.k28_asega_58.get(&h).copied().unwrap_or(false) != on {
                    if on {
                        self.k27_world_20efc0(0);
                    } else {
                        self.k27_world_20f030(3);
                    }
                    self.k28_asega_58.insert(h, on);
                }
            }
            E::Ed { on, a, b } | E::Ef { on, a, b } => {
                let o = if a != -1 { self.ee_sub_15e938(4, a, ctx) } else { tgt };
                if o != 0 && self.ee_sub_132b10(o, 0x30) != 0 {
                    let i = o as usize - 1;
                    match (matches!(c, E::Ed { .. }), on) {
                        (true, true) => self.ee_sub_1f6978(i, b),
                        (true, false) => self.ee_sub_1f69f8(i, b),
                        (false, true) => self.ee_sub_1f6ac8(i, b),
                        (false, false) => self.ee_sub_1f6b48(i, b),
                    }
                }
            }
            // K30: 0x11 FUN_00210538(+0x10 scale, world, +8 oid, +0xc source oid, +0x10 oid, ctx): the request
            // {flags, source, scale, .., +0x30} to the object's vtable +0x19c (slot-51 emit, queued for k28_spawn)
            E::E11(w, f) => {
                let o = self.ee_sub_15e938(4, w[0], ctx);
                if o == 0 {
                    return;
                }
                let mut r = [0u32; 15];
                if w[1] != -1 {
                    let s = self.ee_sub_15e938(4, w[1], ctx);
                    if s != 0 {
                        r[1] = s;
                        r[0] |= 1;
                    }
                }
                if r[0] & 1 == 0 {
                    r[0] |= 1;
                    r[1] = o;
                }
                if w[2] != -1 {
                    let s = self.ee_sub_15e938(4, w[2], ctx);
                    if s != 0 {
                        r[12] = s;
                        r[0] |= 0x20;
                    }
                }
                if f != 1.0 {
                    r[0] |= 0x18;
                    r[2] = f.to_bits();
                }
                self.k30_reqs.push((o, r));
            }
            // K30: 0x12 FUN_001618f8(+0x1c, world): +0x476c = now + t; 0x13 FUN_00161958(world): +0x476c = 0.
            // UNPORTED: the +0x3bc4 HUD timer object calls (vtable +0x18c(-1) with +0x8c = 0 / +0x194)
            E::E12 { t } => self.k30_476c = self.k30_t + t,
            E::E13 => self.k30_476c = 0.0,
            _ => self.k28_events -= 1,
        }
    }
}
