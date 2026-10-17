//! WP-Q9: C2cc340: same follower as C2cc2c0 (shared `follow_*`) with its own limits (DAT_002bd344/348/34c) and busy
//! offset 0x1810.
use crate::engine::classes::q::c2cc2c0::*;
use crate::engine::classes::q::q2mem::QMem;

pub fn ee_sub_145cd0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    follow_v6(m, o)
}
pub fn ee_sub_145d00<M: QMem + ?Sized>(m: &mut M, o: u32) {
    follow_v7(m, o)
}
pub fn ee_sub_145d68<M: QMem + ?Sized>(m: &mut M, o: u32) {
    follow_v8(m, o)
}
pub fn ee_sub_145db0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    follow_v9(m, o, &FOLLOW_340)
}
pub fn ee_sub_146068<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32) {
    follow_v13(m, o, p2)
}
pub fn ee_sub_146158<M: QMem + ?Sized>(m: &mut M, o: u32, new: u32) {
    follow_set_state(m, o, new)
}
