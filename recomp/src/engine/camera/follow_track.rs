//! PART-3: the camera-track part (track +0x164 mode bits 1|2) of C2db858 update 0x00221dd8, on the EE object
//! memory of the class-15 track (`FollowWorld::mem`). Wires the ported track helpers 0x00221ce8 (track
//! parameter update), 0x00220dd8 (focus through the track), 0x00221408 (track frame), 0x002215c0 (camera
//! offset on the track); 0x001a6318 / 0x001a62e8 / 0x001a6fc0 go through `QMem::call` (host-routed).
//! The struct-side policy (`CplcyFollow`) is mirrored into a stack frame for the memory-side helpers:
//! o +0x10 track, o +0x14 params (the track's current +0x118 entry when set), CPDEFI copy.
#![allow(non_snake_case)]

use super::cm::Cpdefi;
use super::follow::FollowParams;
use super::math::*;
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem};

/// Track fields read by 0x00221dd8 (offsets in the class-15 object).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TrackCtx {
    pub t: u32,
    /// +0x164 mode bits (1 track A, 2 track B, 4 free/manual allowed)
    pub grf: u32,
    /// +0x168 / +0x16c
    pub i10c: i32,
    pub i108: i32,
    /// +0x10c path, +300 ring, path +0xe0 / +0xe8
    pub path: u32,
    pub ring: u32,
    pub path_e0: i32,
    pub path_e8: i32,
    /// +0x120 polar, +0x130 track-yaw slew limit, +0x138 time base
    pub polar: bool,
    pub yaw_lim: f32,
    pub t138: f32,
    /// +0x170..+0x17c gain ramps
    pub f170: f32,
    pub f174: f32,
    pub f178: f32,
    pub f17c: f32,
    /// +0x180 normal mode override (-1 = CPDEFI +0x3c), +0x18c no collision, +0x190 look-at object
    pub f180: u32,
    pub f18c: u32,
    pub f190: u32,
    /// +0x1a0..+0x1a8 speed polynomial, +0x1b0/+0x1b4 its clamp
    pub poly: [f32; 3],
    pub lo: f32,
    pub hi: f32,
    /// (yaw, pitch, dist) of the mode-1 / mode-2 camera points (fStack_140.. / fStack_134..)
    pub a: [f32; 3],
    pub b: [f32; 3],
    /// 0x00221408 out_dir (track tangent, auStack_210)
    pub dir: V4,
}

fn v3(v: V4) -> V3 {
    [v[0], v[1], v[2]]
}

fn write_cpdefi<M: QMem + ?Sized>(m: &mut M, a: u32, c: &Cpdefi) {
    for i in 0..4 {
        m.w32(a + 4 * i as u32, c.grfcpd[i]);
    }
    m.wv(a + 0x10, c.pos_base);
    m.wv(a + 0x20, c.v_base);
    m.wf(a + 0x30, c.rad_home);
    m.wf(a + 0x34, c.s_adjust);
    m.wf(a + 0x38, c.t_move_last);
    m.wf(a + 0x3c, c.unk_3c);
    m.wv(a + 0x40, c.pos_adjust);
}

/// Stack frame of the memory-side calls.
pub struct Frame {
    pub sp: u32,
    pub o: u32,
    pub cp: u32,
    pub out: u32,
    pub l: u32,
    pub d: u32,
    pub p1: u32,
    pub prm: u32,
}

const FRAME: u32 = 0x140;

/// 0x00221dd8 head with a track: read +0x164/+0x168/+0x16c, 0x00221ce8(o, cpdefi), then the focus
/// 0x00220dd8(params, cpdefi, out, track). Returns the context, the frame (pop with `end`) and the target.
pub fn begin<M: QMem + ?Sized>(m: &mut M, t: u32, params: &mut FollowParams, c: &Cpdefi) -> (TrackCtx, Frame, V3) {
    let sp = stack_push(m, FRAME);
    let f = Frame { sp, o: sp, cp: sp + 0x20, out: sp + 0x70, l: sp + 0x80, d: sp + 0x90, p1: sp + 0xa0, prm: sp + 0xb0 };
    // o +0x14 = the track's current params entry (+0x118, set by 0x00221818) else a copy of ours
    let cur = m.r32(t + 0x118);
    let prm = if cur != 0 {
        let mut w = [0u32; 36];
        for (i, x) in w.iter_mut().enumerate() {
            *x = m.r32(cur + 4 * i as u32);
        }
        *params = FollowParams::from_words(&w);
        cur
    } else {
        for (i, x) in params.to_words().iter().enumerate() {
            m.w32(f.prm + 4 * i as u32, *x);
        }
        f.prm
    };
    let f = Frame { prm, ..f };
    m.w32(f.o + 0x10, t);
    m.w32(f.o + 0x14, prm);
    write_cpdefi(m, f.cp, c);
    let rf = |m: &mut M, o: u32| m.rf(t + o);
    let path = m.r32(t + 0x10c);
    let mut x = TrackCtx {
        t,
        grf: m.r32(t + 0x164),
        i10c: m.r32(t + 0x168) as i32,
        i108: m.r32(t + 0x16c) as i32,
        path,
        ring: m.r32(t + 300),
        path_e0: if path != 0 { m.r32(path + 0xe0) as i32 } else { 0 },
        path_e8: if path != 0 { m.r32(path + 0xe8) as i32 } else { 0 },
        polar: m.r32(t + 0x120) != 0,
        yaw_lim: rf(m, 0x130),
        t138: rf(m, 0x138),
        f170: rf(m, 0x170),
        f174: rf(m, 0x174),
        f178: rf(m, 0x178),
        f17c: rf(m, 0x17c),
        f180: m.r32(t + 0x180),
        f18c: m.r32(t + 0x18c),
        f190: m.r32(t + 0x190),
        poly: [rf(m, 0x1a0), rf(m, 0x1a4), rf(m, 0x1a8)],
        lo: rf(m, 0x1b0),
        hi: rf(m, 0x1b4),
        ..Default::default()
    };
    crate::engine::classes::q::hard4::ee_sub_221ce8(m, f.o, f.cp);
    crate::engine::classes::q::q_part2::ee_sub_220dd8(m, prm, f.cp, f.out, t);
    let target = v3(m.rv(f.out));
    x.dir = [0.0; 4];
    (x, f, target)
}

