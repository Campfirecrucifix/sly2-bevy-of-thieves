//! WP-Q5: run_001a4258 queue package: per-object light / shadow VIF upload blocks (FUN_001a4478 picks
//! up to 4 lights through LevelLight_distance and appends their 3-qword records, then a 5-qword shadow
//! projector block; FUN_001a4ad8 the explicit-list variant; FUN_001a4258 the capsule-chain visibility
//! test), the beam-quad builder FUN_001a5fb8, the active-pool / DList glue around 0x3dd4bc / 0x3e0e1c
//! and FUN_001a6340 (Euler angles in degrees). 1:1 against `QMem`; the DMA chain cursor is
//! DAT_002dea34 (+0xc of the frame chain 0x2dea28).
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};
use crate::engine::classes::q::q5util::{dist3, f4};

const CUR: u32 = 0x002d_ea34;
const UP: u32 = 0x002a_2dc0;
/// DAT_002bfc60: 3-qword "no light" record
const NO_LIGHT: u32 = 0x002b_fc60;

fn cross(a: V4, b: V4, w: f32) -> V4 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0], w]
}

fn dot3(a: V4, b: V4) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn len3(a: V4) -> f32 {
    dot3(a, a).sqrt()
}

fn sc(a: V4, k: f32) -> V4 {
    [a[0] * k, a[1] * k, a[2] * k, a[3] * k]
}

fn sub(a: V4, b: V4) -> V4 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2], a[3] - b[3]]
}

fn add(a: V4, b: V4) -> V4 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2], a[3] + b[3]]
}

/// `vadda ACC, vf0, a; vmaddx d, b, k` : (a.xyz, a.w + 1) + b * k
fn madd1(a: V4, b: V4, k: f32) -> V4 {
    [a[0] + b[0] * k, a[1] + b[1] * k, a[2] + b[2] * k, 1.0 + a[3] + b[3] * k]
}

/// append VIF UNPACK V4-32 (3 qwords to 3 * i + 0x15) and the 3 qwords at `src`
fn emit_light<M: QMem + ?Sized>(m: &mut M, i: u32, src: u32) {
    let c = m.r32(CUR);
    m.w32(c, (i * 3 + 0x15) | 0x6c03_c000);
    let c = c + 4;
    m.w32(CUR, c);
    for k in 0..12u32 {
        let w = m.r32(src + 4 * k);
        m.w32(c + 4 * k, w);
    }
    m.w32(CUR, c + 0x30);
}

/// pad the cursor with zero words until (cursor & 0xf) == `to`
fn pad<M: QMem + ?Sized>(m: &mut M, to: u32) {
    loop {
        let c = m.r32(CUR);
        if c & 0xf == to {
            break;
        }
        m.w32(c, 0);
        m.w32(CUR, c + 4);
    }
}

/// FUN_001a4258: is `p` within `r` + radius of one of the `n` 0x20-byte spheres at `l` {centre,
/// radius}, or within `r` of the tapered cone hull between consecutive ones (offset by the radius
/// difference, parameter in [0, 1])
pub fn ee_sub_1a4258<M: QMem + ?Sized>(m: &mut M, r: f32, p: V4, n: i32, l: u32) -> u32 {
    for i in 0..n.max(0) as u32 {
        let e = l + 0x20 * i;
        if dist3(p, m.rv(e)) < r + m.rf(e + 0x10) {
            return 1;
        }
    }
    let mut e = l;
    for _ in 1..n.max(1) {
        let a = m.rv(e);
        let b = m.rv(e + 0x20);
        let (ra, rb) = (m.rf(e + 0x10), m.rf(e + 0x30));
        let pa = sub(p, a);
        let d = sub(b, a);
        let len = len3(d);
        let c1 = cross(pa, d, pa[3]);
        let dn = sc(d, 1.0 / len);
        let f = (ra - rb) / len;
        let c2 = cross(dn, c1, c1[3]);
        let nn = sc(c2, 1.0 / len3(c2));
        let s = (1.0 - f * f).sqrt();
        let v = add(sc(dn, f), sc(nn, s));
        let a2 = madd1(a, v, ra);
        let q = sub(p, a2);
        let b2 = madd1(b, v, rb);
        let h = dot3(q, v);
        let ab = sub(b2, a2);
        let t = dot3(q, ab) / dot3(ab, ab);
        if h < r && 0.0 <= t && t <= 1.0 {
            return 1;
        }
        e += 0x20;
    }
    0
}

