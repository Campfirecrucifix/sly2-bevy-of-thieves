//! WIRE-W1E: 1:1 queue ports (against `QMem`) of player natives (jt body/attach states, cane sub-state,
//! SetJtJts, ...), ported from the EE asm (tools/eedis.py). Callees go through `m.call` / `vslot`.
//! QMem convention: integer / pointer args in `a` (register order), quadword (128-bit GPR) and float args
//! in `v` (source order). The original stack frame is emulated with `stack_push` at the same offsets, so
//! out-buffers handed to callees live at their original relative positions. VU registers are tracked as
//! locals so `.xyz` ops keep the destination's previous w exactly as the hardware does; GPR upper 64 bits
//! that the asm leaves undefined (after `mfc1` / `lw` + `sq` / `qmtc2`) are modelled as 0.
#![allow(clippy::too_many_lines, unused_assignments)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

const T_DT: u32 = 0x002d_ded4; // frame dt
const ONE_W: V4 = [0.0, 0.0, 0.0, 1.0]; // vf00

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

/// 64-bit GPR value (`lw` / `mfc1`, sign-extended) moved to a quadword (`qmtc2` / `sq`): upper 64 bits
/// undefined in the asm, modelled as 0
fn gq(lo: u32) -> V4 {
    let hi = if lo & 0x8000_0000 != 0 { u32::MAX } else { 0 };
    [f32::from_bits(lo), f32::from_bits(hi), 0.0, 0.0]
}

fn gqf(x: f32) -> V4 {
    gq(x.to_bits())
}

fn add(a: V4, b: V4) -> V4 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2], a[3] + b[3]]
}

fn sub(a: V4, b: V4) -> V4 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2], a[3] - b[3]]
}

fn scl(a: V4, s: f32) -> V4 {
    [a[0] * s, a[1] * s, a[2] * s, a[3] * s]
}

/// vmul.xyz d,a,a ; vaddw.x ; vadday.x ; vmaddz.x -> x = |a.xyz|^2
fn dot3(a: V4) -> f32 {
    a[0] * a[0] + a[1] * a[1] + a[2] * a[2]
}

fn dotp(a: V4, b: V4) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// vmulax/vmadday/vmaddz.xyz d, (r0,r1,r2), s -> xyz = r0*s.x + r1*s.y + r2*s.z, w = d.w
fn mx3(d: V4, r0: V4, r1: V4, r2: V4, s: V4) -> V4 {
    let mut o = d;
    for i in 0..3 {
        o[i] = r0[i] * s[0] + r1[i] * s[1] + r2[i] * s[2];
    }
    o
}

/// same plus `vmaddw.xyz d, r3, vf00w` (translation)
fn mx4(d: V4, r0: V4, r1: V4, r2: V4, r3: V4, s: V4) -> V4 {
    let mut o = d;
    for i in 0..3 {
        o[i] = r0[i] * s[0] + r1[i] * s[1] + r2[i] * s[2] + r3[i];
    }
    o
}

/// vopmula.xyz ACC,a,b ; vopmsub.xyz d,b,a -> d.xyz = a x b, w = d.w
fn cross(d: V4, a: V4, b: V4) -> V4 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0], d[3]]
}

fn cp16<M: QMem + ?Sized>(m: &mut M, dst: u32, src: u32) {
    let v = m.rv(src);
    m.wv(dst, v);
}

// ---------------------------------------------------------------------------------------------------

/// FUN_00185e40(jt): ledge/pipe "align to surface" step: samples the surface (FUN_00185310) at the jt
/// position and at the 3 feet points (+0x1f30..), builds the surface frame (cross product, FUN_0011b830),
/// stores it at +0x1b0, then movement velocity (+0x190) from the joystick, FUN_001856c0 sweep,
/// FUN_0018c250(jt, 0x15, 0xb / 0xa) and vt slot 34 with the new position.
pub fn ee_sub_185e40<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let s1 = a0;
    let sp = stack_push(m, 0x320);
    let mut vf = [[0.0f32; 4]; 32];
    vf[0] = ONE_W;
    let v0q = m.call(0x001a_62e8, &[s1], &[]).1;
    m.wv(sp + 0x220, v0q);
    let a0w = m.r32(s1 + 0x1f18);
    m.call(0x0018_5310, &[a0w, s1 + 0x1f1c, s1 + 0x1f1c, sp, sp + 0x10], &[v0q]);
    vf[1] = m.rv(sp);
    vf[2] = m.rv(sp + 0x220);
    vf[3] = sub(vf[1], vf[2]);
    let len = dot3(vf[3]).sqrt();
    vf[4] = [len, 0.0, 0.0, 0.0];
    vf[2] = [-1.0, f32::from_bits(u32::MAX), v0q[2], v0q[3]];
    vf[1] = m.rv(sp + 0x10);
    vf[1] = scl(vf[1], vf[2][0]);
    m.wv(sp + 0x90, vf[2]);
    if len < 1e-4 {
        vf[2] = vf[1];
    } else {
        vf[2] = scl(vf[3], 1.0 / len);
    }
    let f0 = -m.rf(s1 + 0x1f30);
    let a2q = m.rv(0x002a_2dc0);
    vf[1] = m.rv(sp);
    vf[3] = gqf(f0);
    // vadda.xyzw ACC, vf01, vf00 ; vmaddx.xyzw vf04, vf02, vf03x
    vf[4] = add(add(vf[1], vf[0]), scl(vf[2], vf[3][0]));
    m.wv(sp + 0x230, vf[4]);
    m.wv(sp + 0x90, vf[3]);
    m.call(0x0011_b830, &[sp + 0xa0], &[vf[2], a2q]);
    let q = m.rv(sp + 0x230);
    m.call(0x0025_6d18, &[sp + 0x50], &[q]);
    m.w32(sp + 0x250, sp + 0x160);
    let (s7, fp) = (sp + 0x90, sp + 0xd0);
    cp16(m, sp + 0x20, sp + 0xa0);
    cp16(m, sp + 0x30, sp + 0xb0);
    cp16(m, sp + 0x40, sp + 0xc0);
    m.w32(sp + 0x4c, 0);
    m.w32(sp + 0x3c, 0);
    m.w32(sp + 0x2c, 0);
    m.wf(sp + 0x5c, 1.0);
    for i in 0..3u32 {
        vf[5] = m.rv(s1 + 0x1f30 + 0x10 * i);
        vf[4] = m.rv(sp + 0x20);
        vf[3] = m.rv(sp + 0x30);
        vf[2] = m.rv(sp + 0x40);
        vf[1] = m.rv(sp + 0x50);
        vf[6] = mx4(vf[6], vf[4], vf[3], vf[2], vf[1], vf[5]);
        let a0w = m.r32(s1 + 0x1f18);
        m.call(0x0018_5310, &[a0w, s1 + 0x1f1c, 0, sp + 0x60 + 0x10 * i, 0], &[vf[6]]);
    }
    vf[3] = m.rv(sp + 0x60);
    vf[1] = m.rv(sp + 0x80);
    vf[2] = m.rv(sp + 0x70);
    vf[1] = sub(vf[1], vf[3]);
    vf[2] = sub(vf[2], vf[3]);
    vf[1] = cross(vf[1], vf[1], vf[2]);
    let l = dot3(vf[1]).sqrt();
    vf[3] = scl(vf[1], 1.0 / l);
    let a2q = m.rv(0x002a_2dc0);
    m.call(0x0011_b830, &[s7], &[vf[3], a2q]);
    let f20 = 0.5f32;
    cp16(m, s1 + 0x1d0, sp + 0xb0);
    cp16(m, s1 + 0x1b0, sp + 0x90);
    cp16(m, s1 + 0x1c0, sp + 0xa0);
    let s5 = s1 + 0x1b0;
    let xf = m.call(0x001a_6318, &[s1], &[]).0;
    let f12 = m.rf(T_DT) * f20;
    m.call(0x0011_c950, &[xf, s5, 0x002b_e920, 0, s7], &[f4(f12)]);
    let q = m.rv(sp + 0x90);
    vslot(m, s1, 35, &[], &[q]);
    let f12 = m.rf(T_DT) * f20;
    let (qa, qb) = (m.rv(sp + 0x220), m.rv(sp + 0x230));
    m.call(0x0011_f088, &[0x002b_e930, sp + 0xa0], &[qa, qb, f4(f12)]);
    vf[5] = m.rv(sp + 0x230);
    vf[4] = m.rv(s1 + 0x1b0);
    for i in 0..3u32 {
        vf[1] = m.rv(s1 + 0x1f30 + 0x10 * i);
        vf[3] = m.rv(s5 + 0x10);
        vf[2] = m.rv(s5 + 0x20);
        vf[1] = mx3(vf[1], vf[4], vf[3], vf[2], vf[1]);
        vf[1] = add(vf[5], vf[1]);
        m.wv(sp + 0x160 + 0x10 * i, vf[1]);
    }
    let f20 = 0.0f32;
    let a0w = m.r32(s1 + 0x1f18);
    m.call(0x0018_56b8, &[a0w, fp], &[]);
    let a0w = m.r32(s1 + 0x1f18);
    let a3 = m.r32(sp + 0x250);
    m.call(0x0018_56c0, &[a0w, fp, 3, a3, 0x002b_e940], &[f4(10.0)]);
    let o150 = m.r32(s1 + 0x150);
    if f20 < m.rf(o150 + 0x1c) {
        let f0 = m.rf(o150 + 0x10);
        let f1 = m.rf(o150 + 0x14);
        m.wf(sp + 0xb0, f20);
        m.wf(sp + 0xb8, f1);
        m.wf(sp + 0xb4, -f0);
        m.call(0x0011_b340, &[sp + 0x1c0, s5], &[]);
        vf[3] = m.rv(sp + 0x1d0);
        vf[2] = m.rv(sp + 0x1e0);
        vf[7] = m.rv(0x002d_dfc0);
        vf[6] = m.rv(0x002d_dfa0);
        vf[5] = m.rv(sp + 0x1c0);
        vf[4] = m.rv(0x002d_dfb0);
        vf[7] = mx3(vf[7], vf[5], vf[3], vf[2], vf[7]);
        vf[6] = mx3(vf[6], vf[5], vf[3], vf[2], vf[6]);
        vf[1] = m.rv(sp + 0xb0);
        vf[5] = mx3(vf[5], vf[5], vf[3], vf[2], vf[4]);
        m.wv(sp + 0x190, vf[6]);
        vf[1] = mx3(vf[1], vf[6], vf[5], vf[7], vf[1]);
        m.wv(sp + 0x1a0, vf[5]);
        m.wv(sp + 0xb0, vf[1]);
        m.wf(sp + 0xb0, f20);
        m.wv(sp + 0x1b0, vf[7]);
        vf[3] = m.rv(sp + 0xb0);
        let len = dot3(vf[3]).sqrt();
        vf[1] = [len, vf[1][1], vf[1][2], vf[1][3]];
        let o150 = m.r32(s1 + 0x150);
        let w = m.r32(o150 + 0x1c);
        vf[2] = [f32::from_bits(w), gq(w)[1], vf[1][2], vf[1][3]];
        let v0q = m.rv(0x002a_2dc0);
        m.wv(sp + 0x1f0, vf[6]);
        m.wv(sp + 0x200, vf[5]);
        m.wv(sp + 0x210, vf[7]);
        m.wv(sp + 0xc0, vf[2]);
        if len < 1e-4 {
            vf[1] = v0q;
        } else {
            vf[1] = scl(vf[3], 1.0 / len);
        }
        vf[1] = scl(vf[1], vf[2][0]);
        vf[2] = m.rv(s1 + 0x1e40);
        m.wv(sp + 0xb0, vf[1]);
        vf[1] = m.rv(s1 + 0x1e60);
        let (v1, a0w) = (m.r32(sp + 0xb4), m.r32(sp + 0xb8));
        vf[3] = gq(v1);
        vf[4] = gq(a0w);
        vf[2] = add(scl(vf[2], vf[3][0]), scl(vf[1], vf[4][0]));
        m.wv(s1 + 0x190, vf[2]);
        let o = m.r32(s1 + 0x1f10);
        vf[1] = gq(m.r32(o + 0x78));
        m.wv(sp + 0xc0, vf[3]);
        m.wv(sp + 0xc0, vf[4]);
        vf[2] = scl(vf[2], vf[1][0]);
        m.wv(s1 + 0x190, vf[2]);
        m.wv(sp + 0xc0, vf[1]);
    } else {
        m.wf(s1 + 0x194, f20);
        m.wf(s1 + 0x198, f20);
        m.wf(s1 + 0x190, f20);
    }
    if (m.r32(sp + 0xd0) as i32) <= 0 {
        vf[1] = m.rv(s1 + 0x190);
    } else {
        let v0 = gqf(-1.0);
        m.wv(sp + 0x240, v0);
        vf[2] = v0;
        vf[1] = m.rv(s1 + 0x190);
        vf[1] = scl(vf[1], vf[2][0]);
        vf[4] = m.rv(s1 + 0x1b0);
        vf[3] = m.rv(s5 + 0x10);
        vf[2] = m.rv(s5 + 0x20);
        vf[5] = mx3(vf[5], vf[4], vf[3], vf[2], vf[1]);
        let a0w = m.r32(s1 + 0x1f18);
        m.wv(sp + 0xb0, v0);
        let r = m.call(0x0018_5a80, &[a0w, fp], &[vf[5]]).1;
        vf[2] = r;
        m.wv(sp + 0x260, vf[2]);
        m.call(0x0011_b340, &[sp + 0x190, s5], &[]);
        vf[6] = m.rv(sp + 0x240);
        vf[2] = m.rv(sp + 0x260);
        vf[4] = m.rv(sp + 0x190);
        vf[2] = scl(vf[2], vf[6][0]);
        vf[3] = m.rv(sp + 0x1a0);
        vf[1] = m.rv(sp + 0x1b0);
        vf[2] = mx3(vf[2], vf[4], vf[3], vf[1], vf[2]);
        m.wv(sp + 0xb0, vf[6]);
        m.wv(s1 + 0x190, vf[2]);
        vf[1] = m.rv(s1 + 0x190);
    }
    let d = dot3(vf[1]);
    let mut v1 = false;
    if 1.0 < d {
        v1 = true;
    } else {
        let o150 = m.r32(s1 + 0x150);
        if m.rf(o150 + 0x1c) != 0.0 {
            v1 = true;
        }
    }
    if v1 {
        m.call(0x0018_c250, &[s1, 0x15, 0xb], &[]);
        let a2 = m.r32(s1 + 0x1f10);
        m.call(0x001a_0450, &[s1, s1 + 0x1e10, a2], &[]);
        vf[5] = gqf(-1.0);
        vf[2] = m.rv(s1 + 0x190);
        vf[3] = m.rv(s1 + 0x1b0);
        vf[2] = scl(vf[2], vf[5][0]);
        vf[1] = m.rv(s5 + 0x10);
        vf[4] = m.rv(s5 + 0x20);
        vf[2] = mx3(vf[2], vf[3], vf[1], vf[4], vf[2]);
        vf[1] = m.rv(sp + 0xa0);
        vf[3] = add(vf[1], vf[2]);
        m.wv(sp + 0xb0, vf[5]);
        vslot(m, s1, 34, &[], &[vf[3]]);
    } else {
        m.call(0x0018_c250, &[s1, 0x15, 0xa], &[]);
        let q = m.rv(sp + 0xa0);
        vslot(m, s1, 34, &[], &[q]);
    }
    stack_pop(m, 0x320);
}


// ---------------------------------------------------------------------------------------------------

const GS: u32 = 0x003d_d490; // game state / world
const T_NOW: u32 = 0x002d_ded0; // frame time
const G_FLAGS: u32 = 0x002d_fbc4; // global flags (bit 0)

/// `(*(jt+0xc)+0x2c4)(jt + adj, a1)` (vt slot 87: set jt body state)
fn jt_slot87<M: QMem + ?Sized>(m: &mut M, s3: u32, a1: u32) {
    let a2 = m.r32(s3);
    vslot(m, a2, 87, &[a1], &[]);
}

/// 0x191934: FUN_001c4738(s7, 0x400); FUN_0019cc50(jt + 0xf00, 0, 0, 0)
fn p_191934<M: QMem + ?Sized>(m: &mut M, s3: u32, s7: u32) {
    m.call(0x001c_4738, &[s7, 0x400], &[]);
    let a0 = m.r32(s3);
    m.call(0x0019_cc50, &[a0 + 0xf00, 0, 0, 0], &[]);
}

