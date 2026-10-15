//! WP-Q5: run_0022fa20 queue package: the animation-binder (ASEG channel) transform helpers.
//! Channel record {+0x00..+0x3f local matrix, +0x90 binder (vtable +0x18 {short adj, fn} pairs),
//! +0x94 kind, +0x98 mode (short), +0x9a sub-mode, +0xa0 parent, +0xa4 stamp, +0xb0 rate quad,
//! +0xc0.. curve quads}: parent-space composition (FUN_0022fd38 3x3 / FUN_0022ff40 4x4), the binder
//! list sort FUN_002301d0, mode sets (FUN_00230310 / FUN_00230348), the mode switch FUN_002303a0
//! and its triggers (FUN_00230598 / FUN_00230630), the per-channel matrix / velocity evaluation
//! FUN_00230710, the chain accumulation FUN_00230a98 and the per-frame binder pass FUN_00230cf0.
//! 1:1 from the disassembly against `QMem` (VU0 vmula/vmadd chains keep the untouched w lanes).
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem, V4};
use crate::engine::classes::q::q5util::f4;

/// `vmulax.xyz ACC, a, v; vmadday.xyz ACC, b, v; vmaddz.xyz d, c, v` with d's old w kept
fn lin(a: V4, b: V4, c: V4, v: V4, w: f32) -> V4 {
    let f = |i: usize| a[i] * v[0] + b[i] * v[1] + c[i] * v[2];
    [f(0), f(1), f(2), w]
}

fn copy<M: QMem + ?Sized>(m: &mut M, d: u32, s: u32, n: u32) {
    for k in (0..n).step_by(4) {
        let v = m.r32(s + k);
        m.w32(d + k, v);
    }
}

/// C++ virtual call through a `{short this-adjust, fn}` pair at vtable +`at`
fn vcall<M: QMem + ?Sized>(m: &mut M, o: u32, vt: u32, at: u32, a: &[u32], v: &[V4]) -> (u32, V4) {
    let adj = m.r16(vt + at) as i16 as i32;
    let f = m.r32(vt + at + 4);
    let mut args = vec![o.wrapping_add(adj as u32)];
    args.extend_from_slice(a);
    m.call(f, &args, v)
}

/// FUN_0022fa20: when flagged (+0x28 bit 60) and the stage id +0x140 (-1 = any) differs from the
/// owner's (+0x60 -> +0xa4): FUN_001d6310 + FUN_001d6378; without a parent (+0x10) the location
/// (0x3dd490) gets FUN_00147e88(+0x14)
pub fn ee_sub_22fa20<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.r64(o + 0x28) >> 0x3c & 1 == 0 {
        return;
    }
    let id = m.r32(o + 0x140);
    let ow = m.r32(o + 0x60);
    if id != 0xffff_ffff && m.r32(ow + 0xa4) != id {
        m.call(0x001d_6310, &[o], &[]);
        m.call(0x001d_6378, &[o], &[]);
    }
    if m.r32(o + 0x10) == 0 {
        let v = m.r32(o + 0x14);
        m.call(0x0014_7e88, &[0x003d_d490, v], &[]);
    }
}

