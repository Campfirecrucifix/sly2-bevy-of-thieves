//! WP-P26: the carry / throw machine FUN_002206c8 (SetState) + its aim arc FUN_0021fb70.
//!
//! The machine is embedded in every player character (JT layout +0x11b0; Bentley +0x11b0, Murray +0x1910),
//! word k = +off + 4k: [0] owner, [1] state, [2] state time, [3] reference object, [4] carried object
//! (DAT_002dd5bc = the first Bentley when Murray carries him), [6] aim yaw range, [7] / [8] aim distance range,
//! [9] flight time, [0x10] / [0x13] blend weights, [0x16] [0x17] [0x18] [0x19] [0x26] [0x27] [0x28] [0x2f]
//! state anims, [0x1a] [0x1d] [0x20] [0x23] [0x29] [0x2c] aim blend weights, +0xc0 local aim point,
//! +0xd0 local launch point, +0xe0 target, +0xf0 launch velocity, [0x4a], [0x4b] arc display, [0x4c] arc time.
//!
//! Character switching (slot 78 = FUN_001406e8 via vt +0x27c): entering state 3 makes the owner the
//! controlled character, leaving state 3 for any state but 4 gives control to DAT_002dd5bc (Bentley).
//! Leaving to -1 with Bentley carried (Bentley +0x1800 == 0) throws him: FUN_00136530(Bentley, 1) and
//! slot 71 (vt +0x11c) velocity = horizontal dir(Bentley - owner) * 400 + up * 600.
//!
//! The machine runs on the class memory (`Mem`, handles = object index + 1). Engine calls the port has no owner
//! for (anim play FUN_001305b8, facing FUN_00204660 / 0x2046b0, FUN_001cd098 / 0x1cd2c0 owner+0x900 component,
//! FUN_00134320 list, power FUN_00141cd0, carried vt +0x264 / FUN_001a58d0 / 0x1f7bf8 drop, arc display
//! vt calls) are returned in `CarryOut::calls` (REPLACED: recorded, see Findings P26).
use crate::engine::classes::k17_55::Mem;
use crate::engine::classes::shared::launch::FUN_001d6848;
use crate::engine::classes::ClassHost;

/// Carry machine offset in the JT layout (Sly, Bentley) / in Murray.
pub const CARRY_JT: u32 = 0x11b0;
pub const CARRY_MURRAY: u32 = 0x1910;
/// FUN_00132b10 class of the carryable objects (C2dbb10 subtree 100..102).
pub const CLASS_CARRYABLE: i32 = 100;
/// 0x2c40e4: flight time factor with splice bif 138(0x24) set.
pub const DAT_002C40E4: f32 = 1.35;
/// 0x2c4100 / 0x2c4104: arc time / aim blend rates (per second).
pub const DAT_002C4100: f32 = 3.0;
pub const DAT_002C4104: f32 = 2.0;
/// 0x2a96a0 = FLT_MAX
const DAT_002A96A0: f32 = f32::MAX;
/// FUN_0021fb70: arc samples (0x1d).
pub const ARC_N: usize = 0x1d;

/// Everything FUN_002206c8 reads outside the machine words.
#[derive(Clone, Debug, Default)]
pub struct CarryCx {
    /// owner handle (word [0])
    pub owner: u32,
    /// DAT_002dd5bc: the first Bentley (handle, 0 = none)
    pub bentley: u32,
    /// Bentley +0x1800 (0 -> he is thrown on release)
    pub bentley_1800: i32,
    /// carried +0x15c (FUN_00141cd0 power cost of state 7)
    pub carried_15c: u32,
    /// carried +0x10 / reference [3] +0x10 (same parent -> drop to the world)
    pub carried_10: u32,
    pub ref_10: u32,
    /// carried +0xe8 -> +0x80: gravity of the carried object
    pub gravity: [f32; 3],
    /// DAT_002dded0 / DAT_002dded4
    pub t: f32,
    pub dt: f32,
    /// splice bif 138(0x24)
    pub bif138: bool,
}

