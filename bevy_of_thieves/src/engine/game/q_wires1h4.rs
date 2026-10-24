//! WIRE-S1: 1:1 QMem ports of vm_state PersistentHost script-method natives (approved option b).
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem, V4};

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

fn cpw<M: QMem + ?Sized>(m: &mut M, dst: u32, src: u32, n: u32) {
    for k in (0..n).step_by(4) {
        let w = m.r32(src + k);
        m.w32(dst + k, w);
    }
}

/// refcounted handle release at `slot`: --rc, on 0 FUN_00256d40(+0x10, 2), FUN_00129ec8(+8),
/// FUN_002550f8, FUN_0011cd98(0x4520a8, h)
fn release<M: QMem + ?Sized>(m: &mut M, slot: u32) {
    let h = m.r32(slot);
    if h == 0 {
        return;
    }
    let v = m.r32(h).wrapping_sub(1);
    m.w32(h, v);
    if v == 0 {
        m.call(0x0025_6d40, &[h + 0x10, 2], &[]);
        m.call(0x0012_9ec8, &[h + 8], &[]);
        m.call(0x0025_50f8, &[h], &[]);
        m.call(0x0011_cd98, &[0x0045_20a8, h], &[]);
    }
}

/// FUN_00123ad0(a0): FUN_001294c0(a0, DAT_003e0f9c); returns a0
pub fn ee_sub_123ad0<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let a1 = m.r32(0x003e_0f9c);
    m.call(0x0012_94c0, &[a0, a1], &[]);
    a0
}

/// FUN_0010adb8: no-arg method, result = FUN_00142530(obj) != 0 as bool
pub fn ee_sub_10adb8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x40);
    m.call(0x0020_1360, &[sp, 0, a3, 0], &[]);
    let r = m.call(0x0014_2530, &[a1], &[]).0;
    m.call(0x0012_9598, &[sp, (r != 0) as u32], &[]);
    m.call(0x0012_91a8, &[a0, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x40);
    a0
}

fn void0<M: QMem + ?Sized>(m: &mut M, f: u32, a0: u32, a1: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x40);
    m.call(0x0020_1360, &[sp, 0, a3, 0], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    m.call(f, &[a1], &[]);
    m.call(0x0012_91a8, &[a0, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x40);
    a0
}

/// FUN_00102340: no-arg void method -> FUN_001feeb0(obj)
pub fn ee_sub_102340<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    void0(m, 0x001f_eeb0, a0, a1, a3)
}

/// FUN_00105628: no-arg void method -> FUN_00149758(obj)
pub fn ee_sub_105628<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    void0(m, 0x0014_9758, a0, a1, a3)
}

/// FUN_0010b140: no-arg void method -> FUN_00143380(obj)
pub fn ee_sub_10b140<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    void0(m, 0x0014_3380, a0, a1, a3)
}

/// FUN_0010d468: no-arg void method -> FUN_001b9d00(obj)
pub fn ee_sub_10d468<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    void0(m, 0x001b_9d00, a0, a1, a3)
}

/// FUN_0010eb88: no-arg void method -> FUN_00162940(obj)
pub fn ee_sub_10eb88<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    void0(m, 0x0016_2940, a0, a1, a3)
}

/// FUN_0010f9c0: no-arg void method -> FUN_00153800(obj)
pub fn ee_sub_10f9c0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    void0(m, 0x0015_3800, a0, a1, a3)
}

/// FUN_001042d0: 1-arg (fmt 0x2002) method, int result = FUN_00154158(obj, x) via FUN_00129518
pub fn ee_sub_1042d0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x50);
    m.w32(sp + 0x10, 0x2002);
    m.call(0x0020_1360, &[sp, 1, a3, sp + 0x10, sp + 0x14], &[]);
    let x = m.r32(sp + 0x14);
    let r = m.call(0x0015_4158, &[a1, x], &[]).0;
    m.call(0x0012_9518, &[sp, r], &[]);
    m.call(0x0012_91a8, &[a0, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x50);
    a0
}

