//! WP-Q5: run_001dc598 queue package: the job / challenge HUD caption builders (mission tracker
//! 0x3da0f0 / 0x3da150, challenge table 0x2bcd00 (stride 0x20), loot table 0x2c3284 (stride 0xc);
//! localised strings FUN_0013e368(0x3dd490, id), strcpy FUN_0021ed58, sprintf FUN_0021ecd8), the
//! C2d8468 / C2d83e8 / C2d8368 menu-screen constructors and the C2d8468 job-list screen (FUN_001dd300
//! mode switch, FUN_001dced8 list rebuild, FUN_001dce90 selection clamp). 1:1 against `QMem`.
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem};

const LOC: u32 = 0x003d_d490;
/// challenge table (0x20 per entry): +8 target, +0x14 name, +0x18 hint, +0x1c format
const CH: u32 = 0x002b_cd00;
/// DAT_002ddf0c: real clock
const CLOCK: u32 = 0x002d_df0c;

fn loc<M: QMem + ?Sized>(m: &mut M, id: u32) -> u32 {
    m.call(0x0013_e368, &[LOC, id], &[]).0
}

fn cpy<M: QMem + ?Sized>(m: &mut M, d: u32, s: u32) {
    m.call(0x0021_ed58, &[d, s], &[]);
}

/// the job id of list row `i` (0x3da19c + (DAT_003da3a0 + i) * 8)
fn row<M: QMem + ?Sized>(m: &mut M, i: u32) -> u32 {
    let b = m.r32(0x003d_a3a0).wrapping_add(i);
    m.r32(b.wrapping_mul(8).wrapping_add(0x003d_a19c))
}

/// FUN_001dc598: current-challenge caption: with a running message (DAT_003da120 != -1) its string
/// (DAT_003da128); no challenge (DAT_003da138 == -1) -> string 0xdd; else the target line (format
/// +0x1c with the rank string 0x2c3268[rank] three times, or string 0xe1 for an unranked flagged
/// challenge) is sprintf'd with name +0x14 and hint +0x18 through 0x2a7a18
pub fn ee_sub_1dc598<M: QMem + ?Sized>(m: &mut M, d: u32) {
    if m.r32(0x003d_a120) != 0xffff_ffff {
        let id = m.r32(0x003d_a128);
        let s = loc(m, id);
        cpy(m, d, s);
        return;
    }
    let c = m.r32(0x003d_a138);
    if c == 0xffff_ffff {
        let s = loc(m, 0xdd);
        cpy(m, d, s);
        return;
    }
    let sp = stack_push(m, 0x100);
    let j = m.call(0x001d_ae20, &[0x003d_a0f0], &[]).0;
    let w = m.call(0x001c_4840, &[LOC], &[]).0;
    let c = m.r32(0x003d_a138);
    let rank = m.call(0x0014_1e78, &[w, c], &[]).0 as i32;
    let hint;
    if m.r32(j + 0x1c) & 1 != 0 && rank < 0 {
        let s = loc(m, 0xe1);
        cpy(m, sp, s);
        hint = 0x002a_7a10;
    } else {
        let r = if rank < 0 { 0x002a_7a10 } else { m.r32(0x002c_3268u32.wrapping_add((rank as u32) << 2)) };
        let c = m.r32(0x003d_a138);
        let f = m.r32(CH + (c << 5) + 0x1c);
        let fs = loc(m, f);
        m.call(0x0021_ecd8, &[sp, fs, r, r, r], &[]);
        let c = m.r32(0x003d_a138);
        let h = m.r32(CH + (c << 5) + 0x18);
        hint = loc(m, h);
    }
    let c = m.r32(0x003d_a138);
    let n = m.r32(CH + (c << 5) + 0x14);
    let name = loc(m, n);
    let sep = if m.r8(hint) == 0 { 0x002a_7a10 } else { 0x002a_7a28 };
    m.call(0x0021_ecd8, &[d, 0x002a_7a18, name, sep, hint, sp], &[]);
    stack_pop(m, 0x100);
}

