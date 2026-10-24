//! WIRE-S2: 1:1 QMem ports of live-frontier natives (shard 3/4).
#![allow(unused)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

const T: u32 = 0x002d_ded0; // g_clock t
const CM: u32 = 0x002d_df40; // camera manager
const GS: u32 = 0x003d_d490; // game state / world

fn dist_xy(a: V4, b: V4) -> f32 {
    let x = a[0] - b[0];
    let y = a[1] - b[1];
    (x * x + y * y).sqrt()
}

/// FUN_0015ad48(o, a1 *quad, a2 *quad): mat = vslot 31(**(o+8), &buf) frame; FUN_0011c168(&m, +0x40, &buf);
/// *a1 = FUN_001a72d0(&m, 0, *a1); mode(+0x18) != 3 -> *a2 = FUN_001a7348(&m, 0, *a2); mode != 2 ->
/// *a2 += FUN_0022d348(frame, 0, mode == 1 ? *a1 : frame[+0x70]).
pub fn ee_sub_15ad48<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    let p = m.r32(a0 + 8);
    let obj = m.r32(p);
    if obj == 0 {
        return;
    }
    let s4 = a0 + 0x10;
    let sp = stack_push(m, 0x80);
    let s1 = vslot(m, obj, 31, &[sp], &[]).0;
    let s0 = sp + 0x40;
    m.call(0x0011_c168, &[s0, s1 + 0x40, sp], &[]);
    let q = m.rv(a1);
    let r = m.call(0x001a_72d0, &[s0, 0], &[q]).1;
    m.wv(a1, r);
    if m.r32(s4 + 8) != 3 {
        let q = m.rv(a2);
        let r = m.call(0x001a_7348, &[s0, 0], &[q]).1;
        m.wv(a2, r);
        let md = m.r32(s4 + 8);
        if md != 2 {
            let q = if md != 1 { m.rv(s1 + 0x70) } else { m.rv(a1) };
            let r = m.call(0x0022_d348, &[s1, 0], &[q]).1;
            let c = m.rv(a2);
            m.wv(a2, [c[0] + r[0], c[1] + r[1], c[2] + r[2], c[3] + r[3]]);
        }
    }
    stack_pop(m, 0x80);
}

/// FUN_00172658(o): target-in-range test. a = +0x10; !(a[+0xfe8] & 2) || a[+0xe78] == 2 -> 1; b = a[+0xe74]
/// null -> 0; f20 = |pos(b) - pos(a)|.xy; 1 if |FUN_00172760(o) - pos(+0x10)|.xy < f20 + 50.
pub fn ee_sub_172658<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let a = m.r32(a0 + 0x10);
    if m.r32(a + 0xfe8) & 2 == 0 {
        return 1;
    }
    if m.r32(a + 0xe78) == 2 {
        return 1;
    }
    let b = m.r32(a + 0xe74);
    if b == 0 {
        return 0;
    }
    let p1 = m.call(0x001a_62e8, &[b], &[]).1;
    let a = m.r32(a0 + 0x10);
    let p2 = m.call(0x001a_62e8, &[a], &[]).1;
    let f20 = dist_xy(p1, p2);
    let q = m.call(0x0017_2760, &[a0], &[]).1;
    let a = m.r32(a0 + 0x10);
    let p3 = m.call(0x001a_62e8, &[a], &[]).1;
    let f0 = dist_xy(q, p3);
    (f0 < f20 + 50.0) as u32
}

/// FUN_001285d8(o, e): e[0] 0x1f/0x20/0x22: sync the sign bit of e[1] (or *(e[1])[+4]) with DAT_002dd598;
/// 0x1f / 0x20 on change append to the +0xc/+0x10 resp. +0x14/+0x18 list.
pub fn ee_sub_1285d8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let k = m.r32(a1);
    let g = 0x002d_d598;
    match k {
        0x1f | 0x20 => {
            let e = if k == 0x1f { a1 } else { m.r32(a1 + 4) };
            let v = m.r32(g);
            let w = m.r32(e + 4);
            if w >> 31 == v {
                return;
            }
            m.w32(e + 4, (w & 0x7fff_ffff) | (v << 31));
            let (cn, ar) = if k == 0x1f { (0xc, 0x10) } else { (0x14, 0x18) };
            let n = m.r32(a0 + cn);
            let base = m.r32(a0 + ar);
            m.w32((n << 2).wrapping_add(base), e);
            m.w32(a0 + cn, n.wrapping_add(1));
        }
        0x22 => {
            let e = m.r32(a1 + 4);
            let v = m.r32(g);
            let w = m.r32(e + 4);
            m.w32(e + 4, (w & 0x7fff_ffff) | (v << 31));
        }
        _ => {}
    }
}

