//! WP-Q2: the JT's run-time blend ASEGs (class 0x6e) 1:1.
//!
//! - FUN_00137a30(owner, stride, count, entries, ntables, table_offs, table_counts, &out): the union
//!   of the entry ASEGs' channels (word0 & 0x3fffffff; flag-0x40000000 channels skipped), one blend
//!   curve per channel (kind 5 vector, kind 2 scalar for word0 < 0) whose +0xc array holds every
//!   entry's curve of that channel, +0x10 = FUN_0022ca18 of the bound bone, +8 = the first phase
//!   table (+0xd0). Weights: 1 for the entry holding the first non-NULL ASEG, else 0; then
//!   FUN_001384c0.
//! - FUN_001384c0: select (+0xd4) = last entry with weight > 0.9999 (else -1, FUN_001383f0), playing
//!   ASEGAs' times rescaled to the new length, per phase table FUN_001380f0 (or, without tables,
//!   length = sum of weight * entry length).
//! - FUN_001380f0: per phase: len = sum(weight * entry phase len); start of phase 0 = fmod(circular
//!   mean of the entries' phase-0 offsets / length) * total; next starts chained, wrapped by total.
//! - FUN_00137fb8: per entry, phase offsets = ASEG label times (FUN_0012f990: event list 1 kind 0xb
//!   with the id), lengths = cyclic differences (+ entry length when negative).
//! - FUN_0020b868 (run blends, +0x1320 / +0x1c30): 2 entries stride 0x40, label tables
//!   0x2c3dc0 {0x17d,0x17e} at entry +0x20 and 0x2c3dc8 {0x17f,0x180} at +0x30; channels whose bone
//!   +0x9c is 0x10/0x12/0x14 use the second table, 0x11/0x13/0x15 the first, others none.
//! - FUN_0018f908 (+0x1970 -> +0x1994) and FUN_001a0300 (+0x1ac0 -> +0x1bc0): no phase tables.
//! - FUN_0020bb58(u, +0x1320): weight1 = u (entry 1 present) and weight0 = 1 - u, else weight0 = 1.
//! - FUN_00190520 (jump blend weights): w0 = a*b, w1 = a*(1-b), w2 = 1-a, a/b approach per jtbs
//!   targets with FUN_0011d018 (SMP 0x2bf570 / 0x2bf580); time steered to L*(f+1)/2 at |svt| <= 3.
//! Unported: kind-2 scalar blend channels (they drive no bone), the entry +0x10 root velocity
//! (FUN_0020b868/FUN_001a0300 speed matching). The event time rescale (+0xd8 / +0xa4) is ported (ee_sub_1384c0).

use super::{aseg_of, ee_sub_15e938_below, JtRig};
use crate::engine::anim::aseg::{Aseg, EventData};
use crate::engine::anim::blend::{BlendAseg, BlendEntry, BlendVecCurve, PhaseTable};
use crate::engine::anim::keys::{Segment, V3};
use crate::engine::anim::motion::{ee_sub_22ca18, BoneState};
use crate::engine::camera::math::{Smp, FUN_0011d018};
use crate::engine::gfx::attach::{alo_of, ee_sub_22d6b0, BoneRef};
use crate::engine::world::World;

/// 0x2c3dc0 / 0x2c3dc8: label ids of the two run phase tables (FUN_0020b868).
pub const RUN_LABELS: [[i32; 2]; 2] = [[0x17d, 0x17e], [0x17f, 0x180]];
/// SMP 0x2bf570 (+0x199c) and 0x2bf580 (+0x1998) of FUN_00190520.
pub const SMP_JUMP_A: Smp = [1.0, 0.0, 0.1, 0.0];
pub const SMP_JUMP_B: Smp = [4.0, 0.0, 0.1, 1.7];
/// DAT_002bf590: FUN_00190520 |svt| limit.
pub const JUMP_SVT_MAX: f32 = 3.0;

/// One blend entry: its ASEG (world object), weight (+0), per phase table the (offset, len) pairs.
#[derive(Clone, Debug, Default)]
pub struct JtBlendEntry {
    pub aseg: Option<usize>,
    pub sub_length: f32,
    pub weight: f32,
    pub phases: Vec<Vec<(f32, f32)>>,
}

