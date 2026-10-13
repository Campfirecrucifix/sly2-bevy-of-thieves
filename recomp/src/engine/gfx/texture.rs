//! WP-C: level textures. Per directory entry FUN_001eb748 loads (globals reset per entry):
//!   FUN_001eae40 -> CLUT table DAT_003dcffc (0x20 each, FUN_001ea8a8),
//!   FUN_001eb098 -> bitmap table DAT_003dcff4 (0x28 each, FUN_001eaf50),
//!   FUN_001ed7a8 objects, FUN_001eb2f0 -> shader table DAT_003dcfec (0x2c each) whose
//!   FUN_001eb1a8 frames (0x14 each, "TEX") point at bitmaps and CLUTs,
//!   FUN_0020f640 blob: records with kind (+4) == 1 keep a blob offset instead of own data
//!   (FUN_001ea568 rebases them).
//! GS upload (FUN_001ece58 / FUN_001ecc28 -> FUN_00170650 BITBLTBUF/TRXREG/TRXDIR + IMAGE) sends a
//! bitmap with its own PSM/TBW and a CLUT as a PSMCT32 image 8 wide (<= 16 colours) or 16 wide.
//! A host->local transfer and a texture read with the same PSM/TBW cancel the GS swizzle, so
//! bitmap bytes are linear in their PSM; only the CLUT CSM1 layout must be applied
//! (FUN_001ec9d0: TEX2 PSM = T4 if <= 16 colours else T8, CPSM = CT32, CSM = CSM1, CLD = 1).
//! The GS itself (VRAM allocator FUN_00170548, packets) is replaced by RGBA8 images.

use crate::engine::world::entry::{EntrySections, Rec1ea8a8, Rec1eaf50, Rec1eb2f0};

/// GS pixel storage modes used by the bitmap PSM byte (+0xc).
pub const PSMCT32: u8 = 0x00;
pub const PSMCT24: u8 = 0x01;
pub const PSMCT16: u8 = 0x02;
pub const PSMCT16S: u8 = 0x0a;
pub const PSMT8: u8 = 0x13;
pub const PSMT4: u8 = 0x14;
pub const PSMT8H: u8 = 0x1b;
pub const PSMT4HL: u8 = 0x24;
pub const PSMT4HH: u8 = 0x2c;

fn u16le(b: &[u8], o: usize) -> u16 {
    u16::from_le_bytes([b[o], b[o + 1]])
}
fn u32le(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes(b[o..o + 4].try_into().unwrap())
}

/// Resolve a record's pixel/colour bytes: own data, or (kind 1) `len` bytes at blob + offset
/// (FUN_001ea568). None = filtered out / missing.
fn resolve<'a>(kind: u8, own: Option<&'a Vec<u8>>, blob_off: u32, len: usize, level: &'a [u8], blob: (u32, i32)) -> Option<&'a [u8]> {
    if kind == 1 {
        let start = blob.0 as usize + blob_off as usize;
        let end = start.checked_add(len)?;
        if blob_off as i64 + len as i64 > blob.1 as i64 || end > level.len() {
            return None;
        }
        Some(&level[start..end])
    } else {
        own.map(|v| v.as_slice())
    }
}

/// CLUT record (CLUTF of Sly 1): +0 id, +4 kind, +5 filter count, +8 u16 colours,
/// +0xa u16 GS blocks, +0xc data (blob offset when kind 1).
#[derive(Clone, Debug, PartialEq)]
pub struct Clut {
    pub id: u32,
    pub kind: u8,
    pub colors: u16,
    /// RGBA as stored (PS2 alpha, 0x80 = 1.0); FUN_001ed278 recolour is only active with the
    /// DAT_003e12c8 option (off by default), not applied.
    pub rgba: Option<Vec<[u8; 4]>>,
}

pub fn clut_from(r: &Rec1ea8a8, level: &[u8], blob: (u32, i32)) -> Clut {
    let h = &r.head;
    let colors = u16le(h, 8);
    let data = resolve(h[4], r.data.as_ref(), u32le(h, 0xc), colors as usize * 4, level, blob);
    Clut {
        id: u32le(h, 0),
        kind: h[4],
        colors,
        rgba: data.map(|d| d.chunks_exact(4).map(|c| [c[0], c[1], c[2], c[3]]).collect()),
    }
}

/// Bitmap record (BMPF): +0 id, +4 kind, +5 filter count, +8 u16 width, +0xa u16 height,
/// +0xc u8 PSM, +0xd u8 TBW, +0xe u16 GS blocks, +0x10 s32 bytes, +0x14 data.
#[derive(Clone, Debug, PartialEq)]
pub struct Bitmap {
    pub id: u32,
    pub kind: u8,
    pub width: u16,
    pub height: u16,
    pub psm: u8,
    pub tbw: u8,
    pub pixels: Option<Vec<u8>>,
}

