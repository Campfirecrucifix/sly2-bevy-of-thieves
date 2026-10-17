//! WP-Q5: run_001ee100 queue package: the C2d92a0 texture-animation playback state (FUN_001ee100
//! mode switch / FUN_001ee3b0 seek), the animated-texture dirty sweep FUN_001ee7b0, the ASCII ->
//! font glyph code mapping (FUN_001ee988 / FUN_001eea70), the guard-brain behaviour list helpers
//! (record {+4 count, +8 behaviours}: FUN_001eec70 pools, FUN_001eed10 slot 9 sweep, FUN_001eeee0
//! kind lookup), the C2cc1c0 behaviour enter / leave counters (FUN_001eefc0 / FUN_001ef038) and the
//! behaviour factory FUN_001ef138 (ids 0x9c..0xc6 -> size / vtable). 1:1 against `QMem`.
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem};
use crate::engine::classes::q::q5util::f4;

/// DAT_002dded0: frame clock
const CLOCK: u32 = 0x002d_ded0;

fn frand<M: QMem + ?Sized>(m: &mut M, lo: f32, hi: f32) -> f32 {
    m.call(0x0011_d448, &[], &[f4(lo), f4(hi)]).1[0]
}

/// FUN_001ee100: switch animation `o` to mode `k` (+0x64) and stamp +0x68 with the clock. Mode 0
/// (play): frame +0x74 = (count +0x1c - 1) * start fraction +0x78, rate +0x70 = 2 * count * (1 -
/// fraction) / +0x50, delay +0x6c = 0 when rand(0, 1) < +0x5c else rand(+0x54, +0x58); mode 2 (end):
/// frame = count - 1 (unchanged when already there). Same mode: nothing.
pub fn ee_sub_1ee100<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32) {
    if k == m.r32(o + 100) {
        return;
    }
    if k == 0 {
        let n = m.r32(o + 0x1c) as i32;
        let fr = m.rf(o + 0x78);
        let rate = ((n << 1) as f32 * (1.0 - fr)) / m.rf(o + 0x50);
        m.wf(o + 0x74, (n - 1) as f32 * fr);
        m.wf(o + 0x70, rate);
        let r = frand(m, 0.0, 1.0);
        let d = if r < m.rf(o + 0x5c) {
            0.0
        } else {
            let (a, b) = (m.rf(o + 0x54), m.rf(o + 0x58));
            frand(m, a, b)
        };
        m.wf(o + 0x6c, d);
    } else if k == 2 {
        let last = m.r32(o + 0x1c).wrapping_sub(1) as i32;
        if m.rf(o + 0x74) as i32 != last {
            m.wf(o + 0x74, last as f32);
        }
    }
    m.w32(o + 100, k);
    let c = m.r32(CLOCK);
    m.w32(o + 0x68, c);
}

/// FUN_001ee3b0: seek animation `o` to fraction `f` (+0x78): >= 1 ends it (mode 2), else restarts
/// it (mode -1 then 0); shows frame int(+0x74) (FUN_001eda68)
pub fn ee_sub_1ee3b0<M: QMem + ?Sized>(m: &mut M, f: f32, o: u32) {
    m.wf(o + 0x78, f);
    if 1.0 <= f {
        ee_sub_1ee100(m, o, 2);
    } else {
        m.w32(o + 100, 0xffff_ffff);
        ee_sub_1ee100(m, o, 0);
    }
    let fr = m.rf(o + 0x74) as i32 as u32;
    crate::engine::game::q_001ec688::ee_sub_1eda68(m, o, fr);
}

/// FUN_001ee678: +0x64 = b, +0x60 = a
/// VFIX: ABI per asm: o = a0, a = $f12 -> +0x60, b = $f13 -> +0x64 (swc1)
pub fn ee_sub_1ee678<M: QMem + ?Sized>(m: &mut M, o: u32, a: f32, b: f32) {
    m.wf(o + 100, b);
    m.wf(o + 0x60, a);
}

/// FUN_001ee7b0: for every animated texture in DAT_003e159c (next +0x40) marked dirty (+0x2c):
/// clear the mark and re-apply its records (+0x34 list, next +0x10) through FUN_001ec718
pub fn ee_sub_1ee7b0<M: QMem + ?Sized>(m: &mut M) {
    let mut p = m.r32(0x003e_159c);
    while p != 0 {
        if m.r32(p + 0x2c) != 0 {
            m.w32(p + 0x2c, 0);
            let mut r = m.r32(p + 0x34);
            while r != 0 {
                crate::engine::game::q_001ec688::ee_sub_1ec718(m, r);
                r = m.r32(r + 0x10);
            }
        }
        p = m.r32(p + 0x40);
    }
}

