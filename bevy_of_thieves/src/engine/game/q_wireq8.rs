//! WIRE-Q8: 1:1 queue ports (against `QMem`) of live-frontier natives (wireall_frontier.tsv: functions the
//! live dispatch reaches whose only port is a state-struct / host one). Ported from the EE asm; callees go
//! through `m.call` (the live dispatch routes ported ones).
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem, V4};

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

/// FUN_002121c8(s): sequencer idle = +0x58 < 2 && +0x64 < 2.
pub fn ee_sub_2121c8<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    ((m.r32(a0 + 0x58) as i32) < 2 && (m.r32(a0 + 0x64) as i32) < 2) as u32
}

/// FUN_00204d48(po, s): +0x1f4.. = DAT_002c3cb8 (12 bytes) scaled (x*s, y*s, z/sqrt(s)); +0x1f0 = s.
pub fn ee_sub_204d48<M: QMem + ?Sized>(m: &mut M, a0: u32, s: f32) {
    for k in 0..3 {
        let w = m.r32(0x002c_3cb8 + 4 * k);
        m.w32(a0 + 0x1f4 + 4 * k, w);
    }
    let x = m.rf(a0 + 0x1f4) * s;
    let y = m.rf(a0 + 0x1f8) * s;
    m.wf(a0 + 0x1f4, x);
    m.wf(a0 + 0x1f8, y);
    let inv = 1.0 / s.sqrt();
    m.wf(a0 + 0x1f0, s);
    let z = m.rf(a0 + 0x1fc) * inv;
    m.wf(a0 + 0x1fc, z);
}

/// FUN_00204660(o, f): +0x1a8 = f; FUN_0011b500(tmp, q=DAT_002a2dc0) builds a 3x4 matrix copied to +0x1b0..+0x1e0.
pub fn ee_sub_204660<M: QMem + ?Sized>(m: &mut M, a0: u32, f: f32) {
    let sp = stack_push(m, 0x50);
    m.wf(a0 + 0x1a8, f);
    let q = m.rv(0x002a_2dc0);
    m.call(0x0011_b500, &[sp], &[q, f4(f)]); // f12 passes through
    for k in 0..3 {
        let v = m.rv(sp + 0x10 * k);
        m.wv(a0 + 0x1b0 + 0x10 * k, v);
    }
    stack_pop(m, 0x50);
}

/// FUN_0012dc18(a0, a1, a2, a3, t0, q): r = FUN_0012db50(1, a0, a1, a2, a3); r+0x1c = t0; r+0x20 = q;
/// FUN_00230018(a0, a1, r); returns r (pin constraint).
pub fn ee_sub_12dc18<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, t0: u32, q: V4) -> u32 {
    let r = m.call(0x0012_db50, &[1, a0, a1, a2, a3], &[]).0;
    m.w32(r + 0x1c, t0);
    m.wv(r + 0x20, q);
    m.call(0x0023_0018, &[a0, a1, r], &[]);
    r
}

/// FUN_001f0730(o, goal): unless FUN_001f0638(o, goal) or goal == -1: idx = FUN_001f0400(o+0x24, goal);
/// if idx != -1 and FUN_001f0b80(o+0x24, o+0x48, cur, idx, 0) then FUN_001f0660(o, goal). cur = +0x38 if +0x30 else +0x34.
pub fn ee_sub_1f0730<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if m.call(0x001f_0638, &[a0, a1], &[]).0 != 0 || a1 == u32::MAX {
        return;
    }
    let g = m.r32(a0 + 0x24);
    let idx = m.call(0x001f_0400, &[g, a1], &[]).0;
    if idx == u32::MAX {
        return;
    }
    let cur = if m.r32(a0 + 0x30) != 0 { m.r32(a0 + 0x38) } else { m.r32(a0 + 0x34) };
    let (g, mask) = (m.r32(a0 + 0x24), m.r32(a0 + 0x48));
    if m.call(0x001f_0b80, &[g, mask, cur, idx, 0], &[]).0 != 0 {
        m.call(0x001f_0660, &[a0, a1], &[]);
    }
}

/// FUN_001f0b80(g, mask, from, to): collect the edges (24-byte records at +0x5c, count +0x58) enabled by mask
/// that leave `from` one hop closer to `to` (FUN_001f0498 distance; bit 0 = bidirectional), then
/// FUN_001f0ad0(g, n, list) picks one; 0 when none.
pub fn ee_sub_1f0b80<M: QMem + ?Sized>(m: &mut M, a0: u32, mask: u32, from: u32, to: u32) -> u32 {
    let sp = stack_push(m, 0x1b0);
    let d = if from == to { 1 } else { m.call(0x001f_0498, &[a0, mask, from, to], &[]).0 };
    let want = d.wrapping_sub(1);
    let mut n = 0u32;
    let mut i = 0i32;
    while i < m.r32(a0 + 0x58) as i32 {
        let e = m.r32(a0 + 0x5c).wrapping_add(0x18 * i as u32);
        if m.r32(e + 0xc) & mask != 0 {
            let mut hit = false;
            if m.r32(e + 4) == from {
                let b = m.r32(e + 8);
                hit = m.call(0x001f_0498, &[a0, mask, b, to], &[]).0 == want;
            }
            if !hit && m.r32(e + 0xc) & 1 != 0 && m.r32(e + 8) == from {
                let b = m.r32(e + 4);
                hit = m.call(0x001f_0498, &[a0, mask, b, to], &[]).0 == want;
            }
            if hit {
                m.w32(sp + 4 * n, e);
                n += 1;
            }
        }
        i += 1;
    }
    let r = if n != 0 { m.call(0x001f_0ad0, &[a0, n, sp], &[]).0 } else { 0 };
    stack_pop(m, 0x1b0);
    r
}

/// FUN_0011bff0(mat): matrix -> euler (x, y, z, sign-extension word of z) via asin(clamp(-m[2])) / atan2.
pub fn ee_sub_11bff0<M: QMem + ?Sized>(m: &mut M, a0: u32) -> V4 {
    let n8 = -m.rf(a0 + 8);
    let c = m.call(0x0011_cfe0, &[], &[f4(n8), f4(1.0)]).1[0];
    let y = m.call(0x0024_57f0, &[], &[f4(c)]).1[0];
    let lim = f32::from_bits(0x3fc9_0c94);
    let at = |m: &mut M, a: f32, b: f32| m.call(0x0024_58f0, &[], &[f4(a), f4(b)]).1[0];
    let (x, z) = if y < lim && -lim < y {
        let (p, q) = (m.rf(a0 + 0x18), m.rf(a0 + 0x28));
        let x = at(m, p, q);
        let (p, q) = (m.rf(a0 + 4), m.rf(a0));
        (x, at(m, p, q))
    } else if y < lim {
        let (p, q) = (m.rf(a0 + 0x10), m.rf(a0 + 0x20));
        (0.0, at(m, -p, -q))
    } else {
        let (p, q) = (m.rf(a0 + 0x10), m.rf(a0 + 0x20));
        (0.0, at(m, -p, q))
    };
    let w = if (z.to_bits() as i32) < 0 { f32::from_bits(u32::MAX) } else { 0.0 };
    [x, y, z, w]
}

fn mat_apply<M: QMem + ?Sized>(m: &mut M, rows: [u32; 4], v: [f32; 3], w: f32) -> V4 {
    let r: Vec<V4> = rows.iter().map(|&a| m.rv(a)).collect();
    let mut o = [0.0, 0.0, 0.0, w];
    for k in 0..3 {
        o[k] = r[0][k] * v[0] + r[1][k] * v[1] + r[2][k] * v[2] + r[3][k];
    }
    o
}

/// FUN_0014f268(cm, q): screen -> world. v = (z, -x, y) (x/y scaled by z when z > 1e-4) then by cm+0x40/+0x44,
/// transformed by the 4x3 matrix at cm+0x60 (rows +0x60/+0x70/+0x80, translation +0x90).
/// (returned .w is the stale VU vf06.w in the original; 0 here.)
pub fn ee_sub_14f268<M: QMem + ?Sized>(m: &mut M, a0: u32, q: V4) -> V4 {
    let (x, y, z) = (q[0], q[1], q[2]);
    let (mut b, mut c) = (-x, y);
    if f32::from_bits(0x38d1_b717) < z {
        c = y * z;
        b = b * z;
    }
    let b = b * m.rf(a0 + 0x40);
    let c = c * m.rf(a0 + 0x44);
    mat_apply(m, [a0 + 0x60, a0 + 0x70, a0 + 0x80, a0 + 0x90], [z, b, c], 0.0)
}

/// FUN_0014f308(cm, q): world -> screen: inverse (FUN_0011c318) of the cm+0x60 matrix applied to q, then
/// (-y/+0x40, z/+0x44, x), the first two divided by x when x > 1e-4.
pub fn ee_sub_14f308<M: QMem + ?Sized>(m: &mut M, a0: u32, q: V4) -> V4 {
    let sp = stack_push(m, 0x80);
    m.wv(sp + 0x50, q);
    m.call(0x0011_c318, &[sp + 0x10, a0 + 0x60], &[]);
    let w = m.rv(sp + 0x10)[3];
    let r = mat_apply(m, [sp + 0x10, sp + 0x20, sp + 0x30, sp + 0x40], [q[0], q[1], q[2]], w);
    stack_pop(m, 0x80);
    let mut f3 = r[2] / m.rf(a0 + 0x44);
    let mut f1 = -r[1] / m.rf(a0 + 0x40);
    if f32::from_bits(0x38d1_b717) < r[0] {
        f3 /= r[0];
        f1 /= r[0];
    }
    [f1, f3, r[0], r[3]]
}

/// FUN_002183e8(mv, idx, xf, pa, pb, dt): movie element draw/advance. Element e = mv+8[idx] (0x18 bytes, lh +8
/// = clip; < 0 -> nothing). With a parent 2x3 transform (mv+0x2c) xf is pre-composed into a stack copy.
/// e+0x10 set: FUN_002188b0(clip, idx, xf', &*pa, &*pb); else e+0xc time += dt (reset +0xa/+0xc at the clip
/// end, clip+8) then FUN_00219fe0(clip, idx, xf', &*pa, &*pb).
pub fn ee_sub_2183e8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, t0: u32, dt: f32) {
    let sp = stack_push(m, 0x70);
    let e = m.r32(a0 + 8).wrapping_add(a1.wrapping_mul(0x18));
    let clip = m.r16(e + 8) as i16 as i32;
    if clip >= 0 {
        let p = m.r32(a0 + 0x2c);
        let mut t2 = a2;
        if p != 0 {
            let x: Vec<f32> = (0..6).map(|k| m.rf(a2 + 4 * k)).collect();
            let q: Vec<f32> = (0..6).map(|k| m.rf(p + 4 * k)).collect();
            m.wf(sp + 0x40, x[0] * q[0] + x[1] * q[2]);
            m.wf(sp + 0x44, x[0] * q[1] + x[1] * q[3]);
            m.wf(sp + 0x48, x[2] * q[0] + x[3] * q[2]);
            m.wf(sp + 0x4c, x[2] * q[1] + x[3] * q[3]);
            m.wf(sp + 0x50, x[4] * q[0] + x[5] * q[2] + q[4]);
            m.wf(sp + 0x54, x[4] * q[1] + x[5] * q[3] + q[5]);
            let (u, v) = (m.rv(sp + 0x40), m.rv(sp + 0x50));
            m.wv(sp, u);
            m.wv(sp + 0x20, u);
            m.wv(sp + 0x10, v);
            m.wv(sp + 0x30, v);
            t2 = sp;
        }
        let c = m.r32(a0 + 4).wrapping_add((clip as u32).wrapping_mul(0x18));
        let target = if m.r32(e + 0x10) != 0 {
            0x0021_88b0
        } else {
            let t = m.rf(e + 0xc);
            if m.rf(c + 8) <= t {
                m.w16(e + 0xa, 0);
                m.w32(e + 0xc, 0);
            } else {
                m.wf(e + 0xc, t + dt);
            }
            0x0021_9fe0
        };
        let (wa, wb) = (m.r32(a3), m.r32(t0));
        m.w32(sp + 0x20, wa);
        m.w32(sp + 0x30, wb);
        m.call(target, &[c, a1, t2, sp + 0x20, sp + 0x30], &[]);
    }
    stack_pop(m, 0x70);
}

