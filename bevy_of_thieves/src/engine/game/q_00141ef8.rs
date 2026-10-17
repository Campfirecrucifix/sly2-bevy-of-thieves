//! WP-Q5: run_00141ef8 queue package (player-character glue reached through the VM field thunks
//! 0x10a848.. and the JT update): state-change requests on the +0x4c0 state machine, the +0x11b0 carry /
//! throw record, the +0x10f4 head-look rig (FUN_00142af8) and the episode-name key search
//! (FUN_00143410); ported 1:1 against `QMem` (out-of-package callees by address).
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};
use crate::engine::classes::q::q5util::{dist3, f4, pos};

pub const CLOCK: u32 = 0x002d_ded0;

/// FUN_00141ef8: resolve the three "%d"-numbered sub-objects (sprintf 0x2a4d50, lookup 0x2a4d58 under
/// slot 102's root) into +0x1180 + 0xc * i, then slot 104
pub fn ee_sub_141ef8<M: QMem + ?Sized>(m: &mut M, s: u32) {
    let sp = stack_push(m, 0x20);
    for i in 0..3u32 {
        m.call(0x0021_ecd8, &[sp, 0x002a_4d50, i], &[]);
        let root = vslot(m, s, 102, &[], &[]).0;
        m.call(0x001d_8a38, &[sp + 0x10, root, 0x002a_4d58, sp, 0], &[]);
        let v = m.r32(sp + 0x14);
        m.w32(s + 0x1180 + 0xc * i, v);
        m.call(0x0012_9ec8, &[sp + 0x10], &[]);
    }
    stack_pop(m, 0x20);
    vslot(m, s, 104, &[], &[]);
}

/// request state `k` (FUN_001928f8 on +0x4c0 with `p`) or, with p == 0, drop back to state 0 (slot 87)
/// when currently in `k`
fn req<M: QMem + ?Sized>(m: &mut M, s: u32, k: u32, p: u32) -> bool {
    if p == 0 {
        if m.r32(s + 0x4c8) == k {
            vslot(m, s, 87, &[0], &[]);
        }
        false
    } else {
        m.call(0x0019_28f8, &[s + 0x4c0, k, p, 0], &[]);
        true
    }
}

/// FUN_00142140: state 5 request / release
pub fn ee_sub_142140<M: QMem + ?Sized>(m: &mut M, s: u32, p: u32) {
    req(m, s, 5, p);
}

/// FUN_001421a0: release state 5, then (re)attach (FUN_001cd098) or detach (FUN_001cd2c0) +0x900
pub fn ee_sub_1421a0<M: QMem + ?Sized>(m: &mut M, s: u32, p: u32) {
    ee_sub_142140(m, s, 0);
    if p == 0 {
        m.call(0x001c_d2c0, &[s + 0x900], &[]);
    } else {
        m.call(0x001c_d098, &[s + 0x900, p], &[]);
    }
}

/// FUN_001421f8: state 5 request also binding +0x900 (FUN_001cd308)
pub fn ee_sub_1421f8<M: QMem + ?Sized>(m: &mut M, s: u32, p: u32) {
    if req(m, s, 5, p) {
        m.call(0x001c_d308, &[s + 0x900, p], &[]);
    }
}

/// FUN_00142278: start carrying `obj` from `from`: FUN_001f7c68(obj, 0.25), +0x11c0 = obj, +0x1290 =
/// from's position, reset +0x11b0 (FUN_0021fb70), state 7 (slot 87) and record state 7 (FUN_002206c8)
pub fn ee_sub_142278<M: QMem + ?Sized>(m: &mut M, s: u32, obj: u32, from: u32) {
    m.call(0x001f_7c68, &[obj], &[f4(0.25)]);
    m.w32(s + 0x11c0, obj);
    let p = pos(m, from);
    m.wv(s + 0x1290, p);
    m.call(0x0021_fb70, &[s + 0x11b0, 0], &[]);
    vslot(m, s, 87, &[7], &[]);
    m.call(0x0022_06c8, &[s + 0x11b0, 7], &[]);
}

