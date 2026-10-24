//! WIRE-S1: 1:1 QMem ports of vm_state PersistentHost script-method natives (approved option b).
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem, V4};

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

/// word copy (ldl/ldr + sdl/sdr / lw + sw of the static arg-type table into the frame)
fn cpw<M: QMem + ?Sized>(m: &mut M, dst: u32, src: u32, n: u32) {
    for k in 0..n {
        let w = m.r32(src + 4 * k);
        m.w32(dst + 4 * k, w);
    }
}

/// FUN_00123b00(r): bif 101 splice: FUN_001294c0(r, DAT_003e0fa0); returns r.
pub fn ee_sub_123b00<M: QMem + ?Sized>(m: &mut M, r: u32) -> u32 {
    let a1 = m.r32(0x003e_0fa0);
    m.call(0x0012_94c0, &[r, a1], &[]);
    r
}

/// FUN_0010e248(ret, obj, _, argv): script method, 0 args: bool result of FUN_001cb2c0(obj).
pub fn ee_sub_10e248<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    let sp = stack_push(m, 0x40);
    m.call(0x0020_1360, &[sp, 0, argv, 0], &[]);
    let v = m.call(0x001c_b2c0, &[obj], &[]).0;
    m.call(0x0012_9598, &[sp, (v != 0) as u32], &[]);
    m.call(0x0012_91a8, &[ret, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x40);
    ret
}

