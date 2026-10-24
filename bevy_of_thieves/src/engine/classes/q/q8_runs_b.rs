//! WP-Q8: small file packages (run_00141968 .. run_001f7bf8), 1:1 on `QMem`.
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem};
use crate::engine::classes::q::q6::{callf, ro32, rof, s, set_tagged, vcall, CLOCK};
use crate::engine::classes::q::q8_runs_a::pools_init;

// ---------------------------------------------------------------- run_00143af0
/// FUN_00143af0: l = FUN_00143410(+0x18); if l: {0x15, p2} -> FUN_001291a8 copy; FUN_00258200(buf, l, copy, 0x3d4a60);
/// the three temporaries destroyed with FUN_00129ec8; then {0x15, -1} + virtual 0x33c -> FUN_001d8b20(out, key, v,
/// 0x2a4db0, 0) and both destroyed; returns l
pub fn ee_sub_143af0<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32) -> u32 {
    let a = stack_push(m, 0x10);
    let b = stack_push(m, 0x10);
    let c = stack_push(m, 0x10);
    let p18 = m.r32(o + 0x18);
    let l = m.call(0x0014_3410, &[p18], &[]).0;
    if l != 0 {
        m.w32(c, 0x15);
        m.w32(c + 4, p2);
        m.call(0x0012_91a8, &[b, c], &[]);
        m.call(0x0025_8200, &[a, l, b, 0x003d_4a60], &[]);
        m.call(0x0012_9ec8, &[a], &[]);
        m.call(0x0012_9ec8, &[b], &[]);
        m.call(0x0012_9ec8, &[c], &[]);
    }
    m.w32(b, 0x15);
    m.w32(b + 4, u32::MAX);
    let v = vcall(m, o, 0xc, 0x33c, &[], &[]).0;
    m.call(0x001d_8b20, &[c, b, v, 0x002a_4db0, 0], &[]);
    m.call(0x0012_9ec8, &[b], &[]);
    m.call(0x0012_9ec8, &[c], &[]);
    stack_pop(m, 0x30);
    l
}
/// FUN_00143bd8: r = FUN_001d8a38(buf, vslot 0x33c, 0x2a4db0, 0); destroy buf; if buf[1] not in {0, -1}:
/// (DAT_003e1084 & 0x40) ? FUN_00143af0(o, r) : FUN_001432e0(o, r)
pub fn ee_sub_143bd8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let buf = stack_push(m, 0x10);
    let v = vcall(m, o, 0xc, 0x33c, &[], &[]).0;
    m.call(0x001d_8a38, &[buf, v, 0x002a_4db0, 0], &[]);
    m.call(0x0012_9ec8, &[buf], &[]);
    let r = m.r32(buf + 4);
    stack_pop(m, 0x10);
    if 1 < r.wrapping_add(1) {
        if m.r32(0x003e_1084) & 0x40 == 0 {
            m.call(0x0014_32e0, &[o, r], &[]);
        } else {
            ee_sub_143af0(m, o, r);
        }
    }
}
/// FUN_00143de8: swap the active object at +0x1084
pub fn ee_sub_143de8<M: QMem + ?Sized>(m: &mut M, o: u32, new: u32) {
    let old = m.r32(o + 0x1084);
    if new == old {
        return;
    }
    if old != 0 {
        vcall(m, old, 0xc, 0x14, &[], &[]);
        let st = m.r32(o + 0x4c8) as i32;
        if st < 7 && 4 < st {
            vcall(m, o, 0xc, 0x2c4, &[0], &[]);
        }
    }
    m.w32(o + 0x1084, new);
    if new != 0 {
        vcall(m, new, 0xc, 0xc, &[], &[]);
        m.call(0x001e_ecd0, &[o + 0x26c], &[]);
        if m.call(0x0014_43d8, &[o], &[]).0 != 0 {
            m.call(0x0019_cd38, &[o + 0xf00, u32::MAX], &[]);
        }
        if m.r32(o + 0x11b4) != u32::MAX {
            let cur = m.r32(o + 0x1084);
            let p = m.r32(cur + 0x40);
            if m.r32(p) != 0x1e {
                vcall(m, o, 0xc, 0x2c4, &[0], &[]);
            }
        }
    }
}

// ---------------------------------------------------------------- run_00179d30
/// FUN_00179d30: state change of the mode word at +0x38
pub fn ee_sub_179d30<M: QMem + ?Sized>(m: &mut M, o: u32, new: u32) {
    if new == m.r32(o + 0x38) {
        return;
    }
    let p10 = m.r32(o + 0x10);
    if m.r32(o + 0x38) == 0 {
        let p18 = m.r32(o + 0x18);
        m.call(0x0017_1c18, &[p18, p10 + 0x2dc], &[]);
    }
    m.call(0x001e_f0d8, &[o + 0x18], &[]);
    m.w32(o + 0x38, new);
    let t = m.r32(CLOCK);
    m.w32(o + 0x3c, t);
    let n = new as i32;
    if n < 3 {
        if n < 1 {
            if n == 0 {
                let u = m.call(0x0020_5438, &[p10, 0xa9], &[]).0;
                let p40 = m.r32(o + 0x40);
                m.call(0x0017_1258, &[u, p40, 0, p10 + 0x2dc], &[]);
                vcall(m, u, 0xc, 0x44, &[], &[]);
                ee_sub_179e80(m, o);
                m.w32(o + 0x34, ro32(0x002a_5590));
            }
        } else {
            let u = m.call(0x0020_5438, &[p10, 0x9e], &[]).0;
            vcall(m, u, 0xc, 0x44, &[], &[]);
            ee_sub_179ef8(m, o);
            m.w32(o + 0x30, ro32(0x002a_5590));
        }
    } else if new == 3 {
        let u = m.call(0x0020_5438, &[p10, 0x9e], &[]).0;
        m.w32(u + 0x3c, 1);
        vcall(m, u, 0xc, 0x44, &[], &[]);
    }
}
fn timer<M: QMem + ?Sized>(m: &mut M, o: u32, f: u32, lim: u32, k0: u32, k1: u32) {
    let clk = m.rf(CLOCK);
    let lim = rof(lim);
    if lim <= m.rf(o + f) {
        let p10 = m.r32(o + 0x10);
        let f2 = m.rf(p10 + k0);
        if f2 == lim {
            m.wf(o + f, lim);
        } else {
            let b = m.rf(p10 + k1);
            let r = callf(m, 0x0011_d448, &[], &[s(f2), s(b)]);
            m.wf(o + f, clk + r);
        }
    }
}
/// FUN_00179e80
pub fn ee_sub_179e80<M: QMem + ?Sized>(m: &mut M, o: u32) {
    timer(m, o, 0x30, 0x002a_5594, 0x2cc, 0x2d0)
}
/// FUN_00179ef8
pub fn ee_sub_179ef8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    timer(m, o, 0x34, 0x002a_5598, 0x2d4, 0x2d8)
}

