//! WP-B2: class `load` virtuals (vtable +0x74) that do not go through the ALO chain
//! (FUN_001f31e0). Stream helpers: FUN_00117bd0/00117d00 u8, FUN_00117c20 u16,
//! FUN_00117d50 s16, FUN_00117c80/00117db8 u32/s32, FUN_00117e38 f32, FUN_00117eb0 vec3 (12 bytes).
//! Allocation (FUN_001ae1d0/2a0) and VU0 derived data (normals, bounding spheres) are not
//! stream data; the parsed values are kept so the derived fields can be computed by users.

use super::objects::{LoadData, WorldError};
use super::props::{ee_sub_13f810, Prop};
use crate::engine::disc::lz::BinaryInputStream;

fn f32_(s: &mut BinaryInputStream) -> f32 {
    f32::from_bits(s.read_u32())
}
/// FUN_00117eb0
fn vec3(s: &mut BinaryInputStream) -> [f32; 3] {
    [f32_(s), f32_(s), f32_(s)]
}

fn props(s: &mut BinaryInputStream) -> Result<Vec<Prop>, WorldError> {
    ee_sub_13f810(s).map_err(|e| WorldError::BadField { field: e.field, offset: e.offset })
}

/// FUN_00231768: s16 (vtable +0x18c -> +0x50), FUN_00117f18 4x3 matrix (+0x60), u32 (FUN_001a6d28).
#[derive(Clone, Debug, PartialEq)]
pub struct Xform {
    pub s: i16,
    pub xform: [u8; 48],
    pub word: u32,
}
pub fn ee_sub_231768(s: &mut BinaryInputStream) -> Xform {
    // 0x00231768
    let sv = s.read_s16();
    let mut xform = [0u8; 48];
    s.read(48, Some(&mut xform));
    Xform { s: sv, xform, word: s.read_u32() }
}

/// One FUN_00117fd0 surface (0x20 bytes): 3 vertex indices (+0x1c = first), 4 floats read and
/// discarded (the plane is recomputed by FUN_00119308), u8 n edges of 3 u16.
#[derive(Clone, Debug, PartialEq)]
pub struct Surface {
    pub verts: [u16; 3],
    pub discarded: [f32; 4],
    /// (u16, u16, u16 surface link) per edge record (+0, +2, +6 of the 0x10-byte record)
    pub edges: Vec<[u16; 3]>,
}

/// FUN_00117fd0: collision geometry (+0 vertex count, +4 vertices, +8/+0xc surfaces,
/// +0x10/+0x14 edge record pool).
#[derive(Clone, Debug, PartialEq)]
pub struct Geom {
    pub verts: Vec<[f32; 3]>,
    pub surfaces: Vec<Surface>,
    /// u16 read into +0x10: size of the edge record pool
    pub edge_pool: u16,
}
pub fn ee_sub_117fd0(s: &mut BinaryInputStream) -> Geom {
    // 0x00117fd0
    let nv = s.read_u16();
    let verts = (0..nv).map(|_| vec3(s)).collect();
    let ns = s.read_u16();
    let edge_pool = s.read_u16();
    let surfaces = (0..ns)
        .map(|_| {
            let verts = [s.read_u16(), s.read_u16(), s.read_u16()];
            let discarded = [f32_(s), f32_(s), f32_(s), f32_(s)];
            let n = s.read_u8();
            let edges = (0..n).map(|_| [s.read_u16(), s.read_u16(), s.read_u16()]).collect();
            Surface { verts, discarded, edges }
        })
        .collect();
    Geom { verts, surfaces, edge_pool }
}

