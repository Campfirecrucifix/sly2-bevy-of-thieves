//! WP-Q5: run_00166008 queue package: the GIF packet builder (packet object: +0xc cursor, +0x18 vtable
//! 0x2cf508 whose slot +0xc (this adjust i16 at +8) reserves space, +0x1c open GIF tag, +0x20 NREG,
//! +0x24 loop count): tag open / close and the PACKED (A+D, RGBAQ, ST, UV, XYZ2, XYZF2) and REGLIST
//! emitters, plus the save-buffer restore FUN_00166008, the BSP edge query FUN_001665f0 and the
//! +0x4c reset FUN_00167310. 1:1 against `QMem`. 64-bit register arguments are `u64` here; callers
//! routing through `QMem::call` pass them as (lo, hi) pairs.
//! SKIPPED: FUN_00166428 (batch transform through the VU0 micro subroutines 0x120 / 0xf0 of the
//! microcode uploaded from 0x259df0; the micro program is not decoded).
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem};

fn w64<M: QMem + ?Sized>(m: &mut M, a: u32, v: u64) {
    m.w32(a, v as u32);
    m.w32(a + 4, (v >> 32) as u32);
}

/// vtable slot +0xc of the packet (reserve `n`): (*(vt+0xc))(p + *(i16*)(vt+8), n)
fn reserve<M: QMem + ?Sized>(m: &mut M, p: u32, n: u32) {
    let vt = m.r32(p + 0x18);
    let adj = m.r16(vt + 8) as i16 as i32 as u32;
    let f = m.r32(vt + 0xc);
    m.call(f, &[p.wrapping_add(adj), n], &[]);
}

/// take `n` bytes at the cursor
fn take<M: QMem + ?Sized>(m: &mut M, p: u32, n: u32) -> u32 {
    let c = m.r32(p + 0xc);
    m.w32(p + 0xc, c + n);
    c
}

/// FUN_00166008: restore the pending save image DAT_002deb4c (0x5408 bytes) into 0x3d4a60, clear it,
/// FUN_001d54b8, FUN_001d8ff8(0x3d4a60)
pub fn ee_sub_166008<M: QMem + ?Sized>(m: &mut M) {
    let src = m.r32(0x002d_eb4c);
    if src == 0 {
        return;
    }
    for i in 0..0x5408u32 {
        let b = m.r8(src + i);
        m.w8(0x003d_4a60 + i, b);
    }
    m.w32(0x002d_eb4c, 0);
    m.call(0x001d_54b8, &[], &[]);
    m.call(0x001d_8ff8, &[0x003d_4a60], &[]);
}

/// FUN_001665f0: collect into `out` (at most `max`) the distinct edges {u16 a, u16 b, .., next +8} of the
/// BSP leaves around vertex `v` of mesh `mesh` (+4 vertex quads) that touch `v`: PruneBsp (FUN_00118a88,
/// radius 1.0) over `bsp`+0xc into a frame-pool array of min(bsp+4, 0x1555) 12-byte hits; 0 when the
/// query does not return 2
pub fn ee_sub_1665f0<M: QMem + ?Sized>(m: &mut M, mesh: u32, bsp: u32, v: u32, max: i32, out: u32) -> i32 {
    let sp = stack_push(m, 0x30);
    m.call(0x001a_e478, &[sp], &[]);
    let n = m.r32(bsp + 4) as i32;
    let cnt = if 0x1554 < n { 0x1555 } else { n };
    m.w32(sp + 0x10, (cnt * 0xc) as u32);
    m.w32(sp + 0x14, 0x10);
    m.w32(sp + 0x20, cnt as u32);
    let hits = m.call(0x001a_e318, &[sp + 0x10], &[]).0;
    m.w32(sp + 0x24, hits);
    let verts = m.r32(mesh + 4);
    let q = m.rv(verts + v * 0x10);
    let root = m.r32(bsp + 0xc);
    let r = m.call(0x0011_8a88, &[root, 0x7fff_ffff, sp + 0x20, sp + 0x24], &[q, [1.0, 0.0, 0.0, 0.0]]).0;
    let mut k = 0i32;
    if r == 2 {
        let nh = (m.r32(sp + 0x24).wrapping_sub(hits) as i32) / 0xc;
        for i in 0..nh.max(0) as u32 {
            let leaf = m.r32(hits + i * 0xc + 8);
            let mut e = m.r32(leaf + 0x10);
            while e != 0 {
                if v == m.r16(e) as u32 || v == m.r16(e + 2) as u32 {
                    let seen = (0..k as u32).any(|j| m.r32(out + 4 * j) == e);
                    if !seen && k < max {
                        m.w32(out + 4 * k as u32, e);
                        k += 1;
                    }
                }
                e = m.r32(e + 8);
            }
        }
    }
    m.call(0x001a_e4a0, &[sp, 2], &[]);
    stack_pop(m, 0x30);
    k
}

