//! WP-B4: the remaining class `load` virtuals (vtable +0x74) and the class-6 handler
//! FUN_001c9b08. Stream helpers as in loads.rs: FUN_00117bd0 u8, FUN_00117d00 s8 (compared
//! with -1 by its callers), FUN_00117c20 u16, FUN_00117d50 s16, FUN_00117c80/00117db8 u32/s32,
//! FUN_00117e38 f32, FUN_00117eb0 vec3, FUN_00117ed0 3 x vec3, FUN_00117968 raw bytes.
//! Allocation / derived data (normals, adjacency tables, GS packets) are not stream data; the
//! parsed values are kept so users can compute them.

use super::class::{self, ClassId};
use super::loads::{ee_sub_117fd0, ee_sub_118220, ee_sub_118420, ee_sub_231170, ee_sub_231768, Bsp, Geom, Xform};
use super::objects::{ee_sub_13fb40, LoadData, Lo, ObjectSink, Payload, WorldError};
use super::props::{ee_sub_13f810, Prop, PropValue};
use crate::engine::disc::level_file::memory_stream;
use crate::engine::disc::lz::BinaryInputStream;

type S<'a> = &'a mut BinaryInputStream;

fn f32_(s: S) -> f32 {
    f32::from_bits(s.read_u32())
}
/// FUN_00117eb0
fn vec3(s: S) -> [f32; 3] {
    [f32_(s), f32_(s), f32_(s)]
}
fn props(s: S) -> Result<Vec<Prop>, WorldError> {
    ee_sub_13f810(s).map_err(|e| WorldError::BadField { field: e.field, offset: e.offset })
}
fn raw(s: S, n: usize) -> Vec<u8> {
    let mut v = vec![0u8; n];
    s.read(n, Some(&mut v));
    v
}

/// Loads with no vtable in classes_gen (the ctor stores it from a separate init function):
/// class -> (vtable, vtable +0x74).
pub fn vtable_load(cls: ClassId) -> Option<(u32, u32)> {
    match cls {
        53 => Some((0x002d2e98, 0x001885b8)),  // ctor 0x1116d8 -> FUN_002563e8
        54 => Some((0x002d44c8, 0x001b4888)),  // ctor 0x111700 -> FUN_00256440
        126 => Some((0x002daf30, 0x00212298)), // ctor 0x112be8 -> FUN_00212158
        130 => Some((0x002c9670, 0x00212298)), // ctor 0x112cd0 -> FUN_00211c58
        _ => None,
    }
}

/// The load virtual actually called for `cls` (`load` = classes_gen value). FUN_001885b8 and
/// FUN_001b4888 call FUN_002039c0 (ALO + FUN_00118420 blob) and then only initialise fields,
/// so they are mapped to it for the ALO chain.
pub fn effective_load(cls: ClassId, load: u32) -> u32 {
    let l = if load == 0 { vtable_load(cls).map_or(0, |v| v.1) } else { load };
    match l {
        0x001885b8 | 0x001b4888 => 0x002039c0,
        _ => l,
    }
}

/// FUN_00159240: point/triangle set (allocated when (+0xe4)->+0x10 == 3).
#[derive(Clone, Debug, PartialEq)]
pub struct Shape159240 {
    pub points: Vec<[f32; 3]>,
    /// 3 point indices + f32 (summed into +0x14; the triangle normal is accumulated per point)
    pub tris: Vec<([u16; 3], f32)>,
    pub v: [f32; 3],
}
fn ee_sub_159240(s: S) -> Shape159240 {
    // 0x00159240
    let n = s.read_u16();
    let points = (0..n).map(|_| vec3(s)).collect();
    let m = s.read_u16();
    let tris = (0..m).map(|_| ([s.read_u16(), s.read_u16(), s.read_u16()], f32_(s))).collect();
    Shape159240 { points, tris, v: vec3(s) }
}

/// FUN_002584a8 / FUN_00258f40: u16 n; if n != 0: u8 (+0xcc), n u32 colours (FUN_001ed278,
/// only the first min(n, 32) kept).
#[derive(Clone, Debug, PartialEq)]
pub struct Colors {
    pub flag: u8,
    pub colors: Vec<u32>,
}
fn ee_sub_2584a8(s: S) -> Option<Colors> {
    // 0x002584a8 (0x00258f40 identical)
    let n = s.read_u16();
    if n == 0 {
        return None;
    }
    let flag = s.read_u8();
    Some(Colors { flag, colors: (0..n).map(|_| s.read_u32()).collect() })
}

