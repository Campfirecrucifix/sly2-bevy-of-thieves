//! Save game: the Splice save-variable blob (0x3d4a60, 0x5408 bytes) and its variable table
//! (world+0x3a18 = 0x3e0ea8, from the level file via FUN_001d7b88), plus the memory-card manager
//! (mcman, 0x2e6550 / slot structs of 0x5464 bytes) replaced by files in `saves/`.
//!
//! Blob layout (byte-exact with the PS2 card file `BASCUS-97316YAOTWTD!<n>`, 0x5408 bytes):
//!   +0x0000..+0x5400 variable data (offsets from the table records),
//!   +0x5400 u32 layout version = record[n-1].offset (u16 at +4 of the last 10-byte record),
//!   +0x5404 u32 layout checksum = byte sum of all n*10 record bytes.
//! FUN_001d7c80 / FUN_001d7d30 compare both and clear the data on mismatch (the original "checksum").
//! Values are little-endian (EE).

use crate::engine::disc::level_file::Block3a18;
use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};

pub const DATA_SIZE: usize = 0x5400;
/// bytes written to the card per slot (FUN_001ac420: FUN_001ac900(.., slot, 0x5408))
pub const SAVE_SIZE: usize = 0x5408;
/// in-memory slot struct (data + 0x5c-byte card directory entry at +0x5408)
pub const SLOT_STRUCT_SIZE: usize = 0x5464;
pub const SLOT_COUNT: usize = 3;
/// 0x2a6c60
pub const SAVE_DIR: &str = "BASCUS-97316YAOTWTD!";

/// One 10-byte variable record.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SaveVarRec {
    /// +0 symbol id
    pub sym: i16,
    /// +2 value type (-1 = struct/namespace with children)
    pub kind: i8,
    /// +3 != 0: array (indexed by the next path element; kind 4 = bit array)
    pub array: u8,
    /// +4 data offset
    pub offset: u16,
    /// +6 first child record
    pub child: u16,
    /// +8 next sibling record
    pub next: u16,
}

impl SaveVarRec {
    pub fn parse(b: &[u8; 10]) -> Self {
        let u = |o: usize| u16::from_le_bytes([b[o], b[o + 1]]);
        Self { sym: u(0) as i16, kind: b[2] as i8, array: b[3], offset: u(4), child: u(6), next: u(8) }
    }
}

/// The variable table at 0x3e0ea8.
#[derive(Clone, Debug, Default)]
pub struct SaveVars {
    pub raw: Vec<[u8; 10]>,
    pub recs: Vec<SaveVarRec>,
    /// +8 hash table (FUN_001d8720: 0x35 buckets, hash FUN_001d8640) — name -> symbol id
    pub names: HashMap<Vec<u8>, u16>,
}

/// A value read/written through the table (CRef payload kinds used by FUN_001d7ed8/FUN_001d80c8).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SaveValue {
    /// lookup failed / kind -1 (CRef tag -1)
    Nil,
    /// kinds 0,1,2,9,10 (u32 slot)
    Int(i32),
    /// kind 3
    Float(f32),
    /// kind 4 (word, or bit when `array`)
    Bool(bool),
    /// kinds 5 and 6 (16 bytes; vector / clq)
    Quad(u32, [u32; 4]),
    /// kind 7 (8 bytes)
    Pair([u32; 2]),
    /// kind 8 (12 bytes)
    Triple([u32; 3]),
}

/// FUN_001d8640: name hash (upper-cased chars, h = h*33 + c).
pub fn ee_sub_1d8640(name: &[u8]) -> i32 {
    // 0x001d8640
    let mut h: i32 = 0;
    for &c in name {
        let u = if c.is_ascii_lowercase() { c - 0x20 } else { c } as i32;
        h = h.wrapping_mul(0x21).wrapping_add(u);
    }
    h
}

impl SaveVars {
    pub fn from_block(b: &Block3a18) -> Self {
        let names = b.names.iter().filter_map(|(v, s)| s.clone().map(|s| (s, *v))).collect();
        Self { raw: b.recs.clone(), recs: b.recs.iter().map(SaveVarRec::parse).collect(), names }
    }

