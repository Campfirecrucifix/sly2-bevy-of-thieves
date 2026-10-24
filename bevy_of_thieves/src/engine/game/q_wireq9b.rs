//! WIRE-Q9: FUN_00195fb8 JmpSetJtbs (player JMP sub-state switch), 1:1 from the EE asm (queue port).
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

const GS: u32 = 0x003d_d490;

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

fn jt58<M: QMem + ?Sized>(m: &mut M, j: u32) -> (u32, u32) {
    let jt = m.r32(j);
    (jt, m.r32(jt + 0x58))
}

fn zero_vel<M: QMem + ?Sized>(m: &mut M, j: u32) {
    let jt = m.r32(j);
    m.w32(jt + 0x194, 0);
    m.w32(jt + 0x198, 0);
    m.w32(jt + 0x190, 0);
}

fn zero_q<M: QMem + ?Sized>(m: &mut M, j: u32) {
    let jt = m.r32(j);
    let z = m.rv(0x002a_2dd0);
    m.wv(jt + 0x190, z);
}

fn cam_restore<M: QMem + ?Sized>(m: &mut M, j: u32) {
    if m.r32(j + 0x174) != 0 {
        let jt = m.r32(j);
        let f = m.rf(j + 0x168);
        m.call(0x0020_4da8, &[jt], &[f4(f)]);
        let jt = m.r32(j);
        m.call(0x0014_0c88, &[jt, 0], &[]);
        m.w32(j + 0x174, 0);
    }
}

fn sub(a: V4, b: V4) -> V4 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2], a[3] - b[3]]
}

