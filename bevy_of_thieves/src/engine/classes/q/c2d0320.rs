//! WP-Q7: class C2d0320 (parent C2d02a0, 0x70-byte behaviour object) queue package: slots 5..9 and
//! FUN_0017a468. Written from the disassembly. Layout: +0x10 owner (actor; +0x2c0 u64 input-lock mask,
//! +0xf80 facing mode, +0xf84 / +0xf90 frames, +0xfd0 facing controller), +0x30 busy, +0x3c mask,
//! +0x40 target record, +0x50 goal point, +0x60 goal object, +0x64 goal param, +0x68 goal kind,
//! +0x6c has-frame, +0x70..+0x9c goal frame (3 rows).
//! Slot functions that only forward to the parent take the caller's remaining argument registers in
//! `rest` and pass them through unchanged (the original leaves a1.. untouched across the `jal`).
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem};
use crate::engine::classes::q::q7util::*;

/// DAT_002dfb78 (mask built) / DAT_002dfb70 (u64 mask of bits 0x1b, 0x27, 0x28, 0x29)
pub const MASK_DONE: u32 = 0x002d_fb78;
pub const MASK: u32 = 0x002d_fb70;

fn fwd(o: u32, rest: &[u32]) -> Vec<u32> {
    let mut a = vec![o];
    a.extend_from_slice(rest);
    a
}

/// FUN_001144d0 (slot 5)
pub fn c2d0320_v5() -> u32 {
    0x002b_b858
}

/// FUN_00179f70 (slot 6): parent FUN_00174920, then +0x3c = 0xf
pub fn c2d0320_v6<M: QMem + ?Sized>(m: &mut M, o: u32, rest: &[u32]) {
    m.call(0x0017_4920, &fwd(o, rest), &[]);
    m.w32(o + 0x3c, 0xf);
}

/// FUN_0017a3d0 (slot 8): parent FUN_00174b28, then FUN_001ef038
pub fn c2d0320_v8<M: QMem + ?Sized>(m: &mut M, o: u32, rest: &[u32]) {
    m.call(0x0017_4b28, &fwd(o, rest), &[]);
    m.call(0x001e_f038, &[o], &[]);
}

/// FUN_0017a400 (slot 9): parent FUN_00174b68; with a goal frame, idle (+0x30 == 0) and a target record
/// count > 1: FUN_00209840(owner +0xfd0, +0x70)
pub fn c2d0320_v9<M: QMem + ?Sized>(m: &mut M, o: u32, rest: &[u32]) {
    m.call(0x0017_4b68, &fwd(o, rest), &[]);
    if m.r32(o + 0x6c) != 0 && m.r32(o + 0x30) == 0 {
        let t = m.r32(o + 0x40);
        let r = m.r32(t + 8);
        if 1 < m.r32(r + 0x10) as i32 {
            let ow = m.r32(o + 0x10);
            m.call(0x0020_9840, &[ow + 0xfd0, o + 0x70], &[]);
        }
    }
}

/// FUN_0017a468: goal point -= owner world origin (+0x60 +0x30) + goal frame * (owner +0xf90 +0x90)
pub fn ee_sub_17a468<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let ow = m.r32(o + 0x10);
    let fr_ = m.r32(ow + 0xf90);
    let w = m.r32(ow + 0x60);
    let v = m.rv(fr_ + 0x90);
    let org = m.rv(w + 0x30);
    let g = m.rv(o + 0x50);
    let r = rows3(m, o + 0x70);
    let off = mul3(&r, v, v[3]);
    m.wv(o + 0x50, sub(sub(g, org), off));
}

