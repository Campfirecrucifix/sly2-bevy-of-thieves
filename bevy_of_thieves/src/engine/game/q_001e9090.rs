//! WP-Q4: run 0x001e9090..0x001eb050 - rope / path helpers of class C2d9040 (+0x40 start object, +0x44 end
//! object, +0x48 rope object {+0xe4 closed, +0xe8 point count, +0xf0 arc lengths, +0xf4 points}, +0x54 points,
//! +0x58 velocities, +0x5c arc lengths, +0x60 damping, +0x84 max points, +0x88 segment length), the patrol-rail
//! visibility ranges (0x9c-byte record: +0 path, +4 / +8 ranges, +0xc count, +0x10 16 pairs, +0x90 / +0x94
//! volume ids, +0x98 stamp), the level-load pointer fix-ups and the 256-entry glow CLUT upload.
//! 1:1 against `QMem` (classes::q::q2mem); every callee outside the run goes through `QMem::call` by address.
//! FUN_002333b4 (libc memset) / FUN_001ae5a0 (memcpy) are called through `call` like the original.
//! VU0 macro code from the disassembly: `adda vf0, x` adds vf0.w = 1 to the w lane; maddw.xyz leaves w.
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

/// DAT_002dded4: frame time step
pub const DT: u32 = 0x002d_ded4;
/// DAT_002a2da0: fallback direction when two rope points coincide
pub const DEFAULT_DIR: u32 = 0x002a_2da0;
/// DAT_002c3678: slack factor of the last segment (FUN_001e95b0)
pub const SLACK: u32 = 0x002c_3678;
/// DAT_002c367c: sample step along a path
pub const PATH_STEP: u32 = 0x002c_367c;
/// DAT_003e1200: range rebuild stamp
pub const STAMP: u32 = 0x003e_1200;

fn sub(a: V4, b: V4) -> V4 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2], a[3] - b[3]]
}
/// mul.xyz + addw.x / adday.x / maddz.x
fn dot3(a: V4, b: V4) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
/// `adda.xyzw ACC, vf0, a; madd*.xyzw d, b, s`: a + b * s, w + 1
fn amadd(a: V4, b: V4, s: f32) -> V4 {
    [a[0] + b[0] * s, a[1] + b[1] * s, a[2] + b[2] * s, 1.0 + a[3] + b[3] * s]
}
/// mulax / madday / maddaz / maddw.xyz: rows `m` (+0, +0x10, +0x20, +0x30) times point p (w kept from `w`)
fn xform(m: [V4; 4], p: V4, w: f32) -> V4 {
    let r: [f32; 3] = std::array::from_fn(|k| m[0][k] * p[0] + m[1][k] * p[1] + m[2][k] * p[2] + m[3][k]);
    [r[0], r[1], r[2], w]
}
fn alloc_copy<M: QMem + ?Sized>(m: &mut M, size: u32, src: u32) -> u32 {
    let s = stack_push(m, 0x10);
    m.w32(s, size);
    m.w32(s + 4, 0x10);
    let r = m.call(0x001a_e238, &[s, src], &[]).0;
    stack_pop(m, 0x10);
    r
}
fn ff<M: QMem + ?Sized>(m: &mut M, f: u32, a: &[u32], v: &[V4]) -> f32 {
    m.call(f, a, v).1[0]
}

