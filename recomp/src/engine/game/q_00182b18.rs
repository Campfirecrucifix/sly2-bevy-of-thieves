//! WP-Q4: run 0x00182b18..0x00185110 - the attach-target member callbacks (hang point / yaw per target
//! class: rail 0x183040 / 0x1830d8, sway 0x183218, pole 0x1834d0 / 0x183540 / 0x183600, point 0x183818,
//! 0x184208 / 0x184510 / 0x1845a0 / 0x184f98), the binoc-state switch FUN_00183b80, attach helpers
//! (0x1841f0, 0x184380, 0x184670), the stream close (0x184950 / 0x184cf0) and the per-TU pool static
//! inits (= FUN_001ea410). The value-level Rust model of the member callbacks is
//! `player::p5_attach::FUN_00182cd8`; these are the 1:1 QMem ports for the EE-layout callers.
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};
use crate::engine::game::q_001e9090::ee_sub_1ea410;

/// DAT_002a2dc0 (+z) / DAT_002a2da0 (+x) unit quadwords
pub const UNIT_Z: u32 = 0x002a_2dc0;
pub const UNIT_X: u32 = 0x002a_2da0;
const PI: f32 = std::f32::consts::PI;

fn f(v: f32) -> V4 {
    [v, 0.0, 0.0, 0.0]
}
fn add(a: V4, b: V4) -> V4 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2], a[3] + b[3]]
}
fn sub(a: V4, b: V4) -> V4 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2], a[3] - b[3]]
}
/// pos + (0,0,0,1) + n * r (adda vf0 / maddx)
fn madd(p: V4, n: V4, r: f32) -> V4 {
    [p[0] + n[0] * r, p[1] + n[1] * r, p[2] + n[2] * r, p[3] + 1.0 + n[3] * r]
}
fn atan2<M: QMem + ?Sized>(m: &mut M, y: f32, x: f32) -> f32 {
    m.call(0x0024_58f0, &[], &[f(y), f(x)]).1[0]
}
fn rand<M: QMem + ?Sized>(m: &mut M, lo: f32, hi: f32) -> f32 {
    m.call(0x0011_d448, &[], &[f(lo), f(hi)]).1[0]
}
fn pos_of<M: QMem + ?Sized>(m: &mut M, o: u32) -> V4 {
    m.call(0x001a_62e8, &[o], &[]).1
}
/// normalize(tan x +z) (VU0 opmula / opmsub), +x below 0.0001
fn side_normal<M: QMem + ?Sized>(m: &mut M, t: V4) -> V4 {
    let u: V4 = m.rv(UNIT_Z);
    let c = [t[1] * u[2] - t[2] * u[1], t[2] * u[0] - t[0] * u[2], t[0] * u[1] - t[1] * u[0], 0.0];
    let l = (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt();
    if l < 0.0001 {
        m.rv(UNIT_X)
    } else {
        let q = 1.0 / l;
        [c[0] * q, c[1] * q, c[2] * q, c[3] * q]
    }
}

/// FUN_00182b18 / 00184750 / 00184df8 / 00184fd8 (per-TU static pool init) = FUN_001ea410
pub fn ee_sub_182b18<M: QMem + ?Sized>(m: &mut M, init: u32, prio: u32) {
    ee_sub_1ea410(m, init, prio);
}
pub fn ee_sub_184750<M: QMem + ?Sized>(m: &mut M, init: u32, prio: u32) {
    ee_sub_1ea410(m, init, prio);
}
pub fn ee_sub_184df8<M: QMem + ?Sized>(m: &mut M, init: u32, prio: u32) {
    ee_sub_1ea410(m, init, prio);
}
pub fn ee_sub_184fd8<M: QMem + ?Sized>(m: &mut M, init: u32, prio: u32) {
    ee_sub_1ea410(m, init, prio);
}
/// FUN_00182c50 / 184888 / 184f30 / 185110: (1, 0xffff)
pub fn ee_sub_182c50<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_1ea410(m, 1, 0xffff);
}
pub fn ee_sub_184888<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_1ea410(m, 1, 0xffff);
}
pub fn ee_sub_184f30<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_1ea410(m, 1, 0xffff);
}
pub fn ee_sub_185110<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_1ea410(m, 1, 0xffff);
}