/// 0x19197c: v0 (+0x500) == 0 && !(G_FLAGS & 1) && !(s7+0x30 & 0x20) -> slot 87 (0)
fn p_19197c<M: QMem + ?Sized>(m: &mut M, s3: u32, s7: u32, v0: u32) {
    if v0 != 0 || m.r32(G_FLAGS) & 1 != 0 || m.r32(s7 + 0x30) & 0x20 != 0 {
        return;
    }
    jt_slot87(m, s3, 0);
}

/// 0x191270: FUN_001c4660(s7, 0x400, 0.15) && slot 98 (3) -> 0x191934
fn p_191270<M: QMem + ?Sized>(m: &mut M, s3: u32, s7: u32) {
    if m.call(0x001c_4660, &[s7, 0x400], &[f4(0.15)]).0 == 0 {
        return;
    }
    let a2 = m.r32(s3);
    if vslot(m, a2, 98, &[3], &[]).0 == 0 {
        return;
    }
    p_191934(m, s3, s7);
}

/// 0x191380 (attach state 8, sub-state not 7/8): BSP query around the jt position for blockers
/// (FUN_0014c6a0 hits with nonzero +0x40 -> abort), then query along the jt's attach segment (+0x268 +0x90)
/// and test each found object (FUN_001a9b28 / FUN_00118a88 + FUN_001a8b50); none -> FUN_00192bd8(s3, 8).
fn p_191380<M: QMem + ?Sized>(m: &mut M, s3: u32, sp: u32) {
    let mut vf = [[0.0f32; 4]; 8];
    m.call(0x001a_e478, &[sp], &[]);
    let a0 = m.r32(s3);
    let v0q = m.call(0x001a_62e8, &[a0], &[]).1;
    let a1 = m.r32(s3);
    let v1 = m.r32(0x002b_f5b0);
    vf[2] = m.rv(a1 + 0x240);
    vf[1] = m.rv(0x002a_2dc0);
    vf[3] = gq(v1);
    vf[4] = v0q;
    vf[1] = scl(vf[1], vf[3][0]);
    vf[2] = add(vf[4], vf[2]);
    let t0 = m.r32(a1 + 0xec);
    vf[4] = add(vf[2], vf[1]);
    m.wv(sp + 0x340, vf[4]);
    let (s4, s6) = (sp + 0x320, sp + 0x324);
    m.wv(sp + 0x390, vf[2]);
    let a1q = m.rv(sp + 0x340);
    m.wv(sp + 0x310, vf[3]);
    m.call(0x0020_faa0, &[GS, 0x0020_f948, t0, s4, s6], &[a1q, vf[2]]);
    vf[2] = m.rv(sp + 0x390);
    let mut s2 = 0u32;
    if (m.r32(sp + 0x320) as i32) > 0 {
        loop {
            let v1 = m.r32(sp + 0x324);
            let a2q = m.rv(sp + 0x340);
            let a0 = m.r32(v1 + 4 * s2);
            m.wv(sp + 0x390, vf[2]);
            let n = m.call(0x0014_c6a0, &[a0, 8, sp + 0x10], &[vf[2], a2q]).0 as i32;
            vf[2] = m.rv(sp + 0x390);
            for k in 0..n.max(0) as u32 {
                if m.rf(sp + 0x10 + 0x40 + 0x60 * k) != 0.0 {
                    m.call(0x001a_e4a0, &[sp, 2], &[]);
                    return;
                }
            }
            s2 += 1;
            if (s2 as i32) >= m.r32(sp + 0x320) as i32 {
                break;
            }
        }
    }
    let mut s5 = 0i32;
    let jt = m.r32(s3);
    let s1 = m.r32(jt + 0x268);
    m.call(0x001f_38a8, &[s1], &[]);
    let s0 = s1 + 0x90;
    let (a0, a1) = (m.r32(s1 + 0x90), m.r32(s0 + 4));
    let q = m.call(0x0011_f288, &[a0, a1], &[]).1;
    m.wv(sp + 0x350, q);
    let (a0, a1) = (m.r32(s1 + 0x90), m.r32(s0 + 4));
    let q = m.call(0x0011_f2b8, &[a0, a1], &[]).1;
    m.wv(sp + 0x360, q);
    let jt = m.r32(s3);
    let t0 = m.r32(jt + 0xec);
    let (qa, qb) = (m.rv(sp + 0x350), m.rv(sp + 0x360));
    m.call(0x0020_faa0, &[GS, 0x0020_f948, t0, s4, s6], &[qa, qb]);
    let n = m.r32(sp + 0x320) as i32;
    if n > 0 {
        let mut a1 = m.r32(sp + 0x324);
        for _ in 0..n {
            let o = m.r32(a1);
            a1 += 4;
            let v1 = m.r32(o + 0x6c) as i32;
            if v1 >= s5 {
                s5 = v1;
            }
        }
    }
    if 0x1554 < s5 {
        s5 = 0x1555;
    }
    m.w32(sp + 0x310, s5.wrapping_mul(12) as u32);
    m.w32(sp + 0x314, 0x10);
    let s6 = m.call(0x001a_e318, &[sp + 0x310], &[]).0;
    vf[1] = m.rv(sp + 0x350);
    vf[2] = gqf(0.5);
    vf[3] = m.rv(sp + 0x360);
    vf[1] = add(scl(vf[1], vf[2][0]), scl(vf[3], vf[2][0]));
    m.wv(sp + 0x310, vf[2]);
    m.wv(sp + 0x370, vf[1]);
    let d0 = m.rv(sp + 0x370);
    let d = sub(d0, m.rv(sp + 0x350));
    let f20 = dot3(d).sqrt();
    s2 = 0;
    if (m.r32(sp + 0x320) as i32) > 0 {
        loop {
            let lst = m.r32(sp + 0x324);
            let s0 = m.r32(lst + 4 * s2);
            m.w32(sp + 0x328, 0);
            let a1;
            if m.r64(s0 + 0x10) & (0x8000u64 << 34) != 0 {
                m.call(0x001a_9b28, &[GS, s0, s1, 0, 0, sp + 0x328], &[]);
                a1 = m.r32(sp + 0x328);
            } else {
                let a0 = m.r32(s0 + 0x74);
                m.w32(sp + 0x32c, s5 as u32);
                m.w32(sp + 0x330, s6);
                let q = m.rv(sp + 0x370);
                let v = m.call(0x0011_8a88, &[a0, 0x7fff_ffff, sp + 0x32c, sp + 0x330], &[q, f4(f20)]).0;
                if v == 2 {
                    let t1 = m.r32(sp + 0x330);
                    let a3 = (t1.wrapping_sub(s6).wrapping_mul(0xaaaa_aaab) as i32) >> 2;
                    let t1 = t1.wrapping_sub(0xc);
                    m.w32(sp + 0x330, t1);
                    let t0 = m.r32(s0 + 0x74);
                    m.call(0x001a_8b50, &[GS, s1, s0, a3 as u32, t0, t1, sp + 0x328], &[]);
                }
                a1 = m.r32(sp + 0x328);
            }
            if a1 != 0 {
                m.call(0x001a_7e10, &[GS], &[]);
                m.call(0x001a_e4a0, &[sp, 2], &[]);
                return;
            }
            s2 += 1;
            if (s2 as i32) >= m.r32(sp + 0x320) as i32 {
                break;
            }
        }
    }
    m.call(0x0019_2bd8, &[s3, 8], &[]);
    m.call(0x001a_e4a0, &[sp, 2], &[]);
}

/// FUN_001911a8(attach): jt attach-state update (states 1..12 via table 0x2a60a0), re-run while the
/// sub-state (+0xc) changed and is not -1.
pub fn ee_sub_1911a8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let s3 = a0;
    let sp = stack_push(m, 0x450);
    let jt = m.r32(s3);
    let s7 = m.r32(jt + 0x150);
    let mut v1 = m.r32(s3 + 0xc);
    loop {
        let st = m.r32(s3 + 8);
        m.w32(sp + 0x334, v1);
        let sub_st = m.r32(sp + 0x334);
        match st {
            1 => {
                let f0 = m.rf(T_NOW) - m.rf(s3 + 0x10);
                if 0.15 < f0 && m.call(0x001c_46c0, &[s7, 0x40], &[f4(0.25)]).0 != 0 {
                    let (a0, a3) = (m.r32(s3), m.r32(s3 + 0x14));
                    m.call(0x0019_5a00, &[a0 + 0x2c0, 7, 0, a3], &[]);
                } else {
                    p_191270(m, s3, s7);
                }
            }
            2 => {
                let a0 = m.r32(s3);
                if (m.call(0x0014_41a0, &[a0], &[]).0 as i32) <= 0 {
                    jt_slot87(m, s3, 0);
                } else {
                    let a2 = m.r32(s3);
                    if m.r32(a2 + 0x10ec) == 4 {
                        vslot(m, a2, 87, &[0], &[]);
                    } else if m.r32(G_FLAGS) & 1 == 0 {
                        let v0 = m.call(0x0014_1ea8, &[a2, 0x28], &[]).0;
                        if m.r32(s7 + 0x30) & v0 == 0 {
                            jt_slot87(m, s3, 0);
                        }
                    }
                }
            }
            3 | 4 => {
                if m.call(0x001c_46c0, &[s7, 0x40], &[f4(0.1)]).0 != 0 {
                    let a0 = m.r32(s3);
                    m.call(0x0019_5a00, &[a0 + 0x2c0, 1, 0, 0], &[]);
                } else {
                    p_191270(m, s3, s7);
                }
            }
            5 => {
                let a0 = m.r32(s3 + 0x14);
                if m.call(0x0013_2b10, &[a0, 0xb], &[]).0 != 0 {
                    let a0 = m.r32(s3);
                    if m.call(0x0014_23a0, &[a0], &[]).0 != 0 {
                        let (a0, v0) = (m.r32(s3), m.r32(s3 + 0x14));
                        m.w32(a0 + 0xf5c, v0);
                        let (v1, f0) = (m.r32(s3), m.rf(s3 + 0x20));
                        m.wf(v1 + 0xf60, f0);
                        jt_slot87(m, s3, 8);
                    }
                }
            }
            7 => {
                let s0 = m.r32(s3 + 0x14);
                jt_slot87(m, s3, 0);
                let a1 = m.r32(s3);
                vslot(m, s0, 50, &[a1], &[]);
            }
            8 => {
                if sub_st == 7 || sub_st == 8 {
                    let jt = m.r32(s3);
                    let a0 = m.r32(jt + 0x1030);
                    if m.call(0x0013_1968, &[a0], &[]).0 != 0 {
                        if sub_st == 7 {
                            m.call(0x0019_2bd8, &[s3, 9], &[]);
                        } else {
                            jt_slot87(m, s3, 0);
                        }
                    }
                } else {
                    p_191380(m, s3, sp);
                }
            }
            9 => {
                let a0 = m.r32(s3 + 0x14);
                let p = m.call(0x001a_62e8, &[a0], &[]).1;
                m.wv(sp + 0x380, p);
                let a0 = m.r32(s3 + 0x474);
                let q = m.call(0x001a_62e8, &[a0], &[]).1;
                let d = sub(m.rv(sp + 0x380), q);
                let f1 = m.rf(0x002b_f5b4);
                if dot3(d).sqrt() < f1 {
                    let a1 = m.r32(s3 + 0x14);
                    m.call(0x001c_d098, &[s3 + 0x440, a1], &[]);
                    jt_slot87(m, s3, 0);
                }
            }
            10 => {
                let jt = m.r32(s3);
                let a0 = m.r32(jt + 0x1030);
                if a0 == 0 || m.call(0x0013_1968, &[a0], &[]).0 != 0 {
                    jt_slot87(m, s3, 0);
                } else {
                    let f0 = m.rf(T_NOW) - m.rf(s3 + 4);
                    if m.rf(0x002b_f5b8) < f0 {
                        let a2 = m.r32(s3);
                        let o = m.r32(a2 + 0x150);
                        let vt = m.r32(a2 + 0xc);
                        if 0.0 < m.rf(o + 0x1c) || m.r32(vt + 0x30) & 0xd0 != 0 {
                            vslot(m, a2, 87, &[0], &[]);
                            stack_pop(m, 0x450);
                            return;
                        }
                    }
                }
            }
            11 | 12 => {
                let jt = m.r32(s3);
                let a0 = m.r32(jt + 0x1030);
                if a0 == 0 || m.call(0x0013_1968, &[a0], &[]).0 != 0 {
                    jt_slot87(m, s3, 0);
                }
            }
            _ => {
                if m.call(0x001c_46c0, &[s7, 0x40], &[f4(0.1)]).0 != 0 {
                    let a0 = m.r32(s3);
                    m.call(0x0019_5a00, &[a0 + 0x2c0, 1, 0, 0], &[]);
                } else if m.call(0x001c_4660, &[s7, 0x400], &[f4(0.15)]).0 == 0 {
                    let v0 = m.r32(s3 + 0x500);
                    p_19197c(m, s3, s7, v0);
                } else {
                    let a2 = m.r32(s3);
                    let r = vslot(m, a2, 98, &[3], &[]).0;
                    let v1 = m.r32(s3 + 8);
                    if r != 0 && (v1 == 0 || v1 == 0xd) {
                        p_191934(m, s3, s7);
                    } else {
                        let v0 = m.r32(s3 + 0x500);
                        p_19197c(m, s3, s7, v0);
                    }
                }
            }
        }
        v1 = m.r32(s3 + 0xc);
        if v1 == m.r32(sp + 0x334) || v1 == u32::MAX {
            break;
        }
    }
    stack_pop(m, 0x450);
}

// ---------------------------------------------------------------------------------------------------

/// 0x18e7f8 / 0x18e834 / 0x18e870 / 0x18e8c0 (cane sub-states 4..7): FUN_0018f560(jt, jt+blk, *(jt+blk+0x18)),
/// sets s3 = 0xb, f20 = *0x2bf430, +0x19b0 = t + *0x2bf434 / *0x2bf438; 6/7 also FUN_001f0400 / FUN_001f09d8
/// (0x17b). Returns (s0, s3, f20).
fn cane_4to7<M: QMem + ?Sized>(m: &mut M, s1: u32, s2: i32) -> (u32, i32, f32) {
    let (blk, kf) = match s2 {
        4 => (0x1a04, 0x002b_f434),
        5 => (0x1a20, 0x002b_f438),
        6 => (0x1a3c, 0x002b_f434),
        _ => (0x1a58, 0x002b_f438),
    };
    let a2 = m.r32(s1 + blk + 0x18);
    m.call(0x0018_f560, &[s1, s1 + blk, a2], &[]);
    let f20 = m.rf(0x002b_f430);
    let s0;
    if s2 <= 5 {
        let f1 = m.rf(kf);
        let f0 = m.rf(T_NOW) + f1;
        s0 = m.r32(s1 + blk + 0x18);
        m.wf(s1 + 0x19b0, f0);
    } else {
        let a0 = m.r32(s1 + 0x1028);
        s0 = m.r32(s1 + blk + 0x18);
        if (m.call(0x001f_0400, &[a0, 0x17b], &[]).0 as i32) >= 0 {
            let a0 = m.r32(s1 + 0x1034);
            m.call(0x001f_09d8, &[a0, 0x17b], &[]);
        }
        let f1 = m.rf(kf);
        let f0 = m.rf(T_NOW) + f1;
        m.wf(s1 + 0x19b0, f0);
    }
    (s0, 0xb, f20)
}

/// FUN_001fbb90(id) -> FUN_001fb2f8(e, 0); FUN_001fb3f0(e, pos(jt))
fn fx_at_jt<M: QMem + ?Sized>(m: &mut M, s1: u32, id: u32) {
    let s0 = m.call(0x001f_bb90, &[id], &[]).0;
    m.call(0x001f_b2f8, &[s0, 0], &[]);
    let p = m.call(0x001a_62e8, &[s1], &[]).1;
    m.call(0x001f_b3f0, &[s0], &[p]);
}

