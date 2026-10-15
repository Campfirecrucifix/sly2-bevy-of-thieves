//! WP-Q5: run_0020f948 queue package: the world proximity query FUN_0020fc60 (bucket list
//! world+0x3d1c {+0x20 centre, +0x30 radius, +0x34 objects (next +0x60), next +0x40} -> up to 256
//! hits in a 0x400-byte scratch block) and its filters (FUN_0020f948 / FUN_0020fa10 / FUN_0020fa50),
//! the unique-owner list FUN_0020fdc8, the script trigger helpers (FUN_0020fff0 / FUN_0020ff70,
//! FUN_002104a0, FUN_00210678, FUN_002108e8), the per-kind object sweep FUN_002102a0, small table
//! cursors and the HUD text formatters FUN_00210fe0..FUN_00211420 (player = FUN_001c4840(0x3dd490)
//! of kind 0x34). 1:1 from the disassembly against `QMem` (the decompiler drops the float / quad /
//! t0-t1 arguments of FUN_0020fc60, FUN_002104a0, FUN_00210678, FUN_002108e8 and FUN_00210fe0).
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};
use crate::engine::classes::q::q5util::f4;

const LOC: u32 = 0x003d_d490;

fn kind<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32) -> bool {
    m.call(0x0013_2b10, &[o, k], &[]).0 != 0
}

fn cpy<M: QMem + ?Sized>(m: &mut M, d: u32, s: u32) {
    m.call(0x0021_ed58, &[d, s], &[]);
}

/// the player when it is kind 0x34, else 0
fn player<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let p = m.call(0x001c_4840, &[LOC], &[]).0;
    if p != 0 && kind(m, p, 0x34) {
        p
    } else {
        0
    }
}

/// FUN_0020fa10: contact `c` counts for `o`: not flagged (+0x10 bit 48), its owner (c+4 -> +0xec)
/// not disabled (+0x70 bit 2) and not `o` itself
pub fn ee_sub_20fa10<M: QMem + ?Sized>(m: &mut M, o: u32, c: u32) -> bool {
    if m.r64(c + 0x10) & (1 << 48) != 0 {
        return false;
    }
    let q = m.r32(c + 4);
    let p = m.r32(q + 0xec);
    m.r64(p + 0x70) & 4 == 0 && p != o
}

/// FUN_0020f948: like FUN_0020fa10, then the two owners' +0x60 relation (slot 60 both ways,
/// FUN_00134310) must be neither 1 nor 3
pub fn ee_sub_20f948<M: QMem + ?Sized>(m: &mut M, o: u32, c: u32) -> bool {
    if !ee_sub_20fa10(m, o, c) {
        return false;
    }
    let q = m.r32(c + 4);
    let p = m.r32(q + 0xec);
    let (a, b) = (m.r32(o + 0x60), m.r32(p + 0x60));
    let x = vslot(m, a, 60, &[b], &[]).0;
    let (a, b) = (m.r32(p + 0x60), m.r32(o + 0x60));
    let y = vslot(m, a, 60, &[b], &[]).0;
    let r = m.call(0x0013_4310, &[x, y], &[]).0;
    r != 1 && r != 3
}

/// FUN_0020fa50: FUN_0020fa10 then slot 61 of the contact's +4 object
pub fn ee_sub_20fa50<M: QMem + ?Sized>(m: &mut M, o: u32, c: u32) -> u32 {
    if !ee_sub_20fa10(m, o, c) {
        return 0;
    }
    let q = m.r32(c + 4);
    vslot(m, q, 61, &[], &[]).0
}

/// FUN_0020fc60: objects of world `w` within `r` of point `p`: scratch block {0x400, 16}
/// (FUN_001ae318) -> *out_list, broadphase refresh (FUN_00133e30 / FUN_00133e80), every bucket
/// whose centre lies within r + its radius contributes its objects passing `filt(ctx, obj)` (no
/// filter: all), at most 256 -> *out_n
pub fn ee_sub_20fc60<M: QMem + ?Sized>(
    m: &mut M,
    r: f32,
    w: u32,
    p: V4,
    filt: u32,
    ctx: u32,
    out_n: u32,
    out_list: u32,
) {
    let sp = stack_push(m, 0x10);
    m.w32(sp, 0x400);
    m.w32(sp + 4, 0x10);
    let l = m.call(0x001a_e318, &[sp], &[]).0;
    stack_pop(m, 0x10);
    m.call(0x0013_3e30, &[w], &[]);
    m.call(0x0013_3e80, &[w], &[]);
    let mut n = 0u32;
    let mut b = m.r32(w + 0x3d1c);
    while b != 0 {
        let rr = r + m.rf(b + 0x30);
        let c = m.rv(b + 0x20);
        let (dx, dy, dz) = (p[0] - c[0], p[1] - c[1], p[2] - c[2]);
        if !(rr * rr < dx * dx + dy * dy + dz * dz) {
            let mut o = m.r32(b + 0x34);
            while o != 0 {
                let ok = filt == 0 || m.call(filt, &[ctx, o], &[]).0 != 0;
                if ok && n < 0x100 {
                    m.w32(l + 4 * n, o);
                    n += 1;
                }
                o = m.r32(o + 0x60);
            }
        }
        b = m.r32(b + 0x40);
    }
    m.w32(out_list, l);
    m.w32(out_n, n);
}

