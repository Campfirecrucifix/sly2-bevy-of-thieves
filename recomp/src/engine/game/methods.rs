//! WP-J4: the Splice method binding table (CMethod refs handed to scripts by get-o).
//!
//! Original data (read from SCUS_973.16 by `method_table.rs`, WP-PUB2):
//!  - property field table 0x002ac880 (1449 x 0x28): kind 0x12 rows are methods. `flags & 1` = get-o
//!    (FUN_001ffd98) builds a CMethod {+8 object, +0xc fn = field+0x10, +0x10 nargs = field+0x18};
//!    rows without bit 1 (0x400-only, thunk at +8) fall through FUN_001ffd98's kind switch and get-o
//!    returns Nil for them (not script-reachable through get-o).
//!  - per-class sorted {symid, field*} tables at 0x00279ad0 (199 x {count, ptr}), binary searched by
//!    FUN_001ffcf8 with the class id of the object's class info (vtable +0x34, +4).
//!  - every method fn is a thunk `fn(out, obj, ?, argv)`: FUN_00201360(out, nargs, argv, kinds, ptrs..)
//!    unpacks the CRefs by field kind (FUN_00201110), calls one engine function, returns Void.

pub mod cls;
pub mod find;
pub mod getters;
pub mod aim;
pub mod lo;

use crate::engine::splice::value::{Method, Ref};
use std::rc::Rc;

pub use super::method_table::{BINDINGS, BINDING_COUNT, GETTERS, GETTER_BINDINGS, GETTER_BINDING_COUNT, GETTER_COUNT, METHODS, METHOD_COUNT};

/// One method field (row of 0x002ac880 with kind 0x12).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MethodDesc {
    /// field index in 0x002ac880
    pub field: u16,
    /// field +4
    pub flags: u32,
    /// thunk address (field +0x10 when flags & 1, else +8); 0 = empty row
    pub thunk: u32,
    /// field +0x18
    pub nargs: u8,
    /// FUN_00201360 kind array (None = not recovered from the thunk body)
    pub kinds: Option<&'static [u32]>,
    /// the engine function the thunk calls (0 = thunk does the work inline)
    pub callee: u32,
}

/// One (class, symid) -> method entry of the 0x00279ad0 tables.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MethodBinding {
    pub class: u16,
    pub symid: u16,
    /// index into `METHODS`
    pub method: u16,
}

/// WP-J9: CRef setter a getter thunk wraps the engine callee's result with before FUN_001291a8(out, ..).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GetterRet {
    /// FUN_00129598 (Bool, value != 0)
    Bool,
    /// FUN_00129558 (F32)
    F32,
    /// FUN_00129518 (S32)
    S32,
    /// FUN_00129a10 (Basic object pointer)
    Basic,
    /// FUN_001296a8 (Vector)
    Vector,
    /// FUN_00129808 (Matrix)
    Matrix,
    /// FUN_00129878 (Clq)
    Clq,
    /// FUN_001298f0 (Lm)
    Lm,
    /// FUN_00129718 (128-bit register -> type 0x1a Vector; WP-J13: not a float pair)
    F32x2,
    /// FUN_00129658 (copy of a CRef)
    Ref,
    /// thunk body not recovered
    Unknown,
}

/// WP-J9: a row of 0x002ac880 with kind != 0x12 and `flags & 1`: FUN_001ffd98 tests bit 1 before the
/// kind switch, so get-o hands out a CMethod for it exactly like a kind-0x12 method; its thunk returns
/// the engine callee's result (a value), not Void. `kind` = field +0 (the value kind of the result).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GetterDesc {
    pub kind: u32,
    pub ret: GetterRet,
    pub m: MethodDesc,
}

/// FUN_001ffcf8 restricted to getter fields (flags & 1, kind != 0x12).
pub fn ee_sub_1ffcf8_getter(class: u16, symid: u16) -> Option<&'static GetterDesc> {
    // 0x001ffcf8
    let lo = GETTER_BINDINGS.partition_point(|b| (b.class, b.symid) < (class, symid));
    let b = GETTER_BINDINGS.get(lo).filter(|b| b.class == class && b.symid == symid)?;
    Some(&GETTERS[b.method as usize])
}

/// Getter by thunk address.
pub fn getter_by_thunk(thunk: u32) -> Option<&'static GetterDesc> {
    GETTERS.iter().find(|g| g.m.thunk == thunk && thunk != 0)
}

impl MethodDesc {
    /// get-o returns a CMethod (FUN_001ffd98 `flags & 1`).
    pub fn script_reachable(&self) -> bool {
        self.flags & 1 != 0
    }
}

/// FUN_001ffcf8 restricted to method fields: binary search of the class's {symid, field} table.
/// Returns None when the symid is not bound for that class or is not a method field.
pub fn ee_sub_1ffcf8(class: u16, symid: u16) -> Option<&'static MethodDesc> {
    // 0x001ffcf8
    let lo = BINDINGS.partition_point(|b| (b.class, b.symid) < (class, symid));
    let b = BINDINGS.get(lo).filter(|b| b.class == class && b.symid == symid)?;
    Some(&METHODS[b.method as usize])
}