/// FUN_0018e540(jt, cane sub-state): leave the old sub-state (+0x18d8), pick the new state's anim / timers
/// (table 0x2a5fa0, states -1..23), swap the 0x19a0/0x19a8 cane objects when entering/leaving 2/0xb, then
/// FUN_001305b8 anim start and optional FUN_00131a90 / effect.
pub fn ee_sub_18e540<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: i32) {
    let s1 = a0;
    let mut s2 = a1;
    let sp = stack_push(m, 0xb0);
    let cur = m.r32(s1 + 0x18d8) as i32;
    if cur == s2 {
        stack_pop(m, 0xb0);
        return;
    }
    let mut s4 = false;
    if s2 != -1 && cur != -1 {
        let v1 = m.r32(0x002b_f448u32.wrapping_add((cur as u32) << 2));
        s4 = s2 as u32 == v1;
    }
    if !s4 {
        let a1 = m.r32(s1 + 0x1a88);
        m.call(0x001f_2ad0, &[s1, a1], &[]);
    }
    let v1 = m.r32(s1 + 0x18d8) as i32;
    let mut s0: u32 = u32::MAX;
    if v1 == 0 {
        m.call(0x001c_cac0, &[s1 + 0x17d0, u32::MAX], &[]);
    } else if v1 == 1 {
        let o = m.r32(s1 + 0x18c4);
        s0 = m.r32(o + 0x130);
        m.call(0x001f_4bf8, &[s1], &[]);
    } else if v1 == 2 {
        m.call(0x001f_4bf8, &[s1], &[]);
    }
    if s0 != u32::MAX {
        fx_at_jt(m, s1, s0);
    }
    let mut f22 = 0.0f32;
    let mut f21 = 1.0f32;
    let mut f20 = f22;
    s0 = 0;
    let mut s3: i32 = -1;
    match s2 {
        -1 => m.w32(s1 + 0x18c0, 0),
        0 => {
            m.call(0x001c_cac0, &[s1 + 0x17d0, 0], &[]);
            f21 = 0.0;
            s0 = m.r32(s1 + 0x1800);
            s3 = 0xf;
            f22 = m.rf(s0 + 0x74) * 0.5;
            m.call(0x001f_4bf8, &[s1], &[]);
            let f0 = m.rf(T_NOW) + 0.1;
            m.wf(s1 + 0x19b0, f0);
        }
        1 | 2 => {
            m.call(0x001f_4bf8, &[s1], &[]);
        }
        3 => {
            let a0 = m.r32(s1 + 0x19a4);
            let p = m.call(0x001a_62e8, &[a0], &[]).1;
            let v = m.call(0x001a_6f00, &[0, s1], &[p]).1;
            let v1 = m.r32(s1 + 0x19c0);
            m.wv(sp + 0x20, v);
            let q = m.rv(v1 + 0x70);
            let vf04 = m.call(0x001a_6f00, &[0, s1], &[q]).1;
            let vf03 = m.rv(sp + 0x20);
            let vf01 = gqf(0.5);
            let vf02 = add(scl(vf03, vf01[0]), scl(vf04, vf01[0]));
            m.wv(sp + 0x10, vf01);
            m.wv(sp, vf02);
            let near = m.r32(s1 + 0x18c8) != 0;
            let (lo, hi) = if near { (4, 5) } else { (6, 7) };
            s2 = if m.r32(0x002e_1e44) != 0 {
                lo
            } else if m.r32(0x002e_1e48) != 0 {
                hi
            } else if m.rf(sp + 4) < 0.0 {
                lo
            } else {
                hi
            };
            (s0, s3, f20) = cane_4to7(m, s1, s2);
        }
        4..=7 => (s0, s3, f20) = cane_4to7(m, s1, s2),
        8 => s0 = m.r32(s1 + 0x1a74),
        9 => s0 = m.r32(s1 + 0x1a78),
        10 => {
            m.call(0x001f_4bf8, &[s1], &[]);
            s0 = m.r32(s1 + 0x14d4);
        }
        11 => {
            m.call(0x001f_4bf8, &[s1], &[]);
            s0 = m.r32(s1 + 0x14d8);
        }
        12..=23 => {
            if s2 >= 22 {
                s0 = if s2 != 0x16 { m.r32(s1 + 0x1d18) } else { m.r32(s1 + 0x1d14) };
            }
            let f1 = m.rf(0x002b_f440);
            let f0 = m.rf(T_NOW);
            f20 = m.rf(0x002b_f43c);
            let f0 = f0 + f1;
            let check = if s2 == 12 || s2 == 13 || s2 >= 22 {
                Some(m.r32(s1 + 0x18c8))
            } else {
                None
            };
            m.w32(s1 + 0x18ec, (s2 < 22) as u32);
            if s2 < 22 {
                s0 = m.r32(s1 + 0x1ce0 + 4 * (s2 as u32 - 12));
            }
            m.wf(s1 + 0x19b0, f0);
            if check == Some(0) {
                let a0 = m.r32(s1 + 0x1028);
                if (m.call(0x001f_0400, &[a0, 0x17b], &[]).0 as i32) >= 0 {
                    let a0 = m.r32(s1 + 0x1034);
                    m.call(0x001f_09d8, &[a0, 0x17b], &[]);
                }
            }
        }
        _ => {}
    }
    if !s4 {
        m.w32(s1 + 0x18e4, 0);
    } else {
        let v0 = m.r32(s1 + 0x18e0);
        s3 = -1;
        let f1 = m.rf(s0 + 0x74);
        let f2 = m.rf(v0 + 0x48);
        let f0 = m.rf(v0 + 0x1c);
        f21 = m.rf(v0 + 0x40) / f2;
        f22 = if f0 < f1 { f0 } else { f1 };
    }
    let v1 = m.r32(s1 + 0x18d8);
    let was = v1 == 0xb || v1 == 2;
    let now = s2 == 0xb || s2 == 2;
    if was != now {
        let (x, y) = if now { (0x19a8, 0x19a0) } else { (0x19a0, 0x19a8) };
        let a1 = m.r32(s1 + x);
        vslot(m, a1, 7, &[], &[]);
        let (a2, a1) = (m.r32(s1 + x), m.r32(s1 + y));
        vslot(m, a2, 37, &[a1], &[]);
        let a1 = m.r32(s1 + y);
        vslot(m, a1, 8, &[], &[]);
    }
    m.w32(s1 + 0x18d8, s2 as u32);
    let f0 = m.rf(T_NOW);
    m.w32(s1 + 0x1880, 0);
    m.wf(s1 + 0x18dc, f0);
    m.call(0x0013_05b8, &[s0, s1, 1, s1 + 0x18e0], &[f4(f22), f4(f21)]);
    if f20 != 0.0 {
        let a0 = m.r32(s1 + 0x18e0);
        let f12 = m.rf(T_NOW) + f20;
        m.call(0x0013_1a90, &[a0], &[f4(f12)]);
    }
    if s3 != -1 {
        fx_at_jt(m, s1, s3 as u32);
    }
    stack_pop(m, 0xb0);
}

// ---------------------------------------------------------------------------------------------------

const JT_MAIN: u32 = 0x002e_1e40; // main jt pointer
const V_ZERO: u32 = 0x002a_2dd0; // zero quadword

/// `*(o+0x40)` flag set: low bit clear -> it is a pointer, OR 0x10 into the pointee; else OR into the word
fn set_flag10<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let v1 = m.r32(a0 + 0x40);
    if (v1 ^ 1) & 1 != 0 {
        let v = m.r32(v1) | 0x10;
        m.w32(v1, v);
    } else {
        m.w32(a0 + 0x40, v1 | 0x10);
    }
}

