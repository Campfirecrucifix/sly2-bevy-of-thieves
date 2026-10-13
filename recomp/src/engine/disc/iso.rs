//! ISO9660 reader over the disc image. Replaces the cdvd SDK (sceCdRead / sceCdSearchFile):
//! raw 2048-byte sector reads by LSN and file lookup by path (SYSTEM.CNF, SCUS_973.16, IRX ...).

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::Mutex;

pub const SECTOR: u64 = 2048;

/// Anything that can serve 2048-byte sectors.
pub trait SectorSource: Send + Sync {
    fn read_sectors(&self, lsn: u32, count: u32) -> io::Result<Vec<u8>>;
    /// Read `len` bytes starting at sector `lsn` (short read at end of image = truncated Vec).
    fn read_bytes(&self, lsn: u32, len: usize) -> io::Result<Vec<u8>> {
        let mut v = self.read_sectors(lsn, len.div_ceil(SECTOR as usize) as u32)?;
        v.truncate(len);
        Ok(v)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IsoEntry {
    pub name: String,
    pub lsn: u32,
    pub size: u32,
    pub is_dir: bool,
}

pub struct IsoImage {
    file: Mutex<File>,
    pub sectors: u64,
    root: IsoEntry,
}

impl SectorSource for IsoImage {
    fn read_sectors(&self, lsn: u32, count: u32) -> io::Result<Vec<u8>> {
        let mut f = self.file.lock().unwrap();
        f.seek(SeekFrom::Start(lsn as u64 * SECTOR))?;
        let mut v = Vec::with_capacity((count as u64 * SECTOR) as usize);
        (&mut *f).take(count as u64 * SECTOR).read_to_end(&mut v)?;
        Ok(v)
    }
}

fn parse_record(r: &[u8]) -> IsoEntry {
    let lsn = u32::from_le_bytes(r[2..6].try_into().unwrap());
    let size = u32::from_le_bytes(r[10..14].try_into().unwrap());
    let nl = r[32] as usize;
    let raw = &r[33..33 + nl];
    let name = match raw {
        [0] => ".".to_string(),
        [1] => "..".to_string(),
        _ => {
            let s = String::from_utf8_lossy(raw);
            s.split(';').next().unwrap_or("").trim_end_matches('.').to_string()
        }
    };
    IsoEntry { name, lsn, size, is_dir: r[25] & 2 != 0 }
}

impl IsoImage {
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let file = File::open(path)?;
        let sectors = file.metadata()?.len() / SECTOR;
        let mut img = Self { file: Mutex::new(file), sectors, root: IsoEntry { name: String::new(), lsn: 0, size: 0, is_dir: true } };
        let pvd = img.read_sectors(16, 1)?;
        if pvd.len() < 2048 || pvd[0] != 1 || &pvd[1..6] != b"CD001" {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "no ISO9660 primary volume descriptor"));
        }
        img.root = parse_record(&pvd[156..190]);
        Ok(img)
    }

    /// Directory listing (skips "." and "..").
    pub fn read_dir(&self, dir: &IsoEntry) -> io::Result<Vec<IsoEntry>> {
        let data = self.read_bytes(dir.lsn, dir.size as usize)?;
        let mut out = Vec::new();
        let mut o = 0;
        while o < data.len() {
            let len = data[o] as usize;
            if len == 0 {
                // records never straddle sectors: skip to the next sector
                o = (o / SECTOR as usize + 1) * SECTOR as usize;
                continue;
            }
            let e = parse_record(&data[o..o + len]);
            if e.name != "." && e.name != ".." {
                out.push(e);
            }
            o += len;
        }
        Ok(out)
    }

    /// Case-insensitive lookup; accepts "cdrom0:\\FOO\\BAR.IRX;1", "/FOO/BAR.IRX" or "FOO/BAR.IRX".
    pub fn find(&self, path: &str) -> io::Result<Option<IsoEntry>> {
        let p = path.strip_prefix("cdrom0:").unwrap_or(path);
        let p = p.split(';').next().unwrap_or("");
        let mut cur = self.root.clone();
        for part in p.split(['/', '\\']).filter(|s| !s.is_empty()) {
            if !cur.is_dir {
                return Ok(None);
            }
            match self.read_dir(&cur)?.into_iter().find(|e| e.name.eq_ignore_ascii_case(part)) {
                Some(e) => cur = e,
                None => return Ok(None),
            }
        }
        Ok(Some(cur))
    }

    pub fn read_file(&self, path: &str) -> io::Result<Vec<u8>> {
        let e = self.find(path)?.ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, path.to_string()))?;
        self.read_bytes(e.lsn, e.size as usize)
    }
}

/// Raw source over disc sectors, bounded to `size` bytes, delivered in the CD stream's chunk size.
/// Replaces FUN_001849a0 (async sceCdRead double-buffering): chunk = (buffer / nbuf) & ~0x7fff for
/// buffers > 0x8000 (FUN_00184d98 nbuf = max(2, size >> 20)); the level stream buffer is 0xd8000.
pub struct DiscSource<S: SectorSource> {
    disc: std::sync::Arc<S>,
    lsn: u32,
    left: u32,
    chunk: u32,
}

/// Chunk size of the level stream (FUN_001848a8(.., 0xd8000, 0x2f7980, 0)).
pub fn cd_stream_chunk(buffer: u32) -> u32 {
    let nbuf = (buffer >> 20).max(2);
    let c = buffer / nbuf;
    if c > 0x8000 { c & 0xffff8000 } else { c & 0xfffff800 }
}

impl<S: SectorSource> DiscSource<S> {
    pub fn new(disc: std::sync::Arc<S>, lsn: u32, size: u32) -> Self {
        Self { disc, lsn, left: size, chunk: cd_stream_chunk(0xd8000) }
    }
}

impl<S: SectorSource> crate::engine::disc::lz::RawSource for DiscSource<S> {
    fn refill(&mut self) -> Vec<u8> {
        let n = self.left.min(self.chunk);
        if n == 0 {
            return Vec::new();
        }
        let v = self.disc.read_bytes(self.lsn, n as usize).unwrap_or_default();
        self.lsn += n.div_ceil(SECTOR as u32);
        self.left = if v.len() as u32 == n { self.left - n } else { 0 };
        v
    }
}
