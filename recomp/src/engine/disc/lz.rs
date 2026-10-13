//! CBinaryInputStream (vtable DAT_002ca5c0 base / DAT_002d2918 CD stream) with its LZ refill.
//!
//! 1:1 port of the game's binary input stream: field comments give the original struct offset.
//! The raw byte supply (vtable slot +0xc: the CD stream's sector refill, FUN_001849a0/FUN_00184c60
//! over sceCdRead) is replaced by the [`RawSource`] trait; everything above it is the game's logic.
//!
//! LZ format (FUN_001179f0): flag byte, LSB first; bit 1 = literal, bit 0 = 16-bit LE ref v:
//! count = (v >> 13) + 3, src = v & 0x1FFF in an 8 KiB window. The game decodes into a
//! 0x2000-byte block at buf[0..], lets the last ref overrun into buf[0x2000..] and memmoves the
//! overrun ("carry") back to buf[0] before decoding the next block.

/// Raw (compressed or plain) byte supply. Replaces the CD stream's vtable refill (+0xc).
/// Returning an empty Vec = end of data (the game sees +0x14 < 1).
pub trait RawSource: Send {
    fn refill(&mut self) -> Vec<u8>;
}

/// In-memory source delivering `data` in chunks of `chunk` bytes (chunk = 0 -> one chunk).
pub struct SliceSource {
    data: Vec<u8>,
    pos: usize,
    chunk: usize,
}

impl SliceSource {
    pub fn new(data: Vec<u8>, chunk: usize) -> Self {
        Self { data, pos: 0, chunk }
    }
}

impl RawSource for SliceSource {
    fn refill(&mut self) -> Vec<u8> {
        let n = if self.chunk == 0 { self.data.len() - self.pos } else { self.chunk.min(self.data.len() - self.pos) };
        let v = self.data[self.pos..self.pos + n].to_vec();
        self.pos += n;
        v
    }
}

/// LZ block size (0x2000) and the decode buffer size the game reserves (FUN_001179d8 is passed
/// 0x4000 bytes of scratchpad at 0x70000000; only 0x2000 + max overrun is ever touched).
pub const LZ_BLOCK: usize = 0x2000;
const LZ_BUF: usize = 0x4000;

/// Flag bit at +0x00: stream is LZ-compressed (set by FUN_001179d8).
pub const STREAM_LZ: u32 = 2;

pub struct BinaryInputStream {
    /// +0x00 flags (bit 1 = LZ)
    pub flags: u32,
    /// +0x04 read cursor into the current output buffer
    cur: usize,
    /// +0x08 bytes left in the output buffer (-1 = EOF hit)
    avail: i32,
    /// +0x0c total bytes delivered into output buffers so far
    total: u32,
    /// +0x10 / +0x14 raw input pointer + bytes left
    raw: Vec<u8>,
    raw_pos: usize,
    raw_left: i32,
    /// +0x18 LZ flag shift register
    lz_flags: u32,
    /// +0x1c LZ overrun carried to the next block
    carry: i32,
    /// +0x24 LZ decode buffer (+0x20 = its size)
    lz_buf: Vec<u8>,
    /// +0x30 vtable -> refill
    src: Box<dyn RawSource>,
}

impl BinaryInputStream {
    /// FUN_001177a8 (base ctor) + FUN_001848a8 (CD stream ctor): all zero, no LZ.
    pub fn new(src: Box<dyn RawSource>) -> Self {
        Self { flags: 0, cur: 0, avail: 0, total: 0, raw: Vec::new(), raw_pos: 0, raw_left: 0, lz_flags: 0, carry: 0, lz_buf: Vec::new(), src }
    }

    /// FUN_001179d8: switch the stream to LZ decoding with a decode buffer.
    pub fn set_lz(&mut self) {
        self.lz_buf = vec![0u8; LZ_BUF];
        self.flags |= STREAM_LZ;
    }

    fn buf(&self) -> &[u8] {
        if self.flags & STREAM_LZ != 0 { &self.lz_buf } else { &self.raw }
    }

    /// vtable +0xc: refill the raw input (+0x10/+0x14).
    fn refill_raw(&mut self) {
        self.raw = self.src.refill();
        self.raw_pos = 0;
        self.raw_left = self.raw.len() as i32;
    }

    /// Bytes consumed by the reader so far (offset into the decompressed stream).
    pub fn tell(&self) -> u32 {
        self.total - self.avail.max(0) as u32
    }

    pub fn at_eof(&self) -> bool {
        self.avail < 0
    }