/// FUN_00142310: throw parameters: f12..f15 -> +0x11c4 / +0x11c8 / +0x11d4 / +0x11d8, 8 bytes at `p` ->
/// +0x11cc, quad `a` -> +0x1280, quad `b` -> +0x11e0
pub fn ee_sub_142310<M: QMem + ?Sized>(m: &mut M, s: u32, p: u32, a: V4, b: V4, f: [f32; 4]) {
    m.wf(s + 0x11c4, f[0]);
    m.wf(s + 0x11c8, f[1]);
    let x = m.r64(p);
    m.w32(s + 0x11cc, x as u32);
    m.w32(s + 0x11d0, (x >> 32) as u32);
    m.wv(s + 0x11e0, b);
    m.wf(s + 0x11d4, f[2]);
    m.wf(s + 0x11d8, f[3]);
    m.wv(s + 0x1280, a);
}

/// FUN_00142340: state 9 request / release
pub fn ee_sub_142340<M: QMem + ?Sized>(m: &mut M, s: u32, p: u32) {
    req(m, s, 9, p);
}

/// FUN_001423a0: in state 5 with a +0x4d4 grab: 0 when the grabbed object (+0x934) is more than 5 from
/// its attach point (slot 31 frame through FUN_002309e8), or for kind-1 grabs (+0x9a4) before +0x994's
/// +0x1c time; else 1
pub fn ee_sub_1423a0<M: QMem + ?Sized>(m: &mut M, s: u32) -> u32 {
    if m.r32(s + 0x4c8) != 5 || m.r32(s + 0x4d4) == 0 {
        return 1;
    }
    let sp = stack_push(m, 0x90);
    let g = m.r32(s + 0x934);
    let r = vslot(m, g, 31, &[sp], &[]).0;
    m.call(0x0023_09e8, &[r, 0, sp + 0x40, 0], &[f4(0.0)]);
    let v = m.rv(sp + 0x30);
    let mm = m.rm(sp + 0x40);
    let mut q = [0.0f32; 4];
    for c in 0..3 {
        q[c] = mm[0][c] * v[0] + mm[1][c] * v[1] + mm[2][c] * v[2] + mm[3][c];
    }
    stack_pop(m, 0x90);
    let g = m.r32(s + 0x934);
    let gp = pos(m, g);
    if 5.0 < dist3(gp, q) {
        return 0;
    }
    if m.r32(s + 0x9a4) != 1 {
        return 1;
    }
    let t = m.r32(s + 0x994);
    if m.rf(CLOCK) < m.rf(t + 0x1c) {
        return 0;
    }
    1
}

/// FUN_001424c0: no carry record (+0x11b4 == -1)
pub fn ee_sub_1424c0<M: QMem + ?Sized>(m: &mut M, s: u32) -> bool {
    m.r32(s + 0x11b4) == 0xffff_ffff
}

/// FUN_001424d0: not in state 9
pub fn ee_sub_1424d0<M: QMem + ?Sized>(m: &mut M, s: u32) -> bool {
    m.r32(s + 0x4c8) != 9
}

/// FUN_001424e0: state 0xc request with target (0 = +0x9c8) -> +0x9cc and +0x1318 = `x`
pub fn ee_sub_1424e0<M: QMem + ?Sized>(m: &mut M, s: u32, mut t: u32, p: u32, x: u32) {
    if t == 0 {
        t = m.r32(s + 0x9c8);
    }
    m.w32(s + 0x9cc, t);
    m.call(0x0019_28f8, &[s + 0x4c0, 0xc, p, 0], &[]);
    m.w32(s + 0x1318, x);
}

/// FUN_00142530: not in state 0xc
pub fn ee_sub_142530<M: QMem + ?Sized>(m: &mut M, s: u32) -> bool {
    m.r32(s + 0x4c8) != 0xc
}

