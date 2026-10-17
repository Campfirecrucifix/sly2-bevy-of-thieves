//! WP-Q4: run_001285d8 (CRef internals). Everything else in the run is the CRef pool / GC / copy-assign
//! machinery that the Rust `Ref` enum (Rc ownership, Clone, Drop) replaces; see port_tracking.
use super::value::Ref;
use std::rc::Rc;

/// 0x00129ac0: CRef value equality (no direct callers; same tag required). Tags 0x15..0x19 and 0x1f..0x23
/// compare the payload word (F32 bitwise, pair / proc / method by identity); vector / CLQ / SMP the 3 floats,
/// LM the 2 floats, matrix 0x40 bytes bitwise (FUN_00233178 = memcmp); keyword / void / nil tags are equal.
pub fn ee_sub_129ac0(a: &Ref, b: &Ref) -> bool {
    if a.tagk() != b.tagk() {
        return false;
    }
    match (a, b) {
        (Ref::F32(x), Ref::F32(y)) => x.to_bits() == y.to_bits(),
        (Ref::Vector(x), Ref::Vector(y)) | (Ref::Clq(x), Ref::Clq(y)) | (Ref::Smp(x), Ref::Smp(y)) => x == y,
        (Ref::Lm(x), Ref::Lm(y)) => x == y,
        (Ref::Matrix(x), Ref::Matrix(y)) => x.iter().zip(y).all(|(p, q)| p.to_bits() == q.to_bits()),
        (Ref::Pair(x), Ref::Pair(y)) => Rc::ptr_eq(x, y),
        (Ref::Proc(x), Ref::Proc(y)) => Rc::ptr_eq(x, y),
        (Ref::Method(x), Ref::Method(y)) => Rc::ptr_eq(x, y),
        _ => a.payload() == b.payload(),
    }
}