/// FUN_001dc790: name of challenge row `i` (empty when out of range or -1)
pub fn ee_sub_1dc790<M: QMem + ?Sized>(m: &mut M, d: u32, i: u32) {
    let c = row(m, i);
    let id = if c < 0x2b { m.r32(CH + (c << 5) + 0x14) } else { 0xffff_ffff };
    if id == 0xffff_ffff {
        m.w8(d, 0);
    } else {
        let s = loc(m, id);
        cpy(m, d, s);
    }
}

/// FUN_001dc820: target number of challenge row `i` ("%d" 0x2a79a0; empty when 0)
pub fn ee_sub_1dc820<M: QMem + ?Sized>(m: &mut M, d: u32, i: u32) {
    let c = row(m, i);
    let v = if c < 0x2b { m.r32(CH + (c << 5) + 8) } else { 0 };
    if v == 0 {
        m.w8(d, 0);
    } else {
        m.call(0x0021_ecd8, &[d, 0x002a_79a0, v], &[]);
    }
}

/// FUN_001dc890: loot row `i`: the name (0x2c3288 + k * 0xc + 4) with " x%d" (0x2a7a30) when the
/// collected count from the save key "<0x2a79d0><0x2a79d8><k>" (FUN_001d8a38 / FUN_00129658) is > 1
pub fn ee_sub_1dc890<M: QMem + ?Sized>(m: &mut M, d: u32, i: u32) {
    let k = row(m, i);
    let mut id = 0xffff_ffff;
    let mut n = 0i32;
    if k < 0x40 {
        let sp = stack_push(m, 0x40);
        m.w32(sp, 0xffff_ffff);
        m.call(0x0021_ecd8, &[sp + 0x10, 0x002a_79a0, k], &[]);
        m.call(0x001d_8a38, &[sp + 0x30, 0x002a_79d0, 0x002a_79d8, sp + 0x10, 0], &[]);
        m.call(0x0012_9658, &[sp, sp + 0x30], &[]);
        m.call(0x0012_9ec8, &[sp + 0x30], &[]);
        n = m.r32(sp + 4) as i32;
        id = m.r32(k.wrapping_mul(0xc).wrapping_add(0x002c_328c));
        m.call(0x0012_9ec8, &[sp], &[]);
        stack_pop(m, 0x40);
    }
    if id == 0xffff_ffff {
        m.w8(d, 0);
        return;
    }
    let s = loc(m, id);
    cpy(m, d, s);
    if n > 1 {
        m.call(0x0021_ecd8, &[d, 0x002a_7a30, d, n as u32], &[]);
    }
}

/// FUN_001dc9c0: value of loot row `i` ("%d"; empty when 0)
pub fn ee_sub_1dc9c0<M: QMem + ?Sized>(m: &mut M, d: u32, i: u32) {
    let k = row(m, i);
    let v = if k < 0x40 { m.r32(k.wrapping_mul(0xc).wrapping_add(0x002c_3288)) } else { 0 };
    if v == 0 {
        m.w8(d, 0);
    } else {
        m.call(0x0021_ecd8, &[d, 0x002a_79a0, v], &[]);
    }
}

/// FUN_001dca30: result caption by DAT_003da180: 0 -> 0xdf, -1 -> 0xde, 1 -> 0xe0, else empty
pub fn ee_sub_1dca30<M: QMem + ?Sized>(m: &mut M, d: u32) {
    let id = match m.r32(0x003d_a180) as i32 {
        0 => 0xdf,
        -1 => 0xde,
        1 => 0xe0,
        _ => {
            m.w8(d, 0);
            return;
        }
    };
    let s = loc(m, id);
    cpy(m, d, s);
}

