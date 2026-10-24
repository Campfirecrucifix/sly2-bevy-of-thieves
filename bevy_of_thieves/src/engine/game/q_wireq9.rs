//! WIRE-Q9: 1:1 queue ports (against `QMem`) of the player-JT live frontier (wireall_frontier.tsv natives
//! FUN_0019cd38 SetBinocState, FUN_00195a00 JMP start) and their unported callees. Ported from the EE asm;
//! callees go through `m.call` / `vslot` (the live dispatch routes ported ones).
use crate::engine::classes::q::q2mem::{vslot, QMem, V4};

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

const CM: u32 = 0x002d_df40; // camera manager (g_cm)
const SB: u32 = 0x003d_d040; // sound bus
const GS: u32 = 0x003d_d490; // game state

/// FUN_0019cd38(b, st): binoc state change. b+0 jt, +4 state, +8 t (= DAT_002dded0), +0xc/+0x10 anim
/// ids, +0x14 blend out, +0x18 cam object, +0x20 kind (0 Sly / 1 Bentley / 2 vehicle), +0x24 HUD,
/// +0x40 vehicle, +0x44 HUD arg.
pub fn ee_sub_19cd38<M: QMem + ?Sized>(m: &mut M, b: u32, st: u32) {
    let old = m.r32(b + 4);
    if st == old {
        return;
    }
    let one = f4(1.0);
    if old == u32::MAX {
        if m.r32(b + 0x20) == 1 {
            m.call(0x0020_f198, &[GS], &[]);
            m.call(0x0020_f0c0, &[GS], &[]);
        }
        let jt = m.r32(b);
        let c = m.r32(jt + 0x1030);
        if c != 0 {
            m.w32(b + 0x18, c);
        } else {
            let d = m.r32(jt + 0x1034);
            if d != 0 {
                let v = m.r32(d + 0x30);
                m.w32(b + 0x18, v);
            }
        }
        let o = m.r32(b + 0x18);
        if o != 0 {
            vslot(m, o, 0, &[], &[]);
            m.call(0x0013_1a00, &[o], &[f4(0.0)]);
        }
    } else if old == 1 {
        let k = m.r32(b + 0x20);
        if k == 1 {
            let jt = m.r32(b);
            m.call(0x0014_e4c8, &[CM, 0xf, 8, CM + 0x520, jt, 0], &[]);
            let v = m.r32(b + 0x40);
            if v != 0 {
                vslot(m, v, 83, &[0x3f, 1], &[]);
                m.call(0x001f_5720, &[v, 0], &[]);
            }
            let jt = m.r32(b);
            vslot(m, jt, 83, &[0x3f, 0], &[]);
        } else if k == 0 {
            let jt = m.r32(b);
            m.call(0x0014_e4c8, &[CM, 0xf, 7, CM + 0x450, jt, 0], &[]);
        } else if k == 2 {
            let v = m.r32(b + 0x40);
            m.call(0x0014_e4c8, &[CM, 0xf, 7, CM + 0x450, v, 0], &[]);
        }
        m.call(0x0014_d8c0, &[CM], &[]);
        m.call(0x001f_ea60, &[SB, 0, 3], &[one]);
        m.call(0x001f_ea60, &[SB, 1, 3], &[one]);
        if m.r32(b + 0x20) != 2 {
            let jt = m.r32(b);
            vslot(m, jt, 86, &[0], &[]);
        }
    }
    m.w32(b + 4, st);
    let t = m.r32(0x002d_ded0);
    m.w32(b + 8, t);
    let mut anim = 0u32;
    match st as i32 {
        0 => anim = m.r32(b + 0xc),
        2 => {
            m.call(0x0019_d660, &[b + 0x24, 2], &[]);
            anim = m.r32(b + 0x10);
        }
        1 => {
            let k = m.r32(b + 0x20);
            if k == 1 {
                let jt = m.r32(b);
                m.call(0x0014_e328, &[CM, 8, CM + 0x520, jt, 0], &[]);
                let f = m.rf(CM + 0x5c4);
                m.call(0x0020_4660, &[jt], &[f4(f)]);
                let v = m.r32(b + 0x40);
                if v != 0 {
                    vslot(m, v, 82, &[0x3f, 1], &[]);
                    m.call(0x001f_5720, &[v, 1], &[]);
                    m.w32(v + 0x194, 0);
                    m.w32(v + 0x198, 0);
                    m.w32(v + 0x190, 0);
                    let z = m.rv(0x002a_2dd0);
                    vslot(m, v, 34, &[], &[z]);
                    let z = m.rv(0x002a_2dd0);
                    vslot(m, v, 35, &[], &[z]);
                }
                let h = m.r32(b + 0x44);
                m.call(0x0019_dca0, &[b + 0x24, h], &[]);
                m.call(0x0019_d660, &[b + 0x24, 0], &[]);
                let jt = m.r32(b);
                vslot(m, jt, 82, &[0x3f, 0], &[]);
            } else if k == 0 {
                let jt = m.r32(b);
                m.call(0x0014_e328, &[CM, 7, CM + 0x450, jt, 0], &[]);
                let f = m.rf(CM + 0x460);
                m.call(0x0020_4660, &[jt], &[f4(f)]);
                m.call(0x0019_d660, &[b + 0x24, 0], &[]);
            } else if k == 2 {
                let v = m.r32(b + 0x40);
                m.call(0x0014_e328, &[CM, 7, CM + 0x450, v, 0], &[]);
                m.call(0x0019_d660, &[b + 0x24, 0], &[]);
            }
            let jt = m.r32(b);
            m.w32(jt + 0x194, 0);
            m.w32(jt + 0x198, 0);
            m.w32(jt + 0x190, 0);
            let z = m.rv(0x002a_2dd0);
            vslot(m, jt, 34, &[], &[z]);
            let jt = m.r32(b);
            let z = m.rv(0x002a_2dd0);
            vslot(m, jt, 35, &[], &[z]);
            m.call(0x0014_d8c0, &[CM], &[]);
            let s = f4(f32::from_bits(0x3f19_999a));
            m.call(0x001f_ea60, &[SB, 0, 3], &[s]);
            m.call(0x001f_ea60, &[SB, 1, 3], &[s]);
            if m.r32(b + 0x20) != 2 {
                let jt = m.r32(b);
                vslot(m, jt, 86, &[1], &[]);
            }
        }
        -1 => {
            let k = m.r32(b + 0x20);
            if k == 1 {
                m.call(0x0019_d660, &[b + 0x24, u32::MAX], &[]);
                m.call(0x0020_f1a8, &[GS], &[]);
                m.call(0x0020_f0f0, &[GS], &[]);
            } else if k == 0 || k == 2 {
                m.call(0x0019_d660, &[b + 0x24, u32::MAX], &[]);
            }
            let o = m.r32(b + 0x18);
            if o != 0 {
                m.call(0x0013_1a00, &[o], &[one]);
                let o = m.r32(b + 0x18);
                vslot(m, o, 1, &[], &[]);
                m.w32(b + 0x18, 0);
            }
            m.w32(b + 0x20, u32::MAX);
        }
        _ => {}
    }
    let jt = m.r32(b);
    if anim != 0 && vslot(m, jt, 93, &[], &[]).0 == 0 {
        anim = 0;
    }
    let jt = m.r32(b);
    m.call(0x0013_05b8, &[anim, jt, 1, b + 0x14], &[f4(0.0), one]);
}

