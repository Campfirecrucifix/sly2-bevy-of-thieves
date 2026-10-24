//! WIRE-W1C: 1:1 queue ports (against `QMem`) of a batch of live-frontier natives, ported from the EE asm;
//! callees go through `m.call` / `vslot` (the live dispatch routes ported ones). QMem convention: integer /
//! pointer args in `a`, quadword (128-bit GPR) and float args in `v`.
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

const T: u32 = 0x002d_ded0; // g_clock t
const CM: u32 = 0x002d_df40; // camera manager
const GS: u32 = 0x003d_d490; // game state / world

fn clamp(x: f32, lo: f32, hi: f32) -> f32 {
    if x < lo {
        lo
    } else if hi < x {
        hi
    } else {
        x
    }
}

/// FUN_00148c08: FUN_001f31e0(a0); FUN_00148738(a0 + 0x150, a0).
pub fn ee_sub_148c08<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001f_31e0, &[a0], &[]);
    m.call(0x0014_8738, &[a0 + 0x150, a0], &[]);
}

/// FUN_001492b0: FUN_001f2d48(a0); +0x1e0 in 1..=2 -> FUN_001490a0(a0, 0).
pub fn ee_sub_1492b0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001f_2d48, &[a0], &[]);
    if m.r32(a0 + 0x1e0).wrapping_sub(1) < 2 {
        m.call(0x0014_90a0, &[a0, 0], &[]);
    }
}

/// FUN_00148c38: FUN_001f36a8(a0); FUN_00148740(a0 + 0x150); FUN_001490a0(a0, 0).
pub fn ee_sub_148c38<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001f_36a8, &[a0], &[]);
    m.call(0x0014_8740, &[a0 + 0x150], &[]);
    m.call(0x0014_90a0, &[a0, 0], &[]);
}

/// FUN_00149358: FUN_00227bb8(a0); FUN_00148738(a0 + 0x130, a0); +0x2c8 != 0 -> +0x1d4 = 1.
pub fn ee_sub_149358<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0022_7bb8, &[a0], &[]);
    m.call(0x0014_8738, &[a0 + 0x130, a0], &[]);
    if m.r32(a0 + 0x2c8) != 0 {
        m.w32(a0 + 0x1d4, 1);
    }
}

/// FUN_00184130(o, a1): +0x10c = a1; vslot 3(6, o); FUN_0020ff70(GS, +0x118, &+0x11c, o, 1);
/// FUN_00161f48(0x2deb30, +0x15c, &+0x160, o); vslot 2(5, 0, 0, 0, 0); +0x114 -> FUN_001dfd58(+0x114, a1).
pub fn ee_sub_184130<M: QMem + ?Sized>(m: &mut M, o: u32, a1: u32) {
    m.w32(o + 0x10c, a1);
    vslot(m, o, 3, &[6, o], &[]);
    let k = m.r32(o + 0x118);
    m.call(0x0020_ff70, &[GS, k, o + 0x11c, o, 1], &[]);
    let k = m.r32(o + 0x15c);
    m.call(0x0016_1f48, &[0x002d_eb30, k, o + 0x160, o], &[]);
    vslot(m, o, 2, &[5, 0, 0, 0, 0], &[]);
    let s = m.r32(o + 0x114);
    if s != 0 {
        m.call(0x001d_fd58, &[s, a1], &[]);
    }
}

/// FUN_00184330(o, a1): FUN_00184130(o, a1); FUN_00132b10(a1, 0x36) -> FUN_001b7000(a1, o).
pub fn ee_sub_184330<M: QMem + ?Sized>(m: &mut M, o: u32, a1: u32) {
    m.call(0x0018_4130, &[o, a1], &[]);
    if m.call(0x0013_2b10, &[a1, 0x36], &[]).0 != 0 {
        m.call(0x001b_7000, &[a1, o], &[]);
    }
}

/// FUN_001cf338(o, tgt (128-bit a1), a2): camera-ish follow step. +0x1c == 0 -> 1; dur +0x18 <= 0 -> 0;
/// u = (t - +0x14) / +0x18; f21 = clamp(+0x30 + u(+0x34 + u +0x38), DAT_002bbb60, DAT_002bbb64);
/// f20 = +0x20 + u(+0x24 + u +0x28); d = tgt - cm[+0x90]; FUN_0014f850(CM, d, bits(f20 * 100)) == 0 -> 1;
/// FUN_0014f898(CM, d, &r, *(+0x1c)[+0x38]) == 0 -> 0; f21 < 1e-4 && f20 < 1e-4 -> 1; else
/// FUN_0011c288(&mat, tgt, a2, f20), FUN_001a5ee8(+0x1c, &copy, f21 * r, 1.0) -> 1.
pub fn ee_sub_1cf338<M: QMem + ?Sized>(m: &mut M, o: u32, tgt: V4, a2: u32) -> u32 {
    if m.r32(o + 0x1c) == 0 {
        return 1;
    }
    let dur = m.rf(o + 0x18);
    if dur <= 0.0 {
        return 0;
    }
    let u = (m.rf(T) - m.rf(o + 0x14)) / dur;
    let x = m.rf(o + 0x30) + u * (m.rf(o + 0x34) + u * m.rf(o + 0x38));
    let f21 = clamp(x, m.rf(0x002b_bb60), m.rf(0x002b_bb64));
    let f20 = m.rf(o + 0x20) + u * (m.rf(o + 0x24) + u * m.rf(o + 0x28));
    let c = m.rv(CM + 0x90);
    let d = [tgt[0] - c[0], tgt[1] - c[1], tgt[2] - c[2], tgt[3] - c[3]];
    let bits = (f20 * 100.0).to_bits();
    if m.call(0x0014_f850, &[CM, bits], &[d]).0 == 0 {
        return 1;
    }
    let sp = stack_push(m, 0xa0);
    let k = m.r32(o + 0x1c);
    let f12 = m.rf(k + 0x38);
    let ok = m.call(0x0014_f898, &[CM, sp + 0x90], &[d, f4(f12)]).0;
    let r = if ok == 0 {
        0
    } else if f21 < 1e-4 && f20 < 1e-4 {
        1
    } else {
        m.call(0x0011_c288, &[sp + 0x50, a2], &[tgt, f4(f20)]);
        let f12 = f21 * m.rf(sp + 0x90);
        for i in 0..16 {
            let w = m.r32(sp + 0x50 + i * 4);
            m.w32(sp + 0x10 + i * 4, w);
        }
        let k = m.r32(o + 0x1c);
        m.call(0x001a_5ee8, &[k, sp + 0x10], &[f4(f12), f4(1.0)]);
        1
    };
    stack_pop(m, 0xa0);
    r
}