/// 0x00221dd8 track angles ((uVar8 & 3) != 0): 0x00221408 frame of the focus; polar tracks wrap the local
/// yaw towards +0x4c within +0x130 (not on the first update); the mode-1 / mode-2 camera points
/// 0x002215c0(+0x140, params +4, track, local, params +0x10, params +0x40, 1|2) -> (yaw, pitch, dist).
pub fn angles<M: QMem + ?Sized>(m: &mut M, x: &mut TrackCtx, f: &Frame, focus: V3, first: bool, track_yaw: &mut f32) {
    if x.grf & 3 == 0 {
        return;
    }
    let q = [focus[0], focus[1], focus[2], 0.0];
    crate::engine::classes::q::q_part1::ee_sub_221408(m, x.t, q, f.l, f.d);
    x.dir = m.rv(f.d);
    if x.polar {
        let l = m.rv(f.l);
        let (a, b, c) = FUN_0011ef08(v3(l));
        let mut a = a;
        if !first {
            let d = FUN_0011cfe0(FUN_0011cf60(*track_yaw - a), x.yaw_lim);
            a += d;
        }
        m.wv(f.l, [a, b, c, l[3]]);
        *track_yaw = a;
    }
    let u = m.rf(x.t + 0x140);
    let du = m.rf(f.prm + 4);
    for (bit, mode) in [(1u32, 1u32), (2, 2)] {
        if x.grf & bit != 0 {
            crate::engine::classes::q::q_part1::ee_sub_2215c0(m, u, du, x.t, f.l, f.prm + 0x10, f.prm + 0x40, mode, f.p1);
            let p = v3(m.rv(f.p1));
            let (y, pi, d) = FUN_0011efb0([p[0] - focus[0], p[1] - focus[1], p[2] - focus[2]]);
            if mode == 1 {
                x.a = [y, pi, d];
            } else {
                x.b = [y, pi, d];
            }
        }
    }
}

/// +0x190 look-at: velocity = dir(0x001a62e8(+0x190) - CPDEFI +0x10) * clamp(poly(|d|), +0x1b0, +0x1b4).
pub fn look_vel<M: QMem + ?Sized>(m: &mut M, x: &TrackCtx, c: &Cpdefi) -> V3 {
    let (_, p) = m.call(0x001a_62e8, &[x.f190], &[]);
    let d = [p[0] - c.pos_base[0], p[1] - c.pos_base[1], p[2] - c.pos_base[2]];
    let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    let n = if l < 0.0001 { [0.0; 3] } else { [d[0] / l, d[1] / l, d[2] / l] };
    let s = x.poly[0] + l * (x.poly[1] + l * x.poly[2]);
    let mut k = x.lo;
    if x.lo <= s {
        k = s;
        if x.hi < s {
            k = x.hi;
        }
    }
    [n[0] * k, n[1] * k, n[2] * k]
}

/// mode 4 without path/ring and not the JT's +0x528 track: weight from the focus heading (0x001a6318 rows,
/// xy, normalised) against the track tangent in world space (0x001a6fc0(track, 0, dir)): f = h.t,
/// w = f^2 (3 - 2|f|); returns (w, use mode-2 point).
pub fn heading_weight<M: QMem + ?Sized>(m: &mut M, x: &TrackCtx, pso: u32) -> (f32, bool) {
    let (rows, _) = m.call(0x001a_6318, &[pso], &[]);
    let r = m.rv(rows);
    let h = [r[0], r[1], 0.0];
    let l = (h[0] * h[0] + h[1] * h[1]).sqrt();
    let h = if l < 0.0001 { [0.0, 0.0, 1.0] } else { [h[0] / l, h[1] / l, 0.0] };
    let (_, t) = m.call(0x001a_6fc0, &[x.t, 0], &[x.dir]);
    let f = h[0] * t[0] + h[1] * t[1] + h[2] * t[2];
    let w = f * f * (3.0 - (f.abs() + f.abs()));
    (w, !(x.grf & 2 == 0 || 0.0 < f))
}

pub fn end<M: QMem + ?Sized>(m: &mut M, _f: Frame) {
    stack_pop(m, FRAME);
}
