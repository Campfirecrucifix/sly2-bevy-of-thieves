//! WP-Q4: run 0x001d4b28..0x001d6828 - state stamps, the 0x574 slot table accessors, the light / lod tree
//! (world +0x3e5c: 0x80-byte nodes of four 0x20-byte entries {+0 pos, +0xc radius, +0x10 range (f32) with
//! u64 bits 32..35 state (signed: -1 free, 0 mesh, 1 child node, 3 / 4 ...), 36..43 mask, 44..49 kind, 50 skip,
//! 51..63 mesh index, +0x18 child / payload}, next node +0x7c), its render walk FUN_001d5808 over the
//! 256-entry node stack (FUN_001d57a8 / 57b8 / 57e0) and the light attach / detach of an ALO (FUN_001d6310 /
//! 1d6378 / 1d6508).
//! FUN_001d5808 writes DMA / VIF words into the world packet 0x2dea28 (game logic, ported 1:1); its VU0
//! micro program (`vcallms 0x1c0`) is the hook `VU0_MS_1C0` (see its doc).
//! 1:1 against `QMem` (classes::q::q2mem); callees outside the run go through `QMem::call` by address.
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};
use crate::engine::game::q_001e9090::ee_sub_1ea410;

/// DAT_002dded0: game clock
pub const NOW: u32 = 0x002d_ded0;
/// world packet (FUN_00157278 layout), cursor +0xc = DAT_002dea34
pub const PKT: u32 = 0x002d_ea28;
/// DAT_003d4a50: render record cursor (16-byte records from 0x3b3980, 0x2000 of them)
pub const REC_CUR: u32 = 0x003d_4a50;
/// DAT_002a7870: initial best score of FUN_001d6508
pub const BEST0: u32 = 0x002a_7870;
/// pseudo address of the VU0 micro program at micro address 0x1c0 (`vcallms 0x1c0` in FUN_001d5808):
/// v = [vf14 = (DAT_002ddf5c, 0, 0, 0), vf15 = entry pos, vf16 = (radius, ..), vf17 = (range, ..),
/// vf10..vf12 = 0x2ddfe0 / 0x2ddff0 / 0x2de000, vf13 = 0x2ddfd0]; returns (vf19.x bits, vf18)
pub const VU0_MS_1C0: u32 = 0xf000_01c0;

/// FUN_001d4b28(o, s): +0x188 changes -> +0x188 = s, +0x18c = now
pub fn ee_sub_1d4b28<M: QMem + ?Sized>(m: &mut M, o: u32, s: u32) {
    if s != m.r32(o + 0x188) {
        m.w32(o + 0x188, s);
        let t = m.r32(NOW);
        m.w32(o + 0x18c, t);
    }
}

/// FUN_001d4ce0(o, i, v, w): slot i (o +0x574 + 12 i): +8 = v, +4 = w
/// VFIX: ABI per asm: o = a0, i = a1, v = $f12 (swc1), w = a2 (Ghidra's param order was wrong)
pub fn ee_sub_1d4ce0<M: QMem + ?Sized>(m: &mut M, o: u32, i: u32, v: f32, w: u32) {
    let s = o.wrapping_add(i.wrapping_mul(0xc)).wrapping_add(0x574);
    m.wf(s + 8, v);
    m.w32(s + 4, w);
}

/// FUN_001d4d00(o, i): slot i +0
pub fn ee_sub_1d4d00<M: QMem + ?Sized>(m: &mut M, o: u32, i: u32) -> u32 {
    m.r32(i.wrapping_mul(0xc).wrapping_add(o).wrapping_add(0x574))
}

/// FUN_001d4d18(o, i, v): slot i +0 = v
pub fn ee_sub_1d4d18<M: QMem + ?Sized>(m: &mut M, o: u32, i: u32, v: u32) {
    m.w32(i.wrapping_mul(0xc).wrapping_add(o).wrapping_add(0x574), v);
}

