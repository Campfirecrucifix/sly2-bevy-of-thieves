//! WP-Q5: run_001e46a0 queue package: the cutscene / sequence (k22_seq) player-steering helpers: the
//! per-op input override FUN_001e46a0 (op +0x40 -> {+0 code, +0x10.. args}, player +0x44, steering
//! record `j` {+0x10 / +0x14 stick, +0x18 heading, +0x1c magnitude, +0x30 flags}), the target point /
//! heading query FUN_001e4d40 and the sequence-slot setters / releasers FUN_001e4f20..FUN_001e5570.
//! Ported 1:1 from the disassembly against `QMem` (the decompiler drops the quadword arguments of
//! FUN_001e6f60 / FUN_001c4768 / FUN_001a6fc0 and mislabels the 0x5a prot3w / pextuw lanes).
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};
use crate::engine::classes::q::q5util::{f4, pos};

const PI: f32 = 3.1415927;

fn atan2<M: QMem + ?Sized>(m: &mut M, y: f32, x: f32) -> f32 {
    m.call(0x0024_58f0, &[], &[f4(y), f4(x)]).1[0]
}

fn kind<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32) -> bool {
    m.call(0x0013_2b10, &[o, k], &[]).0 != 0
}

/// heading of `o` from its slot-30 frame (+4 / +0)
fn frame_heading<M: QMem + ?Sized>(m: &mut M, o: u32) -> f32 {
    let a = vslot(m, o, 30, &[], &[]).0;
    let b = vslot(m, o, 30, &[], &[]).0;
    let (y, x) = (m.rf(a + 4), m.rf(b));
    atan2(m, y, x)
}

/// the "flip" test of a rail parameter: looping rails (+0xe4) compare the two ways round
/// (length = `len_fn`(rail)), others `a < b`
fn behind<M: QMem + ?Sized>(m: &mut M, rail: u32, len_fn: u32, a: f32, b: f32) -> bool {
    if m.r32(rail + 0xe4) == 0 {
        return a < b;
    }
    let (f20, f21);
    if a < b {
        f20 = b - a;
        let l = m.call(len_fn, &[rail], &[]).1[0];
        f21 = l + a - b;
    } else {
        f21 = a - b;
        let l = m.call(len_fn, &[rail], &[]).1[0];
        f20 = l + b - a;
    }
    f20 < f21
}