    /// FUN_001d8720: symbol id of a name, -1 if absent.
    pub fn ee_sub_1d8720(&self, name: &str) -> i32 {
        // 0x001d8720 (bucket = hash % 0x35, then strcmp)
        self.names.get(name.as_bytes()).map_or(-1, |&v| v as i16 as i32)
    }

    /// Layout (version, checksum) = (u16 at last record +4, byte sum of the table).
    pub fn signature(&self) -> (u32, u32) {
        let sum = self.raw.iter().flatten().fold(0u32, |a, &b| a.wrapping_add(b as u32));
        let ver = self.recs.last().map_or(0, |r| r.offset as u32);
        (ver, sum)
    }

    /// FUN_001d7da0: resolve a symbol path -> (record, array index).
    pub fn ee_sub_1d7da0(&self, path: &[u16]) -> Option<(SaveVarRec, u32)> {
        // 0x001d7da0
        let root = self.recs.first()?;
        let mut ui = root.child;
        if ui == 0 {
            return None;
        }
        let mut i = 0usize;
        loop {
            let p = *self.recs.get(ui as usize)?;
            // (long)s16 == (ulong)u16: negative symbols never match
            if path.get(i).is_some_and(|&s| p.sym as i32 == s as i32) {
                let j = i + 1;
                if j == path.len() {
                    return Some((p, 0));
                }
                if p.array != 0 {
                    return if i + 2 == path.len() { Some((p, path[j] as u32)) } else { None };
                }
                if p.kind != -1 {
                    return None;
                }
                ui = p.child;
                i = j;
            } else {
                ui = p.next;
            }
            if ui == 0 {
                return None;
            }
        }
    }

    /// FUN_001d7ed8: read a variable from `data`.
    pub fn ee_sub_1d7ed8(&self, path: &[u16], data: &GameData) -> SaveValue {
        // 0x001d7ed8
        let Some((r, idx)) = self.ee_sub_1d7da0(path) else { return SaveValue::Nil };
        let base = r.offset as usize;
        let w = |k: usize| data.u32_at(base + 4 * k);
        match r.kind {
            -1 => SaveValue::Nil,
            0 | 1 | 2 | 9 | 10 => SaveValue::Int(w(idx as usize) as i32),
            3 => SaveValue::Float(f32::from_bits(w(idx as usize))),
            4 if r.array == 0 => SaveValue::Bool(w(0) != 0),
            4 => SaveValue::Bool(data.byte(base + (idx >> 3) as usize) & (1 << (idx & 7)) != 0),
            5 | 6 => {
                let i = idx as usize * 4;
                SaveValue::Quad(r.kind as u32, [w(i), w(i + 1), w(i + 2), w(i + 3)])
            }
            7 => SaveValue::Pair([w(idx as usize * 2), w(idx as usize * 2 + 1)]),
            8 => {
                let i = idx as usize * 3;
                SaveValue::Triple([w(i), w(i + 1), w(i + 2)])
            }
            _ => SaveValue::Nil,
        }
    }

    /// FUN_001d80c8 (via FUN_00258200): write a variable into `data`. The value's payload is stored as-is
    /// for the variable's kind (no type check, like the original which reads CRef+4 blindly).
    /// Returns false when the path does not resolve (the original returns a Void CRef either way).
    pub fn ee_sub_1d80c8(&self, path: &[u16], v: SaveValue, data: &mut GameData) -> bool {
        // 0x001d80c8
        let Some((r, idx)) = self.ee_sub_1d7da0(path) else { return false };
        if r.kind == -1 {
            return false;
        }
        let base = r.offset as usize;
        let word = match v {
            SaveValue::Int(i) => i as u32,
            SaveValue::Float(f) => f.to_bits(),
            SaveValue::Bool(b) => b as u32,
            _ => 0,
        };
        match (r.kind, v) {
            (0 | 1 | 2 | 3 | 9 | 10, _) => data.set_u32(base + 4 * idx as usize, word),
            (4, _) if r.array == 0 => data.set_u32(base, word),
            (4, _) => {
                let p = base + (idx >> 3) as usize;
                let bit = 1u8 << (idx & 7);
                let b = data.byte(p);
                data.set_byte(p, if word == 0 { b & !bit } else { b | bit });
            }
            (5 | 6, SaveValue::Quad(_, q)) => {
                for (k, x) in q.iter().enumerate() {
                    data.set_u32(base + 16 * idx as usize + 4 * k, *x);
                }
            }
            (7, SaveValue::Pair(q)) => {
                for (k, x) in q.iter().enumerate() {
                    data.set_u32(base + 8 * idx as usize + 4 * k, *x);
                }
            }
            (8, SaveValue::Triple(q)) => {
                for (k, x) in q.iter().enumerate() {
                    data.set_u32(base + 12 * idx as usize + 4 * k, *x);
                }
            }
            _ => return false,
        }
        true
    }