/// (+0xe4)->+0x10 after the property list: FUN_00258a00 / FUN_00259498 set it only while 0.
/// `set` = setter field, `one` / `two` = fields whose accessors force 1 / 2.
fn kind_after_props(p: &[Prop], set: i16, one: [i16; 2], two: i16) -> i32 {
    let mut k = 0; // FUN_00159118: DAT_002de5b0 = 0
    let mut apply = |v: i32| {
        if k != v && k == 0 {
            k = v;
        }
    };
    for pr in p {
        if pr.field == set {
            if let PropValue::Scalar(b) = &pr.value {
                if b.len() == 4 {
                    apply(i32::from_le_bytes([b[0], b[1], b[2], b[3]]));
                }
            }
        } else if one.contains(&pr.field) {
            apply(1);
        } else if pr.field == two {
            apply(2);
        }
        // set-0 accessors (484 / 537) never change a value
    }
    k
}

/// FUN_00214460 leaf {u16,u16,u16,s16,u8,u8,s32, 3 x vec3}
#[derive(Clone, Debug, PartialEq)]
pub struct Node214460 {
    pub f: f32,
    pub a: Vec<([u16; 3], i16, [u8; 2], i32, [[f32; 3]; 3])>,
    pub b: Vec<(u32, u32)>,
    pub c: Vec<u16>,
    pub d: Vec<u8>,
    pub e: Vec<(u8, i32)>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Seg214460 {
    pub f: f32,
    pub i: i32,
    pub nodes: Vec<Node214460>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Grp214460 {
    /// puVar16[1] items, each with `per` 0x70-byte runtime slots (FUN_002156b8, no stream)
    pub items: u16,
    pub per: u16,
    pub segs: Vec<Seg214460>,
}
fn ee_sub_214460(s: S) -> Vec<Grp214460> {
    // 0x00214460
    let n = s.read_u16();
    (0..n)
        .map(|_| {
            let items = s.read_u16();
            let per = s.read_u16();
            let c = s.read_u16();
            let segs = (0..c)
                .map(|_| {
                    let f = f32_(s);
                    let i = s.read_s32();
                    let d = s.read_u16();
                    let nodes = (0..d)
                        .map(|_| {
                            let f = f32_(s);
                            let na = s.read_u8();
                            let a = (0..na)
                                .map(|_| {
                                    let u = [s.read_u16(), s.read_u16(), s.read_u16()];
                                    let h = s.read_s16();
                                    let b = [s.read_u8(), s.read_u8()];
                                    let w = s.read_s32();
                                    (u, h, b, w, [vec3(s), vec3(s), vec3(s)]) // FUN_00117ed0
                                })
                                .collect();
                            let nb = s.read_u8();
                            let b = (0..nb).map(|_| (s.read_u32(), s.read_u32())).collect();
                            let nc = s.read_u8();
                            let c = (0..nc).map(|_| s.read_u16()).collect();
                            let nd = s.read_u8();
                            let d = raw(s, nd as usize);
                            let ne = s.read_u8();
                            let e = (0..ne).map(|_| (s.read_u8(), s.read_s32())).collect();
                            Node214460 { f, a, b, c, d, e }
                        })
                        .collect();
                    Seg214460 { f, i, nodes }
                })
                .collect();
            Grp214460 { items, per, segs }
        })
        .collect()
}

/// FUN_002149d8 record: u16 slot index, u32 kind, kind data.
#[derive(Clone, Debug, PartialEq)]
pub enum Rec2149d8Data {
    /// 1: u16 n x {u8 k, u8, u8, u8, u8, k x {u32, u8, s16, s16}} (text runs; GS packet built)
    Runs(Vec<([u8; 5], Vec<(u32, u8, i16, i16)>)>),
    /// 2: 3 x vec3 + u16 n x 0x18 raw bytes
    Mat { m: [[f32; 3]; 3], data: Vec<u8> },
    /// 3: u32, u32, f32 x3, s8, s8 k, [k in 0..=1: s32]
    K3 { a: u32, b: u32, f: [f32; 3], c: i8, k: i8, oid: Option<i32> },
    /// 4: u16
    K4(u16),
    /// 6/7: 4 s16 rect + s32 (GS sprite packet built)
    Rect { r: [i16; 4], v: i32 },
    /// 8: nested FUN_00214398 list, u32 n x u32, u32 m x f32
    List { recs: Vec<Rec2149d8>, a: Vec<u32>, f: Vec<f32> },
    /// other kinds: no data
    None,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Rec2149d8 {
    pub index: u16,
    pub kind: u32,
    pub data: Rec2149d8Data,
}
fn ee_sub_2149d8(s: S) -> Rec2149d8 {
    // 0x002149d8
    let index = s.read_u16();
    let kind = s.read_u32();
    let data = match kind {
        1 => {
            let n = s.read_u16();
            Rec2149d8Data::Runs(
                (0..n)
                    .map(|_| {
                        let h = [s.read_u8(), s.read_u8(), s.read_u8(), s.read_u8(), s.read_u8()];
                        let v = (0..h[0]).map(|_| (s.read_u32(), s.read_u8(), s.read_s16(), s.read_s16())).collect();
                        (h, v)
                    })
                    .collect(),
            )
        }
        2 => {
            let m = [vec3(s), vec3(s), vec3(s)];
            let n = s.read_u16();
            Rec2149d8Data::Mat { m, data: raw(s, n as usize * 0x18) }
        }
        3 => {
            let a = s.read_u32();
            let b = s.read_u32();
            let f = [f32_(s), f32_(s), f32_(s)];
            let c = s.read_u8() as i8;
            let k = s.read_u8() as i8;
            let oid = (k == 0 || k == 1).then(|| s.read_s32());
            Rec2149d8Data::K3 { a, b, f, c, k, oid }
        }
        4 => Rec2149d8Data::K4(s.read_u16()),
        6 | 7 => Rec2149d8Data::Rect { r: [s.read_s16(), s.read_s16(), s.read_s16(), s.read_s16()], v: s.read_s32() },
        8 => {
            // DAT_002c40a4 = 1 only changes the allocation tag
            let recs = ee_sub_214398(s);
            let n = s.read_u32();
            let a = (0..n as i32).map(|_| s.read_u32()).collect();
            let m = s.read_u32();
            let f = (0..m as i32).map(|_| f32_(s)).collect();
            Rec2149d8Data::List { recs, a, f }
        }
        _ => Rec2149d8Data::None,
    };
    Rec2149d8 { index, kind, data }
}
/// FUN_00214398: u16 n + n FUN_002149d8 records.
fn ee_sub_214398(s: S) -> Vec<Rec2149d8> {
    // 0x00214398
    let n = s.read_u16();
    (0..n).map(|_| ee_sub_2149d8(s)).collect()
}

/// Class-6 handler entry (FUN_001c9b08 list).
#[derive(Clone, Debug, PartialEq)]
pub struct Override {
    /// mode-1 branch: kind (5 = property list); else branch: oid of the template object
    pub key: i32,
    pub class: i16,
    pub slot: i16,
    pub raw: Vec<u8>,
    /// `raw` parsed as a FUN_0013f810 property list when it is exactly one
    pub props: Option<Vec<Prop>>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum B4Load {
    /// FUN_00185150 (classes 8/9)
    L185150 { xform: Xform, points: Vec<[f32; 3]>, a: Vec<[i16; 3]>, b: Vec<[i16; 3]>, props: Vec<Prop> },
    /// FUN_001e5df8 (10-13) / FUN_001bc088 (14)
    L1e5df8 { xform: Xform, a: i8, b: i8, words: Vec<u8>, p0: Vec<[f32; 3]>, p1: Vec<[f32; 3]>, p2: Vec<[f32; 3]>, props: Vec<Prop> },
    /// FUN_00220fd0 (15); nested objects (FUN_002317e0) are siblings (owner = this object's owner)
    L220fd0 { xform: Xform, geom: Geom, bsp: Bsp, props: Vec<Prop>, nested: Vec<usize> },
    /// FUN_001c03c0 (17)
    L1c03c0 { xform: Xform, props: Vec<Prop>, nodes: Vec<([f32; 3], Vec<u16>)>, tris: Vec<[u16; 3]>, pts: Vec<([f32; 3], [u16; 2])> },
    /// FUN_00258598 (29) / FUN_00259030 (32/33)
    L258598 { xform: Xform, props: Vec<Prop>, kind: i32, shape: Option<Shape159240>, colors: Option<Colors> },
    /// FUN_0015c1e8 (31): FUN_00231170 + nested objects (FUN_002317e0)
    L15c1e8 { xform: Xform, props: Vec<Prop>, nested: Vec<usize> },
    /// FUN_0012eb78 (43), FUN_002035a8 (46), FUN_001bea88 (45): FUN_00231170 only
    L231170 { xform: Xform, props: Vec<Prop> },
    /// FUN_001efac0 (111): nested objects first (owner = this object's owner)
    L1efac0 { nested: Vec<usize>, pairs: Vec<(i32, i32)>, links: Vec<[i32; 4]>, link_f: Vec<(f32, i32)>, props: Vec<Prop> },
    /// FUN_001731b8 (119), FUN_0014a008 (123): FUN_001a5c28 only
    Lo(Vec<Prop>),
    /// FUN_00212298 (126-131)
    L212298 { props: Vec<Prop>, flag: i8, f: f32, w: [u32; 3], recs: Vec<Rec2149d8>, groups: Vec<Grp214460>, blob: Option<Vec<u8>> },
    /// FUN_001624a8 (132)
    L1624a8 { props: Vec<Prop>, blob: Vec<u8> },
    /// FUN_001635f8 (133)
    L1635f8 { props: Vec<Prop>, a: Vec<i32>, b: Vec<i32>, blob: Vec<u8> },
    /// FUN_001c9b08 (class-6 subtree records)
    Class6 {
        template: i32,
        mode: i8,
        bone: i16,
        xform: [u8; 48],
        word: u32,
        /// true = mode-1 branch (object created in place)
        in_place: bool,
        /// FUN_001cb770 found the template in the directory
        template_found: bool,
        overrides: Vec<Override>,
        /// mode-1 branch: s32 len + s16 + len-2 skipped bytes
        trailer: Option<(i32, i16)>,
        /// else branch: the s32 before the nested list, and the nested objects
        nested_word: Option<i32>,
        nested: Vec<usize>,
    },
}

/// Loads ported here (by vtable +0x74 address).
pub fn handles(load: u32) -> bool {
    matches!(
        load,
        0x00185150
            | 0x001e5df8
            | 0x001bc088
            | 0x00220fd0
            | 0x001c03c0
            | 0x00258598
            | 0x00259030
            | 0x0015c1e8
            | 0x0012eb78
            | 0x002035a8
            | 0x001bea88
            | 0x001efac0
            | 0x001731b8
            | 0x0014a008
            | 0x00212298
            | 0x001624a8
            | 0x001635f8
    )
}

/// FUN_0013e3c8 + vtable +0x74 for the [`handles`] loads. The object is pushed before its load
/// (nested lists see it created first), then the payload is checked against the record size.
pub fn create_and_load(w: &mut dyn ObjectSink, s: S, lo: Lo, load: u32, mission: i16) -> Result<usize, WorldError> {
    let start = s.tell();
    let (cls, size, owner, entry) = (lo.class, lo.payload_size, lo.owner, lo.entry);
    let idx = w.push(lo);
    let nested_list = |w: &mut dyn ObjectSink, s: S| ee_sub_13fb40(w, owner, entry, s, mission);
    let pe = |s: &mut BinaryInputStream| props(s);
    let d = match load {
        0x00185150 => {
            // 0x00185150
            let xform = ee_sub_231768(s);
            let n = s.read_s16();
            let points = (0..n.max(0)).map(|_| vec3(s)).collect();
            let m = s.read_s16();
            let a = (0..m.max(0)).map(|_| [s.read_s16(), s.read_s16(), s.read_s16()]).collect();
            let b = (0..m.max(0)).map(|_| [s.read_s16(), s.read_s16(), s.read_s16()]).collect();
            B4Load::L185150 { xform, points, a, b, props: pe(s)? }
        }
        0x001e5df8 | 0x001bc088 => {
            // 0x001e5df8 (0x001bc088 calls it then allocates (n-1)*16 bytes)
            let xform = ee_sub_231768(s);
            let a = s.read_u8() as i8;
            let b = s.read_u8() as i8;
            let n = s.read_u16() as usize;
            let words = raw(s, n * 4);
            let p0 = (0..n).map(|_| vec3(s)).collect();
            let (p1, p2) = if a != 0 { ((0..n).map(|_| vec3(s)).collect(), (0..n).map(|_| vec3(s)).collect()) } else { (Vec::new(), Vec::new()) };
            B4Load::L1e5df8 { xform, a, b, words, p0, p1, p2, props: pe(s)? }
        }
        0x00220fd0 => {
            // 0x00220fd0
            let xform = ee_sub_231768(s);
            let geom = ee_sub_117fd0(s);
            let bsp = ee_sub_118220(s);
            let props = pe(s)?;
            // FUN_002317e0: bone stack push of +0x50, FUN_0013fb40(this+0x10 owner, ...)
            let nested = nested_list(w, s)?;
            B4Load::L220fd0 { xform, geom, bsp, props, nested }
        }
        0x001c03c0 => {
            // 0x001c03c0
            let (xform, props) = ee_sub_231170(s)?;
            let n = s.read_u16();
            let m = s.read_u16();
            let nodes = (0..n)
                .map(|_| {
                    let p = vec3(s);
                    let k = s.read_u8();
                    (p, (0..k).map(|_| s.read_u16()).collect())
                })
                .collect();
            let tris = (0..m).map(|_| [s.read_u16(), s.read_u16(), s.read_u16()]).collect();
            let p = s.read_u16();
            let pts = (0..p).map(|_| ([f32_(s), f32_(s), f32_(s)], [s.read_u16(), s.read_u16()])).collect();
            B4Load::L1c03c0 { xform, props, nodes, tris, pts }
        }
        0x00258598 | 0x00259030 => {
            // 0x00258598 / 0x00259030
            let (xform, props) = ee_sub_231170(s)?;
            let kind = if load == 0x00258598 { kind_after_props(&props, 483, [485, 486], 487) } else { kind_after_props(&props, 536, [538, 539], 540) };
            let shape = (kind == 3).then(|| ee_sub_159240(s));
            let colors = ee_sub_2584a8(s);
            B4Load::L258598 { xform, props, kind, shape, colors }
        }
        0x0015c1e8 => {
            // 0x0015c1e8
            let (xform, props) = ee_sub_231170(s)?;
            let nested = nested_list(w, s)?;
            B4Load::L15c1e8 { xform, props, nested }
        }
        0x0012eb78 | 0x002035a8 | 0x001bea88 => {
            let (xform, props) = ee_sub_231170(s)?;
            B4Load::L231170 { xform, props }
        }
        0x001efac0 => {
            // 0x001efac0: FUN_0013fb40(this+0x10, ...) first
            let nested = nested_list(w, s)?;
            let n = s.read_u8();
            let pairs = (0..n).map(|_| (s.read_s32(), s.read_s32())).collect();
            let m = s.read_u8();
            let mut links = Vec::new();
            let mut link_f = Vec::new();
            for _ in 0..m {
                links.push([s.read_s32(), s.read_s32(), s.read_s32(), s.read_s32()]);
                link_f.push((f32_(s), s.read_s32()));
            }
            B4Load::L1efac0 { nested, pairs, links, link_f, props: pe(s)? }
        }
        0x001731b8 | 0x0014a008 => B4Load::Lo(pe(s)?),
        0x00212298 => {
            // 0x00212298
            let props = pe(s)?;
            let flag = s.read_u8() as i8;
            let f = f32_(s);
            let w3 = [s.read_u32(), s.read_u32(), s.read_u32()];
            let (recs, groups, blob) = if flag == 0 {
                let r = ee_sub_214398(s);
                (r, ee_sub_214460(s), None)
            } else {
                (Vec::new(), Vec::new(), Some(raw(s, 0x40))) // FUN_00118458
            };
            B4Load::L212298 { props, flag, f, w: w3, recs, groups, blob }
        }
        0x001624a8 => {
            let props = pe(s)?;
            B4Load::L1624a8 { props, blob: ee_sub_118420(s) }
        }
        0x001635f8 => {
            let props = pe(s)?;
            let n = s.read_u8();
            let a = (0..n).map(|_| s.read_s32()).collect();
            let m = s.read_u8();
            let b = (0..m).map(|_| s.read_s32()).collect();
            B4Load::L1635f8 { props, a, b, blob: ee_sub_118420(s) }
        }
        _ => return Err(WorldError::BadData { what: "loads_b4 dispatch", offset: start }),
    };
    finish(w, s, idx, start, cls, size, d)
}

fn finish(w: &mut dyn ObjectSink, s: S, idx: usize, start: u32, cls: ClassId, size: i32, d: B4Load) -> Result<usize, WorldError> {
    if s.at_eof() {
        return Err(WorldError::Eof { what: "B4 class load", offset: start });
    }
    let consumed = s.tell() - start;
    if consumed != size as u32 {
        return Err(WorldError::SizeMismatch { class: cls, offset: start, size, consumed });
    }
    w.get_mut(idx).payload = Payload::Loaded(LoadData::B4(Box::new(d)));
    w.after_load(idx); // WP-B5: class 15/31 attached objects
    Ok(idx)
}

/// u16 size + bytes; parsed as a property list when it is exactly one.
fn override_props(s: S, key: i32, class: i16, slot: i16) -> Override {
    let n = s.read_u16() as usize;
    let raw = raw(s, n);
    let mut sub = memory_stream([raw.clone(), vec![0u8; 16]].concat());
    let props = ee_sub_13f810(&mut sub).ok().filter(|_| sub.tell() as usize == n);
    Override { key, class, slot, raw, props }
}

/// FUN_001c9b08(world, owner, class, oid, parent, stream, collector). Mode -1 reads the
/// template's directory entry b (+0x60) via [`ObjectSink::template_b`] (FUN_001cb770).
pub fn ee_sub_1c9b08(
    w: &mut dyn ObjectSink,
    s: S,
    mut lo: Lo,
    mission: i16,
) -> Result<Vec<usize>, WorldError> {
    // 0x001c9b08
    let start = s.tell();
    let (cls, size, owner, entry, lo_oid) = (lo.class, lo.payload_size, lo.owner, lo.entry, lo.oid);
    let template = s.read_s32();
    let mode = s.read_u8() as i8;
    let bone = s.read_s16(); // FUN_0022d6b0(ref, owner, 0)
    let mut xform = [0u8; 48];
    s.read(48, Some(&mut xform)); // FUN_00117f18
    let word = s.read_u32();
    // FUN_001cb770 returns -1 for a template that is not in the directory; the original then
    // reads entry[-1]+0x60 = the 0xc bytes before the bump-allocated entry array (stale heap,
    // FUN_001ae1d0 has no header). The template branch would index entry -1 for real
    // (FUN_001ca370), so the in-place branch is taken here (assumption, see ENGINE_PORT B4).
    // Both branches consume the same bytes: the s32 is the byte size of the nested list.
    let template_b = w.template_b(template);
    let in_place = match mode {
        1 => true,
        -1 => template_b.map_or(true, |b| b != 0),
        _ => false,
    };
    lo.payload = Payload::Handler1c9b08;
    // WP-B5: the template branch creates no object for the record itself; the record is kept
    // as a bookkeeping Lo without a parent (so it is never linked/activated) and is not
    // returned to the collector.
    if !in_place {
        lo.owner = None;
    }
    let idx = w.push(lo);
    let mut collected = Vec::new();
    let mut overrides = Vec::new();
    let (trailer, nested_word, nested);
    if in_place {
        // FUN_0013e3c8 + FUN_002313f0 + FUN_001a6d28: no stream data
        let n = s.read_u8();
        for _ in 0..n {
            let key = s.read_s32();
            let class = s.read_s16();
            let slot = s.read_s16(); // FUN_001ffa40 anchor, -1 = none
            let mut o = override_props(s, key, class, slot);
            // kind 5: props applied only if the object is of `class` (FUN_00132b10)
            if key == 5 && !(class >= 0 && class::ee_sub_15e988(cls, class as ClassId)) {
                o.props = None;
            }
            overrides.push(o);
        }
        let len = s.read_s32();
        let h = s.read_s16();
        s.read((len - 2).max(0) as usize, None); // FUN_001179b8
        trailer = Some((len, h));
        nested_word = None;
        nested = Vec::new();
        collected.push(idx); // collector +0xc (1, &object)
    } else {
        // FUN_001ca370 instantiates the template entry: no stream data (WP-B5)
        let tidx = w.template_index(template);
        collected = w.instantiate_template(idx, tidx, owner, bone, lo_oid, xform);
        let n = s.read_u8() as i32;
        // do-while on the u8 count: a count of 0 is never read as 0 by the loop (n > 0 checked)
        for _ in 0..n {
            let oid = s.read_s32();
            let class = s.read_s16();
            let slot = s.read_s16();
            overrides.push(override_props(s, oid, class, slot));
        }
        trailer = None;
        nested_word = Some(s.read_s32());
        // FUN_0022d670(bone) / FUN_0013fb40(owner, ...) / FUN_0022d698
        nested = ee_sub_13fb40(w, owner, entry, s, mission)?;
        w.template_nested(&nested, xform); // WP-B5: +0xcc / +0x12c per nested object
        collected.extend(&nested); // collector: copy objects, then the nested list
    }
    let d = B4Load::Class6 { template_found: template_b.is_some(), template, mode, bone, xform, word, in_place, overrides, trailer, nested_word, nested };
    finish(w, s, idx, start, cls, size, d)?;
    Ok(collected)
}
