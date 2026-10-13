//! Level open (FUN_0020e890) and the FK$X directory / level header parse (FUN_0013da18),
//! ported up to the start of object instantiation (FUN_0013f810 -> FUN_0013e818 is WP-B).
//!
//! Level stream layout (all read through one LZ CBinaryInputStream of exactly `size` raw bytes):
//!   1. directory (FUN_0013da18): s16 count; count x { u8 fid[64] "FK$X<name>"; s32 a,b,c,d,n }
//!   2. FUN_001fd0e8 -> global 0x3dd040 (language-selected descriptor lists, see [`Block3dd040`])
//!   3. FUN_001d7b88 -> world+0x3a18 (string hash table + 10-byte records)
//!   4. FUN_0020f318 level table ([`super::descriptor::read_level_table`])
//!   5. FUN_0013df80 (language-selected string table), FUN_0013e0e0 (binary tree of 16-byte keys)
//!   6. FUN_0013f810(0x3dd490), FUN_0013f810(0x2ddf40): object lists terminated by s16 < 0 (WP-B)
//!
//! Directory fields (stored at entry+0x5c, +0x60, +0x10, +0x14, +0x08 of the 0x6c-byte entry):
//!   a (+0x5c): 0 = instantiate the entry's objects now (+0x3f10 = entry index, each object's
//!              vtable +0x4c "post-load" is called); nonzero = template only, +0x0c load count++.
//!   b (+0x60): copied, not read again by FUN_0013da18.
//!   c (+0x10): owner id passed to FUN_001ff858 (the per-entry list linked at world+0x39e8/+0x39ec).
//!   d (+0x14): copied, not read again by FUN_0013da18.
//!   n (+0x08): instance count: entry+0x18 = n s32 slots set to -1; after loading the entry's
//!              objects (count m), entry+0x04 = m*n pointers, copies 2..n cloned by FUN_0013e510.
//! In the data, row 0 is the level itself (a=1, n=1) and c/d look like byte extents of the entry.

use super::descriptor::{read_level_table, Descriptor, LevelTable};
use super::iso::{DiscSource, SectorSource};
use super::lz::{BinaryInputStream, SliceSource};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DirEntry {
    pub fid: [u8; 64],
    pub a: i32,
    pub b: i32,
    pub c: i32,
    pub d: i32,
    pub n: i32,
}

impl DirEntry {
    /// Name after the "FK$X" tag (FUN_0014b690 on entry+0x1c is used for the name lookup).
    pub fn name(&self) -> String {
        let s = self.fid.strip_prefix(b"FK$X").unwrap_or(&self.fid);
        let e = s.iter().position(|&c| c == 0).unwrap_or(s.len());
        String::from_utf8_lossy(&s[..e]).into_owned()
    }
    /// a == 0: objects are instantiated at load (see module docs).
    pub fn instantiate_now(&self) -> bool {
        self.a == 0
    }
}

/// FUN_0013da18 first loop: the directory.
pub fn read_directory(s: &mut BinaryInputStream) -> Vec<DirEntry> {
    // 0x0013da18
    let count = s.read_s16() as i32;
    (0..count.max(0))
        .map(|_| {
            let fid = s.read_64();
            let a = s.read_s32();
            let b = s.read_s32();
            let c = s.read_s32();
            let d = s.read_s32();
            let n = s.read_s32();
            DirEntry { fid, a, b, c, d, n }
        })
        .collect()
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Rec5c {
    /// +0x40, +0x48
    pub h0: i16,
    pub h1: i16,
    /// language-selected descriptor (+0x00), u8 (+0x4c), u32 (+0x50)
    pub desc: Descriptor,
    pub u8v: u8,
    pub u32v: u32,
    /// +0x58: k x 8 bytes
    pub data: Vec<[u8; 8]>,
}

/// Global 0x3dd040 as filled by FUN_001fd0e8.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Block3dd040 {
    /// +0xa8: descriptor; replaced (and the running stream stopped) when not the same file
    pub desc: Descriptor,
    /// +0xf8/+0xfc
    pub bytes: Vec<u8>,
    /// +0x18/+0x100: map of size count, -1 filled, map[s16] = i for each of the bytes
    pub map: Vec<i32>,
    /// +0x1d0 count, +0xf4 descriptor (only if count > 0), +0x1d4 u16[<=6]
    pub desc2: Option<Descriptor>,
    pub small: Vec<i16>,
    /// +0xec list (FUN_001fd0e8 links them in reverse; kept in file order here)
    pub recs: Vec<Rec5c>,
    /// +0xf0 list: (s16 +0x40, 64-byte block)
    pub recs48: Vec<(i16, [u8; 64])>,
}

