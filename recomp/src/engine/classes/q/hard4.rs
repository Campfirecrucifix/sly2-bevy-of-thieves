//! WP-HARD4: the 32 smallest `skipped` game-logic functions, ported 1:1 from the raw disassembly
//! (splice BIFs, camera-track heading, PS2 recursive-lock kernel wrappers, a pure VU0 vector op, and a
//! handful of thin dispatchers). QMem convention: integer / pointer args in `a`, float / quadword (VU0
//! u128) args in `v` (source order); quadword result is `.1`. External callees and the PS2 kernel
//! syscalls are routed by original address through `QMem::call`, exactly like the other queue ports.
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};
use crate::engine::classes::q::q6::{callf, rov, s};
use crate::engine::math::vu0;

// ---------------------------------------------------------------------------------------------------
// WP-F splice BIFs. Each builds a splice result value in a stack temp and copies it into the BIF return
// slot `p1` (FUN_001291a8), releasing temps (FUN_00129ec8). FUN_0012a0b0(v,tag) sets a value's type tag.
// ---------------------------------------------------------------------------------------------------

/// FUN_00123eb0 (RefOpObjectOp): op `p5` selects a geometry op on the object at args `*(p3+4)`/`*(p3+0xc)`
/// with the trailing quad at `p3+0x10`: 0x6c FUN_00200ca8, 0x6d FUN_00200d10, 0x6e FUN_00200c28,
/// 0x6f FUN_00200ba0 (no args); any other op leaves the result at its 0xffffffff init. The built value is
/// copied into `p1`.
pub fn ee_sub_123eb0<M: QMem + ?Sized>(m: &mut M, p1: u32, _p2: u32, p3: u32, _p4: u32, p5: u32) -> u32 {
    let u1 = m.r32(p3 + 4);
    let u2 = m.r32(p3 + 0xc);
    let sp = stack_push(m, 0x20);
    let (res, tmp) = (sp + 0x10, sp);
    m.w32(res, 0xffff_ffff);
    let mut done = false;
    if p5 == 0x6d {
        m.call(0x0020_0d10, &[tmp, u1, u2, p3 + 0x10], &[]);
    } else if p5 < 0x6e {
        if p5 != 0x6c {
            done = true;
        } else {
            m.call(0x0020_0ca8, &[tmp, u1, u2, p3 + 0x10], &[]);
        }
    } else if p5 != 0x6e {
        if p5 == 0x6f {
            m.call(0x0020_0ba0, &[tmp], &[]);
            m.call(0x0012_9658, &[res, tmp], &[]);
            m.call(0x0012_9ec8, &[tmp], &[]);
        }
        done = true;
    } else {
        m.call(0x0020_0c28, &[tmp, u1, u2, p3 + 0x10], &[]);
    }
    if !done {
        m.call(0x0012_9658, &[res, tmp], &[]);
        m.call(0x0012_9ec8, &[tmp], &[]);
    }
    m.call(0x0012_91a8, &[p1, res], &[]);
    m.call(0x0012_9ec8, &[res], &[]);
    stack_pop(m, 0x20);
    p1
}

/// FUN_00124018 (RefOpSetO, BIF 110): FUN_00123eb0 with op 0x6e.
pub fn ee_sub_124018<M: QMem + ?Sized>(m: &mut M, p1: u32, p2: u32, p3: u32, p4: u32) -> u32 {
    ee_sub_123eb0(m, p1, p2, p3, p4, 0x6e)
}

/// FUN_00124048 (RefOpGetO, BIF 111): FUN_00123eb0 with op 0x6f.
pub fn ee_sub_124048<M: QMem + ?Sized>(m: &mut M, p1: u32, p2: u32, p3: u32, p4: u32) -> u32 {
    ee_sub_123eb0(m, p1, p2, p3, p4, 0x6f)
}

