//! WIRE-W1G1: 1:1 queue ports (against `QMem`) of a batch of EE natives, ported from the asm
//! (tools/eedis.py). Callees go through `m.call` / `vslot`; integer / pointer args in `a` (register order,
//! quadword registers omitted), float args then quadword (128-bit GPR) args in `v`; a float result is
//! `.1[0]`, a quadword result `.1`. VU0 macro-mode blocks are emulated inline lane by lane.
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

const GS: u32 = 0x003d_d490; // game state / world
const CM: u32 = 0x002d_df40; // camera manager
const EV: u32 = 0x002d_eb30; // FUN_001619a0 event sink
const T_NOW: u32 = 0x002d_ded0; // frame time
const T_DT: u32 = 0x002d_ded4; // frame dt
const V_ZERO: u32 = 0x002a_2dd0; // zero quadword

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

fn fb(b: u32) -> f32 {
    f32::from_bits(b)
}

/// 32-bit GPR value moved into a quadword (`qmtc2` of an `lw` / `mfc1` result): low word, sign-extended
/// upper word; the top 64 bits are undefined in the asm, modelled as 0
fn gq(lo: u32) -> V4 {
    let hi = if lo & 0x8000_0000 != 0 { u32::MAX } else { 0 };
    [fb(lo), fb(hi), 0.0, 0.0]
}

fn gqf(x: f32) -> V4 {
    gq(x.to_bits())
}

/// `vsqrt` of x*x + y*y + z*z (the `vmul.xyz / vaddw / vadday / vmaddz / vsqrt` idiom)
fn len3(a: V4) -> f32 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

fn scl(a: V4, s: f32) -> V4 {
    [a[0] * s, a[1] * s, a[2] * s, a[3] * s]
}

/// `vopmula ACC, a, b; vopmsub d, b, a`: d.xyz = a x b (as the VU computes it), d.w undefined (kept 0)
fn opm(a: V4, b: V4) -> V4 {
    [a[1] * b[2] - b[1] * a[2], a[2] * b[0] - b[2] * a[0], a[0] * b[1] - b[0] * a[1], 0.0]
}

/// `vmula.xyzw ACC, r0, v.x; vmadda r1, v.y; vmadda r2, v.z; vmadd d, r3, w` (xyzw)
fn mat4(r0: V4, r1: V4, r2: V4, r3: V4, v: V4, w: f32) -> V4 {
    let mut o = [0.0; 4];
    for i in 0..4 {
        o[i] = r0[i] * v[0] + r1[i] * v[1] + r2[i] * v[2] + r3[i] * w;
    }
    o
}

fn cpn<M: QMem + ?Sized>(m: &mut M, dst: u32, src: u32, n: u32) {
    for k in (0..n).step_by(4) {
        let w = m.r32(src + k);
        m.w32(dst + k, w);
    }
}

/// FUN_00179ba0(o): FUN_001eef98(o); busy = FUN_00209098(+0x10); state machine on +0x38 driving
/// FUN_00179d30 until FUN_001ef0a8(o) != 0 or the state stops changing.
pub fn ee_sub_179ba0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef98, &[o], &[]);
    let a = m.r32(o + 0x10);
    let s2 = m.call(0x0020_9098, &[a], &[]).0;
    loop {
        if m.call(0x001e_f0a8, &[o], &[]).0 != 0 {
            return;
        }
        let s1 = m.r32(o + 0x38) as i32;
        match s1 {
            1 => {
                if s2 != 0 {
                    m.call(0x0017_9d30, &[o, 3], &[]);
                } else if m.rf(o + 0x34) < m.rf(T_NOW) {
                    m.call(0x0017_9d30, &[o, 0], &[]);
                }
            }
            0 => {
                if s2 != 0 {
                    m.call(0x0017_9d30, &[o, 3], &[]);
                } else if m.rf(o + 0x30) < m.rf(T_NOW) {
                    m.call(0x0017_9d30, &[o, 1], &[]);
                } else {
                    let a = m.r32(o + 0x18);
                    if m.call(0x0017_1458, &[a], &[]).0 != 0 {
                        m.w32(o + 0x40, 0);
                        m.call(0x0017_9d30, &[o, 2], &[]);
                    }
                }
            }
            2 => {
                if s2 != 0 {
                    m.call(0x0017_9d30, &[o, 3], &[]);
                }
            }
            3 => {
                if s2 == 0 {
                    if m.r32(o + 0x40) != 0 {
                        m.call(0x0017_9d30, &[o, 1], &[]);
                    } else {
                        m.call(0x0017_9d30, &[o, 2], &[]);
                    }
                }
            }
            _ => {}
        }
        if m.r32(o + 0x38) as i32 == s1 {
            return;
        }
    }
}

/// FUN_00173610(pos, any_slope, filter, a, b): probe segment pos + dir*a .. pos + dir*b (dir = 0x2a2dc0)
/// through FUN_0020faa0 / FUN_0014cc58; on a hit (normal.y != 0, and slope >= 0.7 unless any_slope)
/// *pos = hit point, return 1; else 0.
pub fn ee_sub_173610<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, f12: f32, f13: f32) -> u32 {
    let sp = stack_push(m, 0xb0);
    m.call(0x001a_e478, &[sp], &[]);
    let dir = m.rv(0x002a_2dc0);
    let p = m.rv(a0);
    let lerp = |k: f32| [p[0] + dir[0] * k, p[1] + dir[1] * k, p[2] + dir[2] * k, p[3] + 1.0 + dir[3] * k];
    let end = lerp(f13);
    let start = lerp(f12);
    m.wv(sp + 0xa0, end);
    m.wv(sp + 0x90, start);
    let filt = if a2 == 0 { 0x0020_fa50 } else { a2 };
    m.call(0x0020_faa0, &[GS, filt, 0, sp + 0x80, sp + 0x84], &[start, end]);
    let a3 = m.r32(sp + 0x80);
    let t0 = m.r32(sp + 0x84);
    let hit = m.call(0x0014_cc58, &[1, a3, t0, sp + 0x20], &[start, end]).0;
    let ok = hit != 0 && m.rf(sp + 0x60) != 0.0 && (a1 != 0 || !(m.rf(sp + 0x48) < 0.7));
    let r = if ok {
        let v = m.rv(sp + 0x20);
        m.wv(a0, v);
        m.call(0x001a_e4a0, &[sp, 2], &[]);
        1
    } else {
        m.call(0x001a_e4a0, &[sp, 2], &[]);
        0
    };
    stack_pop(m, 0xb0);
    r
}

/// shared body of FUN_002550f8 / FUN_00255770: unlink node `a` from a packed 13+2-bit prev/next list
/// (`tab` = per-bank base table, `sz` = node size, `cur` / `head` / `tail` = list globals).
fn unlink_packed<M: QMem + ?Sized>(m: &mut M, a: u32, tab: u32, sz: u32, cur: u32, head: u32, tail: u32) {
    let w = m.r32(a + 4);
    let (lo, bk) = (w & 0x1fff, (w >> 13) & 3);
    let t1 = if lo == 0 && bk == 0 { 0 } else { m.r32(tab + bk.wrapping_mul(4)).wrapping_add(lo.wrapping_mul(sz)) };
    let w = m.r32(a + 4);
    let (lo, bk) = ((w >> 16) & 0x1fff, (w >> 29) & 3);
    let t0 = if lo == 0 && bk == 0 { 0 } else { m.r32(tab + bk.wrapping_mul(4)).wrapping_add(lo.wrapping_mul(sz)) };
    if a == m.r32(cur) {
        m.w32(cur, t0);
    }
    if t1 != 0 {
        let h = (m.r16(a + 6) as u32 & 0x1fff) << 16;
        let v = (m.r32(t1 + 4) & 0xe000_ffff) | h;
        m.w32(t1 + 4, v);
        let v = (v & 0x9fff_ffff) | (m.r32(a + 4) & 0x6000_0000);
        m.w32(t1 + 4, v);
    } else if a == m.r32(head) {
        m.w32(head, t0);
    } else if a == m.r32(tail) {
        m.w32(tail, t0);
    }
    if t0 != 0 {
        let v = (m.r32(t0 + 4) & 0xffff_e000) | (m.r32(a + 4) & 0x1fff);
        m.w32(t0 + 4, v);
        let v = (v & 0xffff_9fff) | (m.r32(a + 4) & 0x6000);
        m.w32(t0 + 4, v);
    }
}

/// FUN_002550f8(node): unlink from the 0x14-byte-node list (0x452098 banks; 0x2dd280/288/28c).
pub fn ee_sub_2550f8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    unlink_packed(m, a0, 0x0045_2098, 0x14, 0x002d_d288, 0x002d_d280, 0x002d_d28c);
}

/// FUN_00255770(node): unlink from the 0x28-byte-node list (0x4520e8 banks; 0x2dd2a0/2a8/2ac).
pub fn ee_sub_255770<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    unlink_packed(m, a0, 0x0045_20e8, 0x28, 0x002d_d2a8, 0x002d_d2a0, 0x002d_d2ac);
}

/// FUN_001c7ee8(o, st): st != +0x150 -> (st == 1: notify +0x210 (slot 49 {1, o}) or FUN_001fb2f8(
/// FUN_001fbb90(0x29, &msg), 0); FUN_00156b98(GS, 0, pos, o); slot 2(5,0,0,0,0); linked actor with
/// flag 7 -> FUN_001c7c30(+0x100, 1), slot 8, FUN_001618b0(GS); st = 2); FUN_001c60b8(o, st).
pub fn ee_sub_1c7ee8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if a1 == m.r32(a0 + 0x150) {
        return;
    }
    let mut st = a1;
    if a1 == 1 {
        let sp = stack_push(m, 0x10);
        let a2 = m.r32(a0 + 0x210);
        if a2 != 0 {
            m.w32(sp, a1);
            m.w32(sp + 4, a0);
            vslot(m, a2, 49, &[sp], &[]);
        } else {
            let v = m.call(0x001f_bb90, &[0x29, sp], &[]).0;
            m.call(0x001f_b2f8, &[v, 0], &[]);
        }
        stack_pop(m, 0x10);
        let pos = m.call(0x001a_62e8, &[a0], &[]).1;
        st = 2;
        m.call(0x0015_6b98, &[GS, 0, a0], &[pos]);
        vslot(m, a0, 2, &[5, 0, 0, 0, 0], &[]);
        let x = m.call(0x001c_b658, &[a0], &[]).0;
        if x != 0 && m.call(0x0013_2b10, &[x, 7], &[]).0 != 0 {
            let s0 = m.call(0x001c_b658, &[a0], &[]).0;
            let a = m.r32(s0 + 0x100);
            m.call(0x001c_7c30, &[a, 1], &[]);
            vslot(m, s0, 8, &[], &[]);
            m.call(0x0016_18b0, &[GS], &[]);
        }
    }
    m.call(0x001c_60b8, &[a0, st], &[]);
}

