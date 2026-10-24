//! WIRE-S1: 1:1 QMem ports of vm_state PersistentHost script-method natives (approved option b).
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem};

const PARSE: u32 = 0x0020_1360; // FUN_00201360(val, n, argv, types, outs...)
const DTOR: u32 = 0x0012_9ec8;
const SET_TAG: u32 = 0x0012_a0b0;
const ASSIGN: u32 = 0x0012_91a8;
const FROM_BOOL: u32 = 0x0012_9598;
const FROM_F32: u32 = 0x0012_9558;
const FROM_VAL: u32 = 0x0012_9718;
const FROM_INT: u32 = 0x0012_9a10;

/// 0-arg void method: parse(argv), tmp=void(0x24), f(obj), *ret = tmp.
fn void0<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, argv: u32, f: u32) -> u32 {
    let sp = stack_push(m, 0x40);
    m.call(PARSE, &[sp, 0, argv, 0], &[]);
    m.call(DTOR, &[sp], &[]);
    m.call(SET_TAG, &[sp, 0x24], &[]);
    m.call(f, &[obj], &[]);
    m.call(ASSIGN, &[ret, sp], &[]);
    m.call(DTOR, &[sp], &[]);
    stack_pop(m, 0x40);
    ret
}

/// 1-arg (type `ty`) void method: f(obj, arg).
fn void1<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, argv: u32, ty: u32, f: u32) -> u32 {
    let sp = stack_push(m, 0x50);
    m.w32(sp + 0x10, ty);
    m.call(PARSE, &[sp, 1, argv, sp + 0x10, sp + 0x14], &[]);
    m.call(DTOR, &[sp], &[]);
    m.call(SET_TAG, &[sp, 0x24], &[]);
    let a = m.r32(sp + 0x14);
    m.call(f, &[obj, a], &[]);
    m.call(ASSIGN, &[ret, sp], &[]);
    m.call(DTOR, &[sp], &[]);
    stack_pop(m, 0x50);
    ret
}

/// n-arg void method with type table at `tys` (n words); outs at sp+0x20..; returns sp.
fn parse_n<M: QMem + ?Sized>(m: &mut M, argv: u32, tys: u32, n: u32) -> u32 {
    let sp = stack_push(m, 0x70);
    for k in 0..n {
        let t = m.r32(tys + 4 * k);
        m.w32(sp + 4 * k, t);
    }
    let mut a = vec![sp + 0x10, n, argv, sp];
    for k in 0..n {
        a.push(sp + 0x20 + 4 * k);
    }
    m.call(PARSE, &a, &[]);
    sp
}

fn void_tail<M: QMem + ?Sized>(m: &mut M, ret: u32, sp: u32) -> u32 {
    m.call(ASSIGN, &[ret, sp + 0x10], &[]);
    m.call(DTOR, &[sp + 0x10], &[]);
    stack_pop(m, 0x70);
    ret
}

/// 2-arg (int, int) void method: f(obj, a, b).
fn void2i<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, argv: u32, tys: u32, f: u32) -> u32 {
    let sp = parse_n(m, argv, tys, 2);
    m.call(DTOR, &[sp + 0x10], &[]);
    m.call(SET_TAG, &[sp + 0x10, 0x24], &[]);
    let a = m.r32(sp + 0x20);
    let b = m.r32(sp + 0x24);
    m.call(f, &[obj, a, b], &[]);
    void_tail(m, ret, sp)
}

/// 3-arg (int, int, int) void method: f(obj, a, b, c).
fn void3i<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, argv: u32, tys: u32, f: u32) -> u32 {
    let sp = parse_n(m, argv, tys, 3);
    m.call(DTOR, &[sp + 0x10], &[]);
    m.call(SET_TAG, &[sp + 0x10, 0x24], &[]);
    let a = m.r32(sp + 0x20);
    let b = m.r32(sp + 0x24);
    let c = m.r32(sp + 0x28);
    m.call(f, &[obj, a, b, c], &[]);
    void_tail(m, ret, sp)
}

fn fv(x: f32) -> [f32; 4] {
    [x, 0.0, 0.0, 0.0]
}

/// FUN_0010a8d0: bool method: ret = bool(FUN_001423a0(obj) != 0).
pub fn ee_sub_10a8d0<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    let sp = stack_push(m, 0x40);
    m.call(PARSE, &[sp, 0, argv, 0], &[]);
    let v = m.call(0x0014_23a0, &[obj], &[]).0;
    m.call(FROM_BOOL, &[sp, (v != 0) as u32], &[]);
    m.call(ASSIGN, &[ret, sp], &[]);
    m.call(DTOR, &[sp], &[]);
    stack_pop(m, 0x40);
    ret
}

