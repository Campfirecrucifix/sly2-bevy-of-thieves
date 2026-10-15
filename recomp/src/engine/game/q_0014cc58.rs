//! WP-Q13: file:run_0014cc58 .. run_00167b68 (first half): closest-hit scan, cycle-counter wrap, emitter spawn,
//! camera-flag globals, pixel 5551 packer.
//! SKIPPED here: nothing; PARTIAL: FUN_00167f78 (VIF1 FBRST / SYNC / EI hardware ops dropped), FUN_0015b1b0
//! (argument registers of the virtual spawn call at vtable +0xc are mis-decompiled).
use super::q_00114680::*;
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem, V4};


/// FUN_0014cc58: closest-hit scan over `n` entries of `tbl`; each entry object is probed with FUN_0014c6a0
/// (count 2, results = two 0x60-byte records, distance float at record+0x40); keeps the entry with the
/// smallest distance below DAT_002a4ff4 and copies its 0x60-byte record to `out`. `lVar7` of the decompile
/// (the entry's *address*, kept in a callee-saved register across the call) is used literally.
pub fn ee_sub_14cc58<M: QMem + ?Sized>(m: &mut M, p1: u64, a1: V4, a2: V4, n: i32, tbl: u32, out: u32) -> u32 {
    let _ = a2;
    let mut cnt: i32 = 0;
    let mut best: u32 = 0;
    if 0 < n {
        let sp = stack_push(m, 0x1a0);
        let mut off: u32 = 0;
        let mut dmax = m.rf(0x002a_4ff4);
        loop {
            let l7 = off.wrapping_add(tbl) as i32 as i64;
            let obj = m.r32(off.wrapping_add(tbl));
            m.call(0x0014_c6a0, &[obj, 2, sp], &[a1]);
            let mut l10: i64 = 0;
            let b1;
            if l7 < 1 {
                b1 = l10 < l7;
            } else if m.rf(sp + 0x40) == 0.0 {
                l10 = if p1 & 1 != 0 { 1 } else { 0 };
                b1 = l10 < l7;
            } else {
                b1 = 0 < l7;
            }
            cnt += 1;
            if b1 {
                let rec = sp + (l10 as u32).wrapping_mul(0x60);
                let d = m.rf(rec + 0x40);
                if d < dmax {
                    dmax = d;
                    best = obj;
                    if out != 0 {
                        for i in 0..0x18 {
                            let w = m.r32(rec + 4 * i);
                            m.w32(out + 4 * i, w);
                        }
                    }
                }
            }
            off = (cnt as u32).wrapping_mul(4);
            if !(cnt < n) {
                break;
            }
        }
        stack_pop(m, 0x1a0);
    }
    best
}

/// FUN_0014cdb0: probe every object of `objs` (n) with FUN_0014c6a0 into the 0x60-byte records of `rec`, then
/// FUN_0014c9f8 over the previous ones; returns the last FUN_0014c9f8 result
pub fn ee_sub_14cdb0<M: QMem + ?Sized>(m: &mut M, q1: V4, q2: V4, n: i32, objs: u32, p5: u32, rec: u32) -> u64 {
    let _ = q2;
    let mut u4: u64 = 0;
    let (mut n, mut objs) = (n, objs);
    if 0 < n {
        loop {
            let i3 = u4 as i32;
            let l1 = i3.wrapping_mul(0x60);
            let l5 = (l1 as u32).wrapping_add(rec);
            let o = m.r32(objs);
            m.call(0x0014_c6a0, &[o, p5.wrapping_sub(i3 as u32), l5], &[q1]);
            if l1 != 0 {
                u4 = m.call(0x0014_c9f8, &[i3 as u32, rec, l1 as u32, l5, p5, rec, 0], &[]).0 as u64;
            }
            n -= 1;
            objs += 4;
            if n == 0 {
                break;
            }
        }
    }
    u4
}

/// FUN_0014cfc0 / 0014d3b8 / 0014fc88 / 0014fa88 ...: `init(1, 0xffff)` wrappers of this run
macro_rules! wrappers {
    ($($name:ident => $target:ident),*) => { $(
        pub fn $name<M: QMem + ?Sized>(m: &mut M) { $target(m, 1, 0xffff) }
    )* };
}
wrappers!(ee_sub_14cfc0 => ee_sub_14ce88, ee_sub_14d3b8 => ee_sub_14d280, ee_sub_14fc88 => ee_sub_14fb50);

