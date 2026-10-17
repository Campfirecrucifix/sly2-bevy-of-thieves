//! WP-Q6: shared helpers of the Q6 queue ports (classes/q/q6_*.rs, engine/*/q_*.rs) and the trivial
//! overrides of the Q6 class packages (constant getters / empty slots) as data.
//! Conventions as in q2mem: object memory through `QMem`, callees outside the package by address
//! through `QMem::call` (ints in `a`, floats / quadwords splatted in `v`, float results in `.1[0]`).
//! Read-only constants (.rodata 0x262380..0x2ac744) come from the ELF image (`ro32` / `rof`);
//! .data / .bss globals are host memory (`QMem`).
use crate::engine::classes::q::q2mem::{QMem, V4};

/// .rodata word (SCUS_973.16 image)
pub fn ro32(a: u32) -> u32 {
    crate::engine::disc::exe::exe().w(a)
}
/// .rodata float
pub fn rof(a: u32) -> f32 {
    f32::from_bits(ro32(a))
}
/// .rodata quadword
pub fn rov(a: u32) -> V4 {
    [rof(a), rof(a + 4), rof(a + 8), rof(a + 12)]
}
/// splat a float argument
pub fn s(x: f32) -> V4 {
    [x; 4]
}
/// float-returning call
pub fn callf<M: QMem + ?Sized>(m: &mut M, f: u32, a: &[u32], v: &[V4]) -> f32 {
    m.call(f, a, v).1[0]
}
/// virtual call through an embedded vtable pointer: `(**(code **)(*(obj + vt) + off))(obj + *(short *)(*(obj + vt) + off - 4), ..)`
pub fn vcall<M: QMem + ?Sized>(m: &mut M, obj: u32, vt: u32, off: u32, a: &[u32], v: &[V4]) -> (u32, V4) {
    let t = m.r32(obj + vt);
    let f = m.r32(t + off);
    let adj = m.r16(t + off - 4) as i16 as i32;
    let mut args = vec![obj.wrapping_add(adj as u32)];
    args.extend_from_slice(a);
    m.call(f, &args, v)
}
/// `*(T *)(p) |= 2` on a tagged pointer field: bit 0 set = flags stored inline, else pointer to the flags
/// (the `(x ^ 1) & 1 == 0` pattern of +0x40 in several init slots)
pub fn set_tagged<M: QMem + ?Sized>(m: &mut M, field: u32, bits: u32) {
    let p = m.r32(field);
    if (p ^ 1) & 1 == 0 {
        m.w32(field, p | bits);
    } else {
        let v = m.r32(p);
        m.w32(p, v | bits);
    }
}
/// &DAT_003dd490: the world
pub const WORLD: u32 = 0x003d_d490;
/// DAT_002dded0: world clock (seconds)
pub const CLOCK: u32 = 0x002d_ded0;

/// value of a constant-returning override (CONST_RET / NOPS `return 0`)
pub fn const_ret(f: u32) -> Option<u32> {
    CONST_RET.iter().find(|x| x.0 == f).map(|x| x.2).or_else(|| NOPS.iter().find(|x| x.0 == f).and_then(|x| x.2))
}
/// true if `f` is one of the Q6 empty / constant overrides
pub fn is_trivial(f: u32) -> bool {
    CONST_RET.iter().any(|x| x.0 == f) || NOPS.iter().any(|x| x.0 == f)
}