/// FUN_001020d8: void method -> FUN_001fe738(obj).
pub fn ee_sub_1020d8<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    void0(m, ret, obj, argv, 0x001f_e738)
}
/// FUN_00104bf8: void method -> FUN_001f0598(obj).
pub fn ee_sub_104bf8<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    void0(m, ret, obj, argv, 0x001f_0598)
}
/// FUN_00106990: void method -> FUN_001e0120(obj).
pub fn ee_sub_106990<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    void0(m, ret, obj, argv, 0x001e_0120)
}
/// FUN_0010be70: void method -> FUN_001c7a08(obj).
pub fn ee_sub_10be70<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    void0(m, ret, obj, argv, 0x001c_7a08)
}
/// FUN_0010ea20: void method -> FUN_00162dd0(obj).
pub fn ee_sub_10ea20<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    void0(m, ret, obj, argv, 0x0016_2dd0)
}
/// FUN_0010f2f8: void method -> FUN_00164bc8(obj).
pub fn ee_sub_10f2f8<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    void0(m, ret, obj, argv, 0x0016_4bc8)
}

/// FUN_001021d0: float method: ret = float(FUN_001fcfd0(obj, arg type 0x10)).
pub fn ee_sub_1021d0<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    let sp = stack_push(m, 0x50);
    m.w32(sp + 0x10, 0x10);
    m.call(PARSE, &[sp, 1, argv, sp + 0x10, sp + 0x14], &[]);
    let a = m.r32(sp + 0x14);
    let f = m.call(0x001f_cfd0, &[obj, a], &[]).1[0];
    m.call(FROM_F32, &[sp], &[fv(f)]);
    m.call(ASSIGN, &[ret, sp], &[]);
    m.call(DTOR, &[sp], &[]);
    stack_pop(m, 0x50);
    ret
}

/// FUN_0010c720: bool method: ret = bool(FUN_001c4700(obj, arg type 5) != 0).
pub fn ee_sub_10c720<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    let sp = stack_push(m, 0x50);
    m.w32(sp + 0x10, 5);
    m.call(PARSE, &[sp, 1, argv, sp + 0x10, sp + 0x14], &[]);
    let a = m.r32(sp + 0x14);
    let v = m.call(0x001c_4700, &[obj, a], &[]).0;
    m.call(FROM_BOOL, &[sp, (v != 0) as u32], &[]);
    m.call(ASSIGN, &[ret, sp], &[]);
    m.call(DTOR, &[sp], &[]);
    stack_pop(m, 0x50);
    ret
}

/// FUN_0010b598: void method -> FUN_0018df50(obj, arg type 0).
pub fn ee_sub_10b598<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    void1(m, ret, obj, argv, 0, 0x0018_df50)
}
/// FUN_00103498: void method -> FUN_002085b0(obj, arg type 0x2002).
pub fn ee_sub_103498<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    void1(m, ret, obj, argv, 0x2002, 0x0020_85b0)
}
/// FUN_00104a68: void method -> FUN_001f0730(obj, arg type 6).
pub fn ee_sub_104a68<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    void1(m, ret, obj, argv, 6, 0x001f_0730)
}
/// FUN_0010ab00: void method -> FUN_00142340(obj, arg type 0x2002).
pub fn ee_sub_10ab00<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    void1(m, ret, obj, argv, 0x2002, 0x0014_2340)
}
/// FUN_0010c1d8: void method -> FUN_0020efc0(obj, arg type 0x1058).
pub fn ee_sub_10c1d8<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    void1(m, ret, obj, argv, 0x1058, 0x0020_efc0)
}
/// FUN_0010d2e8: void method -> FUN_001b7a78(obj, arg type 0x2002).
pub fn ee_sub_10d2e8<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    void1(m, ret, obj, argv, 0x2002, 0x001b_7a78)
}
/// FUN_0010def8: void method -> FUN_00211e70(obj, arg type 6).
pub fn ee_sub_10def8<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    void1(m, ret, obj, argv, 6, 0x0021_1e70)
}
/// FUN_0010e910: void method -> FUN_00162600(obj, arg type 0x104e).
pub fn ee_sub_10e910<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    void1(m, ret, obj, argv, 0x104e, 0x0016_2600)
}