/// FUN_00166920: packet constructor: base FUN_00157278, +0x14 = 1, vtable 0x2cf508, no open tag
pub fn ee_sub_166920<M: QMem + ?Sized>(m: &mut M, p: u32) -> u32 {
    m.call(0x0015_7278, &[p], &[]);
    m.w32(p + 0x14, 1);
    m.w32(p + 0x18, 0x002c_f508);
    m.w32(p + 0x1c, 0);
    m.w32(p + 0x20, 0);
    m.w32(p + 0x24, 0);
    p
}

/// FUN_00166968: finish: open an A+D tag when none is open, then reserve(1)
pub fn ee_sub_166968<M: QMem + ?Sized>(m: &mut M, p: u32) {
    if m.r32(p + 0x1c) == 0 {
        ee_sub_166bd8(m, p, 0, 1, 0xe);
    }
    reserve(m, p, 1);
}

/// FUN_001669e0: close the open tag: pad the cursor to 16, NLOOP = count / NREG (trap on NREG 0),
/// EOP (bit 15) when `eop`
pub fn ee_sub_1669e0<M: QMem + ?Sized>(m: &mut M, p: u32, eop: u32) {
    let t = m.r32(p + 0x1c);
    if t == 0 {
        return;
    }
    let nreg = m.r32(p + 0x20) as i32;
    let cnt = m.r32(p + 0x24) as i32;
    assert!(nreg != 0, "FUN_001669e0: trap 7 (NREG 0)");
    let c = m.r32(p + 0xc);
    if c & 0xf != 0 {
        m.w32(p + 0xc, c + 8);
        w64(m, c, 0);
    }
    let mut w = m.r32(t);
    if eop != 0 {
        w |= 0x8000;
    }
    m.w32(t, w | (cnt / nreg) as u32);
    m.w32(p + 0x1c, 0);
}

/// FUN_00166a50: count one loop entry
pub fn ee_sub_166a50<M: QMem + ?Sized>(m: &mut M, p: u32) {
    let c = m.r32(p + 0x24).wrapping_add(1);
    m.w32(p + 0x24, c);
}

/// FUN_00166a60: raw GIF tag {NLOOP nloop, EOP-field eop << 15, PRE, PRIM prim, NREG nreg} + REGS
pub fn ee_sub_166a60<M: QMem + ?Sized>(m: &mut M, p: u32, prim: u64, nloop: u64, nreg: u64, regs: u64, eop: u64) {
    reserve(m, p, 0);
    let c = take(m, p, 0x10);
    w64(m, c + 8, regs);
    w64(m, c, nloop | (eop & 0xffff_ffff) << 0xf | prim << 0x2f | 0x4000_0000_0000 | nreg << 0x3c);
}

/// FUN_00166b10: GIF tag with `nloop` x `nreg` qwords of data left in place (their start to *out)
pub fn ee_sub_166b10<M: QMem + ?Sized>(m: &mut M, p: u32, prim: u64, nloop: u64, nreg: u64, regs: u64, out: u32) {
    reserve(m, p, 0);
    let c = take(m, p, 0x10);
    w64(m, c + 8, regs);
    w64(m, c, nloop | prim << 0x2f | 0x4000_0000_0000 | nreg << 0x3c);
    let cur = m.r32(p + 0xc);
    if out != 0 {
        m.w32(out, cur);
    }
    let n = (nloop as u32).wrapping_mul(nreg as u32).wrapping_mul(0x10);
    m.w32(p + 0xc, cur.wrapping_add(n));
}

/// FUN_00166bd8: open a PACKED tag (PRE, PRIM prim, NREG nreg, REGS regs); loop count reset
pub fn ee_sub_166bd8<M: QMem + ?Sized>(m: &mut M, p: u32, prim: u64, nreg: u64, regs: u64) {
    reserve(m, p, 0);
    let c = take(m, p, 0x10);
    w64(m, c, prim << 0x2f | nreg << 0x3c | 0x4000_0000_0000);
    w64(m, c + 8, regs);
    m.w32(p + 0x20, nreg as u32);
    m.w32(p + 0x1c, c);
    m.w32(p + 0x24, 0);
}

/// FUN_00166c70: PACKED RGBAQ from an 0xAABBGGRR word
pub fn ee_sub_166c70<M: QMem + ?Sized>(m: &mut M, p: u32, rgba: u32) {
    ee_sub_166a50(m, p);
    let c = take(m, p, 0x10);
    m.w32(c, rgba & 0xff);
    m.w32(c + 0xc, rgba >> 0x18);
    m.w32(c + 4, (rgba & 0xff00) >> 8);
    m.w32(c + 8, rgba >> 0x10 & 0xff);
}

/// FUN_00166ce0: PACKED UV / two words
pub fn ee_sub_166ce0<M: QMem + ?Sized>(m: &mut M, p: u32, a: u32, b: u32) {
    ee_sub_166a50(m, p);
    let c = take(m, p, 0x10);
    m.w32(c + 4, b);
    m.w32(c, a);
}

