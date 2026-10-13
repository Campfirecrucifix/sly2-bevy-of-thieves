//! WP-B3: the ALO load chain. FUN_001f31e0 (ALO stream load) = FUN_0022d7a0 (bones/attach
//! points + FUN_00186670/FUN_001603b8/FUN_0021c888) + FUN_00167320 (mesh set with the four
//! per-type mesh loaders 0x16a000/0x16e438/0x16caa0/0x16cf78 + FUN_00167870 LOD table) +
//! collision parts + property list + nested FUN_0013fb40 children (created by objects.rs).
//!
//! Only the stream data is ported: the GS/VIF packet building (FUN_00226e88/FUN_00226fd0
//! unpacks, FUN_001572e0.. DMA chains), VU0 math and allocation are replaced by typed arrays
//! that WP-C turns into Bevy meshes. Arrays that the original VIF-unpacks straight from the
//! stream are kept with their VIF format noted (V3-32 = [f32;3], V4-8 = [u8;4], ...).

use super::entry::{ee_sub_13fad0, ee_sub_1ed7a8};
use super::loads::{ee_sub_117fd0, ee_sub_118220, ee_sub_118420, Bsp, Geom};
use super::objects::WorldError;
use super::props::{ee_sub_13f810, Prop};
use crate::engine::disc::lz::BinaryInputStream;

type S<'a> = &'a mut BinaryInputStream;

fn f32_(s: S) -> f32 {
    f32::from_bits(s.read_u32())
}
fn s8(s: S) -> i8 {
    s.read_u8() as i8
}
/// FUN_00117eb0
fn vec3(s: S) -> [f32; 3] {
    [f32_(s), f32_(s), f32_(s)]
}
/// FUN_00117968 with a size: negative sizes read nothing.
fn bytes(s: S, n: i32) -> Vec<u8> {
    let n = n.max(0) as usize;
    let mut v = vec![0u8; n];
    s.read(n, Some(&mut v));
    v
}
fn arr<const N: usize>(s: S) -> [u8; N] {
    let mut a = [0u8; N];
    s.read(N, Some(&mut a));
    a
}
/// FUN_00117f18: FUN_00117ed0 (3 rows) + translation; w components forced (0,0,0,1).
fn mat4x3(s: S) -> [[f32; 3]; 4] {
    [vec3(s), vec3(s), vec3(s), vec3(s)]
}
fn props(s: S) -> Result<Vec<Prop>, WorldError> {
    ee_sub_13f810(s).map_err(|e| WorldError::BadField { field: e.field, offset: e.offset })
}
/// ((n + 31) / 32) * 4 bytes: one bit per index (MIPS rounding of the signed division kept).
fn bit_words(n: i32) -> i32 {
    let v = if n + 0x1f >= 0 { n + 0x1f } else { n + 0x3e };
    (v >> 5) << 2
}

// ---------------------------------------------------------------- FUN_0022d7a0

/// FUN_0022d7a0 flag 8 block (0x80-byte object, vtable 0x2ca7e0).
#[derive(Clone, Debug, PartialEq)]
pub struct Bone8 {
    pub b0: u8,
    pub f: f32,
    pub v0: [u8; 12],
    pub v1: [u8; 8],
    pub v2: [u8; 12],
    pub v3: [u8; 8],
    pub b1: u8,
    pub b2: u8,
    pub w: i32,
}

/// One FUN_0022d7a0 entry (0x100 bytes each, object +0x54, count +0x50).
#[derive(Clone, Debug, PartialEq)]
pub struct Bone {
    /// s16 oid resolved by vtable +0x19c (-> +0xa0)
    pub r#ref: i16,
    /// +0x94: 8/0xe = attached to another object (s16 ref, identity matrix), else matrix
    pub kind: i16,
    /// +0x9c
    pub word: i32,
    pub flags: u8,
    /// flag 1: one f32 (+0xac = f*1000); flag 2: two f32 (+0xac, +0xae)
    pub scale: Vec<f32>,
    /// flag 4 (0x30-byte object, vtable 0x2ca820)
    pub f4: Option<u8>,
    pub f8: Option<Bone8>,
    /// flag 0x10 (0x28-byte object, vtable 0x2ca7a0)
    pub f10: Option<(u8, f32, f32)>,
    /// flag 0x20 (0x30-byte object, vtable 0x2ca6a0)
    pub f20: Option<[f32; 3]>,
    /// kinds other than 8/0xe: FUN_00117f18 4x3 matrix
    pub xform: Option<[[f32; 3]; 4]>,
    /// kinds 8/0xe: s16 oid resolved by vtable +0x19c (-> +0xa8)
    pub attach: Option<i16>,
}

