//! WP-Q4: class C2c9670 (world class 126, the episode screen label base): post-load (slot 14), the label
//! draw (slot 18: a VIF / GS packet per label item into the packet cursor DAT_002dea34, sorted by
//! FUN_001cd998 records {+0 0x15, +4 depth, +8 packet, +0xc}), the per-item packet builders.
//! The Rust episode screen (ui::episode) drives labels as events; these ports keep the EE packet
//! behaviour for the QMem callers. 1:1 against `QMem`; callees outside the class by address.
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem};
use crate::engine::game::q_001191e8::ee_sub_11a060;

/// DAT_002dea34: packet cursor
pub const PKT: u32 = 0x002d_ea34;
/// DAT_002dea28: second packet pointer (uncached bits toggled with the cursor)
pub const PKT2: u32 = 0x002d_ea28;
/// DAT_003e1078: shared label object (0x374)
pub const LABEL_OBJ: u32 = 0x003e_1078;
/// world
pub const WORLD: u32 = 0x003d_d490;

/// C2c9670 slot 5 (FUN_00114230): class descriptor 0x2bb510
pub fn ee_sub_114230() -> u32 {
    0x002b_b510
}

/// slot 14 (FUN_002123c8): parent FUN_001a5c48; DAT_003e1078 = object 0x374 once (when +0xbc);
/// +0xc8 = self, +0xa8 = -1, +0xa0 = 2048 - +0x98 * 0.4545 / 2, +0x9c = 2048 - ((+0x94 << 9) / 640) / 2;
/// +0x6d -> slot 48(-1); FUN_001a6ca8(o, 1); flags 0x102 on +0x40 (tagged inline or via pointer)
pub fn ee_sub_2123c8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001a_5c48, &[o], &[]);
    if m.r32(o + 0xbc) != 0 && m.r32(LABEL_OBJ) == 0 {
        let r = m.call(0x0015_e938, &[WORLD, 0x405, 0x374, o], &[]).0;
        m.w32(LABEL_OBJ, r);
    }
    let w = m.r32(o + 0x94) as i32;
    m.w32(o + 0xc8, o);
    m.w32(o + 0xa8, u32::MAX);
    let h = m.r32(o + 0x98) as i32 as f32;
    m.wf(o + 0xa0, 2048.0 - h * 0.454_545_47 * 0.5);
    let q = w.wrapping_shl(9) / 0x280;
    m.wf(o + 0x9c, 2048.0 - q as f32 * 0.5);
    if m.r8(o + 0x6d) != 0 {
        vslot(m, o, 48, &[u32::MAX], &[]);
    }
    m.call(0x001a_6ca8, &[o, 1], &[]);
    let p = m.r32(o + 0x40);
    if (p ^ 1) & 1 == 0 {
        m.w32(o + 0x40, p | 0x102);
    } else {
        let v = m.r32(p);
        m.w32(p, v | 0x102);
    }
}

fn put<M: QMem + ?Sized>(m: &mut M, v: u32) {
    let c = m.r32(PKT);
    m.w32(c, v);
    m.w32(PKT, c.wrapping_add(4));
}
fn pad<M: QMem + ?Sized>(m: &mut M, mask: u32, to: u32) {
    loop {
        let c = m.r32(PKT);
        if c & mask == to {
            break;
        }
        m.w32(c, 0);
        m.w32(PKT, c + 4);
    }
}
fn skip<M: QMem + ?Sized>(m: &mut M, n: u32) {
    let c = m.r32(PKT);
    m.w32(PKT, c.wrapping_add(n));
}
/// DMA CNT header {0x10000000, 0} at the cursor, cursor + 16; returns it
fn open<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let p0 = m.r32(PKT);
    m.w32(p0, 0x1000_0000);
    m.w32(p0 + 4, 0);
    m.w32(PKT, p0 + 16);
    p0
}
/// close: header qwc, trailing {0x60000000, 0} (RET), sort record {0x15, depth, packet, key}
fn close<M: QMem + ?Sized>(m: &mut M, p0: u32, depth: f32, key: u32) {
    let c = m.r32(PKT);
    let v = m.r32(p0) | ((c.wrapping_sub(p0) as i32 >> 4) as u32).wrapping_sub(1);
    m.w32(p0, v);
    m.w32(c, 0x6000_0000);
    m.w32(c + 4, 0);
    m.w32(PKT, c + 16);
    let r = m.call(0x001c_d998, &[], &[]).0;
    m.w32(r, 0x15);
    m.w32(r + 8, p0 & 0x0fff_ffff);
    m.w32(r + 0xc, key);
    m.wf(r + 4, depth);
}
fn layer<M: QMem + ?Sized>(m: &mut M, o: u32) -> f32 {
    (m.r32(o + 0xa4) << 0x12) as i32 as f32
}

