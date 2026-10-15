//! WP-Q5: run_00189618 queue package: the player's foot / hand IK targets (rig handles +0x19e4 body
//! lean, +0x19e8 / +0x19ec limbs, foot records +0x1520 / +0x1620 built by FUN_00189c40 and floor-tested
//! by FUN_00189ec8, driven by FUN_0018a0c8) and the rail / pole lean FUN_00189618; plus the
//! climb-target update FUN_0018a4e8. Ported 1:1 from the disassembly (the decompiler drops the
//! FUN_001300b8 / FUN_0012da30 quadword arguments) against `QMem`; locals whose address escapes or
//! that the original reads uninitialised live on the emulated stack.
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem, V4};
use crate::engine::classes::q::q5util::{f4, pos};

/// DAT_002a2dc0 world up, DAT_002a2dd0 zero quad
const UP: u32 = 0x002a_2dc0;
const ZERO: u32 = 0x002a_2dd0;

fn vcall<M: QMem + ?Sized>(m: &mut M, o: u32, off: u32, a: &[u32], v: &[V4]) -> (u32, V4) {
    let vt = m.r32(o + 0xc);
    let adj = m.r16(vt + off - 4) as i16 as i32 as u32;
    let f = m.r32(vt + off);
    let mut args = vec![o.wrapping_add(adj)];
    args.extend_from_slice(a);
    m.call(f, &args, v)
}

fn len3(v: V4) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn scale(v: V4, k: f32) -> V4 {
    [v[0] * k, v[1] * k, v[2] * k, v[3] * k]
}

/// `vadda ACC, vf0, a; vmaddx d, up, k` : (a.xyz, a.w + 1) + up * k
fn up_off(a: V4, up: V4, k: f32) -> V4 {
    [a[0] + up[0] * k, a[1] + up[1] * k, a[2] + up[2] * k, 1.0 + a[3] + up[3] * k]
}