/// FUN_00192bd8(attach, new sub-state): leave the old sub-state (+0xc; table 0x2a6150, old -1 also leaves
/// the attach state +8 via table 0x2a6180), enter the new one (table 0x2a61b0; -1 detaches via table
/// 0x2a61e0, 0/5 pick the anim by attach state via table 0x2a6210), then +0xc / +0x10 and (new != -1) the
/// anim / sound start (FUN_001305b8 / FUN_001f1448), vt slot 89 and FUN_0018c120.
pub fn ee_sub_192bd8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: i32) {
    let s2 = a0;
    let s4 = a1;
    let sp = stack_push(m, 0x110);
    let old = m.r32(s2 + 0xc) as i32;
    if s4 == old {
        stack_pop(m, 0x110);
        return;
    }
    // ---- leave old sub-state
    match old {
        1 => {
            let v1 = m.r32(s2);
            m.w32(v1 + 0x1038, u32::MAX);
        }
        -1 => {
            let a0 = m.r32(s2 + 0x14);
            if a0 != 0 && m.call(0x0013_2b10, &[a0, 0xa], &[]).0 != 0 {
                let jt = m.r32(s2);
                let s0 = m.r32(s2 + 0x14);
                let p = m.call(0x001a_62e8, &[jt], &[]).1;
                m.call(0x001e_6f60, &[s0, 0, 0, 0, s2 + 0x1c, s2 + 0x20], &[p]);
            }
            match m.r32(s2 + 8) {
                1 => {
                    let v1 = m.r32(s2 + 0x14);
                    let v0 = m.r32(s2);
                    m.w32(v1 + 0x190, v0);
                    m.call(0x0018_3b80, &[v1, 2], &[]);
                    let a0 = m.r32(s2);
                    if a0 == m.r32(JT_MAIN) {
                        let a1 = m.r32(a0 + 0x19a0);
                        vslot(m, a1, 8, &[], &[]);
                    }
                }
                2 => {
                    let s0 = m.call(0x001f_bb90, &[0x5c], &[]).0;
                    m.call(0x001f_b2f8, &[s0, 0], &[]);
                    let a0 = m.r32(JT_MAIN);
                    m.call(0x0025_8040, &[a0 + 0x2048, s0], &[]);
                }
                st @ (3 | 4) => {
                    let s0 = m.r32(s2 + 0x14);
                    vslot(m, s0, 2, &[5, 0, 0, 0, 0], &[]);
                    let a1 = m.r32(s2);
                    m.call(0x0013_33c0, &[s0, a1], &[]);
                    let a0 = m.r32(s2);
                    m.call(0x0020_4f78, &[a0, s0, 1], &[]);
                    if st == 4 {
                        let jt = m.r32(s2);
                        let p = m.call(0x001a_62e8, &[s0], &[]).1;
                        m.wv(jt + 0x230, p);
                    } else {
                        let a1 = m.r32(s2);
                        let f12 = m.rf(s2 + 0x1c);
                        m.w32(s2 + 0x28, u32::MAX);
                        m.call(0x001e_67b8, &[s0, a1 + 0x230, sp + 0x10], &[f4(f12)]);
                        m.w32(sp + 8, 0);
                        let vf03 = m.rv(sp);
                        let len = dot3(vf03).sqrt();
                        let v0q = m.rv(0x002a_2da0);
                        let vf01 = if len < 1e-4 { v0q } else { scl(vf03, 1.0 / len) };
                        m.wv(sp, vf01);
                        let vf01 = m.rv(sp);
                        m.wv(sp + 0x50, vf01);
                        let a0 = m.r32(s2);
                        let x = m.call(0x001a_6318, &[a0], &[]).0;
                        let vf02 = m.rv(x);
                        let vf01 = m.rv(sp + 0x50);
                        let f1 = m.rf(0x002b_f63c);
                        m.w32(s2 + 0x24, (dotp(vf01, vf02) < f1) as u32);
                    }
                }
                11 => {
                    let a0 = m.r32(s2 + 0x14);
                    if a0 != 0 {
                        let a1 = m.r32(s2);
                        m.call(0x0013_33c0, &[a0, a1], &[]);
                    }
                }
                12 => {
                    let a1 = m.r32(s2 + 0x14);
                    if a1 != 0 {
                        let a0 = m.r32(s2);
                        m.call(0x0020_46f0, &[a0, a1], &[]);
                    }
                    let a0 = m.r32(s2);
                    m.call(0x0020_4ca8, &[a0], &[]);
                }
                _ => {}
            }
            m.w32(s2 + 0x2c, u32::MAX);
            let v0 = m.r32(s2);
            let f0 = m.rf(T_NOW);
            m.wf(v0 + 0x20c, f0);
        }
        7 => {
            if (9..0xb).contains(&s4) {
                let s0 = m.r32(s2 + 0x14);
                let a0 = m.r32(s0 + 0x15c);
                if a0 != 0 {
                    let p = m.call(0x001a_62e8, &[a0], &[]).1;
                    let a0 = m.r32(s2);
                    let vf03 = p;
                    let vf01 = m.rv(a0 + 0x240);
                    let vf02 = sub(vf03, vf01);
                    m.wv(sp + 0x60, vf03);
                    vslot(m, a0, 28, &[], &[vf02]);
                    let v1 = m.r32(s2);
                    let vf03 = m.rv(sp + 0x60);
                    let a0w = m.r32(v1 + 0x1048);
                    let vf02 = gq(a0w);
                    let vf01 = m.rv(0x002a_2dc0);
                    m.wv(sp, vf02);
                    let vf03 = add(add(vf03, ONE_W), scl(vf01, vf02[0]));
                    m.wv(v1 + 0x1060, vf03);
                    let a0 = m.r32(s0 + 0x15c);
                    let x = m.call(0x001a_6318, &[a0], &[]).0;
                    let a1q = m.rv(x);
                    let a0 = m.r32(s2);
                    m.call(0x0020_46b0, &[a0], &[a1q]);
                    let v0 = m.r32(s2);
                    vslot(m, v0, 29, &[v0 + 0x1b0], &[]);
                    let a3 = m.r32(s2);
                    let z = m.rv(V_ZERO);
                    vslot(m, a3, 34, &[], &[z]);
                    let a2 = m.r32(s2);
                    let z = m.rv(V_ZERO);
                    vslot(m, a2, 35, &[], &[z]);
                }
                let a3 = m.r32(s2);
                let t0 = m.r32(a3 + 0x10c0);
                m.call(0x0014_e328, &[0x002d_df40, 0xa, 0x002d_e400, a3, t0], &[]);
            }
        }
        9 | 10 => {
            if !(9..0xb).contains(&s4) {
                m.call(0x0014_e4c8, &[0x002d_df40, 2, 0xa, 0x002d_e400, 0, 0], &[]);
            }
        }
        _ => {}
    }
    // ---- enter new sub-state
    let mut s3 = 0u32;
    let mut s5 = 0u32;
    let mut s1 = 1u32;
    let mut f20 = 0.0f32;
    let mut f21 = 1.0f32;
    let mut s6 = 0u32;
    // 0 = 0x1937ec (+0x190 = 0, then +0xc), 1 = 0x1937f0 (+0xc)
    let mut zero190 = false;
    match s4 {
        -1 => {
            let st = m.r32(s2 + 8);
            m.w32(s2 + 0x500, 0);
            match st {
                1 => {
                    let a0 = m.r32(s2 + 0x14);
                    m.call(0x0018_3b80, &[a0, 0], &[]);
                    let a0 = m.r32(s2);
                    if a0 == m.r32(JT_MAIN) {
                        let a1 = m.r32(a0 + 0x19a0);
                        vslot(m, a1, 7, &[], &[]);
                    }
                }
                2 => {
                    let v0 = m.r32(JT_MAIN);
                    let a0 = m.r32(v0 + 0x2048);
                    if a0 != 0 {
                        m.call(0x001f_b098, &[a0], &[f4(0.0)]);
                        let a0 = m.r32(JT_MAIN);
                        m.call(0x0025_8040, &[a0 + 0x2048, 0], &[]);
                    }
                }
                3 | 4 => {
                    let s0 = m.r32(s2 + 0x14);
                    vslot(m, s0, 2, &[6, 0, 0, 0, 0], &[]);
                    let a1 = m.r32(s2);
                    m.call(0x0013_3448, &[s0, a1], &[]);
                    let a0 = m.r32(s2);
                    m.call(0x0020_4f78, &[a0, s0, 0], &[]);
                }
                5 | 9 => {
                    m.call(0x001c_cac0, &[s2 + 0x440, u32::MAX], &[]);
                }
                6 => {
                    let a0 = m.r32(s2);
                    if m.r32(a0 + 0x18d8) == 0 {
                        m.call(0x0018_e540, &[a0, u32::MAX], &[]);
                    }
                }
                11 => {
                    let a0 = m.r32(s2 + 0x14);
                    if a0 != 0 {
                        let a1 = m.r32(s2);
                        m.call(0x0013_3448, &[a0, a1], &[]);
                    }
                    let v0 = m.r32(s2);
                    m.w32(v0 + 0x130c, 0);
                    let v1 = m.r32(s2);
                    m.w32(v1 + 0x9bc, u32::MAX);
                }
                12 => {
                    let v1 = m.r32(s2);
                    m.w32(v1 + 0x1318, u32::MAX);
                    let a0 = m.r32(s2);
                    m.call(0x001c_d2c0, &[a0 + 0x900], &[]);
                    let a0 = m.r32(s2);
                    if a0 == m.r32(JT_MAIN) {
                        m.w32(a0 + 0x1bcc, 0);
                    }
                }
                _ => {}
            }
            // 0x193258
            let a0 = m.r32(s2);
            let a1 = m.r32(a0 + 0x1024);
            if a1 != 0 {
                if a1 == m.r32(s2 + 0x68) {
                    m.call(0x0014_0c88, &[a0, 0], &[]);
                } else {
                    let v0 = m.r32(s2 + 0x18);
                    if a1 == m.r32(v0) {
                        m.call(0x0014_0c88, &[a0, 0], &[]);
                    }
                }
            }
            let v0 = m.r32(s2);
            m.w32(v0 + 0x1038, u32::MAX);
            m.wf(s2 + 0x1c, 0.0);
            m.w32(s2 + 0x4fc, u32::MAX);
            m.w32(s2 + 8, u32::MAX);
            m.w32(s2 + 0xc, u32::MAX);
            m.w32(s2 + 0x14, 0);
            m.w32(s2 + 0x18, 0);
            m.wf(s2 + 0x20, 0.0);
            m.w32(s2 + 0x34, 0);
        }
        0 | 5 => {
            zero190 = true;
            match m.r32(s2 + 8) {
                0 => {
                    s3 = m.r32(s2 + 0x174);
                    s1 = 9;
                }
                1 => {
                    s5 = m.r32(s2 + 0x230);
                    s1 = 1;
                }
                2 => {
                    s3 = m.r32(s2 + 0x514);
                    s1 = 9;
                }
                3 => {
                    let v0 = m.r32(s2 + 0x14);
                    if m.r32(v0 + 0x1bc) == 0 {
                        s3 = m.r32(s2 + 0x238);
                    } else {
                        let a0 = m.r32(s2 + 0x314);
                        if a0 != 0 {
                            s3 = a0;
                            m.wf(s2 + 0x318, 0.0);
                            m.wf(s2 + 0x31c, 0.0);
                        } else {
                            s3 = m.r32(s2 + 0x238);
                        }
                    }
                    s1 = 9;
                    s6 = 0x1f;
                }
                4 => {
                    s3 = m.r32(s2 + 0x234);
                    s1 = 9;
                    s6 = 7;
                }
                5 | 9 => {
                    let v0 = m.r32(s2 + 0x14);
                    f21 = 0.0;
                    m.w32(s2 + 0x4c4, v0);
                    m.call(0x001c_cac0, &[s2 + 0x440, 0], &[]);
                    s3 = m.r32(s2 + 0x470);
                    f20 = m.rf(s3 + 0x74) * 0.5;
                }
                6 | 7 => {
                    s3 = m.r32(s2 + 0x320);
                    s1 = 9;
                }
                10 => {
                    let a0 = m.r32(s2);
                    let f1 = m.rf(0x002b_f640);
                    let o = m.r32(a0 + 0x150);
                    s3 = if f1 <= m.rf(o + 0x1c) { m.r32(s2 + 0x4f0) } else { m.r32(s2 + 0x4f4) };
                }
                11 => {
                    if m.r32(s2 + 0x4fc) != 0x15 {
                        s3 = m.r32(s2 + 0x4f8);
                    } else {
                        let v1 = m.r32(0x002f_7900);
                        s3 = m.r32(v1 + 0x1af0);
                        if s3 == 0 {
                            s3 = m.r32(s2 + 0x4f8);
                        }
                    }
                }
                12 => s3 = m.r32(s2 + 0x50c),
                13 => {
                    s3 = if s4 != 0 { m.r32(s2 + 0x174) } else { m.r32(s2 + 0x510) };
                    s1 = 9;
                }
                _ => {}
            }
        }
        1 => {
            let v1 = m.r32(s2 + 8);
            if v1 != 0 {
                s1 = 9;
                if v1 == 0xd {
                    s3 = m.r32(s2 + 0x170);
                }
            } else {
                let jt = m.r32(s2);
                s3 = m.r32(s2 + 0x170);
                if m.r32(jt + 0x1024) == 0 {
                    // 0 = 0x193504 (FUN_00140c88(jt, a1)), 1 = 0x193580 (FUN_00140c88(jt, +0x68))
                    let mut tail: Option<u32> = None;
                    let a0 = m.r32(s2 + 0x14);
                    if m.call(0x0013_2b10, &[a0, 0xc], &[]).0 != 0 {
                        let s0 = m.r32(s2 + 0x14);
                        let a1 = m.r32(s0 + 0x190);
                        if a1 != 0 {
                            tail = Some(a1);
                        } else {
                            m.call(0x0019_4118, &[s2], &[]);
                            let v0 = m.r32(s2 + 0x68);
                            m.w32(v0 + 0x10c, s0);
                        }
                    } else {
                        let a0 = m.r32(s2 + 0x14);
                        if m.call(0x0013_2b10, &[a0, 0x22], &[]).0 != 0 {
                            let o = m.r32(s2 + 0x14);
                            let a1 = m.r32(o + 0xe0);
                            if a1 != 0 {
                                tail = Some(a1);
                            } else {
                                m.call(0x0019_4118, &[s2], &[]);
                                let v1 = m.r32(s2 + 0x68);
                                m.w32(v1 + 0x12c, 1);
                                let s0 = m.r32(s2 + 0x68);
                                let p = m.call(0x001a_62e8, &[o], &[]).1;
                                let a1 = m.r32(s2 + 0x68);
                                let q = m.call(0x001a_6f00, &[0, a1], &[p]).1;
                                m.wv(s0 + 0x150, q);
                                let f0 = m.rf(o + 0x10c);
                                let v0 = m.r32(s2 + 0x68);
                                m.wf(v0 + 0x160, f0);
                            }
                        } else {
                            let a0 = m.r32(s2 + 0x14);
                            if m.call(0x0013_2b10, &[a0, 0x26], &[]).0 == 0 {
                                s1 = 9;
                                tail = Some(u32::MAX); // straight to 0x1937f0
                            } else {
                                let s0 = m.r32(s2 + 0x14);
                                let a1 = m.r32(s0 + 0xe0);
                                if a1 != 0 {
                                    tail = Some(a1);
                                } else {
                                    m.call(0x0019_4118, &[s2], &[]);
                                    let v0 = m.call(0x001a_6318, &[s0], &[]).0;
                                    let a0 = m.r32(s2 + 0x68);
                                    let vf03 = m.rv(v0 + 0x20);
                                    let vf04 = m.rv(0x002b_f650);
                                    let vf02 = m.rv(v0);
                                    let vf01 = m.rv(v0 + 0x10);
                                    let vf05 = mx3([0.0; 4], vf02, vf01, vf03, vf04);
                                    let v1 = m.r32(a0 + 0x114);
                                    m.wf(v1 + 0x10, vf05[0]);
                                    let a0 = m.r32(s2 + 0x68);
                                    let v1 = m.r32(a0 + 0x114);
                                    m.wf(v1 + 0x20, vf05[1]);
                                    let v0 = m.r32(s2 + 0x68);
                                    let v1 = m.r32(v0 + 0x114);
                                    m.wf(v1 + 0x30, vf05[2]);
                                }
                            }
                        }
                    }
                    match tail {
                        Some(u32::MAX) => {}
                        Some(a1) => {
                            let a0 = m.r32(s2);
                            m.call(0x0014_0c88, &[a0, a1], &[]);
                        }
                        None => {
                            let a0 = m.r32(s2);
                            let a1 = m.r32(s2 + 0x68);
                            m.call(0x0014_0c88, &[a0, a1], &[]);
                        }
                    }
                }
                s1 = 9;
            }
        }
        2 => {
            s3 = m.r32(s2 + 0x178);
            zero190 = true;
        }
        3 => {
            s3 = m.r32(s2 + 0x17c);
            zero190 = true;
        }
        4 => {
            let v1 = m.r32(s2 + 8);
            s1 = 9;
            match v1 {
                1 => {
                    s3 = m.r32(s2 + 0x184);
                    m.call(0x0020_bd68, &[s2 + 0x180], &[]);
                }
                2 => {
                    let v0 = m.r32(s2);
                    s3 = m.r32(v0 + 0x1c34);
                    m.call(0x0020_bd68, &[v0 + 0x1c30], &[]);
                }
                3 => {
                    s3 = m.r32(s2 + 0x244);
                    m.call(0x0020_bd68, &[s2 + 0x240], &[]);
                }
                _ => {}
            }
        }
        7 => {
            let s0 = m.r32(s2 + 0x14);
            let a0 = m.r32(s0 + 0x15c);
            s3 = m.r32(s2 + 0x324);
            if a0 != 0 {
                let v0 = m.call(0x001a_6318, &[a0], &[]).0;
                let a1q = m.rv(v0);
                m.wv(sp + 0x20, a1q);
                cp16(m, sp + 0x30, v0 + 0x10);
                cp16(m, sp + 0x40, v0 + 0x20);
                let a0 = m.r32(s2);
                m.call(0x0020_46b0, &[a0], &[a1q]);
            }
            let a0 = m.r32(s2);
            cp16(m, a0 + 0x190, V_ZERO);
            vslot(m, s0, 2, &[5, 0, 0, 0, 0], &[]);
        }
        8 => {
            let a0 = m.r32(s2);
            s3 = m.r32(s2 + 0x328);
            cp16(m, a0 + 0x190, V_ZERO);
            let t2 = m.r32(s2 + 0x14);
            vslot(m, t2, 2, &[6, 0, 0, 0, 0], &[]);
        }
        9 => {
            let v1 = m.r32(s2 + 0xc);
            s5 = m.r32(s2 + 0x32c);
            if v1 == 7 {
                let a0 = m.r32(s2);
                set_flag10(m, a0);
                if a0 == m.r32(JT_MAIN) {
                    let o = m.r32(a0 + 0x19a0);
                    set_flag10(m, o);
                    let j = m.r32(JT_MAIN);
                    let o = m.r32(j + 0x19a4);
                    set_flag10(m, o);
                }
            }
            zero190 = true;
        }
        10 => {
            let a0 = m.r32(s2);
            s3 = m.r32(s2 + 0x430);
            m.w32(a0 + 0x1038, u32::MAX);
            s1 = 9;
            zero190 = true;
        }
        _ => {}
    }
    if zero190 {
        let a0 = m.r32(s2);
        cp16(m, a0 + 0x190, V_ZERO);
    }
    m.w32(s2 + 0xc, s4 as u32);
    let f0 = m.rf(T_NOW);
    m.wf(s2 + 0x10, f0);
    if s4 != -1 {
        let a0 = m.r32(s2);
        m.call(0x0013_0658, &[a0 + 0x1030], &[]);
        let a0 = m.r32(s2);
        m.call(0x001f_15b8, &[a0 + 0x1034], &[]);
        let a1 = m.r32(s2);
        m.call(0x0013_05b8, &[s3, a1, s1, a1 + 0x1030], &[f4(f20), f4(f21)]);
        let a1 = m.r32(s2);
        m.call(0x001f_1448, &[s5, a1, u32::MAX, s1, a1 + 0x1034], &[]);
        let a1 = m.r32(s2);
        vslot(m, a1, 89, &[], &[]);
        let a0 = m.r32(s2);
        if m.call(0x0013_2b10, &[a0, 0x35], &[]).0 != 0 {
            let a0 = m.r32(s2);
            m.call(0x0018_c120, &[a0, s6], &[]);
        }
    }
    stack_pop(m, 0x110);
}

// ---------------------------------------------------------------------------------------------------

const PI_2: f32 = f32::from_bits(0x3fc9_0fdb);
const PI: f32 = f32::from_bits(0x4049_0fdb);

/// `vf01 = |v.xyz| < 1e-4 ? fallback : v * (1/|v|)` (fallback = *0x2a2da0)
fn norm_or<M: QMem + ?Sized>(m: &mut M, v: V4) -> V4 {
    let len = dot3(v).sqrt();
    let fb = m.rv(0x002a_2da0);
    if len < 1e-4 {
        fb
    } else {
        scl(v, 1.0 / len)
    }
}

fn clampf(x: f32, lo: f32, hi: f32) -> f32 {
    if x < lo {
        lo
    } else if hi < x {
        hi
    } else {
        x
    }
}