/// FUN_00179fa0 (slot 7): resolve the goal (+0x60) into a point / frame and the owner input-lock bits it
/// needs (flag 10 path point FUN_001e6f60, else ALO position + heading; flag 0xd turn-around frame, 0x27,
/// 0xb path frame FUN_001c3500, 0x13 kind-3 ALO frame, otherwise drop the goal with FUN_00173610), run the
/// parent update FUN_00174950 with those bits set, restore the mask, then aim the owner's facing
/// controller (+0xfd0) by mode +0xf80 and push the goal into the target record (FUN_0020c6f8);
/// FUN_001eefc0 last.
pub fn c2d0320_v7<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.r32(MASK_DONE) == 0 {
        let mut k = 0u64;
        for b in [0x1b, 0x27, 0x28, 0x29] {
            k |= m.call(0x0020_7a20, &[b], &[]).0 as u64;
        }
        m.w32(MASK_DONE, 1);
        m.w32(MASK, k as u32);
        m.w32(MASK + 4, (k >> 32) as u32);
    }
    let sp = stack_push(m, 0x80);
    let mask = m.r64(MASK);
    let ow = m.r32(o + 0x10);
    let cur = m.r64(ow + 0x2c0);
    let saved = cur & mask;
    w64(m, ow + 0x2c0, cur & !mask);
    let mut s1 = 0u64;
    let g = m.r32(o + 0x60);
    if g != 0 {
        let (y, x) = if m.call(0x0013_2b10, &[g, 0xa], &[]).0 != 0 {
            let ow = m.r32(o + 0x10);
            let pos = m.call(0x001a_62e8, &[ow], &[]).1;
            m.call(0x001e_6f60, &[g, 0, o + 0x50, sp, 0, sp + 0x40], &[pos]);
            (m.rf(sp + 4), m.rf(sp))
        } else {
            let pos = m.call(0x001a_62e8, &[g], &[]).1;
            m.wv(o + 0x50, pos);
            let s0 = m.call(0x001a_6318, &[g], &[]).0;
            let v = m.call(0x001a_6318, &[g], &[]).0;
            (m.rf(s0 + 4), m.rf(v))
        };
        let mut ang = fr(m.call(0x0024_58f0, &[], &[f(y), f(x)]));
        let g = m.r32(o + 0x60);
        if m.call(0x0013_2b10, &[g, 0xd], &[]).0 != 0 {
            s1 |= m.call(0x0020_7a20, &[0x1b], &[]).0 as u64;
            if m.r32(o + 0x64) == 0 {
                ang = fr(m.call(0x0011_cf60, &[], &[f(ang + std::f32::consts::PI)]));
            }
            m.w32(o + 0x6c, 1);
            let ax = m.rv(0x002a_2dc0);
            m.call(0x0011_b500, &[sp + 0x10], &[f(ang), ax]);
            copy(m, o + 0x70, sp + 0x10, 0x30);
        } else if m.call(0x0013_2b10, &[g, 0x27], &[]).0 != 0 {
            s1 |= m.call(0x0020_7a20, &[0x29], &[]).0 as u64;
        } else if m.call(0x0013_2b10, &[g, 0xb], &[]).0 != 0 {
            s1 |= m.call(0x0020_7a20, &[0x27], &[]).0 as u64;
            let (g, a1) = (m.r32(o + 0x60), m.r32(o + 0x64));
            // quirk: sp+0x40 is only written on the flag-10 path above (stale stack otherwise)
            let t = m.rf(sp + 0x40);
            m.w32(o + 0x6c, 1);
            m.call(0x001c_3500, &[g, a1, o + 0x70], &[f(t)]);
            ee_sub_17a468(m, o);
        } else if m.call(0x0013_2b10, &[g, 0x13], &[]).0 != 0 && {
            let g = m.r32(o + 0x60);
            m.r32(g + 0x124) == 3
        } {
            s1 |= m.call(0x0020_7a20, &[0x28], &[]).0 as u64;
            m.w32(o + 0x6c, 1);
            let g = m.r32(o + 0x60);
            let xf = m.call(0x001a_6318, &[g], &[]).0;
            let r0 = m.rv(xf);
            let ax = m.rv(0x002a_2dc0);
            m.call(0x0011_b830, &[sp], &[r0, ax]);
            copy(m, o + 0x70, sp, 0x30);
            ee_sub_17a468(m, o);
        } else {
            let (b, a) = (m.rf(0x002b_e6b8), m.rf(0x002b_e6bc));
            m.call(0x0017_3610, &[o + 0x50, 1, 0], &[f(a), f(b)]);
            m.w32(o + 0x60, 0);
        }
    }
    let ow = m.r32(o + 0x10);
    let v = m.r64(ow + 0x2c0);
    w64(m, ow + 0x2c0, v | s1);
    m.call(0x0017_4950, &[o], &[]);
    let ow = m.r32(o + 0x10);
    let v = m.r64(ow + 0x2c0);
    w64(m, ow + 0x2c0, (v & !s1) | saved);
    if m.r32(o + 0x40) != 0 {
        let ow = m.r32(o + 0x10);
        let mode = m.r32(ow + 0xf80) as i32;
        if mode == 2 {
            let gp = m.rv(o + 0x50);
            let pos = m.call(0x001a_62e8, &[ow], &[]).1;
            let d = sub(gp, pos);
            let f84 = m.r32(ow + 0xf84);
            let xf = m.call(0x001a_6318, &[f84], &[]).0;
            let r0 = m.rv(xf);
            m.call(0x0011_b830, &[sp], &[r0, d]);
            let ow = m.r32(o + 0x10);
            m.call(0x0020_9840, &[ow + 0xfd0, sp], &[]);
        } else if mode == -1 || mode == 0 || mode == 3 {
            let gp = m.rv(o + 0x50);
            let pos = m.call(0x001a_62e8, &[ow], &[]).1;
            let d = sub(gp, pos);
            let ang = fr(m.call(0x0024_58f0, &[], &[f(d[1]), f(d[0])]));
            let ow = m.r32(o + 0x10);
            m.call(0x0020_9818, &[ow + 0xfd0], &[f(ang)]);
        }
        let t = m.r32(o + 0x40);
        let r = m.r32(t + 8);
        let (k, g, p) = (m.r32(o + 0x68), m.r32(o + 0x60), m.r32(o + 0x64));
        m.w32(r + 0x60, k);
        m.w32(r + 0x58, g);
        m.w32(r + 0x5c, p);
        let gp = m.rv(o + 0x50);
        m.call(0x0020_c6f8, &[r], &[gp]);
    }
    m.call(0x001e_efc0, &[o], &[]);
    stack_pop(m, 0x80);
}

pub fn w64<M: QMem + ?Sized>(m: &mut M, a: u32, v: u64) {
    m.w32(a, v as u32);
    m.w32(a + 4, (v >> 32) as u32);
}