/// FUN_00189618: body-lean handle. Blank all three handles while the binoculars are up; otherwise
/// pick the support object (ground +0x208's +0x4c, or in state 7 the +0x4d4 rail / pole for sub-states
/// 3 / 4). On a rail (sub-state 3, rail +0x1bc) the rail point (FUN_001e67b8 at +0x4dc) is advanced by
/// the velocity along it (slot +0x10c . dir * DAT_002bf2d0, clamped to the rail range or wrapped by
/// FUN_0011d898) and, with a sway rig (+0x7d4), the side lean (FUN_0011e290 / DAT_002bf2d4) springs
/// +0x7d8 (FUN_0011d1c0) into the three blend weights +0x7b0 / +0x7bc / +0x7c8. The local offset
/// to the support point (FUN_0022d348 in the support's frame, through the +0x1b0 basis) sets the lean
/// direction (strength curve DAT_002bf310, height min'd with the two foot curves DAT_002bf330).
pub fn ee_sub_189618<M: QMem + ?Sized>(m: &mut M, s: u32) {
    let sp = stack_push(m, 0x70);
    let z = m.rv(ZERO);
    m.wv(sp, z);
    let h = m.r32(s + 0x19e4);
    if h == 0 || m.call(0x0012_d058, &[h], &[]).0 == 0 {
        stack_pop(m, 0x70);
        return;
    }
    if m.call(0x0014_43d8, &[s], &[]).0 != 0 {
        for o in [0x19e4, 0x19e8, 0x19ec] {
            let h = m.r32(s + o);
            let z = m.rv(ZERO);
            m.call(0x0012_da30, &[h], &[z]);
        }
        stack_pop(m, 0x70);
        return;
    }
    let mut sup = 0u32;
    let st = m.r32(s + 0x18c8);
    if st == 0 {
        let g = m.r32(s + 0x208);
        if g != 0 {
            sup = m.r32(g + 0x4c);
        }
    } else if st == 7 && m.r32(s + 0x4cc) == 0 {
        let sub = m.r32(s + 0x4c8);
        if sub == 4 {
            sup = m.r32(s + 0x4d4);
            let p = pos(m, sup);
            m.wv(sp + 0x10, p);
        } else if sub == 3 {
            let r = m.r32(s + 0x4d4);
            sup = r;
            let t = m.rf(s + 0x4dc);
            m.call(0x001e_67b8, &[r, sp + 0x10, sp + 0x20], &[f4(t)]);
            if m.r32(r + 0x1bc) != 0 {
                rail_lean(m, s, r, sp);
            }
        }
    }
    if sup == 0 {
        stack_pop(m, 0x70);
        return;
    }
    let o = vcall(m, sup, 0x104, &[0], &[]).0;
    let l = m.rv(sp + 0x10);
    let q = m.call(0x0022_d348, &[o, 0], &[l]).1;
    m.wv(sp + 0x20, q);
    let (r0, r1, r2) = (m.rv(s + 0x1b0), m.rv(s + 0x1c0), m.rv(s + 0x1d0));
    let dot = |r: V4| r[0] * q[0] + r[1] * q[1] + r[2] * q[2];
    let d = [dot(r0), dot(r1), dot(r2), r0[3]];
    m.wv(sp + 0x40, d);
    let n = (d[0] * d[0] + d[1] * d[1]).sqrt();
    if 0.0001 < n {
        let u = (n / m.rf(0x002b_f300)).min(1.0);
        let c = m.rf(0x002b_f310) + u * (m.rf(0x002b_f314) + u * m.rf(0x002b_f318));
        let (lo, hi) = (m.rf(0x002b_bb60), m.rf(0x002b_bb64));
        let w = if c < lo { lo } else if hi < c { hi } else { c };
        let k = m.rf(0x002b_f2d8);
        m.wf(sp + 0x30, d[0] * k * w / n);
        m.wf(sp + 0x34, d[1] * k * w / n);
        let y = m.rf(0x002b_f2dc) * (1.0 - w * w).sqrt();
        m.wf(sp + 0x38, y);
    } else {
        m.w32(sp + 0x38, 0);
        m.w32(sp + 0x34, 0);
        m.w32(sp + 0x30, 0);
    }
    let fa = m.r32(s + 0x19f0);
    let fb = m.r32(s + 0x19f4);
    let (xa, xb) = (m.rf(fa + 0x28), m.rf(fb + 0x28));
    let pa = m.rf(0x002b_f330) + xa * (m.rf(0x002b_f334) + xa * m.rf(0x002b_f338));
    let pb = m.rf(0x002b_f330) + xb * (m.rf(0x002b_f334) + xb * m.rf(0x002b_f338));
    let y = m.rf(sp + 0x38);
    let hgt = if y < pa && y < pb { y } else if pa < pb { pa } else { pb };
    let w = if hgt.to_bits() & 0x8000_0000 != 0 { f32::from_bits(!0) } else { 0.0 };
    let v = [m.rf(sp + 0x30), m.rf(sp + 0x34), hgt, w];
    let h = m.r32(s + 0x19e4);
    m.call(0x0012_da30, &[h], &[v]);
    let lim = scale(m.rv(sp + 0x30), m.rf(0x002b_f2e0));
    let lim = if m.r32(s + 0x18d8) == 0xffff_ffff { lim } else { m.rv(ZERO) };
    for o in [0x19ec, 0x19e8] {
        let h = m.r32(s + o);
        m.call(0x0012_da30, &[h], &[lim]);
    }
    stack_pop(m, 0x70);
}