/// FUN_001c4f60(o): FUN_001f36a8(o); +0x16c = FUN_0015ed78(GS, 1, 0x12, o) (non-null -> its vslot 8);
/// the oid slots +0x1a4/+0x1ac/+0x1b4/+0x204/+0x20c/+0x214 (!= -1) resolved through FUN_0015e938(GS, 4, oid, o)
/// into the following word; vslot 75(+0x158).
pub fn ee_sub_1c4f60<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001f_36a8, &[o], &[]);
    let v = m.call(0x0015_ed78, &[GS, 1, 0x12, o], &[]).0;
    m.w32(o + 0x16c, v);
    if v != 0 {
        vslot(m, v, 8, &[], &[]);
    }
    for off in [0x1a4, 0x1ac, 0x1b4, 0x204, 0x20c, 0x214] {
        let oid = m.r32(o + off);
        if oid != u32::MAX {
            let r = m.call(0x0015_e938, &[GS, 4, oid, o], &[]).0;
            m.w32(o + off + 4, r);
        }
    }
    let a = m.r32(o + 0x158);
    vslot(m, o, 75, &[a], &[]);
}

/// FUN_001d0190(o): distance-driven camera step. d = cm[+0x90] - o[+0x70]; f22 = |d.xyz| * 0.001;
/// u = (t - +0x14) / +0x18; f23 = (+0x20 + u(+0x24 + u +0x28)) * clamp(poly(DAT_002c1fb0, f22), DAT_002c1fc0..);
/// FUN_0014f850(CM, -d, bits(f23 * 100)) == 0 -> FUN_001ce980(o); else f20 = clamp(+0x30.., DAT_002bbb60..) *
/// clamp(poly(DAT_002c1fd0, f22), DAT_002c1fe0..); FUN_0011b830(&b, d, DAT_002a2dc0); FUN_0011b500(&r, DAT_002a2da0,
/// poly(DAT_002c1ff0, u)); rows r rotated by basis b; FUN_0011c288(&mat, o[+0x70], &rot, f23);
/// FUN_001a5ee8(+0x1c, &copy, f20, 1.0).
pub fn ee_sub_1d0190<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let t = m.rf(T);
    let c = m.rv(CM + 0x90);
    let p = m.rv(o + 0x70);
    let d = [c[0] - p[0], c[1] - p[1], c[2] - p[2], c[3] - p[3]];
    let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    let f22 = len * f32::from_bits(0x3a83_126f);
    let f21 = (t - m.rf(o + 0x14)) / m.rf(o + 0x18);
    let x = m.rf(0x002c_1fb0) + f22 * (m.rf(0x002c_1fb4) + f22 * m.rf(0x002c_1fb8));
    let k = clamp(x, m.rf(0x002c_1fc0), m.rf(0x002c_1fc4));
    let f2 = m.rf(o + 0x20) + f21 * (m.rf(o + 0x24) + f21 * m.rf(o + 0x28));
    let f23 = f2 * k;
    let nd = [-d[0], -d[1], -d[2], -d[3]];
    let bits = (f23 * 100.0).to_bits();
    if m.call(0x0014_f850, &[CM, bits], &[nd]).0 == 0 {
        m.call(0x001c_e980, &[o], &[]);
        return;
    }
    let x = m.rf(o + 0x30) + f21 * (m.rf(o + 0x34) + f21 * m.rf(o + 0x38));
    let f20 = clamp(x, m.rf(0x002b_bb60), m.rf(0x002b_bb64));
    let x = m.rf(0x002c_1fd0) + f22 * (m.rf(0x002c_1fd4) + f22 * m.rf(0x002c_1fd8));
    let k2 = clamp(x, m.rf(0x002c_1fe0), m.rf(0x002c_1fe4));
    let sp = stack_push(m, 0x140);
    let up = m.rv(0x002a_2dc0);
    m.call(0x0011_b830, &[sp + 0xe0], &[d, up]);
    let f20 = f20 * k2;
    let f12 = m.rf(0x002c_1ff0) + f21 * (m.rf(0x002c_1ff4) + f21 * m.rf(0x002c_1ff8));
    for i in 0..12 {
        let w = m.r32(sp + 0xe0 + i * 4);
        m.w32(sp + i * 4, w);
    }
    let ax = m.rv(0x002a_2da0);
    m.call(0x0011_b500, &[sp + 0xd0], &[ax, f4(f12)]);
    let v8 = m.rv(sp + 0xd0);
    let v7 = m.rv(sp + 0xe0);
    let v6 = m.rv(sp + 0xf0);
    let r0 = m.rv(sp);
    let r1 = m.rv(sp + 0x10);
    let r2 = m.rv(sp + 0x20);
    // xyz = r0*v.x + r1*v.y + r2*v.z; w keeps the old VF register (vf03 = d; vf04 / vf05 unknown -> 0)
    let xf = |v: V4, w: f32| -> V4 {
        let mut o = [w; 4];
        for i in 0..3 {
            o[i] = r0[i] * v[0] + r1[i] * v[1] + r2[i] * v[2];
        }
        o
    };
    let v5 = xf(v6, 0.0);
    let v4 = xf(v8, 0.0);
    let v3 = xf(v7, d[3]);
    m.wv(sp + 0x60, v4);
    m.wv(sp + 0x70, v3);
    m.wv(sp + 0x80, v5);
    m.wv(sp + 0x30, v8);
    m.wv(sp + 0x40, v7);
    m.wv(sp + 0x50, v6);
    m.wv(sp + 0x110, v4);
    m.wv(sp + 0xd0, v4);
    m.wv(sp + 0x120, v3);
    m.wv(sp + 0xe0, v3);
    m.wv(sp + 0x130, v5);
    m.wv(sp + 0xf0, v5);
    let q = m.rv(o + 0x70);
    m.call(0x0011_c288, &[sp + 0xd0, sp + 0x60], &[q, f4(f23)]);
    for i in 0..16 {
        let w = m.r32(sp + 0xd0 + i * 4);
        m.w32(sp + 0x90 + i * 4, w);
    }
    let k = m.r32(o + 0x1c);
    m.call(0x001a_5ee8, &[k, sp + 0x90], &[f4(f20), f4(1.0)]);
    stack_pop(m, 0x140);
}