/// FUN_0010d660: 1-arg (fmt 0x2002) method, bool result = FUN_001841f0(obj, x) != 0
pub fn ee_sub_10d660<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x50);
    m.w32(sp + 0x10, 0x2002);
    m.call(0x0020_1360, &[sp, 1, a3, sp + 0x10, sp + 0x14], &[]);
    let x = m.r32(sp + 0x14);
    let r = m.call(0x0018_41f0, &[a1, x], &[]).0;
    m.call(0x0012_9598, &[sp, (r != 0) as u32], &[]);
    m.call(0x0012_91a8, &[a0, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x50);
    a0
}

/// shared shape of the 1-arg void methods: unpack one word (fmt `fmt`) to sp+0x14, then `f(obj, w)`
/// (or `f(obj)` with the word as f12 when `float`)
fn void1<M: QMem + ?Sized>(m: &mut M, fmt: u32, f: u32, float: bool, a0: u32, a1: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x50);
    m.w32(sp + 0x10, fmt);
    m.call(0x0020_1360, &[sp, 1, a3, sp + 0x10, sp + 0x14], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    if float {
        let x = m.rf(sp + 0x14);
        m.call(f, &[a1], &[f4(x)]);
    } else {
        let x = m.r32(sp + 0x14);
        m.call(f, &[a1, x], &[]);
    }
    m.call(0x0012_91a8, &[a0, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x50);
    a0
}

/// FUN_00101190: 1-float method -> FUN_001f7c68(obj, f)
pub fn ee_sub_101190<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    void1(m, 1, 0x001f_7c68, true, a0, a1, a3)
}

/// FUN_00103ca0: 1-arg (fmt 6) method -> FUN_001f17b0(obj, x)
pub fn ee_sub_103ca0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    void1(m, 6, 0x001f_17b0, false, a0, a1, a3)
}

/// FUN_00104df8: 1-arg (fmt 7) method -> FUN_001b2048(obj, x)
pub fn ee_sub_104df8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    void1(m, 7, 0x001b_2048, false, a0, a1, a3)
}

/// FUN_0010b0b8: 1-arg (fmt 6) method -> FUN_001432e0(obj, x)
pub fn ee_sub_10b0b8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    void1(m, 6, 0x0014_32e0, false, a0, a1, a3)
}

/// FUN_0010d028: 1-arg (fmt 6) method -> FUN_001f6b48(obj, x)
pub fn ee_sub_10d028<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    void1(m, 6, 0x001f_6b48, false, a0, a1, a3)
}

/// FUN_0010da68: 1-arg (fmt 0x2071) method -> FUN_001ba980(obj, x)
pub fn ee_sub_10da68<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    void1(m, 0x2071, 0x001b_a980, false, a0, a1, a3)
}

/// FUN_0010e1c0: 1-arg (fmt 6) method -> FUN_0013d328(obj, x)
pub fn ee_sub_10e1c0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    void1(m, 6, 0x0013_d328, false, a0, a1, a3)
}

/// FUN_0010fa38: 1-arg (fmt 6) method -> FUN_00171c98(obj, x)
pub fn ee_sub_10fa38<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    void1(m, 6, 0x0017_1c98, false, a0, a1, a3)
}

/// FUN_0010b6d0: 1-arg (fmt 2, quadword) method -> FUN_0018df58(obj, q)
pub fn ee_sub_10b6d0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x70);
    let s0 = sp + 0x10;
    m.w32(sp + 0x20, 2);
    m.call(0x0020_1360, &[s0, 1, a3, sp + 0x20, sp], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_a0b0, &[s0, 0x24], &[]);
    let q = m.r32(sp);
    m.call(0x0018_df58, &[a1, q], &[]);
    m.call(0x0012_91a8, &[a0, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0x70);
    a0
}

/// FUN_0010de58: 1-arg (fmt 0x8010, value object at sp init -1) method -> FUN_00212828(obj, &v)
pub fn ee_sub_10de58<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x70);
    let s0 = sp + 0x10;
    m.w32(sp + 0x20, 0x8010);
    m.w32(sp, u32::MAX);
    m.call(0x0020_1360, &[s0, 1, a3, sp + 0x20, sp], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_a0b0, &[s0, 0x24], &[]);
    m.call(0x0021_2828, &[a1, sp], &[]);
    m.call(0x0012_91a8, &[a0, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x70);
    a0
}


