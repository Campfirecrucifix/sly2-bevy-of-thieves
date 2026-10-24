//! PORT2: getter rows whose callees were ported in subsystem modules (were partial: "needs X not in
//! ClassLink::run"); here they run on the world / script state the getter already has.
//!  - 0x00101088 (field 54, S32): FUN_001f78d8(alo, mask) = OR of ALO +0x110[i] for mask bit i < 26
//!    (`player::p8_hazard::FUN_001f78d8`, bytes from `ObjectFields::alo_110` written by method 0x00100fe0).
//!  - 0x00101b60 (field 115, F32): FUN_0012f990(aseg, id) label-event time (`playerdraw::jtblend::ee_sub_12f990`).
//!  - 0x00104718 (field 366, Bool): FUN_00227c90(vol, p) point in volume (`classes::guard::sensor::ee_sub_227c90`).
use crate::engine::world::objects::{LoadData, Payload};
use crate::engine::world::World;

/// 0x00101088
pub fn ee_sub_101088(t: Option<&[u8; 26]>, mask: u32) -> i32 {
    // 0x00101088
    crate::engine::player::p8_hazard::FUN_001f78d8(t.unwrap_or(&[0; 26]), mask) as i32
}

/// 0x00101b60: obj must be an ASEG (classes 109/110); anything else reads no events = 0.
pub fn ee_sub_101b60(w: Option<&World>, obj: u32, id: i32) -> f32 {
    // 0x00101b60
    match w.and_then(|w| w.objects.get(obj as usize)).map(|o| &o.payload) {
        Some(Payload::Loaded(LoadData::Aseg(a))) => crate::engine::playerdraw::jtblend::ee_sub_12f990(a, id),
        _ => 0.0,
    }
}

/// 0x00104718: obj = class-22 VOL (solid); volume space = inverse of its world transform.
pub fn ee_sub_104718(w: Option<&World>, obj: u32, p: [f32; 3]) -> bool {
    // 0x00104718
    use crate::engine::classes::guard::sensor::{ee_sub_227c90, Vol};
    use crate::engine::physics::geom::{build_bsp, build_geom};
    let Some(w) = w else { return false };
    let k = obj as usize;
    let solid = match w.objects.get(k).map(|o| &o.payload) {
        Some(Payload::Loaded(LoadData::Solid(s))) => s,
        Some(Payload::Loaded(LoadData::L2282b0 { solid, .. })) => solid,
        _ => return false,
    };
    let mut bw = crate::engine::gfx::attach::BoneWorlds::default();
    let Some(mw) = bw.xfm_world(w, k) else { return false };
    let v = Vol::new(k, crate::engine::gfx::scene::invert(&mw), build_geom(&solid.geom), build_bsp(&solid.bsp));
    ee_sub_227c90(&v, p)
}