/// FUN_001e9090(p): rope step. Point 0 = start object matrix (+0x40) * p +0x20, velocity 0 =
/// FUN_0022d348(start, 0, point 0); then each point i: v' = v + (p+0 - damping v) dt, x' = x + (v + v') dt/2,
/// pulled back to its rest length |s[i] - s[i-1]| along dir = normalize(x' - x[i-1]) (DAT_002a2da0 under
/// 1e-4) and v[i] = v' - ((v' - v[i-1]) . dir) dir. Mode +0x4c == 2 pins the last point to the end object
/// (+0x44, p +0x30) and grows / shrinks the rope (FUN_001e95b0). Points go to rope +0xf4 in rope space
/// (inverse of vtable slot 30's matrix, FUN_0011c318), then FUN_001e6330(rope).
pub fn ee_sub_1e9090<M: QMem + ?Sized>(m: &mut M, p: u32) {
    let dt = m.rf(DT);
    let o = m.r32(p + 0x40);
    let mo = m.rm(o + 0x40);
    let lp = m.rv(p + 0x20);
    // vf6 w: not written here (caller's register), 0 assumed
    let mut x6 = xform(mo, lp, 0.0);
    let rope = m.r32(p + 0x48);
    let mut n = m.r32(rope + 0xe8) as i32;
    let pa = m.r32(p + 0x54);
    m.wv(pa, x6);
    let va = m.r32(p + 0x58);
    let o = m.r32(p + 0x40);
    let mut v7 = m.call(0x0022_d348, &[o, 0], &[x6]).1;
    m.wv(va, v7);
    let h = dt * 0.5;
    let mut i = 1;
    while i < n {
        let off = i as u32 * 0x10;
        let damp = -m.rf(p + 0x60);
        let vb = m.r32(p + 0x58) + off;
        let v = m.rv(vb);
        let a = m.rv(p);
        let acc = amadd(a, v, damp);
        let v4 = amadd(v, acc, dt);
        let t_ = m.r32(p + 0x54);
        let x = m.rv(t_ + off);
        let x2 = {
            let t = amadd(x, v, h);
            [t[0] + v4[0] * h, t[1] + v4[1] * h, t[2] + v4[2] * h, t[3] + v4[3] * h]
        };
        let d = sub(x2, x6);
        let l = dot3(d, d).sqrt();
        let mut dir = m.rv(DEFAULT_DIR);
        if 0.0001 <= l {
            let q = 1.0 / l;
            dir = [d[0] * q, d[1] * q, d[2] * q, d[3] * q];
        }
        let s = m.r32(p + 0x5c) + 4 * i as u32;
        let dv = dot3(sub(v4, v7), dir);
        v7 = amadd(v4, dir, -dv);
        let seg = (m.rf(s) - m.rf(s - 4)).abs();
        x6 = amadd(x6, dir, seg);
        let t_ = m.r32(p + 0x54);
        m.wv(t_ + off, x6);
        let t_ = m.r32(p + 0x58);
        m.wv(t_ + off, v7);
        i += 1;
    }
    if m.r32(p + 0x4c) == 2 {
        let e = m.r32(p + 0x44);
        let me = m.rm(e + 0x40);
        let lp = m.rv(p + 0x30);
        let at = m.r32(p + 0x54) + (n as u32) * 0x10;
        // maddw.xyz into the register holding row 0: w = row 0's w
        m.wv(at - 0x10, xform(me, lp, me[0][3]));
        let a = m.rv(at - 0x10);
        let b = m.rv(at - 0x20);
        let d = sub(b, a);
        let l = dot3(d, d).sqrt();
        let t_ = m.r32(p + 0x5c);
        let s = m.rf(t_ + 4 * n as u32 - 8);
        ee_sub_1e95b0(m, s + l, p);
        let t_ = m.r32(p + 0x48);
        n = m.r32(t_ + 0xe8) as i32;
    }
    let rope = m.r32(p + 0x48);
    let mat = vslot(m, rope, 30, &[], &[]).0;
    let sp = stack_push(m, 0x40);
    m.call(0x0011_c318, &[sp, mat], &[]);
    let inv = m.rm(sp);
    stack_pop(m, 0x40);
    for i in 0..n.max(0) as u32 {
        let t_ = m.r32(p + 0x54);
        let x = m.rv(t_ + i * 0x10);
        let t_ = m.r32(p + 0x48);
        let dst = m.r32(t_ + 0xf4) + i * 0x10;
        m.wv(dst, xform(inv, x, inv[0][3]));
    }
    let rope = m.r32(p + 0x48);
    m.call(0x001e_6330, &[rope], &[]);
}

/// FUN_001e93f0(p, end, obj): attach rope end `end` (0 start / 1 end) to `obj` (+0x10 + 4 end): with an
/// object, +0x40 + 4 end = vtable slot 31 (out buffer) and +0x20 + 0x10 end = buffer +0x30; without, the end
/// sits on the rope path itself: +0x40 = &DAT_002c44f0, point = FUN_001e67b8(FUN_001e6610 / 6630(rope), rope).
pub fn ee_sub_1e93f0<M: QMem + ?Sized>(m: &mut M, p: u32, end: u32, obj: u32) {
    m.w32(p + 4 * end + 0x10, obj);
    if obj == 0 {
        m.w32(p + 4 * end + 0x40, 0x002c_44f0);
        let r = m.r32(p + 0x48);
        let u = m.call(if end == 0 { 0x001e_6610 } else { 0x001e_6630 }, &[r], &[]).1;
        let r = m.r32(p + 0x48);
        m.call(0x001e_67b8, &[r, p + end * 0x10 + 0x20, 0], &[[u[0], 0.0, 0.0, 0.0]]);
    } else {
        let sp = stack_push(m, 0x50);
        let r = vslot(m, obj, 31, &[sp], &[]).0;
        m.w32(p + 4 * end + 0x40, r);
        for k in 0..4 {
            let v = m.r32(sp + 0x30 + 4 * k);
            m.w32(p + end * 0x10 + 0x20 + 4 * k, v);
        }
        stack_pop(m, 0x50);
    }
}