/// FUN_0019de58(o): for the two +0x50 slots: FUN_0012ff10(slot[+4], o[0], 1, 1) -> sample its curve at 0
/// and slot[+4][+0x74] (vfn +0x1c of *(+4)); slot[+0xc] = (a.z - b.z) / len; then FUN_00137a30(o[0], 0x10, 2,
/// o + 0x50, 0, 0, 0, o + 0x70).
pub fn ee_sub_19de58<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let sp = stack_push(m, 0x20);
    let mut s2 = a0 + 0x50;
    for _ in 0..2 {
        let s1 = m.r32(s2 + 4);
        if s1 != 0 {
            let k = m.r32(a0);
            let s0 = m.call(0x0012_ff10, &[s1, k, 1, 1], &[]).0;
            if s0 != 0 {
                for (buf, f12) in [(sp, None), (sp + 0x10, Some(()))] {
                    let a3 = m.r32(s0 + 4);
                    let vt = m.r32(a3);
                    let adj = m.r16(vt + 0x18) as i16 as i32 as u32;
                    let f = m.r32(vt + 0x1c);
                    let t = if f12.is_some() { m.rf(s1 + 0x74) } else { 0.0 };
                    m.call(f, &[a3.wrapping_add(adj), buf, 0], &[f4(t), f4(1.0)]);
                }
                let r = (m.rf(sp + 8) - m.rf(sp + 0x18)) / m.rf(s1 + 0x74);
                m.wf(s2 + 0xc, r);
            }
        }
        s2 += 0x10;
    }
    let k = m.r32(a0);
    m.call(0x0013_7a30, &[k, 0x10, 2, a0 + 0x50, 0, 0, 0, a0 + 0x70], &[]);
    stack_pop(m, 0x20);
}

/// FUN_002020c8(o): for each of +0x30 entries (+0x34[i]) with +0x100: d = |pos - cm[+0x90]|.xy; d > +0x104
/// and flag bit 60 of +0x100[+0x28] clear -> FUN_001f7c68(+0x100, 0.25); d <= +0x104 and bit set ->
/// FUN_001f7bf8(+0x100, 0.25).
pub fn ee_sub_2020c8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let mut i: i32 = 0;
    while i < m.r32(a0 + 0x30) as i32 {
        let s0 = m.r32(a0 + 0x34 + (i as u32) * 4);
        if m.r32(s0 + 0x100) != 0 {
            let p = m.call(0x001a_62e8, &[s0], &[]).1;
            let r = m.rf(s0 + 0x104);
            let c = m.rv(CM + 0x90);
            let d = dist_xy(p, c);
            if r < d {
                let o = m.r32(s0 + 0x100);
                if (m.r64(o + 0x28) >> 60) & 1 == 0 {
                    m.call(0x001f_7c68, &[o], &[f4(0.25)]);
                }
            } else if d <= r {
                let o = m.r32(s0 + 0x100);
                if (m.r64(o + 0x28) >> 60) & 1 != 0 {
                    m.call(0x001f_7bf8, &[o], &[f4(0.25)]);
                }
            }
        }
        i += 1;
    }
}

/// FUN_00137fb8(stride, n, base, off, cnt, keys): for each of n records (base + i*stride) with curve r[+4]:
/// out = r + off; out[j].x = FUN_0012f990(curve, keys[j]); then out[j-1 cyclic].y = out[j].x - prev.x
/// (+ curve[+0x74] if negative).
pub fn ee_sub_137fb8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, t0: u32, t1: u32) {
    let mut i: u32 = 0;
    while (i as i32) < a1 as i32 {
        let v0 = a2.wrapping_add(i.wrapping_mul(a0));
        let s3 = m.r32(v0 + 4);
        if s3 != 0 {
            let base = v0.wrapping_add(a3);
            if t0 as i32 > 0 {
                let mut s1 = base;
                let mut s2 = t1;
                for _ in 0..t0 {
                    let k = m.r32(s2);
                    s2 += 4;
                    let f = m.call(0x0012_f990, &[s3, k], &[]).1[0];
                    m.wf(s1, f);
                    s1 += 8;
                }
            }
            let mut prev = base.wrapping_add((t0 << 3).wrapping_sub(8));
            if t0 as i32 > 0 {
                let mut s1 = base;
                for _ in 0..t0 {
                    let f1 = m.rf(s1) - m.rf(prev);
                    m.wf(prev + 4, f1);
                    if f1 < 0.0 {
                        let l = m.rf(s3 + 0x74);
                        m.wf(prev + 4, f1 + l);
                    }
                    prev = s1;
                    s1 += 8;
                }
            }
        }
        i += 1;
    }
}

