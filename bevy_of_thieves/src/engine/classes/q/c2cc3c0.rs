//! WP-Q5: class C2cc3c0 (vtable 0x2cc3c0) queue package: a brain "behaviour" record (+0x10 owner actor,
//! +0x18 brain node, +0x30 target, +0x40 offset, +0x50 last target position, +0x60 state 0/1/2, +0x64
//! state time, +0x68..+0x78 tuning, +0x80 found point, +0x90 chosen helper) that follows its target and
//! picks a helper actor; ported 1:1 against `QMem` (parents / out-of-package callees by address).
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};
use crate::engine::classes::q::q5util::{bit64, dist3, dist_xy, e8_dc_10, f4, poly, pos};

/// DAT_002dded0: game clock
pub const CLOCK: u32 = 0x002d_ded0;

/// C2cc3c0_v5 (slot 5): class descriptor
pub fn ee_sub_114630() -> u32 {
    0x002b_ba10
}

/// C2cc3c0_v6 (slot 6): parent init (FUN_001eef68) then state -1 and the tuning defaults
pub fn ee_sub_145010<M: QMem + ?Sized>(m: &mut M, s: u32) {
    m.call(0x001e_ef68, &[s], &[]);
    m.w32(s + 0x60, 0xffff_ffff);
    m.wf(s + 0x78, 1.2);
    m.wf(s + 0x68, 0.25);
    m.wf(s + 0x6c, 250.0);
    m.wf(s + 0x70, 350.0);
    m.wf(s + 0x74, 1.0);
}

/// C2cc3c0_v7 (slot 7): parent (FUN_001eef70) then state 0
pub fn ee_sub_145080<M: QMem + ?Sized>(m: &mut M, s: u32) {
    m.call(0x001e_ef70, &[s], &[]);
    ee_sub_145b18(m, s, 0);
}

/// C2cc3c0_v8 (slot 8): parent (FUN_001eef78) then state -1
pub fn ee_sub_1450b0<M: QMem + ?Sized>(m: &mut M, s: u32) {
    m.call(0x001e_ef78, &[s], &[]);
    ee_sub_145b18(m, s, 0xffff_ffff);
}

/// FUN_001450e0: nearest eligible helper: candidates from the owner's +0x1088 list (raw distance to the
/// target) and from the target's FUN_001f8240 group (distance curve 0x2bd310); behaviour bits 7, 0x10,
/// 0x16, 0x17, 0x2a, 0x2b in +0x2c0 exclude; the previous choice (+0x90) gets curve 0x2bd300, flag-3
/// actors curve 0x2bd320; smallest score wins (0 = none).
pub fn ee_sub_1450e0<M: QMem + ?Sized>(m: &mut M, s: u32) -> u32 {
    let mut best = 0u32;
    let mut bf = f32::MAX; // DAT_002a4e30
    let o = m.r32(s + 0x10);
    pos(m, o);
    let t = m.r32(s + 0x30);
    let tp = pos(m, t);
    let mask = [7, 0x10, 0x16, 0x17, 0x2a, 0x2b].iter().fold(0u64, |a, &k| a | bit64(k));
    let score = |m: &mut M, c: u32, d: f32| {
        let mut f = d;
        if c == m.r32(s + 0x90) {
            f = poly(m, 0x002b_d300, f);
        }
        if m.call(0x0020_9a68, &[c, 3], &[]).0 != 0 {
            f = poly(m, 0x002b_d320, f);
        }
        f
    };
    let l = m.r32(o + 0x1088);
    let mut i = 0;
    while i < m.r32(l + 0x28) as i32 {
        let c = m.r32(l + 0x2c + 4 * i as u32);
        if mask & m.r64(c + 0x2c0) == 0 {
            let d = dist3(tp, pos(m, c));
            let f = score(m, c, d);
            if f < bf {
                bf = f;
                best = c;
            }
        }
        i += 1;
    }
    let g = m.call(0x001f_8240, &[t], &[]).0;
    if g != 0 {
        let mut i = 0;
        while i < m.r32(g + 0x28) as i32 {
            let c = m.r32(g + 0x2c + 4 * i as u32);
            if mask & m.r64(c + 0x2c0) == 0 {
                let d = dist3(tp, pos(m, c));
                let d = poly(m, 0x002b_d310, d);
                let f = score(m, c, d);
                if f < bf {
                    bf = f;
                    best = c;
                }
            }
            i += 1;
        }
    }
    best
}