/// FUN_001d4fa0(o, s): +0x150 changes -> +0x150 = s, +0x154 = now; with a sound +0x164 and s != -1
/// FUN_001f0730(+0x164, DAT_002c2248[s])
pub fn ee_sub_1d4fa0<M: QMem + ?Sized>(m: &mut M, o: u32, s: u32) {
    if s == m.r32(o + 0x150) {
        return;
    }
    m.w32(o + 0x150, s);
    let t = m.r32(NOW);
    m.w32(o + 0x154, t);
    let h = m.r32(o + 0x164);
    if h != 0 && s != u32::MAX {
        let v = m.r32(s.wrapping_mul(4).wrapping_add(0x002c_2248));
        m.call(0x001f_0730, &[h, v], &[]);
    }
}

/// FUN_001d4ff8 / 1d5650 / 1d66f0 (identical static pool inits) = FUN_001ea410
pub fn ee_sub_1d4ff8<M: QMem + ?Sized>(m: &mut M, init: u32, prio: u32) {
    ee_sub_1ea410(m, init, prio);
}
/// FUN_001d5130 / 1d5788 / 1d6828: (1, 0xffff)
pub fn ee_sub_1d5130<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_1ea410(m, 1, 0xffff);
}

/// FUN_001d5498: DAT_002c2d8c = 1 when 0
pub fn ee_sub_1d5498<M: QMem + ?Sized>(m: &mut M) {
    if m.r32(0x002c_2d8c) == 0 {
        m.w32(0x002c_2d8c, 1);
    }
}

/// FUN_001d57a8(stack): count = 0
pub fn ee_sub_1d57a8<M: QMem + ?Sized>(m: &mut M, s: u32) -> u32 {
    m.w32(s, 0);
    s
}

/// FUN_001d57b8(stack, v): push (up to 256)
pub fn ee_sub_1d57b8<M: QMem + ?Sized>(m: &mut M, s: u32, v: u32) {
    let n = m.r32(s);
    if n < 0x100 {
        m.w32(s + 4 * (n + 1), v);
        m.w32(s, n + 1);
    }
}

/// FUN_001d57e0(stack): pop, 0 when empty
pub fn ee_sub_1d57e0<M: QMem + ?Sized>(m: &mut M, s: u32) -> u32 {
    let n = m.r32(s);
    if n == 0 {
        return 0;
    }
    m.w32(s, n - 1);
    m.r32(s + 4 * n)
}

fn state(e: u64) -> i64 {
    ((e << 0x1c) as i64) >> 0x3c
}
fn kind(e: u64) -> i64 {
    ((e << 0xe) as i64) >> 0x3a
}
fn mask(e: u64) -> i64 {
    ((e << 0x14) as i64) >> 0x38
}
fn len2(v: V4) -> f32 {
    v[0] * v[0] + v[1] * v[1] + v[2] * v[2]
}
fn cur<M: QMem + ?Sized>(m: &mut M) -> u32 {
    m.r32(PKT + 0xc)
}
fn adv<M: QMem + ?Sized>(m: &mut M, n: u32) -> u32 {
    let c = m.r32(PKT + 0xc);
    m.w32(PKT + 0xc, c.wrapping_add(n));
    c
}
fn copy_q<M: QMem + ?Sized>(m: &mut M, d: u32, s: u32, n: u32) {
    for k in 0..n * 4 {
        let v = m.r32(s + 4 * k);
        m.w32(d + 4 * k, v);
    }
}
fn sd<M: QMem + ?Sized>(m: &mut M, a: u32, v: u64) {
    m.w32(a, v as u32);
    m.w32(a + 4, (v >> 32) as u32);
}
fn pad16<M: QMem + ?Sized>(m: &mut M) {
    while cur(m) & 0xf != 0 {
        let c = adv(m, 4);
        m.w32(c, 0);
    }
}
/// new render record (cursor below 0x3b3980 + 0x20000), None when full
fn new_rec<M: QMem + ?Sized>(m: &mut M) -> Option<u32> {
    let a = m.r32(REC_CUR);
    if ((a.wrapping_sub(0x003b_3980) as i32) >> 4) as u32 >= 0x2000 {
        return None;
    }
    m.w32(REC_CUR, a + 0x10);
    Some(a)
}

