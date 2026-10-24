//! WP-Q14 part C: classes C2d01a0 (slots 5-9 + FUN_0011ed40 / FUN_0017b8b0 / FUN_0017b9c0) and C2d05a0
//! (slots 5-9 + FUN_00175d00 / FUN_00176060 / FUN_0017b7f8 / FUN_0020a7e0 / FUN_0020b140). 1:1 from
//! ghidra_functions(_v2).body on `QMem`; callees outside the package by address through `QMem::call`.
//! Layout: +0x10 owner NPC, +0x18 sub-object, +0x30 state (C2d01a0) / target (C2d05a0), +0x34 state / time.
#![allow(unused_variables, unused_mut, dead_code)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem, V4};
use crate::engine::classes::q::q5util::{e8_dc_10, f4, pos};
use crate::engine::classes::q::q6::{rof, vcall, CLOCK, WORLD};

fn clock<M: QMem + ?Sized>(m: &mut M) -> f32 {
    m.rf(CLOCK)
}
fn sub3(a: V4, b: V4) -> V4 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2], a[3] - b[3]]
}
fn len3(a: V4) -> f32 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

/// FUN_0011ed40(p, a, b, out_pt, out_t, out_dist): closest point of the (unclamped) line a->b to p:
/// t = (p-a).d / d.d (d = b-a); pt = a + d*t; dist = |p - pt| (xyz)
pub fn ee_sub_11ed40<M: QMem + ?Sized>(m: &mut M, p: V4, a: V4, b: V4, out_pt: u32, out_t: u32, out_dist: u32) {
    let d = sub3(b, a);
    let pa = sub3(p, a);
    let num = pa[0] * d[0] + pa[1] * d[1] + pa[2] * d[2];
    let den = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
    let t = num / den;
    if out_t != 0 {
        m.wf(out_t, t);
    }
    if out_pt != 0 || out_dist != 0 {
        // VER3: ACC = a + vf0 (w + 1.0), then vmaddx: pt = ACC + d * t (asm 0x11edb4)
        let pt: V4 = std::array::from_fn(|i| (a[i] + [0.0, 0.0, 0.0, 1.0][i]) + d[i] * t);
        if out_pt != 0 {
            m.wv(out_pt, pt);
        }
        if out_dist != 0 {
            let r = sub3(p, pt);
            m.wf(out_dist, len3(r));
        }
    }
}

