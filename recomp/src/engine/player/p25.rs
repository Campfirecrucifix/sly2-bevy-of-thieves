//! WP-P25: spawn activation FUN_00228750 (class 0x2a C2dc550, called by FUN_0020e890 right after
//! FUN_00227fe8 found the requested spawn object, before the character selection).
//!
//! FUN_00228750(spawn):
//! 1. +0x148 pairs {oid a, oid b} (max 4, appended by Splice CMethod 0x10bbe0 -> FUN_00228958): a =
//!    FUN_0015e938(world, 4, oid a, spawn), b likewise; both found -> a vt+0x134 (slot 37 MoveToObject:
//!    0x1f49c8 ALO / 0x2052e0 PO / 0x1a6b88 LO base: a takes b's position (and orientation)).
//! 2. FUN_0020ff70(world, +0xe0, +0xe4, spawn, 1): per 0x10 record {oid, sub, state1, state0} (max 4,
//!    CMethod 0x10ba70 -> FUN_002288d8 -> FUN_0020fec0) -> FUN_0020fff0 flags 1: the class-0x30 object's
//!    sub state machine to state1 (-1 = none).
//! 3. FUN_00161f48(0x2deb30, +0x124, +0x128, spawn): per {task oid, state} (max 4, CMethod 0x10bb38 ->
//!    FUN_00228920 -> FUN_00161f10) -> FUN_00164640(task, state, spawn).
//! 4. FUN_0015e9c8(world, 5, 0x1a, 0, 0x20): every class-0x1a warp volume (first 0x20, walked last to
//!    first) whose +0x13c state != 2 -> FUN_00228358(volume, 0) (warp state reset).
//! 5. FUN_0014d8c0(g_cm): camera cut (level >= 1).
//! 6. vt+0x1c(spawn, 5, 0, 0, 0, 0) = slot 2 FUN_001a61e0: FUN_001331f8 (the spawn's level-script message 5)
//!    + FollowWorld_activity / FUN_001619a0(0x2deb30, 0x1a, spawn) (job event 0x1a).
//!
//! REPLACED: slot 37 has no run-time object transform owner in the port (world objects keep their load
//! transforms; the JT is placed by `PlayerLevel::spawn`), so the placements are returned in
//! `SpawnActivate::placed`; the script message is posted (K27 `k27_script_msg`, delivered at the next script
//! tick) instead of run synchronously. The three adder CMethods (0x10ba70 / 0x10bb38 / 0x10bbe0) are not
//! wired into the Splice host (game/, not P25): `SpawnLists` holds what they would add (empty otherwise).
use crate::engine::camera::cm::{Cm, FUN_0014d8c0};
use crate::engine::classes::{ClassHost, ClassSet, Inst};
use crate::engine::world::World;
use std::collections::HashMap;

/// Class id of the warp volumes walked by step 4 (C2dc6f0).
pub const CLASS_WARP: u16 = 0x1a;
/// FUN_00228958 / FUN_00228920 / FUN_002288d8: 4 entries each.
pub const MAX_SPAWN_LIST: usize = 4;
/// FUN_001619a0 event of FUN_001a61e0 message 5.
pub const EV_SPAWN_ACTIVATE: i32 = 0x1a;

/// The run-time lists of the class-0x2a objects (keyed by world object index).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SpawnLists {
    /// +0x148 / +0x14c {oid a, oid b}
    pub pairs: HashMap<usize, Vec<(i32, i32)>>,
    /// +0xe0 / +0xe4 {oid, sub, state1, state0}
    pub recs: HashMap<usize, Vec<[i32; 4]>>,
    /// +0x124 / +0x128 {task oid, state}
    pub tasks: HashMap<usize, Vec<(i32, i32)>>,
    /// WP-SCR10: warp-volume task entries (+0x164/+0x168) keyed by object index.
    /// Set by Splice method thunk 0x10b850 -> FUN_002284e0. Max 4 per warp.
    pub warp_tasks: HashMap<usize, Vec<[i32; 2]>>,
}

impl SpawnLists {
    /// 0x00228958: append a pair unless 4 are stored (silently ignored).
    pub fn FUN_00228958(&mut self, obj: usize, a: i32, b: i32) {
        let l = self.pairs.entry(obj).or_default();
        if l.len() < MAX_SPAWN_LIST {
            l.push((a, b));
        }
    }