// ---------------------------------------------------------------- run_0017eae8
/// FUN_0017eae8: if (+0x10)->+0xfe8 & 1 and FUN_00174fb8(+0x18) == 0 -> 0; else FUN_001c1fa0(25.0, o + 0x30, +0x10)
pub fn ee_sub_17eae8<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let p10 = m.r32(o + 0x10);
    if m.r32(p10 + 0xfe8) & 1 != 0 {
        let p18 = m.r32(o + 0x18);
        if m.call(0x0017_4fb8, &[p18], &[]).0 == 0 {
            return 0;
        }
    }
    let p10 = m.r32(o + 0x10);
    m.call(0x001c_1fa0, &[o + 0x30, p10], &[s(f32::from_bits(0x41c8_0000))]).0
}
/// FUN_0017ef38 (same body as FUN_0011ae88)
pub fn ee_sub_17ef38<M: QMem + ?Sized>(m: &mut M, p1: u64, p2: u64) {
    pools_init(m, p1, p2)
}
/// FUN_0017f070
pub fn ee_sub_17f070<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_17ef38(m, 1, 0xffff)
}

// ---------------------------------------------------------------- run_001f7bf8
/// FUN_001f7bf8(f12 = x, a0 = p2): x <= 0 -> FUN_001f7d90(0) + FUN_001f7d10(0, p2); else FUN_001f7d90(-1/x, p2)
pub fn ee_sub_1f7bf8<M: QMem + ?Sized>(m: &mut M, x: f32, p2: u32) {
    if x <= 0.0 {
        m.call(0x001f_7d90, &[p2], &[s(0.0)]);
        m.call(0x001f_7d10, &[p2], &[s(0.0)]);
    } else {
        m.call(0x001f_7d90, &[p2], &[s(-1.0 / x)]);
    }
}

// ---------------------------------------------------------------- run_00206db0
/// FUN_00206db0: for the five ids of the table DAT_002c3d10: if FUN_0015e9c8(world, 0x605, id, 0, 0, 0) then
/// FUN_00208920(o, id)
pub fn ee_sub_206db0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    for i in 0..5u32 {
        let id = m.r32(0x002c_3d10 + 4 * i);
        if m.call(0x0015_e9c8, &[0x003d_d490, 0x605, id, 0, 0, 0], &[]).0 != 0 {
            m.call(0x0020_8920, &[o, id], &[]);
        }
    }
}
/// FUN_00206e50: FUN_00208948(o, id) for the five ids of DAT_002c3d10
pub fn ee_sub_206e50<M: QMem + ?Sized>(m: &mut M, o: u32) {
    for i in 0..5u32 {
        let id = m.r32(0x002c_3d10 + 4 * i);
        m.call(0x0020_8948, &[o, id], &[]);
    }
}

// ---------------------------------------------------------------- run_0016c8e0
fn alloc_aligned<M: QMem + ?Sized>(m: &mut M, size: u32) -> u32 {
    let req = stack_push(m, 8);
    m.w32(req, size);
    m.w32(req + 4, 0x10);
    let p = m.call(0x001a_e378, &[req], &[]).0;
    stack_pop(m, 8);
    p
}
/// FUN_0016c8e0: read a table from stream p2 (count n = FUN_00117c20): {n, rows(n * 8: {m, items(m * 6)}), words(n*4)}
pub fn ee_sub_16c8e0<M: QMem + ?Sized>(m: &mut M, _p1: u32, st: u32) -> u32 {
    let n = m.call(0x0011_7c20, &[st], &[]).0;
    if n == 0 {
        return 0;
    }
    let t = alloc_aligned(m, 0xc);
    m.w32(t, n);
    let rows = alloc_aligned(m, n << 3);
    m.w32(t + 4, rows);
    let mut i = 0u32;
    while (m.r32(t) as i32) > i as i32 {
        let row = m.r32(t + 4).wrapping_add(i * 8);
        i += 1;
        let mut k = m.call(0x0011_7c20, &[st], &[]).0;
        let items = alloc_aligned(m, k.wrapping_mul(6));
        m.w32(row + 4, items);
        if (k as i32) > 0 {
            let mut off = 0u32;
            loop {
                k = k.wrapping_sub(1);
                let p = m.r32(row + 4).wrapping_add(off);
                let a = m.call(0x0011_7c20, &[st], &[]).0;
                m.w16(p, a as u16);
                let b = m.call(0x0011_7c20, &[st], &[]).0;
                off += 6;
                m.w16(p + 2, b as u16);
                let c = m.call(0x0011_7bd0, &[st], &[]).0;
                m.w8(p + 4, c as u8);
                m.call(0x001e_d1a8, &[p], &[]);
                m.call(0x001e_d1a8, &[p + 2], &[]);
                if k == 0 {
                    break;
                }
            }
        }
    }
    let n2 = m.call(0x0011_7c20, &[st], &[]).0;
    let w = alloc_aligned(m, n2 << 2);
    m.w32(t + 8, w);
    let mut j = 0u32;
    while (j as i32) < n2 as i32 {
        let v = m.call(0x0011_7d50, &[st], &[]).0;
        let base = m.r32(t + 8);
        m.w32(base.wrapping_add(j * 4), v);
        j += 1;
    }
    t
}

// ---------------------------------------------------------------- run_001a9b28 (1aa0b0)
/// FUN_001aa0b0: collect the contact nodes returned by vslot 0x1e4 of (p2->+0x60) into the list at p3 + 0x6c
pub fn ee_sub_1aa0b0<M: QMem + ?Sized>(m: &mut M, p1: u32, p2: u32, p3: u32, p4: u32, p5: u32, p6: u32, p7: u32, p8: u32) {
    let buf = stack_push(m, 0x10);
    m.w32(buf, 0);
    let tgt = if m.r32(p4 + 0x18) == 0 { p8 } else { buf };
    let obj = m.r32(p2 + 0x60);
    vcall(m, obj, 0xc, 0x1e4, &[p4, p5, p6, p7, tgt], &[]);
    let head = m.r32(buf);
    if head != 0 {
        let mut node = head;
        let mut first = m.r32(node);
        loop {
            let sel = (first == p4) as u32;
            let a = m.r32(node + (sel * 0x10 + 3) * 4);
            let b = m.r32(p4 + 0x18);
            if m.call(0x001f_7530, &[a, b], &[]).0 != 0 {
                let n = m.call(0x0011_ccc0, &[p1 + 0x3d40], &[]).0;
                for w in 0..0x30u32 {
                    let v = m.r32(node + 4 * w);
                    m.w32(n + 4 * w, v);
                }
                let h = m.r32(p3 + 0x6c);
                m.w32(n + 0x2b * 4, h);
                m.w32(p3 + 0x6c, n);
            }
            node = m.r32(node + 0x2b * 4);
            if node == 0 {
                break;
            }
            first = m.r32(node);
        }
    }
    let head = m.r32(buf);
    stack_pop(m, 0x10);
    if head != 0 {
        m.call(0x001a_9fe0, &[p1, head, p8], &[]);
    }
}

