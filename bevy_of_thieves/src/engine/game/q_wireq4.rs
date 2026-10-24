//! WIRE-Q4: 1:1 queue ports (against `QMem`) of near-duplicate / frontier functions not yet live-dispatched.
//! Callees go through `m.call` / `vslot` (the live dispatch routes ported ones).
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

fn fv(f: f32) -> V4 {
    [f, 0.0, 0.0, 0.0]
}

/// FUN_001caec8(spawner, kind): copy record s4 = FUN_001cade0(spawner, kind) (0 -> 0); old owner released
/// (FUN_001caa60), owner / +0xe8 chain / +0x60..0xa0 transform copied; per +0xf0 slot (0x18 each, kind 5
/// skipped, kind 6 = record +0x10, else FUN_0015e678 lookup) the object goes to +0x14 and gets its
/// flag list applied (FUN_0013e818 over FUN_00117d50 iteration, FUN_00127b40 / FUN_00256ee0 parent);
/// then each +0xe0 template object gets slots 41 / 9 / 36 / 7 / 25 and the two 0xe messages
pub fn ee_sub_1caec8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let s1 = a0;
    let s4 = crate::engine::game::q_wirejob2::ee_sub_1cade0(m, s1, a1 as i32);
    if s4 == 0 {
        return 0;
    }
    let sp = stack_push(m, 0x100);
    let old = m.r32(s4);
    if old != 0 {
        m.call(0x001c_aa60, &[old, s4], &[]);
    }
    m.w32(s4, s1);
    let base = 0x003d_d490u32;
    let e8 = m.r32(s1 + 0xe8);
    m.w32(s4 + 4, e8);
    m.w32(sp + 0x50, s1);
    m.w32(s1 + 0xe8, s4);
    for k in (0..0x40).step_by(4) {
        let w = m.r32(s1 + 0x60 + k);
        m.w32(s4 + 0x10 + k, w);
    }
    let fp = m.call(0x001c_a930, &[s1, s4], &[]).0;
    let d = m.r32(s1 + 0xe0).wrapping_sub(m.r32(base + 0x3f08));
    let idx = (d.wrapping_mul(0x684b_da13) as i32) >> 2;
    let r = m.call(0x001c_9940, &[idx as u32, fp], &[]).0;
    m.w32(sp + 0x40, r);
    let t = m.r32(base + 0x3f0c);
    let s5 = m.call(0x001c_99d8, &[t, 1, sp + 0x40], &[]).0;
    let mut i = 0u32;
    while (i as i32) < m.r32(s1 + 0xec) as i32 {
        let s0 = m.r32(s1 + 0xf0).wrapping_add(i.wrapping_mul(0x18));
        let kind = m.r32(s0);
        i += 1;
        if kind == 5 {
            continue;
        }
        let obj;
        if kind == 6 {
            obj = m.r32(s5 + 0x10);
            m.w32(sp + 0x44, obj);
            if obj == 0 {
                continue;
            }
        } else {
            let r = m.call(0x0015_e678, &[base, 0xa05, kind, 0, s5, 1, sp + 0x44], &[]).0;
            if r != 1 {
                continue;
            }
            obj = m.r32(sp + 0x44);
        }
        m.w32(s0 + 0x14, obj);
        m.call(0x0011_8480, &[sp], &[]);
        let (c, e) = (m.r32(s0 + 0xc), m.r32(s0 + 0x10));
        m.call(0x0011_8500, &[sp, c, e, 0], &[]);
        loop {
            let n = m.call(0x0011_7d50, &[sp], &[]).0;
            if (n as i32) < 0 {
                break;
            }
            let a1 = 0x002a_c880u32.wrapping_add(n.wrapping_mul(0x28));
            let a2 = m.r32(a1 + 4) & 0x2031;
            let o = m.r32(sp + 0x44);
            m.call(0x0013_e818, &[o, a1, a2, sp], &[]);
        }
        let p = m.r32(s0 + 8);
        if p != 0 && m.r32(s1 + 0xe4) == 0 {
            let o = m.r32(sp + 0x44);
            let w = m.r32(o);
            if w != 0 {
                m.call(0x0012_7b40, &[p, w], &[]);
            }
            let p = m.r32(s0 + 8);
            m.call(0x0025_6ee0, &[o, p], &[]);
        }
        m.call(0x0011_84b8, &[sp, 2], &[]);
    }
    let mut j = 0u32;
    loop {
        let v1 = m.r32(s1 + 0xe0);
        let n = m.r32(v1);
        if (j as i32) >= n as i32 {
            break;
        }
        let arr = m.r32(v1 + 4);
        let o = m.r32((fp.wrapping_mul(n).wrapping_add(j)).wrapping_mul(4).wrapping_add(arr));
        j += 1;
        m.w32(sp + 0x48, o);
        let f = m.rf(s1 + 0x38);
        vslot(m, o, 41, &[], &[fv(f)]);
        let o = m.r32(sp + 0x48);
        let (x, y) = (m.r32(s1 + 0x10), m.r32(s1 + 0x50));
        vslot(m, o, 9, &[x, y, 0], &[]);
        let o = m.r32(sp + 0x48);
        vslot(m, o, 36, &[s1 + 0x60], &[]);
        let o = m.r32(sp + 0x48);
        vslot(m, o, 7, &[], &[]);
        let o = m.r32(sp + 0x48);
        vslot(m, o, 25, &[s1], &[]);
        m.w32(sp + 0x4c, sp + 0x48);
        vslot(m, s1, 2, &[0xe, 0, 1, sp + 0x4c, 0], &[]);
        m.w32(sp + 0x4c, sp + 0x50);
        let o = m.r32(sp + 0x48);
        vslot(m, o, 2, &[0xe, 0, 1, sp + 0x4c, 0], &[]);
    }
    stack_pop(m, 0x100);
    s4
}

