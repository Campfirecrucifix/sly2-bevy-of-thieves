//! WP-J16: PSS movie decode (REPLACEMENT of the libmpeg/IPU decoder FUN_001b3ec0 / FUN_001b4058 /
//! FUN_0024b540 and the SPU audio stream FUN_001b3040 / FUN_001b32b8 of FUN_001b3bd0).
//!
//! Format (measured on all 25 movies of the disc, `tests/engine_j16.rs`): a PSS file is an MPEG-2 program
//! stream (pack 0xba, system header 0xbb, padding 0xbe, end 0xb9) with
//! - video: stream 0xe0, MPEG-2 640x480 (decoded by the `na_mpeg2_decoder` crate: the Rust decoder that
//!   replaces the IPU);
//! - audio: private stream 1 (0xbd); every payload starts with 4 bytes (0xff, sub-stream 0xa0+n, 0, 0),
//!   the concatenated rest is `"SShd" u32 0x18 {u32 kind (0x10 = SPU ADPCM), u32 rate, u32 channels,
//!   u32 interleave, u32 loop start, u32 loop end}` + `"SSbd" u32 size` + the ADPCM data, channel-interleaved
//!   in blocks of `interleave` bytes (0x20 on every movie: L/R correlation 0.84 at 0x20, < 0.15 for any other
//!   block size). The SPU plays it at `rate` (44100).
//! Pictures are 640x480 or 720x480 (rows 1, ..). The game shows the frame as a 640x448 sprite (FUN_001b3700: XYZ 0x7000,0x7900 .. 0x9000,0x8700), i.e.
//! the movie fills the 4:3 screen; `movie_rect` keeps that 4:3 picture letter/pillarboxed in any window.

use crate::engine::audio::adpcm::{decode_block, AdpcmState};

/// Bytes read from the disc per demux step (FUN_00132600 reads the stream in 0x80000-byte halves).
pub const READ_CHUNK: usize = 0x40000;
/// Picture aspect the game draws the movie at (the 640x448 4:3 screen).
pub const MOVIE_ASPECT: f32 = 4.0 / 3.0;

/// One program-stream packet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PsPacket {
    pub stream: u8,
    pub pts: Option<i64>,
    pub data: Vec<u8>,
}

/// Streaming MPEG program-stream demuxer (pack / system / PES).
#[derive(Default, Debug)]
pub struct PsDemux {
    buf: Vec<u8>,
    pos: usize,
    /// 0xb9 seen
    pub ended: bool,
}

fn pes_pts(h: &[u8]) -> i64 {
    (((h[0] as i64 >> 1) & 7) << 30) | ((h[1] as i64) << 22) | (((h[2] as i64) >> 1) << 15) | ((h[3] as i64) << 7) | (h[4] as i64 >> 1)
}

impl PsDemux {
    pub fn push(&mut self, data: &[u8]) {
        if self.pos > 0 {
            self.buf.drain(..self.pos);
            self.pos = 0;
        }
        self.buf.extend_from_slice(data);
    }

    /// Next complete packet (None: need more data or ended).
    pub fn next_packet(&mut self) -> Option<PsPacket> {
        loop {
            if self.ended {
                return None;
            }
            let b = &self.buf[self.pos..];
            if b.len() < 6 {
                return None;
            }
            if b[0] != 0 || b[1] != 0 || b[2] != 1 {
                // resync on the next start code
                match b.windows(3).skip(1).position(|w| w == [0, 0, 1]) {
                    Some(i) => self.pos += i + 1,
                    None => self.pos += b.len() - 2,
                }
                continue;
            }
            let code = b[3];
            match code {
                0xb9 => {
                    self.ended = true;
                    return None;
                }
                0xba => {
                    // MPEG-2 pack header: 14 bytes + stuffing; MPEG-1: 12 bytes
                    let n = if b[4] & 0xc0 == 0x40 {
                        if b.len() < 14 {
                            return None;
                        }
                        14 + (b[13] & 7) as usize
                    } else {
                        12
                    };
                    if b.len() < n {
                        return None;
                    }
                    self.pos += n;
                }
                _ => {
                    let len = u16::from_be_bytes([b[4], b[5]]) as usize;
                    if b.len() < 6 + len {
                        return None;
                    }
                    let p = &b[6..6 + len];
                    self.pos += 6 + len;
                    if !(code == 0xbd || (0xc0..=0xef).contains(&code)) || p.len() < 3 {
                        continue;
                    }
                    // MPEG-2 PES header
                    let hl = p[2] as usize;
                    if p[0] & 0xc0 != 0x80 || p.len() < 3 + hl {
                        continue;
                    }
                    let pts = (p[1] & 0x80 != 0 && hl >= 5).then(|| pes_pts(&p[3..8]));
                    return Some(PsPacket { stream: code, pts, data: p[3 + hl..].to_vec() });
                }
            }
        }
    }
}