/// FUN_0014d240: EE `Count` register (CP0 cycle counter, passed in) wrap tracker: +0x58 counts wraps of
/// the 32-bit counter since the last stored value at +0x50. Returns (wraps << 32) | count.
pub fn ee_sub_14d240<M: QMem + ?Sized>(m: &mut M, p: u32, count: u32) -> u64 {
    let last = m.r64(p + 0x50);
    if (count as u64) < last {
        let v = m.r32(p + 0x58).wrapping_add(1);
        m.w32(p + 0x58, v);
    }
    let w = m.r32(p + 0x58);
    m.w32(p + 0x50, count);
    m.w32(p + 0x54, 0);
    (w as u64) << 32 | count as u64
}

/// FUN_0014d470: empty function in the ELF (`jr ra`)
pub fn ee_sub_14d470() {}

/// FUN_0014e078: for every object returned by FUN_0015e9c8(world, 0x105, 5, 0, 0x20) call FUN_002273f0(obj,
/// the quad at +0x90), qsort the list (FUN_002338c0, size 4, cmp 0x14e000), then FUN_0014e148 when the first
/// entry has +0x110 == 1
pub fn ee_sub_14e078<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let q = m.rv(o + 0x90);
    let buf = stack_push(m, 0x90);
    let n = m.call(0x0015_e9c8, &[0x003d_d490, 0x105, 5, 0, 0x20, buf], &[]).0 as i32;
    if 0 < n {
        let mut k = n;
        let mut p = buf;
        let mut cur = m.r32(buf);
        loop {
            p += 4;
            m.call(0x0022_73f0, &[cur], &[q]);
            if k - 1 == 0 {
                break;
            }
            cur = m.r32(p);
            k -= 1;
        }
    }
    m.call(0x0023_38c0, &[buf, n as u32, 4, 0x0014_e000], &[]);
    if 0 < n {
        let first = m.r32(buf);
        if m.r32(first + 0x110) == 1 {
            m.call(0x0014_e148, &[o], &[]);
        }
    }
    stack_pop(m, 0x90);
}

/// FUN_0014f930: `p2 == 0xffff`: p1 == 0 -> FUN_00258290(0x2ddf40, 2) (destroy the Cm), else Cm_new(0x2ddf40)
/// (FUN_0014d3d8) + the six static stores
pub fn ee_sub_14f930<M: QMem + ?Sized>(m: &mut M, p1: u64, p2: u64) {
    if p2 == 0xffff {
        if p1 == 0 {
            m.call(0x0025_8290, &[0x002d_df40, 2], &[]);
        } else {
            m.call(0x0014_d3d8, &[0x002d_df40], &[]);
            static_init_stores(m, 1, 0xffff);
        }
    }
}
pub fn ee_sub_14fa88<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_14f930(m, 1, 0xffff)
}
pub fn ee_sub_14faa8<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_14f930(m, 0, 0xffff)
}

/// FUN_0014fca8: C2cd570 ctor (vtable DAT_002cd570 at +0xc)
pub fn ee_sub_14fca8<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    m.w32(o + 0xc, 0x002c_d570);
    o
}
/// FUN_0014fd68: C2cd528 ctor
pub fn ee_sub_14fd68<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    ee_sub_14fca8(m, o);
    m.w32(o + 0xc, 0x002c_d528);
    o
}
/// FUN_0014fe28
pub fn ee_sub_14fe28<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.w32(o + 0x20, 0);
    m.w32(o + 0x24, 0);
    let a = m.r32(o + 0x2c);
    m.call(0x0014_f5a0, &[a, 0x002d_df40], &[]);
}
/// FUN_00151610: C2cd408 ctor
pub fn ee_sub_151610<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    ee_sub_14fca8(m, o);
    m.w32(o + 0xc, 0x002c_d408);
    o
}
/// FUN_001518c8: C2cd3c0 ctor
pub fn ee_sub_1518c8<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    ee_sub_14fca8(m, o);
    m.w32(o + 0xc, 0x002c_d3c0);
    o
}

