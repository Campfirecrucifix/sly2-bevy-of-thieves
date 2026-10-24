//! WIRE-JOB2: spawner copy scoring (class C2d5dc8, JT +0x2010 = oid 0x21d, bound by FUN_001885b8 row 158 of
//! DAT_002be990: flags 0x300, member 0x2010). Read by the gadget 0x17 can-use FUN_0018d760 via FUN_001cb2c0.
use crate::engine::classes::q::q14_h::{ee_sub_1ca930, ee_sub_1ca958};
use crate::engine::classes::q::q2mem::QMem;

/// FUN_001ca978(o, rec): copy score. rec+8 set -> 0; rec+0 == 0 -> 4; else min over the copy's
/// members (o+0xe0: [0] n, [1] list, member (ca930(rec) * n + i)) of 3 (not in world) / 2 (in world,
/// FUN_001a61a0 0) and returns 1 at the first member in the root set; 5 with no members.
pub fn ee_sub_1ca978<M: QMem + ?Sized>(m: &mut M, o: u32, rec: u32) -> i32 {
    if m.r32(rec + 8) != 0 {
        return 0;
    }
    if m.r32(rec) == 0 {
        return 4;
    }
    let c = ee_sub_1ca930(m, o, rec as i32);
    let mut s1 = 5;
    let mut i = 0i32;
    loop {
        let t = m.r32(o + 0xe0);
        let n = m.r32(t) as i32;
        if i >= n {
            return s1;
        }
        let l = m.r32(t + 4);
        let e = m.r32(l.wrapping_add((c.wrapping_mul(n).wrapping_add(i) as u32) << 2));
        let mut v = 3;
        if (m.r64(e + 0x28) >> 60) & 1 != 0 {
            if crate::engine::game::q_wireq2::ee_sub_1a61a0(m, e) != 0 {
                return 1;
            }
            v = 2;
        }
        if v < s1 {
            s1 = v;
        }
        i += 1;
    }
}

/// FUN_001cade0(o, min): o+0xe4 == 0 && o+0xe8 (current copy) -> it while it scores >= 3 (else 0);
/// otherwise the last copy record in [t+0xc, t+8) whose score >= the running minimum (0 = none).
pub fn ee_sub_1cade0<M: QMem + ?Sized>(m: &mut M, o: u32, min: i32) -> u32 {
    if m.r32(o + 0xe4) == 0 {
        let cur = m.r32(o + 0xe8);
        if cur != 0 {
            return if ee_sub_1ca978(m, o, cur) < 3 { 0 } else { m.r32(o + 0xe8) };
        }
    }
    let mut min = min;
    let mut best = 0u32;
    let t = m.r32(o + 0xe0);
    let mut i = m.r32(t + 0xc) as i32;
    loop {
        let t = m.r32(o + 0xe0);
        if i >= m.r32(t + 8) as i32 {
            break;
        }
        let rec = ee_sub_1ca958(m, o, i) as u32;
        let s = ee_sub_1ca978(m, o, rec);
        if s >= min {
            best = rec;
            min = s;
        }
        i += 1;
    }
    best
}