/// FUN_00195a00(j, st, t, jtbs_arg): JMP start. j+0 jt, +4 state, +8 arg, +0x18 t start, +0x1c,
/// +0x24..+0x3c jump params, +0x40 mode, +0x44..+0x4c times, +0xb0..+0xb8, +0x168..+0x174 (case 9).
pub fn ee_sub_195a00<M: QMem + ?Sized>(m: &mut M, j: u32, st: u32, arg: u32, a3: u32) {
    let sp = crate::engine::classes::q::q2mem::stack_push(m, 0x90);
    let one = 1.0f32.to_bits();
    if st != 6 {
        m.w32(j + 4, st);
    }
    m.w32(j + 8, arg);
    let jt = m.r32(j);
    vslot(m, jt, 87, &[1], &[]);
    if st != 6 {
        m.w32(j + 0x40, 0);
        let t = m.r32(0x002d_ded0);
        m.w32(j + 0x44, t);
        m.w32(j + 0x4c, t);
        m.w32(j + 0x48, t);
    }
    if st == 9 {
        let jt = m.r32(j);
        let f = m.rf(0x002b_f750);
        let w = m.r32(jt + 0x260);
        m.w32(j + 0x168, w);
        m.call(0x0020_4da8, &[jt], &[f4(f)]);
    }
    let launch = m.call(0x0019_a750, &[j, st], &[]).1;
    m.wv(sp, launch);
    let jt = m.r32(j);
    vslot(m, jt, 34, &[], &[launch]);
    let t = m.r32(0x002d_ded0);
    let jt = m.r32(j);
    m.w32(jt + 0x131c, t);
    if st != 6 {
        m.w32(j + 0x18, t);
    }
    // 0: set +0x34 = 1.0 (LAB_00195f3c) and fall to the tail; 1: tail only; 2: return (case 6 / 0xe early)
    let mut tail_34 = false;
    let jtbs = |m: &mut M, k: u32, a: u32| {
        m.call(0x0019_5fb8, &[j, k, a], &[]);
    };
    // adds FUN_0011e1a0(f12, f13, 0) to the launch velocity and sets it (vslot 34)
    let kick = |m: &mut M, f12: f32, f13: f32| {
        let r = m.call(0x0011_e1a0, &[], &[f4(f12), f4(f13), f4(0.0)]).1;
        let l = m.rv(sp);
        let s = [l[0] + r[0], l[1] + r[1], l[2] + r[2], l[3] + r[3]];
        let jt = m.r32(j);
        vslot(m, jt, 34, &[], &[s]);
    };
    match st {
        0 => {
            jtbs(m, 4, a3);
            m.w32(j + 0x28, 1);
            m.w32(j + 0x38, one);
            m.w32(j + 0x24, 0);
            tail_34 = true;
        }
        1 | 2 | 0x12 => {
            jtbs(m, 0, a3);
            m.w32(j + 0x28, 1);
            m.w32(j + 0x38, one);
            m.w32(j + 0x24, 0);
            tail_34 = true;
        }
        3 => {
            let jt = m.r32(j);
            let p = m.r32(jt + 0x150);
            if m.rf(p + 0x1c) < f32::from_bits(0x3e4c_cccd) {
                m.w32(j + 0x40, 3);
                let f12 = m.rf(jt + 0x1a8);
                let f13 = m.rf(0x002b_f754);
                kick(m, f12, f13);
            }
            jtbs(m, 0, a3);
            m.w32(j + 0x28, 1);
            m.w32(j + 0x34, one);
            m.w32(j + 0x38, 0x3f19_999a);
            m.w32(j + 0x24, 0);
        }
        6 => {
            let s = m.r32(j + 4) as i32;
            let k = if s < 0xf && s >= 0xd { 3 } else { 2 };
            jtbs(m, k, a3);
        }
        7 => {
            jtbs(m, 7, a3);
            m.w32(j + 0x28, 1);
            m.w32(j + 0x34, one);
            m.w32(j + 0x24, 0);
            m.w32(j + 0x38, one);
        }
        8 => {
            jtbs(m, 5, a3);
            m.w32(j + 0x24, 0);
            m.w32(j + 0x34, one);
            let l = m.rv(sp);
            m.w32(j + 0x38, one);
            m.w32(j + 0x28, 1);
            m.call(0x0011_ef08, &[j + 0xb0, j + 0xb8, 0], &[l]);
            m.w32(j + 0xb4, one);
        }
        9 => {
            jtbs(m, 0, a3);
            m.w32(j + 0x24, 0);
            m.w32(j + 0x28, 1);
            let jt = m.r32(j);
            let h = m.r32(j + 0x16c);
            m.call(0x0014_0c88, &[jt, h], &[]);
            let a = m.r32(0x002b_f758);
            let b = m.r32(0x002b_f75c);
            m.w32(j + 0x38, a);
            m.w32(j + 0x34, b);
            m.w32(j + 0x174, 1);
        }
        10 => {
            m.w32(j + 0x40, u32::MAX);
            jtbs(m, 0x15, a3);
            m.w32(j + 0x24, 0);
            m.w32(j + 0x28, 0);
            m.w32(j + 0x34, 0);
            m.w32(j + 0x38, 0);
        }
        0xb => {
            jtbs(m, 4, a3);
            m.w32(j + 0x24, 1);
            m.w32(j + 0x38, one);
            m.w32(j + 0x28, 0);
            tail_34 = true;
        }
        0xc => {
            jtbs(m, 1, a3);
            m.w32(j + 0x40, 3);
            m.w32(j + 0x28, 1);
            m.w32(j + 0x38, one);
            m.w32(j + 0x24, 0);
            tail_34 = true;
        }
        0xd => {
            jtbs(m, 1, a3);
            let a = m.r32(0x002b_f768);
            let b = m.r32(0x002b_f76c);
            m.w32(j + 0x28, 1);
            m.w32(j + 0x38, a);
            m.w32(j + 0x34, b);
            m.w32(j + 0x24, 0);
        }
        0xe => {
            jtbs(m, 1, a3);
            let f2 = m.rf(0x002b_f760);
            let b = m.r32(0x002b_f764);
            m.w32(j + 0x28, 1);
            m.w32(j + 0x34, b);
            m.w32(j + 0x24, 0);
            m.wf(j + 0x38, f2);
            let jt = m.r32(j);
            let p = m.r32(jt + 0x150);
            if m.rf(p + 0x1c) < f32::from_bits(0x3e4c_cccd) {
                m.w32(j + 0x40, 3);
                let f12 = m.rf(p + 0x18);
                let f13 = m.rf(0x002b_f730) * f2;
                kick(m, f12, f13);
            }
        }
        0xf | 0x10 => {
            jtbs(m, if st == 0xf { 0 } else { 4 }, a3);
            m.w32(j + 0x24, 0);
            m.w32(j + 0x38, one);
            m.w32(j + 0x28, 0);
            tail_34 = true;
        }
        0x11 => {
            if m.r32(j + 0x1c) == u32::MAX {
                // v - n (v.n) + c, v = frame sp+0x10 (not written here), n = DAT_002a2dc0, c = DAT_002bf770
                let v = m.rv(sp + 0x10);
                let n = m.rv(0x002a_2dc0);
                let d = (v[0] * n[0] + v[1] * n[1]) + v[2] * n[2];
                let c = m.rv(0x002b_f770);
                let mut r = [0.0f32; 4];
                for k in 0..4 {
                    r[k] = (v[k] - n[k] * d) + c[k];
                }
                let jt = m.r32(j);
                vslot(m, jt, 34, &[], &[r]);
                m.w32(j + 0x28, 1);
            }
            jtbs(m, 0x16, 0);
            m.w32(j + 0x24, 0);
            m.w32(j + 0x38, one);
            m.w32(j + 0x2c, 0);
            tail_34 = true;
        }
        _ => {}
    }
    if tail_34 {
        m.w32(j + 0x34, one);
    }
    if st != 6 {
        if st != 0x11 {
            let w = m.r32(j + 0x28);
            m.w32(j + 0x2c, w);
        }
        let jt = m.r32(j);
        if m.call(0x0013_2b10, &[jt, 0x35], &[]).0 != 0 {
            let jt = m.r32(j);
            m.w32(jt + 0x18f0, u32::MAX);
        }
        let jt = m.r32(j);
        let p = m.call(0x001a_62e8, &[jt], &[]).1;
        m.wf(j + 0x3c, p[2]);
    }
    crate::engine::classes::q::q2mem::stack_pop(m, 0x90);
}