/// FUN_00141588(jt, out): +0xf54 != -1 -> FUN_0019e9d8(jt + 0xf50, out); else out = pos (FUN_001a62e8) with
/// z: +0x208 -> +0x238 + +0x1048; else when +0x2dc != 0x14, cm[+0x300] == 0 and cm[+0x260] == jt -> +0x104c;
/// otherwise z += +0x248 + +0x1048.
pub fn ee_sub_141588<M: QMem + ?Sized>(m: &mut M, jt: u32, out: u32) {
    if m.r32(jt + 0xf54) != u32::MAX {
        m.call(0x0019_e9d8, &[jt + 0xf50, out], &[]);
        return;
    }
    let p = m.call(0x001a_62e8, &[jt], &[]).1;
    m.wv(out, p);
    if m.r32(jt + 0x208) != 0 {
        let z = m.rf(jt + 0x238) + m.rf(jt + 0x1048);
        m.wf(out + 8, z);
        return;
    }
    if m.r32(jt + 0x2dc) != 0x14 && m.r32(CM + 0x300) == 0 && m.r32(CM + 0x260) == jt {
        let z = m.rf(jt + 0x104c);
        m.wf(out + 8, z);
        return;
    }
    let f0 = m.rf(jt + 0x1048);
    let z = m.rf(out + 8) + (m.rf(jt + 0x248) + f0);
    m.wf(out + 8, z);
}

/// FUN_001e1c40(o): vslot 0; +0x30 == 0 -> +0x30 = +0x28 and step (FUN_001e1e30; FUN_001e5778 != 0 -> vslot 1,
/// done); scan: cur = +0x30; cur <= +0x2c and (cur[+4] != 0 or FUN_001e3890(o, cur) != 0) -> FUN_001e1d48 and
/// step again; else FUN_001e1d48, +0x2c < +0x30 -> FUN_001e5628(o, o); vslot 1.
pub fn ee_sub_1e1c40<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let sp = stack_push(m, 0x10);
    m.w32(sp, o);
    vslot(m, o, 0, &[], &[]);
    let mut step = false;
    if m.r32(o + 0x30) == 0 {
        let v = m.r32(o + 0x28);
        m.w32(o + 0x30, v);
        step = true;
    }
    loop {
        if step {
            m.call(0x001e_1e30, &[o], &[]);
            if m.call(0x001e_5778, &[o], &[]).0 != 0 {
                let s = m.r32(sp);
                vslot(m, s, 1, &[], &[]);
                break;
            }
        }
        step = true;
        let cur = m.r32(o + 0x30);
        let again = if m.r32(o + 0x2c) < cur {
            false
        } else if m.r32(cur + 4) != 0 {
            true
        } else {
            m.call(0x001e_3890, &[o, cur], &[]).0 != 0
        };
        m.call(0x001e_1d48, &[o], &[]);
        if again {
            continue;
        }
        if m.r32(o + 0x2c) < m.r32(o + 0x30) {
            let s = m.r32(sp);
            m.call(0x001e_5628, &[o, s], &[]);
        }
        let s = m.r32(sp);
        vslot(m, s, 1, &[], &[]);
        break;
    }
    stack_pop(m, 0x10);
}