/// FUN_0020faa0(w, a, b, filter, ctx, pcount, plist): gather SOs whose cell boxes (w+0x3d1c list, next +0x40,
/// min +0x0 / max +0x10, SO chain +0x34 / next +0x60) overlap box(min(a,b), max(a,b)); optional
/// filter(ctx, so) != 0; at most 0x100 into a FUN_001ae318({0x400, 0x10}) buffer; *plist = buf, *pcount = n.
pub fn ee_sub_20faa0<M: QMem + ?Sized>(m: &mut M, a0: u32, a: V4, b: V4, f: u32, ctx: u32, pn: u32, pl: u32) {
    let sp = stack_push(m, 0xd0);
    let lo: Vec<f32> = (0..3).map(|k| a[k].min(b[k])).collect();
    let hi: Vec<f32> = (0..3).map(|k| a[k].max(b[k])).collect();
    m.w32(sp + 0x20, 0x400);
    m.w32(sp + 0x24, 0x10);
    let buf = m.call(0x001a_e318, &[sp + 0x20], &[]).0;
    m.call(0x0013_3e30, &[a0], &[]);
    m.call(0x0013_3e80, &[a0], &[]);
    let mut n = 0u32;
    let mut c = m.r32(a0 + 0x3d1c);
    while c != 0 {
        let mut out = false;
        for k in 0..3 {
            if hi[k] < m.rf(c + 4 * k as u32) || m.rf(c + 0x10 + 4 * k as u32) < lo[k] {
                out = true;
                break;
            }
        }
        if !out {
            let mut s = m.r32(c + 0x34);
            while s != 0 {
                let ok = f == 0 || m.call(f, &[ctx, s], &[]).0 != 0;
                if ok && (n as i32) < 0x100 {
                    m.w32(buf + 4 * n, s);
                    n += 1;
                }
                s = m.r32(s + 0x60);
            }
        }
        c = m.r32(c + 0x40);
    }
    m.w32(pl, buf);
    m.w32(pn, n);
    stack_pop(m, 0xd0);
}

/// FUN_001619a0(a0, a1, a2): FUN_00161a58(a0, a1, a2); FUN_00154a90(0x2de578, a1, a2); for each actor in the
/// 0x3e0f90 list (next +0x88) whose +0x7c is the current world (0x3dd490+0x3b0c): FUN_00165170(o, a1, a2);
/// then FUN_00162bb8(world, a1, a2) when set.
pub fn ee_sub_1619a0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    m.call(0x0016_1a58, &[a0, a1, a2], &[]);
    m.call(0x0015_4a90, &[0x002d_e578, a1, a2], &[]);
    let mut s = m.r32(0x003e_0f90);
    while s != 0 {
        if m.r32(s + 0x7c) == m.r32(0x003d_d490 + 0x3b0c) {
            m.call(0x0016_5170, &[s, a1, a2], &[]);
        }
        s = m.r32(s + 0x88);
    }
    let w = m.r32(0x003d_d490 + 0x3b0c);
    if w != 0 {
        m.call(0x0016_2bb8, &[w, a1, a2], &[]);
    }
}

/// FUN_002121e8(o, id): id 0x314 -> FUN_00218738(o+0x84) == 0; else search the o+0x84 tables (o+0x80 of
/// 0x30-byte entries, each lhu count + 0x18-byte records with id at +0x14) -> FUN_00218710(entry, id, j+1, n).
pub fn ee_sub_2121e8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    if a1 == u32::MAX || m.r32(a0 + 0x7c) == u32::MAX {
        return 0;
    }
    if a1 == 0x314 {
        let p = m.r32(a0 + 0x84);
        return (m.call(0x0021_8738, &[p], &[]).0 == 0) as u32;
    }
    let n = m.r16(a0 + 0x80) as i32;
    let mut e = m.r32(a0 + 0x84);
    for _ in 0..n {
        let cnt = m.r16(e) as i32;
        if cnt != 0 {
            let mut r = m.r32(e + 4) + 0x14;
            for j in 0..cnt {
                if m.r32(r) == a1 {
                    return m.call(0x0021_8710, &[e, a1, (j + 1) as u32, cnt as u32], &[]).0;
                }
                r += 0x18;
            }
        }
        e += 0x30;
    }
    0
}

/// FUN_001cb950(o): FUN_001a5c48(o); if +0x38 is the unset marker (0x501502f9): +0x38 = FUN_001e7d90(o) + DAT_002c1ec0.
pub fn ee_sub_1cb950<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5c48, &[a0], &[]);
    if m.rf(a0 + 0x38) == f32::from_bits(0x5015_02f9) {
        let t = m.call(0x001e_7d90, &[a0], &[]).1[0];
        let d = m.rf(0x002c_1ec0);
        m.wf(a0 + 0x38, t + d);
    }
}

/// FUN_001d3f30(o, a1): FUN_001f5988(o, a1); FUN_001f4da8(o, a1, o+0x1a8+0x60*i) for i < +0x190.
pub fn ee_sub_1d3f30<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.call(0x001f_5988, &[a0, a1], &[]);
    let mut i = 0;
    while i < m.r32(a0 + 0x190) as i32 {
        m.call(0x001f_4da8, &[a0, a1, a0 + 0x1a8 + 0x60 * i as u32], &[]);
        i += 1;
    }
}

/// FUN_001c76d0(o, a1): under a pushed heap mark (FUN_001ae478 / FUN_001ae4a0(mark, 2)): +0x240 =
/// FUN_001ae318({0x580, 0x10}); FUN_001c4ef8(o, a1); FUN_001c7a08(o); +0x240 = FUN_001ae238({+0x23c*0x2c, 0x10}, +0x240).
pub fn ee_sub_1c76d0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let sp = stack_push(m, 0x70);
    m.call(0x001a_e478, &[sp], &[]);
    m.w32(sp + 0x10, 0x580);
    m.w32(sp + 0x14, 0x10);
    let r = m.call(0x001a_e318, &[sp + 0x10], &[]).0;
    m.w32(a0 + 0x240, r);
    m.call(0x001c_4ef8, &[a0, a1], &[]);
    m.call(0x001c_7a08, &[a0], &[]);
    let n = m.r32(a0 + 0x23c).wrapping_mul(0x2c);
    m.w32(sp + 0x10, n);
    m.w32(sp + 0x14, 0x10);
    let p = m.r32(a0 + 0x240);
    let r = m.call(0x001a_e238, &[sp + 0x10, p], &[]).0;
    m.w32(a0 + 0x240, r);
    m.call(0x001a_e4a0, &[sp, 2], &[]);
    stack_pop(m, 0x70);
}

/// FUN_001e6180(path): FUN_001e6330(path); +0x100..+0x140 = *vslot30(path) (world matrix). Polyline (+0xe0 == 0):
/// cumulative arc lengths of the transformed points (+0xf4, count +0xe8) into +0xf0[i]; spline: per segment
/// FUN_001e5fc8(path, i) then len[i+1] = len[i] + +0x174. Then FUN_001e9828(+0x17c) when set.
pub fn ee_sub_1e6180<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    use crate::engine::classes::q::q2mem::vslot;
    m.call(0x001e_6330, &[a0], &[]);
    let mp = vslot(m, a0, 30, &[], &[]).0;
    for k in 0..4 {
        let q = m.rv(mp + 0x10 * k);
        m.wv(a0 + 0x100 + 0x10 * k, q);
    }
    if m.r32(a0 + 0xe0) == 0 {
        let rows = [a0 + 0x100, a0 + 0x110, a0 + 0x120, a0 + 0x130];
        let pts = m.r32(a0 + 0xf4);
        let lens = m.r32(a0 + 0xf0);
        let p0 = m.rv(pts);
        let mut prev = mat_apply(m, rows, [p0[0], p0[1], p0[2]], 0.0);
        m.wf(lens, 0.0);
        let mut acc = 0.0f32;
        let mut i = 1;
        while i < m.r32(a0 + 0xe8) as i32 {
            let pts = m.r32(a0 + 0xf4);
            let p = m.rv(pts + 0x10 * i as u32);
            let q = mat_apply(m, rows, [p[0], p[1], p[2]], 0.0);
            let (dx, dy, dz) = (prev[0] - q[0], prev[1] - q[1], prev[2] - q[2]);
            acc += (dx * dx + dy * dy + dz * dz).sqrt();
            let lens = m.r32(a0 + 0xf0);
            m.wf(lens + 4 * i as u32, acc);
            prev = q;
            i += 1;
        }
    } else {
        let lens = m.r32(a0 + 0xf0);
        m.w32(lens, 0);
        let mut i = 0;
        while i < m.r32(a0 + 0xe8) as i32 - 1 {
            m.call(0x001e_5fc8, &[a0, i as u32], &[]);
            let l = m.r32(a0 + 0xf0) + 4 * i as u32;
            let v = m.rf(l) + m.rf(a0 + 0x174);
            m.wf(l + 4, v);
            i += 1;
        }
    }
    let s = m.r32(a0 + 0x17c);
    if s != 0 {
        m.call(0x001e_9828, &[s], &[]);
    }
}

/// FUN_0017da28(s): guard look/aim update. Skips when +0x30 == -1, actor flag 0xfe8&2 clear or FUN_001ef0a8(s).
/// Picks mode 1 (own target +0x60/+0x70), 2 (target object +0xb8), 3 (target in reach: ray param in (0,1),
/// xy dist < 50, |dz| < 300), 5 (FUN_001c02d0 path test failed: +0x80/+0x90 = target); FUN_0017d450(s, mode);
/// +0xa0 = DAT_002dded0, +0xa4 = *(0x3e1a5c)+0x28.
pub fn ee_sub_17da28<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    if m.r32(a0 + 0x30) == u32::MAX {
        return;
    }
    let act = m.r32(a0 + 0x10);
    if m.r32(act + 0xfe8) & 2 == 0 || m.call(0x001e_f0a8, &[a0], &[]).0 != 0 {
        return;
    }
    let sp = stack_push(m, 0xa0);
    m.call(0x0017_d450, &[a0, 0], &[]);
    m.call(0x0017_d8a8, &[a0], &[]);
    m.call(0x0025_62a8, &[sp], &[]);
    let mut mode: i32;
    if m.r32(a0 + 0xb4) == 0 {
        let (u, v) = (m.rv(a0 + 0x60), m.rv(a0 + 0x70));
        m.wv(sp, u);
        m.wv(sp + 0x10, v);
        mode = 1;
    } else {
        let t = m.r32(a0 + 0xb8);
        let (u, v) = (m.rv(t + 0x60), m.rv(t + 0x70));
        m.wv(sp, u);
        m.wv(sp + 0x10, v);
        mode = 2;
        if m.r32(t + 0x120) == 0 {
            let act = m.r32(a0 + 0x10);
            let pos = m.call(0x001a_62e8, &[act], &[]).1;
            let a = m.call(0x001c_1ec0, &[t + 0x60], &[pos]).1;
            let b = m.call(0x001c_1ec0, &[t + 0x80], &[pos]).1;
            m.call(0x0011_ee08, &[sp + 0x20, sp + 0x30, 0], &[pos, a, b]);
            let u = m.rf(sp + 0x30);
            let mut hit = false;
            if 0.0 < u && u < 1.0 {
                let h = m.rv(sp + 0x20);
                let (dx, dy) = (h[0] - pos[0], h[1] - pos[1]);
                if (dx * dx + dy * dy).sqrt() < 50.0 {
                    hit = (m.rf(sp + 0x28) - pos[2]).abs() < 300.0;
                }
            }
            if hit {
                mode = 3;
            }
        }
    }
    if mode > 0 && mode < 3 {
        let act = m.r32(a0 + 0x10);
        let s1 = m.r32(act + 0x294);
        let s0 = m.call(0x001a_62e8, &[act], &[]).1;
        let a2 = m.call(0x001c_1ec0, &[sp], &[s0]).1;
        if m.r32(a0 + 0xbc) != 0 {
            m.w32(s1 + 0xe8, 2);
            let p = m.r32(s1 + 0xf4);
            m.wv(p, s0);
            let p = m.r32(s1 + 0xf4);
            m.wv(p + 0x10, a2);
            m.call(0x001e_6180, &[s1], &[]);
        } else {
            let act = m.r32(a0 + 0x10);
            let v = m.r32(act + 0xe8);
            let v1 = if v != 0 { m.r32(v + 0xdc) } else { 0 };
            let w = if v1 != 0 { m.r32(v1 + 0x10) } else { 0 };
            if m.call(0x001c_02d0, &[w, s1], &[s0, a2]).0 == 0 {
                mode = 5;
                let (u, v) = (m.rv(sp), m.rv(sp + 0x10));
                m.wv(a0 + 0x80, u);
                m.wv(a0 + 0x90, v);
            }
        }
    }
    m.call(0x0017_d450, &[a0, mode as u32], &[]);
    let t = m.rf(0x002d_ded0);
    m.wf(a0 + 0xa0, t);
    let g = m.r32(0x003e_1a5c);
    let v = m.r32(g + 0x28);
    m.w32(a0 + 0xa4, v);
    stack_pop(m, 0xa0);
}