/// FUN_0022fd38: 3x3 rotation `s` (rows at s) into the space of parent `p` (+0x40 rows) relative
/// to `q` (transposed +0x40 rows, FUN_0011b340) -> `d` (0x30 bytes); either may be 0
pub fn ee_sub_22fd38<M: QMem + ?Sized>(m: &mut M, d: u32, p: u32, q: u32, s: u32) -> u32 {
    if p == 0 && q == 0 {
        copy(m, d, s, 0x30);
        return d;
    }
    let sp = stack_push(m, 0x30);
    let (s0, s1, s2) = (m.rv(s), m.rv(s + 0x10), m.rv(s + 0x20));
    let (a, b, c, w1) = if q != 0 {
        m.call(0x0011_b340, &[sp, q + 0x40], &[]);
        let t = [m.rv(sp), m.rv(sp + 0x10), m.rv(sp + 0x20)];
        if p != 0 {
            let pr = [m.rv(p + 0x40), m.rv(p + 0x50), m.rv(p + 0x60)];
            let q0 = lin(t[0], t[1], t[2], pr[0], pr[0][3]);
            let q1 = lin(t[0], t[1], t[2], pr[1], pr[1][3]);
            let q2 = lin(t[0], t[1], t[2], pr[2], t[0][3]);
            (q0, q1, q2, q0[3])
        } else {
            (t[0], t[1], t[2], t[0][3])
        }
    } else {
        let pr = [m.rv(p + 0x40), m.rv(p + 0x50), m.rv(p + 0x60)];
        (pr[0], pr[1], pr[2], pr[0][3])
    };
    stack_pop(m, 0x30);
    let r0 = lin(a, b, c, s0, s0[3]);
    let r1 = lin(a, b, c, s1, w1);
    let r2 = lin(a, b, c, s2, s2[3]);
    m.wv(d, r0);
    m.wv(d + 0x10, r1);
    m.wv(d + 0x20, r2);
    d
}

/// FUN_0022ff40: 4x4 version: d = s * (inverse(q +0x40, FUN_0011c318) * p +0x40) (FUN_0011c168)
pub fn ee_sub_22ff40<M: QMem + ?Sized>(m: &mut M, d: u32, p: u32, q: u32, s: u32) -> u32 {
    let sp = stack_push(m, 0xc0);
    let (r, e, a) = (sp, sp + 0x40, sp + 0x80);
    let src = if p == 0 {
        if q == 0 {
            stack_pop(m, 0xc0);
            copy(m, d, s, 0x40);
            return d;
        }
        m.call(0x0011_c318, &[e, q + 0x40], &[]);
        e
    } else if q == 0 {
        p + 0x40
    } else {
        m.call(0x0011_c318, &[a, q + 0x40], &[]);
        m.call(0x0011_c168, &[e, a, p + 0x40], &[]);
        e
    };
    m.call(0x0011_c168, &[r, src, s], &[]);
    copy(m, d, r, 0x40);
    stack_pop(m, 0xc0);
    d
}

/// FUN_002301d0: re-sort channel `c`'s binder list (+0x90, next +0x10) by descending priority
/// (+0xc, stable for equal keys inserted in reverse); unchanged head: head slot (+0x18/+0x1c);
/// new head: FUN_0022c840(o, c), old head leave (+0x10/+0x14), new head enter (+0x8/+0xc)
pub fn ee_sub_2301d0<M: QMem + ?Sized>(m: &mut M, o: u32, c: u32) {
    let mut items = Vec::new();
    let mut n = m.r32(c + 0x90);
    while n != 0 {
        items.push(n);
        n = m.r32(n + 0x10);
    }
    let old = m.r32(c + 0x90);
    m.w32(c + 0x90, 0);
    for &it in items.iter().rev() {
        let mut link = c + 0x90;
        let mut cur = m.r32(link);
        while cur != 0 && (m.r32(it + 0xc) as i32) < m.r32(cur + 0xc) as i32 {
            link = cur + 0x10;
            cur = m.r32(link);
        }
        m.w32(it + 0x10, cur);
        m.w32(link, it);
    }
    let head = m.r32(c + 0x90);
    if head == old {
        if old != 0 {
            let vt = m.r32(old + 0x18);
            vcall(m, old, vt, 0x18, &[], &[]);
        }
        return;
    }
    m.call(0x0022_c840, &[o, c], &[]);
    if old != 0 {
        let vt = m.r32(old + 0x18);
        vcall(m, old, vt, 0x10, &[], &[]);
    }
    let head = m.r32(c + 0x90);
    if head != 0 {
        let vt = m.r32(head + 0x18);
        vcall(m, head, vt, 8, &[], &[]);
    }
}

