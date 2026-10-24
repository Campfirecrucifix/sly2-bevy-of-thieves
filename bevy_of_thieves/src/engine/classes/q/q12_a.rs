//! WP-Q12 part A: file packages run_0012da30, run_0013fe18, run_00140750.
//! 1:1 from ghidra_functions(_v2).body on `QMem`; callees outside the package go through `m.call` by address
//! (ints in `a`, floats / quadwords in `v`, floats first, float result `.1[0]`).
#![allow(unused_variables, unused_mut, dead_code)]
use crate::engine::classes::q::q2mem::{QMem, V4};
use crate::engine::classes::q::q6::{rof, rov};
use crate::engine::classes::q::q6_runs::ee_sub_1ba9f8 as pools_init;
use crate::engine::classes::q::q7util::{add, f, sub, CLOCK};

const NEG1: u32 = u32::MAX;
/// FUN_0011d4d0(a, b, eps) with eps = 0x38d1b717 (1e-4)
fn floats_near<M: QMem + ?Sized>(m: &mut M, a: f32, b: f32) -> bool {
    m.call(0x0011_d4d0, &[], &[f(a), f(b), f(f32::from_bits(0x38d1_b717))]).0 != 0
}

// ---------------------------------------------------------------- run_0012da30 (C2ca820 constraint helpers)
/// FUN_0012da30(this, v): delta = v - quad(+0x20); +0x20 = v; when the owner (+8) has this as its +0x90 entry
/// and delta is not (FFloatsNear 1e-4) DAT_002a2dd0.xyz, add delta to owner +0xc0 and +0xe0 and
/// FUN_00230598(+4, owner)
pub fn ee_sub_12da30<M: QMem + ?Sized>(m: &mut M, o: u32, v: V4) {
    let old = m.rv(o + 0x20);
    let d = sub(v, old);
    m.wv(o + 0x20, v);
    let ow = m.r32(o + 8);
    if m.r32(ow + 0x90) == o {
        let q = rov(0x002a_2dd0);
        let mut near = false;
        if floats_near(m, q[0], d[0]) && floats_near(m, q[1], d[1]) {
            near = floats_near(m, q[2], d[2]);
        }
        if !near {
            let ow = m.r32(o + 8);
            let a = m.rv(ow + 0xc0);
            m.wv(ow + 0xc0, add(a, d));
            let a = m.rv(ow + 0xe0);
            m.wv(ow + 0xe0, add(a, d));
            let (a, b) = (m.r32(o + 4), m.r32(o + 8));
            m.call(0x0023_0598, &[a, b], &[]);
        }
    }
}
/// FUN_0012dbb0(a, b, c): r = FUN_0012db50(5, a, b, c, -1); FUN_00230018(a, b, r); r
pub fn ee_sub_12dbb0<M: QMem + ?Sized>(m: &mut M, a: u32, b: u32, c: u32) -> u32 {
    let r = m.call(0x0012_db50, &[5, a, b, c, NEG1], &[]).0;
    m.call(0x0023_0018, &[a, b, r], &[]);
    r
}
/// FUN_0012dd28(a, b, c, d): kind 7, last arg 0xe when b == *(d+8) else 0xd; +0x1c = d
pub fn ee_sub_12dd28<M: QMem + ?Sized>(m: &mut M, a: u32, b: u32, c: u32, d: u32) -> u32 {
    let mut k = 0xe;
    if b != m.r32(d + 8) {
        k = 0xd;
    }
    let r = m.call(0x0012_db50, &[7, a, b, c, k], &[]).0;
    m.w32(r + 0x1c, d);
    m.call(0x0023_0018, &[a, b, r], &[]);
    r
}
/// FUN_0012ddb0(a, b, c, d, e, p): kind 6; +0x1c = e; +0x20.. = 0x40 bytes copied from p
pub fn ee_sub_12ddb0<M: QMem + ?Sized>(m: &mut M, a: u32, b: u32, c: u32, d: u32, e: u32, p: u32) -> u32 {
    let r = m.call(0x0012_db50, &[6, a, b, c, d], &[]).0;
    m.w32(r + 0x1c, e);
    for i in 0..16 {
        let w = m.r32(p + 4 * i);
        m.w32(r + 0x20 + 4 * i, w);
    }
    m.call(0x0023_0018, &[a, b, r], &[]);
    r
}
/// FUN_0012de58(owner, xf, prio, $f12 = x, q3 (a3), q4 (t0)): FUN_0012db50(8, owner, xf, prio, 0x11);
/// +0x1c = x; +0x20 = q3, +0x30 = q4; FUN_00230018(owner, xf, r). PART r17: 1:1 vs asm (was Ghidra order)
pub fn ee_sub_12de58<M: QMem + ?Sized>(m: &mut M, a: u32, b: u32, c: u32, x: f32, q5: V4, q6: V4) -> u32 {
    let r = m.call(0x0012_db50, &[8, a, b, c, 0x11], &[]).0;
    m.wf(r + 0x1c, x);
    m.wv(r + 0x20, q5);
    m.wv(r + 0x30, q6);
    m.call(0x0023_0018, &[a, b, r], &[]);
    r
}
/// FUN_0012e0f0(this, q2, q3): +0x20 = q2, +0x30 = q3; when this is owner(+8)->+0x90: owner +0xc0 = q2, +0xd0 = q3
pub fn ee_sub_12e0f0<M: QMem + ?Sized>(m: &mut M, o: u32, q2: V4, q3: V4) {
    let ow = m.r32(o + 8);
    m.wv(o + 0x20, q2);
    m.wv(o + 0x30, q3);
    if o == m.r32(ow + 0x90) {
        m.wv(ow + 0xc0, q2);
        let ow = m.r32(o + 8);
        m.wv(ow + 0xd0, q3);
    }
}
/// FUN_0012e210(a): FUN_0011cd98(0x3e123c, a)
pub fn ee_sub_12e210<M: QMem + ?Sized>(m: &mut M, a: u32) {
    m.call(0x0011_cd98, &[0x003e_123c, a], &[]);
}
/// FUN_0012e4a8 / 12e9f0 / 12ef98 / 12f0f0: the pool refcount init of FUN_001ba9f8
pub fn ee_sub_12e4a8<M: QMem + ?Sized>(m: &mut M, on: u32, k: u32) {
    pools_init(m, on, k);
}
pub fn ee_sub_12e9f0<M: QMem + ?Sized>(m: &mut M, on: u32, k: u32) {
    pools_init(m, on, k);
}
pub fn ee_sub_12ef98<M: QMem + ?Sized>(m: &mut M, on: u32, k: u32) {
    pools_init(m, on, k);
}
pub fn ee_sub_12f0f0<M: QMem + ?Sized>(m: &mut M, on: u32, k: u32) {
    pools_init(m, on, k);
}
/// FUN_0012e5e0 / 12eb28 / 12f0d0 / 12f228: pools init (1, 0xffff)
pub fn ee_sub_12e5e0<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_12e4a8(m, 1, 0xffff);
}
pub fn ee_sub_12eb28<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_12e9f0(m, 1, 0xffff);
}
pub fn ee_sub_12f0d0<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_12ef98(m, 1, 0xffff);
}
pub fn ee_sub_12f228<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_12f0f0(m, 1, 0xffff);
}
/// FUN_0012e760 (C2cab90_ctor): FUN_0014fca8(this); vtable +0xc = 0x2cab90
pub fn ee_sub_12e760<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    m.call(0x0014_fca8, &[o], &[]);
    m.w32(o + 0xc, 0x002c_ab90);
    o
}
/// FUN_0012ee78: lazily +0xe0 = +0xe8 + 0x14 * +0xe4, then FUN_0012eee0(1.0f, 0, this, -1, 300, 3000)
pub fn ee_sub_12ee78<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let mut r = m.r32(o + 0xe0);
    if r == 0 {
        let v = m.r32(o + 0xe8).wrapping_add(m.r32(o + 0xe4).wrapping_mul(0x14));
        m.w32(o + 0xe0, v);
        m.call(0x0012_eee0, &[o, NEG1, 300, 3000], &[f(1.0), f(0.0)]); // PART r16 ABI: $f12 = 1.0, $f13 = 0.0
        r = m.r32(o + 0xe0);
    }
    r
}