/// FUN_001dcab0: detail caption: with a result (DAT_003da180 != -1) string DAT_003da188; page 0:
/// challenge DAT_003da3a4 with a hint -> sprintf(0x2a7a38, name, hint, 0xd9 / 0xda by the score
/// FUN_00161798(0x2deb30) vs the target) else 0xdc; page 1: loot DAT_003da3a8 -> sprintf(string 0xdb,
/// value) else 0xe3; other pages empty
pub fn ee_sub_1dcab0<M: QMem + ?Sized>(m: &mut M, d: u32) {
    let id = if m.r32(0x003d_a180) != 0xffff_ffff {
        m.r32(0x003d_a188)
    } else {
        match m.r32(0x003d_a190) {
            0 => {
                let c = m.r32(0x003d_a3a4);
                if c < 0x2b && m.r32(CH + (c << 5) + 0x18) != 0xffff_ffff {
                    let e = CH + (c << 5);
                    let sc = m.call(0x0016_1798, &[0x002d_eb30], &[]).0 as i32;
                    let r = if sc < m.r32(e + 8) as i32 { 0xd9 } else { 0xda };
                    let n = m.r32(e + 0x14);
                    let a = loc(m, n);
                    let h = m.r32(e + 0x18);
                    let b = loc(m, h);
                    let c = loc(m, r);
                    m.call(0x0021_ecd8, &[d, 0x002a_7a38, a, b, c], &[]);
                    return;
                }
                0xdc
            }
            1 => {
                let k = m.r32(0x003d_a3a8);
                if k < 0x40 {
                    let f = loc(m, 0xdb);
                    let v = m.r32(k.wrapping_mul(0xc).wrapping_add(0x002c_3288));
                    m.call(0x0021_ecd8, &[d, f, v], &[]);
                    return;
                }
                0xe3
            }
            _ => {
                m.w8(d, 0);
                return;
            }
        }
    };
    let s = loc(m, id);
    cpy(m, d, s);
}

fn ctor<M: QMem + ?Sized>(m: &mut M, o: u32, size: u32, vt: u32) -> u32 {
    m.call(0x001d_a728, &[o], &[]);
    m.w32(o + 0x20, size);
    m.w32(o + 0xc, vt);
    o
}

/// C2d8468_ctor (FUN_001dcc58): C2d85e8 base, id 0x4e0, vtable 0x2d8468, FUN_001dcd08
pub fn ee_sub_1dcc58<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    ctor(m, o, 0x4e0, 0x002d_8468);
    ee_sub_1dcd08(m, o);
    o
}

/// FUN_001dcd08: mode -1, start time = clock, selection cells -1
pub fn ee_sub_1dcd08<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.w32(o + 0x40, 0xffff_ffff);
    let c = m.r32(CLOCK);
    m.w32(o + 0x124, 0xffff_ffff);
    m.w32(o + 0x44, c);
    m.w32(o + 0x11c, 0xffff_ffff);
    m.w32(o + 0x120, 0xffff_ffff);
}

/// FUN_001dce90: selection +0x118 = clamp(k, 0, count - 1) (-1 when empty), FUN_001da9e8
pub fn ee_sub_1dce90<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32) {
    let n = m.r32(o + 0x48) as i32;
    let s = if n == 0 {
        -1
    } else if (k as i32) < 0 {
        0
    } else if n - 1 < k as i32 {
        n - 1
    } else {
        k as i32
    };
    m.w32(o + 0x118, s as u32);
    m.call(0x001d_a9e8, &[o, k], &[]);
}

fn push_row<M: QMem + ?Sized>(m: &mut M, o: u32, e: u32) -> u32 {
    let n = m.r32(o + 0x48);
    m.w32(o + 0x48, n + 1);
    let r = o + n * 0xc + 0x4c;
    m.w32(r + 4, 0xffff_ffff);
    m.w32(r, e);
    m.w32(r + 8, 0xffff_ffff);
    r
}