/// FUN_001e94b0(parent, n): new rope object (class 10, FUN_0013e3c8(10, 0, parent, -1)): +0xe0 / +0xe4 = 0,
/// +0x60 identity rows (memset 0x40, diagonal 1.0), +0xec = {0, 1, .., n-1} as floats, +0xf0 n floats,
/// +0xf4 n points (frame allocator FUN_001ae1d0)
pub fn ee_sub_1e94b0<M: QMem + ?Sized>(m: &mut M, parent: u32, n: i32) -> u32 {
    let o = m.call(0x0013_e3c8, &[10, 0, parent, u32::MAX], &[]).0;
    m.w32(o + 0xe0, 0);
    m.w32(o + 0xe4, 0);
    m.call(0x0023_33b4, &[o + 0x60, 0, 0x40], &[]);
    for off in [0x74, 0x9c, 0x88, 0x60] {
        m.w32(o + off, 0x3f80_0000);
    }
    let al = |m: &mut M, size: u32| {
        let s = stack_push(m, 0x10);
        m.w32(s, size);
        m.w32(s + 4, 0x10);
        let r = m.call(0x001a_e1d0, &[s], &[]).0;
        stack_pop(m, 0x10);
        r
    };
    let a = al(m, (n << 2) as u32);
    m.w32(o + 0xec, a);
    for i in 0..n.max(0) {
        let b = m.r32(o + 0xec);
        m.wf(b + 4 * i as u32, i as f32);
    }
    let a = al(m, (n << 2) as u32);
    m.w32(o + 0xf0, a);
    let a = al(m, (n << 4) as u32);
    m.w32(o + 0xf4, a);
    o
}

fn lerp_slot<M: QMem + ?Sized>(m: &mut M, base: u32, t: f32) {
    // element -0x20 = (1 - t) * element -0x30 + t * element -0x10 (mulax / maddx .xyzw)
    let a = m.rv(base - 0x30);
    let b = m.rv(base - 0x10);
    m.wv(base - 0x20, std::array::from_fn(|k| a[k] * (1.0 - t) + b[k] * t));
}