    /// FUN_001d7d30: layout of `data` matches this table.
    pub fn ee_sub_1d7d30(&self, data: &GameData) -> bool {
        // 0x001d7d30
        let (v, s) = self.signature();
        data.version == v && data.checksum == s
    }

    /// FUN_001d7c80: on layout mismatch clear the data and stamp the new signature.
    pub fn ee_sub_1d7c80(&self, data: &mut GameData) {
        // 0x001d7c80
        let (v, s) = self.signature();
        if v != data.version || s != data.checksum {
            data.data.fill(0);
            data.checksum = s;
            data.version = v;
        }
    }

    /// FUN_001d84b0: copy the live blob (0x3d4a60) into a save slot and set every `from_memcard`
    /// record's first byte to 1 in the copy.
    pub fn ee_str_from_memcard_1d84b0(&self, live: &GameData) -> GameData {
        // 0x001d84b0
        let mut out = live.clone();
        let sym = self.ee_sub_1d8720("from_memcard");
        if sym != -1 {
            for r in &self.recs {
                if r.sym as i32 == sym {
                    out.set_byte(r.offset as usize, 1);
                }
            }
        }
        out
    }

    /// FUN_001d8d80(slot, fKeep): clear the data, keeping the 11 option variables (DAT_003e0fbc..fe4)
    /// when `keep` and the layout matches, then apply FUN_001d54b8 defaults.
    pub fn ee_sub_1d8d80(&self, data: &mut GameData, keep: bool, kept: &[u16], vibration: Option<(u16, i32)>) {
        // 0x001d8d80
        let saved: Vec<SaveValue> =
            if keep && self.ee_sub_1d7d30(data) { kept.iter().map(|&s| self.ee_sub_1d7ed8(&[s], data)).collect() } else { vec![] };
        data.data.fill(0);
        if let Some((sym, mode)) = vibration {
            ee_sub_1d54b8(self, data, sym, mode);
        }
        for (s, v) in kept.iter().zip(saved) {
            self.ee_sub_1d80c8(&[*s], v, data);
        }
    }
}

/// FUN_001d54b8: vibration option DAT_002c2d8c 2 -> fNoVibration (DAT_003e0fd8) = false, 3 -> true.
pub fn ee_sub_1d54b8(vars: &SaveVars, data: &mut GameData, sym_no_vibration: u16, mode: i32) {
    // 0x001d54b8
    match mode {
        2 => {
            vars.ee_sub_1d80c8(&[sym_no_vibration], SaveValue::Bool(false), data);
        }
        3 => {
            vars.ee_sub_1d80c8(&[sym_no_vibration], SaveValue::Bool(true), data);
        }
        _ => {}
    }
}

/// The 0x5408-byte blob (0x3d4a60 / one card file).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameData {
    pub data: Box<[u8; DATA_SIZE]>,
    /// +0x5400
    pub version: u32,
    /// +0x5404
    pub checksum: u32,
}

impl Default for GameData {
    fn default() -> Self {
        Self { data: Box::new([0; DATA_SIZE]), version: 0, checksum: 0 }
    }
}