    /// 0x002288d8(obj, flag, oid, sub, state): flag 0 = no-op; else FUN_0020fec0(world, +0xe4, 4, +0xe0, flag,
    /// oid, sub, state): an existing {oid, sub or -1} record gets slot [2] (flag != 0) else [3]; a new record
    /// sets the other slot to -1. Quirk kept: a new record is added only while count + 1 < 4 (3 usable).
    pub fn FUN_002288d8(&mut self, obj: usize, flag: i64, oid: i32, sub: i32, state: i32) -> bool {
        if flag == 0 {
            return false;
        }
        let l = self.recs.entry(obj).or_default();
        // FUN_0020fec0 (param_5 = flag != 0 here, so always slot [2])
        for r in l.iter_mut() {
            if r[0] == oid && (r[1] == -1 || r[1] == sub) {
                r[2] = state;
                return true;
            }
        }
        if l.len() + 1 < MAX_SPAWN_LIST {
            l.push([oid, sub, state, -1]);
            return true;
        }
        false
    }

    /// 0x00228920 -> FUN_00161f10(.., +0x128, 4, +0x124, oid, state)
    pub fn FUN_00228920(&mut self, obj: usize, oid: i32, state: i32) -> bool {
        let l = self.tasks.entry(obj).or_default();
        if l.len() >= MAX_SPAWN_LIST {
            return false;
        }
        l.push((oid, state));
        true
    }
}

/// What FUN_00228750 did (for the caller and the tests).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SpawnActivate {
    /// slot-37 placements (a, b): object a moved to object b (REPLACED: recorded)
    pub placed: Vec<(usize, usize)>,
    /// warp volumes FUN_00228358(v, 0) ran on (step 4 order)
    pub warp_reset: Vec<usize>,
    /// warp volumes in the FUN_0015e9c8 list
    pub warps: usize,
    pub cut: bool,
}

/// FUN_0015e9c8(world, 5, 0x1a, 0, 0x20): the class-0x1a subtree, per class id then creation order, first 0x20.
pub fn warp_volumes(w: &World) -> Vec<usize> {
    crate::engine::game::spawn::ee_sub_15e9c8(w, None, CLASS_WARP).1
}

/// 0x00228750 (see the module doc). `spawn` = world object index of the class-0x2a object.
pub fn FUN_00228750(w: &World, spawn: usize, lists: &SpawnLists, set: &mut ClassSet, h: &mut dyn ClassHost, cm: Option<&mut Cm>) -> SpawnActivate {
    let mut out = SpawnActivate::default();
    let ctx = spawn as u32 + 1;
    // 1. pairs
    for &(a, b) in lists.pairs.get(&spawn).map(Vec::as_slice).unwrap_or(&[]) {
        let ha = h.ee_sub_15e938(4, a, ctx);
        if ha == 0 {
            continue;
        }
        let hb = h.ee_sub_15e938(4, b, ctx);
        if hb != 0 {
            out.placed.push((ha as usize - 1, hb as usize - 1)); // vt+0x134 (slot 37)
        }
    }
    // 2. FUN_0020ff70(.., flags 1): state1 of each record
    let recs: Vec<[u32; 3]> = lists.recs.get(&spawn).map(|l| l.iter().map(|r| [r[0] as u32, r[1] as u32, r[2] as u32]).collect()).unwrap_or_default();
    if !recs.is_empty() {
        h.ee_sub_20ff70(spawn, &recs);
    }
    // 3. FUN_00161f48
    let tasks: Vec<[u32; 2]> = lists.tasks.get(&spawn).map(|l| l.iter().map(|t| [t.0 as u32, t.1 as u32]).collect()).unwrap_or_default();
    if !tasks.is_empty() {
        h.ee_sub_161f48_list(spawn, &tasks);
    }
    // 4. warp volumes, last to first
    let vols = warp_volumes(w);
    out.warps = vols.len();
    for &v in vols.iter().rev() {
        let Some(k) = set.more.iter().position(|x| x.0 == v && matches!(x.1, Inst::C2dc6f0(_))) else { continue };
        let (_, inst) = &mut set.more[k];
        let Inst::C2dc6f0(c) = inst else { continue };
        if c.f_13c != 2 {
            c.ee_sub_228358(v, 0, &mut set.g, h);
            out.warp_reset.push(v);
        }
    }
    // 5. camera cut
    if let Some(cm) = cm {
        FUN_0014d8c0(cm);
        out.cut = true;
    }
    // 6. vt+0x1c(spawn, 5) = FUN_001a61e0: script message 5 + event 0x1a
    h.k27_script_msg(ctx, 5, &[]);
    h.ee_sub_1619a0(EV_SPAWN_ACTIVATE, ctx);
    out
}

