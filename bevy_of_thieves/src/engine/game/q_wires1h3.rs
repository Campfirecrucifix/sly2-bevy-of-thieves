//! WIRE-S1: 1:1 QMem ports of vm_state PersistentHost script-method natives (approved option b).
//! Shape: FUN_00201360 unpacks the script args (type table, out pointers), result written to `ret`
//! (a0) via FUN_001291a8 then the temp released by FUN_00129ec8. Quad (128-bit GPR) args go in `v`.
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem, V4};

const GS: u32 = 0x003d_d490; // game state

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

/// copy `n` words (the ldl/ldr/lw arg-type table copy onto the frame)
fn cpw<M: QMem + ?Sized>(m: &mut M, dst: u32, src: u32, n: u32) {
    for k in 0..n {
        let w = m.r32(src + 4 * k);
        m.w32(dst + 4 * k, w);
    }
}

/// FUN_00123fe8(a0..a3): ref op ensure: FUN_00123eb0(a0, a1, a2, a3, 0x6d), return a0.
pub fn ee_sub_123fe8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32) -> u32 {
    m.call(0x0012_3eb0, &[a0, a1, a2, a3, 0x6d], &[]);
    a0
}

/// FUN_001244e8(ret, _, ctx, a3): bif 115: object of FUN_0015e958(GS, 4, ctx+4, ctx+0xc, owner+0x3c
/// or GS+0x3f0c) into ret.
pub fn ee_sub_1244e8<M: QMem + ?Sized>(m: &mut M, a0: u32, _a1: u32, a2: u32, _a3: u32) -> u32 {
    let sp = stack_push(m, 0x30);
    let a3 = m.r32(a2 + 0xc);
    m.w32(sp, u32::MAX);
    let mut t0 = if m.r32(a2 + 0x10) != 0 {
        let p = m.r32(a2 + 0x14);
        m.r32(p + 0xc)
    } else {
        a3
    };
    let a2n = m.r32(a2 + 4);
    t0 = if t0 != 0 { m.r32(t0 + 0x3c) } else { m.r32(GS + 0x3f0c) };
    let v = m.call(0x0015_e958, &[GS, 4, a2n, a3, t0], &[]).0;
    m.call(0x0012_9a10, &[sp, v], &[]);
    m.call(0x0012_91a8, &[a0, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x30);
    a0
}

/// FUN_00125510(ret, _, ctx, p): bif 133: walk p (+8 == 1) via +0xc to the first with +0x24 set, then
/// FUN_00127db8(p+0xc, ctx+4, 1) into ret; else error {0x19, 0x86}.
pub fn ee_sub_125510<M: QMem + ?Sized>(m: &mut M, a0: u32, _a1: u32, a2: u32, a3: u32) -> u32 {
    let mut p = a3;
    let mut ok = false;
    while p != 0 && m.r32(p + 8) == 1 {
        if m.r32(p + 0x24) != 0 {
            ok = true;
            break;
        }
        p = m.r32(p + 0xc);
    }
    if ok {
        let x = m.r32(p + 0xc);
        let c = m.r32(a2 + 4);
        let v = m.call(0x0012_7db8, &[x, c, 1], &[]).0;
        if v != 0 {
            m.call(0x0012_91a8, &[a0, v], &[]);
            return a0;
        }
    }
    m.w32(a0, 0x19);
    m.w32(a0 + 4, 0x86);
    a0
}

/// FUN_001022c8(ret, o, _, args): script method: no args, FUN_001feea0(o), ret = tag 0x24.
pub fn ee_sub_1022c8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x40);
    m.call(0x0020_1360, &[sp, 0, a3, 0], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    m.call(0x001f_eea0, &[a1], &[]);
    m.call(0x0012_91a8, &[a0, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x40);
    a0
}

/// FUN_001055b0(ret, o, _, args): script method: no args, FUN_00149738(o), ret = tag 0x24.
pub fn ee_sub_1055b0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x40);
    m.call(0x0020_1360, &[sp, 0, a3, 0], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    m.call(0x0014_9738, &[a1], &[]);
    m.call(0x0012_91a8, &[a0, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x40);
    a0
}

/// FUN_0010afb0(ret, o, _, args): script method: no args, FUN_00142860(o), ret = tag 0x24.
pub fn ee_sub_10afb0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x40);
    m.call(0x0020_1360, &[sp, 0, a3, 0], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    m.call(0x0014_2860, &[a1], &[]);
    m.call(0x0012_91a8, &[a0, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x40);
    a0
}

/// FUN_0010d370(ret, o, _, args): script method: no args, FUN_001b9880(o), ret = tag 0x24.
pub fn ee_sub_10d370<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x40);
    m.call(0x0020_1360, &[sp, 0, a3, 0], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    m.call(0x001b_9880, &[a1], &[]);
    m.call(0x0012_91a8, &[a0, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x40);
    a0
}

/// FUN_0010eb10(ret, o, _, args): script method: no args, FUN_00162e38(o), ret = tag 0x24.
pub fn ee_sub_10eb10<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x40);
    m.call(0x0020_1360, &[sp, 0, a3, 0], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    m.call(0x0016_2e38, &[a1], &[]);
    m.call(0x0012_91a8, &[a0, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x40);
    a0
}

/// FUN_0010f3e8(ret, o, _, args): script method: no args, FUN_00163c28(o), ret = tag 0x24.
pub fn ee_sub_10f3e8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x40);
    m.call(0x0020_1360, &[sp, 0, a3, 0], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    m.call(0x0016_3c28, &[a1], &[]);
    m.call(0x0012_91a8, &[a0, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x40);
    a0
}

/// FUN_0010d158(ret, o, _, args): script method: no args, FUN_001baed8(o), ret = tag 0x24.
pub fn ee_sub_10d158<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x40);
    m.call(0x0020_1360, &[sp, 0, a3, 0], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    m.call(0x001b_aed8, &[a1, 0, 0], &[]);
    m.call(0x0012_91a8, &[a0, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x40);
    a0
}

/// FUN_0010ab88(ret, o, _, args): script method: no args, ret = bool FUN_001424d0(o).
pub fn ee_sub_10ab88<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x40);
    m.call(0x0020_1360, &[sp, 0, a3, 0], &[]);
    let v = m.call(0x0014_24d0, &[a1], &[]).0;
    m.call(0x0012_9598, &[sp, (v != 0) as u32], &[]);
    m.call(0x0012_91a8, &[a0, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x40);
    a0
}

