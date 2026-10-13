//! WP-Q7: class C2cbe68 (parent C2cb230) queue package: slot 4 (message handler), 5 (descriptor), 97 / 98
//! and the in-package callees FUN_0019f220, FUN_001ccc88 (+0x900 sub-object: throw / launch along a
//! curve), FUN_002245b8. 1:1 against `QMem` (layout: +0x4c8 state, +0x4d4 target, +0x900 launcher,
//! +0xdf0 / +0x11b0 / +0x1310 sub-records, +0x1030 / +0x1034 owned objects, +0x1090 / +0x1094 hooks).
//! Out-of-package callees go through `QMem::call` by address (conventions: classes::q::q7util).
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem};
use crate::engine::classes::q::q7util::*;

/// FUN_00113d50 (slot 5): class descriptor
pub fn c2cbe68_v5() -> u32 {
    0x002b_aef8
}

/// FUN_001409b8 (slot 97): 0 when FUN_0020f0b0(world) else -1
pub fn c2cbe68_v97<M: QMem + ?Sized>(m: &mut M) -> u32 {
    if m.call(0x0020_f0b0, &[WORLD], &[]).0 != 0 {
        0
    } else {
        u32::MAX
    }
}

/// FUN_00141c00 (slot 98)
pub fn c2cbe68_v98() -> u32 {
    0
}

/// FUN_0019f220: every object FUN_0015e9c8(w, 0x105, 0x38, 0, 0x20, buf) finds -> FUN_00207030(o, b)
pub fn ee_sub_19f220<M: QMem + ?Sized>(m: &mut M, w: u32, b: u32) {
    let buf = stack_push(m, 0x80);
    let n = m.call(0x0015_e9c8, &[w, 0x105, 0x38, 0, 0x20, buf], &[]).0 as i32;
    for i in 0..n.max(0) as u32 {
        let o = m.r32(buf + 4 * i);
        m.call(0x0020_7030, &[o, b], &[]);
    }
    stack_pop(m, 0x80);
}

/// FUN_002245b8: slot 75 (+0x264) with 5 when +0x15c == 0xb else 3
pub fn ee_sub_2245b8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let k = if m.r32(o + 0x15c) == 0xb { 5 } else { 3 };
    vslot(m, o, 75, &[k], &[]);
}

/// FUN_00140f90 (slot 4): message handler, then FUN_001eed78(+0x26c, msg, p) and the parent's slot 4
/// (FUN_001c3e20).
pub fn c2cbe68_v4<M: QMem + ?Sized>(m: &mut M, o: u32, msg: u32, p: u32) {
    match msg {
        1 => {
            if m.r32(o + 0x4c8) == 0xb && m.r32(o + 0x4d4) == p {
                m.call(0x0013_3448, &[p, o], &[]);
                m.w32(o + 0x4d4, 0);
            }
        }
        3 => {
            let own = m.r32(o + 0x1030) == p || {
                let s = m.r32(o + 0x1034);
                s != 0 && m.r32(s + 0x30) == p
            };
            if own {
                m.w32(o + 0x1038, u32::MAX);
                let st = m.r32(o + 0xdf4);
                if st.wrapping_sub(8) < 5 {
                    m.call(0x001a_15d0, &[o + 0xdf0], &[]);
                } else {
                    let c = m.r32(o + 0x11b4) as i32;
                    if (1..3).contains(&c) {
                        m.call(0x0022_06c8, &[o + 0x11b0, 0], &[]);
                    } else if (5..7).contains(&c) {
                        m.call(0x0022_06c8, &[o + 0x11b0, 4], &[]);
                    }
                }
            }
        }
        4 | 5 => {
            let ok = m.r32(p + 4) == o && (msg == 4 || m.r32(p) == m.r32(o + 0x108c));
            if ok {
                let v = if msg == 4 { m.r32(p) } else { 0 };
                m.w32(o + 0x108c, v);
                let mut i = 0;
                while (i as i32) < m.r32(o + 0x1094) as i32 {
                    let arr = m.r32(o + 0x1090);
                    let e = m.r32(arr + i * 8);
                    vslot(m, e, if msg == 4 { 7 } else { 8 }, &[], &[]);
                    if msg == 4 {
                        let arr = m.r32(o + 0x1090);
                        m.w32(arr + i * 8 + 4, 0);
                    }
                    i += 1;
                }
            }
        }
        9 => {
            let s = m.r32(p);
            if m.r32(s + 0x18) == o {
                msg9(m, o, p);
            }
        }
        0xb => {
            if m.r32(p + 4) == o {
                m.call(0x0013_0658, &[o + 0x1310], &[]);
                m.w32(o + 0x1314, u32::MAX);
            }
        }
        _ => {}
    }
    m.call(0x001e_ed78, &[o + 0x26c, msg, p], &[]);
    m.call(0x001c_3e20, &[o, msg, p], &[]);
}