/// FUN_00131a00(o, f): +0x44 = f; +0x40 = f * +0x48 * +0x4c.
pub fn ee_sub_131a00<M: QMem + ?Sized>(m: &mut M, a0: u32, f: f32) {
    let r = (f * m.rf(a0 + 0x48)) * m.rf(a0 + 0x4c);
    m.wf(a0 + 0x44, f);
    m.wf(a0 + 0x40, r);
}

/// FUN_0014d8c0(cm): +0x300 = max(+0x300, 1).
pub fn ee_sub_14d8c0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let n = m.r32(a0 + 0x300) as i32;
    m.w32(a0 + 0x300, n.max(1) as u32);
}

/// FUN_0019dca0(hud, v): +0x10 = v, or +0xc when v == 0.
pub fn ee_sub_19dca0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let v = if a1 == 0 { m.r32(a0 + 0xc) } else { a1 };
    m.w32(a0 + 0x10, v);
}

/// FUN_0020f0c0(gs): +++0x3d7c; FUN_00187078(4) on 0 -> 1.
pub fn ee_sub_20f0c0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let n = m.r32(a0 + 0x3d7c).wrapping_add(1);
    m.w32(a0 + 0x3d7c, n);
    if n == 1 {
        m.call(0x0018_7078, &[4], &[]);
    }
}