/// FUN_0020fdc8: the distinct owners (+4) of the `n` contacts at `cs` (deduplicated through the
/// +0x148 bit 30 mark, cleared again) -> scratch list {n * 4, 16}: *out_n, *out_list
pub fn ee_sub_20fdc8<M: QMem + ?Sized>(m: &mut M, n: u32, cs: u32, out_n: u32, out_list: u32) {
    let sp = stack_push(m, 0x10);
    m.w32(sp, n << 2);
    m.w32(sp + 4, 0x10);
    let l = m.call(0x001a_e318, &[sp], &[]).0;
    stack_pop(m, 0x10);
    let mut k = 0u32;
    for i in 0..(n as i32).max(0) as u32 {
        let c = m.r32(cs + 4 * i);
        let o = m.r32(c + 4);
        let f = m.r64(o + 0x148);
        if f & 0x4000_0000 == 0 {
            m.w32(o + 0x148, (f | 0x4000_0000) as u32);
            m.w32(l + 4 * k, o);
            k += 1;
        }
    }
    for i in 0..k {
        let o = m.r32(l + 4 * i);
        let f = m.r32(o + 0x148) & !0x4000_0000;
        m.w32(o + 0x148, f);
    }
    m.w32(out_n, k);
    m.w32(out_list, l);
}

/// FUN_0020ff70: FUN_0020fff0 for `n` records of 0x10 bytes at `t`
pub fn ee_sub_20ff70<M: QMem + ?Sized>(m: &mut M, w: u32, n: u32, t: u32, a: u32, fl: u32) {
    for i in 0..(n as i32).max(0) as u32 {
        ee_sub_20fff0(m, w, t + 0x10 * i, a, fl, 0, 0);
    }
}

/// FUN_0020fff0: record `t` {+0 name, +4 key, +8 / +0xc values}: find object `t[0]` (FUN_0015e650
/// class 0x204); a kind-0x30 one's handler for key t[1] (FUN_001f67e0) gets value t[2] (fl bit 0) or
/// t[3] (not -1): with `arg` the handler is addref'd (slot 0) into *out and bound (FUN_001333c0);
/// then started FUN_001f0948 (fl bit 1) or FUN_001f0660
pub fn ee_sub_20fff0<M: QMem + ?Sized>(m: &mut M, w: u32, t: u32, a: u32, fl: u32, arg: u32, out: u32) {
    if out != 0 {
        m.w32(out, 0);
    }
    let sp = stack_push(m, 0x10);
    let name = m.r32(t);
    let r = m.call(0x0015_e650, &[w, 0x204, name, a, 1, sp], &[]).0;
    let o = m.r32(sp);
    stack_pop(m, 0x10);
    if r != 1 || !kind(m, o, 0x30) {
        return;
    }
    let key = m.r32(t + 4);
    let h = m.call(0x001f_67e0, &[o, key], &[]).0;
    if h == 0 {
        return;
    }
    let v = m.r32(t + if fl & 1 != 0 { 8 } else { 0xc });
    if v == 0xffff_ffff {
        return;
    }
    if arg != 0 {
        m.w32(out, h);
        vslot(m, h, 0, &[], &[]);
        m.call(0x0013_33c0, &[h, arg], &[]);
    }
    m.call(if fl & 2 != 0 { 0x001f_0948 } else { 0x001f_0660 }, &[h, v], &[]);
}

/// FUN_00210100: next 0xc-byte entry of the +0x39d0 table (cursor +0x39d4)
pub fn ee_sub_210100<M: QMem + ?Sized>(m: &mut M, w: u32) -> u32 {
    let i = m.r32(w + 0x39d4);
    m.w32(w + 0x39d4, i.wrapping_add(1));
    m.r32(w + 0x39d0).wrapping_add(i.wrapping_mul(0xc))
}

/// FUN_00210120: next entry of the +0x39d8 table (cursor +0x39dc)
pub fn ee_sub_210120<M: QMem + ?Sized>(m: &mut M, w: u32) -> u32 {
    let i = m.r32(w + 0x39dc);
    m.w32(w + 0x39dc, i.wrapping_add(1));
    m.r32(w + 0x39d8).wrapping_add(i.wrapping_mul(0xc))
}