/// FUN_00104108(ret, o, _, args): script method: 1 arg (type 0x2002), ret = int FUN_00154158(o+0xf64, x).
pub fn ee_sub_104108<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x50);
    m.w32(sp + 0x10, 0x2002);
    m.call(0x0020_1360, &[sp, 1, a3, sp + 0x10, sp + 0x14], &[]);
    let x = m.r32(sp + 0x14);
    let o = m.r32(a1 + 0xf64);
    let v = m.call(0x0015_4158, &[o, x], &[]).0;
    m.call(0x0012_9518, &[sp, v], &[]);
    m.call(0x0012_91a8, &[a0, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x50);
    a0
}

/// FUN_0010efb0(ret, o, _, args): script method: 1 arg (type 0x0), FUN_00165118(o, x), ret = tag 0x24.
pub fn ee_sub_10efb0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x50);
    m.w32(sp + 0x10, 0x0);
    m.call(0x0020_1360, &[sp, 1, a3, sp + 0x10, sp + 0x14], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    let x = m.r32(sp + 0x14);
    m.call(0x0016_5118, &[a1, x], &[]);
    m.call(0x0012_91a8, &[a0, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x50);
    a0
}

/// FUN_00103c18(ret, o, _, args): script method: 1 arg (type 0x5), FUN_001f1768(o, x), ret = tag 0x24.
pub fn ee_sub_103c18<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x50);
    m.w32(sp + 0x10, 0x5);
    m.call(0x0020_1360, &[sp, 1, a3, sp + 0x10, sp + 0x14], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    let x = m.r32(sp + 0x14);
    m.call(0x001f_1768, &[a1, x], &[]);
    m.call(0x0012_91a8, &[a0, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x50);
    a0
}

/// FUN_00104d70(ret, o, _, args): script method: 1 arg (type 0x6), FUN_001b2020(o, x), ret = tag 0x24.
pub fn ee_sub_104d70<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x50);
    m.w32(sp + 0x10, 0x6);
    m.call(0x0020_1360, &[sp, 1, a3, sp + 0x10, sp + 0x14], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    let x = m.r32(sp + 0x14);
    m.call(0x001b_2020, &[a1, x], &[]);
    m.call(0x0012_91a8, &[a0, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x50);
    a0
}

/// FUN_0010ac80(ret, o, _, args): script method: 1 arg (type 0x2002), FUN_001421f8(o, x), ret = tag 0x24.
pub fn ee_sub_10ac80<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x50);
    m.w32(sp + 0x10, 0x2002);
    m.call(0x0020_1360, &[sp, 1, a3, sp + 0x10, sp + 0x14], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    let x = m.r32(sp + 0x14);
    m.call(0x0014_21f8, &[a1, x], &[]);
    m.call(0x0012_91a8, &[a0, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x50);
    a0
}

/// FUN_0010c2e8(ret, o, _, args): script method: 1 arg (type 0x2002), FUN_00195608(o, x), ret = tag 0x24.
pub fn ee_sub_10c2e8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x50);
    m.w32(sp + 0x10, 0x2002);
    m.call(0x0020_1360, &[sp, 1, a3, sp + 0x10, sp + 0x14], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    let x = m.r32(sp + 0x14);
    m.call(0x0019_5608, &[a1, x], &[]);
    m.call(0x0012_91a8, &[a0, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x50);
    a0
}

/// FUN_0010d9e0(ret, o, _, args): script method: 1 arg (type 0x1), FUN_00224e70(o, x), ret = tag 0x24.
pub fn ee_sub_10d9e0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x50);
    m.w32(sp + 0x10, 0x1);
    m.call(0x0020_1360, &[sp, 1, a3, sp + 0x10, sp + 0x14], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    let x = m.rf(sp + 0x14);
    m.call(0x0022_4e70, &[a1], &[f4(x)]);
    m.call(0x0012_91a8, &[a0, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x50);
    a0
}

/// FUN_0010e138(ret, o, _, args): script method: 1 arg (type 0x6), FUN_0013c448(o, x), ret = tag 0x24.
pub fn ee_sub_10e138<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x50);
    m.w32(sp + 0x10, 0x6);
    m.call(0x0020_1360, &[sp, 1, a3, sp + 0x10, sp + 0x14], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    let x = m.r32(sp + 0x14);
    m.call(0x0013_c448, &[a1, x], &[]);
    m.call(0x0012_91a8, &[a0, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x50);
    a0
}

/// FUN_0010f508(ret, o, _, args): script method: 1 arg (type 0x10), FUN_001da578(o, x), ret = tag 0x24.
pub fn ee_sub_10f508<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x50);
    m.w32(sp + 0x10, 0x10);
    m.call(0x0020_1360, &[sp, 1, a3, sp + 0x10, sp + 0x14], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    let x = m.r32(sp + 0x14);
    m.call(0x001d_a578, &[a1, x], &[]);
    m.call(0x0012_91a8, &[a0, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x50);
    a0
}

/// FUN_001016c0(ret, o, _, args): script method: 2 args (types at 0x2a3a80), FUN_001ee678(o, x, y), ret = tag 0x24.
pub fn ee_sub_1016c0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x70);
    cpw(m, sp, 0x002a_3a80, 2);
    let s0 = sp + 0x10;
    m.call(0x0020_1360, &[s0, 2, a3, sp, sp + 0x20, sp + 0x24], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_a0b0, &[s0, 0x24], &[]);
    let (x, y) = (m.rf(sp + 0x20), m.rf(sp + 0x24));
    m.call(0x001e_e678, &[a1], &[f4(x), f4(y)]);
    m.call(0x0012_91a8, &[a0, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0x70);
    a0
}