/// FUN_00228e50(z): sound/notice zone update. Marks the previous members (+0x150 list) with flag bit 31 of
/// +0x148, re-collects the actors of the level list (+0xec)+0x64 that pass FUN_001f6838 / FUN_00230380 /
/// FUN_00229468 (bit 32 = inside now), pushes a FUN_001fbb90(0x13) effect + vslot50 event (+0x170) per open
/// contact (FUN_0014cdb0) of falling actors (vslot32 velocity z < -100), then sends vslot3(4 enter / 5 leave)
/// to members whose bit 31 != bit 32 and prunes the list (FUN_00228d20).
pub fn ee_sub_228e50<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    use crate::engine::classes::q::q2mem::vslot;
    m.call(0x001f_36e0, &[a0], &[]);
    if m.r32(a0 + 0xe0) == 0 {
        return;
    }
    let sp = stack_push(m, 0x760);
    let mut s = m.r32(a0 + 0x150);
    while s != 0 {
        let o = m.r32(s + 4);
        let lo = m.r32(o + 0x148);
        m.w32(o + 0x148, lo | 0x8000_0000);
        s = m.r32(s + 0xc);
    }
    m.call(0x001a_e478, &[sp], &[]);
    let n = m.r32(a0 + 0xe0) << 2;
    m.w32(sp + 0x10, n);
    m.w32(sp + 0x14, 0x10);
    let s6 = m.call(0x001a_e318, &[sp + 0x10], &[]).0;
    let r690 = m.call(0x001f_7e08, &[a0, 0x0022_9d60, 0, s6], &[]).0;
    let lv = m.r32(a0 + 0xec);
    let mut s5 = m.r32(lv + 0x64);
    while s5 != 0 {
        let s1 = m.r32(s5 + 0xc);
        'one: {
            let v = m.r32(s1 + 0xe8);
            if v == 0 || m.rf(v + 0xd4) < f32::from_bits(0x38d1_b717) {
                break 'one;
            }
            if m.call(0x001f_6838, &[s1], &[]).0 != 0 {
                break 'one;
            }
            let k = m.r32(s1 + 0x58);
            let id = m.r16(k + 0x98) as i16 as i32 as u32;
            if m.call(0x0023_0380, &[id], &[]).0 == 0 {
                break 'one;
            }
            if m.call(0x0022_9468, &[a0, s1, 0, 0], &[]).1[0] == 0.0 {
                break 'one;
            }
            let lo = m.r32(s1 + 0x148);
            let hi = m.r32(s1 + 0x14c);
            m.w32(s1 + 0x14c, hi | 1);
            if lo & 0x8000_0000 != 0 {
                break 'one;
            }
            let nd = m.call(0x0022_96a0, &[0x003d_d490], &[]).0;
            m.w32(nd, a0);
            m.w32(nd + 4, s1);
            let h = m.r32(a0 + 0x150);
            m.w32(nd + 0xc, h);
            m.w32(a0 + 0x150, nd);
            if m.r32(a0 + 0x170) == 0 {
                break 'one;
            }
            let q = vslot(m, s1, 32, &[], &[]).1;
            m.wv(sp + 0x10, q);
            if !(m.rf(sp + 0x18) < -100.0) {
                break 'one;
            }
            let mut i = 0;
            while i < m.r32(s1 + 0xe0) as i32 {
                let e = m.r32(s1 + 0xe4).wrapping_add(0xb0 * i as u32);
                i += 1;
                if m.r32(e + 0x14) & 0x0005_0000 != 0 {
                    continue;
                }
                let c0 = m.rv(0x002a_2dd0);
                m.wv(sp + 0x640, c0);
                let p = m.rv(e + 0x50);
                m.wv(sp + 0x20, [p[0] - c0[0], p[1] - c0[1], p[2] - c0[2], p[3] - c0[3]]);
                let eo = m.r32(e);
                let p = m.rv(eo + 0x70);
                m.wv(sp + 0x30, [p[0] - c0[0], p[1] - c0[1], p[2] - c0[2], p[3] - c0[3]]);
                let f2 = if m.call(0x0013_2b10, &[s1, 0x33], &[]).0 != 0 {
                    m.rf(s1 + 0x248)
                } else {
                    let w = m.r32(s1 + 0xec);
                    m.rf(w + 8) - m.rf(eo + 0x78)
                };
                let (a, b) = (m.rf(sp + 0x28) + f2, m.rf(sp + 0x38) + f2);
                m.wf(sp + 0x28, a);
                m.wf(sp + 0x38, b);
                let (qa, qb) = (m.rv(sp + 0x20), m.rv(sp + 0x30));
                let r = m.call(0x0014_cdb0, &[r690, s6, 0x10, sp + 0x40], &[qa, qb]).0;
                if r as i32 <= 0 {
                    continue;
                }
                let w = m.r32(s1 + 0xec);
                let f20 = (-m.rf(sp + 0x18)).min(600.0) * f32::from_bits(0x3ada_740e);
                let f21 = ((m.rf(w + 0x10) - m.rf(w)) + (m.rf(w + 0x14) - m.rf(w + 4))) * 0.5;
                let fx = m.call(0x001f_bb90, &[0x13], &[]).0;
                m.call(0x001f_b2f8, &[fx, s1], &[]);
                m.call(0x001f_b4f0, &[fx], &[f4(100.0), f4(1000.0)]);
                m.call(0x001f_b1f8, &[fx], &[f4(f20)]);
                m.call(0x0023_33b4, &[sp + 0x650, 0, 0x40], &[]);
                let h = m.rv(sp + 0x40);
                m.wv(sp + 0x670, [h[0] + c0[0], h[1] + c0[1], h[2] + c0[2], h[3] + c0[3]]);
                m.w32(sp + 0x650, 0xe);
                m.wf(sp + 0x658, f21 * f20);
                let z = m.rv(0x002a_2dc0);
                m.wv(sp + 0x660, z);
                let t = m.r32(a0 + 0x170);
                vslot(m, t, 50, &[sp + 0x650], &[]);
            }
        }
        s5 = m.r32(s5 + 4);
    }
    let mut s = m.r32(a0 + 0x150);
    while s != 0 {
        let t = m.r32(s + 4);
        let (lo, hi) = (m.r32(t + 0x148), m.r32(t + 0x14c));
        if hi & 1 != lo >> 31 {
            m.w32(sp + 0x10, a0);
            m.w32(sp + 0x14, t);
            let k = if lo & 0x8000_0000 != 0 { 5 } else { 4 };
            vslot(m, t, 3, &[k, sp + 0x10], &[]);
        }
        s = m.r32(s + 0xc);
    }
    m.call(0x0022_8d20, &[a0, a0 + 0x150], &[]);
    m.call(0x001a_e4a0, &[sp, 2], &[]);
    stack_pop(m, 0x760);
}

/// FUN_0011cfe0(x, lim): clamp x to [-lim, lim].
pub fn ee_sub_11cfe0<M: QMem + ?Sized>(_m: &mut M, x: f32, lim: f32) -> f32 {
    if lim < x {
        return lim;
    }
    if x < -lim {
        -lim
    } else {
        x
    }
}

/// FUN_0012db50(kind, a1, a2, a3, t0): act = FUN_0012e238(kind); +4/+8/+0xc/+0x14 = a1/a2/a3/t0.
pub fn ee_sub_12db50<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, t0: u32) -> u32 {
    let r = m.call(0x0012_e238, &[a0, a1, a2, a3, t0], &[]).0;
    m.w32(r + 4, a1);
    m.w32(r + 8, a2);
    m.w32(r + 0xc, a3);
    m.w32(r + 0x14, t0);
    r
}

/// FUN_0011b500(dst, axis, angle): (s, c) = FUN_0011d608(angle); FUN_0011b480(tmp, s, c, axis); dst = 0x30 bytes of tmp.
pub fn ee_sub_11b500<M: QMem + ?Sized>(m: &mut M, a0: u32, q: V4, f: f32) -> u32 {
    let sp = stack_push(m, 0x80);
    m.call(0x0011_d608, &[sp + 0x40, sp + 0x44], &[f4(f)]);
    let (s, c) = (m.rf(sp + 0x40), m.rf(sp + 0x44));
    m.call(0x0011_b480, &[sp], &[f4(s), f4(c), q]);
    m.call(0x0023_320c, &[a0, sp, 0x30], &[]);
    stack_pop(m, 0x80);
    a0
}

fn vcall18<M: QMem + ?Sized>(m: &mut M, obj: u32, off: u32) {
    let vt = m.r32(obj + 0x18);
    let adj = m.r16(vt + off) as i16 as i32 as u32;
    let f = m.r32(vt + off + 4);
    m.call(f, &[obj.wrapping_add(adj)], &[]);
}

/// FUN_00230018(a0, owner, act): insert act into owner+0x90 (next +0x10) by descending priority +0xc.
/// New head: FUN_0022c840(a0, owner), old head vt18 slot +0x10 (deactivate), new head vt18 slot +8 (activate);
/// same head: head vt18 slot +0x18 (refresh).
pub fn ee_sub_230018<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    let old = m.r32(a1 + 0x90);
    let mut link = a1 + 0x90;
    let pr = m.r32(a2 + 0xc) as i32;
    let mut v = old;
    while v != 0 && pr < m.r32(v + 0xc) as i32 {
        link = v + 0x10;
        v = m.r32(v + 0x10);
    }
    let nx = m.r32(link);
    m.w32(a2 + 0x10, nx);
    m.w32(link, a2);
    if m.r32(a1 + 0x90) != old {
        m.call(0x0022_c840, &[a0, a1], &[]);
        if old != 0 {
            vcall18(m, old, 0x10);
        }
        let h = m.r32(a1 + 0x90);
        vcall18(m, h, 8);
    } else if old != 0 {
        vcall18(m, old, 0x18);
    }
}