/// FUN_001fb528(o): +0x1c not 3D-ready (+0x28 bit 60 clear) -> FUN_001fb098(0); else k = +0x10 *
/// FUN_001fb778 factor; volume change (x1024) -> slot 14 mute/unmute, +0x44 = k, FUN_001fb658(o, 0x10,
/// 0x10); FUN_001fb268(o, pan).
pub fn ee_sub_1fb528<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let v = m.r32(o + 0x1c);
    if v != 0 && (m.r64(v + 0x28) >> 60) & 1 == 0 {
        m.call(0x001f_b098, &[o], &[f4(0.0)]);
        return;
    }
    let a = m.r32(o + 0x1c);
    let q = m.rv(o + 0x20);
    let p = m.call(0x001a_6f00, &[a, 0], &[q]).1;
    let sp = stack_push(m, 0x10);
    m.call(0x001f_b778, &[o, o + 0x30, sp, sp | 4], &[p]);
    let f20 = m.rf(o + 0x10) * m.rf(sp);
    let f4v = m.rf(o + 0x44);
    if (f20 * 1024.0) as i32 != (f4v * 1024.0) as i32 {
        if f4v == 0.0 {
            vslot(m, o, 14, &[0], &[]);
        } else if f20 == 0.0 {
            vslot(m, o, 14, &[1], &[]);
        }
        m.wf(o + 0x44, f20);
        m.call(0x001f_b658, &[o, 0x10, 0x10], &[]);
    }
    let pan = m.r32(sp + 4);
    stack_pop(m, 0x10);
    m.call(0x001f_b268, &[o, pan], &[]);
}

/// FUN_0011f088(a2, a3, from (128-bit a0), to (128-bit a1), f12) -> quadword: d = to - from, len = |d|;
/// len > 1e-4 -> t = FUN_0011d018(a2, &v, 0, len, f12); a3 -> *a3 = d * (v / len); from + d * (t / len)
/// (w + 1 from the vf00 add); else a3 -> *a3 = 0; to.
pub fn ee_sub_11f088<M: QMem + ?Sized>(m: &mut M, a2: u32, a3: u32, a0: V4, a1: V4, f12: f32) -> V4 {
    let d = [a1[0] - a0[0], a1[1] - a0[1], a1[2] - a0[2], a1[3] - a0[3]];
    let len = len3(d);
    if 0.0001 < len {
        let sp = stack_push(m, 0x10);
        let t = m.call(0x0011_d018, &[a2, sp], &[f4(0.0), f4(len), f4(f12)]).1[0];
        if a3 != 0 {
            let k = m.rf(sp) / len;
            m.wv(a3, scl(d, k));
        }
        stack_pop(m, 0x10);
        let k = t / len;
        [a0[0] + d[0] * k, a0[1] + d[1] * k, a0[2] + d[2] * k, a0[3] + 1.0 + d[3] * k]
    } else {
        if a3 != 0 {
            let z = m.rv(V_ZERO);
            m.wv(a3, z);
        }
        a1
    }
}

/// FUN_0012ff10(tab, actor, id, need_flag) -> entry: first 8-byte entry of +0x60[+0x5c] whose low 20
/// bits == actor+0x18, bit31 clear, (need_flag -> bit30 set) and slot 49(field) +0x94 == id; else 0.
pub fn ee_sub_12ff10<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32) -> u32 {
    let mut i: i32 = 0;
    while i < m.r32(a0 + 0x5c) as i32 {
        let e = m.r32(a0 + 0x60).wrapping_add((i as u32) << 3);
        if a3 == 0 || m.r32(e) & 0x4000_0000 != 0 {
            let w = m.r32(e);
            if ((w << 12) as i32 >> 12) as u32 == m.r32(a1 + 0x18) && w & 0x8000_0000 == 0 {
                let f = ((w << 2) as i32 >> 22) as u32;
                let r = vslot(m, a1, 49, &[f], &[]).0;
                if m.r32(r + 0x94) == a2 {
                    return e;
                }
            }
        }
        i += 1;
    }
    0
}

/// FUN_00229370(o, ev, who): ev == 1 && who != o -> unlink who's node from the +0x150 list (+0xc next,
/// +4 owner), msg {o, node owner}; node -> clear next, FUN_00229738(owner, node), FUN_002296c0(GS, node);
/// owner slot 3(5, &msg). Then FUN_00133590(o, ev, who).
pub fn ee_sub_229370<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    if a1 == 1 && a2 != a0 {
        let mut s0 = m.r32(a0 + 0x150);
        if s0 != 0 {
            if m.r32(s0 + 4) == a2 {
                let n = m.r32(s0 + 0xc);
                m.w32(a0 + 0x150, n);
            } else {
                let mut p = a0 + 0x150;
                loop {
                    let v1 = m.r32(p);
                    s0 = m.r32(v1 + 0xc);
                    if s0 == 0 {
                        break;
                    }
                    p = v1 + 0xc;
                    if m.r32(s0 + 4) == a2 {
                        let n = m.r32(s0 + 0xc);
                        m.w32(v1 + 0xc, n);
                        break;
                    }
                }
            }
        }
        let sp = stack_push(m, 0x10);
        m.w32(sp, a0);
        let own = m.r32(s0.wrapping_add(4));
        m.w32(sp + 4, own);
        if s0 != 0 {
            m.w32(s0 + 0xc, 0);
            let a = m.r32(s0 + 4);
            m.call(0x0022_9738, &[a, s0], &[]);
            m.call(0x0022_96c0, &[GS, s0], &[]);
        }
        let a3 = m.r32(sp + 4);
        vslot(m, a3, 3, &[5, sp], &[]);
        stack_pop(m, 0x10);
    }
    m.call(0x0013_3590, &[a0, a1, a2], &[]);
}

/// FUN_0021c0d8(o): FUN_001a5e28(o); fade +0x12c += dt * rate +0x130, clamped to [0, 1] (rate cleared at
/// either end); faded out with +0x164 >= +0x158 -> slot 8.
pub fn ee_sub_21c0d8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001a_5e28, &[o], &[]);
    let f2 = m.rf(o + 0x130);
    if f2 != 0.0 {
        let f1 = m.rf(o + 0x12c) + m.rf(T_DT) * f2;
        m.wf(o + 0x12c, f1);
        if f1 <= 0.0 {
            m.wf(o + 0x130, 0.0);
            m.wf(o + 0x12c, 0.0);
        } else if 1.0 <= f1 {
            m.wf(o + 0x12c, 1.0);
            m.wf(o + 0x130, 0.0);
        }
    }
    let f0 = m.rf(o + 0x12c);
    if f0 <= 0.0 && m.rf(o + 0x130) <= 0.0 && !((m.r32(o + 0x164) as i32) < m.r32(o + 0x158) as i32) {
        vslot(m, o, 8, &[], &[]);
    }
}

/// FUN_00154080(o, st, new): st && new != st+4 -> (st+4 == 0 -> st+0xc = t); slot 2(0x1c, 0, 3, {st,
/// &st+4, &new}, 0); st+4 = new (possibly rewritten), st+8 = t; new == 1 with +0x10 flagged 0x33 ->
/// FUN_001619a0(EV, 0x13, +0x10).
pub fn ee_sub_154080<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    if a1 == 0 {
        return;
    }
    let v = m.r32(a1 + 4);
    if a2 == v {
        return;
    }
    if v == 0 {
        let t = m.rf(T_NOW);
        m.wf(a1 + 0xc, t);
    }
    let sp = stack_push(m, 0x20);
    m.w32(sp + 0x10, a2);
    m.w32(sp, a1);
    m.w32(sp + 4, a1 + 4);
    m.w32(sp + 8, sp + 0x10);
    vslot(m, a0, 2, &[0x1c, 0, 3, sp, 0], &[]);
    let v1 = m.r32(sp + 0x10);
    stack_pop(m, 0x20);
    m.w32(a1 + 4, v1);
    let t = m.rf(T_NOW);
    m.wf(a1 + 8, t);
    if v1 == 1 {
        let s0 = m.r32(a0 + 0x10);
        if s0 != 0 && m.call(0x0013_2b10, &[s0, 0x33], &[]).0 != 0 {
            m.call(0x0016_19a0, &[EV, 0x13, s0], &[]);
        }
    }
}

/// FUN_00151130(o, a1, a2): stop sounds +0xc0 (FUN_001fb098(s, 0)) / +0xc4 (0.5) and FUN_00258040(&h, 0);
/// clear +0x10/+0x14/+0x40, +0x18 = -1; FUN_0014d8c0(CM); FUN_0014fce8(o, a1, a2).
pub fn ee_sub_151130<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    for (off, f) in [(0xc0u32, 0.0f32), (0xc4, 0.5)] {
        let s1 = a0.wrapping_add(off);
        if s1 != 0 {
            let h = m.r32(s1);
            if h != 0 {
                m.call(0x001f_b098, &[h], &[f4(f)]);
                m.call(0x0025_8040, &[s1, 0], &[]);
            }
        }
    }
    m.w32(a0 + 0x10, 0);
    m.w32(a0 + 0x14, 0);
    m.w32(a0 + 0x40, 0);
    m.wf(a0 + 0x18, -1.0);
    m.call(0x0014_d8c0, &[CM], &[]);
    m.call(0x0014_fce8, &[a0, a1, a2], &[]);
}

/// FUN_00210338(a0, on, a2, a3, f12) -> obj: obj = FUN_0015e938(a0, 4, a2, a3); f12 != 0 and obj flagged
/// 0x30 -> on ? FUN_001f7c68(obj, f12) : FUN_001f7bf8(obj, f12); else on ? slot 7 : slot 8.
pub fn ee_sub_210338<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, f12: f32) -> u32 {
    let s0 = m.call(0x0015_e938, &[a0, 4, a2, a3], &[]).0;
    if s0 != 0 {
        if f12 != 0.0 && m.call(0x0013_2b10, &[s0, 0x30], &[]).0 != 0 {
            if a1 != 0 {
                m.call(0x001f_7c68, &[s0], &[f4(f12)]);
            } else {
                m.call(0x001f_7bf8, &[s0], &[f4(f12)]);
            }
        } else if a1 != 0 {
            vslot(m, s0, 7, &[], &[]);
        } else {
            vslot(m, s0, 8, &[], &[]);
        }
    }
    s0
}

/// FUN_0012fe58(a0, a1, ref, out_obj, out_field): *out_obj = FUN_0015e938(GS, a1 ? 1 : 4, low20(*ref),
/// a1 ? a1 : a0); *out_field = obj ? slot 49(obj, bits 20..29 of *ref) : 0.
pub fn ee_sub_12fe58<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, t0: u32) {
    let id = ((m.r32(a2) << 12) as i32 >> 12) as u32;
    let v = if a1 != 0 {
        m.call(0x0015_e938, &[GS, 1, id, a1], &[]).0
    } else {
        m.call(0x0015_e938, &[GS, 4, id, a0], &[]).0
    };
    m.w32(a3, v);
    let o = m.r32(a3);
    if o == 0 {
        m.w32(t0, 0);
    } else {
        let f = ((m.r32(a2) << 2) as i32 >> 22) as u32;
        // vt+0x19c = slot 50 (8 * 50 + 0xc); ANIM-ALL: was slot 49 (vt+0x194) -> no bone ever bound
        let r = vslot(m, o, 50, &[f], &[]).0;
        m.w32(t0, r);
    }
}

