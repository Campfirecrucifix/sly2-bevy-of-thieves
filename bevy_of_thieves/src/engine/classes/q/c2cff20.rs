//! WP-Q9: C2cff20 slot overrides (v7 stays on the emulator path: FUN_0011b830 takes a 64-bit xy argument).
use crate::engine::classes::q::q2mem::QMem;

/// slot 6: parent FUN_001eef68; FUN_00209890(owner, 0x28)
pub fn ee_sub_17d1c0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef68, &[o], &[]);
    let ow = m.r32(o + 0x10);
    m.call(0x0020_9890, &[ow, 0x28], &[]);
}
/// slot 8: parent FUN_001eef78; FUN_00209970(owner, 0x28)
pub fn ee_sub_17d280<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001e_ef78, &[o], &[]);
    let ow = m.r32(o + 0x10);
    m.call(0x0020_9970, &[ow, 0x28], &[]);
}
