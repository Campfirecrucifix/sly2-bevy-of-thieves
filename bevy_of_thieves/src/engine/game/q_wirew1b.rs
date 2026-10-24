//! WIRE-W1b: 1:1 queue ports (against `QMem`) of live-frontier natives (guard/AI message handling,
//! path closest point, matrix -> axis/angle, sound-ish queue merge, rotor update). Ported from the EE asm
//! (tools/eedis.py); callees go through `m.call` / `vslot` (the live dispatch routes ported ones).
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

const GS: u32 = 0x003d_d490; // game state
const T_NOW: u32 = 0x002d_ded0; // frame time
const T_DT: u32 = 0x002d_ded4; // frame dt

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

fn cpn<M: QMem + ?Sized>(m: &mut M, dst: u32, src: u32, n: u32) {
    for k in (0..n).step_by(4) {
        let w = m.r32(src + k);
        m.w32(dst + k, w);
    }
}

fn sub(a: V4, b: V4) -> V4 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2], a[3] - b[3]]
}

fn scl(a: V4, s: f32) -> V4 {
    [a[0] * s, a[1] * s, a[2] * s, a[3] * s]
}

fn dot3(a: V4, b: V4) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn len3(a: V4) -> f32 {
    dot3(a, a).sqrt()
}

/// `vf00w / |v|` then `v * Q` (xyzw)
fn unit(a: V4) -> V4 {
    scl(a, 1.0 / len3(a))
}

/// rows r0..r2 * v.xyz (+ r3), xyz written; w kept from `w`
fn xf(r: &[V4; 4], v: V4, w: f32, with_t: bool) -> V4 {
    let mut o = [0.0; 4];
    for i in 0..3 {
        o[i] = r[0][i] * v[0] + r[1][i] * v[1] + r[2][i] * v[2] + if with_t { r[3][i] } else { 0.0 };
    }
    o[3] = w;
    o
}

/// virtual call through the vtable at `obj + 0x20` (slot n: fn +8n+0xc, adjust +8n+8)
fn vslot20<M: QMem + ?Sized>(m: &mut M, obj: u32, n: u32, a: &[u32], v: &[V4]) -> (u32, V4) {
    let vt = m.r32(obj + 0x20);
    let f = m.r32(vt + 8 * n + 0xc);
    let adj = m.r16(vt + 8 * n + 8) as i16 as i32;
    let mut args = vec![obj.wrapping_add(adj as u32)];
    args.extend_from_slice(a);
    m.call(f, &args, v)
}

/// FUN_0017d6c8(o): state-machine update loop (FUN_001eef98 / FUN_001ef0a8 frame), +0x30 state.
pub fn ee_sub_17d6c8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef98, &[o], &[]);
    let a = m.r32(o + 0x10);
    if m.r32(a + 0xfe8) & 2 == 0 {
        return;
    }
    loop {
        if m.call(0x001e_f0a8, &[o], &[]).0 != 0 {
            return;
        }
        let s2 = m.r32(o + 0x30) as i32;
        match s2 {
            3 => {
                let a = m.r32(o + 0x18);
                if m.call(0x0017_e570, &[a], &[]).0 != 0 {
                    m.call(0x0017_da28, &[o], &[]);
                }
            }
            5 => {
                let a = m.r32(o + 0x18);
                if m.call(0x0017_eae8, &[a], &[]).0 != 0 {
                    m.call(0x0017_da28, &[o], &[]);
                }
            }
            1 | 2 => 'b: {
                let s1 = m.r32(o + 0x18);
                if s2 == 2 && m.call(0x0017_8d00, &[s1], &[]).0 != 0 {
                    m.call(0x0017_d450, &[o, 3], &[]);
                    break 'b;
                }
                let d = m.rf(T_NOW) - m.rf(o + 0xa0);
                let lim = m.rf(0x002b_e758);
                let g = m.r32(GS + 0x45cc);
                if lim < d || m.r32(o + 0xa4) != m.r32(g + 0x28) {
                    m.call(0x0017_da28, &[o], &[]);
                    break 'b;
                }
                if m.call(0x0017_92e8, &[s1], &[]).0 == 0 {
                    break 'b;
                }
                let f20 = m.rf(s1 + 0x34) + m.rf(0x002b_e75c);
                let p = m.r32(s1 + 0x30);
                let f0 = m.call(0x001e_6648, &[p], &[]).1[0];
                m.w32(o + 0x80, 0);
                m.w32(o + 0x84, 0);
                let u = f0.min(f20);
                let p = m.r32(s1 + 0x30);
                m.call(0x001e_6f10, &[p, o + 0x90, 0], &[f4(u)]);
                let a = m.r32(o + 0x10);
                let q = m.r32(a + 0x218);
                let v = m.rf(q + 8);
                m.wf(o + 0x98, v);
                m.call(0x0017_d450, &[o, 5], &[]);
            }
            _ => {}
        }
        if s2 as u32 == m.r32(o + 0x30) {
            return;
        }
    }
}

/// FUN_00178ee0(o): aim/track update; +0x80 target, +0xc4 rate, +0xd0 last pos, +0xe0 timer.
pub fn ee_sub_178ee0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0017_89f8, &[o], &[]);
    if m.r32(o + 0x4c) >= 2 {
        return;
    }
    let a = m.r32(o + 0x10);
    let p = m.call(0x001a_62e8, &[a], &[]).1;
    let f3 = m.rf(0x002b_e6a0);
    let ax = m.rv(0x002a_2dc0);
    let v3 = sub(m.rv(o + 0x80), p);
    let d = v3[0] * ax[0] + v3[1] * ax[1] + v3[2] * ax[2];
    let v4 = sub(v3, scl(ax, d));
    let f2 = m.rf(0x002b_e6a8);
    let f1 = m.rf(0x002b_e6a4);
    let l = (v4[0] * v4[0] + v4[1] * v4[1]).sqrt();
    let f12 = f3 + l * (f1 + l * f2);
    let lo = m.rf(0x002b_bb60);
    let hi = m.rf(0x002b_bb64);
    let f20 = if f12 < lo { lo } else if hi < f12 { hi } else { f12 };
    let a = m.r32(o + 0x18);
    let s0 = m.r32(a + 0x40);
    let k = m.r32(s0);
    if k == 0 {
        let r = m.rf(o + 0xc4) * f20;
        let a = m.r32(s0 + 8);
        m.call(0x0013_1a00, &[a], &[f4(r)]);
        let t = m.r32(o + 0x10) + 0xfd0;
        let y = m.call(0x0024_58f0, &[], &[f4(v4[1]), f4(v4[0])]).1[0];
        m.call(0x0020_9818, &[t], &[f4(y)]);
    } else if k == 3 {
        let a = m.r32(o + 0x10);
        let mp = m.call(0x001a_6318, &[a], &[]).0;
        let r0 = m.rv(mp);
        let r1 = m.rv(mp + 16);
        let r2 = m.rv(mp + 32);
        let a0 = m.r32(s0 + 4);
        let v8 = [dot3(r0, v4), dot3(r1, v4), dot3(r2, v4), 0.0];
        let q = m.rv(a0 + 16);
        let f0 = m.rf(o + 0xc4) * f20 * (q[0] * q[0] + q[1] * q[1]).sqrt();
        let n = len3(v8);
        let dir = if n < f32::from_bits(0x38d1_b717) { m.rv(0x002a_2dd0) } else { scl(v8, 1.0 / n) };
        let a = m.r32(s0 + 8);
        m.call(0x0020_cb28, &[a], &[scl(dir, f0)]);
        let a = m.r32(s0 + 8);
        let b = m.r32(a + 0x24);
        m.call(0x0013_1a00, &[b], &[f4(f20)]);
    }
    let last = m.rv(o + 0xd0);
    let dd = len3(sub(p, last));
    let f0 = if m.rf(0x002b_e6b0) < dd {
        m.wv(o + 0xd0, p);
        m.rf(0x002b_e694)
    } else {
        m.rf(o + 0xe0) - m.rf(T_DT)
    };
    m.wf(o + 0xe0, f0);
}

