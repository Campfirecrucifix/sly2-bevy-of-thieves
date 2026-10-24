//! WIRE-W1d: 1:1 queue ports (against `QMem`) of the k22 sequence (cutscene/script) interpreter: the
//! per-op executor FUN_001e1e30, the wait-condition test FUN_001e3890, the seq close FUN_001e5628 and the
//! focus setter FUN_001e4eb8. Ported from the EE asm (tools/eedis.py); jump tables read from the ELF.
//! QMem convention: integer / pointer args in `a`, quadword (128-bit GPR) and float args in `v`.
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

const T_NOW: u32 = 0x002d_ded0; // g_clock t
const T_REAL: u32 = 0x002d_df0c; // unscaled clock t
const GS: u32 = 0x003d_d490; // game state / world

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

/// vtable slot index of the fn word at byte offset `off` (fn at +8n+0xc)
const fn sl(off: u32) -> u32 {
    (off - 0xc) / 8
}

/// seq clock: `*(seq->owner(+0x14)+0x40)` (deref when even) & 0x100 -> unscaled clock, else g_clock t
fn seq_t<M: QMem + ?Sized>(m: &mut M, s: u32) -> f32 {
    let o = m.r32(s + 0x14);
    let mut a = m.r32(o + 0x40);
    if (a ^ 1) & 1 != 0 {
        a = m.r32(a);
    }
    if a & 0x100 != 0 {
        m.rf(T_REAL)
    } else {
        m.rf(T_NOW)
    }
}

/// |(a - b).xy|
fn dist_xy(a: V4, b: V4) -> f32 {
    let (x, y) = (a[0] - b[0], a[1] - b[1]);
    (x * x + y * y).sqrt()
}

/// FUN_001e4eb8(seq, a1, focus): seq[+0x40] = a1 (when different); when focus != seq[+0x44]: old focus
/// (if any) gets FUN_00143de8(old, 0), seq[+0x44] = focus, new focus (if any) FUN_00143de8(focus, seq).
pub fn ee_sub_1e4eb8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32) {
    if a1 != m.r32(a0 + 0x40) {
        m.w32(a0 + 0x40, a1);
    }
    let old = m.r32(a0 + 0x44);
    if a2 == old {
        return;
    }
    if old != 0 {
        m.call(0x0014_3de8, &[old, 0], &[]);
    }
    m.w32(a0 + 0x44, a2);
    if a2 != 0 {
        m.call(0x0014_3de8, &[a2, a0], &[]);
    }
}

/// FUN_001e5628(seq): close the sequence unless FUN_001e5778(seq) says it is still busy.
pub fn ee_sub_1e5628<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    if m.call(0x001e_5778, &[a0], &[]).0 != 0 {
        return;
    }
    vslot(m, a0, 0, &[], &[]);
    let o = m.r32(a0 + 0x14);
    m.call(0x0011_a2e8, &[o + 0x5c, a0], &[]);
    for f in [0x001e_5370, 0x001e_53d8, 0x001e_5410, 0x001e_5468, 0x001e_54b0, 0x001e_54e8, 0x001e_5570] {
        m.call(f, &[a0], &[]);
    }
    m.call(0x0014_d1d0, &[T_NOW, 6], &[f4(1.0)]);
    m.call(0x001e_4eb8, &[a0, 0, 0], &[]);
    m.call(0x001e_4f20, &[a0, 0], &[]);
    m.call(0x001e_5000, &[a0, 0], &[]);
    m.call(0x001e_50e0, &[a0, 0, 0], &[]);
    if m.r32(a0 + 0xec) != 0 {
        m.call(0x001f_b098, &[], &[f4(0.0)]);
        m.call(0x0025_8040, &[a0 + 0xec, 0], &[]);
    }
    if m.r32(a0 + 0x9c) != 0 {
        m.call(0x001e_5a98, &[a0, 1], &[]);
    }
    vslot(m, a0, 1, &[], &[]);
    vslot(m, a0, 1, &[], &[]);
}

/// FUN_001e3890(seq, op): is the wait condition of script op `op` satisfied (1) or still pending (0)?
/// Jump table 0x2a7d90 over op[0] - 3 (0x58 entries); unlisted ops are satisfied.
pub fn ee_sub_1e3890<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32) -> u32 {
    let s = a0;
    let p = a1;
    let buf = stack_push(m, 0x20);
    let r = sub_1e3890_body(m, s, p, buf);
    stack_pop(m, 0x20);
    r
}