/// One channel of the blend (+0x60 list) with its kind-5 curve and the bone it drives.
#[derive(Clone, Debug)]
pub struct JtBlendChannel {
    pub key: u32,
    pub table: Option<usize>,
    pub bone: Option<usize>,
    pub curve: BlendVecCurve,
}

/// Class-0x6e blend ASEG built at run time.
#[derive(Clone, Debug)]
pub struct JtBlend {
    pub entries: Vec<JtBlendEntry>,
    /// per phase table (+0xd0, 0xc each): label count and the computed (start, len) phases
    pub tables: Vec<Vec<(f32, f32)>>,
    /// +0x74
    pub length: f32,
    /// +0xd4
    pub select: i32,
    pub channels: Vec<JtBlendChannel>,
    /// +0xa4 array (stride 0x10, float 0 = time): copy of the reference ASEG's timed events (+0xa0)
    pub events: Vec<Option<f32>>,
    /// +0xd8: reference ASEG (first entry with events 0x98 / 0xa8 / 0xa0): (+0x74 length, its +0xa4 times)
    pub ev_ref: Option<(f32, Vec<Option<f32>>)>,
}

/// FUN_0012f990(aseg, id): time of the label event (list 1, kind 0xb, s32 id) or 0.
pub fn ee_sub_12f990(a: &Aseg, id: i32) -> f32 {
    a.events1.iter().find(|e| e.kind == 0xb && e.data == EventData::S32(id)).and_then(|e| e.time).unwrap_or(0.0)
}

/// FUN_00137fb8 for one table: (offset, len) per label of entry ASEG `a`.
pub fn ee_sub_137fb8(a: &Aseg, ids: &[i32]) -> Vec<(f32, f32)> {
    let offs: Vec<f32> = ids.iter().map(|&id| ee_sub_12f990(a, id)).collect();
    let n = offs.len();
    (0..n)
        .map(|k| {
            let mut l = offs[(k + 1) % n] - offs[k];
            if l < 0.0 {
                l += a.length;
            }
            (offs[k], l)
        })
        .collect()
}

/// FUN_0011d898: fmodf, + y when negative.
fn fmod_pos(x: f32, y: f32) -> f32 {
    let r = x % y;
    if r < 0.0 {
        r + y
    } else {
        r
    }
}

/// FUN_001380f0(blend, table, count, phases, &length): phases of table `tb`, returns (phases, total).
pub fn ee_sub_1380f0(entries: &[JtBlendEntry], tb: usize, count: usize) -> (Vec<(f32, f32)>, f32) {
    let ph = |e: &JtBlendEntry, i: usize| e.phases.get(tb).and_then(|p| p.get(i)).copied().unwrap_or((0.0, 0.0));
    let mut out = vec![(0.0f32, 0.0f32); count];
    let mut total = 0.0;
    for (i, o) in out.iter_mut().enumerate() {
        o.1 = entries.iter().filter(|e| e.weight != 0.0).map(|e| ph(e, i).1 * e.weight).sum();
        total += o.1;
    }
    // circular mean of the phase-0 offsets (the first weighted entry counts unweighted)
    let mut acc = 0.0f32;
    let mut first: Option<f32> = None;
    for e in entries.iter().filter(|e| e.weight != 0.0) {
        let f = ph(e, 0).0 / e.sub_length;
        match first {
            None => {
                first = Some(f);
                acc = f;
            }
            Some(f4) => {
                let mut d = f - f4;
                if d <= -0.5 {
                    d += 1.0;
                } else if 0.5 < d {
                    d -= 1.0;
                }
                acc += d * e.weight;
            }
        }
    }
    if count > 0 {
        out[0].0 = fmod_pos(acc, 1.0) * total;
        for i in 1..count {
            let mut s = out[i - 1].0 + out[i - 1].1;
            if total <= s {
                s -= total;
            }
            out[i].0 = s;
        }
    }
    (out, total)
}