/// FUN_00195fb8(j, new, arg) JmpSetJtbs: leave the old jump sub-state (+0x1c), enter `new`, pick its
/// animation (s0) + blend (f22/f23, flags s3) and play it via FUN_001305b8 on jt+0x1030; then jt vslot 89.
pub fn ee_sub_195fb8<M: QMem + ?Sized>(m: &mut M, j: u32, new: u32, arg: u32) {
    let old = m.r32(j + 0x1c);
    if new == old {
        return;
    }
    let sp = stack_push(m, 0x160);
    match old {
        6 => {
            let s0 = m.r32(j + 0xc);
            m.w32(j + 0xc, 0);
            let jt = m.r32(j);
            if m.r32(jt + 0x4c8) != 1 {
                m.call(0x0013_4320, &[GS, s0], &[]);
                if m.r32(s0 + 0x17c) == 1 {
                    m.call(0x0018_3b80, &[s0, 0], &[]);
                }
            }
            let (jt, o) = jt58(m, j);
            m.call(0x0023_03a0, &[jt, o, 2], &[]);
            zero_q(m, j);
        }
        7 => {
            let a = m.r32(j + 0xc);
            m.w32(j + 0xc, 0);
            m.call(0x0013_4320, &[GS, a], &[]);
        }
        8..=0x10 | 0x12 => {
            let (jt, o) = jt58(m, j);
            m.call(0x0023_03a0, &[jt, o, 2], &[]);
            zero_q(m, j);
            let v = m.r32(j + 0x1c) as i32;
            if (0xe..0x10).contains(&v) {
                let jt = m.r32(j);
                if jt == m.r32(0x002e_1e40) {
                    m.call(0x0018_e540, &[jt, u32::MAX], &[]);
                } else {
                    m.call(0x001c_cac0, &[jt + 0x900, u32::MAX], &[]);
                }
            }
        }
        0x11 => {
            let jt = m.r32(j);
            let c = m.r32(j + 0xc8);
            let k = m.r32(c + 4);
            m.call(0x0020_4f78, &[jt, k, 0], &[]);
            m.w32(j + 0xc8, 0);
        }
        0x13 | 0x14 => {
            if !(0x13..0x15).contains(&(new as i32)) {
                let jt = m.r32(j);
                let c = m.r32(jt + 0x1d28);
                if c != 0 {
                    m.call(0x001f_0660, &[c, 0x5f8], &[]);
                }
                let jt = m.r32(j);
                m.call(0x0014_0c88, &[jt, 0], &[]);
            }
        }
        0x15 => {
            m.call(0x0012_e460, &[j + 0x1f0], &[]);
            let (jt, o) = jt58(m, j);
            m.call(0x0023_03a0, &[jt, o, 2], &[]);
            let jt = m.r32(j);
            m.call(0x001f_6860, &[jt, 0], &[]);
        }
        _ => {}
    }
    let jt = m.r32(j);
    let mut s1 = m.r32(jt + 0x1030);
    let mut s0 = 0u32;
    m.w32(j + 0x1c, new);
    m.w32(j + 0xc, arg);
    let mut s3 = 1u32;
    let mut f22 = 0.0f32;
    let mut f23 = 1.0f32;
    let t = m.r32(0x002d_ded0);
    m.w32(j + 0x20, t);
    match new as i32 {
        -1 => {
            cam_restore(m, j);
            m.w32(j + 0xbc, 0);
            m.w32(j + 4, u32::MAX);
        }
        0 | 1 | 4 | 5 | 7 => {
            let jt = m.r32(j);
            s0 = match m.r32(j + 4) {
                0xe => m.r32(j + 0x158),
                0xd => m.r32(j + 0x148),
                0xf => m.r32(jt + 0x1518),
                0x10 => m.r32(jt + 0x1514),
                _ => 0,
            };
            if s0 == 0 {
                let i = m.r32(j + 8);
                s0 = m.r32(j + 0x120 + 4 * i);
                if s0 == 0 {
                    s0 = m.r32(j + 0x120);
                }
            }
        }
        2 | 3 => {
            if m.r32(j + 4) == 0xe {
                s0 = m.r32(j + 0x15c);
            }
            if s0 == 0 {
                let i = m.r32(j + 8);
                s0 = m.r32(j + 0x134 + 4 * i);
                if s0 == 0 {
                    s0 = m.r32(j + 0x134);
                    if s0 == 0 {
                        s0 = m.r32(j + 0x120);
                    }
                }
            }
            m.w32(j + 0x28, 0);
        }
        6 => {
            let (jt, o) = jt58(m, j);
            m.call(0x0023_03a0, &[jt, o, 0x11], &[]);
            m.call(0x0013_4320, &[GS, arg], &[]);
            m.call(0x0018_3b80, &[arg, 1], &[]);
            zero_vel(m, j);
            let jt = m.r32(j);
            let c = m.r32(jt + 0x1030);
            s0 = m.r32(c + 0x14);
        }
        8 => {
            let (jt, o) = jt58(m, j);
            m.call(0x0023_03a0, &[jt, o, 0x11], &[]);
            zero_vel(m, j);
            s0 = m.r32(s1 + 0x14);
        }
        0xb..=0xf => {
            let (jt, o) = jt58(m, j);
            m.call(0x0023_03a0, &[jt, o, 0x11], &[]);
            zero_vel(m, j);
            m.w32(j + 0x28, 0);
            m.w32(j + 8, 0);
            s0 = m.r32(j + 0x120);
            if (0xe..0x10).contains(&(m.r32(j + 0x1c) as i32)) {
                let jt = m.r32(j);
                let a = m.r32(j + 0xc);
                if jt == m.r32(0x002e_1e40) {
                    m.w32(jt + 0x1854, a);
                    m.call(0x0018_e540, &[jt, 0], &[]);
                } else {
                    f23 = 0.0;
                    m.w32(jt + 0x984, a);
                    m.call(0x001c_cac0, &[jt + 0x900, 0], &[]);
                    let jt = m.r32(j);
                    s0 = m.r32(jt + 0x930);
                    f22 = m.rf(s0 + 0x74) * 0.5;
                }
            }
        }
        9 | 10 | 0x10 | 0x12 => {
            let (jt, o) = jt58(m, j);
            m.call(0x0023_03a0, &[jt, o, 0x11], &[]);
            zero_vel(m, j);
            m.w32(j + 0x28, 0);
            let l = m.call(0x0019_a750, &[j, 4], &[]).1;
            let jt = m.r32(j);
            vslot(m, jt, 34, &[], &[l]);
            m.w32(j + 8, 0);
            s0 = if m.r32(j + 0x1c) == 0x10 { m.r32(j + 0x14c) } else { m.r32(j + 0x148) };
            s1 = 0;
        }
        0x11 => {
            s3 = 9;
            let jt = m.r32(j);
            vslot(m, jt, 90, &[], &[]);
            let jt = m.r32(j);
            let c = m.r32(j + 0xc8);
            let k = m.r32(c + 4);
            m.call(0x0020_4f78, &[jt, k, 1], &[]);
            let c = m.r32(j + 0xc8);
            let q = m.rv(j + 0xd0);
            let k = m.r32(c);
            let r = m.call(0x0022_fc48, &[k, 0], &[q]).1;
            let e = m.rv(j + 0xe0);
            let ne = [-e[0], -e[1], -e[2], -e[3]];
            m.wv(sp, r);
            m.wv(sp + 0x40, [-1.0, 0.0, 0.0, 0.0]);
            m.wv(sp + 0x10, ne);
            let a1 = m.rv(0x002a_2dc0);
            let a2 = m.rv(0x002a_2db0);
            m.call(0x0011_e2d8, &[sp + 0x20], &[ne, a1, a2]);
            let f = m.rv(j + 0xf0);
            let c1 = m.rv(sp + 0x30);
            let c5 = m.rv(sp + 0x10);
            let c4 = m.rv(sp + 0x20);
            let mut o = f;
            for i in 0..3 {
                o[i] = (c5[i] * f[0] + c4[i] * f[1]) + c1[i] * f[2];
            }
            let d = sub(m.rv(sp), o);
            m.wv(sp, d);
            m.wv(sp + 0xc0, d);
            let jt = m.r32(j);
            let p = m.call(0x001a_62e8, &[jt], &[]).1;
            let d = m.rv(sp + 0xc0);
            m.wv(jt + 0x1010, sub(d, p));
            zero_vel(m, j);
            m.w32(j + 0x28, 0);
            s0 = m.r32(j + 0x154);
            cam_restore(m, j);
        }
        0x13 => {
            let jt = m.r32(j);
            let i = m.r32(j + 8);
            let f = m.rf(jt + 0x1a8) - m.rf(0x002b_f780 + 4 * i);
            m.call(0x0020_4660, &[jt], &[f4(f)]);
            let jt = m.r32(j);
            m.w32(j + 0x28, 0);
            let c = m.r32(j + 0x1bc);
            s0 = m.r32(j + 0x178);
            m.call(0x0014_0c88, &[jt, c], &[]);
            let v = m.rf(0x002b_f798) * 0.5 + m.rf(0x002b_f79c) * 0.5;
            let c = m.r32(j + 0x1bc);
            let q = m.r32(c + 0x114);
            m.wf(q + 4, v);
            let jt = m.r32(j);
            let c = m.r32(jt + 0x1d28);
            if c != 0 {
                m.call(0x001f_0660, &[c, 0x5f7], &[]);
            }
        }
        0x14 => {
            s3 = 9;
            for o in [0x188, 0x194, 0x1a0, 0x1ac] {
                m.w32(j + o, 0);
            }
            let a = m.r32(j + 0x1b8);
            m.call(0x0013_84c0, &[a], &[]);
            m.wf(j + 0x17c, 1.0);
            s0 = m.r32(j + 0x1b8);
        }
        0x15 => {
            let mut f21 = -1000.0f32;
            let mut f20 = -1.0f32;
            let jt = m.r32(j);
            let o = m.r32(jt + 0x58);
            let p0 = m.rv(o + 0x70);
            m.wv(sp + 0x50, p0);
            let t = m.r32(j + 0x1d0);
            if t != 0 {
                let hit = m.call(0x0013_2b10, &[t, 0x13], &[]).0 != 0 || {
                    let t = m.r32(j + 0x1d0);
                    m.call(0x0013_2b10, &[t, 0xb], &[]).0 != 0
                };
                if hit {
                    f21 = 0.0;
                    f20 = 500.0;
                }
            }
            let jt = m.r32(j);
            let fa = m.rf(0x002b_f7a0);
            let r = m.call(0x0019_f3f8, &[jt, j + 0x1d0], &[f4(fa)]).1;
            m.wv(sp + 0x60, r);
            let d = sub(r, m.rv(sp + 0x50));
            m.wv(sp + 0x70, d);
            let jt = m.r32(j);
            let gp = m.r32(jt + 0xe8);
            let g = m.rv(gp + 0x80);
            m.wv(sp + 0x80, g);
            let f0 = m.call(0x0019_6928, &[], &[f4(1000.0), f4(f21), f4(1.0), f4(f20), d, g]).1[0];
            let tt = m.rf(0x002b_f7a0).max(f0);
            let h = tt * -0.5;
            let q = 1.0 / tt;
            let mut v = [0.0f32; 4];
            for i in 0..4 {
                let acc = d[i] * q + if i == 3 { 1.0 } else { 0.0 };
                v[i] = acc + g[i] * h;
            }
            m.wv(sp + 0x90, v);
            let jt = m.r32(j);
            vslot(m, jt, 34, &[], &[v]);
            let mut ve = [0.0f32; 4];
            for i in 0..4 {
                let acc = v[i] + if i == 3 { 1.0 } else { 0.0 };
                ve[i] = acc + g[i] * tt;
            }
            m.wv(sp + 0xa0, ve);
            let te = m.rf(0x002d_ded0) + tt;
            let (_, o) = jt58(m, j);
            let k = m.r32(o + 0xa0);
            let r = m.rv(sp + 0x60);
            let qa = m.call(0x0022_fc48, &[0, k], &[r]).1;
            let (_, o) = jt58(m, j);
            let k = m.r32(o + 0xa0);
            let qb = m.call(0x0022_fcc8, &[0, k], &[ve]).1;
            let (jt, o) = jt58(m, j);
            let hh = m.call(0x0012_de58, &[jt, o, 5], &[f4(te), qa, qb]).0;
            m.w32(j + 0x1f0, hh);
            let jt = m.r32(j);
            m.call(0x001f_6860, &[jt, 1], &[]);
            zero_vel(m, j);
            let d = m.rv(sp + 0x70);
            let jt = m.r32(j);
            m.call(0x0020_46b0, &[jt], &[d]);
            s0 = m.r32(j + 0x120);
        }
        0x16 => {
            s0 = m.r32(j + 0x1c8);
            s3 = 9;
        }
        _ => {}
    }
    let v = m.r32(j + 0x1c);
    if v == 0 || v == 2 {
        m.w32(j + 0x30, 0);
    } else if v == 0x12 {
        let jt = m.r32(j);
        m.call(0x0013_4320, &[GS, jt], &[]);
    }
    if m.r32(j + 0x1c) != u32::MAX {
        if s1 == 0 || m.r32(s1 + 0x14) != s0 {
            let jt = m.r32(j);
            m.call(0x001f_15b8, &[jt + 0x1034], &[]);
            let jt = m.r32(j);
            m.call(0x0013_05b8, &[s0, jt, s3, jt + 0x1030], &[f4(f22), f4(f23)]);
        }
        let jt = m.r32(j);
        vslot(m, jt, 89, &[], &[]);
    }
    stack_pop(m, 0x160);
}