// ---------------------------------------------------------------- run_001b9880
/// FUN_001b9880: FUN_001b6078(o, 0xf)
pub fn ee_sub_1b9880<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001b_6078, &[o, 0xf], &[]);
}
/// FUN_001b9c00: +0x1530 = p2, +0x1534 = p4; FUN_001b6078(o, 6); when p3 != -1 push it to +0x1530 (if set) and
/// +0x1034 with FUN_001f0660 (p5 == 0) / FUN_001f0948
pub fn ee_sub_1b9c00<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32, p3: u64, p4: u32, p5: u64) {
    m.w32(o + 0x1530, p2);
    m.w32(o + 0x1534, p4);
    m.call(0x001b_6078, &[o, 6], &[]);
    if p3 != u64::MAX {
        let t = m.r32(o + 0x1530);
        let f = if p5 == 0 { 0x001f_0660 } else { 0x001f_0948 };
        if t != 0 {
            m.call(f, &[t, p3 as u32], &[]);
        }
        let u = m.r32(o + 0x1034);
        m.call(f, &[u, p3 as u32], &[]);
    }
}
/// FUN_001b9ca0: FUN_001eecd0(o + 0x26c); p2 != 0 -> FUN_00205438(o, 0xc6) vslot 0x44
pub fn ee_sub_1b9ca0<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u64) {
    m.call(0x001e_ecd0, &[o + 0x26c], &[]);
    if p2 != 0 {
        let u = m.call(0x0020_5438, &[o, 0xc6], &[]).0;
        vcall(m, u, 0xc, 0x44, &[], &[]);
    }
}
/// FUN_001b9d00: +0x13e8 = 0; FUN_001b6078(o, 3)
pub fn ee_sub_1b9d00<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.w32(o + 0x13e8, 0);
    m.call(0x001b_6078, &[o, 3], &[]);
}
/// FUN_001b9d28 (a1 = quadword passed through to vslot 0xec)
pub fn ee_sub_1b9d28<M: QMem + ?Sized>(m: &mut M, o: u32, q: crate::engine::classes::q::q2mem::V4) {
    if m.call(0x001d_aea8, &[0x11], &[]).0 != 0 {
        let p = m.r32(o + 0x1ad0);
        if (m.r32(p + 0x2c) >> 28) & 1 == 0 {
            vcall(m, p, 0xc, 0xec, &[], &[q]);
            let (a, b) = (m.r32(o + 0x1ad4), m.r32(o + 0x1ad0));
            m.call(0x0012_f9b8, &[a, b, 0, 0], &[super::q5util::f4(0.0), super::q5util::f4(1.0)]);
        }
    }
}
/// FUN_001b9db8
pub fn ee_sub_1b9db8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let p = m.r32(o + 0x1ad8);
    if (m.r32(p + 0x2c) >> 28) & 1 == 0 {
        vcall(m, p, 0xc, 0xec, &[], &[]);
        let (a, b) = (m.r32(o + 0x1adc), m.r32(o + 0x1ad8));
        m.call(0x0012_f9b8, &[a, b, 0, 0], &[super::q5util::f4(0.0), super::q5util::f4(1.0)]);
    }
}
/// FUN_001b9e28 (same body as FUN_0011ae88)
pub fn ee_sub_1b9e28<M: QMem + ?Sized>(m: &mut M, p1: u64, p2: u64) {
    pools_init(m, p1, p2)
}
/// FUN_001b9f60
pub fn ee_sub_1b9f60<M: QMem + ?Sized>(m: &mut M) {
    pools_init(m, 1, 0xffff)
}
/// FUN_001ba100: FUN_00132ea0(p2->+4) == 0x62
pub fn ee_sub_1ba100<M: QMem + ?Sized>(m: &mut M, p2: u32) -> bool {
    let a = m.r32(p2 + 4);
    m.call(0x0013_2ea0, &[a], &[]).0 == 0x62
}

// ---------------------------------------------------------------- run_001c7988
/// FUN_001c7988: FUN_001c7a08(o); append a 0x2c-byte record (max 0x20) to the array +0x240 (count +0x23c, current +0x244)
pub fn ee_sub_1c7988<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32, p3: u32) {
    m.call(0x001c_7a08, &[o], &[]);
    let n = m.r32(o + 0x23c) as i32;
    if n < 0x20 {
        m.w32(o + 0x23c, (n + 1) as u32);
        let r = m.r32(o + 0x240).wrapping_add((n as u32).wrapping_mul(0x2c));
        m.w32(o + 0x244, r);
        m.w32(r, p2);
        m.w32(r + 4, p3);
        m.w32(r + 0x18, u32::MAX);
        m.w32(r + 0x1c, 0);
        m.w32(r + 0x14, u32::MAX);
    }
}
/// FUN_001c7a10: append p2 to the current record's list (+0x1c count, < 3, items at +0x20)
pub fn ee_sub_1c7a10<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32) {
    let r = m.r32(o + 0x244);
    let n = m.r32(r + 0x1c);
    if n < 3 {
        m.w32(r + n * 4 + 0x20, p2);
        m.w32(r + 0x1c, n + 1);
    }
}
/// FUN_001c7a58: select the record whose key (+0) == p2 (default: the first), FUN_001f6a68(o), then
/// FUN_001f6978(o, item) for its items
pub fn ee_sub_1c7a58<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32) {
    let cnt = m.r32(o + 0x23c) as i32;
    let base = m.r32(o + 0x240);
    let mut found = false;
    for i in 0..cnt {
        let r = base.wrapping_add(i as u32 * 0x2c);
        if m.r32(r) == p2 {
            m.w32(o + 0x244, r);
            found = true;
            break;
        }
    }
    if !found {
        m.w32(o + 0x244, base);
    }
    m.call(0x001f_6a68, &[o], &[]);
    let r = m.r32(o + 0x244);
    let n = m.r32(r + 0x1c) as i32;
    for i in 0..n {
        let it = m.r32(r + 0x20 + 4 * i as u32);
        m.call(0x001f_6978, &[o, it], &[]);
    }
}
/// FUN_001c7d00: return 0x1e
pub fn ee_sub_1c7d00<M: QMem + ?Sized>(_m: &mut M) -> u32 {
    0x1e
}