/// FUN_001e95b0(len, p): set the rope length to `len` (arc lengths +0x5c, count rope +0xe8): shorter ->
/// drop points past it and end the last one at len; slightly longer (< s[n-2] + slack * seg) -> stretch the
/// last segment; longer -> insert points (one segment +0x88 before the end, interpolated position /
/// velocity / rope point / rope arc) up to +0x84, clearing mode +0x4c when the maximum is reached.
pub fn ee_sub_1e95b0<M: QMem + ?Sized>(m: &mut M, len: f32, p: u32) {
    let rope = m.r32(p + 0x48);
    let mut n = m.r32(rope + 0xe8) as i32;
    let s = m.r32(p + 0x5c);
    let at = (n as u32).wrapping_mul(4).wrapping_add(s);
    if len < m.rf(at.wrapping_sub(4)) {
        let mut i1;
        loop {
            i1 = n;
            n = i1 - 1;
            let t_ = m.r32(p + 0x5c);
            if n < 0 || !(len <= m.rf(t_.wrapping_add((n as u32).wrapping_mul(4)))) {
                break;
            }
        }
        let t_ = m.r32(p + 0x48);
        m.w32(t_ + 0xe8, (i1 + 1) as u32);
        let t_ = m.r32(p + 0x5c);
        m.wf(t_.wrapping_add((i1 as u32).wrapping_mul(4)), len);
    } else {
        let slack = m.rf(SLACK);
        if len < m.rf(at.wrapping_sub(8)) + slack * m.rf(p + 0x88) {
            m.wf(at.wrapping_sub(4), len);
        } else {
            let mut done = false;
            while n < m.r32(p + 0x84) as i32 {
                n += 1;
                let o3 = n as u32 * 0x10;
                let o4 = n as u32 * 4;
                let rope = m.r32(p + 0x48);
                for base in [m.r32(p + 0x54) + o3, m.r32(p + 0x58) + o3] {
                    let v = m.rv(base - 0x20);
                    m.wv(base - 0x10, v);
                }
                let t_ = m.r32(p + 0x5c);
                m.wf(t_ + o4 - 4, len);
                let b = m.r32(rope + 0xf4) + o3;
                let v = m.rv(b - 0x20);
                m.wv(b - 0x10, v);
                let b = m.r32(rope + 0xf0) + o4;
                let v = m.r32(b - 8);
                m.w32(b - 4, v);
                let b = m.r32(p + 0x5c) + o4;
                let seg = m.rf(p + 0x88);
                let s3 = m.rf(b - 0xc);
                m.wf(b - 8, s3 + seg);
                let t = (m.rf(b - 8) - s3) / (m.rf(b - 4) - s3);
                let t_ = m.r32(p + 0x54);
                lerp_slot(m, t_ + o3, t);
                let t_ = m.r32(p + 0x58);
                lerp_slot(m, t_ + o3, t);
                let rope = m.r32(p + 0x48);
                let t_ = m.r32(rope + 0xf4);
                lerp_slot(m, t_ + o3, t);
                let b = m.r32(rope + 0xf0) + o4;
                let (a0, a1) = (m.rf(b - 0xc), m.rf(b - 4));
                m.wf(b - 8, (1.0 - t) * a0 + t * a1);
                let b = m.r32(p + 0x5c) + o4;
                if len < m.rf(b - 8) + slack * m.rf(p + 0x88) {
                    m.wf(b - 4, len);
                    done = true;
                    break;
                }
            }
            if !done {
                m.w32(p + 0x4c, 0);
            }
            let rope = m.r32(p + 0x48);
            m.w32(rope + 0xe8, n as u32);
        }
    }
    let rope = m.r32(p + 0x48);
    m.call(0x001e_6330, &[rope], &[]);
}

/// FUN_001e9828(p): memcpy(p +0x5c, rope +0xf0, 4 * rope +0xe8)
pub fn ee_sub_1e9828<M: QMem + ?Sized>(m: &mut M, p: u32) {
    let rope = m.r32(p + 0x48);
    let (d, s, n) = (m.r32(p + 0x5c), m.r32(rope + 0xf0), m.r32(rope + 0xe8));
    m.call(0x001a_e5a0, &[d, s, n << 2], &[]);
}

/// FUN_001e9c30(o): the rail range record +0x180 (0x9c bytes, FUN_001ae2a0) built once by FUN_001e9c88(rec, o)
pub fn ee_sub_1e9c30<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.r32(o + 0x180) == 0 {
        let s = stack_push(m, 0x10);
        m.w32(s, 0x9c);
        m.w32(s + 4, 0x10);
        let r = m.call(0x001a_e2a0, &[s], &[]).0;
        stack_pop(m, 0x10);
        m.w32(o + 0x180, r);
        ee_sub_1e9c88(m, r, o);
    }
}

