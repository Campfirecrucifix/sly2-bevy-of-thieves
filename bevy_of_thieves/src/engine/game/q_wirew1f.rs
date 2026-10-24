//! WIRE-W1f: 1:1 queue ports (against `QMem`) of live-frontier natives, ported from the EE asm
//! (tools/eedis.py). Callees go through `m.call` / `vslot`; integer / pointer args in `a`, float args then
//! quadword (128-bit GPR) args in `v`; a float result is `.1[0]`, a quadword result `.1`. VU0 macro-mode
//! blocks are emulated inline lane by lane.
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

const T_NOW: u32 = 0x002d_ded0; // frame time
const T_DT: u32 = 0x002d_ded4; // frame dt
const EV: u32 = 0x002d_eb30; // FUN_001619a0 event sink

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

fn fb(b: u32) -> f32 {
    f32::from_bits(b)
}

/// MIPS `mfc1` sign extension of a float's bits into the upper word of a 64-bit GPR
fn sext(x: f32) -> f32 {
    fb(((x.to_bits() as i32) >> 31) as u32)
}

/// `c + x * (c1 + x * c2)` from the 3-float table at `t`
fn poly<M: QMem + ?Sized>(m: &mut M, t: u32, x: f32) -> f32 {
    let (c0, c1, c2) = (m.rf(t), m.rf(t + 4), m.rf(t + 8));
    c0 + x * (c1 + x * c2)
}

/// `v < lo -> lo; hi < v -> hi; v` (the asm's two `c.olt.s` order)
fn clamp(v: f32, lo: f32, hi: f32) -> f32 {
    if v < lo {
        lo
    } else if hi < v {
        hi
    } else {
        v
    }
}

/// FUN_001d19a8(o): steering / wheel update. FUN_001f36e0(o); +0x540 = atan2-ish FUN_002458f0(+0x60 -> +0x44,
/// +0x40); per +0x4fc[i] (count +0x4f8): FUN_001d4e08 == 0 -> FUN_001f0660(+0x20, 0x65); mode +0x180 (0/3 reset,
/// 1 FUN_001d2610, 2 FUN_001d2150); +0x548/+0x558/+0x554 from clamped polys of FUN_0011cf60(+0x544 - +0x540);
/// vslot 0x1e +0x28 > 0.5 -> +0x5c0 = now; +0x4a8 records -> FUN_0012d108; +0x1a0 records smoothed
/// (FUN_0011d2d0); per wheel VU0 projection -> +0x74 / +0x78 / +0xa0 -> FUN_0012d108(+0x34); +0x598 sound
/// (FUN_0011d018, FUN_001fb1f8 / FUN_001fb2c0 / FUN_001fb060); moving -> FUN_001f5078(o).
pub fn ee_sub_1d19a8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001f_36e0, &[o], &[]);
    let p = m.r32(o + 0x60);
    let (y, x) = (m.rf(p + 0x44), m.rf(p + 0x40));
    let a = m.call(0x0024_58f0, &[], &[f4(y), f4(x)]).1[0];
    m.wf(o + 0x540, a);
    let mut i = 0;
    while (i as i32) < m.r32(o + 0x4f8) as i32 {
        let w = m.r32(o + 0x4fc + 4 * i);
        if m.call(0x001d_4e08, &[w], &[]).0 == 0 {
            let h = m.r32(w + 0x20);
            m.call(0x001f_0660, &[h, 0x65], &[]);
        }
        i += 1;
    }
    match m.r32(o + 0x180) {
        0 | 3 => {
            m.wf(o + 0x550, 0.0);
            let v = m.rf(o + 0x540);
            m.wf(o + 0x544, v);
            m.wf(o + 0x554, 0.0);
        }
        1 => {
            m.call(0x001d_2610, &[o], &[]);
        }
        2 => {
            m.call(0x001d_2150, &[o], &[]);
        }
        _ => {}
    }
    let d = m.rf(o + 0x544) - m.rf(o + 0x540);
    let e = m.call(0x0011_cf60, &[], &[f4(d)]).1[0];
    let v = poly(m, 0x002c_2040, e);
    let (lo, hi) = (m.rf(0x002c_2050), m.rf(0x002c_2054));
    m.wf(o + 0x548, clamp(v, lo, hi));
    let d = m.rf(o + 0x544) - m.rf(o + 0x540);
    let e = m.call(0x0011_cf60, &[], &[f4(d)]).1[0].abs();
    let v = poly(m, 0x002c_2060, e);
    let (lo, hi) = (m.rf(0x002c_2070), m.rf(0x002c_2074));
    m.wf(o + 0x558, clamp(v, lo, hi));
    let d = m.rf(o + 0x544) - m.rf(o + 0x540);
    let e = m.call(0x0011_cf60, &[], &[f4(d)]).1[0].abs();
    let v = poly(m, 0x002c_2080, e);
    let (lo, hi) = (m.rf(0x002c_2090), m.rf(0x002c_2094));
    let g = if v < lo {
        lo
    } else if hi < v {
        hi
    } else {
        v
    };
    let mn = m.rf(0x002c_2030).min(m.rf(o + 0x550));
    let r = g * mn * m.rf(o + 0x548) / m.rf(o + 0x4a0);
    m.wf(o + 0x554, r);
    let q = vslot(m, o, 0x1e, &[], &[]).0;
    if 0.5 < m.rf(q + 0x28) {
        let t = m.rf(T_NOW);
        m.wf(o + 0x5c0, t);
    }
    let mut i = 0u32;
    while (i as i32) < m.r32(o + 0x4a4) as i32 {
        let rec = o + 0x4a8 + 0x14 * i;
        let s = m.rf(o + 0x54c);
        i += 1;
        let qv = m.call(0x0011_e1a0, &[], &[f4(s), f4(1.0), f4(0.0)]).1;
        let h = m.r32(rec + 4);
        let h = m.r32(h + 0xa0);
        let r = m.call(0x0022_fcc8, &[0, h], &[qv]).1;
        // prot3w: f12 = lane y, f13 = lane x
        let ang = m.call(0x0024_58f0, &[], &[f4(r[1]), f4(r[0])]).1[0];
        let f = m.call(0x0011_d948, &[o + 0x4b4 + 0x14 * (i - 1)], &[f4(ang)]).1[0];
        let h = m.r32(rec + 8);
        m.call(0x0012_d108, &[h], &[[0.0, 0.0, f, sext(f)]]);
    }
    let mut i = 0u32;
    while (i as i32) < m.r32(o + 0x190) as i32 {
        let rec = o + 0x1a0 + 0x60 * i;
        let tgt = if m.rf(o + 0x550) == 0.0 { 0.0 } else { m.rf(rec + 0xc) * m.rf(o + 0x548) };
        let cur = m.rf(rec + 0x14);
        let dt = m.rf(T_DT);
        i += 1;
        let r = m.call(0x0011_d2d0, &[0x002c_2098, 0], &[f4(cur), f4(tgt), f4(dt)]).1[0];
        m.wf(rec + 0x14, r);
    }
    let me = m.call(0x001a_62e8, &[o], &[]).1;
    let oth = m.r32(o + 0x5a4);
    let ot = m.call(0x001a_62e8, &[oth], &[]).1;
    let oth = m.r32(o + 0x5a4);
    let vv = vslot(m, oth, 0x20, &[], &[]).1;
    if (m.r32(o + 0x4f8) as i32) > 0 {
        // vsub; vmul.xy; vaddy.x; vsqrt
        let dx = ot[0] - me[0];
        let dy = ot[1] - me[1];
        let dist = (dx * dx + dy * dy).sqrt();
        let u = m.rv(0x002a_2dc0);
        let mut s = 0u32;
        loop {
            let w = m.r32(o + 0x4fc + 4 * s);
            if m.r32(w + 0x30) != 0 {
                let mut ok = false;
                if m.rf(o + 0x550) == 0.0 && m.call(0x001d_4e08, &[w], &[]).0 != 0 {
                    ok = true;
                }
                if ok {
                    let v = poly(m, w + 0x50, dist);
                    let (lo, hi) = (m.rf(w + 0x60), m.rf(w + 0x64));
                    let k = clamp(v, lo, hi);
                    let p = if m.r32(w + 0x3c) != 0 {
                        let a0 = m.r32(w + 0x3c);
                        m.call(0x001a_62e8, &[a0], &[]).1
                    } else {
                        let b = m.r32(w + 0x30);
                        m.rv(b + 0x70)
                    };
                    let c = m.rf(w + 0x68) * (k * -0.5);
                    let inv = 1.0 / k;
                    let d5 = [ot[0] - p[0], ot[1] - p[1], ot[2] - p[2], ot[3] - p[3]];
                    let dot = vv[0] * u[0] + vv[1] * u[1] + vv[2] * u[2];
                    let mut r = [0.0f32; 4];
                    for l in 0..4 {
                        let t = vv[l] - u[l] * dot;
                        let one = if l == 3 { 1.0 } else { 0.0 };
                        r[l] = t + one + d5[l] * inv + u[l] * c;
                    }
                    let hz = (r[0] * r[0] + r[1] * r[1]).sqrt();
                    let pitch = m.call(0x0024_58f0, &[], &[f4(r[2]), f4(hz)]).1[0];
                    let len = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt();
                    let a74 = m.call(0x0011_d948, &[w + 0x40], &[f4(pitch)]).1[0];
                    m.wf(w + 0x74, a74);
                    let a78 = m.call(0x0011_d948, &[w + 0x48], &[f4(len)]).1[0];
                    m.wf(w + 0x78, a78);
                    let eps = fb(0x3c23_d70a);
                    let cur = m.rf(w + 0x74);
                    let mut ok2 = 0u32;
                    if m.call(0x0011_d4d0, &[], &[f4(cur), f4(pitch), f4(eps)]).0 != 0 {
                        let cur = m.rf(w + 0x78);
                        ok2 = (m.call(0x0011_d4d0, &[], &[f4(cur), f4(len), f4(eps)]).0 != 0) as u32;
                    }
                    m.w32(w + 0xa0, ok2);
                    if ok2 == 0 {
                        let v = m.rf(w + 0x48);
                        m.w32(w + 0x74, 0);
                        m.wf(w + 0x78, v);
                    }
                } else {
                    let v = m.rf(w + 0x48);
                    m.w32(w + 0xa0, 0);
                    m.wf(w + 0x78, v);
                    m.w32(w + 0x74, 0);
                }
                let n = -m.rf(w + 0x74);
                let h = m.r32(w + 0x34);
                m.call(0x0012_d108, &[h], &[[0.0, n, 0.0, 0.0]]);
            }
            s += 1;
            if (s as i32) >= m.r32(o + 0x4f8) as i32 {
                break;
            }
        }
    }
    if m.r32(o + 0x598) != 0 {
        let tgt = m.rf(o + 0x550) / m.rf(0x002c_20a4);
        let cur = m.rf(o + 0x5a0);
        let dt = m.rf(T_DT);
        let f = m.call(0x0011_d018, &[0x002c_20a8, 0], &[f4(cur), f4(tgt), f4(dt)]).1[0];
        m.wf(o + 0x5a0, f);
        let lo = fb(0x3e4c_cccd);
        let g = clamp(f, lo, 1.0);
        let h = m.r32(o + 0x598);
        m.call(0x001f_b1f8, &[h], &[f4(g)]);
        let h = m.r32(o + 0x598);
        m.call(0x001f_b2c0, &[h], &[f4(g * lo)]);
        let h = m.r32(o + 0x598);
        if m.r16(h + 0x4e) as i16 > 0 {
            m.call(0x001f_b060, &[h], &[]);
        }
    }
    if m.rf(o + 0x550) != 0.0 || m.rf(o + 0x554) != 0.0 {
        m.call(0x001f_5078, &[o], &[]);
    }
}