/// FUN_001023b8(ret, obj, _, argv): script method, 0 args: calls FUN_001feec0(obj), returns void value.
pub fn ee_sub_1023b8<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    let sp = stack_push(m, 0x40);
    m.call(0x0020_1360, &[sp, 0, argv, 0], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    m.call(0x001f_eec0, &[obj], &[]);
    m.call(0x0012_91a8, &[ret, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x40);
    ret
}

/// FUN_00105a78(ret, obj, _, argv): script method, 0 args: calls FUN_0015a160(obj), returns void value.
pub fn ee_sub_105a78<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    let sp = stack_push(m, 0x40);
    m.call(0x0020_1360, &[sp, 0, argv, 0], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    m.call(0x0015_a160, &[obj], &[]);
    m.call(0x0012_91a8, &[ret, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x40);
    ret
}

/// FUN_0010b1b8(ret, obj, _, argv): script method, 0 args: calls FUN_001433c8(obj), returns void value.
pub fn ee_sub_10b1b8<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    let sp = stack_push(m, 0x40);
    m.call(0x0020_1360, &[sp, 0, argv, 0], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    m.call(0x0014_33c8, &[obj], &[]);
    m.call(0x0012_91a8, &[ret, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x40);
    ret
}

/// FUN_0010d8e0(ret, obj, _, argv): script method, 0 args: calls FUN_00156780(obj), returns void value.
pub fn ee_sub_10d8e0<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    let sp = stack_push(m, 0x40);
    m.call(0x0020_1360, &[sp, 0, argv, 0], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    m.call(0x0015_6780, &[obj], &[]);
    m.call(0x0012_91a8, &[ret, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x40);
    ret
}

/// FUN_0010f208(ret, obj, _, argv): script method, 0 args: calls FUN_00164aa8(obj), returns void value.
pub fn ee_sub_10f208<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    let sp = stack_push(m, 0x40);
    m.call(0x0020_1360, &[sp, 0, argv, 0], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    m.call(0x0016_4aa8, &[obj], &[]);
    m.call(0x0012_91a8, &[ret, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x40);
    ret
}

/// FUN_001102c0(ret, obj, _, argv): script method, 0 args: calls FUN_00181728(obj), returns void value.
pub fn ee_sub_1102c0<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    let sp = stack_push(m, 0x40);
    m.call(0x0020_1360, &[sp, 0, argv, 0], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    m.call(0x0018_1728, &[obj], &[]);
    m.call(0x0012_91a8, &[ret, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x40);
    ret
}

/// FUN_00104af0(ret, obj, _, argv): script method (int arg type 6): float result of FUN_001f11a8(obj, a).
pub fn ee_sub_104af0<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    let sp = stack_push(m, 0x50);
    m.w32(sp + 0x10, 6);
    m.call(0x0020_1360, &[sp, 1, argv, sp + 0x10, sp + 0x14], &[]);
    let a = m.r32(sp + 0x14);
    let f = m.call(0x001f_11a8, &[obj, a], &[]).1[0];
    m.call(0x0012_9558, &[sp], &[f4(f)]);
    m.call(0x0012_91a8, &[ret, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x50);
    ret
}

/// FUN_001107b8(ret, obj, _, argv): script method (arg type 0x1065): int result of FUN_001d4d00(obj, a).
pub fn ee_sub_1107b8<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    let sp = stack_push(m, 0x50);
    m.w32(sp + 0x10, 0x1065);
    m.call(0x0020_1360, &[sp, 1, argv, sp + 0x10, sp + 0x14], &[]);
    let a = m.r32(sp + 0x14);
    let v = m.call(0x001d_4d00, &[obj, a], &[]).0;
    m.call(0x0012_9518, &[sp, v], &[]);
    m.call(0x0012_91a8, &[ret, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x50);
    ret
}

/// FUN_00105978(ret, obj, _, argv): script method (float arg, type 1): FUN_0015a138(obj, f), returns void.
pub fn ee_sub_105978<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    let sp = stack_push(m, 0x50);
    m.w32(sp + 0x10, 1);
    m.call(0x0020_1360, &[sp, 1, argv, sp + 0x10, sp + 0x14], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    let f = m.rf(sp + 0x14);
    m.call(0x0015_a138, &[obj], &[f4(f)]);
    m.call(0x0012_91a8, &[ret, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x50);
    ret
}

/// FUN_00101be0(ret, obj, _, argv): script method (1 arg, type 5): calls FUN_00131a40(obj, a), returns void value.
pub fn ee_sub_101be0<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    let sp = stack_push(m, 0x50);
    m.w32(sp + 0x10, 5);
    m.call(0x0020_1360, &[sp, 1, argv, sp + 0x10, sp + 0x14], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    let a = m.r32(sp + 0x14);
    m.call(0x0013_1a40, &[obj, a], &[]);
    m.call(0x0012_91a8, &[ret, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x50);
    ret
}

/// FUN_00103d28(ret, obj, _, argv): script method (1 arg, type 6): calls FUN_001f17b0(obj, a), returns void value.
pub fn ee_sub_103d28<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    let sp = stack_push(m, 0x50);
    m.w32(sp + 0x10, 6);
    m.call(0x0020_1360, &[sp, 1, argv, sp + 0x10, sp + 0x14], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    let a = m.r32(sp + 0x14);
    m.call(0x001f_17b0, &[obj, a], &[]);
    m.call(0x0012_91a8, &[ret, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x50);
    ret
}

/// FUN_0010b510(ret, obj, _, argv): script method (1 arg, type 0x2012): calls FUN_0018de18(obj, a), returns void value.
pub fn ee_sub_10b510<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    let sp = stack_push(m, 0x50);
    m.w32(sp + 0x10, 0x2012);
    m.call(0x0020_1360, &[sp, 1, argv, sp + 0x10, sp + 0x14], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    let a = m.r32(sp + 0x14);
    m.call(0x0018_de18, &[obj, a], &[]);
    m.call(0x0012_91a8, &[ret, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x50);
    ret
}

/// FUN_0010d1d8(ret, obj, _, argv): script method (1 arg, type 6): calls FUN_001baf18(obj, a), returns void value.
pub fn ee_sub_10d1d8<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    let sp = stack_push(m, 0x50);
    m.w32(sp + 0x10, 6);
    m.call(0x0020_1360, &[sp, 1, argv, sp + 0x10, sp + 0x14], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    let a = m.r32(sp + 0x14);
    m.call(0x001b_af18, &[obj, a], &[]);
    m.call(0x0012_91a8, &[ret, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x50);
    ret
}

/// FUN_0010dbf8(ret, obj, _, argv): script method (1 arg, type 6): calls FUN_00213b00(obj, a), returns void value.
pub fn ee_sub_10dbf8<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    let sp = stack_push(m, 0x50);
    m.w32(sp + 0x10, 6);
    m.call(0x0020_1360, &[sp, 1, argv, sp + 0x10, sp + 0x14], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    let a = m.r32(sp + 0x14);
    m.call(0x0021_3b00, &[obj, a], &[]);
    m.call(0x0012_91a8, &[ret, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x50);
    ret
}

/// FUN_0010e398(ret, obj, _, argv): script method (1 arg, type 0x2002): calls FUN_001cb488(obj, a), returns void value.
pub fn ee_sub_10e398<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    let sp = stack_push(m, 0x50);
    m.w32(sp + 0x10, 0x2002);
    m.call(0x0020_1360, &[sp, 1, argv, sp + 0x10, sp + 0x14], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    let a = m.r32(sp + 0x14);
    m.call(0x001c_b488, &[obj, a], &[]);
    m.call(0x0012_91a8, &[ret, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x50);
    ret
}

/// FUN_001100b8(ret, obj, _, argv): script method (1 arg, type 0x2030): calls FUN_00180478(obj, a), returns void value.
pub fn ee_sub_1100b8<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    let sp = stack_push(m, 0x50);
    m.w32(sp + 0x10, 0x2030);
    m.call(0x0020_1360, &[sp, 1, argv, sp + 0x10, sp + 0x14], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    let a = m.r32(sp + 0x14);
    m.call(0x0018_0478, &[obj, a], &[]);
    m.call(0x0012_91a8, &[ret, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x50);
    ret
}

/// FUN_00100c80(ret, obj, _, argv): script method (quad arg, type 2): bool result of obj vt+0x248 virtual(q).
pub fn ee_sub_100c80<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    let sp = stack_push(m, 0x70);
    let s0 = sp + 0x10;
    m.w32(sp + 0x20, 2);
    m.call(0x0020_1360, &[s0, 1, argv, sp + 0x20, sp], &[]);
    let vt = m.r32(obj + 0xc);
    let adj = m.r16(vt + 0x248) as i16 as i32;
    let f = m.r32(vt + 0x24c);
    let q = m.rv(sp);
    let v = m.call(f, &[obj.wrapping_add(adj as u32)], &[q]).0;
    m.call(0x0012_9598, &[s0, (v != 0) as u32], &[]);
    m.call(0x0012_91a8, &[ret, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0x70);
    ret
}

/// FUN_0010f648(ret, obj, _, argv): script method (value arg, type 0x8006, init -1): FUN_001d7780(obj, &v).
pub fn ee_sub_10f648<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    let sp = stack_push(m, 0x70);
    let s0 = sp + 0x10;
    m.w32(sp + 0x20, 0x8006);
    m.w32(sp, u32::MAX);
    m.call(0x0020_1360, &[s0, 1, argv, sp + 0x20, sp], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_a0b0, &[s0, 0x24], &[]);
    m.call(0x001d_7780, &[obj, sp], &[]);
    m.call(0x0012_91a8, &[ret, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x70);
    ret
}

/// FUN_00103f40(ret, obj, _, argv): script method (types @0x2a3b78: int, quad): FUN_0020a100(obj, a, q), returns void.
pub fn ee_sub_103f40<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    let sp = stack_push(m, 0x80);
    let s0 = sp + 0x20;
    cpw(m, sp, 0x002a_3b78, 2);
    m.call(0x0020_1360, &[s0, 2, argv, sp, sp + 0x30, sp + 0x10], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_a0b0, &[s0, 0x24], &[]);
    let a = m.r32(sp + 0x30);
    let q = m.rv(sp + 0x10);
    m.call(0x0020_a100, &[obj, a], &[q]);
    m.call(0x0012_91a8, &[ret, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0x80);
    ret
}

/// FUN_0010a940(ret, obj, _, argv): script method (types @0x2a3b78: int, quad): FUN_00142540(obj, a, q), returns void.
pub fn ee_sub_10a940<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    let sp = stack_push(m, 0x80);
    let s0 = sp + 0x20;
    cpw(m, sp, 0x002a_3b78, 2);
    m.call(0x0020_1360, &[s0, 2, argv, sp, sp + 0x30, sp + 0x10], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_a0b0, &[s0, 0x24], &[]);
    let a = m.r32(sp + 0x30);
    let q = m.rv(sp + 0x10);
    m.call(0x0014_2540, &[obj, a], &[q]);
    m.call(0x0012_91a8, &[ret, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0x80);
    ret
}

/// FUN_0010bb38(ret, obj, _, argv): script method (types @0x002a_3c08, 2 int args): FUN_00228920(obj, a, b), returns void.
pub fn ee_sub_10bb38<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    let sp = stack_push(m, 0x70);
    let s0 = sp + 0x10;
    cpw(m, sp, 0x002a_3c08, 2);
    m.call(0x0020_1360, &[s0, 2, argv, sp, sp + 0x20, sp + 0x24], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_a0b0, &[s0, 0x24], &[]);
    let a = m.r32(sp + 0x20);
    let b = m.r32(sp + 0x24);
    m.call(0x0022_8920, &[obj, a, b], &[]);
    m.call(0x0012_91a8, &[ret, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0x70);
    ret
}

/// FUN_0010fd00(ret, obj, _, argv): script method (types @0x002a_3c70, 2 int args): FUN_001800a0(obj, a, b), returns void.
pub fn ee_sub_10fd00<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    let sp = stack_push(m, 0x70);
    let s0 = sp + 0x10;
    cpw(m, sp, 0x002a_3c70, 2);
    m.call(0x0020_1360, &[s0, 2, argv, sp, sp + 0x20, sp + 0x24], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_a0b0, &[s0, 0x24], &[]);
    let a = m.r32(sp + 0x20);
    let b = m.r32(sp + 0x24);
    m.call(0x0018_00a0, &[obj, a, b], &[]);
    m.call(0x0012_91a8, &[ret, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0x70);
    ret
}

/// FUN_0010ddb0(ret, obj, _, argv): script method (types @0x2a3f70: float, int): FUN_00212f80(obj, b, f).
pub fn ee_sub_10ddb0<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    let sp = stack_push(m, 0x70);
    let s0 = sp + 0x10;
    cpw(m, sp, 0x002a_3f70, 2);
    m.call(0x0020_1360, &[s0, 2, argv, sp, sp + 0x20, sp + 0x24], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_a0b0, &[s0, 0x24], &[]);
    let b = m.r32(sp + 0x24);
    let f = m.rf(sp + 0x20);
    m.call(0x0021_2f80, &[obj, b], &[f4(f)]);
    m.call(0x0012_91a8, &[ret, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0x70);
    ret
}

/// FUN_00101608(ret, obj, _, argv): script method (types @0x2a3a90: int, quad): FUN_001f82b8(obj, a, &q).
pub fn ee_sub_101608<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    let sp = stack_push(m, 0x90);
    let s0 = sp + 0x20;
    let s1 = sp + 0x10;
    cpw(m, sp, 0x002a_3a90, 2);
    m.call(0x0020_1360, &[s0, 2, argv, sp, sp + 0x30, s1], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_a0b0, &[s0, 0x24], &[]);
    let a = m.r32(sp + 0x30);
    m.call(0x001f_82b8, &[obj, a, s1], &[]);
    m.call(0x0012_91a8, &[ret, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0x90);
    ret
}

/// FUN_0010a240(ret, obj, _, argv): script method (types @0x2a3e90: int, float, int): FUN_001e1970(obj, a, c, f).
pub fn ee_sub_10a240<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    let sp = stack_push(m, 0x70);
    let s0 = sp + 0x10;
    cpw(m, sp, 0x002a_3e90, 3);
    m.call(0x0020_1360, &[s0, 3, argv, sp, sp + 0x20, sp + 0x24, sp + 0x28], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_a0b0, &[s0, 0x24], &[]);
    let a = m.r32(sp + 0x20);
    let f = m.rf(sp + 0x24);
    let c = m.r32(sp + 0x28);
    m.call(0x001e_1970, &[obj, a, c], &[f4(f)]);
    m.call(0x0012_91a8, &[ret, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0x70);
    ret
}

/// FUN_0010c120(ret, obj, _, argv): script method (types @0x2a3f30: int, float, int): FUN_001ff108(obj, a, c, f).
pub fn ee_sub_10c120<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    let sp = stack_push(m, 0x70);
    let s0 = sp + 0x10;
    cpw(m, sp, 0x002a_3f30, 3);
    m.call(0x0020_1360, &[s0, 3, argv, sp, sp + 0x20, sp + 0x24, sp + 0x28], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_a0b0, &[s0, 0x24], &[]);
    let a = m.r32(sp + 0x20);
    let f = m.rf(sp + 0x24);
    let c = m.r32(sp + 0x28);
    m.call(0x001f_f108, &[obj, a, c], &[f4(f)]);
    m.call(0x0012_91a8, &[ret, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0x70);
    ret
}

/// FUN_0010fda8(ret, obj, _, argv): script method (types @0x002a_4078, 3 int args): FUN_001800d8(obj, a, b, c).
pub fn ee_sub_10fda8<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    let sp = stack_push(m, 0x70);
    let s0 = sp + 0x10;
    cpw(m, sp, 0x002a_4078, 3);
    m.call(0x0020_1360, &[s0, 3, argv, sp, sp + 0x20, sp + 0x24, sp + 0x28], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_a0b0, &[s0, 0x24], &[]);
    let a = m.r32(sp + 0x20);
    let b = m.r32(sp + 0x24);
    let c = m.r32(sp + 0x28);
    m.call(0x0018_00d8, &[obj, a, b, c], &[]);
    m.call(0x0012_91a8, &[ret, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0x70);
    ret
}

/// FUN_00105440(ret, obj, _, argv): script method (types @0x002a_3bf8, 4 int args): FUN_00149808(obj, a, b, c, d).
pub fn ee_sub_105440<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    let sp = stack_push(m, 0x70);
    let s0 = sp + 0x10;
    cpw(m, sp, 0x002a_3bf8, 4);
    m.call(0x0020_1360, &[s0, 4, argv, sp, sp + 0x20, sp + 0x24, sp + 0x28, sp + 0x2c], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_a0b0, &[s0, 0x24], &[]);
    let a = m.r32(sp + 0x20);
    let b = m.r32(sp + 0x24);
    let c = m.r32(sp + 0x28);
    let d = m.r32(sp + 0x2c);
    m.call(0x0014_9808, &[obj, a, b, c, d], &[]);
    m.call(0x0012_91a8, &[ret, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0x70);
    ret
}

/// FUN_001105b8(ret, obj, _, argv): script method (types @0x002a_4048, 4 int args): FUN_001d4b90(obj, a, b, c, d).
pub fn ee_sub_1105b8<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    let sp = stack_push(m, 0x70);
    let s0 = sp + 0x10;
    cpw(m, sp, 0x002a_4048, 4);
    m.call(0x0020_1360, &[s0, 4, argv, sp, sp + 0x20, sp + 0x24, sp + 0x28, sp + 0x2c], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_a0b0, &[s0, 0x24], &[]);
    let a = m.r32(sp + 0x20);
    let b = m.r32(sp + 0x24);
    let c = m.r32(sp + 0x28);
    let d = m.r32(sp + 0x2c);
    m.call(0x001d_4b90, &[obj, a, b, c, d], &[]);
    m.call(0x0012_91a8, &[ret, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0x70);
    ret
}

/// FUN_0010e820(ret, obj, _, argv): script method (6 args, types @0x2a3fc0: int, 4 floats, quad):
/// FUN_001f8bf0(obj, a, q, f0..f3); arg ptrs 5/6 (sp+0x60, sp+0x30) passed on the stack.
pub fn ee_sub_10e820<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    let sp = stack_push(m, 0xb0);
    let s0 = sp + 0x40;
    m.w32(sp, sp + 0x60);
    m.w32(sp + 8, sp + 0x30);
    cpw(m, sp + 0x10, 0x002a_3fc0, 6);
    m.call(
        0x0020_1360,
        &[s0, 6, argv, sp + 0x10, sp + 0x50, sp + 0x54, sp + 0x58, sp + 0x5c, sp + 0x60, sp + 0x30],
        &[],
    );
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_a0b0, &[s0, 0x24], &[]);
    let a = m.r32(sp + 0x50);
    let f0 = m.rf(sp + 0x54);
    let f1 = m.rf(sp + 0x58);
    let f2 = m.rf(sp + 0x5c);
    let f3 = m.rf(sp + 0x60);
    let q = m.rv(sp + 0x30);
    m.call(0x001f_8bf0, &[obj, a], &[q, f4(f0), f4(f1), f4(f2), f4(f3)]);
    m.call(0x0012_91a8, &[ret, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0xb0);
    ret
}