/// FUN_001dced8: rebuild the list rows {job, sound, sound} at +0x4c: stop the rows' sounds
/// (FUN_00213928 on +0x24), clear; the jobs (FUN_00165c98, sorted by FUN_002338c0 with 0x165c28):
/// mode 0 one row per distinct available district (+0x7c, FUN_00163430 > 0, +0x6c != -1), the one
/// equal to +0x110 selected; mode 1 the district +0x110's jobs (+0x5c != -1) in order, up to the
/// first locked one (+0x54 == 0) or, once a done one (+0x54 == 1) was listed, the next locked one.
/// The list keeps its last +0x10c rows (selection shifted); then FUN_001dce90(selection).
pub fn ee_sub_1dced8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let mut i = 0u32;
    while (i as i32) < m.r32(o + 0x48) as i32 {
        let r = o + 0x4c + 0xc * i;
        for k in [4, 8] {
            if m.r32(r + k) != 0xffff_ffff {
                let s = m.r32(o + 0x24);
                let h = m.r32(r + k);
                m.call(0x0021_3928, &[s, h], &[]);
                m.w32(r + k, 0xffff_ffff);
            }
        }
        i += 1;
    }
    m.w32(o + 0x118, 0);
    m.w32(o + 0x48, 0);
    m.call(0x0023_33b4, &[o + 0x4c, 0, 0], &[]);
    let sp = stack_push(m, 0x10);
    m.call(0x001a_e478, &[sp], &[]);
    m.call(0x0016_5c98, &[sp + 4, sp + 8], &[]);
    let (l, n) = (m.r32(sp + 8), m.r32(sp + 4));
    m.call(0x0023_38c0, &[l, n, 4, 0x0016_5c28], &[]);
    let n = m.r32(sp + 4) as i32;
    let l = m.r32(sp + 8);
    match m.r32(o + 0x40) {
        0 => {
            let mut last = 0u32;
            for i in 0..n.max(0) as u32 {
                let j = m.r32(l + 4 * i);
                let g = m.r32(j + 0x7c);
                if g == 0 || g == last {
                    continue;
                }
                last = g;
                if (m.call(0x0016_3430, &[g], &[]).0 as i32) <= 0 || m.r32(g + 0x6c) == 0xffff_ffff {
                    continue;
                }
                let r = push_row(m, o, g);
                if m.r32(o + 0x110) == g {
                    m.w32(o + 0x118, (r - 0x4c - o) / 0xc);
                }
            }
        }
        1 => {
            let mut seen = false;
            for i in 0..n.max(0) as u32 {
                let j = m.r32(l + 4 * i);
                if m.r32(o + 0x110) != m.r32(j + 0x7c) || m.r32(j + 0x5c) == 0xffff_ffff {
                    continue;
                }
                if seen && m.r32(j + 0x54) == 0 {
                    break;
                }
                push_row(m, o, j);
                let st = m.r32(j + 0x54);
                if st == 1 {
                    seen = true;
                }
                if st == 0 {
                    break;
                }
            }
        }
        _ => {}
    }
    let n = m.r32(o + 0x48) as i32;
    let keep = m.r32(o + 0x10c) as i32;
    if keep < n {
        let src = o + 0x4c + ((n - keep) as u32).wrapping_mul(0xc);
        m.call(0x0023_32b8, &[o + 0x4c, src, (keep as u32).wrapping_mul(0xc)], &[]);
        let cut = m.r32(o + 0x48) as i32 - m.r32(o + 0x10c) as i32;
        let s = m.r32(o + 0x118) as i32;
        m.w32(o + 0x118, if cut < s { (s - cut) as u32 } else { 0 });
        let k = m.r32(o + 0x10c);
        m.w32(o + 0x48, k);
    }
    let s = m.r32(o + 0x118);
    ee_sub_1dce90(m, o, s);
    m.call(0x001a_e4a0, &[sp, 2], &[]);
    stack_pop(m, 0x10);
}

/// panel +0x24 refresh: FUN_00212a70(p, 1), FUN_00212a50(p, 2), slot 15
fn refresh<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let p = m.r32(o + 0x24);
    if m.r32(p + 0x58) == 0 {
        return;
    }
    m.call(0x0021_2a70, &[p, 1], &[]);
    let p = m.r32(o + 0x24);
    m.call(0x0021_2a50, &[p, 2], &[]);
    let p = m.r32(o + 0x24);
    vslot(m, p, 15, &[], &[]);
}