/// FUN_001e46a0: steering override for sequence `s` into record `j` by the current op code:
/// 0x1a follow the +0xf5c rail (tangent from the points at +-50 around +0xf60 through FUN_001c4768,
/// reversed when +0x7c is behind), magnitude 1; 0x1d..0x22 stop (flag 0x20 while the player is
/// attached, +0x4cc != -1); 0x23 face target op[5] (FUN_00204660) and stop; 0x24 walk to target op[4]
/// (speed |op[5]|, flag 8 with op[9]; on rails (+0x4c8 3) along the rail, else towards it scaled by
/// curve 0x2c3630 of the xy distance); 0x29 strafe (+0x4c8 8) in the player frame (FUN_001a6fc0);
/// 0x2a zero; 0x50 / 0x51 set / clear +0x48; 0x58 stop + flag 0x20 + FUN_001c45a8; 0x5a stop, flag
/// 0x40 when +0x1a20 equals +0x19f0 (FFloatsNear 0x2c3640, xyz). With +0x48 set flag 0x10 (op 0x50
/// also FUN_001c45a8).
pub fn ee_sub_1e46a0<M: QMem + ?Sized>(m: &mut M, s: u32, j: u32) {
    let op = m.r32(s + 0x40);
    let pl = m.r32(s + 0x44);
    let code = m.r32(op);
    let sp = stack_push(m, 0x60);
    let stop_tail = |m: &mut M| {
        if m.r32(pl + 0x4cc) != 0xffff_ffff {
            let f = m.r32(j + 0x30) | 0x20;
            m.w32(j + 0x30, f);
        }
    };
    let walk_tail = |m: &mut M| {
        if m.r32(pl + 0x4cc) != 0xffff_ffff {
            let st = m.r32(pl + 0x4c8);
            if st == 0 || st == 0xd {
                let f = m.r32(j + 0x30) | 0x20;
                m.w32(j + 0x30, f);
            }
        }
    };
    match code {
        0x1a => {
            let rail = m.r32(pl + 0xf5c);
            let t = m.rf(pl + 0xf60) + 50.0;
            let u = m.call(0x001e_6728, &[rail], &[f4(t)]).1[0];
            let rail = m.r32(pl + 0xf5c);
            m.call(0x001e_6f10, &[rail, sp, 0], &[f4(u)]);
            let t = m.rf(pl + 0xf60) - 50.0;
            let rail = m.r32(pl + 0xf5c);
            let u = m.call(0x001e_6728, &[rail], &[f4(t)]).1[0];
            let rail = m.r32(pl + 0xf5c);
            m.call(0x001e_6f10, &[rail, sp + 0x10, 0], &[f4(u)]);
            let (a, b) = (m.rv(sp), m.rv(sp + 0x10));
            let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2], a[3] - b[3]];
            let r = m.call(0x001c_4768, &[j], &[d]).1;
            let (x, y) = (r[0], r[1]);
            let h = atan2(m, y, x);
            m.wf(j + 0x18, h);
            m.wf(j + 0x10, x);
            m.wf(j + 0x14, y);
            let rail = m.r32(pl + 0xf5c);
            let (a, b) = (m.rf(s + 0x7c), m.rf(pl + 0xf60));
            if behind(m, rail, 0x001e_6648, a, b) {
                let h = m.rf(j + 0x18) + PI;
                let h = m.call(0x0011_cf60, &[], &[f4(h)]).1[0];
                m.wf(j + 0x18, h);
                let (x, y) = (m.rf(j + 0x10), m.rf(j + 0x14));
                m.wf(j + 0x10, -x);
                m.wf(j + 0x14, -y);
            }
            m.wf(j + 0x1c, 1.0);
        }
        0x1d..=0x22 => {
            m.w32(j + 0x1c, 0);
            stop_tail(m);
        }
        0x23 => {
            let k = m.r32(op + 0x14);
            let o = m.call(0x001e_1db8, &[s, k], &[]).0;
            if o != 0 {
                ee_sub_1e4d40(m, s, o, 0, sp + 0x30);
                let h = m.rf(sp + 0x30);
                m.call(0x0020_4660, &[pl], &[f4(h)]);
                let h = m.rf(sp + 0x30);
                m.wf(j + 0x18, h);
            }
            m.w32(j + 0x1c, 0);
            stop_tail(m);
        }
        0x24 => {
            let v = m.rf(op + 0x14).abs();
            m.wf(j + 0x1c, v);
            if m.r32(op + 0x24) != 0 {
                let f = m.r32(j + 0x30) | 8;
                m.w32(j + 0x30, f);
            }
            let k = m.r32(op + 0x10);
            let o = m.call(0x001e_1db8, &[s, k], &[]).0;
            if o != 0 {
                if m.r32(pl + 0x4cc) != 0xffff_ffff && m.r32(pl + 0x4c8) == 3 {
                    let rail = m.r32(pl + 0x4d4);
                    let t = m.rf(pl + 0x4dc);
                    m.call(0x001e_67b8, &[rail, sp, sp + 0x10], &[f4(t)]);
                    m.w32(sp + 0x18, 0);
                    let d = m.rv(sp + 0x10);
                    let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                    let n = if l < 0.0001 {
                        m.rv(0x002a_2da0)
                    } else {
                        let k = 1.0 / l;
                        [d[0] * k, d[1] * k, d[2] * k, d[3] * k]
                    };
                    m.wv(sp + 0x10, n);
                    let h = atan2(m, n[1], n[0]);
                    m.wf(j + 0x18, h);
                    let (a, b) = (m.rf(s + 0x78), m.rf(pl + 0x4dc));
                    if behind(m, rail, 0x001e_6630, a, b) {
                        let h = m.rf(j + 0x18) + PI;
                        let h = m.call(0x0011_cf60, &[], &[f4(h)]).1[0];
                        m.wf(j + 0x18, h);
                    }
                } else {
                    ee_sub_1e4d40(m, s, o, sp, 0);
                    let q = m.rv(sp);
                    m.wv(sp + 0x40, q);
                    let p = pos(m, pl);
                    let d = [q[0] - p[0], q[1] - p[1], q[2] - p[2], q[3] - p[3]];
                    m.wv(sp + 0x10, d);
                    let h = atan2(m, d[1], d[0]);
                    m.wf(j + 0x18, h);
                    let dist = (d[0] * d[0] + d[1] * d[1]).sqrt();
                    let c = m.rf(0x002c_3630) + dist * (m.rf(0x002c_3634) + dist * m.rf(0x002c_3638));
                    let (lo, hi) = (m.rf(0x002b_bb60), m.rf(0x002b_bb64));
                    let c = if c < lo {
                        lo
                    } else if hi < c {
                        hi
                    } else {
                        c
                    };
                    let v = m.rf(j + 0x1c) * c;
                    m.wf(j + 0x1c, v);
                }
            }
            walk_tail(m);
        }
        0x29 => {
            let v = m.rf(op + 0x14).abs();
            m.wf(j + 0x1c, v);
            let k = m.r32(op + 0x10);
            let o = m.call(0x001e_1db8, &[s, k], &[]).0;
            if o != 0 && m.r32(pl + 0x4cc) != 0xffff_ffff && m.r32(pl + 0x4c8) == 8 {
                ee_sub_1e4d40(m, s, o, sp, 0);
                let q = m.rv(sp);
                m.wv(sp + 0x50, q);
                let p = pos(m, pl);
                let d = [q[0] - p[0], q[1] - p[1], 0.0, q[3] - p[3]];
                m.wv(sp + 0x10, d);
                let r = m.call(0x001a_6fc0, &[0, pl], &[d]).1;
                let k = 1.0 / (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt();
                let n = [r[0] * k, r[1] * k, r[2] * k, r[3] * k];
                m.wv(sp + 0x20, n);
                let v = m.rf(j + 0x1c);
                m.wf(j + 0x14, n[0] * v);
                m.wf(j + 0x10, -n[1] * v);
            }
        }
        0x2a => m.w32(j + 0x1c, 0),
        0x50 => m.w32(s + 0x48, 1),
        0x51 => m.w32(s + 0x48, 0),
        0x58 => {
            m.w32(j + 0x1c, 0);
            let f = m.r32(j + 0x30) | 0x20;
            m.w32(j + 0x30, f);
            m.call(0x001c_45a8, &[j], &[]);
        }
        0x5a => {
            let eps = m.rf(0x002c_3640);
            let (a, b) = (m.rv(pl + 0x1a20), m.rv(pl + 0x19f0));
            let mut same = true;
            for c in 0..3 {
                if m.call(0x0011_d4d0, &[], &[f4(a[c]), f4(b[c]), f4(eps)]).0 == 0 {
                    same = false;
                    break;
                }
            }
            if same {
                let f = m.r32(j + 0x30) | 0x40;
                m.w32(j + 0x30, f);
            }
            m.w32(j + 0x1c, 0);
            m.call(0x001c_45a8, &[j], &[]);
        }
        _ => {}
    }
    if m.r32(s + 0x48) != 0 {
        let f = m.r32(j + 0x30) | 0x10;
        m.w32(j + 0x30, f);
        if m.r32(op) == 0x50 {
            m.call(0x001c_45a8, &[j], &[]);
        }
    }
    stack_pop(m, 0x60);
}