/// FUN_00105398(ret, o, _, args): script method: 2 args (types at 0x2a3b98), FUN_00149798(o, x, y), ret = tag 0x24.
pub fn ee_sub_105398<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x70);
    cpw(m, sp, 0x002a_3b98, 2);
    let s0 = sp + 0x10;
    m.call(0x0020_1360, &[s0, 2, a3, sp, sp + 0x20, sp + 0x24], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_a0b0, &[s0, 0x24], &[]);
    let (x, y) = (m.r32(sp + 0x20), m.r32(sp + 0x24));
    m.call(0x0014_9798, &[a1, x, y], &[]);
    m.call(0x0012_91a8, &[a0, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0x70);
    a0
}

/// FUN_0010b3e8(ret, o, _, args): script method: 2 args (types at 0x2a3f00), FUN_00141fc8(o, x, y), ret = tag 0x24.
pub fn ee_sub_10b3e8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x70);
    cpw(m, sp, 0x002a_3f00, 2);
    let s0 = sp + 0x10;
    m.call(0x0020_1360, &[s0, 2, a3, sp, sp + 0x20, sp + 0x24], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_a0b0, &[s0, 0x24], &[]);
    let (x, y) = (m.r32(sp + 0x20), m.r32(sp + 0x24));
    m.call(0x0014_1fc8, &[a1, x, y], &[]);
    m.call(0x0012_91a8, &[a0, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0x70);
    a0
}