/// FUN_001df3d0(o): per +0x19c path pair (0x150 each, two 0xa0 endpoints): an active endpoint (+4 == 1)
/// advances its segment time by +0x1c0 * dt (0x2dded4), wrapping to the next segment (FUN_00245a18
/// fmodf), sets +0x14 = p + v t + a t^2 / 2 and adds o+0xec to the SO moving list (FUN_0011a220 once,
/// +0x70 bit 2); a moved pair with +0x148 == 0 gets +0x140 / +0x144 from FUN_001def98
pub fn ee_sub_1df3d0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let s1 = a0;
    let sp = stack_push(m, 0xe0);
    let mut i = 0u32;
    while (i as i32) < m.r32(s1 + 0x198) as i32 {
        let s3 = m.r32(s1 + 0x19c).wrapping_add(i.wrapping_mul(0x150));
        i += 1;
        let s4 = s3 + 0x140;
        let mut moved = false;
        let mut s0 = s3;
        while s0 < s4 {
            if m.r32(s0 + 4) == 1 {
                let k = m.r32(s0 + 0x94);
                let mut e = s0 + k.wrapping_mul(0x14) + 0x1c;
                let f12 = m.rf(e + 0x10) + m.rf(s1 + 0x1c0) * m.rf(0x002d_ded4);
                let f13 = m.rf(e + 0xc);
                m.wf(e + 0x10, f12);
                if f13 < f12 {
                    let r = m.call(0x0024_5a18, &[], &[fv(f12), fv(f13)]).1[0];
                    let n = m.r32(s0 + 0x18) as i32;
                    let k = m.r32(s0 + 0x94).wrapping_add(1) as i32;
                    let k = if n == 0 { 0 } else { k.wrapping_rem(n) } as u32;
                    m.w32(s0 + 0x94, k);
                    e = s0 + k.wrapping_mul(0x14) + 0x1c;
                    m.wf(e + 0x10, r);
                }
                let t = m.rf(e + 0x10);
                let pos = m.rf(e) + m.rf(e + 4) * t + m.rf(e + 8) * t * t * 0.5;
                m.wf(s0 + 0x14, pos);
                let a1 = m.r32(s1 + 0xec);
                let fl = m.r32(a1 + 0x70);
                if fl & 2 == 0 {
                    m.w32(a1 + 0x70, fl | 2);
                    let a1 = m.r32(s1 + 0xec);
                    m.call(0x0011_a220, &[0x003e_11c4, a1], &[]);
                }
                moved = true;
            }
            s0 += 0xa0;
        }
        if moved && m.r32(s3 + 0x148) == 0 {
            m.call(0x001d_ef98, &[s1, s3, sp, sp + 0x20], &[]);
            let f = m.rf(sp + 0x20);
            m.wf(s3 + 0x140, f);
            m.wf(s3 + 0x144, f);
        }
    }
    stack_pop(m, 0xe0);
}

/// FUN_001cb250(spawner, alt): single-template spawners only ([+0xe0] count 1): FUN_001caec8(spawner,
/// alt ? 1 : 2); returns the template object FUN_001ca930 picks from +0xe0 (0 when nothing spawned)
pub fn ee_sub_1cb250<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let n = { let p_ = m.r32(a0 + 0xe0); m.r32(p_) };
    if n != 1 {
        return 0;
    }
    let kind = if a1 != 0 { n } else { 2 };
    let r = m.call(0x001c_aec8, &[a0, kind], &[]).0;
    if r == 0 {
        return 0;
    }
    let i = m.call(0x001c_a930, &[a0, r], &[]).0;
    let arr = { let p_ = m.r32(a0 + 0xe0); m.r32(p_ + 4) };
    m.r32(i.wrapping_mul(4).wrapping_add(arr))
}

/// FUN_0013aa70(o, src): FUN_001f6068(o, src) then FUN_0013bc60(o, o)
pub fn ee_sub_13aa70<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.call(0x001f_6068, &[a0, a1], &[]);
    m.call(0x0013_bc60, &[a0, a0], &[]);
}

/// FUN_001bc198(o): FUN_001a5848(o); detach from +0x1d8 (FUN_001bd218), FUN_001bd1e0(o); drop o from the
/// 0x3e1a4c list when present (FUN_0011a380 / FUN_0011a2e8)
pub fn ee_sub_1bc198<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5848, &[a0], &[]);
    let p = m.r32(a0 + 0x1d8);
    if p != 0 {
        m.call(0x001b_d218, &[p], &[]);
    }
    m.call(0x001b_d1e0, &[a0], &[]);
    if m.call(0x0011_a380, &[0x003e_1a4c, a0], &[]).0 != 0 {
        m.call(0x0011_a2e8, &[0x003e_1a4c, a0], &[]);
    }
}

/// FUN_001910f8(o): +0x68 = FUN_0013e3c8(0xf, 0, 0x148, -1) (FUN_00221268(_, 0)), FUN_00194118, two
/// FUN_001a0300 pairs, two FUN_0020b868, FUN_00137a30(world, 0xc, 3, +0x2f0, 0, 0, 0, +0x314),
/// FUN_001cc5e8(+0x440, world, world + 0x1030)
pub fn ee_sub_1910f8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let v = m.call(0x0013_e3c8, &[0xf, 0, 0x148, u32::MAX], &[]).0;
    m.w32(a0 + 0x68, v);
    m.call(0x0022_1268, &[v, 0], &[]);
    m.call(0x0019_4118, &[a0], &[]);
    let w = m.r32(a0);
    m.call(0x001a_0300, &[w, a0 + 0x70, a0 + 0x170], &[]);
    let w = m.r32(a0);
    m.call(0x001a_0300, &[w, a0 + 0x330, a0 + 0x430], &[]);
    m.call(0x0020_b868, &[a0 + 0x180], &[]);
    m.call(0x0020_b868, &[a0 + 0x240], &[]);
    let w = m.r32(a0);
    m.call(0x0013_7a30, &[w, 0xc, 3, a0 + 0x2f0, 0, 0, 0, a0 + 0x314], &[]);
    let w = m.r32(a0);
    m.call(0x001c_c5e8, &[a0 + 0x440, w, w.wrapping_add(0x1030)], &[]);
}