/// FUN_00182fe8(path, out): +0xe4 (looped) -> out = {FUN_001e6620, FUN_001e6648 (length)}; returns +0xe4 != 0
pub fn ee_sub_182fe8<M: QMem + ?Sized>(m: &mut M, path: u32, out: u32) -> bool {
    if m.r32(path + 0xe4) == 0 {
        return false;
    }
    let a = m.call(0x001e_6620, &[path], &[]).1[0];
    m.wf(out, a);
    let b = m.call(0x001e_6648, &[path], &[]).1[0];
    m.wf(out + 4, b);
    true
}

/// FUN_00183040(path, jt, s): s = closest parameter (FUN_001e6f60) to *(jt +0x218); open paths clamp to
/// [30, length - 30]
pub fn ee_sub_183040<M: QMem + ?Sized>(m: &mut M, path: u32, jt: u32, s: u32) {
    let sp = stack_push(m, 0x40);
    let p = m.r32(jt + 0x218);
    let q = m.rv(p);
    m.call(0x001e_6f60, &[path, 0, sp, 0, 0, s], &[q]);
    if m.r32(path + 0xe4) == 0 {
        let l = m.call(0x001e_6648, &[path], &[]).1[0];
        let mut v = m.rf(s);
        if v < 30.0 {
            v = 30.0;
        } else if l - 30.0 < v {
            v = l - 30.0;
        }
        m.wf(s, v);
    }
    stack_pop(m, 0x40);
}

/// FUN_001830d8(path, jt, outp, outyaw) (f12 s): n = normalize(tangent x +z); outp = point + n * jt +0x103c
/// - jt +0x240; outyaw = atan2(n) (n negated when jt +0x1040 == 1)
pub fn ee_sub_1830d8<M: QMem + ?Sized>(m: &mut M, path: u32, jt: u32, outp: u32, outyaw: u32, s: f32) {
    let sp = stack_push(m, 0x70);
    m.call(0x001e_6f10, &[path, sp, sp + 0x10], &[f(s)]);
    let t = m.rv(sp + 0x10);
    let mut n = side_normal(m, t);
    if outp != 0 {
        let r = m.rf(jt + 0x103c);
        let p = m.rv(sp);
        let g = m.rv(jt + 0x240);
        m.wv(outp, sub(madd(p, n, r), g));
    }
    if outyaw != 0 {
        if m.r32(jt + 0x1040) == 1 {
            n = [-n[0], -n[1], -n[2], -n[3]];
        }
        let y = atan2(m, n[1], n[0]);
        m.wf(outyaw, y);
    }
    stack_pop(m, 0x70);
}

/// FUN_00183218(path, _, outp, outdir): s = FUN_001e8008(path, DAT_002be8b8); outp = point(s) +
/// normalize(tangent x +z) * rand(0, DAT_002be8bc); outdir = +z
pub fn ee_sub_183218<M: QMem + ?Sized>(m: &mut M, path: u32, _p2: u32, outp: u32, outdir: u32) {
    let sp = stack_push(m, 0x90);
    let k = m.rf(0x002b_e8b8);
    let s = m.call(0x001e_8008, &[path], &[f(k)]).1[0];
    m.call(0x001e_6f10, &[path, sp, sp + 0x10], &[f(s)]);
    let t = m.rv(sp + 0x10);
    let n = side_normal(m, t);
    let p = m.rv(sp);
    let hi = m.rf(0x002b_e8bc);
    let r = rand(m, 0.0, hi);
    m.wv(outp, madd(p, n, r));
    let z = m.rv(UNIT_Z);
    m.wv(outdir, z);
    stack_pop(m, 0x90);
}

/// FUN_001834d0(pole, jt, out): out = atan2(*(jt +0x218) - pos(pole)) * radius +0x10c (arc parameter)
pub fn ee_sub_1834d0<M: QMem + ?Sized>(m: &mut M, o: u32, jt: u32, out: u32) {
    let pp = m.r32(jt + 0x218);
    let q = m.rv(pp);
    let c = pos_of(m, o);
    let d = sub(q, c);
    let a = atan2(m, d[1], d[0]);
    let r = m.rf(o + 0x10c);
    m.wf(out, a * r);
}

