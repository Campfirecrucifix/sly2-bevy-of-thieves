//! WP-D2: blended curves = vector curve kind 5 (vtable 0x2cb5a8, setup FUN_00138660) and scalar
//! curve kind 2 (vtable 0x2cb588, evaluator FUN_00138ba0). Neither reads stream data (their
//! loads are `jr ra`); they are bound at run time to a class-110 ASEG (blend segment, object
//! size 0xe0) and evaluate the same channel of every weighted sub-segment.
//!
//! Curve object layout (param_1): +4 blend ASEG, +8 optional phase table, +0xc child curve
//! array (one per blend entry, NULL = default), +0x10 default value.
//! Blend ASEG: +0x74 length, +0xc0 entry stride, +0xc4 entry count, +0xc8 entries
//! ({+0 f32 weight, +4 sub ASEG* (its +0x74 = sub length), ...}), +0xd4 selected entry (<0 =
//! blend all). Phase table: +0 byte offset of the per-entry phase pairs inside an entry, +4
//! count, +8 array of {f32 start, f32 len}; per entry and phase {f32 offset, f32 rate}.

use super::keys::{ee_sub_117120, ScalarCurve, Segment, VecCurve, V3};

/// Phase pairs of the blend's phase table (+8): `{start, len}` per phase.
#[derive(Clone, Debug, PartialEq)]
pub struct PhaseTable {
    pub phases: Vec<(f32, f32)>,
}

/// One blend entry (stride +0xc0).
#[derive(Clone, Debug, PartialEq)]
pub struct BlendEntry {
    /// +0: weight (0 = skipped)
    pub weight: f32,
    /// sub ASEG +0x74
    pub sub_length: f32,
    /// per phase {offset, rate} (entry + phase_table+0 + idx*8); empty without a phase table
    pub phase: Vec<(f32, f32)>,
}

/// Class-110 blend ASEG runtime view.
#[derive(Clone, Debug, PartialEq)]
pub struct BlendAseg {
    /// +0x74
    pub length: f32,
    /// +0xc8 / +0xc4
    pub entries: Vec<BlendEntry>,
    /// +0xd4: >= 0 = only this entry's curve is used (no blending)
    pub select: i32,
}

/// FUN_00138370(t, length, count, phases, &local): first phase with len != 0 whose
/// `t - start` (wrapped by +length when negative) is < len + 1e-4. Returns (index, local);
/// index == count when none matches (local then unset: None).
pub fn ee_sub_138370(t: f32, length: f32, phases: &[(f32, f32)]) -> (usize, Option<f32>) {
    // 0x00138370
    for (i, &(start, len)) in phases.iter().enumerate() {
        if len != 0.0 {
            let mut d = t - start;
            if d < 0.0 {
                d += length;
            }
            if d < len + 0.0001 {
                return (i, Some(d));
            }
        }
    }
    (phases.len(), None)
}

/// Sub-segment time and rate for entry `e` at normalised position `u` (shared by
/// FUN_00138660 / FUN_00138ba0). `phase` = (index, phase length) when a table exists.
fn sub_time(e: &BlendEntry, u: f32, phase: Option<(usize, f32)>) -> Option<(f32, f32)> {
    match phase {
        None => Some((u * e.sub_length, 1.0)),
        Some((idx, plen)) => {
            // quirk kept: entries without the phase pair read garbage in the original
            let &(off, rate) = e.phase.get(idx)?;
            let mut t = off + u * rate;
            if e.sub_length < t {
                t -= e.sub_length;
            }
            Some((t, rate / plen))
        }
    }
}

/// Vector blend curve (kind 5).
#[derive(Clone, Debug, PartialEq)]
pub struct BlendVecCurve {
    pub blend: BlendAseg,
    pub phases: Option<PhaseTable>,
    /// +0xc: per entry, the same channel's curve in that sub-segment
    pub children: Vec<Option<VecCurve>>,
    /// +0x10
    pub default: V3,
}

