//! WP-Q10 file packages (run_00131990 and following): small engine helpers ported 1:1.
//! Hidden register args were checked in the disassembly (FUN_00131968 gets a0; FUN_00131b10's a2 is the entry;
//! both FUN_0020efc0 / FUN_0020f030 in FUN_00131990 get a1 = 3 from the shared delay slot).
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem};

const WORLD: u32 = 0x003d_d490;

/// FUN_00131990(o, v): when FUN_00131968(o) == 0 and v != o[+0x58]: v == 0 -> FUN_0020f030(world,3), else
/// FUN_0020efc0(world,3); o[+0x58] = v
pub fn fun_00131990<M: QMem + ?Sized>(m: &mut M, o: u32, v: u32) {
    if m.call(0x0013_1968, &[o], &[]).0 != 0 || v == m.r32(o + 0x58) {
        return;
    }
    if v == 0 {
        m.call(0x0020_f030, &[WORLD, 3], &[]);
    } else {
        m.call(0x0020_efc0, &[WORLD, 3], &[]);
    }
    m.w32(o + 0x58, v);
}

/// FUN_00131a40(o, v): for each node of the chain o[+0x34] (next +0x28): node[+0xc] = v; FUN_002301d0(node[4], node[8])
pub fn fun_00131a40<M: QMem + ?Sized>(m: &mut M, o: u32, v: u32) {
    let mut n = m.r32(o + 0x34);
    while n != 0 {
        m.w32(n + 0xc, v);
        let (a, b) = (m.r32(n + 4), m.r32(n + 8));
        m.call(0x0023_01d0, &[a, b], &[]);
        n = m.r32(n + 0x28);
    }
}

/// FUN_00131a90(v, o): nodes of o[+0x34] passing FUN_0012cee0 whose node[8]->+0x98 (i16) == 7:
/// FUN_002303a0(node[4], node[8], 9); node[8]->+0xa8 = v
pub fn fun_00131a90<M: QMem + ?Sized>(m: &mut M, v: u32, o: u32) {
    let mut n = m.r32(o + 0x34);
    while n != 0 {
        if m.call(0x0012_cee0, &[n], &[]).0 != 0 {
            let t = m.r32(n + 8);
            if m.r16(t + 0x98) as i16 == 7 {
                let a = m.r32(n + 4);
                m.call(0x0023_03a0, &[a, t, 9], &[]);
                let t = m.r32(n + 8);
                m.w32(t + 0xa8, v);
            }
        }
        n = m.r32(n + 0x28);
    }
}

/// FUN_00131b10(key, a1, obj): (obj is the 0x100-byte table entry) first node of its chain [+0x90] (next +0x10) with entry[0] == 0 and
/// entry[+0x24] == key: entry[+0xc] = -1; FUN_002301d0(a1, obj)
pub fn fun_00131b10<M: QMem + ?Sized>(m: &mut M, key: u32, a1: u32, obj: u32) {
    let mut e = m.r32(obj + 0x90);
    while e != 0 {
        if m.r32(e) == 0 && m.r32(e + 0x24) == key {
            m.w32(e + 0xc, u32::MAX);
            m.call(0x0023_01d0, &[a1, obj], &[]);
            return;
        }
        e = m.r32(e + 0x10);
    }
}

/// FUN_00131b60(key, obj, count, pairs): for every 0x100-byte entry of obj[+0x54] (obj[+0x50] of them) look for a
/// (id, val) pair (8 B each, `count` of them) with id == entry[+0x9c] or id == -1 whose val == entry[+0x94] or
/// val == -1; on a hit FUN_00131b10(key, obj, entry)
pub fn fun_00131b60<M: QMem + ?Sized>(m: &mut M, key: u32, obj: u32, count: u32, pairs: u32) {
    let mut i = 0u32;
    while (i as i32) < m.r32(obj + 0x50) as i32 {
        let ent = m.r32(obj + 0x54).wrapping_add(i << 8);
        i += 1;
        let mut j = 0u32;
        while (j as i32) < count as i32 {
            let q = pairs.wrapping_add(j << 3);
            let id = m.r32(q);
            let val = m.r32(q + 4);
            if id != m.r32(ent + 0x9c) && id != u32::MAX {
                j += 1;
                continue;
            }
            if val == m.r32(ent + 0x94) || val == u32::MAX {
                fun_00131b10(m, key, obj, ent);
                break;
            }
            j += 1;
        }
    }
}