/// C2cc3c0_v9 (slot 9): update. Parent FUN_001eef98; settle the state machine (0 idle -> 1 when a point
/// is found / -> 2 when the helper is within range 0x2bd330; 2 -> 0 out of range 0x2bd338 or when the
/// helper's brain node (+0x18 +0x30 == -1) is done, by owner kind 0x35/0x36/0x37; 1 -> 0 when the owner's
/// +0x2dc is -1) until FUN_001ef0a8 or no change; in state 0 re-aim when the target moved > 200 (xy) and
/// drive the owner's +0x1030 speed by the target distance (lerp +0x74..+0x78 over +0x6c..+0x70).
pub fn ee_sub_1453f0<M: QMem + ?Sized>(m: &mut M, s: u32) {
    m.call(0x001e_ef98, &[s], &[]);
    let o = m.r32(s + 0x10);
    let t = m.r32(s + 0x30);
    let op = pos(m, o);
    let tp = pos(m, t);
    let dot = dist_xy(op, tp);
    let h = ee_sub_1450e0(m, s);
    let mut range = m.rf(0x002a_4e34);
    if h != 0 {
        let hp = pos(m, h);
        range = dist3(hp, op).min(dist3(hp, tp));
    }
    let mut st;
    loop {
        if m.call(0x001e_f0a8, &[s], &[]).0 != 0 {
            st = m.r32(s + 0x60);
            break;
        }
        st = m.r32(s + 0x60);
        match st as i32 {
            1 => {
                if m.r32(o + 0x2dc) == 0xffff_ffff {
                    ee_sub_145b18(m, s, 0);
                }
            }
            0 => {
                if ee_sub_1459c0(m, s) != 0 {
                    ee_sub_145b18(m, s, 1);
                } else if m.call(0x0011_d910, &[0x002b_d330], &[f4(range)]).0 != 0 {
                    m.w32(s + 0x90, h);
                    ee_sub_145b18(m, s, 2);
                }
            }
            2 => {
                if ee_sub_1459c0(m, s) != 0 {
                    ee_sub_145b18(m, s, 1);
                } else if m.call(0x0011_d910, &[0x002b_d338], &[f4(range)]).0 == 0 {
                    ee_sub_145b18(m, s, 0);
                } else {
                    let n = m.r32(s + 0x18);
                    let done = match m.call(0x0013_2ea0, &[o], &[]).0 {
                        0x36 => ee_sub_145da0(m, n),
                        0x35 => ee_sub_146790(m, n),
                        0x37 => ee_sub_146298(m, n),
                        _ => false,
                    };
                    if done {
                        ee_sub_145b18(m, s, 0);
                    }
                }
            }
            _ => {}
        }
        if m.r32(s + 0x60) == st {
            break;
        }
    }
    if st != 0 {
        return;
    }
    if e8_dc_10(m, o) == 0 {
        let n = m.r32(s + 0x18);
        m.w32(n + 0x40, 1);
    }
    let last = m.rv(s + 0x50);
    if dist_xy(last, tp) > m.rf(0x002b_d340) {
        ee_sub_1457a8(m, s);
    }
    let sp = m.r32(o + 0x1030);
    if sp != 0 {
        let (lo, hi) = (m.rf(s + 0x6c), m.rf(s + 0x70));
        let u = ((dot - lo) / (hi - lo)).clamp(0.0, 1.0);
        let f = (1.0 - u) * m.rf(s + 0x74) + u * m.rf(s + 0x78);
        m.call(0x0013_1a00, &[sp], &[f4(f)]);
    }
}