/// The results of one FUN_002206c8 / FUN_0021fb70 call.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CarryOut {
    /// vt +0x27c(obj, 1) = slot 78 calls (handles, in order)
    pub slot78: Vec<u32>,
    /// FUN_00136530(Bentley, 1) + slot 71 velocity
    pub throw: Option<(u32, [f32; 3])>,
    /// recorded engine calls (address or vtable offset, object handle, argument)
    pub calls: Vec<(u32, u32, i32)>,
    /// FUN_0021fb70 arc points (landing first, launch point last)
    pub arc: Vec<[f32; 3]>,
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn at(p0: [f32; 3], v: [f32; 3], g: [f32; 3], t: f32) -> [f32; 3] {
    let h = t * t * 0.5;
    [0, 1, 2].map(|k| p0[k] + v[k] * t + g[k] * h)
}

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = sub(a, b);
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

/// 0x002206c8 FUN_002206c8(machine, s): SetState. `off` = machine offset in `m` (the owner's memory).
/// Returns None for s == state (no change).
pub fn ee_sub_2206c8(m: &mut Mem, off: u32, s: i32, cx: &CarryCx, h: &mut dyn ClassHost, hit: &mut dyn FnMut([f32; 3], [f32; 3]) -> Option<[f32; 3]>) -> Option<CarryOut> {
    let w = |k: u32| off + 4 * k;
    let prev = m.i(w(1));
    if s == prev {
        return None;
    }
    let mut out = CarryOut::default();
    let owner = cx.owner;
    let carried = m.u(w(4));
    if prev == -1 {
        if carried != cx.bentley {
            out.calls.push((0x001cd098, owner, 0x900));
        }
        if h.ee_sub_132b10(carried, CLASS_CARRYABLE) != 0 {
            out.calls.push((0x264, carried, 0));
        }
        // owner velocity +0x190 / +0x194 / +0x198 = 0
        m.set_u(0x190, 0);
        m.set_u(0x194, 0);
        m.set_u(0x198, 0);
    } else if prev == 3 && s != 4 {
        out.slot78.push(cx.bentley);
    }
    m.set_i(w(1), s);
    m.set_f(w(2), cx.t);
    out.calls.push((0x00134320, owner, 0));
    if s == -1 {
        // FUN_00204660(FUN_00204740(owner), owner): face the current heading
        out.calls.push((0x00204660, owner, 0));
        m.set_u(0x1a4, 0);
    } else if s == 7 {
        // FUN_002046b0(owner, +0xe0 - pos): face the target
        let d = sub(m.v3(off + 0xe0), h.ee_sub_1a62e8(owner));
        out.calls.push((0x002046b0, owner, d[1].atan2(d[0]).to_bits() as i32));
        m.set_u(0x1a4, 0);
    } else {
        m.set_v3(0x1e0, [0.0; 3]);
        m.set_u(0x1ec, 0);
        m.set_u(0x1a4, 1);
    }
    let mut anim = 0u32;
    let mut mode = 1;
    match s {
        0 => {
            if (prev as u32).wrapping_sub(1) >= 2 {
                m.set_f(w(0x10), 0.5);
                m.set_f(w(0x13), 0.5);
                out.calls.push((0x001384c0, m.u(w(0x16)), 0));
            }
            anim = m.u(w(0x16));
            mode = 9;
        }
        1 => anim = m.u(w(0x17)),
        2 => anim = m.u(w(0x18)),
        3 => {
            anim = m.u(w(0x19));
            out.slot78.push(owner);
        }
        4 => {
            let arc = m.u(w(0x4b));
            if arc != 0 {
                // +0x154 = 1, +0x160 = 0, vt +0x44 (show)
                out.calls.push((0x44, arc, 1));
            }
            if (prev as u32).wrapping_sub(5) >= 2 {
                m.set_f(w(0x4c), 0.0);
                let tgt = h.ee_sub_1a6f00_to_world(owner as usize - 1, m.v3(off + 0xc0));
                m.set_v3(off + 0xe0, tgt);
                m.set_u(off + 0xec, 0);
                let a = ee_sub_21fb70(m, off, false, cx, h, hit);
                out.arc = a;
            }
            anim = m.u(w(0x26));
            mode = 9;
            m.set_u(w(0x4a), 0);
        }
        5 => anim = m.u(w(0x27)),
        6 => anim = m.u(w(0x28)),
        7 => {
            anim = m.u(w(0x2f));
            if h.ee_sub_132b10(carried, CLASS_CARRYABLE) != 0 {
                out.calls.push((0x00141cd0, owner, cx.carried_15c as i32));
            }
        }
        -1 => {
            if carried != 0 && carried == cx.bentley && cx.bentley_1800 == 0 {
                // throw Bentley: horizontal direction owner -> Bentley (DAT_002a2da0 (1,0,0) below 1e-4)
                let mut d = sub(h.ee_sub_1a62e8(carried), h.ee_sub_1a62e8(owner));
                d[2] = 0.0;
                let l = (d[0] * d[0] + d[1] * d[1]).sqrt();
                let n = if l < 0.0001 { [1.0, 0.0, 0.0] } else { [d[0] / l, d[1] / l, 0.0] };
                let v = [n[0] * 400.0, n[1] * 400.0, 600.0];
                out.calls.push((0x00136530, carried, 1));
                out.throw = Some((carried, v));
            } else if cx.carried_10 == cx.ref_10 {
                if h.ee_sub_132b10(carried, CLASS_CARRYABLE) == 0 {
                    out.calls.push((0x001a58d0, carried, 0xf));
                    out.calls.push((0x001f7bf8, carried, 0.25f32.to_bits() as i32));
                } else {
                    out.calls.push((0x264, carried, 4));
                }
            }
            out.calls.push((0x001cd2c0, owner, 0x900));
            let arc = m.u(w(0x4b));
            if arc != 0 {
                out.calls.push((0x4c, arc, 0));
            }
            m.set_u(w(4), 0);
            return Some(out);
        }
        _ => {}
    }
    // tail: FUN_001f15b8(owner+0x1034), FUN_001305b8(0, 1.0, anim, owner, mode, owner+0x1030), vt +0x2d4
    out.calls.push((0x001f15b8, owner, 0x1034));
    out.calls.push((0x001305b8, anim, mode));
    out.calls.push((0x2d4, owner, 0));
    out.calls.push((0x00134320, owner, 0));
    Some(out)
}

