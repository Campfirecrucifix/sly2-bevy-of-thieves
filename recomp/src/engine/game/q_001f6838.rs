//! WP-Q4: run 0x001f6838..0x001f8db8 - ALO/SO helpers: collision-record flag setters (+0xe0 count / +0xe4
//! records stride 0xb0, +0xec physics record, +0x148 flag word), the soft-body cloth mesh (+0x10 work
//! buffers {pos, vel, tri table}, +0x78 / +0x7c points, +0x80 / +0x84 planes stride 0x20, +0x90 / +0x98
//! counts, +0xa0 / +0xa4 edges {u16 v0, v1, f0, f1} stride 0x10) with its integrator and clone, the
//! per-object fade / freeze setters, light records (+0xe8 +0xdc), the scripted sound-emitter records
//! (+0x24 emitter, list +0x90 {kind, .., +0xc, +0x10 next}) and the C2ca2c0 pool static init.
//! 1:1 against `QMem` (classes::q::q2mem); every callee outside the run goes through `QMem::call` by address.
//! FUN_002333b4 (libc memset) is written inline. VU0 macro code: dot = x + y + z (`addw.x` 1, `adday.x`,
//! `maddz.x`), cross = OPMULA / OPMSUB.
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

/// &DAT_003dd490: the world
pub const WORLD: u32 = 0x003d_d490;
/// DAT_003e12ec: renderer-registration switch read by FUN_001f8668
pub const RENDER_REG: u32 = 0x003e_12ec;
/// DAT_002f7900: default owner of a "kind 3" fade record (FUN_001f7950)
pub const FADE_OWNER: u32 = 0x002f_7900;

fn sub(a: V4, b: V4) -> V4 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2], a[3] - b[3]]
}
/// VU0 `mul.xyz` + `addw.x` / `adday.x` / `maddz.x`: x + y + z of the product
fn dot3(a: V4, b: V4) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
/// OPMULA.xyz ACC, a, b; OPMSUB.xyz d, b, a -> d = a x b (w of d untouched = b's w here: callers ignore it)
fn cross(a: V4, b: V4) -> V4 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0], 0.0]
}
fn memset0<M: QMem + ?Sized>(m: &mut M, a: u32, n: u32) {
    for i in 0..n {
        m.w8(a + i, 0);
    }
}
/// `{size, align}` argument block of the frame allocators FUN_001ae1d0 / 2a0 / 378 (on the emulated stack)
fn alloc<M: QMem + ?Sized>(m: &mut M, f: u32, size: u32) -> u32 {
    let s = stack_push(m, 0x10);
    m.w32(s, size);
    m.w32(s + 4, 0x10);
    let r = m.call(f, &[s], &[]).0;
    stack_pop(m, 0x10);
    r
}
/// *(*(p) + off)
fn r32p<M: QMem + ?Sized>(m: &mut M, p: u32, off: u32) -> u32 {
    let b = m.r32(p);
    m.r32(b.wrapping_add(off))
}
fn bits148<M: QMem + ?Sized>(m: &mut M, o: u32) -> u64 {
    m.r64(o + 0x148)
}
fn w64<M: QMem + ?Sized>(m: &mut M, a: u32, v: u64) {
    m.w32(a, v as u32);
    m.w32(a + 4, (v >> 32) as u32);
}
fn isa<M: QMem + ?Sized>(m: &mut M, o: u32, cid: u32) -> bool {
    m.call(0x0013_2b10, &[o, cid], &[]).0 != 0
}

/// FUN_001f6838: bit 2 of the physics record's +0x70 flags (1 when there is no record)
pub fn ee_sub_1f6838<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let r = m.r32(o + 0xec);
    if r != 0 {
        return (m.r64(r + 0x70) >> 2) as u32 & 1;
    }
    1
}

/// FUN_001f6860: set bit 2 of the physics record's +0x70 flags, then FUN_001f4e28(o)
pub fn ee_sub_1f6860<M: QMem + ?Sized>(m: &mut M, o: u32, b: u32) {
    let r = m.r32(o + 0xec);
    if r != 0 {
        let v = m.r64(r + 0x70) & !4 | (b as u64 & 1) << 2;
        w64(m, r + 0x70, v);
        m.call(0x001f_4e28, &[o], &[]);
    }
}

/// FUN_001f6918: +0x148 bit 26, then FUN_00134320(world, o)
pub fn ee_sub_1f6918<M: QMem + ?Sized>(m: &mut M, o: u32, b: u32) {
    let v = bits148(m, o) & !(1 << 26) | (b as u64 & 1) << 26;
    w64(m, o + 0x148, v);
    m.call(0x0013_4320, &[WORLD, o], &[]);
}