/// FUN_00186670 entry (0xe0 bytes)
#[derive(Clone, Debug, PartialEq)]
pub struct Rec186670 {
    pub refs: [i16; 3],
    pub v0: [f32; 3],
    pub r3: i16,
    pub v1: [f32; 3],
}
/// FUN_0021c888 entry (0x50 bytes) with its 0x70-byte link records (s16 refs)
#[derive(Clone, Debug, PartialEq)]
pub struct Rec21c888 {
    pub b: u8,
    pub f: [f32; 4],
    pub v0: [f32; 3],
    pub v1: [f32; 3],
    pub refs: Vec<i16>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AloBase {
    pub bones: Vec<Bone>,
    pub r186670: Vec<Rec186670>,
    /// FUN_001603b8: u8 n x {s16 ref, s16 ref, u8, u8}
    pub r1603b8: Vec<(i16, i16, u8, u8)>,
    pub r21c888: Vec<Rec21c888>,
    /// +0x58/+0x5c/+0x60 (s16 refs via vtable +0x19c)
    pub refs: [i16; 3],
}

pub fn ee_sub_22d7a0(s: S) -> AloBase {
    // 0x0022d7a0
    let n = s.read_s16();
    let mut bones = Vec::new();
    for _ in 0..n.max(0) {
        let r#ref = s.read_s16();
        let kind = s.read_s16();
        let word = s.read_s32();
        let flags = s.read_u8();
        let scale = if flags & 1 != 0 {
            vec![f32_(s)]
        } else if flags & 2 != 0 {
            vec![f32_(s), f32_(s)]
        } else {
            vec![]
        };
        let f4 = (flags & 4 != 0).then(|| s.read_u8());
        let f8 = (flags & 8 != 0).then(|| Bone8 {
            b0: s.read_u8(),
            f: f32_(s),
            v0: arr(s), // FUN_00117f90
            v1: arr(s), // FUN_00117fb0
            v2: arr(s),
            v3: arr(s),
            b1: s.read_u8(),
            b2: s.read_u8(),
            w: s.read_s32(),
        });
        let f10 = (flags & 0x10 != 0).then(|| (s.read_u8(), f32_(s), f32_(s)));
        let f20 = (flags & 0x20 != 0).then(|| vec3(s));
        let (xform, attach) = match kind {
            8 | 0xe => (None, Some(s.read_s16())),
            _ => (Some(mat4x3(s)), None),
        };
        bones.push(Bone { r#ref, kind, word, flags, scale, f4, f8, f10, f20, xform, attach });
    }
    // FUN_0022faa0: VU0 world matrices (no stream data)
    let r186670 = ee_sub_186670(s);
    let r1603b8 = ee_sub_1603b8(s);
    let r21c888 = ee_sub_21c888(s);
    let refs = [s.read_s16(), s.read_s16(), s.read_s16()];
    AloBase { bones, r186670, r1603b8, r21c888, refs }
}

pub fn ee_sub_186670(s: S) -> Vec<Rec186670> {
    // 0x00186670 (u8 n; n != 0 sets +0x148 bit 36)
    let n = s.read_u8();
    (0..n)
        .map(|_| {
            let refs = [s.read_s16(), s.read_s16(), s.read_s16()];
            let v0 = vec3(s);
            let r3 = s.read_s16();
            let v1 = vec3(s);
            Rec186670 { refs, v0, r3, v1 }
        })
        .collect()
}

pub fn ee_sub_1603b8(s: S) -> Vec<(i16, i16, u8, u8)> {
    // 0x001603b8
    let n = s.read_u8();
    (0..n).map(|_| (s.read_s16(), s.read_s16(), s.read_u8(), s.read_u8())).collect()
}

pub fn ee_sub_21c888(s: S) -> Vec<Rec21c888> {
    // 0x0021c888
    let n = s.read_u8();
    (0..n)
        .map(|_| {
            let b = s.read_u8();
            let f = [f32_(s), f32_(s), f32_(s), f32_(s)];
            let v0 = vec3(s);
            let v1 = vec3(s);
            let m = s.read_u8();
            let refs = (0..m).map(|_| s.read_s16()).collect();
            Rec21c888 { b, f, v0, v1, refs }
        })
        .collect()
}

// ---------------------------------------------------------------- FUN_00167320 mesh set

/// Skin weight header shared by the four mesh loaders: u8 n x {s32, s16} (0x50-byte records).
pub type WeightRefs = Vec<(i32, i16)>;

fn weight_refs(s: S) -> (i8, WeightRefs) {
    let n = s8(s);
    (n, (0..n.max(0)).map(|_| (s.read_s32(), s.read_s16())).collect())
}

/// FUN_0016c8e0: u16 n; n != 0: n x {u16 m x (u16, u16, u8)}, u16 c x s16.
#[derive(Clone, Debug, PartialEq)]
pub struct Rec16c8e0 {
    pub groups: Vec<Vec<(u16, u16, u8)>>,
    pub list: Vec<i16>,
}
pub fn ee_sub_16c8e0(s: S) -> Option<Rec16c8e0> {
    // 0x0016c8e0
    let n = s.read_u16();
    if n == 0 {
        return None;
    }
    let groups = (0..n)
        .map(|_| {
            let m = s.read_u16();
            (0..m).map(|_| (s.read_u16(), s.read_u16(), s.read_u8())).collect()
        })
        .collect();
    let c = s.read_u16();
    Some(Rec16c8e0 { groups, list: (0..c).map(|_| s.read_s16()).collect() })
}

/// FUN_0016a000 sub-mesh material record (12 bytes): index count, FUN_001ec8f8 material index
/// (0x2c-byte table at DAT_003dcfec), flags.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Material0 {
    pub count: u32,
    pub material: u32,
    pub flags: u32,
}

/// FUN_0016a000 sub-mesh (one VIF batch group).
#[derive(Clone, Debug, PartialEq)]
pub struct SubMesh0 {
    pub materials: Vec<Material0>,
    pub positions: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub normals: Vec<[f32; 3]>,
    /// RGBA (FUN_001ed278 recolours in place when DAT_003e12c8 is set)
    pub colors: Vec<[u8; 4]>,
    /// u8 (+0x110 flag; forced 1 for mesh types 3..8)
    pub flag: u8,
    /// 4 bone slot indices (u8, stored as +0xc + 4*i)
    pub weight_slots: Option<[u8; 4]>,
    /// min(nweights,4) bytes per vertex
    pub weights: Vec<u8>,
    /// vertex indices, one per strip vertex (sum of materials[].count)
    pub indices: Vec<u8>,
    /// one bit per index (strip restart / ADC bits)
    pub index_bits: Vec<u32>,
}

/// FUN_0016a000 (mesh type 0): the main static/skinned mesh.
#[derive(Clone, Debug, PartialEq)]
pub struct Mesh0 {
    pub flags: u8,
    /// index into MeshSet::descs (<0 = none)
    pub desc: i16,
    pub weight_refs: WeightRefs,
    pub nweights: i8,
    /// s8 != -1: 12 + 8 bytes (u8 scale * 1/127, *6.2e-5) + vec3
    pub quant: Option<([u8; 12], [u8; 8], [f32; 3])>,
    pub r16c8e0: Option<Rec16c8e0>,
    /// flags & 4: s32, s8 != -1 -> (4 f32, 4x3 matrix), 3 u8
    pub f4: Option<(i32, Option<([f32; 4], [[f32; 3]; 4])>, [u8; 3])>,
    pub subs: Vec<SubMesh0>,
}

fn weights(s: S, nw: i8, nv: i32) -> (Option<[u8; 4]>, Vec<u8>) {
    if nw == 0 {
        return (None, vec![]);
    }
    let slots = arr::<4>(s);
    let k = (nw as i32).min(4);
    (Some(slots), bytes(s, k * nv))
}

pub fn ee_sub_16a000(s: S) -> Mesh0 {
    // 0x0016a000
    let flags = s.read_u8();
    let desc = s.read_s16();
    let (nweights, weight_refs) = weight_refs(s);
    let quant = (s8(s) != -1).then(|| (arr::<12>(s), arr::<8>(s), vec3(s)));
    let r16c8e0 = ee_sub_16c8e0(s);
    let f4 = (flags & 4 != 0).then(|| {
        let w = s.read_s32();
        let m = (s8(s) != -1).then(|| ([f32_(s), f32_(s), f32_(s), f32_(s)], mat4x3(s)));
        (w, m, [s.read_u8(), s.read_u8(), s.read_u8()])
    });
    let n = s.read_s16();
    let subs = (0..n.max(0))
        .map(|_| {
            let nm = s.read_u8();
            let materials: Vec<Material0> =
                (0..nm).map(|_| Material0 { count: s.read_u32(), material: s.read_u32(), flags: s.read_u32() }).collect();
            let sum = materials.iter().fold(0i32, |a, m| a.wrapping_add(m.count as i32));
            let nv = s.read_u8() as i32;
            let positions = (0..nv).map(|_| vec3(s)).collect();
            let uvs = (0..nv).map(|_| [f32_(s), f32_(s)]).collect();
            let normals = (0..nv).map(|_| vec3(s)).collect();
            let colors = (0..nv).map(|_| arr::<4>(s)).collect();
            let flag = s.read_u8();
            let (weight_slots, weights) = weights(s, nweights, nv);
            let indices = bytes(s, sum);
            let index_bits = bytes(s, bit_words(sum)).chunks(4).map(|c| u32::from_le_bytes(c.try_into().unwrap())).collect();
            SubMesh0 { materials, positions, uvs, normals, colors, flag, weight_slots, weights, indices, index_bits }
        })
        .collect();
    Mesh0 { flags, desc, weight_refs, nweights, quant, r16c8e0, f4, subs }
}

/// FUN_0016e438 / FUN_0016caa0 strip: VIF-unpacked arrays.
#[derive(Clone, Debug, PartialEq)]
pub struct Strip12 {
    /// FUN_0016caa0 only: u16 material index (FUN_001ec8f8)
    pub material: Option<u16>,
    /// V3-32 unpack
    pub positions: Vec<[f32; 3]>,
    /// u8 k bone slots; k bytes per vertex (S8/V2-8/V3-8/V4-8 unpack per table 0x2be558)
    pub weight_slots: Vec<u8>,
    pub weights: Vec<u8>,
    /// V4-8 unpack (u8 count)
    pub v4_8: Vec<[u8; 4]>,
}

/// FUN_0016e438 (type 1) / FUN_0016caa0 (type 2)
#[derive(Clone, Debug, PartialEq)]
pub struct MeshStrips {
    pub weight_refs: WeightRefs,
    pub strips: Vec<Strip12>,
}

/// VIF unpack element size for table 0x2be558[k] via 0x2c4430 (k = weights per vertex).
fn weight_unpack_size(k: u8, off: u32) -> Result<i32, WorldError> {
    match k {
        0..=4 => Ok(k as i32), // fmt -1 (size 0), S8, V2-8, V3-8, V4-8
        5..=7 => Ok(4),        // fmt 0 (S32)
        _ => Err(WorldError::BadData { what: "weight count", offset: off }),
    }
}

fn strips(s: S, with_material: bool) -> Result<MeshStrips, WorldError> {
    let (nw, weight_refs) = weight_refs(s);
    let n = s.read_s16();
    let mut strips = Vec::new();
    for _ in 0..n.max(0) {
        let material = with_material.then(|| s.read_u16());
        let nv = s.read_u8() as i32;
        let positions = (0..nv).map(|_| vec3(s)).collect();
        let (weight_slots, weights) = if nw != 0 {
            let off = s.tell();
            let k = s.read_u8();
            let slots = bytes(s, k as i32);
            let sz = weight_unpack_size(k, off)?;
            (slots, bytes(s, sz * nv))
        } else {
            (vec![], vec![])
        };
        let m = s.read_u8();
        let v4_8 = (0..m).map(|_| arr::<4>(s)).collect();
        strips.push(Strip12 { material, positions, weight_slots, weights, v4_8 });
    }
    Ok(MeshStrips { weight_refs, strips })
}

/// FUN_0016cf78 (type 3) strip: indexed positions.
#[derive(Clone, Debug, PartialEq)]
pub struct Strip3 {
    pub material: u16,
    pub positions: Vec<[f32; 3]>,
    pub weight_slots: Option<[u8; 4]>,
    pub weights: Vec<u8>,
    /// u8 per strip vertex: index into positions
    pub indices: Vec<u8>,
    pub index_bits: Vec<u32>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Mesh3 {
    pub weight_refs: WeightRefs,
    pub strips: Vec<Strip3>,
}
pub fn ee_sub_16cf78(s: S) -> Mesh3 {
    // 0x0016cf78
    let (nw, weight_refs) = weight_refs(s);
    let n = s.read_s16();
    let strips = (0..n.max(0))
        .map(|_| {
            let material = s.read_u16();
            let nidx = s.read_u8() as i32;
            let nv = s.read_u8() as i32;
            let positions = (0..nv).map(|_| vec3(s)).collect();
            let (weight_slots, weights) = weights(s, nw, nv);
            let indices = bytes(s, nidx);
            let index_bits = bytes(s, bit_words(nidx)).chunks(4).map(|c| u32::from_le_bytes(c.try_into().unwrap())).collect();
            Strip3 { material, positions, weight_slots, weights, indices, index_bits }
        })
        .collect();
    Mesh3 { weight_refs, strips }
}

#[derive(Clone, Debug, PartialEq)]
pub enum MeshData {
    Type0(Mesh0),
    Type1(MeshStrips),
    Type2(MeshStrips),
    Type3(Mesh3),
    /// types >= 4: no data read
    Other,
    /// mission-filtered out: `size` bytes skipped (FUN_001179b8)
    Skipped { size: u32 },
}

/// One FUN_00167320 mesh record (0x14 + 0x28 [+ 0x18] bytes in the original).
#[derive(Clone, Debug, PartialEq)]
pub struct MeshRecord {
    /// s8 n > 0: mission list + u32 size (skip size when rejected)
    pub filter: Option<(Vec<i16>, u32)>,
    /// +3: mesh type (0..3 = loader)
    pub kind: i8,
    /// u16: number of extra slots this record spans (slot index += n + 1)
    pub span: u16,
    /// s16 oid resolved by vtable +0x19c
    pub r#ref: i16,
    /// +0 (low 5 bits -> flags bits 3..7)
    pub b0: i8,
    /// +4: f*|f| unless FLT_MAX
    pub dist: f32,
    pub f1: f32,
    pub word: i32,
    pub u0: u8,
    /// bit 11
    pub u1: u8,
    /// only when the set flag is 0: bounding sphere centre, radius, u32, far distance
    /// (FLT_MAX -> 2000.0 for type 1, 1e10 for types 0/2/3)
    pub bounds: Option<([f32; 3], f32, u32, f32)>,
    pub data: MeshData,
    /// FUN_001699e8 -> FUN_00169ac8: `span` instances of this mesh (types 0..3 only)
    pub instances: Vec<MeshInstance>,
}

/// FUN_00169ac8: one extra slot copying the record with its own transform.
#[derive(Clone, Debug, PartialEq)]
pub struct MeshInstance {
    /// u8 read and discarded
    pub discarded: u8,
    /// bounds present: u32 (+0x14 of the 0x18 bounds record)
    pub bound_word: Option<u32>,
    /// s16 oid resolved by vtable +0x19c
    pub r#ref: i16,
    /// FUN_00117f18 (kept in a 0x40 alloc unless flags & 0x1000)
    pub xform: [[f32; 3]; 4],
    pub word: i32,
    /// bit 11
    pub u1: u8,
    pub r16c8e0: Option<Rec16c8e0>,
}

/// FUN_00169ac8 stream part.
pub fn ee_sub_169ac8(s: S, bounds: bool) -> MeshInstance {
    // 0x00169ac8
    let discarded = s.read_u8();
    let bound_word = bounds.then(|| s.read_u32());
    let r#ref = s.read_s16();
    let xform = mat4x3(s);
    let word = s.read_s32();
    let u1 = s.read_u8();
    let r16c8e0 = ee_sub_16c8e0(s);
    MeshInstance { discarded, bound_word, r#ref, xform, word, u1, r16c8e0 }
}

/// FUN_00167870 LOD slot (0x20 bytes, 4 per node)
#[derive(Clone, Debug, PartialEq)]
pub struct LodSlot {
    /// 4-bit kind (-1 = end of node): 0 -> mesh record index, 1 -> node index
    pub kind: u8,
    pub pos: [f32; 3],
    pub f0: f32,
    /// distance (clamped: > 1e10 -> 1e10 bits 0x501502f9; otherwise truncated to int bits)
    pub f1: f32,
    pub b: u8,
    pub link: Option<i16>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MeshSet {
    /// u8: nonzero = no per-record bounds (+0x44 not allocated)
    pub no_bounds: u8,
    pub center: [f32; 3],
    pub radius: f32,
    /// FUN_001ed7a8(1, ..) objects (class + 0x20 init block)
    pub descs: Vec<Option<(u16, [u8; 0x20])>>,
    pub words: Vec<i32>,
    /// s16 slot count as read (before rejected records are removed)
    pub slots: i16,
    pub records: Vec<MeshRecord>,
    /// FUN_00167870: nodes of up to 4 slots
    pub lods: Vec<Vec<LodSlot>>,
}

impl MeshSet {
    /// (parsed meshes, vertices) — type 0 sub-mesh vertices + type 1/2/3 strip positions
    pub fn stats(&self) -> (usize, usize) {
        let mut m = 0;
        let mut v = 0;
        for r in &self.records {
            match &r.data {
                MeshData::Type0(x) => {
                    m += 1;
                    v += x.subs.iter().map(|s| s.positions.len()).sum::<usize>();
                }
                MeshData::Type1(x) | MeshData::Type2(x) => {
                    m += 1;
                    v += x.strips.iter().map(|s| s.positions.len()).sum::<usize>();
                }
                MeshData::Type3(x) => {
                    m += 1;
                    v += x.strips.iter().map(|s| s.positions.len()).sum::<usize>();
                }
                _ => {}
            }
        }
        (m, v)
    }
}

pub fn ee_sub_167320(s: S, mission: i16) -> Result<MeshSet, WorldError> {
    // 0x00167320
    let no_bounds = s.read_u8();
    let center = vec3(s);
    let radius = f32_(s);
    let nd = s.read_u8();
    let descs = (0..nd).map(|_| ee_sub_1ed7a8(s)).collect();
    let nw = s.read_u8();
    let words = (0..nw).map(|_| s.read_s32()).collect();
    let slots = s.read_s16();
    let mut count = slots as i32;
    let mut records = Vec::new();
    let mut i = 0i32;
    while i < count {
        let k = s8(s);
        let mut filter = None;
        if k != 0 {
            let list: Vec<i16> = (0..k.max(0)).map(|_| s.read_s16()).collect();
            let size = s.read_u32();
            // FUN_0013fad0
            if !ee_sub_13fad0(mission, &list) {
                s.read((size as i32).max(0) as usize, None); // FUN_001179b8
                records.push(MeshRecord {
                    filter: Some((list, size)),
                    kind: -1,
                    span: 0,
                    r#ref: -1,
                    b0: 0,
                    dist: 0.0,
                    f1: 0.0,
                    word: 0,
                    u0: 0,
                    u1: 0,
                    bounds: None,
                    data: MeshData::Skipped { size },
                    instances: vec![],
                });
                count -= 1;
                continue;
            }
            filter = Some((list, size));
        }
        // LAB_0016758c
        let kind = s8(s);
        let span = s.read_u16();
        let r#ref = s.read_s16();
        let b0 = s8(s);
        let d = f32_(s);
        let dist = if d == f32::MAX { d } else { d * d.abs() };
        let f1 = f32_(s);
        let word = s.read_s32();
        let u0 = s.read_u8();
        let u1 = s.read_u8();
        let bounds = (no_bounds == 0).then(|| {
            let c = vec3(s);
            let r = f32_(s);
            let w = s.read_u32();
            let mut far = f32_(s);
            if far == f32::MAX {
                match kind {
                    1 => far = 1000.0 * 2.0,                    // 0x44fa0000
                    0 | 2 | 3 => far = f32::from_bits(0x501502f9), // 1e10
                    _ => {}
                }
            }
            (c, r, w, far)
        });
        let data = match kind {
            0 => MeshData::Type0(ee_sub_16a000(s)),
            1 => MeshData::Type1(strips(s, false)?), // FUN_0016e438
            2 => MeshData::Type2(strips(s, true)?),  // FUN_0016caa0
            3 => MeshData::Type3(ee_sub_16cf78(s)),
            _ => MeshData::Other,
        };
        // each loader ends with FUN_001699e8(.., span): FUN_00169ac8 per extra slot
        let instances = if matches!(data, MeshData::Other) {
            vec![]
        } else {
            (0..span).map(|_| ee_sub_169ac8(s, bounds.is_some())).collect()
        };
        records.push(MeshRecord { filter, kind, span, r#ref, b0, dist, f1, word, u0, u1, bounds, data, instances });
        i += span as i32 + 1;
    }
    let lods = ee_sub_167870(s);
    Ok(MeshSet { no_bounds, center, radius, descs, words, slots, records, lods })
}

pub fn ee_sub_167870(s: S) -> Vec<Vec<LodSlot>> {
    // 0x00167870
    let n = s.read_s16();
    (0..n.max(0))
        .map(|_| {
            let mut node = Vec::new();
            for _ in 0..4 {
                let kind = s.read_u8() & 0xf;
                if kind == 0xf {
                    break; // remaining slots get kind 0xf
                }
                let pos = vec3(s);
                let f0 = f32_(s);
                let f1 = f32_(s);
                let b = s.read_u8();
                let f1 = if f1 > 1e10 { f32::from_bits(0x501502f9) } else { f32::from_bits(f1 as i32 as u32) }; // quirk kept
                let link = (kind == 0 || kind == 1).then(|| s.read_s16());
                node.push(LodSlot { kind, pos, f0, f1, b, link });
            }
            node
        })
        .collect()
}

// ---------------------------------------------------------------- FUN_001f31e0

/// One FUN_001f31e0 collision part (0xb0 bytes at +0xe4, count +0xe0).
#[derive(Clone, Debug, PartialEq)]
pub struct AloPart {
    /// s16 oid resolved by vtable +0x19c (0 -> DAT_002c44f0)
    pub r#ref: i16,
    pub w0: i32,
    pub b0: u8,
    pub b1: u8,
    pub f: f32,
    pub geom: Geom,
    pub bsp: Bsp,
    pub w1: u32,
    pub s0: i16,
    pub s1: i16,
    pub b2: u8,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Alo {
    pub base: AloBase,
    /// u32 passed to FUN_001a6d28 at the end
    pub word: u32,
    pub mesh: MeshSet,
    pub parts: Vec<AloPart>,
    /// parts > 0: f32 (+0xe8 +0xd0) + FUN_00117ed0 3x3 matrix
    pub part_xform: Option<(f32, [[f32; 3]; 3])>,
    pub props: Vec<Prop>,
}

impl Alo {
    pub fn stats(&self) -> (usize, usize) {
        self.mesh.stats()
    }
}

/// FUN_001f31e0 up to (not including) the nested FUN_0013fb40 children, which objects.rs
/// loads right after (with this object as owner), followed by FUN_001a6d28.
pub fn ee_sub_1f31e0(s: S, mission: i16) -> Result<Alo, WorldError> {
    // 0x001f31e0
    let base = ee_sub_22d7a0(s);
    let word = s.read_u32();
    let mesh = ee_sub_167320(s, mission)?;
    // vtable +0x1fc: extra (empty) part slots, kind 0xfff6; no stream data
    let n = s.read_s16();
    let parts: Vec<AloPart> = (0..n.max(0))
        .map(|_| {
            let r#ref = s.read_s16();
            let w0 = s.read_s32();
            let b0 = s.read_u8();
            let b1 = s.read_u8();
            let f = f32_(s);
            let geom = ee_sub_117fd0(s);
            // FUN_00166270 transforms geom when ref != 0 (derived, not stream)
            let bsp = ee_sub_118220(s);
            AloPart { r#ref, w0, b0, b1, f, geom, bsp, w1: s.read_u32(), s0: s.read_s16(), s1: s.read_s16(), b2: s.read_u8() }
        })
        .collect();
    // FUN_001f2b20: derived
    let part_xform = (!parts.is_empty()).then(|| (f32_(s), [vec3(s), vec3(s), vec3(s)]));
    let props = props(s)?;
    // FUN_001f2500: derived
    Ok(Alo { base, word, mesh, parts, part_xform, props })
}

/// Load virtuals that end with FUN_001f31e0 + something after the children.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AloTail {
    None,
    /// FUN_001c3dc8 / FUN_002039c0 / FUN_001553f0: FUN_00118420 blob after FUN_001f31e0
    Blob,
}
/// ALO-based load virtuals by vtable +0x74 address.
pub fn alo_load_kind(load: u32) -> Option<AloTail> {
    match load {
        0x001f31e0 | 0x00223fe0 => Some(AloTail::None),
        0x001c3dc8 | 0x002039c0 | 0x001553f0 => Some(AloTail::Blob),
        _ => None,
    }
}
/// The tail read after FUN_001f31e0 returns.
pub fn read_tail(t: AloTail, s: S) -> Option<Vec<u8>> {
    match t {
        AloTail::None => None,
        AloTail::Blob => Some(ee_sub_118420(s)),
    }
}