// ---------------------------------------------------------------- run_001ce6d0 (class factory / ref-count helpers)
/// FUN_001ce6d0: kind p2 (0..=10) -> FUN_002333b4(o, 0, size) + vtable pointer at +0xec; other kinds fall through
/// and store p2 at *p3; returns the object (or p3 for an unknown kind)
pub fn ee_sub_1ce6d0<M: QMem + ?Sized>(m: &mut M, o: u32, kind: u32, p3: u32) -> u32 {
    let (sz, vt) = match kind {
        0 => (0xf0, 0x002d_7508),
        1 => (0x100, 0x002d_7558),
        2 => (0x100, 0x002d_7468),
        3 => (0x100, 0x002d_7738),
        4 => (0x100, 0x002d_75f8),
        5 => (0xf0, 0x002d_74b8),
        6 => (0x110, 0x002d_7648),
        7 => (0xf0, 0x002d_7788),
        8 => (0x100, 0x002d_7698),
        9 => (0x110, 0x002d_75a8),
        10 => (0x100, 0x002d_76e8),
        _ => {
            m.w32(p3, kind);
            return p3;
        }
    };
    m.call(0x0023_33b4, &[o, 0, sz], &[]);
    m.w32(o + 0xec, vt);
    m.w32(o, kind);
    o
}
/// FUN_001ce980: FUN_0011a2e8((+8) + 0x15c, o); FUN_001ce9d0(o)
pub fn ee_sub_1ce980<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let p = m.r32(o + 8);
    m.call(0x0011_a2e8, &[p + 0x15c, o], &[]);
    ee_sub_1ce9d0(m, o);
}
/// FUN_001ce9b8: empty
pub fn ee_sub_1ce9b8<M: QMem + ?Sized>(_m: &mut M) {}
/// FUN_001ce9c0: ++(+4)
pub fn ee_sub_1ce9c0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let n = m.r32(o + 4).wrapping_add(1);
    m.w32(o + 4, n);
}
/// FUN_001ce9d0: --(+4); at 0: vslot 0x1c of the +0xec table, vslot 0x2c(8, o) on every node of the list +0xe8
/// (freed to pool 0x3e1144), then the object itself to pool 0x3e14e4
pub fn ee_sub_1ce9d0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let n = m.r32(o + 4).wrapping_sub(1);
    m.w32(o + 4, n);
    if n != 0 {
        return;
    }
    vcall(m, o, 0xec, 0x1c, &[], &[]);
    let mut node = m.r32(o + 0xe8);
    while node != 0 {
        let obj = m.r32(node);
        vcall(m, obj, 0xc, 0x2c, &[8, o], &[]);
        let next = m.r32(node + 8);
        m.call(0x0011_cd98, &[0x003e_1144, node], &[]);
        node = next;
    }
    m.w32(o + 0xe8, 0);
    m.call(0x0011_cd98, &[0x003e_14e4, o], &[]);
}
/// FUN_001cea88: p2 != 0 -> FUN_001ce980(o)
pub fn ee_sub_1cea88<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u64) {
    if p2 != 0 {
        ee_sub_1ce980(m, o);
    }
}

// ---------------------------------------------------------------- run_001d7748
/// FUN_001d7748: append {p2, p3, p4} (12 bytes, max 12) at +0x190, count +0x18c
pub fn ee_sub_1d7748<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32, p3: u32, p4: u32) {
    let n = m.r32(o + 0x18c);
    if n < 0xc {
        m.w32(o + 0x18c, n + 1);
        let r = o + n * 0xc + 400;
        m.w32(r + 8, p4);
        m.w32(r, p2);
        m.w32(r + 4, p3);
    }
}
/// FUN_001d7780: FUN_002013d8(buf, p2, 6, 6, o + 0x220, o + 0x224); destroy buf; FUN_001d77d0(o, 0)
pub fn ee_sub_1d7780<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32) {
    let buf = stack_push(m, 0x10);
    m.call(0x0020_13d8, &[buf, p2, 6, 6, o + 0x220, o + 0x224], &[]);
    m.call(0x0012_9ec8, &[buf], &[]);
    stack_pop(m, 0x10);
    ee_sub_1d77d0(m, o, 0);
}
/// FUN_001d77d0: step through the entries of (+0x184) from the current one by p2 (wrapping) until one is in the
/// list +0x224 (count +0x220); FUN_001f0660(+0x188) then, or give up with FUN_001f1448
pub fn ee_sub_1d77d0<M: QMem + ?Sized>(m: &mut M, o: u32, mut step: i64) {
    if m.r32(o + 0x188) == 0 {
        return;
    }
    // WIRE-UI: Ghidra dropped the register args: a0 = +0x188 (FUN_001f07e8), a1 = the matched entry (FUN_001f0660)
    let l = m.r32(o + 0x188);
    let u = m.call(0x001f_07e8, &[l], &[]).0;
    let c = m.r32(o + 0x184);
    let start = m.call(0x001f_0438, &[c, u], &[]).0 as i32;
    let mut i = start;
    loop {
        loop {
            i = i.wrapping_add(step as i32);
            let c = m.r32(o + 0x184);
            if i < 0 {
                i = (m.r32(c + 0x50) as i32) - 1;
            } else if (m.r32(c + 0x50) as i32) <= i {
                i = 0;
            }
            let e = m.call(0x001f_04f8, &[c, i as u32], &[]).0;
            let n = m.r32(o + 0x220) as i32;
            for k in 0..n {
                if m.r32(o + 0x224 + 4 * k as u32) == e {
                    let t = m.r32(o + 0x188);
                    m.call(0x001f_0660, &[t, e], &[]);
                    return;
                }
            }
            if step == 0 {
                break;
            }
            if i == start {
                m.call(0x001f_1448, &[0, o, u32::MAX, 0, o + 0x188], &[]);
                return;
            }
        }
        step = 1;
    }
}
/// FUN_001d78d8 / FUN_001d7a30 (same body as FUN_0011ae88)
pub fn ee_sub_1d78d8<M: QMem + ?Sized>(m: &mut M, p1: u64, p2: u64) {
    pools_init(m, p1, p2)
}
pub fn ee_sub_1d7a30<M: QMem + ?Sized>(m: &mut M, p1: u64, p2: u64) {
    pools_init(m, p1, p2)
}
/// FUN_001d7a10 / FUN_001d7b68
pub fn ee_sub_1d7a10<M: QMem + ?Sized>(m: &mut M) {
    pools_init(m, 1, 0xffff)
}
pub fn ee_sub_1d7b68<M: QMem + ?Sized>(m: &mut M) {
    pools_init(m, 1, 0xffff)
}
/// FUN_001d7e80: FUN_001d7da0(p1, p2, &{rec, n}); p3 + u16(rec + 4) + n * DAT_002c2d90[(i8)rec[2]]
pub fn ee_sub_1d7e80<M: QMem + ?Sized>(m: &mut M, p1: u32, p2: u32, p3: u32) -> u32 {
    let buf = stack_push(m, 8);
    m.call(0x001d_7da0, &[p1, p2, buf], &[]);
    let rec = m.r32(buf);
    let n = m.r32(buf + 4);
    let w = m.r16(rec + 4) as u32;
    let k = m.r8(rec + 2) as i8 as i32 as u32;
    let t = m.r32(0x002c_2d90u32.wrapping_add(k.wrapping_mul(4)));
    stack_pop(m, 8);
    p3.wrapping_add(w).wrapping_add(n.wrapping_mul(t))
}