/// shared shape of the 2-arg void methods: 8-byte fmt from `fmt`, words at sp+0x20/+0x24; `f == 0` is
/// the FUN_0010dd08 shape FUN_00212e40(obj, y) with x as f12
fn void2<M: QMem + ?Sized>(m: &mut M, fmt: u32, f: u32, a0: u32, a1: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x70);
    let s0 = sp + 0x10;
    cpw(m, sp, fmt, 8);
    m.call(0x0020_1360, &[s0, 2, a3, sp, sp + 0x20, sp + 0x24], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_a0b0, &[s0, 0x24], &[]);
    let x = m.r32(sp + 0x20);
    let y = m.r32(sp + 0x24);
    if f != 0 {
        m.call(f, &[a1, x, y], &[]);
    } else {
        m.call(0x0021_2e40, &[a1, y], &[f4(f32::from_bits(x))]);
    }
    m.call(0x0012_91a8, &[a0, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0x70);
    a0
}

/// FUN_00102858: 2-arg method (fmt DAT_002a3b18) -> FUN_00203cb8(obj, x, y)
pub fn ee_sub_102858<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    void2(m, 0x002a_3b18, 0x0020_3cb8, a0, a1, a3)
}

/// FUN_00105508: 2-arg method (fmt DAT_002a3c08) -> FUN_00149828(obj, x, y)
pub fn ee_sub_105508<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    void2(m, 0x002a_3c08, 0x0014_9828, a0, a1, a3)
}

/// FUN_0010b850: 2-arg method (fmt DAT_002a3c08) -> FUN_002284e0(obj, x, y)
pub fn ee_sub_10b850<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    void2(m, 0x002a_3c08, 0x0022_84e0, a0, a1, a3)
}

/// FUN_0010dd08: 2-arg method (fmt DAT_002a3f70) -> FUN_00212e40(obj, y, f=x)
pub fn ee_sub_10dd08<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    void2(m, 0x002a_3f70, 0, a0, a1, a3)
}

/// FUN_0010fb48: 2-arg method (fmt DAT_002a4070) -> FUN_0014a178(obj, x, y)
pub fn ee_sub_10fb48<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    void2(m, 0x002a_4070, 0x0014_a178, a0, a1, a3)
}

/// FUN_0010bff8: 3-arg method (fmt DAT_002a3f20), value result FUN_00129a10(tmp, FUN_00156b98(obj, x, q, z))
pub fn ee_sub_10bff8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x80);
    let s0 = sp + 0x20;
    cpw(m, sp, 0x002a_3f20, 12);
    m.call(0x0020_1360, &[s0, 3, a3, sp, sp + 0x30, sp + 0x10, sp + 0x34], &[]);
    let x = m.r32(sp + 0x30);
    let q = m.r32(sp + 0x10);
    let z = m.r32(sp + 0x34);
    let r = m.call(0x0015_6b98, &[a1, x, q, z], &[]).0;
    m.call(0x0012_9a10, &[s0, r], &[]);
    m.call(0x0012_91a8, &[a0, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0x80);
    a0
}

fn void3<M: QMem + ?Sized>(m: &mut M, fmt: u32, f: u32, a0: u32, a1: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x70);
    let s0 = sp + 0x10;
    cpw(m, sp, fmt, 12);
    m.call(0x0020_1360, &[s0, 3, a3, sp, sp + 0x20, sp + 0x24, sp + 0x28], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_a0b0, &[s0, 0x24], &[]);
    let x = m.r32(sp + 0x20);
    let y = m.r32(sp + 0x24);
    let z = m.r32(sp + 0x28);
    m.call(f, &[a1, x, y, z], &[]);
    m.call(0x0012_91a8, &[a0, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0x70);
    a0
}

/// FUN_001033e0: 3-arg method (fmt DAT_002a3b60) -> FUN_00208018(obj, x, y, z)
pub fn ee_sub_1033e0<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    void3(m, 0x002a_3b60, 0x0020_8018, a0, a1, a3)
}

/// FUN_0010bd10: 3-arg method (fmt DAT_002a3b88) -> FUN_001c64f8(obj, x, y, z)
pub fn ee_sub_10bd10<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    void3(m, 0x002a_3b88, 0x001c_64f8, a0, a1, a3)
}