/// FUN_00177ba0(o): build a 0x70 message (kind at +0, target +0x10, f +0x14) and post via FUN_00176750.
pub fn ee_sub_177ba0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let sp = stack_push(m, 0x70);
    m.call(0x0023_33b4, &[sp, 0, 0x70], &[]);
    'e: {
        let a = m.r32(o + 0x10);
        if m.call(0x0013_2b10, &[a, 0x3b], &[]).0 != 0 {
            let a = m.r32(o + 0x10);
            if (m.r32(a + 0x1010) as i32) > 0 {
                m.w32(sp, 0xd);
                let v = m.call(0x001c_4840, &[GS], &[]).0;
                m.w32(sp + 0x10, v);
                break 'e;
            }
        }
        let a = m.r32(o + 0x10);
        if m.r32(a + 0xfbc) == 0x15 {
            m.w32(sp, 0x12);
            let v = m.call(0x001c_4840, &[GS], &[]).0;
            m.w32(sp + 0x10, v);
            let f = m.rf(0x002a_54d0);
            m.wf(sp + 0x14, f);
            break 'e;
        }
        if m.call(0x0017_7d20, &[o], &[]).0 != 0 {
            m.w32(sp, 0xc);
            break 'e;
        }
        if m.call(0x0017_7d70, &[o], &[]).0 != 0 {
            m.w32(sp, 0xf);
            break 'e;
        }
        let a = m.r32(o + 0x10);
        let s0 = vslot(m, a, 87, &[], &[]).0;
        if s0 != 0 {
            let a = m.r32(o + 0x10);
            if m.call(0x0020_8b38, &[a, s0], &[]).0 != 0 {
                let a = m.r32(o + 0x10);
                m.w32(sp, 0x12);
                m.w32(sp + 0x10, s0);
                let f = m.call(0x0020_8bb0, &[a], &[]).1[0];
                m.wf(sp + 0x14, f);
            } else {
                m.w32(sp + 0x10, s0);
                m.w32(sp, 4);
            }
            break 'e;
        }
        let st = m.r32(o + 0x30) as i32;
        if st >= 0xb || st < 6 {
            m.w32(sp, 1);
            break 'e;
        }
        let a = m.r32(o + 0x10);
        m.w32(sp, 2);
        let p = m.call(0x001a_62e8, &[a], &[]).1;
        let a3 = m.r32(o + 0x10);
        let v = m.call(0x0015_6b98, &[GS, 0, a3], &[p]).0;
        m.w32(sp + 0x10, v);
    }
    m.call(0x0017_6750, &[o, sp], &[]);
    stack_pop(m, 0x70);
}

/// FUN_001758a0(o, tgt, ex): try every attack slot (+0xedc[+0xefc]) of `o` on `tgt`; returns the slot
/// that fired (0 when none / when +0x78 hand-off).
pub fn ee_sub_1758a0<M: QMem + ?Sized>(m: &mut M, o: u32, tgt: u32, ex: u32) -> u32 {
    if m.r32(o + 0xf00) != 0 || tgt == 0 {
        return 0;
    }
    if (m.r64(tgt + 0x28) >> 60) & 1 == 0 {
        return 0;
    }
    if m.rf(T_NOW) < m.rf(o + 0xf04) {
        return 0;
    }
    if m.call(0x0013_2b10, &[tgt, 0x32], &[]).0 != 0 && vslot(m, tgt, 76, &[], &[]).0 != 0 {
        return 0;
    }
    if ex != 0 && m.call(0x0020_a818, &[ex, o], &[]).0 == 0 {
        return 0;
    }
    let pa = m.call(0x001a_62e8, &[o], &[]).1;
    let pb = m.call(0x001a_62e8, &[tgt], &[]).1;
    let d = sub(pb, pa);
    let mut f21 = d[2];
    let mut f20 = (d[0] * d[0] + d[1] * d[1]).sqrt();
    let a = m.r32(tgt + 0x3c);
    let a = m.r32(a);
    let s0 = m.call(0x001c_9950, &[a], &[]).0;
    let a = m.r32(o + 0x3c);
    let a = m.r32(a);
    if s0 == m.call(0x001c_9950, &[a], &[]).0 {
        f20 += if tgt < o { 75.0 } else { -75.0 };
    }
    let k = m.r32(o + 0xfbc);
    if k == 8 || k == 0xd {
        let q = m.r32(o + 0xfc4);
        if q != 0 {
            let f = m.rf(q + 0x20);
            f21 /= f;
            f20 /= f;
        }
    }
    let sp = stack_push(m, 0xc0);
    let mut i = 0i32;
    let mut ret = 0;
    while i < m.r32(o + 0xefc) as i32 {
        let s1 = m.r32(o + 0xedc + 4 * i as u32);
        i += 1;
        if m.r32(s1 + 0xa8) == 0 {
            continue;
        }
        if m.call(0x0011_d910, &[s1 + 0x50], &[f4(f20)]).0 == 0 {
            continue;
        }
        if m.call(0x0011_d910, &[s1 + 0x58], &[f4(f21)]).0 == 0 {
            continue;
        }
        let l = m.call(0x001a_6f00, &[0, o], &[pb]).1;
        let ang = m.call(0x0024_58f0, &[], &[f4(l[1]), f4(l[0])]).1[0];
        let f5 = ang.abs();
        let c = 0x002b_e600;
        let mut f1 = m.rf(c) + f20 * (m.rf(c + 4) + f20 * m.rf(c + 8));
        let lo = m.rf(0x002b_bb60);
        let hi = m.rf(0x002b_bb64);
        if f1 < lo {
            f1 = lo;
        } else if hi < f1 {
            f1 = hi;
        }
        if m.rf(s1 + 0x68) < f1 * f5 {
            continue;
        }
        if m.r32(s1 + 0x70) != 0 {
            m.call(0x001a_e478, &[sp], &[]);
            m.wv(sp + 0x10, pa);
            m.wv(sp + 0x20, pb);
            m.call(0x0020_faa0, &[GS, 0x0017_5820, s1, sp + 0x90, sp + 0x94], &[pa, pb]);
            let a1 = m.rv(sp + 0x10);
            let a2 = m.rv(sp + 0x20);
            let a3 = m.r32(sp + 0x90);
            let t0 = m.r32(sp + 0x94);
            let hit = m.call(0x0014_cc58, &[0, a3, t0, sp + 0x30], &[a1, a2]).0;
            if hit != 0 {
                let dd = len3(sub(m.rv(sp + 0x10), m.rv(sp + 0x30)));
                if dd < m.rf(s1 + 0x6c) {
                    m.call(0x001a_e4a0, &[sp, 2], &[]);
                    continue;
                }
            }
            m.call(0x001a_e4a0, &[sp, 2], &[]);
        }
        let s0 = sp + 0x10;
        m.w32(s0, 0x17);
        m.w32(s0 + 4, 1);
        m.w32(sp + 0x98, s1 + 0x18);
        vslot(m, o, 2, &[0x18, 0, 1, sp + 0x98, s0], &[]);
        if m.r32(sp + 0x10) != 0x17 || m.r32(sp + 0x14) == 0 {
            m.call(0x0012_9ec8, &[s0], &[]);
            continue;
        }
        ret = s1;
        if m.r32(s1 + 0x78) != 0 {
            let r = vslot(m, tgt, 31, &[sp + 0x20], &[]).0;
            let q = m.rv(sp + 0x50);
            m.call(0x0020_ab68, &[o, s1, r], &[q]);
            ret = 0;
            if ex != 0 {
                m.call(0x0020_a9a8, &[ex, 3], &[]);
            }
        }
        m.call(0x0012_9ec8, &[sp + 0x10], &[]);
        break;
    }
    stack_pop(m, 0xc0);
    ret
}