/// FUN_001fd0e8 (stream part). `language` = DAT_002e1e54. Stopping a playing stream on a
/// descriptor change (FUN_002321b8/FUN_00231c48) is left to the audio WP.
pub fn ee_sub_1fd0e8(b: &mut Block3dd040, s: &mut BinaryInputStream, language: u8) {
    // 0x001fd0e8
    let d = Descriptor(s.read_64());
    if !d.same_file(&b.desc) {
        b.desc = d;
    }
    let n = s.read_s16() as i32;
    b.bytes = (0..n.max(0)).map(|_| s.read_u8()).collect();
    let m = s.read_s16() as i32;
    b.map = vec![-1; m.max(0) as usize];
    for i in 0..n.max(0) {
        let k = s.read_s16() as i32;
        // Quirk: unchecked index in the original.
        if let Some(slot) = b.map.get_mut(k as usize) {
            *slot = i;
        }
    }
    let c = s.read_s16() as i32;
    b.desc2 = None;
    if c > 0 {
        b.desc2 = Some(Descriptor(s.read_64()));
    }
    // Quirk: the original writes c u16s into a 0x18-byte area (+0x1d4) without bound.
    b.small = (0..c.max(0)).map(|_| s.read_s16()).collect();
    let nl = s.read_u8() as i32;
    let mut sel = 0;
    for i in 0..nl {
        if s.read_u8() == language {
            sel = i;
        }
    }
    let nr = s.read_s32();
    b.recs.clear();
    for _ in 0..nr.max(0) {
        let mut r = Rec5c { h0: s.read_s16(), h1: s.read_s16(), ..Default::default() };
        for i in 0..nl {
            let d = Descriptor(s.read_64());
            let v = s.read_u8();
            let w = s.read_u32();
            if i == sel {
                (r.desc, r.u8v, r.u32v) = (d, v, w);
            }
        }
        let k = s.read_s32();
        r.data = (0..k.max(0))
            .map(|_| {
                let mut e = [0u8; 8];
                s.read(8, Some(&mut e));
                e
            })
            .collect();
        b.recs.push(r);
    }
    let n48 = s.read_s32();
    b.recs48 = (0..n48.max(0)).map(|_| (s.read_s16(), s.read_64())).collect();
}

/// World+0x3a18 as filled by FUN_001d7b88.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Block3a18 {
    /// FUN_001d8690 entries (hash bucket = hash % 0x35): (u16 value, string)
    pub names: Vec<(u16, Option<Vec<u8>>)>,
    /// u16 count x 10-byte records
    pub recs: Vec<[u8; 10]>,
}

/// FUN_001d7b88 (stream part; FUN_001d7c80 post-processing against 0x3d4a60 is WP-B).
pub fn ee_sub_1d7b88(s: &mut BinaryInputStream) -> Block3a18 {
    // 0x001d7b88
    let n = s.read_u16();
    let names = (0..n)
        .map(|_| {
            let v = s.read_u16();
            (v, s.read_string())
        })
        .collect();
    let m = s.read_u16() as usize;
    let mut raw = vec![0u8; m * 10];
    s.read(m * 10, Some(&mut raw));
    Block3a18 { names, recs: raw.chunks(10).map(|c| c.try_into().unwrap()).collect() }
}

