//! WP-D: animation segment (Sly 1 ASEG) load = class 109/110 `load` FUN_0012f2a8, and the
//! ALO-wrapper load virtuals of the animated actor classes 55, 80-83, 85, 91-96, 98, 103.

use super::keys::{load_scalar_curve, load_vec_curve, ScalarCurve, VecCurve};
use crate::engine::disc::lz::BinaryInputStream;
use crate::engine::world::alo::AloTail;
use crate::engine::world::objects::WorldError;
use crate::engine::world::props::{ee_sub_13f810, Prop};

type S<'a> = &'a mut BinaryInputStream;

fn f32_(s: S) -> f32 {
    f32::from_bits(s.read_u32())
}
fn s8(s: S) -> i8 {
    s.read_u8() as i8
}

/// One FUN_0012f2a8 channel (8 bytes at +0x60, count +0x5c). Word 0 packs
/// `oid & 0xfffff | (kind & 0x3ff) << 20 | flag << 30 | scalar << 31`; word 1 = the curve.
#[derive(Clone, Debug, PartialEq)]
pub struct Channel {
    /// s32 & 0xfffff: target object id
    pub oid: u32,
    /// s16 & 0x3ff: channel target kind (matched against the bone kind at +0x98)
    pub kind: u16,
    /// s8 != -1: FUN_0012b3b0 curve (bit 31 cleared)
    pub vec: Option<VecCurve>,
    /// s8 != -1: FUN_0012cb98 curve; overwrites word 1 and sets bit 31 (a vector curve read
    /// before it is leaked; both kept here)
    pub scalar: Option<ScalarCurve>,
    /// u8 != 0 -> bit 30
    pub flag: bool,
}

/// FUN_0012f2a8 second list (0xc bytes at +0x68, count +0x64).
#[derive(Clone, Debug, PartialEq)]
pub struct ScalarChannel {
    pub b: u8,
    pub word: u32,
    pub curve: ScalarCurve,
}

/// FUN_0012f590 event (0x10 bytes): +0 time (only for list 1: s16 frames / 60), +4 kind,
/// +8/+0xc payload.
#[derive(Clone, Debug, PartialEq)]
pub struct Event {
    pub time: Option<f32>,
    pub kind: u8,
    pub data: EventData,
}

#[derive(Clone, Debug, PartialEq)]
pub enum EventData {
    None,
    /// 0: s16, 3 x s32, 3 x f32 (0x1c-byte block)
    K0 { s: i16, w: [i32; 3], f: [f32; 3] },
    /// 1, 2, 21: s16 (+8); 3: s16 frames (+0xc, seconds) then s16 (+8)
    S16 { a: i16, time2: Option<f32> },
    /// 4, 11, 12, 22: s32
    S32(i32),
    /// 5: s16 frames (seconds, +8) + f32 (+0xc)
    TimeF { t: f32, f: f32 },
    /// 7, 8: s32 + f32
    S32F(i32, f32),
    /// 13-16: s32, s32
    S32x2(i32, i32),
    /// 17: 3 x s32 + f32 (0x10-byte block)
    K17([i32; 3], f32),
    /// 20: s32, s32, f32 (0xc-byte block)
    K20(i32, i32, f32),
    /// 24: 3 x FUN_00117fb0 (8 bytes each) into a 0x18-byte block
    K24([[u8; 8]; 3]),
}

/// FUN_0012f590(this, stream, with_time, &count, &array)
pub fn ee_sub_12f590(s: S, with_time: bool) -> Vec<Event> {
    // 0x0012f590
    let n = s.read_u8();
    (0..n)
        .map(|_| {
            let kind = s.read_u8();
            let time = with_time.then(|| s.read_s16() as f32 * 0.016666668);
            let data = match kind {
                0 => {
                    let sv = s.read_s16();
                    let w = [s.read_s32(), s.read_s32(), s.read_s32()];
                    // stored order +0x10 <- 1st, +0x14 <- 2nd... read order kept
                    let f = [f32_(s), f32_(s), f32_(s)];
                    EventData::K0 { s: sv, w, f }
                }
                1 | 2 | 21 => EventData::S16 { a: s.read_s16(), time2: None },
                3 => {
                    let t = s.read_s16() as f32 * 0.016666668;
                    EventData::S16 { a: s.read_s16(), time2: Some(t) }
                }
                4 | 11 | 12 | 22 => EventData::S32(s.read_s32()),
                5 => {
                    let t = s.read_s16() as f32 * 0.016666668;
                    EventData::TimeF { t, f: f32_(s) }
                }
                7 | 8 => {
                    let a = s.read_s32();
                    EventData::S32F(a, f32_(s))
                }
                13..=16 => {
                    let a = s.read_s32();
                    EventData::S32x2(a, s.read_s32())
                }
                17 => {
                    let w = [s.read_s32(), s.read_s32(), s.read_s32()];
                    EventData::K17(w, f32_(s))
                }
                20 => {
                    let a = s.read_s32();
                    let b = s.read_s32();
                    EventData::K20(a, b, f32_(s))
                }
                24 => {
                    let mut r = [[0u8; 8]; 3];
                    for x in &mut r {
                        s.read(8, Some(x));
                    }
                    EventData::K24(r)
                }
                _ => EventData::None,
            };
            Event { time, kind, data }
        })
        .collect()
}