/// FUN_00133e80(w): when dirty (+0x3ccc, cleared), insertion-sort the 3 lists (+0x3cd4[k], +0x3cd0 entries of
/// 12 bytes {id, h, *key}) by *key ascending; each displaced pair with differing ids -> FUN_00133a38(w, h1, h2, 0).
pub fn ee_sub_133e80<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    if m.r32(a0 + 0x3ccc) == 0 {
        return;
    }
    m.w32(a0 + 0x3ccc, 0);
    if m.r32(a0 + 0x3cd0) == 0 {
        return;
    }
    for k in 0..3u32 {
        let base = m.r32(a0 + 0x3cd4 + 4 * k);
        let mut n = m.r32(a0 + 0x3cd0) as i32 - 1;
        let key = |m: &mut M, e: u32| {
            let p = m.r32(e + 8);
            m.rf(p)
        };
        let mut hi = key(m, base);
        let mut s1 = base + 0xc;
        while n > 0 {
            let f = key(m, s1);
            if hi <= f {
                hi = f;
            } else {
                let mut s0 = s1 - 0xc;
                while s0 >= base && !(key(m, s0) <= f) {
                    if m.r32(s1) != m.r32(s0) {
                        let (h1, h2) = (m.r32(s1 + 4), m.r32(s0 + 4));
                        m.call(0x0013_3a38, &[a0, h1, h2, 0], &[]);
                    }
                    s0 = s0.wrapping_sub(0xc);
                }
                let s0 = s0.wrapping_add(0xc);
                let t = [m.r32(s1), m.r32(s1 + 4), m.r32(s1 + 8)];
                m.call(0x0023_32b8, &[s0 + 0xc, s0, s1 - s0], &[]);
                for (j, w) in t.iter().enumerate() {
                    m.w32(s0 + 4 * j as u32, *w);
                }
            }
            n -= 1;
            s1 += 0xc;
        }
    }
}

/// FUN_00203af0(o, a1): append the next 0xb0 entry (+0xe4[+0xe0++] -> +0x258), FUN_001f70c0(a1, +0x60, 2|3, e),
/// FUN_001f6c08(e); +0x210 = min key (e+0x7c points .z, DAT_002a85f0 cap); append 2 points (+0x240 and
/// +0x240 + DAT_002a2dc0 * +0x214, w + 1 from VF00) [+ DAT_002a2dd0 when a1 is its own target];
/// FUN_001664a0; +0x218/+0x21c = the two new e+0x94 points. Bit 60 of +0x28 brackets with FUN_001f2ba8 / FUN_001f2b20.
pub fn ee_sub_203af0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let bit60 = |m: &mut M| m.r32(a0 + 0x2c) >> 28 & 1 != 0;
    if bit60(m) {
        m.call(0x001f_2ba8, &[a0, a1], &[]);
    }
    let v = m.r32(a0 + 0xe0);
    let e = m.r32(a0 + 0xe4).wrapping_add(v.wrapping_mul(0xb0));
    m.w32(a0 + 0xe0, v + 1);
    m.w32(a0 + 0x258, e);
    let mut s2 = false;
    if m.call(0x0013_2b10, &[a0, 0x34], &[]).0 == 0 {
        s2 = m.r32(a1 + 0x1c) == m.r32(a1 + 0x78);
    }
    let k = m.r32(a0 + 0x60);
    m.call(0x001f_70c0, &[a1, k, if s2 { 3 } else { 2 }, e], &[]);
    let e = m.r32(a0 + 0x258);
    m.call(0x001f_6c08, &[e], &[]);
    let e = m.r32(a0 + 0x258);
    let mut f = m.rf(0x002a_85f0);
    let mut i = 0;
    while i < m.r32(e + 0x1c) as i32 {
        let p = m.r32(e + 0x7c);
        f = m.rf(p + 16 * i as u32 + 8).min(f);
        i += 1;
    }
    m.wf(a0 + 0x210, f);
    let e = m.r32(a0 + 0x258);
    let n = m.r32(e + 0x1c);
    let s1 = n << 4;
    m.w32(e + 0x1c, n + 2);
    let o = m.rv(a0 + 0x240);
    let p = m.r32(e + 0x7c) + s1;
    m.wv(p, o);
    let x = m.rf(a0 + 0x214);
    let c = m.rv(0x002a_2dc0);
    let p = m.r32(e + 0x7c) + s1;
    m.wv(p + 0x10, [o[0] + c[0] * x, o[1] + c[1] * x, o[2] + c[2] * x, o[3] + 1.0 + c[3] * x]);
    if s2 {
        let e = m.r32(a0 + 0x258);
        let z = m.rv(0x002a_2dd0);
        let p = m.r32(e + 0x7c) + s1;
        m.wv(p + 0x20, z);
    }
    let e = m.r32(a0 + 0x258);
    let h = m.r32(e) + 0x40;
    m.call(0x0016_64a0, &[h, e + 0x78, e + 0x90], &[]);
    let e = m.r32(a0 + 0x258);
    let b = m.r32(e + 0x94);
    m.w32(a0 + 0x218, b + s1);
    let b = m.r32(e + 0x94);
    m.w32(a0 + 0x21c, b + s1 + 0x10);
    if bit60(m) {
        m.call(0x001f_2b20, &[a0], &[]);
    }
}

/// FUN_001f0498(g, mask, from, to): t = FUN_001f0458(g, mask); t ? (i8) t+4[from * g+0x50 + to] : 0 (hop distance).
pub fn ee_sub_1f0498<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32) -> u32 {
    let t = m.call(0x001f_0458, &[a0, a1, a2, a3], &[]).0;
    if t == 0 {
        return 0;
    }
    let w = m.r32(a0 + 0x50);
    let b = m.r32(t + 4).wrapping_add(a2.wrapping_mul(w)).wrapping_add(a3);
    m.r8(b) as i8 as i32 as u32
}

/// FUN_0022b490(o, xf, a2, a3, flags, t1): msg {flags, 0x40 bytes of xf (or o+0x3c40), a2, a3}; bit 1 when
/// the copy equals o+0x3c40 (+8 / +0x38 words); t1 != -1 -> FUN_0022b220(0x443740, msg, t1, 1) else FUN_0020df80(o, msg).
pub fn ee_sub_22b490<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, t0: u32, t1: u32) {
    let sp = stack_push(m, 0x60);
    let t2 = a0 + 0x3c40;
    let src = if a1 != 0 { a1 } else { t2 };
    for k in 0..16 {
        let w = m.r32(src + 4 * k);
        m.w32(sp + 4 + 4 * k, w);
    }
    m.w32(sp + 0x44, a2);
    m.w32(sp + 0x48, a3);
    m.w32(sp, t0);
    let v1 = sp + 4;
    let same = if v1 == t2 {
        true
    } else if t2 == 0 {
        false
    } else {
        (m.r32(v1 + 8) ^ m.r32(v1 + 0x38)) == (m.r32(t2 + 8) ^ m.r32(t2 + 0x38))
    };
    if same {
        let f = m.r32(sp) | 2;
        m.w32(sp, f);
    }
    if t1 != u32::MAX {
        m.call(0x0022_b220, &[0x0044_3740, sp, t1, 1], &[]);
    } else {
        m.call(0x0020_df80, &[a0, sp], &[]);
    }
    stack_pop(m, 0x60);
}

/// FUN_00141640(o, out): camera/target request fill. +0x1044 set: out+0x10 = +0x1060, +0x104c = +0x1068.
/// Else z clamp of vslot96(o, tmp).z around the actor z (FUN_001a62e8) with band DAT_002bcca0/a8/b0 (by +0x434 /
/// +0x2dc == 0x16); for the focus actor (0x2ddf40+0x260) out.z = clamp(FUN_0011d018 smoothing); then out.xy.
/// Tail: +0x1070 = FUN_0011f088 smoothed offset added to out+0x10 (-> +0x1060); +0x1050 = out.z; per-part
/// hooks (FUN_0019c8b0 / FUN_0018a4e8 / FUN_001a2160); out+0x34 *= clamp(poly(|vslot33 velocity|)).
pub fn ee_sub_141640<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    use crate::engine::classes::q::q2mem::vslot;
    let sp = stack_push(m, 0x60);
    m.call(0x001f_58f0, &[a0, a1], &[]);
    if m.r32(a0 + 0x1044) != 0 {
        let q = m.rv(a0 + 0x1060);
        m.wv(a1 + 0x10, q);
        let f = m.r32(a0 + 0x1068);
        m.w32(a0 + 0x104c, f);
    } else {
        let p = m.call(0x001a_62e8, &[a0], &[]).1;
        let z = p[2];
        vslot(m, a0, 96, &[sp], &[]);
        let band = if m.r32(a0 + 0x434) != 0 {
            0x002b_cca0
        } else if m.r32(a0 + 0x2dc) == 0x16 {
            0x002b_ccb0
        } else {
            0x002b_cca8
        };
        let (lo, hi) = (z + m.rf(band), z + m.rf(band + 4));
        let mut f0 = m.rf(sp + 8);
        if f0 < lo {
            f0 = lo;
        } else if hi < f0 {
            f0 = hi;
        }
        m.wf(a0 + 0x104c, f0);
        let g = 0x002d_df40;
        if m.r32(g + 0x300) == 0 && m.r32(g + 0x260) == a0 {
            let t = if m.r32(a0 + 0x434) != 0 { 0x002b_ccc8 } else { 0x002b_ccb8 };
            let (x, y, w) = (m.rf(a0 + 0x1050), m.rf(a0 + 0x104c), m.rf(0x002d_ded4));
            let f2 = m.call(0x0011_d018, &[t, 0], &[f4(x), f4(y), f4(w)]).1[0];
            let (lo, hi) = (z + m.rf(0x002b_cc98), z + m.rf(0x002b_cc9c));
            let r = if f2 < lo { lo } else if hi < f2 { hi } else { f2 };
            m.wf(a1 + 0x18, r);
        } else {
            m.wf(a1 + 0x18, f0);
        }
        let (x, y) = (m.r32(sp), m.r32(sp + 4));
        m.w32(a1 + 0x10, x);
        m.w32(a1 + 0x14, y);
    }
    let dt = m.rf(0x002d_ded4);
    let (qa, qb) = (m.rv(a0 + 0x1070), m.rv(0x002a_2dd0));
    let v = m.call(0x0011_f088, &[0x002b_ccd8, 0], &[qa, qb, f4(dt)]).1;
    m.wv(a0 + 0x1070, v);
    let o = m.rv(a1 + 0x10);
    let s = [o[0] + v[0], o[1] + v[1], o[2] + v[2], o[3] + v[3]];
    m.wv(a1 + 0x10, s);
    m.wv(a0 + 0x1060, s);
    let z = m.r32(a1 + 0x18);
    m.w32(a0 + 0x1050, z);
    if m.r32(a0 + 0x2dc) != u32::MAX {
        m.call(0x0019_c8b0, &[a0 + 0x2c0, a1], &[]);
    }
    if m.r32(a0 + 0x4cc) != u32::MAX {
        m.call(0x0018_a4e8, &[a0 + 0x4c0, a1], &[]);
    }
    if m.r32(a0 + 0xdf4) != u32::MAX {
        m.call(0x001a_2160, &[a0 + 0xdf0, a1], &[]);
    }
    let v = vslot(m, a0, 33, &[], &[]).1;
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    let t = 0x002b_ccf0;
    let mut f3 = m.rf(t) + len * (m.rf(t + 4) + len * m.rf(t + 8));
    let (lo, hi) = (m.rf(0x002b_bb60), m.rf(0x002b_bb64));
    if f3 < lo {
        f3 = lo;
    } else if hi < f3 {
        f3 = hi;
    }
    let k = m.rf(a1 + 0x34) * f3;
    m.wf(a1 + 0x34, k);
    stack_pop(m, 0x60);
}

