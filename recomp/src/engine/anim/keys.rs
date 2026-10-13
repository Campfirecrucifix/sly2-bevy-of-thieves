//! WP-D: animation channel keys (Sly 1 ACP/ACR/ACS/ACG family, Sly 2 layout).
//!
//! Two key-object factories are read by the ASEG load (FUN_0012f2a8):
//! - FUN_0012b3b0(kind) = vector curves (vtables 0x2ca9a0.. 0x2cb5a8; slot 0 (+0xc) = stream load,
//!   slot 1 = segment setup into the channel state, slot 2 = direct evaluation).
//! - FUN_0012cb98(kind) = scalar curves (vtables 0x2ca8a0 / 0x2ca8c0 / 0x2cb588).
//!
//! Times are stored in the stream as u16 frames and converted with `* 0.016666668` (1/60 s).
//! The channel state (param_2 of slot 1, offsets +0x80..+0xfc) is [`Segment`].

use crate::engine::disc::lz::BinaryInputStream;

type S<'a> = &'a mut BinaryInputStream;
pub type V3 = [f32; 3];

/// Frame -> seconds as in every key loader (`(float)u16 * 0.016666668`).
pub fn frame_time(f: u16) -> f32 {
    f as f32 * 0.016666668
}
fn f32_(s: S) -> f32 {
    f32::from_bits(s.read_u32())
}
/// FUN_00117eb0
fn vec3(s: S) -> V3 {
    [f32_(s), f32_(s), f32_(s)]
}

/// 0x2ca978 key (0x30 bytes): pos +0, t +0xc, in-tangent +0x10, out-tangent +0x20.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KeyVB {
    pub t: f32,
    pub pos: V3,
    pub tan_in: V3,
    pub tan_out: V3,
}
/// 0x2ca950 key (0x20 bytes): pos +0, t +0xc, tangent +0x10 (same in and out).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KeyV {
    pub t: f32,
    pub pos: V3,
    pub tan: V3,
}

/// FUN_0012b3b0 kinds (vector curves).
#[derive(Clone, Debug, PartialEq)]
pub enum VecCurve {
    /// 0: 0x2ca9a0, FUN_0012b4f8: 12-byte constant (+4)
    Const(V3),
    /// 1: 0x2ca978, FUN_0012b5e0: u16 n x {u16 frame, vec3 pos, vec3 in, vec3 out}
    Bez(Vec<KeyVB>),
    /// 2: 0x2ca950, FUN_0012ba98: u16 n x {u16 frame, vec3 pos, vec3 tangent}
    Bez1(Vec<KeyV>),
    /// 3: 0x2ca928, FUN_0012bf30: vec3 default (+0x20), u16 n frames, u8 axis mask, per axis
    /// (bit i) n x {value, in, out}
    Axis3 { default: V3, times: Vec<f32>, axes: [Option<Vec<[f32; 3]>>; 3] },
    /// 4: 0x2ca900, FUN_0012c570: as 3 with n x {value, tangent}
    Axis2 { default: V3, times: Vec<f32>, axes: [Option<Vec<[f32; 2]>>; 3] },
    /// 5: 0x2cb5a8, load FUN_0012b4f0 is `jr ra` (no stream data); setup FUN_00138660 = blend::BlendVecCurve (run-time bound)
    Kind5,
}

/// FUN_0012cb98 kinds (scalar curves).
#[derive(Clone, Debug, PartialEq)]
pub enum ScalarCurve {
    /// 0: 0x2ca8a0, FUN_0012cc68: f32 (+4); value getter FUN_0012cc98
    Const(f32),
    /// 1: 0x2ca8c0, FUN_0012ccb8: u16 n x {u16 frame, f32 value, f32 in, f32 out}
    Bez(Vec<[f32; 4]>),
    /// 2: 0x2cb588, load FUN_0012cc60 is `jr ra` (no stream data); evaluator FUN_00138ba0 = blend::BlendScalarCurve
    Kind2,
}