/// FUN_00202768(o, st): state change of a two-actor (+0xac / +0xb0) minigame. Leaving / entering states 8..9
/// toggles FUN_0020efc0 / FUN_0020f030(0x3dd490); exit actions by old state, then `*o = st` and entry actions.
pub fn ee_sub_202768<M: QMem + ?Sized>(m: &mut M, o: u32, st: u32) {
    if m.r32(o) == st || m.r32(o + 0xac) == 0 || m.r32(o + 0xb0) == 0 {
        return;
    }
    let was = m.r32(o).wrapping_sub(8) < 2;
    let now = st.wrapping_sub(8) < 2;
    if was != now {
        if now {
            m.call(0x0020_efc0, &[0x003d_d490], &[]);
        } else {
            m.call(0x0020_f030, &[0x003d_d490, 0], &[]);
        }
    }
    let ac = |m: &mut M| m.r32(o + 0xac);
    let bc = |m: &mut M| m.r32(o + 0xb0);
    match m.r32(o) as i32 {
        1 => {
            m.call(0x0020_20c8, &[o], &[]);
            m.call(0x0020_2030, &[o], &[]);
            let a = ac(m);
            m.call(0x0021_3b00, &[a, 0x35d], &[]);
            if (m.r32(o + 0x74) as i32) > 0 {
                let a = ac(m);
                m.call(0x0021_3b00, &[a, 0x35e], &[]);
            }
            m.call(0x0020_21f8, &[o], &[]);
        }
        4 => {
            let b = bc(m);
            m.call(0x0021_3928, &[b, 0x359], &[]);
        }
        5 | 6 => {
            let b = bc(m);
            m.call(0x0021_3928, &[b, 0x35a], &[]);
        }
        7 => {
            let a = ac(m);
            m.call(0x0021_3928, &[a, 0x356], &[]);
            let b = bc(m);
            m.call(0x0021_3928, &[b, 0x356], &[]);
            let b = bc(m);
            m.call(0x0021_3928, &[b, 0x357], &[]);
        }
        8 => {
            let a0 = m.r32(o + 0xa8);
            let a0 = m.r32(a0);
            let v1 = m.r32(a0 + 0x40);
            let a1 = m.r32(o + 0xa0);
            if (v1 ^ 1) & 1 != 0 {
                let v = a1 | (m.r32(v1) & 1);
                m.w32(v1, v);
            } else {
                m.w32(a0 + 0x40, a1 | (v1 & 1));
            }
            m.call(0x0014_d1d0, &[T_NOW, 0], &[f4(1.0)]);
            m.call(0x001f_e5b8, &[0x003d_d040, 1, 1], &[]);
            let a = ac(m);
            m.call(0x0021_3928, &[a, 0x358], &[]);
        }
        9 => {
            m.call(0x0014_fe28, &[0x002d_e390], &[]);
            let a = ac(m);
            m.call(0x0021_4300, &[a, u32::MAX], &[]);
            let a = ac(m);
            vslot(m, a, 0x30, &[0x310], &[]);
            if m.r32(o + 0xa4) != 0 {
                let mut v = m.r32(o + 0x8c);
                let id = m.r32(v + 0xfc);
                if id != u32::MAX {
                    m.call(0x001f_e5f0, &[0x003d_d040, id, 0], &[]);
                    v = m.r32(o + 0x8c);
                }
                let h = m.r32(o + 0xa4);
                let id = m.r32(v + 0xfc);
                m.call(0x0014_7c20, &[h, 0x608, id], &[]);
            }
            let p = m.r32(o + 0x8c);
            m.call(0x0020_36e8, &[p, 1], &[]);
            m.call(0x0016_19a0, &[EV, 10, 0], &[]);
            m.w32(o + 0x8c, 0);
            if m.call(0x0020_22d8, &[o], &[]).0 != 0 {
                let a = ac(m);
                m.call(0x0021_3b00, &[a, 0x35d], &[]);
                if (m.r32(o + 0x74) as i32) > 0 {
                    let a = ac(m);
                    m.call(0x0021_3b00, &[a, 0x35e], &[]);
                }
            }
            m.call(0x0020_21f8, &[o], &[]);
        }
        _ => {}
    }
    let old = m.r32(o);
    m.w32(o, st);
    match st as i32 {
        0 => {
            m.w32(o + 0x8c, 0);
            m.w32(o + 0xc0, u32::MAX);
            m.w32(o + 0xbc, u32::MAX);
            m.call(0x0020_22a0, &[o], &[]);
            let a = ac(m);
            vslot(m, a, 0x30, &[u32::MAX], &[]);
            m.call(0x0020_1e88, &[o], &[]);
            m.call(0x0020_1f90, &[o], &[]);
            m.call(0x0016_19a0, &[EV, 9, 0], &[]);
        }
        1 => {
            let a = ac(m);
            m.call(0x0021_3928, &[a, 0x35d], &[]);
            let b = bc(m);
            vslot(m, b, 0x30, &[0x339], &[]);
        }
        2 => {
            let b = bc(m);
            vslot(m, b, 0x30, &[0x339], &[]);
        }
        3 | 4 => {
            let b = bc(m);
            let p = m.r32(o + 0x8c);
            m.w32(b + 0xf4, p);
            let b = bc(m);
            vslot(m, b, 0x30, &[0x337], &[]);
            let b = bc(m);
            m.call(0x0021_3b00, &[b, 0x359], &[]);
            let b = bc(m);
            m.call(0x0021_2e40, &[b, 0x359], &[f4(0.0)]);
        }
        5 | 6 => {
            let b = bc(m);
            m.call(0x0021_3b00, &[b, 0x35a], &[]);
            let b = bc(m);
            m.call(0x0021_2e40, &[b, 0x35a], &[f4(0.0)]);
        }
        7 => {
            let s = m.call(0x001f_bb90, &[0x52], &[]).0;
            m.call(0x001f_b2f8, &[s, 0], &[]);
            let p = m.r32(o + 0x8c);
            let v = m.r32(p + 0xf4);
            m.w32(o + 0xbc, v);
            let a = ac(m);
            m.call(0x0021_3b00, &[a, 0x356], &[]);
            let b = bc(m);
            m.call(0x0021_3b00, &[b, 0x356], &[]);
            let b = bc(m);
            m.call(0x0021_3b00, &[b, 0x357], &[]);
        }
        8 => {
            let a = ac(m);
            m.call(0x0021_3928, &[a, 0x35b], &[]);
            let a = ac(m);
            m.call(0x0021_3b00, &[a, 0x358], &[]);
            let a1 = m.r32(o + 0xa8);
            let a0 = m.r32(a1);
            let mut a0 = m.r32(a0 + 0x40);
            if (a0 ^ 1) & 1 != 0 {
                a0 = m.r32(a0);
            }
            m.w32(o + 0xa0, a0 & !1);
            let a0 = m.r32(a1);
            let v1 = m.r32(a0 + 0x40);
            if (v1 ^ 1) & 1 != 0 {
                let v = m.r32(v1) | 0x102;
                m.w32(v1, v);
            } else {
                m.w32(a0 + 0x40, v1 | 0x102);
            }
            m.call(0x001f_e5b8, &[0x003d_d040, 1, 0], &[]);
            m.call(0x0014_d1d0, &[T_NOW, 0], &[f4(0.0)]);
        }
        9 => {
            let b = bc(m);
            vslot(m, b, 0x30, &[0x339], &[]);
            let a = ac(m);
            vslot(m, a, 0x30, &[0x35c], &[]);
            let a = ac(m);
            m.call(0x0021_4300, &[a, 0], &[]);
        }
        10 => {
            m.w32(o + 0x8c, 0);
            m.w32(o + 0xc0, u32::MAX);
            m.w32(o + 0xbc, u32::MAX);
            let b = bc(m);
            vslot(m, b, 0x31, &[], &[]);
            let a = ac(m);
            vslot(m, a, 0x31, &[], &[]);
            m.call(0x0020_1f00, &[o], &[]);
            m.call(0x0016_19a0, &[EV, 0xb, 0], &[]);
        }
        -1 => {
            if old != 10 {
                m.call(0x0016_19a0, &[EV, 0xb, 0], &[]);
                let a = ac(m);
                if m.r32(a + 0x58) != 0 {
                    m.call(0x0021_2a70, &[a, 1], &[]);
                    let a = ac(m);
                    m.call(0x0021_2a50, &[a, 1], &[]);
                }
                let b = bc(m);
                if m.r32(b + 0x58) != 0 {
                    m.call(0x0021_2a70, &[b, 1], &[]);
                    let b = bc(m);
                    m.call(0x0021_2a50, &[b, 1], &[]);
                }
            }
        }
        _ => {}
    }
}