// ---- libc / libm leaves the live game reaches (NATIVE before): byte-exact semantics over QMem ----

/// FUN_0023320c memcpy(dst, src, n) -> dst.
pub fn ee_sub_23320c<M: QMem + ?Sized>(m: &mut M, d: u32, s: u32, n: u32) -> u32 {
    for i in 0..n {
        let b = m.r8(s + i);
        m.w8(d + i, b);
    }
    d
}

/// FUN_002332b8 memmove(dst, src, n) -> dst (overlap safe).
pub fn ee_sub_2332b8<M: QMem + ?Sized>(m: &mut M, d: u32, s: u32, n: u32) -> u32 {
    if d <= s || d >= s.wrapping_add(n) {
        return ee_sub_23320c(m, d, s, n);
    }
    for i in (0..n).rev() {
        let b = m.r8(s + i);
        m.w8(d + i, b);
    }
    d
}

/// FUN_002333b4 memset(dst, c, n) -> dst.
pub fn ee_sub_2333b4<M: QMem + ?Sized>(m: &mut M, d: u32, c: u32, n: u32) -> u32 {
    for i in 0..n {
        m.w8(d + i, c as u8);
    }
    d
}

/// FUN_00233178 memcmp(a, b, n): difference of the first differing unsigned bytes.
pub fn ee_sub_233178<M: QMem + ?Sized>(m: &mut M, a: u32, b: u32, n: u32) -> i32 {
    for i in 0..n {
        let (x, y) = (m.r8(a + i), m.r8(b + i));
        if x != y {
            return x as i32 - y as i32;
        }
    }
    0
}

/// FUN_00233470 strncmp(a, b, n).
pub fn ee_sub_233470<M: QMem + ?Sized>(m: &mut M, a: u32, b: u32, n: u32) -> i32 {
    for i in 0..n {
        let (x, y) = (m.r8(a + i), m.r8(b + i));
        if x != y || x == 0 {
            return x as i32 - y as i32;
        }
    }
    0
}

/// FUN_00233638 strncpy(dst, src, n) -> dst (zero pads).
pub fn ee_sub_233638<M: QMem + ?Sized>(m: &mut M, d: u32, s: u32, n: u32) -> u32 {
    let mut end = false;
    for i in 0..n {
        let b = if end { 0 } else { m.r8(s + i) };
        end |= b == 0;
        m.w8(d + i, b);
    }
    d
}

/// FUN_002345b8 tolower(c).
pub fn ee_sub_2345b8(c: u32) -> u32 {
    if (b'A' as u32..=b'Z' as u32).contains(&c) { c + 0x20 } else { c }
}

/// FUN_00244e10 cosf.
pub fn ee_sub_244e10<M: QMem + ?Sized>(_m: &mut M, x: f32) -> f32 {
    x.cos()
}
/// FUN_00244ef8 fabsf.
pub fn ee_sub_244ef8<M: QMem + ?Sized>(_m: &mut M, x: f32) -> f32 {
    f32::from_bits(x.to_bits() & 0x7fff_ffff)
}
/// FUN_00244f18 floorf.
pub fn ee_sub_244f18<M: QMem + ?Sized>(_m: &mut M, x: f32) -> f32 {
    x.floor()
}
/// FUN_00245000 sinf (fdlibm s_sinf): |x| < pi/4 -> __kernel_sinf(x, 0, 0); inf/nan -> x - x; else
/// n = __rem_pio2f(x, y) (0x2488c8) and the quadrant kernel (0x249888 sin / 0x248de0 cos).
pub fn ee_sub_245000<M: QMem + ?Sized>(m: &mut M, x: f32) -> f32 {
    let ix = x.to_bits() & 0x7fff_ffff;
    let k = |z: f32| [z, 0.0, 0.0, 0.0];
    if ix < 0x3f49_0fd9 {
        return m.call(0x0024_9888, &[0], &[k(x), k(0.0)]).1[0];
    }
    if ix >= 0x7f80_0000 {
        return x - x;
    }
    let sp = stack_push(m, 0x20);
    let n = m.call(0x0024_88c8, &[sp], &[k(x)]).0 & 3;
    let (y0, y1) = (m.rf(sp), m.rf(sp + 4));
    let r = match n {
        0 => m.call(0x0024_9888, &[1], &[k(y0), k(y1)]).1[0],
        1 => m.call(0x0024_8de0, &[], &[k(y0), k(y1)]).1[0],
        2 => -m.call(0x0024_9888, &[1], &[k(y0), k(y1)]).1[0],
        _ => -m.call(0x0024_8de0, &[], &[k(y0), k(y1)]).1[0],
    };
    stack_pop(m, 0x20);
    r
}
/// FUN_002450f0 tanf (fdlibm s_tanf): |x| < pi/4 -> __kernel_tanf(x, 0, 1); |x| > max -> x - x; else
/// n = __rem_pio2f(x, y) and __kernel_tanf(y0, y1, 1 - 2*(n & 1)) (0x249990).
pub fn ee_sub_2450f0<M: QMem + ?Sized>(m: &mut M, x: f32) -> f32 {
    let ix = x.to_bits() & 0x7fff_ffff;
    let k = |z: f32| [z, 0.0, 0.0, 0.0];
    if ix < 0x3f49_0fdb {
        return m.call(0x0024_9990, &[1], &[k(x), k(0.0)]).1[0];
    }
    if ix > 0x7f7f_ffff {
        return x - x;
    }
    let sp = stack_push(m, 0x20);
    let n = m.call(0x0024_88c8, &[sp], &[k(x)]).0;
    let kk = ((n & 1) as i32 * -2 + 1) as u32;
    let (y0, y1) = (m.rf(sp), m.rf(sp + 4));
    let r = m.call(0x0024_9990, &[kk], &[k(y0), k(y1)]).1[0];
    stack_pop(m, 0x20);
    r
}
// ---- newlib w_*f wrappers (WIRE-GAP: 1:1, kernel + _LIB_VERSION domain check via m.call) ----
const LIB_VERSION: u32 = 0x002a_c138; // _LIB_VERSION (-1 _IEEE_, 0 _SVID_, 1 _XOPEN_ (ELF), 2 _POSIX_)

fn isnan<M: QMem + ?Sized>(m: &mut M, x: f32) -> bool {
    m.call(0x0024_a0c0, &[], &[f4(x)]).0 != 0
}
fn wd<M: QMem + ?Sized>(m: &mut M, a: u32, d: f64) {
    let b = d.to_bits();
    m.w32(a, b as u32);
    m.w32(a + 4, (b >> 32) as u32);
}
/// struct exception {type +0, name +4, arg1 +8, arg2 +0x10, retval +0x18, err +0x20} on the stack; DOMAIN (1);
/// lib == 2 || !matherr(e) -> *__errno() = EDOM (0x21); err != 0 -> *__errno() = err; return (float)retval.
pub(crate) fn libm_domain<M: QMem + ?Sized>(m: &mut M, name: u32, a1: f32, a2: f32, ret: f64) -> f32 {
    let lib = m.r32(LIB_VERSION);
    let e = stack_push(m, 0x30);
    m.w32(e, 1);
    m.w32(e + 4, name);
    m.w32(e + 0x20, 0);
    wd(m, e + 8, a1 as f64);
    wd(m, e + 0x10, a2 as f64);
    wd(m, e + 0x18, ret);
    if lib == 2 || m.call(0x0024_9cb8, &[e], &[]).0 == 0 {
        let p = m.call(0x0024_01c8, &[], &[]).0;
        if p != 0 {
            m.w32(p, 0x21);
        }
    }
    let err = m.r32(e + 0x20);
    if err != 0 {
        let p = m.call(0x0024_01c8, &[], &[]).0;
        if p != 0 {
            m.w32(p, err);
        }
    }
    let r = f64::from_bits(m.r64(e + 0x18)) as f32;
    stack_pop(m, 0x30);
    r
}
/// FUN_00245?? acosf / asinf body: kernel; lib != -1 && !isnan(x) && fabsf(x) > 1 -> domain error (retval 0).
fn w_arc<M: QMem + ?Sized>(m: &mut M, kernel: u32, name: u32, x: f32) -> f32 {
    let z = m.call(kernel, &[], &[f4(x)]).1[0];
    if m.r32(LIB_VERSION) == 0xffff_ffff || isnan(m, x) || !(1.0 < m.call(0x0024_4ef8, &[], &[f4(x)]).1[0]) {
        return z;
    }
    libm_domain(m, name, x, x, 0.0)
}
/// FUN_002456f0 acosf (newlib w_acosf: __ieee754_acosf = FUN_00247330).
pub fn ee_sub_2456f0<M: QMem + ?Sized>(m: &mut M, x: f32) -> f32 {
    w_arc(m, 0x0024_7330, 0x002a_b8f0, x)
}
/// FUN_002457f0 asinf (newlib w_asinf: __ieee754_asinf = FUN_00247760).
pub fn ee_sub_2457f0<M: QMem + ?Sized>(m: &mut M, x: f32) -> f32 {
    w_arc(m, 0x0024_7760, 0x002a_b8f8, x)
}
/// FUN_00245a18 fmodf (newlib w_fmodf: __ieee754_fmodf = FUN_00247de8); y == 0 (both non-NaN) -> domain
/// error, retval x (_SVID_) else the double at DAT_002ab910 (NaN).
pub fn ee_sub_245a18<M: QMem + ?Sized>(m: &mut M, x: f32, y: f32) -> f32 {
    let z = m.call(0x0024_7de8, &[], &[f4(x), f4(y)]).1[0];
    let lib = m.r32(LIB_VERSION);
    if lib == 0xffff_ffff || isnan(m, y) || isnan(m, x) || y != 0.0 {
        return z;
    }
    let ret = if lib == 0 { x as f64 } else { f64::from_bits(m.r64(0x002a_b910)) };
    libm_domain(m, 0x002a_b908, x, y, ret)
}

/// FUN_0012e238(kind): new act of kind 0..9 (FUN_0012e1f0(size)); +0 = kind, +0x18 = vtable; kind 1 +0x1c = 2,
/// kind 6 identity block at +0x20 (memset 0x40, 1.0 diagonal), kind 9 +0x14 = 0x14. Else 0.
pub fn ee_sub_12e238<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    const T: [(u32, u32); 10] = [
        (0x2c, 0x002c_af08),
        (0x30, 0x002c_a820),
        (0x80, 0x002c_a7e0),
        (0x20, 0x002d_b338),
        (0x28, 0x002c_a7a0),
        (0x30, 0x002c_a760),
        (0x60, 0x002c_a720),
        (0x20, 0x002d_2e58),
        (0x40, 0x002c_a6e0),
        (0x30, 0x002c_a6a0),
    ];
    if a0 >= 10 {
        return 0;
    }
    let (sz, vt) = T[a0 as usize];
    let r = m.call(0x0012_e1f0, &[sz], &[]).0;
    m.w32(r, a0);
    m.w32(r + 0x18, vt);
    match a0 {
        1 => m.w32(r + 0x1c, 2),
        6 => {
            m.call(0x0023_33b4, &[r + 0x20, 0, 0x40], &[]);
            for o in [0x14, 0x3c, 0x28, 0] {
                m.wf(r + 0x20 + o, 1.0);
            }
        }
        9 => m.w32(r + 0x14, 0x14),
        _ => {}
    }
    r
}