fn sub_1e3890_body<M: QMem + ?Sized>(m: &mut M, s: u32, p: u32, buf: u32) -> u32 {
    let op = m.r32(p);
    let b = |c: bool| c as u32;
    match op {
        3 => {
            let mut i = 0i32;
            let mut q = p + 0x14;
            while i < m.r32(p + 0x10) as i32 {
                let o = m.r32(s + 0x14);
                let e = m.r32(q);
                let v = m.call(0x001e_0180, &[o, e, s], &[]).0;
                q += 4;
                if v != 0 {
                    return 0;
                }
                i += 1;
            }
            1
        }
        4 => {
            let o = m.r32(s + 0x14);
            let id = m.r32(p + 0x10);
            let r = m.call(0x001e_00e8, &[o, id], &[]).0;
            if r != 0 && m.r32(p + 0x14) != 0 {
                let o = m.r32(s + 0x14);
                let id = m.r32(p + 0x10);
                m.call(0x001e_0070, &[o, id], &[]);
            }
            r
        }
        0xa => {
            let a = m.r32(s + 0x54);
            if a != 0 {
                if m.call(0x0014_4038, &[a], &[]).0 != 0 {
                    return 0;
                }
                m.w32(s + 0x54, 0);
                return 1;
            }
            let a = m.r32(s + 0x4c);
            if a != 0 && m.call(0x0013_1968, &[a], &[]).0 != 0 {
                let o = m.r32(s + 0x4c);
                vslot(m, o, 1, &[], &[]);
                m.w32(s + 0x4c, 0);
            }
            b(m.r32(s + 0x4c) == 0)
        }
        0xb => {
            let a = m.r32(s + 0x50);
            if a == 0 {
                return 1;
            }
            if m.r32(a + 0x74) == 0 {
                m.w32(s + 0x50, 0);
            }
            b(m.r32(s + 0x50) == 0)
        }
        0xd => {
            let a = m.r32(s + 0x58);
            if a != 0 && (m.call(0x001f_0638, &[a], &[]).0 != 0 || m.r32(s + 0x5c) != 0) {
                m.call(0x001e_5410, &[s], &[]);
            }
            b(m.r32(s + 0x58) == 0)
        }
        0xe => {
            let a = m.r32(s + 0x64);
            if a != 0 && m.call(0x0021_21c8, &[a], &[]).0 != 0 {
                m.call(0x001e_5468, &[s], &[]);
            }
            b(m.r32(s + 0x64) == 0)
        }
        0x1a => {
            let a = m.r32(s + 0x44);
            if a == 0 {
                return 1;
            }
            m.call(0x0019_e110, &[a + 0xf50, buf], &[]);
            let f12 = m.rf(s + 0x7c);
            let f0 = m.call(0x0011_d948, &[buf], &[f4(f12)]).1[0];
            let a = m.r32(s + 0x44);
            let lim = m.rf(p + 0x14);
            let d = (m.rf(a + 0xf60) - f0).abs();
            b(d < lim)
        }
        0x1b => {
            let a = m.r32(s + 0x24);
            if a == 0 || m.call(0x0013_2b10, &[a, 0x34], &[]).0 == 0 {
                return 1;
            }
            let a = m.r32(s + 0x24);
            if m.r32(a + 0x2dc).wrapping_sub(0xb) < 3 {
                return 0;
            }
            if m.r32(a + 0x4cc) as i32 == -1 {
                return 1;
            }
            let st = m.r32(a + 0x4c8);
            if st != 0 && st != 0xd {
                return 1;
            }
            vslot(m, a, sl(0x2c4), &[0], &[]);
            1
        }
        0x1c => {
            let a = m.r32(s + 0x24);
            if a == 0 || m.call(0x0013_2b10, &[a, 0x35], &[]).0 == 0 {
                return 1;
            }
            let k = m.r32(p + 0x14) as i32;
            let a = m.r32(s + 0x24);
            match k {
                4 => b(m.r32(a + 0x18c8) != 2),
                2 => b(m.r32(a + 0x18d8) as i32 == -1),
                8 => b(m.r32(a + 0x18c8) != 9),
                _ => 1,
            }
        }
        0x1d | 0x21 | 0x1e | 0x1f | 0x22 => {
            let a = m.r32(s + 0x44);
            if a == 0 {
                return 1;
            }
            let f = match op {
                0x1e => 0x0014_24c0,
                0x1f => 0x0014_24d0,
                0x22 => 0x0014_2530,
                _ => 0x0014_23a0,
            };
            m.call(f, &[a], &[]).0
        }
        0x23 | 0x32 | 0x33 | 0x53 => {
            let t = seq_t(m, s);
            let t0 = m.rf(s + 0x34);
            let lim = match op {
                0x32 => m.rf(0x002b_d850),
                0x53 => m.rf(p + 0x14),
                _ => m.rf(p + 0x10),
            };
            b(lim < t - t0)
        }
        0x24 => {
            let a = m.r32(s + 0x44);
            if a == 0 {
                return 1;
            }
            let id = m.r32(p + 0x10);
            let e = m.call(0x001e_1db8, &[s, id], &[]).0;
            let mut r = 1;
            if e != 0 {
                m.call(0x001e_4d40, &[s, e, buf + 0x10, 0], &[]);
                let pos = m.call(0x001a_62e8, &[a], &[]).1;
                let tgt = m.rv(buf + 0x10);
                if !(dist_xy(pos, tgt) < m.rf(p + 0x18)) {
                    r = 0;
                }
                m.call(0x0014_48f0, &[a + 0xfd0], &[tgt]);
            }
            if r == 0 {
                return 0;
            }
            if m.r32(p + 0x20) as i32 != -1 {
                m.call(0x0014_3ff8, &[a], &[]);
            }
            r
        }
        0x27 | 0x28 => {
            let a = m.r32(s + 0x24);
            if a == 0 || m.call(0x0013_2b10, &[a, 0x34], &[]).0 == 0 {
                return 1;
            }
            let a = m.r32(s + 0x24);
            let (kind, f) = if op == 0x27 { (0xa8, 0x0017_8d00) } else { (0xc1, 0x0014_4ee8) };
            let x = m.call(0x0020_5458, &[a, kind], &[]).0;
            let r = if x == 0 { 1 } else { b(m.call(f, &[x], &[]).0 != 0) };
            if r == 0 {
                return 0;
            }
            m.call(0x001e_ecd0, &[a + 0x26c], &[]);
            if m.r32(p + 0x24) as i32 != -1 {
                m.call(0x0014_3ff8, &[a], &[]);
            }
            r
        }
        0x29 => {
            let a = m.r32(s + 0x44);
            if a == 0 || m.r32(a + 0x4cc) as i32 == -1 {
                return 1;
            }
            if m.r32(a + 0x4c8) != 8 {
                return 1;
            }
            let id = m.r32(p + 0x10);
            let e = m.call(0x001e_1db8, &[s, id], &[]).0;
            if e == 0 {
                return 1;
            }
            m.call(0x001e_4d40, &[s, e, buf, 0], &[]);
            let pos = m.call(0x001a_62e8, &[a], &[]).1;
            let tgt = m.rv(buf);
            b(dist_xy(pos, tgt) < m.rf(p + 0x18))
        }
        0x2a => {
            let a = m.r32(s + 0x44);
            if a == 0 {
                return 1;
            }
            b(m.r32(a + 0x2dc) as i32 == -1)
        }
        0x2b => {
            let a = m.r32(s + 0x24);
            if a == 0 || m.call(0x0013_2b10, &[a, 0x34], &[]).0 == 0 {
                return 1;
            }
            let a = m.r32(s + 0x24);
            b(m.r32(a + 0x2dc) as i32 == -1)
        }
        0x2c | 0x2d => {
            let a = m.r32(s + 0x60);
            if a != 0 {
                let bit = (m.r32(a + 0x2c) >> 28) & 1;
                if m.r32(p) == 0x2c {
                    if bit != 0 && m.rf(a + 0x12c) == 1.0 {
                        m.w32(s + 0x60, 0);
                    }
                } else if bit == 0 {
                    m.w32(s + 0x60, 0);
                }
            }
            b(m.r32(s + 0x60) == 0)
        }
        0x39 => {
            let r = b(m.rf(s + 0x80) <= m.rf(T_NOW));
            if r == 0 {
                return 0;
            }
            if m.r32(p + 0x40) == 0 {
                m.w32(s + 0x9c, 0);
            } else {
                let k = m.r32(p + 0x44);
                m.call(0x001e_5a98, &[s, k], &[]);
            }
            r
        }
        0x3b => {
            let a = m.r32(s + 0xa0);
            if a == 0 {
                return 1;
            }
            b(m.r32(a + 0xf28) == 1)
        }
        0x3c => {
            let a = m.r32(s + 0xa0);
            if a == 0 {
                return 1;
            }
            b(m.r32(a + 0xf28) as i32 == -1)
        }
        0x3d => m.call(0x0015_0f58, &[0x002d_e460], &[]).0,
        0x3e => m.call(0x0015_1018, &[0x002d_e460], &[]).0,
        0x40 => m.call(0x0015_10d8, &[0x002d_e460], &[]).0,
        0x42 => {
            let r;
            if m.r32(p + 0x14) as i32 == -1 {
                let id = m.r32(p + 0x10);
                r = m.call(0x0018_7d80, &[0x002d_fc00, id], &[]).0;
            } else {
                let mut o = m.r32(0x003e_133c);
                let mut found = false;
                while o != 0 {
                    let a = m.r32(o + 0x7c);
                    let k = m.r32(p + 0x14);
                    if m.call(0x001a_6290, &[a, k], &[]).0 != 0 {
                        found = true;
                        break;
                    }
                    o = m.r32(o + 0x84);
                }
                if found {
                    let id = m.r32(p + 0x10);
                    if m.call(0x001c_4700, &[o, id], &[]).0 == 0 {
                        return 0;
                    }
                }
                r = 1;
                let id = m.r32(p + 0x10);
                m.call(0x0018_7de0, &[0x002d_fc00, id], &[]);
            }
            if r != 0 {
                m.w32(0x0044_35f8, 0);
            }
            r
        }
        0x43 => {
            let a = m.r32(s + 0x24);
            if a == 0 || m.call(0x0013_2b10, &[a, 0x34], &[]).0 == 0 {
                return 1;
            }
            let a = m.r32(s + 0x24);
            let v = vslot(m, a, sl(0x2cc), &[], &[]).0;
            b(v == m.r32(p + 0x10))
        }
        0x44 => b(m.call(0x001d_9880, &[0x003d_a530], &[]).0 == 0),
        0x45 => {
            let id = m.r32(p + 0x10);
            b(m.call(0x001f_e848, &[0x003d_d040, id], &[]).0 == 0)
        }
        0x52 => {
            let o = m.r32(s + 0x68);
            let id = if o != 0 { m.r32(o + 0x10) } else { u32::MAX };
            if m.call(0x001f_e848, &[0x003d_d040, id], &[]).0 == 0 {
                return 1;
            }
            if m.rf(p + 0x10) < 0.0 {
                return 0;
            }
            let f0 = m.call(0x001f_e888, &[0x003d_d040], &[]).1[0];
            b(0.0 <= f0 - m.rf(p + 0x10))
        }
        0x4a | 0x4e => {
            let f0 = m.call(0x001f_e3f8, &[0x003d_d040], &[]).1[0];
            b(m.rf(p + 0x10) <= f0)
        }
        0x4f => {
            let a = m.r32(s + 0x24);
            if m.call(0x0013_2b10, &[a, 0x36], &[]).0 == 0 {
                return 0;
            }
            let a = m.r32(s + 0x24);
            if m.r32(a + 0x13d0) != 6 {
                return 1;
            }
            if m.r32(p + 0x1c) as i32 == -1 {
                return 0;
            }
            let x = m.r32(a + 0x1034);
            let v = m.call(0x001f_07e8, &[x], &[]).0;
            b(v == m.r32(p + 0x1c))
        }
        0x50 | 0x51 => {
            let a = m.r32(s + 0x44);
            if a == 0 {
                return 1;
            }
            let six = m.r32(a + 0x13d0) == 6;
            b(if op == 0x50 { six } else { !six })
        }
        0x55 => {
            let a1 = m.r32(s + 0xdc);
            if a1 != 0 {
                let a0 = m.r32(s + 0xd8);
                if m.call(0x0017_1da8, &[a0, a1], &[]).0 == 0 {
                    m.w32(s + 0xd8, 0);
                    m.w32(s + 0xdc, 0);
                }
            }
            b(m.r32(s + 0xdc) == 0)
        }
        0x58 => {
            if m.r32(s + 0x44) == 0 {
                return 1;
            }
            let id = m.r32(p + 0x10);
            let e = m.call(0x001e_1db8, &[s, id], &[]).0;
            let t = seq_t(m, s);
            let t0 = m.rf(s + 0x34);
            if m.rf(0x002c_3620) < t - t0 {
                return 1;
            }
            let v = m.r32(e + 0x1914) as i32;
            if v >= 7 {
                return 0;
            }
            b(v >= 4)
        }
        0x59 => {
            let a = m.r32(s + 0x44);
            if a == 0 {
                return 1;
            }
            let t = seq_t(m, s);
            let t0 = m.rf(s + 0x34);
            if m.rf(0x002c_3624) < t - t0 {
                return 1;
            }
            let v1 = m.rv(a + 0x19f0);
            let v0 = m.rv(a + 0x1a20);
            let eps = m.rf(0x002c_3628);
            for k in 0..3 {
                if m.call(0x0011_d4d0, &[], &[f4(v0[k]), f4(v1[k]), f4(eps)]).0 == 0 {
                    return 0;
                }
            }
            1
        }
        0x5a => {
            let a = m.r32(s + 0x44);
            if a == 0 {
                return 1;
            }
            let t = seq_t(m, s);
            let t0 = m.rf(s + 0x34);
            if m.rf(0x002c_362c) < t - t0 {
                return 1;
            }
            let v = m.r32(a + 0x1914);
            if v.wrapping_add(1) >= 2 {
                return 0;
            }
            if v == 0 {
                vslot(m, a, sl(0x2c4), &[0], &[]);
            }
            let q = m.r32(p + 0x18);
            b(m.r32(q + 0x2dc) as i32 == -1)
        }
        _ => 1,
    }
}

