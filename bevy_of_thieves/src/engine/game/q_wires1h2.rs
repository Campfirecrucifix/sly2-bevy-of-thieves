//! WIRE-S1: 1:1 QMem ports of vm_state PersistentHost script-method natives (approved option b).
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem, V4};

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

/// copy `n` words of the static arg-kind table `src` to `dst`.
fn kinds<M: QMem + ?Sized>(m: &mut M, dst: u32, src: u32, n: u32) {
    for k in 0..n {
        let w = m.r32(src + 4 * k);
        m.w32(dst + 4 * k, w);
    }
}

/// tail shared by every thunk: FUN_001291a8(ret, &r); FUN_00129ec8(&r); return ret.
fn ret_ref<M: QMem + ?Sized>(m: &mut M, a0: u32, r: u32) -> u32 {
    m.call(0x0012_91a8, &[a0, r], &[]);
    m.call(0x0012_9ec8, &[r], &[]);
    a0
}

/// FUN_00129ec8(&r); FUN_0012a0b0(&r, 0x24): reset r to void.
fn void_ref<M: QMem + ?Sized>(m: &mut M, r: u32) {
    m.call(0x0012_9ec8, &[r], &[]);
    m.call(0x0012_a0b0, &[r, 0x24], &[]);
}

/// no-arg void thunk: unpack 0 args, r = void, f(obj).
fn thunk0_void<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a3: u32, f: u32) -> u32 {
    let sp = stack_push(m, 0x40);
    m.call(0x0020_1360, &[sp, 0, a3, 0], &[]);
    void_ref(m, sp);
    m.call(f, &[a1], &[]);
    let r = ret_ref(m, a0, sp);
    stack_pop(m, 0x40);
    r
}

/// one-arg (kind `k`) void thunk: f(obj, arg).
fn thunk1_void<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a3: u32, k: u32, f: u32) -> u32 {
    let sp = stack_push(m, 0x50);
    m.w32(sp + 0x10, k);
    m.call(0x0020_1360, &[sp, 1, a3, sp + 0x10, sp + 0x14], &[]);
    void_ref(m, sp);
    let x = m.r32(sp + 0x14);
    m.call(f, &[a1, x], &[]);
    let r = ret_ref(m, a0, sp);
    stack_pop(m, 0x50);
    r
}

/// one-arg (kind `k`) bool thunk: r = bool(f(obj, arg)).
fn thunk1_bool<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a3: u32, k: u32, f: u32) -> u32 {
    let sp = stack_push(m, 0x50);
    m.w32(sp + 0x10, k);
    m.call(0x0020_1360, &[sp, 1, a3, sp + 0x10, sp + 0x14], &[]);
    let x = m.r32(sp + 0x14);
    let v = m.call(f, &[a1, x], &[]).0;
    m.call(0x0012_9598, &[sp, (v != 0) as u32], &[]);
    let r = ret_ref(m, a0, sp);
    stack_pop(m, 0x50);
    r
}

/// two-int-arg void thunk (kinds at `tab`): f(obj, x, y).
fn thunk2_void<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a3: u32, tab: u32, f: u32) -> u32 {
    let sp = stack_push(m, 0x70);
    let s0 = sp + 0x10;
    kinds(m, sp, tab, 2);
    m.call(0x0020_1360, &[s0, 2, a3, sp, sp + 0x20, sp + 0x24], &[]);
    void_ref(m, s0);
    let x = m.r32(sp + 0x20);
    let y = m.r32(sp + 0x24);
    m.call(f, &[a1, x, y], &[]);
    let r = ret_ref(m, a0, s0);
    stack_pop(m, 0x70);
    r
}

/// FUN_00123fb8(a0..a3): FUN_00123eb0(a0, a1, a2, a3, 0x6c); return a0.
pub fn ee_sub_123fb8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32) -> u32 {
    m.call(0x0012_3eb0, &[a0, a1, a2, a3, 0x6c], &[]);
    a0
}

/// FUN_0010aa90(ret, obj, _, args): ret = bool(FUN_001424c0(obj)).
pub fn ee_sub_10aa90<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x40);
    m.call(0x0020_1360, &[sp, 0, a3, 0], &[]);
    let v = m.call(0x0014_24c0, &[a1], &[]).0;
    m.call(0x0012_9598, &[sp, (v != 0) as u32], &[]);
    let r = ret_ref(m, a0, sp);
    stack_pop(m, 0x40);
    r
}