/// FUN_00183540(pole, jt, outp, outyaw) (f12 s): th = s / +0x10c; outp = pos + FUN_0011e1a0(th, +0x10c +
/// jt +0x103c, 0) - jt +0x240; outyaw = th (+ pi when jt +0x1040 == 1)
pub fn ee_sub_183540<M: QMem + ?Sized>(m: &mut M, o: u32, jt: u32, outp: u32, outyaw: u32, s: f32) {
    let r = m.rf(o + 0x10c);
    let th = s / r;
    let rr = r + m.rf(jt + 0x103c);
    if outp != 0 {
        let c = pos_of(m, o);
        let e = m.call(0x0011_e1a0, &[], &[f(th), f(rr), f(0.0)]).1;
        let g = m.rv(jt + 0x240);
        m.wv(outp, sub(add(c, e), g));
    }
    if outyaw != 0 {
        let y = if m.r32(jt + 0x1040) == 1 { th + PI } else { th };
        m.wf(outyaw, y);
    }
}

/// FUN_00183600(pole, outp, outdir): outp = pos + FUN_0011e1a0(rand(-pi, pi), rand(0, DAT_002be8c4) +
/// +0x10c, 0); outdir = +z
pub fn ee_sub_183600<M: QMem + ?Sized>(m: &mut M, o: u32, outp: u32, outdir: u32) {
    let a = rand(m, -PI, PI);
    let hi = m.rf(0x002b_e8c4);
    let r = rand(m, 0.0, hi) + m.rf(o + 0x10c);
    let c = pos_of(m, o);
    let e = m.call(0x0011_e1a0, &[], &[f(a), f(r), f(0.0)]).1;
    m.wv(outp, add(c, e));
    let z = m.rv(UNIT_Z);
    m.wv(outdir, z);
}

/// FUN_00183818(point, jt, outp, outyaw): r = jt +0x103c when +0x10c == 0 else 0; row0 of FUN_001a6318;
/// outp = pos - jt +0x240 + row0 * r; outyaw = atan2(row0) (+ pi when jt +0x1040 == 1)
pub fn ee_sub_183818<M: QMem + ?Sized>(m: &mut M, o: u32, jt: u32, outp: u32, outyaw: u32) {
    let r = if m.r32(o + 0x10c) == 0 { m.rf(jt + 0x103c) } else { 0.0 };
    let fr = m.call(0x001a_6318, &[o], &[]).0;
    let row = m.rv(fr);
    if outp != 0 {
        let c = pos_of(m, o);
        let g = m.rv(jt + 0x240);
        m.wv(outp, madd(sub(c, g), row, r));
    }
    if outyaw != 0 {
        let mut y = atan2(m, row[1], row[0]);
        if m.r32(jt + 0x1040) == 1 {
            y += PI;
        }
        m.wf(outyaw, y);
    }
}