/// FUN_001248b0 (RefOpFindObjectsInBoundingSphere, BIF 122): radius = `*(p3+0xc)` (int->float when the arg
/// tag `*(p3+8)` is 0x15); in a frame (FUN_001ae478/FUN_001ae4a0), FUN_0020fc60(radius, 0x3dd490,
/// **(p3+4), 0, 0, &a, &b); (c,d) = FUN_0020fdc8(a, b); FUN_00124078(val, c, d) -> copied into `p1`.
pub fn ee_sub_1248b0<M: QMem + ?Sized>(m: &mut M, p1: u32, _p2: u32, p3: u32) -> u32 {
    let sp = stack_push(m, 0xa0);
    let (res, frame, v70) = (sp, sp + 0x10, sp + 0x20);
    let (u60, u5c, u58, u54) = (sp + 0x40, sp + 0x44, sp + 0x48, sp + 0x4c);
    m.w32(res, 0xffff_ffff);
    let radius = if m.r32(p3 + 8) == 0x15 {
        m.r32(p3 + 0xc) as i32 as f32
    } else {
        m.rf(p3 + 0xc)
    };
    m.call(0x001a_e478, &[frame], &[]);
    let op = m.r32(p3 + 4);
    let obj = m.r32(op);
    m.call(0x0020_fc60, &[0x003d_d490, obj, 0, 0, u60, u5c], &[s(radius)]);
    let (a, b) = (m.r32(u60), m.r32(u5c));
    m.call(0x0020_fdc8, &[a, b, u58, u54], &[]);
    let (c, d) = (m.r32(u58), m.r32(u54));
    m.call(0x0012_4078, &[v70, c, d], &[]);
    m.call(0x0012_9658, &[res, v70], &[]);
    m.call(0x0012_9ec8, &[v70], &[]);
    m.call(0x0012_91a8, &[p1, res], &[]);
    m.call(0x001a_e4a0, &[frame, 2], &[]);
    m.call(0x0012_9ec8, &[res], &[]);
    stack_pop(m, 0xa0);
    p1
}

/// FUN_001247d8 (RefOpFindObjectsInBoundingBox, BIF 121): in a frame, FUN_0020faa0(0x3dd490, **(p3+4),
/// **(p3+0xc), 0, 0, &a, &b); (c,d) = FUN_0020fdc8(a, b); FUN_00124078(val, c, d) -> copied into `p1`.
pub fn ee_sub_1247d8<M: QMem + ?Sized>(m: &mut M, p1: u32, _p2: u32, p3: u32) -> u32 {
    let sp = stack_push(m, 0x90);
    let (res, frame, v60) = (sp, sp + 0x10, sp + 0x20);
    let (u50, u4c, u48, u44) = (sp + 0x40, sp + 0x44, sp + 0x48, sp + 0x4c);
    m.w32(res, 0xffff_ffff);
    m.call(0x001a_e478, &[frame], &[]);
    let (pa, pb) = (m.r32(p3 + 4), m.r32(p3 + 0xc));
    let (a, b) = (m.r32(pa), m.r32(pb));
    m.call(0x0020_faa0, &[0x003d_d490, a, b, 0, 0, u50, u4c], &[]);
    let (x, y) = (m.r32(u50), m.r32(u4c));
    m.call(0x0020_fdc8, &[x, y, u48, u44], &[]);
    let (c, d) = (m.r32(u48), m.r32(u44));
    m.call(0x0012_4078, &[v60, c, d], &[]);
    m.call(0x0012_9658, &[res, v60], &[]);
    m.call(0x0012_9ec8, &[v60], &[]);
    m.call(0x0012_91a8, &[p1, res], &[]);
    m.call(0x001a_e4a0, &[frame, 2], &[]);
    m.call(0x0012_9ec8, &[res], &[]);
    stack_pop(m, 0x90);
    p1
}

/// FUN_001241e8 (Bif113): mode = `*(p3+0xc) != 0 ? 1 : 5`; filter = `*(p3+0x10) ? *(*(p3+0x14)+0xc) :
/// *(p3+0xc)`, class = `filter ? *(filter+0x3c) : DAT_003e139c`; n = FUN_0015e678(0x3dd490, mode,
/// *(p3+4), *(p3+0xc), class, 0x40, buf); FUN_00124078(val, n, buf) -> `p1`.
pub fn ee_sub_1241e8<M: QMem + ?Sized>(m: &mut M, p1: u32, _p2: u32, p3: u32) -> u32 {
    let i1 = m.r32(p3 + 0xc);
    let i4 = if m.r32(p3 + 0x10) != 0 {
        let f = m.r32(p3 + 0x14);
        m.r32(f + 0xc)
    } else {
        i1
    };
    let mode = if i1 != 0 { 1 } else { 5 };
    let class = if i4 != 0 { m.r32(i4 + 0x3c) } else { m.r32(0x003e_139c) };
    let sp = stack_push(m, 0x160);
    let (res, buf, val) = (sp, sp + 0x10, sp + 0x110);
    m.w32(res, 0xffff_ffff);
    let a4 = m.r32(p3 + 4);
    let n = m.call(0x0015_e678, &[0x003d_d490, mode, a4, i1, class, 0x40, buf], &[]).0;
    m.call(0x0012_4078, &[val, n, buf], &[]);
    m.call(0x0012_9658, &[res, val], &[]);
    m.call(0x0012_9ec8, &[val], &[]);
    m.call(0x0012_91a8, &[p1, res], &[]);
    m.call(0x0012_9ec8, &[res], &[]);
    stack_pop(m, 0x160);
    p1
}

