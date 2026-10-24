//! Per-directory-entry sections read by the FUN_0013da18 entry loop before the objects.
//!
//! Entry layout (each entry starts 16-aligned):
//!   FUN_0020f5d0: s32 blob size (DAT_003e1c30), s32 blob key (DAT_003e1c58; changed -> reload)
//!   s32 DAT_003e1c34 (extended records flag)
//!   FUN_001ff858: s32 n; n x FUN_0012a3d8 value [+ 3 u16-length strings if DAT_003e1c34]
//!   FUN_001eb748: FUN_001eae40, FUN_001eb098, u16 n x FUN_001ed7a8, u16 n x FUN_001eb2f0,
//!                 FUN_0020f640 (align 16 + blob of `blob size` bytes)
//!   FUN_001c0ec0: s32 n [FUN_001c0f30]
//!   FUN_0013fb40: objects (objects.rs), then align 16.

use crate::engine::disc::lz::BinaryInputStream;

fn bytes(s: &mut BinaryInputStream, n: usize) -> Vec<u8> {
    let mut v = vec![0u8; n];
    s.read(n, Some(&mut v));
    v
}
fn arr<const N: usize>(s: &mut BinaryInputStream) -> [u8; N] {
    let mut a = [0u8; N];
    s.read(N, Some(&mut a));
    a
}
/// s8 count + count s16 (inclusion list tested by FUN_0013fad0)
fn filter(s: &mut BinaryInputStream, n: i8) -> Vec<i16> {
    (0..n.max(0)).map(|_| s.read_s16()).collect()
}

/// FUN_0013fad0: true when DAT_003e1110 (FUN_00210a88, the current mission) is -1 or listed.
pub fn ee_sub_13fad0(mission: i16, list: &[i16]) -> bool {
    // 0x0013fad0
    mission == -1 || list.contains(&mission)
}

/// FUN_0012a3d8: tagged value (refcounted variant in the original).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Value12a3d8 {
    /// u8; 0xff = null (nothing else read)
    pub tag: u8,
    /// DAT_003e1c34 != 0: s32 + 6 x s16 (read and dropped)
    pub ext: Option<[u8; 16]>,
    /// tag & 0x7f: 0x15 s32, 0x16 f32, 0x17 bool, 0x18/0x19/0x21 u32, 0x1a/0x1c/0x1e 3 f32,
    /// 0x1b 16 f32, 0x1d 2 f32; 0x1f nests a value; others carry no data
    pub data: Vec<u8>,
    pub nested: Option<Box<Value12a3d8>>,
    /// tag & 0x80: attached value (FUN_00256c58)
    pub attached: Option<Box<Value12a3d8>>,
}

pub fn ee_sub_12a3d8(s: &mut BinaryInputStream, ext: bool) -> Value12a3d8 {
    // 0x0012a3d8
    let tag = s.read_u8();
    let mut v = Value12a3d8 { tag, ..Default::default() };
    if tag == 0xff {
        return v;
    }
    if ext {
        v.ext = Some(arr::<16>(s));
    }
    let n = match tag & 0x7f {
        0x15..=0x19 | 0x21 => 4,
        0x1a | 0x1c | 0x1e => 12,
        0x1b => 64,
        0x1d => 8,
        _ => 0,
    };
    v.data = bytes(s, n);
    if tag & 0x7f == 0x1f {
        v.nested = Some(Box::new(ee_sub_12a3d8(s, ext)));
    }
    if tag & 0x80 != 0 {
        v.attached = Some(Box::new(ee_sub_12a3d8(s, ext)));
    }
    v
}

/// FUN_001ea8a8 record (0x20 bytes in the game; 16 read).
#[derive(Clone, Debug, PartialEq)]
pub struct Rec1ea8a8 {
    pub head: [u8; 16],
    pub filter: Vec<i16>,
    /// u16 +8 x 4 bytes (unless +4 == 1); None when filtered out / shared
    pub data: Option<Vec<u8>>,
}
impl Rec1ea8a8 {
    pub fn kind(&self) -> u8 {
        self.head[4]
    }
}