/// FUN_001541d8(o, tgt, mode): visibility / reachability check of `tgt` for sensor `o` (0 / 1 / 2).
pub fn ee_sub_1541d8<M: QMem + ?Sized>(m: &mut M, o: u32, tgt: u32, mode: i32) -> i32 {
    let pl = m.call(0x001c_4840, &[GS], &[]).0;
    if tgt == 0 || m.call(0x001a_61a0, &[tgt], &[]).0 == 0 {
        return 0;
    }
    if m.call(0x0013_2b10, &[tgt, 0x32], &[]).0 != 0 && vslot(m, tgt, 76, &[], &[]).0 == 2 {
        return 0;
    }
    if m.call(0x0013_2b10, &[tgt, 0x34], &[]).0 != 0 && pl != 0 && m.call(0x0013_2b10, &[pl, 0x46], &[]).0 != 0 {
        return 0;
    }
    if m.call(0x0020_f1b8, &[GS], &[]).0 != 0 || m.r32(o + 0xf4) != 0 {
        return 0;
    }
    if m.call(0x0013_2b10, &[tgt, 0x34], &[]).0 != 0 && m.r32(tgt + 0x11ac) != 0 {
        return 0;
    }
    if m.r32(o + 0xf0) != 0 {
        return 1;
    }
    let p = if m.call(0x0013_2b10, &[tgt, 0x33], &[]).0 != 0 {
        let q = m.call(0x001a_62e8, &[tgt], &[]).1;
        let off = m.rv(tgt + 0x240);
        [q[0] + off[0], q[1] + off[1], q[2] + off[2], q[3] + off[3]]
    } else {
        m.call(0x001a_62e8, &[tgt], &[]).1
    };
    let sp = stack_push(m, 0x180);
    let mut s1 = m.r32(0x003e_1480);
    let mut blocked = false;
    while s1 != 0 {
        let q = m.call(0x0015_3e60, &[o], &[]).1;
        let a = m.call(0x001a_6f00, &[0, s1], &[q]).1;
        let b = m.call(0x001a_6f00, &[0, s1], &[p]).1;
        let h = m.r32(s1 + 0xec);
        if (m.call(0x0011_8668, &[h, 0x7fff_ffff, 4, sp], &[a, b]).0 as i32) > 0 {
            blocked = true;
            break;
        }
        s1 = m.r32(s1 + 0x130);
    }
    stack_pop(m, 0x180);
    if blocked {
        return 0;
    }
    let mut s2 = mode;
    let f8: u32;
    match mode {
        0 => {
            let v = m.r32(o + 0xe4);
            if v != 0 {
                let mut l = m.r32(v + 0x64);
                while l != 0 {
                    if m.r32(l + 0xc) == tgt {
                        if m.r32(l) != 0 {
                            return 1;
                        }
                        break;
                    }
                    l = m.r32(l + 4);
                }
            }
            let a = m.r32(o + 0xe8);
            if m.call(0x0022_7c90, &[a], &[p]).0 == 0 {
                return 0;
            }
            let a = m.r32(o + 0xec);
            if a != 0 && m.call(0x0022_7c90, &[a], &[p]).0 == 0 {
                return 0;
            }
            if m.r32(o + 0xf8) & 1 == 0 && m.call(0x0015_4770, &[o, tgt], &[]).0 != 0 {
                return 0;
            }
            if m.r32(o + 0xf8) & 2 != 0 {
                return 1;
            }
            return (m.call(0x0015_45f0, &[o, tgt], &[]).0 == 0) as i32;
        }
        1 => {
            let a = m.r32(o + 0xec);
            if a != 0 && m.call(0x0022_7c90, &[a], &[p]).0 == 0 {
                return 0;
            }
            let a = m.r32(o + 0xe8);
            if m.call(0x0022_7c90, &[a], &[p]).0 == 0 {
                return 2;
            }
            f8 = m.r32(o + 0xf8);
        }
        2 => {
            let a = m.r32(o + 0xec);
            if a != 0 && m.call(0x0022_7c90, &[a], &[p]).0 == 0 {
                return 0;
            }
            let a = m.r32(o + 0xe8);
            let v = m.call(0x0022_7c90, &[a], &[p]).0;
            f8 = m.r32(o + 0xf8);
            if v != 0 {
                if f8 & 2 != 0 {
                    return 1;
                }
                if m.call(0x0015_45f0, &[o, tgt], &[]).0 != 0 {
                    return s2;
                }
                return 1;
            }
        }
        _ => return mode,
    }
    if f8 & 8 != 0 && m.call(0x0015_45f0, &[o, tgt], &[]).0 != 0 {
        s2 = 0;
    }
    s2
}

/// FUN_00209ad0(o, mask): entry (+0x4d0 table, 0x20 stride, +0x4c0 count) whose +0x10 mask best matches
/// `mask` (subset popcount via FUN_00209a28) and shares its lowest FUN_00207a20 bit.
/// The 64-bit masks cannot cross `QMem::call` (u32 regs), so the leaf bit helpers are the ported pure fns.
pub fn ee_sub_209ad0<M: QMem + ?Sized>(m: &mut M, o: u32, mask: u64) -> u32 {
    use crate::engine::classes::q::q12_e::{ee_sub_207a20, ee_sub_209a28};
    let mut bit = 0u64;
    let mut i = 0i64;
    while i < 0x30 {
        bit = ee_sub_207a20(i);
        i += 1;
        if mask & bit != 0 {
            break;
        }
    }
    let mut best = 0i32;
    let mut res = 0u32;
    let mut k = 0i32;
    let mut e = o + 0x4d0;
    while k < m.r32(o + 0x4c0) as i32 {
        let em = m.r64(e + 0x10);
        let n = ee_sub_209a28(em, mask);
        if n != 0 && em & bit != 0 && best < n {
            best = n;
            res = e;
        }
        k += 1;
        e += 0x20;
    }
    res
}