/// FUN_001e9c88(rec, path): sample `path` every DAT_002c367c from FUN_001e6620 to FUN_001e6648 (point
/// FUN_001e6f10, volume FUN_001c0d70): up to 64 distinct volume ids (+0x94, count +0x90) and up to 64
/// [from, to] ranges where the sample leaves the previous volume (id 0, a new id, or FUN_001c0920 false);
/// a closed path (+0xe4) wraps the first / last range. Ranges: +8 (FUN_001ae238 copy), +4 / +0xc count, +0x10
/// memcpy (8 bytes per range: overruns the 16 slots past 16 ranges, kept). +0x98 = -1.
pub fn ee_sub_1e9c88<M: QMem + ?Sized>(m: &mut M, rec: u32, path: u32) {
    let sp = stack_push(m, 0x340);
    let (ranges, ids, pt) = (sp, sp + 0x200, sp + 0x300);
    let mut cur: Option<u32> = None;
    let (mut nids, mut nr) = (0u32, 0i32);
    m.w32(rec, path);
    let f0 = ff(m, 0x001e_6620, &[path], &[]);
    let p0 = m.r32(rec);
    let f1 = ff(m, 0x001e_6648, &[p0], &[]);
    let p0 = m.r32(rec);
    m.call(0x001e_6f10, &[p0, pt, 0], &[[f0, 0.0, 0.0, 0.0]]);
    let v = m.rv(pt);
    let mut prev = m.call(0x001c_0d70, &[0], &[v]).0;
    if f0 < f1 {
        let mut t = f0;
        let mut last = f0;
        loop {
            let p0 = m.r32(rec);
            m.call(0x001e_6f10, &[p0, pt, 0], &[[t, 0.0, 0.0, 0.0]]);
            let v = m.rv(pt);
            let id = m.call(0x001c_0d70, &[0], &[v]).0;
            if id != 0 {
                let known = (0..nids).any(|i| m.r32(ids + 4 * i) == id);
                if !known && nids < 0x40 {
                    m.w32(ids + 4 * nids, id);
                    nids += 1;
                }
            }
            let out = if id == 0 {
                true
            } else if prev == id {
                m.call(0x001c_0920, &[prev, 0], &[v]).0 == 0
            } else {
                true
            };
            if out {
                if cur.is_none() && nr < 0x40 {
                    let r = ranges + nr as u32 * 8;
                    nr += 1;
                    m.wf(r, last);
                    cur = Some(r);
                }
            } else if let Some(r) = cur.take() {
                m.wf(r + 4, t);
            }
            let step = m.rf(PATH_STEP);
            last = t;
            prev = id;
            if !(t + step < f1) {
                break;
            }
            t += step;
        }
    }
    if let Some(r) = cur {
        m.wf(r + 4, f1);
    }
    let t_ = m.r32(rec);
    if 1 < nr && m.r32(t_ + 0xe4) != 0 && m.rf(ranges) == f0 {
        let l = ranges + (nr as u32 - 1) * 8;
        if m.rf(l + 4) == f1 {
            let a = m.rf(l) - f1;
            let b = m.rf(ranges + 4) + f1;
            m.wf(ranges, a);
            m.wf(l + 4, b);
        }
    }
    m.w32(rec + 4, nr as u32);
    let a = alloc_copy(m, (nr << 3) as u32, ranges);
    m.w32(rec + 0xc, nr as u32);
    m.w32(rec + 8, a);
    m.call(0x001a_e5a0, &[rec + 0x10, ranges, (nr << 3) as u32], &[]);
    m.w32(rec + 0x90, nids);
    let a = alloc_copy(m, nids << 2, ids);
    m.w32(rec + 0x94, a);
    m.w32(rec + 0x98, u32::MAX);
    stack_pop(m, 0x340);
}

/// FUN_001e9f50(pos): the first object of the volume at `pos` (FUN_001c0d70) among the up to 64 found by
/// FUN_001c1700(vol, 0, pos, pos, 0x40, buf) that contains `pos` (FUN_001bc9f0); 0 when none
pub fn ee_sub_1e9f50<M: QMem + ?Sized>(m: &mut M, pos: V4) -> u32 {
    let vol = m.call(0x001c_0d70, &[0], &[pos]).0;
    if vol == 0 {
        return 0;
    }
    let buf = stack_push(m, 0x100);
    let n = m.call(0x001c_1700, &[vol, 0, 0x40, buf], &[pos, pos]).0 as i32;
    let mut r = 0;
    for i in 0..n.max(0) as u32 {
        let x = m.r32(buf + 4 * i);
        if m.call(0x001b_c9f0, &[x], &[pos]).0 != 0 {
            r = x;
            break;
        }
    }
    stack_pop(m, 0x100);
    r
}