/// FUN_00102250: void thunk -> FUN_001fee90(obj).
pub fn ee_sub_102250<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    thunk0_void(m, a0, a1, a3, 0x001f_ee90)
}
/// FUN_00105278: void thunk -> FUN_001490e0(obj).
pub fn ee_sub_105278<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    thunk0_void(m, a0, a1, a3, 0x0014_90e0)
}
/// FUN_00106a08: void thunk -> FUN_001dffe0(obj).
pub fn ee_sub_106a08<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    thunk0_void(m, a0, a1, a3, 0x001d_ffe0)
}
/// FUN_0010c0a8: void thunk -> FUN_002102a0(obj).
pub fn ee_sub_10c0a8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    thunk0_void(m, a0, a1, a3, 0x0021_02a0)
}
/// FUN_0010ea98: void thunk -> FUN_00162df8(obj).
pub fn ee_sub_10ea98<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    thunk0_void(m, a0, a1, a3, 0x0016_2df8)
}
/// FUN_0010f370: void thunk -> FUN_00164c58(obj).
pub fn ee_sub_10f370<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    thunk0_void(m, a0, a1, a3, 0x0016_4c58)
}

/// FUN_00103ec0: arg kind 0x2071 -> ret = bool(FUN_00171da8(obj, x)).
pub fn ee_sub_103ec0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    thunk1_bool(m, a0, a1, a3, 0x2071, 0x0017_1da8)
}
/// FUN_0010c7a0: arg kind 5 -> ret = bool(FUN_001c4728(obj, x)).
pub fn ee_sub_10c7a0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    thunk1_bool(m, a0, a1, a3, 5, 0x001c_4728)
}

/// FUN_0010d3e8: arg kind 0 -> FUN_001b9ca0(obj, x), void.
pub fn ee_sub_10d3e8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    thunk1_void(m, a0, a1, a3, 0, 0x001b_9ca0)
}
/// FUN_001036a8: arg kind 0x2002 -> FUN_00208860(obj, x), void.
pub fn ee_sub_1036a8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    thunk1_void(m, a0, a1, a3, 0x2002, 0x0020_8860)
}
/// FUN_00104b70: arg kind 6 -> FUN_001f0948(obj, x), void.
pub fn ee_sub_104b70<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    thunk1_void(m, a0, a1, a3, 6, 0x001f_0948)
}
/// FUN_0010abf8: arg kind 0x2030 -> FUN_001421a0(obj, x), void.
pub fn ee_sub_10abf8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    thunk1_void(m, a0, a1, a3, 0x2030, 0x0014_21a0)
}
/// FUN_0010c260: arg kind 0x1058 -> FUN_0020f030(obj, x), void.
pub fn ee_sub_10c260<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    thunk1_void(m, a0, a1, a3, 0x1058, 0x0020_f030)
}
/// FUN_0010d958: arg kind 0x2030 -> FUN_00224258(obj, x), void.
pub fn ee_sub_10d958<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    thunk1_void(m, a0, a1, a3, 0x2030, 0x0022_4258)
}
/// FUN_0010df80: arg kind 6 -> FUN_00211e98(obj, x), void.
pub fn ee_sub_10df80<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    thunk1_void(m, a0, a1, a3, 6, 0x0021_1e98)
}
/// FUN_0010e998: arg kind 0x104e -> FUN_00162630(obj, x), void.
pub fn ee_sub_10e998<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    thunk1_void(m, a0, a1, a3, 0x104e, 0x0016_2630)
}

/// FUN_0010f7b0: arg kind 2 (quad at sp) -> ret = FUN_00129718(FUN_00155fb0(obj, x)).
pub fn ee_sub_10f7b0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x70);
    let s0 = sp + 0x10;
    m.w32(sp + 0x20, 2);
    m.call(0x0020_1360, &[s0, 1, a3, sp + 0x20, sp], &[]);
    let x = m.r32(sp);
    let v = m.call(0x0015_5fb0, &[a1, x], &[]).0;
    m.call(0x0012_9718, &[s0, v], &[]);
    let r = ret_ref(m, a0, s0);
    stack_pop(m, 0x70);
    r
}

/// FUN_00104188: kinds @0x2a3b80 (2 args) -> ret = FUN_00129518(FUN_001541d8(obj, x, y)).
pub fn ee_sub_104188<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x70);
    let s0 = sp + 0x10;
    kinds(m, sp, 0x002a_3b80, 2);
    m.call(0x0020_1360, &[s0, 2, a3, sp, sp + 0x20, sp + 0x24], &[]);
    let x = m.r32(sp + 0x20);
    let y = m.r32(sp + 0x24);
    let v = m.call(0x0015_41d8, &[a1, x, y], &[]).0;
    m.call(0x0012_9518, &[s0, v], &[]);
    let r = ret_ref(m, a0, s0);
    stack_pop(m, 0x70);
    r
}