/// FUN_0020f0f0(gs): --+0x3d7c; FUN_001870a0(4) on reaching 0.
pub fn ee_sub_20f0f0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let n = m.r32(a0 + 0x3d7c).wrapping_sub(1);
    m.w32(a0 + 0x3d7c, n);
    if n == 0 {
        m.call(0x0018_70a0, &[4], &[]);
    }
}

/// FUN_0011ef08(v, *yaw, *len, *z): yaw = FUN_0011cf60(atan2(v.y, v.x)); len = |v.xy|; z = v.z.
pub fn ee_sub_11ef08<M: QMem + ?Sized>(m: &mut M, v: V4, a1: u32, a2: u32, a3: u32) {
    if a1 != 0 {
        let a = m.call(0x0024_58f0, &[], &[f4(v[1]), f4(v[0])]).1[0];
        let r = m.call(0x0011_cf60, &[], &[f4(a)]).1[0];
        m.wf(a1, r);
    }
    if a2 != 0 {
        m.wf(a2, (v[0] * v[0] + v[1] * v[1]).sqrt());
    }
    if a3 != 0 {
        m.wf(a3, v[2]);
    }
}

/// FUN_00140c88(jt, cam): if FUN_001c4840(game state) is jt: swap jt+0x1024 (FUN_00221268 old 0 / new 1).
pub fn ee_sub_140c88<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    let s = m.call(0x001c_4840, &[GS], &[]).0;
    if s != a0 {
        return;
    }
    let old = m.r32(s + 0x1024);
    if old != 0 {
        m.call(0x0022_1268, &[old, 0], &[]);
    }
    m.w32(s + 0x1024, a1);
    if a1 != 0 {
        m.call(0x0022_1268, &[a1, 1], &[]);
    }
}