/// rail sub-state of FUN_00189618 (frame `sp`: +0 lean, +0x10 rail point, +0x20 rail dir, +0x30 v)
fn rail_lean<M: QMem + ?Sized>(m: &mut M, s: u32, r: u32, sp: u32) {
    let d = m.rv(sp + 0x20);
    let d = scale(d, 1.0 / len3(d));
    m.wv(sp + 0x20, d);
    let vel = vcall(m, s, 0x10c, &[], &[]).1;
    let sp_ = vel[0] * d[0] + vel[1] * d[1] + vel[2] * d[2];
    m.wv(sp + 0x30, scale(d, sp_));
    let mut t = m.rf(s + 0x4e0) + sp_ * m.rf(0x002b_f2d0);
    let hi = m.call(0x001e_6648, &[r], &[]).1[0];
    if m.r32(r + 0xe4) != 0 {
        t = m.call(0x0011_d898, &[], &[f4(t), f4(hi)]).1[0];
    } else {
        let lo = m.call(0x001e_6620, &[r], &[]).1[0];
        if t < lo {
            t = lo;
        } else if hi < t {
            t = hi;
        }
    }
    m.call(0x001e_6f10, &[r, 0, sp + 0x20], &[f4(t)]);
    let v5 = m.rv(sp + 0x20);
    let n = scale(v5, 1.0 / len3(v5));
    let a = scale(n, sp_);
    let b = m.rv(sp + 0x30);
    let lean = [a[0] - b[0], a[1] - b[1], a[2] - b[2], a[3] - b[3]];
    m.wv(sp, lean);
    if m.r32(s + 0x7d4) == 0 {
        return;
    }
    let up = m.rv(UP);
    // OPMULA up, v5 / OPMSUB: up x v5 (w kept from up)
    let c = [up[1] * v5[2] - up[2] * v5[1], up[2] * v5[0] - up[0] * v5[2], up[0] * v5[1] - up[1] * v5[0], up[3]];
    let c = scale(c, 1.0 / len3(c));
    let f = m.call(0x0011_e290, &[0, sp], &[lean, c]).1[0] / m.rf(0x002b_f2d4);
    let mut f = if f < -1.0 {
        -1.0
    } else if 1.0 < f {
        1.0
    } else {
        f
    };
    if m.r32(s + 0x4e4) != 0 {
        f = -f;
    }
    let (a7d8, a7dc, dt) = (m.rf(s + 0x7d8), m.rf(s + 0x7dc), m.rf(0x002d_ded4));
    let x = m.call(0x0011_d1c0, &[0x002b_f320, s + 0x7dc], &[f4(a7d8), f4(a7dc), f4(f * (2.0 - f.abs())), f4(dt)]).1[0];
    m.wf(s + 0x7d8, x);
    let p = x.max(0.0);
    let q = (-x).max(0.0);
    m.wf(s + 0x7b0, p);
    m.wf(s + 0x7c8, q);
    m.wf(s + 0x7bc, 1.0 - p - q);
    let g = m.r32(s + 0x7d4);
    m.call(0x0013_84c0, &[g], &[]);
}

/// FUN_00189c40: build foot record `rec` (+0 cached anim, +4 floor hit, +8 probe length, +0x10 hip
/// point, +0x20 foot point, +0x30 / +0x40 hit, +0x50 foot point lowered by +8) for limb bones `a` /
/// `b` when the driving animation (+0x14 of the state-selected +0x1034 / +0x18e0 / +0x1030 clip) changed:
/// three bone frames through FUN_001300b8 (phase from FUN_0012f938 channel 0x16a), the floor cast
/// FUN_0014c748 between the hip lowered by DAT_002bf340 / DAT_002bf344
pub fn ee_sub_189c40<M: QMem + ?Sized>(m: &mut M, k: f32, s: u32, a: u32, b: u32, _c: u32, rec: u32) {
    let clip = if m.r32(s + 0x18c8) == 0 {
        let st = m.r32(s + 0x18d8) as i32;
        if (6..10).contains(&st) || st == 0xc || st == 0xd {
            m.r32(s + 0x18e0)
        } else {
            let x = m.r32(s + 0x1034);
            m.r32(x + 0x30)
        }
    } else {
        let x = m.r32(s + 0x1034);
        if x == 0 {
            m.r32(s + 0x1030)
        } else {
            m.r32(x + 0x30)
        }
    };
    let an = m.r32(clip + 0x14);
    if an == m.r32(rec) {
        return;
    }
    m.w32(rec, an);
    let sp = stack_push(m, 0x140);
    let ch = m.call(0x0012_f938, &[an, 0x16a], &[]).0;
    let ph = if ch != 0 { m.rf(ch) } else { 0.0 };
    let bone = m.r32(s + 0x60);
    m.call(0x0013_00b8, &[an, s, a, bone, sp, 0], &[f4(ph), f4(0.0)]);
    let up = m.rv(UP);
    let hip = up_off(m.rv(sp + 0x30), up, -k);
    let bone = m.r32(s + 0x60);
    m.call(0x0013_00b8, &[an, s, b, bone, sp + 0x50, 0], &[f4(ph), f4(0.0)]);
    let x = m.r32(b + 0xa8);
    let xb = m.r32(x);
    let bone = m.r32(s + 0x60);
    m.call(0x0013_00b8, &[an, s, xb, bone, sp + 0x90, 0], &[f4(ph), f4(0.0)]);
    let x = m.r32(b + 0xa8);
    let l = m.rv(x + 0xa0);
    let r = [m.rv(sp + 0x90), m.rv(sp + 0xa0), m.rv(sp + 0xb0), m.rv(sp + 0xc0)];
    let mut t = r[0];
    for c in 0..3 {
        t[c] = r[0][c] * l[0] + r[1][c] * l[1] + r[2][c] * l[2] + r[3][c];
    }
    let base = m.rv(sp + 0x80);
    let foot = [
        base[0] + hip[0] - t[0],
        base[1] + hip[1] - t[1],
        base[2] + hip[2] - t[2],
        base[3] + hip[3] - t[3],
    ];
    m.wv(rec + 0x20, foot);
    let len = m.rf(x + 0xc0) + m.rf(x + 0xc4) + 10.0;
    m.wf(rec + 8, len);
    let (d0, d1) = (m.rf(0x002b_f340), m.rf(0x002b_f344));
    let p1 = up_off(hip, up, d1);
    let p0 = up_off(hip, up, d0);
    let hit = m.call(0x0014_c748, &[1, sp + 0xd0], &[f4(len), foot, p0, p1]).0;
    m.w32(rec + 4, (0 < hit as i32) as u32);
    let (h0, h1) = (m.rv(sp + 0xd0), m.rv(sp + 0xe0));
    m.wv(rec + 0x30, h0);
    m.wv(rec + 0x10, hip);
    m.wv(rec + 0x40, h1);
    let f = m.rv(rec + 0x20);
    let l = m.rf(rec + 8);
    m.wv(rec + 0x50, up_off(f, up, -l));
    stack_pop(m, 0x140);
}