/// FUN_00230310: modes driven by the channel's own motion
pub fn ee_sub_230310(k: u32) -> bool {
    matches!(k, 1 | 2 | 3 | 7 | 8 | 9 | 10 | 0xb | 0xc | 0x11 | 0x12 | 0x13 | 0x14 | 0x15)
}

/// FUN_00230348: modes that need no owner refresh
pub fn ee_sub_230348(k: u32) -> bool {
    matches!(k, 0 | 3 | 10 | 0xc | 0x13)
}

/// FUN_00230380: 1 <= k < 4
fn ee_sub_230380(k: i32) -> bool {
    (1..4).contains(&k)
}

/// FUN_002303a0: switch channel `c` of owner `o` to mode `k`. Entering a motion mode from a
/// static one seeds +0xb0 from the current world matrix (FUN_0022cdc0) by kind +0x94: 1/4/5/9/10
/// translation row, 6/7 rotation angles of M * inverse(c) (FUN_0011c318 / FUN_0011c168), 0xb / 0xc
/// two other Euler extractions. Modes 0/0xd..0x11/0x14/0x15 and 1..3 re-bind (FUN_0022c818; 1..3
/// from a non 1..3 mode also clear +0xf0..+0xf8); then mode = k and, unless FUN_00230348(k), the
/// owner's slot (+0xc vtable +0x178) is told 4. Returns the last v0.
pub fn ee_sub_2303a0<M: QMem + ?Sized>(m: &mut M, o: u32, c: u32, k: u32) -> u32 {
    let cur = m.r16(c + 0x98) as i16 as i32 as u32;
    if k == cur {
        return cur;
    }
    if ee_sub_230310(k) && !ee_sub_230310(cur) {
        let sp = stack_push(m, 0xc0);
        let (mm, w, inv) = (sp, sp + 0x40, sp + 0x80);
        m.call(0x0022_cdc0, &[w, c], &[]);
        copy(m, mm, w, 0x40);
        let f = |m: &mut M, at: u32| m.r32(mm + at);
        match m.r32(c + 0x94) {
            1 | 4 | 5 | 9 | 10 => copy(m, c + 0xb0, mm + 0x30, 0x10),
            6 | 7 => {
                m.call(0x0011_c318, &[inv, c], &[]);
                m.call(0x0011_c168, &[w, mm, inv], &[]);
                copy(m, mm, w, 0x40);
                let (a, b, d) = (f(m, 0x18), f(m, 0x20), f(m, 4));
                m.w32(c + 0xb0, a);
                m.w32(c + 0xb8, d);
                m.w32(c + 0xb4, b);
            }
            0xb => {
                let (a, b, d) = (f(m, 0x10), f(m, 0x20), f(m, 0x24));
                m.w32(c + 0xb0, a);
                m.w32(c + 0xb8, d);
                m.w32(c + 0xb4, b);
            }
            0xc => {
                let (a, b, d) = (f(m, 0), f(m, 0x14), f(m, 0x28));
                m.w32(c + 0xb0, a);
                m.w32(c + 0xb8, d);
                m.w32(c + 0xb4, b);
            }
            _ => {}
        }
        stack_pop(m, 0xc0);
    }
    match k {
        0 | 0xd..=0x11 | 0x14 | 0x15 => {
            m.call(0x0022_c818, &[o, c], &[]);
        }
        1..=3 => {
            if !ee_sub_230380(m.r16(c + 0x98) as i16 as i32) {
                m.w32(c + 0xf4, 0);
                m.w32(c + 0xf8, 0);
                m.w32(c + 0xf0, 0);
            }
            m.call(0x0022_c818, &[o, c], &[]);
        }
        _ => {}
    }
    m.w16(c + 0x98, k as u16);
    if ee_sub_230348(k) {
        return 1;
    }
    let vt = m.r32(o + 0xc);
    vcall(m, o, vt, 0x178, &[4], &[]).0
}