/// FUN_00218660(o, i): slot e = +8 + i*0x18 live (e+8 >= 0) -> +0x1c = e+8, +0x18 = e+0xc, +0x10 =
/// *(u64*)0x2ddf18, +0x20 = -1, e+8 = -1, e+0x10 = 0, stop sound e+0x14; FUN_00217f38(o, i).
pub fn ee_sub_218660<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let e = a1.wrapping_mul(0x18).wrapping_add(m.r32(a0 + 8));
    let h = m.r16(e + 8) as i16 as i32;
    if h >= 0 {
        let f = m.r32(e + 0xc);
        m.w32(a0 + 0x1c, h as u32);
        m.w32(a0 + 0x18, f);
        let s1 = e.wrapping_add(0x14);
        let (lo, hi) = (m.r32(0x002d_df18), m.r32(0x002d_df1c));
        m.w32(a0 + 0x20, u32::MAX);
        m.w32(a0 + 0x10, lo);
        m.w32(a0 + 0x14, hi);
        m.w16(e + 8, 0xffff);
        m.w32(e + 0x10, 0);
        if s1 != 0 {
            let s = m.r32(e + 0x14);
            if s != 0 {
                m.call(0x001f_b098, &[s], &[f4(0.0)]);
                m.call(0x0025_8040, &[s1, 0], &[]);
            }
        }
    }
    m.call(0x0021_7f38, &[a0, a1], &[]);
}

/// FUN_00204e38(a0, a1, f12) -> quadword: out = 0; a1 && !FUN_00131968(a1) and e = FUN_0012ff10(a1+0x14,
/// a0, 1, 1) -> e+4 obj's vtable (at +0) slot +0x1c(obj, 0, &out, f12, a1+0x40); out.
pub fn ee_sub_204e38<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, f12: f32) -> V4 {
    let sp = stack_push(m, 0x10);
    let z = m.rv(V_ZERO);
    m.wv(sp, z);
    if a1 != 0 {
        if m.call(0x0013_1968, &[a1], &[]).0 != 0 {
            let r = m.rv(sp);
            stack_pop(m, 0x10);
            return r;
        }
        let t = m.r32(a1 + 0x14);
        let e = m.call(0x0012_ff10, &[t, a0, 1, 1], &[]).0;
        if e != 0 {
            let a3 = m.r32(e + 4);
            let f13 = m.rf(a1 + 0x40);
            let vt = m.r32(a3);
            let adj = m.r16(vt + 0x18) as i16 as u32;
            let f = m.r32(vt + 0x1c);
            m.call(f, &[a3.wrapping_add(adj), 0, sp], &[f4(f12), f4(f13)]);
        }
    }
    let r = m.rv(sp);
    stack_pop(m, 0x10);
    r
}

/// `v * (1 / |v|.xyz)` (xyzw)
fn unit(a: V4) -> V4 {
    scl(a, 1.0 / len3(a))
}

/// FUN_0011b3d0(mat): orthonormalize rows: r0 = |r0|; r2 = |r0 x r1|; r1 = |r2 x r0| (w lanes kept).
pub fn ee_sub_11b3d0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let r0 = m.rv(a0);
    let r1 = m.rv(a0 + 16);
    let a = unit(r0);
    let mut t = opm(a, r1);
    t[3] = r1[3];
    let c = unit(t);
    let mut u = opm(c, a);
    u[3] = a[3];
    m.wv(a0, a);
    m.wv(a0 + 32, c);
    m.wv(a0 + 16, unit(u));
}

/// FUN_00202030(o): collect up to 5 members of +0x34[+0x30] with +0xf8 != -1 and no / mode-1/2 (+0xec
/// +0x54) owner into +0x78[+0x74].
pub fn ee_sub_202030<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.w32(a0 + 0x74, 0);
    let mut i: i32 = 0;
    while i < m.r32(a0 + 0x30) as i32 {
        let a1 = m.r32(a0 + 0x34 + (i as u32) * 4);
        let ok = m.r32(a1 + 0xf8) != u32::MAX && {
            let v = m.r32(a1 + 0xec);
            v == 0 || matches!(m.r32(v + 0x54), 1 | 2)
        };
        if ok {
            let c = m.r32(a0 + 0x74);
            if c < 5 {
                m.w32(a0 + 0x78 + c.wrapping_mul(4), a1);
                m.w32(a0 + 0x74, c + 1);
            }
        }
        i += 1;
    }
}

/// FUN_00182780(o): FUN_001f50e0(o); per +0x1cc entry (0x30 stride, count +0x1c8): FUN_001e7bb8(e+0x24,
/// &a, &b); FUN_001f5480(o, e+0x20, a); FUN_001f5480(o, e+0x20, b).
pub fn ee_sub_182780<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001f_50e0, &[a0], &[]);
    let mut i: i32 = 0;
    if m.r32(a0 + 0x1c8) as i32 > 0 {
        let sp = stack_push(m, 0x20);
        let mut s0 = a0 + 0x1cc;
        loop {
            let a = m.r32(s0 + 0x24);
            m.call(0x001e_7bb8, &[a, sp, sp + 0x10], &[]);
            i += 1;
            let f = m.rf(s0 + 0x20);
            let q = m.rv(sp);
            m.call(0x001f_5480, &[a0], &[q, f4(f)]);
            let f = m.rf(s0 + 0x20);
            let q = m.rv(sp + 0x10);
            m.call(0x001f_5480, &[a0], &[q, f4(f)]);
            s0 += 0x30;
            if i >= m.r32(a0 + 0x1c8) as i32 {
                break;
            }
        }
        stack_pop(m, 0x20);
    }
}

/// FUN_0011d448(lo, hi) -> f32: lo == hi -> lo; else lo + (hi - lo) * (FUN_0011d3c0() % 0x16a1b) *
/// 1.0787603e-5.
pub fn ee_sub_11d448<M: QMem + ?Sized>(m: &mut M, f12: f32, f13: f32) -> f32 {
    if f12 == f13 {
        return f12;
    }
    let v = m.call(0x0011_d3c0, &[], &[]).0 as i32;
    let r = v.wrapping_rem(0x16a1b);
    f12 + (f13 - f12) * (r as f32) * fb(0x3734_fc67)
}

/// FUN_00258598(o, a1): +0xe4 = FUN_001ae1d0({0x100, 0x10}); FUN_00159118(+0xe4); FUN_00231170(o, a1);
/// +0xe4 kind 3 -> FUN_00159240(+0xe4 +0x30, a1); FUN_002584a8(o, a1).
pub fn ee_sub_258598<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let sp = stack_push(m, 0x10);
    m.w32(sp + 4, 0x10);
    m.w32(sp, 0x100);
    let v = m.call(0x001a_e1d0, &[sp], &[]).0;
    stack_pop(m, 0x10);
    m.w32(a0 + 0xe4, v);
    m.call(0x0015_9118, &[v], &[]);
    m.call(0x0023_1170, &[a0, a1], &[]);
    let x = m.r32(a0 + 0xe4);
    if m.r32(x + 0x10) == 3 {
        let a = m.r32(x + 0x30);
        m.call(0x0015_9240, &[a, a1], &[]);
    }
    m.call(0x0025_84a8, &[a0, a1], &[]);
}

/// FUN_001f1078(o, ev, who): ev == 2 && who == +0x30 -> slot 0; FUN_001f0fa0(o); FUN_001f0d88(o, 0); slot 1.
pub fn ee_sub_1f1078<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    if a1 == 2 && a2 == m.r32(a0 + 0x30) {
        vslot(m, a0, 0, &[], &[]);
        m.call(0x001f_0fa0, &[a0], &[]);
        m.call(0x001f_0d88, &[a0, 0], &[]);
        vslot(m, a0, 1, &[], &[]);
    }
}

/// shared body of FUN_00117c80 / FUN_00117e38: read a little-endian u32 from stream +4 (+8 bytes left);
/// short -> FUN_00117968(s, 4, &v, left).
fn stream_u32<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let a3 = m.r32(a0 + 8);
    if (a3 as i32) < 4 {
        let sp = stack_push(m, 0x10);
        m.call(0x0011_7968, &[a0, 4, sp, a3], &[]);
        let v = m.r32(sp);
        stack_pop(m, 0x10);
        v
    } else {
        let p = m.r32(a0 + 4);
        let b = [m.r8(p), m.r8(p + 1), m.r8(p + 2), m.r8(p + 3)];
        m.w32(a0 + 8, a3.wrapping_sub(4));
        m.w32(a0 + 4, p + 4);
        u32::from_le_bytes(b)
    }
}

/// FUN_00117c80(stream) -> u32 (see `stream_u32`).
pub fn ee_sub_117c80<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    stream_u32(m, a0)
}

/// FUN_001fb1f8(o, f12): v = FUN_0011d948(0x2bbb60, f12) (clamp); v != +0x10 (x1024) -> +0x10 = v,
/// FUN_001fb658(o, 0x10, 0x10).
pub fn ee_sub_1fb1f8<M: QMem + ?Sized>(m: &mut M, a0: u32, f12: f32) {
    let f = m.call(0x0011_d948, &[0x002b_bb60], &[f4(f12)]).1[0];
    if (f * 1024.0) as i32 != (m.rf(a0 + 0x10) * 1024.0) as i32 {
        m.wf(a0 + 0x10, f);
        m.call(0x001f_b658, &[a0, 0x10, 0x10], &[]);
    }
}

/// FUN_00231468(o) -> quadword: p = +0x50; p -> FUN_0022d348(p, 0, p's matrix (+0x40) * o+0x90 (xyz,
/// w undefined -> 0)); else zero quadword.
pub fn ee_sub_231468<M: QMem + ?Sized>(m: &mut M, a0: u32) -> V4 {
    let p = m.r32(a0 + 0x50);
    if p == 0 {
        return m.rv(V_ZERO);
    }
    let v = m.rv(a0 + 0x90);
    let (r0, r1, r2, r3) = (m.rv(p + 0x40), m.rv(p + 0x50), m.rv(p + 0x60), m.rv(p + 0x70));
    let mut x = mat4(r0, r1, r2, r3, v, 1.0);
    x[3] = 0.0;
    m.call(0x0022_d348, &[p, 0], &[x]).1
}

/// FUN_00164030(o) -> u32: +0x54 == -1 with +0xd8 -> FUN_002581c0(&tmp, o+0xd8, 0x3d4a60), r = tmp+4,
/// FUN_00129ec8(&tmp), r; else +0x54 >= 2.
pub fn ee_sub_164030<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let mut v = m.r32(a0 + 0x54) as i32;
    if v == -1 {
        if m.r32(a0 + 0xd8) != 0 {
            let sp = stack_push(m, 0x10);
            m.call(0x0025_81c0, &[sp, a0 + 0xd8, 0x003d_4a60], &[]);
            let s0 = m.r32(sp + 4);
            m.call(0x0012_9ec8, &[sp], &[]);
            stack_pop(m, 0x10);
            return s0;
        }
        v = m.r32(a0 + 0x54) as i32;
    }
    (v >= 2) as u32
}

/// FUN_001406e8(o, st): FUN_001c3ed8(o, st); FUN_001f82b8(o, st == 1 ? 0 : -1, 0); st == 1 ->
/// FUN_001eecd0(o + 0x26c).
pub fn ee_sub_1406e8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.call(0x001c_3ed8, &[a0, a1], &[]);
    let x = if a1 == 1 { 0 } else { u32::MAX };
    m.call(0x001f_82b8, &[a0, x, 0], &[]);
    if a1 == 1 {
        m.call(0x001e_ecd0, &[a0 + 0x26c], &[]);
    }
}

