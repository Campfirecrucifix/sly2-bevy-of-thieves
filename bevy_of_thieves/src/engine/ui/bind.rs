//! WP-I3: dynamic text bindings of the rich text system. Table 0x2a8b90 (0x6c rows x 16 B {s32 id, provider,
//! arg, 0}, terminator id 0x7f7fffff) is copied by FUN_0021aa60 to 0x3e1c88 (12-byte {id, fn, arg}).
//! FUN_00216bc8 resolves a text item's string: kind (+0x14) 0 = level string id (FUN_0013e368), 1 = binding
//! id looked up in the table and formatted by the provider `fn(buf, arg, item+8)`, unknown id -> "Unknown".
//!
//! The providers read engine globals; `BindCtx` carries the values the port has (others are `None` and the
//! provider writes nothing, which is also what the originals do when their guard fails).
#![allow(non_snake_case)]
use super::strings::LevelStrings;

pub const TABLE: u32 = 0x002a_8b90;
pub const ROWS: usize = 0x6c;

/// One row of 0x3e1c88.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Binding {
    pub id: i32,
    pub provider: u32,
    pub arg: u32,
}

/// FUN_0021aa60 (init part): read the 0x6c rows of 0x2a8b90 from SCUS_973.16 (`rd(va, n)` = ELF reader).
pub fn FUN_0021aa60(rd: impl Fn(u32, usize) -> Option<Vec<u8>>) -> Option<Vec<Binding>> {
    let b = rd(TABLE, ROWS * 16)?;
    let w = |o: usize| u32::from_le_bytes(b[o..o + 4].try_into().unwrap());
    Some((0..ROWS).map(|i| Binding { id: w(i * 16) as i32, provider: w(i * 16 + 4), arg: w(i * 16 + 8) }).collect())
}

/// Read `n` bytes at virtual address `va` of a MIPS ELF (PT_LOAD segments; bss reads as 0).
pub fn elf_read(elf: &[u8], va: u32, n: usize) -> Option<Vec<u8>> {
    let u32_ = |o: usize| elf.get(o..o + 4).map(|b| u32::from_le_bytes(b.try_into().unwrap()));
    let u16_ = |o: usize| elf.get(o..o + 2).map(|b| u16::from_le_bytes(b.try_into().unwrap()));
    let phoff = u32_(0x1c)? as usize;
    let phnum = u16_(0x2c)? as usize;
    for i in 0..phnum {
        let p = phoff + i * 32;
        let (t, off, vaddr, fsz, msz) = (u32_(p)?, u32_(p + 4)?, u32_(p + 8)?, u32_(p + 16)?, u32_(p + 20)?);
        if t == 1 && vaddr <= va && (va as u64 + n as u64) <= vaddr as u64 + msz as u64 {
            let o = (va - vaddr) as usize;
            let mut out = vec![0u8; n];
            let avail = (fsz as usize).saturating_sub(o).min(n);
            out[..avail].copy_from_slice(elf.get(off as usize + o..off as usize + o + avail)?);
            return Some(out);
        }
    }
    None
}

/// Menu frame view for FUN_00211298 (top of the menu manager stack, `ui::menu`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MenuView {
    /// string ids of the top frame's items (item +0)
    pub ids: Vec<i32>,
    /// frame +0x14 selection
    pub sel: i32,
    /// WP-I4 port rows: literal text replacing the string-table lookup (None = use `ids`)
    pub text: Vec<Option<Vec<u8>>>,
}

/// Engine values the providers read.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BindCtx {
    /// save variable behind DAT_003e0fb0 ("coins"); None = path 0
    pub coins: Option<i32>,
    /// DAT_003e1bf4 / DAT_003e1bf8
    pub count: (i32, i32),
    /// DAT_003e1bfc (timer end) and DAT_002dded0 (clock)
    pub timer_end: f32,
    pub clock: f32,
    /// top menu frame (None = DAT_003d9edc[depth] count 0)
    pub menu: Option<MenuView>,
    /// DAT_003da0d8 (marquee string id, binding 0x2f1)
    pub marquee: i32,
    /// completion save variable of the current episode (FUN_0015d8f8) and whether FUN_001daea8 adds 5
    pub completion: Option<(i32, bool)>,
    /// play time save variable (FUN_0015d9c0), seconds
    pub play_time: Option<f32>,
}

