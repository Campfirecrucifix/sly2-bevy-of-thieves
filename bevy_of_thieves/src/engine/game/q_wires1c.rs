//! WIRE-S1: 1:1 QMem queue ports of live-frontier natives (shard1/2).

use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem, V4};

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

/// FUN_00113aa0: returns class descriptor/table 0x2bab9c.
pub fn ee_sub_113aa0<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_ab9c
}

/// FUN_00113b60: returns class descriptor/table 0x2bac8c.
pub fn ee_sub_113b60<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_ac8c
}

/// FUN_00113c30: returns class descriptor/table 0x2bad90.
pub fn ee_sub_113c30<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_ad90
}

/// FUN_00113c90: returns class descriptor/table 0x2bae08.
pub fn ee_sub_113c90<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_ae08
}

/// FUN_00113f60: returns class descriptor/table 0x2bb18c.
pub fn ee_sub_113f60<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_b18c
}

/// FUN_00114090: returns class descriptor/table 0x2bb308.
pub fn ee_sub_114090<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_b308
}

/// FUN_00114220: returns class descriptor/table 0x2bb4fc.
pub fn ee_sub_114220<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_b4fc
}

/// FUN_001142f0: returns class descriptor/table 0x2bb600.
pub fn ee_sub_1142f0<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_b600
}

/// FUN_001143e0: returns class descriptor/table 0x2bb72c.
pub fn ee_sub_1143e0<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_b72c
}

/// FUN_00114420: returns class descriptor/table 0x2bb77c.
pub fn ee_sub_114420<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_b77c
}

/// FUN_001144a0: returns class descriptor/table 0x2bb81c.
pub fn ee_sub_1144a0<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_b81c
}

/// FUN_001145b0: returns class descriptor/table 0x2bb970.
pub fn ee_sub_1145b0<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_b970
}

/// FUN_00114670: returns class descriptor/table 0x2bba60.
pub fn ee_sub_114670<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x002b_ba60
}

/// FUN_00232408: snd thunk, FUN_00232838(0x18, 0, 0, 0, 0) (send IOP command 0x18, no wait).
pub fn ee_sub_232408<M: QMem + ?Sized>(m: &mut M) -> u32 {
    m.call(0x0023_2838, &[0x18, 0, 0, 0, 0], &[]).0
}

/// FUN_0017ee78(p): if FUN_0017eeb8(p) then FUN_0017dcb8(p->+0x18) else 0.
pub fn ee_sub_17ee78<M: QMem + ?Sized>(m: &mut M, p: u32) -> u32 {
    if m.call(0x0017_eeb8, &[p], &[]).0 == 0 {
        return 0;
    }
    let a = m.r32(p + 0x18);
    m.call(0x0017_dcb8, &[a], &[]).0
}

/// FUN_001bc278(a, b, c): sign (-1/0/1, eps at 0x2c0ab0) of the 2D cross product (a-b)x(b-c)
/// of quadword xy points.
pub fn ee_sub_1bc278<M: QMem + ?Sized>(m: &mut M, a: V4, b: V4, c: V4) -> i32 {
    let f2 = b[0] - c[0];
    let f1 = (a[1] - b[1]) * f2;
    let f0 = (a[0] - b[0]) * (b[1] - c[1]) - f1;
    let eps = m.rf(0x002c_0ab0);
    if f0 < -eps {
        -1
    } else if eps < f0 {
        1
    } else {
        0
    }
}