/// FUN_001dfeb8(o): +0x74 == 0 -> nothing. Clears +0x6c/+0x70/+0x74; walks the list +0x5c (link offset
/// +0x64) under a safe-iterator frame pushed on DAT_002dd2fc, FUN_001e5628(each); FUN_001e0120(o); +0x78 ->
/// FUN_001a6ca8(o, 2) and bit 1 of the +0x40 flags (word or pointed word) cleared; vslot 2(6, 0, 0, 0, 0);
/// iterator frame popped.
pub fn ee_sub_1dfeb8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.r32(o + 0x74) == 0 {
        return;
    }
    const IT: u32 = 0x002d_d2fc;
    m.w32(o + 0x6c, 0);
    m.w32(o + 0x70, 0);
    m.w32(o + 0x74, 0);
    let sp = stack_push(m, 0x10);
    let l = o + 0x5c;
    m.w32(sp + 4, l);
    m.w32(sp, l);
    let prev = m.r32(IT);
    let off = m.r32(l + 8);
    m.w32(IT, sp);
    m.w32(sp + 0xc, prev);
    let mut a = m.r32(o + 0x5c);
    m.w32(sp + 8, off);
    m.w32(sp + 4, a.wrapping_add(off));
    while a != 0 {
        m.call(0x001e_5628, &[a], &[]);
        let nx = m.r32(sp + 4);
        let off = m.r32(sp + 8);
        a = m.r32(nx);
        m.w32(sp + 4, a.wrapping_add(off));
    }
    m.call(0x001e_0120, &[o], &[]);
    if m.r32(o + 0x78) != 0 {
        m.call(0x001a_6ca8, &[o, 2], &[]);
        let f = m.r32(o + 0x40);
        if f & 1 == 0 {
            let v = m.r32(f) & !2;
            m.w32(f, v);
        } else {
            m.w32(o + 0x40, f & !2);
        }
    }
    vslot(m, o, 2, &[6, 0, 0, 0, 0], &[]);
    let prev = m.r32(sp + 0xc);
    m.w32(IT, prev);
    stack_pop(m, 0x10);
}

/// FUN_00133040(base, owner, cnt, f12): ++*cnt; n = FUN_0011ccc0(base + 0x39fc); n = {f12, owner, cnt,
/// id (owner == 0 -> ++GS[+0x3d6c], else -1), 0}; inserted into the f-sorted list DAT_003e0e80 via
/// FUN_0011a0c0(&list, after, n) (after = last node whose key <= f12); returns n.
pub fn ee_sub_133040<M: QMem + ?Sized>(m: &mut M, a0: u32, owner: u32, cnt: u32, f12: f32) -> u32 {
    let c = m.r32(cnt).wrapping_add(1);
    m.w32(cnt, c);
    let n = m.call(0x0011_ccc0, &[a0 + 0x39fc], &[]).0;
    m.w32(n + 4, owner);
    let id = if owner == 0 {
        let v = m.r32(GS + 0x3d6c).wrapping_add(1);
        m.w32(GS + 0x3d6c, v);
        v
    } else {
        u32::MAX
    };
    m.w32(n + 0xc, id);
    m.w32(n + 8, cnt);
    m.wf(n, f12);
    m.w32(n + 0x10, 0);
    const L: u32 = 0x003e_0e80;
    let head = m.r32(L);
    let mut after = 0;
    if head != 0 && !(f12 < m.rf(head)) {
        after = head;
        loop {
            let nx = m.r32(after + 0x10);
            if nx == 0 || m.rf(n) < m.rf(nx) {
                break;
            }
            after = nx;
        }
    }
    m.call(0x0011_a0c0, &[L, after, n], &[]);
    n
}

/// FUN_00136298(o): FUN_00224610(+0x186c) != 0 -> (+0x186c)[+0x16c] = o, FUN_0018d698(o, +0x16b4, +0x186c);
/// FUN_001cb4b8((+0x16b4)[+4], +0x186c); +0x16b4 = +0x186c = 0.
pub fn ee_sub_136298<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let x = m.r32(o + 0x186c);
    if m.call(0x0022_4610, &[x], &[]).0 != 0 {
        let x = m.r32(o + 0x186c);
        m.w32(x + 0x16c, o);
        let a = m.r32(o + 0x16b4);
        let x = m.r32(o + 0x186c);
        m.call(0x0018_d698, &[o, a, x], &[]);
    }
    let a = m.r32(o + 0x16b4);
    let b = m.r32(o + 0x186c);
    let a4 = m.r32(a + 4);
    m.call(0x001c_b4b8, &[a4, b], &[]);
    m.w32(o + 0x16b4, 0);
    m.w32(o + 0x186c, 0);
}

/// FUN_00136118(o): pickup / carry state loop. s = +0x16b0: 0 -> FUN_00131968(+0x1030) != 0 -> FUN_00136040(o, 1);
/// 1..=2 -> hold = +0x186c: hold[+0x158] == 2 -> FUN_00224258(hold, hold[+0x1a0]) then release; == 4 -> vslot 75(1)
/// on hold then release; else FUN_001c4728(+0x150, FUN_00141ea8(o, *(+0x16b4)) | 0x10) == 0 -> vslot 75 + release,
/// != 0 -> FUN_00136040(o, t - +0x20c > 0.1 ? 2 : 1). release = FUN_00136298(o), FUN_00195a00(o + 0x2c0, 0xb, 0, 0).
/// Repeats while +0x16b0 changed and +0x13d0 == 6.
pub fn ee_sub_136118<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let mut s2 = m.r32(o + 0x16b0) as i32;
    loop {
        if s2 == 0 {
            let a = m.r32(o + 0x1030);
            if m.call(0x0013_1968, &[a], &[]).0 != 0 {
                m.call(0x0013_6040, &[o, 1], &[]);
            }
        } else if s2 > 0 && s2 < 3 {
            let s1 = m.r32(o + 0x186c);
            let v = m.r32(s1 + 0x158);
            let mut slot = false;
            let mut release = false;
            if v == 2 {
                let a = m.r32(s1 + 0x1a0);
                m.call(0x0022_4258, &[s1, a], &[]);
                release = true;
            } else if v == 4 {
                slot = true;
            } else {
                let p = m.r32(o + 0x16b4);
                let a = m.r32(p);
                let r = m.call(0x0014_1ea8, &[o, a], &[]).0;
                let b = m.r32(o + 0x150);
                if m.call(0x001c_4728, &[b, r | 0x10], &[]).0 != 0 {
                    let d = m.rf(T) - m.rf(o + 0x20c);
                    let k = if 0.1f32 < d { 2 } else { 1 };
                    m.call(0x0013_6040, &[o, k], &[]);
                } else {
                    slot = true;
                }
            }
            if slot {
                vslot(m, s1, 75, &[1], &[]);
                release = true;
            }
            if release {
                m.call(0x0013_6298, &[o], &[]);
                m.call(0x0019_5a00, &[o + 0x2c0, 0xb, 0, 0], &[]);
            }
        }
        let v = m.r32(o + 0x16b0) as i32;
        if v == s2 || m.r32(o + 0x13d0) != 6 {
            break;
        }
        s2 = m.r32(o + 0x16b0) as i32;
    }
}