/// FUN_001512e0(o, a1, a2): FUN_0012e460(o + 0x14); FUN_0014d8c0(CM); FUN_0014fce8(o, a1, a2).
pub fn ee_sub_1512e0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    m.call(0x0012_e460, &[a0 + 0x14], &[]);
    m.call(0x0014_d8c0, &[CM], &[]);
    m.call(0x0014_fce8, &[a0, a1, a2], &[]);
}

/// FUN_00133448(list, key): find node (+8 chain, +0 key); --refcount (+4) <= 0 -> unlink and
/// FUN_0011cd98(0x3e1144, node).
pub fn ee_sub_133448<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let mut v1 = a0.wrapping_add(8);
    let n = loop {
        let n = m.r32(v1);
        if n == 0 {
            return;
        }
        if m.r32(n) == a1 {
            break n;
        }
        v1 = n + 8;
    };
    let v = m.r32(n + 4).wrapping_sub(1);
    m.w32(n + 4, v);
    if v as i32 <= 0 {
        let nx = m.r32(n + 8);
        m.w32(v1, nx);
        m.call(0x0011_cd98, &[0x003e_1144, n, a1], &[]);
    }
}

/// FUN_001ae9e8(a0): FUN_001d98b0(a0); FUN_0014d1d0(clock 0x2dded0, 0, 0.0); FUN_0020efc0(GS, 0);
/// FUN_001619a0(EV, 0x28, 0).
pub fn ee_sub_1ae9e8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001d_98b0, &[a0], &[]);
    m.call(0x0014_d1d0, &[T_NOW, 0], &[f4(0.0)]);
    m.call(0x0020_efc0, &[GS, 0], &[]);
    m.call(0x0016_19a0, &[EV, 0x28, 0], &[]);
}

/// FUN_001f67e0(id, sub) -> node: first node of the 0x3e126c list (+0x1c next) with +0x28 == id and
/// (sub == -1 or +0x24 +0x18 == sub); else 0.
pub fn ee_sub_1f67e0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let mut a2 = m.r32(0x003e_126c);
    while a2 != 0 {
        if m.r32(a2 + 0x28) == a0 {
            if a1 == u32::MAX {
                return a2;
            }
            let p = m.r32(a2 + 0x24);
            if m.r32(p + 0x18) == a1 {
                return a2;
            }
        }
        a2 = m.r32(a2 + 0x1c);
    }
    0
}

/// FUN_00151900(o, src): FUN_0014fcc8(o, src); FUN_0014f610(CM, 49.0); CM+0x24 = src+0x1bc.
pub fn ee_sub_151900<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.call(0x0014_fcc8, &[a0, a1], &[]);
    m.call(0x0014_f610, &[CM], &[f4(fb(0x4244_0000))]);
    let v = m.rf(a1 + 0x1bc);
    m.wf(CM + 0x24, v);
}

/// FUN_00151648(o, a1, tgt): FUN_0014fcc8(o, a1); +0x10 = tgt; tgt+0x14 slot 0.
pub fn ee_sub_151648<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    m.call(0x0014_fcc8, &[a0, a1], &[]);
    m.w32(a0 + 0x10, a2);
    let o = m.r32(a2 + 0x14);
    vslot(m, o, 0, &[], &[]);
}

/// FUN_00171fd0(o): FUN_001eef78(o); FUN_00172320(o, -1); FUN_00209970(+0x10, 0x1f); FUN_001725e0(o, 0).
pub fn ee_sub_171fd0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001e_ef78, &[a0], &[]);
    m.call(0x0017_2320, &[a0, u32::MAX], &[]);
    let a = m.r32(a0 + 0x10);
    m.call(0x0020_9970, &[a, 0x1f], &[]);
    m.call(0x0017_25e0, &[a0, 0], &[]);
}

/// FUN_0015c150(o): zeroed 0x40-byte msg -> slot 49(&msg).
pub fn ee_sub_15c150<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let sp = stack_push(m, 0x40);
    m.call(0x0023_33b4, &[sp, 0, 0x40], &[]);
    vslot(m, a0, 49, &[sp], &[]);
    stack_pop(m, 0x40);
}

/// FUN_001b7e40(jt, k): jump table 0x2a73f0 (k < 9) -> FUN_001b6078(jt, state) with optional
/// FUN_001b7ba8(jt) first.
pub fn ee_sub_1b7e40<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let (pre, st) = match a1 {
        0 => (false, if m.r32(a0 + 0x13f0) != 0 { 5 } else { 0 }),
        1 => {
            let v = m.r32(a0 + 0x2c4) as i32;
            (!(0xf..0x11).contains(&v), 2)
        }
        2 => (false, 0xd),
        3 => (true, 0xe),
        4 => (false, 0x10),
        5 => (true, 0x11),
        6 => (true, 0x12),
        7 => (true, 0x13),
        8 => (true, 0x14),
        _ => return,
    };
    if pre {
        m.call(0x001b_7ba8, &[a0], &[]);
    }
    m.call(0x001b_6078, &[a0, st], &[]);
}

/// FUN_001e5a98(o, v): +0x90 = v; +0x9c = 0; FUN_0014e4c8(CM, 2, 10, CM+0x500, 0, o+0x84).
pub fn ee_sub_1e5a98<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.w32(a0 + 0x90, a1);
    m.w32(a0 + 0x9c, 0);
    m.call(0x0014_e4c8, &[CM, 2, 10, CM + 0x500, 0, a0 + 0x84], &[]);
}

/// FUN_0023e290(f12): soft-float unpack FUN_0023e108(&f, &u) then FUN_0023df70(u0, u1, u2, (u64)u3 << 30)
/// (the 64-bit a3 is passed as a [lo, hi] quadword in `v`).
pub fn ee_sub_23e290<M: QMem + ?Sized>(m: &mut M, f12: f32) {
    let sp = stack_push(m, 0x20);
    m.wf(sp + 0x10, f12);
    m.call(0x0023_e108, &[sp + 0x10, sp], &[]);
    let w = (m.r32(sp + 0xc) as u64) << 30;
    let (a0, a1, a2) = (m.r32(sp), m.r32(sp + 4), m.r32(sp + 8));
    stack_pop(m, 0x20);
    m.call(0x0023_df70, &[a0, a1, a2], &[[fb(w as u32), fb((w >> 32) as u32), 0.0, 0.0]]);
}

/// FUN_00217ef8(o) -> index of the first free (+8 < 0) 0x18-byte slot of +8[+2]; else 0.
pub fn ee_sub_217ef8<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let n = m.r16(a0 + 2) as i32;
    let mut i = 0;
    if n != 0 {
        let mut p = m.r32(a0 + 8) + 8;
        loop {
            if (m.r16(p) as i16) < 0 {
                return i as u32;
            }
            i += 1;
            if i >= n {
                break;
            }
            p += 0x18;
        }
    }
    0
}

/// FUN_002581c0(out, a1, a2) -> out: FUN_001d7ed8(out, 0x3e0ea8, a1, a2).
pub fn ee_sub_2581c0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) -> u32 {
    m.call(0x001d_7ed8, &[a0, 0x003e_0ea8, a1, a2], &[]);
    a0
}

/// FUN_0011d4d0(a, b, eps) -> |a - b| / max(1, |a|) < eps.
pub fn ee_sub_11d4d0<M: QMem + ?Sized>(_m: &mut M, f12: f32, f13: f32, f14: f32) -> u32 {
    let d = (f12 - f13).abs() / 1.0f32.max(f12.abs());
    (d < f14) as u32
}

/// FUN_001291a8(dst, src) -> dst: copy 2 words; FUN_00129df8(dst, src).
pub fn ee_sub_1291a8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let v = m.r32(a1);
    m.w32(a0, v);
    let v = m.r32(a1 + 4);
    m.w32(a0 + 4, v);
    m.call(0x0012_9df8, &[a0, a1], &[]);
    a0
}

/// FUN_00258248(o): memset(o, 0, 8); +0xc = 0 (byte); +8 = -1.
pub fn ee_sub_258248<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0023_33b4, &[a0, 0, 8], &[]);
    m.w8(a0 + 0xc, 0);
    m.w32(a0 + 8, u32::MAX);
}

/// FUN_0021d320(o): FUN_001a6150(o); FUN_0011a2e8(0x3e13b4, o).
pub fn ee_sub_21d320<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_6150, &[a0], &[]);
    m.call(0x0011_a2e8, &[0x003e_13b4, a0], &[]);
}

/// FUN_001a34e0(o): FUN_001a6150(o); FUN_001a3e88(GS, o).
pub fn ee_sub_1a34e0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_6150, &[a0], &[]);
    m.call(0x001a_3e88, &[GS, a0], &[]);
}

/// FUN_00227e20(o): FUN_001a6178(o); FUN_0011a220(0x3e1480, o).
pub fn ee_sub_227e20<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_6178, &[a0], &[]);
    m.call(0x0011_a220, &[0x003e_1480, a0], &[]);
}

/// FUN_0011d910(range, x) -> range[0] < x < range[1].
pub fn ee_sub_11d910<M: QMem + ?Sized>(m: &mut M, a0: u32, f12: f32) -> u32 {
    (m.rf(a0) < f12 && f12 < m.rf(a0 + 4)) as u32
}

/// FUN_00181238(o): FUN_0017f6b0(o); FUN_00181268(o, 0).
pub fn ee_sub_181238<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0017_f6b0, &[a0], &[]);
    m.call(0x0018_1268, &[a0, 0], &[]);
}

/// FUN_001b1f10(o): FUN_0013b250(o); FUN_0012e460(o + 0x268).
pub fn ee_sub_1b1f10<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0013_b250, &[a0], &[]);
    m.call(0x0012_e460, &[a0 + 0x268], &[]);
}

/// FUN_0022d778(o, a1): FUN_0022d6b0(a1, o+0x10, o).
pub fn ee_sub_22d778<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    // ANIM-ALL: returns FUN_0022d6b0's bone (the vt+0x19c result; was dropped)
    let v = m.r32(a0 + 0x10);
    m.call(0x0022_d6b0, &[a1, v, a0], &[]).0
}

/// FUN_00143cf8(o): FUN_002041c0(o); +0x9e4 = 0.
pub fn ee_sub_143cf8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0020_41c0, &[a0], &[]);
    m.w32(a0 + 0x9e4, 0);
}

/// FUN_0014fcc8(o, v): o[0] = v; o[2] = CM+0x4c; o[1] = CM+0x24.
pub fn ee_sub_14fcc8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.w32(a0, a1);
    let f = m.r32(CM + 0x4c);
    m.w32(a0 + 8, f);
    let f = m.r32(CM + 0x24);
    m.w32(a0 + 4, f);
}

/// FUN_00212a50(o, v): v != +0x64 -> +0x64 = v, +0x68 = *0x2ddf0c.
pub fn ee_sub_212a50<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if m.r32(a0 + 0x64) != a1 {
        m.w32(a0 + 0x64, a1);
        let f = m.r32(0x002d_df0c);
        m.w32(a0 + 0x68, f);
    }
}