/// FUN_001f6a68: every mesh record (+0xc0 stride 0x28, count +0xb8) gets +0xc |= 0x800 and its +0x24
/// object +0x10 bit 50
pub fn ee_sub_1f6a68<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let n = m.r32(o + 0xb8) as i32;
    for i in 0..n.max(0) as u32 {
        let r = m.r32(o + 0xc0) + i * 0x28;
        let x = m.r32(r + 0x24);
        let v = m.r32(r + 0xc) | 0x800;
        m.w32(r + 0xc, v);
        if x != 0 {
            let v = m.r64(x + 0x10) | 1 << 50;
            w64(m, x + 0x10, v);
        }
    }
}

/// FUN_001f6c08: cloth work buffers (once, +0x10 == 0): {pos[+0x90], vel[+0x90], tris[+0x98] x 3 ints}; each
/// face collects its 3 vertices from the edges (+0xa4 {u16 v0, v1, f0, f1} stride 0x10, +0xa0 count), then
/// faces whose (p1 - p0) x (p2 - p0) points against the face plane (+0x84 stride 0x20) swap p1 / p2.
pub fn ee_sub_1f6c08<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.r32(o + 0x10) != 0 {
        return;
    }
    let fr = stack_push(m, 0x10);
    m.call(0x001a_e478, &[fr], &[]);
    let w = alloc(m, 0x001a_e1d0, 0xc);
    m.w32(o + 0x10, w);
    let n90 = m.r32(o + 0x90);
    let a = alloc(m, 0x001a_e2a0, n90 << 4);
    m.w32(w, a);
    let n90 = m.r32(o + 0x90);
    let a = alloc(m, 0x001a_e2a0, n90 << 4);
    m.w32(w + 4, a);
    let n98 = m.r32(o + 0x98);
    let a = alloc(m, 0x001a_e2a0, n98 * 0xc);
    m.w32(w + 8, a);
    let n98 = m.r32(o + 0x98);
    let cnt = alloc(m, 0x001a_e378, n98 << 2);
    let ne = m.r32(o + 0xa0) as i32;
    for e in 0..ne.max(0) as u32 {
        let ed = m.r32(o + 0xa4) + e * 0x10;
        for j in 0..2 {
            let fc = m.r16(ed + 4 + 2 * j) as u32;
            let pc = cnt + fc * 4;
            let tri = r32p(m, o + 0x10, 8) + fc * 0xc;
            for k in 0..2 {
                let v = m.r16(ed + 2 * k) as u32;
                let c = m.r32(pc) as i32;
                let mut i = 0;
                while i < c && m.r32(tri + 4 * i as u32) != v {
                    i += 1;
                }
                if i >= c && i < 3 {
                    m.w32(tri + 4 * c as u32, v);
                    m.w32(pc, (c + 1) as u32);
                }
            }
        }
    }
    let pts = m.r32(o + 0x7c);
    let nt = m.r32(o + 0x98) as i32;
    for i in 0..nt.max(0) as u32 {
        let t = r32p(m, o + 0x10, 8) + i * 0xc;
        let (t0, t1, t2) = (m.r32(t), m.r32(t + 4), m.r32(t + 8));
        let p0 = m.rv(pts + t0 * 0x10);
        let a = sub(m.rv(pts + t1 * 0x10), p0);
        let b = sub(m.rv(pts + t2 * 0x10), p0);
        let pl = m.r32(o + 0x84) + i * 0x20;
        let nrm = m.rv(pl);
        if dot3(cross(a, b), nrm) < 0.0 {
            m.w32(t + 8, t1);
            m.w32(t + 4, t2);
        }
    }
    m.call(0x001a_e4a0, &[fr, 2], &[]);
    stack_pop(m, 0x10);
}

