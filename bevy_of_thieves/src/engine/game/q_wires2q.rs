//! WIRE-S2: 1:1 QMem ports of live-frontier natives (shard 3/4).
#![allow(unused)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

const T: u32 = 0x002d_ded0; // g_clock t
const CM: u32 = 0x002d_df40; // camera manager
const GS: u32 = 0x003d_d490; // game state / world

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

fn set<M: QMem + ?Sized>(m: &mut M, o: u32, s: u32) {
    m.call(0x0020_2768, &[o, s], &[]);
}

/// screen-space projection of a point via FUN_0014f308(CM, p): (x * 320 + 320, y * -246.4 + 246.4)
fn scr<M: QMem + ?Sized>(m: &mut M, p: V4) -> (f32, f32) {
    let r = m.call(0x0014_f308, &[CM], &[p]).1;
    let c = f32::from_bits(0x4376_6667);
    let n = f32::from_bits(0xc376_6667);
    (r[0] * 320.0 + 320.0, r[1] * n + c)
}

/// FUN_00202d38: binoc-recon update: timers, on-screen fit of target +0x8c (+0x90 / +0x94 / +0x98),
/// state machine on +0 (jump table 0x2a85c0) via FUN_00202768, then the pad-prompt tail.
pub fn ee_sub_202d38<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.r32(o + 0xac) == 0 || m.r32(o + 0xb0) == 0 {
        return;
    }
    if m.r32(o + 0xb4) != 0 {
        m.w32(o + 0xb4, 0);
        let a = m.r32(o + 0xa8);
        m.call(0x0019_cce8, &[a], &[]);
    }
    if m.rf(o + 0x9c) + 5.0 < m.rf(T) {
        let a = m.r32(o + 0xac);
        m.call(0x0021_3928, &[a, 0x35b], &[]);
    }
    let st = m.r32(o) as i32;
    if st < 8 && st >= 2 {
        m.call(0x0020_2540, &[o], &[]);
        m.call(0x0020_20c8, &[o], &[]);
        if m.call(0x0020_22d8, &[o], &[]).0 == 0 {
            set(m, o, 1);
        }
    }
    let p = m.r32(o + 0x8c);
    if p != 0 {
        let pos = m.call(0x001a_62e8, &[p], &[]).1;
        let p8c = m.r32(o + 0x8c); let k = m.rf(p8c + 0xe4);
        let up = m.rv(0x002d_dfb0);
        let top = [pos[0] + up[0] * k, pos[1] + up[1] * k, pos[2] + up[2] * k, pos[3] + up[3] * k];
        let (x1, y1) = scr(m, pos);
        let (x2, y2) = scr(m, top);
        let (cx, cy) = (m.rf(o + 0x10), m.rf(o + 0x14));
        let d1 = ((x1 - cx) * (x1 - cx) + (y1 - cy) * (y1 - cy)).sqrt();
        let d2 = ((x1 - x2) * (x1 - x2) + (y1 - y2) * (y1 - y2)).sqrt();
        let f1 = 1.0 - (d1 - 128.0) * f32::from_bits(0x3c00_0000);
        let f2 = d2 * f32::from_bits(0x3c08_8889);
        m.w32(o + 0x98, 1);
        m.wf(o + 0x94, f2);
        m.wf(o + 0x90, 0.0f32.max(1.0f32.min(f1)));
        if 1.0 < f2 {
            m.w32(o + 0x98, 0);
            m.wf(o + 0x94, 2.0 - f2);
        }
        let v = m.rf(o + 0x94).max(0.0);
        m.wf(o + 0x94, v);
    }
    state(m, o);
    tail(m, o);
}