/// FUN_001012c0: kinds @0x2a3a80 (2 floats) -> FUN_001f8038(obj, x, y), void.
pub fn ee_sub_1012c0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x70);
    let s0 = sp + 0x10;
    kinds(m, sp, 0x002a_3a80, 2);
    m.call(0x0020_1360, &[s0, 2, a3, sp, sp + 0x20, sp + 0x24], &[]);
    void_ref(m, s0);
    let x = m.rf(sp + 0x20);
    let y = m.rf(sp + 0x24);
    m.call(0x001f_8038, &[a1], &[f4(x), f4(y)]);
    let r = ret_ref(m, a0, s0);
    stack_pop(m, 0x70);
    r
}

/// FUN_001052f0: kinds @0x2a3b98 -> FUN_00149778(obj, x, y), void.
pub fn ee_sub_1052f0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    thunk2_void(m, a0, a1, a3, 0x002a_3b98, 0x0014_9778)
}

/// FUN_0010b340: kinds @0x2a3ef8 (2 quads) -> FUN_0019eaa0(obj + 0xf50, x, y), void.
pub fn ee_sub_10b340<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x80);
    let s0 = sp + 0x30;
    kinds(m, sp, 0x002a_3ef8, 2);
    m.call(0x0020_1360, &[s0, 2, a3, sp, sp + 0x10, sp + 0x20], &[]);
    void_ref(m, s0);
    let x = m.r32(sp + 0x10);
    let y = m.r32(sp + 0x20);
    m.call(0x0019_eaa0, &[a1.wrapping_add(0xf50), x, y], &[]);
    let r = ret_ref(m, a0, s0);
    stack_pop(m, 0x80);
    r
}

/// FUN_0010d0b0: kinds @0x2a3c08 -> FUN_001bbd48(obj, x, y), void.
pub fn ee_sub_10d0b0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    thunk2_void(m, a0, a1, a3, 0x002a_3c08, 0x001b_bd48)
}

/// FUN_0010ef08: kinds @0x2a4000 -> FUN_00164858(obj, x, y), void.
pub fn ee_sub_10ef08<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    thunk2_void(m, a0, a1, a3, 0x002a_4000, 0x0016_4858)
}

/// FUN_0010ad08: kinds @0x2a3eb8 -> FUN_001424e0(obj, x, y, -1), void.
pub fn ee_sub_10ad08<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x70);
    let s0 = sp + 0x10;
    kinds(m, sp, 0x002a_3eb8, 2);
    m.call(0x0020_1360, &[s0, 2, a3, sp, sp + 0x20, sp + 0x24], &[]);
    void_ref(m, s0);
    let x = m.r32(sp + 0x20);
    let y = m.r32(sp + 0x24);
    m.call(0x0014_24e0, &[a1, x, y, u32::MAX], &[]);
    let r = ret_ref(m, a0, s0);
    stack_pop(m, 0x70);
    r
}

/// FUN_00103100: kinds @0x2a3b60 (3 args) -> FUN_00208058(obj, x, y, z), void.
pub fn ee_sub_103100<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x70);
    let s0 = sp + 0x10;
    kinds(m, sp, 0x002a_3b60, 3);
    m.call(0x0020_1360, &[s0, 3, a3, sp, sp + 0x20, sp + 0x24, sp + 0x28], &[]);
    void_ref(m, s0);
    let x = m.r32(sp + 0x20);
    let y = m.r32(sp + 0x24);
    let z = m.r32(sp + 0x28);
    m.call(0x0020_8058, &[a1, x, y, z], &[]);
    let r = ret_ref(m, a0, s0);
    stack_pop(m, 0x70);
    r
}

/// FUN_0010a790: kinds @0x2a3ea0 (quad, 2 floats) -> FUN_00141968(obj, v, f, g), void.
pub fn ee_sub_10a790<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x80);
    let s0 = sp + 0x20;
    kinds(m, sp, 0x002a_3ea0, 3);
    m.call(0x0020_1360, &[s0, 3, a3, sp, sp + 0x10, sp + 0x30, sp + 0x34], &[]);
    void_ref(m, s0);
    let v = m.r32(sp + 0x10);
    let f = m.rf(sp + 0x30);
    let g = m.rf(sp + 0x34);
    m.call(0x0014_1968, &[a1, v], &[f4(f), f4(g)]);
    let r = ret_ref(m, a0, s0);
    stack_pop(m, 0x80);
    r
}

/// FUN_0010e5d8: kinds @0x2a3f90 (2 args, quad) -> FUN_001f8a78(obj, x, y, v), void.
pub fn ee_sub_10e5d8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x80);
    let s0 = sp + 0x20;
    kinds(m, sp, 0x002a_3f90, 3);
    m.call(0x0020_1360, &[s0, 3, a3, sp, sp + 0x30, sp + 0x34, sp + 0x10], &[]);
    void_ref(m, s0);
    let x = m.r32(sp + 0x30);
    let y = m.r32(sp + 0x34);
    let v = m.r32(sp + 0x10);
    m.call(0x001f_8a78, &[a1, x, y, v], &[]);
    let r = ret_ref(m, a0, s0);
    stack_pop(m, 0x80);
    r
}

