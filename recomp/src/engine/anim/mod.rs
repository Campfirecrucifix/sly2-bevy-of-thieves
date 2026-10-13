//! WP-D: animation (ENGINE_PORT.md). Sly 2 keeps Sly 1's model: an animation segment object
//! (ASEG, classes 109/110, load FUN_0012f2a8) holds per-target channels whose curves come from
//! two key factories (FUN_0012b3b0 vector curves, FUN_0012cb98 scalar curves); bones/attach
//! points live in the ALO (world/alo.rs FUN_0022d7a0) and are evaluated per bone kind
//! (+0x94 kind / +0x98 motion mode; FUN_0022cdc0 = bone velocity, motion.rs). Pose output = per-bone matrices + skinning palette.
//!
//! WP-D2: blend.rs (curve kind 5 FUN_00138660, scalar kind 2 FUN_00138ba0), play.rs (ASEGA
//! playback FUN_00130918 + events + driver system), motion.rs (bone velocity FUN_0022cdc0,
//! FUN_0022d148), skin.rs (feature `game`: palette -> SkinnedMesh joints). The world loader
//! dispatches classes 109/110 and the ALO wrappers below (world/objects.rs).
//!
//! Class -> load mapping owned by this WP (vtable +0x74):
//!   109, 110 -> FUN_0012f2a8 = [`aseg::ee_sub_12f2a8`]
//!   55 (FUN_00134770), 80-82 (FUN_001b1e00), 83 (FUN_00148c08), 85 (FUN_001de478),
//!   91-93, 95 (FUN_001c4ef8), 94 (FUN_001c76d0), 96 (FUN_001b0ca8), 98 (FUN_001baba0),
//!   103 (FUN_0013be58) -> ALO chain, tail per [`aseg::alo_wrapper_tail`] ([`load_alo_wrapper`]).

pub mod actor;
pub mod aseg;
pub mod bind;
pub mod blend;
pub mod blends;
pub mod events;
pub mod keys;
pub mod motion;
pub mod play;
pub mod pose;
pub mod skin_mesh;
pub mod sm;
#[cfg(feature = "game")]
pub mod skin;

use crate::engine::disc::lz::BinaryInputStream;
use crate::engine::world::alo::{ee_sub_1f31e0, read_tail, Alo};
use crate::engine::world::objects::{ee_sub_13fb40, ObjectSink, WorldError};

/// Stream part of an ALO-wrapper load (see [`aseg::alo_wrapper_tail`]): FUN_001f31e0, nested
/// FUN_0013fb40 children owned by `owner` (already pushed, as FUN_0013e3c8 does), then the tail.
/// None if `load` is not one of the wrapper loads.
pub fn load_alo_wrapper(
    load: u32,
    s: &mut BinaryInputStream,
    sink: &mut dyn ObjectSink,
    owner: usize,
    entry: usize,
    mission: i16,
) -> Option<Result<(Alo, Option<Vec<u8>>, Vec<usize>), WorldError>> {
    let tail = aseg::alo_wrapper_tail(load)?;
    Some((|| {
        let alo = ee_sub_1f31e0(s, mission)?;
        let children = ee_sub_13fb40(sink, Some(owner), entry, s, mission)?;
        Ok((alo, read_tail(tail, s), children))
    })())
}
