//! WP-J6: engine functions behind the most-used script method rows that need no class runtime.
//!
//! - 0x00100a90 (field 25) / 0x00100b88 (field 26): inline thunks calling the object's vtable +0x44 /
//!   +0x4c (Add / Remove). Replaced: the LO base entries FUN_001a5478 / FUN_001a5630 (recursive over
//!   the children, `playerdraw::vis::JtVis`) are used for every class (class overrides belong to the
//!   class runtime, not ported here).
//! - 0x00100d18 / 0x0010cc70 -> FUN_001f6978 (show), 0x00100da0 / 0x0010cf18 -> FUN_001f69f8 (hide):
//!   mesh records (and instances) with that oid word (`JtVis::set_mesh`).
//! - 0x00106800 -> FUN_001e0020 / 0x00106888 -> FUN_001e0070: the 8-slot oid list at object +0x84.

use super::super::fields::ObjectFields;
use crate::engine::world::World;

/// FUN_001e0020(obj, oid): append unless present, while count < 8.
pub fn ee_sub_1e0020(objs: &mut ObjectFields, obj: u32, oid: i32) {
    // 0x001e0020
    let l = objs.oid_list.entry(obj).or_default();
    if l.contains(&oid) {
        return;
    }
    if l.len() < 8 {
        l.push(oid);
    }
}

/// FUN_001e0070(obj, oid): remove the first slot equal to oid (memmove of the tail, FUN_002332b8).
pub fn ee_sub_1e0070(objs: &mut ObjectFields, obj: u32, oid: i32) {
    // 0x001e0070
    if let Some(l) = objs.oid_list.get_mut(&obj) {
        if let Some(i) = l.iter().position(|&x| x == oid) {
            l.remove(i);
        }
    }
}

/// Executes one of the rows above; false = not one of them.
pub fn call(objs: &mut ObjectFields, w: Option<&World>, thunk: u32, obj: u32, int_arg: i32) -> bool {
    match thunk {
        0x0010_0a90 | 0x0010_0b88 | 0x0010_0d18 | 0x0010_cc70 | 0x0010_0da0 | 0x0010_cf18 => {
            let Some(w) = w else { return true };
            let o = obj as usize;
            if o >= w.objects.len() {
                return true;
            }
            match thunk {
                // vtable +0x44 -> FUN_001a5478
                0x0010_0a90 => {
                    objs.vis.ee_sub_1a5478(w, o);
                    objs.set_in_world(w, o, true);
                }
                // vtable +0x4c -> FUN_001a5630
                0x0010_0b88 => {
                    objs.vis.ee_sub_1a5630(w, o);
                    objs.set_in_world(w, o, false);
                }
                0x0010_0d18 | 0x0010_cc70 => objs.vis.set_mesh(w, o, int_arg, true),
                _ => objs.vis.set_mesh(w, o, int_arg, false),
            }
        }
        0x0010_6800 => ee_sub_1e0020(objs, obj, int_arg),
        0x0010_6888 => ee_sub_1e0070(objs, obj, int_arg),
        _ => return false,
    }
    true
}

/// Thunks `call` executes.
pub const THUNKS: &[u32] = &[0x0010_0a90, 0x0010_0b88, 0x0010_0d18, 0x0010_cc70, 0x0010_0da0, 0x0010_cf18, 0x0010_6800, 0x0010_6888];