/// FUN_00159968
pub fn ee_sub_159968<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let a = m.r32(o + 0xe4);
    let b = m.r32(a + 8);
    let c = m.r32(b);
    let u = m.call(0x001a_5bf0, &[c], &[]).0;
    m.call(0x001a_58d0, &[o, u, 0], &[]);
}

/// FUN_0015b1b0: spawn `count` FX queue entries of emitter def `d` (get/create the emitter object through the
/// cache cell p2: +4 -> cell, +8 -> flag). PARTIAL: the virtual spawn call at (entry def)+0xc is passed
/// (rand(+0xd8,+0xdc), this, [quad in v], d.+8 owner .+0x10); its real register layout is mis-decompiled.
pub fn ee_sub_15b1b0<M: QMem + ?Sized>(m: &mut M, d: u32, p2: u64, count: i32, p4: u32, p5: u32, p6: u32, p7: u32) {
    let p2 = p2 as u32;
    let mut e = 0u32;
    if p2 != 0 && m.r32(p2 + 4) != 0 {
        let c = m.r32(p2 + 4);
        e = m.r32(c);
    }
    if e == 0 {
        let k = m.r32(d + 0xc4);
        e = m.call(0x001c_dcd0, &[0x003d_d490, k], &[]).0;
        if e == 0 {
            return;
        }
        if p2 != 0 && m.r32(p2 + 4) != 0 {
            let c = m.r32(p2 + 4);
            m.w32(c, e);
        }
        m.call(0x001c_de70, &[e, d], &[]);
        if p2 != 0 && m.r32(p2 + 8) != 0 {
            m.call(0x0013_33c0, &[e], &[]);
        }
    }
    let mut i: i32 = 0;
    loop {
        let q0 = m.r32(d + 0xc0);
        let mut ent = 0;
        if i < count {
            ent = m.call(0x001c_e888, &[0x003d_d490, q0, e], &[]).0;
        }
        if i >= count || ent == 0 {
            let l = m.call(0x0011_a3b0, &[e + 0x15c], &[]).0;
            if l != 0 {
                let vt = m.r32(e + 0xc);
                let adj = m.r16(vt + 0x48) as i16 as i32 as u32;
                let f = m.r32(vt + 0x4c);
                m.call(f, &[e.wrapping_add(adj)], &[]);
            }
            return;
        }
        let def = m.r32(ent + 0xec);
        let adj = m.r16(def + 8) as i16 as i32 as u32;
        let pos = m.rv((i as u32).wrapping_mul(0x10).wrapping_add(p4));
        let (r1, r2) = (m.r32(d + 0xd8), m.r32(d + 0xdc));
        let rnd = m.call(0x0011_d448, &[r1, r2], &[]).0;
        let own = m.r32(d + 8);
        let own10 = m.r32(own + 0x10);
        let f = m.r32(def + 0xc);
        m.call(f, &[rnd, ent.wrapping_add(adj), own10], &[pos]);
        let f0 = m.rf(d + 0xe4);
        if f0 == 0.0 {
            if m.rf(d + 0xe8) != 0.0 {
                m.wf(ent + 0x24, f0);
                let w = m.r32(d + 0xe8);
                m.w32(ent + 0x28, w);
            }
        } else {
            m.wf(ent + 0x24, f0);
            let w = m.r32(d + 0xe8);
            m.w32(ent + 0x28, w);
        }
        let v5 = m.rv((i as u32).wrapping_mul(0x10).wrapping_add(p5));
        m.wv(ent + 0x90, v5);
        let pf = (i as u32).wrapping_mul(4).wrapping_add(p6);
        let a = m.rf(pf);
        m.wf(ent + 0x14, a);
        let b = m.rf((i as u32).wrapping_mul(4).wrapping_add(p7));
        m.wf(ent + 0x18, b - a);
        for k in 0..4 {
            let w = m.r32(d + 0xb0 + 4 * k);
            m.w32(ent + 0x30 + 4 * k, w);
        }
        m.call(0x0015_bf40, &[ent + 0xa0, d], &[]);
        let own = m.r32(d + 8);
        let w = m.r32(own + 8);
        m.w32(ent + 0x1c, w);
        let o8 = m.r32(d + 8);
        if m.r32(o8) != 0 {
            let src = m.call(0x001a_6318, &[], &[]).0;
            for k in 0..12 {
                let w = m.r32(src + 4 * k);
                m.w32(ent + 0x40 + 4 * k, w);
            }
        }
        i += 1;
        let def = m.r32(ent + 0xec);
        let adj = m.r16(def + 0x10) as i16 as i32 as u32;
        let f = m.r32(def + 0x14);
        m.call(f, &[ent.wrapping_add(adj), d], &[]);
    }
}

