//! WP-Q3: run_00131990 Splice user-property setters (FUN_00132bc8..FUN_00132e40) and the method
//! thunks 0x001002a8..0x00100760 (fields 3..10) that call them. Each callee builds one CRef of a
//! fixed tag and stores it with FUN_00200d10 (ensure-o: overwrite the object's user property
//! `symid`, else prepend it) = `fields::ObjectFields::ee_sub_200d10`.

use super::fields::ObjectFields;
use super::methods::MArg;
use crate::engine::splice::value::Ref;

/// Method thunks (field table rows 3..10) -> callee, executed by `call`.
pub const THUNKS: [(u32, u32); 8] = [
    (0x0010_02a8, 0x0013_2bc8),
    (0x0010_0350, 0x0013_2c20),
    (0x0010_03f8, 0x0013_2c78),
    (0x0010_04a0, 0x0013_2cc8),
    (0x0010_0558, 0x0013_2d28),
    (0x0010_0610, 0x0013_2d88),
    (0x0010_06b8, 0x0013_2de0),
    (0x0010_0760, 0x0013_2e40),
];

/// FUN_00132bc8(obj, symid, v): tag 0x17 Bool(v != 0).
pub fn ee_sub_132bc8(o: &mut ObjectFields, obj: u32, symid: u32, v: u32) {
    // 0x00132bc8
    o.ee_sub_200d10(obj, symid, Ref::Bool(v != 0));
}

/// FUN_00132c20(obj, symid, n): tag 0x15 S32.
pub fn ee_sub_132c20(o: &mut ObjectFields, obj: u32, symid: u32, n: i32) {
    // 0x00132c20
    o.ee_sub_200d10(obj, symid, Ref::S32(n));
}

/// FUN_00132c78(f, obj, symid): tag 0x16 F32 (the float comes in $f12, hence first).
pub fn ee_sub_132c78(o: &mut ObjectFields, f: f32, obj: u32, symid: u32) {
    // 0x00132c78
    o.ee_sub_200d10(obj, symid, Ref::F32(f));
}

/// FUN_00132cc8(obj, symid, &v): FUN_00129330 = new tag 0x1c Clq from the 16 bytes at v.
/// FUN_00132d28: FUN_00129398 = tag 0x1d Lm (8 bytes); FUN_00132de0: FUN_00129248 = tag 0x1a
/// Vector (qword). The thunk unpacked `v` by value (kinds 9 / 0xa / 2), so the Rust value is stored as
/// is (a value of another tag would be re-tagged garbage in the original; unk: never seen in scripts).
pub fn ee_sub_132cc8(o: &mut ObjectFields, obj: u32, symid: u32, v: Ref) {
    // 0x00132cc8 / 0x00132d28 / 0x00132de0
    o.ee_sub_200d10(obj, symid, v);
}

/// FUN_00132d88(obj, symid, n): tag 0x15 S32 (same body as FUN_00132c20; arg kind 6).
pub fn ee_sub_132d88(o: &mut ObjectFields, obj: u32, symid: u32, n: i32) {
    // 0x00132d88
    o.ee_sub_200d10(obj, symid, Ref::S32(n));
}

/// FUN_00132e40(obj, symid): FUN_001294c0(ref, 0) = tag 0x21 Basic(null).
pub fn ee_sub_132e40(o: &mut ObjectFields, obj: u32, symid: u32) {
    // 0x00132e40
    o.ee_sub_200d10(obj, symid, Ref::Basic(0));
}

fn int(a: &[MArg], i: usize) -> i32 {
    match a.get(i) {
        Some(MArg::I32(n)) => *n,
        Some(MArg::Word(w)) => *w as i32,
        _ => 0,
    }
}

/// Run thunk `id` on `obj` with the FUN_00201110-unpacked args (arg 0 = symid, kind 0x15);
/// false = not one of `THUNKS`. The thunks return Void (FUN_001291a8 of a tag-0x24 CRef).
pub fn call(o: &mut ObjectFields, id: u32, obj: u32, a: &[MArg]) -> bool {
    let Some(&(_, callee)) = THUNKS.iter().find(|t| t.0 == id) else { return false };
    let sym = int(a, 0) as u32;
    let value = || match a.get(1) {
        Some(MArg::Value(r)) | Some(MArg::Ref(r)) => r.clone(),
        _ => Ref::Void,
    };
    match callee {
        0x0013_2bc8 => ee_sub_132bc8(o, obj, sym, int(a, 1) as u32),
        0x0013_2c20 => ee_sub_132c20(o, obj, sym, int(a, 1)),
        0x0013_2c78 => {
            let f = match a.get(1) {
                Some(MArg::F32(f)) => *f,
                _ => 0.0,
            };
            ee_sub_132c78(o, f, obj, sym)
        }
        0x0013_2cc8 | 0x0013_2d28 | 0x0013_2de0 => ee_sub_132cc8(o, obj, sym, value()),
        0x0013_2d88 => ee_sub_132d88(o, obj, sym, int(a, 1)),
        _ => ee_sub_132e40(o, obj, sym),
    }
    true
}