/// FUN_00230598: stop trigger: mode 3 -> 1, 6 / 10 -> 7, 0xc -> 0xb, 0x13 -> 0x12
pub fn ee_sub_230598<M: QMem + ?Sized>(m: &mut M, o: u32, c: u32) {
    let k = match m.r16(c + 0x98) {
        3 => 1,
        6 | 10 => 7,
        0xc => 0xb,
        0x13 => 0x12,
        _ => return,
    };
    ee_sub_2303a0(m, o, c, k);
}

/// FUN_00230630: start trigger: mode 1 -> 3; 7 / 9 -> 10 (sub 2) or 6 (sub 1); 0xb -> 0xc and
/// 0x12 -> 0x13 (sub 2 only)
pub fn ee_sub_230630<M: QMem + ?Sized>(m: &mut M, o: u32, c: u32) {
    let sub = m.r16(c + 0x9a);
    let k = match (m.r16(c + 0x98), sub) {
        (1, _) => 3,
        (7 | 9, 2) => 10,
        (7 | 9, 1) => 6,
        (0xb, 2) => 0xc,
        (0x12, 2) => 0x13,
        _ => return,
    };
    ee_sub_2303a0(m, o, c, k);
}

/// FUN_00230710: channel `c` at time `t` -> local matrix `mo` and its velocity matrix `vo`:
/// a binder (+0x90) supplies {pose, rate} (slot +0x28/+0x2c) turned into matrices by FUN_0022c8c0
/// and FUN_0022cb90 (kinds 6/7) / FUN_0022cc00; without one, static modes copy +0 and zero `vo`,
/// kinds 6/7 spin the rotation by +0xb0 * t (FUN_0011b570, FUN_00256d18, FUN_0022ccd0), others
/// advance FUN_0022ca18 + (0,0,0,1) + +0xb0 * t (FUN_0022c8c0 / FUN_0022cc00)
pub fn ee_sub_230710<M: QMem + ?Sized>(m: &mut M, t: f32, c: u32, mo: u32, vo: u32) {
    let sp = stack_push(m, 0xb0);
    let kind = m.r32(c + 0x94) as i32;
    let spin = (6..8).contains(&kind);
    let b = m.r32(c + 0x90);
    let res;
    if b != 0 {
        let vt = m.r32(b + 0x18);
        vcall(m, b, vt, 0x28, &[sp, sp + 0x10], &[]);
        let (q0, q1) = (m.rv(sp), m.rv(sp + 0x10));
        m.call(0x0022_c8c0, &[sp + 0x20, c], &[q0]);
        copy(m, mo, sp + 0x20, 0x40);
        if spin {
            m.call(0x0022_cb90, &[sp + 0x20, c, mo], &[q0, q1]);
        } else {
            m.call(0x0022_cc00, &[sp + 0x20, c], &[q1]);
        }
        res = sp + 0x20;
    } else if !ee_sub_230310(m.r16(c + 0x98) as i16 as i32 as u32) {
        copy(m, mo, c, 0x40);
        for k in (0..0x40).step_by(4) {
            m.w32(vo + k, 0);
        }
        stack_pop(m, 0xb0);
        return;
    } else if spin {
        let r = m.rv(c + 0xb0);
        let v = [r[0] * t, r[1] * t, r[2] * t, r[3] * t];
        m.call(0x0011_b570, &[sp + 0x60], &[v]);
        let rr = [m.rv(sp + 0x60), m.rv(sp + 0x70), m.rv(sp + 0x80)];
        let p = [m.rv(c), m.rv(c + 0x10), m.rv(c + 0x20)];
        let n0 = lin(rr[0], rr[1], rr[2], p[0], p[0][3]);
        let n1 = lin(rr[0], rr[1], rr[2], p[1], p[1][3]);
        let n2 = lin(rr[0], rr[1], rr[2], p[2], rr[0][3]);
        for (i, n) in [n0, n1, n2].into_iter().enumerate() {
            let i = i as u32 * 0x10;
            m.wv(sp + i, n);
            m.wv(sp + 0x30 + i, n);
        }
        let p3 = m.rv(c + 0x30);
        m.call(0x0025_6d18, &[mo + 0x30], &[p3]);
        m.wv(mo, n0);
        m.wv(mo + 0x10, n1);
        m.wv(mo + 0x20, n2);
        m.wf(mo + 0x3c, 1.0);
        for a in [0x2c, 0x1c, 0xc] {
            m.w32(mo + a, 0);
        }
        let rate = m.rv(c + 0xb0);
        m.call(0x0022_ccd0, &[sp + 0x30, c, mo], &[rate]);
        res = sp + 0x30;
    } else {
        let r = m.call(0x0022_ca18, &[c, c], &[]).1;
        let rate = m.rv(c + 0xb0);
        let v = [r[0] + rate[0] * t, r[1] + rate[1] * t, r[2] + rate[2] * t, (r[3] + 1.0) + rate[3] * t];
        m.call(0x0022_c8c0, &[sp + 0x10, c], &[v]);
        copy(m, mo, sp + 0x10, 0x40);
        m.call(0x0022_cc00, &[sp + 0x10, c], &[rate]);
        res = sp + 0x10;
    }
    copy(m, vo, res, 0x40);
    stack_pop(m, 0xb0);
}

