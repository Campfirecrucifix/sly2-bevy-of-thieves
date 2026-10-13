//! 64-byte xor-obfuscated file descriptors, the static descriptor table and the level table.
//!
//! Descriptor words w[0..16] (LE u32):  lsn = (w0 ^ w13) + 0xCA0, size = w1 ^ w15, ident = w2 ^ w14.
//! Decoded identically in FUN_001b3bd0 (static/FMV table rows), FUN_0020e890 (level open),
//! FUN_001fd0e8 (same-file check). Two descriptors name the same file when their idents match.

use super::lz::BinaryInputStream;

/// .rodata 0x2a3268: 30 static descriptors (rows 0-21 FMV / mpeg_ps), 64 bytes each.
pub const STATIC_TABLE_ADDR: u32 = 0x002a_3268;
pub const STATIC_TABLE_ROWS: usize = 30;
pub const LSN_BIAS: u32 = 0xca0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Descriptor(pub [u8; 64]);

impl Default for Descriptor {
    fn default() -> Self {
        Self([0; 64])
    }
}

/// FUN_00258248-initialised file location; kind 2 = CD (sector stream).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FileLocation {
    pub lsn: u32,
    pub size: u32,
    pub kind: u32,
}

pub const LOC_CD: u32 = 2;

impl Descriptor {
    pub fn w(&self, i: usize) -> u32 {
        u32::from_le_bytes(self.0[4 * i..4 * i + 4].try_into().unwrap())
    }
    /// (w0 ^ w13) + 0xCA0  — 0x0020e9xx / 0x001b3c4c
    pub fn lsn(&self) -> u32 {
        (self.w(0) ^ self.w(13)).wrapping_add(LSN_BIAS)
    }
    /// w1 ^ w15 (compressed byte count of the stream)
    pub fn size(&self) -> u32 {
        self.w(1) ^ self.w(15)
    }
    /// w2 ^ w14
    pub fn ident(&self) -> u32 {
        self.w(2) ^ self.w(14)
    }
    /// Same-file test used by FUN_0020e890 / FUN_001fd0e8 / FUN_001b3bd0.
    pub fn same_file(&self, o: &Descriptor) -> bool {
        self.ident() == o.ident()
    }
    /// FileLocation as built by FUN_0020e890: only valid when size != 0.
    pub fn location(&self) -> Option<FileLocation> {
        let size = self.size();
        (size != 0).then(|| FileLocation { lsn: self.lsn(), size, kind: LOC_CD })
    }
}

/// Minimal ELF32 vaddr -> bytes (to read .rodata tables from SCUS_973.16).
pub fn elf_bytes(elf: &[u8], vaddr: u32, len: usize) -> Option<&[u8]> {
    let u16at = |o: usize| -> Option<usize> { Some(u16::from_le_bytes(elf.get(o..o + 2)?.try_into().ok()?) as usize) };
    let u32at = |o: usize| -> Option<u32> { Some(u32::from_le_bytes(elf.get(o..o + 4)?.try_into().ok()?)) };
    if elf.get(0..4)? != b"\x7fELF" {
        return None;
    }
    let phoff = u32at(0x1c)? as usize;
    let phentsize = u16at(0x2a)?;
    let phnum = u16at(0x2c)?;
    for i in 0..phnum {
        let p = phoff + i * phentsize;
        let (off, va, filesz) = (u32at(p + 4)?, u32at(p + 8)?, u32at(p + 16)?);
        if vaddr >= va && (vaddr - va) as usize + len <= filesz as usize {
            let s = (off + vaddr - va) as usize;
            return elf.get(s..s + len);
        }
    }
    None
}

/// The 30-row static table at 0x2a3268 (FMVs 0-21, then the boot levels / data file).
pub fn static_table(elf: &[u8]) -> Option<Vec<Descriptor>> {
    let b = elf_bytes(elf, STATIC_TABLE_ADDR, STATIC_TABLE_ROWS * 64)?;
    Some(b.chunks(64).map(|c| Descriptor(c.try_into().unwrap())).collect())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LevelTableEntry {
    /// DAT_003e1c40[i]
    pub desc: Descriptor,
    /// DAT_003e1c44[i]: world/level index (slot index into DAT_003e1c50 when `flag` != 0)
    pub idx: u8,
    /// DAT_003e1c48[i]
    pub b1: u8,
    /// nonzero: desc is also copied to the slot table DAT_003e1c50[idx]
    pub flag: u8,
    /// DAT_003e1c4c[i] = {count, u16 *links}
    pub links: Vec<u16>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LevelTable {
    /// DAT_003e1c3c (second u32 of the header)
    pub x: u32,
    pub entries: Vec<LevelTableEntry>,
    /// DAT_003e1c50: 0x240 bytes = 9 descriptor slots
    pub slots: [Descriptor; 9],
}

/// FUN_002109d0 (WP-C9): row of `desc` in the level table (first same-file row, +8 ^ +0x38), -1 when none.
pub fn ee_sub_2109d0(t: &LevelTable, desc: &Descriptor) -> i32 {
    // 0x002109d0
    t.entries.iter().position(|e| e.desc.same_file(desc)).map_or(-1, |i| i as i32)
}

/// FUN_0020f318: read the level table from the level stream.
pub fn read_level_table(s: &mut BinaryInputStream) -> LevelTable {
    // 0x0020f318
    let n = s.read_u32() as i32;
    let x = s.read_u32();
    let mut t = LevelTable { x, ..Default::default() };
    for _ in 0..n.max(0) {
        let desc = Descriptor(s.read_64());
        let idx = s.read_u8();
        let b1 = s.read_u8();
        let flag = s.read_u8();
        if flag != 0 {
            // Quirk: the game writes 64 bytes at slots + idx*0x40 with no bound (0x240 buffer);
            // out-of-range indices would corrupt the heap there. We drop them.
            if let Some(slot) = t.slots.get_mut(idx as usize) {
                *slot = desc;
            }
        }
        let k = s.read_u8();
        let links = (0..k).map(|_| s.read_u16()).collect();
        t.entries.push(LevelTableEntry { desc, idx, b1, flag, links });
    }
    t
}