/// FUN_0015cad8: `p2 == 0xffff && p1 != 0`: initialise the camera-flag block DAT_002de5b0..0x2de5ec and the
/// 0x70-byte table at 0x2de5f0 from the constants at 0x2a2da0..0x2a2ddc, then the six static stores
pub fn ee_sub_15cad8<M: QMem + ?Sized>(m: &mut M, p1: u64, p2: u64) {
    if p2 != 0xffff || p1 == 0 {
        return;
    }
    // locals auStack_140 (64 bytes zeroed): uStack_130 = -1, rest 0
    m.w32(0x002d_e5c8, 0);
    m.w32(0x002d_e5cc, 0);
    m.w32(0x002d_e5b8, 0);
    m.w32(0x002d_e5bc, 0);
    m.w32(0x002d_e5c0, 0xffff_ffff);
    m.w32(0x002d_e5c4, 0);
    for k in 0..4 {
        let w = m.r32(0x002a_2da0 + 4 * k);
        m.w32(0x002d_e5e0 + 4 * k, w);
    }
    m.w32(0x002d_e5b0, 0);
    m.w32(0x002d_e5b4, 0);
    for (d, s) in [(0x002d_e5d0u32, 0x002a_2dd0u32), (0x002d_e5d4, 0x002a_2dd4), (0x002d_e5d8, 0x002a_2dd8), (0x002d_e5dc, 0x002a_2ddc)] {
        let w = m.r32(s);
        m.w32(d, w);
    }
    // 0x70-byte local table, memset 0 then filled
    let mut t = [0u32; 0x1c];
    t[1] = 0x443b_8000;
    t[6] = 0xc4bb_8000;
    t[8] = m.r32(0x002a_2dc0);
    t[9] = m.r32(0x002a_2dc4);
    t[10] = m.r32(0x002a_2dc8);
    t[11] = m.r32(0x002a_2dcc);
    t[13] = 200;
    t[0] = 0x443b_8000;
    t[2] = 0xbf80_0000;
    t[3] = 0x3f80_0000;
    t[14] = 0x3f80_0000;
    t[15] = 0x3f80_0000;
    t[24] = 0x3f80_0000;
    t[26] = 0xbf80_0000;
    for (i, w) in t.iter().enumerate() {
        m.w32(0x002d_e5f0 + 4 * i as u32, *w);
    }
    static_init_stores(m, 1, 0xffff);
}
pub fn ee_sub_15cd48<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_15cad8(m, 1, 0xffff)
}

