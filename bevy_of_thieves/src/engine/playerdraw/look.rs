//! WIRE-AN: the JT head-look target pick of FUN_00142af8 (+0x10f8 target / +0x10fc priority) and the idle
//! SM mask FUN_001430c8 it feeds (0x18c250 jts 0 entry; FUN_0018a0c8 every jts-0 frame through
//! FUN_001f1350). Look targets are the class-21 C2db378 objects (DAT_003e1474, added by slot 22
//! FUN_0021d460 when in the world): +0xe8 range (2000), +0xec/+0xf0 z window (-800, 800), +0xf4 priority (1).
//! WIRE-EP4: the +0x10dc / +0x10e4 seed (FUN_00140558 guard focus) and live positions: `pick_at`.
//! Not modelled: the 0x3e1b08 / 0x3e1b20 extra
//! target, the +0x1100 hold, DAT_002dd640 suppression (all idle for Sly in the hub).
use crate::engine::classes::k10::C2db378;
use crate::engine::classes::k17_55::ANG_2BD2C0;
use crate::engine::game::fields::{ee_sub_1a62e8, object_props, out_of_world_at_load};
use crate::engine::world::World;

/// Look-at candidates (object index, C2db378 fields) of the level, in load order.
pub fn targets(w: &World) -> Vec<(usize, C2db378)> {
    w.of_class(21, false)
        .filter(|&i| !out_of_world_at_load(w, i))
        .map(|i| {
            // slot 6 FUN_0021d3d8 defaults, then the properties
            let mut c = C2db378 { f_f4: 1, f_e8: 2000.0, f_ec: [-800.0, 800.0], ..Default::default() };
            let o = &w.objects[i];
            if let Some(p) = w.objects.get(o.clone_of.unwrap_or(i)).and_then(object_props) {
                c.feed(p);
            }
            (i, c)
        })
        .collect()
}

/// FUN_001428d8(jt, l, &cur, &dist, &dz) with the joystick object +0x150 at stick magnitude `mag`
/// toward world angle `ang`: biased xy distance (vmul.xy / vaddy.x) and z of `pos(jt) - pos(l)`.
/// The DAT_002bd280 / 2bd290 hysteresis is taken only when `*cur == jt` (bne $s2 = a0, the JT itself, not
/// the candidate): never for a real target, so it is left out.
pub fn ee_sub_1428d8(jt: [f32; 3], l: [f32; 3], ang: f32, mag: f32) -> (f32, f32) {
    let d = [jt[0] - l[0], jt[1] - l[1], jt[2] - l[2]];
    let dist = (d[0] * d[0] + d[1] * d[1]).sqrt();
    let mut r = dist;
    if 0.0001 < dist {
        // FUN_0011e1a0(+0x18, +0x1c, 0): stick vector; DAT_002bd2a0.. cos weight, clamp [0, 1]; DAT_002bd2b0..
        let s = [ang.cos() * mag, ang.sin() * mag];
        let c = (s[0] * d[0] + s[1] * d[1]) / dist;
        let k = (0.5 - 0.5 * c).clamp(0.0, 1.0);
        r += k * (-500.0 - 0.5 * dist);
    }
    (r, d[2])
}

/// FUN_00142af8 pick: (target object, priority) for the JT at `jt` (static object positions, no seed).
pub fn pick(w: &World, list: &[(usize, C2db378)], jt: [f32; 3], ang: f32, mag: f32) -> (Option<usize>, i32) {
    pick_at(list, jt, ang, mag, (None, -1), &mut |i| ee_sub_1a62e8(w, i))
}

/// WIRE-EP4: FUN_00142af8 pick 1:1 with the +0x10dc / +0x10e4 seed (`seed` = (target object, kind), the
/// FUN_00140558 focus; 0x142bcc: s3 = +0x10dc, s1 = +0x10e4, score of the seed or DAT_002a4d68) and the
/// live FUN_001a62e8 position `pos` of each object (actor-owned targets move: prologue 150510 under the
/// class-83 actor 150508).
pub fn pick_at(list: &[(usize, C2db378)], jt: [f32; 3], ang: f32, mag: f32, seed: (Option<usize>, i32), pos: &mut dyn FnMut(usize) -> [f32; 3]) -> (Option<usize>, i32) {
    let (mut best, mut pri) = seed;
    let mut bd = best.map_or(f32::MAX, |t| ee_sub_1428d8(jt, pos(t), ang, mag).0);
    for (i, l) in list {
        let lp = l.f_f4 as i32;
        if lp < pri {
            continue;
        }
        let (dist, dz) = ee_sub_1428d8(jt, pos(*i), ang, mag);
        // !(range < dist) and FUN_0011d910(dz, +0xec): open interval
        if !(l.f_e8 < dist) && l.f_ec[0] < dz && dz < l.f_ec[1] && (pri < lp || (lp == pri && dist < bd)) {
            best = Some(*i);
            bd = dist;
            pri = lp;
        }
    }
    (best, pri)
}

/// FUN_001430c8: idle SM mask of priority `pri` (+0x10fc) and target `t` (+0x10f8) for the JT at `jt`
/// with rotation rows `rot` (world matrix rows; FUN_001a6f00 = local coordinates).
pub fn ee_sub_1430c8(w: &World, t: Option<usize>, pri: i32, jt: [f32; 3], rot: &[[f32; 3]; 3]) -> u32 {
    ee_sub_1430c8_at(t.map(|t| ee_sub_1a62e8(w, t)), pri, jt, rot)
}

/// FUN_001430c8 with the target's world position `tp` (FUN_001a62e8 of +0x10f8).
pub fn ee_sub_1430c8_at(tp: Option<[f32; 3]>, pri: i32, jt: [f32; 3], rot: &[[f32; 3]; 3]) -> u32 {
    let mut side = 0u32;
    if let Some(p) = tp {
        let d = [p[0] - jt[0], p[1] - jt[1], p[2] - jt[2]];
        let l = rot.map(|r| r[0] * d[0] + r[1] * d[1] + r[2] * d[2]);
        let a = l[1].atan2(l[0]);
        if ANG_2BD2C0[0] <= a.abs() {
            side = 3;
            if a.abs() <= ANG_2BD2C0[1] {
                side = if 0.0 < a { 1 } else { 2 };
            }
        }
    }
    match pri {
        1 => 0x10 << side,
        2 => 0x100 << side,
        3 => 0x1000,
        4 => 0x100,
        _ => 8,
    }
}