/// FUN_00142540: clear the brain (+0x26c, FUN_001eecd0); with `t` push behaviour 0xc2 {+0x30 t, +0x40 q}
pub fn ee_sub_142540<M: QMem + ?Sized>(m: &mut M, s: u32, t: u32, q: V4) {
    m.call(0x001e_ecd0, &[s + 0x26c], &[]);
    if t != 0 {
        let b = m.call(0x0020_5438, &[s, 0xc2], &[]).0;
        m.w32(b + 0x30, t);
        m.wv(b + 0x40, q);
        vslot(m, b, 7, &[], &[]);
    }
}

/// FUN_001427c8: look-at target `t` with priority `pri` (+0x10f8 / +0x10fc); a new target while the look
/// timer (+0x114c) is armed starts the glance (+0x1148 = 1) and clears the hold (+0x1100)
pub fn ee_sub_1427c8<M: QMem + ?Sized>(m: &mut M, s: u32, pri: u32, t: u32) {
    if t == m.r32(s + 0x10f8) && pri == m.r32(s + 0x10fc) {
        return;
    }
    m.w32(s + 0x10fc, pri);
    m.w32(s + 0x10f8, t);
    if t != 0 && m.rf(s + 0x114c) != m.rf(0x002a_4d60) {
        m.w32(s + 0x1148, 1);
        ee_sub_142860(m, s);
    }
}

/// FUN_00142830: set the target and hold it (+0x1100 = 1)
pub fn ee_sub_142830<M: QMem + ?Sized>(m: &mut M, s: u32, pri: u32, t: u32) {
    ee_sub_1427c8(m, s, pri, t);
    m.w32(s + 0x1100, 1);
}

/// FUN_00142860: release the hold
pub fn ee_sub_142860<M: QMem + ?Sized>(m: &mut M, s: u32) {
    m.w32(s + 0x1100, 0);
}

