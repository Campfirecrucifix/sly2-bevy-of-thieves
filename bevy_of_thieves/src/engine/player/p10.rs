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
    // CRAWL R67: a clone (CloneSo) carries copies of the source's parts, +0x10 flags included (row 6 class-48
    // clones 2428 / 2429 of 2420: part 0 = the sensor disk; EMU savestate: only part 1 is in their SO bounds)
    let src = w.objects.get(object).and_then(|o| if matches!(o.payload, Payload::Clone) { o.clone_of } else { None });
    match w.objects.get(src.unwrap_or(object)).map(|o| &o.payload) {
        Some(Payload::Loaded(LoadData::Alo { alo, .. })) => alo.parts.get(k).is_some_and(|p| p.b0 & 1 != 0),
        _ => false,
    }
}

/// The JT's collision set: `CollisionWorld::build_so` without sensor parts (see module doc).
pub fn player_collision_world(w: &World) -> CollisionWorld {
    player_collision_world_masked(w, &Default::default())
}

/// WP-EP1d: `player_collision_world` with the run-time part bit-50 state (`CollisionWorld::build_so_masked`).
pub fn player_collision_world_masked(w: &World, on: &std::collections::HashMap<(usize, usize), bool>) -> CollisionWorld {
    let mut on2 = on.clone();
    for (key, v) in character_part_unlinks(w) {
        on2.entry(key).or_insert(v);
    }
    let mut cw = CollisionWorld::build_so_masked(w, &on2);
    cw.colliders.retain(|c| !part_is_sensor(w, c.object, c.source));
    // FEEL R73: SO/SO pairs only exist when max(a slot 60(b), b slot 60(a)) is not 1 / 3 (FUN_00133a38 ->
    // FUN_00133d08); a class-86 filter SO (C2caf88 slot 60 FUN_00132140) answers 1 to every object outside its
    // oid / class lists when +0x178 == 0, so e.g. EP4 hub obj 1832 (oid 77187, list = class 56 guards) is a guard-only
    // wall Sly slid into and wedged on (cap_feel_wall_steps12 slides past it). SLY2_FEEL_C86=0 = old (kept).
    if std::env::var("SLY2_FEEL_C86").map_or(true, |v| v != "0") {
        cw.colliders.retain(|c| !class86_ignores_players(w, c.object));
    }
    cw
}

/// FEEL R73: true when class-86 object `o`'s slot 60 FUN_00132140 answers 1 or 3 (no collision pair) for every
/// player character (Sly 53 / Murray 54 / Bentley 55): `b` = the player's oid is in the +0x154 list (FUN_001a6290,
/// REPLACED: own oid only) or the player is-a a +0x168 class (FUN_00132b10); +0x178 == 0 -> b ? 2 : 1;
/// else b ? 3 : FUN_001f4fc8 (REPLACED: 0, i.e. collides). The JT side (FUN_00144388) can only raise it.
pub fn class86_ignores_players(w: &World, o: usize) -> bool {
    use super::p20::{CLASS_BENTLEY, CLASS_MURRAY, CLASS_SLY};
    let Some(lo) = w.objects.get(o).filter(|lo| lo.class == 86) else { return false };
    // a clone (CloneSo) copies the source's fields
    let src = if matches!(lo.payload, Payload::Clone) { lo.clone_of.unwrap_or(o) } else { o };
    let Some(Payload::Loaded(LoadData::Alo { alo, .. })) = w.objects.get(src).map(|s| &s.payload) else { return false };
    let mut f = crate::engine::classes::actor::c2caf88::C2caf88::default();
    f.feed(&alo.props);
    [CLASS_SLY, CLASS_MURRAY, CLASS_BENTLEY].iter().all(|&pc| {
        let oid_hit = w.objects.iter().any(|p| p.class == pc && f.f_154.contains(&p.oid));
        let b = oid_hit || f.f_168.iter().any(|&c| super::p4::FUN_00132b10(pc as i32, c));
        let rel = if f.f_178 == 0 { if b { 2 } else { 1 } } else if b { 3 } else { 0 };
        rel == 1 || rel == 3
    })
}

/// FEEL R52: the load-time part unlinks of the character classes' FUN_001a64e8 field tables: a row with flags
/// 0x20 (FUN_001f6030 part lookup by id, first match) and 0x200 calls FUN_001f2ad0 (part +0x10 bit 50 set, the
/// part leaves the body's collision list). Tables: JT (class 53) FUN_00188488 0x2be990 (0xad rows) + post-load
/// 0x2bf1b0 (0x12); Bentley (55) 0x2bc390 (0x76); Murray (54) 0x2c0270 (0x89); all three run FUN_001400b8 =
/// 0x2bc9f0 (0x35). Sly's hull variants 199, 201..206, 211, 215..217, 220, 1501 and the cane hit parts are all
/// unlinked this way (without it the motor's mesh hull collided with Sly's own variant octahedra). Run-time
/// toggles in `on` (FUN_001f2a60 / FUN_001f2ad0 later) take precedence. UNPORTED: flag 0x10 rows (FUN_001f5ff8 +
/// FUN_001f69f8).
pub fn character_part_unlinks(w: &World) -> Vec<((usize, usize), bool)> {
    use crate::engine::disc::exe::rows3;
    use super::p20::{CLASS_BENTLEY, CLASS_MURRAY, CLASS_SLY};
    let mut out = Vec::new();
    for (i, o) in w.objects.iter().enumerate() {
        let tables: &[(u32, usize)] = match o.class {
            CLASS_SLY => &[(0x002b_e990, 0xad), (0x002b_c9f0, 0x35), (0x002b_f1b0, 0x12)],
            CLASS_BENTLEY => &[(0x002b_c390, 0x76), (0x002b_c9f0, 0x35)],
            CLASS_MURRAY => &[(0x002c_0270, 0x89), (0x002b_c9f0, 0x35)],
            _ => continue,
        };
        let Payload::Loaded(LoadData::Alo { alo, .. }) = &o.payload else { continue };
        for &(t, n) in tables {
            for (f, id, _) in rows3(t, n) {
                if f & 0x10 == 0 && f & 0x20 != 0 && f & 0x200 != 0 {
                    if let Some(k) = alo.parts.iter().position(|p| p.w0 == id) {
                        out.push(((i, k), false));
                    }
                }
            }
        }
    }
    out
}