// ---------------------------------------------------------------- C2d01a0
pub fn c2d01a0_v5() -> u32 {
    0x002b_b8d0
}
/// FUN_0017b808 (slot 6)
pub fn c2d01a0_v6<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef68, &[o], &[]);
    let p = m.r32(o + 0x10);
    m.call(0x0020_9890, &[p, 0x20], &[]);
    m.w32(o + 0x30, 0xffff_ffff);
}
/// FUN_0017b840 (slot 7)
pub fn c2d01a0_v7<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef70, &[o], &[]);
    ee_sub_17b8b0(m, o, 0);
}
/// FUN_0017b870 (slot 8)
pub fn c2d01a0_v8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef78, &[o], &[]);
    ee_sub_17b8b0(m, o, -1);
    let p = m.r32(o + 0x10);
    m.call(0x0020_9970, &[p, 0x20], &[]);
}
/// FUN_0017b8b0: sub-state change (1 = brain 0xa5 with +0x40 target, 0 = brain 0x9e, 2 = brain 0x9e + flag +0x3c)
pub fn ee_sub_17b8b0<M: QMem + ?Sized>(m: &mut M, o: u32, st: i32) {
    if st == m.r32(o + 0x30) as i32 {
        return;
    }
    m.call(0x001e_f0d8, &[o + 0x18], &[]);
    m.w32(o + 0x30, st as u32);
    let t = m.r32(CLOCK);
    m.w32(o + 0x34, t);
    let p = m.r32(o + 0x10);
    if st == 1 {
        let b = m.call(0x0020_5438, &[p, 0xa5], &[]).0;
        let u = m.r32(o + 0x40);
        m.w32(b + 0x40, 1);
        m.w32(b + 0x30, u);
        vcall(m, b, 0xc, 0x44, &[], &[]);
    } else if st < 2 {
        if st == 0 {
            let b = m.call(0x0020_5438, &[p, 0x9e], &[]).0;
            vcall(m, b, 0xc, 0x44, &[], &[]);
            let p = m.r32(o + 0x10);
            m.call(0x0020_8540, &[p, 0], &[]);
        }
    } else if st == 2 {
        let b = m.call(0x0020_5438, &[p, 0x9e], &[]).0;
        m.w32(b + 0x3c, 1);
        vcall(m, b, 0xc, 0x44, &[], &[]);
    }
}
/// FUN_0017b9c0: periodic (rand 1..2 s) target pick: +0x38 = best candidate of the owner's list at +0xf98
/// (count +0xf94) by distance (+ hysteresis for the current one, + penalties for list +0xfac (count +0xfa8)
/// members near the line), +0x3c = nearest list-B member within 800 not near the line, +0x40 = +0x3c or +0x38
pub fn ee_sub_17b9c0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let now = clock(m);
    if m.rf(o + 0x44) > now {
        return;
    }
    let mut best_pos: V4 = [0.0; 4];
    let mut best = 0u32;
    let r = m.call(0x0011_d448, &[], &[f4(rof(0x002b_e710)), f4(rof(0x002b_e714))]).1[0];
    m.wf(o + 0x44, now + r);
    let p = m.r32(o + 0x10);
    let me = pos(m, p);
    let buf = stack_push(m, 0x20);
    let (tt, dd) = (buf, buf + 4);
    let n = m.r32(p + 0xf94) as i32;
    let mut cur_min = rof(0x002a_55fc);
    let mut i = 0i32;
    while i < n {
        let cand = m.r32(p.wrapping_add(i as u32 * 4).wrapping_add(0xf98));
        i += 1;
        let mut score = cur_min;
        if (m.r64(cand + 0x28) >> 0x3c) & 1 != 0 {
            let cpos = pos(m, cand);
            let dist = len3(sub3(me, cpos));
            score = dist;
            if cand == m.r32(o + 0x38) {
                score = rof(0x002b_e720) + dist * (rof(0x002b_e724) + dist * rof(0x002b_e728));
            }
            let nb = m.r32(p + 0xfa8) as i32;
            let mut j = 0i32;
            while j < nb {
                let other = m.r32(p.wrapping_add(j as u32 * 4).wrapping_add(0xfac));
                let opos = pos(m, other);
                ee_sub_11ed40(m, opos, me, cpos, 0, tt, dd);
                let (t, d) = (m.rf(tt), m.rf(dd));
                if 0.0 <= t && t <= 1.0 {
                    let v = rof(0x002b_e730) + d * (rof(0x002b_e734) + d * rof(0x002b_e738));
                    let mut c = rof(0x002b_e740);
                    if c <= v {
                        c = v;
                        if rof(0x002b_e744) < v {
                            c = rof(0x002b_e744);
                        }
                    }
                    score += c;
                }
                j += 1;
            }
            if score < cur_min {
                best_pos = cpos;
                best = cand;
            } else {
                score = cur_min;
            }
        }
        cur_min = score;
    }
    m.w32(o + 0x38, best);
    let nb = m.r32(p + 0xfa8) as i32;
    let mut pick = 0u32;
    let mut lim = 800.0f32;
    let mut j = 0i32;
    while j < nb {
        let other = m.r32(p.wrapping_add(j as u32 * 4).wrapping_add(0xfac));
        let opos = pos(m, other);
        let dist = len3(sub3(opos, me));
        let mut accept = false;
        if dist <= lim {
            if best == 0 {
                accept = true;
            } else {
                ee_sub_11ed40(m, opos, me, best_pos, 0, tt, dd);
                let (t, d) = (m.rf(tt), m.rf(dd));
                if 0.0 <= t && t <= 1.0 && d <= 500.0 {
                    accept = true;
                }
            }
        }
        if accept {
            pick = other;
            lim = dist;
        }
        j += 1;
    }
    m.w32(o + 0x3c, pick);
    let c = if pick == 0 { m.r32(o + 0x38) } else { pick };
    m.w32(o + 0x40, c);
    stack_pop(m, 0x20);
}
/// FUN_0017bd78 (slot 9)
pub fn c2d01a0_v9<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef98, &[o], &[]);
    let p = m.r32(o + 0x10);
    let busy = m.call(0x0020_9098, &[p], &[]).0 != 0;
    ee_sub_17b9c0(m, o);
    loop {
        if m.call(0x001e_f0a8, &[o], &[]).0 != 0 {
            return;
        }
        let st = m.r32(o + 0x30) as i32;
        match st {
            1 => {
                if !busy {
                    let tg = m.r32(o + 0x40);
                    let mut to0 = true;
                    if tg != 0 {
                        let sub = m.r32(o + 0x18);
                        if tg != m.r32(sub + 0x30) {
                            ee_sub_17b8b0(m, o, -1);
                            ee_sub_17b8b0(m, o, 1);
                            to0 = false;
                        } else if m.call(0x0017_5810, &[sub], &[]).0 == 0 {
                            to0 = false;
                        }
                    }
                    if to0 {
                        ee_sub_17b8b0(m, o, 0);
                    }
                } else {
                    ee_sub_17b8b0(m, o, 2);
                }
            }
            0 => {
                if busy {
                    ee_sub_17b8b0(m, o, 2);
                } else if m.r32(o + 0x40) != 0 {
                    ee_sub_17b8b0(m, o, 1);
                }
            }
            2 => {
                if !busy {
                    ee_sub_17b8b0(m, o, 0);
                }
            }
            _ => {}
        }
        if st == m.r32(o + 0x30) as i32 {
            return;
        }
    }
}