/// FUN_00142af8: head-look rig update (+0x10f4 rig, +0x1120 / +0x1128 yaw / pitch limits, +0x1130 /
/// +0x1134 roll range, +0x1150 rest frame). Picks the best look target (current +0x10dc, the 0x3e1474
/// list by priority +0xf4 / range +0xe8 / cone +0xec, the 0x3e1b20 extra) unless held / suppressed,
/// then aims the rig at it (or at the fixed +0x1110 / +0x1114 / +0x1118 pose when +0x1104) through
/// FUN_0022fd38 and FUN_0012d108; disables it (FUN_0012d020) when there is nothing to look at.
pub fn ee_sub_142af8<M: QMem + ?Sized>(m: &mut M, s: u32) {
    if m.r32(s + 0x10f4) == 0 {
        return;
    }
    let mut out = m.rf(s + 0x1120) < m.rf(s + 0x1124) || m.rf(s + 0x1128) < m.rf(s + 0x112c);
    if m.r32(s + 0xf04) != 0xffff_ffff {
        out = false;
    }
    if m.rf(s + 0x114c) < m.rf(CLOCK) {
        let g = m.r32(s + 0x1148);
        if g != 0 || out {
            m.w32(s + 0x1148, (g == 0) as u32);
            m.call(0x0014_2880, &[s], &[]);
        }
    }
    let f = stack_push(m, 0x140);
    if m.r32(s + 0x1100) == 0 {
        if m.r32(0x002d_d640) != 0 {
            ee_sub_1427c8(m, s, 0xffff_ffff, 0);
        } else {
            let mut best = m.r32(s + 0x10dc);
            let mut pri = m.r32(s + 0x10e4) as i32;
            if best != 0 {
                m.call(0x0014_28d8, &[s, best, s + 0x10f8, f + 0x120, 0], &[]);
            } else {
                let x = m.r32(0x002a_4d68);
                m.w32(f + 0x120, x);
            }
            let mut l = m.r32(0x003e_1474);
            while l != 0 {
                if (m.r32(l + 0xf4) as i32) >= pri {
                    m.call(0x0014_28d8, &[s, l, s + 0x10f8, f + 0x124, f + 0x128], &[]);
                    if !(m.rf(l + 0xe8) < m.rf(f + 0x124)) {
                        let a = m.rf(f + 0x128);
                        if m.call(0x0011_d910, &[l + 0xec], &[f4(a)]).0 != 0 {
                            let lp = m.r32(l + 0xf4) as i32;
                            if pri < lp || (lp == pri && m.rf(f + 0x124) < m.rf(f + 0x120)) {
                                best = l;
                                let d = m.r32(f + 0x124);
                                m.w32(f + 0x120, d);
                                pri = m.r32(best + 0xf4) as i32;
                            }
                        }
                    }
                }
                l = m.r32(l + 0xe0);
            }
            if (m.r32(0x003e_1b08) as i32) > 0 && pri < 3 {
                let x = m.r32(0x003e_1b20);
                if x != 0 {
                    m.call(0x0014_28d8, &[s, x, s + 0x10f8, f + 0x12c, f + 0x130], &[]);
                    if pri < 2 || (pri == 2 && m.rf(f + 0x12c) < m.rf(f + 0x120)) {
                        best = x;
                        pri = 2;
                        let d = m.r32(f + 0x12c);
                        m.w32(f + 0x120, d);
                    }
                }
            }
            ee_sub_1427c8(m, s, pri as u32, best);
        }
    }
    let rig = m.r32(s + 0x10f4);
    let (yaw, pitch, roll);
    if m.r32(s + 0x1104) != 0 {
        m.call(0x0012_cfe0, &[rig], &[]);
        yaw = m.rf(s + 0x1110);
        pitch = m.rf(s + 0x1114);
        roll = m.rf(s + 0x1118);
    } else {
        if m.r32(s + 0x10f8) == 0 || m.r32(s + 0x1148) == 0 || !out {
            m.call(0x0012_d020, &[rig], &[]);
            stack_pop(m, 0x140);
            return;
        }
        m.call(0x0012_cfe0, &[rig], &[]);
        let t = m.r32(s + 0x10f8);
        let tp = pos(m, t);
        let d = m.call(0x001a_6f00, &[0, s], &[tp]).1;
        m.call(0x0011_e958, &[f + 0x134, f + 0x138], &[d]);
        let a = m.rf(f + 0x134);
        let a = m.call(0x0011_d978, &[s + 0x1120], &[f4(a)]).1[0];
        m.wf(f + 0x134, a);
        let b = m.rf(f + 0x138);
        let b = m.call(0x0011_d978, &[s + 0x1128], &[f4(b)]).1[0];
        m.wf(f + 0x138, b);
        let (lo, hi) = (m.rf(s + 0x1120), m.rf(s + 0x1124));
        let u = if hi == lo { 0.5 } else { (a - lo) / (hi - lo) };
        let w = u * u.abs();
        roll = (1.0 - w) * m.rf(s + 0x1130) + w * m.rf(s + 0x1134);
        yaw = a;
        pitch = b;
    }
    // R (0x90) = yaw / pitch frame, A (0xc0) = roll about DAT_002a2da0, C = A * R
    m.call(0x0011_b720, &[f + 0x90], &[f4(yaw), f4(pitch)]);
    let axis = m.rv(0x002a_2da0);
    m.call(0x0011_b500, &[f + 0xc0], &[axis, f4(roll)]);
    let r = m.rm(f + 0x90);
    let a = m.rm(f + 0xc0);
    let c = mul3(&[a[0], a[1], a[2]], &[r[0], r[1], r[2]]);
    for i in 0..3u32 {
        m.wv(f + 0x10 * i, c[i as usize]);
        m.wv(f + 0x60 + 0x10 * i, c[i as usize]);
        m.wv(f + 0xf0 + 0x10 * i, c[i as usize]);
    }
    let rig = m.r32(s + 0x10f4);
    let x = m.r32(rig + 8);
    let bone = m.r32(s + 0x60);
    m.call(0x0022_fd38, &[f + 0x60, 0, bone, x + 0x40], &[]);
    let d = [m.rv(f + 0x60), m.rv(f + 0x70), m.rv(f + 0x80)];
    for i in 0..3u32 {
        m.wv(f + 0x30 + 0x10 * i, d[i as usize]);
    }
    m.call(0x0011_b340, &[f + 0x90, s + 0x1150], &[]);
    let n = [m.rv(f + 0x90), m.rv(f + 0xa0), m.rv(f + 0xb0)];
    let e = mul3(&n, &d);
    for i in 0..3u32 {
        let v = e[i as usize];
        m.wv(f + 0x30 + 0x10 * i, v);
        m.wv(f + 0xc0 + 0x10 * i, v);
        m.wv(f + 0x60 + 0x10 * i, v);
    }
    if c[0][0] + e[0][0] < 0.0 && c[0][1] * e[0][1] < 0.0 {
        for i in 0..3u32 {
            let v = m.rv(s + 0x1150 + 0x10 * i);
            m.wv(f + 0x60 + 0x10 * i, v);
        }
    } else {
        let sr = [m.rv(s + 0x1150), m.rv(s + 0x1160), m.rv(s + 0x1170)];
        let g = mul3(&sr, &c);
        for i in 0..3u32 {
            let v = g[i as usize];
            m.wv(f + 0x60 + 0x10 * i, v);
            m.wv(f + 0xc0 + 0x10 * i, v);
            m.wv(f + 0x90 + 0x10 * i, v);
        }
    }
    let rig = m.r32(s + 0x10f4);
    let x = m.r32(rig + 8);
    let k = m.r32(x + 0xa0);
    let bone = m.r32(s + 0x60);
    m.call(0x0022_fd38, &[f + 0x90, bone, k, f + 0x60], &[]);
    for i in 0..3u32 {
        let v = m.rv(f + 0x90 + 0x10 * i);
        m.wv(f + 0x60 + 0x10 * i, v);
    }
    let q = m.call(0x0011_bff0, &[f + 0x60], &[]).1;
    stack_pop(m, 0x140);
    let rig = m.r32(s + 0x10f4);
    m.call(0x0012_d108, &[rig], &[q]);
}