/// FUN_001ee988: font glyph code of ASCII `c`: digits / upper / lower case are offset through the
/// range table 0x2c3728 {base, first} (k = 0 / 1 / 2: code = base + c - first); the punctuation
/// runs 0x20..0x2f, 0x3a..0x40, 0x5b..0x60, 0x7b..0x7e read the 0x2c36a0 table (stride 4) at
/// c - 0x20, c - 0x2a, c - 0x44, c - 0x5e; others 0
pub fn ee_sub_1ee988<M: QMem + ?Sized>(m: &mut M, c: u32) -> u32 {
    let c = c & 0xff;
    let (gap, k) = if c.wrapping_sub(0x20) < 0x10 {
        (1, 0)
    } else if c.wrapping_sub(0x30) < 10 {
        (0, 0)
    } else if c.wrapping_sub(0x3a) < 7 {
        (0xb, 0)
    } else if c.wrapping_sub(0x41) < 0x1a {
        (0, 1)
    } else if c.wrapping_sub(0x5b) < 6 {
        (0x25, 0)
    } else if c.wrapping_sub(0x61) < 0x1a {
        (0, 2)
    } else if c.wrapping_sub(0x7b) <= 3 {
        (0x3f, 0)
    } else {
        return 0;
    };
    if gap == 0 {
        let base = m.r16(0x002c_3728 + k * 4) as u32;
        let first = m.r16(0x002c_372a + k * 4) as u32;
        return (base + c).wrapping_sub(first) & 0xffff;
    }
    m.r16(0x002c_36a0 + (c - (gap + 0x1f)) * 4) as u32
}

/// FUN_001eea70: C string `s` -> big-endian 16-bit glyph codes at `d`, 0-terminated
pub fn ee_sub_1eea70<M: QMem + ?Sized>(m: &mut M, s: u32, d: u32) {
    let n = m.call(0x0021_ed28, &[s], &[]).0 as i32;
    let mut i = 0u32;
    while (i as i32) < n {
        let c = m.r8(s + i) as u32;
        let g = ee_sub_1ee988(m, c);
        m.w16(d + 2 * i, ((g >> 8) & 0xff | (g & 0xff) << 8) as u16);
        i += 1;
    }
    m.w16(d + 2 * i, 0);
}

/// FUN_001eec68: *p = v
pub fn ee_sub_1eec68<M: QMem + ?Sized>(m: &mut M, p: u32, v: u32) {
    m.w32(p, v);
}

/// FUN_001eec70: eight behaviour pools at +8.. (FUN_001ae1d0({0xf0, 16}))
pub fn ee_sub_1eec70<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let sp = stack_push(m, 0x10);
    for i in 0..8 {
        m.w32(sp, ee_sub_1ef6f0());
        m.w32(sp + 4, 0x10);
        let p = m.call(0x001a_e1d0, &[sp], &[]).0;
        m.w32(o + 8 + 4 * i, p);
    }
    stack_pop(m, 0x10);
}

/// FUN_001eed10: slot 9 of every behaviour, last first
pub fn ee_sub_1eed10<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let mut i = m.r32(o + 4) as i32 - 1;
    while i >= 0 {
        let b = m.r32(o + 8 + 4 * i as u32);
        vslot(m, b, 9, &[], &[]);
        i -= 1;
    }
}

/// FUN_001eeed0: count - 1
pub fn ee_sub_1eeed0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let n = m.r32(o + 4).wrapping_sub(1);
    m.w32(o + 4, n);
}

/// FUN_001eeee0: first behaviour of kind `k` (FUN_00132b10), else 0
pub fn ee_sub_1eeee0<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32) -> u32 {
    let mut i = 0;
    while i < m.r32(o + 4) as i32 {
        let b = m.r32(o + 8 + 4 * i as u32);
        if m.call(0x0013_2b10, &[b, k], &[]).0 != 0 {
            return b;
        }
        i += 1;
    }
    0
}

/// C2cc1c0 slot 6 (FUN_001eef68): empty
pub fn ee_sub_1eef68() {}

/// enter / leave: +0x1c = on; a kind-0x38 owner (+0x10) gets FUN_0020b438 / FUN_0020b478; every
/// node of the +0x14 chain (next +0x14) counts +0x20 by `d`
fn active<M: QMem + ?Sized>(m: &mut M, o: u32, on: bool, f: u32, d: u32) {
    if (m.r32(o + 0x1c) != 0) == on {
        return;
    }
    m.w32(o + 0x1c, on as u32);
    let p = m.r32(o + 0x10);
    if m.call(0x0013_2b10, &[p, 0x38], &[]).0 != 0 {
        let p = m.r32(o + 0x10);
        m.call(f, &[p], &[]);
    }
    let mut n = m.r32(o + 0x14);
    while n != 0 {
        let v = m.r32(n + 0x20).wrapping_add(d);
        m.w32(n + 0x20, v);
        n = m.r32(n + 0x14);
    }
}

/// FUN_001eefc0: activate behaviour `o`
pub fn ee_sub_1eefc0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    active(m, o, true, 0x0020_b438, 1)
}

/// FUN_001ef038: deactivate behaviour `o`
pub fn ee_sub_1ef038<M: QMem + ?Sized>(m: &mut M, o: u32) {
    active(m, o, false, 0x0020_b478, 0xffff_ffff)
}