/// FUN_001ea000(rec): rebuild the visible ranges of a rail record when one of its volumes (+0x94, +0x114
/// change stamp) changed since +0x98: blocked spans (FUN_001e9f50 hit -> FUN_001bd560 sides 0 / 1, 3 = path
/// end, closed flag +0xe4 cleared meanwhile) are merged into the 16 pairs at +0x10, then FUN_0014c9f8 cuts
/// them out of the base ranges (+8, count +4) into +0x10 / +0xc (closed paths wrap). +0x98 = ++DAT_003e1200.
pub fn ee_sub_1ea000<M: QMem + ?Sized>(m: &mut M, rec: u32) {
    let nids = m.r32(rec + 0x90) as i32;
    if nids < 1 {
        return;
    }
    let ids = m.r32(rec + 0x94);
    let stamp = m.r32(rec + 0x98) as i32;
    let mut i = 0;
    while i < nids {
        let v = m.r32(ids + 4 * i as u32);
        if stamp < m.r32(v + 0x114) as i32 {
            break;
        }
        i += 1;
    }
    if i >= nids {
        return;
    }
    let pairs = rec + 0x10;
    let mut np: u32 = 0;
    let p0 = m.r32(rec);
    let f0 = ff(m, 0x001e_6620, &[p0], &[]);
    let p0 = m.r32(rec);
    let f1 = ff(m, 0x001e_6648, &[p0], &[]);
    let base = m.r32(rec + 8);
    let sp = stack_push(m, 0xc20);
    let (pt, bce0, b6e0, e0) = (sp, sp + 0x10, sp + 0x610, sp + 0xc10);
    let nb = m.r32(rec + 4) as i32;
    if f0 < f1 {
        let mut t = f0;
        loop {
            let p0 = m.r32(rec);
            m.call(0x001e_6f10, &[p0, pt, 0], &[[t, 0.0, 0.0, 0.0]]);
            let v = m.rv(pt);
            let hit = ee_sub_1e9f50(m, v);
            if hit != 0 {
                let path = m.r32(rec);
                let keep = m.r32(path + 0xe4);
                m.w32(path + 0xe4, 0);
                if m.call(0x001b_d560, &[hit, path, 0, 0, e0], &[[t, 0.0, 0.0, 0.0]]).0 == 3 {
                    m.wf(e0, f0);
                }
                if m.call(0x001b_d560, &[hit, path, 1, 0, e0 + 4], &[[t, 0.0, 0.0, 0.0]]).0 == 3 {
                    m.wf(e0 + 4, f1);
                }
                let t_ = m.r32(rec);
                m.w32(t_ + 0xe4, keep);
                let (e, d) = (m.rf(e0), m.rf(e0 + 4));
                let mut k = np as i32;
                let mut merge = false;
                loop {
                    if k <= 0 || m.rf(pairs + k as u32 * 8 - 4) < e {
                        break;
                    }
                    k -= 1;
                    if !(e <= m.rf(pairs + k as u32 * 8)) {
                        merge = true;
                        break;
                    }
                }
                np = k as u32 + 1;
                if 0x10 < np {
                    np = k as u32;
                }
                let at = pairs + k as u32 * 8;
                if !merge {
                    m.wf(at, e);
                }
                m.wf(at + 4, d);
                t = d;
            }
            t += m.rf(PATH_STEP);
            if f1 <= t {
                break;
            }
        }
    }
    for j in 0..nb.max(0) as u32 {
        let b = bce0 + j * 0x60;
        m.call(0x0023_33b4, &[b, 0, 0x60], &[]);
        let (a, c) = (m.r32(base + 8 * j), m.r32(base + 8 * j + 4));
        m.w32(b + 0x40, a);
        m.w32(b + 0x44, c);
    }
    for j in 0..np {
        let b = b6e0 + j * 0x60;
        m.call(0x0023_33b4, &[b, 0, 0x60], &[]);
        let (a, c) = (m.r32(pairs + 8 * j), m.r32(pairs + 8 * j + 4));
        m.w32(b + 0x40, a);
        m.w32(b + 0x44, c);
    }
    let n = m.call(0x0014_c9f8, &[np, b6e0, nb as u32, bce0, 0x10, b6e0, 0], &[]).0 as i32;
    for j in 0..n.max(0) as u32 {
        let b = b6e0 + j * 0x60;
        let (a, c) = (m.r32(b + 0x40), m.r32(b + 0x44));
        m.w32(pairs + 8 * j, a);
        m.w32(pairs + 8 * j + 4, c);
    }
    stack_pop(m, 0xc20);
    let t_ = m.r32(rec);
    if 1 < n && m.r32(t_ + 0xe4) != 0 && m.rf(pairs) == f0 {
        let l = pairs + (n as u32 - 1) * 8;
        if m.rf(l + 4) == f1 {
            let a = m.rf(l) - f1;
            let b = m.rf(pairs + 4) + f1;
            m.wf(pairs, a);
            m.wf(l + 4, b);
        }
    }
    m.w32(rec + 0xc, n as u32);
    let s = m.r32(STAMP).wrapping_add(1);
    m.w32(STAMP, s);
    m.w32(rec + 0x98, s);
}

