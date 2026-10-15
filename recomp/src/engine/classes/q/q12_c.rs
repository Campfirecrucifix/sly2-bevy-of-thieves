//! WP-Q12 part C: file package run_001aa820 (memory-card manager glue, mcman result codes, slot table).
//! 1:1 from ghidra_functions(_v2).body on `QMem` (conventions as q12_a). The card thread / semaphore functions
//! (FUN_001aaed8, 001aaf18, 001aaf90, 001ab040) and the save slots as files are replaced (game::save,
//! CardHost); everything else runs over `m.call` by address.
#![allow(unused_variables, unused_mut, dead_code)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem};
use crate::engine::classes::q::q6_runs::ee_sub_1ba9f8 as pools_init;

const NEG1: u32 = u32::MAX;
const CARD: u32 = 0x002e_6550;
const CARD_UI: u32 = 0x002e_65a0;
const SAVEVARS: u32 = 0x003e_0ea8;

/// FUN_001aa820 / FUN_001aa958: pool refcount init of FUN_001ba9f8 ((1, 0xffff))
pub fn ee_sub_1aa820<M: QMem + ?Sized>(m: &mut M, on: u32, k: u32) {
    pools_init(m, on, k);
}
pub fn ee_sub_1aa958<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_1aa820(m, 1, 0xffff);
}
/// FUN_001aa978: FUN_001613f8(0x2deb30); DAT_004425d8 = -1; FUN_001ab128(0x2e65e0, 0, 0)
pub fn ee_sub_1aa978<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x0016_13f8, &[0x002d_eb30], &[]);
    m.w32(0x0044_25d8, NEG1);
    ee_sub_1ab128(m, 0x002e_65e0, 0, 0);
}
/// FUN_001aa9b8(src, dst): sceMcTblGetDir entry (0x40 bytes) -> slot record (0x5c): name (src+0x20), size
/// (+0x10 -> +0x44), attribute bits of u16 +0x14 -> +0x48 (1, 2, 4, 8, 0x20 -> 0x10, 0x80 -> 0x20, 0x800 -> 0x40,
/// 0x1000 -> 0x80), created / modified times
pub fn ee_sub_1aa9b8<M: QMem + ?Sized>(m: &mut M, src: u32, dst: u32) {
    m.call(0x0023_33b4, &[dst, 0, 0x5c], &[]);
    m.call(0x0021_ed58, &[dst, src + 0x20], &[]);
    let v = m.r32(src + 0x10);
    m.w32(dst + 0x44, v);
    let fl = m.r16(src + 0x14) as u32;
    for (bit, out) in [(1, 1), (2, 2), (4, 4), (8, 8), (0x20, 0x10), (0x80, 0x20), (0x800, 0x40), (0x1000, 0x80)] {
        if fl & bit != 0 {
            let w = m.r32(dst + 0x48) | out;
            m.w32(dst + 0x48, w);
        }
    }
    let b = m.r8(src + 5);
    m.w8(dst + 0x4a, b);
    let b = m.r8(src + 4);
    m.w8(dst + 0x4b, b);
    let h = m.r16(src + 6);
    m.w16(dst + 0x4c, h);
    for (s, d) in [(3, 0x4e), (2, 0x4f), (1, 0x50), (0xd, 0x52), (0xc, 0x53)] {
        let b = m.r8(src + s);
        m.w8(dst + d, b);
    }
    let h = m.r16(src + 0xe);
    m.w16(dst + 0x54, h);
    for (s, d) in [(0xb, 0x56), (10, 0x57), (9, 0x58)] {
        let b = m.r8(src + s);
        m.w8(dst + d, b);
    }
}
/// FUN_001aab48: sce result sign: 0 -> 0x1d, > 0 -> 0x1e, < 0 -> 0x1f
pub fn ee_sub_1aab48(l: i64) -> u32 {
    if l != 0 {
        if 0 < l {
            0x1e
        } else {
            0x1f
        }
    } else {
        0x1d
    }
}
/// FUN_001aab68: sync state: 0 -> 0x21, -1 -> 0x20, 1 -> 0x22, else 0x1f
pub fn ee_sub_1aab68(p: i32) -> u32 {
    match p {
        0 => 0x21,
        -1 => 0x20,
        1 => 0x22,
        _ => 0x1f,
    }
}
/// FUN_001aabc0(kind, err): sceMc error code (command kind) -> card error code
pub fn ee_sub_1aabc0(kind: u32, p: i64) -> u32 {
    let tail = |b: bool, u: u32| if b { u } else { 0x1f };
    // `break` tail of the switch
    let brk = |p: i64| tail(p < -9, 0xb);
    // LAB_001aad24
    let d24 = |p: i64| if p == 0 { 0 } else { brk(p) };
    // switchD_001aacf8_caseD_fffffff9
    let f9 = |p: i64| if -1 < p { 0 } else { tail(p < -9, 0xb) };
    // LAB_001aae4c
    let e4c = |p: i64| if p == -2 { 0xc } else { f9(p) };
    let dd0 = |p: i64| if p == -4 { 0x16 } else { brk(p) };
    let i4 = p as i32;
    match kind {
        1 => {
            if p == -2 {
                return 10;
            }
            if p < -1 {
                if p == -5 {
                    return 0xb;
                }
                return tail(p < -9, 0xb);
            }
            if p == -1 {
                return 9;
            }
            d24(p)
        }
        2 => match i4 {
            -7 => 0x19,
            -5 => 6,
            -4 => 0xd,
            -3 => 1,
            -2 => 0xc,
            _ => f9(p),
        },
        3 | 10 => {
            if p == -2 {
                return 0xc;
            }
            if -2 < p {
                return d24(p);
            }
            if p == -4 {
                return 0x10;
            }
            brk(p)
        }
        4 => {
            if p == -4 {
                return 0x10;
            }
            e4c(p)
        }
        5 => {
            if p == -4 {
                return 0x10;
            }
            if -4 < p {
                return e4c(p);
            }
            if p == -5 {
                return 0x11;
            }
            f9(p)
        }
        6 => match i4 {
            -8 => 0x1a,
            -5 => 0x12,
            -4 => 0x10,
            -3 => 1,
            -2 => 0xc,
            _ => f9(p),
        },
        0xb => {
            if p == -3 {
                return 1;
            }
            if p < -2 {
                if p == -4 {
                    return 0x18;
                }
                return tail(p < -9, 0xb);
            }
            if p == -2 {
                return 0xc;
            }
            d24(p)
        }
        0xc => {
            if p == -2 {
                return 0xc;
            }
            if -2 < p {
                return d24(p);
            }
            if p == -4 {
                return 0x15;
            }
            brk(p)
        }
        0xd => {
            if p == -4 {
                return 0x15;
            }
            e4c(p)
        }
        0xe => {
            if p == -2 {
                return 0xc;
            }
            if -2 < p {
                return d24(p);
            }
            dd0(p)
        }
        0xf => match i4 {
            0 => 0,
            -6 => 0x17,
            -5 => 0x13,
            -4 => 0xe,
            -2 => 0xc,
            _ => brk(p),
        },
        0x10 => {
            if p == 0 {
                return 0;
            }
            tail(p < 0, 4)
        }
        0x11 => {
            if p == 0 {
                return 0;
            }
            tail(p < 0, 5)
        }
        0x12 => e4c(p),
        0x13 => {
            if p == -2 {
                return 0xc;
            }
            if p < -1 {
                return dd0(p);
            }
            d24(p)
        }
        0x14 => {
            if p == -0x193 {
                return 0x23;
            }
            if p == 0 {
                return 0;
            }
            0x1f
        }
        0x15 => {
            if -1 < p {
                return 0;
            }
            if (i4 as u32).wrapping_add(100) < 100 {
                return 0x1f;
            }
            tail(p < -99, 0x1e)
        }
        _ => 0x1f,
    }
}
/// FUN_001ab088: +4 != -1 (card thread created)
pub fn ee_sub_1ab088<M: QMem + ?Sized>(m: &mut M, o: u32) -> bool {
    m.r32(o + 4) != NEG1
}
/// FUN_001ab098(sema, sce result, command kind, out): maps an sce result: sign class FUN_001aab48; when 0x1d
/// (zero) sync with sceMcSync(0, &r) -> state FUN_001aab68, *out = r, and state 0x22 -> FUN_001aabc0(kind, r)
pub fn ee_sub_1ab098<M: QMem + ?Sized>(m: &mut M, o: u32, res: u32, kind: u32, out: u32) -> u32 {
    let mut l = ee_sub_1aab48(res as i32 as i64);
    if l == 0x1d {
        let buf = stack_push(m, 0x40);
        let u = m.call(0x0024_4450, &[0, buf, buf | 4], &[]).0;
        l = ee_sub_1aab68(u as i32);
        let r = m.r32(buf + 4);
        if out != 0 {
            m.w32(out, r);
        }
        if l == 0x22 {
            l = ee_sub_1aabc0(kind, r as i32 as i64);
        }
        stack_pop(m, 0x40);
    }
    l
}
/// FUN_001ab128(o, port, slot): FUN_00243cb0(); FUN_001aaed8(o + 0xfd5c); [0] = port, [1] = slot, [2] = -1, rest 0
pub fn ee_sub_1ab128<M: QMem + ?Sized>(m: &mut M, o: u32, port: u32, slot: u32) {
    m.call(0x0024_3cb0, &[], &[]);
    m.call(0x001a_aed8, &[o + 0xfd5c], &[]);
    m.w32(o, port);
    m.w32(o + 4, slot);
    m.w32(o + 8, NEG1);
    for off in [0x10, 0x20, 0xc, 0x14, 0x18, 0x1c] {
        m.w32(o + off, 0);
    }
}
/// FUN_001ab398(a, b): slot-time compare: u16 +0x54, then bytes +0x52, +0x53, +0x56, +0x57, +0x58
pub fn ee_sub_1ab398<M: QMem + ?Sized>(m: &mut M, a: u32, b: u32) -> i32 {
    let d = m.r16(a + 0x54) as i32 - m.r16(b + 0x54) as i32;
    if d != 0 {
        return d;
    }
    for off in [0x52u32, 0x53, 0x56, 0x57] {
        let d = m.r8(a + off) as i32 - m.r8(b + off) as i32;
        if d != 0 {
            return d;
        }
    }
    m.r8(a + 0x58) as i32 - m.r8(b + 0x58) as i32
}
/// FUN_001ab418(o, k): card state change (+8). Leaving 6: +0xfd54 = 0 and, when +0xfd50 != 0 and
/// DAT_002e6594 == 1, dialog mode 3 + busy hide / message 0x23 or 0x24; entering 3 / 0..1 starts the card
/// thread with DAT_002a6ba8 / DAT_002a6ba0; entering 6 picks the newest slot record (+0x24 + n * 0x5464) by
/// FUN_001ab398 (empty ones through FUN_001d8d80(slot, 0)); +0x10 = DAT_002ddf0c
pub fn ee_sub_1ab418<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32) {
    if k == m.r32(o + 8) {
        return;
    }
    if m.r32(o + 8) == 6 {
        m.w32(o + 0xfd54, 0);
        if m.r32(o + 0xfd50) != 0 && m.r32(0x002e_6594) == 1 {
            m.call(0x001a_cc38, &[CARD, 3], &[]);
            if m.call(0x001a_dea8, &[CARD_UI], &[]).0 == 0 {
                m.call(0x001a_cb58, &[CARD, 0x24], &[]);
            } else {
                m.call(0x001a_cb58, &[CARD, 0x23], &[]);
                m.call(0x001a_de70, &[CARD_UI], &[]);
            }
        }
    }
    let ki = k as i32;
    if k == 3 {
        let t = m.r32(0x002a_6ba8);
        m.call(0x001a_af90, &[o + 0xfd5c, t], &[]);
        m.w32(o + 8, 3);
    } else {
        if ki < 4 {
            if ki < 2 && -1 < ki {
                let t = m.r32(0x002a_6ba0);
                m.call(0x001a_af90, &[o + 0xfd5c, t], &[]);
                m.w32(o + 8, k);
                let c = m.r32(0x002d_df0c);
                m.w32(o + 0x10, c);
                return;
            }
        } else {
            if k != 6 {
                m.w32(o + 8, k);
                let c = m.r32(0x002d_df0c);
                m.w32(o + 0x10, c);
                return;
            }
            let mut iv3 = o + 0x24;
            m.w32(o + 0xfd54, iv3);
            let mut iv4 = o + 0x542c;
            let buf = stack_push(m, 0x30);
            for _ in 0..3 {
                let t = m.r32(0x003e_0fac);
                m.call(0x0025_81c0, &[buf, t, iv3], &[]);
                let f1 = m.rf(buf + 4);
                m.call(0x0012_9ec8, &[buf], &[]);
                if f1 == 0.0 {
                    m.call(0x001d_8d80, &[iv3, 0], &[]);
                } else {
                    let cur = m.r32(o + 0xfd54);
                    if ee_sub_1ab398(m, cur + 0x5408, iv4) < 0 {
                        m.w32(o + 0xfd54, iv3);
                    }
                }
                iv4 += 0x5464;
                iv3 += 0x5464;
            }
            stack_pop(m, 0x30);
        }
        m.w32(o + 8, k);
    }
    let c = m.r32(0x002d_df0c);
    m.w32(o + 0x10, c);
}
/// FUN_001ab630(o): state 5 with save vars loaded: FUN_001ab040(thread) (replaced: waits for the card thread),
/// FUN_001ac570 for the 3 slots, +0xc = 6, FUN_001ab418(o, 6)
pub fn ee_sub_1ab630<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.r32(o + 8) == 5 && m.r32(SAVEVARS) != 0 {
        let mut p = o + 0x24;
        m.call(0x001a_b040, &[o + 0xfd5c], &[]);
        for _ in 0..3 {
            ee_sub_1ac570(m, o, p);
            p += 0x5464;
        }
        m.w32(o + 0xc, 6);
        ee_sub_1ab418(m, o, 6);
    }
}
/// FUN_001ab6c0(o): sceMcGetInfo-style FUN_00244848(port, slot) -> kind 0x10: ok -> +0xc = 3, else +0xc = 2 and
/// dialog mode 3
pub fn ee_sub_1ab6c0<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let (a, b) = (m.r32(o), m.r32(o + 4));
    let u = m.call(0x0024_4848, &[a, b], &[]).0;
    let l = ee_sub_1ab098(m, o + 0xfd5c, u, 0x10, 0);
    if l == 0 {
        m.w32(o + 0xc, 3);
    } else {
        m.w32(o + 0xc, 2);
        m.call(0x001a_cc38, &[CARD, 3], &[]);
    }
    l
}
/// FUN_001ab750(p): size of the file at the location in p: FileLocation(&loc); with p: loc = {(p[0] ^ p[0xd]) +
/// 0xca0, p[1] ^ p[0xf], 2}; valid when the second word != 0 -> FUN_0014b670(&loc), else 0
pub fn ee_sub_1ab750<M: QMem + ?Sized>(m: &mut M, p: u32) -> u32 {
    let loc = stack_push(m, 0x10);
    m.w32(loc + 8, 2);
    m.call(0x0025_8248, &[loc], &[]);
    let ok;
    if p == 0 {
        ok = false;
    } else {
        let a = (m.r32(p) ^ m.r32(p + 0x34)).wrapping_add(0xca0);
        m.w32(loc, a);
        m.w32(loc + 8, 2);
        let b = m.r32(p + 4) ^ m.r32(p + 0x3c);
        m.w32(loc + 4, b);
        ok = b != 0;
    }
    let mut r = 0;
    if ok {
        r = m.call(0x0014_b670, &[loc], &[]).0;
    }
    stack_pop(m, 0x10);
    r
}
/// FUN_001ab868(o): card format check: sceMcGetInfo-style (kind 0x12, formatted flag) then, if the free space
/// (+0x1c clusters) holds the icon / save files (FUN_001ab750(0x2a38e8) in KB + 0x47 + 1 or 2) create the dir
/// (FUN_001ab7d0) and write the 3 slots (FUN_001ac468), then icon / dir entries (FUN_001aca40)
pub fn ee_sub_1ab868<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let mut iv7: i32 = 1;
    let buf = stack_push(m, 0xc0);
    let (a, b) = (m.r32(o), m.r32(o + 4));
    let u = m.call(0x0024_4a58, &[a, b, 0x002a_6c00], &[]).0;
    let mut l = ee_sub_1ab098(m, o + 0xfd5c, u, 0x12, buf);
    let r = 'out: {
        if l == 0 {
            if m.r32(buf) == 0 {
                iv7 = 2;
            }
            let iv3 = ee_sub_1ab750(m, 0x002a_38e8) as i32;
            let mut iv4 = iv3.wrapping_add(0x7fe);
            if -1 < iv3.wrapping_add(0x3ff) {
                iv4 = iv3.wrapping_add(0x3ff);
            }
            l = 1;
            if (iv7 + (iv4 >> 10) + 0x47) as i64 <= m.r32(o + 0x1c) as i32 as i64 {
                l = m.call(0x001a_b7d0, &[o], &[]).0;
                let mut p6 = o + 0x24;
                if l == 0 {
                    let mut i = 0;
                    loop {
                        l = m.call(0x001a_c468, &[o, p6], &[]).0;
                        i += 1;
                        if l != 0 {
                            break;
                        }
                        p6 += 0x5464;
                        if 2 < i {
                            if 4 < m.r32(o + 0xc) as i32 {
                                break 'out 0;
                            }
                            m.w32(o + 0xc, 5);
                            break 'out 0;
                        }
                    }
                }
                let mut p6 = o + 0x24;
                let pb = buf + 0x20;
                for _ in 0..3 {
                    m.call(0x001a_bb10, &[o, p6, pb], &[]);
                    ee_sub_1aca40(m, o, pb);
                    p6 += 0x5464;
                }
                ee_sub_1aca40(m, o, 0x002a_6c08);
                ee_sub_1aca40(m, o, 0x002a_6c28);
                ee_sub_1aca40(m, o, 0x002a_6bc8);
                ee_sub_1aca40(m, o, 0x002a_6bb0);
            }
            if l == 0 {
                break 'out 0;
            }
        }
        m.call(0x001a_cc38, &[CARD, 3], &[]);
        l
    };
    stack_pop(m, 0xc0);
    r
}
/// FUN_001aba10(o): list the card dir (kind 0xd, FUN_002446f0 with 0x2a6bb0): no entries -> free space
/// FUN_001ac6a0 < 0x19000 -> +0xc = 4, result 0xe; entries -> read the 3 slots (FUN_001ac4c8), +0xc >= 5
pub fn ee_sub_1aba10<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let buf = stack_push(m, 0x60);
    let (a, b) = (m.r32(o), m.r32(o + 4));
    let u = m.call(0x0024_46f0, &[a, b, 0x002a_6bb0, 0, NEG1, 0], &[]).0;
    let mut l = ee_sub_1ab098(m, o + 0xfd5c, u, 0xd, buf);
    if (m.r32(buf) as i32) < 1 {
        ee_sub_1ac6a0(m, o, buf | 4);
        if (m.r32(buf + 4) as i32) < 0x19000 {
            m.w32(o + 0xc, 4);
        }
        l = 0xe;
    } else if l == 0 {
        let mut p = o + 0x24;
        for _ in 0..3 {
            m.call(0x001a_c4c8, &[o, p], &[]);
            p += 0x5464;
        }
        if (m.r32(o + 0xc) as i32) < 5 {
            m.w32(o + 0xc, 5);
        }
    }
    stack_pop(m, 0x60);
    l
}
/// FUN_001abb60: FUN_001ac598; FUN_001ab7d0; FUN_001ac420(o, +0xfd50): any non-zero -> dialog mode 3; result
pub fn ee_sub_1abb60<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let mut l = ee_sub_1ac598(m, o);
    let mut fail = l != 0;
    if !fail {
        l = m.call(0x001a_b7d0, &[o], &[]).0;
        fail = l != 0;
    }
    if !fail {
        let s = m.r32(o + 0xfd50);
        l = m.call(0x001a_c420, &[o, s], &[]).0;
        fail = l != 0;
    }
    if fail {
        m.call(0x001a_cc38, &[CARD, 3], &[]);
    }
    l
}
/// FUN_001abbe0: FUN_001ac598; FUN_001ac420(o, +0xfd50): non-zero -> dialog mode 3, message 0x23, busy hide
pub fn ee_sub_1abbe0<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let mut l = ee_sub_1ac598(m, o);
    if l == 0 {
        let s = m.r32(o + 0xfd50);
        l = m.call(0x001a_c420, &[o, s], &[]).0;
    }
    if l != 0 {
        m.call(0x001a_cc38, &[CARD, 3], &[]);
        m.call(0x001a_cb58, &[CARD, 0x23], &[]);
        m.call(0x001a_de70, &[CARD_UI], &[]);
    }
    l
}
/// FUN_001abc68: FUN_001ac598; FUN_001ab7d0; FUN_001ac468(o, +0xfd58): any non-zero -> dialog mode 3; result
pub fn ee_sub_1abc68<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let mut l = ee_sub_1ac598(m, o);
    let mut fail = l != 0;
    if !fail {
        l = m.call(0x001a_b7d0, &[o], &[]).0;
        fail = l != 0;
    }
    if !fail {
        let s = m.r32(o + 0xfd58);
        l = m.call(0x001a_c468, &[o, s], &[]).0;
        fail = l != 0;
    }
    if fail {
        m.call(0x001a_cc38, &[CARD, 3], &[]);
    }
    l
}
/// FUN_001abce8: FUN_001d54b8(); FUN_001d84b0(0x3e0ea8, +0xfd50); FUN_001aaf90(thread, DAT_002a6c88)
pub fn ee_sub_1abce8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001d_54b8, &[], &[]);
    let s = m.r32(o + 0xfd50);
    m.call(0x001d_84b0, &[SAVEVARS, s], &[]);
    let t = m.r32(0x002a_6c88);
    m.call(0x001a_af90, &[o + 0xfd5c, t], &[]);
}
/// FUN_001abd98(o, force): autosave: slot chosen, world not busy (FUN_0020e040), DAT_002deb4c == 0, bif 99 set
/// or force: save var DAT_003e0fe4 of 0x3d4a60 == 0 -> DAT_002e659c = 3 + message 0x22, else FUN_001abce8 +
/// busy screen
pub fn ee_sub_1abd98<M: QMem + ?Sized>(m: &mut M, o: u32, force: u32) {
    if m.r32(o + 0xfd50) != 0
        && m.call(0x0020_e040, &[0x003d_d490], &[]).0 == 0
        && m.r32(0x002d_eb4c) == 0
        && (m.call(0x0021_0ae0, &[], &[]).0 != 0 || force != 0)
    {
        let buf = stack_push(m, 0x40);
        let t = m.r32(0x003e_0fe4);
        m.call(0x0025_81c0, &[buf, t, 0x003d_4a60], &[]);
        m.call(0x0012_9ec8, &[buf], &[]);
        if m.r32(buf + 4) == 0 {
            m.w32(0x002e_659c, 3);
            m.call(0x001a_cb58, &[CARD, 0x22], &[]);
        } else {
            ee_sub_1abce8(m, o);
            m.call(0x001a_de38, &[CARD_UI], &[]);
        }
        stack_pop(m, 0x40);
    }
}
/// FUN_001abe78(o, dir, max, out, n_out): list up to `max` (< 0x13) entries of `dir` (null -> "*" at 0x2e6500,
/// longer than 0x40 -> 0xd) into out records (0x5c each) through FUN_001aa9b8; max >= 0x13 -> 0x25
pub fn ee_sub_1abe78<M: QMem + ?Sized>(m: &mut M, o: u32, dir: u32, max: i64, out: u32, n_out: u32) -> u32 {
    if max < 0x13 {
        let mut dir = dir;
        let uv2;
        if dir == 0 {
            uv2 = 1;
            dir = 0x002e_6500;
        } else {
            let n = m.call(0x0021_ed28, &[dir], &[]).0;
            if 0x40 < n {
                return 0xd;
            }
            uv2 = 0;
            m.call(0x0021_ed58, &[0x002e_6500, dir], &[]);
        }
        let mut p6 = 0x002e_6080;
        m.call(0x0023_33b4, &[0x002e_6080, 0, 0x480], &[]);
        let (a, b) = (m.r32(o), m.r32(o + 4));
        let u = m.call(0x0024_46f0, &[a, b, dir, uv2, max as u32, 0x002e_6080], &[]).0;
        let mut l = ee_sub_1ab098(m, o + 0xfd5c, u, 0xd, n_out);
        if l == 0 {
            let mut i = 0;
            let mut out = out;
            if 0 < m.r32(n_out) as i32 {
                loop {
                    ee_sub_1aa9b8(m, p6, out);
                    i += 1;
                    out += 0x5c;
                    p6 += 0x40;
                    if !(i < m.r32(n_out) as i32) {
                        break;
                    }
                }
            }
            l = 0;
        }
        l
    } else {
        0x25
    }
}
/// FUN_001abfc0(o, dir, out): FUN_001abe78(o, dir, 1, out, &n): ok and n == 1 -> 0, ok and n != 1 -> 0xe
pub fn ee_sub_1abfc0<M: QMem + ?Sized>(m: &mut M, o: u32, dir: u32, out: u32) -> u32 {
    let n = stack_push(m, 0x20);
    let mut l = ee_sub_1abe78(m, o, dir, 1, out, n);
    if l == 0 {
        l = 0xe;
        if m.r32(n) == 1 {
            l = 0;
        }
    }
    stack_pop(m, 0x20);
    l
}
/// FUN_001ac238(o, name): FUN_001ac260(o, name, 0x2a38e8, 0x2a6ca0)
pub fn ee_sub_1ac238<M: QMem + ?Sized>(m: &mut M, o: u32, name: u32) -> u32 {
    ee_sub_1ac260(m, o, name, 0x002a_38e8, 0x002a_6ca0)
}
/// FUN_001ac260(o, name, loc, suffix): open the card file "%s%s" (mode 0x202), stream the level file `loc`
/// ({(p[0] ^ p[0xd]) + 0xca0, p[1] ^ p[0xf], 2}) into it in 0x1000-byte chunks (FUN_001ac8a8), close
/// (FUN_001ac808); errors 0x27 (no file) / 7 (short write)
pub fn ee_sub_1ac260<M: QMem + ?Sized>(m: &mut M, o: u32, name: u32, loc: u32, suffix: u32) -> u32 {
    // frame: top-relative offsets as in the original (0x1270 .. 0x80)
    let base = stack_push(m, 0x1280);
    let top = base + 0x1280;
    let a = |x: u32| top - x;
    m.call(0x001a_e478, &[a(0x1270)], &[]);
    m.call(0x0021_ecd8, &[a(0x1260), 0x002a_6cb8, name, suffix], &[]);
    m.w32(a(0x80), NEG1);
    let mut l = ee_sub_1ac7a0(m, o, a(0x1260), 0x202, a(0x80));
    if l == 0 {
        m.w32(a(0x1208), 2);
        m.call(0x0025_8248, &[a(0x1210)], &[]);
        m.w32(a(0x1100), 0x403f);
        m.w32(a(0x10fc), 0x10);
        let u = m.call(0x001a_e318, &[a(0x1100)], &[]).0;
        m.call(0x0018_48a8, &[a(0x10f0), 0x403f, u, 0], &[]);
        let ok;
        if loc == 0 {
            ok = false;
        } else {
            let v = (m.r32(loc) ^ m.r32(loc + 0x34)).wrapping_add(0xca0);
            m.w32(a(0x1210), v);
            m.w32(a(0x1208), 2);
            let w = m.r32(loc + 4) ^ m.r32(loc + 0x3c);
            m.w32(a(0x120c), w);
            ok = w != 0;
        }
        l = 0x27;
        if ok {
            l = m.call(0x0018_4d60, &[a(0x10f0), a(0x1210)], &[]).0;
            if l == 0 {
                l = 0x27;
            } else {
                loop {
                    let n = m.call(0x0011_7898, &[a(0x10f0), 0x1000, a(0x1080)], &[]).0;
                    if n == 0 {
                        l = 0;
                        break;
                    }
                    let h = m.r32(a(0x80));
                    l = ee_sub_1ac8a8(m, o, h, a(0x1080), n, a(0x7c));
                    if l != 0 {
                        break;
                    }
                    l = 7;
                    if m.r32(a(0x7c)) != n {
                        break;
                    }
                }
            }
        }
        m.call(0x0018_4cf0, &[a(0x10f0)], &[]);
        let h = m.r32(a(0x80));
        let l5 = ee_sub_1ac808(m, o, h);
        if l == 0 {
            l = l5;
        }
        m.call(0x0018_4950, &[a(0x10f0), 2], &[]);
    }
    m.call(0x001a_e4a0, &[a(0x1270), 2], &[]);
    stack_pop(m, 0x1280);
    l
}
/// FUN_001ac570(o, slot): FUN_001d7c80(0x3e0ea8) (save vars table check); 0
pub fn ee_sub_1ac570<M: QMem + ?Sized>(m: &mut M, o: u32, slot: u32) -> u32 {
    m.call(0x001d_7c80, &[SAVEVARS], &[]);
    0
}
/// FUN_001ac598(o): sceMcGetInfo (FUN_00244570 -> +0x18 type, +0x1c free, +0x20 format) through kind 1: result
/// state +0xc (0 -> 2, or 3 when formatted and +0xc <= 2; 9 -> 3; 10 / 0xc -> 2 with result 0xc; 0xb -> 1);
/// a wrong card type (+0x18 != 2) turns 0 / 0xc into 0xb and +0xc = 1; +0x14 = DAT_002ddf0c + 0.3
pub fn ee_sub_1ac598<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let (a, b) = (m.r32(o), m.r32(o + 4));
    let u = m.call(0x0024_4570, &[a, b, o + 0x18, o + 0x1c, o + 0x20], &[]).0;
    let mut l = ee_sub_1ab098(m, o + 0xfd5c, u, 1, 0);
    let mut set: Option<u32> = None;
    match l {
        0 => {
            let mut uv1 = 2;
            if m.r32(o + 0x20) != 0 {
                uv1 = 3;
                if 2 < m.r32(o + 0xc) as i32 {
                    uv1 = u32::MAX;
                }
            }
            if uv1 != u32::MAX {
                set = Some(uv1);
            }
        }
        9 => set = Some(3),
        10 | 0xc => {
            set = Some(2);
            l = 0xc;
        }
        0xb => set = Some(1),
        _ => {}
    }
    if let Some(v) = set {
        m.w32(o + 0xc, v);
    }
    if (l == 0 || l == 0xc) && m.r32(o + 0x18) != 2 {
        l = 0xb;
        m.w32(o + 0xc, 1);
    }
    let t = m.rf(0x002d_df0c) + 0.3;
    m.wf(o + 0x14, t);
    l
}
/// FUN_001ac6a0(o, out): FUN_00244570 + kind 1 (9 -> 0, 10 -> 0xc); *out = (+0x1c) << 10 when ok else 0
pub fn ee_sub_1ac6a0<M: QMem + ?Sized>(m: &mut M, o: u32, out: u32) -> u32 {
    let (a, b) = (m.r32(o), m.r32(o + 4));
    let u = m.call(0x0024_4570, &[a, b, o + 0x18, o + 0x1c, o + 0x20], &[]).0;
    let mut l = ee_sub_1ab098(m, o + 0xfd5c, u, 1, 0);
    if l == 9 {
        l = 0;
    } else if l == 10 {
        l = 0xc;
    }
    if l == 0 {
        let v = m.r32(o + 0x1c) << 10;
        m.w32(out, v);
    } else {
        m.w32(out, 0);
    }
    l
}
/// FUN_001ac748(o, path): FUN_00244018 (mkdir), kind 0xb, 0x18 (exists) -> 0
pub fn ee_sub_1ac748<M: QMem + ?Sized>(m: &mut M, o: u32, path: u32) -> u32 {
    let (a, b) = (m.r32(o), m.r32(o + 4));
    let u = m.call(0x0024_4018, &[a, b, path], &[]).0;
    let mut l = ee_sub_1ab098(m, o + 0xfd5c, u, 0xb, 0);
    if l == 0x18 {
        l = 0;
    }
    l
}
/// FUN_001ac7a0(o, path, mode, out): FUN_00243ef0 (open), kind 2
pub fn ee_sub_1ac7a0<M: QMem + ?Sized>(m: &mut M, o: u32, path: u32, mode: u32, out: u32) -> u32 {
    let (a, b) = (m.r32(o), m.r32(o + 4));
    let u = m.call(0x0024_3ef0, &[a, b, path, mode], &[]).0;
    ee_sub_1ab098(m, o + 0xfd5c, u, 2, out)
}
/// FUN_001ac808(o, fd): FUN_00244050 (close), kind 3
pub fn ee_sub_1ac808<M: QMem + ?Sized>(m: &mut M, o: u32, fd: u32) -> u32 {
    let u = m.call(0x0024_4050, &[fd], &[]).0;
    ee_sub_1ab098(m, o + 0xfd5c, u, 3, 0)
}
/// FUN_001ac850(o, fd, buf, n, out): FUN_002441c0 (read), kind 5
pub fn ee_sub_1ac850<M: QMem + ?Sized>(m: &mut M, o: u32, fd: u32, buf: u32, n: u32, out: u32) -> u32 {
    let u = m.call(0x0024_41c0, &[fd, buf, n], &[]).0;
    ee_sub_1ab098(m, o + 0xfd5c, u, 5, out)
}
/// FUN_001ac8a8(o, fd, buf, n, out): FUN_002442d8 (write), kind 6
pub fn ee_sub_1ac8a8<M: QMem + ?Sized>(m: &mut M, o: u32, fd: u32, buf: u32, n: u32, out: u32) -> u32 {
    let u = m.call(0x0024_42d8, &[fd, buf, n], &[]).0;
    ee_sub_1ab098(m, o + 0xfd5c, u, 6, out)
}
/// FUN_001ac900(o, path, buf, n): create+write (mode 0x202), short write -> 7
pub fn ee_sub_1ac900<M: QMem + ?Sized>(m: &mut M, o: u32, path: u32, buf: u32, n: u32) -> u32 {
    let a = stack_push(m, 0x20);
    m.w32(a, NEG1);
    let mut l = ee_sub_1ac7a0(m, o, path, 0x202, a);
    if l == 0 {
        let fd = m.r32(a);
        l = ee_sub_1ac8a8(m, o, fd, buf, n, a | 4);
        if l == 0 {
            l = 7;
            if m.r32(a + 4) == n {
                l = 0;
            }
        }
        let fd = m.r32(a);
        ee_sub_1ac808(m, o, fd);
    }
    stack_pop(m, 0x20);
    l
}
/// FUN_001ac9a0(o, path, buf, n): read (mode 1), short read -> 8
pub fn ee_sub_1ac9a0<M: QMem + ?Sized>(m: &mut M, o: u32, path: u32, buf: u32, n: u32) -> u32 {
    let a = stack_push(m, 0x20);
    m.w32(a, NEG1);
    let mut l = ee_sub_1ac7a0(m, o, path, 1, a);
    if l == 0 {
        let fd = m.r32(a);
        l = ee_sub_1ac850(m, o, fd, buf, n, a | 4);
        if l == 0 {
            l = 8;
            if m.r32(a + 4) == n {
                l = 0;
            }
        }
        let fd = m.r32(a);
        ee_sub_1ac808(m, o, fd);
    }
    stack_pop(m, 0x20);
    l
}
/// FUN_001aca40(o, path): FUN_00244918 (delete), kind 0xf, 0xe (missing) -> 0
pub fn ee_sub_1aca40<M: QMem + ?Sized>(m: &mut M, o: u32, path: u32) -> u32 {
    let (a, b) = (m.r32(o), m.r32(o + 4));
    let u = m.call(0x0024_4918, &[a, b, path], &[]).0;
    let mut l = ee_sub_1ab098(m, o + 0xfd5c, u, 0xf, 0);
    if l == 0xe {
        l = 0;
    }
    l
}
/// FUN_001aca98 (C2d3750_ctor): C2d85e8_ctor(this); vtable +0xc = 0x2d3750; +0x20 = 0x52c; +0x40 = -1; FUN_001da7d0
pub fn ee_sub_1aca98<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    m.call(0x001d_a728, &[o], &[]);
    m.w32(o + 0xc, 0x002d_3750);
    m.w32(o + 0x20, 0x52c);
    m.w32(o + 0x40, NEG1);
    m.call(0x001d_a7d0, &[o], &[]);
    o
}