/// FUN_0013a688(o): save-record writer on stream 0x2dea28: record 5 (4 words) and record 6 (flag/count/id words),
/// then the arrays. The two stack out-pointers (sp, sp+4) receive the record buffers from FUN_00226e88.
pub fn fun_0013a688<M: QMem + ?Sized>(m: &mut M, o: u32) {
    const S: u32 = 0x002d_ea28;
    let sp = stack_push(m, 0x10);
    m.call(0x0022_6e88, &[S, 0xc, 1, 5, 0, sp], &[]);
    let p = m.r32(sp);
    let v = m.r32(0x002d_e040);
    m.w32(p, v);
    let v = m.r32(0x002d_e054);
    m.w32(p + 4, v);
    let v = m.r32(o + 0x440);
    m.w32(p + 8, v);
    let v = m.r32(0x002d_ded0);
    m.w32(p + 12, v);
    m.call(0x0022_6e88, &[S, 0xc, 1, 6, 0, sp + 4], &[]);
    let q = m.r32(sp + 4);
    stack_pop(m, 0x10);
    m.w32(q, 0);
    if m.r32(o + 0x434) != 0 {
        let v = m.r32(q) | 1;
        m.w32(q, v);
    }
    let n = m.r32(o + 0x3b0);
    m.w32(q + 4, n);
    let v = m.r32(o + 0x1a0);
    m.w32(q + 8, v);
    let v = m.r32(0x002d_df10);
    m.w32(q + 12, v);
    m.call(0x0022_6f28, &[S, 0xe, n, o + 0x3b4, 0x30u32.wrapping_sub(n)], &[]);
    m.call(0x0022_6f28, &[S, 8, 1, o + 0x130, 1], &[]);
    m.call(0x0022_6f28, &[S, 8, 1, o + 0x110, 2], &[]);
    m.call(0x0022_6f28, &[S, 8, 1, o + 0x140, 3], &[]);
    m.call(0x0022_6f28, &[S, 8, 1, o + 0x150, 4], &[]);
    let a = m.r32(o + 0x438);
    m.call(0x0022_6f28, &[S, 0xc, 4, a, 0x30], &[]);
    let id = m.r32(o + 0x1a0);
    m.call(0x0022_7010, &[S, 0xc, id, o + 0x1b0, 0xf, 0], &[]);
}
/// FUN_0013a830(a0)
pub fn fun_0013a830<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0016_0918, &[a0, 0x25_a0c0, 0x25_af90], &[]);
}
/// FUN_0013a858(a0)
pub fn fun_0013a858<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0015_74b8, &[a0], &[]);
    m.call(0x0022_6d28, &[a0], &[]);
    m.call(0x0015_76f0, &[a0], &[]);
}

/// FUN_001585d8(o, a, b, c, d) -> o: FUN_0011a550(o, a << 1); o[0x14..0x17] = a,b,c,d;
/// o[0x18] = FUN_001ae378({o[0] * 0x60, 0x10})
pub fn fun_001585d8<M: QMem + ?Sized>(m: &mut M, o: u32, a: u32, b: u32, c: u32, d: u32) -> u32 {
    m.call(0x0011_a550, &[o, a << 1], &[]);
    m.w32(o + 0x50, a);
    let sp = stack_push(m, 0x10);
    let n = m.r32(o).wrapping_mul(0x60);
    m.w32(sp, n);
    m.w32(o + 0x54, b);
    m.w32(o + 0x58, c);
    m.w32(o + 0x5c, d);
    m.w32(sp + 4, 0x10);
    let r = m.call(0x001a_e378, &[sp], &[]).0;
    stack_pop(m, 0x10);
    m.w32(o + 0x60, r);
    o
}

/// FUN_00158958(o, p) -> o[0x60] + ((p - o[8]) / 0x28 via the *0xcccccccd >> 3 idiom) * 0x60
pub fn fun_00158958<M: QMem + ?Sized>(m: &mut M, o: u32, p: u32) -> u32 {
    let d = (p.wrapping_sub(m.r32(o + 8)) as i32).wrapping_mul(-0x3333_3333) >> 3;
    m.r32(o + 0x60).wrapping_add((d as u32).wrapping_mul(0x60))
}

