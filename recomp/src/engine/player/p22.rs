//! WP-P22: Bentley / Murray own ground movement (class slot 15 + slot 69 FUN_00204108 on the class's PO fields).
//!
//! 1:1: FUN_0020bb58 / FUN_0020bad8 (blend weights {1-u, u} of the two entries, +0x190 = -rate * sum w_i *
//! v_i * len_i / blend length), FUN_0020b868's entry root velocity v_i = (root(L) - root(0)) / L of the
//! entry ASEG (root channel, `ee_sub_20b868_entry`), FUN_00204108's angular part (+0x1a4 == 0: servo
//! FUN_001f59a0 of the angular xf toward the +0x1b0 rows with {+0x1f4*+0x200, +0x1f8*+0x200, +0x1fc};
//! +0x1a4 == 1: alpha += (+0x1e0 - w) / dt), FUN_00203fc0 (+0x180 = clamp(1.5 - |wrap(+0x1a8 - yaw)|, 0, 1)).
//! The entry tables (`LOCO_55` / `LOCO_54`) are the FUN_001a64e8 rows of DAT_002bc390 / DAT_002c0270.
//! REPLACED: the linear part of 0x204108 is the slot-56 contact motor FUN_002044b0 (contact velocity
//! += tangent(R(+0x1b0) . +0x190) * +0x180); the driver has no tangential contact rows, so the body's
//! horizontal velocity is driven to -R . +0x190 * +0x180 in one step while it touches the ground (no force
//! in the air, as the motor only acts through a contact). The blend length is sum w_i len_i and the blend
//! rate (+0x40 of the blend slot, FUN_0020bb48) is 1 (the class anim slots are not driven).
//! WP-P24: the linear part now runs through the E8 contact motor (`po_motor` -> physics::motor::po_step in
//! `Jt::update`: 0x204ac8 kind-0xe record + 0x2044b0 + 0x1f9410 with friction rows); `ee_sub_204108`'s
//! linear result is no longer used by `Jt::forces` (kept for reference / tests).
#![allow(non_snake_case)]

use super::p5_states::{FUN_001f59a0, Acc};
use crate::engine::physics::driver::AloPhys;
use crate::engine::physics::geom::V3;
use crate::engine::world::World;

/// (blend block offset, entry 0 ASEG oid (+0x14), entry 1 ASEG oid (+0x54)) of Bentley (DAT_002bc390).
pub const LOCO_55: [(u32, i32, i32); 3] = [(0x1320, 0x1a0, 0x19e), (0x1540, 0x20f, 0x211), (0x1600, 0x1e9, 0x1e8)];
/// The same for Murray (DAT_002c0270).
pub const LOCO_54: [(u32, i32, i32); 4] = [(0x1320, 0x1a0, 0x19e), (0x1460, 0x1e2, 0x1e3), (0x1540, 0x1e9, 0x1e8), (0x1630, 0x1af, 0x1ae)];

/// One blend entry: root velocity (+0x10 of the entry) and length (+0x24).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LocoEntry {
    pub v: V3,
    pub len: f32,
}

/// FUN_0020b868 per entry: the entry ASEG's root velocity (root(L) - root(0)) / L (None: no ASEG).
pub fn ee_sub_20b868_entry(w: &World, obj: usize, oid: i32) -> Option<LocoEntry> {
    let o = crate::engine::playerdraw::ee_sub_15e938_below(w, obj, oid)?;
    let a = crate::engine::playerdraw::aseg_of(w, o)?;
    let len = a.length;
    let ch = a.channels.iter().find(|c| c.flag && c.kind == 1).and_then(|c| c.vec.as_ref());
    let v = match ch {
        Some(c) if len > 0.0 => match (c.eval(0.0, 1.0), c.eval(len, 1.0)) {
            (Some(p0), Some(p1)) => [0, 1, 2].map(|i| (p1[i] - p0[i]) / len),
            _ => [0.0; 3],
        },
        _ => [0.0; 3],
    };
    Some(LocoEntry { v, len })
}

/// FUN_0020b868 of every blend block of the po's class: (block, [entry 0, entry 1]).
pub fn loco_table(w: &World, obj: usize, class: u16) -> Vec<(u32, [Option<LocoEntry>; 2])> {
    let rows: &[(u32, i32, i32)] = match class {
        super::p20::CLASS_BENTLEY => &LOCO_55,
        super::p20::CLASS_MURRAY => &LOCO_54,
        _ => &[],
    };
    rows.iter().map(|&(b, o0, o1)| (b, [ee_sub_20b868_entry(w, obj, o0), ee_sub_20b868_entry(w, obj, o1)])).collect()
}