/// FUN_0019a750(j, st) Jt_launch: v = jt vslot 32 (velocity); v.z = max(v.z, launch speed for jump kind st).
/// g = jt+0xe8 -> +0x88 (gravity, negative).
pub fn ee_sub_19a750<M: QMem + ?Sized>(m: &mut M, j: u32, st: u32) -> (u32, V4) {
    let sp = crate::engine::classes::q::q2mem::stack_push(m, 0xb0);
    let jt = m.r32(j);
    let p = m.call(0x001a_62e8, &[jt], &[]).1;
    m.wv(sp, p);
    let jt = m.r32(j);
    let mut v = vslot(m, jt, 32, &[], &[]).1;
    m.wv(sp + 0x10, v);
    let jt = m.r32(j);
    let gp = m.r32(jt + 0xe8);
    let g = m.rf(gp + 0x88);
    let z = v[2];
    let rf = |m: &mut M, a: u32| m.rf(a);
    let f0 = match st {
        2 => rf(m, 0x002b_f8c8),
        3 => rf(m, 0x002b_f8d0),
        0x12 => rf(m, 0x002b_f8cc),
        1 | 7 | 9 | 0xd | 0xe | 0xf => {
            let f4 = match st {
                0xd => rf(m, 0x002b_f8dc),
                9 => rf(m, 0x002b_f8d4),
                0xe => rf(m, 0x002b_f8d8),
                _ => rf(m, 0x002b_f738),
            };
            let jt = m.r32(j);
            let hp = m.r32(jt + 0x218);
            let h = m.rf(hp + 8);
            let y = m.rf(jt + 0x238);
            let d = (g + g) * ((h - y) - f4);
            0.0f32.max(d).sqrt()
        }
        4 => {
            let c0 = rf(m, 0x002b_f8b0);
            let c1 = rf(m, 0x002b_f8b4);
            let c2 = rf(m, 0x002b_f8b8);
            let lo = rf(m, 0x002b_f8c0);
            let hi = rf(m, 0x002b_f8c4);
            let f3 = c0 + z * (c1 + z * c2);
            if f3 < lo {
                lo
            } else if hi < f3 {
                hi
            } else {
                f3
            }
        }
        6 => {
            let m2 = -2.0f32;
            let h738 = rf(m, 0x002b_f738);
            let mut f20 = (g * (h738 * m2)).sqrt();
            let f27 = -f20;
            let f24 = f27 / g;
            let jt = m.r32(j);
            let w = vslot(m, jt, 32, &[], &[]).1[2];
            let f14 = rf(m, 0x002b_f738);
            let h73c = rf(m, 0x002b_f73c);
            let f21 = f20.min(w);
            let f23 = (f21 - f20) / g;
            let f1 = g * ((h73c - f14) * m2);
            f20 = f23 * (f20 + g * (f23 * 0.5));
            let f13 = f1.sqrt();
            m.call(0x0011_d508, &[sp + 0x20], &[f4(g * 0.5), f4(f13), f4(f14)]);
            let r1 = m.rf(sp + 0x24) - f24;
            if 0.0 < f21 {
                let h = rf(m, 0x002b_f73c);
                (g * ((h - f20) * m2)).sqrt()
            } else {
                if f21 < f27 {
                    f20 = 0.0;
                }
                let t = ((f24 + f24) + r1) - f23;
                -f20 / t - g * (t * 0.5)
            }
        }
        _ => z,
    };
    v[2] = f0.max(z);
    m.wv(sp + 0x10, v);
    crate::engine::classes::q::q2mem::stack_pop(m, 0xb0);
    (0, v)
}