/// FUN_00200ba0 -> FUN_001ffd98 for a method field: the CMethod ref get-o returns (symid < 0x4d1).
/// `None` = not a method of this class (caller handles ordinary fields / user properties);
/// `Some(Ref::Nil)` = method row without bit 1 (falls through the kind switch).
pub fn get_o_method(object: u32, class: u16, symid: u16) -> Option<Ref> {
    // 0x001ffd98 (flags & 1 branch): CMethod +8 obj, +0xc field+0x10, +0x10 field+0x18
    let m = match ee_sub_1ffcf8(class, symid) {
        Some(m) => m,
        // WP-J9: getter rows (any kind, flags & 1)
        None => &ee_sub_1ffcf8_getter(class, symid)?.m,
    };
    if !m.script_reachable() {
        return Some(Ref::Nil);
    }
    Some(Ref::Method(Rc::new(Method { object, method_id: m.thunk, nargs: m.nargs as usize })))
}

/// Method by thunk address (the `Method::method_id` the port uses).
pub fn by_thunk(thunk: u32) -> Option<&'static MethodDesc> {
    METHODS.iter().find(|m| m.thunk == thunk && thunk != 0)
}

/// A method argument after FUN_00201110 conversion.
#[derive(Clone, Debug)]
pub enum MArg {
    /// kind 1: float (S32 converted by FUN_00129da8)
    F32(f32),
    /// kinds 5..8, 0xe..0x11, 0x13..0x15: int (non-S32 converted by FUN_00129d58)
    I32(i32),
    /// kind 0 / 0x1000 / 0x2000 kinds: the payload word (object pointer, bool, ...)
    Word(u32),
    /// kinds 0x8000: the CRef itself
    Ref(Ref),
    /// kinds 2/9/4/10/0xb: payload copied by value (vector/matrix/...)
    Value(Ref),
    /// kinds 0xd/0x16: nothing written
    None,
}

/// FUN_00201110: unpack `args` by the thunk's kind array.
pub fn ee_sub_201110(kinds: &[u32], args: &[Ref]) -> Vec<MArg> {
    // 0x00201110
    kinds
        .iter()
        .zip(args)
        .map(|(&k, a)| match k {
            1 => MArg::F32(a.as_f32()),
            2 | 9 | 4 | 10 | 0xb => MArg::Value(a.clone()),
            5..=8 | 0xe..=0x11 | 0x13..=0x15 => MArg::I32(match a {
                Ref::S32(n) => *n,
                // FUN_00129d58 CRef::GetInt: F32 truncated, others 0
                Ref::F32(f) => *f as i32,
                _ => 0,
            }),
            0xd | 0x16 => MArg::None,
            k if k & 0x8000 != 0 => MArg::Ref(a.clone()),
            _ => MArg::Word(a.payload()),
        })
        .collect()
}

/// Thunks whose engine side exists in the port and are executed by `vm_state::PersistentHost`.
pub const IMPLEMENTED: &[u32] = &[
    0x0010_1e18, // FUN_001fb018 emitter play++ (class 151)
    0x0010_1e90, // FUN_001fb060 emitter play--
    0x0010_1f08, // FUN_001fb098 emitter stop (fade f32)
    0x0010_26e0, // FUN_0014f750 cm shake (class 135)
    0x0010_2768, // FUN_0014d8c0 cm cut
    0x0010_b8f8, // FUN_002289b0 spawn alias 0x189 (class 42)
    0x0010_b970, // FUN_002289d0 spawn alias 0x18a
    0x0010_b9e8, // FUN_00228988 spawn alias (arg)
    0x0010_ba70, // WP-P25: FUN_002288d8 spawn rec (class 42, +0xe0/+0xe4)
    0x0010_bb38, // WP-P25: FUN_00228920 spawn task (class 42, +0x124/+0x128)
    0x0010_bbe0, // WP-P25: FUN_00228958 spawn pair (class 42, +0x148/+0x14c)
    0x0010_b850, // WP-SCR10: FUN_002284e0 warp-volume task (class 26, +0x164/+0x168)
    0x0010_1d20, // WP-J5: FUN_00130b18 ASEG retract (class 109/110) -> fields::ObjectFields::retract
];

/// Coverage of the binding table (stored by tests/engine_j4.rs).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Coverage {
    pub methods: usize,
    pub empty_rows: usize,
    pub script_reachable: usize,
    pub bindings: usize,
    pub kinds_recovered: usize,
    pub implemented: usize,
}

pub fn coverage() -> Coverage {
    Coverage {
        methods: METHOD_COUNT,
        empty_rows: METHODS.iter().filter(|m| m.thunk == 0).count(),
        script_reachable: METHODS.iter().filter(|m| m.script_reachable() && m.thunk != 0).count(),
        bindings: BINDING_COUNT,
        kinds_recovered: METHODS.iter().filter(|m| m.kinds.is_some()).count(),
        implemented: METHODS.iter().filter(|m| IMPLEMENTED.contains(&m.thunk)).count(),
    }
}