/// virtual call through the vtable at `obj + 0x20` (slot n: fn +8n+0xc, adjust +8n+8)
fn vslot20<M: QMem + ?Sized>(m: &mut M, obj: u32, n: u32, a: &[u32], v: &[V4]) -> (u32, V4) {
    let vt = m.r32(obj + 0x20);
    let f = m.r32(vt + 8 * n + 0xc);
    let adj = m.r16(vt + 8 * n + 8) as i16 as i32;
    let mut args = vec![obj.wrapping_add(adj as u32)];
    args.extend_from_slice(a);
    m.call(f, &args, v)
}

/// `seq[+0x24] && FUN_00132b10(seq[+0x24], k)` -> Some(seq[+0x24]) (re-read after the call)
fn guard<M: QMem + ?Sized>(m: &mut M, s: u32, k: u32) -> Option<u32> {
    let a = m.r32(s + 0x24);
    if a == 0 || m.call(0x0013_2b10, &[a, k], &[]).0 == 0 {
        return None;
    }
    Some(m.r32(s + 0x24))
}

/// FUN_001e1e30(seq): advance seq[+0x30] by one 0x50-byte op (stop past seq[+0x2c]) and execute it.
/// Prologue resolves the op's subject into seq[+0x24] (op field per jump table 0x2a7ae0; -1 ->
/// seq[+0x20] / seq[+0x1c] / owner[+0x6c]); the op dispatch is jump table 0x2a7c20 (0x5b entries).
/// Every executed op ends with seq[+0x34] = seq clock and vslot 1(seq); slot 0 brackets the start.
pub fn ee_sub_1e1e30<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    let s = a0;
    let n = m.r32(s + 0x30).wrapping_add(0x50);
    m.w32(s + 0x30, n);
    if m.r32(s + 0x2c) < n {
        return;
    }
    let buf = stack_push(m, 0x1d0);
    sub_1e1e30_body(m, s, buf);
    stack_pop(m, 0x1d0);
}