/// FUN_00174898(obj) -> id: the 0x38-tagged owner of obj (FUN_00132b10(**(obj+0x14), 0x38)), else (when
/// FUN_00132b10(obj, 0x4f)) that of obj[+0x200], else 0
pub fn fun_00174898<M: QMem + ?Sized>(m: &mut M, obj: u32) -> u32 {
    if obj == 0 {
        return 0;
    }
    let first = |m: &mut M, o: u32| {
        let p = m.r32(o + 0x14);
        m.r32(p)
    };
    let a = first(m, obj);
    if m.call(0x0013_2b10, &[a, 0x38], &[]).0 != 0 {
        return a;
    }
    if m.call(0x0013_2b10, &[obj, 0x4f], &[]).0 == 0 {
        return 0;
    }
    let o2 = m.r32(obj + 0x200);
    let b = first(m, o2);
    if m.call(0x0013_2b10, &[b, 0x38], &[]).0 == 0 {
        return 0;
    }
    b
}

/// FUN_001749c0(o, st): state change on o[+0x30] (0 = release/start the +0x40 handle, 1 = stop), o[+0x34] = 1.0 (DAT_002dded0)
pub fn fun_001749c0<M: QMem + ?Sized>(m: &mut M, o: u32, st: u32) {
    if m.r32(o + 0x30) == st {
        return;
    }
    m.w32(o + 0x30, st);
    let one = m.rf(0x002d_ded0);
    m.wf(o + 0x34, one);
    let own = m.r32(o + 0x10);
    if st == 0 {
        m.call(0x0020_9788, &[own + 0xfd0, o], &[]);
        let h = m.r32(o + 0x40);
        let hv = m.r32(h);
        let mut fv = 0.0f32;
        if hv == 0 && m.call(0x0020_9a68, &[own, 9], &[]).0 != 0 {
            let own = m.r32(o + 0x10);
            if m.rf(0x002d_ded0) == m.rf(own + 0xff4) {
                let h = m.r32(o + 0x40);
                let p = m.r32(h + 4);
                fv = m.rf(p + 0x74) * m.rf(own + 0xff0);
            }
        }
        let h = m.r32(o + 0x40);
        let own = m.r32(o + 0x10);
        let fl = m.r32(h + 0xc) | 1;
        m.call(0x0020_9278, &[h, own, fl], &[[fv, 0.0, 0.0, 0.0]]);
    } else if st == 1 {
        m.call(0x001e_f038, &[o], &[]);
        let h = m.r32(o + 0x40);
        let own = m.r32(o + 0x10);
        if h == 0 {
            m.w32(own + 0x1a4, 0);
            return;
        }
        if m.r32(h) == 0 && m.call(0x0020_9a68, &[own, 9], &[]).0 != 0 {
            let (p4, p8) = (m.r32(h + 4), m.r32(h + 8));
            let v = m.rf(p8 + 0x1c) / m.rf(p4 + 0x74);
            m.wf(own + 0xff0, v);
            let one = m.rf(0x002d_ded0);
            m.wf(own + 0xff4, one);
        }
        m.call(0x0020_9360, &[h], &[]);
        m.w32(o + 0x40, 0);
        m.call(0x0020_9790, &[own + 0xfd0, o], &[]);
        m.w32(own + 0x1a4, 0);
    }
}

/// FUN_00174b68(o): FUN_001eef98(o); while state o[+0x30] changes: in state 0 FUN_00209430(o[+0x40]) != 0 -> FUN_001749c0(o, 1)
pub fn fun_00174b68<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef98, &[o], &[]);
    let mut s = m.r32(o + 0x30);
    loop {
        if s == 0 {
            let h = m.r32(o + 0x40);
            if m.call(0x0020_9430, &[h], &[]).0 != 0 {
                fun_001749c0(m, o, 1);
            }
        }
        if s == m.r32(o + 0x30) {
            break;
        }
        s = m.r32(o + 0x30);
    }
}