/// FUN_0011e4b0(a (128-bit a0), b (128-bit a1), axis_out): angle between a and b. |a| or |b| < 1e-4 -> axis_out
/// (if any) = DAT_002a2da0, 0.0. With axis_out: c = a x b, normalised when |c| > 1.0e-8 (0x322bcc77), else
/// c = FUN_0011e390(a); returns FUN_002456f0(dot(a, b) / (|a| |b|)) (acos).
pub fn ee_sub_11e4b0<M: QMem + ?Sized>(m: &mut M, a: V4, b: V4, out: u32) -> f32 {
    let la = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
    let lb = (b[0] * b[0] + b[1] * b[1] + b[2] * b[2]).sqrt();
    let eps = f32::from_bits(0x38d1_b717);
    if la < eps || lb < eps {
        if out != 0 {
            let v = m.rv(0x002a_2da0);
            m.wv(out, v);
        }
        return 0.0;
    }
    if out != 0 {
        // vopmula/vopmsub: xyz = cross; w keeps the old vf03 (unknown -> 0)
        let c = [a[1] * b[2] - b[1] * a[2], a[2] * b[0] - b[2] * a[0], a[0] * b[1] - b[0] * a[1], 0.0];
        let l = (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt();
        let v = if f32::from_bits(0x322b_cc77) < l {
            let q = 1.0 / l;
            [c[0] * q, c[1] * q, c[2] * q, c[3] * q]
        } else {
            m.call(0x0011_e390, &[], &[a]).1
        };
        m.wv(out, v);
    }
    let d = a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    m.call(0x0024_56f0, &[], &[f4(d / (la * lb))]).1[0]
}

/// FUN_001fb778(a0, p (128-bit a1), range, out_k, out_deg): ref = FUN_001a62e8(FUN_001c4a48(GS)) or cm[+0x90];
/// d = p - ref; k = FUN_0011d948(&DAT_002bbb60, (range[1] - |d|) / (range[1] - range[0])); out_k (opt) = k;
/// out_deg (opt): k > 0 -> int(deg(atan2(-r.y, r.x) (+ pi when range[1] < range[0]))) with
/// r = FUN_001a7348(0, &cm[+0x60], d), else 0.
pub fn ee_sub_1fb778<M: QMem + ?Sized>(m: &mut M, _a0: u32, p: V4, range: u32, out_k: u32, out_deg: u32) {
    let v = m.call(0x001c_4a48, &[GS], &[]).0;
    let r = if v != 0 { m.call(0x001a_62e8, &[v], &[]).1 } else { m.rv(CM + 0x90) };
    let d = [p[0] - r[0], p[1] - r[1], p[2] - r[2], p[3] - r[3]];
    let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    let (r0, r1) = (m.rf(range), m.rf(range + 4));
    let f12 = (r1 - len) / (r1 - r0);
    let k = m.call(0x0011_d948, &[0x002b_bb60], &[f4(f12)]).1[0];
    if out_k != 0 {
        m.wf(out_k, k);
    }
    if out_deg == 0 {
        return;
    }
    if 0.0 < k {
        let rv = m.call(0x001a_7348, &[0, CM + 0x60], &[d]).1;
        let mut a = m.call(0x0024_58f0, &[], &[f4(-rv[1]), f4(rv[0])]).1[0];
        if m.rf(range + 4) < m.rf(range) {
            a += f32::from_bits(0x4049_0fdb);
        }
        let deg = (a * f32::from_bits(0x4265_2ee0)) as i32;
        m.w32(out_deg, deg as u32);
    } else {
        m.w32(out_deg, 0);
    }
}

/// FUN_00210740(a0, id, oid, ix, iy, def, ctx, _t3, out (stack arg), f12, f13, f14): id == -1 -> nothing.
/// tgt = oid != -1 ? FUN_0015e938(a0, 4, oid, ctx) : def; tgt is-a 0x38 with tgt[+0xfc4] -> f12 +=
/// clamp(poly(DAT_002c3ea0, (+0xfc4)[+0x20]), DAT_002c3eb0, DAT_002c3eb4); f12 += a0[+0x475c];
/// s = FUN_001fbb90(id); FUN_001fb2f8(s, tgt); FUN_001fb4f0(s, ix, iy as floats); FUN_001fb1f8(s, f14);
/// FUN_001fb2c0(s, f12); FUN_001fb520(s, f13); vslot 7(s) == 2 -> FUN_00258040(out, s).
#[allow(clippy::too_many_arguments)]
pub fn ee_sub_210740<M: QMem + ?Sized>(
    m: &mut M,
    a0: u32,
    id: u32,
    oid: u32,
    ix: u32,
    iy: u32,
    def: u32,
    ctx: u32,
    _t3: u32,
    out: u32,
    f12: f32,
    f13: f32,
    f14: f32,
) {
    if id == u32::MAX {
        return;
    }
    let mut f20 = f12;
    let tgt = if oid == u32::MAX { def } else { m.call(0x0015_e938, &[a0, 4, oid, ctx], &[]).0 };
    if tgt != 0 && m.call(0x0013_2b10, &[tgt, 0x38], &[]).0 != 0 {
        let p = m.r32(tgt + 0xfc4);
        if p != 0 {
            let x = m.rf(p + 0x20);
            let y = m.rf(0x002c_3ea0) + x * (m.rf(0x002c_3ea4) + x * m.rf(0x002c_3ea8));
            f20 += clamp(y, m.rf(0x002c_3eb0), m.rf(0x002c_3eb4));
        }
    }
    f20 += m.rf(a0 + 0x475c);
    let s = m.call(0x001f_bb90, &[id], &[]).0;
    m.call(0x001f_b2f8, &[s, tgt], &[]);
    m.call(0x001f_b4f0, &[s], &[f4(ix as i32 as f32), f4(iy as i32 as f32)]);
    m.call(0x001f_b1f8, &[s], &[f4(f14)]);
    m.call(0x001f_b2c0, &[s], &[f4(f20)]);
    m.call(0x001f_b520, &[s], &[f4(f13)]);
    if vslot(m, s, 7, &[], &[]).0 == 2 {
        m.call(0x0025_8040, &[out, s], &[]);
    }
}

/// FUN_001733f8(o): FUN_001a5c48(o); for each of the +0x50 entries (stride 0x1c at +0x54): kind 0 / 2 ->
/// +8 = FUN_0015e938(GS, 5, +4, 0); kind 1 -> the same, then e = FUN_0013e3c8(0x71, 0, 0x2cb, -1),
/// FUN_00170c50(e, +4, 1, -1), +0xc = e, +0x10 = e[+0x54]. +0x60 = FUN_001ae2a0(&{+0x5c * 0x30, 0x10});
/// FUN_001a6ca8(o, 1); bit 1 of the +0x40 flags (word or pointed word) set.
pub fn ee_sub_1733f8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001a_5c48, &[o], &[]);
    let mut i = 0i32;
    while i < m.r32(o + 0x50) as i32 {
        let e = m.r32(o + 0x54).wrapping_add(i as u32 * 0x1c);
        let k = m.r32(e) as i32;
        if k == 1 {
            let a = m.r32(e + 4);
            let v = m.call(0x0015_e938, &[GS, 5, a, 0], &[]).0;
            m.w32(e + 8, v);
            let s0 = m.call(0x0013_e3c8, &[0x71, 0, 0x2cb, u32::MAX], &[]).0;
            let a = m.r32(e + 4);
            m.call(0x0017_0c50, &[s0, a, 1, u32::MAX], &[]);
            let v = m.r32(s0 + 0x54);
            m.w32(e + 0xc, s0);
            m.w32(e + 0x10, v);
        } else if k == 0 || k == 2 {
            let a = m.r32(e + 4);
            let v = m.call(0x0015_e938, &[GS, 5, a, 0], &[]).0;
            m.w32(e + 8, v);
        }
        i += 1;
    }
    let sp = stack_push(m, 0x10);
    let n = m.r32(o + 0x5c).wrapping_mul(0x30);
    m.w32(sp + 4, 0x10);
    m.w32(sp, n);
    let v = m.call(0x001a_e2a0, &[sp], &[]).0;
    stack_pop(m, 0x10);
    m.w32(o + 0x60, v);
    m.call(0x001a_6ca8, &[o, 1], &[]);
    let f = m.r32(o + 0x40);
    if f & 1 == 0 {
        let v = m.r32(f) | 2;
        m.w32(f, v);
    } else {
        m.w32(o + 0x40, f | 2);
    }
}