/// FUN_001ea8a8. Bytes consumed do not depend on the filter or on FUN_001ed5d0 (an
/// already-resident copy is skipped instead of read).
pub fn ee_sub_1ea8a8(s: &mut BinaryInputStream, mission: i16) -> Rec1ea8a8 {
    // 0x001ea8a8
    let head = arr::<16>(s);
    let n = u16::from_le_bytes([head[8], head[9]]) as usize * 4;
    let filter = filter(s, head[5] as i8);
    let mut data = None;
    let pass = head[5] as i8 <= 0 || ee_sub_13fad0(mission, &filter);
    if head[4] != 1 {
        let d = bytes(s, n);
        if pass {
            data = Some(d);
        }
    }
    Rec1ea8a8 { head, filter, data }
}

/// FUN_001eaf50 record (0x28 bytes; 0x18 read): +4 kind, +5 s8 filter count, +0x10 s32 size.
#[derive(Clone, Debug, PartialEq)]
pub struct Rec1eaf50 {
    pub head: [u8; 0x18],
    pub filter: Vec<i16>,
    pub data: Option<Vec<u8>>,
}

pub fn ee_sub_1eaf50(s: &mut BinaryInputStream, mission: i16) -> Rec1eaf50 {
    // 0x001eaf50
    let head = arr::<0x18>(s);
    let n = i32::from_le_bytes(head[0x10..0x14].try_into().unwrap()).max(0) as usize;
    let filter = filter(s, head[5] as i8);
    let pass = head[5] as i8 <= 0 || ee_sub_13fad0(mission, &filter);
    let mut data = None;
    if head[4] != 1 {
        let d = bytes(s, n);
        if pass {
            data = Some(d);
        }
    }
    Rec1eaf50 { head, filter, data }
}

/// FUN_001ed7a8: u16 class (table 0x2baaf8 ctor) + 0x20-byte init block (vtable +0x3c).
/// 0xffff = none (compared as -1; the loader sign-extends).
pub fn ee_sub_1ed7a8(s: &mut BinaryInputStream) -> Option<(u16, [u8; 0x20])> {
    // 0x001ed7a8
    let c = s.read_u16();
    if c == 0xffff {
        return None;
    }
    Some((c, arr::<0x20>(s)))
}

/// FUN_001eb1a8: 8 bytes; +6 u8 n6 x u16 (index into the FUN_001eb098 table),
/// +7 u8 n7 x u16 (index into the FUN_001eae40 table, 0xffff = none).
#[derive(Clone, Debug, PartialEq)]
pub struct Rec1eb1a8 {
    pub head: [u8; 8],
    pub refs_eb098: Vec<u16>,
    pub refs_eae40: Vec<u16>,
}

pub fn ee_sub_1eb1a8(s: &mut BinaryInputStream) -> Rec1eb1a8 {
    // 0x001eb1a8
    let head = arr::<8>(s);
    let refs_eb098 = (0..head[6]).map(|_| s.read_u16()).collect();
    let refs_eae40 = (0..head[7]).map(|_| s.read_u16()).collect();
    Rec1eb1a8 { head, refs_eb098, refs_eae40 }
}

/// FUN_001eb2f0 record (0x2c bytes; 0x18 read): +4 kind (buffer layout, FUN_001eb960 GS setup),
/// +5 flags, +0x17 u8 sub count; s16 index into the FUN_001ed7a8 table (-1 = none).
#[derive(Clone, Debug, PartialEq)]
pub struct Rec1eb2f0 {
    pub head: [u8; 0x18],
    pub ed7a8: i16,
    pub subs: Vec<Rec1eb1a8>,
}

pub fn ee_sub_1eb2f0(s: &mut BinaryInputStream) -> Rec1eb2f0 {
    // 0x001eb2f0 (GS buffer allocation by kind is WP-C)
    let head = arr::<0x18>(s);
    let ed7a8 = s.read_s16();
    let subs = (0..head[0x17]).map(|_| ee_sub_1eb1a8(s)).collect();
    Rec1eb2f0 { head, ed7a8, subs }
}

/// FUN_001c0f30 (world+0x45cc, 0x30 bytes): n x (u16 idx into b, u16, u16), a x 16 bytes,
/// b x s32.
#[derive(Clone, Debug, PartialEq)]
pub struct Block1c0f30 {
    pub items: Vec<[u16; 3]>,
    pub vecs: Vec<[u8; 16]>,
    pub words: Vec<i32>,
}

