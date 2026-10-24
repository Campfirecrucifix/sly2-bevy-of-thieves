//! WIRE-ROOTS: 1:1 QMem queue ports of top-ranked unwired roots (tools/wire_roots.py), ported from the EE
//! code; callees go through `m.call` / `vslot`. Integer / pointer args in `a`, floats in `v`.
#![allow(unused_variables, unused_mut, clippy::all)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem, V4};

/// shared body of the Ref value ctors: FUN_00129ec8(o); o[0] = tag; box = FUN_0011ccc0(pool); box+rc = 1;
/// o[1] = box; copy `n` words of `src` into the box.
fn ref_box<M: QMem + ?Sized>(m: &mut M, o: u32, src: u32, tag: u32, pool: u32, rc: u32, n: u32) -> u32 {
    m.call(0x0012_9ec8, &[o], &[]);
    m.w32(o, tag);
    let b = m.call(0x0011_ccc0, &[pool], &[]).0;
    m.w32(b + rc, 0);
    m.w32(o + 4, b);
    let r = m.r32(b + rc);
    m.w32(b + rc, r.wrapping_add(1));
    for i in 0..n {
        let w = m.r32(src + 4 * i);
        m.w32(b + 4 * i, w);
    }
    o
}

/// FUN_00129808(o, src): Ref tag 0x1c, 16-byte box from pool 0x4521b8 (refcount at +0x10).
pub fn ee_sub_129808<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    ref_box(m, a0, a1, 0x1c, 0x0045_21b8, 0x10, 4)
}

/// FUN_00129878(o, src): Ref tag 0x1d, 8-byte box from pool 0x4521d0 (refcount at +8).
pub fn ee_sub_129878<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    ref_box(m, a0, a1, 0x1d, 0x0045_21d0, 8, 2)
}

/// FUN_001298f0(o, src): Ref tag 0x1e, 12-byte box from pool 0x4521e8 (refcount at +0xc).
pub fn ee_sub_1298f0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    ref_box(m, a0, a1, 0x1e, 0x0045_21e8, 0xc, 3)
}

/// FUN_00181a90(o, group): FUN_00181b60; L = FUN_001e6648(g+0x24); for each rider r (list g+0x10, next +0x60):
/// r+0x24 = -1; FUN_00181eb0(o, r, 0); r+0x40 = 0; r+0x3c = L (i + 0.5) / g+0xc; r+0x4c = g+0x1c.
pub fn ee_sub_181a90<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.call(0x0018_1b60, &[a0, a1], &[]);
    let p = m.r32(a1 + 0x24);
    let l = m.call(0x001e_6648, &[p], &[]).1[0];
    let mut r = m.r32(a1 + 0x10);
    let mut i = 0i32;
    while r != 0 {
        m.w32(r + 0x24, 0xffff_ffff);
        m.call(0x0018_1eb0, &[a0, r, 0], &[]);
        let n = m.r32(a1 + 0xc) as i32;
        m.w32(r + 0x40, 0);
        m.wf(r + 0x3c, (l * (i as f32 + 0.5)) / n as f32);
        let hp = m.r32(a1 + 0x1c);
        m.w32(r + 0x4c, hp);
        r = m.r32(r + 0x60);
        i += 1;
    }
}