/// 0x191a90 (attach state 0 / 13): rail/rope "hang & shimmy": project the stick onto the attach (FUN_00182cd8,
/// FUN_0011f088), velocity +0x190 in jt space, then sub-state 0/1/2/3/5 by speed / end-stop / stick.
fn p_191a90<M: QMem + ?Sized>(m: &mut M, s1: u32, sp: u32, s2_in: u32) -> u32 {
    let mut s2 = s2_in;
    let (a0, a1) = (m.r32(s1 + 0x18), m.r32(s1));
    m.call(0x0018_2cd8, &[a0, a1, s1 + 0x20, sp + 0x20, sp + 0x80], &[]);
    let a0 = m.r32(s1);
    let f22;
    if m.r32(a0 + 0x1040) == 1 {
        let v0 = m.r32(a0 + 0x1030);
        let v = m.rv(s1 + 0xc0);
        f22 = dot3(v).sqrt() * m.rf(v0 + 0x40);
    } else {
        let v = m.rv(s1 + 0x100);
        let v0 = m.r32(a0 + 0x1030);
        f22 = dot3(v).sqrt() * m.rf(v0 + 0x40);
    }
    let f0 = m.rf(sp + 8);
    m.wf(sp + 0x28, f0);
    let a0q = m.rv(sp);
    let f12 = m.rf(T_DT);
    let s3 = sp + 0x30;
    let a1q = m.rv(sp + 0x20);
    m.wf(sp + 0x38, f32::from_bits(0x3e99_999a));
    m.wf(sp + 0x30, f22);
    m.w32(sp + 0x34, 0);
    m.call(0x0011_f088, &[sp + 0x30, sp + 0x40], &[a0q, a1q, f4(f12)]);
    let mut f21;
    let vf01;
    let a0 = m.r32(s1 + 0x14);
    if m.call(0x0013_2b10, &[a0, 0x26], &[]).0 != 0 {
        vf01 = [0.0; 4];
        f21 = 0.0;
    } else {
        let v0 = m.r32(s1);
        let f1 = m.rf(sp + 0x80);
        let f12 = if m.r32(v0 + 0x1040) == 1 { f1 - PI_2 } else { f1 + PI_2 };
        f21 = 1.0;
        let f20 = 0.0f32;
        let v = m.call(0x0011_e1a0, &[], &[f4(f12), f4(f21), f4(f20)]).1;
        m.wv(sp + 0x50, v);
        let f12 = m.rf(s2 + 0x18);
        let vf02 = m.call(0x0011_e1a0, &[], &[f4(f12), f4(f21), f4(f20)]).1;
        let a = m.rv(sp + 0x50);
        let d = dotp(a, vf02);
        let f1 = m.rf(0x002b_f5bc) * d;
        m.wv(sp + 0x60, vf02);
        let mut f3 = if f1 < -1.0 {
            -1.0
        } else if f21 < f1 {
            f21
        } else {
            f1
        };
        f3 *= m.rf(s2 + 0x1c);
        if f3.abs() < 0.2 {
            f3 = 0.0;
        }
        let v40 = m.rv(sp + 0x40);
        let f2 = f22 * f22 - dot3(v40);
        f21 = f3 * f22;
        let f1 = f2.sqrt();
        f21 = if f1 < f21 { f1 } else { f21 };
        let a0 = m.r32(s1 + 0x14);
        let mut f0 = f21;
        if m.call(0x0013_2b10, &[a0, 0xc], &[]).0 != 0 {
            s2 = m.r32(s1 + 0x14);
            if m.r32(s2 + 0xe4) == 0 {
                let f20 = 30.0f32;
                let g = m.call(0x001e_6620, &[s2], &[]).1[0];
                let (f12, f14) = (m.rf(s1 + 0x20), m.rf(T_DT));
                m.call(0x0011_d018, &[s3, sp + 0x84], &[f4(f12), f4(g + f20), f4(f14)]);
                let g = m.call(0x001e_6648, &[s2], &[]).1[0];
                let (f14, f12) = (m.rf(T_DT), m.rf(s1 + 0x20));
                m.call(0x0011_d018, &[s3, sp + 0x88], &[f4(f12), f4(g - f20), f4(f14)]);
                let lo = m.rf(sp + 0x84);
                let hi = m.rf(sp + 0x88);
                f0 = clampf(f21, lo, hi);
            }
        }
        let q = gqf(f0);
        let vf02 = m.rv(sp + 0x50);
        m.wv(sp + 0x70, q);
        vf01 = scl(vf02, q[0]);
    }
    // 0x191d68
    let v1 = m.r32(s1);
    let vf03 = add(m.rv(sp + 0x40), vf01);
    let vf02 = m.rv(v1 + 0x1c0);
    let vf04 = m.rv(v1 + 0x1d0);
    let r0 = m.rv(v1 + 0x1b0);
    let mul = |a: V4| [a[0] * vf03[0], a[1] * vf03[1], a[2] * vf03[2], a[3] * vf03[3]];
    let (v8, v7, v6) = (mul(r0), mul(vf02), mul(vf04));
    let mut o = r0;
    o[0] = v8[0] + v8[1] + v8[2];
    o[1] = v7[1] + v7[0] + v7[2];
    o[2] = v6[2] + v6[0] + v6[1];
    let vf05 = gqf(-1.0);
    m.wv(sp + 0x50, vf05);
    m.wv(v1 + 0x190, scl(o, vf05[0]));
    let v1 = m.r32(s1);
    let vel = m.rv(v1 + 0x190);
    let mut s0: u32;
    // true = 0x191f9c/0x191fa0 directly, false = through 0x191f88
    if 25.0 < dot3(vel).sqrt() {
        s0 = 1;
        if m.r32(s1 + 8) == 0 {
            let v40 = m.rv(sp + 0x40);
            if 45.0 < dot3(v40).sqrt() {
                m.call(0x0019_40b0, &[s1, 0], &[]);
            } else {
                let f1 = m.rf(v1 + 0x194);
                let side = m.r32(v1 + 0x1040) == 1;
                if 20.0 < f1 {
                    m.call(0x0019_40b0, &[s1, if side { 1 } else { 2 }], &[]);
                } else if f1 < -20.0 {
                    m.call(0x0019_40b0, &[s1, if side { 2 } else { 1 }], &[]);
                }
            }
        }
    } else {
        s0 = 0;
        let a0 = m.r32(s1 + 0x14);
        if m.call(0x0013_2b10, &[a0, 0xc], &[]).0 != 0 {
            let v1 = m.r32(s1 + 0x14);
            let mut alt: Option<u32> = None;
            if f21 < -25.0 && m.r32(v1 + 0x1bc) & 1 != 0 {
                s0 = 2;
                if m.r32(s1 + 8) == 0 {
                    alt = Some(3);
                }
            } else if 25.0 < f21 && m.r32(v1 + 0x1bc) & 2 != 0 {
                s0 = 3;
                if m.r32(s1 + 8) == 0 {
                    alt = Some(2);
                }
            }
            if let Some(a0) = alt {
                let v0 = m.r32(s1);
                if m.r32(v0 + 0x1040) == 0 {
                    s0 = a0;
                }
            }
        }
        if m.r32(s1 + 8) == 0 {
            m.call(0x0019_40b0, &[s1, 3], &[]);
        }
    }
    let a0 = m.r32(s1);
    let f12 = m.rf(sp + 0x80);
    m.call(0x0020_4660, &[a0], &[f4(f12)]);
    if m.r32(s1 + 0xc) == 5 {
        let a0 = m.r32(s1);
        let f0 = m.call(0x0020_4740, &[a0], &[]).1[0];
        let f12 = f0 - m.rf(sp + 0x80);
        let f0 = m.call(0x0011_cf60, &[], &[f4(f12)]).1[0];
        if 0.25 < f0.abs() {
            s0 = 5;
        }
    }
    m.call(0x0019_2bd8, &[s1, s0], &[]);
    m.r32(s1 + 0xc)
}

/// 0x19210c (attach state 3): pole/swing climb: FUN_001e6ed8 sample, end-stop exit (FUN_00195a00), side
/// selection (+0x28 / +0x24) from the stick vs. pole direction, climb speed -> sub-state 4 / 0.
fn p_19210c<M: QMem + ?Sized>(m: &mut M, s1: u32, sp: u32, s2: u32) -> u32 {
    let v0 = m.r32(s1);
    let s0 = m.r32(s1 + 0x14);
    let v1 = m.r32(v0 + 0x218);
    let f12 = m.rf(s1 + 0x1c);
    let a1q = m.rv(v1);
    m.call(0x001e_6ed8, &[s0, 0, sp + 0x20, sp + 0x30, s1 + 0x1c, s1 + 0x20], &[a1q, f4(f12)]);
    if m.r32(s0 + 0x1bc) != 0 && m.r32(s0 + 0xe4) == 0 {
        let f20 = f32::from_bits(0x3c23_d70a);
        let f0 = m.call(0x001e_6610, &[s0], &[]).1[0] + f20;
        let mut exit = m.rf(s1 + 0x1c) < f0;
        if !exit {
            let f0 = m.call(0x001e_6630, &[s0], &[]).1[0] - f20;
            exit = f0 < m.rf(s1 + 0x1c);
        }
        if exit {
            let a0 = m.r32(s1);
            m.call(0x0019_5a00, &[a0 + 0x2c0, 0, 0, 0], &[]);
            return m.r32(s1 + 0xc);
        }
    }
    m.w32(sp + 0x38, 0);
    let v = m.rv(sp + 0x30);
    let v = norm_or(m, v);
    m.wv(sp + 0x30, v);
    let (f12, f13) = (m.rf(sp + 0x34), m.rf(sp + 0x30));
    let f0 = m.call(0x0024_58f0, &[], &[f4(f12), f4(f13)]).1[0];
    let f12 = m.rf(s2 + 0x18);
    let mut f21 = f0;
    let v = m.call(0x0011_e1a0, &[], &[f4(f12), f4(1.0), f4(0.0)]).1;
    m.wv(sp + 0x50, v);
    let a0 = m.r32(s0 + 0x1c8);
    if a0 == 0 {
        cp16(m, sp + 0x40, sp + 0x30);
    } else {
        let v0 = m.r32(s1);
        let v1 = m.r32(v0 + 0x218);
        let q = m.rv(v1);
        m.call(0x001e_6f60, &[a0, 0, 0, sp + 0x40, 0, 0], &[q]);
    }
    m.w32(sp + 0x48, 0);
    let v = m.rv(sp + 0x40);
    let vf03 = norm_or(m, v);
    let vf01 = m.rv(sp + 0x50);
    let mut f20 = dotp(vf01, vf03);
    let v1 = m.r32(s1 + 0x28) as i32;
    m.wv(sp + 0x40, vf03);
    match v1 {
        0 => {
            let f1 = m.rf(s2 + 0x1c);
            if f1 < 0.25 {
                m.w32(s1 + 0x28, u32::MAX);
            } else if f20 < f32::from_bits(0xbf33_3333) {
                m.w32(s1 + 0x28, 1);
            }
        }
        1 => {
            let f1 = m.rf(s2 + 0x1c);
            if f1 < 0.25 {
                m.w32(s1 + 0x28, u32::MAX);
            } else if f32::from_bits(0x3f33_3333) < f20 {
                m.w32(s1 + 0x28, 0);
            }
        }
        -1 => {
            let f1 = m.rf(s2 + 0x1c);
            if f32::from_bits(0x3eb3_3333) < f1 {
                m.w32(s1 + 0x28, if 0.0 <= f20 { 0 } else { 1 });
            }
        }
        _ => {}
    }
    let v1 = m.r32(s1 + 0x28);
    if v1 == 0 {
        m.w32(s1 + 0x24, 0);
    } else if v1 == 1 {
        m.w32(s1 + 0x24, v1);
    }
    if m.r32(s0 + 0x1bc) != 0 {
        f20 = 0.0;
    } else {
        let f0 = m.rf(s1 + 0x260);
        let f2 = m.rf(s1 + 0x2a0);
        let f1 = m.rf(0x002b_f5c8);
        m.wf(sp + 0x64, f0);
        m.wf(sp + 0x68, f1);
        m.wf(sp + 0x60, f2);
        if m.r32(s0 + 0xe4) != 0 {
            m.wf(sp + 0x8c, f2);
        } else if m.r32(s1 + 0x24) != 0 {
            let (f12, f14) = (m.rf(s1 + 0x20), m.rf(T_DT));
            m.call(0x0011_d018, &[sp + 0x60, sp + 0x8c], &[f4(f12), f4(50.0), f4(f14)]);
            let f1 = -m.rf(sp + 0x8c);
            m.wf(sp + 0x8c, f1);
        } else {
            let f0 = m.call(0x001e_6648, &[s0], &[]).1[0];
            let (f14, f12) = (m.rf(T_DT), m.rf(s1 + 0x20));
            m.call(0x0011_d018, &[sp + 0x60, sp + 0x8c], &[f4(f12), f4(f0 - 50.0), f4(f14)]);
        }
        let f2 = m.rf(sp + 0x64);
        let f1 = m.rf(sp + 0x8c);
        let f0 = m.rf(sp + 0x60);
        let f12 = m.rf(s2 + 0x1c);
        let f1 = (f1 - f2) / (f0 - f2);
        f20 = if f12 < 0.0 {
            0.0
        } else if f1 < f12 {
            f1
        } else {
            f12
        };
    }
    if m.r32(s1 + 0x24) != 0 {
        f21 = m.call(0x0011_cf60, &[], &[f4(f21 + PI)]).1[0];
    }
    if 0.0 < f20 {
        m.call(0x0019_2bd8, &[s1, 4], &[]);
        let s0 = s1 + 0x240;
        m.call(0x0020_bb58, &[s0], &[f4(f20)]);
        let a0 = m.r32(s1);
        m.call(0x0020_4660, &[a0], &[f4(f21)]);
        m.call(0x0020_bc08, &[s0], &[]);
    } else {
        m.call(0x0019_2bd8, &[s1, 0], &[]);
        let a0 = m.r32(s1);
        m.call(0x0020_4660, &[a0], &[f4(f21)]);
    }
    m.r32(s1 + 0xc)
}

/// FUN_00191a10(attach): per-frame attach-state update (table 0x2a60d0 on the attach state +8), then the
/// sub-state epilogue (0: state 3 pipe bones FUN_00189c40 / FUN_00193c98; 1: FUN_001a0450 + +0x50 max).
pub fn ee_sub_191a10<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let s1 = a0;
    let sp = stack_push(m, 0x100);
    let jt = m.r32(s1);
    let p = m.call(0x001a_62e8, &[jt], &[]).1;
    m.wv(sp, p);
    let a1 = m.r32(s1);
    let q = vslot(m, a1, 32, &[], &[]).1;
    let st = m.r32(s1 + 8);
    m.wv(sp + 0x10, q);
    let jt = m.r32(s1);
    let s2 = m.r32(jt + 0x150);
    let s0: u32 = match st {
        0 | 13 => p_191a90(m, s1, sp, s2),
        1 | 2 => {
            let mut go4 = false;
            if st == 1 {
                let v0 = m.r32(s1 + 0x14);
                if m.r32(v0 + 0x18c) != 0 && 0.0 < m.rf(s2 + 0x1c) && m.r32(s1 + 0x184) != 0 {
                    m.call(0x0019_2bd8, &[s1, 4], &[]);
                    let s0 = s1 + 0x180;
                    let f12 = m.rf(s2 + 0x1c);
                    m.call(0x0020_bb58, &[s0], &[f4(f12)]);
                    let a0 = m.r32(s1);
                    let o = m.r32(a0 + 0x150);
                    let f12 = m.rf(o + 0x18);
                    m.call(0x0020_4660, &[a0], &[f4(f12)]);
                    m.call(0x0020_bc08, &[s0], &[]);
                    go4 = true;
                }
            } else {
                let a0 = m.r32(s1);
                let f12 = m.rf(0x002b_f5c0);
                m.call(0x0014_41e8, &[a0], &[f4(f12)]);
                let f1 = m.rf(s2 + 0x1c);
                let thr = if m.r32(s1 + 0xc) == 0 { 0.1 } else { 0.0 };
                if thr < f1 {
                    m.call(0x0019_2bd8, &[s1, 4], &[]);
                    let f12 = m.rf(s2 + 0x1c) * m.rf(0x002b_f5c4);
                    let a0 = m.r32(s1);
                    m.call(0x0020_bb58, &[a0 + 0x1c30], &[f4(f12)]);
                    let f12 = m.rf(s2 + 0x18);
                    let a0 = m.r32(s1);
                    m.call(0x0020_4660, &[a0], &[f4(f12)]);
                    go4 = true;
                }
            }
            if !go4 {
                m.call(0x0019_2bd8, &[s1, 0], &[]);
            }
            m.r32(s1 + 0xc)
        }
        3 => p_19210c(m, s1, sp, s2),
        4 => {
            let f12 = m.rf(s2 + 0x18);
            let a0 = m.r32(s1);
            m.call(0x0020_4660, &[a0], &[f4(f12)]);
            m.r32(s1 + 0xc)
        }
        5 | 9 => {
            m.call(0x001c_c2e8, &[s1 + 0x440], &[]);
            m.r32(s1 + 0xc)
        }
        8 => {
            let v1 = m.r32(s1 + 0xc) as i32;
            if !(7..9).contains(&v1) {
                let f0 = m.rf(s2 + 0x1c);
                let f20 = 0.0f32;
                if f20 < f0 {
                    m.call(0x0019_2bd8, &[s1, 0xa], &[]);
                    let v0 = if m.r32(s1 + 0x394) == 0 { s1 + 0x370 } else { s1 + 0x390 };
                    let f2 = -m.rf(s2 + 0x14);
                    let vf02 = m.rv(s1 + 0x360);
                    let vf01 = m.rv(v0 + 0x10);
                    let l1 = (vf01[0] * vf01[0] + vf01[1] * vf01[1]).sqrt();
                    let l2 = (vf02[0] * vf02[0] + vf02[1] * vf02[1]).sqrt();
                    let f3 = m.rf(s2 + 0x10);
                    let f2 = f2 * l1;
                    let jt = m.r32(s1);
                    let f3 = f3 * l2;
                    m.wf(jt + 0x190, f2);
                    m.wf(jt + 0x198, f20);
                    m.wf(jt + 0x194, f3);
                    let (a0, a2) = (m.r32(s1), m.r32(s1 + 0x430));
                    m.call(0x001a_0450, &[a0, s1 + 0x330, a2], &[]);
                    let a0 = m.r32(s1);
                    let f12 = -m.rf(a0 + 0x194);
                    let f0 = m.rf(0x002b_f5cc);
                    let vf02 = m.rv(0x002a_2da0);
                    let vf01 = m.rv(a0 + 0x190);
                    let d = dotp(vf01, vf02);
                    let f12 = f12 / f0;
                    m.wv(a0 + 0x190, scl(vf02, d));
                    let f1 = m.rf(T_DT);
                    let a0 = m.r32(s1);
                    let f12 = f12 * f1;
                    let f0 = m.rf(a0 + 0x1a8);
                    m.call(0x0020_4660, &[a0], &[f4(f0 + f12)]);
                } else {
                    m.call(0x0019_2bd8, &[s1, 9], &[]);
                }
                let a0 = m.r32(s1);
                let a3 = if a0 == m.r32(JT_MAIN) { 0x002b_f5d0 } else { 0x002b_f600 };
                m.call(0x0020_5910, &[a0, 0x002b_f630, 3, a3], &[]);
            }
            m.r32(s1 + 0xc)
        }
        10 => {
            m.w32(sp + 0x28, 0);
            m.w32(sp + 0x24, 0);
            m.w32(sp + 0x20, 0);
            let a1 = m.r32(s1);
            let s0 = m.r32(a1 + 0x1030);
            if s0 != 0 {
                let a0 = m.r32(s0 + 0x14);
                let v0 = m.call(0x0012_ff10, &[a0, a1, 1, 1], &[]).0;
                if v0 != 0 {
                    let a3 = m.r32(v0 + 4);
                    let f13 = m.rf(s0 + 0x40);
                    let vt = m.r32(a3);
                    let f12 = m.rf(s0 + 0x1c);
                    let adj = m.r16(vt + 0x18) as i16 as i32;
                    let f = m.r32(vt + 0x1c);
                    m.call(f, &[a3.wrapping_add(adj as u32), 0, sp + 0x20], &[f4(f12), f4(f13)]);
                    let vf01 = m.rv(sp + 0x20);
                    let vf02 = gqf(-1.0);
                    m.wv(sp + 0x30, vf02);
                    m.wv(sp + 0x20, scl(vf01, vf02[0]));
                }
            }
            let v1 = m.r32(s1);
            cp16(m, v1 + 0x190, sp + 0x20);
            m.r32(s1 + 0xc)
        }
        11 => {
            let a1 = m.r32(s1 + 0x14);
            if a1 != 0 {
                let a0 = m.r32(s1);
                m.call(0x0020_46f0, &[a0, a1], &[]);
                let a0 = m.r32(s1);
                m.call(0x0020_4ca8, &[a0], &[]);
            }
            m.r32(s1 + 0xc)
        }
        _ => m.r32(s1 + 0xc),
    };
    if s0 == 0 {
        if m.r32(s1 + 8) == 3 {
            let s0 = m.r32(s1);
            let f20 = 5.0f32;
            let s2 = s0 + 0x1520;
            let (a1, a2, a3) = (m.r32(s0 + 0x19d8), m.r32(s0 + 0x19e0), m.r32(s0 + 0x19f0));
            m.call(0x0018_9c40, &[s0, a1, a2, a3, s2], &[f4(f20)]);
            let s1b = s0 + 0x1620;
            let (a1, a2, a3) = (m.r32(s0 + 0x19d4), m.r32(s0 + 0x19dc), m.r32(s0 + 0x19f4));
            m.call(0x0018_9c40, &[s0, a1, a2, a3, s1b], &[f4(f20)]);
            let a2 = m.r32(s0 + 0x19f0);
            m.call(0x0019_3c98, &[s0, s2, a2], &[]);
            let a2 = m.r32(s0 + 0x19f4);
            m.call(0x0019_3c98, &[s0, s1b, a2], &[]);
        }
    } else if s0 == 1 {
        let (a0, a2) = (m.r32(s1), m.r32(s1 + 0x170));
        m.call(0x001a_0450, &[a0, s1 + 0x70, a2], &[]);
        let v0 = m.r32(s1);
        let a0 = m.r32(s1 + 0x68);
        if m.r32(v0 + 0x1024) == a0 {
            let v1 = m.r32(s1 + 0x2c) as i32;
            if v1 < 3 && v1 > 0 {
                let f0 = m.rf(s1 + 0x50);
                let f1 = if v1 != 1 { m.rf(s1 + 0x70) } else { m.rf(s1 + 0x90) };
                m.wf(s1 + 0x50, if f0 < f1 { f1 } else { f0 });
            }
        }
    }
    stack_pop(m, 0x100);
}