/// FUN_001f6ef8(o, dt): cloth step. Per point i (+0x78): v' = v + a dt (work buf 0 / 1), p' = p + (v + v') dt/2
/// (all four lanes, VU `adda vf0` adds vf0.w = 1 to w); +0xa8 = -1 once a point moves more than 1 (squared);
/// +8 = the largest |p'|; then every plane (+0x80, +0x84 stride 0x20) is rebuilt by FUN_00119308(plane, p0, p1, p2).
pub fn ee_sub_1f6ef8<M: QMem + ?Sized>(m: &mut M, o: u32, dt: f32) {
    let w = m.r32(o + 0x10);
    let (mut pv, mut pa) = (m.r32(w), m.r32(w + 4));
    let pts = m.r32(o + 0x7c);
    let mut pp = pts;
    let h = dt * 0.5;
    let mut mx = 0.0f32;
    let mut i = 0;
    while i < m.r32(o + 0x78) as i32 {
        let v = m.rv(pv);
        let p = m.rv(pp);
        let a = m.rv(pa);
        let v2: V4 = std::array::from_fn(|k| (if k == 3 { 1.0 } else { 0.0 }) + v[k] + a[k] * dt);
        let p2: V4 = std::array::from_fn(|k| (if k == 3 { 1.0 } else { 0.0 }) + p[k] + v[k] * h + v2[k] * h);
        if m.r32(o + 0xa8) as i32 != -1 {
            let d = sub(p, p2);
            if 1.0 < dot3(d, d) {
                m.w32(o + 0xa8, u32::MAX);
            }
        }
        let l = dot3(p2, p2);
        m.wv(pp, p2);
        m.wv(pv, v2);
        if mx < l {
            mx = l;
        }
        i += 1;
        pv += 0x10;
        pp += 0x10;
        pa += 0x10;
    }
    m.wf(o + 8, mx.sqrt());
    let tris = r32p(m, o + 0x10, 8);
    let mut pl = m.r32(o + 0x84);
    let mut i = 0;
    while i < m.r32(o + 0x80) as i32 {
        let t = tris + i as u32 * 0xc;
        let q: Vec<V4> = (0..3).map(|k| {
            let ix = m.r32(t + 4 * k);
            m.rv(pts + ix * 0x10)
        }).collect();
        m.call(0x0011_9308, &[pl], &q);
        pl += 0x20;
        i += 1;
    }
}

/// FUN_001f70c0(src, owner, extra, dst): clone the cloth `src` into `dst` for `owner` with `extra` more points
/// inserted after point +0x1c: dst {+0 owner, +4 / +0xc / +0x1c copied, +8 max |p|}; point array grown
/// (FUN_001ae378 + 2 FUN_001ae5a0 copies), transformed by inverse(owner +0x40) * (src +0 +0x40)
/// (FUN_0011c318 / FUN_0011c168) into dst +0x78, then by owner +0x40 into dst +0x90 (FUN_00166270); planes
/// FUN_001191e8(src +0x68, src +0x90, dst +0x68, dst +0x90).
pub fn ee_sub_1f70c0<M: QMem + ?Sized>(m: &mut M, src: u32, owner: u32, extra: u32, dst: u32) {
    let sp = stack_push(m, 0x110);
    let (fr, pts, m80, mc0) = (sp + 0x60, sp + 0x40, sp + 0x80, sp + 0xc0);
    m.call(0x001a_e478, &[fr], &[]);
    m.w32(dst, owner);
    for off in [4, 0xc, 0x1c] {
        let v = m.r32(src + off);
        m.w32(dst + off, v);
    }
    let mut mx = 0.0f32;
    let n = m.r32(src + 0x78) as i32;
    let mut p = m.r32(src + 0x7c);
    for _ in 0..n.max(0) {
        let v = m.rv(p);
        let l = dot3(v, v);
        if mx < l {
            mx = l;
        }
        p += 0x10;
    }
    m.wf(dst + 8, mx.sqrt());
    for k in 0..6 {
        let v = m.r32(src + 0x78 + 4 * k);
        m.w32(pts + 4 * k, v);
    }
    let cnt = m.r32(pts);
    let buf = alloc(m, 0x001a_e378, (cnt.wrapping_add(extra)) << 4);
    let s0 = m.r32(src + 0x1c);
    let old = m.r32(pts + 4);
    m.call(0x001a_e5a0, &[buf, old, s0 << 4], &[]);
    let cnt = m.r32(pts);
    let old = m.r32(pts + 4);
    m.call(0x001a_e5a0, &[buf + ((s0 + extra) << 4), old + (s0 << 4), (cnt.wrapping_sub(s0)) << 4], &[]);
    let cnt = m.r32(pts);
    m.w32(pts + 4, buf);
    m.w32(pts, cnt.wrapping_add(extra));
    m.call(0x0011_c318, &[mc0, owner + 0x40], &[]);
    let sm = m.r32(src);
    m.call(0x0011_c168, &[m80, mc0, sm + 0x40], &[]);
    let mat = m.rm(m80);
    m.wm(sp, mat);
    m.call(0x0016_6270, &[pts, sp, dst + 0x78], &[]);
    m.call(0x0016_6270, &[dst + 0x78, owner + 0x40, dst + 0x90], &[]);
    m.call(0x0011_91e8, &[src + 0x68, src + 0x90, dst + 0x68, dst + 0x90], &[]);
    m.call(0x001a_e4a0, &[fr, 2], &[]);
    stack_pop(m, 0x110);
}