/// FUN_001a4478: light + shadow blocks for draw record `rec` {+4 -> local position, +0x10 light
/// enable, +0x14 / +0x18 light qwords / start, +0x1c shadow radius, +0x20 shadow length, +0x24 /
/// +0x28 / +0x2c shadow block}: the position (through matrix `mat` when given) selects up to 4 lights
/// (FUN_001a3f78 on `lv`); the shadow source is their weighted centroid (directional lights +0xe0 0
/// contribute position - DAT_002bfc90 * dir, point lights their slot +0xfc +0x30 origin), pushed out to
/// DAT_002bfc94 * radius; the block holds the FUN_0011e2d8 basis (z scaled to the length), the source,
/// {FUN_001a4258 camera visibility, near / length ratio}, ..
pub fn ee_sub_1a4478<M: QMem + ?Sized>(m: &mut M, lv: u32, mat: u32, rec: u32) {
    let sp = stack_push(m, 0x150);
    let pp = m.r32(rec + 4);
    let mut pos = m.rv(pp);
    if mat != 0 {
        let mm = m.rm(mat);
        for c in 0..3 {
            pos[c] = mm[0][c] * pos[0] + mm[1][c] * pos[1] + mm[2][c] * pos[2] + mm[3][c];
        }
    }
    m.wv(sp + 0x110, pos);
    m.call(0x001a_3f78, &[lv, 4, sp], &[pos]);
    if m.rf(rec + 0x10) == 0.0 {
        m.w32(rec + 0x14, 0);
        m.w32(rec + 0x18, 0);
    } else {
        let c = m.r32(CUR);
        m.w32(rec + 0x18, c);
        let mut i = 0u32;
        let mut lt = m.r32(sp + 4);
        while lt != 0 {
            emit_light(m, i, lt + 0x120);
            i += 1;
            if i >= 4 {
                break;
            }
            lt = m.r32(sp + 8 * i + 4);
        }
        while i < 4 {
            emit_light(m, i, NO_LIGHT);
            i += 1;
        }
        pad(m, 0);
        let c = m.r32(CUR);
        let s = m.r32(rec + 0x18);
        m.w32(rec + 0x14, (c.wrapping_sub(s) as i32 >> 4) as u32);
    }
    if m.rf(rec + 0x1c) == 0.0 {
        m.w32(rec + 0x24, 0);
        zero_shadow(m, rec);
        stack_pop(m, 0x150);
        return;
    }
    let mut wsum = 0.0f32;
    let mut acc = [0.0f32; 4];
    let mut i = 0u32;
    let mut lt = m.r32(sp + 4);
    while lt != 0 {
        if m.r32(lt + 0x158) != 0 {
            let w = m.rf(sp + 8 * i);
            match m.r32(lt + 0xe0) {
                0 => {
                    wsum += w;
                    let k = m.rf(0x002b_fc90);
                    let dir = m.rv(lt + 0xa0);
                    let src = madd1(pos, dir, -k);
                    acc = madd1(acc, src, w);
                }
                1 => {
                    wsum += w;
                    let o = vslot(m, lt, 30, &[], &[]).0;
                    let org = m.rv(o + 0x30);
                    acc = madd1(acc, org, w);
                }
                _ => {}
            }
        }
        i += 1;
        if i >= 4 {
            break;
        }
        lt = m.r32(sp + 8 * i + 4);
    }
    if wsum == 0.0 {
        m.w32(rec + 0x24, 0);
        zero_shadow(m, rec);
        stack_pop(m, 0x150);
        return;
    }
    let mut src = sc(acc, 1.0 / wsum);
    let d = sub(pos, src);
    let mut dist = len3(d);
    let near = m.rf(0x002b_fc94) * m.rf(rec + 0x1c);
    let dir;
    if dist < near {
        dir = if dist < 1.0 { m.rv(UP) } else { sc(d, 1.0 / dist) };
        dist = near;
        src = madd1(pos, dir, -near);
    } else {
        dir = sc(d, 1.0 / dist);
    }
    let len = dist + m.rf(rec + 0x20);
    let (a1, a2) = (m.rv(0x002a_2db0), m.rv(0x002a_2da0));
    m.call(0x0011_e2d8, &[sp + 0x30], &[dir, a1, a2]);
    let z = sc(dir, len);
    m.wv(sp + 0x50, z);
    m.call(0x0025_6d18, &[sp + 0x90], &[src]);
    let r0 = m.rv(sp + 0x30);
    m.wv(sp + 0x60, r0);
    let r1 = m.rv(sp + 0x40);
    m.wv(sp + 0x70, r1);
    m.wv(sp + 0x80, z);
    let rad = m.rf(rec + 0x1c);
    for o in [0x8c, 0x7c, 0x6c] {
        m.w32(sp + o, 0);
    }
    m.wf(sp + 0x9c, 1.0);
    m.wf(sp + 0xa4, dist / len);
    m.wv(sp + 0xb0, pos);
    m.wv(sp + 0xd0, add(src, z));
    m.wf(sp + 0xe0, rad * len / dist);
    m.wv(sp + 0xf0, madd1(src, z, 1.5));
    m.wf(sp + 0xc0, rad);
    m.w32(sp + 0x100, 0);
    let cam = m.rv(0x002d_dfd0);
    let cr = m.rf(0x002d_df88);
    let vis = ee_sub_1a4258(m, cr, cam, 3, sp + 0xb0);
    m.w32(sp + 0xa0, vis);
    let c = m.r32(CUR);
    m.w32(rec + 0x2c, c);
    m.w32(rec + 0x24, c + 0x10);
    pad(m, 0xc);
    let c = m.r32(CUR);
    m.w32(c, 0x6c05_c00e);
    for k in 0..20u32 {
        let w = m.r32(sp + 0x60 + 4 * k);
        m.w32(c + 4 + 4 * k, w);
    }
    m.w32(CUR, c + 4 + 0x50);
    let s = m.r32(rec + 0x2c);
    m.w32(rec + 0x28, ((c + 0x54).wrapping_sub(s) as i32 >> 4) as u32);
    stack_pop(m, 0x150);
}