/// FUN_001d5808(stack, layer): render walk of a layer's light / lod tree. Slot 7 of every object in
/// +0x28 (count +0x24) passing FUN_001eda20; FUN_001681f8(layer) when +0x4c != DAT_003e11ec; then the tree
/// from +0x48 (node stack `stack`): every used entry whose mask meets DAT_002ddf54 and whose VU0 test
/// (VU0_MS_1C0) gives a non-zero factor f: state 1 pushes the child, 4 defers it (up to 512, slot 18 at
/// the end), 3 / 0 append a render record {kind (5 / 6 for kinds 3 / 4 when f < 1), distance^2 (or mesh
/// +4 for flag 0x400), dma, arg}: state 3 = ref of the child's packet (f >= 1) or a call wrapper with the
/// {f, 1, layer +0x50, 0} row; state 0 = mesh (layer +0x40 + 0x28 i, +0x3c + 0x14 i) packet: its matrix
/// (parent +8 * +4 +0x40), skin rows (+0x1c: inverse * bind 0x6f0 / 0x7f0 rows), the row, kinds 7 / 8 the
/// in-range flag (distance < radius + DAT_002ddf88), closed with a ret tag.
pub fn ee_sub_1d5808<M: QMem + ?Sized>(m: &mut M, stk: u32, layer: u32) {
    let n = m.r32(layer + 0x24) as i32;
    for i in 0..n.max(0) as u32 {
        let b = m.r32(layer + 0x28);
        let o = m.r32(b + 4 * i);
        if m.call(0x001e_da20, &[o], &[]).0 != 0 {
            vslot(m, o, 7, &[], &[]);
        }
    }
    if m.r32(layer + 0x4c) != m.r32(0x003e_11ec) {
        m.call(0x0016_81f8, &[layer], &[]);
    }
    let root = m.r32(layer + 0x48);
    ee_sub_1d57b8(m, stk, root);
    let sp = stack_push(m, 0x8e0);
    let (row, m10, m40, m50, m80, flag) = (sp + 0x800, sp + 0x810, sp + 0x840, sp + 0x850, sp + 0x880, sp + 0x8d0);
    let defer = sp;
    let mut nd: u32 = 0;
    let msk = m.r32(0x002d_df54) as i32 as i64;
    loop {
        let node = ee_sub_1d57e0(m, stk);
        if node == 0 {
            break;
        }
        for s1 in 0..4u32 {
            let e = node + 0x20 * s1;
            if state(m.r64(e + 0x10)) == -1 {
                continue;
            }
            let vf = [
                [f32::from_bits(m.r32(0x002d_df5c)), 0.0, 0.0, 0.0],
                m.rv(e),
                [m.rf(e + 0xc), 0.0, 0.0, 0.0],
                [m.rf(e + 0x10), 0.0, 0.0, 0.0],
                m.rv(0x002d_dfe0),
                m.rv(0x002d_dff0),
                m.rv(0x002d_e000),
                m.rv(0x002d_dfd0),
            ];
            let (f19, vf3) = m.call(VU0_MS_1C0, &[], &vf);
            let ev = m.r64(e + 0x10);
            if mask(ev) & msk == 0 {
                continue;
            }
            let f = f32::from_bits(f19);
            if f == 0.0 {
                continue;
            }
            match state(ev) {
                1 => {
                    let c = m.r32(e + 0x18);
                    ee_sub_1d57b8(m, stk, c);
                }
                4 => {
                    if nd < 0x200 {
                        let c = m.r32(e + 0x18);
                        m.w32(defer + 4 * nd, c);
                        nd += 1;
                    }
                }
                3 => {
                    if ev & 0x4_0000_0000_0000 != 0 {
                        continue;
                    }
                    let Some(r) = new_rec(m) else { continue };
                    let k = kind(ev);
                    m.w32(r, k as u32);
                    m.wf(r + 4, len2(vf3));
                    if f < 1.0 {
                        if k == 3 {
                            m.w32(r, 5);
                        } else if k == 4 {
                            m.w32(r, 6);
                        }
                    }
                    if 1.0 <= f {
                        let c = m.r32(e + 0x18);
                        m.w32(r + 0xc, 0);
                        m.w32(r + 8, c);
                        continue;
                    }
                    let c0 = cur(m);
                    m.w32(r + 8, c0);
                    let ch = m.r32(e + 0x18);
                    m.w32(r + 0xc, ch.wrapping_add(0x70));
                    sd(m, c0, (ch.wrapping_add(0x10) as u64) << 32 | 0x4000_0006);
                    m.wf(row, f);
                    m.wf(row + 4, 1.0);
                    m.w32(row + 0xc, 0);
                    let a = adv(m, 0x10);
                    let w = m.r32(layer + 0x50);
                    m.w32(row + 8, w);
                    sd(m, a + 0x10, 0x1000_0002);
                    let a = adv(m, 0x10);
                    m.w32(a + 0x10, 0x6c01_c008);
                    adv(m, 4);
                    let d = cur(m);
                    copy_q(m, d, row, 1);
                    adv(m, 0x10);
                    pad16(m);
                    let c = adv(m, 0x10);
                    sd(m, c, 0x6000_0000);
                }
                0 => {
                    let u = (ev >> 51) as u32;
                    let s5 = m.r32(layer + 0x40).wrapping_add(u.wrapping_mul(0x28));
                    let a1 = m.r32(layer + 0x3c).wrapping_add(u.wrapping_mul(0x14));
                    if ev & 0x4_0000_0000_0000 != 0 {
                        continue;
                    }
                    let Some(r) = new_rec(m) else { continue };
                    m.w32(r, kind(ev) as u32);
                    if f < 1.0 {
                        let k = kind(m.r64(e + 0x10));
                        if k == 3 {
                            m.w32(r, 5);
                        } else if k == 4 {
                            m.w32(r, 6);
                        }
                    }
                    let d2 = if m.r32(s5 + 0xc) & 0x400 == 0 { len2(vf3) } else { m.rf(a1 + 4) };
                    m.wf(r + 4, d2);
                    let fp = cur(m);
                    sd(m, fp, 0x1000_0000);
                    adv(m, 0x10);
                    let par = m.r32(s5 + 8);
                    if par != 0 {
                        let b = m.r32(s5 + 4) + 0x40;
                        m.call(0x0011_c168, &[m50, par, b], &[]);
                        copy_q(m, m10, m50, 4);
                        let c = adv(m, 4);
                        m.w32(c, 0x6c04_c000);
                        let d = cur(m);
                        copy_q(m, d, m10, 4);
                    } else {
                        let c = cur(m);
                        m.w32(c, 0x6c04_c000);
                        let b = m.r32(s5 + 4) + 0x40;
                        adv(m, 4);
                        let d = cur(m);
                        copy_q(m, d, b, 4);
                    }
                    adv(m, 0x40);
                    let s2 = m.r32(s5 + 0x1c);
                    if s2 != 0 {
                        let par = m.r32(s5 + 8);
                        if par != 0 {
                            let b = m.r32(s5 + 4) + 0x40;
                            m.call(0x0011_c168, &[sp + 0x890, par, b], &[]);
                            m.call(0x0011_c318, &[m40, sp + 0x890], &[]);
                        } else {
                            let b = m.r32(s5 + 4) + 0x40;
                            m.call(0x0011_c318, &[m40, b], &[]);
                        }
                        copy_q(m, row, m40, 4);
                        let mut s0 = m.r32(s2 + 0x20);
                        let mut i = 0;
                        while i < m.r32(s2 + 0x10) as i32 {
                            if m.r32(s2 + 0x14) != 0 {
                                let b = m.r32(s2 + 0xc) + (i as u32) * 0x40 + 0x6f0;
                                m.call(0x0011_c168, &[m80, row, b], &[]);
                                copy_q(m, m40, m80, 4);
                                let c = adv(m, 4);
                                m.w32(c, s0 | 0x6c04_c000);
                                let d = cur(m);
                                copy_q(m, d, m40, 4);
                                s0 += 4;
                                adv(m, 0x40);
                            }
                            if m.r32(s2 + 0x18) != 0 {
                                let b = m.r32(s2 + 0xc) + (i as u32) * 0x40 + 0x7f0;
                                let c = adv(m, 4);
                                m.w32(c, s0 | 0x6c04_c000);
                                let d = cur(m);
                                copy_q(m, d, b, 4);
                                s0 += 4;
                                adv(m, 0x40);
                            }
                            i += 1;
                        }
                    }
                    m.wf(row, f);
                    let w = m.r32(layer + 0x50);
                    m.w32(row + 8, w);
                    m.wf(row + 4, 1.0);
                    m.w32(row + 0xc, 0);
                    let c = adv(m, 4);
                    m.w32(c, 0x6c01_c008);
                    let d = cur(m);
                    copy_q(m, d, row, 1);
                    adv(m, 0x10);
                    let k = kind(m.r64(e + 0x10));
                    if (((k as i32) - 7) as u32) < 2 {
                        let rr = m.rf(e + 0xc) + m.rf(0x002d_df88);
                        m.w32(flag, (len2(vf3) < rr * rr) as u32);
                        let c = adv(m, 4);
                        m.w32(c, 0x6001_c012);
                        let v = m.r32(flag);
                        m.w32(c + 4, v);
                        adv(m, 4);
                    }
                    pad16(m);
                    let c = cur(m);
                    let t = m.r32(fp) | (((c.wrapping_sub(fp) as i32) >> 4) - 1) as u32;
                    m.w32(fp, t);
                    let c = adv(m, 0x10);
                    sd(m, c, 0x6000_0000);
                    m.w32(r + 8, fp);
                    let v = m.r32(s5 + 0x10);
                    m.w32(r + 0xc, v);
                }
                _ => {}
            }
        }
        let nx = m.r32(node + 0x7c);
        if nx != 0 {
            ee_sub_1d57b8(m, stk, nx);
        }
    }
    for i in 0..nd {
        let o = m.r32(defer + 4 * i);
        vslot(m, o, 18, &[], &[]);
    }
    stack_pop(m, 0x8e0);
}