/// FUN_00230a98: walk the channel chain `c` .. `end` (parent +0xa0) at time `t` accumulating the
/// matrix `pm` (identity when 0) and its velocity `pv` (zero when 0): v = L * v + V * m, m = L * m
/// (FUN_00230710 + FUN_0011c168); written back to the non-null outputs
pub fn ee_sub_230a98<M: QMem + ?Sized>(m: &mut M, t: f32, mut c: u32, end: u32, pm: u32, pv: u32) {
    let sp = stack_push(m, 0x180);
    let (l, v, mm, vv, x, y) = (sp, sp + 0x40, sp + 0x80, sp + 0xc0, sp + 0x100, sp + 0x140);
    if pv == 0 {
        for k in (0..0x40).step_by(4) {
            m.w32(vv + k, 0);
        }
    } else {
        copy(m, vv, pv, 0x40);
    }
    if pm == 0 {
        for k in (0..0x40).step_by(4) {
            m.w32(mm + k, 0);
        }
        for k in 0..4 {
            m.wf(mm + k * 0x14, 1.0);
        }
    } else {
        copy(m, mm, pm, 0x40);
    }
    while c != end {
        ee_sub_230710(m, t, c, l, v);
        m.call(0x0011_c168, &[x, l, vv], &[]);
        m.call(0x0011_c168, &[y, v, mm], &[]);
        for k in 0..4 {
            let (a, b) = (m.rv(x + 0x10 * k), m.rv(y + 0x10 * k));
            m.wv(vv + 0x10 * k, [a[0] + b[0], a[1] + b[1], a[2] + b[2], a[3] + b[3]]);
        }
        m.call(0x0011_c168, &[x, l, mm], &[]);
        copy(m, mm, x, 0x40);
        c = m.r32(c + 0xa0);
    }
    if pm != 0 {
        copy(m, pm, mm, 0x40);
    }
    if pv != 0 {
        copy(m, pv, vv, 0x40);
    }
    stack_pop(m, 0x180);
}

/// FUN_00230cd0: FUN_00230a98 at t = 0
pub fn ee_sub_230cd0<M: QMem + ?Sized>(m: &mut M, c: u32, end: u32, pm: u32, pv: u32) {
    ee_sub_230a98(m, 0.0, c, end, pm, pv)
}