/// FUN_00124578 (Bif116): filter = `*(p3+0x10) ? *(*(p3+0x14)+0xc) : *(p3+0xc)`, class = `filter ?
/// *(filter+0x3c) : DAT_003e139c`; n = FUN_0015e678(0x3dd490, 4, *(p3+4), *(p3+0xc), class, 0x40, buf);
/// FUN_00124078(val, n, buf) -> `p1`.
pub fn ee_sub_124578<M: QMem + ?Sized>(m: &mut M, p1: u32, _p2: u32, p3: u32) -> u32 {
    let i3 = if m.r32(p3 + 0x10) != 0 {
        let f = m.r32(p3 + 0x14);
        m.r32(f + 0xc)
    } else {
        m.r32(p3 + 0xc)
    };
    let class = if i3 != 0 { m.r32(i3 + 0x3c) } else { m.r32(0x003e_139c) };
    let sp = stack_push(m, 0x160);
    let (res, buf, val) = (sp, sp + 0x10, sp + 0x110);
    m.w32(res, 0xffff_ffff);
    let (a4, ac) = (m.r32(p3 + 4), m.r32(p3 + 0xc));
    let n = m.call(0x0015_e678, &[0x003d_d490, 4, a4, ac, class, 0x40, buf], &[]).0;
    m.call(0x0012_4078, &[val, n, buf], &[]);
    m.call(0x0012_9658, &[res, val], &[]);
    m.call(0x0012_9ec8, &[val], &[]);
    m.call(0x0012_91a8, &[p1, res], &[]);
    m.call(0x0012_9ec8, &[res], &[]);
    stack_pop(m, 0x160);
    p1
}

/// FUN_00123e18 (RefOpDeferObjectUpdate, BIF 107): FUN_0012a0b0(val, 0x24) (Void); obj = `*(p3+4)`;
/// FUN_00129da8(tmp, p3+8) reads a float into tmp+4; `*(obj+0x30) = DAT_002dded0 + that`; result -> `p1`.
pub fn ee_sub_123e18<M: QMem + ?Sized>(m: &mut M, p1: u32, _p2: u32, p3: u32) -> u32 {
    let sp = stack_push(m, 0x80);
    let (res, tmp) = (sp, sp + 0x10);
    m.call(0x0012_a0b0, &[res, 0x24], &[]);
    let obj = m.r32(p3 + 4);
    m.call(0x0012_9da8, &[tmp, p3 + 8], &[]);
    let f = m.rf(tmp + 4);
    m.call(0x0012_9ec8, &[tmp], &[]);
    let base = m.rf(0x002d_ded0);
    m.wf(obj + 0x30, base + f);
    m.call(0x0012_91a8, &[p1, res], &[]);
    m.call(0x0012_9ec8, &[res], &[]);
    stack_pop(m, 0x80);
    p1
}

/// FUN_001250d8 (RefOpConvertObjectMatrix, BIF 127): FUN_0012a0b0(val, 0x1b); FUN_001a71b0(mat, *(p3+4),
/// *(p3+0xc), *(p3+0x14)) builds a 0x40-byte matrix copied into the value's buffer `*(val+4)`; -> `p1`.
pub fn ee_sub_1250d8<M: QMem + ?Sized>(m: &mut M, p1: u32, _p2: u32, p3: u32) -> u32 {
    let sp = stack_push(m, 0x60);
    let (res, mat) = (sp, sp + 0x10);
    m.call(0x0012_a0b0, &[res, 0x1b], &[]);
    let (a4, ac, a14) = (m.r32(p3 + 4), m.r32(p3 + 0xc), m.r32(p3 + 0x14));
    m.call(0x001a_71b0, &[mat, a4, ac, a14], &[]);
    let dst = m.r32(res + 4);
    for i in 0..16u32 {
        let w = m.r32(mat + 4 * i);
        m.w32(dst + 4 * i, w);
    }
    m.call(0x0012_91a8, &[p1, res], &[]);
    m.call(0x0012_9ec8, &[res], &[]);
    stack_pop(m, 0x60);
    p1
}