/// FUN_002130c0(o): screen setup packet: FLUSH, DIRECT 3 (0x442210), DIRECT 0x25 (0x2df010), FLUSH;
/// depth = layer +0xa4 << 18
pub fn ee_sub_2130c0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let p0 = open(m);
    put(m, 0x1100_0000);
    pad(m, 0xf, 0xc);
    put(m, 0x5000_0003);
    let c = m.r32(PKT);
    m.call(0x001a_e528, &[c, 0x0044_2210, 3], &[]);
    skip(m, 0x30);
    pad(m, 0xf, 0xc);
    put(m, 0x5000_0025);
    let c = m.r32(PKT);
    m.call(0x001a_e528, &[c, 0x002d_f010, 0x25], &[]);
    skip(m, 0x250);
    put(m, 0x1100_0000);
    pad(m, 0xf, 0);
    let d = layer(m, o);
    close(m, p0, d, 0);
}

/// FUN_00217298(z, o, mode): mask pass packet: modes 1..4 add {0x13000000, STMOD-like 0x14000000 /
/// 0x14000006 / 0x14000020 x2, (0xc0 / 0xc0 / 0x60 / 0x90) >> 3 | 0x14000000, 0x13000000}; depth z + layer
pub fn ee_sub_217298<M: QMem + ?Sized>(m: &mut M, z: f32, o: u32, mode: i32) {
    let p0 = open(m);
    let sel = match mode {
        1 => Some((0x1400_0000u32, 0xc0u32)),
        2 => Some((0x1400_0006, 0xc0)),
        3 => Some((0x1400_0020, 0x60)),
        4 => Some((0x1400_0020, 0x90)),
        _ => None,
    };
    if let Some((a, b)) = sel {
        put(m, 0x1300_0000);
        put(m, a);
        put(m, b >> 3 | 0x1400_0000);
        put(m, 0x1300_0000);
    }
    pad(m, 0xf, 0);
    let d = z + layer(m, o);
    close(m, p0, d, 0);
}

/// shared head of FUN_002174c8 / 2177e0: the 0x40-byte xyz/uv block from pos (x * 0.8, y * 0.4545,
/// screen offsets +0x9c / +0xa0 of the label (*(item +8) +200) + 786432) then UNPACK 4 (0x6c04c000) of it
/// and UNPACK 3 (0x6c03c004) of rec
fn quad<M: QMem + ?Sized>(m: &mut M, item: u32, pos: u32) -> u32 {
    let b = stack_push(m, 0x40);
    for k in 0..16 {
        m.w32(b + 4 * k, 0);
    }
    let lb = m.r32(item + 8);
    let scr = m.r32(lb + 200);
    for (o, k, f) in [(0, 0, 0.8f32), (4, 4, 0.454_545_47), (0x10, 8, 0.8), (0x14, 0xc, 0.454_545_47)] {
        let v = m.rf(pos + k) * f;
        m.wf(b + o, v);
    }
    let x = m.rf(pos + 0x10) * 0.8 + m.rf(scr + 0x9c) + 786_432.0;
    m.wf(b + 0x20, x);
    let y = m.rf(pos + 0x14) * 0.454_545_47 + m.rf(scr + 0xa0) + 786_432.0;
    m.wf(b + 0x24, y);
    m.wf(b + 0x28, 1.0);
    m.wf(b + 0x3c, 1.0);
    b
}
fn quad_emit<M: QMem + ?Sized>(m: &mut M, b: u32, rec: u32) -> u32 {
    let p0 = open(m);
    put(m, 0x6c04_c000);
    for k in 0..16 {
        let v = m.r32(b + 4 * k);
        put(m, v);
    }
    put(m, 0x6c03_c004);
    for k in 0..12 {
        let v = m.r32(rec + 4 * k);
        put(m, v);
    }
    stack_pop(m, 0x40);
    p0
}