/// FUN_00165280: when DAT_003e1084 bit 3 is set: run FUN_00164270 over every object of FUN_00165c98 with their
/// +0xf4/+0xf8 saved and forced (3, 0), then the same for the focus object FUN_0015e938(world, 0x105, +0x18),
/// FUN_00164df0, a message through FUN_001d8b20, FUN_001657d0 over all, DAT_003e10c8/10cc updated and flags
/// bit 3 -> bit 2
pub fn ee_sub_165280<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.r32(0x003e_1084) & 8 == 0 {
        return;
    }
    let sp = stack_push(m, 0xa0);
    let (s90, s8c, s88, s84) = (sp + 0x10, sp + 0x14, sp + 0x18, sp + 0x1c);
    let (s80, s7c, s78, s74) = (sp + 0x20, sp + 0x24, sp + 0x28, sp + 0x2c);
    let (s70, s6c, s60, s5c) = (sp + 0x30, sp + 0x34, sp + 0x40, sp + 0x44);
    m.call(0x001a_e478, &[sp], &[]);
    m.call(0x0016_5c98, &[s60, s5c], &[]);
    let n = m.r32(s60) as i32;
    let arr = m.r32(s5c);
    for i in 0..n.max(0) {
        let ob = m.r32(arr + 4 * i as u32);
        m.w32(s8c, ob);
        m.w32(s90, 0);
        if ob != 0 {
            let a = m.r32(ob + 0xf4);
            m.w32(s88, a);
            m.w32(ob + 0xf4, 3);
            let b = m.r32(ob + 0xf8);
            m.w32(s84, b);
            m.w32(ob + 0xf8, 0);
        }
        m.call(0x0016_4270, &[], &[]);
        restore(m, s90, s8c, s88, s84);
    }
    let k = m.r32(o + 0x18);
    let f = m.call(0x0015_e938, &[0x003d_d490, 0x105, k, 0], &[]).0;
    let t = m.r32(f + 0x7c);
    let t = m.r32(t + 0x18);
    m.w32(o + 0x10, t);
    m.w32(s90, 0);
    let t = m.r32(f + 0x18);
    m.w32(o + 0x14, t);
    if f != 0 {
        let a = m.r32(f + 0xf4);
        m.w32(s88, a);
        m.w32(f + 0xf4, 3);
        let b = m.r32(f + 0xf8);
        m.w32(s84, b);
        m.w32(f + 0xf8, 0);
    }
    m.w32(s8c, f);
    m.call(0x0016_4df0, &[f, 1], &[]);
    let c0 = m.r32(f + 0xc0);
    if c0 != 0xffff_ffff {
        m.w32(s70, 0x15);
        m.w32(s6c, c0);
        m.call(0x001d_8b20, &[s80, s70, 0x002a_5238, 0], &[]);
        m.call(0x0012_9ec8, &[s80], &[]);
        m.call(0x0012_9ec8, &[s70], &[]);
    }
    for i in 0..n.max(0) {
        let ob = m.r32(arr + 4 * i as u32);
        m.w32(s7c, ob);
        m.w32(s80, 0);
        if ob != 0 {
            let a = m.r32(ob + 0xf4);
            m.w32(s78, a);
            m.w32(ob + 0xf4, 3);
            let b = m.r32(ob + 0xf8);
            m.w32(s74, b);
            m.w32(ob + 0xf8, 0);
        }
        m.call(0x0016_57d0, &[ob, 0xffff_ffff], &[]);
        restore(m, s80, s7c, s78, s74);
    }
    let w = m.r32(f + 0xb8);
    m.w32(0x003e_10c8, w);
    let fl = m.r32(0x003e_1084);
    m.w32(0x003e_1084, fl & 0xffff_fff7 | 4);
    m.w32(0x003e_10cc, 0xffff_ffff);
    restore(m, s90, s8c, s88, s84);
    m.call(0x001a_e4a0, &[sp, 2], &[]);
    stack_pop(m, 0xa0);
}
/// the scope-exit restore of the saved +0xf4/+0xf8 (and +0xf0/+0xf4 of the second slot when set)
fn restore<M: QMem + ?Sized>(m: &mut M, second: u32, first: u32, sa: u32, sb: u32) {
    let s2 = m.r32(second);
    if s2 != 0 {
        let (a, b) = (m.r32(sa), m.r32(sb));
        m.w32(s2 + 0xf0, a);
        m.w32(s2 + 0xf4, b);
    }
    let s1 = m.r32(first);
    if s1 != 0 {
        let (a, b) = (m.r32(sa), m.r32(sb));
        m.w32(s1 + 0xf4, a);
        m.w32(s1 + 0xf8, b);
    }
}