/// Class 109/110 object (0xc0/0xe0 bytes): Sly 1 ASEG.
#[derive(Clone, Debug, PartialEq)]
pub struct Aseg {
    /// +0x74: f32 (segment length in seconds)
    pub length: f32,
    pub channels: Vec<Channel>,
    pub scalars: Vec<ScalarChannel>,
    /// +0x98/+0x9c (no time)
    pub events0: Vec<Event>,
    /// +0xa0/+0xa4 (timed)
    pub events1: Vec<Event>,
    /// +0xa8/+0xac (no time)
    pub events2: Vec<Event>,
    pub props: Vec<Prop>,
}

fn bad(what: &'static str, s: S) -> WorldError {
    WorldError::BadData { what, offset: s.tell() }
}

/// FUN_0012f2a8: ASEG stream load (vtable +0x74 of classes 109 and 110).
pub fn ee_sub_12f2a8(s: S) -> Result<Aseg, WorldError> {
    // 0x0012f2a8
    let length = f32_(s);
    let n = s.read_s16();
    let mut channels = Vec::new();
    for _ in 0..n.max(0) {
        let oid = s.read_s32() as u32 & 0xfffff;
        let kind = s.read_s16() as u16 & 0x3ff;
        let a = s8(s);
        let vec = if a != -1 { Some(load_vec_curve(a, s).ok_or_else(|| bad("FUN_0012b3b0 kind", s))?) } else { None };
        let b = s8(s);
        let scalar = if b != -1 { Some(load_scalar_curve(b, s).ok_or_else(|| bad("FUN_0012cb98 kind", s))?) } else { None };
        let flag = s.read_u8() != 0;
        channels.push(Channel { oid, kind, vec, scalar, flag });
    }
    let m = s.read_u8();
    let mut scalars = Vec::new();
    for _ in 0..m {
        let b = s.read_u8();
        let word = s.read_u32();
        let k = s8(s);
        let curve = load_scalar_curve(k, s).ok_or_else(|| bad("FUN_0012cb98 kind", s))?;
        scalars.push(ScalarChannel { b, word, curve });
    }
    let events0 = ee_sub_12f590(s, false);
    let events1 = ee_sub_12f590(s, true);
    let events2 = ee_sub_12f590(s, false);
    let props = ee_sub_13f810(s).map_err(|e| WorldError::BadField { field: e.field, offset: e.offset })?;
    Ok(Aseg { length, channels, scalars, events0, events1, events2, props })
}

impl Aseg {
    /// ASEG +0x78 (rate): FUN_0012f248 default 1, overridden by the property list (field 109 = +0x78, e.g.
    /// 0.8 on the EP2 guard walk ASEG oid 416; PCSX2 rec10 +0x48 of its ASEGA = 0.8)
    pub fn f78(&self) -> f32 {
        use crate::engine::world::props::PropValue;
        self.props.iter().rev().find_map(|p| match &p.value {
            PropValue::Scalar(v) if p.field == 109 && v.len() >= 4 => Some(f32::from_le_bytes([v[0], v[1], v[2], v[3]])),
            _ => None,
        }).unwrap_or(1.0)
    }
    /// (channels, keys over all curves)
    pub fn stats(&self) -> (usize, usize) {
        let vk = |c: &VecCurve| match c {
            VecCurve::Const(_) => 1,
            VecCurve::Bez(k) => k.len(),
            VecCurve::Bez1(k) => k.len(),
            VecCurve::Axis3 { times, .. } | VecCurve::Axis2 { times, .. } => times.len(),
            VecCurve::Kind5 => 0,
        };
        let sk = |c: &ScalarCurve| match c {
            ScalarCurve::Const(_) => 1,
            ScalarCurve::Bez(k) => k.len(),
            ScalarCurve::Kind2 => 0,
        };
        let keys = self.channels.iter().map(|c| c.vec.as_ref().map_or(0, vk) + c.scalar.as_ref().map_or(0, sk)).sum::<usize>()
            + self.scalars.iter().map(|c| sk(&c.curve)).sum::<usize>();
        (self.channels.len() + self.scalars.len(), keys)
    }
}

/// Load virtuals of the anim-WP classes, by vtable +0x74 address.
pub const LOAD_ASEG: u32 = 0x0012f2a8;

/// ALO-wrapper load virtuals: FUN_001f31e0 (or FUN_002039c0 = FUN_001f31e0 + FUN_00118420)
/// followed by work that reads no stream data (allocations FUN_001ae*, FUN_001a64e8 defaults,
/// FUN_002303a0, FUN_00148738, FUN_0013c440, FUN_001c7a08, FUN_001b1bf0 = derived state).
/// The stream layout is therefore exactly [`crate::engine::world::alo`]'s chain + children.
pub fn alo_wrapper_tail(load: u32) -> Option<AloTail> {
    match load {
        // class 55: FUN_002039c0 + FUN_001a64e8(this, 0x76, 0x2bc390)
        0x00134770 => Some(AloTail::Blob),
        // 80-82: FUN_001b1e00; 83: FUN_00148c08; 85: FUN_001de478; 91-93,95: FUN_001c4ef8;
        // 94: FUN_001c76d0 (-> FUN_001c4ef8); 96: FUN_001b0ca8; 98: FUN_001baba0; 103: FUN_0013be58
        0x001b1e00 | 0x00148c08 | 0x001de478 | 0x001c4ef8 | 0x001c76d0 | 0x001b0ca8 | 0x001baba0 | 0x0013be58 => Some(AloTail::None),
        _ => None,
    }
}