/// FUN_0012b3b0 + vtable slot 0. None = kind outside 0..5 (the original returns NULL and
/// then calls through it: crash).
pub fn load_vec_curve(kind: i8, s: S) -> Option<VecCurve> {
    Some(match kind {
        0 => VecCurve::Const(vec3(s)), // 0x0012b4f8: FUN_00117968(s, 0xc, this+4)
        1 => {
            // 0x0012b5e0
            let n = s.read_u16();
            VecCurve::Bez(
                (0..n)
                    .map(|_| {
                        let t = frame_time(s.read_u16());
                        KeyVB { t, pos: vec3(s), tan_in: vec3(s), tan_out: vec3(s) }
                    })
                    .collect(),
            )
        }
        2 => {
            // 0x0012ba98
            let n = s.read_u16();
            VecCurve::Bez1(
                (0..n)
                    .map(|_| {
                        let t = frame_time(s.read_u16());
                        KeyV { t, pos: vec3(s), tan: vec3(s) }
                    })
                    .collect(),
            )
        }
        3 | 4 => {
            // 0x0012bf30 / 0x0012c570
            let default = vec3(s);
            let n = s.read_u16();
            let times = (0..n).map(|_| frame_time(s.read_u16())).collect();
            let mask = s.read_u8();
            if kind == 3 {
                let axes = std::array::from_fn(|i| (mask & (1 << i) != 0).then(|| (0..n).map(|_| [f32_(s), f32_(s), f32_(s)]).collect()));
                VecCurve::Axis3 { default, times, axes }
            } else {
                let axes = std::array::from_fn(|i| (mask & (1 << i) != 0).then(|| (0..n).map(|_| [f32_(s), f32_(s)]).collect()));
                VecCurve::Axis2 { default, times, axes }
            }
        }
        5 => VecCurve::Kind5, // 0x0012b4f0
        _ => return None,
    })
}

/// FUN_0012cb98 + vtable +0xc.
pub fn load_scalar_curve(kind: i8, s: S) -> Option<ScalarCurve> {
    Some(match kind {
        0 => ScalarCurve::Const(f32_(s)), // 0x0012cc68
        1 => {
            // 0x0012ccb8
            let n = s.read_u16();
            ScalarCurve::Bez((0..n).map(|_| [frame_time(s.read_u16()), f32_(s), f32_(s), f32_(s)]).collect())
        }
        2 => ScalarCurve::Kind2, // 0x0012cc60
        _ => return None,
    })
}

// ---------------------------------------------------------------- math

/// FUN_00117120: cubic Hermite over a segment of length `dt`, `u` = time into the segment,
/// `rate` scales the derivatives. Returns (value, velocity, acceleration) as the original's
/// three optional outputs. Tangents are per second (multiplied by dt). u/dt > 1 extrapolates.
pub fn ee_sub_117120(dt: f32, u: f32, rate: f32, p0: V3, v0: V3, p1: V3, v1: V3) -> (V3, V3, V3) {
    // 0x00117120
    let t = u / dt;
    let s = 1.0 - t;
    let a = t * s * s;
    let b = t * t * s;
    let h00 = s * s * s + a * 3.0;
    let h10 = a * dt;
    let h11 = -b * dt;
    let h01 = t * t * t + b * 3.0;
    let pos = std::array::from_fn(|i| p0[i] * h00 + v0[i] * h10 + v1[i] * h11 + p1[i] * h01);
    let d = (t * (6.0 - t * 6.0)) / dt;
    let e = 1.0 - t * (4.0 - t * 3.0);
    let f = t * (t * 3.0 + -2.0);
    let vel = std::array::from_fn(|i| ((p1[i] - p0[i]) * d + v0[i] * e + v1[i] * f) * rate);
    let k = (rate * rate) / dt;
    let g = (6.0 - t * 12.0) / dt;
    let h = t * 6.0 - 4.0;
    let j = t * 6.0 - 2.0;
    let acc = std::array::from_fn(|i| ((p1[i] - p0[i]) * g + v0[i] * h + v1[i] * j) * k);
    (pos, vel, acc)
}

/// FUN_00117030: scalar FUN_00117120 (value, velocity).
pub fn ee_sub_117030(dt: f32, u: f32, rate: f32, p0: f32, v0: f32, p1: f32, v1: f32) -> (f32, f32) {
    // 0x00117030
    let t = u / dt;
    let s = 1.0 - t;
    let a = t * s * s;
    let b = t * t * s;
    let val = (((s * s * s + a * 3.0) * p0 + a * dt * v0) - b * dt * v1) + (t * t * t + b * 3.0) * p1;
    let vel = (((t * s * 6.0) / dt) * (p1 - p0) + (1.0 - t * (4.0 - t * 3.0)) * v0 + t * (t * 3.0 + -2.0) * v1) * rate;
    (val, vel)
}

/// FUN_0011d4d0: relative float compare, |a-b| / max(1,|a|) < eps.
pub fn ee_sub_11d4d0(a: f32, b: f32, eps: f32) -> bool {
    // 0x0011d4d0
    let fa = a.abs();
    (a - b).abs() / if fa < 1.0 { 1.0 } else { fa } < eps
}
/// eps 0x38d1b717 used by every segment setup
pub const SEG_EPS: f32 = 1.0e-4;