/// FUN_00221120(o, out) -> +0x10c ? (*out = o+0x10c, 1) : 0.
pub fn ee_sub_221120<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    if m.r32(a0 + 0x10c) == 0 {
        return 0;
    }
    m.w32(a1, a0 + 0x10c);
    1
}

/// FUN_001441a0(o) -> +0x12fc ? *+0x12fc : +0x1300.
pub fn ee_sub_1441a0<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let p = m.r32(a0 + 0x12fc);
    if p != 0 {
        m.r32(p)
    } else {
        m.r32(a0 + 0x1300)
    }
}

/// FUN_00174fb8(o) -> +0x30 == 1.
pub fn ee_sub_174fb8<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    (m.r32(a0 + 0x30) == 1) as u32
}

/// FUN_001e6620(o) -> f32 *(+0xf0).
pub fn ee_sub_1e6620<M: QMem + ?Sized>(m: &mut M, a0: u32) -> f32 {
    let p = m.r32(a0 + 0xf0);
    m.rf(p)
}

/// FUN_001eef98: empty.
pub fn ee_sub_1eef98<M: QMem + ?Sized>(_m: &mut M) {}

/// FUN_00132b08: empty.
pub fn ee_sub_132b08<M: QMem + ?Sized>(_m: &mut M) {}

/// FUN_00116b28: empty.
pub fn ee_sub_116b28<M: QMem + ?Sized>(_m: &mut M) {}

/// FUN_0015c1e0: empty.
pub fn ee_sub_15c1e0<M: QMem + ?Sized>(_m: &mut M) {}

/// FUN_0014fd38: empty.
pub fn ee_sub_14fd38<M: QMem + ?Sized>(_m: &mut M) {}

/// FUN_001eef70: empty.
pub fn ee_sub_1eef70<M: QMem + ?Sized>(_m: &mut M) {}

/// FUN_001af388: empty.
pub fn ee_sub_1af388<M: QMem + ?Sized>(_m: &mut M) {}

/// FUN_0012cf20: empty.
pub fn ee_sub_12cf20<M: QMem + ?Sized>(_m: &mut M) {}

/// FUN_001a6148: empty.
pub fn ee_sub_1a6148<M: QMem + ?Sized>(_m: &mut M) {}

/// FUN_0022fbd8: empty.
pub fn ee_sub_22fbd8<M: QMem + ?Sized>(_m: &mut M) {}

/// FUN_001a6110: empty.
pub fn ee_sub_1a6110<M: QMem + ?Sized>(_m: &mut M) {}

/// FUN_002564b8(a0) -> a0.
pub fn ee_sub_2564b8<M: QMem + ?Sized>(_m: &mut M, a0: u32) -> u32 {
    a0
}

/// FUN_0022dd00(o, st): window check st+0x80 <= *st+0x88 <= st+0x84 (or +0x9a >= 3); no +0x90 -> +0x9a = 0,
/// FUN_0022c818(o, st), FUN_002303a0(o, st, 0); else +0x90 (vtable +0x18) slot 3, then by kind +0x98 / phase
/// +0x9a: FUN_002303a0(o, st, 6 / 7 / 0xb / 0x12).
pub fn ee_sub_22dd00<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let s0 = a1;
    let p = m.r32(s0 + 0x88);
    let f0 = m.rf(s0 + 0x80);
    let f1 = m.rf(p);
    if !(f1 < f0) && !(m.rf(s0 + 0x84) < f1) && (m.r16(s0 + 0x9a) as i16) < 3 {
        return;
    }
    let o = m.r32(s0 + 0x90);
    if o == 0 {
        m.w16(s0 + 0x9a, 0);
        m.call(0x0022_c818, &[a0, s0], &[]);
        m.call(0x0023_03a0, &[a0, s0, 0], &[]);
        return;
    }
    let vt = m.r32(o.wrapping_add(0x18));
    let adj = m.r16(vt.wrapping_add(0x20)) as i16 as u32;
    let f = m.r32(vt.wrapping_add(0x24));
    m.call(f, &[o.wrapping_add(adj)], &[]);
    let k = m.r16(s0 + 0x98) as i16;
    let ph = m.r16(s0 + 0x9a);
    let st = match k {
        10 => {
            if (ph as u32).wrapping_sub(3) < 2 {
                7
            } else if ph as i16 == 2 {
                return;
            } else {
                6
            }
        }
        6 => {
            if (ph as u32).wrapping_sub(3) < 2 {
                7
            } else {
                return;
            }
        }
        0xc => {
            if ph as i16 == 2 {
                return;
            }
            0xb
        }
        0x13 => {
            if ph as i16 == 2 {
                return;
            }
            0x12
        }
        _ => return,
    };
    m.call(0x0023_03a0, &[a0, s0, st], &[]);
}

/// FUN_001624a8(o, ld): reserve pools (FUN_001ae318 {size, 0x10}) for +0x90/+0x98/+0x58/+0x60;
/// FUN_001a5c28(o, ld); shrink them (FUN_001ae238) to +0x8c*16 / +0x94*0x18 / +0x54*4 / +0x5c*4;
/// FUN_00118420(ld); FUN_001ae4a0(&mark, 2).
pub fn ee_sub_1624a8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let sp = stack_push(m, 0x20);
    let s0 = sp + 0x10;
    m.call(0x001a_e478, &[sp], &[]);
    for (sz, off) in [(0x200u32, 0x90u32), (0x300, 0x98), (0x80, 0x58), (0x80, 0x60)] {
        m.w32(s0, sz);
        m.w32(s0 + 4, 0x10);
        let v = m.call(0x001a_e318, &[s0], &[]).0;
        m.w32(a0 + off, v);
    }
    m.call(0x001a_5c28, &[a0, a1], &[]);
    for (cnt, off) in [(0x8cu32, 0x90u32), (0x94, 0x98), (0x54, 0x58), (0x5c, 0x60)] {
        let c = m.r32(a0 + cnt);
        let sz = match cnt {
            0x8c => c << 4,
            0x94 => c.wrapping_mul(0x18),
            _ => c << 2,
        };
        let p = m.r32(a0 + off);
        m.w32(s0, sz);
        m.w32(s0 + 4, 0x10);
        let v = m.call(0x001a_e238, &[s0, p], &[]).0;
        m.w32(a0 + off, v);
    }
    m.call(0x0011_8420, &[a1], &[]);
    m.call(0x001a_e4a0, &[sp, 2], &[]);
    stack_pop(m, 0x20);
}

/// FUN_00171258(o, path, a2, start): path != +0x30 -> tear down the old one (FUN_001ef0d8(o+0x18), slot 2(3),
/// FUN_00256ee0(o, 0), FUN_002580a8(o+4, 0), clear +0x34/+0x38/+0x40/+0x4c); +0x30 = path; path -> FUN_00256ee0(
/// o, *path), slot 2(2), +0x4c = a2, cursor from start (if *start) else path / path+0x50; cursor -> slot 14;
/// +0x44 = 0.
pub fn ee_sub_171258<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32) {
    let cur = m.r32(a0 + 0x30);
    if a1 == cur {
        return;
    }
    if cur != 0 {
        m.call(0x001e_f0d8, &[a0 + 0x18], &[]);
        vslot(m, a0, 2, &[3, 0, 0, 0, 0], &[]);
        m.call(0x0025_6ee0, &[a0, 0], &[]);
        m.call(0x0025_80a8, &[a0 + 4, 0], &[]);
        m.w32(a0 + 0x34, 0);
        m.w32(a0 + 0x38, 0);
        m.w32(a0 + 0x40, 0);
        m.w32(a0 + 0x4c, 0);
    }
    m.w32(a0 + 0x30, a1);
    if a1 == 0 {
        return;
    }
    let v = m.r32(a1);
    m.call(0x0025_6ee0, &[a0, v], &[]);
    vslot(m, a0, 2, &[2, 0, 0, 0, 0], &[]);
    m.w32(a0 + 0x4c, a2);
    let v = if a3 != 0 { m.r32(a3) } else { 0 };
    if a3 != 0 && v != 0 {
        m.w32(a0 + 0x38, v);
        m.w32(a0 + 0x44, a3);
        let w = m.r32(a3 + 4);
        m.w32(a0 + 0x34, w);
    } else {
        let p = m.r32(a0 + 0x30);
        let w = m.r32(p + 0x50);
        m.w32(a0 + 0x34, p);
        m.w32(a0 + 0x38, w);
    }
    let c = m.r32(a0 + 0x38);
    if c != 0 {
        vslot(m, a0, 14, &[c], &[]);
    }
    m.w32(a0 + 0x44, 0);
}

/// FUN_00210538(world, id, id2, id3, ctx, scale): o = FUN_0015e938(world, 4, id, ctx); o -> msg {flags,
/// target (id2's object or o), .., scale (flags |= 0x18 when != 1), +0x30 id3's object (flags |= 0x20)}
/// -> o slot 49(&msg).
pub fn ee_sub_210538<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, t0: u32, f12: f32) {
    let s3 = m.call(0x0015_e938, &[a0, 4, a1, t0], &[]).0;
    if s3 == 0 {
        return;
    }
    let sp = stack_push(m, 0x40);
    m.w32(sp, 0);
    if a2 != u32::MAX {
        let v = m.call(0x0015_e938, &[a0, 4, a2, t0], &[]).0;
        if v != 0 {
            m.w32(sp + 4, v);
            let f = m.r32(sp) | 1;
            m.w32(sp, f);
        }
    }
    let f = m.r32(sp);
    if (f ^ 1) & 1 != 0 {
        m.w32(sp + 4, s3);
        m.w32(sp, f | 1);
    }
    if a3 != u32::MAX {
        let v = m.call(0x0015_e938, &[a0, 4, a3, t0], &[]).0;
        if v != 0 {
            m.w32(sp + 0x30, v);
            let f = m.r32(sp) | 0x20;
            m.w32(sp, f);
        }
    }
    if f12 != 1.0 {
        let f = m.r32(sp);
        m.wf(sp + 8, f12);
        m.w32(sp, f | 0x18);
    }
    vslot(m, s3, 49, &[sp], &[]);
    stack_pop(m, 0x40);
}

/// FUN_0011b830(mat, fwd (128-bit a1), up (128-bit a2)) -> mat: r0 = |fwd| (or 0x2a2da0 when |fwd| < 1e-4);
/// r1 = |up x r0| (|.| < 1e-4 -> FUN_0011e390(r0)); r2 = r0 x r1.
pub fn ee_sub_11b830<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: V4, a2: V4) -> u32 {
    let eps = fb(0x38d1_b717);
    let l = len3(a1);
    let r0 = if l < eps { m.rv(0x002a_2da0) } else { scl(a1, 1.0 / l) };
    let v1 = opm(a2, r0);
    m.wv(a0 + 16, v1);
    let l2 = len3(v1);
    m.wv(a0, r0);
    m.wv(a0 + 16, scl(v1, 1.0 / l2));
    if l2 < eps {
        let v = m.call(0x0011_e390, &[], &[r0]).1;
        m.wv(a0 + 16, v);
    }
    let x = m.rv(a0);
    let y = m.rv(a0 + 16);
    let mut z = opm(x, y);
    z[3] = x[3];
    m.wv(a0 + 32, z);
    a0
}