// ---------------------------------------------------------------- C2d05a0
pub fn c2d05a0_v5() -> u32 {
    0x002b_b7cc
}
fn anim<M: QMem + ?Sized>(m: &mut M, o: u32, st: u32) {
    m.call(0x0017_6458, &[o, st], &[]);
}
/// FUN_00175758 (slot 6)
pub fn c2d05a0_v6<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef68, &[o], &[]);
    let p = m.r32(o + 0x10);
    m.call(0x0020_9890, &[p, 0x1d], &[]);
    m.w32(o + 0x34, 0xffff_ffff);
}
/// FUN_00175790 (slot 7)
pub fn c2d05a0_v7<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef70, &[o], &[]);
    anim(m, o, 3);
    let p = m.r32(o + 0x10);
    let t = m.r32(o + 0x30);
    m.call(0x0020_8540, &[p, t], &[]);
}
/// FUN_001757d0 (slot 8)
pub fn c2d05a0_v8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef78, &[o], &[]);
    anim(m, o, 0xffff_ffff);
    let p = m.r32(o + 0x10);
    m.call(0x0020_9970, &[p, 0x1d], &[]);
}
/// FUN_0017b7f8
pub fn ee_sub_17b7f8<M: QMem + ?Sized>(m: &mut M, o: u32) -> bool {
    m.r32(o + 0x30) == 0xffff_ffff
}
/// FUN_0020a7e0
pub fn ee_sub_20a7e0<M: QMem + ?Sized>(m: &mut M, o: u32) -> bool {
    rof(0x002c_3db4) < clock(m) - m.rf(o + 4)
}
/// FUN_00176060
pub fn ee_sub_176060<M: QMem + ?Sized>(m: &mut M, o: u32) -> bool {
    if m.r32(o + 0x40) == 0 {
        let p = m.r32(o + 0x10);
        if m.r32(p + 0x3d4) == 0 {
            let c = m.r32(p + 0x3cc);
            return ee_sub_20a7e0(m, c);
        }
    }
    false
}
/// FUN_0020b140(npc, target): spawn the alert marker object (+0x30 target position, +0x44 target, +0x54 = 4)
/// and arm the cooldown +0xf44
pub fn ee_sub_20b140<M: QMem + ?Sized>(m: &mut M, p: u32, t: u32) {
    let q = pos(m, t);
    let out = stack_push(m, 0x10);
    m.w32(out, 0);
    if m.r32(p + 0xe60) == 0 {
        let me = pos(m, p);
        let g = m.r32(0x003e_14c0);
        let r = m.call(0x0015_6698, &[g, p], &[me]).0;
        m.w32(out, r);
    } else {
        let me = pos(m, p);
        let c = m.r32(p + 0xe60);
        m.call(0x0015_6548, &[c, p, out], &[me]);
    }
    let a = m.r32(out);
    m.wv(a + 0x30, q);
    m.w32(a + 0x44, t);
    m.w32(a + 0x54, 4);
    let v = clock(m) + rof(0x002c_3db8);
    m.wf(p + 0xf44, v);
    stack_pop(m, 0x10);
}
/// FUN_00175d00: can the NPC notice / engage its target (+0x30) from here (line-of-sight cast included)
pub fn ee_sub_175d00<M: QMem + ?Sized>(m: &mut M, o: u32) -> u64 {
    let p = m.r32(o + 0x10);
    if m.r32(p + 0xfe8) & 8 == 0 || m.r32(p + 0xf00) != 0 || m.r32(o + 0x3c) != 0 {
        return 0;
    }
    let t = m.r32(o + 0x30);
    if t == 0 || (m.r64(t + 0x28) >> 0x3c) & 1 == 0 {
        return 0;
    }
    if m.call(0x0013_2b10, &[t, 0x32], &[]).0 != 0 {
        if vcall(m, t, 0xc, 0x26c, &[], &[]).0 == 2 {
            return 0;
        }
    }
    let mut b2 = false;
    if m.call(0x0013_2b10, &[t, 0x34], &[]).0 != 0 {
        let s = m.r32(t + 0x4c8) as i32;
        if m.r32(t + 0x4cc) != 0xffff_ffff {
            b2 = s < 5 && 2 < s;
        }
        let k = m.r32(t + 0x2dc);
        if k != 0xffff_ffff && k == 0x11 {
            b2 = true;
        }
        if m.r32(t + 0xf54) != 0xffff_ffff {
            b2 = true;
        }
        if m.call(0x0013_2b10, &[t, 0x35], &[]).0 != 0 {
            let c = m.r32(t + 0x18c8) as i32;
            if c == 3 || (2 < c && c < 0x17 && 0x14 < c) {
                b2 = true;
            }
        }
    }
    let me = pos(m, p);
    let tp = pos(m, t);
    let d = sub3(tp, me);
    let heading = m.call(0x0020_4740, &[p], &[]).1[0];
    let sp = stack_push(m, 0x200);
    let (a0, a1, a2, a3) = (sp + 0x70, sp + 0x74, sp + 0x78, sp + 0x7c);
    m.call(0x0011_efb0, &[a0, a1, a2], &[d]);
    let dist = m.rf(a2);
    let ok = |m: &mut M, v: f32, at: u32| m.call(0x0011_d910, &[at], &[f4(v)]).0 != 0;
    if !ok(m, dist, p + 0xf48) {
        stack_pop(m, 0x200);
        return 0;
    }
    let yaw = m.rf(a0);
    let w = m.call(0x0011_cf60, &[], &[f4(yaw - heading)]).1[0];
    if !ok(m, w, p + 0xf50) {
        stack_pop(m, 0x200);
        return 0;
    }
    let fwd = m.call(0x001a_6318, &[p], &[]).0;
    let fv = m.rv(fwd);
    m.call(0x0011_e958, &[0, a3], &[fv]);
    let pitch = m.rf(a1);
    let fp = m.rf(a3);
    let w = m.call(0x0011_cf60, &[], &[f4(pitch - fp)]).1[0];
    if !ok(m, w, p + 0xf58) {
        stack_pop(m, 0x200);
        return 0;
    }
    if !b2 {
        if m.r32(o + 0x34) == 0 {
            if clock(m) - m.rf(o + 0x38) < 3.0 {
                stack_pop(m, 0x200);
                return 0;
            }
        } else if m.r32(o + 0x34) != 2 {
            stack_pop(m, 0x200);
            return 0;
        }
    }
    m.call(0x001a_e478, &[sp], &[]);
    let w0 = m.r32(p + 0xec);
    m.call(0x0020_faa0, &[WORLD, 0x0020_fa10, w0, sp + 0x80, sp + 0x84], &[me, tp]);
    let (i90, i8c) = (m.r32(sp + 0x80), m.r32(sp + 0x84));
    let hit = m.call(0x0014_cc58, &[0, i90, i8c, sp + 0x10], &[me, tp]).0;
    let mut r = 0u64;
    if hit == 0 || {
        let a = m.r32(hit + 4);
        let b = m.r32(a + 0x14);
        m.r32(b) == t
    } {
        r = 1;
    }
    m.call(0x001a_e4a0, &[sp, 2], &[]);
    stack_pop(m, 0x200);
    r
}
/// FUN_00176098 (slot 9): guard AI state machine on +0x34 (0 / 3 = watch / chase, 1, 2, 4, 5)
pub fn c2d05a0_v9<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef98, &[o], &[]);
    let mut st0 = m.r32(o + 0x34) as i32;
    loop {
        let p = m.r32(o + 0x10);
        let sub = m.r32(o + 0x18);
        match st0 {
            0 | 3 => {
                if m.call(0x001e_f0a8, &[o], &[]).0 == 0 {
                    if m.call(0x0020_9098, &[p], &[]).0 != 0 {
                        anim(m, o, 5);
                    } else if ee_sub_176060(m, o) {
                        anim(m, o, 4);
                    } else if m.call(0x0017_5ff0, &[o], &[]).0 != 0 {
                        anim(m, o, 1);
                    } else if ee_sub_175d00(m, o) != 0 {
                        anim(m, o, 2);
                    } else {
                        let tg = m.r32(o + 0x30);
                        let tp = pos(m, tg);
                        let me = pos(m, p);
                        let d = sub3(tp, me);
                        let mut dist = (d[0] * d[0] + d[1] * d[1]).sqrt();
                        let k = m.r32(p + 0xfbc);
                        if k == 8 || k == 0xd {
                            let c = m.r32(p + 0xfc4);
                            if c != 0 {
                                dist /= m.rf(c + 0x20);
                            }
                        }
                        let sf = m.call(0x001c_0d70, &[1], &[tp]).0;
                        let cur = m.r32(o + 0x34);
                        if cur == 0 {
                            let mut skip = false;
                            if dist <= m.rf(p + 0x318) && sf == e8_dc_10(m, p) {
                                skip = true;
                            }
                            if !skip {
                                anim(m, o, 3);
                            }
                        } else if cur == 3 {
                            let mut go0 = false;
                            let mut last = m.r32(o + 0x44);
                            if dist < m.rf(p + 0x314) && sf == e8_dc_10(m, p) {
                                go0 = true;
                            }
                            if go0 {
                                anim(m, o, 0);
                            } else {
                                let mut skip = false;
                                if last == sf {
                                    let q = m.rv(o + 0x50);
                                    if len3(sub3(tp, q)) <= rof(0x002b_e614) {
                                        skip = true;
                                    }
                                }
                                if !skip {
                                    m.w32(o + 0x44, sf);
                                    m.wv(o + 0x50, tp);
                                    let b = stack_push(m, 0x40);
                                    m.call(0x0025_62a8, &[b], &[]);
                                    m.wv(b + 0x10, tp);
                                    let sb = m.r32(o + 0x18);
                                    m.call(0x0017_dd38, &[sb, b], &[]);
                                    stack_pop(m, 0x40);
                                }
                            }
                        }
                    }
                }
            }
            1 => {
                if m.call(0x0020_9098, &[p], &[]).0 == 0 {
                    if m.call(0x001e_f0a8, &[o], &[]).0 == 0 {
                        if m.call(0x0017_5748, &[sub], &[]).0 != 0 {
                            anim(m, o, 0);
                        }
                    }
                } else {
                    anim(m, o, 5);
                }
            }
            2 => {
                if m.call(0x0020_9098, &[p], &[]).0 != 0 {
                    anim(m, o, 5);
                } else if m.call(0x001e_f0a8, &[o], &[]).0 == 0 {
                    if ee_sub_17b7f8(m, sub) {
                        anim(m, o, 0);
                    }
                }
            }
            4 => {
                if m.call(0x001e_f0a8, &[o], &[]).0 == 0 {
                    if m.call(0x0017_4fb8, &[sub], &[]).0 != 0 {
                        anim(m, o, 0xffff_ffff);
                    }
                }
            }
            5 => {
                if m.call(0x001e_f0a8, &[o], &[]).0 == 0 {
                    if m.call(0x0020_9098, &[p], &[]).0 == 0 {
                        anim(m, o, 3);
                    }
                }
            }
            _ => {}
        }
        let st1 = m.r32(o + 0x34) as i32;
        if st0 == st1 {
            let p = m.r32(o + 0x10);
            if m.r32(p + 0xf40) != 0
                && st0 < 4
                && -1 < st0
                && m.rf(p + 0xf44) <= clock(m)
                && m.rf(0x003e_1be8) <= clock(m)
            {
                let t = m.r32(o + 0x30);
                ee_sub_20b140(m, p, t);
            }
            return;
        }
        st0 = st1;
    }
}