/// FUN_002102a0: FUN_001c4a60(w, 0), then slot 23 of every object of the 0xc7 per-kind lists at
/// w+0x3038 (stride 0xc, next +0x20)
pub fn ee_sub_2102a0<M: QMem + ?Sized>(m: &mut M, w: u32) {
    m.call(0x001c_4a60, &[w, 0], &[]);
    for i in 0..0xc7u32 {
        let mut o = m.r32(w + 0x3038 + i * 0xc);
        while o != 0 {
            vslot(m, o, 23, &[], &[]);
            o = m.r32(o + 0x20);
        }
    }
}

/// C2c94e0 slot 23 (FUN_00210330): empty
pub fn ee_sub_210330() {}

/// FUN_002104a0: object `id` (FUN_0015e938(w, 4, id, x); id -1 -> `obj`) of kind 0x30 gets
/// FUN_001f6ac8(o, a) when `on` else FUN_001f6b48(o, a)
pub fn ee_sub_2104a0<M: QMem + ?Sized>(m: &mut M, w: u32, on: u32, id: u32, a: u32, obj: u32, x: u32) {
    let o = if id != 0xffff_ffff { m.call(0x0015_e938, &[w, 4, id, x], &[]).0 } else { obj };
    if o == 0 || !kind(m, o, 0x30) {
        return;
    }
    m.call(if on != 0 { 0x001f_6ac8 } else { 0x001f_6b48 }, &[o, a], &[]);
}

/// FUN_00210678: seek object `id` (kind 0x30, FUN_001f80d0) and animated texture `tex` (kind 0x8e,
/// FUN_001ee3b0) to fraction `f`
pub fn ee_sub_210678<M: QMem + ?Sized>(m: &mut M, f: f32, w: u32, id: u32, tex: u32) {
    let mut o = 0;
    if id != 0xffff_ffff {
        let x = m.call(0x0015_e938, &[w, 4, id], &[]).0;
        if x != 0 && kind(m, x, 0x30) {
            o = x;
        }
    }
    let mut t = 0;
    if tex != 0xffff_ffff {
        t = crate::engine::game::q_001ec688::ee_sub_1ec918(m, tex);
        if t != 0 && !kind(m, t, 0x8e) {
            t = 0;
        }
    }
    if o != 0 {
        m.call(0x001f_80d0, &[o], &[f4(f)]);
    }
    if t != 0 {
        crate::engine::game::q_001ee100::ee_sub_1ee3b0(m, f, t);
    }
}

/// FUN_002108e8: with a handle *p whose +0x60 is `id` (or any for -1): FUN_001fb098(*p, 0.0),
/// FUN_00258040(p, 0)
pub fn ee_sub_2108e8<M: QMem + ?Sized>(m: &mut M, id: u32, p: u32) {
    let h = m.r32(p);
    if h == 0 || (id != 0xffff_ffff && m.r32(h + 0x60) != id) {
        return;
    }
    m.call(0x001f_b098, &[h], &[f4(0.0)]);
    m.call(0x0025_8040, &[p, 0], &[]);
}

/// FUN_00210940: w+0x4638 = FUN_0015e938(w, 5, 0x620, 0)
pub fn ee_sub_210940<M: QMem + ?Sized>(m: &mut M, w: u32) {
    let v = m.call(0x0015_e938, &[w, 5, 0x620, 0], &[]).0;
    m.w32(w + 0x4638, v);
}

/// FUN_002109b8: DAT_003e1c4c + i * 8
pub fn ee_sub_2109b8<M: QMem + ?Sized>(m: &mut M, i: u32) -> u32 {
    m.r32(0x003e_1c4c).wrapping_add(i.wrapping_mul(8))
}

/// FUN_00210b28: the save value "episodeBest" (key 0x2a8a78 via FUN_001d8a38, value +4, released
/// FUN_00129ec8)
pub fn ee_sub_210b28<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let sp = stack_push(m, 0x10);
    m.call(0x001d_8a38, &[sp, 0x002a_8a78, 0], &[]);
    let v = m.r32(sp + 4);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x10);
    v
}

/// FUN_00210b68: 0
pub fn ee_sub_210b68() -> u32 {
    0
}

/// FUN_00210eb0: strcpy(d, s)
pub fn ee_sub_210eb0<M: QMem + ?Sized>(m: &mut M, d: u32, s: u32) {
    cpy(m, d, s)
}