/// FUN_001e4d40: point (`out_q`) and heading (`out_h`) of target `o` for sequence `s` (actor
/// +0x24, else the player +0x44). No target: the actor's position (plus +0x240 and heading +0x1a8
/// when it is kind 0x33, else its frame heading). A rail target (kind 10): the closest rail point
/// to the actor (FUN_001e6f60) and the rail tangent heading; other targets their position / frame
/// heading.
pub fn ee_sub_1e4d40<M: QMem + ?Sized>(m: &mut M, s: u32, o: u32, out_q: u32, out_h: u32) {
    let mut a = m.r32(s + 0x24);
    if a == 0 {
        a = m.r32(s + 0x44);
    }
    let sp = stack_push(m, 0x20);
    let h;
    if o != 0 {
        if kind(m, o, 10) {
            let p = pos(m, a);
            m.call(0x001e_6f60, &[o, 0, sp, sp + 0x10, 0, 0], &[p]);
            let (y, x) = (m.rf(sp + 0x14), m.rf(sp + 0x10));
            h = atan2(m, y, x);
        } else {
            let p = pos(m, o);
            m.wv(sp, p);
            h = frame_heading(m, o);
        }
    } else {
        let p = pos(m, a);
        m.wv(sp, p);
        if kind(m, a, 0x33) {
            let d = m.rv(a + 0x240);
            m.wv(sp, [p[0] + d[0], p[1] + d[1], p[2] + d[2], p[3] + d[3]]);
            h = m.rf(a + 0x1a8);
        } else {
            h = frame_heading(m, a);
        }
    }
    if out_q != 0 {
        let q = m.rv(sp);
        m.wv(out_q, q);
    }
    if out_h != 0 {
        m.wf(out_h, h);
    }
    stack_pop(m, 0x20);
}