/// FUN_001ef0a8: +0x20 > 0
pub fn ee_sub_1ef0a8<M: QMem + ?Sized>(m: &mut M, o: u32) -> bool {
    (m.r32(o + 0x20) as i32) > 0
}

/// C2cc1c0 slot 13 (FUN_001ef0c0): b+0x1c = 0, b+0x18 = owner (+0x10) +0x1a8
pub fn ee_sub_1ef0c0<M: QMem + ?Sized>(m: &mut M, o: u32, b: u32) {
    m.w32(b + 0x1c, 0);
    let p = m.r32(o + 0x10);
    let v = m.r32(p + 0x1a8);
    m.w32(b + 0x18, v);
}

/// FUN_001ef0d8: release the behaviour at *p: slot 8, its owner's list (+0x26c) count - 1, *p = 0
pub fn ee_sub_1ef0d8<M: QMem + ?Sized>(m: &mut M, p: u32) {
    let b = m.r32(p);
    if b == 0 {
        return;
    }
    let own = m.r32(b + 0x10);
    vslot(m, b, 8, &[], &[]);
    ee_sub_1eeed0(m, own + 0x26c);
    m.w32(p, 0);
}

/// FUN_001ef138 size / vtable per behaviour id (0x9c..0xc6; 0x9c, 0xba, 0xbf, 0xc0 have extra init)
const BEHAVIOURS: [(u32, u32, u32); 40] = [
    (0x9c, 0xb0, 0x002d_04a0),
    (0x9d, 0x50, 0x002d_0ab0),
    (0x9e, 0x50, 0x002d_0a30),
    (0x9f, 0x60, 0x002d_07b0),
    (0xa1, 0xc0, 0x002d_0930),
    (0xa2, 0xc0, 0x002d_0830),
    (0xa3, 0xc0, 0x002d_08b0),
    (0xa4, 0x50, 0x002d_0620),
    (0xa5, 0x60, 0x002d_05a0),
    (0xa7, 0xf0, 0x002d_06a0),
    (0xa8, 0xe0, 0x002c_c4c0),
    (0xa9, 0x60, 0x002c_f7f8),
    (0xaa, 0x50, 0x002d_0520),
    (0xab, 0xe0, 0x002c_fa10),
    (0xac, 0xa0, 0x002d_0320),
    (0xad, 0x40, 0x002d_0220),
    (0xae, 0x40, 0x002d_4af8),
    (0xaf, 0x60, 0x002d_02a0),
    (0xb0, 0x40, 0x002d_0420),
    (0xb1, 0x70, 0x002d_03a0),
    (0xb2, 0x50, 0x002d_01a0),
    (0xb3, 0x40, 0x002d_0120),
    (0xb4, 0x40, 0x002c_ddc0),
    (0xb5, 0x60, 0x002c_dd40),
    (0xb6, 0x50, 0x002c_ce90),
    (0xb7, 0x70, 0x002d_00a0),
    (0xb8, 0x40, 0x002d_0020),
    (0xb9, 0x40, 0x002c_ffa0),
    (0xba, 0xd0, 0x002c_fe20),
    (0xbb, 0x30, 0x002c_ff20),
    (0xbd, 0x50, 0x002c_fd20),
    (0xbe, 0x50, 0x002c_fca0),
    (0xbf, 0x50, 0x002c_fc20),
    (0xc0, 0x60, 0x002c_fda0),
    (0xc1, 0x50, 0x002c_c440),
    (0xc2, 0xa0, 0x002c_c3c0),
    (0xc3, 0x40, 0x002c_c340),
    (0xc4, 0x40, 0x002c_c2c0),
    (0xc5, 0x40, 0x002c_c240),
    (0xc6, 0x40, 0x002c_c1c0),
];

/// FUN_001ef138: construct behaviour `id` in place at `p` (memset size, +0 / +4 = 0, vtable +0xc);
/// 0x9c also clears 0x70 bytes at +0x30, 0xba inits the three quads at +0x40 / +0x60 / +0x80 and
/// 0xbf the one at +0x30 (FUN_002562a8). Unknown ids (0xa0, 0xa6, 0xbc, out of range) return `dflt`.
pub fn ee_sub_1ef138<M: QMem + ?Sized>(m: &mut M, id: u32, p: u32, dflt: u32) -> u32 {
    let Some(&(_, size, vt)) = BEHAVIOURS.iter().find(|e| e.0 == id) else {
        return dflt;
    };
    m.call(0x0023_33b4, &[p, 0, size], &[]);
    m.w32(p, 0);
    m.w32(p + 4, 0);
    m.w32(p + 0xc, vt);
    match id {
        0x9c => {
            m.call(0x0023_33b4, &[p + 0x30, 0, 0x70], &[]);
        }
        0xba => {
            for o in [0x40, 0x60, 0x80] {
                m.call(0x0025_62a8, &[p + o], &[]);
            }
        }
        0xbf => {
            m.call(0x0025_62a8, &[p + 0x30], &[]);
        }
        _ => {}
    }
    p
}

/// FUN_001ef6f0: behaviour pool element size 0xf0
pub fn ee_sub_1ef6f0() -> u32 {
    0xf0
}