/// vtable-at-+0 call `(*(*obj + 0xc))(obj + *(short*)(*obj + 8), a1, a2)` (the load-list sink objects)
fn vcall0<M: QMem + ?Sized>(m: &mut M, obj: u32, a1: u32, a2: u32) {
    let vt = m.r32(obj);
    let adj = m.r16(vt + 8) as i16 as i32 as u32;
    let f = m.r32(vt + 0xc);
    m.call(f, &[obj.wrapping_add(adj), a1, a2], &[]);
}

/// FUN_0013fb40(parent, bis, sink): load the object list of a level stream. cnt = FUN_00117d50(bis); per entry:
/// k = FUN_00117d00 (k > 0 or k < -2 -> FUN_00117968(bis, 2k, &name)); cls = d50, oid = db8, cls2 = d50, skip = db8.
/// k < -2 / k > 0: FUN_0013fad0(k, &name) != 0 -> create; else cls 0x84/0x85 -> o = FUN_0013e3c8(cls, parent, oid,
/// -1), o[+0x50] = 1, vslot 13(bis), sink(1, &o); other cls -> FUN_001179b8(bis, skip). k -2 / -1: a local
/// proxy sink {DAT_002cbe38, flag, FUN_001c9940(GS[+0x3f10], -1)} after FUN_0015e938 / FUN_0015e958(GS, 0x405 /
/// 0x805, oid, 0, FUN_001c99d8(GS[+0x3f0c], 1, &h)) found nothing (found -> skip). k == 0 -> create.
/// create: FUN_0015e988(cls, 6) -> FUN_001c9b08(GS, parent, cls, oid, cls2, bis, sink); else o = FUN_0013e3c8(cls,
/// parent, oid, cls2), vslot 13(bis), sink(1, &o).
pub fn ee_sub_13fb40<M: QMem + ?Sized>(m: &mut M, parent: u32, bis: u32, fp: u32) {
    let sp = stack_push(m, 0x130);
    m.w32(sp + 0x11c, parent);
    m.w32(sp, 0x002c_be38);
    m.w32(sp + 8, u32::MAX);
    m.w32(sp + 4, u32::MAX);
    let cnt = m.call(0x0011_7d50, &[bis], &[]).0 as i32;
    if cnt > 0 {
        m.w32(sp + 0x120, cnt as u32);
        loop {
            let s0 = m.call(0x0011_7d00, &[bis], &[]).0 as i32;
            if s0 > 0 || s0 < -2 {
                m.call(0x0011_7968, &[bis, (s0 << 1) as u32, sp + 0x10], &[]);
            }
            let mut s2 = fp;
            let s3 = m.call(0x0011_7d50, &[bis], &[]).0;
            let s4 = m.call(0x0011_7db8, &[bis], &[]).0;
            let s6 = m.call(0x0011_7d50, &[bis], &[]).0;
            let s7 = m.call(0x0011_7db8, &[bis], &[]).0;
            // 0 = create, 1 = skip (FUN_001179b8), 2 = nothing more
            let act = if s0 < -2 || s0 > 0 {
                if m.call(0x0013_fad0, &[s0 as u32, sp + 0x10], &[]).0 != 0 {
                    0
                } else if s3.wrapping_sub(0x84) < 2 {
                    let a1 = m.r32(sp + 0x11c);
                    let o = m.call(0x0013_e3c8, &[s3, a1, s4, u32::MAX], &[]).0;
                    m.w32(sp + 0x114, o);
                    m.w32(o + 0x50, 1);
                    let o = m.r32(sp + 0x114);
                    vslot(m, o, 13, &[bis], &[]);
                    if fp != 0 {
                        vcall0(m, fp, 1, sp + 0x114);
                    }
                    2
                } else {
                    1
                }
            } else if s0 < 0 {
                let a = m.r32(GS + 0x3f10);
                let v = m.call(0x001c_9940, &[a, u32::MAX], &[]).0;
                m.w32(sp + 0x110, v);
                let a = m.r32(GS + 0x3f0c);
                let t0 = m.call(0x001c_99d8, &[a, 1, sp + 0x110], &[]).0;
                let (f, kind, flag) =
                    if s0 == -2 { (0x0015_e938, 0x405, 1) } else { (0x0015_e958, 0x805, 0) };
                if m.call(f, &[GS, kind, s4, 0, t0], &[]).0 != 0 {
                    1
                } else {
                    m.w32(sp + 4, flag);
                    let h = m.r32(sp + 0x110);
                    s2 = sp;
                    m.w32(sp + 8, h);
                    0
                }
            } else {
                0
            };
            if act == 0 {
                if m.call(0x0015_e988, &[s3, 6], &[]).0 != 0 {
                    let a1 = m.r32(sp + 0x11c);
                    m.call(0x001c_9b08, &[GS, a1, s3, s4, s6, bis, s2], &[]);
                } else {
                    let a1 = m.r32(sp + 0x11c);
                    let o = m.call(0x0013_e3c8, &[s3, a1, s4, s6], &[]).0;
                    m.w32(sp + 0x118, o);
                    vslot(m, o, 13, &[bis], &[]);
                    if s2 != 0 {
                        vcall0(m, s2, 1, sp + 0x118);
                    }
                }
            } else if act == 1 {
                m.call(0x0011_79b8, &[bis, s7], &[]);
            }
            let c = m.r32(sp + 0x120).wrapping_sub(1);
            m.w32(sp + 0x120, c);
            if c == 0 {
                break;
            }
        }
    }
    stack_pop(m, 0x130);
}