// ---------------------------------------------------------------------------------------------------

/// FUN_0018c250(jt, new state, new sub-state): player state machine transition. Leaves the old state
/// +0x18c8 (table 0x2a5b50), enters the new one (table 0x2a5c50), then the sub-state +0x18d0 (table
/// 0x2a5cc0 on sub+1), stores both with timestamps, vt slot 89, FUN_0018c120, optional fx, and finally
/// restarts the anim (+0x1030 via FUN_0012f9b8) or sound (+0x1034 via FUN_001f03a8).
pub fn ee_sub_18c250<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: i32) {
    let s1 = a0;
    let s3 = a1;
    let s5 = a2;
    let sp = stack_push(m, 0x170);
    let mut s4: u32 = 0;
    let mut s6: u32 = 0;
    let mut s7: u32 = u32::MAX;
    let mut fp: u32 = 1;
    let mut f22: f32 = 0.0;
    let mut f23: f32 = 1.0;
    m.w32(sp + 0x54, u32::MAX);
    m.w32(sp + 0x58, 0);
    m.w32(sp + 0x5c, 0xfffc);
    m.w32(sp + 0x60, 0);
    let same_state = m.r32(s1 + 0x18c8) == s3;
    if same_state && m.r32(s1 + 0x18d0) as i32 == s5 {
        stack_pop(m, 0x170);
        return;
    }
    if !same_state {
        // ---- leave old state
        if s3 == 2 {
            let v1 = m.r32(s1 + 0x18c8);
            let v = match v1 {
                8 => m.r32(s1 + 0xf5c),
                3 | 0x16 => m.r32(s1 + 0x18c4),
                0x15 => m.r32(s1 + 0x1f18),
                _ => 0,
            };
            m.w32(s1 + 0x37c, v);
        }
        match m.r32(s1 + 0x18c8) {
            0 => {
                let a1 = m.r32(s1 + 0x58);
                m.call(0x0023_03a0, &[s1, a1, 2], &[]);
                let a1 = m.r32(s1 + 0x5c);
                m.call(0x0023_03a0, &[s1, a1, 2], &[]);
                match m.r32(s1 + 0x18d8) {
                    6 => {
                        m.call(0x0018_e540, &[s1, 4], &[]);
                    }
                    7 => {
                        m.call(0x0018_e540, &[s1, 5], &[]);
                    }
                    8 | 9 => {
                        m.call(0x0018_e540, &[s1, u32::MAX], &[]);
                    }
                    12 => {
                        if s5 != 6 {
                            m.call(0x0018_e540, &[s1, 0xe], &[]);
                        }
                    }
                    13 => {
                        if s5 != 7 {
                            m.call(0x0018_e540, &[s1, 0xf], &[]);
                        }
                    }
                    _ => {}
                }
                m.call(0x0014_2880, &[s1], &[]);
            }
            1 | 10 | 12 => {
                let v0 = m.r32(s1 + 0x1030);
                let v1 = m.r32(v0 + 0x14);
                let f = m.rf(v0 + 0x1c) / m.rf(v1 + 0x74);
                m.wf(s1 + 0x1a0, f);
            }
            2 => {
                // 0 = 0x18c3c8 (fx), 1 = 0x18c40c, 2 = 0x18c410
                let path = match s3 {
                    7 => {
                        if m.r32(s1 + 0x4c8) == 1 {
                            2
                        } else {
                            0
                        }
                    }
                    3 | 0x12 => 2,
                    8 => 1,
                    _ => 0,
                };
                if path == 0 {
                    let s0 = m.call(0x001f_bb90, &[0x3d], &[]).0;
                    m.call(0x001f_b2f8, &[s0, s1], &[]);
                    m.call(0x001f_b1f8, &[s0], &[f4(f32::from_bits(0x3f33_3333))]);
                    m.call(0x001f_b4f0, &[s0], &[f4(500.0), f4(1000.0)]);
                }
                if m.r32(s1 + 0x18d8) == 0 && s3 != 8 && s3 != 3 && s3 != 0x16 {
                    m.call(0x0018_e540, &[s1, u32::MAX], &[]);
                }
                if m.r32(s1 + 0x108c) == 0 {
                    let mut s2 = 0u32;
                    match s3 {
                        0 | 1 | 5 | 9 | 24 => s2 = m.r32(s1 + 0x1aa8),
                        7 => {
                            let v1 = m.r32(s1 + 0x4c8) as i32;
                            if v1 != 1 {
                                s2 = if v1 <= 0 || v1 >= 5 || v1 < 3 {
                                    m.r32(s1 + 0x1aa8)
                                } else {
                                    m.r32(s1 + 0x1aac)
                                };
                            }
                        }
                        _ => {}
                    }
                    if s2 != 0 {
                        vslot(m, s2, 7, &[], &[]);
                        m.call(0x0015_c150, &[s2], &[]);
                        vslot(m, s2, 8, &[], &[]);
                    }
                }
                m.call(0x0019_5fb8, &[s1 + 0x2c0, u32::MAX, 0], &[]);
            }
            3 => {
                let t2 = m.r32(s1 + 0x18c4);
                vslot(m, t2, 2, &[6, 0, 0, 0, 0], &[]);
                let a0 = m.r32(s1 + 0x18c4);
                m.call(0x0013_3448, &[a0, s1], &[]);
                m.call(0x0018_e540, &[s1, u32::MAX], &[]);
                let (a1, a2) = (m.r32(s1 + 0x19c0), m.r32(s1 + 0x1a00));
                m.call(0x0023_0100, &[s1, a1, a2], &[]);
                let a1 = m.r32(s1 + 0x1900);
                if a1 != 0 {
                    let a0 = m.r32(a1 + 4);
                    m.call(0x0022_9738, &[a0], &[]);
                    let a1 = m.r32(s1 + 0x1900);
                    m.call(0x0022_96c0, &[0x003d_d490, a1], &[]);
                    m.w32(s1 + 0x1900, 0);
                }
                m.call(0x0014_0c88, &[s1, 0], &[]);
            }
            4 => {
                m.call(0x001a_0b40, &[s1 + 0xdf0, u32::MAX], &[]);
            }
            6 => {
                m.call(0x0013_75d0, &[s1 + 0x1ff0, u32::MAX], &[]);
            }
            7 => {
                m.call(0x0019_2bd8, &[s1 + 0x4c0, u32::MAX], &[]);
            }
            8 => {
                m.call(0x0019_eab0, &[s1 + 0xf50, u32::MAX], &[]);
                let mut a1 = u32::MAX;
                if s3 == 2 && m.r32(s1 + 0x14d8) != 0 {
                    a1 = 0xb;
                }
                m.call(0x0018_e540, &[s1, a1], &[]);
            }
            9 => {
                m.w32(s1 + 0x18c0, 0);
                let a1 = m.r32(s1 + 0x1a90);
                m.call(0x001f_2a60, &[s1, a1], &[]);
            }
            11 => {
                let v = m.r32(s1 + 0x10e0) & 0xffff_fffd;
                m.w32(s1 + 0x10e0, v);
            }
            13 => m.w32(s1 + 0x18c0, 0),
            16 => {
                let a0 = m.r32(s1 + 0x1030);
                if a0 != 0 && m.call(0x0013_1968, &[a0], &[]).0 == 0 {
                    let a0 = m.r32(s1 + 0x10dc);
                    m.call(0x0020_9ee0, &[a0], &[]);
                }
                let v0 = m.r32(s1 + 0x10e0);
                m.w32(s1 + 0x1d2c, 0);
                m.w32(s1 + 0x10e0, v0 & 0xffff_fffb);
            }
            17 => {
                m.w32(s1 + 0x1d30, 0);
                m.call(0x0016_19a0, &[0x002d_eb30, 0x38, s1], &[]);
            }
            18 | 19 | 20 => {
                let a1 = m.r32(s1 + 0x58);
                m.call(0x0023_03a0, &[s1, a1, 2], &[]);
                let a1 = m.r32(s1 + 0x5c);
                m.call(0x0023_03a0, &[s1, a1, 2], &[]);
            }
            21 => {
                m.call(0x001f_5720, &[s1, 0], &[]);
                let t2 = m.r32(s1 + 0x1f18);
                vslot(m, t2, 2, &[6, 0, 0, 0, 0], &[]);
                let v1 = m.r32(s1 + 0x1f18);
                if m.r32(s1 + 0x1024) == m.r32(v1 + 0x110) {
                    m.call(0x0014_0c88, &[s1, 0], &[]);
                }
                m.w32(s1 + 0x1f18, 0);
            }
            22 => {
                m.call(0x001f_5720, &[s1, 0], &[]);
                let t2 = m.r32(s1 + 0x18c4);
                vslot(m, t2, 2, &[6, 0, 0, 0, 0], &[]);
                let a0 = m.r32(s1 + 0x18c4);
                m.call(0x0013_3448, &[a0, s1], &[]);
                m.w32(s1 + 0x18c4, 0);
                m.call(0x0014_0c88, &[s1, 0], &[]);
            }
            23 => {
                m.call(0x0022_06c8, &[s1 + 0x11b0, u32::MAX], &[]);
            }
            24 => {
                m.wf(s1 + 0x200, 1.0);
                let v0 = m.r32(s1 + 0x150);
                let f0 = m.rf(s1 + 0x1a8);
                m.wf(v0 + 0x18, f0);
            }
            _ => {}
        }
        // ---- enter new state
        let zero190 = |m: &mut M| {
            m.w32(s1 + 0x194, 0);
            m.w32(s1 + 0x198, 0);
            m.w32(s1 + 0x190, 0);
        };
        match s3 {
            0 => {
                let v1 = m.r32(s1 + 0x1028);
                let a1 = m.r32(s1 + 0x58);
                m.w32(sp + 0x58, v1);
                m.call(0x0023_03a0, &[s1, a1, 1], &[]);
                let a1 = m.r32(s1 + 0x5c);
                m.call(0x0023_03a0, &[s1, a1, 1], &[]);
                let v1 = m.r32(s1 + 0x18c8);
                m.w32(s1 + 0x1514, 0);
                s7 = match v1 {
                    10 => 0x177,
                    12 => 0x176,
                    2 => {
                        let mut v = m.r32(s1 + 0x18f0);
                        if v == u32::MAX {
                            let p = m.call(0x001a_62e8, &[s1], &[]).1;
                            let f1 = m.rf(s1 + 0x2fc) - p[1];
                            v = (m.rf(0x002b_f3e0) < f1) as u32;
                        }
                        if v == 1 {
                            0x179
                        } else {
                            0x178
                        }
                    }
                    _ => 0x175,
                };
                let a0 = m.r32(sp + 0x58);
                let v0 = m.call(0x001f_0400, &[a0, s7], &[]).0 as i32;
                if v0 < 0 {
                    s7 = u32::MAX;
                }
                let v0 = m.call(0x0014_30c8, &[s1], &[]).0;
                m.w32(sp + 0x5c, v0);
                zero190(m);
                s6 = 0x1f;
                if m.r32(s1 + 0x1a54) != 0 && m.r32(s1 + 0x1a70) != 0 {
                    match m.r32(s1 + 0x18d8) {
                        4 => {
                            m.call(0x0018_e540, &[s1, 6], &[]);
                        }
                        5 => {
                            m.call(0x0018_e540, &[s1, 7], &[]);
                        }
                        _ => {}
                    }
                }
                m.call(0x0014_2868, &[s1, 1], &[]);
            }
            1 | 12 => {
                let (o, a0) = if s3 == 1 { (0x1324, s1 + 0x1320) } else { (0x1c34, s1 + 0x1c30) };
                s4 = m.r32(s1 + o);
                fp = 9;
                let f1 = m.rf(s1 + 0x1a0);
                f22 = f1 * m.rf(s4 + 0x74);
                m.call(0x0020_bd68, &[a0], &[]);
            }
            3 => {
                m.w32(s1 + 0x18f8, u32::MAX);
                m.wf(s1 + 0x1998, 1.0);
                m.wf(s1 + 0x199c, 1.0);
                let a1 = m.r32(s1 + 0x1a00);
                let f0 = m.rf(T_NOW);
                f23 = 0.0;
                m.wf(s1 + 0x18f4, f0);
                s4 = m.r32(s1 + 0x1994);
                m.wf(a1 + 0x24, f23);
                m.wf(a1 + 0x28, f23);
                m.wf(a1 + 0x20, f23);
                f22 = 0.5;
                let (a1, a2) = (m.r32(s1 + 0x19c0), m.r32(s1 + 0x1a00));
                m.call(0x0023_0018, &[s1, a1, a2], &[]);
                let a0 = m.r32(s1 + 0x19a4);
                let p = m.call(0x001a_62e8, &[a0], &[]).1;
                m.wv(sp + 0x70, p);
                let q = m.call(0x0019_0c58, &[s1], &[]).1;
                let vf02 = m.rv(sp + 0x70);
                m.wv(s1 + 0x1960, sub(vf02, q));
                m.call(0x0019_0e18, &[s1], &[]);
                let t2 = m.r32(s1 + 0x18c4);
                vslot(m, t2, 2, &[5, 0, 0, 0, 0], &[]);
                let a0 = m.r32(s1 + 0x18c4);
                m.call(0x0013_33c0, &[a0, s1], &[]);
                m.call(0x0019_0be0, &[s1, sp + 0x50], &[]);
                let f12 = m.rf(sp + 0x50) + PI;
                let f20 = m.call(0x0011_cf60, &[], &[f4(f12)]).1[0];
                let v0 = m.r32(s1 + 0x18c4);
                let v1 = if v0 == 0 { 1 } else { m.r32(v0 + 0x124) as i32 };
                // Some(true) = face +0x50 (0x18cbac), Some(false) = face reversed (0x18cbc0)
                let face = match v1 {
                    1 => Some(true),
                    0 => {
                        let v0 = m.r32(s1 + 0x150);
                        let f12 = m.rf(sp + 0x50) - m.rf(v0 + 0x18);
                        let f0 = m.call(0x0011_cf60, &[], &[f4(f12)]).1[0];
                        Some(f0.abs() < PI_2)
                    }
                    2 => Some(false),
                    _ => None,
                };
                match face {
                    Some(true) => {
                        let f12 = m.rf(sp + 0x50);
                        m.call(0x0020_4660, &[s1], &[f4(f12)]);
                        m.w32(s1 + 0x18fc, 0);
                    }
                    Some(false) => {
                        m.call(0x0020_4660, &[s1], &[f4(f20)]);
                        m.w32(s1 + 0x18fc, 1);
                    }
                    None => {}
                }
                let v0 = m.r32(s1 + 0x18c4);
                let v = m.r32(v0 + 0x12c);
                m.w32(sp + 0x54, v);
                m.call(0x0018_e540, &[s1, 1], &[]);
                zero190(m);
                m.call(0x0018_fc60, &[s1], &[]);
                let v0 = m.r32(s1 + 0x18c4);
                let a1 = m.r32(v0 + 0x100);
                m.call(0x0014_0c88, &[s1, a1], &[]);
            }
            4 => {
                m.call(0x0019_cd38, &[s1 + 0xf00, u32::MAX], &[]);
                m.call(0x0018_df50, &[s1, 0], &[]);
            }
            6 => {
                let s0 = s1 + 0x1ff0;
                m.w32(sp + 0x60, 1);
                let v0 = m.call(0x0013_74f0, &[s0], &[]).0;
                m.call(0x0013_75d0, &[s0, v0], &[]);
            }
            7 => {
                let v1 = m.r32(s1 + 0x4c8) as i32;
                if v1 < 5 {
                    s6 = if v1 < 3 { 0 } else { 7 };
                }
            }
            8 => {
                m.call(0x0019_eab0, &[s1 + 0xf50, 0], &[]);
                m.w32(sp + 0x60, 1);
            }
            9 => {
                m.call(0x0018_e540, &[s1, u32::MAX], &[]);
                m.call(0x0018_f688, &[s1], &[]);
                s4 = m.r32(s1 + 0x1a7c);
            }
            10 => {
                s4 = m.r32(s1 + 0x1bc0);
                let f1 = m.rf(s1 + 0x1a0);
                fp = 9;
                let f0 = m.rf(s4 + 0x74);
                m.w32(s1 + 0x1038, u32::MAX);
                f22 = f1 * f0;
            }
            11 => {
                let v1 = m.r32(s1 + 0x10e0);
                s4 = m.r32(s1 + 0x1bc8);
                m.w32(s1 + 0x10e0, v1 | 2);
                s6 = 100;
                cp16(m, s1 + 0x190, V_ZERO);
                m.w32(s1 + 0x1bd4, 0);
            }
            13 => {
                m.call(0x0018_e540, &[s1, u32::MAX], &[]);
                zero190(m);
                m.call(0x0018_f740, &[s1], &[]);
                let f0 = m.rf(T_NOW) + m.rf(0x002b_f3e4);
                s4 = m.r32(s1 + 0x1a80);
                m.wf(s1 + 0x19b0, f0);
            }
            14 => {
                m.call(0x0018_e540, &[s1, u32::MAX], &[]);
                zero190(m);
                s4 = m.r32(s1 + 0x1a84);
            }
            16 => {
                let v0 = m.r32(s1 + 0x10e0) | 4;
                m.w32(s1 + 0x10e0, v0);
                if m.call(0x001d_aea8, &[0x1c], &[]).0 != 0 {
                    s4 = m.r32(s1 + 0x2050);
                    m.call(0x0018_e1e0, &[s1], &[]);
                    let vf05: V4 = [-1.0, f32::from_bits(u32::MAX), 0.0, 0.0];
                    let vf02 = scl(m.rv(s1 + 0x190), -1.0);
                    let vf04 = m.rv(s1 + 0x1d0);
                    let vf01 = m.rv(s1 + 0x1c0);
                    let vf03 = m.rv(s1 + 0x1b0);
                    let vf06 = mx3([0.0; 4], vf03, vf01, vf04, vf02);
                    m.wv(sp, vf05);
                    vslot(m, s1, 34, &[], &[vf06]);
                    m.call(0x0020_4ca8, &[s1], &[]);
                } else {
                    let a0 = m.r32(s1 + 0x10dc);
                    s4 = m.r32(s1 + 0x1d1c);
                    let f21 = 0.0f32;
                    let p = m.call(0x001a_62e8, &[a0], &[]).1;
                    m.wv(sp + 0x80, p);
                    let q = m.call(0x001a_62e8, &[s1], &[]).1;
                    let vf01 = m.rv(sp + 0x80);
                    let vf02 = sub(vf01, q);
                    let neg: V4 = [-1.0, f32::from_bits(u32::MAX), 0.0, 0.0];
                    let vf03 = scl(vf02, -1.0);
                    m.wv(sp, neg);
                    m.call(0x0020_46b0, &[s1], &[vf03]);
                    m.call(0x0020_4ca8, &[s1], &[]);
                    let mut f20 = m.call(0x0012_f990, &[s4, 0x173], &[]).1[0];
                    let v0 = m.r32(s1 + 0x19a4);
                    let a3 = m.r32(s1 + 0x60);
                    let a2 = m.r32(v0 + 0x60);
                    m.call(0x0013_00b8, &[s4, s1, a2, a3, sp + 0x10, 0], &[f4(f20), f4(f21)]);
                    f20 /= m.rf(s4 + 0x78);
                    let a0 = m.r32(s1 + 0x10dc);
                    let vf02 = m.call(0x001a_62e8, &[a0], &[]).1;
                    let sgn = if (f20.to_bits() as i32) < 0 { u32::MAX } else { 0 };
                    let fq: V4 = [f20, f32::from_bits(sgn), 0.0, 0.0];
                    m.wv(sp + 0x90, fq);
                    m.wv(sp, fq);
                    m.wv(sp + 0xa0, vf02);
                    let a1 = m.r32(s1 + 0x10dc);
                    let vf04 = vslot(m, a1, 32, &[], &[]).1;
                    let vf02 = m.rv(sp + 0xa0);
                    let vf01 = m.rv(sp + 0x90);
                    let vf02 = add(add(vf02, ONE_W), scl(vf04, vf01[0]));
                    let vf05 = m.rv(s1 + 0x1d0);
                    let vf01m = m.rv(s1 + 0x1c0);
                    let vf04 = m.rv(s1 + 0x1b0);
                    let vf03 = m.rv(sp + 0x40);
                    let vf03 = mx3(vf03, vf04, vf01m, vf05, vf03);
                    let vf02 = sub(vf02, vf03);
                    m.wv(sp + 0xa0, vf02);
                    let vf06 = m.call(0x001a_62e8, &[s1], &[]).1;
                    let vf01 = m.rv(sp + 0x90);
                    let vf02 = m.rv(sp + 0xa0);
                    m.wv(sp, vf01);
                    let vf02 = sub(vf02, vf06);
                    let qv = 1.0 / vf01[0];
                    m.wv(s1 + 0x190, scl(vf02, qv));
                    m.wf(s1 + 0x198, f21);
                    let a1q = m.rv(s1 + 0x190);
                    vslot(m, s1, 34, &[], &[a1q]);
                }
            }
            17 => {
                s4 = m.r32(s1 + 0x1d34);
                fp = 9;
                m.call(0x0016_19a0, &[0x002d_eb30, 0x37, s1], &[]);
            }
            18 | 19 | 20 => {
                if s3 == 18 {
                    let v1 = m.r32(s1 + 0x102c);
                    let a1 = m.r32(s1 + 0x58);
                    m.w32(sp + 0x58, v1);
                    m.call(0x0023_03a0, &[s1, a1, 1], &[]);
                    let a1 = m.r32(s1 + 0x5c);
                    m.call(0x0023_03a0, &[s1, a1, 1], &[]);
                    zero190(m);
                    m.call(0x0018_e540, &[s1, u32::MAX], &[]);
                }
                let k = if s3 == 20 { 2 } else { 1 };
                let a1 = m.r32(s1 + 0x58);
                m.call(0x0023_03a0, &[s1, a1, k], &[]);
                let a1 = m.r32(s1 + 0x5c);
                m.call(0x0023_03a0, &[s1, a1, k], &[]);
                zero190(m);
                m.call(0x0018_e540, &[s1, u32::MAX], &[]);
            }
            21 | 22 => {
                if s3 == 21 {
                    let p = m.call(0x001a_62e8, &[s1], &[]).1;
                    let a0 = m.r32(s1 + 0x1f18);
                    m.call(0x0018_5310, &[a0, 0, s1 + 0x1f1c, 0, 0], &[p]);
                } else {
                    s4 = m.r32(s1 + 0x1f90);
                    fp = 9;
                    m.call(0x0018_e540, &[s1, u32::MAX], &[]);
                }
                m.call(0x001f_5720, &[s1, 1], &[]);
                let z = m.rv(V_ZERO);
                vslot(m, s1, 34, &[], &[z]);
                let z = m.rv(V_ZERO);
                vslot(m, s1, 35, &[], &[z]);
                zero190(m);
                if s3 == 21 {
                    let t2 = m.r32(s1 + 0x1f18);
                    vslot(m, t2, 2, &[5, 0, 0, 0, 0], &[]);
                    let v0 = m.r32(s1 + 0x1f18);
                    let a1 = m.r32(v0 + 0x110);
                    if a1 != 0 {
                        m.call(0x0014_0c88, &[s1, a1], &[]);
                    }
                } else {
                    let t2 = m.r32(s1 + 0x18c4);
                    vslot(m, t2, 2, &[5, 0, 0, 0, 0], &[]);
                    let a0 = m.r32(s1 + 0x18c4);
                    m.call(0x0013_33c0, &[a0, s1], &[]);
                    let v0 = m.r32(s1 + 0x18c4);
                    let a1 = m.r32(v0 + 0x100);
                    m.call(0x0014_0c88, &[s1, a1], &[]);
                }
            }
            23 => m.w32(sp + 0x60, 1),
            24 => {
                let f20 = 0.0f32;
                fp = 9;
                s6 = 0x18;
                let a0 = m.r32(s1 + 0x2038);
                m.wf(s1 + 0x202c, f20);
                m.wf(s1 + 0x2014, f20);
                m.wf(s1 + 0x2020, 1.0);
                m.call(0x0013_84c0, &[a0], &[]);
                let v0 = m.r32(s1 + 0x150);
                s4 = m.r32(s1 + 0x2038);
                let f12 = m.rf(v0 + 0x18);
                m.call(0x0020_4660, &[s1], &[f4(f12)]);
                let (f12, f13) = (m.rf(s1 + 0x1a8), m.rf(0x002b_f3e8));
                let q = m.call(0x0011_e1a0, &[], &[f4(f12), f4(f13), f4(f20)]).1;
                vslot(m, s1, 34, &[], &[q]);
                m.wf(s1 + 0x194, f20);
                m.wf(s1 + 0x198, f20);
                m.wf(s1 + 0x190, f20);
            }
            _ => {}
        }
    }
    // ---- 0x18d1fc: enter new sub-state
    if m.r32(s1 + 0x18d0) as i32 != s5 {
        let zero190 = |m: &mut M| {
            m.w32(s1 + 0x194, 0);
            m.w32(s1 + 0x198, 0);
            m.w32(s1 + 0x190, 0);
        };
        match s5 {
            0 | 1 | 2 => {
                let v0 = m.r32(s1 + 0x1030);
                let v1 = m.r32(s1 + 0x1994);
                if v0 != 0 && m.r32(v0 + 0x14) == v1 {
                    m.w32(sp + 0x60, 1);
                } else {
                    s4 = v1;
                    f23 = 0.0;
                    f22 = m.rf(s4 + 0x74) * 0.5;
                }
            }
            3 => {
                let v1 = m.r32(s1 + 0x1950);
                m.w32(sp + 0x58, v1);
            }
            4 => {
                let v0 = m.r32(s1 + 0x1bc4);
                m.w32(sp + 0x58, v0);
                zero190(m);
            }
            5 => {
                s4 = m.r32(s1 + 0x14d0);
                m.w32(s1 + 0x1038, u32::MAX);
                fp = 9;
                zero190(m);
            }
            6 | 7 => {
                m.call(0x0018_e540, &[s1, u32::MAX], &[]);
                s4 = m.r32(s1 + if s5 == 6 { 0x1d08 } else { 0x1d0c });
                zero190(m);
                m.w32(s1 + 0x18e8, 0);
                m.w32(s1 + 0x18e4, 0);
            }
            8 | 9 => {
                s4 = m.r32(s1 + if s5 == 8 { 0x1d10 } else { 0x1fe8 });
                m.w32(s1 + 0x18e4, 0);
                zero190(m);
            }
            10 => {
                let v1 = m.r32(s1 + 0x1f14);
                m.w32(sp + 0x58, v1);
            }
            11 => {
                s4 = m.r32(s1 + 0x1f10);
                fp = 9;
            }
            12 | 13 => {
                s6 = 0x18;
                let f0 = if s5 == 12 {
                    m.rf(0x002b_f358)
                } else {
                    m.w32(sp + 0x60, 1);
                    m.rf(0x002b_f35c)
                };
                m.wf(s1 + 0x200, f0);
            }
            _ => {}
        }
    }
    // ---- 0x18d34c
    if m.r32(s1 + 0x18c8) != s3 {
        m.w32(s1 + 0x18c8, s3);
        let f0 = m.rf(T_NOW);
        m.wf(s1 + 0x18cc, f0);
    }
    if m.r32(s1 + 0x18d0) as i32 != s5 {
        m.w32(s1 + 0x18d0, s5 as u32);
        let f0 = m.rf(T_NOW);
        m.wf(s1 + 0x18d4, f0);
    }
    vslot(m, s1, 89, &[], &[]);
    m.call(0x0018_c120, &[s1, s6], &[]);
    let v1 = m.r32(sp + 0x54);
    if v1 != u32::MAX {
        let s0 = m.call(0x001f_bb90, &[v1], &[]).0;
        m.call(0x001f_b2f8, &[s0, 0], &[]);
        let p = m.call(0x001a_62e8, &[s1], &[]).1;
        m.call(0x001f_b3f0, &[s0], &[p]);
    }
    if m.r32(sp + 0x60) == 0 {
        let a0 = m.r32(s1 + 0x1030);
        if a0 != 0 {
            m.call(0x0013_0b18, &[a0], &[]);
            let a1 = m.r32(s1 + 0x1030);
            vslot(m, a1, 1, &[], &[]);
            m.w32(s1 + 0x1030, 0);
        } else {
            let a0 = m.r32(s1 + 0x1034);
            if a0 != 0 {
                m.call(0x001f_0598, &[a0], &[]);
                let a1 = m.r32(s1 + 0x1034);
                vslot(m, a1, 1, &[], &[]);
                m.w32(s1 + 0x1034, 0);
            }
        }
        if s4 != 0 {
            m.call(0x0012_f9b8, &[s4, s1, fp, s1 + 0x1030], &[f4(f22), f4(f23)]);
            let a1 = m.r32(s1 + 0x1030);
            vslot(m, a1, 0, &[], &[]);
        } else {
            let v1 = m.r32(sp + 0x58);
            if v1 != 0 {
                let t0 = m.r32(sp + 0x5c);
                let v0 = m.call(0x001f_03a8, &[v1, s1, s7, fp, t0], &[]).0;
                m.w32(s1 + 0x1034, v0);
                vslot(m, v0, 0, &[], &[]);
            }
        }
    }
    stack_pop(m, 0x170);
}