/// FUN_00246060 sqrtf (newlib w_sqrtf): z = __ieee754_sqrtf(x) (0x248ca8); lib != -1 && !isnan(x) && x < 0 ->
/// struct exception {DOMAIN, "sqrtf" 0x2ab938, arg1 = arg2 = (double)x via 0x23e290, retval = lib==0 ? 0 :
/// DAT_002ab940}; lib == 2 || !matherr(e) -> *__errno() = EDOM; err -> *__errno() = err; z = (float)retval
/// via 0x23dfa0.
pub fn ee_sub_246060<M: QMem + ?Sized>(m: &mut M, x: f32) -> f32 {
    let z = m.call(0x0024_8ca8, &[], &[f4(x)]).1[0];
    let lib = m.r32(LIB_VERSION);
    if lib == 0xffff_ffff || m.call(0x0024_a0c0, &[], &[f4(x)]).0 != 0 || !(x < 0.0) {
        return z;
    }
    let e = stack_push(m, 0x30);
    m.w32(e, 1);
    m.w32(e + 4, 0x002a_b938);
    m.w32(e + 0x20, 0);
    // 0x23e290 = __extendsfdf2 (64-bit v0); the QMem call ABI is 32-bit, so the value is formed here.
    m.call(0x0023_e290, &[], &[f4(x)]);
    wd(m, e + 8, x as f64);
    wd(m, e + 0x10, x as f64);
    let rv = if lib == 0 { 0.0 } else { f64::from_bits(m.r64(0x002a_b940)) };
    wd(m, e + 0x18, rv);
    if lib == 2 || m.call(0x0024_9cb8, &[e], &[]).0 == 0 {
        let p = m.call(0x0024_01c8, &[], &[]).0;
        m.w32(p, 0x21);
    }
    let err = m.r32(e + 0x20);
    if err != 0 {
        let p = m.call(0x0024_01c8, &[], &[]).0;
        m.w32(p, err);
    }
    let rv = m.r64(e + 0x18);
    let r = m.call(0x0023_dfa0, &[rv as u32, (rv >> 32) as u32], &[]).1[0];
    stack_pop(m, 0x30);
    r
}

/// FUN_0022c840(owner, act): kinds (lh +0x98) other than 0-3/13-16/20/21 reset the act clock (+0x80/+0x84 =
/// DAT_002a98d0, +0x88/+0x8c = 0x2c44e0); +0x9a = 0; vslot46(owner, 4).
pub fn ee_sub_22c840<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let k = m.r16(a1 + 0x98) as i16 as i32;
    if !matches!(k, 0..=3 | 13..=16 | 20 | 21) {
        let f = m.r32(0x002a_98d0);
        m.w32(a1 + 0x88, 0x002c_44e0);
        m.w32(a1 + 0x80, f);
        m.w32(a1 + 0x84, f);
        m.w32(a1 + 0x8c, 0x002c_44e0);
    }
    m.w16(a1 + 0x9a, 0);
    crate::engine::classes::q::q2mem::vslot(m, a0, 46, &[4], &[]);
}

/// FUN_00194a40(w, jt, mask): target-candidate scan. Four 0x30 slots on the stack (+8 = DAT_002bf664, +0xc = -1);
/// FUN_00194688(w, jt, pos, o, o+sub, 4, slots) over the class lists picked by mask bits (8/0x10/0x80/4/0x400/
/// 0x200/2, bit 1 = list 0x3e13b4 filtered by +0xe8&1 and FUN_00132b10(o, 0x13), 0x20 = 0x3e1408); then
/// FUN_00195368 / FUN_00194f70(w, jt, n, slots) and the binoc music cue (0x3da0c0: FUN_001da578 / FUN_001da4b8).
pub fn ee_sub_194a40<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    if m.call(0x001c_40c0, &[a1], &[]).0 == 0 {
        return;
    }
    let sp = stack_push(m, 0x180);
    m.call(0x0023_33b4, &[sp, 0, 0xc0], &[]);
    let f = m.r32(0x002b_f664);
    for i in 0..4 {
        m.w32(sp + 0x30 * i + 0xc, u32::MAX);
        m.w32(sp + 0x30 * i + 8, f);
    }
    let pos = m.call(0x001a_62e8, &[a1], &[]).1;
    m.wv(sp + 0xc0, pos);
    let scan = |m: &mut M, head: u32, sub: u32, link: u32| {
        let mut s = m.r32(head);
        while s != 0 {
            m.call(0x0019_4688, &[a0, a1, s, s + sub, 4, sp], &[pos]);
            s = m.r32(s + link);
        }
    };
    for (bit, head, sub, link) in [
        (8, 0x003e_13f0, 0x190, 0x1b4),
        (0x10, 0x003e_13fc, 0xe0, 0x104),
        (0x80, 0x003e_1414, 0xe0, 0x104),
        (4, 0x003e_1438, 0x130, 0x154),
        (0x400, 0x003e_13d8, 0xe0, 0x104),
        (0x200, 0x003e_13cc, 0x190, 0x1b4),
        (2, 0x003e_13c0, 0x190, 0x1b4),
    ] {
        if a2 & bit != 0 {
            scan(m, head, sub, link);
        }
    }
    if a2 & 1 != 0 {
        let mut s = m.r32(0x003e_13b4);
        while s != 0 {
            if m.r32(s + 0xe8) & 1 != 0 && m.call(0x0013_2b10, &[s, 0x13], &[]).0 != 0 {
                m.call(0x0019_4688, &[a0, a1, s, s + 0x100, 4, sp], &[pos]);
            }
            s = m.r32(s + 0xe0);
        }
    }
    if a2 & 0x20 != 0 {
        scan(m, 0x003e_1408, 0xe0, 0x104);
    }
    let mut n = 4u32;
    while n > 0 && m.r32(sp + 0x30 * (n - 1)) == 0 {
        n -= 1;
    }
    let g = 0x003d_d490;
    m.call(0x0019_5368, &[g, a1, n, sp], &[]);
    m.call(0x0019_4f70, &[g, a1, n, sp], &[]);
    let mus = 0x003d_a0c0;
    let s1 = m.call(0x001d_a5b0, &[mus], &[]).0;
    if s1 == u32::MAX || s1 == m.r32(g + 0x4740) {
        let mut s0 = u32::MAX;
        if m.r32(g + 0x4678) != 0 {
            let c = m.rv(g + 0x46a0);
            let (dx, dy, dz) = (pos[0] - c[0], pos[1] - c[1], pos[2] - c[2]);
            let d = (dx * dx + dy * dy + dz * dz).sqrt();
            if !(m.rf(0x002b_f660) < d) {
                let a = m.r32(g + 0x4690);
                let v = if a != 0 { m.call(0x001c_b658, &[a], &[]).0 } else { 0 };
                s0 = if v != 0 && m.r32(v + 0x18) == 0x624 && m.rf(0x002b_f70c) < m.rf(g + 0x4694) {
                    0x125
                } else {
                    let t = m.r32(g + 0x4684);
                    m.r32(t + 0x14)
                };
            }
        }
        if s0 != s1 {
            if s0 == u32::MAX {
                let cur = m.r32(g + 0x4740);
                m.call(0x001d_a578, &[mus, cur], &[]);
            } else {
                m.call(0x001d_a4b8, &[mus, s0], &[f4(-1.0)]);
            }
            m.w32(g + 0x4740, s0);
        }
    }
    stack_pop(m, 0x180);
}

fn hermite_117120<M: QMem + ?Sized>(m: &mut M, b: u32, out: u32) -> V4 {
    let f0 = m.rf(b + 0x80);
    let (p88, p8c) = (m.r32(b + 0x88), m.r32(b + 0x8c));
    let f13 = m.rf(p88) - f0;
    let f12 = m.rf(b + 0x84) - f0;
    let f14 = m.rf(p8c);
    let q: Vec<V4> = (0..4).map(|k| m.rv(b + 0xc0 + 0x10 * k)).collect();
    m.call(0x0011_7120, &[0, out, 0], &[q[0], q[1], q[2], q[3], f4(f12), f4(f13), f4(f14)]);
    m.rv(out)
}

/// FUN_0022cdc0(out, bone): bone local velocity matrix (0x40 bytes to out; returns out). Dispatch on kind
/// (lh +0x98), channel mode +0x94 and key mode (lh +0x9a) through the asm's jump tables (0x2a9a00 / 0x2a9a50 /
/// 0x2a9a90 / 0x2a9af0 / 0x2a9b10 / 0x2a9b30 / 0x2a9b90): zero (memset), parent's negated (+0xa8, recursive),
/// FUN_0022ccd0 / FUN_0022cb90 / FUN_0022cc00 compose with hermite (FUN_0011f... 0x117120) / euler (FUN_0011bff0) keys.
pub fn ee_sub_22cdc0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let sp = stack_push(m, 0xc0);
    let zero = m.rv(0x002a_2dd0);
    let kind = m.r16(a1 + 0x98) as i16 as i32;
    let km1 = (m.r16(a1 + 0x98).wrapping_sub(1)) as i16 as i32;
    let key = m.r16(a1 + 0x9a) as i16 as i32;
    let group_a = matches!(kind, 0 | 3 | 10 | 12..=16 | 19);
    let t3 = |k: i32| -> u8 {
        match k {
            0 | 1 | 6 | 7 | 8 | 10 | 16 | 17 | 19 | 20 => 1,
            4 | 5 => 2,
            _ => 0,
        }
    };
    if group_a {
        m.call(0x0023_33b4, &[a0, 0, 0x40], &[]);
    } else {
        match m.r32(a1 + 0x94) {
            8 | 14 => {
                let p = m.r32(a1 + 0xa8);
                m.call(0x0022_cdc0, &[sp, p], &[]);
                m.call(0x0023_320c, &[a0, sp, 0x40], &[]);
                for o in [0x30, 0x34, 0x38] {
                    let v = -m.rf(a0 + o);
                    m.wf(a0 + o, v);
                }
            }
            6 | 7 => match t3(km1) {
                1 => {
                    let q = m.rv(a1 + 0xb0);
                    m.call(0x0022_ccd0, &[sp, a1, a1], &[q]);
                    m.call(0x0023_320c, &[a0, sp, 0x40], &[]);
                }
                2 => {
                    if key == 5 {
                        m.call(0x0023_33b4, &[a0, 0, 0x40], &[]);
                    } else {
                        let s0 = match key {
                            0 => m.call(0x0022_ca18, &[a1, a1], &[]).1,
                            1 => hermite_117120(m, a1, sp),
                            2..=4 => m.rv(a1 + 0xc0),
                            _ => zero, // key 5 handled above; out of range
                        };
                        let s0 = if key == 6 - 1 { m.call(0x0011_bff0, &[a1 + 0xc0], &[]).1 } else { s0 };
                        let t = match key {
                            1 => hermite_117120(m, a1, sp),
                            3 | 4 => m.rv(a1 + 0xd0),
                            _ => zero,
                        };
                        m.call(0x0022_cb90, &[sp, a1, a1], &[s0, t]);
                        m.call(0x0023_320c, &[a0, sp, 0x40], &[]);
                    }
                }
                _ => {}
            },
            1 | 4 | 5 | 9..=12 => {
                match t3(km1) {
                    1 => {
                        let q = m.rv(a1 + 0xb0);
                        m.wv(sp, q);
                    }
                    2 => {
                        let v = match key {
                            1 => hermite_117120(m, a1, sp + 0x10),
                            3 | 4 => m.rv(a1 + 0xd0),
                            _ => zero,
                        };
                        m.wv(sp, v);
                    }
                    _ => {}
                }
                let q = m.rv(sp);
                m.call(0x0022_cc00, &[sp + 0x40, a1], &[q]);
                m.call(0x0023_320c, &[a0, sp + 0x40, 0x40], &[]);
            }
            _ => {}
        }
    }
    stack_pop(m, 0xc0);
    a0
}

