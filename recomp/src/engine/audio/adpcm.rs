//! PS2 SPU2 ADPCM ("VAG") decoder. Replaces the SPU2 hardware voice decoder (not game code).
//! Block = 16 bytes: b0 = predictor<<4 | shift, b1 = flags, 14 bytes = 28 nibbles (low nibble first).
//! Flags: bit0 = loop end (jump to loop start / stop), bit1 = loop repeat, bit2 = loop start.
//! flag 7 (all set) = silent terminator block used by 989snd/VAG tools.

pub const COEF: [(i32, i32); 5] = [(0, 0), (60, 0), (115, -52), (98, -55), (122, -60)];

pub const FLAG_END: u8 = 1;
pub const FLAG_REPEAT: u8 = 2;
pub const FLAG_START: u8 = 4;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AdpcmState {
    pub s1: i32,
    pub s2: i32,
}

/// Decode one 16-byte block into 28 samples (SPU2 behaviour: shift >12 treated as 9 like the hardware's
/// effective clamp; predictor >4 uses 0).
pub fn decode_block(b: &[u8], st: &mut AdpcmState, out: &mut Vec<i16>) {
    let mut shift = (b[0] & 0x0f) as i32;
    if shift > 12 {
        shift = 9;
    }
    let p = (b[0] >> 4) as usize;
    let (f0, f1) = if p < 5 { COEF[p] } else { (0, 0) };
    for i in 0..28 {
        let byte = b[2 + i / 2];
        let nib = if i & 1 == 0 { byte & 0x0f } else { byte >> 4 };
        let s = ((nib as i16) << 12) as i32 >> shift; // sign-extend nibble
        let v = (s + ((st.s1 * f0 + st.s2 * f1 + 32) >> 6)).clamp(-32768, 32767);
        st.s2 = st.s1;
        st.s1 = v;
        out.push(v as i16);
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Decoded {
    pub pcm: Vec<i16>,
    /// sample index of the loop start (block with FLAG_START) when the sample loops
    pub loop_start: Option<usize>,
    /// true when the end block had FLAG_REPEAT (voice jumps to loop start instead of stopping)
    pub looped: bool,
}

/// Decode a mono ADPCM body until the first end flag (inclusive) or the end of the data.
/// Leading all-zero block (VAG convention) decodes to silence like on hardware.
pub fn decode(data: &[u8]) -> Decoded {
    let mut st = AdpcmState::default();
    let mut d = Decoded::default();
    for b in data.chunks_exact(16) {
        let fl = b[1];
        if fl == 7 {
            break;
        }
        if fl & FLAG_START != 0 && d.loop_start.is_none() {
            d.loop_start = Some(d.pcm.len());
        }
        decode_block(b, &mut st, &mut d.pcm);
        if fl & FLAG_END != 0 {
            d.looped = fl & FLAG_REPEAT != 0;
            break;
        }
    }
    if !d.looped {
        d.loop_start = None;
    }
    d
}

/// Decode a raw stream body ignoring end flags (streams set them per buffer half).
pub fn decode_stream(data: &[u8], st: &mut AdpcmState, out: &mut Vec<i16>) {
    for b in data.chunks_exact(16) {
        decode_block(b, st, out);
    }
}