/// FUN_001ea410(init, prio): the six pools (same static init as FUN_001f8c40 without the descriptor)
pub fn ee_sub_1ea410<M: QMem + ?Sized>(m: &mut M, init: u32, prio: u32) {
    if prio != 0xffff || init == 0 {
        return;
    }
    const POOLS: [(u32, u32, u32, u32); 6] = [
        (0x002d_d2e0, 0x0045_20a8, 0x14, 0x2000),
        (0x002d_d2e4, 0x0045_20d0, 0x18, 0x800),
        (0x002d_d2e8, 0x0045_20f8, 0x28, 0x200),
        (0x002d_d2ec, 0x0045_2120, 0x1c, 0x800),
        (0x002d_d2f0, 0x0045_2148, 0xc, 200),
        (0x002d_d2f4, 0x0045_2170, 0x14, 0x100),
    ];
    for (c, p, sz, n) in POOLS {
        let v = m.r32(c).wrapping_add(1);
        m.w32(c, v);
        if v == 1 {
            m.call(0x0011_cca8, &[p, sz, n, 0x10], &[]);
        }
    }
}

/// FUN_001ea548: FUN_001ea410(1, 0xffff)
pub fn ee_sub_1ea548<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_1ea410(m, 1, 0xffff);
}

/// FUN_001ea568(base): relocate the loaded tables: records DAT_003dcffc (stride 0x20, count DAT_003dcff8)
/// with kind (+4) 1 get +0xc += base and each of their (u16 +8) words fixed by FUN_001ed278; records
/// DAT_003dcff4 (stride 0x28, count DAT_003dcff0) with kind 1 get +0x14 += base
pub fn ee_sub_1ea568<M: QMem + ?Sized>(m: &mut M, base: u32) {
    let mut i = 0;
    while i < m.r32(0x003d_cff8) as i32 {
        let r = m.r32(0x003d_cffc) + i as u32 * 0x20;
        i += 1;
        if m.r8(r + 4) == 1 {
            let mut u = base.wrapping_add(m.r32(r + 0xc));
            m.w32(r + 0xc, u);
            while u < m.r32(r + 0xc).wrapping_add(m.r16(r + 8) as u32 * 4) {
                m.call(0x001e_d278, &[u], &[]);
                u += 4;
            }
        }
    }
    let n = m.r32(0x003d_cff0) as i32;
    let mut r = m.r32(0x003d_cff4);
    for _ in 0..n.max(0) {
        if m.r8(r + 4) == 1 {
            let v = base.wrapping_add(m.r32(r + 0x14));
            m.w32(r + 0x14, v);
        }
        r += 0x28;
    }
}

/// FUN_001ea9f0(world, tex): the 256-entry glow CLUT: colours k = 0..2 from FUN_0011dfa0(DAT_003e1300 + 0x10 k)
/// (r | g << 8 | b << 16) and alphas DAT_003e12f4[k] / 255; entry n (8 x 4 x 8 walk, DAT_002c3680 skips):
/// n & 0x81 = 0 -> 0, 1 -> colour 1, 0x80 -> colour 0, 0x81 -> colour 2, alpha byte =
/// (int)(max(max((n & 0x7f) - 1, 0) / 126, 0) * alpha * 128). Uploaded as a 16x16 CSM CLUT (GS packet
/// FUN_00166920 .. FUN_0016f8b8) at world +0x3ea0 (FUN_00170548(tex, 4) << 37 | 0x2000000001b00000 when tex);
/// nothing when neither tex nor a CLUT address is set.
pub fn ee_sub_1ea9f0<M: QMem + ?Sized>(m: &mut M, w: u32, tex: u32) {
    if tex == 0 && m.r64(w + 0x3ea0) & 0x7_ffe0_0000_0000 == 0 {
        return;
    }
    let tab = ee_sub_1ea9f0_table(m);
    let sp = stack_push(m, 0xc70);
    let (pkt, buf, out, clut) = (sp, sp + 0x30, sp + 0x830, sp + 0x840);
    for (i, v) in tab.iter().enumerate() {
        m.w32(clut + 4 * i as u32, *v);
    }
    if tex != 0 {
        let l = m.call(0x0017_0548, &[tex, 4], &[]).0 as u64;
        let v = l << 0x25 | 0x2000_0000_01b0_0000;
        m.w32(w + 0x3ea0, v as u32);
        m.w32(w + 0x3ea4, (v >> 32) as u32);
    }
    m.call(0x0016_6920, &[pkt], &[]);
    m.call(0x0015_7330, &[pkt, 0x80, buf], &[]);
    m.call(0x0015_74b8, &[pkt], &[]);
    let cbp = (m.r64(w + 0x3ea0) >> 0x25) as u32 & 0x3fff;
    m.call(0x0017_0650, &[cbp, 1, 0, 0x10, 0x10, 0x40, pkt], &[]);
    m.call(0x0015_7500, &[pkt, 0x40, clut], &[]);
    m.call(0x0016_6bd8, &[pkt, 0, 1, 0xe], &[]);
    m.call(0x0016_6e90, &[pkt, 0x3f, 0], &[]);
    m.call(0x0016_6e90, &[pkt, 0x61, 0], &[]);
    m.call(0x0016_6968, &[pkt], &[]);
    m.call(0x0015_76b0, &[pkt], &[]);
    m.call(0x0015_7348, &[pkt, 0, out], &[]);
    let (a, b) = (m.r32(0x002d_e598), m.r32(out));
    m.call(0x0016_f8b8, &[a, b], &[]);
    stack_pop(m, 0xc70);
}