/// FUN_00176be8(o, msg, p): AI message handler (msg 0xc = damage/notify packet `p`, else FUN_001ef0b8).
pub fn ee_sub_176be8<M: QMem + ?Sized>(m: &mut M, o: u32, msg: u32, p: u32) -> u32 {
    if msg != 0xc {
        return m.call(0x001e_f0b8, &[o, msg, p], &[]).0;
    }
    let sp = stack_push(m, 0xc0);
    let r = msg_c(m, o, p, sp);
    stack_pop(m, 0xc0);
    r
}

fn post_copy<M: QMem + ?Sized>(m: &mut M, sp: u32, kind: u32, src: u32) {
    m.w32(sp, kind);
    cpn(m, sp + 0x10, src, 0x60);
}

fn msg_c<M: QMem + ?Sized>(m: &mut M, o: u32, p: u32, sp: u32) -> u32 {
    const G7900: u32 = 0x002f_7900;
    let mut s5 = p;
    let a = m.r32(o + 0x10);
    if vslot(m, a, 85, &[p], &[]).0 != 0 {
        let g = m.r32(G7900);
        let v1 = m.r32(p + 0xc);
        if !(g != 0 && m.r32(p) == g && v1 != 0x40000) {
            if g != 0 && m.r32(p) == g && m.r32(g + 0x143c) != 0 {
                let a = m.r32(o + 0x10);
                let pos = m.call(0x001a_62e8, &[a], &[]).1;
                let g = m.r32(G7900);
                m.call(0x001b_9db8, &[g], &[pos]);
            }
            if m.r32(p + 0xc) == 0x40000 {
                let a = m.r32(o + 0x10);
                let s0 = m.r32(0x003e_14bc);
                let pos = m.call(0x001a_62e8, &[a], &[]).1;
                let a = m.r32(o + 0x10);
                m.call(0x0015_6698, &[s0, a], &[pos]);
            }
        }
        m.call(0x0023_33b4, &[sp, 0, 0x70], &[]);
        post_copy(m, sp, 0xb, p);
        m.w32(sp + 0x54, 0x40);
        m.call(0x0017_6750, &[o, sp], &[]);
        return 1;
    }
    let a0 = m.r32(o + 0x10);
    let k = m.r32(a0 + 0xfbc) as i32;
    if k == 0xd || k == 8 {
        let v = m.r32(p + 0x10) << 2;
        m.w32(p + 0x10, v);
    } else if k == 0xe {
        m.call(0x0020_7030, &[a0, u32::MAX], &[]);
    }
    let a = m.r32(o + 0x10);
    if m.r32(a + 0x29c) != 0 {
        m.w32(p + 0x10, 0);
    }
    let a0 = m.r32(o + 0x10);
    let st = m.r32(o + 0x30) as i32;
    if m.r32(a0 + 0x4a4) != 0 && st != 0xb && st != 5 {
        let t = m.r32(p + 0xc);
        if t == 0x4000 || t == 0x2000 {
            m.call(0x001f_7a68, &[a0, 0], &[]);
            m.call(0x0023_33b4, &[sp, 0, 0x70], &[]);
            m.w32(sp, 0x13);
            m.call(0x0017_6750, &[o, sp], &[]);
            return 1;
        }
    }
    match st {
        0xa => {
            if m.r32(p + 0xc) & m.r32(a0 + 0x354) == 0 {
                return 1;
            }
        }
        9 => {
            let t = m.r32(p + 0xc);
            if m.call(0x001f_78d8, &[a0, t], &[]).0 & 0x40 == 0 {
                let a = m.r32(o + 0x10);
                let mk = m.r32(a + 0x350);
                if m.r32(p + 0xc) & mk == 0 {
                    return 1;
                }
                let b = m.r32(o + 0x18);
                if m.r32(b + 0x5c) & mk != 0 {
                    return 1;
                }
            }
        }
        0xb => {
            let mk = m.r32(a0 + 0x358);
            if m.r32(p + 0xc) & mk == 0 {
                return 1;
            }
            let b = m.r32(o + 0x18);
            if m.r32(b + 0x5c) & mk != 0 {
                return 1;
            }
        }
        _ => {}
    }
    let a0 = m.r32(o + 0x10);
    if m.r32(a0 + 0x38c) == m.r32(p) {
        let a = m.r32(a0 + 0x3cc);
        m.call(0x0020_a9a8, &[a, 5], &[]);
    }
    let v1 = m.r32(p + 0xc);
    let s0 = m.r32(p + 0x44);
    'tail: {
        if v1 == 0x80000 && s0 & 8 != 0 {
            if m.r32(o + 0x30) == 8 {
                let b = m.r32(o + 0x18);
                m.call(0x0017_a5b0, &[b, 0], &[]);
                let f = (m.rf(T_NOW) + m.rf(0x002b_e620)).max(m.rf(b + 0x58));
                m.wf(b + 0x58, f);
                break 'tail;
            }
            let a = m.r32(o + 0x10);
            let v = m.r32(a + 0xe88).wrapping_add(m.r32(p + 0x10));
            m.w32(a + 0xe88, v);
            let a = m.r32(o + 0x10);
            if (m.r32(a + 0xe88) as i32) > 0 {
                let h = m.r32(a + 0xe8c);
                m.call(0x0013_05b8, &[h, a, 1, a + 0xe90], &[f4(0.0), f4(1.0)]);
            } else {
                let v = m.r32(a + 0xe84);
                m.w32(a + 0xe88, v);
                m.call(0x0023_33b4, &[sp, 0, 0x70], &[]);
                m.w32(sp, 8);
                m.call(0x0017_6750, &[o, sp], &[]);
            }
            break 'tail;
        }
        if v1 == 0x200000 && s0 & 8 != 0 {
            let b = m.call(0x0020_7a20, &[6], &[]).0;
            let a = m.r32(o + 0x10);
            let q = m.r32(p);
            if m.call(0x0020_a378, &[a, q, b], &[]).0 != 0 {
                let a2 = m.r32(o + 0x10);
                let v = m.r32(a2 + 0x33c).wrapping_add(m.r32(p + 0x10)) as i32;
                m.w32(a2 + 0x33c, if 4 < v { v as u32 } else { 5 });
                let a = m.r32(o + 0x10);
                let pos = m.call(0x001a_62e8, &[a], &[]).1;
                let h = m.r32(GS + 0x4028);
                let a = m.r32(o + 0x10);
                m.call(0x0015_6698, &[h, a], &[pos]);
                m.call(0x0023_33b4, &[sp, 0, 0x70], &[]);
                let q = m.r32(p);
                m.w32(sp, 7);
                m.w32(sp + 0x10, q);
                m.call(0x0017_6750, &[o, sp], &[]);
                break 'tail;
            }
        }
        let v1 = m.r32(p + 0xc);
        let s0 = m.r32(p + 0x44);
        if (v1 == 0x0100_0000 || v1 == 0x0200_0000) && s0 & 8 != 0 {
            m.call(0x0023_33b4, &[sp, 0, 0x70], &[]);
            let b = m.call(0x0020_7a20, &[0x17], &[]).0;
            let a = m.r32(o + 0x10);
            let q = m.r32(p);
            let v = m.call(0x0020_a298, &[a, q, b], &[]).0;
            post_copy(m, sp, if v == 0 { 9 } else { 0xa }, p);
            m.w32(o + 0x30, u32::MAX);
            m.call(0x0017_6750, &[o, sp], &[]);
            let a = m.r32(o + 0x10);
            m.call(0x0020_7030, &[a, if v1 == 0x0100_0000 { 8 } else { 0x22 }], &[]);
            break 'tail;
        }
        if s0 & 0xf0 == 0 {
            break 'tail;
        }
        m.call(0x0023_33b4, &[sp, 0, 0x70], &[]);
        let mut s1 = s0;
        let s4 = if m.r32(p + 0xc) == 0x40000 { m.r32(0x003e_14bc) } else { m.r32(0x003e_14b0) };
        let a = m.r32(o + 0x10);
        let pos = m.call(0x001a_62e8, &[a], &[]).1;
        let a = m.r32(o + 0x10);
        m.call(0x0015_6698, &[s4, a], &[pos]);
        let st = m.r32(o + 0x30) as i32;
        if st < 0xb && st >= 8 {
            let q = m.r32(p);
            if m.call(0x0013_2b10, &[q, 0x65], &[]).0 != 0 {
                s1 = if s0 & 0xb0 != 0 { s0 | 0x40 } else { s0 };
            }
        }
        let b = m.call(0x0020_7a20, &[0x17], &[]).0;
        let a = m.r32(o + 0x10);
        let q = m.r32(p);
        if m.call(0x0020_a298, &[a, q, b], &[]).0 != 0 {
            s1 |= 0x80;
        }
        if s1 & 0x40 != 0 {
            s1 &= !0xb0;
        }
        if s1 & 0xa0 != 0 {
            s1 &= !0x10;
        }
        if s1 & 0x80 != 0 {
            s1 &= !0x20;
            let t = m.r32(p + 0xc);
            let b = m.r32(o + 0x10);
            let i = m.call(0x001f_78a0, &[t], &[]).0;
            let s4 = m.r32(b.wrapping_add(i << 4) + 0x8d0);
            if s4 == u32::MAX {
                let t = m.r32(p + 0xc);
                let b = m.r32(o + 0x10);
                let i = m.call(0x001f_78a0, &[t], &[]).0;
                if m.r32(b.wrapping_add(i << 4) + 0xa70) != s4 {
                    s1 = (s1 & !0x80) | 0x20;
                }
            }
            let t = m.r32(p + 0xc);
            let b = m.r32(o + 0x10);
            let i = m.call(0x001f_78a0, &[t], &[]).0;
            if m.r32(b.wrapping_add(i << 4) + 0x8d0) == u32::MAX {
                let b = m.call(0x0020_7a20, &[6], &[]).0;
                let a = m.r32(o + 0x10);
                if m.call(0x0020_9ad0, &[a, b], &[]).0 == 0 {
                    s1 = (s1 & !0x80) | 0x20;
                }
            }
        }
        if s1 & 0x10 != 0 {
            let a0 = m.r32(o + 0x10);
            if (m.r32(a0 + 0x340) as i32) <= 0 {
                s1 |= 0x20;
            } else {
                let v = m.r32(a0 + 0x344).wrapping_add(m.r32(p + 0x10));
                m.w32(a0 + 0x344, v);
                let a = m.r32(o + 0x10);
                if (m.r32(a + 0x344) as i32) > 0 {
                    let h = m.r32(a + 0x348);
                    m.call(0x0013_05b8, &[h, a, 1, a + 0x34c], &[f4(0.0), f4(1.0)]);
                    let a = m.r32(o + 0x10);
                    let t = m.rf(T_NOW);
                    m.wf(a + 0x360, t);
                } else {
                    s1 |= 0x20;
                }
            }
        }
        if s1 & 0xa0 != 0 {
            let a = m.r32(o + 0x10);
            let v = m.r32(a + 0x340);
            m.w32(a + 0x344, v);
            let a0 = m.r32(o + 0x10);
            let c = m.r32(a0 + 0x340) as i32;
            let a1 = if c > 0 { c.wrapping_neg() as u32 } else { m.r32(p + 0x10) };
            let nv = m.r32(a0 + 0x33c).wrapping_add(a1) as i32;
            m.w32(a0 + 0x33c, if -1 < nv { nv as u32 } else { 0 });
            let a = m.r32(o + 0x10);
            if (m.r32(a + 0x33c) as i32) > 0 {
                post_copy(m, sp, if s1 & 0x80 != 0 { 0xa } else { 9 }, p);
                s5 = sp + 0x10;
            } else {
                s1 |= 0x40;
            }
        }
        if s1 & 0x40 != 0 {
            post_copy(m, sp, 0xb, s5);
            s5 = sp + 0x10;
        }
        if s1 & 0xe0 == 0 {
            break 'tail;
        }
        let q = m.r32(s5);
        if m.call(0x0013_2b10, &[q, 0x34], &[]).0 != 0 {
            let a = m.r32(o + 0x10);
            let pa = m.call(0x001a_62e8, &[a], &[]).1;
            let q = m.r32(s5);
            let pb = m.call(0x001a_62e8, &[q], &[]).1;
            let q = m.r32(s5);
            let mp = m.call(0x001a_6318, &[q], &[]).0;
            let r0 = m.rv(mp);
            let k = -150.0f32;
            let v1 = [pb[0] + r0[0] * k, pb[1] + r0[1] * k, pb[2] + r0[2] * k, pb[3] + r0[3] * k];
            let mut d = sub(pa, v1);
            let q = m.r32(s5);
            if m.call(0x0013_2b10, &[q, 0x36], &[]).0 != 0 {
                let s0 = m.r32(s5);
                if m.r32(s0 + 0x13d0) == 0xc {
                    let mut l = m.call(0x001a_6fc0, &[0, s0], &[d]).1;
                    l[0] *= 0.25;
                    d = m.call(0x001a_6fc0, &[s0, 0], &[l]).1;
                }
            }
            let u = unit(d);
            m.wv(s5 + 0x30, scl(u, 500.0));
        } else {
            let a = m.r32(o + 0x10);
            if m.r32(a + 0xfbc) == u32::MAX {
                let q = m.r32(s5);
                if m.call(0x0013_2b10, &[q, 0x38], &[]).0 != 0 {
                    let q = m.r32(s5);
                    if m.r32(q + 0xfbc) == 0x22 {
                        let a = m.r32(o + 0x10);
                        m.call(0x0020_7030, &[a, 0x22], &[]);
                    }
                }
            }
        }
        m.w32(o + 0x30, u32::MAX);
        m.call(0x0017_6750, &[o, sp], &[]);
    }
    if m.r32(G7900) != 0 && m.r32(s5 + 0xc) == 0x20000 {
        let a = m.r32(o + 0x10);
        let pos = m.call(0x001a_62e8, &[a], &[]).1;
        let g = m.r32(G7900);
        m.call(0x001b_9d28, &[g], &[pos]);
    }
    1
}

