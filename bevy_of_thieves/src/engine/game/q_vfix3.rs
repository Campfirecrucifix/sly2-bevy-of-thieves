//! VFIX3: 1:1 QMem ports (from the asm) of struct-model class methods whose struct model cannot express
//! the original memory effects (tagged +0x40 flag pointer, list init, record fields via a1). The struct
//! methods (classes::shared::c71 / k13 / c2cf378 / k21_68) are kept for the class runtime.
use crate::engine::classes::q::q2mem::QMem;

/// +0x40 flags word: bit 0 set = inline flags (+0x40 |= b), else +0x40 points at the shared flags word
fn flags_or<M: QMem + ?Sized>(m: &mut M, o: u32, v: u32, b: u32) {
    if v & 1 != 0 {
        m.w32(o + 0x40, v | b);
    } else {
        let w = m.r32(v);
        m.w32(v, w | b);
    }
}

/// FUN_0017f090 (C71 slot 6): base FUN_001f2108; +0x180 = 10; +0x1a0 = q DAT_002be7d0; +0x1b0 = 8 bytes
/// DAT_002be7e0 (ldl/ldr, sdl/sdr); flags |= 2
pub fn ee_sub_17f090<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001f_2108, &[o], &[]);
    m.w32(o + 0x180, 10);
    for k in 0..4 {
        let w = m.r32(0x002b_e7d0 + 4 * k);
        m.w32(o + 0x1a0 + 4 * k, w);
    }
    for k in 0..8 {
        let b = m.r8(0x002b_e7e0 + k);
        m.w8(o + 0x1b0 + k, b);
    }
    let v = m.r32(o + 0x40);
    flags_or(m, o, v, 2);
}

/// FUN_00228b48 (C2dca30 slot 6): base FUN_001f2108; +0x180 = 1, +0x184 = -10, +0x188 = 0x2000; flags |= 2
pub fn ee_sub_228b48<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001f_2108, &[o], &[]);
    let v = m.r32(o + 0x40);
    m.w32(o + 0x180, 1);
    m.w32(o + 0x184, (-10i32) as u32);
    m.w32(o + 0x188, 0x2000);
    flags_or(m, o, v, 2);
}

/// FUN_00162448 (C2cf378 slot 6): base FUN_001a5270; +0x9c = 480.0, +0xa0 = 720.0; +0xb4/+0x6c/+0x70/+0xac/
/// +0xb0 = -1; FUN_0011a050(this + 0x80, 0x84)
pub fn ee_sub_162448<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001a_5270, &[o], &[]);
    m.wf(o + 0x9c, 480.0);
    m.wf(o + 0xa0, 720.0);
    for off in [0xb4, 0x6c, 0x70, 0xac, 0xb0] {
        m.w32(o + off, u32::MAX);
    }
    m.call(0x0011_a050, &[o + 0x80, 0x84], &[]);
}

/// FUN_00152350 (C2cd748 slot 49): FUN_001f58f0(a0, rec); rec +0x30 -= 3pi/4 (0x4016cbe4); returns it in $f0
pub fn ee_sub_152350<M: QMem + ?Sized>(m: &mut M, o: u32, rec: u32) -> f32 {
    m.call(0x001f_58f0, &[o, rec], &[]);
    let f = crate::engine::math::ps2f::ps2_sub(m.rf(rec + 0x30), f32::from_bits(0x4016_cbe4));
    m.wf(rec + 0x30, f);
    f
}

/// PHYS1: FUN_001cf268 (C2d74b8 slot 4, effect update) 1:1 from the asm: g_clock (DAT_002dded0) - +0x14 >
/// +0x18 (life) -> FUN_001ce980(o); else unless (+0x8)->+0x150 == 1, with owner +0xe0 (and its +0xe8):
/// flag = owner bit 60 of +0x28 ? (owner slot 0x24c (adj 0x248)(+0x70) != 0) : 0; then +0xec slot 0x3c
/// (adj 0x38)(o, flag). (gfx::fx::ee_sub_1cf268_expired is the expiry predicate of the struct model.)
pub fn ee_sub_1cf268<M: QMem + ?Sized>(m: &mut M, o: u32) {
    use crate::engine::math::ps2f::ps2_sub;
    if m.rf(o + 0x18) < ps2_sub(m.rf(0x002d_ded0), m.rf(o + 0x14)) {
        m.call(0x001c_e980, &[o], &[]);
        return;
    }
    let v1 = m.r32(o + 8);
    if m.r32(v1 + 0x150) == 1 {
        return;
    }
    let a2 = m.r32(o + 0xe0);
    if a2 == 0 || m.r32(a2 + 0xe8) == 0 {
        return;
    }
    let mut flag = 0;
    if (m.r64(a2 + 0x28) >> 60) & 1 != 0 {
        let vt = m.r32(a2 + 0xc);
        let adj = m.r16(vt + 0x248) as i16 as i32 as u32;
        let f = m.r32(vt + 0x24c);
        let a1 = m.r32(o + 0x70);
        flag = (m.call(f, &[a2.wrapping_add(adj), a1], &[]).0 != 0) as u32;
    }
    let vt = m.r32(o + 0xec);
    let adj = m.r16(vt + 0x38) as i16 as i32 as u32;
    let f = m.r32(vt + 0x3c);
    m.call(f, &[o.wrapping_add(adj), flag], &[]);
}