pub fn bitmap_from(r: &Rec1eaf50, level: &[u8], blob: (u32, i32)) -> Bitmap {
    let h = &r.head;
    let len = (u32le(h, 0x10) as i32).max(0) as usize;
    let data = resolve(h[4], r.data.as_ref(), u32le(h, 0x14), len, level, blob);
    Bitmap { id: u32le(h, 0), kind: h[4], width: u16le(h, 8), height: u16le(h, 10), psm: h[0xc], tbw: h[0xd], pixels: data.map(|d| d.to_vec()) }
}

/// GS CSM1 CLUT addressing for an 8-bit index: the 256-entry palette is stored in 16x16 CT32
/// blocks with bits 3 and 4 of the index swapped (the 16-colour 8x2 layout is linear).
pub fn csm1_index(i: u8) -> usize {
    ((i & 0xe7) | ((i & 0x08) << 1) | ((i & 0x10) >> 1)) as usize
}

/// PS2 alpha (0x80 = 1.0, up to 0xff) -> 8-bit unorm, saturating.
pub fn ps2_alpha(a: u8) -> u8 {
    ((a as u32 * 255 + 64) / 128).min(255) as u8
}

#[derive(Clone, Debug, PartialEq)]
pub enum DecodeError {
    NoPixels,
    NoClut,
    Short { need: usize, have: usize },
    UnknownPsm(u8),
}

/// RGBA8 image (sRGB), alpha already rescaled from the PS2 0x80 = 1.0 convention.
#[derive(Clone, Debug, PartialEq)]
pub struct TexImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl TexImage {
    /// true when some texel is not fully opaque (PS2 alpha < 0x80)
    pub fn has_alpha(&self) -> bool {
        self.rgba.chunks_exact(4).any(|p| p[3] < 0xff)
    }
}

/// Decode a bitmap with an optional CLUT (CT32 palette, CSM1) to RGBA8.
pub fn decode(b: &Bitmap, clut: Option<&Clut>) -> Result<TexImage, DecodeError> {
    let px = b.pixels.as_deref().ok_or(DecodeError::NoPixels)?;
    let (w, h) = (b.width as usize, b.height as usize);
    let n = w * h;
    let bits = match b.psm {
        PSMCT32 => 32,
        PSMCT24 => 24,
        PSMCT16 | PSMCT16S => 16,
        PSMT8 | PSMT8H => 8,
        PSMT4 | PSMT4HL | PSMT4HH => 4,
        p => return Err(DecodeError::UnknownPsm(p)),
    };
    let need = (n * bits).div_ceil(8);
    if px.len() < need {
        return Err(DecodeError::Short { need, have: px.len() });
    }
    let mut out = Vec::with_capacity(n * 4);
    let pal = |idx: usize, csm1: bool| -> Result<[u8; 4], DecodeError> {
        let c = clut.and_then(|c| c.rgba.as_ref()).ok_or(DecodeError::NoClut)?;
        let i = if csm1 { csm1_index(idx as u8) } else { idx };
        Ok(c.get(i).copied().unwrap_or([0, 0, 0, 0]))
    };
    for i in 0..n {
        let c = match bits {
            32 => [px[4 * i], px[4 * i + 1], px[4 * i + 2], px[4 * i + 3]],
            // TEXA.TA0 (0x80) for CT24 / CT16 with A bit 0 -> TA0 = 0, A bit 1 -> TA1 = 0x80
            24 => [px[3 * i], px[3 * i + 1], px[3 * i + 2], 0x80],
            16 => {
                let v = u16le(px, 2 * i);
                let e = |x: u16| (((x & 0x1f) << 3) | ((x & 0x1f) >> 2)) as u8;
                [e(v), e(v >> 5), e(v >> 10), if v & 0x8000 != 0 { 0x80 } else { 0 }]
            }
            8 => pal(px[i] as usize, true)?,
            _ => {
                let byte = px[i / 2];
                // PSMT4: low nibble = even pixel
                pal(if i & 1 == 0 { byte & 0xf } else { byte >> 4 } as usize, false)?
            }
        };
        out.extend_from_slice(&[c[0], c[1], c[2], ps2_alpha(c[3])]);
    }
    Ok(TexImage { width: w as u32, height: h as u32, rgba: out })
}