/// FUN_0010f590: 3-arg method (fmt DAT_002a4038) -> FUN_001d7748(obj, x, y, z)
pub fn ee_sub_10f590<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    void3(m, 0x002a_4038, 0x001d_7748, a0, a1, a3)
}

/// FUN_0010e690: 3-arg method (fmt DAT_002a3fa0, third arg by pointer) -> FUN_001f8ad8(obj, x, y, &v)
pub fn ee_sub_10e690<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0xc0);
    let s0 = sp + 0x50;
    let s1 = sp + 0x10;
    cpw(m, sp, 0x002a_3fa0, 12);
    m.call(0x0020_1360, &[s0, 3, a3, sp, sp + 0x60, sp + 0x64, s1], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_a0b0, &[s0, 0x24], &[]);
    let x = m.r32(sp + 0x60);
    let y = m.r32(sp + 0x64);
    m.call(0x001f_8ad8, &[a1, x, y, s1], &[]);
    m.call(0x0012_91a8, &[a0, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0xc0);
    a0
}

/// FUN_001101f8: 4-arg method (fmt DAT_002a3bf8) -> FUN_001816e8(obj, x, y, z, w)
pub fn ee_sub_1101f8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x70);
    let s0 = sp + 0x10;
    cpw(m, sp, 0x002a_3bf8, 16);
    m.call(0x0020_1360, &[s0, 4, a3, sp, sp + 0x20, sp + 0x24, sp + 0x28, sp + 0x2c], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_a0b0, &[s0, 0x24], &[]);
    let x = m.r32(sp + 0x20);
    let y = m.r32(sp + 0x24);
    let z = m.r32(sp + 0x28);
    let w = m.r32(sp + 0x2c);
    m.call(0x0018_16e8, &[a1, x, y, z, w], &[]);
    m.call(0x0012_91a8, &[a0, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0x70);
    a0
}

/// FUN_0010f030: 4-arg method (fmt DAT_002a4008, 2nd arg value object init -1) ->
/// FUN_00163838(obj, x, &v, z, w)
pub fn ee_sub_10f030<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x90);
    let s0 = sp + 0x20;
    let s1 = sp + 0x10;
    cpw(m, sp, 0x002a_4008, 16);
    m.w32(s1, u32::MAX);
    m.call(0x0020_1360, &[s0, 4, a3, sp, sp + 0x30, s1, sp + 0x34, sp + 0x38], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_a0b0, &[s0, 0x24], &[]);
    let x = m.r32(sp + 0x30);
    let z = m.r32(sp + 0x34);
    let w = m.r32(sp + 0x38);
    m.call(0x0016_3838, &[a1, x, s1, z, w], &[]);
    m.call(0x0012_91a8, &[a0, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_9ec8, &[s1], &[]);
    stack_pop(m, 0x90);
    a0
}