// ---- WIRE-GAP b2: libdma channel helpers (1:1; the port has no DMA engine, so a started transfer is
// complete by the next wait: FUN_00241368 clears CHCR.STR instead of spinning on hardware) ----
/// FUN_00241368(ch): DmaSync - wait while CHCR (+0) bit 8 (STR) is set.
pub fn ee_sub_241368<M: QMem + ?Sized>(m: &mut M, ch: u32) {
    let c = m.r32(ch);
    if c & 0x100 != 0 {
        m.w32(ch, c & !0x100);
    }
}
/// FUN_002413e0(addr): scratchpad (top nibble 7) -> SPR bit 31 + 28-bit address.
pub fn ee_sub_2413e0<M: QMem + ?Sized>(_m: &mut M, a: u32) -> u32 {
    if a >> 28 == 7 {
        (a & 0x0fff_ffff) | 0x8000_0000
    } else {
        a
    }
}
/// FUN_002417f0(ch, mode): mode 1 -> CHCR.STR poll; else DmaSync and 0.
pub fn ee_sub_2417f0<M: QMem + ?Sized>(m: &mut M, ch: u32, mode: u32) -> u32 {
    if mode == 1 {
        return m.r32(ch) >> 8 & 1;
    }
    m.call(0x0024_1368, &[ch], &[]);
    0
}
/// FUN_00241728(ch, tag): chain-mode start: TADR (+0x30, unless -1) = tag, QWC (+0x20) = 0, CHCR = STR|chain.
pub fn ee_sub_241728<M: QMem + ?Sized>(m: &mut M, ch: u32, tag: u32) {
    let t = m.call(0x0024_13e0, &[tag], &[]).0;
    m.call(0x0024_1368, &[ch], &[]);
    if m.r32(ch + 0x30) != 0xffff_ffff {
        m.w32(ch + 0x30, t);
    }
    m.w32(ch + 0x20, 0);
    let c = m.r32(ch);
    m.w32(ch, c & 0xffff_fff3 | 0x105);
}
/// FUN_002417a0(ch): restart in the current mode: QWC = 0, CHCR = STR | (dir) with mode bits cleared.
pub fn ee_sub_2417a0<M: QMem + ?Sized>(m: &mut M, ch: u32) {
    m.call(0x0024_1368, &[ch], &[]);
    m.w32(ch + 0x20, 0);
    let c = m.r32(ch);
    m.w32(ch, c & 0xffff_fff2 | 0x104);
}

/// FUN_0014f898(cm, out, d, s) FInsideCmMrd (1:1, as camera::cm::FUN_0014f898): |d|^2 within (s * mrd)^2;
/// *out = fade over the last 400 * mrd units (1 inside).
pub fn ee_sub_14f898<M: QMem + ?Sized>(m: &mut M, cm: u32, out: u32, d: V4, s: V4) -> u32 {
    let dd = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
    let mrd = m.rf(cm + 0x1c);
    let s = s[0] * mrd;
    let s2 = s * s;
    if s2 < dd {
        return 0;
    }
    let i = s - mrd * 400.0;
    let i2 = i * i;
    m.wf(out, if i2 < dd { (s2 - dd) / (s2 - i2) } else { 1.0 });
    1
}

// ---- WIRE-GAP b3: CRef setters {tag +0, payload +4} (1:1: Release old value FUN_00129ec8, then store) ----
fn ref_set<M: QMem + ?Sized>(m: &mut M, r: u32, tag: u32, p: u32) -> u32 {
    m.call(0x0012_9ec8, &[r], &[]);
    m.w32(r + 4, p);
    m.w32(r, tag);
    r
}
/// FUN_00129518(r, v): = S32 (tag 0x15)
pub fn ee_sub_129518<M: QMem + ?Sized>(m: &mut M, r: u32, v: u32) -> u32 {
    ref_set(m, r, 0x15, v)
}
/// FUN_00129558(r, f): = F32 (tag 0x16); f arrives in $f12
pub fn ee_sub_129558<M: QMem + ?Sized>(m: &mut M, r: u32, f: f32) -> u32 {
    ref_set(m, r, 0x16, f.to_bits())
}
/// FUN_001295d8(r, id): = Symid (tag 0x18)
pub fn ee_sub_1295d8<M: QMem + ?Sized>(m: &mut M, r: u32, v: u32) -> u32 {
    ref_set(m, r, 0x18, v)
}
/// FUN_00129970(r, pair): AddRef pair, = Pair (tag 0x1f)
pub fn ee_sub_129970<M: QMem + ?Sized>(m: &mut M, r: u32, p: u32) -> u32 {
    let n = m.r32(p);
    m.w32(p, n.wrapping_add(1));
    ref_set(m, r, 0x1f, p)
}
/// FUN_00129a70(r, method): AddRef, = Method (tag 0x22)
pub fn ee_sub_129a70<M: QMem + ?Sized>(m: &mut M, r: u32, p: u32) -> u32 {
    let n = m.r32(p);
    m.w32(p, n.wrapping_add(1));
    ref_set(m, r, 0x22, p)
}
/// FUN_00129480(r, pair): init (no release): = Pair (tag 0x1f), AddRef
pub fn ee_sub_129480<M: QMem + ?Sized>(m: &mut M, r: u32, p: u32) {
    m.w32(r + 4, p);
    m.w32(r, 0x1f);
    let n = m.r32(p);
    m.w32(p, n.wrapping_add(1));
}
/// FUN_001294c0(r, obj): init basic object ref (tag 0x21); obj != 0 -> its vtable slot (+0xc)+0xc AddRef
pub fn ee_sub_1294c0<M: QMem + ?Sized>(m: &mut M, r: u32, o: u32) -> u32 {
    if o != 0 {
        let vt = m.r32(o + 0xc);
        let adj = m.r16(vt + 8) as i16 as i32 as u32;
        let f = m.r32(vt + 0xc);
        m.call(f, &[o.wrapping_add(adj)], &[]);
    }
    m.w32(r + 4, o);
    m.w32(r, 0x21);
    r
}

/// FUN_001875b8(pad, mode, kind): pad mode switch (+0x100c mode, +0x1010 kind): actuator re-init for kind > 2
/// (FUN_001d5150), clear +0x10c0 on entering / leaving kind 4, reset sticks / buttons / deadzones when
/// entering or leaving mode 3; stamp +0x1014 = now.
pub fn ee_sub_1875b8<M: QMem + ?Sized>(m: &mut M, p: u32, mut mode: u32, kind: u32) {
    if mode == m.r32(p + 0x100c) && kind == m.r32(p + 0x1010) {
        return;
    }
    if kind == 0 {
        m.w32(p + 0x1008, 0);
        mode = 1;
    }
    if (m.r32(p + 0x1010) as i32) < 3 && 2 < kind as i32 {
        let (a, b) = (m.r32(p + 0x1000), m.r32(p + 0x1004));
        m.call(0x001d_5150, &[p + 0x10cc, a, b], &[]);
    }
    if (m.r32(p + 0x1010) == 4) != (kind == 4) {
        m.call(0x0023_33b4, &[p + 0x10c0, 0, 0xc], &[]);
    }
    if (m.r32(p + 0x100c) == 3) != (mode == 3) {
        for o in [0x1018, 0x1080, 0x104c, 0x10b8, 0x10b4] {
            m.w32(p + o, 0);
        }
        m.w32(p + 0x10bc, 0xffff);
        m.call(0x0023_33b4, &[p + 0x10c0, 0, 0xc], &[]);
        for o in [0x1028, 0x10ac, 0x1044, 0x1040, 0x103c, 0x1078, 0x1074, 0x1070, 0x10a8, 0x10a4, 0x10b0, 0x1024] {
            m.w32(p + o, 0);
        }
        for i in 0..4 {
            m.w32(p + 0x1084 + 8 * i, 0xbf40_0000);
            m.w32(p + 0x1050 + 8 * i, 0xbf40_0000);
            m.w32(p + 0x1088 + 8 * i, 0x3f40_0000);
            m.w32(p + 0x1054 + 8 * i, 0x3f40_0000);
        }
    }
    m.w32(p + 0x100c, mode);
    m.w32(p + 0x1010, kind);
    let t = m.r32(0x002d_df0c);
    m.w32(p + 0x1014, t);
}

/// FUN_001d51b8(act): pad actuator (vibration) tick: state 2 plays the pattern (+0xc table {n; {b4, b5, dur}})
/// while +0x14 > 0 (FUN_00243870 scePadSetActDirect per step), states 3 / 4 count +0x18 down (FUN_001d5468) then
/// FUN_001d53c0.
pub fn ee_sub_1d51b8<M: QMem + ?Sized>(m: &mut M, p: u32) {
    let dt = m.rf(0x002d_df10);
    let st = m.r32(p + 8) as i32;
    if st == 2 {
        if m.rf(p + 0x14) <= 0.0 {
            m.call(0x001d_53c0, &[p, 3], &[]);
            return;
        }
        let f = m.rf(p + 0x18) - dt;
        m.wf(p + 0x18, f);
        if f <= 0.0 {
            let tb = m.r32(p + 0xc);
            let mut i = m.r32(p + 0x10).wrapping_add(1);
            m.w32(p + 0x10, i);
            if m.r32(tb) as i32 <= i as i32 {
                i = 0;
                m.w32(p + 0x10, 0);
            }
            let e = tb + i * 8;
            let sp = stack_push(m, 0x10);
            let (b4, b5) = (m.r8(e + 4), m.r8(e + 5));
            m.w8(sp, b4);
            m.w8(sp + 1, b5);
            let (a0, a1) = (m.r32(p), m.r32(p + 4));
            m.call(0x0024_3870, &[a0, a1, sp], &[]);
            stack_pop(m, 0x10);
            let (tb, i) = (m.r32(p + 0xc), m.r32(p + 0x10));
            let d = m.r32(tb + i * 8 + 8);
            m.w32(p + 0x18, d);
        }
        let r = m.rf(p + 0x14) - dt;
        m.wf(p + 0x14, r);
    } else if 1 < st && st < 5 {
        if 0.0 < m.rf(p + 0x18) {
            m.call(0x001d_5468, &[p], &[]);
            let r = m.rf(p + 0x18) - dt;
            m.wf(p + 0x18, r);
        } else {
            let k = (m.r32(p + 8) == 3) as u32;
            m.call(0x001d_53c0, &[p, k], &[]);
        }
    }
}

/// FUN_00215850(p, o, mat, a, b): p+4 == 4 -> FUN_002183e8(*(p+8)+0x84 + p+0x10 * 0x30, o+0x58, mat, &*a, &*b, f12)
/// (the two words copied to the stack, f12 passed through).
pub fn ee_sub_215850<M: QMem + ?Sized>(m: &mut M, p: u32, o: u32, mat: u32, a: u32, b: u32, f12: f32) {
    if m.r32(p + 4) != 4 {
        return;
    }
    let sp = stack_push(m, 0x30);
    let (wa, wb) = (m.r32(a), m.r32(b));
    m.w32(sp, wa);
    m.w32(sp + 0x10, wb);
    let q = m.r32(p + 8);
    let base = m.r32(q + 0x84);
    let x = base.wrapping_add(m.r32(p + 0x10).wrapping_mul(0x30));
    let y = m.r32(o + 0x58);
    m.call(0x0021_83e8, &[x, y, mat, sp, sp + 0x10], &[f4(f12)]);
    stack_pop(m, 0x30);
}