/// FUN_00129bc8(o, src): deep clone of a Ref by tag *src: 0x1a..0x1e -> FUN_001291e0 / 1292b0 / 129330 / 129398 /
/// 129408(o, src[1]); 0x1f: tmp = FUN_00128cb8(src[1]) -> FUN_00129480(o, tmp), drop tmp (rc at +0: last ref ->
/// FUN_00256d40(tmp+0x10, 2), FUN_00129ec8(tmp+8), FUN_002550f8(tmp), FUN_0011cd98(0x4520a8, tmp)); 0x20: tmp =
/// FUN_00128f70(src[1]) -> FUN_001294a0(o, tmp), last ref -> FUN_00257148(tmp, 3); 0x21 -> FUN_001294c0;
/// else FUN_001291a8(o). Returns o.
pub fn ee_sub_129bc8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let tag = m.r32(a1);
    let x = m.r32(a1 + 4);
    match tag {
        0x1a => { m.call(0x0012_91e0, &[a0, x], &[]); }
        0x1b => { m.call(0x0012_92b0, &[a0, x], &[]); }
        0x1c => { m.call(0x0012_9330, &[a0, x], &[]); }
        0x1d => { m.call(0x0012_9398, &[a0, x], &[]); }
        0x1e => { m.call(0x0012_9408, &[a0, x], &[]); }
        0x1f | 0x20 => {
            let sp = stack_push(m, 0x10);
            m.w32(sp, 0);
            m.call(if tag == 0x1f { 0x0012_8cb8 } else { 0x0012_8f70 }, &[sp, x], &[]);
            let t = m.r32(sp);
            m.call(if tag == 0x1f { 0x0012_9480 } else { 0x0012_94a0 }, &[a0, t], &[]);
            if t != 0 {
                let rc = m.r32(t).wrapping_sub(1);
                m.w32(t, rc);
                if rc == 0 {
                    if tag == 0x1f {
                        m.call(0x0025_6d40, &[t + 0x10, 2], &[]);
                        m.call(0x0012_9ec8, &[t + 8], &[]);
                        m.call(0x0025_50f8, &[t], &[]);
                        m.call(0x0011_cd98, &[0x0045_20a8, t], &[]);
                    } else {
                        m.call(0x0025_7148, &[t, 3], &[]);
                    }
                }
            }
            stack_pop(m, 0x10);
        }
        0x21 => { m.call(0x0012_94c0, &[a0, x], &[]); }
        _ => { m.call(0x0012_91a8, &[a0], &[]); }
    }
    a0
}

/// FUN_0015f148(font, pkt): pkt == 0x2dea00 (the shared UI packet) -> upload once per font id (bit *font of
/// DAT_002dea4c+0x2c, set here). On upload: FUN_001576f0(pkt); FUN_00157500(pkt, font[0x16], font[0x17]) and
/// (font[0x30], font[0x31]); font[0x32] != 0 -> also (font[0x32], [0x33]) and (font[0x4c], [0x4d]); FUN_001574b8.
pub fn ee_sub_15f148<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let up = if a1 == 0x002d_ea00 {
        let bit = 1u32 << (m.r32(a0) & 0x1f);
        let p = m.r32(0x002d_ea4c) + 0x2c;
        let old = m.r32(p);
        m.w32(p, old | bit);
        old & bit == 0
    } else {
        true
    };
    if up {
        m.call(0x0015_76f0, &[a1], &[]);
        for (i, cond) in [(0x16u32, false), (0x30, false), (0x32, true), (0x4c, true)] {
            if cond && m.r32(a0 + 0x32 * 4) == 0 {
                continue;
            }
            let (x, y) = (m.r32(a0 + i * 4), m.r32(a0 + i * 4 + 4));
            m.call(0x0015_7500, &[a1, x, y], &[]);
        }
        m.call(0x0015_74b8, &[a1], &[]);
    }
    up as u32
}

