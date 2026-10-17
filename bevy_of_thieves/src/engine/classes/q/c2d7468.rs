//! WP-Q8: C2d7468 / C2d74b8 / C2d7508 (particle-like objects: position +0x70, previous +0x80, basis rows
//! +0x40..+0x60, velocity +0xd0) slot overrides.
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem, V4};
use crate::engine::classes::q::q8util::*;

// ---------------------------------------------------------------- C2d7468
/// FUN_001ce920 (slot 0, ctor): a0 = this, a1 = position (quadword), a2 = +0xe0, f12 = +0x20;
/// +0x18 = 1.0, +0x14 = clock, +0x70 / +0x80 = position, +0x30 = DAT_002c1f50, +0x40 = DAT_002a2d00,
/// +0x50 = DAT_002a2d10, +0x60 = DAT_002a2d20
pub fn ee_sub_1ce920<M: QMem + ?Sized>(m: &mut M, o: u32, pos: V4, a2: u32, f12: f32) {
    let clk = m.r32(CLOCK);
    m.w32(o + 0xe0, a2);
    m.w32(o + 0x18, 0x3f80_0000);
    m.w32(o + 0x14, clk);
    m.wf(o + 0x20, f12);
    m.wv(o + 0x70, pos);
    m.wv(o + 0x80, pos);
    for (d, a) in [(0x30, 0x002c_1f50), (0x40, 0x002a_2d00), (0x50, 0x002a_2d10), (0x60, 0x002a_2d20)] {
        m.wv(o + d, rov(a));
    }
}
fn len2(v: V4) -> f32 {
    v[0] * v[0] + v[1] * v[1] + v[2] * v[2]
}
/// FUN_001cefc8: |v|^2 < 4.0
pub fn ee_sub_1cefc8(v: V4) -> bool {
    len2(v) < 4.0
}
/// FUN_001cf010: |v|^2 < 0.0004
pub fn ee_sub_1cf010(v: V4) -> bool {
    len2(v) < 0.0004
}
/// FUN_001cf058: |v|^2 < 25.0
pub fn ee_sub_1cf058(v: V4) -> bool {
    len2(v) < 25.0
}
/// FUN_001d1108 (slot 1, copy): parent FUN_001ce9b8(this, src); +0xf0 = FUN_0015e938(world, 4, src[+0xf0], **(src + 8)); +0xf4 = src[+0xf4]
pub fn ee_sub_1d1108<M: QMem + ?Sized>(m: &mut M, o: u32, src: u32) {
    m.call(0x001c_e9b8, &[o, src], &[]);
    let id = m.r32(src + 0xf0);
    let p = m.r32(src + 8);
    let w = m.r32(p);
    let h = m.call(0x0015_e938, &[WORLD, 4, id, w], &[]).0;
    m.w32(o + 0xf0, h);
    let r = m.r32(src + 0xf4);
    m.w32(o + 0xf4, r);
}
/// FUN_001d1160 (update): parent FUN_001cf268; with an emitter target (+0xf0): +0xf8 += dt * rate (+0xf4); per whole
/// unit send vslot 0x19c(&{4, ..., position +0x70 at +0x20}) and subtract 1.0
pub fn ee_sub_1d1160<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001c_f268, &[o], &[]);
    if m.r32(o + 0xf0) != 0 {
        let dt = m.rf(0x002d_ded4);
        let rate = m.rf(o + 0xf4);
        let mut f = m.rf(o + 0xf8) + dt * rate;
        m.wf(o + 0xf8, f);
        while 1.0 <= f {
            let buf = stack_push(m, 0x30);
            m.w32(buf, 4);
            let p = m.rv(o + 0x70);
            m.wv(buf + 0x20, p);
            let h = m.r32(o + 0xf0);
            vcall(m, h, 0xc, 0x19c, &[buf], &[]);
            stack_pop(m, 0x30);
            f = m.rf(o + 0xf8) - 1.0;
            m.wf(o + 0xf8, f);
        }
    }
}