/// FUN_001d6310(o): detach the light +0x144: its state bits = 0xf (free), +0x140 = -1, +0x144 = 0,
/// clear bit 7 of the flags +0x40 (inline when tagged odd, else through the pointer)
pub fn ee_sub_1d6310<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let l = m.r32(o + 0x144);
    if l == 0 {
        return;
    }
    let h = m.r32(l + 0x14);
    m.w32(l + 0x14, h | 0xf);
    m.w32(o + 0x140, u32::MAX);
    let p = m.r32(o + 0x40);
    m.w32(o + 0x144, 0);
    if (p ^ 1) & 1 != 0 {
        let v = m.r32(p) & 0xffff_ff7f;
        m.w32(p, v);
    } else {
        m.w32(o + 0x40, p & 0xffff_ff7f);
    }
}

/// FUN_001d6378(o): attach a light: position = slot-30 matrix * o +0x90 (w = row 0 w), radius =
/// FUN_0011cab0(matrix) * +0xa0, FUN_001d6508(world 0x3dd490, radius, +0x38, pos, byte +0x2d) -> +0x144;
/// found: state 4, +0x18 = o, flags +0x40 bit 7, +0x140 = (+0x60) +0xa4
pub fn ee_sub_1d6378<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let mp = vslot(m, o, 30, &[], &[]).0;
    let mt = m.rm(mp);
    let sp = stack_push(m, 0x50);
    m.wm(sp, mt);
    let p = m.rv(o + 0x90);
    let pos: V4 = [
        mt[0][0] * p[0] + mt[1][0] * p[1] + mt[2][0] * p[2] + mt[3][0],
        mt[0][1] * p[0] + mt[1][1] * p[1] + mt[2][1] * p[2] + mt[3][1],
        mt[0][2] * p[0] + mt[1][2] * p[1] + mt[2][2] * p[2] + mt[3][2],
        mt[0][3],
    ];
    m.wv(sp + 0x40, pos);
    let s = m.call(0x0011_cab0, &[sp], &[]).1[0];
    stack_pop(m, 0x50);
    let rad = s * m.rf(o + 0xa0);
    let r2 = m.rf(o + 0x38);
    let mk = m.r8(o + 0x2d) as u32;
    let l = ee_sub_1d6508(m, rad, r2, 0x003d_d490, pos, mk);
    m.w32(o + 0x144, l);
    if l == 0 {
        return;
    }
    let h = m.r32(l + 0x14);
    m.w32(l + 0x14, (h & 0xffff_fff0) | 4);
    let l = m.r32(o + 0x144);
    m.w32(l + 0x18, o);
    let p = m.r32(o + 0x40);
    if (p ^ 1) & 1 == 0 {
        m.w32(o + 0x40, p | 0x80);
    } else {
        let v = m.r32(p) | 0x80;
        m.w32(p, v);
    }
    let a = m.r32(o + 0x60);
    let v = m.r32(a + 0xa4);
    m.w32(o + 0x140, v);
}