/// FUN_001ce888(o, kind, owner): owner 0 -> FUN_001cdcd0(0x3dd490, 0) (none -> 0); node = FUN_0011ccc0(o +
/// 0x4054) (none -> 0); r = FUN_001ce6d0(node, kind), r+8 = owner, FUN_0011a220(owner + 0x15c, r),
/// FUN_001ce9c0(r); returns r
pub fn ee_sub_1ce888<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) -> u32 {
    let mut s1 = a2;
    if s1 == 0 {
        s1 = m.call(0x001c_dcd0, &[0x003d_d490, 0], &[]).0;
        if s1 == 0 {
            return 0;
        }
    }
    let v = m.call(0x0011_ccc0, &[a0 + 0x4054], &[]).0;
    if v == 0 {
        return 0;
    }
    let s0 = m.call(0x001c_e6d0, &[v, a1], &[]).0;
    m.w32(s0 + 8, s1);
    m.call(0x0011_a220, &[s1 + 0x15c, s0], &[]);
    m.call(0x001c_e9c0, &[s0], &[]);
    s0
}

/// FUN_002021f8(o): per +0x74 entry i: FUN_00213bd8(+0xac, FUN_00203758(+0x78[i]) ? 0x33a : 0x33b,
/// DAT_002c3588[i])
pub fn ee_sub_2021f8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let mut i = 0u32;
    while (i as i32) < m.r32(a0 + 0x74) as i32 {
        let x = m.r32(a0 + 0x78 + 4 * i);
        let s0 = m.r32(0x002c_3588 + 4 * i);
        let id = if m.call(0x0020_3758, &[x], &[]).0 != 0 { 0x33a } else { 0x33b };
        let t = m.r32(a0 + 0xac);
        m.call(0x0021_3bd8, &[t, id, s0], &[]);
        i += 1;
    }
}

/// FUN_00162780(o): FUN_001a5c48(o); unless FUN_001633e8(o), FUN_00162670(o) -> FUN_001620b0(o, 5, +0x8c,
/// +0x90) + FUN_001623d0(o, 5, +0x94, +0x98, 1); +0x7c / +0xa4 from the +0xd0 / +0xe0 FUN_002581c0
/// lookups (0x3d4a60); +0xa8 = FUN_001d7e80(0x3e0ea8, +0xe8, 0x3d4a60); +0xb8 = FUN_0015e938(0x3dd490,
/// 0x405, +0xac, 0) unless +0xac == -1; +0xbc = FUN_0015e938(0x3dd490, 0x405, 0x379, 0)
pub fn ee_sub_162780<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let s1 = a0;
    m.call(0x001a_5c48, &[s1], &[]);
    if m.call(0x0016_33e8, &[s1], &[]).0 == 0 && m.call(0x0016_2670, &[s1], &[]).0 != 0 {
        let (a2, a3) = (m.r32(s1 + 0x8c), m.r32(s1 + 0x90));
        m.call(0x0016_20b0, &[s1, 5, a2, a3], &[]);
        let (a2, a3) = (m.r32(s1 + 0x94), m.r32(s1 + 0x98));
        m.call(0x0016_23d0, &[s1, 5, a2, a3, 1], &[]);
    }
    let sp = stack_push(m, 0x20);
    if m.r32(s1 + 0xd0) != 0 {
        m.call(0x0025_81c0, &[sp, s1 + 0xd0, 0x003d_4a60], &[]);
        let s0 = m.r32(sp + 4);
        m.call(0x0012_9ec8, &[sp], &[]);
        m.w32(s1 + 0x7c, s0);
    }
    if m.r32(s1 + 0xe0) != 0 {
        m.call(0x0025_81c0, &[sp, s1 + 0xe0, 0x003d_4a60], &[]);
        let f = m.r32(sp + 4);
        m.call(0x0012_9ec8, &[sp], &[]);
        m.w32(s1 + 0xa4, f);
    }
    stack_pop(m, 0x20);
    let v = m.call(0x001d_7e80, &[0x003e_0ea8, s1 + 0xe8, 0x003d_4a60], &[]).0;
    m.w32(s1 + 0xa8, v);
    let a2 = m.r32(s1 + 0xac);
    if a2 != u32::MAX {
        let v = m.call(0x0015_e938, &[0x003d_d490, 0x405, a2, 0], &[]).0;
        m.w32(s1 + 0xb8, v);
    }
    let v = m.call(0x0015_e938, &[0x003d_d490, 0x405, 0x379, 0], &[]).0;
    m.w32(s1 + 0xbc, v);
}