/// "SShd" header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SShd {
    pub kind: u32,
    pub rate: u32,
    pub channels: u32,
    pub interleave: u32,
}

/// Private-stream-1 audio: header parse + ADPCM decode into interleaved i16.
#[derive(Default, Debug)]
pub struct PssAudio {
    pub head: Option<SShd>,
    /// "SSbd" size (bytes of ADPCM data)
    pub body_size: u32,
    hdr: Vec<u8>,
    data: Vec<u8>,
    st: Vec<AdpcmState>,
    /// ADPCM bytes consumed
    pub consumed: u64,
}

fn le(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]])
}

impl PssAudio {
    /// One 0xbd payload; decoded samples (interleaved by channel) are appended to `out`.
    pub fn push(&mut self, payload: &[u8], out: &mut Vec<i16>) {
        let Some(p) = payload.get(4..) else { return };
        if self.head.is_none() {
            self.hdr.extend_from_slice(p);
            if self.hdr.len() < 40 {
                return;
            }
            let h = std::mem::take(&mut self.hdr);
            if &h[0..4] != b"SShd" || &h[32..36] != b"SSbd" {
                // not a PSS audio stream: ignored
                self.head = Some(SShd { kind: 0, rate: 0, channels: 0, interleave: 0 });
                return;
            }
            let s = SShd { kind: le(&h, 8), rate: le(&h, 12), channels: le(&h, 16), interleave: le(&h, 20) };
            self.head = Some(s);
            self.body_size = le(&h, 36);
            self.st = vec![AdpcmState::default(); s.channels as usize];
            self.data.extend_from_slice(&h[40..]);
        } else {
            self.data.extend_from_slice(p);
        }
        self.decode(out);
    }

    fn decode(&mut self, out: &mut Vec<i16>) {
        let Some(h) = self.head else { return };
        let (ch, il) = (h.channels as usize, h.interleave as usize);
        if h.kind != 0x10 || ch == 0 || il == 0 || il % 16 != 0 {
            self.data.clear();
            return;
        }
        let row = ch * il;
        let mut o = 0;
        let mut tmp = vec![Vec::new(); ch];
        while self.data.len() - o >= row {
            for (c, t) in tmp.iter_mut().enumerate() {
                t.clear();
                for f in (0..il).step_by(16) {
                    let s = o + c * il + f;
                    decode_block(&self.data[s..s + 16], &mut self.st[c], t);
                }
            }
            for i in 0..tmp[0].len() {
                for t in &tmp {
                    out.push(t.get(i).copied().unwrap_or(0));
                }
            }
            o += row;
        }
        self.consumed += o as u64;
        self.data.drain(..o);
    }
}