/// FUN_001249a8(out, kind, _, args): overlap query. Empty args -> FUN_00129658 copy. Else walks the +4
/// list (node +0xc obj, +0x10 next), each obj's +0xe4 table (+0xe0 count, 0xb0 stride, skip bit 50 of
/// +0x10) via FUN_0014c6a0(e, a[1], a[2], 1, &hit); kind 0x7b collects refcounted {obj, p0, p1}
/// records into a head/tail list, other kinds return the first hit record.
pub fn ee_sub_1249a8<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, _a2: u32, a3: u32) -> u32 {
    let sp = stack_push(m, 0x180);
    let out = a0;
    let fp = a1;
    let s5 = a3;
    let v1 = m.r32(s5);
    m.w32(sp, u32::MAX);
    if v1 == 0 {
        m.call(0x0012_9658, &[sp, s5], &[]);
    } else {
        m.call(0x0025_6c38, &[sp + 0x10, 0], &[]);
        m.call(0x0025_6c38, &[sp + 0x20, 0], &[]);
        let mut s4 = m.r32(s5 + 4);
        let s7 = s5 + 0x10;
        let s6 = s5 + 8;
        while s4 != 0 {
            let s1 = m.r32(s4 + 0xc);
            if s1 != 0 {
                let mut s2: i32 = 0;
                while s2 < m.r32(s1 + 0xe0) as i32 {
                    let tab = m.r32(s1 + 0xe4);
                    let e = tab.wrapping_add((s2 as u32).wrapping_mul(0xb0));
                    s2 += 1;
                    if m.r64(e + 0x10) & (0x8000u64 << 35) != 0 {
                        continue;
                    }
                    let p1 = m.r32(s6 + 4);
                    let p2 = m.r32(s7 + 4);
                    let q1 = m.r32(p1);
                    let q2 = m.r32(p2);
                    let hit = m.call(0x0014_c6a0, &[e, q1, q2, 1, sp + 0x30], &[]).0;
                    if hit as i32 <= 0 {
                        continue;
                    }
                    let x = m.call(0x0025_6b48, &[], &[]).0;
                    m.call(0x0025_6c38, &[sp + 0x90, x], &[]);
                    let x = m.call(0x0025_6b48, &[], &[]).0;
                    m.call(0x0025_6c38, &[sp + 0xa0, x], &[]);
                    let h90 = m.r32(sp + 0x90);
                    m.call(0x0012_9a10, &[h90 + 8, s1], &[]);
                    let h90 = m.r32(sp + 0x90);
                    let ha0 = m.r32(sp + 0xa0);
                    m.call(0x0025_6c58, &[h90 + 0x10, ha0], &[]);
                    if fp != 0x7b {
                        let ha0 = m.r32(sp + 0xa0);
                        m.call(0x0012_96a8, &[ha0 + 8, sp + 0x40], &[]);
                        let h90 = m.r32(sp + 0x90);
                        m.call(0x0012_9970, &[sp, h90], &[]);
                        m.call(0x0012_91a8, &[out, sp], &[]);
                        release(m, sp + 0xa0);
                        release(m, sp + 0x90);
                        release(m, sp + 0x20);
                        release(m, sp + 0x10);
                        m.call(0x0012_9ec8, &[sp], &[]);
                        stack_pop(m, 0x180);
                        return out;
                    }
                    let x = m.call(0x0025_6b48, &[], &[]).0;
                    m.call(0x0025_6c38, &[sp + 0xb0, x], &[]);
                    let x = m.call(0x0025_6b48, &[], &[]).0;
                    m.call(0x0025_6c38, &[sp + 0xc0, x], &[]);
                    let ha0 = m.r32(sp + 0xa0);
                    m.call(0x0012_96a8, &[ha0 + 8, sp + 0x30], &[]);
                    let ha0 = m.r32(sp + 0xa0);
                    let hb0 = m.r32(sp + 0xb0);
                    m.call(0x0025_6c58, &[ha0 + 0x10, hb0], &[]);
                    let hb0 = m.r32(sp + 0xb0);
                    m.call(0x0012_96a8, &[hb0 + 8, sp + 0x40], &[]);
                    let hc0 = m.r32(sp + 0xc0);
                    let h90 = m.r32(sp + 0x90);
                    m.call(0x0012_9970, &[hc0 + 8, h90], &[]);
                    if m.r32(sp + 0x10) != 0 {
                        let h20 = m.r32(sp + 0x20);
                        let hc0 = m.r32(sp + 0xc0);
                        m.call(0x0025_6c58, &[h20 + 0x10, hc0], &[]);
                    } else {
                        let hc0 = m.r32(sp + 0xc0);
                        m.call(0x0025_6c58, &[sp + 0x10, hc0], &[]);
                    }
                    let hc0 = m.r32(sp + 0xc0);
                    m.call(0x0025_6c58, &[sp + 0x20, hc0], &[]);
                    release(m, sp + 0xc0);
                    release(m, sp + 0xb0);
                    release(m, sp + 0xa0);
                    release(m, sp + 0x90);
                }
            }
            s4 = m.r32(s4 + 0x10);
        }
        let h10 = m.r32(sp + 0x10);
        if h10 == 0 {
            m.w32(sp, 0);
        } else {
            m.call(0x0012_9970, &[sp, h10], &[]);
        }
        release(m, sp + 0x20);
        release(m, sp + 0x10);
    }
    m.call(0x0012_91a8, &[out, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x180);
    out
}