impl GameData {
    fn byte(&self, o: usize) -> u8 {
        self.data.get(o).copied().unwrap_or(0)
    }
    fn set_byte(&mut self, o: usize, v: u8) {
        if let Some(b) = self.data.get_mut(o) {
            *b = v;
        }
    }
    fn u32_at(&self, o: usize) -> u32 {
        u32::from_le_bytes([self.byte(o), self.byte(o + 1), self.byte(o + 2), self.byte(o + 3)])
    }
    fn set_u32(&mut self, o: usize, v: u32) {
        for (k, b) in v.to_le_bytes().into_iter().enumerate() {
            self.set_byte(o + k, b);
        }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut v = Vec::with_capacity(SAVE_SIZE);
        v.extend_from_slice(&self.data[..]);
        v.extend_from_slice(&self.version.to_le_bytes());
        v.extend_from_slice(&self.checksum.to_le_bytes());
        v
    }
    pub fn from_bytes(b: &[u8]) -> Option<Self> {
        if b.len() != SAVE_SIZE {
            return None;
        }
        let mut d = Box::new([0u8; DATA_SIZE]);
        d.copy_from_slice(&b[..DATA_SIZE]);
        let w = |o: usize| u32::from_le_bytes(b[o..o + 4].try_into().unwrap());
        Some(Self { data: d, version: w(DATA_SIZE), checksum: w(DATA_SIZE + 4) })
    }
}

/// mcman error codes (FUN_001ac4c8 / FUN_001abfc0 results).
#[derive(Debug)]
pub enum McError {
    /// 0xe: file not found
    NoFile,
    /// 0x14: directory entry attributes not a readable file
    Attr,
    /// 0x1c: file size != 0x5408
    Size(u64),
    Io(io::Error),
}

/// Memory card manager replaced by a directory (`saves/` next to the exe). Paths keep the card layout:
/// `<root>/BASCUS-97316YAOTWTD!/BASCUS-97316YAOTWTD!<n>` (format "/%s/%s%d", n = slot + 1, FUN_001abb10).
/// icon.sys / memcard_icon.ico (FUN_001ac008 / FUN_001ac238) are not written (PS2 browser only).
#[derive(Clone, Debug)]
pub struct MemoryCard {
    pub root: PathBuf,
}

impl MemoryCard {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
    /// `saves/` next to the executable.
    pub fn next_to_exe() -> Self {
        let d = std::env::current_exe().ok().and_then(|p| p.parent().map(Path::to_path_buf)).unwrap_or_default();
        Self::new(d.join("saves"))
    }
    /// FUN_001abb10
    pub fn slot_path(&self, slot: usize) -> PathBuf {
        self.root.join(SAVE_DIR).join(format!("{SAVE_DIR}{}", slot + 1))
    }
    /// FUN_001ac420: write one slot (0x5408 bytes). FUN_001ab7d0 (create the save directory) first.
    pub fn ee_sub_1ac420(&self, slot: usize, d: &GameData) -> io::Result<()> {
        // 0x001ac420
        std::fs::create_dir_all(self.root.join(SAVE_DIR))?;
        let p = self.slot_path(slot);
        let tmp = p.with_extension("tmp");
        std::fs::write(&tmp, d.to_bytes())?;
        std::fs::rename(tmp, p)
    }
    /// FUN_001ac468: write an empty slot (FUN_001d8d80(slot, 0) then FUN_001ac420).
    pub fn ee_sub_1ac468(&self, slot: usize, vars: &SaveVars, d: &mut GameData) -> io::Result<()> {
        // 0x001ac468
        vars.ee_sub_1d8d80(d, false, &[], None);
        self.ee_sub_1ac420(slot, d)
    }
    /// FUN_001ac4c8: read one slot; size must be exactly 0x5408.
    pub fn ee_sub_1ac4c8(&self, slot: usize) -> Result<GameData, McError> {
        // 0x001ac4c8
        let p = self.slot_path(slot);
        let md = match std::fs::metadata(&p) {
            Ok(m) => m,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Err(McError::NoFile),
            Err(e) => return Err(McError::Io(e)),
        };
        if !md.is_file() {
            return Err(McError::Attr);
        }
        if md.len() != SAVE_SIZE as u64 {
            return Err(McError::Size(md.len()));
        }
        let b = std::fs::read(&p).map_err(McError::Io)?;
        GameData::from_bytes(&b).ok_or(McError::Size(b.len() as u64))
    }
}