// ---------------------------------------------------------------------------------------------------

/// 0x197f14 / 0x197f18 / 0x197f1c: FUN_00195fb8(s2, a1, 0)
fn st_195fb8<M: QMem + ?Sized>(m: &mut M, s2: u32, a1: u32) {
    m.call(0x0019_5fb8, &[s2, a1, 0], &[]);
}

/// FUN_00197500(climb state): per-frame climb / ledge state update. Re-runs the state handler (table
/// 0x2a6410 on +0x1c) until the state is stable or -1; pre-pass handles the jump / attack / 0x1b-ability
/// inputs while not in states 9..22.
pub fn ee_sub_197500<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let s2 = a0;
    let sp = stack_push(m, 0xd0);
    let mut s4 = false;
    let mut s6: u32 = 0;
    let jt = m.r32(s2);
    let p = m.call(0x001a_62e8, &[jt], &[]).1;
    m.wv(sp, p);
    let a1 = m.r32(s2);
    let q = vslot(m, a1, 32, &[], &[]).1;
    m.wv(sp + 0x10, q);
    let v0 = m.r32(s2);
    let v1 = m.r32(v0 + 0x208);
    let s3 = m.r32(v0 + 0x150);
    if v1 != 0 && f32::from_bits(0x3f33_3333) < m.rf(v1 + 0x88) {
        s4 = true;
    }
    loop {
        let v0 = m.r32(s2 + 0xc);
        let s5 = m.r32(s2 + 0x1c);
        if v0 != 0 {
            let w = m.r32(v0 + 0x2c);
            if (w >> 28) & 1 == 0 {
                m.call(0x0019_c828, &[s2], &[]);
            }
        }
        let v1 = m.r32(s2 + 0x1c) as i32;
        if !(9..0x17).contains(&v1) {
            if m.r32(s2 + 0x28) != 0
                && m.call(0x001c_4660, &[s3, 0x40], &[f4(0.0)]).0 != 0
            {
                let a0 = m.r32(s2);
                if m.call(0x0014_3d20, &[a0], &[f4(f32::from_bits(0x3d99_999a))]).0 == 0 {
                    m.call(0x001c_4738, &[s3, 0x40], &[]);
                    let a2 = m.r32(s2 + 8);
                    m.call(0x0019_5a00, &[s2, 6, a2, 0], &[]);
                }
            }
            if m.call(0x001c_4660, &[s3, 0x20], &[f4(0.0)]).0 != 0 {
                let s0 = m.r32(s2 + 0x1c);
                m.call(0x0019_b110, &[s2, 0], &[]);
                if m.r32(s2 + 0x1c) != s0 {
                    m.call(0x001c_4738, &[s3, 0x20], &[]);
                }
            }
            if m.call(0x001c_4660, &[s3, 8], &[f4(0.0)]).0 != 0
                && m.r32(s2) == m.r32(JT_MAIN)
                && m.call(0x001d_aea8, &[0x1b], &[]).0 != 0
            {
                let a0 = m.r32(s2);
                if m.call(0x0014_3d20, &[a0], &[f4(f32::from_bits(0x3e99_999a))]).0 == 0 {
                    let a0 = m.r32(s2);
                    m.call(0x0014_1cd0, &[a0, 0x1b], &[]);
                    m.call(0x001c_4738, &[s3, 8], &[]);
                    let a2 = m.r32(JT_MAIN);
                    vslot(m, a2, 99, &[0x1b], &[]);
                }
            }
        }
        let st = m.r32(s2 + 0x1c);
        // 0x1979f0 / 0x197ebc tails
        let tail_19a098 = |m: &mut M| {
            let a0 = m.r32(s2);
            let f12 = m.rf(s2 + 0x20);
            if m.call(0x0019_a098, &[a0, a0 + 0x131c], &[f4(f12)]).0 != 0 {
                let a0 = m.r32(s2);
                m.call(0x0019_f590, &[a0, 0], &[]);
            }
        };
        let tail_slot87 = |m: &mut M| {
            let a2 = m.r32(s2);
            vslot(m, a2, 87, &[0], &[]);
        };
        // 0x1977a8: idle -> FUN_00197370 when stick pushed
        let idle_push = |m: &mut M| {
            if m.r32(s2 + 0x30) == 0 && 0.5 < m.rf(s3 + 0x1c) {
                m.call(0x0019_7370, &[s2], &[]);
            }
        };
        // 0x1978ac: anim end -> FUN_0019c688
        let anim_end = |m: &mut M| {
            let a0 = m.r32(s2);
            let v1 = m.r32(a0 + 0x58);
            if m.rf(v1 + 0xa8) < m.rf(T_NOW) {
                m.call(0x0019_c688, &[s2], &[]);
            }
        };
        match st {
            0 => {
                if m.rf(0x002b_f818) < m.rf(T_NOW) - m.rf(s2 + 0x20) {
                    if m.r32(s3 + 0x30) & 0x40 == 0 && m.r32(s2 + 0x40) != 3 {
                        m.w32(s2 + 0x24, 1);
                    }
                    let a2 = m.r32(s2 + 0xc);
                    m.call(0x0019_5fb8, &[s2, 1, a2], &[]);
                } else {
                    idle_push(m);
                }
            }
            1 | 7 | 8 => {
                let go = if st == 8 {
                    let a0 = m.r32(s2);
                    let v1 = m.r32(a0 + 0x58);
                    m.rf(v1 + 0xa8) < m.rf(T_NOW)
                } else {
                    m.rf(sp + 0x18) < 0.0
                };
                if go {
                    let a2 = m.r32(s2 + 0xc);
                    m.call(0x0019_5fb8, &[s2, 4, a2], &[]);
                }
            }
            2 => {
                if m.rf(0x002b_f81c) < m.rf(T_NOW) - m.rf(s2 + 0x20) {
                    let a2 = m.r32(s2 + 0xc);
                    m.call(0x0019_5fb8, &[s2, 3, a2], &[]);
                } else {
                    idle_push(m);
                }
            }
            3 | 4 => {
                if s4 {
                    if m.r32(s2 + 4) == 0xe {
                        let a0 = m.r32(s2);
                        m.call(0x0019_28f8, &[a0 + 0x4c0, 0xa, 0, 0], &[]);
                    } else {
                        tail_slot87(m);
                    }
                } else {
                    let s0 = m.call(0x0019_9d48, &[s2, sp + 0x40, sp + 0x20], &[]).0;
                    if s0 == 0 {
                        tail_19a098(m);
                    } else {
                        let k = m.r32(sp + 0x40);
                        let v0 = s0.wrapping_add((1u32.wrapping_sub(k)) << 6);
                        let v1 = m.r32(v0);
                        m.w32(s2 + 0xc8, v1);
                        let a1 = m.r32(v1);
                        let a2q = m.rv(v0 + 0x10);
                        let r = m.call(0x0022_fc48, &[0, a1], &[a2q]).1;
                        m.wv(s2 + 0xd0, r);
                        let v0 = m.r32(s0 + 0x9c);
                        m.w32(s2 + 0x100, v0);
                        let v1 = if m.r32(sp + 0x40) != 0 { m.r32(s0 + 0xa0) } else { m.r32(s0 + 0xa4) };
                        let q = m.rv(sp + 0x20);
                        m.w32(s2 + 0x104, v1);
                        m.wv(s2 + 0xe0, q);
                        st_195fb8(m, s2, 0x11);
                    }
                }
            }
            5 => {
                if 0.25 < m.rf(T_NOW) - m.rf(s2 + 0x20) && s4 {
                    tail_slot87(m);
                } else {
                    tail_19a098(m);
                }
            }
            6 => {
                let v0 = m.r32(s2 + 0xc);
                if m.r32(v0 + 0x17c) == 3 {
                    st_195fb8(m, s2, 4);
                } else {
                    anim_end(m);
                }
            }
            9..=16 => anim_end(m),
            17 => {
                if 0.25 < m.rf(T_NOW) - m.rf(s2 + 0x20)
                    && m.call(0x001c_46c0, &[s3, 0x40], &[]).0 != 0
                {
                    m.call(0x0019_5a00, &[s2, 3, 0, 0], &[]);
                } else {
                    let v0 = m.r32(s2 + 0xc8);
                    let v1 = m.r32(v0 + 4);
                    if m.r32(v1 + 0xec) == 0 {
                        st_195fb8(m, s2, 4);
                    } else {
                        let s0 = m.r32(s2);
                        let a1q = m.rv(V_ZERO);
                        let f12 = m.rf(T_DT);
                        let a0q = m.rv(s0 + 0x1010);
                        let r = m.call(0x0011_f088, &[0x002b_f820, 0], &[a0q, a1q, f4(f12)]).1;
                        m.wv(s0 + 0x1010, r);
                        let v0 = m.r32(s2 + 0xc8);
                        let a2q = m.rv(s2 + 0xd0);
                        let a0 = m.r32(v0);
                        let r = m.call(0x0022_fc48, &[0, a0], &[a2q]).1;
                        m.wv(sp + 0x20, r);
                        let mut ok = true;
                        if f32::from_bits(0x3cf5_c28f) < m.rf(T_NOW) - m.rf(s2 + 0x20) {
                            let (a1, a3, t0) = (m.r32(s2 + 0xc8), m.r32(s2 + 0x100), m.r32(s2 + 0x104));
                            ok = m.call(0x0019_99d0, &[s2, a1, a3, t0, s2 + 0xe0], &[r]).0 != 0;
                        }
                        if !ok {
                            st_195fb8(m, s2, 4);
                        } else {
                            let vf01 = m.rv(s2 + 0xe0);
                            let vf02: V4 = [-1.0, f32::from_bits(u32::MAX), 0.0, 0.0];
                            let a0 = m.r32(s2);
                            m.wv(sp + 0x30, vf02);
                            m.call(0x0020_46b0, &[a0], &[scl(vf01, -1.0)]);
                            if m.call(0x001c_4660, &[s3, 0x400], &[f4(f32::from_bits(0x3e19_999a))]).0 != 0 {
                                let a2 = m.r32(s2);
                                if vslot(m, a2, 98, &[3], &[]).0 != 0 {
                                    m.call(0x001c_4738, &[s3, 0x400], &[]);
                                    let a0 = m.r32(s2);
                                    m.call(0x0019_cc50, &[a0 + 0xf00, 0, 0, 0], &[]);
                                }
                            }
                        }
                    }
                }
            }
            18 => {
                let g = m.r32(0x002f_7900);
                let a0 = m.r32(g + 0x1974);
                let f0 = m.call(0x0012_f990, &[a0, 0x1c7], &[]).1[0];
                let a0 = m.r32(s2);
                let v0 = m.r32(a0 + 0x58);
                let f1 = m.rf(v0 + 0xa8) - f0;
                let mut handled = false;
                if f1 <= m.rf(T_NOW) {
                    let g = m.r32(0x002f_7900);
                    if m.r32(g + 0x1914) != 3 {
                        m.call(0x0022_06c8, &[g + 0x1910, 3], &[]);
                        handled = true;
                    }
                }
                if !handled {
                    anim_end(m);
                }
            }
            19 | 20 => {
                let v0 = m.r32(s2);
                let a0 = m.r32(v0 + 0xec);
                if m.call(0x0019_74c8, &[a0], &[]).0 != 0 {
                    st_195fb8(m, s2, 4);
                } else {
                    let a0 = m.r32(s2);
                    let v0 = m.call(0x0014_1ea8, &[a0, 0x1b], &[]).0;
                    if m.r32(s3 + 0x30) & (v0 | 8) == 0 {
                        st_195fb8(m, s2, 4);
                    } else if m.r32(s2 + 0x1c) == 0x13 {
                        let v0 = m.r32(s2);
                        let a0 = m.r32(v0 + 0x1030);
                        if a0 == 0 || m.call(0x0013_1968, &[a0], &[]).0 != 0 {
                            st_195fb8(m, s2, 0x14);
                        }
                    }
                }
            }
            21 => {
                let v1 = m.r32(s2 + 0x1f0);
                let f2 = m.rf(T_NOW);
                let f1 = m.rf(v1 + 0x1c);
                if f1 < f2 {
                    let a0 = m.r32(s2);
                    let mut s0 = 0u32;
                    if m.call(0x0013_2b10, &[a0, 0x35], &[]).0 != 0 {
                        s0 = m.r32(s2);
                        s6 = m.r32(s0 + 0x18f0);
                    }
                    let a1 = m.r32(s2);
                    let q = vslot(m, a1, 32, &[], &[]).1;
                    m.wv(sp + 0x20, q);
                    m.wf(sp + 0x20, 0.0);
                    m.wf(sp + 0x24, 0.0);
                    let a2 = m.r32(s2);
                    let a1q = m.rv(sp + 0x20);
                    vslot(m, a2, 34, &[], &[a1q]);
                    m.call(0x0019_5a00, &[s2, 0xb, 0, 0], &[]);
                    if s0 != 0 {
                        m.w32(s0 + 0x18f0, s6);
                    }
                } else {
                    let mut done = false;
                    if m.r32(s2 + 0x1d4) != 0 && f1 < f2 + 0.25 {
                        let mut hit = false;
                        for k in [0x13, 0xb, 0xd, 0x27, 0xc, 0x22, 0x26] {
                            let a0 = m.r32(s2 + 0x1d0);
                            if m.call(0x0013_2b10, &[a0, k], &[]).0 != 0 {
                                hit = true;
                                break;
                            }
                        }
                        if hit {
                            m.call(0x0019_5a00, &[s2, 0xb, 0, 0], &[]);
                            let a1 = m.r32(s2 + 0x1d0);
                            m.call(0x0019_b110, &[s2, a1], &[]);
                            done = true;
                        }
                    }
                    if !done {
                        let a1 = m.r32(s2);
                        let v0 = m.r32(a1 + 0x58);
                        if m.r32(s2 + 0x28) != 0 && m.rf(v0 + 0xb8) < 0.0 {
                            let a0 = m.r32(s2 + 0x134);
                            m.call(0x0013_05b8, &[a0, a1, 1, a1 + 0x1030], &[f4(0.0), f4(1.0)]);
                            m.w32(s2 + 0x28, 0);
                            let s1 = m.r32(s2);
                            let v = m.call(0x0019_a750, &[s2, 6], &[]).1;
                            vslot(m, s1, 34, &[], &[v]);
                        } else if m.r32(s2 + 0x1d4) != 0 || m.r32(s2 + 0x1d8) != 0 {
                            let v0 = m.r32(s2 + 0x1f0);
                            let f12 = (m.rf(v0 + 0x1c) - m.rf(T_NOW)).min(f32::from_bits(0x3f33_3333));
                            let a0 = m.r32(s2);
                            let q = m.call(0x0019_f3f8, &[a0, s2 + 0x1d0], &[f4(f12)]).1;
                            let v1 = m.r32(s2);
                            let v0 = m.r32(v1 + 0x58);
                            let a1 = m.r32(v0 + 0xa0);
                            let r = m.call(0x0022_fc48, &[0, a1], &[q]).1;
                            let a0 = m.r32(s2 + 0x1f0);
                            let a2q = m.rv(a0 + 0x30);
                            m.call(0x0012_e0f0, &[a0], &[r, a2q]);
                        }
                    }
                }
            }
            22 => {
                let a0 = m.r32(s2);
                let v0 = m.call(0x0014_41a0, &[a0], &[]).0 as i32;
                if v0 <= 0 {
                    st_195fb8(m, s2, 4);
                } else {
                    let f3 = m.rf(s2 + 0x20);
                    let thr = m.rf(0x002b_f82c);
                    let mut to_2c = true;
                    if thr < m.rf(T_NOW) - f3 && s4 {
                        tail_slot87(m);
                        to_2c = false;
                    } else if thr < m.rf(T_NOW) - f3 {
                        let a0 = m.r32(s2);
                        let v0 = m.call(0x0014_1ea8, &[a0, 0xc], &[]).0;
                        if m.r32(s3 + 0x30) & v0 == 0 {
                            st_195fb8(m, s2, 4);
                            to_2c = false;
                        }
                    }
                    if to_2c && m.rf(0x002b_f830) < m.rf(T_NOW) - m.rf(s2 + 0x20) {
                        st_195fb8(m, s2, 4);
                    }
                }
            }
            _ => {}
        }
        let v1 = m.r32(s2 + 0x1c);
        if v1 == s5 || v1 == u32::MAX {
            break;
        }
    }
    stack_pop(m, 0xd0);
}