/// FUN_0019d540(b): binoc targets by the JT class (*(*b)): is-a 0x35 -> oids 0x35f / 0x368, 0x37 -> 0x360 / 0x369,
/// 0x36 -> 0x361 / 0x36a, resolved with FUN_0015e938(GS, 0x405, oid, 0) into +8 / +0xc; +0x14 ->
/// FUN_00201dc8(+0x14); FUN_0019dca0(b, +0xc).
pub fn ee_sub_19d540<M: QMem + ?Sized>(m: &mut M, b: u32) {
    let mut ids = None;
    for (k, x, y) in [(0x35, 0x35f, 0x368), (0x37, 0x360, 0x369), (0x36, 0x361, 0x36a)] {
        let p = m.r32(b);
        let jt = m.r32(p);
        if m.call(0x0013_2b10, &[jt, k], &[]).0 != 0 {
            ids = Some((x, y));
            break;
        }
    }
    if let Some((x, y)) = ids {
        let v = m.call(0x0015_e938, &[GS, 0x405, x, 0], &[]).0;
        m.w32(b + 8, v);
        let v = m.call(0x0015_e938, &[GS, 0x405, y, 0], &[]).0;
        m.w32(b + 0xc, v);
    }
    let a = m.r32(b + 0x14);
    if a != 0 {
        m.call(0x0020_1dc8, &[a], &[]);
    }
    let c = m.r32(b + 0xc);
    m.call(0x0019_dca0, &[b, c], &[]);
}