/// FUN_00183b80(o, state): +0x17c change: leaving 2 with +0x18c resets the binoc pieces +0x58 / +0x5c
/// (FUN_002303a0 1, +0x9a = 0), vslot 34(DAT_002a2dd0), FUN_001f5720 / 1f57c0 / 1f68a8(0), then vslot
/// 2(6), FUN_00147f40(world), +0x190 = 0; entering 2 with +0x18c aims the pieces at the target
/// (+0x190 +0x10bc or +0x60) via vslot 9, euler of the bone (FUN_0022fd38 / 11bff0) and FUN_0022fc48,
/// then vslot 2(5); +0x180 = DAT_002dded0
pub fn ee_sub_183b80<M: QMem + ?Sized>(m: &mut M, o: u32, st: u32) {
    let cur = m.r32(o + 0x17c);
    if cur == st {
        return;
    }
    if cur == 2 {
        if m.r32(o + 0x18c) != 0 {
            m.call(0x001a_58d0, &[o, 0, 0xf], &[]);
            for off in [0x58, 0x5c] {
                let p = m.r32(o + off);
                m.call(0x0023_03a0, &[o, p, 1], &[]);
                let p = m.r32(o + off);
                m.w16(p + 0x9a, 0);
            }
            let v = m.rv(0x002a_2dd0);
            vslot(m, o, 34, &[], &[v]);
            m.call(0x001f_5720, &[o, 0], &[]);
            m.call(0x001f_57c0, &[o, 0], &[]);
            m.call(0x001f_68a8, &[o, 0], &[]);
        }
        vslot(m, o, 2, &[6, 0, 0, 0, 0], &[]);
        m.call(0x0014_7f40, &[0x003d_d490], &[]);
        m.w32(o + 0x190, 0);
    }
    if st == 2 {
        if m.r32(o + 0x18c) != 0 {
            m.call(0x001f_5720, &[o, 1], &[]);
            m.call(0x001f_57c0, &[o, 1], &[]);
            let a = m.r32(o + 0x190);
            let mut t = m.r32(a + 0x10bc);
            if t == 0 {
                t = m.r32(a + 0x60);
            }
            vslot(m, o, 9, &[a, t, 3], &[]);
            let sp = stack_push(m, 0xb0);
            let fr = m.call(0x001a_6318, &[o], &[]).0;
            m.call(0x0011_b938, &[fr, 0, sp], &[]);
            let b = m.r32(o + 0x5c);
            let bone = m.r32(b + 0xa0);
            m.call(0x0022_fd38, &[sp + 0x30, 0, bone, sp], &[]);
            for k in 0..3 {
                let v = m.rv(sp + 0x30 + 0x10 * k);
                m.wv(sp + 0x10 * k, v);
            }
            let e = m.call(0x0011_bff0, &[sp], &[]).1;
            let b = m.r32(o + 0x5c);
            m.call(0x0023_03a0, &[o, b, 7], &[]);
            let b = m.r32(o + 0x5c);
            m.w32(b + 0xc0, 0);
            m.wf(b + 0xc8, e[2]);
            m.w32(b + 0xc4, 0);
            m.w16(b + 0x9a, st as u16);
            let off: V4 = m.rv(0x002b_e8f0);
            let tp = m.rv(t + 0x70);
            let a8 = m.r32(o + 0x58);
            let bone = m.r32(a8 + 0xa0);
            let r = m.call(0x0022_fc48, &[0, bone], &[add(tp, off)]).1;
            let a8 = m.r32(o + 0x58);
            m.call(0x0023_03a0, &[o, a8, 7], &[]);
            let a8 = m.r32(o + 0x58);
            m.wv(a8 + 0xc0, r);
            m.w16(a8 + 0x9a, st as u16);
            m.call(0x001f_68a8, &[o, 1], &[]);
            stack_pop(m, 0xb0);
        }
        vslot(m, o, 2, &[5, 0, 0, 0, 0], &[]);
    }
    m.w32(o + 0x17c, st);
    let t = m.r32(0x002d_ded0);
    m.w32(o + 0x180, t);
}

/// FUN_001841f0(a, b): (a +0xfc & b +0x1000) != 0
pub fn ee_sub_1841f0<M: QMem + ?Sized>(m: &mut M, a: u32, b: u32) -> bool {
    m.r32(a + 0xfc) & m.r32(b + 0x1000) != 0
}

/// FUN_00184208(o, _, out): out = pos(o)
pub fn ee_sub_184208<M: QMem + ?Sized>(m: &mut M, o: u32, _p2: u32, out: u32) {
    if out != 0 {
        let p = pos_of(m, o);
        m.wv(out, p);
    }
}

/// FUN_00184380(a, b): b +0xf4 == a +0x19c -> FUN_001a6af8(b, a +0x180) else 1
pub fn ee_sub_184380<M: QMem + ?Sized>(m: &mut M, a: u32, b: u32) -> u32 {
    if m.r32(b + 0xf4) == m.r32(a + 0x19c) {
        let t = m.r32(a + 0x180);
        m.call(0x001a_6af8, &[b, t], &[]).0
    } else {
        1
    }
}

