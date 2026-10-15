//! WP-Q14 part B: classes C2cc240 (slots 5-9, 13 + FUN_00146e18) and C2d0120 (slots 5-9 + FUN_0017bff8 /
//! FUN_0017c550 / FUN_001c0a40). 1:1 from ghidra_functions(_v2).body on `QMem` (conventions as q7util / q9util).
//! Layout: +0x10 owner (the NPC), +0x18 sub-object, +0x30 state (-1 = none), +0x34 time of the last state change,
//! +0x38 target object; owner +0x294 path, +0x3cc config, +0xfe8 flags.
#![allow(unused_variables, unused_mut, dead_code)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem, V4};
use crate::engine::classes::q::q5util::{e8_dc_10, f4, pos};
use crate::engine::classes::q::q6::{rof, vcall, CLOCK};

fn clock<M: QMem + ?Sized>(m: &mut M) -> f32 {
    m.rf(CLOCK)
}

// ---------------------------------------------------------------- C2cc240
pub fn c2cc240_v5() -> u32 {
    0x002b_ba4c
}
/// FUN_001466c0 (slot 6)
pub fn c2cc240_v6<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef68, &[o], &[]);
    m.w32(o + 0x30, 0xffff_ffff);
}
/// FUN_001466f0 (slot 7)
pub fn c2cc240_v7<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef70, &[o], &[]);
    ee_sub_146e18(m, o, 0);
    let p = m.r32(o + 0x10);
    let t = m.r32(o + 0x38);
    vcall(m, p, 0xc, 0x2fc, &[4, t, 4, t], &[]);
    let q = m.r32(o + 0x10);
    let v = m.r32(q + 0x10e0);
    m.w32(q + 0x10e0, v | 1);
}
/// FUN_00146758 (slot 8)
pub fn c2cc240_v8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef78, &[o], &[]);
    let p = m.r32(o + 0x10);
    let v = m.r32(p + 0x10e0);
    m.w32(p + 0x10e0, v & 0xffff_fffe);
}
/// FUN_00146e18: state change (old states >= 2 and < 5 call FUN_001ef038; new states 2..4 call FUN_001eefc0)
pub fn ee_sub_146e18<M: QMem + ?Sized>(m: &mut M, o: u32, new: i32) {
    let cur = m.r32(o + 0x30) as i32;
    if new == cur {
        return;
    }
    if cur < 5 {
        if cur < 2 {
            m.w32(o + 0x30, new as u32);
        } else {
            m.call(0x001e_f038, &[o], &[]);
            m.w32(o + 0x30, new as u32);
        }
    } else {
        m.w32(o + 0x30, new as u32);
    }
    let t = m.r32(CLOCK);
    m.w32(o + 0x34, t);
    if new < 5 && 1 < new {
        m.call(0x001e_efc0, &[o], &[]);
    }
}
/// FUN_001467a0 (slot 9): approach / attack state machine toward the target (+0x38)
pub fn c2cc240_v9<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef98, &[o], &[]);
    if m.r32(o + 0x30) as i32 == -1 {
        return;
    }
    let p = m.r32(o + 0x10);
    let tgt = m.r32(o + 0x38);
    let ok = m.r32(p + 0xdf4) == 0xffff_ffff && m.call(0x001a_61a0, &[tgt], &[]).0 != 0 && m.r32(tgt + 0x330) == 0;
    if !ok {
        ee_sub_146e18(m, o, -1);
        return;
    }
    let a = pos(m, tgt);
    let b = pos(m, p);
    let mut d = [a[0] - b[0], a[1] - b[1], 0.0, a[3] - b[3]];
    // vmul; vadd x+y+z (z = 0); vsqrt
    let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    let dir: V4 = if len < 0.0001 {
        [m.rf(0x002a_2da0), m.rf(0x002a_2da4), m.rf(0x002a_2da8), m.rf(0x002a_2dac)]
    } else {
        let q = 1.0 / len;
        d.map(|x| x * q)
    };
    let mut st = m.r32(o + 0x30) as i32;
    loop {
        let before = st;
        match st {
            0 => {
                if len < rof(0x002b_d360) {
                    let fwd = m.call(0x001a_6318, &[p], &[]).0;
                    let fv = m.rv(fwd);
                    let dot = fv[0] * dir[0] + fv[1] * dir[1] + fv[2] * dir[2];
                    if 0.8f32 < dot {
                        let r = m.call(0x0011_d448, &[], &[f4(0.0), f4(1.0)]).1[0];
                        if r < 0.3f32 {
                            ee_sub_146e18(m, o, 3);
                        } else {
                            ee_sub_146e18(m, o, 2);
                        }
                    }
                } else if len > rof(0x002b_d36c) {
                    ee_sub_146e18(m, o, 1);
                }
            }
            1 => {
                if len < rof(0x002b_d368) {
                    ee_sub_146e18(m, o, 0);
                }
            }
            2 => {
                if m.r32(p + 0x18d8) == 0xffff_ffff && clock(m) - m.rf(o + 0x34) > 0.1f32 {
                    ee_sub_146e18(m, o, -1);
                }
            }
            3 => {
                if clock(m) - m.rf(o + 0x34) > 0.1f32 {
                    let k = m.r32(p + 0x2dc) as i32;
                    if k == -1 {
                        ee_sub_146e18(m, o, -1);
                    } else if k == 1 && !(clock(m) - m.rf(p + 0x2e0) <= rof(0x002b_d370)) {
                        ee_sub_146e18(m, o, 4);
                    }
                }
            }
            4 => {
                if 0.1f32 < clock(m) - m.rf(o + 0x34) && m.r32(p + 0x18c8) != 0xd {
                    ee_sub_146e18(m, o, -1);
                }
            }
            _ => {}
        }
        st = m.r32(o + 0x30) as i32;
        if st == before {
            return;
        }
    }
}
/// FUN_00146b18 (slot 13): steering output (+0x18 heading, +0x1c speed, +0x30 flags) of the output record `r`
pub fn c2cc240_v13<M: QMem + ?Sized>(m: &mut M, o: u32, r: u32) {
    m.call(0x001e_f0c0, &[o, r], &[]);
    let st = m.r32(o + 0x30) as i32;
    let aim = |m: &mut M| -> V4 {
        let t = { let t_ = m.r32(o + 0x38); pos(m, t_) };
        let p = { let t_ = m.r32(o + 0x10); pos(m, t_) };
        [t[0] - p[0], t[1] - p[1], t[2] - p[2], t[3] - p[3]]
    };
    match st {
        1 | 2 | 4 => {
            let d = aim(m);
            m.wf(r + 0x1c, 1.0);
            let h = m.call(0x0024_58f0, &[], &[f4(d[1]), f4(d[0])]).1[0];
            m.wf(r + 0x18, h);
            if st == 1 {
                return;
            }
            if 0.1f32 <= clock(m) - m.rf(o + 0x34) {
                return;
            }
            if st == 2 {
                let v = m.r32(r + 0x30) | 0x80;
                m.w32(r + 0x30, v);
            } else {
                let v = m.r32(r + 0x30) | 0x80;
                m.w32(r + 0x30, v);
            }
            m.call(0x001c_45a8, &[r], &[]);
        }
        3 => {
            let d = aim(m);
            let len = (d[0] * d[0] + d[1] * d[1]).sqrt();
            let v = len * (len * 0.0 + f32::from_bits(0x3acc_cccc)) + -0.25;
            let (mut lo, hi) = (m.rf(0x002a_4e58), m.rf(0x002a_4e5c));
            if lo <= v {
                lo = v;
                if hi < v {
                    lo = hi;
                }
            }
            m.wf(r + 0x1c, lo);
            let h = m.call(0x0024_58f0, &[], &[f4(d[1]), f4(d[0])]).1[0];
            m.wf(r + 0x18, h);
            if 0.1f32 <= clock(m) - m.rf(o + 0x34) {
                return;
            }
            let p = m.r32(o + 0x10);
            if m.r32(p + 0x2dc) != 0xffff_ffff {
                return;
            }
            let v = m.r32(r + 0x30) | 0x40;
            m.w32(r + 0x30, v);
            m.call(0x001c_45a8, &[r], &[]);
        }
        _ => {}
    }
}

