//! WP-P20: Bentley / Murray player layer (the controlled character, DAT_003e138c / world +0x3efc).
//!
//! Original character switch: the job task's +0xc0 oid (prop 1276) is written to the splice global
//! `oidPoCurrent` by FUN_00165280 / FUN_00165538 (FUN_001d8b20 message 0x15, `JobMgr::calls`
//! (0x1d8b20, c0)); after the level load FUN_0020e890 reads it back (FUN_001d8a38 0x2a8a88), resolves it
//! with FUN_0015e938(world, 5, oid, 0) and falls back to DAT_002e1e40 (Sly, class 53), DAT_002dd5bc (Bentley,
//! class 55), DAT_002f7900 (Murray, class 54); then vtable +0x44 and slot 78 +0x27c(po, 1) = FUN_001406e8
//! (`ClassSet::k18_set_active`), which runs FUN_001c3ed8 -> FUN_001c4940 (world +0x3efc).
//!
//! 1:1: the FUN_0020e890 po selection (`ee_sub_20e890_po`), the oidPoCurrent source (`oid_po_current`).
//! REPLACED: the controlled Bentley / Murray body is the player's `Jt` ("shadow" JT): its JtUpdate /
//! forces move and collide the body (the class-55/54 slot 69 forces FUN_00204108 / component parts are
//! engine hooks with no port), the classes run their own slot 15 update on the joystick the JT read
//! (`k18_feed_joy`), the camera follows the JT focus (= the active character) and the active character's
//! ALO tree is drawn at the JT body (rest pose: the JT anim fields do not map onto the Bentley / Murray
//! ASEGs). `PoHost::po_call` answers the embedded JT-layout predicate components from the shadow JT
//! (FUN_00140af8 +0, FUN_0019c8d0 +0x2c0, FUN_00194558 +0x4c0); every other component call returns 0
//! (its motion part runs inside the shadow JT's own update). Env `SLY2_PO_OID=<oid>` overrides
//! `oidPoCurrent` (port addition, stands in for the save variable / job start on levels entered directly).
//! UNPORTED: vtable +0x44 of the selected po, the FUN_001e1e30 switch case. (WP-P23: the slot-78 callers
//! FUN_002206c8 / FUN_001c8730 go through player::p23::apply_switches, the class-55/54 anims through
//! playerdraw::po; FUN_001d7268 calls class 70's own slot 78 FUN_001d7418, not a character switch.)
#![allow(non_snake_case)]

use super::Jt;
use crate::engine::world::World;

pub const CLASS_SLY: u16 = 53;
pub const CLASS_MURRAY: u16 = 54;
pub const CLASS_BENTLEY: u16 = 55;

/// FUN_0020e890 po selection: the object of `oid` (FUN_0015e938 kind 5 = first by oid), else the first
/// Sly / Bentley / Murray (DAT_002e1e40 / DAT_002dd5bc / DAT_002f7900).
pub fn ee_sub_20e890_po(w: &World, oid: i32) -> Option<usize> {
    if oid != -1 {
        if let Some(&o) = w.find_oid(oid).first() {
            return Some(o);
        }
    }
    [CLASS_SLY, CLASS_BENTLEY, CLASS_MURRAY].iter().find_map(|&c| w.objects.iter().position(|o| o.class == c))
}

/// `oidPoCurrent`: the last FUN_001d8b20 message 0x15 the job manager sent (-1 none).
pub fn oid_po_current(jobs: &crate::engine::jobs::JobMgr) -> i32 {
    std::env::var("SLY2_PO_OID")
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .or_else(|| jobs.calls.iter().rev().find(|c| c.0 == 0x001d8b20).map(|c| c.1))
        .unwrap_or(-1)
}

/// The controlled non-JT character (None: Sly / no level).
#[derive(bevy_ecs::prelude::Resource, Default, Clone, Debug)]
pub struct PoCurrent {
    /// object index of the active Bentley / Murray
    pub obj: Option<usize>,
    pub class: u16,
    /// world the selection was made for
    pub world_key: usize,
    /// WP-P22: FUN_0020b868 blend entries of the po (player::p22::loco_table)
    pub loco: Vec<(u32, [Option<super::p22::LocoEntry>; 2])>,
    /// WP-P22: bodies of the characters not controlled now (object or p22::BODY_SLY, pos, rows)
    pub bodies: Vec<(usize, crate::engine::physics::geom::V3, [crate::engine::physics::geom::V3; 3])>,
    /// WP-P23: the po's last FUN_0020bb58(u, blk) of the frame (blk, u), read by playerdraw::po
    pub bb58: Option<(u32, f32)>,
    /// WP-P23: live ASEGAs on the po (ASEG object, time) after the last class tick (p23::po_plays), read by
    /// playerdraw::po (the class runtime is lent to the scripts while the draw systems run)
    pub plays: Vec<(usize, f32)>,
}

/// ClassHost::po_call target: the shadow JT of the controlled character.
#[derive(Clone, Debug)]
pub struct PoHost {
    pub obj: usize,
    pub jt: Box<Jt>,
}

impl PoHost {
    pub fn po_call(&self, addr: u32, obj: usize, _off: u32, args: &[u32]) -> i64 {
        if obj != self.obj {
            return 0;
        }
        let a0 = args.first().copied().unwrap_or(0);
        match addr {
            0x00140af8 => self.jt.FUN_00140af8(a0 != 0) as i64,
            0x0019c8d0 => self.jt.FUN_0019c8d0(a0 as i32) as i64,
            0x00194558 => self.jt.FUN_00194558(a0 as i32) as i64,
            _ => 0,
        }
    }
}

/// Make the active character for world `w`: select the po (`ee_sub_20e890_po`) and, for a Bentley /
/// Murray, hand it to the class set (slot 78(1)). The body stays where the player was placed: FUN_0020e890
/// puts the selected po at the level's spawn object (J12, `spawn_player`), the Bentley / Murray objects
/// themselves sit at the origin in the level data. Returns false while the class set has not built the object yet (retry next frame).
pub fn apply_po(w: &World, oid: i32, set: &mut crate::engine::classes::ClassSet, host: &mut dyn crate::engine::classes::ClassHost, cur: &mut PoCurrent) -> bool {
    // WP-P23: slot-13 anim / SM bindings of every Bentley / Murray (p23::bind_all, once per instance)
    super::p23::bind_all(w, set, host);
    let Some(o) = ee_sub_20e890_po(w, oid) else { return true };
    let class = w.objects[o].class;
    if class != CLASS_BENTLEY && class != CLASS_MURRAY {
        return true;
    }
    set.k18_set_active(Some(o), host);
    if set.k18_active != Some(o) {
        return false;
    }
    cur.obj = Some(o);
    cur.class = class;
    cur.loco = super::p22::loco_table(w, o, class);
    true
}