/// One decoded RGBA video frame.
#[derive(Clone, Debug)]
pub struct VideoFrame {
    pub index: u32,
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// MPEG-1/2 frame rate by frame_rate_code.
pub fn frame_rate(code: u8) -> f32 {
    match code {
        1 => 24000.0 / 1001.0,
        2 => 24.0,
        3 => 25.0,
        4 => 30000.0 / 1001.0,
        5 => 30.0,
        6 => 50.0,
        7 => 60000.0 / 1001.0,
        8 => 60.0,
        _ => 30000.0 / 1001.0,
    }
}

/// Demux + decode of a whole PSS byte stream, fed in chunks.
pub struct PssDecoder {
    pub demux: PsDemux,
    video: na_mpeg2_decoder::Decoder,
    pub audio: PssAudio,
    /// frames decoded so far
    pub frames: u32,
    /// sequence header frame rate (frames/s)
    pub fps: Option<f32>,
    /// video decode errors (frames lost)
    pub errors: u32,
    /// decoder restarts after a decoder panic
    pub restarts: u32,
    /// sequence header (+ extensions) unit, re-fed after a restart
    seq: Vec<u8>,
}

impl Default for PssDecoder {
    fn default() -> Self {
        Self { demux: PsDemux::default(), video: na_mpeg2_decoder::Decoder::new(), audio: PssAudio::default(), frames: 0, fps: None, errors: 0, restarts: 0, seq: Vec::new() }
    }
}

impl PssDecoder {
    /// Feed file bytes; decoded frames go to `on_frame`, audio samples to `pcm`. `rgba` = false skips the
    /// colour conversion (frames still counted).
    pub fn feed(&mut self, data: &[u8], rgba: bool, pcm: &mut Vec<i16>, on_frame: &mut dyn FnMut(VideoFrame)) {
        self.demux.push(data);
        while let Some(p) = self.demux.next_packet() {
            match p.stream {
                0xe0 => {
                    if self.fps.is_none() {
                        if let Some(i) = p.data.windows(4).position(|w| w == [0, 0, 1, 0xb3]) {
                            if let Some(&b) = p.data.get(i + 7) {
                                self.fps = Some(frame_rate(b & 0xf));
                            }
                            // sequence header + extensions up to the GOP / first picture
                            let e = p.data[i + 4..].windows(4).position(|w| w[..3] == [0, 0, 1] && (w[3] == 0xb8 || w[3] == 0)).map_or(p.data.len(), |e| i + 4 + e);
                            self.seq = p.data[i..e].to_vec();
                        }
                    }
                    // the decoder crate can panic on some motion vectors near the picture edge
                    // (videodsp::emulated_edge_mc index): the picture is dropped and the decoder restarts at
                    // the next sequence header, like the IPU's error concealment would skip a bad picture.
                    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.video.decode_shared(&p.data, p.pts)));
                    match r {
                        Ok(Ok(fs)) => {
                            for f in fs {
                                self.emit(&f, rgba, on_frame);
                            }
                        }
                        Ok(Err(_)) => self.errors += 1,
                        Err(_) => {
                            self.errors += 1;
                            self.restarts += 1;
                            self.video = na_mpeg2_decoder::Decoder::new();
                            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.video.decode_shared(&self.seq, None)));
                        }
                    }
                }
                0xbd => self.audio.push(&p.data, pcm),
                _ => {}
            }
        }
    }

    /// End of stream: the last reference frame.
    pub fn finish(&mut self, rgba: bool, on_frame: &mut dyn FnMut(VideoFrame)) {
        if let Ok(fs) = self.video.flush_shared() {
            for f in fs {
                self.emit(&f, rgba, on_frame);
            }
        }
    }

    fn emit(&mut self, f: &na_mpeg2_decoder::Frame, rgba: bool, on_frame: &mut dyn FnMut(VideoFrame)) {
        let mut buf = Vec::new();
        if rgba {
            buf = vec![0; f.width * f.height * 4];
            na_mpeg2_decoder::frame_to_rgba_bt601_limited(f, &mut buf);
        }
        on_frame(VideoFrame { index: self.frames, width: f.width as u32, height: f.height as u32, rgba: buf });
        self.frames += 1;
    }
}

/// The 4:3 movie picture fitted into a `w` x `h` window: pillarbox when wider (16:9, 21:9, 32:9),
/// letterbox when taller. Returns (width, height) of the picture, centred.
pub fn movie_rect(w: f32, h: f32) -> (f32, f32) {
    if w <= 0.0 || h <= 0.0 {
        return (0.0, 0.0);
    }
    if w / h > MOVIE_ASPECT {
        (h * MOVIE_ASPECT, h)
    } else {
        (w, w / MOVIE_ASPECT)
    }
}