/// FUN_00135ad8(world, msg, arg): FUN_00140f90(world, msg, arg); msg 1: FUN_0018d698(world, slot, 0) for
/// each of the 6 +0x1824 slots (0xc each) whose +8 == arg; msg 9 from the world's own handle (+0x1030):
/// arg+8 == 0x216 spawns +0x186c = FUN_001cb250(+0x16b4 spawner, 0) (FUN_001cb488, +0x15c owner, slot 75(0));
/// from +0x16cc: arg+8 == 0x243 -> FUN_001357c0(world, 0x80000); else FUN_00136ed0(world + 0x1710, 9, arg)
pub fn ee_sub_135ad8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    let s1 = a0;
    m.call(0x0014_0f90, &[a0, a1, a2], &[]);
    if a1 == 1 {
        let mut s0 = s1 + 0x1824;
        for _ in 0..6 {
            if m.r32(s0 + 8) == a2 {
                m.call(0x0018_d698, &[s1, s0, 0], &[]);
            }
            s0 += 0xc;
        }
        return;
    }
    if a1 != 9 {
        return;
    }
    let h = m.r32(a2);
    if h == m.r32(s1 + 0x1030) {
        if m.r32(a2 + 8) != 0x216 {
            return;
        }
        let sp = { let p_ = m.r32(s1 + 0x16b4); m.r32(p_ + 4) };
        let o = m.call(0x001c_b250, &[sp, 0], &[]).0;
        m.w32(s1 + 0x186c, o);
        let sp = { let p_ = m.r32(s1 + 0x16b4); m.r32(p_ + 4) };
        m.call(0x001c_b488, &[sp, o], &[]);
        let w = { let p_ = m.r32(s1 + 0x16b4); m.r32(p_) };
        let o = m.r32(s1 + 0x186c);
        m.w32(o + 0x15c, w);
        vslot(m, o, 75, &[0], &[]);
    } else if h == m.r32(s1 + 0x16cc) {
        if m.r32(a2 + 8) == 0x243 {
            m.call(0x0013_57c0, &[s1, 0x8_0000], &[]);
        }
    } else {
        m.call(0x0013_6ed0, &[s1 + 0x1710, 9, a2], &[]);
    }
}

/// FUN_00212a70(o, state): no-op for the current +0x58 state; crossing the "< 2" line moves the
/// 0x3dd490 +0x3bec counter (+0xa4 >= 0x10) and the DAT_002dd318 counter (+0xb0 set; first in / last out
/// call FUN_0014d1d0(0x2dded0, 7, 0.0 / 1.0)); leaving state 0 runs FUN_002124f0; +0x5c = clock time,
/// +0x60 = DAT_002be538; state 0..4 enter: FUN_00212780 / FUN_00213898 / FUN_00213830(o, +0x70/+0x74/+0x78)
pub fn ee_sub_212a70<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let old = m.r32(a0 + 0x58);
    if old == a1 {
        return;
    }
    let (was, now) = ((old as i32) < 2, (a1 as i32) < 2);
    if m.r32(a0 + 0xa4) as i32 >= 0x10 {
        let c = 0x003d_d490 + 0x3bec;
        if was && !now {
            let v = m.r32(c).wrapping_add(1);
            m.w32(c, v);
        } else if !was && now {
            let v = m.r32(c).wrapping_sub(1);
            m.w32(c, v);
        }
    }
    if m.r32(a0 + 0xb0) != 0 {
        let c = 0x002d_d318;
        if was && !now {
            let v = m.r32(c).wrapping_add(1);
            m.w32(c, v);
            if v == 1 {
                m.call(0x0014_d1d0, &[0x002d_ded0, 7], &[fv(0.0)]);
            }
        } else if !was && now {
            let v = m.r32(c).wrapping_sub(1);
            m.w32(c, v);
            if v == 0 {
                m.call(0x0014_d1d0, &[0x002d_ded0, 7], &[fv(1.0)]);
            }
        }
    }
    if m.r32(a0 + 0x58) == 0 {
        m.call(0x0021_24f0, &[a0], &[]);
    }
    m.w32(a0 + 0x58, a1);
    let t = m.r32(0x002d_ded0 + 0x3c);
    m.w32(a0 + 0x5c, t);
    let v = m.r32(0x002b_e538);
    m.w32(a0 + 0x60, v);
    match a1 {
        0 => {
            m.call(0x0021_2780, &[a0], &[]);
        }
        1 => {
            m.call(0x0021_3898, &[a0], &[]);
        }
        2..=4 => {
            let x = m.r32(a0 + 0x70 + 4 * (a1 - 2));
            m.call(0x0021_3830, &[a0, x], &[]);
        }
        _ => {}
    }
}

/// FUN_001c8030(o): FUN_001a5c48(o); +0x100 = next 0x3dd490 +0x4760 id; FUN_001c7bb8(id) -> slot 8(o)
pub fn ee_sub_1c8030<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001a_5c48, &[a0], &[]);
    let c = 0x003d_d490 + 0x4760;
    let id = m.r32(c);
    m.w32(a0 + 0x100, id);
    m.w32(c, id.wrapping_add(1));
    let id = m.r32(a0 + 0x100);
    if m.call(0x001c_7bb8, &[id], &[]).0 != 0 {
        vslot(m, a0, 8, &[], &[]);
    }
}

/// FUN_00179aa0(o): FUN_001eef70; +0x40 = target +0x2e8, else +0x2ec when +0xfe8 bit 1, else state 2
/// (FUN_00179d30(o, 2)); a +0x2dc target runs FUN_00170e90(+0x40, FUN_001a62e8(t), t + 0x2dc); then
/// FUN_00179d30(o, FUN_00209098(t) ? 3 : 0)
pub fn ee_sub_179aa0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001e_ef70, &[a0], &[]);
    let t = m.r32(a0 + 0x10);
    let v = m.r32(t + 0x2e8);
    if v != 0 {
        m.w32(a0 + 0x40, v);
    } else if m.r32(t + 0xfe8) & 2 != 0 {
        let v = m.r32(t + 0x2ec);
        m.w32(a0 + 0x40, v);
    } else {
        m.w32(a0 + 0x40, 0);
        m.call(0x0017_9d30, &[a0, 2], &[]);
        return;
    }
    let t = m.r32(a0 + 0x10);
    if m.r32(t + 0x2dc) != 0 {
        let v = m.call(0x001a_62e8, &[t], &[]).1;
        let t = m.r32(a0 + 0x10);
        let x = m.r32(a0 + 0x40);
        m.call(0x0017_0e90, &[x, t + 0x2dc], &[v]);
    }
    let t = m.r32(a0 + 0x10);
    let s = if m.call(0x0020_9098, &[t], &[]).0 != 0 { 3 } else { 0 };
    m.call(0x0017_9d30, &[a0, s], &[]);
}