fn state<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let p = m.r32(o + 0x8c);
    match m.r32(o) as i32 {
        0 => {
            let a = m.r32(o + 0xac);
            if m.call(0x0021_21e8, &[a, 0x314], &[]).0 != 0 {
                set(m, o, 1);
            }
        }
        1 => {
            m.call(0x0020_20c8, &[o], &[]);
            if m.call(0x0020_22d8, &[o], &[]).0 != 0 {
                set(m, o, 2);
            }
        }
        2 => {
            if p != 0 && m.r32(p + 0x108) != 2 {
                set(m, o, 4);
            }
        }
        3 => {
            if p == 0 {
                set(m, o, 2);
            } else if m.r32(p + 0x108) != 3 {
                set(m, o, 4);
            }
        }
        4 => {
            if p == 0 {
                set(m, o, 2);
            } else if m.r32(p + 0x108) == 3 {
                set(m, o, 3);
            } else if 1.0 <= m.rf(o + 0x90) {
                let s = if m.r32(o + 0x98) != 0 { 5 } else { 6 };
                set(m, o, s);
            }
        }
        5 | 6 => {
            if p == 0 {
                set(m, o, 2);
            } else if m.rf(o + 0x90) < 1.0 {
                set(m, o, 4);
            } else if 1.0 - m.rf(p + 0xe8) <= m.rf(o + 0x94) {
                set(m, o, 7);
            } else {
                let a = m.r32(o + 0xb0);
                m.call(0x0021_2e40, &[a, 0x35a], &[]);
            }
        }
        7 => {
            if p == 0 {
                set(m, o, 2);
            } else if m.rf(o + 0x90) < 1.0 || m.rf(o + 0x94) < 1.0 - m.rf(p + 0xe8) {
                let s = if m.r32(o + 0x98) == 0 { 6 } else { 5 };
                set(m, o, s);
            }
        }
        8 => state8(m, o),
        9 => {
            let a = m.r32(o + 0xac);
            if m.call(0x0021_21e8, &[a, 0x35c], &[]).0 != 0 {
                return;
            }
            set(m, o, 2);
            let mut all = true;
            let mut s1 = o + 0x34;
            let mut i = 0i32;
            while i < m.r32(o + 0x30) as i32 {
                let s0 = m.r32(s1);
                let v1 = m.call(0x0020_3758, &[s0], &[]).0;
                let e = m.r32(s0 + 0xec);
                let a0 = e == 0 || m.r32(e + 0x54) == 1;
                if v1 == 0 && a0 {
                    all = false;
                    break;
                }
                s1 += 4;
                i += 1;
            }
            if all {
                m.w32(o + 0xb4, 1);
            }
        }
        10 => {
            let a = m.r32(o + 0xac);
            if m.call(0x0021_21e8, &[a, 0x314], &[]).0 == 0 {
                set(m, o, u32::MAX);
            }
        }
        _ => {}
    }
}

fn state8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let ac = m.r32(o + 0xac);
    if m.call(0x0021_21e8, &[ac, 0x358], &[]).0 != 0 {
        return;
    }
    let a0 = m.r32(o + 0x8c);
    let hit = a0 != 0
        && m.r32(a0 + 0x108) == 1
        && 1.0 <= m.rf(o + 0x90)
        && 1.0 - m.rf(a0 + 0xe8) <= m.rf(o + 0x94);
    if hit {
        if m.r32(a0 + 0x10c) == 0 {
            set(m, o, 9);
            return;
        }
        let v = m.r32(a0 + 0x110);
        m.w32(o + 0xc0, v);
        set(m, o, 7);
    } else {
        let (off, s) = if a0 != 0 && m.r32(a0 + 0x108) != 1 {
            (0x110, 3)
        } else {
            let f2 = m.rf(o + 0x90);
            if 1.0 <= f2 && m.r32(o + 0x98) != 0 {
                (0x114, 5)
            } else if 1.0 <= f2 && m.r32(o + 0x98) == 0 {
                (0x118, 6)
            } else if a0 != 0 {
                (0x11c, 4)
            } else {
                (0x120, 2)
            }
        };
        let ac = m.r32(o + 0xac);
        let v = m.r32(ac + off);
        m.w32(o + 0xc0, v);
        set(m, o, s);
    }
    let ac = m.r32(o + 0xac);
    m.call(0x0021_3b00, &[ac, 0x35b], &[]);
    let t = m.rf(T);
    m.wf(o + 0x9c, t);
}

fn tail<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let a8 = m.r32(o + 0xa8);
    if m.r32(a8 + 0x20) == 1 {
        return;
    }
    let st = m.r32(o) as i32;
    if st < 2 {
        return;
    }
    let n = if st < 7 {
        10
    } else if st == 7 {
        0xfa
    } else {
        return;
    };
    if m.call(0x0014_fd50, &[0x002d_e2e0], &[]).0 != 0 {
        return;
    }
    let g = m.r32(0x002e_1e40);
    let a = m.r32(g + 0x150);
    if m.call(0x001c_46c0, &[a, n], &[f4(0.0)]).0 != 0 {
        set(m, o, 8);
    }
}

// SKIP 0x1a93c8: 0x3760-byte frame of 128-bit lq/sq records, VU0 macro heavy (vsqrt/vdiv Q, ACC chains,
// quad args via qmfc2 to FUN_00118668/FUN_00119390) — not ported.

fn lerp(a: V4, b: V4, t: f32) -> V4 {
    let u = 1.0 - t;
    [a[0] * u + b[0] * t, a[1] * u + b[1] * t, a[2] * u + b[2] * t, a[3] * u + b[3] * t]
}