/// FUN_00124f00 (RefOpHitTestObjects, BIF 123): FUN_001249a8(tmp, 0x7b, p2, p3, p4) -> copied into `p1`.
pub fn ee_sub_124f00<M: QMem + ?Sized>(m: &mut M, p1: u32, p2: u32, p3: u32, p4: u32) -> u32 {
    hit_test(m, p1, 0x7b, p2, p3, p4)
}

/// FUN_00124f80 (RefOpHitTestObjectsFirst, BIF 124): FUN_001249a8(tmp, 0x7c, p2, p3, p4) -> `p1`.
pub fn ee_sub_124f80<M: QMem + ?Sized>(m: &mut M, p1: u32, p2: u32, p3: u32, p4: u32) -> u32 {
    hit_test(m, p1, 0x7c, p2, p3, p4)
}

fn hit_test<M: QMem + ?Sized>(m: &mut M, p1: u32, op: u32, p2: u32, p3: u32, p4: u32) -> u32 {
    let sp = stack_push(m, 0x20);
    let (res, tmp) = (sp + 0x10, sp);
    m.w32(res, 0xffff_ffff);
    m.call(0x0012_49a8, &[tmp, op, p2, p3, p4], &[]);
    m.call(0x0012_9658, &[res, tmp], &[]);
    m.call(0x0012_9ec8, &[tmp], &[]);
    m.call(0x0012_91a8, &[p1, res], &[]);
    m.call(0x0012_9ec8, &[res], &[]);
    stack_pop(m, 0x20);
    p1
}