// ---------------------------------------------------------------- C2d74b8
/// `r_i = b0 * m_i.x + b1 * m_i.y + b2 * m_i.z` (vmulabc / vmaddabc / vmaddbc, all four lanes)
fn mulrow(b: &[V4; 3], r: V4) -> V4 {
    std::array::from_fn(|i| b[0][i] * r[0] + b[1][i] * r[1] + b[2][i] * r[2])
}
/// FUN_001a7050: out (3 quadwords) = rows of matrix `mat` composed with the basis of `a` (FUN_001a6318, when
/// non-null), then with the transposed / inverse basis of `c` (FUN_0011b340 of FUN_001a6318(c), when non-null)
pub fn ee_sub_1a7050<M: QMem + ?Sized>(m: &mut M, out: u32, a: u32, c: u32, mat: u32) {
    let mut r = [m.rv(mat), m.rv(mat + 0x10), m.rv(mat + 0x20)];
    if a != 0 {
        let bp = m.call(0x001a_6318, &[a], &[]).0;
        let b = [m.rv(bp), m.rv(bp + 0x10), m.rv(bp + 0x20)];
        r = r.map(|x| mulrow(&b, x));
    }
    if c != 0 {
        let cp = m.call(0x001a_6318, &[c], &[]).0;
        let t = stack_push(m, 0x30);
        m.call(0x0011_b340, &[t, cp], &[]);
        let b = [m.rv(t), m.rv(t + 0x10), m.rv(t + 0x20)];
        stack_pop(m, 0x30);
        r = r.map(|x| mulrow(&b, x));
    }
    for (i, v) in r.into_iter().enumerate() {
        m.wv(out + 0x10 * i as u32, v);
    }
}
/// FUN_001d08d8 (slot 0, ctor): parent FUN_001ce9b8; v = (rand(-1,1), rand(-1,1), rand(0.1,0.9) + 0.1) through
/// FUN_0011d448; FUN_0011b7e0(buf, v) -> rows to +0x40 / +0x50 / +0x60.
/// PARTIAL: 4th lane of v is uninitialised stack in the original (0 here); arg of FUN_0011b7e0 taken as the quadword.
pub fn ee_sub_1d08d8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001c_e9b8, &[o], &[]);
    let x = callf(m, 0x0011_d448, &[], &[s(-1.0), s(1.0)]);
    let y = callf(m, 0x0011_d448, &[], &[s(-1.0), s(1.0)]);
    let z = callf(m, 0x0011_d448, &[], &[s(f32::from_bits(0x3dcc_cccd)), s(f32::from_bits(0x3f66_6666))]);
    let z = z + f32::from_bits(0x3dcc_cccd);
    let buf = stack_push(m, 0x30);
    m.call(0x0011_b7e0, &[buf], &[[x, y, z, 0.0]]);
    for i in 0..3u32 {
        let r = m.rv(buf + 0x10 * i);
        m.wv(o + 0x40 + 0x10 * i, r);
    }
    stack_pop(m, 0x30);
}
/// FUN_001d0bf8: unless p2[1] is class 0x33 (FUN_00132b10): spawn FUN_001cf6b0(this, this[+0x1c], p2[0], q1, q2);
/// its +0x40 rows = FUN_001a7050(buf, 0, p2[1], row0 of the spawn); FUN_001ce980(this); returns 1 (else 0)
pub fn ee_sub_1d0bf8<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32, q1: V4, q2: V4) -> u32 {
    let p21 = m.r32(p2 + 4);
    if m.call(0x0013_2b10, &[p21, 0x33], &[]).0 != 0 {
        return 0;
    }
    let a1 = m.r32(o + 0x1c);
    let a2 = m.r32(p2);
    let r = m.call(0x001c_f6b0, &[o, a1, a2], &[q1, q2]).0;
    if r != 0 {
        let buf = stack_push(m, 0x30);
        ee_sub_1a7050(m, buf, 0, p21, r + 0x40);
        for i in 0..3u32 {
            let v = m.rv(buf + 0x10 * i);
            m.wv(r + 0x40 + 0x10 * i, v);
        }
        stack_pop(m, 0x30);
    }
    m.call(0x001c_e980, &[o], &[]);
    1
}

// ---------------------------------------------------------------- C2d7508
/// FUN_001d0e48 (slot 0, ctor): parent FUN_001ce920 (same args); +0x20 = 0.33; +0xa0 = DAT_002c1f40 * -0.2;
/// +0xc4 = -8.0
pub fn ee_sub_1d0e48<M: QMem + ?Sized>(m: &mut M, o: u32, pos: V4, a2: u32, f12: f32) {
    ee_sub_1ce920(m, o, pos, a2, f12);
    m.w32(o + 0x20, 0x3ea8_f5c3);
    let k = f32::from_bits(0xbe4c_cccd);
    let v = rov(0x002c_1f40).map(|x| x * k);
    m.wv(o + 0xa0, v);
    m.w32(o + 0xc4, 0xc100_0000);
}
/// FUN_001d10e8: FUN_001cea88(this, p2 == 0)
pub fn ee_sub_1d10e8<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32) {
    m.call(0x001c_ea88, &[o, (p2 == 0) as u32], &[]);
}