/// FUN_00189ec8: floor test for foot record `rec` (when +4): gather (FUN_0020faa0) around the three
/// world points of +0x30 / +0x40 / +0x50, cast +0x30 -> +0x40, else +0x40 -> +0x50; on a hit with a
/// non-zero +0x40 and a normal z > 0.7 the handle `h` gets the offset of the hit (lifted by -up) from
/// the hip, else zero (and 0 is returned when grounded with the foot below -85)
pub fn ee_sub_189ec8<M: QMem + ?Sized>(m: &mut M, s: u32, rec: u32, h: u32) -> u32 {
    if m.r32(rec + 4) == 0 {
        return 1;
    }
    let sp = stack_push(m, 0xc0);
    m.call(0x001a_e478, &[sp + 0x60], &[]);
    let w: Vec<V4> = [0x30, 0x40, 0x50]
        .iter()
        .map(|o| {
            let q = m.rv(rec + o);
            m.call(0x001a_6f00, &[s, 0], &[q]).1
        })
        .collect();
    let mut lo = w[0];
    let mut hi = w[0];
    for c in 0..3 {
        lo[c] = w[0][c].min(w[1][c].min(w[2][c]));
        hi[c] = w[0][c].max(w[1][c].max(w[2][c]));
    }
    let ec = m.r32(s + 0xec);
    m.call(0x0020_faa0, &[0x003d_d490, 0x0020_f948, ec, sp + 0x80, sp + 0x84], &[lo, hi]);
    let (n, l) = (m.r32(sp + 0x80), m.r32(sp + 0x84));
    let mut hit = m.call(0x0014_cc58, &[0, n, l, sp], &[w[0], w[1]]).0 != 0;
    if !hit {
        let (n, l) = (m.r32(sp + 0x80), m.r32(sp + 0x84));
        hit = m.call(0x0014_cc58, &[0, n, l, sp], &[w[1], w[2]]).0 != 0;
    }
    let mut r = 1;
    if hit && m.rf(sp + 0x40) != 0.0 && 0.7 < m.rf(sp + 0x28) {
        let up = m.rv(UP);
        let p = m.rv(sp);
        let p = [p[0] - up[0], p[1] - up[1], p[2] - up[2], p[3] - up[3]];
        m.wv(sp, p);
        let q = m.call(0x001a_6f00, &[0, s], &[p]).1;
        let hip = m.rv(rec + 0x10);
        m.call(0x0012_da30, &[h], &[[q[0] - hip[0], q[1] - hip[1], q[2] - hip[2], q[3] - hip[3]]]);
    } else {
        let z = m.rv(ZERO);
        m.call(0x0012_da30, &[h], &[z]);
        if m.r32(s + 0x208) != 0 && m.rf(rec + 0x48) < -85.0 {
            r = 0;
        }
    }
    m.call(0x001a_e4a0, &[sp + 0x60, 2], &[]);
    stack_pop(m, 0xc0);
    r
}