/// FUN_001f7530: first of `o` and its parents (+0x10) for which FUN_001f78d8(x, a1) holds (0 = none)
pub fn ee_sub_1f7530<M: QMem + ?Sized>(m: &mut M, mut o: u32, a1: u32) -> u32 {
    while o != 0 {
        if m.call(0x001f_78d8, &[o, a1], &[]).0 != 0 {
            return o;
        }
        o = m.r32(o + 0x10);
    }
    0
}

/// FUN_001f7950: re-arm the fade of `o` (physics record +0xec, after vtable +0x254 slot 73 when record +0x6c
/// is set): fade kind = +0x148 bits 24..25; kind 0 -> nothing; FUN_001f7a68(o, 0), then a 0x60-byte request
/// {+0 owner, +8 o, +0xc 0x20000, +0x10 (s8)+0x149, +0x20 FUN_001a62e8(o), +0x30 vtable +0x10c slot 32} to
/// FUN_001f7598; kind 3 objects of class 0x38 use {DAT_002f7900, 0x40000, -50} instead.
pub fn ee_sub_1f7950<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.r32(o + 0xec) == 0 {
        return;
    }
    let r = m.r32(o + 0xec);
    if m.r32(r + 0x6c) != 0 {
        vslot(m, o, 73, &[], &[]);
    }
    if m.r32(o + 0xec) == 0 {
        return;
    }
    let k = (bits148(m, o) >> 24) as u32 & 3;
    if k == 0 {
        return;
    }
    ee_sub_1f7a68(m, o, 0, 0, 0, 0);
    let s = stack_push(m, 0x60);
    memset0(m, s, 0x60);
    m.w32(s + 8, o);
    let q = m.call(0x001a_62e8, &[o], &[]).1;
    m.wv(s + 0x20, q);
    let q = vslot(m, o, 32, &[], &[]).1;
    m.wv(s + 0x30, q);
    if k == 3 && isa(m, o, 0x38) {
        m.w32(s + 0xc, 0x40000);
        m.w32(s + 0x10, -50i32 as u32);
        let d = m.r32(FADE_OWNER);
        m.w32(s, d);
    } else {
        let c = m.r8(o + 0x149) as i8 as i32;
        m.w32(s + 0xc, 0x20000);
        m.w32(s + 0x10, c as u32);
        m.w32(s, o);
    }
    m.call(0x001f_7598, &[s], &[]);
    stack_pop(m, 0x60);
}

/// FUN_001f7a68(o, kind, a5, a6, a7): set the fade kind (+0x148 bits 24..25). Collision records (+0xe4 stride
/// 0xb0) whose +0x18 equals `from` get {+0x14 (s16) = pri, +0x18 = to}: kind 2 {from 0, to 0x10000, pri =
/// (s8)+0x148}, kinds 0/1/3 {from 0x10000, to 0, pri -10}; any other kind uses the caller's (a5 to, a6 pri, a7 from).
pub fn ee_sub_1f7a68<M: QMem + ?Sized>(m: &mut M, o: u32, kind: u32, a5: u32, a6: i16, a7: u32) {
    if kind as u64 == (bits148(m, o) >> 24) & 3 {
        return;
    }
    let k = kind as i32;
    let (to, pri, from) = if k == 2 {
        (0x10000, m.r8(o + 0x148) as i8 as i16, 0)
    } else if (0..=3).contains(&k) {
        (0, -10, 0x10000)
    } else {
        (a5, a6, a7)
    };
    let mut i = 0;
    while i < m.r32(o + 0xe0) as i32 {
        let r = m.r32(o + 0xe4) + i as u32 * 0xb0;
        if m.r32(r + 0x18) == from {
            m.w16(r + 0x14, pri as u16);
            m.w32(r + 0x18, to);
        }
        i += 1;
    }
    let v = bits148(m, o) & !(3 << 24) | (kind as u64 & 3) << 24;
    w64(m, o + 0x148, v);
}