/// FUN_00125060 (RefOpConvertObjectVector, BIF 126): FUN_0012a0b0(val, 0x1a); the quadword from
/// FUN_001a6fc0(*(p3+4), *(p3+0xc), **(p3+0x14)) is stored into the value's buffer `*(val+4)`; -> `p1`.
pub fn ee_sub_125060<M: QMem + ?Sized>(m: &mut M, p1: u32, _p2: u32, p3: u32) -> u32 {
    let sp = stack_push(m, 0x10);
    m.call(0x0012_a0b0, &[sp, 0x1a], &[]);
    let (a4, ac, p14) = (m.r32(p3 + 4), m.r32(p3 + 0xc), m.r32(p3 + 0x14));
    let a14 = m.r32(p14);
    let q = m.call(0x001a_6fc0, &[a4, ac, a14], &[]).1;
    let dst = m.r32(sp + 4);
    m.wv(dst, q);
    m.call(0x0012_91a8, &[p1, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x10);
    p1
}

/// FUN_00125000 (RefOpConvertObjectPosition, BIF 125): FUN_00129248(val, FUN_001a6f00(*(p3+4), *(p3+0xc),
/// **(p3+0x14))) (position value from the quadword) -> copied into `p1`.
pub fn ee_sub_125000<M: QMem + ?Sized>(m: &mut M, p1: u32, _p2: u32, p3: u32) -> u32 {
    let (a4, ac, p14) = (m.r32(p3 + 4), m.r32(p3 + 0xc), m.r32(p3 + 0x14));
    let a14 = m.r32(p14);
    let q = m.call(0x001a_6f00, &[a4, ac, a14], &[]).1;
    let sp = stack_push(m, 0x10);
    m.call(0x0012_9248, &[sp], &[q]);
    m.call(0x0012_91a8, &[p1, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x10);
    p1
}

/// FUN_00125738 (RefOpEmitSmokeCloud, BIF 139): FUN_0012a0b0(val, 0x24) (Void); FUN_001daf38(*(p3+4),
/// *(p3+0xc)); result -> `p1`.
pub fn ee_sub_125738<M: QMem + ?Sized>(m: &mut M, p1: u32, _p2: u32, p3: u32) -> u32 {
    let sp = stack_push(m, 0x10);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    let (a4, ac) = (m.r32(p3 + 4), m.r32(p3 + 0xc));
    m.call(0x001d_af38, &[a4, ac], &[]);
    m.call(0x0012_91a8, &[p1, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x10);
    p1
}

/// FUN_001256e0 (Bif138): FUN_00129598(val, FUN_001daea8(*(p3+4)) != 0) (bool value) -> copied into `p1`.
pub fn ee_sub_1256e0<M: QMem + ?Sized>(m: &mut M, p1: u32, _p2: u32, p3: u32) -> u32 {
    let sp = stack_push(m, 0x10);
    m.w32(sp, 0xffff_ffff);
    let a4 = m.r32(p3 + 4);
    let v = m.call(0x001d_aea8, &[a4], &[]).0;
    m.call(0x0012_9598, &[sp, (v != 0) as u32], &[]);
    m.call(0x0012_91a8, &[p1, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x10);
    p1
}

/// FUN_00123dd8 (Bif106): FUN_00133130(0x3dd490, *(p3+4)); FUN_0012a0b0(p1, 0x24) (Void directly into the
/// return slot); returns `p1`.
pub fn ee_sub_123dd8<M: QMem + ?Sized>(m: &mut M, p1: u32, _p2: u32, p3: u32) -> u32 {
    let a4 = m.r32(p3 + 4);
    m.call(0x0013_3130, &[0x003d_d490, a4], &[]);
    m.call(0x0012_a0b0, &[p1, 0x24], &[]);
    p1
}

/// FUN_00123b60 (Bif103): `p1` = value {tag 0x16, FUN_00154c90(0x2de578)}.
pub fn ee_sub_123b60<M: QMem + ?Sized>(m: &mut M, p1: u32) -> u32 {
    let v = m.call(0x0015_4c90, &[0x002d_e578], &[]).1[0]; // float return
    m.wf(p1 + 4, v);
    m.w32(p1, 0x16);
    p1
}

/// FUN_00123a60 (Bif98): `p1` = value {tag 0x15, FUN_00210a98()}.
pub fn ee_sub_123a60<M: QMem + ?Sized>(m: &mut M, p1: u32) -> u32 {
    let v = m.call(0x0021_0a98, &[], &[]).0;
    m.w32(p1 + 4, v);
    m.w32(p1, 0x15);
    p1
}

/// FUN_00123a98 (Bif99): `p1` = value {tag 0x15, FUN_00210ae0()}.
pub fn ee_sub_123a98<M: QMem + ?Sized>(m: &mut M, p1: u32) -> u32 {
    let v = m.call(0x0021_0ae0, &[], &[]).0;
    m.w32(p1 + 4, v);
    m.w32(p1, 0x15);
    p1
}

/// FUN_0010cfa0 (Splice thunk): FUN_00201360(tmp, 1, p4, &{6}, out); FUN_00129ec8(tmp); FUN_0012a0b0(tmp,
/// 0x24); FUN_001bbcf0(p2, out[0]); result -> copied into `p1`.
pub fn ee_sub_10cfa0<M: QMem + ?Sized>(m: &mut M, p1: u32, p2: u32, _p3: u32, p4: u32) -> u32 {
    let sp = stack_push(m, 0x50);
    let (val, six, out) = (sp, sp + 0x10, sp + 0x14);
    m.w32(six, 6);
    m.call(0x0020_1360, &[val, 1, p4, six, out], &[]);
    m.call(0x0012_9ec8, &[val], &[]);
    m.call(0x0012_a0b0, &[val, 0x24], &[]);
    let o0 = m.r32(out);
    m.call(0x001b_bcf0, &[p2, o0], &[]);
    m.call(0x0012_91a8, &[p1, val], &[]);
    m.call(0x0012_9ec8, &[val], &[]);
    stack_pop(m, 0x50);
    p1
}

// ---------------------------------------------------------------------------------------------------
// Camera-track heading (WP-I3).
// ---------------------------------------------------------------------------------------------------

/// FUN_00221ce8 (follow-track update): track = `*(o+0x10)`; nothing unless track and (its mode `+0x10c`
/// or `+0x12c`). FUN_00220dd8(*(o+0x14), p2, tgt, 0) fills a quad local `tgt`. Mode 0 (heading track):
/// d = FUN_001a6f00(0, track, tgt[0]) - *(track+0x150); heading = atan2(d.y, d.x) (FUN_002458f0), wrapped
/// to [0, 2pi); `*(track+0x140) = heading * *(track+0x160)`. Mode != 0: FUN_001e7380(*(track+0x140),
/// mode, tgt[0], 0,0,0,0, track+0x140) (spline track).
pub fn ee_sub_221ce8<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32) {
    let t = m.r32(o + 0x10);
    if t == 0 {
        return;
    }
    let mode = m.r32(t + 0x10c);
    if mode == 0 && m.r32(t + 0x12c) == 0 {
        return;
    }
    let u2 = m.r32(o + 0x14);
    let sp = stack_push(m, 0x10);
    m.call(0x0022_0dd8, &[u2, p2, sp, 0], &[]);
    let a30 = m.r32(sp);
    if mode == 0 {
        let res = m.call(0x001a_6f00, &[0, t, a30], &[]).1;
        let d = vu0::sub(res, m.rv(t + 0x150));
        let mut h = callf(m, 0x0024_58f0, &[], &[s(d[1]), s(d[0])]);
        if h < 0.0 {
            h += 6.283_185_5;
        }
        let k = m.rf(t + 0x160);
        m.wf(t + 0x140, h * k);
    } else {
        let p1 = m.r32(t + 0x140);
        ee_sub_1e7380(m, p1, mode, a30, 0, 0, 0, 0);
    }
    stack_pop(m, 0x10);
}

/// FUN_001e7380 (track spline helper): obj = FUN_001e64c8(); FUN_001e6ed8(obj, p1, a1, p2, p3, p4, p5, p6).
/// The decompile shows a stray 128-bit a1 move (`por`); the caller (0x221ce8) passes `a1` as the track
/// mode integer, so it is forwarded as an integer. FUN_001e6ed8 (the spline evaluator) is unported.
#[allow(clippy::too_many_arguments)]
pub fn ee_sub_1e7380<M: QMem + ?Sized>(m: &mut M, p1: u32, a1: u32, p2: u32, p3: u32, p4: u32, p5: u32, p6: u32) {
    let obj = m.call(0x001e_64c8, &[], &[]).0;
    m.call(0x001e_6ed8, &[obj, p1, a1, p2, p3, p4, p5, p6], &[]);
}

// ---------------------------------------------------------------------------------------------------
// PS2 recursive-lock kernel wrappers (WP-Q2). The syscalls (GetThreadId 0x234910, CreateSema 0x234a20,
// WaitSema 0x234a60, SignalSema 0x234a40, ChangeThreadPriority 0x2348b0, CreateThread 0x234820,
// FUN_002333b4 memset) are routed by address like the other render-thread ports.
// ---------------------------------------------------------------------------------------------------

/// FUN_0021efd0 (CreateSema wrapper): zero a 0x18-byte sema param (FUN_002333b4), set +4 = init (`p2`),
/// +8 = max (`p1`), return CreateSema(param).
pub fn ee_sub_21efd0<M: QMem + ?Sized>(m: &mut M, p1: u32, p2: u32) -> u32 {
    let sp = stack_push(m, 0x20);
    m.call(0x0023_33b4, &[sp, 0, 0x18], &[]);
    m.w32(sp + 4, p2);
    m.w32(sp + 8, p1);
    let r = m.call(0x0023_4a20, &[sp], &[]).0;
    stack_pop(m, 0x20);
    r
}

/// FUN_0021f028 (recursive-lock init): `*(o+4) = -1`; `*(o+8) = FUN_0021efd0(1, 1)` (the guard sema).
pub fn ee_sub_21f028<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.w32(o + 4, 0xffff_ffff);
    let r = ee_sub_21efd0(m, 1, 1);
    m.w32(o + 8, r);
}

/// FUN_0021f060 (recursive lock): owner (`*(o+4)`) is this thread -> ++count (`*o`); else WaitSema(*(o+8)),
/// owner = thread id, count = 1.
pub fn ee_sub_21f060<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let tid = m.call(0x0023_4910, &[], &[]).0;
    if tid == m.r32(o + 4) {
        let n = m.r32(o).wrapping_add(1);
        m.w32(o, n);
    } else {
        let sema = m.r32(o + 8);
        m.call(0x0023_4a60, &[sema], &[]);
        m.w32(o + 4, tid);
        m.w32(o, 1);
    }
}

/// FUN_0021f0c0 (recursive unlock): --count (`*o`); at 0 clear owner (`*(o+4) = -1`) and SignalSema(*(o+8)).
pub fn ee_sub_21f0c0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let n = m.r32(o).wrapping_sub(1);
    m.w32(o, n);
    if n == 0 {
        m.w32(o + 4, 0xffff_ffff);
        let sema = m.r32(o + 8);
        m.call(0x0023_4a40, &[sema], &[]);
    }
}

