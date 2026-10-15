//! WP-Q5: run_0014b468 queue package: the TU static initialisers (shared pool registration bodies, see
//! game::q_0021c6c8), the ISO-9660 path normaliser + "cdrom0:" path builder, the memory-card
//! presence / settings-load state machine (DAT_002bd4c0, CRC-32 over the 0x800-byte buffer 0x2dd680),
//! the timed callback-list dispatcher (FUN_0014bff8) and a few event thunks. 1:1 against `QMem`
//! (out-of-package callees by address; the libmc calls 0x241e88 / 0x242840 / 0x242ab8 / 0x242ae0 /
//! 0x242c30 stay routed, the Rust host answers them).
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem, V4};
use crate::engine::game::q_0021c6c8::{ee_sub_21ed28 as strlen, ee_sub_21eda0 as strcat, ee_sub_21ee30 as strupr};

/// FUN_00205458 (component lookup), FUN_001fbb90 (emitter_create)
const COMP: u32 = 0x0020_5458;
const EMITTER: u32 = 0x001f_bb90;

/// FUN_0014b468 (RECORDED_J11): message to component 0x9c: kind 0xe with `p` (+0x10), or with p == 0
/// kind 1 when component 0xb6 exists (FUN_00176750 on a zeroed 0x70-byte record)
pub fn ee_sub_14b468<M: QMem + ?Sized>(m: &mut M, s: u32, p: u32) {
    let sp = stack_push(m, 0x70);
    m.call(0x0023_33b4, &[sp, 0, 0x70], &[]);
    let c = m.call(COMP, &[s, 0x9c], &[]).0;
    if p != 0 {
        m.w32(sp + 0x10, p);
        m.w32(sp, 0xe);
        m.call(0x0017_6750, &[c, sp], &[]);
    } else if m.call(COMP, &[s, 0xb6], &[]).0 != 0 {
        m.w32(sp, 1);
        m.call(0x0017_6750, &[c, sp], &[]);
    }
    stack_pop(m, 0x70);
}

/// FUN_0014b670: +4 when the tag (+8) is 2, else 0
pub fn ee_sub_14b670<M: QMem + ?Sized>(m: &mut M, p: u32) -> u32 {
    if m.r32(p + 8) == 2 {
        m.r32(p + 4)
    } else {
        0
    }
}

/// FUN_0014b690: "fid: unknown"
pub fn ee_sub_14b690() -> u32 {
    0x002a_4ef0
}

/// FUN_0014b7f8: normalise `p` in place to ISO-9660 components (upper case, separator appended, each
/// component cut to 8.3 with other characters as '_', a trailing ";1" kept), NUL-separated; returns the
/// component count
pub fn ee_sub_14b7f8<M: QMem + ?Sized>(m: &mut M, p: u32) -> u32 {
    strupr(m, p);
    let mut n = strlen(m, p);
    if n != 0 {
        let c = m.r8(p + n - 1);
        if c != b'\\' && c != b'/' {
            strcat(m, p, 0x002a_4f00);
            n = strlen(m, p);
        }
    }
    let end = p + n;
    let (mut cnt, mut len, mut max) = (0u32, 0u32, 8u32);
    let (mut src, mut out) = (p, p);
    while src < end {
        let c = m.r8(src);
        if c == b'/' || c == b'\\' {
            if len != 0 {
                m.w8(out, 0);
                out += 1;
                len = 0;
                max = 8;
                cnt += 1;
            }
            src += 1;
            continue;
        }
        if c == b'.' {
            if len != 0 && max == 8 {
                m.w8(out, b'.');
                out += 1;
                len = 0;
                max = 3;
            }
            src += 1;
            continue;
        }
        if c == b';' && len != 0 && src == end - 2 && m.r8(src + 1) == b'1' {
            let (a, b) = (m.r8(end - 2), m.r8(end - 1));
            m.w8(out, a);
            m.w8(out + 1, b);
            out += 2;
            len += 2;
            src = end;
            continue;
        }
        if len < max {
            let o = if c.is_ascii_uppercase() || c.is_ascii_digit() { c } else { b'_' };
            m.w8(out, o);
            out += 1;
            len += 1;
        }
        src += 1;
    }
    if len != 0 {
        m.w8(out, 0);
        cnt += 1;
    }
    cnt
}

/// FUN_0014b9c0: refresh the card type DAT_002bd4c4 (FUN_00242ab8); true when it is 0x14 (PS2 card)
pub fn ee_sub_14b9c0<M: QMem + ?Sized>(m: &mut M) -> bool {
    let t = m.call(0x0024_2ab8, &[], &[]).0;
    m.w32(0x002b_d4c4, t);
    t == 0x14
}

/// FUN_0014ba00: PS2 card present -> FUN_00242c30(2)
pub fn ee_sub_14ba00<M: QMem + ?Sized>(m: &mut M) {
    if m.r32(0x002b_d4c4) == 0x14 {
        m.call(0x0024_2c30, &[2], &[]);
    }
}