/// FUN_002174c8(item, pos, rec, fade): glyph quad: rec +4 = (item +0x12 >= 2), rec +0xc = 0x7fff, with
/// fade: a scale rec +0x2c < 1 (reset to 1.0) limits the count to max(3, min(+0x14, +0x14 * scale));
/// key item +0x1c
pub fn ee_sub_2174c8<M: QMem + ?Sized>(m: &mut M, item: u32, pos: u32, rec: u32, fade: u32) {
    let b = quad(m, item, pos);
    let big = (m.r16(item + 0x12) >= 2) as u32;
    m.w32(rec + 4, big);
    m.w32(rec + 0xc, 0x7fff);
    if fade != 0 {
        let s = m.rf(rec + 0x2c);
        m.wf(rec + 0x2c, 1.0);
        if s < 1.0 {
            let mut n = m.r32(item + 0x14) as i32;
            let v = (n as f32 * s) as i32;
            if v < n {
                n = v;
            }
            let u = if 2 < n { n as u32 } else { 3 };
            m.w32(rec + 0xc, if 0x7fff < u { 0x7fff } else { u });
        }
    }
    let p0 = quad_emit(m, b, rec);
    pad(m, 0xf, 0);
    let lb = m.r32(item + 8);
    let d = m.rf(rec + 8) + layer(m, lb);
    let k = m.r32(item + 0x1c);
    close(m, p0, d, k);
}

/// FUN_002177e0(item, pos, rec): image quad (kind 6: texture record +0x14 {+0x18 qwc, +0x1c data}, key
/// +0x1c; kind 7: inline +0x1c qwc / +0x20 data, key +0x18): quad + DIRECT of the GS data
pub fn ee_sub_2177e0<M: QMem + ?Sized>(m: &mut M, item: u32, pos: u32, rec: u32) {
    let kind = m.r32(item + 4);
    let d = match kind {
        6 => m.r32(item + 0x14),
        7 => m.r32(item + 0x1c),
        _ => return,
    };
    if d == 0 {
        return;
    }
    let b = quad(m, item, pos);
    m.w32(rec + 0xc, 0x7fff);
    m.w32(rec + 4, 0);
    let p0 = quad_emit(m, b, rec);
    let (n, src) = if kind == 6 {
        let t = m.r32(item + 0x14);
        (m.r32(t + 0x18), m.r32(t + 0x1c))
    } else {
        (m.r32(item + 0x1c), m.r32(item + 0x20))
    };
    pad(m, 0xf, 0xc);
    put(m, n | 0x5000_0000);
    let c = m.r32(PKT);
    m.call(0x001a_e528, &[c, src, n], &[]);
    skip(m, n.wrapping_mul(16));
    pad(m, 0xf, 0);
    let lb = m.r32(item + 8);
    let dz = m.rf(rec + 8) + layer(m, lb);
    let k = if kind == 6 { m.r32(item + 0x1c) } else { m.r32(item + 0x18) };
    close(m, p0, dz, k);
}

/// FUN_002159e0(item, pos, rec, fade): by item kind +4: 1 FUN_002174c8, 3 FUN_00216bc8, 6 / 7 FUN_002177e0
pub fn ee_sub_2159e0<M: QMem + ?Sized>(m: &mut M, item: u32, pos: u32, rec: u32, fade: u32) {
    match m.r32(item + 4) {
        1 => ee_sub_2174c8(m, item, pos, rec, fade),
        3 => {
            m.call(0x0021_6bc8, &[item, pos, rec, fade], &[]);
        }
        6 | 7 => ee_sub_2177e0(m, item, pos, rec),
        _ => {}
    }
}

/// FUN_002185d0(p): +0x28 -> FUN_002185f8(p, 0, 0)
pub fn ee_sub_2185d0<M: QMem + ?Sized>(m: &mut M, p: u32) {
    if m.r32(p + 0x28) != 0 {
        m.call(0x0021_85f8, &[p, 0, 0], &[]);
    }
}