/// FUN_00167e58: pack the 16-byte colour records of a palette image into 5551 words: records
/// [+0x3078 .. ) -> u16 table at +0x3084 (bit15 from the record's own alpha), records [+0x307c, +0x3080) ->
/// u16 table at +0x3088 (bit15 from the *first* cursor's alpha, original quirk), alpha bytes -> +0x3094 and,
/// when +0x3090 != 0, the first cursor's alpha -> +0x3090 (all relative to +0x3010)
pub fn ee_sub_167e58<M: QMem + ?Sized>(m: &mut M, p: u32) {
    let base = m.r32(p + 0x3010);
    let end = m.r32(p + 0x3080);
    let mut p7 = m.r32(p + 0x3094).wrapping_add(base);
    let idx = m.r32(p + 0x3078);
    let mut p5 = p.wrapping_add(idx.wrapping_mul(0x10));
    let mut p8 = m.r32(p + 0x3084).wrapping_add(base);
    let mut p4 = m.r32(p + 0x3090).wrapping_add(base);
    let mut p3 = m.r32(p + 0x3088).wrapping_add(base);
    let mut p6 = p.wrapping_add(m.r32(p + 0x307c).wrapping_mul(0x10));
    let lim = p.wrapping_add(end.wrapping_mul(0x10));
    while p6 < lim {
        let g = |m: &mut M, a: u32| m.r8(a) as u16;
        let v5 = (g(m, p5 + 4) >> 3) << 5 | (g(m, p5) >> 3) | (g(m, p5 + 8) >> 3) << 10 | (g(m, p5 + 0xc) >> 7) << 15;
        m.w16(p8, v5);
        p8 += 2;
        let v6 = (g(m, p6 + 4) >> 3) << 5 | (g(m, p6) >> 3) | (g(m, p6 + 8) >> 3) << 10 | (g(m, p5 + 0xc) >> 7) << 15;
        m.w16(p3, v6);
        let a = m.r8(p6 + 0xc);
        m.w8(p7, a);
        p7 += 1;
        if m.r32(p + 0x3090) != 0 {
            let a = m.r8(p5 + 0xc);
            m.w8(p4, a);
            p4 += 1;
        }
        p5 += 0x10;
        p3 += 2;
        p6 += 0x10;
    }
}
/// FUN_00167f78: VIF1 interrupt tail: pack the VU1-side image at 0x1100c320, return -1. PARTIAL: FlushCache,
/// VIF1_FBRST = 8, SYNC, EI are PS2 hardware ops with no engine equivalent
pub fn ee_sub_167f78<M: QMem + ?Sized>(m: &mut M) -> u32 {
    ee_sub_167e58(m, 0x1100_c320);
    0xffff_ffff
}


// ---- run_00167b68 .. run_001684d0
/// FUN_00167b68: bind the skin: per bone-group (+0x38 count, groups of 0x14 at +0x3c, 0x28 at +0x40) resolve
/// every 0x50-byte bone entry's target object (FUN_0015e938(world, 4, key, owner)), fetch its inverse bind
/// pose (vslot +0x19c) and store the 4x4 at +0x10; then link the extra list at +0x58 through FUN_0022b7a0
pub fn ee_sub_167b68<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.r32(o + 0x4c) != 0xffff_ffff {
        let t = m.r32(0x003e_11ec);
        m.w32(o + 0x4c, t);
    }
    let cnt = m.r32(o + 0x38) as i32;
    let mut g = 0i32;
    while g < cnt {
        let i10 = m.r32(o + 0x3c).wrapping_add((g * 0x14) as u32);
        let i7 = m.r32(o + 0x40).wrapping_add((g * 0x28) as u32);
        g += 1;
        let n = m.r8(i10 + 1) as i8 as i32;
        let mut j = 0i32;
        let mut off = 0u32;
        while j < n {
            let e = m.r32(i7 + 0x18).wrapping_add(off);
            let key = m.r32(e);
            let own = m.r32(o);
            let t = m.call(0x0015_e938, &[0x003d_d490, 4, key, own], &[]).0;
            m.w32(e + 4, t);
            if t != 0 {
                let vt = m.r32(t + 0xc);
                let adj = m.r16(vt + 0x198) as i16 as i32 as u32;
                let f = m.r32(vt + 0x19c);
                let a = m.r32(e + 8);
                let r = m.call(f, &[t.wrapping_add(adj), a], &[]).0;
                m.w32(e + 0xc, r);
            }
            let p3 = m.r32(e + 0xc);
            let parent = m.r32(i7 + 4);
            if p3 == 0 {
                if parent == 0 {
                    m.call(0x0023_33b4, &[e + 0x10, 0, 0x40], &[]);
                    for k in [9u32, 0x13, 0xe, 4] {
                        m.w32(e + 4 * k, 0x3f80_0000);
                    }
                } else {
                    for k in 0..16 {
                        let w = m.r32(parent + 0x40 + 4 * k);
                        m.w32(e + 0x10 + 4 * k, w);
                    }
                }
            } else {
                let sp = stack_push(m, 0x80);
                let (d, tmp) = (sp, sp + 0x40);
                if parent == 0 {
                    m.call(0x0011_c318, &[d, p3 + 0x40], &[]);
                } else {
                    m.call(0x0011_c318, &[tmp, p3 + 0x40], &[]);
                    m.call(0x0011_c168, &[d, tmp, parent + 0x40], &[]);
                }
                for k in 0..16 {
                    let w = m.r32(d + 4 * k);
                    m.w32(e + 0x10 + 4 * k, w);
                }
                stack_pop(m, 0x80);
            }
            j += 1;
            off += 0x50;
        }
    }
    let mut x = m.r32(o + 0x58);
    if x != 0 {
        loop {
            let key = m.r32(x + 8);
            let own = m.r32(o);
            let t = m.call(0x0015_e938, &[0x003d_d490, 4, key, own], &[]).0;
            if t != 0 {
                m.call(0x0022_b7a0, &[t, x], &[]);
            }
            x = m.r32(x + 0x9c);
            if x == 0 {
                break;
            }
        }
    }
}

