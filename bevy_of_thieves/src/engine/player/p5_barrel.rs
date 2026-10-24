//! BOSS R22: the DAT_003e13e4 block of FUN_00197f98 (JMP air update, jt+0x2c0): while airborne in jtbs 1 / 3 /
//! 4 / 5 / 0x11 / >= 0x17 the JT picks an idle TNT barrel (class 88 C2d20d0, listed in DAT_003e13e4 by its
//! slot 22 FUN_00183998; TARGET sub-struct at +0x150, FUN_00183900 range +0x158 = 300) and flies into it:
//! FUN_00195fb8(jmp, 6, barrel) (entry: xf kind 0x11, FUN_00183b80(barrel, 1)), arrival FUN_0019c688 jtbs 6 ->
//! FUN_001928f8(1, barrel) -> FUN_00192bd8 kind 1 -> FUN_00183b80(barrel, 2) (worn). No button: jumping near
//! the barrel is enough (rec11 C:/Temp/emu_rec11_ep3 'Sly jumped in the TNT barrel', barrel +0x17c 0 -> 2).
//! The barrels are fed from the live image by classes::q::barrel::feed (state +0x17c, FUN_001a62e8 position).
#![allow(non_snake_case)]
use super::p5_target::{FUN_0011ee08, FUN_0019a4a0};
use crate::engine::physics::geom::V3;

/// DAT_002bf83c (accept distance of the closest approach) / 0x48afc800 (xy distance² cut) / 0x44bb8000 (cost cut)
pub const DAT_002BF83C: f32 = 100.0;
const XY2_MAX: f32 = 360000.0;
const COST_MAX: f32 = 1500.0;

/// One DAT_003e13e4 entry: world object index, FUN_001a62e8 position, vt slot 32 velocity, +0x158 TARGET
/// range, +0x17c state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Barrel {
    pub id: usize,
    pub pos: V3,
    pub vel: V3,
    pub range: f32,
    pub state: u32,
}

/// Result of the barrel search: barrel, f23 = arrival time, aim (sp+0x40) and aim velocity (sp+0x60).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BarrelPick {
    pub id: usize,
    pub t: f32,
    pub aim: V3,
    pub aim_vel: V3,
}

/// jtbs that run the search (switch 0x2a6470 cases 1 / 3 / 4 / 5 / 0x11 and the default >= 0x17).
pub fn searches(jtbs: i32) -> bool {
    jtbs as u32 >= 0x17 || matches!(jtbs, 1 | 3 | 4 | 5 | 0x11)
}

/// FUN_00197f98 0x198128.. DAT_003e13e4 loop (1:1 order; the DAT_003e1420 class-40 loop that follows is not
/// part of this). `pos` = FUN_001a62e8(jt), `vel` = jt vt slot 32, `hofs` = jt+0x240, `gz` = gravity z.
pub fn FUN_00197f98_barrel(pos: V3, vel: V3, hofs: V3, gz: f32, barrels: &[Barrel]) -> Option<BarrelPick> {
    let mut best = None;
    let mut f22 = COST_MAX;
    for b in barrels.iter().filter(|b| b.state == 0) {
        let p = b.pos;
        let d = [p[0] - pos[0], p[1] - pos[1]];
        if XY2_MAX < d[0] * d[0] + d[1] * d[1] {
            continue;
        }
        if p[2] + b.range < pos[2] {
            continue;
        }
        // FUN_0019a4a0(jmp, X, &T, &cost, &aim, &aimVel; vin, X - jt+0x240, 0) (+ the aimVel.z rewrite)
        let tp = [p[0] - hofs[0], p[1] - hofs[1], p[2] - hofs[2]];
        let r = FUN_0019a4a0(pos, gz, vel, b.vel, tp, [0.0; 3]);
        if f22 < r.cost {
            continue;
        }
        let dd = [pos[0] - p[0], pos[1] - p[1], 0.0];
        let f21 = 0.25f32;
        let f0 = f21.min(r.t);
        let dv = [vel[0] - b.vel[0], vel[1] - b.vel[1], vel[2] - b.vel[2]];
        let q = [dd[0] + dv[0] * f0, dd[1] + dv[1] * f0, 0.0];
        // FUN_0011ee08(0, dd, q, 0, &alt, &f2)
        let (c, alt) = FUN_0011ee08([0.0; 3], dd, q);
        let f2 = (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt();
        let f1 = DAT_002BF83C;
        let ok = (!(f1 < f2) && alt != 0.0) || (!(f1 * f21 < f2) && !(0.0 < vel[2]));
        if ok {
            let (Some(aim), Some(av)) = (r.aim, r.aim_vel) else { continue };
            best = Some(BarrelPick { id: b.id, t: r.t, aim, aim_vel: [av[0], av[1], vel[2] + r.t * gz] });
            f22 = r.cost;
        }
    }
    best
}