/// FUN_001e6f60(path, flags, outp, outd, outu, outr, pos): closest point on path `path` (+0xe0 kind,
/// +0xe8 count, +0xec params, +0xf4 points) to `pos` in the object's frame (slot 30 matrix);
/// `flags` (a2) optionally projects deltas through FUN_001fa970.
#[allow(clippy::too_many_arguments)]
pub fn ee_sub_1e6f60<M: QMem + ?Sized>(m: &mut M, o: u32, s2: u32, s4: u32, s5: u32, s6: u32, s7: u32, pos: V4) {
    let mp = vslot(m, o, 30, &[], &[]).0;
    let r = [m.rv(mp), m.rv(mp + 16), m.rv(mp + 32), m.rv(mp + 48)];
    let kind = m.r32(o + 0xe0);
    if kind == 0 {
        let pts = m.r32(o + 0xf4);
        let mut prev = xf(&r, m.rv(pts), 0.0, true);
        let mut best = m.rf(0x002a_8138);
        let mut f22 = -1.0f32;
        let mut i = 1i32;
        while i < m.r32(o + 0xe8) as i32 {
            let pts = m.r32(o + 0xf4);
            let mut d6 = sub(pos, prev);
            let cur = xf(&r, m.rv(pts + 16 * i as u32), pos[3], true);
            let mut e3 = sub(cur, prev);
            if s2 != 0 {
                e3 = m.call(0x001f_a970, &[s2], &[e3]).1;
                d6 = m.call(0x001f_a970, &[s2], &[d6]).1;
            }
            let mut t = dot3(d6, e3) / dot3(e3, e3);
            if t < 0.0 {
                t = 0.0;
            } else if 1.0 < t {
                t = 1.0;
            }
            let u = 1.0 - t;
            let q = [
                prev[0] * u + cur[0] * t,
                prev[1] * u + cur[1] * t,
                prev[2] * u + cur[2] * t,
                prev[3] * u + cur[3] * t,
            ];
            let mut rr = sub(pos, q);
            if s2 != 0 {
                rr = m.call(0x001f_a970, &[s2], &[rr]).1;
            }
            let dist = len3(rr);
            if dist < best {
                best = dist;
                let w = m.r32(o + 0xec) + 4 * i as u32;
                f22 = u * m.rf(w - 4) + t * m.rf(w);
                if s4 != 0 {
                    m.wv(s4, q);
                }
                if s5 != 0 {
                    m.wv(s5, unit(sub(cur, prev)));
                }
            }
            prev = cur;
            i += 1;
        }
        if s6 != 0 {
            m.wf(s6, f22);
        }
        if s7 != 0 {
            let v = m.call(0x001e_63c0, &[o], &[f4(f22)]).1[0];
            m.wf(s7, v);
        }
    } else if kind == 1 {
        let mut best = m.rf(0x002a_8138);
        let mut bi = -1i32;
        let mut i = 0i32;
        while i < m.r32(o + 0xe8) as i32 {
            let pts = m.r32(o + 0xf4);
            let c = xf(&r, m.rv(pts + 16 * i as u32), r[0][3], true);
            let mut rr = sub(pos, c);
            if s2 != 0 {
                rr = m.call(0x001f_a970, &[s2], &[rr]).1;
            }
            let dist = len3(rr);
            if dist < best {
                best = dist;
                bi = i;
            }
            i += 1;
        }
        let w = m.r32(o + 0xec).wrapping_add((bi as u32) << 2);
        let f = m.rf(w);
        m.call(0x001e_6ed8, &[o, s2, s4, s5, s6, s7], &[pos, f4(f)]);
    }
}

