//! WP-P24: ASEG event kinds 7 / 8 (FUN_00131290 -> FUN_00210338) for the JT's own ASEGAs. The glider SMA
//! (JT +0x1d28, SM 0x5f6) opens with ASEG 5517 (t 0: kind 7 on 0x5f5 = show the glider model, channel kind 2
//! scale 0 -> 1 over 0.25 s; t 1/12: kind 0xd (7, 3775) / (7, 3784) = show the JT's glider mesh records) and closes
//! with 5518 (t 0: kind 8 on 0x5f5 = remove it, kind 0xe = hide the records).

use super::vis::JtVis;
use super::{aseg_of, ee_sub_15e938_below};
use crate::engine::anim::aseg::EventData;
use crate::engine::world::World;

/// FUN_00210338(f, oid, on): FUN_0015e938(world, 4, oid) (here: below the JT); f == 0 (or the object
/// without FUN_00132b10(0x30)) -> on ? vtable +0x44 (FUN_001a5478 show) : +0x4c (FUN_001a5630 remove).
/// f != 0 fades (FUN_001f7bf8 / FUN_001f7c68): UNPORTED, applied as the instant switch. Returns whether
/// the drawn state changed.
pub fn ee_sub_210338(w: &World, vis: &mut JtVis, oid: i32, on: bool) -> bool {
    let Some(o) = ee_sub_15e938_below(w, vis.jt, oid) else { return false };
    let before = vis.removed.len();
    if on {
        vis.ee_sub_1a5478(w, o);
    } else {
        vis.ee_sub_1a5630(w, o);
    }
    vis.removed.len() != before
}

/// FUN_00210408(world, on, oid, mesh oid, ctx): oid != -1 -> FUN_0015e938(world, 4, oid) (here: below the JT);
/// an ALO (FUN_00132b10(0x30)) -> FUN_001f6978 (show) / FUN_001f69f8 (hide) its mesh records with word `mesh`.
/// The glider's wing / harness meshes are JT records with words 3775 / 3784 (u1 bit 0: hidden at load),
/// shown by ASEG 5517 kind 0xd (7, 3775) / (7, 3784) and hidden by 5518 kind 0xe. oid -1 (ctx object): skipped.
pub fn ee_sub_210408(w: &World, vis: &mut JtVis, oid: i32, mesh: i32, on: bool) -> bool {
    if oid == -1 {
        return false;
    }
    let Some(o) = ee_sub_15e938_below(w, vis.jt, oid).filter(|&o| crate::engine::gfx::attach::alo_of(w, o).is_some()) else { return false };
    let before = vis.hidden.clone();
    vis.set_mesh(w, o, mesh, on);
    vis.hidden != before
}

/// Kinds 7 / 8 and 0xd / 0xe of the fired timed events (ASEG object, event index, reverse): `on` = (kind ==
/// 7 | 0xd) ^ reverse.
pub fn apply_e7(w: &World, vis: &mut JtVis, fired: &[(usize, usize, bool)]) -> bool {
    let mut changed = false;
    for &(o, i, rev) in fired {
        let Some(ev) = aseg_of(w, o).and_then(|a| a.events1.get(i)) else { continue };
        match (ev.kind, &ev.data) {
            (7 | 8, EventData::S32F(oid, _)) => changed |= ee_sub_210338(w, vis, *oid, (ev.kind == 7) ^ rev),
            (0xd | 0xe, EventData::S32x2(oid, mesh)) => changed |= ee_sub_210408(w, vis, *oid, *mesh, (ev.kind == 0xd) ^ rev),
            _ => {}
        }
    }
    changed
}