/// FUN_001e4f20: replace the sequence's sound / emitter +0x6c: the old one is stopped
/// (FUN_00221268(-1)), its +0xe0 cleared and, when owned (+0x70), deleted (slot 8); the new one
/// (owned when its +0x28 bit 60 is clear -> slot 7) gets the actor (+0x24 of kind 0x30, else
/// FUN_001c4a48) as +0xe0 and is started (FUN_00221268(1))
pub fn ee_sub_1e4f20<M: QMem + ?Sized>(m: &mut M, s: u32, e: u32) {
    let old = m.r32(s + 0x6c);
    if old != 0 {
        m.call(0x0022_1268, &[old, 0xffff_ffff], &[]);
        let old = m.r32(s + 0x6c);
        m.w32(old + 0xe0, 0);
        if m.r32(s + 0x70) != 0 {
            let old = m.r32(s + 0x6c);
            vslot(m, old, 8, &[], &[]);
        }
        m.w32(s + 0x6c, 0);
    }
    if e == 0 {
        return;
    }
    m.w32(s + 0x6c, e);
    let own = (m.r64(e + 0x28) >> 60) & 1 == 0;
    m.w32(s + 0x70, own as u32);
    if own {
        vslot(m, e, 7, &[], &[]);
    }
    let a = m.r32(s + 0x24);
    let x = if a != 0 && kind(m, a, 0x30) {
        m.r32(s + 0x24)
    } else {
        m.call(0x001c_4a48, &[0x003d_d490], &[]).0
    };
    m.w32(e + 0xe0, x);
    m.call(0x0022_1268, &[e, 1], &[]);
}

/// FUN_001e5000: set the sequence's look target +0x74 for a kind-0x34 actor (+0x24): releasing
/// keeps the current look as an offset (+0x1070 = +0x1060 - position, w 0) and clears +0x1044; a new
/// target sets +0x1044, +0x1060 = its point (FUN_001e4d40) raised by +0x1048, offset 0
pub fn ee_sub_1e5000<M: QMem + ?Sized>(m: &mut M, s: u32, t: u32) {
    let a = m.r32(s + 0x24);
    let k = if a != 0 && kind(m, a, 0x34) { m.r32(s + 0x24) } else { 0 };
    if m.r32(s + 0x74) != 0 {
        if k != 0 {
            let q = m.rv(k + 0x1060);
            m.w32(k + 0x1044, 0);
            let p = pos(m, k);
            m.wv(k + 0x1070, [q[0] - p[0], q[1] - p[1], q[2] - p[2], q[3] - p[3]]);
            m.w32(k + 0x1078, 0);
        }
        m.w32(s + 0x74, 0);
    }
    if t == 0 {
        return;
    }
    if k != 0 {
        m.w32(k + 0x1044, 1);
        ee_sub_1e4d40(m, s, t, k + 0x1060, 0);
        let z = m.rf(k + 0x1068) + m.rf(k + 0x1048);
        m.wf(k + 0x1068, z);
        let zq = m.rv(0x002a_2dd0);
        m.wv(k + 0x1070, zq);
    }
    m.w32(s + 0x74, t);
}

/// FUN_001e51a0: attach kind-0x33 object `o` with (a, b) to the sequence's 4-entry list (+0xa4
/// count, +0xa8 objects, +0xb8 / +0xc8 args) unless present; slot 82(a, b)
pub fn ee_sub_1e51a0<M: QMem + ?Sized>(m: &mut M, s: u32, o: u32, a: u32, b: u32) {
    if !kind(m, o, 0x33) {
        return;
    }
    let n = m.r32(s + 0xa4);
    for i in 0..(n as i32).max(0) as u32 {
        if m.r32(s + 0xa8 + 4 * i) == o {
            return;
        }
    }
    if n < 4 {
        m.w32(s + n * 4 + 0xb8, a);
        let n = m.r32(s + 0xa4);
        m.w32(s + n * 4 + 200, b);
        let n = m.r32(s + 0xa4);
        m.w32(s + n * 4 + 0xa8, o);
        m.w32(s + 0xa4, n + 1);
        vslot(m, o, 82, &[a, b], &[]);
    }
}

/// FUN_001e5280: detach `o` from that list: the object entries after it shift down (FUN_002332b8,
/// the arg arrays are not shifted), count - 1, slot 83(a, b)
pub fn ee_sub_1e5280<M: QMem + ?Sized>(m: &mut M, s: u32, o: u32, a: u32, b: u32) {
    if !kind(m, o, 0x33) {
        return;
    }
    let n = m.r32(s + 0xa4) as i32;
    for i in 0..n.max(0) {
        let e = s + 0xa8 + 4 * i as u32;
        if m.r32(e) == o {
            m.call(0x0023_32b8, &[e, e + 4, ((n - i) * 4 - 4) as u32], &[]);
            let c = m.r32(s + 0xa4).wrapping_sub(1);
            m.w32(s + 0xa4, c);
            vslot(m, o, 83, &[a, b], &[]);
            return;
        }
    }
}

