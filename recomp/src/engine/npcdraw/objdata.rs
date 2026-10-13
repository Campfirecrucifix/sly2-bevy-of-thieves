//! WP-A7: the class-state side of `objects` (no Bevy): where the K13 / K14 objects are drawn.

use crate::engine::classes::k14::C2d0b30;
use crate::engine::classes::shared::proj::Proj;
use crate::engine::classes::ClassHost;
use crate::engine::gfx::attach::{affine_from_bytes, BoneWorlds};
use crate::engine::gfx::scene::{mul, Affine};
use crate::engine::world::World;

/// Class 81 in flight: FUN_0013a9e8 sets +0x200 = self; FUN_001d6b00 (FUN_0013bc60) sets it to the launcher.
pub fn proj_in_flight(obj: usize, p: &Proj) -> bool {
    p.f_200 != obj as u32 + 1
}

/// Class 81 body world (xfAng rows, xfLin +0x30) as a level-space affine.
pub fn proj_world(p: &Proj) -> Affine {
    let r = &p.phys.rot;
    [r[0], r[1], r[2], p.phys.pos]
}

/// Placement of an object copy (clone without entities of its own): bone 0 world * placed (npcdraw m0).
pub fn clone_world(w: &World, bw: &mut BoneWorlds, o: usize) -> Affine {
    match w.life.placed.get(&o).map(affine_from_bytes) {
        Some(pm) => mul(&bw.get(w, o, 0), &pm),
        None => bw.get(w, o, 0),
    }
}

/// Class 76 model object (+0x1c4, resolved handle = object + 1), template of a clone.
pub fn swarm_model(w: &World, s: &C2d0b30) -> Option<usize> {
    let m = s.f_1c4 as usize;
    (m > 0 && m <= w.objects.len()).then(|| super::template_of(w, m - 1))
}

/// Class 76 slot 18 FUN_001821c0: one level-space affine per alive rider (FUN_001a5ee8 draws the model
/// subtree rigidly with its root at that matrix).
pub fn rider_worlds(s: &C2d0b30, obj: usize, h: &mut dyn ClassHost) -> Vec<Affine> {
    s.ee_sub_1821c0(obj, h).iter().map(|m| [[m[0][0], m[0][1], m[0][2]], [m[1][0], m[1][1], m[1][2]], [m[2][0], m[2][1], m[2][2]], [m[3][0], m[3][1], m[3][2]]]).collect()
}