/// rows of `a` through basis `b`: out[i].xyz = b0 * a[i].x + b1 * a[i].y + b2 * a[i].z, out[i].w = a[i].w
fn mul3(a: &[V4; 3], b: &[V4; 3]) -> [V4; 3] {
    let mut o = *a;
    for i in 0..3 {
        for c in 0..3 {
            o[i][c] = b[0][c] * a[i][0] + b[1][c] * a[i][1] + b[2][c] * a[i][2];
        }
    }
    o
}

/// FUN_00143090: copy the three 8-byte limit pairs to +0x1120 / +0x1128 / +0x1130
pub fn ee_sub_143090<M: QMem + ?Sized>(m: &mut M, s: u32, a: u32, b: u32, c: u32) {
    for (o, p) in [(0x1120, a), (0x1128, b), (0x1130, c)] {
        let x = m.r64(p);
        m.w32(s + o, x as u32);
        m.w32(s + o + 4, (x >> 32) as u32);
    }
}

/// FUN_001431d0: when the +0x10dc object's behaviour 0xb1 targets `s`, clear its +0x60
pub fn ee_sub_1431d0<M: QMem + ?Sized>(m: &mut M, s: u32) {
    let o = m.r32(s + 0x10dc);
    if o == 0 {
        return;
    }
    let b = m.call(0x001e_eee0, &[o + 0x26c, 0xb1], &[]).0;
    if b != 0 && m.r32(b + 0x5c) == s {
        m.w32(b + 0x60, 0);
    }
}