/// FUN_001e5370: release +0x4c (FUN_00130b18, slot 1) and +0x54 (FUN_00143ff8)
pub fn ee_sub_1e5370<M: QMem + ?Sized>(m: &mut M, s: u32) {
    let a = m.r32(s + 0x4c);
    if a != 0 {
        m.call(0x0013_0b18, &[a], &[]);
        let a = m.r32(s + 0x4c);
        vslot(m, a, 1, &[], &[]);
        m.w32(s + 0x4c, 0);
    }
    let b = m.r32(s + 0x54);
    if b != 0 {
        m.call(0x0014_3ff8, &[b], &[]);
        m.w32(s + 0x54, 0);
    }
}

/// FUN_001e53d8: release +0x50 (FUN_001dfeb8)
pub fn ee_sub_1e53d8<M: QMem + ?Sized>(m: &mut M, s: u32) {
    let a = m.r32(s + 0x50);
    if a != 0 {
        m.call(0x001d_feb8, &[a], &[]);
        m.w32(s + 0x50, 0);
    }
}

/// FUN_001e5410: release +0x58 (FUN_00133448(it, s), slot 1), clear +0x5c
pub fn ee_sub_1e5410<M: QMem + ?Sized>(m: &mut M, s: u32) {
    let a = m.r32(s + 0x58);
    if a != 0 {
        m.call(0x0013_3448, &[a, s], &[]);
        let a = m.r32(s + 0x58);
        vslot(m, a, 1, &[], &[]);
        m.w32(s + 0x5c, 0);
        m.w32(s + 0x58, 0);
    }
}

/// FUN_001e5468: release +0x64 (slot 49)
pub fn ee_sub_1e5468<M: QMem + ?Sized>(m: &mut M, s: u32) {
    let a = m.r32(s + 100);
    if a != 0 {
        vslot(m, a, 49, &[], &[]);
        m.w32(s + 100, 0);
    }
}

/// FUN_001e54b0: release +0xa0 (FUN_0019cce8 on +0xf00)
pub fn ee_sub_1e54b0<M: QMem + ?Sized>(m: &mut M, s: u32) {
    let a = m.r32(s + 0xa0);
    if a != 0 {
        m.call(0x0019_cce8, &[a + 0xf00], &[]);
        m.w32(s + 0xa0, 0);
    }
}

/// FUN_001e54e8: detach every attached object, last first
pub fn ee_sub_1e54e8<M: QMem + ?Sized>(m: &mut M, s: u32) {
    loop {
        let n = m.r32(s + 0xa4);
        if n == 0 {
            break;
        }
        let i = (n - 1) * 4;
        let (o, a, b) = (m.r32(s + 0xa8 + i), m.r32(s + 0xb8 + i), m.r32(s + 200 + i));
        ee_sub_1e5280(m, s, o, a, b);
        if m.r32(s + 0xa4) == n {
            // the original spins here; a non-0x33 entry cannot be detached
            break;
        }
    }
}

/// FUN_001e5570: when the op cursor +0x30 is inside (+0x28, +0x2c) and its code is 0x27 / 0x28,
/// a kind-0x34 actor's +0x26c gets FUN_001eecd0
pub fn ee_sub_1e5570<M: QMem + ?Sized>(m: &mut M, s: u32) {
    let p = m.r32(s + 0x30);
    if !(m.r32(s + 0x28) < p && p < m.r32(s + 0x2c)) {
        return;
    }
    let c = m.r32(p);
    if c != 0x27 && c != 0x28 {
        return;
    }
    let a = m.r32(s + 0x24);
    if a != 0 && kind(m, a, 0x34) {
        let a = m.r32(s + 0x24);
        m.call(0x001e_ecd0, &[a + 0x26c], &[]);
    }
}

/// FUN_001e6648(dma_chain): read the last word written (stack[count-1])
/// VFIX: the asm returns the word in $f0 (lwc1); returned as (bits, [f, ..]) so m.call users of both `.0`
/// and `.1[0]` see it (the adapter used to give `.1` = 0, so float callers read length 0)
pub fn ee_sub_1e6648<M: QMem + ?Sized>(m: &mut M, p: u32) -> (u32, V4) {
    let idx = m.r32(p + 0xe8);
    let buf = m.r32(p + 0xf0);
    let w = m.r32(buf.wrapping_add(idx.wrapping_mul(4)).wrapping_sub(4));
    (w, [f32::from_bits(w), 0.0, 0.0, 0.0])
}