/// FUN_001f7b98: spawn object 0x187 under `o` unless it exists (FUN_0015e938(world, 2, 0x187, o) ->
/// FUN_0013e3c8(0x12, o, 0x187, -1))
pub fn ee_sub_1f7b98<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.call(0x0015_e938, &[WORLD, 2, 0x187, o], &[]).0 == 0 {
        m.call(0x0013_e3c8, &[0x12, o, 0x187, u32::MAX], &[]);
    }
}

/// FUN_001f7bf8(o, t): fade over `t` seconds: t <= 0 -> FUN_001f7d90(o, 0) + FUN_001f7d10(o, 0);
/// else FUN_001f7d90(o, -1 / t)
pub fn ee_sub_1f7bf8<M: QMem + ?Sized>(m: &mut M, o: u32, t: f32) {
    if t <= 0.0 {
        ee_sub_1f7d90(m, o, 0.0);
        ee_sub_1f7d10(m, o, 0.0);
    } else {
        ee_sub_1f7d90(m, o, -1.0 / t);
    }
}

/// FUN_001f7d10(o, a): alpha +0x12c = a; a > 0 -> vtable slot 7 (add to world); a <= 0 -> slot 8 (remove)
/// unless `o` is class 0x69
pub fn ee_sub_1f7d10<M: QMem + ?Sized>(m: &mut M, o: u32, a: f32) {
    if a <= 0.0 {
        if !isa(m, o, 0x69) {
            vslot(m, o, 8, &[], &[]);
        }
    } else {
        vslot(m, o, 7, &[], &[]);
    }
    m.wf(o + 300, a);
}

/// FUN_001f7d90(o, rate): fade rate +0x130; a non-zero rate adds `o` (slot 7) and sets bit 1 of its +0x40
/// update flags (tagged word when bit 0 is set, else the word it points to)
pub fn ee_sub_1f7d90<M: QMem + ?Sized>(m: &mut M, o: u32, r: f32) {
    if r != 0.0 {
        vslot(m, o, 7, &[], &[]);
        let p = m.r32(o + 0x40);
        if p & 1 != 0 {
            m.w32(o + 0x40, p | 2);
        } else {
            let v = m.r32(p) | 2;
            m.w32(p, v);
        }
    }
    m.wf(o + 0x130, r);
}

/// FUN_001f7e08(o, pred, ctx, out): collision records (+0xe4 stride 0xb0) accepted by `pred(ctx, rec)`
/// (every one when pred is 0) written to `out`; returns the count
pub fn ee_sub_1f7e08<M: QMem + ?Sized>(m: &mut M, o: u32, pred: u32, ctx: u32, mut out: u32) -> u32 {
    let mut n = 0;
    let mut i = 0;
    while i < m.r32(o + 0xe0) as i32 {
        let r = m.r32(o + 0xe4) + i as u32 * 0xb0;
        if pred == 0 || m.call(pred, &[ctx, r], &[]).0 != 0 {
            m.w32(out, r);
            n += 1;
            out += 4;
        }
        i += 1;
    }
    n
}

/// FUN_001f7f48: +0x148 bit 34; on change FUN_001f55c8(o)
pub fn ee_sub_1f7f48<M: QMem + ?Sized>(m: &mut M, o: u32, b: u32) {
    let v = bits148(m, o);
    if b as u64 != (v >> 34) & 1 {
        w64(m, o + 0x148, v & !(1 << 34) | (b as u64 & 1) << 34);
        m.call(0x001f_55c8, &[o], &[]);
    }
}

fn each_attached<M: QMem + ?Sized>(m: &mut M, o: u32, cid: u32, f: u32, pre: &[u32]) {
    let mut i = 0;
    while i < m.r32(o + 0xa4) as i32 {
        let x = r32p(m, o + 0xa8, 4 * i as u32);
        if isa(m, x, cid) {
            let mut a = pre.to_vec();
            a.push(x);
            m.call(f, &a, &[]);
        }
        i += 1;
    }
}

/// FUN_001f7fa0(o, a, b): FUN_001ee678(x, a, b) for every attached object (+0xa8, count +0xa4) of class 0x8f
/// VFIX: ABI per asm: o = a0, a = $f12, b = $f13 (kept in f21/f20 across the loop)
pub fn ee_sub_1f7fa0<M: QMem + ?Sized>(m: &mut M, o: u32, a: f32, b: f32) {
    let mut i = 0;
    while i < m.r32(o + 0xa4) as i32 {
        let x = r32p(m, o + 0xa8, 4 * i as u32);
        if isa(m, x, 0x8f) {
            m.call(0x001e_e678, &[x], &[[a, 0.0, 0.0, 0.0], [b, 0.0, 0.0, 0.0]]);
        }
        i += 1;
    }
}