/// FUN_0013ff70(world): FUN_00203948; sub-object inits (FUN_00195950 +0x2c0, FUN_00191090 +0x4c0,
/// FUN_0019f3e8 +0x9e0, FUN_001a0b18 +0xdf0, FUN_0019cbf8 +0xf00, FUN_0021f2e0 +0x11b0, FUN_0019de10 +0xf50,
/// FUN_00144638 +0xfd0, each (sub, world)); default tunables; FUN_001f7918(world, 0x3ffffff, 0) and
/// (world, 0x10ff00, 8)
pub fn ee_sub_13ff70<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let s0 = a0;
    m.call(0x0020_3948, &[s0], &[]);
    for (off, f) in [
        (0x2c0, 0x0019_5950),
        (0x4c0, 0x0019_1090),
        (0x9e0, 0x0019_f3e8),
        (0xdf0, 0x001a_0b18),
        (0xf00, 0x0019_cbf8),
        (0x11b0, 0x0021_f2e0),
        (0xf50, 0x0019_de10),
        (0xfd0, 0x0014_4638),
    ] {
        m.call(f, &[s0 + off, s0], &[]);
    }
    m.wf(s0 + 0x103c, 20.0);
    m.wf(s0 + 0x1048, 125.0);
    m.wf(s0 + 0x10d4, 1.0);
    m.wf(s0 + 0x1138, 1.5);
    m.wf(s0 + 0x113c, 2.5);
    m.wf(s0 + 0x1140, 3.0);
    m.wf(s0 + 0x1144, 5.0);
    m.w32(s0 + 0x12f0, 1);
    m.wf(s0 + 0x1020, 100.0);
    m.w32(s0 + 0x1300, 0x64);
    m.wf(s0 + 0x1304, -60.0);
    m.w32(s0 + 0x10c8, u32::MAX);
    m.w32(s0 + 0x10f0, 1);
    m.wf(s0 + 0x10d8, 1.0);
    m.w32(s0 + 0x10ec, u32::MAX);
    m.w32(s0 + 0x10e4, u32::MAX);
    m.w32(s0 + 0x1318, u32::MAX);
    m.call(0x001f_7918, &[s0, 0x03ff_ffff, 0], &[]);
    m.call(0x001f_7918, &[s0, 0x0010_ff00, 8], &[]);
}

/// FUN_0015c1e8(o, src): FUN_00231170(o, src); collects FUN_002317e0(o, src, &set) into a FUN_001ae2a0
/// (count * 4) array at +0xe4: every member gets +0x28 bit 57; FUN_00132b10(x, 0x1e) members are kept
/// (+0xe0 count), the others get slot 8 (FUN_001ae478 / FUN_001ae4a0 frame)
pub fn ee_sub_15c1e8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let s2 = a0;
    m.call(0x0023_1170, &[a0, a1], &[]);
    let sp = stack_push(m, 0x80);
    m.call(0x001a_e478, &[sp], &[]);
    let set = sp + 0x10;
    m.call(0x0013_f888, &[set], &[]);
    m.call(0x0023_17e0, &[s2, a1, set], &[]);
    let n = m.r32(sp + 0x18);
    m.w32(sp + 0x20, n.wrapping_mul(4));
    m.w32(sp + 0x24, 0x10);
    let arr = m.call(0x001a_e2a0, &[sp + 0x20, 0x10], &[]).0;
    m.w32(s2 + 0xe4, arr);
    let mut i = 0u32;
    while (i as i32) < m.r32(sp + 0x18) as i32 {
        let o = { let p_ = m.r32(sp + 0x1c); m.r32(p_ + 4 * i) };
        let hi = m.r32(o + 0x2c);
        m.w32(o + 0x2c, hi | 0x0200_0000);
        if m.call(0x0013_2b10, &[o, 0x1e], &[]).0 != 0 {
            let k = m.r32(s2 + 0xe0);
            let a = m.r32(s2 + 0xe4);
            m.w32(a + 4 * k, o);
            m.w32(s2 + 0xe0, k + 1);
        } else {
            vslot(m, o, 8, &[], &[]);
        }
        i += 1;
    }
    m.call(0x001a_e4a0, &[sp, 2], &[]);
    stack_pop(m, 0x80);
}

/// FUN_0019cbf8(o, world): o = world, +0x20 = +4 = -1, FUN_0019d4d0(o + 0x24, o)
pub fn ee_sub_19cbf8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    m.w32(a0, a1);
    m.w32(a0 + 0x20, u32::MAX);
    m.w32(a0 + 4, u32::MAX);
    m.call(0x0019_d4d0, &[a0 + 0x24, a0], &[]);
}