/// FUN_001c0ec0: s32 n; n != 0 -> FUN_001c0f30.
pub fn ee_sub_1c0ec0(s: &mut BinaryInputStream) -> Option<Block1c0f30> {
    // 0x001c0ec0 / 0x001c0f30
    let n = s.read_s32();
    if n == 0 {
        return None;
    }
    let a = s.read_s32();
    let b = s.read_s32();
    let items = (0..n.max(0)).map(|_| [s.read_u16(), s.read_u16(), s.read_u16()]).collect();
    let vecs = (0..a.max(0)).map(|_| arr::<16>(s)).collect();
    let words = (0..b.max(0)).map(|_| s.read_s32()).collect();
    Some(Block1c0f30 { items, vecs, words })
}

/// Everything an entry holds besides its objects.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EntrySections {
    /// FUN_0020f5d0
    pub blob_size: i32,
    pub blob_key: i32,
    /// DAT_003e1c34
    pub ext: bool,
    /// FUN_001ff858 list (owner = DirEntry::c); FUN_0013fb40 parent indices point here
    pub anchors: Vec<Value12a3d8>,
    /// FUN_001eae40 (DAT_003dcffc, 0x20 each)
    pub eae40: Vec<Rec1ea8a8>,
    /// FUN_001eb098 (DAT_003dcff4, 0x28 each)
    pub eb098: Vec<Rec1eaf50>,
    /// DAT_003dd004
    pub ed7a8: Vec<Option<(u16, [u8; 0x20])>>,
    /// DAT_003dcfec (0x2c each)
    pub eb2f0: Vec<Rec1eb2f0>,
    /// FUN_0020f640 blob: stream offset (16-aligned) and size; loaded by FUN_001ea568 (WP-C)
    pub blob_offset: u32,
    pub block_45cc: Option<Block1c0f30>,
}

/// The entry loop body of FUN_0013da18 up to FUN_0013fb40.
pub fn read_entry_sections(s: &mut BinaryInputStream, mission: i16) -> EntrySections {
    let mut e = EntrySections::default();
    ee_sub_20f5d0(s, &mut e);
    ee_sub_1ff858(s, &mut e);
    // 0x001eb748
    e.eae40 = ee_sub_1eae40(s, mission);
    e.eb098 = ee_sub_1eb098(s, mission);
    let n = s.read_u16();
    e.ed7a8 = (0..n).map(|_| ee_sub_1ed7a8(s)).collect();
    let n = s.read_u16();
    e.eb2f0 = (0..n).map(|_| ee_sub_1eb2f0(s)).collect();
    ee_sub_20f640(s, &mut e);
    e.block_45cc = ee_sub_1c0ec0(s);
    e
}

/// FUN_0020f5d0: blob size, blob key, ext flag.
pub fn ee_sub_20f5d0(s: &mut BinaryInputStream, e: &mut EntrySections) {
    e.blob_size = s.read_s32();
    e.blob_key = s.read_s32();
    e.ext = s.read_s32() != 0;
}

/// FUN_001ff858: s32 n; n x FUN_0012a3d8 value [+ 3 strings if ext].
pub fn ee_sub_1ff858(s: &mut BinaryInputStream, e: &mut EntrySections) {
    let n = s.read_s32();
    for _ in 0..n.max(0) {
        e.anchors.push(ee_sub_12a3d8(s, e.ext));
        if e.ext {
            for _ in 0..3 {
                s.skip_string();
            }
        }
    }
}

/// FUN_001eae40: u16 n x FUN_001ea8a8.
pub fn ee_sub_1eae40(s: &mut BinaryInputStream, mission: i16) -> Vec<Rec1ea8a8> {
    let n = s.read_u16();
    (0..n).map(|_| ee_sub_1ea8a8(s, mission)).collect()
}

/// FUN_001eb098: u16 n x FUN_001eaf50.
pub fn ee_sub_1eb098(s: &mut BinaryInputStream, mission: i16) -> Vec<Rec1eaf50> {
    let n = s.read_u16();
    (0..n).map(|_| ee_sub_1eaf50(s, mission)).collect()
}

/// FUN_0020f640: align 16, then the blob (skipped when the key is unchanged; same bytes).
pub fn ee_sub_20f640(s: &mut BinaryInputStream, e: &mut EntrySections) {
    s.align(0x10);
    e.blob_offset = s.tell();
    s.read(e.blob_size.max(0) as usize, None);
}