// ---------------------------------------------------------------- jts 6 (gadget 0x18)
//
// Gadget 0x18 (usable in jts 5 beside the focus object, FUN_0018da48 -> SetJtJts(6)) runs the jt+0x1ff0 block:
// init 0x1371b8 ([0] = jt, [1] = -1), input 0x1371c8 (0x18a8c8 case 6), update 0x137218 (0x18b7e8 case 6),
// forces 0x137248 (P5, orbit of the +0x10dc target at radius 250), contact +0x94 0x1374e0 (p5_hull), enter /
// exit 0x1375d0 with the side pick 0x1374f0 (SetJtJts case 6).

use super::{Jt, JtAnim, JTBS_NIL, JTS_STAND};
use crate::engine::physics::geom::V3;

/// jts 6 side ASEGs jt+0x2000 / +0x2004 (block [4] / [5]; post-load oids 0x119 / 0x11a).
pub const ASEG_6: [u32; 2] = [0x2000, 0x2004];
/// DAT_002bc9b4: 0x1374e0 contact +0x94.
pub const DAT_002BC9B4: f32 = 0.0;
/// DAT_002bc9b8: 0x1375d0 exit velocity scale.
pub const DAT_002BC9B8: f32 = 0.5;

/// jt+0x1ff0 block.
#[derive(Clone, Debug, PartialEq)]
pub struct Jts6 {
    /// [1] requested side (-1 = out of jts 6)
    pub side: i32,
    /// [2] DAT_002dded0 at the last side change
    pub t: f32,
    /// [3] side whose ASEG runs (0 / 1)
    pub cur: i32,
}

impl Default for Jts6 {
    /// 0x001371b8: [1] = -1 ([3] is zero-initialized with the JT)
    fn default() -> Self {
        Self { side: -1, t: 0.0, cur: 0 }
    }
}

fn sub3(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

impl Jt {
    /// 0x001371c8 (jts 6 input): the side ASEG jt+0x1030 is done -> vt+0x2c4(jt, 0).
    pub fn FUN_001371c8(&mut self, anim: &mut dyn JtAnim) {
        if self.p16_aseg_done(ASEG_6[(self.j6.cur & 1) as usize]) {
            self.set_jts(JTS_STAND, JTBS_NIL, anim);
        }
    }

    /// 0x00137218 (jts 6 update): +0x10dc set -> FUN_002046f0 face it.
    pub fn FUN_00137218(&mut self) {
        if self.p13.f_10dc != 0 {
            if let Some(p) = self.p13.pos_10dc {
                let d = sub3(p, self.phys.pos);
                self.face_dir([d[0], d[1], 0.0]);
            }
        }
    }

    /// 0x001374f0 side pick: stick held and a +0x10dc target -> 1 when the target is not on the side of the
    /// stick direction turned by -pi/2 (dot <= 0), else 0; otherwise the current side [3].
    pub fn FUN_001374f0(&self) -> i32 {
        if !(0.0 < self.joy.mag) || self.p13.f_10dc == 0 {
            return self.j6.cur;
        }
        let Some(p) = self.p13.pos_10dc else { return self.j6.cur };
        let dir = super::FUN_0011e1a0(self.joy.angle - std::f32::consts::FRAC_PI_2, 1.0, 0.0);
        let d = sub3(p, self.phys.pos);
        (dir[0] * d[0] + dir[1] * d[1] + dir[2] * d[2] <= 0.0) as i32
    }

    /// 0x001375d0(block, side): side -1 = exit (velocity * DAT_002bc9b8 through vt+0x10c / +0x11c); 0 / 1 =
    /// face the target (0x2046f0), velocity from the side ASEG root channel, +0x190 = -offset, [3] = side, then
    /// FUN_001f15b8(jt+0x1034) + FUN_001305b8 play the ASEG into jt+0x1030.
    /// REPLACED: the root channel (FUN_0012ff10 + channel vt+0x1c) has no port: velocity / +0x190 untouched
    /// (recorded 0x12ff10); FUN_001f15b8 recorded.
    pub fn FUN_001375d0(&mut self, side: i32, anim: &mut dyn JtAnim) {
        if side == self.j6.side {
            return;
        }
        self.j6.side = side;
        self.j6.t = self.t;
        if side == -1 {
            let v = self.velocity_pub();
            self.set_velocity_pub([v[0] * DAT_002BC9B8, v[1] * DAT_002BC9B8, v[2] * DAT_002BC9B8]);
            return;
        }
        if (0..2).contains(&side) {
            self.FUN_00137218(); // FUN_002046f0(jt, +0x10dc)
            self.calls.push((0x0012ff10, side));
            self.j6.cur = side;
            self.p16.t_off = 0.0; // REPLACED jt+0x1030 clock restarts with the side ASEG
        }
        self.calls.push((0x001f15b8, 0));
        if (0..2).contains(&side) {
            anim.play_aseg(ASEG_6[side as usize], 6, -1);
        }
    }
}