/// FUN_0018a0c8: foot IK frame: FUN_001f1350(+0x1034, FUN_001430c8()); when slot +0x2e4 is clear and
/// grounded, FUN_00204660 gets +0x1a8 + DAT_002dded4 * FUN_0011cfe0(surface slot +0x114 z, 10); with a
/// left foot handle (+0x19f0) both foot records are built and floor-tested; the first miss (when
/// +0x1514 is clear and +0x1028 has channel 0x17c, FUN_001f09d8 on +0x1034) latches +0x1514
pub fn ee_sub_18a0c8<M: QMem + ?Sized>(m: &mut M, s: u32) {
    let x = m.call(0x0014_30c8, &[], &[]).0;
    let a = m.r32(s + 0x1034);
    m.call(0x001f_1350, &[a, x], &[]);
    if vcall(m, s, 0x2e4, &[], &[]).0 == 0 {
        let g = m.r32(s + 0x208);
        if g != 0 {
            let o = m.r32(g + 0x4c);
            let q = vcall(m, o, 0x114, &[], &[]).1;
            let f = m.call(0x0011_cfe0, &[], &[f4(q[2]), f4(10.0)]).1[0];
            let v = m.rf(s + 0x1a8) + f * m.rf(0x002d_ded4);
            m.call(0x0020_4660, &[s], &[f4(v)]);
        }
    }
    let fa = m.r32(s + 0x19f0);
    if fa == 0 {
        return;
    }
    let (a, b) = (m.r32(s + 0x19d8), m.r32(s + 0x19e0));
    ee_sub_189c40(m, 5.0, s, a, b, fa, s + 0x1520);
    let (a, b, c) = (m.r32(s + 0x19d4), m.r32(s + 0x19dc), m.r32(s + 0x19f4));
    ee_sub_189c40(m, 5.0, s, a, b, c, s + 0x1620);
    let h = m.r32(s + 0x19f0);
    let ok = ee_sub_189ec8(m, s, s + 0x1520, h) != 0 && {
        let h = m.r32(s + 0x19f4);
        ee_sub_189ec8(m, s, s + 0x1620, h) != 0
    };
    if ok || m.r32(s + 0x1514) != 0 {
        return;
    }
    let an = m.r32(s + 0x1028);
    if an == 0 {
        return;
    }
    if (m.call(0x001f_0400, &[an, 0x17c], &[]).0 as i32) >= 0 {
        let a = m.r32(s + 0x1034);
        m.call(0x001f_09d8, &[a, 0x17c], &[]);
    }
    m.w32(s + 0x1514, 1);
}