/// FUN_00143410: find the key path whose 7th component (0..14) entry (FUN_001d7ed8 / FUN_00129658)
/// has id `id`: walk the 0x3e0ea8 key tree (keys 0x2a4d88 / 0x2a4d90 / 0x2a4d98 / 0x2a4da0 fixed,
/// levels 1 and 3 enumerated through the 10-byte node table 0x3e0eac); return a frame-pool copy
/// (FUN_001ae318) of {count, keys} or 0
pub fn ee_sub_143410<M: QMem + ?Sized>(m: &mut M, id: u32) -> u32 {
    const TREE: u32 = 0x003e_0ea8;
    let sp = stack_push(m, 0x80);
    m.w32(sp + 0x70, id);
    m.w32(sp + 0x10, 0);
    m.w32(sp + 0x14, sp);
    for (o, k) in [(0u32, 0x002a_4d88u32), (4, 0x002a_4d90), (8, 0x002a_4d98), (0xa, 0x002a_4da0)] {
        let v = m.call(0x001d_8720, &[0x003e_0eb0, k], &[]).0;
        m.w16(sp + o, v as u16);
    }
    m.w32(sp + 0x10, 1);
    m.call(0x001d_7da0, &[TREE, sp + 0x10, sp + 0x20], &[]);
    let r = m.r32(sp + 0x20);
    let mut u1 = m.r16(r + 6) as u32;
    let mut res = 0;
    'outer: while u1 != 0 {
        let n1 = m.r32(0x003e_0eac) + u1 * 10;
        m.w32(sp + 0x10, 3);
        let k = m.r16(n1);
        m.w16(sp + 2, k);
        if m.call(0x001d_7da0, &[TREE, sp + 0x10, sp + 0x30], &[]).0 != 0 {
            let r = m.r32(sp + 0x30);
            let mut u2 = m.r16(r + 6) as u32;
            while u2 != 0 {
                let n2 = m.r32(0x003e_0eac) + u2 * 10;
                let k = m.r16(n2);
                m.w32(sp + 0x40, 0xffff_ffff);
                m.w16(sp + 6, k);
                for i in 0..0xfu32 {
                    m.w16(sp + 0xc, i as u16);
                    m.w32(sp + 0x10, 7);
                    m.call(0x001d_7ed8, &[sp + 0x50, TREE, sp + 0x10, 0x003d_4a60], &[]);
                    m.call(0x0012_9658, &[sp + 0x40, sp + 0x50], &[]);
                    m.call(0x0012_9ec8, &[sp + 0x50], &[]);
                    if m.r32(sp + 0x44) == m.r32(sp + 0x70) {
                        m.w32(sp + 0x60, 8);
                        m.w32(sp + 0x64, 0x10);
                        let h = m.call(0x001a_e318, &[sp + 0x60], &[]).0;
                        m.call(0x001a_e5a0, &[h, sp + 0x10, 8], &[]);
                        let n = m.r32(sp + 0x10) << 1;
                        m.w32(sp + 0x50, n);
                        m.w32(sp + 0x54, 0x10);
                        let ks = m.call(0x001a_e318, &[sp + 0x50], &[]).0;
                        m.w32(h + 4, ks);
                        let src = m.r32(sp + 0x14);
                        let n = m.r32(sp + 0x10) << 1;
                        m.call(0x001a_e5a0, &[ks, src, n], &[]);
                        m.call(0x0012_9ec8, &[sp + 0x40], &[]);
                        res = h;
                        break 'outer;
                    }
                }
                m.call(0x0012_9ec8, &[sp + 0x40], &[]);
                u2 = m.r16(n2 + 8) as u32;
            }
        }
        u1 = m.r16(n1 + 8) as u32;
    }
    stack_pop(m, 0x80);
    res
}