/// FUN_00167fc0: build the 0x505 actor matrix table packet: header 1 / count*3+1, the 0x30-byte matrices
/// at +0x120 of every object of kind +0x160 == 0 (<= 0x100) in chunks of 0x20
pub fn ee_sub_167fc0<M: QMem + ?Sized>(m: &mut M, p: u32) {
    let sp = stack_push(m, 0x6c0);
    let list = sp;
    let (hdr, cur) = (sp + 0x600, sp + 0x604);
    m.call(0x0016_0918, &[p, 0x0025_e090, 0x0025_e668], &[]);
    m.call(0x0015_74b8, &[p], &[]);
    m.call(0x0022_6d28, &[p], &[]);
    m.call(0x0022_6ca8, &[p, 0x32, 0], &[]);
    let n = m.call(0x0015_e9c8, &[0x003d_d490, 0x505, 4, 0, 0x180, list], &[]).0 as i32;
    let mut cnt = 0i32;
    for i in 0..n.max(0) {
        let ob = m.r32(list + 4 * i as u32);
        if m.r32(ob + 0x160) == 0 {
            cnt += 1;
        }
    }
    if 0x100 < cnt {
        cnt = 0x100;
    }
    m.call(0x0022_6e88, &[p, 0xc, 1, 0, 0, hdr], &[]);
    let h = m.r32(hdr);
    m.w32(h, 1);
    m.w32(h + 4, (cnt * 3 + 1) as u32);
    m.w32(cur, 0);
    let (mut done, mut lim, mut k, mut idx) = (0i32, 0i32, 0i32, 1i32);
    while k < n {
        let ob = m.r32(list + 4 * k as u32);
        if m.r32(ob + 0x160) == 0 {
            if lim <= done {
                lim = 0x20;
                if cnt - done < 0x20 {
                    lim = cnt - done;
                }
                lim += done;
                m.call(0x0022_6e88, &[p, 0xc, ((lim - done) * 3) as u32, idx as u32, 0, cur], &[]);
            }
            done += 1;
            let dst = m.r32(cur);
            m.call(0x001a_e5a0, &[dst, ob + 0x120, 0x30], &[]);
            idx += 3;
            m.w32(cur, dst.wrapping_add(0x30));
            if 0xff < done {
                break;
            }
        }
        k += 1;
    }
    m.call(0x0015_76f0, &[p], &[]);
    stack_pop(m, 0x6c0);
}

/// FUN_001681b0
pub fn ee_sub_1681b0<M: QMem + ?Sized>(m: &mut M, p: u32) {
    m.call(0x0015_74b8, &[p], &[]);
    m.call(0x0022_6d28, &[p], &[]);
    m.call(0x0022_6ca8, &[p, 0x32, 0x1e7], &[]);
    m.call(0x0015_76f0, &[p], &[]);
}