/// FUN_001f8038(a0, a1, o): as FUN_001f7fa0 with FUN_001edad8
pub fn ee_sub_1f8038<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, o: u32) {
    each_attached(m, o, 0x8f, 0x001e_dad8, &[a0, a1]);
}

/// FUN_001f80d0(a0, o): FUN_001ee3b0(a0, x) for attached class-0x8e objects, then recursively for every
/// class-0x30 child (+0x64 .. +0x70 sentinel, linked by +0x24)
pub fn ee_sub_1f80d0<M: QMem + ?Sized>(m: &mut M, a0: u32, o: u32) {
    each_attached(m, o, 0x8e, 0x001e_e3b0, &[a0]);
    let mut c = m.r32(o + 0x64);
    while c != m.r32(o + 0x70) {
        if isa(m, c, 0x30) {
            ee_sub_1f80d0(m, a0, c);
        }
        c = m.r32(c + 0x24);
    }
}

/// FUN_001f81e0(o, id): +0xb4 = -(i + 3) for the index of `id` in the +0xb0 list (count +0xac), else 0
pub fn ee_sub_1f81e0<M: QMem + ?Sized>(m: &mut M, o: u32, id: i32) {
    if id != -1 {
        let mut i = 0;
        while i < m.r32(o + 0xac) as i32 {
            if r32p(m, o + 0xb0, 4 * i as u32) as i32 == id {
                m.w32(o + 0xb4, (-(i + 3)) as u32);
                return;
            }
            i += 1;
        }
    }
    m.w32(o + 0xb4, 0);
}

/// FUN_001f8238: +0xb4 = 0
pub fn ee_sub_1f8238<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.w32(o + 0xb4, 0);
}

/// FUN_001f8300: the light record of `o` (+0xe8 record +0xdc), created on first use (0x70 bytes,
/// FUN_001ae2a0): {+0 o, +8 range 150 for class 0x34 else 75, +0xc 1.0}; 0 without +0xe8
pub fn ee_sub_1f8300<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let e = m.r32(o + 0xe8);
    if e == 0 {
        return 0;
    }
    if m.r32(e + 0xdc) == 0 {
        let p = alloc(m, 0x001a_e2a0, 0x70);
        m.w32(p, o);
        let r = if isa(m, o, 0x34) { 0x4316_0000 } else { 0x4296_0000 };
        m.w32(p + 8, r);
        m.w32(p + 0xc, 0x3f80_0000);
        let e = m.r32(o + 0xe8);
        m.w32(e + 0xdc, p);
    }
    let e = m.r32(o + 0xe8);
    m.r32(e + 0xdc)
}

fn set_light<M: QMem + ?Sized>(m: &mut M, o: u32, off: u32, v: f32) {
    let p = ee_sub_1f8300(m, o);
    if p != 0 {
        m.wf(p + off, v);
    }
}
/// FUN_001f83b0(o, f12): light record +8
pub fn ee_sub_1f83b0<M: QMem + ?Sized>(m: &mut M, o: u32, v: f32) {
    set_light(m, o, 8, v)
}
/// FUN_001f8410(o, f12): light record +0xc
pub fn ee_sub_1f8410<M: QMem + ?Sized>(m: &mut M, o: u32, v: f32) {
    set_light(m, o, 0xc, v)
}
/// FUN_001f8470(o, f12): light record +4
pub fn ee_sub_1f8470<M: QMem + ?Sized>(m: &mut M, o: u32, v: f32) {
    set_light(m, o, 4, v)
}

/// FUN_001f8510 (C2ca2c0 ctor)(o, sub, id): the first entry (+0x54 stride 0x100, count +0x50) with +0x94 == id
/// and (sub == -1 or +0x9c == sub) gets a 0x30-byte C2ca2c0 handle from pool 0x3e15cc (FUN_0011ccc0):
/// {+0xc vtable 0x2ca2c0, +0x20 o, +0x24 entry}; 0 when none
pub fn ee_sub_1f8510<M: QMem + ?Sized>(m: &mut M, o: u32, sub: i32, id: u32) -> u32 {
    let mut i = 0;
    while i < m.r32(o + 0x50) as i32 {
        let e = m.r32(o + 0x54) + i as u32 * 0x100;
        if m.r32(e + 0x94) == id && (sub == -1 || m.r32(e + 0x9c) as i32 == sub) {
            let u = m.call(0x0011_ccc0, &[0x003e_15cc], &[]).0;
            memset0(m, u, 0x30);
            m.w32(u + 0xc, 0x002c_a2c0);
            m.w32(u + 0x20, o);
            m.w32(u + 0x24, e);
            m.w32(u, 0);
            m.w32(u + 4, 0);
            return u;
        }
        i += 1;
    }
    0
}