/// FUN_00216bc8 string part. `kind` = item +0x14, `id` = +0x18. Returns the bytes of the 128-byte buffer.
pub fn FUN_00216bc8_text(table: &[Binding], strings: &LevelStrings, ctx: &BindCtx, kind: i32, id: i32) -> Vec<u8> {
    match kind {
        1 => {
            // linear search of the 0x6c rows (the terminator row is never matched: loop ends at 0x6c)
            match table.iter().take(ROWS).find(|b| b.id == id) {
                Some(b) => provide(b, strings, ctx),
                None => b"Unknown".to_vec(), // s_Unknown_002a8b80
            }
        }
        0 => strings.FUN_0013e368(id).to_vec(),
        _ => Vec::new(),
    }
}

/// Dispatch to the ported providers.
pub fn provide(b: &Binding, strings: &LevelStrings, ctx: &BindCtx) -> Vec<u8> {
    match b.provider {
        0x0021_0ed8 => FUN_00210ed8(ctx),
        0x0021_0f50 => FUN_00210f50(ctx),
        0x0021_0f80 => FUN_00210f80(ctx),
        0x0021_1298 => FUN_00211298(strings, ctx, b.arg as i32),
        0x0021_13e0 => strings.FUN_0013e368(ctx.marquee).to_vec(), // FUN_002113e0
        0x0015_d8f8 => FUN_0015d8f8(ctx),
        0x0015_d9c0 => FUN_0015d9c0(ctx),
        // FUN_00210eb0 strcpy(buf, arg): the debug rows' arg is a "Widget Item N" string; not resolvable
        // without the ELF here -> empty. Other providers (player / shop / save-slot state) are unported.
        _ => Vec::new(),
    }
}

/// FUN_00210ed8 (binding 0x2cf, arg &DAT_003e0fb0): sprintf("%d", var) when the path is set.
pub fn FUN_00210ed8(ctx: &BindCtx) -> Vec<u8> {
    ctx.coins.map_or_else(Vec::new, |v| format!("{v}").into_bytes())
}

/// FUN_00210f50 (0x2d0): sprintf("%d/%d", DAT_003e1bf4, DAT_003e1bf8).
pub fn FUN_00210f50(ctx: &BindCtx) -> Vec<u8> {
    format!("{}/{}", ctx.count.0, ctx.count.1).into_bytes()
}

/// FUN_00210f80 (0x2d1): s = (int)(DAT_003e1bfc - clock); sprintf("%d:%02d", s / 60, s % 60) (C truncation).
pub fn FUN_00210f80(ctx: &BindCtx) -> Vec<u8> {
    let s = (ctx.timer_end - ctx.clock) as i32;
    format!("{}:{:02}", s / 60, s % 60).into_bytes()
}

/// FUN_00211298 (0x2d2..0x2db, arg = row): idx = row + sel - 3, wrapped into [0, count) -> level string of
/// item idx. Count 0 -> nothing.
pub fn FUN_00211298(strings: &LevelStrings, ctx: &BindCtx, row: i32) -> Vec<u8> {
    let Some(m) = &ctx.menu else { return Vec::new() };
    let n = m.ids.len() as i32;
    if n == 0 {
        return Vec::new();
    }
    let mut i = row + m.sel - 3;
    while i < 0 {
        i += n;
    }
    let i = (i % n) as usize;
    match m.text.get(i) {
        Some(Some(t)) => t.clone(),
        _ => strings.FUN_0013e368(m.ids[i]).to_vec(),
    }
}

/// FUN_0015d8f8 (0x52f): sprintf("%d%%", v * 95 / 100 (+5 when FUN_001daea8)).
pub fn FUN_0015d8f8(ctx: &BindCtx) -> Vec<u8> {
    ctx.completion.map_or_else(Vec::new, |(v, bonus)| {
        let p = v * 0x5f / 100;
        format!("{}%", if bonus { p + 5 } else { p }).into_bytes()
    })
}

/// FUN_0015d9c0 (0x530): sprintf("%01d:%02d", (int)(t / 3600), (int)(fmod(t, 3600) / 60)).
pub fn FUN_0015d9c0(ctx: &BindCtx) -> Vec<u8> {
    ctx.play_time.map_or_else(Vec::new, |t| format!("{}:{:02}", (t * 0.000_277_777_78) as i32, ((t % 3600.0) * 0.016_666_668) as i32).into_bytes())
}