/// FUN_0010e550: value method: ret = value(FUN_001e77b0(obj, arg type 2)).
pub fn ee_sub_10e550<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    let sp = stack_push(m, 0x70);
    let s0 = sp + 0x10;
    m.w32(sp + 0x20, 2);
    m.call(PARSE, &[s0, 1, argv, sp + 0x20, sp], &[]);
    let a = m.r32(sp);
    let v = m.call(0x001e_77b0, &[obj, a], &[]).0;
    m.call(FROM_VAL, &[s0, v], &[]);
    m.call(ASSIGN, &[ret, s0], &[]);
    m.call(DTOR, &[s0], &[]);
    stack_pop(m, 0x70);
    ret
}

/// FUN_001014e8: int method: ret = int(FUN_001f8510(obj, a, b)), types DAT_002a3a88.
pub fn ee_sub_1014e8<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    let sp = parse_n(m, argv, 0x002a_3a88, 2);
    let a = m.r32(sp + 0x20);
    let b = m.r32(sp + 0x24);
    let v = m.call(0x001f_8510, &[obj, a, b], &[]).0;
    m.call(FROM_INT, &[sp + 0x10, v], &[]);
    void_tail(m, ret, sp)
}

/// FUN_00101218: void method -> FUN_001f7fa0(obj, x, y) (floats), types DAT_002a3a80.
pub fn ee_sub_101218<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    let sp = parse_n(m, argv, 0x002a_3a80, 2);
    m.call(DTOR, &[sp + 0x10], &[]);
    m.call(SET_TAG, &[sp + 0x10, 0x24], &[]);
    let x = m.rf(sp + 0x20);
    let y = m.rf(sp + 0x24);
    m.call(0x001f_7fa0, &[obj], &[fv(x), fv(y)]);
    void_tail(m, ret, sp)
}

/// FUN_00104228: void method -> FUN_00154038(obj, a, b), types DAT_002a3b80.
pub fn ee_sub_104228<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    void2i(m, ret, obj, argv, 0x002a_3b80, 0x0015_4038)
}
/// FUN_0010af08: void method -> FUN_00142830(obj, a, b), types DAT_002a3ed0.
pub fn ee_sub_10af08<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    void2i(m, ret, obj, argv, 0x002a_3ed0, 0x0014_2830)
}
/// FUN_0010bdc8: void method -> FUN_001c7988(obj, a, b), types DAT_002a3f18.
pub fn ee_sub_10bdc8<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    void2i(m, ret, obj, argv, 0x002a_3f18, 0x001c_7988)
}
/// FUN_0010ee60: void method -> FUN_00164640(obj, a, b), types DAT_002a4000.
pub fn ee_sub_10ee60<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    void2i(m, ret, obj, argv, 0x002a_4000, 0x0016_4640)
}

/// FUN_00124730(ret, _, args): find class objects: FUN_0015e9c8(GS, +0xc ? 0x401 : 0x405, +4, +0xc,
/// 0x180, buf), FUN_00124078 list -> value, ret = it.
pub fn ee_sub_124730<M: QMem + ?Sized>(m: &mut M, ret: u32, _a1: u32, args: u32) -> u32 {
    let sp = stack_push(m, 0x660);
    let s1 = sp + 0x10;
    let t0 = m.r32(args + 0xc);
    let a2 = m.r32(args + 4);
    let sel = if t0 == 0 { 0x405 } else { 0x401 };
    m.w32(sp, u32::MAX);
    let n = m.call(0x0015_e9c8, &[0x003d_d490, sel, a2, t0, 0x180, s1], &[]).0;
    let s0 = sp + 0x610;
    m.call(0x0012_4078, &[s0, n, s1], &[]);
    m.call(0x0012_9658, &[sp, s0], &[]);
    m.call(DTOR, &[s0], &[]);
    m.call(ASSIGN, &[ret, sp], &[]);
    m.call(DTOR, &[sp], &[]);
    stack_pop(m, 0x660);
    ret
}