fn v3_eq(a: V3, b: V3) -> bool {
    // x, then prot3w (y), then pextuw (z)
    ee_sub_11d4d0(a[0], b[0], SEG_EPS) && ee_sub_11d4d0(a[1], b[1], SEG_EPS) && ee_sub_11d4d0(a[2], b[2], SEG_EPS)
}

/// Key search shared by FUN_0012b6d8 (stride 0x30), FUN_0012bb70 (0x20), FUN_0012c0e0 /
/// FUN_0012c708 (f32 array): returns k in [1, n-1] with times[k-1] <= t < times[k] (clamped).
/// Quirk kept: with n < 2 it still returns 1 (the original then reads key 1 past the end;
/// callers here clamp that read to the last key).
pub fn key_search(t: f32, times: impl Fn(usize) -> f32, n: usize) -> usize {
    // 0x0012b6d8
    let last = n as i32 - 1;
    let mut k = 1i32;
    if 1 < last && times(1) <= t {
        k = 2;
        while k < last && times(k as usize) <= t {
            k += 1;
        }
    }
    k as usize
}

/// Channel segment state (+0x80 t0, +0x84 t1, +0x9a mode, +0xc0 p0, +0xd0 v0, +0xe0 p1, +0xf0 v1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Segment {
    pub t0: f32,
    pub t1: f32,
    pub p0: V3,
    pub v0: V3,
    pub p1: V3,
    pub v1: V3,
    /// +0x9a: 1 = Hermite, 2 = constant (p0==p1 and both tangents 0)
    pub mode: u16,
    /// kind 0 / empty kind 3,4 rewire the channel's time pointers (+0x88 -> +0x80, +0x8c ->
    /// +0x84): the channel then reads t = t0 = 0 and rate = t1 = 1.
    pub rewired_time: bool,
}

fn const_seg(p: Option<V3>) -> Segment {
    let p = p.unwrap_or([0.0; 3]);
    Segment { t0: 0.0, t1: 1.0, p0: p, v0: [0.0; 3], p1: p, v1: [0.0; 3], mode: 2, rewired_time: false }
}

fn finish(t0: f32, t1: f32, p0: V3, v0: V3, p1: V3, v1: V3, rewired_time: bool) -> Segment {
    // tail of 0x0012b738 / 0x0012bbd0 / 0x0012c140 / 0x0012c768
    let zero = [0.0; 3]; // DAT_002a2dd0
    let c = v3_eq(p0, p1) && v3_eq(zero, v0) && v3_eq(zero, v1);
    Segment { t0, t1, p0, v0, p1, v1, mode: if c { 2 } else { 1 }, rewired_time }
}

impl VecCurve {
    /// Vtable slot 1: set up the segment containing time `t`. None for kind 5 (run-time
    /// bound blend: blend::BlendVecCurve::ee_sub_138660).
    pub fn segment(&self, t: f32) -> Option<Segment> {
        Some(match self {
            VecCurve::Const(v) => {
                // 0x0012b520
                Segment { t0: 0.0, t1: 1.0, p0: *v, v0: [0.0; 3], p1: *v, v1: [0.0; 3], mode: 2, rewired_time: true }
            }
            // with < 2 keys the original reads past the array; treated as a constant here
            VecCurve::Bez(k) if k.len() < 2 => const_seg(k.first().map(|x| x.pos)),
            VecCurve::Bez1(k) if k.len() < 2 => const_seg(k.first().map(|x| x.pos)),
            VecCurve::Bez(k) => {
                // 0x0012b738
                let i = key_search(t, |i| k[i].t, k.len());
                let (a, b) = (&k[i - 1], &k[i]);
                finish(a.t, b.t, a.pos, a.tan_out, b.pos, b.tan_in, false)
            }
            VecCurve::Bez1(k) => {
                // 0x0012bbd0 (search FUN_0012bb70)
                let i = key_search(t, |i| k[i].t, k.len());
                let (a, b) = (&k[i - 1], &k[i]);
                finish(a.t, b.t, a.pos, a.tan, b.pos, b.tan, false)
            }
            VecCurve::Axis3 { default, times, axes } => axis_segment(t, default, times, |i, k| axes[i].as_ref().map(|a| (a[k - 1][0], a[k - 1][2], a[k][0], a[k][1]))),
            VecCurve::Axis2 { default, times, axes } => axis_segment(t, default, times, |i, k| axes[i].as_ref().map(|a| (a[k - 1][0], a[k - 1][1], a[k][0], a[k][1]))),
            VecCurve::Kind5 => return None,
        })
    }