/// FUN_001fb3f0(o, pos (128-bit a1)): +0x20 = pos; pos == 0 (FUN_0011d4d0 per x/y/z, eps 1e-4) ->
/// (+0x1c == 0 -> FUN_001fb658(o, 0x200, 0)); else FUN_001fb658(o, 0x200, 0x200).
pub fn ee_sub_1fb3f0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: V4) {
    let eps = f4(fb(0x38d1_b717));
    m.wv(a0 + 0x20, a1);
    let z = m.rv(V_ZERO);
    let mut same = false;
    if m.call(0x0011_d4d0, &[], &[f4(z[0]), f4(a1[0]), eps]).0 != 0
        && m.call(0x0011_d4d0, &[], &[f4(z[1]), f4(a1[1]), eps]).0 != 0
    {
        same = m.call(0x0011_d4d0, &[], &[f4(z[2]), f4(a1[2]), eps]).0 != 0;
    }
    if same {
        if m.r32(a0 + 0x1c) == 0 {
            m.call(0x001f_b658, &[a0, 0x200, 0], &[]);
        }
    } else {
        m.call(0x001f_b658, &[a0, 0x200, 0x200], &[]);
    }
}

/// FUN_001f09d8(o, key): !FUN_001f0638(o, key) -> slot 0; i = FUN_001f0438(+0x24, key); unchanged
/// (+0x34 == i, +0x3c == i or -1) -> slot 1; else release +0x30 (FUN_00130b18), +0x3c = +0x34 = i, +0x38 = -1,
/// FUN_001f13a8(o, old, i), FUN_001f0d88(o, 0), slot 1.
pub fn ee_sub_1f09d8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if m.call(0x001f_0638, &[a0, a1], &[]).0 != 0 {
        return;
    }
    vslot(m, a0, 0, &[], &[]);
    let t = m.r32(a0 + 0x24);
    let s1 = m.call(0x001f_0438, &[t, a1], &[]).0;
    if m.r32(a0 + 0x34) == s1 {
        let v = m.r32(a0 + 0x3c);
        if v == s1 || v == u32::MAX {
            vslot(m, a0, 1, &[], &[]);
            return;
        }
    }
    let h = m.r32(a0 + 0x30);
    if h != 0 {
        m.w32(a0 + 0x30, 0);
        m.call(0x0013_0b18, &[h], &[]);
    }
    let old = m.r32(a0 + 0x34);
    m.w32(a0 + 0x3c, s1);
    m.w32(a0 + 0x34, s1);
    m.w32(a0 + 0x38, u32::MAX);
    m.call(0x001f_13a8, &[a0, old, s1], &[]);
    m.call(0x001f_0d88, &[a0, 0], &[]);
    vslot(m, a0, 1, &[], &[]);
}

/// FUN_001dfb10(o, ld): +0x54 = FUN_001ae318({0xa000, 0x10}); FUN_001e0290(o, -1); +0x7c = 1;
/// FUN_001a5c28(o, ld); +0x58 = max(+0x58, +0x50) + 1; +0x54 = FUN_001ae1d0({+0x58 * 0x50, 0x10}) with the
/// old +0x50 * 0x50 bytes copied (FUN_001ae5a0); FUN_001ae4a0(&mark, 2).
pub fn ee_sub_1dfb10<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let sp = stack_push(m, 0x20);
    let s1 = sp + 0x10;
    m.call(0x001a_e478, &[sp], &[]);
    m.w32(s1, 0xa000);
    m.w32(s1 + 4, 0x10);
    let v = m.call(0x001a_e318, &[s1], &[]).0;
    m.w32(a0 + 0x54, v);
    m.call(0x001e_0290, &[a0, u32::MAX], &[]);
    m.w32(a0 + 0x7c, 1);
    m.call(0x001a_5c28, &[a0, a1], &[]);
    let v0 = m.r32(a0 + 0x58) as i32;
    let n50 = m.r32(a0 + 0x50) as i32;
    let s2 = m.r32(a0 + 0x54);
    let n = if n50 < v0 { v0 } else { n50 }.wrapping_add(1);
    m.w32(a0 + 0x58, n as u32);
    m.w32(s1, (n as u32).wrapping_mul(0x50));
    m.w32(s1 + 4, 0x10);
    let p = m.call(0x001a_e1d0, &[s1], &[]).0;
    let c = m.r32(a0 + 0x50).wrapping_mul(0x50);
    m.w32(a0 + 0x54, p);
    m.call(0x001a_e5a0, &[p, s2, c], &[]);
    m.call(0x001a_e4a0, &[sp, 2], &[]);
    stack_pop(m, 0x20);
}

/// FUN_001b8008(jt): pick the current state's animation (+0x13d0: 5 -> +0x1ab0[+0x1458], 2 -> +0x1abc or
/// FUN_0019c870(jt+0x2c0, &k), 0xd -> FUN_001944c8(jt+0x4c0, &k), 0xe -> FUN_001a23f8(jt+0xdf0, &k), else
/// +0x1a58[state]); 0 -> +0x268; FUN_00203d00(jt, anim, k).
pub fn ee_sub_1b8008<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let sp = stack_push(m, 0x10);
    m.w32(sp, 0);
    let st = m.r32(a0 + 0x13d0);
    let mut v = match st {
        5 => {
            let i = m.r32(a0 + 0x1458);
            m.r32(a0.wrapping_add(i << 2) + 0x1ab0)
        }
        2 => {
            if m.r32(a0 + 0x13f0) != 0 {
                m.r32(a0 + 0x1abc)
            } else {
                m.call(0x0019_c870, &[a0 + 0x2c0, sp], &[]).0
            }
        }
        0xd => m.call(0x0019_44c8, &[a0 + 0x4c0, sp], &[]).0,
        0xe => m.call(0x001a_23f8, &[a0 + 0xdf0, sp], &[]).0,
        _ => {
            let s = m.r32(a0 + 0x13d0);
            m.r32(a0.wrapping_add(s << 2) + 0x1a58)
        }
    };
    if v == 0 {
        v = m.r32(a0 + 0x268);
    }
    let k = m.r32(sp);
    stack_pop(m, 0x10);
    m.call(0x0020_3d00, &[a0, v, k], &[]);
}

/// FUN_001f0660(o, key): !FUN_001f0638(o, key) and i = (key == -1 ? -1 : FUN_001f0438(+0x24, key)) != +0x3c
/// -> slot 0; +0x3c = i; +0x30 ? (+0x40 +0xc & 2 -> FUN_00130b18(+0x30)) : FUN_001f0d88(o, 0); slot 1.
pub fn ee_sub_1f0660<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if m.call(0x001f_0638, &[a0, a1], &[]).0 != 0 {
        return;
    }
    let s1 = if a1 == u32::MAX {
        u32::MAX
    } else {
        let t = m.r32(a0 + 0x24);
        m.call(0x001f_0438, &[t, a1], &[]).0
    };
    if s1 == m.r32(a0 + 0x3c) {
        return;
    }
    vslot(m, a0, 0, &[], &[]);
    let h = m.r32(a0 + 0x30);
    m.w32(a0 + 0x3c, s1);
    if h != 0 {
        let p = m.r32(a0 + 0x40);
        if m.r32(p + 0xc) & 2 != 0 {
            m.call(0x0013_0b18, &[h], &[]);
        }
    } else {
        m.call(0x001f_0d88, &[a0, 0], &[]);
    }
    vslot(m, a0, 1, &[], &[]);
}

/// FUN_0012b9d0(curve, out_pos, out_vel, t, f13): k = FUN_0012b6d8(curve, .., t, f13); keys p = +8 + k*0x30
/// and p - 0x30 -> FUN_00117120(out_pos, out_vel, 0, {p0}, p0 tangent, {p1}, p1 tangent, p.t - p0.t,
/// t - p0.t, f13).
pub fn ee_sub_12b9d0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, f12: f32, f13: f32) {
    let k = m.call(0x0012_b6d8, &[a0, a1, a2], &[f4(f12), f4(f13)]).0;
    let p = k.wrapping_mul(0x30).wrapping_add(m.r32(a0 + 8));
    let sp = stack_push(m, 0x20);
    cpn(m, sp, p.wrapping_sub(0x30), 12);
    cpn(m, sp + 0x10, p, 12);
    let f0 = m.rf(p.wrapping_sub(0x24));
    let g12 = m.rf(p + 0xc) - f0;
    let g13 = f12 - f0;
    let q3 = m.rv(p + 16);
    let q0 = m.rv(sp);
    let q1 = m.rv(p.wrapping_sub(16));
    let q2 = m.rv(sp + 16);
    stack_pop(m, 0x20);
    m.call(0x0011_7120, &[a1, a2, 0], &[f4(g12), f4(g13), f4(f13), q0, q1, q2, q3]);
}

/// FUN_001d04e8(o, pos (128-bit a1)): FUN_001ce920(o, pos); +0x100 = FUN_0014f308(CM, pos); +0x90.. =
/// FUN_0011d448 per axis of def (+0xf0) +0x20..+0x38; +0x18 = *0x2c2000; +0x30 = *0x2bbb50; +0x20 = def+0x50.
pub fn ee_sub_1d04e8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: V4) {
    m.call(0x001c_e920, &[a0], &[a1]);
    let s0 = m.r32(a0 + 0xf0);
    let v = m.call(0x0014_f308, &[CM], &[a1]).1;
    m.wv(a0 + 0x100, v);
    let mut r = [0.0f32; 3];
    for i in 0..3u32 {
        let (lo, hi) = (m.rf(s0 + 0x20 + 4 * i), m.rf(s0 + 0x30 + 4 * i));
        r[i as usize] = m.call(0x0011_d448, &[], &[f4(lo), f4(hi)]).1[0];
    }
    m.wf(a0 + 0x90, r[0]);
    let f1 = m.rf(0x002c_2000);
    m.wf(a0 + 0x94, r[1]);
    m.wf(a0 + 0x98, r[2]);
    m.wf(a0 + 0x18, f1);
    let q = m.rv(0x002b_bb50);
    m.wv(a0 + 0x30, q);
    let q = m.rv(s0 + 0x50);
    m.wv(a0 + 0x20, q);
}

/// FUN_001c5098(o, ev, msg): FUN_00133590(o, ev, msg); ev 10 from +0x168 with code 0x186 -> +0x218 slot 49
/// ({1, o}); slot 74(2).
pub fn ee_sub_1c5098<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    m.call(0x0013_3590, &[a0, a1, a2], &[]);
    if a1 == 10 && m.r32(a2) == m.r32(a0 + 0x168) && m.r32(a2 + 8) == 0x186 {
        let o = m.r32(a0 + 0x218);
        if o != 0 {
            let sp = stack_push(m, 0x10);
            m.w32(sp + 4, a0);
            m.w32(sp, 1);
            vslot(m, o, 49, &[sp], &[]);
            stack_pop(m, 0x10);
        }
        vslot(m, a0, 74, &[2], &[]);
    }
}