/// FUN_001dd300: switch the job screen to mode `k` (forced -1 without a panel +0x24): leaving -1
/// pauses (FUN_0020efc0, slot 9); 0 = district list (10 rows, labels 0x4e3 / 0x4e2 / 0x4e8 back from
/// jobs else 0x4e1); 1 = the selected district's jobs (+0x110, 4 rows, +0x114 = came from the list,
/// labels 0x4e6 / 0x4e5 / 0x4e7 or 0x4e4); -1 = close (FUN_0020f030, slot 10, FUN_00212a50(p, 1)
/// with a shown panel, selection cells -1). Then the list is rebuilt and slot 13(-1) runs.
pub fn ee_sub_1dd300<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32) {
    let k = if m.r32(o + 0x24) != 0 { k } else { 0xffff_ffff };
    if k == m.r32(o + 0x40) {
        return;
    }
    if m.r32(o + 0x40) == 0xffff_ffff {
        m.call(0x0020_efc0, &[LOC, 0], &[]);
        vslot(m, o, 9, &[], &[]);
    }
    let prev = m.r32(o + 0x40);
    m.w32(o + 0x40, k);
    let c = m.r32(CLOCK);
    m.w32(o + 0x44, c);
    match k as i32 {
        0 => {
            m.w32(o + 0x10c, 10);
            let p = m.r32(o + 0x24);
            m.w32(p + 0x70, 0x4e3);
            let p = m.r32(o + 0x24);
            m.w32(p + 0x78, 0x4e2);
            let p = m.r32(o + 0x24);
            m.w32(p + 0x74, if prev == 1 { 0x4e8 } else { 0x4e1 });
            refresh(m, o);
        }
        1 => {
            let s = m.r32(o + 0x118) as i32;
            let g = if s >= 0 && s < m.r32(o + 0x48) as i32 {
                m.r32(o + 0x4c + 0xc * s as u32)
            } else {
                m.r32(0x003e_0f9c)
            };
            m.w32(o + 0x110, g);
            let p = m.r32(o + 0x24);
            m.w32(o + 0x114, (prev == 0) as u32);
            m.w32(o + 0x10c, 4);
            m.w32(p + 0x70, 0x4e6);
            let p = m.r32(o + 0x24);
            m.w32(p + 0x78, 0x4e5);
            let p = m.r32(o + 0x24);
            m.w32(p + 0x74, if prev == 0 { 0x4e7 } else { 0x4e4 });
            refresh(m, o);
        }
        -1 => {
            m.call(0x0020_f030, &[LOC, 0], &[]);
            vslot(m, o, 10, &[], &[]);
            let p = m.r32(o + 0x24);
            if m.r32(p + 0x58) as i32 > 0 {
                m.call(0x0021_2a50, &[p, 1], &[]);
            }
            m.w32(o + 0x124, 0xffff_ffff);
            m.w32(o + 0x11c, 0xffff_ffff);
            m.w32(o + 0x120, 0xffff_ffff);
        }
        _ => {}
    }
    ee_sub_1dced8(m, o);
    vslot(m, o, 13, &[0xffff_ffff], &[]);
}

/// C2d83e8_ctor (FUN_001dd648): C2d85e8 base, id 0x52d, vtable 0x2d83e8, FUN_001da7d0
pub fn ee_sub_1dd648<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    ctor(m, o, 0x52d, 0x002d_83e8);
    m.call(0x001d_a7d0, &[o], &[]);
    o
}

/// FUN_001dd6c8: with a panel: slot 9, panel slot 48(-1)
pub fn ee_sub_1dd6c8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.r32(o + 0x24) == 0 {
        return;
    }
    vslot(m, o, 9, &[], &[]);
    let p = m.r32(o + 0x24);
    vslot(m, p, 48, &[0xffff_ffff], &[]);
}

/// FUN_001dd728 / FUN_001dd888: with a panel: slot 10, panel slot 49
pub fn ee_sub_1dd728<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.r32(o + 0x24) == 0 {
        return;
    }
    vslot(m, o, 10, &[], &[]);
    let p = m.r32(o + 0x24);
    vslot(m, p, 49, &[], &[]);
}

/// FUN_001dd780: panel shown and settled (+0x58 == 2 and +0x64 == 2)
pub fn ee_sub_1dd780<M: QMem + ?Sized>(m: &mut M, o: u32) -> bool {
    let p = m.r32(o + 0x24);
    p != 0 && m.r32(p + 0x58) == 2 && m.r32(p + 0x64) == 2
}