/// FUN_0018a4e8: climb target `p` {player, .., +8 state, +0x14 rail, +0x20 rail t, +0x2c mode, +0x40
/// target, +0x50 speed, +0x68 owner} for result `r`: r+0x30 = FUN_00204740(player); when the player's
/// +0x1024 is the owner: modes 1 / 2 extend the target 125 * speed^2 along the rail (clamped to the
/// rail ends, +-100 past flagged ends, FUN_001e6f10) or along the facing (FUN_001a6318 +0x10), mode 0
/// takes the position; the target is raised by 125 (rail) / 50. On a rail (FUN_00132b10 kind 0xc,
/// non-looping) the owner's +0x114 +0x10 lean approaches a +-150 edge push (PosApproachTrapezoid
/// 0x2bf348). States 0 / 0xd set r+0x3c, 1 / 3 / 4 nothing, others r bit 0
pub fn ee_sub_18a4e8<M: QMem + ?Sized>(m: &mut M, p: u32, r: u32) {
    if m.r32(p + 8) != 0 {
        return;
    }
    let pl = m.r32(p);
    let f = m.call(0x0020_4740, &[pl], &[]).1[0];
    m.wf(r + 0x30, f);
    let pl = m.r32(p);
    if m.r32(pl + 0x1024) != m.r32(p + 0x68) {
        return;
    }
    m.w32(r + 0x34, 0);
    let mode = m.r32(p + 0x2c) as i32;
    if mode < 3 && mode >= 0 {
        let add;
        if mode == 0 {
            let pl = m.r32(p);
            let q = pos(m, pl);
            m.wv(p + 0x40, q);
            add = 50.0;
        } else {
            let rl = m.r32(p + 0x14);
            if m.call(0x0013_2b10, &[rl, 0xc], &[]).0 != 0 {
                let rl = m.r32(p + 0x14);
                let v = m.rf(p + 0x50);
                let d = v * (v * 125.0);
                let mut t;
                if mode == 1 {
                    t = m.rf(p + 0x20) + d;
                    if m.r32(rl + 0xe4) == 0 {
                        let mut e = m.call(0x001e_6648, &[rl], &[]).1[0];
                        if m.r32(rl + 0x1bc) & 2 != 0 {
                            e += 100.0;
                        }
                        t = e.min(t);
                    }
                } else {
                    t = m.rf(p + 0x20) - d;
                    if m.r32(rl + 0xe4) == 0 {
                        let mut e = m.call(0x001e_6620, &[rl], &[]).1[0];
                        if m.r32(rl + 0x1bc) & 1 != 0 {
                            e -= 100.0;
                        }
                        t = e.max(t);
                    }
                }
                m.call(0x001e_6f10, &[rl, p + 0x40, 0], &[f4(t)]);
                add = 125.0;
            } else {
                let pl = m.r32(p);
                let q = pos(m, pl);
                let v = m.rf(p + 0x50);
                let d = v * (v * 125.0);
                let pl = m.r32(p);
                let fr = m.call(0x001a_6318, &[pl], &[]).0;
                let dir = m.rv(fr + 0x10);
                let sg = if mode == 1 { 1.0 } else { -1.0 };
                let mut o = q;
                for c in 0..4 {
                    o[c] = q[c] + sg * dir[c] * d;
                }
                m.wv(p + 0x40, o);
                add = 50.0;
            }
        }
        let z = m.rf(p + 0x48) + add;
        m.wf(p + 0x48, z);
    }
    let rl = m.r32(p + 0x14);
    if m.call(0x0013_2b10, &[rl, 0xc], &[]).0 != 0 {
        let rl = m.r32(p + 0x14);
        if m.r32(rl + 0xe4) == 0 {
            let e = m.call(0x001e_6648, &[rl], &[]).1[0];
            let t = m.rf(p + 0x20);
            let b = ((e - 30.0) - 30.0) * 0.5;
            let b = b.min(300.0);
            let top = (e - 30.0) - b;
            let push = if t < b + 30.0 {
                let u = ((b + 30.0) - t) / b;
                let u = if u < 0.0 { 0.0 } else if 1.0 < u { 1.0 } else { u };
                -u * u * 150.0
            } else if top < t {
                let u = (t - top) / b;
                let u = if u < 0.0 { 0.0 } else if 1.0 < u { 1.0 } else { u };
                u * u * 150.0
            } else {
                0.0
            };
            let o = m.r32(p + 0x68);
            let l = m.r32(o + 0x114);
            let cur = m.rf(l + 0x10);
            let dt = m.rf(0x002d_ded4);
            let v = m.call(0x0011_d018, &[0x002b_f348, 0], &[f4(cur), f4(push), f4(dt)]).1[0];
            let o = m.r32(p + 0x68);
            let l = m.r32(o + 0x114);
            m.wf(l + 0x10, v);
        }
    }
    match m.r32(p + 8) {
        0 | 0xd => m.w32(r + 0x3c, 1),
        1 | 3 | 4 => {}
        _ => {
            let x = m.r32(r) | 1;
            m.w32(r, x);
        }
    }
}