/// FUN_001f8668(o, b): +0x148 bit 35 (hidden) on `o` and its class-0x30 children (+0x64 list, +0x24);
/// with DAT_003e12ec set the change also (un)registers it: FUN_001d6310 (b = 0) / FUN_001d6378 (b = 1)
pub fn ee_sub_1f8668<M: QMem + ?Sized>(m: &mut M, o: u32, b: u32) {
    let v = bits148(m, o);
    if b as u64 == (v >> 35) & 1 {
        return;
    }
    w64(m, o + 0x148, v & !(1 << 35) | (b as u64 & 1) << 35);
    if m.r32(RENDER_REG) != 0 {
        m.call(if b == 0 { 0x001d_6310 } else { 0x001d_6378 }, &[o], &[]);
    }
    let mut c = m.r32(o + 0x64);
    while c != 0 {
        if isa(m, c, 0x30) {
            ee_sub_1f8668(m, c, b);
        }
        c = m.r32(c + 0x24);
    }
}

/// FUN_001f88d8(h, ev): ev 1 -> FUN_00230598(h +0x20, h +0x24); returns h +0x24
pub fn ee_sub_1f88d8<M: QMem + ?Sized>(m: &mut M, h: u32, ev: u32) -> u32 {
    let (a, b) = (m.r32(h + 0x20), m.r32(h + 0x24));
    if ev == 1 {
        m.call(0x0023_0598, &[a, b], &[]);
    }
    m.r32(h + 0x24)
}

/// FUN_001f8910(h, kind, val, key): the emitter (h +0x24) entry of `kind` (list +0x90, next +0x10): val -1
/// removes it (FUN_00230100 + FUN_0012e210, returns 0); else a new key (+0xc) -> FUN_002301d0, and when it
/// is the head FUN_002303a0(.., val) + FUN_00230598. Returns the entry (0 when none).
pub fn ee_sub_1f8910<M: QMem + ?Sized>(m: &mut M, h: u32, kind: u32, val: i32, key: u32) -> u32 {
    let mut p = r32p(m, h + 0x24, 0x90);
    while p != 0 && m.r32(p) != kind {
        p = m.r32(p + 0x10);
    }
    if p == 0 {
        return 0;
    }
    let (a, b) = (m.r32(h + 0x20), m.r32(h + 0x24));
    if val == -1 {
        m.call(0x0023_0100, &[a, b, p], &[]);
        m.call(0x0012_e210, &[p], &[]);
        return 0;
    }
    if key != m.r32(p + 0xc) {
        m.w32(p + 0xc, key);
        m.call(0x0023_01d0, &[a, b], &[]);
    }
    let e = m.r32(h + 0x24);
    if p == m.r32(e + 0x90) {
        let a = m.r32(h + 0x20);
        m.call(0x0023_03a0, &[a, e, val as u32], &[]);
        let (a, b) = (m.r32(h + 0x20), m.r32(h + 0x24));
        m.call(0x0023_0598, &[a, b], &[]);
    }
    p
}

/// FUN_001f89e0(h, val, key, q): kind-1 entry: existing -> +0x14 = val, FUN_0012d108(e, q); missing and
/// val != -1 -> FUN_0012dc18(h +0x20, h +0x24, key, val, 2, q)
pub fn ee_sub_1f89e0<M: QMem + ?Sized>(m: &mut M, h: u32, val: i32, key: u32, q: V4) {
    let e = ee_sub_1f8910(m, h, 1, val, key);
    if e == 0 {
        if val != -1 {
            let (a, b) = (m.r32(h + 0x20), m.r32(h + 0x24));
            m.call(0x0012_dc18, &[a, b, key, val as u32, 2], &[q]);
        }
    } else {
        m.w32(e + 0x14, val as u32);
        m.call(0x0012_d108, &[e], &[q]);
    }
}

/// FUN_001f8a78(h, val, key, q): FUN_001f89e0 with q = FUN_0022fc48(0, emitter +0xa0, q)
pub fn ee_sub_1f8a78<M: QMem + ?Sized>(m: &mut M, h: u32, val: i32, key: u32, q: V4) {
    let e = m.r32(h + 0x24);
    let x = m.r32(e + 0xa0);
    let q = m.call(0x0022_fc48, &[0, x], &[q]).1;
    ee_sub_1f89e0(m, h, val, key, q);
}