/// 0x0021fb70 FUN_0021fb70(machine, smooth): the throw arc and the aim blend weights.
/// p0 = owner local +0xd0 in the world, v = FUN_001d6848(u, p0, +0xe0, g) with u = clamp((horizontal
/// |+0xe0 - owner| - [7]) / ([8] - [7])); flight time [9] (x 1.35 with bif 138(0x24)) sampled in 0x1d steps;
/// the first blocked segment (FUN_0014cc58, `hit`) gives the landing time (root of the z quadratic nearest to
/// the hit point); [0x4c] follows it at 3/s; points from [0x4c] back to p0. Then the 4 aim weights from the
/// yaw offset / [6] and u (rate limited at 2/s when `smooth`). REPLACED: ClearWorld (box query) is the
/// caller's `hit`; the arc display records (+0x158 / +0x15c, FUN_0021c5f8) are the returned points.
pub fn ee_sub_21fb70(m: &mut Mem, off: u32, smooth: bool, cx: &CarryCx, h: &mut dyn ClassHost, hit: &mut dyn FnMut([f32; 3], [f32; 3]) -> Option<[f32; 3]>) -> Vec<[f32; 3]> {
    let w = |k: u32| off + 4 * k;
    let owner = cx.owner;
    let p0 = h.ee_sub_1a6f00_to_world(owner as usize - 1, m.v3(off + 0xd0));
    let tgt = m.v3(off + 0xe0);
    let g = cx.gravity;
    let pos = h.ee_sub_1a62e8(owner);
    let dh = ((tgt[0] - pos[0]).powi(2) + (tgt[1] - pos[1]).powi(2)).sqrt();
    let r = (dh - m.f(w(7))) / (m.f(w(8)) - m.f(w(7)));
    let u = if r < 0.0 { 0.0 } else { r.min(1.0) };
    let v = FUN_001d6848(u, p0, tgt, g).unwrap_or([0.0; 3]);
    let tf = if cx.bif138 { DAT_002C40E4 * m.f(w(9)) } else { m.f(w(9)) };
    m.set_v3(off + 0xf0, v);
    m.set_u(off + 0xfc, 0);
    let step = tf * 0.034_482_76;
    let mut prev = p0;
    let mut t_hit = 0.0;
    for i in 1..ARC_N as u32 {
        t_hit = i as f32 * step;
        let p = at(p0, v, g, t_hit);
        if let Some(hp) = hit(prev, p) {
            // SolveQuadratic(g.z / 2, v.z, p0.z - hit.z): the root >= 0 nearest the hit point
            t_hit = 0.0;
            let (a, b, c) = (g[2] * 0.5, v[2], p0[2] - hp[2]);
            let mut roots = vec![];
            if a.abs() < 1e-12 {
                if b != 0.0 {
                    roots.push(-c / b);
                }
            } else {
                let disc = b * b - 4.0 * a * c;
                if disc >= 0.0 {
                    let s = disc.sqrt();
                    roots.push((-b - s) / (2.0 * a));
                    roots.push((-b + s) / (2.0 * a));
                }
            }
            let mut best = DAT_002A96A0;
            for t in roots {
                if t >= 0.0 {
                    let d = dist(at(p0, v, g, t), hp);
                    if d < best {
                        best = d;
                        t_hit = t;
                    }
                }
            }
            break;
        }
        prev = p;
    }
    let cur = m.f(w(0x4c));
    let lim = DAT_002C4100 * cx.dt;
    let mut t = cur - lim;
    if t <= t_hit {
        t = if t_hit <= cur + lim { t_hit } else { cur + lim };
    }
    m.set_f(w(0x4c), t);
    let mut pts = vec![];
    if t >= 0.1 {
        let mut k = 0u32;
        let mut tt = t;
        loop {
            k += 1;
            pts.push(at(p0, v, g, tt));
            if k > 0x1c {
                break;
            }
            tt = m.f(w(0x4c)) - k as f32 * step;
            if tt < 0.1 {
                break;
            }
        }
    }
    pts.push(p0);
    // aim blend weights
    let d = sub(tgt, pos);
    let yaw = h.ee_sub_204740(owner as usize - 1);
    let a = crate::engine::player::p5_attach::FUN_0011cf60(d[1].atan2(d[0]) - yaw) / m.f(w(6));
    let mut side = a.clamp(-1.0, 1.0) * 0.5 + 0.5;
    let mut elev = u;
    if smooth {
        let pu = m.f(w(0x2c));
        let ps = if 0.5 < pu { m.f(w(0x23)) / pu } else { m.f(w(0x1d)) / (1.0 - pu) };
        let r = DAT_002C4104 * cx.dt;
        let lim = |x: f32, p: f32| if p - r <= x { if p + r < x { p + r } else { x } } else { p - r };
        elev = lim(u, pu);
        side = lim(side, ps);
    }
    let e1 = 1.0 - elev;
    m.set_f(w(0x23), side * elev);
    m.set_f(w(0x1d), side * e1);
    m.set_f(w(0x20), (1.0 - side) * elev);
    m.set_f(w(0x1a), (1.0 - side) * e1);
    m.set_f(w(0x29), e1);
    m.set_f(w(0x2c), elev);
    pts
}