/// CRC-32 (table DAT_002a2e68, init !0, no final xor) over the 0x800 bytes at 0x2dd680
pub fn crc_2dd680<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let mut c = 0xffff_ffffu32;
    for i in 0..0x800u32 {
        let b = m.r8(0x002d_d680 + i) as u32;
        c = m.r32(0x002a_2e68 + ((c ^ b) & 0xff) * 4) ^ (c >> 8);
    }
    c
}

/// FUN_0014ba38: card settings state machine DAT_002bd4c0: 1 = idle (when 0x3da4f0 allows,
/// FUN_001dd780, and a PS2 card is in, start the read of DAT_002a2e5c + 0xca0 -> 2), 2 = reading (on
/// completion: CRC matches DAT_002a2e60 -> apply FUN_001dd728, 3; else back to 1), 3 = loaded (card
/// pulled while DAT_003dd0e4 -> FUN_001dd6c8, 1)
pub fn ee_sub_14ba38<M: QMem + ?Sized>(m: &mut M) {
    const ST: u32 = 0x002b_d4c0;
    let st = m.r32(ST);
    let mut ns = st;
    if st == 2 {
        let done = ee_sub_14bd30(m, 0);
        ns = m.r32(ST);
        if done {
            ns = 1;
            if crc_2dd680(m) == m.r32(0x002a_2e60) {
                m.call(0x001d_d728, &[0x003d_a4f0], &[]);
                ns = 3;
            }
        }
    } else if (st as i32) < 3 {
        if st == 1 {
            let ok = m.call(0x001d_d780, &[0x003d_a4f0], &[]).0 != 0;
            ns = m.r32(ST);
            if ok {
                let f = m.r32(0x002a_2e5c).wrapping_add(0xca0);
                let card = ee_sub_14b9c0(m);
                ns = m.r32(ST);
                if card {
                    let r = m.call(0x0024_2840, &[f, 1, 0x002d_d680, 0x002a_2e50], &[]).0;
                    ns = if r == 1 { 2 } else { m.r32(ST) };
                }
            }
        }
    } else if st == 3 && m.r32(0x003d_d0e4) != 0 {
        let card = ee_sub_14b9c0(m);
        ns = m.r32(ST);
        if !card {
            m.call(0x001d_d6c8, &[0x003d_a4f0], &[]);
            ns = 1;
        }
    }
    m.w32(ST, ns);
}

/// FUN_0014bbb0: build the device path of `name` into `out`: "\" + name normalised (FUN_0014b7f8,
/// components re-joined with '\') formatted with "cdrom0:" (cd != 0) or the host prefix 0x2a4fe8
pub fn ee_sub_14bbb0<M: QMem + ?Sized>(m: &mut M, out: u32, name: u32, cd: u32) {
    let sp = stack_push(m, 0x100);
    let h = m.r16(0x002a_4fd0);
    m.w16(sp, h);
    strcat(m, sp, name);
    let n = ee_sub_14b7f8(m, sp) as i32;
    for _ in 1..n {
        let l = strlen(m, sp);
        m.w8(sp + l, b'\\');
    }
    let pre = if cd == 0 { 0x002a_4fe8 } else { 0x002a_4fe0 };
    m.call(0x0021_ecd8, &[out, 0x002a_4fd8, pre, sp], &[]);
    stack_pop(m, 0x100);
}

/// FUN_0014bc78: blocking card call FUN_00242840(a, b, c, mode 1 ? 0x2a2e50 : 0x2a2e58) retried until it
/// returns 1 (DAT_002dde80), syncing (FUN_00241e88(0)) and latching FUN_00242ae0 in DAT_002dde84
pub fn ee_sub_14bc78<M: QMem + ?Sized>(m: &mut M, a: u32, b: u32, c: u32, mode: u32) {
    let d = if mode == 1 { 0x002a_2e50 } else { 0x002a_2e58 };
    loop {
        let r = m.call(0x0024_2840, &[a, b, c, d], &[]).0;
        m.w32(0x002d_de80, r);
        if r == 1 {
            break;
        }
        m.call(0x0024_1e88, &[0], &[]);
        let e = m.call(0x0024_2ae0, &[], &[]).0;
        m.w32(0x002d_de84, e);
    }
}

/// FUN_0014bd30: card sync FUN_00241e88(nowait = p == 0) == 0 (done)
pub fn ee_sub_14bd30<M: QMem + ?Sized>(m: &mut M, p: u32) -> bool {
    m.call(0x0024_1e88, &[(p == 0) as u32], &[]).0 == 0
}

/// FUN_0014bf18: clear +0x10 of every node of list `l` (head *l, next +0x18)
pub fn ee_sub_14bf18<M: QMem + ?Sized>(m: &mut M, l: u32) {
    let mut n = m.r32(l);
    while n != 0 {
        m.w32(n + 0x10, 0);
        n = m.r32(n + 0x18);
    }
}