    /// FUN_001179f0: decode the next 0x2000-byte LZ block into the decode buffer.
    fn lz_refill(&mut self) {
        // 0x001179f0
        let mut i: usize = 0;
        let mut left = self.raw_left;
        let mut p = self.raw_pos;
        let mut fl = self.lz_flags;
        if self.carry > 0 {
            // FUN_0023320c(buf, buf + 0x2000): move the overrun of the last ref to the front
            let c = self.carry as usize;
            self.lz_buf.copy_within(LZ_BLOCK..LZ_BLOCK + c, 0);
            i = c;
        }
        while i < LZ_BLOCK {
            fl = ((fl as i32) >> 1) as u32;
            left -= 1;
            if fl & 0x100 == 0 {
                if left < 0 {
                    self.refill_raw();
                    if self.raw_left < 1 {
                        break;
                    }
                    p = 0;
                    left = self.raw_left - 1;
                }
                let b = self.raw[p];
                p += 1;
                fl = b as u32 + 0xff00;
                left -= 1;
            }
            if left < 0 {
                self.refill_raw();
                if self.raw_left < 1 {
                    break;
                }
                p = 0;
                left = self.raw_left - 1;
            }
            let b = self.raw[p];
            p += 1;
            if fl & 1 == 0 {
                left -= 1;
                if left < 0 {
                    // Quirk kept: no end check here; the game reads whatever *ptr holds.
                    // With exact-size level streams this never triggers; we read 0.
                    self.refill_raw();
                    p = 0;
                    left = self.raw_left - 1;
                }
                let b1 = self.raw.get(p).copied().unwrap_or(0);
                p += 1;
                let v = b as usize + b1 as usize * 0x100;
                let mut s = v & 0x1fff;
                for _ in 0..(v >> 13) + 3 {
                    self.lz_buf[i] = self.lz_buf[s & 0x1fff];
                    s += 1;
                    i += 1;
                }
            } else {
                self.lz_buf[i] = b;
                i += 1;
            }
        }
        self.cur = 0;
        if i < LZ_BLOCK {
            self.avail = i as i32;
            self.carry = 0;
        } else {
            self.avail = LZ_BLOCK as i32;
            self.carry = (i - LZ_BLOCK) as i32;
        }
        self.lz_flags = fl;
        self.raw_pos = p;
        self.raw_left = left;
    }

    /// FUN_00117810: make output bytes available if the buffer is empty.
    fn fill(&mut self) {
        // 0x00117810
        if self.avail != 0 {
            return;
        }
        if self.flags & STREAM_LZ == 0 {
            self.refill_raw();
            self.avail = self.raw_left;
            self.cur = self.raw_pos;
            self.raw_pos += self.raw_left as usize;
            self.raw_left = 0;
        } else {
            self.lz_refill();
        }
        self.total += self.avail as u32;
    }

    /// FUN_00117898 (via FUN_00117968): read `n` bytes into `dst` (None = skip). Returns bytes read.
    pub fn read(&mut self, n: usize, mut dst: Option<&mut [u8]>) -> usize {
        // 0x00117898
        if self.avail < 0 {
            return 0;
        }
        let mut left = n;
        let mut o = 0;
        while left > 0 {
            self.fill();
            if self.avail == 0 {
                self.avail = -1;
                return n - left;
            }
            let k = left.min(self.avail as usize);
            if let Some(d) = dst.as_deref_mut() {
                let c = self.cur;
                d[o..o + k].copy_from_slice(&self.buf()[c..c + k]);
            }
            o += k;
            self.cur += k;
            self.avail -= k as i32;
            left -= k;
        }
        n
    }

    fn read_arr<const N: usize>(&mut self) -> [u8; N] {
        // The fast paths of the typed readers read straight from the buffer; the slow path reads
        // into a stack temp (uninitialised on EOF in the original; zero here).
        let mut a = [0u8; N];
        self.read(N, Some(&mut a));
        a
    }

    /// FUN_00117bd0 / FUN_00117d00: u8
    pub fn read_u8(&mut self) -> u8 {
        self.read_arr::<1>()[0]
    }
    /// FUN_00117c20: u16
    pub fn read_u16(&mut self) -> u16 {
        u16::from_le_bytes(self.read_arr())
    }
    /// FUN_00117d50: s16 (sign-extended)
    pub fn read_s16(&mut self) -> i16 {
        i16::from_le_bytes(self.read_arr())
    }
    /// FUN_00117db8 / FUN_00117c80 / FUN_00117e38: 32-bit LE
    pub fn read_s32(&mut self) -> i32 {
        i32::from_le_bytes(self.read_arr())
    }
    pub fn read_u32(&mut self) -> u32 {
        u32::from_le_bytes(self.read_arr())
    }
    /// FUN_00118458: 64-byte descriptor / name block
    pub fn read_64(&mut self) -> [u8; 64] {
        self.read_arr()
    }
    /// FUN_00117f70: 16-byte vector
    pub fn read_16(&mut self) -> [u8; 16] {
        self.read_arr()
    }
    /// FUN_00118380: u16 length + bytes (NUL appended by the game); 0 length -> None.
    pub fn read_string(&mut self) -> Option<Vec<u8>> {
        let n = self.read_u16() as usize;
        if n == 0 {
            return None;
        }
        let mut v = vec![0u8; n];
        self.read(n, Some(&mut v));
        Some(v)
    }
    /// FUN_00118420: skip a length-prefixed string.
    pub fn skip_string(&mut self) {
        let n = self.read_u16() as usize;
        self.read(n, None);
    }
    /// FUN_00117988: align the read pointer to `a` (power of two). The original aligns the
    /// memory address; all game buffers are >=64-aligned and refills are multiples of 16, so
    /// aligning the in-buffer cursor is identical.
    pub fn align(&mut self, a: usize) {
        let c = self.cur;
        let n = (c + a - 1) & !(a - 1);
        self.cur = n;
        self.avail -= (n - c) as i32;
    }

    /// Read everything left (tool/test helper, not a game function).
    pub fn read_to_end(&mut self) -> Vec<u8> {
        let mut out = Vec::new();
        while self.avail >= 0 {
            self.fill();
            if self.avail <= 0 {
                break;
            }
            let c = self.cur;
            let k = self.avail as usize;
            out.extend_from_slice(&self.buf()[c..c + k]);
            self.cur += k;
            self.avail = 0;
        }
        out
    }
}

/// Decompress a whole LZ stream held in memory (same decoder as the game).
pub fn lz_decompress(raw: &[u8]) -> Vec<u8> {
    let mut s = BinaryInputStream::new(Box::new(SliceSource::new(raw.to_vec(), 0)));
    s.set_lz();
    s.read_to_end()
}