/// FUN_0021f100 (main-thread setup): DAT_004425d0 = GetThreadId(); ChangeThreadPriority(it, 5); build a
/// thread param {+4 entry FUN_00160b70, +8 stack 0x4425f0, +0xc size 0x1000, +0x10 gp 0x2e5270, +0x14
/// prio 2} (zeroed first); DAT_004425d4 = CreateThread(param).
pub fn ee_sub_21f100<M: QMem + ?Sized>(m: &mut M) {
    let tid = m.call(0x0023_4910, &[], &[]).0;
    m.w32(0x0044_25d0, tid);
    m.call(0x0023_48b0, &[tid, 5], &[]);
    let sp = stack_push(m, 0x50);
    m.call(0x0023_33b4, &[sp, 0, 0x30], &[]);
    m.w32(sp + 4, 0x0016_0b70);
    m.w32(sp + 8, 0x0044_25f0);
    m.w32(sp + 0xc, 0x1000);
    m.w32(sp + 0x10, 0x002e_5270);
    m.w32(sp + 0x14, 2);
    let th = m.call(0x0023_4820, &[sp], &[]).0;
    stack_pop(m, 0x50);
    m.w32(0x0044_25d4, th);
}

// ---------------------------------------------------------------------------------------------------
// Miscellaneous.
// ---------------------------------------------------------------------------------------------------