/// FUN_001dbef0(o): HUD slot refresh for the +0x24 widget (only when +0x24 -> +0x58 == 2). A 0x20-byte stack
/// record (memset FUN_002333b4, [0] = [3] = 1.0, [5] = y offset) is passed to FUN_00213cd8; slots
/// +0x250 .. FUN_001dbba0(o) (records at +0x4c + 8i, labels DAT_002c3588) shown via FUN_00213a08 / FUN_00213bd8.
pub fn ee_sub_1dbef0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if { let t_ = m.r32(o + 0x24); m.r32(t_ + 0x58) } != 2 {
        return;
    }
    let sp = stack_push(m, 0x20);
    let step = 0x002c_35b0;
    m.call(0x0023_33b4, &[sp, 0, 0x20], &[]);
    m.wf(sp, 1.0);
    m.wf(sp + 0xc, 1.0);
    let n = m.call(0x001d_bba0, &[o], &[]).0 as i32;
    let w = |m: &mut M| m.r32(o + 0x24);
    if (m.r32(o + 0x250) as i32) > 0 {
        let id = if m.r32(o + 0x40) == 0 { 0x4da } else { 0x4a1 };
        if m.r32(0x003d_a570) != 0 {
            let a = w(m);
            m.call(0x0021_3cd8, &[a, id, sp], &[]);
            let v = m.rf(sp + 0x14) + m.rf(step);
            m.wf(sp + 0x14, v);
        }
        let a = w(m);
        m.call(0x0021_3db0, &[a, id, o + 0x260], &[]);
    } else {
        let a = w(m);
        m.call(0x0021_3db0, &[a, u32::MAX, o + 0x260], &[]);
    }
    let mut i = m.r32(o + 0x250) as i32;
    while i < n {
        let rec = (o as i32 + 0x4c + 8 * i) as u32;
        let base = m.r32(o + 0x250) as i32;
        let lbl = m.r32((0x002c_3588i32 + 4 * (i - base)) as u32);
        match m.r32(o + 0x40) {
            0 => {
                let k = m.r32(rec);
                if k != u32::MAX {
                    let t = m.r32(0x002b_cd04u32.wrapping_add(k << 5));
                    match t {
                        7 => m.w32(rec + 4, 0x456),
                        8 => m.w32(rec + 4, 0x457),
                        9 => m.w32(rec + 4, 0x458),
                        _ => {}
                    }
                    let a = w(m);
                    m.call(0x0021_3bd8, &[a, 0x455, lbl], &[]);
                    let a = w(m);
                    let b = m.r32(rec + 4);
                    m.call(0x0021_3bd8, &[a, b, lbl], &[]);
                } else {
                    let a = w(m);
                    m.call(0x0021_3a08, &[a, 0x455, lbl], &[]);
                    let a = w(m);
                    let b = m.r32(rec + 4);
                    m.call(0x0021_3a08, &[a, b, lbl], &[]);
                    m.w32(rec + 4, u32::MAX);
                }
            }
            1 => {
                let a = w(m);
                if m.r32(rec) != u32::MAX {
                    m.call(0x0021_3bd8, &[a, 0x455, lbl], &[]);
                } else {
                    m.call(0x0021_3a08, &[a, 0x455, lbl], &[]);
                }
            }
            _ => {}
        }
        i += 1;
    }
    if (m.r32(o + 0x24c) as i32) >= 0 {
        let a = w(m);
        m.call(0x0021_3b00, &[a, 0x437], &[]);
        m.call(0x0023_33b4, &[sp, 0, 0x20], &[]);
        let d = (m.r32(o + 0x24c) as i32).wrapping_sub(m.r32(o + 0x250) as i32) as f32;
        let y = m.rf(step) * d;
        m.wf(sp, 1.0);
        m.wf(sp + 0xc, 1.0);
        m.wf(sp + 0x14, y);
        let a = w(m);
        m.call(0x0021_3cd8, &[a, 0x437, sp], &[]);
    } else {
        let a = w(m);
        m.call(0x0021_3928, &[a, 0x437], &[]);
    }
    if n < m.r32(o + 0x48) as i32 {
        let id = if m.r32(o + 0x40) == 0 { 0x4dd } else { 0x4a4 };
        if m.r32(0x003d_a570) != 0 {
            let a = w(m);
            m.call(0x0021_3cd8, &[a, id, sp], &[]);
            let v = m.rf(sp + 0x14) + m.rf(step);
            m.wf(sp + 0x14, v);
        }
        let a = w(m);
        m.call(0x0021_3db0, &[a, id, o + 0x264], &[]);
    } else {
        let a = w(m);
        m.call(0x0021_3db0, &[a, u32::MAX, o + 0x264], &[]);
    }
    let id = if m.r32(o + 0x30) == u32::MAX { 0x439 } else { 0x43b };
    let a = w(m);
    m.call(0x0021_3db0, &[a, id, o + 0x25c], &[]);
    stack_pop(m, 0x20);
}