/// FUN_001545f0(sensor, t): guard eye -> target line of sight. FUN_001ae478(frame) marks the scratch frame;
/// eye = FUN_00153e60(sensor); buf = FUN_001ae318({0x10, 0x10}); one target point buf[0] =
/// FUN_001a6f00(t, 0, *(qw*)0x2bdbf0); segment bbox lo / hi = vmini / vmax.xyz(eye, buf[0]) (w from eye);
/// FUN_0020faa0(0x3dd490, lo, hi, filter, sensor, &n, &list) with filter FUN_00154568, or FUN_00154528 when
/// sensor+0xf8 & 4; blocked = FUN_0014cc58(0, eye, buf[0], n, list, 0) != 0; FUN_001ae4a0(frame, 2).
/// Returns 1 = blocked (asm: slti / xori 1 on the clip loop counter).
pub fn ee_sub_1545f0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let sp = stack_push(m, 0x30);
    m.call(0x001a_e478, &[sp], &[]);
    let eye: V4 = m.call(0x0015_3e60, &[a0], &[]).1;
    m.w32(sp + 0x10, 0x10);
    m.w32(sp + 0x14, 0x10);
    let buf = m.call(0x001a_e318, &[sp + 0x10], &[]).0;
    let q = m.rv(0x002b_dbf0);
    let p = m.call(0x001a_6f00, &[a1, 0], &[q]).1;
    m.wv(buf, p);
    let (mut lo, mut hi) = (eye, eye);
    for i in 0..3 {
        lo[i] = eye[i].min(p[i]);
        hi[i] = eye[i].max(p[i]);
    }
    let filt = if m.r32(a0 + 0xf8) & 4 != 0 { 0x0015_4528 } else { 0x0015_4568 };
    m.call(0x0020_faa0, &[0x003d_d490, filt, a0, sp + 0x20, sp + 0x24], &[lo, hi]);
    let (n, l) = (m.r32(sp + 0x20), m.r32(sp + 0x24));
    let p0 = m.rv(buf);
    let hit = m.call(0x0014_cc58, &[0, n, l, 0], &[eye, p0]).0 != 0;
    m.call(0x001a_e4a0, &[sp, 2], &[]);
    stack_pop(m, 0x30);
    hit as u32
}