/// FUN_00166d38: PACKED ST(Q) words x, y, z (w untouched); the packet is the 4th argument
/// VFIX: asm takes the packet in a0 and x/y/z in $f12/$f13/$f14 (swc1), not Ghidra's 4 int args
pub fn ee_sub_166d38<M: QMem + ?Sized>(m: &mut M, p: u32, x: f32, y: f32, z: f32) {
    ee_sub_166a50(m, p);
    let c = take(m, p, 0x10);
    m.w32(c + 8, z.to_bits());
    m.w32(c, x.to_bits());
    m.w32(c + 4, y.to_bits());
}

/// FUN_00166da0: PACKED qword {a, b, d64}
pub fn ee_sub_166da0<M: QMem + ?Sized>(m: &mut M, p: u32, a: u32, b: u32, d: u64) {
    ee_sub_166a50(m, p);
    let c = take(m, p, 0x10);
    w64(m, c + 8, d);
    m.w32(c, a);
    m.w32(c + 4, b);
}

/// FUN_00166e08: PACKED XYZ(F)2 {x, y, z, (f & 0xff) << 4}
pub fn ee_sub_166e08<M: QMem + ?Sized>(m: &mut M, p: u32, x: u32, y: u32, z: u32, f: u32) {
    ee_sub_166a50(m, p);
    let c = take(m, p, 0x10);
    m.w32(c + 0xc, (f & 0xff) << 4);
    m.w32(c, x);
    m.w32(c + 4, y);
    m.w32(c + 8, z);
}

/// FUN_00166e90: PACKED A+D {data, reg}
pub fn ee_sub_166e90<M: QMem + ?Sized>(m: &mut M, p: u32, reg: u64, data: u64) {
    ee_sub_166a50(m, p);
    let c = take(m, p, 0x10);
    w64(m, c + 8, reg);
    w64(m, c, data);
}

/// FUN_00166ef0: open a REGLIST tag (FLG 1, NREG n, REGS regs)
pub fn ee_sub_166ef0<M: QMem + ?Sized>(m: &mut M, p: u32, n: u64, regs: u64) {
    reserve(m, p, 0);
    let c = take(m, p, 0x10);
    w64(m, c + 8, regs);
    w64(m, c, n << 0x3c | 0x0400_0000_0000_0000);
    m.w32(p + 0x20, n as u32);
    m.w32(p + 0x1c, c);
    m.w32(p + 0x24, 0);
}

/// FUN_00166f70(pkt a0, lo a1, f f12): REGLIST dword (lo as u32) | f32 bits(f) << 32
pub fn ee_sub_166f70<M: QMem + ?Sized>(m: &mut M, p: u32, lo: u64, f: f32) {
    ee_sub_166a50(m, p);
    let c = take(m, p, 8);
    w64(m, c, lo & 0xffff_ffff | (f.to_bits() as u64) << 0x20);
}

fn sx16(y: i32) -> u64 {
    (y << 0x10) as i64 as u64
}

/// FUN_00166fe8: REGLIST dword x | y << 16 (UV / XY)
pub fn ee_sub_166fe8<M: QMem + ?Sized>(m: &mut M, p: u32, x: u64, y: i32) {
    ee_sub_166a50(m, p);
    let c = take(m, p, 8);
    w64(m, c, x | sx16(y));
}

/// FUN_00167048: REGLIST XYZ2 dword x | y << 16 | z << 32
pub fn ee_sub_167048<M: QMem + ?Sized>(m: &mut M, p: u32, x: u64, y: i32, z: u64) {
    ee_sub_166a50(m, p);
    let c = take(m, p, 8);
    w64(m, c, x | sx16(y) | z << 0x20);
}

/// FUN_001670b8: REGLIST XYZF2 dword x | y << 16 | (z & 0xffffff) << 32 | f << 56
pub fn ee_sub_1670b8<M: QMem + ?Sized>(m: &mut M, p: u32, x: u64, y: i32, z: u64, f: u64) {
    ee_sub_166a50(m, p);
    let c = take(m, p, 8);
    w64(m, c, x | sx16(y) | (z & 0xff_ffff) << 0x20 | f << 0x38);
}

/// FUN_00167150: IMAGE-mode tag (FLG 2) with NLOOP-word `w`
pub fn ee_sub_167150<M: QMem + ?Sized>(m: &mut M, p: u32, w: u64) {
    reserve(m, p, 0);
    let c = take(m, p, 0x10);
    w64(m, c, w | 0x0800_0000_0000_0000);
    w64(m, c + 8, 0);
}

/// FUN_00167310: +0x4c = -1
pub fn ee_sub_167310<M: QMem + ?Sized>(m: &mut M, p: u32) {
    m.w32(p + 0x4c, 0xffff_ffff);
}