/// FUN_001c03c0(o, bin): load a graph from the binary stream `bin` (FUN_00117c20 int, FUN_00117bd0 count,
/// FUN_00117e38 float, FUN_00117eb0 vector): nodes (+0xe0 / +0xe4, 0x20 each, edge lists via FUN_001ae1d0),
/// triangles (+0xe8 / +0xec, 3 node ptrs), points (+0xf0 / +0xf4, 0x10 each); FUN_001c0880(o + 0xe0); class
/// 0x2ca registers itself at 0x3e1a58; +0x114 = ++0x3e1200. Alloc requests are an 8-byte stack {size, 0x10}.
pub fn ee_sub_1c03c0<M: QMem + ?Sized>(m: &mut M, o: u32, bin: u32) {
    let sp = stack_push(m, 8);
    m.call(0x0023_1170, &[o, bin], &[]);
    let n = m.call(0x0011_7c20, &[bin], &[]).0;
    m.w32(sp + 4, 0x10);
    m.w32(sp, n << 5);
    m.w32(o + 0xe0, n);
    let p = m.call(0x001a_e2a0, &[sp], &[]).0;
    m.w32(o + 0xe4, p);
    let n = m.call(0x0011_7c20, &[bin], &[]).0;
    m.w32(sp + 4, 0x10);
    m.w32(o + 0xe8, n);
    m.w32(sp, n.wrapping_mul(0xc));
    let p = m.call(0x001a_e2a0, &[sp], &[]).0;
    m.w32(o + 0xec, p);
    let mut i = 0u32;
    while (i as i32) < m.r32(o + 0xe0) as i32 {
        let nd = m.r32(o + 0xe4) + (i << 5);
        i += 1;
        m.call(0x0011_7eb0, &[bin, nd], &[]);
        let c = m.call(0x0011_7bd0, &[bin], &[]).0;
        m.w32(nd + 0x10, c);
        m.w32(sp, (c << 3).wrapping_add(8));
        m.w32(sp + 4, 0x10);
        let e = m.call(0x001a_e1d0, &[sp], &[]).0;
        m.w32(nd + 0x14, e);
        let mut j = 0u32;
        while (j as i32) < m.r32(nd + 0x10) as i32 {
            let slot = m.r32(nd + 0x14) + (j << 3);
            j += 1;
            let k = m.call(0x0011_7c20, &[bin], &[]).0;
            let v = m.r32(o + 0xe4).wrapping_add(k << 5);
            m.w32(slot, v);
        }
    }
    let mut i = 0u32;
    while (i as i32) < m.r32(o + 0xe8) as i32 {
        let mut t = m.r32(o + 0xec) + i.wrapping_mul(0xc);
        i += 1;
        for _ in 0..3 {
            let k = m.call(0x0011_7c20, &[bin], &[]).0;
            let v = m.r32(o + 0xe4).wrapping_add(k << 5);
            m.w32(t, v);
            t += 4;
        }
    }
    let n = m.call(0x0011_7c20, &[bin], &[]).0;
    m.w32(sp + 4, 0x10);
    m.w32(sp, n << 4);
    m.w32(o + 0xf0, n);
    let p = m.call(0x001a_e2a0, &[sp], &[]).0;
    m.w32(o + 0xf4, p);
    let mut i = 0u32;
    while (i as i32) < m.r32(o + 0xf0) as i32 {
        let r = m.r32(o + 0xf4) + (i << 4);
        i += 1;
        for k in 0..3 {
            let f = m.call(0x0011_7e38, &[bin], &[]).1[0];
            m.wf(r + 4 * k, f);
        }
        let a = m.call(0x0011_7c20, &[bin], &[]).0;
        m.w16(r + 0xc, a as u16);
        let b = m.call(0x0011_7c20, &[bin], &[]).0;
        m.w16(r + 0xe, b as u16);
    }
    m.call(0x001c_0880, &[o + 0xe0], &[]);
    if m.r32(o + 0x18) == 0x2ca {
        m.w32(0x003e_1a58, o);
    }
    let c = m.r32(0x003e_1200).wrapping_add(1);
    m.w32(0x003e_1200, c);
    m.w32(o + 0x114, c);
    stack_pop(m, 8);
}