/// FUN_001de478(o, ld): +0x19c = FUN_001ae378({0x5400, 0x10}); FUN_001f31e0(o, ld); shrink +0x19c to
/// +0x198 * 0x150 (FUN_001ae238); FUN_001ae4a0(&mark, 2).
pub fn ee_sub_1de478<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let sp = stack_push(m, 0x20);
    let s0 = sp + 0x10;
    m.call(0x001a_e478, &[sp], &[]);
    m.w32(s0, 0x5400);
    m.w32(s0 + 4, 0x10);
    let v = m.call(0x001a_e378, &[s0], &[]).0;
    m.w32(a0 + 0x19c, v);
    m.call(0x001f_31e0, &[a0, a1], &[]);
    let n = m.r32(a0 + 0x198).wrapping_mul(0x150);
    let p = m.r32(a0 + 0x19c);
    m.w32(s0, n);
    m.w32(s0 + 4, 0x10);
    let v = m.call(0x001a_e238, &[s0, p], &[]).0;
    m.w32(a0 + 0x19c, v);
    m.call(0x001a_e4a0, &[sp, 2], &[]);
    stack_pop(m, 0x20);
}

/// FUN_00151240(o, a1, a2): FUN_0014fcc8(o, a1); +0x10 = a2; CM FUN_0014f610(25.0), FUN_0014f5a0(1.15);
/// +0x14 = FUN_0012dc18(*o, +0x10, 1, 8, 3, zero quad); +0x18 = -1.0.
pub fn ee_sub_151240<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    m.call(0x0014_fcc8, &[a0, a1], &[]);
    m.w32(a0 + 0x10, a2);
    m.call(0x0014_f610, &[CM], &[f4(25.0)]);
    m.call(0x0014_f5a0, &[CM], &[f4(fb(0x3f93_3333))]);
    let (p, q) = (m.r32(a0), m.r32(a0 + 0x10));
    let z = m.rv(V_ZERO);
    let v = m.call(0x0012_dc18, &[p, q, 1, 8, 3], &[z]).0;
    m.w32(a0 + 0x14, v);
    m.wf(a0 + 0x18, -1.0);
}

/// FUN_001b83b8(jt, msg) -> state 0xf, or (0x14 and msg 0x4000) -> 1; state 0xd -> FUN_00194640(jt+0x4c0,
/// msg) || FUN_00140ed8(jt, msg); else FUN_00140ed8(jt, msg).
pub fn ee_sub_1b83b8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let st = m.r32(a0 + 0x13d0);
    if st == 0xf || (st == 0x14 && a1 == 0x4000) {
        return 1;
    }
    if st == 0xd && m.call(0x0019_4640, &[a0 + 0x4c0, a1], &[]).0 != 0 {
        return 1;
    }
    m.call(0x0014_0ed8, &[a0, a1], &[]).0
}

/// FUN_001d05a0(o): FUN_00117120(o+0x100, o+0x90, 0, max(+0x14 + +0x18 - now, dt), dt, 1.0, {+0x100},
/// {+0x90}, def{+0}, def{+0x10}); +0x80 = +0x70; +0x70 = FUN_0014f268(CM, {+0x100}).
pub fn ee_sub_1d05a0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let f2 = m.rf(T_NOW);
    let f0 = m.rf(T_DT);
    let d = m.r32(a0 + 0xf0);
    let f12 = (m.rf(a0 + 0x14) + m.rf(a0 + 0x18) - f2).max(f0);
    let a3 = m.rv(d + 16);
    let a2 = m.rv(d);
    let q0 = m.rv(a0 + 0x100);
    let q1 = m.rv(a0 + 0x90);
    m.call(0x0011_7120, &[a0 + 0x100, a0 + 0x90, 0], &[f4(f12), f4(f0), f4(1.0), q0, q1, a2, a3]);
    let v = m.rv(a0 + 0x70);
    m.wv(a0 + 0x80, v);
    let q = m.rv(a0 + 0x100);
    let r = m.call(0x0014_f268, &[CM], &[q]).1;
    m.wv(a0 + 0x70, r);
}

/// FUN_00177ef0(o): FUN_001eef70(o); +0x80 = FUN_001a62e8(+0x10); +0x64 = max(+0x64, lo); +0x6c = min(hi,
/// +0x64); +0x68 = min(cap, +0x64 / 4); slot 14(0).
pub fn ee_sub_177ef0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001e_ef70, &[a0], &[]);
    let p = m.r32(a0 + 0x10);
    let v = m.call(0x001a_62e8, &[p], &[]).1;
    let f0 = m.rf(0x002b_e62c).max(m.rf(a0 + 0x64));
    let f3 = f0 * 0.25;
    let f2 = m.rf(0x002b_e624);
    let f1 = m.rf(0x002b_e628);
    m.wv(a0 + 0x80, v);
    m.wf(a0 + 0x64, f0);
    m.wf(a0 + 0x6c, f1.min(f0));
    m.wf(a0 + 0x68, f2.min(f3));
    vslot(m, a0, 14, &[0], &[]);
}

/// FUN_0011e208(a, b, r) -> quad {cb*sa*r, cb*ca*r, sb*r} from two FUN_0011d608 (sin/cos pairs).
pub fn ee_sub_11e208<M: QMem + ?Sized>(m: &mut M, f12: f32, f13: f32, f14: f32) -> V4 {
    let sp = stack_push(m, 0x20);
    m.call(0x0011_d608, &[sp, sp + 4], &[f4(f12)]);
    m.call(0x0011_d608, &[sp + 8, sp + 12], &[f4(f13)]);
    let (a, b, c, d) = (m.rf(sp), m.rf(sp + 4), m.rf(sp + 8), m.rf(sp + 12));
    stack_pop(m, 0x20);
    let x = d * b * f14;
    let y = d * a * f14;
    let z = c * f14;
    [x, y, z, gqf(z)[1]]
}

/// FUN_00117e38(stream) -> f32 next 4 bytes (FUN_00117968 refill when < 4 buffered).
pub fn ee_sub_117e38<M: QMem + ?Sized>(m: &mut M, a0: u32) -> f32 {
    let n = m.r32(a0 + 8);
    if (n as i32) < 4 {
        let sp = stack_push(m, 0x10);
        m.call(0x0011_7968, &[a0, 4, sp, n], &[]);
        let f = m.rf(sp);
        stack_pop(m, 0x10);
        return f;
    }
    let p = m.r32(a0 + 4);
    let w = (m.r8(p) as u32)
        .wrapping_add((m.r8(p + 1) as u32) << 8)
        .wrapping_add((m.r8(p + 2) as u32) << 16)
        .wrapping_add((m.r8(p + 3) as u32) << 24);
    m.w32(a0 + 8, n.wrapping_sub(4));
    m.w32(a0 + 4, p + 4);
    fb(w)
}

/// FUN_001713e8(o): FUN_001eef98(o); while +0x38 && !FUN_00171468(o): +0x38 = next (+0x18), slot 14.
pub fn ee_sub_1713e8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001e_ef98, &[a0], &[]);
    if m.r32(a0 + 0x38) == 0 {
        return;
    }
    loop {
        if m.call(0x0017_1468, &[a0], &[]).0 != 0 {
            return;
        }
        let c = m.r32(a0 + 0x38);
        let n = m.r32(c + 0x18);
        m.w32(a0 + 0x38, n);
        if n == 0 {
            return;
        }
        vslot(m, a0, 14, &[n], &[]);
        if m.r32(a0 + 0x38) == 0 {
            return;
        }
    }
}

fn rot3(r0: V4, r1: V4, r2: V4, v: V4) -> V4 {
    let mut o = v;
    for i in 0..3 {
        o[i] = r0[i] * v[0] + r1[i] * v[1] + r2[i] * v[2];
    }
    o
}

/// FUN_0022fcc8(a, b, v (128-bit a2)) -> v rotated by a's matrix (+0x40) then by FUN_0011b340(b+0x40).
pub fn ee_sub_22fcc8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: V4) -> V4 {
    let mut v = a2;
    if a0 != 0 {
        let (r0, r1, r2) = (m.rv(a0 + 0x40), m.rv(a0 + 0x50), m.rv(a0 + 0x60));
        v = rot3(r0, r1, r2, v);
    }
    if a1 != 0 {
        let sp = stack_push(m, 0x40);
        m.wv(sp + 0x30, v);
        m.call(0x0011_b340, &[sp, a1 + 0x40], &[]);
        let (r0, r1, r2) = (m.rv(sp), m.rv(sp + 16), m.rv(sp + 32));
        v = m.rv(sp + 0x30);
        stack_pop(m, 0x40);
        v = rot3(r0, r1, r2, v);
    }
    v
}

/// FUN_0013aae8(o, ev, msg): FUN_00133590; ev 4 from o with +0x150 == 0 -> FUN_0013b5c8(o, 0.0).
pub fn ee_sub_13aae8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    m.call(0x0013_3590, &[a0, a1, a2], &[]);
    if a1 == 4 && m.r32(a2 + 4) == a0 && m.r32(a0 + 0x150) == 0 {
        m.call(0x0013_b5c8, &[a0], &[f4(0.0)]);
    }
}

/// FUN_001442d0(o, a1, on): FUN_00205478(o, a1, on); on and ++(+0x12f4) == 1 -> FUN_00143ee0(o, 0x60b, 9, 5),
/// FUN_0019cd38(o+0xf00, -1).
pub fn ee_sub_1442d0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    m.call(0x0020_5478, &[a0, a1, a2], &[]);
    if a2 == 0 {
        return;
    }
    let v = m.r32(a0 + 0x12f4).wrapping_add(1);
    m.w32(a0 + 0x12f4, v);
    if v == 1 {
        m.call(0x0014_3ee0, &[a0, 0x60b, 9, 5], &[]);
        m.call(0x0019_cd38, &[a0 + 0xf00, u32::MAX], &[]);
    }
}

/// FUN_001acb58(o, v): +0x40 = v; +0x24 -> slot 9(v), +0x24 slot 48(-1).
pub fn ee_sub_1acb58<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.w32(a0 + 0x40, a1);
    if m.r32(a0 + 0x24) == 0 {
        return;
    }
    vslot(m, a0, 9, &[a1], &[]);
    let o = m.r32(a0 + 0x24);
    vslot(m, o, 48, &[u32::MAX], &[]);
}

/// FUN_0018d698(o, slot, x): detach slot+8 (FUN_00133448(old, o)); x -> slot+8 = x, FUN_001333c0(x, o).
pub fn ee_sub_18d698<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    let old = m.r32(a1 + 8);
    if old != 0 {
        m.call(0x0013_3448, &[old, a0], &[]);
        m.w32(a1 + 8, 0);
    }
    if a2 != 0 {
        m.w32(a1 + 8, a2);
        m.call(0x0013_33c0, &[a2, a0], &[]);
    }
}

/// FUN_001b0ca8(o, ld): FUN_001f31e0(o, ld); FUN_002303a0(o, +0x58, 1); clear +0x58 +0xf0..+0xf8;
/// +0x1d4 = (short)(+0x5c)+0x98; FUN_001b1bf0(o).
pub fn ee_sub_1b0ca8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.call(0x001f_31e0, &[a0, a1], &[]);
    let s = m.r32(a0 + 0x58);
    m.call(0x0023_03a0, &[a0, s, 1], &[]);
    let v = m.r32(a0 + 0x58);
    m.w32(v + 0xf4, 0);
    m.w32(v + 0xf8, 0);
    m.w32(v + 0xf0, 0);
    let p = m.r32(a0 + 0x5c);
    let h = m.r16(p + 0x98) as i16 as i32 as u32;
    m.w32(a0 + 0x1d4, h);
    m.call(0x001b_1bf0, &[a0], &[]);
}