// ---------------------------------------------------------------- run_001fcba8
/// FUN_001fcba8: +0x40 = p1; p1 == 0: FUN_001fb060(o), +0x52 = 0, (+0x50 & 8) == 0 -> vslot 0x3c; else
/// FUN_001fb658(o, 0x1000, 0x1000); finally vslot 0x14
pub fn ee_sub_1fcba8<M: QMem + ?Sized>(m: &mut M, p1: u64, o: u32) {
    m.w32(o + 0x40, p1 as u32);
    if p1 == 0 {
        m.call(0x001f_b060, &[o], &[]);
        m.w16(o + 0x52, 0);
        if m.r16(o + 0x50) & 8 == 0 {
            vcall(m, o, 0xc, 0x3c, &[], &[]);
        }
    } else {
        m.call(0x001f_b658, &[o, 0x1000, 0x1000], &[]);
    }
    vcall(m, o, 0xc, 0x14, &[], &[]);
}
/// FUN_001fcc30: inverse selection of FUN_001fcba8 (no +0x40 store, no vslot 0x3c)
pub fn ee_sub_1fcc30<M: QMem + ?Sized>(m: &mut M, p1: u64, o: u32) {
    if p1 == 0 {
        m.call(0x001f_b658, &[o, 0x1000, 0x1000], &[]);
    } else {
        m.w16(o + 0x52, 0);
        m.call(0x001f_b060, &[o], &[]);
    }
    vcall(m, o, 0xc, 0x14, &[], &[]);
}
/// FUN_001fccf0: FUN_0011ccc0(0x3dd074)
pub fn ee_sub_1fccf0<M: QMem + ?Sized>(m: &mut M) -> u32 {
    // ANIM-ALL: tail call of FUN_0011ccc0 returns the pool block (was dropped)
    m.call(0x0011_ccc0, &[0x003d_d074], &[]).0
}
/// FUN_001fcd38: constructor
pub fn ee_sub_1fcd38<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    m.w32(o, 0);
    m.w32(o + 4, 0);
    m.w32(o + 0xc, 0x002c_a510);
    m.call(0x0011_cca8, &[o + 0x1c, 0x70, 0x80, 0x10], &[]);
    m.call(0x0011_cca8, &[o + 0x34, 0x3c0, 0x10, 0x10], &[]);
    m.w32(o + 0x200, 0);
    for i in 0..8u32 {
        m.w32(o + 0x208 + 4 * i, 0);
    }
    m.w32(o + 0x228, 0);
    m.call(0x0011_a1f8, &[o + 0x4c, 0x38], &[]);
    m.w32(o + 0x12c, 0x3f80_0000);
    m.w32(o + 0x68, 3);
    m.w32(o + 0x10, 0);
    m.w32(o + 0x104, 0);
    m.w32(o + 0x134, 0x3e80_0000);
    m.w32(o + 0x130, 0);
    m.w32(o + 0x120, 0x3f80_0000);
    m.w32(o + 0x128, 0x3e80_0000);
    m.w32(o + 0x124, 0);
    m.w32(o + 0x114, 0x4040_0000);
    m.w32(o + 0x118, 0);
    m.w32(o + 0x11c, 0x3e80_0000);
    let half = f32::from_bits(0x3f00_0000);
    m.call(0x0011_d8d8, &[o + 0x80], &[s(0.0), s(rof(0x002a_83bc)), s(half), s(rof(0x002a_83b8))]);
    m.call(0x0011_d8d8, &[o + 0x90], &[s(0.0), s(rof(0x002a_83c4)), s(half), s(rof(0x002a_83c0))]);
    m.w32(o + 0x58, 0);
    o
}
/// FUN_001fcfd0: l = FUN_001fcfa0(); l ? l->+0x50 : 0
pub fn ee_sub_1fcfd0<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let l = m.call(0x001f_cfa0, &[], &[]).0;
    if l != 0 { m.r32(l + 0x50) } else { 0 }
}
/// FUN_001fcff8: the node of the list +0xf0 (link +0x44) whose +0x40 == p2, else 0
pub fn ee_sub_1fcff8<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32) -> u32 {
    let mut n = m.r32(o + 0xf0);
 
    while n != 0 {
        if m.r32(n + 0x40) == p2 {
            return n;
        }
        n = m.r32(n + 0x44);
    }
    0
}
/// FUN_001fd080: 1 if p2 is in the list +0xf0 (link +0x44); else 0 if p2 == +0xf4, else 2
pub fn ee_sub_1fd080<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32) -> u32 {
    let mut n = m.r32(o + 0xf0);
    while n != 0 {
        if n == p2 {
            return 1;
        }
        n = m.r32(n + 0x44);
    }
    if m.r32(o + 0xf4) == p2 { 0 } else { 2 }
}