/// FUN_0020bb58(u, blk) -> +0x190: weights {1-u, u} (entry 1 absent: {1, 0}), FUN_0020bad8 times -rate.
pub fn ee_sub_20bb58(e: &[Option<LocoEntry>; 2], u: f32, rate: f32) -> V3 {
    let w = if e[1].is_none() { [1.0, 0.0] } else { [1.0 - u, u] };
    let ent = e.map(|x| x.unwrap_or_default());
    let blen = w[0] * ent[0].len + w[1] * ent[1].len;
    let mut s = [0.0; 3];
    if blen > 0.0 {
        for i in 0..2 {
            let k = w[i] * (ent[i].len / blen);
            for j in 0..3 {
                s[j] += ent[i].v[j] * k;
            }
        }
    }
    s.map(|x| -x * rate)
}

/// The class PO fields 0x204108 reads (copied from the class memory each frame).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PoMove {
    /// slot 69 calls FUN_00204108 this state (else FUN_001f9b20 only: gravity)
    pub on: bool,
    /// +0x190 local move velocity (in the +0x1b0 frame)
    pub v190: V3,
    /// +0x1a8 target yaw
    pub f_1a8: f32,
    /// +0x1b0 / +0x1c0 / +0x1d0 target rows
    pub rows_1b0: [V3; 3],
    /// +0x1a4 angular mode
    pub f_1a4: i32,
    /// +0x1e0 target angular velocity (mode 1)
    pub w_1e0: V3,
    /// +0x1f4 / +0x1f8 / +0x1fc / +0x200
    pub mv: [f32; 4],
}

impl PoMove {
    /// The PO fields of a class memory (`m.f` / `m.v3` / `m.i` readers).
    pub fn from_mem(on: bool, f: impl Fn(u32) -> f32, i: impl Fn(u32) -> i32) -> PoMove {
        let v3 = |o: u32| [f(o), f(o + 4), f(o + 8)];
        PoMove {
            on,
            v190: v3(0x190),
            f_1a8: f(0x1a8),
            rows_1b0: [v3(0x1b0), v3(0x1c0), v3(0x1d0)],
            f_1a4: i(0x1a4),
            w_1e0: v3(0x1e0),
            mv: [f(0x1f4), f(0x1f8), f(0x1fc), f(0x200)],
        }
    }
}

/// Body yaw (atan2 of the x row).
pub fn body_yaw(b: &AloPhys) -> f32 {
    b.rot[0][1].atan2(b.rot[0][0])
}

/// FUN_00203fc0 (+0x1a4 == 0): +0x180 = clamp(1.5 - |wrap(+0x1a8 - yaw)|, 0, 1).
pub fn ee_sub_203fc0(f_1a8: f32, yaw: f32) -> f32 {
    let a = crate::engine::camera::math::FUN_0011cf60(f_1a8 - yaw).abs();
    (1.5 - a).clamp(0.0, 1.0)
}

/// FUN_00204108 of a non-JT PO (see the module doc): angular servo 1:1, linear contact motor REPLACED.
/// WP-P24: `Jt::forces` keeps only the angular part; the linear drive is the E8 motor (`po_motor`).
pub fn ee_sub_204108(b: &AloPhys, m: &PoMove, grounded: bool, dt: f32) -> Acc {
    let mut acc = Acc::default();
    if !m.on || dt <= 0.0 {
        return acc;
    }
    if m.f_1a4 == 0 {
        let p = [m.mv[0] * m.mv[3], m.mv[1] * m.mv[3], m.mv[2]];
        if p[2] != 0.0 && p[0] > 0.0 {
            acc.ang = FUN_001f59a0(&b.rot, b.w, [0.0; 3], &m.rows_1b0, p, dt);
        }
    } else if m.f_1a4 == 1 {
        acc.ang = [0, 1, 2].map(|i| (m.w_1e0[i] - b.w[i]) / dt);
    }
    if grounded {
        let k = ee_sub_203fc0(m.f_1a8, body_yaw(b));
        let r = &m.rows_1b0;
        let wv = [0, 1, 2].map(|j| m.v190[0] * r[0][j] + m.v190[1] * r[1][j] + m.v190[2] * r[2][j]);
        let vt = [-wv[0] * k, -wv[1] * k];
        acc.lin = [(vt[0] - b.v[0]) / dt, (vt[1] - b.v[1]) / dt, 0.0];
    }
    acc
}

/// WP-P24: the 0x14c6d8 probe segment of the PO motor (P0 + up * 50 .. P0 - up * 60, as E8's real-level test).
pub const PO_PROBE: (f32, f32) = (50.0, 60.0);