// ---------------------------------------------------------------- run_0013fe18
/// FUN_0013fe18 / FUN_0013ff50: the pool refcount init of FUN_001ba9f8, (1, 0xffff)
pub fn ee_sub_13fe18<M: QMem + ?Sized>(m: &mut M, on: u32, k: u32) {
    pools_init(m, on, k);
}
pub fn ee_sub_13ff50<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_13fe18(m, 1, 0xffff);
}

// ---------------------------------------------------------------- run_00140750 (player HUD fade)
/// FUN_001409e8(this): 2 when FUN_001443d8() != 0, no +0x10c4 target, or FUN_00156698(+0x10c4, this, pos) == 0;
/// otherwise 0, 2 when any world-list (DAT_003e1444, link +0xfe0) object has FUN_00209a68(o, 0x1d) / (o, 0xb)
/// or 1 when its slot 88 (+0x2cc) accepts the target; 1 when a DAT_003e1468 (link +0x104) entry passes
/// FUN_00156fe8(e, target); finally FUN_00156780(target)
pub fn ee_sub_1409e8<M: QMem + ?Sized>(m: &mut M, o: u32) -> i32 {
    let mut r = 2;
    if m.call(0x0014_43d8, &[], &[]).0 == 0 {
        let mut t = 0;
        if m.r32(o + 0x10c4) != 0 {
            let p = m.call(0x001a_62e8, &[o], &[]).1;
            let a = m.r32(o + 0x10c4);
            t = m.call(0x0015_6698, &[a, o], &[p]).0;
        }
        if t == 0 {
            r = 2;
        } else {
            r = 0;
            let mut e = m.r32(0x003e_1444);
            while e != 0 {
                if m.call(0x0020_9a68, &[e, 0x1d], &[]).0 != 0 {
                    r = 2;
                    break;
                }
                if m.call(0x0020_9a68, &[e, 0xb], &[]).0 != 0 {
                    r = 2;
                    break;
                }
                if vs(m, e, 88, &[t]) != 0 {
                    r = 1;
                    break;
                }
                e = m.r32(e + 0xfe0);
            }
            let mut e = m.r32(0x003e_1468);
            if r == 0 {
                while e != 0 {
                    if m.call(0x0015_6fe8, &[e, t], &[]).0 != 0 {
                        r = 1;
                        break;
                    }
                    e = m.r32(e + 0x104);
                }
            }
            m.call(0x0015_6780, &[t], &[]);
        }
    }
    r
}
fn vs<M: QMem + ?Sized>(m: &mut M, o: u32, slot: u32, a: &[u32]) -> u32 {
    crate::engine::classes::q::q2mem::vslot(m, o, slot, a, &[]).0
}
/// float select as emitted: `f` when `c`, else bits 0
fn sel(c: bool, f: f32) -> f32 {
    if c { f } else { 0.0 }
}
/// FUN_00140750(this): fade values +0x10cc (target fade: FUN_001409e8 1 -> up at DAT_002bcc6c capped 1.0,
/// 0 -> down at DAT_002bcc70 floored 0, 2 -> 0), +0x10c8 (-1 / 0 / 1 state from +0xdf4 and the fade),
/// +0x10d0 (slot 97 result k; k = 2 when +0x10c8 != 1 and k == 1; -1 / 2 down at DAT_002bcc78, 1 up at
/// DAT_002bcc74, 0 -> 0); when +0x10d0 == 1.0: FUN_00156548(+0x10c4, pos, this, 0)
pub fn ee_sub_140750<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.r32(o + 0x10c4) == 0 {
        return;
    }
    let dt = m.rf(0x002d_ded4);
    match m.call(0x0014_09e8, &[o], &[]).0 as i32 as i64 {
        1 => {
            let x = m.rf(o + 0x10cc) + rof(0x002b_cc6c) * dt;
            let r = if x < 1.0 {
                x
            } else if x >= 1.0 {
                1.0
            } else {
                0.0
            };
            m.wf(o + 0x10cc, r);
        }
        0 => {
            let x = m.rf(o + 0x10cc) + rof(0x002b_cc70) * dt;
            m.wf(o + 0x10cc, sel(0.0 < x, x));
        }
        2 => m.w32(o + 0x10cc, 0),
        _ => {}
    }
    if m.r32(o + 0xdf4) as i32 == -1 {
        let x = m.rf(o + 0x10cc);
        if x == 0.0 {
            m.w32(o + 0x10c8, NEG1);
        } else if x == 1.0 {
            m.w32(o + 0x10c8, 1);
        } else if m.r32(o + 0x10c8) == 1 {
            m.w32(o + 0x10c8, if 0.0 < x { 1 } else { 0 });
        } else {
            m.w32(o + 0x10c8, 0);
        }
    } else {
        m.w32(o + 0x10c8, NEG1);
    }
    let k3 = vs(m, o, 97, &[]) as i32 as i64;
    let mut k = k3;
    if m.r32(o + 0x10c8) != 1 {
        k = 2;
        if k3 != 1 {
            k = k3;
        }
    }
    match k {
        0 => m.w32(o + 0x10d0, 0),
        -1 | 2 => {
            let mut x = m.rf(o + 0x10d0) + rof(0x002b_cc78) * dt;
            if x < 0.0 {
                x = 0.0;
            } else if 1.0 < x {
                x = 1.0;
            }
            m.wf(o + 0x10d0, x);
        }
        1 => {
            let mut x = m.rf(o + 0x10d0) + rof(0x002b_cc74) * dt;
            if 0.0 <= x {
                if 1.0 < x {
                    x = 1.0;
                }
            } else {
                x = 0.0;
            }
            m.wf(o + 0x10d0, x);
        }
        _ => {}
    }
    if m.rf(o + 0x10d0) == 1.0 {
        let p = m.call(0x001a_62e8, &[o], &[]).1;
        let a = m.r32(o + 0x10c4);
        m.call(0x0015_6548, &[a, o, 0], &[p]);
    }
}