/// FUN_00220fd0(o, src): FUN_00231768; +0xe4 / +0xfc lists (FUN_00117fd0 / FUN_00118220); +0x114 =
/// FUN_001ae318(0x240), FUN_00221818(o, 0x30), FUN_0013f810(o, src), FUN_00221940(o); +0x114 / +0x118 =
/// FUN_001ae238(+0x110 * 0x90, +0x114), +0x11c = 1; a single FUN_002317e0 match becomes +0x10c (bit 57);
/// no +0x10c clears +0x164 bit 1
pub fn ee_sub_220fd0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let s3 = a0;
    m.call(0x0023_1768, &[a0, a1], &[]);
    m.call(0x0011_7fd0, &[a1, s3 + 0xe4], &[]);
    m.call(0x0011_8220, &[a1, s3 + 0xe4, s3 + 0xfc], &[]);
    let sp = stack_push(m, 0x80);
    m.call(0x001a_e478, &[sp], &[]);
    let s0 = sp + 0x10;
    m.w32(s0, 0x240);
    m.w32(s0 + 4, 0x10);
    let v = m.call(0x001a_e318, &[s0], &[]).0;
    m.w32(s3 + 0x114, v);
    m.call(0x0022_1818, &[s3, 0x30], &[]);
    m.call(0x0013_f810, &[s3, a1], &[]);
    m.call(0x0022_1940, &[s3], &[]);
    let n = m.r32(s3 + 0x110).wrapping_mul(0x90);
    m.w32(s0, n);
    m.w32(s0 + 4, 0x10);
    let old = m.r32(s3 + 0x114);
    let v = m.call(0x001a_e238, &[s0, old], &[]).0;
    m.w32(s3 + 0x11c, 1);
    m.w32(s3 + 0x118, v);
    m.w32(s3 + 0x114, v);
    m.call(0x0013_f888, &[s0], &[]);
    m.call(0x0023_17e0, &[s3, a1, s0], &[]);
    if m.r32(sp + 0x18) == 1 {
        let o = { let p_ = m.r32(sp + 0x1c); m.r32(p_) };
        m.w32(s3 + 0x10c, o);
        let hi = m.r32(o + 0x2c);
        m.w32(o + 0x2c, hi | 0x0200_0000);
    }
    if m.r32(s3 + 0x10c) == 0 {
        let f = m.r32(s3 + 0x164);
        m.w32(s3 + 0x164, f & !2);
    }
    m.call(0x001a_e4a0, &[sp, 2], &[]);
    stack_pop(m, 0x80);
}

/// FUN_00209618(src, cam): kind [src] 0 / 1 (target [src+8] / [src+8]+0x30) -> dir = FUN_00204e38(cam, t,
/// t+0x1c), up = FUN_00204ed8(cam, t, t+0x1c); kind 2 -> FUN_0020c0e8([src+8]) and FUN_00204e38 on +0x38
/// unless its state is 1, 2 or 4; kind 3 -> dir = [src+8]+0x10; dir scaled by the +0xfc4 zoom word;
/// cam+0x1e0 = up (default DAT_002a2dd0), cam+0x190 = -dir
pub fn ee_sub_209618<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let s1 = a1;
    let mut s2 = m.rv(0x002a_2dd0);
    let mut dir = s2;
    let k = m.r32(a0) as i32;
    match k {
        0 | 1 => {
            let s0 = if k == 0 { m.r32(a0 + 8) } else { let p_ = m.r32(a0 + 8); m.r32(p_ + 0x30) };
            if s0 != 0 {
                let f = m.rf(s0 + 0x1c);
                dir = m.call(0x0020_4e38, &[s1, s0], &[fv(f)]).1;
                let f = m.rf(s0 + 0x1c);
                s2 = m.call(0x0020_4ed8, &[s1, s0], &[fv(f)]).1;
            }
        }
        2 => {
            let t = m.r32(a0 + 8);
            m.call(0x0020_c0e8, &[t], &[]);
            let t = m.r32(a0 + 8);
            let st = m.r32(t + 0x10) as i32;
            if !(st == 1 || st == 2 || st == 4) {
                let x = m.r32(t + 0x38);
                let f = m.rf(x + 0x1c);
                dir = m.call(0x0020_4e38, &[s1, x], &[fv(f)]).1;
            }
        }
        3 => {
            let t = m.r32(a0 + 8);
            dir = m.rv(t + 0x10);
        }
        _ => {}
    }
    let w = m.r32(s1 + 0xfc4);
    if w != 0 {
        let s = { let p_ = m.r32(w + 8); m.rf(p_) };
        dir = dir.map(|x| x * s);
    }
    m.wv(s1 + 0x1e0, s2);
    m.wv(s1 + 0x190, dir.map(|x| x * -1.0));
}