/// WP-P24: the E8 `PoMotor` of the class PO fields: +0x1b0 rows, +0x190, +0x180 = FUN_00203fc0 (written by
/// 0x204108; 0 while the state does not call it: REPLACED, the original keeps the last value), flags bit 0 on.
/// +0x210 / +0x248 / mass: E8 defaults (0 / 0 / 1; P0 is the JT sphere bottom).
pub fn po_motor(b: &AloPhys, m: &PoMove) -> crate::engine::physics::motor::PoMotor {
    crate::engine::physics::motor::PoMotor {
        rows_1b0: m.rows_1b0,
        v190: m.v190,
        k180: if m.on { ee_sub_203fc0(m.f_1a8, body_yaw(b)) } else { 0.0 },
        ..Default::default()
    }
}

/// The PO fields of the controlled class object `o` (Bentley: slot 69 FUN_00135940 calls 0x204108 unless state
/// 0xd; Murray FUN_001b67f8 unless state 0x14), with +0x190 from this frame's last FUN_0020bb58(u, blk) call
/// (`bb58`: (obj, blk, u)) when there was one.
pub fn po_move_of(set: &mut crate::engine::classes::ClassSet, o: usize, class: u16, loco: &[(u32, [Option<LocoEntry>; 2])], bb58: &[(usize, u32, f32)]) -> Option<PoMove> {
    let mut m = match class {
        super::p20::CLASS_BENTLEY => {
            let b = set.k16_get(o)?;
            PoMove::from_mem(b.state != 0xd, |x| b.m.f(x), |x| b.m.i(x))
        }
        super::p20::CLASS_MURRAY => {
            let b = &set.k18_murray(o)?.b;
            PoMove::from_mem(b.state != 0x14, |x| b.m.f(x), |x| b.m.i(x))
        }
        _ => return None,
    };
    if let Some(&(_, blk, u)) = bb58.iter().rev().find(|r| r.0 == o) {
        if let Some((_, e)) = loco.iter().find(|r| r.0 == blk) {
            m.v190 = ee_sub_20bb58(e, u, 1.0);
        }
    }
    Some(m)
}

/// WP-P22 player API for mid-mission character switches (slot 78 callers FUN_002206c8 / FUN_001c8730, K22
/// sequence action 7): make the object `o` the controlled character. Bentley / Murray: slot 78(1) through
/// `ClassSet::k18_set_active(Some(o))`; Sly (class 53) or None: the controlled Bentley / Murray's slot 78(0)
/// (`k18_set_active(None)`), Sly's JT takes over. The player layer (`follow_active`) picks the change up.
pub fn set_active_character(w: &World, set: &mut crate::engine::classes::ClassSet, h: &mut dyn crate::engine::classes::ClassHost, o: Option<usize>) {
    match o.map(|o| (o, w.objects[o].class)) {
        Some((o, super::p20::CLASS_BENTLEY | super::p20::CLASS_MURRAY)) => set.k18_set_active(Some(o), h),
        _ => set.k18_set_active(None, h),
    }
}

/// Key of Sly's JT in `PoCurrent::bodies`.
pub const BODY_SLY: usize = usize::MAX;

/// Follow `ClassSet::k18_active` after a switch: the body of the character left behind is stored, the new
/// character's stored body (if it was controlled before) is restored into the JT (REPLACED: the original keeps
/// one body per character; here the JT is the single controlled body, see p20). Returns true on a change.
pub fn follow_active(w: &World, active: Option<usize>, cur: &mut super::p20::PoCurrent, jt: &mut super::Jt) -> bool {
    if active == cur.obj {
        return false;
    }
    let old = cur.obj.unwrap_or(BODY_SLY);
    cur.bodies.retain(|b| b.0 != old);
    cur.bodies.push((old, jt.phys.pos, jt.phys.rot));
    let new = active.unwrap_or(BODY_SLY);
    if let Some(&(_, p, r)) = cur.bodies.iter().find(|b| b.0 == new) {
        jt.phys.pos = p;
        jt.phys.rot = r;
        jt.phys.v = [0.0; 3];
        jt.phys.w = [0.0; 3];
    }
    match active {
        Some(o) => {
            cur.obj = Some(o);
            cur.class = w.objects[o].class;
            cur.loco = loco_table(w, o, cur.class);
        }
        None => {
            cur.obj = None;
            cur.class = 0;
            cur.loco.clear();
            // Sly's JT: identity body rows (its facing is the JT yaw)
            jt.phys.rot = crate::engine::physics::driver::AloPhys::default().rot;
        }
    }
    jt.po_move = None;
    true
}
