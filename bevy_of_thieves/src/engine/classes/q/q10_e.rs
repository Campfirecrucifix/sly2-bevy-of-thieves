//! WP-Q10 file packages: 47 identical C++ static-init pairs (pool-allocator constructors: FUN_<x> with
//! (param_1 = 1, param_2 = 0xffff) guard + 32-byte wrapper FUN_<x'> calling it with (1, 0xffff)).
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem};

/// refcount DATs (DAT_002dd2e0..f4) -> pool creation args (FUN_0011cca8 a0..a3) on the first reference
const POOLS: [(u32, u32, u32, u32, u32); 6] = [
    (0x002d_d2e0, 0x45_20a8, 0x14, 0x2000, 0x10),
    (0x002d_d2e4, 0x45_20d0, 0x18, 0x800, 0x10),
    (0x002d_d2e8, 0x45_20f8, 0x28, 0x200, 0x10),
    (0x002d_d2ec, 0x45_2120, 0x1c, 0x800, 0x10),
    (0x002d_d2f0, 0x45_2148, 0xc, 200, 0x10),
    (0x002d_d2f4, 0x45_2170, 0x14, 0x100, 0x10),
];

/// the shared body of every static-init FUN (47 byte-identical copies in the ELF)
/// 00188330 / 001b0a20 / 001d1218 / 001d70e0 / 001df980 / 001e5c28 / 00220be8
pub fn pool_static_init<M: QMem + ?Sized>(m: &mut M, p1: u32, p2: u32) {
    pool_static_init_n(m, p1, p2, 6)
}

/// FUN_0012a1a0 variant: 11 pools (POOLS + POOLS_EXT)
pub fn fun_0012a1a0<M: QMem + ?Sized>(m: &mut M, p1: u32, p2: u32) {
    pool_static_init_n(m, p1, p2, 11)
}
pub fn fun_0012a3b8<M: QMem + ?Sized>(m: &mut M) {
    fun_0012a1a0(m, 1, 0xffff)
}

const POOLS_EXT: [(u32, u32, u32, u32, u32); 5] = [
    (0x002d_d300, 0x45_2188, 0x20, 0x100, 0x10),
    (0x002d_d304, 0x45_21a0, 0x50, 0x100, 0x10),
    (0x002d_d308, 0x45_21b8, 0x20, 0x40, 0x10),
    (0x002d_d30c, 0x45_21d0, 0xc, 0x40, 0x10),
    (0x002d_d310, 0x45_21e8, 0x10, 0x20, 0x10),
];

fn pool_static_init_n<M: QMem + ?Sized>(m: &mut M, p1: u32, p2: u32, n: usize) {
    if p2 == 0xffff && p1 != 0 {
        for (dat, a0, a1, a2, a3) in POOLS.into_iter().chain(POOLS_EXT).take(n) {
            let n = m.r32(dat).wrapping_add(1);
            m.w32(dat, n);
            if n == 1 {
                m.call(0x0011_cca8, &[a0, a1, a2, a3], &[]);
            }
        }
    }
}

macro_rules! pair {
    ($($init:ident, $wrap:ident);* $(;)?) => {$(
        pub fn $init<M: QMem + ?Sized>(m: &mut M, p1: u32, p2: u32) { pool_static_init(m, p1, p2) }
        pub fn $wrap<M: QMem + ?Sized>(m: &mut M) { pool_static_init(m, 1, 0xffff) }
    )*};
}
pair! {
    fun_0011e040, fun_0011e178; fun_0011f2e8, fun_0011f420; fun_00125798, fun_001258d0;
    fun_00126030, fun_00126168; fun_00126188, fun_001262c0; fun_001262e0, fun_00126418;
    fun_0012e4a8, fun_0012e5e0; fun_0012e9f0, fun_0012eb28;
    fun_0012ef98, fun_0012f0d0; fun_0012f0f0, fun_0012f228; fun_00137780, fun_001378b8;
    fun_001378d8, fun_00137a10; fun_0013a890, fun_0013a9c8; fun_0013bc98, fun_0013bdd0;
    fun_0013fe18, fun_0013ff50; fun_001444e0, fun_00144618; fun_001449d0, fun_00144b08;
    fun_00149eb0, fun_00149fe8; fun_00151f70, fun_001520a8; fun_00158480, fun_001585b8;
    fun_00158fc0, fun_001590f8; fun_0015db28, fun_0015dc60; fun_0015e228, fun_0015e360;
    fun_0015eda8, fun_0015eee0; fun_00173000, fun_00173138; fun_00174740, fun_00174878;
    fun_001864b8, fun_001865f0; fun_00186ed8, fun_00187010; fun_00188330, fun_00188468;
    fun_0018e3e8, fun_0018e520; fun_0018f7b0, fun_0018f8e8; fun_001a09c0, fun_001a0af8;
    fun_001aa820, fun_001aa958; fun_001b0a20, fun_001b0b58; fun_001b1c60, fun_001b1d98;
    fun_001d1218, fun_001d1350; fun_001d70e0, fun_001d7218; fun_001d9280, fun_001d93b8;
    fun_001df980, fun_001dfab8; fun_001e5c28, fun_001e5d60; fun_0020b6e8, fun_0020b820;
    fun_0020ccd0, fun_0020ce08; fun_0020d478, fun_0020d5b0; fun_0020d878, fun_0020d9b0;
    fun_00220be8, fun_00220d20; fun_0022c6c0, fun_0022c7f8;
}

/// FUN_001252d0(list, out): out[0] = list length (chain via +0x10), out[1] = u16 buffer (FUN_001ae318, 2 B/node,
/// align 0x10) holding node[+0xc] for the nodes with node[+8] == 0x15 or 0x18 at the node's index
pub fn fun_001252d0<M: QMem + ?Sized>(m: &mut M, list: u32, out: u32) {
    m.w32(out, 0);
    m.w32(out + 4, 0);
    let mut n = 0u32;
    let mut p = list;
    while p != 0 {
        n = n.wrapping_add(1);
        p = m.r32(p + 0x10);
    }
    m.w32(out, n);
    if n == 0 {
        return;
    }
    let sp = stack_push(m, 0x10);
    m.w32(sp, n << 1);
    m.w32(sp + 4, 0x10);
    let buf = m.call(0x001a_e318, &[sp], &[]).0;
    stack_pop(m, 0x10);
    m.w32(out + 4, buf);
    let mut i = 0u32;
    let mut p = list;
    while p != 0 {
        let t = m.r32(p + 8);
        if t == 0x15 || t == 0x18 {
            let v = m.r32(p + 0xc) as u16;
            m.w16(buf.wrapping_add(i * 2), v);
        }
        p = m.r32(p + 0x10);
        i += 1;
    }
}