/// FUN_001681f8: for every bone group with +8 set build a GIF/DMA packet at the DMA cursor DAT_002dea34:
/// tag, a 0x50-byte payload (colour word, pos+radius, 4x4 matrix) and queue it through FUN_001cd998
pub fn ee_sub_1681f8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    const CUR: u32 = 0x002d_ea34;
    let sp = stack_push(m, 0x100);
    let (c170, p160, mat) = (sp, sp + 0x10, sp + 0x20);
    let (m110, t0) = (sp + 0x60, sp + 0xa0);
    let n = m.r32(o + 0x38) as i32;
    let mut g = 0;
    while g < n {
        let i14 = m.r32(o + 0x3c).wrapping_add((g * 0x14) as u32);
        let i12 = m.r32(o + 0x40).wrapping_add((g * 0x28) as u32);
        if m.r32(i14 + 8) != 0 {
            let w = m.r32(i12 + 0x14);
            m.w32(c170, w);
            let flags = m.r32(i12 + 0xc);
            if flags & 0x1000 == 0 {
                let a = m.r32(i12 + 4);
                let par = m.r32(i12 + 8);
                m.call(0x0023_320c, &[m110, a + 0x40, 0x40], &[]);
                if par != 0 {
                    m.call(0x0011_c168, &[t0, m110, par], &[]);
                    m.call(0x0023_320c, &[m110, t0, 0x40], &[]);
                }
                for k in 0..16 {
                    let w = m.r32(m110 + 4 * k);
                    m.w32(mat + 4 * k, w);
                }
            } else {
                let a = m.r32(i12 + 0x10);
                m.call(0x001a_e5a0, &[mat, a + 0x14, 0x40], &[]);
            }
            let src = m.r32(i12 + 0x24);
            if src == 0 {
                let r = m.r32(o + 0x44).wrapping_add((g * 0x18) as u32);
                m.call(0x001a_e5a0, &[m110, r, 0xc], &[]);
                let pos = m.rv(m110);
                let (x, y, z) = (pos[0], pos[1], pos[2]);
                let (r0, r1, r2, r3) = (m.rv(mat), m.rv(mat + 0x10), m.rv(mat + 0x20), m.rv(mat + 0x30));
                let mut out = [0f32; 4];
                for k in 0..4 {
                    out[k] = r0[k] * x + r1[k] * y + r2[k] * z + r3[k];
                }
                let s = m.call(0x0011_cab0, &[mat], &[]).1[0];
                let rad = m.rf(r + 0xc);
                out[3] = s * rad;
                m.wv(p160, out);
            } else {
                for k in 0..3 {
                    let w = m.r32(src + 4 * k);
                    m.w32(p160 + 4 * k, w);
                }
                let w = m.r32(src + 0xc);
                m.w32(p160 + 0xc, w);
            }
            let p7 = m.r32(CUR);
            m.w32(p7, 0x1000_0000);
            m.w32(p7 + 4, 0);
            let c = p7 + 0x10;
            m.w32(c, 0x6c06_c301);
            let c = c + 4;
            for k in 0..20 {
                let w = m.r32(c170 + 4 * k);
                m.w32(c + 4 * k, w);
            }
            let mut c = c + 0x60;
            while c & 0xf != 0 {
                m.w32(c, 0);
                c += 4;
            }
            let tag = m.r32(p7);
            m.w32(p7, tag | ((c.wrapping_sub(p7) as i32 >> 4) - 1) as u32);
            m.w32(c, 0x6000_0000);
            m.w32(c + 4, 0);
            m.w32(CUR, c + 0x10);
            let q = m.call(0x001c_d998, &[], &[]).0;
            m.w32(q + 8, p7);
            m.w32(q, 0);
            m.w32(q + 4, 0);
            let k = m.r32(i14 + 8);
            m.w32(q + 0xc, k);
        }
        g += 1;
    }
    let t = m.r32(0x003e_11ec);
    m.w32(o + 0x4c, t);
    stack_pop(m, 0x100);
}
// FUN_001684d0 SKIPPED: the packet is filled by VU0 macro-mode micro subroutine 0x60 (_vcallms) between lqc2 / sqc2
// shuffles of 0x50-byte records; the micro program is not decoded.

/// FUN_00128138
pub fn ee_sub_128138<M: QMem + ?Sized>(m: &mut M, p: u32) -> u32 {
    m.w32(p, 0);
    p
}
/// FUN_0012b390: `init(1, 0xffff)` wrapper of FUN_0012b258
pub fn ee_sub_12b390<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_12b258(m, 1, 0xffff)
}
/// C2ca9c8_v0 (FUN_0012b4f0): empty virtual
pub fn ee_sub_12b4f0() {}