/// FUN_00223a70(o, a (quad a1), b (quad a2), out): follow-camera cut placement: bound box of a/b/target ->
/// FUN_0020faa0 world query; probe FUN_0014eb70(.., 500); if b is blocked (FUN_0014ea08 t < 1) sweep yaw
/// offsets around dir(b - a) keeping the best scored hit; push out via FUN_0014eda0(CM, .., 0x223a10) and
/// write the final (possibly lerped) point to out.
pub fn ee_sub_223a70<M: QMem + ?Sized>(m: &mut M, o: u32, a: V4, b: V4, out: u32) {
    let sp = stack_push(m, 0xb0);
    let s5 = sp + 0x40;
    let tgt = m.r32(o);
    m.call(0x001a_e478, &[s5], &[]);
    let mut f20 = 0.0f32;
    let p = m.call(0x001a_62e8, &[tgt], &[]).1;
    let mut lo = [0.0f32; 4];
    for i in 0..3 {
        lo[i] = a[i].min(b[i].min(p[i]));
    }
    let p = m.call(0x001a_62e8, &[tgt], &[]).1;
    let mut hi = [0.0f32; 4];
    for i in 0..3 {
        hi[i] = a[i].max(b[i].max(p[i]));
    }
    m.call(0x0020_faa0, &[GS, 0x0014_f030, 0, sp + 0x90, sp + 0x94], &[lo, hi]);
    let (s0, s1) = (sp + 0x98, sp + 0x9c);
    let p = m.call(0x001a_62e8, &[tgt], &[]).1;
    let (w0, w1) = (m.r32(sp + 0x90), m.r32(sp + 0x94));
    let q = m.call(0x0014_eb70, &[w0, w1], &[p, f4(500.0)]).1;
    let (w0, w1) = (m.r32(sp + 0x90), m.r32(sp + 0x94));
    m.call(0x0014_ea08, &[w0, w1, s0, s1], &[q, b, f4(0.0)]);
    let r = if m.rf(sp + 0x9c) < 1.0 {
        m.w32(o + 0x34, 4);
        let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2], b[3] - a[3]];
        m.call(0x0011_ef08, &[sp + 0xa0, sp + 0xa4, sp + 0xa8], &[d]);
        let f27 = 1.25f32;
        let f22 = f32::from_bits(0x3ef1_463b);
        let mut f21 = m.rf(0x002a_9700);
        let f23 = 0.0f32;
        let f26 = f32::from_bits(0x3e92_3a14);
        let f24 = f32::from_bits(0x4049_0fdb);
        loop {
            let f12 = m.rf(sp + 0xa0) + f20;
            let (f13, f14) = (m.rf(sp + 0xa4), m.rf(sp + 0xa8));
            let rr = m.call(0x0011_e1a0, &[], &[f4(f12), f4(f13), f4(f14)]).1;
            let v2 = [a[0] + rr[0], a[1] + rr[1], a[2] + rr[2], a[3] + rr[3]];
            m.wv(sp + 0x50, rr);
            m.wv(sp + 0x60, v2);
            let (w0, w1) = (m.r32(sp + 0x90), m.r32(sp + 0x94));
            m.call(0x0014_ea08, &[w0, w1, s0, s1], &[q, v2, f4(f23)]);
            let f2 = m.rf(sp + 0x9c);
            let f1 = (f27 - f2) * (f20.abs() + f22);
            if f1 < f21 {
                f21 = f1;
                m.wv(sp + 0x70, lerp(q, v2, f2));
            }
            let mut f1 = -f20;
            if f20 <= f23 {
                f1 += f26;
            }
            f20 = f1;
            if f21 < (f1.abs() + f22) * 0.25 || !(f1 < f24) {
                break;
            }
        }
        m.rv(sp + 0x70)
    } else {
        b
    };
    m.wv(sp + 0x30, r);
    let f = m.rf(CM + 0x48);
    let (w0, w1) = (m.r32(sp + 0x90), m.r32(sp + 0x94));
    let pp = m.call(0x0014_eda0, &[CM, w0, w1, 0x0022_3a10], &[r, r, f4(f), f4(f)]).1;
    let (w0, w1) = (m.r32(sp + 0x90), m.r32(sp + 0x94));
    m.call(0x0014_ea08, &[w0, w1, s0, s1], &[r, pp, f4(0.0)]);
    let f2 = m.rf(sp + 0x9c);
    let v = if f2 == 1.0 {
        pp
    } else {
        let t = (m.rf(sp + 0x98) + f2) * 0.5;
        lerp(r, pp, t)
    };
    m.wv(out, v);
    m.call(0x001a_e4a0, &[s5, 2], &[]);
    stack_pop(m, 0xb0);
}
