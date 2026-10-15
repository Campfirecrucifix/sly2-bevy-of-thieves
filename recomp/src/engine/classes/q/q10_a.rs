//! WP-Q10: class packages C2ca280, C2caf08, C2caf48, C2cc440, C2cddc0 (+ constant getters / empty slots of
//! the later Q10 class packages as data in `CONST_RET` / `NOPS`).
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem};
use crate::engine::classes::q::q7util::{f, CLOCK};

/// Q10 overrides whose whole body is `return <const>;` (the slot-5 class-name getters): (fn, value)
pub const CONST_RET: &[(u32, u32)] = &[
    (0x114370, 0x2bb6a0),
    (0x114270, 0x2bb560),
    (0x114620, 0x2bb9fc),
    (0x114550, 0x2bb8f8),
    (0x114340, 0x2bb664),
    (0x1144a0, 0x2bb81c),
    (0x1145b0, 0x2bb970),
    (0x1145d0, 0x2bb998),
    (0x1145a0, 0x2bb95c),
    (0x1144b0, 0x2bb830),
    (0x114450, 0x2bb7b8),
    (0x114400, 0x2bb754),
    (0x114440, 0x2bb7a4),
    (0x114420, 0x2bb77c),
    (0x1143f0, 0x2bb740),
    (0x1143e0, 0x2bb72c),
    (0x114300, 0x2bb614),
    (0x1142f0, 0x2bb600),
    (0x1142d0, 0x2bb5d8),
    (0x114290, 0x2bb588),
];
/// Q10 overrides whose body is `jr ra` only
pub const NOPS: &[u32] = &[0x1eef98, 0x1d9878, 0x1d98a8];

// ---- C2ca280 / C2caf48 shared refcount pair (ref +0x10) --------------------------------------------

/// FUN_001f8730 (C2ca280_v0) / FUN_001308a8 (C2caf48_v0): ++refcount (+0x10)
pub fn ref_inc<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let c = m.r32(o + 0x10);
    m.w32(o + 0x10, c.wrapping_add(1));
}
/// FUN_001f8740 (C2ca280_v1, pool 0x3e15cc) / FUN_001308b8 (C2caf48_v1, pool 0x3e1224): --refcount; when < 1:
/// FUN_00256ee0(o,0); FUN_002580a8(o+4,0); FUN_001334a8(o); FUN_0011cd98(pool, o)
pub fn ref_dec<M: QMem + ?Sized>(m: &mut M, o: u32, pool: u32) {
    let c = (m.r32(o + 0x10) as i32).wrapping_sub(1);
    m.w32(o + 0x10, c as u32);
    if c < 1 {
        m.call(0x0025_6ee0, &[o, 0], &[]);
        m.call(0x0025_80a8, &[o + 4, 0], &[]);
        m.call(0x0013_34a8, &[o], &[]);
        m.call(0x0011_cd98, &[pool, o], &[]);
    }
}
pub fn c2ca280_v1<M: QMem + ?Sized>(m: &mut M, o: u32) {
    ref_dec(m, o, 0x3e_15cc)
}
pub fn c2caf48_v1<M: QMem + ?Sized>(m: &mut M, o: u32) {
    ref_dec(m, o, 0x3e_1224)
}

// ---- C2caf08 --------------------------------------------------------------------------------------