/// C2d8368_ctor (FUN_001dd7d8): C2d85e8 base, id 0x5d2, vtable 0x2d8368, FUN_001da7d0
pub fn ee_sub_1dd7d8<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    ctor(m, o, 0x5d2, 0x002d_8368);
    m.call(0x001d_a7d0, &[o], &[]);
    o
}

/// FUN_001dd820: FUN_001dd6c8 then FUN_001d5498
pub fn ee_sub_1dd820<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.r32(o + 0x24) == 0 {
        return;
    }
    ee_sub_1dd6c8(m, o);
    m.call(0x001d_5498, &[], &[]);
}

// WIRE-A: C2d84e8 (ThiefNet shop) list helpers, 1:1 against `QMem` (the struct port lives in
// ui/thiefnet.rs; these run the same code on the flat QMem object so the live menu populates).

/// FUN_001da950(o, mode, p3): +0x38 = p3, +0x30 = mode; mode 0 -> +0x34 = DAT_002a7984 + sound 0x29;
/// mode -1 -> +0x34 = -10.0; else +0x34 = clock + DAT_002c3254 + sound 0x36
pub fn ee_sub_1da950<M: QMem + ?Sized>(m: &mut M, o: u32, mode: u32, p3: u32) {
    m.w32(o + 0x38, p3);
    m.w32(o + 0x30, mode);
    if mode == 0 {
        let v = m.r32(0x002a_7984);
        m.w32(o + 0x34, v);
        let s = m.call(0x001f_bb90, &[0x29], &[]).0;
        m.call(0x001f_b2f8, &[s, 0], &[]);
    } else if mode == 0xffff_ffff {
        m.w32(o + 0x34, 0xc120_0000);
    } else {
        let t = f32::from_bits(m.r32(CLOCK)) + f32::from_bits(m.r32(0x002c_3254));
        m.w32(o + 0x34, t.to_bits());
        let s = m.call(0x001f_bb90, &[0x36], &[]).0;
        m.call(0x001f_b2f8, &[s, 0], &[]);
    }
}

/// FUN_001da9e8: FUN_001da950(o, -1, -1)
pub fn ee_sub_1da9e8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    ee_sub_1da950(m, o, 0xffff_ffff, 0xffff_ffff);
}

/// FUN_001dae58(id): gadget table 0x2bcd10[id*0x20] != -1 && <= FUN_00210b28() (episodeBest)
pub fn ee_sub_1dae58<M: QMem + ?Sized>(m: &mut M, id: u32) -> u32 {
    let req = m.r32(0x002b_cd10 + id.wrapping_mul(0x20)) as i32;
    if req == -1 {
        return 0;
    }
    let best = m.call(0x0021_0b28, &[], &[]).0 as i32;
    (req <= best) as u32
}

/// FUN_001daea8(id): save value ("powerUps"/"mppukfAcquired"/"%d" id) +4; 0 for id -1
pub fn ee_sub_1daea8<M: QMem + ?Sized>(m: &mut M, id: u32) -> u32 {
    if id == 0xffff_ffff {
        return 0;
    }
    let base = stack_push(m, 0x70);
    let top = base + 0x70;
    let l = |x: u32| top - x;
    m.w32(l(0x60), 0xffff_ffff);
    m.call(0x0021_ecd8, &[l(0x50), 0x002a_79a0, id], &[]);
    m.call(0x001d_8a38, &[l(0x30), 0x002a_79a8, 0x002a_79b8, l(0x50), 0], &[]);
    m.call(0x0012_9658, &[l(0x60), l(0x30)], &[]);
    m.call(0x0012_9ec8, &[l(0x30)], &[]);
    let r = m.r32(l(0x5c));
    m.call(0x0012_9ec8, &[l(0x60)], &[]);
    stack_pop(m, 0x70);
    r
}

/// FUN_001dbba0(o): visible-row bound
pub fn ee_sub_1dbba0<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let first = m.r32(o + 0x250) as i32;
    let mut r = first.wrapping_add(m.r32(0x002c_35b4) as i32);
    if m.r32(0x003d_a570) != 0 {
        r -= (0 < first) as i32;
        r -= (r < m.r32(o + 0x48) as i32) as i32;
    }
    r as u32
}

