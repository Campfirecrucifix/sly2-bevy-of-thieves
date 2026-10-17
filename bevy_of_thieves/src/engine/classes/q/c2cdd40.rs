//! WP-Q9: C2cdd40 (brain-driven approach state machine: state +0x30, since +0x34, owner +0x10, anim +0x18,
//! table +0x38, goal +0x40, target +0x4c, counter +0x50, handle +0x54) slot overrides.
use crate::engine::classes::q::q2mem::{QMem, V4};
use crate::engine::classes::q::q9util::*;

/// FUN_00155a68: pick a goal from the table at +0x38: pos(+0x4c) accepted by FUN_00227c90(tbl[+0xc], pos) -> tbl[+4];
/// else accepted with tbl[+0x1c] -> tbl[+0x14]; else tbl[+4]
pub fn ee_sub_155a68<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let t = m.r32(o + 0x38);
    let tg = m.r32(o + 0x4c);
    let q = m.call(0x001a_62e8, &[tg], &[]).1;
    let a = m.r32(t + 0xc);
    if m.call(0x0022_7c90, &[a], &[q]).0 & 0xff != 0 {
        return rr(m, o + 0x38, 4);
    }
    let t = m.r32(o + 0x38);
    let q = pos_at(m, o + 0x4c);
    let a = m.r32(t + 0x1c);
    let ok = m.call(0x0022_7c90, &[a], &[q]).0 & 0xff != 0;
    let t = m.r32(o + 0x38);
    if ok {
        m.r32(t + 0x14)
    } else {
        m.r32(t + 4)
    }
}
/// FUN_00155ae0(this, new): state change. Leaving 1: FUN_0020ac28(owner, 0, 0, quad DAT_002a2dd0); leaving 3: owner+0xf04 =
/// clock + FUN_0011d448(h+0x60, h+0x64), +0x54 = 0; FUN_001ef0d8(this+0x18); +0x30 = new; +0x34 = clock; entering 1: brain 0x9e,
/// target -> +0x38, run; 0: goal = FUN_00155a68, brain 0x9d (+0x3c = 0xb), face the goal (atan2 + pi -> FUN_00209818(owner+0xfd0)),
/// +0x3c = DAT_002a5138; 2: brain 0xa7 + route record (owner+0x294) ...; 3: brain 0xa4 (+0x3c = +0x54, +0x30 = target), +0x50++
pub fn ee_sub_155ae0<M: QMem + ?Sized>(m: &mut M, o: u32, new: u32) {
    let clock = m.rf(CLOCK);
    let cur = m.r32(o + 0x30);
    if new == cur {
        return;
    }
    let owner = m.r32(o + 0x10);
    if cur == 1 {
        m.call(0x0020_ac28, &[owner, 0, 0], &[rov(0x002a_2dd0)]);
    } else if cur == 3 {
        let h = m.r32(o + 0x54);
        let (a, b) = (m.rf(h + 0x60), m.rf(h + 0x64));
        let f = callf(m, 0x0011_d448, &[], &[s(a), s(b)]);
        let owner = m.r32(o + 0x10);
        m.wf(owner + 0xf04, clock + f);
        m.w32(o + 0x54, 0);
    }
    m.call(0x001e_f0d8, &[o + 0x18], &[]);
    m.w32(o + 0x30, new);
    let c = m.r32(CLOCK);
    m.w32(o + 0x34, c);
    let owner = m.r32(o + 0x10);
    if new == 1 {
        let b = m.call(0x0020_5438, &[owner, 0x9e], &[]).0;
        let t = m.r32(o + 0x4c);
        m.w32(b + 0x38, t);
        vcall(m, b, 0xc, 0x44, &[], &[]);
    } else if (new as i32) < 2 {
        if new == 0 {
            let g = ee_sub_155a68(m, o);
            m.w32(o + 0x40, g);
            let b = m.call(0x0020_5438, &[owner, 0x9d], &[]).0;
            m.w32(b + 0x3c, 0xb);
            let gp = pos_at(m, o + 0x40);
            let op = pos_at(m, o + 0x10);
            let d: V4 = std::array::from_fn(|i| gp[i] - op[i]);
            let ang = callf(m, 0x0024_58f0, &[], &[s(d[1]), s(d[0])]);
            let ow = m.r32(o + 0x10);
            m.call(0x0020_9818, &[ow + 0xfd0], &[s(ang + 3.1415927)]);
            vcall(m, b, 0xc, 0x44, &[], &[]);
            let k = ro32(0x002a_5138);
            m.w32(o + 0x3c, k);
        }
    } else if new == 2 {
        let b = m.call(0x0020_5438, &[owner, 0xa7], &[]).0;
        let c = rr(m, o + 0x10, 0x294);
        m.w32(c + 0xe8, 2);
        let p1 = m.r32(c + 0xf4);
        let op = pos_at(m, o + 0x10);
        m.wv(p1, op);
        let p2 = m.r32(c + 0xf4);
        let gp = pos_at(m, o + 0x40);
        m.wv(p2 + 0x10, gp);
        m.call(0x001e_6180, &[c], &[]);
        m.call(0x0017_8d90, &[b, c, 0], &[]);
        m.w32(b + 0xc0, 8);
        let t = m.r32(o + 0x4c);
        m.w32(b + 0x40, 1);
        m.w32(b + 0xc8, t);
        vcall(m, b, 0xc, 0x44, &[], &[]);
    } else if new == 3 {
        let b = m.call(0x0020_5438, &[owner, 0xa4], &[]).0;
        let h = m.r32(o + 0x54);
        m.w32(b + 0x3c, h);
        let t = m.r32(o + 0x4c);
        m.w32(b + 0x30, t);
        vcall(m, b, 0xc, 0x44, &[], &[]);
        let owner = m.r32(o + 0x10);
        let k = ro32(0x002a_5138);
        m.w32(owner + 0xf04, k);
        let n = m.r32(o + 0x50);
        m.w32(o + 0x50, n.wrapping_add(1));
    }
}
/// FUN_001559b0 (v6): parent FUN_001eef68; FUN_00209890(owner, 0x23); +0x30 = -1; +0x3c = DAT_002a5134
pub fn ee_sub_1559b0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef68, &[o], &[]);
    let ow = m.r32(o + 0x10);
    m.call(0x0020_9890, &[ow, 0x23], &[]);
    let k = ro32(0x002a_5134);
    m.w32(o + 0x30, u32::MAX);
    m.w32(o + 0x3c, k);
}
/// FUN_001559f8 (v8): parent FUN_001eef78; FUN_00155ae0(this, -1); FUN_00209970(owner, 0x23)
pub fn ee_sub_1559f8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef78, &[o], &[]);
    ee_sub_155ae0(m, o, u32::MAX);
    let ow = m.r32(o + 0x10);
    m.call(0x0020_9970, &[ow, 0x23], &[]);
}
/// FUN_00155a38 (v7): parent FUN_001eef70; FUN_00155ae0(this, 0)
pub fn ee_sub_155a38<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef70, &[o], &[]);
    ee_sub_155ae0(m, o, 0);
}
/// FUN_00155d90 (v9, update): parent FUN_001eef98; repeat until the state stops changing (FUN_001ef0a8(this) != 0 returns):
/// 1: after 2.5 s in it (since +0x48) and 0.1 s in the state: handle(+0x54) = FUN_001758a0(owner, target, 0) != 0 -> 3;
/// 0: FUN_00174fb8(anim) -> owner moves toward (pos(goal) - owner+0x240) (virtual 0xec / 0x11c / 0x124), +0x48 = clock, -> 1;
/// 2: FUN_00178d00(anim) -> 1; 3: +0x44 = clock; FUN_00175748(anim) -> new goal; changed -> 2 else 1
pub fn ee_sub_155d90<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef98, &[o], &[]);
    loop {
        if m.call(0x001e_f0a8, &[o], &[]).0 != 0 {
            return;
        }
        let st = m.r32(o + 0x30) as i32;
        let clock = m.rf(CLOCK);
        if st == 1 {
            if 2.5 < clock - m.rf(o + 0x48) && 0.1 < clock - m.rf(o + 0x34) {
                let (ow, t) = (m.r32(o + 0x10), m.r32(o + 0x4c));
                let h = m.call(0x0017_58a0, &[ow, t, 0], &[]).0;
                m.w32(o + 0x54, h);
                if h != 0 {
                    ee_sub_155ae0(m, o, 3);
                }
            }
        } else if st < 2 {
            let a = m.r32(o + 0x18);
            if st == 0 && m.call(0x0017_4fb8, &[a], &[]).0 != 0 {
                let g = m.r32(o + 0x40);
                let gp = m.call(0x001a_62e8, &[g], &[]).1;
                let ow = m.r32(o + 0x10);
                let c = m.rv(ow + 0x240);
                let d: V4 = std::array::from_fn(|i| gp[i] - c[i]);
                vcall(m, ow, 0xc, 0xec, &[], &[d]);
                vcall(m, ow, 0xc, 0x11c, &[], &[]);
                vcall(m, ow, 0xc, 0x124, &[], &[]);
                let c = m.r32(CLOCK);
                m.w32(o + 0x48, c);
                ee_sub_155ae0(m, o, 1);
            }
        } else if st == 2 {
            let a = m.r32(o + 0x18);
            if m.call(0x0017_8d00, &[a], &[]).0 != 0 {
                ee_sub_155ae0(m, o, 1);
            }
        } else if st == 3 {
            m.wf(o + 0x44, clock);
            let a = m.r32(o + 0x18);
            if m.call(0x0017_5748, &[a], &[]).0 != 0 {
                let mut u = 1;
                let g = ee_sub_155a68(m, o);
                if g != m.r32(o + 0x40) {
                    m.w32(o + 0x40, g);
                    u = 2;
                }
                ee_sub_155ae0(m, o, u);
            }
        }
        if m.r32(o + 0x30) as i32 == st {
            return;
        }
    }
}