/// FUN_00131c40 (v0)
pub fn c2caf08_v0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let a = m.r32(o + 0x20);
    if a == 0 {
        m.call(0x0012_ce98, &[o], &[]);
    } else {
        let p8 = m.r32(o + 8);
        let t = m.r32(p8 + 0xa8);
        m.w32(t + 0xd0, a);
        let b = m.r32(o + 0x24).wrapping_add(0x1c);
        m.w32(t + 0xd4, b);
    }
}
/// FUN_00131c90 (v1)
pub fn c2caf08_v1<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.r32(o + 0x20) == 0 {
        m.call(0x0012_ced8, &[o], &[]);
    } else {
        let p8 = m.r32(o + 8);
        let t = m.r32(p8 + 0xa8);
        m.w32(t + 0xd0, 0);
        m.w32(t + 0xd4, 0);
    }
}
/// FUN_00131cd8 (v3)
pub fn c2caf08_v3<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let c = m.r32(o + 0x1c);
    if c == 0 {
        let (p4, p8) = (m.r32(o + 4), m.r32(o + 8));
        m.w16(p8 + 0x9a, 0);
        m.call(0x0023_03a0, &[p4, p8, 0], &[]);
        m.call(0x0022_c818, &[p4, p8], &[]);
    } else {
        let p8 = m.r32(o + 8);
        let p24 = m.r32(o + 0x24);
        m.w32(p8 + 0x88, p24.wrapping_add(0x1c));
        m.w32(p8 + 0x8c, p24.wrapping_add(0x40));
        vslot(m, c, 1, &[p8], &[]);
    }
}
/// FUN_00131d68 (v4): dt in $f12; (a1, a2) forwarded
pub fn c2caf08_v4<M: QMem + ?Sized>(m: &mut M, o: u32, dt: f32, a1: u32, a2: u32) {
    let c = m.r32(o + 0x1c);
    if c == 0 {
        m.call(0x0012_cf30, &[o, a1, a2], &[f(dt)]);
    } else {
        let t = m.r32(o + 0x24);
        let y = m.rf(t + 0x40);
        let x = m.rf(t + 0x1c) + dt * y;
        let sp = stack_push(m, 8);
        m.wf(sp, x);
        m.wf(sp + 4, y);
        m.call(0x0013_17d8, &[t, sp, sp | 4], &[]);
        let (x2, y2) = (m.rf(sp), m.rf(sp + 4));
        vslot(m, c, 2, &[a1, a2], &[f(x2), f(y2)]);
        stack_pop(m, 8);
    }
}

// ---- C2caf48 --------------------------------------------------------------------------------------

/// FUN_00131738 (v3)
pub fn c2caf48_v3<M: QMem + ?Sized>(m: &mut M, o: u32, a1: u32, a2: u32) {
    vslot(m, o, 0, &[], &[]);
    let p = m.r32(o + 0x18);
    if p != 0 {
        vslot(m, p, 4, &[a1, a2], &[]);
    }
    m.call(0x0013_3508, &[o, a1, a2], &[]);
    vslot(m, o, 1, &[], &[]);
}

// ---- C2cc440 --------------------------------------------------------------------------------------

/// FUN_00144e20 (v6)
pub fn c2cc440_v6<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef68, &[o], &[]);
    m.w32(o + 0x34, 0x4220_0000);
    m.w32(o + 0x3c, 0x3f80_0000);
    m.w32(o + 0x38, 1);
}
/// FUN_00144e68 (v7)
pub fn c2cc440_v7<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef70, &[o], &[]);
    let p10 = m.r32(o + 0x10);
    let n = m.call(0x0020_5438, &[p10, 0xa8], &[]).0;
    let v = m.r32(o + 0x30);
    m.w32(n + 0x40, v);
    let v = m.r32(o + 0x34);
    m.w32(n + 100, v);
    let v = m.r32(o + 0x38);
    m.w32(n + 0xcc, v);
    let v = m.r32(o + 0x3c);
    m.w32(n + 0xd0, v);
    let v = m.r32(o + 0x40);
    m.call(0x0014_4f48, &[o, v], &[]);
    vslot(m, n, 7, &[], &[]);
}
/// FUN_001eef78 (v8): FUN_001ef0d8(o + 0x18)
pub fn c2cc440_v8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_f0d8, &[o + 0x18], &[]);
}

// ---- C2cddc0 --------------------------------------------------------------------------------------