fn cp16<M: QMem + ?Sized>(m: &mut M, dst: u32, src: u32) {
    for k in (0..16).step_by(4) {
        let w = m.r32(src + k);
        m.w32(dst + k, w);
    }
}

/// FUN_001b6078(jt, st): Jt (player) state change. Exit action by old +0x13d0 (jump table 0x2a7230), then
/// +0x13d0 = st, +0x13d4 = now; entry action by st (table 0x2a72d0) choosing the anim (+0x1030 via
/// FUN_00130658 / FUN_001305b8) and the sound/effect (+0x1034 via FUN_001f15b8 / FUN_001f14f8); optional
/// anim start offset (FUN_0012f938(anim, 0x1ac) -> FUN_00131a90); finally vslot 0x59.
pub fn ee_sub_1b6078<M: QMem + ?Sized>(m: &mut M, o: u32, st: u32) {
    let cur = m.r32(o + 0x13d0);
    if st == cur {
        return;
    }
    match cur {
        0 => {
            let a = m.r32(o + 0x58);
            m.call(0x0023_03a0, &[o, a, 2], &[]);
            let a = m.r32(o + 0x5c);
            m.call(0x0023_03a0, &[o, a, 2], &[]);
            m.call(0x0014_2880, &[o], &[]);
        }
        1 | 10 => {
            let p = m.r32(o + 0x1030);
            let q = m.r32(p + 0x14);
            let v = m.rf(p + 0x1c) / m.rf(q + 0x74);
            m.wf(o + 0x1a0, v);
        }
        2 => {
            if m.r32(o + 0x108c) == 0 {
                let s0 = match st {
                    0 | 1 | 4 | 5 | 9 | 11 | 12 => m.r32(o + 0x1a50),
                    13 => {
                        let k = m.r32(o + 0x4c8) as i32;
                        if k == 1 {
                            0
                        } else if k <= 0 || k >= 5 || k < 3 {
                            m.r32(o + 0x1a50)
                        } else {
                            m.r32(o + 0x1a54)
                        }
                    }
                    _ => 0,
                };
                if s0 != 0 {
                    vslot(m, s0, 7, &[], &[]);
                    m.call(0x0015_c150, &[s0], &[]);
                    vslot(m, s0, 8, &[], &[]);
                }
            }
            m.call(0x0019_5fb8, &[o + 0x2c0, u32::MAX, 0], &[]);
        }
        3 => {
            m.call(0x001b_6da8, &[o, u32::MAX], &[]);
        }
        4 => {
            m.call(0x001b_93e8, &[o, u32::MAX], &[]);
        }
        5 => {
            m.call(0x001b_5658, &[o, u32::MAX], &[]);
        }
        6 => m.w32(o + 0x1530, 0),
        9 => {
            m.call(0x001b_5d30, &[o, u32::MAX], &[]);
        }
        11 => {
            m.call(0x001b_86b0, &[o, u32::MAX], &[]);
        }
        12 => {
            m.call(0x001b_8d30, &[o, u32::MAX], &[]);
        }
        13 => {
            m.call(0x0019_2bd8, &[o + 0x4c0, u32::MAX], &[]);
        }
        14 => {
            m.call(0x001a_0b40, &[o + 0xdf0, u32::MAX], &[]);
        }
        15 => {
            let p = m.r32(o + 0x191c);
            vslot(m, p, 8, &[], &[]);
            m.call(0x0022_06c8, &[o + 0x1910, u32::MAX], &[]);
            m.call(0x001f_5720, &[o, 0], &[]);
        }
        16..=18 => {
            let a = m.r32(o + 0x58);
            m.call(0x0023_03a0, &[o, a, 2], &[]);
            let a = m.r32(o + 0x5c);
            m.call(0x0023_03a0, &[o, a, 2], &[]);
        }
        20 => {
            m.call(0x0019_eab0, &[o + 0xf50, u32::MAX], &[]);
        }
        _ => {}
    }
    let old = m.r32(o + 0x13d0);
    m.w32(o + 0x13d0, st);
    let (mut s3, mut s5, mut s6, mut s4, mut s7) = (0u32, 0u32, 1u32, u32::MAX, 4u32);
    let t = m.rf(T_NOW);
    m.wf(o + 0x13d4, t);
    const DEF: u32 = 0x002a_2dd0;
    match st {
        0 => {
            let a = m.r32(o + 0x58);
            m.call(0x0023_03a0, &[o, a, 1], &[]);
            let a = m.r32(o + 0x5c);
            m.call(0x0023_03a0, &[o, a, 1], &[]);
            s5 = m.r32(o + 0x1028);
            cp16(m, o + 0x190, DEF);
            s4 = if old == 2 {
                0x178
            } else if old != 10 {
                0x175
            } else {
                0x176
            };
            let r = m.call(0x001f_0400, &[s5, s4], &[]).0 as i32;
            if r < 0 {
                s4 = u32::MAX;
            }
            s7 = m.call(0x0014_30c8, &[o], &[]).0;
            m.call(0x0014_2868, &[o, 1], &[]);
        }
        1 | 10 => {
            let (a, b) = if st == 1 { (0x1324, 0x1320) } else { (0x1544, 0x1540) };
            s3 = m.r32(o + a);
            m.call(0x0020_bd68, &[o + b], &[]);
            s6 = 9;
        }
        3 => {
            m.call(0x001b_6da8, &[o, 0], &[]);
            let a = m.r32(o + 0x1ae8);
            s3 = if a != 0 && m.r32(o + 0x13e8) != 0 { a } else { m.r32(o + 0x13e0) };
        }
        4 => {
            match m.r32(o + 0x144c) {
                0 => s3 = m.r32(o + 0x1440),
                1 => s3 = m.r32(o + 0x1444),
                _ => {}
            }
            cp16(m, o + 0x190, DEF);
            m.call(0x001b_93e8, &[o, 0], &[]);
        }
        5 | 9 => {
            let f = if st == 5 { 0x001b_5658 } else { 0x001b_5d30 };
            m.call(f, &[o, 0], &[]);
            vslot(m, o, 0x59, &[], &[]);
            return;
        }
        6 => s5 = m.r32(o + 0x1520),
        7 => {
            s3 = m.r32(o + 0x140c);
            if m.r32(o + 0x1ae0) != 0 && m.call(0x001d_aea8, &[0x11], &[]).0 != 0 {
                s3 = m.r32(o + 0x1ae0);
            }
            cp16(m, o + 0x190, DEF);
            m.call(0x001b_5e98, &[o], &[]);
            let q = m.rv(o + 0x1420);
            let p = m.call(0x001a_62e8, &[o], &[]).1;
            let d = [q[0] - p[0], q[1] - p[1], q[2] - p[2], q[3] - p[3]];
            m.call(0x0020_46b0, &[o], &[d]);
        }
        8 => {
            let a = m.r32(o + 0x1ae4);
            s3 = if a != 0 && m.r32(o + 0x143c) != 0 { a } else { m.r32(o + 0x1438) };
            cp16(m, o + 0x190, DEF);
        }
        14 => {
            m.call(0x0019_cd38, &[o + 0xf00, u32::MAX], &[]);
        }
        15 => {
            let p = m.r32(o + 0x191c);
            vslot(m, p, 7, &[], &[]);
            let v = m.r32(0x002d_d5bc);
            m.w32(o + 0x1920, v);
            m.call(0x0022_06c8, &[o + 0x1910, 0], &[]);
            m.call(0x001f_5720, &[o, 1], &[]);
            let q = m.rv(DEF);
            vslot(m, o, 0x22, &[], &[q]);
            return;
        }
        16..=18 => {
            if st == 16 {
                s5 = m.r32(o + 0x102c);
            }
            let a = m.r32(o + 0x58);
            m.call(0x0023_03a0, &[o, a, 1], &[]);
            let a = m.r32(o + 0x5c);
            m.call(0x0023_03a0, &[o, a, 1], &[]);
            cp16(m, o + 0x190, DEF);
        }
        20 => {
            m.call(0x0019_eab0, &[o + 0xf50, 0], &[]);
            return;
        }
        21 => {
            s3 = m.r32(o + 0x1aec);
            m.w32(o + 0x194, 0);
            s6 = 9;
            m.w32(o + 0x198, 0);
            m.w32(o + 0x190, 0);
        }
        _ => {}
    }
    m.call(0x0013_0658, &[o + 0x1030], &[]);
    m.call(0x001f_15b8, &[o + 0x1034], &[]);
    m.call(0x0013_05b8, &[s3, o, s6, o + 0x1030], &[f4(0.0), f4(1.0)]);
    m.call(0x001f_14f8, &[s5, o, s4, s6, s7, o + 0x1034], &[]);
    if s3 != 0 || s5 != 0 {
        let mut f = 0.0f32;
        if s3 != 0 {
            let p = m.call(0x0012_f938, &[s3, 0x1ac], &[]).0;
            if p != 0 {
                f = m.rf(p);
            }
        }
        if f != 0.0 {
            let a = m.r32(o + 0x1030);
            let x = m.rf(T_NOW) + f / m.rf(a + 0x40);
            m.call(0x0013_1a90, &[a], &[f4(x)]);
        }
    }
    vslot(m, o, 0x59, &[], &[]);
}