/// Q6 overrides whose whole body is `return <const>;` (class slot 5 = class descriptor getter into the
/// 0x002baae8 table; 0x18de08 / 0x1cf5c8 other slots): (fn, owner name, value).
pub const CONST_RET: &[(u32, &str, u32)] = &[
    (0x113a10, "C2cb1f0_v5", 0x2baae8),
    (0x113a90, "C2d2cb8_v5", 0x2bab88),
    (0x113af0, "C2d4de0_v5", 0x2bac00),
    (0x113b10, "C2cdba0_v5", 0x2bac28),
    (0x113b50, "C2c9a80_v5", 0x2bac78),
    (0x113ba0, "C2c9c20_v5", 0x2bacdc),
    (0x113bd0, "C2cb5d8_v5", 0x2bad18),
    (0x113bf0, "C2ce9d0_v5", 0x2bad40),
    (0x113cf0, "C2d9f70_v5", 0x2bae80),
    (0x113d40, "C2da110_v5", 0x2baee4),
    (0x113d60, "C2d2e98_v5", 0x2baf0c),
    (0x113d90, "C2da568_v5", 0x2baf48),
    (0x113da0, "C2cf520_v5", 0x2baf5c),
    (0x113db0, "C2c9dc0_v5", 0x2baf70),
    (0x113dd0, "C2d4820_v5", 0x2baf98),
    (0x113de0, "C2d7188_v5", 0x2bafac),
    (0x113df0, "C2c91c8_v5", 0x2bafc0),
    (0x113e00, "C2ccbb8_v5", 0x2bafd4),
    (0x113e20, "C2c8ef0_v5", 0x2baffc),
    (0x113e30, "C2c8c18_v5", 0x2bb010),
    (0x113e40, "C2c8940_v5", 0x2bb024),
    (0x113e50, "C2cd748_v5", 0x2bb038),
    (0x113e70, "C2d80d0_v5", 0x2bb060),
    (0x113e80, "C2d1878_v5", 0x2bb074),
    (0x113e90, "C2d15d0_v5", 0x2bb088),
    (0x113eb0, "C2d1328_v5", 0x2bb0b0),
    (0x113ec0, "C2d0dd8_v5", 0x2bb0c4),
    (0x113ee0, "C2d7ca8_v5", 0x2bb0ec),
    (0x113ef0, "C2d5b30_v5", 0x2bb100),
    (0x113f50, "C2d89a0_v5", 0x2bb178),
    (0x113fb0, "C2d6b98_v5", 0x2bb1f0),
    (0x113fc0, "C2d6928_v5", 0x2bb204),
    (0x113fd0, "C2d6448_v5", 0x2bb218),
    (0x113fe0, "C2d66b8_v5", 0x2bb22c),
    (0x113ff0, "C2d61d8_v5", 0x2bb240),
    (0x114000, "C2d5f68_v5", 0x2bb254),
    (0x114040, "C2d5450_v5", 0x2bb2a4),
    (0x1140b0, "C2d8dd8_v5", 0x2bb330),
    (0x1140c0, "C2dcc98_v5", 0x2bb344),
    (0x1140f0, "C2ca300_v5", 0x2bb380),
    (0x114180, "C2cfa90_v5", 0x2bb434),
    (0x1141b0, "C2cda10_v5", 0x2bb470),
    (0x1141e0, "C2da3d8_v5", 0x2bb4ac),
    (0x1141f0, "C2daf30_v5", 0x2bb4c0),
    (0x114240, "C2ceeb8_v5", 0x2bb524),
    (0x114280, "C2ca4d0_v5", 0x2bb574),
    (0x1142a0, "C2ca240_v5", 0x2bb59c),
    (0x114330, "C2da9d0_v5", 0x2bb650),
    (0x114350, "C2d9938_v5", 0x2bb678),
    (0x114360, "C2ca2c0_v5", 0x2bb68c),
    (0x114390, "C2ca510_v5", 0x2bb6c8),
    (0x1143a0, "C2c94a0_v5", 0x2bb6dc),
    (0x1143b0, "C2ca490_v5", 0x2bb6f0),
    (0x1143c0, "C2d9480_v5", 0x2bb704),
    (0x114480, "C2d06a0_v5", 0x2bb7f4),
    (0x1144c0, "C2cfa10_v5", 0x2bb844),
    (0x114510, "C2d0420_v5", 0x2bb8a8),
    (0x18de08, "C2d2e98_v102", 0x2a5ef0),
    (0x1cf5c8, "ee_sub_1cf5c8", 0x110),
    (0x258f30, "C2ced18_v5", 0x2bad2c),
    (0x2599c8, "C2ce680_v5", 0x2bad68),
];

/// Q6 overrides whose body is empty (`jr ra`; Some(0) = `return 0`): (fn, owner name, return).
pub const NOPS: &[(u32, &str, Option<u32>)] = &[
    (0x116a90, "C2ca578_v2", None),
    (0x116b28, "C2ca550_v2", None),
    (0x11d9d8, "ee_sub_11d9d8", None),
    (0x12cf20, "C2ca6a0_v2", None),
    (0x132b08, "C2c94a0_v1", None),
    (0x14b460, "C2ccbb8_v85", Some(0)),
    (0x14fcc0, "C2cab90_v0", None),
    (0x14fd30, "C2cab90_v3", None),
    (0x14fd38, "C2cab90_v4", None),
    (0x14fd40, "C2cd570_v5", None),
    (0x14fd48, "C2cab90_v6", None),
    (0x182cc0, "C2d28d8_v0", Some(0)),
    (0x182cd0, "C2d28d8_v2", None),
    (0x182d70, "C2d28d8_v3", None),
    (0x182e78, "C2d28d8_v5", Some(0)),
    (0x1a6ee8, "C2c94e0_v42", Some(0)),
    (0x1a6ef0, "C2c94e0_v43", None),
    (0x1a6ef8, "C2c94e0_v44", None),
    (0x1cf570, "C2d7468_v2", None),
    (0x1d9870, "C2d86d0_v6", None),
    (0x1eef70, "C2d9480_v7", None),
    (0x1fc9a8, "C2d9be0_v14", None),
];
