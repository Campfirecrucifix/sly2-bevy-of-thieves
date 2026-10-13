//! WP-P10: player polish.
//!
//! Sensor parts: FUN_001f31e0 stores the part stream flag bytes as part +0x10 bits 48 (b0), 49 (b1, sphere)
//! and 50 (b2). Bit 48 marks a SENSOR part: FUN_001aa1d8 never pairs two of them and routes every contact
//! with one to the touching list only (iStack_c4 -> obj +0x6c via FUN_001a9fe0), never to the solver list,
//! and the ledge clearance FUN_001997d8 skips them. `CollisionWorld::build_so` keeps them as solids (row 10:
//! obj 1102, a 6600 x 5600 x 1500 box around the spawn, popped Sly to its top at z 686), so the player
//! collides with `player_collision_world` = the SO set without bit-48 parts. REPLACED: the touching-list
//! records of sensor parts (consumers are class code, not the JT) are not produced.
#![allow(non_snake_case)]

use crate::engine::physics::query::Source;
use crate::engine::physics::CollisionWorld;
use crate::engine::world::objects::{LoadData, Payload};
use crate::engine::world::World;

/// Part +0x10 bit 48 (FUN_001f31e0 first flag byte & 1): sensor part.
pub fn part_is_sensor(w: &World, object: usize, source: Source) -> bool {
    let Source::AloPart(k) = source else { return false };
    match w.objects.get(object).map(|o| &o.payload) {
        Some(Payload::Loaded(LoadData::Alo { alo, .. })) => alo.parts.get(k).is_some_and(|p| p.b0 & 1 != 0),
        _ => false,
    }
}

/// The JT's collision set: `CollisionWorld::build_so` without sensor parts (see module doc).
pub fn player_collision_world(w: &World) -> CollisionWorld {
    let mut cw = CollisionWorld::build_so(w);
    cw.colliders.retain(|c| !part_is_sensor(w, c.object, c.source));
    cw
}