/// FUN_0020e3c0(world) (level start): resolve the fixed save-variable handles into world +0x3b1c..+0x3b54 and
/// the 9 episode handles (+0x3b58 / +0x3b7c), store "flags/lang" = DAT_002e1e54, "mission" (0x3b24) =
/// FUN_00210a88, "episode" (0x3b28) = FUN_00210ae0 into the save store 0x3d4a60, raise "episodeBest" to the
/// episode, then FUN_001619a0(0x2deb30, 3, 0).
pub fn ee_sub_20e3c0<M: QMem + ?Sized>(m: &mut M, w: u32) {
    use crate::engine::classes::q::q2mem::stack_push;
    use crate::engine::classes::q::q8_runs_b::{ee_sub_1d88d0, ee_sub_1d8b20};
    let h = |m: &mut M, p1: u32, a: &[u32]| {
        let mut v = [0u32; 7];
        v[..a.len()].copy_from_slice(a);
        ee_sub_1d88d0(m, p1, v)
    };
    for (off, p1, a) in [
        (0x3b1c, 0x002a_8968, &[][..]),
        (0x3b24, 0x002a_8970, &[][..]),
        (0x3b28, 0x002a_8980, &[][..]),
        (0x3b20, 0x002a_8990, &[0x002a_8998][..]),
        (0x3b2c, 0x002a_89a0, &[0x002a_89a8][..]),
        (0x3b30, 0x002a_89a0, &[0x002a_89b0][..]),
        (0x3b34, 0x002a_89a0, &[0x002a_89c0][..]),
        (0x3b38, 0x002a_89a0, &[0x002a_89d0][..]),
        (0x3b3c, 0x002a_89a0, &[0x002a_89e0][..]),
        (0x3b40, 0x002a_89a0, &[0x002a_89f0][..]),
        (0x3b44, 0x002a_89a0, &[0x002a_8a00][..]),
        (0x3b48, 0x002a_89a0, &[0x002a_8a10][..]),
        (0x3b4c, 0x002a_89a0, &[0x002a_8a20][..]),
        (0x3b50, 0x002a_89a0, &[0x002a_8a38][..]),
        (0x3b54, 0x002a_89a0, &[0x002a_8a48][..]),
    ] {
        let r = h(m, p1, a);
        m.w32(w + off, r);
    }
    for i in 0..9u32 {
        let name = m.r32(0x002a_39e8 + 4 * i);
        let r = h(m, 0x002a_8a58, &[name, 0x002a_8968]);
        m.w32(w + 0x3b58 + 4 * i, r);
        let r = h(m, 0x002a_8a58, &[name, 0x002a_8a60]);
        m.w32(w + 0x3b7c + 4 * i, r);
    }
    let set = |m: &mut M, handle: u32, v: u32| {
        let c0 = stack_push(m, 0x30);
        let (b0, a0) = (c0 + 0x10, c0 + 0x20);
        m.w32(c0, 0x15);
        m.w32(c0 + 4, v);
        // FUN_001291a8: copy + FUN_00129df8 (int values hold no reference)
        m.w32(b0, 0x15);
        m.w32(b0 + 4, v);
        m.call(0x0025_8200, &[a0, handle, b0, 0x003d_4a60], &[]);
        for x in [a0, b0, c0] {
            m.call(0x0012_9ec8, &[x], &[]);
        }
    };
    let lang = m.r32(0x002e_1e54);
    let hl = m.r32(w + 0x3b2c);
    set(m, hl, lang);
    let mis = m.r32(0x003e_1110);
    let hm = m.r32(w + 0x3b24);
    set(m, hm, mis);
    let ep = ee_sub_210ae0(m);
    let he = m.r32(w + 0x3b28);
    set(m, he, ep);
    let ep = ee_sub_210ae0(m);
    let best = crate::engine::game::q_0020f948::ee_sub_210b28(m);
    if (best as i32) < ep as i32 {
        let c = stack_push(m, 0x20);
        m.w32(c + 0x10, 0x15);
        m.w32(c + 0x14, ep);
        ee_sub_1d8b20(m, c, c + 0x10, 0x002a_8a78, [0; 5]);
        m.call(0x0012_9ec8, &[c], &[]);
        m.call(0x0012_9ec8, &[c + 0x10], &[]);
    }
    m.call(0x0016_19a0, &[0x002d_eb30, 3, 0], &[]);
}

/// FUN_00210ae0: the episode of the current mission (DAT_003e1c44[DAT_003e1110]), -1 without one
pub fn ee_sub_210ae0<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let mis = m.r32(0x003e_1110);
    if mis == u32::MAX {
        return u32::MAX;
    }
    let t = m.r32(0x003e_1c44);
    m.r32(t.wrapping_add(mis.wrapping_mul(4)))
}