/// FUN_001f8ad8(h, val, key): FUN_001f89e0 with q = FUN_0011bff0(copy of FUN_0022fd38(.., 0, emitter +0xa0))
pub fn ee_sub_1f8ad8<M: QMem + ?Sized>(m: &mut M, h: u32, val: i32, key: u32) {
    let s = stack_push(m, 0x60);
    let e = m.r32(h + 0x24);
    let x = m.r32(e + 0xa0);
    m.call(0x0022_fd38, &[s + 0x30, 0, x], &[]);
    for k in 0..3 {
        let v = m.rv(s + 0x30 + 0x10 * k);
        m.wv(s + 0x10 * k, v);
    }
    let q = m.call(0x0011_bff0, &[s], &[]).1;
    stack_pop(m, 0x60);
    ee_sub_1f89e0(m, h, val, key, q);
}

/// FUN_001f8b58(a0, a1, h, val, key): kind-4 entry: existing -> {+0x20 a1, +0x14 val, +0x1c a0}; missing and
/// val != -1 -> FUN_0012dca0(a0, a1, h +0x20, h +0x24, key, val)
pub fn ee_sub_1f8b58<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, h: u32, val: i32, key: u32) {
    let e = ee_sub_1f8910(m, h, 4, val, key);
    if e == 0 {
        if val != -1 {
            let (a, b) = (m.r32(h + 0x20), m.r32(h + 0x24));
            m.call(0x0012_dca0, &[a0, a1, a, b, key, val as u32], &[]);
        }
    } else {
        m.w32(e + 0x20, a1);
        m.w32(e + 0x14, val as u32);
        m.w32(e + 0x1c, a0);
    }
}

/// FUN_001f8bf0(h, a1, q, f12, f13, f14, f15): emitter kind 0x10 (h +0x24 +0x98) params record
/// (*(emitter +0xa8) +0x60): {+8 a1, +0xc f12, +0x10 f13, +0x14 f14, +0x18 f15 degrees -> radians, +0x20 q}
/// (disasm: a0 is the handle; the decompile's parameter order is wrong)
pub fn ee_sub_1f8bf0<M: QMem + ?Sized>(m: &mut M, h: u32, a1: u32, q: V4, f: [f32; 4]) {
    let e = m.r32(h + 0x24);
    if m.r16(e + 0x98) as i16 != 0x10 {
        return;
    }
    let r = r32p(m, e + 0xa8, 0x60);
    m.wv(r + 0x20, q);
    m.w32(r + 8, a1);
    m.wf(r + 0xc, f[0]);
    m.wf(r + 0x10, f[1]);
    m.wf(r + 0x14, f[2]);
    m.wf(r + 0x18, f[3] * 0.017453294);
}

/// FUN_001f8c40(init, prio): static init (prio 0xffff): C2ca2c0 descriptor 0x3dd020 = {DAT_002a2dc0 qword,
/// +0x10 = {1, 0, 0, 0}} and the six pools (FUN_0011cca8, first user of each counter 0x2dd2e0..0x2dd2f4).
pub fn ee_sub_1f8c40<M: QMem + ?Sized>(m: &mut M, init: u32, prio: u32) {
    if prio != 0xffff || init == 0 {
        return;
    }
    let q = m.rv(0x002a_2dc0);
    m.wv(0x003d_d020, q);
    m.wv(0x003d_d030, [f32::from_bits(1), 0.0, 0.0, 0.0]);
    const POOLS: [(u32, u32, u32, u32); 6] = [
        (0x002d_d2e0, 0x0045_20a8, 0x14, 0x2000),
        (0x002d_d2e4, 0x0045_20d0, 0x18, 0x800),
        (0x002d_d2e8, 0x0045_20f8, 0x28, 0x200),
        (0x002d_d2ec, 0x0045_2120, 0x1c, 0x800),
        (0x002d_d2f0, 0x0045_2148, 0xc, 200),
        (0x002d_d2f4, 0x0045_2170, 0x14, 0x100),
    ];
    for (c, p, sz, n) in POOLS {
        let v = m.r32(c) + 1;
        m.w32(c, v);
        if v == 1 {
            m.call(0x0011_cca8, &[p, sz, n, 0x10], &[]);
        }
    }
}

/// FUN_001f8db8: FUN_001f8c40(1, 0xffff)
pub fn ee_sub_1f8db8<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_1f8c40(m, 1, 0xffff);
}
