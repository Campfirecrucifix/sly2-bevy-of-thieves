//! WIRE-S1: 1:1 QMem ports of vm_state PersistentHost script-method natives (approved option b).
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem, V4};

fn f4(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}

/// FUN_00123b30: bif 102 splice: FUN_001294c0(ret, DAT_003e0fa4); returns ret.
pub fn ee_sub_123b30<M: QMem + ?Sized>(m: &mut M, a0: u32) -> u32 {
    let a1 = m.r32(0x003e_0fa4);
    m.call(0x0012_94c0, &[a0, a1], &[]);
    a0
}

/// FUN_00100c08: script method, no args: unpack(0), FUN_001f8238(obj); ret = nil.
pub fn ee_sub_100c08<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, args: u32) -> u32 {
    let sp = stack_push(m, 0x40);
    m.call(0x0020_1360, &[sp, 0, args, 0], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    m.call(0x001f_8238, &[obj], &[]);
    m.call(0x0012_91a8, &[ret, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x40);
    ret
}

/// FUN_001027e0: script method, no args: unpack(0), FUN_00221940(obj); ret = nil.
pub fn ee_sub_1027e0<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, args: u32) -> u32 {
    let sp = stack_push(m, 0x40);
    m.call(0x0020_1360, &[sp, 0, args, 0], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    m.call(0x0022_1940, &[obj], &[]);
    m.call(0x0012_91a8, &[ret, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x40);
    ret
}

/// FUN_00105af0: script method, no args: unpack(0), FUN_001594d8(obj); ret = nil.
pub fn ee_sub_105af0<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, args: u32) -> u32 {
    let sp = stack_push(m, 0x40);
    m.call(0x0020_1360, &[sp, 0, args, 0], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    m.call(0x0015_94d8, &[obj], &[]);
    m.call(0x0012_91a8, &[ret, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x40);
    ret
}

/// FUN_0010b760: script method, no args: unpack(0), FUN_00228170(obj); ret = nil.
pub fn ee_sub_10b760<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, args: u32) -> u32 {
    let sp = stack_push(m, 0x40);
    m.call(0x0020_1360, &[sp, 0, args, 0], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    m.call(0x0022_8170, &[obj], &[]);
    m.call(0x0012_91a8, &[ret, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x40);
    ret
}

/// FUN_0010e0c0: script method, no args: unpack(0), FUN_0013c440(obj); ret = nil.
pub fn ee_sub_10e0c0<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, args: u32) -> u32 {
    let sp = stack_push(m, 0x40);
    m.call(0x0020_1360, &[sp, 0, args, 0], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    m.call(0x0013_c440, &[obj], &[]);
    m.call(0x0012_91a8, &[ret, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x40);
    ret
}

/// FUN_0010f280: script method, no args: unpack(0), FUN_00164b38(obj); ret = nil.
pub fn ee_sub_10f280<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, args: u32) -> u32 {
    let sp = stack_push(m, 0x40);
    m.call(0x0020_1360, &[sp, 0, args, 0], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    m.call(0x0016_4b38, &[obj], &[]);
    m.call(0x0012_91a8, &[ret, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x40);
    ret
}

/// FUN_00110338: script method, no args: unpack(0), FUN_00181780(obj); ret = nil.
pub fn ee_sub_110338<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, args: u32) -> u32 {
    let sp = stack_push(m, 0x40);
    m.call(0x0020_1360, &[sp, 0, args, 0], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    m.call(0x0018_1780, &[obj], &[]);
    m.call(0x0012_91a8, &[ret, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x40);
    ret
}

/// FUN_0010b490: script method (int 0x1049): ret = FUN_00129518(FUN_00141e78(obj, x)).
pub fn ee_sub_10b490<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, args: u32) -> u32 {
    let sp = stack_push(m, 0x50);
    m.w32(sp + 0x10, 0x1049);
    m.call(0x0020_1360, &[sp, 1, args, sp + 0x10, sp + 0x14], &[]);
    let a1 = m.r32(sp + 0x14);
    let v = m.call(0x0014_1e78, &[obj, a1], &[]).0;
    m.call(0x0012_9518, &[sp, v], &[]);
    m.call(0x0012_91a8, &[ret, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x50);
    ret
}

/// FUN_00101d98: script method, 1 arg (type 0x0): FUN_00230cf0(obj, x); ret = nil.
pub fn ee_sub_101d98<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, args: u32) -> u32 {
    let sp = stack_push(m, 0x50);
    m.w32(sp + 0x10, 0x0);
    m.call(0x0020_1360, &[sp, 1, args, sp + 0x10, sp + 0x14], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    let a1 = m.r32(sp + 0x14);
    m.call(0x0023_0cf0, &[obj, a1], &[]);
    m.call(0x0012_91a8, &[ret, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x50);
    ret
}

/// FUN_001024b0: script method, 1 arg (type 0x13): FUN_001fe310(obj, x); ret = nil.
pub fn ee_sub_1024b0<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, args: u32) -> u32 {
    let sp = stack_push(m, 0x50);
    m.w32(sp + 0x10, 0x13);
    m.call(0x0020_1360, &[sp, 1, args, sp + 0x10, sp + 0x14], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    let a1 = m.r32(sp + 0x14);
    m.call(0x001f_e310, &[obj, a1], &[]);
    m.call(0x0012_91a8, &[ret, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x50);
    ret
}

/// FUN_00103e38: script method, 1 arg (type 0x2071): FUN_00171d58(obj, x); ret = nil.
pub fn ee_sub_103e38<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, args: u32) -> u32 {
    let sp = stack_push(m, 0x50);
    m.w32(sp + 0x10, 0x2071);
    m.call(0x0020_1360, &[sp, 1, args, sp + 0x10, sp + 0x14], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    let a1 = m.r32(sp + 0x14);
    m.call(0x0017_1d58, &[obj, a1], &[]);
    m.call(0x0012_91a8, &[ret, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x50);
    ret
}

/// FUN_0010a848: script method, 1 arg (type 0x2002): FUN_00142140(obj, x); ret = nil.
pub fn ee_sub_10a848<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, args: u32) -> u32 {
    let sp = stack_push(m, 0x50);
    m.w32(sp + 0x10, 0x2002);
    m.call(0x0020_1360, &[sp, 1, args, sp + 0x10, sp + 0x14], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    let a1 = m.r32(sp + 0x14);
    m.call(0x0014_2140, &[obj, a1], &[]);
    m.call(0x0012_91a8, &[ret, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x50);
    ret
}

/// FUN_0010bee8: script method, 1 arg (type 0x6): FUN_001c7a10(obj, x); ret = nil.
pub fn ee_sub_10bee8<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, args: u32) -> u32 {
    let sp = stack_push(m, 0x50);
    m.w32(sp + 0x10, 0x6);
    m.call(0x0020_1360, &[sp, 1, args, sp + 0x10, sp + 0x14], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    let a1 = m.r32(sp + 0x14);
    m.call(0x001c_7a10, &[obj, a1], &[]);
    m.call(0x0012_91a8, &[ret, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x50);
    ret
}

/// FUN_0010d260: script method, 1 arg (type 0x2024): FUN_001b7000(obj, x); ret = nil.
pub fn ee_sub_10d260<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, args: u32) -> u32 {
    let sp = stack_push(m, 0x50);
    m.w32(sp + 0x10, 0x2024);
    m.call(0x0020_1360, &[sp, 1, args, sp + 0x10, sp + 0x14], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    let a1 = m.r32(sp + 0x14);
    m.call(0x001b_7000, &[obj, a1], &[]);
    m.call(0x0012_91a8, &[ret, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x50);
    ret
}

/// FUN_0010dc80: script method, 1 arg (type 0x6): FUN_00213928(obj, x); ret = nil.
pub fn ee_sub_10dc80<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, args: u32) -> u32 {
    let sp = stack_push(m, 0x50);
    m.w32(sp + 0x10, 0x6);
    m.call(0x0020_1360, &[sp, 1, args, sp + 0x10, sp + 0x14], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    let a1 = m.r32(sp + 0x14);
    m.call(0x0021_3928, &[obj, a1], &[]);
    m.call(0x0012_91a8, &[ret, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x50);
    ret
}

/// FUN_0010e420: script method, 1 arg (type 0x2002): FUN_001cb4b8(obj, x); ret = nil.
pub fn ee_sub_10e420<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, args: u32) -> u32 {
    let sp = stack_push(m, 0x50);
    m.w32(sp + 0x10, 0x2002);
    m.call(0x0020_1360, &[sp, 1, args, sp + 0x10, sp + 0x14], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    m.call(0x0012_a0b0, &[sp, 0x24], &[]);
    let a1 = m.r32(sp + 0x14);
    m.call(0x001c_b4b8, &[obj, a1], &[]);
    m.call(0x0012_91a8, &[ret, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x50);
    ret
}

/// FUN_0010bf70: script method (type 2 vector): ret = FUN_00129a10(FUN_001c4af8(obj, v)).
pub fn ee_sub_10bf70<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, args: u32) -> u32 {
    let sp = stack_push(m, 0x70);
    let s0 = sp + 0x10;
    m.w32(sp + 0x20, 2);
    m.call(0x0020_1360, &[s0, 1, args, sp + 0x20, sp], &[]);
    // a1 = the 128-bit quad at sp (a V4 arg)
    let q = m.rv(sp);
    let v = m.call(0x001c_4af8, &[obj], &[q]).0;
    m.call(0x0012_9a10, &[s0, v], &[]);
    m.call(0x0012_91a8, &[ret, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0x70);
    ret
}

/// FUN_00124150: bif 112 splice: FUN_0015e958(GS, a3?1:5, +4, +0xc, owner +0x3c or GS+0x3f0c) -> FUN_00129a10 into ret.
pub fn ee_sub_124150<M: QMem + ?Sized>(m: &mut M, a0: u32, _a1: u32, a2: u32) -> u32 {
    let sp = stack_push(m, 0x30);
    let a3 = m.r32(a2 + 0xc);
    m.w32(sp, 0xffff_ffff);
    let mut t0 = if m.r32(a2 + 0x10) != 0 {
        let p = m.r32(a2 + 0x14);
        m.r32(p + 0xc)
    } else {
        a3
    };
    let a1 = if a3 != 0 { 1 } else { 5 };
    let a6 = m.r32(a2 + 4);
    t0 = if t0 != 0 { m.r32(t0 + 0x3c) } else { m.r32(0x003d_d490 + 0x3f0c) };
    let v = m.call(0x0015_e958, &[0x003d_d490, a1, a6, a3, t0], &[]).0;
    m.call(0x0012_9a10, &[sp, v], &[]);
    m.call(0x0012_91a8, &[a0, sp], &[]);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x30);
    a0
}

/// FUN_00100fe0: script method, 2 args (types at DAT_002a3a78): FUN_001f7918(obj, ...); ret = nil.
pub fn ee_sub_100fe0<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, args: u32) -> u32 {
    let sp = stack_push(m, 0x70);
    let s0 = sp + 0x10;
    let w = m.r32(0x002a_3a78);
    m.w32(sp + 0x0, w);
    let w = m.r32(0x002a_3a7c);
    m.w32(sp + 0x4, w);
    m.call(0x0020_1360, &[s0, 2, args, sp, sp + 0x20, sp + 0x24], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_a0b0, &[s0, 0x24], &[]);
    let x0 = m.r32(sp + 0x20);
    let x1 = m.r32(sp + 0x24);
    m.call(0x001f_7918, &[obj, x0, x1], &[]);
    m.call(0x0012_91a8, &[ret, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0x70);
    ret
}

/// FUN_0010a9e8: script method, 2 args (types at DAT_002a3eb0): FUN_00142278(obj, ...); ret = nil.
pub fn ee_sub_10a9e8<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, args: u32) -> u32 {
    let sp = stack_push(m, 0x70);
    let s0 = sp + 0x10;
    let w = m.r32(0x002a_3eb0);
    m.w32(sp + 0x0, w);
    let w = m.r32(0x002a_3eb4);
    m.w32(sp + 0x4, w);
    m.call(0x0020_1360, &[s0, 2, args, sp, sp + 0x20, sp + 0x24], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_a0b0, &[s0, 0x24], &[]);
    let x0 = m.r32(sp + 0x20);
    let x1 = m.r32(sp + 0x24);
    m.call(0x0014_2278, &[obj, x0, x1], &[]);
    m.call(0x0012_91a8, &[ret, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0x70);
    ret
}

/// FUN_0010bbe0: script method, 2 args (types at DAT_002a3c70): FUN_00228958(obj, ...); ret = nil.
pub fn ee_sub_10bbe0<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, args: u32) -> u32 {
    let sp = stack_push(m, 0x70);
    let s0 = sp + 0x10;
    let w = m.r32(0x002a_3c70);
    m.w32(sp + 0x0, w);
    let w = m.r32(0x002a_3c74);
    m.w32(sp + 0x4, w);
    m.call(0x0020_1360, &[s0, 2, args, sp, sp + 0x20, sp + 0x24], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_a0b0, &[s0, 0x24], &[]);
    let x0 = m.r32(sp + 0x20);
    let x1 = m.r32(sp + 0x24);
    m.call(0x0022_8958, &[obj, x0, x1], &[]);
    m.call(0x0012_91a8, &[ret, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0x70);
    ret
}

/// FUN_0010e4a8: script method, 2 args (types at DAT_002a3f88): FUN_00147c20(obj, ...); ret = nil.
pub fn ee_sub_10e4a8<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, args: u32) -> u32 {
    let sp = stack_push(m, 0x70);
    let s0 = sp + 0x10;
    let w = m.r32(0x002a_3f88);
    m.w32(sp + 0x0, w);
    let w = m.r32(0x002a_3f8c);
    m.w32(sp + 0x4, w);
    m.call(0x0020_1360, &[s0, 2, args, sp, sp + 0x20, sp + 0x24], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_a0b0, &[s0, 0x24], &[]);
    let x0 = m.r32(sp + 0x20);
    let x1 = m.r32(sp + 0x24);
    m.call(0x0014_7c20, &[obj, x0, x1], &[]);
    m.call(0x0012_91a8, &[ret, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0x70);
    ret
}

/// FUN_00110838: script method, 2 args (types at DAT_002a40e0): FUN_001d4d18(obj, ...); ret = nil.
pub fn ee_sub_110838<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, args: u32) -> u32 {
    let sp = stack_push(m, 0x70);
    let s0 = sp + 0x10;
    let w = m.r32(0x002a_40e0);
    m.w32(sp + 0x0, w);
    let w = m.r32(0x002a_40e4);
    m.w32(sp + 0x4, w);
    m.call(0x0020_1360, &[s0, 2, args, sp, sp + 0x20, sp + 0x24], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_a0b0, &[s0, 0x24], &[]);
    let x0 = m.r32(sp + 0x20);
    let x1 = m.r32(sp + 0x24);
    m.call(0x001d_4d18, &[obj, x0, x1], &[]);
    m.call(0x0012_91a8, &[ret, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0x70);
    ret
}

/// FUN_00104060: script method, 2 args (types at DAT_002a3b80): FUN_00154038(obj+0xf64 ptr, ...); ret = nil.
pub fn ee_sub_104060<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, args: u32) -> u32 {
    let sp = stack_push(m, 0x70);
    let s0 = sp + 0x10;
    let w = m.r32(0x002a_3b80);
    m.w32(sp + 0x0, w);
    let w = m.r32(0x002a_3b84);
    m.w32(sp + 0x4, w);
    m.call(0x0020_1360, &[s0, 2, args, sp, sp + 0x20, sp + 0x24], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_a0b0, &[s0, 0x24], &[]);
    let x0 = m.r32(sp + 0x20);
    let x1 = m.r32(sp + 0x24);
    let o = m.r32(obj + 0xf64);
    m.call(0x0015_4038, &[o, x0, x1], &[]);
    m.call(0x0012_91a8, &[ret, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0x70);
    ret
}

/// FUN_001019d0: script method, 3 args (types at DAT_002a3ad0): FUN_0012fde8(obj, z) with f12/f13 = x, y; ret = nil.
pub fn ee_sub_1019d0<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, args: u32) -> u32 {
    let sp = stack_push(m, 0x70);
    let s0 = sp + 0x10;
    let w = m.r32(0x002a_3ad0);
    m.w32(sp + 0x0, w);
    let w = m.r32(0x002a_3ad4);
    m.w32(sp + 0x4, w);
    let w = m.r32(0x002a_3ad8);
    m.w32(sp + 0x8, w);
    m.call(0x0020_1360, &[s0, 3, args, sp, sp + 0x20, sp + 0x24, sp + 0x28], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_a0b0, &[s0, 0x24], &[]);
    let a1 = m.r32(sp + 0x28);
    let f12 = m.rf(sp + 0x20);
    let f13 = m.rf(sp + 0x24);
    m.call(0x0012_fde8, &[obj, a1], &[f4(f12), f4(f13)]);
    m.call(0x0012_91a8, &[ret, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0x70);
    ret
}

/// FUN_0010a620: script method, 3 args (types at DAT_002a3c88): FUN_001e1ad0(obj, ...); ret = nil.
pub fn ee_sub_10a620<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, args: u32) -> u32 {
    let sp = stack_push(m, 0x70);
    let s0 = sp + 0x10;
    let w = m.r32(0x002a_3c88);
    m.w32(sp + 0x0, w);
    let w = m.r32(0x002a_3c8c);
    m.w32(sp + 0x4, w);
    let w = m.r32(0x002a_3c90);
    m.w32(sp + 0x8, w);
    m.call(0x0020_1360, &[s0, 3, args, sp, sp + 0x20, sp + 0x24, sp + 0x28], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_a0b0, &[s0, 0x24], &[]);
    let x0 = m.r32(sp + 0x20);
    let x1 = m.r32(sp + 0x24);
    let x2 = m.r32(sp + 0x28);
    m.call(0x001e_1ad0, &[obj, x0, x1, x2], &[]);
    m.call(0x0012_91a8, &[ret, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0x70);
    ret
}

/// FUN_0010c5f0: script method, 3 args (types at DAT_002a3f50): FUN_001b1b18(obj, ...); ret = nil.
pub fn ee_sub_10c5f0<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, args: u32) -> u32 {
    let sp = stack_push(m, 0x70);
    let s0 = sp + 0x10;
    let w = m.r32(0x002a_3f50);
    m.w32(sp + 0x0, w);
    let w = m.r32(0x002a_3f54);
    m.w32(sp + 0x4, w);
    let w = m.r32(0x002a_3f58);
    m.w32(sp + 0x8, w);
    m.call(0x0020_1360, &[s0, 3, args, sp, sp + 0x20, sp + 0x24, sp + 0x28], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_a0b0, &[s0, 0x24], &[]);
    let x0 = m.r32(sp + 0x20);
    let x1 = m.r32(sp + 0x24);
    let x2 = m.r32(sp + 0x28);
    m.call(0x001b_1b18, &[obj, x0, x1, x2], &[]);
    m.call(0x0012_91a8, &[ret, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0x70);
    ret
}

/// FUN_00110140: script method, 3 args (types at DAT_002a40a8): FUN_001816c0(obj, ...); ret = nil.
pub fn ee_sub_110140<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, args: u32) -> u32 {
    let sp = stack_push(m, 0x70);
    let s0 = sp + 0x10;
    let w = m.r32(0x002a_40a8);
    m.w32(sp + 0x0, w);
    let w = m.r32(0x002a_40ac);
    m.w32(sp + 0x4, w);
    let w = m.r32(0x002a_40b0);
    m.w32(sp + 0x8, w);
    m.call(0x0020_1360, &[s0, 3, args, sp, sp + 0x20, sp + 0x24, sp + 0x28], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_a0b0, &[s0, 0x24], &[]);
    let x0 = m.r32(sp + 0x20);
    let x1 = m.r32(sp + 0x24);
    let x2 = m.r32(sp + 0x28);
    m.call(0x0018_16c0, &[obj, x0, x1, x2], &[]);
    m.call(0x0012_91a8, &[ret, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0x70);
    ret
}

/// FUN_0010ba70: script method, 4 args (types at DAT_002a3bf8): FUN_002288d8(obj, ...); ret = nil.
pub fn ee_sub_10ba70<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, args: u32) -> u32 {
    let sp = stack_push(m, 0x70);
    let s0 = sp + 0x10;
    let w = m.r32(0x002a_3bf8);
    m.w32(sp + 0x0, w);
    let w = m.r32(0x002a_3bfc);
    m.w32(sp + 0x4, w);
    let w = m.r32(0x002a_3c00);
    m.w32(sp + 0x8, w);
    let w = m.r32(0x002a_3c04);
    m.w32(sp + 0xc, w);
    m.call(0x0020_1360, &[s0, 4, args, sp, sp + 0x20, sp + 0x24, sp + 0x28, sp + 0x2c], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_a0b0, &[s0, 0x24], &[]);
    let x0 = m.r32(sp + 0x20);
    let x1 = m.r32(sp + 0x24);
    let x2 = m.r32(sp + 0x28);
    let x3 = m.r32(sp + 0x2c);
    m.call(0x0022_88d8, &[obj, x0, x1, x2, x3], &[]);
    m.call(0x0012_91a8, &[ret, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0x70);
    ret
}

/// FUN_00103b38: script method, 2 object args (types at DAT_002a3b70): FUN_0020a058(obj, &x, &y); ret = nil.
pub fn ee_sub_103b38<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, args: u32) -> u32 {
    let sp = stack_push(m, 0xa0);
    let s0 = sp + 0x30;
    let s1 = sp + 0x20;
    let s2 = sp + 0x10;
    let w = m.r32(0x002a_3b70);
    m.w32(sp + 0x0, w);
    let w = m.r32(0x002a_3b74);
    m.w32(sp + 0x4, w);
    m.w32(s1, 0xffff_ffff);
    m.w32(s2, 0xffff_ffff);
    m.call(0x0020_1360, &[s0, 2, args, sp, s2, s1], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_a0b0, &[s0, 0x24], &[]);
    m.call(0x0020_a058, &[obj, s2, s1], &[]);
    m.call(0x0012_91a8, &[ret, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_9ec8, &[s1], &[]);
    m.call(0x0012_9ec8, &[s2], &[]);
    stack_pop(m, 0xa0);
    ret
}

/// FUN_0010f118: script method, 6 args (types at DAT_002a4018): FUN_001638a0(obj, x0..x5); ret = nil.
pub fn ee_sub_10f118<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, args: u32) -> u32 {
    let sp = stack_push(m, 0xa0);
    let s0 = sp + 0x30;
    m.w32(sp, sp + 0x50);
    m.w32(sp + 8, sp + 0x54);
    let w = m.r32(0x002a_4018);
    m.w32(sp + 0x10 + 0x0, w);
    let w = m.r32(0x002a_401c);
    m.w32(sp + 0x10 + 0x4, w);
    let w = m.r32(0x002a_4020);
    m.w32(sp + 0x10 + 0x8, w);
    let w = m.r32(0x002a_4024);
    m.w32(sp + 0x10 + 0xc, w);
    let w = m.r32(0x002a_4028);
    m.w32(sp + 0x10 + 0x10, w);
    let w = m.r32(0x002a_402c);
    m.w32(sp + 0x10 + 0x14, w);
    m.call(0x0020_1360, &[s0, 6, args, sp + 0x10, sp + 0x40, sp + 0x44, sp + 0x48, sp + 0x4c, sp + 0x50, sp + 0x54], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    m.call(0x0012_a0b0, &[s0, 0x24], &[]);
    let x0 = m.r32(sp + 0x40);
    let x1 = m.r32(sp + 0x44);
    let x2 = m.r32(sp + 0x48);
    let x3 = m.r32(sp + 0x4c);
    let x4 = m.r32(sp + 0x50);
    let x5 = m.r32(sp + 0x54);
    m.call(0x0016_38a0, &[obj, x0, x1, x2, x3, x4, x5], &[]);
    m.call(0x0012_91a8, &[ret, s0], &[]);
    m.call(0x0012_9ec8, &[s0], &[]);
    stack_pop(m, 0xa0);
    ret
}