/// FUN_00187998(pad): read the pad (FUN_002432d0 into a 0x40 buffer); no data -> 1 while < 2 s since the last
/// read (+0x1018). Else buttons (inverted bytes 2:3) -> +0x10b4 held / +0x10b8 pressed / +0x10bc latch, mode
/// byte 1 -> +0x1008 (change -> 0), kind 4 copies 0xc bytes (FUN_001ae5a0), sticks (FUN_00187768, or cleared for
/// kind < 2), d-pad override of the left stick (unless DAT_002de24c with DAT_002de254 == 0x2de2e0), stick
/// velocities / dt, FUN_0014bf48, idle timer +0x101c / +0x1020 and the 15 s activity event 0xe.
pub fn ee_sub_187998<M: QMem + ?Sized>(m: &mut M, p: u32) -> u32 {
    const NOW: u32 = 0x002d_df0c;
    let mut ret = 1;
    let buf = stack_push(m, 0x40);
    m.call(0x0023_33b4, &[buf, 0, 0x40], &[]);
    let (a0, a1) = (m.r32(p + 0x1000), m.r32(p + 0x1004));
    let n = m.call(0x0024_32d0, &[a0, a1, buf], &[]).0 as i32;
    if n < 1 {
        let r = (m.rf(NOW) - m.rf(p + 0x1018) < 2.0) as u32;
        stack_pop(m, 0x40);
        return r;
    }
    let old = m.r32(p + 0x10b4);
    let btn = ((m.r8(buf + 2) as u32) << 8 | m.r8(buf + 3) as u32) ^ 0xffff;
    let latch = m.r32(p + 0x10bc);
    m.w32(p + 0x10b8, btn & !old & !latch);
    m.w32(p + 0x10bc, latch & btn);
    m.w32(p + 0x10b4, btn);
    let mode = m.r8(buf + 1) as u32;
    let cur = m.r32(p + 0x1008);
    if cur != 0 && cur != mode {
        ret = 0;
    }
    m.w32(p + 0x1008, mode);
    if m.r32(p + 0x1010) == 4 {
        m.call(0x001a_e5a0, &[p + 0x10c0, buf + 8, 0xc], &[]);
    }
    let (f9, f8, f11, f10) = (m.rf(p + 0x103c), m.rf(p + 0x1040), m.rf(p + 0x1070), m.rf(p + 0x1074));
    if (m.r32(p + 0x1010) as i32) < 2 {
        for o in [0x1070, 0x1044, 0x1040, 0x103c, 0x1078, 0x1074] {
            m.w32(p + o, 0);
        }
    } else {
        let (b6, b7, b4, b5) = (m.r8(buf + 6) as u32, m.r8(buf + 7) as u32, m.r8(buf + 4) as u32, m.r8(buf + 5) as u32);
        m.call(0x0018_7768, &[p, b6, b7, p + 0x103c, p + 0x1040, p + 0x1044, p + 0x1048, p + 0x1049], &[]);
        m.call(0x0018_7768, &[p, b4, b5, p + 0x1070, p + 0x1074, p + 0x1078, p + 0x107c, p + 0x107d], &[]);
    }
    let skip = m.r32(0x002d_e24c) != 0 && m.r32(0x002d_e254) == 0x002d_e2e0;
    if !skip {
        let u = m.r32(p + 0x10b4);
        if u & 0xf000 != 0 {
            let s = m.rf(p + 0x10ec);
            m.w32(p + 0x103c, 0);
            m.w32(p + 0x1044, 0);
            m.w32(p + 0x1040, 0);
            if u & 0x8000 != 0 {
                m.wf(p + 0x103c, -s);
            } else if u & 0x2000 != 0 {
                m.wf(p + 0x103c, s);
            }
            if u & 0x1000 != 0 {
                m.wf(p + 0x1040, s);
            } else if u & 0x4000 != 0 {
                m.wf(p + 0x1040, -s);
            }
            let x = m.rf(p + 0x103c);
            if x * m.rf(p + 0x1040) != 0.0 {
                m.wf(p + 0x103c, x * 0.707_106_77);
                let y = m.rf(p + 0x1040);
                m.wf(p + 0x1040, y * 0.707_106_77);
            }
            m.w32(p + 0x1044, 0x3f80_0000);
        }
    }
    let dt = m.rf(0x002d_df10);
    let v = (m.rf(p + 0x103c) - f9) / dt;
    m.wf(p + 0x10a4, v);
    let v = (m.rf(p + 0x1040) - f8) / dt;
    m.wf(p + 0x10a8, v);
    let v = (m.rf(p + 0x1070) - f11) / dt;
    m.wf(p + 0x10ac, v);
    let v = (m.rf(p + 0x1074) - f10) / dt;
    m.wf(p + 0x10b0, v);
    m.call(0x0014_bf48, &[0x002d_dec0, p, old & 0xffff], &[]);
    let now = m.rf(NOW);
    m.wf(p + 0x1018, now);
    if m.r32(p + 0x10b4) == 0 && m.rf(p + 0x1044) == 0.0 && m.rf(p + 0x1078) == 0.0 {
        let d = (now - m.rf(p + 0x101c)) as i32;
        m.w32(p + 0x1020, d as u32);
    } else {
        m.w32(p + 0x1020, 0);
        m.wf(p + 0x101c, now);
        if 15.0 < now - m.rf(0x002d_eb30) {
            let u = m.call(0x001c_4840, &[0x003d_d490], &[]).0;
            m.call(0x0016_19a0, &[0x002d_eb30, 0xe, u], &[]);
            m.wf(0x002d_eb30, now);
        }
    }
    stack_pop(m, 0x40);
    ret
}

/// FUN_0014e168(cm, mat): cm +0x60..+0xa0 = the 4x4 at mat, then FUN_0014dc58(cm, mat) (a0/a1 kept).
pub fn ee_sub_14e168<M: QMem + ?Sized>(m: &mut M, cm: u32, mat: u32) {
    for k in 0..4 {
        let q = m.rv(mat + 0x10 * k);
        m.wv(cm + 0x60 + 0x10 * k, q);
    }
    m.call(0x0014_dc58, &[cm, mat, cm], &[]);
}

/// FUN_00127b18(frame, x): +8 = 1, FUN_00256ee0(frame + 0xc, x).
pub fn ee_sub_127b18<M: QMem + ?Sized>(m: &mut M, f: u32, x: u32) {
    m.w32(f + 8, 1);
    m.call(0x0025_6ee0, &[f + 0xc, x], &[]);
}

/// FUN_00127cf0(out, frame, sym, val): FUN_00127db8(frame, sym, 1) -> slot; slot != 0 -> FUN_00129658(slot,
/// val); out = a fresh tag-0x24 ref (FUN_0012a0b0 / FUN_001291a8); returns out.
pub fn ee_sub_127cf0<M: QMem + ?Sized>(m: &mut M, out: u32, f: u32, sym: u32, val: u32) -> u32 {
    let sp = stack_push(m, 0x10);
    m.w32(sp, 0xffff_ffff);
    let s = m.call(0x0012_7db8, &[f, sym, 1], &[]).0;
    if s != 0 {
        m.call(0x0012_9658, &[s, val], &[]);
    }
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    m.call(0x0012_91a8, &[out, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x10);
    out
}

/// FUN_0014e6d0(cm, pos, vel, target, k, d) (VU0 macro, 1:1 incl. the vf00 w = 1 adds): cm +0x300 != 0 ->
/// *pos = target, vel.xyz = 0. Else a = (target - pos) k + vel d; vel += vf00 + a dt; pos += vf00 + vel dt
/// (dt = DAT_002dded4).
pub fn ee_sub_14e6d0<M: QMem + ?Sized>(m: &mut M, cm: u32, pos: u32, vel: u32, t: V4, k: f32, d: f32) {
    if m.r32(cm + 0x300) != 0 {
        m.wv(pos, t);
        m.w32(vel, 0);
        m.w32(vel + 8, 0);
        m.w32(vel + 4, 0);
        return;
    }
    let dt = m.rf(0x002d_ded4);
    let p = m.rv(pos);
    let v = m.rv(vel);
    let one = [0.0, 0.0, 0.0, 1.0];
    let mut nv = [0.0f32; 4];
    for i in 0..4 {
        let a = (t[i] - p[i]) * k + v[i] * d;
        nv[i] = (v[i] + one[i]) + a * dt;
    }
    m.wv(vel, nv);
    let mut np = [0.0f32; 4];
    for i in 0..4 {
        np[i] = (p[i] + one[i]) + nv[i] * dt;
    }
    m.wv(pos, np);
}

/// vtable call by byte offset `off` (this-adjust halfword at off - 4).
fn vc_off<M: QMem + ?Sized>(m: &mut M, obj: u32, off: u32, a: &[u32]) {
    let vt = m.r32(obj + 0xc);
    let f = m.r32(vt + off);
    let adj = m.r16(vt + off - 4) as i16 as i32 as u32;
    let mut args = vec![obj.wrapping_add(adj)];
    args.extend_from_slice(a);
    m.call(f, &args, &[]);
}

/// FUN_0014d478(cm): camera update. With cm +0x30c > 0 walks the +0x314 policy chain (target +0x318: slot
/// 0x194 fills cm +0x270, re-aim FUN_0014e920 when the target / chain changed, FUN_0014e768, +0x260 = target;
/// policy slot 0x34(cm +0x270, active pad 0x2dfc00 / 0x2e0d00 by DAT_002dfbc0 bit 0)) until the chain is stable.
/// Then without a +0x300 override the fov eases +0x20 -> +0x24 (FUN_0011d1c0, FUN_0014db70), else
/// FUN_0014f5a0(cm, +0x24); +0x300 = 0; with DAT_003e12c8 the +0x620 shake decays (FUN_0011d018).
pub fn ee_sub_14d478<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let mut i3: u32;
    'chain: {
        if (m.r32(o + 0x30c) as i32) < 1 {
            i3 = m.r32(o + 0x300);
            break 'chain;
        }
        i3 = 0;
        if m.r32(o + 0x314) != 0 {
            let mut i1 = m.r32(o + 0x318);
            let mut i7 = 0u32;
            let mut i2 = m.r32(o + 0x314);
            loop {
                let i6;
                if i1 == 0 {
                    m.w32(o + 0x260, 0);
                    i6 = 0;
                    i3 = 0;
                } else {
                    i6 = o + 0x270;
                    if i1 != i3 {
                        vc_off(m, i1, 0x194, &[i6]);
                        if i7 != 0 || i1 != m.r32(o + 0x260) {
                            m.call(0x0014_e920, &[o, i6], &[]);
                        }
                        m.call(0x0014_e768, &[o, i1, i6], &[]);
                        m.w32(o + 0x260, i1);
                        i3 = i1;
                    }
                }
                let pad = if m.r32(0x002d_fbc0) & 1 == 0 { 0x002e_0d00 } else { 0x002d_fc00 };
                vc_off(m, i2, 0x34, &[i6, pad]);
                if (m.r32(o + 0x30c) as i32) < 1 {
                    i3 = m.r32(o + 0x300);
                    break 'chain;
                }
                if m.r32(o + 0x314) == i2 {
                    break;
                }
                i1 = m.r32(o + 0x318);
                i7 = i2;
                i2 = m.r32(o + 0x314);
            }
        }
        i3 = m.r32(o + 0x300);
    }
    if i3 == 0 {
        let (cur, goal) = (m.rf(o + 0x20), m.rf(o + 0x24));
        if goal != cur {
            let (f13, dt) = (m.rf(o + 0x28), m.rf(0x002d_ded4));
            let r = m.call(0x0011_d1c0, &[o + 0x2c, o + 0x28], &[f4(cur), f4(f13), f4(goal), f4(dt)]).1[0];
            m.wf(o + 0x20, r);
            m.call(0x0014_db70, &[o], &[]);
        }
    } else {
        let g = m.rf(o + 0x24);
        m.call(0x0014_f5a0, &[o], &[f4(g)]);
    }
    m.w32(o + 0x300, 0);
    if m.r32(0x003e_12c8) != 0 {
        let (a, b, c) = (m.rf(o + 0x620), m.rf(o + 0x624), m.rf(0x002d_dee0));
        let r = m.call(0x0011_d018, &[0x002b_d810, 0], &[f4(a), f4(b), f4(c)]).1[0];
        m.wf(o + 0x620, r);
    }
}