/// FUN_00164df0(o, st): level/zone state change (+0x54 state, +0xf0 pending), with the 0x2deb28 guard.
pub fn ee_sub_164df0<M: QMem + ?Sized>(m: &mut M, o: u32, st: i32) {
    const BUSY: u32 = 0x002d_eb28;
    let old = m.r32(o + 0x54) as i32;
    if st == old || m.r32(BUSY) != 0 {
        return;
    }
    m.w32(BUSY, 1);
    let sel = if st < 3 {
        if st > 0 {
            2
        } else {
            -1
        }
    } else if st == 3 {
        3
    } else {
        -1
    };
    let sp = stack_push(m, 0x60);
    if sel != -1 {
        m.call(0x001a_e478, &[sp], &[]);
        m.w32(sp + 0x50, 0);
        let v = m.r32(0x003e_0f8c) << 2;
        m.w32(sp + 0x10, v);
        m.w32(sp + 0x14, 0x10);
        let s3 = m.call(0x001a_e378, &[sp + 0x10], &[]).0;
        m.w32(o + 0xf0, st as u32);
        m.call(0x0016_4ce8, &[o, sel as u32, sp + 0x50, s3], &[]);
        let mut i = 0i32;
        while i < m.r32(sp + 0x50) as i32 {
            let a0 = m.r32(s3 + 4 * i as u32);
            let v1 = m.r32(o + 0xf4);
            let a1 = m.r32(o + 0xf8);
            m.w32(sp + 0x10, 0);
            m.w32(sp + 0x14, a0);
            if a0 != 0 {
                let v = m.r32(a0 + 0xf4);
                m.w32(sp + 0x18, v);
                m.w32(a0 + 0xf4, v1);
                let b = m.r32(sp + 0x14);
                let v = m.r32(b + 0xf8);
                m.w32(sp + 0x1c, v);
                m.w32(b + 0xf8, a1);
            }
            let s = m.r32(a0 + 0xf0);
            m.call(0x0016_5a60, &[a0, s], &[]);
            let b = m.r32(sp + 0x10);
            if b != 0 {
                let v = m.r32(sp + 0x18);
                m.w32(b + 0xf0, v);
                let b = m.r32(sp + 0x10);
                let v = m.r32(sp + 0x1c);
                m.w32(b + 0xf4, v);
            }
            let b = m.r32(sp + 0x14);
            if b != 0 {
                let v = m.r32(sp + 0x18);
                m.w32(b + 0xf4, v);
                let b = m.r32(sp + 0x14);
                let v = m.r32(sp + 0x1c);
                m.w32(b + 0xf8, v);
            }
            i += 1;
        }
        let mut i = 0i32;
        while i < m.r32(sp + 0x50) as i32 {
            let b = m.r32(s3 + 4 * i as u32);
            m.w32(b + 0xf0, u32::MAX);
            i += 1;
        }
        m.w32(o + 0xf0, u32::MAX);
        m.call(0x001a_e4a0, &[sp, 2], &[]);
    }
    m.call(0x0016_5a60, &[o, st as u32], &[]);
    m.w32(BUSY, 0);
    let v = m.call(0x0021_0ae0, &[], &[]).0;
    if v < 9 {
        let s3 = m.r32(GS + 0x3b7c + 4 * v);
        if s3 != 0 {
            let r = m.call(0x0016_1660, &[0x002d_eb30], &[]).0;
            m.w32(sp + 0x20, 0x15);
            m.w32(sp + 0x24, r);
            m.call(0x0012_91a8, &[sp + 0x30, sp + 0x20], &[]);
            m.call(0x0025_8200, &[sp + 0x40, s3, sp + 0x30, 0x003d_4a60], &[]);
            m.call(0x0012_9ec8, &[sp + 0x40], &[]);
            m.call(0x0012_9ec8, &[sp + 0x30], &[]);
            m.call(0x0012_9ec8, &[sp + 0x20], &[]);
        }
    }
    stack_pop(m, 0x60);
    if m.call(0x0016_37e8, &[o], &[]).0 == 0 || old == -1 || old >= 2 {
        return;
    }
    if (m.r32(o + 0x54) as i32) < 2 {
        return;
    }
    let a0 = m.r32(o + 0x7c);
    if a0 != 0 {
        m.call(0x0016_2cd0, &[a0, o], &[]);
        if m.r32(o + 0x7c) != 0 {
            return;
        }
    }
    if m.r32(o + 0x94) != 0 {
        let n = m.r32(o + 0xa0) as i32;
        if n > 0 {
            let arr = m.r32(o + 0xa4);
            let mut i = 0i32;
            loop {
                let e = m.r32(arr + 4 * i as u32);
                if m.r32(e + 0x7c) == 0 {
                    return;
                }
                i += 1;
                if i >= n {
                    break;
                }
            }
        }
    }
    m.call(0x001a_bd98, &[0x002e_65e0, 0], &[]);
}