/// FUN_00155560 (v6)
pub fn c2cddc0_v6<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef68, &[o], &[]);
    let p = m.r32(o + 0x10);
    m.call(0x0020_9890, &[p, 0x22], &[]);
    m.w32(o + 0x30, u32::MAX);
}
/// FUN_00155598 (v8)
pub fn c2cddc0_v8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef78, &[o], &[]);
    c2cddc0_set_state(m, o, u32::MAX);
    let p = m.r32(o + 0x10);
    m.call(0x0020_9970, &[p, 0x22], &[]);
    m.call(0x0020_8540, &[p, 0], &[]);
}
/// FUN_001555e0 (v7)
pub fn c2cddc0_v7<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef70, &[o], &[]);
    let p = m.r32(o + 0x10);
    let a = m.r32(o + 0x38);
    m.call(0x0020_8540, &[p, a], &[]);
    let d = (m.r32(p + 0x1018) as i32).wrapping_sub(m.r32(p + 0x33c) as i32);
    let t = if d < 0x5a {
        m.r32(o + 0x3c)
    } else {
        let r = c2cddc0_pick(m, o);
        m.w32(o + 0x3c, r);
        r
    };
    c2cddc0_set_state(m, o, (t == 0) as u32);
}
/// FUN_00155788: state change (+0x30): 0 -> spawn class 0xb5 child, 1 -> class 0xa5 child
pub fn c2cddc0_set_state<M: QMem + ?Sized>(m: &mut M, o: u32, st: u32) {
    let cur = m.r32(o + 0x30);
    if st == cur {
        return;
    }
    if cur == 0 {
        let t = m.r32(o + 0x3c);
        m.w32(t + 0x20, 1);
    }
    m.call(0x001e_f0d8, &[o + 0x18], &[]);
    m.w32(o + 0x30, st);
    let clk = m.r32(CLOCK);
    m.w32(o + 0x34, clk);
    let p10 = m.r32(o + 0x10);
    if st == 0 {
        let n = m.call(0x0020_5438, &[p10, 0xb5], &[]).0;
        let v = m.r32(o + 0x3c);
        m.w32(n + 0x38, v);
        let v = m.r32(o + 0x38);
        m.w32(n + 0x4c, v);
        vslot(m, n, 7, &[], &[]);
        let v = m.r32(p10 + 0x33c);
        m.w32(p10 + 0x1018, v);
    } else if st == 1 {
        let n = m.call(0x0020_5438, &[p10, 0xa5], &[]).0;
        let v = m.r32(o + 0x38);
        m.w32(n + 0x40, 1);
        m.w32(n + 0x30, v);
        vslot(m, n, 7, &[], &[]);
    }
}
/// FUN_00155660 (v9): while the state is 0 and the target (+0x38) is within 850 (xy) / 400 (|dz|) of the
/// owner (+0x10) and the child (+0x18) state (+0x30) != 0, switch to state 1; repeat once when the state changed
pub fn c2cddc0_v9<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef98, &[o], &[]);
    loop {
        if m.call(0x001e_f0a8, &[o], &[]).0 != 0 {
            return;
        }
        let st = m.r32(o + 0x30);
        if st == 0 {
            let c = m.r32(o + 0x18);
            let p10 = m.r32(o + 0x10);
            let a = m.call(0x001a_62e8, &[p10], &[]).1;
            let p38 = m.r32(o + 0x38);
            let b = m.call(0x001a_62e8, &[p38], &[]).1;
            let d: [f32; 4] = std::array::from_fn(|i| a[i] - b[i]);
            let dxy = (d[0] * d[0] + d[1] * d[1]).sqrt();
            if dxy < 850.0 && d[2].abs() < 400.0 && m.r32(c + 0x30) != 0 {
                c2cddc0_set_state(m, o, 1);
            }
        }
        if m.r32(o + 0x30) == st {
            return;
        }
    }
}
/// FUN_00155868: of the entries (stride 0x24, count p10+0x1010, base p10+0x1014) not skipped (DAT_002bdc50 == 0
/// && entry+0x20 != 0), the one whose nearer endpoint (+4 / +0x14 object positions, 3D distance to the target
/// +0x38 position) is farthest; start threshold DAT_002a5130 (-FLT_MAX); 0 when none
pub fn c2cddc0_pick<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let p38 = m.r32(o + 0x38);
    let pos = m.call(0x001a_62e8, &[p38], &[]).1;
    let p10 = m.r32(o + 0x10);
    let n = m.r32(p10 + 0x1010) as i32;
    let mut best = 0u32;
    let mut thr = f32::from_bits(m.r32(0x002a_5130));
    for i in 0..n.max(0) as u32 {
        let e = m.r32(p10 + 0x1014).wrapping_add(i * 0x24);
        if m.r32(0x002b_dc50) == 0 && m.r32(e + 0x20) != 0 {
            continue;
        }
        let (e4, e14) = (m.r32(e + 4), m.r32(e + 0x14));
        let a = m.call(0x001a_62e8, &[e4], &[]).1;
        let b = m.call(0x001a_62e8, &[e14], &[]).1;
        let len = |v: [f32; 4]| {
            let d: [f32; 3] = std::array::from_fn(|k| v[k] - pos[k]);
            (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
        };
        let dist = len(a).min(len(b));
        if thr < dist {
            thr = dist;
            best = e;
        }
    }
    best
}