/// FUN_00215ba8(p, a1, d, q): RichText init: copies font/layout fields of descriptor `d` and qwords
/// q[1], q[2] into p, then FUN_00215ca8(p, 0, d, ctx). Returns p.
pub fn ee_sub_215ba8<M: QMem + ?Sized>(m: &mut M, p: u32, a1: u32, d: u32, q: u32) -> u32 {
    let gs = 0x003d_d490u32;
    let t1 = [m.r32(q + 0x10), m.r32(q + 0x14), m.r32(q + 0x18), m.r32(q + 0x1c)];
    let t2 = [m.r32(q + 0x20), m.r32(q + 0x24), m.r32(q + 0x28), m.r32(q + 0x2c)];
    m.w32(p, a1);
    m.w32(p + 0x6c, 0);
    m.w32(p + 0x70, 0);
    let f = m.rf(d + 0x24);
    m.wf(p + 0x74, f);
    let f = m.rf(d + 0x28);
    m.wf(p + 0x80, 1.0);
    m.wf(p + 0x7c, 1.0);
    m.wf(p + 0x78, f);
    let v = m.r32(d + 0x2c);
    m.w32(p + 0x90, v);
    let v = m.r32(d + 0x30);
    m.w32(p + 0x8c, v);
    let x = m.r32(d + 8);
    let x = m.r32(x + 0xc8);
    let f = m.rf(x + 0x9c);
    m.wf(p + 0x64, f);
    let x = m.r32(d + 8);
    let x = m.r32(x + 0xc8);
    let f = m.rf(x + 0xa0);
    m.wf(p + 0x68, f);
    let x = m.r32(d + 8);
    let v = m.r32(x + 0xa4);
    m.w32(p + 0x94, v);
    let f0 = m.rf(d + 0x20);
    m.wf(p + 0x84, f0);
    m.wf(p + 0x88, f0 * f32::from_bits(0x3ca0_0000));
    let g = m.r32(gs + 0x3be8);
    let a3 = m.r32(g + 0x54);
    m.w32(p + 0xc, a3);
    m.w32(p + 8, a3);
    let g = m.r32(gs + 0x3be8);
    let v = m.r32(g + 0x54);
    m.w32(p + 0x10, v.wrapping_add(0x40));
    let v = m.r32(d + 0x1c);
    for k in 0..4 {
        m.w32(p + 0x40 + 4 * k, t1[k as usize]);
    }
    m.w32(p + 0x18, v);
    for k in 0..4 {
        m.w32(p + 0x50 + 4 * k, t2[k as usize]);
    }
    m.call(0x0021_5ca8, &[p, 0, d, a3], &[]);
    p
}

/// FUN_001e7410(p, f, ctx, a3, t0, out_t, out_v): over the p->+0xec float table (count +0xe8) pick the
/// t minimizing f(ctx, t) (start bound 0x2b813c), then FUN_001e67b8(p, a3, t0, t) if a3|t0,
/// *out_t = t, *out_v = FUN_001e63c0(p, t).
#[allow(clippy::too_many_arguments)]
pub fn ee_sub_1e7410<M: QMem + ?Sized>(
    m: &mut M,
    p: u32,
    f: u32,
    ctx: u32,
    a3: u32,
    t0: u32,
    out_t: u32,
    out_v: u32,
) {
    let mut best_t = 0.0f32;
    let mut best = m.rf(0x002b_813c);
    let mut i: i32 = 0;
    while i < m.r32(p + 0xe8) as i32 {
        let tab = m.r32(p + 0xec);
        let t = m.rf(tab.wrapping_add((i as u32) << 2));
        let r = m.call(f, &[ctx], &[f4(t)]).1[0];
        if r < best {
            best = r;
            best_t = t;
        }
        i += 1;
    }
    if a3 != 0 || t0 != 0 {
        m.call(0x001e_67b8, &[p, a3, t0], &[f4(best_t)]);
    }
    if out_t != 0 {
        m.wf(out_t, best_t);
    }
    if out_v != 0 {
        let v = m.call(0x001e_63c0, &[p], &[f4(best_t)]).1[0];
        m.wf(out_v, v);
    }
}