/// FUN_00211a80(a0): record writer for the 0x25afa0 table
pub fn fun_00211a80<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0016_0918, &[a0, 0x25_afa0, 0x25_b870], &[]);
    m.call(0x0015_74b8, &[a0], &[]);
    m.call(0x0022_6d10, &[a0], &[]);
    m.call(0x0015_7598, &[a0, 0x2d_f800], &[]);
    m.call(0x0022_6ca8, &[a0, 0, 0], &[]);
    for (a, b, c, d) in [
        (0xc, 7, 0x44_21a0, 0xd),
        (0xc, 2, 0x44_24b0, 0x15),
        (0xc, 2, 0x44_24d0, 0x17),
        (0xc, 3, 0x44_24f0, 0x19),
        (0xc, 4, 0x44_2520, 0x1c),
        (0xc, 3, 0x44_25a0, 0x20),
        (0xc, 4, 0x44_2560, 0x23),
    ] {
        m.call(0x0022_6f28, &[a0, a, b, c, d], &[]);
    }
    m.call(0x0022_6ca8, &[a0, 0x32, 0x1e7], &[]);
    m.call(0x0022_6ce0, &[a0, 0xc0], &[]);
    m.call(0x0022_6d28, &[a0], &[]);
    m.call(0x0022_70c8, &[a0, 0x27, 0x44_2240, 0], &[]);
    m.call(0x0022_6d28, &[a0], &[]);
    m.call(0x0022_6ce0, &[a0, 0x100], &[]);
    m.call(0x0022_6d28, &[a0], &[]);
    m.call(0x0015_76f0, &[a0], &[]);
}
/// FUN_00211c10(a0)
pub fn fun_00211c10<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x0015_74b8, &[a0], &[]);
    m.call(0x0022_6ce0, &[a0, 0xc0], &[]);
    m.call(0x0022_6d28, &[a0], &[]);
    m.call(0x0015_76f0, &[a0], &[]);
}

/// FUN_0018de18(o, p): p == 0 -> when o[+0x4c8] == 6 virtual slot 0x2c4 (slot 87) with 0; else o[+0x9c0] = 1 and
/// FUN_001928f8(o + 0x4c0, 6, p, 0)
pub fn fun_0018de18<M: QMem + ?Sized>(m: &mut M, o: u32, p: u32) {
    if p == 0 {
        if m.r32(o + 0x4c8) == 6 {
            crate::engine::classes::q::q2mem::vslot(m, o, 87, &[0], &[]);
        }
    } else {
        m.w32(o + 0x9c0, 1);
        m.call(0x0019_28f8, &[o + 0x4c0, 6, p, 0], &[]);
    }
}
/// FUN_0018df58(o, q): o[+0x1fa0..+0x1fac] = quadword q (a1)
pub fn fun_0018df58<M: QMem + ?Sized>(m: &mut M, o: u32, q: [u32; 4]) {
    for (i, w) in q.into_iter().enumerate() {
        m.w32(o + 0x1fa0 + 4 * i as u32, w);
    }
}
/// FUN_0018e330(): FUN_0015e9c8(world, 0x505, 0x38, 0, 0x20, buf[32]) entries -> FUN_001f1900(entry) for each
pub fn fun_0018e330<M: QMem + ?Sized>(m: &mut M) {
    let buf = stack_push(m, 0x80);
    let mut n = m.call(0x0015_e9c8, &[WORLD, 0x505, 0x38, 0, 0x20, buf], &[]).0 as i32;
    if n > 0 {
        let mut p = buf;
        loop {
            n -= 1;
            p += 4;
            let e = m.r32(buf);
            m.call(0x001f_1900, &[e], &[]);
            if n == 0 {
                break;
            }
            let v = m.r32(p);
            m.w32(buf, v);
        }
    }
    stack_pop(m, 0x80);
}

/// FUN_001b1b10(i): i < 6 (unsigned 64-bit)
pub fn fun_001b1b10(i: u64) -> bool {
    i < 6
}
/// FUN_001b1b18(o, i, v, w): slots 0,1,3,4 of the 0xc-byte table at o + 0x17c: [0] = v, [2] = w unless w == -1
pub fn fun_001b1b18<M: QMem + ?Sized>(m: &mut M, o: u32, i: u64, v: u32, w: i64) {
    if fun_001b1b10(i) && i != 2 && i != 5 {
        let p = o.wrapping_add((i as u32).wrapping_mul(0xc)).wrapping_add(0x17c);
        m.w32(p, v);
        if w != -1 {
            m.w32(p + 8, w as u32);
        }
    }
}