fn msg9<M: QMem + ?Sized>(m: &mut M, o: u32, p: u32) {
    match m.r32(p + 8) {
        0x1c7 => {
            if m.r32(o + 0x4c8) == 0xc {
                let a = m.r32(o + 0x1030);
                m.call(0x0013_1a00, &[a], &[f(1.0)]);
                let t = m.r32(o + 0x4d4);
                m.call(0x001c_d098, &[o + 0x900, t], &[]);
            }
        }
        0x1c8 => {
            if m.r32(o + 0x4c8) == 0xc {
                let t = m.r32(o + 0x4d4);
                ee_sub_1ccc88(m, o + 0x900, t);
            }
        }
        0x161 => m.w32(o + 0x1044, 1),
        0x15e => {
            // vector FUN_00204e38(f = p[1], o); +0x190 = -v; slot 34 with the world-rotated v (rows +0x1b0)
            let s = m.rf(p + 4);
            let v = m.call(0x0020_4e38, &[o], &[f(s)]).1;
            m.wv(o + 0x190, scale(v, -1.0));
            let r = rows3(m, o + 0x1b0);
            let w = mul3(&r, v, 0.0);
            vslot(m, o, 34, &[], &[w]);
        }
        0x162 => {
            let t = m.rv(o + 0x1060);
            m.w32(o + 0x1044, 0);
            let pos = m.call(0x001a_62e8, &[o], &[]).1;
            m.wv(o + 0x1070, sub(t, pos));
            m.w32(o + 0x1078, 0);
        }
        0x27b => {
            if m.r32(o + 0x11b4) != u32::MAX {
                m.call(0x0022_05a8, &[o + 0x11b0], &[]);
            }
        }
        0x27c => {
            if m.r32(o + 0x4c8) == 0xb {
                msg_27c(m, o);
            }
        }
        _ => {}
    }
}

fn msg_27c<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let t = m.r32(o + 0x4d4);
    if t != 0 && m.call(0x0013_2b10, &[t, 100], &[]).0 != 0 && m.call(0x0022_4590, &[t], &[]).0 != 0 {
        ee_sub_2245b8(m, t);
    }
    if m.r32(o + 0x130c) != 0 {
        if m.r32(o + 0x1314) == m.r32(o + 0x9bc) {
            m.call(0x0013_0658, &[o + 0x1310], &[]);
            m.w32(o + 0x1314, u32::MAX);
        } else {
            m.call(0x0014_1cd0, &[o], &[]);
            let a = m.r32(o + 0x130c);
            m.call(0x0013_05b8, &[a, o, 8, o + 0x1310], &[f(0.0), f(1.0)]);
            let k = m.r32(o + 0x9bc);
            m.w32(o + 0x1314, k);
        }
        m.w32(o + 0x130c, 0);
    } else if m.r32(o + 0x9bc) == u32::MAX {
        m.w32(o + 0x130c, 0);
    } else {
        m.call(0x0014_1cd0, &[o], &[]);
        match m.r32(o + 0x9bc) {
            0x15 => ee_sub_19f220(m, WORLD, 0x15),
            0xe => {
                m.call(0x0019_f170, &[WORLD, 1], &[]);
            }
            0x17 => {
                let g = m.r32(0x002e_1e40);
                let a = m.r32(g + 0x2010);
                let r = m.call(0x001c_b250, &[a, 0], &[]).0;
                if r != 0 {
                    m.call(0x001a_58d0, &[r, 0, 3], &[]);
                }
                safe_walk(m, 0x003e_148c, |m, e| {
                    if m.r32(e + 0x54) == 4 && m.r32(e + 0x44) == o {
                        m.call(0x0015_6780, &[e], &[]);
                    }
                });
                // DAT_003e1be8 = clock + DAT_002bcc94
                let t = m.rf(CLOCK) + m.rf(0x002b_cc94);
                m.wf(0x003e_1be8, t);
            }
            _ => {}
        }
        m.w32(o + 0x130c, 0);
    }
    m.w32(o + 0x9bc, u32::MAX);
}

