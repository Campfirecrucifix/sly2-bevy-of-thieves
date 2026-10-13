#![allow(non_snake_case)]
//! WP-I3: game text. Level strings (world+0x4640 count, +0x4644 n x {s32 id, char *str}, loaded by
//! FUN_0013df80 = `disc::level_file::ee_sub_13df80`, already parsed into `LevelHeader.strings`) and the
//! runtime string table at 0x3e1c20 that FUN_001b3700 (text panel) reads through FUN_00210ca8.
//!
//! Strings are the raw cp1252 bytes of the disc (0x92 etc., `&2X&.` button escapes, printf `%d`), which is
//! what the font char map (`ui::font`) indexes, so they are kept as bytes.

/// DAT_002a4be0 / DAT_002a8a98: the empty string returned on a miss.
pub const EMPTY: &[u8] = b"";

/// world+0x4640/+0x4644.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LevelStrings(pub Vec<(i32, Vec<u8>)>);

impl LevelStrings {
    /// From `LevelHeader.strings` (s16 id, string or None for an empty slot).
    pub fn from_header(strings: &[(i16, Option<Vec<u8>>)]) -> Self {
        LevelStrings(strings.iter().map(|(id, s)| (*id as i32, s.clone().unwrap_or_default())).collect())
    }

    /// FUN_0013e368(world, id): -1 -> ""; first entry with the id (linear scan); missing -> "".
    pub fn FUN_0013e368(&self, id: i32) -> &[u8] {
        if id == -1 {
            return EMPTY;
        }
        self.0.iter().find(|(k, _)| *k == id).map_or(EMPTY, |(_, s)| s.as_slice())
    }
}

/// The table at 0x3e1c20: +0 count, +4 entries {s32 id, char *str}, +8 arena size, +0xc arena base.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StringTable {
    /// +0 / +4: entries (str None = 0 pointer)
    pub entries: Vec<(i32, Option<Vec<u8>>)>,
}

impl StringTable {
    /// FUN_00210cf0(1, 0xffff) part for the table: count = 0, entries = DAT_002c3eb8, arena (0, 0x3e1c60).
    /// (The pool inits FUN_0011cca8 of the same function are allocator setup, not needed in Rust.)
    pub fn FUN_00210cf0() -> Self {
        StringTable { entries: Vec::new() }
    }

    /// FUN_00210be0(table): only when the level has strings (DAT_003e1ad0 = world+0x4640 != 0): every entry's
    /// string = FUN_0013e368(world, id); empty -> 0, else copied into the arena (FUN_001169d8, align 4).
    pub fn FUN_00210be0(&mut self, level: &LevelStrings) {
        if level.0.is_empty() {
            return;
        }
        for e in &mut self.entries {
            let s = level.FUN_0013e368(e.0);
            e.1 = if s.is_empty() { None } else { Some(s.to_vec()) };
        }
    }

    /// FUN_00210ca8(table, id): first entry with the id and a non-0 string, else "".
    pub fn FUN_00210ca8(&self, id: i32) -> &[u8] {
        self.entries.iter().find(|(k, s)| *k == id && s.is_some()).and_then(|(_, s)| s.as_deref()).unwrap_or(EMPTY)
    }

    /// Port addition (see Findings I3): in SCUS_973.16 nothing ever stores the count (+0) after
    /// FUN_00210cf0 zeroes it, so FUN_00210ca8 always returns "" and the FUN_001b3700 panel is blank in
    /// retail. The port fills the entry list with every level string id (as if the count were set) so the
    /// panel shows the level text for its id; FUN_00210be0 then resolves them 1:1.
    pub fn with_level_ids(level: &LevelStrings) -> Self {
        let mut t = StringTable { entries: level.0.iter().map(|(id, _)| (*id, None)).collect() };
        t.FUN_00210be0(level);
        t
    }
}

/// cp1252 bytes -> String (for logs / tests only; drawing uses the bytes).
pub fn to_string(b: &[u8]) -> String {
    b.iter()
        .map(|&c| match c {
            0x80 => '€',
            0x92 => '\u{2019}',
            0x93 => '\u{201c}',
            0x94 => '\u{201d}',
            0x96 => '\u{2013}',
            0x97 => '\u{2014}',
            0x85 => '\u{2026}',
            c => c as char,
        })
        .collect()
}
