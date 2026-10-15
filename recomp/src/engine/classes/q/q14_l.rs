//! WP-Q14 part L: FUN_001bc9f0, FUN_00205670 / 002057e8 (segment vs. the two triangles of the object's quad) and
//! the static-init FUN_00205d18. 1:1 from ghidra_functions(_v2).body on `QMem`; partial notes in port_tracking.
#![allow(unused_variables, unused_mut, dead_code)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem, V4};

fn dot3(a: V4, b: V4) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// FUN_001bc9f0(obj, v): 1 unless some plane quad (obj+0x1ec)[i], i < obj[0xe8]-1, has dot(q, v) > 0.
/// VU0 scratch calls (FUN_002564b8) zero stack buffers with no semantic effect; dot3 matches the
/// vmul/vaddabc/vmaddabc/vmsubbc pipeline's x-component output.
pub fn ee_sub_1bc9f0<M: QMem + ?Sized>(m: &mut M, obj: u32, v: V4) -> u32 {
    let sp = stack_push(m, 0x80);
    m.call(0x001b_d400, &[], &[]);
    // FUN_002564b8 zeroes stack scratch (auStack_80, auStack_70)
    m.call(0x0025_64b8, &[sp], &[]);
    m.call(0x0025_64b8, &[sp + 0x10], &[]);
    let n = (m.r32(obj + 0xe8) as i32) - 1;
    let mut i = 0;
    while i < n {
        let b = m.r32(obj + 0x1ec);
        let q = m.rv(b + i as u32 * 0x10);
        // VU0 pipeline: vmul(q, v) -> accumulate -> check x component > 0
        if 0.0 < dot3(q, v) {
            stack_pop(m, 0x80);
            return 0;
        }
        i += 1;
    }
    stack_pop(m, 0x80);
    1
}

/// FUN_002057e8(obj, out): the 2 triangles (3 quads each, 0x30 bytes) of the oriented square around quad *(obj+0x218)
/// with half extent obj[0x220], spanned by the two rows of FUN_001a6318(obj); corner table at 0x2c3cc8.
pub fn ee_sub_2057e8<M: QMem + ?Sized>(m: &mut M, obj: u32, out: u32) {
    let ap = m.r32(obj + 0x218);
    let a = m.rv(ap);
    let rp = m.call(0x001a_6318, &[obj], &[]).0;
    let r0 = m.rv(rp);
    let r1 = m.rv(rp + 0x10);
    let pos = m.call(0x001a_62e8, &[obj], &[]).1;
    let s = m.rf(obj + 0x220);
    let add = |r: V4, k: f32| [a[0] + r[0] * k, a[1] + r[1] * k, a[2] + r[2] * k, a[3] + r[3] * k];
    let c = [pos, add(r0, s), add(r1, s), add(r0, -s), add(r1, -s)];
    for t in 0..2u32 {
        for k in 0..3u32 {
            let ix = m.r32(0x002c_3cc8 + (t * 3 + k) * 4) as usize;
            m.wv(out + t * 0x30 + k * 0x10, c[ix]);
        }
    }
}

/// FUN_00205670(obj, p0, p1, out): sweep p0 -> p1 against the two triangles (FUN_002057e8) with
/// FUN_0011ebb0(dir, p0, v0, v1, v2, &t, 0) (all five as 128-bit vector args); hit within |p1 - p0| ->
/// out = p0 + dir * t, returns 1. |p1 - p0| < 1e-4: dir = DAT_002a2dc0 and the EE code reuses the low word of
/// that vector as the length limit (kept 1:1).
pub fn ee_sub_205670<M: QMem + ?Sized>(m: &mut M, obj: u32, p0: V4, p1: V4, out: u32) -> u32 {
    if m.r32(obj + 0x208) == 0 {
        return 0;
    }
    let sp = stack_push(m, 0x70);
    let (tri, tf) = (sp, sp + 0x60);
    ee_sub_2057e8(m, obj, tri);
    let d = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2], p1[3] - p0[3]];
    let len = dot3(d, d).sqrt();
    let (dir, lim) = if len < 0.0001 {
        let u = m.rv(0x002a_2dc0);
        (u, u[0])
    } else {
        ([d[0] / len, d[1] / len, d[2] / len, d[3] / len], len)
    };
    for t in 0..2u32 {
        let b = tri + t * 0x30;
        let (v0, v1, v2) = (m.rv(b), m.rv(b + 0x10), m.rv(b + 0x20));
        let r = m.call(0x0011_ebb0, &[tf, 0], &[dir, p0, v0, v1, v2]).0;
        let f = m.rf(tf);
        if f <= lim && r & 1 != 0 {
            m.wv(out, [p0[0] + dir[0] * f, p0[1] + dir[1] * f, p0[2] + dir[2] * f, p0[3] + dir[3] * f]);
            stack_pop(m, 0x70);
            return 1;
        }
    }
    stack_pop(m, 0x70);
    0
}

/// FUN_00205d18(p1, p2): six ref-counted pool inits
pub fn ee_sub_205d18<M: QMem + ?Sized>(m: &mut M, p1: u32, p2: u32) {
    if p2 == 0xffff && p1 != 0 {
        for (ctr, a, b, c) in [
            (0x002d_d2e0u32, 0x0045_20a8u32, 0x14u32, 0x2000u32),
            (0x002d_d2e4, 0x0045_20d0, 0x18, 0x800),
            (0x002d_d2e8, 0x0045_20f8, 0x28, 0x200),
            (0x002d_d2ec, 0x0045_2120, 0x1c, 0x800),
            (0x002d_d2f0, 0x0045_2148, 0xc, 200),
            (0x002d_d2f4, 0x0045_2170, 0x14, 0x100),
        ] {
            let n = m.r32(ctr).wrapping_add(1);
            m.w32(ctr, n);
            if n == 1 {
                m.call(0x0011_cca8, &[a, b, c, 0x10], &[]);
            }
        }
    }
}