/// FUN_00206518(cam): FUN_001cb518; +0x344 / +0xe88 / +0x33c copies; FUN_001eecd0(+0x26c),
/// FUN_00209778(+0xfd0), FUN_00209870(cam, 0), +0x498 / +0x49c resets; +0x300 = FUN_001a62e8(cam), +0x310 =
/// FUN_00204740(cam) (into the +0x2f0 view), FUN_00204660; +0xfe8 bits from the FUN_00207a20(9 / 8 / 0xf /
/// 0x11) options (no 9 -> +0xe78 = 2); four FUN_00153ef8 registrations; +0x494 -> FUN_001f1a40 /
/// FUN_001f1cf8; slot 8(+0xfec); slot 7(FUN_00205438(cam, 0x9c)); FUN_001a58d0(+0xf64, cam, 0xf), +0xf64
/// +0xe0 / +0xe4
pub fn ee_sub_206518<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let s1 = a0;
    m.call(0x001c_b518, &[s1], &[]);
    let v = m.r32(s1 + 0x340);
    m.w32(s1 + 0x344, v);
    let v = m.r32(s1 + 0xe84);
    m.w32(s1 + 0xe88, v);
    let v = m.r32(s1 + 0x338);
    m.w32(s1 + 0x33c, v);
    m.call(0x001e_ecd0, &[s1 + 0x26c], &[]);
    m.call(0x0020_9778, &[s1 + 0xfd0], &[]);
    m.call(0x0020_9870, &[s1, 0], &[]);
    let x = m.r32(s1 + 0x498);
    m.call(0x0020_ce40, &[x], &[]);
    let x = m.r32(s1 + 0x49c);
    m.call(0x0020_d1c8, &[x], &[]);
    let x = m.r32(s1 + 0x498);
    m.call(0x0020_d1a0, &[x], &[]);
    let p = m.call(0x001a_62e8, &[s1], &[]).1;
    m.wv(s1 + 0x300, p);
    let f = m.call(0x0020_4740, &[s1], &[]).1[0];
    m.wf(s1 + 0x310, f);
    let v = m.r32(s1 + 0x2f0);
    m.w32(v + 8, s1 + 0x300);
    let v = m.r32(s1 + 0x2f0);
    let f = m.rf(s1 + 0x310);
    m.wf(v + 0xc, f);
    let f = m.rf(s1 + 0x310);
    m.call(0x0020_4660, &[s1], &[fv(f)]);
    m.w32(s1 + 0xfe8, 0);
    for (opt, bit) in [(9u32, 2u32), (8, 4), (0xf, 1), (0x11, 8)] {
        let x = m.call(0x0020_7a20, &[opt], &[]).0;
        if m.call(0x0020_9ad0, &[s1, x], &[]).0 != 0 {
            let f = m.r32(s1 + 0xfe8);
            m.w32(s1 + 0xfe8, f | bit);
        } else if opt == 9 {
            m.w32(s1 + 0xe78, 2);
        }
    }
    for g in [0x002e_1e40u32, 0x002f_7900, 0x002d_d5bc, 0x002d_d648] {
        let a = m.r32(s1 + 0xf64);
        let c = m.r32(g);
        m.call(0x0015_3ef8, &[a, s1, c], &[]);
    }
    if m.r32(s1 + 0x494) != 0 {
        m.call(0x001f_1a40, &[s1], &[]);
        m.call(0x001f_1cf8, &[s1], &[]);
    }
    let o = m.r32(s1 + 0xfec);
    if o != 0 {
        vslot(m, o, 8, &[], &[]);
    }
    let r = m.call(0x0020_5438, &[s1, 0x9c], &[]).0;
    vslot(m, r, 7, &[], &[]);
    let a = m.r32(s1 + 0xf64);
    m.call(0x001a_58d0, &[a, s1, 0xf], &[]);
    let a = m.r32(s1 + 0xf64);
    m.w32(a + 0xe0, s1);
    let v = m.r32(s1 + 0xec);
    m.w32(a + 0xe4, v);
}

/// FUN_00133a38(world, a, b, mode): link record of a -> b (a+0x64 list keyed by b+0x60); mode 1 / 0 / 2 =
/// on / keep (+0x10 of the record) / off. On: existing record pair gets +0x10 = on, a missing one (after
/// FUN_00133d08 allows) is created as a FUN_00133c68 pair; off with a record: slot 53 both ways, the
/// FUN_00133cb8 pair is unlinked (FUN_001a7e10, slot 61 -> FUN_001f5078) and freed to +0x3cf8 (FUN_0011cd98)
pub fn ee_sub_133a38<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32) {
    let (s4, s3, s2) = (a0, a1, a2);
    let key = m.r32(s2 + 0x60);
    let mut s0 = m.r32(s3 + 0x64);
    while s0 != 0 && m.r32(s0 + 0xc) != key {
        s0 = m.r32(s0 + 4);
    }
    let mut s1 = s0;
    // modes outside 0..2 leave s5 uninitialised in the original; 0 here
    let s5 = match a3 {
        1 => 1,
        0 if s0 != 0 => m.r32(s0 + 0x10),
        _ => 0,
    };
    if s5 == 0 && m.call(0x0013_3d08, &[s4, s3, s2], &[]).0 == 0 {
        if s1 == 0 {
            return;
        }
        let (x, y) = (m.r32(s3 + 0x60), m.r32(s2 + 0x60));
        vslot(m, x, 53, &[y], &[]);
        let (x, y) = (m.r32(s2 + 0x60), m.r32(s3 + 0x60));
        vslot(m, x, 53, &[y], &[]);
        let r1 = m.call(0x0013_3cb8, &[s4, s3, s2], &[]).0;
        let r2 = m.call(0x0013_3cb8, &[s4, s2, s3], &[]).0;
        let w = m.r32(r1);
        if w != 0 {
            m.call(0x001a_7e10, &[s4, w], &[]);
            for r in [r1, r2] {
                let o = m.r32(r + 0xc);
                if vslot(m, o, 61, &[], &[]).0 == 0 {
                    let o = m.r32(r + 0xc);
                    m.call(0x001f_5078, &[o], &[]);
                }
            }
        }
        m.call(0x0011_cd98, &[s4 + 0x3cf8, r1], &[]);
        m.call(0x0011_cd98, &[s4 + 0x3cf8, r2], &[]);
        return;
    }
    if s1 == 0 {
        let p = m.call(0x0013_3c68, &[s4, s3, s2], &[]).0;
        let q = m.call(0x0013_3c68, &[s4, s2, s3], &[]).0;
        m.w32(p + 8, q);
        m.w32(q + 8, p);
        s1 = p;
    }
    let o = m.r32(s1 + 8);
    m.w32(o + 0x10, s5);
    m.w32(s1 + 0x10, s5);
}

/// FUN_00231768(o, stream): +0x50 = slot 48(o, FUN_00117d50(stream)); FUN_00117f18(stream, o + 0x60);
/// FUN_002313f0(o); FUN_001a6d28(o, FUN_00117c80(stream))
pub fn ee_sub_231768<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let v = m.call(0x0011_7d50, &[a1], &[]).0;
    let r = vslot(m, a0, 48, &[v], &[]).0;
    m.w32(a0 + 0x50, r);
    m.call(0x0011_7f18, &[a1, a0 + 0x60], &[]);
    m.call(0x0023_13f0, &[a0], &[]);
    let v = m.call(0x0011_7c80, &[a1], &[]).0;
    m.call(0x001a_6d28, &[a0, v], &[]);
}