fn zero_shadow<M: QMem + ?Sized>(m: &mut M, rec: u32) {
    m.w32(rec + 0x28, 0);
    m.w32(rec + 0x2c, 0);
}

/// FUN_001a4ad8: light block of record `rec` from the `n` lights at `l` (FUN_001a3968 refreshes
/// each), padded to 4 with the "no light" record
pub fn ee_sub_1a4ad8<M: QMem + ?Sized>(m: &mut M, _a0: u32, n: i32, l: u32, rec: u32) {
    let c = m.r32(CUR);
    m.w32(rec + 0x18, c);
    let mut i = 0u32;
    while (i as i32) < n {
        let lt = m.r32(l + 4 * i);
        m.call(0x001a_3968, &[lt], &[]);
        emit_light(m, i, lt + 0x120);
        i += 1;
    }
    while i < 4 {
        emit_light(m, i, NO_LIGHT);
        i += 1;
    }
    let c = m.r32(CUR);
    let s = m.r32(rec + 0x18);
    m.w32(rec + 0x14, (c.wrapping_sub(s) as i32 >> 4) as u32);
}

/// FUN_001a53a0: give `o` an active-pool node (FUN_0011ccc0 on 0x3e0e1c) pointing back at it
pub fn ee_sub_1a53a0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.r32(o + 0x14) == 0 {
        let n = m.call(0x0011_ccc0, &[0x003e_0e1c], &[]).0;
        m.w32(o + 0x14, n);
        m.w32(n, o);
    }
}

/// FUN_001a53e0: release `o`'s node: unlink from 0x3dd4bc when listed, FUN_00147e20, back to the pool
pub fn ee_sub_1a53e0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let n = m.r32(o + 0x14);
    if m.call(0x0011_a380, &[0x003d_d4bc, n], &[]).0 != 0 {
        let n = m.r32(o + 0x14);
        m.call(0x0011_a2e8, &[0x003d_d4bc, n], &[]);
    }
    let n = m.r32(o + 0x14);
    m.call(0x0014_7e20, &[0x003d_d490, n], &[]);
    let n = m.r32(o + 0x14);
    m.call(0x0011_cd98, &[0x003e_0e1c, n], &[]);
    m.w32(o + 0x14, 0);
}