// ---------------------------------------------------------------- C2d0120
pub fn c2d0120_v5() -> u32 {
    0x002b_b8e4
}
/// FUN_0017bed0 (slot 6)
pub fn c2d0120_v6<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef68, &[o], &[]);
    let p = m.r32(o + 0x10);
    m.call(0x0020_9890, &[p, 0x19], &[]);
    m.w32(o + 0x30, 0xffff_ffff);
}
/// FUN_0017bf08 (slot 7)
pub fn c2d0120_v7<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef70, &[o], &[]);
    let p = m.r32(o + 0x10);
    if m.r32(p + 0xfe8) & 4 == 0 {
        ee_sub_17c550(m, o, 0);
    } else {
        let s = e8_dc_10(m, p);
        let a = pos(m, p);
        let b = { let t_ = m.r32(o + 0x10); pos(m, t_) };
        let path = { let a_ = m.r32(o + 0x10); m.r32(a_ + 0x294) };
        m.call(0x001c_02d0, &[s, path], &[a, b]);
        ee_sub_17c550(m, o, 1);
    }
}
/// FUN_0017bfb8 (slot 8)
pub fn c2d0120_v8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef78, &[o], &[]);
    ee_sub_17c550(m, o, -1);
    let p = m.r32(o + 0x10);
    m.call(0x0020_9970, &[p, 0x19], &[]);
}
/// FUN_0017c550: sub-state change; 0 = brain action 0x9e, 1 = brain action 0xa7 (path follow, flag 8)
pub fn ee_sub_17c550<M: QMem + ?Sized>(m: &mut M, o: u32, st: i32) {
    if st == m.r32(o + 0x30) as i32 {
        return;
    }
    m.call(0x001e_f0d8, &[o + 0x18], &[]);
    m.w32(o + 0x30, st as u32);
    let p = m.r32(o + 0x10);
    if st == 0 {
        let b = m.call(0x0020_5438, &[p, 0x9e], &[]).0;
        let c = { let a_ = m.r32(p + 0x3cc); m.r32(a_ + 0xc) };
        m.w32(b + 0x38, c);
        vcall(m, b, 0xc, 0x44, &[], &[]);
    } else if st == 1 {
        let u = m.call(0x0020_5438, &[p, 0xa7], &[]).0;
        let path = { let a_ = m.r32(o + 0x10); m.r32(a_ + 0x294) };
        m.call(0x0017_8d90, &[u, path, 0], &[]);
        m.w32(u + 0xc0, 8);
        let c = { let a_ = m.r32(o + 0x10); let b_ = m.r32(a_ + 0x3cc); m.r32(b_ + 0xc) };
        m.w32(u + 200, c);
        vcall(m, u, 0xc, 0x44, &[], &[]);
    }
}
/// FUN_001c0a40(world, a, b, flags, out): FUN_001c0960 then every clique FUN_001c1700 finds (FUN_001bc688 +
/// FUN_0014c9f8 accumulate); 0 as soon as one step fails
pub fn ee_sub_1c0a40<M: QMem + ?Sized>(m: &mut M, w: u32, a: V4, b: V4, flags: u32, out: u32) -> u32 {
    let buf = stack_push(m, 0x200 + 0xc0);
    let rec = buf + 0x200;
    let mut r = m.call(0x001c_0960, &[w, flags, out, 2], &[a, b]).0;
    if r != 0 {
        let n = m.call(0x001c_1700, &[w, 0, 0x80, buf], &[a, b]).0 as i32;
        let mut i = 0;
        while i < n {
            let c = m.r32(buf + 4 * i as u32);
            let i3 = m.call(0x001b_c688, &[c, 2, rec], &[a, b]).0;
            r = m.call(0x0014_c9f8, &[r, out, i3, rec, flags, out, 1], &[]).0;
            i += 1;
            if r == 0 {
                stack_pop(m, 0x200 + 0xc0);
                return 0;
            }
        }
    }
    stack_pop(m, 0x200 + 0xc0);
    r
}
/// FUN_0017bff8(angle, o, surf, origin) -> V4: direction probe.
/// First probe heads toward the path end; 3-pass fan sweep selects the closest navigable direction.
pub fn ee_sub_17bff8<M: QMem + ?Sized>(m: &mut M, angle: f32, o: u32, surf: u32, origin: V4) -> V4 {
    let sp = stack_push(m, 0x460);
    let (lo_hi_a, lo_hi_b, outq, tmp) = (sp + 0x10, sp + 0x20, sp, sp + 0x40);
    let mut p = m.r32(o + 0x10);
    let mut ra = [m.rf(p + 0x31c), m.rf(p + 0x320)];
    let mut rb = [m.rf(p + 0x324), m.rf(p + 0x328)];
    if m.r32(p + 0xfbc) == 8 || m.r32(p + 0xfbc) == 0xd {
        let k = m.r32(p + 0xfc4);
        if k != 0 {
            let s = m.rf(k + 0x20);
            ra = [ra[0] * s, ra[1] * s];
            rb = [rb[0] * s, rb[1] * s];
        }
    }
    m.wf(lo_hi_a, ra[0]);
    m.wf(lo_hi_a + 4, ra[1]);
    m.wf(lo_hi_b, rb[0]);
    m.wf(lo_hi_b + 4, rb[1]);
    p = m.r32(o + 0x10);
    let cfg = m.r32(p + 0x3cc);
    let pp = pos(m, p);
    let d = [pp[0] - origin[0], pp[1] - origin[1], pp[2] - origin[2], pp[3] - origin[3]];
    let (o1, o2) = (sp + 0x100, sp + 0x104);
    m.call(0x0011_ef08, &[o1, o2, 0], &[d]);
    let ang = m.rf(o1);
    let mag = m.rf(o2);
    let w = m.call(0x0011_cf60, &[], &[f4(ang - angle)]).1[0];
    let r14 = m.rf(cfg + 0x14);
    let step = m.call(0x0011_cfe0, &[], &[f4(w), f4(r14)]).1[0];
    let cl = m.call(0x0011_d948, &[lo_hi_a], &[f4(mag)]).1[0];
    let dv = m.call(0x0011_e1a0, &[], &[f4(angle + step), f4(cl), f4(0.0)]).1;
    let q = [origin[0] + dv[0], origin[1] + dv[1], origin[2] + dv[2], origin[3] + dv[3]];
    let hit = m.call(0x001c_0b50, &[surf, outq], &[q]).0;
    let r = m.rv(outq);
    if hit != 0 {
        let d2 = [r[0] - origin[0], r[1] - origin[1], r[2] - origin[2], r[3] - origin[3]];
        m.call(0x0011_ef08, &[sp + 0x108, sp + 0x10c, 0], &[d2]);
        let a2 = m.rf(sp + 0x108);
        let w2 = m.call(0x0011_cf60, &[], &[f4(a2 - angle)]).1[0];
        let (r18, mg2) = (m.rf(cfg + 0x18), m.rf(sp + 0x10c));
        if w2.abs() < r18 && m.call(0x0011_d910, &[lo_hi_b], &[f4(mg2)]).0 != 0 {
            stack_pop(m, 0x460);
            return r;
        }
    }
    // fan sweep: 3 passes (0 = +0x14/lo_hi_a/10 steps, 1 = +0x18/lo_hi_b/10 steps, 2 = pi/zero/15 steps)
    let big = m.rf(0x002a_5600); // DAT_002a5600 = sentinel "infinity"
    for pass in 0..3i32 {
        let (steps, half_range, mag_lo_hi): (i32, f32, u32) = match pass {
            0 => (10, m.rf(cfg + 0x14), lo_hi_a),
            1 => (10, m.rf(cfg + 0x18), lo_hi_b),
            _ => {
                // pass 2: lo_hi_b with lo component zeroed, range = pi
                m.wf(lo_hi_b, 0.0);
                (15, core::f32::consts::PI, lo_hi_b)
            }
        };
        let cfg10 = m.r32(cfg + 0x10);
        let eff_range = if cfg10 == -1i32 as u32 { core::f32::consts::PI } else { half_range };
        let mut best_dist = big;
        let mut best_pt: V4 = [0.0; 4];
        if steps > 0 {
            let half = (steps - 1) as f32 * 0.5;
            for i in 0..steps {
                let t = (i as f32 / half - 1.0) * eff_range;
                let cl2 = m.call(0x0011_d948, &[mag_lo_hi], &[f4(1.0)]).1[0];
                let dv2 = m.call(0x0011_e1a0, &[], &[f4(angle + t), f4(cl2), f4(0.0)]).1;
                let q2 = [origin[0] + dv2[0], origin[1] + dv2[1], origin[2] + dv2[2], origin[3] + dv2[3]];
                let nhit = m.call(0x001c_0a40, &[surf, outq], &[q2]).0;
                for _h in 0..nhit {
                    let hv = m.rv(outq);
                    let (ho1, ho2) = (sp + 0x100, sp + 0x104);
                    m.call(0x0011_ee08, &[ho1, ho2, 0], &[hv]);
                    let hdist = m.rf(ho1);
                    if hdist < best_dist {
                        best_pt = hv;
                        best_dist = hdist;
                    }
                }
            }
        }
        if best_dist != big {
            stack_pop(m, 0x460);
            return best_pt;
        }
    }
    // fallback: snap to surface at origin
    let res = m.call(0x001c_0b50, &[surf, outq], &[origin]).0;
    let ret = if res == 0 { origin } else { m.rv(outq) };
    stack_pop(m, 0x460);
    ret
}
/// FUN_0017c3e8 (slot 9)
pub fn c2d0120_v9<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef98, &[o], &[]);
    if m.r32(o + 0x30) == 0 {
        return;
    }
    let p = m.r32(o + 0x10);
    let path = m.r32(p + 0x294);
    let surf = e8_dc_10(m, p);
    let cfg = m.r32(p + 0x3cc);
    let owner = m.r32(cfg + 0xc);
    let angle = m.call(0x0020_a690, &[cfg], &[]).1[0];
    let pw = pos(m, owner);
    let sub = m.r32(o + 0x18);
    if m.rf(sub + 0x70) <= rof(0x002b_e748) {
        let cnt = m.r32(path + 0xe8);
        let arr = m.r32(path + 0xf4);
        let last = m.rv(cnt.wrapping_mul(0x10).wrapping_add(arr).wrapping_sub(0x10));
        let d = [last[0] - pw[0], last[1] - pw[1], last[2] - pw[2], last[3] - pw[3]];
        let sp = stack_push(m, 0x10);
        m.call(0x0011_ef08, &[sp, sp | 4, 0], &[d]);
        let a = m.rf(sp);
        let mg = m.rf(sp + 4);
        stack_pop(m, 0x10);
        let w = m.call(0x0011_cf60, &[], &[f4(a - angle)]).1[0];
        let rp = m.r32(o + 0x10);
        if w.abs() <= m.rf(cfg + 0x18) && m.call(0x0011_d910, &[rp + 0x324], &[f4(mg)]).0 != 0 {
            return;
        }
    }
    let org = { let t_ = m.r32(o + 0x10); pos(m, t_) };
    let dirq = ee_sub_17bff8(m, angle, o, surf, org);
    m.call(0x001c_02d0, &[surf, path], &[org, dirq]);
    m.call(0x0017_8d90, &[sub, path, 0], &[]);
}
