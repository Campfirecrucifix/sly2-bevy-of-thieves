//! WP-Q9: C2cfc20 (brain-driven approach: owner +0x10, anim +0x18, state/pos block +0x30) slot overrides.
use crate::engine::classes::q::q2mem::{QMem, V4};
use crate::engine::classes::q::q9util::*;

/// FUN_0017e9b0: FUN_001ef0d8(this+0x18); e = FUN_001c1ec0(this+0x30, pos(owner)); FUN_00173610(150.0, -250.0, e, 1, 0) == 0 ->
/// FUN_0017d608(e); owner+0xfe8 bit 0 clear: FUN_0020b210(); e -= owner+0x240; owner virtuals 0xec(e) / 0x11c / 0x124;
/// else brain 0xac: e -> brain+0x50, brain+0x68 = 1, run (0x44)
pub fn ee_sub_17e9b0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    // VFIX3 1:1 per asm: e lives in the frame (sp+0): FUN_00173610(a0 = &e, 1, 0, f12 = 150, f13 = -250),
    // FUN_0017d608(&e); e re-read after; virtuals 0x11c / 0x124 get a1 = q DAT_002a2dd0
    use crate::engine::classes::q::q2mem::{stack_pop, stack_push};
    m.call(0x001e_f0d8, &[o + 0x18], &[]);
    let ow = m.r32(o + 0x10);
    let p = m.call(0x001a_62e8, &[ow], &[]).1;
    let e = m.call(0x001c_1ec0, &[o + 0x30], &[p]).1;
    let sp = stack_push(m, 0x10);
    m.wv(sp, e);
    if m.call(0x0017_3610, &[sp, 1, 0], &[s(150.0), s(-250.0)]).0 == 0 {
        m.call(0x0017_d608, &[sp], &[]);
    }
    let ow = m.r32(o + 0x10);
    if m.r32(ow + 0xfe8) & 1 == 0 {
        m.call(0x0020_b210, &[ow], &[]); // VFIX3: a0 = owner (lw a0, 0x10(s1) before the branch)
        let ow = m.r32(o + 0x10);
        let c = m.rv(ow + 0x240);
        let e = m.rv(sp);
        let d: V4 = std::array::from_fn(|i| e[i] - c[i]);
        m.wv(sp, d);
        vcall(m, ow, 0xc, 0xec, &[], &[d]);
        let k = m.rv(0x002a_2dd0);
        let ow = m.r32(o + 0x10);
        vcall(m, ow, 0xc, 0x11c, &[], &[k]);
        let k = m.rv(0x002a_2dd0);
        let ow = m.r32(o + 0x10);
        vcall(m, ow, 0xc, 0x124, &[], &[k]);
    } else {
        let b = m.call(0x0020_5438, &[ow, 0xac], &[]).0;
        let e = m.rv(sp);
        m.wv(b + 0x50, e);
        m.w32(b + 0x68, 1);
        vcall(m, b, 0xc, 0x44, &[], &[]);
    }
    stack_pop(m, 0x10);
}
/// FUN_0017e888 (v6): parent FUN_001eef68; FUN_00209890(owner, 0x2d)
pub fn ee_sub_17e888<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef68, &[o], &[]);
    let ow = m.r32(o + 0x10);
    m.call(0x0020_9890, &[ow, 0x2d], &[]);
}
/// FUN_0017e8b8 (v7): parent FUN_001eef70; FUN_0017e9b0(this)
pub fn ee_sub_17e8b8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef70, &[o], &[]);
    ee_sub_17e9b0(m, o);
}
/// FUN_0017e8e8 (v8): parent FUN_001eef78; FUN_00209970(owner, 0x2d)
pub fn ee_sub_17e8e8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef78, &[o], &[]);
    let ow = m.r32(o + 0x10);
    m.call(0x0020_9970, &[ow, 0x2d], &[]);
}
/// FUN_0017e918 (v9): parent FUN_001eef98; owner+0xfe8 bit 0 clear: FUN_001c1fa0(25.0, this+0x30) == 0 -> FUN_0017e9b0;
/// else FUN_00174fb8(anim) != 0 and FUN_001c1fa0(25.0, this+0x30, owner) == 0 -> FUN_0017e9b0
pub fn ee_sub_17e918<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef98, &[o], &[]);
    let ow = m.r32(o + 0x10);
    if m.r32(ow + 0xfe8) & 1 == 0 {
        if m.call(0x001c_1fa0, &[o + 0x30], &[s(25.0)]).0 == 0 {
            ee_sub_17e9b0(m, o);
        }
    } else {
        let a = m.r32(o + 0x18);
        if m.call(0x0017_4fb8, &[a], &[]).0 != 0 && m.call(0x001c_1fa0, &[o + 0x30, ow], &[s(25.0)]).0 == 0 {
            ee_sub_17e9b0(m, o);
        }
    }
}