/// FUN_0010d798(ret, o, _, args): script method: 2 args (types at 0x2a3c08), FUN_001840f8(o, x, y), ret = tag 0x24.
pub fn ee_sub_10d798<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x70);
    cpw(m, sp, 0x002a_3c08, 2);
    let s0 = sp + 0x10;
    m.call(0x0020_1360, &[s0, 2, a3, sp, sp + 0x20, sp + 0x24], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_a0b0, &[s0, 0x24], &[]);
    let (x, y) = (m.r32(sp + 0x20), m.r32(sp + 0x24));
    m.call(0x0018_40f8, &[a1, x, y], &[]);
    m.call(0x0012_91a8, &[a0, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0x70);
    a0
}

/// FUN_0010f460(ret, o, _, args): script method: 2 args (types at 0x2a4030), FUN_001da4b8(o, x, y), ret = tag 0x24.
pub fn ee_sub_10f460<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x70);
    cpw(m, sp, 0x002a_4030, 2);
    let s0 = sp + 0x10;
    m.call(0x0020_1360, &[s0, 2, a3, sp, sp + 0x20, sp + 0x24], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_a0b0, &[s0, 0x24], &[]);
    let (x, y) = (m.r32(sp + 0x20), m.rf(sp + 0x24));
    m.call(0x001d_a4b8, &[a1, x], &[f4(y)]);
    m.call(0x0012_91a8, &[a0, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0x70);
    a0
}

/// FUN_00103270(ret, o, _, args): script method: 3 args (types at 0x2a3b60), FUN_00208058(o, x, y, z), ret = tag 0x24.
pub fn ee_sub_103270<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x70);
    cpw(m, sp, 0x002a_3b60, 3);
    let s0 = sp + 0x10;
    m.call(0x0020_1360, &[s0, 3, a3, sp, sp + 0x20, sp + 0x24, sp + 0x28], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_a0b0, &[s0, 0x24], &[]);
    let (x, y, z) = (m.r32(sp + 0x20), m.r32(sp + 0x24), m.r32(sp + 0x28));
    m.call(0x0020_8058, &[a1, x, y, z], &[]);
    m.call(0x0012_91a8, &[a0, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0x70);
    a0
}

/// FUN_0010eda8(ret, o, _, args): script method: 3 args (types at 0x2a3b88), FUN_00162660(o, x, y, z), ret = tag 0x24.
pub fn ee_sub_10eda8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x70);
    cpw(m, sp, 0x002a_3b88, 3);
    let s0 = sp + 0x10;
    m.call(0x0020_1360, &[s0, 3, a3, sp, sp + 0x20, sp + 0x24, sp + 0x28], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_a0b0, &[s0, 0x24], &[]);
    let (x, y, z) = (m.r32(sp + 0x20), m.r32(sp + 0x24), m.r32(sp + 0x28));
    m.call(0x0016_2660, &[a1, x, y, z], &[]);
    m.call(0x0012_91a8, &[a0, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0x70);
    a0
}

/// FUN_0010f6e8(ret, o, _, args): script method: 4 args (types at 0x2a4048), FUN_00155530(o, x, y, z, w), ret = tag 0x24.
pub fn ee_sub_10f6e8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x70);
    cpw(m, sp, 0x002a_4048, 4);
    let s0 = sp + 0x10;
    m.call(0x0020_1360, &[s0, 4, a3, sp, sp + 0x20, sp + 0x24, sp + 0x28, sp + 0x2c], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_a0b0, &[s0, 0x24], &[]);
    let (x, y) = (m.r32(sp + 0x20), m.r32(sp + 0x24));
    let (z, w) = (m.r32(sp + 0x28), m.r32(sp + 0x2c));
    m.call(0x0015_5530, &[a1, x, y, z, w], &[]);
    m.call(0x0012_91a8, &[a0, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0x70);
    a0
}