/// FUN_00184510(o, _, out): DAT_002dfb84 == 0: out = FUN_001a6f00(o, 0, +0x110) with z = pos(+0x15c).z,
/// else out = pos(+0x15c); out.z += DAT_002be908
pub fn ee_sub_184510<M: QMem + ?Sized>(m: &mut M, o: u32, _p2: u32, out: u32) {
    if out == 0 {
        return;
    }
    let tgt = m.r32(o + 0x15c);
    let z = if m.r32(0x002d_fb84) == 0 {
        let l = m.rv(o + 0x110);
        let v = m.call(0x001a_6f00, &[o, 0], &[l]).1;
        m.wv(out, v);
        pos_of(m, tgt)[2]
    } else {
        let v = pos_of(m, tgt);
        m.wv(out, v);
        m.rf(out + 8)
    };
    let c = m.rf(0x002b_e908);
    m.wf(out + 8, z + c);
}

/// FUN_001845a0(o, outp, outdir): outp = FUN_001a6f00(o, 0, +0x110) + FUN_0011e1a0(rand(-pi, pi),
/// rand(0, DAT_002be90c), 0); outdir = +z
pub fn ee_sub_1845a0<M: QMem + ?Sized>(m: &mut M, o: u32, outp: u32, outdir: u32) {
    let a = rand(m, -PI, PI);
    let hi = m.rf(0x002b_e90c);
    let r = rand(m, 0.0, hi);
    let l = m.rv(o + 0x110);
    let c = m.call(0x001a_6f00, &[o, 0], &[l]).1;
    let e = m.call(0x0011_e1a0, &[], &[f(a), f(r), f(0.0)]).1;
    m.wv(outp, add(c, e));
    let z = m.rv(UNIT_Z);
    m.wv(outdir, z);
}

/// FUN_00184670(o): the attach extension of o by class (0x58 +0x150, 0x1b +0x130, 0x22 / 0x23 / 0x27
/// +0xe0, 0xc / 0xb / 0xd +400, 0x13 +0x100), 0 otherwise
pub fn ee_sub_184670<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    if o == 0 {
        return 0;
    }
    for (c, off) in [(0x58, 0x150), (0x1b, 0x130), (0x22, 0xe0), (0xc, 400), (0x23, 0xe0), (0x13, 0x100), (0xb, 400), (0xd, 400), (0x27, 0xe0)] {
        if m.call(0x0013_2b10, &[o, c], &[]).0 != 0 {
            return o + off;
        }
    }
    0
}

/// FUN_00184950(s, flags): +0x30 = vtable 0x2d2918, FUN_00184cf0(s), flags & 1 -> FUN_001ae520(s) (free)
pub fn ee_sub_184950<M: QMem + ?Sized>(m: &mut M, s: u32, fl: u32) {
    m.w32(s + 0x30, 0x002d_2918);
    ee_sub_184cf0(m, s);
    if fl & 1 != 0 {
        m.call(0x001a_e520, &[s], &[]);
    }
}

/// FUN_00184cf0(s): close the stream: fd +0x3c >= 0 -> FUN_00238138(fd), -1; +0x50 >= 0 -> (+0x44 ->
/// FUN_00242b78) -1; FUN_001177f0(s)
pub fn ee_sub_184cf0<M: QMem + ?Sized>(m: &mut M, s: u32) {
    let fd = m.r32(s + 0x3c);
    if (fd as i32) >= 0 {
        m.call(0x0023_8138, &[fd], &[]);
        m.w32(s + 0x3c, u32::MAX);
    }
    if (m.r32(s + 0x50) as i32) >= 0 {
        if m.r32(s + 0x44) != 0 {
            m.call(0x0024_2b78, &[], &[]);
        }
        m.w32(s + 0x50, u32::MAX);
    }
    m.call(0x0011_77f0, &[s], &[]);
}

/// FUN_00184f98(o, _, out): out = pos(o), z += DAT_002be910
pub fn ee_sub_184f98<M: QMem + ?Sized>(m: &mut M, o: u32, _p2: u32, out: u32) {
    let p = pos_of(m, o);
    m.wv(out, p);
    let c = m.rf(0x002b_e910);
    let z = m.rf(out + 8);
    m.wf(out + 8, z + c);
}