/// FUN_00139dc0(o): class 28 (C2cb5d8) particle step, slot 16 CPU branch (native twin classes/k21_28.rs).
/// +0x160 == 0 (motion by the +0x170 frame): DAT_002bc9c0 == 0 -> FUN_00139ab8(o) (VU0 path) and return; else
/// M = FUN_0015bfd8(o+0x170). Pushes the safe-iteration frame {list o+0x448, &next, link o+0x450, prev} on
/// DAT_002dd2fc, then per group g (double buffers at g + buf*0x840: +0 pos, +0x2b0 vel (w = death time), +0x560
/// spin; +0x1080 buf, +0x1084 count, +0x1088 mode-2 records): dead (vel.w <= DAT_002dded0) are dropped; mode 0
/// vel' = M vel (w = 1), modes 0/1 pos' = pos + vel' dt (DAT_002dded4); mode 2 FUN_00117120 Hermite (its VU0
/// control registers are absent from the decompile -> position carried, record compacted to the new index);
/// pos.w / vel.w / spin copied, +0x43c == 2 -> spin.w = FUN_0011cf60(spin.w + spin.z dt); bbox lo/hi over the
/// new positions. g+0x1084 -= dead, o+0x444 -= dead; empty -> FUN_00139098(g) else flip the buffer.
/// End: o+0x444 < 1 -> vtable +0x4c(o + vt+0x48 (i16)); else +0xf0 = (lo+hi)/2, +0x100 = |c - hi| + o+0x120.
pub fn ee_sub_139dc0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let mode = m.r32(a0 + 0x160) as i32;
    let sp = stack_push(m, 0x60);
    if mode == 0 {
        if m.r32(0x002b_c9c0) == 0 {
            stack_pop(m, 0x60);
            m.call(0x0013_9ab8, &[a0], &[]);
            return;
        }
        m.call(0x0015_bfd8, &[sp + 0x20, a0 + 0x170], &[]);
    }
    let mx = m.rm(sp + 0x20);
    let (fmax, fmin) = (m.rf(0x002a_4afc), m.rf(0x002a_4b00));
    let mut lo: V4 = [fmax, fmax, fmax, 0.0];
    let mut hi: V4 = [fmin, fmin, fmin, 0.0];
    let link = m.r32(a0 + 0x450);
    let prev = m.r32(0x002d_d2fc);
    let mut g = m.r32(a0 + 0x448);
    m.w32(sp, a0 + 0x448);
    m.w32(sp + 4, g.wrapping_add(link));
    m.w32(sp + 8, link);
    m.w32(sp + 0xc, prev);
    m.w32(0x002d_d2fc, sp);
    let now = m.rf(0x002d_ded0);
    let dt = m.rf(0x002d_ded4);
    let spin2 = m.r32(a0 + 0x43c) == 2;
    while g != 0 {
        let cur = m.r32(g + 0x1080);
        let nb = cur.wrapping_add(1) & 1;
        let n = m.r32(g + 0x1084) as i32;
        let (src, dst) = (g + cur * 0x840 + 0x560, g + nb * 0x840 + 0x560);
        let (mut dead, mut k) = (0i32, 0u32);
        for j in 0..n.max(0) as u32 {
            let (sp_, dp) = (src + j * 0x10, dst + k * 0x10);
            let vel = m.rv(sp_ - 0x2b0);
            if vel[3] <= now {
                dead += 1;
                continue;
            }
            let pos = m.rv(sp_ - 0x560);
            match mode {
                0 | 1 => {
                    let v = if mode == 0 {
                        let mut r = vel;
                        for c in 0..3 {
                            r[c] = mx[0][c] * vel[0] + mx[1][c] * vel[1] + mx[2][c] * vel[2] + mx[3][c];
                        }
                        r
                    } else {
                        vel
                    };
                    m.wv(dp - 0x2b0, v);
                    m.wv(dp - 0x560, [pos[0] + v[0] * dt, pos[1] + v[1] * dt, pos[2] + v[2] * dt, pos[3] + v[3] * dt]);
                }
                2 => {
                    m.wv(dp - 0x560, pos);
                    if k != j {
                        let base = m.r32(g + 0x1088);
                        for w in 0..8 {
                            let x = m.r32(base + j * 0x20 + w * 4);
                            m.w32(base + k * 0x20 + w * 4, x);
                        }
                    }
                }
                _ => {}
            }
            let (pw, vw) = (m.r32(sp_ - 0x554), m.r32(sp_ - 0x2a4));
            m.w32(dp - 0x554, pw);
            m.w32(dp - 0x2a4, vw);
            let s = m.rv(sp_);
            m.wv(dp, s);
            if spin2 {
                let r = m.call(0x0011_cf60, &[], &[[s[3] + s[2] * dt, 0.0, 0.0, 0.0]]).1[0];
                m.wf(dp + 0xc, r);
            }
            let p = m.rv(dp - 0x560);
            for c in 0..3 {
                lo[c] = lo[c].min(p[c]);
                hi[c] = hi[c].max(p[c]);
            }
            k += 1;
        }
        m.w32(g + 0x1084, (n - dead) as u32);
        let t = m.r32(a0 + 0x444) as i32 - dead;
        m.w32(a0 + 0x444, t as u32);
        if n - dead < 1 {
            m.call(0x0013_9098, &[g], &[]);
        } else {
            m.w32(g + 0x1080, nb);
        }
        let nx = m.r32(sp + 4);
        g = m.r32(nx);
        m.w32(sp + 4, g.wrapping_add(link));
    }
    if (m.r32(a0 + 0x444) as i32) < 1 {
        let vt = m.r32(a0 + 0xc);
        let f = m.r32(vt + 0x4c);
        let adj = m.r16(vt + 0x48) as i16 as i32 as u32;
        m.call(f, &[a0.wrapping_add(adj)], &[]);
    } else {
        let c: V4 = [0, 1, 2, 3].map(|i| lo[i] * 0.5 + hi[i] * 0.5);
        m.wv(a0 + 0xf0, c);
        let d = [c[0] - hi[0], c[1] - hi[1], c[2] - hi[2]];
        let r = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() + m.rf(a0 + 0x120);
        m.wf(a0 + 0x100, r);
    }
    m.w32(0x002d_d2fc, prev);
    stack_pop(m, 0x60);
}