/// FUN_0011d508(out, a, b, c) SolveQuadratic: roots of a t^2 + b t + c (ascending), returns the count.
pub fn ee_sub_11d508<M: QMem + ?Sized>(m: &mut M, out: u32, a: f32, b: f32, c: f32) -> u32 {
    let d = b * b - a * (c * 4.0);
    if d < 0.0 {
        return 0;
    }
    let a2 = a + a;
    let h = d.sqrt() / a2;
    let q = b / a2;
    if h.abs() < f32::from_bits(0x38d1_b717) {
        m.wf(out, -q);
        return 1;
    }
    let (r1, r0) = (-q - h, -q + h);
    m.wf(out + 4, r1);
    m.wf(out, r0);
    if r1 < r0 {
        m.wf(out + 4, r0);
        m.wf(out, r1);
    }
    2
}

/// FUN_00221268(o, on): if +0x13c != on: +0x13c = on; FUN_00221298(o, 0).
pub fn ee_sub_221268<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) {
    if m.r32(a0 + 0x13c) != a1 {
        m.w32(a0 + 0x13c, a1);
        m.call(0x0022_1298, &[a0, 0], &[]);
    }
}

/// FUN_00187078(bits): DAT_002dfbc4 |= bits; FUN_001870d0().
pub fn ee_sub_187078<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let v = m.r32(0x002d_fbc4) | a0;
    m.w32(0x002d_fbc4, v);
    m.call(0x0018_70d0, &[], &[]);
}

/// FUN_001870a0(bits): DAT_002dfbc4 &= !bits; FUN_001870d0().
pub fn ee_sub_1870a0<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let v = m.r32(0x002d_fbc4) & !a0;
    m.w32(0x002d_fbc4, v);
    m.call(0x0018_70d0, &[], &[]);
}

/// FUN_001870d0(): pause-mode from the request bits DAT_002dfbc4 -> DAT_002dfbc0 (0x10/1 -> 5, 0xc -> 0,
/// 2 -> 4, else 7); when the mode lacks bit 2, DAT_002e0cbc = 0xffff.
pub fn ee_sub_1870d0<M: QMem + ?Sized>(m: &mut M) {
    let w = m.r32(0x002d_fbc4);
    if w & 0x10 != 0 {
        m.w32(0x002d_fbc0, 5);
    } else if w & 0xc != 0 {
        m.w32(0x002d_fbc0, 0);
    } else if w & 1 != 0 {
        m.w32(0x002d_fbc0, 5);
    } else if w & 2 != 0 {
        m.w32(0x002d_fbc0, 4);
    } else {
        m.w32(0x002d_fbc0, 7);
    }
    if m.r32(0x002d_fbc0) & 2 == 0 {
        m.w32(0x002e_0cbc, 0xffff);
    }
}