/// One FUN_00118220 node (0xc bytes): surface index (0xffff = leaf DAT_002bbaf0) and two
/// child node indices (-1 = none).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BspNode {
    pub surface: u16,
    pub children: Option<[i32; 2]>,
}
/// FUN_00118220: BSP over the FUN_00117fd0 surfaces.
#[derive(Clone, Debug, PartialEq)]
pub struct Bsp {
    pub a: u32,
    pub b: u32,
    pub nodes: Vec<BspNode>,
}
pub fn ee_sub_118220(s: &mut BinaryInputStream) -> Bsp {
    // 0x00118220
    let n = s.read_u32() as i32;
    let a = s.read_u32();
    let b = s.read_u32();
    let nodes = (0..n.max(0))
        .map(|_| {
            let surface = s.read_u16();
            let children = (surface != 0xffff).then(|| [s.read_s32(), s.read_s32()]);
            BspNode { surface, children }
        })
        .collect();
    Bsp { a, b, nodes }
}

/// FUN_00118420: u16 n, then n bytes skipped (FUN_00117968 with no destination).
pub fn ee_sub_118420(s: &mut BinaryInputStream) -> Vec<u8> {
    // 0x00118420
    let n = s.read_u16() as usize;
    let mut v = vec![0u8; n];
    s.read(n, Some(&mut v));
    v
}

/// FUN_00228250: 64-byte record; kept (FUN_001ae238 copy) unless words 1 and 15 are equal.
pub fn ee_sub_228250(s: &mut BinaryInputStream) -> Option<[u8; 64]> {
    // 0x00228250
    let b = s.read_64();
    (b[4..8] != b[60..64]).then_some(b)
}

pub fn ee_sub_231170(s: &mut BinaryInputStream) -> Result<(Xform, Vec<Prop>), WorldError> {
    // 0x00231170
    let x = ee_sub_231768(s);
    Ok((x, props(s)?))
}

/// FUN_00227bb8: FUN_00231170 + geometry (+0xf0) + BSP (+0xe0); bounding sphere +0x110/+0x120
/// derived (FUN_0011f288/FUN_0011f2b8).
#[derive(Clone, Debug, PartialEq)]
pub struct Solid {
    pub xform: Xform,
    pub props: Vec<Prop>,
    pub geom: Geom,
    pub bsp: Bsp,
}
pub fn ee_sub_227bb8(s: &mut BinaryInputStream) -> Result<Solid, WorldError> {
    // 0x00227bb8
    let (xform, props) = ee_sub_231170(s)?;
    let geom = ee_sub_117fd0(s);
    let bsp = ee_sub_118220(s);
    Ok(Solid { xform, props, geom, bsp })
}

/// Load virtuals ported here, by vtable +0x74 address (None = not ported).
pub fn load(addr: u32, s: &mut BinaryInputStream) -> Option<Result<LoadData, WorldError>> {
    Some(match addr {
        // FUN_001dfb10: heap setup + FUN_001a5c28
        0x001dfb10 => props(s).map(LoadData::Lo),
        // FUN_00227380: FUN_00231768, u32 +0xe0, geometry +0xf8, BSP +0xe8, property list
        0x00227380 => (|| {
            let xform = ee_sub_231768(s);
            let word = s.read_u32();
            let geom = ee_sub_117fd0(s);
            let bsp = ee_sub_118220(s);
            Ok(LoadData::L227380 { xform, word, geom, bsp, props: props(s)? })
        })(),
        // FUN_00228140 / FUN_00228720: FUN_00231170 + FUN_00118420
        0x00228140 | 0x00228720 => ee_sub_231170(s).map(|(xform, props)| LoadData::L231170Blob { xform, props, blob: ee_sub_118420(s) }),
        // FUN_00227bb8; FUN_00149358 = FUN_00227bb8 + FUN_00148738 (no stream data)
        0x00227bb8 | 0x00149358 => ee_sub_227bb8(s).map(LoadData::Solid),
        // FUN_002282b0: FUN_00227bb8, FUN_00228250 (+0x144), u8 n, n x FUN_00228250 (+0x14c)
        0x002282b0 => (|| {
            let solid = ee_sub_227bb8(s)?;
            let first = ee_sub_228250(s);
            let n = s.read_u8();
            let list = (0..n).map(|_| ee_sub_228250(s)).collect();
            Ok(LoadData::L2282b0 { solid, first, list })
        })(),
        _ => return None,
    })
}