/// FUN_001457a8: re-aim: remember the target position (+0x50); goal = target position pushed back by
/// +0x68 along the target's facing (kind 0x33: axis-angle +0x1a8 about DAT_002a2dc0 applied to +0x190;
/// else slot 32 velocity scaled by +0x68) plus the target-frame offset +0x40; then either set the owner's
/// mover (+0x294) to a 2-point path (no surface) or route it through the surface (FUN_001c02d0), and
/// hand the mover to the brain node (FUN_00178d90).
pub fn ee_sub_1457a8<M: QMem + ?Sized>(m: &mut M, s: u32) {
    let t = m.r32(s + 0x30);
    let p = pos(m, t);
    m.wv(s + 0x50, p);
    let rows: [V4; 3];
    let mut goal: V4;
    let p50 = m.rv(s + 0x50);
    if m.call(0x0013_2b10, &[t, 0x33], &[]).0 != 0 {
        let buf = stack_push(m, 0x40);
        let axis = m.rv(0x002a_2dc0);
        let ang = m.rf(t + 0x1a8);
        m.call(0x0011_b500, &[buf], &[axis, f4(ang)]);
        rows = [m.rv(buf), m.rv(buf + 0x10), m.rv(buf + 0x20)];
        stack_pop(m, 0x40);
        let k = -m.rf(s + 0x68);
        let v = m.rv(t + 0x190);
        let mut r = v;
        for c in 0..3 {
            r[c] = rows[0][c] * v[0] + rows[1][c] * v[1] + rows[2][c] * v[2];
        }
        goal = [p50[0] + r[0] * k, p50[1] + r[1] * k, p50[2] + r[2] * k, 1.0 + p50[3] + r[3] * k];
    } else {
        let mp = m.call(0x001a_6318, &[t], &[]).0;
        rows = [m.rv(mp), m.rv(mp + 0x10), m.rv(mp + 0x20)];
        let k = m.rf(s + 0x68);
        let v = vslot(m, t, 32, &[], &[]).1;
        goal = [p50[0] + v[0] * k, p50[1] + v[1] * k, p50[2] + v[2] * k, 1.0 + p50[3] + v[3] * k];
    }
    let off = m.rv(s + 0x40);
    for c in 0..3 {
        goal[c] += rows[0][c] * off[0] + rows[1][c] * off[1] + rows[2][c] * off[2];
    }
    goal[3] += off[3];
    let o = m.r32(s + 0x10);
    let mv = m.r32(o + 0x294);
    let surf = e8_dc_10(m, o);
    let op = pos(m, o);
    if surf == 0 {
        let path = m.r32(mv + 0xf4);
        m.w32(mv + 0xe8, 2);
        m.wv(path, op);
        m.wv(path + 0x10, goal);
        m.call(0x001e_6180, &[mv], &[]);
    } else if ee_sub_145aa8(m, s) == surf {
        if m.call(0x001c_02d0, &[surf, mv], &[op, goal]).0 == 0 {
            m.call(0x001c_02d0, &[surf, mv], &[op, op]);
        }
    } else {
        m.call(0x001c_02d0, &[surf, mv], &[op, op]);
    }
    let n = m.r32(s + 0x18);
    m.call(0x0017_8d90, &[n, mv, 0], &[]);
}

/// FUN_001459c0: the target stands on a different surface than the owner (not a 0x2ca one; kind-0x34
/// targets only when their +0x2dc is -1), reachable (FUN_001c09a0) and FUN_00173610(100, -150) finds a
/// point near the target: store it at +0x80, 1; else 0.
pub fn ee_sub_1459c0<M: QMem + ?Sized>(m: &mut M, s: u32) -> u32 {
    let ts = ee_sub_145aa8(m, s);
    if ts == 0 {
        return 0;
    }
    let o = m.r32(s + 0x10);
    if ts == e8_dc_10(m, o) {
        return 0;
    }
    if m.r32(ts + 0x18) == 0x2ca {
        return 0;
    }
    let t = m.r32(s + 0x30);
    if m.call(0x0013_2b10, &[t, 0x34], &[]).0 != 0 && m.r32(t + 0x2dc) != 0xffff_ffff {
        return 0;
    }
    let p = pos(m, t);
    if ee_sub_1c09a0(m, ts, p, 0) == 0 {
        return 0;
    }
    let buf = stack_push(m, 0x10);
    m.wv(buf, p);
    let ok = m.call(0x0017_3610, &[buf, 0, 0], &[f4(100.0), f4(-150.0)]).0 != 0;
    let q = m.rv(buf);
    stack_pop(m, 0x10);
    if ok {
        m.wv(s + 0x80, q);
        return 1;
    }
    0
}