/// FUN_0014bff8: timed dispatch of list `l` {head, ?, link offset +8, due time +0xc}: once due
/// (0 < due <= DAT_002ddf0c) push the iteration frame on DAT_002dd2f8, flush nodes with +0x14 set
/// (FUN_0011a118), pick the highest-priority (+4) node whose count (+0x10) reached it, fire emitter 0x34
/// and its callback (+8)(+0xc), clear the counts and the due time
pub fn ee_sub_14bff8<M: QMem + ?Sized>(m: &mut M, l: u32) {
    let due = m.rf(l + 0xc);
    if due == 0.0 || !(due <= m.rf(0x002d_df0c)) {
        return;
    }
    let sp = stack_push(m, 0x10);
    let off = m.r32(l + 8);
    let prev = m.r32(0x002d_d2f8);
    m.w32(0x002d_d2f8, sp);
    m.w32(sp + 0xc, prev);
    let mut n = m.r32(l);
    m.w32(sp, l);
    m.w32(sp + 4, n.wrapping_add(off));
    m.w32(sp + 8, off);
    let mut best = 0u32;
    while n != 0 {
        if m.r32(n + 0x14) != 0 {
            m.call(0x0011_a118, &[l, n], &[]);
            m.w32(n + 0x14, 0);
            m.w32(n + 0x10, 0);
        } else {
            let pri = m.r32(n + 4) as i32;
            if pri <= m.r32(n + 0x10) as i32 && (best == 0 || (m.r32(best + 4) as i32) < pri) {
                best = n;
            }
        }
        let np = m.r32(sp + 4);
        n = m.r32(np);
        let o = m.r32(sp + 8);
        m.w32(sp + 4, n.wrapping_add(o));
    }
    if best != 0 {
        let e = m.call(EMITTER, &[0x34], &[]).0;
        m.call(0x001f_b2f8, &[e, 0], &[]);
        let (f, a) = (m.r32(best + 8), m.r32(best + 0xc));
        m.call(f, &[a], &[]);
    }
    ee_sub_14bf18(m, l);
    let prev = m.r32(sp + 0xc);
    m.w32(l + 0xc, 0);
    m.w32(0x002d_d2f8, prev);
    stack_pop(m, 0x10);
}

/// FUN_0014c140: FUN_001fe9f8(0, 0x3dd040), then post event 0x189 with `p` on 0x3dd490 (FUN_0022b490)
pub fn ee_sub_14c140<M: QMem + ?Sized>(m: &mut M, p: u32) {
    m.call(0x001f_e9f8, &[0, 0x003d_d040], &[]);
    m.call(0x0022_b490, &[0x003d_d490, 0, 0x189, !0, p, !0], &[]);
}

/// FUN_0014c190: post event 0x189 from FUN_00210b98() (falling back to FUN_00210b98(0))
pub fn ee_sub_14c190<M: QMem + ?Sized>(m: &mut M) {
    let mut s = m.call(0x0021_0b98, &[], &[]).0;
    if s == 0 {
        s = m.call(0x0021_0b98, &[0], &[]).0;
    }
    m.call(0x0022_b490, &[0x003d_d490, s, 0x189, !0, 0, !0], &[]);
}

/// FUN_0014c200: fewer than 100 (FUN_001d82b8 on the 0x3e0ea8 tree) -> emitter 0x3e, else flag 0x1f on
pub fn ee_sub_14c200<M: QMem + ?Sized>(m: &mut M) {
    if (m.call(0x001d_82b8, &[0x003e_0ea8, 0x003d_4a60], &[]).0 as i32) < 100 {
        let e = m.call(EMITTER, &[0x3e], &[]).0;
        m.call(0x001f_b2f8, &[e, 0], &[]);
    } else {
        m.call(0x001d_af38, &[0x1f, 1], &[]);
    }
}

/// FUN_0014c258: with DAT_002e1e40: clear its 0x1f entry (FUN_00141e78 / FUN_00141fc8(.., -1)), flag
/// 0x1f off
pub fn ee_sub_14c258<M: QMem + ?Sized>(m: &mut M) {
    let x = m.r32(0x002e_1e40);
    if x == 0 {
        return;
    }
    let i = m.call(0x0014_1e78, &[x, 0x1f], &[]).0;
    if i != 0xffff_ffff {
        m.call(0x0014_1fc8, &[x, i, !0], &[]);
    }
    m.call(0x001d_af38, &[0x1f, 0], &[]);
}

/// FUN_0014c2b0: flag `f` on, then FUN_0019f0a8(0x3dd490)
pub fn ee_sub_14c2b0<M: QMem + ?Sized>(m: &mut M, f: u32) {
    m.call(0x001d_af38, &[f, 1], &[]);
    m.call(0x0019_f0a8, &[0x003d_d490], &[]);
}

/// FUN_0014c6a0: FUN_0014c6d8(s, s+0x74 word, q1, q2, a3, t0) (quad args shifted one register)
pub fn ee_sub_14c6a0<M: QMem + ?Sized>(m: &mut M, s: u32, q1: V4, q2: V4, a3: u32, t0: u32) {
    let k = m.r32(s + 0x74);
    m.call(0x0014_c6d8, &[s, k, a3, t0], &[q1, q2]);
}