/// FUN_00210fe0: the player's gadget level text: v = +0xf3c * (DAT_002de3b8 when +0xf20 == 0,
/// DAT_002de514 when 1, else 0); kind 0x35 -> sprintf(0x2a8ab8, int v); kind 0x37 ->
/// sprintf(0x2a8ac0, FUN_0023e290(v)); kind 0x36 -> 0x2a8ac8 / 0x2a8ad0 / 0x2a8ad8 by int v < 4 / < 8
pub fn ee_sub_210fe0<M: QMem + ?Sized>(m: &mut M, d: u32) {
    let p = player(m);
    if p == 0 {
        return;
    }
    let v = match m.r32(p + 0xf20) {
        0 => m.rf(0x002d_e3b8) * m.rf(p + 0xf3c),
        1 => m.rf(0x002d_e514) * m.rf(p + 0xf3c),
        _ => 0.0,
    };
    let iv = v as i32;
    if kind(m, p, 0x35) {
        m.call(0x0021_ecd8, &[d, 0x002a_8ab8, iv as u32], &[]);
    } else if kind(m, p, 0x37) {
        let x = m.call(0x0023_e290, &[], &[f4(v)]).0;
        m.call(0x0021_ecd8, &[d, 0x002a_8ac0, x], &[]);
    } else if kind(m, p, 0x36) {
        let f = if iv < 4 {
            0x002a_8ac8
        } else if iv < 8 {
            0x002a_8ad0
        } else {
            0x002a_8ad8
        };
        m.call(0x0021_ecd8, &[d, f, iv as u32], &[]);
    }
}

/// FUN_00211148: the player's gadget (+0xf38) name (k 0: FUN_00202730, 1: FUN_002026f8) into `d`
pub fn ee_sub_211148<M: QMem + ?Sized>(m: &mut M, d: u32, k: u32) {
    let p = player(m);
    if p == 0 {
        return;
    }
    let g = m.r32(p + 0xf38);
    let s = match (g, k) {
        (0, _) => 0,
        (_, 0) => m.call(0x0020_2730, &[g, 0], &[]).0,
        (_, 1) => m.call(0x0020_26f8, &[g, 0], &[]).0,
        _ => 0,
    };
    if s != 0 {
        cpy(m, d, s);
    }
}

/// FUN_002111f0: localised name (+0xf8) of entry `i` (< +0x74, list +0x78) of the player's gadget
pub fn ee_sub_2111f0<M: QMem + ?Sized>(m: &mut M, d: u32, i: u32) {
    let p = player(m);
    if p == 0 {
        return;
    }
    let g = m.r32(p + 0xf38);
    if g == 0 || (i as i32) >= m.r32(g + 0x74) as i32 {
        return;
    }
    let e = m.r32(g + i.wrapping_mul(4) + 0x78);
    let id = m.r32(e + 0xf8);
    let s = m.call(0x0013_e368, &[LOC, id], &[]).0;
    cpy(m, d, s);
}

/// FUN_00211348: localised string o[+0xcc + 4i] into `d`
pub fn ee_sub_211348<M: QMem + ?Sized>(m: &mut M, d: u32, i: u32, o: u32) {
    let id = m.r32(o.wrapping_add(i.wrapping_mul(4)) + 0xcc);
    let s = m.call(0x0013_e368, &[LOC, id], &[]).0;
    cpy(m, d, s);
}

/// FUN_00211420: name of what the player is carrying (+0x9a0 while +0x4c8 == 0xc): kind 0x5e ->
/// loot name 0x2c328c[+0x244 -> +4], kind 0x5c -> its +0x224, else the player's +0x1318; empty
/// when none
pub fn ee_sub_211420<M: QMem + ?Sized>(m: &mut M, d: u32) {
    let mut id = 0xffff_ffff;
    let p = player(m);
    if p != 0 {
        let c = m.r32(p + 0x9a0);
        if c != 0 && m.r32(p + 0x4c8) == 0xc {
            id = if kind(m, c, 0x5e) {
                let e = m.r32(c + 0x244);
                let k = m.r32(e + 4);
                m.r32(k.wrapping_mul(0xc).wrapping_add(0x002c_328c))
            } else if kind(m, c, 0x5c) {
                m.r32(c + 0x224)
            } else {
                m.r32(p + 0x1318)
            };
        }
    }
    if id == 0xffff_ffff {
        m.w8(d, 0);
    } else {
        let s = m.call(0x0013_e368, &[LOC, id], &[]).0;
        cpy(m, d, s);
    }
}

/// FUN_00210e70: TU static init thunk FUN_00210cf0(1, 0xffff) (string table, ui::strings)
pub fn ee_sub_210e70<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x0021_0cf0, &[1, 0xffff], &[]);
}

/// FUN_00210e90: TU static destroy thunk FUN_00210cf0(0, 0xffff)
pub fn ee_sub_210e90<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x0021_0cf0, &[0, 0xffff], &[]);
}