impl JtBlend {
    /// FUN_001384c0. `times`: the playing ASEGAs' local times (+0x50 chain), kept at the same
    /// normalised position.
    pub fn ee_sub_1384c0(&mut self, times: &mut [f32]) {
        self.select = self.entries.iter().rposition(|e| e.weight > 0.9999).map_or(-1, |i| i as i32);
        for t in times.iter_mut() {
            *t /= self.length;
        }
        if self.tables.is_empty() {
            self.length = self.entries.iter().filter(|e| e.weight != 0.0).map(|e| e.sub_length * e.weight).sum();
        } else {
            for tb in 0..self.tables.len() {
                let (p, total) = ee_sub_1380f0(&self.entries, tb, self.tables[tb].len());
                self.tables[tb] = p;
                self.length = total;
            }
        }
        for t in times.iter_mut() {
            *t *= self.length;
        }
        // event times (+0xa4) rescaled from the reference ASEG (+0xd8) to the new length
        if let Some((rl, rt)) = &self.ev_ref {
            let k = self.length / rl;
            for (e, r) in self.events.iter_mut().zip(rt) {
                if let (Some(e), Some(r)) = (e.as_mut(), r) {
                    *e = k * r;
                }
            }
        }
        let blend = |tb: Option<usize>| BlendAseg {
            length: self.length,
            entries: self
                .entries
                .iter()
                .map(|e| BlendEntry { weight: e.weight, sub_length: e.sub_length, phase: tb.and_then(|tb| e.phases.get(tb).cloned()).unwrap_or_default() })
                .collect(),
            select: self.select,
        };
        for c in &mut self.channels {
            c.curve.blend = blend(c.table);
            c.curve.phases = c.table.map(|tb| PhaseTable { phases: self.tables[tb].clone() });
        }
    }

    /// Channel segments at blend time `t` (curve slot 1 = FUN_00138660) and their bones.
    pub fn segments(&self, t: f32) -> (Vec<Option<Segment>>, Vec<Option<usize>>) {
        (self.channels.iter().map(|c| c.curve.ee_sub_138660(t)).collect(), self.channels.iter().map(|c| c.bone).collect())
    }

    /// FUN_0020bb58(u): run blend weights, then FUN_001384c0.
    pub fn ee_sub_20bb58(&mut self, u: f32, times: &mut [f32]) {
        if self.entries.get(1).is_some_and(|e| e.aseg.is_some()) {
            self.entries[1].weight = u;
            self.entries[0].weight = 1.0 - u;
        } else {
            self.entries[0].weight = 1.0;
        }
        self.ee_sub_1384c0(times);
    }
}

/// FUN_0012fe58 target bone of channel word `key` (oid & 0xfffff, kind << 20): rig bone and the
/// bone's +0x9c word and FUN_0022ca18 value.
fn bind_key(w: &World, rig: &JtRig, key: u32) -> (Option<usize>, i32, V3) {
    let oid = ((key << 12) as i32) >> 12;
    let kind = ((key >> 20) & 0x3ff) as i16;
    let Some(t) = ee_sub_15e938_below(w, rig.jt, oid) else { return (None, 0, [0.0; 3]) };
    match ee_sub_22d6b0(w, kind, w.objects[t].owner, Some(t), BoneRef::None) {
        BoneRef::Bone(o, k) => {
            let word = alo_of(w, o).and_then(|a| a.base.bones.get(k)).map_or(0, |b| b.word);
            let rb = rig.index.get(&(o, k)).copied();
            let v = rb.map_or([0.0; 3], |i| {
                let b = &rig.pose.bones[i];
                ee_sub_22ca18(&BoneState::new(b.kind, rig.pose.rest[i]))
            });
            (rb, word, v)
        }
        _ => (None, 0, [0.0; 3]),
    }
}