/// FUN_0013df80: world+0x4640/+0x4644 = n x (s16, language-selected string).
pub fn ee_sub_13df80(s: &mut BinaryInputStream, language: u8) -> Vec<(i16, Option<Vec<u8>>)> {
    // 0x0013df80
    let nl = s.read_u8() as i32;
    let mut sel = 0;
    for i in 0..nl {
        if s.read_u8() as i32 == language as i32 {
            sel = i;
        }
    }
    let n = s.read_s16() as i32;
    (0..n.max(0))
        .map(|_| {
            let k = s.read_s16();
            let mut text = None;
            for i in 0..nl {
                if i == sel { text = s.read_string() } else { s.skip_string() }
            }
            (k, text)
        })
        .collect()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Child {
    /// s16 >= 0: index of another node (+0x10/+0x14 pointer)
    Node(usize),
    /// s16 < 0: leaf number allocated in order (+0x18/+0x1c, counter world+0x4560)
    Leaf(i32),
}

/// FUN_0013e0e0: world+0x4558 nodes of 0x20 bytes: 16-byte key + two children.
pub fn ee_sub_13e0e0(s: &mut BinaryInputStream) -> (Vec<([u8; 16], Child, Child)>, i32) {
    // 0x0013e0e0
    let n = s.read_u16();
    let mut leaves = 0;
    let mut child = |s: &mut BinaryInputStream| {
        let v = s.read_s16();
        if v < 0 {
            leaves += 1;
            Child::Leaf(leaves - 1)
        } else {
            Child::Node(v as usize)
        }
    };
    let nodes = (0..n)
        .map(|_| {
            let k = s.read_16();
            let l = child(s);
            let r = child(s);
            (k, l, r)
        })
        .collect();
    (nodes, leaves)
}

/// Everything FUN_0013da18 reads before the first FUN_0013f810 object list.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LevelHeader {
    pub directory: Vec<DirEntry>,
    pub block_3dd040: Block3dd040,
    pub block_3a18: Block3a18,
    /// stream offset of the level table (u32 N)
    pub level_table_offset: u32,
    pub level_table: LevelTable,
    pub strings: Vec<(i16, Option<Vec<u8>>)>,
    pub tree: Vec<([u8; 16], Child, Child)>,
    /// world+0x4560
    pub tree_leaves: i32,
    /// stream offset where FUN_0013f810(0x3dd490, ..) starts (WP-B entry point)
    pub objects_offset: u32,
}

/// FUN_0013da18 up to object instantiation. `language` = DAT_002e1e54.
pub fn ee_str_spaprops_dbp_13da18(s: &mut BinaryInputStream, language: u8) -> LevelHeader {
    // 0x0013da18
    let mut h = LevelHeader { directory: read_directory(s), ..Default::default() };
    ee_sub_1fd0e8(&mut h.block_3dd040, s, language);
    h.block_3a18 = ee_sub_1d7b88(s);
    h.level_table_offset = s.tell();
    h.level_table = read_level_table(s);
    h.strings = ee_sub_13df80(s, language);
    (h.tree, h.tree_leaves) = ee_sub_13e0e0(s);
    h.objects_offset = s.tell();
    h
}

/// Pending level request at world+0x3bf4 (0x4c bytes): flags then the descriptor.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LevelRequest {
    /// +0x3bf4; (flags & 0xcd) == 0 -> keep the fade timer (world+0x476c)
    pub flags: u32,
    pub desc: Descriptor,
}

/// FUN_0020e890 stream setup: CD stream of exactly `size` raw bytes at `lsn`, LZ on
/// (FUN_001848a8 + FUN_00184d60 + FUN_001179d8 + FUN_00184ca0). None when the descriptor has
/// size 0 (the game then skips FUN_0013da18 and only resets the world).
pub fn open_level_stream<S: SectorSource + 'static>(disc: Arc<S>, desc: &Descriptor) -> Option<BinaryInputStream> {
    // 0x0020e890
    let loc = desc.location()?;
    let mut s = BinaryInputStream::new(Box::new(DiscSource::new(disc, loc.lsn, loc.size)));
    s.set_lz();
    Some(s)
}

/// Stream over already decompressed level bytes (mods / cached levels): the plain non-LZ path
/// of FUN_00117810.
pub fn memory_stream(data: Vec<u8>) -> BinaryInputStream {
    BinaryInputStream::new(Box::new(SliceSource::new(data, 0)))
}