impl BlendVecCurve {
    /// FUN_00138660 (vtable slot 1): segment setup at time `t` (= **(state+0x88)).
    /// Blending: mode 4 with p0 = sum(w * pos), v0 = sum(w * vel) (VU0 vmadd with the weight
    /// broadcast). Selected entry: mode 2 constant default (time pointers rewired) when its
    /// curve is NULL, else the child's own setup. None where the original reads
    /// uninitialised data (no phase matches; kind-5 child inside a blend).
    pub fn ee_sub_138660(&self, t: f32) -> Option<Segment> {
        // 0x00138660
        let b = &self.blend;
        if b.select >= 0 {
            return match self.children.get(b.select as usize)? {
                None => Some(Segment { t0: 0.0, t1: 1.0, p0: self.default, v0: [0.0; 3], p1: self.default, v1: [0.0; 3], mode: 2, rewired_time: true }),
                Some(c) => c.segment(t),
            };
        }
        let (u, phase) = match &self.phases {
            None => (t / b.length, None),
            Some(p) => {
                let (i, local) = ee_sub_138370(t, b.length, &p.phases);
                let plen = p.phases.get(i)?.1;
                (local? / plen, Some((i, plen)))
            }
        };
        let (mut pos, mut vel) = ([0.0f32; 3], [0.0f32; 3]);
        for (i, e) in b.entries.iter().enumerate() {
            if e.weight == 0.0 {
                continue;
            }
            let (p, v) = match self.children.get(i).and_then(|c| c.as_ref()) {
                None => (self.default, [0.0; 3]),
                Some(c) => {
                    let (st, rate) = sub_time(e, u, phase)?;
                    // without a phase table the rate is sub_length / blend length
                    let rate = if phase.is_none() { e.sub_length / b.length } else { rate };
                    let s = c.segment(st)?;
                    let (p, v, _) = ee_sub_117120(s.t1 - s.t0, st - s.t0, rate, s.p0, s.v0, s.p1, s.v1);
                    (p, v)
                }
            };
            for k in 0..3 {
                pos[k] += p[k] * e.weight;
                vel[k] += v[k] * e.weight;
            }
        }
        // +0x9a = 4, +0xc0 = pos, +0xd0 = vel; other fields untouched (zero here)
        Some(Segment { t0: 0.0, t1: 0.0, p0: pos, v0: vel, p1: [0.0; 3], v1: [0.0; 3], mode: 4, rewired_time: false })
    }
}

/// Scalar blend curve (kind 2).
#[derive(Clone, Debug, PartialEq)]
pub struct BlendScalarCurve {
    pub blend: BlendAseg,
    pub phases: Option<PhaseTable>,
    pub children: Vec<Option<ScalarCurve>>,
    /// +0x10
    pub default: f32,
}

impl BlendScalarCurve {
    /// FUN_00138ba0(t, rate, this, &value, &velocity): weighted sum of the children's values
    /// and velocities. Unlike kind 5 there is no `select` path and the caller's rate scales
    /// the sub rate. None where the original reads uninitialised data.
    pub fn ee_sub_138ba0(&self, t: f32, rate: f32) -> Option<(f32, f32)> {
        // 0x00138ba0
        let b = &self.blend;
        let (u, phase) = match &self.phases {
            None => (t / b.length, None),
            Some(p) => {
                let (i, local) = ee_sub_138370(t, b.length, &p.phases);
                let plen = p.phases.get(i)?.1;
                (local? / plen, Some((i, plen)))
            }
        };
        let (mut val, mut vel) = (0.0f32, 0.0f32);
        for (i, e) in b.entries.iter().enumerate() {
            if e.weight == 0.0 {
                continue;
            }
            let (v, d) = match self.children.get(i).and_then(|c| c.as_ref()) {
                None => (self.default, 0.0),
                Some(c) => {
                    let (st, r) = sub_time(e, u, phase)?;
                    let r = if phase.is_none() { rate * e.sub_length / b.length } else { rate * r };
                    c.eval(st, r)?
                }
            };
            val += e.weight * v;
            vel += e.weight * d;
        }
        Some((val, vel))
    }
}