/// FUN_00137a30 over `entries` (ASEG objects) with `labels` phase tables (FUN_00137fb8 already
/// applied per table). None when no entry has an ASEG (*out = 0).
pub fn ee_sub_137a30(w: &World, rig: &JtRig, asegs: &[Option<usize>], labels: &[[i32; 2]]) -> Option<JtBlend> {
    let first = asegs.iter().position(|a| a.is_some())?;
    let mut entries: Vec<JtBlendEntry> = asegs
        .iter()
        .map(|&o| {
            let a = o.and_then(|o| aseg_of(w, o));
            JtBlendEntry {
                aseg: o.filter(|_| a.is_some()),
                sub_length: a.map_or(0.0, |a| a.length),
                weight: 0.0,
                phases: labels.iter().map(|ids| a.map(|a| ee_sub_137fb8(a, ids)).unwrap_or_default()).collect(),
            }
        })
        .collect();
    let mut channels: Vec<JtBlendChannel> = vec![];
    let n = asegs.len();
    for (i, e) in entries.iter().enumerate() {
        let Some(a) = e.aseg.and_then(|o| aseg_of(w, o)) else { continue };
        for c in &a.channels {
            // scalar (kind 2) and flagged channels drive no bone
            if c.flag || c.scalar.is_some() {
                continue;
            }
            let key = (c.oid & 0xfffff) | ((c.kind as u32 & 0x3ff) << 20);
            let ch = match channels.iter().position(|x| x.key == key) {
                Some(j) => &mut channels[j],
                None => {
                    let (bone, _, default) = bind_key(w, rig, key);
                    channels.push(JtBlendChannel {
                        key,
                        table: (!labels.is_empty()).then_some(0),
                        bone,
                        curve: BlendVecCurve {
                            blend: BlendAseg { length: 0.0, entries: vec![], select: -1 },
                            phases: None,
                            children: vec![None; n],
                            default,
                        },
                    });
                    channels.last_mut().unwrap()
                }
            };
            ch.curve.children[i] = c.vec.clone();
        }
    }
    for (i, e) in entries.iter_mut().enumerate() {
        e.weight = if i == first { 1.0 } else { 0.0 };
    }
    // FUN_00137a30: reference ASEG = first entry with events 0x98 / 0xa8 / 0xa0 (+0xd8), its +0xa0 list copied to +0xa4
    let ref_a = asegs.iter().filter_map(|&o| o.and_then(|o| aseg_of(w, o))).find(|a| !a.events0.is_empty() || !a.events2.is_empty() || !a.events1.is_empty());
    let events: Vec<Option<f32>> = ref_a.map(|a| a.events1.iter().map(|e| e.time).collect()).unwrap_or_default();
    let ev_ref = ref_a.map(|a| (a.length, events.clone()));
    let mut b = JtBlend { entries, tables: labels.iter().map(|l| vec![(0.0, 0.0); l.len()]).collect(), length: 1.0, select: -1, channels, events, ev_ref };
    b.ee_sub_1384c0(&mut []);
    Some(b)
}

/// FUN_0020b868: run blend of a {jt, out, entries[2] stride 0x40} block.
pub fn ee_sub_20b868(w: &World, rig: &JtRig, asegs: [Option<usize>; 2]) -> Option<JtBlend> {
    asegs[0]?;
    let mut b = ee_sub_137a30(w, rig, &asegs, &RUN_LABELS)?;
    for c in &mut b.channels {
        let (_, word, _) = bind_key(w, rig, c.key);
        c.table = match word {
            0x10 | 0x12 | 0x14 => Some(1),
            0x11 | 0x13 | 0x15 => Some(0),
            _ => None,
        };
    }
    b.ee_sub_1384c0(&mut []);
    Some(b)
}

/// JT +0x1998 / +0x199c of FUN_00190520.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct JumpWeights {
    pub a: f32,
    pub b: f32,
}

impl Default for JumpWeights {
    /// 0x18c250 sets both to 1 when the jump starts.
    fn default() -> Self {
        JumpWeights { a: 1.0, b: 1.0 }
    }
}

/// FUN_00190520 weight + time part for jtbs (0x18d0) at blend time `t`. `f` = the clamped lean
/// (angle / DAT_002bf58c). Returns the new svt (None for jtbs 3: no update).
pub fn ee_sub_190520(b: &mut JtBlend, jw: &mut JumpWeights, jtbs: i32, dt: f32, f: f32, t: &mut f32) -> Option<f32> {
    let (ta, tb) = match jtbs {
        3 => return None,
        1 => (0.0, if 0.5 < jw.b { 1.0 } else { 0.0 }),
        2 => (1.0, 0.0),
        0 => (1.0, 1.0),
        // other jtbs keep the targets of the original's uninitialised registers: hold
        _ => (jw.a, jw.b),
    };
    jw.a = FUN_0011d018(jw.a, ta, dt, &SMP_JUMP_A).0;
    jw.b = FUN_0011d018(jw.b, tb, dt, &SMP_JUMP_B).0;
    if b.entries.len() >= 3 {
        b.entries[0].weight = jw.a * jw.b;
        b.entries[2].weight = 1.0 - jw.a;
        b.entries[1].weight = jw.a * (1.0 - jw.b);
    }
    b.ee_sub_1384c0(std::slice::from_mut(t));
    let f = f.clamp(-1.0, 1.0);
    Some(crate::engine::camera::math::FUN_0011cfe0((b.length * (f + 1.0) * 0.5 - *t) / dt, JUMP_SVT_MAX))
}