/// the CLUT words of FUN_001ea9f0 (pure part)
pub fn ee_sub_1ea9f0_table<M: QMem + ?Sized>(m: &mut M) -> Vec<u32> {
    let mut col = [0u32; 3];
    let mut alp = [0f32; 3];
    let out = stack_push(m, 0x10);
    for k in 0..3u32 {
        m.call(0x0011_dfa0, &[0x003e_1300 + 0x10 * k, out], &[]);
        let (r, g, b) = (m.rf(out) as i32 as u32, m.rf(out + 4) as i32 as u32, m.rf(out + 8) as i32 as u32);
        alp[k as usize] = m.rf(0x003e_12f4 + 4 * k) * 0.003921569;
        col[k as usize] = r | g << 8 | b << 16;
    }
    stack_pop(m, 0x10);
    let mut t = Vec::with_capacity(256);
    let mut n: u32 = 0;
    for _ in 0..8 {
        for j in 0..4u32 {
            for _ in 0..8 {
                let k = match n & 0x81 {
                    0 => None,
                    1 => Some(1),
                    0x80 => Some(0),
                    _ => Some(2),
                };
                t.push(match k {
                    None => 0,
                    Some(k) => {
                        let f = (((n & 0x7f) as i32 - 1).max(0) as f32 * 0.007936508).max(0.0);
                        col[k] | ((f * (alp[k] * 128.0)) as i32 as u32) << 24
                    }
                });
                n = n.wrapping_add(1);
            }
            n = n.wrapping_add(m.r32(0x002c_3680 + 4 * j));
        }
    }
    t
}

/// FUN_001ead98(dst, src): copy the 16-byte table header, fix each of its (u16 +8) words at +0xc
/// (FUN_001ed278), +0x10 = qword count (4 n + 15) >> 4
pub fn ee_sub_1ead98<M: QMem + ?Sized>(m: &mut M, d: u32, s: u32) -> u32 {
    for k in 0..4 {
        let v = m.r32(s + 4 * k);
        m.w32(d + 4 * k, v);
    }
    let mut u = m.r32(d + 0xc);
    while u < m.r32(d + 0xc).wrapping_add(m.r16(d + 8) as u32 * 4) {
        m.call(0x001e_d278, &[u], &[]);
        u += 4;
    }
    let n = m.r16(d + 8) as u32;
    m.w32(d + 0x10, (n * 4 + 0xf) >> 4);
    d
}

/// FUN_001eb050(dst, src): copy 0x18 bytes, +0x18 = (+0x10 + 15) >> 4
pub fn ee_sub_1eb050<M: QMem + ?Sized>(m: &mut M, d: u32, s: u32) -> u32 {
    for k in 0..6 {
        let v = m.r32(s + 4 * k);
        m.w32(d + 4 * k, v);
    }
    let v = m.r32(d + 0x10).wrapping_add(0xf) >> 4;
    m.w32(d + 0x18, v);
    d
}