// ---------------------------------------------------------------- run_002037f0 / run_0020f0b0 / run_00212158 / run_00229468
/// FUN_002037f0 (same body as FUN_0011ae88)
pub fn ee_sub_2037f0<M: QMem + ?Sized>(m: &mut M, p1: u64, p2: u64) {
    pools_init(m, p1, p2)
}
/// FUN_00203928
pub fn ee_sub_203928<M: QMem + ?Sized>(m: &mut M) {
    pools_init(m, 1, 0xffff)
}
/// FUN_00203cb8: l = FUN_001f6030(); l != 0 -> FUN_00203d00(p1, l, p3)
pub fn ee_sub_203cb8<M: QMem + ?Sized>(m: &mut M, p1: u32, p3: u32) {
    let l = m.call(0x001f_6030, &[], &[]).0;
    if l != 0 {
        m.call(0x0020_3d00, &[p1, l, p3], &[]);
    }
}
/// FUN_0020f0b0: 0 < +0x3d78
pub fn ee_sub_20f0b0<M: QMem + ?Sized>(m: &mut M, o: u32) -> bool {
    0 < m.r32(o + 0x3d78) as i32
}
/// FUN_0020f128: ++(+0x3d80) == 1 -> FUN_001a3078(0x2e1e60)
pub fn ee_sub_20f128<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let n = m.r32(o + 0x3d80).wrapping_add(1);
    m.w32(o + 0x3d80, n);
    if n == 1 {
        m.call(0x001a_3078, &[0x002e_1e60], &[]);
    }
}
/// FUN_0020f160: --(+0x3d80) == 0 -> FUN_001a30d0(0x2e1e60)
pub fn ee_sub_20f160<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let n = m.r32(o + 0x3d80).wrapping_sub(1);
    m.w32(o + 0x3d80, n);
    if n == 0 {
        m.call(0x001a_30d0, &[0x002e_1e60], &[]);
    }
}
/// FUN_0020f198: ++(+0x3d84)
pub fn ee_sub_20f198<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let n = m.r32(o + 0x3d84).wrapping_add(1);
    m.w32(o + 0x3d84, n);
}
/// FUN_0020f1a8: --(+0x3d84)
pub fn ee_sub_20f1a8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let n = m.r32(o + 0x3d84).wrapping_sub(1);
    m.w32(o + 0x3d84, n);
}
/// FUN_0020f2d8: +0x3e24 = f; FUN_0020f1c8(o)
pub fn ee_sub_20f2d8<M: QMem + ?Sized>(m: &mut M, f: f32, o: u32) {
    m.wf(o + 0x3e24, f);
    m.call(0x0020_f1c8, &[o], &[]);
}
/// FUN_0020f2f8: +0x3e20 = f; FUN_0020f1c8(o)
pub fn ee_sub_20f2f8<M: QMem + ?Sized>(m: &mut M, f: f32, o: u32) {
    m.wf(o + 0x3e20, f);
    m.call(0x0020_f1c8, &[o], &[]);
}
/// FUN_002296a0: FUN_0011ccc0(o + 0x40e8)
pub fn ee_sub_2296a0<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    // ANIM-ALL: tail call of FUN_0011ccc0 returns the pool block (was dropped)
    m.call(0x0011_ccc0, &[o + 0x40e8], &[]).0
}
/// FUN_00212158: constructor
pub fn ee_sub_212158<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    m.w32(o + 0x7c, u32::MAX);
    m.w32(o + 0xc, 0x002d_af30);
    m.w32(o, 0);
    m.w32(o + 4, 0);
    for i in 0..10u32 {
        m.w32(o + 0xcc + 4 * i, u32::MAX);
    }
    m.call(0x0011_a050, &[o + 0xfc, 0x68], &[]);
    o
}

fn clear_tagged<M: QMem + ?Sized>(m: &mut M, field: u32, mask: u32) {
    let p = m.r32(field);
    if (p ^ 1) & 1 == 0 {
        m.w32(field, p & mask);
    } else {
        let v = m.r32(p);
        m.w32(p, v & mask);
    }
}
// ---------------------------------------------------------------- run_00212158 (rest) / run_001e77b0
/// FUN_00212780: reset (+0xf8, +0x50, +0x54, +0x80 (u16), +0x84 = 0), FUN_0011a060(o + 0xfc), (+0xc0)->+0x48 = +0x44 = 0
pub fn ee_sub_212780<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.w32(o + 0xf8, 0);
    m.w32(o + 0x50, 0);
    m.w32(o + 0x54, 0);
    m.w16(o + 0x80, 0);
    m.w32(o + 0x84, 0);
    m.call(0x0011_a060, &[o + 0xfc], &[]);
    let p = m.r32(o + 0xc0);
    m.w32(p + 0x48, 0);
    m.w32(p + 0x44, 0);
}
/// FUN_002127c8: over the list at DAT_003e106c (link +0x4c, record +0x40): a record with +0x58 != 0 whose age
/// (+0x58 == 1: DAT_002be538 - +0x60, else +0x64) is < 2 -> 0; else 1
pub fn ee_sub_2127c8<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let mut n = m.r32(0x003e_106c);
    while n != 0 {
        let r = m.r32(n + 0x40);
        let k = m.r32(r + 0x58);
        if k != 0 {
            let age = if k == 1 {
                m.r32(0x002b_e538).wrapping_sub(m.r32(r + 0x60)) as i32
            } else {
                m.r32(r + 100) as i32
            };
            if age < 2 {
                return 0;
            }
        }
        n = m.r32(n + 0x4c);
    }
    1
}
/// FUN_00212828: FUN_002013d8(buf, p2, 0x10, 10, tmp, o + 0xcc); destroy buf; vslot 0x18c(-1)
pub fn ee_sub_212828<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32) {
    let buf = stack_push(m, 0x10);
    let tmp = stack_push(m, 0x10);
    m.call(0x0020_13d8, &[buf, p2, 0x10, 10, tmp, o + 0xcc], &[]);
    m.call(0x0012_9ec8, &[buf], &[]);
    stack_pop(m, 0x20);
    vcall(m, o, 0xc, 0x18c, &[u32::MAX], &[]);
}
/// FUN_00212880: copy min(p2, 10) words from p3 to +0xcc; vslot 0x18c(-1)
pub fn ee_sub_212880<M: QMem + ?Sized>(m: &mut M, o: u32, p2: i32, p3: u32) {
    let mut i = 0u32;
    if 0 < p2 {
        loop {
            let v = m.r32(p3 + 4 * i);
            m.w32(o + 0xcc + 4 * i, v);
            i += 1;
            if p2 <= i as i32 || 9 < i {
                break;
            }
        }
    }
    vcall(m, o, 0xc, 0x18c, &[u32::MAX], &[]);
}
/// FUN_001e77b0: FUN_001e6f60(p1, p2, 0, &out, 0, 0, 0); out[0]
pub fn ee_sub_1e77b0<M: QMem + ?Sized>(m: &mut M, p1: u32, p2: u32) -> u32 {
    let b = stack_push(m, 0x10);
    m.call(0x001e_6f60, &[p1, p2, 0, b, 0, 0, 0], &[]);
    let r = m.r32(b);
    stack_pop(m, 0x10);
    r
}
/// FUN_001e7930: FUN_001e93f0(FUN_001e7840(o, 1), 0, p2)
pub fn ee_sub_1e7930<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32) {
    let u = m.call(0x001e_7840, &[o, 1], &[]).0;
    m.call(0x001e_93f0, &[u, 0, p2], &[]);
}
/// FUN_001e7968: FUN_001e93f0(FUN_001e7840(o, 1), 1, p2)
pub fn ee_sub_1e7968<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32) {
    let u = m.call(0x001e_7840, &[o, 1], &[]).0;
    m.call(0x001e_93f0, &[u, 1, p2], &[]);
}
/// FUN_001e79a0: +0xe8 < 1 ? 0 : last word of FUN_001e7840(o, 0)->+0x5c table
pub fn ee_sub_1e79a0<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let n = m.r32(o + 0xe8);
    if (n as i32) < 1 {
        return 0;
    }
    let u = m.call(0x001e_7840, &[o, 0], &[]).0;
    let t = m.r32(u + 0x5c);
    m.r32(n.wrapping_mul(4).wrapping_add(t).wrapping_sub(4))
}
/// FUN_001e79f0: FUN_001e95b0(p1, FUN_001e7840(p2, 1))
pub fn ee_sub_1e79f0<M: QMem + ?Sized>(m: &mut M, o: u32, f: f32) {
    // VFIX3 1:1 per asm: FUN_001e7840(a0, 1); FUN_001e95b0(v0, f12 = f)
    let u = m.call(0x001e_7840, &[o, 1], &[]).0;
    m.call(0x001e_95b0, &[u], &[[f, 0.0, 0.0, 0.0]]);
}
/// FUN_001e7a28: p2 == -1: FUN_001a6ca8(o, 0), flags |= 0x20, &= ~0xc; else FUN_001e7840(o, 1), FUN_001a6ca8(o, 0),
/// &= ~0x20, |= 0xc; then (+0x17c)->+0x4c = p2
pub fn ee_sub_1e7a28<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32) {
    if p2 == u32::MAX {
        m.call(0x001a_6ca8, &[o, 0], &[]);
        set_tagged(m, o + 0x40, 0x20);
        clear_tagged(m, o + 0x40, 0xffff_fff3);
    } else {
        m.call(0x001e_7840, &[o, 1], &[]);
        m.call(0x001a_6ca8, &[o, 0], &[]);
        clear_tagged(m, o + 0x40, 0xffff_ffdf);
        set_tagged(m, o + 0x40, 0xc);
    }
    let p = m.r32(o + 0x17c);
    if p != 0 {
        m.w32(p + 0x4c, p2);
    }
}