/// rec {+0 1, +8 z, +0x10 rgba - 127, +0x20 clamp((b[4..8] - 127) / 127, -1, 1)} (colour from item n)
fn rec_init<M: QMem + ?Sized>(m: &mut M, r: u32, z: f32, col: Option<u32>) {
    for k in 0..12 {
        m.w32(r + 4 * k, 0);
    }
    m.w32(r, 1);
    m.wf(r + 8, z);
    if let Some(n) = col {
        for k in 0..4 {
            let v = m.r8(n + k) as f32 - 127.0;
            m.wf(r + 0x10 + 4 * k, v);
        }
        for k in 0..4 {
            let v = (m.r8(n + 4 + k) as i32 - 0x7f) as f32 * 0.007_874_016;
            m.wf(r + 0x20 + 4 * k, v.min(1.0).max(-1.0));
        }
    }
}

/// slot 18 (FUN_00213300): parent FUN_001a5ed8; when FUN_002132d0: FUN_002185d0(+0x84), cursor padded to
/// 64 and uncached, FUN_002130c0, then the item list +0xfc (next +0x68): kind 1 (mask start) / 4 (mask
/// end, draws the remembered item between mask passes 1..4), else coloured items at z (+1024 each);
/// the list is unlinked and emptied afterwards
pub fn ee_sub_213300<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001a_5ed8, &[o], &[]);
    if m.call(0x0021_32d0, &[o], &[]).0 == 0 {
        return;
    }
    let mut z = 16.0f32;
    let p = m.r32(o + 0x84);
    ee_sub_2185d0(m, p);
    pad(m, 0x3f, 0);
    let c = m.r32(PKT);
    m.w32(PKT, c | 0x3000_0000);
    let c = m.r32(PKT2);
    m.w32(PKT2, c | 0x3000_0000);
    ee_sub_2130c0(m, o);
    let s = stack_push(m, 0x60);
    let (ra, rb) = (s, s + 0x30);
    let mut prev = 0u32;
    let mut n = m.r32(o + 0xfc);
    while n != 0 {
        let kind = m.r32(n + 0x34);
        if kind == 1 {
            rec_init(m, ra, z, None);
            ee_sub_217298(m, z - 1.0, o, 1);
            let (it, fl) = (m.r32(n + 0x30), m.r8(n + 0x61) as u32);
            ee_sub_2159e0(m, it, n + 0x10, ra, fl);
            let zz = m.rf(ra + 8) + 1.0;
            ee_sub_217298(m, zz, o, 3);
            prev = n;
        } else if kind == 4 {
            rec_init(m, ra, z, None);
            ee_sub_217298(m, z - 1.0, o, 2);
            let (it, fl) = (m.r32(prev + 0x30), m.r8(prev + 0x61) as u32);
            ee_sub_2159e0(m, it, prev + 0x10, ra, fl);
            prev = 0;
            let zz = m.rf(ra + 8) + 1.0;
            ee_sub_217298(m, zz, o, 4);
            if m.r32(n + 0x30) != 0 {
                z += 1024.0;
                rec_init(m, rb, z, Some(n));
                let (it, fl) = (m.r32(n + 0x30), m.r8(n + 0x61) as u32);
                ee_sub_2159e0(m, it, n + 0x10, rb, fl);
            }
        } else if m.r32(n + 0x30) != 0 {
            rec_init(m, ra, z, Some(n));
            let (it, fl) = (m.r32(n + 0x30), m.r8(n + 0x61) as u32);
            ee_sub_2159e0(m, it, n + 0x10, ra, fl);
        }
        n = m.r32(n + 0x68);
        z += 1024.0;
    }
    stack_pop(m, 0x60);
    pad(m, 0x3f, 0);
    let c = m.r32(PKT);
    m.w32(PKT, c & 0x0fff_ffff);
    let c = m.r32(PKT2);
    m.w32(PKT2, c & 0x0fff_ffff);
    let mut it = m.r32(o + 0xfc);
    while it != 0 {
        let nx = m.r32(it + 0x68);
        m.w32(it + 0x68, 0);
        it = nx;
    }
    ee_sub_11a060(m, o + 0xfc);
}