/// FUN_001ccc88 (launcher `l` = owner +0x900, l[0] = owner): drop the three handles +0x94..+0x9c
/// (FUN_0012e460), sample the owner's +0x1030 animation at label 0x1c7, build the hand matrix (owner
/// rotation +0x1b0 / position, through the anim pose FUN_001300b8 and the attach parent FUN_001cca40),
/// launch `obj` along the ballistic arc to its target (FUN_00196928 flight time, clamped >= the remaining
/// anim time) via slot 34 (velocity), and restart its motion / spin channels (FUN_0012de58 / FUN_0012dc18).
pub fn ee_sub_1ccc88<M: QMem + ?Sized>(m: &mut M, l: u32, obj: u32) {
    for k in [0x94, 0x98, 0x9c] {
        m.call(0x0012_e460, &[l + k], &[]);
    }
    let sp = stack_push(m, 0x200);
    let own = m.r32(l);
    let s7 = m.r32(own + 0x1030);
    let s0 = m.r32(s7 + 0x14);
    let t_lbl = fr(m.call(0x0012_f990, &[s0, 0x1c7], &[]));
    let rem = (t_lbl - m.rf(s7 + 0x1c)) / m.rf(s7 + 0x40);
    let sub_o = m.r32(l + 0x34);
    let r31 = vslot(m, sub_o, 31, &[sp], &[]).0;
    let own = m.r32(l);
    let own60 = m.r32(own + 0x60);
    m.call(0x0013_00b8, &[s0, own, r31, own60, sp + 0x40, 0], &[f(t_lbl), f(0.0)]);
    let own = m.r32(l);
    let pos = m.call(0x001a_62e8, &[own], &[]).1;
    m.call(0x0025_6d18, &[sp + 0xb0], &[pos]);
    copy(m, sp + 0x80, own + 0x1b0, 0x30);
    for a in [0x8c, 0xac, 0x9c] {
        m.wf(sp + a, 0.0);
    }
    m.wf(sp + 0xbc, 1.0);
    m.call(0x0011_c168, &[sp + 0x140, sp + 0x80, sp + 0x40], &[]);
    m.call(0x0011_c168, &[sp + 0x100, sp + 0x140, sp], &[]);
    copy(m, sp + 0xc0, sp + 0x100, 0x40);
    let own = m.r32(l);
    let par = m.call(0x001c_ca40, &[own, obj], &[]).0;
    if par != 0 {
        let x = vslot(m, par, 30, &[], &[]).0;
        m.call(0x001a_71b0, &[sp + 0x140, 0, obj, x], &[]);
        copy(m, sp + 0x100, sp + 0x140, 0x40);
        m.call(0x0011_c318, &[sp + 0x180, sp + 0x100], &[]);
        m.call(0x0011_c168, &[sp + 0x140, sp + 0xc0, sp + 0x180], &[]);
        copy(m, sp + 0xc0, sp + 0x140, 0x40);
    }
    m.call(0x001a_58d0, &[obj, 0, 0xf], &[]);
    let e8 = m.r32(obj + 0xe8);
    let q = if e8 == 0 { 0x002c_1ee0 } else { e8 + 0x80 };
    let g = m.rv(q);
    let hand = m.rv(sp + 0xf0);
    let p = m.call(0x001a_62e8, &[obj], &[]).1;
    let d = sub(hand, p);
    let c = m.rf(0x002c_1ef0);
    let t = fr(m.call(0x0019_6928, &[], &[f(c), f(-c), f(1.0), f(-1.0), d, g]));
    let t = t.max(rem);
    m.call(0x0013_1a00, &[s7], &[f(rem / t)]);
    let inv = 1.0 / t;
    let vel = add(scale(d, inv), scale(g, -0.5 * t));
    let v_end = add(scale(d, inv), scale(g, 0.5 * t));
    vslot(m, obj, 34, &[], &[vel]);
    let t_end = m.rf(CLOCK) + t;
    let m58 = m.r32(obj + 0x58);
    let k58 = m.r32(m58 + 0xa0);
    let a = m.call(0x0022_fc48, &[0, k58], &[hand]).1;
    let b = m.call(0x0022_fcc8, &[0, k58], &[v_end]).1;
    let h = m.call(0x0012_de58, &[obj, m58, 5], &[f(t_end), a, b]).0;
    m.w32(l + 0x94, h);
    let m5c = m.r32(obj + 0x5c);
    let k5c = m.r32(m5c + 0xa0);
    m.call(0x0022_fd38, &[sp + 0x130, 0, k5c, sp + 0xc0], &[]);
    copy(m, sp + 0x100, sp + 0x130, 0x10);
    copy(m, sp + 0x110, sp + 0x140, 0x10);
    copy(m, sp + 0x120, sp + 0x150, 0x10);
    let qv = m.call(0x0011_bff0, &[sp + 0x100], &[]).1;
    let m5c = m.r32(obj + 0x5c);
    let h = m.call(0x0012_dc18, &[obj, m5c, 5, 9, 2], &[qv]).0;
    m.w32(l + 0x9c, h);
    let m5c = m.r32(obj + 0x5c);
    if m.r32(m5c + 0x90) == h {
        m.wf(m5c + 0xa8, t_end);
    }
    m.call(0x001f_6860, &[obj, 1], &[]);
    stack_pop(m, 0x200);
}