/// FUN_001048f0(ret, o, _, args): script method: 4 args (types at 0x2a3bd0), ret = object FUN_001f03a8(o, x, y, z, w).
pub fn ee_sub_1048f0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x70);
    cpw(m, sp, 0x002a_3bd0, 4);
    let s0 = sp + 0x10;
    m.call(0x0020_1360, &[s0, 4, a3, sp, sp + 0x20, sp + 0x24, sp + 0x28, sp + 0x2c], &[]);
    let (x, y) = (m.r32(sp + 0x20), m.r32(sp + 0x24));
    let (z, w) = (m.r32(sp + 0x28), m.r32(sp + 0x2c));
    let v = m.call(0x001f_03a8, &[a1, x, y, z, w], &[]).0;
    m.call(0x0012_9a10, &[s0, v], &[]);
    m.call(0x0012_91a8, &[a0, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0x70);
    a0
}

/// FUN_0010f838(ret, o, _, args): script method: 5 args (types at 0x2a4058, 5th out as stack arg),
/// FUN_001524d8(o, 5 args), ret = tag 0x24.
pub fn ee_sub_10f838<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0xa0);
    m.w32(sp, sp + 0x50);
    cpw(m, sp + 0x10, 0x002a_4058, 5);
    let s0 = sp + 0x30;
    m.call(
        0x0020_1360,
        &[s0, 5, a3, sp + 0x10, sp + 0x40, sp + 0x44, sp + 0x48, sp + 0x4c, sp + 0x50],
        &[],
    );
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_a0b0, &[s0, 0x24], &[]);
    let (x, y) = (m.r32(sp + 0x40), m.r32(sp + 0x44));
    let (z, w) = (m.r32(sp + 0x48), m.r32(sp + 0x4c));
    let u = m.r32(sp + 0x50);
    m.call(0x0015_24d8, &[a1, x, y, z, w, u], &[]);
    m.call(0x0012_91a8, &[a0, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0xa0);
    a0
}

/// FUN_001047a0(ret, o, _, args): script method: 2 args (quad + float, types at 0x2a3bb8),
/// ret = bool FUN_00227d20(o, q, f).
pub fn ee_sub_1047a0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x80);
    cpw(m, sp, 0x002a_3bb8, 2);
    let s0 = sp + 0x20;
    m.call(0x0020_1360, &[s0, 2, a3, sp, sp + 0x10, sp + 0x30], &[]);
    let q = m.rv(sp + 0x10);
    let f = m.rf(sp + 0x30);
    let v = m.call(0x0022_7d20, &[a1], &[q, f4(f)]).0;
    m.call(0x0012_9598, &[s0, (v != 0) as u32], &[]);
    m.call(0x0012_91a8, &[a0, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0x80);
    a0
}

/// FUN_0010b618(ret, o, _, args): script method: 3 quad args (types at 0x2a3f08), FUN_00185e30(o, q1, q2, q3).
pub fn ee_sub_10b618<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x90);
    cpw(m, sp, 0x002a_3f08, 3);
    let s0 = sp + 0x40;
    m.call(0x0020_1360, &[s0, 3, a3, sp, sp + 0x10, sp + 0x20, sp + 0x30], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_a0b0, &[s0, 0x24], &[]);
    let (q1, q2, q3) = (m.rv(sp + 0x10), m.rv(sp + 0x20), m.rv(sp + 0x30));
    m.call(0x0018_5e30, &[a1], &[q1, q2, q3]);
    m.call(0x0012_91a8, &[a0, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0x90);
    a0
}

/// FUN_0010b230(ret, o, _, args): script method: 7 args (types at 0x2a3ed8; 3 outs as stack args),
/// FUN_00142310(o, &v40, q50, q60, f80, f84, f88, f8c).
pub fn ee_sub_10b230<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0xe0);
    let s0 = sp + 0x70;
    let s1 = sp + 0x40;
    m.w32(sp + 0x10, sp + 0x60);
    m.w32(sp, sp + 0x8c);
    m.w32(sp + 8, sp + 0x50);
    cpw(m, sp + 0x20, 0x002a_3ed8, 7);
    m.call(
        0x0020_1360,
        &[s0, 7, a3, sp + 0x20, sp + 0x80, sp + 0x84, s1, sp + 0x88, sp + 0x8c, sp + 0x50, sp + 0x60],
        &[],
    );
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_a0b0, &[s0, 0x24], &[]);
    let (f0, f1) = (m.rf(sp + 0x80), m.rf(sp + 0x84));
    let (f2, f3) = (m.rf(sp + 0x88), m.rf(sp + 0x8c));
    let (q2, q3) = (m.rv(sp + 0x50), m.rv(sp + 0x60));
    m.call(0x0014_2310, &[a1, s1], &[q2, q3, f4(f0), f4(f1), f4(f2), f4(f3)]);
    m.call(0x0012_91a8, &[a0, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0xe0);
    a0
}