fn sub_1e1e30_body<M: QMem + ?Sized>(m: &mut M, s: u32, buf: u32) {
    vslot(m, s, 0, &[], &[]);
    let p = m.r32(s + 0x30);
    let op = m.r32(p);
    let fld: Option<u32> = match op {
        0x16 | 0x1a | 0x1c | 0x1e | 0x23 => Some(0x18),
        0x27 | 0x28 => Some(0x20),
        0x22 | 0x24 | 0x29 | 0x2a | 0x2b | 0x4a => Some(0x1c),
        0x1b | 0x47 | 0x48 | 0x4f | 0x50 | 0x51 | 0x57 => Some(0x10),
        0xb | 0x18 | 0x19 | 0x1d | 0x1f | 0x20 | 0x21 | 0x43 | 0x4b | 0x58 | 0x59 | 0x5a => Some(0x14),
        _ => None,
    };
    let a1 = match fld {
        Some(o) => m.r32(p + o),
        None => u32::MAX,
    };
    let mut s0: u32;
    if a1 as i32 == -1 {
        s0 = m.r32(s + 0x20);
        if s0 == 0 {
            s0 = m.r32(s + 0x1c);
            if s0 == 0 {
                let o = m.r32(s + 0x14);
                s0 = m.r32(o + 0x6c);
            }
        }
    } else {
        s0 = m.call(0x001e_1db8, &[s, a1], &[]).0;
    }
    m.w32(s + 0x24, s0);
    let op = m.r32(p);
    match op {
        0 => {
            let o = m.r32(s + 0x14);
            let v = m.call(0x001e_5ae0, &[o, p, s], &[]).0;
            let n = m.r32(v + 0x2c);
            m.w32(s + 0x30, n);
            m.call(0x001e_1c40, &[v], &[]);
        }
        2 => {
            let a1 = m.r32(p + 0x10);
            let v = if a1 as i32 == -1 {
                s
            } else {
                let o = m.r32(s + 0x14);
                m.call(0x001e_0180, &[o, a1, 0], &[]).0
            };
            if v != 0 {
                m.call(0x001e_5628, &[v], &[]);
            }
        }
        5 | 6 => {
            let a1 = m.r32(p + if op == 5 { 0x14 } else { 0x10 });
            let a = if a1 as i32 == -1 {
                m.r32(s + 0x14)
            } else {
                s0 = m.call(0x001e_1db8, &[s, a1], &[]).0;
                if s0 != 0 && m.call(0x0013_2b10, &[s0, 0x72], &[]).0 != 0 { s0 } else { 0 }
            };
            if a != 0 {
                if op == 5 {
                    let k = m.r32(p + 0x10);
                    m.call(0x001e_0020, &[a, k], &[]);
                } else {
                    m.call(0x001d_feb8, &[a], &[]);
                }
            }
        }
        7 => {
            let id = m.r32(p + 0x10);
            let x = m.call(0x001e_1db8, &[s, id], &[]).0;
            if x != 0 && m.call(0x0013_2b10, &[x, 0x32], &[]).0 != 0 {
                vslot(m, x, sl(0x27c), &[1], &[]);
            }
        }
        8 => {
            let id = m.r32(p + 0x10);
            let x = m.call(0x001e_1db8, &[s, id], &[]).0;
            m.w32(s + 0x20, x);
        }
        9 => {
            let o = m.r32(s + 0x14);
            m.w32(buf + 0x1b0, p + 0x10);
            m.w32(o + 0x68, s);
            let o = m.r32(s + 0x14);
            vslot(m, o, sl(0x1c), &[4, 0, 1, buf + 0x1b0, 0], &[]);
            let o = m.r32(s + 0x14);
            m.w32(o + 0x68, 0);
        }
        0xa => {
            let id = m.r32(p + 0x14);
            let mut s1 = m.call(0x001e_1db8, &[s, id], &[]).0;
            let mut done = false;
            if s1 != 0 {
                if m.call(0x0013_2b10, &[s1, 0x34], &[]).0 != 0 {
                    let a1 = m.r32(p + 0x10);
                    let a2 = m.r32(p + 0x18) | 1;
                    m.call(0x0014_3ee0, &[s1, a1, a2, 5], &[]);
                    if m.r32(p + 4) == 0 {
                        m.w32(s + 0x54, s1);
                    }
                    done = true;
                } else if m.call(0x0013_2b10, &[s1, 0x30], &[]).0 == 0 {
                    s1 = 0;
                }
            }
            if !done {
                let id = m.r32(p + 0x10);
                s0 = m.call(0x001e_1db8, &[s, id], &[]).0;
                if s0 != 0 && m.call(0x0013_2b10, &[s0, 0x6d], &[]).0 != 0 {
                    let a2 = m.r32(p + 0x18);
                    let v = m.call(0x0013_0488, &[s0, s1, a2], &[f4(0.0), f4(1.0)]).0;
                    if m.r32(p + 4) == 0 {
                        m.w32(s + 0x4c, v);
                        vslot(m, v, 0, &[], &[]);
                    }
                }
            }
        }
        0xb => {
            let id = m.r32(p + 0x10);
            let s1 = m.call(0x001e_1db8, &[s, id], &[]).0;
            if s1 != 0 && m.call(0x0013_2b10, &[s1, 0x72], &[]).0 != 0 {
                m.call(0x001d_fd58, &[s1, s0], &[]);
                if m.r32(p + 4) == 0 {
                    m.w32(s + 0x50, s1);
                }
            }
        }
        0xc => {
            let id = m.r32(p + 0x10);
            let v = m.call(0x001e_1db8, &[s, id], &[]).0;
            m.w32(s + 0x18, v);
            if v == 0 {
                let o = m.r32(s + 0x14);
                m.w32(s + 0x18, o);
            }
        }
        0xd => {
            let a = buf + 0x20;
            let w0 = m.r32(p + 0x10);
            m.w32(a, w0);
            let w1 = m.r32(p + 0x14);
            m.w32(a + 4, w1);
            let w2 = m.r32(p + 0x18);
            m.w32(a + 0xc, u32::MAX);
            m.w32(a + 8, w2);
            let a2 = m.r32(s + 0x18);
            let a3 = if m.r32(p + 0x1c) == 0 { 1 } else { 3 };
            m.call(0x0020_fff0, &[GS, a, a2, a3, s, s + 0x58], &[]);
            if m.r32(p + 4) != 0 {
                m.call(0x001e_5410, &[s], &[]);
            } else {
                let a0 = m.r32(s + 0x58);
                if a0 != 0 && m.r32(s + 0x5c) == 0 && m.call(0x001f_0638, &[a0], &[]).0 == 0 {
                    let a0 = m.r32(s + 0x58);
                    let v = m.call(0x001f_07e8, &[a0], &[]).0;
                    if v == m.r32(p + 0x18) {
                        m.w32(s + 0x5c, 1);
                    }
                }
            }
        }
        0xe | 0xf => {
            let id = m.r32(p + 0x10);
            s0 = m.call(0x0015_e938, &[GS, 0x405, id, 0], &[]).0;
            if s0 != 0 && m.call(0x0013_2b10, &[s0, 0x7e], &[]).0 != 0 {
                if m.r32(p) == 0xe {
                    let k = m.r32(p + 0x14);
                    vslot(m, s0, sl(0x18c), &[k], &[]);
                    if m.r32(p + 4) == 0 {
                        m.w32(s + 0x64, s0);
                    }
                } else {
                    vslot(m, s0, sl(0x194), &[], &[]);
                }
            }
        }
        0x10 => {
            let f = m.rf(p + 0x10);
            m.call(0x0014_d1d0, &[T_NOW, 6], &[f4(f)]);
            let o = m.r32(s + 0x14);
            let v1 = m.r32(o + 0x40);
            let even = (v1 ^ 1) & 1 != 0;
            if m.rf(p + 0x10) == 0.0 {
                if even {
                    let w = m.r32(v1) | 0x100;
                    m.w32(v1, w);
                } else {
                    m.w32(o + 0x40, v1 | 0x100);
                }
            } else if even {
                let w = m.r32(v1) & !0x100;
                m.w32(v1, w);
            } else {
                m.w32(o + 0x40, v1 & !0x100);
            }
        }
        0x11 => {
            let a3 = m.r32(s + 0x18);
            m.call(0x0016_1f48, &[0x002d_eb30, 1, p + 0x10, a3], &[]);
        }
        0x12 => {
            let a1 = m.r32(p + 0x10);
            let a2 = m.r32(p + 0x14);
            m.call(0x001e_50e0, &[s, a1, a2], &[]);
        }
        0x14 | 0x15 => {
            let go = if m.r32(p) == 0x15 {
                // FUN_00210b68 takes no meaningful args (a0 holds the jump target there)
                let v = m.call(0x0021_0b68, &[], &[]).0;
                v == m.r32(p + 0x14)
            } else {
                true
            };
            if go {
                let end = m.r32(s + 0x2c);
                let mut q = m.r32(s + 0x28).wrapping_add(0x50);
                let mut found = 0u32;
                let mut depth = 0i32;
                while q < end {
                    let k = m.r32(q) as i32;
                    if k == 1 {
                        depth -= 1;
                    } else if k == 0 {
                        depth += 1;
                    } else if k == 0x13 && depth == 0 && m.r32(q + 0x10) == m.r32(p + 0x10) {
                        found = q;
                        q = end;
                    }
                    q = q.wrapping_add(0x50);
                }
                if found != 0 {
                    m.w32(s + 0x30, found);
                }
            }
        }
        0x16 => {
            let id = m.r32(p + 0x10);
            let s1 = m.call(0x001e_1db8, &[s, id], &[]).0;
            if s1 != 0 && m.call(0x0013_2b10, &[s1, 0x29], &[]).0 != 0 {
                m.call(0x0022_8170, &[s1], &[]);
            } else if m.r32(s + 0x24) != 0 {
                if s1 != 0 && m.call(0x0013_2b10, &[s1, 0x1b], &[]).0 != 0 {
                    let t = m.r32(s + 0x24);
                    if m.call(0x0013_2b10, &[t, 0x34], &[]).0 != 0 {
                        let a = m.r32(s + 0x24) + 0x4c0;
                        m.call(0x0019_28f8, &[a, 8, s1, 0], &[]);
                        m.call(0x0019_2bd8, &[a, 9], &[]);
                        m.call(0x001e_4eb8, &[s, 0, 0], &[]);
                    }
                } else {
                    m.call(0x001e_4d40, &[s, s1, buf + 0x20, buf + 0x1b4], &[]);
                    let t = m.r32(s + 0x24);
                    if m.call(0x0013_2b10, &[t, 0x33], &[]).0 != 0 {
                        let o = m.r32(s + 0x24);
                        let pos = m.rv(buf + 0x20);
                        let base = m.rv(o + 0x240);
                        let d = [pos[0] - base[0], pos[1] - base[1], pos[2] - base[2], pos[3] - base[3]];
                        vslot(m, o, sl(0xec), &[], &[d]);
                        let f = m.rf(buf + 0x1b4);
                        m.call(0x0020_4660, &[o], &[f4(f)]);
                    } else {
                        let o = m.r32(s + 0x24);
                        let pos = m.rv(buf + 0x20);
                        vslot(m, o, sl(0xec), &[], &[pos]);
                    }
                    let f = m.rf(buf + 0x1b4);
                    let up = m.rv(0x002a_2dc0);
                    m.call(0x0011_b500, &[buf + 0x60], &[up, f4(f)]);
                    for k in (0..0x30).step_by(4) {
                        let w = m.r32(buf + 0x60 + k);
                        m.w32(buf + 0x30 + k, w);
                    }
                    let o = m.r32(s + 0x24);
                    vslot(m, o, sl(0xf4), &[buf + 0x30], &[]);
                    let t = m.r32(s + 0x24);
                    if m.call(0x0013_2b10, &[t, 0x34], &[]).0 != 0 {
                        let o = m.r32(s + 0x24);
                        let c = m.rv(0x002a_2dc0);
                        let xw = m.r32(o + 0x1048);
                        let x = f32::from_bits(xw);
                        // sqc2 of the qmtc2'd GPR: x = word, y = its sign extension (upper half unknown -> 0)
                        m.w32(buf + 0x60, xw);
                        m.w32(buf + 0x64, if (xw as i32) < 0 { u32::MAX } else { 0 });
                        m.w32(buf + 0x68, 0);
                        m.w32(buf + 0x6c, 0);
                        let b = m.rv(buf + 0x20);
                        let r = [b[0] + c[0] * x, b[1] + c[1] * x, b[2] + c[2] * x, b[3] + c[3] * x];
                        m.wv(o + 0x1060, r);
                        if m.r32(p + 0x14) == 0 && m.r32(o + 0x12f4) == 0 {
                            m.call(0x0019_5a00, &[o + 0x2c0, 0xb, 0, 0], &[]);
                            m.call(0x001f_4e28, &[o], &[]);
                        }
                        m.call(0x001e_4eb8, &[s, 0, 0], &[]);
                    }
                    let k = m.rv(0x002a_2dd0);
                    let o = m.r32(s + 0x24);
                    vslot(m, o, sl(0x11c), &[], &[k]);
                    let k = m.rv(0x002a_2dd0);
                    let o = m.r32(s + 0x24);
                    vslot(m, o, sl(0x124), &[], &[k]);
                }
            }
        }
        0x17 => {
            m.call(0x0014_d8c0, &[0x002d_df40], &[]);
        }
        0x18 => {
            let id = m.r32(p + 0x10);
            s0 = m.call(0x001e_1db8, &[s, id], &[]).0;
            if s0 != 0 && m.call(0x0013_2b10, &[s0, 0xf], &[]).0 != 0 {
                m.call(0x001e_4f20, &[s, s0], &[]);
            }
        }
        0x19 => {
            let id = m.r32(p + 0x10);
            let x = m.call(0x001e_1db8, &[s, id], &[]).0;
            m.call(0x001e_5000, &[s, x], &[]);
        }
        0x1a => {
            if let Some(o) = guard(m, s, 0x34) {
                s0 = o;
                if m.r32(s0 + 0xf54) as i32 != -1 {
                    let id = m.r32(p + 0x10);
                    let e = m.call(0x001e_1db8, &[s, id], &[]).0;
                    m.call(0x001e_4d40, &[s, e, buf + 0x20, 0], &[]);
                    let v = m.rv(buf + 0x20);
                    let a0 = m.r32(s0 + 0xf5c);
                    m.call(0x001e_6f60, &[a0, 0, 0, 0, 0, s + 0x7c], &[v]);
                    m.call(0x001e_4eb8, &[s, p, s0], &[]);
                }
            }
        }
        0x1c => {
            if guard(m, s, 0x35).is_some() {
                let k = m.r32(p + 0x14) as i32;
                let a0 = m.r32(s + 0x24);
                match k {
                    4 => {
                        m.call(0x0018_c250, &[a0, 0xd, u32::MAX], &[]);
                    }
                    2 => {
                        m.call(0x0018_e540, &[a0, 3], &[]);
                    }
                    8 => {
                        m.call(0x0018_c250, &[a0, 9, u32::MAX], &[]);
                    }
                    _ => {}
                }
            }
        }
        0x1d | 0x1f | 0x20 | 0x21 => {
            if let Some(o) = guard(m, s, 0x34) {
                s0 = o;
                let id = m.r32(p + 0x10);
                let x = m.call(0x001e_1db8, &[s, id], &[]).0;
                let f = match op {
                    0x1d => 0x0014_2140,
                    0x1f => 0x0014_2340,
                    0x20 => 0x0014_21a0,
                    _ => 0x0014_21f8,
                };
                m.call(f, &[s0, x], &[]);
                if op != 0x20 {
                    m.call(0x001e_4eb8, &[s, p, s0], &[]);
                }
            }
        }
        0x1e => {
            if let Some(s1) = guard(m, s, 0x34) {
                let id = m.r32(p + 0x10);
                let x = m.call(0x001e_1db8, &[s, id], &[]).0;
                let id = m.r32(p + 0x14);
                let y = m.call(0x001e_1db8, &[s, id], &[]).0;
                m.call(0x0014_2278, &[s1, x, y], &[]);
                m.call(0x001e_4eb8, &[s, p, s1], &[]);
            }
        }
        0x22 => {
            if let Some(o) = guard(m, s, 0x34) {
                s0 = o;
                let id = m.r32(p + 0x10);
                let s1 = m.call(0x0015_e938, &[GS, 1, id, s0], &[]).0;
                let a1 = m.r32(p + 0x14);
                let v = if a1 == 3 {
                    let tgt = m.rv(0x002d_dfd0);
                    m.wv(buf + 0x1c0, tgt);
                    let pos = m.call(0x001a_62e8, &[s0], &[]).1;
                    let tgt = m.rv(buf + 0x1c0);
                    let d = [tgt[0] - pos[0], tgt[1] - pos[1], tgt[2] - pos[2], tgt[3] - pos[3]];
                    m.call(0x0020_46b0, &[s0], &[d]);
                    0
                } else {
                    m.call(0x001e_1db8, &[s, a1], &[]).0
                };
                let a3 = m.r32(p + 0x18);
                m.call(0x0014_24e0, &[s0, s1, v, a3], &[]);
                m.call(0x001e_4eb8, &[s, p, s0], &[]);
            }
        }
        0x23 => {
            let id = m.r32(p + 0x14);
            if m.call(0x001e_1db8, &[s, id], &[]).0 != 0 {
                if let Some(o) = guard(m, s, 0x34) {
                    m.call(0x001e_4eb8, &[s, p, o], &[]);
                }
            }
        }
        0x24 => {
            if let Some(s1) = guard(m, s, 0x34) {
                let k = m.r32(p + 0x20);
                if k as i32 != -1 {
                    m.call(0x0014_3ee0, &[s1, k, 9, 6], &[]);
                }
                let id = m.r32(p + 0x10);
                s0 = m.call(0x001e_1db8, &[s, id], &[]).0;
                if s0 != 0 {
                    let mut hit = false;
                    for c in [0x22, 0xc, 0x26, 0x58, 0xb, 0x13] {
                        if m.call(0x0013_2b10, &[s0, c], &[]).0 != 0 {
                            hit = true;
                            break;
                        }
                    }
                    if hit {
                        let a1 = if m.call(0x0013_2b10, &[s1, 0x35], &[]).0 != 0 {
                            0xffff
                        } else if m.call(0x0013_2b10, &[s1, 0x37], &[]).0 != 0 {
                            0xf8fe
                        } else {
                            0xf8fa
                        };
                        m.call(0x0019_38c0, &[s1 + 0x4c0, a1, s0], &[]);
                    }
                    if m.r32(s1 + 0x4cc) as i32 != -1 && m.r32(s1 + 0x4c8) == 3 {
                        let a0 = m.r32(s1 + 0x4d4);
                        m.call(0x001e_4d40, &[s, s0, buf + 0x20, 0], &[]);
                        let v = m.rv(buf + 0x20);
                        m.call(0x001e_6f60, &[a0, 0, 0, 0, s + 0x78, 0], &[v]);
                    }
                }
                m.call(0x001e_4eb8, &[s, p, s1], &[]);
            }
        }
        0x25 => m.w32(s + 0xe0, 1),
        0x26 => m.w32(s + 0xe0, 0),
        0x27 => {
            if guard(m, s, 0x34).is_some() {
                let id = m.r32(p + 0x10);
                let s1 = m.call(0x001e_1db8, &[s, id], &[]).0;
                if s1 != 0 && m.call(0x0013_2b10, &[s1, 0xa], &[]).0 != 0 {
                    let k = m.r32(p + 0x24);
                    s0 = m.r32(s + 0x24);
                    if k as i32 != -1 {
                        m.call(0x0014_3ee0, &[s0, k, 9, 6], &[]);
                    }
                    s0 = m.call(0x0020_5438, &[s0, 0xa8], &[]).0;
                    let e0 = m.r32(s + 0xe0);
                    m.w32(s0 + 0x40, e0);
                    let w = m.r32(p + 0x18);
                    m.w32(s0 + 0xcc, w);
                    let w = m.r32(p + 0x1c);
                    m.w32(s0 + 0xd0, w);
                    let a2 = (m.r32(p + 0x14) == 0) as u32;
                    m.call(0x0017_8d90, &[s0, s1, a2], &[]);
                    vslot(m, s0, sl(0x44), &[], &[]);
                }
            }
        }
        0x28 => {
            if let Some(o) = guard(m, s, 0x34) {
                s0 = o;
                let id = m.r32(p + 0x10);
                let e = m.call(0x001e_1db8, &[s, id], &[]).0;
                m.call(0x001e_4d40, &[s, e, buf + 0x20, 0], &[]);
                let k = m.r32(p + 0x24);
                if k as i32 != -1 {
                    m.call(0x0014_3ee0, &[s0, k, 9, 6], &[]);
                }
                s0 = m.call(0x0020_5438, &[s0, 0xc1], &[]).0;
                let e0 = m.r32(s + 0xe0);
                m.w32(s0 + 0x30, e0);
                for k in 0..3 {
                    let w = m.r32(p + 0x14 + 4 * k);
                    m.w32(s0 + 0x34 + 4 * k, w);
                }
                let v = m.rv(buf + 0x20);
                m.call(0x0014_4f48, &[s0], &[v]);
                vslot(m, s0, sl(0x44), &[], &[]);
            }
        }
        0x29 => {
            if let Some(o) = guard(m, s, 0x34) {
                if m.r32(o + 0x4cc) as i32 != -1 && m.r32(o + 0x4c8) == 8 {
                    m.call(0x001e_4eb8, &[s, p, o], &[]);
                }
            }
        }
        0x2a => {
            if let Some(s1) = guard(m, s, 0x34) {
                let id = m.r32(p + 0x10);
                let s4 = m.call(0x001e_1db8, &[s, id], &[]).0;
                let s5 = buf + 0x20;
                m.call(0x0023_33b4, &[s5, 0, 0x20], &[]);
                let mut skip = false;
                if s4 == 0 {
                    let pos = m.call(0x001a_62e8, &[s1], &[]).1;
                    let o = m.rv(s1 + 0x240);
                    m.wv(buf + 0x30, [pos[0] + o[0], pos[1] + o[1], pos[2] + o[2], pos[3] + o[3]]);
                } else {
                    let a3 = m.call(0x0018_4670, &[s4], &[]).0;
                    m.w32(buf + 0x24, a3);
                    if a3 != 0 {
                        if m.r32(s1 + 0xf54) as i32 != -1 && m.r32(s1 + 0xf5c) == s4 {
                            skip = true;
                        } else {
                            vslot20(m, a3, 1, &[s1, buf + 0x30], &[]);
                            m.w32(buf + 0x20, s4);
                        }
                    } else {
                        m.call(0x001e_4d40, &[s, s4, buf + 0x30, 0], &[]);
                        m.call(0x0017_3610, &[buf + 0x30, 0, 0], &[f4(100.0), f4(-150.0)]);
                    }
                }
                if !skip {
                    m.call(0x0019_f590, &[s1, s5], &[]);
                    if m.r32(p + 0x14) == 1 {
                        m.w32(s1 + 0x2e8, 1);
                    }
                    if m.call(0x0013_2b10, &[s1, 0x35], &[]).0 != 0 {
                        let w = m.r32(p + 0x18);
                        m.w32(s1 + 0x18f0, w);
                    }
                    m.call(0x001e_4eb8, &[s, p, s1], &[]);
                }
            }
        }
        0x2b => {
            if guard(m, s, 0x34).is_some() {
                let id = m.r32(p + 0x10);
                let e = m.call(0x001e_1db8, &[s, id], &[]).0;
                s0 = m.r32(s + 0x24);
                m.call(0x001e_4d40, &[s, e, buf + 0x20, 0], &[]);
                let v = m.rv(buf + 0x20);
                let (f12, f13) = (m.rf(p + 0x14), m.rf(p + 0x18));
                m.call(0x0014_1968, &[s0], &[v, f4(f12), f4(f13)]);
            }
        }
        0x2c | 0x2d => {
            let eq = (m.r32(p) == 0x2c) as u32;
            let id = m.r32(p + 0x10);
            let f = m.rf(p + 0x14);
            let a3 = m.r32(s + 0x18);
            let t0 = m.r32(s + 0x14);
            s0 = m.call(0x0021_0338, &[GS, eq, id, a3, t0], &[f4(f)]).0;
            if s0 != 0 && m.r32(p + 4) == 0 && m.call(0x0013_2b10, &[s0, 0x30], &[]).0 != 0 {
                m.w32(s + 0x60, s0);
            }
        }
        0x2e | 0x2f | 0x30 | 0x31 => {
            let (k, f) = if op <= 0x2f { (0x2e, 0x0021_0408) } else { (0x30, 0x0021_04a0) };
            let eq = (m.r32(p) == k) as u32;
            let a3 = m.r32(p + 0x14);
            let a2 = m.r32(p + 0x10);
            let t1 = m.r32(s + 0x18);
            let t2 = m.r32(s + 0x14);
            m.call(f, &[GS, eq, a2, a3, 0, t1, t2], &[]);
        }
        0x32 => {
            let a1 = m.r32(p + 0x10);
            m.call(0x0014_f750, &[0x002d_df40, a1], &[]);
        }
        0x33 => {
            let f = m.rf(p + 0x10);
            let a1 = m.r32(p + 0x14);
            m.call(0x0018_7e28, &[0x002d_fc00, a1], &[f4(f)]);
        }
        0x34 => {
            let f = m.rf(p + 0x1c);
            let (a1, a2, a3) = (m.r32(p + 0x10), m.r32(p + 0x14), m.r32(p + 0x18));
            let (t0, t1) = (m.r32(s + 0x18), m.r32(s + 0x14));
            m.call(0x0021_0538, &[GS, a1, a2, a3, t0, t1], &[f4(f)]);
        }
        0x35 => {
            let f = m.rf(p + 0x18);
            let (a1, a2) = (m.r32(p + 0x10), m.r32(p + 0x14));
            let (a3, t0) = (m.r32(s + 0x18), m.r32(s + 0x14));
            m.call(0x0021_0678, &[GS, a1, a2, a3, t0], &[f4(f)]);
        }
        0x36 => {
            let t2 = m.r32(s + 0x18);
            let f14 = m.rf(p + 0x28);
            let (a1, a2, a3, t0) = (m.r32(p + 0x10), m.r32(p + 0x14), m.r32(p + 0x18), m.r32(p + 0x1c));
            let (f12, f13) = (m.rf(p + 0x20), m.rf(p + 0x24));
            let t3 = m.r32(s + 0x14);
            m.call(0x0021_0740, &[GS, a1, a2, a3, t0, 0, t2, t3, s + 0xec], &[f4(f12), f4(f13), f4(f14)]);
        }
        0x37 => {
            let a1 = m.r32(p + 0x10);
            let a2 = m.r32(s + 0x14);
            m.call(0x0021_08e8, &[GS, a1, a2, s + 0xec], &[]);
        }
        0x38 => {
            let f12 = m.rf(p + 0x40);
            let (mut f20, s5) = if f12 == 0.0 { (0.0f32, true) } else { (f12, false) };
            let mut q = p + 0x50;
            let mut s4 = 0u32;
            let mut depth = 0i32;
            if q < m.r32(s + 0x2c) {
                loop {
                    let k = m.r32(q) as i32;
                    let tail = if k == 1 {
                        depth -= 1;
                        false
                    } else if k == 0 {
                        depth += 1;
                        false
                    } else if k == 0x39 {
                        if depth == 0 {
                            s4 = q;
                        }
                        false
                    } else {
                        true
                    };
                    if tail && depth == 0 && s5 {
                        f20 += m.call(0x001e_4530, &[s, q], &[]).1[0];
                    }
                    q = q.wrapping_add(0x50);
                    if !(q < m.r32(s + 0x2c)) || s4 != 0 {
                        break;
                    }
                }
            }
            if s4 != 0 {
                m.call(0x001e_57a0, &[s, p, s4], &[f4(f20)]);
            }
        }
        0x39 => {
            if m.rf(s + 0x80) == m.rf(0x002a_7ad0) {
                if m.r32(p + 0x40) == 0 {
                    m.w32(s + 0x9c, 0);
                } else {
                    let a1 = m.r32(p + 0x44);
                    m.call(0x001e_5a98, &[s, a1], &[]);
                }
            }
        }
        0x3a => {
            let a1 = m.r32(p + 0x10);
            m.call(0x001e_5a98, &[s, a1], &[]);
        }
        0x3b => {
            let id = m.r32(p + 0x14);
            let s1 = m.call(0x001e_1db8, &[s, id], &[]).0;
            if s1 != 0 {
                // result unused in the asm (a2 = op[6] is reloaded either way)
                m.call(0x0013_2b10, &[s1, 0x33], &[]);
            }
            let a2 = m.r32(p + 0x18);
            let s4 = m.call(0x0015_e938, &[GS, 0x405, a2, 0], &[]).0;
            let id = m.r32(p + 0x10);
            s0 = m.call(0x001e_1db8, &[s, id], &[]).0;
            if s0 != 0 && m.call(0x0013_2b10, &[s0, 0x34], &[]).0 != 0 {
                m.w32(s + 0xa0, s0);
                m.call(0x0019_cc50, &[s0 + 0xf00, 1, s1, s4], &[]);
            }
        }
        0x3c => {
            m.call(0x001e_54b0, &[s], &[]);
        }
        0x3d | 0x3e => {
            let id = m.r32(p + 0x10);
            let x = m.call(0x001e_1db8, &[s, id], &[]).0;
            let f = if op == 0x3d { 0x0015_0e20 } else { 0x0015_0e60 };
            m.call(f, &[0x002d_e460, x], &[]);
        }
        0x3f => {
            let f = m.rf(p + 0x30);
            let (q1, q2) = (m.rv(p + 0x10), m.rv(p + 0x20));
            m.call(0x0015_0ea0, &[0x002d_e460], &[q1, q2, f4(f)]);
        }
        0x40 => {
            let f = m.rf(p + 0x10);
            m.call(0x0015_0ee8, &[0x002d_e460], &[f4(f)]);
        }
        0x41 => {
            m.call(0x0015_0ef0, &[0x002d_e460], &[]);
        }
        0x42 => {
            let v = m.r32(p + 0x10) & 0xf0;
            m.w32(0x0044_35f8, v);
        }
        0x44 => {
            m.call(0x001d_d820, &[0x003d_a530], &[]);
        }
        0x45 => {
            let (a1, a2) = (m.r32(p + 0x10), m.r32(p + 0x14));
            m.call(0x001f_e5f0, &[0x003d_d040, a1, a2], &[]);
            m.w32(s + 0x68, p);
        }
        0x46 => {
            m.call(0x0022_7788, &[0x0044_35f0], &[]);
        }
        0x47 | 0x48 => {
            if s0 != 0 {
                let (a2, a3) = (m.r32(p + 0x14), m.r32(p + 0x18));
                let f = if op == 0x47 { 0x001e_51a0 } else { 0x001e_5280 };
                m.call(f, &[s, s0, a2, a3], &[]);
            }
        }
        0x49 => {
            if m.r32(0x002d_e570) != 0 {
                let end = m.r32(s + 0x2c);
                let mut q = p + 0x50;
                let mut n = 0u32;
                let mut depth = 0i32;
                let mut t0 = buf + 0x90;
                while q < end {
                    let k = m.r32(q) as i32;
                    if k == 1 {
                        depth -= 1;
                    } else if k == 0 {
                        depth += 1;
                    } else if k == 0x49 {
                        if depth == 0 {
                            q = m.r32(s + 0x2c);
                        }
                    } else if k == 0x4a && depth == 0 && n < 6 {
                        let w = m.r32(q + 0x10);
                        m.w32(t0, w);
                        let w = m.r32(q + 0x14);
                        m.w32(t0 + 4, w);
                        t0 += 0x30;
                        n += 1;
                    }
                    q = q.wrapping_add(0x50);
                }
                let a0 = m.r32(0x002d_e570);
                let a1 = m.r32(p + 0x10);
                m.call(0x0015_31a8, &[a0, a1, n, buf + 0x90], &[]);
            }
        }
        0x4a => {
            if guard(m, s, 0x44).is_some() {
                let a0 = m.r32(0x002d_e570);
                if a0 != 0 {
                    let f = m.rf(p + 0x10);
                    let a1 = m.r32(s + 0x24);
                    let (a2, a3) = (m.r32(p + 0x14), m.r32(p + 0x18));
                    m.call(0x0015_32d8, &[a0, a1, a2, a3], &[f4(f)]);
                }
            }
        }
        0x4b => {
            if let Some(o) = guard(m, s, 0x44) {
                s0 = o;
                let a2 = m.r32(p + 0x10);
                let v = if a2 as i32 == -1 { 0 } else { m.call(0x0015_e938, &[GS, 4, a2, s0], &[]).0 };
                m.call(0x0015_24a0, &[s0, v, 0], &[]);
            }
        }
        0x4c | 0x4d => {
            let a1 = m.r32(p + 0x10);
            let f = if op == 0x4c { 0x001f_e150 } else { 0x001f_e310 };
            m.call(f, &[0x003d_d040, a1], &[]);
        }
        0x4f => {
            if let Some(o) = guard(m, s, 0x36) {
                s0 = o;
                let id = m.r32(p + 0x14);
                let e = m.call(0x001e_1db8, &[s, id], &[]).0;
                if e != 0 {
                    let v = m.call(0x001f_67e0, &[e, u32::MAX], &[]).0;
                    if v != 0 {
                        let (t0, a2, a3) = (m.r32(p + 0x20), m.r32(p + 0x1c), m.r32(p + 0x18));
                        m.call(0x001b_9c00, &[s0, v, a2, a3, t0], &[]);
                    }
                }
            }
        }
        0x50 | 0x51 => {
            if let Some(o) = guard(m, s, 0x37) {
                m.call(0x001e_4eb8, &[s, p, o], &[]);
            }
        }
        0x53 => {
            let f = m.rf(p + 0x14);
            let a1 = m.r32(p + 0x10);
            m.call(0x001d_a4b8, &[0x003d_a0c0, a1], &[f4(f)]);
        }
        0x54 => {
            let a1 = m.r32(p + 0x10);
            m.call(0x001d_a578, &[0x003d_a0c0, a1], &[]);
        }
        0x55 => {
            let id = m.r32(p + 0x10);
            let s1 = m.call(0x001e_1db8, &[s, id], &[]).0;
            let id = m.r32(p + 0x14);
            s0 = m.call(0x001e_1db8, &[s, id], &[]).0;
            m.call(0x0017_1cf8, &[s1, s0], &[]);
            if m.r32(p + 4) == 0 {
                m.w32(s + 0xd8, s1);
                m.w32(s + 0xdc, s0);
            }
        }
        0x56 => {
            let id = m.r32(p + 0x10);
            s0 = m.call(0x001e_1db8, &[s, id], &[]).0;
            let id = m.r32(p + 0x14);
            let v = m.call(0x001e_1db8, &[s, id], &[]).0;
            m.call(0x0017_1d58, &[s0, v], &[]);
        }
        0x57 => {
            if let Some(o) = guard(m, s, 0x36) {
                m.call(0x001b_9880, &[o], &[]);
            }
        }
        0x58 => {
            if let Some(o) = guard(m, s, 0x37) {
                s0 = o;
                let id = m.r32(p + 0x10);
                let e = m.call(0x001e_1db8, &[s, id], &[]).0;
                if e != 0 && m.call(0x0013_2b10, &[e, 0x36], &[]).0 != 0 {
                    let a1 = if m.r32(s0 + 0x2dc) != 0x11 { 1 } else { 3 };
                    m.call(0x0019_5a00, &[s0 + 0x2c0, a1, 0, 0], &[]);
                    m.call(0x001e_4eb8, &[s, p, s0], &[]);
                }
            }
        }
        0x59 | 0x5a => {
            if let Some(o) = guard(m, s, 0x36) {
                s0 = o;
                let v = m.r32(s0 + 0x1914) as i32;
                if (4..7).contains(&v) {
                    if op == 0x5a {
                        let w = m.r32(s0 + 0x1910);
                        m.w32(p + 0x18, w);
                    }
                    let id = m.r32(p + 0x10);
                    let e = m.call(0x001e_1db8, &[s, id], &[]).0;
                    if e != 0 {
                        m.call(0x0022_0508, &[s0 + 0x1910, e], &[]);
                        if op == 0x5a {
                            m.call(0x001e_4eb8, &[s, p, s0], &[]);
                        }
                    }
                }
            }
        }
        _ => {}
    }
    let _ = s0;
    let t = seq_t(m, s);
    m.wf(s + 0x34, t);
    vslot(m, s, 1, &[], &[]);
}