/// FUN_0011bcf8(mat, out_angle, out_axis): rotation matrix -> angle (FUN_002456f0 acos) + unit axis.
pub fn ee_sub_11bcf8<M: QMem + ?Sized>(m: &mut M, s1: u32, s2: u32, s0: u32) {
    let c = (m.rf(s1) + m.rf(s1 + 0x14) + m.rf(s1 + 0x28) - 1.0) * 0.5;
    if f32::from_bits(0x3f7f_ffef) < c {
        m.w32(s2, 0);
        cpn(m, s0, 0x002a_2da0, 16);
        return;
    }
    let mut f2 = 1.0f32;
    let f3 = f2 - c;
    for k in 0..2 {
        let x = (m.rf(s1 + 0x14 * k) - c) / f3;
        let mut f1 = x.abs();
        if f2 < f1 {
            f1 = f2;
        } else if f1 < 0.0 {
            f1 = 0.0;
        }
        m.wf(s0 + 4 * k, f1.sqrt());
        f2 -= f1;
    }
    m.wf(s0 + 8, f2.sqrt());
    if c < f32::from_bits(0xbf7f_f972) {
        m.wf(s2, f32::from_bits(0x4049_0fdb));
        let x = m.rf(s0);
        let y = m.rf(s0 + 4);
        let z = m.rf(s0 + 8);
        if y < x && z < x {
            if m.rf(s1 + 4) < 0.0 {
                m.wf(s0 + 4, -y);
            }
            if m.rf(s1 + 8) < 0.0 {
                let v = m.rf(s0 + 8);
                m.wf(s0 + 8, -v);
            }
        } else if !(y < x) && z < y {
            if m.rf(s1 + 0x10) < 0.0 {
                m.wf(s0, -x);
            }
            if m.rf(s1 + 0x18) < 0.0 {
                let v = m.rf(s0 + 8);
                m.wf(s0 + 8, -v);
            }
        } else {
            if m.rf(s1 + 0x20) < 0.0 {
                m.wf(s0, -x);
            }
            if m.rf(s1 + 0x24) < 0.0 {
                let v = m.rf(s0 + 4);
                m.wf(s0 + 4, -v);
            }
        }
        return;
    }
    let a = m.call(0x0024_56f0, &[], &[f4(c)]).1[0];
    m.wf(s2, a);
    if m.rf(s1 + 4) < m.rf(s1 + 0x10) {
        let v = m.rf(s0 + 8);
        m.wf(s0 + 8, -v);
    }
    if m.rf(s1 + 0x20) < m.rf(s1 + 8) {
        let v = m.rf(s0 + 4);
        m.wf(s0 + 4, -v);
    }
    if m.rf(s1 + 0x18) < m.rf(s1 + 0x24) {
        let v = m.rf(s0);
        m.wf(s0, -v);
    }
}

/// FUN_00194f70(o, ctx, n, items): merge `n` sorted 0x30-byte items into the live list (+0x4680, count
/// +0x4678): matched entries are tracked/smoothed and removed from `items`, unmatched live entries are
/// faded (FUN_001f7bf8) and dropped, remaining items are appended (FUN_001956d0 / FUN_001619a0).
pub fn ee_sub_194f70<M: QMem + ?Sized>(m: &mut M, s4: u32, ctx: u32, n: i32, s6: u32) {
    const CMP: u32 = 0x0019_4f48;
    let sp = stack_push(m, 0x30);
    let mut s5 = n;
    m.w32(sp + 0x20, ctx);
    if s5 > 0 {
        m.call(0x0023_38c0, &[s6, s5 as u32, 0x30, CMP], &[]);
        let a = m.r32(s6 + 4);
        if m.r32(a + 0x14) != u32::MAX {
            m.w32(s6 + 0xc, 1);
        }
    }
    let mut s1 = m.r32(s4 + 0x4678) as i32 - 1;
    m.w32(sp + 0x24, s4 + 0x4680);
    while s1 >= 0 {
        let s0 = s4 + 0x4680 + (s1 as u32) * 0x30;
        let fp = s1 - 1;
        let mut s3 = s5 - 1;
        let mut matched = false;
        while s3 >= 0 {
            let s2 = s6 + (s3 as u32) * 0x30;
            if m.r32(s0) != m.r32(s2) {
                s3 -= 1;
                continue;
            }
            matched = true;
            let v = m.rf(s2 + 8);
            m.wf(s0 + 8, v);
            if m.r32(s0 + 0x10) == 0 {
                let v = m.rf(s2 + 0x18);
                m.w32(s0 + 0x1c, 0);
                m.wf(s0 + 0x18, v);
            } else {
                let a2 = m.r32(s2 + 4);
                let mut wrap = vslot20(m, a2, 0, &[sp], &[]).0;
                let mut f20 = 0.0f32;
                if wrap != 0 {
                    f20 = m.rf(sp + 4) - m.rf(sp);
                    let f1 = m.rf(s0 + 0x18);
                    let mut f2 = m.rf(s2 + 0x18) - f1;
                    if f20 * 0.5 < f2 {
                        f2 -= f20;
                        m.wf(s2 + 0x18, f1 + f2);
                    } else if f2 < f20 * -0.5 {
                        f2 += f20;
                        m.wf(s2 + 0x18, f1 + f2);
                    } else {
                        wrap = 0;
                    }
                }
                if m.r32(0x002e_1e50) != 0 {
                    let f2 = m.rf(s0 + 0x1c);
                    let f6 = m.rf(s0 + 0x18);
                    let f0 = (m.rf(s2 + 0x18) - f6) * m.rf(0x002b_f710);
                    let f1 = (m.rf(s2 + 0x1c) - f2) * m.rf(0x002b_f714);
                    let f3 = m.rf(T_DT);
                    let nv = f2 + f3 * (f0 + f1);
                    m.wf(s0 + 0x1c, nv);
                    m.wf(s0 + 0x18, f6 + f3 * ((f2 + nv) * 0.5));
                } else {
                    let a = m.rf(s0 + 0x18);
                    let b = m.rf(s2 + 0x18);
                    let dt = m.rf(T_DT);
                    let r = m.call(0x0011_d018, &[0x002b_f718, s0 + 0x1c], &[f4(a), f4(b), f4(dt)]).1[0];
                    m.wf(s0 + 0x18, r);
                }
                if wrap != 0 {
                    let f1 = m.rf(s0 + 0x18);
                    if m.rf(sp + 4) < f1 {
                        m.wf(s0 + 0x18, f1 - f20);
                    } else if f1 < m.rf(sp) {
                        m.wf(s0 + 0x18, f1 + f20);
                    }
                }
                let t0 = m.r32(s0 + 4);
                let a1 = m.r32(sp + 0x20);
                let f = m.rf(s0 + 0x18);
                vslot20(m, t0, 2, &[a1, sp + 0x10, 0], &[f4(f)]);
                let a2 = m.r32(s0 + 0x10);
                let v = m.rv(sp + 0x10);
                vslot(m, a2, 28, &[], &[v]);
            }
            let cnt = ((s5 - s3) as u32).wrapping_mul(0x30).wrapping_sub(0x30);
            s5 -= 1;
            m.call(0x0023_32b8, &[s2, s2 + 0x30, cnt], &[]);
            break;
        }
        if !matched {
            let a0 = m.r32(s0 + 0x10);
            if a0 != 0 {
                m.call(0x001f_7bf8, &[a0], &[f4(0.5)]);
            }
            let cnt = ((3 - s1) as u32).wrapping_mul(0x30);
            m.call(0x0023_32b8, &[s0, s0 + 0x30, cnt], &[]);
            let v = m.r32(s4 + 0x4678).wrapping_sub(1);
            m.w32(s4 + 0x4678, v);
        }
        s1 = fp;
    }
    let mut s0 = s6;
    let mut s3 = 0i32;
    while s3 < s5 {
        let c = m.r32(sp + 0x20);
        let a1 = m.r32(c + 0x9c4);
        let s2 = s0;
        if a1 != 0 && m.r32(s2 + 0xc) == 1 {
            let a2 = m.call(0x0019_56d0, &[s4, a1, s2], &[]).0;
            if a2 == 0 {
                break;
            }
            m.w32(s2 + 0x10, a2);
        }
        let a3 = m.r32(s4 + 0x4678);
        let dst = s4 + 0x4680 + a3.wrapping_mul(0x30);
        cpn(m, dst, s0, 0x30);
        s3 += 1;
        m.w32(s4 + 0x4678, a3.wrapping_add(1));
        let id = m.r32(s0);
        m.call(0x0016_19a0, &[0x002d_eb30, 0x3d, id], &[]);
        s0 += 0x30;
    }
    let a0 = m.r32(sp + 0x24);
    let c = m.r32(s4 + 0x4678);
    m.call(0x0023_38c0, &[a0, c, 0x30, CMP], &[]);
    stack_pop(m, 0x30);
}