/// FUN_001c4468(o): +0x80 == 0 -> FUN_001c42e8(o); == 1 -> load tuning block (DAT_002dfbc0 & 2 ? 0x2dfc00 :
/// 0x2e0d00) into +0x10/+0x14/+0x20/+0x24/+0x30, FUN_0011bc88(0x2ddfa0, &a, &b, mask, +0x7c); FUN_0011b720(&mat,
/// a, b) -> +0x40..+0x70; FUN_001c4330 for both axes; FUN_001c45a8(o).
pub fn ee_sub_1c4468<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let sp = stack_push(m, 0x40);
    m.wf(sp + 0x30, 0.0);
    m.wf(sp + 0x34, 0.0);
    let v1 = m.r32(a0 + 0x80);
    if v1 == 0 {
        m.call(0x001c_42e8, &[a0], &[]);
    } else if v1 == 1 {
        let b: u32 = if m.r32(0x002d_fbc0) & 2 != 0 { 0x002d_fc00 } else { 0x002e_0d00 };
        let f = m.rf(b + 0x103c);
        let t0 = m.r32(a0 + 0x7c);
        m.wf(a0 + 0x10, f);
        let f = m.rf(b + 0x1040);
        m.wf(a0 + 0x14, f);
        let f = m.rf(b + 0x1070);
        m.wf(a0 + 0x20, f);
        let f = m.rf(b + 0x1074);
        m.wf(a0 + 0x24, f);
        let msk = m.r32(b + 0x10bc);
        let v = m.r32(b + 0x10b4);
        let a3 = m.r32(t0 + 0x154);
        m.w32(a0 + 0x30, (v & !msk) | a3);
        m.call(0x0011_bc88, &[0x002d_dfa0, sp + 0x30, sp + 0x34, a3, t0], &[]);
    }
    let f12 = m.rf(sp + 0x30);
    let f13 = m.rf(sp + 0x34);
    m.call(0x0011_b720, &[sp], &[f4(f12), f4(f13)]);
    for k in 0..3 {
        let q = m.rv(sp + k * 0x10);
        m.wv(a0 + 0x40 + k * 0x10, q);
    }
    let f12 = m.rf(sp + 0x30);
    m.call(0x001c_4330, &[a0, a0 + 0x10, a0 + 0x14, a0 + 0x18, a0 + 0x1c, a0 + 0x70], &[f4(f12)]);
    let f12 = m.rf(sp + 0x30);
    m.call(0x001c_4330, &[a0, a0 + 0x20, a0 + 0x24, a0 + 0x28, a0 + 0x2c, a0 + 0x74], &[f4(f12)]);
    m.call(0x001c_45a8, &[a0], &[]);
    stack_pop(m, 0x40);
}

/// FUN_001da5c8(o, s): state change (+0x10) to s. 3 -> 1 at same tick becomes 2; leaving 4 restores +0x18/+0x1c
/// from +0x20/+0x24; s == 0 -> +0x18 = -1; with +0x28: 4 -> 2 FUN_00213830(+0x28, 0x310); s 3 -> vslot 49;
/// 1 -> FUN_00212880(+0x28, 1, o + 0x18); 4 -> FUN_00213830(+0x28, 0x312); +0x10 = s, +0x14 = t.
pub fn ee_sub_1da5c8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let mut s1 = a1;
    let v1 = m.r32(a0 + 0x10);
    if s1 == v1 {
        return;
    }
    if v1 == 3 {
        if s1 == 1 && m.rf(a0 + 0x14) == m.rf(T) {
            s1 = 2;
        }
    } else if v1 == 4 {
        let v = m.r32(a0 + 0x20);
        let f = m.rf(a0 + 0x24);
        m.w32(a0 + 0x18, v);
        m.wf(a0 + 0x1c, f);
        m.w32(a0 + 0x20, u32::MAX);
        m.wf(a0 + 0x24, -1.0);
    }
    if s1 == 0 {
        m.w32(a0 + 0x18, u32::MAX);
    }
    let a = m.r32(a0 + 0x28);
    if a != 0 {
        if m.r32(a0 + 0x10) == 4 && s1 == 2 {
            m.call(0x0021_3830, &[a, 0x310], &[]);
        }
        if s1 == 3 {
            let o = m.r32(a0 + 0x28);
            vslot(m, o, 49, &[], &[]);
        } else if (s1 as i32) < 4 {
            if s1 == 1 {
                let o = m.r32(a0 + 0x28);
                m.call(0x0021_2880, &[o, 1, a0 + 0x18], &[]);
            }
        } else if s1 == 4 {
            let o = m.r32(a0 + 0x28);
            m.call(0x0021_3830, &[o, 0x312], &[]);
        }
    }
    m.w32(a0 + 0x10, s1);
    let t = m.rf(T);
    m.wf(a0 + 0x14, t);
}