/// FUN_00174490(h, req, pos, a3, t0): spawn from request: kind 0 -> FUN_001cb250(req->+8); kinds 1..2 ->
/// shuffle h's spawn points (+0x90 count, +0x94 list), place each at pos (slot +0xe8) / a3 (slot +0xf0)
/// until FUN_001cb250 yields an object; kind 1 also links it (+0x2e8, FUN_00205458 + two FUN_00176750
/// messages). Sets result+0x1000 = 1.
pub fn ee_sub_174490<M: QMem + ?Sized>(m: &mut M, h: u32, req: u32, pos: u32, a3: u32, t0: u32) -> u32 {
    let sp = stack_push(m, 0x90);
    let mut s4: u32 = 0;
    let kind = m.r32(req) as i32;
    'out: {
        if kind == 0 {
            let a = m.r32(req + 8);
            s4 = m.call(0x001c_b250, &[a, 0], &[]).0;
            break 'out;
        }
        if !(0..3).contains(&kind) {
            break 'out;
        }
        let f13 = m.rf(0x002b_e5f0);
        let f12 = m.rf(0x002b_e5f4);
        m.call(0x0017_3610, &[pos, 1, 0], &[f4(f12), f4(f13)]);
        let n = m.r32(h + 0x90) as i32;
        let mut i: i32 = 0;
        while i < n {
            m.w32(sp + 4 * i as u32, i as u32);
            i += 1;
        }
        let mut a2 = m.r32(h + 0x90) as i32;
        if a2 > 0 {
            let mut s1: i32 = 0;
            let mut s0 = sp;
            loop {
                let r = m.call(0x0011_d3f8, &[0, (a2 - 1) as u32], &[]).0;
                s1 += 1;
                let e = sp.wrapping_add(r << 2);
                let a1 = m.r32(s0);
                a2 = m.r32(h + 0x90) as i32;
                let v = m.r32(e);
                m.w32(s0, v);
                m.w32(e, a1);
                s0 += 4;
                if s1 >= a2 {
                    break;
                }
            }
        }
        if m.r32(h + 0x90) as i32 > 0 {
            let mut s2: i32 = 0;
            let mut s1 = sp;
            loop {
                let idx = m.r32(s1);
                let list = m.r32(h + 0x94);
                let o = m.r32((idx << 2).wrapping_add(list));
                let q = m.rv(pos);
                let vt = m.r32(o + 0xc);
                let adj = m.r16(vt + 0xe8) as i16 as i32 as u32;
                let fnp = m.r32(vt + 0xec);
                m.call(fnp, &[o.wrapping_add(adj)], &[q]);
                let vt = m.r32(o + 0xc);
                let adj = m.r16(vt + 0xf0) as i16 as i32 as u32;
                let fnp = m.r32(vt + 0xf4);
                m.call(fnp, &[o.wrapping_add(adj), a3], &[]);
                s4 = m.call(0x001c_b250, &[o, 0], &[]).0;
                s2 += 1;
                if s4 != 0 {
                    break;
                }
                if s2 >= m.r32(h + 0x90) as i32 {
                    break;
                }
                s1 += 4;
            }
        }
        let s2 = m.r32(req);
        if s2 != 1 {
            break 'out;
        }
        if s4 == 0 {
            stack_pop(m, 0x90);
            return 0;
        }
        let x = m.r32(req + 0x10);
        m.w32(x + 8, t0);
        let v = m.r32(req + 0xc);
        m.w32(s4 + 0x2e8, v);
        let s1 = m.call(0x0020_5458, &[s4, 0x9c], &[]).0;
        let buf = sp + 0x20;
        m.call(0x0023_33b4, &[buf, 0, 0x70], &[]);
        m.w32(buf, 0xffff_ffff);
        m.call(0x0017_6750, &[s1, buf], &[]);
        m.w32(buf, s2);
        m.call(0x0017_6750, &[s1, buf], &[]);
    }
    if s4 != 0 {
        m.w32(s4 + 0x1000, 1);
    }
    stack_pop(m, 0x90);
    s4
}