/// FUN_0023dfa0(x: u64) -> FUN_0023e260(w0, w1, w2, sticky-rounded top word) over FUN_0023d468(&x, &tmp)
/// (soft-float unpack / repack).
pub fn ee_sub_23dfa0<M: QMem + ?Sized>(m: &mut M, a0: u64) -> u32 {
    let sp = stack_push(m, 0x30);
    m.w32(sp + 0x20, a0 as u32);
    m.w32(sp + 0x24, (a0 >> 32) as u32);
    m.call(0x0023_d468, &[sp + 0x20, sp], &[]);
    let v = (m.r32(sp + 0x10) as u64) | ((m.r32(sp + 0x14) as u64) << 32);
    let (w0, w1, w2) = (m.r32(sp), m.r32(sp + 4), m.r32(sp + 8));
    stack_pop(m, 0x30);
    let t0 = (((v << 2) as i64) >> 32) as u32;
    let a3 = if v & 0x3fff_ffff == 0 { t0 } else { t0 | 1 };
    m.call(0x0023_e260, &[w0, w1, w2, a3], &[]).0
}

/// FUN_001d0108(o): FUN_001ce920(o); +0x18 = 0.5; +0x30 = *0x2c1f90; +0x20 = *0x2c1fa0.
pub fn ee_sub_1d0108<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001c_e920, &[a0], &[]);
    m.wf(a0 + 0x18, 0.5);
    let v = m.rv(0x002c_1f90);
    m.wv(a0 + 0x30, v);
    let v = m.rv(0x002c_1fa0);
    m.wv(a0 + 0x20, v);
}

/// FUN_00117bd0(stream) -> next byte (FUN_00117968 refill when empty).
pub fn ee_sub_117bd0<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let n = m.r32(a0 + 8);
    if (n as i32) > 0 {
        let p = m.r32(a0 + 4);
        let b = m.r8(p) as u32;
        m.w32(a0 + 8, n.wrapping_sub(1));
        m.w32(a0 + 4, p + 1);
        return b;
    }
    let sp = stack_push(m, 0x10);
    m.call(0x0011_7968, &[a0, 1, sp], &[]);
    let b = m.r8(sp) as u32;
    stack_pop(m, 0x10);
    b
}

/// FUN_002219c8(o): FUN_0014fcc0(o); *0x2c41a8.. = *0x2a96f8..(2 floats); +0x14 = +0x10 = 0.
pub fn ee_sub_2219c8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0014_fcc0, &[a0], &[]);
    let (f1, f0) = (m.rf(0x002a_96f8), m.rf(0x002a_96fc));
    m.wf(0x002c_41a8, f1);
    m.wf(0x002c_41ac, f0);
    m.w32(a0 + 0x14, 0);
    m.w32(a0 + 0x10, 0);
}

/// FUN_0014fce8(o): *o = 0; CM FUN_0014f610(o[2]); CM+0x24 = o[1].
pub fn ee_sub_14fce8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.w32(a0, 0);
    let f = m.rf(a0 + 8);
    m.call(0x0014_f610, &[CM], &[f4(f)]);
    let f = m.rf(a0 + 4);
    m.wf(CM + 0x24, f);
}

/// FUN_0014f850(cam, p (128-bit a1), q (128-bit a2)) -> all lanes of rows(+0xa0,+0xb0,+0xc0)·p - q.x < 0
/// (VU0 MAC flags == 0xf0).
pub fn ee_sub_14f850<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: V4, a2: V4) -> u32 {
    let (r2, r1, r3) = (m.rv(a0 + 0xa0), m.rv(a0 + 0xb0), m.rv(a0 + 0xc0));
    (0..4).all(|i| {
        let acc = r2[i] * a1[0] + r1[i] * a1[1] + r3[i] * a1[2] - a2[0];
        acc < 0.0
    }) as u32
}

/// FUN_00231550(o, out) -> copy +0x60..+0xa0 to out (if any); +0x50 or 0x2c44f0.
pub fn ee_sub_231550<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    if a1 != 0 {
        for k in 0..4 {
            let v = m.rv(a0 + 0x60 + 16 * k);
            m.wv(a1 + 16 * k, v);
        }
    }
    let v = m.r32(a0 + 0x50);
    if v == 0 {
        0x002c_44f0
    } else {
        v
    }
}

/// FUN_001cd998() -> bump-allocate 0x10 from *0x3d4a50 (FUN_002333b4(p+8, 0, 8) first).
pub fn ee_sub_1cd998<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let p = m.r32(0x003d_4a50);
    m.call(0x0023_33b4, &[p + 8, 0, 8], &[]);
    let v = m.r32(0x003d_4a50);
    m.w32(0x003d_4a50, v + 0x10);
    v
}

/// FUN_002079a0(o): FUN_001f3e00(o); FUN_0011a2e8(0x3e1444, o); +0xff8 = now.
pub fn ee_sub_2079a0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001f_3e00, &[a0], &[]);
    m.call(0x0011_a2e8, &[0x003e_1444, a0], &[]);
    let f = m.rf(T_NOW);
    m.wf(a0 + 0xff8, f);
}

/// FUN_002079e0(o): FUN_001f3ea8(o); FUN_0011a220(0x3e1444, o); +0xff8 = *0x2a8868.
pub fn ee_sub_2079e0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001f_3ea8, &[a0], &[]);
    m.call(0x0011_a220, &[0x003e_1444, a0], &[]);
    let f = m.rf(0x002a_8868);
    m.wf(a0 + 0xff8, f);
}

/// FUN_00231170(o, a1): FUN_00231768(o, a1); FUN_0013f810(o, a1).
pub fn ee_sub_231170<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.call(0x0023_1768, &[a0, a1], &[]);
    m.call(0x0013_f810, &[a0, a1], &[]);
}

/// FUN_00207598(o) -> +0x32c || +0x298 || FUN_00209a68(o, 0x2a) != 0.
pub fn ee_sub_207598<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    if m.r32(a0 + 0x32c) != 0 || m.r32(a0 + 0x298) != 0 {
        return 1;
    }
    (m.call(0x0020_9a68, &[a0, 0x2a], &[]).0 != 0) as u32
}

/// FUN_001f78a0(x) -> index of lowest set bit (capped at 26).
pub fn ee_sub_1f78a0<M: QMem + ?Sized>(_m: &mut M, a0: u32) -> u32 {
    if a0 & 1 != 0 {
        return 0;
    }
    for i in 1..0x1a {
        if a0 & (1 << i) != 0 {
            return i;
        }
    }
    0x1a
}

/// FUN_0021bbd0(o): FUN_001f2c30(o); +0x15c -> FUN_0021bc08(o).
pub fn ee_sub_21bbd0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001f_2c30, &[a0], &[]);
    if m.r32(a0 + 0x15c) != 0 {
        m.call(0x0021_bc08, &[a0], &[]);
    }
}

/// FUN_001ff470(o): FUN_001a6150(o); FUN_0011a2e8(0x3e13d8, o).
pub fn ee_sub_1ff470<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_6150, &[a0], &[]);
    m.call(0x0011_a2e8, &[0x003e_13d8, a0], &[]);
}

/// FUN_001844d8(o): FUN_001a6150(o); FUN_0011a2e8(0x3e1438, o).
pub fn ee_sub_1844d8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_6150, &[a0], &[]);
    m.call(0x0011_a2e8, &[0x003e_1438, a0], &[]);
}

/// FUN_0021d460(o): FUN_001a6178(o); FUN_0011a220(0x3e1474, o).
pub fn ee_sub_21d460<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_6178, &[a0], &[]);
    m.call(0x0011_a220, &[0x003e_1474, a0], &[]);
}

/// FUN_0011d948(range, x) -> clamp(x, range[0], range[1]).
pub fn ee_sub_11d948<M: QMem + ?Sized>(m: &mut M, a0: u32, f12: f32) -> f32 {
    let lo = m.rf(a0);
    if f12 < lo {
        return lo;
    }
    let hi = m.rf(a0 + 4);
    if hi < f12 {
        hi
    } else {
        f12
    }
}

/// FUN_0012d058(o) -> o is in the +0x10-linked list at (*(o+8))+0x90.
pub fn ee_sub_12d058<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let p = m.r32(a0 + 8);
    let mut v = m.r32(p + 0x90);
    while v != 0 {
        if v == a0 {
            return 1;
        }
        v = m.r32(v + 0x10);
    }
    0
}

/// FUN_001a3550(o): FUN_002312a0(o); FUN_001a3968(o).
pub fn ee_sub_1a3550<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0023_12a0, &[a0], &[]);
    m.call(0x001a_3968, &[a0], &[]);
}

/// FUN_0015c628(o, a1) -> slot 51(a1).
pub fn ee_sub_15c628<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    vslot(m, a0, 51, &[a1], &[]).0
}

/// FUN_00221cc0(o): +0x10 = +0x14 = 0; FUN_0014fce8(o).
pub fn ee_sub_221cc0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.w32(a0 + 0x10, 0);
    m.w32(a0 + 0x14, 0);
    m.call(0x0014_fce8, &[a0], &[]);
}

/// FUN_0014d1d0(o, i, x): o[i]+0x18 = x; FUN_0014d1f8(o, o + i*4, x).
pub fn ee_sub_14d1d0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, f12: f32) {
    let p = a0.wrapping_add(a1 << 2);
    m.wf(p + 0x18, f12);
    m.call(0x0014_d1f8, &[a0, p], &[f4(f12)]);
}

/// FUN_00156698(src, pos (128-bit a1), arg): FUN_00156548(src, pos, arg, &out) with out on the stack; return out.
/// BOSS R24: 1:1 form (PART r6's held-back fix) now that the live frame expires the records (FUN_00226550's
/// FUN_00156728 walk over world+0x3ffc, `LiveMem` tick).
pub fn ee_sub_156698<M: QMem + ?Sized>(m: &mut M, a0: u32, pos: V4, a2: u32) -> u32 {
    let sp = stack_push(m, 0x10);
    m.call(0x0015_6548, &[a0, a2, sp], &[pos]);
    let v = m.r32(sp);
    stack_pop(m, 0x10);
    v
}

/// FUN_0015e938(a0..a3) -> FUN_0015e958(a0, a1, a2, a3, 0).
pub fn ee_sub_15e938<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32) -> u32 {
    m.call(0x0015_e958, &[a0, a1, a2, a3, 0], &[]).0
}

/// FUN_00229680(a0, a1) -> FUN_001f50e0(a0, a1).
pub fn ee_sub_229680<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    m.call(0x001f_50e0, &[a0, a1], &[]).0
}

/// FUN_00140f70(o, a1) -> FUN_001a23c0(o + 0xdf0, a1).
pub fn ee_sub_140f70<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    m.call(0x001a_23c0, &[a0 + 0xdf0, a1], &[]).0
}