/// |a - cam|^2 over xyz (vsub.xyzw / vmul.xyz / vaddw.x / vadday.x / vmaddz.x)
fn d2(a: V4, b: V4) -> f32 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    d[0] * d[0] + d[1] * d[1] + d[2] * d[2]
}

/// FUN_00159c80(o): emitter update. FUN_0015a1c0 (when +0xe0), FUN_001a5e28; only when +0x118 == -1 and in
/// camera range (+0x38 * DAT_002ddf5c, against camera DAT_002ddfd0; FUN_0011a3b0(+0x11c) empty -> self, else
/// the +0x11c chain via +0x128 filtered by FUN_001a61a0) and not FUN_0015a170: count by mode +0xf0, spawn via
/// FUN_0015bd20(count, +0xe4, &{+0x150, +0x14c, o}); rate refresh at +0x138; FUN_00159a20(o).
pub fn ee_sub_159c80<M: QMem + ?Sized>(m: &mut M, o: u32) {
    const CAM: u32 = 0x002d_df40;
    if m.r32(o + 0xe0) != 0 {
        m.call(0x0015_a1c0, &[o], &[]);
    }
    m.call(0x001a_5e28, &[o], &[]);
    if m.r32(o + 0x118) != u32::MAX {
        return;
    }
    if m.call(0x0011_a3b0, &[o + 0x11c], &[]).0 != 0 {
        if m.r32(o + 0x10) != 0 {
            let r = m.rf(o + 0x38) * m.rf(CAM + 0x1c);
            let p = m.call(0x001a_62e8, &[o], &[]).1;
            let c = m.rv(CAM + 0x90);
            if r * r < d2(p, c) {
                return;
            }
        }
    } else {
        let mut s = m.r32(o + 0x11c);
        loop {
            if s == 0 {
                return;
            }
            if m.call(0x001a_61a0, &[s], &[]).0 != 0 {
                if m.r32(s + 0x10) == 0 {
                    break;
                }
                let r = m.rf(s + 0x38) * m.rf(CAM + 0x1c);
                let p = m.call(0x001a_62e8, &[s], &[]).1;
                let c = m.rv(CAM + 0x90);
                if d2(p, c) < r * r {
                    break;
                }
            }
            s = m.r32(s + 0x128);
        }
    }
    if m.call(0x0015_a170, &[o], &[]).0 != 0 {
        return;
    }
    let mut n: i32 = 0;
    let take = |m: &mut M, n: &mut i32| {
        let v = m.r32(o + 0xf4) as i32;
        if v >= 0 {
            if *n >= v {
                *n = v;
            }
            m.w32(o + 0xf4, v.wrapping_sub(*n) as u32);
        }
    };
    let mut skip = false;
    match m.r32(o + 0xf0) {
        0 => {
            let f = m.rf(o + 0x144) + m.rf(o + 0x130) * m.rf(o + 0x13c) * m.rf(T_DT);
            n = f as i32;
            m.wf(o + 0x144, f - n as f32);
            if n == 0 {
                skip = true;
            } else {
                take(m, &mut n);
            }
        }
        1 => {
            let c = (m.rf(o + 0x110) * m.rf(o + 0x13c)) as i32;
            let e = m.r32(o + 0xe4);
            let have = if m.r32(e + 4) != 0 {
                let a = m.r32(o + 0x14c);
                if a != 0 {
                    m.call(0x0011_a3c0, &[a + 0x15c], &[]).0 as i32
                } else {
                    0
                }
            } else {
                let a = m.r32(o + 0x150);
                if a != 0 {
                    m.r32(a + 0x444) as i32
                } else {
                    0
                }
            };
            let d = c.wrapping_sub(have);
            n = if d > 0 { d } else { 0 };
        }
        2 => {
            n = m.rf(o + 0x130) as i32;
            take(m, &mut n);
            let mut upd = true;
            if m.r32(o + 0x114) != 0 {
                m.call(0x0015_a150, &[o], &[]);
            } else {
                let r = m.call(0x0011_d448, &[], &[f4(0.0), f4(1.0)]).1[0];
                if r <= m.rf(o + 0x104) {
                    let (a, b) = (m.rf(o + 0x108), m.rf(o + 0x10c));
                    let v = m.call(0x0011_d448, &[], &[f4(a), f4(b)]).1[0];
                    m.call(0x0015_a138, &[o], &[f4(v)]);
                }
            }
            if upd {
                let (a, b) = (m.rf(o + 0xf8), m.rf(o + 0xfc));
                let v = m.call(0x0011_d448, &[], &[f4(a), f4(b)]).1[0];
                m.wf(o + 0x130, v);
                upd = false;
            }
            let _ = upd;
        }
        3 => {
            let f = m.rf(o + 0x144) + m.rf(o + 0x130) * m.rf(o + 0x13c) * m.rf(T_DT);
            n = f as i32;
            let nf = n as f32;
            m.wf(o + 0x144, f - nf);
            if 1.0 < nf {
                take(m, &mut n);
                let mut go = true;
                if m.r32(o + 0x114) != 0 {
                    m.call(0x0015_a150, &[o], &[]);
                } else {
                    let r = m.call(0x0011_d448, &[], &[f4(0.0), f4(1.0)]).1[0];
                    if r <= m.rf(o + 0x104) {
                        let (a, b) = (m.rf(o + 0x108), m.rf(o + 0x10c));
                        let v = m.call(0x0011_d448, &[], &[f4(a), f4(b)]).1[0];
                        m.call(0x0015_a138, &[o], &[f4(v)]);
                    } else {
                        go = false;
                    }
                }
                if go {
                    let (a, b) = (m.rf(o + 0xf8), m.rf(o + 0xfc));
                    let v = m.call(0x0011_d448, &[], &[f4(a), f4(b)]).1[0];
                    m.wf(o + 0x130, v * 60.0);
                }
            }
        }
        _ => {}
    }
    if !skip && n != 0 {
        let sp = stack_push(m, 0xc);
        m.call(0x0023_33b4, &[sp, 0, 0xc], &[]);
        m.w32(sp + 8, o);
        m.w32(sp + 4, o + 0x14c);
        m.w32(sp, o + 0x150);
        let l = { let t_ = m.r32(o + 0xe4); m.r32(t_ + 8) };
        let s0 = m.r32(l);
        if s0 == 0 && m.r32(l.wrapping_add(4)) == 0 {
            m.w32(l, o);
        }
        let e = m.r32(o + 0xe4);
        m.call(0x0015_bd20, &[n as u32, e, sp], &[]);
        let l = { let t_ = m.r32(o + 0xe4); m.r32(t_ + 8) };
        m.w32(l, s0);
        stack_pop(m, 0xc);
    }
    if m.rf(o + 0x138) <= m.rf(T_NOW) {
        let (a, b) = (m.rf(o + 0xf8), m.rf(o + 0xfc));
        let v = m.call(0x0011_d448, &[], &[f4(a), f4(b)]).1[0];
        m.wf(o + 0x130, v);
        let t = m.rf(T_NOW) + m.rf(o + 0x134);
        m.wf(o + 0x138, t);
    }
    m.call(0x0015_9a20, &[o], &[]);
}