// ---------------------------------------------------------------- run_001d8600 (string-format helpers) / run_001e77b0 (rest)
fn alloc_req<M: QMem + ?Sized>(m: &mut M, f: u32, size: u32) -> u32 {
    let req = stack_push(m, 8);
    m.w32(req, size);
    m.w32(req + 4, 0x10);
    let p = m.call(f, &[req], &[]).0;
    stack_pop(m, 8);
    p
}
/// FUN_001d8600: +0xd4 = n; +0xd8 = FUN_001ae1d0({n << 4, 0x10})
pub fn ee_sub_1d8600<M: QMem + ?Sized>(m: &mut M, o: u32, n: u32) {
    m.w32(o + 0xd4, n);
    let p = alloc_req(m, 0x001a_e1d0, n << 4);
    m.w32(o + 0xd8, p);
}
/// FUN_001d8690: pop a 16-byte entry {hash, key, value, next} of the pool (+0xd8, count +0xd4) into bucket
/// hash % 0x35 of the table at o
pub fn ee_sub_1d8690<M: QMem + ?Sized>(m: &mut M, o: u32, key: u32, val: u32) {
    let h = m.call(0x001d_8640, &[key], &[]).0;
    let n = m.r32(o + 0xd4);
    m.w32(o + 0xd4, n.wrapping_sub(1));
    let e = m.r32(o + 0xd8).wrapping_add(n.wrapping_mul(0x10)).wrapping_sub(0x10);
    m.w32(e + 4, key);
    m.w32(e + 8, val);
    m.w32(e, h);
    let b = o.wrapping_add((((h as i32) % 0x35) as u32).wrapping_mul(4));
    let hd = m.r32(b);
    m.w32(e + 0xc, hd);
    m.w32(b, e);
}
/// write the varargs a2..a8 as consecutive 8-byte slots, returns the block (caller pops)
fn varargs<M: QMem + ?Sized>(m: &mut M, a: &[u32]) -> u32 {
    let blk = stack_push(m, 8 * a.len() as u32);
    for (i, &v) in a.iter().enumerate() {
        m.w32(blk + 8 * i as u32, v);
        m.w32(blk + 8 * i as u32 + 4, 0);
    }
    blk
}
/// FUN_001d88d0(p1, a2..a8): a 2-word string object {len, buf} built by FUN_001d87c0 and shrunk to len * 2
pub fn ee_sub_1d88d0<M: QMem + ?Sized>(m: &mut M, p1: u32, a: [u32; 7]) -> u32 {
    let blk = varargs(m, &a);
    let f = crate::engine::classes::q::q8util::frame_begin(m);
    let r = alloc_req(m, 0x001a_e1d0, 8);
    m.call(0x001d_87c0, &[p1, blk, r], &[]);
    let len = m.r32(r);
    let req = stack_push(m, 8);
    m.w32(req, len << 1);
    m.w32(req + 4, 0x10);
    let old = m.r32(r + 4);
    let n = m.call(0x001a_e238, &[req, old], &[]).0;
    stack_pop(m, 8);
    m.w32(r + 4, n);
    crate::engine::classes::q::q8util::frame_end(m, f, 2);
    stack_pop(m, 0x38);
    r
}
/// FUN_001d89a8(p1, a2..a8): same string object from the allocator FUN_001ae318, size 8
pub fn ee_sub_1d89a8<M: QMem + ?Sized>(m: &mut M, p1: u32, a: [u32; 7]) -> u32 {
    let blk = varargs(m, &a);
    let r = alloc_req(m, 0x001a_e318, 8);
    m.call(0x001d_87c0, &[p1, blk, r], &[]);
    stack_pop(m, 0x38);
    r
}
/// FUN_001d8a38(out, p2, a3..a8)
pub fn ee_sub_1d8a38<M: QMem + ?Sized>(m: &mut M, out: u32, p2: u32, a: [u32; 6]) -> u32 {
    let blk = varargs(m, &a);
    let s0 = stack_push(m, 0x10);
    m.w32(s0, u32::MAX);
    let f = crate::engine::classes::q::q8util::frame_begin(m);
    let s1 = stack_push(m, 0x10);
    let s2 = stack_push(m, 0x10);
    m.call(0x001d_87c0, &[p2, blk, s1], &[]);
    m.call(0x001d_7ed8, &[s2, 0x003e_0ea8, s1, 0x003d_4a60], &[]);
    m.call(0x0012_9658, &[s0, s2], &[]);
    m.call(0x0012_9ec8, &[s2], &[]);
    m.call(0x0012_91a8, &[out, s0], &[]);
    stack_pop(m, 0x20);
    crate::engine::classes::q::q8util::frame_end(m, f, 2);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0x10);
    stack_pop(m, 0x30);
    out
}
/// FUN_001d8b20(out, p2, p3, a4..a8)
pub fn ee_sub_1d8b20<M: QMem + ?Sized>(m: &mut M, out: u32, p2: u32, p3: u32, a: [u32; 5]) -> u32 {
    let blk = varargs(m, &a);
    let s0 = stack_push(m, 0x10);
    m.w32(s0, u32::MAX);
    let f = crate::engine::classes::q::q8util::frame_begin(m);
    let s1 = stack_push(m, 0x10);
    let s2 = stack_push(m, 0x10);
    m.call(0x001d_87c0, &[p3, blk, s1], &[]);
    m.call(0x001d_80c8, &[s2, 0x003e_0ea8, s1, p2, 0x003d_4a60], &[]);
    m.call(0x0012_9658, &[s0, s2], &[]);
    m.call(0x0012_9ec8, &[s2], &[]);
    m.call(0x0012_91a8, &[out, s0], &[]);
    stack_pop(m, 0x20);
    crate::engine::classes::q::q8util::frame_end(m, f, 2);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0x10);
    stack_pop(m, 0x28);
    out
}
/// FUN_001d8c10(p1, a2..a8)
pub fn ee_sub_1d8c10<M: QMem + ?Sized>(m: &mut M, p1: u32, a: [u32; 7]) -> u32 {
    let blk = varargs(m, &a);
    let s0 = stack_push(m, 0x10);
    m.w32(s0, u32::MAX);
    let f = crate::engine::classes::q::q8util::frame_begin(m);
    let s1 = stack_push(m, 0x10);
    m.call(0x001d_87c0, &[p1, blk, s1], &[]);
    let r = m.call(0x001d_7e80, &[0x003e_0ea8, s1, 0x003d_4a60], &[]).0;
    stack_pop(m, 0x10);
    crate::engine::classes::q::q8util::frame_end(m, f, 2);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0x10);
    stack_pop(m, 0x38);
    r
}
/// FUN_001d8cc8: concatenation of two {len, u16 buf} strings
pub fn ee_sub_1d8cc8<M: QMem + ?Sized>(m: &mut M, a: u32, b: u32) -> u32 {
    let r = alloc_req(m, 0x001a_e318, 8);
    let (la, lb) = (m.r32(a), m.r32(b));
    m.w32(r, la.wrapping_add(lb));
    let buf = alloc_req(m, 0x001a_e318, la.wrapping_add(lb).wrapping_mul(2));
    m.w32(r + 4, buf);
    let sa = m.r32(a + 4);
    m.call(0x001a_e5a0, &[buf, sa, la << 1], &[]);
    let sb = m.r32(b + 4);
    m.call(0x001a_e5a0, &[buf.wrapping_add(la.wrapping_mul(2)), sb, lb << 1], &[]);
    r
}
/// FUN_001e8008(f12 = p1, f13 = p2, a0 = o)
pub fn ee_sub_1e8008<M: QMem + ?Sized>(m: &mut M, mut p1: f32, p2: f32, o: u32) {
    let fv2;
    if m.r32(o + 0xe4) == 0 {
        let len = callf(m, 0x001e_6648, &[o], &[]);
        let mut f3 = p2;
        if p2 <= p1 {
            f3 = len - p2;
            if p1 <= len - p2 {
                f3 = p1;
            }
        }
        let f1 = f3 + p2;
        fv2 = if f1 < len { f1 } else { len };
        p1 = if 0.0 < f3 - p2 { f3 - p2 } else { 0.0 };
    } else {
        fv2 = p1 + p2;
        p1 -= p2;
    }
    let mut f3 = callf(m, 0x0011_d448, &[], &[s(-1.0), s(1.0)]);
    let mode = m.r32(0x002c_3650) as i32;
    if mode == 1 {
        f3 *= f3.abs();
    } else if mode > 1 {
        if mode == 2 {
            let k = if 0.0 < f3 { 3.0 } else { -3.0 };
            f3 = f3 * f3 * (k - (f3 + f3));
        } else if mode == 3 {
            f3 = if 0.0 < f3 { f3.sqrt() } else { -(-f3).sqrt() };
        }
    }
    f3 = f3 * 0.5 + 0.5;
    if m.r32(o + 0xe4) != 0 {
        m.call(0x001e_66c8, &[o], &[s((1.0 - f3) * p1 + f3 * fv2)]);
    }
}
/// FUN_001e81c0: constructor of the path follower over the object p2 (+0xe8 points, table +0xf4, matrix by vslot 0xfc)
pub fn ee_sub_1e81c0<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32) -> u32 {
    m.w32(o + 0x48, p2);
    m.w32(o + 0x4c, u32::MAX);
    let n = m.r32(p2 + 0xe8);
    let a = alloc_req(m, 0x001a_e2a0, n << 4);
    m.w32(o + 0x54, a);
    let b = alloc_req(m, 0x001a_e2a0, n << 4);
    m.w32(o + 0x58, b);
    let c = alloc_req(m, 0x001a_e2a0, n << 2);
    m.w32(o + 8, 0xc4da_c000);
    m.w32(o + 0x5c, c);
    m.w32(o + 0x60, 0x4040_0000);
    m.w32(o + 0x6c, 0x43fa_0000);
    m.w32(o + 0x70, 0x40a0_0000);
    m.w32(o, 0);
    m.w32(o + 4, 0);
    let ob = m.r32(o + 0x48);
    let cnt = m.r32(ob + 0xe8);
    m.w32(o + 0x88, 0x4396_0000);
    m.w32(o + 0x84, cnt);
    let mat = vcall(m, ob, 0xc, 0xfc, &[], &[]).0;
    let mut i = 0u32;
    if 0 < rr2(m, o + 0x48, 0xe8) as i32 {
        loop {
            let tbl = rr2(m, o + 0x48, 0xf4);
            let p = m.rv(tbl + i * 0x10);
            let (r0, r1, r2, r3) = (m.rv(mat), m.rv(mat + 0x10), m.rv(mat + 0x20), m.rv(mat + 0x30));
            let v: [f32; 4] = std::array::from_fn(|k| r0[k] * p[0] + r1[k] * p[1] + r2[k] * p[2] + r3[k]);
            let dst = m.r32(o + 0x54);
            m.wv(dst + i * 0x10, v);
            i += 1;
            if rr2(m, o + 0x48, 0xe8) as i32 <= i as i32 {
                break;
            }
        }
    }
    m.w32(o + 0x40, 0x002c_44f0);
    m.w32(o + 0x44, 0x002c_44f0);
    let cnt = rr2(m, o + 0x48, 0xe8) as i32;
    if 0 < cnt {
        let p = m.r32(o + 0x54);
        for k in 0..4u32 {
            let w = m.r32(p + 4 * k);
            m.w32(o + 0x20 + 4 * k, w);
        }
        let last = p.wrapping_add((cnt as u32).wrapping_mul(0x10)).wrapping_sub(0x10);
        for k in 0..4u32 {
            let w = m.r32(last + 4 * k);
            m.w32(o + 0x30 + 4 * k, w);
        }
    }
    m.call(0x001e_9828, &[o], &[]);
    o
}

fn rr2<M: QMem + ?Sized>(m: &mut M, a: u32, off: u32) -> u32 {
    let p = m.r32(a);
    m.r32(p + off)
}