/// FUN_00230cf0: per-frame binder pass of anim set `o` (skipped while FUN_00131968(o)): every live
/// node of +0x34 (next +0x28, FUN_0012cee0) binds its channel (FUN_0022dd00(owner +4, channel +8));
/// modes 0 / 0xd skip, 7 / 8 / 9 / 0xb need `force`; the channel's pose (sub-mode +0x9a: 0
/// FUN_0022ca18, 1 curve FUN_00117120, 2..4 +0xc0, 5 FUN_0011bff0(+0xc0), else DAT_002a2dd0)
/// becomes its matrix (FUN_0022c8c0), rate +0xb0 = DAT_002a2dd0, stamp +0xa4 = ++DAT_003e11e8;
/// up to 16 distinct owners then get slot +0x90/+0x94, and every +0x38 listener +0x10/+0x14
pub fn ee_sub_230cf0<M: QMem + ?Sized>(m: &mut M, o: u32, force: u32) {
    if m.call(0x0013_1968, &[o], &[]).0 != 0 {
        return;
    }
    let sp = stack_push(m, 0x60);
    let (cur, mat) = (sp, sp + 0x10);
    let mut owners: Vec<u32> = Vec::new();
    let mut n = m.r32(o + 0x34);
    while n != 0 {
        if m.call(0x0012_cee0, &[n], &[]).0 == 0 {
            n = m.r32(n + 0x28);
            continue;
        }
        let ch = m.r32(n + 8);
        let ow = m.r32(n + 4);
        m.call(0x0022_dd00, &[ow, ch], &[]);
        let mode = m.r16(ch + 0x98);
        let go = match mode {
            0 | 0xd => false,
            7 | 8 | 9 | 0xb => force != 0,
            _ => true,
        };
        if go {
            if !owners.contains(&ow) && owners.len() < 0x10 {
                owners.push(ow);
            }
            let q = match m.r16(ch + 0x9a) {
                0 => m.call(0x0022_ca18, &[ch, ch], &[]).1,
                1 => {
                    let t0 = m.rf(ch + 0x80);
                    let a = m.rf(ch + 0x84) - t0;
                    let (pb, pc) = (m.r32(ch + 0x88), m.r32(ch + 0x8c));
                    let b = m.rf(pb) - t0;
                    let cc = m.rf(pc);
                    let k = [m.rv(ch + 0xc0), m.rv(ch + 0xd0), m.rv(ch + 0xe0), m.rv(ch + 0xf0)];
                    m.call(0x0011_7120, &[cur, 0, 0], &[f4(a), f4(b), f4(cc), k[0], k[1], k[2], k[3]]);
                    m.rv(cur)
                }
                2..=4 => m.rv(ch + 0xc0),
                5 => m.call(0x0011_bff0, &[ch + 0xc0], &[]).1,
                _ => m.rv(0x002a_2dd0),
            };
            m.call(0x0022_c8c0, &[mat, ch], &[q]);
            copy(m, ch, mat, 0x40);
            let r = m.rv(0x002a_2dd0);
            m.wv(ch + 0xb0, r);
            let s = m.r32(0x003e_11e8).wrapping_add(1);
            m.w32(0x003e_11e8, s);
            m.w32(ch + 0xa4, s);
        }
        n = m.r32(n + 0x28);
    }
    stack_pop(m, 0x60);
    for ow in owners {
        let vt = m.r32(ow + 0xc);
        vcall(m, ow, vt, 0x90, &[], &[]);
    }
    let mut l = m.r32(o + 0x38);
    while l != 0 {
        let obj = m.r32(l);
        let vt = m.r32(obj);
        let a1 = m.r32(l + 4);
        let (f0, f1) = (m.rf(o + 0x1c), m.rf(o + 0x40));
        vcall(m, obj, vt, 0x10, &[a1, 0], &[f4(f0), f4(f1)]);
        l = m.r32(l + 0x10);
    }
}