/// FUN_0022c028(o): per-frame rotor/UV update of the +0x60 entries (0xd0 stride, +0x54 count):
/// kind 0 = spinning 2x2 basis (FUN_0011cf60 wrap + FUN_0011d608 sincos), kind 1 = two-axis rotation
/// (FUN_0022bf70 angles, FUN_0011b500) written as (R - I) into the +0x6f0 matrices; +0x50 last time.
pub fn ee_sub_22c028<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001a_5e28, &[o], &[]);
    if m.rf(T_NOW) == m.rf(o + 0x50) {
        return;
    }
    if m.r32(o + 0x8fc) != 0 {
        m.call(0x0022_b7d8, &[o], &[]);
    }
    let sp = stack_push(m, 0xb0);
    let s5 = sp + 0x70;
    let mut i = 0i32;
    while i < m.r32(o + 0x54) as i32 {
        let s0 = o + 0x60 + (i as u32) * 0xd0;
        let k = m.r32(s0 + 4) << 6;
        let s2 = o.wrapping_add(k + 0x7f0);
        let s1 = o.wrapping_add(k + 0x6f0);
        let kind = m.r32(s0);
        if kind == 0 {
            let f = (m.rf(T_NOW) * m.rf(s0 + 0x68) + m.rf(s0 + 0x6c)) * f32::from_bits(0x40c9_0fdb);
            let f = m.call(0x0011_cf60, &[], &[f4(f)]).1[0];
            m.call(0x0011_d608, &[sp + 0xa0, sp + 0xa4], &[f4(f)]);
            let s = m.rf(sp + 0xa0);
            let c = m.rf(sp + 0xa4);
            let a = m.rv(s0 + 0x20);
            let b = m.rv(s0 + 0x40);
            let cc = m.rv(s0 + 0x30);
            let d = m.rv(s0 + 0x50);
            let idx = m.r32(s0 + 0x60);
            let lin = |x: V4, p: f32, y: V4, q: f32| [x[0] * p + y[0] * q, x[1] * p + y[1] * q, x[2] * p + y[2] * q];
            let r1 = lin(a, c, b, s);
            let r2 = lin(a, -s, b, c);
            let r3 = lin(cc, c, d, s);
            let r4 = lin(cc, -s, d, c);
            let p1 = s1.wrapping_add(idx << 4);
            let p2 = s1.wrapping_add(idx.wrapping_add(1) << 4);
            let q1 = s2.wrapping_add(idx << 4);
            let q2 = s2.wrapping_add(idx.wrapping_add(1) << 4);
            m.wf(p1, r1[0]);
            m.wf(p1 + 8, r1[2]);
            m.wf(p1 + 4, r1[1]);
            m.wf(p2, r2[0]);
            m.wf(p2 + 8, r2[2]);
            m.wf(p2 + 4, r2[1]);
            m.wf(q1, r3[0]);
            m.wf(q1 + 4, r3[1]);
            m.wf(q2, r4[0]);
            m.wf(q2 + 4, r4[1]);
        } else if kind == 1 {
            let f20 = m.call(0x0022_bf70, &[s0 + 0x30], &[]).1[0];
            let f12 = m.call(0x0022_bf70, &[s0 + 0x80], &[]).1[0];
            let u = unit(m.rv(s0 + 0x20));
            m.call(0x0011_b500, &[s5], &[u, f4(f12)]);
            let v = unit(m.rv(s0 + 0x10));
            let rr = [m.rv(sp + 0x70), m.rv(sp + 0x80), m.rv(sp + 0x90), [0.0; 4]];
            let w = xf(&rr, v, v[3], false);
            let n = unit(w);
            m.call(0x0011_b500, &[s5], &[n, f4(f20)]);
            let r0 = m.rv(sp + 0x70);
            let r1 = m.rv(sp + 0x80);
            let r2 = m.rv(sp + 0x90);
            m.wf(s1 + 0x28, r2[2] - 1.0);
            m.wf(s1, r0[0] - 1.0);
            m.wf(s1 + 4, r0[1]);
            m.wf(s1 + 8, r0[2]);
            m.wf(s1 + 0x10, r1[0]);
            m.wf(s1 + 0x14, r1[1] - 1.0);
            m.wf(s1 + 0x18, r1[2]);
            m.wf(s1 + 0x20, r2[0]);
            m.wf(s1 + 0x24, r2[1]);
        }
        i += 1;
    }
    stack_pop(m, 0xb0);
    let t = m.rf(T_NOW);
    m.wf(o + 0x50, t);
}