/// FUN_00190c58 (pure VU0 vector math): child = `*(o+0x18c4)`; pos = FUN_001a62e8(child);
/// d = pos - `*(*(o+0x19cc)+0x70)`; len = |d.xyz|; dir = len >= 1e-4 ? d / len : DAT_002a2dc0; returns
/// (pos + (0,0,0,1)) + dir * `*(child+0x134)` (broadcast; a `VADDA ACC, pos, vf0` adds 1.0 to pos.w).
pub fn ee_sub_190c58<M: QMem + ?Sized>(m: &mut M, o: u32) -> V4 {
    let child = m.r32(o + 0x18c4);
    let pos = m.call(0x001a_62e8, &[child], &[]).1;
    let other = m.r32(o + 0x19cc);
    let d = vu0::sub(pos, m.rv(other + 0x70));
    let len = vu0::vsqrt(vu0::dot3(d, d));
    let dir = if f32::from_bits(0x38d1_b717) <= len {
        vu0::scale(d, vu0::vdiv(1.0, len))
    } else {
        rov(0x002a_2dc0)
    };
    let k = m.rf(child + 0x134);
    let acc = vu0::add(pos, [0.0, 0.0, 0.0, 1.0]);
    std::array::from_fn(|i| vu0::fix(acc[i] + dir[i] * k))
}

/// FUN_001d7268 (P22 set-active-character caller): FUN_001f36a8(); then virtual slot 78 (vtable +0x27c /
/// this-adjust +0x278) with arg 0.
pub fn ee_sub_1d7268<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001f_36a8, &[], &[]);
    vslot(m, o, 78, &[0], &[]);
}

/// FUN_00128148 (CRef pool stats copy): six 32-bit copies DAT[src+8] = DAT[src] for src in
/// 0x2dd280, 0x2dd290, 0x2dd2a0, 0x2dd2b0, 0x2dd2c0, 0x2dd2d0 (snapshot current -> previous).
pub fn ee_sub_128148<M: QMem + ?Sized>(m: &mut M) {
    for src in [0x002d_d280, 0x002d_d290, 0x002d_d2a0, 0x002d_d2b0, 0x002d_d2c0, 0x002d_d2d0] {
        let v = m.r32(src);
        m.w32(src + 8, v);
    }
}