/// One FUN_001eb1a8 texture frame resolved to table indices.
#[derive(Clone, Debug, PartialEq)]
pub struct TexFrame {
    /// head +0 (TEX OID): used as the mod override id
    pub id: u32,
    /// head +4: bit 0 = CLAMP_1 WMS clamp, bit 1 = WMT clamp (FUN_001ea750)
    pub flags: u16,
    pub bitmaps: Vec<u16>,
    /// None = 0xffff
    pub cluts: Vec<Option<u16>>,
}

/// FUN_001ea688: CLUT sets per bitmap (shader passes reading different palettes).
pub fn ee_sub_1ea688(kind: u8, flags: u8) -> usize {
    // 0x001ea688
    match kind {
        1 => 2,
        7 => {
            if flags & 0x20 == 0 {
                3
            } else {
                2
            }
        }
        _ => 1,
    }
}

/// Shader (SHDF): +0 id, +4 kind, +5 flags, +8/+0xc/+0x10 RGBA colours, +0x17 frame count.
#[derive(Clone, Debug, PartialEq)]
pub struct Shader {
    pub id: u32,
    pub kind: u8,
    pub flags: u8,
    pub colors: [[u8; 4]; 3],
    pub frames: Vec<TexFrame>,
}

pub fn shader_from(r: &Rec1eb2f0) -> Shader {
    let h = &r.head;
    let c = |o: usize| [h[o], h[o + 1], h[o + 2], h[o + 3]];
    Shader {
        id: u32le(h, 0),
        kind: h[4],
        flags: h[5],
        colors: [c(8), c(0xc), c(0x10)],
        frames: r
            .subs
            .iter()
            .map(|f| TexFrame {
                id: u32le(&f.head, 0),
                flags: u16le(&f.head, 4),
                bitmaps: f.refs_eb098.clone(),
                cluts: f.refs_eae40.iter().map(|&c| (c != 0xffff).then_some(c)).collect(),
            })
            .collect(),
    }
}

impl Shader {
    /// The palette set used by the lit base pass: kinds 1/7 draw CLUT set 1 (diffuse) first
    /// (FUN_001eb960 `FUN_001ea750(frame, 1, ..)`), others set 0.
    pub fn base_pass(&self) -> usize {
        match self.kind {
            1 | 7 => 1,
            _ => 0,
        }
    }
    /// FUN_001ea6c8: bitmap 0 with CLUT [0 * sets + pass] (TEX0 written last for index 0).
    pub fn frame_tex(&self, frame: usize, pass: usize) -> Option<(u16, Option<u16>)> {
        let f = self.frames.get(frame)?;
        let b = *f.bitmaps.first()?;
        let sets = ee_sub_1ea688(self.kind, self.flags);
        Some((b, f.cluts.get(pass.min(sets - 1)).copied().flatten()))
    }
}

/// All texture tables of one directory entry.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EntryTextures {
    pub cluts: Vec<Clut>,
    pub bitmaps: Vec<Bitmap>,
    pub shaders: Vec<Shader>,
}

impl EntryTextures {
    pub fn from_sections(e: &EntrySections, level: &[u8]) -> Self {
        let blob = (e.blob_offset, e.blob_size);
        EntryTextures {
            cluts: e.eae40.iter().map(|r| clut_from(r, level, blob)).collect(),
            bitmaps: e.eb098.iter().map(|r| bitmap_from(r, level, blob)).collect(),
            shaders: e.eb2f0.iter().map(shader_from).collect(),
        }
    }

    /// Decode shader `s` frame `f` for its base pass.
    pub fn decode_shader(&self, s: usize, f: usize) -> Option<Result<TexImage, DecodeError>> {
        let sh = self.shaders.get(s)?;
        self.decode_shader_pass(s, f, sh.base_pass())
    }

    /// Decode shader `s` frame `f` with CLUT set `pass` (FUN_001ea750 second argument; WP-C3
    /// extra passes: kinds 1/7 additive layer = set 0, kind 7 Z pre-pass = set 2).
    pub fn decode_shader_pass(&self, s: usize, f: usize, pass: usize) -> Option<Result<TexImage, DecodeError>> {
        let sh = self.shaders.get(s)?;
        let (b, c) = sh.frame_tex(f, pass)?;
        let bmp = self.bitmaps.get(b as usize)?;
        let needs_clut = matches!(bmp.psm, PSMT8 | PSMT8H | PSMT4 | PSMT4HL | PSMT4HH);
        let clut = c.and_then(|c| self.cluts.get(c as usize));
        if needs_clut && clut.is_none() {
            return Some(Err(DecodeError::NoClut));
        }
        Some(decode(bmp, clut))
    }
}

/// Mod override path (WP-A ModOverlay relative path): `textures/<level lsn>/<TEX id>.png`.
pub fn texture_override_path(lsn: u32, tex_id: u32) -> String {
    format!("textures/{lsn}/{tex_id}.png")
}