/// FUN_00159500(o): emitter post-load. FUN_002586d8(o); shared list (+0xe4 -> +8) -> FUN_00149a50(world, o,
/// DAT_002a5140, 0); mode +0xf0 (-1 -> 0/1/3 from +0x104 / +0x114 / +0x110) seeds the rate (+0x130 / +0x134 /
/// +0x138) and the first burst (FUN_0015a150 / FUN_0015a138); box metrics +0x140 / +0x13c; +0x118 != -1 ->
/// find (FUN_0015e938) or spawn (FUN_0013e3c8 0x1d + param copy, FUN_001a6ca8, vslot 0x1d) the shared emitter,
/// FUN_0011a220(shared + 0x11c, o), clear flag bit 1 at +0x40.
pub fn ee_sub_159500<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0025_86d8, &[o], &[]);
    if { let u1_ = { let t_ = m.r32(o + 0xe4); m.r32(t_ + 8) }; m.r32(u1_) } != 0 {
        let k = m.r32(0x002a_5140);
        m.call(0x0014_9a50, &[0x003d_d490, o, k, 0], &[]);
    }
    if m.r32(o + 0xf0) == u32::MAX {
        let md = if m.rf(o + 0x104) != 0.0 || m.r32(o + 0x114) != 0 {
            3
        } else if m.rf(o + 0x110) == 0.0 {
            0
        } else {
            1
        };
        m.w32(o + 0xf0, md);
    }
    let md = m.r32(o + 0xf0) as i32;
    let burst = match md {
        2 | 3 => {
            let t = m.rf(0x002a_5148);
            m.wf(o + 0x138, t);
            let (a, b) = (m.rf(o + 0xf8), m.rf(o + 0xfc));
            let v = m.call(0x0011_d448, &[], &[f4(a), f4(b)]).1[0];
            m.wf(o + 0x130, if md == 3 { v * 60.0 } else { v });
            true
        }
        0 | 1 => {
            let a = m.rf(o + 0xf8);
            m.wf(o + 0x134, 1.0);
            let b = m.rf(o + 0xfc);
            m.wf(o + 0x138, 1.0);
            let v = m.call(0x0011_d448, &[], &[f4(a), f4(b)]).1[0];
            m.wf(o + 0x130, v);
            false
        }
        _ => false,
    };
    if burst {
        if m.r32(o + 0x114) != 0 {
            m.call(0x0015_a150, &[o], &[]);
        } else {
            let r = m.call(0x0011_d448, &[], &[f4(0.0), f4(1.0)]).1[0];
            if r <= m.rf(o + 0x104) {
                let (a, b) = (m.rf(o + 0x108), m.rf(o + 0x10c));
                let v = m.call(0x0011_d448, &[], &[f4(a), f4(b)]).1[0];
                m.call(0x0015_a138, &[o], &[f4(v)]);
            }
        }
    }
    m.wf(o + 0x13c, 1.0);
    let e = m.r32(o + 0xe4);
    if m.r32(e + 0x10) == 1 {
        let b = m.r32(e + 0x30);
        if { let t_ = m.r32(e + 8); m.r32(t_) } != 0 {
            let x = (m.rf(b + 0x10) - m.rf(b)).abs();
            let y = (m.rf(b + 0x14) - m.rf(b + 4)).abs();
            let z = (m.rf(b + 0x18) - m.rf(b + 8)).abs();
            let mx = if y < x && z < x {
                x
            } else if z < y {
                y
            } else {
                z
            };
            m.wf(o + 0x140, mx * 0.5);
        }
        if m.r32(o + 0x100) != 0 {
            let z = m.rf(b + 0x18) - m.rf(b + 8);
            let v = if m.rf(o + 0x110) == 0.0 {
                (m.rf(b + 0x14) - m.rf(b + 4)) * z * fb(0x38d1_b717)
            } else {
                (m.rf(b + 0x10) - m.rf(b)) * (m.rf(b + 0x14) - m.rf(b + 4)) * z * fb(0x3586_37bd)
            };
            m.wf(o + 0x13c, v);
        }
    }
    let id = m.r32(o + 0x118);
    if id == u32::MAX {
        return;
    }
    let mut s = m.call(0x0015_e938, &[0x003d_d490, 5, id, 0], &[]).0;
    if s == 0 {
        let id = m.r32(o + 0x118);
        s = m.call(0x0013_e3c8, &[0x1d, 0, id, u32::MAX], &[]).0;
        let sp = stack_push(m, 8);
        let e = m.r32(o + 0xe4);
        m.w32(sp, 0x100);
        m.w32(sp + 4, 0x10);
        let p = m.call(0x001a_e238, &[sp, e], &[]).0;
        m.w32(s + 0xe4, p);
        m.w32(p, 1);
        let l = { let t_ = m.r32(o + 0xe4); m.r32(t_ + 8) };
        if m.r32(l) != 0 {
            m.w32(sp + 4, 0x10);
            m.w32(sp, 0x14);
            let q = m.call(0x001a_e238, &[sp, l], &[]).0;
            let se = m.r32(s + 0xe4);
            m.w32(se + 8, q);
            let se = m.r32(s + 0xe4);
            m.w32(se + 0x20, u32::MAX);
            let se = m.r32(s + 0xe4);
            let q = m.r32(se + 8);
            m.w32(q, 0);
        }
        stack_pop(m, 8);
        for off in [0xf0u32, 0xf4, 0xf8, 0xfc, 0x100, 0x104, 0x108, 0x10c, 0x110, 0x114] {
            let v = m.r32(o + off);
            m.w32(s + off, v);
        }
        m.call(0x001a_6ca8, &[s, 1], &[]);
        let r = m.call(0x001a_6318, &[o], &[]).0;
        vslot(m, s, 0x1d, &[r], &[]);
    }
    m.w32(o + 0xe0, 0);
    m.call(0x0011_a220, &[s + 0x11c, o], &[]);
    let a = m.r32(o + 0x40);
    if (a ^ 1) & 1 != 0 {
        let v = m.r32(a) & !2;
        m.w32(a, v);
    } else {
        m.w32(o + 0x40, a & !2);
    }
}