/// FUN_00145aa8: surface under the target: kind 0x30 -> its +0xe8/+0xdc/+0x10, else FUN_001c0d70(pos, 1)
pub fn ee_sub_145aa8<M: QMem + ?Sized>(m: &mut M, s: u32) -> u32 {
    let t = m.r32(s + 0x30);
    if m.call(0x0013_2b10, &[t, 0x30], &[]).0 == 0 {
        let p = pos(m, t);
        m.call(0x001c_0d70, &[1], &[p]).0
    } else {
        e8_dc_10(m, t)
    }
}

/// FUN_00145b18: enter state `st` (no-op when unchanged): reset the brain node (FUN_001ef0d8 +0x18),
/// stamp +0x64; 0: push behaviour 0xa8 (kind-0x32 targets: +0xc0 = target +0x150, +0xc4 = 8, +0xc8 =
/// random 0.1..0.3), re-aim, start it (slot 7); 1: FUN_0019f540(owner, +0x80); 2: by owner kind 0x35 /
/// 0x36 / 0x37 push behaviour 0xc5 / 0xc3 / 0xc4 with +0x38 = helper (+0x90) and start it.
pub fn ee_sub_145b18<M: QMem + ?Sized>(m: &mut M, s: u32, st: u32) {
    if st == m.r32(s + 0x60) {
        return;
    }
    m.call(0x001e_f0d8, &[s + 0x18], &[]);
    m.w32(s + 0x60, st);
    let c = m.r32(CLOCK);
    m.w32(s + 0x64, c);
    let o = m.r32(s + 0x10);
    match st as i32 {
        1 => {
            let p = m.rv(s + 0x80);
            m.call(0x0019_f540, &[o], &[p]);
        }
        0 => {
            let b = m.call(0x0020_5438, &[o, 0xa8], &[]).0;
            let t = m.r32(s + 0x30);
            if m.call(0x0013_2b10, &[t, 0x32], &[]).0 != 0 {
                let x = m.r32(t + 0x150);
                m.w32(b + 0xc0, x);
                m.w32(b + 0xc4, 8);
                let r = m.call(0x0011_d448, &[], &[f4(0.1), f4(0.3)]).1[0];
                m.wf(b + 200, r);
            }
            ee_sub_1457a8(m, s);
            vslot(m, b, 7, &[], &[]);
        }
        2 => {
            let k = match m.call(0x0013_2ea0, &[o], &[]).0 {
                0x35 => 0xc5,
                0x36 => 0xc3,
                0x37 => 0xc4,
                _ => return,
            };
            let b = m.call(0x0020_5438, &[o, k], &[]).0;
            let h = m.r32(s + 0x90);
            m.w32(b + 0x38, h);
            vslot(m, b, 7, &[], &[]);
        }
        _ => {}
    }
}

/// FUN_00145da0 / FUN_00146298 / FUN_00146790: brain node finished (+0x30 == -1)
pub fn ee_sub_145da0<M: QMem + ?Sized>(m: &mut M, n: u32) -> bool {
    m.r32(n + 0x30) == 0xffff_ffff
}
pub fn ee_sub_146298<M: QMem + ?Sized>(m: &mut M, n: u32) -> bool {
    ee_sub_145da0(m, n)
}
pub fn ee_sub_146790<M: QMem + ?Sized>(m: &mut M, n: u32) -> bool {
    ee_sub_145da0(m, n)
}

/// FUN_001c09a0: point `p` is on surface `obj` (FUN_001bf110 on +0xe0) and inside none of the up to 0x80
/// blockers FUN_001c1700 gathers there (FUN_001bc9f0): 1, else 0.
pub fn ee_sub_1c09a0<M: QMem + ?Sized>(m: &mut M, obj: u32, p: V4, a2: u32) -> u32 {
    if m.call(0x001b_f110, &[obj + 0xe0, a2], &[p]).0 == 0 {
        return 0;
    }
    let buf = stack_push(m, 0x200);
    let n = m.call(0x001c_1700, &[obj, 0, 0x80, buf], &[p, p]).0 as i32;
    let mut r = 1;
    for i in 0..n.max(0) as u32 {
        let b = m.r32(buf + 4 * i);
        if m.call(0x001b_c9f0, &[b], &[p]).0 != 0 {
            r = 0;
            break;
        }
    }
    stack_pop(m, 0x200);
    r
}