/// FUN_0019d660(b, st): binoc HUD state. Same state -> nothing; old != -1 -> FUN_0019d948(b); +4 = st.
/// 0: event 0xc (kind 1) / 7 via FUN_001619a0(0x2deb30, .., 0); +8: vslot 48(-1), FUN_00213b00(+8, 0x352 / 0x351),
/// FUN_00212e40(+8, 0x352, (jt[+0x1a8] + pi) / 2pi), FUN_00212e40(+8, 0x351, (*b)[+0x3c]); kind 1 ->
/// FUN_00227788(0x4435f0) and +0x10 vslot 48(-1); +0x14 -> FUN_00202768(+0x14, 0).
/// 2: event 0xd / 8; +8 / +0x10 vslot 49; +0x14 -> FUN_00202768(+0x14, 0xa).
/// -1: (old != 2 -> event 0xd / 8); +8 / +0x10 with [+0x58] -> FUN_00212a70(x, 1), FUN_00212a50(x, 1);
/// +0x14 -> FUN_00202768(+0x14, -1).
pub fn ee_sub_19d660<M: QMem + ?Sized>(m: &mut M, b: u32, st: u32) {
    const EV: u32 = 0x002d_eb30;
    let old = m.r32(b + 4);
    if old == st {
        return;
    }
    let old = if old != u32::MAX {
        old
    } else {
        m.call(0x0019_d948, &[b], &[]);
        m.r32(b + 4)
    };
    m.w32(b + 4, st);
    let kind1 = |m: &mut M| {
        let p = m.r32(b);
        m.r32(p + 0x20) == 1
    };
    match st as i32 {
        0 => {
            let e = if kind1(m) { 0xc } else { 7 };
            m.call(0x0016_19a0, &[EV, e, 0], &[]);
            let h = m.r32(b + 8);
            if h != 0 {
                vslot(m, h, 48, &[u32::MAX], &[]);
                let h = m.r32(b + 8);
                m.call(0x0021_3b00, &[h, 0x352], &[]);
                let h = m.r32(b + 8);
                m.call(0x0021_3b00, &[h, 0x351], &[]);
                let p = m.r32(b);
                let q = m.r32(p);
                let f = (m.rf(q + 0x1a8) + f32::from_bits(0x4049_0fdb)) * f32::from_bits(0x3e22_f983);
                let h = m.r32(b + 8);
                m.call(0x0021_2e40, &[h, 0x352], &[f4(f)]);
                let p = m.r32(b);
                let f = m.rf(p + 0x3c);
                let h = m.r32(b + 8);
                m.call(0x0021_2e40, &[h, 0x351], &[f4(f)]);
            }
            if kind1(m) {
                m.call(0x0022_7788, &[0x0044_35f0], &[]);
                let h = m.r32(b + 0x10);
                if h != 0 {
                    vslot(m, h, 48, &[u32::MAX], &[]);
                }
            }
            let a = m.r32(b + 0x14);
            if a != 0 {
                m.call(0x0020_2768, &[a, 0], &[]);
            }
        }
        2 => {
            let e = if kind1(m) { 0xd } else { 8 };
            m.call(0x0016_19a0, &[EV, e, 0], &[]);
            for off in [8, 0x10] {
                let h = m.r32(b + off);
                if h != 0 {
                    vslot(m, h, 49, &[h], &[]);
                }
            }
            let a = m.r32(b + 0x14);
            if a != 0 {
                m.call(0x0020_2768, &[a, 0xa], &[]);
            }
        }
        -1 => {
            if old != 2 {
                let e = if kind1(m) { 0xd } else { 8 };
                m.call(0x0016_19a0, &[EV, e, 0], &[]);
            }
            for off in [8, 0x10] {
                let h = m.r32(b + off);
                if h != 0 && m.r32(h + 0x58) != 0 {
                    m.call(0x0021_2a70, &[h, 1], &[]);
                    let h = m.r32(b + off);
                    m.call(0x0021_2a50, &[h, 1], &[]);
                }
            }
            let a = m.r32(b + 0x14);
            if a != 0 {
                m.call(0x0020_2768, &[a, u32::MAX], &[]);
            }
        }
        _ => {}
    }
}

/// FUN_001b6728(jt, flags): +0x13d0 not 0 / 0x10 with +0x4cc -> +0x1080 = t; FUN_00141640(jt, flags); by
/// +0x13d0: 3 / 9 / 12 -> *flags |= 1; 11 -> (+0x10e4 == 4 -> *flags |= 1); 15 -> FUN_00220388(jt + 0x1910, flags);
/// 20 -> FUN_0019ea00(jt + 0xf50, flags).
pub fn ee_sub_1b6728<M: QMem + ?Sized>(m: &mut M, jt: u32, fl: u32) {
    let s = m.r32(jt + 0x13d0);
    if s != 0 && s != 0x10 && m.r32(jt + 0x4cc) != 0 {
        let t = m.rf(T);
        m.wf(jt + 0x1080, t);
    }
    m.call(0x0014_1640, &[jt, fl], &[]);
    match m.r32(jt + 0x13d0) {
        3 | 9 | 12 => {
            let v = m.r32(fl) | 1;
            m.w32(fl, v);
        }
        11 => {
            if m.r32(jt + 0x10e4) == 4 {
                let v = m.r32(fl) | 1;
                m.w32(fl, v);
            }
        }
        15 => {
            m.call(0x0022_0388, &[jt + 0x1910, fl], &[]);
        }
        20 => {
            m.call(0x0019_ea00, &[jt + 0xf50, fl], &[]);
        }
        _ => {}
    }
}