/// FUN_001dbbe0(o, sel): clamp the selection (+0x24c), scroll (+0x250), selected id (+0x254 / +0x258)
pub fn ee_sub_1dbbe0<M: QMem + ?Sized>(m: &mut M, o: u32, sel: u32) {
    let count = m.r32(o + 0x48) as i32;
    if count == 0 {
        m.w32(o + 0x250, 0);
        m.w32(o + 0x258, 0xffff_ffff);
        m.w32(o + 0x24c, 0xffff_ffff);
        m.w32(o + 0x254, 0xffff_ffff);
    } else {
        let mut p = (sel as i32).max(0).min(count - 1);
        m.w32(o + 0x24c, p as u32);
        if m.r32(0x003d_a570) == 0 {
            let cur = m.r32(o + 0x250) as i32;
            let lo = (p - m.r32(0x002c_35b4) as i32 + 1).max(0);
            if cur < lo {
                m.w32(o + 0x250, lo as u32);
            } else {
                if cur <= p {
                    p = cur;
                }
                m.w32(o + 0x250, p as u32);
            }
        } else {
            if p < m.r32(o + 0x250) as i32 {
                m.w32(o + 0x250, p as u32);
            } else {
                while (ee_sub_1dbba0(m, o) as i32) - 1 < m.r32(o + 0x24c) as i32 {
                    let f = m.r32(o + 0x250);
                    m.w32(o + 0x250, f.wrapping_add(1));
                }
            }
            if m.r32(o + 0x250) == 1 {
                m.w32(o + 0x250, 0);
            }
        }
        let s = m.r32(o + 0x24c);
        let id = m.r32(o + 0x4c + s.wrapping_mul(8));
        m.w32(o + 0x254, id);
        m.w32(o + 0x258, id);
    }
    ee_sub_1da9e8(m, o);
}

/// FUN_001dbd10(o): rebuild the list (+0x48 count, +0x4c entries stride 8): page 0 = available,
/// not-yet-acquired gadgets 7..0x2b; page 1 = loot kinds 0..0x40 with a non-zero save value
pub fn ee_sub_1dbd10<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let n = m.r32(o + 0x48) as i32;
    for i in 0..n.max(0) as u32 {
        m.w32(o + 0x4c + i * 8, 0xffff_ffff);
    }
    m.w32(o + 0x48, 0);
    let page = m.r32(o + 0x40);
    let push = |m: &mut M, id: u32| -> bool {
        let c = m.r32(o + 0x48);
        if 0x40 < c {
            return false;
        }
        m.w32(o + 0x4c + c * 8, id);
        m.w32(o + 0x48, c + 1);
        true
    };
    if page == 0 {
        for g in 7..0x2b {
            if m.call(0x001d_ae58, &[g], &[]).0 != 0 && m.call(0x001d_aea8, &[g], &[]).0 == 0 && !push(m, g) {
                break;
            }
        }
    } else if page == 1 {
        let base = stack_push(m, 0xf0);
        let top = base + 0xf0;
        let l = |x: u32| top - x;
        for i in 0..0x40 {
            m.w32(l(0xe0), 0xffff_ffff);
            m.call(0x0021_ecd8, &[l(0xd0), 0x002a_79a0, i], &[]);
            m.call(0x001d_8a38, &[l(0xb0), 0x002a_79d0, 0x002a_79d8, l(0xd0), 0], &[]);
            m.call(0x0012_9658, &[l(0xe0), l(0xb0)], &[]);
            m.call(0x0012_9ec8, &[l(0xb0)], &[]);
            let ok = m.r32(l(0xdc)) == 0 || push(m, i);
            m.call(0x0012_9ec8, &[l(0xe0)], &[]);
            if !ok {
                break;
            }
        }
        stack_pop(m, 0xf0);
    }
    let sel = m.r32(o + 0x24c);
    ee_sub_1dbbe0(m, o, sel);
}