/// FUN_00131100(o): event-track scrub. track +0x14 with events (+0xa0 count, +0xa4 16-byte entries); vslot 0;
/// +0x20 = +0x1c; until FUN_00131968(o): step cursor +0x50 forward/backward (by +0x40 sign) across events
/// whose time passes +0x20, firing FUN_00131290(o) for kinds (7 + table bits 0x3c3d3); else +0x1c = +0x20;
/// vslot 1.
pub fn ee_sub_131100<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let tr = m.r32(a0 + 0x14);
    if m.r32(tr + 0xa0) == 0 {
        return;
    }
    vslot(m, a0, 0, &[], &[]);
    let f = m.rf(a0 + 0x1c);
    m.wf(a0 + 0x20, f);
    loop {
        if m.call(0x0013_1968, &[a0], &[]).0 != 0 {
            break;
        }
        let v1 = m.r32(a0 + 0x14);
        let a = m.r32(a0 + 0x50);
        let f2 = m.rf(a0 + 0x20);
        let mut ev: Option<u32> = None;
        if (a as i32) < m.r32(v1 + 0xa0) as i32 {
            let e = (a << 4).wrapping_add(m.r32(v1 + 0xa4));
            if m.rf(e) <= f2 && 0.0 <= m.rf(a0 + 0x40) {
                m.w32(a0 + 0x50, a.wrapping_add(1));
                ev = Some(e);
            }
        }
        if ev.is_none() {
            if a as i32 > 0 {
                let e = (a << 4).wrapping_add(m.r32(v1 + 0xa4));
                if !(f2 <= m.rf(e.wrapping_sub(0x10))) {
                    m.wf(a0 + 0x1c, f2);
                    break;
                }
                if m.rf(a0 + 0x40) < 0.0 {
                    let n = a.wrapping_sub(1);
                    m.w32(a0 + 0x50, n);
                    ev = Some(m.r32(v1 + 0xa4).wrapping_add(n << 4));
                }
            }
        }
        let e = match ev {
            Some(e) => e,
            None => {
                m.wf(a0 + 0x1c, f2);
                break;
            }
        };
        let idx = m.r32(e + 4).wrapping_sub(7);
        if idx < 0x12 && (0x3c3d3u32 >> idx) & 1 != 0 {
            m.call(0x0013_1290, &[a0], &[]);
        }
    }
    vslot(m, a0, 1, &[], &[]);
}

/// FUN_00210cf0(on, id): id == 0xffff only. on == 0 -> FUN_00258348(GS, 2); else FUN_0020d9d0(GS), reset
/// the 0x3e1c20 header, and bump the six pool refcounts (0x2dd2e0..0x2dd2f4), FUN_0011cca8-initialising each
/// pool on its first reference.
pub fn ee_sub_210cf0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if a1 != 0xffff {
        return;
    }
    if a0 == 0 {
        m.call(0x0025_8348, &[GS, 2], &[]);
        return;
    }
    m.call(0x0020_d9d0, &[GS], &[]);
    let h = 0x003e_1c20;
    m.w32(h, 0);
    m.w32(h + 0xc, 0x003e_1c60);
    m.w32(h + 4, 0x002c_3eb8);
    let pools: [(u32, u32, u32, u32); 6] = [
        (0x002d_d2e0, 0x0045_20a8, 0x14, 0x2000),
        (0x002d_d2e4, 0x0045_20d0, 0x18, 0x800),
        (0x002d_d2e8, 0x0045_20f8, 0x28, 0x200),
        (0x002d_d2ec, 0x0045_2120, 0x1c, 0x800),
        (0x002d_d2f0, 0x0045_2148, 0xc, 0xc8),
        (0x002d_d2f4, 0x0045_2170, 0x14, 0x100),
    ];
    for (k, (cnt, pool, sz, n)) in pools.into_iter().enumerate() {
        let c = m.r32(cnt).wrapping_add(1);
        m.w32(cnt, c);
        if k == 0 {
            m.w32(h + 8, 0);
        }
        if c == 1 {
            m.call(0x0011_cca8, &[pool, sz, n, 0x10], &[]);
        }
    }
}