/// FUN_001d4b90(o, a, b, c, d): if o+0x4f8 < 8, new 0xb0 node (vt 0x2c94a0) via FUN_001ae4f8, init
/// FUN_001d4d30(node, o), +0x14/+0x18/+0x2c/+0x38 = a/b/c/d, append to o+0x4fc[] (count +0x4f8).
pub fn ee_sub_1d4b90<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, t0: u32) {
    if m.r32(a0 + 0x4f8) >= 8 {
        return;
    }
    let n = m.call(0x001a_e4f8, &[0xb0], &[]).0;
    m.w32(n, 0);
    m.w32(n + 0xc, 0x002c_94a0);
    m.w32(n + 4, 0);
    m.call(0x001d_4d30, &[n, a0], &[]);
    let c = m.r32(a0 + 0x4f8);
    m.w32(n + 0x14, a1);
    m.w32(n + 0x18, a2);
    m.w32(a0.wrapping_add(c << 2) + 0x4fc, n);
    m.w32(a0 + 0x4f8, c.wrapping_add(1));
    m.w32(n + 0x2c, a3);
    m.w32(n + 0x38, t0);
}

/// FUN_0010ec00: kinds @0x2a3fd8 (4 args) -> FUN_001626c0(obj, w, x, y, z), void.
pub fn ee_sub_10ec00<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x70);
    let s0 = sp + 0x10;
    kinds(m, sp, 0x002a_3fd8, 4);
    m.call(0x0020_1360, &[s0, 4, a3, sp, sp + 0x20, sp + 0x24, sp + 0x28, sp + 0x2c], &[]);
    void_ref(m, s0);
    let w = m.r32(sp + 0x20);
    let x = m.r32(sp + 0x24);
    let y = m.r32(sp + 0x28);
    let z = m.r32(sp + 0x2c);
    m.call(0x0016_26c0, &[a1, w, x, y, z], &[]);
    let r = ret_ref(m, a0, s0);
    stack_pop(m, 0x70);
    r
}

/// FUN_0010ecc8: kinds @0x2a3fe8 (5 args) -> FUN_00162728(obj, v, w, x, y, z), void.
pub fn ee_sub_10ecc8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0xa0);
    let s0 = sp + 0x30;
    m.w32(sp, sp + 0x50);
    kinds(m, sp + 0x10, 0x002a_3fe8, 5);
    m.call(
        0x0020_1360,
        &[s0, 5, a3, sp + 0x10, sp + 0x40, sp + 0x44, sp + 0x48, sp + 0x4c, sp + 0x50],
        &[],
    );
    void_ref(m, s0);
    let v = m.r32(sp + 0x40);
    let w = m.r32(sp + 0x44);
    let x = m.r32(sp + 0x48);
    let y = m.r32(sp + 0x4c);
    let z = m.r32(sp + 0x50);
    m.call(0x0016_2728, &[a1, v, w, x, y, z], &[]);
    let r = ret_ref(m, a0, s0);
    stack_pop(m, 0xa0);
    r
}

/// FUN_00102f38: kinds @0x2a3b48 (6 args: ref, int, 4 floats) -> FUN_00207e08(obj, &ref, n, f0..f3), void;
/// the ref local (+0x30, init -1) is released after.
pub fn ee_sub_102f38<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0xc0);
    let s0 = sp + 0x40;
    let s1 = sp + 0x30;
    kinds(m, sp + 0x10, 0x002a_3b48, 6);
    m.w32(sp + 8, sp + 0x60);
    m.w32(s1, u32::MAX);
    m.w32(sp, sp + 0x5c);
    m.call(
        0x0020_1360,
        &[s0, 6, a3, sp + 0x10, s1, sp + 0x50, sp + 0x54, sp + 0x58, sp + 0x5c, sp + 0x60],
        &[],
    );
    void_ref(m, s0);
    let n = m.r32(sp + 0x50);
    let f0 = m.rf(sp + 0x54);
    let f1 = m.rf(sp + 0x58);
    let f2 = m.rf(sp + 0x5c);
    let f3 = m.rf(sp + 0x60);
    m.call(0x0020_7e08, &[a1, s1, n], &[f4(f0), f4(f1), f4(f2), f4(f3)]);
    m.call(0x0012_91a8, &[a0, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_9ec8, &[s1], &[]);
    stack_pop(m, 0xc0);
    a0
}