    /// Value at `t` with `rate` = the segment value as the motion code reads it (mode 1 ->
    /// FUN_00117120 value, mode 2/3/4 -> p0; see motion.rs for FUN_0022cdc0).
    pub fn eval(&self, t: f32, rate: f32) -> Option<V3> {
        let s = self.segment(t)?;
        Some(eval_segment(&s, if s.rewired_time { s.t0 } else { t }, rate))
    }
}

/// 0x0012c140 / 0x0012c768: per-axis curves with shared times.
fn axis_segment(t: f32, default: &V3, times: &[f32], get: impl Fn(usize, usize) -> Option<(f32, f32, f32, f32)>) -> Segment {
    let n = times.len();
    if n == 0 {
        // count 0: t0 = 0, t1 = 1, time pointers rewired. The original then indexes the axis
        // arrays with an uninitialised k; empty curves with a mask bit are not in the data, so
        // every axis takes the default here.
        return finish(0.0, 1.0, *default, [0.0; 3], *default, [0.0; 3], true);
    }
    // n == 1: the original reads key 1 past the end; axes take the default here (divergence)
    let k = if n > 1 { key_search(t, |i| times[i], n) } else { 0 };
    let (mut p0, mut v0, mut p1, mut v1) = ([0.0; 3], [0.0; 3], [0.0; 3], [0.0; 3]);
    for i in 0..3 {
        match if n > 1 { get(i, k) } else { None } {
            Some((a, b, c, d)) => {
                p0[i] = a;
                v0[i] = b;
                p1[i] = c;
                v1[i] = d;
            }
            None => {
                p0[i] = default[i];
                p1[i] = default[i];
            }
        }
    }
    let (t0, t1) = if n > 1 { (times[k - 1], times[k]) } else { (times[0], times[0]) };
    finish(t0, t1, p0, v0, p1, v1, false)
}

/// FUN_0022cdc0 (case 6/7) value of a set-up segment at time `t`.
pub fn eval_segment(s: &Segment, t: f32, rate: f32) -> V3 {
    match s.mode {
        1 => ee_sub_117120(s.t1 - s.t0, t - s.t0, rate, s.p0, s.v0, s.p1, s.v1).0,
        _ => s.p0,
    }
}

impl KeyVB {
    /// FUN_0012b9d0 (kind 1 slot 2): direct evaluation (value, velocity) at `t`.
    pub fn eval(keys: &[KeyVB], t: f32, rate: f32) -> (V3, V3) {
        // 0x0012b9d0
        if keys.len() < 2 {
            return (keys.first().map(|k| k.pos).unwrap_or([0.0; 3]), [0.0; 3]);
        }
        let k = key_search(t, |i| keys[i].t, keys.len());
        let (a, b) = (&keys[k - 1], &keys[k]);
        let (p, v, _) = ee_sub_117120(b.t - a.t, t - a.t, rate, a.pos, a.tan_out, b.pos, b.tan_in);
        (p, v)
    }
}

impl ScalarCurve {
    /// Scalar evaluators: FUN_0012cc98 (const) / FUN_0012cda0 (Hermite). None for kind 2
    /// (run-time bound: blend::BlendScalarCurve::ee_sub_138ba0).
    pub fn eval(&self, t: f32, rate: f32) -> Option<(f32, f32)> {
        match self {
            ScalarCurve::Const(v) => Some((*v, 0.0)), // 0x0012cc98: value only
            ScalarCurve::Bez(k) => {
                // 0x0012cda0: its own search uses `t < key.t` and runs up to n (not n-1);
                // t >= last key time therefore selects k = n and reads one key past the end.
                // That read is clamped to the last segment here (documented divergence).
                let n = k.len();
                if n < 2 {
                    return Some((k.first().map(|x| x[1]).unwrap_or(0.0), 0.0));
                }
                let mut i = 1usize;
                if !(t < k[1][0]) {
                    i = 2;
                    while i < n && !(t < k[i][0]) {
                        i += 1;
                    }
                }
                let i = i.min(n - 1);
                let (a, b) = (&k[i - 1], &k[i]);
                Some(ee_sub_117030(b[0] - a[0], t - a[0], rate, a[1], a[3], b[1], b[2]))
            }
            ScalarCurve::Kind2 => None,
        }
    }
}