/// FUN_001a5448: the child list / cursor of parent `p` (+0x64 / +0x70), or the world's (0x3dd4a0 /
/// 0x3dd4ac)
pub fn ee_sub_1a5448<M: QMem + ?Sized>(m: &mut M, p: u32, list: u32, cur: u32) {
    if p != 0 {
        m.w32(list, p + 100);
        m.w32(cur, p + 0x70);
    } else {
        m.w32(list, 0x003d_d4a0);
        m.w32(cur, 0x003d_d4ac);
    }
}

/// FUN_001a57a8: unlink `o` from its parent's (+0x10) child list, advancing the cursor past it
pub fn ee_sub_1a57a8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let sp = stack_push(m, 0x10);
    let p = m.r32(o + 0x10);
    ee_sub_1a5448(m, p, sp, sp | 4);
    let cur = m.r32(sp + 4);
    if m.r32(cur) == o {
        let nx = m.r32(o + 0x24);
        m.w32(cur, nx);
    }
    let l = m.r32(sp);
    m.call(0x0011_a2e8, &[l, o], &[]);
    stack_pop(m, 0x10);
}

/// C2c94e0 slot 27 (FUN_001a5be8): 0
pub fn ee_sub_1a5be8() -> u32 {
    0
}

/// FUN_001a5d08: virtual slot 8
pub fn ee_sub_1a5d08<M: QMem + ?Sized>(m: &mut M, o: u32) {
    vslot(m, o, 8, &[], &[]);
}

/// FUN_001a5d30: push a callback node (FUN_001a5d58) running FUN_001a5d08
pub fn ee_sub_1a5d30<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let n = ee_sub_1a5d58(m, o);
    m.w32(n, 0x001a_5d08);
}

/// FUN_001a5d58: new node (FUN_00210100 on 0x3dd490) pushed on `o`+0x34
pub fn ee_sub_1a5d58<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let n = m.call(0x0021_0100, &[0x003d_d490], &[]).0;
    let h = m.r32(o + 0x34);
    m.w32(n + 8, h);
    m.w32(o + 0x34, n);
    n
}

/// FUN_001a5fb8: camera-facing beam quad from `a` to `b` of half-width `w`: with c = (a - camera) x
/// (b - a) not degenerate (> 0.01), the frame {normalize(c x d), c * w / |c|, d * 0.01, a} goes to
/// FUN_001a5ee8(o, frame, k, k)
pub fn ee_sub_1a5fb8<M: QMem + ?Sized>(m: &mut M, w: f32, k: f32, o: u32, a: V4, b: V4) {
    let d = sub(b, a);
    let cam = m.rv(0x002d_dfd0);
    let e = sub(a, cam);
    let c = cross(e, d, e[3]);
    let cl = len3(c);
    if !(0.01 < cl) {
        return;
    }
    let sp = stack_push(m, 0x70);
    let s = w / cl;
    let n = cross(c, d, c[3]);
    let n = sc(n, 1.0 / len3(n));
    m.wv(sp, n);
    m.wv(sp + 0x10, sc(c, s));
    m.wv(sp + 0x20, sc(d, 0.01));
    m.call(0x0025_6d18, &[sp + 0x60], &[a]);
    for i in 0..3u32 {
        let v = m.rv(sp + 0x10 * i);
        m.wv(sp + 0x30 + 0x10 * i, v);
        m.w32(sp + 0x3c + 0x10 * i, 0);
    }
    m.wf(sp + 0x6c, 1.0);
    m.call(0x001a_5ee8, &[o, sp + 0x30], &[f4(k), f4(k)]);
    stack_pop(m, 0x70);
}

/// FUN_001a6340: Euler angles (FUN_0011bff0 of the FUN_001a6318 frame) in degrees, packed as the
/// original (w = sign extension of z)
pub fn ee_sub_1a6340<M: QMem + ?Sized>(m: &mut M, o: u32) -> V4 {
    let f = m.call(0x001a_6318, &[o], &[]).0;
    let e = m.call(0x0011_bff0, &[f], &[]).1;
    let k = 57.295776f32;
    let z = e[2] * k;
    let w = if z.to_bits() & 0x8000_0000 != 0 { f32::from_bits(!0) } else { 0.0 };
    [e[0] * k, e[1] * k, z, w]
}