fn walk_records<M: QMem + ?Sized>(m: &mut M, o: u32, key: u32, f: f32, all_fn: u32, rec_fn: u32) {
    // VER4: 1:1 from asm (o = a0, key = a1, f = f12; rec_fn(rec, idx, 0, f), all_fn(o+0x84, f))
    if key == 0x314 {
        let a = m.r32(o + 0x84);
        m.call(all_fn, &[a], &[[f, 0.0, 0.0, 0.0]]);
        return;
    }
    let mut i = 0u32;
    while i < m.r16(o + 0x80) as u32 {
        let rec = m.r32(o + 0x84).wrapping_add(i.wrapping_mul(0x30));
        i += 1;
        let mut j = 0u32;
        while j < m.r16(rec) as u32 {
            let tp = m.r32(rec + 4).wrapping_add(j.wrapping_mul(0x18));
            j += 1;
            if key != m.r32(tp.wrapping_add(0x14)) {
                continue;
            }
            let n = m.r16(rec + 2) as u32;
            let mut idx = -1i32;
            let mut ps = m.r32(rec + 8).wrapping_add(8);
            for k in 0..n {
                if m.r16(ps) as i16 != -1 {
                    idx = k as i32;
                }
                ps = ps.wrapping_add(0x18);
            }
            if idx >= 0 {
                m.call(rec_fn, &[rec, idx as u32, 0], &[[f, 0.0, 0.0, 0.0]]);
            }
        }
    }
}
/// FUN_00212e40(o, key, f)
pub fn fun_00212e40<M: QMem + ?Sized>(m: &mut M, o: u32, key: u32, f: f32) {
    walk_records(m, o, key, f, 0x0021_7fb8, 0x0021_7fe0)
}
/// FUN_00212f80(o, key, f)
pub fn fun_00212f80<M: QMem + ?Sized>(m: &mut M, o: u32, key: u32, f: f32) {
    walk_records(m, o, key, f, 0x0021_8038, 0x0021_8060)
}
/// FUN_0015e380(base, k) = base + ((k * 0x95675) & 0x1ff) * 0xc + 0x38
pub fn fun_0015e380(base: u32, k: u32) -> u32 {
    base.wrapping_add((k.wrapping_mul(0x95675) & 0x1ff).wrapping_mul(0xc)).wrapping_add(0x38)
}
/// FUN_0015e3a8(base, k) = base + k * 0xc + 0x3038
pub fn fun_0015e3a8(base: u32, k: u32) -> u32 {
    base.wrapping_add(k.wrapping_mul(0xc)).wrapping_add(0x3038)
}

/// FUN_0015ed78(a0, a1, a2, a3) -> first result word of FUN_0015e9c8(a0, a1 | 0x200, a2, a3, 1, buf), 0 when it returns 0
pub fn fun_0015ed78<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x10);
    let r = m.call(0x0015_e9c8, &[a0, a1 | 0x200, a2, a3, 1, sp], &[]).0;
    if r == 0 {
        m.w32(sp, 0);
    }
    let v = m.r32(sp);
    stack_pop(m, 0x10);
    v
}
/// FUN_0014a178(o, a, b): push (a, b) onto the 0xc-byte array o[+0x58] while o[+0x54] < 0x100
pub fn fun_0014a178<M: QMem + ?Sized>(m: &mut M, o: u32, a: u32, b: u32) {
    let n = m.r32(o + 0x54);
    if (n as i32) < 0x100 {
        let p = m.r32(o + 0x58).wrapping_add(n.wrapping_mul(0xc));
        m.w32(p + 4, b);
        m.w32(p, a);
        m.w32(o + 0x54, n.wrapping_add(1));
    }
}
/// FUN_001594d8(a0)
pub fn fun_001594d8<M: QMem + ?Sized>(m: &mut M, a0: u32) {
    m.call(0x001c_4848, &[WORLD, a0], &[]);
}

/// FUN_0022c818(_, o): empty-list init: o[+0x88] = &o[+0x80], o[+0x8c] = &o[+0x84], o[+0x84] = 1.0, o[+0x80] = 0
/// VFIX: the object is a1 (asm uses $a1 only); a0 is unused
pub fn fun_0022c818<M: QMem + ?Sized>(m: &mut M, _a0: u32, o: u32) {
    m.w32(o + 0x88, o + 0x80);
    m.w32(o + 0x8c, o + 0x84);
    m.w32(o + 0x84, 0x3f80_0000);
    m.w32(o + 0x80, 0);
}