/// FUN_00101c68: void method -> FUN_00130d30(obj, a, x, y), types DAT_002a3af8.
pub fn ee_sub_101c68<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    let sp = parse_n(m, argv, 0x002a_3af8, 3);
    m.call(DTOR, &[sp + 0x10], &[]);
    m.call(SET_TAG, &[sp + 0x10, 0x24], &[]);
    let a = m.r32(sp + 0x20);
    let x = m.rf(sp + 0x24);
    let y = m.rf(sp + 0x28);
    m.call(0x0013_0d30, &[obj, a], &[fv(x), fv(y)]);
    void_tail(m, ret, sp)
}
/// FUN_0010a6d8: void method -> FUN_001e1b10(obj, a, b, c), types DAT_002a3c88.
pub fn ee_sub_10a6d8<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    void3i(m, ret, obj, argv, 0x002a_3c88, 0x001e_1b10)
}
/// FUN_0010d6e0: void method -> FUN_001840b8(obj, a, b, c), types DAT_002a3b88.
pub fn ee_sub_10d6e0<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    void3i(m, ret, obj, argv, 0x002a_3b88, 0x0018_40b8)
}
/// FUN_001108e0: void method -> FUN_001d4ce0(obj, a, b, x), types DAT_002a40e8.
pub fn ee_sub_1108e0<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    let sp = parse_n(m, argv, 0x002a_40e8, 3);
    m.call(DTOR, &[sp + 0x10], &[]);
    m.call(SET_TAG, &[sp + 0x10, 0x24], &[]);
    let a = m.r32(sp + 0x20);
    let b = m.r32(sp + 0x24);
    let x = m.rf(sp + 0x28);
    m.call(0x001d_4ce0, &[obj, a, b], &[fv(x)]);
    void_tail(m, ret, sp)
}

/// FUN_0010e758: void method -> FUN_001f8b58(obj, a, b, x, y), types DAT_002a3fb0.
pub fn ee_sub_10e758<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    let sp = parse_n(m, argv, 0x002a_3fb0, 4);
    m.call(DTOR, &[sp + 0x10], &[]);
    m.call(SET_TAG, &[sp + 0x10, 0x24], &[]);
    let a = m.r32(sp + 0x20);
    let b = m.r32(sp + 0x24);
    let x = m.rf(sp + 0x28);
    let y = m.rf(sp + 0x2c);
    m.call(0x001f_8b58, &[obj, a, b], &[fv(x), fv(y)]);
    void_tail(m, ret, sp)
}

/// FUN_0010ae28: void method -> FUN_00143090(obj, &a, &b, &c) (3 outs), types DAT_002a3ec0.
pub fn ee_sub_10ae28<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    let sp = stack_push(m, 0xc0);
    let (s0, s1, s2, s3) = (sp + 0x40, sp + 0x20, sp + 0x30, sp + 0x10);
    for k in 0..3 {
        let t = m.r32(0x002a_3ec0 + 4 * k);
        m.w32(sp + 4 * k, t);
    }
    m.call(PARSE, &[s0, 3, argv, sp, s3, s1, s2], &[]);
    m.call(DTOR, &[s0], &[]);
    m.call(SET_TAG, &[s0, 0x24], &[]);
    m.call(0x0014_3090, &[obj, s3, s1, s2], &[]);
    m.call(ASSIGN, &[ret, s0], &[]);
    m.call(DTOR, &[s0], &[]);
    stack_pop(m, 0xc0);
    ret
}

/// FUN_00102d30: void method -> FUN_00207cf8(obj, &val, a, b, c, d), 5 args, types DAT_002a3b30.
pub fn ee_sub_102d30<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    let sp = stack_push(m, 0xb0);
    let (s0, s1) = (sp + 0x40, sp + 0x30);
    for k in 0..5 {
        let t = m.r32(0x002a_3b30 + 4 * k);
        m.w32(sp + 0x10 + 4 * k, t);
    }
    m.w32(s1, u32::MAX);
    m.w32(sp, sp + 0x5c);
    m.call(PARSE, &[s0, 5, argv, sp + 0x10, s1, sp + 0x50, sp + 0x54, sp + 0x58, sp + 0x5c], &[]);
    m.call(DTOR, &[s0], &[]);
    m.call(SET_TAG, &[s0, 0x24], &[]);
    let a = m.r32(sp + 0x50);
    let b = m.r32(sp + 0x54);
    let c = m.r32(sp + 0x58);
    let d = m.r32(sp + 0x5c);
    m.call(0x0020_7cf8, &[obj, s1, a, b, c, d], &[]);
    m.call(ASSIGN, &[ret, s0], &[]);
    m.call(DTOR, &[s0], &[]);
    m.call(DTOR, &[s1], &[]);
    stack_pop(m, 0xb0);
    ret
}