/// FUN_001d6480: new tree node (0x80 bytes, FUN_001ae2a0) with all four entries free (state 0xf)
pub fn ee_sub_1d6480<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let s = stack_push(m, 0x10);
    m.w32(s, 0x80);
    m.w32(s + 4, 0x10);
    let n = m.call(0x001a_e2a0, &[s], &[]).0;
    stack_pop(m, 0x10);
    for off in [0x74, 0x54, 0x34, 0x14] {
        let h = m.r32(n + off);
        m.w32(n + off, h | 0xf);
    }
    n
}

/// FUN_001d6508(r, r2, world, pos, mk): find the light slot for a sphere (pos, r) in the tree world
/// +0x3e5c: descend through the closest state-1 entry containing it (distance + r <= radius, range 1e10
/// or distance + r2 <= range, mask covering mk; score radius * min(range, 1e5)^2 below DAT_002a7870,
/// best = lowest); at the leaf level take the first free entry (or a new node FUN_001d6480 appended at
/// +0x7c of the last node): pos, +0xc = r, +0x10 = r2, mask bits 36..43 = mk. 0 without a tree.
pub fn ee_sub_1d6508<M: QMem + ?Sized>(m: &mut M, r: f32, r2: f32, w: u32, pos: V4, mk: u32) -> u32 {
    let mut s0 = m.r32(w + 0x3e5c);
    if s0 == 0 {
        return 0;
    }
    let a2 = mk as u64;
    let mut a3;
    loop {
        let mut best = m.rf(BEST0);
        let mut t2 = 0u32;
        a3 = 0u32;
        loop {
            let t0 = m.r32(s0 + 0x7c);
            for i in 0..4 {
                let e = s0 + 0x20 * i;
                let ev = m.r64(e + 0x10);
                let st = state(ev);
                if st == -1 {
                    if a3 == 0 {
                        a3 = e;
                    }
                    continue;
                }
                if st != 1 {
                    continue;
                }
                let c = m.rv(e);
                let d = len2([pos[0] - c[0], pos[1] - c[1], pos[2] - c[2], 0.0]).sqrt();
                let rad = m.rf(e + 0xc);
                if !(d + r <= rad) {
                    continue;
                }
                let rg = m.rf(e + 0x10);
                if !(rg == 1e10 || d + r2 <= rg) {
                    continue;
                }
                if a2 & !(mask(ev) as u64) != 0 {
                    continue;
                }
                let g = rg.min(100000.0);
                let sc = rad * g * g;
                if sc < best {
                    best = sc;
                    t2 = m.r32(e + 0x18);
                }
            }
            if t0 == 0 {
                break;
            }
            s0 = t0;
        }
        if t2 == 0 {
            break;
        }
        s0 = t2;
    }
    if a3 == 0 {
        a3 = ee_sub_1d6480(m);
        m.w32(s0 + 0x7c, a3);
    }
    m.wf(a3 + 0x10, r2);
    m.wv(a3, pos);
    let v = m.r64(a3 + 0x10);
    m.wf(a3 + 0xc, r);
    let v = (v & 0xffff_f00f_ffff_ffff) | (a2 & 0xff) << 36;
    sd(m, a3 + 0x10, v);
    a3
}