/// `vmulax/vmadday/vmaddaz.xyz` by `l` + `vmaddw.xyz` row 3 (w lane keeps row 0's w)
fn xform<M: QMem + ?Sized>(m: &mut M, mat: u32, l: V4) -> V4 {
    let r: [V4; 4] = [m.rv(mat), m.rv(mat + 0x10), m.rv(mat + 0x20), m.rv(mat + 0x30)];
    let mut o = r[0];
    for i in 0..3 {
        o[i] = r[0][i] * l[0] + r[1][i] * l[1] + r[2][i] * l[2] + r[3][i];
    }
    o
}

fn len3(a: V4) -> f32 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

/// `|v| < 1e-4 ? DAT_002a2da0 : v * (1 / |v|)` (xyzw)
fn unit_or<M: QMem + ?Sized>(m: &mut M, v: V4) -> V4 {
    let d = len3(v);
    if d < fb(0x38d1_b717) {
        m.rv(0x002a_2da0)
    } else {
        let q = 1.0 / d;
        [v[0] * q, v[1] * q, v[2] * q, v[3] * q]
    }
}

/// FUN_001d6b00(ctl, obj): place / launch `obj` at the controller's attach point. P = pos(obj); world matrix =
/// FUN_0011c168(parent (+0x60 vslot 0x1f, else identity, base 0x2c44f0) + 0x40, local); point = M * +0x50;
/// optional FUN_002309e8 refresh when the clamped poly(+0x70) is non-zero; T = P + dir * clamp(poly(+0x90));
/// class 0x4f -> FUN_0013bc60 / FUN_00134320; vslot 7, FUN_001f5720 / FUN_001f57c0 / FUN_001a58d0; by class
/// 0x52 (FUN_001f4898), 0x51 (velocity vslot 0x22 + FUN_0011b7e0 + vslot 0x1d) or else FUN_001d6848 ->
/// vslot 0x22; random spin vslot 0x23 when +0xac > 0; class 0x50 -> vslot 0x4d(+0x60, &+0x50).
pub fn ee_sub_1d6b00<M: QMem + ?Sized>(m: &mut M, c: u32, o: u32) {
    let sp = stack_push(m, 0xc0);
    let s3 = sp + 0x40;
    let p = m.call(0x001a_62e8, &[o], &[]).1;
    let par = m.r32(c + 0x60);
    let s2 = if par != 0 {
        vslot(m, par, 0x1f, &[sp], &[]).0
    } else {
        m.call(0x0023_33b4, &[sp, 0, 0x40], &[]);
        m.wf(sp, 1.0);
        m.wf(sp + 0x3c, 1.0);
        m.wf(sp + 0x28, 1.0);
        m.wf(sp + 0x14, 1.0);
        0x002c_44f0
    };
    m.call(0x0011_c168, &[s3, s2 + 0x40, sp], &[]);
    let l = m.rv(c + 0x50);
    let w = xform(m, s3, l);
    let mut v3 = [w[0] - p[0], w[1] - p[1], w[2] - p[2], w[3] - p[3]];
    let d = len3(v3);
    let g = poly(m, c + 0x70, d);
    let (lo, hi) = (m.rf(c + 0x80), m.rf(c + 0x84));
    if clamp(g, lo, hi) != 0.0 {
        m.call(0x0023_09e8, &[s2, 0, s3, 0], &[]);
        m.call(0x0011_c168, &[sp + 0x80, s3, sp], &[]);
        let l = m.rv(c + 0x50);
        let w = xform(m, sp + 0x80, l);
        v3 = [w[0] - p[0], w[1] - p[1], w[2] - p[2], w[3] - p[3]];
    }
    let d = len3(v3);
    let g = poly(m, c + 0x90, d);
    let (lo, hi) = (m.rf(c + 0xa0), m.rf(c + 0xa4));
    let f2 = clamp(g, lo, hi);
    m.wv(s3, [f2, sext(f2), 0.0, 0.0]);
    let dir = unit_or(m, v3);
    let mut t = [0.0f32; 4];
    for i in 0..4 {
        t[i] = p[i] + if i == 3 { 1.0 } else { 0.0 } + dir[i] * f2;
    }
    if m.call(0x0013_2b10, &[o, 0x4f], &[]).0 != 0 {
        m.call(0x0013_bc60, &[o, c], &[]);
        m.call(0x0013_4320, &[0x003d_d490, o], &[]);
    }
    vslot(m, o, 7, &[], &[]);
    m.call(0x001f_5720, &[o, 0], &[]);
    m.call(0x001f_57c0, &[o, 0], &[]);
    m.call(0x001a_58d0, &[o, 0, 3], &[]);
    if m.call(0x0013_2b10, &[o, 0x52], &[]).0 != 0 {
        let f = m.rf(o + 0x260);
        let q = [f, 0.0, 0.0, 0.0];
        m.wv(s3, q);
        m.call(0x001f_4898, &[o], &[q]);
    } else if m.call(0x0013_2b10, &[o, 0x51], &[]).0 != 0 {
        let v = [t[0] - p[0], t[1] - p[1], t[2] - p[2], t[3] - p[3]];
        let s = m.rf(o + 0x260);
        m.wv(s3, [s, sext(s), 0.0, 0.0]);
        let u = unit_or(m, v);
        let vel = [u[0] * s, u[1] * s, u[2] * s, u[3] * s];
        vslot(m, o, 0x22, &[], &[vel]);
        m.call(0x0011_b7e0, &[s3], &[vel]);
        vslot(m, o, 0x1d, &[s3], &[]);
    } else {
        let q = m.call(0x001f_5860, &[o], &[]).1;
        let k = m.rf(c + 0xb0);
        let r = m.call(0x001d_6848, &[], &[p, t, q, f4(k)]).1;
        vslot(m, o, 0x22, &[], &[r]);
    }
    let ac = m.rf(c + 0xac);
    if 0.0 < ac {
        let a8 = m.rf(c + 0xa8);
        let f24 = m.call(0x0011_d448, &[], &[f4(a8), f4(ac)]).1[0];
        let f23 = m.call(0x0011_d448, &[], &[f4(-1.0), f4(1.0)]).1[0];
        let f20 = m.call(0x0011_d448, &[], &[f4(-1.0), f4(1.0)]).1[0];
        let f0 = m.call(0x0011_d448, &[], &[f4(-1.0), f4(1.0)]).1[0];
        let ax = [f23, f20, f0, sext(f0)];
        m.wv(s3, ax);
        let u = unit_or(m, ax);
        m.wv(s3, [f24, sext(f24), 0.0, 0.0]);
        let sp_v = [u[0] * f24, u[1] * f24, u[2] * f24, u[3] * f24];
        vslot(m, o, 0x23, &[], &[sp_v]);
    }
    if m.call(0x0013_2b10, &[o, 0x50], &[]).0 != 0 {
        let q = m.rv(c + 0x50);
        m.wv(s3, q);
        let a = m.r32(c + 0x60);
        vslot(m, o, 0x4d, &[a, s3], &[]);
    }
    stack_pop(m, 0xc0);
}