/// FUN_002132d0(o): +0x7c == -1 -> 0, else FUN_002137f0(o) == 0
pub fn ee_sub_2132d0<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    if m.r32(a0 + 0x7c) == u32::MAX {
        return 0;
    }
    (m.call(0x0021_37f0, &[a0], &[]).0 == 0) as u32
}

/// FUN_00214398(o, stream, a2, &n, &buf): n = FUN_00117c20(stream); buf = FUN_00116918(+0xc4, (n + 1) << 6,
/// DAT_002c40a4 ? 0x13 : 0x11); n times FUN_002149d8(o, stream, a2, n, buf)
pub fn ee_sub_214398<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, t0: u32) {
    let n = m.call(0x0011_7c20, &[a1], &[]).0;
    m.w32(a3, n);
    let sel = if m.r32(0x002c_40a4) == 0 { 0x11 } else { 0x13 };
    let h = m.r32(a0 + 0xc4);
    let v = m.call(0x0011_6918, &[h, n.wrapping_add(1) << 6, sel], &[]).0;
    m.w32(t0, v);
    let mut i = 1i32;
    while m.r32(a3) as i32 >= i {
        let (c, b) = (m.r32(a3), m.r32(t0));
        m.call(0x0021_49d8, &[a0, a1, a2, c, b], &[]);
        i += 1;
    }
}

fn mark_word<M: QMem + ?Sized>(m: &mut M, a: u32) -> bool {
    let w = m.r32(a + 4);
    let mk = m.r32(0x002d_d598);
    if w >> 31 == mk {
        return false;
    }
    m.w32(a + 4, (w & 0x7fff_ffff) | mk << 31);
    true
}

fn next_handle<M: QMem + ?Sized>(m: &mut M, a: u32, tab: u32, sz: u32) -> u32 {
    let w = m.r32(a + 4);
    let (k, i) = ((w >> 29) & 3, (w >> 16) & 0x1fff);
    if i == 0 && k == 0 {
        return 0;
    }
    m.r32(tab + 4 * k).wrapping_add(i.wrapping_mul(sz))
}

/// FUN_001286e0(gc): flips the gc phase word, FUN_002574e0..257508 pre-hooks; three FUN_001ae318 root
/// arrays (+8 / +0x10 / +0x18 sized by 0x452108 / 0x4520b8 / 0x452130 counts); marks (bit 31 = DAT_002dd598)
/// the 0x2dd2ac chain (unmarked -> +8 array), the 0x2dd2cc chain and its +8 children (FUN_001285d8(gc,
/// child + 0xc)), the 0x3e0e80 list's +8 objects (unmarked -> +0x10 array); FUN_001281b0(gc), post-hooks
/// FUN_00257510 / 257678 / 2577e0 / 257948 / 257ab0 / 257c18
pub fn ee_sub_1286e0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let s3 = a0;
    let v = (m.r32(s3) == 0) as u32;
    m.w32(s3, v);
    for f in [0x0025_74e0u32, 0x0025_74e8, 0x0025_74f0, 0x0025_74f8, 0x0025_7500, 0x0025_7508] {
        m.call(f, &[], &[]);
    }
    let sp = stack_push(m, 0xc0);
    m.call(0x001a_e478, &[sp], &[]);
    let s0 = sp + 0x10;
    for (cnt, arr, n) in [(0x0045_2108u32, 8u32, 4u32), (0x0045_20b8, 0x10, 0xc), (0x0045_2130, 0x18, 0x14)] {
        let c = m.r32(cnt);
        m.w32(s0, c << 2);
        m.w32(s0 + 4, 0x10);
        let v = m.call(0x001a_e318, &[s0], &[]).0;
        m.w32(s3 + arr, v);
        m.w32(s3 + n, 0);
    }
    let mut a1 = m.r32(0x002d_d2ac);
    while a1 != 0 {
        if mark_word(m, a1) {
            let (c, a) = (m.r32(s3 + 4), m.r32(s3 + 8));
            m.w32(a + 4 * c, a1);
            m.w32(s3 + 4, c + 1);
        }
        a1 = next_handle(m, a1, 0x0045_20e8, 0x28);
    }
    let mut s2 = m.r32(0x002d_d2cc);
    while s2 != 0 {
        mark_word(m, s2);
        let mut c = m.r32(s2 + 8);
        while c != 0 {
            let w = m.r32(c + 4);
            let mk = m.r32(0x002d_d598);
            m.w32(c + 4, (w & 0x7fff_ffff) | mk << 31);
            m.call(0x0012_85d8, &[s3, c + 0xc], &[]);
            c = m.r32(c + 0x14);
        }
        s2 = next_handle(m, s2, 0x0045_2138, 0xc);
    }
    let mut a2 = m.r32(0x003e_0e80);
    while a2 != 0 {
        let o = m.r32(a2 + 8);
        if mark_word(m, o) {
            let (c, a) = (m.r32(s3 + 0xc), m.r32(s3 + 0x10));
            m.w32(a + 4 * c, o);
            m.w32(s3 + 0xc, c + 1);
        }
        a2 = m.r32(a2 + 0x10);
    }
    m.call(0x0012_81b0, &[s3], &[]);
    for f in [0x0025_7510u32, 0x0025_7678, 0x0025_77e0, 0x0025_7948, 0x0025_7ab0, 0x0025_7c18] {
        m.call(f, &[], &[]);
    }
    m.call(0x001a_e4a0, &[sp, 2], &[]);
    stack_pop(m, 0xc0);
}
